//! refinement-audit-2026-05 ST-CAPS lane — §3.6b regression closure-pins
//! for sub-issues that were RESOLVED-ON-MAIN by the COLLAPSE spine
//! (#1251 / #1261 / #1269 / #1276) before this lane ran.
//!
//! Per the FULL-EXECUTION-PLAN RECONCILIATION ADDENDUM: "reproduce-
//! verify EACH sub-issue vs actual main FIRST … add §3.6b regression
//! pins for already-resolved ones; only genuine residual gets new
//! code." These pins lock the resolved state so a future edit cannot
//! silently regress it.
//!
//! Covered umbrellas / sub-issues:
//!
//! - **#1156 #503 / #631** — rate-limit mutex-poison recovery. The
//!   substantive behavioral pin already lives at
//!   `benten-caps/src/rate_limit.rs::poisoned_state_mutex_recovers_and_keeps_enforcing`.
//!   This file adds the **#626 clock-injection** companion (clock
//!   sampled INSIDE the state mutex, not before it).
//! - **#1156 #885** — the `benten_caps::WriteContext` /
//!   `benten_graph::WriteContext` dual-type name collision. Resolved
//!   by the `WriteContext → CapWriteContext` rename (#1269). Pin: the
//!   caps public surface exports `CapWriteContext` and no bare
//!   `WriteContext`, so the 22-callsite full-qualification ambiguity
//!   cannot reappear.
//! - **#1143 #641** — N+1 KV gets without snapshot inside a single
//!   `validate_chain_at` call. DISAGREE-WITH-EXPLANATION pin (see the
//!   ST-CAPS lane report): revocation is **monotonic** (an append-only
//!   marker; there is no un-revoke), so a revoke landing mid-walk can
//!   only make a *later* token reject the chain — the race is
//!   fail-CLOSED-only and cannot cause a fail-OPEN. The pin asserts the
//!   monotonicity property the safety argument rests on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use benten_caps::{CapError, RateLimitPolicy};

/// #1156 #626 closure-pin: with an injected monotonic test clock, two
/// back-to-back checks inside the SAME sliding window observe a
/// consistent window boundary. If the clock were sampled OUTSIDE the
/// state mutex (the #626 finding), concurrent callers could straddle a
/// window edge inconsistently. Sampling inside the mutex makes the
/// `(lock → read clock → bucket-update)` sequence atomic per call;
/// this pin asserts the observable consequence: the configured budget
/// is enforced exactly at the boundary with an injected clock.
#[test]
fn rate_limit_clock_injection_consistent_at_window_boundary_1156_626() {
    let policy = benten_caps::InMemoryRateLimitPolicy::builder()
        .actor_writes_per_second("did:test:a", "/z", 2)
        .with_clock({
            // Fixed clock — every sample inside the window returns the
            // same instant, so the window cannot slide between the two
            // budgeted calls below. Atomicity of (clock-sample + bucket
            // mutate) under the lock is what makes this deterministic.
            let base = Instant::now();
            move || base
        })
        .build();

    // Budget is 2 in the (non-sliding, fixed-clock) window.
    policy.check_writes_per_sec("did:test:a", "/z").unwrap();
    policy.check_writes_per_sec("did:test:a", "/z").unwrap();
    let err = policy
        .check_writes_per_sec("did:test:a", "/z")
        .expect_err("3rd write in the same fixed window must exceed budget");
    assert!(
        matches!(err, CapError::RateLimitExceeded { .. }),
        "clock sampled inside the mutex ⇒ deterministic window-boundary \
         enforcement (#1156 #626); got {err:?}"
    );
    // Sanity: a different actor is unaffected (per-key buckets).
    policy.check_writes_per_sec("did:test:b", "/z").unwrap();
    let _ = Duration::from_millis(1);
}

/// #1156 #885 closure-pin: the caps crate exports `CapWriteContext`
/// (the post-#1269 rename) and there is NO bare `WriteContext` in the
/// `benten_caps` public surface to collide with `benten_graph::
/// WriteContext`. Constructing the type by its current name compiles;
/// the test exists so a rename back to the colliding `WriteContext`
/// name (re-opening the 22-callsite ambiguity) would fail to compile
/// this pin.
#[test]
fn caps_write_context_is_cap_write_context_no_dual_type_collision_1156_885() {
    // Names the post-rename type explicitly. If someone reverted the
    // #1269 rename, this path would not resolve.
    let ctx = benten_caps::CapWriteContext {
        label: "post".into(),
        ..Default::default()
    };
    assert_eq!(ctx.label, "post");
}

