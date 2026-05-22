//! TF-3f pins — G-CORE-3f Drop bundle revocation-reach (R6).
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition.
//!
//! **SPEC-GAP SURFACED:** eventual destination is `benten-drop` (see
//! sibling file `tf3f_drop_bundle_offline_consume.rs` header). Lands in
//! `benten-sync/tests/` as a temporary home; relocate at G-CORE-3f.
//!
//! Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3f (A-2):
//!     "Forever-valid Drop (revocation-reach R6): UCAN inside a
//!     DropBundle revoked AFTER distribution → the Drop bundle still
//!     decrypts (documents the R6 reality; the test asserts
//!     SECURITY-POSTURE.md §'Revocation reach' section is present +
//!     cites this behavior — pim-2 sub-rule-4 doc-coupling)."
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R6
//!     "Revocation reach": "explicit doc in SECURITY-POSTURE.md
//!     (revocation cuts future serves, already-derived keys remain
//!     decryptable; Drop bundles forever-valid once distributed)."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def: "the R6
//!     revocation-reach section (UCAN revocation cuts future serves but
//!     already-derived keys remain decryptable; mitigation = tight
//!     `nbf`/`exp` + key rotation; Drop bundles are forever-valid once
//!     distributed)."
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3f (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_drop::DropBundle;
use benten_id::keypair::Keypair;

// ---------------------------------------------------------------------------
// PIN 1 — A-2: Drop bundle decrypts AFTER UCAN revocation
// (forever-valid-once-distributed).
// ---------------------------------------------------------------------------
// Alice publishes a Drop bundle to Bob; bundle carries Alice's UCAN
// granting Bob scope X. Alice revokes the UCAN. Bob's offline consume
// still decrypts the bundle — the cryptographic property is that the
// key material was already distributed; revocation can only cut FUTURE
// serves. This is the R6 reality documented at SECURITY-POSTURE.md.
//
// Would-FAIL-IF-NO-OP'd: a stub that consults a "revocation set" at
// decrypt time and refuses post-revocation would APPEAR to provide
// stronger semantics — but the property is cryptographically
// unimplementable (the key material is already in Bob's hands). The
// test asserts the correct (R6-honest) behavior.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3f"]
fn tf3f_drop_bundle_decrypts_after_ucan_revocation_forever_valid() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();

    // Alice produces + distributes bundle.
    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&kp_alice, &kp_bob);
    let cbor = bundle.to_cbor_bytes().unwrap();

    // Alice revokes the embedded UCAN (in her own revocation store).
    // Bob's local store may or may not know about the revocation.
    let revocation_record = DropBundle::synthesize_revocation_for_embedded_ucan(&bundle, &kp_alice);
    // (revocation_record stored at Alice's side; Bob doesn't have it.)

    // Bob consumes offline. Bundle MUST still decrypt — the cryptographic
    // property: key material already distributed.
    let recovered_after_revocation = bundle
        .consume_offline(&kp_bob)
        .expect("Drop bundle still decrypts after revocation — R6 reality");
    assert_eq!(
        recovered_after_revocation.len(),
        5,
        "Drop bundle MUST still decrypt all 5 Recipe Nodes AFTER UCAN \
         revocation. This is the R6 forever-valid-once-distributed \
         property (the cryptographic limit). A stub that refuses \
         post-revocation would be 'stronger semantics' but \
         cryptographically unimplementable."
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — Doc-coupling: SECURITY-POSTURE.md documents the R6 reach.
// ---------------------------------------------------------------------------
// Per pim-2 §3.6b sub-rule-4 doc-coupling: the R6 reach MUST be
// explicitly documented at SECURITY-POSTURE.md (named section
// "Revocation reach"), citing:
//   - revocation cuts future serves
//   - already-derived keys remain decryptable
//   - mitigation = tight nbf/exp + key rotation
//   - Drop bundles forever-valid once distributed
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3f"]
fn tf3f_security_posture_md_documents_revocation_reach_section() {
    let doc = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SECURITY-POSTURE.md"
    ))
    .expect("SECURITY-POSTURE.md must be present at repo root /docs/");

    // Named section MUST be present.
    assert!(
        doc.contains("Revocation reach") || doc.contains("revocation reach"),
        "SECURITY-POSTURE.md MUST have a named 'Revocation reach' section \
         (per R6 ratification). Add it at G-CORE-3 doc-coupling step."
    );

    // Section MUST mention the forever-valid-once-distributed property
    // for Drop bundles.
    assert!(
        doc.contains("forever-valid")
            || doc.contains("once distributed")
            || doc.contains("Drop bundles are forever-valid"),
        "SECURITY-POSTURE.md 'Revocation reach' section MUST mention \
         the forever-valid-once-distributed property of Drop bundles \
         (R6 reality; cryptographic limit)."
    );

    // Mitigations named.
    assert!(
        doc.contains("nbf")
            || doc.contains("`exp`")
            || doc.contains("tight expiry")
            || doc.contains("key rotation"),
        "SECURITY-POSTURE.md 'Revocation reach' section MUST name the \
         mitigations: tight nbf/exp + key rotation."
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — Online path DOES enforce revocation (sibling to R6 reality).
// ---------------------------------------------------------------------------
// Companion to PIN 1: the R6 reality is bundle-specific. The ONLINE
// path (G-CORE-3e custom-ALPN handler) DOES enforce revocation —
// revoked grants are refused on future requests. This pin documents
// the asymmetry: offline-Drop is forever-valid; online-pull respects
// revocation. (The online-pull behavior is pinned in
// `tf3e_replay_attack_ucan_expired.rs::tf3e_revoked_grant_yields_typed_revoked`.)
//
// This test asserts the documentation of the asymmetry — that the
// R6 section names BOTH halves.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3f"]
fn tf3f_security_posture_md_names_online_vs_offline_revocation_asymmetry() {
    let doc = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SECURITY-POSTURE.md"
    ))
    .expect("SECURITY-POSTURE.md must be present");

    // The doc MUST name the "future serves cut" vs "already-derived keys
    // remain decryptable" distinction.
    let has_future_serves = doc.contains("future serve") || doc.contains("future serves");
    let has_already_derived = doc.contains("already-derived") || doc.contains("already derived");

    assert!(
        has_future_serves && has_already_derived,
        "SECURITY-POSTURE.md MUST name BOTH halves of the R6 asymmetry: \
         (a) future serves cut (online-pull respects revocation); \
         (b) already-derived keys remain decryptable (offline-Drop \
         forever-valid). Got: future-serves={}, already-derived={}",
        has_future_serves,
        has_already_derived
    );
}
