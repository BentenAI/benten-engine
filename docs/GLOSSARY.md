# Glossary

Terms that have specific meaning in Benten and may differ from common usage. Alphabetical.

---

**Adoption Driver** — Pillar 2 of the three-pillar vision. The Personal AI Assistant (Phase 6 committed) that replaces users' paid software stack with one system they own. See [`VISION.md`](VISION.md).

**ADDL** — Agent-Driven Development Lifecycle. The project's development methodology. Pre-work → spec → test-plan → test-writing → test-review → implementation groups → mini-reviews → post-implementation review → quality council. See `DEVELOPMENT-METHODOLOGY.md`.

**Algorithm B** — The IVM (Incremental View Maintenance) strategy selected for Benten. Dependency-tracked incremental maintenance. Chosen after benchmarking A (eager invalidation), B (incremental), and C (DBSP Z-set) in `docs/research/ivm-benchmark/`. The engine uses per-view strategy selection: B for most views, A for rarely-read views, C deferred.

**Anchor** (Anchor Node) — A Node with stable identity that never changes. External Edges point to anchors. The anchor has a CURRENT edge to its latest version Node. See "Version chain."

**Atrium** — A Benten community tier: peer-to-peer direct connections between a small number of trusted individuals (family, close friends, partners, student-school). Private, selective, bidirectional sync of chosen subgraphs. No formal governance. The smallest community unit. **Phase 3 committed** — ships with the P2P sync layer (two or more peers syncing = an Atrium).

**Benten** — The engine and the broader platform. Japanese goddess of eloquence, music, and fortune.

**BentenAI** — The company building Benten.

