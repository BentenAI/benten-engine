//! Phase 2b G10-A-browser must-pass — bundle size regression guard.
//!
//! Pin source: plan §3 G10-A-browser must-pass list + wasm-r1-7
//! (≤500KB gzipped browser-bundle cap) +
//! `.addl/phase-2b/00-implementation-plan.md` G10-A-browser row.
//!
//! ## Why this exists
//!
//! Above ~500KB gzipped, browser page-load latency spikes break the
//! "personal AI assistant cold-start <1s" Phase-3 commitment. The cap
//! is enforced by both:
//!   - this Rust-side integration test (drift-detector against the
//!     committed canonical bundle at `bindings/napi/dist/browser/`),
//!   - the `wasm-browser.yml` workflow (regression guard on PR — the
//!     workflow rebuilds the bundle from source and re-checks the cap
//!     so a stale committed artifact can't mask drift).
//!
//! ## Test gating shape
//!
//! The bundle artifact is produced by `wasm-browser.yml` and is NOT
//! committed to the source tree by default (build artifacts shouldn't
//! land in git history). The test is therefore SKIP-when-absent (it
//! returns `Ok(())` without asserting if the artifact directory is
//! missing) so that the local `cargo test -p benten-napi` path stays
//! green for engine developers without a wasm32 toolchain installed.
//!
//! When the artifact IS present (CI builds it as a workflow step
//! before invoking nextest, or a local developer ran the bundle
//! build), the test enforces:
//!   1. Gzipped bundle size ≤ 500KB.
//!   2. The browser dist directory does NOT contain a `*.node`
//!      napi binary (wasm-r1-6 — bundle conflation drift detector).
//!
//! The skip-on-absent shape is intentionally chosen over `#[ignore]`
//! so that CI without an explicit `--ignored` flag still runs the
//! drift detector when the artifact is present. The `wasm-browser.yml`
//! workflow runs `cargo nextest run -p benten-napi --test \
//! wasm32_unknown_unknown_bundle_size_under_threshold` after the
//! bundle build, so the assertion path executes there with no
//! `--ignored` ceremony.
//!
//! Owned by G10-A-browser per plan §3.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    // Diagnostics on the skip-when-absent path. The skip path is the
    // load-bearing UX for engine developers running cargo test
    // without a wasm32 toolchain — eprintln makes the skip visible
    // in test output without failing the run.
    clippy::print_stderr,
)]

use std::path::PathBuf;

/// Plan-pinned hard cap from wasm-r1-7. Tighter caps belong in
/// per-bundle drift-detection (`bundle-size.yml`); this is the
/// last-line defense.
const BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 500 * 1024;

/// Dist directory the `wasm-browser.yml` workflow writes into.
fn dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist/browser")
}

/// Canonical gzipped bundle path the workflow emits.
fn bundle_path() -> PathBuf {
    dist_dir().join("benten_engine.wasm.gz")
}

#[test]
fn wasm32_unknown_unknown_bundle_size_under_threshold() {
    let bundle = bundle_path();

    // Skip-when-absent: keeps the local `cargo test` path green for
    // engine developers without a wasm32 toolchain. CI's
    // `wasm-browser.yml` builds the bundle before invoking the test
    // binary, so the assertion path executes there.
    let bytes = match std::fs::read(&bundle) {
        Ok(b) => b,
        Err(_) => {
            eprintln!(
                "wasm-browser bundle not present at {} — skipping bundle-size \
                 check. Build via wasm-browser.yml or `wasm-pack build \
                 --target web` to populate dist/browser/.",
                bundle.display()
            );
            return;
        }
    };

    assert!(
        bytes.len() <= BROWSER_BUNDLE_MAX_BYTES_GZIPPED,
        "browser bundle is {} bytes gzipped, exceeds wasm-r1-7 cap of \
         {} bytes — investigate dep bloat / dead-code-elimination",
        bytes.len(),
        BROWSER_BUNDLE_MAX_BYTES_GZIPPED
    );
}

/// wasm-r1-6 sister assertion — bundle conflation drift detector.
///
/// Same skip-on-absent shape: a developer without the wasm32
/// toolchain doesn't need to materialize the dist dir to run
/// `cargo test -p benten-napi`.
#[test]
fn browser_bundle_excludes_napi_node_binary() {
    let dist = dist_dir();

    let dir = match std::fs::read_dir(&dist) {
        Ok(d) => d,
        Err(_) => {
            eprintln!(
                "wasm-browser dist directory not present at {} — skipping \
                 napi-binary exclusion check.",
                dist.display()
            );
            return;
        }
    };

    for entry in dir {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        assert!(
            !name_str.ends_with(".node"),
            "browser bundle dir must NOT contain a napi node binary \
             ({}); wasm-r1-6 forbids bundling the node target into \
             the browser distribution",
            name_str
        );
    }
}
