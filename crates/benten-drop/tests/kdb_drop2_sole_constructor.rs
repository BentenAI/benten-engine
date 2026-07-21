//! GAP-KDB Shape-B — DROP-2: `RecipientBinding` sole-constructor, no-fallback-
//! door (C4) + seal-ineligible rejects. W2 benten-drop RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 (`RecipientBinding`) + C4 (the
//! sole `resolve` constructor; no `From`, no `pub` field, no `_for_test`
//! escape, no `(kem_pub, did)` fallback door at v1-beta) + C5 (PQ floor) + §6
//! (bare `did:key` degenerate → `NoKemCommitment`) + `R2-LANDSCAPE` DROP-2.
//!
//! # What this pins
//! `RecipientBinding::resolve` is the ONLY door, and it is seal-ELIGIBILITY-
//! gating: a bare `did:key` (commits no KEM key → `NoKemCommitment`) and a
//! below-PQ-floor `0x6400` key-set (classical-only X25519, HNDL-exposed) both
//! fail closed — neither can be turned into a sealable recipient. A source
//! grep-net pins that the real type ships with NO fallback constructor (the C4
//! anti-downgrade property; a fallback door re-opens GAP-KDB).
//!
//! # would_fail_on_revert
//! - Drop the `NoKemCommitment` gate ⇒ a bare `did:key` yields a binding.
//! - Drop the C5 PQ floor ⇒ a `0x6400` classical-only recipient is sealable.
//! - Add ANY fallback constructor (`From`, `pub` field, `_for_test`, raw
//!   `(kem_pub, did)`) ⇒ the grep-net flips (GAP-KDB re-opens).
//!
//! # R5 un-ignore
//! Mint benten-drop's real `RecipientBinding` in `layer_c.rs` with `resolve`
//! as the sole constructor + no fallback door; repoint the stub; drop
//! `#[ignore]`. (The grep-net targets `layer_c.rs` where the real type lands.)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::layer_c::RecipientBinding;
use benten_id::kdb_testing::{self as kdb, KeySetDocument};

/// DROP-2 — a bare `did:key` commits NO KEM key, so `resolve` fails closed
/// (`NoKemCommitment`-class): a signing-only principal is not a sealable Drop
/// recipient (design §6).
#[test]
fn drop2_bare_did_key_is_seal_ineligible() {
    // A bare hybrid `did:key` — no committed key-set CID at all.
    let did_key = benten_id::did::Did::from_hybrid_public_key(&kdb::hybrid_keypair().public());
    // Even handed a well-formed key-set doc, a `did:key` commits nothing to
    // check it against → resolve MUST reject (no KEM commitment exists).
    let doc = KeySetDocument::v1_hybrid(
        kdb::signing_multikey_of(&kdb::hybrid_keypair().public()),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("drop2/bare/x"),
            &kdb::det_mlkem768_ek("drop2/bare/ek"),
        ),
    );
    assert!(
        RecipientBinding::resolve(&did_key, &doc).is_err(),
        "DROP-2: a bare did:key commits no KEM key → RecipientBinding::resolve MUST fail closed \
         (NoKemCommitment). would-FAIL-on-revert: a fallback that pairs the doc's KEM key with a \
         KEM-less DID re-opens GAP-KDB."
    );
}

/// DROP-2 — a below-PQ-floor `0x6400` (classical-only X25519) key-set is
/// HNDL-exposed and MUST be rejected by `resolve`, even when its CID + embedded
/// signing key are internally consistent (design C5).
#[test]
fn drop2_below_pq_floor_0x6400_is_seal_ineligible() {
    let sig_mk = kdb::signing_multikey_of(&kdb::hybrid_keypair().public());
    // A CLASSICAL-only kem multikey (`0xec ‖ x25519(32)`, NO ML-KEM half) + a
    // kem_cp declaring the below-floor 0x6400 suite.
    let classical_kem = kdb::kem_multikey_classical(&kdb::det_x25519_pub("drop2/floor/x"));
    let doc = KeySetDocument::v1_with(
        kdb::KEYSET_DOC_VERSION,
        sig_mk,
        classical_kem,
        kdb::SIG_CP_LAMPS_MLDSA65_ED25519,
        kdb::KEM_CP_CLASSICAL_X25519_FLOOR, // 0x6400 — below PQ floor
    );
    // A DID that self-commits this doc (CID + embedded-sig checks pass; the
    // reject can ONLY come from the PQ-floor arm).
    let did = kdb::self_committed_did(&doc);
    assert!(
        RecipientBinding::resolve(&did, &doc).is_err(),
        "DROP-2 (C5): a 0x6400 classical-only (below-PQ-floor) key-set MUST fail closed — never \
         silently sealed to. would-FAIL-on-revert: dropping the PQ floor makes an HNDL-exposed \
         recipient sealable."
    );
}

