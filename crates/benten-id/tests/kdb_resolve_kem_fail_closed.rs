//! GAP-KDB Shape-B — `resolve_kem` fail-closed matrix + the FLAGSHIP
//! active-substitution regression (families RK-1..RK-7, RK-2 ★). W0-canary
//! RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §2 Tier-2 (resolve_kem 4-step
//! verify) + corrections C2/C3/C5 + §5 (GAP-KDB closure). `R2-LANDSCAPE`
//! FLAGSHIP-1 RK-2 + RK-1/3/4/5/6/7.
//!
//! # THE FLAGSHIP (RK-2 ★ — GAP-KDB active substitution)
//! A substituted recipient KEM key — a key-set doc whose canonical
//! BLAKE3-256 CID `!=` the audience DID's committed CID — MUST fail closed.
//! Removing the CID 2nd-preimage check makes `resolve_kem` return the
//! attacker's KEM key (active substitution succeeds) and the `expect_err`
//! flips to `Ok` — the flagship regression fails on revert.
//!
//! # would_fail_on_revert (per arm)
//! - RK-2: drop step-1 (CID 2nd-preimage) ⇒ attacker doc resolves ⇒ Err→Ok.
//! - RK-3: drop step-2 (doc.sig == embedded) ⇒ spliced-sig doc resolves.
//! - RK-4: drop the kem_cp⟺components cross-check ⇒ algorithm-confusion.
//! - RK-5: drop the PQ floor (step-3) ⇒ a `0x6400` key-set whose components
//!   MATCH that classical codepoint is ACCEPTED outright — step-4's
//!   `RecipientPublic::from_kem_multikey(0x6400, classical_multikey)` decodes
//!   successfully (ground-truthed at the R6 tail close-out pass). Step-3 is
//!   therefore the SOLE gate for this input, and the arm pins its EXACT typed
//!   variant so a revert fails at the RIGHT gate rather than merely
//!   "somewhere".
//!
//! # Fixture note (R6 tail close-out) — RK-4 / RK-5 require a REAL signing key
//!
//! RK-4 and RK-5 previously built their `did:benten` from
//! `kdb::det_signing_multikey(..)`, which fabricates deterministic BYTES that
//! are not a valid LAMPS composite public key. `Did::resolve_kem` step-2
//! (`benten_embedded_signing_multikey`) PARSES that embedded key, so both arms
//! short-circuited with `DidError::InvalidHybridPublicKey` and NEVER reached
//! the gate each one names — RK-4 never exercised the kem_cp⟺components
//! algorithm-confusion cross-check (step-4) and RK-5 never exercised the PQ
//! floor (step-3). Both still "passed" under a bare `is_err()` because the
//! resolver is fail-closed for an unrelated reason. Both now build their DID
//! from a REAL `kdb::signing_multikey_of(&kdb::hybrid_keypair().public())`
//! (the idiom RK-3 already used), so steps 1-2 pass by construction and each
//! arm reaches — and pins the exact typed variant of — the gate it names.
//! - RK-1/6: a no-op resolve_kem returns no key ⇒ the positive `expect` fails.
//!
//! # R5 un-ignore
//! Mint `Did::resolve_kem` (steps 1–4, fail-closed typed-reject); repoint
//! the stub; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::cipher_suite::{ML_KEM_768_EK_LEN, X25519_PUBLIC_LEN};
use benten_id::errors::DidError;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair;

// ── RK-1 — happy path (committed KEM key recovered) ───────────────────────

#[test]
fn rk1_resolve_kem_recovers_committed_kem_key() {
    let (did, doc) = kdb::honest_recipient_scenario();
    let recipient = kdb::resolve_kem(&did, &doc).expect("honest key-set resolves the KEM key");

    // The recovered RecipientPublic is bound to the hybrid suite and its
    // bytes are the committed X25519 ‖ ML-KEM-768 EK (RecipientPublic::
    // to_bytes is x25519-first for 0x647a).
    assert_eq!(
        recipient.codepoint().raw(),
        0x647a,
        "recovered KEM key is the hybrid suite"
    );
    let mut expected = Vec::new();
    expected.extend_from_slice(&kdb::det_x25519_pub("honest/x25519"));
    expected.extend_from_slice(&kdb::det_mlkem768_ek("honest/mlkem"));
    assert_eq!(
        recipient.to_bytes(),
        expected,
        "resolve_kem MUST recover the exact committed KEM key (x25519 ‖ mlkem768_ek)"
    );
    assert_eq!(expected.len(), X25519_PUBLIC_LEN + ML_KEM_768_EK_LEN);
}

