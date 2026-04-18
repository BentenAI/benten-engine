// Vitest smoke suite for the napi-rs v3 Engine class bindings (G8-A).
//
// Canonical fixture: `benten_core::testing::canonical_test_node` — labels
// `["Post"]` with properties `{title, published, views, tags}`. The CID
// round-trip test asserts the base32 string the Rust spike committed.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

// napi-rs v3 emits a platform-suffixed `.node` addon plus an `index.js` CJS
// loader. Vitest runs this file as ESM (package.json sets `"type": "module"`)
// so the `require()` statements inside the generated loader throw. We bypass
// the loader and `createRequire` the platform-specific `.node` binary
// directly — every Phase-1 CI lane runs one platform at a time so this is
// equivalent to what `index.js` would dispatch.
const require = createRequire(import.meta.url);
function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  const name = `./benten-napi.${platform}-${arch}.node`;
  return require(name);
}
const native = loadNative();

const CANONICAL_CID = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";
const CANONICAL_LABELS = ["Post"];
const CANONICAL_PROPS = {
  title: "Hello, Benten",
  published: true,
  views: 42,
  tags: ["rust", "graph"],
};

let tmp: string;
let engine: any;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("ts_roundtrip_cid_matches_rust_fixture", () => {
  it("hashes the canonical test Node to the committed CID", () => {
    const cid = engine.createNode(CANONICAL_LABELS, CANONICAL_PROPS);
    expect(cid).toBe(CANONICAL_CID);
    const fetched = engine.getNode(cid);
    expect(fetched).not.toBeNull();
    expect(fetched.labels).toEqual(CANONICAL_LABELS);
    expect(fetched.properties).toEqual(CANONICAL_PROPS);
  });
});

describe("ts_crud_full_cycle", () => {
  it("creates, reads, updates, and deletes a Node", () => {
    const cid1 = engine.createNode(["post"], { title: "first" });
    expect(engine.getNode(cid1).properties.title).toBe("first");

    const cid2 = engine.updateNode(cid1, ["post"], { title: "updated" });
    expect(cid2).not.toBe(cid1);
    expect(engine.getNode(cid2).properties.title).toBe("updated");

    engine.deleteNode(cid2);
    expect(engine.getNode(cid2)).toBeNull();
  });

  it("creates an edge and reads it back via edges_from / edges_to", () => {
    const a = engine.createNode(["post"], { title: "a" });
    const b = engine.createNode(["post"], { title: "b" });
    const edgeCid = engine.createEdge(a, b, "RELATED_TO");
    const edge = engine.getEdge(edgeCid);
    expect(edge.source).toBe(a);
    expect(edge.target).toBe(b);
    expect(edge.label).toBe("RELATED_TO");

    const out = engine.edgesFrom(a);
    expect(out.length).toBeGreaterThanOrEqual(1);
    expect(out[0].target).toBe(b);

    const inbound = engine.edgesTo(b);
    expect(inbound.length).toBeGreaterThanOrEqual(1);
    expect(inbound[0].source).toBe(a);
  });
});

describe("ts_subgraph_register_and_call", () => {
  it("registers a crud handler and dispatches an op through it", () => {
    const handlerId = engine.registerCrud("post");
    expect(typeof handlerId).toBe("string");
    const outcome = engine.call(handlerId, "create", { title: "p1" });
    expect(typeof outcome).toBe("object");
    // The outcome carries either a `createdCid` or a `cid` alias; both forms
    // indicate the CRUD create path ran end-to-end.
    const reportedCid = outcome.createdCid ?? outcome.cid;
    expect(typeof reportedCid).toBe("string");
    // Tighter: CIDv1 multibase base32 starts with `b` and runs ~59 chars
    // for a BLAKE3-256 digest. This catches regressions that return empty
    // strings, UUIDs, or plain hex.
    expect(reportedCid.startsWith("b")).toBe(true);
    expect(reportedCid.length).toBeGreaterThanOrEqual(50);
    // Round-trip: the reported CID must resolve to the node we wrote.
    const fetched = engine.getNode(reportedCid);
    expect(fetched).not.toBeNull();
    expect(fetched.properties.title).toBe("p1");
  });
});

describe("ts_trace_contains_per_node_timings", () => {
  it("returns a trace with per-step durationUs", () => {
    const handlerId = engine.registerCrud("post");
    const trace = engine.trace(handlerId, "create", { title: "traced" });
    expect(Array.isArray(trace.steps)).toBe(true);
    expect(trace.steps.length).toBeGreaterThan(0);
    for (const step of trace.steps) {
      expect(typeof step.nodeCid).toBe("string");
      expect(typeof step.durationUs).toBe("number");
      expect(step.durationUs).toBeGreaterThan(0);
    }
  });
});

// TODO(r6): 10 napi-exported methods have zero Vitest coverage and are
// spot-checked only on the Rust side. Add one smoke test per method
// before R6 quality council:
//   * callAs
//   * countNodesWithLabel
//   * createView
//   * deleteEdge
//   * emitEvent
//   * grantCapability
//   * ivmSubscriberCount
//   * openWithPolicy
//   * readView
//   * revokeCapability
// Priority: grantCapability + revokeCapability + readView (exit-criterion
// #3 / #6 adjacency). See r4b-rust-test-coverage r4b-rtc-2.

describe("napi misc surfaces", () => {
  it("reports a non-negative change_event_count after writes", () => {
    const before = engine.changeEventCount();
    engine.createNode(["post"], { title: "bump" });
    const after = engine.changeEventCount();
    expect(after).toBeGreaterThanOrEqual(before);
  });

  it("renders a registered handler as Mermaid source", () => {
    const handlerId = engine.registerCrud("post");
    const mermaid = engine.handlerToMermaid(handlerId);
    expect(typeof mermaid).toBe("string");
    expect(mermaid).toMatch(/flowchart/);
  });
});
