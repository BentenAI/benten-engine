# DSL Specification

The TypeScript DSL surface (`@benten/engine`) for composing Benten
operation subgraphs. Each primitive method (e.g. `subgraph(...).read({...})`)
emits a `SubgraphNode` whose `args` property bag is consumed by the
Rust evaluator at `crates/benten-eval/src/primitives/<p>.rs::execute`.

This document is the user-facing description of how DSL surface fields
translate to the eval-side property-bag keyspace. The structural defense
against drift between the two surfaces lives in the LOAD-BEARING parity
meta-test at
`crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs`
(see Phase-3 G19-D §7.10 + D-PHASE-3-9).

## Translation contract

Each `*Args` interface in `packages/engine/src/dsl.ts` is paired with a
`translateXxxArgs` helper that maps the DSL surface field names onto the
eval-side primitive's actual keyspace. The translation lives at the DSL
spread (the `subgraph(...).<primitive>(args)` builder method) so the
eval-side reader sees its canonical key shape with zero round-trip
surface knowledge.

| DSL surface field | Eval-side keyspace key | Primitive |
|---|---|---|
| `ReadArgs.label` | `label` | READ |
| `ReadArgs.by: "cid"`/`"id"` | `query_kind: "by_cid"` | READ |
| `ReadArgs.by: "_listView"` | `query_kind: "list_view"` | READ |
| `ReadArgs.value` | `target_cid` | READ |
| `BranchArgs.on` | `match_value` | BRANCH |
| `IterateArgs.over` | `items` | ITERATE |
| `IterateArgs.max` | `max` (verbatim) | ITERATE |
| `TransformArgs.expr` | `expr` (verbatim) | TRANSFORM |
| `TransformArgs.as` | `result` | TRANSFORM |
| `CallArgs.handler` | `target` | CALL |
| `CallArgs.action` | `call_op` | CALL |
| `CallArgs.input` | `input` (verbatim) | CALL |
| `CallArgs.isolated: true` | `child_scope: true` | CALL |
| `RespondArgs.body` | `body` (verbatim) | RESPOND |
| `RespondArgs.status` | `status` (verbatim) | RESPOND |
| `RespondArgs.edge` | (edge-table-driven; not in args bag) | RESPOND |
| `EmitArgs.event` | `channel` | EMIT |
| `EmitArgs.payload` | `payload` (verbatim) | EMIT |
| `WaitArgs.signal` | `signal` (verbatim) | WAIT |
| `WaitArgs.duration` (bare) | `duration_ms: parseDurationToMs(d)` | WAIT |
| `WaitArgs.duration` (with signal) | `timeout_ms: parseDurationToMs(d)` | WAIT |
| `WaitArgs.signal_shape` | `signal_shape` (verbatim) | WAIT |
| `SubscribeArgs.event` | `pattern` | SUBSCRIBE |
| `SubscribeArgs.handler` | `handler` (verbatim) | SUBSCRIBE |
| `SandboxArgs.module`/`caps`/`fuel`/`input` | verbatim | SANDBOX |
| `SandboxArgs.wallclockMs` | `wallclock_ms` | SANDBOX |
| `SandboxArgs.outputLimitBytes` | `output_limit` | SANDBOX |

Compile-path-supplied keys (NOT spread from the DSL surface):

- BRANCH: `cases`, `has_default`, `conditions`, `condition_value` —
  populated by the engine compile path from the BRANCH node's outgoing
  edge table (`CASE:<value>` labels stamped by `.case(value, body)`).
- ITERATE: `requires` — capability decl from a separate sources.
- TRANSFORM: `input` — populated from the upstream binding by the
  engine compile path.
- CALL: `parent_scope`, `requires`, `timeout_ms`, `elapsed_ms` —
  populated by the engine compile path from the surrounding CALL frame.
- WRITE: `properties`, `requires` — extracted by `WriteSpec` extraction
  at the napi boundary (`bindings/napi/src/subgraph.rs::extract_write_args`).

## SUBSCRIBE handler-id-router worked example (G14-D wave-5a)

