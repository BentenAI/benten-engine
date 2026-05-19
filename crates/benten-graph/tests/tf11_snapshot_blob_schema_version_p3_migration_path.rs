//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — TF-11 P-III half.
//! Agent **R3-B4**. The §8-B-(i) `SnapshotBlob.schema_version 1→2`
//! P-III pin. This file is the **benten-graph** half of TF-11 (the
//! mode-b/c light-client half lives in
//! `crates/benten-sync/tests/tf11_benten_sync_light_client_mode_b_c.rs`).
//!
//! ## Why this lives in benten-graph (not benten-sync) — §8-B-(i)
//!
//! Per §8-B-(i) (Ben-ACKED 2026-05-19) + the plan's own ground-truth:
//! `SnapshotBlob` + `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 1` + the
//! `SchemaVersion`-mismatch error ALL live in
//! `crates/benten-graph/src/backends/snapshot_blob.rs` (ZERO
//! `SnapshotBlob` in `crates/benten-sync/src/`). The mode-(c)
//! checkpoint field therefore forces an **in-place `benten-graph`
//! struct mutation** — so the P-III pin is structurally a
//! benten-graph surface. `benten-sync` does NOT depend on
//! `benten-graph` (the documented layered-dependency intent in
//! `benten-sync` `[lib]` + `tests/dependency_edges.rs`); putting this
//! pin in a benten-sync test would force a new `benten-sync →
//! benten-graph` dev-dep edge crossing that layering. Placing it in
//! benten-graph respects both the §8-B-(i) split AND the dependency
//! layering. The `MerkleRangeProofBackend` (mode-b) + signed-
//! checkpoint (mode-c) pure-trait/non-wire surface stays in
//! benten-sync per §8-B "keep benten-graph thin".
//!
//! ## Provenance / R2-map
//!
//! - r2-test-landscape.md **TF-11** + §2.B "the schema-version-bump
//!   conformance arm is freeze-deferred per P-III" + §4-C **C10** row.
//! - Plan **§1.A.FROZEN item 4** (the `SnapshotBlob.schema_version
//!   1→2` bump co-scheduled into the D2 P-III decision-point) + the
//!   **§8-B-(i)** structural resolution + **§8-F** D2 wire-freeze.
//!
//! ## §8-B-(i) P-III pin — EXPLICIT (Ben-ACKED 2026-05-19)
//!
//! The `1→2` bump is a **P-III wire/on-disk format change
//! co-scheduled into the SAME G-CORE-9 D2-freeze Ben decision-point**
//! (§1.A.FROZEN item 4 / §8-F). It is **NOT landed autonomously** by
//! this sub-lane. These pins assert the EXISTING, DOCUMENTED
//! backward-compat path (the `SchemaVersion` strict-mismatch
//! reader-reject IS the migration mechanism the freeze ratifies) +
//! a §3.5m P-III tripwire that the constant is still `1`. Both are
//! GREEN at HEAD (verify-STAYS-regression guards), NOT
//! RED-against-undelivered — flagged distinctly in the §3.6e split
//! (see R3-B4 report).
//!
//! ## R3-brief inherited-discipline pre-flight checklist (§3.6g —
//! reproduced as LITERAL lines, NOT a §-reference; fix-6 directive)
//!
//! - [x] §3.5b HARDENED (pim-1): tests-only.
//! - [x] §3.6b + sub-rule 4 (pim-2): PRODUCTION-ARM (the real `SnapshotBlobBackend::from_bytes` reader) + OBSERVABLE (typed `SchemaVersion` reject) + WOULD-FAIL (silent accept).
//! - [x] §3.6e (pim-12): these are verify-STAYS-regression guards (GREEN, not #[ignore]d) — distinct from the RED mode-b/c staged-pins in the benten-sync file (split documented).
//! - [x] §3.6f (pim-18): production reader call-site, substantive body, aspirational-prose-gap check.
//! - [x] §3.5g / §3.6g / §3.6h / §3.6i / §3.6j: no mint here; report carries canonical disposition; reproduced as literal lines.
//! - [x] §3.13: per-test locals; no shared static.
//! - [x] §3.5h / §3.5l: pre-merge full-workspace verify.
//! - [x] §3.5m P-III: the `1→2` bump is Ben-scheduled at G-CORE-9 — this lane does NOT land it; the constant-is-still-1 pin is the P-III side-effect tripwire.
//! - [x] §3.5n: orchestrator ground-truth every finding.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_graph::backends::snapshot_blob::{
    SNAPSHOT_BLOB_SCHEMA_VERSION, SnapshotBlob, SnapshotBlobBackend, SnapshotBlobError,
};
use std::collections::BTreeMap;

