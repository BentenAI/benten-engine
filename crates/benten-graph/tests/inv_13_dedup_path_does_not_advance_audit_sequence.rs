//! Phase 2a R3 security — Inv-13 dedup audit-sequence (atk-3 / sec-r1-4).
//!
//! **Attack class (companion to `inv_13_dedup_does_not_emit_changeevent`).**
//! The audit log maintains a monotonically-advancing sequence counter per
//! transaction commit. If the dedup path silently bumps the audit sequence
//! (e.g. via a shared counter in the transaction machinery), an attacker
//! who re-puts identical bytes observes the sequence increment — linking
//! "nothing happened" to an observable time-ordered effect.
//!
//! **Prerequisite.** Same as sibling: attacker reaches engine-privileged
//! write path. Observer can read the audit sequence or infer it from
//! ordering side-channels.
//!
//! **Attack sequence.**
//!  1. Observe current audit sequence S.
//!  2. Re-issue identical grant bytes via `grant_capability`.
//!  3. Observe audit sequence post-call.
//!  4. Mitigation: the sequence MUST be unchanged — the dedup path is
//!     pure-read (Compromise #N+1).
//!
//! **Impact.** Side-channel information leak about transaction counter;
//! also cleanliness regression — audit log gains phantom "commits" that
//! never produced a ChangeEvent.
//!
//! **Recommended mitigation.** Same as sibling: §9.11 row 3 branches
//! BEFORE any transaction.pending_ops accounting. No counter bump, no
//! event, no audit trace.
//!
//! **Red-phase contract.** Phase 1 HEAD has no public audit-sequence API;
//! the test is shape-locked against the G5-A-era mitigation. Until G5-A
//! lands a `TransactionStats` / `audit_sequence()` accessor, the test is
//! `#[ignore]`d. A compilation-only fixture asserts current APIs still
//! resolve.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// atk-3 companion: audit sequence does not advance on dedup.
///
/// G11-A Wave 1: `Engine::audit_sequence()` landed as a public accessor
/// reading the per-engine committed-writes counter. The dedup branch in
/// `RedbBackend::put_node_with_context` is a pure-read short-circuit
/// (§9.11 row 3) — it returns the existing CID without advancing the
/// committed-writes total, so a re-issued grant observes the same
/// counter value.
#[test]
fn inv_13_dedup_path_does_not_advance_audit_sequence() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let alice = engine.create_principal("alice").unwrap();

    let _cid_1 = engine
        .grant_capability(&alice, "store:post:write")
        .expect("first issuance");

    let seq_before = engine.audit_sequence();
    let _cid_2 = engine
        .grant_capability(&alice, "store:post:write")
        .expect("dedup path");
    let seq_after = engine.audit_sequence();
    assert_eq!(
        seq_before, seq_after,
        "dedup path (Compromise #N+1) MUST NOT advance the audit \
         sequence; before={seq_before}, after={seq_after}"
    );
}
