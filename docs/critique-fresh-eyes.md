# Fresh Eyes Critique: Benten Engine Specification

**Author:** Devil's Advocate / Senior Architect (no attachment to this codebase)
**Date:** 2026-04-11
**Contrarian Score: 8/10** -- I would change the fundamental approach.

---

## The Honest Assessment

This specification describes building a custom database engine in Rust with: graph storage, hash and B-tree indexes, MVCC, IVM, version chains, CRDT sync, a Cypher parser, reactive subscriptions, WAL persistence, capability enforcement, WASM bindings, and napi-rs bindings.

That is not a 4-8 week project. That is not a 4-8 month project. That is a multi-year effort, and pretending otherwise is the single most dangerous decision this project could make.

Let me walk through each question with brutal honesty.

---

## 1. Should You Build a Database Engine at All?

**No. Not unless you have already exhausted every alternative and can demonstrate that no combination of existing tools can achieve your goals.**

The specification itself lists the reasons to build:

> No existing tool combines: graph data model + reactive incremental view maintenance + CRDT sync + capability enforcement + embeddable + WASM + true concurrent read/write.

This is technically true. No single tool does all of this. But this is a classic trap: defining your requirements as the exact intersection of features that no existing product has, then using that gap as justification to build from scratch.

The question is not "does a tool exist that does all 10 things?" The question is: **"Which of these 10 things actually matter for shipping a product, and which are aspirational?"**

Let me score each requirement by near-term necessity:

| Requirement | Actually Needed Now? | Justification |
|-------------|---------------------|---------------|
| Graph data model | Yes | Core to the platform |
| Reactive IVM | No | Can be implemented in TypeScript on top of any DB |
| CRDT sync | No | P2P is a future vision, not a near-term shipping requirement |
| Capability enforcement | No | The existing RBAC system works. Capabilities are a better model but not blocking anything. |
| Embeddable | Maybe | For edge/browser, yes. But PostgreSQL works fine for server. |
| WASM | No | Not needed until you actually ship a browser-embedded version |
| Concurrent R/W | Yes | But PostgreSQL already provides this |
| Version chains | No | The existing revision system works |
| Cypher parser | No | Apache AGE already provides Cypher |
| WAL persistence | No | PostgreSQL already handles this |

Out of 10 requirements, 2 are genuinely needed now. The rest are either aspirational, already solved by the existing stack, or can be built incrementally on top of existing tools.

### The Sled Cautionary Tale

The sled embedded database in Rust has been in development since 2016 -- a decade -- and still has not reached 1.0. Its storage subsystem is being completely rewritten. Memory layout is being completely rewritten. It is a key-value store, dramatically simpler than what the Benten Engine specification describes.

Building a database engine is not a project. It IS the product. If you start this, you are no longer building a universal composable platform. You are building a database.

---

## 2. Is the Scope Realistic?

**No. The scope is approximately 10x larger than the spec implies.**

Let me enumerate the actual engineering work:

### Crate: benten-core (Node, Edge, Value types)
- Estimated effort: 1-2 weeks
- Complexity: Low
- This is the easy part. Type definitions, serialization, basic validation.

### Crate: benten-graph (Graph storage, indexes, traversal)
- Estimated effort: 2-4 months
- Complexity: Very High
- Hash indexes and B-tree indexes that handle concurrent access, compaction, and crash recovery. This is the core of a storage engine. Every database project in history underestimates this.
- Traversal algorithms that handle cycles, depth limits, and memory bounds.
- The spec targets <0.01ms for node lookup. Achieving this while maintaining ACID guarantees is non-trivial.

