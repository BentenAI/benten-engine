# Benten Architecture

**Created:** 2026-04-14
**Last Updated:** 2026-04-14 (6-crate Phase 1 plan after critic review; added `benten-ivm` and `benten-caps` as separate crates to keep engine thin; version-chain convention stayed in `benten-core`; moved exploratory layers to `future/`)
**Status:** Committed architecture for Phase 1-4. Speculative layers documented in [`future/`](future/).
**Audience:** Developers, architects, and critics who need to understand how the Phase 1-4 scope composes.

---

## The Committed Architecture (Phase 1-4)

Benten's committed architecture has three layers. Two more layers are proposed but not committed — they live in [`future/`](future/).

```
  +------------------------------------------------------+
  |  Layer 3: APPLICATIONS (Phase 4)                     |
  |  Thrum CMS migration; reference implementation       |
  +------------------------------------------------------+
  |  Layer 2: SYNC & IDENTITY (Phase 3)                  |
  |  iroh P2P, CRDT merge, Merkle Search Tree diff,      |
  |  UCAN capabilities, DID/VC, Ed25519, HLC             |
  +------------------------------------------------------+
  |  Layer 1: GRAPH ENGINE (Phase 1-2)                   |
  |  Nodes, Edges, 12 primitives, version chains,        |
  |  capability hooks, IVM, storage, WASM target         |
  +------------------------------------------------------+
```

