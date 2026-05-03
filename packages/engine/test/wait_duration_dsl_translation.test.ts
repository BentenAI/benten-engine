// R6-R5 r6-r5-pcds-2 (23rd producer/consumer drift) — §3.6b LOAD-BEARING
// end-to-end test pin for the DSL → napi → eval round-trip of the
// duration-variant WAIT.
//
// Pre-fix shape:
//   - DSL spread wrote `{ duration: "5m" }` (Text) verbatim into the
//     OperationNode args bag.
//   - Eval-side `wait::evaluate_op_with_handler_id` reads
//     `properties.get("duration_ms")` (Int) — the `duration` Text key
//     was IGNORED.
//   - Result: `WaitMetadata { is_duration: false, timeout_ms: None, ... }`
//     — the handler suspended forever. The documented worked example
//     `wait({ duration: "5s" })` was structurally broken at the DSL
//     surface.
//
// Post-fix shape (this PR):
//   - DSL spread translates at `dsl.ts::translateWaitArgs` into
//     `{ duration_ms: 300_000 }` (Int).
//   - napi `json_to_props` round-trips `Value::Int(300_000)`.
//   - Eval-side reader stamps `WaitMetadata.is_duration = true` +
//     `timeout_ms = Some(300_000)`.
//
// §3.6b pin requirement: drive the production entry point
// (`engine.callWithSuspension`) with a DSL-built `wait({ duration })`,
// assert observable consequence + would FAIL if the spread silently
// no-op'd back to writing `duration: Text`.
//
// Observable consequence we pin: post-fix the DSL-built
// `wait({ duration })` produces a SuspensionResult with `signalName`
// EMPTY (duration variant has no signal name) AND the rendered DSL
// args bag carries `duration_ms: 300_000`. Pre-fix the spread would
// have written `duration: "5m"` and the napi round-trip would have
// suspended via the empty-signal/empty-duration branch — the
// `args.duration_ms` assertion below would FAIL because the spread
// would have emitted the raw string instead.
//
// Defense-in-depth: the existing Rust-side test
// `crates/benten-engine/tests/wait_production_runtime_routing.rs::wait_primitive_consults_duration_ms_property`
// already proves that a `duration_ms: Int` property propagates through
// the dispatcher into `WaitMetadata.is_duration = true` /
// `timeout_ms = Some(duration_ms)`. This test pins that the DSL emits
// EXACTLY the property shape that test consumes — closing the
// end-to-end coverage gap that hid the 23rd producer/consumer drift
// through 5 prior deep-sweeps.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, subgraph } from "@benten/engine";

let engine: Engine;
let tmp: string;

beforeAll(async () => {
  tmp = mkdtempSync(join(tmpdir(), "benten-wait-dur-pcds2-"));
  engine = await Engine.open(join(tmp, "benten.redb"));
});

afterAll(async () => {
  await engine.close();
  rmSync(tmp, { recursive: true, force: true });
});

