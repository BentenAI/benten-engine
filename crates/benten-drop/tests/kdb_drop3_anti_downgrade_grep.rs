//! GAP-KDB Shape-B — DROP-3: anti-downgrade — NO legacy raw-`&RecipientPublic`
//! seal entry (Inv-23 catch-net; retires the two-param seal). W2 benten-drop
//! RED-PHASE, source grep-net.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §5 (the two independently-choosable
//! params collapse into one `RecipientBinding`) + §1 R1-verdict §A ("the
//! `RecipientBinding` typestate doesn't just add a check, it DELETES the
//! vulnerable API") + `R2-LANDSCAPE` DROP-3.
//!
//! # What this pins (grep-absence net, `ct_signature_eq` idiom)
//! The GAP-KDB closure is by CONSTRUCTION: the vulnerable seal signature
//! `seal_sealed_sender(recipient_pub: &RecipientPublic, audience_did:
//! &AudienceDid, …)` — two independently-chosen params with nothing binding the
//! KEM key to the audience DID — must be DELETED, not merely wrapped. If it
//! survives, an attacker downgrades a `did:benten` recipient back to the
//! un-cross-checked path and re-opens GAP-KDB. The net asserts the retirement
//! (absence of the two-param door) AND the replacement (a binding-typed door).
//!
//! # would_fail_on_revert
//! Restoring `seal_sealed_sender(recipient_pub: &RecipientPublic, audience_did:
//! &AudienceDid, …)` (or any public raw-`&RecipientPublic` recipient seal entry)
//! makes the forbidden adjacency reappear → the grep-absence flips.
//!
//! # R5 un-ignore
//! Migrate the public seal API to `&RecipientBinding` / `&[RecipientBinding]`
//! (design §5); drop `#[ignore]`. (Targets `layer_c.rs`.)

#![allow(clippy::unwrap_used, clippy::expect_used)]

/// Whitespace-insensitive source of `layer_c.rs` (robust to rustfmt line
/// breaks) — the grep operates on the squished form.
fn squished_layer_c() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir).join("src/layer_c.rs");
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

/// DROP-3 — the two-independent-param single-recipient seal door is DELETED.
/// The forbidden adjacency `recipient_pub:&RecipientPublic,audience_did:
/// &AudienceDid` (unique to the public `seal_sealed_sender` / `seal_plaintext_
/// sender` two-param door; the internal `seal_inner` pairs `recipient_pub` with
/// `sender_did`, not `audience_did`) must not appear.
#[test]
#[ignore = "RED-PHASE: DROP-3 two-param single-recipient seal door retired — un-ignore at R5"]
fn drop3_two_param_single_seal_door_is_retired() {
    let squished = squished_layer_c();
    assert!(
        !squished.contains("recipient_pub:&RecipientPublic,audience_did:&AudienceDid"),
        "DROP-3: the two-independent-param seal door \
         `seal_*(recipient_pub: &RecipientPublic, audience_did: &AudienceDid, …)` MUST be DELETED \
         (design §5 — the params collapse into one RecipientBinding). would-FAIL-on-revert: \
         restoring it lets an attacker downgrade a did:benten recipient back to the \
         un-cross-checked path (GAP-KDB re-opens)."
    );
}

/// DROP-3 — the replacement binding-typed seal door EXISTS (single +/or group).
/// This is the positive half: the vulnerable API was replaced, not just
/// removed.
#[test]
#[ignore = "RED-PHASE: DROP-3 binding-typed seal door replaces the raw-pubkey entry — un-ignore at R5"]
fn drop3_binding_typed_seal_door_exists() {
    let squished = squished_layer_c();
    // Param-name-agnostic: any `<name>: &RecipientBinding` (single) or
    // `<name>: &[RecipientBinding]` (group) seal parameter.
    assert!(
        squished.contains(":&RecipientBinding") || squished.contains(":&[RecipientBinding]"),
        "DROP-3: the seal API MUST take `&RecipientBinding` / `&[RecipientBinding]` (design §5) — \
         the KEM key arrives PROVEN-committed. would-FAIL-on-revert: absence means the raw \
         `&RecipientPublic` entry was never retired."
    );
}
