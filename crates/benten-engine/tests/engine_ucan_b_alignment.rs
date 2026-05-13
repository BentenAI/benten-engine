//! R3-A RED-PHASE pin: `Engine<B>` and `UCANBackend<B>` share the same
//! backend-generic parameter at construction time
//! (G13-B wave-2 + G14-B wave-4b; arch-r1-5).
//!
//! Pin source: r2-test-landscape §2.1 G13-B row
//! `engine_generic_b_aligns_with_ucan_backend_b_at_construction_time`;
//! arch-r1-5.
//!
//! ## What this pins (2-axis genericism per arch-r1-5)
//!
//! Per arch-r1-5: BOTH `Engine<B>` AND `UCANBackend<B>` are generic
//! over `B: GraphBackend`, AND at engine construction time the SAME
//! concrete `B` is passed to both. Phase-3 default: both = RedbBackend.
//!
//! Misalignment (e.g. `Engine<RedbBackend>` carrying a
//! `UCANBackend<BrowserBackend>`) would split the durable-grant store
//! from the durable-graph store, with cross-store consistency failures
//! at every grant lookup. The compile-time alignment pin defends
//! against that mistake.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — Engine<B> + UCANBackend<B> alignment. G13-B + G14-B both shipped (commits 4238ed7 + 496e144); test body pins Engine-UCAN-backend generic-alignment structural contract; un-ignore at Phase-4-Foundation pre-tag sweep per docs/future/phase-4-backlog.md §4.29 (HARD RULE 12 clause-(b))."]
fn engine_generic_b_aligns_with_ucan_backend_b_at_construction_time() {
    // G13-B + G14-B implementer wires this:
    //   use benten_engine::EngineGeneric;
    //   use benten_caps::backends::UCANBackend;
    //   use benten_graph::RedbBackend;
    //
    //   // Phase-3 default — both share RedbBackend:
    //   fn assert_aligned<B: benten_graph::GraphBackend>() {
    //       fn _ctor<B: benten_graph::GraphBackend>(
    //           graph: B,
    //           caps: UCANBackend<B>,
    //       ) -> EngineGeneric<B> {
    //           EngineGeneric::with_caps(graph, caps)
    //       }
    //   }
    //   assert_aligned::<RedbBackend>();
    //
    // OBSERVABLE consequence: a misaligned construction
    // `EngineGeneric::<RedbBackend>::with_caps(graph, ucan_browser)`
    // fails to compile because the type signature requires
    // `UCANBackend<RedbBackend>`. arch-r1-5 names this the load-bearing
    // anti-misalignment defense.
    unimplemented!("G13-B + G14-B wire 2-axis genericism alignment compile-time pin");
}
