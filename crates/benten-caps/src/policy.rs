//! The [`CapabilityPolicy`] pre-write hook trait + [`CapWriteContext`] +
//! [`ReadContext`].
//!
//! The policy fires at commit time (not per-WRITE) so a multi-write
//! transaction is permitted or denied atomically. See
//! `tests/check_write_called_at_commit.rs` for the contract; the actual wiring
//! into the transaction primitive lands in G3.

use benten_core::Cid;

use crate::DEFAULT_BATCH_BOUNDARY;
use crate::error::CapError;

// =====================================================================
// G-CORE-9 V1-FROZEN-INTERFACE row 6 — sealed-discipline HARD-SEAL
// =====================================================================
//
// CLAUDE.md baked-in #7 sealed-discipline refinement (Ben-ratified
// 2026-05-18): `CapabilityPolicy` is **Benten-internal; NOT a
// documented third-party public extension contract.**
//
// **G-CORE-9 HARD-SEAL.** The sealed-discipline is enforced by `rustc`
// via a private [`sealed::Sealed`] supertrait. External crates CANNOT
// implement [`CapabilityPolicy`] because [`sealed::Sealed`] is
// unreachable from outside `benten-caps` (the `sealed` module is
// `pub(crate)`). At G-CORE-9 V1-FROZEN-INTERFACE wave the previous
// soft-seal marker `sealed_marker::SealedCapabilityPolicy` was DELETED
// (no deprecation alias per HARD RULE 12 + CLAUDE.md #5 no-shims
// discipline); the ~20 workspace test-double impls received
// `impl ext_seal::Sealed for X {}` siblings as part of the same
// migration (workspace test crates use the `pub(crate)`-visible
// [`crate::ext_seal::Sealed`] re-export which the workspace tests reach
// via the `testing` feature gate — see `crate::ext_seal`).
//
// Object-safety preserved: the [`sealed::Sealed`] supertrait carries
// NO methods, so `Arc<dyn CapabilityPolicy>` continues to construct
// unchanged. The compile-test pin at
// `crates/benten-engine/tests/g_core_8_capability_policy_sealed_compile_test.rs`
// is the structural backstop.
//
// V1-FROZEN-INTERFACE.md item 8 / §1.A.FROZEN item 8 freezes this
// shape as `pub trait CapabilityPolicy: sealed::Sealed + Send + Sync`.
//
// Workspace tests in other crates that need to implement
// `CapabilityPolicy` for test-doubles use the `#[doc(hidden)] pub`
// re-export at [`crate::__sealed_for_workspace_tests`] which is gated
// behind the `testing` feature — production downstream consumers do
// NOT enable the `testing` feature, so the hard-seal contract holds for
// non-`testing` builds. The `testing` feature is the explicit opt-in.
pub(crate) mod sealed {
    //! Private sealing module. The `pub(crate)` visibility means the
    //! [`Sealed`] trait is unreachable from outside `benten-caps` —
    //! external crates therefore cannot implement
    //! [`super::CapabilityPolicy`] (which has `Sealed` as a supertrait
    //! bound) UNLESS they enable the `testing` feature (workspace test
    //! crates only).

    /// Marker supertrait sealing [`super::CapabilityPolicy`]. Carries
    /// no methods (preserves object-safety). Benten-internal
    /// implementations of [`super::CapabilityPolicy`] MUST also impl
    /// this trait; the workspace-wide migration is part of G-CORE-9
    /// V1-FROZEN-INTERFACE row 6.
    pub trait Sealed {}
}

/// Re-export of [`benten_core::WriteAuthority`]. Single canonical type
/// across benten-core, benten-graph, and benten-caps.
pub use benten_core::WriteAuthority;

