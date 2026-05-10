//! R3-B RED-PHASE pins: EMIT handler-id-router (G14-D wave-5a;
//! seq-major-8 + stream-r1-2).
//!
//! Pin sources (per r2-test-landscape §2.2 G14-D + §3.D parity cluster):
//!
//! - `tests/emit_handler_id_router_routes_emit_event_through_named_handler` — seq-major-8
//! - `tests/emit_handler_id_router_routing_observably_differs_from_default_fan_out_end_to_end` — stream-r1-2
//!
//! ## Architectural intent
//!
//! Sibling to `subscribe_handler_id_router_routes_change_event_through_named_handler`
//! (R3-B's `subscribe_handler_router.rs`). The same handler-id-router
//! routes EMIT events through a named handler. Per stream-r1-2 the
//! ROUTING must observably differ from the default fan-out — this is
//! the load-bearing structural property that closes the producer/
//! consumer drift recurrence at the runtime layer.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the un-ignored test must
//! drive the production EMIT entry point + assert observable
//! behavioral difference vs default fan-out.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §2.3 (i) WriteContext audience + clock threading — production prerequisite NOT YET shipped at HEAD (NAMED destination = v1-assessment-window per CLAUDE.md item #15). Sibling rationale to `crates/benten-eval/tests/subscribe_handler_router.rs::subscribe_handler_id_router_routes_change_event_through_named_handler`. Handler-id-router seam (`HandlerRoute::Named`) shipped at G14-D wave-5a + GREEN sentinel-presence at `crates/benten-engine/tests/g14_d_wave_5a_closed_claims.rs::emit_handler_id_router_routing_observably_differs_from_default_fan_out_end_to_end`; deeper end-to-end through actual handler-execution requires WriteContext threading axes (1)/(2)/(3) per §2.3 (i). Un-ignore at v1-assessment-window outcome. CROSS-CRATE TEST-NAME-DUPLICATION: distinct from the GREEN engine-side pin (different test name; sibling-shape)."]
fn emit_handler_id_router_routes_emit_event_through_named_handler() {
    // seq-major-8 pin. G14-D implementer wires this:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let named_handler_id = "demo:on_create_post_emit";
    //
    //   // Register the named handler subgraph:
    //   engine.register_subgraph(named_handler_id, handler_subgraph).unwrap();
    //
    //   // EMIT with explicit handler-id routing:
    //   engine.emit_with_handler(
    //       "create_post",
    //       &event_payload,
    //       benten_engine::HandlerRoute::Named(named_handler_id.to_string()),
    //   ).unwrap();
    //
    //   // Named handler observably ran:
    //   let probe_count = engine.read_zone("/zone/probe").unwrap().len();
    //   assert!(probe_count > 0);
    //
    //   // Default fan-out did NOT fire:
    //   let default_count = engine.metrics().default_fan_out_count();
    //   assert_eq!(default_count, 0);
    //
    // OBSERVABLE consequence: EMIT routes through the named handler
    // when HandlerRoute::Named is specified.
    unimplemented!("G14-D wires EMIT handler-id-router routing emit-events through named handler");
}

#[test]
#[ignore = "phase-3-backlog §2.3 (i) WriteContext threading — production prerequisite NOT YET shipped at HEAD. The end-to-end execution-trace differential (Named route vs DefaultFanOut route producing observably different probe-Node side-effects) requires actual handler EXECUTION which requires WriteContext threading per §2.3 (i). The seam-level differential is GREEN at `crates/benten-engine/tests/g14_d_wave_5a_closed_claims.rs::emit_handler_id_router_routing_observably_differs_from_default_fan_out_end_to_end` (which checks `log.default_fan_out_count()` vs `log.named_routes()` — the route-log differential, not the runtime-execution differential). Un-ignore at §2.3 (i) v1-assessment-window outcome. NAME-DUPLICATION: identical to the engine-side GREEN pin (drift hazard per pim-7 §3.5 dim #5; the eval-side pin is the FUTURE runtime-execution differential the engine-side pin's route-log assertion doesn't cover)."]
fn emit_handler_id_router_routing_observably_differs_from_default_fan_out_end_to_end() {
    // stream-r1-2 LOAD-BEARING pin. The router must produce
    // OBSERVABLY DIFFERENT execution traces depending on the
    // HandlerRoute variant. Without this difference, the router seam
    // is decorative and the producer/consumer drift recurrence
    // continues at runtime.
    //
    // Implementer wires:
    //
    //   // Setup: register two named handlers with distinguishable side-effects.
    //   engine.register_subgraph("h_a", subgraph_writes_probe("A"));
    //   engine.register_subgraph("h_b", subgraph_writes_probe("B"));
    //   // Default fan-out registers handler "default" via emit fanOut config.
    //   engine.register_subgraph("default", subgraph_writes_probe("default"));
    //
    //   // EMIT via named route → only h_a fires:
    //   engine.emit_with_handler("evt", &payload,
    //       benten_engine::HandlerRoute::Named("h_a".into())).unwrap();
    //   assert_eq!(probe_log(), vec!["A"]);
    //   reset_probe_log();
    //
    //   // EMIT via default fan-out → only "default" fires:
    //   engine.emit_with_handler("evt", &payload,
    //       benten_engine::HandlerRoute::DefaultFanOut).unwrap();
    //   assert_eq!(probe_log(), vec!["default"]);
    //
    //   // Critically: the two routes produce OBSERVABLY DIFFERENT
    //   // execution traces. Without this, the seam is purely
    //   // notational and silently no-ops.
    //
    // OBSERVABLE consequence: end-to-end execution-trace difference
    // between Named(h_a) and DefaultFanOut routes; closes pim-2-shape
    // sentinel-presence concern at the routing layer.
    unimplemented!(
        "G14-D wires end-to-end execution-trace difference between Named and DefaultFanOut routes"
    );
}
