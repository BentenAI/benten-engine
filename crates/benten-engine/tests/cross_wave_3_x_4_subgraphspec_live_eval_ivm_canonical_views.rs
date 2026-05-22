//! Phase-4-Meta-Core — R3-W5 — Cross-wave integration test §4-B:
//! G-CORE-3 × G-CORE-4 — SubgraphSpec live-eval correctly invalidates
//! on writes. The §4-B claim: Alice grants Bob a UCAN scoped to a
//! SubgraphSpec; Alice writes a new Recipe matching the spec; the
//! IVM CanonicalViews subscription emits a ChangeEvent the resolver
//! consumer hears; Bob's next request returns the new Recipe (sub-
//! graph SHAPES that evolve, NOT frozen snapshots — D-4M-R5 R-CRYPTO
//! resolver-evaluation model = live-per-request).
//!
//! ============================================================================
//! RED-PHASE STATUS
//! ============================================================================
//!
//! `#[ignore = "RED-PHASE: un-ignore at G-CORE-3 + G-CORE-4 composition
//! wave"]`. G-CORE-4 (D1 CanonicalViews A2 seam + IVM 5-arm + §4.24
//! materializer + §4.6 vocab) HAS LANDED at HEAD (PR #1311, verified at
//! main `c9c11c56`). G-CORE-3 has NOT — so this file is RED on the
//! missing G-CORE-3 walker + grant + resolver-eval half.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD c9c11c56)
//! ============================================================================
//!
//!   * G-CORE-4: `benten_ivm::CanonicalViews` registry-query type
//!     SHIPPED at PR #1311; materializer recursive walk into
//!     vocabulary edges SHIPPED; IVM 5-arm byte-equivalence SHIPPED.
//!   * G-CORE-3: SubgraphSpec walker + AuthorizationGrant + the
//!     live-per-request resolver eval are NOT yet built. The R-CRYPTO
//!     resolver-eval pin will land in `Engine::walk_share_scope`.
//!   * The §4-B invariant the composition tests: the resolver REUSES
//!     G-CORE-4's CanonicalViews subscription as its IVM-cache-
//!     invalidation seam (per plan §1.A.FROZEN item 15 sub-clause (j)
//!     "IVM-cache-invalidation seam reuses G-CORE-4's CanonicalViews
//!     subscription").
//!
//! ============================================================================
//! §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
//! ============================================================================
//!
//!  1. §3.5b HARDENED — G-CORE-3e implementer sweeps SECURITY-POSTURE.md
//!     §"Revocation reach" + ENGINE-SPEC.md resolver-eval section.
//!  2. §3.6b sub-rule 4 — SPECIFIC arm = NEW writes flow into Bob's
//!     accessible scope automatically (live-per-request); OBSERVABLE =
//!     Bob's second request returns the post-write Recipe; WOULD-FAIL
//!     if a future agent ships frozen-snapshot semantics (Bob would
//!     get the pre-write set on his second request).
//!  3. §3.6e — RED-PHASE staged-pin; un-ignore at G-CORE-3
//!     composition wave (specifically G-CORE-3e online path).
//!  4. §3.6f (pim-18) — production call-site = future
//!     `Engine::walk_share_scope(grant)` + the existing
//!     `benten_ivm::CanonicalViews::subscribe(...)`; substantive body
//!     drives both halves end-to-end through the engine.
//!  5. §3.13 — per-test-static decomposition: each test owns its own
//!     Engine pair (Alice + Bob); NO shared static.
//!  6. §3.5g — no new ErrorCode.
//!  7. §3.5n — R3-W5 verified G-CORE-4 CanonicalViews SHIPPED;
//!     G-CORE-3 not.
//!
//! Pin source: R2-test-landscape.md §4-B + plan §1.A.FROZEN item 15
//! sub-clause (j) live-per-request resolver-eval + D-4M-R5/R6
//! ratification rows.

#![allow(clippy::unwrap_used)]

/// §4-B ARM 1 — A SubgraphSpec UCAN-grant resolver-evaluates LIVE per
/// request: Alice writes a new Recipe matching the spec AFTER issuing
/// the grant; Bob's next request includes the new Recipe.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3 + G-CORE-4 composition wave (G-CORE-3e online path + G-CORE-4 CanonicalViews subscription)."]
fn subgraphspec_grant_returns_writes_made_after_issuance_live_per_request() {
    // Body intent at un-ignore:
    //
    //   let mut engine_alice = Engine::test_native_for_principal(alice);
    //   let mut engine_bob = Engine::test_native_for_principal(bob);
    //
    //   // Alice writes Recipe-1 matching the spec BEFORE the grant.
    //   let r1 = engine_alice.write_recipe("r1", &spec_topology);
    //   // Alice issues a SubgraphSpec UCAN to Bob.
    //   let grant_to_bob = engine_alice.issue_subgraphspec_grant(&bob_did, &spec);
    //   // Bob's first request — sees Recipe-1.
    //   let first = engine_bob.walk_share_scope(&grant_to_bob).recipes();
    //   assert!(first.contains(&r1), "pre-grant write visible at first request");
    //
    //   // Alice writes Recipe-2 AFTER the grant, also matching the spec.
    //   let r2 = engine_alice.write_recipe("r2", &spec_topology);
    //
    //   // §4-B invariant: Bob's NEXT request returns BOTH Recipes
    //   // (live-per-request semantics; the spec describes sub-graph
    //   // SHAPES that evolve, NOT frozen snapshots).
    //   let second = engine_bob.walk_share_scope(&grant_to_bob).recipes();
    //   assert!(second.contains(&r1) && second.contains(&r2),
    //           "§4-B: live-per-request semantics — post-grant writes \
    //            matching the spec MUST flow into Bob's accessible scope \
    //            automatically. Would-FAIL on frozen-snapshot impl.");
    //
    panic!(
        "RED-PHASE: §4-B G-CORE-3×G-CORE-4 live-per-request resolver \
         eval composition. At synced HEAD G-CORE-4 (CanonicalViews + \
         IVM 5-arm) is shipped; G-CORE-3 (walker + grant + resolver \
         live-eval) is NOT. Un-ignore at G-CORE-3e online-path wave; \
         wire the body against the produced walk_share_scope + \
         CanonicalViews-subscription IVM-cache seam. Would-FAIL on a \
         frozen-snapshot resolver impl (which is the wrong shape per \
         R-CRYPTO ratification)."
    );
}

