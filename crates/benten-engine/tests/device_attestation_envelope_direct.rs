//! §13.8 Direct unit tests for `DeviceAttestationEnvelope` public API
//! surface (Phase-3 G16-D wave-6b cryptographic envelope).
//!
//! Pin source: `docs/future/phase-3-backlog.md` §13.8 (BLOCKER —
//! public-API direct-test pin gap: ~12 surfaces have zero direct
//! tests, only exercised implicitly through `Engine::open_atrium` +
//! `apply_atrium_merge` integration paths).
//!
//! ## What this pins
//!
//! The 6-method public surface of
//! [`benten_engine::engine_sync::DeviceAttestationEnvelope`]:
//!   - `new_unsigned()` — legacy attestation=None constructor
//!   - `new_signed(...)` — V2 signed constructor
//!   - `declared_device_did()` — accessor
//!   - `to_canonical_bytes()` — DAG-CBOR serialization
//!   - `from_canonical_bytes(...)` — DAG-CBOR deserialization + version
//!     validation
//!   - `verify(...)` — composition of envelope-signature check +
//!     `Acceptor::accept_at` + payload-hash binding
//!
//! ## Coverage matrix
//!
//! - Round-trip (new_signed → to_canonical_bytes → from_canonical_bytes
//!   → declared_device_did matches).
//! - Round-trip (new_unsigned → to_canonical_bytes → from_canonical_bytes).
//! - Signature-tamper failure-path → `E_DEVICE_ATTESTATION_FORGED`.
//! - Payload-tamper failure-path → `E_DEVICE_ATTESTATION_FORGED`.
//! - Acceptor failure-path (replay via nonce store) →
//!   `E_DEVICE_ATTESTATION_FORGED`.
//! - Version validation: V3+ rejected at decode with
//!   `AtriumError::InvalidState`.
//! - new_unsigned `verify` is a permissive backward-compat no-op
//!   (per docstring: receiver falls back to local `device_cid`).
//!
//! ## Pairs with
//!
//!   - `tests/integration/atrium_two_device.rs` —
//!     `forged_device_did_rejected_at_envelope_verify` /
//!     `replayed_envelope_rejected_by_acceptor_nonce_store` /
//!     `frame_pair_payload_swap_rejected_by_payload_hash_binding` /
//!     `future_wire_version_rejected_at_decode` — INTEGRATION shape
//!     drives the same failure-mode contracts through the full
//!     two-device sync apex. The direct tests here close the
//!     pin-gap at the type-level + composition boundary.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_engine::engine_sync::DeviceAttestationEnvelope;
use benten_id::device_attestation::{
    Acceptor, CapabilityEnvelope, DeviceAttestation, FreshnessPolicy, UptimePolicy, ZoneScope,
};
use benten_id::did::Did;
use benten_id::keypair::Keypair;

fn issue_attestation(parent_kp: &Keypair, device_did: Did) -> DeviceAttestation {
    DeviceAttestation::issue(
        parent_kp,
        device_did,
        CapabilityEnvelope {
            runs_sandbox: true,
            holds_zones: ZoneScope::Full,
            online_uptime: UptimePolicy::AlwaysOn,
            runs_atrium_peer: true,
        },
    )
    .unwrap()
}

/// §13.8 round-trip pin: `new_signed` → `to_canonical_bytes` →
/// `from_canonical_bytes` → `declared_device_did` matches.
///
/// Asserts the V2 canonical-bytes shape is round-trip-stable + the
/// declared device-DID survives the encode/decode boundary verbatim.
#[test]
fn new_signed_canonical_bytes_round_trip_preserves_declared_device_did() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_attestation(&parent_kp, device_did.clone());

    let loro_payload = b"round-trip-payload-bytes";
    let envelope =
        DeviceAttestationEnvelope::new_signed(attestation, loro_payload, &device_kp).unwrap();

    // Pre-encode invariants.
    assert_eq!(envelope.version, DeviceAttestationEnvelope::WIRE_VERSION);
    assert_eq!(
        envelope.declared_device_did(),
        Some(device_did.as_str()),
        "declared_device_did MUST return the attestation's device_did"
    );
    assert!(
        !envelope.envelope_signature.is_empty(),
        "new_signed MUST populate envelope_signature"
    );

    // Encode + decode.
    let bytes = envelope.to_canonical_bytes().unwrap();
    let decoded = DeviceAttestationEnvelope::from_canonical_bytes(&bytes).unwrap();

    assert_eq!(decoded.version, envelope.version);
    assert_eq!(decoded.payload_hash, envelope.payload_hash);
    assert_eq!(decoded.session_nonce, envelope.session_nonce);
    assert_eq!(decoded.envelope_signature, envelope.envelope_signature);
    assert_eq!(
        decoded.declared_device_did(),
        Some(device_did.as_str()),
        "round-tripped envelope MUST preserve declared_device_did"
    );
}

