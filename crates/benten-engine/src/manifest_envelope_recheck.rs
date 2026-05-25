//! Phase-4-Foundation R4b-FP-1 Seam 3 — manifest-envelope recheck
//! port for `Engine::apply_atrium_merge`'s defense-in-depth row loop.
//!
//! **What this seam does**
//!
//! Phase-3 G16-B-F PR #161 (sec-r4r1-2 BLOCKER closure) wired
//! structural-always-on per-row cap-recheck inside `apply_atrium_merge`.
//! Phase-4-Foundation EXTENDS that recheck path to additionally call
//! the manifest-envelope chain validator — Layer 3 of CLAUDE.md #18
//! trust model. Defense narrative (T8 + R2 §5 Gap fix #4):
//!
//! - A hostile peer constructs a UCAN chain whose every step verifies
//!   cryptographically + appears in the sender's local grant store.
//! - The chain leaks past the sender's `CapabilityPolicy::check_write`
//!   (which doesn't enforce manifest-envelope semantics).
//! - On the RECEIVING peer, the per-row recheck at
//!   `apply_atrium_merge` consults THIS port; the rechecker walks the
//!   chain through `benten_caps::manifest_envelope_chain_validation::
//!   validate_chain_with_manifest_envelope` against the local
//!   manifest store. If any step is OUTSIDE the envelope, the row is
//!   rejected before the merge Version Node is minted.
//!
//! **Dep-direction discipline**
//!
//! The port lives in `benten-engine` (the trust-boundary owner per
//! `benten-sync` INTERNALS §"Structural-always-on per-row cap-
//! recheck"). The concrete implementation lives in the engine adapter
//! crate that owns the plugin library — typically
//! `benten-platform-foundation` (via a thin glue type that wraps
//! `PluginLibrary` + `benten-caps::manifest_envelope_chain_validation`).
//! The engine consumes the port; the foundation crate provides it.
//! Engine code path here NEVER imports `benten-platform-foundation`.

use crate::EngineError;
use benten_errors::ErrorCode;

/// **R6 R1 FP-F4 §S4 (Row D-18 closure)** — whether `did_str` is the
/// synthesized-fallback DID shape `node-id:NNN` produced by
/// `Engine::resolve_peer_dids` when a peer_node_id is absent from
/// the local trust-store registry.
///
/// Defense narrative: per L2-MAJ-1, an adversarial peer that
/// presents an unmapped peer_node_id surfaces as a synthesized DID
/// (`node-id:N`); a substantive rechecker has no manifest registered
/// under that synthetic key + would admit-by-default if not hardened.
/// This helper is the substrate-level forensic anchor; production
/// rechecker impls (e.g. `ProductionManifestEnvelopeRechecker`)
/// SHOULD consult this to map synthesized DIDs to
/// `UnresolvedDeny` arm (substantive-rechecker-installed detection +
/// over-fire-protection per Row D-18's
/// substantive-rechecker-detection-couple-not-naive-blanket clause).
#[must_use]
pub fn is_synthesized_node_id(did_str: &str) -> bool {
    did_str.starts_with("node-id:")
}

