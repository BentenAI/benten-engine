// G19-D 6 TS DSL Args drifts — pure-DSL round-trip pins
// (wave-7 parallel; §7.9 + r1-napi-3 + D-PHASE-3-29).
//
// Each Args interface lands a `translateXxxArgs` helper at the DSL→
// OperationNode-property-bag boundary. These tests assert the
// post-translation property bag carries the eval-side canonical keys —
// the same structural defense the Rust-side LOAD-BEARING parity
// meta-test pins at compile/cargo-test time.
//
// Why pure-DSL not engine-end-to-end:
//
//   The 6 *Args drifts close at the DSL spread layer (translators
//   inside `packages/engine/src/dsl.ts`). Verifying the spread is
//   correct does NOT require booting the Rust engine — `subgraph(...).
//   <primitive>(args).build()` returns a JSON-serializable Subgraph
//   whose nodes carry the post-translation args bag. A pure-DSL test
//   asserts the args bag has the eval-side canonical keys; the
//   Rust-side LOAD-BEARING parity meta-test
//   (`crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs`)
//   independently asserts the eval-side canonical keys are read by the
//   primitive's execute().
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-D):
//
//   - tests/branch_args_ts_dsl_round_trip_to_eval_properties (× 6 Args drifts)

import { describe, it, expect } from "vitest";
import { subgraph } from "@benten/engine";

