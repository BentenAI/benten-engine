//! **R6 R2 batch-A Item 8 (Row D-8 closure / F3 anti-replay TOCTOU
//! CAS)** — verifies that `FrameReplayMarker::mark_and_check_frame`
//! admits AT MOST ONE concurrent caller for the SAME nonce.
//!
//! ## Why this test exists (Compromise #23 in-window-racy retraction)
//!
//! Pre-Item-8: `mark_and_check_frame` did a non-atomic get + put
//! pair under SEPARATE redb write transactions. Two concurrent
//! inbound `apply_atrium_merge` presentations of the same
//! session-nonce could BOTH observe "absent" between their respective
//! `get(nonce)` calls (no exclusive write-txn held between the get +
//! the put) and BOTH proceed past the replay-check arm — silent
//! double-admission of a replayed frame within its freshness window.
//! Compromise #23 retensed-to-acknowledge in-window-racy at v1-beta.
//!
//! Post-Item-8: routes through
//! `benten_graph::KVBackend::compare_and_insert` which, on the
//! redb-backed backend, runs the get + insert + commit inside a
//! SINGLE redb write transaction. Redb's write-txn exclusivity
//! (only ONE write-txn open per-handle at a time) gates concurrent
//! CAS attempts: at most ONE admits.
//!
//! ## Test shape (§3.6f SUBSTANTIVE-arm)
//!
//! 1. Spin up `N` (=16) threads that all call
//!    `mark_and_check_frame(&SAME_NONCE)` concurrently against the
//!    same redb-backed `FrameReplayMarker`.
//! 2. Count `Ok(false)` results — these are the "first-observation
//!    arm" admissions; AT MOST ONE may return false.
//! 3. All other threads MUST return `Ok(true)` (the replay-rejected
//!    arm).
//! 4. Cross-arm: a second SEQUENTIAL call after all threads complete
//!    MUST also return `Ok(true)` (state persisted; idempotent at the
//!    replay arm).
//!
//! ## Would-FAIL-on-revert (§3.6f)
//!
//! Revert `mark_and_check_frame` to the non-atomic get + put pair
//! → multiple threads observe "absent" in their respective `get`
//! calls before any `put` lands → multiple `Ok(false)` admissions →
//! `first_observers >= 1` assertion (post-fix) inverts to
//! `first_observers > 1` (pre-fix racy) → would-FAIL fires.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used)]

use benten_caps::FrameReplayMarker;
use benten_graph::RedbBackend;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

#[test]
fn frame_replay_marker_cas_atomic_admits_at_most_one_concurrent_caller_for_same_nonce() {
    let dir = tempfile::tempdir().unwrap();
    let backend = Arc::new(RedbBackend::open_or_create(dir.path().join("test.redb")).unwrap());

    let marker = Arc::new(FrameReplayMarker::new(Arc::clone(&backend)));
    let nonce = vec![0xDEu8; 32];

    const N_THREADS: usize = 16;
    let first_observers = Arc::new(AtomicUsize::new(0));
    let replay_observers = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(N_THREADS);
    for _ in 0..N_THREADS {
        let marker = Arc::clone(&marker);
        let nonce = nonce.clone();
        let first_observers = Arc::clone(&first_observers);
        let replay_observers = Arc::clone(&replay_observers);
        let errors = Arc::clone(&errors);
        handles.push(thread::spawn(move || {
            match marker.mark_and_check_frame(&nonce) {
                Ok(false) => {
                    // first observer: this is the legitimate admit.
                    first_observers.fetch_add(1, Ordering::SeqCst);
                }
                Ok(true) => {
                    // replay-rejected: this is the safe arm.
                    replay_observers.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    errors.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let first = first_observers.load(Ordering::SeqCst);
    let replay = replay_observers.load(Ordering::SeqCst);
    let err = errors.load(Ordering::SeqCst);

    assert_eq!(
        err, 0,
        "Item 8 CAS LOAD-BEARING: no caller may surface a backend \
         error from the concurrent CAS race; observed={err}"
    );
    assert_eq!(
        first, 1,
        "Item 8 CAS LOAD-BEARING: AT MOST ONE concurrent caller may \
         admit the first-observer arm (`Ok(false)`); pre-Item-8 the \
         non-atomic get+put pair could admit several. observed \
         first_observers={first}, replay_observers={replay}"
    );
    assert_eq!(
        replay,
        N_THREADS - 1,
        "Item 8 CAS LOAD-BEARING: every other concurrent caller must \
         take the replay-rejected arm (`Ok(true)`); observed={replay} \
         (expected {})",
        N_THREADS - 1
    );

    // Sequential follow-up: the marker is persisted; another call
    // MUST also surface the replay arm.
    assert!(
        marker.mark_and_check_frame(&nonce).unwrap(),
        "post-concurrent: subsequent presentations of the same nonce \
         MUST observably return Ok(true) (replay-rejected; marker \
         persisted)"
    );
}

#[test]
fn frame_replay_marker_cas_admits_distinct_nonces_independently() {
    // Cross-arm: two distinct nonces both admit at first observation
    // (the CAS is per-key, not a global lock; distinct keys do NOT
    // contend).
    let dir = tempfile::tempdir().unwrap();
    let backend = Arc::new(RedbBackend::open_or_create(dir.path().join("test2.redb")).unwrap());
    let marker = FrameReplayMarker::new(backend);

    let nonce_a = vec![0xA1u8; 32];
    let nonce_b = vec![0xB2u8; 32];

    assert!(
        !marker.mark_and_check_frame(&nonce_a).unwrap(),
        "first observation of nonce_a admits (Ok(false))"
    );
    assert!(
        !marker.mark_and_check_frame(&nonce_b).unwrap(),
        "distinct nonce_b admits independently of nonce_a (Ok(false))"
    );
    assert!(
        marker.mark_and_check_frame(&nonce_a).unwrap(),
        "second presentation of nonce_a is replay-rejected (Ok(true))"
    );
    assert!(
        marker.mark_and_check_frame(&nonce_b).unwrap(),
        "second presentation of nonce_b is replay-rejected (Ok(true))"
    );
}