/// Outcome of a manifest-envelope recheck call.
///
/// # G-CORE-8 §4.36 fail-CLOSED flip + (a-sub) typed-arm split
///
/// Phase-4-Meta-Core G-CORE-8 closes the **security-r1-1 + security-r1-2
/// BLOCKERs** by ADDING a typed [`Self::UnresolvedDeny`] arm
/// (distinct from the legitimate [`Self::NotApplicable`] arm) and
/// flipping the [`outcome_to_row_reject`] mapping so the new arm
/// row-rejects with typed
/// [`ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny`].
///
/// **The pre-G-CORE-8 BLOCKER:** the rechecker had no `UnresolvedDeny`
/// typed-arm — a substantive rechecker that detected an unresolvable
/// peer-DID or a sentinel `<unresolved-peer>` had no way to express
/// "I tried but couldn't positively place this row" except by
/// admitting (`NotApplicable`) or fabricating an `OutsideEnvelope`
/// error with synthesized fields. The result: an attacker who induced
/// peer-DID resolution failure could route an inbound row through the
/// admit path. G-CORE-8 fixes this by minting `UnresolvedDeny` + the
/// typed reject so substantive rechecker implementations can
/// fail-CLOSED honestly on unresolvable cases.
///
/// **The post-flip enum invariant (security-r1-2):** `Admitted` and
/// `NotApplicable` are the proceed paths; `UnresolvedDeny` and
/// `OutsideEnvelope` are the reject paths. The semantic split:
///
/// - `NotApplicable` = "no plugin chain present in this row — Layer-3
///   envelope semantics don't apply here; the Layer-1 user-root +
///   per-row cap-recheck checks (enforced elsewhere) are the relevant
///   defenses." This is the legitimate "user wrote it directly" case
///   AND the legitimate "no rechecker installed → nothing to check"
///   case (the Noop returns this).
/// - `UnresolvedDeny` = "I expected to verify a plugin chain BUT
///   couldn't (peer-DID unresolvable, sentinel `<unresolved-peer>`,
///   no installed manifest, ambiguous resolution)." Substantive
///   `ProductionManifestEnvelopeRechecker` impls return this on the
///   failure paths; row-rejects fail-CLOSED.
/// - `Admitted` = "plugin chain positively verified against the
///   manifest envelope" — proceed.
/// - `OutsideEnvelope` = "plugin chain present BUT a step is outside
///   the source plugin's `shares` policy" — row-rejects with the
///   pre-existing typed code.
///
/// `#[non_exhaustive]` per V1-FROZEN-INTERFACE.md item 11 + item 12 —
/// a future fifth variant would break every consumer's `match`; the
/// attribute makes the variant-set additively extensible.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEnvelopeRecheckOutcome {
    /// No manifest-envelope chain is in scope for this row (e.g.
    /// user-issued write not delegated through any plugin; or the
    /// installed rechecker has no PluginLibrary state to consult, the
    /// Noop case). Layer-3 envelope semantics do not apply. Layer-1
    /// user-root + per-row cap-recheck checks (enforced elsewhere)
    /// remain the relevant defenses.
    ///
    /// **Note** — this is distinct from [`Self::UnresolvedDeny`]: the
    /// former says "no chain here, nothing to check at this layer";
    /// the latter says "chain expected, but the rechecker could not
    /// positively place it." Substantive `Production*Rechecker` impls
    /// MUST return `UnresolvedDeny` on the failure path (NEVER
    /// `NotApplicable`), so peer-DID resolution failure cannot route
    /// through the admit path.
    NotApplicable,
    /// **G-CORE-8 §4.36 fail-CLOSED arm (security-r1-1 + security-r1-2
    /// BLOCKER closures; new variant).**
    ///
    /// The rechecker EXPECTED to verify a plugin chain for this row
    /// but could NOT positively place it against the local manifest
    /// store + UserDidRegistry. Causes: unresolvable peer-DID
    /// (sentinel `<unresolved-peer>`); no installed manifest for the
    /// inbound row's source plugin-DID; ambiguous-resolution outcome
    /// where the rechecker cannot produce a positive `Admitted`
    /// verdict.
    ///
    /// **Invariant (security-r1-2):** this outcome MUST row-reject via
    /// [`outcome_to_row_reject`] (typed
    /// [`ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny`]). NEVER
    /// admit on this arm. The pre-G-CORE-8 BLOCKER was the *absence*
    /// of this typed arm — substantive rechecker impls had to either
    /// admit-on-unresolved or fabricate fields. Production impls now
    /// return this arm honestly + fail-CLOSED at the merge boundary.
    UnresolvedDeny,
    /// A manifest-envelope chain was found AND every step fits the
    /// source plugin's `shares` policy. Admit the row.
    ///
    /// **Invariant (security-r1-2):** this is the **positive proceed
    /// path**; returned ONLY on a positively-verified envelope/chain
    /// match (a substantive impl that returns `Admitted` is asserting
    /// it walked the chain + every step satisfied the `shares` policy).
    Admitted,
    /// A manifest-envelope chain was found BUT a step is outside the
    /// envelope (Layer 2 ↔ Layer 3 mismatch). Reject the row with the
    /// typed [`ErrorCode::PluginDelegationOutsideManifestEnvelope`].
    OutsideEnvelope {
        /// The plugin-DID whose envelope was violated.
        offending_plugin_did: String,
        /// The cap pattern that exceeded the envelope.
        cap_pattern: String,
    },
}

