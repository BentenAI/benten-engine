//! GAP-KDB Shape-B — **DOCS-7** doc-conformance catch-net (R3 wave
//! **W4-docs-invariants**). Gate **C** (doc-registration).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` R1 §6 **FORK A** + §3.1 worry-1
//! (the entire UCAN authority path — chain-walk, rotation-verify,
//! device-attestation — is hardcoded Ed25519 today; a `did:benten` embeds
//! a LAMPS composite signing key, so making it a UCAN issuer requires ONE
//! codepoint-dispatched hybrid verify; the failure mode to design against
//! is a **silent PQ-strip** on the authority path — verifying only the
//! Ed25519 half of a composite). R2 landscape §1 **DOCS-7**:
//!   "Fork-A authority hybrid-migration **doc-registration**."
//!
//! This is the doc catch-net for FLAGSHIP-2 (AUTH-3 silent-PQ-strip). When
//! the authority-path migration lands (W3 wave), some authority-facing doc
//! MUST register it — that did:benten composite issuers verify on the
//! authority path AND that the silent-PQ-strip is defended (an
//! Ed25519-only-signed did:benten token rejects). At the R3 base NO doc
//! registers it (`did:benten` = 0 hits across docs; no "silent PQ-strip"
//! language).
//!
//! # RED-PHASE discipline
//! BASELINE (NOT ignored) recovers a live authority-facing fact (proving
//! the parser reads the docs). RED arm (`#[ignore = "RED-PHASE: DOCS-7 …
//! un-ignore at R5"]`) asserts the Fork-A registration across the
//! authority-facing doc set (robust to which doc the R5 wave chooses).
//! would-FAIL-on-revert: dropping the migration registration flips the arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn read_doc(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_default()
}

/// The authority-facing doc set — the Fork-A migration may be registered
/// in any of these; searching all is robust to the R5 wave's placement.
fn authority_docs_combined() -> String {
    [
        "docs/THREAT-MODEL.md",
        "docs/SECURITY-POSTURE.md",
        "docs/V1-WIRE-FORMAT-INVENTORY.md",
        "docs/V1-FROZEN-INTERFACE.md",
        "docs/V1-FROZEN-INTERFACE-DEFERRED.md",
        "docs/CRYPTO-CODEPOINTS.md",
    ]
    .iter()
    .map(|p| read_doc(p))
    .collect::<Vec<_>>()
    .join("\n\n")
}

// ===========================================================================
// BASELINE arm (NOT ignored) — drive the REAL authority-doc parser.
// ===========================================================================

/// DOCS-7 BASELINE — the parser recovers a live authority-path fact: the
/// UCAN-Varsig header surface is inventoried in `V1-WIRE-FORMAT-INVENTORY`.
/// Proves the parser reads the docs; an inert read would make the RED
/// registration arm vacuously pass.
#[test]
fn docs_7_authority_path_present_baseline() {
    let combined = authority_docs_combined();
    assert!(
        combined.contains("UCAN-Varsig") || combined.contains("UCAN"),
        "the authority-facing docs MUST inventory the UCAN authority surface \
         at the R3 base (proves the parser reads the docs)."
    );
    // Post-R5 (GAP-KDB doc-wave): the Fork-A authority migration landed — the
    // sanity guard flips from "no authority-facing doc registers did:benten at
    // R3 base" to "now registered", a live post-condition co-pinning the R5
    // registration alongside the RED arm below.
    assert!(
        combined.contains("did:benten"),
        "post-R5: an authority-facing doc now registers the did:benten Fork-A \
         hybrid-migration (composite issuer verify + silent-PQ-strip defense)."
    );
}

// ===========================================================================
// RED arm (ignored until R5) — the Fork-A migration is registered.
// ===========================================================================

/// DOCS-7 RED — an authority-facing doc registers the Fork-A migration:
/// (1) the authority path (UCAN chain-walk / rotation-verify /
/// device-attestation) verifies a `did:benten` **composite / hybrid**
/// issuer signature via codepoint dispatch; AND (2) the **silent-PQ-strip**
/// is defended — an Ed25519-only-signed did:benten token REJECTS on the
/// authority path.
///
/// would-FAIL-on-revert: dropping the migration registration (either the
/// composite-issuer wiring OR the silent-PQ-strip defense) flips the arm.
#[test]
fn docs_7_registers_fork_a_authority_hybrid_migration() {
    let combined = authority_docs_combined();
    let lc = combined.to_lowercase();

    // (1) did:benten as an authority-path (UCAN issuer / rotation / device)
    //     COMPOSITE / hybrid verify.
    let composite_issuer = combined.contains("did:benten")
        && (lc.contains("composite")
            || lc.contains("hybrid")
            || lc.contains("codepoint-dispatch")
            || lc.contains("codepoint dispatch"))
        && (lc.contains("ucan")
            || lc.contains("issuer")
            || lc.contains("authority")
            || lc.contains("rotation")
            || lc.contains("device"));
    assert!(
        composite_issuer,
        "an authority-facing doc MUST register that the authority path (UCAN \
         walk / rotation / device-attestation) verifies a did:benten \
         COMPOSITE/hybrid issuer via codepoint dispatch (design Fork-A)."
    );

    // (2) the silent-PQ-strip defense — verifying only the Ed25519 half of a
    //     composite MUST be rejected (FLAGSHIP-2 / AUTH-3).
    let pq_strip_defended = lc.contains("pq-strip")
        || lc.contains("pq strip")
        || (lc.contains("ed25519")
            && (lc.contains("only") || lc.contains("half"))
            && (lc.contains("reject") || lc.contains("downgrade") || lc.contains("strip")));
    assert!(
        pq_strip_defended,
        "an authority-facing doc MUST register the silent-PQ-strip defense — \
         a did:benten issuer's Ed25519-only-signed token REJECTS (verifying \
         only the classical half of a composite is a silent PQ downgrade; \
         design Fork-A / worry-1 / AUTH-3)."
    );
}
