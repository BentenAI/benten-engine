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

// ===========================================================================
// R3-W5 EXTENSION (Phase-4-Meta-Core R3 — cross-wave-integration partition):
// W5 owns the freeze-conformance arms of TF-11 per R2 §7. The R3-B4 pins
// (above) cover the strict-mismatch reject + the P-III tripwire. The W5
// extensions below add the FREEZE-conformance complementary arms:
//   ARM-W5-1: post-bump v2 reader rejects v1 blob (forward-compat
//             side of the migration; mirrors the existing
//             v1-reader-rejects-v2 arm). RED until 1→2 bump lands.
//   ARM-W5-2: typed-error-code is the deterministic surface even
//             across the bump (the typed reject doesn't accidentally
//             change variant identity at G-CORE-9, preserving the
//             ErrorCode mirror per §3.5g).
// ===========================================================================

/// **TF-11 / W5 ARM-W5-1 — Post-bump v2 reader strict-rejects a v1 blob.**
///
/// This is the FORWARD-COMPAT direction of the migration path (the
/// existing pin above is BACKWARD direction: v1-reader sees v2-blob).
/// Both directions MUST strict-reject. Without this arm, a future
/// agent could ship a v2 reader that silently accepts v1 (a leniency
/// that defeats the P-III strict-reject contract).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9 1→2 bump landing (when SNAPSHOT_BLOB_SCHEMA_VERSION = 2). Mirrors the existing v1-reader-rejects-v2 arm at the forward direction. r2-test-landscape §7 W5 + §4-E."]
fn snapshot_blob_v2_reader_strict_rejects_v1_blob_forward_compat_direction() {
    // Body intent at un-ignore (post-G-CORE-9 1→2 bump):
    //
    //   // Forge a blob declaring schema_version = 1 (a v1 blob).
    //   let v1_payload = SnapshotBlob {
    //       schema_version: 1,  // The PRE-bump version after the 1→2 lands.
    //       anchor_cid: None,
    //       nodes: BTreeMap::new(),
    //       system_zone_index: BTreeMap::new(),
    //   };
    //   let v1_bytes = v1_payload.to_canonical_bytes().unwrap();
    //
    //   // The post-bump reader (SNAPSHOT_BLOB_SCHEMA_VERSION == 2)
    //   // strict-rejects a v1 blob via SchemaVersion error.
    //   let result = SnapshotBlobBackend::from_bytes(&v1_bytes);
    //   match result {
    //       Err(SnapshotBlobError::SchemaVersion { expected, actual }) => {
    //           assert_eq!(expected, SNAPSHOT_BLOB_SCHEMA_VERSION); // = 2
    //           assert_eq!(actual, 1);
    //       }
    //       other => panic!(
    //           "Forward-compat: v2 reader MUST strict-reject v1 blob; \
    //            got {other:?}. Would-FAIL on lenient accept (which \
    //            would defeat the P-III strict-reject contract)."
    //       ),
    //   }
    //
    panic!(
        "RED-PHASE (W5 extension): post-bump v2-reader-rejects-v1 \
         forward-compat arm. Un-ignore at G-CORE-9 1→2 bump landing. \
         Pre-bump (HEAD) this body cannot run because the const is 1; \
         post-bump it asserts the SAME strict-reject machinery fires \
         on the inverse direction (forward compat). Would-FAIL if a \
         future agent ships a v2 reader that silently accepts v1."
    );
}

/// **TF-11 / W5 ARM-W5-2 — Typed-error-code identity preserved
/// across the 1→2 bump.**
///
/// The `SnapshotBlobError::SchemaVersion` variant identity (its
/// position in the enum + its mapping to `ErrorCode::Serialize` per
/// `snapshot_blob.rs:204`) MUST NOT shift at the G-CORE-9 bump. A
/// silent variant-shape change would break the §3.5g cross-language
/// ErrorCode mirror (and the TS-side handler patterns). This pin is
/// a GREEN regression-guard against the bump silently re-shaping the
/// error variant.
#[test]
fn snapshot_blob_schema_version_error_maps_to_errorcode_serialize_invariant() {
    use benten_errors::ErrorCode;
    use benten_graph::backends::snapshot_blob::SnapshotBlobError;
    // Construct the variant directly (no I/O).
    let err = SnapshotBlobError::SchemaVersion {
        expected: 1,
        actual: 99,
    };
    // The §3.5g cross-language ErrorCode mirror property: this
    // variant's ErrorCode mapping is stable across the bump. Without
    // a stable mapping, the TS-side mirror (errors.generated.ts) would
    // drift across the freeze.
    let code: ErrorCode = err.code();
    assert_eq!(
        code,
        ErrorCode::Serialize,
        "§3.5g + §4-E: SnapshotBlobError::SchemaVersion MUST map to \
         ErrorCode::Serialize (per snapshot_blob.rs:204 at HEAD); this \
         mapping is the load-bearing cross-language mirror surface. \
         Would-FAIL if a future agent (at the 1→2 bump or any other \
         wave) shifts the variant-to-ErrorCode mapping, breaking the \
         TS-side errors.generated.ts mirror."
    );
}
