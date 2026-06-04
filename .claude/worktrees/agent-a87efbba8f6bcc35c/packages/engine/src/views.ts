// Phase-2b G8-B — `engine.registerUserView(spec)` DSL builder for
// user-defined IVM views (renamed from `createView`/`createUserView` per
// R6-FP r6-arch-2 to align with the Engine's `register_*` lifecycle
// pattern).
//
// The TS surface is the public face of `Engine::register_user_view`
// (Rust side, renamed from `create_user_view` by R6-FP Group 1) plus
// the napi `engine.registerUserView` bridge. Callers see a
// `UserViewSpec` -> `UserView` transformation; the builder shape is
// intentionally narrow in 2b — the wider Phase-3 sync surface (cursors,
// causal vectors, distributed snapshots) lands alongside the iroh port.
//
// One-cycle deprecation alias: `engine.createView(spec)` continues to
// work, forwarding to `registerUserView`. The legacy
// `engine.createView(viewDef)` for the 5 Phase-1 hand-written views is
// unchanged (semantically distinct — instantiates from a registry).
//
// D8-RESOLVED contract:
//   - `strategy` defaults to `'B'` (the user-view default per D8).
//   - `strategy === 'A'` is REFUSED with `E_VIEW_STRATEGY_A_REFUSED`
//     because Strategy A is reserved for the 5 Phase-1 hand-written
//     views (Rust-only — not user-registerable from TS).
//   - `strategy === 'C'` is REFUSED with `E_VIEW_STRATEGY_C_RESERVED`
//     (Phase-3+ Z-set / DBSP cancellation slot).
//
// The refusal paths are pinned in two places — the napi layer and the
// engine layer — so both bypass paths surface a typed error rather than
// silently coercing the strategy.

import type {
  Strategy,
  UserView,
  UserViewInputPattern,
  UserViewSpec,
  ViewDelta,
} from "./types.js";

/**
 * Canonical IVM view ids whose underlying hand-written view has
 * HARDCODED label semantics. A user calling `registerUserView` with
 * one of these ids + a `Label(...)` that disagrees with the hardcoded
 * label will silently get a view filtering on the wrong label
 * (per r6-ivm-3 finding). The fail-loud rejection in
 * [`validateUserViewSpec`] surfaces a typed error before the napi
 * boundary so the silent-mismatch foot-gun is closed.
 *
 * Mirrors the engine-side hardcoded-label dispatch in
 * `crates/benten-ivm/src/algorithm_b.rs::AlgorithmBView::for_id`. The
 * fifth canonical id `content_listing` is intentionally excluded —
 * its dispatch arm DOES honor `definition.input_pattern_label`.
 */
const CANONICAL_HARDCODED_LABEL_VIEW_IDS: ReadonlyMap<string, string> =
  new Map([
    ["capability_grants", "system:CapabilityGrant"],
    ["version_current", "NEXT_VERSION"],
    ["event_dispatch", "system:EventDispatch"],
    ["governance_inheritance", "system:GovernanceInheritance"],
  ]);

/**
 * Validate a [`UserViewSpec`] before it crosses the napi boundary.
 *
 * Returns `null` when the spec is well-formed; returns a typed error
 * message string when something is wrong. The caller
 * (`Engine.registerUserView`) lifts the message into the appropriate
 * typed error class (`EDslInvalidShape` / `E_INV_REGISTRATION`).
 */
export function validateUserViewSpec(spec: UserViewSpec): string | null {
  if (typeof spec !== "object" || spec === null) {
    return "registerUserView spec: must be an object";
  }
  if (typeof spec.id !== "string" || spec.id.length === 0) {
    return "registerUserView spec.id: required non-empty string";
  }
  if (
    typeof spec.inputPattern !== "object" ||
    spec.inputPattern === null
  ) {
    return "registerUserView spec.inputPattern: required object with `label` or `anchorPrefix`";
  }
  const ip = spec.inputPattern as Partial<{
    label: unknown;
    anchorPrefix: unknown;
  }>;
  const hasLabel = typeof ip.label === "string" && ip.label.length > 0;
  const hasPrefix =
    typeof ip.anchorPrefix === "string" && ip.anchorPrefix.length > 0;
  if (!hasLabel && !hasPrefix) {
    return "registerUserView spec.inputPattern: must carry either `label: string` or `anchorPrefix: string`";
  }
  if (spec.strategy !== undefined) {
    if (
      spec.strategy !== "A" &&
      spec.strategy !== "B" &&
      spec.strategy !== "C"
    ) {
      return `registerUserView spec.strategy: must be 'A' | 'B' | 'C' (got ${JSON.stringify(spec.strategy)})`;
    }
  }
  // r6-ivm-3 fail-loud reject: when the spec id matches one of the 4
  // canonical view ids whose hand-written view has hardcoded label
  // semantics + the user-supplied label disagrees, surface a typed
  // error. The engine-side equivalent rejection lives in
  // `crates/benten-engine/src/engine_views.rs::register_user_view`
  // (R6-R3-FP r6-r3-ivm-1 Rust-side closure surfacing
  // `EngineError::ViewLabelMismatch` / catalog `E_VIEW_LABEL_MISMATCH`);
  // this TS-side guard is pre-napi-boundary defence so callers don't
  // have to round-trip through the napi error envelope to learn the
  // spec is malformed. Both surfaces use the same canonical mapping
  // sourced from `benten_ivm::algorithm_b::CANONICAL_HARDCODED_LABELS`.
  if (hasLabel && typeof ip.label === "string") {
    const hardcodedLabel = CANONICAL_HARDCODED_LABEL_VIEW_IDS.get(spec.id);
    if (hardcodedLabel !== undefined && hardcodedLabel !== ip.label) {
      return (
        `registerUserView spec.id "${spec.id}" is reserved for the canonical IVM view with the hardcoded label "${hardcodedLabel}"; ` +
        `cannot register with a different label "${ip.label}". ` +
        `Use a different spec.id (the user-defined fallback honors any label) ` +
        `OR change spec.inputPattern.label to "${hardcodedLabel}".`
      );
    }
  }
  return null;
}

