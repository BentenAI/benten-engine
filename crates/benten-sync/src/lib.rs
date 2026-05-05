//! Phase-3 G16-A canary STUB — `benten-sync` test-shell crate.
//!
//! ## Native-only per CLAUDE.md baked-in #17
//!
//! `benten-sync` compiles for `x86_64-*` and `aarch64-*` targets, **NOT**
//! `wasm32-*`. Full Atrium peer functionality requires native-runtime
//! capabilities (raw QUIC sockets, persistent storage, long-lived
//! processes). Browser tabs participate as authenticated thin-client
//! views via the D-PHASE-3-N protocol (snapshot CID + authenticated
//! POST + Server-Sent Events / WebSocket subscription against a full
//! peer), NOT as full peers themselves. The architectural pin
//! `tests/wasm32_excluded.rs` enforces this commitment.
//!
//! ## What this is
//!
//! Empty stub crate landed by R3-C (R3 RED-PHASE test-writing) so that
//! R3-C test pins targeting the not-yet-existing `benten-sync` API
//! surface compile-but-fail (or stay `#[ignore]`'d behind a RED-PHASE
//! rationale). G16-A wave-6 canary fills this lib with iroh transport
//! + peer-id derivation + typed transport errors; G16-B wave-6b adds
//! Loro CRDT integration; G16-C adds MST diff + light-client; G16-D
//! adds peer-discovery + DID handshake.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A/B/C/D rows (iroh + Loro + MST + DID handshake).
//! - plan §3 G16 group rows (file ownership + must-pass tests).
//! - CLAUDE.md baked-in #17 (native-only commitment).
//! - `arch-r1-11` + `D-PHASE-3-14` (no benten-engine / benten-eval dep
//!   per layered dependency architecture; engine consumes sync, not the
//!   reverse).
//! - `cag-2` (state graph-encoded; no opaque CRDT blobs).
//! - `cag-6` (Loro merged nodes are graph-encoded; not opaque CRDT
//!   blobs at the storage layer).
//! - `net-blocker-2` (typed `E_ATRIUM_RELAY_UNREACHABLE` +
//!   `E_ATRIUM_TRANSPORT_DEGRADED` errors).
//! - `net-blocker-3` (revocation-message-kind ordered before data at
//!   handshake + MST diff drain).
//! - `net-blocker-4` (peer-handshake metadata carries peer-DID AND
//!   device-DID).
//! - `D-C` / `D-PHASE-3-22` hybrid (iii) (Loro merges produce new
//!   Version Nodes via Anchor + Version + CURRENT pattern;
//!   AttributionFrame at the new Version captures contributing
//!   peer-DIDs).
//! - `device-mesh-exploration` brief-edits (revocation-order at
//!   reconnect; Inv-14 device-grain attribution).
//!
//! ## Why a stub at R3 wave-1pre completion
//!
//! Per `.addl/phase-3/r2-test-landscape.md` §13 R3 dispatch protocol,
//! R3-C lands the test-shell + ~75-80 RED-phase test pins for G15 +
//! G16 surface ownership. The G16 test pins cite `benten_sync::*` API
//! (e.g. `benten_sync::transport::Endpoint`, `benten_sync::crdt::LoroDoc`,
//! `benten_sync::handshake::Handshake`); without the crate existing,
//! the test files don't even compile-but-fail (they fail at the `use`
//! line, which is a less-precise red signal). Landing the crate stub
//! now means:
//!
//! 1. `cargo check -p benten-sync` passes at R3-C landing time (empty
//!    crate, no public surface yet).
//! 2. R3-C test files compile-but-fail at the `use` line because the
//!    cited `benten_sync::transport::Endpoint` does not yet exist —
//!    canonical RED-phase behavior. (Tests that would reference the
//!    real types stay `#[ignore]`'d so the workspace `cargo test`
//!    stays green; only the `#[ignore]` rationale documents the
//!    RED-phase pin.)
//! 3. G16-A implementer un-ignores progressively as each module lands
//!    (`transport.rs` → un-ignore the transport tests; `crdt.rs` →
//!    un-ignore loro_lww.rs etc.).
//!
//! ## Public surface (intentionally empty at R3 landing time)
//!
//! G16-A wave-6 canary fills:
//!
//! ```text
//! pub mod transport;     // iroh QUIC + holepunch + relay-default per D-PHASE-3-3
//! pub mod errors;        // typed atrium-transport error variants per net-blocker-2
//! pub mod peer_id;       // peer-id derived from Ed25519 keypair
//! ```
//!
//! G16-B wave-6b adds:
//!
//! ```text
//! pub mod crdt;          // Loro at Node-property granularity (per-property LWW + HLC)
//! ```
//!
//! G16-C wave-6b adds:
//!
//! ```text
//! pub mod mst;           // Merkle Search Tree diff for subgraph sync
//! pub mod mst_proto;     // wire-protocol shape (typed `revocation` message-kind)
//! pub mod light_client;  // light-client verification API (distinct from MST diff)
//! ```
//!
//! G16-D wave-6b adds:
//!
//! ```text
//! pub mod handshake;        // DID-based mutual auth + UCAN-grant exchange
//! pub mod peer_discovery;   // iroh relay default; opt-in dedicated peer-list
//! ```
//!
//! At R3-C landing time NONE of these modules exist; this stub crate
//! intentionally exposes nothing.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

// G16-A wave-6 canary fills this; do not add public items here at R3-C
// RED-phase landing time. The crate compiles as an empty rlib so that
// the workspace `cargo check` stays green at R3-C dispatch time.
