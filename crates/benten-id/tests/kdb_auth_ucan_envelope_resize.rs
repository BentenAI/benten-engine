//! GAP-KDB Shape-B / Fork-A — AUTH-11 `MAX_UCAN_ENVELOPE_BYTES`
//! re-size (TIER-A frozen value) + AUTH-12 did:benten as UCAN audience.
//! W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A "FS-1
//! `MAX_UCAN_ENVELOPE_BYTES` re-sizing for composite chains" +
//! `R2-LANDSCAPE` AUTH-11 (TIER-A: admits 32-link composite, rejects
//! over-cap, value frozen) + AUTH-12 (delegation target).
//!
//! At the freeze base `MAX_UCAN_ENVELOPE_BYTES = 64 * 1024` (`ucan.rs`)
//! is sized for the Ed25519 (64-byte-sig) world. A `did:benten` issuer
//! embeds a LAMPS composite signing key + a ~3373-byte composite
//! signature; a full-depth (`MAX_UCAN_PROOF_DEPTH = 32`) composite chain
//! dwarfs 64 KiB. Fork-A re-sizes the cap so a legitimate 32-link
//! composite chain is admitted, while an over-cap blob still rejects at
//! the byte boundary (F2 / META #629 DoS defense preserved).
//!
//! # would_fail_on_revert
//! - AUTH-11: a 32-link composite chain needs at least
//!   `32 * signature_byte_len_for(HYBRID)` bytes of signature alone
//!   (≈107,936 B, from named upstream constants — never a literal),
//!   which exceeds the 64 KiB base cap. The "admits 32-link" assertion
//!   FAILS at the base cap and PASSES only after the re-size; reverting
//!   the re-size flips it. The over-cap-reject control pins the DoS
//!   ceiling still fires.
//! - AUTH-12: a chain issued BY and bound TO a `did:benten` validates
//!   under the audience-bound hybrid walk; a replay to a different
//!   audience rejects (`AudienceMismatch`). A walk that cannot verify a
//!   did:benten issuer, or that drops did:benten audience binding,
//!   flips these.
//!
//! # R5 un-ignore
//! Re-size `MAX_UCAN_ENVELOPE_BYTES` for composite chains (M-20:
//! throwaway-capture the exact frozen value into
//! `AUTH11_R5_FROZEN_MAX_UCAN_ENVELOPE_BYTES`); migrate the
//! audience-bound walk to hybrid dispatch; repoint the shims; drop
//! `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::sig;
use benten_crypto_suite::{SigCodepoint, SignatureSuite};
use benten_id::CanonicalBytes;
use benten_id::did::Did;
use benten_id::errors::UcanError;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair as Ed25519Keypair;
use benten_id::ucan::{
    Capability, MAX_UCAN_ENVELOPE_BYTES, MAX_UCAN_PROOF_DEPTH, Ucan, UcanClaims,
};

const NOW: u64 = 1_900_000_000;

// ─────────────────────────────────────────────────────────────────────────
// R5 ENTRY SHIM — the Fork-A hybrid audience-bound UCAN walk.
// At R5: body := benten_id::ucan::validate_chain_for_audience(chain, aud).
// ─────────────────────────────────────────────────────────────────────────

fn r5_validate_chain_for_audience(chain: &[Ucan], aud: &Did) -> Result<(), UcanError> {
    // R5: the audience-bound walk is codepoint-dispatched (Fork-A) — a
    // did:benten issuer verifies its composite; the audience binding holds
    // for did:benten audiences.
    benten_id::ucan::validate_chain_for_audience(chain, aud)
}

// ── AUTH-11 — MAX_UCAN_ENVELOPE_BYTES admits a 32-link composite chain ────

#[test]
fn auth11_envelope_cap_admits_32_link_composite_chain() {
    // A 32-link composite chain carries at MINIMUM one composite
    // signature per link — the signature bytes alone are a hard lower
    // bound (iss/aud composite did:benten strings + attenuation add
    // more). Sourced from the named upstream size accessor, never a
    // literal (#5).
    let composite_sig_len =
        SignatureSuite::v1_default().signature_byte_len_for(SigCodepoint::HYBRID_ED25519_MLDSA65);
    let min_32_link_signature_bytes = MAX_UCAN_PROOF_DEPTH * composite_sig_len;

    // Precondition sanity: the composite-sig floor genuinely exceeds the
    // Ed25519-era 64 KiB base cap (else the pin proves nothing).
    assert!(
        min_32_link_signature_bytes > 64 * 1024,
        "precondition: 32 composite signatures ({min_32_link_signature_bytes} B) must exceed \
         the Ed25519-era 64 KiB base cap"
    );

    assert!(
        MAX_UCAN_ENVELOPE_BYTES >= min_32_link_signature_bytes,
        "AUTH-11 (Fork-A): MAX_UCAN_ENVELOPE_BYTES ({MAX_UCAN_ENVELOPE_BYTES} B) MUST be \
         re-sized to admit a full-depth ({MAX_UCAN_PROOF_DEPTH}-link) composite chain — the \
         signature bytes alone require ≥ {min_32_link_signature_bytes} B. The 64 KiB \
         Ed25519-era cap rejects legitimate did:benten chains."
    );
}

