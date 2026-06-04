// R3-D RED-PHASE pin — browser module-manifest survives page reload
// (G18-A wave-5a; plan §3 G18-A).
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
  it.skip("phase-3-backlog §4.3 G18-A-followup: IndexedDB integration + Playwright fixture authoring (production prerequisite NOT YET shipped at HEAD; G18-A wave-5a is PARTIAL per phase-3-backlog §4.2)", () => {
    // RE-DISPOSITION RATIONALE (pre-v1 Class A un-ignore, 2026-05-10):
    //
    // G18-A wave-5a (PR #114) shipped PARTIAL per phase-3-backlog §4.2
    // closure narrative: schema + handler scaffolding landed at
    // `bindings/napi/src/browser_indexeddb.rs` +
    // `bindings/napi/src/browser_blob_store.rs`; the wasm32 `web-sys`
    // / `js-sys` / `wasm-bindgen-futures` plumbing arms of
    // `apply_migration_step` + `close_database` are stubs.
    // `BrowserManifestStore::is_persistent()` returns `false` HONESTLY
    // until §4.3 G18-A-followup wires the wasm32 IDB plumbing
    // end-to-end. Page-reload survival requires both (a) the wasm32
    // IDB calls + (b) Playwright fixture authoring (or fake-indexeddb
    // under vitest) — neither at HEAD. Destination NAMED: phase-3-backlog
    // §4.3 (existing entry; covers both halves under one named scope).
    // Test stays `.skip` until §4.3 G18-A-followup lands; un-ignore
    // sits at the §4.3 implementer-wave's LANDING boundary, not at
    // some phantom "next orchestrator-direct fix-pass batch."
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
    throw new Error(
      "RED-PHASE: G18-A wave-5a wires page-reload survival end-to-end (Playwright cell or fake-indexeddb under vitest) + drops .skip + un-comments assertions",
    );
  });
});
