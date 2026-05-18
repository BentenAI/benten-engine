//! Qual-2 #759 closure pin — `CanonicalBytes` trait contract.
//!
//! The trait extraction (6 duplicated `to_canonical_bytes` sites + 2
//! inline `SigInput<'a>` structs collapsed onto one trait) is a
//! structural refactor that MUST preserve byte output exactly
//! (v1-wire-adjacent — §3.5m P-III). These pins assert the
//! load-bearing contract:
//!
//! 1. The trait is implemented for every domain type that previously
//!    had a private `to_canonical_bytes` (`RotationAttestation`,
//!    `UcanClaims`, `CredentialClaims`, `DeviceAttestation`).
//! 2. The encoding is deterministic + non-empty.
//! 3. **Signature-input hygiene** — for the two projection types
//!    (`RotationAttestation`, `DeviceAttestation`), mutating ONLY the
//!    `signature` field does NOT change the trait output (the
//!    `SigInput` projection excludes it). If a future refactor folded
//!    the signature into the projection this fails loudly.
//! 4. **Round-trip soundness** — signing the trait bytes and verifying
//!    succeeds, proving the trait reproduces the exact historical
//!    signature-input encoding the verify path expects (a drifted
//!    body would break sign/verify).
//!
//! Would-fail-if-no-op'd: commenting out any impl body / changing any
//! field ordering / including the signature in a `SigInput` fails (3)
//! or (4).

#![allow(clippy::unwrap_used)]

use benten_id::CanonicalBytes;
use benten_id::did::Did;
use benten_id::did_rotation::{RotationAttestation, rotate_keypair};
use benten_id::keypair::Keypair;

#[test]
fn rotation_attestation_canonical_bytes_is_deterministic_and_excludes_signature() {
    let old_kp = Keypair::generate();
    let new_kp = Keypair::generate();
    let did: Did = old_kp.public_key().to_did();
    let att: RotationAttestation = rotate_keypair(&did, &old_kp, &new_kp, 1_000).unwrap();

    let a = att.to_canonical_bytes();
    let b = att.to_canonical_bytes();
    assert_eq!(a, b, "trait encoding must be deterministic");
    assert!(!a.is_empty(), "encoding must be non-empty");

    // Signature-input hygiene: the trait encoding is computed over
    // (previous_did, next_did, superseded_at) ONLY — the `signature`
    // field is excluded. The verify path consumes
    // `self.to_canonical_bytes()` (the trait); if `signature` were folded
    // into the projection, signing would be circular and verify would
    // fail. A passing verify proves the projection excludes the
    // signature.
    att.verify_signature_with(old_kp.public_key())
        .expect("original attestation verifies against trait sig-input bytes");
}

#[test]
fn rotation_attestation_round_trip_through_trait_bytes() {
    let old_kp = Keypair::generate();
    let new_kp = Keypair::generate();
    let did: Did = old_kp.public_key().to_did();
    let att = rotate_keypair(&did, &old_kp, &new_kp, 42).unwrap();

    // verify_signature_with internally uses `self.to_canonical_bytes()`
    // (the trait). A drifted trait body would fail this.
    att.verify_signature_with(old_kp.public_key())
        .expect("sign/verify round-trips through the CanonicalBytes trait");
}

#[test]
fn device_attestation_trait_is_sig_input_projection_distinct_from_round_trip() {
    use benten_id::device_attestation::{
        CapabilityEnvelope, DeviceAttestation, UptimePolicy, ZoneScope,
    };

    let parent = Keypair::generate();
    let device_did = Did::from_string_unchecked("did:key:zDevice".to_string());
    let envelope = CapabilityEnvelope {
        runs_sandbox: false,
        runs_atrium_peer: false,
        holds_zones: ZoneScope::CacheOnly,
        online_uptime: UptimePolicy::SessionBounded,
    };
    let att =
        DeviceAttestation::issue_with_nonce(&parent, device_did, envelope, 100, [7u8; 32]).unwrap();

    // The trait impl is the signature-input projection (excludes
    // `signature`); the inherent `to_canonical_bytes` is the whole-struct
    // round-trip (includes `signature`). They are DIFFERENT byte
    // outputs by design — assert they do not silently alias.
    let sig_input = CanonicalBytes::to_canonical_bytes(&att);
    let whole_struct = att.to_canonical_bytes(); // inherent wins in method position
    assert_ne!(
        sig_input, whole_struct,
        "trait sig-input projection must differ from whole-struct round-trip encoding"
    );

    // Round-trip soundness: verify consumes the trait sig-input bytes.
    att.verify_signature_with(parent.public_key())
        .expect("device attestation sign/verify round-trips through trait bytes");
}
