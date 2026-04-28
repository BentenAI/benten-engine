// SUBSCRIBE example runner — registers the handler, then writes a few
// `post` Nodes to drive the subscription.
//
// The SUBSCRIBE handler runs once per ChangeEvent matching its
// declared event pattern, projecting the change into a derived
// `post-summary` Node. Reactive handlers do not need `engine.call` —
// the engine drives them off the change-event bus.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/subscribe-example.ts

import { Engine, crud } from "@benten/engine";
import {
  subscribeHandler,
  subscribeHandlerId,
} from "./subscribe-handler.js";

async function main(): Promise<void> {
  const engine = await Engine.open(".benten/example-subscribe.redb");
  try {
    // Register the post CRUD handler so we can WRITE Nodes that drive
    // the subscription.
    await engine.registerSubgraph(crud("post"));

    // Register the SUBSCRIBE handler — Engine wires it into the
    // change-event bus on registration.
    await engine.registerSubgraph(subscribeHandler);

    // Open a downstream subscription so this runner can observe the
    // emit the projection fires after each successful summary write.
    // Cursor `{ kind: "latest" }` skips backlog and starts at the next
    // arriving event.
    const sub = engine.onChange(
      "post-summary:built",
      (seq, chunk) => {
        process.stdout.write(
          `summary built: seq=${seq} payload=${chunk.length}B\n`,
        );
      },
      { kind: "latest" },
    );
    process.stdout.write(
      `subscription open: pattern=${sub.pattern} active=${sub.active}\n`,
    );

    // Drive the SUBSCRIBE handler by writing posts. Each WRITE fires
    // a `post:changed` ChangeEvent; the SUBSCRIBE handler runs once
    // per event and itself emits `post-summary:built`.
    await engine.call("post-handler", "post:create", {
      title: "Hello Benten",
      body: "First post.",
    });
    await engine.call("post-handler", "post:create", {
      title: "Reactive views work",
      body: "Second.",
    });

    // Wait briefly for the projection's downstream events. Real apps
    // would shut down via a signal handler; here we just sleep.
    await new Promise((resolve) => setTimeout(resolve, 500));

    process.stdout.write(
      `subscribed handler: ${subscribeHandlerId} (delivered seq up to ${sub.maxDeliveredSeq})\n`,
    );
    sub.unsubscribe();
  } finally {
    await engine.close();
  }
}

main().catch((err: unknown) => {
  process.stderr.write(`subscribe-example failed: ${String(err)}\n`);
  process.exit(1);
});
