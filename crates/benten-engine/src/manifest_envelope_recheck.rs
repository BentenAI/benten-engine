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
/// at `docs/future/phase-4-backlog.md §4.19` as a Phase-4-Meta carry.
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
