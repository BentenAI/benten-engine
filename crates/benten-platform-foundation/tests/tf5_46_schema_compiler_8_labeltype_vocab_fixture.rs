//! ADDL R3 (TDD red-phase) — Phase-4-Meta-Core, Wave R3-B, agent R3-B2,
//! family **TF-5**. RED-phase pin for the **§4.6 G23-A strict 4-of-4
//! input-dialect validation + arbitrary-schema proptest + the
//! 8-`LabelType` vocab-fixture** (C4-adjacent; G-CORE-4 substrate).
//!
//! ## Pin provenance
//!
//! - R2 `.addl/phase-4-meta/r2-test-landscape.md` TF-5 ("the §4.6
//!   G23-A strict 4-of-4 input-dialect validation + arbitrary-schema
//!   proptest + the 8-`LabelType` vocab-fixture (the 4 missing
//!   FieldObject/FieldMap/FieldEnum/FieldUnion variants + per-label
//!   emit-side construction-site assertions; the `#[ignore]`d proptest
//!   un-ignored per §3.6e — reviewer verifies landing-status)") + §2.B
//!   "§4.6 `#[ignore]`d proptest un-ignore" row + §4-D §3.6e row.
//! - Plan G-CORE-4 group def ("§4.6 G23-A strict 4-of-4 input-dialect
//!   validation + arbitrary-schema proptest + the 8-`LabelType`
//!   vocab-fixture (the 4 missing ... per §3.6e — reviewer verifies
//!   landing-status)").
//! - Named destination `docs/future/phase-4-backlog.md §4.6`
//!   acceptance-criteria addendum (schema-lang-r6-r2-2): the
//!   vocab-fixture coverage test pin
//!   `schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension.rs`
//!   currently exercises only 4 of the 8 declared `LabelType` variants
//!   (SchemaRoot / FieldScalar / FieldList / FieldRef). The remaining 4
//!   (FieldObject / FieldMap / FieldEnum / FieldUnion) MUST be added to
//!   the fixture set when this row lands, alongside per-label
//!   assertions that the emit-side construction-site fires for each
//!   label and produces the corresponding vocab edges (implicit-via-
//!   recursion parent→child for objects; KEY_TYPE+VALUE_TYPE for maps;
//!   VARIANT for enums/unions).
//!
//! ## Ground-truth at HEAD `ed03729a`
//!
//! - The 8 `VocabLabel` variants ARE already defined in
//!   `crates/benten-platform-foundation/src/schema_compiler/vocab.rs`
//!   (`SchemaRoot` … `FieldUnion`) — the GAP is test-side coverage:
//!   the existing pin exercises only 4 of 8.
//! - The §4.6 arbitrary-schema proptest is the still-`#[ignore]`d
//!   `crates/benten-platform-foundation/tests/prop_schema_compile_is_idempotent_arbitrary_schemas.rs`
//!   (ignore message names `§4.6` as the un-ignore destination — it
//!   needs the strict 4-of-4 input-dialect grammar finalized first).
//!   §3.6e RED-PHASE staged-pin: the G-CORE-4 implementer un-ignores
//!   THAT file when the strict dialect lands; the reviewer verifies
//!   landing-status, not just spec-pin presence. THIS file carries the
//!   un-ignore obligation marker so it cannot be silently skipped.
//!
//! ## §3.6b sub-rule-4 production-arm shape
//!
//! - PRODUCTION RUNTIME ARM: per-label fixtures exercising EACH of the
//!   8 `VocabLabel` variants are `compile`d via the production
//!   `benten_platform_foundation::schema_compiler::compile`, including
//!   the 4 missing FieldObject/FieldMap/FieldEnum/FieldUnion.
//! - OBSERVABLE CONSEQUENCE: each label compiles to a Subgraph whose
//!   primitives are all within the canonical 12 (no new PrimitiveKind)
//!   AND the per-label emit-side construction-site fires the expected
//!   vocab edge(s) (FieldMap → KEY_TYPE+VALUE_TYPE; FieldEnum/FieldUnion
//!   → VARIANT; FieldObject → implicit-via-recursion child edges).
//! - WOULD-FAIL-IF-NO-OP: dropping a per-label emit arm in
//!   `emit_vocabulary_edges` drops the corresponding edge count to zero
//!   for that label's fixture — the per-label edge assertion fires.
//!
//! ## SHAPE-FLAG (not faked)
//!
//! The per-label FieldObject/FieldMap/FieldEnum/FieldUnion fixtures + a
//! `compile`-then-assert-edges harness depend on the strict 4-of-4
//! input-dialect grammar being finalized (the §4.6 carry-criterion).
//! Until G-CORE-4 lands that, the 8-of-8 coverage test is `#[ignore]`d
//! (§3.6e). The substantive shape (per-label compile + per-label edge
//! assertion) is encoded so the implementer wires it against an
//! already-correct substantive baseline, not a sentinel.
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): tests only; G-CORE-4 sweeps docs.
//! - [x] §3.6b + sub-rule-4: per-label compile + per-label edge SPECIFIC arm; NOT an umbrella "vocab composes" sentinel.
//! - [x] §3.6e (pim-12): RED-PHASE staged-pin; this file ALSO names the sibling `prop_schema_compile_is_idempotent_arbitrary_schemas.rs` un-ignore obligation explicitly so it is not silently skipped; reviewer verifies landing-status of BOTH.
//! - [x] §3.6f (pim-18): SHAPE-not-SUBSTANCE — per-label edge-count assertions, not "8 labels enumerated".
//! - [x] §3.5g: VOCAB_* consts are a cross-side mirror (helper ↔ schema_compiler vocab); the existing fixture pin already grep-asserts the lengths — this file preserves that mirror, changes neither side.
//! - [x] §3.5i: benten-platform-foundation/tests SCHEMA-COMPILER-VOCAB lane (TF-5). Disjoint from R3-B5 (engine/caps) + R3-B4 (plugin_lifecycle/manifest_store). New file, no shared path.
//! - [x] §3.6h: no rule-naming-origin codified here.
//! - [x] §3.6i/§3.6j: N/A. §3.13: no shared static introduced.
//! - [x] §3.6g: this checklist IS the literal reproduction.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

