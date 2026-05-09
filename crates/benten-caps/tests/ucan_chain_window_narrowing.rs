//! R4-FP-R3-B RED-PHASE pins: UCAN parent-child time-window narrowing
//! at durable layer (G14-B wave-4b; cap-r4-2 MAJOR closure of
//! cap-major-1 fix-now-action).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-2):
//!
//! - `tests/ucan_chain_rejects_child_expires_after_parent` — cap-r4-2 (a)
//! - `tests/ucan_chain_rejects_child_not_before_earlier_than_parent` — cap-r4-2 (b)
//! - `tests/ucan_chain_validation_at_replay_time_uses_current_clock_not_issuance_clock` — cap-r4-2 (c)
//!
//! ## Architectural intent (cap-r4-2 MAJOR closure)
//!
//! cap-major-1 (R1 capability-system) specified four named pins to
//! close UCAN time-window correctness. The R3 corpus shipped:
//! - `ucan_chain_walk_propagates_nbf_exp_through_attenuation`
//!   (parent-expired propagates to child — ONE direction)
//! - `prop_ucan_chain_attenuation_never_widens_authority` (10k cases —
//!   AUTHORITY dimension only, NOT window dimension)
//! - `ucan_backend_chain_walk_rejects_expired_proof_at_durable_store_lookup`
//!   (parameterized current-clock — adjacent shape)
//!
//! The MISSING pins addressed here:
//! - **child_expires_after_parent**: explicit child-claims-later-exp
//!   rejection (parent.exp = T+60; child.exp = T+86400 → MUST reject).
//! - **child_not_before_earlier_than_parent**: explicit child-claims-
//!   earlier-nbf rejection (parent.nbf = T+1000; child.nbf = T+500 →
//!   MUST reject).
//! - **replay_time_uses_current_clock**: durable-store re-validates
//!   against fresh wall-clock; issuance-time clock not cached.
//!
//! Note: the 10k window-only proptest `prop_ucan_chain_time_window_never_widens`
//! is added to `crates/benten-id/tests/prop_ucan_attenuation.rs` by a
//! separate fix (per tcc-r1-5 R3-A territory). This file pins the
//! deterministic cases at the durable layer (G14-B).
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-B
//! implementer un-ignores AND replaces stub bodies. Per §3.6b pim-2
//! these tests must drive the production
//! `UCANBackend::validate_chain_at` path + assert typed-error variants
//! naming the specific window violation.

#![allow(clippy::unwrap_used)]

use benten_id::errors::UcanError;
use benten_id::keypair::Keypair;
use benten_id::ucan::{Ucan, validate_chain_at};

/// Build a UCAN with the supplied issuer / audience / window /
/// capability. Helper for the cap-r4-2 (a)/(b)/(c) pins below.
fn build_ucan(
    issuer: &Keypair,
    audience_did: &str,
    resource: &str,
    ability: &str,
    nbf: u64,
    exp: u64,
) -> Ucan {
    Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str().to_string())
        .audience(audience_did.to_string())
        .capability(resource, ability)
        .not_before(nbf)
        .expiry(exp)
        .sign(issuer)
}

#[test]
fn ucan_chain_rejects_child_expires_after_parent() {
    // cap-r4-2 (a) pin. The child's exp window MUST NOT exceed the
    // parent's exp; widening rejects at chain-walk with
    // `AttenuationViolated` (time-window axis joins the same
    // attenuation family as the authority axis per the §5
    // narrowing-rule comment in `crates/benten-id/src/ucan.rs`).
    let root_kp = Keypair::generate();
    let middle_kp = Keypair::generate();
    let issuance_secs = 1_000_000_000u64;

    // Parent: short exp window (60s).
    let parent = build_ucan(
        &root_kp,
        middle_kp.public_key().to_did().as_str(),
        "/zone/posts",
        "read",
        issuance_secs,
        issuance_secs + 60,
    );
    // Child: claims much later exp (86400s) — widening.
    let widening_child = build_ucan(
        &middle_kp,
        middle_kp.public_key().to_did().as_str(),
        "/zone/posts",
        "read",
        issuance_secs,
        issuance_secs + 86_400,
    );

    // Leaf-first chain: [child, parent].
    let chain = [widening_child, parent];
    let err = validate_chain_at(&chain, issuance_secs + 30).unwrap_err();
    assert!(
        matches!(err, UcanError::AttenuationViolated { .. }),
        "validate_chain_at must reject child.exp > parent.exp per cap-r4-2 (a); got {err:?}"
    );
    // Drill into the diagnostic context: the violation must name the
    // time-window axis, distinguishing it from the cap-attenuation axis.
    if let UcanError::AttenuationViolated {
        child_cap,
        parent_caps,
        ..
    } = &err
    {
        assert!(
            child_cap.contains("time-window") && child_cap.contains("exp"),
            "violation diagnostic must name the time-window exp axis; got child_cap={child_cap}"
        );
        assert!(
            parent_caps.iter().any(|s| s.contains("exp")),
            "parent_caps must mention the exp constraint; got {parent_caps:?}"
        );
    }
}

