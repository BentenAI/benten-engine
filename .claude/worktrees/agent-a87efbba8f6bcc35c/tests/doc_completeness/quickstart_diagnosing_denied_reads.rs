//! Phase 2a R4 qa-r4-7 / cov-5 + dx-r1-7: `docs/QUICKSTART.md` §Diagnosing
//! denied reads names the Option-C evaluator-path so readers understand why
//! denied reads surface as None (not as an error).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .join("docs/QUICKSTART.md")
}

#[test]
fn quickstart_diagnosing_denied_reads_section_mentions_evaluator_path() {
    let contents = fs::read_to_string(doc_path()).expect("docs/QUICKSTART.md must exist at docs/");

    assert!(
        contents.contains("Diagnosing denied reads"),
        "docs/QUICKSTART.md must carry a `Diagnosing denied reads` section (dx-r1-7)"
    );
    assert!(
        contents.contains("Option C") || contents.contains("option C"),
        "Diagnosing denied reads section must name the Option-C contract \
         (dx-r1-7): `Option C` string present in the file."
    );
    assert!(
        contents.contains("diagnoseRead") || contents.contains("diagnose_read"),
        "Diagnosing denied reads section must reference the `diagnoseRead` / \
         `diagnose_read` surface so operators know how to tell denial apart \
         from miss (dx-r1-7)."
    );
}