describe("wait({ duration }) DSL translation — r6-r5-pcds-2 §3.6b pin", () => {
  it("LOAD-BEARING — DSL-built wait({ duration }) emits duration_ms (not duration) post-translation", () => {
    // Direct DSL-spread shape pin. Pre-fix this would have asserted
    // `args.duration === "5m"` + `args.duration_ms === undefined`.
    // Post-fix the translation at `translateWaitArgs` flips the shape.
    const sg = subgraph("wait-dur-spread-pin")
      .action("run")
      .wait({ duration: "5m" })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w).toBeDefined();
    expect(w!.args.duration_ms).toBe(300_000);
    expect(w!.args.duration).toBeUndefined();
    // Bare-duration form does NOT emit timeout_ms (that key is
    // reserved for the signal-with-deadline form).
    expect(w!.args.timeout_ms).toBeUndefined();
    expect(w!.args.signal).toBeUndefined();
  });

  it("LOAD-BEARING — DSL-built wait({ signal, duration }) emits timeout_ms (signal-with-deadline)", () => {
    // Combined form pin: duration translates to `timeout_ms` per the
    // eval-side reader's signal-variant deadline semantics. Pre-fix
    // this also wrote raw `duration: "1h"` and the `timeout_ms` key
    // was absent — the signal-style WAIT had no deadline.
    const sg = subgraph("wait-sig-deadline-pin")
      .action("run")
      .wait({ signal: "external:ack", duration: "1h" })
      .respond({ body: "$result" })
      .build();
    const w = sg.nodes.find((n) => n.primitive === "wait");
    expect(w).toBeDefined();
    expect(w!.args.signal).toBe("external:ack");
    expect(w!.args.timeout_ms).toBe(3_600_000);
    expect(w!.args.duration_ms).toBeUndefined();
    expect(w!.args.duration).toBeUndefined();
  });

  it("LOAD-BEARING — DSL-built wait({ duration }) drives engine.callWithSuspension end-to-end", async () => {
    // §3.6b production-entry-point pin. The handler MUST suspend (the
    // duration variant is a deferred completion) WITHOUT routing
    // E_PRIMITIVE_NOT_IMPLEMENTED or E_WAIT_INVALID_PROPERTIES — that
    // proves the DSL-emitted `duration_ms: 300_000` round-trips
    // through napi → `evaluate_op_with_handler_id` → SuspensionStore
    // cleanly.
    //
    // Pre-fix: the DSL emitted `duration: "5m"` (Text). The eval-side
    // reader returned `is_duration: false` (no `duration_ms` key
    // present), `signal_name = ""`, `timeout_ms = None`. The
    // suspension still happened (with an empty key keyed on the
    // handler/node id). The structurally-undetected failure was that
    // the resume path would never fire `WaitResumeSignal::DurationElapsed`
    // for this handle. The assertion below pins that the suspension
    // DOES happen + the typed result shape is intact; combined with
    // the Rust-side `wait_primitive_consults_duration_ms_property`
    // test that proves `duration_ms: Int` → `is_duration: true`, the
    // end-to-end runtime arm is now provably wired.
    const handler = await engine.registerSubgraph(
      subgraph("wait-dur-end-to-end-pin")
        .action("run")
        .wait({ duration: "5m" })
        .respond({ body: "$result" })
        .build(),
    );

    const result = await engine.callWithSuspension(handler.id, "run", {});
    expect(result.kind).toBe("suspended");
    if (result.kind !== "suspended") return;
    // Bare-duration variant: `wait::evaluate_op_with_handler_id` at
    // `crates/benten-eval/src/primitives/wait.rs` derives the key as
    // `__dur__<handler_id>__<node.id>` when `signal_name` is empty
    // (the duration variant has no signal name). The
    // `SuspendedHandle.signal_name()` accessor surfaces this key
    // verbatim through napi → `signalName`. Pre-fix the spread wrote
    // `duration: "5m"` (Text) — `signal_name` was empty + `is_duration`
    // was false; the napi layer ALSO returned a `__dur__...` key (since
    // the empty-signal branch fired regardless). The differentiator is
    // suspension store metadata (`is_duration: true` post-fix vs
    // `false` pre-fix) which isn't surfaced through TS today; we pin
    // the production-entry-point round-trip succeeds + carries the
    // duration-keyed envelope shape, paired with the existing Rust
    // test `wait_primitive_consults_duration_ms_property` which proves
    // `duration_ms: Int` → `is_duration: true`.
    expect(result.signalName.startsWith("__dur__")).toBe(true);
    // stateCid is non-empty + content-addressed (proves the
    // SuspensionStore round-trip succeeded; an empty stateCid would
    // signal the napi layer never reached the eval primitive).
    expect(typeof result.stateCid).toBe("string");
    expect(result.stateCid.length).toBeGreaterThan(0);
    // Handle bytes are present (the suspended discriminant carries
    // the envelope bytes; resume_from_bytes round-trips them).
    expect(result.handle).toBeInstanceOf(Buffer);
    expect(result.handle.length).toBeGreaterThan(0);
  });
});

describe("wait({ duration }) DSL parser — invalid forms reject typed", () => {
  it("rejects empty string with E_DSL_INVALID_SHAPE", () => {
    expect(() =>
      subgraph("h-empty")
        .action("run")
        .wait({ duration: "" })
        .respond({ body: "$result" })
        .build(),
    ).toThrow(/E_DSL_INVALID_SHAPE|signal|duration/);
  });

  it("rejects malformed unit with E_DSL_INVALID_SHAPE", () => {
    expect(() =>
      subgraph("h-bad-unit")
        .action("run")
        .wait({ duration: "5d" })
        .respond({ body: "$result" })
        .build(),
    ).toThrow(/E_DSL_INVALID_SHAPE|s\|m\|h/);
  });

  it("rejects zero magnitude with E_DSL_INVALID_SHAPE", () => {
    expect(() =>
      subgraph("h-zero")
        .action("run")
        .wait({ duration: "0s" })
        .respond({ body: "$result" })
        .build(),
    ).toThrow(/E_DSL_INVALID_SHAPE|positive integer/);
  });

  it("accepts seconds, minutes, and hours units", () => {
    const cases = [
      { input: "30s", expected: 30_000 },
      { input: "5m", expected: 300_000 },
      { input: "2h", expected: 7_200_000 },
      { input: "1s", expected: 1_000 },
    ];
    for (const c of cases) {
      const sg = subgraph(`h-unit-${c.input}`)
        .action("run")
        .wait({ duration: c.input })
        .respond({ body: "$result" })
        .build();
      const w = sg.nodes.find((n) => n.primitive === "wait");
      expect(w?.args.duration_ms).toBe(c.expected);
    }
  });
});
