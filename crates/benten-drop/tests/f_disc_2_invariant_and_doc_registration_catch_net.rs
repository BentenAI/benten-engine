//! **F-DISC-2** — Invariant + doc registration catch-net
//! (GAP-5a — §9.1-7 doc-wave gate).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Pure doc-coupling family (reuses the `tf3f` shape: read the registry
//! docs from disk via `CARGO_MANIFEST_DIR`, assert the end-state
//! registration properties).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-DISC-2**:
//!   "`INVARIANT-COVERAGE.md` REGISTERS Inv-16..22 AND Inv-15 NOT
//!    re-registered AND header count correct end-state (M-15); 4 new docs
//!    exist (`CRYPTO-CODEPOINTS.md`, `THREAT-MODEL.md`,
//!    `SECURITY-PROOFS.md`, `compute-marketplace.md`);
//!    `V1-FROZEN-INTERFACE-DEFERRED.md` has D-28/D-29 (in-tree max D-27);
//!    EP-1 roster + 'Rust engine plugin' naming landed; §0.4 supersession
//!    recorded.  §9.1-7, M-15/M-16; pim-13.  FG/CF.  ~6-8 tests.
//!    No dimension owned the doc-wave deliverables that gate the freeze."
//!
//! **RED-PHASE (pim-12 §3.6e):** at baseline `INVARIANT-COVERAGE.md` tops
//! out at Inv-15; the 4 new docs are ABSENT; V1-FROZEN tops out at D-27.
//! The end-state registration arms are
//! `#[ignore = "RED-PHASE: F-DISC-2 ..."]`; R5's doc-wave un-ignores them.
//! The NON-ignored baseline arms drive the REAL on-disk
//! `INVARIANT-COVERAGE.md` parser (proving the harness reads the doc, not
//! a literal) — it recovers Inv-15 today and would-FAIL if the parser
//! went inert. NEVER `assert_eq!(CONST, CONST_VAL)`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::BTreeSet;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve from crates/benten-drop")
}

fn invariant_coverage_md() -> String {
    std::fs::read_to_string(repo_root().join("docs/INVARIANT-COVERAGE.md"))
        .expect("INVARIANT-COVERAGE.md must be present")
}

/// Enumerate the registered `Inv-<N>` numbers FROM the doc (not a literal).
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

// ===========================================================================
// BASELINE ARMS (NOT ignored) — drive the REAL invariant-doc parser.
// Pass green now; prove the parser reads the doc, not a literal.
// ===========================================================================

/// PIN 0 (baseline) — the parser enumerates invariants FROM the real
/// on-disk `INVARIANT-COVERAGE.md`. At baseline Inv-15 is the top
/// registered invariant; the parser MUST recover it. A no-op parser
/// (returning {}) regresses the catch-net silently — this is the
/// would-FAIL-if-no-op'd guard for the parser itself.
#[test]
fn f_disc_2_parser_enumerates_invariants_from_doc_baseline() {
    let doc = invariant_coverage_md();
    let invs = registered_invariants(&doc);
    assert!(
        invs.contains(&15),
        "registered_invariants MUST enumerate FROM the on-disk \
         INVARIANT-COVERAGE.md. At baseline Inv-15 is registered and MUST \
         be recovered. Got: {:?}. An inert parser would make every \
         registration pin vacuously pass.",
        invs
    );
    // Sanity: a low invariant (Inv-1 etc.) is also present — the doc is
    // a real registry, not a stub.
    assert!(
        invs.iter().any(|&n| n <= 14),
        "the invariant registry MUST carry the baseline Inv-1..Inv-14 set."
    );
}

// ===========================================================================
// RED-PHASE ARMS (ignored until R5) — assert the F-full registration
// end-state.
// ===========================================================================

