//! R3-C RED-PHASE pins for D-C HYBRID Loro → Version-chain
//! (G16-B wave-6b; per r2-test-landscape §2.4 G16-B + §9 + plan §3
//! G16-B row + plan §1 deliverable + arch-r1-4 + D-PHASE-3-22).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `loro_merge_produces_new_version_node_via_anchor_version_chain` +
//!   `loro_merge_produces_new_version_node_in_anchor_chain` +
//!   `loro_merged_version_carries_peer_did_set_in_attribution_frame` +
//!   `anchor_current_pointer_advances_on_loro_merge` +
//!   `old_version_remain_queryable_after_loro_merge` +
//!   `loro_merge_attribution_frame_captures_contributing_peer_dids` +
//!   `loro_merged_node_is_graph_encoded_not_opaque_crdt_blob`.
//! - r2-test-landscape §9 D-PHASE-3-22 hybrid (iii) Loro→Version-chain.
//! - r2-test-landscape §3.B Loro CLR-1 cluster.
//! - plan §3 G16-B row "D-C HYBRID per arch-r1-4 / D-PHASE-3-22 RESOLVED".
//! - `arch-r1-4` (Loro merges via Anchor + Version + CURRENT pattern;
//!   no opaque CRDT blob in the storage layer).
//! - `cag-6` (Loro merged nodes are graph-encoded).
//!
//! ## D-C HYBRID narrative (Ben's D-C decision; D-PHASE-3-22 RESOLVED)
//!
//! Loro merges produce **new Version Nodes** via the existing
//! Anchor + Version + CURRENT pattern (Phase-1 shipped). The
//! AttributionFrame at the new Version captures the set of
//! contributing peer-DIDs (the peers whose writes participated in
//! the merge). The CURRENT pointer advances atomically. Old
//! Versions remain queryable for content-addressing replay.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b wires D-C HYBRID Loro→Version-chain"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-C / D-PHASE-3-22 / arch-r1-4 — Loro merge → new Version Node"]
fn loro_merge_produces_new_version_node_via_anchor_version_chain() {
    // D-C / D-PHASE-3-22 / arch-r1-4 LOAD-BEARING pin. G16-B
    // implementer wires this against the production merge arm:
    //
    //   let anchor_id = engine.create_anchor("post:p1");
    //   engine.write_version_under_anchor(anchor_id, props_v1, peer_a_did);
    //   engine.write_version_under_anchor(anchor_id, props_v2, peer_b_did);
    //   // peer_a + peer_b concurrently wrote; merge resolves:
    //   engine.atrium.merge_remote_change(remote_loro_change).unwrap();
    //
    //   // After merge, a NEW Version Node exists under the same Anchor:
    //   let versions = engine.list_versions_for_anchor(anchor_id);
    //   assert!(versions.len() >= 3, "Loro merge must mint a new Version Node");
    //   let merged = versions.last().unwrap();
    //   assert_eq!(merged.label(), "version");
    //   assert_eq!(merged.parent_anchor(), Some(anchor_id));
    //
    // OBSERVABLE consequence: a Loro merge produces a graph-visible
    // new Version Node under the existing Anchor. The Anchor +
    // Version + CURRENT pattern is preserved (no opaque CRDT blob
    // alongside the graph; the merge IS in the graph).
    unimplemented!(
        "G16-B wires Loro merge → new Version Node assertion against the production arm"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 hybrid (iii) — pin-rephrasing for cross-lens visibility"]
fn loro_merge_produces_new_version_node_in_anchor_chain() {
    // r2-test-landscape §3.B notes this is a "pin-rephrasing for
    // cross-lens visibility" of the prior pin. The assertion is
    // shape-equivalent but exercises the contract under a slightly
    // different fixture corpus to defend against test-fixture-only
    // false positives.
    //
    // G16-B implementer wires this against a multi-Anchor fixture:
    //
    //   let anchors: Vec<AnchorId> = (0..5).map(|i| engine.create_anchor(&format!("post:p{i}"))).collect();
    //   for &aid in &anchors {
    //       engine.write_version_under_anchor(aid, props_v1, peer_a_did);
    //       engine.write_version_under_anchor(aid, props_v2, peer_b_did);
    //   }
    //   apply_concurrent_loro_merge_across_atrium(&anchors).unwrap();
    //
    //   for &aid in &anchors {
    //       let versions = engine.list_versions_for_anchor(aid);
    //       assert!(versions.len() >= 3, "every Anchor must receive a merged Version Node");
    //   }
    //
    // OBSERVABLE consequence: the property holds across many
    // Anchors in the same Atrium under cross-cutting merge.
    unimplemented!("G16-B wires multi-Anchor Loro merge → new Version Node assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-C — AttributionFrame captures contributing peer-DIDs"]
fn loro_merge_attribution_frame_captures_contributing_peer_dids() {
    // D-C LOAD-BEARING pin. The AttributionFrame at the new Version
    // captures the SET of peer-DIDs whose writes participated in
    // the merge. This is what makes Loro merges auditable +
    // composable with UCAN attribution.
    //
    //   let anchor_id = engine.create_anchor("post:p1");
    //   engine.write_version_under_anchor(anchor_id, props_v1, peer_a_did);
    //   engine.write_version_under_anchor(anchor_id, props_v2, peer_b_did);
    //   engine.write_version_under_anchor(anchor_id, props_v3, peer_c_did);
    //
    //   apply_concurrent_loro_merge_across_atrium(&[anchor_id]).unwrap();
    //
    //   let versions = engine.list_versions_for_anchor(anchor_id);
    //   let merged = versions.last().unwrap();
    //   let frame = merged.attribution_frame();
    //   let contributing: BTreeSet<PeerDid> = frame.contributing_peer_dids().cloned().collect();
    //   assert_eq!(contributing, BTreeSet::from([peer_a_did, peer_b_did, peer_c_did]));
    //
    // OBSERVABLE consequence: the AttributionFrame for a merged
    // Version is a SET of peer-DIDs (3 or more), not a single
    // peer-DID. Defends against the failure shape where the merge
    // attributes only to the merging peer (which would lose
    // provenance).
    unimplemented!("G16-B wires AttributionFrame contributing-peer-DIDs assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-C — peer-DID set in AttributionFrame (variant)"]
fn loro_merged_version_carries_peer_did_set_in_attribution_frame() {
    // r2-test-landscape §3.B variant pin (cross-lens visibility for
    // the AttributionFrame contract). G16-B implementer wires this
    // against the AttributionFrame public surface:
    //
    //   let merged_version = run_concurrent_merge_fixture();
    //   let frame = merged_version.attribution_frame();
    //   // The set must include all contributing peers:
    //   assert!(frame.contains_peer_did(&peer_a_did));
    //   assert!(frame.contains_peer_did(&peer_b_did));
    //   // The set is a true set (no duplicates):
    //   let unique_dids: BTreeSet<_> = frame.contributing_peer_dids().cloned().collect();
    //   assert_eq!(unique_dids.len(), frame.contributing_peer_dids().count());
    //
    // OBSERVABLE consequence: the AttributionFrame surface
    // exposes peer-DIDs as a deduplicated set; defends against the
    // failure shape where multi-write-per-peer cases produce
    // duplicated DIDs.
    unimplemented!("G16-B wires AttributionFrame peer-DID set deduplication assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — CURRENT pointer advances atomically"]
fn anchor_current_pointer_advances_on_loro_merge() {
    // D-PHASE-3-22 pin. After a Loro merge mints a new Version Node,
    // the Anchor's CURRENT pointer advances to the merged Version.
    // The advance is atomic (no observer sees CURRENT pointing at
    // an in-flight intermediate state).
    //
    //   let anchor_id = engine.create_anchor("post:p1");
    //   engine.write_version_under_anchor(anchor_id, props_v1, peer_a_did);
    //   let pre_merge_current = engine.read_current_for_anchor(anchor_id).unwrap();
    //
    //   apply_concurrent_loro_merge_across_atrium(&[anchor_id]).unwrap();
    //
    //   let post_merge_current = engine.read_current_for_anchor(anchor_id).unwrap();
    //   assert_ne!(pre_merge_current.cid(), post_merge_current.cid());
    //   // The post-merge CURRENT is the merged Version, not a
    //   // pre-existing parent:
    //   assert_eq!(post_merge_current.label(), "version");
    //   assert!(post_merge_current.attribution_frame().contributing_peer_dids().count() >= 2);
    //
    // OBSERVABLE consequence: CURRENT pointer advance is observable
    // through the read API; defends against the failure shape where
    // Loro merge produces a Version but CURRENT is left at the
    // pre-merge state.
    unimplemented!("G16-B wires CURRENT-pointer-advance assertion under Loro merge");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-22 — old Versions remain queryable"]
fn old_version_remain_queryable_after_loro_merge() {
    // D-PHASE-3-22 + content-addressing preservation pin. After a
    // Loro merge advances CURRENT, the OLD Version Nodes remain
    // queryable by their CIDs — content-addressing replay is
    // preserved.
    //
    //   let anchor_id = engine.create_anchor("post:p1");
    //   let v1_cid = engine.write_version_under_anchor(anchor_id, props_v1, peer_a_did);
    //   let v2_cid = engine.write_version_under_anchor(anchor_id, props_v2, peer_b_did);
    //
    //   apply_concurrent_loro_merge_across_atrium(&[anchor_id]).unwrap();
    //
    //   // Old Versions still queryable by CID:
    //   let v1 = engine.read_node_by_cid(&v1_cid).unwrap();
    //   let v2 = engine.read_node_by_cid(&v2_cid).unwrap();
    //   assert_eq!(v1.property("title").unwrap(), props_v1.title);
    //   assert_eq!(v2.property("title").unwrap(), props_v2.title);
    //
    // OBSERVABLE consequence: post-merge state preserves access to
    // pre-merge Versions; defends against the failure shape where a
    // CRDT-blob-style storage would lose access to pre-merge state.
    unimplemented!("G16-B wires old-Versions-remain-queryable assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — cag-6 — merged node is graph-encoded"]
fn loro_merged_node_is_graph_encoded_not_opaque_crdt_blob() {
    // cag-6 architectural pin. The merged Version Node MUST be a
    // graph Node (queryable by label, indexable by IVM, traversable
    // via Edges) — NOT an opaque CRDT blob stored alongside the
    // graph.
    //
    //   let anchor_id = engine.create_anchor("post:p1");
    //   engine.write_version_under_anchor(anchor_id, props_v1, peer_a_did);
    //   engine.write_version_under_anchor(anchor_id, props_v2, peer_b_did);
    //   apply_concurrent_loro_merge_across_atrium(&[anchor_id]).unwrap();
    //
    //   let merged = engine.read_current_for_anchor(anchor_id).unwrap();
    //   // Merged is a graph Node:
    //   assert_eq!(merged.label(), "version");
    //   assert!(merged.property_keys().contains(&"title".to_string()));
    //   // It's discoverable through the standard label-pattern view:
    //   let view = engine.user_view("all_versions").materialize();
    //   assert!(view.rows().iter().any(|n| n.cid() == merged.cid()));
    //
    // OBSERVABLE consequence: the merged Version is just a Node;
    // every graph-side affordance applies (IVM, attribution,
    // traversal). Defends against an architectural drift where Loro
    // adds a parallel storage layer.
    unimplemented!("G16-B wires graph-encoded merged-Node assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — cag-r4-6 MINOR — SUBSCRIBE fires notification on affected zone after Loro merge"]
fn loro_merge_fires_subscribe_notification_on_affected_zone_per_charter_9() {
    // cag-r4-6 MINOR pin (Charter 9 — CRDT merges produce graph-Node
    // EVENTS reachable via standard engine surfaces). Closes the
    // notification-shape gap that ivm_view_subscribe_compose.rs
    // (R4-FP) covers IVM read-side composition but does NOT cross-
    // reference Loro merge as a producer of SUBSCRIBE events.
    //
    // The contract: a Loro-merged Version Node lands in the graph
    // (storage-shape pinned by `loro_merged_node_is_graph_encoded_*`)
    // AND fires SUBSCRIBE notifications to subscribers registered on
    // the affected zone — same ChangeEvent fan-out as a local write.
    // Without this assertion, a regression could route Loro merges
    // through a code path that bypasses the SUBSCRIBE registry.
    //
    // G16-B implementer wires this:
    //
    //   let anchor_id = engine.create_anchor("post:p1");
    //
    //   // Register SUBSCRIBE handler on the affected zone:
    //   let received: std::sync::Arc<std::sync::Mutex<Vec<benten_graph::ChangeEvent>>> =
    //       std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    //   let received_clone = received.clone();
    //   engine.subscribe_zone("/zone/posts", move |event| {
    //       received_clone.lock().unwrap().push(event.clone());
    //   }).unwrap();
    //
    //   // Set up concurrent peers + apply a cross-Atrium Loro merge:
    //   engine.write_version_under_anchor(anchor_id, props_v1, peer_a_did);
    //   engine.write_version_under_anchor(anchor_id, props_v2, peer_b_did);
    //   apply_concurrent_loro_merge_across_atrium(&[anchor_id]).unwrap();
    //
    //   // The merged Version Node fires a ChangeEvent referencing
    //   // its CID:
    //   let events = received.lock().unwrap();
    //   let merged = engine.read_current_for_anchor(anchor_id).unwrap();
    //   assert!(events.iter().any(|e| e.affected_cid() == merged.cid()),
    //       "SUBSCRIBE handler MUST receive a ChangeEvent referencing the \
    //        merged Version Node's CID after Loro merge per cag-r4-6 (Charter 9)");
    //
    //   // The event references the affected zone (so per-zone subscriptions filter):
    //   assert!(events.iter().any(|e| e.zone() == "/zone/posts"),
    //       "ChangeEvent from Loro merge MUST reference the affected zone per cag-r4-6");
    //
    // OBSERVABLE consequence: SUBSCRIBE handlers see Loro merges
    // through the standard ChangeEvent fan-out — same as local
    // writes. Defends against a regression where Loro merges route
    // through a code path that bypasses the SUBSCRIBE registry,
    // making remote merges silently invisible to local subscribers.
    unimplemented!(
        "G16-B wires Loro-merge → SUBSCRIBE-notification pin: ChangeEvent referencing \
         merged-Version-CID + affected-zone delivered to SUBSCRIBE handler per cag-r4-6 (Charter 9)"
    );
}
