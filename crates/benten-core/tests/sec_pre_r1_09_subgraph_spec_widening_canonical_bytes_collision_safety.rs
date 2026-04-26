//! G12-D sec-pre-r1-09 pin: assert the widened `SubgraphSpec.primitives` shape
//! produces collision-safe canonical bytes (Inv-13).
//!
//! Per `r1-security-auditor.json` sec-pre-r1-09 + plan §5 D6 ruling:
//!   "all R1-decision options MUST be canonical-bytes-stable (Inv-13
//!    collision-safe). CBOR-passthrough is RULED OUT pre-R1 — collision-prone
//!    (multiple wire-encodings of the same logical bag yield distinct CIDs
//!    without violating CBOR spec)."
//!
//! The mild-lean RECOMMEND option `BTreeMap<String, Value>` (sorted by key)
//! gives canonical bytes for free because BTreeMap iteration order is
//! lexicographic; this test pins the property:
//!
//! For two LOGICALLY identical per-primitive props bags built via permuted
//! input order, the resulting canonical bytes (and CID) MUST be identical.
//!
//! Counter-pin: CBOR-passthrough is OUT — no test exercises a passthrough path.
//!
//! TDD red-phase. Owner: R5 G12-D (qa-r4-03 sec-pre-r1-09 carry; R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::{Subgraph, SubgraphBuilder};

#[test]
#[ignore = "R5 G12-D red-phase: BTreeMap permuted-input canonical-bytes stability not yet asserted"]
fn widening_btreemap_canonical_bytes_stable_across_permuted_input_order_2_keys() {
    // Build a SANDBOX primitive with props {"a": 1, "b": 2}; build the same
    // logical primitive but insert "b" before "a"; assert canonical_bytes
    // equality across both builds. BTreeMap sorts by key so this should hold.
    let _build_a_then_b = SubgraphBuilder::new("perm-ab");
    let _build_b_then_a = SubgraphBuilder::new("perm-ba");

    todo!(
        "R5 G12-D: build sg_ab with props inserted (a, b); build sg_ba with (b, a); \
         assert sg_ab.canonical_bytes() == sg_ba.canonical_bytes()"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: 5-key permuted-input canonical-bytes stability not yet asserted"]
fn widening_btreemap_canonical_bytes_stable_across_permuted_input_order_5_keys() {
    // 5-key permutation: 5! = 120 orderings; spot-check 4 distinct orderings
    // produce identical canonical bytes.
    todo!(
        "R5 G12-D: build 4 sgs with the same 5-key bag inserted in different orders; \
         assert all canonical_bytes() are byte-identical"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: nested Value::Map canonical-bytes stability not yet asserted"]
fn widening_nested_value_map_canonical_bytes_stable_across_permuted_input() {
    // Inner Value::Map (BTreeMap-backed per benten-core conventions) also
    // sorts keys; pin the nested case.
    todo!(
        "R5 G12-D: build prop with Value::Map nested 3 levels deep; \
         assert canonical_bytes stability across permuted nested-map keys"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: typed-enum-bag option ruled out by CID equality (alternative path) not yet asserted"]
fn widening_uses_btreemap_path_not_typed_enum_path() {
    // Per D6 RECOMMEND lean: BTreeMap<String, Value>, NOT typed enum
    // PrimitiveProp::String/Cid/Number. This test inspects the type to confirm
    // the chosen path. Catches accidental switch to the typed-enum option.
    let _ = std::any::type_name::<Subgraph>();
    todo!(
        "R5 G12-D: type-name assertion that PrimitiveSpec.props field type \
         contains 'BTreeMap<' substring (NOT 'enum PrimitiveProp')"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: cbor-passthrough rejection not yet asserted"]
fn widening_rejects_cbor_passthrough_byte_blob_at_props_position() {
    // sec-pre-r1-09 explicit RULED OUT: CBOR-passthrough (raw `Bytes` carrying
    // a CBOR blob) at the props position. If R5 G12-D accidentally accepts a
    // passthrough variant, this test catches it.
    todo!(
        "R5 G12-D: attempt to construct PrimitiveSpec with raw CBOR-passthrough \
         bytes at the props position; assert build_validated returns an Err \
         (or the API simply doesn't expose this path — either passes the test)"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: cid stability across logical-equivalence permutations not yet asserted"]
fn widening_cid_stable_across_permuted_input_for_3_key_bag() {
    // Top-level pin tying Inv-13: cid(sg_perm_a) == cid(sg_perm_b) for
    // logically-identical inputs.
    todo!(
        "R5 G12-D: build sg_perm_a, sg_perm_b for 3-key bag in 2 different input orders; \
         assert sg_perm_a.cid().expect(\"cid\") == sg_perm_b.cid().expect(\"cid\")"
    )
}
