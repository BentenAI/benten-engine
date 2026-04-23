// Phase 2a G3-B (dx-r1-8): TS-side validation of the `wait({ signal })` vs
// `wait({ duration })` variants.
//
// Scope — DSL surface only. This file runs under Vitest in isolation from
// the native binding (imports `@benten/engine` / `@benten/engine-native`
// surfaces that live in the DSL package), exercising the contract that
//
//   - `signal` and `duration` are BOTH permitted (exclusive or combined:
//     signal-with-duration is the "signal with fallback timeout" form)
//   - Neither being set is a fast-fail (`E_DSL_INVALID_SHAPE`) at
//     `.wait()` time rather than at registration
//   - `signal_shape` is optional on the signal form; absent = untyped
//
// The heavy integration gates (engine.callWithSuspension / resumeFromBytes
// round trip) live in `wait.test.ts` and are exercised against the built
// napi binary. This file is pure DSL.

import { describe, expect, it } from "vitest";
import { subgraph, wait } from "./dsl.js";

describe("wait DSL signal vs duration variants (dx-r1-8)", () => {
  it("wait_dsl_signal_keyed_not_for_keyed", () => {
    // The canonical form uses `signal:` (keyword), NOT `for:` or a bare
    // positional. A build that wrote `for: "external:ack"` would be a
    // shape violation and not produce a valid subgraph.
    const sg = subgraph("h")
      .action("run")
      .wait({ signal: "external:ack" })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w).toBeTruthy();
    expect(w?.args.signal).toBe("external:ack");
    expect((w?.args as Record<string, unknown>).for).toBeUndefined();
  });

  it("wait_signal_shape_defaults_untyped", () => {
    // With no signal_shape the node args carry `signal` but NOT
    // signal_shape — the absence is semantically "accept any Value".
    const sg = subgraph("h2")
      .action("run")
      .wait({ signal: "external:tick" })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w?.args.signal_shape).toBeUndefined();
  });

  it("wait_signal_shape_validates_against_schema_when_set", () => {
    // When signal_shape is set, the node args round-trip it unchanged;
    // the engine-side runtime validator is exercised by the Rust
    // integration tests (wait_signal_shape_optional_typing.rs), not
    // here.
    const schema = "{ amount: Int, currency: Text }";
    const sg = subgraph("h3")
      .action("run")
      .wait({ signal: "external:payment", signal_shape: schema })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w?.args.signal_shape).toBe(schema);
  });

  it("wait_duration_variant_preserves_phase_1_shape", () => {
    const sg = subgraph("h4")
      .action("run")
      .wait({ duration: "5m" })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w?.args.duration).toBe("5m");
    expect(w?.args.signal).toBeUndefined();
  });

  it("wait_signal_with_fallback_duration_combines", () => {
    // Combined form — suspend on signal but fire E_WAIT_TIMEOUT if no
    // signal arrives within the duration.
    const sg = subgraph("h5")
      .action("run")
      .wait({ signal: "external:ack", duration: "1h" })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w?.args.signal).toBe("external:ack");
    expect(w?.args.duration).toBe("1h");
  });

  it("wait_empty_args_rejects_at_dsl_build_time", () => {
    expect(() =>
      subgraph("h6")
        .action("run")
        // @ts-expect-error — deliberately invalid
        .wait({})
        .respond({ body: "$result" }),
    ).toThrow(/signal|duration|E_DSL_INVALID_SHAPE/);
  });

  it("top_level_wait_helper_accepts_signal_form", () => {
    const w = wait({ signal: "external:go" });
    expect(w.primitive).toBe("wait");
    expect(
      (w.args as { signal?: string; duration?: string }).signal,
    ).toBe("external:go");
  });
});