**`bentend`** — The proposed peer daemon for general-purpose compute (containers, VMs, WASM workloads beyond Benten's own). A single Rust binary installable on any Linux. **Exploratory** (see [`future/bentend-daemon.md`](future/bentend-daemon.md)) — not on the committed roadmap.

**Benten Credits** — The platform currency. USD-pegged 1:1 initially. Treasury-backed (reserves in US Treasury bonds). Zero transaction fees within the network. **Phase 8 committed** (MVP).

**Benten Runtime** — The proposed WinterTC-compliant edge runtime that would run the Benten engine (compiled to WASM) as a host for communities. **Exploratory** (see [`future/benten-runtime.md`](future/benten-runtime.md)) — not on the committed roadmap. Distinct from the engine's WASM build target, which IS committed (Phase 2).

**CIDv1** — Content Identifier version 1. IPLD standard: version byte + multicodec + multihash. Benten hashes are valid CIDv1 values at 2 extra bytes cost.

**Code-as-graph** — The paradigm where application logic is represented AS graph structure, not stored IN graph properties. A handler is a subgraph of operation Nodes connected by control-flow Edges. The engine walks the subgraph to execute it.

**CURRENT pointer** — An Edge from an Anchor Node to the latest Version Node. Atomic update moves the pointer from version N to version N+1 within a storage transaction.

**DAG-CBOR** — The IPLD subset of CBOR with canonical encoding (map keys sorted). Used for content-addressed serialization via the `serde_ipld_dagcbor` crate. Equivalent to RFC 8949 deterministic encoding for string-keyed maps.

**Digital Garden** — A Benten community tier above Atrium. Community spaces like a Discord server, Wikipedia, or knowledge base — but decentralized. Member-mesh (each member syncs the graph locally, no central server). Admin-configured governance. **Phase 7 committed** (MVP). Full Groves with fractal/polycentric governance remain exploratory.

**ed25519-dalek** — The chosen Rust Ed25519 implementation for signatures.

**Fork is a right** — Any participant can fork any subgraph at any time, keeping full history. Creates evolutionary pressure toward good governance; communities that govern well retain members.

**GATE** — **Retired 2026-04-14.** Was in the original 12 primitives as a "custom logic escape hatch." Removed during primitive revision: capability checks now use the `requires` property on any Node (engine-enforced automatic BRANCH on failure); custom validation uses TRANSFORM or SANDBOX. See [`ENGINE-SPEC.md`](ENGINE-SPEC.md) Section 3 revision history.

**Grove** — A Benten community tier above Digital Garden. Governed communities with fractal, polycentric, polyfederated governance. Voting on rules, smart contracts as operation subgraphs, formal decision-making. Sub-Groves with inherited or overridden governance. Fork-and-compete dynamics. **Exploratory** — Gardens MVP (Phase 7) covers the simpler admin-governance tier; full Groves remain exploratory.

**HLC** — Hybrid Logical Clock. Monotonic timestamps combining physical and logical clocks, used for causal ordering in CRDT merge. Crate: `uhlc`.

**iroh** — The Rust P2P networking library chosen for Phase 3. QUIC-based, dial-by-public-key, built-in holepunching with relay fallback. Simpler and more modern than libp2p.

**IVM** — Incremental View Maintenance. The engine maintains materialized views (pre-computed query results) and updates them incrementally on writes. Read from view: O(1). Critical for event handler resolution, capability checks, content listings, governance.

**`KVBackend`** — The storage trait at `benten-graph` that abstracts over the underlying key-value store. Native implementation uses redb. Future WASM implementation will fetch content-addressed data from the peer network. Assumes minimal operations: get/put/delete/scan/atomic-batch. Introduced 2026-04-14 to keep the engine storage-backend-agnostic.

**`napi-rs`** — The Rust-to-Node.js binding framework. v3 (July 2025) compiles to both native and WASM targets from the same codebase, auto-generates TypeScript `.d.ts` files.

**`NoAuthBackend`** — The default implementation of the `benten-caps` pre-write hook trait. Allows all writes without capability checks. Ships as the engine's default so embedded/local-only users don't pay capability-system overhead. UCAN backend (Phase 3+) is the alternative for networked/multi-user deployments. Introduced 2026-04-14.

**Operation Node** — A Node representing one of the 12 operation primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM). Operation subgraphs are DAGs of these Nodes connected by NEXT / ON_ERROR / ON_NOT_FOUND / etc. edges. (Revised 2026-04-14: dropped VALIDATE + GATE, added SUBSCRIBE + STREAM.)

**Operation subgraph** — A handler represented as a DAG of operation Nodes. Bounded (max depth, max fan-out, max nodes, iteration budget). Deterministically evaluable. Content-hashed. Immutable once registered.

**PARA** — Projects, Areas, Resources, Archives. Knowledge organization methodology by Tiago Forte. Used by the Personal AI Assistant (Phase 6 committed) to organize user knowledge in the graph. **Projects** are finite outcomes with deadlines; **Areas** are ongoing responsibilities; **Resources** are topics of interest; **Archives** are inactive items from any of the other three.

**Paper prototype** — An empirical validation exercise (2026-04-11). Five real handlers were decomposed into operation subgraphs using the original 12 primitives to prove the vocabulary is sufficient. Result: 2.5% SANDBOX rate. See [`validation/paper-prototype-handlers.md`](validation/paper-prototype-handlers.md). **Note:** primitive set was revised 2026-04-14 (dropped VALIDATE + GATE, added SUBSCRIBE + STREAM). Re-validation against the revised set is a Phase 1 task.

**Personal AI Assistant** — Phase 6 committed deliverable. An AI assistant anchored to the user's own data that organizes their life via PARA methodology, generates tools on demand by composing operation subgraphs from primitives, and replaces the paid software stack with one the user owns. The adoption driver (Pillar 2 of the three-pillar vision).

**Proof of Sampling (PoSP)** — A compute verification strategy. Random re-execution of 5-10% of jobs on a different peer, compare output hashes. Reputation as penalty makes cheating irrational under Nash equilibrium. From Hyperbolic Labs research.

