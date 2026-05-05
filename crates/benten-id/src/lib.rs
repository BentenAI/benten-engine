//! Phase-3 G14-A1 canary STUB — `benten-id` test-shell crate.
//!
//! ## What this is
//!
//! Empty stub crate landed by R3-A (R3 RED-PHASE test-writing) so that
//! R3-A test pins targeting the not-yet-existing `benten-id` API surface
//! compile-but-fail (or stay `#[ignore]`'d behind `// IGNORED until
//! G14-A1`). G14-A1 wave-4a fills this lib with the real Ed25519 keypair
//! + did:key + UCAN implementation per:
//!
//! - `crypto-blocker-1` BLOCKER (zeroize-on-drop + no-Clone + redacted
//!   Debug for `SecretKey`).
//! - `crypto-major-2` (`Keypair::generate` pinned to `OsRng`).
//! - `crypto-major-5` (`Keypair::from_seed_bytes` DAG-CBOR envelope
//!   schema with version tag).
//! - `crypto-major-4` (constant-time UCAN chain-walk via `subtle`).
//! - `crypto-blocker-2` BLOCKER + CLR-2 (UCAN `nbf`/`exp` time-window
//!   enforcement at chain-walk site).
//! - `crypto-minor-3` (did:key z-multibase prefix + 0xed01 multicodec
//!   per W3C spec).
//!
//! ## Why a stub at R3 wave-1pre completion
//!
//! Per `.addl/phase-3/r2-test-landscape.md` §13 R3 dispatch protocol,
//! R3-A is the canary that lands the test-shell + ~60-65 RED-phase test
//! pins for G13 / G14-pre-D / G14-A1 surface ownership. The test pins
//! cite `benten_id::*` API (e.g. `benten_id::keypair::Keypair`,
//! `benten_id::did::Did`, `benten_id::ucan::Ucan`); without the crate
//! existing, the test files don't even compile-but-fail (they fail at
//! the `use` line, which is a less-precise red signal). Landing the
//! crate stub now means:
//!
//! 1. `cargo check -p benten-id` passes at R3-A landing time (empty
//!    crate, no public surface yet).
//! 2. R3-A test files compile-but-fail at the `use` line because the
//!    cited `benten_id::keypair::Keypair` does not yet exist —
//!    canonical RED-phase behavior. (Tests that would reference the
//!    real types stay `#[ignore]`'d so the workspace `cargo test`
//!    stays green; only the `#[ignore]` rationale documents the
//!    RED-phase pin.)
//! 3. G14-A1 implementer un-ignores progressively as each module lands
//!    (`keypair.rs` → un-ignore the keypair test file; `did.rs` →
//!    un-ignore did_key.rs; `ucan.rs` → un-ignore ucan.rs and
//!    prop_ucan_attenuation.rs).
//!
//! ## Public surface (intentionally empty at R3 landing time)
//!
//! G14-A1 fills:
//!
//! ```text
//! pub mod keypair;       // Ed25519 wrapper; Keypair, SecretKey (Zeroize, no Clone)
//! pub mod did;           // did:key DID; z-multibase + 0xed01 multicodec
//! pub mod ucan;          // UCAN claim envelope + chain-walk + nbf/exp enforcement
//! ```
//!
//! And G14-A2 fills:
//!
//! ```text
//! pub mod vc;                  // Verifiable Credential issuance + verify
//! pub mod multi_sig;           // MultiSigSurface trait + Ed25519SingleKey default
//! pub mod did_rotation;        // Did::rotate_keypair + superseded_by chain
//! pub mod device_attestation;  // Device-DID capability-attestation + replay-resistance
//! ```
//!
//! At R3-A landing time NONE of these modules exist; this stub crate
//! intentionally exposes nothing.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

// G14-A1 fills this; do not add public items here at R3 RED-phase
// landing time. The crate compiles as an empty rlib so that the
// workspace `cargo check` stays green at R3 dispatch time.
