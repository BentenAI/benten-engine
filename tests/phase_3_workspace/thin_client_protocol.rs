//! R3-E RED-PHASE pins for thin-client protocol shape (D-PHASE-3-N +
//! CLAUDE.md baked-in #17).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §3.E):
//!
//! - `tests/integration/atrium_browser_tab_as_thin_client_view_into_full_peer_e2e` — D-PHASE-3-N + exit-criterion 19
//! - `tests/browser_tab_thin_client_authenticated_view_into_full_peer` — exit-criterion 19
//! - thin-client protocol-shape pins for fetch / SSE / device-DID auth
//!
//! ## What thin-client protocol establishes (D-PHASE-3-N)
//!
//! Per CLAUDE.md baked-in #17: browser tabs + Phase-9+ edge workers are
//! authenticated **thin-client views** into full peers — NOT full peers
//! themselves. The protocol shape:
//!
//! - **Snapshot reads**: HTTP fetch GET against the full-peer endpoint
//! - **Writes**: HTTP POST with auth (UCAN delegation chain) against the
//!   full-peer endpoint
//! - **Device-DID auth**: every authenticated request carries the
//!   thin-client's device-DID in the auth header
//! - **Change events**: Server-Sent Events (SSE) OR WebSocket stream from
//!   the full peer; per F6 SUBSCRIBE filtering applied at the thin-client
//!   edge by the full peer (NOT at the thin-client itself)
//!
//! ## Co-ownership per r2-test-landscape §13 ambiguous-ownership
//!
//! - R3-B owns full-peer-side filtering pin
//! - **R3-D** owns browser-side IndexedDB cache extension
//! - **R3-E (this file): protocol-shape pins** — fetch / POST auth / SSE
//!   subscription / device-DID auth header (the wire-protocol contracts)
//!
//! Disjoint test-fn ownership within the integration target.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D + G18-A — thin-client snapshot read via fetch GET (D-PHASE-3-N)"]
fn thin_client_snapshot_read_via_fetch_get_against_full_peer_endpoint() {
    // D-PHASE-3-N protocol-shape pin per baked-in #17. G14-D + G18-A
    // implementer wires this:
    //
    //   // Stand up a full-peer engine + bind it to a local HTTP listener:
    //   let full_peer = benten_engine::Engine::open_in_memory().unwrap();
    //   let server_addr = full_peer.testing_bind_thin_client_endpoint().unwrap();
    //
    //   // Drive a snapshot read via plain fetch GET (the thin-client
    //   // wire shape; in production a browser tab does this):
    //   let url = format!("http://{}/v1/snapshot/{}", server_addr, "post:1");
    //   let response = ureq::get(&url).call().unwrap();
    //
    //   assert_eq!(response.status(), 200,
    //       "thin-client snapshot fetch GET must succeed");
    //   assert_eq!(response.header("content-type"),
    //       Some("application/dag-cbor"),
    //       "snapshot wire format must be DAG-CBOR per content-addressing");
    //
    // OBSERVABLE consequence: thin-client snapshot reads work via plain
    // HTTP fetch — no Loro/iroh required at the thin client per
    // baked-in #17.
    unimplemented!("G14-D + G18-A wires thin-client snapshot fetch GET protocol pin");
}

#[test]
#[ignore = "RED-PHASE: G14-D + G18-A — thin-client write via POST with device-DID auth header (D-PHASE-3-N)"]
fn thin_client_write_via_post_with_device_did_auth_header() {
    // D-PHASE-3-N + Inv-14 device-grain pin. Implementer wires this:
    //
    //   // Auth header contract: Authorization: Bearer <UCAN-delegation-chain>
    //   //   where the UCAN claim envelope binds the request to a specific
    //   //   device-DID. Defends against cross-device replay.
    //
    //   let device_did = "did:key:zEXAMPLE_DEVICE_DID";
    //   let ucan_chain = build_test_ucan_chain_for_device(device_did);
    //
    //   let url = format!("http://{}/v1/write/post", server_addr);
    //   let body = serde_json::json!({"title": "from thin client"});
    //   let response = ureq::post(&url)
    //       .set("Authorization", &format!("Bearer {}", ucan_chain))
    //       .set("Content-Type", "application/json")
    //       .send_string(&body.to_string())
    //       .unwrap();
    //
    //   assert_eq!(response.status(), 200);
    //
    //   // OBSERVABLE consequence: the write landed AND the attribution
    //   // frame on the resulting node carries the device-DID:
    //   let attribution = full_peer.testing_last_write_attribution_frame();
    //   assert_eq!(attribution.device_did, device_did,
    //       "Inv-14 device-DID grain must propagate from thin-client write");
    //
    // Defends against the failure mode where thin-client writes lose
    // device-DID attribution at the protocol boundary.
    unimplemented!("G14-D + G18-A wires thin-client POST + device-DID auth header pin");
}

