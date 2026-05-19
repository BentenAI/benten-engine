//! TF-12 / META #907 — `#[non_exhaustive]` ONE-COHERENT-SWEEP freeze
//! conformance pin (G-CORE-9 FREEZE wave).
//!
//! ADDL R3-C1 (Phase-4-Meta-Core; last R3 wave; freeze-time). TDD
//! red-phase. Tests-only — NO production source.
//!
//! ## §3.6g LITERAL discipline checklist (reproduced, not §-referenced)
//!
//!  1. Land-when = FREEZE. Every RED pin here carries
//!     `#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]`. This file has
//!     NO GREEN arm (the §4.69 GREEN guard lives in a sibling file).
//!  2. Campaign-tail landed-vs-RED split (§3.5n): the #907 ONE-coherent
//!     sweep manifest + per-type recorded freeze-decision is a
//!     freeze-DELIVERABLE NOT yet built (ground-truth: several anchor
//!     types already carry `#[non_exhaustive]` but `benten-graph
//!     WriteContext` + `benten-graph store::ChangeEvent` do NOT, and no
//!     coherent recorded-decision manifest is committed) → RED.
//!  3. SHAPE-not-SUBSTANCE (pim-18 / §3.6f): this is NECESSARILY a
//!     structural/enumeration invariant — see the module-level "Why a
//!     structural pin is correct here" note. The would-FAIL signal is
//!     concrete: a real workspace source-tree scan + the committed
//!     enumeration manifest TSV; the pin FAILS if any APPLY-row type
//!     lacks the attribute OR any PENDING row survives the freeze OR a
//!     newly-added public enum/struct is absent from the manifest.
//!  4. pim-2 sub-rule-4 (§3.6b): the pin exercises the SPECIFIC #907
//!     obligation (every enumerated workspace public enum/struct has a
//!     recorded apply-or-D8 decision AND every APPLY row HAS the attr),
//!     not an umbrella "non_exhaustive is good" assertion.
//!  5. §3.13: no shared process-scoped static — per-test locals only.
//!  6. §3.5j: file compiles + passes MSRV-1.95 clippy AND
//!     `cargo +stable clippy` (scoped to benten-engine — never
//!     `--workspace`).
//!  7. §3.6e: no stranded `#[ignore]` pin citing a freeze surface is
//!     introduced or left dangling by this file; the manifest TSV's
//!     PENDING rows ARE the named G-CORE-9 un-ignore destination.
//!
//! ## Why a structural/enumeration pin is correct here (pim-18 waiver)
//!
//! A behavioral test is impossible for #907: `#[non_exhaustive]`'s
//! ENTIRE effect is on DOWNSTREAM (other-crate / post-v1) match/struct
//! exhaustiveness, and its ABSENCE is not observable from inside the
//! defining crate at all. `cargo-public-api` does NOT diff it (absence
//! of an attribute is not a public-API surface delta — SemVer-
//! asymmetric: ADDING `#[non_exhaustive]` post-v1 is the breaking
//! change, so the decision MUST be made AT the freeze). The only
//! faithful pin is: (a) a HAND-ENUMERATED manifest of every public
//! enum/struct workspace-wide with a per-type recorded freeze-decision
//! (the manifest IS the deliverable), plus (b) a source-tree scan that
//! FAILS if an APPLY row lacks the attribute, a PENDING row survives
//! the freeze, or a public type is missing from the manifest. This
//! mirrors R3-B6's #838 seam-shape reasoning: the test asserts the
//! freeze PROPERTY holds, with a concrete would-FAIL signal, because
//! the substantive consequence is structurally unobservable here.
//!
//! Pin source: r2-test-landscape.md TF-12 obligation (4) + plan
//! §1.A.FROZEN item 11 + §0 Freeze-completeness cluster (b) + the
//! batched `MORNING-DECISION-non-exhaustive-public-api.md` D8 carve-out.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// One enumeration-manifest row.
#[derive(Debug)]
struct ManifestRow {
    crate_name: String,
    ty: String,
    kind: String,
    src_path: String,
    freeze_decision: String,
    ground_truth: String,
}

