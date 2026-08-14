//! F-AUDIT-1 (R3-W6 gov-audit) — audit-event MUST flow through the
//! ENFORCED engine-API WRITE path so the attribution triple
//! `(actor_cid, handler_cid, capability_grant_cid)` is SET; a bare
//! backend `put_node` leaves the triple `None` and is NOT the audit path.
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-AUDIT-1; merges
//! K1 + T-G1 + GNI-14, m-15 GNC-2):
//!   - R0.5 plan §2.6 (governance-as-signed-config-Node + audit-as-
//!     version-chain), §3.8 GNC-2 (`is_actor_active`-gated WRITE), §4.2,
//!     §9.1-6 (governance/audit graph-native exit criterion).
//!   - `crates/benten-graph/src/store.rs:468` — `ChangeEvent` attribution
//!     fields are `Option` BECAUSE the ingest path may or may not know
//!     them: an engine-API write fills them in, a bare `put_node` via the
//!     backend leaves them unset. Tamper-evidence is NOT free — it is a
//!     property of routing through the enforced path.
//!   - Clone-shape: `crates/benten-core/tests/attribution_mirror.rs`
//!     (Inv-14/Inv-13 attribution-string-mirror) +
//!     `crates/benten-graph/tests/inv_13_dedup_path_does_not_advance_audit_sequence.rs`.
//!
//! # SHIPPED STATUS (R17 retense; formerly RED-PHASE pim-12 §3.6e)
//!
//! The `benten-membership-set` crate (15th workspace member) + its
//! `audit` module + the `AdminOp`/`emit_audit_event_via_engine` surface
//! EXIST at HEAD. This file `use`s the REAL surface (see the `use` below);
//! every arm is a live `#[test]` (NO `#[ignore]`). The prior RED-PHASE
//! staging — a local self-contained stub-shim module (`mset_w6_audit_stub`)
//! matching the intended W6 public surface, `#[ignore = "RED-PHASE…"]`-gated
//! until the W6 closing wave minted the crate — is fully discharged: the
//! stub is deleted, the real `use` is wired, and the arms run green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Two complementary arms:
//!   (a) The `enforced_write_*` arms exercise the production
//!       audit-emit surface (membership-set → engine enforced WRITE);
//!       load-bearing property = the triple is populated ONLY via the
//!       enforced path, never via a bare backend `put_node`.
//!   (b) The `engine_enforced_path_*` arm drives the **REAL** `Engine`
//!       — `audit_sequence()` advances on an enforced grant WRITE but NOT
//!       on a dedup-replay — proving the enforced-vs-unenforced distinction
//!       is live in the substrate the W6 audit chain rides on.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_engine::Engine;

// =====================================================================
// W6 R5 (Wave w-gov-audit): real `benten_membership_set::audit` surface.
// =====================================================================
use benten_membership_set::audit::{
    AdminOp, AuditEmitResult, emit_audit_event_via_bare_put, emit_audit_event_via_engine,
};

fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// F-AUDIT-1 (a): admitting a member through the ENFORCED engine WRITE
/// path populates the full `(actor_cid, handler_cid, capability_grant_cid)`
/// triple AND advances the audit version-chain.
///
/// would-FAIL if W6 emits audit Nodes via a bare `put_node` (triple stays
/// `None`) — the tamper-evidence-is-not-free property.
#[test]
fn enforced_write_path_populates_full_attribution_triple() {
    let set_id = fixed_cid(0x51);
    let actor = fixed_cid(0xA0);
    let grant = fixed_cid(0x67);

    let result = emit_audit_event_via_engine(&set_id, &actor, &grant, AdminOp::AdmitMember);

    assert!(
        result.actor_cid.is_some(),
        "F-AUDIT-1: the ENFORCED engine WRITE path MUST populate actor_cid \
         (store.rs:468 — an engine-API write fills the triple in); would-FAIL \
         if W6 emits via a bare backend put_node"
    );
    assert!(
        result.handler_cid.is_some(),
        "F-AUDIT-1: the enforced path MUST populate handler_cid"
    );
    assert!(
        result.capability_grant_cid.is_some(),
        "F-AUDIT-1: the enforced path MUST populate capability_grant_cid — \
         the audit Node records WHICH grant authorized the admin op"
    );
    assert_eq!(
        result.capability_grant_cid,
        Some(grant),
        "F-AUDIT-1: the recorded grant CID MUST be the grant that authorized \
         the op, not a synthesized or zero value"
    );
    assert!(
        result.chain_advanced,
        "F-AUDIT-1: a genuine admin op MUST advance the audit version-chain"
    );
}

/// F-AUDIT-1 (b): the ANTI-pattern — emitting via a bare backend
/// `put_node` leaves the triple `None` and does NOT advance the chain.
/// This is the negative control proving the enforced path is load-bearing
/// (without it the positive assertion is trivially satisfiable).
#[test]
fn bare_put_node_leaves_attribution_triple_unset_and_chain_not_advanced() {
    let set_id = fixed_cid(0x51);
    let actor = fixed_cid(0xA0);

    let result = emit_audit_event_via_bare_put(&set_id, &actor, AdminOp::AdmitMember);

    assert!(
        result.actor_cid.is_none()
            && result.handler_cid.is_none()
            && result.capability_grant_cid.is_none(),
        "F-AUDIT-1: a bare backend put_node MUST leave ALL three attribution \
         fields None (store.rs:468) — it is NOT the audit path; tamper-evidence \
         is NOT free, it is a property of routing through the enforced WRITE"
    );
    assert!(
        !result.chain_advanced,
        "F-AUDIT-1: a bare put_node MUST NOT advance the audit version-chain"
    );
}

/// F-AUDIT-1 (b'): SUBSTANTIVE arm — drives the REAL `Engine`
/// to prove the enforced-vs-unenforced audit-sequence
/// distinction is LIVE in the substrate the W6 audit chain rides on.
/// `Engine::audit_sequence()` advances on an enforced grant WRITE but NOT
/// on a dedup-replay of identical bytes (pure-read short-circuit). It pins
/// the enforced-path substrate W6 builds on so the membership-set audit-emit
/// assertions above are anchored to a real engine property.
#[test]
fn engine_enforced_path_advances_audit_sequence_but_dedup_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let alice = engine.caps().create_principal("alice").unwrap();

    let seq_before_first = engine.audit_sequence();
    let _first = engine
        .caps()
        .grant_capability(&alice, "audit:set:write")
        .expect("first enforced grant WRITE must succeed");
    let seq_after_first = engine.audit_sequence();
    assert!(
        seq_after_first > seq_before_first,
        "F-AUDIT-1 substrate: an enforced grant WRITE MUST advance the audit \
         sequence (the version-chain CURRENT moves); before={seq_before_first}, \
         after={seq_after_first}"
    );

    // Dedup-replay of identical bytes is a pure-read short-circuit — the
    // audit sequence MUST NOT advance (no phantom audit event).
    let seq_before_dedup = engine.audit_sequence();
    let _dup = engine
        .caps()
        .grant_capability(&alice, "audit:set:write")
        .expect("dedup path returns existing CID");
    let seq_after_dedup = engine.audit_sequence();
    assert_eq!(
        seq_before_dedup, seq_after_dedup,
        "F-AUDIT-1 substrate: a dedup-replay MUST NOT advance the audit \
         sequence — no phantom audit event on a no-op WRITE; \
         before={seq_before_dedup}, after={seq_after_dedup}"
    );
}
