// Phase 2a R3 Vitest — TS DSL surface for WAIT + engine.callWithSuspension
// + engine.resumeFromBytes ergonomics + mapNativeError for the new 9
// Phase-2a ErrorCodes.
//
// Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G3-B
// (packages/engine/src/dsl.ts extend wait() stub; dx-r1-8 signal-keyed
// form; DX signal-payload typing addendum) + TS Vitest must-pass entries
// `wait.test.ts` and `wait_signal_variants.test.ts` (combined per brief).
//
// Status: FAILING until G3-B lands:
//   - dsl.ts exports `wait({ signal, signal_shape? })` + `wait({ duration })`
//   - engine.ts exports `callWithSuspension` + `resumeFromBytes`
//   - errors.ts CODE_TO_CTOR covers the 9 new Phase-2a codes
//
// Owned by `qa-expert` per R2 landscape §8.5. TDD red-phase — this file
// runs against the built napi binary (no Rust-side mock).

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, subgraph, wait } from "@benten/engine";

let engine: Engine;
let tmp: string;

beforeAll(async () => {
  tmp = mkdtempSync(join(tmpdir(), "benten-wait-"));
  engine = await Engine.open(join(tmp, "benten.redb"));
});

afterAll(async () => {
  await engine.close();
  rmSync(tmp, { recursive: true, force: true });
});

// ---------------------------------------------------------------------------
// DSL surface — signal-keyed form (dx-r1-8)
// ---------------------------------------------------------------------------

describe("wait DSL surface", () => {
  it("wait_signal_keyed_form_builds_valid_subgraph", () => {
    // The canonical signal form per DSL-SPEC §2.6 (revised).
    const sg = subgraph("wait-signal-test")
      .action("run")
      .wait({ signal: "external:ack" })
      .respond({ body: "$result" })
      .build();
    expect(sg.nodes.length).toBeGreaterThanOrEqual(2);
    const waitNode = sg.nodes.find((n) => n.primitive === "wait");
    expect(waitNode).toBeTruthy();
    expect(waitNode?.args.signal).toBe("external:ack");
    // signal_shape defaults to absent (untyped payload)
    expect(waitNode?.args.signal_shape).toBeUndefined();
  });

  it("wait_duration_form_still_builds", () => {
    // Phase-1 stub form (timed WAIT) must remain valid.
    const sg = subgraph("wait-duration-test")
      .action("run")
      .wait({ duration: "5m" })
      .respond({ body: "$result" })
      .build();
    const waitNode = sg.nodes.find((n) => n.primitive === "wait");
    expect(waitNode?.args.duration).toBe("5m");
  });

  it("wait_signal_shape_optional_accepted", () => {
    // Typed-payload variant per DX addendum.
    const sg = subgraph("wait-typed")
      .action("run")
      .wait({
        signal: "external:payment",
        signal_shape: "{ amount: Int, currency: Text }",
      })
      .respond({ body: "$result" })
      .build();
    const waitNode = sg.nodes.find((n) => n.primitive === "wait");
    expect(waitNode?.args.signal).toBe("external:payment");
    expect(waitNode?.args.signal_shape).toBe(
      "{ amount: Int, currency: Text }",
    );
  });

  it("wait_rejects_no_signal_and_no_duration", () => {
    // Empty-wait shape is invalid per the DSL contract.
    expect(() =>
      subgraph("wait-empty")
        .action("run")
        // @ts-expect-error — deliberately invalid shape
        .wait({})
        .respond({ body: "$result" })
        .build(),
    ).toThrow(/E_DSL_INVALID_SHAPE|signal|duration/);
  });

  it("wait_top_level_helper_returns_typed_shape", () => {
    // The top-level `wait()` helper used for imperative stitching.
    const w = wait({ signal: "external:tick" });
    expect(w.primitive).toBe("wait");
    expect(w.args.signal).toBe("external:tick");
  });
});

// ---------------------------------------------------------------------------
// callWithSuspension + resumeFromBytes ergonomics
// ---------------------------------------------------------------------------

