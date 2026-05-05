// R3-E RED-PHASE pins for G20-B atrium examples compile and run
// (wave 8b; plan §3 G20-B + cag-4).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.8 G20-B):
//
//   - tests/atrium_examples_compile_and_run (Vitest)
//
// What G20-B establishes:
//
//   packages/engine/examples/** — NEW Atrium peer-mgmt + sync-trigger +
//   UCAN-grant + DID-resolution example handlers. Each example must
//   compile under the standard build + run end-to-end without errors.
//
// RED-PHASE discipline:
//
//   Examples don't exist yet. R5 implementer authors them + drops .skip.

import { describe, it, expect } from "vitest";

describe("G20-B Atrium examples compile + run", () => {
  it.skip("RED-PHASE: G20-B wave-8b — atrium peer-management example handler compiles + runs", async () => {
    // G20-B pin. Implementer wires this (B-prime DSL per Ben's D1):
    //
    //   const { example } = await import("@benten/engine/examples/atrium-peer-mgmt");
    //   const result = await example.run();
    //   expect(result.ok).toBe(true);
    //   // Example demonstrates: engine.atrium({config}).join() factory +
    //   // handle.listPeers, trustPeer, revokePeer, onPeerJoin, onPeerLeave
    //   // at minimum.
  });

  it.skip("RED-PHASE: G20-B wave-8b — atrium sync-trigger example handler compiles + runs", async () => {
    // G20-B pin. Implementer wires:
    //
    //   const { example } = await import("@benten/engine/examples/atrium-sync-trigger");
    //   const result = await example.run();
    //   expect(result.ok).toBe(true);
    //   // Example demonstrates: subgraph-trigger sync via atrium primitives.
  });

  it.skip("RED-PHASE: G20-B wave-8b — UCAN-grant example handler compiles + runs", async () => {
    // G20-B pin.
    //
    //   const { example } = await import("@benten/engine/examples/ucan-grant");
    //   const result = await example.run();
    //   expect(result.ok).toBe(true);
    //   // Example demonstrates: UCAN delegation + chain-walk + revoke.
  });

  it.skip("RED-PHASE: G20-B wave-8b — DID-resolution example handler compiles + runs", async () => {
    // G20-B pin.
    //
    //   const { example } = await import("@benten/engine/examples/did-resolution");
    //   const result = await example.run();
    //   expect(result.ok).toBe(true);
    //   // Example demonstrates: did:key generation + resolution.
  });

  it.skip("RED-PHASE: G20-B wave-8b — atrium examples compose entirely from existing 12 primitives (cag-4)", async () => {
    // cag-4 architectural pin (companion to Rust-side
    // tests/atriums_no_new_primitives.rs in benten-engine — we pin the
    // same invariant from the TS side here as a redundant-distinct pin).
    //
    //   // Inspect the example handler subgraphs + verify their
    //   // OperationNode kinds are all in the canonical 12-primitive set:
    //   const allowed = new Set([
    //     "READ", "WRITE", "TRANSFORM", "BRANCH", "ITERATE", "WAIT",
    //     "CALL", "RESPOND", "EMIT", "SANDBOX", "SUBSCRIBE", "STREAM",
    //   ]);
    //   for (const example of [atriumPeerMgmt, atriumSyncTrigger, ucanGrant, didResolution]) {
    //     for (const node of example.subgraph.nodes) {
    //       expect(allowed.has(node.kind)).toBe(true);
    //     }
    //   }
    //
    // Composition shape under B-prime: examples may freely call
    // `engine.atrium({config}).join()` etc. — that's a factory/method
    // composition, not a new primitive kind. Per cag-4, no new
    // OperationNode kinds should appear.
    //
    // OBSERVABLE consequence: Atrium DX surface composes via existing
    // primitives. Defends against the failure mode where a new
    // primitive kind sneaks in via "atrium examples need it."
  });
});
