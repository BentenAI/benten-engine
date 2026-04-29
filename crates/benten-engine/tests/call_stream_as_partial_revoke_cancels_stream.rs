//! R6FP-Group-1 (r6-stream-2) regression pin —
//! `Engine::call_stream_as` cancels the producer when the actor is
//! revoked mid-stream.
//!
//! Pre-fix, `call_stream_as` accepted an `actor: &Cid` arg but
//! `call_stream_inner` shadowed it as `_actor: Option<Cid>` and the
//! value was dropped. The wave-8c-cont docstring claimed cap-recheck
//! would fire mid-stream once the executor wired in, but the executor
//! wired in (wave-8c-stream-infra) without the principal threading.
//! Production handlers using `call_stream_as` got NO per-chunk
//! cap-recheck — the documented contract was a lie.
//!
//! R6FP-G1 threads the actor through to a `CapRecheckProducer` wrapper
//! that consults the engine's revoked-actors set on every `produce()`
//! call. A mid-stream revoke surfaces `ChunkSinkError::ClosedByPeer`
//! to terminate the producer + the bridge winds down (consumer sees
//! EOS). Mirrors the SUBSCRIBE-side `DeliveryCapRecheck` pattern.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, EngineError, PrimitiveSpec, SubgraphSpec};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;
use std::collections::BTreeMap;

fn alice_cid() -> benten_core::Cid {
    benten_core::Cid::from_blake3_digest(*blake3::hash(b"alice").as_bytes())
}

/// R6FP-G1 (r6-stream-2): when an actor is revoked BEFORE
/// `call_stream_as` is invoked, the stream-open call surfaces
/// `E_CAP_REVOKED_MID_EVAL` rather than silently producing chunks.
/// Pre-flight cap-check at the engine surface catches the
/// already-revoked-at-call case.
#[test]
fn call_stream_as_pre_revoked_actor_refused_at_call() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open engine");

    // Build a STREAM-bearing handler. Source = $input.upTo so the
    // producer would emit N chunks if it ran. Pre-revoke ensures the
    // call refuses BEFORE any chunk produces.
    let mut stream_props = BTreeMap::new();
    stream_props.insert("source".into(), Value::text("$input.upTo"));
    stream_props.insert("chunkSize".into(), Value::Int(1));
    let spec = SubgraphSpec::builder()
        .handler_id("stream:revoke-pre")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Stream,
            properties: stream_props,
        })
        .build();
    engine
        .register_subgraph(spec)
        .expect("register stream handler");

    let alice = alice_cid();
    // Revoke BEFORE the call.
    engine.testing_revoke_cap_mid_call(&alice);

    let mut input_props = BTreeMap::new();
    input_props.insert("upTo".into(), Value::Int(100));
    let input = Node::new(Vec::new(), input_props);

    let result = engine.call_stream_as("stream:revoke-pre", "run", input, &alice);
    match result {
        Err(EngineError::Other { code, .. }) => assert_eq!(
            code,
            ErrorCode::CapRevokedMidEval,
            "call_stream_as MUST refuse with E_CAP_REVOKED_MID_EVAL when \
             the actor is already revoked at call time (R6FP-G1 r6-stream-2 \
             pre-flight cap-check)"
        ),
        other => panic!("expected E_CAP_REVOKED_MID_EVAL at call time, got {other:?}"),
    }
}

/// R6FP-G1 (r6-stream-2) BLOCKER pin: a stream opened by an
/// authorised actor cancels mid-stream when the actor is revoked
/// after some chunks have already been produced. The cap-recheck
/// closure consults the revoked-actors set on every produce() call;
/// a revoked actor terminates the producer via ClosedByPeer + the
/// consumer observes EOS.
#[test]
fn call_stream_as_partial_revoke_cancels_stream() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open engine");

    let mut stream_props = BTreeMap::new();
    stream_props.insert("source".into(), Value::text("$input.upTo"));
    stream_props.insert("chunkSize".into(), Value::Int(1));
    let spec = SubgraphSpec::builder()
        .handler_id("stream:revoke-mid")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Stream,
            properties: stream_props,
        })
        .build();
    engine
        .register_subgraph(spec)
        .expect("register stream handler");

    let alice = alice_cid();
    // Source emits 1000 chunks; without revocation, the consumer
    // would drain all 1000.
    let mut input_props = BTreeMap::new();
    input_props.insert("upTo".into(), Value::Int(1_000));
    let input = Node::new(Vec::new(), input_props);

    let mut handle = engine
        .call_stream_as("stream:revoke-mid", "run", input, &alice)
        .expect("authorised actor opens stream");

    // Pull a few chunks to confirm the stream is producing.
    let chunk_a = handle.next_chunk().expect("first chunk should produce");
    assert!(
        chunk_a.is_some(),
        "stream must produce chunks before revocation"
    );

    // Revoke alice mid-stream. The next produce() observes the
    // revocation + surfaces ClosedByPeer to the bridge.
    engine.testing_revoke_cap_mid_call(&alice);

    // Drain remaining chunks. Eventually we hit None (EOS) — well
    // before the 1000th chunk because the producer terminates as
    // soon as the cap-recheck sees the revocation.
    let mut total = 1; // chunk_a already counted
    while let Ok(Some(_)) = handle.next_chunk() {
        total += 1;
        if total >= 1_000 {
            panic!(
                "stream produced all 1000 chunks despite mid-stream revoke — \
                 R6FP-G1 (r6-stream-2) per-chunk cap-recheck NOT firing; \
                 verify CapRecheckProducer wraps the inner producer + \
                 inner_engine.is_actor_active() consults the revoked-actors set"
            );
        }
    }

    assert!(
        total < 1_000,
        "post-revoke, the stream MUST terminate before draining all \
         pre-configured chunks; observed {total} chunks (expected <1000)"
    );
}
