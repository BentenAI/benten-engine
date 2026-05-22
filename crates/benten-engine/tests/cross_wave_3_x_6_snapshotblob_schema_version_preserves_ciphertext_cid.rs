//! Phase-4-Meta-Core — R3-W5 — Cross-wave integration test §4-E:
//! G-CORE-3 × G-CORE-6 / §8-B SnapshotBlob — Encrypted Node
//! ciphertext-CIDs preserve their identity under SnapshotBlob
//! `schema_version 1→2` bump (P-III freeze surface). The §4-E claim:
//! a pre-freeze pin asserts `schema_version=1` at HEAD (the
//! §3.5m P-III tripwire); a post-freeze pin (G-CORE-9 brief)
//! asserts the bump to 2 migrates ciphertext-CIDs identically.
//!
//! ============================================================================
//! GREEN-vs-RED SPLIT (P-III tripwire pattern — same shape as
//! tf11_snapshot_blob_schema_version_p3_migration_path.rs)
//! ============================================================================
//!
//! This file follows the §3.5m P-III pattern from the §8-B-(i)
//! sibling pin in benten-graph: the **schema_version=1 tripwire** is
//! GREEN at HEAD (NOT #[ignore]'d) — it asserts the autonomous bump
//! HAS NOT happened; the bump is a SCHEDULED G-CORE-9 P-III Ben
//! decision, so an autonomous bump would be a HARD-FAIL of §3.5m.
//! The **ciphertext-CID-preservation** pin is RED (#[ignore]'d) — it
//! requires both G-CORE-3d (per-Node AEAD + two-CID mapping) AND
//! the G-CORE-9 1→2 bump to land before it can exercise.
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD c9c11c56)
//! ============================================================================
//!
//!   * `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 1` lives in
//!     `crates/benten-graph/src/backends/snapshot_blob.rs:88` (verified
//!     by R3-W5 author).
//!   * G-CORE-3d (per-Node AEAD + two-CID mapping):  NOT BUILT.
//!     `ciphertext_cid` is not a field of SnapshotBlob at HEAD.
//!   * §4-E composition: when 1→2 lands (G-CORE-9 P-III decision),
//!     existing ciphertext_cids in the table MUST migrate identically
//!     (no re-encryption / no re-AEAD pass; just a struct version
//!     bump).
//!
//! ============================================================================
//! §3.6g LITERAL discipline checklist
//! ============================================================================
//!
//!  1. §3.5b HARDENED — implementer sweeps SECURITY-POSTURE.md +
//!     ENGINE-SPEC.md SnapshotBlob section on the 1→2 bump.
//!  2. §3.6b sub-rule 4 — SPECIFIC arm = ciphertext_cid identity is
//!     INVARIANT across the bump; OBSERVABLE = byte-equal CID
//!     sequences pre- and post-bump for the SAME plaintext_cid set;
//!     WOULD-FAIL on a re-encryption-induced CID drift (the wrong
//!     migration shape).
//!  3. §3.6e — un-ignore at G-CORE-3d + G-CORE-9 bump landing.
//!  4. §3.6f — production call-site = future SnapshotBlob v2 reader
//!     + two-CID mapping persistence; substantive body drives a
//!     real snapshot write then a v2 read.
//!  5. §3.5m P-III — the schema_version=1 tripwire enforces the
//!     bump is NOT landed autonomously.
//!  6. §3.13 — per-test-static.
//!  7. §3.5n — verified version constant = 1 at HEAD; no
//!     ciphertext_cid field exists on SnapshotBlob.
//!
//! Disjointness vs `crates/benten-graph/tests/tf11_snapshot_blob_schema_version_p3_migration_path.rs`
//! (HARD §3.5i): the benten-graph file owns the SchemaVersion-strict-
//! mismatch error path + the constant-is-still-1 tripwire AT THE
//! GRAPH-CRATE UNIT LEVEL. THIS file owns the ENGINE-INTEGRATION
//! cross-wave composition: an encrypted Node written via the engine
//! survives the 1→2 bump with byte-equal ciphertext_cid. Different
//! crate (benten-engine vs benten-graph); different surface.
//!
//! Pin source: R2-test-landscape.md §4-E + plan §1.A.FROZEN item 4
//! (D2 + SnapshotBlob 1→2 co-scheduled P-III at G-CORE-9) + §8-B-(i)
//! structural resolution.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_graph::backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION;

