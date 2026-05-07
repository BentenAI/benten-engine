//! G16-C wave-6b LANDED pin: MST diff drains revocation kind first
//! per r2-test-landscape §2.4 G16-C + net-blocker-3 BLOCKER.
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
//! the revocation messages drain FIRST regardless of arrival order.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `MstDiffSession::enqueue` + `drain` API.
//! Would FAIL if the drainer were silently no-op'd to FIFO across
//! both kinds.

#![allow(clippy::unwrap_used)]

use benten_sync::mst::MstCid;
use benten_sync::mst_proto::{MessageKind, MstDiffMessage, MstDiffSession};

fn cid_seed(seed: u8) -> MstCid {
    MstCid::from_blake3_digest([seed; 32])
}

#[test]
fn mst_diff_drains_revocation_kind_first_under_concurrent_arrival() {
    // net-blocker-3 BLOCKER pin.
    let mut session = MstDiffSession::new();

    // Queue mixed message kinds in arrival order (data first):
    session.enqueue(MstDiffMessage::data(cid_seed(0xD1), vec![1]));
    session.enqueue(MstDiffMessage::revocation(cid_seed(0xA1), b"r1".to_vec()));
    session.enqueue(MstDiffMessage::data(cid_seed(0xD2), vec![2]));
    session.enqueue(MstDiffMessage::revocation(cid_seed(0xA2), b"r2".to_vec()));
    session.enqueue(MstDiffMessage::data(cid_seed(0xD3), vec![3]));

    // Drain order: all revocations, then all data.
    let drain: Vec<_> = session.drain();
    let n_drained = drain.len();
    assert_eq!(n_drained, 5);

    let revoke_count = drain
        .iter()
        .take_while(|m| m.kind == MessageKind::Revocation)
        .count();
    assert_eq!(
        revoke_count, 2,
        "all revocations must drain before any data per net-blocker-3"
    );

    // Remaining drained: data, in arrival order.
    assert!(drain[2..].iter().all(|m| m.kind == MessageKind::Data));
    assert_eq!(drain[2].cid, cid_seed(0xD1));
    assert_eq!(drain[3].cid, cid_seed(0xD2));
    assert_eq!(drain[4].cid, cid_seed(0xD3));

    // Revocations preserve their relative arrival order WITHIN tier.
    assert_eq!(drain[0].cid, cid_seed(0xA1));
    assert_eq!(drain[1].cid, cid_seed(0xA2));
}