fn parse_manifest() -> Vec<ManifestRow> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("tf12_fixtures")
        .join("non_exhaustive_workspace_enumeration.tsv");
    let body = std::fs::read_to_string(&manifest).unwrap_or_else(|e| {
        panic!(
            "TF-12 #907 enumeration manifest MUST exist at {} (it IS the \
             deliverable per §1.A.FROZEN item 11); error: {e}",
            manifest.display()
        )
    });
    let mut rows = Vec::new();
    for line in body.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if line.starts_with("crate\ttype\t") {
            continue; // header
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(
            cols.len() == 6,
            "TF-12 #907 manifest row must have 6 tab-separated columns; \
             got {} in line: {line:?}",
            cols.len()
        );
        rows.push(ManifestRow {
            crate_name: cols[0].to_string(),
            ty: cols[1].to_string(),
            kind: cols[2].to_string(),
            src_path: cols[3].to_string(),
            freeze_decision: cols[4].to_string(),
            ground_truth: cols[5].to_string(),
        });
    }
    rows
}

/// RED — un-ignore at G-CORE-9. The #907 ONE-coherent-sweep is COMPLETE:
/// zero `PENDING-G-CORE-9` rows survive the freeze (each anchor +
/// every workspace public enum/struct has a recorded apply-or-D8
/// decision). Pre-freeze this FAILS because the manifest deliberately
/// carries PENDING rows (WriteContext / store::ChangeEvent / the
/// benten-sync set / the benten-engine 11+ set) that the G-CORE-9
/// brief must resolve. Would-FAIL if the freeze ships with any
/// undecided public type (the exact SemVer-asymmetric break #907
/// guards).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn non_exhaustive_sweep_has_zero_pending_decisions_post_freeze() {
    let rows = parse_manifest();
    let pending: Vec<&ManifestRow> = rows
        .iter()
        .filter(|r| r.freeze_decision == "PENDING-G-CORE-9")
        .collect();
    assert!(
        pending.is_empty(),
        "TF-12 #907: {} public type(s) still PENDING the apply-or-D8 \
         freeze-decision at G-CORE-9 (the freeze MUST NOT ship with an \
         undecided public enum/struct — adding `#[non_exhaustive]` \
         post-v1 is the SemVer break #907 prevents): {:#?}",
        pending.len(),
        pending
    );
}

