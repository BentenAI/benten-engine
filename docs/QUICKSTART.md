# Quickstart

10 minutes from zero to a working Benten handler.

---

## 1. Install

```sh
npx create-benten-app my-app
cd my-app
npm install
npm test
```

The scaffolder drops a minimal TypeScript project:

- `@benten/engine` — the napi-rs-wrapped engine
- A handler file with a `crud('post')` one-liner
- A smoke test exercising create / get / list / update / delete

## 2. Your first handler

The zero-config path:

```typescript
import { crud } from "@benten/engine";

export const postHandlers = crud("post");
```

`crud("post")` exposes `create`, `get`, `list`, `update`, and `delete` actions with sensible defaults: properties inferred from input, no authentication required, local storage.

## 3. Use it

```typescript
import { Engine } from "@benten/engine";
import { postHandlers } from "./handlers.js";

const engine = await Engine.open(".benten/my-app.redb");
const handler = await engine.registerSubgraph(postHandlers);
// `handler.id` is "crud:post" — a stable content-addressed id.

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

## Adding capabilities

When you need authentication, open the engine with the grant-backed policy and stamp a capability on the handler:

```typescript
import { Engine, PolicyKind, crud } from "@benten/engine";

const engine = await Engine.openWithPolicy(
  ".benten/my-app.redb",
  PolicyKind.GrantBacked,
);
const handler = await engine.registerSubgraph(
  crud("post", { capability: "store:post:*" }),
);

// Grant the wildcard capability — permits create/update/delete under the
// `post` label because `store:post:*` attenuates to `store:post:write`.
await engine.grantCapability({ actor: "alice", scope: "store:post:*" });

// `callAs` accepts either a real CID or a friendly principal string.
await engine.callAs(handler.id, "post:create", { title: "x" }, "alice");
```

The default `PolicyKind.NoAuth` permits everything (the embedded / single-user model). Swap in `PolicyKind.GrantBacked` for the revocation-aware Phase-1 policy. UCAN lands in Phase 3.

## Diagnosing denied reads

Under a grant-backed policy, a denied read returns `null` — byte-identical with a genuine miss. That's deliberate: an unauthorized caller cannot distinguish existence from permission by probing CIDs.

This symmetric-None surface now covers more than just `Engine::get_node`: Phase 2a G4-A threaded Option C into the evaluator dispatch itself, so a READ primitive inside a user subgraph observes the same collapse (denied → `null`, backend miss → `null`) through `PrimitiveHost::check_read_capability`. Handlers running through `engine.call(...)` honour the same honest-no boundary end-to-end — there is no evaluator-side backdoor around the public-API contract.

If you're the operator and need to tell "denied" apart from "not found" (debugging a missing grant, for example), grant yourself the `store:debug:read` capability and call `engine.diagnoseRead`:

```typescript
await engine.grantCapability({ actor: "alice", scope: "store:debug:read" });

const info = await engine.diagnoseRead(cid);
if (info.notFound) {
  console.log("never written (or deleted)");
} else if (info.deniedByPolicy) {
  console.log(`exists, missing grant for ${info.deniedByPolicy}`);
} else {
  console.log("exists and is readable");
}
```

Without `store:debug:read`, `diagnoseRead` throws `E_CAP_DENIED` — ordinary callers still cannot distinguish the two cases. Under `PolicyKind.NoAuth` the method is open.

## Suspending and resuming (Phase 2a)

Some workflows wait for an external event — a webhook confirming payment, a human approval, an AI assistant's next turn. WAIT suspends execution and hands back a `SuspendedHandle` you persist:

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
  const bytes = result.handle;
  await fs.writeFile(".benten/suspended/checkout-c-42.cbor", bytes);
}

// Later, in a different process, after restart:
const bytes = await fs.readFile(".benten/suspended/checkout-c-42.cbor");
const outcome = await engine.resumeFromBytes(bytes, {
  amount: 19900,
  currency: "USD",
});
```

Tampered bytes, the wrong principal, or a grant revoked between suspend and resume all surface as typed errors before any write runs (`E_EXEC_STATE_TAMPERED`, `E_RESUME_ACTOR_MISMATCH`, `E_RESUME_SUBGRAPH_DRIFT`, `E_CAP_REVOKED_MID_EVAL`). The timed form `wait({ duration: "5m" })` fires `E_WAIT_TIMEOUT` if no resume arrives in time.

## Inspecting handlers

Handlers are data. You can visualize them:

```typescript
console.log(handler.toMermaid());
// Mermaid flowchart you can paste into any Markdown viewer.
```

And trace a call:

```typescript
const trace = await engine.trace(handler.id, "post:create", {
  title: "Test",
  body: "Trace me",
});
console.log(trace.steps);
// Array of { nodeCid, primitive, durationUs, inputs?, outputs? } — one entry
// per OperationNode executed. `engine.trace` does not persist the outcome or
// fire a ChangeEvent; it's safe to run repeatedly.
```

## What works today

Phase 1 shipped and Phase 2a is in flight. Live:

- `crud('post')` zero-config path
- All eight Phase-1 primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT)
- WAIT with signal + duration variants and the full 4-step resume protocol (Phase 2a, just landed)
- Capability enforcement via `PolicyKind.GrantBacked` + `grantCapability` + `revokeCapability`
- `handler.toMermaid()` visualization
- `engine.trace()` step-by-step evaluation records
- `engine.diagnoseRead()` operator introspection

Not yet live:

- SANDBOX, STREAM, SUBSCRIBE as user-visible primitives (Phase 2b)
- P2P sync and UCAN capabilities (Phase 3)
- Dev server with hot reload (Phase 2a close-out)

If something in the "live" list doesn't behave as documented, file an issue.

---

Next:

- [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md) — plain-English tour of Benten
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — depth on crates, invariants, and storage
- [`GLOSSARY.md`](GLOSSARY.md) — terms that mean something specific here
- [`ERROR-CATALOG.md`](ERROR-CATALOG.md) — every error code and its context
