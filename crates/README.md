# crates/

Rust workspace crates for the Benten Engine.

Phase 1 creates **six crates** (revised 2026-04-14 after critic review — added `benten-ivm` and `benten-caps` to keep the engine thin):

- **`benten-core/`** — Node, Edge, Value types; content hashing (BLAKE3 + DAG-CBOR + CIDv1); version chain primitives (opt-in convention, not mandatory)
- **`benten-graph/`** — Storage via `KVBackend` trait (redb implementation); indexes (hash + B-tree); MVCC via redb transactions; change notification stream (for SUBSCRIBE to consume)
- **`benten-ivm/`** — Incremental View Maintenance; subscribes to graph change stream via SUBSCRIBE primitive; per-view strategy (Algorithm B default, A/C optional). The evaluator is ignorant of IVM.
- **`benten-caps/`** — Capability types + pre-write hook trait + `NoAuthBackend` default. UCAN becomes one backend in `benten-id` (Phase 3); engine ships with `NoAuthBackend` so embedded/local-only users pay zero cost.
- **`benten-eval/`** — 12 operation primitives; iterative evaluator; structural validation (14 invariants); transaction primitive; wasmtime SANDBOX host
- **`benten-engine/`** — Orchestrator: composes the crates above into the public API; wires capability backend, storage backend, IVM subscriber

Additional crates added in Phase 3:
- `benten-sync/` — CRDT merge (per-property LWW + HLC, Loro for rich types), Merkle Search Tree diff, sync protocol over iroh
- `benten-id/` — Ed25519, UCAN as capability backend, DID/VC support

Optional future crate (if demand emerges):
- `benten-query/` — Cypher parser beyond operation subgraphs

These directories **do not yet exist** — they will be created during the Phase 1 spike (see `CLAUDE.md` Step 4) so the crate structure reflects what actually compiles against the validated dependencies. Creating empty skeletons ahead of the spike would lock in structure before we know what the stack tolerates.

## The Thinness Test

A developer should be able to use `benten-core` + `benten-graph` + `benten-engine` with `NoAuthBackend`, versioning disabled, and no IVM subscribers — and get a pure content-addressed graph database with no Benten-specific conventions. If that configuration requires anything from `benten-eval`, `benten-ivm`, or `benten-caps`, the engine is too thick.

See [`../docs/ENGINE-SPEC.md`](../docs/ENGINE-SPEC.md) Section 11 for the full specification, and [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) for how the crates compose.
