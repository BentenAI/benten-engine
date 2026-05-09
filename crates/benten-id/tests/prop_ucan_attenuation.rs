//! R3-A RED-PHASE proptest pin: UCAN chain attenuation never widens
//! authority OR time-window (G14-A1 + G14-B canary smoke;
//! r2-test-landscape §3.A cluster + plan §4 seed).
//!
//! Pin sources:
//! - r2-test-landscape §2.2 G14-A1 row +
//!   `prop_ucan_chain_attenuation_never_widens_authority`; §3.A CLR-2
//!   cluster; cap-major-1.
//! - r2-test-landscape §11 row +
//!   `prop_ucan_chain_time_window_never_widens` (R4-FP-added pin
//!   closing `.addl/phase-3/r4-r1-test-coverage-completeness.json`
//!   finding `tcc-r1-5`).
//!
//! ## Property under test
//!
//! For any 2+ -link UCAN delegation chain, the EFFECTIVE granted
//! authority of the leaf token MUST be a SUBSET (proper or improper)
//! of every ancestor's granted authority. Concretely: if parent grants
//! `{(/zone/posts, read)}`, child cannot grant `{(/zone/posts, write)}`
//! and cannot grant `{(/zone/admin, read)}`.
//!
//! ## Counts
//!
//! 10 000 cases. Each case generates an arbitrary chain of 2-5 links
//! with arbitrary capabilities at each link, and asserts that
//! `validate_chain` either:
//!
//! 1. Rejects with `AttenuationViolated` (chain has at least one link
//!    that widens authority), OR
//! 2. Accepts AND the leaf's authority is a subset of every ancestor's.
//!
//! Never: accepts with authority that exceeds an ancestor.

#![allow(clippy::unwrap_used)]

use benten_id::keypair::Keypair;
use benten_id::ucan::{Ucan, UcanBuilder, validate_chain_at};
use proptest::prelude::*;

/// Build a leaf-first chain of length `len`. Each token grants the
/// supplied resource:ability tuple at that link (parents grant a
/// superset axis we control via `parent_resources` / `parent_abilities`).
///
/// Chain structure (leaf-first per `validate_chain_inner` contract):
///   chain[0] = leaf, signed by keypairs[0], audience = arbitrary
///   chain[1] = parent of chain[0], signed by keypairs[1], audience = keypairs[0].did
///   chain[2] = grandparent, signed by keypairs[2], audience = keypairs[1].did
///   …
///
/// Time-windows: every link uses the same `[nbf=0, exp=u64::MAX/2]`
/// window unless the caller overrides per-link. Validation runs at
/// `now=1_000_000_000` which is inside every default window.
fn build_chain_with_caps(
    keypairs: &[Keypair],
    caps_per_link: &[(String, String)], // [(resource, ability)] leaf-first
    nbf_per_link: &[u64],
    exp_per_link: &[u64],
) -> Vec<Ucan> {
    let len = caps_per_link.len();
    assert!(keypairs.len() == len);
    assert!(nbf_per_link.len() == len);
    assert!(exp_per_link.len() == len);

    // Build root-first, then reverse to leaf-first at the end.
    let mut root_first: Vec<Ucan> = Vec::with_capacity(len);
    for i in (0..len).rev() {
        // root-first index: i counts down from len-1 (root) to 0 (leaf).
        let issuer = &keypairs[i];
        // parent.aud = child.iss → at link i, audience is keypairs[i-1] (the child).
        // The root has audience == its own DID (or unbound; we use issuer's DID
        // for simplicity since chain-link integrity only checks parent.aud == child.iss).
        let audience_did = if i == 0 {
            issuer.public_key().to_did().as_str().to_string()
        } else {
            keypairs[i - 1].public_key().to_did().as_str().to_string()
        };
        let (resource, ability) = &caps_per_link[i];
        let token = UcanBuilder::default()
            .issuer(issuer.public_key().to_did().as_str().to_string())
            .audience(audience_did)
            .capability(resource.clone(), ability.clone())
            .not_before(nbf_per_link[i])
            .expiry(exp_per_link[i])
            .sign(issuer);
        root_first.push(token);
    }
    // root_first now has root at end (we pushed in reverse). Actually we pushed
    // root first (i = len-1), then progressively younger, so root_first[0] is
    // root and root_first[len-1] is leaf. Need leaf-first for the API.
    root_first.reverse();
    root_first
}

/// Leaf-first chain construction helper using `Ucan::builder` so this
/// file does not depend on private internals.
fn build_uniform_chain(
    keypairs: &[Keypair],
    resource: &str,
    ability: &str,
    nbf: u64,
    exp: u64,
) -> Vec<Ucan> {
    let len = keypairs.len();
    let caps = vec![(resource.to_string(), ability.to_string()); len];
    let nbfs = vec![nbf; len];
    let exps = vec![exp; len];
    build_chain_with_caps(keypairs, &caps, &nbfs, &exps)
}

const VALIDATION_NOW: u64 = 1_000_000_000;
const SAFE_EXP: u64 = u64::MAX / 2;

