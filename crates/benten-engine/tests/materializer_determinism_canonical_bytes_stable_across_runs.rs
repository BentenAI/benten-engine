//! R4-FP-3 RED-PHASE pin: CI-side materializer determinism gate
//! (canonical bytes stable across runs).
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.13 row 2.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - mat-r1-3: materializer output MUST be deterministic across runs;
//!   the per-crate test
//!   `crates/benten-platform-foundation/tests/materializer_canonical_bytes_determinism_across_runs.rs`
//!   shipped at R3 Family E; the CI-side cross-run gate composes with
//!   it (different test runner / different worker / repeated invocation
//!   produces identical canonical bytes).
//!
//! ## What this pin asserts
//!
//! Two arms (defense in depth):
//!
//! 1. The companion per-crate test
//!    `materializer_canonical_bytes_determinism_across_runs.rs` exists
//!    at `crates/benten-platform-foundation/tests/`.
//! 2. That companion test is REGISTERED ON THE REQUIRED LANE — i.e. its
//!    target stem appears in `.github/frozen-bytes-corpus.txt`, the corpus
//!    `frozen-bytes.yml` runs as the required `frozen-bytes corpus
//!    (v1-beta wire freeze)` context.
//!
//! ## F-073 — why arm 2 changed shape
//!
//! Arm 2 used to scan `.github/workflows/*.yml` for the word "materializer",
//! against a `G26-B wave-10` destination that shipped two phases ago. NO
//! workflow has ever contained that word, so the arm was never true — and it
//! was `#[ignore]`d, so nothing said so. The gap it was hiding is real: a
//! materializer canonical-bytes / canonical-CID regression rode a green board,
//! because the companion test's only lane (`ci.yml`'s `build+test`) is not a
//! required context (see `frozen-bytes.yml`'s own header on that gap).
//!
//! The required-lane registration surface at HEAD is the corpus FILE, not a
//! workflow body, so that is what arm 2 now reads.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(&manifest_dir)
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

/// FAILS ON THIS ONE-LINE MUTATION: delete the
/// `materializer_canonical_bytes_determinism_across_runs` line from
/// `.github/frozen-bytes-corpus.txt` — arm 2 fires. (The corpus floor in
/// `frozen-bytes.yml` fires too; both are deliberate, and neither fired
/// before F-073 because this pin was ignored and the line did not exist.)
#[test]
fn materializer_determinism_canonical_bytes_stable_across_runs() {
    let root = workspace_root();

    // ARM 1: per-crate determinism test exists.
    let per_crate_test =
        root.join("crates/benten-platform-foundation/tests/materializer_canonical_bytes_determinism_across_runs.rs");
    assert!(
        per_crate_test.is_file(),
        "Per-crate materializer determinism test MUST exist at {} (shipped at R3 Family E \
         per r2-test-landscape §2.5 G23-B). Without it the CI gate has nothing to gate.",
        per_crate_test.display()
    );

    // ARM 2: the companion test is registered on the REQUIRED lane.
    //
    // Substring matching is not enough here: a `#` comment mentioning the stem
    // would satisfy it while registering nothing. Match a whole non-comment
    // line, which is exactly the shape `frozen-bytes.yml` itself selects on
    // (`grep -vE '^[[:space:]]*(#|$)'`).
    let corpus_path = root.join(".github/frozen-bytes-corpus.txt");
    let corpus = std::fs::read_to_string(&corpus_path).unwrap_or_else(|e| {
        panic!(
            "frozen-bytes corpus MUST exist at {} ({e})",
            corpus_path.display()
        )
    });

    let registered = corpus
        .lines()
        .map(str::trim)
        .any(|line| line == "materializer_canonical_bytes_determinism_across_runs");

    assert!(
        registered,
        "`materializer_canonical_bytes_determinism_across_runs` MUST be listed in \
         {} so it runs on the REQUIRED `frozen-bytes corpus (v1-beta wire freeze)` \
         context. Its only other lane is `ci.yml`'s `build+test` job, which is NOT a \
         required context — a materializer canonical-bytes or canonical-CID regression \
         would ride a green board. That is the F-073 gap this arm exists to catch.",
        corpus_path.display()
    );
}
