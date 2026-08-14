//! The SANDBOX runtime constructs **exactly one** `wasmtime::Engine` per
//! process, and that is a load-bearing security property — not a performance
//! detail.
//!
//! RUSTSEC-2026-0222 ("Stores can mix up type indices between engines",
//! GHSA-hgjw-h833-99q9) has **no patched release in the 43.x line** we pin.
//! Its precondition is two or more live `Engine` instances whose `Store`s can
//! be confused with one another. `sandbox::instance::SHARED_ENGINE` is a
//! `OnceLock<Engine>` initialised once via `get_or_init`, so the precondition
//! cannot arise: there is no second engine to mix indices with.
//!
//! That reasoning is recorded as the `deny.toml` ignore rationale for
//! RUSTSEC-2026-0222 — which makes it a claim CI acts on. A claim CI acts on
//! must be one CI can falsify. Without this scan, someone adding a second
//! `Engine::new` re-opens the vulnerability while the ignore entry keeps
//! asserting it is unreachable, and nothing anywhere reports the change: the
//! advisory stays suppressed, silently and wrongly.
//!
//! **Falsification arm (proven by mutation, not assumed):** add any second
//! `Engine::new(` call site under `src/` and this test fails naming the file
//! and line. Delete the singleton entirely and it fails on the vacuity floor
//! rather than passing over an empty scan.
//!
//! If a second engine is ever genuinely wanted, this test failing is the
//! signal to **remove the `deny.toml` ignore and bump wasmtime to a patched
//! release** (>= 46.0.2, or >= 36.0.13 within the older line) — not to relax
//! the assertion.

use std::path::{Path, PathBuf};

/// Files scanned must be at least this many, or the scan is vacuous and the
/// pass means nothing. Guards the "someone moved the directory" failure mode
/// that makes an empty scan look like a clean one.
const MIN_FILES_SCANNED: usize = 10;

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn exactly_one_wasmtime_engine_construction_site() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src, &mut files);
    files.sort();

    assert!(
        files.len() >= MIN_FILES_SCANNED,
        "vacuity floor: scanned only {} files under {} (expected >= {}). \
         The scan found nothing to read, so a PASS here would be meaningless. \
         Fix the scan path before trusting this test.",
        files.len(),
        src.display(),
        MIN_FILES_SCANNED,
    );

    let mut sites: Vec<String> = Vec::new();
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip doc comments and ordinary comments: prose that MENTIONS
            // `Engine::new` (the panics doc on `shared_engine`, this header)
            // is not a construction site.
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains("Engine::new(") {
                sites.push(format!("{}:{}", file.display(), idx + 1));
            }
        }
    }

    assert_eq!(
        sites.len(),
        1,
        "expected EXACTLY ONE wasmtime Engine construction site (the \
         `SHARED_ENGINE` OnceLock in sandbox/instance.rs); found {}: {:#?}\n\n\
         A second engine re-opens RUSTSEC-2026-0222, whose deny.toml ignore is \
         justified *solely* by there being one engine. Either revert the new \
         construction site, or remove the ignore and bump wasmtime to a patched \
         release (>= 46.0.2).",
        sites.len(),
        sites,
    );

    assert!(
        sites[0].contains("sandbox"),
        "the single Engine construction site moved out of the sandbox module \
         (now at {}). The deny.toml rationale cites \
         `sandbox::instance::SHARED_ENGINE` by name; update both together.",
        sites[0],
    );
}