/// The 4 currently-uncovered label variants (the §4.6 acceptance gap).
const MISSING_FOUR_LABELS: [&str; 4] = ["FieldObject", "FieldMap", "FieldEnum", "FieldUnion"];

/// RUNNABLE guard (NOT ignored): the vocab-mirror invariant + the gap
/// statement. Asserts the 8-label set is declared (helper-side mirror)
/// and that the 4 missing labels are a strict subset of it — so the
/// G-CORE-4 implementer has an unambiguous, machine-checked statement
/// of exactly which 4 fixtures must be added. Would-FAIL now only if
/// the vocab mirror drifts (a real cross-side drift guard, §3.5g).
#[test]
fn tf5_46_eight_label_vocab_mirror_and_missing_four_are_a_subset() {
    assert_eq!(
        schema_fixtures::VOCAB_LABELS.len(),
        8,
        "8-label vocabulary (D-4F-NEW-TYPED-FIELD-NODE-VOCAB) — helper \
         mirror must stay at 8"
    );
    assert_eq!(schema_fixtures::VOCAB_EDGES.len(), 5, "5 labeled edges");
    assert_eq!(schema_fixtures::VOCAB_SCALARS.len(), 8, "8 scalars");
    assert_eq!(
        schema_fixtures::VOCAB_FIELD_PROPS.len(),
        4,
        "4 mandatory field properties"
    );
    for missing in MISSING_FOUR_LABELS {
        assert!(
            schema_fixtures::VOCAB_LABELS.contains(&missing),
            "§4.6 acceptance gap: `{missing}` is one of the 4 labels the \
             existing vocab-fixture pin does NOT exercise (it covers \
             only SchemaRoot/FieldScalar/FieldList/FieldRef); it MUST be \
             in the declared 8-label vocabulary"
        );
    }
}