/// PIN 1 — `INVARIANT-COVERAGE.md` REGISTERS Inv-16..22 (all 7 new
/// invariants). Parametrized over the closed range; enumerated from the
/// doc so an extra new invariant is auto-detected by PIN 0's parser.
/// Would-FAIL if any of Inv-16..22 is claimed-but-unregistered.
#[test]
fn f_disc_2_registers_inv_16_through_22() {
    let doc = invariant_coverage_md();
    let invs = registered_invariants(&doc);
    let missing: Vec<u32> = (16u32..=22).filter(|n| !invs.contains(n)).collect();
    assert!(
        missing.is_empty(),
        "INVARIANT-COVERAGE.md MUST register Inv-16..22 (the 7 new F-full \
         invariants). Missing: {:?}. R5 doc-wave registers them.",
        missing
    );
}

/// PIN 2 — header invariant-COUNT is the correct end-state (M-15) AND
/// Inv-15 is NOT RE-registered (no duplicate). The header count MUST
/// equal the highest registered invariant (22). Would-FAIL if the header
/// drifts from the body (a classic registry-drift) or Inv-15 is duplicated.
#[test]
fn f_disc_2_header_count_correct_and_inv15_not_re_registered() {
    let doc = invariant_coverage_md();
    let invs = registered_invariants(&doc);
    let highest = invs.iter().copied().max().unwrap_or(0);
    assert_eq!(
        highest, 22,
        "the highest registered invariant MUST be Inv-22 at the F-full \
         end-state. Got highest = Inv-{highest}."
    );
    // The header MUST name the count (22) — drift-defense between header
    // and body. We look for the literal count token near a header marker.
    assert!(
        doc.contains("22 invariant")
            || doc.contains("22 Invariant")
            || doc.contains("Inv-1..Inv-22")
            || doc.contains("Inv-1 .. Inv-22"),
        "the INVARIANT-COVERAGE.md header MUST state the end-state count \
         (22 invariants) so header and body don't drift (M-15)."
    );
    // Inv-15 appears, but MUST NOT be RE-registered as a NEW row in the
    // F-full mint block (no duplicate registration). We bound the count of
    // distinct `Inv-15` *registration-row* markers — at most one.
    let inv15_registration_rows = doc
        .lines()
        .filter(|l| {
            l.contains("Inv-15")
                && (l.contains("| Inv-15")
                    || l.trim_start().starts_with("### Inv-15")
                    || l.trim_start().starts_with("## Inv-15"))
        })
        .count();
    assert!(
        inv15_registration_rows <= 1,
        "Inv-15 MUST NOT be RE-registered by the F-full doc-wave (it was \
         minted at Phase-4-Meta-Core already). Found {inv15_registration_rows} \
         registration rows."
    );
}

/// PIN 3 — the 4 new docs EXIST at the repo `docs/` root:
/// `CRYPTO-CODEPOINTS.md`, `THREAT-MODEL.md`, `SECURITY-PROOFS.md`,
/// `future/compute-marketplace.md`. Doc-coupling existence. Would-FAIL if
/// the doc-wave gate is claimed-complete with a doc missing.
#[test]
fn f_disc_2_four_new_docs_exist() {
    let docs = repo_root().join("docs");
    let required = [
        docs.join("CRYPTO-CODEPOINTS.md"),
        docs.join("THREAT-MODEL.md"),
        docs.join("SECURITY-PROOFS.md"),
        docs.join("future/compute-marketplace.md"),
    ];
    let missing: Vec<String> = required
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.display().to_string())
        .collect();
    assert!(
        missing.is_empty(),
        "the 4 new F-full docs MUST exist (the §9.1-7 doc-wave gate). \
         Missing: {:?}. R5 doc-wave creates them.",
        missing
    );
}

