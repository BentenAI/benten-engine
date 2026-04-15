# V2 Completeness Review: Benten Platform Specification

**Reviewer:** Composability & Extensibility Specialist
**Document:** `docs/BENTEN-PLATFORM-SPECIFICATION.md` (v2)
**Prior reviews:** `critique-composability.md` (v1, score 3/10), `critique-data-integrity.md` (v1, score 3/10)
**Date:** 2026-04-11
**Score: 6/10**

---

## Executive Summary

V2 is a substantial improvement over v1. The specification went from a database engine document to a platform specification covering vision, networking, governance, economics, and legal structure. Of the 8 specific concerns raised in this review prompt, v2 meaningfully addresses 4, partially addresses 2, and leaves 2 unaddressed. The 12 operation primitives cover the described use cases more completely than v1, but several vision scenarios still have structural gaps in how they would actually be built from those primitives.

---

## Part A: Does V2 Address the V1 Critique Items?

### 1. Write Interception (was missing -- now GATE?)

**V1 problem:** No mechanism for modules to intercept and transform/reject writes before they happen. The `pipeline` and `filter` dispatch modes from Thrum's TypeScript event system had no analogue.

**V2 answer:** Partially addressed, through two mechanisms:

**(a) GATE primitive (Section 2.2, item 7):** Described as "Custom logic escape hatch" with "Capability checking via `requires` property on any Node." The route handler example in Section 2.1 shows GATE used for capability checking with an ON_DENIED error edge. This covers the **rejection** side of write interception (capability gating stops unauthorized writes).

**(b) Subgraph composition:** Because route handlers ARE subgraphs, a module can insert a VALIDATE or GATE node before a WRITE node in any handler. This is more powerful than the TypeScript pipeline/filter pattern because it is structural -- the interception is visible in the graph, not hidden in a priority-ordered callback list.

**What is still missing:** GATE is described as doing "complex validation/transformation that can't be expressed as TRANSFORM." But the spec never defines WHAT GATE executes. It is simultaneously described as a "custom logic escape hatch" and as the capability checker -- two different roles conflated into one primitive. Critically:

- Does GATE execute arbitrary Rust code? If so, how is that code registered? (Plugin loading model problem.)
- Does GATE execute a WASM module? If so, it overlaps with SANDBOX.
- Does GATE execute an operation subgraph? If so, it overlaps with CALL.
- Can GATE transform its input (like TypeScript's `pipeline` mode)? Or can it only accept/reject (like TypeScript's `filter` mode)?

**Verdict:** The structural pattern (insert nodes into subgraphs before WRITE) is a better interception model than callbacks. But GATE itself is under-specified -- it is a placeholder name for "custom logic goes here" without defining the interface.

### 2. Storage Backend Abstraction

**V1 problem:** redb hardcoded, no `PersistenceBackend` trait, no way to swap storage.

**V2 answer:** Partially addressed. Section 2.10 shows a revised crate structure:

```
benten-graph/      # Graph storage, indexes (hash + B-tree), MVCC, persistence (redb), IVM
```

The crate consolidation (10 crates to 4-5) means persistence is now inside `benten-graph` rather than a separate `benten-persist` crate. This actually makes abstraction *harder* -- persistence is coupled to graph operations, not separated behind a trait.

Section 9 (Open Questions, item 6) asks "Disk-backed graph: redb B-tree backed, LRU cache for hot Nodes, or custom storage engine?" This shows the team acknowledges the question but has not answered it.

**What is still missing:** No `PersistenceBackend` trait. No builder pattern for swapping backends. The v1 critique recommended exactly this, and v2 did not adopt it.

**Verdict:** Not addressed. The structural change (consolidating into fewer crates) actually makes the problem harder to fix later because persistence and graph logic are now in the same crate.

### 3. Custom Index Types

**V1 problem:** Only hash and B-tree indexes. No trait for custom index types (full-text, vector, spatial, time-series).

**V2 answer:** Not addressed. Section 2.10 still lists "indexes (hash + B-tree)" in the `benten-graph` crate description. No `IndexProvider` trait. No mention of full-text, vector, spatial, or time-series indexes anywhere in the document.

**What the competition offers (2026):** SurrealDB 2.0 now supports HNSW and M-tree vector indexes, full-text search indexes with custom analyzers, and unique indexes -- all definable via `DEFINE INDEX`. PostgreSQL's extension ecosystem continues to be the gold standard (pgvector, PostGIS, pg_trgm, etc.). For a platform targeting AI-native use cases (Section 1.3, principle 8), the absence of vector/embedding indexes is a significant gap.

