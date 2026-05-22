//! Phase-4-Meta-Core — R3-W5 — Cross-wave integration test §4-A:
//! G-CORE-3 × G-CORE-1 — SubgraphSpec read scoped to DID X cannot leak
//! from DID Y partition. The §4-A row's load-bearing claim is that the
//! partition isolation (#989 namespace_did) fires BEFORE the per-Node
//! AEAD attempt, so a SubgraphSpec walker invoked under
//! `WriteContext::namespace_did=X` returns `NotFound` for any Node
//! whose ciphertext_cid lives in Y's partition, even if X's
//! AuthorizationGrant nominally lists that root. **Composes C1
//! authority-isolation + C2 confidentiality-isolation** at the engine
//! integration boundary — distinct from W3's per-crate tests in
//! `crates/benten-graph/tests/tf3d_partition_before_crypto.rs` (which
//! pins the same property at the graph-crate unit-level; HARD §3.5i
//! disjointness — different crate, different module surface).
//!
//! ============================================================================
//! RED-PHASE STATUS
//! ============================================================================
//!
//! Every `#[test]` here carries `#[ignore = "RED-PHASE: un-ignore at
//! G-CORE-3 + G-CORE-1 composition wave (after G-CORE-3d two-CID
//! mapping + per-Node AEAD lands)"]`. §3.6e — closing-wave checklist
//! un-ignores; reviewer verifies landing-status not spec-pin presence.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD c9c11c56 — R3-W5 §3.5n pass)
//! ============================================================================
//!
//!   * G-CORE-1 (#989) HAS LANDED: `WriteContext::namespace_did:
//!     Option<Cid>` ships at `crates/benten-graph/src/lib.rs:894+`
//!     (verified). `ScopedBackend` per-DID partition routing wired
//!     through `put_node_with_context`.
//!   * G-CORE-3 (#1301 + 3a-w-d-e-f) HAS NOT YET LANDED. There is NO
//!     `SubgraphSpec` Subgraph-walker on the engine; NO two-CID
//!     mapping; NO per-Node AEAD. The §4-A composition requires BOTH
//!     halves to even exercise. So this file is RED purely on the
//!     missing G-CORE-3 half.
//!
//! ============================================================================
//! WHY A CROSS-WAVE PIN (and not subsumed by W2/W3 per-crate pins)
//! ============================================================================
//!
//! W2 owns the SubgraphSpec walker shape (caps/walker contains() +
//! BFS-canonical-path). W3 owns the graph-AEAD per-Node + two-CID
//! mapping. NEITHER per-wave R3 author surfaces the cross-DID-with-
//! SubgraphSpec composition because each owns only one half of the
//! load-bearing seam. The §4-A row exists precisely to test the
//! C1+C2 composition (Inv-11 strengthening: namespace_did partition
//! + per-Node AEAD with K_principal-rooted key derivation). Engine
//! integration tests are the right home (per R2 §4 "lives at
//! integration-test level (recommend `crates/benten-engine/tests/`")).
//!
//! ============================================================================
//! §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
//! ============================================================================
//!
//!  1. §3.5b HARDENED (pim-1) — tests-only here; G-CORE-3d/3e
//!     implementer sweeps SECURITY-POSTURE.md §"Inv-11 under per-Node
//!     AEAD" + ENGINE-SPEC.md:312 trusted-engine-qualifier before push.
//!  2. §3.6b + sub-rule 4 (pim-2) — SPECIFIC arm = partition routing
//!     wins BEFORE crypto attempt (i.e. cross-DID gives NotFound, not
//!     "AEAD error"); OBSERVABLE = the typed NotFound error surface;
//!     WOULD-FAIL if a future agent inverts the layer order (crypto
//!     first, partition second → leak path would surface "AEAD-error"
//!     for a Y-Node accessed from X's view, signalling Y's existence).
//!  3. §3.6e (pim-12) — RED-PHASE staged-pin; un-ignore at G-CORE-3
//!     composition wave.
//!  4. §3.6f (pim-18) — production call-site = the future
//!     `Engine::walk_share_scope(grant)` Subgraph-walker entry +
//!     `Engine::read_node_as(principal, cid)` (Class B β; SHIPPED at
//!     PR #184); substantive body exercises the engine end-to-end (not
//!     a constructibility/grep assertion).
//!  5. §3.13 — per-test-static decomposition: each test instantiates
//!     its own `Engine` + writes its own Nodes under distinct
//!     namespace_did values; NO shared static under the parallel
//!     `cargo nextest` runner (CRITICAL — partition-isolation tests
//!     are the canonical §3.13 risk surface per the R3 brief).
//!  6. §3.5g — no new ErrorCode minted here (re-uses NotFound +
//!     UCAN/Scope error variants the W2 + W3 implementers will mint).
//!  7. §3.5n — R3-W5 ground-truth-verified G-CORE-1 landed; G-CORE-3
//!     NOT landed; the engine has `Engine::read_node_as` but no
//!     `walk_share_scope` yet. Test asserts the composition predicate
//!     against the FUTURE surface.
//!
//! Pin source: R2-test-landscape.md §4-A row + plan §1.A.FROZEN item
//! 15 sub-clauses (g) two-CID mapping + (h) walker in `benten_core` +
//! the C1+C2 composition C1+C2 exit obligations.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

