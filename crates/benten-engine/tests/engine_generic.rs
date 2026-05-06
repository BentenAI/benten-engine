//! G13-B GREEN-PHASE pins for `EngineGeneric<B>` cascade (Phase-3 R5 wave-2).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + plan §3 G13-B
//! must-pass column):
//!
//! - `crates/benten-engine/tests/engine_generic.rs::engine_generic_compiles_with_redb_default` — plan §3 G13-B
//! - `crates/benten-engine/tests/engine_generic.rs::engine_generic_cascade_no_inherent_redb_references_outside_default_alias` — D-PHASE-3-1
//!
//! ## What G13-B introduces
//!
//! `crates/benten-engine/src/engine.rs` introduces:
//!
//! ```text
//! pub struct EngineGeneric<B: GraphBackend> { ... }
//! pub type Engine = EngineGeneric<benten_graph::RedbBackend>;
//! ```
//!
//! The default alias `Engine = EngineGeneric<RedbBackend>` preserves
//! API stability. The browser-target binding (G13-C wave-3) will ship
//! `Engine = EngineGeneric<BrowserBackend>` per cargo feature
//! `browser-backend`; until G13-C lands, the alias is unconditionally
//! the redb specialization.

#![allow(clippy::unwrap_used, clippy::used_underscore_items)]

use benten_engine::{Engine, EngineGeneric};
use benten_graph::RedbBackend;

#[test]
fn engine_generic_compiles_with_redb_default() {
    // Compile-time alias-equality witness: `Engine` is a type alias for
    // `EngineGeneric<RedbBackend>`. Asserting equality through a
    // function that requires the resolved alias as both an
    // `EngineGeneric<RedbBackend>` AND an `Engine` proves the two are
    // literally the same type.
    fn _alias_equality(_: &EngineGeneric<RedbBackend>, _: &Engine) {}
    fn _alias_equality_reversed(a: &Engine, b: &EngineGeneric<RedbBackend>) {
        _alias_equality(a, b);
    }

    // Runtime check: open via the default alias compiles + runs
    // unchanged after the cascade lands. Defends against G13-B
    // accidentally breaking API stability for existing callers
    // (napi binding, integration tests).
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("g13_b_alias.redb");
    let engine: Engine = Engine::open(&db_path).unwrap();

    // Use the engine through the alias to exercise method-resolution
    // on the resolved-alias `impl Engine` block + on the generic
    // `impl<B: GraphBackend> EngineGeneric<B>` block (e.g. accessors
    // like `is_read_only_snapshot`).
    assert!(!engine.is_read_only_snapshot());
    drop(engine);
}

