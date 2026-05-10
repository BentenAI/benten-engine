//! Phase 1 R3 integration — Version-chain CURRENT pointer resolution.
//!
//! Append 5 versions to a single Anchor; at each step assert that View 5
//! (version_current) resolves `anchor -> current_version_cid` in O(1) and
//! that the pointer equals the most recently appended version.
//!
//! Exercises C6 (Anchor + NEXT_VERSION + CURRENT conventions in benten-core),
//! I7 (View 5 in benten-ivm), and N7 (engine system-zone API for versions).
//!
//! **Status:** FAILING until C6 + G5-C + G7 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;
use std::time::Instant;

fn version(n: u32) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("draft-v{n}")));
    props.insert("body".into(), Value::Text(format!("content-{n}")));
    Node::new(vec!["post".into()], props)
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — version-chain anchor-lifecycle. §1.5 Compromise #18 durable handler-version chain CLOSED at G14-C wave-4b (PR #110); Engine::create_anchor / append_version / read_current_version anchor-lifecycle APIs live at HEAD. Test body pins specific O(1) View-5 wire-through contract; un-ignore at §5.1-followup-a landing per Wave-E rationale-only sweep."]
fn version_current_o1_resolution_at_every_step() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let anchor = engine
        .create_anchor("post-anchor-1")
        .expect("anchor created");

    let mut last_cid = None;
    for n in 1..=5u32 {
        let cid = engine.append_version(&anchor, &version(n)).expect("append");
        last_cid = Some(cid);

        // Resolve via View 5 (version_current). Timing bound: < 50us hot cache.
        let t = Instant::now();
        let current = engine
            .read_current_version(&anchor)
            .expect("resolved")
            .expect("present");
        let elapsed = t.elapsed();

        assert_eq!(
            current, cid,
            "CURRENT must equal the most recently appended version after step {n}"
        );
        assert!(
            elapsed.as_micros() < 1_000,
            "version_current must be O(1); took {elapsed:?} at step {n}"
        );
    }

    // Full walk returns linear order v1..v5
    let history: Vec<_> = engine.walk_versions(&anchor).expect("walk").collect();
    assert_eq!(history.len(), 5, "chain has all 5 versions");
    assert_eq!(history.last(), last_cid.as_ref(), "last in walk == CURRENT");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — version-chain anchor-lifecycle synchronous View-5 updates. §1.5 Compromise #18 CLOSED at G14-C; anchor-lifecycle APIs live at HEAD. Test body pins specific synchronous View-5 update contract; un-ignore at §5.1-followup-a landing per Wave-E rationale-only sweep."]
fn version_current_updates_synchronously_with_append() {
    // Protects against a regression where View 5 lags append by one ChangeEvent
    // (visible to crud list consumers but invisible to version-chain callers).
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let anchor = engine.create_anchor("post-anchor-2").unwrap();

    for n in 1..=3u32 {
        let cid = engine.append_version(&anchor, &version(n)).unwrap();
        let resolved = engine.read_current_version(&anchor).unwrap();
        assert_eq!(
            resolved,
            Some(cid),
            "version_current must update synchronously with append at step {n}"
        );
    }
}