/// §4-A ARM 1 — A SubgraphSpec walker invoked under
/// `WriteContext::namespace_did=X` returns NO Nodes from Y's
/// partition, even if Y-Nodes' CIDs appear in the grant's roots.
///
/// Would-FAIL if a future agent rewires the order so that the AEAD
/// attempt happens BEFORE the partition lookup (then a cross-DID
/// access would surface a typed AEAD-error revealing Y-Node existence,
/// instead of the structurally-required NotFound). The §4-A invariant
/// is: partition isolation FIRES FIRST; crypto is the secondary layer.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3 + G-CORE-1 composition wave (after G-CORE-3d two-CID mapping + walker land; §3.6e). r2-test-landscape §4-A; plan §1.A.FROZEN item 15 (g)+(h)."]
fn subgraphspec_walker_under_did_x_returns_notfound_for_did_y_nodes_even_with_y_cid_in_roots() {
    // Body intent at un-ignore (this file is RED at HEAD because the
    // engine has no `walk_share_scope` and no two-CID mapping yet):
    //
    //   let mut engine_x = Engine::test_native_with_namespace(did_x);
    //   let mut engine_y = Engine::test_native_with_namespace(did_y);
    //   // Alice (DID Y) writes a Recipe Subgraph under Y's partition.
    //   let recipe_root_cid = engine_y.put_subgraph_under_principal(&recipe);
    //   // Construct a SubgraphSpec listing recipe_root_cid as roots,
    //   // wrap in an AuthorizationGrant audience=bob_did from
    //   // alice_did, sign, deliver to Bob (DID X)'s engine.
    //   let grant = AuthorizationGrant::issue(alice, &bob_did, &spec);
    //   // Bob's engine walks the spec UNDER his own namespace_did=X.
    //   let result = engine_x.walk_share_scope(&grant);
    //   // The §4-A invariant: even though the grant is structurally
    //   // valid (signature + scope check pass), no Y-partition Nodes
    //   // surface to X's view; the walk returns an EMPTY set OR a
    //   // typed NotFound — NEVER a typed AEAD error (which would
    //   // signal Y's existence).
    //   assert_eq!(result.nodes_yielded(), 0);
    //   // And the error path, if any, is NotFound — partition first.
    //   for err in result.errors() {
    //       assert!(matches!(err, EngineError::NotFound(_)),
    //               "partition isolation MUST fire BEFORE AEAD attempt; \
    //                an AEAD-error would leak Y-Node existence to X");
    //   }
    //
    // RED-PHASE: at HEAD the engine has no walk_share_scope / no
    // two-CID mapping, so this assertion CANNOT yet compile against
    // production surfaces. The wave that lands G-CORE-3d (two-CID
    // mapping) + G-CORE-3w (walker) + G-CORE-3e (sync handler) will
    // mint the production calls; this test's un-ignore body wires
    // them. WOULD-FAIL if a future agent ships a walker that calls
    // crypto BEFORE the partition lookup.
    panic!(
        "RED-PHASE: §4-A G-CORE-3×G-CORE-1 partition-before-crypto \
         composition test. At synced HEAD G-CORE-1 (#989 \
         namespace_did) is shipped but G-CORE-3 (walker + two-CID + \
         per-Node AEAD) is NOT. Un-ignore at the G-CORE-3 closing wave; \
         wire the body against the produced `Engine::walk_share_scope` \
         + production AuthorizationGrant surfaces. This pin's failure \
         mode covers two future regressions: (a) walker fails to thread \
         namespace_did into reads; (b) crypto layer runs before \
         partition lookup, leaking Y-existence to X via AEAD-error \
         observability."
    );
}

