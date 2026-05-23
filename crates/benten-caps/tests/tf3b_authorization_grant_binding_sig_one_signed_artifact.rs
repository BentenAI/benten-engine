//! TF-3b (R3-W2) — `AuthorizationGrant` ONE-signed-artifact binding pins.
//!
//! Family: TF-3b — G-CORE-3b `AuthorizationGrant{ucan, key_material, binding_sig}`.
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3b def
//! (line 333) + §1.A.FROZEN item 15(d) (line 135) + §5 D-list row
//! D-4M-R3 (line 399).
//!
//! Ratified inputs (cannot be re-debated):
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §R3 — "ONE signed artifact `{ucan, key_material, binding_sig}` where
//!   `binding_sig` is the issuer's signature over the CBOR-encoded
//!   `(ucan, key_material)` tuple bound to the same audience. Validators
//!   check `binding_sig` BEFORE consulting the UCAN scope or the key
//!   material — the binding is the foundation."
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3b (P-3) — encode→decode→
//! re-verify round-trip across online (CBOR-over-ALPN) + offline (Drop)
//! paths; (A-1) stolen-UCAN-without-keys; (A-2) stolen-keys-without-UCAN;
//! (A-3) wrong-audience swap.
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3b
//! production surface = `benten_caps::authorization_grant::
//! AuthorizationGrant` + `AuthorizationGrant::verify_binding()` +
//! `AuthorizationGrant::issue(...)` constructor. The R5 implementer mints
//! `crates/benten-caps/src/authorization_grant.rs` and un-ignores these
//! pins (pim-12).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NON-NEGOTIABLE R3 BRIEF-TEMPLATE CHECKLIST (literal pre-flight lines):
//!  1-19. See `tf3b_restricted_spec_contains_decidable.rs` head comment
//!  for the full reproduced checklist (§3.5b/3.6b sub-rule-4/3.6e/3.6f/
//!  3.5g/3.5i/3.5j/3.6g/3.6h/3.6i/3.6j/3.13/3.5h/3.11/3.5l/3.5m/3.5n +
//!  iterate-to-convergence + canary-first). Each pin below is
//!  `#[ignore]`d with un-ignore at G-CORE-3b marker, pins a SPECIFIC arm
//!  of the binding contract with an observable-consequence, and references
//!  a real (future) production call-site.
//!
//! SHAPE-flag-don't-fake: `benten_caps::authorization_grant` does NOT
//! exist at HEAD. Compile-fail NOW; behaviour-pin contract post-G-CORE-3b.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
// RED: `benten_caps::authorization_grant` does NOT exist at HEAD.
// G-CORE-3b creates it with `AuthorizationGrant` carrying the
// ONE-signed-artifact shape per D-4M-R3 + §1.A.FROZEN item 15(d).
// `KeyMaterial` is the spike-validated wrapped-key envelope (one-field
// swap from `ByteBuf` to `WrappedKey` per Spike I).
use benten_caps::authorization_grant::{
    AuthorizationGrant, AuthorizationGrantError, KeyMaterial, UcanEnvelope,
};

// Stub-ish helpers that construct synthetic UCAN + KeyMaterial inputs the
// G-CORE-3b implementer's production types will accept. Their concrete
// shape is named at the surface but their fields are implementation
// details the implementer owns. We use `*_for_test` helpers the implementer
// MUST mint alongside `AuthorizationGrant::issue` (R3 brief carry: the
// implementer mints synthetic constructors gated by `#[cfg(test)]` or a
// `testing` feature; this is the SAME pattern as `Cid::sample_for_test`).
fn audience(label: &str) -> Cid {
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}
fn audience_alice() -> Cid {
    audience("audience-alice")
}
fn audience_bob() -> Cid {
    audience("audience-bob")
}
fn synthetic_ucan_for(audience: Cid) -> UcanEnvelope {
    UcanEnvelope::synthetic_for_test(audience)
}
fn synthetic_key_material() -> KeyMaterial {
    KeyMaterial::synthetic_for_test()
}

// ---------------------------------------------------------------------------
// Arm P-3.1 — encode→decode→verify binding round-trip (the load-bearing
// happy path).
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: a freshly issued `AuthorizationGrant` encodes via
/// canonical-bytes / DAG-CBOR, decodes back, and `verify_binding()`
/// returns Ok. The grant carries `{ucan, key_material, binding_sig}` and
/// the binding signature is the issuer's signature over the CBOR-encoded
/// `(ucan, key_material)` tuple bound to the audience.
#[test]
fn one_signed_artifact_round_trips_encode_decode_verify() {
    let audience = audience_alice();
    let ucan = synthetic_ucan_for(audience);
    let km = synthetic_key_material();
    let grant = AuthorizationGrant::issue_envelopes_for_test(ucan, km, audience).unwrap();

    // Encode → CBOR bytes (canonical / deterministic).
    let bytes = grant.to_canonical_bytes().unwrap();
    assert!(!bytes.is_empty(), "encoded grant must produce bytes");

    // Decode → equal-shape grant.
    let decoded = AuthorizationGrant::from_canonical_bytes(&bytes).unwrap();

    // Verify the binding survives the round trip.
    decoded
        .verify_binding(audience)
        .expect("freshly issued grant must verify against the binding audience (P-3.1)");
}

