// G12-B red-phase: TS Vitest harness expecting JS `BentenDevServer` + napi
// bridge through to the real Rust devserver routing through Engine.
//
// Per plan §3.2 G12-B "files owned": "tools/benten-dev/test/*.test.ts —
// un-ignore the Vitest harness expecting JS `BentenDevServer` + napi bridge."
//
// TDD red-phase: tests are skipped via `it.skip` (the `phase_2b_landed`-equivalent
// gate for TS — Vitest doesn't compile away ignored tests but skip preserves the
// red-phase contract). Lifts to `it(...)` after G12-B JS surface lands.
//
// Owner: R5 G12-B (qa-r4-01 R3-followup).

import { describe, it, expect } from "vitest";

describe("BentenDevServer (G12-B JS surface)", () => {
  it.skip("registers a handler from a DSL source file via the napi bridge", async () => {
    // Drive: import { BentenDevServer } from "@benten/dev"; spawn against
    // a temp DSL file; assert handler is callable via the engine surface.
    throw new Error("R5 G12-B: implement BentenDevServer JS class + napi bridge");
  });

  it.skip("hot-reload preserves cap-grants when routed through Engine.register_subgraph", async () => {
    throw new Error(
      "R5 G12-B: drive cap-grant preservation property through the JS surface",
    );
  });

  it.skip("in-flight evaluations complete against v1 before swap to v2", async () => {
    throw new Error("R5 G12-B: drive in-flight property through the JS surface");
  });

  it.skip("propagates a typed Diagnostic for bad DSL input", async () => {
    // Pin: bad DSL → JS-side surface receives { error_code: 'E_DSL_PARSE_ERROR',
    // line, column, message } — NOT a generic Error.
    throw new Error(
      "R5 G12-B: assert Diagnostic surface includes error_code + line + column",
    );
  });

  it.skip("inspect-state pretty-printer remains available post-routing-refactor", async () => {
    // Phase-2a inspect-state surface (`tools/benten-dev/test/inspect_state_pretty_prints.test.ts`)
    // must keep working after G12-B refactor.
    throw new Error("R5 G12-B: re-validate inspect-state output shape against routed engine");
  });
});
