//! Safe-4 #636: multi-thread smoke test pinning the documented
//! concurrent-correctness claims on `Hlc` and the shared u64-id `Anchor`
//! chain. The compile-time `Send + Sync` assertions live next to the
//! type definitions (`hlc.rs`, `version.rs`, `lib.rs`); this test
//! exercises the *runtime* path those auto-traits enable so a regression
//! in the locking discipline (not just the trait bounds) is caught.

use std::sync::Arc;
use std::thread;

use benten_core::hlc::Hlc;
use benten_core::{Anchor, Cid, Node, append_version, walk_versions};

fn frozen_clock() -> u64 {
    1_000
}

#[test]
fn hlc_now_is_strictly_monotonic_under_concurrent_callers() {
    let hlc = Arc::new(Hlc::new(7, frozen_clock));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let h = Arc::clone(&hlc);
            thread::spawn(move || {
                let mut local = Vec::with_capacity(64);
                for _ in 0..64 {
                    local.push(h.now());
                }
                local
            })
        })
        .collect();

    let mut all: Vec<_> = threads
        .into_iter()
        .flat_map(|t| t.join().expect("hlc thread must not panic"))
        .collect();

    // Every emitted HLC across every thread must be distinct: the
    // internal spin::Mutex serializes emission and the logical counter
    // advances even though the physical clock is frozen.
    all.sort();
    let len_before = all.len();
    all.dedup();
    assert_eq!(
        all.len(),
        len_before,
        "HLC emitted a duplicate timestamp under concurrent callers — \
         the documented Send+Sync serialize-cleanly contract regressed"
    );
}

#[test]
fn shared_u64_anchor_chain_accepts_concurrent_appends() {
    let anchor = Arc::new(Anchor::new());
    let n_threads = 6;
    let per_thread = 20;

    let handles: Vec<_> = (0..n_threads)
        .map(|t| {
            let a = Arc::clone(&anchor);
            thread::spawn(move || {
                for i in 0..per_thread {
                    let node =
                        Node::new(vec![format!("V{t}_{i}")], std::collections::BTreeMap::new());
                    let cid: Cid = append_version(&a, &node).expect("append must succeed");
                    // touch the returned CID so the optimizer cannot
                    // elide the work
                    assert_ne!(cid.as_bytes()[0], 0xFF);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("append thread must not panic");
    }

    let walked: Vec<_> = walk_versions(&anchor).expect("walk must succeed");
    assert_eq!(
        walked.len(),
        n_threads * per_thread,
        "every concurrent append must be retained in the shared chain"
    );
}
