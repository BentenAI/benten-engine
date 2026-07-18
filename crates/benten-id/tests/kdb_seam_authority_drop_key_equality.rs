//! GAP-KDB Shape-B / Fork-A — SEAM-1 cross-seam key-equality
//! (authority-signing-key == Drop-doc-committed-signing-key). W3
//! RED-PHASE gap-fill.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §2 Tier-2 step-2 ("`doc.sig
//! == self.embedded_signing_multikey()` — mutual consistency … stops a
//! splice that pairs a victim's embedded signing key with an attacker's
//! KEM key") + FORK-A. `R2-LANDSCAPE` SEAM-1 (the cross-seam gap-fill;
//! "needs BOTH resolve_signing and resolve_kem from W0").
//!
//! The two DID seams must agree on ONE principal:
//! - **Authority seam** (`resolve_signing`, Tier-1): recovers the
//!   signing key a UCAN/rotation/device-attestation authenticates as.
//! - **Confidentiality seam** (`resolve_kem`, Tier-2): recovers the KEM
//!   key a Drop is sealed TO — but ONLY after cross-checking the
//!   received key-set doc's `sig` field EQUALS the DID's embedded
//!   signing multikey.
//!
//! SEAM-1 pins that these cannot be made to disagree: you cannot
//! authenticate as principal A (`resolve_signing` → A) yet receive Drops
//! sealed to a key-set doc that claims a DIFFERENT signing principal B —
//! `resolve_kem` fails closed on the `doc.sig != embedded` mismatch
//! (confused-principal reject).
//!
//! # would_fail_on_revert
//! - Positive: for an honest DID, `resolve_signing(did)` recovers the
//!   SAME composite key that the DID's committed key-set doc embeds in
//!   its `sig` field, and `resolve_kem(did, honest_doc)` succeeds — the
//!   two seams agree.
//! - Negative (confused-principal): a doc whose `sig` field is a
//!   DIFFERENT principal B's signing key (while the DID embeds A) MUST
//!   make `resolve_kem` reject — dropping the `doc.sig == embedded`
//!   cross-check lets an attacker seal to a doc that authenticates as
//!   someone else; the `expect_err` flips Err→Ok.
//!
//! # R5 un-ignore
//! Mint `Did::resolve_signing` + `Did::resolve_kem` (with the step-2
//! `doc.sig == embedded_signing` cross-check); repoint the W0 stubs;
//! drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::sig;
use benten_id::kdb_testing as kdb;

/// The hybrid KEM multikey for a scenario tag.
fn kem_for(tag: &str) -> Vec<u8> {
    kdb::kem_multikey_hybrid(
        &kdb::det_x25519_pub(&format!("{tag}/x")),
        &kdb::det_mlkem768_ek(&format!("{tag}/ek")),
    )
}

// ── SEAM-1 positive — both seams resolve the SAME principal ───────────────

#[test]
#[ignore = "RED-PHASE: SEAM-1 authority-key == Drop-doc-committed-key (positive) — un-ignore at R5"]
fn seam1_authority_and_drop_seams_agree_on_one_principal() {
    let signer = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&signer.public());
    let honest_doc = kdb::KeySetDocument::v1_hybrid(sig_mk.clone(), kem_for("seam1/honest"));
    let did = kdb::self_committed_did(&honest_doc);

    // Authority seam: the DID authenticates as `signer`'s composite key.
    let authority_key = kdb::resolve_signing(&did).expect("resolve_signing recovers the DID key");
    assert_eq!(
        authority_key.to_lamps_composite_bytes().unwrap(),
        signer.public().to_lamps_composite_bytes().unwrap(),
        "SEAM-1: resolve_signing MUST recover the DID's embedded composite signing key"
    );

    // The committed doc embeds THAT SAME signing key in its `sig` field
    // (the redundancy design §1.2 relies on).
    assert_eq!(
        honest_doc.sig(),
        sig_mk.as_slice(),
        "SEAM-1: the key-set doc's `sig` field IS the DID's embedded signing multikey"
    );

    // Confidentiality seam: resolving the KEM key against the SAME doc
    // succeeds — both seams agree on one principal.
    assert!(
        kdb::resolve_kem(&did, &honest_doc).is_ok(),
        "SEAM-1: resolve_kem MUST succeed for the DID's own committed key-set doc \
         (authority principal == Drop-recipient principal)"
    );
}

// ── SEAM-1 negative — confused-principal reject ───────────────────────────

#[test]
#[ignore = "RED-PHASE: SEAM-1 confused-principal reject (doc.sig != embedded) — un-ignore at R5"]
fn seam1_confused_principal_doc_sig_mismatch_rejects() {
    // The DID embeds principal A's signing key + commits A's honest doc.
    let signer_a = kdb::hybrid_keypair();
    let sig_mk_a = kdb::signing_multikey_of(&signer_a.public());
    let honest_doc = kdb::KeySetDocument::v1_hybrid(sig_mk_a.clone(), kem_for("seam1/A"));
    let did = kdb::self_committed_did(&honest_doc);

    // Authority seam still authenticates as A.
    let authority_key = kdb::resolve_signing(&did).expect("resolve_signing recovers A");
    assert_eq!(
        authority_key.to_lamps_composite_bytes().unwrap(),
        signer_a.public().to_lamps_composite_bytes().unwrap(),
        "precondition: the DID authenticates as principal A"
    );

    // A CONFUSED doc: its `sig` field claims a DIFFERENT principal B, and
    // it carries B's KEM key. Sealing to this doc would attribute the
    // recipient identity to B while the DID authenticates as A.
    let signer_b = kdb::hybrid_keypair();
    let sig_mk_b = kdb::signing_multikey_of(&signer_b.public());
    let confused_doc = kdb::KeySetDocument::v1_hybrid(sig_mk_b, kem_for("seam1/B"));

    // resolve_kem MUST reject: `confused_doc.sig` (B) != the DID's
    // embedded signing multikey (A) — the step-2 cross-check. (It ALSO
    // fails the CID check, but the cross-seam property SEAM-1 pins is the
    // signing-principal equality: authority == Drop-recipient.)
    assert!(
        kdb::resolve_kem(&did, &confused_doc).is_err(),
        "SEAM-1: resolve_kem MUST reject a key-set doc whose `sig` field is a DIFFERENT \
         signing principal than the DID's authority key — a confused-principal seal where \
         you authenticate as A but receive Drops sealed to B's key-set is unconstructible"
    );
}
