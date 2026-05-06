//! G14-A1 wave-4a — `did:key` test pins (un-ignored at landing).
//!
//! Pin sources (per `r2-test-landscape` §2.2 G14-A1 + plan §3 G14-A1).
//!
//! W3C did:key spec (`https://w3c-ccg.github.io/did-method-key/`):
//! `did:key:z<base58btc(0xed01 || <32 pubkey bytes>)>`.

#![allow(clippy::unwrap_used)]

use benten_id::did::Did;
use benten_id::keypair::Keypair;

#[test]
fn did_key_generation_deterministic_from_pubkey() {
    let kp = Keypair::generate();
    let pk = kp.public_key().clone();
    let did1 = Did::from_public_key(&pk);
    let did2 = Did::from_public_key(&pk);
    assert_eq!(did1.as_str(), did2.as_str());

    let pk_clone = pk.clone();
    let did3 = Did::from_public_key(&pk_clone);
    assert_eq!(did1.as_str(), did3.as_str());
}

#[test]
fn did_key_uses_z_multibase_prefix_per_w3c_spec() {
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    assert!(
        did.as_str().starts_with("did:key:z"),
        "did:key MUST use 'z' multibase prefix per W3C spec; got: {}",
        did.as_str()
    );
    let body = did.as_str().strip_prefix("did:key:z").unwrap();
    let _decoded: Vec<u8> = bs58::decode(body).into_vec().unwrap();
}

#[test]
fn did_key_uses_0xed01_multicodec_for_ed25519_per_w3c_spec() {
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    let body = did.as_str().strip_prefix("did:key:z").unwrap();
    let decoded = bs58::decode(body).into_vec().unwrap();
    assert!(decoded.len() >= 2 + 32, "decoded body too short");
    assert_eq!(
        &decoded[0..2],
        &[0xed, 0x01],
        "did:key Ed25519 multicodec MUST be 0xed01 (varint) per W3C spec"
    );
    assert_eq!(
        &decoded[2..2 + 32],
        kp.public_key().to_bytes().as_slice(),
        "did:key body bytes after multicodec MUST equal the 32-byte public key"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-minor-3 — external didkit test vector lands at G14-A2 follow-up if external test infra is wired (out of scope for G14-A1 canary; W3C spec example #1 vector is RFC-stable but pulling exact hex into the codebase requires a side-by-side review of the spec's appendix A test vectors which is G14-A2 cryptography-reviewer scope)"]
fn did_key_resolves_against_external_didkit_test_vector() {
    // Stays #[ignore]'d until G14-A2 cryptography-reviewer pass.
    // The internal round-trip property (across 10 000 cases) is
    // covered by `prop_did_key.rs::prop_did_key_round_trip_byte_identity`,
    // which is the load-bearing self-consistency pin. The external-
    // vector pin is the cross-implementation interop assertion;
    // wiring it requires confirming the canonical W3C example #1
    // hex vector against the spec's appendix A — G14-A2 scope.
    unreachable!("RED-PHASE; un-ignored at G14-A2 with a verified W3C test vector");
}
