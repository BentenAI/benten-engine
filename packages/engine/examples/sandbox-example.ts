// SANDBOX example runner — installs a module manifest, registers the
// handler, exercises the SANDBOX call.
//
// The example demonstrates the SHAPE of the SANDBOX call path. It
// expects a pre-existing WASM module CID + manifest CID; in
// production these are produced by `engine.computeManifestCid()` over
// a manifest authored against compiled module bytes.
//
// Usage:
//   cd packages/engine && npm run build
//   node --experimental-strip-types examples/sandbox-example.ts

import { Engine, PolicyKind, crud } from "@benten/engine";
import type { ModuleManifest } from "@benten/engine";
import { buildSandboxHandler, sandboxHandlerId } from "./sandbox-handler.js";

// In a real workload these are produced by `npm run compile-wasm`
// (the Phase-3 marketplace adds a friendlier path); for the example
// we hard-code placeholder CIDs — the example aborts before SANDBOX
// dispatch when the placeholder module is not actually installed.
const SUMMARIZE_MODULE_CID =
  "bafyr4igexample0summarize0module0cid0placeholder0replace0me0aaaaa";
const MANIFEST_CID =
  "bafyr4igexample0manifest0cid0placeholder0replace0me0aaaaaaaaaaaa";

const summarizerManifest: ModuleManifest = {
  name: "example.summarizer",
  version: "0.1.0",
  modules: [
    {
      name: "summarize-v1",
      cid: SUMMARIZE_MODULE_CID,
      requires: [
        "host:compute:kv:read",
        "host:compute:log",
        "host:compute:time",
      ],
    },
  ],
};

async function main(): Promise<void> {
  const engine = await Engine.openWithPolicy(
    ".benten/example-sandbox.redb",
    PolicyKind.GrantBacked,
  );
  try {
    // Grant the manifest's caps to ourselves so the SANDBOX call
    // passes per-boundary + per-call rechecks.
    await engine.grantCapability({
      actor: "alice",
      scope: "host:compute:time",
    });
    await engine.grantCapability({
      actor: "alice",
      scope: "host:compute:log",
    });
    await engine.grantCapability({
      actor: "alice",
      scope: "host:compute:kv:read",
    });

    // Install the manifest. Returns the manifest CID on success.
    const installedCid = await engine.installModule(
      summarizerManifest,
      MANIFEST_CID,
    );
    process.stdout.write(`installed manifest: ${installedCid}\n`);

    // Need a `doc` label to read from inside the SANDBOX handler.
    await engine.registerSubgraph(crud("doc"));
    await engine.callAs(
      "doc-handler",
      "doc:create",
      {
        id: "d-42",
        body:
          "The quick brown fox jumps over the lazy dog. " +
          "Repeated text to give the summariser something to do.",
      },
      "alice",
    );

    // Register + call the SANDBOX handler. The module reference uses
    // the registry-name form (`<manifestName>:<moduleName>`) so the
    // engine resolves it via the named-manifest registry.
    const handler = buildSandboxHandler(
      `${summarizerManifest.name}:${summarizerManifest.modules[0].name}`,
    );
    await engine.registerSubgraph(handler);

    const out = await engine.callAs(
      sandboxHandlerId,
      "default",
      { doc_id: "d-42" },
      "alice",
    );
    process.stdout.write(`summary: ${JSON.stringify(out)}\n`);
  } finally {
    await engine.close();
  }
}

main().catch((err: unknown) => {
  process.stderr.write(`sandbox-example failed: ${String(err)}\n`);
  process.exit(1);
});
