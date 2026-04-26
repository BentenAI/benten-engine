//! G12-B red-phase: compiler emits the **post-G12-D widened**
//! `SubgraphSpec.primitives` shape (per-primitive properties bag).
//!
//! Per plan §3.2 G12-B "DSL compiler emits the wider shape" + plan §3.2 G12-D
//! "Each new primitive needs the wider shape to declare its config" + sec-pre-r1-09
//! BTreeMap<String, Value> canonical-bytes-stable constraint.
//!
//! TDD red-phase: this test verifies that compile output structure carries
//! per-primitive properties, not the Phase-2a stub flat shape. Lifts to green
//! after BOTH G12-D widening landed AND G12-B compiler updated to emit it.
//!
//! Owner: R5 G12-B (depends on G12-D landing first; qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: widened-spec emission not yet implemented (gates on G12-D)"]
fn dsl_compiler_emits_subgraph_spec_with_per_primitive_props_bag() {
    // DSL with per-primitive config — e.g. WAIT with ttl_hours, SANDBOX with
    // wallclock_ms — round-trips through SubgraphSpec.primitives carrying
    // those props in the BTreeMap<String, Value> bag (D6 mild lean).
    let _src = r"handler 'with-wait' { read('post') -> wait({ ttl_hours: 24 }) -> respond }";
    todo!(
        "R5 G12-B: compile_str(src) -> spec; spec.primitives[1] is WAIT variant; \
           wait_props bag (BTreeMap<String, Value>) contains ttl_hours: 24"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: widened-spec emission not yet implemented (gates on G12-D)"]
fn dsl_compiler_emits_sandbox_primitive_with_wallclock_ms_prop() {
    let _src =
        r"handler 'with-sandbox' { sandbox('module-cid', { wallclock_ms: 30000 }) -> respond }";
    todo!("R5 G12-B: spec.primitives[0] is SANDBOX variant; bag contains wallclock_ms: 30000")
}

#[test]
#[ignore = "R5 G12-B red-phase: per-primitive property bag canonical-bytes pin (gates on G12-D)"]
fn dsl_compiler_widened_emission_canonical_bytes_stable_for_permuted_prop_keys() {
    // sec-pre-r1-09 carry: even if the DSL surface presents props in different
    // order, the emitted SubgraphSpec.primitives bag (BTreeMap sorted-by-key)
    // produces identical canonical bytes.
    let _src_a =
        r"handler 'h' { sandbox('m', { wallclock_ms: 30000, output_limit: 65536 }) -> respond }";
    let _src_b =
        r"handler 'h' { sandbox('m', { output_limit: 65536, wallclock_ms: 30000 }) -> respond }";
    todo!("R5 G12-B: compile both; assert canonical_bytes(spec_a) == canonical_bytes(spec_b)")
}