### Crate: benten-ivm (Incremental View Maintenance)
- Estimated effort: 3-6 months
- Complexity: Extremely High
- IVM is an active research area. Materialize (a funded company with a team of database PhDs) has spent years on their IVM engine. Feldera came out of VMware Research.
- The spec claims "reads are O(1)" but IVM is not free -- the cost shifts to writes. Every write must identify which views are affected and update them incrementally. For complex query patterns (joins, aggregations, nested traversals), determining the delta efficiently is a hard computer science problem.
- Your own research doc (`explore-database-is-application.md`) noted: "IVM is not effective when a base table is modified frequently" and "the cost of maintenance can be larger than refresh from scratch." A CMS with active editors could hit this.

### Crate: benten-version (Version chains)
- Estimated effort: 2-4 weeks
- Complexity: Medium
- Anchor nodes, CURRENT pointers, version traversal. Straightforward if the graph layer is solid. But interacts with everything: IVM must understand versions, sync must exchange versions, capability checks must be version-aware.

### Crate: benten-capability (Capability enforcement)
- Estimated effort: 1-2 months
- Complexity: High
- UCAN-compatible capability grants with attenuation, delegation, and revocation. UCAN itself is a non-trivial protocol with cryptographic requirements.
- "Checked at every operation boundary" means this is on the critical path for every read and write. Any performance overhead here multiplies across the entire system.

### Crate: benten-sync (CRDT merge, sync protocol)
- Estimated effort: 3-6 months
- Complexity: Extremely High
- Per-field last-write-wins with Hybrid Logical Clocks sounds simple. It is not. HLC drift, clock skew, causality violations, and merge ordering edge cases are well-documented sources of subtle bugs.
- Edge add-wins semantics must handle: concurrent edge creation to deleted nodes, concurrent edge deletion and recreation, cycles created by concurrent edge additions.
- "Schema validation on receive" means you need a schema evolution strategy that handles: sender and receiver at different schema versions, fields added on one side but not the other, type changes, required fields that don't exist on the other side.
- Tree CRDTs (which is effectively what a graph with compositions is) are among the most complex CRDT types. Concurrent structural changes to tree hierarchies have known unsolved edge cases.
- Your own research noted: "CRDTs work beautifully for text but are complex for structured data like block trees."

### Crate: benten-query (Cypher parser + query planner)
- Estimated effort: 2-4 months
- Complexity: Very High
- Cypher is a complex language. MATCH patterns, WHERE clauses, RETURN projections, ORDER BY, LIMIT, SKIP, OPTIONAL MATCH, UNWIND, WITH, CREATE, MERGE, DELETE, SET, REMOVE, CALL procedures, aggregation functions, list comprehensions, pattern comprehensions, existential subqueries.
- The openCypher grammar is non-trivial. Building a parser is maybe 2-4 weeks. Building a correct query planner that generates efficient execution plans is months of work. A naive implementation will be orders of magnitude slower than PostgreSQL's query planner for any non-trivial query.
- You could use the openCypher front-end parser, but it is JVM-based. Porting to Rust or building a new one is the effort.

### Crate: benten-persist (WAL, snapshots, disk storage)
- Estimated effort: 2-4 months
- Complexity: Very High
- WAL correctness is where databases live or die. Write-ahead logging with crash recovery, checkpoint management, log compaction, and fsync correctness is one of the most error-prone areas in systems programming.
- redb (the planned backend) is solid but is a key-value store. Mapping graph operations onto KV pairs efficiently requires a careful encoding scheme.
- Snapshot management for MVCC: when can old versions be garbage collected? How do you handle long-running readers holding references to old snapshots?

### Crate: benten-reactive (Subscriptions, notifications)
- Estimated effort: 1-2 months
- Complexity: Medium-High
- Subscription management, change detection, notification delivery. Must handle: subscriber disconnection, backpressure, subscription to query patterns (not just individual nodes), and efficient change propagation.

### Crate: benten-engine (Orchestrator)
- Estimated effort: 1-2 months
- Complexity: High
- Tying all crates together with a coherent API. Transaction coordination across storage, IVM, versioning, capabilities, and subscriptions.

