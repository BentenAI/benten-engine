// `@benten/engine-devserver` — TypeScript wrapper around the napi-rs
// `DevServer` class shipped by `@benten/engine-native`. Phase 2b Wave-8f.
//
// Three responsibilities:
//   1. Lazy-load the napi-rs native artifact via `createRequire()` so
//      ESM consumers can `import { BentenDevServer } from "@benten/engine-devserver"`
//      without hitting the "ERR_REQUIRE_ESM" / "cannot find .node" traps.
//   2. Provide the JS-side `BentenDevServer` class with the full
//      lifecycle (start / stop / register / replace / subscribe).
//   3. Provide `editHandler` / `waitForReload` async helpers that the
//      Vitest harness drives — these are the surface the
//      `tools/benten-dev/test/*.test.ts` suite consumes.
//
// The wrapper is intentionally thin — invariant enforcement, DSL
// compilation, registration, and version-chain bookkeeping all happen
// Rust-side. This module transports shapes, not semantics.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { createRequire } from "node:module";

import { mapNativeError } from "@benten/engine/errors";

// ---------------------------------------------------------------------------
// Native binding shape
// ---------------------------------------------------------------------------

interface NativeReloadEvent {
  handlerId: string;
  op: string;
  versionTag: string;
  newCid?: string;
  previousCid?: string;
}

interface NativeReloadSubscriber {
  drain(): NativeReloadEvent[];
  hasEvents(): boolean;
  unsubscribe(): void;
}

interface NativeDevServer {
  start(): void;
  stop(): void;
  registerHandlerFromDsl(
    handlerId: string,
    op: string,
    source: string,
  ): string;
  replaceHandlerFromDsl(
    handlerId: string,
    op: string,
    source: string,
  ): string;
  // R6FP-tail (Round-2 Instance 10) — structured replace outcome.
  // Returns either the full RegisterReplaceOutcome JSON
  // `{ handlerId, cid, previousCid, chainDepth, versionTag, replaced }`
  // when engine routing is enabled, or `{ legacyOnly: true, handlerId }`
  // when the dev-server is in legacy in-memory mode.
  replaceHandlerFromDslWithOutcome?(
    handlerId: string,
    op: string,
    source: string,
  ): unknown;
  grantCapability(actor: string, scope: string): void;
  grantExists(actor: string, scope: string): boolean;
  subscribeToReloadEvents(): NativeReloadSubscriber;
  workspaceRoot: string;
}

interface NativeDevServerCtor {
  new (workspaceRoot: string): NativeDevServer;
}

interface NativeModule {
  DevServer: NativeDevServerCtor;
}

let __native: NativeModule | undefined;

function loadNative(): NativeModule {
  if (__native) return __native;
  try {
    const require = createRequire(import.meta.url);
    const mod = require("@benten/engine-native") as NativeModule;
    if (!mod || typeof mod.DevServer !== "function") {
      throw new Error(
        "@benten/engine-native did not export a `DevServer` class — binding may be stale (Wave-8f) or built without the napi-export feature",
      );
    }
    __native = mod;
    return __native;
  } catch (err) {
    const e = new Error(
      `@benten/engine-native not loadable — did \`napi build\` run in bindings/napi? (${(err as Error).message ?? err})`,
    );
    e.name = "BentenNativeNotLoaded";
    throw e;
  }
}

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/**
 * Per-event shape surfaced to JS consumers via
 * {@link BentenDevServer.subscribeToReloadEvents}.
 */
export interface ReloadEvent {
  /** Handler id whose body was registered or replaced. */
  readonly handlerId: string;
  /** Op the source was registered under (`"run"` / `"create"` / …). */
  readonly op: string;
  /** Surrogate `vN` tag stamped by the dev-server. */
  readonly versionTag: string;
  /** New live CID — present only when engine routing is enabled. */
  readonly newCid?: string;
  /** Predecessor CID — present only on a real replace (not first registration / not idempotent). */
  readonly previousCid?: string;
}

/**
 * R6FP-tail (Round-2 Instance 10) discriminated return shape for
 * {@link BentenDevServer.replaceHandlerWithOutcome}. Engine-routed
 * branches return the structured outcome; legacy in-memory mode
 * returns the discriminated `{ legacyOnly: true, handlerId }` shape so
 * callers can pivot consistently. R6 Round-3 r6-r3-napi-2 lifted the
 * legacy-fallback synth to honour the discriminator.
 */
export type ReplaceOutcomeResult =
  | {
      handlerId: string;
      cid: string;
      previousCid: string | null;
      chainDepth: number;
      versionTag: string;
      replaced: boolean;
    }
  | { legacyOnly: true; handlerId: string };

