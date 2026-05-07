//! Phase-3 G16-A canary — `benten-sync` Atrium P2P transport core.
//!
//! Native-only — compiles for `x86_64-*` and `aarch64-*`, NOT
//! `wasm32-*`. Browser tabs participate as authenticated thin-client
//! views via D-PHASE-3-30 protocol, NOT as full peers.
//!
//! ## Native-only per CLAUDE.md baked-in #17
//!
//! `benten-sync` compiles for `x86_64-*` and `aarch64-*` targets,
//! **NOT** `wasm32-*`. Full Atrium peer functionality requires
//! native-runtime capabilities (raw QUIC sockets, persistent storage,
//! long-lived processes). Browser tabs participate as authenticated
//! thin-client views via the D-PHASE-3-30 protocol (snapshot CID +
//! authenticated POST + Server-Sent Events / WebSocket subscription
//! against a full peer), NOT as full peers themselves.
//!
//! Two layers defend the native-only commitment:
//!
//! 1. The top-level `compile_error!` macro below fires immediately if a
//!    downstream consumer attempts to compile this crate for
//!    `target_arch = "wasm32"`. The error message names CLAUDE.md
//!    baked-in #17 + points the consumer to `benten-engine`'s
//!    thin-client surfaces.
//! 2. The `Cargo.toml` dependency tables for `iroh` + `tokio` live
//!    behind `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`,
//!    so even if a consumer attempts to bypass the lib.rs gate by
//!    pulling individual modules directly, the iroh/tokio dependency
//!    chain is not resolvable on wasm32 builds at all.
//!
//! The architectural pin
//! `crates/benten-sync/tests/wasm32_excluded.rs::benten_sync_does_not_compile_for_wasm32_unknown_unknown_per_thin_client_commitment`
//! enforces both layers.
//!
//! ## What G16-A canary lands (this PR)
//!
//! - [`transport`] — iroh QUIC transport core: [`transport::Endpoint`]
//!   binds a local iroh endpoint, [`transport::Connection`] carries
//!   bytes between two peers, [`transport::TransportStatus`] +
//!   [`transport::TransportKind`] surface the connection state for
//!   net-blocker-2 observability. Loopback round-trip is the
//!   load-bearing canary that gates G16-A landing per Q7 RESOLVED.
//! - [`errors`] — typed atrium-transport errors per net-blocker-2
//!   BLOCKER: [`errors::AtriumTransportError::RelayUnreachable`] maps
//!   to [`benten_errors::ErrorCode::AtriumRelayUnreachable`];
//!   [`errors::AtriumTransportError::TransportDegraded`] maps to
//!   [`benten_errors::ErrorCode::AtriumTransportDegraded`].
//! - [`peer_id`] — peer-id derived from `benten-id::Keypair` Ed25519
//!   pubkey per net-minor-2 + ds-8 + crypto-minor-4: PeerId == iroh
//!   EndpointId == Ed25519 pubkey design. Cross-process determinism is
//!   load-bearing for the multi-process / multi-device peer-mesh per
//!   D-PHASE-3-25.
//! - [`handshake_wire`] — wire-format struct
//!   [`handshake_wire::HandshakeFrame`] carrying BOTH peer-DID AND
//!   device-DID per net-blocker-4 BLOCKER. SCAFFOLDING ONLY: the
//!   actual handshake-protocol exchange (initiate / respond / finalise
//!   / replay-rejection / UCAN-grant exchange) is G16-D wave-6b scope.
//!   This module ships the canonical-bytes wire shape so G16-D wires
//!   the protocol bodies against a stable on-the-wire envelope.
//!
//! ## What G16-A canary does NOT land (wave-6b seams)
//!
//! G16-A is the canary-first sub-wave. The following modules / surfaces
//! are intentionally NOT shipped here; they land in wave-6b parallel-3
//! (G16-B / G16-C / G16-D) AFTER this PR merges:
//!
//! - **G16-B Loro CRDT integration** (`crdt.rs`). G16-A reserves no
//!   stub; the seam is the persistent-state graph-encoding pattern at
//!   `tests/graph_encoded_state.rs`.
//! - **G16-C MST diff + light-client** (`mst.rs`, `mst_proto.rs`,
//!   `light_client.rs`). G16-A reserves no stub.
//! - **G16-D handshake protocol body + peer discovery**
//!   (`handshake.rs`, `peer_discovery.rs`). G16-A reserves the
//!   wire-format struct only at [`handshake_wire`]; the protocol state
//!   machine is G16-D scope.
//!
//! Test pins for the wave-6b surfaces (e.g. `tests/handshake.rs`,
//! `tests/loro_lww.rs`, `tests/mst_diff.rs`) stay `#[ignore]`'d at
//! G16-A landing; the wave-6b implementers un-ignore them progressively
//! per the dispatch-conventions §3.6b end-to-end-test discipline.
//!
//! ## Architectural commitments (CLAUDE.md baked-in references)
//!
//! - **#1 (12 primitives)**: SANDBOX is the escape hatch for compute
//!   that doesn't fit the other 11 primitives. `benten-sync` does NOT
//!   add a 13th primitive — it's a transport surface consumed by the
//!   evaluator's existing primitive arms (READ / WRITE / EMIT /
//!   SUBSCRIBE) per arch-r1-11 + D-PHASE-3-14 layered architecture.
//! - **#3 Code-as-graph**: persistent atrium state (peer rosters,
//!   atrium membership, sync cursors) is graph-encoded per cag-2.
//!   G16-A canary scope (transport-only, no persistent state) defers
//!   this enforcement to G16-B/C/D where the persistent state actually
//!   lands; cag-2 / cag-r4-3 test pins stay `#[ignore]`'d at G16-A.
//! - **#5 Content-addressed hashing**: BLAKE3 + DAG-CBOR + CIDv1.
//!   `handshake_wire::HandshakeFrame::to_canonical_bytes` uses DAG-CBOR
//!   so the handshake envelope is canonical-bytes-symmetric with the
//!   rest of the engine.
//! - **#11 Capability system as pluggable policy**: the relay-trust
//!   posture (Compromise #22 in `docs/SECURITY-POSTURE.md`) is the
//!   pluggable-policy lens applied to relay infrastructure: public
//!   iroh relays carry a documented metadata-leakage compromise; Phase
//!   7 Garden-relays land as the operator-controlled alternative.
//! - **#17 Full-peer / thin-client deployment shapes**: this crate
//!   carries the full-peer transport surface. The thin-client surface
//!   lives in `benten-engine` + `bindings/napi` + the TS DSL; it
//!   speaks the D-PHASE-3-30 thin-client protocol against a connected
//!   full peer rather than running iroh natively.
//! - **arch-r1-11 + D-PHASE-3-14 layered architecture**: `benten-sync`
//!   does NOT depend on `benten-engine` or `benten-eval`. The
//!   dependency direction is engine → sync (engine consumes Atrium
//!   surface from sync), never the reverse. Pinned by
//!   `tests/dependency_edges.rs`.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A row (file ownership + must-pass tests).
//! - plan §3 G16-A row.
//! - CLAUDE.md baked-in #17 (native-only commitment).
//! - `D-PHASE-3-3` RESOLVED-at-R1 (iroh QUIC + holepunch + relay-default).
//! - `net-blocker-2` BLOCKER (typed `E_ATRIUM_RELAY_UNREACHABLE` +
//!   `E_ATRIUM_TRANSPORT_DEGRADED`).
//! - `net-blocker-4` BLOCKER (peer-handshake metadata carries peer-DID
//!   AND device-DID).
//! - `net-minor-1` (single-process two-Endpoint loopback round-trip).
//! - `net-minor-2` + `ds-8` + `crypto-minor-4` (peer-id derived
//!   deterministically from Ed25519 pubkey + iroh EndpointId reuse).
//! - `scope-real-10` (CI-conditional gating: holepunch smoke gated to
//!   specific runner cell; loopback + relay-fallback required-on-every-PR).
//! - `arch-r1-11` + `D-PHASE-3-14` (no benten-engine / benten-eval dep
//!   per layered dependency architecture).

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

