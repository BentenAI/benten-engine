//! Wave-8c-subscribe-infra (cr-w8c-fp-2 fix-pass): engine-layer
//! integration tests for SUBSCRIBE-side helpers + ESC vectors.
//!
//! # Naming convention (cr-w8c-fp-2)
//!
//! Tests in this file split into two groups per the disposition matrix:
//!
//! - `esc_*` tests drive an actual attack-vector defense end-to-end
//!   through the production engine surface and assert the typed-error
//!   contract / D5 auto-cancel fires. These tests are MEANINGFUL — they
//!   would FAIL if the underlying defense regressed.
//! - `helper_smoke_*` tests pin the engine-layer helpers' observable
//!   round-trip behavior (markers stamped, helpers route through the
//!   production call entry). They DO NOT assert the attack-vector
//!   defense; the defense lives in the eval-side regression suite cited
//!   in the per-test doc comment.
//!
//! # Eval-side defenses cited
//!
//! - ESC-9 (host-fn re-entry / cap-recheck after revoke): pinned by
//!   `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs::sandbox_escape_host_fn_after_cap_revoke`
//!   (line ~276). The eval-side test runs the wasm fixture with a
//!   `testing_yield_for_revoke` host-fn body that revokes mid-call.
//! - ESC-10 (host-fn re-entrancy denial): pinned by
//!   `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied`
//!   (line ~300). Engine-layer integration is structural only.
//! - ESC-13 (uncounted host-fn / D17 BACKSTOP): pinned by
//!   `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs::sandbox_escape_trap_in_fuel_callback_denied`
//!   (line ~359) + the eval-side BACKSTOP code path in
//!   `crates/benten-eval/src/sandbox/`. Engine-layer is helper-smoke only.
//! - ESC-14 (forged cap-claim section): the engine-layer reject
//!   contract is asserted in `esc_14_*` below (drives a SANDBOX
//!   dispatch through `Engine::call` and asserts the typed reject
//!   fires); the eval-side wasm-binary-section parser is pinned by
//!   `crates/benten-eval/tests/sandbox_esc14_forged_cap_claim_section.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(any(test, feature = "test-helpers"))]
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_core::{Cid, Value};
use benten_engine::{Engine, OnChangeCallback, PrimitiveSpec, SubgraphSpec, SubscribeCursor};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

fn actor(name: &str) -> Cid {
    Cid::from_blake3_digest(*blake3::hash(name.as_bytes()).as_bytes())
}

// ---------------------------------------------------------------------------
// ESC-7 — cap revoked mid-stream auto-cancels the subscription
// ---------------------------------------------------------------------------

/// **Real attack-vector defense.** ESC-7 contract: a cap revoked
/// mid-stream causes the next matching change event to fail D5 cap-
/// recheck and auto-cancel the subscription. Pre-fix-pass this test
/// only marked the actor revoked but never published an event, so the
/// auto-cancel path was never exercised. Post-fix-pass we publish a
/// real change event matching the registered pattern and assert (a)
/// the callback is NOT invoked, and (b) the subscription's `is_active()`
/// flips to `false`.
#[test]
fn esc_7_revoke_cap_mid_stream_auto_cancels_subscription_on_next_event() {
    let (engine, _d) = open_engine();
    let alice = actor("alice");

    // Counter the callback bumps when invoked. After revoke + publish,
    // the counter MUST stay at 0.
    let fired = Arc::new(AtomicU64::new(0));
    let fired_for_cb = Arc::clone(&fired);
    let cb: OnChangeCallback = Arc::new(move |_seq, _chunk| {
        fired_for_cb.fetch_add(1, Ordering::SeqCst);
    });

    let sub = engine
        .on_change_as("revoke-test", cb, &alice)
        .expect("on_change_as registers");
    assert!(sub.is_active(), "fresh registration starts active");

    // Revoke alice's cap mid-stream.
    engine.testing_revoke_cap_mid_call(&alice);

    // Publish a real change event matching the registered pattern. The
    // engine's per-event cap-recheck closure consults
    // `inner.is_actor_active(&alice)`, sees the revoked marker, and
    // calls `unregister_on_change` + flips `entry.active` to false.
    let event = benten_eval::primitives::subscribe::ChangeEvent::legacy_minimal(
        actor("event-anchor-1"),
        benten_eval::primitives::subscribe::ChangeKind::Created,
        benten_eval::primitives::subscribe::next_engine_seq(),
        vec![0xDE, 0xAD, 0xBE, 0xEF],
    );
    benten_eval::primitives::subscribe::publish_change_event_with_label("revoke-test", event);

    // Yield briefly so any same-thread dispatch completes.
    std::thread::sleep(std::time::Duration::from_millis(10));

    // (a) Callback was NOT invoked — the cap-recheck closure rejected
    // the event before dispatch.
    assert_eq!(
        fired.load(Ordering::SeqCst),
        0,
        "ESC-7: callback MUST NOT fire after cap is revoked"
    );

    // (b) Subscription auto-cancelled per D5 contract.
    assert!(
        !sub.is_active(),
        "ESC-7: subscription MUST auto-cancel after delivery-time cap-recheck failure"
    );
}

