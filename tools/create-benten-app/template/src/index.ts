// Entry point for the {{name}} Benten project.
//
// This file opens an Engine against a local redb database, registers the
// `crud('post')` zero-config handler, and exposes the registered handler
// object. `npm run dev` executes this file directly via tsx; `npm run test`
// imports `./handlers` and exercises the same handler via Vitest.

import { join } from "node:path";
import { Engine } from "@benten/engine";
import { postHandlers } from "./handlers.js";

async function main(): Promise<void> {
  const dbPath = join(process.cwd(), ".benten", "{{name}}.redb");
  const engine = await Engine.open(dbPath);
  const handler = await engine.registerSubgraph(postHandlers);
  // eslint-disable-next-line no-console
  console.log(`registered handler: ${handler.id}`);
  // eslint-disable-next-line no-console
  console.log(`actions: ${handler.actions.join(", ")}`);
  await engine.close();
}

// Only run main() when executed directly (not when imported by tests).
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((err) => {
    // eslint-disable-next-line no-console
    console.error(err);
    process.exit(1);
  });
}
