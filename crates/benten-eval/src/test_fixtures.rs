//! Phase-3 G17-B SANDBOX `.wat`/`.wasm` fixture loader (phase-3-backlog
//! §6.2 + r1-wsa-5 + r4-r1-wsa-9).
//!
//! ## Loader contract
//!
//! `load_fixture(name)` resolves a fixture by stem-name (no extension)
//! relative to `crates/benten-eval/tests/fixtures/sandbox/`:
//!
//! 1. **Prefer the committed `.wasm` bytes** at
//!    `tests/fixtures/sandbox/<name>.wasm`. These are produced by
//!    `cargo bench-wat-rebake` (alias in `.cargo/config.toml` →
//!    `tools/bench-wat-rebake/`) using the workspace-locked exact-
//!    version `wat` crate (`=1.248.0` per `[workspace.dependencies] wat`).
//! 2. **Fall back to assembling the `.wat`** at
//!    `tests/fixtures/sandbox/<name>.wat` via `wat::parse_file`.
//!    This path covers the fresh-checkout-before-rebake case and the
//!    `escape/<name>` subdir whose .wasm bytes may not yet be committed
//!    (G17-B-onwards lands the .wasm fanout).
//!
//! ## Why prefer-committed-then-fallback
//!
//! - **Cross-platform CID stability** (phase-3-backlog §6.2 + r1-wsa-5):
//!   Two CI runners on different architectures (Linux x86_64 + macOS
//!   arm64) loading the SAME committed `.wasm` see byte-identical
//!   bytes, so any downstream CID is byte-stable. The fallback path
//!   (`wat::parse_file`) is exact-pinned to one wat version so a
//!   fresh-checkout fallback also produces stable bytes — but the
//!   committed path is the cheaper + more durable surface.
//! - **CI runtime cost** (phase-3-backlog §6.2 second motivator):
//!   parsing 14+ `.wat` sources every test run added ~5-30s of
//!   per-fixture cost; loading bytes from disk is sub-millisecond.
//! - **Determinism contract**: a `wat` minor bump that changes
//!   emitted bytes is caught by `tests/fixture_wasm_hashes_stable`
//!   (drift detector) BEFORE the committed bytes are touched.
//!
//! ## Subdirectory addressing
//!
//! Stem-names with a `/` (e.g. `escape/forged_cap_claim_section`)
//! resolve to subdirectories. This matches the on-disk layout where
//! G17-A1 ESC fixtures + G17-A2 host-fn fixtures live at
//! `tests/fixtures/sandbox/escape/`. Stems WITHOUT `/` (e.g.
//! `depth_nest_1`) resolve at the fixture root.
//!
//! ## Producer/consumer pairing
//!
//! - **Producer**: `tools/bench-wat-rebake/` writes the committed
//!   `.wasm` bytes by walking the same fixture root + invoking
//!   `wat::parse_file` from the same exact-version `wat` crate.
//! - **Consumer**: this module (`load_fixture`) reads the committed
//!   bytes — or, on cache miss, recompiles via the same crate.
//! - Both producer + consumer link the SAME exact-version `wat` so
//!   the bytes round-trip is closed under the workspace pin.

#![allow(
    clippy::missing_errors_doc,
    reason = "result-type errors documented inline"
)]

use std::path::{Path, PathBuf};

/// Errors surfaced by the fixture loader.
#[derive(Debug)]
pub enum FixtureError {
    /// Neither the committed `.wasm` nor the `.wat` source resolved.
    NotFound(PathBuf, PathBuf),
    /// Committed `.wasm` exists but is unreadable.
    WasmRead(PathBuf, std::io::Error),
    /// `.wat` source exists but is unreadable.
    WatRead(PathBuf, std::io::Error),
    /// `.wat` source failed to parse.
    WatParse(PathBuf, wat::Error),
    /// Committed `.wasm` bytes do not start with the WASM magic
    /// number `\0asm` — corrupted or misnamed.
    WasmInvalid(PathBuf),
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(wasm, wat) => write!(
                f,
                "fixture not found: neither {} nor {} resolved",
                wasm.display(),
                wat.display()
            ),
            Self::WasmRead(p, e) => write!(f, "read .wasm {}: {e}", p.display()),
            Self::WatRead(p, e) => write!(f, "read .wat {}: {e}", p.display()),
            Self::WatParse(p, e) => write!(f, "parse .wat {}: {e}", p.display()),
            Self::WasmInvalid(p) => write!(
                f,
                ".wasm bytes at {} do not start with WASM magic `\\0asm`",
                p.display()
            ),
        }
    }
}

