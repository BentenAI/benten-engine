//! #992 closure-pin (refinement-audit-2026-05 wire-format cluster):
//! the redb on-disk file carries a schema-version envelope, unlike the
//! pre-fix state which had NO whole-file format envelope (only redb's own
//! page version) — asymmetric with `SnapshotBlob` which refuses unknown
//! `schema_version`s.
//!
//! WIRE-FORMAT FREEZE ITEM. The "absence ≡ v1" rule is only sound to
//! establish pre-v1 (every existing DB IS the v1 5-prefix layout). These
//! assertions WOULD FAIL against the pre-fix backend (no meta-key written,
//! no `SchemaVersionMismatch` variant, no refusal on a future version).
//!
//! Three arms (per design-wireformat-3.md):
//!  1. round-trip — open-or-create stamps v1; reopen-existing accepts.
//!  2. implied-v1 — a file with NO envelope (pre-envelope build) opens
//!     successfully as implied-v1 (the rule that freezes at v1).
//!  3. mismatch-refusal — a file declaring a version this build does not
//!     understand is REFUSED with `GraphError::SchemaVersionMismatch` /
//!     `ErrorCode::GraphSchemaVersionMismatch`, not silently mis-routed.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;
use benten_graph::{GraphError, RedbBackend};
use tempfile::tempdir;

#[test]
fn arm1_open_or_create_stamps_v1_and_reopen_existing_accepts() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("envelope.redb");

    // Fresh open-or-create stamps the envelope = 1.
    let be = RedbBackend::open_or_create(&path).unwrap();
    assert_eq!(
        be.read_schema_version_for_test().unwrap(),
        Some(1),
        "open-or-create must stamp the schema-version envelope = 1"
    );
    drop(be);

    // Reopen-existing of a correctly-versioned file succeeds.
    RedbBackend::open_existing(&path).expect("open-existing of a v1 file must succeed");
}

#[test]
fn arm2_pre_envelope_file_opens_as_implied_v1() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("pre-envelope.redb");

    // Create a DB then DELETE the envelope key to simulate a file written
    // by a pre-#992 build (no schema-version meta-row at all).
    let be = RedbBackend::open_or_create(&path).unwrap();
    be.force_schema_version_for_test(None).unwrap();
    assert_eq!(
        be.read_schema_version_for_test().unwrap(),
        None,
        "precondition: envelope key removed"
    );
    drop(be);

    // open-existing must ACCEPT (absence ≡ v1 — a pre-envelope file IS the
    // v1 5-prefix layout by definition) and must NOT back-write the key
    // (read-only intent).
    let reopened =
        RedbBackend::open_existing(&path).expect("pre-envelope file must open as implied-v1");
    assert_eq!(
        reopened.read_schema_version_for_test().unwrap(),
        None,
        "open-existing must not back-write the envelope (read-only intent)"
    );
}

#[test]
fn arm3_version_mismatch_is_refused_not_misrouted() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("future.redb");

    // Stamp a future version this build does not understand.
    let be = RedbBackend::open_or_create(&path).unwrap();
    be.force_schema_version_for_test(Some(99)).unwrap();
    drop(be);

    // open-existing must REFUSE — not silently mis-route reads against a
    // future prefix schema.
    let err = RedbBackend::open_existing(&path)
        .expect_err("a version-99 file must be refused by a v1 build");
    match err {
        GraphError::SchemaVersionMismatch { expected, actual } => {
            assert_eq!(expected, 1);
            assert_eq!(actual, 99);
            assert_eq!(err.code(), ErrorCode::GraphSchemaVersionMismatch);
        }
        other => panic!("expected GraphError::SchemaVersionMismatch, got {other:?}"),
    }

    // open-or-create against the same mismatched file must ALSO refuse
    // (a present-but-unknown version is never overwritten).
    let err2 = RedbBackend::open_or_create(&path)
        .expect_err("open-or-create must also refuse a mismatched version");
    assert!(matches!(
        err2,
        GraphError::SchemaVersionMismatch {
            expected: 1,
            actual: 99
        }
    ));
}