/// Post-G-CORE-4 GREEN landing: per-label emit-side construction-site
/// assertions for all 8 vocabulary labels (the 4 existing
/// SchemaRoot/FieldScalar/FieldList/FieldRef PLUS the 4 missing
/// FieldObject/FieldMap/FieldEnum/FieldUnion). For each fixture:
///   (a) the schema compiles (12-primitive irreducibility — no new
///       `PrimitiveKind` variant was minted to absorb the new labels);
///   (b) the per-label emit-side construction site fired the expected
///       vocab edge(s) — FieldMap → KEY_TYPE + VALUE_TYPE;
///       FieldEnum/FieldUnion → VARIANT; FieldObject →
///       implicit-via-recursion child edges (FieldScalar nested under
///       the FieldObject root).
#[test]
#[allow(clippy::too_many_lines)]
fn tf5_46_schema_compiler_vocab_fixture_covers_all_8_labeltype_variants_with_per_label_edges() {
    use benten_core::PrimitiveKind;
    use benten_platform_foundation::schema_compiler::{compile, vocab::VocabEdge};
    use std::collections::HashSet;

    // 12-primitive irreducibility allowlist (CLAUDE.md baked-in #1).
    let allowed: HashSet<PrimitiveKind> = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ]
    .into_iter()
    .collect();

    // The 8 per-label fixtures. The 4 missing labels each have a
    // dedicated fixture added at G-CORE-4 (see
    // `common/schema_fixtures.rs` for the FieldObject/FieldMap/FieldEnum/
    // FieldUnion byte literals); the existing 4 reuse pre-G-CORE-4
    // fixtures that already exercise their label.
    let fixtures: [(&str, &[u8]); 8] = [
        ("SchemaRoot", schema_fixtures::minimal_schema_bytes()),
        ("FieldScalar", schema_fixtures::minimal_schema_bytes()),
        (
            "FieldList",
            schema_fixtures::benign_schema_round_trip_bytes(),
        ),
        (
            "FieldRef",
            schema_fixtures::canonical_note_type_schema_bytes(),
        ),
        ("FieldObject", schema_fixtures::field_object_fixture_bytes()),
        ("FieldMap", schema_fixtures::field_map_fixture_bytes()),
        ("FieldEnum", schema_fixtures::field_enum_fixture_bytes()),
        ("FieldUnion", schema_fixtures::field_union_fixture_bytes()),
    ];

    // Sanity: the 4 missing labels are covered by the fixture set.
    for missing in MISSING_FOUR_LABELS {
        assert!(
            fixtures.iter().any(|(l, _)| *l == missing),
            "§4.6 G-CORE-4 deliverable: per-label fixture for `{missing}` \
             must be present in the 8-tuple"
        );
    }

    for (label, bytes) in fixtures {
        let spec = compile(bytes).unwrap_or_else(|e| {
            panic!("{label} fixture must compile (G-CORE-4 §4.6 strict dialect): {e:?}")
        });
        // (a) 12-primitive irreducibility (no new PrimitiveKind).
        for p in spec.primitives() {
            assert!(
                allowed.contains(&p.kind()),
                "{label} compiled to non-canonical PrimitiveKind {:?}; \
                 CLAUDE.md baked-in #1 — 12 primitives are irreducible",
                p.kind()
            );
        }

        // (b) per-label emit-side construction-site fired the expected
        //     vocab edge(s). Grep the spec's subgraph edges by label.
        let edge_labels: HashSet<&str> = spec
            .as_subgraph()
            .edges()
            .iter()
            .map(|(_, _, l)| l.as_str())
            .collect();

        match label {
            "FieldMap" => {
                assert!(
                    edge_labels.contains(VocabEdge::KeyType.as_str()),
                    "FieldMap fixture must emit KEY_TYPE edge; saw: {edge_labels:?}"
                );
                assert!(
                    edge_labels.contains(VocabEdge::ValueType.as_str()),
                    "FieldMap fixture must emit VALUE_TYPE edge; saw: {edge_labels:?}"
                );
            }
            "FieldEnum" | "FieldUnion" => {
                assert!(
                    edge_labels.contains(VocabEdge::Variant.as_str()),
                    "{label} fixture must emit VARIANT edge; saw: {edge_labels:?}"
                );
            }
            "FieldObject" => {
                // FieldObject is implicit-via-recursion: there is no
                // FIELD edge; the assertion is that the FieldObject
                // root's child Field* primitives appear in the
                // emitted Subgraph nodes (proves the recursion fired).
                // The `WithObject` fixture has a FieldScalar child
                // `nickname`; assert it shows up via field-path.
                let has_child_field_path = spec.as_subgraph().nodes().iter().any(|op| {
                    op.property(
                        benten_platform_foundation::schema_compiler::FIELD_PATH_PROPERTY_KEY,
                    )
                    .and_then(|v| match v {
                        benten_core::Value::Text(s) => Some(s.contains('.')),
                        _ => None,
                    })
                    .unwrap_or(false)
                });
                assert!(
                    has_child_field_path,
                    "FieldObject fixture must recurse into a nested child \
                     field (dotted field_path expected)"
                );
            }
            "FieldList" => {
                assert!(
                    edge_labels.contains(VocabEdge::ItemType.as_str()),
                    "FieldList fixture must emit ITEM_TYPE edge; saw: {edge_labels:?}"
                );
            }
            "FieldRef" => {
                assert!(
                    edge_labels.contains(VocabEdge::RefTarget.as_str()),
                    "FieldRef fixture must emit REF_TARGET edge; saw: {edge_labels:?}"
                );
            }
            _ => { /* SchemaRoot + FieldScalar: presence-only */ }
        }
    }
    // Marker used to silence the dead-code warning when the loop
    // happens to enumerate the MISSING_FOUR_LABELS implicitly above.
    let _ = MISSING_FOUR_LABELS;
}

