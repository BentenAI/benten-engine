//! R6FP-Group-1 (r6-stream-3) regression pin —
//! `Engine::call` against a STREAM-bearing handler surfaces a typed
//! `E_PRIMITIVE_NOT_IMPLEMENTED` error rather than silently
//! succeeding with a no-op.
//!
//! Pre-fix, `eval-side primitives::stream::execute` was a silent
//! no-op: it allocated a sink+source via `make_chunk_sink` and
//! immediately discarded both, returning `Ok(StepResult{edge_label:
//! "ok", output: Null})`. A SubgraphSpec with a STREAM node invoked
//! via `engine.call()` (NOT `engine.call_stream`) executed the STREAM
//! primitive as a no-op — the operator saw a successful "OK" outcome
//! with no chunks emitted, no error, no signal that they used the
//! wrong dispatch path.
//!
//! R6FP-G1 fails LOUDLY: the executor returns
//! `EvalError::PrimitiveNotImplemented(Stream)` with an actionable
//! message naming `engine.call_stream` as the correct route.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, EngineError, PrimitiveSpec, SubgraphSpec};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;
use std::collections::BTreeMap;

#[test]
fn engine_call_returns_typed_error_on_stream_bearing_handler() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open engine");

    // Register a STREAM-bearing handler. The intended dispatch is
    // engine.call_stream; this test drives engine.call instead and
    // asserts the typed error.
    let mut stream_props = BTreeMap::new();
    stream_props.insert("source".into(), Value::text("$input.upTo"));
    let spec = SubgraphSpec::builder()
        .handler_id("stream:wrong-dispatch")
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Stream,
            properties: stream_props,
        })
        .build();
    engine
        .register_subgraph(spec)
        .expect("register stream handler");

    let mut input_props = BTreeMap::new();
    input_props.insert("upTo".into(), Value::Int(5));
    let input = Node::new(Vec::new(), input_props);

    let result = engine.call("stream:wrong-dispatch", "run", input);
    match result {
        Err(EngineError::Other { code, .. }) => assert_eq!(
            code,
            ErrorCode::PrimitiveNotImplemented,
            "engine.call against a STREAM-bearing handler MUST surface \
             E_PRIMITIVE_NOT_IMPLEMENTED naming engine.call_stream as \
             the correct route (R6FP-G1 r6-stream-3: pre-fix the \
             eval-side stream::execute body was a silent no-op + the \
             call returned a successful OK outcome with no chunks)"
        ),
        Err(other) => panic!(
            "expected EngineError::Other(PrimitiveNotImplemented), got \
             different EngineError variant: {other:?}"
        ),
        Ok(outcome) => panic!(
            "engine.call against a STREAM-bearing handler succeeded with \
             outcome {outcome:?} — this is the pre-fix silent-no-op \
             behaviour the R6FP-G1 (r6-stream-3) fix-pass closes"
        ),
    }
}
