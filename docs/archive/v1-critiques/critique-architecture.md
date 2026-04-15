# Benten Engine Specification — Architecture Critique

**Date:** 2026-04-11
**Reviewer:** Architecture Purity Agent
**Scope:** `docs/SPECIFICATION.md`, `CLAUDE.md`, 5 research explorations, `V4-ARCHITECTURAL-EVOLUTION.md`

---

## Score: 6/10

**Justification:** The vision is clear and genuinely compelling. The research is exceptional — 15 explorations, honest assessments of alternatives, principled elimination of options. The synthesis ("nothing does everything, so compose from Rust crates") is correct. But the specification has a gap between vision and buildable artifact: it describes WHAT the engine does without specifying HOW the hardest parts work. The 10-crate structure front-loads concerns that should be deferred. And the spec contradicts the platform's own "thin engine" philosophy by baking too much into the engine layer.

---

## 1. Over-Engineering

### 1.1 Ten crates is too many for a first release

The proposed structure has 10 crates. For context, `redb` (the proposed storage backend) is a single crate. `petgraph` is a single crate. `sled` is a single crate. IndraDB — an actual graph database — has 3 crates (core, client, server). The 10-crate structure makes sense as an eventual decomposition, but building all 10 before you have a working system means you are designing abstractions before you know what the boundaries should be.

The first release should be 3-4 crates maximum:

| Crate | Contains | Why |
|-------|----------|-----|
| `benten-core` | Node, Edge, Value, types | Same as spec |
| `benten-graph` | In-memory graph + indexes + persistence (redb) + MVCC + transactions | Merge graph, persist, and the concurrency model into one crate. Persistence is not separable from the graph — it defines the consistency model. |
| `benten-query` | Cypher parser + planner + execution | This is logically separate (you can use the graph without Cypher) |
| `benten-engine` | Orchestrator + napi-rs bindings | Ties it together |

Everything else — IVM, versioning, capabilities, sync, reactive — should be proven as patterns on top of the graph before they get their own crates. The spec even admits this: Section 8 lists "How much IVM in first release?" as an open question. The answer should be: prove the concept with the graph primitives first, then extract.

### 1.2 CRDT sync in a first release is scope creep

The spec puts `benten-sync` in the build order. The research documents explicitly recommend starting with single-instance (Option B: optimistic locking) for V1. CRDT sync is a massive undertaking — yrs and automerge are each thousands of lines with years of development. Including sync in the spec creates the illusion that it is part of the first deliverable. It should be an explicitly deferred crate with a clear "syncs with no engine changes" design constraint.

### 1.3 The reactive crate duplicates IVM

`benten-reactive` (subscription management, change notifications) and `benten-ivm` (incremental view maintenance) are the same concern. IVM maintains computed results incrementally. Reactivity is "notify when a result changes." The notification IS the IVM system telling you the view was updated. These should be one crate. Splitting them creates an artificial boundary: you need the IVM dependency graph to know which subscriptions to fire, and you need the subscription infrastructure to deliver IVM results. They are inseparable.

### 1.4 Version chains don't need their own crate

The explore-graph-native-protocol.md document makes a strong case that version chains are a PATTERN on existing primitives (Nodes + Edges + conventions), not a new primitive. Quoting the document: "The thin engine test: Version chains are a pattern on top of the existing primitives (Nodes + Edges), not a new primitive." If that is true — and the research argues convincingly that it is — then `benten-version` is a library of helper functions for creating and traversing a specific Node/Edge pattern. That is a module, not a crate. It belongs in `benten-graph` as a submodule, or in a `patterns/` directory in the engine crate.

---

## 2. Under-Specification

### 2.1 IVM dependency tracking algorithm is completely unspecified

The spec says: "When a write occurs, the engine identifies which views are affected. Affected views are incrementally updated (not recomputed from scratch)." This is the ENTIRE value proposition of the engine, and it has zero specification.

Questions that must be answered before writing code:

- **What is the dependency tracking data structure?** A DAG of query nodes? A dataflow graph? A red-green tree (like rustc's incremental compilation)?
- **How are view dependencies captured?** When `createView()` is called with a Cypher query, how does the engine determine which Nodes/Edges the view depends on? Does it analyze the query statically (parse the MATCH pattern and extract label/property references)? Or does it trace the query execution dynamically (record every Node/Edge read)?
- **What is the incremental update algorithm?** Differential dataflow (Materialize/Feldera approach)? Semi-naive evaluation (Datalog approach, as in crepe/datafrog)? Red-green invalidation (compiler approach)? Each has fundamentally different performance characteristics and implementation complexity.
- **What is the granularity of invalidation?** If Node X changes and view V reads 1000 Nodes including X, does V recompute entirely, or does the engine compute a delta? For a simple "MATCH (n:Page) RETURN n ORDER BY n.updatedAt DESC LIMIT 20" view, does changing one page re-sort 10,000 pages or insert/remove one entry?

The spec references "crepe or datafrog" as dependencies, which suggests semi-naive Datalog evaluation. But the query language is Cypher, not Datalog. The spec does not explain how Cypher queries are lowered to a form that Datalog engines can maintain incrementally. This is a hard research problem — Materialize employs dozens of engineers to solve it for SQL.

**Recommendation:** Specify the IVM algorithm explicitly. The most pragmatic path:

1. Use the red-green invalidation pattern from rustc: track which graph elements each view reads, mark views dirty when those elements change, lazily recompute on next read.
2. This is NOT true IVM (it recomputes fully, just lazily). But it delivers 80% of the value with 10% of the complexity.
3. True incremental maintenance (differential dataflow) should be a Phase 2 optimization for specific view patterns, not the initial implementation.

### 2.2 MVCC is mentioned but not designed

The spec says "MVCC: readers see a consistent snapshot while writers modify" and "Concurrent readers: Unlimited (MVCC snapshots)." But there is no design for:

- **Isolation level:** Snapshot isolation? Read committed? Serializable? These have different correctness guarantees and wildly different implementation complexity.
- **Snapshot mechanism:** How is a snapshot created? Copy-on-write? Multi-version with garbage collection? Timestamp-based? The spec mentions `dashmap` for concurrent HashMap, but dashmap does not provide MVCC — it provides concurrent access with per-shard locking.
- **Garbage collection:** If every read creates a snapshot and every write creates a new version, when are old versions reclaimed? Without GC design, memory grows without bound.
- **Interaction with version chains:** Version chains already store historical state as Nodes. Are MVCC snapshots the same as version Nodes, or are they a separate mechanism? If separate, you have two versioning systems. If the same, how does transaction rollback interact with the version chain (you don't want to create a permanent version Node for a transaction that gets rolled back)?

**Recommendation:** Look at SurrealKV and SkipDB for reference implementations. Start with snapshot isolation (the most common for embedded databases). Design MVCC and version chains as ONE system — a transaction creates a tentative version Node that is promoted to permanent on commit and discarded on rollback.

### 2.3 The Cypher parser strategy is unspecified

The spec says Cypher is the query language but does not specify whether to:

1. **Use the existing `open-cypher` crate** (pest-based parser, parses openCypher subset, last updated Feb 2025, minimal ecosystem)
2. **Use KyuGraph's hand-written parser** (more complete, but part of a separate database project)
3. **Write a custom parser** (maximum control, significant effort)
4. **Use the openCypher ANTLR grammar** and generate a Rust parser

Each option has different tradeoffs. The open-cypher crate covers a subset of Cypher. Writing a full Cypher parser is a months-long project. GQL (ISO/IEC 39075, published April 2024) is the successor to openCypher — should the engine target GQL instead?

**Recommendation:** Start with the `open-cypher` crate for parsing, with a Rust-native API as the primary interface and Cypher as a convenience layer. This matches open question #1 in the spec. The Cypher subset supported by openCypher covers MATCH, WHERE, RETURN, ORDER BY, LIMIT — which is sufficient for the Thrum use cases documented in the research.

### 2.4 Performance targets lack methodology

The spec lists targets like "Node lookup by ID: <0.01ms" and "Materialized view read: <0.01ms" but does not specify:

- Under what load? One reader? 100 concurrent readers?
- With what data size? 1,000 Nodes? 1,000,000 Nodes?
- On what hardware? The targets are meaningless without a reference machine.
- Measured how? End-to-end including napi-rs overhead? Rust-only?

0.01ms (10 microseconds) for a Node lookup is achievable for an in-memory HashMap lookup in Rust (a HashMap lookup is ~50-100ns). But the moment you add persistence (redb), MVCC (snapshot creation), and napi-rs serialization (Rust->JS conversion), you are likely at 10-100 microseconds, not sub-10. The targets should be specified as Rust-internal (no serialization overhead) vs TypeScript-visible (including napi-rs).

---

## 3. Contradictions

### 3.1 "Thin engine" philosophy vs 10-crate engine

The V4-ARCHITECTURAL-EVOLUTION.md document explicitly states the engine philosophy:

> **What the Engine IS:** Node, Edge, Store interface. That's it. Three things. Everything else composes.

Then it lists as "NOT engine": Events, Versioning, Capabilities, Services, Code execution, Sync, P2P, Rendering, Multi-tenancy.

But the specification puts IVM (events/computation), version chains, capabilities, sync, and reactive notifications IN the engine. Five of the nine things the architecture document says are "not engine" are in the engine specification.

This is the most fundamental contradiction in the documents. Either the thin engine philosophy is correct (and the spec should be smaller), or the spec is correct (and the architecture document should be updated to reflect that the engine is thick by design).

**My read:** The thin engine philosophy is correct for the TypeScript layer (where the engine is Node + Edge + Store). But the Rust engine is a different beast — it IS the database, and databases contain concurrency control, persistence, and query execution natively. The contradiction arises from conflating two meanings of "engine." Resolve it by being explicit: "The Rust binary is thick (it contains a database). The API exposed to TypeScript is thin (Nodes, Edges, queries, subscriptions)."

### 3.2 "Graph IS the runtime" vs IVM pre-computation

Section 2.2 says materialized views are pre-computed and reads are O(1). Section 6.2 of the graph-native protocol doc says event handlers should "stay in-memory" because "the event system is the engine's hot path." But if IVM exists and maintains views at O(1), why would event handlers need to be in-memory? The IVM view IS the in-memory cache.

This suggests the spec author is not fully confident that IVM can serve the hot path. If IVM works as described, the "in-memory vs graph" distinction dissolves — the materialized view IS the in-memory cache, maintained by the graph. If IVM cannot serve the hot path (because maintenance overhead is too high), then the "answers exist before questions" claim is overstated.

**Resolution:** Be honest about IVM scope. IVM for content listings and permission checks (updated infrequently, read frequently) is high-value and achievable. IVM for event handler resolution (updated at startup, read on every event) is achievable but the maintenance overhead must be measured. IVM for arbitrary Cypher queries (updated in real-time) is a research project, not a V1 feature.

### 3.3 Performance targets contradict CRDT sync overhead

"Node creation: <0.01ms" and "Sync (100 version Nodes): <10ms." But if every Node creation triggers version chain maintenance, IVM updates, CRDT metadata (HLC timestamps), and persistence (WAL write) — the 0.01ms target is unrealistic. A single redb write is ~10-100 microseconds. Adding version chain creation, IVM dependency checking, and HLC timestamp generation puts you well above 0.01ms.

**Recommendation:** Separate "in-memory only" targets from "durable write" targets. The 0.01ms target is achievable for in-memory operations. Durable writes will be 0.1-1ms depending on persistence strategy.

---

## 4. Wrong Abstractions

### 4.1 benten-query cannot be decoupled from benten-graph

The spec places the query engine in a separate crate from the graph. But query planning requires intimate knowledge of the graph's index structures, storage layout, and statistics (cardinality estimates). A query optimizer that does not know the graph's B-tree layout cannot produce good plans. In practice, `benten-query` will depend heavily on `benten-graph` internals.

**Recommendation:** Make `benten-query` a feature flag or submodule of `benten-graph`, not a separate crate. The graph defines the storage; the query engine exploits that storage structure. They are one concern.

### 4.2 benten-persist should be part of benten-graph

The spec separates persistence (WAL, snapshots, redb) from the graph (storage, indexes, traversal). But persistence IS the graph's storage. The graph's in-memory representation and its on-disk representation must be co-designed — you cannot design one without the other. Decisions like "are edges stored inline with nodes or in a separate B-tree?" and "is the adjacency list stored as a sorted array or a hash map?" depend on whether the primary copy is in-memory or on-disk.

This is already acknowledged implicitly: Section 4.2 lists `redb` as a dependency of the overall project, but the spec does not say which crate owns it. If `benten-persist` owns redb and `benten-graph` is in-memory only, then every durable operation requires crossing a crate boundary. If `benten-graph` also uses redb directly, then `benten-persist` is redundant.

**Recommendation:** Merge `benten-persist` into `benten-graph`. The graph crate owns both in-memory and on-disk representation. Expose a `StorageBackend` trait if you want pluggable backends (redb vs memory vs RocksDB), but the trait lives in `benten-graph`.

### 4.3 Capability enforcement placement

`benten-capability` is listed as a separate crate, but capabilities are checked "at every operation boundary" (Section 2.4). This means every method in `benten-graph` (createNode, getNode, createEdge, traverse) must call into `benten-capability` for permission checking. This creates a circular dependency concern: graph operations need capability checks, but capabilities are stored AS Nodes in the graph.

**Recommendation:** Define a `CapabilityChecker` trait in `benten-core`. Implement it in `benten-capability`. Inject it into `benten-graph`. This breaks the circular dependency and allows the graph to operate without capabilities (for testing, for bootstrapping).

---

## 5. Missing Concerns

### 5.1 No backup/restore design

For a system that claims "every person, family, or organization runs their own instance," backup and restore is critical. The spec mentions WAL and snapshots but does not address:

- How does a user back up their instance?
- How does restore work? Is it "replay the WAL from the last snapshot"?
- What about point-in-time recovery?
- Can you export the graph as a portable format (JSON-LD, RDF, custom)?

### 5.2 No schema evolution strategy

The research documents identify schema evolution during sync as an open question. Content types change over time — fields are added, removed, renamed. When Instance A has schema v2 and Instance B has schema v1, what happens during sync? The spec does not address this.

### 5.3 No memory management design

The spec targets WASM deployment, but WASM environments have limited memory (typically 2-4GB). With Nodes, version chains, IVM materialized views, and CRDT metadata all in memory, the engine needs explicit memory budgets and eviction policies. None are specified.

### 5.4 No testing strategy for the engine itself

The spec mentions "cargo test + criterion" but does not address:

- **Property-based testing** (proptest/quickcheck): Essential for a database. "For all possible sequences of operations, the graph maintains consistency."
- **Fuzz testing** (cargo-fuzz): Essential for the Cypher parser. Malformed queries should never crash the engine.
- **Deterministic simulation testing** (like FoundationDB's approach): Essential for CRDT sync. "For all possible message orderings, convergence is guaranteed."
- **Jepsen-style correctness testing**: If the engine claims ACID transactions and MVCC, it must be tested under failure conditions.

### 5.5 No observability design

No mention of tracing (tokio-tracing), metrics (prometheus), or structured logging. For a database engine, observability is not optional — it is how you debug production issues.

### 5.6 No error model specified

The existing Thrum engine has `EngineError` with 17 error codes. The Rust engine spec does not define its error types. What errors can the Cypher parser return? What errors can a capability violation produce? Are errors typed enums (Rust-idiomatic) or string codes (TypeScript-compatible)?

---

## 6. The Thin Engine Test

The Thrum philosophy is "thin engine, everything composed." The V4 architecture document says:

> "If someone uses the engine to build a real-time game with no CMS, no admin, no content types, no sync, no P2P — does the engine have dead weight?"

Under the current spec, the answer is **yes**. The game developer gets:

- **IVM** — useful, but only if they define views
- **Version chains** — dead weight if the game doesn't need undo/history
- **Capabilities** — dead weight if the game has its own auth system
- **CRDT sync** — dead weight if the game is single-player
- **Reactive notifications** — useful

The spec should define a minimal core and make everything else opt-in:

**Minimal core (always present):**
- benten-core: types
- benten-graph: graph storage + indexes + persistence + MVCC + transactions + queries

**Opt-in features (Cargo feature flags or separate crates):**
- Version chains (convention on top of graph primitives)
- IVM + reactive (maintains views, fires notifications)
- Capabilities (UCAN grants, checked at operation boundaries)
- Sync (CRDT merge, deferred to post-V1)

This way the game developer gets `benten-core` + `benten-graph` and pays for nothing else. Feature flags in Rust are zero-cost when disabled — no binary bloat, no runtime overhead.

---

## 7. Build Order and Minimal Viable First Release

### Proposed build order from spec:

```
core -> graph -> persist -> version -> capability -> ivm -> query -> reactive -> sync -> engine
```

### Critique:

1. **persist should be merged with graph** (see Section 4.2)
2. **version should come after IVM**, not before. Version chains are a pattern on graph primitives — you don't need special infrastructure to create them. IVM is the harder problem that will reshape the graph's internal design.
3. **capability should come after query**. You need to be able to query the graph to check capabilities (capabilities are stored as Nodes). Building the capability system before you can query the graph means hardcoding capability lookups.
4. **sync should be explicitly out of V1 scope**

### Recommended build order:

```
Phase 1 (Minimal viable engine — proves the concept):
  benten-core (types) -> benten-graph (graph + persistence + MVCC + basic traversal)

Phase 2 (Query and compute):
  benten-query (Cypher parser + execution against benten-graph)

Phase 3 (Smart reads):
  benten-ivm (incremental view maintenance + reactive notifications, ONE crate)

Phase 4 (Security):
  benten-capability (UCAN grants, injected into graph via trait)

Phase 5 (Orchestrator + bindings):
  benten-engine (ties everything together) + napi-rs bindings

Phase 6 (Post-V1, proven needed):
  benten-sync (CRDT merge)
  benten-version (helper library for version chain patterns, if needed as separate crate)
```

### What proves the concept (Phase 1-2):

A working graph database that you can:
1. Create/read/update/delete Nodes and Edges
2. Persist to disk and recover
3. Query with Cypher
4. Access from Node.js via napi-rs

This is enough to validate the entire architecture. If Cypher queries against the native graph are faster than PostgreSQL+AGE queries (which they should be — no network hop, no SQL overhead), you have proven the core thesis. Everything else is incremental improvement on a working system.

### What proves IVM (Phase 3):

After the graph works, add IVM for ONE use case: content listing. Define a view, insert content Nodes, verify the view updates incrementally. If the IVM overhead on writes is acceptable and the read speedup is real, the thesis is proven. Then generalize.

---

## 8. Specific Recommendations

### File: `docs/SPECIFICATION.md`

**S1.** Add a "Phases" section that defines what is in V1 vs V2 vs V3. The current spec reads as "everything at once." Explicit phasing prevents scope creep and aligns expectations.

**S2.** Section 2.2 (IVM): Replace the hand-wavy "engine identifies which views are affected" with a specific algorithm description. Recommend starting with the red-green invalidation pattern (mark dirty on write, recompute lazily on read) and upgrading to differential dataflow for high-frequency views in a later phase.

**S3.** Section 2.7 (Concurrency): Specify the isolation level (recommend snapshot isolation). Design MVCC and version chains as one system. Define garbage collection for old snapshots.

**S4.** Section 4.1 (Crate Structure): Reduce to 4-5 crates for V1. Merge persist into graph. Merge reactive into IVM. Make version a submodule, not a crate. Defer sync.

**S5.** Section 4.2 (Dependencies): Remove `crepe or datafrog` until you have a concrete plan for Datalog evaluation of Cypher queries. Replace with `open-cypher` for parsing.

**S6.** Section 4.3 (API Surface): The TypeScript API is well-designed. Add error types for each operation. Specify what happens on invalid input (malformed Cypher, nonexistent NodeId, capability violation).

**S7.** Section 6 (Performance Targets): Split into "Rust-internal" and "TypeScript-visible (including napi-rs overhead)." Add data size and concurrency columns. Add a "Reference hardware" row.

### File: `CLAUDE.md`

**S8.** Update the crate structure to reflect the reduced V1 scope. The current structure implies all 10 crates are in scope.

**S9.** Add a "V1 Definition of Done" section that lists the exact capabilities of the first release.

### New file needed: `docs/IVM-DESIGN.md`

**S10.** The IVM system is the core innovation. It deserves its own design document specifying: the dependency tracking data structure, the invalidation algorithm, the update granularity, the interaction with MVCC, and benchmark criteria.

### New file needed: `docs/MVCC-DESIGN.md`

**S11.** MVCC is critical infrastructure. Document: isolation level, snapshot mechanism, garbage collection, interaction with version chains, interaction with IVM.

---

## 9. If I Were Starting From Scratch

I would build this as **two crates plus bindings**:

1. **benten-graph**: A persistent property graph with snapshot-isolation MVCC, B-tree and hash indexes, Cypher query support (via `open-cypher` crate), and a `CapabilityChecker` trait for injection. Storage via redb. This is the database.

2. **benten-ivm**: Incremental view maintenance and reactive subscriptions, built as a layer on top of benten-graph. Intercepts writes, maintains a dependency graph, invalidates/updates materialized views, fires subscription callbacks. This is the compute layer.

3. **benten-napi**: napi-rs bindings exposing the TypeScript API from Section 4.3.

Everything else — version chain helpers, capability grant management, UCAN serialization, CRDT sync — would be TypeScript libraries or Rust libraries added as needed, proven by usage before being promoted to engine crates.

The reason for this radical simplification: **you are building a database.** Database engineering is one of the hardest problems in computer science. The fewer concerns in your initial implementation, the more likely you are to get the fundamentals right. MVCC correctness, persistence durability, query planning, and IVM maintenance are each individually hard enough to deserve your full attention. Adding capabilities, sync, and version chains to the same development timeline dilutes focus on the problems that MUST be right for the engine to be trustworthy.

Ship the graph database. Ship IVM. Prove the thesis. Then add capabilities and sync as modules that compose on top of a proven foundation.

---

## Summary of Issues

| # | Category | Issue | Severity |
|---|----------|-------|----------|
| 1 | Over-engineering | 10 crates is too many for V1 | High |
| 2 | Over-engineering | CRDT sync in first release is scope creep | High |
| 3 | Over-engineering | Reactive and IVM are the same concern | Medium |
| 4 | Over-engineering | Version chains don't need their own crate | Medium |
| 5 | Under-specification | IVM dependency tracking algorithm unspecified | Critical |
| 6 | Under-specification | MVCC design missing | Critical |
| 7 | Under-specification | Cypher parser strategy unspecified | High |
| 8 | Under-specification | Performance targets lack methodology | Medium |
| 9 | Contradiction | "Thin engine" philosophy vs thick engine spec | High |
| 10 | Contradiction | "Graph IS the runtime" vs "keep handlers in-memory" | Medium |
| 11 | Contradiction | Performance targets vs CRDT/persistence overhead | Medium |
| 12 | Wrong abstraction | benten-query tightly coupled to benten-graph | Medium |
| 13 | Wrong abstraction | benten-persist should be part of benten-graph | Medium |
| 14 | Wrong abstraction | Capability circular dependency with graph | Medium |
| 15 | Missing concern | No backup/restore design | High |
| 16 | Missing concern | No schema evolution strategy | High |
| 17 | Missing concern | No memory management design (WASM) | High |
| 18 | Missing concern | No property-based/fuzz/simulation testing plan | High |
| 19 | Missing concern | No observability (tracing, metrics) | Medium |
| 20 | Missing concern | No error model specified | Medium |

**Critical issues (must resolve before writing code):** #5 (IVM algorithm), #6 (MVCC design)
**High issues (must resolve before V1 ships):** #1, #2, #7, #9, #15, #16, #17, #18

---

## Sources

Research referenced:
- [Rust Incremental Compilation — Red/Green Algorithm](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html)
- [Materialize — Incremental Computation in the Database](https://materialize.com/guides/incremental-computation/)
- [Adapton — Incremental Computation for Rust](https://docs.rs/adapton)
- [IndraDB — Graph Database in Rust](https://github.com/indradb/indradb)
- [open-cypher — Cypher Parser in Rust](https://github.com/a-poor/open-cypher)
- [KyuGraph — Embedded Property Graph Database in Rust](https://github.com/offbit-ai/kyugraph)
- [SurrealKV — Versioned Embedded KV with MVCC](https://github.com/surrealdb/surrealkv)
- [SkipDB — Serializable Snapshot Isolation](https://github.com/al8n/skipdb)
- [Stoolap — Embedded SQL Database with MVCC](https://stoolap.io/)
- [CrepeDB — MVCC with Snapshot Isolation](https://lib.rs/crates/crepedb-redb)
- [openCypher Specification](https://opencypher.org/)
- [GraphLite — ISO GQL Standard in Rust](https://crates.io/crates/graphlite)