**Redb** — The chosen Rust embedded key-value store. Pure Rust, ACID, MVCC (concurrent readers with single writer), crash-safe via copy-on-write B-trees. v4 validated production-ready (April 2026).

**SANDBOX** — An operation primitive. The WASM computation escape hatch for Turing-complete work that cannot be expressed as operation Nodes. No re-entrancy. Fuel-metered via wasmtime. Max output 1MB.

**`serde_ipld_dagcbor`** — The CBOR serialization crate. Deterministic by default (sorts map keys during serialization). IPLD-native. Chosen over `ciborium` which does not sort map keys.

**STREAM** — An operation primitive added 2026-04-14. Produces partial/ongoing output with back-pressure. For Server-Sent Events, WebSocket messages, LLM token streams, large JSON responses, progress updates. Cannot be cleanly composed from RESPOND (terminal) + ITERATE (no back-pressure), so it earned primitive status.

**Subgraph** (operation subgraph) — See "Operation subgraph."

**SUBSCRIBE** — An operation primitive added 2026-04-14. Reactive change notification. The base primitive that IVM views, sync delta propagation, and event-driven handlers all compose on top. Extracted to make IVM composable rather than engine-internal — without SUBSCRIBE, IVM would be load-bearing in the evaluator; with SUBSCRIBE, IVM becomes its own crate that watches graph changes.

**Three Pillars** — The articulation of Benten's vision: (1) the engine for the decentralized web, (2) personal AI assistants as the adoption driver, (3) treasury interest on USD-pegged credits as the economic engine. See [`VISION.md`](VISION.md). Replaces the earlier "three products, one engine" framing.

**Thrum** — The TypeScript application platform (CMS, modules, web app) that consumes the Benten engine. Lives at `/Users/benwork/Documents/thrum/`. Migrated to run on the Benten engine in **Phase 4 committed**.

**`transactional: true`** — A subgraph property that wraps all WRITEs inside the subgraph evaluation in a single atomic transaction. If any WRITE fails, all WRITEs roll back. Load-bearing for multi-node transaction semantics. (Note: after engine-philosophy critic review, this is syntactic sugar over a transaction primitive in `benten-eval` that exposes begin/commit/rollback; the sugar is what the DSL surfaces, the primitive is what the engine implements.)

**Trust tier primitives** — Four orthogonal trust mechanisms workloads can require: cryptographic identity gating, reputation, TEE remote attestation, Verifiable Credentials + SBTs. Not a hierarchy — a filter.

**UCAN** — User-Controlled Authorization Networks. Capability-based auth tokens. Benten models them as Nodes with GRANTED_TO Edges. UCAN is one implementation of the `benten-caps` pre-write hook trait (Phase 3); `NoAuthBackend` is the default.

**Version chain** — The persistent history mechanism. Each versionable entity has an Anchor + Version Nodes + NEXT_VERSION edges + a CURRENT pointer. History = traverse. Undo = move CURRENT. Sync = exchange version Nodes. Fork = stop syncing, keep full history. Concurrent edits become a commit DAG. In the engine, this is an **opt-in convention** in `benten-core` — ephemeral data (cache, presence) does not pay versioning cost.

**`wasmtime`** — The WASM runtime chosen for SANDBOX. Rust-native, Bytecode Alliance, fuel-metered execution, Component Model support. Replaces the initial `@sebastianwessel/quickjs` reference (which was a JS library and inappropriate for a Rust engine).

**WinterTC** — Ecma TC55, the standard for cross-platform edge runtimes (formerly WinterCG). Defines a Minimum Common API implemented by Node, Deno, Bun, Cloudflare Workers, Vercel Edge, Netlify, Fastly. The engine's WASM build target will target WinterTC-compliant environments (Phase 2 committed). A full **Benten Runtime** product that hosts communities on WinterTC is exploratory — see [`future/benten-runtime.md`](future/benten-runtime.md).
