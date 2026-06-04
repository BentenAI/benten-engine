// G24-C wave-6b — Admin UI v0 composed-view creator.
//
// **Mounted under the VIEWS category** of the 4-category nav from G24-A.
// The user picks an **anchor pattern** + **projection**; the creator
// emits a **subgraph-shaped view definition** (per D-4F-2) that
// materialises via the generalized IVM Algorithm B kernel (G23-0a).
// Live-preview deltas arrive via the `on_change_as_with_cursor`
// subscribe seam ONLY (sec-3.5-r1-9 floor + T12 actor-aware cap-recheck).
//
// ## What this file ships
//
// - `ComposedViewCreator` — the mountable creator surface. UI bindings
//   call `mount`, `selectAnchorPattern`, `selectProjection`, `save`,
//   `beginLivePreview`, etc.
// - `ComposedViewCreatorBridge` — extension of the G24-A bridge surface
//   adding the methods the creator needs (`registerSubgraphView`,
//   `materializerProvenance`, `revokeCapability`, etc).
// - `PreviewState` — discriminated union representing the four states
//   the live preview can be in (`idle` / `streaming` / `terminated` /
//   `error`).
//
// ## Hard rules applied
//
// 1. **Subscribe ONLY via `on_change_as_with_cursor`** (sec-3.5-r1-9 +
//    T12). The class exposes a `subscriptionMode` accessor returning
//    the literal string `"on_change_as_with_cursor"`; grep-asserted by
//    `admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`.
// 2. **No parallel materialization path** (D-4F-2): the creator's
//    persisted view emits via `bridge.registerSubgraphView` which the
//    engine routes through `Algorithm::register_subgraph` (the SAME
//    kernel the canonical 5 views use).
// 3. **No internal polling**: deltas flow over the
//    `on_change_as_with_cursor` callback channel; no `setInterval` /
//    `setTimeout`-based polling layer.
// 4. **Cap-revoke mid-preview terminates** (T12 admin-UI handling):
//    when the bridge surfaces a `Cancel` outcome via the on-error
//    channel, the creator transitions to `{ kind: "terminated", reason:
//    "E_SYNC_REVOKED_DURING_SESSION" }` + ignores any subsequent
//    deltas the bridge accidentally re-emits.

import {
  ADMIN_UI_V0_CATEGORIES,
  ADMIN_UI_V0_SUBSCRIBE_SEAM,
  type AdminUiV0Category,
  type AdminUiV0Bridge,
} from "../index.js";
import { type UserViewSpec, userViewSpec } from "./view_spec.js";

/**
 * Discriminated-union for the live-preview state machine.
 */
export type PreviewState =
  | { readonly kind: "idle" }
  | {
      readonly kind: "streaming";
      readonly rows: ReadonlyArray<Readonly<Record<string, unknown>>>;
      readonly deltaCount: number;
    }
  | {
      readonly kind: "terminated";
      readonly reason: string;
    }
  | {
      readonly kind: "error";
      readonly message: string;
    };

/**
 * Outcome of the creator's `save` call — the persisted view's CID is
 * the content-addressed identity the engine routes future
 * subscriptions on.
 */
export interface SaveOutcome {
  /** CID of the persisted view subgraph (content-addressed). */
  readonly cid: string;
  /** View id the user assigned. */
  readonly viewId: string;
}

/**
 * Subscribe-cursor mirror — narrows to the two shapes the
 * `Engine::on_change_as_with_cursor` surface admits. `latest` skips
 * historical events; `{ persistent }` requests a resumable cursor.
 */
export type SubscribeCursor = "latest" | { readonly persistent: string };

