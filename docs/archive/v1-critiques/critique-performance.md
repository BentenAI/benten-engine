# Performance & Scalability Critique: Benten Engine Specification

**Reviewer:** Performance & Scalability Agent
**Date:** 2026-04-11
**Scope:** SPECIFICATION.md performance targets, IVM overhead model, memory scaling, MVCC GC, petgraph viability, version chain compaction
**Input Data:** Specification, Grafeo load test results, Grafeo investigation results, PGlite+AGE spike results, 15 exploration documents

---

## Performance Score: 5/10

The specification describes a compelling architectural vision, but the performance targets are stated without justification, the IVM cost model is absent, the memory model for version chains at scale is unaddressed, and critical operational concerns (GC, compaction, write amplification) are not mentioned at all. The spike data actually *contradicts* several targets, and the spec does not acknowledge the gaps.

---

## Issue 1: IVM Write Amplification is Unquantified and Likely Dominates at Scale

**Severity: Critical**

The specification claims Node creation at <0.01ms (Section 6). This target includes "Write + IVM update." But the IVM update cost is unbounded and depends on two variables the spec never addresses: (a) how many views reference the affected data, and (b) the complexity of each view's incremental update rule.

**The math the spec avoids:**

Consider a content Node creation. This single write could affect:
- A "content listing by type" view (filter + sort + paginate)
- A "content count by type" view (aggregate)
- A "composition dependency" view (traversal)
- An "event handlers for content:afterCreate" view (already resolved, but the write triggers dispatch)
- A "capability check" view (if the write triggers capability re-evaluation)
- Any user-defined subscription views

If 10 views are affected and each incremental update takes 0.005ms, the total write cost is 0.06ms — already 6x over the <0.01ms target. At 50 views (realistic for a platform with modules, each defining their own views), write latency becomes 0.25ms.

**What production IVM systems actually report:**

Materialize and Feldera amortize IVM cost over micro-batches, not single writes. Feldera's DBSP model processes changes in batches to amortize the view dependency graph traversal cost. The specification assumes single-write IVM propagation with sub-0.01ms total latency, which is inconsistent with how every production IVM system works.

The `explore-database-is-application.md` document itself acknowledges this tradeoff: "Writes become slightly more expensive because they trigger incremental updates. This is the right tradeoff for a CMS -- reads vastly outnumber writes." But it never quantifies "slightly more expensive," and the specification's performance table erases this nuance entirely by claiming <0.01ms for writes.

**Recommendation:** Define the IVM cost model explicitly. State: (a) maximum view fan-out per write, (b) incremental update budget per view, (c) whether updates are synchronous or asynchronous (and if async, what consistency model reads see). Add a "write with N views" benchmark target. A realistic target for single-node-creation with 10 dependent views is 0.05-0.2ms, not <0.01ms.

---

## Issue 2: Memory Model for 20M Nodes with Version Chains is Untenable in petgraph

**Severity: Critical**

The specification lists `petgraph` as the graph data structure library (Section 4.2) and claims the engine handles version chains where "every mutation creates a version Node" (Section 2.3).

**The scaling arithmetic:**

Grafeo's spike data shows ~2KB/node RSS at scale (Large tier: 460K nodes, +896MB). petgraph's `Graph<N, E>` uses an adjacency list backed by `Vec<Node<N>>` and `Vec<Edge<E>>`. Each node entry is `size_of::<N>() + 2 * size_of::<NodeIndex>()` (next incoming/outgoing). Each edge is `size_of::<E>() + 4 * size_of::<NodeIndex>()` (source, target, next in/out).

For Thrum's Node type (`id: String, labels: Vec<Label>, properties: HashMap<String, Value>, version: u64`), a conservative estimate is 200-500 bytes per node in petgraph memory, depending on property count. Edges add ~100 bytes each.

Now consider a CMS with 10,000 content items, each edited 20 times on average:
- 10,000 anchor Nodes
- 200,000 version Nodes (snapshots, per Section 2.3)
- 10,000 CURRENT edges
- 200,000 NEXT_VERSION edges
- 200,000 HAS_VERSION edges
- Plus: platform Nodes (modules, event handlers, capabilities, field types, content types) ~5,000
- Plus: edges for relationships between content items, blocks, compositions

Total: ~215,000 Nodes, ~415,000 Edges. At 300 bytes/node and 100 bytes/edge: **~105MB** just for graph structure in memory.

