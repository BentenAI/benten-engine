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
//      DID) and `hlc` (causal stamp) fields. NOTE: at Phase-3 close
//      the napi-side `parse_grant_json`
//      (`bindings/napi/src/policy.rs::parse_grant_json`) reads only
//      `actor` + `scope` and silently drops `issuer` + `hlc` before
//      they reach the durable backend. The fields are reserved for
//      G21 T2 napi-UCAN-wireup which widens `parse_grant_json` to
//      thread them through to `UCANBackend`. See
//      `docs/future/phase-3-backlog.md` §2.3.
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

/**
 * Run the UCAN-grant-flow example. Exported as `run` so the
 * `atrium_examples` companion pin can import the module without
 * triggering napi side effects on import.
 */
export async function run(): Promise<{ ok: true }> {
  const engine = await Engine.openWithPolicy(
    ".benten/example-ucan-grant.redb",
    PolicyKind.Ucan,
  );
  try {
    const aliceDid =
      "did:key:z6MkrJVnaZkeFzdQyMZu1csdAuLAaKSzjGpJSjm9V1F4xzm";

    // G21-T1 typed-CALL: keypair_from_seed — the issuer side of a
    // grant flow derives its keypair via the typed-CALL
    // `engine:typed:keypair_from_seed` op (deterministic seed →
    // Ed25519 keypair). A handler-author building a fresh issuer
    // identity uses the seed-from-bytes path; T2 napi widens
    // `engine.typedCall(...)` so TS callers reach the same op.
    //
    // G21-T1 typed-CALL: ucan_validate_chain — when a grant is
    // consumed (the `callAs` invocation below), the engine-side
    // policy check delegates UCAN chain validation to the typed-CALL
    // `engine:typed:ucan_validate_chain` op. The op walks the chain,
    // verifies signatures, checks nbf/exp windows, and asserts the
    // requested capability is granted by the leaf claim. Per
    // CLAUDE.md baked-in #16, this is primitive composition (typed
    // engine op), not SANDBOX. Per phase-3-backlog §2.3, the napi
    // `parse_grant_json` widening at G21 T2 carries the issuer + hlc
    // fields through to the durable UCANBackend.
    //
    // G21-T1 typed-CALL: vc_verify — sibling op; verifiable
    // credentials presented during grant issuance are verified via
    // `engine:typed:vc_verify`.

    // Grant alice the wildcard capability on the `post` zone. Phase-3
    // optional `issuer` field carries the UCAN-grounding root DID;
    // `hlc` carries the causal stamp the chain-walker uses for
    // before/after ordering. The grant CID returned is what
    // `revokeCapability(grantCid, actor)` consumes downstream.
    //
    // NOTE (Phase-3-close honest state): the napi `parse_grant_json`
    // reads only `actor` + `scope` today; `issuer` and `hlc` are
    // dropped before reaching the backend. G21 T2 widens the parser
    // and wires the durable `UCANBackend` so these fields flow
    // through. Until then, this call surfaces `E_CAP_NOT_IMPLEMENTED`
    // at the first WRITE (`callAs` below).
    const grantCid = await engine.grantCapability({
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

    // Revoke the grant. `revokeCapability(grantCid, actor)` writes
    // the revocation Node; UCANBackend consults it on the next
    // dispatch + any in-flight handler that re-checks the cap surfaces
    // E_CAP_REVOKED_MID_EVAL per Phase-2a hardening.
    await engine.revokeCapability(grantCid, aliceDid);

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
  return { ok: true };
}

const isMainModule =
  typeof process !== "undefined" &&
  process.argv[1] !== undefined &&
  import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  run().catch((err: unknown) => {
    process.stderr.write(`ucan-grant-flow failed: ${String(err)}\n`);
    process.exit(1);
  });
}
