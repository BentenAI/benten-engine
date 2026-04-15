// Phase 1 R3 Vitest: handler.toMermaid() produces parseable Mermaid.
// Exit-criterion #5. Uses @mermaid-js/parser as dev-dep per plan §1.
// Status: FAILING until B6 + E7 land.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, crud } from "@benten/engine";
import { parse as mermaidParse } from "@mermaid-js/parser";

let engine: Engine;
let tmp: string;

beforeAll(async () => {
  tmp = mkdtempSync(join(tmpdir(), "benten-mermaid-"));
  engine = await Engine.open(join(tmp, "benten.redb"));
});

afterAll(async () => {
  await engine.close();
  rmSync(tmp, { recursive: true, force: true });
});

describe("handler.toMermaid()", () => {
  it("mermaid_output_parses_as_flowchart", async () => {
    // Exit-criterion #5.
    const handler = await engine.registerSubgraph(crud("post"));
    const mermaid = handler.toMermaid();

    expect(mermaid).toMatch(/^flowchart /);
    expect(mermaid).toContain("-->");

    // Authoritative grammar check: if @mermaid-js/parser accepts it, Mermaid renders it.
    // R4 triage (m17): removed the `??` fallback — if the parser throws or
    // returns an unexpected shape, let the exception propagate so the test
    // fails cleanly rather than silently passing on a wrong-API call.
    const parsed = await mermaidParse("flowchart", mermaid);
    expect(parsed).toBeTruthy();
    const asParsed = parsed as { lexerErrors?: unknown[]; parserErrors?: unknown[] };
    expect(asParsed.lexerErrors).toBeDefined();
    expect(asParsed.parserErrors).toBeDefined();
    expect(asParsed.lexerErrors!).toHaveLength(0);
    expect(asParsed.parserErrors!).toHaveLength(0);
  });

  it("mermaid_output_is_pure_and_deterministic", async () => {
    // toMermaid is a pure function over subgraph structure.
    const handler = await engine.registerSubgraph(crud("post"));
    const a = handler.toMermaid();
    const b = handler.toMermaid();
    expect(a).toBe(b);
  });
});
