//! GAP-KDB Shape-B / Fork-A — rotation-attestation hybrid verify
//! (families AUTH-8, AUTH-9, AUTH-10). W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §3 (rotation) + §6 FORK-A
//! ("the entire UCAN authority path (chain-walk, rotation-verify,
//! device-attestation) is hardcoded Ed25519 today"). `R2-LANDSCAPE`
//! AUTH-8/9/10.
//!
//! At the freeze base `RotationAttestation::verify_signature_with` +
//! `RotationLog::accept_rotation_event` (`did_rotation.rs`) extract a
//! `[u8; 64]` Ed25519 signature and resolve `previous_did` via
//! `Did::resolve()` (Ed25519-only). Fork-A makes both dispatch on the
//! prev DID's multicodec: a `did:benten` `previous_did` embeds a LAMPS
//! composite key, so the rotation signature is a composite verified via
//! `SignatureSuite::verify`. A composite with the PQ half stripped MUST
//! reject — and the authenticity gate MUST fire BEFORE the HLC /
//! verbatim-replay checks (AUTH-9).
//!
//! # would_fail_on_revert
//! - AUTH-8: a did:benten-`prev` rotation whose composite signature has
//!   the PQ half removed MUST reject; a silent-strip verify accepts it →
//!   Err→Ok flip. Positive control: the full composite verifies.
//! - AUTH-9: `accept_rotation_event` handed a PQ-stripped did:benten
//!   attestation MUST reject at the authenticity gate (BadSignature) and
//!   NOT record the entry — even when the HLC ordering would otherwise
//!   admit it. A verifier that runs HLC/replay before authenticity, or
//!   that silent-strips, records a forged rotation.
//! - AUTH-10: a classical `did:key` rotation still verifies with the
//!   64-byte Ed25519 arm unchanged (backward-compat regression guard).
//!
//! # R5 un-ignore
//! Migrate `verify_signature_with` + `accept_rotation_event` to
//! codepoint-dispatched hybrid verify (resolve_signing composite arm +
//! `SignatureSuite::verify`); repoint the shims; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::sig::{self, HybridSignature};
use benten_crypto_suite::SignatureSuite;
use benten_id::CanonicalBytes;
use benten_id::did::Did;
use benten_id::did_rotation::{RotationAttestation, RotationLog, rotate_keypair};
use benten_id::errors::DidRotationError;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair as Ed25519Keypair;

const HLC_BASE: u64 = 1_900_000_000;

// ─────────────────────────────────────────────────────────────────────────
// R5 ENTRY SHIMS — Fork-A hybrid rotation verify.
//
// At R5:
//   r5_rotation_verify(att, pk)  := att.verify_signature_with(pk)
//       (once verify_signature_with takes a composite `sig::PublicKey`)
//   r5_accept_rotation_event(log, att) := log.accept_rotation_event(att)
//       (once the authenticity gate resolve_signing's the did:benten prev
//        + hybrid-verifies BEFORE the HLC/replay checks)
// `todo!()` so no red-phase run false-greens against the stub.
// ─────────────────────────────────────────────────────────────────────────

fn r5_rotation_verify(
    _att: &RotationAttestation,
    _prev_signing_pk: &sig::PublicKey,
) -> Result<(), DidRotationError> {
    todo!(
        "RED-PHASE (AUTH-8): RotationAttestation hybrid verify lands at R5. \
         Body := att.verify_signature_with(prev_signing_pk) once it takes a composite key. \
         un-ignore then."
    )
}

fn r5_accept_rotation_event(
    _log: &mut RotationLog,
    _att: &RotationAttestation,
) -> Result<(), DidRotationError> {
    todo!(
        "RED-PHASE (AUTH-9): accept_rotation_event hybrid authenticity gate lands at R5. \
         Body := log.accept_rotation_event(att) once the gate resolve_signing's the \
         did:benten prev + hybrid-verifies BEFORE HLC/replay. un-ignore then."
    )
}

// ── Fixtures ──────────────────────────────────────────────────────────────

/// A `did:benten` embedding `signer`'s composite signing key (frozen
/// §1.1 layout; the KEM half is inert for the rotation path).
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

/// A rotation attestation from `prev` → `next` (both did:benten),
/// signed with `sig_wire` (caller-chosen manipulation). `sig_wire` is
/// over `to_canonical_bytes` = `(previous_did, next_did, superseded_at)`.
fn benten_rotation(prev: &Did, next: &Did, superseded_at: u64, sig_wire: Vec<u8>) -> RotationAttestation {
    RotationAttestation {
        previous_did: prev.as_str().to_string(),
        next_did: next.as_str().to_string(),
        superseded_at,
        signature: sig_wire,
    }
}

/// The full valid composite signature over a rotation's canonical bytes.
fn composite_sign_rotation(signer: &sig::Keypair, att: &RotationAttestation) -> HybridSignature {
    SignatureSuite::v1_default().sign(signer, &att.to_canonical_bytes())
}

// ── AUTH-8 — RotationAttestation verify: did:benten prev composite ────────