/// Context handed to the capability policy at commit time.
///
/// The field set is a union of the axes the R1 triage + R2 landscape named
/// across the various test writers. Fields are deliberately public so
/// downstream policy implementors can match on them without going through
/// accessors — the trait is meant to be easy to implement from a single
/// match expression.
///
/// `TODO(phase-3)`: `actor_hint` is a `String` placeholder for the eventual
/// DID / VC identity. Phase 3 `benten-id` replaces it with a typed principal.
///
/// A pending write enqueued inside the transaction primitive's batch.
/// G3-A landed the [`CapWriteContext::pending_ops`] surface (R4 pass-2 residual
/// g4-uc-5) so commit-time policies can reason about the whole batch — not
/// just the "primary" op reflected in the convenience fields.
///
/// The enum is deliberately lean — policies only need the label and CID of
/// each op to route denials. Richer shapes (full Node body, property diffs)
/// are a Phase-2 concern and would require `benten-caps` to take a direct
/// dep on `benten-graph` (a layering break).
///
/// `#[non_exhaustive]` (Phase-2a R6 wsa-3): Phase-2b adds a
/// `HostFunctionCall` variant (SANDBOX host-function manifest, plan §9.3),
/// which would otherwise be a breaking change for downstream policy
/// implementors. Sealing the variant set behind `non_exhaustive` makes
/// future additions a minor version bump and forces external `match`
/// expressions to include a `_ =>` arm — the same forward-compat
/// discipline `ErrorCode` and `GraphError` already enforce.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PendingOp {
    /// A Node write. `labels` is the full label set of the Node being put.
    PutNode {
        /// The content-addressed CID of the Node after encoding.
        cid: Cid,
        /// Every label the Node carries.
        labels: Vec<String>,
    },
    /// An Edge write. `label` is the Edge's single label.
    PutEdge {
        /// The content-addressed CID of the Edge after encoding.
        cid: Cid,
        /// The Edge's label.
        label: String,
    },
    /// A Node deletion by CID.
    ///
    /// `labels` is the label set captured at delete time via read-before-
    /// delete (see `benten_graph::Transaction::delete_node`). The engine
    /// threads the captured labels into this variant so the capability
    /// policy can derive the same `store:<label>:write` scope it uses for
    /// the PutNode side. An empty `labels` means the delete targeted an
    /// already-absent CID (idempotent miss); the policy treats that as a
    /// no-op scope with no grant required. See r6-sec-8.
    DeleteNode {
        /// The target Node CID.
        cid: Cid,
        /// Labels of the Node being deleted (captured via read-before-
        /// delete). Empty on idempotent miss.
        labels: Vec<String>,
    },
    /// An Edge deletion by CID.
    ///
    /// `label` is the Edge's single label captured at delete time.
    /// `None` means the delete targeted an already-absent CID (idempotent
    /// miss); the policy treats that as a no-op scope. See r6-sec-8.
    DeleteEdge {
        /// The target Edge CID.
        cid: Cid,
        /// Label of the Edge being deleted (captured via read-before-
        /// delete). `None` on idempotent miss.
        label: Option<String>,
    },
}

