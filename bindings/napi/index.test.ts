// Phase 1 R3 Vitest: TS <-> Rust round-trip end-to-end.
// Closes SPIKE punt #2. Status: FAILING until B2/B3/B4/B8 land.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

// R4 triage (m19): use ESM-friendly createRequire for the compiled native
// artifact. napi-rs emits a `.node` CJS lib; Vitest's default ESM mode
// cannot `import` it directly, but `createRequire` bridges the two worlds
// cleanly without an eslint suppression.
const require = createRequire(import.meta.url);
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

describe("napi CRUD surface (R4 triage M17 — B3 napi layer)", () => {
  it("napi_update_node_roundtrip", () => {
    const cid1 = native.createNode(["post"], { title: "first" });
    const cid2 = native.updateNode(cid1, ["post"], { title: "first-updated" });
    expect(cid2).not.toBe(cid1);
    const fetched = native.getNode(cid2);
    expect(fetched.properties.title).toBe("first-updated");
  });

  it("napi_delete_node_removes_entry", () => {
    const cid = native.createNode(["post"], { title: "to-delete" });
    native.deleteNode(cid);
    expect(native.getNode(cid)).toBeNull();
  });

  it("napi_create_edge_and_read_back", () => {
    const a = native.createNode(["post"], { title: "a" });
    const b = native.createNode(["post"], { title: "b" });
    const edgeCid = native.createEdge(a, b, "RELATED_TO");
    const edge = native.getEdge(edgeCid);
    expect(edge.source).toBe(a);
    expect(edge.target).toBe(b);
    expect(edge.label).toBe("RELATED_TO");
  });

  it("napi_edges_from_returns_outbound", () => {
    const a = native.createNode(["post"], { title: "a" });
    const b = native.createNode(["post"], { title: "b" });
    native.createEdge(a, b, "RELATED_TO");
    const out = native.edgesFrom(a);
    expect(out.length).toBeGreaterThanOrEqual(1);
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
    // R4 triage (M11): B8 declares a bytes_len limit of 1 MiB. Allocate
    // 1.5x that (~1.5 MB) — enough to exceed the limit without the 32 MB
    // CI allocation cost the v1 test incurred.
    const BYTES_LIMIT_MB = 1;
    const big = new Uint8Array(Math.floor(BYTES_LIMIT_MB * 1.5 * 1024 * 1024));
    expect(() => native.createNode(["blob"], { data: big })).toThrow(/E_INPUT_LIMIT/);
  });

  it("napi_rejects_malformed_cid", () => {
    expect(() => native.getNode("not-a-real-cid")).toThrow();
  });
});