**Verdict:** Not addressed.

### 4. Plugin Loading Model

**V1 problem:** No specification for how third-party code gets into the engine.

**V2 answer:** Significantly improved. The spec now provides TWO plugin mechanisms:

**(a) Operation subgraphs (code-as-graph):** Modules are composed from the 12 primitives. "Installing a module = syncing its operation subgraphs. Code travels as data." (Section 2.1.) This is a genuine innovation -- plugins are DAGs of operation nodes, not loaded binaries. They are versionable, inspectable, syncable, and capability-gated.

**(b) SANDBOX primitive for escape hatches (Section 2.9):** WASM computation via @sebastianwessel/quickjs. "No re-entrancy. Fuel-metered. Time-limited." For logic that cannot be expressed as operation nodes.

This is a clean two-tier model: most plugin logic is expressed as operation subgraphs (safe, inspectable, deterministic); complex computation is delegated to WASM sandboxes (metered, isolated).

**What is still missing:** The spec does not address how Rust-native extensions work. For core extensions (custom index types, custom storage backends, custom CRDT strategies), operation subgraphs and WASM are insufficient -- these need native performance. The v1 critique recommended a two-tier model (WASM for application extensions, compiled crates for core extensions). V2 provides the WASM tier but not the compiled-crate tier.

**Verdict:** Mostly addressed for application-level plugins. Not addressed for engine-level extensions.

### 5. ACID Guarantees

**V1 problem:** Transactions described at high level but no formal guarantees. No failure-mode analysis.

**V2 answer:** Partially improved. Section 2.2 specifies that WRITE has "Auto version-stamp" and "Typed error: ON_CONFLICT, ON_DENIED." The WRITE primitive includes "conditional CAS" (compare-and-swap), which provides optimistic concurrency control at the primitive level. Section 2.3 structural invariants include "Immutable once registered (new version for changes)" (invariant 13), which prevents TOCTOU attacks.

**What is still missing:** The v1 data integrity critique identified that "MVCC" and "serializable" were conflated. V2 drops the explicit mention of "serializable transactions" from the main body (it was in v1 Section 2.7) but does not replace it with a clear isolation level commitment. Open Question 4 (Section 9) asks "MVCC implementation: Snapshot isolation vs serializable. Garbage collection strategy for old snapshots." -- acknowledging the question without answering it.

The CAS on WRITE is a step forward for single-node atomicity. But multi-node transactions (e.g., "transfer funds from account A to account B atomically") are not specified. The 12 primitives have no TRANSACTION primitive -- if a subgraph contains two WRITE nodes and the second fails, is the first rolled back? The "Compensate" pattern is listed as a composition (BRANCH on error + CALL undo subgraph), but compensation is eventual, not atomic.

**Verdict:** Single-node integrity improved (CAS, version stamps). Multi-node transactional atomicity still unspecified.

### 6. WAL Design

**V1 problem:** WAL mentioned but no protocol specified. No crash recovery procedure. No durability guarantees.

**V2 answer:** Not addressed. V2 does not mention WAL at all. The v1 spec mentioned `benten-persist` handling "WAL, snapshots, disk storage" and exposed `engine.checkpoint()`. V2 consolidated persistence into `benten-graph` and removed all WAL references. Open Question 6 asks about "redb B-tree backed, LRU cache for hot Nodes" but not about WAL or crash recovery.

redb (the chosen storage engine) provides its own crash-safe transactions using a two-phase commit protocol with checksummed pages. If the spec relies on redb's guarantees, it should say so explicitly.

**Verdict:** Not addressed. Arguably regressed from v1 (which at least mentioned WAL).

### 7. Constraint Enforcement

**V1 problem:** No mechanism for unique constraints, required properties, type enforcement, or edge cardinality constraints.

**V2 answer:** Improved through the VALIDATE primitive. Section 2.2, item 12: "VALIDATE: Schema + referential integrity check. Before writes, on sync receive." This is a dedicated primitive for constraint checking, which is a significant improvement over v1 (which had no validation mechanism at all).

