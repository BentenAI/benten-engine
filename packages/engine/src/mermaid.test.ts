// Phase 1 R3 Vitest: handler.toMermaid() produces parseable Mermaid.
// Exit-criterion #5.
//
// G8 fix-pass note: the original test used `@mermaid-js/parser` to
// authoritatively parse the output, but `@mermaid-js/parser@0.6` does
// not ship a `flowchart` parser — its exported overloads are
// `info | packet | pie | architecture | gitGraph | radar | treemap`.
// We replaced the parser-based assertion with a structural regex check
// that verifies the output starts with `flowchart <dir>`, declares at
// least one labeled node, and contains at least one `-->` edge.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, crud } from "@benten/engine";

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

    // Structural flowchart shape check (replaces the parser-based
    // assertion — see top-of-file note).
    expect(mermaid).toMatch(/^flowchart (TD|LR|TB|BT|RL)\b/m);
    expect(mermaid).toMatch(/-->/); // at least one edge
    expect(mermaid).toMatch(/\[.*\]/); // at least one labeled node
  });

  it("mermaid_output_is_pure_and_deterministic", async () => {
    // toMermaid is a pure function over subgraph structure.
    const handler = await engine.registerSubgraph(crud("post"));
    const a = handler.toMermaid();
    const b = handler.toMermaid();
    expect(a).toBe(b);
  });

  // Phase 2a R4 qa-r4-8 / dx-r1-9: WAIT renders as a stadium-shape node
  // (`([text])`) plus a dashed resume edge labelled `on resume`. TDD
  // red-phase: `packages/engine/src/mermaid.ts` does NOT yet specialise
  // WAIT (falls through to rectangle); G3-B lands the dashed-edge shape.
  it("wait_renders_as_stadium_with_dashed_resume_edge", async () => {
    const { subgraph } = await import("@benten/engine");
    const waitHandler = await engine.registerSubgraph(
      subgraph("mermaid-wait")
        .action("run")
        .wait({ signal: "external:continue" })
        .respond({ body: "$result" })
        .build(),
    );
    const mermaid = waitHandler.toMermaid();

    // Stadium shape `([text])` — Mermaid's "stadium" variant. Phase-1
    // uses it only for RESPOND/EMIT; G3-B adds WAIT.
    expect(mermaid).toMatch(/\(\[\s*WAIT:/);
    // Dashed resume edge `-.->` labelled `on resume` — explicit signal
    // that this edge fires post-suspend.
    expect(mermaid).toMatch(/-\.->\s*\|?on resume/);
  });
});