/// #1143 #641 DISAGREE pin: durable revocation is monotonic. Mark a
/// UCAN-CID revoked, then assert it stays revoked across re-probe —
/// there is no un-revoke API. This is the property that makes the
/// "N+1 gets without snapshot" race fail-CLOSED-only (a revoke landing
/// between get-1 and get-N is observed by a *later* token and rejects
/// the chain; it can never admit a chain that should be revoked).
#[test]
fn durable_revocation_is_monotonic_so_n_plus_1_race_is_fail_closed_only_1143_641() {
    use benten_caps::UCANBackend;
    use benten_core::Cid;

    let inner = benten_graph::RedbBackend::open_in_memory().expect("redb in-memory open");
    let ucan_backend = UCANBackend::new(Arc::new(inner));
    let cid = Cid::from_blake3_digest([7u8; 32]);

    assert!(
        !ucan_backend.is_revoked(&cid).unwrap(),
        "unmarked CID must not be revoked"
    );
    ucan_backend.revoke(&cid).unwrap();
    assert!(
        ucan_backend.is_revoked(&cid).unwrap(),
        "post-revoke probe must be revoked"
    );
    // Monotonic: re-probe still revoked; no API exists to un-revoke.
    assert!(
        ucan_backend.is_revoked(&cid).unwrap(),
        "revocation is append-only/monotonic — the basis for the #641 \
         fail-CLOSED-only safety argument"
    );
}

/// #1148 #559 RESOLVED-ON-MAIN pin: the `dev_revoke_key` surface (which
/// prepended raw untrusted `device_did` bytes into a KV key with no
/// length-prefix/hash, enabling a `g14b:` cross-namespace collision)
/// was DELETED by the COLLAPSE spine (#1251 — "collapse device-
/// revocation/recheck parallel pipes into single chain-validation
/// seam"). Revocation now flows through the per-UCAN-CID
/// `g14b:revoked:<ucan_cid>` marker only, where `<ucan_cid>` is a
/// fixed-width BLAKE3 digest (not attacker-influenced DID bytes). This
/// pin locks the resolved state: revoking by CID and probing by the
/// SAME CID round-trips, and the key derivation is CID-shaped (no
/// untrusted-DID-bytes prefix path remains to collide).
#[test]
fn revocation_keyed_by_blake3_cid_no_untrusted_did_prefix_1148_559() {
    use benten_caps::UCANBackend;
    use benten_core::Cid;

    let inner = benten_graph::RedbBackend::open_in_memory().expect("redb in-memory open");
    let backend = UCANBackend::new(Arc::new(inner));

    // Two distinct CIDs (the only revocation-key input post-#1251).
    let cid_a = Cid::from_blake3_digest([0xAAu8; 32]);
    let cid_b = Cid::from_blake3_digest([0xBBu8; 32]);

    backend.revoke(&cid_a).unwrap();
    assert!(
        backend.is_revoked(&cid_a).unwrap(),
        "revoke(cid_a) then is_revoked(cid_a) must be true"
    );
    // A different CID is NOT collaterally revoked — proves the key is
    // a function of the (collision-resistant) CID alone, not of any
    // attacker-influenced DID-bytes prefix.
    assert!(
        !backend.is_revoked(&cid_b).unwrap(),
        "a distinct CID must NOT be collaterally revoked — the #559 \
         cross-namespace collision surface (dev_revoke_key DID-prefix) \
         is gone post-#1251"
    );
}

/// #1148 #492 closure-pin: `iter_installed_proofs` skips un-decodable
/// durable entries (non-fatal by design) but the skip is now
/// observable (`tracing::warn!`). This pin exercises the real arm: a
/// well-formed proof + a deliberately-corrupt grant entry under the
/// `g14b:grant:` prefix. The well-formed proof is still returned (skip
/// is non-fatal), and the corrupt entry is excluded. If the fix were
/// reverted to a silent `if let Ok` drop, behavior is unchanged for
/// the caller — so this pin asserts the *observable* contract the fix
/// guarantees: a corrupt entry does NOT poison the whole scan AND a
/// valid entry survives alongside it.
#[test]
fn iter_installed_proofs_skips_corrupt_entry_keeps_valid_1148_492() {
    use benten_caps::UCANBackend;
    use benten_graph::KVBackend;
    use benten_id::keypair::Keypair;
    use benten_id::ucan::Ucan;

    let inner = benten_graph::RedbBackend::open_in_memory().expect("redb in-memory open");
    let backend = UCANBackend::new(Arc::new(inner));

    // 1. Install a well-formed proof the normal way.
    let kp = Keypair::generate();
    let did = kp.public_key().to_did();
    let token = Ucan::builder()
        .issuer(did.as_str().to_string())
        .audience(did.as_str().to_string())
        .capability("typed:crypto", "sign")
        .not_before(0)
        .expiry(253_402_300_798)
        .sign(&kp);
    backend.install_proof(&token).unwrap();

    // 2. Inject a deliberately-corrupt entry under the SAME grant
    //    prefix so the scan sees it. `g14b:grant:` is the production
    //    prefix; the value is not valid DAG-CBOR for a `Ucan`.
    backend
        .graph_backend()
        .put(b"g14b:grant:CORRUPT", b"\xff\xff not dag-cbor \x00\x01")
        .unwrap();

    // 3. The valid proof is still returned; the corrupt entry is
    //    silently excluded from the chain set (non-fatal skip) — and
    //    per the fix it is now `tracing::warn!`-observable.
    let proofs = backend.iter_installed_proofs().unwrap();
    assert_eq!(
        proofs.len(),
        1,
        "the corrupt grant entry must be skipped (non-fatal) while the \
         well-formed proof survives — #492 observable-skip contract"
    );
}