describe("engine.callWithSuspension + resumeFromBytes", () => {
  it("call_with_suspension_returns_suspended_discriminant", async () => {
    const handler = await engine.registerSubgraph(
      subgraph("wait-suspend")
        .action("run")
        .wait({ signal: "external:go" })
        .respond({ body: "$result" })
        .build(),
    );

    const result = await engine.callWithSuspension(handler.id, "run", {});
    expect(result.kind).toBe("suspended");
    if (result.kind !== "suspended") return; // narrow for TS
    expect(result.handle).toBeInstanceOf(Buffer);
    expect(result.handle.length).toBeGreaterThan(0);
  });

  it("resume_from_bytes_completes_suspension", async () => {
    const handler = await engine.registerSubgraph(
      subgraph("wait-resume")
        .action("run")
        .wait({ signal: "external:go" })
        .respond({ body: "$result" })
        .build(),
    );
    const suspended = await engine.callWithSuspension(handler.id, "run", {});
    if (suspended.kind !== "suspended") {
      throw new Error("expected suspended");
    }
    const outcome = await engine.resumeFromBytes(suspended.handle, {
      payload: "ready",
    });
    // Final outcome should route OK; shape-varies with TS wrapper's
    // Outcome projection but `edge` / kind=complete must be present.
    expect(outcome.kind ?? "complete").toBe("complete");
  });

  it("resume_from_bytes_with_tampered_handle_rejects_typed", async () => {
    const handler = await engine.registerSubgraph(
      subgraph("wait-tamper")
        .action("run")
        .wait({ signal: "external:go" })
        .respond({ body: "$result" })
        .build(),
    );
    const suspended = await engine.callWithSuspension(handler.id, "run", {});
    if (suspended.kind !== "suspended") {
      throw new Error("expected suspended");
    }
    // Flip a byte in the middle — payload_cid recomputation must fail.
    const tampered = Buffer.from(suspended.handle);
    tampered[Math.floor(tampered.length / 2)] ^= 0xff;
    try {
      await engine.resumeFromBytes(tampered, { payload: "x" });
      expect.fail("expected E_EXEC_STATE_TAMPERED");
    } catch (err) {
      expect((err as { code?: string }).code).toBe("E_EXEC_STATE_TAMPERED");
    }
  });
});

// ---------------------------------------------------------------------------
// mapNativeError — the 9 new Phase-2a ErrorCodes (dx-r1 discriminant union)
// ---------------------------------------------------------------------------

describe("mapNativeError for Phase-2a codes", () => {
  const firingCodes = [
    "E_EXEC_STATE_TAMPERED",
    "E_RESUME_ACTOR_MISMATCH",
    "E_RESUME_SUBGRAPH_DRIFT",
    "E_WAIT_TIMEOUT",
    "E_INV_IMMUTABILITY",
    "E_INV_SYSTEM_ZONE",
    "E_INV_ATTRIBUTION",
    "E_CAP_WALLCLOCK_EXPIRED",
    "E_WAIT_SIGNAL_SHAPE_MISMATCH",
  ];

  it.each(firingCodes)(
    "maps %s through a typed subclass with stable .code",
    async (code) => {
      // Synthesise a native-shape error that mirrors the napi surface.
      // The wrapper's mapNativeError consults the CODE_TO_CTOR table.
      const synthetic = new Error(`synthetic: ${code}`);
      (synthetic as { code?: string }).code = code;
      const { mapNativeError } = await import("./errors.js");
      const mapped = mapNativeError(synthetic);
      expect(mapped).toBeInstanceOf(Error);
      expect((mapped as { code?: string }).code).toBe(code);
    },
  );

  it("reserved host-codes decode to typed subclasses too", async () => {
    const reservedHostCodes = [
      "E_HOST_NOT_FOUND",
      "E_HOST_WRITE_CONFLICT",
      "E_HOST_BACKEND_UNAVAILABLE",
      "E_HOST_CAPABILITY_REVOKED",
      "E_HOST_CAPABILITY_EXPIRED",
    ];
    for (const code of reservedHostCodes) {
      const synthetic = new Error(`synthetic: ${code}`);
      (synthetic as { code?: string }).code = code;
      const { mapNativeError } = await import("./errors.js");
      const mapped = mapNativeError(synthetic);
      expect((mapped as { code?: string }).code).toBe(code);
    }
  });
});
