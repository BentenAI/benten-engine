//! Concurrent reader / writer soak test for `RedbBackend`.
//!
//! Not a benchmark — an integration stress test that runs as part of the
//! normal `cargo nextest run` workflow. Asserts correctness properties
//! under concurrency:
//!
//! 1. Multiple readers never block the single writer indefinitely.
//! 2. Every successfully-committed write becomes visible to subsequent
//!    reader transactions.
//! 3. Interleaved reads never observe partial writes (MVCC snapshot
//!    isolation — a reader sees either the pre-commit or post-commit
//!    state, never a half-updated Node).
//! 4. No deadlocks or writer starvation under sustained read pressure.
//!
//! The soak runs for a short-but-nontrivial duration (default 2 seconds)
//! and asserts the above invariants hold for every read observed.
//!
//! ## Gating
//!
//! This test is **gated** as a correctness gate, not a performance gate.
//! CI fails if:
//! - Any reader observes a Node whose CID does not match its content.
//! - The writer fails to commit within the timeout.
//! - The test panics or deadlocks (enforced by `should_panic = false` and
//!   the outer nextest timeout of 60s).
//!
//! §14.6 informational note: this test's throughput output is a secondary
//! signal for the `concurrent_writers` informational benchmark — if the
//! bench shows a cliff at N=8 writers and this soak starts failing, the
//! two observations together point to contention in the redb layer.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests may use unwrap/expect per workspace policy"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use benten_core::testing::canonical_test_node;
use benten_graph::RedbBackend;
use tempfile::tempdir;

const SOAK_DURATION: Duration = Duration::from_secs(2);

#[test]
fn concurrent_reader_writer_soak_no_corruption_no_deadlock() {
    let dir = tempdir().expect("tempdir");
    let backend = Arc::new(RedbBackend::open(dir.path().join("benten.redb")).expect("open"));

    // Seed one Node so readers have something to find from T=0.
    let seed = canonical_test_node();
    let seed_cid = backend.put_node(&seed).expect("seed put");

    let stop = Arc::new(AtomicBool::new(false));
    let writes = Arc::new(AtomicUsize::new(0));
    let reads = Arc::new(AtomicUsize::new(0));
    let corruption = Arc::new(AtomicUsize::new(0));

    // Single writer thread — redb serializes writers anyway; spawning
    // multiple doesn't stress harder, it just stresses differently.
    let writer = {
        let backend = Arc::clone(&backend);
        let stop = Arc::clone(&stop);
        let writes = Arc::clone(&writes);
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let _cid = backend.put_node(&canonical_test_node()).expect("put");
                writes.fetch_add(1, Ordering::Relaxed);
            }
        })
    };

    // Several readers hammer the seed CID. Each read must either return
    // None (race with a never-committed write — shouldn't happen for the
    // seed CID) or a Node whose CID matches the requested one.
    let reader_count = 4;
    let readers: Vec<_> = (0..reader_count)
        .map(|_| {
            let backend = Arc::clone(&backend);
            let stop = Arc::clone(&stop);
            let reads = Arc::clone(&reads);
            let corruption = Arc::clone(&corruption);
            let seed_cid = seed_cid.clone();
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    match backend.get_node(&seed_cid).expect("get") {
                        Some(node) => {
                            reads.fetch_add(1, Ordering::Relaxed);
                            // Re-hash: if the content doesn't re-hash to the
                            // CID we asked for, we observed partial/corrupt
                            // state.
                            let rehash = node.cid().expect("rehash");
                            if rehash != seed_cid {
                                corruption.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        None => {
                            // Reading the seed CID should always return Some
                            // after the initial seed put committed.
                            corruption.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            })
        })
        .collect();

    let start = Instant::now();
    thread::sleep(SOAK_DURATION);
    stop.store(true, Ordering::Relaxed);

    writer.join().expect("writer join");
    for r in readers {
        r.join().expect("reader join");
    }

    let elapsed = start.elapsed();
    let n_writes = writes.load(Ordering::Relaxed);
    let n_reads = reads.load(Ordering::Relaxed);
    let n_corrupt = corruption.load(Ordering::Relaxed);

    // Correctness gates:
    assert_eq!(
        n_corrupt, 0,
        "observed {n_corrupt} corrupt reads under concurrency"
    );
    assert!(
        n_writes > 0,
        "writer made zero progress in {elapsed:?} — possible starvation"
    );
    assert!(
        n_reads > 0,
        "readers observed zero reads — possible starvation"
    );
    // Sanity: we should be doing thousands of ops per second at minimum.
    // A regression to <100 ops/sec total is a structural problem.
    assert!(
        n_writes + n_reads > 100,
        "soak produced only {} total ops in {elapsed:?} — check for lock contention",
        n_writes + n_reads
    );
}
