//! TF-3d pins — G-CORE-3d AAD-binds-plaintext-CID rebinding-attack prevention.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3d (A-2)
//!     "Plaintext-CID rebinding: take ciphertext for plaintext CID_p,
//!     present at plaintext CID_q → AEAD authentication fails (the
//!     rebinding-attack defense documented at §1.A.FROZEN item 15 +
//!     SECURITY-POSTURE.md per the R0.8 retense corrective)."
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §5
//!     "AAD-rebinding / plaintext-CID rebinding. Owner: G-CORE-3d
//!     (per-Node AEAD with AAD-binds-plaintext-CID)."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def: "per-Node AEAD with
//!     AAD-binds-plaintext-CID + envelope-sig over AuthorizationGrant
//!     (per Spike G + Spike H + R3 ratification — the per-Node ciphertext
//!     is cryptographically bound to its plaintext CID; the envelope
//!     signature binds the UCAN to the key material in ONE signed
//!     artifact)."
//!   - Doc-coupling pin: SECURITY-POSTURE.md §"rebinding-attack-prevention"
//!     section asserted to be present (pim-2 §3.6b sub-rule-4 doc-coupling).
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 + Spike G/H.
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3d (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
// RED-PHASE failure point.
use benten_graph::aead_wrap::{AeadError, EncryptedNode, decrypt, encrypt};

fn node_titled(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".to_string(), Value::text(title));
    Node::new(vec!["Doc".to_string()], props)
}

// ---------------------------------------------------------------------------
// PIN 1 — A-2 adversarial: ciphertext-rebinding to a different plaintext-CID
// fails AEAD authentication.
// ---------------------------------------------------------------------------
// Attacker observes a valid ciphertext for plaintext-CID_p. They mount
// the ciphertext at plaintext-CID_q. The AAD binds the plaintext-CID,
// so the AEAD authenticator fails at decrypt time.
//
// Would-FAIL-IF-NO-OP'd: if AAD omits plaintext-CID, an attacker can
// re-target ciphertext silently → relocation attack succeeds.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_aad_binds_plaintext_cid_rebinding_attack_defeated() {
    let node_p = node_titled("Original recipe P");
    let node_q = node_titled("Different recipe Q");
    let plaintext_cid_p = node_p.cid().unwrap();
    let plaintext_cid_q = node_q.cid().unwrap();
    assert_ne!(plaintext_cid_p, plaintext_cid_q);

    let key = node_p.derive_key_for_test();
    let bytes_p = node_p.to_canonical_bytes().unwrap();

    let encrypted: EncryptedNode =
        EncryptedNode::encrypt(&bytes_p, &plaintext_cid_p, &key).expect("encrypt OK");

    // Attacker presents `encrypted` at the wrong plaintext-CID (CID_q).
    // The AEAD authenticator MUST fail. Decryption MUST NOT return
    // plaintext bytes.
    let result = decrypt(&encrypted, &key).map(|_| ()).err();
    // Whole-AEAD decryption uses the embedded plaintext_cid as AAD.
    // To exercise the rebinding attack, we mount the same ciphertext
    // bytes but rewrite the AAD-bound CID to CID_q.
    let attacker_envelope = encrypted.with_aad_plaintext_cid_for_test(plaintext_cid_q.clone());
    let attacker_result = decrypt(&attacker_envelope, &key);
    assert!(
        matches!(attacker_result, Err(AeadError::Authentication { .. })),
        "Rebinding attack (mount ciphertext-for-P at AAD-CID-Q) MUST \
         fail AEAD authentication. NEVER silently return plaintext for \
         the wrong CID. got: {:?}",
        attacker_result
            .as_ref()
            .map(|_| "Ok(_)")
            .unwrap_or("Err(_)")
    );

    // Sanity: the same ciphertext at the correct CID decrypts.
    let ok_result = decrypt(&encrypted, &key).expect("legit decrypt OK");
    assert_eq!(
        ok_result, bytes_p,
        "control: legitimate ciphertext-at-correct-CID round-trips"
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — F-1 truncated ciphertext yields typed AEAD `Tag` mismatch.
// ---------------------------------------------------------------------------
// Fail-closed arm: strip the last 16 bytes (the AEAD tag) from the
// ciphertext. Decryption MUST yield a typed Tag-mismatch error, never
// silently produce partial plaintext.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_truncated_ciphertext_yields_typed_tag_mismatch() {
    let node = node_titled("Recipe truncate test");
    let plaintext_cid = node.cid().unwrap();
    let key = node.derive_key_for_test();
    let bytes = node.to_canonical_bytes().unwrap();
    let encrypted: EncryptedNode =
        EncryptedNode::encrypt(&bytes, &plaintext_cid, &key).expect("encrypt OK");

    let truncated = encrypted.truncate_tag_for_test();
    let result = decrypt(&truncated, &key);
    assert!(
        matches!(
            result,
            Err(AeadError::TagMismatch { .. })
                | Err(AeadError::Authentication { .. })
                | Err(AeadError::CiphertextTooShort { .. })
        ),
        "Truncated ciphertext (last 16 bytes stripped) MUST yield a typed \
         AEAD Tag-mismatch / authentication / too-short error, NEVER \
         partial plaintext. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — SECURITY-POSTURE.md doc-coupling pin
// (pim-2 §3.6b sub-rule-4: doc destination is the SPECIFIC section).
// ---------------------------------------------------------------------------
// Per the R0.8 R1.4-multitenant-r1.4-2 corrective, SECURITY-POSTURE.md
// MUST gain a section covering "rebinding-attack-prevention" via
// per-Node-AEAD-AAD-binds-plaintext-CID. This is a doc-coupling pin:
// the test reads SECURITY-POSTURE.md and asserts the section exists +
// names the load-bearing mechanism.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_security_posture_md_documents_rebinding_attack_prevention() {
    let doc = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SECURITY-POSTURE.md"
    ))
    .expect("SECURITY-POSTURE.md must be present at repo root /docs/");

    // Must mention rebinding-attack-prevention as a named section.
    assert!(
        doc.contains("rebinding-attack-prevention")
            || doc.contains("Rebinding Attack Prevention")
            || doc.contains("AAD-binds-plaintext-CID"),
        "SECURITY-POSTURE.md MUST document the rebinding-attack-prevention \
         pattern (per R0.8 multitenant-r1.4-2 corrective). Search for one \
         of: 'rebinding-attack-prevention', 'Rebinding Attack Prevention', \
         or 'AAD-binds-plaintext-CID'."
    );

    // Must mention the AEAD mechanism (ChaCha20-Poly1305 per #5 refinement).
    assert!(
        doc.contains("ChaCha20-Poly1305") || doc.contains("ChaCha20Poly1305"),
        "SECURITY-POSTURE.md MUST cite the ChaCha20-Poly1305 AEAD as the \
         mechanism (per CLAUDE.md #5 refinement)."
    );
}