/// §3.6e un-ignore-obligation MARKER for the sibling §4.6 proptest —
/// POST-G-CORE-4 DISCHARGED.
///
/// G-CORE-4 un-ignored
/// `crates/benten-platform-foundation/tests/prop_schema_compile_is_idempotent_arbitrary_schemas.rs`
/// and wired its `arbitrary_valid_schema_bytes()` generator across all 8
/// strict-dialect fixtures (the 4 pre-G-CORE-4 + the 4 new per-label
/// fixtures from `common/schema_fixtures.rs`). This marker now asserts
/// the obligation has been DISCHARGED (the proptest is live, NOT
/// `#[ignore]`d) and that the §4.6 cite is preserved for traceability.
///
/// If this assert fails because the proptest got re-ignored, an §3.6e
/// regression occurred — STOP. If it fails because the §4.6 cite was
/// dropped, the named-destination was severed (HARD RULE 12 clause-(b)
/// violation) — STOP.
#[test]
fn tf5_46_sibling_arbitrary_schema_proptest_un_ignore_obligation_is_discharged() {
    use std::path::Path;
    let proptest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("prop_schema_compile_is_idempotent_arbitrary_schemas.rs");
    assert!(
        proptest_path.exists(),
        "§4.6 obligation target must exist: {}",
        proptest_path.display()
    );
    let src = std::fs::read_to_string(&proptest_path).unwrap();
    // The proptest must NOT be `#[ignore]`d post-G-CORE-4. We tolerate
    // the substring `#[ignore` only inside doc-comments / string
    // literals that NAME the historical RED-PHASE marker; gate on the
    // attribute form `#[ignore` appearing on a line NOT inside `//!`/`///`.
    let still_ignored_as_attr = src.lines().any(|l| {
        let trimmed = l.trim_start();
        !trimmed.starts_with("//") && trimmed.starts_with("#[ignore")
    });
    let cites_46 = src.contains("§4.6");
    assert!(
        !still_ignored_as_attr,
        "§3.6e RED-PHASE staged-pin obligation discharged at G-CORE-4: \
         `prop_schema_compile_is_idempotent_arbitrary_schemas.rs` must \
         NOT carry an `#[ignore]` attribute post-G-CORE-4. If you see \
         this fail, the proptest was re-ignored — §3.6e regression."
    );
    assert!(
        cites_46,
        "§4.6 named-destination traceability: the un-ignored proptest \
         must still cite `§4.6` so the historical destination is \
         preserved (HARD RULE 12 clause-(b))."
    );
}
