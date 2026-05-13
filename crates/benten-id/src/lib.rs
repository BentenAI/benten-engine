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
//! ## G14-A2 wave-4a' scope (LIVE)
//!
//! - [`vc`] — Verifiable Credential issuance + verify (W3C VC v1.1-
//!   INSPIRED field shape over the existing DAG-CBOR + Ed25519
//!   surface; **NOT wire-format-compatible with external W3C JSON-LD
//!   VC consumers** — that interop layer + `ssi` re-introduction
//!   deferred to G14-B per `docs/future/phase-3-backlog.md
//!   §2.1-followup`. See `crates/benten-id/src/vc.rs` module-level
//!   note for the DISAGREE-WITH-EXPLANATION rationale per HARD RULE
//!   rule-12 disposition (c)).
//! - [`multi_sig`] — [`multi_sig::MultiSigSurface`] trait +
//!   [`multi_sig::Ed25519SingleKey`] default impl + compile-only
//!   [`multi_sig::ThresholdMultiSig`] extension-point per
//!   D-PHASE-3-24 deferral.
//! - [`did_rotation`] — [`did_rotation::rotate_keypair`] +
//!   [`did_rotation::RotationAttestation`] +
//!   [`did_rotation::RotationLog`] in-RAM chain-walk helper.
//! - [`device_attestation`] — [`device_attestation::DeviceAttestation`] +
//!   [`device_attestation::Acceptor`] (freshness + nonce-store +
//!   revocation) + [`device_attestation::DeviceRevocation`] +
//!   `runs_sandbox=true`+browser-target rejection at construction
//!   per `br-r4-r1-4` / `br-r4-r2-3` MAJOR.
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
//!
//! ## G27-C wave (Phase 4-Foundation §4.3) scope (LIVE)
//!
//! - [`grant_reader`] — sibling `GrantReader` trait at the `benten-id`
//!   layer with a CID-keyed companion method
//!   `has_unrevoked_grant_for_grant_cid(&Cid)` that closes the §13.11
//!   structural-lesson architectural gap. Sibling (not extension) of
//!   `benten-caps::grant_backed::GrantReader` because arch-r1-10
//!   forbids `benten-id → benten-caps` dependency edges; the two
//!   traits coexist + may be implemented together by concrete types
//!   that need both key shapes.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod device_attestation;
pub mod did;
pub mod did_rotation;
pub mod errors;
pub mod grant_reader;
pub mod keypair;
pub mod multi_sig;
pub mod ucan;
pub mod vc;

pub use errors::{
    DeviceAttestationError, DidError, DidRotationError, KeypairError, MultiSigError,
    SeedImportError, UcanError, VcError,
};