/// M-20 frozen-value pin (TIER-A). The re-sized cap is a permanent
/// v1-beta wire budget — freeze the EXACT value captured from the real
/// re-sized const at R5, never a hand-authored literal.
///
/// R5 capture: `MAX_UCAN_ENVELOPE_BYTES = MAX_UCAN_PROOF_DEPTH (32) ×
/// MAX_UCAN_PER_LINK_BYTES (16 × 1024 = 16_384) = 524_288` (512 KiB). The
/// un-ignored assertion below reconciles this frozen value against the real
/// const — a mis-sized const would flip it.
const AUTH11_R5_FROZEN_MAX_UCAN_ENVELOPE_BYTES: usize = 524_288;

#[test]
fn auth11_envelope_cap_frozen_value_m20() {
    assert_eq!(
        MAX_UCAN_ENVELOPE_BYTES, AUTH11_R5_FROZEN_MAX_UCAN_ENVELOPE_BYTES,
        "AUTH-11 TIER-A: the re-sized MAX_UCAN_ENVELOPE_BYTES is a FROZEN v1-beta budget. \
         Fill AUTH11_R5_FROZEN_MAX_UCAN_ENVELOPE_BYTES via M-20 throwaway-capture from the \
         real re-sized const at R5 (never hand-author — a hand golden matching a mis-sized \
         const freezes the mistake)."
    );
}

#[test]
fn auth11_over_cap_envelope_still_rejected() {
    // Non-ignored control (REAL now + after re-size): the byte-boundary
    // DoS ceiling still fires — an envelope one byte over the cap rejects
    // BEFORE serde materializes it (F2 / META #629). This clause of
    // AUTH-11 holds at every cap value; it pins that the re-size widens
    // the budget WITHOUT removing the ceiling.
    let over = vec![0u8; MAX_UCAN_ENVELOPE_BYTES + 1];
    let res = Ucan::from_canonical_bytes_bounded(&over, MAX_UCAN_PROOF_DEPTH);
    assert!(
        matches!(res, Err(UcanError::EnvelopeTooLarge { .. })),
        "AUTH-11: an envelope exceeding MAX_UCAN_ENVELOPE_BYTES MUST reject with \
         EnvelopeTooLarge at the byte boundary (the DoS ceiling survives the re-size); got {res:?}"
    );
}

// ── AUTH-12 — did:benten as UCAN audience (delegation target) ─────────────

/// A `did:benten` whose embedded signing multikey is `signer`'s
/// composite key (frozen §1.1 layout via the W0 fixtures).
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

#[test]
fn auth12_did_benten_audience_binding_validates_and_rejects_replay() {
    // Issuer = did:benten (composite; needs hybrid verify), audience =
    // did:benten (the delegation target). The audience-bound walk MUST
    // (a) validate when bound to the correct did:benten audience and
    // (b) reject a replay to a DIFFERENT audience.
    let issuer_signer = kdb::hybrid_keypair();
    let issuer = benten_did_for(&issuer_signer, "auth12/iss");
    let audience_signer = kdb::hybrid_keypair();
    let audience = benten_did_for(&audience_signer, "auth12/aud");

    let claims = UcanClaims {
        iss: issuer.as_str().to_string(),
        aud: audience.as_str().to_string(),
        att: vec![Capability::new("/zone/posts", "read")],
        nbf: Some(NOW - 1),
        exp: Some(NOW + 3600),
        prf: Vec::new(),
    };
    let sig = SignatureSuite::v1_default().sign(&issuer_signer, &claims.to_canonical_bytes());
    let token = Ucan {
        signature: sig.to_wire_bytes(),
        claims,
    };

    assert!(
        r5_validate_chain_for_audience(std::slice::from_ref(&token), &audience).is_ok(),
        "AUTH-12: a chain issued by a did:benten and bound to a did:benten audience MUST \
         validate under the hybrid audience-bound walk (did:benten is a first-class \
         delegation target)"
    );

    let other_audience = Ed25519Keypair::generate().public_key().to_did();
    let replay = r5_validate_chain_for_audience(std::slice::from_ref(&token), &other_audience);
    assert!(
        matches!(replay, Err(UcanError::AudienceMismatch { .. })),
        "AUTH-12: replaying a did:benten-bound token at a DIFFERENT audience MUST reject with \
         AudienceMismatch (binding holds for did:benten audiences); got {replay:?}"
    );
}
