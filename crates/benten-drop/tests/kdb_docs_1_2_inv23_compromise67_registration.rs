//! GAP-KDB Shape-B — **DOCS-1 + DOCS-2** doc-conformance catch-nets
//! (R3 wave **W4-docs-invariants**, the LAST GAP-KDB wave).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` (ref `3bea1294`) §5 (Inv-23
//! mint) + §8 / §5-residual (Compromise #67) + R1 §4 FREEZE-item 5 & 9;
//! R2 landscape `GAP-KDB-B-R2-LANDSCAPE.md` §1 docs catch-nets:
//!   - **DOCS-1** — "`INVARIANT-COVERAGE.md` REGISTERS Inv-23 AND
//!     `SECURITY-POSTURE.md` REGISTERS Compromise #67 (header/count
//!     coherence; baseline recovers Inv-1..22 / #1..#66)."
//!   - **DOCS-2** — "Compromise #67 honest disclosure + **no-overclaim**
//!     (TOFU / first-contact / bind-once; does **NOT** claim
//!     elimination)."
//!
//! # RED-PHASE discipline (pim-12 §3.6e + R2 note (a))
//!
//! This is a pure doc-coupling family — it reads the on-disk registry
//! docs via `CARGO_MANIFEST_DIR` and asserts registration/disclosure
//! properties (the exact `f_disc_2` / `tf3f` shape). Two arm categories:
//!
//! - **BASELINE arms (NOT `#[ignore]`d)** DRIVE the REAL on-disk doc
//!   parser and recover TODAY's state (Inv-22 is the top invariant;
//!   Compromise #66 is the top compromise). They pass now and
//!   would-FAIL if the parser went inert — proving every RED arm below
//!   reads the doc, NOT a literal. NEVER `assert_eq!(CONST, CONST)`.
//! - **RED arms (`#[ignore = "RED-PHASE: DOCS-… un-ignore at R5"]`)**
//!   assert the R5 doc-wave end-state: Inv-23 registered, Compromise #67
//!   registered with the honest bind-once framing and NO elimination
//!   claim. At the R3 freeze base these are ABSENT, so each RED arm fails
//!   (hence ignored); R5's doc-wave un-ignores them. would-FAIL-on-revert:
//!   deleting the R5 registration/disclosure flips each RED arm back to
//!   failing.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

/// Repo root from `crates/benten-drop/tests/` (the `f_disc_2` shape).
fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn read_doc(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel))
        .unwrap_or_else(|_| panic!("{rel} must be present at the repo docs root"))
}

/// Enumerate the registered `Inv-<N>` numbers FROM the doc text (not a
/// literal). Mirrors the `f_disc_2` registered-invariant parser.
fn registered_invariants(doc: &str) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    for line in doc.lines() {
        let mut search = line;
        while let Some(idx) = search.find("Inv-") {
            let after = &search[idx + "Inv-".len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                out.insert(n);
            }
            let consumed = idx + "Inv-".len() + digits.len().max(1);
            if consumed >= search.len() {
                break;
            }
            search = &search[consumed..];
        }
    }
    out
}

/// Enumerate the registered `Compromise #<N>` numbers FROM the doc text.
fn registered_compromises(doc: &str) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    for line in doc.lines() {
        let mut search = line;
        while let Some(idx) = search.find("Compromise #") {
            let after = &search[idx + "Compromise #".len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                out.insert(n);
            }
            let consumed = idx + "Compromise #".len() + digits.len().max(1);
            if consumed >= search.len() {
                break;
            }
            search = &search[consumed..];
        }
    }
    out
}

// ===========================================================================
// BASELINE arms (NOT ignored) — drive the REAL doc parsers.
// ===========================================================================

/// DOCS-1 BASELINE — the invariant parser enumerates FROM the on-disk
/// `INVARIANT-COVERAGE.md`. At the R3 base Inv-22 is the top invariant and
/// the parser MUST recover it (plus the low Inv-1..14 baseline). An inert
/// parser (returning {}) would make every registration RED arm vacuously
/// pass — this is the would-FAIL-if-no-op'd guard for the parser itself.
#[test]
fn docs_1_invariant_parser_reads_doc_baseline_inv22_top() {
    let invs = registered_invariants(&read_doc("docs/INVARIANT-COVERAGE.md"));
    assert!(
        invs.contains(&22),
        "the invariant parser MUST enumerate FROM the on-disk \
         INVARIANT-COVERAGE.md — at the R3 base Inv-22 is the top registered \
         invariant and MUST be recovered. Got: {invs:?}"
    );
    assert!(
        invs.iter().any(|&n| n <= 14),
        "the registry MUST carry the baseline Inv-1..Inv-14 set — a real \
         registry, not a stub. Got: {invs:?}"
    );
}

