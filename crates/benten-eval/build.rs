//! Phase-3 G17-B build script (phase-3-backlog §6.2 + r1-wsa-5 +
//! r4-r1-wsa-9).
//!
//! ## Role
//!
//! Emits `cargo:rerun-if-changed=tests/fixtures/sandbox/...` for every
//! `.wat` source file under the sandbox fixture root. When a `.wat`
//! source is edited (intentionally or accidentally), this triggers
//! recompilation of the affected test binaries — which then run the
//! `tests/fixture_wasm_hashes_stable` drift detector + the
//! `tests/d26_wasm_present` paired-bytes assertion. Both fire if the
//! committed `.wasm` is now stale.
//!
//! ## Why not regenerate `.wasm` here?
//!
//! Two reasons:
//!
//! 1. **Source-tree side-effects from `build.rs` are anti-pattern.** A
//!    build script writing into the source tree would surprise editors
//!    + CI cache layers that assume `cargo build` is read-only on
//!    `tests/`. The committed `.wasm` is a CHECKED-IN ARTIFACT — it's
//!    intended to round-trip via the `cargo bench-wat-rebake`
//!    regenerator binary (`tools/bench-wat-rebake/`), invoked
//!    explicitly by humans + verified by the drift detector.
//! 2. **wasm32-target builds DO compile this crate** (the wasm32 cuts
//!    SANDBOX itself but still compiles `benten-eval`'s non-sandbox
//!    surface). The wasm32 build CANNOT link `wat` (workspace
//!    [workspace.dependencies] entry, but the rebake binary owns the
//!    only direct dep). The build.rs early-exits on wasm32 to avoid
//!    pulling `wat` into the build-script's dep graph.
//!
//! ## Cross-platform CID stability
//!
//! Defended by:
//!   - Workspace `wat = "=1.248.0"` exact-version pin (`Cargo.toml:309`)
//!   - Committed `.wasm` bytes (this file's rerun-if-changed triggers
//!     drift detection if the bytes go stale)
//!   - `tests/fixture_wasm_hashes_stable` (drift detector, BLAKE3 pin
//!     per fixture)
//!   - `tests/d26_wasm_present::d26_cross_platform_fixture_cid_stable`
//!     (loader-level CID-stability + exact-version pin assertion)

use std::path::{Path, PathBuf};

fn main() {
    // Per the module docstring: skip on wasm32 to avoid build-time
    // entanglement with the SANDBOX fixture surface that's already
    // cfg-cut from the wasm32 lib build. CARGO_CFG_TARGET_ARCH is set
    // by cargo for the COMPILATION TARGET (host vs cross-target).
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target == "wasm32" {
        return;
    }

    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let fixture_root = PathBuf::from(&manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("sandbox");

    if !fixture_root.exists() {
        // No fixture root yet — nothing to track. (Defensive: would
        // happen pre-G7-B if someone reorganised the tree.)
        return;
    }

    // Track the directory itself so file additions / removals retrigger
    // the build.
    println!("cargo:rerun-if-changed={}", fixture_root.display());

    // Walk the fixture root + first-level subdirs (escape/ etc) and
    // emit rerun-if-changed for every .wat source. A future deeper
    // tree would need recursion; the current layout is two levels.
    walk_and_emit(&fixture_root);
    let escape_dir = fixture_root.join("escape");
    if escape_dir.is_dir() {
        println!("cargo:rerun-if-changed={}", escape_dir.display());
        walk_and_emit(&escape_dir);
    }

    // Also retrigger if Cargo.toml changes (workspace `wat` pin bump
    // would invalidate the committed bytes).
    println!("cargo:rerun-if-changed=Cargo.toml");
}

fn walk_and_emit(dir: &Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && (ext == "wat" || ext == "wasm")
        {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
