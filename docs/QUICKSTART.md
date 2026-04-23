# Quickstart

**Status:** Planning (engine code lands in Phase 1). This document specifies what the 10-minute developer experience will look like.

The goal is that a developer lands on the README, follows three commands, and has a working Benten instance creating and querying Nodes within 10 minutes. No Rust knowledge required. No community/governance/marketplace concepts required.

## The 10-minute path (target)

### 1. Install

```sh
npx create-benten-app my-app
cd my-app
npm install
npm run dev
```

`create-benten-app` will (when it ships) scaffold a TypeScript project with:
- `@benten/engine` (the napi-rs wrapped engine)
- A minimal handler registered via `crud()`
- A dev server with hot reload (`bentend dev` equivalent)
- A sample handler file to edit

### 2. Write your first handler

The zero-config path:

```typescript
import { crud } from "@benten/engine";

// Register a content type with defaults:
// - properties inferred from the caller-supplied input
// - `NoAuth` policy (no grants required)
// - local storage only
export const postHandlers = crud("post");
```

That's the simplest useful Benten code. `crud("post")` exposes `create`, `get`, `list`, `update`, and `delete` actions with sensible defaults.

### 3. Use it

```typescript
import { Engine } from "@benten/engine";
import { postHandlers } from "./handlers.js";

const engine = await Engine.open(".benten/my-app.redb");
const handler = await engine.registerSubgraph(postHandlers);
// `handler.id` is `"crud:post"` — a stable content-addressed handler id.

// Create
const created = await engine.call(handler.id, "post:create", {
  title: "Hello Benten",
  body: "First post.",
});
console.log(created.cid);

// List
const listed = await engine.call(handler.id, "post:list", {});
console.log(listed.items);

// Update
await engine.call(handler.id, "post:update", {
  cid: created.cid,
  patch: { body: "Edited body." },
});

// Delete
await engine.call(handler.id, "post:delete", { cid: created.cid });
```

## Adding capabilities (when you're ready)

When you need authentication, open the engine with the grant-backed policy and stamp a `capability` on the CRUD handler:

```typescript
import { Engine, PolicyKind, crud } from "@benten/engine";

const engine = await Engine.openWithPolicy(
  ".benten/my-app.redb",
  PolicyKind.GrantBacked,
);
const handler = await engine.registerSubgraph(
  crud("post", { capability: "store:post:*" }),
);

// Grant the wildcard capability — permits create / update / delete under
// the `post` label because `store:post:*` attenuates to the concrete
// `store:post:write` the engine derives at commit time.
await engine.grantCapability({ actor: "alice", scope: "store:post:*" });

// `callAs` accepts either a real CID or a friendly principal string; the
// friendly string is hashed into a deterministic synthetic CID.
await engine.callAs(handler.id, "post:create", { title: "x" }, "alice");
```

The engine ships with `PolicyKind.NoAuth` by default (every call permitted); swap in `PolicyKind.GrantBacked` when you want the revocation-aware Phase-1 policy. UCAN backends land in Phase 3.

## Diagnosing denied reads (Option C)

Under the grant-backed policy, a denied read returns `null` from `engine.getNode(cid)` — byte-identical with a genuine miss. That's the Phase-1 posture for named compromise #2 (see `docs/SECURITY-POSTURE.md`): an unauthorised caller cannot fish existence out of the CID space.

If you're the operator and need to tell "denied" apart from "not found" (to debug a missing grant, say), grant yourself the `debug:read` capability and use `engine.diagnoseRead`:

```typescript
await engine.grantCapability({ actor: "alice", scope: "store:debug:read" });

const info = await engine.diagnoseRead(cid);
// { cid, existsInBackend: boolean, deniedByPolicy: string | null, notFound: boolean }
if (info.notFound) {
  console.log("never written (or deleted)");
} else if (info.deniedByPolicy) {
  console.log(`exists, missing grant for ${info.deniedByPolicy}`);
} else {
  console.log("exists and is readable");
}
```

Without `store:debug:read`, `diagnoseRead` throws `E_CAP_DENIED` — so ordinary callers still cannot distinguish the two cases. Under `PolicyKind.NoAuth` the method is open (matches the embedded / single-user trust model).

## Suspending + resuming on a signal (Phase 2a)

Some workflows need to wait for an external event — a webhook confirming a payment, a human approval, an AI assistant's next turn. WAIT suspends execution and hands you back a `SuspendedHandle` you can persist:

