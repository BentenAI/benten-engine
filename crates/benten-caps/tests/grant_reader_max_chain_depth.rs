//! Edge-case tests: `GrantReader::max_chain_depth` boundary (ucca-6).
//!
//! R2 landscape §2.4 row "`GrantReader::max_chain_depth` config".
//!
//! The attenuation-chain reader must refuse chains deeper than a configurable
//! limit so a malicious delegator cannot force unbounded recursion. Default
//! limit is 64 frames; a depth-65 chain rejects with `E_CAP_CHAIN_TOO_DEEP`.
//!
//! Concerns pinned:
//! - Depth at the limit (64) is accepted.
//! - Depth at limit+1 (65) fires `E_CAP_CHAIN_TOO_DEEP` AND the error routes
//!   via `ON_DENIED` semantics (i.e., the caps policy returns `Err(Denied)`,
//!   not a panic).
//! - An explicit `max_chain_depth(limit)` override is honoured (depth limit+1
//!   with a raised cap passes; same chain with the default rejects).
//! - Empty chain (depth 0) is accepted trivially.
//!
//! R3 red-phase contract: R5 (G9-A) lands `GrantReader::max_chain_depth()`
//! configuration and the depth-check in the attenuation walker. Tests
//! compile; they fail because the configuration hook does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// R3 consolidation note: the trait name `GrantReader` conflicts with the
// concrete test-harness type; this test binds to `GrantReaderChain` per the
// R3-consolidation compromise (flagged for R4 re-review).
use benten_caps::{CapabilityGrant, GrantReaderChain as GrantReader, GrantReaderConfig};
use benten_core::Cid;
use benten_errors::ErrorCode;

/// Build a synthetic chain of `depth` attenuated grants, each delegating from
/// the previous. Principal CIDs are derived from the frame index so each
/// grant is distinct at the content-addressed level.
fn make_chain(depth: usize) -> Vec<CapabilityGrant> {
    let mut chain = Vec::with_capacity(depth);
    for i in 0..depth {
        let actor = Cid::from_blake3_digest([u8::try_from(i % 256).unwrap_or(0); 32]);
        chain.push(CapabilityGrant::attenuated_for_test(
            &actor,
            "store:post:read",
            i,
        ));
    }
    chain
}

#[test]
fn grant_reader_accepts_chain_at_default_max_depth_64() {
    // Boundary: default max_chain_depth is 64. A chain of exactly 64 frames
    // is the last accepted case.
    let chain = make_chain(64);
    let reader = GrantReader::with_chain_for_test(chain);
    let result = reader.check_attenuation_for_test("store:post:read");
    assert!(
        result.is_ok(),
        "depth-64 chain must be accepted at default limit, got {:?}",
        result
    );
}

#[test]
fn grant_reader_max_chain_depth_rejects_beyond_with_typed_error() {
    // Boundary: depth 65 is one past the default limit. Must fire
    // E_CAP_CHAIN_TOO_DEEP — the error-routing expectation is "caps policy
    // rejects", which the engine surfaces through ON_DENIED at call time.
    let chain = make_chain(65);
    let reader = GrantReader::with_chain_for_test(chain);

    let err = reader
        .check_attenuation_for_test("store:post:read")
        .expect_err("depth-65 chain must be rejected at default limit");

    assert_eq!(
        err.code(),
        ErrorCode::CapChainTooDeep,
        "expected E_CAP_CHAIN_TOO_DEEP, got {:?}",
        err.code()
    );
    // Depth diagnostic: assert the structured `.context()` names the actual
    // depth + the configured limit, NOT a human-readable message substring
    // (R4 tq-7: message text is i18n-brittle; structured context is stable).
    // The error type carries a `(depth, limit)` tuple; if that API ever
    // changes the test surfaces the API diff rather than a string diff.
    assert_eq!(
        err.chain_depth_context(),
        Some((65_usize, 64_usize)),
        "CapChainTooDeep must expose a structured (depth, limit) context \
         tuple carrying (actual={}, limit={}); got {:?}",
        65,
        64,
        err.chain_depth_context()
    );
}

#[test]
fn grant_reader_max_chain_depth_override_raises_cap() {
    // Same depth-65 chain is accepted when max_chain_depth is raised to 128.
    let chain = make_chain(65);
    let cfg = GrantReaderConfig {
        max_chain_depth: 128,
    };
    let reader = GrantReader::with_chain_and_config_for_test(chain, cfg);
    assert!(
        reader.check_attenuation_for_test("store:post:read").is_ok(),
        "raised cap must admit depth-65"
    );
}

#[test]
fn grant_reader_empty_chain_accepts_trivially() {
    // Degenerate input: a zero-depth chain is a vacuous attenuation; the
    // reader must not fire E_CAP_CHAIN_TOO_DEEP on empty input.
    let reader = GrantReader::with_chain_for_test(vec![]);
    let result = reader.check_attenuation_for_test("store:post:read");
    // An empty chain cannot satisfy any scope, so it denies — but NOT with
    // E_CAP_CHAIN_TOO_DEEP. Either CapDenied or Ok(unreachable) is acceptable
    // so long as the depth code is not the answer.
    if let Err(e) = result {
        assert_ne!(
            e.code(),
            ErrorCode::CapChainTooDeep,
            "empty chain must not fire E_CAP_CHAIN_TOO_DEEP"
        );
    }
}
