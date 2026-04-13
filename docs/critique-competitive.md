# Benten Engine -- Competitive Analysis Critique

**Date:** 2026-04-11
**Reviewer:** Competitive Analysis Agent
**Document reviewed:** `docs/SPECIFICATION.md`
**Score: 4/10**

---

## Executive Summary

The Benten Engine specification proposes building a custom graph execution engine that unifies data storage, computation, reactivity (IVM), CRDT sync, and capability enforcement into a single Rust binary. After evaluating it against seven competitors in their 2026 state, the assessment is that **the specification significantly underestimates how much the competitive landscape has shifted since its research was conducted**, contains **factual errors about competitor capabilities**, and proposes building from scratch when a composition strategy using existing crates would deliver faster and at lower risk.

The core *vision* (data-sovereign, embeddable, P2P-syncable graph runtime) is genuinely differentiated. But the proposed *execution path* (10-crate custom engine) is not the only or best way to get there, and the specification does not adequately justify why existing tools cannot be composed to reach 80%+ of the goal.

---

## Competitor Landscape (2026 State)

### 1. Grafeo (Embedded Graph, Rust)

**Spec's claim:** "No sorted indexes, no MVCC, single-writer lock"
**2026 reality:** Grafeo has evolved dramatically since the spec's research. As of March 2026:
- Full ACID compliance with **MVCC-based snapshot isolation** (the spec says "no MVCC" -- this is wrong)
- Supports **GQL, Cypher, Gremlin, GraphQL, SPARQL, and SQL/PGQ** (6 query languages)
- **napi-rs bindings** (Node.js/TypeScript) and **wasm-bindgen** (WASM) -- exactly what Benten proposes to build
- CDC (Change Data Capture) with before/after snapshots -- a form of reactive notifications
- HNSW vector search, BM25 full-text search
- Fastest on LDBC Social Network Benchmark in both embedded and server modes
- Multi-language bindings: Python, Go, C, C#, Dart
- Standalone server mode with REST API and web UI
- AI features (vector/text/hybrid search), graph algorithms, parallel execution

**What Grafeo still lacks:** No IVM. No CRDT sync. No version chains. No capability enforcement. No P2P. But it covers the graph storage + embedded + MVCC + Cypher + WASM + napi-rs surface area that represents roughly 40% of the Benten spec's effort.

**Critical finding:** The spec's dismissal of Grafeo is based on outdated information. The "no MVCC, single-writer lock" claim appears to be stale. The specification needs to be updated against Grafeo's current state before committing to building benten-graph and benten-persist from scratch.

### 2. SurrealDB 3.0 (Multi-Model, Rust)

**2026 reality:** SurrealDB 3.0 reached GA on 2026-02-17 with $23M Series A extension. 2.3M downloads, 31K GitHub stars.
- Multi-model: document, graph, relational, time-series, geospatial, vector, key-value -- all in one
- Embedded mode (in-process, WASM, edge) -- same target as Benten
- LIVE queries for real-time subscriptions (reactive notifications)
- Row-level, field-level access control with DEFINE ACCESS (not UCAN, but fine-grained)
- Record-based authentication with multi-tenant scoping
- MCP integration for AI agent memory
- Custom API endpoints definable within the database itself
- BSL 1.1 license (converts to Apache 2.0 after 4 years per release; v3.0 becomes Apache 2.0 on 2030-01-01)

**What SurrealDB lacks:** No CRDT sync/P2P. No version chains. No IVM. No UCAN capabilities. License is BSL (not true open source until 2030 for v3.0).

**Critical finding:** SurrealDB's BSL license is a real blocker for Benten's "decentralized web" vision where anyone forks and self-hosts. But SurrealDB demonstrates that a single Rust binary can embed multi-model data, real-time subscriptions, fine-grained access control, and WASM deployment. Benten needs to articulate why its approach is better than "SurrealDB + CRDT sync layer + version chain layer."

### 3. CozoDB (Datalog + Graph, Rust)

