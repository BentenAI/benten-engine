//! R3-C RED-PHASE pin: MST diff drains revocation kind first
//! (G16-C wave-6b; per r2-test-landscape §2.4 G16-C + net-blocker-3).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-C row
//!   `mst_diff_drains_revocation_kind_first_under_concurrent_arrival`.
//! - `net-blocker-3` BLOCKER (revocation-message-kind ordered before
//!   data at handshake + MST diff drain).
//! - plan §3 G16-C row.
//!
//! ## What this pins
//!
//! Companion to `atrium_revoke_order.rs::atrium_revocation_message_kind_ordered_before_data_at_handshake`
//! at the MST diff layer: when the MST diff session has both
//! revocation-typed and data-typed messages queued for application,
//! the revocation messages drain FIRST.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-C wave-6b wires MST diff drain priority"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-C wave-6b — net-blocker-3 — MST diff drains revocation first"]
fn mst_diff_drains_revocation_kind_first_under_concurrent_arrival() {
    // net-blocker-3 BLOCKER pin. G16-C implementer wires this:
    //
    //   use benten_sync::mst_proto::{MstDiffSession, MessageKind};
    //   let mut session = MstDiffSession::new();
    //
    //   // Queue mixed message kinds in arrival order:
    //   session.enqueue(MessageKind::Data, data_msg_1);
    //   session.enqueue(MessageKind::Revocation, revoke_msg);
    //   session.enqueue(MessageKind::Data, data_msg_2);
    //   session.enqueue(MessageKind::Revocation, revoke_msg_2);
    //   session.enqueue(MessageKind::Data, data_msg_3);
    //
    //   // Drain order: all revocations, then all data:
    //   let drain: Vec<_> = session.drain().collect();
    //   let n_drained = drain.len();
    //   let revoke_count = drain.iter().take_while(|m| m.kind() == MessageKind::Revocation).count();
    //   assert_eq!(revoke_count, 2);
    //   // Remaining drained = data, in arrival order:
    //   assert!(drain[2..].iter().all(|m| m.kind() == MessageKind::Data));
    //
    // OBSERVABLE consequence: revocations always drain before data
    // at the MST-diff session layer. Defends against the
    // arrival-order ambiguity that net-blocker-3 named as BLOCKER.
    unimplemented!("G16-C wires MST diff drain priority — revocation-first");
}