#[test]
fn ucan_chain_rejects_child_not_before_earlier_than_parent() {
    // cap-r4-2 (b) pin. The child's nbf MUST NOT precede the parent's
    // nbf; predating rejects at chain-walk with `AttenuationViolated`.
    let root_kp = Keypair::generate();
    let middle_kp = Keypair::generate();
    let issuance_secs = 1_000_000_000u64;

    // Parent: nbf = T+1000.
    let parent = build_ucan(
        &root_kp,
        middle_kp.public_key().to_did().as_str(),
        "/zone/posts",
        "read",
        issuance_secs + 1_000,
        issuance_secs + 5_000,
    );
    // Child: claims nbf = T+500 — earlier than parent → widening backwards.
    let backdating_child = build_ucan(
        &middle_kp,
        middle_kp.public_key().to_did().as_str(),
        "/zone/posts",
        "read",
        issuance_secs + 500,
        issuance_secs + 5_000,
    );

    let chain = [backdating_child, parent];
    // Validate at T+1500 (within both nominal windows so the per-link
    // time-window check does NOT pre-empt the narrowing check).
    let err = validate_chain_at(&chain, issuance_secs + 1_500).unwrap_err();
    assert!(
        matches!(err, UcanError::AttenuationViolated { .. }),
        "validate_chain_at must reject child.nbf < parent.nbf per cap-r4-2 (b); got {err:?}"
    );
    if let UcanError::AttenuationViolated { child_cap, .. } = &err {
        assert!(
            child_cap.contains("time-window") && child_cap.contains("nbf"),
            "violation diagnostic must name the time-window nbf axis; got child_cap={child_cap}"
        );
    }
}

#[test]
fn ucan_chain_validation_at_replay_time_uses_current_clock_not_issuance_clock() {
    // cap-r4-2 (c) pin. `validate_chain_at` MUST use the supplied
    // `now` parameter on every call; no caching of an earlier-passing
    // result. The same chain validated at two `now` values flips
    // disposition based on the supplied wallclock.
    use std::sync::Arc;

    use benten_caps::UCANBackend;
    use benten_graph::RedbBackend;

    let issuer = Keypair::generate();
    let issuance_secs = 1_000_000_000u64;
    let exp_secs = issuance_secs + 60;

    let ucan = build_ucan(
        &issuer,
        issuer.public_key().to_did().as_str(),
        "/zone/posts",
        "read",
        issuance_secs,
        exp_secs,
    );

    // Same durable backend across both validation calls.
    let inner = Arc::new(RedbBackend::open_in_memory().unwrap());
    let backend = UCANBackend::new(inner);
    backend.install_proof(&ucan).unwrap();

    // 1. Validate at issuance + 30: passes (within window).
    backend
        .validate_chain_at(std::slice::from_ref(&ucan), issuance_secs + 30)
        .expect("chain valid at issuance + 30");

    // 2. Same backend / same chain; validate at exp + 30 — MUST reject.
    // The backend MUST re-evaluate the time-window against the
    // supplied `now`, not cache the earlier passing result.
    let err = backend
        .validate_chain_at(std::slice::from_ref(&ucan), exp_secs + 30)
        .expect_err("chain MUST reject post-exp at replay time");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::CapUcanExpired,
        "post-exp replay MUST surface E_CAP_UCAN_EXPIRED per cap-r4-2 (c); got {err:?}"
    );

    // 3. Subsequent call MUST also reject (no negative caching).
    let err2 = backend
        .validate_chain_at(&[ucan], exp_secs + 60)
        .expect_err("subsequent post-exp call MUST also reject");
    assert_eq!(
        err2.code(),
        benten_errors::ErrorCode::CapUcanExpired,
        "subsequent replay-time call MUST also surface E_CAP_UCAN_EXPIRED per cap-r4-2 (c)"
    );
}
