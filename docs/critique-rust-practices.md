# Benten Engine -- Rust Best Practices Critique (April 2026)

**Modernity Score: 6.5/10**

The specification demonstrates strong architectural thinking and a clear vision. However, the dependency selections contain several outdated or suboptimal choices, the async model has a fundamental design question that needs resolution, and several areas of Rust 2024 edition best practice are not addressed. This critique identifies 12 issues across dependencies, architecture, testing, and tooling.

---

## 1. Dependency Audit

### 1.1 petgraph -- Reconsider for this use case (Medium Risk)

**Issue:** petgraph is a general-purpose graph algorithms library (shortest path, MST, isomorphism). It is NOT a graph storage engine. Using petgraph as the foundation for a database engine that needs MVCC, persistent storage, concurrent read/write, and indexed traversals is a category error.

**What petgraph gives you:** Adjacency list/matrix data structures, DFS/BFS, Dijkstra, topological sort.

**What you need:** Concurrent multi-reader MVCC snapshots, fine-grained per-node locking, property indexes (hash + B-tree), disk-backed persistence, WAL, and incremental maintenance hooks on every mutation.

**Recommendation:** Do not use petgraph. Build a custom graph storage layer from scratch using:
- A `HashMap<NodeId, Node>` (or `papaya` / `dashmap` for concurrent access) for node storage
- Custom adjacency index structures (forward + reverse edge indexes per edge type)
- B-tree indexes (use the `BTreeMap` from `std` or a custom concurrent variant) for sorted property queries
- MVCC managed via epoch-based reclamation or snapshot isolation

petgraph's value is in graph algorithms, not graph storage. You can always pull petgraph in later if you need cycle detection or topological sort on top of your own storage, but it should not be the storage layer.

### 1.2 crepe / datafrog -- Both are research-grade; consider DBSP/FlowLog (High Risk)

**Issue:** Both crepe and datafrog are referenced as options for IVM. Neither is designed for production incremental view maintenance at the scale this engine requires.

- **crepe** (by Eric Zhang) -- A Datalog compiler embedded as a procedural macro. Elegant for static rule sets, but it compiles Datalog rules at Rust compile time. It cannot handle dynamically-defined views (which the spec requires via `engine.createView()`). Last meaningful commit activity has slowed.
- **datafrog** (by Frank McSherry, now rust-lang) -- A lightweight Datalog engine designed for Rust compiler internals (Polonius borrow checker). Extremely minimal API. No built-in incremental maintenance -- you manually build and apply update rules. Suitable as a reference implementation, not a foundation.

**What you actually need for IVM:**
1. Dynamic rule/view registration at runtime
2. Incremental delta propagation when base facts change
3. Support for aggregation, sorting, and pagination in materialized views
4. Efficient handling of view dependencies (view A depends on view B)

