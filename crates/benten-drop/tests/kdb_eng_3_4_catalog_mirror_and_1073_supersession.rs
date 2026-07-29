//! GAP-KDB Shape-B — **ENG-3 + ENG-4** doc-conformance catch-nets
//! (R3 wave **W4-docs-invariants**).
//!
//! Anchors: design `GAP-KDB-B-DESIGN-R1.md` §2/§5 (`resolve_kem` +
//! `RecipientBinding` are new typed-reject boundaries → new boundary-
//! crossing ErrorCodes) + R1 §4 FREEZE-item 10 ("Retire ratified decision
//! #1073's did:key-only-preservation clause — did:benten now exists in
//! resolve at v1"). R2 landscape §1:
//!   - **ENG-3** — "error-catalog **mirror** for new `resolve_kem` /
//!     `RecipientBinding` boundary-crossing ErrorCodes
//!     (`CATALOG_VARIANT_COUNT`)."
//!   - **ENG-4** — "supersession record — ratified **#1073**
//!     did:key-only-resolve clause **RETIRED**."
//!
//! ENG-3 guards §3.5g (pub-error-variant → `ErrorCode` mirror): every new
//! boundary-crossing error MUST be mirrored in `docs/ERROR-CATALOG.md` AND
//! counted. ENG-4 guards the #1073 supersession record: #1073 preserved
//! "did:key-only in `Did::resolve`" at v1 (DISAGREE, audit-trail); Shape-B
//! ratification SUPERSEDES that clause — the record must say so.
//!
//! # RED-PHASE discipline
//! BASELINE arms (NOT ignored) DRIVE the REAL enum↔catalog mirror (via
//! `benten_errors::ErrorCode::as_str`) and the REAL on-disk docs — proving
//! they read real state, not a literal. RED arms (`#[ignore = "RED-PHASE:
//! ENG-… un-ignore at R5"]`) assert the R5 end-state. would-FAIL-on-revert:
//! reverting the mint/mirror/supersession flips each RED arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_errors::ErrorCode;

/// The R3 freeze-base throwable-ErrorCode count on the canary tree (both
/// the `Throwable enum variants` row AND the `Regression-list entries` row
/// of `docs/ERROR-CATALOG.md` state **200**). GAP-KDB R5 mints new
/// resolve_kem / RecipientBinding boundary codes → the count must GROW past
/// this floor. Reverting the mint drops it back → the RED count arm fails.
const CATALOG_THROWABLE_COUNT_AT_R3_BASE: u32 = 200;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn error_catalog() -> String {
    std::fs::read_to_string(repo_root().join("docs/ERROR-CATALOG.md"))
        .expect("ERROR-CATALOG.md must be present")
}

fn phase_4_backlog() -> String {
    std::fs::read_to_string(repo_root().join("docs/future/phase-4-backlog.md"))
        .expect("docs/future/phase-4-backlog.md must be present")
}

/// Extract the first `**<digits>**` bolded integer on the line carrying
/// `label`, from `docs/ERROR-CATALOG.md`. This is the doc-STATED count
/// (parsed from the doc, never a code literal).
fn doc_stated_count(doc: &str, label: &str) -> Option<u32> {
    let line = doc.lines().find(|l| l.contains(label))?;
    let mut rest = line;
    while let Some(open) = rest.find("**") {
        let after = &rest[open + 2..];
        if let Some(close) = after.find("**") {
            let inner = &after[..close];
            if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit()) {
                return inner.parse::<u32>().ok();
            }
            rest = &after[close + 2..];
        } else {
            break;
        }
    }
    None
}

// ===========================================================================
// ENG-3 BASELINE (NOT ignored) — drive the REAL enum↔catalog mirror.
// ===========================================================================

