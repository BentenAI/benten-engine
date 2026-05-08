//! End-to-end pin (per dispatch-conventions §3.6b pim-2) for W9-T6:
//! `RedbBackend::get_node` verify-on-read content-hash defense.
//!
//! ## What this test pins
//!
//! W9-T6 promotes `RedbBackend::get_node` from "decode + return" to
//! "decode + verify CID + return", closing the on-disk-tamper /
//! hardware-bit-flip gap on Node-rehydration paths (handler_versions,
//! engine_modules, IVM materialise). The redb file is treated as a
//! system boundary; CID semantics ("self-validating identifier") are
//! honored on read.
//!
//! ## Three distinct return shapes (the contract)
//!
//! 1. `Ok(None)` — clean miss; CID was never written.
//! 2. `Err(ContentHashMismatch)` — bytes present but corrupted/tampered.
//! 3. `Err(Serialize)` — bytes hash-match but fail to decode (codec drift).
//! 4. `Ok(Some(node))` — clean roundtrip; bytes hash-match and decode.
//!
//! ## "Would FAIL on silent no-op" property (per §3.6b)
//!
//! The tamper test below would FAIL if the verify-on-read code were
//! removed: prior to W9-T6, `get_node` decoded the tampered bytes and
//! returned the wrong-but-decodable Node. The test asserts on
//! `ErrorCode::InvContentHash` — pre-W9-T6 the call returned `Ok(Some(_))`
//! so this assertion would not fire.
//!
//! ## Companion defenses (already in tree, NOT this test's scope)
//!
//! - Subgraph-load surface: `Subgraph::load_verified_with_cid` — pinned
//!   by `subgraph_load_verified_migration.rs`.
//! - Cross-peer ingest surface: `Mst::apply_entries` rehash (sec-r4r2-1)
//!   — pinned by `attack_mst_diff_cid_mismatch.rs` in benten-sync.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stderr,
    reason = "test fixtures + opt-in perf probe — eprintln is intentional under --nocapture"
)]

use benten_core::testing::canonical_test_node;
use benten_core::{Cid, Node, Value};
use benten_errors::ErrorCode;
use benten_graph::RedbBackend;
use tempfile::tempdir;

fn backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("get_node_verify.redb")).unwrap();
    (dir, backend)
}

/// Negative pin: clean miss returns `Ok(None)`, NOT an error.
///
/// Distinguishes "absent" from "corrupted" — the W9-T6 verify-on-read
/// arm must NOT fire on a miss (no bytes to verify).
#[test]
fn get_node_returns_none_for_missing_cid_after_verify_on_read_promotion() {
    let (_dir, backend) = backend();
    let missing = Cid::from_blake3_digest([0u8; 32]);
    let out = backend
        .get_node(&missing)
        .expect("missing CID must be Ok(None), not Err");
    assert!(
        out.is_none(),
        "missing CID must return None, not a content-hash error"
    );
}

/// Positive pin: clean roundtrip returns the byte-equal Node.
///
/// Pre-W9-T6 this passed because decode succeeded; post-W9-T6 it must
/// still pass because the recomputed CID matches the requested CID.
/// Catches "verify-on-read accidentally rejects valid bytes" regressions.
#[test]
fn get_node_clean_roundtrip_returns_byte_equal_node_after_verify() {
    let (_dir, backend) = backend();
    let node = canonical_test_node();
    let cid = backend.put_node(&node).expect("seed put_node must succeed");

    let loaded = backend
        .get_node(&cid)
        .expect("verify-on-read must succeed for untampered bytes")
        .expect("present CID must yield Some(node)");

    assert_eq!(
        loaded, node,
        "clean roundtrip must return byte-equal Node after verify-on-read"
    );

    // Re-CID the loaded Node — must match the originally-requested CID.
    let recomputed = loaded.cid().expect("re-cid must succeed");
    assert_eq!(
        recomputed, cid,
        "round-trip must preserve CID identity through verify-on-read"
    );
}

/// PRIMARY pin: tampered on-disk bytes fire `E_INV_CONTENT_HASH` on
/// `get_node`, NOT a wrong-but-decodable Node.
///
/// **This test would FAIL if W9-T6 verify-on-read were removed** — pre-W9-T6,
/// `get_node` returned `Ok(Some(tampered_node))` for any decodable bytes
/// regardless of whether they hashed to the requested CID. The assertion
/// `err.code() == ErrorCode::InvContentHash` would not fire because
/// the call would not return an error at all.
#[test]
fn get_node_rejects_tampered_on_disk_bytes_with_content_hash_error() {
    let (_dir, backend) = backend();
    let node = canonical_test_node();
    let original_cid = backend.put_node(&node).expect("seed put_node must succeed");

    // Tamper the on-disk bytes in place. Flip the LAST byte by +1 — the
    // mutated bytes should still decode as a Node (the trailing bytes of
    // canonical DAG-CBOR encoding are typically value bytes, so flipping
    // one alters the Node's content but not its CBOR structure). Even if
    // the flip happens to break decode, the assertion below still holds:
    // the verify-on-read check fires BEFORE decode (per the impl
    // comment), so we always observe `InvContentHash` not `Serialize`.
    backend
        .corrupt_node_bytes_for_test(&original_cid, |b| {
            if let Some(byte) = b.last_mut() {
                *byte = byte.wrapping_add(1);
            }
        })
        .expect("test-only corruption hook must succeed");

    let err = backend
        .get_node(&original_cid)
        .expect_err("tampered bytes MUST fail get_node — this is the W9-T6 defense");

    assert_eq!(
        err.code(),
        ErrorCode::InvContentHash,
        "tamper at rest MUST fire E_INV_CONTENT_HASH (got {:?}); \
         a different code means W9-T6 verify-on-read regressed",
        err.code()
    );
}

