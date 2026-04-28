//! G12-B green-phase: compiler emits the **post-G12-D widened**
//! per-primitive properties bag (BTreeMap<String, Value>) — sec-pre-r1-09
//! BTreeMap<String, Value> canonical-bytes-stable constraint.
//!
//! Lifted from red-phase 2026-04-28 (R5 G12-B implementer).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::Value;
use benten_dsl_compiler::{PrimitiveKind, compile_str};

#[test]
fn dsl_compiler_emits_subgraph_spec_with_per_primitive_props_bag() {
    let src = "handler 'with-wait' { read('post') -> wait({ ttl_hours: 24 }) -> respond }";
    let c = compile_str(src).unwrap();
    assert_eq!(c.primitives.len(), 3);
    let wait = &c.primitives[1];
    assert_eq!(wait.kind, PrimitiveKind::Wait);
    assert_eq!(wait.properties.get("ttl_hours"), Some(&Value::Int(24)));
}

#[test]
fn dsl_compiler_emits_sandbox_primitive_with_wallclock_ms_prop() {
    let src =
        "handler 'with-sandbox' { sandbox('module-cid', { wallclock_ms: 30000 }) -> respond }";
    let c = compile_str(src).unwrap();
    let sb = &c.primitives[0];
    assert_eq!(sb.kind, PrimitiveKind::Sandbox);
    assert_eq!(sb.properties.get("wallclock_ms"), Some(&Value::Int(30000)));
}

#[test]
fn dsl_compiler_widened_emission_canonical_bytes_stable_for_permuted_prop_keys() {
    // sec-pre-r1-09 carry: even if the DSL surface presents props in different
    // order, the emitted Subgraph properties bag (BTreeMap sorted-by-key)
    // produces identical canonical bytes.
    let src_a =
        "handler 'h' { sandbox('m', { wallclock_ms: 30000, output_limit: 65536 }) -> respond }";
    let src_b =
        "handler 'h' { sandbox('m', { output_limit: 65536, wallclock_ms: 30000 }) -> respond }";
    let a = compile_str(src_a).unwrap();
    let b = compile_str(src_b).unwrap();
    assert_eq!(
        a.subgraph.canonical_bytes().unwrap(),
        b.subgraph.canonical_bytes().unwrap()
    );
}
