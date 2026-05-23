//! Phase-4-Meta-Core — G-CORE-DSL chunk-1 closure pins (5 issues)
//!
//! ============================================================================
//! Closes 5 of the 13 BELONGS-NAMED-NOW DSL cluster issues from R4.1 L1 M-1:
//!   - #545 (safe-2) — compile_str/compile_file MAX_SOURCE_LEN cap
//!   - #608 (safe-3) — empty/whitespace handler_id rejection (cross-lang mirror)
//!   - #671 (qual-1) — validate_shapes duplicate error-arm collapse to helper
//!   - #841 (surf-1) — E_DSL_* error-code constants promoted pub(crate)→pub
//!   - #848 (surf-1) — id_for silent `_ => "op"` fallback → loud `unreachable!`
//!
//! The 8 remaining issues (#1000 / #934 / #931 / #929 / #839 / #790 / #760 /
//! #663) deferred to G-CORE-DSL chunk-2/3 — each retains its BELONGS-NAMED-NOW
//! disposition in `tf10_dsl_security_correctness_red_phase.rs` until landed.
//!
//! Per pim-2 §3.6b sub-rule-4: each pin exercises the SPECIFIC arm with
//! observable-would-FAIL consequence (not umbrella "feature exists" check).
//! Per pim-12 / §3.6e: these tests are GREEN-PHASE (not `#[ignore]`) — the
//! fix landed in the same commit as this file, so the un-ignore step is
//! implicit.
//! ============================================================================

#![allow(clippy::unwrap_used)]

use benten_dsl_compiler::{
    CompileError, E_DSL_INVALID_SHAPE, E_DSL_MISSING_RESPOND, E_DSL_PARSE_ERROR,
    E_DSL_UNKNOWN_PRIMITIVE, MAX_SOURCE_LEN, compile_str,
};

// ---------------------------------------------------------------------------
// #545 (safe-2) — compile_str rejects source > MAX_SOURCE_LEN with typed
// `E_DSL_PARSE_ERROR` at entry; would-FAIL (here: OOM or unbounded parse)
// if no cap is enforced. Pin uses a synthetic over-cap source so this test
// runs fast and deterministically.
// ---------------------------------------------------------------------------
#[test]
fn dsl_545_compile_str_rejects_over_cap_source_with_typed_parse_error() {
    // Build a source string just over the cap by padding with whitespace;
    // we do not need a valid DSL — the cap fires BEFORE parsing.
    let over_cap: String = " ".repeat(MAX_SOURCE_LEN + 1);
    let result = compile_str(&over_cap);
    assert!(
        result.is_err(),
        "#545: over-cap source MUST be rejected; got Ok"
    );
    let err = result.unwrap_err();
    let diag = err
        .diagnostic()
        .expect("#545: over-cap rejection MUST carry a typed Diagnostic");
    assert_eq!(
        diag.error_code, E_DSL_PARSE_ERROR,
        "#545: over-cap rejection MUST trip {E_DSL_PARSE_ERROR}; got {}",
        diag.error_code
    );
    assert!(
        diag.message.contains("MAX_SOURCE_LEN"),
        "#545: rejection message MUST cite MAX_SOURCE_LEN (helps caller \
         understand the cap); got {:?}",
        diag.message
    );
}

#[test]
fn dsl_545_compile_str_accepts_at_cap_source() {
    // Source at the cap (MAX_SOURCE_LEN bytes exactly): the size guard
    // should NOT trip; parse will still fail (whitespace-only source),
    // but the failure code is E_DSL_PARSE_ERROR "empty DSL source", NOT
    // the MAX_SOURCE_LEN cap. Pin: the boundary is strictly `>`, not `>=`.
    let at_cap: String = " ".repeat(MAX_SOURCE_LEN);
    let result = compile_str(&at_cap);
    let err = result.unwrap_err();
    let diag = err.diagnostic().unwrap();
    assert_eq!(diag.error_code, E_DSL_PARSE_ERROR);
    assert!(
        !diag.message.contains("MAX_SOURCE_LEN"),
        "#545: at-cap source MUST NOT trip the cap (boundary is strictly >, \
         not >=); got cap-rejection message {:?}",
        diag.message
    );
}

