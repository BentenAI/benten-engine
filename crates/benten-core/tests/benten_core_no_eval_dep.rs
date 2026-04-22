//! R3 unit test for phil-r1-3 / §9.14 arch-1 dep-break CI gate (companion #2):
//! `benten-core` MUST NOT depend on `benten-eval`.
//!
//! Reads `crates/benten-core/Cargo.toml` at test time and fails loudly if
//! `benten-eval` shows up in any dep section. Runs as part of
//! `.github/workflows/arch-1-dep-break.yml`.
//!
//! Owner: rust-test-writer-unit (R2 landscape §8.1 + §9.14).

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn cargo_toml_path() -> PathBuf {
    // CARGO_MANIFEST_DIR points at this crate's root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn benten_core_no_eval_dep() {
    let path = cargo_toml_path();
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    // A dependency line references benten-eval in [dependencies] /
    // [dev-dependencies] / [build-dependencies] via either the
    // `benten-eval = ...` form or the `[dependencies.benten-eval]` table form.
    // A false-positive guard: we only reject lines that start with
    // `benten-eval` at a line start (stripping leading whitespace) or the
    // explicit table header.
    let offenders: Vec<&str> = src
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            t.starts_with("benten-eval ")
                || t.starts_with("benten-eval=")
                || t.starts_with("[dependencies.benten-eval]")
                || t.starts_with("[dev-dependencies.benten-eval]")
                || t.starts_with("[build-dependencies.benten-eval]")
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "arch-1 dep-break regressed: benten-core must not depend on benten-eval \
         (phil-r1-3). Offending lines in {}: {:?}",
        path.display(),
        offenders
    );
}