#[test]
#[ignore = "RED-PHASE: G14-D + G18-A — thin-client SSE/WebSocket ChangeEvent stream (D-PHASE-3-N)"]
fn thin_client_change_event_stream_via_sse_or_websocket_from_full_peer() {
    // D-PHASE-3-N + F6 SUBSCRIBE pin. Implementer wires this:
    //
    //   // Thin-client subscribes to changes via SSE (Server-Sent Events)
    //   // OR WebSocket. The full peer applies F6 SUBSCRIBE filtering at
    //   // the edge (per-subscriber cap-recheck, NOT at the thin-client
    //   // itself).
    //
    //   let device_did = "did:key:zEXAMPLE_DEVICE_DID";
    //   let ucan_chain = build_test_ucan_chain_for_device(device_did);
    //
    //   // Open SSE stream:
    //   let url = format!("http://{}/v1/subscribe?label=post", server_addr);
    //   let mut req = ureq::get(&url)
    //       .set("Authorization", &format!("Bearer {}", ucan_chain))
    //       .set("Accept", "text/event-stream");
    //   let response = req.call().unwrap();
    //   let mut reader = response.into_reader();
    //
    //   // Trigger a change on the full peer:
    //   full_peer.call(post_sg, "post:create", json!({"title": "live"})).unwrap();
    //
    //   // Read SSE event:
    //   let mut buf = String::new();
    //   let mut chunk = [0u8; 4096];
    //   reader.read(&mut chunk).unwrap();
    //   buf.push_str(std::str::from_utf8(&chunk).unwrap());
    //   assert!(buf.contains("event: change") || buf.contains("data:"),
    //       "thin-client must receive SSE-formatted change event");
    //
    //   // OBSERVABLE consequence: F6 filtering applied at full-peer
    //   // edge: a different device-DID with no cap to "post" zone gets
    //   // EMPTY stream instead of seeing the change.
    //
    // Defends against the failure mode where thin-clients see all
    // changes (cross-trust-boundary leak).
    unimplemented!("G14-D + G18-A wires thin-client SSE change event protocol pin");
}

#[test]
#[ignore = "RED-PHASE: G14-D + G18-A — F6 SUBSCRIBE filtering applied at thin-client edge by full peer (D-PHASE-3-N)"]
fn thin_client_f6_subscribe_filtering_applied_at_full_peer_edge_not_thin_client() {
    // D-PHASE-3-N + exit-criterion 3 pin. Implementer wires this:
    //
    //   // Thin client A has cap to "post" zone only; thin client B has
    //   // cap to "user" zone only. Both subscribe to ALL changes via
    //   // SSE; the full peer authoritatively filters at delivery.
    //
    //   let stream_a = thin_client_subscribe_all(server_addr, ucan_a);
    //   let stream_b = thin_client_subscribe_all(server_addr, ucan_b);
    //
    //   // Trigger changes on both zones:
    //   full_peer.call(post_sg, "post:create", json!({})).unwrap();
    //   full_peer.call(user_sg, "user:create", json!({})).unwrap();
    //
    //   // OBSERVABLE consequence: filtering at full-peer edge:
    //   //   stream_a sees ONLY post change
    //   //   stream_b sees ONLY user change
    //   let events_a = read_sse_events(stream_a, /* timeout */);
    //   let events_b = read_sse_events(stream_b, /* timeout */);
    //   assert!(events_a.iter().all(|e| e.label == "post"),
    //       "thin-client A must only see post changes (its cap)");
    //   assert!(events_b.iter().all(|e| e.label == "user"),
    //       "thin-client B must only see user changes (its cap)");
    //
    // Defends against the failure mode where the thin-client itself
    // receives all changes + filters client-side (a confused-deputy
    // / cross-trust-boundary leak).
    unimplemented!("G14-D + G18-A wires thin-client F6 filtering protocol pin");
}

#[test]
#[ignore = "RED-PHASE: G14-D + G18-A — browser tab as thin-client view into full peer end-to-end (exit-criterion 19)"]
fn integration_atrium_browser_tab_as_thin_client_view_into_full_peer_e2e() {
    // exit-criterion 19 pin (the LOAD-BEARING end-to-end thin-client
    // browser test). Implementer wires this:
    //
    //   // Boot a full-peer engine on Node + open a browser tab via
    //   // Playwright that connects as a thin client. Drive an end-to-end
    //   // scenario:
    //   //   1. Browser tab fetches initial snapshot
    //   //   2. Browser tab opens SSE subscription
    //   //   3. Node side (full peer) writes a new node
    //   //   4. Browser tab receives the change via SSE
    //   //   5. Browser tab issues a POST write back
    //   //   6. Full peer applies the write + propagates to other
    //   //      subscribers
    //   //
    //   // Each step verified via Playwright assertions.
    //
    //   // R3-D extends this with browser-side IndexedDB cache assertion;
    //   // R3-B extends with full-peer-side filtering assertion; this
    //   // R3-E pin asserts the WIRE PROTOCOL shape end-to-end.
    //
    // OBSERVABLE consequence: browser tabs operate as authenticated
    // thin-client views — exit-criterion 19 holds.
    unimplemented!("G14-D + G18-A wires browser-tab-as-thin-client e2e exit-criterion 19 pin");
}
