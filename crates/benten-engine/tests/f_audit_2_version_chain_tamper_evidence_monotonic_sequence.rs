//! F-AUDIT-2 (R3-W6 gov-audit) — the audit log is an Anchor + immutable
//! Version Nodes + CURRENT pointer (Crosby-Wallach tamper-evident log
//! equivalent); the sequence advances monotonically per admin WRITE, and a
//! dedup/no-op WRITE does NOT advance the sequence / emit a phantom event
//! (Inv-13 side-channel re-asked at the membership-audit layer).
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-AUDIT-2; merges
//! K1-tamper + T-G2 + GNI-15):
//!   - R0.3 plan §2.6, §3.8, GN-4, §9.1-6.
//!   - Clone-shape:
//!     `crates/benten-graph/tests/inv_13_dedup_path_does_not_advance_audit_sequence.rs`
//!     + `inv_13_dedup_does_not_emit_changeevent.rs`
//!     + `crates/benten-graph/tests/get_node_verifies_content_hash_on_read.rs`
//!     (tamper mid-chain Version Node → CID linkage fails on read).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 audit version-chain surface (`AuditChain` over membership admin
//! ops) does not exist at this SHA. This file commits a self-contained
//! stub-shim so it compiles green at baseline; bodies `unimplemented!()`
//! so a forgotten un-ignore fails LOUD. W6 R5 implementer:
//!   1. DELETE `mset_w6_audit_chain_stub`,
//!   2. INSERT `use benten_membership_set::audit::AuditChain;`,
//!   3. UN-IGNORE the stub-driven tests,
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! The stub-driven arms pin: (1) append-only monotonic sequence; (2) dedup
//! does NOT advance / emit; (3) tampering a mid-chain Version Node breaks
//! the CID linkage detected on read. The baseline `#[test]` arm drives the
//! REAL `Engine` `audit_sequence()` monotonicity so the red-phase chain is
//! anchored to a live substrate property.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_engine::Engine;

