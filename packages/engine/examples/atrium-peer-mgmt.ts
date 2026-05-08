// Atrium peer-management example (Phase 3 G16-D wave-6b — D1 B-prime
// factory-handle DSL per Ben's 2026-05-05 ratification).
//
// Demonstrates the SHAPE of the Atrium peer-management surface:
//
//   1. `engine.atrium({ atriumId })` — callable factory returns a
//      typed `Atrium` handle. Multi-Atrium-as-default: each call
//      returns a fresh per-session handle.
//   2. `family.join()` — initiate peer discovery + handshake.
//   3. `family.trustPeer(did)` — extend trust to a peer DID.
//   4. `family.listPeers()` — observe the trusted set.
//   5. `family.revokePeer(did)` — revoke trust + terminate active
//      subscriptions per exit-criterion 15 (D-PHASE-3-15).
//   6. `family.onPeerJoin(cb)` / `family.onPeerLeave(cb)` — lifecycle
//      hooks.
//   7. `family.leave()` — tear down per-session state.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/atrium-peer-mgmt.ts

import { Engine, PolicyKind } from "@benten/engine";

/**
 * Run the Atrium peer-management example end-to-end.
 *
 * Exported as `run` (rather than auto-invoked on import) so the
 * `quickstart_examples_compile_and_run` companion pin
 * `atrium_examples` can import the module under Vitest without
 * triggering napi side effects. Direct CLI execution still works:
 * the bottom of the module gates `run()` on `import.meta.url`
 * matching the launching script.
 */
export async function run(): Promise<{ ok: true }> {
  // Phase 3 layered the durable UCANBackend on top of GrantBacked.
  // PolicyKind.Ucan opens an engine wired to consult UCAN-grounded
  // grants (issuer + audience + nbf/exp validation walking through
  // benten-id's chain validator).
  const engine = await Engine.openWithPolicy(
    ".benten/example-atrium-peer-mgmt.redb",
    PolicyKind.Ucan,
  );
  try {
    // Construct a fresh per-session Atrium handle. The atriumId is the
    // caller-chosen logical key; multiple calls with the same id
    // return distinct handles routing to the same logical Atrium.
    const family = engine.atrium({ atriumId: "family" });

    // Wire the lifecycle hooks BEFORE join so early peer-join events
    // are captured. Hooks are local to this handle; revoking a peer
    // notifies onPeerLeave subscribers locally per the wave-6b
    // shim semantics.
    family.onPeerJoin((peerDid) => {
      process.stdout.write(`peer joined: ${peerDid}\n`);
    });
    family.onPeerLeave((peerDid) => {
      process.stdout.write(`peer left: ${peerDid}\n`);
    });

    await family.join();
    process.stdout.write(
      `joined Atrium ${family.atriumId} (isJoined=${family.isJoined})\n`,
    );

    // Extend trust to two peer DIDs (laptop + phone-OS app).
    const laptopDid =
      "did:key:z6MkrJVnaZkeFzdQyMZu1csdAuLAaKSzjGpJSjm9V1F4xzm";
    const phoneDid =
      "did:key:z6MkfRiv4MwBfhJjXMrRsXsZJSjEmDjRCZxvZbEDHUExMRKx";
    await family.trustPeer(laptopDid);
    await family.trustPeer(phoneDid);

    const peers = family.listPeers();
    process.stdout.write(`trusted peers: ${JSON.stringify(peers)}\n`);

    // Revoke the laptop's trust. The handle's lifecycle hook fires
    // locally; remote subscriptions terminate per exit-criterion 15.
    await family.revokePeer(laptopDid);
    const after = family.listPeers();
    process.stdout.write(`after revoke: ${JSON.stringify(after)}\n`);

    await family.leave();
  } finally {
    await engine.close();
  }
  return { ok: true };
}

// Direct-invocation guard — run only when the example is the
// process entry point (e.g. `node --experimental-strip-types
// examples/atrium-peer-mgmt.ts`).
const isMainModule =
  typeof process !== "undefined" &&
  process.argv[1] !== undefined &&
  import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  run().catch((err: unknown) => {
    process.stderr.write(`atrium-peer-mgmt failed: ${String(err)}\n`);
    process.exit(1);
  });
}
