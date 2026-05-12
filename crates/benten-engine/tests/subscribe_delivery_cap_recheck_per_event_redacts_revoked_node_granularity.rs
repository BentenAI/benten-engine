//! Phase 4-Foundation R1-FP wave-1 G22-FP-1 (sec-4f-r1-1 BLOCKER closure)
//! — per-Node cap-recheck-at-delivery elides revoked events from a
//! SUBSCRIBE stream WITHOUT auto-cancelling the whole subscription.
//!
//! Closes the architectural fork ratified by Ben on 2026-05-12 (option-D
//! `CapRecheckOutcome { Keep, Drop, Cancel }` enum). The post-R1-triage
//! ratification #7 wanted per-Node fail-soft silent elision (admin UI
//! per-cap-revocation UX); the Phase-3 R6-FP Wave-C1 SHIPPED contract
//! wanted whole-subscription auto-cancel on cap revoke. Option-D
//! reconciles both by enriching the cap-recheck return type so the
//! eval-side publish loop dispatches `Keep`/`Drop`/`Cancel` per event,
//! and `Engine::on_change_as_with_cursor` wires the per-event
//! `CapabilityPolicy::check_read` gate to return `Drop` on `Err(_)`
//! (silently elide; stream stays open) while keeping the whole-actor-
//! revoke (`is_actor_active=false`) path on `Cancel`.
//!
//! # § 3.6b would-FAIL-if-no-op'd
//!
//! If the per-event `check_read` were reverted to the prior
//! `is_actor_active`-only gate, then revoking a single grant for actor-A
//! would NOT terminate the subscription (the actor is still active), so
//! the post-revoke event in step 4 below would deliver normally and the
//! "callback MUST NOT fire post-revoke" assertion would fire. If the
//! per-event check returned `Cancel` instead of `Drop`, the whole-
//! subscription cancel path would fire and `sub.is_active()==true`
//! step 5 + the post-revoke-on-different-label-still-delivered step 6
//! would both fire.
//!
//! Mirrors the existing principal-aware `check_read` test path
//! exercised by `engine_read_node_as_put_node_pre_v1_closure.rs::
//! grant_read_capability_for_testing_permits_read_after_grant` (PR #209
//! G22-FP-3) but rotates the consumption from `read_node_as` to the
//! SUBSCRIBE delivery seam.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_engine::{Engine, OnChangeCallback};

fn yield_for_dispatch() {
    std::thread::sleep(std::time::Duration::from_millis(20));
}

fn publish_real_event(label: &str, payload: Vec<u8>) -> u64 {
    let seq = benten_eval::primitives::subscribe::next_engine_seq();
    // Construct the event WITH `labels` populated explicitly — the
    // legacy `legacy_minimal` constructor sets `labels = Vec::new()`,
    // and the publish entrypoint uses its `labels: &[String]` arg
    // only for pattern routing (NOT to back-fill the event's own
    // `labels` field). Option-D's per-event `check_read` reads
    // `event.labels.first()` for the `ReadContext.label`, so we must
    // populate the field on the event itself.
    let mut event = benten_eval::primitives::subscribe::ChangeEvent::legacy_minimal(
        benten_core::Cid::from_blake3_digest(*blake3::hash(label.as_bytes()).as_bytes()),
        benten_eval::primitives::subscribe::ChangeKind::Created,
        seq,
        payload,
    );
    event.labels = vec![label.to_string()];
    benten_eval::primitives::subscribe::publish_change_event_with_label(label, event);
    seq
}

