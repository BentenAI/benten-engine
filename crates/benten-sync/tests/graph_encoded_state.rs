//! G16-A canary scope pin: `benten-sync` introduces NO persistent
//! state (cag-2 + cag-r4-3 architectural floor).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-A row
//!   `benten_sync_persistent_state_graph_encoded`.
//! - `cag-2` (Code-as-Graph: persistent state is encoded as graph
//!   Nodes/Edges, not as opaque blobs alongside the graph).
//! - `cag-r4-3` MAJOR (per-shape granularity for sync-cursor +
//!   grant-cross-reference).
//!
//! ## G16-A canary scope vs G16-B/D persistent-state shapes
//!
//! G16-A canary scope ships ONLY transport core (iroh QUIC +
//! handshake wire-format struct + peer-id derivation). It introduces
//! NO persistent state — peer rosters, atrium membership,
//! sync-cursor HLC checkpoints, grant-cross-reference, and
//! handler-version Anchor pointers all land at G16-B/C/D
//! wave-6b. The cag-2 + cag-r4-3 enforcement therefore runs at the
//! wave-6b implementer's scope: `benten-sync` will at no point
//! between G16-A landing and G16-B/C/D landing carry side-table
//! redb keys with no graph-side handle.
//!
//! ## Floor-assertion (this file's G16-A pin)
//!
//! The `benten_sync_introduces_no_persistent_state_at_g16_a_canary_scope`
//! pin below is the load-bearing G16-A assertion: it walks the
//! `benten-sync` source surface + asserts NO `KVBackend` /
//! `redb::Database` / persistence-API import appears. This defends
//! against the failure shape where G16-A drifts into persistent state
//! (which would defeat the cag-2/cag-r4-3 wave-6b pin coverage).
//!
//! ## BELONGS-NAMED-NOW per HARD RULE rule-12 (b) — wave-6b destinations
//!
//! The following per-shape cag-r4-3 pins remain `#[ignore]`'d at G16-A
//! landing because the persistent-state shapes don't yet exist; each
//! is named-NOW to its specific G16-B/G16-D wave-6b implementer
//! destination per pim-4 §3.10 wave-paired closure pattern:
//!
//! - `benten_sync_persistent_state_graph_encoded` →
//!   G16-B wave-6b (Atrium-membership shape) +
//!   G16-D wave-6b (peer-roster shape).
//! - `atrium_sync_cursor_persisted_as_graph_node_keyed_by_peer_did_zone` →
//!   G16-B wave-6b (sync-cursor HLC checkpoints land at Loro CRDT
//!   wiring per D-PHASE-3-22).
//! - `atrium_grant_cross_reference_via_graph_edge_not_side_table` →
//!   G16-D wave-6b (UCAN-grant-scoped-to-Atrium edges land at the
//!   handshake protocol body).
//!
//! Each `#[ignore]` rationale below names the destination explicitly.

#![allow(clippy::unwrap_used)]

#[test]
fn benten_sync_introduces_no_persistent_state_at_g16_a_canary_scope() {
    // cag-2 + cag-r4-3 floor pin. G16-A canary scope is transport-only
    // — no `KVBackend`, no `redb::Database`, no `benten-graph`
    // backend wiring should appear in the `benten-sync` src tree at
    // canary landing. Walk the src tree + assert.
    //
    // OBSERVABLE consequence: a future PR that drifts G16-A scope
    // into persistent state (e.g. an inadvertent `benten-graph` dep
    // + KVBackend.put() call site) fails this pin loudly. The wave-6b
    // implementers (G16-B Loro, G16-C MST, G16-D handshake) un-skip
    // the per-shape cag-r4-3 pins as their persistent state lands;
    // until then, the canary surface stays clean.
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let manifest = std::fs::read_to_string(crate_root.join("Cargo.toml")).expect("read Cargo.toml");

    // No `benten-graph` dep at G16-A canary scope. (G16-B/C/D wave-6b
    // adds it for the persistent-state surfaces.)
    assert!(
        !manifest.contains("benten-graph"),
        "G16-A canary scope MUST NOT depend on benten-graph yet. \
         The benten-graph dep adds at G16-B/C/D wave-6b alongside the \
         persistent-state surfaces (Loro CRDT / MST diff / peer rosters). \
         A drift here defeats the cag-2/cag-r4-3 per-shape pin coverage \
         that the wave-6b implementers wire."
    );

    // No redb / KVBackend imports in the src tree.
    let mut found_persistence_import = None;
    for entry in walk_files(&src_dir) {
        let contents = std::fs::read_to_string(&entry).expect("read src file");
        for forbidden in &["use redb", "redb::Database", "KVBackend"] {
            if contents.contains(forbidden) {
                found_persistence_import = Some((entry.clone(), *forbidden));
                break;
            }
        }
    }
    assert!(
        found_persistence_import.is_none(),
        "G16-A canary scope MUST NOT import redb / KVBackend. \
         Found: {found_persistence_import:?}. The persistent-state \
         surfaces land at G16-B/C/D wave-6b — until then, the canary \
         transport surface stays free of persistence concerns."
    );
}

