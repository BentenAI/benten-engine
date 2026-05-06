//! Phase-3 `benten-id` — identity primitives for the Benten engine.
//!
//! ## G14-A1 wave-4a scope (LIVE)
//!
//! - [`keypair`] — Ed25519 keypair wrapping `ed25519-dalek` with
//!   secret-bytes hygiene (`Zeroize + ZeroizeOnDrop`, no `Clone`,
//!   redacted `Debug`); `Keypair::generate()` pinned to `OsRng` per
//!   `crypto-major-2`; `Keypair::from_seed_bytes` separate import path
//!   with DAG-CBOR envelope schema `{version, alg, secret_bytes}` per
//!   `crypto-major-5`.
//! - [`did`] — `did:key` DIDs using multibase prefix `z` + multicodec
//!   `0xed01` for Ed25519 per W3C did-method-key spec / `crypto-minor-3`.
//! - [`ucan`] — UCAN claim envelope + chain validation; `nbf` / `exp`
//!   time-window enforcement at chain-walk site per `crypto-blocker-2`
//!   BLOCKER; constant-time-comparison via `subtle::ConstantTimeEq`
//!   per `crypto-major-4`; audience-binding rejection.
//!
//! ## G14-A2 wave-4a' scope (NOT YET LANDED)
//!
//! - `vc` — Verifiable Credential issuance + verify.
//! - `multi_sig` — `MultiSigSurface` trait + `Ed25519SingleKey` default impl.
//! - `did_rotation` — `Did::rotate_keypair` + `superseded_by` chain.
//! - `device_attestation` — Device-DID capability-attestation.
//!
//! ## Architectural commitments (CLAUDE.md baked-in references)
//!
//! - **#3 Code-as-graph**: durable identity surfaces (G14-A2's
//!   keypair-anchor / DID-rotation-attestation / VC-receipt /
//!   UCAN-grant / device-attestation) are graph Nodes, not opaque CBOR
//!   blobs. The G14-A1 surfaces here (Keypair / Did / Ucan) are
//!   in-memory primitives; durable persistence flows through G14-B's
//!   UCAN backend in `benten-caps` + G14-C's manifest-signing wire-up
//!   in `benten-engine`.
//! - **#17 Full-peer / thin-client deployment shapes**: this crate is
//!   consumed by both. The full peer (native Rust) carries the full
//!   surface; the thin compute surface (wasm32 browser tab) declares
//!   its identity envelope via the TS `identity.ts` shim and
//!   handshakes that envelope to the full peer. No host-fn surface
//!   here that would only work native.
//! - **arch-r1-10**: `benten-id` does NOT depend on `benten-graph`,
//!   `benten-engine`, `benten-eval`, or `benten-caps`. The dependency
//!   graph layer is enforced by `crates/benten-id/tests/dependency_edges.rs`.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod did;
pub mod errors;
pub mod keypair;
pub mod ucan;

pub use errors::{DidError, KeypairError, SeedImportError, UcanError};
