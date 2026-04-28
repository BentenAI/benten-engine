// Phase 2b G7-C — SANDBOX TS surface (DSL-composition only).
//
// This module owns ONE thing: the SANDBOX-related re-exports + helpers
// that operate purely against the DSL composition shape. The actual
// `subgraph(...).sandbox(args)` builder method lives in `./dsl.ts`
// alongside every other primitive helper — it stays there so a reader
// scanning the DSL surface sees all 12 primitives in one file. This
// file aggregates the SANDBOX-specific surface area:
//
//   - The SANDBOX argument types (`SandboxArgs`, `SandboxArgsByName`,
//     `SandboxArgsByCaps`) — re-exports from `./types.ts`.
//   - The SANDBOX result + descriptor types
//     (`SandboxResult`, `SandboxNodeDescription`).
//   - The DX helper `assertSandboxComposed` — a tiny runtime probe used
//     in Phase-3 P2P-routing call sites that need to prove a subgraph
//     contains a SANDBOX node before shipping it over the wire to a
//     Node-resident peer.
//
// **Surface contract (per dx-r1-2b SANDBOX, single-surface corrected):**
//
//   - SANDBOX is composed via `subgraph(...).sandbox({ module, manifest? | caps? })`
//     EXCLUSIVELY. There is no top-level `engine.sandbox(...)` method.
//     Composition-only because SANDBOX is a subgraph primitive
//     analogous to TRANSFORM, not a top-level CALL surface — exposing
//     `engine.sandbox` would bypass the evaluator (Inv-4 nest-depth,
//     Inv-14 attribution, capability gates).
//
//   - Module lifecycle (`engine.installModule(manifest, manifestCid)` /
//     `engine.uninstallModule(cid)` / `engine.computeManifestCid(manifest)`)
//     is SEPARATE — owned by G10-B in `engine.ts` directly. The
//     manifest install path doesn't execute wasm, so it stays present
//     even on browser builds where SANDBOX execution is gated by
//     `E_SANDBOX_UNAVAILABLE_ON_WASM`.
//
//   - Browser/wasm32 build behaviour: this module STAYS PRESENT — only
//     the executor-backed call path differs. `subgraph(...).sandbox(...)`
//     compiles + builds a valid Subgraph payload; calling the handler
//     locally on a wasm32 build surfaces `E_SANDBOX_UNAVAILABLE_ON_WASM`
//     at execution time (the moment the evaluator walk reaches the
//     SANDBOX node). See `packages/engine/test/wasm_browser_target.test.ts`.
//
// **Cross-references:**
//   - `./dsl.ts` — `SubgraphBuilder.sandbox(args)` + the top-level
//     `sandbox(args)` helper (one-shot Node builder).
//   - `./types.ts` — `SandboxArgs`, `SandboxArgsByName`,
//     `SandboxArgsByCaps`, `SandboxResult`, `SandboxNodeDescription`,
//     `ModuleManifest`, `ModuleManifestEntry`, `ManifestSignature`.
//   - `./engine.ts` — `Engine.targetSupportsSandbox()`,
//     `Engine.describeSandboxNode(handlerId, nodeId)`,
//     `Engine.installModule(manifest, manifestCid)` (G10-B),
//     `Engine.uninstallModule(cid)` (G10-B),
//     `Engine.computeManifestCid(manifest)` (G10-B).
//   - `docs/SANDBOX-LIMITS.md` — enforcement axes, default knobs,
//     wasm32 availability gate.
//   - `docs/HOST-FUNCTIONS.md` — host-fn surface; capability-derived.

import type {
  ManifestSignature,
  ModuleManifest,
  ModuleManifestEntry,
  Primitive,
  SandboxArgs,
  SandboxArgsByCaps,
  SandboxArgsByName,
  SandboxNodeDescription,
  SandboxResult,
  Subgraph,
  SubgraphNode,
} from "./types.js";

// ---------------------------------------------------------------------------
// Type re-exports
// ---------------------------------------------------------------------------

export type {
  ManifestSignature,
  ModuleManifest,
  ModuleManifestEntry,
  SandboxArgs,
  SandboxArgsByCaps,
  SandboxArgsByName,
  SandboxNodeDescription,
  SandboxResult,
};

// ---------------------------------------------------------------------------
// `assertSandboxComposed` — DX helper for Phase-3 P2P routing call sites
// ---------------------------------------------------------------------------

/**
 * Runtime probe — asserts that the supplied [`Subgraph`] contains at
 * least one SANDBOX node. Phase-3 P2P routing call sites in browser
 * builds use this before shipping the subgraph over the wire to a
 * Node-resident peer (the routing decision is made structurally:
 * "if the handler contains a SANDBOX node, route to a peer because the
 * local wasm32 build can't execute it").
 *
 * Returns the matching SANDBOX nodes for inspection. An empty array
 * means the subgraph is local-executable on this build regardless of
 * platform.
 *
 * **Why not just check `targetSupportsSandbox()` once at engine open?**
 * The platform answer is build-time fixed, but the *handler-by-handler*
 * routing decision is per-handler — a browser may register many
 * handlers and only need to route the SANDBOX-bearing ones. This helper
 * lets the routing layer decide per-handler without re-walking the
 * subgraph node-by-node.
 */
export function assertSandboxComposed(sg: Subgraph): SubgraphNode[] {
  if (!sg || !Array.isArray(sg.nodes)) {
    throw new TypeError(
      "assertSandboxComposed: argument must be a built Subgraph (call .build() on the SubgraphBuilder)",
    );
  }
  const sandboxNodes = sg.nodes.filter(
    (n): n is SubgraphNode => n.primitive === ("sandbox" as Primitive),
  );
  return sandboxNodes;
}

/**
 * Type-guard companion to [`assertSandboxComposed`]. Returns `true` when
 * the subgraph contains at least one SANDBOX node — the routing-layer
 * predicate without the matching-nodes return.
 */
export function isSandboxBearing(sg: Subgraph): boolean {
  return assertSandboxComposed(sg).length > 0;
}
