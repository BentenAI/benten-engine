//! Phase 1 R3 security test — TRANSFORM parser fuzz harness (T12 §Fuzz harness).
//!
//! Attack class: unknown / unnamed syntactic constructs slipping past the
//! allowlist grammar. The 25-class denylist in `docs/TRANSFORM-GRAMMAR.md` is
//! informational — the real contract is "anything not in the BNF is rejected".
//! A human-authored denylist can't enumerate every JS escape hatch (new ones
//! land with every ECMAScript edition), so the load-bearing property is
//! structural: **every accepted input must parse to an AST using ONLY
//! allowlisted nodes**, and **no input may panic**.
//!
//! This harness fuzzes 10k generated JS-like snippets through `parse_transform`
//! and asserts two structural properties on every outcome:
//!
//!   1. If `Ok(ast)` — every node in `ast` is in the documented allowlist
//!      (checked via `ast.uses_only_allowlisted_nodes()`).
//!   2. If `Err(e)` — it is `E_TRANSFORM_SYNTAX`, not some other error kind,
//!      and emphatically not a panic.
//!
//! Harness is marked `#[ignore]` per T12 spec — it runs via
//! `cargo test -p benten-eval -- --ignored fuzz_transform_parser` in a
//! dedicated CI job (not the default `cargo test` path, because 10k
//! iterations × proptest shrinking is a multi-second fuzz run).
//!
//! TDD contract: FAIL at R3 — the parser, the AST-introspection helper, and
//! the `E_TRANSFORM_SYNTAX` code are all E4 + T12 deliverables for R5.
//!
//! Cross-refs:
//! - `docs/TRANSFORM-GRAMMAR.md` §"Fuzz harness"
//! - `.addl/phase-1/r1-security-auditor.json` finding #8
//! - `.addl/phase-1/r2-test-landscape.md` §7 `transform_grammar_fuzz_harness`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::ErrorCode;
use benten_eval::transform::{AstIntrospect, parse_transform};
use proptest::prelude::*;

/// Strategy producing arbitrary UTF-8 strings plausibly-parseable as JS.
/// Biased toward tokens the parser will actually lex (keywords, operators,
/// identifiers, literals) so the harness exercises real parser paths, not
/// just immediate tokenizer rejection on random bytes.
fn fuzzy_js_snippet() -> impl Strategy<Value = String> {
    let tokens = prop::sample::select(vec![
        // identifiers the grammar allows
        "x",
        "y",
        "foo",
        "bar",
        "Math",
        "Date",
        // built-ins (allowlisted)
        "min",
        "max",
        "round",
        "abs",
        "lowercase",
        "uppercase",
        "length",
        // literals
        "1",
        "0",
        "3.14",
        "\"hello\"",
        "true",
        "false",
        "null",
        // allowlisted operators
        "+",
        "-",
        "*",
        "/",
        "%",
        "===",
        "!==",
        "<",
        ">",
        "<=",
        ">=",
        "&&",
        "||",
        "!",
        "?",
        ":",
        ".",
        ",",
        "(",
        ")",
        "[",
        "]",
        "{",
        "}",
        // NOT in allowlist — the harness should see these frequently enough
        // to exercise rejection paths
        "new",
        "this",
        "with",
        "yield",
        "await",
        "typeof",
        "instanceof",
        "=>",
        "**",
        "??",
        "?.",
        "++",
        "--",
        "|",
        "&",
        "^",
        "~",
        "<<",
        ">>",
        "=",
        "function",
        "return",
        "throw",
        "delete",
        "import",
        "eval",
        "__proto__",
        "constructor",
        "Symbol",
        "Reflect",
        "...",
        " ",
        " ", // whitespace weight
    ]);

    prop::collection::vec(tokens, 1..20)
        .prop_map(|parts| parts.into_iter().collect::<Vec<_>>().join(""))
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        max_shrink_iters: 256,
        .. ProptestConfig::default()
    })]

    /// The load-bearing property: every outcome is either a clean
    /// allowlist-only AST or a clean `E_TRANSFORM_SYNTAX`. No panics, no
    /// other error kinds, no AST using non-allowlisted nodes.
    #[test]
    #[ignore = "fuzz harness — run via `cargo test -- --ignored`"]
    fn fuzz_transform_parser(src in fuzzy_js_snippet()) {
        match parse_transform(&src) {
            Ok(ast) => {
                prop_assert!(
                    ast.uses_only_allowlisted_nodes(),
                    "Accepted input produced an AST with non-allowlisted \
                     nodes. This is a grammar hole — add the escape to the \
                     denylist in docs/TRANSFORM-GRAMMAR.md and extend \
                     transform_grammar_rejections.rs. src={src:?} ast={ast:?}"
                );
            }
            Err(e) => {
                prop_assert_eq!(
                    e.code(),
                    ErrorCode::TransformSyntax,
                    "rejected input produced the wrong error code. src={:?}",
                    src
                );
                // Contract: the error NEVER panics and ALWAYS includes the
                // offset + original expression.
                prop_assert!(e.offset() <= src.len());
                prop_assert_eq!(e.expression(), src);
            }
        }
    }
}
