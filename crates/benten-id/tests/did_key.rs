//! R3-A RED-PHASE pins for `did:key` DID generation (G14-A1 wave-4a).
//!
//! Pin sources (per r2-test-landscape §2.2 G14-A1 + plan §3 G14-A1
//! must-pass column):
//!
//! - `tests/did_key_generation_deterministic_from_pubkey` — plan §3 G14-A1
//! - `tests/did_key_uses_z_multibase_prefix_per_w3c_spec` — `crypto-minor-3`
//! - `tests/did_key_uses_0xed01_multicodec_for_ed25519_per_w3c_spec` — `crypto-minor-3`
//! - `tests/did_key_resolves_against_external_didkit_test_vector` — `crypto-minor-3`
//!
//! W3C did:key spec (per crypto-minor-3 + `https://w3c-ccg.github.io/did-method-key/`):
//!
//! - **Multibase prefix `z`** = base58-btc.
//! - **Multicodec `0xed01`** = Ed25519 public key.
//! - Form: `did:key:z<base58btc(0xed01 || <32 pubkey bytes>)>`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A1 — plan §3 G14-A1 — did:key deterministic from pubkey"]
fn did_key_generation_deterministic_from_pubkey() {
    // G14-A1 implementer wires this:
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let pk = kp.public_key();
    //   let did1 = benten_id::did::Did::from_public_key(&pk);
    //   let did2 = benten_id::did::Did::from_public_key(&pk);
    //   assert_eq!(did1.as_str(), did2.as_str());
    //   // The DID is a pure function of the public key — no entropy.
    //   let pk_clone = pk.clone(); // owned copy
    //   let did3 = benten_id::did::Did::from_public_key(&pk_clone);
    //   assert_eq!(did1.as_str(), did3.as_str());
    //
    // OBSERVABLE consequence: 3 DIDs from the same pubkey are
    // byte-identical strings. Defends against accidental nondeterminism
    // (e.g. embedding wallclock or a salt) in the DID derivation.
    unimplemented!("G14-A1 wires Did::from_public_key determinism assertion");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-minor-3 — did:key z-multibase prefix"]
fn did_key_uses_z_multibase_prefix_per_w3c_spec() {
    // crypto-minor-3 pin. Per W3C did-key spec, the DID body MUST
    // start with `z` (base58-btc multibase prefix). Other multibase
    // prefixes (e.g. `m` for base64, `f` for base16) are not valid
    // for did:key.
    //
    // Implementer wires:
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let did = benten_id::did::Did::from_public_key(&kp.public_key());
    //   assert!(did.as_str().starts_with("did:key:z"),
    //       "did:key MUST use 'z' multibase prefix per W3C spec; got: {}", did.as_str());
    //   // Body after the `z` prefix decodes as base58-btc.
    //   let body = did.as_str().strip_prefix("did:key:z").unwrap();
    //   let _decoded: Vec<u8> = bs58::decode(body).into_vec().unwrap();
    //
    // OBSERVABLE consequence: did:key strings ARE round-trippable by
    // any spec-conformant DID resolver (didkit, ssi, didcomm) because
    // the multibase prefix is correct.
    unimplemented!(
        "G14-A1 wires did:key 'z' multibase prefix assertion + base58 decode round-trip"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-minor-3 — 0xed01 multicodec for Ed25519"]
fn did_key_uses_0xed01_multicodec_for_ed25519_per_w3c_spec() {
    // crypto-minor-3 pin. Per W3C did-key spec, Ed25519 public keys
    // are tagged with the multicodec varint `0xed01` (varint encoding
    // of the unsigned-LEB128 number 0xed). Other multicodec values
    // identify other key algorithms; using the wrong tag leads to
    // verifiers attempting to interpret an Ed25519 32-byte pubkey as
    // a different algorithm's key material (catastrophic).
    //
    // Implementer wires:
    //   let kp = benten_id::keypair::Keypair::generate();
    //   let did = benten_id::did::Did::from_public_key(&kp.public_key());
    //   let body = did.as_str().strip_prefix("did:key:z").unwrap();
    //   let decoded = bs58::decode(body).into_vec().unwrap();
    //   // First 2 bytes MUST be the 0xed01 varint:
    //   assert_eq!(&decoded[0..2], &[0xed, 0x01],
    //       "did:key Ed25519 multicodec MUST be 0xed01 (varint) per W3C spec");
    //   // Remaining 32 bytes MUST equal the public key:
    //   assert_eq!(&decoded[2..], kp.public_key().to_bytes());
    //
    // OBSERVABLE consequence: spec-conformant resolvers correctly
    // identify the key algorithm from the DID alone.
    unimplemented!("G14-A1 wires did:key 0xed01 multicodec varint assertion");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-minor-3 — external didkit test vector"]
fn did_key_resolves_against_external_didkit_test_vector() {
    // crypto-minor-3 pin. This is the load-bearing
    // INTEROPERABILITY pin: we don't just assert our own format is
    // self-consistent; we cross-check against a fixed test vector
    // from the W3C / didkit reference implementation.
    //
    // Implementer wires (test vector from W3C did-method-key spec
    // `https://w3c-ccg.github.io/did-method-key/#example-1` 2026):
    //
    //   // Known Ed25519 pubkey + corresponding did:key from the W3C spec:
    //   const PUBKEY: [u8; 32] = hex!("a36a3eef9d8db1a89c7c2cd3a5ed6c8c5e1f3...");
    //   const EXPECTED_DID: &str = "did:key:zABCDEF...";
    //   let pk = benten_id::keypair::PublicKey::from_bytes(&PUBKEY).unwrap();
    //   let did = benten_id::did::Did::from_public_key(&pk);
    //   assert_eq!(did.as_str(), EXPECTED_DID,
    //       "did:key derivation must match W3C spec test vector");
    //
    // OBSERVABLE consequence: cross-implementation interoperability.
    // A did:key generated by ssi/didkit/our crate ALL produce
    // byte-identical strings for the same pubkey.
    unimplemented!(
        "G14-A1 wires assertion against W3C did-method-key spec test vector (load-bearing interop)"
    );
}
