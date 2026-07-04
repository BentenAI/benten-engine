//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pin for the
//! `DropBundle` canonical DAG-CBOR (Rows D-9 / D-79).
//!
//! Per Row D-9, the offline-consume + tamper + size-envelope pins
//! already landed (the tf3f corpus). This file adds the MISSING ABSOLUTE
//! hex byte-pin: the full `to_cbor_bytes` DAG-CBOR encoding of a
//! fully-deterministic `DropBundle` is frozen to literal bytes so no
//! field reorder / serde-shape change slips past silently.
//!
//! # per_node_attestation is pinned as the RESERVED placeholder
//!
//! Per the brief + the field docstring (Row D-66), `per_node_attestation`
//! is a FROZEN `serde_bytes` `Vec<u8>` size-reservation placeholder at
//! v1-beta (the typed `Vec<Signature>` upgrade was DEFERRED past the
//! freeze). This fixture carries a non-empty placeholder blob so the
//! frozen-placeholder shape is captured in the golden bytes; a future
//! commit that upgrades the field type to a typed `Vec<Signature>` would
//! change the CBOR + fail this pin, signalling the additive-upgrade
//! must be a deliberate wire event.
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! A `_for_test` builder is NOT usable — it seals `content` via a random
//! ChaCha20-Poly1305 nonce. So this fixture constructs the `DropBundle`
//! struct DIRECTLY from deterministic parts: FIXED-seed Ed25519 keypairs
//! (RFC-8032 deterministic signing), `content` cells built from
//! directly-constructed fixed-nonce `AeadEnvelope`s, an empty
//! `RestrictedScope`, and a fixed placeholder attestation. The
//! `envelope_sig` is a deterministic Ed25519 signature over the frozen
//! `build_envelope_message`. Hex captured via throwaway compute + pasted
//! below.
//!
//! This is a WIRE-SHAPE fixture (it pins the on-disk CBOR framing), not a
//! decrypt-round-trip fixture — the content nonces are synthetic, so a
//! `consume_offline` would fail AEAD auth; the pin asserts the frozen
//! *bytes*, which is the freeze-discipline surface.

#![allow(clippy::unwrap_used)]

use benten_caps::authorization_grant::AuthorizationGrant;
use benten_caps::restricted_spec::RestrictedScope;
use benten_core::Cid;
use benten_crypto_suite::aead::AeadEnvelope;
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_drop::bundle::{DropBundle, DropBundleVersion, DropContentMode, EncryptedContent};
use benten_drop::envelope_sig::{build_envelope_message, sign_envelope};
use benten_graph::aead_wrap::EncryptedNode;
use benten_id::keypair::{ENVELOPE_ALG, ENVELOPE_VERSION, Keypair};
use serde::Serialize;

/// Lowercase-hex encoder (no `hex` crate dep in this workspace).
fn to_hex(bytes: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Replicates the private `benten_id::keypair::SeedEnvelope` serde shape
/// so a FIXED 32-byte seed becomes a deterministic `Keypair`.
#[derive(Serialize)]
struct SeedEnvelopeShim {
    version: u8,
    alg: String,
    secret_bytes: serde_bytes::ByteBuf,
}

fn det_keypair(seed: [u8; 32]) -> Keypair {
    let env = SeedEnvelopeShim {
        version: ENVELOPE_VERSION,
        alg: ENVELOPE_ALG.to_string(),
        secret_bytes: serde_bytes::ByteBuf::from(seed.to_vec()),
    };
    let bytes = serde_ipld_dagcbor::to_vec(&env).unwrap();
    Keypair::from_seed_bytes(&bytes).unwrap()
}

fn fixed_cid(seed: u8) -> Cid {
    Cid::from_blake3_digest([seed; 32])
}

/// A fixed content cell — `EncryptedNode::Whole` at 0x647a with a fixed
/// nonce + ciphertext (deterministic; does NOT call `wrap()`).
fn fixed_content_cell(cid_seed: u8) -> EncryptedContent {
    let node = EncryptedNode::Whole {
        plaintext_cid: fixed_cid(cid_seed),
        envelope: AeadEnvelope {
            format_version: 0x01,
            cipher_codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            nonce: vec![
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
            ],
            ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33],
        },
    };
    EncryptedContent::from_encrypted_node(&node).unwrap()
}

