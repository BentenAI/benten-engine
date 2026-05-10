//! Phase-3 G21-T3 Section D — combined fix-pass pins.
//!
//! Covers the four Section D fold-ins from the G21-T3 brief:
//!
//! 1. **§2.2 SUBSCRIBE delivery-time cap-recheck partial-revoke pin**
//!    — drive the eval-side delivery loop's cap-recheck closure so
//!    a partial-revoke (only ONE subscription path's cap removed)
//!    cancels the affected path while leaving unrelated subscriptions
//!    delivering. Exercises the production cap-recheck arm at
//!    `crates/benten-eval/src/primitives/subscribe.rs::publish_change_event_with_labels`.
//!
//! 2. **corr-minor-1 — UCAN forged-sig pin** — forge a UCAN token
//!    with a tampered signature → run through `ucan_validate_chain`
//!    typed-CALL op → assert `valid: false` (the production defense
//!    surfaces the rejection without panicking on the malformed
//!    sig).
//!
//! 3. **corr-minor-2 — engine.call full-stack typed-CALL test** —
//!    drive `Engine::dispatch_typed_call` end-to-end through a
//!    typed-CALL op + assert the result Map shape threads through.
//!    The integration of typed-CALL ops INSIDE a registered subgraph
//!    (where a CALL Node carries `target: "engine:typed:..."`) is
//!    pinned at `crates/benten-engine/tests/typed_call_engine_dispatch.rs::ed25519_sign_then_verify_round_trip_via_dispatch_typed_call`
//!    via the eval-side `call::execute` fork; this file's test
//!    drives the engine-layer entry point.
//!
//! 4. **audit-6-3 — DeviceAttestation handshake integration test
//!    SCAFFOLD** — pins the integration-test SHAPE for the eventual
//!    end-to-end flow (declare on peer A → join → peer B sees in
//!    handshake metadata). The full wire-up is owned by G21-T2 (napi
//!    bridge to engine). Until then, this test pins the engine-side
//!    `AtriumHandle` surface that T2 will widen + asserts the
//!    handshake-wire `device_did` field carries through. See
//!    audit-6-3 in `.addl/phase-3/g20-pre-r6-audits/audit-6-napi-
//!    wireup-drift.md`.

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use benten_core::{Cid, OperationNode, Value};
use benten_engine::Engine;
use benten_eval::{PrimitiveHost, PrimitiveKind, TypedCallOp};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

fn map_value(pairs: &[(&str, Value)]) -> Value {
    let mut m = BTreeMap::new();
    for (k, v) in pairs {
        m.insert((*k).to_string(), v.clone());
    }
    Value::Map(m)
}

// =====================================================================
// §2.2 SUBSCRIBE partial-revoke pin
// =====================================================================

