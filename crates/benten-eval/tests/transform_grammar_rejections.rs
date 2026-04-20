//! Phase 1 R3 security test — TRANSFORM grammar rejections (T12, R1 major #8).
//!
//! Attack class: TRANSFORM expression escape hatches. The grammar is a
//! positive allowlist; any construct not produced by the BNF in
//! `docs/TRANSFORM-GRAMMAR.md` is a parse error with `E_TRANSFORM_SYNTAX`.
//! This file asserts one rejection per class in the doc's "Rejected
//! constructs" appendix. The per-test name matches the appendix's named-test
//! list so a grep across doc ↔ code is mechanical.
//!
//! Contract for every test:
//!   1. `parse(expr)` returns `Err(E_TRANSFORM_SYNTAX)`.
//!   2. The error's `offset` field points at the byte offset of the FIRST
//!      rejected token (per the grammar doc's error-format contract). This
//!      is load-bearing for DX: the DSL source-map (B9) uses `offset` to
//!      highlight the right character.
//!
//! TDD contract: FAIL at R3 — `parse_transform`, `TransformParseError`, and
//! the `E_TRANSFORM_SYNTAX` error code are E4 + T12 deliverables landed in R5.
//!
//! Cross-refs:
//! - `docs/TRANSFORM-GRAMMAR.md` — BNF + 25-class denylist + named tests
//! - `.addl/phase-1/r1-security-auditor.json` finding #8 (major)
//! - `.addl/phase-1/r1-triage.md` T12 deliverable

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::transform::{TransformParseError, parse_transform};

/// Assert that `expr` is rejected with `E_TRANSFORM_SYNTAX`. The expected
/// offset is computed from the input string by locating `token` (the FIRST
/// rejected token per the grammar-doc contract). Rewritten at R4 triage
/// (M24) — hardcoded integers were brittle to parser error-message-format
/// refactors. Computing the offset from the source text makes every
/// rejection test robust to re-tokenization changes.
#[track_caller]
fn assert_rejected_with(expr: &str, token: &str) {
    let expected_offset = expr
        .find(token)
        .unwrap_or_else(|| panic!("test setup: `{token}` must appear in `{expr}`"));
    let err: TransformParseError =
        parse_transform(expr).expect_err(&format!("expected rejection of `{expr}`"));
    assert_eq!(
        err.code(),
        ErrorCode::TransformSyntax,
        "wrong error code for `{expr}`: {:?}",
        err.code()
    );
    assert_eq!(
        err.offset(),
        expected_offset,
        "error offset must point at the FIRST rejected token per grammar \
         doc contract. expr=`{expr}` token=`{token}` expected={expected_offset} got={}",
        err.offset()
    );
    assert_eq!(err.expression(), expr);
}

/// Legacy wrapper for existing tests that pass an absolute offset. Still
/// supported for statements that start at offset 0 (the entire token IS the
/// reject point) where the offset-from-text form adds no clarity.
#[track_caller]
fn assert_rejected_at(expr: &str, expected_offset: usize) {
    let err: TransformParseError =
        parse_transform(expr).expect_err(&format!("expected rejection of `{expr}`"));
    assert_eq!(
        err.code(),
        ErrorCode::TransformSyntax,
        "wrong error code for `{expr}`: {:?}",
        err.code()
    );
    assert_eq!(
        err.offset(),
        expected_offset,
        "error offset must point at the FIRST rejected token per grammar \
         doc contract. expr=`{expr}` expected={expected_offset} got={}",
        err.offset()
    );
    assert_eq!(err.expression(), expr);
}

// -- Class 1: closures (two named variants) ---------------------------------

#[test]
fn transform_grammar_rejects_closure_arrow() {
    // `(x) => x + 1` — arrow function. First rejected token is the `=>`.
    assert_rejected_with("(x) => x + 1", "=>");
}

