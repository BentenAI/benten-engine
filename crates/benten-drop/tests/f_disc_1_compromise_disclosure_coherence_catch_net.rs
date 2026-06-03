//! **F-DISC-1** — Compromise disclosure-coherence catch-net (GAP-2).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Reuses the `tf3f_revocation_reach_forever_valid_documented.rs`
//! doc-coupling shape (read `SECURITY-POSTURE.md` from disk via
//! `CARGO_MANIFEST_DIR`, grep-assert structural disclosure properties).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-DISC-1**:
//!   "for EACH Compromise #30..#63: (a) `SECURITY-POSTURE.md` row exists
//!    with correct `disposition_class` (ATO/SGD/CHD/OOS/MIT); (b) OOS/SGD
//!    disclosure text present + not over-claimed; (c) the BR-2 re-point
//!    triple (#31=LAMPS, #62=revocation-reach, #30=unaudited-PQ) at correct
//!    slots; #32 Decap-CT + #34 + #36 + #39 + #41 + #53 + #59 disclosures
//!    present.  §5.2, §9.1-7, BR-2; pim-13 §3.12.  FG/DC.  ~10-14 tests.
//!    Reuses `tf3f` shape. The extra-reflection-pass elegant single-shape
//!    per `feedback_extra_reflection_pass_for_elegant_permanent_shape`."
//!
//! **Single-shape rationale + R4 NOTE (§5 thinness mitigation):** this
//! family closes 18 honest-disclosure Compromise rows in ONE parametrized
//! family. The parametrization enumerates from the DOC row-set
//! (`extract_compromise_rows`), NOT a literal hand-list — so a new
//! Compromise row authored at R5 is auto-included in the coherence sweep.
//! R4 should verify the parametrization enumerates from the doc.
//!
//! **R4-FIX (F4-043) — word-boundary disposition-class match.** PIN 2
//! previously matched a disposition-class token with a bare substring scan
//! (`window.contains("MIT")`), which spuriously satisfies the coverage
//! requirement when the 3-letter token appears INSIDE an unrelated word
//! ("MIT" inside "co**MIT**ted"/"sub**MIT**", "ATO" inside "neg**ATO**r",
//! "OOS" inside "l**OOS**e", "SGD" / "CHD" inside hex/identifiers). A row
//! authored WITHOUT a real disposition class but whose window happens to
//! contain such a substring would pass the §5 auto-include coherence sweep
//! — defeating its whole point. The fix replaces the substring scan with a
//! `window_has_disposition_class` word-boundary matcher (the token must be
//! delimited by non-`[A-Za-z0-9_-]` boundaries on both sides, or be at a
//! line/window edge), so only a stand-alone disposition-class TOKEN counts.
//! A `_baseline` arm proves the matcher itself rejects an embedded
//! substring (would-FAIL if the matcher relaxed back to `.contains`).
//!
//! **RED-PHASE (pim-12 §3.6e):** the F-full disclosure work
//! (Compromise #32..#63 rows + the `disposition_class` taxonomy
//! ATO/SGD/CHD/OOS/MIT) does NOT exist at baseline — only #30/#31 are
//! present and the taxonomy is MIT/ATO-only. The end-state coherence
//! arms are `#[ignore = "RED-PHASE: F-DISC-1 ..."]`; R5's doc-wave
//! un-ignores them. The NON-ignored baseline arms drive the REAL on-disk
//! `SECURITY-POSTURE.md` + the REAL `extract_compromise_rows` parser
//! (proving the harness parses the doc, not a literal) and pass green now.
//!
//! Each pin drives an OBSERVABLE consequence (a structural grep over the
//! real doc) + is would-FAIL-if-no-op'd (a stub that returns `true`
//! unconditionally would let an over-claimed OOS row or a wrong-slot
//! re-point through). NEVER `assert_eq!(CONST, CONST_VAL)`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::BTreeSet;

// ===========================================================================
// SELF-CONTAINED STUB-SHIM (no cross-wave module deps; parallel-safe).
// R5 keeps this parser (it reads the real doc) OR lifts it into a shared
// `tests/support` module; it intentionally has NO dependency on another
// R3 wave's crate so this file compiles in isolation.
// ===========================================================================