fn walk_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
    out
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — cag-2 persistent-state graph-encoded pin. G16-B wave-6b SHIPPED at PR #155; G16-D wave-6b SHIPPED at PR #163. Test body panic!() pins the load-bearing claim that atrium membership / sync-cursor / peer-roster persistent state is queryable through standard graph + IVM surfaces (Charter 4 symmetry); un-ignore at the §7.3.D `benten-sync` test-bodies cluster landing wave (cross-process atrium driver scaffolding) per Wave-E rationale-only sweep."]
fn benten_sync_persistent_state_graph_encoded() {
    // cag-2 pin. G16-B wave-6b lands the Atrium-membership shape +
    // sync-cursor (the load-bearing Loro CRDT persistent state); G16-D
    // wave-6b lands the peer-roster shape (handshake-completion
    // adds peers to the local roster).
    //
    // OBSERVABLE consequence (when un-ignored at wave-6b): atrium
    // membership / sync cursors / peer rosters are queryable
    // through standard graph + IVM surfaces; Code-as-Graph symmetry
    // preserved. Defends against the architectural drift toward
    // "side-table state alongside the graph" per Charter 4.
    panic!(
        "G16-B / G16-D wave-6b wires graph-encoded persistent atrium state + IVM view subscription"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — cag-r4-3 MAJOR sync-cursor graph-encoding pin. G16-B wave-6b SHIPPED at PR #155; test body pins HLC-checkpoint-per-peer-per-zone as graph Node keyed by (peer_did, zone) (NOT KVBackend.put with composite key — Charter 4 per-shape granularity); un-ignore at the §7.3.D `benten-sync` test-bodies cluster landing wave per Wave-E rationale-only sweep."]
fn atrium_sync_cursor_persisted_as_graph_node_keyed_by_peer_did_zone() {
    // cag-r4-3 MAJOR pin (Charter 4 per-shape granularity for
    // sync-cursor). HLC checkpoints per-peer per-zone are the
    // load-bearing state for Loro CRDT replay (D-PHASE-3-22). The
    // cursor MUST be a graph Node keyed by (peer_did, zone) — NOT
    // a KVBackend.put() with a composite key.
    //
    // G16-B wave-6b (Loro CRDT integration) lands this pin per
    // pim-4 §3.10 wave-paired closure.
    panic!(
        "G16-B wave-6b wires sync-cursor graph-encoding pin: Node label `sync:cursor` + \
         structured properties (peer_did/zone/hlc_checkpoint) + IVM-view-subscribable"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — cag-r4-3 MAJOR grant-cross-reference graph-encoding pin. G16-D wave-6b SHIPPED at PR #163; test body pins UCAN grant scoping as graph Edge from grant Node to Atrium Node (NOT KVBackend.put with composite key — Charter 4 per-shape granularity); un-ignore at the §7.3.D `benten-sync` test-bodies cluster landing wave per Wave-E rationale-only sweep."]
fn atrium_grant_cross_reference_via_graph_edge_not_side_table() {
    // cag-r4-3 MAJOR pin (Charter 4 per-shape granularity for
    // grant-cross-reference). UCAN grants scoped to which Atriums
    // MUST be encoded as an Edge from the grant Node to the Atrium
    // Node — NOT as a KVBackend.put() with a composite key.
    //
    // G16-D wave-6b (handshake protocol body + UCAN-grant exchange)
    // lands this pin per pim-4 §3.10 wave-paired closure.
    panic!(
        "G16-D wave-6b wires grant-cross-reference graph-Edge pin: GRANT_SCOPED_TO_ATRIUM Edge \
         from UCAN grant Node to Atrium Node — NOT a side-table KV entry"
    );
}