impl std::error::Error for FixtureError {}

/// Canonical fixture root: `crates/benten-eval/tests/fixtures/sandbox/`.
///
/// `CARGO_MANIFEST_DIR` is set by cargo at compile time to the
/// `crates/benten-eval` dir, so this resolves correctly regardless of
/// `cwd` at test time.
pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sandbox")
}

/// Resolve a fixture by stem-name to its committed `.wasm` and source
/// `.wat` paths. The stem may contain `/` for subdir addressing
/// (`"escape/forged_cap_claim_section"` resolves under
/// `tests/fixtures/sandbox/escape/`).
pub fn fixture_paths(stem: &str) -> (PathBuf, PathBuf) {
    let root = fixture_root();
    let parts: Vec<&str> = stem.split('/').collect();
    let mut wasm = root.clone();
    let mut wat = root;
    for (idx, part) in parts.iter().enumerate() {
        if idx + 1 == parts.len() {
            wasm.push(format!("{part}.wasm"));
            wat.push(format!("{part}.wat"));
        } else {
            wasm.push(part);
            wat.push(part);
        }
    }
    (wasm, wat)
}

/// Load fixture bytes by stem-name. Prefers the committed `.wasm`;
/// falls back to compiling the `.wat` via `wat::parse_file`.
///
/// Returns the raw wasm bytes (`\0asm`-prefixed). Caller is responsible
/// for handing these to wasmtime / runtime / hashing logic.
pub fn load_fixture(stem: &str) -> Result<Vec<u8>, FixtureError> {
    let (wasm_path, wat_path) = fixture_paths(stem);
    if wasm_path.exists() {
        let bytes =
            std::fs::read(&wasm_path).map_err(|e| FixtureError::WasmRead(wasm_path.clone(), e))?;
        if bytes.len() < 4 || &bytes[..4] != b"\0asm" {
            return Err(FixtureError::WasmInvalid(wasm_path));
        }
        return Ok(bytes);
    }
    if wat_path.exists() {
        return load_via_wat(&wat_path);
    }
    Err(FixtureError::NotFound(wasm_path, wat_path))
}

/// Force the `.wat`-fallback path (skipping the committed `.wasm` even
/// if present). Test-only helper used by
/// `d26_wasm_runtime_loader_prefers_wasm_falls_back_to_wat` to verify
/// the fallback round-trips to valid WASM bytes without the move-aside-
/// + restore dance the brief sketched.
pub fn load_fixture_wat_only(stem: &str) -> Result<Vec<u8>, FixtureError> {
    let (wasm_path, wat_path) = fixture_paths(stem);
    if !wat_path.exists() {
        return Err(FixtureError::NotFound(wasm_path, wat_path));
    }
    load_via_wat(&wat_path)
}

fn load_via_wat(wat_path: &Path) -> Result<Vec<u8>, FixtureError> {
    let _read_check = std::fs::metadata(wat_path)
        .map_err(|e| FixtureError::WatRead(wat_path.to_path_buf(), e))?;
    wat::parse_file(wat_path).map_err(|e| FixtureError::WatParse(wat_path.to_path_buf(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_paths_resolves_root_stem() {
        let (wasm, wat) = fixture_paths("depth_nest_1");
        assert!(wasm.ends_with("tests/fixtures/sandbox/depth_nest_1.wasm"));
        assert!(wat.ends_with("tests/fixtures/sandbox/depth_nest_1.wat"));
    }

    #[test]
    fn fixture_paths_resolves_subdir_stem() {
        let (wasm, wat) = fixture_paths("escape/foo_bar");
        assert!(wasm.ends_with("tests/fixtures/sandbox/escape/foo_bar.wasm"));
        assert!(wat.ends_with("tests/fixtures/sandbox/escape/foo_bar.wat"));
    }

    #[test]
    fn load_fixture_prefers_committed_wasm_when_present() {
        // depth_nest_1.wasm is committed in-tree (Phase-2b G7-B).
        let bytes = load_fixture("depth_nest_1").expect("fixture must load");
        assert_eq!(&bytes[..4], b"\0asm", "WASM magic prefix");
    }

    #[test]
    fn load_fixture_wat_only_round_trips_to_wasm_magic() {
        let bytes = load_fixture_wat_only("depth_nest_1").expect("wat-only path must compile");
        assert_eq!(&bytes[..4], b"\0asm");
    }

    #[test]
    fn load_fixture_unknown_stem_yields_not_found() {
        match load_fixture("nonexistent_fixture_phase_3_g17b") {
            Err(FixtureError::NotFound(_, _)) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }
}