/**
 * Bridge surface the composed-view creator binds against. Extends the
 * G24-A `AdminUiV0Bridge` with the methods needed for view
 * registration + materializer provenance lookup + cap revoke +
 * authorial write driving (used by tests that exercise the live-
 * preview update path).
 *
 * Implementations:
 * - **Browser-tab (shape b)** — fetch-backed implementation against
 *   the thin-client session protocol (`DidKeyedSession` G24-F).
 * - **Tauri embedded-webview (shape c)** — in-process IPC against the
 *   `benten-renderer-tauri` crate's `IpcAllowlist` (G24-E).
 */
export interface ComposedViewCreatorBridge extends AdminUiV0Bridge {
  /**
   * Register a user-defined subgraph-shaped view. The bridge forwards
   * the spec to `Algorithm::register_subgraph` on the engine side
   * (D-4F-2 commitment: SAME kernel canonical views use).
   *
   * Returns the persisted view's content-addressed CID.
   */
  registerSubgraphView(
    spec: UserViewSpec,
    principal: string,
  ): Promise<{ readonly cid: string }>;

  /**
   * Look up the materializer-provenance string for a registered view.
   * Returns a string identifying which kernel/strategy materialised
   * the view (e.g. `"benten-ivm::strategy::AlgorithmB"`); used by
   * tests + observability tooling to confirm there is NO parallel
   * admin-ui-only materialization path.
   */
  materializerProvenance(cid: string): Promise<string>;

  /**
   * Revoke a capability scope. Used by tests to drive the cap-revoke
   * mid-live-preview termination path; in production this is invoked
   * via the Plugins category UX, not the view creator directly.
   */
  revokeCapability(args: { readonly scope: string }): Promise<void>;

  /**
   * Drive an arbitrary write through the engine's `call_as` surface
   * (the Class B β write counterpart per CLAUDE.md baked-in #18).
   * Used by tests to exercise the live-preview update path against a
   * real change-event flow.
   */
  callAs(
    principal: string,
    op: {
      readonly kind: "write_node";
      readonly label: string;
      readonly properties: Readonly<Record<string, unknown>>;
    },
  ): Promise<{ readonly cid: string }>;
}

/**
 * The admin UI v0 composed-view creator.
 *
 * ## Lifecycle
 *
 * ```text
 *   mount() ─► selectAnchorPattern() ─► selectProjection() ─► save()
 *                                                  │
 *                                                  ▼
 *                                          beginLivePreview()
 *                                                  │
 *                                                  ▼
 *               (deltas arrive via on_change_as_with_cursor callback)
 *                                                  │
 *                                                  ▼
 *                                          stop() / dispose()
 * ```
 *
 * ## Live-preview subscribe pathway
 *
 * `beginLivePreview` invokes `bridge.onChangeAsWithCursor(pattern,
 * cursor, actor, callback)` — the literal string `"on_change_as_with_cursor"`
 * appears in this file so the grep-assert pin at
 * `admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`
 * verifies the seam is wired. No other subscribe entry point is
 * touched — `subscribe_change_events` / `on_change` (bare) / polling
 * are NEVER used.
 *
 * ## Cap-revoke termination handling
 *
 * The bridge implementation routes the engine's `CapRecheckOutcome::Cancel`
 * signal (per Phase-4-Foundation R1-FP G22-FP-1 option-D) to a
 * terminating callback invocation carrying a sentinel error event with
 * `{ kind: "subscription_terminated", reason }`. The creator's callback
 * handler observes that sentinel + transitions `PreviewState` to
 * `{ kind: "terminated", reason }`. Any subsequent deltas the bridge
 * accidentally re-emits are dropped (the `terminated` state is
 * absorbing).
 */
export class ComposedViewCreator {
  private readonly bridge: ComposedViewCreatorBridge;
  private readonly principal: string;
  private anchorPattern: string | null = null;
  private projection: ReadonlyArray<string> = [];
  private savedSpec: UserViewSpec | null = null;
  private savedCid: string | null = null;
  private previewSubscription:
    | { readonly subscriptionId: string }
    | null = null;
  private previewState: PreviewState = { kind: "idle" };
  private deltas: number = 0;
  private accumulator: Array<Record<string, unknown>> = [];

