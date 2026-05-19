//! TF-12 obligation (1) — `cargo-public-api` baseline empty-diff
//! post-FREEZE + napi-surfaces-non-broken (G-CORE-9 FREEZE wave).
//!
//! ADDL R3-C1 (Phase-4-Meta-Core; last R3 wave; freeze-time). TDD
//! red-phase. Tests-only — NO production source.
//!
//! ## §3.6g LITERAL discipline checklist (reproduced, not §-referenced)
//!
//!  1. Land-when = FREEZE. Every RED pin carries
//!     `#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]`.
//!  2. Campaign-tail landed-vs-RED split (§3.5n): R3-C1 ground-truthed
//!     `.github/workflows/cargo-public-api.yml` at ed03729a — it is
//!     **INFORMATIONAL ONLY** ("the workflow always exits 0";
//!     "Promotion to required is a Phase-9+ ... decision"). The FREEZE
//!     obligation (a regenerated+committed baseline whose post-freeze
//!     delta is a CI **FAILURE**, §1.A.FROZEN item 9) is therefore a
//!     freeze-DELIVERABLE NOT yet built → RED.
//!  3. SHAPE-not-SUBSTANCE (pim-18 / §3.6f): this is necessarily a
//!     structural backstop pin (see "Why structural" note). The
//!     would-FAIL signal is concrete: the real workflow YAML body + the
//!     real `docs/public-api/<crate>` baseline files + the
//!     freeze-marker. The pin FAILS if the gate stays informational, a
//!     baseline is missing/stale-at-freeze, or the freeze marker is
//!     absent. NOT a type-constructibility assertion.
//!  4. pim-2 sub-rule-4 (§3.6b): exercises the SPECIFIC item-9
//!     obligation (baseline regenerated+committed AT the freeze + gate
//!     promoted to deny-on-delta + napi non-broken under the chosen
//!     `pub`→`pub(crate)` visibility path), not an umbrella "we have a
//!     public-api workflow".
//!  5. §3.13: no shared process-scoped static — per-test locals only.
//!  6. §3.5j: compiles + MSRV-1.95 clippy AND `cargo +stable clippy`
//!     (scoped to benten-engine — never `--workspace`).
//!  7. §3.6e: introduces no stranded `#[ignore]` pin; THIS pin's named
//!     un-ignore destination IS G-CORE-9.
//!
//! ## Why a structural backstop pin is correct (pim-18 waiver, §4-A row)
//!
//! r2-test-landscape.md §4-A TF-12 row: "A baseline that is regenerated
//! AFTER an unintended delta 'passes' trivially. The pin asserts the
//! baseline is committed at the FREEZE and any post-freeze delta is a
//! CI FAIL — the test is the structural backstop, not a snapshot of
//! whatever-shipped." Running `cargo public-api` here is impossible/
//! wrong in a unit test (needs nightly rustdoc-json, network, the
//! whole workspace built); the faithful pin asserts the FREEZE CONTRACT
//! shape: (a) the gate is promoted from informational→deny-on-delta at
//! the freeze, (b) a committed baseline exists per crate, (c) a
//! freeze-marker pins the baseline to the G-CORE-9 commit. The actual
//! diff is the workflow's job at PR-time; this pin is the structural
//! guarantee the freeze did not skip it. Same reasoning class as
//! R3-B6's #838 seam-shape pin.
//!
//! Pin source: r2-test-landscape.md TF-12 obligation (1) + §2.A S9 +
//! plan §1.A.FROZEN item 9 + §4 CI additions (public-API drift gate).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The crates whose public API the §1.A.FROZEN item 9 baseline must
/// cover (the workflow's own per-crate loop, ground-truthed from
/// cargo-public-api.yml at ed03729a).
const FROZEN_BASELINE_CRATES: &[&str] = &[
    "benten-core",
    "benten-errors",
    "benten-graph",
    "benten-caps",
    "benten-ivm",
    "benten-eval",
    "benten-engine",
    "benten-dsl-compiler",
];

/// RED — un-ignore at G-CORE-9. The public-API drift gate is PROMOTED
/// from informational to deny-on-delta at the freeze (§1.A.FROZEN item
/// 9: "any post-freeze public-API delta is a CI failure"). Pre-freeze
/// the workflow self-documents as informational ("the workflow always
/// exits 0") so this FAILS now; post-freeze the gate must enforce.
/// Would-FAIL if the freeze leaves the gate informational (a silent
/// frozen-surface mutation would then pass CI — the exact §1.A.FROZEN
/// "structural backstop" the escape-valve relies on).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn public_api_gate_is_deny_on_delta_post_freeze() {
    let wf = workspace_root().join(".github/workflows/cargo-public-api.yml");
    let yaml = std::fs::read_to_string(&wf)
        .unwrap_or_else(|e| panic!("cargo-public-api.yml must exist at {}: {e}", wf.display()));
    // The freeze must remove the "always exits 0 / informational"
    // posture and enforce a non-zero exit on drift. We assert the
    // informational escape hatch is GONE and an enforcing marker is
    // present (the G-CORE-9 brief decides the exact mechanism — e.g.
    // dropping the trailing `|| true`, using `--deny=all`, or marking
    // the job a required check; the pin asserts the PROPERTY: drift
    // fails CI).
    let still_informational = yaml.contains("the workflow always exits 0")
        || yaml.contains("INFORMATIONAL ONLY")
        || yaml.contains("|| true");
    assert!(
        !still_informational,
        "TF-12 (1)/§1.A.FROZEN item 9: the public-API drift gate is \
         STILL informational post-freeze (found the always-exit-0 / \
         INFORMATIONAL ONLY / `|| true` escape hatch). A frozen-surface \
         mutation must FAIL CI — promote to deny-on-delta at G-CORE-9. \
         Workflow body:\n{yaml}"
    );
    assert!(
        yaml.contains("--deny") || yaml.contains("FROZEN") || yaml.contains("required"),
        "TF-12 (1): post-freeze the gate must carry an enforcing marker \
         (`--deny`, a FROZEN baseline reference, or required-check \
         promotion). Workflow body:\n{yaml}"
    );
}