**Spec's claim:** "Abandoned since Nov 2024"
**2026 reality:** Not abandoned. Last commits December 2024, issues opened February 2025. v0.7 released with MinHash-LSH, full-text search, JSON value support. WASM bindings available.
- Datalog query language (powerful for graph pattern matching and recursive queries)
- Embeddable like SQLite
- Multiple storage backends (in-memory, SQLite, RocksDB)
- Time-travel capability (historical queries)
- Vector search (HNSW)
- Rust + Python + Node.js + Go bindings

**What CozoDB lacks:** Small community. Slow release cadence. No CRDT sync. No reactive subscriptions. No capability enforcement. Limited indexing options.

**Critical finding:** The "abandoned" claim is inaccurate but the concern about velocity is fair. CozoDB's Datalog engine is interesting as a reference for IVM (Datalog is the theoretical foundation of incremental view maintenance). The spec proposes using `crepe` or `datafrog` for Datalog evaluation -- studying CozoDB's approach would be valuable before reinventing.

### 4. TerminusDB v12 (Graph + Version Control)

**2026 reality:** Now maintained by DFRNT (maintenance transferred in 2025). Version 12 with Rust storage backend.
- **Git-like version control** natively: branch, merge, diff, push, pull, clone, time-travel
- Delta encoding (append-only, succinct data structures)
- CRDT integration for conflict resolution
- GraphQL, WOQL (Datalog), REST API
- Schema constraints for data quality
- Apache 2.0 license

**What TerminusDB lacks:** Not embeddable as a library (server-only). No WASM. Limited adoption. DFRNT maintenance raises sustainability questions. Performance not competitive with Grafeo/SurrealDB on benchmarks.

**Critical finding:** TerminusDB already implements the version control + CRDT conflict resolution model that Benten proposes for `benten-version` and `benten-sync`. The spec does not reference TerminusDB at all -- this is a significant oversight. TerminusDB's delta encoding and branch/merge semantics are directly relevant prior art that should be studied, even if the implementation is not reusable (Prolog-based core, server-only).

### 5. Neo4j (Industry Standard Graph)

**2026 reality:** $200M+ ARR. Infinigraph distributed architecture. 100TB+ scale.
- Cypher query language (the standard Benten also targets)
- ACID, MVCC, multi-database
- Vector embeddings (billions of vectors)
- Change Data Capture (CDC)
- Multi-model expansion (documents, full-text, temporal)

**What Neo4j lacks:** Not embeddable. Not WASM-compatible. No CRDT sync. No version chains. No capability enforcement at data layer. JVM-based (Java). Commercial license for enterprise features.

**Critical finding:** Neo4j is not a direct competitor -- it targets enterprise graph analytics, not embeddable data sovereignty. But its Cypher implementation is the reference. Benten's `benten-query` Cypher parser must be compatible with Neo4j's Cypher semantics or risk confusing developers. The openCypher standard and GQL (ISO standard) are more relevant targets than a proprietary Cypher dialect.

### 6. Dgraph v25 (Distributed Graph)

**2026 reality:** Acquired by Istari Digital (Oct 2025, after Hypermode). Production-ready. Go-based.
- Distributed, horizontally scalable
- GraphQL native (not Cypher)
- Cluster-wide ACID transactions
- Synchronous replication
- Terabyte-scale

**What Dgraph lacks:** Not embeddable. No WASM. No CRDT sync. No version chains. Corporate instability (two acquisitions in 2 years). Go, not Rust.

**Critical finding:** Dgraph demonstrates that graph databases can scale horizontally, but its instability (two acquisitions) and GraphQL-only query language make it irrelevant to Benten's architecture. No further analysis needed.

### 7. TigerGraph (Analytics Graph)

**2026 reality:** Cloud-native (Savanna platform). Enterprise-focused. Proprietary GSQL language.
- Native parallel graph (NPG) architecture
- Real-time analytics at scale
- Preconfigured kits (fraud detection, customer insights)
- Snowflake/Iceberg connectors

