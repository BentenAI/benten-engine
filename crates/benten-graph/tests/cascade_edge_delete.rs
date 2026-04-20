//! Cascade edge delete (r6b-ivm-1 regression — R4b brief).
//!
//! When a Node is deleted, every Edge whose source or target references
//! that Node must also be deleted, and each cascade must surface as a
//! `ChangeKind::EdgeDeleted` event on the ChangeSubscriber fan-out.
//!
//! Prior to this fix, `Transaction::delete_node` dropped the Node alone —
//! the prototype bug the R5 MUST clause explicitly forbids regressing.
//! Views driven off edge events (governance inheritance, version current)
//! never saw the cascade and their derived state drifted silently.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use benten_core::{Edge, Node, Value};
use benten_graph::{ChangeEvent, ChangeKind, ChangeSubscriber, RedbBackend};
use tempfile::tempdir;

/// Subscriber that records every event it sees, in order.
struct RecordingSubscriber {
    events: Arc<Mutex<Vec<ChangeEvent>>>,
}

impl ChangeSubscriber for RecordingSubscriber {
    fn on_change(&self, event: &ChangeEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event.clone());
    }
}

fn labeled_node(label: &str, n: i64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("n".into(), Value::Int(n));
    Node::new(vec![label.to_string()], props)
}

/// The core cascade contract: creating A, B, C and wiring three edges
/// (A→B, A→C, B→A), then deleting A, must emit:
///
/// - three `EdgeDeleted` events (A→B, A→C, B→A — dedup: B→A shares no
///   endpoint with the outbound set, so it surfaces independently)
/// - one `Deleted` event for the Node itself
///
/// All four events must land on the subscriber before the transaction
/// returns — the cascade is atomic with the node delete.
#[test]
fn cascade_edge_delete_emits_events_for_all_referencing_edges() {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();

    // Seed three Nodes.
    let a = backend.put_node(&labeled_node("NodeA", 1)).unwrap();
    let b = backend.put_node(&labeled_node("NodeB", 2)).unwrap();
    let c = backend.put_node(&labeled_node("NodeC", 3)).unwrap();

    // Three edges: A→B, A→C, B→A.
    let e_ab = backend
        .put_edge(&Edge::new(a.clone(), b.clone(), "X".to_string(), None))
        .unwrap();
    let e_ac = backend
        .put_edge(&Edge::new(a.clone(), c.clone(), "Y".to_string(), None))
        .unwrap();
    let e_ba = backend
        .put_edge(&Edge::new(b.clone(), a.clone(), "Z".to_string(), None))
        .unwrap();

    // Register the recording subscriber AFTER setup so the fan-out only
    // captures the cascade batch.
    let events = Arc::new(Mutex::new(Vec::<ChangeEvent>::new()));
    let sub = Arc::new(RecordingSubscriber {
        events: Arc::clone(&events),
    });
    backend.register_subscriber(sub).unwrap();

    // Delete A inside a transaction — the closure returns Ok, the backend
    // commits, then fans ChangeEvents out to the recording subscriber.
    backend
        .transaction(|tx| {
            tx.delete_node(&a)?;
            Ok(())
        })
        .unwrap();

    let captured = events
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();

    // Categorize events by kind.
    let edge_deletes: Vec<&ChangeEvent> = captured
        .iter()
        .filter(|e| e.kind == ChangeKind::EdgeDeleted)
        .collect();
    let node_deletes: Vec<&ChangeEvent> = captured
        .iter()
        .filter(|e| e.kind == ChangeKind::Deleted)
        .collect();

    assert_eq!(
        node_deletes.len(),
        1,
        "exactly one Node Deleted event; got {captured:?}",
    );
    assert_eq!(
        node_deletes[0].cid, a,
        "Node Deleted event must carry the deleted Node's CID",
    );

    assert_eq!(
        edge_deletes.len(),
        3,
        "three EdgeDeleted events — one per cascaded edge (A→B, A→C, B→A); got {captured:?}",
    );

    // Every cascaded edge's CID must appear as an EdgeDeleted event.
    let deleted_edge_cids: std::collections::BTreeSet<_> =
        edge_deletes.iter().map(|e| e.cid.clone()).collect();
    assert!(
        deleted_edge_cids.contains(&e_ab),
        "A→B cascade missing from fan-out",
    );
    assert!(
        deleted_edge_cids.contains(&e_ac),
        "A→C cascade missing from fan-out",
    );
    assert!(
        deleted_edge_cids.contains(&e_ba),
        "B→A cascade missing from fan-out",
    );

    // Post-cascade state: no edge referencing A survives the commit.
    assert!(
        backend.edges_from(&a).unwrap().is_empty(),
        "edges_from(A) must be empty after cascade",
    );
    assert!(
        backend.edges_to(&a).unwrap().is_empty(),
        "edges_to(A) must be empty after cascade",
    );
}

/// Self-loop edges (source == target == deleted Node) are cascaded
/// exactly once — the cascade helper dedupes across the `es:` and `et:`
/// prefix scans via a BTreeSet.
#[test]
fn cascade_edge_delete_dedupes_self_loop() {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();

    let a = backend.put_node(&labeled_node("NodeA", 1)).unwrap();
    let loop_edge = backend
        .put_edge(&Edge::new(a.clone(), a.clone(), "self".to_string(), None))
        .unwrap();

    let events = Arc::new(Mutex::new(Vec::<ChangeEvent>::new()));
    let sub = Arc::new(RecordingSubscriber {
        events: Arc::clone(&events),
    });
    backend.register_subscriber(sub).unwrap();

    backend
        .transaction(|tx| {
            tx.delete_node(&a)?;
            Ok(())
        })
        .unwrap();

    let captured = events
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let edge_deletes: Vec<&ChangeEvent> = captured
        .iter()
        .filter(|e| e.kind == ChangeKind::EdgeDeleted && e.cid == loop_edge)
        .collect();
    assert_eq!(
        edge_deletes.len(),
        1,
        "self-loop cascade must fire exactly one EdgeDeleted event; got {captured:?}",
    );
}

/// Deleting a Node with no referencing edges must NOT emit spurious
/// EdgeDeleted events — the cascade is precise.
#[test]
fn node_delete_without_edges_emits_no_edge_events() {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();

    let lonely = backend.put_node(&labeled_node("Lonely", 0)).unwrap();

    let events = Arc::new(Mutex::new(Vec::<ChangeEvent>::new()));
    let sub = Arc::new(RecordingSubscriber {
        events: Arc::clone(&events),
    });
    backend.register_subscriber(sub).unwrap();

    backend
        .transaction(|tx| {
            tx.delete_node(&lonely)?;
            Ok(())
        })
        .unwrap();

    let captured = events
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(
        captured.iter().all(|e| e.kind != ChangeKind::EdgeDeleted),
        "no-edge Node delete must not emit EdgeDeleted events; got {captured:?}",
    );
    assert_eq!(
        captured
            .iter()
            .filter(|e| e.kind == ChangeKind::Deleted)
            .count(),
        1,
        "exactly one Node Deleted event",
    );
}
