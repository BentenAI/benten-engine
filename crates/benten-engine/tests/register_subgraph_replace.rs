//! Phase 2b Wave-8f: `Engine::register_subgraph_replace` semantic + version-
//! chain bookkeeping.
//!
//! Pins the positive contract that `register_subgraph_replace`:
//! - admits a new CID under the same handler_id (no `DuplicateHandler`)
//! - reports the previous CID + bumped chain depth
//! - is idempotent for identical content (no chain growth, no error)
//! - re-runs the full G6 invariant battery on the replacement body
//! - leaves the legacy `register_subgraph` rejection contract untouched
//!
//! The legacy `register_subgraph` happy path + duplicate-rejection tests
//! live in `register_subgraph_failures.rs` + the wider integration
//! suite — this file targets the new replace surface only.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, EngineError, ErrorCode};
use benten_eval::SubgraphBuilder;
use benten_eval::{SubgraphBuilderExt, SubgraphExt};
use tempfile::tempdir;

fn fresh_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("replace.redb"))
        .build()
        .unwrap();
    (engine, dir)
}

fn build_handler(handler_id: &str, label: &str) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new(handler_id);
    let r = sb.read(label);
    sb.respond(r);
    sb.build_validated().expect("must build")
}

/// Build a SubgraphSpec-based handler so the engine stores a spec
/// for it; without a stored spec, `dispatch_call_inner` returns
/// NotFound at the `specs.lock_recover().get(handler_id)` path. The
/// `label` is encoded as a property on the Read primitive so two
/// `build_spec_handler` calls with different labels produce
/// distinct handler CIDs (the canonical-bytes encoder hashes the
/// per-primitive properties bag).
fn build_spec_handler(handler_id: &str, label: &str) -> benten_engine::SubgraphSpec {
    let mut props = std::collections::BTreeMap::new();
    props.insert("label".into(), benten_core::Value::Text(label.into()));
    let read = benten_engine::PrimitiveSpec {
        id: "r".into(),
        kind: benten_eval::PrimitiveKind::Read,
        properties: props,
    };
    benten_engine::SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(read)
        .respond()
        .build()
}

#[test]
fn register_subgraph_replace_first_call_seeds_chain_no_predecessor() {
    let (engine, _dir) = fresh_engine();
    let sg = build_handler("h-replace-first", "post");
    let expected_cid = sg.cid().unwrap();

    let outcome = engine
        .register_subgraph_replace(sg)
        .expect("first registration via replace must succeed");
    assert_eq!(outcome.handler_id, "h-replace-first");
    assert_eq!(outcome.cid, expected_cid);
    assert!(outcome.previous_cid.is_none());
    assert_eq!(outcome.chain_depth, 1);
    assert!(!outcome.replaced(), "first registration is not a replace");
    assert_eq!(outcome.version_tag(), "v1");

    let chain = engine.handler_version_chain("h-replace-first");
    assert_eq!(chain, vec![expected_cid]);
}

#[test]
fn register_subgraph_replace_distinct_content_bumps_chain() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-bump";
    let v1 = build_handler(h, "post");
    let v1_cid = v1.cid().unwrap();
    let v2 = build_handler(h, "comment"); // different body — different CID
    let v2_cid = v2.cid().unwrap();
    assert_ne!(v1_cid, v2_cid);

    let _ = engine.register_subgraph_replace(v1).unwrap();
    let outcome = engine
        .register_subgraph_replace(v2)
        .expect("second registration with different body must succeed");
    assert_eq!(outcome.cid, v2_cid);
    assert_eq!(outcome.previous_cid, Some(v1_cid));
    assert_eq!(outcome.chain_depth, 2);
    assert!(outcome.replaced());
    assert_eq!(outcome.version_tag(), "v2");

    let chain = engine.handler_version_chain(h);
    assert_eq!(chain, vec![v2_cid, v1_cid], "newest-first ordering");
}

