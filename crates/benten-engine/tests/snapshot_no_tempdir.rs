//! G13-D wave-3 (GREEN-PHASE): `Engine::from_snapshot_blob` no longer
//! uses a tempdir hydration path.
//!
//! Pin source: r2-test-landscape §2.1 G13-D row
//! `from_snapshot_blob_no_tempdir_in_path`; plan §3 G13-D.
//!
//! ## What G13-D delivers
//!
//! Phase-2b's `Engine::from_snapshot_blob(bytes)` hydrated the blob
//! into a tempdir-resident redb file (writable on-disk path) before
//! returning the engine. G13-D drops the tempdir — hydration goes into
//! [`benten_graph::RedbBackend::open_in_memory`] so the function never
//! touches the filesystem. The full direct-wire to
//! `EngineGeneric<SnapshotBlobBackend>` (no in-memory redb hop) is a
//! follow-up tracked at `docs/future/phase-3-backlog.md §1.2-followup`.
//!
//! Pin shape: source-cite assertion (Option B from the R3-A pin
//! template). Reads `crates/benten-engine/src/engine_snapshot.rs` and
//! verifies that the body of `from_snapshot_blob` does NOT call
//! `tempfile::tempdir()` / `TempDir::new`. Defends against the
//! regression where G13-D landed the GraphBackend impl but forgot to
//! retire `from_snapshot_blob`'s on-disk hydration path.
//!
//! Companion runtime witness: the load-bearing integration tests in
//! `tests/integration/snapshot_blob_round_trip.rs` exercise the
//! end-to-end export → from_snapshot_blob → re-export round-trip + the
//! delete-via-dispatch read-only contract; they pass with the in-memory
//! redb hydration introduced by G13-D, which is the runtime witness
//! that the no-tempdir change preserves observable engine behavior.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

/// Read the engine_snapshot.rs source and assert that the
/// `from_snapshot_blob` function body contains no tempdir construction
/// site. Defends against silent regression to the Phase-2b on-disk
/// hydration shape.
#[test]
fn from_snapshot_blob_no_tempdir_in_path() {
    // Resolve `crates/benten-engine/src/engine_snapshot.rs` from the
    // test target's `CARGO_MANIFEST_DIR` (= `crates/benten-engine`).
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("src").join("engine_snapshot.rs");
    let src = std::fs::read_to_string(&path).expect("engine_snapshot.rs must be readable");

    // Walk the source line by line. We only care about substantive
    // (non-comment) lines that fall inside the `fn from_snapshot_blob`
    // body. The function ends at the matching closing brace; we use a
    // brace-depth counter to track when the function body terminates.
    let mut in_fn_body = false;
    let mut depth: i32 = 0;
    let mut violations: Vec<(usize, String)> = Vec::new();

    for (idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        // Strip line comments before scanning so the doc-comment block
        // above the function (which intentionally narrates the retired
        // tempdir shape) does not produce false positives.
        let code_only = match trimmed.find("//") {
            Some(pos) => &trimmed[..pos],
            None => trimmed,
        };

        if !in_fn_body && code_only.contains("pub fn from_snapshot_blob(") {
            in_fn_body = true;
            depth = code_only.matches('{').count() as i32 - code_only.matches('}').count() as i32;
            continue;
        }

        if in_fn_body {
            depth += code_only.matches('{').count() as i32;
            depth -= code_only.matches('}').count() as i32;

            // Forbidden tempdir construction sites — the Phase-2b path
            // used `tempfile::tempdir()` / `TempDir::new()` to spin a
            // writable on-disk redb file. G13-D direct-wires the
            // hydration to `RedbBackend::open_in_memory()` so neither
            // call should appear inside the function body.
            if code_only.contains("tempdir()") || code_only.contains("TempDir::new") {
                violations.push((idx + 1, line.to_string()));
            }

            if depth <= 0 {
                in_fn_body = false;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "engine_snapshot.rs::from_snapshot_blob retains tempdir hydration \
         after G13-D direct-wire — Phase-2b path must be retired in favor \
         of RedbBackend::open_in_memory(). Violations: {violations:?}"
    );
}
