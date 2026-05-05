//! R3-B RED-PHASE pin: SUBSCRIBE handler-id-router routes change
//! events through named handler (G14-D wave-5a; seq-major-8 LOAD-BEARING).
//!
//! Pin source: r2-test-landscape §2.2 G14-D + §3.D producer-consumer
//! parity meta-tests cluster:
//!
//! - `tests/subscribe_handler_id_router_routes_change_event_through_named_handler` — seq-major-8 LOAD-BEARING
//!
//! ## Architectural intent
//!
//! Per seq-major-8 (R1 sequencing-systems lens), the SUBSCRIBE handler-
//! id-router is the load-bearing seam where change events route
//! through a NAMED handler (rather than fan-out to every default
//! consumer). Without this seam, `EMIT` and `SUBSCRIBE` produce
//! observable behavior that differs only by name; the router is what
//! makes them semantically distinct producer/consumer surfaces.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the test must drive the
//! production routing entry point + assert the change event
//! observably arrives at the named handler (not the default
//! fan-out).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — seq-major-8 LOAD-BEARING — handler-id-router routes change events"]
fn subscribe_handler_id_router_routes_change_event_through_named_handler() {
    // seq-major-8 LOAD-BEARING pin. G14-D implementer wires this:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let named_handler_id = "demo:on_post_create";
    //
    //   // Register the named handler subgraph (TRANSFORM that records
    //   // its invocation into a probe Node):
    //   let handler_subgraph = subgraph_recording_invocation_to_probe();
    //   engine.register_subgraph(named_handler_id, handler_subgraph).unwrap();
    //
    //   // Subscribe with explicit handler-id routing:
    //   engine.subscribe_with_handler(
    //       "/zone/posts",
    //       benten_engine::HandlerRoute::Named(named_handler_id.to_string()),
    //   ).unwrap();
    //
    //   // Trigger a change event:
    //   engine.write_node(&post_node).unwrap();
    //
    //   // The named handler observably ran (probe Node was written):
    //   let probe_count = engine.read_zone("/zone/probe").unwrap().len();
    //   assert!(probe_count > 0,
    //       "handler-id-router must route the change event through the named handler");
    //
    //   // Crucially — the default fan-out delivery DID NOT fire (a
    //   // sibling default consumer would have been a no-op):
    //   let default_fan_out_invocations = engine.metrics().default_fan_out_count();
    //   assert_eq!(default_fan_out_invocations, 0);
    //
    // OBSERVABLE consequence: the change event observably arrives at
    // the named handler subgraph, NOT the default fan-out path. Per
    // §3.D parity meta-tests, this is structural fix for the 24-
    // instance producer/consumer recurrence — the router seam is
    // what makes SUBSCRIBE / EMIT distinguishable at runtime.
    unimplemented!(
        "G14-D wires handler-id-router that routes SUBSCRIBE change events through named handler subgraph"
    );
}