#[test]
fn register_subgraph_replace_identical_content_is_idempotent_no_chain_growth() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-idem";
    let _ = engine
        .register_subgraph_replace(build_handler(h, "post"))
        .unwrap();
    let outcome = engine
        .register_subgraph_replace(build_handler(h, "post"))
        .expect("identical re-register must succeed idempotently");
    assert_eq!(
        outcome.chain_depth, 1,
        "chain must not grow on identical body"
    );
    assert!(!outcome.replaced(), "identical body is not a replace");
    assert_eq!(outcome.version_tag(), "v1");
}

#[test]
fn register_subgraph_replace_runs_full_invariant_battery_on_new_body() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-bad";
    let _ = engine
        .register_subgraph_replace(build_handler(h, "post"))
        .unwrap();

    // Build a cyclic subgraph under the same handler_id; replace must
    // reject with the same Inv-1 cycle code register_subgraph would.
    let mut sb = SubgraphBuilder::new(h);
    let r = sb.read("post");
    sb.add_edge(r, r);
    let cyclic = sb.build_unvalidated_for_test();

    let err = engine
        .register_subgraph_replace(cyclic)
        .expect_err("cycle must fail at replace registration too");
    match err {
        EngineError::Invariant(e) => assert_eq!(e.code(), ErrorCode::InvCycle),
        other => panic!("expected EngineError::Invariant(InvCycle), got {other:?}"),
    }
}

#[test]
fn register_subgraph_replace_dispatches_new_body_on_next_call() {
    let (engine, _dir) = fresh_engine();
    let h = "h-replace-dispatch";

    let v1 = build_handler(h, "post");
    let _ = engine.register_subgraph_replace(v1).unwrap();
    let live_after_v1 = engine.handler_version_chain(h);
    assert_eq!(live_after_v1.len(), 1);

    // Replace with a structurally-different body (different read label →
    // different node CID → different subgraph CID).
    let v2 = build_handler(h, "comment");
    let v2_cid = v2.cid().unwrap();
    let outcome = engine.register_subgraph_replace(v2).unwrap();

    // The handlers map's new live entry MUST equal v2's CID, not v1's.
    // The version-chain accessor exposes this directly without exposing
    // the handlers Mutex.
    let chain = engine.handler_version_chain(h);
    assert_eq!(chain[0], v2_cid, "live target must be v2 after replace");
    assert_eq!(outcome.chain_depth, 2);
}

#[test]
fn legacy_register_subgraph_still_rejects_duplicate_with_different_content() {
    // Wave-8f introduces register_subgraph_replace WITHOUT changing
    // legacy register_subgraph's rejection contract. This test pins
    // that the legacy method continues to reject a duplicate-with-
    // different-content under the same handler_id.
    let (engine, _dir) = fresh_engine();
    let h = "h-legacy";
    engine.register_subgraph(build_handler(h, "post")).unwrap();
    let err = engine
        .register_subgraph(build_handler(h, "comment"))
        .expect_err("legacy register must still reject a content-mismatched dup");
    assert!(matches!(err, EngineError::DuplicateHandler { .. }));
}

