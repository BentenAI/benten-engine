// Vitest smoke suite for the napi-rs v3 Engine class bindings (G8-A).
//
// Canonical fixture: `benten_core::testing::canonical_test_node` — labels
// `["Post"]` with properties `{title, published, views, tags}`. The CID
// round-trip test asserts the base32 string the Rust spike committed.

import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it } from "vitest";
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

// Closes r4b-rtc-2: one Vitest smoke test per napi method that had zero
// TS-side coverage. Tests that need a Phase-2 primitive to exercise real
// behavior are `it.skip`'d with a TODO(phase-2-*) reason rather than
// passing vacuously.
describe("napi engine — extended surface", () => {
  let extDir: string;
  let ext: any;

  beforeEach(() => {
    extDir = mkdtempSync(join(tmpdir(), "benten-napi-ext-"));
    ext = new native.Engine(join(extDir, "benten.redb"));
  });

  afterEach(() => {
    rmSync(extDir, { recursive: true, force: true });
  });

  it("grantCapability writes a system:CapabilityGrant Node", () => {
    const cid = ext.grantCapability({
      actor: "alice",
      scope: "store:post:write",
    });
    expect(typeof cid).toBe("string");
    expect(cid.startsWith("b")).toBe(true);
    expect(cid.length).toBeGreaterThanOrEqual(50);
    const node = ext.getNode(cid);
    expect(node).not.toBeNull();
    expect(node.labels).toContain("system:CapabilityGrant");
    expect(node.properties.scope).toBe("store:post:write");
    expect(node.properties.revoked).toBe(false);
  });

  it("revokeCapability writes a CapabilityRevocation record", () => {
    const grantCid = ext.grantCapability({
      actor: "bob",
      scope: "store:post:delete",
    });
    const beforeCount = ext.countNodesWithLabel("system:CapabilityRevocation");
    ext.revokeCapability(grantCid, "bob");
    const afterCount = ext.countNodesWithLabel("system:CapabilityRevocation");
    expect(afterCount).toBe(beforeCount + 1);
    // The original grant Node is untouched by the Phase-1 revocation path
    // (the revocation is a separate record, per engine docs at
    // crates/benten-engine/src/lib.rs#1175). Assert it still resolves.
    const grant = ext.getNode(grantCid);
    expect(grant).not.toBeNull();
    expect(grant.labels).toContain("system:CapabilityGrant");
  });

  it("readView returns a structured ok outcome for a live view id", () => {
    // Phase-1 contract: `Engine::read_view` surfaces an `Outcome` shape
    // with `ok: true` and an (intentionally empty) `list` for a live,
    // non-stale view id. The full materialized-read surface is Phase-2
    // scope per crates/benten-engine/src/lib.rs#1340:
    //   "Healthy view — return empty listing (Phase 1: view's full
    //    read API surface is Phase 2)."
    // This smoke test pins the napi boundary's projection of the
    // Outcome so a Phase-2 switch to real payloads trips the populated-
    // data assertion below, not the shape one.
    ext.createNode(["post"], { title: "p-1", createdAt: 1000 });
    const outcome = ext.readView("content_listing", {});
    expect(typeof outcome).toBe("object");
    expect(outcome.ok).toBe(true);
    expect(Array.isArray(outcome.list)).toBe(true);
    // Phase-1: list is empty. Phase-2 will populate — flip this
    // assertion when that lands.
    expect(outcome.list.length).toBe(0);
  });

  it.skip(
    "readView returns populated list after matching writes [TODO(phase-2-view-read)]",
    () => {
      // Blocked: Phase-1 `read_view` returns an empty list even for
      // live, fresh views (engine/src/lib.rs#1340). The full read API
      // surface — including paginated, sorted, label-filtered payload
      // projection — is Phase-2 scope. Un-skip when the evaluator's
      // view-read path plumbs `View::read` through to the Outcome.
      ext.createNode(["post"], { title: "p-1", createdAt: 1000 });
      ext.createNode(["post"], { title: "p-2", createdAt: 2000 });
      const outcome = ext.readView("content_listing", {});
      expect(outcome.list.length).toBeGreaterThanOrEqual(2);
      const titles = outcome.list.map((n: any) => n.properties?.title);
      expect(titles).toContain("p-1");
      expect(titles).toContain("p-2");
    },
  );

  it("callAs dispatches through a handler with an explicit actor CID", () => {
    const handlerId = ext.registerCrud("post");
    // Any valid CID works as the actor principal under NoAuth — just
    // re-use a Node CID as a stand-in for a Phase-1 actor identifier.
    const actorCid = ext.createNode(["Actor"], { handle: "carol" });
    const outcome = ext.callAs(handlerId, "create", { title: "via-callAs" }, actorCid);
    expect(typeof outcome).toBe("object");
    expect(outcome.ok).toBe(true);
    const createdCid = outcome.createdCid ?? outcome.cid;
    expect(typeof createdCid).toBe("string");
    expect(createdCid.startsWith("b")).toBe(true);
    const fetched = ext.getNode(createdCid);
    expect(fetched).not.toBeNull();
    expect(fetched.properties.title).toBe("via-callAs");
  });

  it("createView registers a view that bumps ivmSubscriberCount", () => {
    const before = ext.ivmSubscriberCount();
    // `content_listing_<label>` is a Phase-1 canonical id family that the
    // engine auto-instantiates as a live ContentListingView.
    const viewCid = ext.createView({ viewId: "content_listing_article" });
    expect(typeof viewCid).toBe("string");
    expect(viewCid.startsWith("b")).toBe(true);
    const after = ext.ivmSubscriberCount();
    expect(after).toBe(before + 1);
    // The definition Node is persisted with label `system:IVMView` and
    // should round-trip via getNode.
    const defNode = ext.getNode(viewCid);
    expect(defNode).not.toBeNull();
    expect(defNode.labels).toContain("system:IVMView");
  });

  it("emitEvent surfaces E_PRIMITIVE_NOT_IMPLEMENTED per G8 fix-pass", () => {
    // Phase-1 contract: standalone EMIT is deferred to Phase 2 (the
    // change-stream fan-out is driven by storage WRITEs today). The
    // binding rejects the call with a typed error rather than silently
    // no-op'ing so callers learn their emit had no visible effect.
    expect(() => ext.emitEvent("user.signed_up", { userId: "u1" })).toThrow(
      /E_PRIMITIVE_NOT_IMPLEMENTED/,
    );
  });

  it("countNodesWithLabel returns the number of Nodes stored under a label", () => {
    expect(ext.countNodesWithLabel("widget")).toBe(0);
    ext.createNode(["widget"], { n: 1 });
    ext.createNode(["widget"], { n: 2 });
    ext.createNode(["widget"], { n: 3 });
    // An unrelated label must not inflate the widget count.
    ext.createNode(["gadget"], { n: 99 });
    expect(ext.countNodesWithLabel("widget")).toBe(3);
    expect(ext.countNodesWithLabel("gadget")).toBe(1);
    expect(ext.countNodesWithLabel("nonexistent-label")).toBe(0);
  });

  it("ivmSubscriberCount reports the default engine's pre-wired view set", () => {
    // The default EngineBuilder wires exactly one ContentListingView
    // (label `post`) — see crates/benten-engine/src/lib.rs#1764.
    // Additional canonical views (capability_grants, event_dispatch,
    // governance_inheritance, version_current) are Phase-2 auto-wire
    // scope per the createView source comment. Assert the Phase-1 shape
    // so a future expansion to 5 flips this test loudly.
    const count = ext.ivmSubscriberCount();
    expect(typeof count).toBe("number");
    expect(count).toBe(1);
  });

  it("openWithPolicy opens with NoAuth and accepts writes", () => {
    const dir2 = mkdtempSync(join(tmpdir(), "benten-napi-policy-"));
    try {
      const e = native.Engine.openWithPolicy(
        join(dir2, "benten.redb"),
        "NoAuth",
      );
      const cid = e.createNode(["post"], { title: "noauth-works" });
      expect(typeof cid).toBe("string");
      expect(cid.startsWith("b")).toBe(true);
      const node = e.getNode(cid);
      expect(node).not.toBeNull();
      expect(node.properties.title).toBe("noauth-works");
    } finally {
      rmSync(dir2, { recursive: true, force: true });
    }
  });

  it("openWithPolicy(Ucan) opens; handler dispatch surfaces the Phase-3 stub error", () => {
    // The UcanBackend stub exists so operators who wire it via a config
    // receive a clean typed error naming Phase 3 rather than silent
    // misbehavior. Opening must succeed. Direct `createNode` does NOT
    // trigger the capability hook in Phase 1 (the hook runs at tx-commit
    // inside the evaluator per crates/benten-engine/src/lib.rs#1959);
    // dispatching through a registered handler is the path that invokes
    // `policy.check_write` and therefore the UCAN stub.
    const dir2 = mkdtempSync(join(tmpdir(), "benten-napi-ucan-"));
    try {
      const e = native.Engine.openWithPolicy(
        join(dir2, "benten.redb"),
        "Ucan",
      );
      const handlerId = e.registerCrud("post");
      expect(typeof handlerId).toBe("string");
      // The call outcome may surface the Phase-3 stub as either a thrown
      // napi error or a non-ok Outcome with an errorCode. Accept both to
      // stay resilient across the evaluator's error-propagation shape.
      let surfaced: string | undefined;
      try {
        const outcome = e.call(handlerId, "create", { title: "via-ucan" });
        surfaced = outcome?.errorCode ?? JSON.stringify(outcome);
        expect(outcome.ok).toBe(false);
      } catch (err: any) {
        surfaced = String(err?.message ?? err);
      }
      expect(surfaced).toMatch(/E_CAP_NOT_IMPLEMENTED|NotImplemented|UCAN|capability/i);
    } finally {
      rmSync(dir2, { recursive: true, force: true });
    }
  });

  it("deleteEdge removes an edge so getEdge returns null", () => {
    const a = ext.createNode(["post"], { title: "src" });
    const b = ext.createNode(["post"], { title: "dst" });
    const edgeCid = ext.createEdge(a, b, "RELATED_TO");
    expect(ext.getEdge(edgeCid)).not.toBeNull();
    ext.deleteEdge(edgeCid);
    expect(ext.getEdge(edgeCid)).toBeNull();
    // edgesFrom/edgesTo must no longer surface the deleted edge either.
    const out = ext.edgesFrom(a);
    expect(out.some((e: any) => e.target === b && e.label === "RELATED_TO")).toBe(false);
  });
});

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
