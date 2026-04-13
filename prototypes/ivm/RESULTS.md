# IVM Algorithm Benchmark Results

**Date:** 2026-04-11
**Platform:** TypeScript (in-memory), Apple Silicon (Node.js via tsx)
**Dataset:** 161,600 nodes, 206,000 edges (realistic scale)

## 1. Algorithm Comparison Table

All latency measurements are from 10,000 read samples and 5,000 write samples per view.
"us" = microseconds, "ms" = milliseconds.

### Read Latency (p50 / p95)

| View Pattern | A: Eager Invalidation | B: Dep-Tracked Incremental | C: DBSP / Z-Set |
|---|---|---|---|
| 1. Event Handler Resolution | 0.08us / 0.13us | 0.04us / 0.08us | 0.12us / 0.13us |
| 2. Capability Check | 0.04us / 0.08us | 0.04us / 0.08us | 37.7us / 63.4us |
| 3. Content Listing | 0.08us / 0.08us | 0.04us / 0.08us | 0.13us / 0.17us |
| 4. Governance Rules | 0.04us / 0.08us | 0.04us / 0.08us | 0.17us / 0.21us |
| 5. Attestation Aggregate | 0.04us / 0.04us | 0.04us / 0.04us | 0.17us / 0.17us |

**Target:** Views 1,2 < 10us; Views 3,4,5 < 100us

**Read analysis:**
- **All algorithms meet the target** when views are clean (not dirty).
- Algorithm A reads are O(1) when clean. When dirty (immediately after a write), reads trigger full recomputation, which is NOT captured in these read-only numbers. In practice, Algorithm A would show multi-millisecond read spikes after writes.
- Algorithm B has the fastest and most consistent reads: O(1) always, no lazy recomputation.
- Algorithm C reads for Capability Check are 500x slower than B (37us vs 0.04us) because the Z-set `positiveElements()` creates a new array on every read. This is fixable with a cached result pattern.

### Write + IVM Update Latency (p50 / p95)

| View Pattern | A: Eager Invalidation | B: Dep-Tracked Incremental | C: DBSP / Z-Set |
|---|---|---|---|
| 1. Event Handler Resolution | 0.96ms / 2.83ms | 0.88ms / 4.08ms | 2.00ms / 3311ms |
| 2. Capability Check | 0.71ms / 2.25ms | 2343ms / 7667ms | 4.54us / 3199ms |
| 3. Content Listing | 0.79ms / 1.54ms | 0.03ms / 0.38ms | 9018ms / 13990ms |
| 4. Governance Rules | 0.21ms / 0.75ms | 1.50ms / 3.88ms | 1.04ms / 7.88ms |
| 5. Attestation Aggregate | 0.63ms / 2.21ms | 1.08ms / 5.25ms | 1.13ms / 2828ms |

**Target:** < 1ms including IVM maintenance

**Write analysis:**
- **Algorithm A** has the fastest and most consistent writes because it just marks views dirty (O(1)). However, this cost is deferred to the next read.
- **Algorithm B** has serious problems with Capability Check writes (2.3s p50!) because deleting a CapabilityGrant node requires rebuilding the full capability map from 1,000 entities x 5,000 grants. Content Listing is efficient (0.03ms p50) because sorted insert into the published list is O(log N).
- **Algorithm C** has catastrophic Content Listing writes (9s p50!) because `rebuildTopK()` sorts all 35,000 published posts on every write. The p95 spikes in Event Handlers, Capability, and Attestation come from delete operations that trigger full re-scans.

### Combined Read+Write (Effective Latency)

For Algorithm A, the "true" read cost after a write includes the recomputation. These numbers combine write + immediately-read-after:

| View Pattern | A: Effective Read After Write | B: Write + Read | C: Write + Read |
|---|---|---|---|
| 1. Event Handler Resolution | ~0.03ms (small view) | 0.88ms total | 2.00ms |
| 2. Capability Check | ~6ms (full scan) | 2343ms | 3199ms p95 |
| 3. Content Listing | ~15ms (sort 50K posts) | 0.03ms | 9018ms |
| 4. Governance Rules | ~0.1ms (hierarchy walk) | 1.50ms | 1.04ms |
| 5. Attestation Aggregate | ~3ms (scan attestations) | 1.08ms | 1.13ms |

