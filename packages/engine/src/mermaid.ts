// Render a Subgraph as a Mermaid `flowchart` diagram.
//
// Pure function. No runtime dependency on `@benten/engine-native`.
// Output conforms to the Mermaid flowchart grammar (verified in
// `mermaid.test.ts` using `@mermaid-js/parser`).
//
// Shape: one node per subgraph primitive, one directed edge per
// outgoing edge. Edge labels are preserved (`NEXT`, `CASE:<value>`,
// `ON_NOT_FOUND`, etc.). Node shapes are chosen per primitive to aid
// readability when rendered — Mermaid otherwise renders every node as
// a plain rectangle.
//
// Node-id rule: Mermaid ids must be alphanumeric-ish. We sanitize the
// DSL node ids (e.g. `read-3`) into `n_read_3` so Mermaid accepts them
// regardless of whether the DSL id scheme changes.

import type { Subgraph, SubgraphNode } from "./types.js";

/**
 * Render the given `Subgraph` as a Mermaid flowchart string.
 *
 * Guarantees:
 *   * Output starts with `flowchart TD\n`.
 *   * Contains at least one `-->` edge when the subgraph has ≥2 linked
 *     nodes. (A one-node subgraph has no edges; we emit just the node
 *     declaration, still parseable.)
 *   * Stable across runs (no timestamps, no random ids).
 *   * Every node appears exactly once, regardless of how many edges
 *     reference it.
 */
export function toMermaid(sg: Subgraph): string {
  const lines: string[] = ["flowchart TD"];

  // Stable node ordering: sort by DSL id for deterministic output
  // independent of insertion order changes.
  const nodes = [...sg.nodes].sort((a, b) =>
    a.id < b.id ? -1 : a.id > b.id ? 1 : 0,
  );

  // Node declarations first, then edges. This ensures Mermaid knows
  // about every node before an edge references it.
  for (const n of nodes) {
    lines.push(`  ${mermaidId(n.id)}${nodeShape(n)}`);
  }

  // Edges in a stable order (by source id, then edge label).
  for (const n of nodes) {
    const edgeKeys = Object.keys(n.edges).sort();
    for (const edgeLabel of edgeKeys) {
      const targetId = n.edges[edgeLabel];
      const label = edgeLabel === "NEXT" ? "" : edgeLabel;
      if (label) {
        lines.push(
          `  ${mermaidId(n.id)} -->|${escapeLabel(label)}| ${mermaidId(targetId)}`,
        );
      } else {
        lines.push(`  ${mermaidId(n.id)} --> ${mermaidId(targetId)}`);
      }
    }
  }

  // Ensure trailing newline for a textual hash-stable output.
  return lines.join("\n") + "\n";
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function mermaidId(dslId: string): string {
  // Replace any character Mermaid may not accept in a bare id.
  // Alphanumerics + underscore is the safe subset.
  return "n_" + dslId.replace(/[^a-zA-Z0-9]/g, "_");
}

function nodeShape(n: SubgraphNode): string {
  const label = escapeLabel(`${n.primitive.toUpperCase()}: ${shortArgs(n)}`);
  switch (n.primitive) {
    case "branch":
      return `{${label}}`;
    case "respond":
    case "emit":
      return `([${label}])`;
    case "iterate":
      return `[/${label}/]`;
    case "call":
      return `[[${label}]]`;
    case "transform":
      return `>${label}]`;
    default:
      return `[${label}]`;
  }
}

function shortArgs(n: SubgraphNode): string {
  // Pick a couple of interesting keys per primitive for the label. Keep
  // the rendered label short — Mermaid struggles with extremely long
  // labels and the intent is at-a-glance visual, not full detail.
  const a = n.args;
  const pick = (k: string): string =>
    a[k] === undefined ? "" : String(a[k]);
  switch (n.primitive) {
    case "read":
      return [pick("label"), pick("by")].filter(Boolean).join(":");
    case "write":
      return pick("label");
    case "transform":
      return pick("expr").slice(0, 40);
    case "branch":
      return pick("on");
    case "iterate":
      return `${pick("over")} x${pick("max")}`;
    case "call":
      return [pick("handler"), pick("action")].filter(Boolean).join("/");
    case "respond":
      return pick("edge") || pick("body") || "";
    case "emit":
      return pick("event");
    case "wait":
      return pick("duration");
    case "stream":
      return pick("source");
    case "subscribe":
      return pick("event");
    case "sandbox":
      return pick("module");
    default:
      return "";
  }
}

function escapeLabel(s: string): string {
  // Mermaid labels don't tolerate quotes or closing brackets without
  // escaping. We strip aggressive punctuation rather than escaping —
  // the rendered label is already a short at-a-glance string.
  return s
    .replace(/"/g, "'")
    .replace(/[\]\[{}()|<>]/g, "")
    .trim();
}