#[test]
fn transform_grammar_rejects_closure_function_expression() {
    // `function (x) { return x }` — `function` keyword at offset 0.
    assert_rejected_at("function (x) { return x }", 0);
}

// -- Class 2: this ---------------------------------------------------------

#[test]
fn transform_grammar_rejects_this_keyword() {
    assert_rejected_at("this.x", 0);
}

// -- Class 3: imports / requires -------------------------------------------

#[test]
fn transform_grammar_rejects_import_statement() {
    assert_rejected_at("import x from 'y'", 0);
}

#[test]
fn transform_grammar_rejects_require_call() {
    // `require` is a plain identifier; the attack is invoking it as a call.
    // Grammar rejects `require` in expression position because it's not in
    // the built-in allowlist AND the call target resolves unsafely.
    assert_rejected_at("require('x')", 0);
}

// -- Class 4: prototype access (three named variants) ----------------------

#[test]
fn transform_grammar_rejects_proto_access() {
    // `obj.__proto__` — rejected at the `.__proto__` span.
    assert_rejected_with("obj.__proto__", ".__proto__");
}

#[test]
fn transform_grammar_rejects_constructor_access() {
    assert_rejected_with("obj.constructor", ".constructor");
}

#[test]
fn transform_grammar_rejects_prototype_access() {
    assert_rejected_with("obj.prototype", ".prototype");
}

// -- Class 5: tagged templates ---------------------------------------------

#[test]
fn transform_grammar_rejects_tagged_template() {
    // `tag`lit`` — backtick.
    assert_rejected_with("tag`literal`", "`");
}

// -- Class 6: template literals with expressions ---------------------------

#[test]
fn transform_grammar_rejects_template_literal_with_expression() {
    // Backtick at 0.
    assert_rejected_at("`prefix ${x} suffix`", 0);
}

// -- Class 7: optional chaining --------------------------------------------

#[test]
fn transform_grammar_rejects_optional_chaining() {
    // `obj?.method?.()` — `?.` token (first occurrence).
    assert_rejected_with("obj?.method?.()", "?.");
}

// -- Class 8: computed property names --------------------------------------

#[test]
fn transform_grammar_rejects_computed_property_name() {
    // `{ [expr]: 1 }` — `[` at the computed-property opener.
    assert_rejected_with("{ [expr]: 1 }", "[");
}

// -- Class 9: new ----------------------------------------------------------

#[test]
fn transform_grammar_rejects_new_keyword() {
    assert_rejected_at("new Foo()", 0);
}

// -- Class 10: with statement ----------------------------------------------

#[test]
fn transform_grammar_rejects_with_statement() {
    assert_rejected_at("with (obj) { x }", 0);
}

// -- Class 11: destructuring -----------------------------------------------

#[test]
fn transform_grammar_rejects_destructuring_pattern() {
    // `const { x } = …` — `const` is a statement, but even destructuring in
    // assignment position is rejected. We test the pattern form.
    assert_rejected_at("const { x } = obj", 0);
}

// -- Class 12: eval / new Function -----------------------------------------

#[test]
fn transform_grammar_rejects_eval_call() {
    assert_rejected_at("eval('1')", 0);
}

#[test]
fn transform_grammar_rejects_new_function() {
    assert_rejected_at("new Function('x', 'return x')", 0);
}

// -- Class 13: coroutines / promises ---------------------------------------

#[test]
fn transform_grammar_rejects_yield() {
    assert_rejected_at("yield x", 0);
}

#[test]
fn transform_grammar_rejects_async_await() {
    assert_rejected_at("await x", 0);
}

// -- Class 14: meta-programming --------------------------------------------

#[test]
fn transform_grammar_rejects_symbol_access() {
    assert_rejected_at("Symbol.iterator", 0);
}

#[test]
fn transform_grammar_rejects_reflect_access() {
    assert_rejected_at("Reflect.get(x, 'y')", 0);
}

