# Benten Engine Specification -- Correctness Critique

**Date:** 2026-04-11
**Reviewer:** Correctness & Edge Cases Agent
**Document reviewed:** `docs/SPECIFICATION.md` + `CLAUDE.md`
**Supporting data:** Grafeo load test results, Grafeo investigation results, PGlite+AGE spike results, `explore-database-is-application.md`, `explore-execution-in-data-2026.md`, `explore-in-process-graph-db-2026.md`

---

## Correctness Score: 4/10

The specification describes a compelling vision but contains multiple claims that contradict the project's own empirical data, underspecifies critical subsystems to the point where feasibility cannot be assessed, and conflates aspirational targets with engineering commitments. Several core mechanisms (IVM, MVCC, CRDT conflict resolution) are named but not defined with sufficient rigor to implement, test, or estimate costs.

---

## 1. Confirmed Bugs (Logic Errors in the Specification)

### BUG-1: Performance targets contradict spike data and violate physics of the proposed architecture

**The claim (Section 6):**
- Node creation: <0.01ms (10 microseconds)
- Edge creation: <0.01ms
- Both stated as "Write + IVM update"

**What the data says:**

Grafeo -- which does NO IVM -- achieves `createNode()` at ~4.2 microseconds per op at the 10K tier (238K ops/s at 50K). That is the baseline write cost WITHOUT any view maintenance, WITHOUT any version chain creation, WITHOUT any capability check, WITHOUT any WAL flush.

The spec's own architecture adds to every write:
1. Create the data node/edge
2. Create a version node (version chain)
3. Create NEXT_VERSION edge
4. Update CURRENT pointer edge
5. Identify affected materialized views (IVM dependency scan)
6. Incrementally update each affected view
7. Check capabilities (materialized view read)
8. WAL write for durability
9. Notify reactive subscribers

Even if steps 1-3 each cost 4 microseconds (Grafeo baseline), the write is already 12 microseconds BEFORE IVM, capability checks, or persistence. The <0.01ms target for "Write + IVM update" is physically impossible given the architecture described.

**Severity:** Critical. Downstream decisions (CRDT sync budgets, IVM complexity budget, transaction composition) will be based on these targets. If the real write cost is 50-200 microseconds (a reasonable estimate for all the above steps on fast hardware), the entire system's throughput model changes.

**Fix:** Benchmark the actual write path on the proposed architecture. Budget each step separately. A realistic write target for the full path is likely 0.05-0.2ms, not <0.01ms.

### BUG-2: "Concurrent writers: Lock-free or fine-grained — Per-Node locking" contradicts MVCC

**The claim (Section 6):** The concurrency model claims both "MVCC snapshots for readers" AND "lock-free or fine-grained per-Node locking" for writers.

**The contradiction:** MVCC and per-node locking are different concurrency control mechanisms that serve different purposes and have different failure modes:

- **MVCC** means readers never block and see a consistent snapshot. Writers create new versions. This is well-defined (PostgreSQL, Datomic, etc.).
- **Per-Node locking** means writers acquire locks on specific nodes. This is pessimistic concurrency control.
- **Lock-free** means no locks at all -- typically achieved via CAS (compare-and-swap) operations on atomic variables.

These three cannot simultaneously be the concurrency model. "Lock-free or fine-grained" suggests the decision hasn't been made, but the spec presents it as a completed design. The interaction between per-node locking and MVCC snapshot isolation is non-trivial -- what happens when writer A holds a lock on Node X and writer B's transaction snapshot includes Node X? Does B see the pre-lock version? Does B's commit fail if A modified X?

**Severity:** High. The concurrency model is the single most impactful architectural decision for a multi-writer database. Leaving it as "lock-free or fine-grained" in a specification document means the spec does not actually specify the behavior.

