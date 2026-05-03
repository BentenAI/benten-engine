# {{name}}

Scaffolded with `create-benten-app`. This project runs a Benten graph engine locally, with a zero-config `crud('post')` handler registered in `src/handlers.ts`.

## The 10-minute path

```sh
cd {{name}}
npm install          # already run by the scaffolder unless you passed --skip-install
npm run test         # runs the six-gate smoke test
npm run dev          # registers the handler against a local .benten/{{name}}.redb
npm run build        # tsc -> dist/
```

## Files

- `src/handlers.ts` — `crud('post')` zero-config handler. Edit this to add capabilities, schemas, or custom TRANSFORM logic.
- `src/index.ts` — entry point. Opens the engine, registers the handlers, logs the registered handler id.
- `test/smoke.test.ts` — Phase 1 exit-criterion smoke tests. Six `it()` blocks:
  1. `register_succeeds`
  2. `three_creates_list_returns_them`
  3. `cap_denial_routes_on_denied`
  4. `trace_non_zero_timing`
  5. `mermaid_output_parses`
  6. `ts_rust_cid_roundtrip`

## Next steps

- **Add a capability.** Open the engine with the grant-backed policy and stamp a `capability` on your CRUD handler. `callAs` accepts a real CID or a friendly principal string — the napi layer hashes the latter into a deterministic synthetic CID so `"alice"` works without minting a Node first:

  ```ts
  import { Engine, PolicyKind, crud } from "@benten/engine";
  const engine = await Engine.openWithPolicy("./.benten/{{name}}.redb", PolicyKind.GrantBacked);
  const handler = await engine.registerSubgraph(
    crud("post", { capability: "store:post:*" }),
  );
  // A wildcard grant like `store:post:*` permits the derived concrete
  // scopes (`store:post:write`, `store:post:read`, …).
  await engine.grantCapability({ actor: "alice", scope: "store:post:*" });
  await engine.callAs(handler.id, "post:create", { title: "x" }, "alice");
  ```

  Unrevoked `system:CapabilityGrant` Nodes authorize matching writes; `engine.revokeCapability(grantCid, "alice")` turns them off.
- **Use the full CRUD surface.** `crud("post")` exposes five actions — `create`, `get`, `list`, `update`, `delete` — dispatched via `engine.call(handler.id, "post:<action>", input)`. The engine assigns `handler.id` as `crud:<label>` for `crud()`-registered handlers (see `Engine::register_crud` at `crates/benten-engine/src/engine.rs::register_crud` — symbol cite per `dispatch-conventions.md` §3.5b high-churn-surface preference; line cites in `engine.rs` drifted across waves 4-8). Capturing the returned handle and passing `handler.id` (as `src/index.ts` does) is the resilient pattern; future label-format changes won't break call sites. The action is the second argument.
- **Inspect the subgraph.** `console.log(handler.toMermaid())` renders the handler as a Mermaid flowchart — paste into any Mermaid renderer.
- **Trace an evaluation.** `engine.trace(handler.id, "post:create", { title: "x" })` returns per-node timings; each `step.nodeCid` cross-references a node id rendered by `handler.toMermaid()`. Traced calls do not persist the outcome and do not fire ChangeEvents.

See `docs/QUICKSTART.md` in the Benten repo for the full walkthrough.