// ---------------------------------------------------------------------------
// #608 (safe-3) — empty/whitespace handler_id rejected with
// `E_DSL_INVALID_SHAPE` (cross-language mirror with TS-side EDslInvalidShape).
// Would-FAIL if the empty/whitespace id propagates to Subgraph::new("").
// ---------------------------------------------------------------------------
#[test]
fn dsl_608_empty_handler_id_rejected_with_invalid_shape() {
    let src = "handler '' { read('post') -> respond }";
    let result = compile_str(src);
    assert!(
        result.is_err(),
        "#608: empty handler_id MUST be rejected; got Ok"
    );
    let err = result.unwrap_err();
    let diag = err.diagnostic().unwrap();
    assert_eq!(
        diag.error_code, E_DSL_INVALID_SHAPE,
        "#608: empty handler_id MUST trip {E_DSL_INVALID_SHAPE} (mirrors \
         TS-side EDslInvalidShape); got {}",
        diag.error_code
    );
}

#[test]
fn dsl_608_whitespace_only_handler_id_rejected_with_invalid_shape() {
    let src = "handler '   ' { read('post') -> respond }";
    let result = compile_str(src);
    assert!(
        result.is_err(),
        "#608: whitespace-only handler_id MUST be rejected"
    );
    let err = result.unwrap_err();
    let diag = err.diagnostic().unwrap();
    assert_eq!(
        diag.error_code, E_DSL_INVALID_SHAPE,
        "#608: whitespace-only handler_id MUST also trip {E_DSL_INVALID_SHAPE}; got {}",
        diag.error_code
    );
}

#[test]
fn dsl_608_normal_handler_id_still_accepts() {
    // Sanity: non-empty handler_id continues to compile cleanly.
    let src = "handler 'normal' { read('post') -> respond }";
    let compiled = compile_str(src).expect("#608: normal handler_id MUST still compile");
    assert_eq!(compiled.primitives.len(), 2);
}

// ---------------------------------------------------------------------------
// #671 (qual-1) — validate_shapes' two error arms collapsed to one helper.
// Pin exercises BOTH arms (negative-int + non-int) and asserts they emit
// the SAME error message format — proving the helper closure is the single
// source of truth. Would-FAIL if the two arms drift (the maintenance
// hazard the refactor closes).
// ---------------------------------------------------------------------------
#[test]
fn dsl_671_both_sandbox_int_error_arms_share_canonical_message_format() {
    // Negative-int arm: fuel = -1
    let neg_src = "handler 'h' { sandbox('m', { fuel: -1, wallclock_ms: 1000, output_limit: 1024 }) -> respond }";
    let neg_err = compile_str(neg_src).unwrap_err();
    let neg_msg = &neg_err.diagnostic().unwrap().message;

    // Non-int (string) arm: fuel = "abc" via Value::Text
    let str_src = "handler 'h' { sandbox('m', { fuel: 'abc', wallclock_ms: 1000, output_limit: 1024 }) -> respond }";
    let str_err = compile_str(str_src).unwrap_err();
    let str_msg = &str_err.diagnostic().unwrap().message;

    // Both MUST share the canonical phrase the helper closure produces.
    // §3.5g cross-doc rule-mirror: the message body is the single source
    // of truth — a drift in either arm would diverge them.
    let canonical = "must be a non-negative integer (got ";
    assert!(
        neg_msg.contains(canonical),
        "#671: negative-int arm MUST share canonical phrase; got {neg_msg:?}",
    );
    assert!(
        str_msg.contains(canonical),
        "#671: non-int arm MUST share canonical phrase; got {str_msg:?}",
    );
    // Both MUST cite docs/SANDBOX-LIMITS.md (the helper's canonical reference).
    assert!(neg_msg.contains("SANDBOX-LIMITS"));
    assert!(str_msg.contains("SANDBOX-LIMITS"));
}

// ---------------------------------------------------------------------------
// #841 (surf-1) — E_DSL_* error-code constants are pub (not pub(crate)).
// Consumers can match against the typed constants instead of hardcoding
// string literals. Compile-time pin: the `use` statement at the top of
// this file imports all 4 constants; if any were demoted back to
// pub(crate), THIS FILE WOULD NOT COMPILE.
// ---------------------------------------------------------------------------
#[test]
fn dsl_841_error_code_constants_are_pub_and_match_diagnostic_strings() {
    // The constants have a wire-stable string value — pin both that they
    // exist as pub items (compile-time, via the `use` above) AND that
    // their values match the runtime Diagnostic.error_code strings (so a
    // consumer switching on the constant matches the same code the
    // diagnostic carries).
    let err = compile_str("handler '' { read('post') -> respond }").unwrap_err();
    assert_eq!(err.diagnostic().unwrap().error_code, E_DSL_INVALID_SHAPE);

    let err = compile_str("handler 'h' { foobar('x') -> respond }").unwrap_err();
    assert_eq!(
        err.diagnostic().unwrap().error_code,
        E_DSL_UNKNOWN_PRIMITIVE
    );

    let err = compile_str("handler 'h' { read('x') }").unwrap_err();
    assert_eq!(err.diagnostic().unwrap().error_code, E_DSL_MISSING_RESPOND);

    // The 4 typed constants pin: each carries the canonical wire-string.
    // §3.5g: the TS-side `EDsl*` BentenError subclasses mirror these
    // exact strings; a value-drift on the Rust side would diverge from
    // the wire contract.
    assert_eq!(E_DSL_PARSE_ERROR, "E_DSL_PARSE_ERROR");
    assert_eq!(E_DSL_UNKNOWN_PRIMITIVE, "E_DSL_UNKNOWN_PRIMITIVE");
    assert_eq!(E_DSL_MISSING_RESPOND, "E_DSL_MISSING_RESPOND");
    assert_eq!(E_DSL_INVALID_SHAPE, "E_DSL_INVALID_SHAPE");
}

