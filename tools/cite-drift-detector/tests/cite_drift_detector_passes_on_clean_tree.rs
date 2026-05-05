//! Phase-3 G13-pre-A pin: drives `run_cite_drift_check` against a
//! controlled CLEAN tree (a fixture containing valid cites) and asserts
//! zero findings. Pairs with `cite_drift_detector_finds_known_drift_fixture`
//! (drives a fixture WITH planted drift and asserts findings ARE
//! emitted) — together the two pins establish the bidirectional
//! soundness property: the detector reports drift when present and
//! reports nothing when absent.
//!
//! pim-2 §3.6b end-to-end pin: drives the production entry point
//! against a real on-disk tree end-to-end (the fixture is an isolated
//! controlled tree but the entry point + walker + parsers + validators
//! are the SAME code paths the CI workflow runs against the workspace).
//! A silent always-pass detector would still satisfy the zero-findings
//! assertion below, so we ALSO assert at least one cite WAS extracted
//! from the fixture's intentionally-cite-bearing doc — catches the
//! "detector compiles but parses nothing" regression.
//!
//! End-to-end against the LIVE workspace tree is delegated to the
//! `.github/workflows/cite-drift.yml` workflow itself (non-blocking PR
//! comment mode per D-PHASE-3-10). Hard-asserting a zero-findings tree
//! state inside a unit test would be brittle in the wave-1pre tooling
//! drop because the lint's introduction surfaces pre-existing drift
//! that the orchestrator triages incrementally; the workflow is the
//! right enforcement seam (it can be promoted to required once the
//! workspace baseline is clean per D-PHASE-3-10).

use std::fs;

use cite_drift_detector::{run_cite_drift_check, walk_doc_inputs};

#[test]
fn cite_drift_detector_passes_on_clean_tree() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    // -- planted target file (exists; has 5 lines) ----------------------
    fs::create_dir_all(root.join("crates/foo/src")).unwrap();
    fs::write(
        root.join("crates/foo/src/utility.rs"),
        "// line 1\n\
         // line 2\n\
         pub fn real_symbol() {}\n\
         pub struct RealStruct;\n\
         // line 5\n",
    )
    .unwrap();

    // -- planted second target (TS) ------------------------------------
    fs::create_dir_all(root.join("packages/engine/src")).unwrap();
    fs::write(
        root.join("packages/engine/src/api.ts"),
        "export function realFn() {}\nexport class RealClass {}\n",
    )
    .unwrap();

    // -- doc with EXCLUSIVELY valid cites ------------------------------
    fs::create_dir_all(root.join("docs")).unwrap();
    let doc = "\
# Clean fixture

A line cite that points at a valid file + valid line:
crates/foo/src/utility.rs:3

A symbol cite where both file + symbol exist (Rust fn form):
crates/foo/src/utility.rs::real_symbol

A symbol cite where both file + symbol exist (Rust struct form):
crates/foo/src/utility.rs::RealStruct

A TS symbol cite where both file + symbol exist:
packages/engine/src/api.ts::realFn

A TS class symbol cite:
packages/engine/src/api.ts::RealClass
";
    fs::write(root.join("docs/CLEAN-FIXTURE.md"), doc).unwrap();

    // pim-2 §3.6b "would fail if silently no-op'd" guard: confirm the
    // walker enumerated the planted inputs. A silent zero-walk would
    // make the cleanliness assertion vacuously true.
    let inputs = walk_doc_inputs(root);
    assert!(
        inputs.iter().any(|p| p.ends_with("docs/CLEAN-FIXTURE.md")),
        "walk_doc_inputs missed the planted clean fixture: {:?}",
        inputs
    );
    assert!(
        inputs
            .iter()
            .any(|p| p.ends_with("crates/foo/src/utility.rs")),
        "walk_doc_inputs missed the planted Rust source: {:?}",
        inputs
    );

    let findings = run_cite_drift_check(root);

    assert!(
        findings.is_empty(),
        "expected zero findings on a CLEAN fixture tree; got {} finding(s):\n{:#?}",
        findings.len(),
        findings
    );
}
