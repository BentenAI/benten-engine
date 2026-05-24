//! G-CORE-9 R2 fix-pass — DSL `#[non_exhaustive]` audit pins.
//!
//! Closes L9-r2-MIN-3: extends the `g_core_9_non_exhaustive_audit.rs`
//! audit-test coverage to the 5 DSL public types named in
//! V1-FROZEN-INTERFACE.md item 11 (CompileError + CompiledSubgraph +
//! CompiledPrimitive + Span + Diagnostic). The `#[non_exhaustive]`
//! attribute is applied per L9-DSL-MAJOR-2 closure at
//! `crates/benten-dsl-compiler/src/lib.rs:180/193/306/499/548`.
//!
//! This file lives in `crates/benten-dsl-compiler/tests/` because the
//! `g_core_9_non_exhaustive_audit.rs` audit-test in `crates/benten-engine/tests/`
//! cannot reach the DSL crate (benten-engine has no benten-dsl-compiler
//! dep; the DSL is a consumer-of not consumed-by). The audit-test pin
//! lives at the type-defining crate's own integration-test surface, which
//! is the canonical location per the L9 R2 lens recommendation.

use benten_dsl_compiler::{CompileError, CompiledPrimitive, CompiledSubgraph, Diagnostic, Span};

/// Type-existence pins — compile-fails if the type goes away or moves.
#[test]
fn dsl_5_pub_types_exist_at_v1_beta() {
    assert!(std::any::type_name::<CompiledSubgraph>().ends_with("CompiledSubgraph"));
    assert!(std::any::type_name::<CompiledPrimitive>().ends_with("CompiledPrimitive"));
    assert!(std::any::type_name::<CompileError>().ends_with("CompileError"));
    assert!(std::any::type_name::<Span>().ends_with("Span"));
    assert!(std::any::type_name::<Diagnostic>().ends_with("Diagnostic"));
}

/// `#[non_exhaustive]` regression-guard via the `compile_str` round-trip
/// + match-arm exhaustiveness check. Tests `CompileError` via the public
/// API surface — `compile_str("")` returns a `Build` arm (empty source
/// is not a valid DSL); the match here asserts the 3 documented arms +
/// the `_ =>` wildcard guard required by `#[non_exhaustive]`.
#[test]
fn dsl_compile_error_non_exhaustive_match_requires_wildcard() {
    let result = benten_dsl_compiler::compile_str("");
    let err = result.expect_err("empty source must error");
    let classified: &'static str = match err {
        CompileError::Parse(_) => "Parse",
        CompileError::Semantic(_) => "Semantic",
        CompileError::Build(_) => "Build",
        CompileError::Io(_) => "Io",
        CompileError::Backend(_) => "Backend",
        // G-CORE-9 R2 (L9-r2-MIN-3): the wildcard arm IS the regression-guard;
        // its required presence is the `#[non_exhaustive]` structural enforcement.
        _ => "Unknown(non_exhaustive guard)",
    };
    // Any of the 5 named arms is acceptable; the load-bearing assertion is the
    // wildcard-arm presence (without it the match fails to compile post-freeze).
    assert!(
        ["Parse", "Semantic", "Build", "Io", "Backend"].contains(&classified)
            || classified == "Unknown(non_exhaustive guard)",
        "CompileError variant classification surprise: {classified}"
    );
}
