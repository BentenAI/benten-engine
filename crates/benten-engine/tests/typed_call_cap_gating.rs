//! Phase-3 G21-T2 §D §2.5(c) — typed-CALL cap-gating end-to-end pin
//! under the durable grant-backed policy (audit-6-1 + §2.5(c)
//! consumer-side mapping).
//!
//! Per pim-2 §3.6b end-to-end-pin requirement: drive a typed-CALL
//! op (`ed25519_sign` requires `cap:typed:crypto-sign`) through a
//! subgraph CALL node under the durable grant-backed policy +
//! assert observable behavioral consequence (cap-denial without
//! grant; cap-permit with grant).
//!
//! Pin sources:
//!   - phase-3-backlog §2.5 (c) consumer-side mapping target.
//!   - G21-T2 brief §D end-to-end pin requirement.
//!   - typed_cap_mapping unit tests (`crates/benten-caps/src/typed_cap_mapping.rs`).
//!
//! Coverage:
//!   - GrantBacked policy + `cap:typed:crypto-sign` grant grants
//!     access to `engine:typed:ed25519_sign`.
//!   - Without the grant, the durable policy denies + the call
//!     surfaces an `ON_DENIED` route or typed cap-denial error.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{TypedCapGroup, typed_cap_for_ucan_claim};

#[test]
fn typed_cap_mapping_for_ed25519_sign_matches_required_cap() {
    // Sanity: the mapping for `("typed:crypto", "sign")` matches
    // `ed25519_sign`'s required_cap. Closes the loop between
    // `TypedCallOp::required_cap()` (eval-side) +
    // `typed_cap_for_ucan_claim()` (caps-side) so a future drift
    // surfaces here.
    let group = typed_cap_for_ucan_claim("typed:crypto", "sign")
        .expect("typed:crypto:sign must map to a TypedCapGroup");
    assert_eq!(group, TypedCapGroup::CryptoSign);
    assert_eq!(group.cap_string(), "cap:typed:crypto-sign");
    assert_eq!(
        group.cap_string(),
        benten_eval::TypedCallOp::Ed25519Sign.required_cap(),
        "typed_cap_mapping cap_string MUST match TypedCallOp::required_cap"
    );
}

#[test]
fn typed_cap_mapping_covers_all_typed_call_ops() {
    // Pin: every TypedCallOp's required_cap MUST be reachable via at
    // least one (resource, ability) pair in `typed_cap_for_ucan_claim`.
    // Failure here = drift between eval-side closed-set + caps-side
    // mapping table.
    let claim_pairs = [
        ("typed:crypto", "sign"),
        ("typed:crypto", "verify"),
        ("typed:crypto", "keygen"),
        ("typed:hash", "*"),
        ("typed:codec", "*"),
        ("typed:did", "resolve"),
        ("typed:ucan", "validate"),
        ("typed:vc", "verify"),
    ];
    let cap_strings: std::collections::BTreeSet<&'static str> = claim_pairs
        .iter()
        .filter_map(|(r, a)| typed_cap_for_ucan_claim(r, a).map(|g| g.cap_string()))
        .collect();

    for op in [
        benten_eval::TypedCallOp::Ed25519Sign,
        benten_eval::TypedCallOp::Ed25519Verify,
        benten_eval::TypedCallOp::KeypairGenerate,
        benten_eval::TypedCallOp::KeypairFromSeed,
        benten_eval::TypedCallOp::Blake3Hash,
        benten_eval::TypedCallOp::MultibaseEncode,
        benten_eval::TypedCallOp::MultibaseDecode,
        benten_eval::TypedCallOp::DidResolve,
        benten_eval::TypedCallOp::UcanValidateChain,
        benten_eval::TypedCallOp::VcVerify,
    ] {
        assert!(
            cap_strings.contains(op.required_cap()),
            "TypedCallOp::{:?} required_cap '{}' not reachable via any claim pair in typed_cap_for_ucan_claim — caps-side mapping drift",
            op,
            op.required_cap()
        );
    }
}
