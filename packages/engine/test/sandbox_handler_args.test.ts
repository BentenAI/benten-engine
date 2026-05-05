// R3-D RED-PHASE pins — 24th p/c drift acceptance criterion (TS DSL
// camelCase → napi snake_case translation; pim-2 LOAD-BEARING; G17-C
// wave-5b; phase-3-backlog §6.6).
//
// ## What this pins
//
// Phase-3 G17-C ships `packages/engine/src/dsl.ts::translateSandboxArgs`
// (NEW, mirrors PR #76 `translateWaitArgs` precedent). The translator
// converts:
//
//   { wallclockMs: 100, outputLimitBytes: 4096 }   // camelCase, DSL surface
//
// to:
//
//   { wallclock_ms: 100, output_limit: 4096 }  // snake_case, napi argv
//
// before crossing the napi boundary. The Rust eval-side then reads
// snake_case correctly (existing surface — `primitive_host.rs:877`
// reads `op.properties.get("output_limit")`; `dsl-compiler/src/lib.rs:761+765`
// emits `output_limit: 65536`).
//
// ## R4-FP recalibration (r4-r1-wsa-1 BLOCKER)
//
// The TS DSL surface keeps `outputLimitBytes` (camelCase, with `Bytes`
// for type-clarity). The translation MUST drop the `Bytes` suffix to
// produce the canonical eval-side `output_limit` (NOT `output_limit_bytes`).
// At R3-D the translation target was authored as `output_limit_bytes`
// by symmetry with `wallclockMs` → `wallclock_ms`; r4-r1-wsa-1 caught
// this as the 25th p/c drift recurrence shape (eval-side reader silently
// drops the unknown key + OutputOverflow assertion passes by
// default-fallthrough rather than by ceiling enforcement). Recalibrated
// at R4-FP per the canonical eval-side property name verification.
//
// ## §3.6b end-to-end pin shape
//
// Per pim-2 §3.6b: drive the production entry point
// (`engine.callWithSuspension` or DSL builder → engine.run path) with
// a DSL-built SANDBOX node carrying camelCase args, observe the
// snake_case translation OBSERVABLE at the eval-side ceiling
// enforcement. A regression that leaves `wallclockMs` un-translated
// would fail the eval-side wallclock-trip assertion (the eval reads
// `wallclock_ms` and gets a default = much-larger ceiling).
//
// Pairs with `crates/benten-eval/tests/sandbox_handler_args.rs` (Rust
// eval-side observable end of the same round-trip).
//
// ## Pin sources
//
// - r2-test-landscape §2.5 G17-C
//   `sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
// - r2-test-landscape §3.D 24th p/c drift
// - phase-3-backlog §6.6 (24th p/c drift acceptance criterion)
// - PR #76 precedent (`translateWaitArgs`)

import { describe, it, expect } from "vitest";

