// The TypeScript DSL — the developer-facing surface for composing
// operation subgraphs without hand-writing the Node / Edge graph.
//
// Shape rules:
//   * Every chain method returns the builder instance for fluent use
//     (`subgraph('x').read({...}).respond({...}).build()`).
//   * `.build()` returns a `Subgraph` — a JSON-serializable,
//     content-addressable shape ready for `engine.registerSubgraph()`.
//   * The four Phase 2 primitives (`wait`, `stream`, `subscribe`,
//     `sandbox`) build valid Subgraph Nodes but their executors error
//     at call time with `E_PRIMITIVE_NOT_IMPLEMENTED` (enforced in
//     Rust). The DSL does not gate them — the engine does.
//
// This module is pure — no runtime dependency on `@benten/engine-native`.
// That keeps `.toMermaid()` callable without loading the compiled
// binary, and it means typecheck/build/test can all run on a box where
// the napi artifact hasn't been built yet.

import { EDslInvalidShape } from "./errors.js";
import type {
  JsonValue,
  Primitive,
  SandboxArgs,
  SandboxArgsByCaps,
  SandboxArgsByName,
  Subgraph,
  SubgraphNode,
} from "./types.js";

// Re-export the SANDBOX argument shapes through the DSL surface so DSL
// callers can `import type { SandboxArgs } from "@benten/engine"` without
// reaching into the types module directly. The discriminated-union
// shape is the contract for `subgraph(...).sandbox(args)` per Phase 2b
// G7-C.
export type { SandboxArgs, SandboxArgsByCaps, SandboxArgsByName };

// ---------------------------------------------------------------------------
// Inv-14 attribution stamp (Phase 2a G11-A EVAL wave-1, D12.7 Decision 1)
// ---------------------------------------------------------------------------

/**
 * Property key on every DSL-emitted OperationNode that declares the node
 * consumes causal attribution (Inv-14). The Rust-side registration-time
 * validator expects `Value::Bool(true)` exactly (opt-out is Phase-6
 * scope); the DSL stamps this default so hand-built subgraphs that go
 * through the builder never hit `E_INV_ATTRIBUTION` at registration.
 * Mirrors
 * `benten_eval::invariants::attribution::ATTRIBUTION_PROPERTY_KEY`.
 */
export const ATTRIBUTION_PROPERTY_KEY = "attribution";

/** Return a copy of `args` with the Inv-14 attribution stamp applied. */
function stampAttribution(
  args: Record<string, JsonValue>,
): Record<string, JsonValue> {
  // Only stamp when absent — callers that explicitly set the property
  // (future Phase-6 opt-out callers, or tests that probe the reject
  // path) retain their declared value.
  if (args[ATTRIBUTION_PROPERTY_KEY] !== undefined) {
    return args;
  }
  return { ...args, [ATTRIBUTION_PROPERTY_KEY]: true };
}

// ---------------------------------------------------------------------------
// Primitive-specific argument shapes (public; consumed by the DSL methods)
// ---------------------------------------------------------------------------
//
// # Phase-3 G19-D §7.9 + r1-napi-3 / D-PHASE-3-29 surface-parity sweep
//
// Each *Args interface declares the user-facing surface; the DSL
// builder's primitive method (e.g. `subgraph(...).read(args)`) routes
// the args through a per-primitive `translateXxxArgs` helper that maps
// the DSL surface field names onto the eval-side primitive's actual
// keyspace (the keys that `crates/benten-eval/src/primitives/<p>.rs::execute`
// reads via `op.properties.get("...")`).
//
// This mirrors the WAIT precedent (PR #76 `translateWaitArgs`) + the
// SANDBOX precedent (G17-C wave-5b `translateSandboxArgs`). Pre-G19-D
// six pre-existing TS DSL Args drifts existed where the user-facing
// fields were spread verbatim into the OperationNode property bag and
// the eval-side primitive read DIFFERENT keys — silent value-loss /
// no-op routing for any DSL caller exercising the affected primitive
// end-to-end.
//
// The corresponding eval-side keyspaces (per
// `crates/benten-eval/src/primitives/*.rs::execute`):
//
//   ReadArgs       → `query_kind` / `target_cid` / `label`
//                    DSL surface: { label, by, value, as }
//   BranchArgs     → `match_value` / `condition_value` / `cases` /
//                    `has_default` / `conditions`
//                    DSL surface: { on }
//   IterateArgs    → `items` / `requires`
//                    DSL surface: { over, max }
//   TransformArgs  → `expr` / `input` / `result`
//                    DSL surface: { expr, as }
//   CallArgs       → `child_scope` / `parent_scope` / `target` /
//                    `call_op` / `requires` / `timeout_ms`
//                    DSL surface: { handler, action, input, isolated }
//   RespondArgs    → `status` / `body`
//                    DSL surface: { body, edge, status }
//
// A translator-output orphan (DSL field with no eval-side reader) OR a
// canonical-key orphan (eval-side reader with no DSL producer) is
// caught structurally by the LOAD-BEARING parity meta-test at
// `crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs`.
// See `translateReadArgs` / `translateBranchArgs` / `translateIterateArgs`
// / `translateTransformArgs` / `translateCallArgs` / `translateRespondArgs`
// below for the per-primitive translation contract.

export interface ReadArgs {
  /** Label to read from. Translates to eval-side `label` (verbatim). */
  label: string;
  /**
   * Lookup key. `"cid"` translates to eval-side `query_kind: "by_cid"`
   * + `target_cid: <value>` (the eval-side READ primitive's
   * by-CID query path); `"_listView"` translates to eval-side
   * `query_kind: "list_view"`; `"id"` is treated as the by-CID alias
   * for ergonomics. Other values pass through as `query_kind` verbatim
   * — the by-property-name path lights up in Phase-3+ when the
   * generalized READ keyspace expands.
   */
  by?: string;
  /**
   * Literal value to filter on (consumed by the corresponding `by`
   * mode). When `by === "cid"`, the value translates to eval-side
   * `target_cid: Text(<value>)`.
   */
  value?: JsonValue;
  /** Bind the READ result under this key on `$result` (DSL-side; not read by eval). */
  as?: string;
}

export interface WriteArgs {
  /** Label for the Node being written. Eval-side reads `label` verbatim. */
  label: string;
  /** Properties to write. Eval-side reads `properties` verbatim. */
  properties?: Record<string, JsonValue>;
  /** Optional `requires` capability (gates the WRITE at commit). */
  requires?: string;
}

export interface TransformArgs {
  /**
   * The TRANSFORM expression source (a subset of JS per
   * `docs/TRANSFORM-GRAMMAR.md`). Parsed at registration. Eval-side
   * reads `expr` verbatim.
   */
  expr: string;
  /**
   * Where to bind the result on `$result`. DSL-side bind hint; the
   * eval-side primitive reads `result` (the projection target key)
   * after evaluation — `as` translates to `result` per
   * `translateTransformArgs`.
   */
  as?: string;
}