/// ENG-3 BASELINE — the enum↔catalog mirror is LIVE today: the REAL
/// `ErrorCode::TypedCallUnknownOp.as_str()` string appears as a `### E_…`
/// heading in the on-disk `ERROR-CATALOG.md`. Drives the real enum + the
/// real doc; an inert doc read would make the RED registration arm
/// vacuously pass.
#[test]
fn eng_3_enum_to_catalog_mirror_live_baseline() {
    let doc = error_catalog();
    let code = ErrorCode::TypedCallUnknownOp.as_str();
    assert_eq!(code, "E_TYPED_CALL_UNKNOWN_OP");
    assert!(
        doc.contains(&format!("### {code}")) || doc.contains(code),
        "ERROR-CATALOG.md MUST mirror the live ErrorCode::{code:?} — the \
         enum↔catalog mirror (§3.5g) is a real property, not a literal."
    );
}

/// ENG-3 BASELINE — the two doc-stated counts (`Throwable enum variants`
/// row + `Regression-list entries` row) are parsed FROM the doc and are
/// EQUAL (the 1:1 mirror the catalog itself claims; today both = 200).
/// Drives the real doc parser; would-FAIL if the two rows drifted.
#[test]
fn eng_3_catalog_count_rows_agree_baseline() {
    let doc = error_catalog();
    let throwable = doc_stated_count(&doc, "Throwable enum variants")
        .expect("the Throwable-enum-variants row MUST state a **count**");
    let regression = doc_stated_count(&doc, "Regression-list entries")
        .expect("the Regression-list-entries row MUST state a **count**");
    assert_eq!(
        throwable, regression,
        "the catalog's Throwable-enum-variants count ({throwable}) MUST equal \
         its Regression-list-entries count ({regression}) — the 1:1 mirror \
         (`catalog_roster.rs.in` + `src/lib.rs::catalog_roster_pin`)."
    );
    // Post-R5 (GAP-KDB doc-wave): the catalog GREW past the R3 base — the
    // sanity guard flips from "== R3 base (200)" to "> R3 base", a live
    // post-condition co-pinning the R5 mint (`E_RECIPIENT_KEM_NOT_COMMITTED`)
    // alongside the RED arm below. Kept as `>` (not `== 201`) so the
    // strategy-C integrator can reconcile the count if a sibling wave also
    // mints, per the historical #1319↔#1318 collision pattern. The 1:1
    // rows-agree mirror above is the load-bearing half.
    assert!(
        throwable > CATALOG_THROWABLE_COUNT_AT_R3_BASE,
        "post-R5: the throwable count ({throwable}) MUST have GROWN past the R3 \
         freeze base ({CATALOG_THROWABLE_COUNT_AT_R3_BASE}) — the GAP-KDB \
         recipient-binding boundary code was minted + mirrored."
    );
}

// ===========================================================================
// ENG-3 RED (ignored until R5) — GAP-KDB codes minted + mirrored + counted.
// ===========================================================================

/// ENG-3 RED — the ERROR-CATALOG mirrors the NEW GAP-KDB boundary-crossing
/// ErrorCodes AND the throwable count GREW past the R3 base. Name-agnostic
/// (the exact `E_…` names finalize at R5): asserts BOTH (1) the count grew
/// past 200, AND (2) the catalog documents a `E_…` code BOUND to the
/// GAP-KDB recipient-binding / KEM-commitment boundary.
///
/// would-FAIL-on-revert: dropping the mint lowers the count back to 200 AND
/// removes the boundary-bound code from the catalog.
#[test]
fn eng_3_catalog_mirrors_gapkdb_boundary_codes() {
    let doc = error_catalog();

    // (1) The doc-stated throwable count GREW past the R3 base (new codes
    //     minted + mirrored). The two count rows still agree (mirror held).
    let throwable =
        doc_stated_count(&doc, "Throwable enum variants").expect("throwable count row present");
    let regression =
        doc_stated_count(&doc, "Regression-list entries").expect("regression count row present");
    assert_eq!(
        throwable, regression,
        "the enum↔catalog 1:1 mirror MUST still hold after the GAP-KDB mint."
    );
    assert!(
        throwable > CATALOG_THROWABLE_COUNT_AT_R3_BASE,
        "the throwable ErrorCode count MUST GROW past the R3 base \
         ({CATALOG_THROWABLE_COUNT_AT_R3_BASE}) — GAP-KDB mints new \
         resolve_kem / RecipientBinding boundary codes. Got {throwable}."
    );

    // (2) The catalog documents a NEW `E_…` code BOUND to the GAP-KDB
    //     recipient-binding / KEM-commitment boundary (registry-binding, not
    //     a bare count bump).
    let bound = doc.lines().any(|l| {
        l.contains("E_")
            && (l.contains("resolve_kem")
                || l.contains("RecipientBinding")
                || l.contains("KEM commitment")
                || l.contains("KEM-commitment")
                || l.contains("committed by")
                || l.contains("key-set")
                || l.contains("did:benten"))
    });
    assert!(
        bound,
        "ERROR-CATALOG.md MUST document a NEW E_… code BOUND to the GAP-KDB \
         recipient-binding / KEM-commitment boundary (resolve_kem / \
         RecipientBinding typed-reject; §3.5g mirror). A bare count bump with \
         no boundary-bound code fails."
    );
}