/// Port the foundation adapter implements to drive manifest-envelope
/// recheck from inside `Engine::apply_atrium_merge`'s per-row loop.
///
/// Implementations are typically a thin glue over
/// `benten_platform_foundation::plugin_library::PluginLibrary` (the
/// `ManifestEnvelopeLookup`) + a `UserDidRegistry` over the engine's
/// install-record store + `benten_caps::manifest_envelope_chain_validation::
/// validate_chain_with_manifest_envelope`.
///
/// The default implementation
/// [`NoopManifestEnvelopeRechecker`] returns `NotApplicable` for every
/// call — equivalent to the Phase-3-shipped behavior. Engines built
/// WITHOUT a configured rechecker behave exactly as before.
pub trait ManifestEnvelopeRechecker: Send + Sync {
    /// Recheck a single per-row write originating from a remote peer.
    ///
    /// `peer_did_str` is the resolved originating peer-DID (the engine
    /// has already resolved it from `seed.peer_node_ids`).
    /// `zone` is the merge zone. `key` is the row key inside the
    /// Loro op-log (engine includes this for diagnostic correlation).
    ///
    /// Implementations CONSULT the install-record / plugin library
    /// state to determine whether the originating peer's write was
    /// delegated through a plugin chain; if yes, walk the chain
    /// through the manifest-envelope chain validator + return the
    /// matching outcome variant.
    fn recheck_row(
        &self,
        peer_did_str: &str,
        zone: &str,
        key: &str,
    ) -> ManifestEnvelopeRecheckOutcome;
}

/// Default rechecker — returns
/// [`ManifestEnvelopeRecheckOutcome::NotApplicable`] for every call.
///
/// **Semantics:** "no PluginLibrary state to consult → no chain in
/// scope at this layer." Equivalent to the Phase-3 baseline: Layer-3
/// envelope semantics don't apply, Layer-1 user-root + per-row
/// cap-recheck (enforced elsewhere) remain in force.
///
/// **G-CORE-8 §4.36 note:** the §4.36 fail-CLOSED flip is the addition
/// of the typed [`ManifestEnvelopeRecheckOutcome::UnresolvedDeny`]
/// arm (NOT a change to this Noop's outcome). Substantive
/// `Production*Rechecker` impls — which actually have PluginLibrary
/// state — return `UnresolvedDeny` on the unresolvable-peer / missing-
/// manifest paths AND row-reject via [`outcome_to_row_reject`] with
/// typed [`ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny`]. The
/// Noop continues to return `NotApplicable` because it has no
/// PluginLibrary state by definition — it is the "no Layer-3 enforcement
/// installed" default and the typed reject would over-fire on every
/// inbound row (breaking every existing sync test that doesn't wire a
/// real PluginLibrary).
///
/// **Production-wiring contract:** deployments that install a real
/// PluginLibrary swap in a `ProductionManifestEnvelopeRechecker`
/// (typically the platform-foundation glue type that wraps
/// `PluginLibrary` + `validate_chain_with_manifest_envelope`) via
/// [`crate::Engine::set_manifest_envelope_rechecker`]. The Noop is the
/// default for deployments that have no Layer-3 trust model installed.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopManifestEnvelopeRechecker;

impl ManifestEnvelopeRechecker for NoopManifestEnvelopeRechecker {
    fn recheck_row(
        &self,
        _peer_did_str: &str,
        _zone: &str,
        _key: &str,
    ) -> ManifestEnvelopeRecheckOutcome {
        // The Noop has no PluginLibrary state to consult; by
        // construction it cannot answer "is this chain inside the
        // source plugin's manifest envelope?" — there's no manifest
        // store to look in. The right answer at this layer is
        // NotApplicable ("no Layer-3 enforcement installed; defer to
        // Layer-1 user-root + per-row cap-recheck enforced elsewhere").
        //
        // Production deployments with a PluginLibrary install a real
        // ProductionManifestEnvelopeRechecker that returns
        // UnresolvedDeny on the unresolvable-peer path → row-rejects
        // (G-CORE-8 §4.36 fail-CLOSED via the new typed arm).
        ManifestEnvelopeRecheckOutcome::NotApplicable
    }
}