export interface BranchArgs {
  /**
   * Expression over `$result` / `$input` to switch on. Translates to
   * eval-side `match_value: Text(<expr>)` (the canonical match-on-text
   * key the BRANCH primitive reads). Per-case routing is handled
   * separately by the builder via `.case(value, body)` calls which
   * stamp `CASE:<value>` outgoing edges; the `cases` / `has_default` /
   * `conditions` keys the eval-side primitive reads are populated by
   * the engine builder's compile path from the edge table (NOT spread
   * into the args bag here).
   */
  on: string;
}

export interface IterateArgs {
  /**
   * Source list expression. Translates to eval-side
   * `items: Text(<expr>)` — the iteration-over-list key the eval-side
   * ITERATE primitive reads.
   */
  over: string;
  /**
   * Max iteration count (required — invariant 9). Translates to
   * eval-side `max` (verbatim — Inv-9 budget is enforced from the
   * `max` property on the OperationNode at evaluator setup time).
   */
  max: number;
}

export interface CallArgs {
  /**
   * Handler id to CALL. Translates to eval-side `target: Text(<id>)`
   * (the canonical target-handler key the CALL primitive's dispatch
   * path reads).
   */
  handler: string;
  /**
   * Action on the target handler (e.g. `"post:get"`). Translates to
   * eval-side `call_op: Text(<action>)` (the action-name key the CALL
   * primitive reads when dispatching to the named handler).
   */
  action?: string;
  /**
   * Input expression bound to the callee's `$input`. DSL-side bind
   * hint; preserved verbatim into the args bag for the engine's
   * compile path.
   */
  input?: string;
  /**
   * If `true`, the CALL enters an isolated capability scope and cannot
   * delegate parent caps (default `false`). Translates to eval-side
   * `child_scope: Bool(true)` when set; absent (`undefined` / `false`)
   * produces no `child_scope` key (the eval-side CALL primitive's
   * default scope-inheritance path applies).
   */
  isolated?: boolean;
}

export interface RespondArgs {
  /**
   * Response body expression. Eval-side reads `body` verbatim
   * (`primitives/respond.rs::execute` line 41).
   */
  body?: string;
  /**
   * Optional typed error edge to route through (e.g. `"ON_NOT_FOUND"`).
   * DSL-side routing hint; surfaces on the OperationNode's outgoing
   * edge table (NOT spread into the args bag — the BRANCH/RESPOND
   * routing is edge-driven, not properties-driven, per the engine's
   * compile path).
   */
  edge?: string;
  /**
   * Optional status code override (HTTP mapping — not enforced in
   * Phase 1). Eval-side reads `status` verbatim
   * (`primitives/respond.rs::execute` line 38).
   */
  status?: number;
}

export interface EmitArgs {
  /** Event label. */
  event: string;
  /** Event payload expression. */
  payload?: string;
}

// Phase 2a G3-B (dx-r1-8): WAIT has two variants — a signal-keyed form
// (`wait({ signal, signal_shape? })`) and the Phase-1 timed form
// (`wait({ duration })`). Exactly one of `signal` / `duration` must be
// present; the builder throws `EDslInvalidShape` at `.wait()` time if
// neither is supplied. When `signal` is present, an optional
// `signal_shape` (a TRANSFORM-style schema string) constrains the
// resume-time payload. See docs/DSL-SPECIFICATION.md §2.6.
export type WaitArgs = WaitSignalArgs | WaitDurationArgs;

export interface WaitSignalArgs {
  /** Signal name the WAIT suspends on (e.g. `"external:payment"`). */
  signal: string;
  /**
   * Optional schema string constraining the resume-time payload. If
   * omitted (default) any Value is accepted; if set, a resume with a
   * mismatched payload fires `E_WAIT_SIGNAL_SHAPE_MISMATCH` before any
   * downstream primitive executes.
   */
  signal_shape?: string;
  /** Optional deadline — if the signal does not arrive in time, `E_WAIT_TIMEOUT` fires. */
  duration?: string;
}

export interface WaitDurationArgs {
  /** Duration string (e.g. `"5m"`, `"30s"`, `"2h"`). */
  duration: string;
  /** Signal-keyed form never co-occurs with the bare-duration form. */
  signal?: never;
  signal_shape?: never;
}

// ---------------------------------------------------------------------------
// WAIT duration-string parser (R6-R5 r6-r5-pcds-2 — 23rd producer/consumer
// drift fix-pass)
// ---------------------------------------------------------------------------

/**
 * Parse a duration string of the form `<N>(s|m|h)` into integer
 * milliseconds. Mirrors the DSL contract documented at
 * `WaitDurationArgs.duration` ("e.g. `5m`, `30s`, `2h`") + the joined
 * `WaitSignalArgs.duration` deadline form.
 *
 * Pre-R6-R5 the DSL spread wrote the raw `duration: "5m"` (Text)
 * property into the OperationNode args bag, but the eval-side
 * `wait::evaluate_op_with_handler_id` reader at
 * `crates/benten-eval/src/primitives/wait.rs::evaluate_op_with_handler_id`
 * reads `duration_ms` (Int) — there was NO translation layer. A DSL-built
 * `wait({ duration: "5m" })` therefore suspended without a deadline +
 * never auto-resumed. The R6-R5 producer/consumer deep-sweep
 * (`r6-r5-pcds-2`) closed this by translating at the DSL spread before
 * the `addNode("wait", ...)` call. See the deep-sweep report at
 * `.addl/phase-2b/r6-r5-producer-consumer-deep-sweep.json`.
 *
 * Throws `EDslInvalidShape` (mapped to `E_DSL_INVALID_SHAPE`) for any
 * other form (mirroring the existing `.wait()` validation at
 * `SubgraphBuilder.wait` / `CaseBuilder.wait` which already throws on
 * empty wait shapes).
 */
function parseDurationToMs(s: string): number {
  if (typeof s !== "string" || s.length === 0) {
    throw new EDslInvalidShape(
      "wait({ duration }) requires a non-empty string of the form `<N>(s|m|h)` (E_DSL_INVALID_SHAPE)",
    );
  }
  const match = /^(\d+)(s|m|h)$/.exec(s);
  if (!match) {
    throw new EDslInvalidShape(
      `wait({ duration: "${s}" }) — duration must match \`<N>(s|m|h)\` (e.g. "5s", "30m", "2h"); E_DSL_INVALID_SHAPE`,
    );
  }
  const n = parseInt(match[1]!, 10);
  if (!Number.isFinite(n) || n <= 0) {
    throw new EDslInvalidShape(
      `wait({ duration: "${s}" }) — magnitude must be a positive integer; E_DSL_INVALID_SHAPE`,
    );
  }
  const unit = match[2]!;
  const multiplier = unit === "s" ? 1_000 : unit === "m" ? 60_000 : 3_600_000;
  return n * multiplier;
}

