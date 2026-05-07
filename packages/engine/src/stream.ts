// STREAM DSL surface (Phase 2b G6-B).
//
// This module exposes the TS-side `engine.callStream` / `engine.openStream`
// methods + the `subgraph(...).stream(args)` DSL builder method (which
// already lives in `dsl.ts`; here we layer the AsyncIterable wrapper that
// the napi `StreamHandleJs` class needs to implement
// `AsyncIterable<Chunk>`).
//
// Per plan §3 G6-B (dx-optimizer R1):
//
//   subgraph(...).stream(args)              // composition primitive (dsl.ts)
//   engine.callStream(handlerId, action, input) -> AsyncIterable<Chunk>
//   engine.openStream(...) -> StreamHandle (AsyncIterable + explicit close)
//   engine.testingOpenStreamForTest(chunks) // ts-r4-2 R4 vitest harness
//
// The wrapper is intentionally thin — all chunk transport, back-pressure,
// and persistence happens Rust-side via G6-A's executor + the napi class
// `StreamHandleJs`. We only add the JS-language `AsyncIterable` polish.
//
// Phase-3 G19-C2 wave-7 additions (§7.1.2 + stream-r1-4 + stream-r1-10):
//
//   - `wrapStreamHandle` plumbs the `requiresExplicitClose` accessor.
//   - `armLeakDetector` arms a `FinalizationRegistry` leak detector
//     against handles flagged `requiresExplicitClose: true` so an
//     unclosed `openStream(...)` handle that gets GC'd fires the
//     registered `engine.onStreamLeaked` callbacks with a typed
//     `E_STREAM_HANDLE_LEAKED` payload.
//   - `disarmLeakDetector` is invoked on explicit `close()` /
//     natural completion / handler shutdown so natural-completion
//     does NOT fire a false-positive (stream-r1-4 scenario c
//     negative pin).
//   - Native-vs-browser-target divergence per stream-r1-10: the
//     leak detector is V8/Node + WHATWG only (any runtime exposing
//     a global `FinalizationRegistry` constructor). When the
//     constructor is absent the wrapper falls back silently —
//     resource ownership stays correct regardless (Drop joins
//     producer thread); only the JS-surface observability is
//     skipped.

import { EDslInvalidShape, mapNativeError } from "./errors.js";
import type { Chunk, JsonValue, StreamHandle } from "./types.js";

// ---------------------------------------------------------------------------
// Native shape — the napi-rs-generated `StreamHandleJs` class
// ---------------------------------------------------------------------------

/**
 * Mirrors the native `StreamHandleJs` class shape from
 * `bindings/napi/src/lib.rs`. We type-erase to an interface here so the
 * wrapper compiles even when the native binding hasn't been rebuilt yet
 * (the wrapper falls back to a clean `E_DSL_INVALID_SHAPE` at call time).
 */
export interface NativeStreamHandle {
  next(): Buffer | null;
  close(): void;
  isDrained(): boolean;
  /**
   * Returns engine-assigned u64 narrowed to JS `number` via napi-rs's
   * i64 mapping (R6 Round-2 Instance 11 — widened from u32 to i64;
   * exact for values < Number.MAX_SAFE_INTEGER).
   */
  seqSoFar(): number;
  /**
   * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): `true` for handles
   * produced by `engine.openStream(...)` (explicit-close lifecycle);
   * `false` for handles produced by `engine.callStream(...)`
   * (AsyncIterable auto-close). Optional on the type so older napi
   * cdylibs (pre-G19-C2) still type-check; the TS-side wrapper falls
   * back to `false` (assume auto-close lifecycle) if the symbol is
   * absent.
   */
  requiresExplicitClose?: () => boolean;
}

// ---------------------------------------------------------------------------
// AsyncIterable wrapper
// ---------------------------------------------------------------------------

