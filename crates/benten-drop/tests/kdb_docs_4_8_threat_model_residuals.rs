//! GAP-KDB Shape-B — **DOCS-4 + DOCS-8** doc-conformance catch-nets
//! (R3 wave **W4-docs-invariants**).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` §0 (recipient-side gap = the
//! address-book-trusting seam) + R1 §4 DEFER list ("THREAT-MODEL addition:
//! the seal-path revocation-reach gap — rotating away from a compromised
//! key does NOT stop inbound Drops from any sender holding the old key-set
//! doc … distinct from Compromise #67") + §8 worry-3 (offline-first-send).
//! R2 landscape §1:
//!   - **DOCS-4** — "`THREAT-MODEL` rung-4 recipient-key **closure** +
//!     **seal-path revocation-reach** residual (distinct from #67)."
//!   - **DOCS-8** — "honest-residual **offline-first-send**
//!     doc-availability disclosure (distinct from #67 AND revocation-reach)."
//!
//! GAP-KDB creates a THREE-way residual distinction the doc must make
//! explicit so none is silently conflated:
//!   1. recipient-key SUBSTITUTION — **CLOSED** by the commitment (Inv-23).
//!   2. seal-path REVOCATION-REACH — an OPEN residual: a sender holding an
//!      OLD key-set doc can still seal inbound Drops after the recipient
//!      rotates away from a compromised key (distinct from #67).
//!   3. offline-first-send AVAILABILITY — an OPEN residual: sending a first
//!      Drop to an uncached/brand-new contact needs their key-set doc
//!      (distinct from #67 AND from revocation-reach).
//!   (#67 itself = first-contact / TOFU DID-authenticity.)
//!
//! # RED-PHASE discipline
//! BASELINE arms (NOT ignored) drive the REAL on-disk `THREAT-MODEL.md`
//! parser and recover today's trust-tier matrix (proving the parser reads
//! the doc). RED arms (`#[ignore = "RED-PHASE: DOCS-… un-ignore at R5"]`)
//! assert the R5 doc-wave additions. At the R3 base the closure note +
//! both residuals are ABSENT → RED fails (ignored). would-FAIL-on-revert:
//! deleting the R5 THREAT-MODEL additions flips each RED arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn threat_model() -> String {
    std::fs::read_to_string(repo_root().join("docs/THREAT-MODEL.md"))
        .expect("THREAT-MODEL.md must be present")
}

/// Combined honest-residual home (THREAT-MODEL + SECURITY-POSTURE) — the
/// offline-first-send disclosure (DOCS-8) may land in either; searching
/// both is robust to the R5 doc-wave's placement choice.
fn threat_and_posture() -> String {
    let mut s = threat_model();
    s.push('\n');
    s.push_str(
        &std::fs::read_to_string(repo_root().join("docs/SECURITY-POSTURE.md"))
            .expect("SECURITY-POSTURE.md must be present"),
    );
    s
}

// ===========================================================================
// BASELINE arm (NOT ignored) — drive the REAL THREAT-MODEL parser.
// ===========================================================================

/// DOCS-4/8 BASELINE — the parser recovers the on-disk THREAT-MODEL
/// trust-tier matrix (Network observer + Untrusted host tiers). Proves the
/// parser reads the doc; an inert read would make the RED arms vacuously
/// pass.
#[test]
fn docs_4_threat_model_trust_tiers_present_baseline() {
    let doc = threat_model();
    assert!(
        doc.contains("Trust-tier") || doc.contains("trust tier") || doc.contains("Trust tier"),
        "THREAT-MODEL.md MUST carry the trust-tier × operation matrix."
    );
    assert!(
        doc.contains("Network observer") && doc.contains("Untrusted host"),
        "THREAT-MODEL.md MUST enumerate the Network-observer + Untrusted-host \
         tiers (the recipient-key threat surface the closure note extends)."
    );
    // Post-R5 (GAP-KDB doc-wave): the closure note landed — the sanity guard
    // flips from "Inv-23 absent at R3 base" to "Inv-23 now cited", a live
    // post-condition co-pinning the R5 closure alongside the RED arms below.
    assert!(
        doc.contains("Inv-23"),
        "post-R5: THREAT-MODEL now cites Inv-23 (the recipient-key closure the \
         R5 doc-wave added). The parser recovers it FROM the on-disk doc."
    );
}

// ===========================================================================
// RED arms (ignored until R5) — the closure note + the two residuals.
// ===========================================================================