/**
 * Translate the public `WaitArgs` shape (`signal` / `duration` / `signal_shape`)
 * into the OperationNode property bag the eval-side `wait` primitive
 * actually reads (`signal: Text` / `duration_ms: Int` / `timeout_ms: Int`
 * / `signal_shape: Value`). Mirrors the EMIT precedent (`channel: args.event`)
 * and SUBSCRIBE precedent (`pattern: args.event`) — translation lives at
 * the DSL spread so the eval-side reader sees its canonical key shape.
 *
 * Branching rules per `WaitSignalArgs` / `WaitDurationArgs`:
 *   - signal-only form → `{ signal: <s> }`
 *   - duration-only form → `{ duration_ms: parseDurationToMs(d) }`
 *   - signal-with-duration form → `{ signal: <s>, timeout_ms: parseDurationToMs(d) }`
 *     (per `WaitSignalArgs.duration` JSDoc: "Optional deadline — if the
 *     signal does not arrive in time, `E_WAIT_TIMEOUT` fires.")
 *   - signal_shape (when present) is forwarded verbatim.
 *
 * The empty-args + neither-set rejection happens at the public
 * `.wait()` builder boundary BEFORE this function fires; this helper
 * assumes at least one of `signal` / `duration` is set.
 */
function translateWaitArgs(
  args: WaitArgs,
): Record<string, JsonValue> {
  const a = args as {
    signal?: string;
    signal_shape?: string;
    duration?: string;
  };
  const props: Record<string, JsonValue> = {};
  if (typeof a.signal === "string" && a.signal.length > 0) {
    props.signal = a.signal;
    if (typeof a.duration === "string") {
      // Signal-with-deadline form — duration translates to `timeout_ms`
      // per the eval-side reader's signal-variant deadline semantics.
      props.timeout_ms = parseDurationToMs(a.duration);
    }
  } else if (typeof a.duration === "string") {
    // Bare-duration form — duration translates to `duration_ms` so the
    // eval-side suspension store stamps `WaitMetadata.is_duration = true`.
    props.duration_ms = parseDurationToMs(a.duration);
  }
  if (typeof a.signal_shape === "string") {
    props.signal_shape = a.signal_shape;
  }
  return props;
}
export interface StreamArgs {
  source: string;
  /** Optional chunk-size hint. */
  chunkSize?: number;
}
export interface SubscribeArgs {
  /**
   * Event/pattern the SUBSCRIBE matches. DSL surface name retained for
   * developer ergonomics; translates to eval-side
   * `pattern: Text(<event>)` per the SUBSCRIBE primitive's match path
   * (`crates/benten-eval/src/primitives/subscribe.rs::execute` line 1282).
   */
  event: string;
  /**
   * Phase-3 G19-D §7.10 + §7.9 + r1-napi-3 (D-PHASE-3-29):
   * **SubscribeArgs.handler RE-INTRODUCED** post-Phase-2b removal. The
   * eval-side handler-id-router seam was wired in G14-D wave-5a per
   * seq-major-8 (`crates/benten-eval/src/primitives/subscribe.rs::execute`
   * lines 1295-1317); G19-D wave-7 restores the corresponding TS DSL
   * surface field that PR #75's R6-R4-narrow fix-pass had to drop
   * pending the eval-side wiring.
   *
   * When set, the SUBSCRIBE primitive routes change-event delivery
   * THROUGH the named handler instead of the default broadcast
   * fan-out. Translates to eval-side `handler: Text(<id>)` per the
   * G14-D handler-id-router seam. See `docs/DSL-SPECIFICATION.md`
   * worked example for the handler-id-router routing model.
   *
   * Closes the 21st producer/consumer drift (R6-R4-narrow-pcds-1)
   * loop: pre-G14-D the field was a phantom (TS DSL produced it; eval
   * never read it); post-G14-D + post-G19-D the field is wired
   * end-to-end with the LOAD-BEARING parity meta-test asserting no
   * orphan reads/writes on either side.
   */
  handler?: string;
}
/**
 * Phase-3 G17-C wave-5b (phase-3-backlog §6.6 — 24th p/c drift
 * acceptance criterion; pim-2 LOAD-BEARING). Translate the user-facing
 * camelCase [`SandboxArgs`] to the snake_case property bag the
 * eval-side primitive at
 * `crates/benten-engine/src/primitive_host.rs::execute_sandbox` reads.
 *
 * Mirrors the `translateWaitArgs` precedent (PR #76) where a similar
 * casing drift between DSL surface + eval reader caused signal-with-
 * deadline WAIT calls to silently misroute. The 24th p/c drift named
 * the same shape for SANDBOX:
 *
 *   - DSL surface: `wallclockMs`, `outputLimitBytes` (camelCase, with
 *     `Bytes` for type-clarity at the user-facing surface).
 *   - Eval-side reader: `wallclock_ms`, `output_limit` (NOTE: drops
 *     `Bytes` — symmetric with `wallclock_ms` not carrying
 *     `_milliseconds`; r4-r1-wsa-1 BLOCKER recalibration verified
 *     against `primitive_host.rs::execute_sandbox` which reads
 *     `op.properties.get("output_limit")`).
 *   - `module`, `caps`, `input`, `fuel` translate verbatim (already
 *     match the eval-side reader's keys).
 *
 * The translation is applied at the SubgraphBuilder.sandbox() boundary
 * (both the public + internal builder variants) so every SANDBOX node
 * authored via the DSL crosses the napi boundary with snake_case keys.
 * A regression that drops a translation site (or omits a new arg from
 * the translator) is caught by the load-bearing eval-side end-to-end
 * pin at
 * `crates/benten-eval/tests/sandbox_handler_args.rs::sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
 * + the TS-side meta-pin at
 * `packages/engine/test/sandbox_handler_args.test.ts`.
 */
function translateSandboxArgs(
  args: SandboxArgs,
): Record<string, JsonValue> {
  // Treat `args` as an open-shape record so the translator handles both
  // discriminants (by-name vs by-caps) without re-narrowing.
  const a = args as {
    module?: string;
    input?: string;
    fuel?: number;
    wallclockMs?: number;
    outputLimitBytes?: number;
    caps?: readonly string[];
  };
  const props: Record<string, JsonValue> = {};
  if (typeof a.module === "string") {
    props.module = a.module;
  }
  if (typeof a.input === "string") {
    props.input = a.input;
  }
  if (typeof a.fuel === "number") {
    // `fuel` is the canonical eval-side key (no token to translate);
    // primitive_host.rs::execute_sandbox reads `op.properties.get("fuel")`.
    props.fuel = a.fuel;
  }
  if (typeof a.wallclockMs === "number") {
    // wallclockMs (camelCase, DSL) → wallclock_ms (snake_case, eval-side).
    // primitive_host.rs::execute_sandbox reads `op.properties.get("wallclock_ms")`.
    props.wallclock_ms = a.wallclockMs;
  }
  if (typeof a.outputLimitBytes === "number") {
    // outputLimitBytes (camelCase, DSL — preserves `Bytes` for type-
    // clarity) → output_limit (snake_case, eval-side — DROPS `Bytes`
    // per r4-r1-wsa-1 verification against primitive_host.rs::execute_sandbox which
    // reads `op.properties.get("output_limit")`). Symmetric with
    // `wallclock_ms` not carrying `_milliseconds`.
    props.output_limit = a.outputLimitBytes;
  }
  if (Array.isArray(a.caps)) {
    // by-caps escape hatch — caps key is canonical eval-side.
    props.caps = a.caps as unknown as JsonValue;
  }
  return props;
}

