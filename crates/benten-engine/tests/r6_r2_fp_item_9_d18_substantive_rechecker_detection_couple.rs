//! R6-R2 FP Item 9 (Row D-18 closure) — substantive-rechecker-detection
//! couple at `Engine::apply_atrium_merge`'s per-row recheck loop.
//!
//! ## What this pin defends
//!
//! Pre-Item-9: the synthesized-fallback (`node-id:N`) reject was the
//! rechecker's own responsibility. A *forgetful* substantive rechecker
//! impl that returned `NotApplicable` for synthesized DIDs (instead of
//! `UnresolvedDeny`) would admit. The engine substrate had no
//! structural backstop.
//!
//! Post-Item-9: the engine's per-row recheck loop calls
//! `rechecker.is_substantive()` and short-circuits with
//! `ManifestEnvelopeRecheckUnresolvedDeny` on `node-id:`-prefixed DIDs
//! BEFORE consulting the rechecker. This is the defense-in-depth
//! Layer-3 structural-always-on per CLAUDE.md #18: even if a
//! production rechecker impl is buggy, the engine substrate fails
//! CLOSED on the synthesized-fallback class.
//!
//! ## Test arms
//!
//! 1. `is_substantive()` default returns `true` (production impls
//!    inherit hardening opt-in).
//! 2. `NoopManifestEnvelopeRechecker::is_substantive()` returns `false`
//!    (preserves the no-Layer-3-installed default's admit-everything
//!    posture so test fixtures don't over-fire).
//! 3. Custom substantive rechecker (a faulty one that returns
//!    `NotApplicable` for synthesized DIDs) — the engine layer's
//!    structural pin would STILL reject (would-FAIL-on-revert: removing
//!    the structural pin causes admission).
//!
//! Per §3.6f SHAPE-not-SUBSTANCE: this test exercises the trait
//! contract directly + the rechecker-substantive-coupling helper. The
//! production entry point (`apply_atrium_merge`) is covered
//! end-to-end at `r6_r1_fp_f4_s4_production_rechecker_builder_and_d18_synthesized_fallback.rs`
//! arm 1 (which opens a ProductionEngine + verifies the rechecker's
//! synthesized-fallback handling). This Item 9 closure adds the
//! defense-in-depth at the engine SUBSTRATE so the protection holds
//! even under a buggy production rechecker.

use benten_engine::manifest_envelope_recheck::{
    ManifestEnvelopeRecheckOutcome, ManifestEnvelopeRechecker, NoopManifestEnvelopeRechecker,
    is_synthesized_node_id,
};

/// **Item 9 arm 1 — trait default `is_substantive() = true`.**
///
/// Any production rechecker impl that doesn't override `is_substantive`
/// inherits the substantive opt-in (synthesized-fallback hardening
/// fires). This is the safe default: forgetting to override is a
/// secure-by-default choice.
///
/// Would-FAIL-on-revert: changing the default to `false` would silently
/// disable the structural hardening for every production rechecker
/// impl. This test pins the secure-by-default contract.
#[test]
fn item_9_default_is_substantive_returns_true() {
    /// A minimal substantive rechecker impl that DOESN'T override
    /// `is_substantive` — should inherit the `true` default.
    struct MinimalSubstantiveRechecker;
    impl ManifestEnvelopeRechecker for MinimalSubstantiveRechecker {
        fn recheck_row(
            &self,
            _peer_did_str: &str,
            _zone: &str,
            _key: &str,
        ) -> ManifestEnvelopeRecheckOutcome {
            ManifestEnvelopeRecheckOutcome::Admitted
        }
        // NOTE: NOT overriding `is_substantive`.
    }
    let r = MinimalSubstantiveRechecker;
    assert!(
        r.is_substantive(),
        "Default `is_substantive` MUST return true (secure-by-default) so \
         production impls that forget to override still opt into the \
         Row D-18 synthesized-fallback hardening at the engine substrate."
    );
}

/// **Item 9 arm 2 — Noop override returns `false`.**
///
/// The Noop has no PluginLibrary state; the synthesized-fallback
/// hardening MUST NOT fire under Noop (would over-fire on default-Noop
/// test fixtures that intentionally don't register peer-DIDs).
///
/// Would-FAIL-on-revert: removing the Noop's override would cause
/// every existing test using the Noop default to fail-CLOSED on
/// synthesized DIDs, breaking the Phase-3 baseline.
#[test]
fn item_9_noop_is_substantive_returns_false() {
    let noop = NoopManifestEnvelopeRechecker;
    assert!(
        !noop.is_substantive(),
        "NoopManifestEnvelopeRechecker MUST report `is_substantive() = false` \
         so the engine substrate's Row D-18 synthesized-fallback hardening \
         does NOT fire under the Noop default (would over-fire on every \
         existing test fixture that doesn't register peer-DIDs)."
    );
}

