//! GAP-KDB Shape-B / Fork-A — UCAN chain-walk hybrid-verify + the
//! **silent-PQ-strip flagship** (families AUTH-1, AUTH-2, AUTH-3★,
//! AUTH-4, AUTH-5, AUTH-6). W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A + Worry-#1 (the
//! biggest integration risk — verifying only the Ed25519 half of a
//! composite `did:benten` issuer is a **silent PQ-downgrade on the
//! authority path**; the walk MUST become ONE codepoint-dispatched
//! hybrid verify, not a bolt-on). `R2-LANDSCAPE` AUTH-1..6, FLAGSHIP-2.
//!
//! At the freeze base the UCAN chain-walk (`validate_chain_inner`,
//! `ucan.rs`) is hardcoded Ed25519: it extracts a `[u8; 64]` sig,
//! resolves the issuer via `Did::resolve()` (Ed25519-only), and calls
//! `pk.as_verifying_key().verify(..)`. Fork-A makes this dispatch on the
//! **issuer DID's leading multicodec** (`did:key` 0xed01 → Ed25519
//! 64-byte; hybrid `did:key` / `did:benten` 0x1211 → the LAMPS composite
//! via `SignatureSuite::verify`, which is already strip-resistant). The
//! `did:benten` issuer's embedded signing key is the composite, so a
//! token carrying only the classical half (or a zeroed/forged ML-DSA
//! half, or a classical-suite signature) MUST reject.
//!
//! # would_fail_on_revert (per R2 AUTH-3 flagship)
//! A chain whose issuer is a `did:benten` but whose signature is (a) a
//! bare 64-byte Ed25519 sig, (b) a composite with the PQ half removed,
//! (c) a composite with a valid Ed25519 half + zeroed ML-DSA half, or
//! (d) a classical-codepoint (0x0002) signature — each MUST reject. A
//! silent-PQ-strip verifier (checks only the Ed25519 half) accepts (b)
//! and (c) — the `expect_err` flips Err→Ok. AUTH-1 is the paired
//! positive control: a FULL valid composite from the same issuer PASSES,
//! proving the reject is due to the PQ manipulation, not `did:benten`
//! being unsupported.
//!
//! # R5 un-ignore
//! Migrate `validate_chain_inner` to codepoint-dispatched hybrid verify
//! (resolve_signing composite arm + reconstruct the `HybridSignature`
//! from the token wire bytes + `SignatureSuite::verify`); repoint the
//! `r5_validate_chain_at` shim to `benten_id::ucan::validate_chain_at`;
//! drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::sig::{self, HybridSignature};
use benten_crypto_suite::sizes::ml_dsa_65_sig_len;
use benten_crypto_suite::{SigCodepoint, SignatureSuite, SuiteConfig};
use benten_id::CanonicalBytes;
use benten_id::did::Did;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair as Ed25519Keypair;
use benten_id::ucan::{Capability, Ucan, UcanClaims};

// ─────────────────────────────────────────────────────────────────────────
// R5 ENTRY SHIM — the Fork-A hybrid-codepoint-dispatched UCAN chain-walk.
//
// At R5 the body becomes `benten_id::ucan::validate_chain_at(chain, now)`
// once `validate_chain_inner` dispatches on the issuer DID's leading
// multicodec instead of the hardcoded Ed25519 `[u8; 64]` + `pk.verify`.
// `todo!()` so no red-phase run false-greens against the stub — the ONLY
// way each pin goes green is against the real migrated walk.
// ─────────────────────────────────────────────────────────────────────────

fn r5_validate_chain_at(chain: &[Ucan], now: u64) -> Result<(), benten_id::errors::UcanError> {
    // R5: `validate_chain_inner` is now codepoint-dispatched (Fork-A) — the
    // issuer key is resolved via `resolve_signing` and the verify routes
    // through the single `authority_verify` helper.
    benten_id::ucan::validate_chain_at(chain, now)
}

