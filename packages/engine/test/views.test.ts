// R3-followup (R4-FP B-1) red-phase — engine.registerUserView (G8-B exclusive).
//
// Tests are RED at landing time; G8-B (TS-side) makes them green.
//
// Surface contract (per dx-r1-2b + plan §3 G8-B + D8-RESOLVED):
//   - engine.registerUserView(spec) -> Promise<UserView>
//     * spec carries id, inputPattern, strategy?, project?
//     * strategy DEFAULTS to 'B' (D8: hand-written = Rust-only;
//       user views always go through Algorithm B).
//     * strategy === 'A' MUST throw a typed error.
//   - The returned UserView exposes:
//     * snapshot() -> AsyncIterable<Row>  (current rows)
//     * onUpdate(cb) -> Subscription      (event-emitter for diffs)
//
// R6-FP r6-arch-2: renamed from engine.createView → engine.registerUserView
// to align with the Engine's register_* lifecycle pattern. The legacy
// engine.createView(spec) overload remains as a one-cycle deprecation
// alias forwarding to registerUserView.
//
// Pin sources: r2-test-landscape.md §7 rows 462-463; r2 §1.7 + §8 D8;
// r4-qa-expert.json qa-r4-04.

import { describe, it, expect } from "vitest";
import {
  Engine,
  buildUserViewHandle,
  resolveUserViewStrategy,
  validateUserViewSpec,
} from "@benten/engine";
import type {
  UserView,
  UserViewSpec,
  ViewDelta,
} from "@benten/engine";

