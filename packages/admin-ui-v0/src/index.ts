// Phase-4-Foundation R3 RED-PHASE stub for the admin UI v0 package.
//
// This module is the entrypoint that the wasm32-unknown-unknown bundle
// imports + the Tauri 2.x embedded webview loads. At R3 the surface is
// a placeholder so the F2 test pins compile against a real module
// rather than a missing import. G24-A wave-6 + G24-B/C wave-6b fill
// the real component tree:
//
// - 4-category navigation IA (Plugins / Workflows / Content Types / Views)
//   per `docs/ADMIN-UI.md` §2.
// - Install consent flow (Layer 2 per CLAUDE.md baked-in #18).
// - Workflow editor (drag-drop primitive picker; schema-driven form gen).
// - Composed-view creator (anchor pattern + projection; live-preview via
//   SUBSCRIBE seam).
// - Plugin library subgraph + active-version reference manager.
//
// The renderer-backend swappability pattern (CLAUDE.md #17) means this
// module does NOT import a specific renderer; the runtime injects a
// `Renderer` impl via the bootstrap surface (BrowserRender for the
// browser-tab shape; TauriRender from `benten-renderer-tauri` for the
// embedded-webview shape).

/**
 * Placeholder bootstrap entrypoint. G24-A wave-6 fills with the real
 * mount surface.
 *
 * @returns a tagged stub so consumers (and tests) can assert the module
 *   exports something at R3 RED-PHASE.
 */
export function placeholder(): { readonly stage: "r3-red-phase" } {
  return { stage: "r3-red-phase" };
}
