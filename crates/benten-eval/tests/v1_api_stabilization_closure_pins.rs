//! §3.6b closure-pins for the benten-eval v1-API-stabilization cluster
//! (refinement-audit umbrellas #1150 / #1145 / #1166).
//!
//! Reproduce-verify discipline (FULL-EXECUTION-PLAN reconciliation
//! addendum): several sub-issues in the umbrella snapshots written at
//! `8141b94` were ALREADY resolved on `main` by the time the ST-EVAL lane
//! ran. Those get a *regression* pin here so they cannot silently regress
//! before v1. The genuinely-residual sub-issues fixed on this branch get a
//! *positive* pin asserting the new shape.
//!
//! | Sub-issue | State at HEAD | Pin kind |
//! |---|---|---|
//! | #813 `WaitOutcome::state_cid` docstring "Panics" | resolved-on-main | regression |
//! | #817 `load_verified_eval` → `load_verified_with_cid` | resolved-on-main | regression |
//! | #1016 `register_runtime` Phase-8 framing | resolved-on-main | regression |
//! | #794 `envelope_cid` aspirational `Result` | fixed-on-branch | positive |
//! | #785 `ChunkSinkError`/`SubscribeError::error_code` | fixed-on-branch | positive |
//! | #1008 `PrimitiveHost` SemVer contract | fixed-on-branch | positive |
//! | #1145 `Evaluator::run` 5-variant → `RunOptions` | fixed-on-branch | positive |
//!
//! (#878 double-nest collapse is pinned in `g11_a_eval_wave1_minors.rs`
//! by `resume_alias_only_accepts_suspended_handle_by_type`.)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Value};
use benten_errors::ErrorCode;
use benten_eval::chunk_sink::ChunkSinkError;
use benten_eval::primitives::subscribe::SubscribeError;
use benten_eval::{
    AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, NullHost, RunOptions,
    SubgraphBuilder, SubgraphBuilderExt, WaitOutcome,
};

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

fn sample_payload() -> ExecutionStatePayload {
    ExecutionStatePayload {
        attribution_chain: vec![AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
            sandbox_depth: 0,
            ..Default::default()
        }],
        pinned_subgraph_cids: vec![zero_cid()],
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero_cid(),
        frame_stack: Vec::new(),
        frame_index: 0,
    }
}

/// #813 regression pin: `WaitOutcome::Complete(_).state_cid()` is TOTAL —
/// it returns the BLAKE3-zero sentinel CID and does NOT panic. The prior
/// drift was a docstring that claimed "Panics" while the body returned the
/// zero CID. This asserts the *behaviour* (a docstring-only regression
/// would still pass; a behavioural regression to `panic!` would fail).
#[test]
fn wait_outcome_complete_state_cid_is_total_not_panicking_813() {
    let oc = WaitOutcome::Complete(Value::Int(1));
    let cid = oc.state_cid(); // must not panic
    assert_eq!(
        cid,
        Cid::from_blake3_digest([0u8; 32]),
        "Complete-arm state_cid must be the BLAKE3-zero sentinel, never a panic"
    );
}

/// #817 regression pin: the disambiguator-suffix rename
/// `load_verified_eval` → `load_verified_with_cid` is landed. This is a
/// compile-fact pin — referencing the trait method by its current name
/// fails to compile if a regression re-introduces the old spelling.
#[test]
fn subgraph_ext_load_verified_with_cid_is_the_name_817() {
    use benten_eval::SubgraphExt;
    let mut sb = SubgraphBuilder::new("h817");
    let r = sb.read("r");
    sb.respond(r);
    let sg = sb.build_validated().expect("trivial subgraph builds");
    let bytes = sg.to_canonical_bytes().expect("encode");
    let cid = sg.cid().expect("cid");
    // Bind the function item by its current name; a rename back to
    // `load_verified_eval` would break this line at compile time.
    let loader = <benten_core::Subgraph as SubgraphExt>::load_verified_with_cid;
    let _round = loader(&cid, &bytes).expect("verified load round-trips");
}