// ── RK-2 ★ FLAGSHIP — active substitution fails closed ────────────────────

#[test]
fn rk2_flagship_substituted_kem_key_fails_closed() {
    let (victim_did, honest_doc, attacker_doc) = kdb::substituted_recipient_scenario();

    // Precondition: the attacker doc has a DIFFERENT canonical CID (2nd-
    // preimage resistance means it CANNOT match the victim's committed CID).
    assert_ne!(
        honest_doc.cid().as_bytes(),
        attacker_doc.cid().as_bytes(),
        "precondition: attacker key-set has a distinct CID"
    );

    // THE FLAGSHIP: resolving the victim DID against the ATTACKER doc MUST
    // fail closed — the substituted KEM key is not committed by the DID.
    assert!(
        matches!(
            kdb::resolve_kem(&victim_did, &attacker_doc),
            Err(DidError::KeysetCommitmentMismatch)
        ),
        "RK-2 ★: a KEM key NOT committed by the audience DID MUST fail closed at \
         step-1 with the typed `KeysetCommitmentMismatch` (BLAKE3-256 CID \
         2nd-preimage). If this is Ok, active substitution succeeds — the whole \
         GAP-KDB closure is defeated; if it is a DIFFERENT DidError the reject \
         fired for the wrong reason."
    );

    // Control: the HONEST doc (whose CID the DID commits) resolves — proving
    // the check discriminates, not blanket-rejects.
    assert!(
        kdb::resolve_kem(&victim_did, &honest_doc).is_ok(),
        "RK-2 control: the honestly-committed key-set MUST resolve"
    );
}

// ── RK-3 — doc.sig == embedded-signing cross-check (spliced-sig reject) ────

#[test]
fn rk3_spliced_doc_sig_mismatch_fails_closed() {
    // Attacker pairs the VICTIM's embedded signing key (in the DID string)
    // with a doc whose `sig` field is the ATTACKER's. The DID commits
    // cid(spliced_doc), so step-1 passes — but step-2 (doc.sig == embedded)
    // MUST reject (design §2 step 2).
    let victim_sig_mk = kdb::signing_multikey_of(&kdb::hybrid_keypair().public());
    let attacker_sig_mk = kdb::signing_multikey_of(&kdb::hybrid_keypair().public());
    assert_ne!(
        victim_sig_mk, attacker_sig_mk,
        "precondition: distinct signing keys"
    );

    let spliced_doc = kdb::KeySetDocument::v1_hybrid(
        attacker_sig_mk, // doc carries the ATTACKER signing key
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("rk3/x"),
            &kdb::det_mlkem768_ek("rk3/ek"),
        ),
    );
    // DID embeds the VICTIM signing key but commits cid(spliced_doc).
    let payload = kdb::did_benten_payload(&victim_sig_mk, &spliced_doc.cid());
    let did = kdb::did_benten_from_payload_for_test(&payload);

    assert!(
        matches!(
            kdb::resolve_kem(&did, &spliced_doc),
            Err(DidError::KeysetEmbeddedSigningMismatch)
        ),
        "RK-3: resolve_kem MUST reject at step-2 with the typed \
         `KeysetEmbeddedSigningMismatch` when doc.sig != the DID's embedded \
         signing multikey (defeats an embedded-sig ⊕ attacker-KEM splice). \
         Step-1 passes here by construction, so any OTHER DidError means the \
         reject fired for the wrong reason."
    );
}

// ── RK-4 — kem_cp ⟺ components cross-check (algorithm-confusion reject) ────

#[test]
fn rk4_kem_cp_component_mismatch_fails_closed() {
    // kem_cp declares the HYBRID suite (0x647a) but the kem multikey carries
    // only a classical X25519 component (no ML-KEM). resolve_kem MUST reject
    // (design C2 — cross-check kem_cp ⟺ {0xec,0x120c}; defeats confusion).
    // REAL signing multikey (not `det_signing_multikey`): step-2 PARSES the
    // DID's embedded signing key, so fabricated bytes reject with
    // `InvalidHybridPublicKey` before step-4 is ever reached — see the
    // "Fixture note" in this file's module doc.
    let sig_mk = kdb::signing_multikey_of(&kdb::hybrid_keypair().public());
    let classical_only_kem = kdb::kem_multikey_classical(&kdb::det_x25519_pub("rk4/x"));
    let doc = kdb::KeySetDocument::v1_with(
        1,
        sig_mk,
        classical_only_kem,
        0x0001,
        0x647a, // claims hybrid, components are classical-only
    );
    let did = kdb::self_committed_did(&doc);
    assert!(
        matches!(
            kdb::resolve_kem(&did, &doc),
            Err(DidError::MalformedKemMultikey)
        ),
        "RK-4: kem_cp=0x647a with classical-only components MUST fail closed at \
         step-4 with the typed `MalformedKemMultikey` (algorithm-confusion). \
         Steps 1-3 pass by construction (self-committed DID, hybrid kem_cp), so \
         any OTHER DidError means the reject fired for the wrong reason."
    );
}