### Bindings: napi-rs + WASM
- Estimated effort: 1-2 months
- Complexity: Medium
- napi-rs is mature. WASM via wasm-bindgen is mature. But the Benten Engine uses tokio for async, dashmap for concurrency -- these have specific WASM compilation challenges. ThreadsafeFunction in napi-rs is documented as difficult to use correctly.

### Total Realistic Estimate

| Component | Optimistic | Realistic | Pessimistic |
|-----------|-----------|-----------|-------------|
| benten-core | 1 week | 2 weeks | 3 weeks |
| benten-graph | 2 months | 4 months | 6 months |
| benten-ivm | 3 months | 6 months | 12 months |
| benten-version | 2 weeks | 1 month | 2 months |
| benten-capability | 1 month | 2 months | 4 months |
| benten-sync | 3 months | 6 months | 12 months |
| benten-query | 2 months | 4 months | 6 months |
| benten-persist | 2 months | 4 months | 6 months |
| benten-reactive | 1 month | 2 months | 3 months |
| benten-engine | 1 month | 2 months | 3 months |
| Bindings | 1 month | 2 months | 3 months |
| Integration testing | 1 month | 3 months | 6 months |
| **TOTAL** | **~18 months** | **~36 months** | **~63 months** |

**Even with a 5x AI acceleration multiplier, the realistic estimate is 7 months. The pessimistic is over a year.**

And these are calendar estimates. They do not include the cognitive cost of context-switching between "building a database" and "building a platform." Once you start the engine, every bug in the platform waits. Every feature request waits. Every user-facing improvement waits.

### The Non-Linear Complexity Problem

Database engines have a property that most software does not: the last 20% of correctness takes 80% of the time. The happy path -- creating nodes, running queries, getting results -- can work in weeks. But:

- What happens when the process crashes mid-write?
- What happens when two transactions conflict on the same node?
- What happens when an IVM view references a deleted node?
- What happens when a CRDT merge produces a state that violates a capability?
- What happens when a snapshot is taken while a long-running transaction is active?
- What happens when the WAL grows beyond available disk space?
- What happens when a WASM client and a native client sync and their HLC clocks are 30 seconds apart?

Each of these edge cases is a week of debugging. There are hundreds of them. They compound.

---

## 3. What Could Go Wrong (Failure Modes)

### Technical Failures

1. **Performance does not meet targets.** The spec targets <0.01ms for node lookup. This is ~10 microseconds. That is achievable for an in-memory hash map lookup in Rust. But with MVCC (snapshot isolation requires checking visibility), capability enforcement (must check grants), and IVM (must identify affected views), the actual cost per operation could be 10-100x higher. If the engine is slower than PostgreSQL+AGE for real-world Thrum queries, the entire effort was wasted.

2. **IVM overhead kills write performance.** IVM shifts read cost to write time. A CMS with active editors (the actual use case) means frequent writes. If maintaining materialized views makes writes 10x slower, the "O(1) reads" benefit is negated by unacceptable write latency.

3. **CRDT merge produces invalid graph states.** Two instances independently create edges that form a cycle in a tree structure. The CRDT merge succeeds (add-wins) but the resulting graph violates application invariants. Now what? You need application-level conflict resolution on top of the CRDT merge. The engine cannot solve this alone.

4. **Cypher parser incompleteness.** You implement 80% of Cypher. The 20% you skipped includes the exact feature a third-party module needs. Now you are in a perpetual game of whack-a-mole, implementing Cypher features on demand.

5. **napi-rs binding instability.** The V8-to-Rust boundary is a source of subtle memory issues. Tokio async runtime interactions with Node.js event loop can cause deadlocks or resource exhaustion under specific patterns.

6. **WASM compilation fails for specific crates.** dashmap uses `std::thread`. Tokio uses OS-level async primitives. These do not compile to WASM without significant changes. NAPI-RS v3's wasm32-wasip1-threads target helps but is not battle-tested for complex database workloads.