/// Phase-3 §2.2 fold-in: drive the eval-side SUBSCRIBE delivery
/// cap-recheck closure with a custom recheck that returns false for
/// ONE subscription path's anchor + true for another. Asserts the
/// affected subscription is auto-cancelled (active flag flipped to
/// false) while the unrelated subscription continues to deliver
/// normally. Exercises the production cap-recheck arm in
/// `crates/benten-eval/src/primitives/subscribe.rs::publish_change_event_with_labels`
/// (the loop at lines 1238-1247).
///
/// This test is the IN-PROCESS analogue of the engine-layer
/// `subscribe_partial_revoke_cancels_subscription_path` red-phase
/// pin which is `#[ignore]`'d pending G14-D durable backend wireup.
/// The eval-side path runs today; this pin proves the partial-
/// revoke contract through the existing closure surface.
#[test]
#[allow(clippy::too_many_lines)] // Round-1/Round-2/Round-3 narrative is the test contract.
fn subscribe_partial_revoke_via_cap_recheck_closure_cancels_only_affected_path() {
    use benten_core::change_stream::{ChangeEvent, ChangeKind};
    use benten_eval::primitives::subscribe::{
        ChangePattern, DeliveryCapRecheck, OnChangeDeliveryCallback, SubscribeCursor,
        next_engine_seq, publish_change_event_with_labels, register_on_change,
        unregister_on_change,
    };

    // Two subscriptions on different anchors. Subscription A's
    // cap_recheck always returns true (cap stable). Subscription B's
    // cap_recheck observes a shared atomic flag that flips after one
    // delivery — simulating partial revocation of B's grant only.

    let a_calls = Arc::new(AtomicU64::new(0));
    let b_calls = Arc::new(AtomicU64::new(0));
    let b_revoked = Arc::new(AtomicBool::new(false));

    let a_calls_cb = Arc::clone(&a_calls);
    let cb_a: OnChangeDeliveryCallback = Arc::new(move |_evt: &ChangeEvent| {
        a_calls_cb.fetch_add(1, Ordering::SeqCst);
    });
    let b_calls_cb = Arc::clone(&b_calls);
    let cb_b: OnChangeDeliveryCallback = Arc::new(move |_evt: &ChangeEvent| {
        b_calls_cb.fetch_add(1, Ordering::SeqCst);
    });

    let recheck_a: DeliveryCapRecheck = Arc::new(|_evt: &ChangeEvent| true);
    let revoked_clone = Arc::clone(&b_revoked);
    let recheck_b: DeliveryCapRecheck = Arc::new(move |_evt: &ChangeEvent| {
        // Returns true while cap is held; flips false on revoke.
        !revoked_clone.load(Ordering::SeqCst)
    });

    let active_a = Arc::new(AtomicBool::new(true));
    let active_b = Arc::new(AtomicBool::new(true));
    let max_seq_a = Arc::new(AtomicU64::new(0));
    let max_seq_b = Arc::new(AtomicU64::new(0));

    let id_a = register_on_change(
        ChangePattern::LabelGlob("Posts:*".to_string()),
        SubscribeCursor::Latest,
        cb_a,
        Some(recheck_a),
        Arc::clone(&active_a),
        Arc::clone(&max_seq_a),
        None,
        Arc::new(std::sync::Mutex::new(None)),
    )
    .unwrap();
    let id_b = register_on_change(
        ChangePattern::LabelGlob("Admin:*".to_string()),
        SubscribeCursor::Latest,
        cb_b,
        Some(recheck_b),
        Arc::clone(&active_b),
        Arc::clone(&max_seq_b),
        None,
        Arc::new(std::sync::Mutex::new(None)),
    )
    .unwrap();

    // Helper: build a minimal ChangeEvent for a given label.
    let mk_event = |label: &str| {
        ChangeEvent::legacy_minimal(
            Cid::from_blake3_digest(*blake3::hash(label.as_bytes()).as_bytes()),
            ChangeKind::Created,
            next_engine_seq(),
            Vec::new(),
        )
    };

    // Round 1: emit an event for each subscription's pattern. Both
    // recheck closures return true → both deliver.
    let evt_posts = mk_event("Posts:hello");
    let evt_admin = mk_event("Admin:settings");
    publish_change_event_with_labels(&["Posts:hello".to_string()], evt_posts);
    publish_change_event_with_labels(&["Admin:settings".to_string()], evt_admin);

    assert_eq!(
        a_calls.load(Ordering::SeqCst),
        1,
        "A subscription MUST receive the matching event"
    );
    assert_eq!(
        b_calls.load(Ordering::SeqCst),
        1,
        "B subscription MUST receive the matching event"
    );

    // Partial revoke: flip B's cap-revoked flag (simulates G14-D
    // durable-backend partial revoke of B's grant only). A's grant
    // remains valid.
    b_revoked.store(true, Ordering::SeqCst);

    // Round 2: emit one event for each pattern. A's cap-recheck still
    // returns true (delivers). B's cap-recheck returns false (auto-
    // cancels — `subscribe.rs` lines 1238-1247: cap_recheck returns
    // false → entry.active = false + unregister).
    let evt_posts_2 = mk_event("Posts:after-revoke");
    let evt_admin_2 = mk_event("Admin:after-revoke");
    publish_change_event_with_labels(&["Posts:after-revoke".to_string()], evt_posts_2);
    publish_change_event_with_labels(&["Admin:after-revoke".to_string()], evt_admin_2);

    // A delivers normally — partial revoke isolates correctly per F6
    // / Compromise #2 D5.
    assert_eq!(
        a_calls.load(Ordering::SeqCst),
        2,
        "A subscription MUST continue delivering after B's partial revoke (F6 isolation)"
    );
    // B's count stays at 1 — second event was rejected at delivery
    // time by the cap-recheck closure.
    assert_eq!(
        b_calls.load(Ordering::SeqCst),
        1,
        "B subscription MUST NOT receive event after partial revoke (cap-recheck false → auto-cancel)"
    );
    // B's active flag is observably flipped to false post-cancel.
    assert!(
        !active_b.load(Ordering::SeqCst),
        "B subscription's active flag MUST be flipped to false after cap-recheck rejection"
    );
    // A's active flag is unchanged.
    assert!(
        active_a.load(Ordering::SeqCst),
        "A subscription's active flag MUST remain true (unrelated path)"
    );

    // Round 3: emit ANOTHER event matching B's pattern — confirm B
    // does NOT recover (auto-cancel is permanent until re-subscribe).
    let evt_admin_3 = mk_event("Admin:final");
    publish_change_event_with_labels(&["Admin:final".to_string()], evt_admin_3);
    assert_eq!(
        b_calls.load(Ordering::SeqCst),
        1,
        "B subscription MUST NOT recover from auto-cancel"
    );

    // Cleanup.
    unregister_on_change(&id_a);
    unregister_on_change(&id_b);
}