/**
 * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): `E_STREAM_HANDLE_LEAKED`
 * payload shape passed to every `engine.onStreamLeaked(callback)`
 * callback when the FinalizationRegistry detects an unclosed
 * explicit-close handle being GC'd. Mirrors the
 * `benten_errors::ErrorCode::StreamHandleLeaked` typed catalog code.
 */
export interface StreamHandleLeakedEvent {
  readonly code: "E_STREAM_HANDLE_LEAKED";
  /**
   * Best-effort cause discriminator. `"gc-without-close"` fires from
   * the FinalizationRegistry callback path; `"shutdown-drain"` fires
   * when `Engine.shutdown()` walks still-open handles.
   */
  readonly cause: "gc-without-close" | "shutdown-drain";
}

/**
 * Process-wide leak callback registry. `engine.onStreamLeaked(cb)`
 * appends; the FinalizationRegistry callback + `Engine.shutdown()`
 * drain path consult this list. Module-scoped (NOT per-Engine) because
 * `FinalizationRegistry` is process-global; multi-Engine tests should
 * unregister via the returned disposer before constructing a fresh
 * engine.
 */
const leakCallbacks: Array<(ev: StreamHandleLeakedEvent) => void> = [];

/**
 * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): register a leak
 * callback. Returns a disposer that removes the callback from the
 * registry. The TS-side `Engine.onStreamLeaked` simply forwards here.
 */
export function registerStreamLeakCallback(
  cb: (ev: StreamHandleLeakedEvent) => void,
): () => void {
  leakCallbacks.push(cb);
  return () => {
    const idx = leakCallbacks.indexOf(cb);
    if (idx >= 0) leakCallbacks.splice(idx, 1);
  };
}

/**
 * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): fire every registered
 * callback with the supplied event. Used both by the
 * FinalizationRegistry callback path AND by the `Engine.shutdown()`
 * drain. Errors thrown by individual callbacks are caught + swallowed
 * so a misbehaving consumer cannot starve other consumers (matching
 * the `change_stream` drop-oldest discipline).
 */
export function fireStreamLeak(ev: StreamHandleLeakedEvent): void {
  for (const cb of leakCallbacks.slice()) {
    try {
      cb(ev);
    } catch {
      // Swallow — leak observability must not be load-bearing for any
      // single consumer.
    }
  }
}

/**
 * Per-handle bookkeeping registered with the `FinalizationRegistry`.
 * `disarmed` is flipped on explicit `close()` / natural completion so
 * the GC callback can short-circuit (scenario-c negative pin).
 */
interface LeakBookkeeping {
  disarmed: boolean;
}

/**
 * Process-wide finalization registry. Lazily constructed on first arm
 * so runtimes lacking the constructor (older WHATWG / non-Node) skip
 * cleanly. Per stream-r1-10 the leak detector is V8/Node + WHATWG
 * only; native ownership semantics stay correct regardless (Drop
 * joins the producer thread).
 */
let leakRegistry: FinalizationRegistry<LeakBookkeeping> | null = null;
let leakRegistryAttempted = false;

function ensureLeakRegistry(): FinalizationRegistry<LeakBookkeeping> | null {
  if (leakRegistryAttempted) return leakRegistry;
  leakRegistryAttempted = true;
  const Ctor = (globalThis as { FinalizationRegistry?: typeof FinalizationRegistry })
    .FinalizationRegistry;
  if (typeof Ctor !== "function") return null;
  leakRegistry = new Ctor((bookkeeping: LeakBookkeeping) => {
    if (bookkeeping.disarmed) return;
    fireStreamLeak({
      code: "E_STREAM_HANDLE_LEAKED",
      cause: "gc-without-close",
    });
  });
  return leakRegistry;
}