// ---------------------------------------------------------------------------
// Phase-3 G19-D §7.9 + r1-napi-3 — per-primitive Args→eval-keyspace translators
// ---------------------------------------------------------------------------
//
// Each translator mirrors the WAIT (PR #76 `translateWaitArgs`) +
// SANDBOX (G17-C wave-5b `translateSandboxArgs`) precedents: the DSL
// surface is the user-facing field-name vocabulary; the eval-side
// primitive at `crates/benten-eval/src/primitives/<p>.rs::execute`
// reads its OWN canonical key vocabulary. The translator bridges the
// two so a regression that drops a translation site (or omits a new
// field from the translator) is caught by the LOAD-BEARING
// `dsl_args_vs_eval_properties_parity_meta_test` at structural layer.
//
// Translation discipline (per pim-11 §3.6d + §3.6 consumer-audit):
//   - The translator's OUTPUT keyspace MUST be a subset of the
//     eval-side primitive's actual `op.properties.get("...")` reads.
//   - Every eval-side reader MUST have a translator producer (catches
//     orphan-reader shape: a key the eval reads with no DSL producer).
//   - Field-by-field translation, NOT a verbatim spread (catches
//     phantom-field shape: a DSL field the eval never reads).

/**
 * G19-D §7.9 — translate `ReadArgs` to the eval-side READ primitive's
 * keyspace per `crates/benten-eval/src/primitives/read.rs::execute`.
 *
 *   DSL surface          → eval-side keyspace
 *   {label}              → label: Text(<label>)
 *   {by: "cid"}          → query_kind: Text("by_cid")
 *   {by: "id"}           → query_kind: Text("by_cid") (id is alias)
 *   {by: "_listView"}    → query_kind: Text("list_view")
 *   {by: "<other>"}      → query_kind: Text("<other>") (passthrough)
 *   {value: <v>}         → target_cid: Text(<v>) (when by === "cid"/"id")
 *
 * `as` is a DSL-side bind hint NOT spread to the eval side (the eval
 * READ primitive does not project on a bind alias; the engine compile
 * path consumes `as` upstream of property-bag construction when
 * relevant).
 */
function translateReadArgs(args: ReadArgs): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.label === "string") {
    props.label = args.label;
  }
  if (typeof args.by === "string") {
    if (args.by === "cid" || args.by === "id") {
      props.query_kind = "by_cid";
      if (args.value !== undefined) {
        // The eval-side reader at primitives/read.rs lines 52-57
        // accepts `target_cid` as Bytes OR Text; the TS DSL spread
        // produces Text — the napi `json_to_props` round-trip
        // preserves it as Value::Text which the eval-side fallback
        // arm handles.
        props.target_cid = args.value as JsonValue;
      }
    } else if (args.by === "_listView") {
      props.query_kind = "list_view";
    } else {
      // Passthrough — Phase-3+ widening of the READ primitive's
      // by-property path will read additional `query_kind` discriminants;
      // the meta-test would fire if the discriminant has no eval-side
      // reader.
      props.query_kind = args.by;
      if (args.value !== undefined) {
        props.target_cid = args.value as JsonValue;
      }
    }
  }
  return props;
}

/**
 * G19-D §7.9 — translate `BranchArgs` to the eval-side BRANCH
 * primitive's keyspace per `primitives/branch.rs::execute`.
 *
 *   DSL surface → eval-side keyspace
 *   {on}        → match_value: Text(<on-expr>)
 *
 * The `cases` / `has_default` / `conditions` keys the eval-side
 * primitive reads (lines 53 / 66 / 102) are populated by the engine
 * compile path from the BRANCH node's outgoing edge table (`CASE:<v>`
 * labels), NOT spread into the args bag from the DSL surface. The DSL
 * builder's `.case(value, body)` calls stamp those edges; the engine
 * compile path consumes them and emits the per-case keyspace as
 * needed.
 */
function translateBranchArgs(args: BranchArgs): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.on === "string") {
    props.match_value = args.on;
  }
  return props;
}

/**
 * G19-D §7.9 — translate `IterateArgs` to the eval-side ITERATE
 * primitive's keyspace per `primitives/iterate.rs::execute`.
 *
 *   DSL surface  → eval-side keyspace
 *   {over}       → items: Text(<over-expr>)
 *   {max}        → max: Int(<max>) (Inv-9 budget — verbatim key)
 *
 * The eval-side `requires` key (line 94) is populated by the engine
 * compile path from a separate capability declaration when relevant;
 * NOT spread from the DSL surface here.
 */
function translateIterateArgs(args: IterateArgs): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.over === "string") {
    props.items = args.over;
  }
  if (typeof args.max === "number") {
    // `max` is canonical eval-side key — Inv-9 budget enforced from
    // the OperationNode property bag at evaluator setup time.
    props.max = args.max;
  }
  return props;
}

/**
 * G19-D §7.9 — translate `TransformArgs` to the eval-side TRANSFORM
 * primitive's keyspace per `primitives/transform.rs::execute`.
 *
 *   DSL surface → eval-side keyspace
 *   {expr}      → expr: Text(<expr>) (verbatim — canonical key)
 *   {as}        → result: Text(<bind-key>) (projection-target key)
 *
 * The eval-side `input` key (line 61) is populated by the engine
 * compile path when the TRANSFORM has an upstream binding; NOT spread
 * from the DSL surface here.
 */
function translateTransformArgs(
  args: TransformArgs,
): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.expr === "string") {
    props.expr = args.expr;
  }
  if (typeof args.as === "string") {
    props.result = args.as;
  }
  return props;
}

/**
 * G19-D §7.9 — translate `CallArgs` to the eval-side CALL primitive's
 * keyspace per `primitives/call.rs::execute`.
 *
 *   DSL surface       → eval-side keyspace
 *   {handler}         → target: Text(<handler-id>)
 *   {action}          → call_op: Text(<action>)
 *   {input}           → input: Text(<input-expr>) (DSL-side bind hint
 *                       preserved verbatim for the engine compile path)
 *   {isolated: true}  → child_scope: Bool(true)
 *
 * The eval-side `parent_scope` / `requires` / `timeout_ms` /
 * `elapsed_ms` keys (lines 82 / 66 / 100 / 101) are populated by the
 * engine compile path from the surrounding CALL frame, NOT spread from
 * the DSL surface. (Per CallArgs JSDoc above: timeout/scope-inheritance
 * is engine-driven, not DSL-surface-driven.)
 */
function translateCallArgs(args: CallArgs): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.handler === "string") {
    props.target = args.handler;
  }
  if (typeof args.action === "string") {
    props.call_op = args.action;
  }
  if (typeof args.input === "string") {
    // Preserve DSL-side bind hint; engine compile path consumes.
    props.input = args.input;
  }
  if (args.isolated === true) {
    // Set `child_scope` only when truthy — the eval-side path treats
    // absence as "inherit parent scope" (the default behavior).
    props.child_scope = true;
  }
  return props;
}

