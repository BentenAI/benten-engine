//! G12-B green-phase: bad DSL input fires a typed `CompileError::Parse(...)`
//! (or `Semantic` / `Build`) variant whose `Diagnostic::error_code` matches
//! the stable `E_DSL_*` discriminant the devserver renders.
//!
//! G-CORE-DSL chunk-3 (#790 closure): the prior `Emit` variant was renamed
//! to `Build` to disambiguate from `PrimitiveKind::Emit` (the runtime
//! operation primitive). #1000 closure: `Diagnostic::line` / `column`
//! field-accesses are now method accessors on the underlying `Span`.
//!
//! Per `r1-architect-reviewer.json` G12-B-scope: the public surface includes
//! `CompileError` enum + `Diagnostic` shape — so devserver can switch on
//! discriminant without prose-string parsing.
//!
//! Lifted from red-phase 2026-04-28 (R5 G12-B implementer).

#![allow(clippy::unwrap_used)]

use benten_dsl_compiler::{CompileError, compile_str};

#[test]
fn dsl_compiler_rejects_unbalanced_braces_with_e_dsl_parse_error() {
    let src = "handler 'oops' { read('post') -> respond"; // no closing brace
    let err = compile_str(src).unwrap_err();
    assert!(matches!(err, CompileError::Parse(_)));
    assert_eq!(err.diagnostic().unwrap().error_code, "E_DSL_PARSE_ERROR");
}

#[test]
fn dsl_compiler_rejects_unknown_primitive_with_typed_semantic_error() {
    let src = "handler 'oops' { read('post') -> teleport -> respond }";
    let err = compile_str(src).unwrap_err();
    assert!(matches!(err, CompileError::Semantic(_)));
    assert_eq!(
        err.diagnostic().unwrap().error_code,
        "E_DSL_UNKNOWN_PRIMITIVE"
    );
}

#[test]
fn dsl_compiler_rejects_handler_without_respond_with_typed_build_error() {
    let src = "handler 'no-respond' { read('post') }";
    let err = compile_str(src).unwrap_err();
    // G-CORE-DSL chunk-3 #790 closure: variant renamed `Emit` → `Build`.
    assert!(matches!(err, CompileError::Build(_)));
    assert_eq!(
        err.diagnostic().unwrap().error_code,
        "E_DSL_MISSING_RESPOND"
    );
}

#[test]
fn dsl_compiler_rejects_empty_source_with_typed_parse_error() {
    let err = compile_str("").unwrap_err();
    assert!(matches!(err, CompileError::Parse(_)));
    let d = err.diagnostic().unwrap();
    assert_eq!(d.error_code, "E_DSL_PARSE_ERROR");
    // Empty source has no source span — line/column reported as None.
    // G-CORE-DSL chunk-3 #1000 closure: now via method accessors.
    assert!(d.line().is_none());
    assert!(d.column().is_none());
    assert!(d.span.is_none(), "empty source has no span");
}

#[test]
fn dsl_compiler_diagnostic_carries_line_and_column_for_parse_failure() {
    let src = "handler 'has-newline' {\nread('post') ->\nteleport\n-> respond\n}";
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().unwrap();
    // The unknown `teleport` is on line 3 (1-indexed).
    // G-CORE-DSL chunk-3 #1000 closure: now via method accessors.
    assert_eq!(d.line(), Some(3));
    assert_eq!(d.column(), Some(1));
}
