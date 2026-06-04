// SUBSCRIBE example handler — pure DSL (Phase 2b G6).
//
// `post-summary-view` subscribes to ChangeEvents on the `post` label,
// projects each event into a `{ id, title }` summary shape, persists
// it under `post-summary`, and emits a downstream
// `post-summary:built` event so other handlers can react.
//
// Reactive handlers do NOT need an explicit `engine.call` — the
// engine drives them off the change-event bus once registered.

import { subgraph } from "@benten/engine";
import type { Subgraph } from "@benten/engine";

export const subscribeHandler: Subgraph = subgraph("post-summary-view")
  .subscribe({ event: "post:changed" })
  .transform({
    expr: "{ id: $event.cid, title: $event.body.title }",
    as: "summary",
  })
  .write({ label: "post-summary" })
  .emit({ event: "post-summary:built", payload: "$result.summary" })
  .build();

export const subscribeHandlerId = "post-summary-view";