describe("engine.registerUserView", () => {
  it.skip("DISAGREE-WITH-EXPLANATION (HARD RULE clause-c): redundant with GREEN graceful-fallback + ViewDelta-shape pins; phase-3-backlog §7.1.3 CLOSED-AT-G19-C1+G19-C1-fp", async () => {
    // RE-DISPOSITION RATIONALE (pre-v1 Class A un-ignore, 2026-05-10):
    //
    // Original RED-PHASE body asserted end-to-end IVM materialization
    // through `engine.registerUserView` → `registerSubgraph(crud("post"))`
    // → 3x `engine.call(...)` → `for await (row of view.snapshot())`.
    // Body uses `{} as never` placeholder for the subgraph spec (broken
    // type-cast — would not run).
    //
    // phase-3-backlog §7.1.3 (CLOSED-AT-G19-C1 + G19-C1-fp wave-7):
    // (a) `view.snapshot()` AsyncIterable + (b) `view.onUpdate()`
    // AsyncIterableIterator BOTH SHIPPED. The substantive runtime
    // contract is covered at the GREEN pins below in this file:
    //   - "graceful-fallback (no runtime shim) — iterator yields zero
    //     deltas + closes cleanly" (line 293) — drives runtime: null
    //     branch (pre-G19-C1 cdylib path).
    //   - "yields deltas wrapped in ViewDelta { kind: 'change', payload }
    //     shape" (line 367) — drives the buildOnUpdateIterator runtime
    //     dispatch with a stub runtime, asserts the wrapper shape.
    //   - "cancellation via iterator.return() stops polling cleanly"
    //     (line 317) — drives the iterator.return() lifecycle.
    //   - "native-binding fault during drainUpdates closes iterator
    //     cleanly" (line 410) — drives the error-recovery path.
    // Plus Rust-side end-to-end at
    //   `crates/benten-engine/tests/user_view_strategy_b_default.rs` +
    //   the §7.1.3 CLOSURE narrative names `user_view_snapshot_returns_current_materialized_rows`
    //   (Rust integration) as the production-runtime end-to-end pin.
    //
    // The end-to-end skipped scenario adds nothing the GREEN pins
    // don't already cover at finer grain (the runtime shim stub IS the
    // materialization seam; the engine-side Rust integration test IS
    // the materialization runtime). Keeping skipped with
    // DISAGREE-WITH-EXPLANATION; redundant body retained below for
    // retrospective traceability.
    const engine = await Engine.open(":memory:");

    const spec: UserViewSpec = {
      id: "user_posts_by_author",
      inputPattern: { label: "post" },
      // strategy omitted — exercises the 'B' default (D8).
      project: (evt) => ({
        author: evt.attribution.actorCid,
        label: evt.label,
      }),
    };

    const view: UserView = await engine.registerUserView(spec);
    expect(view.id).toBe("user_posts_by_author");

    // Emit synthetic events through the engine's normal write path.
    const post = await engine.registerSubgraph(/* crud("post") */ {} as never);
    for (let i = 0; i < 3; i++) {
      await engine.call(post.id, "post:create", { i });
    }

    // Snapshot is an AsyncIterable of materialized rows.
    const rows: unknown[] = [];
    for await (const row of view.snapshot()) rows.push(row);
    expect(rows).toHaveLength(3);

    await engine.close();
  });

  it("strategy defaults to 'B' for user views (D8) — pure resolver", () => {
    // G8-B pure-resolver test: pin the default-strategy contract at the
    // TS DSL layer without spinning a Rust engine. The engine integration
    // test lives Rust-side in
    // crates/benten-engine/tests/user_view_strategy_b_default.rs. The
    // resolver is the load-bearing surface the napi bridge round-trips.
    // (r6-dx-4 closure: stale `Engine.open(":memory:")` comment removed —
    // the in-memory backend IS wired in builder.rs (`IN_MEMORY_SENTINEL`
    // routing in `open_backend_for_path`) and active in
    // sandbox.test.ts / snapshot_blob_round_trip.test.ts.)
    const spec: UserViewSpec = {
      id: "user_default_strategy",
      inputPattern: { label: "post" },
      // No strategy field — DEFAULTS to 'B' per D8-RESOLVED.
    };
    const resolved = resolveUserViewStrategy(spec);
    expect(resolved).toBe("B");
  });

  it("refuses strategy 'A' with typed error via engine boundary (D8)", async () => {
    // R6 Round-2 r6-r2-napi-2 closure: the prior `.skip` cited
    // Instance 8 (mapNativeError structured-context metadata) as the
    // remaining blocker; Instance 8 IS LANDED at HEAD (engine_err
    // emits the structured JSON envelope per G19-B; mapNativeError
    // parses the JSON-shape carrier — supersedes the pre-G19-B
    // `$$benten-context$$` sentinel suffix). The Rust-side coverage at
    // `crates/benten-engine/tests/user_view_strategy_refusals.rs`
    // pins the typed-error firing; this TS-side test pins the
    // round-trip through the napi boundary using the
    // `E_VIEW_STRATEGY_A_REFUSED` typed subclass.
    const engine = await Engine.open(":memory:");

    const badSpec = {
      id: "user_a_attempt",
      inputPattern: { label: "post" },
      strategy: "A" as const,
    } as unknown as UserViewSpec;

    await expect(engine.registerUserView(badSpec)).rejects.toMatchObject({
      message: expect.stringMatching(
        /E_VIEW_STRATEGY_A_REFUSED|hand-written|Strategy::A/i,
      ),
    });

    await engine.close();
  });

  it("explicit 'B' opt-in matches default behavior — pure resolver", () => {
    // Pure-resolver assertion: resolveUserViewStrategy returns 'B' both
    // for an explicit `strategy: 'B'` and for the default omission.
    // (r6-dx-4 closure: prior "in-memory backend pending" carry was stale.)
    const explicit = resolveUserViewStrategy({
      id: "user_explicit_b",
      inputPattern: { label: "post" },
      strategy: "B",
    });
    const fallback = resolveUserViewStrategy({
      id: "user_fallback_b",
      inputPattern: { label: "post" },
    });
    expect(explicit).toBe(fallback);
    expect(explicit).toBe("B");
  });

  it("refuses strategy 'C' as Phase-3 reserved via engine boundary", async () => {
    // R6 Round-2 r6-r2-napi-2 closure: same as Strategy-'A' refusal
    // above. Instance 8 mapNativeError structured-context metadata
    // round-trip is wired at HEAD; the prior `.skip` rationale is
    // stale.
    const engine = await Engine.open(":memory:");

    const reservedSpec = {
      id: "user_c_attempt",
      inputPattern: { label: "post" },
      strategy: "C" as const,
    } as unknown as UserViewSpec;

    await expect(engine.registerUserView(reservedSpec)).rejects.toMatchObject({
      message: expect.stringMatching(/E_VIEW_STRATEGY_C_RESERVED|Phase 3|Z-set/i),
    });

    await engine.close();
  });

  it("validateUserViewSpec fail-loud rejects canonical-id with mismatched label (r6-ivm-3)", () => {
    // r6-ivm-3 closure: the AlgorithmBView::for_id dispatcher honors
    // `input_pattern_label` only for `content_listing`; the other 4
    // canonical view ids (capability_grants / version_current /
    // event_dispatch / governance_inheritance) have HARDCODED label
    // semantics. Pre-fix-pass, registering with one of those ids + a
    // user-supplied label silently filtered on the hardcoded label.
    // Post-fix-pass: validateUserViewSpec returns a typed error
    // message before the napi boundary so the silent-mismatch
    // foot-gun is closed.

    const versionCurrentBadLabel = validateUserViewSpec({
      id: "version_current",
      inputPattern: { label: "post" },
    });
    expect(versionCurrentBadLabel).toMatch(/version_current/);
    expect(versionCurrentBadLabel).toMatch(/NEXT_VERSION/);
    expect(versionCurrentBadLabel).toMatch(/post/);

    const capGrantsBadLabel = validateUserViewSpec({
      id: "capability_grants",
      inputPattern: { label: "user-grant" },
    });
    expect(capGrantsBadLabel).toMatch(/capability_grants/);
    expect(capGrantsBadLabel).toMatch(/system:CapabilityGrant/);

    // Matching label is accepted.
    const versionCurrentOk = validateUserViewSpec({
      id: "version_current",
      inputPattern: { label: "NEXT_VERSION" },
    });
    expect(versionCurrentOk).toBeNull();

    // content_listing is not in the canonical-hardcoded-label set;
    // any user-supplied label is fine.
    const contentListingAnyLabel = validateUserViewSpec({
      id: "content_listing",
      inputPattern: { label: "post" },
    });
    expect(contentListingAnyLabel).toBeNull();

    // User-defined ids (anything outside the 5 canonical) are not
    // restricted.
    const userDefined = validateUserViewSpec({
      id: "user_custom_view",
      inputPattern: { label: "any-label" },
    });
    expect(userDefined).toBeNull();
  });

  it("validateUserViewSpec rejects malformed spec with typed message", () => {
    // Pure-validator coverage so the napi-bridge pre-validation contract
    // is pinned regardless of backend availability.
    const missingId = validateUserViewSpec({
      // @ts-expect-error — missing required field for negative test
      inputPattern: { label: "post" },
    });
    expect(missingId).toMatch(/id/);

    const missingPattern = validateUserViewSpec({
      // @ts-expect-error — missing required field for negative test
      id: "x",
    });
    expect(missingPattern).toMatch(/inputPattern/);

    const badStrategy = validateUserViewSpec({
      id: "x",
      inputPattern: { label: "post" },
      // @ts-expect-error — bad value for negative test
      strategy: "Z",
    });
    expect(badStrategy).toMatch(/strategy/);

    // Well-formed spec returns null.
    const ok = validateUserViewSpec({
      id: "ok",
      inputPattern: { label: "post" },
    });
    expect(ok).toBeNull();
  });
});