/**
 * G19-D §7.9 — translate `RespondArgs` to the eval-side RESPOND
 * primitive's keyspace per `primitives/respond.rs::execute`.
 *
 *   DSL surface → eval-side keyspace
 *   {body}      → body: Text(<body-expr>) (verbatim — line 41)
 *   {status}    → status: Int(<status>) (verbatim — line 38)
 *   {edge}      → (NOT spread; routing is edge-driven via
 *                  the OperationNode's outgoing edge table — the
 *                  engine compile path consumes the `edge` hint to
 *                  stamp the appropriate routing edge label).
 *
 * RespondArgs is the closest-to-no-drift Args interface — `body` +
 * `status` translate verbatim. The `edge` hint is by-design omitted
 * from the property bag (its surface lives on the edge table).
 */
function translateRespondArgs(args: RespondArgs): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.body === "string") {
    props.body = args.body;
  }
  if (typeof args.status === "number") {
    props.status = args.status;
  }
  return props;
}

/**
 * G19-D §7.10 — translate `SubscribeArgs` to the eval-side SUBSCRIBE
 * primitive's keyspace per `primitives/subscribe.rs::execute`.
 *
 *   DSL surface → eval-side keyspace
 *   {event}     → pattern: Text(<event>) (line 1282 — match path)
 *   {handler}   → handler: Text(<handler-id>) (lines 1295-1317 —
 *                 handler-id-router seam wired in G14-D wave-5a per
 *                 seq-major-8; routes change-event delivery through
 *                 the named handler instead of default fan-out)
 *
 * Mirrors the EMIT translation precedent — both primitives carry an
 * optional `handler` field in their respective Args interfaces, and
 * both eval primitives read the same key (`primitives/emit.rs::execute`
 * lines 61-77 + `primitives/subscribe.rs::execute` lines 1295-1317).
 * The G19-D §7.10 worked example in `docs/DSL-SPECIFICATION.md` shows
 * the handler-id-router routing model end-to-end.
 */
function translateSubscribeArgs(
  args: SubscribeArgs,
): Record<string, JsonValue> {
  const props: Record<string, JsonValue> = {};
  if (typeof args.event === "string") {
    props.pattern = args.event;
  }
  if (typeof args.handler === "string") {
    props.handler = args.handler;
  }
  return props;
}

// SandboxArgs is defined in `./types.ts` as the discriminated union
// `SandboxArgsByName | SandboxArgsByCaps` (Phase 2b G7-C). Imported and
// re-exported above so DSL callers see one canonical shape.
//
// Per dx-r1-2b SANDBOX surface: a SANDBOX node is composed via
// `subgraph(...).sandbox(args)`. There is no top-level
// `engine.sandbox(...)` — that would bypass the evaluator + Inv-4 nest
// depth + Inv-14 attribution chaining + capability resolution. The
// composition-only contract is pinned by `packages/engine/test/sandbox.test.ts`.

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/**
 * SubgraphBuilder — fluent, append-only DSL over the 12 primitives.
 *
 * One instance represents one Subgraph under construction. Call
 * `.build()` to materialize the JSON-serializable shape.
 *
 * Node ids are counted per-instance so concurrent builders produce
 * independent, stable id sequences — two `crud('post')` calls in
 * parallel Vitest workers both assign `read-1`, `write-2`, etc. and
 * therefore yield identical content-addressed handler CIDs.
 */
export class SubgraphBuilder {
  protected readonly handlerId: string;
  protected readonly nodes: SubgraphNode[] = [];
  protected rootId?: string;
  protected lastId?: string;
  protected readonly actions: Set<string> = new Set();
  // Per-instance node counter. See class-level JSDoc above.
  protected nodeCounter = 0;

  public constructor(handlerId: string) {
    if (typeof handlerId !== "string" || handlerId.length === 0) {
      throw new EDslInvalidShape("handlerId must be a non-empty string");
    }
    this.handlerId = handlerId;
  }

  protected nextNodeId(prefix: string): string {
    this.nodeCounter = (this.nodeCounter + 1) | 0;
    return `${prefix}-${this.nodeCounter}`;
  }

  protected addNode(
    primitive: Primitive,
    args: Record<string, JsonValue>,
  ): this {
    const id = this.nextNodeId(primitive);
    // Phase 2a G11-A EVAL wave-1 (D12.7 Decision 1): every OperationNode
    // the DSL emits declares `attribution: true` by default so the
    // Rust-side Inv-14 registration-time validator does not reject a
    // DSL-built subgraph. Callers that want to opt out (Phase-6 extension
    // point) must bypass the DSL entirely. The stamp lives under `args`
    // so it rides through the napi wire shape into the engine's
    // OperationNode property bag without a dedicated parallel channel.
    const stampedArgs = stampAttribution(args);
    const node: SubgraphNode = {
      id,
      primitive,
      args: stampedArgs,
      edges: {},
    };
    // Link previous node via a default `NEXT` edge (unless the previous
    // was a BRANCH — its edges are managed by `.case()`).
    if (this.lastId) {
      const prev = this.nodes.find((n) => n.id === this.lastId);
      if (prev && prev.primitive !== "branch") {
        prev.edges.NEXT = id;
      }
    }
    this.nodes.push(node);
    this.rootId ??= id;
    this.lastId = id;
    return this;
  }

  public read(args: ReadArgs): this {
    // G19-D §7.9: translate DSL surface → eval-side READ keyspace
    // (`label` / `query_kind` / `target_cid` per primitives/read.rs::execute).
    return this.addNode("read", translateReadArgs(args));
  }
  public write(args: WriteArgs): this {
    // WRITE: DSL surface ALREADY matches eval keyspace (label / properties / requires).
    // Spread verbatim; no translation step needed (the engine compile path
    // owns the WriteSpec extraction at `bindings/napi/src/subgraph.rs::extract_write_args`).
    return this.addNode("write", { ...args } as Record<string, JsonValue>);
  }
  public transform(args: TransformArgs): this {
    // G19-D §7.9: translate DSL surface → eval-side TRANSFORM keyspace
    // (`expr` / `result` per primitives/transform.rs::execute).
    return this.addNode("transform", translateTransformArgs(args));
  }
  public iterate(args: IterateArgs): this {
    if (typeof args.max !== "number" || args.max <= 0) {
      throw new EDslInvalidShape(
        "iterate requires a positive integer `max` (invariant E_INV_ITERATE_MAX_MISSING)",
      );
    }
    // G19-D §7.9: translate DSL surface → eval-side ITERATE keyspace
    // (`items` / `max` per primitives/iterate.rs::execute).
    return this.addNode("iterate", translateIterateArgs(args));
  }
  public call(args: CallArgs): this {
    // G19-D §7.9: translate DSL surface → eval-side CALL keyspace
    // (`target` / `call_op` / `input` / `child_scope` per primitives/call.rs::execute).
    return this.addNode("call", translateCallArgs(args));
  }
  public respond(args: RespondArgs = {}): this {
    // G19-D §7.9: translate DSL surface → eval-side RESPOND keyspace
    // (`body` / `status` per primitives/respond.rs::execute; `edge` is
    // edge-table routing, not properties-bag).
    return this.addNode("respond", translateRespondArgs(args));
  }
  public emit(args: EmitArgs): this {
    // R6 Round-2 r6-r2-mpc-1 fix-pass: the eval-side EMIT executor at
    // `crates/benten-eval/src/primitives/emit.rs:41` reads the `channel`
    // property; the public DSL `EmitArgs.event` field maps onto that
    // property name. Pre-fix the spread set `event: ...` and the EMIT
    // primitive silently dropped the publish (no `channel` property →
    // `host.emit_event(...)` never fired).
    const props: Record<string, JsonValue> = { channel: args.event };
    if (args.payload !== undefined) {
      props.payload = args.payload;
    }
    return this.addNode("emit", props);
  }

