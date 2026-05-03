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
  UserViewSubscription,
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