/// Context passed to [`CapabilityPolicy::check_write`].
///
/// Carries the pending-ops batch, the actor identity (Phase-3), and a
/// privileged-flag for engine-internal writes. Backends inspect these to
/// decide whether to authorize the transaction.
///
/// `#[non_exhaustive]` application DEFERRED to G-COMP-1 per
/// V1-FROZEN-INTERFACE-DEFERRED.md Row D-17 (G-CORE-9 R1 fix-pass): the
/// attribute application cascades through ~50+ workspace test sites
/// using struct-literal construction with `..Default::default()` (which
/// is blocked from outside the defining crate); the cascade is genuinely
/// large + the migration to `Default::default()` + field-mutation
/// pattern is the right shape for G-COMP-1 to apply atomically. The
/// signature shape is locked at v1-beta; the attribute is the missing
/// piece per V1-FROZEN-INTERFACE.md item 11 — adding it post-v1-beta
/// IS breaking and the v1-beta engineering MUST treat field additions
/// as breaking until Row D-17 closes.
#[derive(Debug, Clone, Default)]
pub struct CapWriteContext {
    /// Label of the Node about to be written. For multi-op batches this
    /// carries the primary label of the first op (convenience field;
    /// structured routing should use [`CapWriteContext::pending_ops`]).
    pub label: String,
    /// Actor CID identity (Phase 3). `None` in Phase 1; reserved so the
    /// struct shape is stable across phases.
    pub actor_cid: Option<Cid>,
    /// The capability scope the operation targets
    /// (e.g. `"store:post:write"`).
    pub scope: String,
    /// True if the caller is engine-privileged (system-zone writes arrive via
    /// the engine API only; user subgraphs never set this).
    pub is_privileged: bool,
    /// Non-Cid actor hint (a string identifier) used in test fixtures and
    /// Phase-1 in-process policies.
    pub actor_hint: Option<String>,
    /// Full pending-writes batch the transaction will commit atomically.
    /// Empty for check paths outside a transaction; G3-A populates this
    /// from the transaction primitive's pending-ops list at commit time.
    ///
    /// Closes R4 pass-2 residual `g4-uc-5`: policies can now inspect the
    /// full batch rather than just the primary op reflected by `label`.
    pub pending_ops: Vec<PendingOp>,
    /// Phase 2a G2-B / ucca-9 / arch-r1-2: authority under which the write
    /// runs. Defaults to [`WriteAuthority::User`].
    pub authority: WriteAuthority,
    /// Phase-3 G16-B canary (r4b-cap-3 BLOCKER closure): device-grain
    /// CID context per D-PHASE-3-25 device-heterogeneity contract.
    ///
    /// `None` for non-attested writes (legacy / local). `Some(cid)` for
    /// attested writes — the CID of the device's signed attestation
    /// envelope at the boundary the engine consults
    /// (cf. `benten_id::device_attestation::DeviceAttestation`).
    /// Heterogeneous policies (per D-PHASE-3-25) dispatch on this field
    /// to surface different decisions for "desktop X writes" vs
    /// "phone X writes" under the SAME logical actor identity.
    ///
    /// Backward-compat: pre-G16-B callers leave the field `None` via
    /// `Default`; existing policies that ignore the field continue to
    /// work unchanged. New heterogeneous policies opt-in via match.
    ///
    /// Engine-side production-runtime threading lands in the post-canary
    /// wave (the engine write-path call sites populate the field at
    /// `CapWriteContext`-construction time per the
    /// `crates/benten-engine/tests/device_cid_runtime_arm.rs::capability_policy_per_device_cid_dispatch_observable_in_runtime_arm`
    /// pin's concrete-shape narrative).
    pub device_cid: Option<Cid>,
    /// **Phase-4-Meta-Core G-CORE-8 §8-E audience-aware enrichment.**
    ///
    /// The audience-DID context for the cap check. Set by the engine
    /// at WRITE-admission time when the request carries an audience
    /// (e.g. a plugin-DID at the delegate boundary, a thin-client
    /// session-bound principal-DID at the bridge boundary). `None`
    /// for legacy writes or writes with no audience-binding.
    ///
    /// Used by the audience-aware [`CapabilityPolicy::check_write_with_audience`]
    /// hook (the §8-E new audience-aware hook). The default
    /// `check_write` impl ignores this field for backward-compat;
    /// audience-aware impls match on `Some(audience_did)` to apply
    /// audience-scoped attenuation.
    pub audience_did: Option<String>,
}

impl CapWriteContext {
    /// Construct a lightweight synthetic context for unit tests. Fields are
    /// stable-but-synthetic placeholders so the unit-test surface does not
    /// depend on the evaluator being wired in.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn synthetic_for_test() -> Self {
        Self {
            label: "synthetic".into(),
            actor_cid: None,
            scope: "synthetic:write".into(),
            is_privileged: false,
            actor_hint: Some("synthetic-actor".into()),
            pending_ops: Vec::new(),
            authority: WriteAuthority::User,
            device_cid: None,
            audience_did: None,
        }
    }
}