/// The five legitimate disposition classes the F-full posture taxonomy
/// uses. Stable token set the doc-wave row scheme must encode.
///   ATO = Accepted-Trade-Off
///   SGD = Scoped-Gap-Disclosure
///   CHD = Closed-by-Hardening/Design
///   OOS = Out-Of-Scope
///   MIT = Mitigated
const DISPOSITION_CLASSES: &[&str] = &["ATO", "SGD", "CHD", "OOS", "MIT"];

/// A parsed Compromise row: its number + the line text we found it on.
#[derive(Debug, Clone)]
struct CompromiseRow {
    number: u32,
    line: String,
}

fn security_posture_md() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/SECURITY-POSTURE.md"
    ))
    .expect("SECURITY-POSTURE.md must be present at repo root /docs/")
}

/// Enumerate Compromise rows FROM THE DOC (not a hand-list). The
/// regex-free scan looks for the literal `Compromise #<N>` token and
/// records the line. This is the load-bearing "parametrize over the
/// doc row-set" property the §5 mitigation calls for.
fn extract_compromise_rows(doc: &str) -> Vec<CompromiseRow> {
    let mut rows = Vec::new();
    for line in doc.lines() {
        // Find every `Compromise #<digits>` occurrence on the line.
        let mut search = line;
        while let Some(idx) = search.find("Compromise #") {
            let after = &search[idx + "Compromise #".len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                rows.push(CompromiseRow {
                    number: n,
                    line: line.to_string(),
                });
            }
            // advance past this hit
            let consumed = idx + "Compromise #".len() + digits.len().max(1);
            if consumed >= search.len() {
                break;
            }
            search = &search[consumed..];
        }
    }
    rows
}

/// Distinct Compromise numbers the doc declares, as a sorted set.
fn distinct_compromise_numbers(doc: &str) -> BTreeSet<u32> {
    extract_compromise_rows(doc)
        .into_iter()
        .map(|r| r.number)
        .collect()
}

/// R4-FIX (F4-043) — is `token` present in `haystack` as a STAND-ALONE
/// word, not merely as a substring? A disposition-class token (`MIT`,
/// `ATO`, …) only "covers" a Compromise row when it appears as its own
/// token — delimited on both sides by something other than an
/// identifier-continuation char `[A-Za-z0-9_-]` (or the string edge).
///
/// This is what distinguishes a real `| ... | MIT |` table cell from the
/// accidental `MIT` inside "com**mit**ted" / "sub**mit**". Without it the
/// §5 auto-include coherence sweep is defeated: a row with NO real class
/// passes because some unrelated English word in its window embeds the
/// 3-letter token.
fn contains_token_word_boundary(haystack: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    let bytes = haystack.as_bytes();
    let tbytes = token.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = haystack[start..].find(token) {
        let at = start + rel;
        let end = at + tbytes.len();
        // Char immediately before the match (None ⇒ at string start).
        let before_ok = at == 0
            || !haystack[..at]
                .chars()
                .next_back()
                .is_some_and(is_ident);
        // Char immediately after the match (None ⇒ at string end).
        let after_ok = end >= bytes.len()
            || !haystack[end..].chars().next().is_some_and(is_ident);
        if before_ok && after_ok {
            return true;
        }
        // Advance past this occurrence and keep scanning.
        start = at + 1;
        if start >= haystack.len() {
            break;
        }
    }
    false
}

/// R4-FIX (F4-043) — does `window` name ANY disposition class as a
/// stand-alone token? (word-boundary, not substring).
fn window_has_disposition_class(window: &str) -> bool {
    DISPOSITION_CLASSES
        .iter()
        .any(|c| contains_token_word_boundary(window, c))
}

// ===========================================================================
// BASELINE ARMS (NOT ignored) — drive the REAL doc + REAL parser.
// These pass GREEN now and prove the harness reads the doc, not a literal.
// They are the would-FAIL-if-no-op'd guarantee for the parser itself.
// ===========================================================================