/**
 * Internal — minimal native shape `resolveReplaceOutcome` consumes.
 * Mirrors the subset of {@link NativeDevServer} used by the wrapper
 * function so unit tests can synthesize stubs without the full native
 * surface.
 */
interface ReplaceOutcomeNativeShape {
  replaceHandlerFromDsl(
    handlerId: string,
    op: string,
    source: string,
  ): string;
  replaceHandlerFromDslWithOutcome?(
    handlerId: string,
    op: string,
    source: string,
  ): unknown;
}

/**
 * R6FP-tail Instance 10 + R6 Round-3 r6-r3-napi-2 — pure helper that
 * resolves the structured replace-outcome shape against either the
 * Instance-10 napi surface (`replaceHandlerFromDslWithOutcome`) or the
 * pre-Instance-10 legacy surface (`replaceHandlerFromDsl` + synthesized
 * `legacyOnly: true` fallback).
 *
 * Exported as a load-bearing helper so the unit-test harness in
 * `packages/engine-devserver/test/replace_outcome.test.ts` can exercise
 * the discriminator-pivot contract without instantiating a full
 * `BentenDevServer` (which requires the native binding to be built).
 *
 * R6 Round-3 r6-r3-napi-2 closure: pre-fix the legacy-fallback branch
 * synthesized `{ chainDepth: 1, versionTag: "v1", replaced: false }`
 * defaults + omitted the `legacyOnly` discriminator the docstring
 * promised; consumer pivot logic `if (result.legacyOnly === true)` then
 * misrouted to the engine-routed branch with fake audit-trail values.
 * Post-fix: legacy fallback returns `{ legacyOnly: true, handlerId }` so
 * callers know the structured outcome was unavailable + can rebuild
 * the native binding to obtain real audit data.
 */
export function resolveReplaceOutcome(
  inner: ReplaceOutcomeNativeShape,
  handlerId: string,
  op: string,
  source: string,
): ReplaceOutcomeResult {
  if (typeof inner.replaceHandlerFromDslWithOutcome !== "function") {
    // Pre-Instance-10 native binding lacks the structured surface; drive
    // the legacy `replaceHandlerFromDsl` for its side-effect (the
    // replace itself still happens) + return the discriminated
    // `{ legacyOnly: true, handlerId }` shape so callers can pivot on
    // `legacyOnly` per the union return type.
    try {
      inner.replaceHandlerFromDsl(handlerId, op, source);
    } catch (err) {
      throw mapNativeError(err);
    }
    return { legacyOnly: true, handlerId };
  }
  let raw: unknown;
  try {
    raw = inner.replaceHandlerFromDslWithOutcome(handlerId, op, source);
  } catch (err) {
    throw mapNativeError(err);
  }
  if (
    raw &&
    typeof raw === "object" &&
    (raw as { legacyOnly?: unknown }).legacyOnly === true
  ) {
    return raw as { legacyOnly: true; handlerId: string };
  }
  return raw as {
    handlerId: string;
    cid: string;
    previousCid: string | null;
    chainDepth: number;
    versionTag: string;
    replaced: boolean;
  };
}

/**
 * Typed error surfaced by {@link BentenDevServer} on harness mis-wires
 * (e.g. {@link BentenDevServer.waitForReload} timing out without a
 * reload event). Carries a stable `name` so test fixtures can assert
 * `err instanceof BentenDevServerError` without string-matching.
 */
export class BentenDevServerError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BentenDevServerError";
  }
}

/**
 * Drainable reload-event subscriber. Returned by
 * {@link BentenDevServer.subscribeToReloadEvents}. Call `.unsubscribe()`
 * to stop receiving events; not calling it is fine — when the JS
 * reference goes out of scope the publisher prunes the subscriber on
 * its next event.
 */
export class ReloadSubscriber {
  private readonly inner: NativeReloadSubscriber;
  private active = true;

  /** @internal — constructed via {@link BentenDevServer.subscribeToReloadEvents}. */
  constructor(inner: NativeReloadSubscriber) {
    this.inner = inner;
  }

  /**
   * Drain all events the publisher has buffered for this subscriber
   * since the last drain. Returns events in arrival order.
   */
  drain(): ReloadEvent[] {
    if (!this.active) return [];
    return this.inner.drain();
  }

  /** Whether the subscriber currently has any buffered events. */
  hasEvents(): boolean {
    if (!this.active) return false;
    return this.inner.hasEvents();
  }

  /** Eagerly unsubscribe. Idempotent. */
  unsubscribe(): void {
    if (!this.active) return;
    this.active = false;
    this.inner.unsubscribe();
  }
}

