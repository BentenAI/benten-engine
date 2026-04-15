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

use benten_core::ErrorCode;
use benten_eval::transform::{TransformParseError, parse_transform};

/// Assert that `expr` is rejected with `E_TRANSFORM_SYNTAX` at `expected_offset`.
/// Every test in this file goes through this helper to keep the rejection
/// contract consistent.
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
    // Context contract: the error carries the expression back to the caller
    // so DX layers can render it without re-threading. (B9 source-map relies
    // on this.)
    assert_eq!(err.expression(), expr);
}

// -- Class 1: closures (two named variants) ---------------------------------

#[test]
fn transform_grammar_rejects_closure_arrow() {
    // `(x) => x + 1` — arrow function. First rejected token is the `=>` at 4.
    assert_rejected_at("(x) => x + 1", 4);
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
    // `obj.__proto__` — rejected at the `.` preceding `__proto__` (offset 3).
    assert_rejected_at("obj.__proto__", 3);
}

#[test]
fn transform_grammar_rejects_constructor_access() {
    assert_rejected_at("obj.constructor", 3);
}

#[test]
fn transform_grammar_rejects_prototype_access() {
    assert_rejected_at("obj.prototype", 3);
}

// -- Class 5: tagged templates ---------------------------------------------

#[test]
fn transform_grammar_rejects_tagged_template() {
    // `tag`lit`` — backtick at offset 3.
    assert_rejected_at("tag`literal`", 3);
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
    // `obj?.method?.()` — `?.` at offset 3.
    assert_rejected_at("obj?.method?.()", 3);
}

// -- Class 8: computed property names --------------------------------------

#[test]
fn transform_grammar_rejects_computed_property_name() {
    // `{ [expr]: 1 }` — `[` at offset 2.
    assert_rejected_at("{ [expr]: 1 }", 2);
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
    // `fn(...args)` — `...` at offset 3.
    assert_rejected_at("fn(...args)", 3);
}

// -- Class 16: comma operator ----------------------------------------------

#[test]
fn transform_grammar_rejects_comma_operator() {
    // `(a, b)` — comma at offset 2 (inside parens — NOT an arg separator).
    assert_rejected_at("(a, b)", 2);
}

// -- Class 17: instanceof / typeof / in ------------------------------------

#[test]
fn transform_grammar_rejects_instanceof() {
    // `x instanceof Y` — `instanceof` at offset 2.
    assert_rejected_at("x instanceof Y", 2);
}

#[test]
fn transform_grammar_rejects_typeof() {
    assert_rejected_at("typeof x", 0);
}

#[test]
fn transform_grammar_rejects_in_operator() {
    // `'k' in obj` — `in` at offset 4.
    assert_rejected_at("'k' in obj", 4);
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
    assert_rejected_at("a & b", 2);
}

#[test]
fn transform_grammar_rejects_bitwise_or() {
    assert_rejected_at("a | b", 2);
}

#[test]
fn transform_grammar_rejects_bitwise_xor() {
    assert_rejected_at("a ^ b", 2);
}

#[test]
fn transform_grammar_rejects_bitshift() {
    // `a << b` — `<<` at offset 2.
    assert_rejected_at("a << b", 2);
}

// -- Class 20: assignment --------------------------------------------------

#[test]
fn transform_grammar_rejects_assignment() {
    // `a = b` — `=` at offset 2. Expressions are pure.
    assert_rejected_at("a = b", 2);
}

// -- Class 21: increment / decrement ---------------------------------------

#[test]
fn transform_grammar_rejects_increment() {
    // `x++` — `++` at offset 1.
    assert_rejected_at("x++", 1);
}

// -- Class 22: exponentiation ----------------------------------------------

#[test]
fn transform_grammar_rejects_exponent() {
    // `a ** b` — `**` at offset 2.
    assert_rejected_at("a ** b", 2);
}

// -- Class 23: nullish coalescing ------------------------------------------

#[test]
fn transform_grammar_rejects_nullish_coalescing() {
    // `a ?? b` — `??` at offset 2.
    assert_rejected_at("a ?? b", 2);
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