/// DOCS-1 BASELINE — the compromise parser enumerates FROM the on-disk
/// `SECURITY-POSTURE.md`. At the R3 base Compromise #66 is the top
/// compromise and MUST be recovered. Guards the #67 RED arm against a
/// vacuous pass on an inert parser.
#[test]
fn docs_1_compromise_parser_reads_doc_baseline_66_top() {
    let comps = registered_compromises(&read_doc("docs/SECURITY-POSTURE.md"));
    assert!(
        comps.contains(&66),
        "the compromise parser MUST enumerate FROM the on-disk \
         SECURITY-POSTURE.md — at the R3 base Compromise #66 is the top and \
         MUST be recovered. Got top: {:?}",
        comps.iter().max()
    );
    assert!(
        !comps.contains(&67),
        "sanity: at the R3 freeze base Compromise #67 is NOT yet registered \
         (the GAP-KDB residual is minted by the R5 doc-wave). If this fires, \
         the base already carries #67 and the RED arm must be re-based."
    );
}

// ===========================================================================
// RED arms (ignored until R5) — assert the GAP-KDB registration end-state.
// ===========================================================================

/// DOCS-1 RED — `INVARIANT-COVERAGE.md` REGISTERS **Inv-23** ("a Layer-C
/// seal's KEM key is committed by its audience DID"; design §5 mint) AND
/// the header invariant-count coheres with the body (highest == 23, no
/// drift). would-FAIL-on-revert: dropping the Inv-23 mint or leaving the
/// header count at 22 flips the arm.
#[test]
#[ignore = "RED-PHASE: DOCS-1 Inv-23 registration lands at R5 doc-wave — un-ignore at R5"]
fn docs_1_registers_inv_23_header_coheres() {
    let doc = read_doc("docs/INVARIANT-COVERAGE.md");
    let invs = registered_invariants(&doc);
    assert!(
        invs.contains(&23),
        "INVARIANT-COVERAGE.md MUST register Inv-23 (the GAP-KDB seal-KEM \
         commitment invariant; design §5). R5 doc-wave mints it."
    );
    let highest = invs.iter().copied().max().unwrap_or(0);
    assert_eq!(
        highest, 23,
        "the highest registered invariant MUST be Inv-23 at the GAP-KDB \
         end-state (no gap, no over-shoot). Got highest = Inv-{highest}."
    );
    // Header ⟷ body count coherence: the header MUST name the end-state
    // count so header and body cannot drift (the classic registry-drift the
    // catch-net guards). Enumerated from the doc, never a literal.
    assert!(
        doc.contains("23 invariant")
            || doc.contains("23 Invariant")
            || doc.contains("Inv-1..Inv-23")
            || doc.contains("Inv-1 .. Inv-23")
            || doc.contains("Inv-1..23"),
        "the INVARIANT-COVERAGE.md header MUST state the end-state count \
         (23 invariants) so header and body don't drift."
    );
}

/// DOCS-1 RED — Inv-23's registration row NAMES its subject (KEM key
/// committed by the audience DID) so a bare `Inv-23` token can't vacuously
/// satisfy the pin. Registry-binding (co-located value ⟷ meaning), the
/// `f_disc_2` PIN-6/7 discipline. would-FAIL-on-revert: registering the
/// number without the binding meaning fails.
#[test]
#[ignore = "RED-PHASE: DOCS-1 Inv-23 subject-binding lands at R5 doc-wave — un-ignore at R5"]
fn docs_1_inv_23_row_binds_kem_commitment_meaning() {
    let doc = read_doc("docs/INVARIANT-COVERAGE.md");
    let bound = doc.lines().any(|l| {
        l.contains("Inv-23")
            && l.contains("KEM")
            && (l.contains("committed") || l.contains("commit"))
            && (l.contains("audience") || l.contains("DID"))
    });
    assert!(
        bound,
        "Inv-23 MUST be registered BOUND to its subject on one row — the \
         Layer-C seal's KEM key is committed by its audience DID (design §5). \
         A bare `Inv-23` mention with no subject binding MUST fail."
    );
}

