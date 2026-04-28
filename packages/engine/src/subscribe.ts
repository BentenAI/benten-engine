// SUBSCRIBE DSL surface (Phase 2b G6-B).
//
// This module exposes the TS-side `engine.onChange(pattern, callback)`
// method. The DSL `subgraph(...).subscribe(args)` builder method
// already lives in `dsl.ts`; here we layer the consumer-side
// `Subscription` wrapper that the napi `on_change` adapter returns.
//
// Per plan §3 G6-B (dx-optimizer R1):
//
//   subgraph(...).subscribe(args)               // composition (dsl.ts)
//   engine.onChange(pattern, callback)
//     -> Subscription
//
// Renamed from `engine.subscribe` to avoid name-collision with the
// DSL `.subscribe` builder method (dx-optimizer R1 finding).
//
// The wrapper is thin — engine-assigned sequence numbers, dedup at the
// handler boundary (D5-RESOLVED exactly-once), cursor management, and
// per-event capability re-check all happen Rust-side. We only render
// the JSON shape the napi adapter returns into a `Subscription` with
// the `unsubscribe()` method exposed.

import { EDslInvalidShape } from "./errors.js";
import type { Chunk, SubscribeCursor, Subscription } from "./types.js";

// ---------------------------------------------------------------------------
// Native shape — JSON returned by the napi `on_change` adapter
// ---------------------------------------------------------------------------

/**
 * JSON shape the napi `on_change` adapter returns. Mirrors
 * `subscription_to_json` in `bindings/napi/src/subscribe.rs`.
 */
export interface NativeSubscriptionJson {
  active: boolean;
  pattern: string;
  cursor: SubscribeCursor;
  maxDeliveredSeq: number;
}

// ---------------------------------------------------------------------------
// Callback shape
// ---------------------------------------------------------------------------

/**
 * Callback signature for an `engine.onChange` registration. Receives a
 * `(seq, chunk)` pair so dedup-aware consumers can correlate with
 * `subscription.maxDeliveredSeq` for cross-process continuation.
 *
 * The chunk is delivered as a Node `Buffer` (the wire-side `Chunk`
 * shape).
 */
export type OnChangeCallback = (seq: number, chunk: Chunk) => void;

// ---------------------------------------------------------------------------
// Subscription factory
// ---------------------------------------------------------------------------

/**
 * Wrap a native subscription JSON shape as a [`Subscription`] handle
 * with an `unsubscribe()` method that flips the active flag (and, once
 * G6-A's change-stream port is wired across the napi boundary,
 * de-registers the callback).
 *
 * The handle's `active` / `maxDeliveredSeq` fields are snapshotted at
 * construction time. Production consumers should call
 * `engine.onChange(...)` again to read fresh state — the JS-side
 * `Subscription` object is intentionally immutable apart from the
 * `unsubscribe()` flip so it composes cleanly with React state, etc.
 */
export function makeSubscription(
  raw: NativeSubscriptionJson,
  onUnsubscribe: () => void = () => {},
): Subscription {
  let live = raw.active;
  return {
    get active(): boolean {
      return live;
    },
    pattern: raw.pattern,
    cursor: raw.cursor,
    maxDeliveredSeq: raw.maxDeliveredSeq,
    unsubscribe(): void {
      if (!live) return;
      live = false;
      onUnsubscribe();
    },
  };
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/**
 * Validate the (pattern, callback) pair an `onChange` call must supply.
 * Throws `EDslInvalidShape` early so caller-side bugs surface a typed
 * error before crossing the napi boundary.
 */
export function validateOnChangeArgs(
  pattern: string,
  callback: OnChangeCallback,
): void {
  if (typeof pattern !== "string" || pattern.length === 0) {
    throw new EDslInvalidShape(
      "onChange: pattern must be a non-empty event-name glob",
    );
  }
  if (typeof callback !== "function") {
    throw new EDslInvalidShape("onChange: callback must be a function");
  }
}

/**
 * Serialize a [`SubscribeCursor`] into the JSON shape the napi
 * `parse_cursor` adapter accepts.
 */
export function serializeCursor(cursor?: SubscribeCursor): unknown {
  if (!cursor) return null;
  switch (cursor.kind) {
    case "latest":
      return { kind: "latest" };
    case "sequence":
      return { kind: "sequence", seq: cursor.seq };
    case "persistent":
      return { kind: "persistent", subscriberId: cursor.subscriberId };
    default: {
      // Exhaustiveness check; surface a clean shape error for unknown
      // cursor variants rather than passing garbage to the napi adapter.
      const _exhaustive: never = cursor;
      void _exhaustive;
      throw new EDslInvalidShape(
        "onChange: unknown cursor kind (must be latest/sequence/persistent)",
      );
    }
  }
}