/**
 * Wrap a [`NativeStreamHandle`] as a [`StreamHandle`] — adds the
 * `[Symbol.asyncIterator]()` glue so consumers can `for await` it,
 * forwards `next` / `close` / `isDrained` / `seqSoFar` straight through.
 *
 * The async-iterator's `next()` calls the native sync `next()` and
 * resolves the result. Real back-pressure handling is Rust-side; the
 * JS-side iterator is a thin shell.
 *
 * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): when the underlying
 * `NativeStreamHandle.requiresExplicitClose()` returns `true`, arms a
 * `FinalizationRegistry` so the JS handle being GC'd without an
 * explicit `close()` fires `E_STREAM_HANDLE_LEAKED` to every
 * registered `engine.onStreamLeaked(...)` callback. Disarmed on
 * explicit `close()` (so the negative pin — natural completion + close
 * + GC — does NOT fire) and on the iterator's `return()` path
 * (`for await ... break`).
 */
export function wrapStreamHandle(native: NativeStreamHandle): StreamHandle {
  // Phase-3 G19-C2 wave-7 (§7.1.2): determine whether to arm leak
  // detection. Pre-G19-C2 cdylibs without the accessor fall back to
  // `false` (assume auto-close lifecycle) — the symbol-presence test
  // pin asserts the accessor exists in the wave-7 build.
  const requiresExplicitClose: boolean =
    typeof native.requiresExplicitClose === "function"
      ? native.requiresExplicitClose()
      : false;

  const bookkeeping: LeakBookkeeping = { disarmed: !requiresExplicitClose };

  const close = () => {
    bookkeeping.disarmed = true;
    native.close();
  };

  const handle: StreamHandle = {
    next: () => native.next(),
    close,
    isDrained: () => native.isDrained(),
    seqSoFar: () => native.seqSoFar(),
    requiresExplicitClose: () => requiresExplicitClose,
    [Symbol.asyncIterator](): AsyncIterator<Chunk> {
      return {
        next: async (): Promise<IteratorResult<Chunk>> => {
          try {
            const chunk = native.next();
            if (chunk === null) {
              // Natural completion — disarm so the negative pin
              // (scenario c) does NOT fire E_STREAM_HANDLE_LEAKED.
              bookkeeping.disarmed = true;
              return { value: undefined as unknown as Chunk, done: true };
            }
            return { value: chunk, done: false };
          } catch (err) {
            // Disarm + re-raise so iterator-rejection paths don't
            // double-count as leaks.
            bookkeeping.disarmed = true;
            // Surface typed errors through the iterator's reject path.
            throw mapNativeError(err);
          }
        },
        return: async (): Promise<IteratorResult<Chunk>> => {
          // `for await ... of` calls `return()` when the consumer breaks
          // out early; close the handle so the underlying executor
          // releases its mpsc receiver promptly.
          close();
          return { value: undefined as unknown as Chunk, done: true };
        },
      };
    },
  };

  if (requiresExplicitClose) {
    const registry = ensureLeakRegistry();
    if (registry) {
      // Per WHATWG FinalizationRegistry contract: register the
      // user-facing `handle` object. When `handle` becomes
      // unreachable, the callback fires with `bookkeeping`; we
      // keep `bookkeeping` reachable via the closure so its
      // `disarmed` flag can be flipped by close() / iterator-end.
      registry.register(handle, bookkeeping);
    }
  }

  return handle;
}

// ---------------------------------------------------------------------------
// Validation helpers used by the Engine wrapper's callStream / openStream
// ---------------------------------------------------------------------------

/**
 * Validate the (handlerId, action, input) tuple a `callStream` /
 * `openStream` call must supply. Throws `EDslInvalidShape` early so
 * caller-side bugs surface a typed error before crossing the napi
 * boundary.
 */
export function validateStreamCallArgs(
  handlerId: string,
  action: string,
  input: JsonValue,
): void {
  if (typeof handlerId !== "string" || handlerId.length === 0) {
    throw new EDslInvalidShape("callStream: handlerId must be a non-empty string");
  }
  if (typeof action !== "string" || action.length === 0) {
    throw new EDslInvalidShape("callStream: action must be a non-empty string");
  }
  if (input !== null && typeof input !== "object") {
    throw new EDslInvalidShape(
      "callStream: input must be a JSON object or null",
    );
  }
}