// ===========================================================================
// ENG-4 BASELINE (NOT ignored) — drive the REAL #1073 record parser.
// ===========================================================================

/// ENG-4 BASELINE — the #1073 record is present in the on-disk
/// `phase-4-backlog.md` in its ORIGINAL "preserve did:key-only in
/// `Did::resolve` at v1 (DISAGREE)" shape. Proves the parser reads the doc;
/// an inert read would make the RED supersession arm vacuously pass.
#[test]
fn eng_4_1073_record_present_baseline() {
    let doc = phase_4_backlog();
    let line = doc
        .lines()
        .find(|l| l.contains("#1073"))
        .expect("phase-4-backlog.md MUST carry the #1073 record");
    assert!(
        line.contains("did:key") && (line.contains("resolve") || line.contains("Did(String)")),
        "the #1073 record MUST name the did:key-only `Did::resolve` \
         preservation clause at the R3 base (the clause Shape-B supersedes)."
    );
    // Post-R5 (GAP-KDB doc-wave): the #1073 supersession landed — the sanity
    // guard flips from "no did:benten at R3 base" to "now recorded", a live
    // post-condition co-pinning the R5 supersession alongside the RED arm.
    assert!(
        doc.contains("did:benten"),
        "post-R5: phase-4-backlog.md now records the did:benten supersession of \
         the #1073 did:key-only-resolve clause (design R1 §4 item 10)."
    );
}

// ===========================================================================
// ENG-4 RED (ignored until R5) — the #1073 clause is SUPERSEDED.
// ===========================================================================

/// ENG-4 RED — the #1073 did:key-only-resolve preservation clause is
/// RETIRED / SUPERSEDED: the record states that did:benten now exists in
/// `Did::resolve` at v1 (the Shape-B ratification supersedes the
/// "preserve did:key-only" DISAGREE clause; design R1 §4 item 10).
///
/// would-FAIL-on-revert: dropping the supersession note (leaving #1073's
/// clause silently authoritative) flips the arm.
#[test]
fn eng_4_1073_clause_superseded_by_did_benten() {
    let doc = phase_4_backlog();
    assert!(
        doc.contains("did:benten"),
        "phase-4-backlog.md MUST record that did:benten now exists in \
         Did::resolve at v1 (design R1 §4 item 10). R5 doc-wave adds it."
    );
    // The supersession is BOUND to #1073 (the specific clause retired), not
    // a stray did:benten mention.
    let superseded = doc.lines().any(|l| {
        l.contains("#1073")
            && (l.contains("supersed")
                || l.contains("retire")
                || l.contains("RETIRE")
                || l.contains("Retire")
                || l.contains("did:benten"))
    }) || {
        // Or a dedicated supersession sentence naming both.
        let lc = doc.to_lowercase();
        lc.contains("did:benten")
            && lc.contains("1073")
            && (lc.contains("supersed") || lc.contains("retire"))
    };
    assert!(
        superseded,
        "the #1073 did:key-only-resolve clause MUST be recorded SUPERSEDED / \
         RETIRED by did:benten (Shape-B ratification; design R1 §4 item 10). \
         A did:benten mention unbound to #1073 does not close the record."
    );
}