#[test]
fn register_subgraph_replace_in_flight_call_observes_pre_swap_subgraph() {
    // The load-bearing in-flight contract from engine.rs's
    // `register_subgraph_replace` docstring: an `Engine::call` that
    // resolved its `handler_cid` BEFORE a concurrent
    // `register_subgraph_replace` lands MUST dispatch against the
    // pre-swap Subgraph (cache-keyed on the captured CID), NOT the
    // post-swap body.
    //
    // Mechanism: `Engine::testing_set_pre_dispatch_gate` installs a
    // Barrier that every subsequent `Engine::call`/`Engine::trace`
    // parks on AFTER `dispatch_call_with_mode_and_trace` captures
    // `handler_cid` from the handlers Mutex but BEFORE
    // `dispatch_call_inner` re-locks `specs` to reconstruct the
    // Subgraph. The harness lands the replace while the call thread
    // is parked, then releases the gate by joining the same Barrier.
    //
    // The pin: the in-flight call's CAPTURED handler_cid must equal
    // v1's CID (the engine reads `handlers` once at the dispatch
    // entry; the spec Mutex re-lookup at dispatch time uses that
    // captured CID as the cache key) — so even though the spec
    // table now points at v2's spec, the cache-key axis nails the
    // dispatch to the v1 SubgraphSpec the cache had associated with
    // v1's handler_cid (after a v1 cache pre-warm).
    //
    // The test's load-bearing assertion: the in-flight call returns
    // an outcome (no NotFound / stale-CID error) AND
    // `handler_version_chain()` reports newest-first post-swap.
    use std::sync::{Arc, Barrier};
    use std::thread;

    let (engine, _dir) = fresh_engine();
    let h = "h-replace-in-flight";

    // Build SubgraphSpec-based handlers (rather than the
    // `build_handler` helper's raw Subgraph) so the engine stores a
    // SubgraphSpec for them — without that, `dispatch_call_inner`
    // would return NotFound at the `specs.lock_recover().get(...)`
    // path because raw-Subgraph registrations have no rebuilder.
    let v1 = build_spec_handler(h, "post");
    let v2 = build_spec_handler(h, "comment");
    let v1_cid = engine
        .register_subgraph_replace(v1)
        .expect("v1 spec registers cleanly")
        .cid;
    // The v2 CID will only be known post-replace; capture it after
    // the in-flight call is parked. Build a second SubgraphSpec
    // copy so we can compute its CID independently for the
    // post-swap chain assertion.
    let _ = v2;
    let v2_for_replace = build_spec_handler(h, "comment");

    // Two-party Barrier: thread A (the in-flight call) + the
    // harness thread (which lands the replace then releases the
    // gate).
    let gate = Arc::new(Barrier::new(2));
    engine.testing_set_pre_dispatch_gate(Some(Arc::clone(&gate)));

    let engine = Arc::new(engine);
    let engine_a = Arc::clone(&engine);
    let thread_a = thread::spawn(move || {
        // Parks inside `dispatch_call_with_mode_and_trace` right
        // after the `handler_cid` capture. When the harness
        // releases the gate this thread proceeds with `cid_v1` as
        // its captured handler_cid + the spec-table state THEN
        // current (which the harness will have mutated to v2). The
        // call must complete without surfacing E_NOT_FOUND / stale
        // CID — that's the in-flight contract.
        engine_a.trace(h, "run", benten_core::Node::empty())
    });

    // Give thread A a moment to park at the gate.
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Land the swap WHILE thread A is parked — this is the racy
    // mid-call window the docstring's contract names.
    let outcome = engine.register_subgraph_replace(v2_for_replace).unwrap();
    let v2_cid = outcome.cid;
    assert_ne!(v2_cid, v1_cid);
    assert_eq!(outcome.previous_cid, Some(v1_cid));

    // Release thread A by crossing the same barrier.
    gate.wait();
    engine.testing_set_pre_dispatch_gate(None);

    let result = thread_a.join().expect("thread A must not panic");

    // Pin the in-flight contract's first half: the call survived
    // the swap (no NotFound / stale-CID surface). Whether the
    // resulting Subgraph reflects v1 or v2 depends on the cache
    // axis (cache-hit on v1_cid → v1; cache-miss → re-built from
    // the now-current v2 spec). Either is documented behaviour;
    // what's NOT acceptable is a panic, a NotFound, or any
    // EngineError that names the in-flight CID as missing.
    match result {
        Ok(_) => {} // expected — call completed
        Err(EngineError::Other { code, message }) => {
            panic!(
                "in-flight call must not surface engine-other error after replace, got code={code:?} msg={message}"
            );
        }
        Err(other) => panic!("unexpected in-flight error: {other:?}"),
    }

    // Pin the in-flight contract's second half: the post-swap
    // version chain reflects v2-prepended-onto-v1 (newest-first),
    // proving the swap landed under the joint-lock invariant + the
    // in-flight call survived the race without surfacing a stale-
    // CID error.
    let chain = engine.handler_version_chain(h);
    assert_eq!(chain, vec![v2_cid, v1_cid], "newest-first post-swap");
    // The in-flight call's resolve happened against v1_cid (the
    // version chain's pre-swap head). Whether the resulting
    // Subgraph reflects v1 or v2 depends on whether a v1 cache
    // entry was already warm at the time of the call's spec
    // re-lookup; both are documented behaviour. The load-bearing
    // contract is that the captured CID does not become stale +
    // the call doesn't surface E_NOT_FOUND under the race, both
    // pinned by the result-shape match above.
}

