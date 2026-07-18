//! GAP-KDB Shape-B / Fork-A — VC-verify silent-PQ-strip behavioral pin
//! (D-53). Mirrors the AUTH-3 UCAN flagship for the VC authority-verify
//! site.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A + Worry-#1. Ben's
//! **D-53**: `vc.rs:453` (`vc::verify`) is INCLUDED in the Fork-A
//! authority-path migration — it is the SAME silent-PQ-strip class as the
//! UCAN chain-walk (a hardcoded Ed25519-only `[u8; 64]` proof extraction +
//! `Did::resolve()`). Fork-A migrates `vc::verify` to the ONE
//! codepoint-dispatched hybrid verify (`resolve_signing` +
//! `authority_verify::verify_authority_signature`), so a `did:benten`-issued
//! VC whose `proof` is a bare / PQ-stripped / forged-half signature MUST
//! reject.
//!
//! # would_fail_on_revert
//! A VC issued BY a `did:benten` whose `proof` is (a) a bare 64-byte Ed25519
//! signature, (b) a composite with the ML-DSA half REMOVED (valid Ed25519
//! half), or (c) a composite with a valid Ed25519 half + a zeroed ML-DSA
//! half — each MUST reject with `VcError::BadSignature`. A silent-PQ-strip
//! verifier (checks only the Ed25519 half, the pre-Fork-A `vc::verify`
//! shape) accepts (b) — the `expect_err` flips Err→Ok. The positive control
//! (a FULL valid composite proof from the same `did:benten` issuer VERIFIES)
//! proves the rejects are attributable to the PQ manipulation, not to
//! `did:benten` being unsupported as a VC issuer. The backward-compat
//! control (a classical `did:key` VC verifies unchanged) pins the Ed25519
//! arm is preserved.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::sig::{self, HybridSignature};
use benten_crypto_suite::sizes::ml_dsa_65_sig_len;
use benten_crypto_suite::{SigCodepoint, SignatureSuite, SuiteConfig};
use benten_id::CanonicalBytes;
use benten_id::VcError;
use benten_id::did::Did;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair as Ed25519Keypair;
use benten_id::vc::{self, Credential};

/// A `did:benten` embedding `signer`'s composite signing key (the KEM half
/// is inert for the VC authority path; it just makes the committed CID
/// well-formed). Frozen §1.1 layout via the W0 fixtures.
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

/// A well-formed VC whose `issuer` field is `benten_issuer` and whose
/// `proof` is `proof_wire` (caller-chosen manipulation). The claims are
/// built via the real production `CredentialBuilder` (with a throwaway
/// classical keypair to satisfy `sign`); the classical proof is then
/// OVERWRITTEN with `proof_wire`. `proof_wire` is over
/// `claims.to_canonical_bytes()` — exactly what `vc::verify` re-derives.
fn benten_vc(benten_issuer: &Did, subject: &Did, proof_wire: Vec<u8>) -> Credential {
    let throwaway = Ed25519Keypair::generate();
    let mut vc = Credential::builder()
        .issuer(benten_issuer)
        .subject(subject)
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .sign(&throwaway)
        .expect("VC claims assemble");
    vc.proof = proof_wire;
    vc
}

/// The full valid composite proof over a VC's canonical claim bytes.
fn composite_proof(signer: &sig::Keypair, vc: &Credential) -> HybridSignature {
    SignatureSuite::v1_default().sign(signer, &CanonicalBytes::to_canonical_bytes(&vc.claims))
}

// ── positive control — did:benten composite proof verifies ────────────────

#[test]
fn vc_did_benten_composite_proof_verifies() {
    let signer = kdb::hybrid_keypair();
    let issuer = benten_did_for(&signer, "vc/pos");
    let subject = Ed25519Keypair::generate().public_key().to_did();

    // Assemble with a placeholder proof, then sign the REAL composite over
    // the assembled claims.
    let mut vc = benten_vc(&issuer, &subject, Vec::new());
    vc.proof = composite_proof(&signer, &vc).to_wire_bytes();

    assert!(
        vc::verify(&vc, &issuer).is_ok(),
        "VC/Fork-A: a did:benten issuer's FULL valid LAMPS composite VC proof MUST verify \
         on the codepoint-dispatched authority verify (did:benten is a first-class VC issuer)"
    );
}

// ── flagship — silent-PQ-strip: PQ-half-removed proof rejects ─────────────