/** Constructor options for {@link BentenDevServer}. */
export interface BentenDevServerOptions {
  /**
   * Workspace root directory. The dev-server opens its embedded engine
   * at `<projectRoot>/.benten/.benten-dev.redb`. The directory is
   * created if it does not exist.
   *
   * Phase-2a-era tests pass `projectRoot` and the dev-server treats
   * `<projectRoot>/.benten/` as its scratch space.
   */
  projectRoot: string;
}

/**
 * JS-side dev-server handle. Mirrors the Rust `benten_dev::DevServer`
 * lifecycle + adds Vitest-fixture helpers (`editHandler`, `waitForReload`).
 *
 * @example
 * ```ts
 * const server = new BentenDevServer({ projectRoot: "/tmp/proj" });
 * await server.start();
 * await server.registerHandler("h1", "run", "handler 'h1' { read('post') -> respond }");
 * await server.stop();
 * ```
 */
export class BentenDevServer {
  private readonly inner: NativeDevServer;
  private readonly projectRoot: string;
  private readonly workspaceDir: string;
  private internalSubscriber?: ReloadSubscriber;

  constructor(opts: BentenDevServerOptions) {
    const native = loadNative();
    this.projectRoot = opts.projectRoot;
    this.workspaceDir = `${opts.projectRoot}/.benten`;
    // The dev-server's redb file lives at `<workspaceDir>/.benten-dev.redb` —
    // the parent dir must exist before `.start()` opens it.
    mkdirSync(this.workspaceDir, { recursive: true });
    this.inner = new native.DevServer(this.workspaceDir);
  }

  /** Open the embedded engine + activate the dev-server. Idempotent. */
  async start(): Promise<void> {
    this.inner.start();
    // Self-subscribe so `waitForReload()` can poll without forcing every
    // caller to manage a subscriber explicitly.
    if (!this.internalSubscriber) {
      this.internalSubscriber = new ReloadSubscriber(
        this.inner.subscribeToReloadEvents(),
      );
    }
  }

  /** Tear down the embedded engine. Idempotent. */
  async stop(): Promise<void> {
    if (this.internalSubscriber) {
      this.internalSubscriber.unsubscribe();
      this.internalSubscriber = undefined;
    }
    this.inner.stop();
  }

