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
    return this.inner.registerHandlerFromDsl(handlerId, op, source);
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
    return this.inner.replaceHandlerFromDsl(handlerId, op, source);
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