/// PIN 0a (baseline) — the parser enumerates Compromise rows FROM the
/// real on-disk doc. At baseline #30 + #31 are present; the parser MUST
/// recover them. A no-op parser returning `[]` fails here.
#[test]
fn f_disc_1_parser_enumerates_from_doc_not_literal_baseline() {
    let doc = security_posture_md();
    let numbers = distinct_compromise_numbers(&doc);

    // The parser recovers real rows from the doc (currently #30 + #31 at
    // minimum). This is the property R4 verifies: enumeration from the
    // doc row-set, not a hand-list.
    assert!(
        numbers.contains(&30) && numbers.contains(&31),
        "extract_compromise_rows MUST enumerate Compromise rows FROM the \
         on-disk SECURITY-POSTURE.md. At baseline #30 (unaudited-PQ) and \
         #31 (LAMPS EUF-CMA) are present and MUST be recovered. Got: {:?}. \
         A no-op parser (returning []) regresses the whole F-DISC-1 \
         catch-net silently — this is the §5 single-point-of-failure guard.",
        numbers
    );
}

/// PIN 0b (baseline) — the disposition-class taxonomy token set is
/// internally well-formed (exactly the 5 stable tokens, no dupes). This
/// is a structural guard so the R5 row scheme can't silently drift the
/// taxonomy. Would-FAIL if a 6th class is smuggled in or one is dropped.
#[test]
fn f_disc_1_disposition_class_taxonomy_is_exactly_five_baseline() {
    let as_set: BTreeSet<&str> = DISPOSITION_CLASSES.iter().copied().collect();
    assert_eq!(
        as_set.len(),
        DISPOSITION_CLASSES.len(),
        "disposition-class taxonomy MUST have no duplicate tokens."
    );
    assert_eq!(
        as_set,
        BTreeSet::from(["ATO", "CHD", "MIT", "OOS", "SGD"]),
        "the F-full posture taxonomy is EXACTLY {{ATO, SGD, CHD, OOS, MIT}}. \
         A drift here means a disclosure row could carry an unrecognized \
         class and pass the coherence sweep — would-FAIL-if-relaxed."
    );
}

/// PIN 0c (baseline) — R4-FIX (F4-043): the word-boundary matcher MUST
/// reject a disposition-class token that appears only as an EMBEDDED
/// substring, and MUST accept it as a stand-alone table-cell token. This
/// is the regression guard for the F4-043 fix itself: if the matcher ever
/// relaxes back to a bare `.contains`, this baseline arm fires RED.
///
/// would-FAIL if `window_has_disposition_class` used substring matching
/// (then `"committed deliverable"` — which embeds `MIT` — would falsely
/// report a class).
#[test]
fn f_disc_1_disposition_class_match_is_word_boundary_not_substring_baseline() {
    // Embedded-only substrings that MUST NOT count as a class:
    //   "committed" / "submit"  embed MIT
    //   "negator" / "ratoned"   embed ATO
    //   "loose"                 embeds OOS
    let embedded_only = [
        "the row was committed to the doc-wave deliverable",
        "we must submit the audit before tag",
        "a conservative negator on the rate-limit",
        "the binding is loose at this boundary",
    ];
    for w in embedded_only {
        assert!(
            !window_has_disposition_class(w),
            "F-DISC-1 (F4-043): an EMBEDDED disposition-class substring \
             MUST NOT count as coverage. Window {w:?} contains no \
             stand-alone class token, yet was reported as covered — the \
             matcher regressed to substring matching."
        );
    }

    // Stand-alone tokens (real table cells / inline notes) MUST count:
    let standalone = [
        "| Compromise #43 | metadata leakage | ATO | 9-eyes |",
        "disposition_class = SGD (scoped-gap disclosure)",
        "this is CHD — closed by design",
        "residual is OOS for v1-beta",
        "#34 password-knowledge … MIT.",
    ];
    for w in standalone {
        assert!(
            window_has_disposition_class(w),
            "F-DISC-1 (F4-043): a STAND-ALONE disposition-class token MUST \
             count as coverage. Window {w:?} carries a real class token but \
             was reported uncovered — the matcher is too strict."
        );
    }
}

// ===========================================================================
// RED-PHASE ARMS (ignored until R5 doc-wave) — assert the F-full
// end-state disclosure coherence over Compromise #30..#63.
// ===========================================================================

/// PIN 1 — every Compromise #30..#63 row EXISTS in the doc.
/// Parametrized over the closed F-full range (the R5 end-state mints all
/// of #32..#63); the sweep is driven by `distinct_compromise_numbers`
/// so any NEW row beyond #63 is auto-swept by PIN 2.
#[test]
#[ignore = "RED-PHASE: F-DISC-1 — all Compromise #30..#63 rows present in \
            SECURITY-POSTURE.md; #32..#63 minted by R5 F-full doc-wave; \
            un-ignore at R5"]