// ─────────────────────────────────────────────────────────────────────────
// Fixtures (self-contained; NOT redefined from the W0 canary fixtures —
// these build the AUTHORITY-side inputs W3 needs: did:benten-issued
// tokens carrying a chosen signature wire).
// ─────────────────────────────────────────────────────────────────────────

const NOW: u64 = 1_900_000_000;

/// A `did:benten` whose embedded signing multikey is exactly `signer`'s
/// composite key (the KEM half is irrelevant to the authority path; it
/// just makes the committed CID well-formed). Built via the frozen §1.1
/// layout (`self_committed_did`) since the production encoder is a W0 stub.
fn benten_did_for(signer: &sig::Keypair) -> Did {
    let sig_mk = kdb::signing_multikey_of(&signer.public());
    let doc = kdb::KeySetDocument::v1_hybrid(
        sig_mk,
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("auth/x"),
            &kdb::det_mlkem768_ek("auth/ek"),
        ),
    );
    kdb::self_committed_did(&doc)
}

/// Leaf-only claims issued by `iss` to `aud`, in-window at [`NOW`].
fn leaf_claims(iss: &Did, aud: &Did) -> UcanClaims {
    UcanClaims {
        iss: iss.as_str().to_string(),
        aud: aud.as_str().to_string(),
        att: vec![Capability::new("/zone/posts", "read")],
        nbf: Some(NOW - 1),
        exp: Some(NOW + 3600),
        prf: Vec::new(),
    }
}

/// The full valid LAMPS-composite signature wire over `claims` by `signer`.
fn composite_over(signer: &sig::Keypair, claims: &UcanClaims) -> HybridSignature {
    SignatureSuite::v1_default().sign(signer, &claims.to_canonical_bytes())
}

// ── AUTH-1 — did:benten composite issuer verifies (positive control) ──────

#[test]
fn auth1_did_benten_composite_issuer_verifies() {
    let signer = kdb::hybrid_keypair();
    let iss = benten_did_for(&signer);
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&iss, &aud);
    let sig = composite_over(&signer, &claims);

    let token = Ucan {
        claims,
        signature: sig.to_wire_bytes(),
    };
    // The FULL composite (both halves valid over the shared M') MUST
    // verify — this is the paired positive control for AUTH-3: it proves
    // a did:benten issuer is a first-class authority principal, so the
    // AUTH-3 rejects are attributable to the PQ manipulation, not to
    // did:benten being unsupported.
    assert!(
        r5_validate_chain_at(&[token], NOW).is_ok(),
        "AUTH-1: a did:benten issuer's FULL valid LAMPS composite UCAN MUST verify \
         on the codepoint-dispatched authority walk"
    );
}

// ── AUTH-2 — backward-compat: did:key 64-byte Ed25519 unchanged ───────────

#[test]
fn auth2_did_key_ed25519_backward_compat_unchanged() {
    // A classical did:key issuer signs a 64-byte Ed25519 token exactly as
    // today. The Fork-A dispatch (multicodec 0xed01 → Ed25519 arm) MUST
    // leave this path byte-for-byte unchanged. Revert that breaks the
    // classical arm → this flips Ok→Err.
    let kp = Ed25519Keypair::generate();
    let iss = kp.public_key().to_did();
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let token = Ucan::builder()
        .issuer_did(&iss)
        .audience_did(&aud)
        .capability("/zone/posts", "read")
        .not_before(NOW - 1)
        .expiry(NOW + 3600)
        .sign(&kp);
    assert_eq!(
        token.signature.len(),
        64,
        "precondition: a did:key token carries a bare 64-byte Ed25519 signature"
    );
    assert!(
        r5_validate_chain_at(&[token], NOW).is_ok(),
        "AUTH-2: a classical did:key 64-byte Ed25519 UCAN MUST still verify \
         unchanged after the Fork-A hybrid dispatch (multicodec 0xed01 arm)"
    );
}

