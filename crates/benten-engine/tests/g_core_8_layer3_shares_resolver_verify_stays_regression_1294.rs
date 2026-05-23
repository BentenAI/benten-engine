//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 — VERIFY-STAYS-REGRESSION arms
//! for the #1294-ALREADY-LANDED Layer-3 `SharesPolicyResolver`
//! enforcement.
//!
//! ## NOT RED-phase — verify-stays-regression
//!
//! #1294 (`39a0e732`, refinement-audit Wave-E) ALREADY landed real
//! Layer-3 manifest-`shares`-envelope enforcement into the
//! `delegate_capability` production path:
//!   - `crates/benten-engine/src/shares_policy_resolver.rs` (the
//!     `SharesPolicyResolver` port + `DelegationResolution` enum +
//!     `NoopSharesPolicyResolver` default + `into_result` mapping);
//!   - the existing closure-pins
//!     `tests/layer3_shares_envelope_enforcement_1197_closure_pin.rs`
//!     and the napi
//!     `cap_delegate_napi_layer3_shares_enforcement_1018_closure_pin.rs`.
//!
//! These tests are NOT `#[ignore]`d. They are
//! verify-stays-regression guards: they assert the #1294 enforcement
//! semantics REMAIN correct through the G-CORE-8 §4.36/§4.37 work
//! (which touches the *adjacent* `apply_atrium_merge` recheck path —
//! the G-CORE-8 changes MUST NOT regress the delegate_capability
//! Layer-3 fail-CLOSED contract #1294 shipped).
//!
//! ## Reconciliation note (the key R3-B5 split)
//!
//! `DelegationResolution::into_result` maps:
//!   - `NotPluginPrincipal | Admitted => Ok(())`
//!   - `Denied | NoManifest => Err(PluginDelegationOutsideManifestEnvelope)`
//! i.e. the `delegate_capability` path is ALREADY fail-CLOSED on
//! no-manifest (`NoManifest => Err`). This is DISTINCT from the
//! `apply_atrium_merge` recheck path which is STILL pre-flip
//! (`NotApplicable => Ok` — admit-on-unresolved). The §4.36 flip is
//! about the recheck path; THIS path is the already-correct precedent
//! the flip mirrors. Do NOT mistake the (correct, shipped)
//! `into_result` mapping for the (still-broken, RED)
//! `outcome_to_row_reject` mapping.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): each arm exercises the
//!     SPECIFIC `DelegationResolution` variant → `into_result` mapping
//!     (production helper, observable typed-ErrorCode consequence,
//!     would-FAIL-if-reverted-to-AllPermit).
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!   - §3.5g cross-language mirror: the napi half is pinned by
//!     `cap_delegate_napi_layer3_shares_enforcement_1018_closure_pin.rs`
//!     (already-landed, #1294); this file is the engine-side mirror.
//!
//! Pins: G-CORE-8 · C8 (verify-stays) · §1.A.FROZEN item 12.
//! R2 map: TF-8 verify-regression arm (#1294-already-landed split).

use benten_engine::shares_policy_resolver::{
    DelegationResolution, NoopSharesPolicyResolver, SharesPolicyResolver,
};
use benten_errors::ErrorCode;

#[test]
fn into_result_admit_paths_proceed_verify_stays() {
    // user-root principal (Layer 1 anchor) + explicit Admitted both
    // proceed. #1294 contract; G-CORE-8 MUST NOT regress.
    assert!(
        DelegationResolution::NotPluginPrincipal
            .into_result()
            .is_ok(),
        "verify-stays: NotPluginPrincipal (user-root) must proceed"
    );
    assert!(
        DelegationResolution::Admitted.into_result().is_ok(),
        "verify-stays: explicit Admitted must proceed"
    );
}

#[test]
fn into_result_deny_path_fail_closed_verify_stays() {
    // A plugin-principal whose `shares` policy DENIES the step → the
    // typed ErrorCode. Reverting to the deleted `AllPermit` shim
    // would make this Ok — the would-FAIL-if-reverted arm.
    assert_eq!(
        DelegationResolution::Denied.into_result(),
        Err(ErrorCode::PluginDelegationOutsideManifestEnvelope),
        "verify-stays: Denied must fail-CLOSED with the typed ErrorCode \
         (would-FAIL if reverted to AllPermit)"
    );
}

#[test]
fn into_result_no_manifest_fail_closed_verify_stays() {
    // The fail-CLOSED-on-unverifiable precedent the §4.36 recheck
    // flip mirrors: a plugin-principal with NO resolvable manifest
    // CANNOT verify the envelope ⇒ REJECT (never fail-OPEN).
    assert_eq!(
        DelegationResolution::NoManifest.into_result(),
        Err(ErrorCode::PluginDelegationOutsideManifestEnvelope),
        "verify-stays: NoManifest must fail-CLOSED — an un-verifiable \
         cross-plugin delegation rejects, never admits"
    );
}

#[test]
fn noop_resolver_classifies_everything_not_plugin_principal_verify_stays() {
    // The DEFAULT `delegate_capability` resolver is the Noop (v1
    // behavior identical: the manifest-`shares` gate does not
    // constrain delegation when no real adapter is installed; Layer-1
    // user-root anchor + attenuation + always-on private-namespace
    // clause still apply). This is the documented #1294 default; it
    // is the same opt-in posture the §4.36 flip will REMOVE for the
    // RECHECK path (NOT this delegate path — see the module doc
    // reconciliation note).
    let r = NoopSharesPolicyResolver;
    assert_eq!(
        r.resolve_delegation("did:key:zSource", "store:notes:read", "did:key:zTarget"),
        DelegationResolution::NotPluginPrincipal,
        "verify-stays: the #1294 Noop resolver classifies every \
         principal NotPluginPrincipal (documented default behavior)"
    );
}