/// §4-B ARM 2 — Adversarial / revocation-reach (R6): Alice deletes a
/// Recipe Bob previously decrypted; Bob's already-derived keys still
/// decrypt the local plaintext (forever-valid per §1.A.FROZEN item 15
/// sub-clause (i)), but his NEXT request returns "not in current
/// scope" (live-eval CUTS FORWARD; revocation cannot un-derive past
/// keys). This is the load-bearing revocation-reach asymmetry the
/// SECURITY-POSTURE.md section must document.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3 composition wave; un-ignore-couples with the SECURITY-POSTURE.md §\"Revocation reach\" doc section per pim-2 sub-rule-4 doc-coupling."]
fn revocation_reach_cuts_forward_decrypted_past_keys_remain_decryptable() {
    // Body intent at un-ignore:
    //
    //   let r1 = engine_alice.write_recipe("r1", ...);
    //   let grant = engine_alice.issue_subgraphspec_grant(&bob_did, &spec);
    //   // Bob fetches + decrypts r1 (key K(r1) materializes Bob-side).
    //   let r1_plain = engine_bob.walk_share_scope(&grant).decrypt(&r1);
    //   assert_eq!(r1_plain.label(), "Recipe");
    //
    //   // Alice revokes / deletes r1.
    //   engine_alice.revoke_recipe(&r1);
    //
    //   // Live-eval CUTS FORWARD: Bob's NEXT request no longer
    //   // includes r1 (or returns it as out-of-current-scope).
    //   let second = engine_bob.walk_share_scope(&grant);
    //   assert!(!second.recipes().contains(&r1),
    //           "live-per-request: post-revocation request does NOT \
    //            return r1 (current-scope check)");
    //
    //   // BUT Bob's already-derived K(r1) STILL DECRYPTS the local
    //   // ciphertext he has (key derivation is not retroactively
    //   // revoked; this is what the SECURITY-POSTURE.md revocation-
    //   // reach section documents).
    //   let still_decryptable = engine_bob.decrypt_with_already_derived_key(&r1, &cached_K_r1);
    //   assert_eq!(still_decryptable.label(), "Recipe",
    //              "revocation-reach R6: already-derived keys remain \
    //               valid forever; only future serves are cut. The \
    //               documented design constant (§1.A.FROZEN item 15 (i)).");
    //
    panic!(
        "RED-PHASE: §4-B revocation-reach asymmetry. At HEAD the walker \
         + revocation seam are unbuilt. Un-ignore at G-CORE-3 + couples \
         with SECURITY-POSTURE.md §\"Revocation reach\" doc section."
    );
}

/// §4-B ARM 3 — CanonicalViews subscription seam structurally exists
/// at HEAD (G-CORE-4 shipped). This pin asserts that the FUTURE
/// resolver IS WIRED TO IT (not a parallel re-implementation). A
/// post-wave grep over `walk_share_scope` impl asserts it calls
/// `CanonicalViews::subscribe`-shaped API for invalidation. This is
/// the §3.6f SHAPE-not-SUBSTANCE waiver path — the SUBSTANTIVE check
/// is the source-level wiring.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3e wave; this pin grep-asserts the wiring at landed-code time."]
fn resolver_reuses_canonical_views_subscription_not_a_parallel_reimplementation() {
    // Body intent at un-ignore (structural source-scan pattern, same
    // shape as `tf12_benten_engine_freeze_conformance_caps_no_regression_469.rs`):
    //
    //   let walk_share_scope_src = std::fs::read_to_string(
    //       "crates/benten-engine/src/engine_share_scope.rs"  // future file
    //   ).expect("walk_share_scope impl present post-G-CORE-3e");
    //   // §1.A.FROZEN item 15 (j) load-bearing: IVM-cache-invalidation
    //   // seam reuses G-CORE-4's CanonicalViews subscription.
    //   assert!(
    //       walk_share_scope_src.contains("CanonicalViews")
    //         || walk_share_scope_src.contains("canonical_views"),
    //       "§4-B + §1.A.FROZEN item 15 (j): the SubgraphSpec resolver \
    //        MUST reuse G-CORE-4's CanonicalViews subscription as its \
    //        IVM-cache-invalidation seam (NOT a parallel reimplementation). \
    //        Would-FAIL if G-CORE-3e ships its own ad-hoc invalidation."
    //   );
    //
    panic!(
        "RED-PHASE: §4-B resolver-reuses-CanonicalViews wiring assertion. \
         Un-ignore at G-CORE-3e landing; grep the live walk_share_scope \
         source for CanonicalViews reference. Would-FAIL on a parallel- \
         reimplementation (the wrong shape per §1.A.FROZEN item 15 (j))."
    );
}
