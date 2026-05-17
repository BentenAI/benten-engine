//! refinement-audit #922 — `SubgraphSpec` field-set drift-defense pin.
//!
//! `benten_ivm::SubgraphSpec` carries a "MUST NOT remove or rename" canary
//! stability commitment (its rustdoc: "Subsequent iterations MAY add fields
//! (additive) but MUST NOT remove or rename"). Pre-#922 that commitment had
//! NO mechanical enforcement — a refactor that renamed/removed a field would
//! compile, silently breaking the Family B/C round-trip canary contract.
//!
//! `SubgraphSpec` is an internal kernel type with no cross-language (TS)
//! mirror, so the §3.5g cross-language-mirror discipline does not apply
//! directly. The in-lane equivalent is an EXHAUSTIVE field destructure: if
//! any field is removed or renamed, this test fails to compile (a hard
//! drift signal at `cargo build`), and an *additive* field also fails to
//! compile until this pin is consciously updated — making field-set
//! evolution a deliberate, reviewed act rather than a silent drift.

#![allow(clippy::unwrap_used)]

use benten_ivm::{LabelPattern, Projection, SubgraphSpec, TypedOutputProjection};

#[test]
fn subgraph_spec_field_set_is_pinned() {
    let spec = SubgraphSpec {
        view_id: "content_listing".to_string(),
        label_pattern: LabelPattern::exact("post"),
        projection: Projection::all_props(),
        typed_output_projection: None,
        self_referential: false,
        budget: Some(42),
    };

    // EXHAUSTIVE destructure — the `..`-free pattern is the drift trap.
    // Adding/removing/renaming a field breaks THIS line at compile time,
    // forcing a conscious canary-contract review (the "MUST NOT remove or
    // rename" commitment, mechanically enforced).
    let SubgraphSpec {
        view_id,
        label_pattern,
        projection,
        typed_output_projection,
        self_referential,
        budget,
    } = spec;

    assert_eq!(view_id, "content_listing");
    assert_eq!(label_pattern, LabelPattern::exact("post"));
    assert_eq!(projection, Projection::all_props());
    assert_eq!(typed_output_projection, None::<TypedOutputProjection>);
    assert!(!self_referential);
    assert_eq!(budget, Some(42));
}
