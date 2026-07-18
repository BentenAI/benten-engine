//! GAP-KDB Shape-B — **DOCS-3** doc-conformance catch-net (R3 wave
//! **W4-docs-invariants**). Gate **B** (security-critical).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` §0 (the recipient side is NOT
//! self-certifying today; "every proof silently trusts an honest address
//! book") + §2/§5 (`resolve_kem` + `RecipientBinding` + Inv-23 discharge
//! the premise); R2 landscape §1 **DOCS-3**:
//!   "`SECURITY-PROOFS` recipient-key premise **cites the binding**
//!    (Inv-23 / `RecipientBinding`), drops the address-book assumption."
//!
//! The `SECURITY-PROOFS.md` §4.1 "Recipient-key premise" today establishes
//! the CEK is wrapped to a REAL entropy-bearing hybrid key (the R9 GAP-1
//! fix — no longer a `[u8;32]` fingerprint) but STILL leaves the premise
//! that the recipient KEM key was *honestly obtained* an ASSUMPTION
//! (address-book trust; design §0). GAP-KDB discharges that assumption:
//! the audience DID *commits* the KEM key (BLAKE3-256 2nd-preimage), so
//! the proof cites the **binding** (Inv-23 / `RecipientBinding`) instead of
//! silently trusting the address book.
//!
//! # RED-PHASE discipline
//! BASELINE arms (NOT ignored) drive the REAL on-disk `SECURITY-PROOFS.md`
//! parser and recover the current §4.1 "Recipient-key premise" text
//! (proving the parser reads the doc, not a literal). RED arms
//! (`#[ignore = "RED-PHASE: DOCS-3 … un-ignore at R5"]`) assert the R5
//! doc-wave update cites the binding. At the R3 base the binding cite is
//! ABSENT → RED fails (ignored). would-FAIL-on-revert: dropping the
//! Inv-23 / `RecipientBinding` cite from §4.1 flips the RED arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn security_proofs() -> String {
    std::fs::read_to_string(repo_root().join("docs/SECURITY-PROOFS.md"))
        .expect("SECURITY-PROOFS.md must be present")
}

/// Slice the "Recipient-key premise" region: from that heading up to the
/// next `**` bolded lead-in (the next proof-shape sub-claim). Keeps the
/// assertion scoped to the premise, not the whole doc.
fn recipient_key_premise_region(doc: &str) -> &str {
    let start = doc
        .find("Recipient-key premise")
        .expect("SECURITY-PROOFS.md §4.1 MUST carry the Recipient-key premise");
    let rest = &doc[start..];
    // End at the next bold sub-claim heading after a healthy minimum.
    let end = rest
        .match_indices("**")
        .map(|(i, _)| i)
        .find(|&i| i > 200)
        .map_or(rest.len(), |i| i);
    &rest[..end]
}

// ===========================================================================
// BASELINE arm (NOT ignored) — drive the REAL SECURITY-PROOFS parser.
// ===========================================================================

/// DOCS-3 BASELINE — the parser recovers the §4.1 "Recipient-key premise"
/// from the on-disk doc and confirms it is the REAL entropy-bearing-key
/// premise (R9 GAP-1: HPKE-wrapped to a real hybrid recipient key,
/// unrecoverable from the public key). Proves the parser reads the doc;
/// an inert parser would make the RED cite-arm vacuously pass.
#[test]
fn docs_3_recipient_key_premise_present_baseline() {
    let doc = security_proofs();
    let region = recipient_key_premise_region(&doc);
    assert!(
        region.contains("HPKE") && region.contains("recipient"),
        "the §4.1 Recipient-key premise MUST establish the CEK is \
         HPKE-key-wrapped to a real recipient key. Region: {region:?}"
    );
    assert!(
        region.contains("unrecoverable") || region.contains("entropy"),
        "the §4.1 premise MUST establish the recipient SECRET carries \
         genuine entropy (unrecoverable from the public key) — the R9 GAP-1 \
         non-fingerprint property the binding builds on."
    );
}

// ===========================================================================
// RED arms (ignored until R5) — §4.1 cites the binding, drops the
// address-book assumption.
// ===========================================================================

/// DOCS-3 RED — the §4.1 Recipient-key premise CITES the binding: it names
/// **Inv-23** AND **`RecipientBinding`** as what discharges the
/// honestly-obtained-KEM-key assumption. would-FAIL-on-revert: removing
/// either cite from the premise flips the arm.
#[test]
#[ignore = "RED-PHASE: DOCS-3 §4.1 cites Inv-23/RecipientBinding — lands at R5 doc-wave — un-ignore at R5"]
fn docs_3_recipient_key_premise_cites_the_binding() {
    let doc = security_proofs();
    let region = recipient_key_premise_region(&doc);
    assert!(
        region.contains("Inv-23"),
        "the §4.1 Recipient-key premise MUST cite Inv-23 (the KEM key is \
         committed by its audience DID) — the binding that discharges the \
         honestly-obtained assumption (design §5). R5 doc-wave adds the cite."
    );
    assert!(
        region.contains("RecipientBinding"),
        "the §4.1 Recipient-key premise MUST cite `RecipientBinding` (the \
         sole-constructor typestate that PROVES the KEM key is committed) — \
         the proof now rests on the binding, not the address book (design §5)."
    );
}

/// DOCS-3 RED — the premise DROPS the address-book assumption: it states
/// the KEM key is COMMITTED BY the audience DID (a 2nd-preimage), rather
/// than assumed honestly obtained. This is the substance of "drops the
/// address-book assumption" — the assumption becomes a discharged
/// commitment. would-FAIL-on-revert: reverting to the bare
/// "assume-honestly-obtained" premise (no commitment) flips the arm.
#[test]
#[ignore = "RED-PHASE: DOCS-3 §4.1 discharges the address-book assumption — lands at R5 doc-wave — un-ignore at R5"]
fn docs_3_recipient_key_premise_discharges_address_book_assumption() {
    let doc = security_proofs();
    let region = recipient_key_premise_region(&doc);
    let lc = region.to_lowercase();
    assert!(
        lc.contains("committed by")
            || lc.contains("commits the")
            || lc.contains("2nd-preimage")
            || lc.contains("second-preimage"),
        "the §4.1 premise MUST state the recipient KEM key is COMMITTED BY \
         the audience DID (a BLAKE3-256 2nd-preimage) — discharging the \
         honestly-obtained assumption rather than silently trusting an \
         address book (design §0/§5)."
    );
}