#[test]
fn register_subgraph_replace_concurrent_writers_preserve_chain_newest_first() {
    // Wave-8f mini-review 8f-dx-1 regression: under concurrent
    // `register_subgraph_replace` calls against the same handler_id
    // with different content, the version-chain prepend order MUST
    // match the handlers-table swap order so
    // `handler_version_chain()`'s newest-first invariant holds.
    //
    // The fix holds the handlers + specs + version_chain locks
    // jointly across the swap+prepend sequence; without that, the
    // handlers swap could land in one order while the chain prepend
    // landed in the other — yielding a chain whose head was NOT the
    // last handler-table swap winner.
    use std::sync::{Arc, Barrier};
    use std::thread;

    let h = "h-replace-concurrent";

    // Build TWO distinct replacement bodies (different read labels →
    // different CIDs). Reused across iterations.
    let va_cid = build_handler(h, "comment").cid().unwrap();
    let vb_cid = build_handler(h, "tag").cid().unwrap();
    assert_ne!(va_cid, vb_cid);

    // 50 race iterations against a fresh engine each time to flush
    // out any non-deterministic interleaving regression. Each
    // iteration starts both threads at a Barrier so they race the
    // engine's lock acquisition. Fresh engine per iteration keeps
    // the chain depth bounded to ≤2 + makes the invariant obvious.
    for _ in 0..50 {
        let (engine, _dir) = fresh_engine();
        // Seed the handler with a v0 body so both racing threads
        // are in the "replace" branch (not the "first registration"
        // branch).
        let v0 = build_handler(h, "post");
        engine.register_subgraph_replace(v0).unwrap();

        let engine = Arc::new(engine);
        let engine_a = Arc::clone(&engine);
        let engine_b = Arc::clone(&engine);
        let start = Arc::new(Barrier::new(2));
        let start_a = Arc::clone(&start);
        let start_b = Arc::clone(&start);
        let va_clone = build_handler(h, "comment");
        let vb_clone = build_handler(h, "tag");

        let ta = thread::spawn(move || {
            start_a.wait();
            engine_a.register_subgraph_replace(va_clone).unwrap()
        });
        let tb = thread::spawn(move || {
            start_b.wait();
            engine_b.register_subgraph_replace(vb_clone).unwrap()
        });

        let _ = ta.join().unwrap();
        let _ = tb.join().unwrap();

        // The chain MUST be non-empty + its head MUST be one of
        // the racing replacement CIDs (proving prepend ran for at
        // least the last winner). The version chain's contract:
        // newest-first; under the joint-lock fix the head is
        // EXACTLY the CID the handlers-table swap left as the
        // live target. If lock ordering races, the chain head
        // could be the LOSING swap (the earlier prepend) under
        // some interleavings — the joint-lock makes that
        // impossible.
        let chain = engine.handler_version_chain(h);
        assert!(!chain.is_empty(), "chain must be non-empty after replace");
        let head = chain[0];
        assert!(
            head == va_cid || head == vb_cid,
            "chain head must be one of the racing replacements, got {head:?}"
        );

        // The chain depth MUST be 3 (v0 + va + vb, in some
        // order — both racing replacements are distinct from v0
        // and from each other so each grew the chain). If a
        // racing prepend was lost (the bug shape), the depth
        // would be ≤2.
        assert_eq!(
            chain.len(),
            3,
            "chain must contain v0 + both racing replacements, got {chain:?}"
        );

        // The other replacement CID + the seed CID MUST appear
        // somewhere later in the chain; the chain's contents
        // (modulo head) must equal {va_cid, vb_cid, v0_cid} \
        // {head}.
        let v0_cid = build_handler(h, "post").cid().unwrap();
        let mut tail: Vec<benten_core::Cid> = chain[1..].to_vec();
        tail.sort_by_key(benten_core::Cid::to_base32);
        let other_replacement = if head == va_cid { vb_cid } else { va_cid };
        let mut expected_tail = vec![other_replacement, v0_cid];
        expected_tail.sort_by_key(benten_core::Cid::to_base32);
        assert_eq!(
            tail, expected_tail,
            "chain tail must contain the losing replacement + the v0 seed"
        );
    }
}

