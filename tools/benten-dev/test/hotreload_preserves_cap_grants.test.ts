// Phase 2b Wave-8f Vitest — devserver hot-reload preserves cap grants +
// in-flight evaluations complete before new registration applies.
//
// Lifted from R3 red-phase + un-skipped per `.addl/phase-2b/wave-8-brief.md`
// §8f. The Phase-2a equivalent integration tests already pin the property
// Rust-side (see `tools/benten-dev/tests/devserver_*.rs`); this file is
// the JS-surface mirror that drives the property through the napi bridge.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { BentenDevServer } from "@benten/engine-devserver";

let tmp: string;
let projectRoot: string;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-dev-reload-"));
  projectRoot = join(tmp, "project");
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("benten-dev hot reload (engine-routed)", () => {
  it("devserver_preserves_cap_grants_across_reload", async () => {
    const server = new BentenDevServer({ projectRoot });
    await server.start();
    try {
      // Seed a grant BEFORE the first registration.
      await server.grantCapability({
        actor: "alice",
        scope: "store:post:write",
      });

      // Register handler v1.
      await server.registerHandler(
        "h-post",
        "run",
        "handler 'h-post' { read('post') -> respond }",
      );

      // Hot-reload: replace with v2 body. Engine-routed via
      // register_subgraph_replace; must NOT throw + must NOT clear
      // grants.
      await server.replaceHandler(
        "h-post",
        "run",
        "handler 'h-post' { read('post') -> transform({ x: $x }) -> respond }",
      );

      // Grant survives.
      expect(
        await server.grantExists({
          actor: "alice",
          scope: "store:post:write",
        }),
      ).toBe(true);
    } finally {
      await server.stop();
    }
  });

  it("devserver_in_flight_subscribers_observe_reload_event_for_each_replace", async () => {
    // The Rust-side integration test
    // `devserver_inflight_call_completes_against_v1_before_engine_register_subgraph_swaps_to_v2`
    // pins the in-flight property at the Rust layer; the JS-side
    // mirror validates that the napi bridge surfaces a reload event for
    // each replace so a JS-side observer can pin the same property
    // from above (the JS surface does NOT expose the slow_transform
    // gate the Rust harness uses, so the JS test asserts the
    // observability pre-condition rather than the racing in-flight
    // dispatch — that's a Rust-side property).
    const server = new BentenDevServer({ projectRoot });
    await server.start();
    try {
      const sub = server.subscribeToReloadEvents();

      await server.registerHandler(
        "h-flow",
        "run",
        "handler 'h-flow' { read('post') -> respond }",
      );
      await server.replaceHandler(
        "h-flow",
        "run",
        "handler 'h-flow' { read('post') -> transform({ x: $x }) -> respond }",
      );
      await server.replaceHandler(
        "h-flow",
        "run",
        "handler 'h-flow' { read('post') -> transform({ y: $y }) -> respond }",
      );

      const events = sub.drain();
      // 3 publications (v1, v2, v3) — the JS-side mirror of the Rust
      // version-chain bookkeeping.
      const versions = events.map((e) => e.versionTag);
      expect(versions).toContain("v1");
      expect(versions).toContain("v2");
      expect(versions).toContain("v3");

      sub.unsubscribe();
    } finally {
      await server.stop();
    }
  });
});
