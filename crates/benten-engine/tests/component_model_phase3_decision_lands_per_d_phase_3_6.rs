//! Phase 3 G20-A3 — D-PHASE-3-6 / D-PHASE-3-16 ratification pin.
//!
//! Pin source:
//!   - `.addl/phase-3/00-implementation-plan.md` D-PHASE-3-6
//!     RESOLVED-at-R1 (HELD CUT — wasmtime Component-Model stays
//!     removed for Phase 3).
//!   - D-PHASE-3-16 ratified 2026-05-05 — named destination
//!     "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA".
//!   - `docs/future/phase-3-backlog.md §7.3.A.8` (rationale rewrite —
//!     CLOSED at G20-A3).
//!   - `docs/FULL-ROADMAP.md` Phase 4 entry — destination must be
//!     named there per HARD RULE clause-(b) IFF clause.
//!
//! This test is the structural drift detector for the
//! D-PHASE-3-6 + D-PHASE-3-16 decision pair: any future agent that
//! tries to silently re-enable Component-Model in Phase 3 trips this
//! pin (Cargo.toml feature absent), and any future agent that drops
//! the Phase-4 named destination from FULL-ROADMAP.md trips the
//! companion pin (HARD RULE clause-(b) compliance check).
//!
//! D-PHASE-3-6 standing rule: if wasmtime Component-Model GA-stability
//! changes materially mid-Phase-3, this pin surfaces the next time CI
//! runs; orchestrator re-opens the decision via Ben.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `wasmtime_component_model_feature_held_cut_per_d_phase_3_6` —
/// scans `crates/benten-eval/Cargo.toml` for an enabled `component-model`
/// feature on the WASMTIME DEP declaration line(s). The decision is
/// HELD CUT (D-PHASE-3-6 RESOLVED-at-R1); enabling the feature in a
/// wasmtime dep stanza requires re-opening the decision via Ben.
///
/// NOTE: a `[features]` table entry like `component-model = []`
/// (Phase-2b vestige + Cargo's own feature-definition shape) is
/// permitted — that's a feature DEFINITION, not an enabled-on-the-
/// wasmtime-dep entry. The check below only flags lines that look
/// like wasmtime dep declarations carrying the feature.
#[test]
fn wasmtime_component_model_feature_held_cut_per_d_phase_3_6() {
    let root = workspace_root();
    let eval_cargo = root.join("crates/benten-eval/Cargo.toml");
    let cargo_src = std::fs::read_to_string(&eval_cargo).unwrap_or_else(|e| {
        panic!(
            "crates/benten-eval/Cargo.toml not found at {} ({}); load-bearing for wasmtime dep declaration.",
            eval_cargo.display(),
            e
        );
    });

    // Walk lines looking for wasmtime dep lines with component-model
    // in the features list. Wasmtime dep declarations look like:
    //   wasmtime = { version = "...", features = ["..."] }
    // or as a [dependencies.wasmtime] table:
    //   [dependencies.wasmtime]
    //   features = ["..."]
    // Conservative heuristic: a line that mentions both `wasmtime`
    // AND `component-model` (case-insensitive) is suspect. A line in
    // a [features] table mapping `component-model = [...]` does NOT
    // mention `wasmtime` — that's a feature DEFINITION, permitted.
    for (line_idx, line) in cargo_src.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        let has_wasmtime_component_model =
            lower.contains("wasmtime") && lower.contains("component-model");
        assert!(
            !has_wasmtime_component_model,
            "crates/benten-eval/Cargo.toml line {}: the wasmtime dep \
             declaration carries an active `component-model` feature — \
             D-PHASE-3-6 RESOLVED-at-R1 (2026-05-05) HELD THE CUT for \
             Phase 3. Re-enabling requires re-opening the decision via \
             Ben + re-running the §7.3.A.8 cluster end-to-end. See \
             .addl/phase-3/00-implementation-plan.md D-PHASE-3-6 + \
             D-PHASE-3-16. Line: {:?}",
            line_idx + 1,
            line
        );
    }
}

/// `phase_3_backlog_names_component_model_destination` —
/// HARD RULE clause-(b) realness check: D-PHASE-3-16's named
/// destination is "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA";
/// per the 2026-05-05 ratification standing rule, the destination is
/// realness-acceptable IFF it is named in the public roadmap doc
/// AND Phase 4 pre-R1 inherits the deferral when Phase 4 opens.
///
/// `docs/FULL-ROADMAP.md` is local-only-tracked (gitignored — the
/// public companion lives in `docs/future/phase-3-backlog.md` which
/// IS tracked). The test asserts the §7.3.A.8 entry in the tracked
/// `phase-3-backlog.md` continues to name the Phase-4 wasmtime
/// Component-Model destination — IFF-clause violation surfaces
/// immediately if a future agent silently drops the named destination.
#[test]
fn phase_3_backlog_names_component_model_destination() {
    let root = workspace_root();
    let backlog = root.join("docs/future/phase-3-backlog.md");
    let src = std::fs::read_to_string(&backlog).unwrap_or_else(|e| {
        panic!(
            "docs/future/phase-3-backlog.md not found at {} ({}); HARD RULE \
             clause-(b) IFF clause cannot be evaluated without the destination doc.",
            backlog.display(),
            e
        );
    });
    let lower = src.to_ascii_lowercase();
    assert!(
        lower.contains("component-model") || lower.contains("component model"),
        "docs/future/phase-3-backlog.md MUST mention wasmtime Component-Model \
         per D-PHASE-3-16 named destination ratification 2026-05-05 (HARD \
         RULE clause-(b) IFF clause). Without the named destination, the \
         §7.3.A.8 ESC-11/-12 test deferral rationales fail HARD RULE \
         compliance."
    );
    assert!(
        lower.contains("d-phase-3-6") || lower.contains("d-phase-3-16"),
        "docs/future/phase-3-backlog.md MUST cite D-PHASE-3-6 or D-PHASE-3-16 \
         by name so the ratification chain from .addl/phase-3 is traceable \
         from the tracked backlog doc (the public-facing companion)."
    );
    assert!(
        lower.contains("phase 4") || lower.contains("phase-4"),
        "docs/future/phase-3-backlog.md MUST name Phase 4 as the deferral \
         destination per D-PHASE-3-16; the IFF-clause states the destination \
         is realness-acceptable only when Phase 4 pre-R1 inherits the deferral."
    );
}
