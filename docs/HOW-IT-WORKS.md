# How It Works

A plain-English tour of Benten — what it is, what problem it solves, how the pieces fit together, and where the project is going.

---

## The idea

Software is usually built as three separate things: a database, an application, and a set of queries that go between them. Each has its own language, its own representation, its own runtime. When you want to change something, you're changing across three layers.

Benten collapses that. The database and the application are the same graph. A handler — the thing you'd usually write as application code — is a subgraph of operation Nodes that the engine reads and walks. When a request comes in, the engine reads the subgraph, executes the operations, reads and writes data, and returns a result. The handler, the data it operated on, the audit trail of what happened, and any capabilities that gate the access all live in the same content-addressed graph.

Because everything is content-addressed, two machines that have the same graph state agree bit-for-bit on what everything is. Sync is content exchange, not schema reconciliation. Because handlers are data, an AI agent can compose or fork them the same way it would edit any other Node.

## Concepts, in the order they're useful

**Node.** The basic unit. A Node has a label (`post`, `user`, `capability-grant`), properties (key-value pairs), and a CID — a content identifier derived from its bytes. Two Nodes with identical content have the same CID.

**Edge.** A typed directional link between two Nodes. Labels like `NEXT`, `ON_ERROR`, `GRANTED_TO`, `CURRENT` live on edges and carry the semantics of control flow, error routing, authorization, and version pointers.

**Primitive.** One of the 12 operations the engine knows how to execute:

```
READ      WRITE     TRANSFORM    BRANCH     ITERATE    WAIT
CALL      RESPOND   EMIT         SANDBOX    SUBSCRIBE  STREAM
```

Each corresponds to an Operation Node with its own property shape. `READ` reads a Node. `WRITE` writes one. `TRANSFORM` runs a pure expression. `BRANCH` routes on a condition. `CALL` invokes another subgraph. `RESPOND` is a terminal that produces the handler's final output. The 12 primitives are the complete vocabulary; anything an application wants to do is composed from these.

Eight primitives have live executors today (READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT). WAIT lands in Phase 2a; SANDBOX, STREAM, and SUBSCRIBE in Phase 2b.

**Subgraph.** A DAG of Operation Nodes wired with control-flow Edges. The engine walks this DAG to execute a handler. Because it's a DAG with bounded fan-out, execution is always bounded; the engine is not Turing complete by construction.

**Handler.** A subgraph registered with the engine under a name. `crud('post')` produces a handler with five actions — create, get, list, update, delete — each action its own walk through the subgraph.

**View.** A materialized query result kept current by subscribing to graph changes. A list-by-label read hits a view; it's O(1) because the view is already computed. Views are regular Nodes that an internal subscriber advances on every ChangeEvent.

**Capability.** A grant — "this actor may write Nodes matching this pattern" — stored as a Node with a `GRANTED_TO` edge. Writes hit a pre-write policy hook; the default policy (`NoAuth`) allows everything; a `GrantBacked` policy checks the grant graph. UCAN lands in Phase 3 as another policy backend.

**Anchor + Version + CURRENT.** When you need history (undo, audit, time-travel), you opt into the version-chain pattern: an Anchor Node with stable identity points at the latest Version Node via a CURRENT edge. Version Nodes are immutable once written; updating is "write a new Version, advance CURRENT atomically." Ephemeral data doesn't pay the versioning cost.

## What happens when you call a handler

A concrete walk-through. You've registered `crud('post')`. Now:

```typescript
await engine.call(handler.id, 'post:create', { title: 'Hello', body: 'Works.' });
```

Here's what the engine does:

1. **Locate the subgraph.** The handler's CID points at a registered subgraph. The engine reads it — reading a subgraph is reading a Node.
2. **Find the action.** `post:create` is an entry point on the subgraph. The evaluator starts at the Operation Node bound to that entry.
3. **Walk.** The evaluator steps through Operation Nodes one at a time. For `create`, the flow is TRANSFORM (build the Node from your input) → WRITE (persist it) → RESPOND (return the CID).
4. **At each WRITE, check the capability hook.** The pre-write hook consults the policy. `NoAuth` approves; `GrantBacked` walks the grant graph. Denial terminates the walk with `E_CAP_DENIED`.
5. **Commit the transaction.** WRITEs are wrapped in a storage transaction (ACID via redb). On commit, content is hashed into CIDs, the audit log is advanced, and ChangeEvents fire.
6. **Update views.** The IVM subscriber sees the ChangeEvents and updates any views whose subscription patterns match. `post:list` will see the new Node on the next read.
7. **Return.** RESPOND's value is returned to the caller.

This is the same code path for the native Rust API, the napi-rs TypeScript wrapper, and eventually a WASM-hosted variant. The engine is agnostic to how the call arrived.

