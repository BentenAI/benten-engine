//! Phase-4-Foundation R3 Family F1 — admin UI v0 test harness scaffolding.
//!
//! Stub at R3 (RED-PHASE) per `.addl/phase-4-foundation/r2-test-landscape.md`
//! §4 "NEW helpers that should land FIRST" item 3.
//!
//! ## Purpose
//!
//! Family F1 (admin UI shell + dogfood paths + thin-client + T2 defense)
//! authors this harness as the canonical canary-shape for downstream
//! sub-families F2/F3 to consume. The harness exists at R3 so test pins
//! can reference its types at the `use` line; bodies stay `unimplemented!()`
//! (canonical RED-PHASE shape). G24-A / G24-F implementer waves fill the
//! implementation.
//!
//! ## What this harness will expose at G24-A / G24-F landing
//!
//! - `AdminUiV0TestHarness` — composed engine + thin-client session +
//!   DID-keyed handshake stub + 2-peer Atrium fixture.
//! - `establish_session(origin: &str) -> SessionToken` — DID-keyed
//!   handshake against an origin; returns an origin-bound session token.
//! - `attempt_cross_origin_replay(token, replay_origin) -> Result<...>`
//!   — simulates capture + replay; pin asserts the replay is denied per T2.
//! - `dispatch_workflow_create(user_did, workflow_spec) -> Result<Cid>`
//!   — substantive dogfood-path (a) production-runtime arm; persists
//!   workflow to redb; returns content CID for round-trip pin.
//! - `replay_workflow_from_cid(cid) -> Trace` — production-runtime arm
//!   for dogfood-path (a) replay-determinism pin per §3.6f
//!   SHAPE-not-SUBSTANCE discipline.
//! - `subgraph_walk_admin_ui() -> impl Iterator<Item = PrimitiveKind>`
//!   — walk of the admin UI subgraph the evaluator dispatches over; pin
//!   asserts every walked node's `PrimitiveKind` is one of the canonical
//!   12 (CLAUDE.md baked-in #1).
//! - `trace_capture(closure)` — runtime-trace harness for SHAPE+SUBSTANCE
//!   pairing: capture which engine surfaces (`read_node_as`,
//!   `on_change_as_with_cursor`, `read_node`) were called during a
//!   subgraph walk.
//!
//! ## Why this lives in `tests/common/` not `src/testing.rs`
//!
//! Per Phase-3 G16 conventions: harness types that compose multi-crate
//! fixtures (engine + iroh-peer + ManifestStore) live in test-side
//! `common/` modules so the production library surface stays minimal.
//! At G24-A / G24-F, the harness graduates to consume the real engine +
//! thin-client session + Atrium peer-mesh seams.

#![allow(dead_code)]

/// Composed harness for admin UI v0 integration tests.
///
/// **Stub.** G24-A wave fills.
pub struct AdminUiV0TestHarness;

impl AdminUiV0TestHarness {
    /// Construct a harness with a 2-peer Atrium fixture + an engine
    /// configured for full-peer shape (a) per CLAUDE.md baked-in #17.
    ///
    /// **Stub.** G24-A wave fills.
    pub fn new() -> Self {
        unimplemented!(
            "G24-A wires AdminUiV0TestHarness::new — composes engine + \
             2-peer Atrium + DID-keyed thin-client session stub"
        )
    }

    /// Construct a thin-client variant (shape b: wasm32-unknown-unknown
    /// browser bundle) backed by a full peer over a loopback transport.
    ///
    /// **Stub.** G24-F wave fills.
    pub fn new_thin_client_against_full_peer() -> Self {
        unimplemented!("G24-F wires thin-client harness variant")
    }
}

impl Default for AdminUiV0TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Opaque session-token handle returned from DID-keyed handshake.
///
/// **Stub shape only.** G24-F wires the real `thin_client_session::SessionToken`.
#[derive(Debug, Clone)]
pub struct SessionTokenStub {
    pub token_bytes: Vec<u8>,
    pub bound_origin: String,
}