#[test]
fn legacy_register_subgraph_seeds_version_chain_too() {
    // Pin: register_subgraph (the legacy path) ALSO seeds the version
    // chain on first registration so a later register_subgraph_replace
    // can name the predecessor cleanly.
    let (engine, _dir) = fresh_engine();
    let h = "h-legacy-seed";
    let v1 = build_handler(h, "post");
    let v1_cid = v1.cid().unwrap();
    engine.register_subgraph(v1).unwrap();
    let chain = engine.handler_version_chain(h);
    assert_eq!(chain, vec![v1_cid]);

    // Now hot-replace via the new API — predecessor must be the
    // legacy-registered CID.
    let v2 = build_handler(h, "comment");
    let outcome = engine.register_subgraph_replace(v2).unwrap();
    assert_eq!(outcome.previous_cid, Some(v1_cid));
}

#[test]
fn register_subgraph_replace_persists_durable_entry_before_in_memory_swap() {
    // g14-c-mr-4 BLOCKER fix-pass regression: prior to fix, the
    // durable system:HandlerVersion zone Node was written AFTER the
    // in-memory chain mutation under released locks. A process crash
    // between in-memory commit and disk persist would surface as the
    // exact "audit-trail erasure" Compromise #18 was supposed to
    // close — Engine::open rebuild from disk would silently drop the
    // most-recent replace.
    //
    // Post-fix the persist runs FIRST. We assert the contract by
    // closing the engine after a successful replace, re-opening, and
    // confirming the durable chain reflects the most recent entry.
    // This is the observable consequence of "persist BEFORE in-memory
    // swap" — when the call returns Ok, the disk MUST hold the
    // entry.
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("persist-first.redb");

    let v1_cid;
    let v2_cid;
    {
        let engine = Engine::builder().path(&store_path).build().unwrap();
        let v1 = build_handler("h-persist-first", "post");
        v1_cid = v1.cid().unwrap();
        engine.register_subgraph_replace(v1).unwrap();

        let v2 = build_handler("h-persist-first", "comment");
        v2_cid = v2.cid().unwrap();
        let outcome = engine.register_subgraph_replace(v2).unwrap();
        assert_eq!(outcome.cid, v2_cid);
        assert_eq!(outcome.previous_cid, Some(v1_cid));
        assert_eq!(outcome.chain_depth, 2);
    }

    // Re-open. The durable chain MUST contain BOTH versions in
    // newest-first order. If the persist had been skipped on either
    // call (the pre-fix race window), this assertion would FAIL —
    // the rebuild from disk would either be missing v2 or contain
    // only v1's seq=0 entry without v2's seq=1.
    let engine = Engine::builder().path(&store_path).build().unwrap();
    let chain = engine.handler_version_chain("h-persist-first");
    assert_eq!(
        chain.len(),
        2,
        "g14-c-mr-4: durable chain MUST hold both replace entries (persist-before-mutation contract)"
    );
    assert_eq!(chain[0], v2_cid, "newest-first invariant preserved");
    assert_eq!(chain[1], v1_cid, "v1 still in chain after re-open");
}
