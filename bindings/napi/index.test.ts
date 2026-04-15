// Phase 1 R3 Vitest: TS <-> Rust round-trip end-to-end.
// Closes SPIKE punt #2. Status: FAILING until B2/B3/B4/B8 land.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

// eslint-disable-next-line @typescript-eslint/no-var-requires
const native = require("./index.node");

const CANONICAL_CID = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";
const CANONICAL_LABELS = ["doc"];
const CANONICAL_PROPS = { title: "canonical", order: 42 };

let tmp: string;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-"));
  native.initEngine(join(tmp, "benten.redb"));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("TS <-> Rust round-trip", () => {
  it("ts_rust_cid_roundtrip_matches_fixture", () => {
    const cid = native.createNode(CANONICAL_LABELS, CANONICAL_PROPS);
    expect(cid).toBe(CANONICAL_CID);
    const fetched = native.getNode(cid);
    expect(fetched).not.toBeNull();
    expect(fetched.labels).toEqual(CANONICAL_LABELS);
    expect(fetched.properties).toEqual(CANONICAL_PROPS);
  });

  it("ts_full_crud_cycle_nodes", () => {
    const cid = native.createNode(["post"], { title: "hello" });
    const fetched = native.getNode(cid);
    expect(fetched.properties.title).toBe("hello");
  });

  it("ts_subgraph_register_and_call", () => {
    const handlerId = native.registerCrudHandler("post");
    const outcome = native.callHandler(handlerId, "post:create", { title: "p1" });
    expect(outcome.ok).toBe(true);
    expect(typeof outcome.cid).toBe("string");
  });

  it("ts_trace_contains_per_node_timings", () => {
    const handlerId = native.registerCrudHandler("post");
    const trace = native.traceHandler(handlerId, "post:create", { title: "traced" });
    expect(trace.steps.length).toBeGreaterThan(0);
    for (const step of trace.steps) { expect(step.durationUs).toBeGreaterThan(0); }
  });
});

describe("napi input validation (B8)", () => {
  it("napi_rejects_oversized_value_map", () => {
    const huge: Record<string, unknown> = {};
    for (let i = 0; i < 100000; i++) huge[`k${i}`] = i;
    expect(() => native.createNode(["post"], huge)).toThrow(/E_INPUT_LIMIT/);
  });

  it("napi_rejects_deep_nested_value", () => {
    let nested: unknown = 0;
    for (let i = 0; i < 2000; i++) nested = { n: nested };
    expect(() => native.createNode(["post"], { nested })).toThrow(/E_INPUT_LIMIT/);
  });

  it("napi_rejects_oversized_bytes", () => {
    const big = new Uint8Array(32 * 1024 * 1024);
    expect(() => native.createNode(["blob"], { data: big })).toThrow(/E_INPUT_LIMIT/);
  });

  it("napi_rejects_malformed_cid", () => {
    expect(() => native.getNode("not-a-real-cid")).toThrow();
  });
});
