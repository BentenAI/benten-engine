//! Phase 2a R3 security — Inv-13 dedup no ChangeEvent (atk-3 / sec-r1-4).
//!
//! **Attack class.** Plan §9.11 5-row matrix: row 3 (`EnginePrivileged` +
//! content matches registered bytes) returns `Ok(cid_dedup)` — the write
//! is a no-op content-addressed dedup. But if the transaction machinery
//! STILL pushes the PendingOp to `pending_ops` and fans out a
//! `ChangeEvent`, an attacker can manufacture audit-log events that look
//! like genuine attribution by re-putting bit-identical bytes.
//!
//! **Prerequisite.** Attacker reaches the engine-privileged write path
//! (e.g. via any code-path that triggers `put_node_with_context`). Re-puts
//! an already-registered grant Node.
//!
//! **Attack sequence.**
//!  1. Alice is granted `store:post:write` via `grant_capability`. Event 1
//!     fires on the change stream.
//!  2. Without Inv-13 dedup hardening, `grant_capability(alice, "store:post:write")`
//!     again goes through the privileged path, re-hashes to the same CID,
//!     and fans out Event 2 — but it carries a NEW attribution timestamp
//!     that looks fresh in the audit log.
//!  3. Attacker repeats N times, inflating the audit log and manufacturing
//!     a trail of "separate grants" that the reviewer interprets as
//!     distinct authorisations.
//!
//! **Impact.** Audit-log forgery via legitimate dedup. Compromise #N+1
//! (new named compromise, sec-r1-4 / atk-3): dedup IS a pure-read semantic
//! — no ChangeEvent emitted.
//!
//! **Recommended mitigation.** §9.11 row 3 (privileged + content matches)
//! branches BEFORE `transaction.pending_ops.push` → return existing CID
//! without fanning out any event.
//!
//! **Red-phase contract.** Phase 1 HEAD emits a ChangeEvent on every
//! `put_node_with_context` regardless of whether the CID is already
//! present — the test observes the dedup path emitting an event today,
//! fails the no-event assertion. G5-A lands dedup branching; test passes.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// atk-3: re-issue an identical grant via the engine-privileged path. The
/// first issuance MAY emit a ChangeEvent (legitimate — first observation);
/// the second MUST NOT emit (dedup is pure-read per Compromise #N+1).
#[test]
fn inv_13_dedup_does_not_emit_changeevent() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let alice = engine.create_principal("alice").unwrap();

    // Attach the probe BEFORE the second grant so we see only the dedup
    // emission (or, under Phase-2a's mitigation, nothing).
    let cid_1 = engine
        .grant_capability(&alice, "store:post:write")
        .expect("first issuance");

    // Subscribe AFTER first issuance so we isolate the dedup emission.
    let probe = engine.subscribe_change_events();

    let cid_2 = engine
        .grant_capability(&alice, "store:post:write")
        .expect("second issuance — dedup path");

    // Content-addressing: same grant → same CID (sanity).
    assert_eq!(
        cid_1, cid_2,
        "identical grant bytes must produce identical CID; got {cid_1} vs \
         {cid_2}"
    );

    // MITIGATION (Compromise #N+1, §9.11 row 3): dedup path MUST NOT
    // emit a ChangeEvent. Today, put_node_with_context fans out on every
    // call — the test observes the emission and fails, driving R5 G5-A
    // to branch before pending_ops.push.
    let events = probe.drain();
    assert!(
        events.is_empty(),
        "Inv-13 dedup (atk-3): re-issuing an identical grant MUST NOT \
         emit a ChangeEvent. Phase-1 HEAD fires {} event(s) on the dedup \
         path — driving R5 G5-A to branch before pending_ops.push. \
         Got: {events:?}",
        events.len()
    );
}
