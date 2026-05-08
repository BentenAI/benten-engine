// UCAN-grant-flow example (Phase 3 G14-B — durable UCANBackend over
// benten-id; CLR-2 / cap-major-2 / crypto-blocker-2 / crypto-major-4
// closures).
//
// Demonstrates the SHAPE of the UCAN-grounded grant flow on the public
// surface that ships with `@benten/engine`:
//
//   1. `Engine.openWithPolicy(path, PolicyKind.Ucan)` — opens an
//      engine wired to consult UCAN-grounded grants.
//   2. `engine.grantCapability({ actor, scope, issuer?, hlc? })` —
//      Phase-1 surface extended with optional `issuer` (UCAN-grounding
//      DID) and `hlc` (causal stamp) fields. Ignored fields tolerated
//      on the wire by the Rust parser; UCANBackend consults them.
//   3. `engine.revokeCapability({ actor, scope })` — revocation is a
//      Node write; UCANBackend re-walks the chain on next check.
//   4. `engine.callAs(handler, action, input, actor)` — dispatch
//      through the policy. Denied calls surface
//      `E_CAP_DENIED`; revoked-mid-flight surfaces
//      `E_CAP_REVOKED_MID_EVAL`; expired UCAN windows surface the
//      typed error per crypto-blocker-2 nbf/exp validation.
//
// Identity primitives (Keypair / SignedUcan / VerifiableCredential) live
// inside `packages/engine/src/identity.ts` but are NOT part of the
// public `@benten/engine` index export — the runtime crypto path runs
// on the napi-side full-peer (Rust) and thin-clients route over the
// authenticated thin-client protocol (D-PHASE-3-30) per CLAUDE.md
// baked-in #17. This example demonstrates the public-surface grant
// flow only; a full peer's UCAN issuer signs grants out-of-band.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/ucan-grant-flow.ts

import { Engine, PolicyKind, crud } from "@benten/engine";

async function main(): Promise<void> {
  const engine = await Engine.openWithPolicy(
    ".benten/example-ucan-grant.redb",
    PolicyKind.Ucan,
  );
  try {
    const aliceDid =
      "did:key:z6MkrJVnaZkeFzdQyMZu1csdAuLAaKSzjGpJSjm9V1F4xzm";

    // Grant alice the wildcard capability on the `post` zone. Phase-3
    // optional `issuer` field carries the UCAN-grounding root DID;
    // `hlc` carries the causal stamp the chain-walker uses for
    // before/after ordering.
    await engine.grantCapability({
      actor: aliceDid,
      scope: "store:post:*",
      issuer: aliceDid, // self-signed root grant
      hlc: Date.now(),
    });

    const handler = await engine.registerSubgraph(
      crud("post", { capability: "store:post:*" }),
    );

    // alice's grant attenuates `store:post:*` to `store:post:write`
    // at the WRITE pre-policy hook; UCANBackend walks the chain
    // (single hop since the grant was self-signed); cap-recheck fires
    // again at every iteration of multi-step handlers per Phase-2a
    // TOCTOU hardening.
    const out = await engine.callAs(
      handler.id,
      "post:create",
      { title: "Hello UCAN", body: "Grant chain walked." },
      aliceDid,
    );
    process.stdout.write(`created: ${JSON.stringify(out)}\n`);

    // Revoke the grant. UCANBackend treats revocation as a Node
    // write that the chain-walker consults on the next dispatch;
    // any in-flight handler that re-checks the cap surfaces
    // E_CAP_REVOKED_MID_EVAL per Phase-2a hardening.
    await engine.revokeCapability({
      actor: aliceDid,
      scope: "store:post:*",
    });

    // The next dispatch fails with E_CAP_DENIED — the chain no longer
    // attenuates to `store:post:write` because the grant is revoked.
    try {
      await engine.callAs(
        handler.id,
        "post:create",
        { title: "Should fail", body: "Grant was revoked." },
        aliceDid,
      );
      process.stdout.write("UNEXPECTED: dispatch succeeded after revoke\n");
    } catch (err: unknown) {
      process.stdout.write(`expected failure post-revoke: ${String(err)}\n`);
    }
  } finally {
    await engine.close();
  }
}

main().catch((err: unknown) => {
  process.stderr.write(`ucan-grant-flow failed: ${String(err)}\n`);
  process.exit(1);
});