  // Phase 2a G3-B (dx-r1-8): WAIT accepts either a signal-keyed form or the
  // Phase-1 timed form; exactly one of `signal` / `duration` must be set.
  // R6-R5 r6-r5-pcds-2 fix-pass (23rd producer/consumer drift): the
  // eval-side primitive at `wait::evaluate_op_with_handler_id` reads
  // `duration_ms: Int` (NOT `duration: Text`) + `timeout_ms: Int` for
  // the signal-with-deadline form. Pre-fix the spread wrote the raw
  // `duration: "5m"` string verbatim and the duration-variant WAIT
  // suspended forever (no `WaitMetadata.is_duration` stamped, no
  // `timeout_ms` deadline). Translate at the DSL spread per
  // `translateWaitArgs` (mirrors EMIT/SUBSCRIBE translation precedents).
  public wait(args: WaitArgs): this {
    const a = args as { signal?: string; duration?: string };
    if (!a || (a.signal == null && a.duration == null)) {
      throw new EDslInvalidShape(
        "wait(args) requires either `signal: string` or `duration: string` (E_DSL_INVALID_SHAPE)",
      );
    }
    return this.addNode("wait", translateWaitArgs(args));
  }
  public stream(args: StreamArgs): this {
    return this.addNode("stream", { ...args } as Record<string, JsonValue>);
  }
  public subscribe(args: SubscribeArgs): this {
    // G19-D §7.10: translate DSL surface → eval-side SUBSCRIBE keyspace
    // (`pattern` / `handler` per primitives/subscribe.rs::execute). The
    // `handler?` field is RE-INTRODUCED post-G14-D handler-id-router
    // wiring; closes 21st p/c drift end-to-end at structural layer.
    // Mirrors the WAIT/SANDBOX/EMIT translation precedents.
    return this.addNode("subscribe", translateSubscribeArgs(args));
  }
  public sandbox(args: SandboxArgs): this {
    // Phase-3 G17-C wave-5b (24th p/c drift acceptance criterion; pim-2
    // LOAD-BEARING): translate camelCase DSL args to snake_case eval-side
    // keys via translateSandboxArgs. Mirrors the WAIT/EMIT/SUBSCRIBE
    // translation precedents above.
    return this.addNode("sandbox", translateSandboxArgs(args));
  }

  /**
   * Open a BRANCH switching on the expression supplied in `args.on`.
   * Case bodies are supplied via `.case(value, s => s.respond(...))`.
   * Each case sub-builder receives a fresh scope; returned sub-nodes
   * are attached to the BRANCH via an edge labeled `CASE:<value>`.
   */
  public branch(args: BranchArgs): BranchBuilder {
    // G19-D §7.9: translate DSL surface → eval-side BRANCH keyspace
    // (`match_value` per primitives/branch.rs::execute line 49). The
    // `cases` / `has_default` / `conditions` keys are populated by the
    // engine compile path from the BRANCH node's outgoing edges
    // (`CASE:<value>` labels stamped by `.case(value, body)` below).
    this.addNode("branch", translateBranchArgs(args));
    const branchNodeId = this.lastId!;
    return new BranchBuilder(this, branchNodeId);
  }

  /** Declare an action string (e.g. `"post:create"`) exposed by this handler. */
  public action(name: string): this {
    if (!name || typeof name !== "string") {
      throw new EDslInvalidShape("action name must be a non-empty string");
    }
    this.actions.add(name);
    return this;
  }

  /** Materialize the finished Subgraph. */
  public build(): Subgraph {
    if (!this.rootId) {
      throw new EDslInvalidShape(
        `subgraph '${this.handlerId}' has no nodes — add at least one primitive before calling .build()`,
      );
    }
    return {
      handlerId: this.handlerId,
      actions: [...this.actions],
      nodes: this.nodes.map(cloneNode),
      root: this.rootId,
    };
  }
}

function cloneNode(n: SubgraphNode): SubgraphNode {
  return {
    id: n.id,
    primitive: n.primitive,
    args: { ...n.args },
    edges: { ...n.edges },
  };
}

/**
 * Sub-builder for BRANCH cases. Each `.case()` spins a small case
 * scope — the scope's nodes are folded back into the parent subgraph's
 * node list when the case closes, and the BRANCH's edge table gets
 * a `CASE:<value>` entry pointing at the case-root.
 */
export class BranchBuilder {
  private readonly parent: SubgraphBuilder;
  private readonly branchNodeId: string;

  public constructor(parent: SubgraphBuilder, branchNodeId: string) {
    this.parent = parent;
    this.branchNodeId = branchNodeId;
  }

  /**
   * Add a case. `body` is a function that receives a scoped builder;
   * whatever primitives it adds get linked under a `CASE:<value>` edge
   * off the parent BRANCH Node.
   */
  public case(
    value: string,
    body: (s: CaseBuilder) => unknown,
  ): BranchBuilder {
    const parentInternal = this.parent as unknown as InternalParent;
    const scope = new CaseBuilder(parentInternal.handlerIdInternal(), (p) =>
      parentInternal.mintNodeIdInternal(p),
    );
    body(scope);
    const caseNodes = scope.drain();
    if (caseNodes.length === 0) {
      throw new EDslInvalidShape(
        `branch.case('${value}') body must add at least one primitive`,
      );
    }
    // Merge case nodes into the parent.
    const ownerNodes = (this.parent as unknown as InternalParent).nodesInternal();
    const branchNode = ownerNodes.find((n) => n.id === this.branchNodeId);
    if (!branchNode) {
      throw new EDslInvalidShape("internal: branch node vanished from parent");
    }
    branchNode.edges[`CASE:${value}`] = caseNodes[0].id;
    ownerNodes.push(...caseNodes);
    return this;
  }

  /** Close the branch and return control to the parent for further chaining. */
  public endBranch(): SubgraphBuilder {
    return this.parent;
  }

  /** Convenience — final step in a chain: build parent directly. */
  public build(): Subgraph {
    return this.parent.build();
  }
}

// Private helper interface — exposes internals to BranchBuilder without
// making them public on SubgraphBuilder's API surface.
interface InternalParent {
  handlerIdInternal(): string;
  nodesInternal(): SubgraphNode[];
  mintNodeIdInternal(prefix: string): string;
}