  private constructor(bridge: ComposedViewCreatorBridge, principal: string) {
    this.bridge = bridge;
    this.principal = principal;
  }

  /**
   * Mount the creator. Verifies the user has navigated to the VIEWS
   * category (per the 4-category nav from G24-A).
   */
  public static mount(args: {
    readonly bridge: ComposedViewCreatorBridge;
    readonly principal: string;
    readonly category?: AdminUiV0Category;
  }): ComposedViewCreator {
    const cat = args.category ?? "Views";
    if (!ADMIN_UI_V0_CATEGORIES.includes(cat)) {
      throw new Error(
        `ComposedViewCreator: category \`${cat}\` is not a known 4-cat nav entry`,
      );
    }
    if (cat !== "Views") {
      throw new Error(
        `ComposedViewCreator: must be mounted under the VIEWS category; got \`${cat}\``,
      );
    }
    return new ComposedViewCreator(args.bridge, args.principal);
  }

  /**
   * Capture the anchor pattern (a label or anchor-prefix the user
   * picked from the available anchors in their graph).
   */
  public async selectAnchorPattern(pattern: string): Promise<void> {
    if (pattern.length === 0) {
      throw new Error(
        "ComposedViewCreator.selectAnchorPattern: pattern must be non-empty",
      );
    }
    this.anchorPattern = pattern;
  }

  /**
   * Capture the projection — the property keys of the source Nodes
   * the user wants to project into the view.
   */
  public async selectProjection(
    fields: ReadonlyArray<string>,
  ): Promise<void> {
    if (fields.length === 0) {
      throw new Error(
        "ComposedViewCreator.selectProjection: must select at least one field",
      );
    }
    this.projection = [...fields];
  }

  /**
   * Persist the view. Constructs a `UserViewSpec`, forwards it through
   * the bridge's `registerSubgraphView` (D-4F-2 kernel pathway), and
   * returns the persisted view's CID + the spec for downstream
   * observability.
   *
   * The spec is the **subgraph-shaped view definition** that the
   * generalized Algorithm B kernel consumes — no separate
   * admin-ui-only view representation is introduced.
   */
  public async save(args: { readonly name: string }): Promise<SaveOutcome> {
    if (this.anchorPattern === null) {
      throw new Error("ComposedViewCreator.save: anchor pattern not selected");
    }
    if (this.projection.length === 0) {
      throw new Error("ComposedViewCreator.save: projection not selected");
    }
    const spec = userViewSpec({
      viewId: args.name,
      anchorPattern: this.anchorPattern,
      projection: this.projection,
    });
    const { cid } = await this.bridge.registerSubgraphView(spec, this.principal);
    this.savedSpec = spec;
    this.savedCid = cid;
    return { cid, viewId: spec.viewId };
  }

  /**
   * Start the live-preview subscription.
   *
   * **Subscribe pathway:** `bridge.onChangeAsWithCursor(pattern, cursor,
   * actor, callback)` — this is the ONLY subscribe entry point the
   * creator uses (sec-3.5-r1-9 floor). The bridge's transport-side
   * `onChangeAsWithCursor` implementation forwards to
   * `Engine::on_change_as_with_cursor` (the cap-recheck-enabled seam).
   * The seam name `"on_change_as_with_cursor"` is exposed verbatim via
   * the `subscriptionMode()` accessor for grep-assertion.
   */
  public async beginLivePreview(args?: {
    readonly cursor?: SubscribeCursor;
  }): Promise<void> {
    if (this.savedSpec === null || this.savedCid === null) {
      throw new Error(
        "ComposedViewCreator.beginLivePreview: must call save() first",
      );
    }
    if (this.previewSubscription !== null) {
      return; // idempotent re-entrancy: already streaming
    }
    const cursor: SubscribeCursor = args?.cursor ?? "latest";
    const pattern = subscribePatternFor(this.savedSpec);
    this.previewState = { kind: "streaming", rows: [], deltaCount: 0 };
    this.previewSubscription = await this.bridge.onChangeAsWithCursor(
      pattern,
      cursor,
      this.principal,
      (event) => this.handleEvent(event),
    );
  }