describe("G19-D 6 TS DSL Args drifts pure-DSL round-trip (§7.9 + r1-napi-3)", () => {
  it("RespondArgs translates to eval-side body / status keyspace", () => {
    // r1-napi-3 (a) pin: RespondArgs(body / edge / status) → eval(status / body).
    //
    // RESPOND is the closest-to-no-drift Args interface — body + status
    // translate verbatim. The `edge` hint is by-design omitted from the
    // property bag (its surface lives on the edge table; consumed by the
    // engine compile path as outgoing edge stamping, not properties-bag).
    const sg = subgraph("respond-test")
      .respond({ body: "$result", status: 200, edge: "OK" })
      .build();
    const respondNode = sg.nodes.find((n) => n.primitive === "respond");
    expect(respondNode).toBeDefined();
    expect(respondNode!.args).toMatchObject({
      body: "$result",
      status: 200,
    });
    // edge is by-design NOT in the args bag (edge-table-driven routing):
    expect(respondNode!.args).not.toHaveProperty("edge");
  });

  it("ReadArgs translates DSL `by: \"id\"`/`\"cid\"` to eval-side query_kind/target_cid", () => {
    // r1-napi-3 (b) pin: ReadArgs(label / by / value / as) →
    // eval(query_kind / target_cid / label) — DSL-compiler-bypass drift fix.
    const sg = subgraph("read-test")
      .read({ label: "post", by: "id", value: "post:1" })
      .build();
    const readNode = sg.nodes.find((n) => n.primitive === "read");
    expect(readNode).toBeDefined();
    expect(readNode!.args).toMatchObject({
      label: "post",
      query_kind: "by_cid",
      target_cid: "post:1",
    });
    // Raw DSL fields MUST NOT appear in the property bag (silent value-loss
    // shape if they did — the eval-side reader would read them).
    expect(readNode!.args).not.toHaveProperty("by");
    expect(readNode!.args).not.toHaveProperty("value");
    expect(readNode!.args).not.toHaveProperty("as");
  });

  it("ReadArgs `by: \"_listView\"` translates to eval-side query_kind: \"list_view\"", () => {
    const sg = subgraph("list-test")
      .read({ label: "post", by: "_listView" })
      .build();
    const readNode = sg.nodes.find((n) => n.primitive === "read");
    expect(readNode!.args).toMatchObject({
      label: "post",
      query_kind: "list_view",
    });
  });

  it("BranchArgs translates `on` → eval-side match_value", () => {
    // r1-napi-3 (c) pin: BranchArgs(on) →
    // eval(match_value / condition_value / cases / has_default / conditions).
    // Compile-path supplies cases/has_default/conditions from the edge
    // table — this test asserts the match_value translation only.
    const sg = subgraph("branch-test")
      .branch({ on: "$input.kind" })
      .case("post", (s) => s.respond({ body: "post" }))
      .endBranch()
      .build();
    const branchNode = sg.nodes.find((n) => n.primitive === "branch");
    expect(branchNode).toBeDefined();
    expect(branchNode!.args).toMatchObject({
      match_value: "$input.kind",
    });
    expect(branchNode!.args).not.toHaveProperty("on");
  });

  it("IterateArgs translates `over` → eval-side items, preserves max", () => {
    // r1-napi-3 (d) pin: IterateArgs(over / max) → eval(items / max).
    const sg = subgraph("iterate-test")
      .iterate({ over: "$input.list", max: 100 })
      .respond()
      .build();
    const iterNode = sg.nodes.find((n) => n.primitive === "iterate");
    expect(iterNode).toBeDefined();
    expect(iterNode!.args).toMatchObject({
      items: "$input.list",
      max: 100,
    });
    expect(iterNode!.args).not.toHaveProperty("over");
  });

  it("CallArgs translates handler/action/isolated to eval-side target/call_op/child_scope", () => {
    // r1-napi-3 (e) pin: CallArgs(handler / action / input / isolated) →
    // eval(child_scope / parent_scope / target / call_op / requires / timeout_ms).
    const sg = subgraph("parent-test")
      .call({
        handler: "child-handler",
        action: "child:main",
        input: "$input",
        isolated: true,
      })
      .respond()
      .build();
    const callNode = sg.nodes.find((n) => n.primitive === "call");
    expect(callNode).toBeDefined();
    expect(callNode!.args).toMatchObject({
      target: "child-handler",
      call_op: "child:main",
      input: "$input",
      child_scope: true,
    });
    // Raw DSL fields MUST NOT appear:
    expect(callNode!.args).not.toHaveProperty("handler");
    expect(callNode!.args).not.toHaveProperty("action");
    expect(callNode!.args).not.toHaveProperty("isolated");
  });

  it("CallArgs without isolated flag omits child_scope (default scope-inheritance)", () => {
    const sg = subgraph("non-isolated-call")
      .call({ handler: "child", action: "main" })
      .respond()
      .build();
    const callNode = sg.nodes.find((n) => n.primitive === "call");
    // child_scope absent (eval-side default = inherit parent scope).
    expect(callNode!.args).not.toHaveProperty("child_scope");
  });

  it("TransformArgs translates `as` → eval-side result, preserves expr", () => {
    // r1-napi-3 (f) pin: TransformArgs(expr / as) → eval(expr / input / result).
    // Compile-path supplies `input` from upstream binding.
    const sg = subgraph("transform-test")
      .transform({ expr: "$input.x * 2", as: "doubled" })
      .respond()
      .build();
    const transformNode = sg.nodes.find((n) => n.primitive === "transform");
    expect(transformNode).toBeDefined();
    expect(transformNode!.args).toMatchObject({
      expr: "$input.x * 2",
      result: "doubled",
    });
    expect(transformNode!.args).not.toHaveProperty("as");
  });

  it("SubscribeArgs translates `event`/`handler` to eval-side pattern/handler (G14-D handler-id-router)", () => {
    // §7.10 + r1-napi-3 SubscribeArgs.handler? re-introduction (post-G14-D
    // wave-5a handler-id-router seam). The DSL surface field name is `event`
    // (developer ergonomics); the eval-side primitive reads `pattern` per
    // primitives/subscribe.rs::execute line 1282.
    const sg = subgraph("subscribe-test")
      .subscribe({ event: "post:created", handler: "post-created-handler" })
      .respond()
      .build();
    const subNode = sg.nodes.find((n) => n.primitive === "subscribe");
    expect(subNode).toBeDefined();
    expect(subNode!.args).toMatchObject({
      pattern: "post:created",
      handler: "post-created-handler",
    });
    expect(subNode!.args).not.toHaveProperty("event");
  });

  it("SubscribeArgs without handler omits the field (default broadcast fan-out)", () => {
    // SubscribeArgs.handler? is OPTIONAL — when unset the eval-side
    // SUBSCRIBE primitive defaults to broadcast fan-out.
    const sg = subgraph("subscribe-default")
      .subscribe({ event: "post:created" })
      .respond()
      .build();
    const subNode = sg.nodes.find((n) => n.primitive === "subscribe");
    expect(subNode!.args).toMatchObject({
      pattern: "post:created",
    });
    expect(subNode!.args).not.toHaveProperty("handler");
  });
});

describe("G19-D parity defense — translator output is structural", () => {
  it("CaseBuilder uses the same translators as SubgraphBuilder (lockstep contract)", () => {
    // The CaseBuilder (used inside `.branch(...).case(...)`) and the
    // top-level SubgraphBuilder MUST route through the same translator
    // helpers — otherwise a BRANCH case body would carry a different
    // property-bag shape than a top-level node, fracturing the structural
    // parity guarantee. Verify by building both paths with the same args
    // and asserting the resulting nodes have identical args.
    const topLevel = subgraph("top")
      .read({ label: "post", by: "id", value: "post:1" })
      .build();
    const inCase = subgraph("case-host")
      .branch({ on: "$.kind" })
      .case("read-it", (s) => s.read({ label: "post", by: "id", value: "post:1" }))
      .endBranch()
      .build();
    const topRead = topLevel.nodes.find((n) => n.primitive === "read");
    const caseRead = inCase.nodes.find((n) => n.primitive === "read");
    // Both READ nodes must have IDENTICAL args (post-translation shape):
    expect(caseRead!.args).toEqual(topRead!.args);
  });
});