#[test]
fn vc_did_benten_pq_stripped_proof_rejects() {
    // THE load-bearing silent-strip case: a real composite whose Ed25519
    // half is VALID over the claim bytes but the ML-DSA half is REMOVED. The
    // pre-Fork-A `vc::verify` (Ed25519-only) accepts it → the expect_err
    // flips Err→Ok on revert.
    let signer = kdb::hybrid_keypair();
    let issuer = benten_did_for(&signer, "vc/strip");
    let subject = Ed25519Keypair::generate().public_key().to_did();

    let mut vc = benten_vc(&issuer, &subject, Vec::new());
    let composite = composite_proof(&signer, &vc);
    let stripped = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(), // VALID Ed25519 half
        Vec::new(),                          // PQ half removed
    );
    vc.proof = stripped.to_wire_bytes();

    assert!(
        matches!(vc::verify(&vc, &issuer), Err(VcError::BadSignature)),
        "VC/Fork-A FLAGSHIP (D-53): a did:benten-issued VC whose composite proof has the \
         ML-DSA half STRIPPED (valid Ed25519 half) MUST reject with BadSignature — accepting \
         it is the silent PQ-strip on the VC authority path"
    );
}

// ── silent-PQ-strip: forged/zeroed ML-DSA half rejects ────────────────────

#[test]
fn vc_did_benten_forged_zeroed_mldsa_half_rejects() {
    // Valid Ed25519 half + a full-length but ZEROED ML-DSA half. Full
    // composite length, but the PQ half does not verify — the ML-DSA half is
    // cryptographically checked, not ignored.
    let signer = kdb::hybrid_keypair();
    let issuer = benten_did_for(&signer, "vc/forged");
    let subject = Ed25519Keypair::generate().public_key().to_did();

    let mut vc = benten_vc(&issuer, &subject, Vec::new());
    let composite = composite_proof(&signer, &vc);
    let forged = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(),
        vec![0u8; ml_dsa_65_sig_len()], // zeroed ML-DSA half (correct length)
    );
    vc.proof = forged.to_wire_bytes();

    assert!(
        matches!(vc::verify(&vc, &issuer), Err(VcError::BadSignature)),
        "VC/Fork-A: a did:benten-issued VC composite proof with a valid Ed25519 half + a \
         zeroed ML-DSA half MUST reject — the ML-DSA half is cryptographically verified"
    );
}

// ── silent-PQ-strip: bare classical-suite proof rejects ───────────────────

#[test]
fn vc_did_benten_bare_ed25519_proof_rejects() {
    // A bare Ed25519 signature (classical-only suite → 64-byte Ed25519 over
    // the raw claim bytes) presented for a did:benten (hybrid-committing)
    // issuer — a composite-committing issuer signed classically is a
    // downgrade → reject.
    let signer = kdb::hybrid_keypair();
    let issuer = benten_did_for(&signer, "vc/bare");
    let subject = Ed25519Keypair::generate().public_key().to_did();

    let mut vc = benten_vc(&issuer, &subject, Vec::new());
    let bare = SignatureSuite::from_config(SuiteConfig::classical_only())
        .sign(&signer, &CanonicalBytes::to_canonical_bytes(&vc.claims));
    assert_eq!(
        bare.to_wire_bytes().len(),
        64,
        "precondition: the classical-only arm is a bare 64-byte Ed25519 signature"
    );
    vc.proof = bare.to_wire_bytes();

    assert!(
        matches!(vc::verify(&vc, &issuer), Err(VcError::BadSignature)),
        "VC/Fork-A: a bare 64-byte Ed25519 proof for a did:benten (hybrid-committing) issuer \
         MUST reject — a composite issuer signed classically is a silent downgrade"
    );
}

// ── backward-compat — classical did:key VC verifies unchanged ─────────────

#[test]
fn vc_did_key_classical_backward_compat_unchanged() {
    // Non-ignored regression guard (REAL now + after migration): a classical
    // did:key issuer's VC still verifies via the 64-byte Ed25519 arm using
    // the REAL production `CredentialBuilder::sign` + `vc::verify`. The
    // Fork-A dispatch (multicodec 0xed01 → Ed25519 arm) MUST leave this
    // unchanged — reverting the classical arm flips Ok→Err.
    let issuer = Ed25519Keypair::generate();
    let issuer_did = issuer.public_key().to_did();
    let subject_did = Ed25519Keypair::generate().public_key().to_did();

    let vc = Credential::builder()
        .issuer(&issuer_did)
        .subject(&subject_did)
        .claim("alumniOf", "ExampleU")
        .issued_at(1_000_000_000)
        .sign(&issuer)
        .expect("classical did:key VC issues");
    assert_eq!(
        vc.proof.len(),
        64,
        "precondition: a did:key VC carries a bare 64-byte Ed25519 proof"
    );
    assert!(
        vc::verify(&vc, &issuer_did).is_ok(),
        "VC/Fork-A: a classical did:key VC MUST still verify via the Ed25519 arm (backward-compat)"
    );
}