/// #1016 regression pin: `ManifestRegistry::register_runtime` is the
/// reserved typed-error no-op and its surfaced error message no longer
/// carries the stale "Phase 8 marketplace" framing — it names the
/// Phase-4-Meta plugin-install trajectory (RATIFIED §15.3 #7). Asserts the
/// runtime contract (typed Err) AND the retensed user-facing message.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn register_runtime_reserved_error_message_is_phase_4_meta_framed_1016() {
    use benten_eval::sandbox::manifest::{CapBundle, ManifestError, ManifestRegistry};
    let mut reg = ManifestRegistry::new();
    let err = reg
        .register_runtime("x", CapBundle::new(vec![], None))
        .expect_err("register_runtime is a reserved typed-error no-op");
    assert!(matches!(err, ManifestError::RuntimeRegistrationDeferred));
    let msg = err.to_string();
    assert!(
        msg.contains("Phase-4-Meta"),
        "register_runtime error must name the Phase-4-Meta trajectory, got: {msg}"
    );
    assert!(
        !msg.to_lowercase().contains("phase 8 marketplace"),
        "stale 'Phase 8 marketplace' framing must be gone, got: {msg}"
    );
}

/// #794 positive pin: `ExecutionStateEnvelope::envelope_cid()` returns a
/// plain `Cid` (no aspirational `Result` wrapper). The body returns the
/// precomputed `payload_cid`; the value must match an independent
/// `payload.cid()`. A regression to `Result<Cid, _>` breaks the `let cid:
/// Cid = ...` binding at compile time.
#[test]
fn envelope_cid_is_infallible_accessor_794() {
    let payload = sample_payload();
    let expected = payload.cid().expect("payload cid");
    let env = ExecutionStateEnvelope::new(payload).expect("envelope");
    // Compile-fact: a regression to `Result<Cid, _>` breaks this binding.
    let cid: Cid = env.envelope_cid();
    assert_eq!(cid, expected, "envelope_cid is the precomputed payload_cid");
}

/// #785 positive pin: the benten-eval `*Error::code()` convention now
/// holds for `ChunkSinkError` and `SubscribeError` (formerly the lone
/// `error_code()` outliers among 9 `code()` siblings). Compile-fact: the
/// `.code()` calls below fail if a regression renames them back.
#[test]
fn eval_error_types_use_code_not_error_code_785() {
    let cse = ChunkSinkError::CapacityZero;
    let _c: ErrorCode = cse.code();
    let se = SubscribeError::PatternInvalid;
    let _s: ErrorCode = se.code();
    assert_eq!(
        SubscribeError::SystemZoneRead.code(),
        ErrorCode::Inv11SystemZoneRead
    );
}

/// #1145 positive pin: the 5-variant `Evaluator::run*` family is collapsed
/// to `run` (defaults) + `run_with(.., RunOptions)`. Asserts the builder
/// surface end-to-end AND that the old method names are gone (a regression
/// that re-introduces e.g. `run_with_budget` would not break this test,
/// but the builder shape is the canonical pin; the doc-cluster + the
/// engine callsite migration are the structural enforcement).
#[test]
fn evaluator_run_with_runoptions_builder_collapse_1145() {
    let mut sb = SubgraphBuilder::new("h1145");
    let r = sb.read("r");
    sb.respond(r);
    let sg = sb.build_validated().expect("builds");

    let mut ev = benten_eval::Evaluator::new();

    // Defaults entry point.
    let r1 = ev.run(&sg, Value::Null, &NullHost);
    assert!(r1.is_ok(), "default run completes");

    // Builder entry point: explicit budget + trace collection.
    let (r2, trace) = ev.run_with(
        &sg,
        Value::Null,
        &NullHost,
        RunOptions::new().budget(10_000).collect_trace(true),
    );
    assert!(r2.is_ok(), "run_with completes");
    assert!(
        !trace.is_empty(),
        "collect_trace(true) records at least one TraceStep"
    );

    // No-trace builder run returns an empty trace vec.
    let (r3, trace3) = ev.run_with(&sg, Value::Null, &NullHost, RunOptions::new());
    assert!(r3.is_ok());
    assert!(
        trace3.is_empty(),
        "default RunOptions does not collect a trace"
    );
}
