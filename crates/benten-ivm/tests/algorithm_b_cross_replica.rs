//! R4-FP/R3-C RED-PHASE pin: cross-replica Algorithm-B + Loro CRDT
//! convergence (G15-B + G16-B; ivm-r4-1 BLOCKER closure).
//!
//! ## Pin sources
//!
//! - `ivm-r4-1` BLOCKER (R4 large-council Round 1 ivm-correctness
//!   lens). R1 ivm-major-4 named THREE concrete recommendations for
//!   the cross-replica IVM seam; R3 corpus did not land any. R4-FP
//!   closes via this dedicated pin file.
//! - R1 ivm-major-4 (cross-replica Algorithm-B-meets-Loro-CRDT).
//! - `D-PHASE-3-21` (User-view replication scope; option (iii) per
//!   Ben's D2 decision 2026-05-04 — content-addressed snapshots +
//!   UCAN-gated `host:atrium:publish_view_result` capability).
//!
//! ## What this pins (distinct from atrium_two_peer.rs)
//!
//! `tests/integration/atrium_two_peer.rs` exercises the end-to-end
//! sync path but its body has zero IVM-view convergence assertion.
//! THIS file pins the cross-replica IVM contract: same user-view
//! definition registered on two atrium peers; concurrent matching
//! writes; post-Loro-merge ChangeEvent stream feeds Algorithm-B
//! incremental kernel; final canonical-bytes equal across replicas.
//!
//! Sister test pin lives at `tests/integration/atrium_two_peer.rs`
//! for end-to-end driving; THIS pin asserts the IVM-side contract
//! decomposed for diagnosability.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale pointing to phase-3-backlog §7.3.D STALE-RATIONALE sweep #2; destination §3.1-followup multi-peer iroh sync (CLOSED at G16-B-E PR #160) + next Phase-3-close orchestrator-direct fix-pass batch.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — Algorithm B incremental state converges across two replicas after Loro merge. G15-A + G15-B + W9-T1 closed §5.1 IVM Algorithm B drift-detector + GenericKernel; G16-B wave-6b shipped Loro CRDT integration. Test body pins cross-replica convergence observable contract; un-ignore at §3.1-followup landing (CLOSED at G16-B-E PR #160; test driver authoring tracked at next Phase-3-close orchestrator-direct fix-pass batch) per Wave-E rationale-only sweep."]
fn algorithm_b_incremental_state_converges_across_two_replicas_after_loro_merge() {
    // ivm-r4-1 BLOCKER pin closing R1 ivm-major-4. G15-B + G16-B
    // implementers wire this:
    //
    //   use benten_engine::Engine;
    //   use benten_ivm::algorithm_b::Algorithm;
    //
    //   // Two engines under different peer-DIDs:
    //   let mut engine_a = test_engine(peer_a_did);
    //   let mut engine_b = test_engine(peer_b_did);
    //
    //   // Register the SAME user-view definition on both:
    //   let view_def = benten_ivm::view_definition::ViewDefinition::user(
    //       "custom:posts_by_author",
    //       LabelPattern::exact("post"),
    //       Projection::all_props(),
    //   );
    //   engine_a.register_user_view(&view_def).unwrap();
    //   engine_b.register_user_view(&view_def).unwrap();
    //
    //   // Both join the same Atrium:
    //   let atrium_a = engine_a.atrium(AtriumConfig { atrium_id: "test", invite }).join().await.unwrap();
    //   let atrium_b = engine_b.atrium(AtriumConfig { atrium_id: "test", invite }).join().await.unwrap();
    //
    //   // Concurrent matching writes on both:
    //   engine_a.write_node(make_post("p1", "title-from-a")).await.unwrap();
    //   engine_b.write_node(make_post("p2", "title-from-b")).await.unwrap();
    //
    //   // Sync to convergence — Loro merges produce new Version
    //   // Nodes (per D-C HYBRID); the post-merge ChangeEvent stream
    //   // feeds Algorithm-B incremental kernel:
    //   wait_for_atrium_convergence(&[&atrium_a, &atrium_b]).await;
    //
    //   // Materialize the user-view on each engine:
    //   let view_a = engine_a.materialize_user_view(&view_def.id()).unwrap();
    //   let view_b = engine_b.materialize_user_view(&view_def.id()).unwrap();
    //
    //   // ASSERTION 1: canonical-bytes-equal post-merge:
    //   assert_eq!(view_a.canonical_bytes(), view_b.canonical_bytes(),
    //       "post-Loro-merge user-view materialization MUST produce \
    //        identical canonical-bytes across replicas");
    //
    //   // ASSERTION 2: incremental-equals-rebuild on each side
    //   // (post-merge ChangeEvent stream feeding Algorithm-B
    //   // produces the same result as from-scratch rebuild from the
    //   // post-merge graph state):
    //   let view_a_rebuild = engine_a.rebuild_user_view(&view_def.id()).unwrap();
    //   let view_b_rebuild = engine_b.rebuild_user_view(&view_def.id()).unwrap();
    //   assert_eq!(view_a.canonical_bytes(), view_a_rebuild.canonical_bytes());
    //   assert_eq!(view_b.canonical_bytes(), view_b_rebuild.canonical_bytes());
    //
    //   // ASSERTION 3: both views contain BOTH peer's contributing
    //   // writes (not just one side's local writes):
    //   let titles_a: BTreeSet<_> = view_a.rows().iter()
    //       .map(|r| r.field("title").unwrap().to_string()).collect();
    //   assert!(titles_a.contains("title-from-a"));
    //   assert!(titles_a.contains("title-from-b"));
    //
    // OBSERVABLE consequence: the cross-replica IVM seam preserves
    // (a) byte-equal materialization across replicas post-merge,
    // (b) incremental-equals-rebuild on each side under post-merge
    // ChangeEvent streams, (c) both replicas observe each other's
    // contributing writes in the materialized view. Defends against
    // the failure shape where Algorithm-B incremental kernel +
    // Loro merge + replica boundary compose unsoundly. Closes
    // R1 ivm-major-4 + ivm-r4-1 BLOCKER.
    unimplemented!(
        "G15-B + G16-B wire cross-replica Algorithm-B + Loro merge convergence — \
         post-merge ChangeEvent stream into Algorithm-B incremental kernel + canonical-bytes byte-equal across replicas"
    );
}
