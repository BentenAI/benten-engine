//! G-CORE-DSL chunk-3 closure pins: #1000 + #839 + #790.
//!
//! Per CLAUDE.md ADDL R5 + pim-2 §3.6b sub-rule 4 + pim-18 §3.6f
//! SHAPE-not-SUBSTANCE pre-flight: each pin exercises the SPECIFIC
//! substantive arm of the closed issue with a would-FAIL-if-no-op'd
//! property. Companion to the inline chunk-1/2 pins at
//! `src/lib.rs::inline_tests` + the existing `tf10_dsl_chunk1_closures.rs`
//! integration tests.
//!
//! - **#1000** — Diagnostic carries `Span` with full half-open byte
//!   range + endpoint (line, column) coords; pre-fix Diagnostic
//!   carried only a single 1-indexed (line, column) point.
//! - **#839** — `CompileError::Backend(String)` 5th variant + the
//!   typed `E_DSL_BACKEND_REJECTED` error code on the wire (mirrors
//!   `benten_errors::ErrorCode::DslBackendRejected`). Pre-fix
//!   downstream consumers (devserver) abused `CompileError::Io` to
//!   carry post-compile rejections.
//! - **#790** — `CompileError::Emit` → `CompileError::Build` rename
//!   (disambiguates from `PrimitiveKind::Emit` the runtime operation
//!   primitive); `fn emit` internal-build → `fn build`. The compile-
//!   build phase now names itself for what it does (build the
//!   Subgraph from the AST) without overloading the wire `Emit`
//!   primitive's name.

#![allow(clippy::unwrap_used)]

use benten_dsl_compiler::{
    CompileError, E_DSL_BACKEND_REJECTED, E_DSL_INVALID_SHAPE, E_DSL_IO_ERROR,
    E_DSL_MISSING_RESPOND, E_DSL_PARSE_ERROR, Span, compile_str,
};

// ---------------------------------------------------------------------------
// #1000 — Diagnostic carries Span (half-open byte range + endpoint coords)
// ---------------------------------------------------------------------------

/// #1000 substantive arm: a successful parse-error round-trips a `Span`
/// whose `start_offset` AND `end_offset` are valid byte indices into
/// the source AND the offending slice
/// (`source[start_offset..end_offset]`) is the literal offending
/// substring.
///
/// **Would-FAIL property**: a regression that dropped `start_offset` /
/// `end_offset` (or left them at 0) would make the slice empty (or
/// wrong-range), tripping the explicit slice-equality assertion.
#[test]
fn dsl_1000_diagnostic_span_byte_offsets_slice_offending_substring() {
    let src = "handler 'h' { read('post') -> teleport -> respond }";
    //                                       ^^^^^^^^ offending keyword
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().expect("diagnostic present");
    let span = d.span.expect("#1000: span present for unknown-primitive");
    let offender = &src[span.start_offset as usize..span.end_offset as usize];
    assert_eq!(
        offender, "teleport",
        "span slice [{}, {}) should be the offending `teleport` keyword; got {offender:?}",
        span.start_offset, span.end_offset,
    );
}

/// #1000 substantive arm: the Span's endpoint `(line, column)` are
/// post-token cursor coordinates — i.e., `end_line == start_line` AND
/// `end_column == start_column + ident_len` for a same-line ident.
/// This is the editor-squiggle UX property — the renderer can highlight
/// the offending range using the explicit endpoint without re-walking
/// the source.
///
/// **Would-FAIL property**: a regression that always set `end == start`
/// (silently turning Span into a point-span) would collapse the
/// highlight range to a single column.
#[test]
fn dsl_1000_diagnostic_span_endpoint_marks_post_token_cursor() {
    let src = "handler 'h' { read('post') -> teleport -> respond }";
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().expect("diagnostic present");
    let span = d.span.expect("#1000: span present");
    // `teleport` is 8 chars on line 1; the parser tracks 1-indexed
    // columns so end_column = start_column + 8.
    assert_eq!(span.start_line, 1, "single-line source");
    assert_eq!(span.end_line, 1, "single-line source");
    assert_eq!(
        span.end_column - span.start_column,
        8,
        "Span endpoint marks post-token cursor (8 chars of `teleport`); got {span:?}",
    );
}

