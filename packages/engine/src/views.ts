// Phase-2b G8-B — `engine.createView(spec)` DSL builder for user-defined
// IVM views.
//
// The TS surface is the public face of `Engine::create_user_view` (Rust
// side) plus the napi `engine.createUserView` bridge. Callers see a
// `UserViewSpec` -> `UserView` transformation; the builder shape is
// intentionally narrow in 2b — the wider Phase-3 sync surface (cursors,
// causal vectors, distributed snapshots) lands alongside the iroh port.
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
  UserViewSubscription,
} from "./types.js";

/**
 * Validate a [`UserViewSpec`] before it crosses the napi boundary.
 *
 * Returns `null` when the spec is well-formed; returns a typed error
 * message string when something is wrong. The caller (`Engine.createView`)
 * lifts the message into the appropriate typed error class.
 */
export function validateUserViewSpec(spec: UserViewSpec): string | null {
  if (typeof spec !== "object" || spec === null) {
    return "createView spec: must be an object";
  }
  if (typeof spec.id !== "string" || spec.id.length === 0) {
    return "createView spec.id: required non-empty string";
  }
  if (
    typeof spec.inputPattern !== "object" ||
    spec.inputPattern === null
  ) {
    return "createView spec.inputPattern: required object with `label` or `anchorPrefix`";
  }
  const ip = spec.inputPattern as Partial<{
    label: unknown;
    anchorPrefix: unknown;
  }>;
  const hasLabel = typeof ip.label === "string" && ip.label.length > 0;
  const hasPrefix =
    typeof ip.anchorPrefix === "string" && ip.anchorPrefix.length > 0;
  if (!hasLabel && !hasPrefix) {
    return "createView spec.inputPattern: must carry either `label: string` or `anchorPrefix: string`";
  }
  if (spec.strategy !== undefined) {
    if (
      spec.strategy !== "A" &&
      spec.strategy !== "B" &&
      spec.strategy !== "C"
    ) {
      return `createView spec.strategy: must be 'A' | 'B' | 'C' (got ${JSON.stringify(spec.strategy)})`;
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
 * Construct a [`UserView`] handle from a resolved spec + the napi-side
 * registration result. The runtime materialization paths (`snapshot()`
 * iterator, `onUpdate()` subscription) light up alongside G8-A's
 * Algorithm B landing — pre-G8-A this returns the empty / no-op
 * implementations so app code is forward-compatible today.
 */
export function buildUserViewHandle(
  spec: UserViewSpec,
  resolvedStrategy: Strategy,
): UserView {
  return {
    id: spec.id,
    strategy: resolvedStrategy,
    inputPattern: spec.inputPattern as UserViewInputPattern,
    snapshot(): AsyncIterable<unknown> {
      return emptyAsyncIterable();
    },
    onUpdate(_cb: (diff: unknown) => void): UserViewSubscription {
      return {
        async unsubscribe(): Promise<void> {
          // No-op pre-G8-A; the subscription handle exists for forward
          // compatibility with the post-G8-A diff-streaming surface.
        },
      };
    },
  };
}

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
