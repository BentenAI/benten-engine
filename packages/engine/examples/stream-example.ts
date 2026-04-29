// STREAM example runner — registers the handler, calls it, consumes
// chunks via `for await ... of`.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/stream-example.ts

import { Engine, crud } from "@benten/engine";
import {
  streamHandler,
  streamHandlerAction,
  streamHandlerId,
} from "./stream-handler.js";

async function main(): Promise<void> {
  const engine = await Engine.open(".benten/example-stream.redb");
  try {
    await engine.registerSubgraph(streamHandler);

    // Register the post CRUD handler so we can WRITE Nodes that the
    // STREAM handler will read. The engine assigns the handler id as
    // `crud:<label>` for crud()-registered handlers — capture the
    // returned handle so the dispatch below uses the engine's exact id.
    const postCrud = await engine.registerSubgraph(crud("post"));

    // Seed the post label with a few rows so the STREAM has something
    // to yield. In real workloads the rows arrive via `crud('post')`
    // / sync / SUBSCRIBE projections / etc.
    await engine.call(postCrud.id, "post:create", {
      title: "First",
      body: "Row 1",
    });
    await engine.call(postCrud.id, "post:create", {
      title: "Second",
      body: "Row 2",
    });

    let chunks = 0;
    for await (const chunk of engine.callStream(
      streamHandlerId,
      streamHandlerAction,
      {},
    )) {
      // chunk is a Buffer; print first 60 bytes for the example.
      const preview = chunk.subarray(0, 60).toString("utf8");
      process.stdout.write(`chunk ${++chunks}: ${preview}\n`);
    }
    process.stdout.write(`drained — total chunks: ${chunks}\n`);
  } finally {
    await engine.close();
  }
}

main().catch((err: unknown) => {
  process.stderr.write(`stream-example failed: ${String(err)}\n`);
  process.exit(1);
});