#[test]
#[ignore = "RED-PHASE: AUTH-8 rotation verify did:benten prev composite (positive) — un-ignore at R5"]
fn auth8_did_benten_prev_composite_rotation_verifies() {
    let prev_signer = kdb::hybrid_keypair();
    let next_signer = kdb::hybrid_keypair();
    let prev = benten_did_for(&prev_signer, "auth8/prev");
    let next = benten_did_for(&next_signer, "auth8/next");

    let mut att = benten_rotation(&prev, &next, HLC_BASE, Vec::new());
    att.signature = composite_sign_rotation(&prev_signer, &att).to_wire_bytes();

    assert!(
        r5_rotation_verify(&att, &prev_signer.public()).is_ok(),
        "AUTH-8: a did:benten-prev rotation with a FULL valid LAMPS composite signature MUST \
         verify against the prev's composite signing key"
    );
}

#[test]
#[ignore = "RED-PHASE: AUTH-8 rotation verify did:benten prev PQ-strip rejects — un-ignore at R5"]
fn auth8_did_benten_prev_pq_stripped_rotation_rejects() {
    let prev_signer = kdb::hybrid_keypair();
    let next_signer = kdb::hybrid_keypair();
    let prev = benten_did_for(&prev_signer, "auth8s/prev");
    let next = benten_did_for(&next_signer, "auth8s/next");

    let mut att = benten_rotation(&prev, &next, HLC_BASE, Vec::new());
    let composite = composite_sign_rotation(&prev_signer, &att);
    // Strip the ML-DSA half: VALID Ed25519 half over the rotation
    // canonical bytes, PQ half removed → silent-strip verify accepts.
    let stripped = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(),
        Vec::new(),
    );
    att.signature = stripped.to_wire_bytes();

    assert!(
        r5_rotation_verify(&att, &prev_signer.public()).is_err(),
        "AUTH-8: a did:benten-prev rotation whose composite signature has the ML-DSA half \
         STRIPPED MUST reject — the rotation verify is the full hybrid, not Ed25519-only"
    );
}

// ── AUTH-9 — accept_rotation_event authenticity gate BEFORE HLC/replay ────

#[test]
#[ignore = "RED-PHASE: AUTH-9 accept_rotation_event hybrid authenticity gate fails-closed before HLC — un-ignore at R5"]
fn auth9_pq_stripped_rotation_rejects_at_authenticity_gate_and_is_not_recorded() {
    let prev_signer = kdb::hybrid_keypair();
    let next_signer = kdb::hybrid_keypair();
    let prev = benten_did_for(&prev_signer, "auth9/prev");
    let next = benten_did_for(&next_signer, "auth9/next");

    let mut att = benten_rotation(&prev, &next, HLC_BASE, Vec::new());
    let composite = composite_sign_rotation(&prev_signer, &att);
    // PQ-stripped: the HLC ordering (a fresh, monotonic superseded_at into
    // an empty log) WOULD admit this; the authenticity gate MUST fire
    // first and reject it fail-closed.
    let stripped = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(),
        Vec::new(),
    );
    att.signature = stripped.to_wire_bytes();

    let mut log = RotationLog::new();
    let res = r5_accept_rotation_event(&mut log, &att);
    assert!(
        matches!(res, Err(DidRotationError::BadSignature)),
        "AUTH-9: accept_rotation_event MUST reject a PQ-stripped did:benten attestation at the \
         authenticity gate (BadSignature), BEFORE the HLC/verbatim-replay checks; got {res:?}"
    );
    assert!(
        log.entries().is_empty(),
        "AUTH-9: a rotation that fails the authenticity gate MUST NOT be recorded in the log \
         (fail-closed — no forged supersession)"
    );
}

// ── AUTH-10 — rotation did:key backward-compat unchanged ──────────────────

#[test]
fn auth10_did_key_rotation_backward_compat_unchanged() {
    // Non-ignored regression guard (REAL now + after migration): a
    // classical did:key rotation still verifies with the 64-byte Ed25519
    // arm and is accepted by the log. The Fork-A dispatch (multicodec
    // 0xed01 → Ed25519) MUST leave this path unchanged. Reverting that
    // breaks the classical arm → this flips Ok→Err.
    let old_kp = Ed25519Keypair::generate();
    let new_kp = Ed25519Keypair::generate();
    let old_did = old_kp.public_key().to_did();

    let att = rotate_keypair(&old_did, &old_kp, &new_kp, HLC_BASE)
        .expect("did:key rotation attestation issues");
    assert_eq!(
        att.signature.len(),
        64,
        "precondition: a did:key rotation carries a bare 64-byte Ed25519 signature"
    );
    // Real classical verify against the old did:key public key.
    assert!(
        att.verify_signature_with(old_kp.public_key()).is_ok(),
        "AUTH-10: a did:key rotation attestation MUST still verify against the OLD key via \
         the classical Ed25519 arm (backward-compat)"
    );

    let mut log = RotationLog::new();
    assert!(
        log.accept_rotation_event(&att).is_ok(),
        "AUTH-10: a valid did:key rotation MUST still be accepted by the rotation log unchanged"
    );
    assert!(
        log.is_superseded(&old_did),
        "AUTH-10: after accepting the rotation the OLD did:key MUST read as superseded"
    );
}
