//! Phase-3 G17-B `bench-wat-rebake` library entry — walks the SANDBOX
//! fixture root + regenerates committed `.wasm` bytes from each
//! `.wat` source via the workspace-locked exact-version `wat` crate
//! (`=1.248.0` per `[workspace.dependencies] wat` + r4-r1-wsa-9 recalibration).
//!
//! Loader strategy mirror: this binary is the **producer** that writes
//! the committed `.wasm` bytes the test-time **consumer** at
//! `crates/benten-eval/src/test_fixtures.rs::load_fixture` reads. The
//! consumer prefers the committed `.wasm` if present; fresh-checkout
//! fallback is `wat::parse_file` on the `.wat`. The producer + consumer
//! both link against the SAME exact-version `wat` crate — this is what
//! defends cross-platform fixture-CID stability (phase-3-backlog §6.2 +
//! r1-wsa-5).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};

/// Errors surfaced by the rebake walk.
#[derive(Debug)]
pub enum RebakeError {
    /// `tests/fixtures/sandbox/` directory missing or unreadable.
    FixtureRootMissing(PathBuf, std::io::Error),
    /// `.wat` source unreadable.
    WatRead(PathBuf, std::io::Error),
    /// `.wat` failed to parse / compile to wasm bytes.
    WatParse(PathBuf, wat::Error),
    /// `.wasm` write failure.
    WasmWrite(PathBuf, std::io::Error),
    /// directory walk failure.
    Walk(PathBuf, std::io::Error),
}

impl std::fmt::Display for RebakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FixtureRootMissing(p, e) => {
                write!(f, "fixture root {} missing or unreadable: {e}", p.display())
            }
            Self::WatRead(p, e) => write!(f, "read .wat {}: {e}", p.display()),
            Self::WatParse(p, e) => write!(f, "parse .wat {}: {e}", p.display()),
            Self::WasmWrite(p, e) => write!(f, "write .wasm {}: {e}", p.display()),
            Self::Walk(p, e) => write!(f, "walk {}: {e}", p.display()),
        }
    }
}

impl std::error::Error for RebakeError {}

/// Outcome of a single fixture rebake (or check).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureOutcome {
    /// Source `.wat` path.
    pub wat_path: PathBuf,
    /// Target `.wasm` path.
    pub wasm_path: PathBuf,
    /// `true` if `.wasm` bytes changed (or were freshly created) vs the
    /// previously-committed bytes; `false` if no change.
    pub changed: bool,
}

/// Walk `fixture_root` recursively (top + first-level subdirs like
/// `escape/`), compile each `.wat` to `.wasm` via `wat::parse_file`,
/// write the bytes to the sibling `.wasm` path. Returns the per-fixture
/// outcomes in walk order.
///
/// `dry_run = true` reports what WOULD change without writing — used by
/// the binary's `--check` mode to flag drift in CI-adjacent contexts.
pub fn rebake_all(fixture_root: &Path, dry_run: bool) -> Result<Vec<FixtureOutcome>, RebakeError> {
    let mut outcomes = Vec::new();
    let mut wat_paths = Vec::new();
    enumerate_wat(fixture_root, &mut wat_paths)?;
    wat_paths.sort();
    for wat_path in wat_paths {
        let wasm_path = wat_path.with_extension("wasm");
        let bytes =
            wat::parse_file(&wat_path).map_err(|e| RebakeError::WatParse(wat_path.clone(), e))?;
        let changed = match std::fs::read(&wasm_path) {
            Ok(existing) => existing != bytes,
            Err(_) => true,
        };
        if changed && !dry_run {
            std::fs::write(&wasm_path, &bytes)
                .map_err(|e| RebakeError::WasmWrite(wasm_path.clone(), e))?;
        }
        outcomes.push(FixtureOutcome {
            wat_path,
            wasm_path,
            changed,
        });
    }
    Ok(outcomes)
}

/// Recursive enumeration of `.wat` paths under `dir`. Restricted to the
/// fixture root tree (no symlink-follow) so a stray `.wat` elsewhere in
/// the workspace can't accidentally be picked up.
fn enumerate_wat(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), RebakeError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| RebakeError::FixtureRootMissing(dir.to_path_buf(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| RebakeError::Walk(dir.to_path_buf(), e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| RebakeError::Walk(path.clone(), e))?;
        if file_type.is_dir() {
            // First-level recursion only is sufficient for the current
            // tree shape (top + escape/). Deeper trees would need a
            // `walkdir` dep; keep this minimal.
            enumerate_wat(&path, out)?;
        } else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some("wat") {
            out.push(path);
        }
    }
    Ok(())
}

/// Resolve the canonical fixture root from a workspace root path.
pub fn fixture_root_from_workspace(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("crates")
        .join("benten-eval")
        .join("tests")
        .join("fixtures")
        .join("sandbox")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_outcome_equality_works() {
        let a = FixtureOutcome {
            wat_path: PathBuf::from("a.wat"),
            wasm_path: PathBuf::from("a.wasm"),
            changed: true,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn fixture_root_resolution_yields_expected_relative_shape() {
        let root = fixture_root_from_workspace(Path::new("/tmp/repo"));
        assert!(root.ends_with("crates/benten-eval/tests/fixtures/sandbox"));
    }

    #[test]
    fn enumerate_wat_picks_up_only_wat_extensions() {
        let tmp = std::env::temp_dir().join("bench-wat-rebake-test-enumerate");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.wat"), b"(module)").unwrap();
        std::fs::write(tmp.join("b.txt"), b"not wat").unwrap();
        std::fs::write(tmp.join("c.wasm"), b"\0asm").unwrap();
        std::fs::create_dir_all(tmp.join("sub")).unwrap();
        std::fs::write(tmp.join("sub").join("d.wat"), b"(module)").unwrap();
        let mut out = Vec::new();
        enumerate_wat(&tmp, &mut out).unwrap();
        out.sort();
        assert_eq!(out.len(), 2);
        assert!(out[0].ends_with("a.wat"));
        assert!(out[1].ends_with("d.wat"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