/// PIN 4 — `V1-FROZEN-INTERFACE-DEFERRED.md` has rows D-28 + D-29
/// (in-tree max is D-27 at baseline). Coheres with F-FREEZE-1 PIN 6 but
/// owned HERE as the registration-catch-net. Would-FAIL if the deferrals
/// are claimed but the doc rows are absent.
#[test]
fn f_disc_2_v1_frozen_has_d28_d29() {
    let doc = std::fs::read_to_string(repo_root().join("docs/V1-FROZEN-INTERFACE-DEFERRED.md"))
        .expect("V1-FROZEN-INTERFACE-DEFERRED.md must be present");
    assert!(
        doc.contains("D-28") && doc.contains("D-29"),
        "V1-FROZEN-INTERFACE-DEFERRED.md MUST register D-28 + D-29 \
         (the compute/economic PHASE-LATER-DEFER rows; baseline max D-27)."
    );
}

/// PIN 5 — the EP-1 extension-point roster + the "Rust engine plugin"
/// naming landed in the architecture docs. Doc-coupling. Would-FAIL if
/// the naming/roster is claimed but absent (the EP-1 §6.3 conformance
/// gate).
#[test]
fn f_disc_2_ep1_roster_and_rust_engine_plugin_naming_landed() {
    let docs = repo_root().join("docs");
    // Search the arch-facing docs for the EP-1 roster + naming.
    let candidates = [
        docs.join("ARCHITECTURE.md"),
        docs.join("CRYPTO-CODEPOINTS.md"),
    ];
    let mut combined = String::new();
    for c in &candidates {
        combined.push_str(&std::fs::read_to_string(c).unwrap_or_default());
        combined.push('\n');
    }
    assert!(
        combined.contains("EP-1") || combined.contains("extension-point roster"),
        "the EP-1 extension-point roster MUST be documented (§6.3 EP-1)."
    );
    assert!(
        combined.contains("Rust engine plugin") || combined.contains("Rust-engine-plugin"),
        "the 'Rust engine plugin' naming MUST land (the engine-extension \
         symmetry naming, M-16)."
    );
}

/// PIN 6 — the §0.4 supersession (MLS-bracket `0x6380/0x6390` supersedes
/// the M-CONS-FINAL F8/F21 MembershipSet/Sealed-Sender assignment;
/// MembershipSetEncryption = `0x6600`) is RECORDED in the codepoint doc.
/// Doc-coupling. Would-FAIL if the supersession is applied in code but the
/// rationale is undocumented (a silent codepoint reassignment).
#[test]
fn f_disc_2_section_0_4_supersession_recorded() {
    let codepoints =
        std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md")).unwrap_or_default();
    assert!(
        codepoints.contains("0x6380") && codepoints.contains("0x6390"),
        "CRYPTO-CODEPOINTS.md MUST record the MLS-bracket codepoints \
         (0x6380 MLS-Application / 0x6390 MLS-Welcome) per the §0.4 \
         supersession."
    );
    assert!(
        codepoints.contains("0x6600"),
        "CRYPTO-CODEPOINTS.md MUST record MembershipSetEncryption = 0x6600 \
         (the §0.4 supersession of the M-CONS-FINAL F8/F21 assignment)."
    );
}

