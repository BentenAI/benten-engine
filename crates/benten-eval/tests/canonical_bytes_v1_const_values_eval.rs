//! S-3 closure — frozen const VALUE pins for `benten-eval` (invariant bounds only).
//!
//! Scope note: the sandbox execution-tuning constants (WALLCLOCK_*,
//! MAX_WASM_STACK_DEFAULT, MODULE_CACHE_MAX_ENTRIES, EPOCH_TICK_INTERVAL,
//! FINGERPRINT_COLLAPSE_THRESHOLD) are deliberately NOT pinned — they are
//! liveness/performance knobs with no wire, key or untrusted-decode role, and
//! `benten_eval::sandbox` is `#[cfg(not(target_arch = "wasm32"))]`. Only the
//! invariant bounds that gate evaluation of attacker-influenceable graphs are
//! frozen here.

use benten_eval::evaluator::DEFAULT_ITERATION_BUDGET;
use benten_eval::exec_state::SYNC_HOP_DEPTH_CAP;
use benten_eval::invariants::budget::DEFAULT_INV_8_BUDGET;
use benten_eval::invariants::sandbox_depth::DEFAULT_MAX_SANDBOX_NEST_DEPTH;
use benten_eval::invariants::sandbox_output::DEFAULT_MAX_SANDBOX_OUTPUT_BYTES;
use benten_eval::limits::{
    DEFAULT_MAX_DEPTH, DEFAULT_MAX_EDGES, DEFAULT_MAX_FANOUT, DEFAULT_MAX_NODES,
};
use benten_eval::typed_call::TYPED_CALL_PREFIX;

// ---------------------------------------------------------------------------
// Group 1 — ENGINE-SPEC 4 invariant limits (Inv-2 / 3 / 5 / 6).
//
// WHAT BREAKS: these are the documented default invariant bounds. They gate
// evaluation of subgraphs that may arrive from a peer or a shared plugin.
// Widening them weakens Inv-2/3/5/6 with no baseline diff and no failing test.
// ---------------------------------------------------------------------------

#[test]
fn evaluator_invariant_limits_are_frozen() {
    assert_eq!(
        DEFAULT_MAX_DEPTH, 64,
        "Inv-2 default max operation-subgraph depth"
    );
    assert_eq!(DEFAULT_MAX_FANOUT, 16, "Inv-3 default max fan-out per node");
    assert_eq!(
        DEFAULT_MAX_NODES, 4_096,
        "Inv-5 default max nodes per subgraph"
    );
    assert_eq!(
        DEFAULT_MAX_EDGES, 8_192,
        "Inv-6 default max edges per subgraph"
    );
}

// ---------------------------------------------------------------------------
// Group 2 — budget + depth caps.
//
// WHAT BREAKS: unbounded evaluation / unbounded SANDBOX nesting / unbounded
// sync-hop recursion. Each is a termination guarantee, not a tuning knob.
// ---------------------------------------------------------------------------

#[test]
fn evaluator_budget_and_depth_caps_are_frozen() {
    assert_eq!(
        DEFAULT_ITERATION_BUDGET, 100_000,
        "default ITERATE budget — the bounded-iteration termination guarantee"
    );
    assert_eq!(DEFAULT_INV_8_BUDGET, 500_000, "Inv-8 default budget");
    assert_eq!(
        DEFAULT_MAX_SANDBOX_NEST_DEPTH, 4,
        "SANDBOX nesting cap — widening re-opens nested-sandbox exhaustion"
    );
    assert_eq!(
        SYNC_HOP_DEPTH_CAP, 8,
        "sync-hop recursion cap over peer-driven evaluation"
    );
    assert_eq!(
        DEFAULT_MAX_SANDBOX_OUTPUT_BYTES,
        16 * 1024 * 1024,
        "SANDBOX output cap — bounds guest-controlled output allocation"
    );
    assert_eq!(
        DEFAULT_MAX_SANDBOX_OUTPUT_BYTES, 16_777_216,
        "literal value"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — typed-call dispatch prefix.
//
// WHAT BREAKS: the reserved dispatch namespace. A change could let a
// user-authored target collide with the engine-reserved prefix.
// ---------------------------------------------------------------------------

#[test]
fn typed_call_prefix_is_frozen() {
    assert_eq!(
        TYPED_CALL_PREFIX, "engine:typed:",
        "reserved typed-call dispatch prefix"
    );
}