/// RED — un-ignore at G-CORE-9. A committed `cargo-public-api` baseline
/// exists for EVERY frozen crate (regenerated AT the freeze) AND a
/// freeze-marker pins it to the G-CORE-9 commit so a later
/// "regenerate-after-an-unintended-delta" cannot trivially pass (the
/// §4-A trap). Would-FAIL if a baseline is missing or the freeze marker
/// is absent.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn frozen_public_api_baseline_committed_for_every_crate() {
    let dir = workspace_root().join("docs/public-api");
    let mut missing: Vec<String> = Vec::new();
    for c in FROZEN_BASELINE_CRATES {
        let txt = dir.join(format!("{c}.txt"));
        let json = dir.join(format!("{c}.json"));
        if !txt.exists() && !json.exists() {
            missing.push(format!("{c} (no .txt or .json baseline)"));
        }
    }
    assert!(
        missing.is_empty(),
        "TF-12 (1)/§1.A.FROZEN item 9: frozen public-API baseline absent \
         for crate(s): {missing:?} (must be regenerated + committed AT \
         G-CORE-9)."
    );
    // Freeze-marker: a tracked doc must pin the baseline to the freeze
    // commit so the §4-A "regenerate-after-delta passes trivially" trap
    // is closed. The G-CORE-9 brief produces `docs/V1-FROZEN-INTERFACE.md`
    // (the FROZEN-INTERFACE CONTRACT) — its presence + a reference to
    // the public-api baseline is the structural anti-trap marker.
    let frozen_doc = workspace_root().join("docs/V1-FROZEN-INTERFACE.md");
    assert!(
        frozen_doc.exists(),
        "TF-12 (1): the FROZEN-INTERFACE CONTRACT doc \
         (docs/V1-FROZEN-INTERFACE.md, §1.A.FROZEN) must exist at \
         G-CORE-9 and pin the cargo-public-api baseline to the freeze \
         commit (the §4-A anti-trap marker)."
    );
    let frozen_body = std::fs::read_to_string(&frozen_doc).unwrap();
    assert!(
        frozen_body.contains("cargo-public-api") || frozen_body.contains("public-api"),
        "TF-12 (1): the FROZEN-INTERFACE CONTRACT must explicitly state \
         the cargo-public-api baseline is the frozen v1 surface \
         (§1.A.FROZEN item 9)."
    );
}

/// RED — un-ignore at G-CORE-9. The chosen `pub`→`pub(crate)`
/// visibility path (§1.A.FROZEN item 1: `Engine::get_node` /
/// `put_node` / `get_node_label_only` / `resolve_subgraph_cid_for_test`
/// tighten + rename + drop `_for_test`) is applied AND the napi
/// surfaces are verified non-broken. Would-FAIL if a `_for_test`
/// suffixed method survives on the public surface OR the napi binding
/// still references the un-tightened symbols at the freeze.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]
fn napi_surfaces_non_broken_under_chosen_visibility_path() {
    let engine_src = workspace_root().join("crates/benten-engine/src");
    // Per §1.A.FROZEN item 1: `resolve_subgraph_cid_for_test` is
    // dropped (the `_for_test` cleanup). Post-freeze NO `pub fn
    // *_for_test` may remain on the engine public surface.
    let mut leaked: Vec<String> = Vec::new();
    for ent in std::fs::read_dir(&engine_src).unwrap() {
        let p = ent.unwrap().path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let body = std::fs::read_to_string(&p).unwrap();
        for (i, l) in body.lines().enumerate() {
            let t = l.trim_start();
            if (t.starts_with("pub fn ") || t.starts_with("pub async fn "))
                && t.contains("_for_test")
            {
                leaked.push(format!(
                    "{}:{} {}",
                    p.file_name().unwrap().to_string_lossy(),
                    i + 1,
                    t
                ));
            }
        }
    }
    assert!(
        leaked.is_empty(),
        "TF-12 (1)/§1.A.FROZEN item 1: `pub fn *_for_test` still on the \
         engine public surface post-freeze (the `_for_test` cleanup + \
         visibility tighten must be applied atomically at G-CORE-9; napi \
         surfaces verified non-broken): {leaked:?}"
    );
    // napi binding must not reference the dropped/tightened symbols.
    let napi_src = workspace_root().join("bindings/napi/src");
    if napi_src.exists() {
        let mut napi_refs: Vec<String> = Vec::new();
        visit_rs(&napi_src, &mut |path, body| {
            for (i, l) in body.lines().enumerate() {
                if l.contains("resolve_subgraph_cid_for_test") {
                    napi_refs.push(format!("{}:{}", path.display(), i + 1));
                }
            }
        });
        assert!(
            napi_refs.is_empty(),
            "TF-12 (1): napi binding still references \
             `resolve_subgraph_cid_for_test` (dropped at G-CORE-9 per \
             §1.A.FROZEN item 1) — napi surface broken: {napi_refs:?}"
        );
    }
}

fn visit_rs(dir: &std::path::Path, f: &mut dyn FnMut(&std::path::Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.is_dir() {
            visit_rs(&p, f);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(body) = std::fs::read_to_string(&p)
        {
            f(&p, &body);
        }
    }
}