// =====================================================================
// corr-minor-1: UCAN forged-sig pin
// =====================================================================

/// Phase-3 G21-T1 fp-mini-review corr-minor-1 fold-in: forge a UCAN
/// token with a tampered signature → run through the
/// `ucan_validate_chain` typed-CALL op → assert `valid: false`.
///
/// pim-2 §3.6b discipline: drives the production engine-layer
/// `dispatch_typed_call` arm with adversarial input + asserts the
/// observable defense fires (validation rejects the forged sig
/// cleanly with `valid: false` rather than panicking or returning
/// `valid: true`).
#[test]
fn ucan_validate_chain_returns_false_on_tampered_signature_corr_minor_1() {
    use benten_id::keypair::Keypair;
    use benten_id::ucan::Ucan;

    let (_dir, engine) = fresh_engine();

    let issuer_kp = Keypair::generate();
    let audience_kp = Keypair::generate();
    let audience_did = audience_kp.public_key().to_did();

    // Forge a structurally-sound, in-window UCAN, then surgically
    // flip a byte INSIDE the signature so the DAG-CBOR shape stays
    // valid + the chain-walker reaches the signature-verify step
    // (which MUST then reject with valid: false).
    let mut ucan = Ucan::builder()
        .issuer_did(&issuer_kp.public_key().to_did())
        .audience_did(&audience_did)
        .capability("zone:user", "write")
        .not_before(1_000)
        .expiry(2_000_000_000)
        .sign(&issuer_kp);

    // Tamper directly with the signature bytes — guarantees the
    // DAG-CBOR envelope remains structurally valid + the verify
    // step is reached.
    assert!(
        ucan.signature.len() >= 32,
        "Ed25519 signatures are 64 bytes; sanity-check"
    );
    ucan.signature[0] ^= 0xff;

    let bytes = serde_ipld_dagcbor::to_vec(&ucan).expect("Ucan DAG-CBOR encode must succeed");

    let input = map_value(&[
        ("tokens", Value::List(vec![Value::Bytes(bytes)])),
        ("audience", Value::Text(audience_did.as_str().to_string())),
        ("capability", Value::Text("zone:user:write".to_string())),
        ("now", Value::Int(1_500_000)),
    ]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::UcanValidateChain, &input)
        .expect("ucan_validate_chain succeeds on tampered chain (clean negative)");
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(false)),
                "tampered-signature UCAN MUST validate: false (corr-minor-1 pin)"
            );
            // Reason should mention signature/decode/chain — the exact
            // wording is implementation-defined but it should be a
            // diagnostic string.
            match m.get("reason") {
                Some(Value::Text(s)) => assert!(
                    !s.is_empty(),
                    "rejection reason MUST be non-empty for diagnostic routing"
                ),
                _ => panic!("reason field MUST be Text on rejection"),
            }
        }
        _ => panic!("ucan_validate_chain MUST return Map"),
    }
}