/// #1000 substantive arm: a multi-line unbalanced-paren span covers
/// the WHOLE walked body from the open paren to end-of-input, not just
/// the open OR close. This is the load-bearing UX win for the
/// AI-loop / editor: the offending range crosses lines, so the renderer
/// surfaces the multi-line construct, not a misleading point.
///
/// **Would-FAIL property**: a regression that anchored the span at
/// only the opening paren OR only the EOF cursor would collapse
/// `end_line - start_line` to 0.
#[test]
fn dsl_1000_unbalanced_paren_span_crosses_multiple_lines() {
    // `branch(...)` body never closes; opening paren on line 1, EOF
    // on line 3 after two newlines inside the body. The span should
    // cover line 1 through line 3.
    let src = "handler 'h' {\n  branch(some.condition\n  || other.condition";
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().expect("diagnostic present");
    let span = d.span.expect("#1000: span present for unbalanced paren");
    assert!(
        span.end_line > span.start_line,
        "multi-line unbalanced-paren span MUST cross lines; got {span:?}",
    );
}

/// #1000 substantive arm: a point-span (no token consumed past start)
/// surfaces as `start == end` coordinates AND offsets, so the renderer
/// places a single cursor rather than degenerately highlighting a
/// 0-width range as a multi-coord span.
///
/// **Would-FAIL property**: a regression that set `end_offset != start_offset` for
/// expected-identifier-at-cursor (where no token consumed) would
/// degenerately make the span non-point. Tests the `parse_identifier`
/// no-progress branch which uses `Span::point()` directly.
#[test]
fn dsl_1000_diagnostic_span_point_construction_collapses_correctly() {
    // `expected identifier` fires inside `parse_identifier` when the
    // cursor sees a non-ascii-alphanumeric+underscore char and makes
    // zero progress. Using `'` (string-open) at the keyword position
    // forces this: parse_handler calls expect_keyword("handler")
    // -> parse_identifier, which sees `'` (not an ident char) +
    // makes zero progress -> emits the point-span "expected identifier".
    let src = "'handler' 'h' { read('post') -> respond }"; // starts with quote
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().expect("diagnostic present");
    let span = d.span.expect("#1000: span present for expected-identifier");
    // Point spans use `Span::point()` semantics — start == end.
    assert_eq!(
        span.start_offset, span.end_offset,
        "point-span MUST have start_offset == end_offset; got {span:?}",
    );
    assert_eq!(
        (span.start_line, span.start_column),
        (span.end_line, span.end_column),
        "point-span MUST have identical start/end coords; got {span:?}",
    );
    // Cursor MUST sit at position 0 (the leading `'`).
    assert_eq!(span.start_offset, 0, "cursor at the leading quote");
}

/// #1000 substantive arm: the new `Diagnostic::line()` / `column()`
/// accessor methods (replacing the prior `.line` / `.column` public
/// fields) unwrap the Span's start coordinates as the canonical
/// backward-readable shape. Test exercises both presence (Some) AND
/// absence (None) cases to confirm the Option<Span> wiring.
///
/// **Would-FAIL property**: a regression that wired the accessors
/// against the WRONG field of Span (e.g. end_line instead of
/// start_line) would yield a different value on a multi-coord span.
#[test]
fn dsl_1000_diagnostic_line_column_accessors_unwrap_span_start() {
    let src = "handler 'h' { read('post') -> teleport -> respond }";
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().expect("diagnostic present");
    let span = d.span.expect("present");
    assert_eq!(d.line(), Some(span.start_line));
    assert_eq!(d.column(), Some(span.start_column));

    // Confirm None case via empty-source path.
    let err2 = compile_str("").unwrap_err();
    let d2 = err2.diagnostic().expect("diagnostic present");
    assert!(d2.span.is_none(), "empty source has no span");
    assert!(d2.line().is_none(), "None when no span");
    assert!(d2.column().is_none(), "None when no span");
}

// ---------------------------------------------------------------------------
// #839 — CompileError::Backend(String) + E_DSL_BACKEND_REJECTED
// ---------------------------------------------------------------------------

/// #839 substantive arm: the new `CompileError::Backend(_)` variant
/// exists AND surfaces its stable error_code via the new
/// `CompileError::error_code()` boundary helper. Pre-fix downstream
/// consumers (devserver `replace_handler_from_dsl_with_outcome`)
/// abused `CompileError::Io` to wrap engine-registration failures —
/// this pin enforces the typed home is reachable + carries the
/// canonical stable code.
///
/// **Would-FAIL property**: a regression that removed the variant or
/// renamed the code constant would fail to compile (compile-time
/// pin); a regression that wired `Backend` → `E_DSL_IO_ERROR` would
/// fail the assert.
#[test]
fn dsl_839_backend_variant_surfaces_typed_error_code() {
    let err = CompileError::Backend("devserver_engine_register: simulated".to_string());
    assert_eq!(
        err.error_code(),
        E_DSL_BACKEND_REJECTED,
        "Backend variant MUST surface E_DSL_BACKEND_REJECTED, not E_DSL_IO_ERROR",
    );
    // diagnostic() must return None — Backend carries free-form string.
    assert!(
        err.diagnostic().is_none(),
        "Backend variant has no inner Diagnostic; carries free-form String",
    );
}