// ── AUTH-3★ — SILENT-PQ-STRIP reject matrix (FLAGSHIP-2) ──────────────────

#[test]
fn auth3a_bare_ed25519_signature_for_benten_issuer_rejects() {
    // (a) A bare Ed25519 signature (classical-only suite → 64-byte
    //     Ed25519 over the RAW claims, the un-migrated did:key shape)
    //     presented for a did:benten issuer that commits the HYBRID
    //     suite. A composite-committing issuer signed with a classical
    //     signature is a downgrade → reject.
    let signer = kdb::hybrid_keypair();
    let iss = benten_did_for(&signer);
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&iss, &aud);

    let bare = SignatureSuite::from_config(SuiteConfig::classical_only())
        .sign(&signer, &claims.to_canonical_bytes());
    assert_eq!(
        bare.to_wire_bytes().len(),
        64,
        "precondition: the classical-only arm is a bare 64-byte Ed25519 signature"
    );
    let token = Ucan {
        claims,
        signature: bare.to_wire_bytes(),
    };
    assert!(
        r5_validate_chain_at(&[token], NOW).is_err(),
        "AUTH-3(a): a bare 64-byte Ed25519 signature for a did:benten (hybrid-committing) \
         issuer MUST reject — a composite issuer signed classically is a silent downgrade"
    );
}

#[test]
fn auth3b_pq_half_stripped_composite_rejects() {
    // (b) THE load-bearing silent-strip case: a real composite whose
    //     Ed25519 half is VALID over M' but the ML-DSA half is removed.
    //     A verifier that checks only the classical half accepts → the
    //     expect_err flips Err→Ok on revert. (Built via the non-gated
    //     `from_parts_internal` + `classical_half_for_test`, pq empty.)
    let signer = kdb::hybrid_keypair();
    let iss = benten_did_for(&signer);
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&iss, &aud);
    let composite = composite_over(&signer, &claims);

    let stripped = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(), // VALID Ed25519 half over M'
        Vec::new(),                          // PQ half removed
    );
    let token = Ucan {
        claims,
        signature: stripped.to_wire_bytes(),
    };
    assert!(
        r5_validate_chain_at(&[token], NOW).is_err(),
        "AUTH-3(b) FLAGSHIP: a composite with a VALID Ed25519 half but the ML-DSA half \
         REMOVED MUST reject — accepting it is the silent PQ-strip on the authority path"
    );
}

#[test]
fn auth3c_forged_zeroed_mldsa_half_rejects() {
    // (c) Valid Ed25519 half + a full-length but ZEROED ML-DSA half.
    //     Full composite length, but the PQ half does not verify. A
    //     silent-strip verifier (ignores the garbage PQ) accepts → flip.
    let signer = kdb::hybrid_keypair();
    let iss = benten_did_for(&signer);
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&iss, &aud);
    let composite = composite_over(&signer, &claims);

    let forged = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(),
        vec![0u8; ml_dsa_65_sig_len()], // zeroed ML-DSA half (correct length)
    );
    assert_eq!(
        forged.to_wire_bytes().len(),
        ml_dsa_65_sig_len() + 64,
        "precondition: the forged composite is full LAMPS length (mldsaSig ‖ tradSig)"
    );
    let token = Ucan {
        claims,
        signature: forged.to_wire_bytes(),
    };
    assert!(
        r5_validate_chain_at(&[token], NOW).is_err(),
        "AUTH-3(c): a composite with a valid Ed25519 half + a zeroed ML-DSA half MUST \
         reject — the ML-DSA half is cryptographically verified, not ignored"
    );
}