7. **Memory leaks in long-running processes.** Graph databases that maintain indexes, materialized views, and version chains have complex memory management. A slow leak that manifests after 48 hours of operation is almost impossible to catch in testing.

### Strategic Failures

8. **The platform stalls.** The Thrum web app has 3,200+ tests, a working admin, a page builder, content types, commerce, communications, media -- a real product. Building the engine means this product stops advancing for months. Competitors ship features while you debug WAL recovery.

9. **The vision shifts.** By the time the engine is production-ready (6-12 months with AI acceleration), the market may have moved. AI agents may have made CMS architectures irrelevant. A new open-source tool may have appeared that does exactly what you need. P2P may have proven unnecessary for your actual users.

10. **Bus factor.** A custom database engine requires deep expertise to maintain. If the primary developer (or AI agent) is unavailable, who fixes the IVM bug? Who debugs the CRDT merge? The existing stack (PostgreSQL, SvelteKit, TypeScript) has millions of developers who understand it.

11. **Testing surface area explosion.** The Benten Engine specification describes 10 crates with deep interdependencies. The combinatorial testing space (transaction isolation x IVM x versioning x capabilities x sync) is enormous. Fuzzing is necessary but takes weeks to set up and months to achieve meaningful coverage.

---

## 4. The Sunk Cost Question

**The existing system is not sunk cost. It is a working product.**

The Thrum codebase has:
- 15 packages, all tested and typed
- ~3,200+ tests, 0 type errors
- A working web app with admin, page builder, content CRUD, commerce
- PostgreSQL+AGE for graph with Cypher
- A module system with lifecycle hooks, dependency resolution, settings persistence
- RBAC with graph-backed permissions
- Block system with 37 blocks
- Composition materializer with a 5-step pipeline

The specification proposes replacing:
- In-memory registries (Maps) with materialized views
- Event bus with reactive subscriptions
- RestrictedEventBus + TIER_MATRIX with capability enforcement
- PostgreSQL+AGE with native graph storage
- PostgreSQL relational with indexed property queries
- createSingleton pattern with service graph nodes
- compositions.previous_blocks with version chains
- content_revisions table with version nodes
- module_settings table with settings graph nodes

That is replacing nearly every data-adjacent component in the stack. The UI, SvelteKit routes, and block components survive. Everything underneath them gets rebuilt.

**The migration cost alone is substantial.** Every test that touches the database (and in Thrum, that is most of them) needs to be rewritten. Every query that uses PostgreSQL features (JSONB, text search, array operators) needs a new implementation. The `@benten/store-postgres` package with its 116 tests gets deleted. The `@benten/auth` AgePermissionStore with its graph-backed permissions gets rewritten against a new graph API.

This is not evolution. It is a rewrite of the data layer.

---

## 5. The Alternative: What If You Did NOT Build a Custom Engine?

Here is the BEST possible outcome using existing tools:

### Architecture: "The Pragmatic Path"

```
Production (Server):     PostgreSQL 18.1 + AGE (already working)
Dev/Test:                PGlite + AGE (already working)
Edge/Browser:            PGlite WASM (already supported by store-postgres)
IVM:                     TypeScript module in @benten/engine (new)
Reactive:                WebSocket service + dependency tracking (new)
Sync:                    ElectricSQL + PGlite (existing technology)
Capabilities:            UCAN library + graph-backed grants (evolution of existing RBAC)
Version Chains:          Graph pattern on top of AGE (evolution of existing revisions)
```

### What this gets you:

