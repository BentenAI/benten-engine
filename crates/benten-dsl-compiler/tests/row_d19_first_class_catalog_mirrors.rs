//! Row D-19 G-COMP-1 wave (Phase-4-Meta-Core R6 R2 FP integration,
//! Cohort 8) — first-class catalog-mirror regression-guard tests for
//! the 3 newly-minted DSL ErrorCodes (`E_DSL_PARSE_ERROR` /
//! `E_DSL_UNKNOWN_PRIMITIVE` / `E_DSL_MISSING_RESPOND`) + the
//! Strategy::C → Reserved atomic 4-surface §3.5g rename.
//!
//! Per `feedback_pim_n_regression_guard_substantive_arm.md` (§3.6f
//! ext): each test invokes the **production entry point**
//! (`benten_dsl_compiler::compile_str`) + asserts an **observable
//! consequence** (the typed `CompileError::code()` returns the
//! first-class catalog variant — NOT `ErrorCode::Unknown(...)`) +
//! **would FAIL on revert** of the Row D-19 mint (pre-mint the
//! `code()` arm routed `Parse|Semantic|Build` through
//! `ErrorCode::Unknown(d.error_code.to_string())` which would surface
//! as an `Unknown` discriminant — these tests would all fail on
//! `assert_eq!(unknown_variant, DslParseError)` etc.).
//!
//! Anchor: `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-19 (closed
//! at this wave) + `docs/V1-BETA-BREAKING-CHANGES.md` Cohort 8 entry.

use benten_dsl_compiler::compile_str;
use benten_errors::ErrorCode;

/// Production entry point: feeding a DSL source with unbalanced
/// parens drives the parser into `CompileError::Parse(_)` carrying
/// `Diagnostic.error_code = E_DSL_PARSE_ERROR`. Post-Row-D-19 the
/// typed `code()` returns the first-class `ErrorCode::DslParseError`
/// (NOT `Unknown(...)`).
///
/// Would-FAIL-on-revert: reverting the Row D-19 mint flips
/// `CompileError::Parse(d) => ErrorCode::DslParseError` back to
/// `ErrorCode::Unknown(d.error_code.to_string())` — this test's
/// `assert_eq!(..., ErrorCode::DslParseError)` then fails because
/// `ErrorCode::Unknown(_) != ErrorCode::DslParseError` (the
/// non-`Unknown` variants compare via `PartialEq` discriminant).
#[test]
fn parse_failure_surfaces_first_class_dsl_parse_error_catalog_variant() {
    // Unbalanced brace at the handler body — the parser refuses with
    // `CompileError::Parse(Diagnostic { error_code: E_DSL_PARSE_ERROR, .. })`.
    let bad_source = "handler 'oops' { read('post')";
    let err = compile_str(bad_source).expect_err("unbalanced brace must refuse");

    // Wire-string surface (pre-D-19 + post-D-19 both surface this).
    assert_eq!(err.error_code(), "E_DSL_PARSE_ERROR");

    // First-class catalog-variant surface (post-D-19 only; pre-D-19
    // would surface `ErrorCode::Unknown("E_DSL_PARSE_ERROR".into())`).
    assert_eq!(
        err.code(),
        ErrorCode::DslParseError,
        "Row D-19 G-COMP-1 wave: CompileError::Parse(_) MUST route to the \
         first-class ErrorCode::DslParseError catalog variant (pre-mint it \
         collapsed to ErrorCode::Unknown(\"E_DSL_PARSE_ERROR\"), defeating \
         discriminant-switching at the napi boundary).",
    );
}

/// Production entry point: feeding a DSL source with an unknown
/// primitive drives the parser's `parse_primitive` dispatcher into
/// the `other =>` arm that returns
/// `CompileError::Semantic(Diagnostic { error_code:
/// E_DSL_UNKNOWN_PRIMITIVE, .. })`. Post-Row-D-19 the typed `code()`
/// returns `ErrorCode::DslUnknownPrimitive`.
#[test]
fn unknown_primitive_surfaces_first_class_dsl_unknown_primitive_catalog_variant() {
    // `fooBar` is not in the 12-primitive set (CLAUDE.md #1).
    let bad_source = "handler 'oops' { fooBar() respond() }";
    let err = compile_str(bad_source).expect_err("unknown primitive must refuse");

    assert_eq!(err.error_code(), "E_DSL_UNKNOWN_PRIMITIVE");
    assert_eq!(
        err.code(),
        ErrorCode::DslUnknownPrimitive,
        "Row D-19 G-COMP-1 wave: CompileError::Semantic(_) MUST route to \
         the first-class ErrorCode::DslUnknownPrimitive catalog variant.",
    );
}

