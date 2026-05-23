//! Phase-4-Meta-Core — ADDL R5 (G-CORE-6b LANDED) — TF-11 P-III half.
//!
//! **Status update 2026-05-23 (G-CORE-6b R5):** the `SnapshotBlob.schema_version
//! 1→2` bump has LANDED in this wave (Ben-authorized autonomously per
//! no-users-yet → the standing P-III caution about existing-data
//! migration does not apply). The tests in this file are UPDATED to
//! reflect the post-bump reality: the constant is now `2`, the typed
//! `SnapshotBlobError::SchemaVersion` lifts to
//! [`ErrorCode::SnapshotBlobSchemaVersionMismatch`] (was `Serialize`
//! pre-G-CORE-6b — the catch-all conflation closed), and the W5
//! forward-compat un-ignored arm now asserts a v2 reader strict-rejects
//! a v1 blob. The in-place struct addition is `merkle_root: Option<Cid>`
//! (§8-B mode-(b) `MerkleRangeProof` hook; `None` until the §8-B
//! consumer wave wires it up). The R3 framing (below, retained for
//! provenance) wrote these tests under the assumption that the bump
//! would NOT land autonomously; that assumption was overridden at the
//! brief level per the no-users-yet rationale.
//!
//! Originally: Agent **R3-B4**. The §8-B-(i) `SnapshotBlob.schema_version 1→2`
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
/// guard (GREEN; post-G-CORE-6b bump).**
///
/// Asserts the documented backward-compat path: a
/// `SnapshotBlobBackend::from_bytes` reader STRICT-rejects a blob
/// whose declared `schema_version` does not match the build's
/// `SNAPSHOT_BLOB_SCHEMA_VERSION`, surfacing the typed
/// `SnapshotBlobError::SchemaVersion`. With the G-CORE-6b `1→2` bump
/// landed, a v2 reader strict-rejects a forged-future v3 blob via the
/// SAME machinery a v1 reader used to reject a v2 blob — the
/// strict-reject is the documented migration mechanism in both
/// directions.
///
/// Would-FAIL if a future change made the reader silently accept a
/// mismatched `schema_version` (defeating the documented migration
/// path the wire-format contract depends on).
#[test]
fn snapshot_blob_schema_version_strict_mismatch_reject_is_the_documented_p3_migration_path() {
    // Forge a blob declaring a FUTURE schema_version (simulating a
    // post-next-bump v3 blob seen by the current v2 reader). We do
    // NOT pre-bump the production constant — we forge the field
    // directly to drive the strict-reject path.
    let future_version = SNAPSHOT_BLOB_SCHEMA_VERSION + 1;
    let forged = SnapshotBlob {
        schema_version: future_version,
        anchor_cid: None,
        nodes: BTreeMap::new(),
        system_zone_index: BTreeMap::new(),
        // v2 (G-CORE-6b): §8-B MerkleRangeProof root hook; None for
        // the strict-reject smoke fixture.
        merkle_root: None,
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
             (the documented backward-compat reject path the wire-format \
             contract relies on) — got {other:?}. \
             Would-FAIL if the reader silently accepted a mismatched \
             schema_version."
        ),
    }
}

