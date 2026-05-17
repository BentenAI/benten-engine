//! Phase-4-Foundation R1-FP G22-FP-2 (cap-r1-1 + cap-r1-9 BLOCKER
//! closure): `UcanGroundedPolicy::typed_cap_permitted_by_proof` MUST
//! bind the chain's `audience()` to the active principal DID BEFORE
//! the chain-walker time-window check, per the
//! `validate_chain_for_audience_at` ordering precedent at
//! `crates/benten-caps/src/backends/ucan.rs` lines 487-510.
//!
//! ## What this pin asserts (§3.6b: production runtime arm + observable consequence + would-FAIL-if-no-op'd)
//!
//! Three production-runtime paths through
//! `UcanGroundedPolicy::check_write` + an ordering proof:
//!
//! 1. **Valid audience + valid time-window → ACCEPT.** Baseline:
//!    audience-binding gate does NOT spuriously reject when the chain
//!    is actually issued to the active principal.
//!
//! 2. **Wrong audience + valid time-window → REJECT with
//!    `UcanAudienceMismatch`.** The cap-r1-1 BLOCKER itself: pre-fix
//!    the chain walked with `validate_chain_at` (no audience binding)
//!    and ACCEPTED the proof; post-fix the chain walks with
//!    `validate_chain_for_audience_at` and the leaf-audience
//!    constant-time-compare against the active principal DID rejects.
//!    `typed_cap_permitted_by_proof` treats the audience-mismatch as
//!    "this chain does not permit," iteration continues, no other
//!    proof matches, and the outer `check_write` surfaces the
//!    grant-backed denial (`CapError::Denied`).
//!
//! 3. **Valid audience + expired time-window → REJECT.** Companion:
//!    audience-binding gate does NOT mask the time-window check —
//!    once audience matches, the chain-walker time-window check still
//!    fires + rejects expired chains.
//!
//! 4. **Ordering proof: wrong audience + expired time-window → MUST
//!    surface audience-denial typed error, NOT time-window typed
//!    error.** When BOTH gates would reject, the audience-binding
//!    gate MUST fire FIRST. This proves `validate_chain_for_audience_at`
//!    composes `validate_chain_for_audience` BEFORE `validate_chain_at`
//!    rather than the reverse — matches the `validate_chain_inner`
//!    surface at `crates/benten-id/src/ucan.rs` lines 397-410 where
//!    audience is checked before the per-link time-window walk.
//!
//! ## Would-FAIL-if-no-op'd analysis (§3.6b)
//!
//! - Path 2 fails if the audience-binding wiring is reverted:
//!   `validate_chain_at` (audience-less) would silently ACCEPT the
//!   wrong-audience proof — the test's `is_err` assertion would flip
//!   to `is_ok`.
//! - Path 4 fails if the ordering is inverted (time-window before
//!   audience): the typed error returned by
//!   `UCANBackend::validate_chain_for_audience_at` would carry
//!   `CapError::UcanTimeWindow*` rather than
//!   `CapError::UcanAudienceMismatch`. The discriminant check on
//!   `CapError::UcanAudienceMismatch` in path 4 would fail.
//!
//! Removing the new `audience` parameter from
//! `typed_cap_permitted_by_proof` + reverting to `validate_chain_at`
//! would make the entire file's wrong-audience assertions flip
//! — observable behavior change at the production write path.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{
    CapError, CapWriteContext, CapabilityPolicy, GrantBackedPolicy, GrantReader, UCANBackend,
    UcanGroundedPolicy,
};
use benten_graph::RedbBackend;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;

/// `GrantReader` that always denies — forces the typed-cap proof-chain
/// path in `UcanGroundedPolicy::check_write` (the audience-binding
/// gate under test).
#[derive(Debug)]
struct DenyAllGrantReader;

impl GrantReader for DenyAllGrantReader {
    fn has_unrevoked_grant_for_scope(&self, _scope: &str) -> Result<bool, CapError> {
        Ok(false)
    }
}

fn fresh_backend() -> UCANBackend<RedbBackend> {
    let inner = RedbBackend::open_in_memory().expect("redb in-memory open");
    UCANBackend::new(Arc::new(inner))
}

fn fresh_policy(
    ucan: Arc<UCANBackend<RedbBackend>>,
    now_secs: u64,
) -> UcanGroundedPolicy<RedbBackend> {
    let inner = GrantBackedPolicy::new(Arc::new(DenyAllGrantReader));
    UcanGroundedPolicy::new(inner, ucan).with_now_for_test(now_secs)
}

/// Build a UCAN issued by `issuer_kp` for `audience_did` granting
/// `(resource, ability)` in the `[nbf, exp]` window.
fn build_ucan_for_audience(
    issuer_kp: &Keypair,
    audience_did: &str,
    resource: &str,
    ability: &str,
    nbf: u64,
    exp: u64,
) -> Ucan {
    Ucan::builder()
        .issuer(issuer_kp.public_key().to_did().as_str().to_string())
        .audience(audience_did.to_string())
        .capability(resource, ability)
        .not_before(nbf)
        .expiry(exp)
        .sign(issuer_kp)
}

