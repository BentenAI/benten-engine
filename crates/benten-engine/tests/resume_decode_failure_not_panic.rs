//! Edge-case tests: `Engine::resume_from_bytes` surfaces decode failures as
//! typed errors, never as panics.
//!
//! R2 landscape §8.2: sec-r1-1 payload shape + decode-failure-not-panic
//! coverage for the new persisted types (`ExecutionStateEnvelope`,
//! `AttributionFrame`, `HostError` serialization boundary).
//!
//! Plan §9.1 resume protocol step 1 deserializes the envelope. Corrupt or
//! tampered bytes at that step MUST produce `E_EXEC_STATE_TAMPERED`
//! (integrity) or `E_SERIALIZE` (pure decode failure) — not panic, not
//! abort. Resume is a user-facing entry point; a panic here is an
//! availability bug.
//!
//! Concerns pinned:
//! - Empty bytes → typed `E_SERIALIZE`.
//! - Single-byte truncation → typed error.
//! - Valid-CBOR-but-wrong-shape (e.g., a Node instead of ExecutionStateEnvelope)
//!   → typed error, not panic.
//! - A byte-flipped envelope body (integrity break) → typed
//!   `E_EXEC_STATE_TAMPERED` (distinguishes integrity from pure decode).
//! - A resume with attribution-chain containing a CID that fails CID parsing
//!   → typed error (not a panic at the multihash-decode layer).
//!
//! R3 red-phase contract: R5 (G3-A / G3-B) lands `resume_from_bytes`. Tests
//! compile; they fail because the entry point does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Value};
use benten_engine::Engine;
use benten_errors::ErrorCode;
use tempfile::tempdir;

fn engine() -> (tempfile::TempDir, Engine) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("resume_decode.redb"))
        .without_versioning()
        .build()
        .unwrap();
    (dir, engine)
}

fn principal() -> Cid {
    Cid::from_blake3_digest([0xa1; 32])
}

#[test]
fn resume_from_bytes_empty_input_returns_typed_error_not_panic() {
    let (_dir, engine) = engine();
    let result = engine.resume_from_bytes_as(&[], Value::unit(), &principal());
    let err = result.expect_err("empty bytes must fail");
    assert!(
        matches!(
            err.code(),
            ErrorCode::Serialize | ErrorCode::ExecStateTampered
        ),
        "empty bytes must fire a typed error (Serialize or ExecStateTampered), got {:?}",
        err.code()
    );
}

#[test]
fn resume_from_bytes_single_byte_truncation_returns_typed_error_not_panic() {
    let (_dir, engine) = engine();
    let result = engine.resume_from_bytes_as(&[0xa0], Value::unit(), &principal());
    let err = result.expect_err("truncated bytes must fail");
    assert!(
        matches!(
            err.code(),
            ErrorCode::Serialize | ErrorCode::ExecStateTampered
        ),
        "truncated bytes must fire typed error, got {:?}",
        err.code()
    );
}

#[test]
fn resume_from_bytes_valid_cbor_but_wrong_shape_returns_typed_error_not_panic() {
    // DAG-CBOR-valid bytes that decode into a *different* typed struct (a
    // plain Value::Int, say) must not be interpreted as an envelope —
    // the decode error is typed, not a panic.
    let (_dir, engine) = engine();

    // Encode something that is NOT an ExecutionStateEnvelope.
    let wrong: Value = Value::Int(42);
    let bytes = serde_ipld_dagcbor::to_vec(&wrong).expect("encode succeeds");

    let result = engine.resume_from_bytes_as(&bytes, Value::unit(), &principal());
    let err = result.expect_err("wrong-shape CBOR must fail resume");
    assert!(
        matches!(
            err.code(),
            ErrorCode::Serialize | ErrorCode::ExecStateTampered
        ),
        "wrong-shape CBOR must fire typed error, got {:?}",
        err.code()
    );
}

#[test]
fn resume_from_bytes_flipped_byte_fires_exec_state_tampered() {
    // Produce a genuine envelope via suspend_to_bytes, flip one byte, and
    // expect E_EXEC_STATE_TAMPERED (the integrity path), NOT plain serialize
    // (the pure-decode path). This proves the two codes are distinguishable
    // and that integrity checking runs before the handle is trusted.
    let (_dir, engine) = engine();

    // Synthesize a valid envelope via the engine's test helper.
    let mut envelope_bytes = engine
        .fabricate_test_suspend_envelope(&principal())
        .expect("test harness must produce a valid envelope");

    // Flip one byte in the payload body (deep enough to survive any prefix
    // sanity check and trip the integrity MAC / CID re-compute).
    let mid = envelope_bytes.len() / 2;
    envelope_bytes[mid] = envelope_bytes[mid].wrapping_add(1);

    let err = engine
        .resume_from_bytes_as(&envelope_bytes, Value::unit(), &principal())
        .expect_err("flipped byte must fail resume");
    assert_eq!(
        err.code(),
        ErrorCode::ExecStateTampered,
        "integrity break must fire E_EXEC_STATE_TAMPERED, got {:?}",
        err.code()
    );
}

#[test]
fn resume_from_bytes_malformed_cid_in_attribution_is_typed_error_not_panic() {
    // Insert a malformed CID inside the attribution chain. The multihash
    // decoder used to panic on some malformed inputs pre-0.19; we pin that
    // the current decode path surfaces a typed error.
    let (_dir, engine) = engine();
    let envelope_bytes = engine
        .fabricate_test_suspend_envelope_with_attribution_cid_bytes(
            &principal(),
            &[0x01, 0x00, 0x00], // malformed multihash
        )
        .expect("test harness must produce an envelope with bad CID bytes");

    let err = engine
        .resume_from_bytes_as(&envelope_bytes, Value::unit(), &principal())
        .expect_err("malformed CID in attribution must fail");
    // Acceptable codes: CidParse, CidUnsupportedCodec, CidUnsupportedHash,
    // ExecStateTampered, Serialize. The point is NO panic and it's typed.
    let c = err.code();
    assert!(
        matches!(
            c,
            ErrorCode::CidParse
                | ErrorCode::CidUnsupportedCodec
                | ErrorCode::CidUnsupportedHash
                | ErrorCode::ExecStateTampered
                | ErrorCode::Serialize
        ),
        "malformed-CID decode must produce a typed error, got {:?}",
        c
    );
}