1. **Graph data model:** Already have it (AGE). Cypher already works.
2. **IVM for hot paths:** Build `createMaterializedIndex<K,V>()` in TypeScript. Your own research doc recommended this. It is a generalization of the existing `sortedSnapshot()` pattern. Estimated effort: 2-4 weeks.
3. **Eager composition materialization:** Cache resolved compositions on write, invalidate on dependency change. Store dependency edges in the graph. Estimated effort: 2-4 weeks.
4. **Reactive subscriptions:** SvelteKit supports WebSockets. Build a subscription manager that notifies clients when materialized data changes. Estimated effort: 2-4 weeks.
5. **P2P sync:** ElectricSQL syncs PostgreSQL subsets to PGlite clients. Add Automerge or Yjs for collaborative editing. Estimated effort: 1-2 months when you actually need it.
6. **Capability system:** Evolve the existing graph-backed RBAC into UCAN-compatible capability grants. The graph structure is already there. Estimated effort: 1-2 months.
7. **Version chains:** Model as a graph pattern (anchor node -> CURRENT -> version node -> NEXT -> version node). This is just a convention on top of AGE. Estimated effort: 1-2 weeks.
8. **Embeddable:** PGlite is already embeddable. For server, PostgreSQL. For browser, PGlite WASM. Already working in test suite.

### Total estimated effort: 3-4 months for all of the above.

### What this does NOT get you:

- <0.01ms node lookup (you get 0.1-0.3ms via PGlite, <1ms via PostgreSQL network)
- True concurrent multi-writer embedded (PGlite is single-connection multiplexed)
- A single unified binary for server + browser + edge

### Does this matter?

For a CMS platform with dozens to hundreds of concurrent users (not thousands), 0.1-0.3ms lookups are more than fast enough. The existing Thrum platform works. The admin loads. Pages render. Content saves. The bottleneck is not database latency -- it is feature completeness, user experience, and getting to market.

---

## 6. The P2P Question

**Is P2P sync between instances a real near-term need?**

No.

P2P sync is a vision statement. It appears in the README, the spec, and the vision document. But:

- There are no P2P users today
- There is no P2P protocol implemented
- There is no sync infrastructure
- There is no conflict resolution UI
- The platform does not yet have basic features like full-text search, user management, or content versioning in the admin

Building a custom database engine to support P2P sync is like building a rocket engine to support Mars colonization before you have a working car. The rocket engine is magnificent, but you cannot drive it to the grocery store.

**Could P2P be added later without a custom engine?**

Yes. ElectricSQL + PGlite provides PostgreSQL-native sync. Automerge or Yjs provides CRDT sync for collaborative editing. A sync service on top of these can sync subgraphs between PostgreSQL instances. This is not as elegant as native engine-level sync, but it ships, it works, and it does not require building a database.

The honest question: **Is the P2P vision the real motivation for building the engine, or is the engine the real motivation with P2P as justification?** If building a database engine is intrinsically interesting (and it is -- it is one of the most intellectually stimulating projects in computer science), that is a valid reason. But it should be acknowledged as such, not dressed up as a business requirement.

---

## 7. Risk-Adjusted Timeline

### The AI Acceleration Assumption

The spec assumes AI-accelerated development. Let me examine this honestly.

AI (including advanced coding agents) excels at:
- Generating boilerplate code
- Implementing well-defined algorithms
- Writing tests for specified behavior
- Refactoring and pattern application
- Documentation

AI struggles with:
- Novel system design decisions
- Performance tuning at the microsecond level
- Debugging race conditions in concurrent systems
- Correctness proofs for distributed protocols (CRDT merge semantics)
- Understanding the implications of design choices across subsystem boundaries
- Fuzzing and finding edge cases that only manifest under specific timing conditions

A database engine is 20% "implement this well-defined algorithm" and 80% "figure out the right design, debug subtle correctness issues, and handle edge cases." AI acceleration helps with the 20%. The 80% is the hard part.

### Honest Timeline