/// Production entry point: feeding a DSL handler missing RESPOND
/// drives the `emit` build-phase pass into `CompileError::Build(
/// Diagnostic { error_code: E_DSL_MISSING_RESPOND, .. })`. Post-Row-D-19
/// the typed `code()` returns `ErrorCode::DslMissingRespond` —
/// distinct from the sibling `E_DSL_INVALID_SHAPE` Build sub-case
/// which routes to `ErrorCode::DslInvalidShape`.
#[test]
fn missing_respond_surfaces_first_class_dsl_missing_respond_catalog_variant() {
    // Handler with one read primitive + no respond terminator.
    let bad_source = "handler 'oops' { read('post') }";
    let err = compile_str(bad_source).expect_err("missing respond must refuse");

    assert_eq!(err.error_code(), "E_DSL_MISSING_RESPOND");
    assert_eq!(
        err.code(),
        ErrorCode::DslMissingRespond,
        "Row D-19 G-COMP-1 wave: CompileError::Build(_) with \
         Diagnostic.error_code == E_DSL_MISSING_RESPOND MUST route to the \
         first-class ErrorCode::DslMissingRespond catalog variant.",
    );
}

/// Row D-19 atomic 4-surface §3.5g rename — wire-string check. The
/// catalog variant `ErrorCode::ViewStrategyReserved` MUST surface
/// `"E_VIEW_STRATEGY_RESERVED"` (the post-rename canonical name),
/// NOT the pre-rename `"E_VIEW_STRATEGY_C_RESERVED"`.
///
/// Would-FAIL-on-revert: reverting the rename flips the `as_str`
/// arm back to `"E_VIEW_STRATEGY_C_RESERVED"` — this test then
/// fails its `assert_eq!(..., "E_VIEW_STRATEGY_RESERVED")`.
#[test]
fn view_strategy_reserved_wire_string_matches_post_rename_canonical_name() {
    let code = ErrorCode::ViewStrategyReserved;
    assert_eq!(
        code.as_static_str(),
        "E_VIEW_STRATEGY_RESERVED",
        "Row D-19 G-COMP-1 wave (Cohort 8) renamed the wire string \
         E_VIEW_STRATEGY_C_RESERVED → E_VIEW_STRATEGY_RESERVED; \
         as_static_str MUST surface the post-rename canonical name.",
    );
    // Round-trip through the parse arm — the new wire string parses
    // to the renamed variant.
    let parsed: ErrorCode = "E_VIEW_STRATEGY_RESERVED"
        .parse()
        .expect("post-rename wire string must parse");
    assert_eq!(parsed, ErrorCode::ViewStrategyReserved);
}

/// CATALOG_VARIANT_COUNT pin — independent of the
/// `crates/benten-errors/tests/stable_shape.rs` test (which lives in
/// the upstream crate), this asserts that the 3 new DSL variants are
/// reachable + distinct from each other + distinct from the pre-existing
/// `ErrorCode::DslInvalidShape` / `DslUnregisteredHandler` /
/// `DslBackendRejected` / `DslIoError` family members. The 3 new
/// variants would be inaccessible if any of the lib.rs mints were
/// dropped on a partial-revert.
#[test]
fn three_new_dsl_catalog_variants_are_distinct_and_reachable() {
    let variants = [
        ErrorCode::DslParseError,
        ErrorCode::DslUnknownPrimitive,
        ErrorCode::DslMissingRespond,
    ];
    let prior = [
        ErrorCode::DslInvalidShape,
        ErrorCode::DslUnregisteredHandler,
        ErrorCode::DslBackendRejected,
        ErrorCode::DslIoError,
    ];

    // Distinctness within the new mints.
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "new DSL variants must be distinct from each other");
            }
        }
    }
    // Distinctness vs the pre-existing DSL family.
    for new_v in &variants {
        for old_v in &prior {
            assert_ne!(
                new_v, old_v,
                "Row D-19 new DSL variants must be distinct from the \
                 pre-existing DSL family ({old_v:?})",
            );
        }
    }

    // Each new variant has a unique wire string per the §3.5g
    // cross-language rule-mirror.
    let wire_strings: Vec<&'static str> = variants.iter().map(|v| v.as_static_str()).collect();
    assert_eq!(
        wire_strings,
        vec![
            "E_DSL_PARSE_ERROR",
            "E_DSL_UNKNOWN_PRIMITIVE",
            "E_DSL_MISSING_RESPOND",
        ],
    );
}
