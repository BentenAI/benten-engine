// Atrium sync-trigger example (Phase 3 — Atrium subscribe surface
// shape per G16-D wave-6b).
//
// Demonstrates the SHAPE of the Atrium subscribe path:
//
//   1. `engine.atrium({...}).join()` — same factory-handle entry as
//      the peer-management example.
//   2. `family.subscribe(path, callback)` — subscribe to a path
//      within the Atrium's per-session scope. The full peer routes
//      ChangeEvents through F6 cap-recheck before delivery.
//   3. The returned `AtriumSubscription.unsubscribe()` tears down
//      the per-session subscription cleanly (exit-criterion 15
//      revoke-mid-flight composes here).
//
// At wave-6b napi-shim scope the subscription is recorded locally;
// G16-B reconciliation drains real change-events through the
// engine's SUBSCRIBE primitive bus. The example's call shape stays
// stable across that landing.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/atrium-sync-trigger.ts

import { Engine, PolicyKind } from "@benten/engine";

async function main(): Promise<void> {
  const engine = await Engine.openWithPolicy(
    ".benten/example-atrium-sync.redb",
    PolicyKind.Ucan,
  );
  try {
    const team = engine.atrium({ atriumId: "team-foo" });
    await team.join();

    // Trust a peer so its writes will route through to subscribers.
    const collaboratorDid =
      "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSshCNyzRznmH5fYMfu";
    await team.trustPeer(collaboratorDid);

    // Subscribe to a zone path. The callback fires per ChangeEvent
    // the full peer routes through; F6 cap-recheck applies at the
    // full-peer edge (D-PHASE-3-N) so the thin client surface never
    // sees disallowed events.
    const sub = await team.subscribe("/zone/posts", (event) => {
      process.stdout.write(`change-event: ${JSON.stringify(event)}\n`);
    });

    // In a real app, leave the subscription open and drive sync via
    // the engine's existing primitives (writes from any trusted peer
    // surface here). For the example, tear down promptly.
    process.stdout.write("subscription active; tearing down\n");
    await sub.unsubscribe();
    await team.leave();
  } finally {
    await engine.close();
  }
}

main().catch((err: unknown) => {
  process.stderr.write(`atrium-sync-trigger failed: ${String(err)}\n`);
  process.exit(1);
});