/// DROP-2 — source grep-net (C4 no-fallback-door), SCOPED to the
/// `RecipientBinding` definition. The real type (benten-drop `layer_c.rs`)
/// ships with `resolve` as its SOLE constructor, PRIVATE fields, and NO
/// fallback door — no `From<… for RecipientBinding>`, no `_for_test`/`unchecked`
/// escape hatch. A fallback door is exactly what re-opens the GAP-KDB
/// substitution, so its absence is load-bearing.
///
/// Mirrors the `ct_signature_eq` / `no_hardcoded_sizes` source-scan idiom. The
/// forbidden-pattern checks are SCOPED to the `struct RecipientBinding` body /
/// `RecipientBinding`-targeting impls, so unrelated structs' `pub audience_did`
/// fields (e.g. the token-binding AADs) never false-positive.
#[test]
fn drop2_recipient_binding_has_no_fallback_door() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir).join("src/layer_c.rs");
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    // At R5 the real type + its sole constructor live here.
    let struct_start = source
        .find("struct RecipientBinding")
        .expect("DROP-2: the real RecipientBinding must live in layer_c.rs at R5");
    assert!(
        source.contains("fn resolve"),
        "DROP-2: RecipientBinding::resolve (the sole constructor) must exist"
    );

    // (1) Fields are PRIVATE — scope to the `struct RecipientBinding { … }` body
    // and assert no field line is declared `pub`. (Doc comments start with `///`
    // so a `pub` mention in prose does not false-positive.)
    let after = &source[struct_start..];
    let open = after.find('{').expect("struct has a body");
    let close = after[open..].find('}').expect("struct body closes") + open;
    let body = &after[open + 1..close];
    for line in body.lines() {
        assert!(
            !line.trim_start().starts_with("pub "),
            "DROP-2 (C4): RecipientBinding fields MUST be private (no `pub` field to overwrite a \
             committed KEM key). Offending line: `{}`",
            line.trim()
        );
    }

    // (2) No `_for_test`/`unchecked` escape hatch anywhere (the R3 stub's
    // `for_test_unchecked` must NOT ship — these names are specific enough to
    // scope by themselves).
    for forbidden in [
        "for_test_unchecked",
        "fn unchecked_binding",
        "fn from_kem_pub",
    ] {
        assert!(
            !source.contains(forbidden),
            "DROP-2 (C4): RecipientBinding MUST NOT expose the escape hatch `{forbidden}` — the \
             sole constructor is `resolve`. would-FAIL-on-revert: any such door re-opens the \
             GAP-KDB (kem_pub, audience_did) substitution."
        );
    }

    // (3) No `From<…> for RecipientBinding` conversion (a `From` from a raw
    // `(RecipientPublic, Did)` pair would be a fallback door). Scoped per
    // `for RecipientBinding` trait-impl target (windowed scan of the preceding
    // header), so unrelated `impl From<>` blocks (error conversions) never
    // false-positive.
    for (idx, _) in source.match_indices("for RecipientBinding") {
        let header = &source[idx.saturating_sub(40)..idx];
        assert!(
            !header.contains("From"),
            "DROP-2 (C4): RecipientBinding MUST NOT implement `From<…>` (no raw (kem_pub, did) \
             fallback door). would-FAIL-on-revert: a From conversion re-opens GAP-KDB."
        );
    }

    // (4) O-04 name-agnostic PRODUCER net: the sole COMMITTED constructor is
    // `resolve` (`-> Result<Self, RecipientBindingError>`). Any OTHER function
    // producing a bare `RecipientBinding` / `Vec<RecipientBinding>` value must
    // be a `*_for_test` fixture — a renamed NON-test constructor (e.g.
    // `fn make_binding(&RecipientPublic) -> RecipientBinding`) would evade the
    // fixed 3-name blocklist above but not this. The `->RecipientBinding` /
    // `Vec<RecipientBinding>` match excludes the `RecipientBindingError` type
    // (its `Result<Self, …Error>` return never starts a bare-binding return).
    let squished: String = source.chars().filter(|c| !c.is_whitespace()).collect();
    for (idx, _) in squished.match_indices("pubfn") {
        let sig_end = squished[idx..]
            .find('{')
            .map_or(squished.len(), |off| idx + off);
        let sig = &squished[idx..sig_end];
        let produces_bare_binding =
            sig.contains("->RecipientBinding") || sig.contains("Vec<RecipientBinding>");
        if produces_bare_binding {
            let after = &sig["pubfn".len()..];
            let name_end = after.find(['(', '<']).unwrap_or(after.len());
            let name = &after[..name_end];
            assert!(
                name.ends_with("_for_test"),
                "DROP-2 (C4/O-04): only `resolve` (committed) or a `*_for_test` \
                 fixture may produce a `RecipientBinding`; found producer `{name}`. \
                 A renamed non-test constructor re-opens the (kem_pub, audience_did) \
                 substitution door (Inv-23)."
            );
        }
    }
}