/// RED — un-ignore at G-CORE-9. Every `APPLY` manifest row's type
/// actually carries `#[non_exhaustive]` in its source file post-freeze;
/// every `D8-CARVE-OUT` row's type does NOT (deliberate exhaustive-by-
/// design — the carve-out is recorded, not silently dropped). This is
/// the would-FAIL backstop: comment out a `#[non_exhaustive]` on any
/// APPLY type and this pin fails post-G-CORE-9.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn non_exhaustive_apply_rows_carry_attribute_and_carveouts_do_not() {
    let root = workspace_root();
    let rows = parse_manifest();
    let mut violations: Vec<String> = Vec::new();

    for r in &rows {
        if r.freeze_decision == "PENDING-G-CORE-9" {
            // Resolved by the sibling zero-pending pin; skip here so
            // this pin's failure isolates the apply/carve-out
            // correctness specifically (pim-2 sub-rule-4 granularity).
            continue;
        }
        // The benten-engine placeholder row is a brief-expansion marker,
        // not a single concrete type — its concrete types are added at
        // G-CORE-9 and then checked by this same loop.
        if r.ty.starts_with("(11+ public enums") {
            continue;
        }
        let src = root.join(&r.src_path);
        let body = std::fs::read_to_string(&src).unwrap_or_else(|e| {
            panic!(
                "TF-12 #907: manifest row {}::{} cites {} which must be \
                 readable; error: {e}",
                r.crate_name,
                r.ty,
                src.display()
            )
        });
        // Heuristic source proximity check: the type's declaration line
        // must have `#[non_exhaustive]` within the preceding 6 lines for
        // APPLY rows (matches the rustfmt attribute-stacking convention
        // verified at ground-truth) and must NOT for D8-CARVE-OUT rows.
        let decl_needle = match r.kind.as_str() {
            "enum" => format!("enum {} ", strip_variant(&r.ty)),
            "struct" => format!("struct {} ", strip_variant(&r.ty)),
            "enum-variant" => {
                // Variant-level carve-out: assert the recorded comment
                // marker is present (the deliberate-NOT-applied note).
                // Ground-truthed at lib.rs:560 — the stable marker is
                // "`#[non_exhaustive]` deliberately NOT" but we match on
                // the robust co-presence of the two stable phrases so a
                // benign reword (e.g. line-wrap) does not false-fail.
                let has_carveout_marker =
                    body.contains("non_exhaustive") && body.contains("deliberately NOT");
                if !has_carveout_marker {
                    violations.push(format!(
                        "{}::{} is a D8-CARVE-OUT variant but the \
                         deliberate-NOT-applied recorded marker is \
                         absent in {}",
                        r.crate_name, r.ty, r.src_path
                    ));
                }
                continue;
            }
            other => panic!("unknown manifest kind {other:?}"),
        };
        let Some(decl_line) = body.lines().position(|l| l.contains(&decl_needle)) else {
            // A decl the manifest cites that no longer exists at the
            // recorded path IS a real would-FAIL signal (manifest /
            // source drift the freeze must reconcile) — record it as a
            // violation rather than panicking, so the failure isolates
            // the #907 conformance contract, not a harness abort.
            violations.push(format!(
                "manifest row {}::{} cites `{}` but `{decl_needle}` was \
                 not found there (manifest/source drift — the G-CORE-9 \
                 sweep must reconcile every enumerated type's path)",
                r.crate_name, r.ty, r.src_path
            ));
            continue;
        };
        let window_start = decl_line.saturating_sub(6);
        let attr_present = body
            .lines()
            .skip(window_start)
            .take(decl_line - window_start + 1)
            .any(|l| l.trim() == "#[non_exhaustive]");

        match r.freeze_decision.as_str() {
            "APPLY" if !attr_present => violations.push(format!(
                "APPLY row {}::{} ({}) is MISSING `#[non_exhaustive]` \
                 post-freeze",
                r.crate_name, r.ty, r.src_path
            )),
            "D8-CARVE-OUT" if attr_present => violations.push(format!(
                "D8-CARVE-OUT row {}::{} ({}) UNEXPECTEDLY carries \
                 `#[non_exhaustive]` (carve-out means deliberately \
                 exhaustive)",
                r.crate_name, r.ty, r.src_path
            )),
            _ => {}
        }
        // ground_truth column is informational provenance only; assert
        // it is one of the two legal tokens so the manifest stays well-
        // formed.
        assert!(
            r.ground_truth == "HAS" || r.ground_truth == "ABSENT",
            "manifest ground_truth column must be HAS|ABSENT, got {:?}",
            r.ground_truth
        );
    }

    assert!(
        violations.is_empty(),
        "TF-12 #907 apply/carve-out conformance failed post-G-CORE-9:\n{}",
        violations.join("\n")
    );
}

fn strip_variant(ty: &str) -> &str {
    ty.split("::").next().unwrap_or(ty)
}

/// RED — un-ignore at G-CORE-9. The §1.A.FROZEN item 11 NAMED anchor
/// set is fully present in the manifest (no named anchor silently
/// dropped during the brief's workspace-wide expansion). This is the
/// enumeration-completeness backstop for the named must-cover surface.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn non_exhaustive_manifest_covers_all_named_frozen_anchors() {
    let rows = parse_manifest();
    let have: std::collections::BTreeSet<String> = rows
        .iter()
        .map(|r| format!("{}::{}", r.crate_name, r.ty))
        .collect();
    // The §1.A.FROZEN item 11 verbatim anchor set.
    let required = [
        "benten-core::WriteAuthority",
        "benten-ivm::AlgorithmError",
        "benten-caps::TypedCapGroup",
        "benten-graph::WriteContext",
        "benten-graph::ChangeEvent",
        "benten-graph::GraphError",
    ];
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|a| !have.contains(*a))
        .collect();
    assert!(
        missing.is_empty(),
        "TF-12 #907: §1.A.FROZEN item 11 named anchor(s) absent from the \
         enumeration manifest (must be hand-enumerated — cargo-public-api \
         will NOT catch the gap): {missing:?}"
    );
}