**What is still missing:** VALIDATE is described as a primitive that can be placed in subgraphs, but the spec does not define:
- How schemas are declared (what does a "required properties" declaration look like in the graph?)
- Whether VALIDATE is automatically inserted before every WRITE, or only when explicitly placed in a subgraph. If the latter, a subgraph author can simply omit VALIDATE and bypass all constraints.
- How VALIDATE interacts with sync. The spec says "on sync receive" but does not specify what happens when a synced node violates a local constraint (reject the sync? accept with warning? quarantine?).
- Whether unique constraints exist at all. VALIDATE checks "schema + referential integrity" but uniqueness is a global constraint, not a schema check. How does the engine enforce "no two User nodes can have the same email"?

**Verdict:** Improved (dedicated primitive exists) but still under-specified. The critical question -- is VALIDATE mandatory or opt-in -- is unanswered.

### 8. Version Chain Atomicity

**V1 problem:** CURRENT pointer updates not atomic. Crash between steps could leave anchor with no CURRENT pointer. Concurrent CURRENT updates could fork the chain.

**V2 answer:** Partially addressed. Section 2.5 describes the same version chain model as v1 but adds that "NEXT_VERSION edges linking the chain (becomes a commit DAG when concurrent edits branch)." The explicit acknowledgment that concurrent edits branch into a commit DAG is new and important -- it means concurrent CURRENT updates are handled by design (both branches are valid) rather than being a corruption scenario.

Section 2.3, invariant 13 states "Immutable once registered (new version for changes)" which, combined with CAS on WRITE, means version nodes are created atomically and are never modified after creation.

**What is still missing:** The atomicity of the anchor-to-CURRENT edge update is still not specified. Creating a new version node (immutable, CAS) is atomic. But updating the CURRENT pointer from v3 to v4 requires: (a) create v4, (b) create NEXT_VERSION edge v3->v4, (c) move CURRENT from v3 to v4. If the process crashes between (b) and (c), the anchor still points to v3 even though v4 exists. The v1 critique's recommendation to "lock anchor, create version, link chain, move CURRENT, unlock" as a single atomic operation has not been adopted.

The commit DAG model elegantly handles concurrent edits, but it introduces a new question: if the version chain has branched, which branch does CURRENT point to? Is there a merge operation? The spec mentions "commit DAG" in passing but provides no merge semantics.

**Verdict:** Improved (commit DAG model for concurrency) but the atomic CURRENT-pointer-update problem remains unspecified.

---

## Part B: Are the 12 Primitives Complete for ALL Use Cases?

The spec describes these use cases: CMS content management, governance voting, knowledge attestation, compute marketplace, AI agents, fractal sub-Groves, P2P sync, Atriums (private sync), Digital Gardens (community spaces), and the economic system (Benten Credits). Let me trace each through the 12 primitives.

### Use Case 1: CMS Content Management

**Can be built?** Yes. READ + WRITE + VALIDATE + TRANSFORM + RESPOND cover CRUD. GATE covers access control. ITERATE covers listing. BRANCH covers conditional logic (published vs draft). This is the primary use case and the primitives handle it well.

**Gap:** No primitive for "streaming" or "pagination." ITERATE processes bounded collections, but for a content listing with 50,000 entries, ITERATE with maxIterations=50000 is wasteful. The spec needs a cursor/pagination mechanism -- either as a property on READ (read with offset/limit) or as an ITERATE variant (iterate with cursor state).

### Use Case 2: Governance Voting

**Can be built?** Mostly. A vote is a WRITE (create vote Node). Tallying votes is READ + ITERATE + TRANSFORM. Time-bounded voting uses WAIT (suspend until deadline). Quadratic voting uses TRANSFORM for the sqrt calculation. Liquid delegation uses READ to traverse delegation edges.

**Gap:** *Atomic tally + enforce.* After a vote closes, the governance outcome must be atomically computed and applied. This is a multi-step operation: READ all votes, TRANSFORM to compute result, WRITE to update governance rules, EMIT to notify. If the WRITE fails mid-tally (e.g., a concurrent governance change), there is no rollback mechanism -- only compensation. For governance decisions that modify capabilities (e.g., "the community voted to revoke Alice's moderator access"), eventual compensation is not acceptable; the capability change must be atomic with the tally.

### Use Case 3: Knowledge Attestation Marketplace

**Can be built?** Yes. An attestation is a WRITE (create ATTESTED_BY edge with cost/timestamp). Fee calculation is TRANSFORM. Payment is a composition of READ (check balance) + VALIDATE (sufficient funds) + WRITE (debit attestor) + WRITE (credit existing attestors) + EMIT (notification). IVM materializes "attestation value per knowledge Node."