#[test]
fn auth3d_classical_codepoint_signature_for_benten_issuer_rejects() {
    // (d) A signature explicitly tagged CLASSICAL_ED25519 (0x0002) — the
    //     classical half over M' — presented for a did:benten issuer that
    //     commits the hybrid suite (0x0001). The dispatch (on the issuer's
    //     composite embedded key) MUST reject a classical-codepoint sig:
    //     `SignatureSuite::verify` already surfaces CodepointMismatch on a
    //     hybrid-key/classical-sig pairing (silent-downgrade defense).
    let signer = kdb::hybrid_keypair();
    let iss = benten_did_for(&signer);
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&iss, &aud);
    let composite = composite_over(&signer, &claims);

    let classical_tagged = HybridSignature::from_parts_internal(
        SigCodepoint::CLASSICAL_ED25519,
        composite.classical_half_for_test(),
        Vec::new(),
    );
    let token = Ucan {
        claims,
        signature: classical_tagged.to_wire_bytes(),
    };
    assert!(
        r5_validate_chain_at(&[token], NOW).is_err(),
        "AUTH-3(d): a CLASSICAL_ED25519 (0x0002) signature for a did:benten issuer that \
         commits the hybrid suite (0x0001) MUST reject on codepoint mismatch — no silent \
         downgrade to the classical arm"
    );
}

// ── AUTH-4 — both halves bind the SAME issuer key (half-splice reject) ────

#[test]
fn auth4_cross_key_half_splice_rejects() {
    // Issuer A's did:benten commits A's composite key. Splice A's valid
    // Ed25519 half (over M') with a DIFFERENT keypair B's ML-DSA half
    // (over the same M'). `SignatureSuite::verify` checks BOTH halves
    // against A's committed composite key → B's ML-DSA half fails against
    // A's ML-DSA verifying key → reject. A verifier that does not bind
    // both halves to the SAME issuer key would accept.
    let signer_a = kdb::hybrid_keypair();
    let signer_b = kdb::hybrid_keypair();
    let iss = benten_did_for(&signer_a);
    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&iss, &aud);

    let comp_a = composite_over(&signer_a, &claims);
    let comp_b = composite_over(&signer_b, &claims);
    let spliced = HybridSignature::from_parts_internal(
        comp_a.codepoint(),
        comp_a.classical_half_for_test(), // A's valid Ed25519 half
        comp_b.pq_half_for_test(),        // B's ML-DSA half — WRONG key
    );
    let token = Ucan {
        claims,
        signature: spliced.to_wire_bytes(),
    };
    assert!(
        r5_validate_chain_at(&[token], NOW).is_err(),
        "AUTH-4: a composite whose Ed25519 half is issuer A's but ML-DSA half is a \
         DIFFERENT key B's MUST reject — both halves bind the SAME issuer key"
    );
}

// ── AUTH-5 — per-link mixed-issuer dispatch (each link own multicodec) ────

#[test]
fn auth5_per_link_mixed_issuer_dispatch_validates() {
    // A 2-link chain: parent issuer = classical did:key (Ed25519), leaf
    // issuer = did:benten (composite). The walk MUST dispatch EACH link
    // on ITS OWN issuer multicodec (parent → 64-byte Ed25519; leaf →
    // composite). A walk that uses one codepoint for the whole chain
    // mis-verifies one link. Positive: correctly-signed mixed chain
    // validates.
    let parent_kp = Ed25519Keypair::generate();
    let parent_did = parent_kp.public_key().to_did();

    let leaf_signer = kdb::hybrid_keypair();
    let leaf_did = benten_did_for(&leaf_signer);

    // Parent delegates TO the leaf: parent.aud == leaf.iss (chain-link
    // integrity), leaf caps ⊆ parent caps (attenuation).
    let parent_claims = UcanClaims {
        iss: parent_did.as_str().to_string(),
        aud: leaf_did.as_str().to_string(),
        att: vec![Capability::new("/zone/posts", "read")],
        nbf: Some(NOW - 1),
        exp: Some(NOW + 3600),
        prf: Vec::new(),
    };
    let parent = Ucan {
        signature: parent_kp
            .sign(&parent_claims.to_canonical_bytes())
            .to_bytes()
            .to_vec(),
        claims: parent_claims,
    };

    let final_aud = Ed25519Keypair::generate().public_key().to_did();
    let leaf_claims_val = UcanClaims {
        iss: leaf_did.as_str().to_string(),
        aud: final_aud.as_str().to_string(),
        att: vec![Capability::new("/zone/posts", "read")],
        nbf: Some(NOW - 1),
        exp: Some(NOW + 3600),
        prf: vec![parent.clone()],
    };
    let leaf_sig =
        SignatureSuite::v1_default().sign(&leaf_signer, &leaf_claims_val.to_canonical_bytes());
    let leaf = Ucan {
        signature: leaf_sig.to_wire_bytes(),
        claims: leaf_claims_val,
    };

    // Leaf-first chain.
    assert!(
        r5_validate_chain_at(&[leaf, parent], NOW).is_ok(),
        "AUTH-5: a chain with a classical did:key parent + a did:benten composite leaf \
         MUST validate — each link is dispatched on its OWN issuer multicodec"
    );
}