Additional exploratory layers (Runtime, Economy, Governance, Marketplace) are proposals, not committed scope. They are documented in [`future/`](future/) and inform Phase 1 abstraction boundaries (so we don't foreclose them) but do not appear here as active engineering.

---

## Layer 1: Graph Engine (Phase 1-2 scope)

The foundation. A Rust-native graph execution engine. **Seven crates** after critic review and the Phase-1 Compromise #3 closure (up from the four in the original draft — added `benten-ivm` and `benten-caps` to keep the engine thin, then extracted `benten-errors` to close the error-catalog-dependency concern):

```
  crates/
    benten-errors/     # Stable ErrorCode catalog discriminants; zero Benten-crate deps
    benten-core/       # Node, Edge, Value, content hashing (BLAKE3 + DAG-CBOR + CIDv1)
    benten-graph/      # Storage (KVBackend trait + redb impl), indexes, MVCC
    benten-ivm/        # Incremental View Maintenance, subscribes to graph changes
    benten-caps/       # Capability types + pre-write hook + NoAuthBackend default
    benten-eval/       # 12 primitives, evaluator, structural validation
    benten-engine/     # Orchestrator: public API, ties crates together

  bindings/
    napi/              # Node.js bindings via napi-rs v3 (also compiles to WASM)
```

### Why seven crates, not four (nor six)

Critic reviews (architecture-purity, engine-philosophy) identified three concerns with the original four-crate plan that together added two crates (`benten-ivm` and `benten-caps`). A fourth extraction (`benten-errors`) landed during Phase 1 to close named compromise #3:

1. **IVM was load-bearing for capability resolution, governance resolution, and event dispatch** — all inside one unvalidated algorithm. Extraction to `benten-ivm` lets the engine evaluator stay ignorant of IVM; IVM subscribes to graph change notifications via the SUBSCRIBE primitive.

2. **Capability enforcement was inside `benten-eval`.** That couples the engine to UCAN forever. Extraction to `benten-caps` with a pre-write hook trait means the engine ships with a `NoAuthBackend` default, and UCAN (or VC, or TEE attestation, or any future policy) is a pluggable backend.

3. **Version chains are a pattern, not a primitive.** They're Nodes + Edges + a CURRENT pointer. Consumers who don't want versioning (ephemeral data, presence channels) shouldn't pay for it.

4. **The `ErrorCode` catalog enum forced a `benten-core` dependency on every crate that only wanted stable string identifiers.** Extraction to `benten-errors` (closes named compromise #3) puts the catalog at the root of the workspace dependency graph — every other crate imports from it with zero Benten-crate coupling, and the TS codegen + drift-detector target a single canonical source.

Version chains land in `benten-core` as an opt-in convention rather than a separate crate — they're thin enough that a trait is overkill, but they're not mandatory.

### Key responsibilities per crate

- **`benten-errors`** — Stable `ErrorCode` enum discriminants mirroring `docs/ERROR-CATALOG.md`. Zero dependencies on other Benten crates; sits at the root of the workspace dependency graph.
- **`benten-core`** — Node, Edge, Value, content-addressed hashing (BLAKE3 + DAG-CBOR via `serde_ipld_dagcbor`, CIDv1 format), version chain primitives (Anchor + Version Node + CURRENT Edge pattern)
- **`benten-graph`** — Storage via a `KVBackend` trait (implemented by `benten-storage-redb` for native; future WASM/network-fetch backends plug in), indexes (hash + B-tree), MVCC via redb transactions
- **`benten-ivm`** — Materialized views maintained via change subscriptions. Per-view strategy selection (Algorithm B default, A for rarely-read, C for complex joins). NOT known to the evaluator.
- **`benten-caps`** — UCAN-compatible capability grants as Nodes. Pre-write hook trait for policy enforcement. `NoAuthBackend` for embedded/local-only use.
- **`benten-eval`** — The 12 operation primitives, iterative evaluator (explicit stack, not recursive), structural validation (14 invariants), transaction primitive (begin/commit/rollback)
- **`benten-engine`** — Composes the above into a public API. Wires the capability hook, storage backend, and IVM subscriber.

### The 12 Operation Primitives (revised 2026-04-14)

| # | Primitive | Purpose |
|---|-----------|---------|
| 1 | READ | Retrieve from graph |
| 2 | WRITE | Mutate graph (auto version-stamp if versioning enabled) |
| 3 | TRANSFORM | Pure data reshaping (sandboxed expression) |
| 4 | BRANCH | Conditional routing (forward-only) |
| 5 | ITERATE | Bounded collection processing |
| 6 | WAIT | Suspend until signal or timeout |
| 7 | CALL | Execute another subgraph |
| 8 | RESPOND | Terminal: single output |
| 9 | EMIT | Fire-and-forget notification |
| 10 | SANDBOX | WASM computation (fuel-metered, no re-entrancy) |
| 11 | SUBSCRIBE | Reactive change notification |
| 12 | STREAM | Partial/ongoing output with back-pressure |

Dropped from the original 12: **VALIDATE** (BRANCH + TRANSFORM + RESPOND), **GATE** (capability checking uses the `requires` property on any Node, not a separate step). Added: **SUBSCRIBE** (the primitive IVM, sync, and EMIT delivery all compose on top of), **STREAM** (WinterTC SSE, WebSocket messages, LLM token streams).

### Design Principles

- **Thin engine.** Every feature that can be a module is a module.
- **Stateless evaluator.** No implicit dependency on local storage during subgraph walks.
- **`KVBackend` trait at `benten-graph`.** redb is one implementation; WASM/network-fetch is a future implementation.
- **Capability hook.** Engine provides the hook; UCAN/NoAuth/custom policies are backends.
- **Not Turing complete.** DAGs only. Bounded iteration. Guaranteed termination.

### Output of Phase 1-2

A working engine. TypeScript developers can create/read/update/delete Nodes, define operation subgraphs, maintain materialized views, enforce capabilities, compile the same engine to WASM for browsers/edge, and benchmark against PostgreSQL + AGE on hot-path queries.

---

## Layer 2: Sync & Identity (Phase 3 scope)

Distributes the engine across instances and users.

```
  crates/
    benten-sync/    # CRDT merge, sync protocol, Merkle Search Tree diff
    benten-id/      # Ed25519, UCAN chains (as `benten-caps` backend), DID/VC
```

### Key responsibilities

- P2P transport via iroh v0.97+ (QUIC-based, dial-by-public-key, NAT traversal with relay fallback)
- CRDT merge using per-property LWW + HLC; Loro for rich CRDT types where needed
- Merkle Search Tree diff for efficient subgraph sync (AT Protocol-validated pattern)
- Light-client verification: received data verified against sender's content-addressed root (IBC-inspired pattern, cheap because we already use content addressing)
- UCAN capability grants with delegation chain verification (as a `benten-caps` policy backend)
- DID-based identity (did:key baseline; did:web, did:plc, did:peer as options)
- HLC timestamps with drift tolerance (uhlc crate)
- Device-to-device key transfer (QR + ephemeral DH)
- Optional Shamir threshold key recovery (Web3Auth tKey or Dark Crystal pattern)

### Output of Phase 3

Two Benten instances sync subgraphs bidirectionally. A user on Device A can work with their graph offline; changes propagate to Device B when peer connectivity exists. Capability grants verified on receipt. Version chain conflicts resolve into commit DAGs.

---

## Layer 3: Applications (Phases 4-8 scope)

Layer 3 is where committed applications run on the engine. The committed scope expands beyond Thrum to include the platform features that make self-composition real, the AI assistant that drives adoption, Gardens MVP for community spaces, and Credits MVP for the economic engine. All compose on the Phase 1 engine without requiring engine changes.

### Phase 4: Thrum CMS migration

- CMS domain (content types, blocks, compositions, field types) expressed as operation subgraphs
- Existing Thrum modules + admin migrate
- 3,200+ behavioral tests pass against the Benten engine
- Performance competitive with or better than PostgreSQL + AGE baseline

### Phase 5: Platform features

- Schema-driven rendering (materializer pipeline as operation subgraphs)
- Self-composing admin (admin UI configurable by editing compositions in the graph)
- Declarative plugin manifest format (requires-caps, provides-subgraphs, migrations)

### Phase 6: Personal AI Assistant MVP

- MCP integration for LLM providers (local + cloud)
- PARA knowledge organization (Projects / Areas / Resources / Archives as graph Nodes)
- On-demand tool generation (assistant composes operation subgraphs from primitives)
- UCAN capability grants for assistant's authority (spending caps, rate limits, intent declarations)
- Causal attribution of every agent action to a signed user intent

See [`research/explore-personal-ai-assistant.md`](research/explore-personal-ai-assistant.md) for scoping.

### Phase 7: Digital Gardens MVP

- Garden creation flow (promote Atrium to Garden, or create new)
- Admin-configured governance (invitations, roles, basic moderation)
- Member-mesh replication with bootstrap from any online member
- Full fractal Groves remain exploratory

See [`research/explore-gardens-mvp.md`](research/explore-gardens-mvp.md) for scoping.

### Phase 8: Benten Credits MVP

- USD-pegged stable currency with treasury-backed reserves
- FedNow on/off ramp for mint/burn
- Multi-sig mint/burn with HSM
- Tab-based periodic net settlement between peers
- Real-time reserve monitoring

All five phases (4-8) are applications composed from the engine's primitives. None require engine core modifications.

### Not yet committed (Phase 9+ exploratory)

- Full Groves with fractal/polycentric governance
- Garden/Grove federation (polycentric, cross-community)
- Knowledge attestation marketplace
- Benten Runtime (WinterTC edge host)
- `bentend` peer daemon (general-purpose compute)
- Broader compute marketplace (arbitrary workloads)
- DAO transition / Governance Grove

See [`FULL-ROADMAP.md`](FULL-ROADMAP.md) for the full partition and [`future/`](future/) for exploratory proposals.

---

## How Requests Flow (Phase 1-4)

### Read request

1. Client calls engine API (via napi-rs bindings or direct Rust)
2. `benten-engine` dispatches to the operation subgraph registered for this request shape
3. `benten-eval` walks the subgraph:
   - READ operations hit IVM materialized views (O(1)) or fall through to graph storage
   - TRANSFORM runs pure expressions
   - CALL invokes sub-subgraphs
   - BRANCH routes based on conditions
4. Each WRITE would invoke the capability hook (`benten-caps`); reads go through without hook by default
5. Response returns via RESPOND (single) or STREAM (chunked)
6. `benten-ivm` gets notified of any changes for view maintenance

### Write (transactional subgraph)

1. Same entry flow
2. Transaction primitive wraps all WRITEs in the subgraph (begin → operations → commit)
3. Each WRITE hits the capability hook; rejection aborts the transaction
4. On commit: content hashes computed, version chain advanced (if versioning enabled on the Node), IVM views updated via change notifications, sync protocol propagates deltas to peers holding this subgraph (Phase 3+)
5. If any WRITE fails, all roll back atomically

### Cross-instance sync (Phase 3+)

1. Instance A writes a Node. HLC timestamp recorded.
2. `benten-sync` packages the version Node and pushes to peer agreements
3. Instance B receives; verifies content hash and capability chain
4. If HLC is within drift tolerance, merge per-property LWW
5. If conflict, branch the version chain into a commit DAG (both branches valid, resolve on read)

---

## The Graph Is the Control Plane

Every layer is configured through the graph:

- **Capability grants** → CapabilityGrant Nodes with GRANTED_TO Edges
- **Sync agreements** → SyncAgreement Nodes with subgraph-scope properties
- **Operation handlers** → registered subgraphs (content-addressed, immutable once registered)
- **IVM view definitions** → ViewDefinition Nodes with strategy property
- **Reputation scores** (when economic layer exists) → materialized IVM views
- **User preferences** (when AI agents exist) → Preference Nodes the agent consults

Configuration is inspectable, forkable, versionable, and auditable by design. No YAML scattered across machines, no opaque admin UIs. The state of any layer is queryable like any other data.

---

## Why This Works

1. **Six crates, one coherent engine.** Each crate has one responsibility. The thinness test (can someone use `benten-engine` for a problem unrelated to communities, sync, or Thrum?) is achievable with `NoAuthBackend` + versioning-disabled + no IVM subscribers.

2. **Phase 1 produces something useful standalone.** The engine with napi-rs bindings is a new kind of embedded graph database with a code-as-graph paradigm. Developers can use it without Phase 2+ layers.

3. **Phase 2+ layers are reachable from Phase 1.** Storage trait, capability hook, content addressing, operation subgraphs, SUBSCRIBE primitive — all designed to support Phase 2-4 without requiring changes to earlier crates.

4. **Commodity where possible, custom where necessary.** iroh, redb, wasmtime, `serde_ipld_dagcbor`, papaya, Ed25519, uhlc, Loro — we compose existing production-quality crates. Our original work is the graph engine itself and eventual economic/governance primitives on top.

5. **Exploratory scope doesn't pressure the engine.** Runtime, Credits, marketplace, bentend — all in [`future/`](future/). The engine is built to not foreclose them, but they do not appear in committed architecture until they earn their place.