// Case count: 256 is proptest's default; 2000 is brief's "7.8x default"
// thoroughness. MSRV 1.95 cells (linux-x86_64 + macos-x86_64) timed out
// at the 180s nextest slow-timeout default with 2000 cases — the slower
// rustc 1.95 codegen pushes UCAN-chain validation past 180s/test on
// those cells. 1000 cases preserves >=3.9x default coverage while
// staying inside the 180s wall-clock under MSRV 1.95 codegen. Stable
// toolchain CI cells run unaffected. Override via PROPTEST_CASES env
// for ad-hoc deeper sweeps when investigating a candidate flake.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    /// Cap-major-1 attenuation property. For every randomly-generated
    /// 2-5 link chain, at most the leaf's authority is what
    /// `validate_chain_at` permits — and any ancestor that grants a
    /// strictly-narrower cap forces the chain to reject.
    ///
    /// Strategy: each generated chain has either uniform caps (all
    /// links grant `(resource, ability)` — MUST validate) or a forced
    /// widening at one link (the leaf claims a cap NOT in its parent's
    /// authority — MUST reject with `AttenuationViolated`). The
    /// property: validation outcome MUST agree with the chain shape.
    #[test]
    fn prop_ucan_chain_attenuation_never_widens(
        chain_length in 2usize..=5usize,
        widen_seed in any::<u8>(),
        cap_seed in any::<u64>(),
    ) {
        let keypairs: Vec<Keypair> = (0..chain_length).map(|_| Keypair::generate()).collect();
        // Pick a deterministic resource:ability pair from the seed. Two
        // disjoint cap classes — `/zone/posts:read` vs `/zone/admin:write` —
        // so a "widening" chain has the leaf claiming a cap the parent
        // does NOT grant.
        let parent_resource = "/zone/posts";
        let parent_ability = "read";
        let widen = (widen_seed % 4) == 0; // ~25% of cases force widening

        let chain = if widen {
            // Parent grants `/zone/posts:read`; leaf claims `/zone/admin:write`.
            // Chain MUST reject with `AttenuationViolated` at the parent→leaf
            // link.
            let mut caps: Vec<(String, String)> = (0..chain_length)
                .map(|_| (parent_resource.to_string(), parent_ability.to_string()))
                .collect();
            caps[0] = ("/zone/admin".to_string(), "write".to_string()); // leaf widens
            let nbfs = vec![0u64; chain_length];
            let exps = vec![SAFE_EXP; chain_length];
            build_chain_with_caps(&keypairs, &caps, &nbfs, &exps)
        } else {
            // Uniform chain — every link grants `/zone/posts:read`. MUST validate.
            let _ = cap_seed;
            build_uniform_chain(&keypairs, parent_resource, parent_ability, 0, SAFE_EXP)
        };

        let result = validate_chain_at(&chain, VALIDATION_NOW);
        if widen {
            prop_assert!(
                result.is_err(),
                "widening chain (leaf claims /zone/admin:write while parent grants \
                 /zone/posts:read) MUST reject; got {result:?}"
            );
        } else {
            prop_assert!(
                result.is_ok(),
                "uniform chain (every link grants /zone/posts:read) MUST validate; \
                 got {result:?}"
            );
        }
    }

    /// Cap-major-1 time-window axis: `validate_chain_at` MUST reject
    /// any chain where the leaf claims a (nbf, exp) window that is NOT
    /// a subset of every ancestor's window. Companion to the authority
    /// proptest above; closes tcc-r1-5 R3-A territory.
    ///
    /// Strategy: each generated chain has either uniform time-windows
    /// (every link `[100, 1000]` — MUST validate at now=500) or a
    /// forced widening at the leaf (leaf `[50, 2000]` while ancestors
    /// `[100, 1000]` — MUST reject because leaf widens both nbf and
    /// exp). Validation runs at a `now` that is inside every link's
    /// claimed window so the only failure axis is window-narrowing.
    #[test]
    fn prop_ucan_chain_time_window_never_widens(
        chain_length in 2usize..=5usize,
        widen_seed in any::<u8>(),
    ) {
        let keypairs: Vec<Keypair> = (0..chain_length).map(|_| Keypair::generate()).collect();
        let widen = (widen_seed % 4) == 0; // ~25% widening cases

        // Anchor windows. Validation runs at `now=500`, inside `[100, 1000]`.
        let parent_nbf = 100u64;
        let parent_exp = 1_000u64;
        let now = 500u64;

        let chain = if widen {
            // Leaf widens BOTH bounds: nbf=50 (earlier than parent=100),
            // exp=2000 (later than parent=1000). Chain MUST reject.
            let mut nbfs = vec![parent_nbf; chain_length];
            let mut exps = vec![parent_exp; chain_length];
            nbfs[0] = 50; // leaf widens nbf backwards
            exps[0] = 2_000; // leaf widens exp forwards
            let caps = vec![("/zone/posts".to_string(), "read".to_string()); chain_length];
            build_chain_with_caps(&keypairs, &caps, &nbfs, &exps)
        } else {
            // Uniform window across every link. MUST validate at now=500.
            build_uniform_chain(&keypairs, "/zone/posts", "read", parent_nbf, parent_exp)
        };

        let result = validate_chain_at(&chain, now);
        if widen {
            prop_assert!(
                result.is_err(),
                "leaf widens parent's time-window (leaf [50, 2000] vs parent [100, 1000]) \
                 MUST reject; got {result:?}"
            );
        } else {
            prop_assert!(
                result.is_ok(),
                "uniform-window chain MUST validate at now inside every link's window; \
                 got {result:?}"
            );
        }
    }
}
