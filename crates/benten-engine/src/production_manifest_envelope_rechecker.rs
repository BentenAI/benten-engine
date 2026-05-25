//! R6 R1 FP-F4 §S4 (Row D-4 closure) — Production
//! `ManifestEnvelopeRechecker` substrate impl.
//!
//! Wires CLAUDE.md baked-in #18 Layer-3 substantive recheck at the
//! sync-merge boundary — replaces the always-mounted
//! `NoopManifestEnvelopeRechecker` when a production engine builder
//! installs this glue (Commit 6 ProductionEngineBuilder).
//!
//! ## Dep-direction discipline
//!
//! The substantive `PluginLibrary` + `UserDidRegistry` data-source
//! glue lives at the foundation-side (G-COMP-1 follow-up); this
//! engine-side impl covers the load-bearing v1-beta hardenings:
//!
//! 1. The synthesized-fallback hardening (Row D-18) — `node-id:N`
//!    DIDs surface `UnresolvedDeny` here when this substantive
//!    rechecker IS installed (default Noop continues to admit-via-
//!    NotApplicable per its Phase-3 baseline contract).
//! 2. The full chain-walk substantive impl is the G-COMP-1
//!    deliverable per Row D-4 narrative; at v1-beta this impl
//!    returns `NotApplicable` for resolvable peer-DIDs (preserving
//!    Phase-3 baseline for resolvable peers + fail-CLOSED for the
//!    synthesized arm).

use crate::manifest_envelope_recheck::{
    ManifestEnvelopeRecheckOutcome, ManifestEnvelopeRechecker, is_synthesized_node_id,
};

/// Production `ManifestEnvelopeRechecker` — the substantive impl
/// that replaces the always-mounted `NoopManifestEnvelopeRechecker`
/// when a production builder installs it. At v1-beta the load-
/// bearing addition is the synthesized-fallback hardening (Row D-18);
/// the full PluginLibrary-driven chain walk is the G-COMP-1
/// deliverable per Row D-4.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProductionManifestEnvelopeRechecker {
    _private: (),
}

impl ProductionManifestEnvelopeRechecker {
    /// New empty production rechecker.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl ManifestEnvelopeRechecker for ProductionManifestEnvelopeRechecker {
    fn recheck_row(
        &self,
        peer_did_str: &str,
        _zone: &str,
        _key: &str,
    ) -> ManifestEnvelopeRecheckOutcome {
        // R6 R1 FP-F4 §S4 + Row D-18: synthesized-fallback hardening
        // fires WHEN this substantive rechecker IS installed
        // (default Noop continues to admit-via-NotApplicable as
        // documented).
        if is_synthesized_node_id(peer_did_str) {
            return ManifestEnvelopeRecheckOutcome::UnresolvedDeny;
        }
        // G-COMP-1 follow-up: walk the chain through
        // `validate_chain_with_manifest_envelope` against the
        // installed PluginLibrary; for v1-beta the resolvable-DID
        // arm admits (NotApplicable preserves Phase-3 baseline
        // for resolvable peers).
        ManifestEnvelopeRecheckOutcome::NotApplicable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesized_peer_did_surfaces_unresolved_deny() {
        let rechecker = ProductionManifestEnvelopeRechecker::new();
        let outcome = rechecker.recheck_row("node-id:42", "zone-a", "key-x");
        assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::UnresolvedDeny);
    }

    #[test]
    fn resolvable_did_admits_via_not_applicable_v1_beta_posture() {
        let rechecker = ProductionManifestEnvelopeRechecker::new();
        let outcome = rechecker.recheck_row("did:key:zResolvable", "zone-a", "key-x");
        assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::NotApplicable);
    }

    #[test]
    fn empty_did_string_is_not_synthesized() {
        let rechecker = ProductionManifestEnvelopeRechecker::new();
        let outcome = rechecker.recheck_row("", "zone-a", "key-x");
        assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::NotApplicable);
    }
}