/// Context handed to the capability policy at read time.
///
/// Phase 1 ships the shape so named compromise #2 has a concrete anchor on
/// [`CapabilityPolicy::check_read`]; the default policy permits every read.
/// Phase 3 `benten-id` swaps in a typed principal and wires real read-grant
/// enforcement.
///
/// See `docs/ERROR-CATALOG.md` for [`crate::CapError::DeniedRead`].
///
/// `#[non_exhaustive]` application DEFERRED to G-COMP-1 per
/// V1-FROZEN-INTERFACE-DEFERRED.md Row D-17 (same rationale as
/// CapWriteContext above): cascades through ~30+ workspace test sites.
#[derive(Debug, Clone, Default)]
pub struct ReadContext {
    /// Label of the Node (or view / anchor) the caller is trying to read.
    pub label: String,
    /// CID of the target entity when known (None if the caller is reading
    /// by label / query).
    pub target_cid: Option<Cid>,
    /// Non-Cid actor hint used by Phase-1 in-process test policies.
    pub actor_hint: Option<String>,
    /// Actor CID identity (Phase 3). Reserved.
    pub actor_cid: Option<Cid>,
    /// Phase-3 G16-B canary (r4b-cap-3 BLOCKER closure): device-grain
    /// CID context paired with [`CapWriteContext::device_cid`]. `None` for
    /// non-attested reads; `Some(cid)` for reads originating from a
    /// device that presented a `DeviceAttestation` envelope at the
    /// thin-client / sync seam. Per D-PHASE-3-25, heterogeneous policies
    /// dispatch on this field for per-device READ scoping.
    pub device_cid: Option<Cid>,
    /// **Phase-4-Meta-Core G-CORE-8 §8-E audience-aware enrichment.**
    ///
    /// The audience-DID context for the cap check. Paired with
    /// [`CapWriteContext::audience_did`]; see that field's docs for
    /// the audience-aware hook contract. `None` for legacy reads or
    /// reads with no audience-binding.
    pub audience_did: Option<String>,
}

impl ReadContext {
    /// Construct a lightweight synthetic context for unit tests.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn synthetic_for_test() -> Self {
        Self {
            label: "synthetic".into(),
            target_cid: None,
            actor_hint: Some("synthetic-actor".into()),
            actor_cid: None,
            device_cid: None,
            audience_did: None,
        }
    }

    /// Construct a `ReadContext` for a CID-only read (no label in scope).
    ///
    /// G11-A EVAL wave-1 (G4-A nit): the "empty-label means CID-only"
    /// convention previously lived as an unwritten rule in
    /// `benten-engine/src/primitive_host.rs` and a few engine
    /// call-sites that constructed `ReadContext { label: String::new(),
    /// target_cid: Some(cid), ..Default::default() }` inline. A typed
    /// constructor makes the convention explicit and gives
    /// `CapabilityPolicy::check_read` implementations a single pattern
    /// to match on.
    #[must_use]
    pub fn by_cid_only(cid: Cid) -> Self {
        Self {
            label: String::new(),
            target_cid: Some(cid),
            actor_hint: None,
            actor_cid: None,
            device_cid: None,
            audience_did: None,
        }
    }

    /// Construct a `ReadContext` for a label-scoped read with no target
    /// CID in hand (e.g. `get_by_label`, `get_by_property`,
    /// `read_view` — the caller is probing a label-filtered index, not
    /// a specific known CID).
    ///
    /// G11-A Wave-2a carry (EVAL Wave-1 M2 follow-up): the paired
    /// constructor to [`ReadContext::by_cid_only`]. Together the two
    /// typed constructors let every engine-side `check_read_capability`
    /// call site describe its read-shape without relying on the
    /// unwritten "empty label means CID-only" convention.
    #[must_use]
    pub fn by_label_only(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            target_cid: None,
            actor_hint: None,
            actor_cid: None,
            device_cid: None,
            audience_did: None,
        }
    }
}