| Phase | Traditional | AI-Accelerated (realistic) | AI-Accelerated (optimistic) |
|-------|------------|---------------------------|----------------------------|
| Core types + graph storage | 4-6 months | 2-3 months | 1-2 months |
| WAL + persistence | 2-4 months | 1-2 months | 3-4 weeks |
| IVM engine | 4-8 months | 2-4 months | 1-2 months |
| Version chains + MVCC | 2-3 months | 1-2 months | 3-4 weeks |
| Capabilities | 1-2 months | 3-4 weeks | 2-3 weeks |
| Cypher parser + planner | 3-5 months | 2-3 months | 1-2 months |
| Reactive subscriptions | 1-2 months | 3-4 weeks | 2-3 weeks |
| CRDT sync | 4-8 months | 2-4 months | 1-2 months |
| napi-rs bindings | 1-2 months | 2-4 weeks | 1-2 weeks |
| WASM bindings | 1-2 months | 2-4 weeks | 1-2 weeks |
| Integration + hardening | 3-6 months | 2-3 months | 1-2 months |
| Migration of Thrum | 2-3 months | 1-2 months | 3-4 weeks |
| **Total** | **28-51 months** | **15-27 months** | **7-13 months** |

The optimistic AI-accelerated path is 7-13 months. This assumes everything goes right, no major design pivots, no show-stopping bugs, and consistent AI assistance quality.

**The realistic path is 15-27 months.** This is the timeline I would plan around.

---

## The Contrarian Recommendations

### Sacred Cows That Might Be Wrong

1. **"The graph IS the runtime."** This sounds profound but may be wrong. Graphs are excellent for relationships. They are mediocre for tabular data, time-series, full-text search, and aggregation. A universal runtime built on graphs will be good at graph things and bad at everything else. PostgreSQL is good at everything and excellent at graphs (with AGE). Being good at everything matters more than being perfect at one thing.

2. **"IVM eliminates the query speed problem."** IVM does not eliminate it. It moves it. Instead of slow reads, you get slow writes. For a read-heavy CMS serving public pages, this is the right tradeoff. For an admin panel where editors are constantly creating and updating content, it might not be. The existing `sortedSnapshot()` pattern in Thrum is already a simple, effective IVM system. Generalizing it in TypeScript is cheaper than building it in Rust.

3. **"Everything should be in one system."** The specification wants graph storage, IVM, versioning, capabilities, sync, and reactivity in one Rust binary. This is the opposite of the Unix philosophy. Each of these is a hard problem. Combining them multiplies the interaction complexity. A capability check that interacts with an IVM view that references a version chain that is being synced via CRDT -- the debugging surface for this is nightmarish.

4. **"Sub-0.01ms is necessary."** The performance targets in the spec (<0.01ms for node lookup, materialized view read, capability check) are sub-10-microsecond. This is achievable for a pure in-memory hash map. But with MVCC visibility checks, capability enforcement, and IVM tracking, the actual cost will be higher. More importantly: is sub-10-microsecond actually necessary? The existing platform works at 0.1-0.3ms. The user cannot perceive the difference. The bottleneck is network latency (50-200ms for a page load), not database lookup.

5. **"Embeddable everywhere."** The vision of running the same engine natively, in WASM, and via napi-rs is appealing. But each target has different constraints (memory limits in WASM, threading model differences, available system calls). Building for all three from day one triples the testing surface. Ship for napi-rs first. Add WASM when you actually need it.

### Architecture Alternatives Worth Considering

#### Alternative 1: Grafeo as the Engine

Your own research recommends Grafeo as the primary in-process graph database. It already exists. It is Rust-native, embeddable, Cypher-compatible, ACID, and Apache 2.0 licensed. It does not have IVM, sync, or capabilities -- but those can be built as TypeScript layers on top.

**Effort:** Build `@benten/store-grafeo` implementing the Store interface (4-6 weeks). Add IVM as a TypeScript caching layer (2-4 weeks). Add reactive subscriptions as a TypeScript service (2-4 weeks). Keep PostgreSQL for relational data.

**Total: 2-3 months** instead of 7-27 months.