/// DOCS-4 RED — recipient-key CLOSURE. THREAT-MODEL records that the
/// recipient KEM-key substitution at the seal boundary is CLOSED by the
/// commitment (the audience DID commits the KEM key; Inv-23 /
/// `RecipientBinding`) — the address-book-trust seam of design §0 is shut
/// on the recipient side. would-FAIL-on-revert: dropping the closure note
/// leaves the rung silently trusting an honest address book.
#[test]
fn docs_4_threat_model_recipient_key_closure() {
    let doc = threat_model();
    assert!(
        doc.contains("Inv-23"),
        "THREAT-MODEL.md MUST cite Inv-23 for the recipient-key closure (the \
         KEM key is committed by its audience DID — no address-book \
         substitution). R5 doc-wave adds it."
    );
    let closes_substitution = doc.lines().any(|l| {
        (l.contains("Inv-23") || l.contains("RecipientBinding") || l.contains("committed"))
            && (l.contains("substitut") || l.contains("KEM") || l.contains("recipient key"))
    });
    assert!(
        closes_substitution,
        "THREAT-MODEL.md MUST record that recipient-key SUBSTITUTION is closed \
         by the DID-commitment (the recipient side becomes symmetric to the \
         self-certifying sender side; design §0)."
    );
}

/// DOCS-4 RED — the seal-path REVOCATION-REACH residual, DISTINCT from
/// Compromise #67. The doc discloses that rotating away from a compromised
/// key does NOT stop inbound Drops from any sender still holding the OLD
/// key-set doc — an open residual that Shape-B does NOT close and that is
/// NOT the TOFU #67 residual.
///
/// would-FAIL-on-revert: dropping the seal-path-revocation-reach note (or
/// conflating it into #67) fails the arm.
#[test]
fn docs_4_seal_path_revocation_reach_residual_distinct_from_67() {
    let doc = threat_model();
    let lc = doc.to_lowercase();
    // The residual is named as a SEAL-PATH revocation-reach gap.
    assert!(
        (lc.contains("revocation") && lc.contains("reach"))
            && (lc.contains("seal") || lc.contains("inbound drop") || lc.contains("old key-set")),
        "THREAT-MODEL.md MUST disclose the SEAL-PATH revocation-reach residual \
         — an OLD-key-set-doc holder can still seal inbound Drops after the \
         recipient rotates away from a compromised key (design R1 §4 DEFER)."
    );
    // It is explicitly DISTINCT from Compromise #67 (not conflated).
    let distinct = doc.lines().any(|l| {
        (l.contains("revocation") || l.contains("reach") || l.contains("inbound"))
            && (l.contains("distinct") || l.contains("#67") || l.contains("not #67"))
    }) || lc.contains("distinct from compromise #67")
        || lc.contains("distinct from #67");
    assert!(
        distinct,
        "the seal-path revocation-reach residual MUST be marked DISTINCT from \
         Compromise #67 (first-contact/TOFU) — the two are different residuals \
         and MUST NOT be conflated (design R1 §4)."
    );
}

/// DOCS-8 RED — the offline-first-send AVAILABILITY residual, DISTINCT
/// from #67 AND from the seal-path revocation-reach. The doc discloses
/// that sending a first Drop to an uncached / brand-new contact needs
/// their key-set doc (carried/cached/fetched); offline + uncached +
/// never-contacted = cannot send — the operational face of #67 but a
/// separately-named ergonomic residual (design §8 worry-3).
///
/// would-FAIL-on-revert: dropping the offline-first-send disclosure (or
/// folding it into #67 / revocation-reach without a distinct name) fails.
#[test]
fn docs_8_offline_first_send_availability_residual_distinct() {
    let doc = threat_and_posture();
    let lc = doc.to_lowercase();
    // The residual is named: first-send to an uncached contact needs the
    // key-set doc; offline + uncached = cannot send.
    let discloses_availability = lc.lines().any(|l| {
        (l.contains("offline")
            || l.contains("uncached")
            || l.contains("first-send")
            || l.contains("first send"))
            && (l.contains("key-set") || l.contains("keyset") || l.contains("key set"))
    });
    assert!(
        discloses_availability,
        "THREAT-MODEL / SECURITY-POSTURE MUST disclose the offline-first-send \
         AVAILABILITY residual — a first Drop to an uncached/brand-new contact \
         needs their key-set doc (offline+uncached = cannot send; design §8 \
         worry-3)."
    );
    // Distinctness: named as its own residual, not merely #67 or the
    // revocation-reach gap.
    let distinct = lc.contains("distinct from")
        && (lc.contains("offline")
            || lc.contains("availability")
            || lc.contains("first-send")
            || lc.contains("first send"));
    assert!(
        distinct,
        "the offline-first-send residual MUST be marked DISTINCT (from both \
         Compromise #67 and the seal-path revocation-reach) so the three \
         residuals are not silently conflated (design §8)."
    );
}