/// #839 substantive arm: `CompileError::Io(_)` surfaces
/// `E_DSL_IO_ERROR` distinct from `E_DSL_BACKEND_REJECTED`. Pre-fix
/// the Io variant was the catch-all bucket; post-fix it preserves
/// its documented semantic ("IO failure reading a source file").
///
/// **Would-FAIL property**: a regression that mapped Io to the
/// generic `E_DSL_COMPILE_ERROR` placeholder OR to
/// `E_DSL_BACKEND_REJECTED` would fail the assert. Confirms the
/// two distinct codes route distinctly per consumer discriminant.
#[test]
fn dsl_839_io_variant_surfaces_distinct_typed_error_code() {
    let err = CompileError::Io("/tmp/missing.dsl: No such file or directory".to_string());
    assert_eq!(
        err.error_code(),
        E_DSL_IO_ERROR,
        "Io variant MUST surface E_DSL_IO_ERROR distinct from E_DSL_BACKEND_REJECTED",
    );
    assert_ne!(
        E_DSL_IO_ERROR, E_DSL_BACKEND_REJECTED,
        "the two codes MUST be distinct (the #839 closure premise)",
    );
}

/// #839 substantive arm + §3.5g cross-language mirror: the Rust
/// `E_DSL_BACKEND_REJECTED` constant matches the TS-side
/// `EDslBackendRejected.code` value (the canonical wire shape).
/// Without this pin the §3.5g cross-language atomic-update discipline
/// is paper-only; with it the drift triggers a compile-time literal
/// mismatch (in Rust) + a runtime test failure (in TS-side suite).
#[test]
fn dsl_839_error_code_mirror_matches_typescript_constant_literal() {
    // The literal must match what packages/engine/src/errors.generated.ts
    // declares as `EDslBackendRejected.code`. If a future agent renames
    // either side without the §3.5g atomic update, this test catches the
    // drift at the Rust-side build (the TS-side side is caught by the
    // catalog round-trip test in benten-errors stable_shape.rs).
    assert_eq!(E_DSL_BACKEND_REJECTED, "E_DSL_BACKEND_REJECTED");
}

// ---------------------------------------------------------------------------
// #790 — CompileError::Emit → CompileError::Build rename
// ---------------------------------------------------------------------------

/// #790 substantive arm: the build-phase variant is `Build`, not
/// `Emit`. A handler missing `respond` was historically rejected with
/// `CompileError::Emit(_)` — now it's `CompileError::Build(_)` with
/// the same diagnostic code (`E_DSL_MISSING_RESPOND`). The wire-stable
/// code is unchanged (load-bearing); only the variant name moves.
///
/// **Would-FAIL property**: a regression that re-introduced `Emit`
/// variant + restored the old wiring would NOT compile against this
/// test (the `Emit` variant no longer exists). This is a load-bearing
/// compile-time pin against accidental revert.
#[test]
fn dsl_790_missing_respond_uses_build_variant_not_emit() {
    let err = compile_str("handler 'h' { read('post') }").unwrap_err();
    // Compile-time pin: `Build` must be a valid variant; pattern uses
    // it directly to fail compile if the variant doesn't exist or was
    // renamed back to Emit.
    assert!(
        matches!(err, CompileError::Build(_)),
        "missing-respond MUST surface as CompileError::Build (post-#790 rename), got {err:?}",
    );
    assert_eq!(
        err.diagnostic().unwrap().error_code,
        E_DSL_MISSING_RESPOND,
        "wire-stable error code preserved across rename — load-bearing",
    );
}

