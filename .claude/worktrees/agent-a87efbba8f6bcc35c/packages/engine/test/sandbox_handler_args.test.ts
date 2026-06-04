// Phase-3 G17-C wave-5b — 24th p/c drift acceptance criterion (TS DSL
// camelCase → eval-side snake_case translation; pim-2 LOAD-BEARING;
// phase-3-backlog §6.6).
//
// ## What this pins
//
// G17-C ships `packages/engine/src/dsl.ts::translateSandboxArgs`
// (NEW, mirrors PR #76 `translateWaitArgs` precedent). The translator
// converts:
//
//   { wallclockMs: 100, outputLimitBytes: 4096 }   // camelCase, DSL surface
//
// to:
//
//   { wallclock_ms: 100, output_limit: 4096 }      // snake_case, napi argv
//
// before crossing the napi boundary. The Rust eval-side then reads
// snake_case correctly (existing surface — `primitive_host.rs:877`
// reads `op.properties.get("output_limit")`; `dsl-compiler/src/lib.rs:761+765`
// emits `output_limit: 65536`).
//
// ## R4-FP recalibration (r4-r1-wsa-1 BLOCKER)
//
// The TS DSL surface keeps `outputLimitBytes` (camelCase, with `Bytes`
// for type-clarity). The translation MUST drop the `Bytes` suffix to
// produce the canonical eval-side `output_limit` (NOT `output_limit_bytes`).
// At R3-D the translation target was authored as `output_limit_bytes`
// by symmetry with `wallclockMs` → `wallclock_ms`; r4-r1-wsa-1 caught
// this as the 25th p/c drift recurrence shape (eval-side reader silently
// drops the unknown key + OutputOverflow assertion passes by
// default-fallthrough rather than by ceiling enforcement). Recalibrated
// at R4-FP per the canonical eval-side property name verification.
//
// ## Pin shape — drives DSL surface to a buildable Subgraph
//
// These pins build a Subgraph through the production `subgraph(...)`
// DSL builder + assert the resulting SANDBOX node's properties bag
// carries snake_case keys (the translator's observable side-effect).
// A regression that forgets a translation site (or adds a new
// camelCase arg without updating the translator) leaves the camelCase
// keys verbatim in the props bag — caught here at unit-test cadence
// before the Vitest end-to-end Engine path runs.
//
// Pairs with `crates/benten-eval/tests/sandbox_handler_args.rs` (Rust
// eval-side observable end of the same round-trip — drives the
// eval-side `execute()` with snake_case keys + asserts the
// observable wallclock-trip + output-overflow boundaries).
//
// ## Pin sources
//
// - r2-test-landscape §2.5 G17-C
//   `sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
// - r2-test-landscape §3.D 24th p/c drift
// - phase-3-backlog §6.6 (24th p/c drift acceptance criterion)
// - PR #76 precedent (`translateWaitArgs`)

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, it, expect } from "vitest";
import { subgraph } from "@benten/engine";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