```typescript
const paymentHandler = subgraph("checkout")
  .action("charge")
  .read({ label: "cart", by: "id", value: "$input.cart_id" })
  .wait({
    signal: "external:payment_confirmed",
    signal_shape: "{ amount: Int, currency: Text }",
  })
  .write({ label: "order", properties: { status: "paid" } })
  .respond({ body: "$result" });

await engine.registerSubgraph(paymentHandler);

const result = await engine.callWithSuspension("checkout", "charge", {
  cart_id: "c-42",
});
if (result.kind === "suspended") {
  // Persist the handle; the worker that receives the webhook later
  // resumes the handler with the matching payload.
  const bytes = result.handle;
  await fs.writeFile(".benten/suspended/checkout-c-42.cbor", bytes);
}

// Later — in a different process, after restart.
const bytes = await fs.readFile(".benten/suspended/checkout-c-42.cbor");
const outcome = await engine.resumeFromBytes(bytes, {
  amount: 19900,
  currency: "USD",
});
```

Tampered bytes, the wrong principal, or a grant revoked between suspend and resume all surface as typed errors before the write runs (`E_EXEC_STATE_TAMPERED`, `E_RESUME_ACTOR_MISMATCH`, `E_RESUME_SUBGRAPH_DRIFT`, `E_CAP_REVOKED_MID_EVAL`). The timed form `wait({ duration: "5m" })` is also supported; its deadline fires `E_WAIT_TIMEOUT` if no resume arrives in time.

## Viewing your operation subgraph

Benten's handlers are subgraphs you can inspect:

```typescript
console.log(handler.toMermaid());
// Prints a Mermaid diagram you can paste into any Markdown viewer.
```

```typescript
const trace = await engine.trace(handler.id, "post:create", {
  title: "Test",
  body: "Trace me",
});
console.log(trace.steps);
// Array of { nodeCid, primitive, durationUs, inputs?, outputs? } — one
// entry per OperationNode executed. `engine.trace` does NOT persist the
// outcome or fire a ChangeEvent; it's safe to run repeatedly.
```

## Next steps

- [`DSL-SPECIFICATION.md`](DSL-SPECIFICATION.md) -- the full TypeScript API
- [`validation/paper-prototype-handlers.md`](validation/paper-prototype-handlers.md) -- 5 realistic handler examples
- [`GLOSSARY.md`](GLOSSARY.md) -- Benten-specific terms
- [`VISION.md`](VISION.md) -- what Benten is and isn't

## What works / what doesn't (honest status as of 2026-04-17)

Phase 1 delivered:

- [x] `npx create-benten-app` scaffolder -- produces a project that runs `npm install && npm test && npm run build && npm run dev` green on a clean machine (`tools/create-benten-app/test/scaffolder.test.ts`).
- [x] `@benten/engine` npm package -- the TypeScript DSL wrapper over `@benten/engine-native`. Exposes `Engine`, `crud()`, `PolicyKind`, the 12 primitive helpers, and the typed error catalog.
- [x] `crud('post')` zero-config path -- one-liner that produces a five-action handler (`create`, `get`, `list`, `update`, `delete`). Optional `capability` / `hlc` / `label` overrides.
- [x] `handler.toMermaid()` visualization -- renders the registered subgraph as a Mermaid flowchart, sourced authoritatively from the Rust engine.
- [x] `engine.trace()` debugging -- per-step records carrying `nodeCid`, `primitive`, `durationUs`, plus `inputs` / `outputs` when the native tracer surfaces them.
- [x] Capability enforcement via `Engine.openWithPolicy(path, PolicyKind.GrantBacked)` + `engine.grantCapability({ actor, scope })` + `engine.revokeCapability(grantCid, actor)` -- the Phase-1 revocation-aware grant policy.

Deferred:

- [ ] Dev server with hot reload -- Phase 2 (needs registered-subgraph reload semantics).
- [ ] Version chains (`anchor.createVersion(...)`) as a first-class DSL shape -- Phase 2.
- [ ] P2P sync across Atriums -- Phase 3 (`benten-sync` crate, iroh transport).
- [ ] UCAN capabilities -- Phase 3 (`PolicyKind.Ucan` opens but its check path surfaces `E_CAP_NOT_IMPLEMENTED` today).
- [ ] WASM `SANDBOX` primitive -- Phase 2 (the primitive is type-defined; executor throws `E_PRIMITIVE_NOT_IMPLEMENTED`).

If you hit anything in the "delivered" list that doesn't behave as documented, please file an issue against `benten-engine` -- the scaffolder's smoke gate is the stability contract.
