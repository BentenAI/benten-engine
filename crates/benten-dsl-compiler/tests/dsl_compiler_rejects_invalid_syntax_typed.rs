//! G12-B red-phase: bad DSL input fires a typed `CompileError::Parse(...)`
//! (or `Semantic` / `Emit`) variant whose `Diagnostic::error_code` matches
//! the stable `E_DSL_*` discriminant the devserver renders.
//!
//! Per `r1-architect-reviewer.json` G12-B-scope: the public surface includes
//! `CompileError` enum + `Diagnostic` shape — so devserver can switch on
//! discriminant without prose-string parsing.
//!
//! TDD red-phase. Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: typed parse error not yet implemented"]
fn dsl_compiler_rejects_unbalanced_braces_with_e_dsl_parse_error() {
    let _src = r"handler 'oops' { read('post') -> respond"; // no closing brace
    todo!(
        "R5 G12-B: assert compile_str returns CompileError::Parse(d) with d.error_code == E_DSL_PARSE_ERROR"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: unknown primitive error not yet implemented"]
fn dsl_compiler_rejects_unknown_primitive_with_typed_semantic_error() {
    let _src = r"handler 'oops' { read('post') -> teleport -> respond }"; // teleport not a primitive
    todo!("R5 G12-B: assert CompileError::Semantic(d) with d.error_code == E_DSL_UNKNOWN_PRIMITIVE")
}

#[test]
#[ignore = "R5 G12-B red-phase: missing-respond emit error not yet implemented"]
fn dsl_compiler_rejects_handler_without_respond_with_typed_emit_error() {
    // Plan §3.2 G12-B inherits the "every handler ends in RESPOND" property
    // from existing SubgraphSpec validation — surfaced via Emit variant.
    let _src = r"handler 'no-respond' { read('post') }";
    todo!("R5 G12-B: assert CompileError::Emit(d) with d.error_code == E_DSL_MISSING_RESPOND")
}

#[test]
#[ignore = "R5 G12-B red-phase: empty-source error not yet implemented"]
fn dsl_compiler_rejects_empty_source_with_typed_parse_error() {
    let _src = "";
    todo!(
        "R5 G12-B: assert CompileError::Parse(d) with non-empty diagnostic message + line/column reported as None"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: diagnostic span not yet implemented"]
fn dsl_compiler_diagnostic_carries_line_and_column_for_parse_failure() {
    let _src = "handler 'has-newline' {\nread('post') ->\nteleport\n-> respond\n}";
    todo!("R5 G12-B: assert Diagnostic.line == Some(3) + Diagnostic.column == Some(1) (1-indexed)")
}