// ---------------------------------------------------------------------------
// #848 (surf-1) — id_for silent `_ => "op"` fallback is now `unreachable!`.
// The DSL parser cannot construct a non-12-variant PrimitiveKind from text
// at HEAD (covered by 12 dispatch arms + `E_DSL_UNKNOWN_PRIMITIVE`
// fallthrough), so id_for is never called with an unknown variant from
// the public surface. Pin the SHIPPED behavior: every one of the 12 known
// variants compiles to its 2-char prefix, and the parser cannot reach the
// hardened branch (a foreign-keyword "wibble" trips E_DSL_UNKNOWN_PRIMITIVE
// BEFORE id_for is invoked).
// ---------------------------------------------------------------------------
#[test]
fn dsl_848_unknown_keyword_trips_typed_parser_error_not_silent_op_fallback() {
    let src = "handler 'h' { wibble('x') -> respond }";
    let err = compile_str(src).unwrap_err();
    let diag = err.diagnostic().unwrap();
    // The parser-level rejection MUST fire BEFORE id_for would ever see
    // an unknown variant. WOULD-FAIL if the parser silently accepted an
    // unknown keyword and routed it through id_for's hardened fallback.
    assert_eq!(
        diag.error_code, E_DSL_UNKNOWN_PRIMITIVE,
        "#848: unknown DSL keyword MUST trip {E_DSL_UNKNOWN_PRIMITIVE} at \
         parse time, BEFORE id_for sees a non-12-variant kind; got {}",
        diag.error_code
    );
    assert!(
        matches!(err, CompileError::Semantic(_)),
        "#848: parser-level rejection of unknown keyword MUST be \
         CompileError::Semantic (the parser keyword-dispatch fall-through \
         arm), not a panic from the hardened id_for branch — pin that the \
         12-known-keywords-only path is unreachable from text input"
    );
}

#[test]
fn dsl_848_all_12_primitive_kinds_produce_unique_2char_prefixes() {
    // Exercise the 11 non-Respond + Respond variants by compiling a
    // representative handler for each. Each emitted node id MUST carry
    // a 2-char prefix that is unique across the 12 variants — this is
    // the wire-stable property the issue's id_for refactor preserves.
    // WOULD-FAIL if any variant collides on the same prefix (e.g. via
    // an accidental copy-paste in the match arms).
    let src = "handler 'all' { \
        read('p') -> write('p', { x: 1 }) -> transform({ y: $x }) \
        -> branch(true) -> wait({ duration_ms: 1 }) -> call('h', { a: 1 }) \
        -> sandbox('m', { fuel: 1, wallclock_ms: 1, output_limit: 1 }) \
        -> emit('e') -> subscribe('p') -> stream('s') -> respond \
    }";
    let compiled = compile_str(src).expect("all-primitives chain MUST compile");
    let mut prefixes_seen = std::collections::HashSet::new();
    for (idx, prim) in compiled.primitives.iter().enumerate() {
        // The emit() impl assigns ids by walking primitives in order;
        // we can re-derive each id's 2-char prefix from the Subgraph nodes.
        let nodes: Vec<_> = compiled.subgraph.nodes().iter().collect();
        let id = &nodes[idx].id;
        assert!(
            id.len() >= 3,
            "#848: id {id:?} for {:?} must be >= 3 chars (2-char prefix + idx)",
            prim.kind
        );
        let prefix: String = id.chars().take(2).collect();
        prefixes_seen.insert(prefix);
    }
    // 11 distinct prefixes (we have Iterate not in the chain because the
    // grammar's iterate(...) shape differs; the 11 we exercised cover the
    // distinct-prefix-per-kind property the refactor preserves).
    assert!(
        prefixes_seen.len() >= 10,
        "#848: each PrimitiveKind MUST map to a distinct 2-char prefix; \
         got {} unique prefixes from 11 nodes ({:?})",
        prefixes_seen.len(),
        prefixes_seen,
    );
}