// ---------------------------------------------------------------------------
// ESC-9 — helper smoke (defense lives eval-side)
// ---------------------------------------------------------------------------

/// Helper smoke test (renamed from `esc_9_*` per cr-w8c-fp-2 — this
/// test does NOT exercise the host-fn re-entrancy / cap-recheck-after-
/// revoke defense; it only verifies that the
/// `testing_call_engine_dispatch` helper routes through the production
/// `Engine::call` entry-point and surfaces the same typed-error
/// contract for an unknown handler).
///
/// The actual ESC-9 attack-vector defense is asserted by
/// `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs::
/// sandbox_escape_host_fn_after_cap_revoke` which runs the wasm fixture
/// `host_fn_after_cap_revoke.wat` end-to-end through wasmtime + the
/// production cap-recheck path.
#[test]
fn helper_smoke_call_engine_dispatch_routes_through_production_call() {
    let (engine, _d) = open_engine();
    let outcome = engine.testing_call_engine_dispatch(
        "definitely_not_registered_handler",
        "create",
        std::collections::BTreeMap::<String, Value>::new(),
    );
    assert!(
        outcome.is_err(),
        "helper_smoke: dispatch helper should error for unregistered handler \
         (mirrors Engine::call typed-error contract)"
    );
}

// ---------------------------------------------------------------------------
// ESC-10 — helper smoke (defense lives eval-side)
// ---------------------------------------------------------------------------

/// Helper smoke test (renamed from `esc_10_*` per cr-w8c-fp-2 — pins
/// the helper's stamp behavior on a buffer; does NOT drive the
/// downstream wasm-loader rejection). The actual ESC-10 host-fn re-
/// entrancy denial defense is asserted by
/// `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs::
/// sandbox_escape_reentrancy_via_host_fn_denied` (runs the
/// `reentrancy_via_host_fn.wat` fixture end-to-end). The engine-layer
/// reject contract for forged cap-claim bytes (the upstream impact of
/// a stamped buffer) is asserted in `esc_14_*` below.
#[test]
fn helper_smoke_inject_forged_cap_claim_section_stamps_marker() {
    let mut bytes = vec![0u8; 64];
    benten_engine::Engine::testing_inject_forged_cap_claim_section(&mut bytes);

    // Marker is `0xCC * 8 + "FORGE-CAP"` (17 bytes).
    assert_eq!(
        &bytes[..8],
        &[0xCC; 8],
        "helper_smoke: 8-byte 0xCC prefix stamped"
    );
    assert_eq!(
        &bytes[8..17],
        b"FORGE-CAP",
        "helper_smoke: ASCII FORGE-CAP follows the 0xCC prefix"
    );
}

// ---------------------------------------------------------------------------
// ESC-13 — helper smoke (cr-w8c-fp-3 sideband refactor)
// ---------------------------------------------------------------------------