/// PIN 7 (F2 / R4.6) — the doc-registration catch-net MUST protect THIS
/// round's own two wire corrections: `CRYPTO-CODEPOINTS.md` MUST register
/// the group-band codepoints `0x6610` (`MEMBERSHIP_SET_GROUP_MULTI_STANZA`,
/// the R4.6-corrected value — was slipped to the `0x6600` set-keying value
/// in "settled" territory) and `0x6520` (`LAYER_C_DROP_MULTI_RECIPIENT`,
/// the R0.7-blinded Layer-C group multi-stanza value).
///
/// Both are **FREEZE** rows in R0.7 §4.0 (allocation table lines 1001 +
/// 1010) / §4.1 (AAD field-set rows), and §4.0 names
/// `docs/CRYPTO-CODEPOINTS.md` as the in-tree home of that table (spec
/// lines 218 + 979). The in-code wire-lock for both constants is strong
/// (`crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs`
/// regression-guards `MEMBERSHIP_SET_GROUP_MULTI_STANZA == 0x6610` and
/// `LAYER_C_DROP_MULTI_RECIPIENT == 0x6520` directly) — so this pin is
/// DOC-coupling completeness, not a byte defect: it closes the
/// doc-registration blind spot that sat exactly where the wire changed
/// this round.
///
/// **Why FIX-NOW (not deferred to the R5 doc-wave):** the red-phase
/// catch-net infrastructure exists NOW and, by its own design, must
/// protect this round's own corrections; deferring leaves the doc-coupling
/// blind spot precisely on the two codepoints R4.6/R0.7 corrected (HARD
/// RULE 12 — no "minor enough to defer"). The test stays `#[ignore]`
/// because the doc itself (`CRYPTO-CODEPOINTS.md`) is created at the R5
/// doc-wave (spec line 206: `ls` → absent at baseline). Additive +
/// byte-neutral (no frozen golden, codepoint constant, or AAD layout
/// changes).
///
/// **Registry-binding (not a bare substring):** §4.0's table BINDS each
/// codepoint to a SYMBOL on one logical row, and the slip this round
/// guards against was precisely a value bound to the WRONG meaning
/// (`0x6610` slipped to the `0x6600` set-keying symbol). So — matching
/// PIN 0's "enumerate FROM the doc, don't match a literal" discipline and
/// PIN 6's row shape — this pin asserts each codepoint is CO-LOCATED with
/// its registered symbol (`0x6610`↔`MEMBERSHIP_SET_GROUP_MULTI_STANZA`,
/// `0x6520`↔`LAYER_C_DROP_MULTI_RECIPIENT`) on the same row. A bare
/// `.contains("0x6610")` would vacuously pass on a prose mention (e.g.
/// inside PIN 6's `0x6600` rationale narrative) — the row-binding check
/// would-FAIL if R5 registered the value against the wrong symbol or only
/// named it in prose.
#[test]
fn f_disc_2_records_r46_group_codepoints_0x6610_0x6520() {
    let codepoints =
        std::fs::read_to_string(repo_root().join("docs/CRYPTO-CODEPOINTS.md")).unwrap_or_default();

    // A codepoint is "registered" only when its value is CO-LOCATED with
    // its symbol on one logical row (the §4.0 allocation-table shape) — the
    // registry-drift guard PIN 6 uses, and the exact class this round's
    // slip (0x6610 bound to the 0x6600 symbol) belongs to.
    let registered_on_one_row = |value: &str, symbol: &str| -> bool {
        codepoints
            .lines()
            .any(|l| l.contains(value) && l.contains(symbol))
    };

    assert!(
        registered_on_one_row("0x6610", "MEMBERSHIP_SET_GROUP_MULTI_STANZA"),
        "CRYPTO-CODEPOINTS.md MUST register 0x6610 BOUND TO its symbol \
         MEMBERSHIP_SET_GROUP_MULTI_STANZA on one row (the R4.6-corrected \
         value; distinct from the 0x6600 set-keying value the R4.5b \
         migration slipped to). R0.7 §4.0 allocation table line 1010 \
         FREEZES it; §4.0 names this doc its in-tree home. A prose-only \
         mention or a value bound to the wrong symbol MUST fail."
    );
    assert!(
        registered_on_one_row("0x6520", "LAYER_C_DROP_MULTI_RECIPIENT"),
        "CRYPTO-CODEPOINTS.md MUST register 0x6520 BOUND TO its symbol \
         LAYER_C_DROP_MULTI_RECIPIENT on one row (the R0.7-blinded Layer-C \
         group multi-stanza value). R0.7 §4.0 allocation table line 1001 \
         FREEZES it (per-stanza AAD BLINDED; see §3.3 / §4.1). A prose-only \
         mention or a value bound to the wrong symbol MUST fail."
    );
}