/// The capability pre-write hook trait.
///
/// Called by the transaction primitive at commit time — not per-WRITE. A
/// multi-write subgraph is either permitted atomically or denied atomically.
///
/// Object-safe: integration tests routinely box this behind `dyn
/// CapabilityPolicy`. Keep any future extensions to this trait object-safe
/// (no `where Self: Sized` defaults that take `self` by value, no generic
/// methods without `where Self: Sized`).
pub trait CapabilityPolicy: sealed::Sealed + Send + Sync {
    /// Permit or deny the pending write batch.
    ///
    /// # Errors
    ///
    /// Implementations return [`CapError`] for any denial or backend failure.
    /// The default [`crate::NoAuthBackend`] always returns `Ok(())`.
    fn check_write(&self, ctx: &CapWriteContext) -> Result<(), CapError>;

    /// Permit or deny an incoming read.
    ///
    /// # Named compromise #2 — `E_CAP_DENIED_READ` leaks existence
    ///
    /// A backend that chooses to DENY a read returns
    /// [`CapError::DeniedRead`], which surfaces "this CID exists but you
    /// cannot see it" — leaking existence to an unauthorized caller. The
    /// identity surface this was waiting on HAS shipped:
    /// `Engine::read_node_as` (Class B β, PR #184) + the Phase-4-Foundation
    /// R1 cap-r1-2 principal-aware read path on
    /// [`crate::GrantBackedPolicy`]. The named-compromise-#2 disposition
    /// (silent-`None` vs `DeniedRead`) post-identity-surface is a
    /// v1-API-stabilization decision tracked at
    /// `docs/future/phase-4-backlog.md` §4.43 (issue #887).
    ///
    /// **Default: PERMIT EVERY READ.** This is correct ONLY for
    /// permit-all dev/embedded policies (e.g. [`crate::NoAuthBackend`]).
    ///
    /// # ⚠️ WARNING for production policy authors
    ///
    /// Reads are a cap-bearing surface. If you implement a non-trivial
    /// [`CapabilityPolicy::check_write`] but DO NOT override `check_read`,
    /// every read silently fail-OPENs through your policy — the engine's
    /// identity surface (`Engine::read_node_as` + cap-r1-2 principal-aware
    /// path on [`crate::GrantBackedPolicy`]) IS shipped, so a forgotten
    /// `check_read` override is now a real authorization gap, not a
    /// Phase-1 placeholder. Any policy that gates writes by principal
    /// MUST also gate reads (or explicitly opt into permit-all reads with
    /// a comment documenting the intent). [`crate::GrantBackedPolicy`] +
    /// [`crate::UcanGroundedPolicy`] override this correctly; use one of
    /// them as the reference shape.
    ///
    /// # Errors
    ///
    /// Return [`CapError::DeniedRead`] to deny. Other variants
    /// ([`CapError::Revoked`], [`CapError::NotImplemented`]) route through
    /// `ON_ERROR` per the evaluator contract in
    /// `tests/ucan_stub_messages.rs`.
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }

    /// Maximum number of ITERATE loop bodies that may execute between
    /// capability-snapshot refreshes, as recommended by this policy.
    ///
    /// The default is [`DEFAULT_BATCH_BOUNDARY`] — the Phase 1 named
    /// compromise #1 boundary. A revocation-sensitive backend (Phase 3
    /// UCAN with a short TTL; a testing backend that wants to force a
    /// refresh every iteration) can override to tighten the bound.
    ///
    /// # Phase-1 wiring caveat
    ///
    /// The evaluator reads its batch cadence from
    /// `benten_eval::PrimitiveHost::iterate_batch_boundary` —
    /// the engine's default `PrimitiveHost` implementation now
    /// delegates to this policy method via
    /// `benten_caps::evaluator_delegation::iterate_batch_boundary_for`
    /// (consumer wired at
    /// `crates/benten-engine/src/primitive_host.rs::PrimitiveHost::iterate_batch_boundary`),
    /// so customising this value on a bespoke `CapabilityPolicy`
    /// affects the evaluator's actual refresh cadence end-to-end.
    ///
    /// Lowering this bound increases capability-check load; raising it
    /// widens the TOCTOU window. Keep in lockstep with the named compromise
    /// prose in `.addl/phase-1/r1-triage.md` if the default is ever
    /// adjusted.
    //
    // The companion wall-clock TOCTOU ceiling (per R4b compromise #1
    // tightening, auditor finding g4-p2-uc-2 — a TRANSFORM-heavy or
    // CALL-heavy handler at 1 iter/10sec pushes past 10 minutes between
    // refreshes under iteration-count alone; the first real capability
    // backend MUST additionally enforce a wall-clock ceiling
    // `min(iteration_count, wall_clock_seconds)`, default ≤300s) is
    // exposed via `wallclock_refresh_ceiling` below; the evaluator
    // consumer for THAT half is registered at
    // `phase-3-backlog §2.3 (ii)` (v1-assessment-window co-routed with
    // §10.1 Compromise #1 TOCTOU window bound; shared
    // iterate-batch-boundary cap-recheck cadence mechanism).
    fn iterate_batch_boundary(&self) -> usize {
        DEFAULT_BATCH_BOUNDARY
    }

    /// Phase 2a G9-A / P1 / §9.13 refresh-point-5: maximum wall-clock
    /// duration between capability-grant revalidations during a long-running
    /// ITERATE or CALL. Default 300s per the dual-source resolution; the
    /// evaluator's monotonic source drives the cadence, the HLC rides
    /// alongside for federation correlation.
    ///
    /// TODO(phase-3 — wallclock-refresh-ceiling evaluator wire-up):
    /// wire into the evaluator's refresh path so the override becomes
    /// load-bearing end-to-end. Carried from Phase-2a G9-A (didn't
    /// land); registered at `phase-3-backlog §2.3 (ii)` (v1-assessment-
    /// window co-routed with §10.1 Compromise #1 TOCTOU window bound;
    /// shared iterate-batch-boundary cap-recheck cadence mechanism).
    /// (Qual-1 #674 / umbrella #1154: the
    /// `evaluator_delegation::wallclock_refresh_ceiling_for` zero-
    /// consumer wrapper that previously fronted this method was
    /// deleted; callers consult this trait method directly once the
    /// §2.3 (ii) `PrimitiveHost` consumer impl lands.)
    fn wallclock_refresh_ceiling(&self) -> core::time::Duration {
        core::time::Duration::from_mins(5)
    }

    // =================================================================
    // Phase-4-Meta-Core G-CORE-8 §8-E — three new defaulted hooks
    // (install-time consent / per-delegation runtime / audience-aware
    // check_write enrichment). All three are ADDITIVE defaulted methods
    // per constraint (g): existing impls + `Arc<dyn CapabilityPolicy>`
    // boxing compile unchanged.
    // =================================================================

    /// **G-CORE-8 §8-E hook #1 — install-time consent.**
    ///
    /// Called at plugin install admission BEFORE the cap-cascade runs
    /// to give the policy a chance to refuse the install based on the
    /// install record's content (e.g. a policy that wants user
    /// re-consent for caps the previous version didn't request).
    ///
    /// The default impl returns `Ok(())` — admit-all-installs;
    /// existing impls do NOT need to change. Audience-aware impls
    /// override to apply install-time policy (e.g. matching against a
    /// curated trust-list of plugin-DIDs).
    ///
    /// # The #887b decision (`check_read` default-impl policy)
    ///
    /// Per constraint (g) the wave decides #887b inline: the
    /// `check_install_consent` default returns `Ok(())` (admit-all-
    /// installs) for the SAME reason `check_read` defaults to admit
    /// (per the existing `check_read` doc): permit-all is correct for
    /// permit-all dev/embedded policies (e.g. [`crate::NoAuthBackend`])
    /// AND for policies that want to defer install enforcement to a
    /// separate install-pipeline layer (the existing pattern at
    /// `benten_platform_foundation::plugin_lifecycle::install_plugin`
    /// where install enforcement is procedural, not policy-routed).
    /// The CONTRAST with `check_write` (which has no default — every
    /// impl MUST implement it) is intentional: writes are the
    /// mandatory cap boundary; install is the SOFT cap boundary
    /// (institutional install-pipeline enforces the hard guarantees;
    /// the policy hook is the customization seam). Documented in
    /// INTERNALS.md §9 + carries forward to the v1-API freeze at
    /// G-CORE-9.
    ///
    /// # Errors
    ///
    /// [`CapError`] to deny install. The default returns `Ok(())`.
    fn check_install_consent(
        &self,
        _install_record_signing_payload_hash: &[u8; 32],
        _plugin_did: &str,
    ) -> Result<(), CapError> {
        Ok(())
    }

    /// **G-CORE-8 §8-E hook #2 — per-delegation runtime check.**
    ///
    /// Called at the cross-plugin delegation runtime boundary (when
    /// plugin A delegates a capability to plugin B at request time).
    /// Distinct from the install-time consent above: this fires
    /// per-request, not per-install.
    ///
    /// The default impl returns `Ok(())` — admit-all-delegations;
    /// existing impls do NOT need to change. Audience-aware impls
    /// override to apply per-delegation policy (e.g. rate-limiting,
    /// time-bounded delegation, audit-trail emission).
    ///
    /// # Errors
    ///
    /// [`CapError`] to deny the runtime delegation. The default
    /// returns `Ok(())`.
    fn check_per_delegation(
        &self,
        _source_plugin_did: &str,
        _target_plugin_did: &str,
        _cap_scope: &str,
    ) -> Result<(), CapError> {
        Ok(())
    }

    /// **G-CORE-8 §8-E hook #3 — audience-aware check_write.**
    ///
    /// Called by the engine's WRITE admission path when the write
    /// carries an audience-binding (e.g. a thin-client session-bound
    /// principal-DID; a plugin-DID at the delegate boundary). The
    /// audience is in [`CapWriteContext::audience_did`].
    ///
    /// The default impl delegates to [`Self::check_write`] (ignoring
    /// audience) so existing impls do NOT need to change. Audience-
    /// aware impls override to apply audience-scoped attenuation
    /// (e.g. denying a plugin-DID's writes that exceed its install-
    /// time manifest envelope, even when the user-DID's check_write
    /// would admit).
    ///
    /// # The (h) constraint (no hard-coded grant-first/chain-second
    /// composition)
    ///
    /// Per constraint (h), the audience-aware hook MUST NOT presume
    /// or entangle the hard-coded grant-first / chain-second
    /// `cap:typed:*` composition the existing `GrantBackedPolicy`
    /// uses. The hook's contract is: "apply audience-scoped policy
    /// given the audience-DID present in ctx" — the implementer is
    /// free to compose grants + chain + envelope in whatever order
    /// makes sense for that policy. The default's delegation to
    /// `check_write` preserves the existing composition for legacy
    /// callers; audience-aware impls compose freshly.
    ///
    /// # Errors
    ///
    /// [`CapError`] to deny. The default delegates to `check_write`.
    fn check_write_with_audience(&self, ctx: &CapWriteContext) -> Result<(), CapError> {
        self.check_write(ctx)
    }
}