Scale to a medium platform with 100,000 content items (20 versions each): **2.15M Nodes, 4.1M Edges**. At the same rates: **~1.05GB in memory**, all in petgraph's `Vec` allocations.

The specification's performance target of "unlimited concurrent readers" via MVCC means each reader holds a snapshot reference. If 50 concurrent readers each hold a snapshot and the graph is being mutated, the MVCC layer must keep old graph states alive until all readers complete. With petgraph's `Vec`-based storage, this requires either copy-on-write (duplicating the entire Vec on each write — catastrophic) or a separate MVCC layer with indirection (adding per-access overhead that invalidates the <0.01ms read targets).

**petgraph is fundamentally an in-memory, single-version data structure.** It has no MVCC, no snapshots, no concurrent read/write support. Adding these on top means you are no longer using petgraph — you are building a database around petgraph's adjacency list, and at that point petgraph's value proposition (simple, fast, well-tested) is buried under layers of wrapper infrastructure.

Research from 2025 confirms this: "existing frameworks, such as PetGraph, offer varying degrees of efficiency but often struggle with scalability and speed when dealing with massive datasets or frequent updates" (Performance Comparison of Graph Representations Which Support Dynamic Graph Updates, arXiv:2502.13862).

**Recommendation:** Decide early: is the graph memory-resident or disk-backed? If memory-resident, petgraph works for the platform metadata subgraph (~5K-50K nodes) but NOT for content + version chains at scale. If disk-backed, you need redb or similar underneath, and petgraph becomes an in-memory cache layer, not the storage layer. The specification conflates these two modes. Define the boundary explicitly: what lives in petgraph (hot), what lives in redb (warm), what lives on disk only (cold).

---

## Issue 3: No Garbage Collection Strategy for Version Chains

**Severity: High**

The specification states: "History IS the graph" (Section 2.3) and "Every versionable entity has Version Nodes linked by NEXT_VERSION edges." The `explore-graph-native-protocol.md` document specifies full snapshot Nodes (not deltas) for each version.

**The unbounded growth problem:**

A composition edited 500 times produces 500 Version Nodes, each containing a full snapshot of the composition's blocks array (potentially 50-200KB of JSONB per snapshot). That single entity's version chain: **25-100MB of version data.**

10,000 compositions with average 50 versions: **500,000 Version Nodes.** If each snapshot averages 10KB: **~5GB** of version data alone.

The specification never mentions:
- **Version chain compaction:** When do old versions get pruned? After 30 days? After 100 versions? Never?
- **Snapshot-to-delta conversion:** The `explore-graph-native-protocol.md` mentions "Deltas are a compression optimization applied later (cold storage, sync payloads)" but the specification has no mechanism for this.
- **Garbage collection of unreachable snapshots:** If MVCC keeps old versions alive for readers, what reclaims them when readers complete? CMU's research on MVCC GC identifies this as "the most important aspect of an MVCC DBMS" — without GC, memory grows monotonically.
- **The redb interaction:** If version Nodes are persisted to redb, redb's own copy-on-write B-tree creates ADDITIONAL versioning. You get double-versioning: the engine's version chains AND redb's page-level copy-on-write. redb reclaims pages only after all read transactions complete, so long-running reads hold open large swaths of disk.

**What PostgreSQL does that the spec ignores:** PostgreSQL's VACUUM process is specifically designed to solve this problem — it reclaims dead tuples from MVCC. It is also PostgreSQL's single most painful operational concern, causing table bloat, autovacuum storms, and production incidents at scale. The specification is building a system with the same fundamental problem and no mention of the equivalent mechanism.

**Recommendation:** Add a Section on "Version Chain Lifecycle" that covers: (a) retention policy (configurable per-entity-type), (b) compaction strategy (merge snapshots into deltas after N days), (c) MVCC snapshot GC (epoch-based reclamation, watermark advancement), (d) redb page reclamation interaction. This is not optional — it determines whether the system can run for more than a few months without running out of memory or disk.

---

## Issue 4: Performance Targets Contradicted by Spike Data

**Severity: High**

The specification's Section 6 performance targets do not cite the spike data that was conducted as input to the specification. In several cases, the targets contradict the measured results.