#[test]
fn transform_grammar_rejects_proxy_construction() {
    // Rejection fires on `new` at offset 0 per class 9; this test asserts
    // the class-14 denial separately in case a future grammar allows
    // `Proxy` as an identifier (it must still reject invocation).
    assert_rejected_at("new Proxy(t, h)", 0);
}

// -- Class 15: spread in call ----------------------------------------------

#[test]
fn transform_grammar_rejects_spread_in_call() {
    // `fn(...args)` — `...` token.
    assert_rejected_with("fn(...args)", "...");
}

// -- Class 16: comma operator ----------------------------------------------

#[test]
fn transform_grammar_rejects_comma_operator() {
    // `(a, b)` — comma inside parens (NOT an arg separator).
    assert_rejected_with("(a, b)", ",");
}

// -- Class 17: instanceof / typeof / in ------------------------------------

#[test]
fn transform_grammar_rejects_instanceof() {
    // `x instanceof Y` — `instanceof` keyword.
    assert_rejected_with("x instanceof Y", "instanceof");
}

#[test]
fn transform_grammar_rejects_typeof() {
    assert_rejected_at("typeof x", 0);
}

#[test]
fn transform_grammar_rejects_in_operator() {
    // `'k' in obj` — `in` keyword (the `find` will hit the first `i` which is
    // the start of `in`; there's no earlier `in` substring in the expression).
    assert_rejected_with("'k' in obj", "in");
}

// -- Class 18: regex literals ----------------------------------------------

#[test]
fn transform_grammar_rejects_regex_literal() {
    // `/ab/g` — first `/` at offset 0. Grammar has no `/`-starts-regex rule,
    // so the tokenizer must reject here rather than mis-parse as division.
    assert_rejected_at("/ab/g", 0);
}

// -- Class 19: bitwise -----------------------------------------------------

#[test]
fn transform_grammar_rejects_bitwise_and() {
    assert_rejected_with("a & b", "&");
}

#[test]
fn transform_grammar_rejects_bitwise_or() {
    assert_rejected_with("a | b", "|");
}

#[test]
fn transform_grammar_rejects_bitwise_xor() {
    assert_rejected_with("a ^ b", "^");
}

#[test]
fn transform_grammar_rejects_bitshift() {
    assert_rejected_with("a << b", "<<");
}

// -- Class 20: assignment --------------------------------------------------

#[test]
fn transform_grammar_rejects_assignment() {
    assert_rejected_with("a = b", "=");
}

// -- Class 21: increment / decrement ---------------------------------------

#[test]
fn transform_grammar_rejects_increment() {
    assert_rejected_with("x++", "++");
}

// -- Class 22: exponentiation ----------------------------------------------

#[test]
fn transform_grammar_rejects_exponent() {
    assert_rejected_with("a ** b", "**");
}

// -- Class 23: nullish coalescing ------------------------------------------

#[test]
fn transform_grammar_rejects_nullish_coalescing() {
    assert_rejected_with("a ?? b", "??");
}

// -- Class 24: statements --------------------------------------------------

#[test]
fn transform_grammar_rejects_return_statement() {
    assert_rejected_at("return x", 0);
}

#[test]
fn transform_grammar_rejects_throw_statement() {
    assert_rejected_at("throw x", 0);
}

// -- Class 25: delete ------------------------------------------------------

#[test]
fn transform_grammar_rejects_delete_statement() {
    assert_rejected_at("delete obj.x", 0);
}

// -- Error-format contract (cross-cutting) ---------------------------------

/// The error carries the grammar-doc pointer in EVERY rejection. This is how
/// the DX layer (B9) produces actionable messages: "see docs/TRANSFORM-
/// GRAMMAR.md §Rejected Constructs #9". If this link drifts, users get a
/// 404 and the contract is broken silently.
#[test]
fn transform_grammar_error_carries_doc_pointer() {
    let err = parse_transform("new Foo()").unwrap_err();
    assert_eq!(err.grammar_doc(), "docs/TRANSFORM-GRAMMAR.md");
}