#[test]
fn engine_generic_cascade_no_inherent_redb_references_outside_default_alias() {
    // D-PHASE-3-1 RESOLVED + arch-r1-2 BLOCKER closure pin.
    //
    // After G13-B cascades the generic parameter, NO impl block on
    // `EngineGeneric<B>` should reference `RedbBackend` directly —
    // every method that needs a backend operation goes through the
    // `B: GraphBackend` bound.
    //
    // ## Allowed sites (the only legitimate `RedbBackend` references
    // in `engine.rs`)
    //
    // 1. The `pub type Engine = EngineGeneric<benten_graph::RedbBackend>;`
    //    line — the default alias.
    // 2. Lines INSIDE `impl Engine { ... }` blocks (the resolved-alias
    //    specialized-impl side — `Engine::open`, `Engine::builder`,
    //    `audit_sequence`, etc.). These are convenience constructors and
    //    methods that legitimately know they want `RedbBackend`.
    // 3. Doc-comment / module-comment lines that NARRATIVELY reference
    //    `RedbBackend` to explain the design contract — these are not
    //    code references and don't violate the cascade.
    //
    // ## Forbidden site
    //
    // `RedbBackend` substring on a code line inside an
    // `impl<B: GraphBackend> EngineGeneric<B> { ... }` block — that
    // would re-introduce the inherent-redb coupling the generic-cascade
    // closes. The pin tracks block context to distinguish.

    // Use CARGO_MANIFEST_DIR so the test runs from any cwd (cargo test
    // sometimes runs from the crate dir, sometimes from the workspace
    // root, depending on how it's invoked).
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let engine_rs = std::path::Path::new(manifest_dir).join("src/engine.rs");
    let src = std::fs::read_to_string(&engine_rs)
        .unwrap_or_else(|e| panic!("engine.rs must exist at {}: {e}", engine_rs.display()));

    // Block-context tracker: per line, is the current source position
    // inside an `impl Engine { ... }` (allowed) vs an
    // `impl<B...> EngineGeneric<B> { ... }` (forbidden) block?
    //
    // Simple state machine: brace-depth + a flag for "current top-level
    // block is the resolved-alias `impl Engine`". The engine.rs file
    // alternates these blocks with a clear boundary; the tracker
    // resets at brace-depth-0 transitions.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum BlockKind {
        ImplEngineAliased,        // `impl Engine { ... }` — RedbBackend allowed
        ImplGenericEngineGeneric, // `impl<B: ...> EngineGeneric<B> { ... }` — RedbBackend forbidden
        Other, // module top level, struct decl, free fn, doc comment continuation, etc.
    }

    let mut block: BlockKind = BlockKind::Other;
    let mut depth: i32 = 0;
    let mut violations: Vec<(usize, String, BlockKind)> = Vec::new();

    for (i, line) in src.lines().enumerate() {
        let lineno = i + 1;
        let trimmed = line.trim_start();

        // Detect block start when at depth 0 (only top-level impls
        // matter — nested impls are inside a method body, which the
        // depth tracker handles).
        if depth == 0 {
            if trimmed.starts_with("impl Engine {")
                || trimmed.starts_with("impl Engine ")
                || trimmed == "impl Engine"
            {
                block = BlockKind::ImplEngineAliased;
            } else if trimmed.starts_with("impl<")
                && line.contains("EngineGeneric<")
                && !line.contains("std::fmt::Debug")
            {
                // `impl<B: GraphBackend> EngineGeneric<B> { ... }`
                // (the Debug impl is on the generic struct shape but
                // doesn't accept method bodies that touch RedbBackend
                // operations — exclude defensively in case the
                // detection logic confuses it).
                block = BlockKind::ImplGenericEngineGeneric;
            } else if trimmed.starts_with("impl") && !line.contains("EngineGeneric") {
                // Other impl block (e.g. `impl EngineInner`,
                // `impl SubgraphCache`, the Debug impl). Mark Other so
                // the violation check skips.
                block = BlockKind::Other;
            }
        }

        // Update brace depth from this line's net braces (excluding
        // braces inside string/char literals — best effort approximation;
        // the engine.rs file doesn't use brace-bearing string literals
        // in code positions that would confuse the tracker).
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth < 0 {
            depth = 0;
        }
        if depth == 0 {
            // Returned to top-level — reset block kind so the next
            // top-level item gets re-detected.
            block = BlockKind::Other;
        }

        // Check for `RedbBackend` substring violations.
        if line.contains("RedbBackend") {
            // Skip rustdoc + line comments — they're narrative.
            if trimmed.starts_with("//") {
                continue;
            }
            // Allowed: the default-alias `pub type` line.
            if line.contains("pub type Engine =") {
                continue;
            }
            // Allowed: any line inside an `impl Engine { ... }` block.
            if block == BlockKind::ImplEngineAliased {
                continue;
            }
            // Allowed: the `impl Engine {` line itself (the impl
            // declaration is re-detected at depth 0 after the brace
            // increment above; the line's trimmed prefix says `impl
            // Engine`).
            if trimmed.starts_with("impl Engine") {
                continue;
            }
            // Everything else is a violation — D-PHASE-3-1 contract
            // requires no inherent `RedbBackend` references in the
            // generic-cascade block or at module top level.
            violations.push((lineno, line.to_string(), block));
        }
    }

    assert!(
        violations.is_empty(),
        "engine.rs cites `RedbBackend` outside the default-alias scope (D-PHASE-3-1 violation):\n{}",
        violations
            .iter()
            .map(|(l, line, blk)| format!("  line {l} ({blk:?}): {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