// Add internals lazily via prototype extension (keeps the public class API clean).
Object.defineProperty(SubgraphBuilder.prototype, "handlerIdInternal", {
  value(this: SubgraphBuilder): string {
    return (this as unknown as { handlerId: string }).handlerId;
  },
  enumerable: false,
});
Object.defineProperty(SubgraphBuilder.prototype, "nodesInternal", {
  value(this: SubgraphBuilder): SubgraphNode[] {
    return (this as unknown as { nodes: SubgraphNode[] }).nodes;
  },
  enumerable: false,
});
Object.defineProperty(SubgraphBuilder.prototype, "mintNodeIdInternal", {
  value(this: SubgraphBuilder, prefix: string): string {
    // Delegate to the instance method so the per-instance counter is
    // shared with `CaseBuilder`s spawned off this SubgraphBuilder.
    return (this as unknown as { nextNodeId(p: string): string }).nextNodeId(
      prefix,
    );
  },
  enumerable: false,
});

/** Lightweight builder used inside a `.case()` body. */
export class CaseBuilder {
  private readonly scopeNodes: SubgraphNode[] = [];
  private lastId?: string;

  // Parent handler id + an id-minting closure threaded from the owning
  // SubgraphBuilder. Using the parent's per-instance counter keeps node
  // ids deterministic across concurrent builder instances (each chain
  // gets `read-1`, `write-2`, ... from its own counter).
  public constructor(
    public readonly handlerId: string,
    private readonly mintId: (prefix: string) => string = (p) =>
      `${p}-${++CaseBuilder.__fallbackCounter}`,
  ) {}

  // Fallback counter used only when a CaseBuilder is constructed
  // without a parent id-minter (legacy callers / direct `new
  // CaseBuilder(id)` sites). New code routes through the parent's
  // counter via the constructor's second argument.
  private static __fallbackCounter = 0;

  private addNode(
    primitive: Primitive,
    args: Record<string, JsonValue>,
  ): this {
    const id = this.mintId(primitive);
    // See SubgraphBuilder.addNode for Inv-14 attribution stamp rationale.
    const stampedArgs = stampAttribution(args);
    const node: SubgraphNode = { id, primitive, args: stampedArgs, edges: {} };
    if (this.lastId) {
      const prev = this.scopeNodes.find((n) => n.id === this.lastId);
      if (prev && prev.primitive !== "branch") {
        prev.edges.NEXT = id;
      }
    }
    this.scopeNodes.push(node);
    this.lastId = id;
    return this;
  }

  public read(a: ReadArgs): this {
    // G19-D §7.9 — see SubgraphBuilder.read() for translation rationale.
    // Both builders MUST stay in lockstep on the spread shape.
    return this.addNode("read", translateReadArgs(a));
  }
  public write(a: WriteArgs): this {
    // WRITE: DSL surface matches eval keyspace verbatim. See SubgraphBuilder.write().
    return this.addNode("write", { ...a } as Record<string, JsonValue>);
  }
  public transform(a: TransformArgs): this {
    // G19-D §7.9 — see SubgraphBuilder.transform() for translation rationale.
    return this.addNode("transform", translateTransformArgs(a));
  }
  public iterate(a: IterateArgs): this {
    // G19-D §7.9 — see SubgraphBuilder.iterate() for translation rationale.
    return this.addNode("iterate", translateIterateArgs(a));
  }
  public call(a: CallArgs): this {
    // G19-D §7.9 — see SubgraphBuilder.call() for translation rationale.
    return this.addNode("call", translateCallArgs(a));
  }
  public respond(a: RespondArgs = {}): this {
    // G19-D §7.9 — see SubgraphBuilder.respond() for translation rationale.
    return this.addNode("respond", translateRespondArgs(a));
  }
  public emit(a: EmitArgs): this {
    // R6 Round-2 r6-r2-mpc-1 fix-pass: map `EmitArgs.event` onto the
    // `channel` property the eval-side EMIT executor reads. See the
    // sibling builder's `emit()` method earlier in this file for the
    // full rationale.
    const props: Record<string, JsonValue> = { channel: a.event };
    if (a.payload !== undefined) {
      props.payload = a.payload;
    }
    return this.addNode("emit", props);
  }
  public wait(a: WaitArgs): this {
    // R6-R5 r6-r5-pcds-2 fix-pass: see the sibling builder's `wait()`
    // method earlier in this file for the full rationale on the
    // duration-string → duration_ms / timeout_ms translation. Both
    // builders MUST stay in lockstep on the spread shape.
    const s = a as { signal?: string; duration?: string };
    if (!s || (s.signal == null && s.duration == null)) {
      throw new EDslInvalidShape(
        "wait(args) requires either `signal: string` or `duration: string` (E_DSL_INVALID_SHAPE)",
      );
    }
    return this.addNode("wait", translateWaitArgs(a));
  }
  public stream(a: StreamArgs): this {
    return this.addNode("stream", { ...a } as Record<string, JsonValue>);
  }
  public subscribe(a: SubscribeArgs): this {
    // G19-D §7.10 — see SubgraphBuilder.subscribe() for translation rationale.
    // `handler?` re-introduced post-G14-D; both builders MUST stay in lockstep.
    return this.addNode("subscribe", translateSubscribeArgs(a));
  }
  public sandbox(a: SandboxArgs): this {
    // Phase-3 G17-C wave-5b (24th p/c drift; pim-2 LOAD-BEARING):
    // translate camelCase DSL args to snake_case eval-side keys via
    // translateSandboxArgs. Both SubgraphBuilder.sandbox() variants
    // route through the same translator so a regression at one site is
    // caught by the load-bearing eval-side end-to-end pin.
    return this.addNode("sandbox", translateSandboxArgs(a));
  }

  /** Move the scope's nodes out for merging into the parent. */
  public drain(): SubgraphNode[] {
    return this.scopeNodes;
  }
}

// ---------------------------------------------------------------------------
// Top-level constructor functions
// ---------------------------------------------------------------------------

/** Create a new subgraph builder with the given handler id. */
export function subgraph(handlerId: string): SubgraphBuilder {
  return new SubgraphBuilder(handlerId);
}

// Individual primitive helpers — occasionally useful for one-shot Nodes
// that need to be stitched into a larger subgraph imperatively. All of
// these simply build a primitive-args object; they do not produce a
// Subgraph on their own.
export function read(args: ReadArgs): { primitive: "read"; args: ReadArgs } {
  return { primitive: "read", args };
}
export function write(args: WriteArgs): { primitive: "write"; args: WriteArgs } {
  return { primitive: "write", args };
}
export function transform(args: TransformArgs): {
  primitive: "transform";
  args: TransformArgs;
} {
  return { primitive: "transform", args };
}
export function branch(args: BranchArgs): {
  primitive: "branch";
  args: BranchArgs;
} {
  return { primitive: "branch", args };
}
export function iterate(args: IterateArgs): {
  primitive: "iterate";
  args: IterateArgs;
} {
  return { primitive: "iterate", args };
}
export function call(args: CallArgs): { primitive: "call"; args: CallArgs } {
  return { primitive: "call", args };
}
export function respond(args: RespondArgs = {}): {
  primitive: "respond";
  args: RespondArgs;
} {
  return { primitive: "respond", args };
}
export function emit(args: EmitArgs): { primitive: "emit"; args: EmitArgs } {
  return { primitive: "emit", args };
}
export function wait(args: WaitArgs): { primitive: "wait"; args: WaitArgs } {
  const s = args as { signal?: string; duration?: string };
  if (!s || (s.signal == null && s.duration == null)) {
    throw new EDslInvalidShape(
      "wait(args) requires either `signal: string` or `duration: string` (E_DSL_INVALID_SHAPE)",
    );
  }
  return { primitive: "wait", args };
}
export function stream(args: StreamArgs): {
  primitive: "stream";
  args: StreamArgs;
} {
  return { primitive: "stream", args };
}
export function subscribe(args: SubscribeArgs): {
  primitive: "subscribe";
  args: SubscribeArgs;
} {
  return { primitive: "subscribe", args };
}
export function sandbox(args: SandboxArgs): {
  primitive: "sandbox";
  args: SandboxArgs;
} {
  return { primitive: "sandbox", args };
}

