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
// per-event capability re-check all happen Rust-side. We hold the
// napi-class `SubscriptionJs` handle alive on the JS-side
// `Subscription` so the underlying Rust `benten_engine::Subscription`
// (which owns the `napi::ThreadsafeFunction` Arc backing the JS
// callback) stays alive across the napi boundary; dropping the
// JS-side handle (or calling `unsubscribe()`) calls into Rust to
// release the registry slot.

import { EDslInvalidShape } from "./errors.js";
import type {
  Chunk,
  EmitSubscription,
  JsonValue,
  SubscribeCursor,
  Subscription,
} from "./types.js";

// ---------------------------------------------------------------------------
// Native shape — `SubscriptionJs` napi class returned by the adapter
// ---------------------------------------------------------------------------

/**
 * Native `SubscriptionJs` napi class (cr-w8c-fp-1 fix-pass).
 *
 * Mirrors `bindings/napi/src/lib.rs::SubscriptionJs`. The handle holds
 * the underlying `benten_engine::Subscription` alive on the Rust side;
 * dropping this handle (or calling `unsubscribe()`) releases the
 * registry slot AND the `napi::ThreadsafeFunction` Arc backing the JS
 * callback. The pre-fix-pass JSON-shape return was unsound — the
 * Subscription dropped at end of method scope and JS callbacks never
 * fired.
 */
export interface NativeSubscriptionJs {
  isActive(): boolean;
  pattern(): string;
  /**
   * Returns engine-assigned u64 narrowed to JS `number` via napi-rs's
   * i64 mapping (R6 Round-2 Instance 11 — widened from u32 to i64;
   * exact for values < Number.MAX_SAFE_INTEGER).
   */
  maxDeliveredSeq(): number;
  unsubscribe(): void;
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
 * Wrap a native `SubscriptionJs` handle as a [`Subscription`] DSL
 * handle. The wrapper retains the native handle internally so the
 * underlying Rust `Subscription` (and its `ThreadsafeFunction` Arc) is
 * held alive for the lifetime of the JS-side reference.
 *
 * `cursor` is captured at registration time on the JS side because the
 * native handle does not currently surface it — the cursor is only
 * meaningful at registration and to JS callers introspecting the shape
 * they passed in.
 *
 * `unsubscribe()` is idempotent: subsequent calls are a no-op + the
 * native handle is told `unsubscribe()` exactly once. The native side
 * is itself idempotent (Rust `Subscription::unsubscribe` is safe to
 * call multiple times).
 */
export function wrapSubscriptionHandle(
  native: NativeSubscriptionJs,
  cursor: SubscribeCursor,
): Subscription {
  // Snapshot the pattern (immutable post-registration) so it can be
  // exposed as a plain readonly field. `active` + `maxDeliveredSeq`
  // both go through live getters that round-trip to the native handle
  // — the underlying engine-side atomic bumps on every delivery, so a
  // snapshotted value goes stale immediately (Round-2 Instance 7
  // closure: prior shape captured `maxDeliveredSeq` once at
  // construction time when the value was 0; consumers reading the
  // field post-delivery saw a permanent zero).
  const pattern = native.pattern();
  let unsubscribed = false;
  return {
    get active(): boolean {
      if (unsubscribed) return false;
      return native.isActive();
    },
    pattern,
    cursor,
    get maxDeliveredSeq(): number {
      // Always read through to the native atomic. After unsubscribe
      // the underlying handle still reports the final value (it is
      // bumped at delivery time, not unsubscribe time), so we don't
      // gate this on the unsubscribed flag.
      return native.maxDeliveredSeq();
    },
    unsubscribe(): void {
      if (unsubscribed) return;
      unsubscribed = true;
      native.unsubscribe();
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

// ---------------------------------------------------------------------------
// EMIT subscription surface (r6-mpc-2 closure — wave-8h audit-gap fix)
// ---------------------------------------------------------------------------

/**
 * Native `EmitSubscriptionJs` napi class.
 *
 * Mirrors `bindings/napi/src/lib.rs::EmitSubscriptionJs` (added by R6-FP
 * Group 1). Holds the underlying engine-side EMIT-broadcast subscriber
 * slot alive on the Rust side; dropping the handle (or calling
 * `unsubscribe()`) releases the slot AND the
 * `napi::ThreadsafeFunction` Arc backing the JS callback.
 */
export interface NativeEmitSubscriptionJs {
  isActive(): boolean;
  channel(): string;
  unsubscribe(): void;
}

/**
 * Callback signature for an `engine.onEmit` registration. Receives a
 * `(channel, payload)` pair — the channel name the EMIT primitive was
 * invoked with, and the payload value (deserialized from the engine's
 * Rust-side `Value` to a JS-side JSON-shape).
 */
export type OnEmitCallback = (channel: string, payload: JsonValue) => void;

/**
 * Wrap a native `EmitSubscriptionJs` handle as an [`EmitSubscription`]
 * DSL handle. The wrapper retains the native handle internally so the
 * underlying Rust EMIT-broadcast slot (and its `ThreadsafeFunction`
 * Arc) is held alive for the lifetime of the JS-side reference.
 *
 * `unsubscribe()` is idempotent: subsequent calls are a no-op + the
 * native handle is told `unsubscribe()` exactly once. The native side
 * is itself idempotent.
 */
export function wrapEmitSubscriptionHandle(
  native: NativeEmitSubscriptionJs,
): EmitSubscription {
  const channel = native.channel();
  let unsubscribed = false;
  return {
    get active(): boolean {
      if (unsubscribed) return false;
      return native.isActive();
    },
    channel,
    unsubscribe(): void {
      if (unsubscribed) return;
      unsubscribed = true;
      native.unsubscribe();
    },
  };
}

/**
 * Validate the (channel, callback) pair an `onEmit` call must supply.
 * Throws `EDslInvalidShape` early so caller-side bugs surface a typed
 * error before crossing the napi boundary.
 */
export function validateOnEmitArgs(
  channel: string,
  callback: OnEmitCallback,
): void {
  if (typeof channel !== "string" || channel.length === 0) {
    throw new EDslInvalidShape(
      "onEmit: channel must be a non-empty string",
    );
  }
  if (typeof callback !== "function") {
    throw new EDslInvalidShape("onEmit: callback must be a function");
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