**Risk:** Grafeo is v0.5 and young. But it is backed by a team, has a growing community, and is doing the hard Rust database engineering work so you do not have to.

#### Alternative 2: The Pragmatic Stack (No Custom Engine)

PostgreSQL + AGE for production. PGlite + AGE for dev/test/edge. TypeScript IVM module (`createMaterializedIndex`). WebSocket reactivity service. ElectricSQL for sync when needed.

**Effort:** 3-4 months for everything.

**Risk:** Dependent on PostgreSQL and PGlite ecosystems. But those ecosystems have millions of users, decades of engineering, and active maintenance.

#### Alternative 3: Use Grafeo + Build Only the Differentiators

If the custom engine is truly about IVM + CRDT sync + capabilities (not basic graph storage), then use Grafeo for the graph layer and build only the novel parts:

- IVM as a Rust crate that wraps Grafeo (or any graph store)
- CRDT sync as a Rust crate that operates on Grafeo subgraphs
- Capabilities as a Rust crate that decorates Grafeo operations

This is a focused, scoped project. You are building 3 crates, not 10. The graph storage, indexes, persistence, WAL, MVCC, and query execution are Grafeo's problem.

**Effort:** 4-8 months.

**Risk:** Dependency on Grafeo's API stability. But you have contributed to a focused, achievable project instead of trying to boil the ocean.

### What to Cut

If the engine must be built, here is what does NOT belong in the first release:

1. **CRDT sync** -- defer entirely. Build the engine as a single-instance system first. Sync is the hardest part and the least needed.
2. **WASM bindings** -- defer. Ship napi-rs for Node.js. Add WASM later.
3. **Full Cypher parser** -- start with a Rust-native API (typed, safe, fast). Add Cypher as a convenience layer later. This eliminates months of parser work.
4. **Capability enforcement** -- defer. Use the existing RBAC system through the napi-rs bindings. Add engine-level capabilities when you actually have untrusted third-party code.
5. **Reactive subscriptions** -- defer. Build polling-based reads first. Add subscriptions when you have WebSocket infrastructure.

**The MVP engine:** Graph storage + indexes + MVCC + version chains + WAL persistence + napi-rs bindings.

**Estimated effort for MVP:** 3-5 months with AI acceleration.

This is still a significant project, but it is achievable and provides the foundation for everything else.

### What to Add (That Nobody Is Thinking About)

1. **A compatibility shim.** Build the engine so it can run behind the existing `Store` interface. This means the Thrum platform can switch between PostgreSQL, Grafeo, and the custom engine with zero code changes. The engine does not need to replace PostgreSQL on day one -- it needs to prove itself alongside PostgreSQL first.

2. **A benchmark suite from day one.** Not "we will benchmark later." A CI pipeline that runs the exact queries Thrum needs (handler lookup, capability check, composition resolution, content CRUD) against every commit. If a change makes handler lookup slower, the build fails.

3. **Property-based testing (proptest/quickcheck).** For MVCC, version chains, and (eventually) CRDT merge, random inputs and state machines are the only way to find the bugs that structured tests miss. This should be a requirement, not an afterthought.

4. **An escape hatch.** If the engine fails to meet targets, the platform must still work. Design for graceful degradation: if the engine is unavailable, fall back to PostgreSQL. This is insurance against the project's largest risk.

---

## The 30-Day Version

If you had 30 days to ship the most essential thing, it would NOT be a custom engine.

It would be:

1. **Week 1-2:** Build `createMaterializedIndex<K,V>()` in `@benten/engine`. Generalize the `sortedSnapshot()` pattern. Apply it to event handlers, content type schemas, and the composition materializer cache.

2. **Week 2-3:** Add dependency tracking to composition resolution. Store DEPENDS_ON edges in AGE. Eagerly materialize compositions on write. Cache resolved compositions. Invalidate on dependency change.

