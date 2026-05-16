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

/// Outcome of a manifest-envelope recheck call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEnvelopeRecheckOutcome {
    /// No manifest-envelope chain found for this row (e.g. user-issued
    /// write not delegated through any plugin) — recheck passes
    /// trivially. The Layer 1 user-root-anchor check in
    /// `CapabilityPolicy::check_write` still applies.
    NotApplicable,
    /// A manifest-envelope chain was found AND every step fits the
    /// source plugin's `shares` policy. Admit the row.
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

/// Default rechecker — returns `NotApplicable` for every call.
/// Behavior is observably identical to Phase-3 (no envelope recheck).
/// Engines built at the default (post-R6-FP-A: `Engine::default` installs
/// `Some(Arc::new(Noop))`) get this seam structurally wired so the
/// recheck-path always fires; operators swap in a real adapter via
/// `Engine::set_manifest_envelope_rechecker(Arc::new(<real>))`. A fluent
/// `EngineBuilder::with_manifest_envelope_rechecker` setter is named
/// at `docs/future/phase-4-backlog.md §4.36` as a Phase-4-Meta carry.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopManifestEnvelopeRechecker;

impl ManifestEnvelopeRechecker for NoopManifestEnvelopeRechecker {
    fn recheck_row(
        &self,
        _peer_did_str: &str,
        _zone: &str,
        _key: &str,
    ) -> ManifestEnvelopeRecheckOutcome {
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
        ManifestEnvelopeRecheckOutcome::NotApplicable
        | ManifestEnvelopeRecheckOutcome::Admitted => Ok(()),
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
/// it on inbound sync. P5 (`feat(#669)`) GENERALIZES this same
/// predicate over device-envelope **and** plugin-manifest `shares`
/// (the CONSOLIDATE move per impl-design-COLLAPSE §2) so the manifest
/// path calls THIS primitive, not a parallel one — the seam stays
/// single (the #707 parallel-pipe shape the COLLAPSE exists to kill).
///
/// **P5 / #1241 / F2 — capability-predicate completion.** This now
/// discriminates on the inbound writer's effective **cap-resources**
/// (`writer_cap_resources`) — the literal CLAUDE.md #17 predicate
/// (*"a `runs_sandbox=false` principal must not exercise
/// `host:sandbox:*`"*) — NOT a synthetic `{zone}:write` zone-scope.
/// The COLLAPSE-P3 shipped a zone-scoped proxy
/// (`sec-review-1238 F2`); that proxy was verified INERT at HEAD
/// (`F2-exploitability-investigation.md` — a `host:sandbox:*` cap
/// does nothing on the inbound data-zone write path), so completing
/// the predicate to the literal cap.resource is strictly-more-
/// enforcement of a baked-in commitment, not a regression. The
/// zone-write scope is still passed + checked (defense-in-depth: a
/// `host:sandbox:*`-named zone is also caught) but the cap.resource
/// arm is the load-bearing #17 predicate.
///
/// **One mechanism, two callers** (DECISION-RECORD §4
/// build-constraint iii): this routes through the ONE
/// `benten_caps::plugin_delegation::ceiling_admits_cap` —
/// the SAME function the plugin-manifest delegation gate
/// (`Engine::delegate_capability` →
/// `benten_caps::plugin_delegation::check_delegation_within_envelope`)
/// calls. There is NO parallel device-vs-manifest ceiling pipe (the
/// #707 / META-#1140 shape the COLLAPSE exists to kill).
///
/// Returns `Ok(())` when every checked cap-resource is within the
/// ceiling (or no ceiling is present — legacy unsigned envelope /
/// non-wire merge);
/// `Err(EngineError::Other { DeviceAttestationForged, .. })` when the
/// ceiling forbids a cap-resource (a `host:sandbox:*` cap-resource —
/// or `host:sandbox:*`-prefixed zone-scope — under a
/// `runs_sandbox=false` ceiling).
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
    writer_cap_resources: &[&str],
    zone: &str,
    key: &str,
) -> Result<(), EngineError> {
    let Some(env) = ceiling else {
        // No verified ceiling for this merge (legacy unsigned
        // envelope, or a non-wire merge path) — nothing to AND.
        return Ok(());
    };
    // The ONE unified ceiling-check (build-constraint iii). The device
    // envelope's `runs_sandbox` dimension is the input; the plugin-
    // manifest delegation gate calls the SAME `ceiling_admits_cap`
    // with an `EnvelopeCeiling::PluginShares` ceiling.
    let device_ceiling: benten_caps::plugin_delegation::EnvelopeCeiling<
        '_,
        // No `shares` policy on the device arm — the type parameter is
        // unused for `EnvelopeCeiling::Device`; a zero-sized stub
        // satisfies the `SharesPolicyView` bound.
        DeviceArmSharesStub,
    > = benten_caps::plugin_delegation::EnvelopeCeiling::Device {
        runs_sandbox: env.runs_sandbox,
    };

    // #1241 / F2 — the load-bearing CLAUDE.md #17 predicate: check the
    // inbound writer's actual cap-RESOURCES (not a synthetic zone
    // proxy). A `runs_sandbox=false` writer self-delegating
    // `host:sandbox:exec` is rejected here even if it targets an
    // ordinary data zone (the gap sec-review-1238 F2 flagged; INERT at
    // HEAD per F2-exploitability-investigation.md, completed here so
    // the baked-in commitment is enforced regardless of inertness).
    for cap_resource in writer_cap_resources {
        if benten_caps::plugin_delegation::ceiling_admits_cap(&device_ceiling, cap_resource)
            .is_err()
        {
            return Err(EngineError::Other {
                code: ErrorCode::DeviceAttestationForged,
                message: format!(
                    "apply_atrium_merge: inbound row exceeds verified device \
                     envelope-ceiling (zone='{zone}' key='{key}' \
                     cap_resource='{cap_resource}'): a runs_sandbox=false-attested \
                     writer cannot exercise host:sandbox:* — unified J8 ceiling-AND \
                     (CLAUDE.md #17 thin-shape property; #1241 cap.resource predicate; \
                     COLLAPSE single seam)"
                ),
            });
        }
    }

    // Defense-in-depth: also reject when the row's zone-write scope
    // itself names the sandbox-authority dimension (the P3-shipped
    // proxy — kept so a `host:sandbox:*`-named zone is still caught
    // even if the writer's cap-resources were not threaded). Routes
    // through the SAME unified mechanism.
    if benten_caps::plugin_delegation::ceiling_admits_cap(&device_ceiling, scope).is_err() {
        return Err(EngineError::Other {
            code: ErrorCode::DeviceAttestationForged,
            message: format!(
                "apply_atrium_merge: inbound row exceeds verified device \
                 envelope-ceiling (zone='{zone}' key='{key}' scope='{scope}'): a \
                 runs_sandbox=false-attested writer cannot exercise host:sandbox:* — \
                 unified J8 ceiling-AND (CLAUDE.md #17 thin-shape property; \
                 COLLAPSE single seam)"
            ),
        });
    }
    Ok(())
}

/// Zero-sized `SharesPolicyView` stub for the device arm of the
/// unified ceiling type. The device-envelope ceiling
/// (`EnvelopeCeiling::Device`) never consults a `shares` policy — the
/// type parameter exists only because the unified `EnvelopeCeiling`
/// enum is generic over the plugin-manifest arm. `permits` is
/// unreachable for the device arm; it conservatively denies (fail-
/// closed) if ever called.
#[cfg(not(feature = "browser-backend"))]
#[doc(hidden)]
pub struct DeviceArmSharesStub;

#[cfg(not(feature = "browser-backend"))]
impl benten_caps::plugin_delegation::SharesPolicyView for DeviceArmSharesStub {
    fn permits(&self, _cap_pattern: &str, _target_plugin_did: &benten_id::did::Did) -> bool {
        false
    }
}