  /**
   * First-time registration of a handler from a DSL source string.
   *
   * Routes through `Engine::register_subgraph_replace` underneath so a
   * subsequent `registerHandler` with the same `handlerId` and a different
   * body is treated as a hot-reload (NOT a `DuplicateHandler` error).
   *
   * Returns the engine-side handler id (the one the DSL declared,
   * normalised to the caller-supplied id).
   */
  async registerHandler(
    handlerId: string,
    op: string,
    source: string,
  ): Promise<string> {
    try {
      return this.inner.registerHandlerFromDsl(handlerId, op, source);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Explicit hot-replace alias for {@link BentenDevServer.registerHandler}.
   *
   * Same semantics as `registerHandler` (which already handles replace
   * via the engine's `register_subgraph_replace` API). The alias makes
   * intent explicit at the JS surface for callers that want to assert
   * "this is a hot-reload, not a first-registration".
   */
  async replaceHandler(
    handlerId: string,
    op: string,
    source: string,
  ): Promise<string> {
    try {
      return this.inner.replaceHandlerFromDsl(handlerId, op, source);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * R6FP-tail (Round-2 Instance 10) — replace a handler from DSL +
   * return the structured replace outcome when engine routing is
   * enabled.
   *
   * Returns the full `{ handlerId, cid, previousCid, chainDepth,
   * versionTag, replaced }` shape (matching
   * `Engine.replaceSubgraph(...)`'s structured return) so DevServer
   * consumers can correlate hot-replace observability without
   * subscribing to reload events. When the dev-server is in legacy
   * in-memory mode (engine routing disabled), returns
   * `{ legacyOnly: true, handlerId }`.
   *
   * Pre-Instance-10 the only return shape was the new-CID string
   * (`replaceHandler`); `previousCid` / `chainDepth` / `versionTag`
   * were dropped at the napi boundary.
   */
  async replaceHandlerWithOutcome(
    handlerId: string,
    op: string,
    source: string,
  ): Promise<ReplaceOutcomeResult> {
    return resolveReplaceOutcome(this.inner, handlerId, op, source);
  }

  /**
   * Grant a capability to a friendly principal. Mirrors
   * `Engine.grantCapability` but bound to the dev-server's grant table
   * (which survives hot-reload).
   */
  async grantCapability(opts: {
    actor: string;
    scope: string;
  }): Promise<void> {
    this.inner.grantCapability(opts.actor, opts.scope);
  }

  /** Whether a friendly principal currently holds the named scope. */
  async grantExists(opts: {
    actor: string;
    scope: string;
  }): Promise<boolean> {
    return this.inner.grantExists(opts.actor, opts.scope);
  }

  /**
   * Subscribe to hot-reload events. Returns a {@link ReloadSubscriber}
   * the caller drains at its own cadence. Unsubscribe via the returned
   * subscriber's `.unsubscribe()` method.
   */
  subscribeToReloadEvents(): ReloadSubscriber {
    return new ReloadSubscriber(this.inner.subscribeToReloadEvents());
  }

  /**
   * Vitest-fixture helper: edit a handler source file relative to the
   * project root + register the new content via
   * {@link BentenDevServer.replaceHandler}. The relative path is treated
   * as a hint for the JS-side handler-id; the DSL's declared handler id
   * is the authoritative one routed to the engine.
   *
   * # Honest scope (r6-dx-3 closure)
   *
   * Vitest-fixture helper: write a handler source file relative to the
   * project root. **THIS HELPER DOES NOT REGISTER THE SOURCE** — pair
   * with explicit {@link BentenDevServer.registerHandler} /
   * {@link BentenDevServer.replaceHandler} so the dev-server publishes
   * a reload event {@link BentenDevServer.waitForReload} can observe.
   *
   * The wave-8f-fp-amended waitForReload throws on timeout so the
   * silent-failure footgun (caller writes a file, calls waitForReload,
   * waits 1500ms with no diagnostic) is mitigated; it now surfaces a
   * BentenDevServerError naming the likely mis-wire.
   *
   * Prior docstring claimed this helper "immediately registers" the
   * source AND "emits a synthetic reload tick" — neither was ever
   * implemented. The honest contract is a plain disk write.
   */
  async editHandler(relPath: string, content: string): Promise<void> {
    const fullPath = `${this.projectRoot}/${relPath}`;
    mkdirSync(dirname(fullPath), { recursive: true });
    writeFileSync(fullPath, content, "utf8");
  }

  /**
   * Vitest-fixture helper: wait for at least one reload event to be
   * observed via the internal subscriber.
   *
   * Defaults: 1500ms timeout, 25ms poll interval. A successful
   * `registerHandler` / `replaceHandler` immediately publishes a reload
   * event so the wait typically returns on the first poll.
   *
   * @throws {@link BentenDevServerError} if no reload event is observed
   * within `timeoutMs`. Fail-loud is the correct DX default — a silent
   * timeout would cause a fixture that expects editHandler-then-
   * waitForReload to silently move past the wait + then surface a
   * misleading assertion failure on whatever line of test code uses the
   * (never-published) reload outcome. The explicit throw names the
   * mis-wire (no `registerHandler` / `replaceHandler` call matched the
   * expected handler_id, OR the `editHandler` helper was called without
   * a follow-up `replaceHandler`).
   */
  async waitForReload(opts?: {
    timeoutMs?: number;
    pollMs?: number;
  }): Promise<void> {
    const timeoutMs = opts?.timeoutMs ?? 1500;
    const pollMs = opts?.pollMs ?? 25;
    const sub = this.internalSubscriber;
    if (!sub) {
      throw new BentenDevServerError(
        "BentenDevServer.waitForReload(): server has not been started — call .start() first",
      );
    }
    const deadline = Date.now() + timeoutMs;
    // Fast-path: events already pending.
    if (sub.hasEvents()) {
      sub.drain();
      return;
    }
    while (Date.now() < deadline) {
      await new Promise<void>((resolve) => setTimeout(resolve, pollMs));
      if (sub.hasEvents()) {
        sub.drain();
        return;
      }
    }
    throw new BentenDevServerError(
      `BentenDevServer.waitForReload(): no reload event observed within ${timeoutMs}ms — did the editHandler/registerHandler/replaceHandler call match the expected handler_id? The editHandler helper writes the source file but does NOT register through the engine; pair it with replaceHandler to publish a reload event.`,
    );
  }

  /** Workspace directory (the `<projectRoot>/.benten/` dir). */
  get workspaceRoot(): string {
    return this.workspaceDir;
  }
}

// Re-export types so consumers don't need to dig for them.
export type { BentenDevServerOptions as DevServerOptions };

// Re-export Engine from @benten/engine so a single
// `import { BentenDevServer, Engine } from "@benten/engine-devserver"`
// covers both surfaces. Phase-2b devserver users will commonly want
// to hand-construct an Engine alongside a BentenDevServer (e.g. a
// CLI that drives engine.call against the devserver's redb file
// after a `dev.stop()`); the re-export saves a separate import
// path. See finding 8f-dx-8 in r5-mr-w8f-dx-optimizer.json.
export { Engine } from "@benten/engine";