| Operation | Spec Target | Best Measured (Any Backend) | Gap |
|-----------|-------------|---------------------------|-----|
| Node lookup by ID | <0.01ms | Grafeo: 0.034ms p50 | 3.4x over |
| Materialized view read | <0.01ms | No IVM system was tested | Unvalidated |
| Event handler resolution | <0.01ms | Grafeo: 0.149ms p50, PGlite SQL: 0.243ms p50 | 15-24x over |
| Capability check | <0.01ms | Grafeo: 0.153ms p50, PGlite SQL: 0.162ms p50 | 15-16x over |
| Content listing (paginated) | <0.1ms | PGlite SQL: 0.461ms p50 at 1K items | 4.6x over |
| Node creation | <0.01ms | Grafeo: 0.004ms (240K ops/s) | Achievable (Grafeo only, no IVM) |
| Edge creation | <0.01ms | Not benchmarked independently | Unvalidated |
| Transaction (5 ops) | <0.1ms | Not benchmarked | Unvalidated |
| Version chain traversal | <0.1ms | PGlite Cypher: 0.719ms at 1K | 7.2x over |

**The implicit assumption:** The targets assume that a custom Rust implementation will be 10-30x faster than Grafeo (a Rust-based in-memory graph database) and 15-50x faster than PGlite (in-process PostgreSQL). This is not impossible for pure data structure operations, but becomes implausible once you add MVCC, IVM, capability checking, and persistence (WAL writes).

The only target that is clearly achievable is raw Node creation without IVM (Grafeo demonstrates 240K ops/s = 0.004ms/op). But the spec claims <0.01ms *including* IVM update, which has never been measured.

**The IVM read target is especially problematic:** <0.01ms for a materialized view read implies returning a pre-computed result from an in-memory lookup. This is technically feasible (a `DashMap::get()` on a key is ~20-50ns), but it assumes the materialized view result fits in a single hashmap entry. A paginated content listing view cannot be a single hashmap value — it must support cursor-based pagination over a sorted set, which is O(log n) for seek + O(k) for page retrieval, not O(1).

**Recommendation:** Tier the performance targets by what has been validated vs. aspirational. Separate "raw operation" targets (achievable) from "operation + IVM + MVCC + capability check + WAL" targets (unvalidated). Cite the spike data as the baseline and state explicitly how much faster the custom engine needs to be, and why that improvement is expected (e.g., "eliminate SQL parsing overhead," "eliminate network hop," "eliminate WASM overhead").

---

## Issue 5: MVCC Snapshot Isolation Without a Concurrency Cost Model

**Severity: High**

Section 2.7 claims: "True multi-threaded read/write, MVCC snapshots, unlimited concurrent readers, lock-free or fine-grained per-Node locking." Section 4.2 lists `dashmap` for concurrent HashMap.

**What this actually requires:**

MVCC for a graph database is substantially harder than for a key-value store or relational database:

1. **Node-level MVCC:** Each Node needs a version list (or similar structure). A read at snapshot T must see the latest version <= T. This means every `getNode()` call includes a version scan. With dashmap, each Node ID maps to a version chain. Scanning the version chain is O(versions) per access.

2. **Edge-level MVCC:** Edges must also be version-aware. When a write transaction adds an edge, concurrent readers at older snapshots must NOT see it. This means edge iteration (critical for graph traversal) must filter by snapshot timestamp on every edge. This makes every traversal step O(degree * versions) instead of O(degree).

3. **Phantom prevention:** A traversal query like "all EventHandlers with LISTENS_TO edges to EventType X" must not see handlers added by concurrent transactions. This requires predicate locking or gap locking on edge types, which is notoriously expensive.

4. **Snapshot memory cost:** Each active reader holds a snapshot. If 50 readers are active and the write rate is 100 writes/sec, then in the 100ms an average read takes, 10 new versions are created. Those 10 versions must be kept alive until the slowest reader finishes. Under sustained write load, this creates an ever-growing tail of retained versions — the same problem PostgreSQL's VACUUM exists to solve.

**dashmap is insufficient:** dashmap provides concurrent HashMap access but not MVCC. You would need to build an MVCC layer *on top of* dashmap, where each dashmap value is a version chain. At that point, dashmap's lock-free reads hit a version chain that requires its own synchronization (what if a writer is appending to the same chain?). The specification does not describe this layer.

