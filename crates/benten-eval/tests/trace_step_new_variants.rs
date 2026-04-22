//! R3 unit tests for G3-A / G4-A / dx-r1 (§9.12): new `TraceStep` variants —
//! FROZEN shape.
//!
//! Three new variants land in Phase 2a:
//! - `TraceStep::SuspendBoundary { state_cid: Cid }`
//! - `TraceStep::ResumeBoundary { state_cid: Cid, signal_value: Value }`
//! - `TraceStep::BudgetExhausted { budget_type, consumed, limit, path }` (§9.12)
//!
//! TDD red-phase: the `TraceStep` type in `benten-eval` does not yet carry these
//! variants. Tests fail to compile until G3-A / G4-A land.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.2 + §9.12).

#![allow(clippy::unwrap_used)]

use benten_core::{Cid, Value};
use benten_eval::TraceStep;

fn zero_cid() -> Cid {
    // R5 G3-A note: `from_bytes` on an all-zero buffer fails CID-header
    // validation; the zero-digest CID is the intended fixture.
    Cid::from_blake3_digest([0u8; 32])
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn trace_step_suspend_boundary_variant_present() {
    let step = TraceStep::SuspendBoundary {
        state_cid: zero_cid(),
    };
    match step {
        TraceStep::SuspendBoundary { state_cid } => {
            assert_eq!(state_cid, zero_cid());
        }
        _ => panic!("expected SuspendBoundary variant"),
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn trace_step_resume_boundary_variant_present() {
    let step = TraceStep::ResumeBoundary {
        state_cid: zero_cid(),
        signal_value: Value::text("hello"),
    };
    match step {
        TraceStep::ResumeBoundary {
            state_cid,
            signal_value,
        } => {
            assert_eq!(state_cid, zero_cid());
            match signal_value {
                Value::Text(s) => assert_eq!(s.as_str(), "hello"),
                other => panic!("expected Text signal, got {other:?}"),
            }
        }
        _ => panic!("expected ResumeBoundary variant"),
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn trace_step_budget_exhausted_variant_present() {
    // §9.12 shape: `{ budget_type: &'static str, consumed: u64, limit: u64,
    // path: Vec<NodeId> }`. The shared shape fires from Inv-8 and Phase-2b
    // SANDBOX fuel; Phase-2a only pins the layout.
    let step = TraceStep::BudgetExhausted {
        budget_type: "inv_8_iteration",
        consumed: 42,
        limit: 40,
        path: vec!["iter_0".to_string(), "iter_1".to_string()],
    };
    match step {
        TraceStep::BudgetExhausted {
            budget_type,
            consumed,
            limit,
            path,
        } => {
            assert_eq!(budget_type, "inv_8_iteration");
            assert_eq!(consumed, 42);
            assert_eq!(limit, 40);
            assert_eq!(path, vec!["iter_0".to_string(), "iter_1".to_string()]);
        }
        _ => panic!("expected BudgetExhausted variant"),
    }
}