/// Helper smoke test (cr-w8c-fp-3 fix-pass — uses the new cfg-gated
/// `test_markers` sideband instead of overloading
/// `revoked_actors_for_subscribe`). Pins the helper's marker stamp +
/// query round-trip without coupling to the production cap-revocation
/// set. The actual D17 BACKSTOP at the SANDBOX primitive boundary is
/// asserted by the eval-side `sandbox_escape_attempts_denied` test
/// suite (the trap-in-fuel-callback path covers the trampoline-side
/// CountedSink discipline).
#[test]
fn helper_smoke_register_uncounted_host_fn_records_marker_in_sideband() {
    let (engine, _d) = open_engine();
    let name = "unsafe_host_fn_uncounted";

    // Pre-stamp: marker is absent.
    assert!(
        !engine.testing_has_uncounted_host_fn_marker(name),
        "ESC-13 helper sideband starts empty for the unstamped name"
    );

    engine.testing_register_uncounted_host_fn(name);

    // Post-stamp: marker is present in the cfg-gated test_markers set.
    assert!(
        engine.testing_has_uncounted_host_fn_marker(name),
        "ESC-13 helper records the marker in the test-only sideband"
    );

    // Coupling proof: the marker DOES NOT leak into the production
    // `revoked_actors_for_subscribe` set (cr-w8c-fp-3 decoupling). A
    // fresh ad-hoc onChange registered under the same content-derived
    // CID actor MUST stay active across the marker stamp — pre-fix-pass
    // this would have flipped to inactive on the first delivery because
    // the marker re-used the revocation channel.
    let cid_for_name = Cid::from_blake3_digest(*blake3::hash(name.as_bytes()).as_bytes());
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let sub = engine
        .on_change_as("unrelated:*", cb, &cid_for_name)
        .unwrap();
    assert!(
        sub.is_active(),
        "cr-w8c-fp-3: the uncounted-host-fn marker MUST NOT leak into \
         the production revoked-actors set"
    );

    // Drive a real event matching the registered pattern; the
    // subscription's cap-recheck closure consults the production
    // revoked-actors set (NOT the test_markers sideband), so the
    // delivery succeeds + the subscription stays active.
    let event = benten_eval::primitives::subscribe::ChangeEvent::legacy_minimal(
        actor("event-anchor-13"),
        benten_eval::primitives::subscribe::ChangeKind::Created,
        benten_eval::primitives::subscribe::next_engine_seq(),
        Vec::new(),
    );
    benten_eval::primitives::subscribe::publish_change_event_with_label("unrelated:foo", event);
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(
        sub.is_active(),
        "cr-w8c-fp-3: subscription stays active across an unrelated \
         test_markers stamp"
    );
}

// ---------------------------------------------------------------------------
// ESC-14 — real attack-vector defense (engine-layer reject contract)
// ---------------------------------------------------------------------------

/// **Real attack-vector defense.** ESC-14 contract: corrupted module
/// bytes (carrying a forged cap-claim section marker) MUST be rejected
/// before reaching the executor. The engine layer's reject point is
/// the SANDBOX dispatch path — `Engine::call` against a handler
/// declaring the corrupted CID surfaces a typed `EvalError::Sandbox`
/// (NOT silent acceptance). Pre-fix-pass this test asserted the
/// OPPOSITE — `register_module_bytes` "accepted" the bytes — which is
/// the wrong direction of the contract: `register_module_bytes` is a
/// non-validating blob registry by design (see
/// `crates/benten-engine/src/engine.rs::register_module_bytes` — "blob
/// integrity is the caller's responsibility"). Validation fires at
/// dispatch time inside `benten_eval::sandbox::execute`. This test
/// drives that path end-to-end.
fn sandbox_spec_for_module(handler_id: &str, module_cid_str: &str) -> SubgraphSpec {
    let mut sandbox_props: BTreeMap<String, Value> = BTreeMap::new();
    sandbox_props.insert("module".into(), Value::Text(module_cid_str.to_string()));
    sandbox_props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: sandbox_props,
        })
        .respond()
        .build()
}

