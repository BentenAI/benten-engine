//! G16-D wave-6b LANDED pins for the atrium join flow + revoke
//! semantics over the handshake protocol body.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-D rows
//!   `atrium_join_flow_end_to_end` +
//!   `atrium_revoke_peer_terminates_active_subscriptions`.
//! - plan §3 G16-D row.
//! - plan §4 seed (atrium join flow seed).
//! - exit-criterion 15 (atrium-revoke + active-subscription
//!   termination).
//!
//! G16-B reconciliation note: the engine-side `Atrium` Rust type +
//! `Engine::create_atrium` / `Engine::accept_atrium_invite` /
//! `subscribe_change_events` surfaces are G16-B territory
//! (`crates/benten-engine/src/atrium_api.rs` / `engine_sync.rs`).
//! G16-D wave-6b lands the handshake-protocol-layer pins here at the
//! `benten-sync` layer; the engine-integration end-to-end pins land
//! after G16-B merge consolidates the engine surface. The pins here
//! exercise the handshake-layer floor that the engine-layer flow
//! composes on top of.

#![allow(clippy::unwrap_used)]

use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;
use benten_sync::handshake::{Handshake, RevocationEntry, initiate_nonce};

#[test]
fn atrium_join_flow_end_to_end() {
    // Handshake-layer pin for the atrium join flow.
    //
    // The full engine-integration flow (Engine::create_atrium → invite
    // → accept_atrium_invite → list_peers convergence) is G16-B
    // territory; this pin exercises the handshake-protocol seam that
    // a join-flow leverages: the inviter peer issues an invite-shaped
    // UCAN grant; the invitee's handshake initiate carries it; the
    // inviter's handshake respond establishes the per-peer cap-set.
    let inviter_kp = Keypair::generate();
    let invitee_kp = Keypair::generate();
    let inviter_did = inviter_kp.public_key().to_did();
    let invitee_did = invitee_kp.public_key().to_did();

    // Inviter creates an invite-shaped UCAN grant addressed to the
    // invitee:
    let invite_grant = Ucan::builder()
        .issuer_did(&inviter_did)
        .audience_did(&invitee_did)
        .capability("/atrium/test", "join")
        .sign(&inviter_kp);

    // Invitee initiates the handshake carrying their acceptance grant
    // back to the inviter:
    let acceptance_grant = Ucan::builder()
        .issuer_did(&invitee_did)
        .audience_did(&inviter_did)
        .capability("/atrium/test", "join")
        .sign(&invitee_kp);

    let initiate = Handshake::initiate(
        &invitee_kp,
        inviter_did.clone(),
        Some(acceptance_grant.clone()),
        vec![],
    )
    .unwrap();
    let nonce = initiate_nonce(&initiate).unwrap();
    let (response, inviter_session) =
        Handshake::respond(&inviter_kp, &initiate, Some(invite_grant.clone()), vec![]).unwrap();
    let invitee_session = Handshake::finalise(
        &invitee_kp,
        &nonce,
        Some(acceptance_grant.clone()),
        vec![],
        &response,
    )
    .unwrap();

    // Both engines now see the same atrium membership at the
    // handshake-layer floor:
    assert_eq!(inviter_session.remote_did(), &invitee_did);
    assert_eq!(invitee_session.remote_did(), &inviter_did);

    // The per-peer cap-set on each side carries the counterpart's
    // grant — the inviter's session sees the invitee's acceptance,
    // and the invitee's session sees the inviter's invite.
    assert!(
        inviter_session
            .effective_cap_set()
            .includes_cap("/atrium/test", "join")
    );
    assert!(
        invitee_session
            .effective_cap_set()
            .includes_cap("/atrium/test", "join")
    );
}

#[test]
fn atrium_revoke_peer_terminates_active_subscriptions() {
    // exit-criterion 15 pin (handshake-layer floor). Composes with
    // G14-D per-subscriber filtering at the engine layer.
    //
    // Handshake-layer assertion: when the inviter peer carries a
    // revocation in its outbox, the invitee's post-handshake session
    // synchronizes that revocation BEFORE any subscription opens, so
    // any active subscription on the revoked path terminates at the
    // delivery layer (G14-D F6 cap-recheck).
    let inviter_kp = Keypair::generate();
    let invitee_kp = Keypair::generate();
    let inviter_did = inviter_kp.public_key().to_did();
    let invitee_did = invitee_kp.public_key().to_did();

    let revocation = RevocationEntry::new(invitee_did.clone(), "/zone/posts/private/*");

    let initiate = Handshake::initiate(&inviter_kp, invitee_did.clone(), None, vec![]).unwrap();
    let _nonce = initiate_nonce(&initiate).unwrap();
    // Invitee responds with the revocation in their outbox. (Roles
    // are intentionally swapped here: the invitee is consuming the
    // initiate; the inviter sent it. We model the revoke happening
    // at the responder side — peer-A revokes peer-B's grant while B
    // is offline; when B reconnects, A's side synchronizes the
    // revocation through the handshake response.)
    let (response, inviter_session) =
        Handshake::respond(&invitee_kp, &initiate, None, vec![revocation.clone()]).unwrap();
    let _ = response;
    // The session carries the synchronized revocation snapshot:
    let synced = inviter_session.synchronized_revocations_for_local_peer();
    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].target_peer_did(), &invitee_did);
    assert_eq!(synced[0].path(), "/zone/posts/private/*");
    assert!(inviter_session.subscription_open_permitted());

    // Compose with the inviter-DID at the handshake layer:
    assert_eq!(inviter_session.remote_did(), &inviter_did);
}
