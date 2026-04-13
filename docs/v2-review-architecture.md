# Benten Platform Specification v2 -- Architecture Review

**Date:** 2026-04-11
**Reviewer:** Architecture Purity Agent
**Input:** `BENTEN-PLATFORM-SPECIFICATION.md` (v2), `critique-architecture.md` (v1), `critique-correctness.md` (v1), plus 14 additional v1 critiques
**Method:** Systematic comparison of v1 findings against v2 text, plus fresh analysis of the v2-specific additions (self-evaluating graph model, 12 operation primitives, governance, economics)

---

## Score: 7.5/10

**Justification:** v2 is a dramatically better document than v1. It addresses the two most damaging structural critiques (crate count, scope confusion), introduces a genuine architectural innovation (the 12-primitive self-evaluating graph), and expands the spec from "engine only" to "complete platform" which resolves the thin-engine contradiction by reframing the engine as one layer of a larger system. However, v2 inherits several v1 problems by deferring them to "Open Questions" rather than resolving them, introduces new risks through the operation primitive model, and the migration strategy is dangerously thin for the amount of existing code at stake.

---

## Part A: V1 Critique Resolution Scorecard

### Architecture Critique (v1: 6/10, 20 findings)

| # | v1 Finding | Severity | v2 Resolution | Verdict |
|---|-----------|----------|---------------|---------|
| 1 | 10 crates too many | High | Reduced to 4 core + 2 incremental. benten-core, benten-graph, benten-eval, benten-engine. | RESOLVED. Exactly matches the v1 recommendation of 4-5 crates. |
| 2 | CRDT sync in first release | High | Sync deferred to Phase 3. Not in core crate set. | RESOLVED. Explicitly phased out. |
| 3 | Reactive and IVM same concern | Medium | IVM now in benten-graph (merged with storage). No separate reactive crate. | RESOLVED. |
| 4 | Version chains don't need own crate | Medium | Version chain primitives in benten-core. No benten-version crate. | RESOLVED. |
| 5 | IVM algorithm unspecified | Critical | Section 2.7 still describes WHAT IVM does, not HOW. Deferred to Open Question #5. | NOT RESOLVED. See Issue 1 below. |
| 6 | MVCC design missing | Critical | Section mentions MVCC in benten-graph scope. Deferred to Open Question #4. | PARTIALLY RESOLVED. Acknowledged as in-scope for benten-graph, but still no algorithm specified. |
| 7 | Cypher parser strategy unspecified | High | Section 2.10 lists benten-query as incremental ("if needed beyond operation subgraphs"). Replaced by operation primitives as primary model. | STRUCTURALLY RESOLVED. The 12-primitive model sidesteps the Cypher problem by making it optional. Smart architectural move. |
| 8 | Performance targets lack methodology | Medium | v2 removes the per-operation target table entirely. Only target: "<0.1ms for hot-path queries" in Phase 1 build order. | IMPROVED. Honest demotion from fake precision to directional target. But no methodology still. |
| 9 | "Thin engine" vs thick engine spec | High | Reframed: the engine IS the runtime (Section 2.1), not a thin abstraction layer. The TypeScript API is thin; the Rust binary is thick by design. | RESOLVED. v2 is explicit that "there is no distinction between database and application." The contradiction dissolves. |
| 10 | "Graph IS runtime" vs "keep handlers in-memory" | Medium | Operation subgraphs ARE graph structure. No separate in-memory concern. Handlers are subgraphs evaluated by the walker. | RESOLVED by the code-as-graph model. |
| 11 | Performance targets vs CRDT overhead | Medium | CRDT deferred to Phase 3. Phase 1 is single-instance only. | RESOLVED by phasing. |
| 12 | benten-query coupled to benten-graph | Medium | benten-query listed as incremental, not core. May not exist if operation subgraphs suffice. | RESOLVED by deferral. |
| 13 | benten-persist should be part of benten-graph | Medium | Merged. benten-graph now includes "persistence (redb)." | RESOLVED. |
| 14 | Capability circular dependency | Medium | Capabilities in benten-eval, not benten-graph. benten-eval depends on benten-graph (not circular). | RESOLVED. |
| 15 | No backup/restore design | High | Not addressed. | NOT RESOLVED. |
| 16 | No schema evolution strategy | High | Not addressed. | NOT RESOLVED. |
| 17 | No memory management design | High | Open Question #7 mentions WASM binary size. Memory management still absent. | NOT RESOLVED. |
| 18 | No testing strategy | High | Not addressed. | NOT RESOLVED. |
| 19 | No observability design | Medium | Not addressed. | NOT RESOLVED. |
| 20 | No error model | Medium | Not addressed. | NOT RESOLVED. |