**What TigerGraph lacks:** Not embeddable. Not open source. No CRDT. No WASM. Enterprise SaaS model antithetical to Benten's vision.

**Critical finding:** TigerGraph is irrelevant to Benten's use case. It targets enterprise analytics, not embeddable data-sovereign applications. Including it in the competitor list is padding.

### Bonus: NextGraph (Decentralized, CRDT, Graph)

**Not in the spec's competitor list but should be.** NextGraph is the closest conceptual competitor to Benten's vision:
- Decentralized, local-first, E2E encrypted
- CRDT sync (Automerge, Yjs, and custom Graph CRDT for RDF)
- Reactive ORM with TypeScript SDK (signals-based, framework-agnostic)
- Capability-based access control (cryptographic)
- Graph database (RDF/SPARQL)
- Funded by European Commission (ELFA consortium)
- FOSDEM 2026 presentations in both Local-First and Decentralized Internet devrooms

**What NextGraph lacks:** RDF-based (not LPG). SPARQL (not Cypher). No IVM. Smaller community. Early stage.

**Critical finding:** NextGraph shares ~80% of Benten Engine's *vision* (data sovereignty, CRDT sync, capability enforcement, graph as runtime, P2P). The spec references NextGraph in the research list but does not position against it. This is a glaring competitive gap. If a potential user discovers both projects, they will ask: "Why not contribute to NextGraph instead of building a parallel system?"

---

## Issue #1: The Specification's Competitor Table Is Factually Outdated (Severity: HIGH)

The spec's Table 1.1 states:

| Database | What's Missing |
|----------|----------------|
| Grafeo | No sorted indexes, no MVCC, single-writer lock |

This was likely accurate in late 2024 / early 2025. As of March 2026, Grafeo has MVCC-based snapshot isolation, is the fastest on LDBC benchmarks, has napi-rs and WASM bindings, and supports 6 query languages. The "single-writer lock" claim appears to be outdated based on multiple sources describing concurrent access.

**Impact:** If the decision to build a custom engine was partly justified by "Grafeo can't do X," and Grafeo now can do X, the justification weakens. The spec needs a fresh evaluation of Grafeo v0.5.x before proceeding.

**Recommendation:** Run Grafeo's current LDBC benchmarks in the Benten test environment. Verify sorted index support, MVCC behavior, and concurrent write performance. Update Table 1.1 with dated evidence.

---

## Issue #2: The Build-vs-Compose Decision Is Not Justified (Severity: CRITICAL)

The spec proposes 10 Rust crates built from scratch. But the spec never answers: **Why can't we compose Grafeo + yrs/automerge + a capability layer + an IVM layer?**

A composition approach:

| Benten Crate | Could Use Instead |
|--------------|-------------------|
| benten-core | Grafeo's node/edge types (or define thin types wrapping Grafeo) |
| benten-graph | Grafeo (embeddable, Rust, MVCC, Cypher, WASM, napi-rs) |
| benten-persist | Grafeo's persistence layer (redb or RocksDB) |
| benten-query | Grafeo's Cypher parser (already implements openCypher) |
| benten-version | Custom (no existing solution fits -- this IS unique) |
| benten-capability | Custom (UCAN + graph = unique) |
| benten-sync | yrs or automerge-rs (proven CRDT libraries) |
| benten-ivm | Custom (no existing embeddable IVM for graphs -- this IS unique) |
| benten-reactive | Grafeo CDC + custom subscription manager |
| benten-engine | Custom orchestrator |

This reduces the custom build from 10 crates to ~4-5 crates (version chains, capabilities, IVM, reactive, orchestrator) while getting graph storage, persistence, query parsing, and multi-language bindings for free.

**Impact:** The difference is likely 12-18 months of development vs 6-9 months. For a pre-revenue project, this is existential.