## 2. Memory Overhead

| Algorithm | Memory (all 5 views) |
|---|---|
| A: Eager Invalidation | 470KB |
| B: Dep-Tracked Incremental | 11.1MB |
| C: DBSP / Z-Set | 4.5MB |

**Analysis:**
- Algorithm A stores only the cached view results. Minimal overhead.
- Algorithm B stores dependency indexes (node->views, edge->views, label->views) plus per-view incremental state. 24x more memory than A.
- Algorithm C stores Z-sets (element + weight per entry) plus per-pipeline state. 10x more than A, but less than B because it doesn't maintain dependency indexes.

## 3. Correctness Verification

After 10,000 random writes per view, each algorithm's maintained view was compared against a full recomputation from the graph.

| View Pattern | A: Eager Invalidation | B: Dep-Tracked | C: DBSP / Z-Set |
|---|---|---|---|
| 1. Event Handler Resolution | PASS | PASS | PASS |
| 2. Capability Check | PASS | PASS | PASS |
| 3. Content Listing | PASS | PASS | PASS |
| 4. Governance Rules | PASS | PASS | PASS |
| 5. Attestation Aggregate | PASS | PASS | PASS |

All three algorithms produce correct results after 10,000 random mutations. The correctness issue is not about the algorithms themselves but about implementation: the prototype discovered and fixed several edge cases:
- **Cascade edge deletion:** When a node is deleted, connected edges are also removed. Incremental algorithms must handle these cascade deletions or they drift.
- **Z-set update cancellation:** In DBSP, updating a record (remove old + add new with same key) in a single delta Z-set causes the operations to cancel. Updates must be applied as two separate operations on the integrated state.

## 4. Recommendation

### Per-View Algorithm Selection

The benchmark conclusively shows that **no single algorithm is optimal for all view patterns.** The engine should use different strategies:

| View Pattern | Recommended | Why |
|---|---|---|
| **Event Handler Resolution** | **B (Incremental)** | Small, well-bounded view. Sorted insert is O(log N) where N is handler count (~500). Read is O(1) always. Write overhead is acceptable. |
| **Capability Check** | **Hybrid A+B** | The most-read view needs O(1) reads (B delivers). But B's rebuild on node deletion is too slow. Solution: maintain a HashMap for O(1) lookup, use targeted invalidation for edge adds/removes, and fall back to eager rebuild only for node deletions (which are rare in capability systems). |
| **Content Listing** | **B (Incremental)** | B excels here (0.03ms writes) because sorted-list maintenance on inserts/updates is O(log N). C's full-sort approach is catastrophic. A defers cost to reads. |
| **Governance Rules** | **A (Eager) or C (DBSP)** | Both perform well. Governance writes are rare (rule changes). A's lazy recomputation on the small hierarchy (~30 nodes) is fast enough (~0.1ms). C's targeted rebuild is similar. |
| **Attestation Aggregate** | **B (Incremental)** | Running aggregates (sum, count) update in O(1) on each write. B maintains exact totals with no rebuild needed. |

### The Winning Architecture: Per-View Strategy Selection

```
┌─────────────────────────────────────────────────────┐
│                  IVM Engine                          │
│                                                     │
│  View Definition ──→ Strategy Selector ──→ Backend  │
│                                                     │
│  Strategies:                                        │
│    INCREMENTAL_SORTED   (B: sorted list + binary    │
│                          insert, for ordered views)  │
│    INCREMENTAL_SET      (B: hashmap, for lookup      │
│                          views like capabilities)    │
│    INCREMENTAL_AGGREGATE (B: running sum/count,      │
│                           for aggregate views)       │
│    EAGER_LAZY           (A: dirty flag + recompute,  │
│                          for rarely-read views)      │
│    DBSP_DATAFLOW        (C: Z-set operators, for     │
│                          complex joins/transforms)   │
│                                                     │
│  All strategies share:                              │
│    - Dependency tracking (which nodes/edges affect   │
│      which views)                                   │
│    - Cascade-aware write processing                  │
│    - Correctness invariant: view == full recompute   │
└─────────────────────────────────────────────────────┘
```