/**
 * Resolve a user-view strategy. Defaults to `'B'` per D8-RESOLVED.
 * Pure function — exposed so tests pin the default behavior without
 * spinning an engine.
 */
export function resolveUserViewStrategy(spec: UserViewSpec): Strategy {
  return spec.strategy ?? "B";
}

/**
 * Internal helper: compose the napi-side JSON shape from a TS-side
 * [`UserViewSpec`]. Strips the `project` callback (functions cannot
 * cross napi); the projection lives TS-side once G8-A's Algorithm B port
 * lands and the engine surface adds a per-event projection callback bridge.
 */
export function userViewSpecToNativeJson(
  spec: UserViewSpec,
): Record<string, unknown> {
  const out: Record<string, unknown> = {
    id: spec.id,
    inputPattern: spec.inputPattern,
  };
  if (spec.strategy !== undefined) {
    out.strategy = spec.strategy;
  }
  return out;
}

/**
 * Phase-3 G19-C1 (§7.1.3) — runtime-materialization shim the
 * [`Engine.registerUserView`] caller threads in so [`buildUserViewHandle`]
 * can light up `view.snapshot()` + `view.onUpdate()` against the live
 * napi cdylib. The shim isolates the napi surface from `views.ts` (a
 * pure module) so unit tests can stub it without spinning a native
 * cdylib.
 *
 * - `snapshotRows(viewId)` — drives the engine-side
 *   `Engine::user_view_snapshot` napi accessor; returns `null` for an
 *   unknown view id and an array of node rows otherwise.
 * - `currentChangeOffset()` — current head cursor of the engine's
 *   ChangeEvent stream; the `view.onUpdate()` async iterator stamps
 *   this as its starting cursor.
 * - `drainUpdates(viewId, sinceOffset)` — drains incremental deltas
 *   the view observed since `sinceOffset`; the iterator records
 *   `next_offset` per call so subsequent steps replay only events
 *   strictly newer than the prior cursor.
 */
export interface UserViewRuntimeShim {
  snapshotRows(viewId: string): unknown[] | null;
  currentChangeOffset(): number;
  drainUpdates(
    viewId: string,
    sinceOffset: number,
  ): { registered: boolean; events: unknown[]; nextOffset: number };
}

/**
 * Construct a [`UserView`] handle from a resolved spec + the napi-side
 * registration result. The runtime materialization paths (`snapshot()`
 * iterator, `onUpdate()` AsyncIterableIterator) consult the threaded
 * [`UserViewRuntimeShim`]; older napi cdylib builds (pre-G19-C1) lack
 * the runtime accessors — the shim's `snapshotRows` returns `null` and
 * `drainUpdates.registered` is `false`, surfacing as no-op iterables
 * so app code is forward-compatible.
 */
export function buildUserViewHandle(
  spec: UserViewSpec,
  resolvedStrategy: Strategy,
  runtime: UserViewRuntimeShim | null = null,
): UserView {
  return {
    id: spec.id,
    strategy: resolvedStrategy,
    inputPattern: spec.inputPattern as UserViewInputPattern,
    snapshot(): AsyncIterable<unknown> {
      if (runtime === null) {
        return emptyAsyncIterable();
      }
      const rows = runtime.snapshotRows(spec.id);
      if (rows === null || rows.length === 0) {
        return emptyAsyncIterable();
      }
      return rowArrayAsyncIterable(rows);
    },
    onUpdate(): AsyncIterableIterator<ViewDelta> {
      return buildOnUpdateIterator(spec.id, runtime);
    },
  };
}