**Recommendation:** Before writing any Rust code, build a proof-of-concept that embeds Grafeo as a library, adds version chain nodes on top, and measures whether the performance targets in Section 6 are achievable. If yes, the composition path is strictly dominant.

---

## Issue #3: IVM Is Claimed As "The Key Innovation" But Has No Design (Severity: HIGH)

Section 2.2 states: "This is the key innovation." But the specification provides:
- Zero details on IVM algorithms
- Zero details on which query patterns can be incrementally maintained
- Zero details on the cost model (memory overhead of materialized views, update propagation cost)
- Zero analysis of existing IVM implementations (Materialize, RisingWave, DBToaster, Differential Dataflow)

IVM for graph databases is an active research area. MV4PG (2024) demonstrated materialized views on property graphs for Neo4j and TuGraph. Differential Dataflow (the engine behind Materialize) can maintain incrementally-updated views over changing datasets. The spec lists `crepe` or `datafrog` as dependencies but doesn't explain how Datalog evaluation connects to the Cypher query patterns Thrum actually uses.

Key questions unaddressed:
1. Which of Thrum's ~20 query patterns are IVM-eligible? (Not all queries can be incrementally maintained efficiently.)
2. What is the memory overhead? If every "view" doubles the data footprint, a device with 2GB RAM can store half as much data.
3. What happens when a write affects 100 materialized views? Is the write latency acceptable?
4. How does IVM interact with CRDT sync? (A remote merge could invalidate hundreds of views simultaneously.)

**Impact:** IVM is the riskiest component. If it doesn't work for Thrum's query patterns, the entire "answers exist before questions" thesis collapses, and the engine reverts to being "Grafeo but slower because we built it ourselves."

**Recommendation:** Before any implementation, produce a design document that: (a) enumerates every materialized view Thrum would need, (b) specifies the IVM algorithm for each, (c) models memory overhead, and (d) benchmarks a prototype against Grafeo with manual caching.

---

## Issue #4: NextGraph Exists and Shares 80% of the Vision (Severity: HIGH)

NextGraph is a decentralized, local-first, E2E encrypted platform with:
- CRDT sync (Automerge + Yjs + custom Graph CRDT)
- Reactive ORM (TypeScript SDK, signals-based)
- Capability-based access control (cryptographic, similar to UCAN)
- Graph database (RDF)
- P2P sync
- European Commission funding (ELFA consortium for a full collaboration suite)

The spec does not position Benten against NextGraph. A potential contributor or user will ask: "Why build Benten Engine when NextGraph already exists and has EU institutional funding?"

The honest answer is: NextGraph uses RDF/SPARQL (not LPG/Cypher), which doesn't align with Thrum's existing graph model. NextGraph's TypeScript ORM is tied to RDF semantics. And NextGraph's community is small. But these are legitimate differentiation points that need to be stated explicitly.

**Recommendation:** Add a "Why Not NextGraph?" section to the spec that honestly addresses: (a) RDF vs LPG data model mismatch, (b) SPARQL vs Cypher query language mismatch, (c) whether contributing to NextGraph with an LPG adapter would be faster than building from scratch, (d) NextGraph's sustainability model (EU grants vs organic community).

---

## Issue #5: The "Database IS the Application Runtime" Claim Is Unsubstantiated (Severity: MEDIUM)

Section 1.2 states: "It is not a database that an application queries. It IS the application's runtime -- data and computation are unified."

But Section 5 ("What the Engine Does NOT Do") lists: rendering, HTTP serving, code sandboxing, UI, module discovery, P2P networking. So the engine does NOT handle computation -- it handles data storage with reactive notifications. That's a very capable database with built-in pub/sub, not a "runtime."

The distinction matters because:
1. If it's a runtime, you'd expect user-defined functions, triggers, stored procedures, or code execution in the data plane. The spec has none of these.
2. If it's a database with reactive notifications, that's exactly what SurrealDB LIVE queries + Grafeo CDC already provide (minus CRDT and version chains).
3. The "data and computation are unified" claim currently means "writes trigger IVM updates and reactive notifications." That's event-driven architecture, not unification of data and computation.