  /**
   * Return the current preview state. UI bindings re-render off this
   * accessor (or off a registered observer; not in scope for the
   * canary surface).
   */
  public previewStateSnapshot(): PreviewState {
    return this.previewState;
  }

  /**
   * Alias used by the §3.6f-substantive test pin. Returns the same
   * snapshot the UI consumes.
   */
  public previewStateValue(): PreviewState {
    return this.previewState;
  }

  /**
   * Per-delta counter — useful in tests asserting that no further
   * deltas are applied after termination (T12 admin-UI handling).
   */
  public deltaCount(): number {
    return this.deltas;
  }

  /**
   * The literal subscribe-seam name the bridge forwards the creator's
   * subscription request through. Pinned by the
   * `admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`
   * grep-assert (the seam name appears verbatim in this method).
   */
  public subscriptionMode(): typeof ADMIN_UI_V0_SUBSCRIBE_SEAM {
    return ADMIN_UI_V0_SUBSCRIBE_SEAM;
  }

  /**
   * Stop the live preview. Idempotent; safe to call from `terminated`
   * or `error` states (transitions to `idle`).
   */
  public async stop(): Promise<void> {
    this.previewSubscription = null;
    this.previewState = { kind: "idle" };
  }

  // ---- private ----

  private handleEvent(event: unknown): void {
    // After termination, drop subsequent events. The state is
    // absorbing per T12 admin-UI handling.
    if (this.previewState.kind === "terminated") {
      return;
    }
    if (isTerminationSentinel(event)) {
      this.previewSubscription = null;
      this.previewState = {
        kind: "terminated",
        reason: event.reason,
      };
      return;
    }
    if (isErrorSentinel(event)) {
      this.previewSubscription = null;
      this.previewState = {
        kind: "error",
        message: event.message,
      };
      return;
    }
    // Treat any other event as a row delta. Live-preview accumulates
    // rows; production-side UX may evolve toward windowed
    // tail-truncation, but at the canary surface we keep the full
    // accumulated list.
    if (isRowDelta(event)) {
      this.deltas += 1;
      this.accumulator.push(event.row);
      this.previewState = {
        kind: "streaming",
        rows: [...this.accumulator],
        deltaCount: this.deltas,
      };
    }
  }
}

function subscribePatternFor(spec: UserViewSpec): string {
  switch (spec.labelPattern.kind) {
    case "exact":
      return `view:${spec.viewId}:${spec.labelPattern.label}`;
    case "anchor_prefix":
      return `view:${spec.viewId}:${spec.labelPattern.prefix}*`;
  }
}

function isTerminationSentinel(
  event: unknown,
): event is { readonly kind: "subscription_terminated"; readonly reason: string } {
  return (
    typeof event === "object" &&
    event !== null &&
    "kind" in event &&
    (event as { kind: unknown }).kind === "subscription_terminated" &&
    "reason" in event &&
    typeof (event as { reason: unknown }).reason === "string"
  );
}

function isErrorSentinel(
  event: unknown,
): event is { readonly kind: "error"; readonly message: string } {
  return (
    typeof event === "object" &&
    event !== null &&
    "kind" in event &&
    (event as { kind: unknown }).kind === "error" &&
    "message" in event &&
    typeof (event as { message: unknown }).message === "string"
  );
}

function isRowDelta(
  event: unknown,
): event is { readonly row: Record<string, unknown> } {
  return (
    typeof event === "object" &&
    event !== null &&
    "row" in event &&
    typeof (event as { row: unknown }).row === "object" &&
    (event as { row: unknown }).row !== null
  );
}
