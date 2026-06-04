// SANDBOX example handler — pure DSL (Phase 2b G7).
//
// `summarize` reads a document by id, hands its body to a
// pre-installed WASM module via SANDBOX, persists the resulting
// summary, and responds with the summary text.
//
// The handler is composed exactly like a non-SANDBOX handler — the
// SANDBOX node is one primitive among READ / WRITE / RESPOND. The
// only sandbox-specific bit is the `module` reference (resolved
// against the named-manifest registry; format `<manifest>:<module>`).
//
// SANDBOX is **composition-only** — there is no top-level
// `engine.sandbox(...)` API. A SANDBOX node always lives inside a
// handler so capability resolution + Inv-4 nest-depth + Inv-14
// attribution chaining all flow through the evaluator.

import { subgraph } from "@benten/engine";
import type { Subgraph } from "@benten/engine";

/**
 * Build the SANDBOX summarisation handler.
 *
 * @param moduleRef Either a `<manifestName>:<moduleName>` name (resolved
 *                  via the named-manifest registry) or a raw module CID
 *                  (used together with an explicit `caps` array).
 */
export function buildSandboxHandler(moduleRef: string): Subgraph {
  return subgraph("summarize")
    .read({ label: "doc", by: "id", value: "$input.doc_id", as: "doc" })
    .sandbox({
      module: moduleRef, // e.g. "example.summarizer:summarize-v1"
      fuel: 1_000_000, // wasmtime fuel cap (per-call)
      wallclockMs: 30_000, // hard wallclock kill (per-call)
      outputLimitBytes: 1_048_576, // Inv-7 ceiling (per-call)
    })
    .write({ label: "summary" })
    .respond({ body: "$result" })
    .build();
}

export const sandboxHandlerId = "summarize";
export const sandboxHandlerAction = "default";