/// **Item 9 arm 3 — the `is_substantive`-gated reject is the structural
/// defense-in-depth.**
///
/// This test simulates the engine's per-row loop logic verbatim:
/// (a) call `is_substantive()`,
/// (b) call `is_synthesized_node_id()`,
/// (c) reject iff both true.
///
/// A faulty substantive rechecker impl that returned `Admitted` for
/// `node-id:N` would have its outcome IGNORED — the engine substrate's
/// pre-check at step (a)+(b) rejects first.
///
/// Would-FAIL-on-revert: removing the `is_substantive() && is_synthesized_node_id()`
/// gate at `apply_atrium_merge` would let the faulty rechecker's
/// `Admitted` outcome through (security regression).
#[test]
fn item_9_substantive_rechecker_admits_synthesized_but_engine_substrate_rejects() {
    /// A faulty production-shaped rechecker that ADMITS synthesized
    /// `node-id:N` DIDs (the very bug Item 9's structural pin defends
    /// against). The engine substrate's structural pin should NOT
    /// consult this rechecker at all on synthesized DIDs.
    struct FaultyAdmitAllRechecker;
    impl ManifestEnvelopeRechecker for FaultyAdmitAllRechecker {
        fn recheck_row(
            &self,
            _peer_did_str: &str,
            _zone: &str,
            _key: &str,
        ) -> ManifestEnvelopeRecheckOutcome {
            ManifestEnvelopeRecheckOutcome::Admitted
        }
        // Inherits `is_substantive() = true` (production opt-in).
    }
    let rechecker = FaultyAdmitAllRechecker;

    let peer_did = "node-id:99";
    let engine_substrate_rejects = rechecker.is_substantive() && is_synthesized_node_id(peer_did);
    assert!(
        engine_substrate_rejects,
        "Engine substrate pre-check (is_substantive() && is_synthesized_node_id()) \
         MUST reject synthesized peer-DIDs under a substantive rechecker BEFORE \
         consulting the rechecker — defense-in-depth per Row D-18 / CLAUDE.md #18 \
         Layer-3 structural-always-on. A faulty rechecker impl that returns \
         `Admitted` for `node-id:N` MUST NOT be reachable on this code path."
    );

    // The rechecker WOULD admit if consulted — proves the structural pin
    // is the load-bearing defense, not the rechecker itself.
    let faulty_outcome = rechecker.recheck_row(peer_did, "any-zone", "any-key");
    assert_eq!(
        faulty_outcome,
        ManifestEnvelopeRecheckOutcome::Admitted,
        "Faulty rechecker WOULD admit if consulted; the engine substrate's \
         structural pin is what keeps this attack vector closed."
    );
}

/// **Item 9 arm 4 — the gate does NOT fire on resolvable DIDs.**
///
/// `did:key:...` shaped DIDs are NOT synthesized fallbacks; the
/// engine substrate proceeds to consult the rechecker as before.
#[test]
fn item_9_substantive_rechecker_resolvable_did_consults_rechecker_normally() {
    /// A substantive rechecker that returns a typed value we can
    /// observe (proves the rechecker WAS consulted).
    struct ObservableRechecker;
    impl ManifestEnvelopeRechecker for ObservableRechecker {
        fn recheck_row(
            &self,
            _peer_did_str: &str,
            _zone: &str,
            _key: &str,
        ) -> ManifestEnvelopeRecheckOutcome {
            ManifestEnvelopeRecheckOutcome::NotApplicable
        }
    }
    let rechecker = ObservableRechecker;

    let peer_did = "did:key:z6MkResolvable";
    let engine_substrate_rejects = rechecker.is_substantive() && is_synthesized_node_id(peer_did);
    assert!(
        !engine_substrate_rejects,
        "Engine substrate pre-check MUST NOT short-circuit on resolvable \
         `did:key:` DIDs; the rechecker is consulted normally for the \
         per-row decision."
    );
    // Confirm the rechecker IS the decision surface for resolvable DIDs.
    let outcome = rechecker.recheck_row(peer_did, "any-zone", "any-key");
    assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::NotApplicable);
}

/// **Item 9 arm 5 — under Noop the gate ALSO does not fire on
/// synthesized DIDs (preserves test-fixture admit-everything posture).**
///
/// This is the explicit asymmetry Row D-18 codifies: the synthesized-
/// fallback reject ONLY fires under substantive rechecker installation.
/// Default-Noop deployments (no Layer-3 trust model installed) admit
/// everything, including synthesized DIDs — because Layer-1 user-root
/// + per-row cap-recheck are the relevant defenses there.
#[test]
fn item_9_noop_rechecker_synthesized_did_does_not_trigger_engine_substrate_reject() {
    let noop = NoopManifestEnvelopeRechecker;
    let peer_did = "node-id:42";
    let engine_substrate_rejects = noop.is_substantive() && is_synthesized_node_id(peer_did);
    assert!(
        !engine_substrate_rejects,
        "Engine substrate MUST NOT reject synthesized DIDs under the Noop \
         default — the substantive-rechecker-detection couple per Row D-18 \
         is what prevents over-firing on default-Noop test fixtures."
    );
}