**Recommendations:**
- **Primary: Study DBSP** (Feldera's incremental computation model, VLDB 2023 paper by Budiu, McSherry, Ryzhyk, Tannen). DBSP provides a formal framework for automatically converting any query into an incremental dataflow program. The `dbsp` crate on crates.io is the Rust implementation. Feldera demonstrated that DBSP can handle full SQL semantics incrementally -- your Cypher/graph query subset is a smaller problem.
- **Secondary: Study FlowLog** (VLDB 2026 paper). FlowLog is built atop Differential Dataflow and outperforms state-of-the-art Datalog engines. It supports both batch and incremental execution and adds recursion-aware optimizations.
- **Practical approach:** Build a custom IVM engine inspired by DBSP's Z-set algebra (multiset with positive/negative weights for insertions/deletions). Use crepe or datafrog as learning references, not production dependencies.

### 1.3 yrs vs automerge -- Missing the best option: Loro (Medium Risk)

**Issue:** The spec lists "yrs or automerge" for CRDT sync. Both are viable, but a third option has emerged as the leader for your use case.

| Library | Strengths | Weaknesses |
|---------|-----------|------------|
| **yrs** (Y-CRDT Rust port) | Mature, proven Yjs algorithm, good text CRDT | Focused on text/document collaboration; graph sync is secondary |
| **automerge** | JSON data model, good Rust core | Historically high memory usage (improved in v3), slower merge |
| **loro** | Fugue algorithm (minimal interleaving), Rust-native, maps/lists/trees/counters, fast merge, low memory, active 2026 development | Newer, smaller ecosystem |

**Recommendation:** Evaluate Loro seriously. Its data model (maps, lists, trees, counters) maps more naturally to graph properties than yrs's text-focused model. Loro is built in Rust from the ground up (not a port), has excellent WASM support, and its Fugue algorithm produces better merge results for concurrent edits. Its performance benchmarks (memory, CPU, loading speed) lead the field as of early 2026.

For your version-chain model specifically, you may want to implement a custom thin CRDT layer rather than adopting any library wholesale. Your sync semantics (per-field LWW with HLC, add-wins edges) are simpler than what these libraries handle -- you don't need rich text CRDTs for graph property sync.

### 1.4 redb -- Solid choice, but evaluate Fjall 3.0 (Low Risk)

**Issue:** redb is a good pure-Rust embedded KV store (B-tree based, ACID, stable file format). However, the spec has not evaluated Fjall 3.0 (released January 2026).

| | redb | Fjall 3.0 | RocksDB |
|-|------|-----------|---------|
| Language | Pure Rust | Pure Rust | C++ with Rust bindings |
| Architecture | B-tree (copy-on-write) | LSM-tree | LSM-tree |
| Write throughput | Moderate | High (LSM advantage) | High |
| Read throughput | High (B-tree advantage) | Moderate (compaction reads) | Moderate |
| Binary size | Small | ~2.2 MB | Large |
| Compression | No | Default (zlib-rs) | Yes |
| WASM compat | Good (pure Rust) | Good (pure Rust) | Poor (C++ deps) |

**Recommendation:** For a write-heavy graph engine with version chains (every mutation creates a version node), LSM-tree write amplification characteristics may actually be better than B-tree copy-on-write. Profile both redb and Fjall 3.0 with your actual write pattern (many small versioned writes). redb's B-tree advantage matters more for read-heavy workloads -- but your reads are served by IVM caches, not the storage layer. The storage layer is predominantly write-path.

RocksDB should be eliminated from consideration due to C++ dependencies breaking WASM targets.

### 1.5 dashmap -- Replace with papaya (Medium Risk)

**Issue:** dashmap uses sharded synchronous locks internally. Holding a reference to an item in dashmap can deadlock, particularly in async contexts (a dashmap guard held across an await point will block the shard for all other tasks on that thread).

**papaya** is a newer concurrent hashmap with a lock-free API that makes deadlocking impossible by design. Its API is designed around epoch-based reclamation rather than lock guards.

**Recommendation:** Use papaya instead of dashmap for all concurrent map structures, especially in the MVCC and IVM subsystems where long-lived references to map entries are likely. If papaya's API doesn't fit a specific use case, use `std::sync::RwLock<HashMap>` (or `parking_lot::RwLock<HashMap>`) rather than dashmap.

### 1.6 Missing dependency: parking_lot (Low Risk)

**Issue:** The CLAUDE.md lists `parking_lot` under concurrency but the SPECIFICATION.md does not mention it. parking_lot's Mutex is 1.5-5x faster than std::sync::Mutex, its RwLock can be up to 50x faster, and it provides useful features like reentrant mutexes and deadlock detection.

**Recommendation:** Explicitly include `parking_lot` as a core dependency. Use `parking_lot::Mutex` and `parking_lot::RwLock` everywhere instead of std equivalents. The size overhead is negligible and the performance difference matters for a database engine.

---

## 2. Async Model (High Risk -- Architectural Decision)

### 2.1 Should the core engine be async?

**Issue:** The spec lists tokio as a dependency and implies async throughout. This is the single most consequential design decision in the project and it deserves more explicit treatment.

**The case for a synchronous core engine:**
- Database engines are CPU-bound (graph traversal, IVM computation, CRDT merge), not I/O-bound
- MVCC reads are in-memory hash lookups -- adding async overhead to a 10-nanosecond operation is pure waste
- Async Rust has significant ergonomic costs: `Send + Sync + 'static` bounds propagate everywhere, future sizes are unpredictable, and cancellation safety is hard
- napi-rs can bridge sync Rust to async JavaScript seamlessly (spawn blocking tasks on the tokio runtime that napi-rs already manages)
- The WASM target does not support multi-threading in most environments -- async is irrelevant there

**The case for async at specific boundaries:**
- Disk I/O (WAL writes, snapshots) benefits from async
- Network I/O (sync protocol) requires async
- Reactive subscriptions/notifications need async channels

**Recommendation: Synchronous core, async at the edges.** The engine's hot path (node lookup, IVM read, capability check, graph traversal) should be fully synchronous. Async should only appear in:
1. `benten-persist` -- WAL writes and snapshot I/O
2. `benten-sync` -- network protocol
3. `benten-reactive` -- subscription notification delivery
4. The napi-rs binding layer -- wrapping sync calls in `spawn_blocking`

This means `benten-core`, `benten-graph`, `benten-ivm`, `benten-version`, `benten-capability`, and `benten-query` should have zero tokio dependency. Only the edge crates and the orchestrator (`benten-engine`) should depend on tokio.

This architecture also keeps WASM compilation clean -- the core crates compile to WASM without any async runtime, and the WASM binding layer uses single-threaded JavaScript promises.

---

## 3. Error Handling Strategy (Medium Risk)

### 3.1 No error strategy defined

**Issue:** The specification mentions `EngineError` codes carried forward from the TypeScript engine but does not define the Rust error handling approach.

**2026 Rust consensus:**
- **Libraries (crates): `thiserror`** -- define typed, matchable error enums. Each crate should have its own error type.
- **Applications (binaries): `anyhow`** -- aggregate errors with context for top-level reporting.
- **Always preserve error chains** via `#[source]` or `#[from]`
- **Always derive `Debug`** on error types

**Recommendation:** Each `benten-*` crate should define its own error type via `thiserror`:

```rust
// benten-graph/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("edge not found: {0}")]
    EdgeNotFound(EdgeId),
    #[error("duplicate node: {0}")]
    DuplicateNode(NodeId),
    #[error("capability violation: {action} on {scope}")]
    CapabilityViolation { action: String, scope: String },
    #[error("transaction conflict: {0}")]
    TransactionConflict(String),
    #[error(transparent)]
    Storage(#[from] PersistError),
}
```

The orchestrator crate (`benten-engine`) should use a top-level `EngineError` that wraps all sub-crate errors. The napi-rs binding layer should use `anyhow` to convert Rust errors into JavaScript exceptions with full context chains.

---

## 4. Memory Allocator (Low Risk, High Impact)

### 4.1 No allocator specified

**Issue:** The spec does not mention a memory allocator. For a database engine with frequent small allocations (node properties, edge metadata, IVM delta sets), the default system allocator is suboptimal.

**2026 benchmarks:**
- **mimalloc:** 15% lower P99 latency than jemalloc for small, frequent allocations. 5.3x faster than glibc malloc under heavy multithreaded workloads. 50% lower RSS.
- **jemalloc:** Better fragmentation control for long-running processes. Maintains throughput with increased parallelism.
- **Both:** Within 3% for long-running stable workloads.

**Recommendation:** Use **mimalloc** as the default allocator via `mimalloc = { version = "0.1", default-features = false }` with `#[global_allocator]`. mimalloc's advantage on small allocations and multithreaded workloads matches your engine's profile (many small Node/Edge/Property allocations across concurrent readers). It also has a security mode with guard pages and encrypted free lists at ~10% cost -- useful for a platform that handles untrusted module data.

Make it configurable via a cargo feature flag so users can switch to jemalloc or the system allocator.

---

## 5. Testing Strategy (Medium Risk)

### 5.1 Incomplete testing framework specification

**Issue:** CLAUDE.md mentions "cargo test + criterion (benchmarks)" but does not address property-based testing, fuzzing, or the criterion vs divan decision.

**Recommendations:**

**5.1.1 Replace Criterion with Divan**
Criterion has not been in active development for ~4 years. Divan is the current standard for Rust benchmarking in 2026:
- Attribute-based registration (`#[divan::bench]`) vs boilerplate-heavy criterion
- Built-in allocation counting
- Better CI noise reduction via sample size scaling
- Type-generic benchmarks
- Actively maintained

**5.1.2 Add proptest for property-based testing**
A graph engine has complex invariants that are perfect for property-based testing:
- "After N random mutations, all materialized views are consistent with a full recompute"
- "After N random sync operations between two instances, both converge to the same state"
- "Capability enforcement is never bypassed regardless of operation sequence"
- "Version chains never have gaps or cycles"

Use `proptest` (v1.10.0 as of 2026). It supports strategy composition, automatic shrinking, and persistence of failing cases.

**5.1.3 Add fuzzing targets**
The Cypher parser (`benten-query`) and the sync protocol (`benten-sync`) are attack surfaces that should have `cargo-fuzz` targets from day one. Use `bolero` as a front-end that unifies fuzzing and property testing.

**5.1.4 Miri for unsafe code**
If any crate uses `unsafe` (likely in the MVCC layer, concurrent data structures, or FFI), run `cargo miri test` in CI to detect undefined behavior.

---

## 6. Rust 2024 Edition Usage (Low Risk)

### 6.1 Edition features not leveraged in the design

**Issue:** The CLAUDE.md correctly specifies Rust 2024 edition, but the specification does not discuss how edition 2024 features influence the design. As of April 2026, Rust is at version 1.94.1 (1.95.0 in beta). Key features to leverage:

**6.1.1 Async closures (`async || {}`)** -- Stabilized in 1.85.0 (edition 2024). Useful for the reactive subscription system where callbacks need to be async. Use `AsyncFn` / `AsyncFnMut` traits instead of `Box<dyn Fn() -> Pin<Box<dyn Future>>>`.

**6.1.2 `unsafe extern` blocks** -- Required in edition 2024. The napi-rs bindings will need this. Ensure napi-rs version used supports edition 2024 (current versions do, via wasm-bindgen issue #4218 resolution).

**6.1.3 `unsafe_op_in_unsafe_fn` default warning** -- Edition 2024 requires explicit `unsafe {}` blocks inside `unsafe fn`. This is good hygiene for a database engine. Do not suppress this lint.

**6.1.4 RPIT lifetime capture changes** -- Edition 2024 changes how `impl Trait` return types capture lifetimes. This affects the query API where traversal iterators return `impl Iterator<Item = Node>`. Use explicit `use<'a>` bounds where needed.

**6.1.5 `gen` blocks (reserved keyword)** -- Reserved for future generator blocks. Relevant to your reactive subscription system where generating a stream of changes is a natural fit. Do not use `gen` as an identifier anywhere.

**Recommendation:** Add a section to SPECIFICATION.md documenting which edition 2024 features the engine will actively use and how.

---

## 7. Crate Architecture Issues

### 7.1 benten-reactive should merge into benten-ivm (Medium Risk)

**Issue:** The spec separates IVM (`benten-ivm`) and reactive subscriptions (`benten-reactive`). In practice, IVM IS the reactive notification mechanism -- when a materialized view updates, that IS the subscription notification. Having two crates creates an artificial boundary that will require constant cross-crate coordination.

**Recommendation:** Merge into a single `benten-ivm` crate with two modules: `ivm::views` (materialized view maintenance) and `ivm::subscriptions` (change notification delivery). The subscription system is the consumer-facing API of the IVM engine, not a separate concern.

### 7.2 benten-capability should be integrated into benten-graph (Medium Risk)

**Issue:** Capability checks happen on every graph mutation. If capabilities are a separate crate, every write path requires a cross-crate call. This adds indirection and makes it easy to accidentally bypass capability checks.

**Recommendation:** Capability enforcement should be embedded in `benten-graph`'s write path as a mandatory step. The capability DEFINITIONS (grant/revoke API, UCAN serialization) can be a separate crate, but the enforcement CHECK must be in the graph layer. Pattern:

```rust
// benten-graph/src/write.rs
pub fn create_node(&self, ctx: &CapabilityContext, labels: &[Label], props: Properties) -> Result<NodeId, GraphError> {
    self.enforce_capability(ctx, Action::Create, &labels)?; // cannot be bypassed
    // ... actual creation
}
```

### 7.3 Missing: benten-index crate (Low Risk)

**Issue:** The spec mentions hash and B-tree indexes but houses them in `benten-graph`. For a database engine, the index subsystem is complex enough to warrant its own crate (index creation, maintenance, query planning integration, concurrent index updates during MVCC).

**Recommendation:** Extract `benten-index` with:
- `HashIndex<K, V>` -- for ID lookups
- `BTreeIndex<K, V>` -- for sorted/range queries
- `CompositeIndex` -- for multi-property queries
- Concurrent update protocol for MVCC consistency

---

## 8. Concurrency Model Gaps

### 8.1 MVCC implementation not specified (High Risk)

**Issue:** The spec says "MVCC: readers see a consistent snapshot while writers modify" but does not specify the MVCC mechanism. This is the hardest part of the entire engine.

**Key decisions needed:**
- **Snapshot isolation vs serializable?** The spec says "serializable transactions" but also "MVCC snapshots." These are different isolation levels with very different implementation complexity.
- **Epoch-based vs timestamp-based?** Epoch-based reclamation (crossbeam-epoch) is simpler but coarser. Timestamp-based (HLC or logical clock) gives finer granularity.
- **Garbage collection?** Old versions must be reclaimed. When? How? This interacts with the version chain feature -- if history IS the graph, when do you GC?

**Recommendation:** Study the `stoolap` project (Rust embedded SQL DB with MVCC) and the `mvcc-rs` project (optimistic MVCC for main-memory databases). Define the isolation level, the versioning mechanism, and the GC policy explicitly in the specification before writing code.

### 8.2 "Lock-free or fine-grained" is not a decision (Medium Risk)

**Issue:** The performance target table says "Concurrent writers: Lock-free or fine-grained." This is a hand-wave, not a design.

**Recommendation:** Decide: per-node `parking_lot::Mutex` (simple, predictable, sufficient for most workloads) or lock-free optimistic concurrency (higher throughput but much harder to implement correctly, and version chains make conflict detection complex). For a first release, per-node locking is the pragmatic choice. Lock-free can be an optimization in a later version after you have comprehensive correctness tests.

---

## 9. WASM Target Constraints (Medium Risk)

### 9.1 wasm-bindgen has threading limitations not addressed

**Issue:** The spec targets both napi-rs (Node.js) and wasm-bindgen (browser). But the concurrency model assumes multi-threading, and WASM in browsers has severe threading constraints:
- `SharedArrayBuffer` requires cross-origin isolation headers
- Web Workers are the only concurrency mechanism
- No shared mutable state between workers without `SharedArrayBuffer`
- `tokio` does not work in WASM (no OS threads)

**Recommendation:** Design the WASM build as single-threaded from the start:
- Use `#[cfg(target_arch = "wasm32")]` to compile out all multi-threading code
- Replace `parking_lot::Mutex` with `RefCell` in WASM builds
- Replace `papaya`/`dashmap` with `HashMap` in WASM builds
- No MVCC needed in WASM (single writer by definition)
- The WASM build gets simpler persistence (IndexedDB-backed, no WAL needed)

This means the core crates need `#[cfg]` gates around concurrency primitives, which is easier to design in from the start than to retrofit.

---

## 10. Cypher Parser Strategy (Low Risk)

### 10.1 No parser library specified

**Issue:** `benten-query` needs a Cypher parser but no parsing library is mentioned.

**Recommendations:**
- **winnow** (successor to nom) for a hand-written parser -- most control, best error messages
- **pest** for a PEG grammar-based approach -- easier to maintain, grammar is declarative
- **tree-sitter** if you want incremental reparsing (overkill for a query language)

For a query language parser that needs good error messages (user-facing), winnow is the 2026 consensus choice. It is actively maintained and produces zero-copy parsers with excellent performance.

---

## 11. Build and CI Gaps

### 11.1 No mention of cargo-deny, cargo-audit, or MSRV

**Recommendation:**
- `cargo-deny` -- license checking, duplicate dependency detection, advisory database
- `cargo-audit` -- CVE scanning for dependencies
- Define MSRV (Minimum Supported Rust Version) -- suggest 1.85.0 (edition 2024 minimum) or 1.94.0 (current stable minus one)
- `cargo-semver-checks` -- ensure crate API changes respect semver (important since the TypeScript layer depends on stable Rust API)

### 11.2 No mention of clippy configuration

**Recommendation:** Create a `.clippy.toml` or workspace-level `[lints]` in `Cargo.toml`:
```toml
[workspace.lints.clippy]
all = { level = "warn" }
pedantic = { level = "warn" }
nursery = { level = "warn" }
unwrap_used = { level = "deny" }
expect_used = { level = "warn" }
panic = { level = "deny" }  # No panics in a database engine
```

A database engine should never panic in production. Deny `unwrap()` and `panic!()` at the lint level.

---

## 12. Missing: Observability and Debugging

### 12.1 No tracing/logging strategy

**Issue:** The spec has no mention of observability. A database engine needs structured logging and distributed tracing.

**Recommendation:** Use the `tracing` crate (de facto standard in Rust 2026):
- Structured spans for transaction boundaries
- Events for node/edge mutations
- Integration with the napi-rs layer to propagate trace context from Node.js
- `tracing-subscriber` for configuration
- Feature-gated: `tracing` behind a `tracing` feature flag so it compiles to nothing when disabled (zero overhead for WASM/production builds that don't need it)

---

## Summary of Recommendations by Priority

| Priority | Issue | Action |
|----------|-------|--------|
| **Critical** | petgraph is wrong abstraction | Build custom graph storage |
| **Critical** | crepe/datafrog inadequate for IVM | Study DBSP/FlowLog, build custom |
| **Critical** | Async model undecided | Choose sync core + async edges |
| **Critical** | MVCC not specified | Design isolation level, versioning, GC |
| **High** | dashmap deadlock risk | Switch to papaya |
| **High** | Loro not evaluated for CRDT | Evaluate alongside yrs/automerge |
| **High** | No error handling strategy | thiserror per crate, anyhow at edges |
| **High** | No property-based testing | Add proptest + bolero |
| **Medium** | benten-reactive should merge with benten-ivm | Merge crates |
| **Medium** | benten-capability enforcement in wrong layer | Embed in benten-graph |
| **Medium** | WASM threading not designed | cfg-gate concurrency from start |
| **Medium** | criterion is unmaintained | Use divan |
| **Medium** | No memory allocator specified | mimalloc with feature flag |
| **Low** | No Cypher parser library chosen | winnow |
| **Low** | No cargo-deny/audit/clippy config | Add CI tooling |
| **Low** | No tracing strategy | tracing crate, feature-gated |
| **Low** | Fjall 3.0 not evaluated vs redb | Benchmark both |
| **Low** | parking_lot not explicit | Add as core dependency |
| **Low** | Edition 2024 features not documented | Document usage plan |

---

## Sources

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [Rust 1.85.0 and Rust 2024 Announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- [Rust in 2026: New Features and Best Practices](https://dasroot.net/posts/2026/04/rust-2026-new-features-best-practices/)
- [Rust Versions / Changelogs](https://releases.rs/)
- [petgraph on GitHub](https://github.com/petgraph/petgraph)
- [Designing Papaya: A Fast Concurrent Hash Table](https://ibraheem.ca/posts/designing-papaya/)
- [DashMap on GitHub](https://github.com/xacrimon/dashmap)
- [redb on GitHub](https://github.com/cberner/redb)
- [Releasing Fjall 3.0](https://fjall-rs.github.io/post/fjall-3/)
- [Fjall 3.0 on Phoronix Forums](https://www.phoronix.com/forums/forum/software/programming-compilers/1603716-rust-based-fjall-3-0-released-for-key-value-storage-engine-akin-to-rocksdb)
- [DBSP: Automatic IVM for Rich Query Languages (VLDB)](https://docs.feldera.com/assets/files/vldb23-1bfe30b29f95168c8e1f427fccfc6da2.pdf)
- [Feldera Incremental Computation Engine on GitHub](https://github.com/feldera/feldera)
- [FlowLog: Efficient and Extensible Datalog via Incrementality (VLDB 2026)](https://arxiv.org/abs/2511.00865)
- [FlowLog GitHub Artifact](https://github.com/flowlog-rs/vldb26-artifact)
- [crepe on GitHub](https://github.com/ekzhang/crepe)
- [datafrog on GitHub](https://github.com/rust-lang/datafrog)
- [Loro CRDT on GitHub](https://github.com/loro-dev/loro)
- [Loro Documentation](https://loro.dev/)
- [Automerge on GitHub](https://github.com/automerge/automerge)
- [Yrs (Y-CRDT) on Open Collective](https://opencollective.com/y-collective/projects/y-crdt)
- [Tokio vs Smol: The Async Rust Showdown (Feb 2026)](https://medium.com/@bhesaniyavatsal/tokio-vs-smol-the-async-rust-showdown-nobody-gave-you-a-cheat-sheet-for-a0952a2e7dca)
- [async-std Discontinued, Use smol](https://weeklyrust.substack.com/p/goodbye-async-std-welcome-smol)
- [The State of Async Rust: Runtimes](https://corrode.dev/blog/async/)
- [parking_lot on GitHub](https://github.com/Amanieu/parking_lot)
- [Rust Error Handling: thiserror, anyhow, and When to Use Each](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/)
- [How to Design Error Types with thiserror and anyhow (Jan 2026)](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view)
- [Memory Allocator Benchmarks: mimalloc vs jemalloc vs tcmalloc in 2026](https://stratcraft.ai/nexusfix/news/memory-allocator-benchmarks-2026)
- [The Power of jemalloc and mimalloc in Rust](https://medium.com/@syntaxSavage/the-power-of-jemalloc-and-mimalloc-in-rust-and-when-to-use-them-820deb8996fe)
- [Divan: Fast and Simple Benchmarking for Rust](https://nikolaivazquez.com/blog/divan/)
- [Divan on GitHub](https://github.com/nvzqz/divan)
- [proptest on GitHub](https://github.com/proptest-rs/proptest)
- [bolero on GitHub](https://github.com/camshaft/bolero)
- [Stoolap: High-performance embedded SQL database in Rust](https://stoolap.io/)
- [mvcc-rs on GitHub](https://github.com/avinassh/mvcc-rs)
- [NAPI-RS Documentation](https://napi.rs/)
- [napi-rs on GitHub](https://github.com/napi-rs/napi-rs)
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [Rust Testing Patterns for Reliable Releases (March 2026)](https://dasroot.net/posts/2026/03/rust-testing-patterns-reliable-releases/)
- [Rust Async Practical Patterns (Feb 2026)](https://dasroot.net/posts/2026/02/rust-async-practical-patterns-high-performance-tools/)