3. **Week 3-4:** Build a WebSocket subscription service for the admin panel. When a materialized composition changes, push the update to connected editors.

This gives you:
- O(1) event handler resolution (IVM)
- O(1) composition resolution for cached pages (IVM)
- Real-time admin updates (reactivity)
- All built on the existing PostgreSQL+AGE stack
- No new languages, no Rust compilation, no napi-rs debugging

**This is what your own `explore-database-is-application.md` research recommended.** And then the conclusion of that research was ignored in favor of building a custom engine.

---

## Bold Prediction: CMS in 2028

By 2028, the CMS landscape will look like this:

1. **AI agents will author most content.** The admin panel becomes less important. The API becomes everything. The engine that powers the API needs to be fast and correct, not novel.

2. **Local-first will become mainstream.** PGlite, ElectricSQL, and TanStack DB are already converging on this. A custom engine is not needed -- the ecosystem is building the sync layer for you.

3. **Graph databases will be commoditized.** Grafeo, Kuzu forks, and Neo4j's embedded offerings will make in-process graph databases as common as SQLite. Building your own will look like building your own key-value store in 2015.

4. **The differentiator will be the platform, not the engine.** WordPress won not because of its database layer (MySQL, the most boring choice possible) but because of its ecosystem: themes, plugins, community. Thrum's differentiator should be its composability model, its module ecosystem, its AI integration -- not its storage engine.

**Is Thrum positioned for 2028?** It is well-positioned IF it ships. The module system, the composition model, the block architecture, the AI tooling (MCP) -- these are the differentiators. The database engine is infrastructure that should be as boring as possible so the interesting stuff can shine.

Building a custom engine risks making Thrum's story "we built an amazing database that nobody used because the platform never shipped."

---

## Final Verdict

**Contrarian Score: 8/10**

The specification describes an intellectually magnificent project. It is well-researched, technically sound, and architecturally coherent. As a database design document, it is excellent.

As a business decision for a platform that needs to ship, it is wrong.

The right path is:
1. Keep PostgreSQL+AGE for production
2. Use Grafeo or PGlite for in-process hot-path operations
3. Build IVM and reactivity in TypeScript on top of the existing stack
4. Defer P2P/CRDT sync until there are actual users who need it
5. Ship the platform

If, after shipping and gaining users, the database is genuinely the bottleneck -- then build the engine. With real workload data, real performance profiles, and real user requirements. Not from a specification. From evidence.

The best engine is the one that lets you ship the platform.

---

## Sources

- [Grafeo: High-Performance Graph Database in Rust](https://github.com/GrafeoDB/grafeo)
- [Grafeo HN Discussion](https://news.ycombinator.com/item?id=47467567)
- [sled: Embedded Database in Rust (since 2016, still not 1.0)](https://github.com/spacejam/sled)
- [Everything You Need to Know About IVM](https://materializedview.io/p/everything-to-know-incremental-view-maintenance)
- [NAPI-RS v3 Announcement (WASM support)](https://napi.rs/blog/announce-v3)
- [Why Build Another Database? Motivation and Known Challenges](https://medium.com/@gawry/why-build-another-database-motivation-and-known-challenges-f421a66d487b)
- [Database startups and truisms (HN)](https://news.ycombinator.com/item?id=26444119)
- [CRDT Implementation Guide (2025)](https://velt.dev/blog/crdt-implementation-guide-conflict-free-apps)
- [The CRDT Dictionary: A Field Guide](https://www.iankduncan.com/engineering/2025-11-27-crdt-dictionary/)
- [openCypher Front-End Parser](https://github.com/opencypher/front-end)
- [Database Development with AI in 2026](https://www.brentozar.com/archive/2026/01/database-development-with-ai-in-2026/)
- [redb: Embedded Key-Value Database in Rust](https://www.redb.org/)
- [Databases and Unnecessary Complexity (HN)](https://news.ycombinator.com/item?id=38929474)