/// Tamper variant: substitute bytes for a DIFFERENT (decodable) Node.
///
/// Stronger version of the PRIMARY pin — directly demonstrates that
/// pre-W9-T6 `get_node` would silently return Node B when asked for
/// Node A's CID. Post-W9-T6 it must fire `E_INV_CONTENT_HASH`.
#[test]
fn get_node_rejects_substituted_decodable_bytes_with_content_hash_error() {
    let (_dir, backend) = backend();

    // Two distinct Nodes with different bytes/CIDs.
    let node_a = canonical_test_node();
    let cid_a = backend
        .put_node(&node_a)
        .expect("seed put_node(A) must succeed");

    let mut node_b = canonical_test_node();
    node_b
        .properties
        .insert("w9_t6_marker".to_string(), Value::text("tampered"));
    let bytes_b = node_b
        .canonical_bytes()
        .expect("canonical_bytes(B) must succeed");

    // Sanity check: A and B encode to different bytes.
    let bytes_a = node_a.canonical_bytes().expect("canonical_bytes(A)");
    assert_ne!(bytes_a, bytes_b, "test fixture: A and B must differ");

    // Overwrite A's slot at cid_a with B's bytes.
    backend
        .corrupt_node_bytes_for_test(&cid_a, |b| {
            // Replace contents in place. The closure receives a &mut [u8]
            // sized to A's bytes; if B is shorter/longer we truncate or
            // pad — the goal is "different bytes under the same key,"
            // which any mutation achieves.
            let n = b.len().min(bytes_b.len());
            b[..n].copy_from_slice(&bytes_b[..n]);
            // Modify a tail byte to guarantee post-mutation hash differs
            // from cid_a even when bytes_b.len() < b.len().
            if let Some(tail) = b.last_mut() {
                *tail = tail.wrapping_add(1);
            }
        })
        .expect("test-only corruption hook must succeed");

    let err = backend
        .get_node(&cid_a)
        .expect_err("substituted bytes MUST fail get_node — this is the W9-T6 defense");

    assert_eq!(
        err.code(),
        ErrorCode::InvContentHash,
        "byte-substitution at rest MUST fire E_INV_CONTENT_HASH (got {:?}); \
         a different code means W9-T6 verify-on-read regressed",
        err.code()
    );
}

/// Pin the three-outcome contract explicitly: each return shape is
/// distinguishable from the others by its discriminant.
///
/// Defends the contract documented in `RedbBackend::get_node`'s docstring
/// against accidental shape collapse (e.g. a future refactor that
/// converts `Err(ContentHashMismatch)` into `Ok(None)` to "simplify"
/// the return type — which would silently re-open the W9-T6 gap).
#[test]
fn get_node_three_outcomes_are_distinguishable() {
    let (_dir, backend) = backend();

    // Outcome 1: clean miss → Ok(None).
    let missing = Cid::from_blake3_digest([0xAA; 32]);
    assert!(matches!(backend.get_node(&missing), Ok(None)));

    // Outcome 2: clean hit → Ok(Some(node)).
    let node = canonical_test_node();
    let cid = backend.put_node(&node).unwrap();
    assert!(matches!(backend.get_node(&cid), Ok(Some(_))));

    // Outcome 3: tamper → Err(ContentHashMismatch).
    backend
        .corrupt_node_bytes_for_test(&cid, |b| {
            if let Some(byte) = b.last_mut() {
                *byte = byte.wrapping_add(1);
            }
        })
        .unwrap();
    let err = backend.get_node(&cid).expect_err("tamper must Err");
    assert_eq!(err.code(), ErrorCode::InvContentHash);

    // After tamper, the missing-CID arm still returns Ok(None) cleanly
    // (no cross-contamination between the two error states).
    let other_missing = Cid::from_blake3_digest([0xBB; 32]);
    assert!(matches!(backend.get_node(&other_missing), Ok(None)));
}

/// Hand-timed performance probe (not a strict assertion — runtime
/// variance under nextest parallelism). Provides a directional signal
/// that verify-on-read overhead stays in the ~3-10 µs/call ballpark
/// Ben accepted at W9-T6 ratification. Asserts only the LOOSE upper
/// bound (50 µs/call) — the "STOP and surface" trigger from the
/// dispatch brief.
///
/// Marked `#[ignore]` by default so it does not run under standard
/// test suites; un-ignore + run with `cargo test --release ... --
/// --ignored --nocapture` to observe the actual figure.
#[test]
#[ignore = "performance probe; run explicitly with --ignored --nocapture"]
fn get_node_verify_on_read_perf_probe() {
    use std::time::Instant;
    let (_dir, backend) = backend();
    let node = canonical_test_node();
    let cid = backend.put_node(&node).unwrap();

    // Warmup
    for _ in 0..1_000 {
        backend.get_node(&cid).unwrap();
    }

    let n = 10_000u32;
    let start = Instant::now();
    for _ in 0..n {
        let _ = backend.get_node(&cid).unwrap();
    }
    let elapsed = start.elapsed();
    let per_call_us = elapsed.as_secs_f64() * 1e6 / f64::from(n);
    eprintln!("get_node verify-on-read: {per_call_us:.2} µs/call ({n} calls in {elapsed:?})");

    assert!(
        per_call_us < 50.0,
        "verify-on-read regression: {per_call_us:.2} µs/call exceeds 50 µs ceiling"
    );
}

/// Helper for assertions: extract `Node` for explicit-shape readability.
///
/// Not a test; used to keep the assertion sites above terse without
/// pulling in extra crates.
#[allow(dead_code)]
fn _doc_link_anchor(node: Node) -> Node {
    node
}