Phase-3 G14-D wired the eval-side handler-id-router seam at
`crates/benten-eval/src/primitives/subscribe.rs::execute` lines
1295-1317. The seam routes change-event delivery THROUGH a named
handler instead of the default broadcast fan-out.

G19-D wave-7 restored the corresponding TS DSL surface
(`SubscribeArgs.handler?`) so a developer can express the
handler-id-router model end-to-end:

```ts
import { subgraph, Engine } from "@benten/engine";

// 1. Define the per-event handler subgraph (the router target).
const onPostCreated = subgraph("post-created-handler")
  .action("handle")
  .read({ label: "post", by: "cid", value: "$input.cid" })
  .transform({ expr: "$result | append-to-feed", as: "feed" })
  .write({ label: "feed", properties: { post_cid: "$input.cid" } })
  .respond()
  .build();

// 2. Define the SUBSCRIBE that routes through the handler. The
//    `handler` field translates to eval-side `handler: Text(<id>)`
//    per the G14-D handler-id-router seam.
const subscribeFeed = subgraph("subscribe-feed")
  .subscribe({
    event: "post:created",
    handler: "post-created-handler",
  })
  .respond()
  .build();

// 3. Register both with the engine. The SUBSCRIBE's published change
//    events route to the handler instead of fan-out to all listeners.
const engine = await Engine.open(":memory:");
await engine.registerSubgraph(onPostCreated);
await engine.registerSubgraph(subscribeFeed);
```

### Routing semantics

Without `handler`, the SUBSCRIBE primitive uses the default
broadcast-fan-out: every active subscription matching the `pattern`
receives the event payload via the engine's shared change-stream
delivery path (`ChangeBroadcast` → `subscribe::publish_change_event_with_labels`).

With `handler`, the primitive routes the event through the named
handler subgraph as if the publisher had explicitly invoked
`engine.call(handlerId, "handle", changeEvent)`. The named handler
becomes the single consumer of the routed events; broadcast
subscribers do NOT receive the routed events on the same delivery.

### Why a per-handler router?

The router seam closes the 21st producer/consumer drift instance
(R6-R4-narrow-pcds-1): pre-G14-D the `SubscribeArgs.handler` field was
a phantom (TS DSL produced it; eval never read it) which PR #75's
R6-R4-narrow fix-pass had to drop. G14-D wave-5a wired the eval-side
seam; G19-D wave-7 re-introduces the corresponding TS DSL surface;
the LOAD-BEARING parity meta-test asserts no orphan reads/writes on
either side end-to-end.

## Testing the translation contract

Two tests defend against drift recurrence:

1. **`crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs::dsl_args_vs_eval_properties_parity_meta_test_no_drift_across_all_primitives`** — walks every `*Args` interface's translator output keyspace against the canonical eval-side keyspace; FAILS if any translator emits a key the eval primitive does not read.

2. **`packages/engine/test/dsl_args_drift.test.ts`** — pure-DSL round-trip pins asserting each `subgraph(...).<primitive>(args).build()` produces a SubgraphNode whose `args` property bag carries the eval-side canonical keys (the structural defense at the TS test layer).

Together these make the 6-Args drift fix structural rather than per-instance.

## Cross-references

- `crates/benten-eval/src/primitives/<p>.rs::execute` — eval-side
  property-bag readers (canonical keyspace source-of-truth).
- `packages/engine/src/dsl.ts::translate*Args` — DSL→eval translator
  helpers (translation contract authors).
- `crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs` —
  LOAD-BEARING parity meta-test (D-PHASE-3-9 EXPANDED + pim-12).
- `crates/benten-engine/tests/ts_surface_parity_meta_test.rs` —
  TS-surface-parity meta-test (Edge interface + napi struct surface).
- `.addl/phase-3/00-implementation-plan.md` §7.9 / §7.10 — Phase-3
  wave-7 G19-D scope contract.
- Phase-2b retrospective (24-instance long-tail recurrence): see
  `docs/future/phase-2-backlog.md` + `docs/future/phase-3-backlog.md`.