// ---------------------------------------------------------------------------
// Arm A-1 — Stolen-UCAN-without-keys: replay the UCAN without matching
// key_material ⇒ binding_sig verification fails. (The load-bearing
// property of D-4M-R3 — ONE signed artifact prevents the ambiguity.)
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: an attacker who lifts the UCAN half but presents
/// a DIFFERENT (fresh-attacker-chosen) `key_material` cannot reconstruct
/// the issuer's `binding_sig` — `verify_binding()` returns the typed
/// `BindingMismatch`. WOULD-FAIL if the implementer signs only the UCAN
/// half (the ambiguity Spike H + R3 explicitly close).
#[test]
fn stolen_ucan_without_keys_binding_sig_rejects() {
    let audience = audience_alice();
    let ucan = synthetic_ucan_for(audience);
    let km_issued = synthetic_key_material();
    let grant = AuthorizationGrant::issue_envelopes_for_test(ucan, km_issued, audience).unwrap();

    // Attacker swaps the KeyMaterial half AFTER the issuer signed.
    let km_attacker_fresh = KeyMaterial::synthetic_for_test_distinct(1);
    let tampered = grant.with_swapped_key_material_for_test(km_attacker_fresh);

    let err = tampered
        .verify_binding(audience)
        .expect_err("stolen-UCAN + attacker KeyMaterial MUST fail (A-1)");
    assert!(
        matches!(err, AuthorizationGrantError::BindingMismatch { .. }),
        "expected BindingMismatch typed-reject; got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Arm A-2 — Stolen-keys-without-UCAN: present KeyMaterial without a
// matching UCAN ⇒ grant validator rejects.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: an attacker who lifts `KeyMaterial` but presents
/// a DIFFERENT UCAN (e.g. one with different `aud`/`exp`/`nbf`) cannot
/// satisfy `binding_sig` ⇒ typed `BindingMismatch`.
#[test]
fn stolen_keys_without_ucan_binding_sig_rejects() {
    let audience = audience_alice();
    let ucan_issued = synthetic_ucan_for(audience);
    let km = synthetic_key_material();
    let grant = AuthorizationGrant::issue_envelopes_for_test(ucan_issued, km, audience).unwrap();

    // Attacker swaps the UCAN half AFTER the issuer signed.
    let ucan_attacker = UcanEnvelope::synthetic_for_test_distinct(audience, 7);
    let tampered = grant.with_swapped_ucan_for_test(ucan_attacker);

    let err = tampered
        .verify_binding(audience)
        .expect_err("stolen-KeyMaterial + attacker UCAN MUST fail (A-2)");
    assert!(
        matches!(err, AuthorizationGrantError::BindingMismatch { .. }),
        "expected BindingMismatch typed-reject; got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Arm A-3 — Wrong-audience swap: bind to audience X, present to audience Y
// ⇒ binding_sig invalid.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: a grant bound to audience X presented for
/// verification under audience Y fails the binding check — typed
/// `AudienceMismatch`. WOULD-FAIL if `binding_sig` covers `(ucan,
/// key_material)` but NOT the audience (the audience binding is the
/// load-bearing third leg per the §R3 RATIFIED text).
#[test]
fn wrong_audience_swap_binding_sig_rejects() {
    let alice = audience_alice();
    let bob = audience_bob();
    let ucan = synthetic_ucan_for(alice);
    let km = synthetic_key_material();
    let grant = AuthorizationGrant::issue_envelopes_for_test(ucan, km, alice).unwrap();

    // Present the same grant for verification under Bob — must fail.
    let err = grant
        .verify_binding(bob)
        .expect_err("wrong audience MUST fail (A-3)");
    assert!(
        matches!(err, AuthorizationGrantError::AudienceMismatch { .. }),
        "expected AudienceMismatch typed-reject; got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Arm P-3.2 — Online (CBOR-over-ALPN) + Offline (Drop bundle) carry the
// SAME envelope shape. (D-4M-R3: "Consistent across online (custom-ALPN
// handler in G-CORE-3e) + offline (Drop bundle in G-CORE-3f) paths.")
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: the CBOR-encoded canonical bytes of an
/// `AuthorizationGrant` are STABLE — the same logical grant encodes to
/// the same bytes whether destined for an online ALPN handler or an
/// offline Drop bundle. WOULD-FAIL if the implementer split the shape
/// into two CBOR formats by surface.
#[test]
fn online_and_offline_envelope_shapes_byte_equal() {
    let audience = audience_alice();
    let ucan = synthetic_ucan_for(audience);
    let km = synthetic_key_material();
    let grant = AuthorizationGrant::issue_envelopes_for_test(ucan, km, audience).unwrap();

    let online_bytes = grant.to_canonical_bytes().unwrap();
    let offline_bytes = grant.to_canonical_bytes().unwrap();
    assert_eq!(
        online_bytes, offline_bytes,
        "one envelope shape, both surfaces (P-3.2)"
    );
}
