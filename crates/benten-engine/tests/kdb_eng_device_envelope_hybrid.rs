//! GAP-KDB Shape-B / Fork-A — ENG-1 engine seam: DeviceAttestation-
//! Envelope hybrid device-verify. W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A (the authority
//! path — incl. device attestation — is hardcoded Ed25519 today).
//! `R2-LANDSCAPE` ENG-1 ("DeviceAttestationEnvelope::verify engine seam
//! — did:benten device composite + strip reject; None-legacy skip
//! preserved").
//!
//! At the freeze base `benten_engine::engine_sync::DeviceAttestation-
//! Envelope::verify` (step 1) resolves `attestation.device_did` via
//! `Did::resolve()` (Ed25519-only) and verifies a `[u8; 64]` envelope
//! signature. Fork-A dispatches on the device DID's multicodec: a
//! `did:benten` device (design §4 — each device is its own did:benten
//! with its own KEM key) signs the envelope with a LAMPS composite,
//! verified via the hybrid path. A composite with the PQ half stripped
//! MUST surface `AtriumError::DeviceAttestationForged`.
//!
//! # would_fail_on_revert
//! - Strip: a did:benten-device envelope whose composite envelope
//!   signature has the ML-DSA half removed (valid Ed25519 half) MUST
//!   reject; a silent-strip verify accepts it → the `matches!(..Forged)`
//!   flips.
//! - None-legacy skip preserved (NON-ignored, REAL now): a
//!   `new_unsigned()` (attestation = None) envelope's `verify` is a
//!   backward-compat no-op returning `Ok(None)`. The migration MUST NOT
//!   break this skip (the receiver falls back to its own device_cid).
//!
//! # R5 un-ignore
//! Migrate `DeviceAttestationEnvelope::{new_signed, verify}` step-1 to
//! codepoint-dispatched hybrid verify (composite device key +
//! `SignatureSuite::verify`); repoint the shims; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_crypto_suite::sig::{self, HybridSignature};
use benten_engine::engine_sync::{AtriumError, AtriumResult, DeviceAttestationEnvelope};
use benten_id::device_attestation::{CapabilityEnvelope, DeviceAttestation};
use benten_id::did::Did;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair as Ed25519Keypair;

const NOW: u64 = 1_900_000_000;
const WINDOW: u64 = 3600;

// ─────────────────────────────────────────────────────────────────────────
// R5 ENTRY SHIMS — Fork-A hybrid device-attestation envelope.
//
// At R5:
//   r5_new_signed_did_benten_device(att, payload, device_kp)
//       := DeviceAttestationEnvelope::new_signed(att, payload, device_kp)
//          once new_signed signs a COMPOSITE envelope signature for a
//          did:benten device keypair.
//   r5_verify_did_benten_device_envelope(env, payload, window, now)
//       := env.verify(payload, window, now)
//          once step-1 codepoint-dispatches the device-DID verify.
// `todo!()` so no red-phase run false-greens against the stub.
// ─────────────────────────────────────────────────────────────────────────

fn r5_new_signed_did_benten_device(
    _attestation: DeviceAttestation,
    _loro_payload: &[u8],
    _device_kp: &sig::Keypair,
) -> DeviceAttestationEnvelope {
    todo!(
        "RED-PHASE (ENG-1): hybrid DeviceAttestationEnvelope::new_signed for a did:benten \
         device lands at R5 (composite envelope signature). un-ignore then."
    )
}

fn r5_verify_did_benten_device_envelope(
    _env: &DeviceAttestationEnvelope,
    _loro_payload: &[u8],
    _freshness_window_secs: u64,
    _now_secs: u64,
) -> AtriumResult<Option<CapabilityEnvelope>> {
    todo!(
        "RED-PHASE (ENG-1): DeviceAttestationEnvelope::verify step-1 codepoint-dispatch for a \
         did:benten device lands at R5. Body := env.verify(payload, window, now). un-ignore then."
    )
}

/// A `did:benten` embedding `signer`'s composite signing key.
fn benten_did_for(signer: &sig::Keypair, tag: &str) -> Did {
    let doc = kdb::KeySetDocument::v1_hybrid(
        kdb::signing_multikey_of(&signer.public()),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub(&format!("{tag}/x")),
            &kdb::det_mlkem768_ek(&format!("{tag}/ek")),
        ),
    );
    kdb::self_committed_did(&doc)
}

// ── ENG-1 — did:benten device envelope PQ-strip rejects ───────────────────

#[test]
#[ignore = "RED-PHASE: ENG-1 DeviceAttestationEnvelope did:benten device PQ-strip rejects — un-ignore at R5"]
fn eng1_did_benten_device_envelope_pq_stripped_rejects() {
    // Parent is a classical did:key (step-2 link stays Ed25519); the
    // DEVICE is a did:benten whose envelope signature is a composite.
    let parent_kp = Ed25519Keypair::generate();
    let device_signer = kdb::hybrid_keypair();
    let device_did = benten_did_for(&device_signer, "eng1/device");
    let attestation =
        DeviceAttestation::issue(&parent_kp, device_did, CapabilityEnvelope::default())
            .expect("attestation issues for a did:benten device");

    let payload = b"eng1-loro-payload-bytes";
    // R5 hybrid signer → composite envelope signature.
    let env = r5_new_signed_did_benten_device(attestation, payload, &device_signer);

    // Strip the ML-DSA half of the composite envelope signature (valid
    // Ed25519 half remains) — the silent-PQ-strip a bolt-on verify accepts.
    let composite = HybridSignature::from_lamps_composite_wire(&env.envelope_signature)
        .expect("the R5 envelope carries a full LAMPS composite signature");
    let stripped = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(),
        Vec::new(),
    );
    let mut forged = env;
    forged.envelope_signature = stripped.to_wire_bytes();

    let res = r5_verify_did_benten_device_envelope(&forged, payload, WINDOW, NOW);
    assert!(
        matches!(res, Err(AtriumError::DeviceAttestationForged { .. })),
        "ENG-1: a did:benten device envelope whose composite signature has the ML-DSA half \
         STRIPPED MUST surface DeviceAttestationForged — the step-1 device verify is the full \
         hybrid, not Ed25519-only; got {res:?}"
    );
}

// ── ENG-1 — None-legacy skip preserved (backward-compat) ──────────────────

#[test]
fn eng1_none_legacy_envelope_skip_preserved() {
    // Non-ignored regression guard (REAL now + after migration): an
    // attestation=None envelope's verify is a permissive backward-compat
    // no-op returning Ok(None) (receiver falls back to its own
    // device_cid). The Fork-A device-DID dispatch MUST preserve this
    // skip — a migration that resolves a None device DID would break it.
    let env = DeviceAttestationEnvelope::new_unsigned();
    let verified = env
        .verify(b"any-payload", WINDOW, NOW)
        .expect("None-legacy envelope verify is a permissive no-op");
    assert!(
        verified.is_none(),
        "ENG-1: a new_unsigned (attestation=None) envelope MUST verify as Ok(None) — the \
         backward-compat legacy skip is preserved across the Fork-A device-DID dispatch"
    );
}