## Why content addressing is load-bearing

Two Benten machines with the same set of Nodes agree bit-for-bit on what everything is. A Node's CID is a hash of its canonical DAG-CBOR bytes; byte-identical content yields byte-identical CIDs. This has consequences:

- **Sync is content exchange.** When Phase 3 ships, two peers synchronizing don't reconcile schema-level changes; they exchange content-addressed Nodes and the receiver cryptographically verifies what arrived.
- **Handlers are referenceable by hash.** A subgraph registered under `crud:post` has a CID. Forking it points at a different CID. Sending a handler across the network lets the receiver verify it matches what you claimed before running it.
- **Audit trails are free.** Every write produces a content-hashed change record; the history is the graph.
- **Dedup is automatic.** Two writers producing identical content produce a single Node.

Content addressing also shapes security. The default read posture returns `null` for both "not found" and "access denied" — a byte-identical response, so an unauthorized caller cannot distinguish existence from permission by probing CIDs. Explicit diagnostic methods gated behind a debug capability exist for operators who need to distinguish the two.

## What the engine is not

- **Not Turing complete.** Every handler terminates. The escape hatch for genuine compute — arbitrary TypeScript, ML inference, an image resize — is SANDBOX (Phase 2b, WASM, fuel-metered).
- **Not a relational database.** You can model relational shapes in a graph, but the engine isn't optimized for row-oriented scans across large tables. Read patterns that scale are the ones that hit IVM views.
- **Not an app framework.** It's the engine underneath one. You write handlers in TypeScript; the DSL produces registered subgraphs. A CMS, a chat service, a personal assistant runs on top.
- **Not finished.** Phase 1 shipped; Phase 2a and 2b extend the primitive set; Phase 3 adds P2P sync. The current state is usable for local single-process work and for evaluating whether the model fits a problem you have.

## The path from here

**Phase 1 (done)** gave the engine a working 8-primitive evaluator, content-addressed storage with MVCC, hand-written IVM views, a pluggable capability policy, TypeScript bindings, a scaffolder, and debug tooling. The canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` is stable across Linux / macOS / Windows.

**Phase 2a (in flight)** finishes the evaluator. Adds WAIT (suspend/resume with DAG-CBOR persisted state). Completes the 14 structural invariants: system-zone enforcement at runtime, multiplicative iteration budgets across CALL/ITERATE nesting, immutability rejection for non-dedup writes, causal attribution threaded through every trace step. Hardens capability TOCTOU with wall-clock revocation checks on a dual monotonic + HLC clock source. Ships the 4-step resume protocol that guards against tampered state, principal mismatch, stale subgraph references, and mid-eval grant revocation.

**Phase 2b** adds the WASM SANDBOX (wasmtime-backed, fuel-metered, capability-manifested host functions), the STREAM and SUBSCRIBE primitives (user-visible back-pressured output and reactive change notifications), and Algorithm B generalized for user-registered views with explicit strategy selection.

**Phase 3** adds P2P sync (iroh transport, CRDT-merged content, Merkle Search Tree diff, UCAN capability chains, DID-based identity, HLC timestamps). After Phase 3, two Benten instances hold the same graph and exchange Nodes cryptographically — the first configuration where "Benten communities" become real.

**Phases 4–8** are applications composed from the engine: a CMS migration that exercises the engine under realistic load, platform features (schema-driven rendering, self-composing admin, declarative plugin manifests), a Personal AI Assistant MVP (MCP, PARA knowledge organization, on-demand tool composition), community spaces, and a USD-pegged currency for the network's economic layer.

Each phase layers on the previous without requiring changes below. The engine's abstractions were designed so that Phase 3 sync doesn't need Phase 1 changes; Phase 6 AI composition doesn't need Phase 2 changes.

## Where to go from here

- **Try it.** See [`QUICKSTART.md`](QUICKSTART.md) for the 10-minute path. `npx create-benten-app my-app` gives you a scaffolded project with a `crud('post')` handler.
- **Understand the architecture.** [`ARCHITECTURE.md`](ARCHITECTURE.md) walks the seven crates, the invariant set, the storage layer, and the evaluator's request flow.
- **Read the error catalog.** [`ERROR-CATALOG.md`](ERROR-CATALOG.md) is the stable contract: every error the engine surfaces, by discriminant, with context.
- **Look at the glossary.** [`GLOSSARY.md`](GLOSSARY.md) names the concepts above and a few more.
- **Read the source.** The engine is deliberately small. `crates/benten-engine/src/lib.rs` is the integration surface; `crates/benten-eval/src/evaluator.rs` is the walk loop; `crates/benten-graph/src/redb_backend.rs` is the storage. If the docs are confusing, the code is the ground truth.
