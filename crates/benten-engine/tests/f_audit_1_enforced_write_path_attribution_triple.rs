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
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The `benten-membership-set` crate (15th workspace member) + its
//! `audit` module + the `AdminOp`/`emit_audit_event_via_engine` surface
//! DO NOT YET EXIST at this SHA — R5 (the W6 closing wave) mints them.
//! Per the in-tree RED-phase precedent (`tf3a_structural_kdf_*`), this
//! file commits a **local self-contained stub-shim module** matching the
//! intended W6 public surface so the file COMPILES GREEN at baseline +
//! `#[ignore = "RED-PHASE…"]` keeps the runtime gate (pim-12). The stub
//! bodies `unimplemented!()` so a forgotten un-ignore / left-in stub
//! fails LOUD, never silent-green (the OPPOSITE of a pim-18 SHAPE-trap).
//!
//! The W6 R5 closing-wave implementer MUST:
//!   1. DELETE the local `mset_w6_audit_stub` module,
//!   2. INSERT `use benten_membership_set::audit::{...};`,
//!   3. UN-IGNORE the stub-driven tests (`#[ignore = "RED-PHASE…"]` →
//!      `#[test]`),
//!   4. Verify all pins PASS green.
//! Reviewer verifies landing-status (un-ignored + green), not just
//! spec-pin presence (pim-12 §3.6e).
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Two complementary arms:
//!   (a) The `enforced_write_*` STUB-driven arms exercise the production
//!       audit-emit surface (membership-set → engine enforced WRITE);
//!       load-bearing property = the triple is populated ONLY via the
//!       enforced path, never via a bare backend `put_node`.
//!   (b) The `engine_enforced_path_*` arm drives the **REAL** `Engine`
//!       at baseline (no stub) — `audit_sequence()` advances on an
//!       enforced grant WRITE but NOT on a dedup-replay — proving the
//!       enforced-vs-unenforced distinction is live in the substrate the
//!       W6 audit chain rides on. This arm is `#[test]` (green now).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_engine::Engine;

// =====================================================================
// RED-PHASE stub-shim — DELETE at W6 implementation; replace with:
//     use benten_membership_set::audit::{
//         AdminOp, AuditEmitResult, emit_audit_event_via_engine,
//         emit_audit_event_via_bare_put,
//     };
// =====================================================================
mod mset_w6_audit_stub {
    //! Local stub matching the intended W6 `benten_membership_set::audit`
    //! public surface. Every method `unimplemented!()`s so any forgotten
    //! `#[ignore]` / stub-left-in-place fails LOUD (not silent-green).

    /// The administrative operation an audit Version Node records. W6 real
    /// shape lives in `benten_membership_set::audit::AdminOp`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum AdminOp {
        AdmitMember,
        KickMember,
        RotateKey,
        PromoteRole,
        GovernanceChange,
    }

    /// The attribution triple as observed on the emitted audit Version
    /// Node. `None` for any field the ingest path did not know — a bare
    /// `put_node` leaves ALL three `None`. The enforced engine WRITE
    /// populates all three.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct AuditEmitResult {
        pub actor_cid: Option<[u8; 32]>,
        pub handler_cid: Option<[u8; 32]>,
        pub capability_grant_cid: Option<[u8; 32]>,
        /// Whether the audit version-chain advanced (CURRENT moved).
        pub chain_advanced: bool,
    }

    /// W6 stub: emit an audit event through the `is_actor_active`-gated
    /// engine WRITE path. Real impl routes through
    /// `Engine`-enforced WRITE so the triple is SET.
    pub fn emit_audit_event_via_engine(
        _set_id: &[u8; 32],
        _actor: &[u8; 32],
        _grant_cid: &[u8; 32],
        _op: AdminOp,
    ) -> AuditEmitResult {
        unimplemented!(
            "W6 stub — R5 replaces this module with `use benten_membership_set::audit::*;`; \
             real impl routes the admin op through the `is_actor_active`-gated enforced \
             engine WRITE so (actor_cid, handler_cid, capability_grant_cid) is SET"
        )
    }

    /// W6 stub: the ANTI-pattern — emitting via a bare backend `put_node`.
    /// Real impl demonstrates the triple is left `None` and the chain does
    /// NOT advance — this path is NOT the audit path (store.rs:468).
    pub fn emit_audit_event_via_bare_put(
        _set_id: &[u8; 32],
        _actor: &[u8; 32],
        _op: AdminOp,
    ) -> AuditEmitResult {
        unimplemented!("W6 stub — bare `put_node` leaves the attribution triple unset")
    }
}

use mset_w6_audit_stub::{
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
#[ignore = "RED-PHASE: F-AUDIT-1 — audit-event via enforced engine WRITE sets the attribution triple; un-ignore at W6 R5 (delete mset_w6_audit_stub; insert real `use benten_membership_set::audit::*;`)"]
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
#[ignore = "RED-PHASE: F-AUDIT-1 — bare put_node leaves attribution triple None (NOT the audit path); un-ignore at W6 R5"]
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

/// F-AUDIT-1 (b'): SUBSTANTIVE baseline arm — drives the REAL `Engine`
/// (no stub) to prove the enforced-vs-unenforced audit-sequence
/// distinction is LIVE in the substrate the W6 audit chain rides on.
/// `Engine::audit_sequence()` advances on an enforced grant WRITE but NOT
/// on a dedup-replay of identical bytes (pure-read short-circuit). This is
/// `#[test]` (green now) — it is NOT red-phase; it pins the enforced-path
/// substrate W6 builds on so the red-phase stub assertions above are
/// anchored to a real engine property.
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