/// Helper used inside [`crate::Engine::apply_atrium_merge`]'s per-row
/// loop — converts an `OutsideEnvelope` outcome into the typed engine
/// error.
///
/// Exposed as `pub` (R6-FP-A-fp mr-7) so test pins can exercise the
/// recheck-outcome → row-reject mapping without spinning up a full
/// Engine + iroh + Atrium harness. The end-to-end wire-up at
/// `apply_atrium_merge` is the same code path; this helper IS the
/// boundary the per-row loop calls.
pub fn outcome_to_row_reject(
    outcome: ManifestEnvelopeRecheckOutcome,
    zone: &str,
    key: &str,
) -> Result<(), EngineError> {
    match outcome {
        // Proceed paths: `Admitted` (positive verification) +
        // `NotApplicable` (no chain in scope at this layer — Layer-1
        // user-root + per-row cap-recheck enforced elsewhere are the
        // relevant defenses).
        ManifestEnvelopeRecheckOutcome::Admitted
        | ManifestEnvelopeRecheckOutcome::NotApplicable => Ok(()),
        // **G-CORE-8 §4.36 fail-CLOSED flip (security-r1-1 + security-r1-2
        // BLOCKER closures):** the new `UnresolvedDeny` typed arm
        // row-rejects with the typed ErrorCode. Substantive
        // `Production*Rechecker` impls return this on the unresolvable-
        // peer / missing-manifest / ambiguous-resolution paths so
        // peer-DID resolution failure CANNOT route through the admit
        // path. The pre-G-CORE-8 BLOCKER was the absence of this typed
        // arm — substantive impls had to admit-on-unresolved or fabricate
        // OutsideEnvelope fields.
        ManifestEnvelopeRecheckOutcome::UnresolvedDeny => Err(EngineError::Other {
            code: ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny,
            message: format!(
                "apply_atrium_merge: manifest-envelope recheck cannot positively place row \
                 (zone='{zone}' key='{key}'): unresolvable peer-DID, sentinel \
                 '<unresolved-peer>', missing installed manifest, or ambiguous resolution \
                 — fail-CLOSED per G-CORE-8 §4.36 (security-r1-1 + security-r1-2)"
            ),
        }),
        ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
            offending_plugin_did,
            cap_pattern,
        } => Err(EngineError::Other {
            code: ErrorCode::PluginDelegationOutsideManifestEnvelope,
            message: format!(
                "apply_atrium_merge: manifest-envelope recheck rejected row \
                 (zone='{zone}' key='{key}' offending_plugin_did='{offending_plugin_did}' \
                 cap_pattern='{cap_pattern}'); T8 defense-in-depth at sync merge boundary"
            ),
        }),
    }
}

/// COLLAPSE (P3) — the **single** J8 envelope-ceiling AND, applied
/// inside [`crate::Engine::apply_atrium_merge`]'s per-row recheck.
///
/// DECISION-RECORD §4 RATIFIED: the device-attestation pipe is no
/// longer a distinct trust-root. A signed device
/// [`benten_id::device_attestation::CapabilityEnvelope`] declares a
/// *ceiling* on what the inbound writer's effective caps may include.
/// The unified chain-validation seam ANDs that ceiling into the
/// writer's effective authority. This function IS that AND for the
/// device-envelope shape.
///
/// **One code path, unified with #669 (build-constraint iii).** The
/// `runs_sandbox=false → reject host:sandbox:*` rule is the
/// load-bearing CLAUDE.md #17 thin-shape property. It lived in the
/// (un-wired) `benten_id::ucan::validate_chain_with_attestations`
/// before COLLAPSE; the deleted `Acceptor::accept_at` never enforced
/// it on inbound sync. COLLAPSE P2 (CONSOLIDATE) MOVED + generalized
/// it to `benten_caps::chain_authority::validate_chain_with_envelope_ceiling`
/// (factored predicate `envelope_ceiling_rejects_cap`). P5
/// (`feat(#669)`) GENERALIZES this same factored predicate over
/// device-envelope **and** plugin-manifest `shares` (per
/// impl-design-COLLAPSE §2) so the manifest path calls THIS primitive,
/// not a parallel one — the seam stays single (the #707 parallel-pipe
/// shape the COLLAPSE exists to kill).
///
/// Returns `Ok(())` when the row's `scope` is within the ceiling (or
/// no ceiling is present — legacy unsigned envelope / non-wire merge);
/// `Err(EngineError::Other { DeviceAttestationForged, .. })` when the
/// ceiling forbids the scope (currently: `host:sandbox:*` scope under
/// a `runs_sandbox=false` ceiling — the only envelope dimension a
/// sync row's cap-scope can exercise; broader dimensions ride P5's
/// generalization).
///
/// **Native (full-peer) only — cfg-gated like `manifest_signing`.**
/// The `benten_id::device_attestation::CapabilityEnvelope` ceiling
/// type transitively pulls `getrandom`, which rejects the
/// `wasm32-unknown-unknown` browser-backend bundle (this fn's sole
/// production caller, `Engine::apply_atrium_merge`, is itself inside
/// the `#[cfg(not(feature = "browser-backend"))] impl Engine` block).
/// Per CLAUDE.md baked-in #17 + DECISION-RECORD §4: device-envelope
/// ceiling-recheck is full-peer work; the thin/browser wasm32 client
/// is a *view into* a full peer and does not perform device-envelope
/// ceiling-recheck itself — the full peer still enforces, so excluding
/// this surface from the browser bundle is architecturally correct,
/// NOT a security regression. Mirrors the existing native-only
/// `manifest_signing` module precedent (`benten_engine::manifest_signing`).
#[cfg(not(feature = "browser-backend"))]
pub fn envelope_ceiling_admits_row(
    ceiling: Option<&benten_id::device_attestation::CapabilityEnvelope>,
    scope: &str,
    zone: &str,
    key: &str,
) -> Result<(), EngineError> {
    let Some(env) = ceiling else {
        // No verified ceiling for this merge (legacy unsigned
        // envelope, or a non-wire merge path) — nothing to AND.
        return Ok(());
    };
    // J8: a `runs_sandbox=false`-attested inbound writer MUST NOT be
    // able to land a row that exercises `host:sandbox:*`. ct-eq is
    // unnecessary here (the scope string is the public cap schema, and
    // this is a prefix structural test, not a secret compare) — the
    // project's ct-eq UNIFORMITY commitment applies to identity/secret
    // compares, not cap-schema prefix routing.
    if scope.starts_with("host:sandbox:") && !env.runs_sandbox {
        return Err(EngineError::Other {
            code: ErrorCode::DeviceAttestationForged,
            message: format!(
                "apply_atrium_merge: inbound row exceeds verified device \
                 envelope-ceiling (zone='{zone}' key='{key}' scope='{scope}'): a \
                 runs_sandbox=false-attested writer cannot exercise host:sandbox:* — \
                 J8 ceiling-AND (CLAUDE.md #17 thin-shape property; COLLAPSE single seam)"
            ),
        });
    }
    Ok(())
}

