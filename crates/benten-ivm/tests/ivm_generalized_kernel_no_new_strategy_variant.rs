//! G23-0a IVM-subgraph generalization: NO NEW Strategy variants.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.2 row 1 (cag-r1-2).
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0a.
//!
//! ## What this asserts
//!
//! G23-0a renames `Strategy::C` → `Strategy::Reserved` (arch-r1-14) +
//! generalizes the Algorithm B kernel to consume `SubgraphSpec`. The
//! Strategy enum MUST remain a 3-variant closed set
//! `{ A, B, Reserved }` post-generalization. **NO new Strategy variant
//! is minted to carry the generalized-kernel surface** (CLAUDE.md
//! baked-in #2 — `Strategy::B` already IS the generalized Algorithm B).
//!
//! ## §3.6b would-FAIL-if-no-op'd substantive arm
//!
//! Variant count is grepped from production source (`strategy.rs`). A
//! no-op G23-0a implementation that adds a `Strategy::Generalized` or
//! `Strategy::Subgraph` variant would fail the count assertion. The
//! grep-assert is the load-bearing observable: matchsite enumeration
//! over the production enum body, not a runtime poke.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0a when the rename + generalization land. Per
//! pim-12 §3.6e the implementer un-ignores this pin at landing time.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

mod common_kernel_canary;
use common_kernel_canary::EXPECTED_STRATEGY_VARIANTS;

#[test]
fn strategy_enum_remains_3_variants_post_generalization() {
    // Locate `crates/benten-ivm/src/strategy.rs` via CARGO_MANIFEST_DIR.
    // Workspace layout: <root>/crates/benten-ivm/{tests,src}.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is always set under cargo test");
    let strategy_rs = PathBuf::from(&manifest_dir).join("src/strategy.rs");
    let source = fs::read_to_string(&strategy_rs)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", strategy_rs.display()));

    // Substantive arm — extract the variant identifiers from the `enum
    // Strategy { ... }` body. A naive line-count won't do (rustdoc lines
    // intermix); we walk lines inside the block + accept identifiers
    // ending with `,` (closed-enum convention) or terminating the block.
    let mut in_enum_block = false;
    let mut variants: Vec<String> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub enum Strategy") {
            in_enum_block = true;
            continue;
        }
        if !in_enum_block {
            continue;
        }
        if trimmed == "}" {
            break;
        }
        // Variant lines look like `A,` or `Reserved,`. Skip doc-comments
        // + attributes + empty lines.
        if trimmed.is_empty()
            || trimmed.starts_with("///")
            || trimmed.starts_with("//")
            || trimmed.starts_with("#[")
        {
            continue;
        }
        // Variant lines are bare identifiers (no `=`, no `{`, no `(`)
        // — closed enum with no payload per D8-RESOLVED.
        if !trimmed.contains('=') && !trimmed.contains('{') && !trimmed.contains('(') {
            let ident = trimmed.trim_end_matches(',').trim().to_string();
            if ident.chars().all(|c| c.is_alphanumeric() || c == '_') && !ident.is_empty() {
                variants.push(ident);
            }
        }
    }

    assert_eq!(
        variants.len(),
        EXPECTED_STRATEGY_VARIANTS.len(),
        "Strategy enum must have exactly 3 variants post-G23-0a — \
         {{ A, B, Reserved }}. Got `{:?}`. A new variant minted to carry \
         the generalized-kernel surface would violate CLAUDE.md baked-in \
         #2 (Strategy::B IS the generalized Algorithm B; do not split).",
        variants
    );

    for expected in EXPECTED_STRATEGY_VARIANTS {
        assert!(
            variants.iter().any(|v| v == expected),
            "expected variant `Strategy::{expected}` missing from production \
             source; found variants `{variants:?}`"
        );
    }
}

#[test]
fn strategy_enum_does_not_contain_strategy_c() {
    // arch-r1-14 rename verification — post-G23-0a, no `C` variant in
    // the Strategy enum body. Grep-assert because the rename is
    // intentionally observable at source level (closes CRATES-DEEP-DIVE
    // §4 named-but-deferred item per arch-r1-14).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is always set under cargo test");
    let strategy_rs = PathBuf::from(&manifest_dir).join("src/strategy.rs");
    let source = fs::read_to_string(&strategy_rs)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", strategy_rs.display()));

    // Walk into the enum body + scan for a bare `C,` variant line.
    let mut in_enum_block = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub enum Strategy") {
            in_enum_block = true;
            continue;
        }
        if !in_enum_block {
            continue;
        }
        if trimmed == "}" {
            break;
        }
        let ident = trimmed.trim_end_matches(',').trim();
        assert_ne!(
            ident, "C",
            "Strategy::C must be renamed to Strategy::Reserved per arch-r1-14 \
             G23-0a (closes CRATES-DEEP-DIVE §4 named-but-deferred item). \
             Source line: `{line}`"
        );
    }
}

#[test]
fn strategy_enum_contains_reserved_variant() {
    // Companion-positive pin to the negative `_does_not_contain_strategy_c`
    // pin above — both must hold to confirm the rename is complete.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is always set under cargo test");
    let strategy_rs = PathBuf::from(&manifest_dir).join("src/strategy.rs");
    let source = fs::read_to_string(&strategy_rs)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", strategy_rs.display()));

    let mut found_reserved = false;
    let mut in_enum_block = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub enum Strategy") {
            in_enum_block = true;
            continue;
        }
        if !in_enum_block {
            continue;
        }
        if trimmed == "}" {
            break;
        }
        let ident = trimmed.trim_end_matches(',').trim();
        if ident == "Reserved" {
            found_reserved = true;
            break;
        }
    }

    assert!(
        found_reserved,
        "Strategy::Reserved must be present in production source \
         post-G23-0a rename (arch-r1-14)."
    );
}