// =====================================================================
// corr-minor-2: engine.call full-stack typed-CALL test
// =====================================================================

/// Phase-3 G21-T1 fp-mini-review corr-minor-2 fold-in: drive a
/// typed-CALL op through the engine-layer
/// `Engine::dispatch_typed_call` entry point + assert the typed
/// result Map threads through with the expected schema.
///
/// The eval-side CALL primitive's typed-CALL fork (when a CALL Node
/// carries `target: "engine:typed:..."`) is pinned at
/// `tests/typed_call_engine_dispatch.rs::
/// ed25519_sign_then_verify_round_trip_via_dispatch_typed_call`. This
/// test is the engine-API-direct sibling: confirms a typed-CALL is
/// driven end-to-end without registering a subgraph (the typed-CALL
/// registry is closed; user subgraph composition is the alternative
/// driver pinned by the eval-side test).
#[test]
fn engine_dispatch_typed_call_blake3_full_stack_corr_minor_2() {
    let (_dir, engine) = fresh_engine();

    // Drive blake3_hash via the engine entry point — this exercises
    // the production `impl PrimitiveHost for Engine` arm
    // (`crates/benten-engine/src/primitive_host.rs::dispatch_typed_call`).
    let input = map_value(&[("data", Value::Bytes(b"engine.call full-stack pin".to_vec()))]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::Blake3Hash, &input)
        .expect("blake3_hash MUST succeed");

    let Value::Map(m) = out else {
        panic!("blake3_hash MUST return Value::Map");
    };
    let hash = match m.get("hash") {
        Some(Value::Bytes(b)) => b.clone(),
        _ => panic!("hash field MUST be Value::Bytes"),
    };
    assert_eq!(hash.len(), 32, "BLAKE3 hash MUST be 32 bytes");

    // Reference digest computed directly via the blake3 crate.
    let expected = blake3::hash(b"engine.call full-stack pin");
    assert_eq!(
        hash,
        expected.as_bytes().to_vec(),
        "typed-CALL blake3_hash MUST match the BLAKE3 reference digest \
         end-to-end through Engine::dispatch_typed_call"
    );
}

/// corr-minor-2 sibling: drive the full CALL primitive path (the
/// eval-side fork at `crates/benten-eval/src/primitives/call.rs`)
/// through the engine. A CALL Node staged with `target:
/// "engine:typed:blake3_hash"` MUST dispatch through the typed-CALL
/// fork + return the typed result on the `"ok"` edge.
#[test]
fn call_primitive_typed_call_fork_dispatches_through_engine_corr_minor_2() {
    use benten_eval::primitives::call;

    let (_dir, engine) = fresh_engine();
    let op = OperationNode::new("c0", PrimitiveKind::Call)
        .with_property("target", Value::text("engine:typed:blake3_hash"))
        .with_property(
            "input",
            map_value(&[("data", Value::Bytes(b"corr-minor-2".to_vec()))]),
        );

    let step = call::execute(&op, &engine).expect("typed-CALL via CALL primitive MUST succeed");
    assert_eq!(
        step.edge_label, "ok",
        "successful typed-CALL MUST route on the `ok` edge"
    );
    let Value::Map(m) = step.output else {
        panic!("typed-CALL output MUST be Value::Map");
    };
    let hash = match m.get("hash") {
        Some(Value::Bytes(b)) => b.clone(),
        _ => panic!("hash field MUST be Value::Bytes"),
    };
    let expected = blake3::hash(b"corr-minor-2");
    assert_eq!(
        hash,
        expected.as_bytes().to_vec(),
        "CALL-primitive-fork typed-CALL MUST thread the BLAKE3 result through to the output"
    );
}

// =====================================================================
// audit-6-3: DeviceAttestation handshake integration test scaffold
// =====================================================================