// =====================================================================
// RED-PHASE stub-shim — DELETE at W6 implementation; replace with:
//     use benten_membership_set::audit::{AuditChain, AuditChainError, AdminOp};
// =====================================================================
mod mset_w6_audit_chain_stub {
    //! Local stub matching the intended W6 `AuditChain` surface — an
    //! append-only Anchor + immutable Version Nodes + CURRENT. Bodies
    //! `unimplemented!()`.

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum AdminOp {
        AdmitMember,
        KickMember,
        RotateKey,
        PromoteRole,
        GovernanceChange,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum AuditChainError {
        /// A mid-chain Version Node's content hash does not match its
        /// declared CID — the log was tampered (Crosby-Wallach linkage
        /// break detected on read).
        TamperDetectedLinkageBroken { at_seq: u64 },
        /// Sequence must be strictly monotonic / append-only.
        NonMonotonicAppend,
    }

    /// The append-only audit log. `seq()` reads the CURRENT sequence;
    /// `append_op` advances exactly once per genuine admin op; replaying
    /// an identical op is a dedup pure-read (no advance, no event).
    pub struct AuditChain;

    impl AuditChain {
        pub fn new(_set_id: &[u8; 32]) -> Self {
            unimplemented!(
                "W6 stub — R5 replaces this module with \
                 `use benten_membership_set::audit::AuditChain;`"
            )
        }
        /// Current sequence number (number of immutable Version Nodes on
        /// the chain since the Anchor).
        pub fn seq(&self) -> u64 {
            unimplemented!("W6 stub")
        }
        /// Append a genuine admin op → new immutable Version Node, CURRENT
        /// advances, sequence +1. Returns the new sequence.
        pub fn append_op(&mut self, _actor: &[u8; 32], _op: AdminOp) -> u64 {
            unimplemented!("W6 stub — append-only Version-Node chain")
        }
        /// Replay an identical op already at CURRENT → dedup pure-read.
        /// Returns `true` iff a phantom event was (incorrectly) emitted.
        /// Real impl returns `false` (Inv-13: no phantom event/advance).
        pub fn replay_identical_op_emitted_phantom_event(
            &mut self,
            _actor: &[u8; 32],
            _op: AdminOp,
        ) -> bool {
            unimplemented!("W6 stub — dedup is a pure-read; no phantom event")
        }
        /// Verify the full chain on read; tampering a mid-chain Version
        /// Node's bytes breaks its CID linkage (Crosby-Wallach).
        pub fn verify_with_tampered_node_at(
            &self,
            _seq: u64,
        ) -> Result<(), AuditChainError> {
            unimplemented!("W6 stub — content-hash-on-read linkage check")
        }
    }
}

use mset_w6_audit_chain_stub::{AdminOp, AuditChain, AuditChainError};

fn fixed_cid(byte: u8) -> [u8; 32] {
    [byte; 32]
}

/// F-AUDIT-2 (a): the audit chain advances monotonically — one Version
/// Node per genuine admin op, append-only.
///
/// would-FAIL if W6 lets the sequence stall, go backwards, or skip.
#[test]
#[ignore = "RED-PHASE: F-AUDIT-2 — audit chain advances monotonically per admin WRITE; un-ignore at W6 R5 (delete mset_w6_audit_chain_stub; insert real `use`)"]
fn audit_chain_advances_monotonically_per_admin_write() {
    let mut chain = AuditChain::new(&fixed_cid(0x51));
    let actor = fixed_cid(0xA0);

    let s0 = chain.seq();
    let s1 = chain.append_op(&actor, AdminOp::AdmitMember);
    assert_eq!(
        s1,
        s0 + 1,
        "F-AUDIT-2: the first admin op MUST advance the sequence by exactly 1"
    );
    let s2 = chain.append_op(&actor, AdminOp::PromoteRole);
    assert_eq!(
        s2,
        s1 + 1,
        "F-AUDIT-2: each subsequent admin op MUST advance the sequence by \
         exactly 1 (append-only monotonic Version-Node chain)"
    );
    assert!(
        s2 > s1 && s1 > s0,
        "F-AUDIT-2: the sequence MUST be strictly monotonic — append-only"
    );
}

/// F-AUDIT-2 (b): a dedup/no-op replay does NOT advance the sequence and
/// does NOT emit a phantom event (Inv-13 re-asked at the membership-audit
/// layer). Negative control for the monotonic assertion.
#[test]
#[ignore = "RED-PHASE: F-AUDIT-2 — dedup replay does not advance sequence / emit phantom event (Inv-13); un-ignore at W6 R5"]
fn dedup_replay_does_not_advance_sequence_or_emit_phantom_event() {
    let mut chain = AuditChain::new(&fixed_cid(0x51));
    let actor = fixed_cid(0xA0);

    let _ = chain.append_op(&actor, AdminOp::AdmitMember);
    let seq_before = chain.seq();

    let phantom = chain.replay_identical_op_emitted_phantom_event(&actor, AdminOp::AdmitMember);
    assert!(
        !phantom,
        "F-AUDIT-2: replaying an identical admin op already at CURRENT is a \
         dedup pure-read — it MUST NOT emit a phantom ChangeEvent (Inv-13 \
         side-channel re-asked at the membership-audit layer)"
    );

    let seq_after = chain.seq();
    assert_eq!(
        seq_before, seq_after,
        "F-AUDIT-2: a dedup replay MUST NOT advance the audit sequence; \
         before={seq_before}, after={seq_after}"
    );
}

/// F-AUDIT-2 (c): tampering a mid-chain immutable Version Node breaks its
/// CID linkage, detected on read (Crosby-Wallach tamper-evident log).
///
/// would-FAIL if W6 stores audit events in a mutable structure where a
/// mid-chain edit goes undetected.
#[test]
#[ignore = "RED-PHASE: F-AUDIT-2 — mid-chain tamper breaks CID linkage on read (Crosby-Wallach); un-ignore at W6 R5"]
fn mid_chain_tamper_breaks_cid_linkage_on_read() {
    let mut chain = AuditChain::new(&fixed_cid(0x51));
    let actor = fixed_cid(0xA0);
    let _ = chain.append_op(&actor, AdminOp::AdmitMember);
    let mid_seq = chain.append_op(&actor, AdminOp::PromoteRole);
    let _ = chain.append_op(&actor, AdminOp::KickMember);

    let verify = chain.verify_with_tampered_node_at(mid_seq);
    match verify {
        Err(AuditChainError::TamperDetectedLinkageBroken { at_seq }) => {
            assert_eq!(
                at_seq, mid_seq,
                "F-AUDIT-2: tamper detection MUST identify the exact mid-chain \
                 sequence whose CID linkage broke"
            );
        }
        other => panic!(
            "F-AUDIT-2: tampering a mid-chain immutable Version Node MUST be \
             detected on read as a broken CID linkage (Crosby-Wallach); got \
             {other:?} instead of TamperDetectedLinkageBroken"
        ),
    }
}

/// F-AUDIT-2 (substrate): REAL `Engine` audit-sequence monotonicity — the
/// `#[test]` baseline arm anchoring the red-phase chain to a live property
/// (the membership audit chain rides on the same committed-writes counter).
#[test]
fn engine_audit_sequence_is_monotonic_across_distinct_enforced_writes() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let alice = engine.caps().create_principal("alice").unwrap();
    let seq_a = engine.audit_sequence();
    let _g1 = engine
        .caps()
        .grant_capability(&alice, "audit:set:read")
        .unwrap();
    let seq_b = engine.audit_sequence();
    let _g2 = engine
        .caps()
        .grant_capability(&alice, "audit:set:write")
        .unwrap();
    let seq_c = engine.audit_sequence();

    assert!(
        seq_c > seq_b && seq_b > seq_a,
        "F-AUDIT-2 substrate: distinct enforced WRITES MUST advance the audit \
         sequence monotonically; a={seq_a}, b={seq_b}, c={seq_c}"
    );
}