/// Build the fully-deterministic DropBundle fixture.
fn fixture_bundle() -> DropBundle {
    let issuer_kp = det_keypair([0x11; 32]);
    let audience_kp = det_keypair([0x22; 32]);

    // Deterministic auth grant (Ed25519 sign is RFC-8032 deterministic).
    let auth_grant = AuthorizationGrant::issue_for_test(
        &issuer_kp,
        audience_kp.public_key(),
        RestrictedScope::new(),
        1_000_000_000,
    );
    let audience = auth_grant.audience_binding;

    let content = vec![fixed_content_cell(0xA1), fixed_content_cell(0xA2)];

    let mut bundle = DropBundle {
        version: DropBundleVersion::V1,
        mode: DropContentMode::OfflineDrop,
        spec_cid: fixed_cid(0x33),
        audience,
        auth_grant,
        content,
        restricted_spec: RestrictedScope::default(),
        // Row D-66: FROZEN reserved-size placeholder (Vec<u8>) at v1-beta.
        per_node_attestation: vec![0xA5; 52],
        envelope_sig: Vec::new(),
        issuer_verifying_key: issuer_kp.public_key().to_bytes().to_vec(),
    };

    // Deterministic envelope-sig over the frozen header message.
    let msg = build_envelope_message(&bundle);
    bundle.envelope_sig = sign_envelope(&issuer_kp, &msg);
    bundle
}

#[test]
fn drop_bundle_canonical_bytes_golden_hex() {
    let bundle = fixture_bundle();
    let bytes = bundle.to_cbor_bytes().unwrap();
    let got = to_hex(&bytes);

    // ABSOLUTE golden hex — captured via throwaway compute over the fixed
    // fixture (issuer 0x11.., audience 0x22.., 2 fixed-nonce content
    // cells, empty scope, reserved 52-byte per_node_attestation).
    let expected = "aa646d6f64656b4f66666c696e6544726f7067636f6e74656e7482a1656279746573583f3d0001711e20a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1ae01647a0c0102030405060708090a0b0cdeadbeef00112233a1656279746573583f3d0001711e20a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2ae01647a0c0102030405060708090a0b0cdeadbeef001122336776657273696f6ea1637461676256316861756469656e6365582401711e20a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f068737065635f636964582401711e2033333333333333333333333333333333333333333333333333333333333333336a617574685f6772616e74a7647563616ea56861756469656e6365582401711e20a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0686578705f736563731a3b9aca00686e62665f73656373006d6469736372696d696e61746f72006f756e7265736f6c7665645f70656572f46573636f7065a665726f6f7473f6696d61785f6465707468f66e656467655f616c6c6f776c697374f66e6c6162656c5f64656e796c697374f66f6c6162656c5f616c6c6f776c697374f67370726f70657274795f657175616c6974696573a06b62696e64696e675f736967984018df187618cd189118761896188b183c187418c718d518ec184b18f6181a18d0181a187205185318e818da184818af185d18d218ec183618af18af1821182518a818810c189618d3188318b2185818fd1895184718870d18dc18741876183818cb185218d218cc18f0184f189c186818c018bc16183c18f51885016c6b65795f6d6174657269616ca1656279746573982018aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa6f61756469656e63655f7075626b65795820a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f07061756469656e63655f62696e64696e67582401711e20a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0746973737565725f766572696679696e675f6b65795820d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787376c656e76656c6f70655f736967584046f5bb9f11e1e2349f5abfecf91f08c698ba348a0fbb5e5b32827da3c6f75ee4b0db4fa7c696c946931d25abe01212f3b1fd5acc7d72fa54bf4c07f386bc0f0f6f726573747269637465645f73706563a665726f6f7473f6696d61785f6465707468f66e656467655f616c6c6f776c697374f66e6c6162656c5f64656e796c697374f66f6c6162656c5f616c6c6f776c697374f67370726f70657274795f657175616c6974696573a0746973737565725f766572696679696e675f6b65795820d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737747065725f6e6f64655f6174746573746174696f6e5834a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5";
    assert_eq!(
        got, expected,
        "DropBundle canonical DAG-CBOR drifted from the frozen v1-beta bytes"
    );

    // The bundle round-trips through parse (proves it is a real,
    // structurally-valid v1 bundle — not just arbitrary bytes).
    let reparsed = DropBundle::parse_cbor_bytes(&bytes).unwrap();
    assert_eq!(reparsed.content_count(), 2);
    // Envelope-sig verifies (the header binding is self-consistent).
    reparsed.verify_envelope_signature().unwrap();
}