fn f_disc_1_all_compromise_30_through_63_rows_present() {
    let doc = security_posture_md();
    let numbers = distinct_compromise_numbers(&doc);

    let mut missing: Vec<u32> = Vec::new();
    for n in 30u32..=63 {
        if !numbers.contains(&n) {
            missing.push(n);
        }
    }
    assert!(
        missing.is_empty(),
        "SECURITY-POSTURE.md MUST carry a row for EVERY Compromise \
         #30..#63 (the F-full closed range). Missing: {:?}. R5 doc-wave \
         mints #32..#63.",
        missing
    );
}

/// PIN 2 — disclosure-coherence: every row the doc declares carries a
/// recognized `disposition_class` token on (or adjacent to) its line.
/// Parametrized over the DOC row-set (not a hand-list) so new rows are
/// auto-included — the §5 mitigation made executable. Would-FAIL if a
/// row is added without a disposition class (the silent-drift case).
///
/// R4-FIX (F4-043): coverage requires a stand-alone disposition-class
/// TOKEN (`window_has_disposition_class`, word-boundary), NOT a bare
/// substring — so a row whose window merely embeds "MIT"/"ATO"/… inside
/// an unrelated word is NOT spuriously reported as covered.
#[test]
#[ignore = "RED-PHASE: F-DISC-1 — every declared Compromise row carries a \
            disposition_class (ATO/SGD/CHD/OOS/MIT); auto-includes new rows; \
            un-ignore at R5"]
fn f_disc_1_every_declared_row_has_a_disposition_class() {
    let doc = security_posture_md();
    let rows = extract_compromise_rows(&doc);
    assert!(
        !rows.is_empty(),
        "doc must declare at least one Compromise row (parser sanity)."
    );

    // For each declared row, the line (or its immediate vicinity) must
    // name one of the 5 classes as a STAND-ALONE token (R4-FIX F4-043).
    // We allow a small lookahead window since the disposition can sit in
    // the same table row / following cell.
    let lines: Vec<&str> = doc.lines().collect();
    let mut uncovered: Vec<u32> = Vec::new();
    for row in &rows {
        // Locate the row's line index, then scan a small window.
        let mut covered = false;
        for (i, l) in lines.iter().enumerate() {
            if l.contains(&format!("Compromise #{}", row.number)) {
                let lo = i.saturating_sub(1);
                let hi = (i + 3).min(lines.len());
                let window = lines[lo..hi].join(" ");
                if window_has_disposition_class(&window) {
                    covered = true;
                    break;
                }
            }
        }
        if !covered {
            uncovered.push(row.number);
        }
    }
    // dedup
    uncovered.sort_unstable();
    uncovered.dedup();
    assert!(
        uncovered.is_empty(),
        "every declared Compromise row MUST carry a disposition_class token \
         (ATO/SGD/CHD/OOS/MIT) as a stand-alone token within its row window. \
         Uncovered: {:?}. Because this enumerates from the DOC (not a \
         literal list) AND matches on word boundaries (not substrings), a \
         new Compromise row authored without a real class fails HERE — the \
         auto-include property that closes the §5 single-point-of-failure.",
        uncovered
    );
}

/// PIN 3 — the BR-2 re-point triple is at the correct slots:
///   #31 = LAMPS (EUF-CMA / Composite ML-DSA),
///   #62 = revocation-reach,
///   #30 = unaudited-PQ.
/// Would-FAIL if a re-point lands on the wrong number (e.g. #31 narrated
/// as revocation-reach), which would silently mis-route every reader.
#[test]
#[ignore = "RED-PHASE: F-DISC-1 — BR-2 re-point triple (#31=LAMPS, \
            #62=revocation-reach, #30=unaudited-PQ) at correct slots; \
            un-ignore at R5"]
