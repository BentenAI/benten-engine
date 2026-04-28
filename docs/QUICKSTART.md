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
// `handler.id` is "post-handler" — derived as `${label}-handler` from the
// `crud("post")` label. The action is the second argument to `engine.call`.

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
import { promises as fs } from "node:fs";

const paymentHandler = subgraph("checkout")
  .action("charge")
  .read({ label: "cart", by: "id", value: "$input.cart_id" })
  .wait({
    signal: "external:payment_confirmed",
    signal_shape: "{ amount: Int, currency: Text }",
  })
  .write({ label: "order", properties: { status: "paid" } })
  .respond({ body: "$result" });

await engine.registerSubgraph(paymentHandler.build());

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

## Streaming results back to the client (Phase 2b)

Long-running queries (large list pulls, log tailers, aggregate
exports) shouldn't materialise the full result before responding. The
STREAM primitive yields chunks as they're produced; the JS-side
`engine.callStream` returns an `AsyncIterable<Chunk>` you can `for
await ... of`:

```typescript
import { Engine, subgraph } from "@benten/engine";

const exportHandler = subgraph("export-feed")
  .read({ label: "post", as: "rows" })
  .iterate({ over: "$result.rows", max: 100_000 })
  .stream({ source: "$loop.row", chunkSize: 64 })
  .respond({ body: "{ status: \"streamed\" }" })
  .build();

await engine.registerSubgraph(exportHandler);

for await (const chunk of engine.callStream("export-feed", "default", {})) {
  process.stdout.write(chunk);          // chunks arrive as `Buffer`
}
// `for await` calls `return()` on early break → underlying mpsc
// receiver closes promptly; no leaked tasks.
```

Back-pressure is handled Rust-side; the JS iterator is a thin shell.
If the consumer slows, the producer (the IVM-driven evaluator) blocks
on the mpsc bound and stops doing work. Closing the iterator tears
down the producer in O(1).

## Reacting to changes (SUBSCRIBE — Phase 2b)

SUBSCRIBE is the reactive primitive: a handler subscribes to a
label's ChangeEvent stream and runs each event through a downstream
DAG (commonly TRANSFORM + WRITE to project a derived view).

```typescript
const projectHandler = subgraph("post-summary-view")
  .subscribe({ event: "post:changed" })
  .transform({ expr: "{ id: $event.cid, title: $event.body.title }" })
  .write({ label: "post-summary" })
  .emit({ event: "post-summary:built" })
  .build();

await engine.registerSubgraph(projectHandler);

// Engine wires the SUBSCRIBE node into the change-event bus on
// register. Each post:changed event runs the downstream DAG once.
// SUBSCRIBE handlers do NOT need an explicit `engine.call` — the
// runtime drives them.
```

A SUBSCRIBE handler that errors fires an `E_SUBSCRIBE_PROJECTION` on
the engine error channel; the handler stays subscribed (an erroring
event doesn't poison the subscription) and the next event drives a
fresh frame. Cursor durability — the "where in the change-event
stream did this subscriber leave off" — is persisted by the
G12-E generalised SuspensionStore.

## Calling out to WASM (SANDBOX — Phase 2b)

When you need pure-CPU compute over arbitrary bytes (text
summarisation, image resampling, format conversion, hashing beyond
BLAKE3) the SANDBOX primitive runs a precompiled WASM module under
the wasmtime host with capability-derived host-fn manifest:

```typescript
import { Engine, subgraph } from "@benten/engine";

// Install a module manifest declaring one or more WASM modules.
// `moduleCid` is the CIDv1 of the compiled WASM bytes (produced by
// your build pipeline); `manifestCid` is the canonical-DAG-CBOR CID
// of the manifest itself (compute via `engine.computeManifestCid()`).
const installedCid = await engine.installModule({
  name: "example.summarizer",
  version: "0.1.0",
  modules: [{
    name: "summarize-v1",
    cid: moduleCid,
    requires: ["host:compute:log", "host:compute:time", "host:compute:kv:read"],
  }],
}, manifestCid);

const summariseHandler = subgraph("summarize")
  .read({ label: "doc", by: "id", value: "$input.doc_id" })
  .sandbox({
    module: "example.summarizer:summarize-v1",   // <manifestName>:<moduleName>
    fuel: 1_000_000,              // wasmtime fuel cap (per-call)
    wallclockMs: 30_000,          // hard wallclock kill (per-call)
    outputLimitBytes: 1_048_576,  // Inv-7 ceiling (per-call)
  })
  .write({ label: "summary" })
  .respond({ body: "$result" })
  .build();

await engine.registerSubgraph(summariseHandler);
const out = await engine.call("summarize", "default", { doc_id: "d-42" });
```

A SANDBOX call with a manifest the caller's policy doesn't grant
fires `E_CAP_DENIED` at SANDBOX entry; an in-module host-fn call to
a host-fn outside the manifest fires
`E_SANDBOX_HOST_FN_NOT_ON_MANIFEST`. Inv-4 (nest depth) and Inv-7
(cumulative output) both fire as `E_INV_SANDBOX_DEPTH` and
`E_INV_SANDBOX_OUTPUT` respectively. See [`HOST-FUNCTIONS.md`](HOST-FUNCTIONS.md)
for the full host-fn surface and [`SANDBOX-LIMITS.md`](SANDBOX-LIMITS.md)
for limit defaults.

SANDBOX is **composition-only** — there is no top-level
`engine.sandbox(...)` API. A SANDBOX node always lives inside a
handler (so capability resolution, Inv-4 nest-depth, and Inv-14
attribution chaining all flow through the evaluator).

Runnable example handlers ship at
[`packages/engine/examples/`](../packages/engine/examples/) covering
all three Phase-2b primitives.

## What works today

Phase 1 shipped, Phase 2a closed at tag `phase-2a-close`, and Phase
2b closes with this commit. Live:

- `crud('post')` zero-config path
- **All 12 primitives** (READ, WRITE, TRANSFORM, BRANCH, ITERATE,
  WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM) — Phase-2b
  added the last three.
- WAIT with signal + duration variants and the full 4-step resume protocol
- Capability enforcement via `PolicyKind.GrantBacked` + `grantCapability` + `revokeCapability`
- `handler.toMermaid()` visualization
- `engine.trace()` step-by-step evaluation records
- `engine.diagnoseRead()` operator introspection
- `engine.callStream()` AsyncIterable + `engine.openStream()`
- Reactive SUBSCRIBE handlers driven off the change-event bus
- SANDBOX primitive with wasmtime host + named-manifest registry

Not yet live:

- P2P sync and UCAN capabilities (Phase 3)
- Marketplace / dynamic manifest registration (`register_runtime` reserved with `E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED`; Phase 8)
- `random` host-fn (deferred to Phase 2c pending CSPRNG framework decision)

If something in the "live" list doesn't behave as documented, file an issue.

---

Next:

- [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md) — plain-English tour of Benten
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — depth on crates, invariants, and storage
- [`GLOSSARY.md`](GLOSSARY.md) — terms that mean something specific here
- [`ERROR-CATALOG.md`](ERROR-CATALOG.md) — every error code and its context