**Impact:** Overstating the engine's scope creates expectations that won't be met. It also obscures the actual differentiation (IVM + version chains + CRDT sync + capabilities in a single embeddable binary).

**Recommendation:** Replace "application runtime" language with precise claims: "an embeddable graph database with built-in incremental view maintenance, version control, CRDT sync, and capability enforcement." Save "runtime" for when the engine actually executes user-defined logic (which may come via @sebastianwessel/quickjs integration, but isn't part of this spec).

---

## Issue #6: Performance Targets Lack Basis (Severity: MEDIUM)

Section 6 claims:
- Node lookup: <0.01ms (10 microseconds)
- Materialized view read: <0.01ms
- Node creation + IVM update: <0.01ms

These are aspirational but have no justification. Grafeo achieves <0.1ms for graph operations and is the LDBC benchmark leader. Benten claims 10x faster on reads and equivalent on writes -- without explaining how.

The 10 microsecond target for "materialized view read" is plausible (it's a HashMap lookup) but only if:
- The view fits in memory
- No serialization/deserialization is needed for the napi-rs bridge
- The MVCC snapshot doesn't add overhead

The <0.01ms for "Node creation + IVM update" is suspect. A single node creation that triggers IVM updates across multiple views will involve:
1. Write the node (allocate, index)
2. Identify affected views (scan view definitions)
3. Incrementally update each affected view
4. Persist the write (WAL append)
5. Notify subscribers

Doing all of that in 10 microseconds is extremely aggressive, especially if multiple views are affected.

**Recommendation:** Replace fixed targets with tiered targets: (a) "warm read" (view already computed, in-memory): <0.01ms, (b) "write with 1 affected view": <0.05ms, (c) "write with 10 affected views": <0.5ms, (d) "write with IVM + sync propagation": <5ms. Benchmark Grafeo at the same operations for a realistic baseline.

---

## Issue #7: The Cypher Parser Is a Multi-Year Project Disguised as One Crate (Severity: MEDIUM)

The spec lists `benten-query` as: "Cypher parser + query planner." Cypher (and now GQL/ISO) is a complex query language with:
- Pattern matching (variable-length paths, optional matches)
- Aggregation (GROUP BY, COLLECT, UNWIND)
- Subqueries
- MERGE semantics
- Query optimization (cost-based planning)
- WHERE clause expression evaluation

Neo4j's Cypher implementation has been developed over 15+ years. Grafeo's Cypher parser is built on the openCypher reference parser. Writing a Cypher parser + query planner from scratch is a 6-12 month effort for a small team, and getting it to parity with openCypher is a multi-year effort.

The spec already lists Grafeo as a known entity and Grafeo supports Cypher. Using Grafeo's query layer (or the openCypher reference parser in Rust) would save an enormous amount of work.

**Recommendation:** Do not build a Cypher parser from scratch. Either: (a) embed Grafeo and use its query layer, (b) use/fork the openCypher Rust parser (`opencypher-rs` or similar), or (c) start with a minimal query DSL and add Cypher compatibility incrementally.

---

## Benten's Genuine Unique Advantages

Despite the issues above, the Benten Engine specification proposes a combination that no single competitor offers:

| Feature | Grafeo | SurrealDB | CozoDB | TerminusDB | Neo4j | NextGraph | **Benten** |
|---------|--------|-----------|--------|------------|-------|-----------|------------|
| Embeddable library | Yes | Yes | Yes | No | No | Yes | **Yes** |
| WASM | Yes | Yes | Partial | No | No | Yes | **Yes** |
| LPG + Cypher | Yes | No (SurrealQL) | No (Datalog) | No (GraphQL/WOQL) | Yes | No (RDF/SPARQL) | **Yes** |
| MVCC concurrency | Yes | Yes | Limited | No | Yes | N/A | **Yes** |
| Version chains (native) | No | No | Time-travel | Git-like | No | Commit DAG | **Yes** |
| IVM (materialized views) | No | No | No | No | No | No | **Yes** |
| CRDT sync | No | No | No | Partial | No | Yes (Automerge/Yjs) | **Yes** |
| Capability enforcement | No | Row-level ACL | No | No | RBAC | Cryptographic | **UCAN** |
| Reactive subscriptions | CDC | LIVE queries | No | No | CDC | Reactive ORM | **Yes** |
| P2P sync | No | No | No | Push/Pull | No | Yes | **Yes** |
| Open source | Apache 2.0 | BSL 1.1 | MPL 2.0 | Apache 2.0 | Commercial | AGPL 3.0 | **TBD** |

**The genuine differentiation is the combination:** LPG graph + IVM + version chains + CRDT sync + UCAN capabilities + embeddable + WASM. No single product offers all of these together.

However, none of the individual features is novel. The value proposition is integration, not invention.

---

## Strategic Recommendations

### 1. Re-evaluate Grafeo as the Storage Foundation (Priority: IMMEDIATE)
Run Grafeo v0.5.x benchmarks. If it meets Thrum's graph storage needs (MVCC, Cypher, napi-rs, WASM, persistence), embed it as the storage layer and build IVM, version chains, capabilities, and sync on top. This cuts the build from 10 crates to 4-5.

### 2. Prototype IVM Before Committing (Priority: IMMEDIATE)
Build a standalone IVM prototype using `differential-dataflow` or `datafrog` over a small Grafeo instance. Validate that IVM can maintain Thrum's actual query patterns incrementally. If IVM proves infeasible for graph patterns, the entire engine thesis needs revision.

### 3. Position Against NextGraph Explicitly (Priority: HIGH)
NextGraph is the closest conceptual competitor. The spec needs a clear "Why Not NextGraph?" section that addresses the RDF/LPG mismatch, the SPARQL/Cypher mismatch, and whether contributing to NextGraph would be more efficient than building independently.

### 4. Update Competitor Analysis With Dated Evidence (Priority: HIGH)
Every claim about a competitor should include the version number and date checked. "Grafeo has no MVCC" was true in 2024 and false in 2026. Specifications that justify billion-dollar-effort decisions based on outdated competitor assessments are dangerous.

### 5. Use Existing CRDT Libraries, Do Not Build Custom (Priority: MEDIUM)
The spec correctly identifies `yrs` (Yrs/Yjs Rust port) and `automerge` as dependencies. This is the right approach. Extend it: use existing Cypher parsers, existing persistence engines, existing reactive primitives where possible. Minimize the custom surface area to what is genuinely unique (IVM, version chains, capability enforcement, orchestration).

### 6. Choose a License Before Building (Priority: MEDIUM)
The spec doesn't specify a license. For a "decentralized web" project, this matters enormously. Apache 2.0 enables maximum adoption but allows proprietary forks. AGPL 3.0 (like NextGraph) ensures modifications stay open but may deter corporate contributors. BSL (like SurrealDB) is antithetical to the stated vision. This decision affects competitive positioning and must be made before the first line of code.

---

## Market Positioning

**Current positioning (implied by spec):** "A custom graph execution engine for our platform."
**Problem:** This positions Benten as infrastructure for Thrum, not as a standalone product. It limits the addressable community to Thrum users.

**Recommended positioning:** "The embeddable graph engine for data-sovereign applications. Grafeo for storage, IVM for speed, CRDTs for sync, UCANs for trust. One binary, every platform."

This positions Benten as:
1. Building on Grafeo (credibility, not reinventing)
2. Differentiated by IVM + sync + capabilities (genuine novelty)
3. Useful beyond Thrum (broader community)
4. "One binary" (clear deployment model)

---

## Summary of Issues

| # | Issue | Severity | Action |
|---|-------|----------|--------|
| 1 | Competitor table has factual errors (Grafeo MVCC, CozoDB "abandoned") | HIGH | Re-research with dated evidence |
| 2 | Build-vs-compose decision not justified; Grafeo covers 40% of scope | CRITICAL | PoC with Grafeo as storage foundation |
| 3 | IVM claimed as key innovation but has no design or feasibility analysis | HIGH | IVM design doc + prototype before implementation |
| 4 | NextGraph shares 80% of vision and is not addressed | HIGH | Add explicit "Why Not NextGraph?" positioning |
| 5 | "Application runtime" claim is overstated | MEDIUM | Use precise language about actual capabilities |
| 6 | Performance targets lack justification or baselines | MEDIUM | Tiered targets + Grafeo baseline benchmarks |
| 7 | Cypher parser is a multi-year project in one crate | MEDIUM | Use existing parser (Grafeo or openCypher) |

---

## Sources

- [SurrealDB 3.0 GA](https://surrealdb.com/)
- [SurrealDB $23M raise - SiliconANGLE](https://siliconangle.com/2026/02/17/surrealdb-raises-23m-expand-ai-native-multi-model-database/)
- [SurrealDB 3.0 Features - TechnicalBeep](https://technicalbeep.com/multi-model-database-surrealdb-3-0/)
- [SurrealDB BSL License](https://surrealdb.com/license)
- [SurrealDB for AI Agents - The New Stack](https://thenewstack.io/surrealdb-3-ai-agents/)
- [Grafeo GitHub Repository](https://github.com/GrafeoDB/grafeo)
- [Grafeo Homepage](https://grafeo.dev/)
- [Grafeo HN Discussion](https://news.ycombinator.com/item?id=47467567)
- [Grafeo - JavaScriptDoctor Analysis](https://www.javascriptdoctor.blog/2026/03/grafeo-rust-powered-graph-database.html)
- [Grafeo - DEV Community](https://dev.to/alanwest/grafeo-an-embeddable-graph-database-in-rust-that-actually-makes-sense-1nik)
- [Neo4j Infinigraph - SiliconANGLE](https://siliconangle.com/2025/09/04/neo4j-unifies-real-time-transactions-graph-analytics-scale/)
- [Neo4j Trends - Calmops](https://calmops.com/database/neo4j/neo4j-trends/)
- [CozoDB GitHub](https://github.com/cozodb/cozo)
- [CozoDB Releases](https://github.com/cozodb/cozo/releases)
- [TerminusDB GitHub](https://github.com/terminusdb/terminusdb)
- [TerminusDB Homepage](https://terminusdb.com/)
- [Dgraph v25 - GitHub](https://github.com/dgraph-io/dgraph)
- [Dgraph Alternatives 2026 - PuppyGraph](https://www.puppygraph.com/blog/dgraph-alternatives)
- [TigerGraph Homepage](https://www.tigergraph.com/)
- [TigerGraph Savanna - TechTarget](https://www.techtarget.com/searchdatamanagement/news/366618412/TigerGraph-launches-Savanna-to-aid-AI-development)
- [NextGraph Homepage](https://nextgraph.org/)
- [NextGraph FOSDEM 2026 - Sync Engine](https://fosdem.org/2026/schedule/event/J3ZBYC-nextgraph-sync-engine-sdk-reactive-orm/)
- [NextGraph FOSDEM 2026 - E2EE Platform](https://fosdem.org/2026/schedule/event/CRSTQ8-nextgraph/)
- [NextGraph CRDT Documentation](https://docs.nextgraph.org/en/framework/crdts/)
- [IVM Complete Guide 2026 - RisingWave](https://risingwave.com/blog/incremental-materialized-views-complete-guide/)
- [MV4PG: Materialized Views for Property Graphs](https://arxiv.org/html/2411.18847v1)
- [Best P2P/CRDT Databases 2026](https://genosdb.com/popular-p2p-distributed-databases)
- [Best Graph Databases 2026 - Galaxy](https://www.getgalaxy.io/articles/best-graph-databases-2026)