describe("UserView.onUpdate", () => {
  it.skip("DISAGREE-WITH-EXPLANATION (HARD RULE clause-c): redundant with GREEN ViewDelta-shape + cancellation pins; phase-3-backlog §7.1.3 CLOSED-AT-G19-C1+G19-C1-fp", async () => {
    // RE-DISPOSITION RATIONALE (pre-v1 Class A un-ignore, 2026-05-10):
    //
    // Same disposition as the sibling `it.skip` above. The end-to-end
    // engine.call → onUpdate ViewDelta scenario is covered at finer
    // grain by the GREEN pins below (lines 293/317/367/410) which drive
    // the runtime shim's `drainUpdates` path with controllable stubs;
    // the engine-side Rust integration test
    // (`user_view_strategy_b_default.rs`) drives the IVM materialization
    // runtime directly. Body retained below for retrospective trace-
    // ability; phase-3-backlog §7.1.3 marked CLOSED-AT-G19-C1+G19-C1-fp.
    const engine = await Engine.open(":memory:");

    const view = await engine.registerUserView({
      id: "user_onupdate_test",
      inputPattern: { label: "post" },
    });

    const post = await engine.registerSubgraph(/* crud("post") */ {} as never);

    // Drive a write before opening the iterator; the iterator stamps
    // the head cursor at construction so it picks up only events
    // strictly newer than that.
    const collected: ViewDelta[] = [];
    const iter = view.onUpdate();

    // Concurrent producer: write while the consumer is iterating.
    const producer = (async () => {
      await engine.call(post.id, "post:create", { v: 1 });
    })();

    // Bounded consume: read up to 1 delta, then break (which calls
    // iterator.return() implicitly).
    for await (const delta of iter) {
      collected.push(delta);
      if (collected.length >= 1) break;
    }

    await producer;
    expect(collected.length).toBeGreaterThan(0);
    expect(collected[0]?.kind).toBe("change");

    await engine.close();
  });

  it("graceful-fallback (no runtime shim) — iterator yields zero deltas + closes cleanly", async () => {
    // End-to-end test pin (per dispatch-conventions §3.6b pim-2): drives
    // the production entry point `for await (const delta of view.onUpdate())`
    // through `buildUserViewHandle` with `runtime: null` — the path that
    // older napi cdylib builds (pre-G19-C1) take. Asserts the iterator
    // closes cleanly via `done: true` rather than hanging or throwing.
    // Would FAIL if the AsyncIterableIterator's null-runtime branch were
    // silently no-op'd to a never-resolving Promise.
    const view: UserView = buildUserViewHandle(
      {
        id: "user_no_runtime",
        inputPattern: { label: "post" },
      },
      "B",
      null, // runtime shim absent — pre-G19-C1 cdylib path
    );

    const collected: ViewDelta[] = [];
    for await (const delta of view.onUpdate()) {
      collected.push(delta);
    }
    expect(collected).toEqual([]);
  });

  it("cancellation via iterator.return() stops polling cleanly", async () => {
    // End-to-end test pin (per dispatch-conventions §3.6b pim-2): drives
    // the production cancellation path `iterator.return()` against a
    // controllable runtime stub. Asserts:
    //   1. The iterator transitions to `done: true` after `return()`.
    //   2. `drainUpdates` is no longer called after cancellation (the
    //      timer is cleared so no leaked polling occurs).
    // Would FAIL if the `return()` implementation forgot to clear the
    // pending timer or didn't flip the closed flag — a regression here
    // would cause the iterator to keep polling indefinitely.
    let drainCallCount = 0;
    const stubRuntime = {
      snapshotRows: () => null,
      currentChangeOffset: () => 0,
      drainUpdates: () => {
        drainCallCount += 1;
        return { registered: true, events: [], nextOffset: 0 };
      },
    };

    const view: UserView = buildUserViewHandle(
      {
        id: "user_cancel_test",
        inputPattern: { label: "post" },
      },
      "B",
      stubRuntime,
    );

    const iter = view.onUpdate();

    // Wait one poll cycle to confirm polling is active.
    await new Promise((r) => setTimeout(r, 40));
    expect(drainCallCount).toBeGreaterThanOrEqual(1);

    // Cancel — the iterator's return() stops the polling loop.
    const returnResult = await iter.return!();
    expect(returnResult.done).toBe(true);

    // Subsequent next() returns done=true deterministically.
    const next = await iter.next();
    expect(next.done).toBe(true);

    // No further drainUpdates calls after cancellation. Wait long
    // enough for any orphan timer to have fired.
    const drainCountAfterReturn = drainCallCount;
    await new Promise((r) => setTimeout(r, 80));
    expect(drainCallCount).toBe(drainCountAfterReturn);
  });

  it("yields deltas wrapped in ViewDelta { kind: 'change', payload } shape", async () => {
    // End-to-end test pin: drives the production entry point + asserts
    // the runtime-emitted ViewDelta wraps the underlying ChangeEvent
    // payload verbatim under `payload`. Would FAIL if the
    // buildOnUpdateIterator forgot to wrap (e.g. yielded the raw event)
    // or set the wrong discriminator.
    const events = [{ id: 1 }, { id: 2 }, { id: 3 }];
    let drainCalls = 0;
    const stubRuntime = {
      snapshotRows: () => null,
      currentChangeOffset: () => 0,
      drainUpdates: (_viewId: string, _since: number) => {
        drainCalls += 1;
        if (drainCalls === 1) {
          return { registered: true, events, nextOffset: 3 };
        }
        // Subsequent polls return the empty + signal end via
        // `registered: false` to terminate the iterator cleanly.
        return { registered: false, events: [], nextOffset: 3 };
      },
    };

    const view: UserView = buildUserViewHandle(
      {
        id: "user_payload_shape",
        inputPattern: { label: "post" },
      },
      "B",
      stubRuntime,
    );

    const collected: ViewDelta[] = [];
    for await (const delta of view.onUpdate()) {
      collected.push(delta);
    }

    expect(collected).toHaveLength(3);
    for (let i = 0; i < 3; i++) {
      expect(collected[i]?.kind).toBe("change");
      expect(collected[i]?.payload).toEqual(events[i]);
    }
  });

  it("native-binding fault during drainUpdates closes iterator cleanly", async () => {
    // Defends against the regression where a native-binding throw
    // during a backgrounded poll surfaces as an unhandled rejection.
    // The iterator must catch + close cleanly so consumers observe
    // `done: true`.
    const stubRuntime = {
      snapshotRows: () => null,
      currentChangeOffset: () => 0,
      drainUpdates: () => {
        throw new Error("simulated native-binding fault");
      },
    };

    const view: UserView = buildUserViewHandle(
      {
        id: "user_fault_test",
        inputPattern: { label: "post" },
      },
      "B",
      stubRuntime,
    );

    const collected: ViewDelta[] = [];
    for await (const delta of view.onUpdate()) {
      collected.push(delta);
    }
    expect(collected).toEqual([]);
  });
});
