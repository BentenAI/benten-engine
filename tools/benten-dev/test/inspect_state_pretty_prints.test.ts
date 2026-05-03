// Phase 2a R3 Vitest — `benten-dev inspect-state <path>` pretty-prints a
// suspended ExecutionState.
//
// Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G11-A
// (`benten-dev inspect-state` tool — deferred DX-R1 item, ergonomic
// suspended-state pretty-printer) + §9.1 "Debuggability concern (Option C
// motivation) addressed separately: `benten-dev inspect-state <path>`
// ships in G11 (dev-server group) as a pretty-print command. JSON-
// equivalent readability without paying a format cost."
//
// Status: this CLI surface (`tools/benten-dev/bin/benten-dev.mjs`) is
// not yet shipped; the Rust-side pretty-printer entry point at
// `tools/benten-dev/src/inspect_state.rs::pretty_print_envelope_bytes`
// IS shipped, but the wrapping `node bin/benten-dev.mjs` thin-CLI
// front-door lands in Phase 3 per `docs/future/phase-3-backlog.md` §6.9
// (benten-dev `inspect-state` thin-CLI front-door). These tests stay
// `it.skip`'d until that ships — the JS-side hot-reload harness in
// `devserver.test.ts` + `hotreload_preserves_cap_grants.test.ts` is the
// load-bearing Wave-8f surface. HARD RULE compliance: destination
// exists at phase-3-backlog §6.9 with concrete Phase-3 deliverables.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, subgraph } from "@benten/engine";

describe.skip("benten-dev inspect-state", () => {
  let tmp: string;
  let bytesPath: string;
  let engine: Engine;

  beforeAll(async () => {
    tmp = mkdtempSync(join(tmpdir(), "benten-inspect-"));
    engine = await Engine.open(join(tmp, "benten.redb"));
    // Suspend a handler and write the bytes to disk for the CLI to read.
    const handler = await engine.registerSubgraph(
      subgraph("inspect-test")
        .action("run")
        .wait({ signal: "external:probe" })
        .respond({ body: "$result" })
        .build(),
    );
    const outcome = await engine.callWithSuspension(handler.id, "run", {});
    if (outcome.kind !== "suspended") {
      throw new Error("expected suspended outcome");
    }
    bytesPath = join(tmp, "suspended.cbor");
    writeFileSync(bytesPath, outcome.handle);
  });

  afterAll(async () => {
    await engine.close();
    rmSync(tmp, { recursive: true, force: true });
  });

  it.skip("inspect_state_pretty_prints_envelope_shape", () => {
    // CLI binary path — G11-A publishes this at
    // `tools/benten-dev/bin/benten-dev.mjs`.
    const cliPath = join(
      __dirname,
      "..",
      "bin",
      "benten-dev.mjs",
    );
    const output = execFileSync(
      "node",
      [cliPath, "inspect-state", bytesPath],
      { encoding: "utf8" },
    );
    // The pretty-printer must surface each documented envelope field
    // (plan §9.1 ExecutionStateEnvelope).
    expect(output).toMatch(/schema_version\s*[:=]\s*1/);
    expect(output).toMatch(/payload_cid\s*[:=]/);
    expect(output).toMatch(/attribution_chain/);
    expect(output).toMatch(/pinned_subgraph_cids/);
    expect(output).toMatch(/context_binding_snapshots/);
    expect(output).toMatch(/resumption_principal_cid/);
    expect(output).toMatch(/frame_stack/);
    expect(output).toMatch(/frame_index/);
  });

  it.skip("inspect_state_surfaces_resume_protocol_hints", () => {
    // The pretty-printer should also echo the 4-step resume protocol
    // headers so operators know what to check against: payload_cid
    // recomputation, resumption_principal match, pinned_subgraph re-
    // verification, and check_write re-call (plan §9.1).
    const cliPath = join(
      __dirname,
      "..",
      "bin",
      "benten-dev.mjs",
    );
    const output = execFileSync(
      "node",
      [cliPath, "inspect-state", bytesPath, "--with-protocol-hints"],
      { encoding: "utf8" },
    );
    expect(output).toMatch(/payload_cid recomputation/i);
    expect(output).toMatch(/resumption_principal/i);
    expect(output).toMatch(/pinned_subgraph_cids re-verification/i);
    expect(output).toMatch(/check_write/i);
  });

  it.skip("inspect_state_rejects_nonexistent_path_with_typed_exit", () => {
    const cliPath = join(
      __dirname,
      "..",
      "bin",
      "benten-dev.mjs",
    );
    expect(() =>
      execFileSync("node", [cliPath, "inspect-state", "/tmp/does-not-exist"], {
        encoding: "utf8",
        stdio: "pipe",
      }),
    ).toThrow();
  });
});