**Gap:** *Multi-party payment atomicity.* "Fees flow to existing attestors (distribution set by community)" -- if there are 100 existing attestors, the fee distribution requires 100 WRITE operations (one per attestor). If WRITE #73 fails, the first 72 attestors have been credited but #73-#100 have not. Without multi-node transactions, the fee distribution is not atomic. The compensation pattern (reverse all 72 credits) is fragile and expensive.

### Use Case 4: Compute Marketplace

**Can be built?** Partially. Listing compute resources is READ. Bidding/purchasing is WRITE. Payment is the same credit transfer pattern as knowledge attestation.

**Gap:** *Verification of computation.* The spec says "verification of computation through verifiable services (storage, bandwidth) initially, general compute later" (Section 5.3). But the 12 primitives have no mechanism for verifying that a remote computation was performed correctly. SANDBOX runs computation locally in WASM. There is no VERIFY_REMOTE or ATTEST_COMPUTATION primitive. The spec implicitly requires a trust model for compute verification, but does not define one. This is deferred to "general compute later" which is reasonable for Phase 1, but the primitives should be designed to accommodate it.

### Use Case 5: AI Agents as First-Class Citizens

**Can be built?** Yes, for graph-operating agents. An AI agent is an entity with capability grants. It discovers the graph via READ, reasons via TRANSFORM, acts via WRITE (gated by capabilities), and communicates via EMIT. The graph is "self-describing and inspectable" (Section 1.3, principle 8) which is ideal for LLM-based agents.

**Gap:** *Agent-to-agent communication.* Two AI agents operating on the same graph need to coordinate (e.g., one agent discovers content, another translates it). The 12 primitives have no direct inter-agent communication mechanism. EMIT is fire-and-forget. WAIT can pause until a signal, but the spec does not define how Agent A sends a signal to Agent B's WAIT node. The implicit answer is "write a coordination Node that Agent B subscribes to via IVM" -- but this is a pattern, not a primitive, and the spec should describe it.

### Use Case 6: Fractal Sub-Groves with Inherited Governance

**Can be built?** The governance model is well-specified. Section 4.3 describes three override modes (REPLACE, EXTEND, EXEMPT) with prototypal resolution. IVM materializes "effective rules" for O(1) checks.

**Gap:** *Governance conflict resolution across polycentric parents.* Section 4.4 says a Grove can have MULTIPLE parent Groves (DAG, not tree), with conflicts resolved by "explicit priority, union (strictest wins), local override, or mediation." The "mediation" option implies a process -- potentially involving human deliberation -- but the 12 primitives have no NEGOTIATE or MEDIATE primitive. Mediation would need to be composed from WAIT (suspend until mediators respond) + READ (gather mediator votes) + BRANCH (decide outcome). This is expressible but the spec should acknowledge that mediation workflows are a key composition pattern and provide a reference subgraph.

### Use Case 7: Fork-and-Compete

**Can be built?** The mechanism is described: fork = stop syncing + modify governance + publish. Members choose which governance model they prefer.

**Gap:** *Fork integrity.* The v1 data integrity critique (Issue 10) flagged that fork provides no consistency guarantee. V2 does not address this. A fork taken mid-sync may capture partial state. After fork, some edges may reference nodes that were not yet synced (dangling references). The spec needs to define fork as a consistent snapshot operation, not just "stop syncing."

---

## Part C: Specific Issues Found

### Issue 1: GATE Is Three Things Pretending to Be One (Critical)

GATE is simultaneously described as:
- A capability checker ("require capability store:read:post/*" in the Section 2.1 example)
- A "custom logic escape hatch" (Section 2.2 description)
- Something for "complex validation/transformation that can't be expressed as TRANSFORM" (Section 2.2)

These are three different roles. Capability checking should be automatic (the spec says "every WRITE checks capabilities before executing" in Section 2.8), not requiring explicit GATE nodes. Custom logic needs a defined execution model (what runs inside GATE?). Validation has its own primitive (VALIDATE). GATE is doing too much and defining too little.

**Recommendation:** Split GATE:
- Capabilities are enforced automatically by the engine on every WRITE (as Section 2.8 already states). Remove GATE from the capability-checking role.
- Rename GATE to GUARD for explicit additional authorization checks beyond automatic capability enforcement.
- Define GUARD's execution model: does it evaluate a subgraph (via CALL)? A condition expression (via TRANSFORM semantics)? A WASM module (via SANDBOX)?

