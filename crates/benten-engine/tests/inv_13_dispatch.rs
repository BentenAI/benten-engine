//! R3-C RED-PHASE pins for Inv-13 row-4 SPLIT (sync-replica + Inv-13
//! immutability dispatch; G16-B wave-6b; per r2-test-landscape §2.4
//! G16-B + §3.C + plan §3 G16-B row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `inv_13_row_4a_loro_merge_applicable_user_data_resolves_via_d_c_version_chain` +
//!   `inv_13_row_4b_system_zone_anchor_immutable_divergent_cid_rejects_with_e_sync_divergent_cid_rejected`.
//! - r2-test-landscape §3.C sync-replica + Inv-13 immutability
//!   dispatch (D-PHASE-3-23 + ds-4 row-4 split).
//! - plan §3 G16-B row line "Inv-13 row-4 SPLIT per ds-4: row-4a
//!   (Loro-merge-applicable user-data resolves via D-C version-chain
//!   pattern) vs row-4b (system-zone/Anchor-immutable rejects with
//!   E_SYNC_DIVERGENT_CID_REJECTED)".
//! - `ds-4` (row-4 SPLIT recommendation).
//! - `sec-r1-2` (system-zone immutability).
//! - `D-PHASE-3-23` (sync-replica + Inv-13 immutability dispatch
//!   resolution).
//!
//! ## Inv-13 row-4 SPLIT narrative
//!
//! Phase-1 Inv-13 row-4 had a single rule: "divergent-CID writes
//! reject". Phase-3 SPLITS this into two rules under ds-4:
//!
//! - **row-4a (Loro-merge-applicable user-data):** divergent CIDs
//!   between sync replicas trigger a Loro merge that produces a new
//!   Version Node via the D-C version-chain pattern. Old Versions
//!   remain queryable; CURRENT advances atomically.
//!
//! - **row-4b (system-zone / Anchor-immutable data):** divergent
//!   CIDs reject with `E_SYNC_DIVERGENT_CID_REJECTED`. System-zone
//!   facts (governance rules, zone definitions, capability
//!   delegations) MUST NOT silently merge — operators choose which
//!   side wins via explicit re-write.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale pointing to phase-3-backlog §7.3.D STALE-RATIONALE sweep #2; destination §3.1-followup multi-peer iroh sync (CLOSED at G16-B-E PR #160).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — Inv-13 row-4a Loro-merge-applicable resolves. G16-B wave-6b shipped Loro CRDT integration; test body pins Inv-13 row-4a dispatch contract; un-ignore at §3.1-followup landing (CLOSED at G16-B-E PR #160; test driver authoring tracked at next Phase-3-close orchestrator-direct fix-pass batch) per Wave-E rationale-only sweep."]
fn inv_13_row_4a_loro_merge_applicable_user_data_resolves_via_d_c_version_chain() {
    // ds-4 + D-PHASE-3-23 pin. G16-B implementer wires this against
    // a fixture where two peers under sync-replica trust write
    // divergent values into the SAME user-data zone:
    //
    //   let mut engine_a = test_engine_with_peer_did(peer_a);
    //   let mut engine_b = test_engine_with_peer_did(peer_b);
    //   let atrium_a = engine_a.open_atrium(shared_config()).unwrap();
    //   let atrium_b = engine_b.open_atrium(shared_config()).unwrap();
    //
    //   // peer_a + peer_b each write a different value to /zone/posts/p1
    //   engine_a.write_node_in_zone("/zone/posts", make_post_with_title("p1", "title-from-A")).unwrap();
    //   engine_b.write_node_in_zone("/zone/posts", make_post_with_title("p1", "title-from-B")).unwrap();
    //
    //   // Sync triggers divergent-CID detection; row-4a path:
    //   atrium_a.sync_subgraph("/zone/posts").unwrap();
    //   atrium_b.sync_subgraph("/zone/posts").unwrap();
    //
    //   // Row-4a: Loro merge resolves; new Version Node minted; both peers converge:
    //   let p1_on_a = engine_a.read_current_for_anchor_in_zone("/zone/posts", "p1").unwrap();
    //   let p1_on_b = engine_b.read_current_for_anchor_in_zone("/zone/posts", "p1").unwrap();
    //   assert_eq!(p1_on_a.cid(), p1_on_b.cid(), "row-4a: Loro merge resolves; same CID across peers");
    //   // The merged Version's AttributionFrame includes both peers:
    //   let frame = p1_on_a.attribution_frame();
    //   assert!(frame.contains_peer_did(&peer_a));
    //   assert!(frame.contains_peer_did(&peer_b));
    //
    // OBSERVABLE consequence: divergent-CID writes in user-data
    // zones trigger Loro merge per D-C; both peers converge on the
    // merged Version. Defends against the failure shape where Phase-1
    // Inv-13 row-4 would have rejected the writes.
    unimplemented!("G16-B wires Inv-13 row-4a Loro-merge-applicable resolution path");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — Inv-13 row-4b system-zone immutability. G16-B wave-6b shipped Loro integration; test body pins Inv-13 row-4b system-zone-write rejection; un-ignore at §3.1-followup landing per Wave-E rationale-only sweep."]
fn inv_13_row_4b_system_zone_anchor_immutable_divergent_cid_rejects_with_e_sync_divergent_cid_rejected()
 {
    // ds-4 + sec-r1-2 pin. System-zone / Anchor-immutable data is
    // EXEMPT from the row-4a Loro-merge-applicable resolution path.
    // Phase-3 row-4b: divergent CIDs reject with
    // `E_SYNC_DIVERGENT_CID_REJECTED`.
    //
    //   let mut engine_a = test_engine_with_peer_did(peer_a);
    //   let mut engine_b = test_engine_with_peer_did(peer_b);
    //   let atrium_a = engine_a.open_atrium(shared_config()).unwrap();
    //   let atrium_b = engine_b.open_atrium(shared_config()).unwrap();
    //
    //   // peer_a + peer_b each write a different governance rule
    //   // to /system/zone/governance/rule-1
    //   engine_a.write_node_in_zone("/system/zone/governance",
    //       make_governance_rule("rule-1", policy_a())).unwrap();
    //   engine_b.write_node_in_zone("/system/zone/governance",
    //       make_governance_rule("rule-1", policy_b())).unwrap();
    //
    //   // Sync triggers divergent-CID detection; row-4b path:
    //   let result = atrium_a.sync_subgraph("/system/zone/governance");
    //   match result {
    //       Err(e) if e.code() == ErrorCode::E_SYNC_DIVERGENT_CID_REJECTED => {}
    //       other => panic!("expected E_SYNC_DIVERGENT_CID_REJECTED, got {other:?}"),
    //   }
    //
    //   // Operators can resolve by explicit re-write (one peer or
    //   // the other adopts the other's rule):
    //   engine_a.write_node_in_zone("/system/zone/governance",
    //       make_governance_rule("rule-1", policy_b())).unwrap();
    //   atrium_a.sync_subgraph("/system/zone/governance").unwrap();
    //
    // OBSERVABLE consequence: divergent system-zone CIDs reject
    // loudly; operators must choose explicitly. Defends against
    // silent merges of governance/capability/zone-definition data
    // (sec-r1-2 named these as MUST-NOT-merge surfaces).
    unimplemented!("G16-B wires Inv-13 row-4b E_SYNC_DIVERGENT_CID_REJECTED for system-zone");
}