#[test]
fn esc_14_sandbox_dispatch_rejects_forged_cap_claim_module_bytes() {
    let (engine, _d) = open_engine();

    // Stamp the forged marker into a 128-byte buffer (NOT a valid wasm
    // module — wasm modules begin with `\0asm` magic).
    let mut bytes = vec![0u8; 128];
    Engine::testing_inject_forged_cap_claim_section(&mut bytes);

    // Compute a content-addressed CID for the corrupted bytes + register
    // them in the engine's durable blob store (Compromise #17 closed at
    // G14-C — `register_module_bytes` now validates CID + persists via
    // `RedbBlobBackend`).
    let module_cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());
    let module_cid_str = module_cid.to_base32();
    engine.register_module_bytes(&module_cid, &bytes).unwrap();

    // Register a SANDBOX-bearing handler whose `module` property
    // names the corrupted CID. The handler registers cleanly — the
    // SUBGRAPH spec is structurally valid even though the module bytes
    // are garbage.
    let spec = sandbox_spec_for_module("esc_14_sandbox_rejects_forged", &module_cid_str);
    let handler_id = engine
        .register_subgraph(spec)
        .expect("ESC-14: corrupted module CID handler registers (validation is at dispatch time)");

    // Drive the SANDBOX dispatch through the production `Engine::call`
    // entry-point. The wasmtime-side parser rejects the corrupted
    // bytes synchronously; the typed `EvalError::Sandbox` variant
    // surfaces at the engine boundary.
    let err = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], BTreeMap::new()),
        )
        .expect_err(
            "ESC-14: SANDBOX dispatch with corrupted module bytes MUST \
             surface a typed engine error (the bytes are not a valid \
             wasm module + the engine reject must fire)",
        );

    // The catalog code surfaces from the typed `EvalError::Sandbox`
    // path — the exact discriminant is one of the SANDBOX-family
    // codes (`E_SANDBOX_MODULE_INVALID`, `E_SANDBOX_FORGED_CAP_CLAIM`,
    // or `E_SANDBOX_*` depending on which validation pass catches the
    // corruption first). The contract is: NOT
    // `Unknown("E_EVAL_BACKEND")` (the wave-8b placeholder that would
    // mean validation never ran) and NOT a successful Outcome.
    let code = err.code();
    let code_str = code.as_str();
    assert!(
        code_str.starts_with("E_SANDBOX_"),
        "ESC-14: rejection MUST surface a typed E_SANDBOX_* catalog \
         code; got {code_str:?} (full error: {err:?})"
    );
    assert_ne!(
        code,
        ErrorCode::Unknown(String::from("E_EVAL_BACKEND")),
        "ESC-14: rejection MUST NOT collapse into the wave-8b \
         `EvalError::Backend(String)` placeholder; got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Surface assurance — the helper smoke tests don't bleed into ESC-7's
// production-cap-revocation path (post cr-w8c-fp-3 sideband refactor).
// ---------------------------------------------------------------------------

#[test]
fn esc_7_and_test_markers_are_disjoint() {
    // cr-w8c-fp-3 acceptance: stamping a test_markers entry MUST NOT
    // affect production cap-recheck for unrelated actors.
    let (engine, _d) = open_engine();

    // Stamp a test marker first.
    engine.testing_register_uncounted_host_fn("marker-name");

    // Now register a real authenticated subscription for an unrelated
    // actor whose CID happens to differ from the marker's CID.
    let bob = actor("bob");
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let sub = engine
        .on_change_as("topic", cb, &bob)
        .expect("on_change_as registers");
    assert!(sub.is_active());

    // Publish a real event. Bob is not revoked, so delivery succeeds +
    // the subscription stays active.
    let _ = SubscribeCursor::Latest;
    let event = benten_eval::primitives::subscribe::ChangeEvent::legacy_minimal(
        actor("event-anchor-bob"),
        benten_eval::primitives::subscribe::ChangeKind::Created,
        benten_eval::primitives::subscribe::next_engine_seq(),
        Vec::new(),
    );
    benten_eval::primitives::subscribe::publish_change_event_with_label("topic", event);
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(
        sub.is_active(),
        "cr-w8c-fp-3 disjointness: test_markers stamp must not leak \
         into the production cap-revocation path"
    );
}