### Issue 2: No Multi-Node Transaction Primitive (Critical)

The 12 primitives include WRITE for single mutations and ITERATE for bounded loops. But there is no TRANSACTION primitive that groups multiple WRITEs into an atomic unit. The "Compensate" composition (BRANCH on error + CALL undo subgraph) provides eventual consistency, not atomicity.

Real use cases that require atomicity:
- Transfer credits between accounts (debit A, credit B -- must both succeed or both fail)
- Publish a content node + update its routing table + emit a notification (partial publish = broken page)
- Governance vote tally + rule application (partial = inconsistent governance)
- Knowledge attestation fee distribution to multiple attestors

The spec says WRITE supports "conditional CAS" which provides single-node atomicity. But Benten Credits, governance, and marketplace features all require multi-node atomic operations.

**Recommendation:** Either:
(a) Add a TRANSACTION primitive that wraps a subgraph in an atomic boundary, or
(b) Define that a subgraph evaluation IS a transaction (all WRITEs in a single subgraph evaluation are atomic), or
(c) Explicitly state that multi-node atomicity is NOT supported and all multi-write operations must be designed for eventual consistency via compensation.

Option (b) is the most elegant and aligns with the "subgraph as unit of computation" philosophy. It would mean: if any WRITE in a subgraph fails, all WRITEs in that evaluation are rolled back. This is feasible because subgraphs are DAGs with bounded execution -- the engine knows all nodes at registration time and can pre-allocate a transaction scope.

### Issue 3: IVM Algorithm Is Still an Open Question (High)

Section 9, Open Question 5: "IVM algorithm: DBSP (Feldera) Z-set algebra vs red-green invalidation (rustc approach) vs custom." IVM is described as "the key innovation" (v1 Section 2.2) and is load-bearing for governance resolution (O(1) check), capability enforcement, event handler dispatch, and content listings. Yet the actual algorithm is an open question.

This matters for the specification because the choice of IVM algorithm constrains what views can be expressed:
- DBSP supports arbitrary SQL-like queries including joins, aggregations, and nested subqueries. It handles incremental updates on all of these.
- Red-green invalidation only marks views as dirty and recomputes them on next read. It does not support incremental updates.
- A custom approach needs to be defined.

If the spec promises "O(1) reads from materialized views" but the IVM algorithm only supports red-green invalidation, views are NOT O(1) -- they are O(recompute) on first read after a change.

**Recommendation:** Commit to an IVM approach. DBSP (Feldera) is the strongest candidate for a graph database because it handles incremental maintenance of recursive queries (needed for transitive closure, which governance inheritance and capability chain resolution both require). State this in the spec rather than leaving it as an open question.

### Issue 4: CRDT Merge Strategies Are Still Hardcoded (High)

The v1 critique flagged per-field LWW as insufficient for text, counters, and sets. V2 Section 3.2 specifies the exact same two strategies:
- "Node properties: per-field last-write-wins with Hybrid Logical Clocks"
- "Edges: add-wins with per-edge-type policies (capability revocation MUST win)"

V2 adds "per-edge-type policies" which is an improvement (edge merge behavior can vary by edge type), but property-level merge is still locked to LWW.

**Impact:** For a platform that wants collaborative editing (Digital Gardens as "a Wikipedia"), LWW on text properties means one editor's changes are silently discarded on concurrent edits. This is a known, solved problem -- Yrs (Yjs for Rust), Automerge, and Loro all provide CRDT text types. The spec mentions none of them despite the v1 critique specifically recommending surfacing their extensibility.

The new addition of "non-deterministic values captured in version chain, not replayed" (Section 3.2) is a smart detail -- it means LWW decisions are auditable. But auditable data loss is still data loss.

**Recommendation:** At minimum, add a `mergeStrategy` annotation on property schemas. Default to LWW. Support `text` (use Yrs/Automerge CRDT text), `counter` (PN-Counter), and `set` (Add-Wins Set) as built-in alternatives. Allow custom strategies via WASM modules for domain-specific needs.

### Issue 5: Expression Language for TRANSFORM Is Undefined (High)

Section 2.2 says TRANSFORM provides "Sandboxed expression with arithmetic, array built-ins (filter, map, sum, etc.), object construction. No I/O." Section 9, Open Question 1 asks: "Expression language for TRANSFORM: What specific syntax? JavaScript subset? Custom DSL? How much of @benten/expressions carries forward?"