/// Option-D acceptance: per-Node cap-recheck Err(_) returns `Drop`
/// (silent elision); the stream stays active; future events the
/// principal still covers continue delivering.
#[test]
fn subscribe_delivery_cap_recheck_per_event_redacts_revoked_node_granularity() {
    // Build engine with `GrantBackedPolicy` so the per-event
    // `CapabilityPolicy::check_read` actually consults grants (NoAuth
    // would permit everything and the test would be vacuous).
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine builds with GrantBackedPolicy");

    // Create actor-A as a principal; the principal CID is the actor_cid
    // threaded into `on_change_as` + the grant binding.
    let actor_a = engine
        .create_principal("subscribe-cap-recheck-actor-a")
        .expect("seed principal");

    // Grant actor-A read coverage for two distinct labels: "post:created"
    // (the revoked-mid-stream label) + "comment:created" (the still-
    // covered label that proves the subscription stays open + future
    // events on OTHER covered labels still deliver). The wildcard
    // variant logic in `wildcard_variants` enumerates 2^N alternatives;
    // we grant the exact concrete `store:<label>:read` scopes here.
    let post_grant_cid = engine
        .grant_capability(&actor_a, "store:post:created:read")
        .expect("post grant must succeed");
    let _comment_grant_cid = engine
        .grant_capability(&actor_a, "store:comment:created:read")
        .expect("comment grant must succeed");

    // Subscribe via `on_change_as` — the per-event `check_read` gate
    // wired in option-D returns `Drop` on `Err(_)` so revoked-mid-
    // stream events silently elide. Pattern matches BOTH labels via
    // glob (we'll publish under both label strings).
    let count_post = Arc::new(AtomicU64::new(0));
    let count_comment = Arc::new(AtomicU64::new(0));
    let cb_post = Arc::clone(&count_post);
    let cb_comment = Arc::clone(&count_comment);
    // Use a single callback that demultiplexes by the chunk payload
    // first byte — we encode the label tag in the payload so the test
    // body can count per-label deliveries without re-architecting the
    // subscription pattern.
    let cb: OnChangeCallback = Arc::new(move |_seq, chunk| {
        if let Some(&tag) = chunk.bytes.first() {
            if tag == 0xAA {
                cb_post.fetch_add(1, Ordering::SeqCst);
            } else if tag == 0xBB {
                cb_comment.fetch_add(1, Ordering::SeqCst);
            }
        }
    });
    // Glob pattern covering both label families. Pattern matching uses
    // `matches_label` against ChangePattern::LabelGlob; `*:created`
    // covers both "post:created" + "comment:created".
    let sub = engine
        .on_change_as("*:created", cb, &actor_a)
        .expect("on_change_as must register");
    assert!(sub.is_active(), "subscription starts active");

    // Step 1+2: publish an event on the post-created label. Actor-A
    // has the grant — event delivers; callback fires once.
    publish_real_event("post:created", vec![0xAA, 1]);
    yield_for_dispatch();
    assert_eq!(
        count_post.load(Ordering::SeqCst),
        1,
        "step 2: pre-revoke event on covered label must deliver"
    );
    assert!(
        sub.is_active(),
        "step 2: subscription still active after first delivery"
    );

    // Step 3: revoke the post:created grant mid-session. Actor-A no
    // longer covers post:created, but still covers comment:created.
    engine
        .revoke_capability_by_grant_cid(&post_grant_cid, &actor_a)
        .expect("revoke post grant");

    // Step 4: publish another event on post:created. Per-event
    // `check_read` returns Err → option-D `Drop` → silent elision.
    // Callback MUST NOT fire.
    publish_real_event("post:created", vec![0xAA, 2]);
    yield_for_dispatch();
    assert_eq!(
        count_post.load(Ordering::SeqCst),
        1,
        "step 4: post-revoke event on revoked label MUST silently elide \
         (callback fires 0 times; total stays at 1)"
    );

    // Step 5: subscription is STILL ACTIVE. Per option-D `Drop` does
    // NOT flip `entry.active.store(false)`. Would-FAIL-if-no-op'd:
    // if the per-event gate returned `Cancel` (the pre-option-D
    // semantic on the bool=false path) instead of `Drop`, this
    // assertion would fire.
    assert!(
        sub.is_active(),
        "step 5: subscription MUST stay active after per-Node revoke \
         (option-D Drop semantic; NOT Cancel)"
    );

    // Step 6: publish an event on the STILL-COVERED comment:created
    // label. Actor-A still has the comment grant — event delivers;
    // callback fires. Proves the subscription is genuinely open and
    // per-Node-granular (not whole-stream-blocked).
    publish_real_event("comment:created", vec![0xBB, 3]);
    yield_for_dispatch();
    assert_eq!(
        count_comment.load(Ordering::SeqCst),
        1,
        "step 6: post-revoke event on a STILL-COVERED label MUST \
         deliver (subscription stays open + per-Node-granular filter)"
    );
    assert!(
        sub.is_active(),
        "step 6: subscription still active after a covered-label delivery"
    );

    // No SUBSCRIBE_REVOKED_MID_STREAM_COUNT bump for the Drop path
    // (option-D contract): silent elision is observably DIFFERENT
    // from the SHIPPED Cancel path's typed-error counter bump.
    // (We can't pin an exact count cross-test because the counter is
    // process-wide; but the test's deliver-vs-elide counts above pin
    // the substantive Drop semantic.)
}