**Fix:** Choose one model and specify it completely. Recommendation: MVCC with optimistic concurrency control (like Datomic or PostgreSQL's serializable snapshot isolation). Per-node locking is a poor fit for a graph database because graph traversals touch many nodes and locking during traversal creates deadlock risks.

### BUG-3: CRDT "add-wins for edges" creates orphan references by design

**The claim (Section 2.5):** "Edges: add-wins semantics"

**The problem:** Instance A creates edge `(X)-[:KNOWS]->(Y)`. Instance B deletes Node Y. After sync, add-wins means the edge exists but points to a deleted node. The spec acknowledges this in the critique prompt but provides no resolution.

This is not an edge case -- it is the NORMAL operation of any system where two instances modify related data concurrently. In a CMS context: Instance A adds a block reference to composition Z pointing to composition Y. Instance B deletes composition Y. After sync, composition Z references a deleted composition. `resolveComposition()` will fail or produce broken output.

The current Thrum codebase handles this via `composition_refs` with FK constraints (`ON DELETE RESTRICT`). The engine spec's CRDT model explicitly removes this safety mechanism by allowing add-wins to override deletions.

**Severity:** High. This will corrupt data during normal multi-instance operation. Every graph CRDT system must solve the "tombstone + dangling reference" problem. The spec doesn't mention tombstones at all.

**Fix:** The spec must define:
1. How deleted nodes are represented (tombstones? version chain with deleted marker?)
2. How edges pointing to deleted nodes are handled on sync receipt (reject? mark as broken? cascade delete?)
3. Whether edge-add-wins applies universally or only when the target node is alive
4. Conflict resolution UI/API for structural conflicts that cannot be auto-resolved

---

## 2. Race Conditions

### RACE-1: IVM update ordering during concurrent writes

Two concurrent transactions each create a node. Both trigger IVM updates to the same materialized view (e.g., "all content of type page"). What is the ordering guarantee?

If IVM updates are applied in transaction commit order, the system must serialize commits for any transactions that affect the same view. This reintroduces the single-writer bottleneck that the spec claims to eliminate.

If IVM updates are applied independently per transaction, the materialized view can become inconsistent (both transactions see a partial view during their IVM update phase).

The spec says nothing about this. Materialize and Feldera solve it via deterministic dataflow processing with explicit timestamps. The spec needs an equivalent mechanism.

### RACE-2: Version chain CURRENT pointer during concurrent updates

Two writers update the same node concurrently:
1. Writer A reads anchor -> CURRENT -> v3
2. Writer B reads anchor -> CURRENT -> v3
3. Writer A creates v4, updates CURRENT to v4
4. Writer B creates v4' (based on v3), updates CURRENT to v4'

Now v4 exists but is orphaned -- CURRENT points to v4', and v4 has no incoming CURRENT edge. The version chain forks: v3 -> v4 and v3 -> v4'. NEXT_VERSION edges create a DAG, not a chain.

This is the classic lost-update problem. The spec mentions "serializable transactions for atomic multi-operation writes" but doesn't specify how the version chain model interacts with concurrent writes to the SAME node. PostgreSQL solves this with row-level locks or serializable snapshot isolation (SSI) that detects read-write conflicts. The spec's "per-Node locking" might solve this but contradicts the "lock-free" claim.

### RACE-3: Capability revocation during in-flight operations

A capability is revoked while an operation that depends on that capability is in progress:
1. Module X has capability `{domain: 'store', action: 'write', scope: 'content/*'}`
2. Module X begins a write transaction (capability checked: passes)
3. Admin revokes Module X's capability
4. Module X's transaction commits (capability was valid at start, revoked during execution)

Should the commit succeed? The spec says capabilities are "checked at every operation boundary" but doesn't define whether that means at transaction start, at each operation within the transaction, or at commit time. Each choice has different correctness and performance implications.

---

## 3. Null Safety Gaps / Underspecification

### NULL-1: IVM algorithm is entirely unspecified

The spec says "the engine identifies which views are affected" and "affected views are incrementally updated (not recomputed from scratch)" but provides zero detail on HOW.

IVM is a research-grade problem with multiple approaches, each with different tradeoffs:
- **Differential Dataflow** (Materialize, Feldera): Represents changes as collections of `(data, time, diff)` triples. Requires a total ordering of times. Works for arbitrary SQL but has complex implementation.
- **Counting-based IVM**: Tracks how many times each row appears in the view. Additions increment, deletions decrement. Simple for SPJ (select-project-join) queries but breaks for aggregation, DISTINCT, LIMIT.
- **Trigger-based IVM**: Rewrites the view definition into delta rules that fire on changes. Works for simple views but becomes combinatorially complex for multi-way joins.

The spec claims IVM will work for:
- Event handler resolution (pattern match + sort by priority)
- Capability checks (edge traversal + property filter)
- Content listings (label scan + edge hop + ORDER BY + LIMIT)

The third case is the hardest. Maintaining a sorted, paginated, incrementally-updated view of content is equivalent to maintaining a top-K sorted index. When a new content item is created, the engine must determine whether it displaces an existing item in the top-K, which requires comparison with all items in the view. When an item's sort key changes, it must be repositioned. This is not O(1) maintenance -- it is O(log n) at best (B-tree insert) and the spec claims O(1).

**Why this matters:** The "database-is-application" exploration (`explore-database-is-application.md`) explicitly noted that IVM can be applied incrementally to Thrum's existing hot paths WITHOUT building a general IVM engine. It recommended `createMaterializedIndex<K, V>()` as a practical first step. The spec instead commits to a general-purpose IVM system without specifying the algorithm, which is a much larger scope.

### NULL-2: MVCC garbage collection is unspecified

MVCC creates multiple versions of data (that's the point -- readers see snapshots). But old versions must eventually be cleaned up, or memory grows without bound.

The spec mentions:
- MVCC snapshots for readers
- Version chains (anchor + NEXT_VERSION + CURRENT)

These are TWO separate versioning mechanisms. MVCC creates transient versions visible only during a transaction. Version chains create persistent versions for history/undo. How do these interact? Does an MVCC snapshot see the version chain? If a reader starts a snapshot while writer A is creating version v4, does the reader see v3 (snapshot isolation) or the CURRENT pointer (which might be mid-update)?

Garbage collection strategies for MVCC include:
- PostgreSQL's VACUUM (periodic cleanup of dead tuples)
- Datomic's approach (immutable, no GC needed for data, only for indexes)
- LMDB's copy-on-write B-tree (old versions freed when no readers reference them)

The spec doesn't pick one, or even acknowledge the problem exists.

### NULL-3: Cypher parser scope and effort is unacknowledged

The spec lists `benten-query` as a crate that provides "Cypher parser + query planner." Section 8 asks "Should the Cypher query language be the primary API?" as an open question.

Building a full Cypher parser is a multi-month effort. OpenCypher's grammar has ~200 production rules. Key Cypher features include:
- Pattern matching with variable-length paths (`*1..3`)
- OPTIONAL MATCH
- WITH clause (query pipelining)
- UNWIND (list comprehension)
- Aggregation (count, collect, sum, avg, min, max)
- CASE expressions
- Subqueries (EXISTS, CALL)
- CREATE, MERGE, SET, DELETE, REMOVE
- Parameter injection
- Type coercions

Existing Rust Cypher parsers:
- `cypher-parser-rs`: Rust bindings to libcypher-parser (C library). Mature but FFI overhead.
- No pure-Rust production-grade Cypher parser exists as of the research.

The spec should acknowledge this is 3-6 months of work for a capable team, or scope down to a subset (e.g., read-only pattern matching with property filters).

### NULL-4: petgraph memory model at 20M nodes

The spec lists `petgraph` as the graph data structure. petgraph uses adjacency lists stored in Rust `Vec`s. This is an in-memory data structure.

For 20M nodes with properties (estimated 200-500 bytes per node including HashMap overhead in Rust):
- Nodes alone: 4-10 GB
- Edges (assuming 3:1 edge-to-node ratio): 12-30 GB for edge metadata
- Version chains (3 versions per node average): triple the node count
- MVCC snapshots: additional copies of modified data

Conservative estimate: 20-50 GB for a 20M node graph with version chains. This exceeds the RAM of most developer machines and many production servers.

The Grafeo spike showed 2KB/node RSS at scale (stabilizing at Large tier). At 20M nodes, that's 40 GB -- and Grafeo has NO version chains, NO IVM state, NO MVCC overhead.

petgraph is the wrong foundation for this scale. The spec needs either:
1. A disk-backed graph structure (B-tree indexed adjacency lists via redb)
2. A memory-mapped approach (mmap the graph, let the OS handle paging)
3. A hybrid where hot subgraphs are in petgraph and cold data is on disk

The spec lists `redb` for persistence but positions it as WAL/snapshot storage, not as the primary graph storage. This implies the full graph must fit in RAM, which is unrealistic at 20M nodes.

---

## 4. Boundary Violations

### BOUNDARY-1: "Unlimited concurrent readers" claim

Section 6 claims "Concurrent readers: Unlimited." This is physically false. Each MVCC snapshot consumes memory (either a copy of modified pages or a reference to a version chain position). Each active reader prevents garbage collection of versions it can see. With truly unlimited readers, memory consumption is unbounded.

In practice, PostgreSQL limits `max_connections` (default 100). Datomic's peer model can handle thousands of concurrent readers, but each peer caches a snapshot of the database in its process memory. "Unlimited" should be replaced with a realistic bound and its resource cost.

### BOUNDARY-2: Transaction abort + IVM rollback

If a transaction creates nodes and edges, triggering IVM updates to materialized views, and then the transaction aborts -- what happens to the IVM state? 

Options:
1. IVM updates are deferred until commit (simple, but reads during the transaction see stale views)
2. IVM updates are applied speculatively and rolled back on abort (complex, requires versioned view state)
3. IVM updates only reflect committed data (requires two-phase: first commit data, then update views -- but this creates a window where views are stale)

Each option has correctness implications for readers who are concurrently reading the materialized views. The spec doesn't address this.

### BOUNDARY-3: Sync of 100 version nodes at <10ms

The spec targets "Sync (100 version Nodes): <10ms." This budget must cover:
1. Serialize 100 version nodes + their edges (version chain links)
2. Network transfer (even localhost has overhead)
3. CRDT merge logic per node (HLC comparison, conflict detection)
4. Graph insertion of 100 nodes + edges
5. IVM update for all affected views
6. Capability checks on received data

At the Grafeo baseline of ~4 microseconds per createNode(), just inserting 100 nodes costs 400 microseconds. Adding edges (NEXT_VERSION, CURRENT updates) could double that. CRDT merge logic (HLC comparison per property per node) adds further. IVM updates for 100 new nodes could trigger many view recomputations.

The <10ms target might be achievable if IVM updates are batched and deferred, but the spec doesn't indicate this.

---

## 5. Architectural Feasibility Concerns

### ARCH-1: The spec contradicts its own research

The `explore-database-is-application.md` document explicitly recommended:

> "Do NOT adopt an external IVM service (Materialize, Feldera, RisingWave). They add operational complexity for a problem that can be solved in-process."

And:

> "The most powerful insight from this research is not any single paradigm but their intersection. [...] The database IS the application -- not because we replace PostgreSQL with something exotic, but because we use the graph we already have as the substrate for computation, not just storage."

The recommended approach was a graduated, in-process IVM applied to Thrum's existing PostgreSQL+AGE architecture. The specification instead proposes building a custom Rust database engine with a general-purpose IVM system -- exactly the "replace PostgreSQL with something exotic" path the research warned against.

The research recommended specific, bounded improvements:
1. `createMaterializedIndex<K, V>()` primitive
2. Dependency tracking in the materializer
3. Eager materialization on write
4. Graph-native event dispatch
5. Selective WebSocket reactivity
6. Local-first page builder via PGlite

Each of these is achievable in the current architecture within 1-2 months. The spec proposes replacing the architecture entirely, which is 12-24 months of work for a small team, with research-grade subsystems (IVM, CRDT sync, Cypher parser, MVCC) that each individually represent months of effort.

### ARCH-2: Scope vs. team size mismatch

The spec describes 10 Rust crates:
1. benten-core (types)
2. benten-graph (storage, indexes, traversal)
3. benten-ivm (incremental view maintenance)
4. benten-version (version chains)
5. benten-capability (UCAN enforcement)
6. benten-sync (CRDT merge)
7. benten-query (Cypher parser + query planner)
8. benten-persist (WAL + disk storage)
9. benten-reactive (subscriptions)
10. benten-engine (orchestrator)

Plus 2 binding layers (napi-rs, wasm-bindgen). This is roughly equivalent in scope to building a new database engine. For reference:

- **redb** (pure Rust embedded KV): 2 years of development by a small team, ~15K LOC
- **sled** (embedded DB): 5 years, abandoned, cited extreme complexity
- **Grafeo**: Built by a funded team, still v0.5 with known limitations
- **DuckDB**: 4+ years by a team of database researchers, backed by a foundation

Each of the IVM, CRDT sync, and Cypher parser crates is individually a 6-12 month effort for experienced systems programmers. Combined with the other crates, this is 2-4 years of work.

### ARCH-3: The "What Gets Replaced" table hides incremental migration impossibility

Section 3.2 shows a clean replacement table (in-memory registries -> materialized views, event bus -> reactive subscriptions, etc.). But the existing Thrum codebase has ~2,900+ tests and 15 packages built on the current architecture. 

The migration path is not specified. Can the engine be adopted incrementally (replace one subsystem at a time), or must it be all-or-nothing? If incremental, which subsystem goes first? How do the two systems coexist during migration?

The TypeScript layer description (Section 3.3) says "The TypeScript layer does NOT contain: Storage logic, Event dispatch, Capability checks, Version management, Sync logic." This implies a complete cutover, not an incremental migration. That means the entire Thrum codebase must be rewritten to use the new engine before any value is delivered.

---

## 6. Recommendations

### R1: Define IVM scope and algorithm before committing to build

Pick a specific IVM approach (counting-based for simple views, differential dataflow for complex ones) and specify it in the document. Prototype the IVM crate in isolation with benchmarks before building the rest of the engine. If IVM maintenance costs make write performance unacceptable, the entire architecture needs rethinking.

**Concrete next step:** Write 3-5 specific view definitions (event handlers by event name sorted by priority, capability grants for a module, content of type X sorted by createdAt DESC limited to 20) and implement them as standalone Rust code with benchmarks. Measure actual write amplification.

### R2: Replace petgraph with a disk-backed graph structure

Use redb (or a similar embedded KV) as the primary graph storage, not petgraph. Index nodes by ID (hash index), by label (B-tree on label -> node ID), and by property (B-tree on label+property -> node ID). Keep a small LRU cache for hot nodes (modules, event handlers, capabilities). This solves the 20M node memory problem.

### R3: Specify the concurrency model as one concrete design

Recommendation: MVCC with optimistic concurrency control. Writers create new versions. Readers see a consistent snapshot at transaction start time. Conflicts are detected at commit time (if two transactions modify the same node, the second to commit is aborted and retried). This is well-understood, well-tested (PostgreSQL SSI, Datomic), and avoids deadlocks.

### R4: Define CRDT conflict resolution for structural operations

At minimum, specify:
- Tombstone representation for deleted nodes
- Edge behavior when source or target is tombstoned
- Schema validation on sync receipt (reject structurally invalid data)
- Conflict resolution API for cases that cannot be auto-resolved
- Whether composition references have special CRDT semantics (e.g., "if target is deleted, mark reference as broken, not orphaned")

### R5: Add a phased delivery plan that delivers value incrementally

Phase 1: benten-core + benten-graph + benten-persist (basic graph storage with disk backing, <3 months)
Phase 2: benten-version (version chains, <2 months)
Phase 3: benten-query (Cypher subset: read-only MATCH with property filters, <3 months)
Phase 4: benten-ivm (materialized views for 3-5 specific patterns, <4 months)
Phase 5: napi-rs bindings + Thrum integration for ONE subsystem (e.g., event handler resolution)

This lets the project prove value at each phase. If Phase 4 reveals that IVM maintenance costs are too high, the project can pivot to a simpler caching approach without having wasted 2 years.

### R6: Revise performance targets to be evidence-based

Replace the <0.01ms write targets with ranges based on the component costs measured in spikes. Present targets as:
- Read (cached/materialized): <0.01ms (feasible, confirmed by Grafeo spike)
- Read (indexed lookup): <0.05ms (feasible, confirmed by PGlite spike)
- Write (single node, no IVM): <0.05ms (realistic for Rust native with WAL)
- Write (single node, with IVM): 0.05-0.5ms depending on view count (honest, needs measurement)
- Transaction (5 ops): 0.5-2ms (realistic given WAL + IVM)

### R7: Reconcile with the research's own recommendation

The `explore-database-is-application.md` document recommended a graduated approach within the existing architecture. The spec should explicitly address why that path was abandoned in favor of a custom engine, what new information changed the calculus, and what the risks are of the larger scope.

---

## Summary

The specification captures a compelling vision for a unified graph runtime. However, it suffers from:

1. **Performance targets that violate the architecture's own overhead** (BUG-1)
2. **An unresolved concurrency model** (BUG-2) that leaves the most critical design decision open
3. **CRDT semantics that create data corruption by design** (BUG-3)
4. **Zero algorithmic detail for the claimed key innovation (IVM)** (NULL-1)
5. **Unacknowledged memory constraints** that make the proposed scale targets infeasible with petgraph (NULL-4)
6. **A scope that contradicts the project's own research recommendations** (ARCH-1)
7. **Multi-year scope disguised as a specification** (ARCH-2)

The spec needs to be either dramatically scoped down (focus on graph storage + version chains + basic indexing, defer IVM and CRDT sync) or dramatically more detailed (specify IVM algorithm, MVCC implementation, GC strategy, Cypher subset, migration plan). In its current form, it is a vision document, not an implementable specification.
