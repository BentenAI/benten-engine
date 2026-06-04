// Pre-R4b orchestrator-direct batch item #1: 5 historical `it.skip(...)`
// pins in this file (originally rationaled "Phase 2b G12-E pending" and
// then "Phase 3 (post-G12-E TS DSL ttl_hours)") are SUPERSEDED by the
// equivalent-intent active tests in
// `packages/engine/test/wait_ttl_dsl.test.ts` (G19-C1 wave-7 landing).
//
// HISTORY: the original 5 pins assumed a `wait({ ttl_hours })` DSL
// surface + `engine.callToSuspend` + `engine.testingAdvanceWaitClock` +
// `engine.resumeWithMeta(envelope, signal)` shape. G19-C1 (phase-3-
// backlog §7.1.4) landed an EQUIVALENT-INTENT but different-shape
// surface:
//
//   - `subgraph(...).waitWithTtl({ signal, ttlMs })` (NOT
//     `wait({ ttl_hours })`) — see `dsl.ts::waitWithTtl`.
//   - `engine.callWithSuspension(handlerId, op, input)` returning a
//     typed `SuspensionResult` discriminated union (NOT `callToSuspend`
//     returning a bare envelope) — see `engine.ts::callWithSuspension`.
//   - `engine.resumeWithMeta(envelope, signal)` ergonomic wrapper —
//     see `engine.ts::resumeWithMeta`.
//   - `engine.testingAdvanceWaitClock(deltaMs)` napi-bridged helper —
//     see `engine.ts:167` + `bindings/napi/src/wait.rs`.
//
// Equivalent-intent coverage in `wait_ttl_dsl.test.ts`:
//
//   - Old pin (1) "ttl_hours: 24 compiles + suspends"  →
//     `wait_ttl_dsl_subgraph_builder_round_trip` + `waitWithTtl
//     positional overload yields the same shape`.
//   - Old pin (3) "ttl_hours: 0 rejected with E_WAIT_TTL_INVALID" →
//     `waitWithTtl rejects non-positive ttlMs with
//     E_DSL_INVALID_SHAPE` (typed-error code shifted from
//     `E_WAIT_TTL_INVALID` to `E_DSL_INVALID_SHAPE` because the
//     rejection now fires at the DSL boundary rather than at
//     registration; the protective intent — early failure on bad TTL —
//     is identical).
//   - Old pin (5) "resume after expiry throws E_WAIT_TTL_EXPIRED" →
//     `engine_resume_with_meta_ergonomic_wrapper` (TS-side ergonomics
//     pin) + the runtime end-to-end behavioral pin lives at
//     `bindings/napi/tests/wait_clock.rs::testing_advance_wait_clock_napi_binding_present`
//     (Rust side; bypasses napi extern shape).
//
// Old pins (2) "omitted ttl_hours defaults to 24" + (4) "721 rejected;
// 720 accepted" assumed validation rules that the landed surface does
// NOT carry: the DSL has no default-TTL (omitting ttlMs means "no
// deadline") + no upper-bound cap (engine accepts any positive ttlMs).
// Those validation rules were never landed; the original assumption
// was design-time speculation that didn't survive G19-C1's design.
//
// This stub file remains as an archaeological pointer rather than as
// runnable pins; the equivalent-intent runtime arms are exercised in
// `wait_ttl_dsl.test.ts` per HARD RULE rule-12 disposition (b)
// BELONGS-NAMED-NOW (named-NOW destination = `wait_ttl_dsl.test.ts`,
// which exists + has GREEN pins).

import { describe, it, expect } from "vitest";

describe("WAIT ttl_hours TS DSL — superseded by wait_ttl_dsl.test.ts", () => {
  it("equivalent-intent pins live in wait_ttl_dsl.test.ts (G19-C1)", () => {
    // Sentinel pin: keeps this file in the test suite as an
    // archaeological pointer. The active pins for the WAIT TTL TS DSL
    // surface live at `wait_ttl_dsl.test.ts`; see the file header for
    // the per-pin mapping.
    expect(true).toBe(true);
  });
});
