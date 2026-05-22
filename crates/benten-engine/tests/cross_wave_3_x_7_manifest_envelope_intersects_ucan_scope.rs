//! Phase-4-Meta-Core — R3-W5 — Cross-wave integration test §4-C:
//! G-CORE-3 × G-CORE-7 install-lifecycle composition. Plugin manifest
//! scope correctly intersects with the UCAN-gated SubgraphSpec scope
//! (chain-validator + manifest envelope BOTH must admit; one denying
//! causes overall deny). The §4-C claim: install plugin P with
//! manifest scope {A,B}; grant P a UCAN scoping {B,C}; P's effective
//! scope = {B} (the structural intersection); requests for A or C
//! return `OutOfScope`.
//!
//! ============================================================================
//! RED-PHASE STATUS
//! ============================================================================
//!
//! `#[ignore = "RED-PHASE: un-ignore at G-CORE-3b + G-CORE-7 composition
//! wave"]`. G-CORE-7 (PR #1312 install/lifecycle hardening) was OPEN at
//! R2 dispatch time; per R2 §3 retrospective + the R3-W5 §3.5n
//! re-check, **PR #1312 has NOT YET MERGED at HEAD `c9c11c56`**
//! (verified `git log origin/main..` is empty; PR #1312 still BLOCKED/
//! OPEN per night-shift state). G-CORE-3b (RestrictedSpec + Scope +
//! AuthorizationGrant) is also not landed. So this file is RED on
//! BOTH halves.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD c9c11c56 — R3-W5 §3.5n pass)
//! ============================================================================
//!
//!   * G-CORE-3b: `RestrictedSpec` / `AuthorizationGrant` / structured
//!     `Scope` enum — NOT BUILT. The Scope-enum's two arms
//!     (`Hashes(Vec<Hash>)` + `RestrictedSelector(RestrictedSpec)`)
//!     per §1.A.FROZEN item 15 (c) are unbuilt.
//!   * G-CORE-7: PR #1312 (manifest store + caps-grew fresh-consent +
//!     §4.20 validate_with_clock + Steps 9/10/11 rollback + §4.36
//!     production rechecker) NOT yet merged.
//!   * §4-C intersection rule is the load-bearing composition claim;
//!     a plugin's effective scope is the structural intersection of
//!     its manifest's declared scope AND its UCAN-grant scope.
//!
//! ============================================================================
//! §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
//! ============================================================================
//!
//!  1. §3.5b HARDENED — implementer sweeps SECURITY-POSTURE.md +
//!     PLUGIN-MANIFEST.md before push.
//!  2. §3.6b sub-rule 4 — SPECIFIC arm = INTERSECTION of manifest +
//!     UCAN scopes (not umbrella "auth works"); OBSERVABLE = a
//!     request for a row in only ONE of the two scopes returns
//!     `OutOfScope`; WOULD-FAIL if a future agent ships UNION (a row
//!     in either-not-both surfaces in plugin's effective scope) or
//!     manifest-only (UCAN narrowing ignored).
//!  3. §3.6e — RED-PHASE staged-pin.
//!  4. §3.6f (pim-18) — production call-site = future
//!     `plugin_lifecycle::install_plugin` + the §4.36 production
//!     `ManifestEnvelopeRechecker` (security-r1-1) + the future
//!     `Engine::walk_share_scope` chain-validator. Substantive body
//!     drives all three through a real plugin install.
//!  5. §3.13 — per-test-static; each test owns its own Engine.
//!  6. §3.5g — no new ErrorCode.
//!  7. §3.5n — verified PR #1312 not yet merged.
//!
//! Pin source: R2-test-landscape.md §4-C + plan G-CORE-7 § + G-CORE-3b
//! sub-wave §.

#![allow(clippy::unwrap_used)]