/// §13.8 round-trip pin: `new_unsigned` → canonical-bytes → decode.
///
/// Asserts the legacy attestation=None shape round-trips cleanly + the
/// declared device-DID is None (backward-compat fallback path the
/// receiver falls back on local `device_cid` for).
#[test]
fn new_unsigned_canonical_bytes_round_trip_carries_no_declared_device_did() {
    let envelope = DeviceAttestationEnvelope::new_unsigned();

    assert_eq!(envelope.version, DeviceAttestationEnvelope::WIRE_VERSION);
    assert!(envelope.attestation.is_none());
    assert_eq!(envelope.payload_hash, [0u8; 32]);
    assert_eq!(envelope.session_nonce, [0u8; 32]);
    assert!(envelope.envelope_signature.is_empty());
    assert_eq!(envelope.declared_device_did(), None);

    let bytes = envelope.to_canonical_bytes().unwrap();
    let decoded = DeviceAttestationEnvelope::from_canonical_bytes(&bytes).unwrap();

    assert_eq!(decoded.declared_device_did(), None);
    assert!(decoded.attestation.is_none());
}

/// §13.8 verify-success pin: a freshly-constructed V2 envelope
/// verifies cleanly against the matching Acceptor + payload.
#[test]
fn verify_succeeds_for_signed_envelope_with_matching_payload_and_acceptor() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_attestation(&parent_kp, device_did);

    let loro_payload = b"verify-success-payload";
    let envelope =
        DeviceAttestationEnvelope::new_signed(attestation, loro_payload, &device_kp).unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    envelope
        .verify(loro_payload, &acceptor, 0)
        .expect("freshly-constructed signed envelope MUST verify");
}

/// §13.8 verify-success pin: an `attestation = None` envelope skips
/// verification entirely (legacy backward-compat semantics — receiver
/// falls back to its own `device_cid` per `Engine::apply_atrium_merge`).
#[test]
fn verify_is_noop_for_unsigned_envelope_backward_compat() {
    let envelope = DeviceAttestationEnvelope::new_unsigned();
    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    // Any payload + any timestamp must pass — the verify is a no-op
    // for the legacy attestation=None path.
    envelope
        .verify(b"any-payload", &acceptor, 0)
        .expect("attestation=None envelope verify MUST be a no-op (legacy fallback)");
    envelope
        .verify(b"another-payload", &acceptor, 1_000_000_000)
        .expect("attestation=None envelope verify MUST be no-op regardless of time");
}

/// §13.8 verify-failure pin: tampered envelope_signature MUST reject
/// with `E_DEVICE_ATTESTATION_FORGED` (DID-forgery defense).
#[test]
fn verify_rejects_tampered_envelope_signature_with_forged_code() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_attestation(&parent_kp, device_did);

    let loro_payload = b"tampered-sig-payload";
    let mut envelope =
        DeviceAttestationEnvelope::new_signed(attestation, loro_payload, &device_kp).unwrap();

    // Flip every byte in the signature. The resulting signature is
    // overwhelmingly unlikely to validate against any input.
    for byte in &mut envelope.envelope_signature {
        *byte = !*byte;
    }

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    let err = envelope
        .verify(loro_payload, &acceptor, 0)
        .expect_err("tampered envelope_signature MUST reject");

    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "tampered signature MUST surface E_DEVICE_ATTESTATION_FORGED; got {err:?}"
    );
}

/// §13.8 verify-failure pin: tampered payload_hash MUST reject with
/// `E_DEVICE_ATTESTATION_FORGED` (frame-pair binding defense). A
/// MITM that swaps the Loro payload while preserving the envelope is
/// detected via the BLAKE3 mismatch.
#[test]
fn verify_rejects_swapped_payload_with_forged_code_frame_pair_binding() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_attestation(&parent_kp, device_did);

    let signed_payload = b"original-payload-the-envelope-signed-over";
    let swapped_payload = b"different-payload-bytes-the-mitm-substituted";
    let envelope =
        DeviceAttestationEnvelope::new_signed(attestation, signed_payload, &device_kp).unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    let err = envelope
        .verify(swapped_payload, &acceptor, 0)
        .expect_err("payload swap MUST reject via payload_hash binding");

    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "swapped payload MUST surface E_DEVICE_ATTESTATION_FORGED; got {err:?}"
    );
}

