//! R19 secret-hygiene pin: `GroupAadInputs.k_set` (the group key `K_Set`
//! copy) must NOT render into `Debug` output and is zeroized on drop.
//!
//! Mirrors the crypto-suite `RecipientSecret` / `f_va_4_*` redacted-Debug
//! convention — the assertion is on the actual rendered string with a
//! distinct-byte `k_set` fixture so a leaking Debug is unambiguous.

#![allow(clippy::unwrap_used)]

use benten_membership_set::aad::GroupAadInputs;
use benten_membership_set::codepoints::MEMBERSHIP_SET_GROUP_MULTI_STANZA;

#[test]
fn group_aad_inputs_debug_redacts_k_set() {
    // Distinct-byte K_Set so a leaking Debug is unambiguous.
    let mut k_set = [0u8; 32];
    let distinctive: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xD0];
    k_set[..8].copy_from_slice(&distinctive);

    let inputs = GroupAadInputs {
        codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        body_cid: vec![0x01, 0x71, 0x1e, 0x20],
        member_dids: vec!["did:key:zAAA".to_string()],
        k_set,
        stanza_index: 0,
        stanza_count: 1,
        member_key_generation: 1,
        membership_set_id: vec![0xAB, 0xCD],
        membership_set_generation: 1,
        role_assignments_generation: 1,
        sealed_inner: vec![0x99],
        plaintext_sender_did: None,
    };

    let rendered = format!("{inputs:?}");
    // R6-tail F-41: assert the CONTIGUOUS decimal SEQUENCE a leaking (derived)
    // `Debug` would emit for the distinctive prefix — NOT the individual
    // decimals. A bare `222` / `173` could false-positively collide with an
    // unrelated integer field's rendering (`codepoint`, `membership_set_id`,
    // the HLC/generation counters), making the old scan both fragile and
    // imprecise. This matches the `LEAK_DECIMAL` convention in
    // `crates/benten-engine/tests/f_secret_hygiene_roster.rs`.
    // `#[derive(Debug)]` on a `[u8; 32]` renders `[222, 173, 190, 239, 202,
    // 254, 186, 208, 0, 0, ...]`, so this needle fires on the exact regression.
    const LEAK_DECIMAL: &str = "222, 173, 190, 239, 202, 254, 186, 208";
    assert!(
        !rendered.contains(LEAK_DECIMAL),
        "GroupAadInputs Debug MUST redact k_set (the group key) — a coredump / \
         log line MUST NOT contain it; a derived Debug leaks it as \
         `{LEAK_DECIMAL}`; rendered=`{rendered}`"
    );
    // Positive guard: the marker must replace the `k_set` FIELD wholesale, not
    // merely appear somewhere in the render.
    assert!(
        rendered.contains("k_set: \"<redacted>\""),
        "GroupAadInputs Debug MUST replace the k_set field wholesale with the \
         redaction marker; rendered=`{rendered}`"
    );
    // Non-secret fields still render (Debug is a redaction, not a blackout).
    assert!(
        rendered.contains("codepoint"),
        "non-secret fields must still render for diagnostics; rendered=`{rendered}`"
    );
}
