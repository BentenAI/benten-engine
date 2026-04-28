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
  seqSoFar(): number;
}

// ---------------------------------------------------------------------------
// AsyncIterable wrapper
// ---------------------------------------------------------------------------

/**
 * Wrap a [`NativeStreamHandle`] as a [`StreamHandle`] — adds the
 * `[Symbol.asyncIterator]()` glue so consumers can `for await` it,
 * forwards `next` / `close` / `isDrained` / `seqSoFar` straight through.
 *
 * The async-iterator's `next()` calls the native sync `next()` and
 * resolves the result. Real back-pressure handling is Rust-side; the
 * JS-side iterator is a thin shell.
 */
export function wrapStreamHandle(native: NativeStreamHandle): StreamHandle {
  const handle: StreamHandle = {
    next: () => native.next(),
    close: () => native.close(),
    isDrained: () => native.isDrained(),
    seqSoFar: () => native.seqSoFar(),
    [Symbol.asyncIterator](): AsyncIterator<Chunk> {
      return {
        next: async (): Promise<IteratorResult<Chunk>> => {
          try {
            const chunk = native.next();
            if (chunk === null) {
              return { value: undefined as unknown as Chunk, done: true };
            }
            return { value: chunk, done: false };
          } catch (err) {
            // Surface typed errors through the iterator's reject path.
            throw mapNativeError(err);
          }
        },
        return: async (): Promise<IteratorResult<Chunk>> => {
          // `for await ... of` calls `return()` when the consumer breaks
          // out early; close the handle so the underlying executor
          // releases its mpsc receiver promptly.
          native.close();
          return { value: undefined as unknown as Chunk, done: true };
        },
      };
    },
  };
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