// ---------------------------------------------------------------------------
// crud('post') zero-config shorthand
// ---------------------------------------------------------------------------

/**
 * HLC stamp source — returns a monotonically-increasing millisecond
 * epoch reading. Phase 1 uses a single-process HLC: wall-clock max'd
 * with (last + 1) to guarantee strict-monotone even across calls that
 * fall within the same millisecond. Peer-sync HLC (with physical +
 * logical components and peer-id tiebreaker) is Phase 3.
 */
let __lastHlc = 0;
function hlcNow(): number {
  const now = Date.now();
  __lastHlc = now > __lastHlc ? now : __lastHlc + 1;
  return __lastHlc;
}

export interface CrudOptions {
  /** Override the label (default: the first `crud()` argument). */
  label?: string;
  /** Supply your own HLC source (useful for deterministic tests). */
  hlc?: () => number;
  /**
   * Capability expression required to execute the mutating actions
   * (`create`, `update`, `delete`) of this handler. Stamped as a
   * `requires` property on each WRITE Node in the produced subgraph.
   *
   * When the engine is opened with `PolicyKind.GrantBacked`, the
   * capability is checked at commit time: an unrevoked
   * `system:CapabilityGrant` Node whose `scope` matches the
   * expression must exist. With `PolicyKind.NoAuth` (default) the
   * property is informational only.
   */
  capability?: string;
  /**
   * When `true`, flags this handler as expecting the `debug:read`
   * capability — callers who hold the grant can use
   * [`Engine.diagnoseRead`] to probe denied / missing reads against
   * Nodes of this handler's label (named compromise #2, Option C;
   * see `docs/SECURITY-POSTURE.md`). The flag is informational in
   * Phase 1 (the actual gate is `engine.grantCapability({ actor,
   * scope: "store:debug:read" })` — the DSL surface is a hint for
   * tooling). Defaults `false`.
   */
  debugRead?: boolean;
}

/**
 * A registrable object returned by `crud(label)`. Carries the
 * constructed subgraph plus convenience action wrappers. Typically used as:
 *
 *     const handler = await engine.registerSubgraph(crud('post'));
 *     await engine.call(handler.id, 'post:create', { title: 'x' });
 *
 * The convenience methods (`crud('post').create(...)`) expect the
 * returned object to be registered against an engine before they can
 * execute; they are thin wrappers that require a bound Engine
 * reference, which is added by `engine.registerSubgraph()` at
 * registration time (see `engine.ts`).
 */
export interface CrudHandler {
  /**
   * The underlying Subgraph. `engine.registerSubgraph()` consumes this
   * directly — the CRUD object IS a Subgraph plus convenience methods.
   */
  readonly subgraph: Subgraph;
  /** The (static) action strings exposed by this handler. */
  readonly actions: string[];
  /** Label name for this CRUD handler (the first `crud()` argument). */
  readonly label: string;
  /** Returns an HLC-stamped createdAt ms-since-epoch. */
  stampCreatedAt(): number;
}

/**
 * Zero-config CRUD. Builds a Subgraph with five canonical actions
 * (`<label>:create`, `:get`, `:list`, `:update`, `:delete`) and returns
 * a handle ready to pass to `engine.registerSubgraph()`.
 *
 * The generated Subgraph preserves structural identity for a given
 * `(label, hlc source)` pair: re-invoking `crud('post')` in a separate
 * call site yields a structurally-identical subgraph so handler CIDs
 * are stable across invocations.
 */
export function crud(label: string, opts: CrudOptions = {}): CrudHandler {
  if (!label || typeof label !== "string") {
    throw new EDslInvalidShape("crud(label) requires a non-empty string label");
  }
  const actualLabel = opts.label ?? label;
  const stamp = opts.hlc ?? hlcNow;
  const cap = opts.capability;

  // The canonical shape: one dispatch BRANCH on `$input.action`, five
  // cases. Each case is a tiny linear chain. Action names are bare
  // (`create` / `get` / …) so `handler.actions` reads naturally; the
  // label-prefixed form (`post:create`) is how CALLERS name the op
  // they want to dispatch and is parsed by the engine.
  const sg = subgraph(`${actualLabel}-handler`)
    .action("create")
    .action("get")
    .action("list")
    .action("update")
    .action("delete")
    .branch({ on: "$input.action" })
    .case("create", (s) =>
      s
        .write({
          label: actualLabel,
          properties: { from: "$input" },
          ...(cap ? { requires: cap } : {}),
        })
        .respond({ body: "$result" }),
    )
    .case("get", (s) =>
      s
        .read({ label: actualLabel, by: "cid", value: "$input.cid" })
        .respond({ body: "$result" }),
    )
    .case("list", (s) =>
      s
        .read({ label: actualLabel, by: "_listView" })
        .respond({ body: "$result" }),
    )
    .case("update", (s) =>
      s
        .write({
          label: actualLabel,
          properties: { cid: "$input.cid", patch: "$input.patch" },
          ...(cap ? { requires: cap } : {}),
        })
        .respond({ body: "$result" }),
    )
    .case("delete", (s) =>
      s
        .write({
          label: actualLabel,
          properties: { cid: "$input.cid", tombstone: true },
          ...(cap ? { requires: cap } : {}),
        })
        .respond({ body: "$result" }),
    )
    .build();

  return {
    subgraph: sg,
    actions: sg.actions,
    label: actualLabel,
    stampCreatedAt: stamp,
  };
}

/**
 * Type guard: is `x` a `CrudHandler`? Used by the engine wrapper to
 * detect `engine.registerSubgraph(crud('post'))` vs.
 * `engine.registerSubgraph(handBuilt)`.
 */
export function isCrudHandler(x: unknown): x is CrudHandler {
  return (
    typeof x === "object" &&
    x !== null &&
    "subgraph" in x &&
    "label" in x &&
    "stampCreatedAt" in x &&
    typeof (x as { stampCreatedAt: unknown }).stampCreatedAt === "function"
  );
}

/**
 * Type guard: is `x` a raw `Subgraph`? Used by the engine wrapper.
 */
export function isSubgraph(x: unknown): x is Subgraph {
  return (
    typeof x === "object" &&
    x !== null &&
    "handlerId" in x &&
    "nodes" in x &&
    Array.isArray((x as { nodes: unknown }).nodes) &&
    "root" in x
  );
}