TRANSFORM is one of the most-used primitives (it appears in almost every subgraph), and its expression language is undefined. This is not a minor detail -- it determines:
- What plugins can compute without reaching for SANDBOX
- Whether existing @benten/expressions code can be reused
- Whether expressions are statically analyzable (critical for cost estimation and determinism classification)
- Whether expressions can be content-addressed (required for sync)

**Recommendation:** Define the expression language in the spec. The safest choice is a subset of JavaScript (familiar to developers, tooling exists for static analysis) with explicit restrictions: no closures, no `this`, no prototype access, no I/O, no `eval`, deterministic Math. This aligns with the existing @benten/expressions evaluator (jsep + custom AST walker).

### Issue 6: No Pagination/Cursor Mechanism (Medium)

READ can retrieve "by ID, query, materialized view." ITERATE processes bounded collections. Neither supports cursor-based pagination natively. For a CMS with thousands of content nodes, "get me items 51-100 of a sorted content list" requires either:
- ITERATE over all items with maxIterations=100, skip 50 (wasteful)
- A materialized view that is pre-paginated (impractical -- you would need a view per page)
- A cursor-based READ variant (not specified)

**Recommendation:** Add `offset` and `limit` properties to READ (or to the query pattern it evaluates). This is standard in every database and CMS. Alternatively, make IVM views support indexed access (retrieve view[50..100]).

### Issue 7: Sync Envelope / Referential Integrity Still Unaddressed (Medium)

The v1 data integrity critique (Issue 2) flagged that syncing a subgraph that references nodes outside the sync boundary creates dangling edges on the receiving instance. V2 Section 3.2 adds "Schema validation on receive" and "Move = atomic CRDT operation (not decomposed to delete+create)" which are improvements. But the core orphan-edge problem is unchanged: if I sync Node X which has an edge to Node Y, and Node Y is outside my sync scope, I receive a dangling reference.

V2 also adds "Triangle convergence" with deduplication, which is a practical improvement for multi-hop sync. But triangle convergence does not solve the referential integrity problem -- it just ensures changes propagate to all relevant peers.

**Recommendation:** Define a "sync envelope" rule: edges whose targets are outside the sync scope are either (a) included as "stub" anchor nodes (identity only, no version data), (b) tagged as "unresolved" and lazily fetched on traversal, or (c) excluded from the sync with a manifest of excluded references. Option (b) is the most practical for a P2P system.

---

## Part D: Comparison to Competitors (2026)

### Payload CMS 3.x

Payload's plugin system remains the gold standard for CMS extensibility:
- Config-based: plugins receive the existing config and return a modified config
- Collection hooks: beforeChange, afterChange, beforeRead, afterRead, beforeDelete, afterDelete -- at collection and field levels
- Custom admin panel components (React)
- Full context (req, user, locale) available in every hook
- Plugins are just npm packages -- no special build step

**What Benten v2 offers that Payload doesn't:** Code-as-graph (inspectable, versionable handlers), WASM sandboxing, capability-gated execution, P2P sync of module code, AI-inspectable operation DAGs.

**What Payload offers that Benten v2 doesn't:** A defined module registration API, lifecycle hooks at every data operation, admin UI extension mechanism, custom field types with UI components.

### Strapi 5

Strapi's plugin extensibility includes:
- Server + admin panel dual registration for custom fields
- Content-type lifecycle hooks (beforeCreate, afterCreate, beforeUpdate, etc.)
- Custom controllers, services, routes, policies, middlewares
- Admin Panel API (register, bootstrap, registerTrads)

**Key limitation Strapi has that Benten avoids:** Custom fields "cannot add new data types" -- must use built-in types. Benten's schema-free graph has no such limitation.

**What Strapi offers that Benten v2 doesn't:** A mature, documented plugin lifecycle with clear extension points at every layer (routes, controllers, services, models, admin panel).

### SurrealDB 2.0

SurrealDB has leapfrogged significantly since the v1 critique:
- Custom functions compiled to WASM with "full query engine access"
- HNSW and M-tree vector indexes (AI-native)
- Full-text search with custom analyzers
- DEFINE EVENT for server-side triggers on data changes
- Record-level access control via DEFINE ACCESS
- Live queries for reactive subscriptions

