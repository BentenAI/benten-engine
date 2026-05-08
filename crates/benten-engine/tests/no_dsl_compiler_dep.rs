//! arch-N pin: `benten-engine` MUST NOT depend on `benten-dsl-compiler` in
//! Phase 2b — the compiler is a sibling-of-engine crate consumed by
//! `tools/benten-dev` directly. Per `r1-architect-reviewer.json` G12-B-scope
//! item (e): "Devserver consumes `benten-dsl-compiler` directly (not via
//! `benten-engine`); `benten-engine` does NOT take a `benten-dsl-compiler`
//! dep in 2b — keeps the dep edge optional."
//!
//! R6-R3 r6-r3-arch-3: previously TDD red-phase with `todo!()` bodies +
//! `#[ignore]`-gated. G12-B has merged + the dep-break invariant is real
//! at HEAD; the test now scans `Cargo.toml` + asserts the public surface
//! lacks `register_handler_from_str` so the load-bearing arch-1
//! invariant has CI defense.
//!
//! Owner: R5 G12-B (qa-r4-01 R3-followup); lifted to GREEN at R6-R3-FP.

#![allow(clippy::unwrap_used, clippy::expect_used)]

/// Scan `crates/benten-engine/Cargo.toml` for any entry matching
/// `benten-dsl-compiler`. The compiler is a sibling crate consumed by
/// `tools/benten-dev` directly; threading it through `benten-engine`
/// would break the arch-1 dep-break invariant.
///
/// Approach: read the manifest as a string + walk lines looking for any
/// `benten-dsl-compiler` mention in `[dependencies]` /
/// `[dev-dependencies]` / `[build-dependencies]` sections. The string
/// scan is sufficient (Cargo.toml's TOML grammar admits both inline
/// `benten-dsl-compiler = "x.y"` and table form `[dependencies.benten-dsl-compiler]`;
/// both contain the literal substring).
#[test]
fn benten_engine_does_not_depend_on_benten_dsl_compiler() {
    let manifest = include_str!("../Cargo.toml");

    // Walk lines; only flag lines that are NOT comments. The narrative
    // header may legitimately mention `benten-dsl-compiler` in prose;
    // those lines start with `#` so they don't count.
    let mut offending_lines: Vec<(usize, &str)> = Vec::new();
    for (idx, line) in manifest.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if line.contains("benten-dsl-compiler") {
            offending_lines.push((idx + 1, line));
        }
    }

    assert!(
        offending_lines.is_empty(),
        "arch-1 dep-break invariant: benten-engine MUST NOT depend on \
         benten-dsl-compiler in Phase 2b. Found {} offending non-comment \
         line(s) in Cargo.toml: {offending_lines:?}",
        offending_lines.len(),
    );
}

/// Per arch-reviewer: the optional `register_handler_from_str` API is
/// explicitly NOT shipped in 2b — keeps the cargo-public-api baseline
/// narrow. Asserted via a lib-source scan: the symbol must not be
/// declared with `pub fn register_handler_from_str` in `crates/benten-engine/src/`.
///
/// We use a source-tree scan (vs. cargo-public-api dump) because
/// (a) the source-tree scan runs in-process with zero infrastructure
/// dependencies, (b) the public-method invariant is structural at the
/// `pub fn` declaration site, and (c) cargo-public-api drift is caught
/// independently by the `cargo-public-api` CI workflow per
/// `.addl/phase-2b/00-implementation-plan.md` §3.1.
#[test]
fn benten_engine_register_handler_from_str_not_publicly_surfaced_in_phase_2b() {
    use std::path::PathBuf;

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");

    let mut offending: Vec<String> = Vec::new();
    walk_rs_files(&src_dir, &mut |path, contents| {
        for (idx, line) in contents.lines().enumerate() {
            // Match `pub fn register_handler_from_str` (with optional
            // whitespace + generics + parens) in any Rust source file.
            // The exact spelling is what cargo-public-api would surface
            // as a stable symbol.
            let trimmed = line.trim_start();
            if trimmed.starts_with("pub fn register_handler_from_str")
                || trimmed.starts_with("pub(crate) fn register_handler_from_str")
            {
                offending.push(format!("{}:{}: {}", path.display(), idx + 1, line.trim()));
            }
        }
    });

    assert!(
        offending.is_empty(),
        "arch-1 dep-break invariant: Engine MUST NOT publicly surface \
         `register_handler_from_str` in Phase 2b (it would imply a \
         benten-dsl-compiler dep edge). Found {} offending declaration \
         site(s):\n{}",
        offending.len(),
        offending.join("\n"),
    );
}

/// Recursively walk the `src/` tree visiting every `*.rs` file.
fn walk_rs_files(dir: &std::path::Path, visit: &mut dyn FnMut(&std::path::Path, &str)) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir({}): {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let ft = entry.file_type().expect("file_type");
        if ft.is_dir() {
            walk_rs_files(&path, visit);
        } else if ft.is_file() && path.extension().is_some_and(|e| e == "rs") {
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read_to_string({}): {e}", path.display()));
            visit(&path, &contents);
        }
    }
}