/// §4-E ARM 1 — P-III tripwire (GREEN at HEAD; NOT #[ignore]'d).
/// Asserts the SnapshotBlob schema_version is STILL 1 — i.e. no
/// autonomous bump occurred. This is a §3.5m guard: the 1→2 bump is
/// a scheduled G-CORE-9 P-III Ben decision-point. If it bumps
/// autonomously outside the freeze gate, this fails — which is the
/// intended P-III side-effect tripwire. Mirrors the GREEN guard
/// pattern in `crates/benten-graph/tests/tf11_snapshot_blob_schema_version_p3_migration_path.rs`
/// but at the ENGINE INTEGRATION boundary (so a cross-wave G-CORE-3
/// implementer that touches the schema_version is also caught).
#[test]
fn engine_integration_snapshot_blob_schema_version_is_still_1_p3_tripwire() {
    assert_eq!(
        SNAPSHOT_BLOB_SCHEMA_VERSION, 1,
        "§4-E / §3.5m P-III tripwire (engine integration mirror of \
         the benten-graph sibling): the SnapshotBlob schema_version \
         1→2 bump is a SCHEDULED G-CORE-9 D2-freeze Ben decision-point \
         (§1.A.FROZEN item 4 / §8-F). It MUST NOT be landed \
         autonomously by ANY wave including G-CORE-3d / G-CORE-3e / \
         G-CORE-8. If this fails at the engine-integration level, an \
         out-of-band P-III bump occurred through a cross-wave \
         implementer. The benten-graph sibling pin catches it at the \
         graph-crate unit level; this pin catches it at the engine \
         composition level."
    );
}

/// §4-E ARM 2 — Post-bump ciphertext-CID-preservation (RED until
/// G-CORE-3d ships + G-CORE-9 ratifies the 1→2 bump). The §4-E
/// invariant: after the bump, every existing ciphertext_cid produced
/// pre-bump remains byte-equal in the v2 SnapshotBlob. The bump is
/// a STRUCT VERSION change, not a re-encryption.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9 + G-CORE-3d composition wave (the 1→2 bump + the per-Node AEAD + two-CID mapping landing together; §3.6e)."]
fn ciphertext_cid_identity_preserved_across_schema_version_1_to_2_bump() {
    // Body intent at un-ignore:
    //
    //   // (Step 1) Construct a SnapshotBlob v1 with two encrypted
    //   // Nodes, recording each {plaintext_cid, ciphertext_cid}.
    //   let v1 = build_v1_snapshot_with_two_encrypted_nodes();
    //   let v1_bytes = v1.to_canonical_bytes().unwrap();
    //   let v1_cids: Vec<Cid> = v1.ciphertext_cids();
    //
    //   // (Step 2) Read v1_bytes with a v2 reader; v2 must migrate
    //   // identically.
    //   let v2 = SnapshotBlobBackend::from_bytes_with_migration(&v1_bytes)
    //       .expect("v2 reader migrates v1 bytes; the documented \
    //                backward-compat path is the SchemaVersion-strict-\
    //                reject + a separate migration entry");
    //
    //   // (Step 3) The ciphertext_cids in v2 are byte-equal to v1's.
    //   let v2_cids: Vec<Cid> = v2.ciphertext_cids();
    //   assert_eq!(v1_cids, v2_cids,
    //       "§4-E: 1→2 bump preserves ciphertext-CID identity. \
    //        Would-FAIL if migration re-encrypts (which would \
    //        derive new K(N) → new ciphertext → new ciphertext_cid; \
    //        the WRONG migration shape per §1.A.FROZEN item 15 (g) \
    //        two-CID mapping contract)");
    //
    panic!(
        "RED-PHASE: §4-E ciphertext-CID identity preservation across \
         1→2 bump. At HEAD G-CORE-3d (two-CID mapping) is not built, \
         and the 1→2 bump is the scheduled G-CORE-9 P-III decision. \
         Un-ignore at G-CORE-9 + G-CORE-3d composition wave. Would- \
         FAIL on a re-encryption-induced migration (wrong shape)."
    );
}

/// §4-E ARM 3 — Strict-mismatch path is the documented migration
/// path: a v1 reader presented with a v2 blob strict-rejects via the
/// `SchemaVersion` error (Inv-13 P-III preservation). This pin
/// asserts the engine integration boundary surfaces the typed error
/// to the caller (not a silent skip). Mirrors the benten-graph
/// sibling's `snapshot_blob_schema_version_strict_mismatch_reject`
/// pin at engine integration level.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9 1→2 bump landing (the v1-reader-sees-v2-blob path becomes exercisable post-bump)."]
fn engine_surfaces_typed_schema_version_mismatch_to_caller_post_bump() {
    // Body at un-ignore: load a v2 blob (the bumped format) via an
    // engine method that uses a v1 reader internally; assert the
    // typed SchemaVersion-mismatch error propagates to the caller.
    // The §3.5m discipline: the bump is GREEN-on-graph-side after
    // G-CORE-9; this pin verifies engine integration callers see
    // the strict-reject path, not a silent accept.
    panic!(
        "RED-PHASE: §4-E engine integration surfaces typed strict- \
         mismatch. Un-ignore at G-CORE-9 1→2 bump. Mirrors the \
         tf11_snapshot_blob_schema_version_p3_migration_path.rs sibling \
         at engine boundary."
    );
}