**Architecture resolution rate:** 11/20 resolved, 2/20 partially resolved, 7/20 not resolved.

### Correctness Critique (v1: 4/10, 15+ findings)

| # | v1 Finding | v2 Resolution | Verdict |
|---|-----------|---------------|---------|
| BUG-1 | Performance targets contradict spike data | Removed granular targets. Single "<0.1ms" directional. | IMPROVED but not resolved -- the evaluator's "200-300ns per node" claim (Section 2.4) is untested and will face the same scrutiny once Phase 1 ships. |
| BUG-2 | "Lock-free or fine-grained" contradicts MVCC | Open Question #4 acknowledges "Snapshot isolation vs serializable." Still unresolved. | NOT RESOLVED. |
| BUG-3 | CRDT add-wins creates orphan edges | Section 3.2 adds "per-edge-type policies (capability revocation MUST win)" and "Schema validation on receive." | PARTIALLY RESOLVED. Per-edge-type policies are an improvement, but no tombstone design, no dangling reference handling. |
| RACE-1 | IVM update ordering during concurrent writes | Not addressed. | NOT RESOLVED. |
| RACE-2 | Version chain CURRENT pointer concurrent updates | Not addressed. | NOT RESOLVED. |
| RACE-3 | Capability revocation during in-flight operations | Not addressed. | NOT RESOLVED. |
| NULL-1 | IVM algorithm unspecified | Still unspecified. | NOT RESOLVED. |
| NULL-2 | MVCC garbage collection unspecified | Open Question #4 mentions "garbage collection strategy." | NOT RESOLVED. |
| NULL-4 | petgraph memory at 20M nodes | v2 removes mention of petgraph. Open Question #6 asks "redb B-tree backed, LRU cache for hot Nodes, or custom storage engine?" | IMPROVED. Acknowledging the question is progress. But still no answer. |
| ARCH-1 | Spec contradicts own research | v2 reframes the engine scope -- it is now explicitly a platform runtime, not just a database. The research's "don't replace PostgreSQL with something exotic" advice is superseded by the full platform vision. | STRUCTURALLY RESOLVED by scope change. The engine IS the platform now, not a PostgreSQL replacement. |
| ARCH-2 | Scope vs team size | Not addressed. 6-phase build order with no time estimates. | NOT RESOLVED. |
| ARCH-3 | Migration path unspecified | Section 7.3 adds: "engine exposes TypeScript API via napi-rs that implements existing Thrum Store interface." | PARTIALLY RESOLVED. The adapter approach is named but not specified. See Issue 5 below. |

**Correctness resolution rate:** 3/12 resolved, 3/12 partially resolved, 6/12 not resolved.

---

## Part B: New Issues in v2

### Issue 1: IVM Remains the Specification's Central Unsolved Problem (CRITICAL)

v1 called this critical. v2 defers it to Open Question #5: "IVM algorithm: DBSP (Feldera) Z-set algebra vs red-green invalidation (rustc approach) vs custom."

This is an improvement over v1 (which did not acknowledge the question at all), but it is still the single largest risk in the entire specification. The self-evaluating graph model DEPENDS on IVM working:

- Section 2.7: "Event handler resolution, capability checks, content listings, knowledge attestation values, governance rule resolution" all use IVM.
- Section 4.3: "Governance resolution is O(1) at check time because the 'effective rules' view is maintained incrementally."
- Section 2.4: "Every WRITE checks capabilities before executing" -- if capability checking is IVM-backed, every write depends on IVM correctness.

The operation primitive model (Section 2.2) makes this MORE critical, not less. Now code execution itself is graph traversal, meaning the evaluator's hot path depends on IVM-maintained views for operation dispatch, capability resolution, and handler routing.

**What 2026 state-of-art says:** Airtable recently described their Rust IVM implementation for their in-memory database (napi-rs extension), computing compact diffs per write. Feldera's DBSP provides a formal model with a Rust runtime. Both approaches are viable but represent months of dedicated engineering. Neither is a library you drop in -- they are architectural commitments that shape every other decision.

**The honest question:** Can the self-evaluating graph model work WITHOUT general-purpose IVM? If IVM is scoped to 3-5 specific materialized views (event handlers, capabilities, content listings) maintained by hand-written incremental update logic rather than a general IVM engine, the architecture is dramatically simpler and more likely to ship. This is what the v1 architecture critique recommended ("red-green invalidation: mark dirty on write, lazily recompute on read"). v2 does not explain why this simpler approach was rejected.

**Recommendation:** Before Phase 2 begins, implement ONE materialized view (capability grants for a module) with the chosen IVM approach and benchmark it. If general DBSP is chosen, prototype it. If red-green invalidation is chosen, document it. The spec cannot remain silent on its most load-bearing subsystem.

### Issue 2: The 12-Primitive Model Is an Unproven Programming Language (HIGH)

Section 2.2 introduces 12 operation primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, GATE, CALL, RESPOND, EMIT, SANDBOX, VALIDATE). This is v2's most significant new idea and its biggest risk.

**What it IS:** A domain-specific language for web application logic, represented as graph structure rather than text. Deliberately NOT Turing complete (DAGs with bounded iteration).

