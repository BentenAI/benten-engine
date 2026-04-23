# Glossary

Terms that have specific meaning in Benten. Alphabetical.

---

**ADDL** — Agent-Driven Development Lifecycle. The project's development methodology: plan → critic review → test planning → test writing → test review → implementation → mini-review → quality council. Referenced in commit messages.

**Algorithm B** — The IVM (Incremental View Maintenance) strategy Benten selects for most views: dependency-tracked incremental maintenance. Phase 1 ships five hand-written views; Phase 2b generalizes to a registration-time strategy-selection API.

**Anchor (Anchor Node)** — A Node with stable identity that never changes. External edges point to anchors, not to versions. The anchor has a `CURRENT` edge to its latest Version Node. See "Version chain."

**BLAKE3** — The cryptographic hash function used for CID derivation. Fast, tree-hash-friendly, multi-threaded.

**CID / CIDv1** — Content Identifier version 1. IPLD standard: version byte + multicodec + multihash. Benten uses CIDv1 with multicodec `0x71` (dag-cbor) and multihash `0x1e` (BLAKE3).

**Code-as-graph** — The paradigm where application logic is represented AS graph structure, not stored IN graph properties. A handler is a subgraph of Operation Nodes connected by control-flow Edges. The engine walks the subgraph to execute it.

**Content-addressed** — A storage model where an item's identity is derived from its bytes. Identical content has identical identity; different content has different identity. Enables cryptographic verification, dedup, and peer sync without schema reconciliation.

**CURRENT pointer** — An Edge from an Anchor Node to its latest Version Node. Atomic update moves the pointer within a storage transaction, giving versioned entities "single latest" semantics while preserving history.

**DAG-CBOR** — The IPLD subset of CBOR with canonical (map-keys-sorted, no indefinite-length) encoding. The on-the-wire format for content-addressed Nodes. Implemented via `serde_ipld_dagcbor`.

**Edge** — A typed directional link between two Nodes. Labels include `NEXT` (control flow), `ON_ERROR`, `ON_NOT_FOUND`, `GRANTED_TO`, `CURRENT`, etc.

**ExecutionStateEnvelope** — The DAG-CBOR-serialized shape a Phase-2a WAIT primitive produces when suspending. Carries the frame stack, pinned subgraph CIDs, resumption principal, and context bindings needed to resume atomically across process boundaries. Envelope CID is content-addressed for tamper detection.

**Handler** — A registered subgraph that acts as an entry point for external calls. `crud('post')` produces a handler with five actions.

**HLC** — Hybrid Logical Clock. Monotonic timestamps combining physical and logical clocks, used for causal ordering. Relevant in Phase 3 (P2P sync) and in Phase-2a capability wall-clock revocation paths. Crate: `uhlc`.

**Invariant** — A structural or runtime check the engine enforces. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full 14-invariant list and their phase landing.

**iroh** — The P2P networking library (QUIC, dial-by-public-key, NAT traversal with relay fallback) used in Phase 3.

**IVM** — Incremental View Maintenance. Benten keeps materialized views up to date via change subscriptions; common reads hit them in O(1).

**`KVBackend`** — The storage trait in `benten-graph` that abstracts over the key-value store. The Phase-1 implementation is redb; a future WASM implementation will fetch content-addressed Nodes from peer storage.

**napi-rs** — The Rust-to-Node.js binding framework. v3 compiles the same codebase to native and WASM targets and auto-generates TypeScript `.d.ts` files.

**`NoAuthBackend`** — The default `benten-caps` policy: allows all writes without capability checks. Ships as the engine's default so embedded / local-only users pay no capability-system overhead.

**Node** — The basic unit of Benten storage. A Node has a label, properties (key-value pairs), and a CID derived from its bytes.

**Operation Node** — A Node representing one of the 12 operation primitives. Operation subgraphs are DAGs of Operation Nodes connected by control-flow Edges.

**Operation subgraph** — A handler represented as a DAG of Operation Nodes. Bounded (max depth, max fan-out, max Nodes, iteration budget). Deterministically evaluable. Content-hashed. Immutable once registered.

**redb** — The Phase-1 embedded key-value store: pure Rust, ACID, MVCC (concurrent readers with single writer), crash-safe via copy-on-write B-trees.

**SANDBOX** — The WASM computation escape hatch (Phase 2b, wasmtime-backed, fuel-metered, no re-entrancy, max 1 MB output).

**`serde_ipld_dagcbor`** — The CBOR serialization crate Benten uses. Deterministic by default (sorts map keys); IPLD-native.

**STREAM** — A Phase-2b primitive producing partial/ongoing output with back-pressure. For Server-Sent Events, WebSocket messages, LLM token streams, progress updates.

**Subgraph** — See "Operation subgraph."

**SUBSCRIBE** — A Phase-2b primitive providing reactive change notification. The base primitive on which IVM views, sync delta propagation, and event-driven handlers all compose.

**TOCTOU** — Time-of-check-to-time-of-use. The security class where a permission check succeeds but the underlying permission changes before the protected action runs. Phase-2a hardens five TOCTOU points across capability enforcement (commit, CALL entry, ITERATE boundary, WAIT resume, wall-clock revocation ceiling).

**Transaction primitive** — An engine-provided begin/commit/rollback cycle wrapping all WRITEs in a subgraph evaluation. If any WRITE fails, all WRITEs in the transaction roll back atomically.

**UCAN** — User-Controlled Authorization Networks. Capability-based auth tokens. Phase 3 ships UCAN as a `benten-caps` policy backend alongside the default `NoAuthBackend` and the Phase-1 `GrantBackedPolicy`.

**Version chain** — Benten's opt-in history pattern: Anchor + Version Nodes + `NEXT_VERSION` edges + `CURRENT` pointer. History = traverse. Undo = move `CURRENT`. Sync (Phase 3) = exchange version Nodes. Ephemeral data does not pay versioning cost.

**WAIT** — A Phase-2a primitive that suspends execution until an external signal arrives or a duration elapses. The engine produces an `ExecutionStateEnvelope` at suspend time; resume runs a 4-step integrity + principal + pin + capability protocol before continuing.

**`wasmtime`** — The WASM runtime for Phase-2b SANDBOX. Rust-native, Bytecode Alliance, fuel-metered, Component Model support.