// ---------------------------------------------------------------------------
// Path 1: valid audience + valid time-window → ACCEPT
// ---------------------------------------------------------------------------

#[test]
fn valid_audience_plus_valid_time_window_permits() {
    let backend = Arc::new(fresh_backend());
    // Issuer = audience = same keypair (self-issued grant; simplest
    // shape exercising the audience-binding gate with NO cross-actor
    // dimension).
    let kp = Keypair::generate();
    let active_did = kp.public_key().to_did().as_str().to_string();

    // Chain audience = active principal DID; time window: [0, far future].
    let token =
        build_ucan_for_audience(&kp, &active_did, "typed:crypto", "sign", 0, 253_402_300_798);
    backend.install_proof(&token).unwrap();

    let policy = fresh_policy(Arc::clone(&backend), 1_000_000_000);
    let ctx = CapWriteContext {
        label: "cap:typed:crypto-sign".to_string(),
        scope: "cap:typed:crypto-sign".to_string(),
        actor_hint: Some(active_did.clone()),
        ..Default::default()
    };

    assert!(
        policy.check_write(&ctx).is_ok(),
        "valid audience + valid time-window MUST permit (cap-r1-1 \
         baseline: audience-binding gate does NOT spuriously reject \
         when audience matches active principal)"
    );
}

// ---------------------------------------------------------------------------
// Path 2: wrong audience + valid time-window → REJECT (the cap-r1-1 BLOCKER)
// ---------------------------------------------------------------------------