/// §4-C ARM 1 — A plugin installed with manifest scope {A,B} granted
/// a UCAN scoping {B,C} has effective scope = {B}. Requests for A
/// (manifest-only) and C (UCAN-only) both return `OutOfScope`.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3b + G-CORE-7 composition wave."]
fn manifest_scope_intersect_ucan_scope_yields_intersection_not_union() {
    // Body intent at un-ignore:
    //
    //   let plugin_manifest = PluginManifest::for_scope(&[A, B], &alice);
    //   let install_result = engine.install_plugin(&plugin_manifest);
    //   assert!(install_result.is_ok());
    //
    //   // User grants the plugin a UCAN scoped to {B, C}.
    //   let grant = engine.issue_ucan_to_plugin(
    //       &plugin_did,
    //       &Scope::RestrictedSelector(restricted_spec_for(&[B, C])),
    //   );
    //
    //   // Effective scope = intersection {B}.
    //   let req_a = engine.plugin_request(&plugin_did, &grant, &row_in_A);
    //   assert!(matches!(req_a, Err(EngineError::OutOfScope(_))),
    //           "§4-C: A is in manifest but NOT in UCAN — intersection \
    //            denies. Would-FAIL on union semantics (which would \
    //            admit A on the manifest-only basis)");
    //
    //   let req_b = engine.plugin_request(&plugin_did, &grant, &row_in_B);
    //   assert!(req_b.is_ok(), "§4-C: B is in BOTH — intersection admits");
    //
    //   let req_c = engine.plugin_request(&plugin_did, &grant, &row_in_C);
    //   assert!(matches!(req_c, Err(EngineError::OutOfScope(_))),
    //           "§4-C: C is in UCAN but NOT in manifest — intersection \
    //            denies. Would-FAIL on UCAN-overrides-manifest semantics.");
    //
    panic!(
        "RED-PHASE: §4-C G-CORE-3b×G-CORE-7 intersection composition. \
         At HEAD G-CORE-3b RestrictedSpec/Scope unbuilt; G-CORE-7 \
         install_plugin/manifest-store not yet merged (PR #1312 OPEN). \
         Un-ignore at composition wave; wire body against produced \
         `Engine::install_plugin` + `plugin_request` + Scope::Restricted \
         intersection logic. Would-FAIL on union OR manifest-only OR \
         UCAN-only effective-scope semantics."
    );
}

/// §4-C ARM 2 — Adversarial: a plugin with manifest scope {A,B}
/// holding a UCAN that nominally widens to {A,B,C} STILL has
/// effective scope = {A,B}. The chain-validator (G-CORE-3b
/// RestrictedSpec non-widening) catches the attempted widening at
/// install-time OR at access-time — but EITHER WAY the plugin
/// cannot exceed its manifest envelope. This is the **chain non-
/// widening** property R1 ratified (Path (a) restricted-spec
/// language only; Path b structurally unsound per Spike H+1.1).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3b chain-validator landing + G-CORE-7 install-pass."]
fn ucan_widening_attempt_beyond_manifest_envelope_rejected_not_admitted() {
    // Body intent at un-ignore:
    //
    //   let plugin_manifest = PluginManifest::for_scope(&[A, B], &alice);
    //   engine.install_plugin(&plugin_manifest)?;
    //
    //   // Attempt to issue a UCAN that WIDENS beyond the manifest's {A,B}.
    //   let widening_grant = AuthorizationGrant::issue_unchecked(
    //       &alice, &plugin_did,
    //       &Scope::RestrictedSelector(restricted_spec_for(&[A, B, C])),
    //   );
    //
    //   // Two acceptable rejection points (both must hold individually):
    //   //   (a) at grant-issuance time (chain-validator rejects);
    //   //   (b) at access-time for any row in C (OutOfScope).
    //
    //   // For (b) — the safety net even if (a) is bypassed:
    //   let req_c = engine.plugin_request(&plugin_did, &widening_grant, &row_in_C);
    //   assert!(matches!(req_c, Err(EngineError::OutOfScope(_)) | Err(EngineError::ChainNotNarrowing(_))),
    //           "§4-C non-widening: a UCAN naming C cannot grant access \
    //            to C if the manifest envelope is {A,B}. Spike H+1.1 R1 \
    //            ratification: chains MUST narrow, never widen.");
    //
    panic!(
        "RED-PHASE: §4-C adversarial non-widening — chain-validator \
         rejects widening AND manifest envelope holds even if chain \
         check bypassed. Un-ignore at G-CORE-3b landing."
    );
}

/// §4-C ARM 3 — Composition of revocations: revoking the UCAN cuts
/// the plugin's effective scope to ∅ (no access), even if the
/// manifest still nominally permits A,B; revoking the manifest
/// (uninstall) ALSO cuts to ∅, even if the UCAN still nominally
/// scopes B. This pins that EITHER revocation suffices — the
/// intersection structure means neither is sufficient alone.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3 + G-CORE-7 + G-CORE-8 §4.36 production rechecker landing."]
fn either_manifest_uninstall_or_ucan_revoke_collapses_plugin_effective_scope_to_empty() {
    // Body intent at un-ignore: install plugin, grant UCAN, verify
    // {B} access works, revoke UCAN, verify {B} access fails;
    // re-grant UCAN, verify {B} works again, UNINSTALL plugin, verify
    // {B} access fails (manifest envelope gone). Either-suffices
    // property.
    panic!(
        "RED-PHASE: §4-C either-revoke-collapses-scope. Un-ignore at \
         the composition wave. Would-FAIL if a future agent ships a \
         leaky uninstall (UCAN-grant-survives-uninstall path)."
    );
}
