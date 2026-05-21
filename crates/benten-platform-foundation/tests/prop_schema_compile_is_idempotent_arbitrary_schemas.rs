//! G-CORE-4 §4.6 GREEN landing of the schema-compile idempotency proptest.
//!
//! Post-G-CORE-4: the strict 4-of-4 input-dialect grammar is finalised
//! (FieldObject/FieldMap/FieldEnum/FieldUnion all accepted via
//! `parse_field` — see `crates/benten-platform-foundation/src/schema_compiler/parse.rs`
//! variant-string-shorthand widening + the four per-label fixtures in
//! `tests/common/schema_fixtures.rs`). With the dialect settled, the
//! arbitrary-schema generator drives valid-schema bytes selection across
//! all 8 vocabulary labels; each generated schema MUST compile twice
//! into byte-identical canonical Subgraph encodings (schema-compiler is
//! a pure function).
//!
//! Pin provenance: r2-test-landscape §2.4 row 11 + phase-4-backlog §4.6
//! (un-ignore destination cited in pre-G-CORE-4 ignore message). The
//! `tf5_46_sibling_arbitrary_schema_proptest_un_ignore_obligation_is_still_pending`
//! marker in `tf5_46_schema_compiler_8_labeltype_vocab_fixture.rs` is
//! updated in the same wave to assert this proptest is live + un-ignored.
//!
//! ## §3.6g inherited-discipline pre-flight (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): G-CORE-4 sweeps adjacent docs.
//! - [x] §3.6b + sub-rule-4 (pim-2): production runtime (`compile` over
//!   every generated valid schema) + observable consequence
//!   (canonical-bytes byte-equality) + would-FAIL (any non-pure
//!   compile arm regresses).
//! - [x] §3.6e (pim-12): un-ignored at G-CORE-4 per the §4.6 un-ignore
//!   destination.
//! - [x] §3.6f (pim-18): substantive proptest over the strict dialect
//!   generator — not a sentinel.
//! - [x] §3.5g: no cross-language mirror touched here.
//! - [x] §3.5i: file-disjoint from G-CORE-7.
//! - [x] §3.6h: no rule codifying an origin here.
//! - [x] §3.6i/§3.6j: N/A. §3.13: no per-test static introduced.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_core::canonical_subgraph_bytes;
use benten_platform_foundation::schema_compiler::compile;
use proptest::prelude::*;

/// Generator over the 8 valid-schema byte fixtures the G-CORE-4 strict
/// dialect supports (the 4 pre-G-CORE-4 + the 4 new per-label fixtures).
/// Each yields canonical-bytes-compatible schema JSON; the property
/// asserts compile() is a pure function over each input.
fn arbitrary_valid_schema_bytes() -> impl Strategy<Value = &'static [u8]> {
    prop_oneof![
        Just(schema_fixtures::minimal_schema_bytes()),
        Just(schema_fixtures::benign_schema_round_trip_bytes()),
        Just(schema_fixtures::canonical_note_type_schema_bytes()),
        Just(schema_fixtures::field_object_fixture_bytes()),
        Just(schema_fixtures::field_map_fixture_bytes()),
        Just(schema_fixtures::field_enum_fixture_bytes()),
        Just(schema_fixtures::field_union_fixture_bytes()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        // Modest case count — 8 fixtures × ~30 cases visits the
        // dialect surface several times each, keeping the proptest
        // wall-clock under ~1s on MSRV CI.
        cases: 64,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_schema_compile_is_idempotent_arbitrary_schemas(bytes in arbitrary_valid_schema_bytes()) {
        let spec1 = compile(bytes).expect("strict dialect fixture must compile");
        let spec2 = compile(bytes).expect("strict dialect fixture must compile");
        let bytes1 = canonical_subgraph_bytes(spec1.as_subgraph())
            .expect("canonical-bytes encoding must succeed for valid Subgraph");
        let bytes2 = canonical_subgraph_bytes(spec2.as_subgraph())
            .expect("canonical-bytes encoding must succeed for valid Subgraph");
        prop_assert_eq!(
            bytes1,
            bytes2,
            "schema_compiler::compile must be a pure function — repeat \
             compiles of the same bytes must yield byte-identical \
             canonical-Subgraph encodings (§4.6 idempotency)",
        );
    }
}