/// #790 substantive arm: shape-validation failures (e.g. SANDBOX
/// `fuel` declared as a non-integer) ALSO surface as
/// `CompileError::Build`, not `Emit`. The validate_shapes pass shares
/// the build-phase variant with the missing-respond path; this pin
/// confirms ALL build-phase errors carry the new variant name.
///
/// **Would-FAIL property**: a regression that re-introduced a separate
/// `Emit` variant for shape-validation only would split the build-
/// phase errors across two variants, defeating the #790 unification
/// intent.
#[test]
fn dsl_790_shape_validation_uses_build_variant_not_emit() {
    let src = "handler 'h' { sandbox('mod', { fuel: 'high' }) -> respond }";
    let err = compile_str(src).unwrap_err();
    assert!(
        matches!(err, CompileError::Build(_)),
        "shape-validation MUST surface as CompileError::Build, got {err:?}",
    );
    assert_eq!(err.diagnostic().unwrap().error_code, E_DSL_INVALID_SHAPE);
}

/// #790 substantive arm: the Display body of `CompileError::Build`
/// reads "DSL build error: ..." not "DSL emit error: ...". The
/// human-readable surface matches the variant name so log greppers
/// and user-facing prose align with the typed shape.
///
/// **Would-FAIL property**: a regression that left the `#[error("DSL
/// emit error: ...")]` attribute would produce the wrong prose under
/// `Display`.
#[test]
fn dsl_790_build_variant_display_prose_matches_variant_name() {
    let err = compile_str("handler 'h' { read('post') }").unwrap_err();
    let display = format!("{err}");
    assert!(
        display.starts_with("DSL build error:"),
        "Display prose MUST match variant name; got {display:?}",
    );
}

// ---------------------------------------------------------------------------
// Canonical-bytes stability: the chunk-3 changes are diagnostic-shape
// only — they MUST NOT alter `canonical_subgraph_bytes` for ANY DSL
// handler. The existing inline pin
// `permuted_keys_yield_identical_canonical_bytes` (in src/lib.rs)
// covers the permutation-stability property; this pin is the
// chunk-3-specific anchor.
// ---------------------------------------------------------------------------

/// Canonical-bytes anchor pin: a successful compile under chunk-3 produces
/// byte-identical canonical bytes to what it would have produced pre-fix.
/// We can't compare to a frozen pre-fix snapshot (the snapshot would
/// already drift), but we CAN verify the canonical bytes stability of the
/// emission path itself: two compiles of the same source MUST yield
/// identical canonical bytes (deterministic). This is the would-FAIL
/// property against accidental non-determinism introduced by Span-tracking
/// changes (e.g. if the parser's byte-offset tracking accidentally
/// affected the AST shape).
#[test]
fn dsl_chunk3_canonical_bytes_stable_under_diagnostic_shape_change() {
    let src = "handler 'h' { sandbox('m', { fuel: 100, wallclock_ms: 500 }) -> respond }";
    let a = compile_str(src).unwrap();
    let b = compile_str(src).unwrap();
    assert_eq!(
        a.subgraph.to_canonical_bytes().unwrap(),
        b.subgraph.to_canonical_bytes().unwrap(),
        "deterministic canonical bytes — chunk-3 Span/Build/Backend changes are diagnostic-shape ONLY",
    );
}

// ---------------------------------------------------------------------------
// Span::point sanity
// ---------------------------------------------------------------------------

/// `Span::point` constructor produces a degenerate-equal Span where
/// start == end on all three axes (line / column / offset). The
/// constructor is the load-bearing convenience for point-span
/// diagnostics; this pin confirms it doesn't drift.
#[test]
fn dsl_1000_span_point_constructor_produces_degenerate_equal_endpoints() {
    let s = Span::point(7, 13, 42);
    assert_eq!(s.start_line, 7);
    assert_eq!(s.end_line, 7);
    assert_eq!(s.start_column, 13);
    assert_eq!(s.end_column, 13);
    assert_eq!(s.start_offset, 42);
    assert_eq!(s.end_offset, 42);
}

// ---------------------------------------------------------------------------
// E_DSL_PARSE_ERROR still drives Parse variant — sanity counterpart to
// confirm the diagnostic accessors haven't broken existing wiring.
// ---------------------------------------------------------------------------

/// Sanity counterpart: Parse-variant diagnostics still carry their
/// stable code AND now a Span (the chunk-3 change preserves all
/// existing wire shapes; only adds Span coordinates).
#[test]
fn dsl_chunk3_parse_variant_still_carries_stable_error_code() {
    let src = "handler 'oops' { read('post') -> respond"; // missing `}`
    let err = compile_str(src).unwrap_err();
    let d = err.diagnostic().expect("diagnostic present");
    assert_eq!(d.error_code, E_DSL_PARSE_ERROR);
    assert!(
        d.span.is_some(),
        "post-#1000 parse-error MUST carry a Span (not None)",
    );
}
