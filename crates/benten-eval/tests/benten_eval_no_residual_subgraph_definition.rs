//! G12-C-cont (Phase 2b R6 A1 closure): assert `benten-eval` has NO
//! `pub struct Subgraph` definition after the type migration to `benten-core`
//! — only a re-export.
//!
//! Per plan §3.2 G12-C: "delete eval-side `Subgraph` + the two `todo!()`
//! stubs at `:859,867`; re-export from `benten-core`."
//!
//! Static-source scan: walks `crates/benten-eval/src/**/*.rs`, asserts no
//! line matches the patterns:
//!   - `pub struct Subgraph {`           (definition; should be 0)
//!   - `pub struct SubgraphBuilder {`    (definition; should be 0)
//!   - `pub enum PrimitiveKind {`        (definition; should be 0)
//!   - `pub struct OperationNode {`      (definition; should be 0)
//!   - `pub struct NodeHandle(`          (definition; should be 0)
//!
//! And asserts the re-export pattern is present in `lib.rs`:
//!   - `pub use benten_core::{... Subgraph ...}` covering all relocated types.
//!
//! TDD red-phase un-ignored under G12-C-cont.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

const FORBIDDEN_DEFS: &[&str] = &[
    "pub struct Subgraph {",
    "pub struct Subgraph<",
    "pub struct SubgraphBuilder {",
    "pub struct SubgraphBuilder<",
    "pub enum PrimitiveKind {",
    "pub enum PrimitiveKind<",
    "pub struct OperationNode {",
    "pub struct OperationNode<",
    "pub struct NodeHandle(",
];

fn walk_eval_src(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_eval_src(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn eval_src_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR points at the test crate's root (benten-eval).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    Path::new(&manifest_dir).join("src")
}

#[test]
fn benten_eval_does_not_define_subgraph_or_relocated_companion_types() {
    let mut files = Vec::new();
    walk_eval_src(&eval_src_root(), &mut files);
    assert!(
        !files.is_empty(),
        "must find at least one .rs file under benten-eval/src"
    );

    let mut violations = Vec::new();
    for path in &files {
        let content = fs::read_to_string(path).expect("read file");
        for (lineno, line) in content.lines().enumerate() {
            // Skip lines inside `//` comments (single-line) — `///` doc
            // comments are also fine because we only catch raw struct/enum
            // declarations, not docstring mentions.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for forbidden in FORBIDDEN_DEFS {
                if trimmed.starts_with(forbidden) {
                    violations.push(format!(
                        "{}:{}: forbidden definition `{}` (must live in benten-core post-G12-C-cont)",
                        path.display(),
                        lineno + 1,
                        forbidden
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "benten-eval must NOT redefine relocated types; found:\n{}",
        violations.join("\n")
    );
}

#[test]
fn benten_eval_re_exports_subgraph_companions_from_benten_core() {
    let lib = fs::read_to_string(eval_src_root().join("lib.rs")).expect("read lib.rs");
    // Assert the relocated types are re-exported by name. The exact `pub use`
    // syntax can vary (single line vs. multi-line braces); we look for each
    // type's appearance in a `pub use benten_core::{...}` block.
    let needed_re_exports = [
        "Subgraph",
        "SubgraphBuilder",
        "PrimitiveKind",
        "OperationNode",
        "NodeHandle",
    ];
    let pub_use_block_present = lib.contains("pub use benten_core::");
    assert!(
        pub_use_block_present,
        "benten-eval/src/lib.rs must contain `pub use benten_core::` block"
    );
    for name in needed_re_exports {
        assert!(
            lib.contains(name),
            "benten-eval/src/lib.rs must re-export `{name}` from benten-core"
        );
    }
}

#[test]
fn benten_eval_subgraph_lib_no_residual_todo_stubs_in_subgraph_impls() {
    // Per plan §3.2 G12-C explicit cleanup: "delete eval-side Subgraph + the
    // two `todo!()` stubs". Pin: no `todo!()` calls inside any `impl
    // SubgraphExt for Subgraph` or `impl SubgraphBuilderExt for
    // SubgraphBuilder` block.
    let lib = fs::read_to_string(eval_src_root().join("lib.rs")).expect("read lib.rs");
    let ext =
        fs::read_to_string(eval_src_root().join("subgraph_ext.rs")).expect("read subgraph_ext.rs");
    for (name, content) in [("lib.rs", &lib), ("subgraph_ext.rs", &ext)] {
        // Walk impl blocks for the relocated types and look for `todo!()`.
        let mut in_subgraph_impl = false;
        let mut depth: i32 = 0;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("impl ")
                && (trimmed.contains("SubgraphExt for Subgraph")
                    || trimmed.contains("SubgraphBuilderExt for SubgraphBuilder")
                    || trimmed.contains("NodeHandleExt for NodeHandle"))
            {
                in_subgraph_impl = true;
                depth = 0;
            }
            if in_subgraph_impl {
                depth += i32::try_from(trimmed.matches('{').count()).unwrap_or(0);
                depth -= i32::try_from(trimmed.matches('}').count()).unwrap_or(0);
                assert!(
                    !trimmed.contains("todo!()"),
                    "{name}: `todo!()` residue in Subgraph extension impl: {trimmed}"
                );
                if depth <= 0 && trimmed.starts_with('}') {
                    in_subgraph_impl = false;
                }
            }
        }
    }
}