// ── AUTH-6 — unknown/reserved sig codepoint typed-reject (no fallback) ────

#[test]
fn auth6_reserved_sig_codepoint_typed_rejects_at_dispatcher() {
    // Non-ignored foundation (REAL now): the codepoint dispatcher
    // typed-rejects a reserved/unimplemented suite codepoint. This is the
    // "no silent fallback" property the authority walk relies on — a
    // reserved codepoint must NOT default to the Ed25519 arm.
    assert!(
        SignatureSuite::resolve_codepoint(SigCodepoint::HYBRID_MLDSA65_SLHDSA).is_err(),
        "AUTH-6: a reserved/unimplemented sig codepoint (0x0003 HYBRID_MLDSA65_SLHDSA) MUST \
         typed-reject at the dispatcher (no silent fallback)"
    );
    assert!(
        SignatureSuite::resolve_codepoint(SigCodepoint::HYBRID_ED25519_MLDSA65).is_ok(),
        "AUTH-6 control: the live hybrid default codepoint resolves"
    );
}

#[test]
fn auth6_unknown_signing_multicodec_issuer_rejects_no_silent_fallback() {
    // A did:benten-shaped issuer whose embedded signing multikey leads
    // with an UNKNOWN/reserved multicodec (not 0xed01 classical, not
    // 0x1211 composite). `resolve_signing` MUST typed-reject rather than
    // silently treat the tail as Ed25519 — so the authority walk rejects
    // the token. No silent fallback to a known arm.
    let signer = kdb::hybrid_keypair();
    let good_doc = kdb::KeySetDocument::v1_hybrid(
        kdb::signing_multikey_of(&signer.public()),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("auth6/x"),
            &kdb::det_mlkem768_ek("auth6/ek"),
        ),
    );
    // Craft a payload whose LEADING signing multicodec is bogus (0x99 0x24
    // is unassigned) while the rest of the frozen §1.1 layout is intact.
    let good_sig_mk = kdb::signing_multikey_of(&signer.public());
    let mut bad_sig_mk = good_sig_mk.clone();
    bad_sig_mk[0] = 0x99; // corrupt the ML-DSA multicodec low byte → unknown
    let payload = kdb::did_benten_payload(&bad_sig_mk, &good_doc.cid());
    let bad_iss = kdb::did_benten_from_payload_for_test(&payload);

    let aud = Ed25519Keypair::generate().public_key().to_did();
    let claims = leaf_claims(&bad_iss, &aud);
    // Signature bytes are irrelevant — the issuer key resolve fails first.
    let token = Ucan {
        signature: composite_over(&signer, &claims).to_wire_bytes(),
        claims,
    };
    assert!(
        r5_validate_chain_at(&[token], NOW).is_err(),
        "AUTH-6: a did:benten issuer whose embedded signing multikey leads with an \
         unknown/reserved multicodec MUST reject on the walk (resolve_signing typed-reject; \
         no silent fallback to the Ed25519 arm)"
    );
}