describe("R3-D 24th p/c drift — sandbox handler-args camelCase→snake_case round-trip", () => {
  it.skip("RED-PHASE: G17-C wave-5b authors translateSandboxArgs DSL helper (24th p/c drift acceptance criterion)", () => {
    // Pin source: r2-test-landscape §2.5 G17-C
    // `sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
    //
    // G17-C implementer (TS-side) wires this:
    //
    //   import { Engine, subgraph } from "@benten/engine";
    //
    //   // Build an engine + register a manifest:
    //   const engine = await Engine.openInMemory(/* config */);
    //   await engine.registerModuleBytes("compute:safe-default", testFixtureBytes());
    //
    //   // DSL composes a SANDBOX node with camelCase per-handler args:
    //   const sg = subgraph("test").sandbox({
    //     manifestName: "compute:safe-default",
    //     wallclockMs: 100,            // <-- camelCase in DSL surface
    //     outputLimitBytes: 4096,      // <-- camelCase in DSL surface
    //   });
    //
    //   await engine.registerSubgraph(sg);
    //
    //   // Drive a long-running guest that exceeds the 100 ms ceiling:
    //   const result = await engine.run({ subgraphName: "test", input: longRunningInput() });
    //
    //   // eval-side wallclock-trip is observable at the DSL surface:
    //   expect(result.error).toBeDefined();
    //   expect(result.error.code).toBe("E_SANDBOX_WALLCLOCK_EXCEEDED");
    //
    // OBSERVABLE consequence: the DSL camelCase wallclockMs setting is
    // OBSERVED at the eval-side guest-trip boundary. A regression that
    // forgets to call translateSandboxArgs (or adds a new arg without
    // updating the translator) silently widens the ceiling to default
    // and fails this expectation.
    expect.fail(
      "G17-C must wire packages/engine/src/dsl.ts::translateSandboxArgs + engine end-to-end DSL→napi→eval round-trip",
    );
  });

  it.skip("RED-PHASE: G17-C wave-5b — outputLimitBytes axis (camelCase DSL) drops `Bytes` to snake_case `output_limit` (canonical eval-side; r4-r1-wsa-1 recalibration)", () => {
    // Pin source: r2-test-landscape §2.5 G17-C
    // `sandbox_per_handler_output_limit_camel_case_dsl_round_trips`
    // (RECALIBRATED at R4-FP per r4-r1-wsa-1 BLOCKER — canonical
    // eval-side property is `output_limit`, NOT `output_limit_bytes`).
    //
    // G17-C implementer (TS-side) wires this:
    //
    //   const sg = subgraph("test").sandbox({
    //     manifestName: "compute:safe-default",
    //     outputLimitBytes: 4096,      // <-- camelCase DSL surface
    //                                  //     (preserves `Bytes` for type-clarity)
    //   });
    //
    //   // After translateSandboxArgs, the napi argv reads:
    //   //   { manifest_name: "compute:safe-default", output_limit: 4096 }
    //   //                                            ^^^^^^^^^^^^
    //   //                                            (drops `Bytes` — canonical eval-side
    //   //                                             per primitive_host.rs:877)
    //
    //   // Guest that emits 8 KB:
    //   const result = await engine.run({ subgraphName: "test", input: emit8KbInput() });
    //
    //   expect(result.error).toBeDefined();
    //   expect(result.error.code).toBe("E_SANDBOX_OUTPUT_OVERFLOW");
    //
    // Distinct observable consequence per pim-2 §3.6b — defends
    // against the failure shape "translator covers wallclockMs but
    // forgets outputLimitBytes (or vice versa) OR translates to a
    // wrong snake_case target like `output_limit_bytes` that eval
    // silently ignores (the 25th p/c drift recurrence shape per
    // r4-r1-wsa-1)."
    expect.fail(
      "G17-C must wire outputLimitBytes → output_limit camelCase round-trip (drops `Bytes`; canonical eval-side per primitive_host.rs:877)",
    );
  });

  it.skip("RED-PHASE: G17-C wave-5b — sandbox.test.ts existing 3 .skip'd tests re-pinned to production-flow shape", () => {
    // Pin source: r2-test-landscape §2.5 G17-C
    // `sandbox_test_skips_re_pinned_to_production_flow_shape`
    //
    // The existing `packages/engine/test/sandbox.test.ts` carries 3
    // `.skip`'d tests pending Phase-3 module-bytes registration via
    // `engine.registerModuleBytes`. G17-C un-skips them — but distinct
    // from a naive un-skip, the bodies must drive the PRODUCTION FLOW
    // (DSL → registerModuleBytes → registerSubgraph → run), not just
    // the surface-shape contract.
    //
    // This pin is a meta-assertion that the un-skip happened + the
    // bodies use the production flow (per pim-2 §3.6b).
    //
    //   const testSrc = readFileSync(
    //     resolve(__dirname, "sandbox.test.ts"),
    //     "utf8",
    //   );
    //   // The previously .skip'd tests now drive registerModuleBytes:
    //   expect(testSrc).toMatch(/registerModuleBytes/);
    //   // And no `.skip(` remains in the file from the prior 3 tests:
    //   const skipCount = (testSrc.match(/\.skip\(/g) ?? []).length;
    //   expect(skipCount).toBeLessThanOrEqual(0);  // implementer pins exact baseline
    //
    // OBSERVABLE consequence: the un-skip lands AND the un-skipped
    // tests drive production flow (not just stub-call assertions).
    expect.fail(
      "G17-C un-skips the 3 sandbox.test.ts skips with bodies that drive registerModuleBytes production flow per pim-2",
    );
  });
});
