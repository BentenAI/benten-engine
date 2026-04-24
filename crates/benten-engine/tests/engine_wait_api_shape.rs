//! R3 unit tests for G3-B / N3: `Engine::call_with_suspension`,
//! `suspend_to_bytes`, `resume_from_bytes_unauthenticated`,
//! `resume_from_bytes_as` public API shape — FROZEN interface.
//!
//! Also the 4-step resume protocol shape pins from §9.1 (protocol steps 1-4).
//!
//! TDD red-phase: none of these methods exist on `Engine` yet. Tests fail to
//! compile until G3-B lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.6.1).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::{Engine, SuspensionOutcome};
use benten_errors::ErrorCode;

fn open_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open");
    (dir, engine)
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn engine_call_with_suspension_returns_suspended_or_complete() {
    let (_d, engine) = open_engine();

    // Must register SOMETHING so we have a handler id; for shape-pin purposes
    // a no-op synthetic handler stub is sufficient. The happy path is the
    // Complete-arm return.
    engine
        .register_subgraph(benten_engine::testing::minimal_respond_handler("noop"))
        .expect("register");

    let out = engine
        .call_with_suspension("noop", "run", Node::empty())
        .expect("call_with_suspension");

    match out {
        SuspensionOutcome::Complete(_outcome) => {
            // pass — Complete is a valid variant.
        }
        SuspensionOutcome::Suspended(_handle) => {
            // pass — Suspended is a valid variant. Happy path for a no-wait
            // handler is Complete, but both variants must compile.
        }
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn engine_call_with_suspension_complete_path_equals_engine_call() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_respond_handler("eq"))
        .expect("register");

    let plain = engine.call("eq", "run", Node::empty()).expect("plain call");
    let via_susp = engine
        .call_with_suspension("eq", "run", Node::empty())
        .expect("suspension call");

    match via_susp {
        SuspensionOutcome::Complete(outcome) => {
            assert_eq!(
                outcome.edge_taken(),
                plain.edge_taken(),
                "Complete variant must match plain engine.call"
            );
        }
        SuspensionOutcome::Suspended(_) => {
            panic!("no-WAIT handler must Complete, not Suspend")
        }
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn suspend_to_bytes_produces_dagcbor_envelope() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("waiter"))
        .expect("register");

    let out = engine
        .call_with_suspension("waiter", "run", Node::empty())
        .expect("call");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("wait handler must suspend"),
    };
    let bytes = engine.suspend_to_bytes(&handle).expect("suspend_to_bytes");

    // The bytes decode as an ExecutionStateEnvelope via serde_ipld_dagcbor.
    let envelope: benten_eval::ExecutionStateEnvelope =
        serde_ipld_dagcbor::from_slice(&bytes).expect("decode DAG-CBOR envelope");
    assert_eq!(envelope.schema_version, 1, "envelope must be version 1");
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn resume_from_bytes_accepts_suspend_to_bytes_output() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("round"))
        .expect("register");

    let out = engine
        .call_with_suspension("round", "run", Node::empty())
        .expect("call");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("wait handler must suspend"),
    };
    let bytes = engine.suspend_to_bytes(&handle).expect("suspend");

    // Round-trip: resume_from_bytes_unauthenticated accepts the exact output
    // of suspend_to_bytes.
    let _resumed = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("ok"))
        .expect("resume_from_bytes_unauthenticated must accept suspend_to_bytes output");
}

// ---- Resume protocol steps (§9.1 steps 1-4) ------------------------------

#[test]
fn resume_recomputes_payload_cid_rejects_tamper() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("tamper"))
        .expect("register");
    let out = engine
        .call_with_suspension("tamper", "run", Node::empty())
        .expect("call");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("must suspend"),
    };
    let mut bytes = engine.suspend_to_bytes(&handle).expect("bytes");

    // Flip one byte to simulate tampering.
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x5A;

    let err = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("x"))
        .expect_err("tamper rejected");
    assert_eq!(
        err.code(),
        ErrorCode::ExecStateTampered,
        "tamper must fire E_EXEC_STATE_TAMPERED"
    );
}

