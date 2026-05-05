// R3-E RED-PHASE pins for G19-C1 WAIT TTL DSL helpers
// (wave-7 parallel; §7.1.4 + r6-napi-2 closure).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-C1 +
// .addl/phase-3/00-implementation-plan.md §3 G19-C1 must-pass column):
//
//   - tests/wait_ttl_dsl_subgraph_builder_round_trip — §7.1.4
//   - tests/engine_resume_with_meta_ergonomic_wrapper — §7.1.4
//   - tests/testing_advance_wait_clock_napi_binding_present — §7.1.4
//
// What G19-C1 establishes (§7.1.4):
//
//   - subgraph(...).waitWithTtl(signal, { ttlMs }) DSL helper —
//     replaces the existing wait({signal, ttlHours}) workaround the
//     R6-napi-2 finding documented; lifts WaitArgs to accept ttlMs.
//   - engine.resumeWithMeta(envelope, signal) — typed-envelope shape
//     replacing the raw-bytes resumeFromBytesUnauthenticated path.
//   - bindings/napi/src/wait.rs::testingAdvanceWaitClock — test helper
//     for cross-process WAIT TTL expiry (R6-FP r6-napi-2 surface).
//
// RED-PHASE discipline:
//
//   These tests assert the post-G19-C1 shape (typed envelope; ttlMs
//   accepted in WaitArgs; testingAdvanceWaitClock present on Engine
//   class). The current state ships none of these surfaces. R5
//   implementer drops .skip and wires real assertions.

import { describe, it, expect } from "vitest";

describe("G19-C1 WAIT TTL TS DSL (§7.1.4)", () => {
  it.skip("RED-PHASE: G19-C1 wave-7 — subgraph(...).waitWithTtl(signal, {ttlMs}) compiles + suspends", async () => {
    // §7.1.4 pin. G19-C1 implementer wires this:
    //
    //   const { Engine, subgraph } = await import("@benten/engine");
    //   const engine = await Engine.open(":memory:");
    //
    //   const sg = subgraph("ttl-test")
    //     .read("post:1")
    //     .waitWithTtl({ signal: "user-confirmation", ttlMs: 60_000 })
    //     .respond("ok");
    //
    //   const result = await engine.callWithSuspension(sg.id, "main", {});
    //   // Suspended-branch envelope MUST carry ttlMs:
    //   if (result.kind !== "suspended") {
    //     throw new Error("expected suspended kind");
    //   }
    //   expect(result.envelope.ttlMs).toBe(60_000);
    //
    // OBSERVABLE consequence: WaitArgs.ttlMs flows through napi to the
    // engine-side WaitMetadata.timeout_ms (D12-RESOLVED engine wiring;
    // closes the TS-side surface gap r6-napi-2 named).
  });

  it.skip("RED-PHASE: G19-C1 wave-7 — engine.resumeWithMeta accepts typed envelope", async () => {
    // §7.1.4 + r6-napi-2 pin. G19-C1 implementer wires this:
    //
    //   const engine = await Engine.open(":memory:");
    //   const sg = subgraph("resume-meta-test")
    //     .waitWithTtl({ signal: "go", ttlMs: 60_000 })
    //     .respond("done");
    //
    //   const suspended = await engine.callWithSuspension(sg.id, "main", {});
    //   if (suspended.kind !== "suspended") throw new Error("kind");
    //
    //   // Typed-envelope shape: NO raw Uint8Array; the envelope is an
    //   // object with named fields the TS layer can introspect.
    //   const result = await engine.resumeWithMeta(suspended.envelope, "go");
    //   expect(result.kind).toBe("complete");
    //
    // OBSERVABLE consequence: resumeWithMeta is the ergonomic typed
    // entry point. Defends against r6-napi-2's "actual is
    // resumeFromBytesUnauthenticated taking raw bytes" gap.
  });

  it.skip("RED-PHASE: G19-C1 wave-7 — engine.testingAdvanceWaitClock napi method present", async () => {
    // §7.1.4 + r6-napi-2 pin. G19-C1 implementer wires this:
    //
    //   const engine = await Engine.open(":memory:");
    //   // Sentinel-presence first (the napi binding exists):
    //   expect(typeof (engine as any).testingAdvanceWaitClock).toBe("function");
    //
    //   // End-to-end pin per §3.6b — drive a real TTL expiry path:
    //   const sg = subgraph("ttl-expiry-test")
    //     .waitWithTtl({ signal: "never-arrives", ttlMs: 60_000 })
    //     .respond("expired");
    //   const suspended = await engine.callWithSuspension(sg.id, "main", {});
    //   if (suspended.kind !== "suspended") throw new Error("kind");
    //
    //   // Advance the clock past the TTL boundary:
    //   await engine.testingAdvanceWaitClock(70_000);
    //
    //   // Resume now triggers the TTL-expired branch (typed error):
    //   await expect(engine.resumeWithMeta(suspended.envelope, "never-arrives"))
    //     .rejects.toMatchObject({ code: "E_WAIT_TIMEOUT" });
    //
    // OBSERVABLE consequence: the TTL expiry path is exercisable from
    // TS tests without real wallclock advance. Defends against the
    // sentinel-presence-only failure mode (binding exists but is dead).
  });

  it.skip("RED-PHASE: G19-C1 wave-7 — resumeWithMeta round-trips cap_snapshot_hash for cross-process resume (stream-r1-6)", async () => {
    // stream-r1-6 cross-pin: G19-C1's resumeWithMeta TS DSL must consume
    // the cap_snapshot_hash semantic established by G14-D wave-5a at
    // Engine::resume_from_bytes. Without this round-trip, G19-C1 ships
    // a TS-DSL-vs-engine cross-process asymmetry (a 25th p/c drift
    // candidate).
    //
    //   const engine = await Engine.open(":memory:");
    //   const sg = subgraph("xprocess").waitWithTtl({ signal: "go", ttlMs: 60_000 }).respond("done");
    //   const suspended = await engine.callWithSuspension(sg.id, "main", {});
    //   if (suspended.kind !== "suspended") throw new Error("kind");
    //
    //   // The envelope MUST expose cap_snapshot_hash so a different
    //   // process / engine instance can verify the cap snapshot at
    //   // resume time:
    //   expect(typeof suspended.envelope.capSnapshotHash).toBe("string");
    //   expect(suspended.envelope.capSnapshotHash.length).toBeGreaterThan(0);
    //
    //   // Round-trip through serialization (simulates cross-process):
    //   const serialized = JSON.stringify(suspended.envelope);
    //   const restored = JSON.parse(serialized);
    //   const result = await engine.resumeWithMeta(restored, "go");
    //   expect(result.kind).toBe("complete");
    //
    // OBSERVABLE consequence: cap_snapshot_hash binds the UCAN proof
    // chain at the WAIT-resume envelope across process boundaries.
    // Closes the engine-side asymmetry (Compromise #10) end-to-end.
  });
});
