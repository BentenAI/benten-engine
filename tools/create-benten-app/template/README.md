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

- `src/handlers.ts` — `crud('post')` zero-config handler. Edit this to add capabilities, schemas, or custom TRANSFORM logic. See `docs/DSL-SPECIFICATION.md` in the Benten repo for the full options shape.
- `src/index.ts` — entry point. Opens the engine, registers the handlers, logs the registered handler id.
- `test/smoke.test.ts` — Phase 1 exit-criterion smoke tests. Six `it()` blocks:
  1. `register_succeeds`
  2. `three_creates_list_returns_them`
  3. `cap_denial_routes_on_denied`
  4. `trace_non_zero_timing`
  5. `mermaid_output_parses`
  6. `ts_rust_cid_roundtrip`

## Next steps

- **Add a capability.** Open the engine with the grant-backed policy and stamp a `capability` on your CRUD handler:

  ```ts
  import { Engine, PolicyKind, crud } from "@benten/engine";
  const engine = await Engine.openWithPolicy("./.benten/{{name}}.redb", PolicyKind.GrantBacked);
  const handler = await engine.registerSubgraph(crud("post", { capability: "store:post:write" }));
  await engine.grantCapability({ actor: "alice", scope: "store:post:write" });
  await engine.call(handler.id, "post:create", { title: "x" });
  ```

  Unrevoked `system:CapabilityGrant` Nodes authorize matching writes; `engine.revokeCapability(grantCid, "alice")` turns them off.
- **Inspect the subgraph.** `console.log(handler.toMermaid())` renders the handler as a Mermaid flowchart — paste into any Mermaid renderer.
- **Trace an evaluation.** `engine.trace(handler.id, 'post:create', { title: 'x' })` returns per-node timings and (when the native tracer surfaces them) per-step inputs / outputs.

See `docs/QUICKSTART.md` in the Benten repo for the full walkthrough.
