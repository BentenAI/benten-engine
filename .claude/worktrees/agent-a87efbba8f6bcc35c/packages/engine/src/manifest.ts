// Phase 2b G10-B — TypeScript-side module manifest surface.
//
// Re-exports the canonical `ModuleManifest` types (the source of truth
// for the cross-language shape lives in `./types.ts`) plus a thin
// `ManifestSummary` helper that mirrors the Rust
// `crate::module_manifest::ManifestSummary` rendering used in
// `E_MODULE_MANIFEST_CID_MISMATCH` error bodies.
//
// The top-level engine surface — `engine.installModule(manifest, cid)`,
// `engine.uninstallModule(cid)`, `engine.computeManifestCid(manifest)` —
// lives on the `Engine` class itself in `./engine.ts` per the
// dx-optimizer-corrected DSL surface (the SANDBOX top-level entries are
// exclusively manifest-lifecycle methods; SANDBOX execution itself
// composes through `subgraph(...).sandbox(...)`).
//
// ## Pin sources
//
// - `r1-security-auditor.json` D9 RESOLVED — canonical DAG-CBOR.
// - `r1-security-auditor.json` D16 RESOLVED-FURTHER — REQUIRED
//   `manifestCid` arg on `engine.installModule(manifest, manifestCid)`.
// - `packages/engine/test/manifest_schema_parity.test.ts` — the TS shape
//   MUST mirror the Rust `ModuleManifest` field-for-field.

export type {
  ManifestSignature,
  ModuleManifest,
  ModuleManifestEntry,
} from "./types.js";

/**
 * 1-line operator-readable summary of a [`ModuleManifest`] — mirrors
 * the Rust `ManifestSummary` rendering used in
 * `E_MODULE_MANIFEST_CID_MISMATCH` error bodies.
 *
 * Display shape: `<name> v<version> modules=<n> caps=<n>`.
 *
 * `caps` is the **deduplicated** count of unique `requires` strings
 * across every module entry (so the operator sees the manifest's
 * effective capability surface, not a per-module total).
 */
export interface ManifestSummary {
  name: string;
  version: string;
  modules: number;
  caps: number;
}

import type { ModuleManifest } from "./types.js";

/**
 * Build the [`ManifestSummary`] for a manifest — mirrors the Rust
 * `ModuleManifest::summary()` impl. Used for client-side display in
 * devtools / log lines without round-tripping through native.
 */
export function manifestSummary(manifest: ModuleManifest): ManifestSummary {
  const uniqueCaps = new Set<string>();
  for (const m of manifest.modules) {
    for (const r of m.requires) {
      uniqueCaps.add(r);
    }
  }
  return {
    name: manifest.name,
    version: manifest.version,
    modules: manifest.modules.length,
    caps: uniqueCaps.size,
  };
}

/**
 * Render a [`ManifestSummary`] in the canonical 1-line form that
 * mirrors the Rust `Display` impl — the form that appears in
 * `E_MODULE_MANIFEST_CID_MISMATCH` error bodies.
 */
export function renderManifestSummary(summary: ManifestSummary): string {
  return `${summary.name} v${summary.version} modules=${summary.modules} caps=${summary.caps}`;
}
