// STREAM example handler — pure DSL (Phase 2b G6).
//
// `export-feed` reads every Node under the `post` label, iterates over
// the rows, streams each one as a chunk back to the consumer, and
// returns a final JSON status when the iteration drains.
//
// The handler value (a `Subgraph`) is import-safe — it carries no
// runtime engine dependency and can be diffed / hashed / mermaided in
// isolation.

import { subgraph } from "@benten/engine";
import type { Subgraph } from "@benten/engine";

export const streamHandler: Subgraph = subgraph("export-feed")
  .read({ label: "post", as: "rows" })
  .iterate({ over: "$result.rows", max: 100_000 })
  .stream({
    source: "$loop.row",
    chunkSize: 64, // ~64 bytes / chunk hint to the executor
  })
  .respond({ body: '{ "status": "streamed" }' })
  .build();

export const streamHandlerId = "export-feed";
export const streamHandlerAction = "default";
