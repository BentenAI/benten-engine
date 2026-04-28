//! Phase 2b R3-B — Inv-4 sandbox-depth runtime + D20 inheritance unit
//! tests (G7-B).
//!
//! D20-RESOLVED:
//!   - `AttributionFrame.sandbox_depth: u8` — counter on the evaluator
//!     frame (NOT on the SANDBOX executor — per-call instance lifecycle
//!     would discard a Store-resident counter).
//!   - INHERITED across CALL boundaries (NOT reset). Handler A SANDBOXes
//!     → CALLs handler B → SANDBOXes is depth-2, not two depth-1s.
//!   - Default max_nest_depth = 4. Saturates to typed error
//!     E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED on overflow.
//!
//! Pin sources: plan §3 G7-B, D20-RESOLVED, sec-pre-r1-03 (frame
//! threading), wsa-6 suggested fix.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "pending G7-A executor wiring; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30)"]
fn invariant_4_sandbox_runtime_depth_traps() {
    // Plan §3 G7-B — runtime depth check fires for TRANSFORM-computed
    // SANDBOX targets that exceed max_nest_depth at evaluation time.
    //
    // Test:
    //   1. Register a handler whose CALL target is computed at runtime
    //      (TRANSFORM produces the target handler CID); the chain ends
    //      with depth-5 nesting.
    //   2. Invoke the handler from outside SANDBOX context.
    //   3. Assert: evaluation fails at depth 5 with
    //      E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED (or
    //      E_INV_SANDBOX_DEPTH per the plan §3 G7-A error catalog list).
    //   4. The check fires BEFORE wasmtime instantiation of the depth-5
    //      SANDBOX (no cold-start cost paid for a rejected depth).
    todo!("R5 G7-B — runtime TRANSFORM-computed depth chain + depth-5 trap");
}

#[test]
#[ignore = "pending G7-A executor wiring; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30)"]
fn invariant_4_depth_inherited_across_call_boundary() {
    // D20 inherit-not-reset — the actual security claim. Handler A
    // SANDBOXes → CALLs handler B → SANDBOXes is depth-2, NOT two
    // separate depth-1s.
    //
    // wsa-6 suggested fix:
    //   1. Register handler B that contains 1 SANDBOX node.
    //   2. Register handler A that contains: 1 SANDBOX node containing
    //      a CALL to handler B.
    //   3. Invoke handler A from outside SANDBOX context.
    //   4. Assert: B's SANDBOX evaluates with frame.sandbox_depth == 2
    //      (NOT 1; NOT 0).
    //   5. Now extend with one more nesting level: B's CALL target is
    //      handler C which also SANDBOXes; the depth-3 attempt rejects
    //      iff max_nest_depth=2; succeeds iff max_nest_depth=3.
    //
    // White-box helper: a host-fn that reads
    // `current_frame().sandbox_depth` and writes it to the test's
    // capture sink so the assertion can read the actual value seen.
    todo!("R5 G7-B — A→SANDBOX→CALL→B→SANDBOX chain + depth=2 assertion");
}

#[test]
#[ignore = "pending G7-A executor wiring; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30)"]
fn invariant_4_depth_inherited_through_attribution_frame() {
    // D20 white-box — assert the inheritance mechanism. The
    // AttributionFrame propagation pattern from Phase-2a sec-r6r1-01
    // closure carries `sandbox_depth: u8` across CALL boundaries.
    //
    // White-box test:
    //   1. Construct parent frame with sandbox_depth = 1.
    //   2. Push child frame for a CALL primitive (no SANDBOX in this
    //      hop).
    //   3. Assert: child_frame.sandbox_depth == 1 (inherited; CALL
    //      itself does NOT increment — only SANDBOX entry does).
    //   4. Push child frame for a SANDBOX primitive.
    //   5. Assert: this child_frame.sandbox_depth == 2 (incremented by
    //      SANDBOX entry).
    todo!("R5 G7-B — push child frames + assert depth at each boundary");
}
