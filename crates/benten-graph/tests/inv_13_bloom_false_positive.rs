//! Edge-case tests: bloom-filter CID-existence cache falsy-positive path for
//! Invariant 13 (immutability).
//!
//! R2 landscape §2.3 row "Bloom-filter CID-existence cache + reject on re-put".
//!
//! The bloom filter is a fast-path accelerator; a false positive MUST fall
//! through to an exact redb lookup and produce a correct answer. Concerns:
//!
//! - Bloom reports "present" for a CID that is actually absent → exact check
//!   corrects to Ok (first-put path).
//! - Bloom reports "present" for a CID that IS present → exact check confirms
//!   and fires `E_INV_IMMUTABILITY`.
//! - Bloom is warmed on first put (present after the first call) — pins the
//!   cache-warming invariant.
//! - Bloom cache does not short-circuit correctness on an empty store.
//!
//! R3 red-phase contract: R5 (G2-A) lands the bloom filter cache and the
//! forced-collision test hook. These tests compile; they fail because
//! `RedbBackend::force_bloom_positive_for_test` does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_errors::ErrorCode;
use benten_graph::{NodeStore, RedbBackend, WriteAuthority, WriteContext};
use tempfile::tempdir;

fn node(tag: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("tag".into(), Value::text(tag));
    Node::new(vec!["Doc".into()], props)
}

fn user_ctx() -> WriteContext {
    WriteContext {
        label: "Doc".into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    }
}

#[test]
fn bloom_false_positive_absent_cid_falls_through_to_exact_check_and_accepts_first_put() {
    // Force the bloom filter to report "present" for a CID that is NOT in the
    // store. The exact redb check must correct the false positive and allow
    // the first put to succeed.
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("bloom_fp.redb")).unwrap();

    let n = node("absent_but_bloom_lies");
    // Compute the CID that the put will produce.
    let precomputed_cid = n.cid().expect("content-addressed cid must compute");

    // Poison the bloom filter.
    backend.force_bloom_positive_for_test(&precomputed_cid);

    // First put: bloom says "maybe present", exact check says "no, absent",
    // so the put succeeds.
    let cid = backend
        .put_node_with_context(&n, &user_ctx())
        .expect("bloom false-positive must NOT reject a genuinely-absent CID");
    assert_eq!(cid, precomputed_cid);
}

#[test]
fn bloom_false_positive_present_cid_exact_check_fires_immutability() {
    // Bloom correctly reports "present" (true positive after seed). Exact
    // check confirms and fires Inv-13 on the second User put.
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("bloom_tp.redb")).unwrap();

    let n = node("present_exact_check");
    let cid = backend.put_node_with_context(&n, &user_ctx()).unwrap();

    // After a successful put, bloom must be warm.
    assert!(
        backend.bloom_may_contain_for_test(&cid),
        "bloom must be warmed on first put"
    );

    let err = backend
        .put_node_with_context(&n, &user_ctx())
        .expect_err("second User put must fire Inv-13");
    assert_eq!(err.code(), ErrorCode::InvImmutability);
}

#[test]
fn bloom_cache_warms_on_first_put_not_on_first_get() {
    // The bloom filter is populated by writes, not by reads. A get on a
    // never-written CID must not warm the bloom.
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("bloom_warm.redb")).unwrap();

    let untouched = Cid::from_blake3_digest([0x42u8; 32]);
    let _ = backend.get_node(&untouched).unwrap();
    assert!(
        !backend.bloom_may_contain_for_test(&untouched),
        "get on untouched CID must NOT warm bloom"
    );

    let n = node("warmup");
    let cid = backend.put_node_with_context(&n, &user_ctx()).unwrap();
    assert!(
        backend.bloom_may_contain_for_test(&cid),
        "put must warm bloom"
    );
}

#[test]
fn bloom_empty_store_does_not_short_circuit_bogus_present() {
    // Defensive: a freshly-created backend must not spuriously report
    // "present" for any CID — bloom is empty at creation.
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("bloom_empty.redb")).unwrap();

    // Sample 16 pseudo-random CIDs; none should report present on an empty
    // store unless the bloom false-positive rate is catastrophically high.
    let mut spurious = 0usize;
    for i in 0..16u8 {
        let cid = Cid::from_blake3_digest([i; 32]);
        if backend.bloom_may_contain_for_test(&cid) {
            spurious += 1;
        }
    }
    assert!(
        spurious <= 1,
        "empty bloom filter has suspiciously high FPR: {spurious}/16"
    );
}