**Recommendation:** Add a "Concurrency Architecture" section that specifies: (a) the MVCC implementation strategy (timestamp-ordered version chains, or copy-on-write, or epoch-based), (b) how edge traversal interacts with MVCC (filtering per edge vs. snapshot-consistent adjacency lists), (c) the snapshot lifecycle (when created, when released, what GC mechanism frees retained versions), (d) expected throughput under concurrent load with specific reader/writer ratios (citing the Grafeo spike's Pattern A/B/C as the baseline to beat).

---

## Issue 6: IVM Dependency Graph Scales Quadratically with Interacting Views

**Severity: Medium-High**

The specification describes views as query patterns (Section 2.2) and states that "when a write occurs, the engine identifies which views are affected." This identification step is itself a scalability concern.

**The dependency tracking problem:**

For each write (create/update/delete Node or Edge), the engine must:
1. Determine which labels/properties changed
2. Look up which views depend on those labels/properties
3. For each affected view, compute the incremental delta

Step 2 requires a "dependency index" — a mapping from `(label, property)` pairs to the set of views that reference them. If there are V views and each view depends on D label/property pairs on average, the dependency index has V * D entries.

But this is the simple case. Graph views often have *join dependencies*: "all Nodes with label X that have an edge of type Y to a Node with label Z where Z.prop = W." A write to a Node with label X must check if it participates in any view that involves label X. A write to an edge of type Y must check both endpoints. A write to a Node with label Z must check all views that traverse to Z.

This creates a dependency fan-out where a single write can trigger O(V) view lookups, and each lookup may involve traversals to verify whether the written data is actually in the view's scope.

At 100 views (realistic: each module defines content listing views, event handler views, capability views, statistics views), a single Node write triggers 100 dependency checks. At 500 views (a mature platform with many modules), this becomes the dominant cost of every write.

**Recommendation:** Specify the dependency tracking strategy. Options: (a) coarse-grained (label-level invalidation, fast but over-invalidates), (b) fine-grained (per-row tracking, precise but expensive), (c) hybrid (label-level plus bloom filters per view). State the expected view count at maturity and the per-write dependency resolution budget.

---

## Issue 7: Version Chain Traversal at <0.1ms is Unrealistic for Content with Rich Snapshots

**Severity: Medium**

The specification targets <0.1ms for version chain traversal (Section 6). The `explore-graph-native-protocol.md` specifies that each version Node contains a full snapshot including the blocks array.

**The data size problem:**

A composition with 20 blocks, each with props, has a snapshot size of roughly 5-20KB (measured from existing Thrum compositions). Traversing 20 versions of this entity means reading 100-400KB of data.

Even with sequential memory access at memory bandwidth (~50GB/s for L3 cache), reading 400KB takes ~8 microseconds for raw memory access. But version Nodes are not sequential in memory — they are individual Nodes in a graph structure, each potentially allocated separately. With pointer-chasing through 20 Nodes, each requiring a hash lookup (dashmap) or index dereference (petgraph), the realistic per-hop cost is 0.05-0.5 microseconds.

20 hops * 0.5us/hop = 10us = 0.01ms for traversal PLUS 400KB of data deserialization/access. If properties are stored as `HashMap<String, Value>`, each property access involves a hash computation and potential cache miss.

At 100 versions, this becomes 0.05ms for traversal + significant data access cost, making <0.1ms tight but potentially achievable for the traversal itself (not for returning all version data).

**The real concern is returning version data:** If `getHistory(nodeId)` returns all version Nodes with their snapshots, the response size is O(versions * snapshot_size). For 100 versions at 10KB each, that is 1MB of data to serialize and return over napi-rs. Serialization alone will exceed 0.1ms.

**Recommendation:** Distinguish between version chain metadata traversal (anchor -> version -> version -> ..., returning only version numbers and timestamps) and full version data retrieval (returning complete snapshots). The former can meet <0.1ms. The latter cannot, and should have a separate, higher target (e.g., <1ms for 20 full snapshots).

---

## Issue 8: redb as the Persistence Layer Creates a Write Amplification Cliff

**Severity: Medium**

Section 4.2 lists `redb` as the embedded persistent key-value storage. redb uses copy-on-write B-trees, meaning every write to a key requires copying the entire B-tree page containing that key, plus all parent pages up to the root.

**Write amplification math:**

redb's B-tree pages are typically 4KB (configurable). If a Node's serialized data is 300 bytes, it shares a page with ~13 other Nodes. Updating one Node rewrites the entire 4KB page + parent pages (typically 2-3 levels for millions of keys). Write amplification: ~12-16KB written to disk per Node update.

For a write burst (bulk import of 1,000 content items, each creating an anchor + version + 3 edges = 5 graph operations), the actual disk writes are:
- 5,000 key insertions * 12KB write amplification = ~60MB of disk I/O
- Plus WAL writes (sequential, so ~5,000 * 300 bytes = 1.5MB)

redb's benchmarks show bulk load at ~1770ms for comparable workloads (vs. lmdb at 976ms). For individual transactions, redb's fsync-per-commit model adds 1-10ms per transaction depending on the storage device.

**The specification's <0.01ms Node creation target is incompatible with durable persistence.** An fsync takes ~0.1-10ms on NVMe, ~1-50ms on SATA SSD. Even group commit (batching multiple writes into one fsync) only amortizes this — individual write latency is still bounded by fsync frequency.

**Recommendation:** Clarify whether the <0.01ms target is for in-memory writes (with async WAL flush) or durable writes. If in-memory with async durability, state the durability window (e.g., "up to 100ms of writes may be lost on crash"). If durable, the target must be revised to ~0.1-1ms per transaction. The Grafeo spike showed 138s to persist XL data (2.3M nodes) — the specification should state the expected persistence overhead.

---

## Issue 9: Cypher Parsing Overhead Contradicts Sub-0.01ms Targets

**Severity: Medium**

Section 4.1 includes `benten-query` for "Cypher parser + query planner." The API surface (Section 4.3) shows `engine.query(cypher: string, params?)` and `engine.createView(name: string, query: string)`.

The PGlite spike measured Cypher parsing + execution overhead:
- Simple node lookup via Cypher: 0.154ms (vs. 0.131ms for equivalent SQL)
- Event handler lookup via Cypher: 0.291ms (vs. 0.243ms for SQL)

The overhead is ~0.02-0.05ms for parsing alone (the difference between Cypher and direct index lookup). A custom Cypher parser in Rust can be faster than PGlite's AGE parser, but parsing is inherently string processing — regex/PEG parsing of even a simple `MATCH (n:Label {id: $1}) RETURN n` involves tokenization, AST construction, and query plan generation.

For hot-path operations (capability checks, event handler resolution), Cypher parsing on every call would violate the <0.01ms target. The specification should clarify that hot-path operations use the Rust-native API (direct function calls), not Cypher queries, with Cypher reserved for ad-hoc queries and view definitions.

**Recommendation:** Explicitly define two API tiers: (a) Rust-native typed API for hot-path operations (getNode, getEdges, readView — no parsing), (b) Cypher string API for view definitions and ad-hoc queries. State that materialized view reads bypass the query parser entirely.

---

## Issue 10: No Benchmark for IVM Under Concurrent Write Load

**Severity: Medium**

The Grafeo spike tested concurrent workloads (Pattern A: 50 readers/2 writers, Pattern B: 5 readers/20 writers) and found catastrophic degradation under write-heavy load (XL: 6 ops/s total for Pattern B). The specification claims IVM eliminates this problem because "reads are O(1) lookups into pre-computed results."

**The untested claim:**

IVM shifts work from read time to write time. Under write-heavy load, this means:
- Each write triggers multiple view updates
- View updates require write locks on the materialized view data structures
- Concurrent readers of the same materialized view may contend with the writer updating it

If the view update is synchronous (writer blocks until all affected views are updated), write throughput is bounded by the slowest view update. If async (writer returns immediately, views update in background), readers may see stale data — violating the "answers exist before questions" guarantee.

The specification's Pattern B scenario (20 writers) with IVM would mean 20 concurrent write streams, each triggering ~10 view updates, for 200 concurrent view update operations. Without careful design (e.g., read-copy-update, lock-free swap of materialized view pointers), this creates write contention on the view storage that could be worse than the Grafeo baseline.

**Recommendation:** Define a benchmark scenario: "20 concurrent writers creating content Nodes, 50 concurrent readers reading a content listing view. Measure: view staleness (ms between write commit and view update), write throughput (ops/s), read latency (p95)." This is the scenario that validates the entire IVM architecture.

---

## Scale Limits Summary

| Subsystem | Estimated Breaking Point | Bottleneck |
|-----------|------------------------|------------|
| petgraph in-memory graph | ~2-5M Nodes (1-3GB RSS) | Memory; Vec reallocation on growth |
| Version chains (snapshot model) | ~50K entities * 50 versions = 2.5M version Nodes | Memory + GC; no compaction strategy |
| IVM view count | ~100-200 views | Write amplification; O(V) dependency checks per write |
| IVM under write-heavy load | ~50 writes/sec with 50+ views | View update contention; sync vs async tradeoff |
| redb persistence | ~10M keys | Write amplification from CoW B-tree; fsync latency |
| MVCC snapshot retention | ~100 concurrent readers under write load | GC backpressure; version chain growth |
| Cypher query parsing | N/A for hot paths (must bypass) | String parsing overhead; ~0.02-0.05ms minimum |

---

## Optimization Recommendations (Prioritized by Impact)

### 1. Define the Hot/Warm/Cold Memory Architecture (Impact: Critical)

Separate the graph into tiers:
- **Hot (petgraph):** Platform metadata — modules, event handlers, capabilities, content type schemas. ~5K-50K Nodes. Fits in memory. No version chains needed (these are append-mostly configuration).
- **Warm (redb + in-memory cache):** Content entities — anchor Nodes + recent N versions. Loaded on demand, cached with LRU eviction.
- **Cold (redb only):** Old version Nodes. Read on explicit history request, never cached.

This avoids the "entire graph in memory" problem while preserving O(1) for hot-path reads.

### 2. Specify IVM as Async with Bounded Staleness (Impact: Critical)

Define IVM view updates as asynchronous with a configurable staleness budget (e.g., "views are consistent within 10ms of the triggering write"). This decouples write latency from view count and allows batching of view updates under write-heavy load. Readers get a "last consistent snapshot" pointer that updates atomically (read-copy-update pattern).

### 3. Add Version Chain Compaction and Retention Policy (Impact: High)

Define a background compaction process:
- After 30 days (configurable), convert full snapshots to deltas against the previous version
- After 90 days, merge multiple deltas into a single checkpoint + delta chain
- Provide a configurable retention limit (e.g., keep last 100 versions, compact everything older)
- For MVCC: use epoch-based reclamation where version Nodes are freed only when no active snapshot references them

### 4. Separate Rust-Native API from Cypher API (Impact: High)

Hot-path operations (getNode, readView, checkCapability) must be direct Rust function calls exposed via napi-rs, NOT Cypher string parsing. Cypher is for view definitions and ad-hoc queries. The TypeScript API should have typed methods for all hot-path operations.

### 5. Revise Performance Targets with Validated Baselines (Impact: Medium)

Replace the current aspirational targets with tiered, evidence-based targets:
- **Tier 1 (in-memory, no IVM, no persistence):** <0.01ms reads, <0.01ms writes
- **Tier 2 (in-memory with IVM, no persistence):** <0.01ms reads, <0.05ms writes (10 views)
- **Tier 3 (full stack: IVM + MVCC + WAL):** <0.01ms reads, <0.5ms durable writes
- Cite Grafeo/PGlite spikes as reference points and state the expected improvement factor with justification.

### 6. Design MVCC Around Epoch-Based Reclamation (Impact: Medium)

Use epoch-based memory reclamation (similar to crossbeam-epoch in Rust) rather than per-snapshot reference counting. This gives O(1) amortized GC cost and avoids the "long-running reader holds all versions alive" problem by advancing the global epoch when the oldest reader completes.

---

## Sources

- [petgraph Documentation](https://docs.rs/petgraph/)
- [Performance Comparison of Graph Representations Which Support Dynamic Graph Updates (2025)](https://arxiv.org/html/2502.13862v1)
- [petgraph Internals — Timothy Hobbs](https://timothy.hobbs.cz/rust-play/petgraph-internals.html)
- [Incremental View Maintenance for Property Graph Queries](https://www.researchgate.net/publication/325374100_Incremental_View_Maintenance_for_Property_Graph_Queries)
- [Enzyme: Incremental View Maintenance for Data Engineering](https://arxiv.org/html/2603.27775)
- [Incremental Materialized Views: The Complete Guide (2026) — RisingWave](https://risingwave.com/blog/incremental-materialized-views-complete-guide/)
- [Scalable Garbage Collection for In-Memory MVCC Systems](https://www.semanticscholar.org/paper/Scalable-Garbage-Collection-for-In-Memory-MVCC-B%C3%B6ttcher-Leis/5e27c111391c3585896c111660734497f2335bb1)
- [CMU 15-721: MVCC Garbage Collection Lecture](https://15721.courses.cs.cmu.edu/spring2020/notes/05-mvcc3.pdf)
- [redb Design Document](https://github.com/cberner/redb/blob/master/docs/design.md)
- [redb DeepWiki](https://deepwiki.com/cberner/redb)
- [DashMap — Concurrent HashMap for Rust](https://github.com/xacrimon/dashmap)
- [SurrealKV — Versioned Embedded KV with MVCC](https://github.com/surrealdb/surrealkv)
- [Materialize IVM Engine](https://materialize.com/blog/ivm-database-replica/)