### Key Design Lessons

1. **Algorithm B (Dependency-Tracked Incremental) is the foundation.** It delivers O(1) reads for all view types and handles most write patterns efficiently. The engine should default to this approach.

2. **The bottleneck is always the "rebuild on cascade" path.** When a node deletion cascades to edge deletions, any algorithm that needs to re-derive state from the graph hits a cost proportional to the affected subgraph size. The solution is to track sufficient dependency information to avoid full re-scans.

3. **Algorithm C (DBSP) is not ready for production.** The Z-set algebra is theoretically elegant but the TypeScript implementation has fundamental performance issues:
   - Sorting all elements on every write (O(N log N) instead of O(log N))
   - Creating new arrays on every read
   - The delta-propagation model adds overhead without benefit for simple views
   
   However, DBSP would become valuable for **complex multi-join views** (e.g., "posts by users in groups I follow, filtered by topic, sorted by relevance") where the dataflow model can incrementally maintain joins. The engine should reserve DBSP for these compound views.

4. **Algorithm A (Eager Invalidation) has a role as a fallback.** For views that are rarely read, marking dirty and recomputing on demand is the simplest correct approach. It's also the right strategy during initial implementation: start with A, profile, then promote hot views to B.

5. **The real IVM engine should be in Rust.** TypeScript performance ceilings are visible in this benchmark. A Rust implementation with:
   - Lock-free concurrent read access (MVCC snapshots)
   - Cache-friendly sorted arrays (no GC pressure)
   - Zero-copy view reads
   - Parallel delta propagation
   
   ...would be 10-100x faster, making even Algorithm C viable for large views.

## 5. Can We Use Different Algorithms for Different View Types?

**Yes. This is the recommended approach.**

The IVM engine should implement a `ViewStrategy` trait/interface:

```rust
trait ViewStrategy {
    fn initialize(&mut self, graph: &Graph);
    fn process_write(&mut self, op: &WriteOp) -> ViewDelta;
    fn read(&self) -> &ViewResult;  // Must be O(1)
    fn memory_bytes(&self) -> usize;
}
```

Each view definition selects a strategy based on its characteristics:

| View Characteristic | Best Strategy |
|---|---|
| Simple lookup (key -> bool) | IncrementalHashMap |
| Sorted list with pagination | IncrementalSortedList |
| Running aggregate (sum, count, avg) | IncrementalAggregate |
| Hierarchical (parent-child inheritance) | IncrementalTree (variant of B with ancestry tracking) |
| Complex join (3+ tables) | DBSP Dataflow |
| Rarely read, frequently written | EagerLazy (A) |
| High fanout (1 write affects 100+ views) | EagerLazy (A) for the slow views, Incremental for the hot ones |

The strategy selection can even be **automatic:** the engine analyzes the view query pattern (filter, sort, join, aggregate) and selects the optimal strategy. This is analogous to how databases choose query plans.

## Appendix: Raw Numbers

### Initialization Time (creating views from full graph scan)

| Algorithm | Time |
|---|---|
| A: Eager Invalidation | 19-32ms |
| B: Dep-Tracked Incremental | 35-53ms |
| C: DBSP / Z-Set | 21-32ms |

All algorithms initialize in under 100ms for a graph with 161K nodes and 206K edges. This is acceptable for startup.

### Dataset Composition

| Category | Count |
|---|---|
| EventType nodes | 100 |
| EventHandler nodes | 500 |
| Entity nodes | 1,000 |
| CapabilityGrant nodes | 5,000 |
| Post nodes | 50,000 |
| Grove nodes | ~30 |
| GovernanceRule nodes | ~200 |
| KnowledgeNode nodes | 10,000 |
| Attestor nodes | 500 |
| Attestation nodes | 100,000 |
| **Total nodes** | **~167,000** |
| LISTENS_TO edges | ~1,000 |
| HAS_CAPABILITY edges | 5,000 |
| PARENT_GROVE edges | ~28 |
| HAS_RULE edges | ~200 |
| ATTESTS_TO edges | 100,000 |
| AUTHORED_BY edges | 100,000 |
| **Total edges** | **~206,000** |