/// DOCS-2 RED — `SECURITY-POSTURE.md` REGISTERS **Compromise #67** as the
/// top compromise (design §8 / R1 §5). would-FAIL-on-revert: dropping the
/// #67 mint flips the arm.
#[test]
#[ignore = "RED-PHASE: DOCS-2 Compromise #67 registration lands at R5 doc-wave — un-ignore at R5"]
fn docs_2_registers_compromise_67() {
    let comps = registered_compromises(&read_doc("docs/SECURITY-POSTURE.md"));
    assert!(
        comps.contains(&67),
        "SECURITY-POSTURE.md MUST register Compromise #67 (the GAP-KDB \
         first-contact/TOFU residual; design §8). R5 doc-wave mints it."
    );
    assert_eq!(
        comps.iter().copied().max().unwrap_or(0),
        67,
        "Compromise #67 MUST be the top compromise at the GAP-KDB end-state \
         (SECURITY-POSTURE tops at #66 today → #67 next-free)."
    );
}

/// DOCS-2 RED — Compromise #67 is disclosed HONESTLY with the bind-once /
/// TOFU / first-contact framing AND does **NOT** over-claim elimination.
/// The two halves both matter: (1) the correct residual is named
/// (first-contact DID-authenticity is out-of-band / the user's
/// responsibility); (2) the doc does NOT claim GAP-KDB "eliminates" /
/// "closes" first-contact substitution.
///
/// would-FAIL-on-revert (two directions): removing the bind-once/TOFU
/// disclosure fails the honesty half; adding an "eliminates first-contact
/// substitution" over-claim fails the no-overclaim half.
#[test]
#[ignore = "RED-PHASE: DOCS-2 Compromise #67 no-overclaim disclosure lands at R5 doc-wave — un-ignore at R5"]
fn docs_2_compromise_67_honest_no_overclaim() {
    let doc = read_doc("docs/SECURITY-POSTURE.md");

    // Isolate the #67 section body (from its heading to the next
    // Compromise heading) so we assert over the RIGHT compromise, not a
    // stray mention elsewhere in the doc.
    let start = doc
        .find("Compromise #67")
        .expect("Compromise #67 section MUST exist (DOCS-2 registration)");
    let rest = &doc[start..];
    let end = rest[1..]
        .find("### Compromise #")
        .map_or(rest.len(), |i| i + 1);
    let section = &rest[..end];
    let lc = section.to_lowercase();

    // (1) HONEST residual — the bind-once / first-contact / TOFU framing.
    assert!(
        lc.contains("first-contact") || lc.contains("first contact"),
        "Compromise #67 MUST disclose the FIRST-CONTACT DID-authenticity \
         residual (the honest bind-once posture; design §8)."
    );
    assert!(
        lc.contains("tofu") || lc.contains("bind-once") || lc.contains("out-of-band"),
        "Compromise #67 MUST frame the residual as TOFU / bind-once / \
         out-of-band verification (the Signal-safety-number posture; design §8)."
    );

    // (2) NO over-claim — GAP-KDB reduces the trust window (continuous →
    //     bind-once); it does NOT ELIMINATE first-contact substitution. The
    //     doc must NOT claim it does.
    let overclaims = [
        "eliminates first-contact",
        "eliminates the first-contact",
        "closes first-contact",
        "closes the first-contact",
        "prevents first-contact substitution",
        "solves first-contact",
    ];
    for phrase in overclaims {
        assert!(
            !lc.contains(phrase),
            "Compromise #67 MUST NOT over-claim — it does NOT eliminate/close \
             first-contact DID substitution (only reduces the window to \
             bind-once). Found the over-claim phrase: {phrase:?}"
        );
    }
    // Positive no-overclaim signal: the doc names that the residual is NOT
    // eliminated (honest framing), matching design §8's "do NOT pretend to
    // eliminate".
    assert!(
        lc.contains("not eliminated")
            || lc.contains("does not eliminate")
            || lc.contains("cannot authenticate the first")
            || lc.contains("reduces the")
            || lc.contains("bind-once"),
        "Compromise #67 MUST state the residual is NOT eliminated (only \
         reduced to a bind-once trust window) — the honest-disclosure half."
    );
}
