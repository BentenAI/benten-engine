//! R3-A RED-PHASE pin: `Engine::from_snapshot_blob` no longer uses a
//! tempdir hydration path (G13-D wave 3; plan §3 G13-D).
//!
//! Pin source: r2-test-landscape §2.1 G13-D row
//! `from_snapshot_blob_no_tempdir_in_path`; plan §3 G13-D.
//!
//! ## What G13-D does
//!
//! Phase-2b's `Engine::from_snapshot_blob(bytes)` hydrated the blob
//! into a tempdir (writable redb path) before returning the engine.
//! G13-D direct-wires `SnapshotBlobBackend` as a first-class
//! `GraphBackend` so the function returns
//! `EngineGeneric<SnapshotBlobBackend>` directly — no tempdir, no
//! filesystem touch, no copy.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-D wave 3 — drop tempdir hydration"]
fn from_snapshot_blob_no_tempdir_in_path() {
    // G13-D implementer wires this:
    //
    // Option A — runtime check (preferred):
    //   // Read /proc/<pid>/maps or use platform-specific mechanism to
    //   // count tempdir creations during `from_snapshot_blob`. If the
    //   // function does NOT create a tempdir, the count delta is 0.
    //
    // Option B — source-cite assertion:
    //   let src = std::fs::read_to_string("crates/benten-engine/src/engine_snapshot.rs").unwrap();
    //   // Drop any reference to `tempfile::tempdir` / `TempDir::new`
    //   // inside `from_snapshot_blob` impl. The Phase-2b path used
    //   // tempdir to write a redb file; G13-D direct-wires SnapshotBlob.
    //   for (i, line) in src.lines().enumerate() {
    //       let trimmed = line.trim_start();
    //       if trimmed.starts_with("//") { continue; }
    //       assert!(!(trimmed.contains("tempdir()") || trimmed.contains("TempDir::new")),
    //           "engine_snapshot.rs:{} retains tempdir hydration after G13-D direct-wire: {}",
    //           i + 1, line);
    //   }
    //
    // OBSERVABLE consequence: opening a snapshot-blob engine does NOT
    // touch the filesystem. Defends against the regression where
    // G13-D landed the GraphBackend impl but forgot to update
    // `from_snapshot_blob` to use it directly.
    unimplemented!("G13-D wires no-tempdir source-cite assertion for from_snapshot_blob");
}