/// **TF-11 / §8-B-(i) — post-G-CORE-6b bump pin (GREEN; the bump LANDED).**
///
/// Asserts the production constant is now `2`. The G-CORE-6b R5 wave
/// (Phase-4-Meta-Core, 2026-05-23) landed the `1→2` bump autonomously
/// per Ben's no-users-yet authorization — the standing P-III
/// scheduling caution (§3.5m: P-III wire/CID/on-disk changes are
/// orchestrator-forbidden by default precisely because of existing-data
/// migration concerns) does not apply when there is no existing data
/// in the wild. The pre-bump form of this test asserted `==1` as a
/// scheduling tripwire; that tripwire CORRECTLY fires on the bump,
/// and the bump landing is the intentional edit point that updates
/// it to `==2`.
///
/// At a future cross-version bump (`2→3`, etc.), the same shape applies:
/// a deliberate single-line update in the bump PR documents the new
/// constant; a silent drift would still be caught by this assertion.
#[test]
fn snapshot_blob_schema_version_constant_is_2_after_g_core_6b_bump_landed() {
    assert_eq!(
        SNAPSHOT_BLOB_SCHEMA_VERSION, 2,
        "G-CORE-6b R5 (2026-05-23, P-III Ben-authorized autonomously \
         per no-users-yet): the SnapshotBlob schema_version is now 2; \
         v2 added the `merkle_root: Option<Cid>` field for the §8-B \
         mode-(b) MerkleRangeProof hook. If this fails to `==2`, \
         either the bump regressed or a subsequent bump landed \
         without updating this pin."
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

/// **TF-11 / W5 ARM-W5-1 — Post-bump v2 reader strict-rejects a v1 blob
/// (un-ignored at G-CORE-6b R5; FORWARD-COMPAT direction).**
///
/// This is the FORWARD-COMPAT direction of the migration path (the
/// `_strict_mismatch_reject_is_the_documented_p3_migration_path` pin
/// above covers the backward direction via a forged v3 blob). Both
/// directions MUST strict-reject. Without this arm, a future agent
/// could ship a v2 reader that silently accepts v1 (a leniency that
/// defeats the strict-reject contract).
///
/// At G-CORE-6b, the bump LANDED + this body is the live assertion
/// (no longer staged-pin commentary).
#[test]
fn snapshot_blob_v2_reader_strict_rejects_v1_blob_forward_compat_direction() {
    // Forge a blob declaring schema_version = 1 (the pre-bump version
    // a legacy on-disk peer-handed blob would carry). Note: we
    // construct the v2 struct shape (including the v2 `merkle_root`
    // tail-append field) but with the schema_version DISCRIMINATOR
    // field set to 1 — that is what an actual v1 producer would
    // emit pre-bump (canonical bytes differ in both the
    // schema-version discriminator AND the tail-append-field
    // absence). For this test the discriminator alone is enough to
    // drive the strict-reject — the reader checks the discriminator
    // before any structural validation of the trailing fields.
    let v1_payload = SnapshotBlob {
        schema_version: 1, // The PRE-G-CORE-6b version.
        anchor_cid: None,
        nodes: BTreeMap::new(),
        system_zone_index: BTreeMap::new(),
        merkle_root: None,
    };
    let v1_bytes = v1_payload
        .to_canonical_bytes()
        .expect("encode v1-discriminator blob");

    // The post-bump reader (SNAPSHOT_BLOB_SCHEMA_VERSION == 2)
    // strict-rejects via SchemaVersion error.
    let result = SnapshotBlobBackend::from_bytes(&v1_bytes);
    match result {
        Err(SnapshotBlobError::SchemaVersion { expected, actual }) => {
            assert_eq!(
                expected, SNAPSHOT_BLOB_SCHEMA_VERSION,
                "v2 reader reports build's expected version = {SNAPSHOT_BLOB_SCHEMA_VERSION}"
            );
            assert_eq!(actual, 1, "v2 reader reports observed v1 discriminator");
        }
        other => panic!(
            "Forward-compat: v2 reader MUST strict-reject v1 blob; \
             got {other:?}. Would-FAIL on lenient accept (which would \
             defeat the strict-reject contract)."
        ),
    }
}

/// **TF-11 / W5 ARM-W5-2 — Typed-error-code identity at G-CORE-6b.**
///
/// The `SnapshotBlobError::SchemaVersion` variant maps to the typed
/// [`ErrorCode::SnapshotBlobSchemaVersionMismatch`] (G-CORE-6b minted
/// this code; the pre-bump mapping was the catch-all
/// [`ErrorCode::Serialize`]). The R3-authored pre-bump form of this
/// test asserted `==Serialize` as a STABILITY pin under the assumption
/// the bump would NOT land autonomously; that assumption was overridden
/// at the G-CORE-6b brief level per no-users-yet, and the cross-version
/// mismatch was intentionally LIFTED out of the generic `Serialize`
/// family so callers can match it specifically. This is the only
/// surface change that needed updating across the §3.5g cross-language
/// ErrorCode mirror; the TS-side `errors.generated.ts` is regenerated
/// atomically in the same wave.
///
/// Would-FAIL if a future agent un-lifts the typed code back into a
/// generic catch-all family, or if the variant identity in
/// `SnapshotBlobError` is re-shaped without updating the mapping.
#[test]
fn snapshot_blob_schema_version_error_maps_to_typed_snapshot_blob_schema_version_mismatch_post_g_core_6b()
 {
    use benten_errors::ErrorCode;
    use benten_graph::backends::snapshot_blob::SnapshotBlobError;
    // Construct the variant directly (no I/O).
    let err = SnapshotBlobError::SchemaVersion {
        expected: 2,
        actual: 99,
    };
    // The §3.5g cross-language ErrorCode mirror property: this
    // variant's ErrorCode mapping is now the typed
    // SnapshotBlobSchemaVersionMismatch (G-CORE-6b lift; mirrors the
    // GraphSchemaVersionMismatch posture for the snapshot-blob surface).
    let code: ErrorCode = err.code();
    assert_eq!(
        code,
        ErrorCode::SnapshotBlobSchemaVersionMismatch,
        "§3.5g + §4-E post-G-CORE-6b: SnapshotBlobError::SchemaVersion \
         MUST map to ErrorCode::SnapshotBlobSchemaVersionMismatch \
         (lifted out of the generic Serialize family at G-CORE-6b). \
         The TS-side errors.generated.ts mirror tracks \
         E_SNAPSHOT_BLOB_SCHEMA_VERSION_MISMATCH. Would-FAIL if a \
         future agent un-lifts the typed code back into a catch-all \
         family, or re-shapes the SnapshotBlobError variant identity."
    );
}
