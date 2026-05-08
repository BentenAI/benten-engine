// DID-resolution example (Phase 3 G14-A1 — `benten-id` `did:key`
// surface).
//
// Demonstrates the SHAPE of the `did:key` round-trip surface as
// consumed via the public Atrium handle:
//
//   1. `engine.atrium({...}).join()` — Atrium handle exposes the
//      DID-aware peer surface.
//   2. `family.trustPeer(did)` — accepts `did:key:z<...>` strings
//      directly; the napi binding routes through `benten-id`'s
//      `did:key` parser to canonicalize before recording.
//   3. `family.declareDeviceAttestation({ deviceDid, capabilities,
//      freshnessWindow })` — declares a device-DID's per-claim
//      capabilities (CLAUDE.md baked-in #17 device-mesh
//      composition; D-PHASE-3-25 heterogeneity contract).
//   4. `family.listDeclaredDeviceAttestations()` — round-trip the
//      declared envelopes (pcds-r4-r1-2 round-trip pin).
//
// `did:plc` is reserved for Phase 9+ exploration (see `docs/future/`);
// at Phase 3 close `did:key` is the canonical method.
//
// Note: the standalone `Keypair.generate()` / `verifySignature()`
// surfaces in `packages/engine/src/identity.ts` are not part of the
// public `@benten/engine` index export. Identity ops route to the
// full peer (napi cdylib) on full-peer targets and to the thin-client
// protocol on wasm32 targets per CLAUDE.md baked-in #17.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/did-resolution.ts

import { Engine, PolicyKind } from "@benten/engine";
import type { DeviceAttestation } from "@benten/engine";

/**
 * Run the DID-resolution example. Exported as `run` so the
 * `atrium_examples` companion pin can import the module without
 * triggering napi side effects on import.
 */
export async function run(): Promise<{ ok: true }> {
  const engine = await Engine.openWithPolicy(
    ".benten/example-did-resolution.redb",
    PolicyKind.Ucan,
  );
  try {
    const home = engine.atrium({ atriumId: "home" });
    await home.join();

    // Round-trip a `did:key:z<...>` string through the trust surface.
    // The napi binding parses the DID via benten-id's did_key parser
    // and rejects malformed strings with a typed error.
    //
    // G21-T1 typed-CALL: did_resolve — the engine-internal DID parse
    // is the typed-CALL `engine:typed:did_resolve` op. A handler-author
    // composing a DID-routing subgraph invokes it via a CALL Node
    // `target: "engine:typed:did_resolve"` to get the public-key bytes
    // back as `Value::Map { method, public_key }` per the op's output
    // schema (see `crates/benten-eval/src/typed_call.rs`). The T2 napi
    // widening exposes `engine.typedCall(...)` as a TS sibling so the
    // same op is reachable from JS land.
    const peerDid =
      "did:key:z6MkrJVnaZkeFzdQyMZu1csdAuLAaKSzjGpJSjm9V1F4xzm";
    await home.trustPeer(peerDid);
    const trusted = home.listPeers();
    process.stdout.write(`canonicalized peer DIDs: ${JSON.stringify(trusted)}\n`);

    // Declare a device-attestation envelope. Per the Phase-3 device
    // mesh exploration (CLAUDE.md baked-in #17), the envelope
    // narrows what this device may exercise against the full peer's
    // store. Replay-resistance via UCAN chain attenuation.
    const myDevice: DeviceAttestation = {
      deviceDid:
        "did:key:z6MkfRiv4MwBfhJjXMrRsXsZJSjEmDjRCZxvZbEDHUExMRKx",
      capabilities: [
        { path: "/zone/posts/*", ability: "read" },
        { path: "/zone/posts/draft/*", ability: "write" },
      ],
      freshnessWindow: 3600, // re-declare hourly
    };
    await home.declareDeviceAttestation(myDevice);

    // Round-trip the declared attestations back out — pcds-r4-r1-2
    // pin: the typed-struct contract from napi -> TS preserves the
    // shape so callers get compile-time type-checking rather than
    // implicit `any`/`unknown`.
    const declared = await home.listDeclaredDeviceAttestations();
    process.stdout.write(
      `declared attestations: ${JSON.stringify(declared, null, 2)}\n`,
    );

    await home.leave();
  } finally {
    await engine.close();
  }
  return { ok: true };
}

const isMainModule =
  typeof process !== "undefined" &&
  process.argv[1] !== undefined &&
  import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  run().catch((err: unknown) => {
    process.stderr.write(`did-resolution failed: ${String(err)}\n`);
    process.exit(1);
  });
}
