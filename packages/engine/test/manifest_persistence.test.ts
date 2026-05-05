// R3-D RED-PHASE pin — browser module-manifest survives page reload
// (G18-A wave 5a; plan §3 G18-A).
//
// Pin source: r2-test-landscape §2.6 G18-A row
// `browser_module_manifest_survives_page_reload`.
//
// ## Persistence end-to-end shape
//
// The IndexedDB-backed BrowserManifestStore (G18-A wave-5a) stores
// module manifests durably. A browser tab that:
//
// 1. Calls `engine.registerModuleBytes("compute:safe-default", bytes)`,
// 2. Closes (page navigation / reload),
// 3. Reopens,
//
// MUST observe the manifest still registered (no need to re-call
// `registerModuleBytes`).
//
// ## §3.6b end-to-end pin shape
//
// Per pim-2: drive the production entry point (`engine.registerModuleBytes`)
// + the production retrieval path (the SANDBOX node referencing the
// manifest by name resolves correctly post-reload).
//
// Pairs with `bindings/napi/tests/indexeddb_schema.rs` (Rust source-
// cite assertions for the persistence implementation) and
// `bindings/napi/tests/browser_manifest_store.rs` (the
// `is_persistent → true` flip).

import { describe, it, expect } from "vitest";

describe("R3-D G18-A — browser module-manifest persistence end-to-end", () => {
  it.skip("RED-PHASE: G18-A wave 5a wires IndexedDB manifest persistence + page-reload survival per plan §3 G18-A", () => {
    // plan §3 G18-A pin. G18-A implementer wires this:
    //
    //   import { Engine } from "@benten/engine";
    //
    //   // Page session 1: register a manifest.
    //   {
    //     const engine = await Engine.openInBrowser({ /* IndexedDB-backed */ });
    //     await engine.registerModuleBytes("compute:safe-default", testFixtureBytes());
    //     await engine.close();
    //   }
    //
    //   // Page session 2: reopen — manifest still registered:
    //   {
    //     const engine = await Engine.openInBrowser({ /* same IndexedDB origin */ });
    //
    //     // The manifest is observable post-reload:
    //     const cid = await engine.lookupModuleManifestCid("compute:safe-default");
    //     expect(cid).not.toBeNull();
    //
    //     // And a SANDBOX subgraph referencing the manifest registers
    //     // successfully (no re-call to registerModuleBytes needed):
    //     const sg = subgraph("test").sandbox({ manifestName: "compute:safe-default" });
    //     await expect(engine.registerSubgraph(sg)).resolves.not.toThrow();
    //   }
    //
    // OBSERVABLE consequence: a browser-tab user who registers a
    // module need not re-register on every page reload. Defends
    // br-r1-8 (is_persistent → true) + IndexedDB schema-versioning
    // discipline.
    //
    // CAVEAT: this test runs under Playwright (or wasm-bindgen-test
    // with a real IndexedDB shim like `fake-indexeddb`); a vanilla
    // node runtime cannot drive this end-to-end.
    expect.fail(
      "G18-A wires page-reload survival end-to-end (Playwright cell or fake-indexeddb under vitest)",
    );
  });
});