/// **TF-11 / §1.A.FROZEN item 4 / §8-B-(i) — P-III migration-path
/// guard (NOT RED; GREEN at HEAD).**
///
/// Asserts the EXISTING documented backward-compat path: a
/// `SnapshotBlobBackend::from_bytes` reader STRICT-rejects a blob
/// whose declared `schema_version` does not match the build's
/// `SNAPSHOT_BLOB_SCHEMA_VERSION`, surfacing the typed
/// `SnapshotBlobError::SchemaVersion`. When the P-III `1→2` bump
/// lands (the scheduled G-CORE-9 D2-freeze Ben decision), an old
/// `v1` reader strict-rejects a `v2` blob rather than silently
/// mis-decoding — THAT is the migration mechanism the freeze
/// ratifies.
///
/// Would-FAIL if a future change made the reader silently accept a
/// mismatched `schema_version` (defeating the documented migration
/// path the P-III freeze depends on).
#[test]
fn snapshot_blob_schema_version_strict_mismatch_reject_is_the_documented_p3_migration_path() {
    // Forge a blob declaring a FUTURE schema_version (simulating a
    // post-P-III-bump v2 blob seen by a v1 reader). We do NOT bump
    // the production constant.
    let future_version = SNAPSHOT_BLOB_SCHEMA_VERSION + 1;
    let forged = SnapshotBlob {
        schema_version: future_version,
        anchor_cid: None,
        nodes: BTreeMap::new(),
        system_zone_index: BTreeMap::new(),
    };
    let bytes = forged
        .to_canonical_bytes()
        .expect("encode forged-future-version blob");

    let result = SnapshotBlobBackend::from_bytes(&bytes);
    match result {
        Err(SnapshotBlobError::SchemaVersion { expected, actual }) => {
            assert_eq!(
                expected, SNAPSHOT_BLOB_SCHEMA_VERSION,
                "strict-reject reports the build's expected version"
            );
            assert_eq!(
                actual, future_version,
                "strict-reject reports the blob's declared version"
            );
        }
        other => panic!(
            "§8-B-(i) P-III migration-path guard: a schema_version \
             mismatch MUST surface SnapshotBlobError::SchemaVersion \
             (the documented backward-compat reject path the G-CORE-9 \
             D2-freeze 1→2 bump relies on) — got {other:?}. \
             Would-FAIL if the reader silently accepted a mismatched \
             schema_version."
        ),
    }
}

/// **TF-11 / §8-B-(i) / §3.5m — P-III scheduling tripwire (NOT RED;
/// GREEN at HEAD).**
///
/// Asserts the production constant is STILL `1` — i.e. this
/// autonomous sub-lane has NOT performed the P-III `1→2` bump (the
/// scheduled G-CORE-9 D2-freeze Ben decision-point per §1.A.FROZEN
/// item 4 / §8-F). If a future autonomous change bumped it outside
/// the P-III freeze gate, this guard FAILS — the intended P-III
/// side-effect tripwire (§3.5m: P-III wire/CID/on-disk changes are
/// Ben-scheduled, never an orchestrator side-effect).
///
/// At G-CORE-9, when Ben ratifies the D2 freeze + the co-scheduled
/// `1→2` bump, THIS assertion is the one the freeze wave updates
/// (deliberately, in the freeze PR) — a clean, single, intentional
/// edit point rather than a silent drift.
#[test]
fn snapshot_blob_schema_version_constant_is_still_1_p3_bump_not_landed_autonomously() {
    assert_eq!(
        SNAPSHOT_BLOB_SCHEMA_VERSION, 1,
        "§8-B-(i) / §3.5m P-III tripwire: the SnapshotBlob \
         schema_version 1→2 bump is a SCHEDULED G-CORE-9 D2-freeze \
         Ben decision-point (§1.A.FROZEN item 4 / §8-F) — it MUST NOT \
         be landed autonomously by the benten-sync light-client \
         sub-lane. If this fails, an out-of-band P-III bump occurred."
    );
}