/// §13.8 verify-failure pin: Acceptor rejection (here via parent-issued
/// attestation-nonce replay) surfaces as `E_DEVICE_ATTESTATION_FORGED`.
/// The same attestation cannot be successfully verified twice against
/// the same Acceptor — the nonce-store catches the replay.
#[test]
fn verify_rejects_replayed_attestation_nonce_with_forged_code() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_attestation(&parent_kp, device_did);

    let loro_payload = b"replay-test-payload";
    // Two envelopes sharing the same attestation (the parent-issued
    // nonce is reused). Each envelope has a fresh session_nonce + a
    // fresh signature; the Acceptor's nonce-store is the binding.
    let envelope_1 =
        DeviceAttestationEnvelope::new_signed(attestation.clone(), loro_payload, &device_kp)
            .unwrap();
    let envelope_2 =
        DeviceAttestationEnvelope::new_signed(attestation, loro_payload, &device_kp).unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    envelope_1
        .verify(loro_payload, &acceptor, 0)
        .expect("first verify must succeed (consumes the attestation nonce)");
    let err = envelope_2
        .verify(loro_payload, &acceptor, 0)
        .expect_err("replayed attestation nonce MUST reject via Acceptor nonce-store");

    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "replay rejection MUST surface E_DEVICE_ATTESTATION_FORGED; got {err:?}"
    );
}

/// §13.8 version-rejection pin: future-version envelopes (V3+) MUST
/// reject at `from_canonical_bytes` decode time so a newer peer's
/// envelope shape doesn't silently surface as V2 fields with possibly
/// different semantics. Mirrors `atrium_two_device.rs::
/// future_wire_version_rejected_at_decode` at the direct-test level.
#[test]
fn from_canonical_bytes_rejects_future_wire_version() {
    // Build a V2 envelope, decode-encode to mutate the version byte,
    // then re-feed through from_canonical_bytes.
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_attestation(&parent_kp, device_did);
    let mut envelope =
        DeviceAttestationEnvelope::new_signed(attestation, b"payload", &device_kp).unwrap();

    // Mutate the version field in-place + re-encode (re-encoding the
    // struct directly so we don't have to walk DAG-CBOR bytes).
    envelope.version = u8::MAX;
    let future_bytes = envelope.to_canonical_bytes().unwrap();

    let result = DeviceAttestationEnvelope::from_canonical_bytes(&future_bytes);
    let err = result.expect_err("future-version envelope MUST reject at decode");

    let msg = format!("{err}");
    assert!(
        msg.contains("MAX_WIRE_VERSION") || msg.contains("version"),
        "future-version rejection MUST cite version mismatch; got: {msg}"
    );
}

/// §13.8 malformed-bytes pin: garbage input MUST reject at decode time
/// with `AtriumError::InvalidState` (not panic). Defends the
/// `from_canonical_bytes` boundary against truncated / corrupted wire
/// payloads.
#[test]
fn from_canonical_bytes_rejects_malformed_input() {
    let garbage: &[u8] = b"this-is-definitely-not-a-valid-dag-cbor-envelope";
    let result = DeviceAttestationEnvelope::from_canonical_bytes(garbage);
    let err = result.expect_err("garbage bytes MUST reject at decode");
    let msg = format!("{err}");
    assert!(
        msg.contains("decode failed") || msg.contains("invalid"),
        "malformed-bytes rejection MUST cite decode failure; got: {msg}"
    );
}

/// §13.8 wire-version-constant pin: `WIRE_VERSION` + `MAX_WIRE_VERSION`
/// values are part of the public on-wire contract. A change to either
/// is a wire-format break and MUST land alongside a backward-compat
/// shim or a version-rejection rebake.
#[test]
fn wire_version_constants_are_pinned_at_2() {
    assert_eq!(
        DeviceAttestationEnvelope::WIRE_VERSION,
        2,
        "WIRE_VERSION change is a wire-format break — coordinate with \
         Atrium peer-rollout protocol before bumping"
    );
    assert_eq!(
        DeviceAttestationEnvelope::MAX_WIRE_VERSION,
        2,
        "MAX_WIRE_VERSION change is a wire-format break — coordinate \
         with Atrium peer-rollout protocol before bumping"
    );
}