#[test]
fn wrong_audience_plus_valid_time_window_rejects() {
    let backend = Arc::new(fresh_backend());
    let issuer_kp = Keypair::generate();
    let victim_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();

    // UCAN issued to the VICTIM (audience = victim_did), but the
    // attacker is the active principal at write-time. Pre-cap-r1-1-fix
    // this would have been ACCEPTED — the audience field was never
    // checked.
    let victim_did = victim_kp.public_key().to_did().as_str().to_string();
    let attacker_did = attacker_kp.public_key().to_did().as_str().to_string();
    let token = build_ucan_for_audience(
        &issuer_kp,
        &victim_did,
        "typed:crypto",
        "sign",
        0,
        253_402_300_798,
    );
    backend.install_proof(&token).unwrap();

    let policy = fresh_policy(Arc::clone(&backend), 1_000_000_000);
    let ctx = CapWriteContext {
        label: "cap:typed:crypto-sign".to_string(),
        scope: "cap:typed:crypto-sign".to_string(),
        actor_hint: Some(attacker_did.clone()),
        ..Default::default()
    };

    let err = policy.check_write(&ctx).expect_err(
        "wrong audience + valid time-window MUST reject (cap-r1-1 \
         BLOCKER: pre-fix the chain walked without audience binding \
         and silently ACCEPTED a UCAN issued to someone else)",
    );

    // Audience-mismatch causes the chain-walker to skip the proof;
    // `typed_cap_permitted_by_proof` returns `Ok(false)` and the
    // outer `check_write` surfaces the grant-backed denial. The
    // typed audience-mismatch error fires only on the
    // missing-principal short-circuit (no actor_hint at all); on the
    // wrong-audience-but-principal-bound path the typed error
    // surfaces from the chain-walker but is consumed by the
    // iterate-to-next-proof behavior. The OBSERVABLE distinguishing
    // assertion is: under pre-fix behavior this returned `Ok(())`;
    // under post-fix it returns ANY `Err(_)`.
    assert!(
        matches!(err, CapError::Denied { .. }),
        "wrong-audience-but-principal-bound surfaces grant-backed \
         denial via iterate-to-next behavior; got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Path 2b: missing principal (no actor_hint) + typed-cap requirement
//   → PERMIT via legacy audience-less chain walk (engine-internal path
//   compatibility; mirrors FP-3 default-collapses-to-scope-only-when-
//   actor-None pattern)
// ---------------------------------------------------------------------------

#[test]
fn missing_principal_did_with_typed_cap_requirement_falls_back_to_audience_less_walk() {
    let backend = Arc::new(fresh_backend());
    let kp = Keypair::generate();
    let token = build_ucan_for_audience(
        &kp,
        kp.public_key().to_did().as_str(),
        "typed:crypto",
        "sign",
        0,
        253_402_300_798,
    );
    backend.install_proof(&token).unwrap();

    let policy = fresh_policy(Arc::clone(&backend), 1_000_000_000);
    // No actor_hint, no actor_cid → no principal DID resolvable.
    // Post-fix the walker falls back to `validate_chain_at`
    // (audience-less; legacy pre-fix walker) so the proof is judged on
    // signature + time-window + attenuation + revocation alone.
    // Preserves engine-internal typed-CALL paths that don't yet thread
    // actor (e.g., `Engine::dispatch_typed_call_public` at
    // `engine_wait.rs::881-891`). Full actor-threading is the
    // cap-r1-16 + CapWriteContext::now follow-up at G24-D files-owned.
    let ctx = CapWriteContext {
        label: "cap:typed:crypto-sign".to_string(),
        scope: "cap:typed:crypto-sign".to_string(),
        actor_hint: None,
        ..Default::default()
    };

    assert!(
        policy.check_write(&ctx).is_ok(),
        "typed-cap requirement + no principal DID MUST PERMIT via \
         legacy audience-less chain walk (preserves Phase-1/2 fixtures \
         + engine-internal typed-CALL paths). Audience binding only \
         fires when caller threads a principal."
    );
}

// ---------------------------------------------------------------------------
// Path 3: valid audience + expired time-window → REJECT (time-window still fires)
// ---------------------------------------------------------------------------

#[test]
fn valid_audience_plus_expired_time_window_rejects_via_time_check() {
    let backend = Arc::new(fresh_backend());
    let kp = Keypair::generate();
    let active_did = kp.public_key().to_did().as_str().to_string();

    // Chain audience = active principal DID; time window: [0, 100],
    // but we'll evaluate at now=200 → expired.
    let token = build_ucan_for_audience(&kp, &active_did, "typed:crypto", "sign", 0, 100);
    backend.install_proof(&token).unwrap();

    let policy = fresh_policy(Arc::clone(&backend), 200);
    let ctx = CapWriteContext {
        label: "cap:typed:crypto-sign".to_string(),
        scope: "cap:typed:crypto-sign".to_string(),
        actor_hint: Some(active_did.clone()),
        ..Default::default()
    };

    let err = policy.check_write(&ctx).expect_err(
        "valid audience + expired time-window MUST reject (companion: \
         audience-binding gate does NOT mask the time-window check)",
    );

    // The audience gate passes; the time-window check fires inside the
    // chain-walker; `typed_cap_permitted_by_proof` treats the failure
    // as "this chain does not permit" + iterates → grant-backed denial
    // bubbles.
    assert!(
        matches!(err, CapError::Denied { .. }),
        "valid-audience-expired-chain surfaces grant-backed denial via \
         iterate-to-next behavior; got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Path 4: ORDERING PROOF — wrong audience + expired time-window
//   → audience-binding error surfaces (NOT time-window error)
// ---------------------------------------------------------------------------

/// This is the load-bearing ordering pin per cap-r1-9. To distinguish
/// audience-FIRST from time-window-FIRST, we drive
/// `UCANBackend::validate_chain_for_audience_at` directly with a
/// chain that's BOTH wrong-audience AND expired, and assert the typed
/// error is `UcanAudienceMismatch`, NOT a time-window code.
///
/// Driving the backend method directly (rather than
/// `UcanGroundedPolicy::check_write`) avoids the iterate-to-next
/// behavior in `typed_cap_permitted_by_proof` that consumes typed
/// errors as "this chain does not permit." The backend's
/// `validate_chain_for_audience_at` is what
/// `typed_cap_permitted_by_proof` calls underneath; the ordering
/// inside that method is what the gate inherits.
#[test]
fn wrong_audience_plus_expired_chain_surfaces_audience_mismatch_first() {
    let backend = fresh_backend();
    let issuer_kp = Keypair::generate();
    let victim_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();

    // Chain: audience = victim, but expired (exp=100 against now=200).
    let victim_did_str = victim_kp.public_key().to_did().as_str().to_string();
    let token =
        build_ucan_for_audience(&issuer_kp, &victim_did_str, "typed:crypto", "sign", 0, 100);

    // Validate the chain against the ATTACKER's DID at a time AFTER
    // the chain's exp. Both gates would reject; the question is which
    // typed error surfaces.
    let attacker_did = attacker_kp.public_key().to_did();
    let err = backend
        .validate_chain_for_audience_at(std::slice::from_ref(&token), &attacker_did, 200)
        .expect_err(
            "wrong audience + expired chain MUST reject (BOTH gates \
             would fire; this assertion proves the audience gate \
             fires FIRST per cap-r1-9 ordering)",
        );

    // The load-bearing assertion: audience-mismatch SURFACES, time-
    // window error DOES NOT. If the ordering were reversed (time-
    // window before audience), this would surface
    // `CapError::UcanTimeWindow*` and the discriminant check would
    // fail.
    assert!(
        matches!(err, CapError::UcanAudienceMismatch { .. }),
        "ordering proof FAILED: expected CapError::UcanAudienceMismatch \
         to surface BEFORE the time-window check; got {err:?}. \
         If the typed code is a time-window variant, the \
         validate_chain_for_audience_at ordering has been inverted \
         (cap-r1-9 regression)."
    );
}