#[test]
fn resume_requires_matching_resumption_principal() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("princ"))
        .expect("register");

    let alice = benten_engine::testing::principal_cid("alice");
    let eve = benten_engine::testing::principal_cid("eve");

    let out = engine
        .call_as_with_suspension("princ", "run", Node::empty(), &alice)
        .expect("call as alice");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("must suspend"),
    };
    let bytes = engine.suspend_to_bytes(&handle).expect("bytes");

    // Eve attempts to resume.
    let err = engine
        .resume_from_bytes_as(&bytes, benten_core::Value::text("x"), &eve)
        .expect_err("principal mismatch rejected");
    assert_eq!(err.code(), ErrorCode::ResumeActorMismatch);
}

#[test]
fn resume_re_verifies_pinned_subgraph_cids() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("drift"))
        .expect("register");
    let out = engine
        .call_with_suspension("drift", "run", Node::empty())
        .expect("call");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("must suspend"),
    };
    let bytes = engine.suspend_to_bytes(&handle).expect("bytes");

    // Re-register the SAME handler_id under different CID between suspend and
    // resume (test hook forces content divergence).
    engine
        .testing_force_reregister_with_different_cid("drift")
        .expect("force drift");

    let err = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("x"))
        .expect_err("pin drift rejected");
    assert_eq!(err.code(), ErrorCode::ResumeSubgraphDrift);
}

/// G11-A Decision 3: `resume_from_bytes_unauthenticated` documents its
/// missing step via its name. The 4-step protocol's step 2 (principal
/// binding) is skipped by design — a caller who holds valid envelope
/// bytes can resume without proving principal identity. A paired test
/// `resume_requires_matching_resumption_principal` above already pins
/// the positive `_as` path; this test pins the negative contract that
/// the `_unauthenticated` surface does NOT enforce step 2.
#[test]
fn resume_from_bytes_unauthenticated_skips_step_2_by_design() {
    let (_d, engine) = open_engine();
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("skip2"))
        .expect("register");

    // Suspend as alice.
    let alice = benten_engine::testing::principal_cid("alice");
    let out = engine
        .call_as_with_suspension("skip2", "run", Node::empty(), &alice)
        .expect("call as alice");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("must suspend"),
    };
    let bytes = engine.suspend_to_bytes(&handle).expect("bytes");

    // Resume via the `_unauthenticated` surface: no principal is supplied,
    // so step 2 is skipped entirely. The envelope's
    // `resumption_principal_cid` is alice but no caller is claiming
    // anything, so there's nothing to compare against.
    let resumed = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("sig"))
        .expect("unauthenticated resume must succeed without a principal");
    assert!(
        resumed.is_ok_edge(),
        "unauthenticated resume of alice's envelope must NOT fire \
         E_RESUME_ACTOR_MISMATCH — step 2 is skipped by design"
    );
}

#[test]
fn resume_re_calls_check_write() {
    // Step 4: a mock policy counts invocations. Resume must fire exactly one
    // check_write before any evaluator step.
    let dir = tempfile::tempdir().expect("tempdir");
    let (policy, counter) = benten_engine::testing::counting_policy();
    let engine = Engine::builder()
        .capability_policy(policy)
        .open(dir.path().join("eng.redb"))
        .expect("open");
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("rep"))
        .expect("register");

    let out = engine
        .call_with_suspension("rep", "run", Node::empty())
        .expect("call");
    let handle = match out {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("must suspend"),
    };
    let bytes = engine.suspend_to_bytes(&handle).expect("bytes");

    let pre = counter.load();
    let _ = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("sig"))
        .expect("resume");
    let post = counter.load();
    assert_eq!(
        post - pre,
        1,
        "resume must re-call check_write exactly once (§9.1 step 4)"
    );
}
