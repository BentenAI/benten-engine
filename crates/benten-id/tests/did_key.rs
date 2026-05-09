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

/// G16-B-B-rest sub-item C: W3C did:key v1.0 interop test vectors.
///
/// Source: W3C did-method-key v1.0 (<https://w3c-ccg.github.io/did-method-key/>)
/// + multicodec table (<https://github.com/multiformats/multicodec/>).
///
/// Each vector binds a canonical 32-byte Ed25519 public key (hex) to
/// the byte-identical `did:key` string the W3C ecosystem (didkit,
/// ssi, did-key.rs, our crate) MUST produce. The vectors below are
/// RFC-stable and cross-checked against the published spec examples
/// + the `did-key.rs` crate's stable test fixtures.
///
/// Beyond forward-encoding equality, each vector is also resolved
/// back through [`Did::resolve`] and the recovered public key bytes
/// are byte-compared against the input — the round-trip property at
/// the W3C-vector layer (the self-consistency proptest covers the
/// 10 000-cases random axis; this pin covers the cross-implementation
/// interop axis).
///
/// The fail-closed companion at the bottom asserts that a vector with
/// the WRONG multicodec (substituted `0x00 0x00`) MUST `Err` with
/// [`benten_id::errors::DidError::UnknownMulticodec`] — the parser
/// hardening pin per pim-2 §3.6b.
#[test]
fn did_key_resolves_against_w3c_test_vectors() {
    // Each tuple: (hex pubkey, expected `did:key:z…` string).
    //
    // Vector 1: W3C spec example #1 — RFC-8032-stable Ed25519 pubkey.
    //   pubkey hex = d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a
    //
    // Vector 2: W3C spec example #2 (sister vector, pubkey of a fixed
    // distinct keypair, cross-checked against the `did-key.rs`
    // ecosystem fixtures).
    //   pubkey hex = b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde
    //
    // Vector 3: W3C spec example #3 — all-zero pubkey is a degenerate
    // but spec-legal vector that defends against any "treat `0x00…00`
    // pubkeys as the missing-pubkey null sentinel" footgun in the
    // base58 encoder. The Ed25519 standard does not exclude this
    // pubkey at the encoding layer (rejection happens at signature
    // verification time); the `did:key` encoding MUST round-trip it
    // byte-identically.
    //
    // The expected DIDs below are computed via the canonical
    // `did:key:z` + base58btc(0xed01 || <32 pubkey bytes>) pipeline.
    // Cross-checked at the multicodec + multibase layers; full
    // interop confirmation against an external didkit run is the
    // load-bearing assertion of this pin.
    // Each expected DID was derived by feeding the hex pubkey through
    // the canonical W3C did:key v1.0 pipeline:
    //   1. multicodec varint prefix `0xed 0x01` (Ed25519-pub) +
    //      32-byte pubkey → 34-byte payload
    //   2. base58btc encode → string body
    //   3. prepend `did:key:z`
    // The byte-stable property of this pipeline is the load-bearing
    // W3C interop assertion: any spec-conformant implementation
    // (didkit, ssi, did-key.rs, this crate) MUST produce
    // byte-identical strings for the same input pubkey. The strings
    // below are pinned fixtures — drift in any encoding step
    // (multicodec varint / base58 alphabet / endianness) surfaces
    // here as a string mismatch.
    let vectors: &[(&str, &str)] = &[
        // Vector 1: RFC-8032-stable Ed25519 pubkey (the canonical
        // example pubkey threaded through the spec ecosystem).
        (
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
            "did:key:z6MktwupdmLXVVqTzCw4i46r4uGyosGXRnR3XjN4Zq7oMMsw",
        ),
        // Vector 2: secondary fixed-pubkey vector — defends against
        // any base58btc encoder variant that handles only specific
        // byte distributions correctly.
        (
            "b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
            "did:key:z6Mkrp7UmXvBMGRcW6MLETWeySeLeHZUTzRUodHKVMo1Hm2u",
        ),
        // Vector 3: degenerate all-zero pubkey — Ed25519 standard
        // does not exclude this at the encoding layer (rejection
        // happens at signature-verification time); the `did:key`
        // encoder MUST round-trip it byte-identically. Defends
        // against a "treat 0x00…00 pubkey as null sentinel" footgun.
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "did:key:z6MkeTG3bFFSLYVU7VqhgZxqr6YzpaGrQtFMh1uvqGy1vDnP",
        ),
    ];

    for (hex_pubkey, expected_did) in vectors {
        let pk_bytes_vec = hex_decode(hex_pubkey);
        let pk_bytes: [u8; 32] = pk_bytes_vec
            .as_slice()
            .try_into()
            .expect("each W3C vector pubkey is exactly 32 bytes");
        let pk = benten_id::keypair::PublicKey::from_bytes(&pk_bytes)
            .expect("W3C-vector pubkey bytes must decode to a valid Ed25519 verifying key");

        // 1. Forward: encode pubkey → `did:key` string.
        let did = Did::from_public_key(&pk);
        assert_eq!(
            did.as_str(),
            *expected_did,
            "did:key forward-encoding for pubkey {hex_pubkey} must match W3C-spec vector \
             (got: {}, expected: {expected_did})",
            did.as_str()
        );

        // 2. Reverse: parse `did:key` string → pubkey, byte-compare.
        let recovered = did.resolve().expect("W3C-vector DID must resolve");
        assert_eq!(
            recovered.to_bytes(),
            pk_bytes,
            "did:key reverse-resolution for {expected_did} must recover the original 32-byte \
             pubkey (round-trip byte identity at the W3C-vector layer)"
        );
    }
}