describe("G17-C 24th p/c drift — sandbox handler-args camelCase→snake_case round-trip", () => {
  it("translateSandboxArgs maps wallclockMs → wallclock_ms in the SANDBOX node properties bag", () => {
    // Pin source: r2-test-landscape §2.5 G17-C
    // `sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
    //
    // Build a Subgraph through the production DSL builder + introspect
    // the resulting SANDBOX node's properties bag. The wallclockMs
    // camelCase setting MUST appear under `wallclock_ms` (the
    // eval-side reader's canonical key) — the translator dropped the
    // camelCase mid-conversion before the props crossed into the
    // sandbox node's `properties` BTreeMap.
    const sg = subgraph("test")
      .action("run")
      .sandbox({
        module: "echo:identity",
        wallclockMs: 100,
        outputLimitBytes: 4096,
        fuel: 100_000,
      })
      .respond({ body: "$result" })
      .build();

    const sandboxNode = sg.nodes.find((n) => n.primitive === "sandbox");
    expect(sandboxNode).toBeDefined();
    const props = sandboxNode!.args as Record<string, unknown>;

    // Translator wrote snake_case keys:
    expect(props.wallclock_ms).toBe(100);

    // The camelCase key MUST NOT appear (otherwise the eval-side
    // reader silently defaults to 30 seconds — the failure shape
    // pim-2 catches):
    expect(props).not.toHaveProperty("wallclockMs");

    // OBSERVABLE consequence: the DSL camelCase wallclockMs setting
    // is OBSERVED at the SANDBOX-node-properties boundary as the
    // canonical snake_case `wallclock_ms` key. A regression that
    // forgets to call `translateSandboxArgs` (or adds a new arg
    // without updating the translator) silently widens the ceiling
    // to default and fails this expectation. The full round-trip
    // through the eval-side wallclock-trip boundary is pinned at
    // `crates/benten-eval/tests/sandbox_handler_args.rs`.
  });

  it("translateSandboxArgs maps outputLimitBytes → output_limit (DROPS `Bytes` per r4-r1-wsa-1)", () => {
    // Pin source: r2-test-landscape §2.5 G17-C
    // `sandbox_per_handler_output_limit_camel_case_dsl_round_trips`
    // (RECALIBRATED at R4-FP per r4-r1-wsa-1 BLOCKER — canonical
    // eval-side property is `output_limit`, NOT `output_limit_bytes`).
    const sg = subgraph("test")
      .action("run")
      .sandbox({
        module: "echo:identity",
        outputLimitBytes: 4096,
      })
      .respond({ body: "$result" })
      .build();

    const sandboxNode = sg.nodes.find((n) => n.primitive === "sandbox");
    expect(sandboxNode).toBeDefined();
    const props = sandboxNode!.args as Record<string, unknown>;

    // Translator dropped `Bytes` to match the canonical eval-side
    // key (`primitive_host.rs:877` reads
    // `op.properties.get("output_limit")`):
    expect(props.output_limit).toBe(4096);

    // NEITHER the camelCase original NOR the wrong-snake_case
    // `output_limit_bytes` drift form may appear — both would
    // bypass the eval-side reader (1 MB default takes over) and
    // silently widen the ceiling.
    expect(props).not.toHaveProperty("outputLimitBytes");
    expect(props).not.toHaveProperty("output_limit_bytes");

    // OBSERVABLE consequence: defends against the failure shape
    // "translator covers wallclockMs but forgets outputLimitBytes"
    // (or vice versa) AND "translator emits the WRONG snake_case
    // target like `output_limit_bytes` that eval silently ignores"
    // (the 25th p/c drift recurrence shape per r4-r1-wsa-1). The
    // full round-trip through the eval-side output-overflow
    // boundary is pinned at
    // `crates/benten-eval/tests/sandbox_handler_args.rs`.
  });

  it("translateSandboxArgs preserves verbatim keys (module, fuel, caps, input)", () => {
    // The non-translation keys MUST pass through unchanged — they
    // already match the eval-side canonical names. A regression
    // that accidentally re-cases one of these (e.g. mapping
    // `module` → `module_cid`) would break the SANDBOX dispatch
    // entirely.
    const sg = subgraph("test")
      .action("run")
      .sandbox({
        module: "bafy_module_cid_or_named_lookup",
        caps: ["host:compute:time"],
        input: "$input",
        fuel: 250_000,
      })
      .respond({ body: "$result" })
      .build();

    const sandboxNode = sg.nodes.find((n) => n.primitive === "sandbox");
    expect(sandboxNode).toBeDefined();
    const props = sandboxNode!.args as Record<string, unknown>;

    expect(props.module).toBe("bafy_module_cid_or_named_lookup");
    expect(props.caps).toEqual(["host:compute:time"]);
    expect(props.input).toBe("$input");
    expect(props.fuel).toBe(250_000);
  });

  it("sandbox.test.ts existing .skip'd tests re-pinned to production-flow shape", () => {
    // Pin source: r2-test-landscape §2.5 G17-C
    // `sandbox_test_skips_re_pinned_to_production_flow_shape`.
    //
    // The pre-G17-C `packages/engine/test/sandbox.test.ts` carried
    // 2 `.skip`'d tests pending Phase-3 module-bytes registration via
    // `engine.registerModuleBytes` plus 1 `.skip`'d test at
    // `install_module.test.ts::"engine.uninstallModule(cid) clean release"`.
    // G17-C un-skips them WITH bodies that drive the PRODUCTION FLOW
    // (DSL → engine.installModule → engine.registerSubgraph →
    // observable typed error rejection on uninstalled-name path), NOT
    // just stub-call assertions.
    //
    // OBSERVABLE consequence: the un-skip lands AND the un-skipped
    // tests drive production flow. A regression that re-introduces
    // `.skip(` for these named tests is caught here at unit-test
    // cadence.
    const sandboxTestSrc = readFileSync(
      resolve(__dirname, "sandbox.test.ts"),
      "utf8",
    );
    const installTestSrc = readFileSync(
      resolve(__dirname, "install_module.test.ts"),
      "utf8",
    );

    // The 3 originally-skipped tests are now `it(...)` not `it.skip(...)`.
    // Reference each by its descriptive name fragment so a future
    // rename of the test description trips this pin if the un-skip
    // is reverted.
    expect(sandboxTestSrc).toMatch(
      /it\(\s*"compose SANDBOX inside a handler subgraph/,
    );
    expect(sandboxTestSrc).not.toMatch(
      /it\.skip\(\s*"compose SANDBOX inside a handler subgraph/,
    );
    expect(sandboxTestSrc).toMatch(
      /it\(\s*"registerSubgraph rejects unresolved SANDBOX manifest/,
    );

    expect(installTestSrc).toMatch(
      /it\(\s*"engine\.uninstallModule\(cid\) clean release"/,
    );
    expect(installTestSrc).not.toMatch(
      /it\.skip\(\s*"engine\.uninstallModule\(cid\) clean release"/,
    );

    // The un-skipped tests drive the production flow — registerSubgraph
    // surfaces in both files (positive resolution + negative
    // rejection paths):
    expect(sandboxTestSrc).toMatch(/registerSubgraph/);
    expect(installTestSrc).toMatch(/registerSubgraph/);
  });
});
