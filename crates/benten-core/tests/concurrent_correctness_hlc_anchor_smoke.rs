//! Safe-4 #636: multi-thread smoke test pinning the documented
//! concurrent-correctness claims on `Hlc`. The compile-time `Send + Sync`
//! assertions live next to the type definitions (`hlc.rs`, `version.rs`);
//! this test exercises the *runtime* path those auto-traits enable so a
//! regression in the locking discipline (not just the trait bounds) is
//! caught.
//!
//! The shared `u64`-id `Anchor` chain arm was removed when that surface was
//! deleted for #1003 (RATIFIED 2026-05-17). `#636`'s remaining valid
//! concurrent targets are `Hlc` (this file) and the Cid-head
//! `version::Anchor` (whose `Send + Sync` compile-pin lives at
//! `version.rs`).

use std::sync::Arc;
use std::thread;

use benten_core::hlc::Hlc;

fn frozen_clock() -> u64 {
    1_000
}

// The u64-id Anchor concurrent-append arm was deleted with the #1003
// surface removal (RATIFIED 2026-05-17). The live `version::Anchor`
// concurrent contract is pinned by its own `Send + Sync` compile-time
// assertion at `crates/benten-core/src/version.rs`.

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