/// Phase-3 audit-6-3 fold-in: pin the engine-side AtriumHandle +
/// handshake-wire DeviceDid surface that the eventual end-to-end
/// integration test will exercise.
///
/// **Status:** the full end-to-end flow (declare DeviceAttestation
/// on peer A → join Atrium → peer B sees attestation in handshake
/// metadata) is owned by **G21-T2** (napi bridge to engine + wire
/// `JsAtrium::declare_device_attestation` into `Engine::open_atrium`).
/// Today, the Rust-engine-side `AtriumHandle` does NOT carry a
/// per-handle `DeviceAttestation` slot — the napi shim stores
/// declarations in-memory only (audit-6-3 root cause).
///
/// This test pins the handshake-wire `device_did` field that IS
/// production-wired today (`crates/benten-sync/src/handshake_wire.rs`),
/// asserting the structural contract per net-blocker-4: a peer's
/// handshake frame MUST carry a `device_did` field. When G21-T2
/// completes, the integration-test extension lands here (per the
/// audit-6-3 disposition + named in `phase-3-backlog.md` follow-on
/// — destination is THIS test file).
#[test]
fn handshake_wire_carries_device_did_audit_6_3_scaffold() {
    use benten_id::keypair::Keypair;
    use benten_sync::peer_id::PeerId;

    let peer_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let peer_id = PeerId::from_bytes(*blake3::hash(b"audit-6-3-peer-id").as_bytes());

    let peer_did = peer_kp.public_key().to_did();
    let device_did = device_kp.public_key().to_did();

    let frame = benten_sync::handshake_wire::HandshakeFrame::builder()
        .peer_did(peer_did.clone())
        .device_did(device_did.clone())
        .peer_id(peer_id)
        .build();

    assert_eq!(
        frame.peer_did, peer_did,
        "handshake-wire frame MUST carry peer_did verbatim"
    );
    assert_eq!(
        frame.device_did, device_did,
        "handshake-wire frame MUST carry device_did verbatim — \
         net-blocker-4 + audit-6-3 carry pin"
    );
}

/// Audit-6-3 follow-on RED-PHASE pin: the full end-to-end flow
/// (declare on peer A → peer B observes via handshake metadata) is
/// blocked on G21-T2 napi wireup of `JsAtrium::declare_device_attestation`
/// into the engine. Pinned `#[ignore]`'d here as the named
/// destination per HARD RULE rule-12 clause (b): destination NAMED +
/// receives the entry NOW (this file, this test). When G21-T2 lands
/// the engine-side declared-attestation slot on `AtriumHandle`, this
/// test un-ignores + drives the cross-peer flow end-to-end.
#[tokio::test]
#[ignore = "phase-3-backlog §7.3.D — declared device attestation flows \
            through handshake to remote peer. G21-T2 napi-UCAN-wireup \
            CLOSED at PR #148 commit 7a6c36a; G16-D wave-6b PR #163 shipped \
            on-the-wire device-DID-attestation envelope. Test body pins \
            specific cross-peer flow contract for declared-attestation \
            propagation through Engine::open_atrium → JsAtrium → handshake; \
            the engine-side slot exists at HEAD but the napi shim's \
            in-memory dedup gap (audit-6-3 root cause) composes with §2.5(f) \
            DX-4 multi-Atrium handle dedup (v1-assessment-window per \
            CLAUDE.md item #15; D1 ratification re-evaluation). Body \
            un-ignore at §2.5(f) DX-4 v1-assessment-window landing per \
            Wave-E rationale-only sweep."]
async fn declared_device_attestation_flows_through_handshake_to_remote_peer() {
    // Test shape (un-ignores when G21-T2 lands engine-side slot):
    //
    //   let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    //   let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    //
    //   let device_kp = Keypair::generate();
    //   let attestation = DeviceAttestation::issue(...);
    //   peer_a.declare_device_attestation(attestation.clone()).await.unwrap();
    //
    //   let peer_b_addr = peer_b.loopback_addr().expect("loopback addr");
    //   let peer_b_clone = peer_b.clone();
    //   let accept_task = tokio::spawn(async move { peer_b_clone.accept_handshake().await });
    //   tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    //   peer_a.handshake(peer_b_addr).await.unwrap();
    //   let observed = accept_task.await.unwrap().unwrap();
    //
    //   assert_eq!(
    //       observed.declared_device_attestation,
    //       Some(attestation),
    //       "peer B's handshake metadata MUST carry peer A's declared attestation"
    //   );
    unimplemented!(
        "G21-T2 wires declare_device_attestation through Engine::open_atrium → handshake; \
         this test un-ignores when the engine-side slot lands per audit-6-3 disposition"
    );
}