// ---------------------------------------------------------------------------
// CLAUDE.md baked-in #17 — native-only compile gate.
// ---------------------------------------------------------------------------
//
// If a downstream consumer attempts to build this crate for
// `target_arch = "wasm32"`, this macro fires at compile time with a
// clear error message naming the architectural commitment + the
// thin-client alternative. Pair with the
// `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` cfg-gate
// in Cargo.toml so the iroh/tokio chain is not even resolvable on
// wasm32 builds. The architectural pin
// `crates/benten-sync/tests/wasm32_excluded.rs::benten_sync_does_not_compile_for_wasm32_unknown_unknown_per_thin_client_commitment`
// asserts both defenses are in place.
#[cfg(target_arch = "wasm32")]
compile_error!(
    "benten-sync is native-only per CLAUDE.md baked-in #17. \
     Browser tabs participate via the D-PHASE-3-30 thin-client protocol \
     (snapshot CID + authenticated POST + SSE/WS subscription against a \
     full peer), NOT as full Atrium peers. Use `benten-engine`'s \
     thin-client surfaces from wasm32 builds (see \
     `crates/benten-engine/src/thin_client_subscribe.rs` + \
     `bindings/napi/src/wasm_browser.rs` + \
     `packages/engine/src/atrium.ts`)."
);

// Native-only modules. The cfg-gate here mirrors the Cargo.toml
// `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` table
// so the modules and their iroh/tokio dependencies share a single
// architectural seam.
#[cfg(not(target_arch = "wasm32"))]
pub mod errors;

#[cfg(not(target_arch = "wasm32"))]
pub mod handshake_wire;

// G16-C wave-6b: light-client verification API + Merkle proof
// construction + verification against published roots. Distinct
// deliverable from MST diff per ROADMAP-2 — works WITHOUT full
// subgraph download.
#[cfg(not(target_arch = "wasm32"))]
pub mod light_client;

// G16-C wave-6b: Merkle Search Tree diff for subgraph sync. Converges
// in O(log n) rounds; canonical fixture corpus depth 4 / branch 8 per
// net-major-2.
#[cfg(not(target_arch = "wasm32"))]
pub mod mst;

// G16-C wave-6b: MST diff wire-protocol shape. `MessageKind::Revocation`
// is ordered before `MessageKind::Data` per net-blocker-3 BLOCKER —
// both at the wire-protocol enum (variant ordering / discriminant) AND
// at the runtime drainer (revocation drains first under concurrent
// arrival).
#[cfg(not(target_arch = "wasm32"))]
pub mod mst_proto;

#[cfg(not(target_arch = "wasm32"))]
pub mod peer_id;

#[cfg(not(target_arch = "wasm32"))]
pub mod transport;

// G16-B wave-6b — Loro CRDT integration at Node-property granularity per
// D-PHASE-3-4 RESOLVED-at-R1. Native-only alongside iroh + Loro deps.
#[cfg(not(target_arch = "wasm32"))]
pub mod crdt;