/**
 * Construct the AsyncIterableIterator returned by [`UserView.onUpdate`].
 *
 * The iterator drives the napi `userViewDrainUpdates` accessor at a
 * 25ms polling cadence (per [`POLL_MS`]) and yields each delivered
 * ChangeEvent wrapped in a [`ViewDelta`]. When the runtime shim is
 * unavailable (pre-G19-C1 cdylib) OR the underlying view is unregistered
 * (the engine-side `Ok(None)` contract from `Engine::user_view_drain_updates_since`),
 * the iterator yields zero deltas + closes cleanly via `done: true`.
 *
 * Cancellation contract: the consumer calls `iterator.return()` (which
 * `for-await ... break` invokes implicitly) to stop the polling loop.
 * `return()` flips `closed = true`, resolves any pending `next()` with
 * `{ done: true }`, and clears the pending timer so no further polls
 * occur. Subsequent `next()` calls return `{ done: true }` deterministically.
 *
 * Defends against:
 *   - native-binding faults during a `drainUpdates` call (caught + close
 *     cleanly so the iterator's caller doesn't observe an unhandled
 *     rejection from a backgrounded tick).
 *   - racing `return()` against an in-flight scheduled tick (the tick
 *     re-checks `closed` after re-entering the closure and exits without
 *     scheduling the next poll).
 */
function buildOnUpdateIterator(
  viewId: string,
  runtime: UserViewRuntimeShim | null,
): AsyncIterableIterator<ViewDelta> {
  // Buffer of pre-yielded ChangeEvents drained from the napi accessor.
  // The polling loop fills it; `next()` pulls from it FIFO.
  const buffer: ViewDelta[] = [];
  // Pending `next()` awaiters when the buffer is empty + the loop is
  // still active. Resolved either by the next poll's events or by
  // `return()` + final `next()` to flush done-state.
  type Resolver = (result: IteratorResult<ViewDelta>) => void;
  const pending: Resolver[] = [];

  let closed = runtime === null;
  // Cursor stamped at iterator start so `for await` from registration
  // time sees only events strictly newer than the head at subscription.
  let cursor = runtime !== null ? runtime.currentChangeOffset() : 0;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const drainPending = (result: IteratorResult<ViewDelta>): void => {
    while (pending.length > 0) {
      const resolve = pending.shift()!;
      resolve(result);
    }
  };

  const closeIterator = (): void => {
    if (closed) return;
    closed = true;
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
    drainPending({ value: undefined, done: true });
  };

  const tick = (): void => {
    timer = null;
    if (closed || runtime === null) return;
    let drained: ReturnType<UserViewRuntimeShim["drainUpdates"]>;
    try {
      drained = runtime.drainUpdates(viewId, cursor);
    } catch {
      // Native-binding faults close the iterator cleanly — consumers
      // observe `done: true` rather than an unhandled rejection from a
      // backgrounded scheduled tick.
      closeIterator();
      return;
    }
    if (!drained.registered) {
      // `Engine::user_view_drain_updates_since(viewId)` returned
      // `Ok(None)` — the view is not registered (or has been
      // unregistered). End-of-stream contract.
      closeIterator();
      return;
    }
    cursor = drained.nextOffset;
    for (const ev of drained.events) {
      const delta: ViewDelta = { kind: "change", payload: ev };
      if (pending.length > 0) {
        const resolve = pending.shift()!;
        resolve({ value: delta, done: false });
      } else {
        buffer.push(delta);
      }
    }
    if (!closed) {
      timer = setTimeout(tick, POLL_MS);
    }
  };

  // Kick off the first poll if the runtime shim is available. When
  // runtime is null, `closed` is already true so `next()` returns
  // `{ done: true }` immediately on first call.
  if (runtime !== null) {
    timer = setTimeout(tick, POLL_MS);
  }

  const iterator: AsyncIterableIterator<ViewDelta> = {
    next(): Promise<IteratorResult<ViewDelta>> {
      if (buffer.length > 0) {
        const value = buffer.shift()!;
        return Promise.resolve({ value, done: false });
      }
      if (closed) {
        return Promise.resolve({ value: undefined, done: true });
      }
      return new Promise<IteratorResult<ViewDelta>>((resolve) => {
        pending.push(resolve);
      });
    },
    return(): Promise<IteratorResult<ViewDelta>> {
      closeIterator();
      return Promise.resolve({ value: undefined, done: true });
    },
    [Symbol.asyncIterator](): AsyncIterableIterator<ViewDelta> {
      return iterator;
    },
  };
  return iterator;
}

/**
 * Polling cadence for `view.onUpdate()` async-iterator step. The napi
 * `userViewDrainUpdates` adapter is cheap (atomic-load + filtered Vec
 * iteration), so a 25ms cadence is responsive without burning CPU on
 * an idle subscription. Tunable post-G19-C1 if back-pressure surfaces.
 */
const POLL_MS = 25;

function emptyAsyncIterable(): AsyncIterable<unknown> {
  return {
    [Symbol.asyncIterator](): AsyncIterator<unknown> {
      return {
        next(): Promise<IteratorResult<unknown>> {
          return Promise.resolve({ value: undefined, done: true });
        },
      };
    },
  };
}

function rowArrayAsyncIterable(rows: unknown[]): AsyncIterable<unknown> {
  return {
    [Symbol.asyncIterator](): AsyncIterator<unknown> {
      let idx = 0;
      return {
        next(): Promise<IteratorResult<unknown>> {
          if (idx >= rows.length) {
            return Promise.resolve({ value: undefined, done: true });
          }
          const value = rows[idx];
          idx += 1;
          return Promise.resolve({ value, done: false });
        },
      };
    },
  };
}