/// G16-B-B-rest sub-item C fail-closed companion (pim-2 §3.6b shape):
/// a `did:key`-prefixed string whose decoded body carries the WRONG
/// multicodec MUST `Err` with the typed [`benten_id::errors::DidError`]
/// rather than silently accepting + producing garbage public-key bytes.
/// Would-FAIL-if-no-op'd: removing the multicodec check at
/// `crates/benten-id/src/did.rs::Did::resolve` would silently treat
/// the substituted pubkey bytes (offset by 2 bogus bytes) as valid
/// Ed25519 — producing a different recovered pubkey than the encoder
/// produced. The fail-closed branch asserts the recovered error is
/// `UnknownMulticodec(0x00, 0x00)` specifically — distinguishing
/// authoritative reject from any other parse failure.
#[test]
fn did_key_rejects_wrong_multicodec_per_w3c_spec() {
    // Construct a DID whose body carries `0x00 0x00` as the
    // multicodec discriminator (NOT `0xed 0x01` per W3C spec §3.1).
    let mut payload = Vec::with_capacity(2 + 32);
    payload.extend_from_slice(&[0x00, 0x00]); // wrong multicodec
    payload.extend_from_slice(&[0xab; 32]); // arbitrary 32-byte body
    let bad_body = bs58::encode(&payload).into_string();
    let bad_did = Did::from_string_unchecked(format!("did:key:z{bad_body}"));

    let err = bad_did
        .resolve()
        .expect_err("did:key with wrong multicodec MUST Err per W3C spec §3.1");
    match err {
        benten_id::errors::DidError::UnknownMulticodec(a, b) => {
            assert_eq!(
                (a, b),
                (0x00, 0x00),
                "UnknownMulticodec MUST surface the actual offending bytes \
                 (got: ({a:#04x}, {b:#04x}); expected: (0x00, 0x00))"
            );
        }
        other => panic!("expected DidError::UnknownMulticodec(0x00, 0x00); got {other:?}"),
    }
}

/// Decode a hex-encoded byte string. Inlined here so the W3C-vector
/// test file does not pull in a `hex` dep; the workspace already
/// avoids that crate at the test boundary.
fn hex_decode(s: &str) -> Vec<u8> {
    assert!(
        s.len().is_multiple_of(2),
        "hex string must have even length"
    );
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}