/// COLLAPSE P5 — the **#1241 / F2 cap-predicate-complete** ceiling-AND.
///
/// DECISION-RECORD §4b RATIFIED: the F2 ceiling-predicate is shipped
/// **(a) zone-scoped for v1** at the inbound-merge surface
/// ([`envelope_ceiling_admits_row`] above — the gap was verified
/// *INERT* at HEAD by `F2-exploitability-investigation.md`: the
/// inbound data-zone write path never dispatches a SANDBOX host-fn, so
/// a `host:sandbox:*` cap on that path is an unused string). The
/// **cap-predicate completion (#1241)** "lands WITH P5's
/// #669-unified-ceiling — ONE mechanism, not parallel."
///
/// This function IS that ONE mechanism on the engine side: it
/// discriminates on the inbound writer's **actual delegated
/// `cap.resource` strings** (not the synthetic `{zone}:write` scope
/// `envelope_ceiling_admits_row` uses), delegating to the single
/// shared predicate
/// [`benten_caps::envelope_ceiling_first_rejected_resource`] — the
/// SAME `envelope_ceiling_rejects_cap` core the device chain-walk
/// ([`benten_caps::validate_chain_with_envelope_ceiling`]) and the
/// #669 plugin-manifest chain-walk
/// ([`benten_caps::validate_chain_with_manifest_ceiling`]) call. ONE
/// code path, build-constraint iii — NOT a parallel pipe.
///
/// **v1 wiring posture (per §4b):** the inbound-sync wire frame does
/// not currently thread the inbound writer's full UCAN delegation
/// chain to the per-row merge seam (threading it is a *new wire
/// surface* — a genuine arch change §4b explicitly deferred past v1
/// given the INERT verdict). The engine inbound-merge therefore
/// continues to call the (a) zone-scoped [`envelope_ceiling_admits_row`]
/// for v1; THIS cap-resource-complete seam is the ready, shared
/// mechanism every surface that *does* have the writer's cap-resource
/// set (the chain-walk validators; future wire-chain-threaded merge in
/// Phase-4-Meta) enforces through — so when #1241's wire-threading
/// lands there is no second predicate to write. The completion is the
/// predicate; the wiring is mechanical and §4b-deferred.
#[cfg(not(feature = "browser-backend"))]
pub fn envelope_ceiling_admits_cap_resources<'a>(
    ceiling: Option<&benten_id::device_attestation::CapabilityEnvelope>,
    cap_resources: impl IntoIterator<Item = &'a str>,
    zone: &str,
    key: &str,
) -> Result<(), EngineError> {
    if let Some(offending) =
        benten_caps::envelope_ceiling_first_rejected_resource(ceiling, cap_resources)
    {
        return Err(EngineError::Other {
            code: ErrorCode::DeviceAttestationForged,
            message: format!(
                "apply_atrium_merge: inbound writer's delegated capability \
                 '{offending}' exceeds the verified envelope-ceiling (zone='{zone}' \
                 key='{key}'): a runs_sandbox=false principal cannot exercise \
                 host:sandbox:* even via an otherwise-valid chain — #1241/F2 \
                 cap-predicate-complete ceiling-AND (CLAUDE.md #17 thin-shape \
                 property; COLLAPSE P5 single shared predicate)"
            ),
        });
    }
    Ok(())
}