/// §4-A ARM 2 — Adversarial: a DropBundle of Y's Recipes installed
/// under X's namespace fails. EITHER the AEAD authentication fails
/// (per-Node key K(N) derived from K_principal_Y not derivable by X)
/// OR partition routing rejects the install at the engine boundary.
/// What MUST NOT happen: silent acceptance + cross-DID Y-data
/// surfacing in X's view.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3f (Drop bundle) + G-CORE-3 composition wave. r2-test-landscape §4-A adversarial."]
fn dropbundle_of_y_recipes_installed_under_x_partition_fails_either_aead_or_partition_route() {
    // Body intent at un-ignore:
    //
    //   let drop_y = DropBundle::produce(&engine_y, &spec_y, audience=carol);
    //   // Carol's engine has its own namespace_did=carol_did, NOT
    //   // Y's, NOT X's. Carol's engine cannot derive Y's K_principal.
    //   let install_result = engine_carol.install_drop_bundle(&drop_y);
    //   // Two acceptable failure modes:
    //   match install_result {
    //       Err(EngineError::AeadAuthenticationFailed(_)) => { /* OK */ }
    //       Err(EngineError::PartitionRouteRejected(_)) => { /* OK */ }
    //       // SILENT acceptance OR a return that surfaces Y-bytes under
    //       // carol_did's partition: HARD FAIL.
    //       _ => panic!("§4-A: a Y-DropBundle MUST NOT silently install \
    //                    under a non-Y partition (Inv-11 strengthening: \
    //                    namespace_did partition + per-Node AEAD with \
    //                    K_principal-rooted key derivation)"),
    //   }
    //
    // RED at HEAD: G-CORE-3f (benten-drop crate) does not exist yet;
    // DropBundle / install_drop_bundle are unbuilt.
    panic!(
        "RED-PHASE: §4-A adversarial — cross-principal DropBundle \
         install fails at AEAD layer (cannot derive Y's K_principal) \
         OR at partition route (refuses cross-DID install). Un-ignore \
         at G-CORE-3f (benten-drop) landing wave."
    );
}

/// §4-A ARM 3 — Composition-direction symmetry: the same partition-
/// before-crypto layering holds for BOTH read direction (walker
/// returns NotFound) AND write direction (writes under X's namespace
/// cannot inadvertently land in Y's partition even if a poisoned
/// AuthorizationGrant claims X has Y-scope). The asymmetric (read
/// vs write) handling is what Inv-13 + Inv-11 jointly enforce; this
/// arm pins the engine integration boundary on both directions.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3 composition wave + G-CORE-8 §4.23 write-boundary chain validator. r2-test-landscape §4-A + plan C1 exit obligation."]
fn partition_isolation_holds_symmetric_under_read_and_write_paths_with_subgraphspec_in_scope() {
    // Body intent at un-ignore:
    //
    //   // (write half) — engine_x.put_subgraph_with_grant(&forged_grant_naming_y);
    //   // expects: write rejected — either grant rejected at §4.23
    //   // (chain validator: chain does not terminate at user-root for
    //   // the claimed scope) OR partition route refuses (the write
    //   // payload's declared namespace_did=Y disagrees with engine_x's
    //   // configured namespace_did=X).
    //   //
    //   // (read half) — engine_x.walk_share_scope(&forged_grant_for_y);
    //   // expects: walker returns NotFound / empty (per ARM 1).
    //
    //   // The composition predicate: NEITHER direction allows X to
    //   // see/touch Y's partition bytes via SubgraphSpec routing,
    //   // regardless of what a (potentially malicious) grant claims.
    //
    // RED at HEAD: G-CORE-3 walker + G-CORE-8 §4.23 not landed.
    panic!(
        "RED-PHASE: §4-A read+write symmetry — partition-before-crypto \
         + chain-validator-before-partition-touch hold on BOTH \
         directions. Un-ignore at G-CORE-3 composition wave + G-CORE-8 \
         §4.23 landing."
    );
}