**Why it matters:** This replaces Cypher as the primary computation model, which sidesteps the Cypher parser problem (v1 finding #7). It also makes the "code as graph" vision concrete -- modules are literally subgraphs that the engine walks.

**The unproven claim:** That 12 primitives are sufficient for the full range of platform operations. The spec lists composed patterns (Retry, Parallel, Compensate, DryRun, Audit, Map/Filter/Reduce) and asserts they decompose into the 12 primitives. But:

1. **TRANSFORM is doing too much work.** "Sandboxed expression with arithmetic, array built-ins (filter, map, sum, etc.), object construction. No I/O." This is a programming language embedded inside one primitive. The expression language for TRANSFORM is Open Question #1. If the expression language is too limited, every complex transformation falls through to SANDBOX (WASM). If it is too powerful, it becomes a security surface.

2. **GATE is a backdoor.** "Custom logic escape hatch. For complex validation/transformation that can't be expressed as TRANSFORM." This admits the 12 primitives are insufficient. GATE exists because the primitive set cannot cover all cases. The question is how much logic flows through GATE vs through the structured primitives. If most real-world modules use GATE extensively, the "operation subgraph" model degrades into "graph-structured source code" with GATE nodes acting as opaque function calls -- losing the inspectability and static analyzability that justify the model.

3. **No working examples.** The spec shows ONE example (Section 2.1: a simple GET /api/posts handler with 4 nodes). Real platform operations are far more complex. What does a content type creation with field validation, schema compilation, table creation, and event emission look like as an operation subgraph? How many nodes? Is it readable? Is it debuggable?

**Recommendation:** Before committing to the 12-primitive model, implement 5 representative operations as operation subgraphs (content CRUD, user authentication, permission check, module installation, composition resolution). Count the nodes. Measure readability. If the average operation requires >20 nodes or >30% GATE usage, the model needs revision.

### Issue 3: MVCC + Version Chains Dual Versioning Is Still Unresolved (HIGH)

v1 correctness critique (NULL-2) identified this: MVCC creates transient versions for transaction isolation. Version chains create persistent versions for history. How do they interact?

v2 defers this to Open Question #4: "Snapshot isolation vs serializable. Garbage collection strategy for old snapshots."

The problem is sharper in v2 because version chains are now in benten-core (primitives level) while MVCC is in benten-graph (storage level). This means benten-core defines version chain semantics without knowing the MVCC model, and benten-graph must somehow reconcile its MVCC snapshots with benten-core's version chain structure.

**redb's model:** redb provides serializable isolation with copy-on-write B-trees. Read transactions see a consistent snapshot; write transactions are serialized. Old pages are freed when all referencing readers complete. This is a mature, proven MVCC model. But redb's MVCC operates on B-tree pages, not on graph Nodes. Mapping graph-level version chains onto redb's page-level MVCC requires a design that the spec does not provide.

**The simplest resolution:** Version chains are an APPLICATION-LEVEL pattern built on top of the graph. MVCC is a STORAGE-LEVEL mechanism internal to benten-graph. They do not interact -- a transaction that creates version Node v4 is just a write that creates a Node, and MVCC handles the isolation for that write like any other. The version chain is just Nodes and Edges from the storage layer's perspective. v2 hints at this ("version chain primitives" in benten-core), but never states it explicitly. Making this explicit would resolve the issue.

### Issue 4: The Migration Strategy Is a Single Paragraph for a 3,200-Test Codebase (HIGH)

Section 7.3:

> "The engine exposes a TypeScript API (via napi-rs) that implements the existing Thrum Store interface. Existing modules can run unmodified against this adapter. Migration is then incremental -- each module is rewritten to use operation subgraphs at its own pace."

This is three sentences for migrating 15 packages, ~2,900 tests, and 37 database tables. The claim that "existing modules can run unmodified" is aspirational, not demonstrated.

**What "implements the existing Thrum Store interface" requires:**

The Thrum Store interface has 16 methods including graph operations (createNode, getNode, createEdge, traverse), relational operations (createRecord, getRecord, updateRecord, queryRecords), file operations (storeFile, getFile, deleteFile), and transaction support. The napi-rs adapter must:

1. Map Thrum's `Node` type (5 fields) to the engine's Node type
2. Map Thrum's `Edge` type to engine Edges
3. Implement `queryRecords` with SQL-like filtering against a graph backend
4. Implement `storeFile`/`getFile`/`deleteFile` (the engine spec does not mention file storage AT ALL)
5. Implement `createTable` (DDL -- the engine has no concept of tables)
6. Handle `SecurityContext` and `TrustTier` mapping to engine capabilities

Items 4 and 5 are not trivially achievable. The engine is a graph; Thrum has relational tables and file storage. The adapter must either (a) emulate tables as labeled Node collections with property indexes, or (b) maintain a parallel relational store alongside the graph. Neither is mentioned.

**Recommendation:** The migration section needs a compatibility matrix: for each of the 16 Store methods, state whether the engine supports it natively, via adapter, or not at all. Identify the methods that CANNOT be adapted (likely `createTable`, file operations) and state the migration plan for the packages that depend on them.

### Issue 5: Governance and Economics Are Premature for a Phase 1 Engine (MEDIUM)

Parts 4 (Governance), 5 (Economics), and 6 (Business Model) comprise roughly 40% of the spec's content. None of this is buildable until Phase 5 (per the build order in Part 8). Including it in the spec is not wrong -- it is important for vision alignment -- but it creates a scope illusion. A reader could conclude that the platform must support fractal governance, attestation marketplaces, and a fiat on/off ramp before it ships. This contradicts the phased approach.

**The risk:** Decision paralysis. If the Phase 1 engine design must accommodate governance resolution ("O(1) via IVM"), attestation economics ("IVM materializes attestation value per knowledge Node"), and multi-parent governance ("DAG, not tree"), then Phase 1 is overloaded with future requirements that may never materialize.

**Recommendation:** Add an explicit "Phase 1 Scope Boundary" section that states what Phase 1 does NOT include. Something like: "Phase 1 delivers a single-instance graph engine with persistence, operation evaluation, and capability enforcement. It does NOT include sync, governance, economics, identity federation, or attestation. These are documented here for architectural alignment but are not Phase 1 deliverables."

### Issue 6: Structural Invariants Need Enforcement Specification (MEDIUM)

Section 2.3 lists 14 structural invariants (DAG-only subgraphs, max depth, max fan-out, etc.). These are good -- they are exactly what makes the model tractable. But the spec does not say WHEN they are enforced:

- **Registration time only?** (Section 2.3, invariant #12: "Registration-time structural validation.") If so, what prevents runtime modification of subgraphs?
- **Invariant #13 answers this:** "Immutable once registered (new version for changes)." So subgraphs cannot be modified after registration -- mutations create new versions.
- **But:** The evaluator (Section 2.4) uses an explicit execution stack. If a CALL primitive invokes another subgraph, and that subgraph is being replaced by a new version mid-execution, the evaluator could see an inconsistent state. The spec says nothing about version pinning during execution.

**Recommendation:** State explicitly: "When the evaluator begins walking a subgraph, it pins the CURRENT version of that subgraph and all transitively-CALLed subgraphs. Version changes during execution do not affect in-progress evaluations."

### Issue 7: No Error Model, Testing Strategy, or Observability (Inherited from v1) (MEDIUM)

Seven v1 findings (#15-20 from architecture, plus testing from correctness) remain completely unaddressed:

- No backup/restore design
- No schema evolution strategy
- No memory management design (WASM)
- No property-based/fuzz/simulation testing strategy
- No observability (tracing, metrics, structured logging)
- No error model (typed error enums, cross-boundary error codes)

These are all must-haves before shipping. Their continued absence suggests the spec is focused on the happy-path architecture and has not yet grappled with operational reality. For a system that claims users will "run their own instance," operational concerns are first-class requirements.

---

## Part C: What v2 Gets Right

### C1: The Crate Consolidation Is Correct

4 crates + 2 incremental is exactly what the v1 critique recommended. benten-core (types), benten-graph (storage + persistence + MVCC + IVM), benten-eval (evaluator + capabilities), benten-engine (orchestrator). This is a buildable structure. The merging of persist into graph, reactive into IVM, and version chains into core demonstrates that the v1 feedback was heard and acted on.

### C2: The Self-Evaluating Graph Model Is Genuinely Novel

The 12-primitive operation subgraph model is the most interesting idea in the spec. It resolves the v1 "thin engine vs thick engine" contradiction by eliminating the distinction between data and code. It sidesteps the Cypher parser problem. It makes the system inspectable and AI-native by construction. It is the right architectural bet for a platform that wants AI agents as first-class citizens.

The model needs proving (see Issue 2), but the concept is sound: a deliberately non-Turing-complete DAG evaluator with bounded iteration is statically analyzable, terminates guaranteed, and can be reasoned about by both humans and machines.

### C3: The Phased Build Order Is Realistic

Phase 1 (core + graph + bindings) -> Phase 2 (evaluator + capabilities + IVM) -> Phase 3 (sync) -> Phase 4 (CMS migration) -> Phase 5 (governance + economics) -> Phase 6 (polish). Each phase delivers a testable artifact. Phase 1 can be validated independently. This is exactly the incremental proof approach the v1 critique recommended.

### C4: The Capability System Is Well-Designed

UCAN-compatible, operator-configured, enforced at the engine level, with scope attenuation. The `rs-ucan` Rust crate (updated February 2026) provides a solid foundation. Placing capability enforcement in benten-eval (separate from benten-graph) avoids the circular dependency the v1 critique identified. The replacement of fixed trust tiers with configurable capabilities is an improvement over both Thrum V3 and v1's design.

### C5: The Platform Vision Provides Strategic Clarity

Parts 3-6 (Networking, Governance, Economics, Business Model) transform the document from "engine spec" to "platform spec." This is appropriate because the engine's design decisions (IVM, capabilities, version chains, CRDT sync) only make sense in the context of the full platform. A reader now understands WHY the engine needs version chains (for fork-and-compete), WHY it needs capabilities (for P2P trust), and WHY it needs IVM (for governance resolution). v1 lacked this strategic context.

---

## Part D: Summary

### Issues Found

| # | Category | Issue | Severity |
|---|----------|-------|----------|
| 1 | Inherited | IVM algorithm still unspecified despite being the most load-bearing subsystem | CRITICAL |
| 2 | New | 12-primitive model unproven -- TRANSFORM scope unclear, GATE is a backdoor | HIGH |
| 3 | Inherited | MVCC + version chains dual versioning still unresolved | HIGH |
| 4 | New | Migration strategy is 3 sentences for a 3,200-test codebase | HIGH |
| 5 | New | Governance/economics scope creates Phase 1 decision paralysis | MEDIUM |
| 6 | New | Structural invariants lack enforcement timing specification | MEDIUM |
| 7 | Inherited | No error model, testing strategy, observability, backup/restore, schema evolution, memory management | MEDIUM (aggregate) |

### Comparison to v1

| Dimension | v1 | v2 | Change |
|-----------|-----|-----|--------|
| Architecture score | 6/10 | 7.5/10 | +1.5 |
| Crate count | 10 | 4+2 | Resolved |
| IVM specification | Absent | Open Question | Acknowledged, not resolved |
| MVCC specification | Absent | Open Question | Acknowledged, not resolved |
| Performance targets | Unrealistic | Directional only | Improved |
| Thin engine contradiction | Present | Eliminated | Resolved via reframing |
| Cypher dependency | Hard requirement | Optional (operation primitives primary) | Resolved via innovation |
| Build order | 10-step waterfall | 6-phase incremental | Improved |
| Platform vision | Absent | Comprehensive | New strategic clarity |
| Operational concerns | Absent | Still absent | No change (7 findings carried) |

### Bottom Line

v2 is a strong specification for a platform vision. It resolves the structural problems (crate explosion, scope confusion, build order) and introduces a genuinely interesting computation model (12-primitive self-evaluating graph). The remaining gaps are all in the "how" -- IVM algorithm, MVCC design, migration details, operational concerns. These are solvable problems, but they must be solved before Phase 2 begins, not deferred indefinitely as open questions. The spec earns a 7.5 because the architecture is now sound, but the engineering specification for the hardest subsystems is still absent.

---

## Sources

- [Airtable: Rewriting Our Database in Rust (IVM implementation)](https://medium.com/airtable-eng/rewriting-our-database-in-rust-f64e37a482ef)
- [Feldera DBSP: Automatic Incremental View Maintenance](https://docs.feldera.com/assets/files/vldb23-1bfe30b29f95168c8e1f427fccfc6da2.pdf)
- [Materialize: Incremental Computation in the Database](https://materialize.com/guides/incremental-computation/)
- [redb Design Documentation (MVCC, copy-on-write B-trees)](https://github.com/cberner/redb/blob/master/docs/design.md)
- [rs-ucan: Rust UCAN Implementation](https://github.com/ucan-wg/rs-ucan)
- [DBSP: Incremental Computation on Streams (CMU)](https://db.cs.cmu.edu/events/dbsp-incremental-computation-on-streams-and-its-applications-to-databases/)
- [Rust 2026: Module First, Crate Last](https://dasroot.net/posts/2026/04/rust-2026-new-features-best-practices/)
- [Modern Rust Best Practices 2026](https://onehorizon.ai/blog/modern-rust-best-practices-in-2026-beyond-the-borrow-checker)
- [Stoolap: Embedded SQL Database in Rust with MVCC](https://news.ycombinator.com/item?id=46239372)
