//! G-COMP-1 pull-forward #1 — ABSOLUTE golden-hex byte-pin for the
//! `AuthorizationGrant` canonical DAG-CBOR (Rows D-9 / D-79).
//!
//! Per Row D-9, the binding-sig tamper-detection pins already landed
//! (`authorization_grant.rs` unit tests + the tf3b corpus). This file
//! adds the MISSING ABSOLUTE hex byte-pin: the full `to_canonical_bytes`
//! DAG-CBOR encoding of a fully-deterministic grant is frozen to literal
//! bytes so no field reorder / serde-shape change / binding-message
//! layout change can slip past silently.
//!
//! # Determinism (golden-hex-via-throwaway-compute, memory M-20)
//!
//! The whole grant is byte-deterministic when built from FIXED seeds:
//! Ed25519 signing (RFC 8032) is deterministic, so the `binding_sig` +
//! `issuer_verifying_key` + `audience_pubkey` + `audience_binding` are
//! all fixed given fixed issuer/audience seeds. The grant is issued via
//! the production-shaped [`AuthorizationGrant::issue_for_test`] (which
//! signs with `SigningKey::from_bytes(issuer_kp.secret_bytes())` — pure
//! Ed25519, NOT the randomized `issue_envelopes_for_test` OsRng path)
//! with an empty `RestrictedScope` + a fixed `exp_secs`. Hex captured
//! via throwaway compute + pasted below.
//!
//! would-FAIL-on-drift: a field reorder, a serde-attribute change (e.g.
//! dropping `#[serde(with = "serde_bytes")]` on a byte field), or the
//! 7-segment binding-message layout changing (which changes `binding_sig`)
//! all change these bytes.

#![allow(clippy::unwrap_used)]

use benten_caps::authorization_grant::AuthorizationGrant;
use benten_caps::restricted_spec::RestrictedScope;
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
/// (`{version: u8, alg: String, secret_bytes: Bytes(32)}`) so a FIXED
/// 32-byte seed can be turned into a deterministic `Keypair` via the
/// public `Keypair::from_seed_bytes` import path. Same field names +
/// same DAG-CBOR canonical encoding ⇒ byte-identical to what
/// `export_seed_envelope` would emit.
#[derive(Serialize)]
struct SeedEnvelopeShim {
    version: u8,
    alg: String,
    secret_bytes: serde_bytes::ByteBuf,
}

/// Deterministic keypair from a fixed 32-byte seed.
fn det_keypair(seed: [u8; 32]) -> Keypair {
    let env = SeedEnvelopeShim {
        version: ENVELOPE_VERSION,
        alg: ENVELOPE_ALG.to_string(),
        secret_bytes: serde_bytes::ByteBuf::from(seed.to_vec()),
    };
    let bytes = serde_ipld_dagcbor::to_vec(&env).unwrap();
    Keypair::from_seed_bytes(&bytes).unwrap()
}

#[test]
fn authorization_grant_canonical_bytes_golden_hex() {
    // Fixed issuer + audience seeds ⇒ fully deterministic grant.
    let issuer_kp = det_keypair([0x11; 32]);
    let audience_kp = det_keypair([0x22; 32]);

    // Empty scope + a fixed absolute expiry. `issue_for_test` signs with
    // the issuer's real Ed25519 key (deterministic) over the 7-segment
    // binding message.
    let grant = AuthorizationGrant::issue_for_test(
        &issuer_kp,
        audience_kp.public_key(),
        RestrictedScope::new(),
        1_000_000_000, // fixed exp_secs
    );

    let bytes = grant.to_canonical_bytes().unwrap();
    let got = to_hex(&bytes);

    // ABSOLUTE golden hex — captured via throwaway compute over the fixed
    // seeds (issuer 0x11.., audience 0x22.., empty scope, exp=1e9).
    // R6-final F-01: regenerated after the binding-message length prefixes
    // migrated LE→BE + `BINDING_SIG_DOMAIN` bumped v4→v5 (only the
    // `binding_sig` value changes; all other fields are byte-identical).
    let expected = "a7647563616ea56861756469656e6365582401711e20a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0686578705f736563731a3b9aca00686e62665f73656373006d6469736372696d696e61746f72006f756e7265736f6c7665645f70656572f46573636f7065a665726f6f7473f6696d61785f6465707468f66e656467655f616c6c6f776c697374f66e6c6162656c5f64656e796c697374f66f6c6162656c5f616c6c6f776c697374f67370726f70657274795f657175616c6974696573a06b62696e64696e675f73696798401833188018b418b118c418c818dc18d01851185218d618b1182d181a185c18e118a618f0182a18db185e18d918b1185318e718a618d3189918df18d3188b1899182f18810018a718cc18be181a188b18c71841182e1855185e1828131842188f18c21843189a18b7185d18e11846184418f118d41893188e183018b1006c6b65795f6d6174657269616ca1656279746573982018aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa18aa6f61756469656e63655f7075626b65795820a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f07061756469656e63655f62696e64696e67582401711e20a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0746973737565725f766572696679696e675f6b65795820d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737";
    assert_eq!(
        got, expected,
        "AuthorizationGrant canonical DAG-CBOR drifted from the frozen v1-beta bytes"
    );

    // Sanity: the deterministic grant still verifies (proves the fixture
    // is a real, self-consistent grant — not just arbitrary bytes).
    let audience_cid = grant.audience_binding;
    grant.verify_binding(audience_cid).unwrap();
}