fn f_disc_1_br2_re_point_triple_at_correct_slots() {
    let doc = security_posture_md();
    let lines: Vec<&str> = doc.lines().collect();

    let row_window = |n: u32| -> String {
        for (i, l) in lines.iter().enumerate() {
            if l.contains(&format!("Compromise #{}", n)) {
                let lo = i.saturating_sub(1);
                let hi = (i + 4).min(lines.len());
                return lines[lo..hi].join(" ");
            }
        }
        String::new()
    };

    let w31 = row_window(31);
    assert!(
        w31.contains("LAMPS") || w31.contains("Composite ML-DSA") || w31.contains("EUF-CMA"),
        "Compromise #31 MUST be the LAMPS Composite ML-DSA / EUF-CMA-only \
         re-point slot (BR-2). Got window: {:?}",
        w31
    );

    let w62 = row_window(62);
    assert!(
        w62.contains("revocation-reach")
            || w62.contains("revocation reach")
            || w62.contains("forever-valid"),
        "Compromise #62 MUST be the revocation-reach re-point slot (BR-2)."
    );

    let w30 = row_window(30);
    assert!(
        w30.contains("unaudited") || w30.contains("PQ") || w30.contains("audit"),
        "Compromise #30 MUST be the unaudited-PQ re-point slot (BR-2)."
    );
}

/// PIN 4 — OOS / SGD disclosure text PRESENT and NOT over-claimed for the
/// honest-disclosure rows the landscape enumerates
/// (#37/#38/#40/#44/#47/#49/#50/#51/#55/#57). Over-claim guard: an
/// honest-disclosure row must NOT use closure language ("fully mitigated",
/// "fully closed", "eliminated", "no residual risk"). Would-FAIL if a
/// scoped-gap row is dressed up as fully closed.
#[test]
#[ignore = "RED-PHASE: F-DISC-1 — OOS/SGD honest-disclosure rows present + \
            not over-claimed (no 'fully closed/eliminated/no residual' on a \
            scoped-gap row); un-ignore at R5"]
fn f_disc_1_oos_sgd_disclosures_present_and_not_over_claimed() {
    let doc = security_posture_md();
    let lines: Vec<&str> = doc.lines().collect();

    // The honest-disclosure-only rows per §2.2 of the landscape.
    let honest_only: &[u32] = &[37, 38, 40, 44, 47, 49, 50, 51, 55, 57];
    let over_claim_tokens: &[&str] = &[
        "fully mitigated",
        "fully closed",
        "no residual risk",
        "completely eliminated",
        "entirely eliminated",
    ];

    let mut absent: Vec<u32> = Vec::new();
    let mut over_claimed: Vec<u32> = Vec::new();
    for &n in honest_only {
        let mut found = false;
        for (i, l) in lines.iter().enumerate() {
            if l.contains(&format!("Compromise #{}", n)) {
                found = true;
                let lo = i.saturating_sub(1);
                let hi = (i + 4).min(lines.len());
                let window = lines[lo..hi].join(" ").to_ascii_lowercase();
                if over_claim_tokens.iter().any(|t| window.contains(t)) {
                    over_claimed.push(n);
                }
            }
        }
        if !found {
            absent.push(n);
        }
    }
    assert!(
        absent.is_empty(),
        "honest-disclosure (OOS/SGD) Compromise rows MUST be present: \
         missing {:?}.",
        absent
    );
    assert!(
        over_claimed.is_empty(),
        "honest-disclosure (OOS/SGD) rows MUST NOT be over-claimed with \
         closure language. Over-claimed: {:?}. A scoped-gap disclosure \
         dressed as 'fully closed' is exactly the dishonesty BR-2 guards.",
        over_claimed
    );
}

/// PIN 5 — the load-bearing thinly-touched disclosures are PRESENT:
/// #32 Decap-CT, #34 password-knowledge, #36 RAM/coredump, #39
/// supply-chain+secrecy, #41 sync-UX-vs-crypto, #53 TransportConfig,
/// #59 KEM-key-confirmation. These were enumerated explicitly in the
/// F-DISC-1 spec body (not just the parametrized sweep). Would-FAIL if
/// any of the seven is silently dropped.
#[test]
#[ignore = "RED-PHASE: F-DISC-1 — load-bearing disclosures present \
            (#32/#34/#36/#39/#41/#53/#59); un-ignore at R5"]
fn f_disc_1_load_bearing_disclosures_present() {
    let doc = security_posture_md();
    let numbers = distinct_compromise_numbers(&doc);
    let required: &[u32] = &[32, 34, 36, 39, 41, 53, 59];
    let missing: Vec<u32> = required
        .iter()
        .copied()
        .filter(|n| !numbers.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "the load-bearing thinly-touched Compromise disclosures named in \
         the F-DISC-1 spec MUST be present: missing {:?}.",
        missing
    );
}