// ── RK-5 — PQ floor (0x6400 classical-only reject, C5) ────────────────────

#[test]
fn rk5_below_pq_floor_classical_only_fails_closed() {
    // A key-set committing 0x6400 (classical-only X25519) is a below-PQ-floor,
    // HNDL-exposed recipient. resolve_kem MUST NEVER silently seal to it
    // (design C5 / baked-in #5/#18).
    // REAL signing multikey — same reason as RK-4 (see the module-doc
    // "Fixture note"): with fabricated bytes this arm rejected at step-2 and
    // never reached the PQ floor it exists to pin.
    let sig_mk = kdb::signing_multikey_of(&kdb::hybrid_keypair().public());
    let classical_kem = kdb::kem_multikey_classical(&kdb::det_x25519_pub("rk5/x"));
    let doc = kdb::KeySetDocument::v1_with(1, sig_mk, classical_kem, 0x0001, 0x6400);
    let did = kdb::self_committed_did(&doc);
    assert!(
        matches!(
            kdb::resolve_kem(&did, &doc),
            Err(DidError::KeysetBelowPqFloor { kem_cp: 0x6400 })
        ),
        "RK-5: a 0x6400 (classical-only) key-set is below the PQ floor and MUST \
         fail closed at step-3 with the typed `KeysetBelowPqFloor {{ kem_cp: \
         0x6400 }}`. Step-3 is the SOLE gate for this input — with it reverted, \
         step-4's `from_kem_multikey(0x6400, classical_multikey)` ACCEPTS \
         (ground-truthed R6-tail), so the recipient would be sealed to an \
         HNDL-exposed classical-only key. The EXACT-variant assertion pins that \
         the reject came from the PQ floor and not from an unrelated earlier \
         gate."
    );
}

// ── RK-6 — strict-canonical consumer (no raw-byte compare, C3) ────────────

#[test]
fn rk6_resolve_kem_over_strict_canonical_decoded_doc() {
    // resolve_kem recomputes the commitment CID over a re-canonicalized /
    // strict-canonical decode of the doc (design C3), NOT a raw-byte compare.
    // A doc that survives the strict decode round-trip resolves identically.
    let (did, doc) = kdb::honest_recipient_scenario();
    let redecoded = kdb::KeySetDocument::from_canonical_bytes(&doc.to_canonical_bytes())
        .expect("the honest doc is strict-canonical and MUST decode");
    assert_eq!(
        redecoded.cid().as_bytes(),
        doc.cid().as_bytes(),
        "canonical decode preserves the CID"
    );
    assert!(
        kdb::resolve_kem(&did, &redecoded).is_ok(),
        "RK-6: resolve_kem MUST accept the strict-canonical-decoded form of a committed doc"
    );
}

// ── RK-7 — bare did:key → NoKemCommitment degenerate ──────────────────────

#[test]
fn rk7_bare_did_key_has_no_kem_commitment() {
    // A bare did:key is the signing-only degenerate identity (design §6): it
    // commits NO key-set → resolve_kem MUST reject (NoKemCommitment) and
    // there is no committed keyset CID to recover.
    let kp = Keypair::generate();
    let bare = kp.public_key().to_did();
    let (_did, any_doc) = kdb::honest_recipient_scenario();
    assert!(
        matches!(
            kdb::resolve_kem(&bare, &any_doc),
            Err(DidError::NoKemCommitment)
        ),
        "RK-7: a bare did:key commits no KEM key → resolve_kem MUST fail closed \
         with the typed `NoKemCommitment`"
    );
    assert!(
        matches!(
            kdb::committed_keyset_cid(&bare),
            Err(DidError::NoKemCommitment)
        ),
        "RK-7: a bare did:key has no committed key-set CID to recover \
         (typed `NoKemCommitment`)"
    );
}