**What Benten v2 offers that SurrealDB doesn't:** Code-as-graph, version chains with content addressing, P2P CRDT sync, governance as configurable subgraphs, the entire community/economics layer.

**What SurrealDB offers that Benten v2 doesn't:** Custom index types (vector, full-text), WASM functions with query engine re-entrancy (Benten's SANDBOX explicitly forbids re-entrancy), event triggers (DEFINE EVENT), and a fully defined query language.

---

## Part E: Things Described in the Vision That CANNOT Be Built with Current Primitives

1. **Multi-node atomic transactions** -- Required by Benten Credits (transfer), knowledge attestation (multi-party fee distribution), and governance (tally + enforce). No TRANSACTION primitive exists. Compensation is not equivalent to atomicity.

2. **Collaborative text editing in Digital Gardens** -- "Like a Wikipedia" requires concurrent editing of rich text. LWW on text properties discards one editor's work. No CRDT text type specified.

3. **Compute marketplace verification** -- "Verification of computation through verifiable services" requires a trust/attestation mechanism for remote computation. No primitive addresses this. (Acknowledged as deferred.)

4. **Custom indexes for AI-native use cases** -- "AI agents are first-class citizens" but there are no vector/embedding indexes for semantic search. An AI agent cannot do "find the 10 most semantically similar documents" without an approximate nearest neighbor index.

5. **Real-time collaborative presence** -- Digital Gardens imply collaborative spaces, but the 12 primitives have no ephemeral state mechanism. WRITE creates persistent version nodes. There is no way to represent "Alice is currently editing paragraph 3" without polluting the version history with transient state.

---

## Score Justification: 6/10

| Category | V1 Score | V2 Score | Reasoning |
|----------|----------|----------|-----------|
| Write interception | 0 | 6 | Subgraph composition is structurally better than callbacks, but GATE is under-defined |
| Storage abstraction | 0 | 1 | Still hardcoded to redb, crate consolidation made it harder to abstract |
| Custom indexes | 0 | 0 | Unchanged from v1 |
| Plugin loading | 0 | 7 | Code-as-graph + WASM sandbox is a strong two-tier model for application plugins |
| ACID / transactions | 2 | 4 | CAS on WRITE is good, but multi-node atomicity is absent |
| WAL / crash recovery | 1 | 0 | Regressed -- v1 mentioned WAL, v2 does not |
| Constraint enforcement | 0 | 4 | VALIDATE primitive exists but is under-specified |
| Version chain atomicity | 2 | 5 | Commit DAG model for concurrent edits is elegant, CURRENT pointer atomicity still missing |
| Vision completeness | N/A | 6 | 12 primitives cover ~80% of described use cases; critical gaps in multi-node atomicity and collaborative editing |

**V1 composability averaged: 3/10. V2: 6/10.** Substantial progress, but the spec still has critical gaps in engine-level extensibility (storage, indexes), multi-node atomicity, and several "open questions" that are actually load-bearing design decisions masquerading as optional.

---

## Priority Recommendations

### P0 -- Must resolve before implementation begins

1. **Define multi-node transaction semantics.** Either make subgraph evaluation atomic (recommended) or explicitly choose eventual consistency. This affects the credit system, governance, and marketplace -- all core to the vision.

2. **Split GATE into defined components.** Automatic capability enforcement (engine-level), explicit guards (subgraph-level authorization), and custom logic (defined execution model -- subgraph? expression? WASM?).

3. **Commit to an IVM algorithm.** DBSP is the strongest candidate. This is the "key innovation" of the engine -- it cannot remain an open question.

### P1 -- Must resolve before first usable release

4. **Define the TRANSFORM expression language.** JavaScript subset with restrictions, matching @benten/expressions.

5. **Add configurable CRDT merge strategies.** LWW default + text CRDT + counter + set as built-in alternatives.

6. **Add pagination/cursor to READ.** Every content listing needs this.

7. **Define VALIDATE as mandatory-before-WRITE** (engine-enforced, not opt-in). Otherwise constraint enforcement is an illusion.

### P2 -- Must resolve for ecosystem growth

8. **Define a PersistenceBackend trait.** Separate storage from graph logic in benten-graph.

9. **Define an IndexProvider trait.** Support custom index types for vector, full-text, spatial.

10. **Define sync envelope rules.** How dangling references from partial sync are handled.

11. **Add ephemeral state mechanism.** For presence, cursors, and collaborative awareness without polluting version history.
