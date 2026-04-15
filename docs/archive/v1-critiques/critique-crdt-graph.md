# CRDT Merge for Graph Data -- Deep Critique

**Reviewer:** Correctness & Edge Cases Specialist
**Documents reviewed:** `SPECIFICATION.md` (Section 2.5), `explore-p2p-sync-precedents.md`, `explore-graph-native-protocol.md`, `explore-sync-as-fundamental.md`, `explore-nextgraph-deep-dive.md`, `critique-data-integrity.md`
**Date:** 2026-04-11
**Score: 3/10** (for the CRDT sync section specifically -- not the full spec)

---

## Executive Summary

The specification proposes "per-field last-write-wins with Hybrid Logical Clocks for Node properties" and "add-wins semantics for Edges." This is presented as sufficient for graph CRDT sync. It is not. Graph data has structural invariants -- referential integrity, reachability, schema constraints, capability consistency -- that do not exist in flat key-value stores. The proposed strategy either violates these invariants silently or requires so many additional mechanisms (tombstones, causal stability, transitive closure enforcement, schema negotiation) that the two-sentence description in the spec is roughly 5% of what needs to be specified.

The precedent research documents are excellent at identifying the right problems. But the specification collapses those nuanced findings into a simplistic strategy that the research itself warns against. This critique traces seven specific conflict scenarios through the proposed mechanism, identifies what breaks in each, and then evaluates what existing systems actually do.

---

## Scenario 1: Create-Delete Conflict (The Tombstone Problem)

**Setup:**
- Instance A creates Node X at time T1.
- Instance C syncs with A, receives X.
- Instance A deletes X at time T2.
- Instance C syncs with B (which never saw X), giving B a copy of X.
- Instance A syncs with B.

**Question:** Does B have X or not?

**Analysis under the spec's rules:**

The spec says "Edges: add-wins semantics" but says nothing about Node deletion semantics. There are three possible interpretations:

1. **Tombstone-based deletion:** A's delete creates a tombstone marker. When A syncs with B, the tombstone has a higher HLC than X's creation. B applies the tombstone. X is deleted on B. **Problem:** Tombstones must be retained forever (or until all peers have acknowledged them). If B goes offline for a year, comes back, syncs with C (which still has X because it synced with C before the tombstone propagated), and C never received the tombstone... X resurrects on C's next sync with B. The tombstone must be immortal or have a causal stability protocol that guarantees all peers have seen it before garbage collection.

2. **No tombstones (deletion is local-only):** A deletes X locally. The delete does not propagate. When A syncs with B, B has X and A does not. Under add-wins, A re-acquires X from B. **Problem:** Deletion is impossible in a CRDT without tombstones or a causal stability mechanism. This is a fundamental property, not a design choice.

3. **Version-chain-based deletion:** Following the graph-native protocol doc, deletion is a new version: the anchor Node gets a version that marks it as deleted (soft delete). The CURRENT pointer moves to the "deleted" version. When syncing, the deleted version has a higher sequence number. Peers adopt it. **Problem:** This requires all peers to understand the deletion convention and apply it uniformly. It also means "deleted" Nodes still consume storage (their anchor + all versions + the deleted-version persist). And it breaks under concurrent create-delete: if Instance A deletes Node X and Instance B simultaneously creates Edge Y pointing to X, after sync B has an edge to a "deleted" Node.

**What the spec needs to say:** Which deletion model is used (tombstone, soft-delete-version, or no distributed delete). The garbage collection strategy for tombstones or soft-delete markers. The causal stability condition under which a deletion can be considered final. How deletion interacts with add-wins edge semantics (see Scenario 2).

**What existing systems do:**
- **Automerge:** Uses tombstones internally for deleted elements in sequences and maps. Tombstones are part of the operation history and are never garbage-collected in the current implementation. This is a known scalability concern.
- **Yjs:** Uses a similar tombstone approach (deleted items in the YATA sequence are marked but retained). GC is possible but destroys the ability to undo past the GC point.
- **Holochain:** Deletion is an Action on the source chain. The DHT marks the entry as deleted, but validation authorities may still hold the original data. Deletion propagates via gossip. Agents who received the entry before the delete retain it until they receive the delete action.
- **NextGraph:** Deletion is a CRDT operation (SPARQL DELETE). The SU-Set CRDT handles this via observed-remove semantics: a delete only removes operations that the deleter has observed. Concurrent adds win over deletes the deleter has not seen.

**Verdict:** The spec's silence on deletion is a critical gap. "Per-field LWW" does not address deletion at all because deletion is not a field update -- it is the removal of the entire Node. Add-wins for edges exacerbates the problem by keeping edges to deleted Nodes alive.

---

## Scenario 2: Edge to Deleted Node (Orphan Edge Problem)

**Setup:**
- Instance A creates Edge E from Node P to Node Q.
- Instance B deletes Node Q.
- A and B sync.

**Analysis under the spec's rules:**

The spec says edges have "add-wins semantics." Instance A added Edge E. Instance B did not delete Edge E (it deleted Node Q). Therefore Edge E survives. But its target (Node Q) is deleted.

**The three options and their consequences:**

**(a) Keep the orphan edge.** This violates referential integrity. Any traversal that follows E reaches a non-existent target. The application layer must handle null/missing targets everywhere. Every single `getEdges()` -> `getNode(target)` code path needs a null check. This is the Gun.js approach: no referential integrity, all references are weak.

**(b) Cascade-delete the edge.** When Node Q is deleted, all edges pointing to Q are also deleted. But under add-wins, if Edge E was added concurrently, it should survive. These two rules contradict each other. You cannot have both add-wins and cascade-delete.

**(c) Resurrect Node Q.** The edge implies Q should exist. Re-create Q as a stub/placeholder. But what properties does the resurrected Q have? If B deleted Q because it violated a schema constraint, resurrecting it re-introduces the violation. If B deleted Q intentionally (user action), resurrecting it overrides user intent.

**The deeper problem:** In a non-graph CRDT (e.g., a JSON document), deleting a field does not create dangling references because there are no references -- only nested structures. Graphs are fundamentally different: edges create cross-entity references that introduce referential integrity as a concern. No standard CRDT primitive handles this automatically.

**What existing systems do:**
- **NextGraph (SU-Set CRDT):** Operates on RDF triples, not on separate Nodes and Edges. Deleting a Node means deleting all triples where it appears as subject or object. This is effectively cascade-delete, but implemented at the triple level. The OR-set semantics (add-wins over observed deletes) mean that a concurrently-added triple referencing the deleted node survives if the adder had not observed the deletion. This CAN produce orphan triples, and NextGraph's validation layer must handle them.
- **Holochain:** Links are stored at the base entry's DHT address. Deleting the target entry does not automatically delete links pointing to it. The app's validation rules must handle dangling links (typically by checking target existence during validation and rejecting operations that create links to non-existent targets, or by allowing dangling links and handling them in the UI).
- **Academic (Kleppmann et al., "A Conflict-Free Replicated JSON Datatype," 2017):** JSON CRDTs handle deletion of map keys and array elements using tombstones within the nested structure. They do not address cross-document references because JSON does not have references -- only nesting.

**What the spec needs:** An explicit policy on referential integrity during sync. The choice between (a), (b), and (c) has cascading implications for every graph operation. The spec should define: (1) whether edges can point to non-existent Nodes, (2) what happens when a traversal encounters such an edge, (3) whether deletion cascades to edges, and (4) how this interacts with add-wins semantics.

**My recommendation:** Option (a) with a "pending resolution" queue. Orphan edges are allowed to exist temporarily but are flagged. A background reconciliation process periodically checks for orphan edges and either resolves them (by re-fetching the target from the peer that created the edge) or removes them (after a configurable grace period). This preserves add-wins semantics while providing eventual referential integrity. This is similar to how Git handles "dangling objects" -- they exist temporarily but are cleaned up during GC.

---

## Scenario 3: Version Chain Conflict (Branching Versions)

**Setup:**
- Node X is at version 3 on all instances.
- Instance A edits X, creating version 4A.
- Instance B edits X, creating version 4B (different content, different HLC timestamp).
- A and B sync.

**Analysis under the spec's rules:**

The spec says "per-field last-write-wins with Hybrid Logical Clocks." Two interpretations:

**Interpretation 1: LWW applies to the entire version.** The version with the higher HLC wins. If 4A has HLC=100 and 4B has HLC=101, then 4B becomes the canonical v4 and 4A is discarded. This is data loss. Instance A's edit vanishes.

**Interpretation 2: LWW applies per-field within the version.** If 4A changed the `title` field and 4B changed the `body` field, both changes survive (title from 4A if its HLC is higher for that field, body from 4B if its HLC is higher for that field). If both changed `title`, the higher HLC wins for that field. This preserves more data but produces a version 4 that neither A nor B authored -- a "Frankenstein version" that is a field-level merge of two different edits.

**The version chain problem:** The graph-native protocol doc (Section 3.2, 5.3) describes version chains as a linked list: v1 -> v2 -> v3 -> v4. If both A and B create a v4, the chain forks:

```
v3 -> v4A  (Instance A)
v3 -> v4B  (Instance B)
```

This is no longer a linked list. It is a DAG. The spec's version chain design assumes a linear sequence (`NEXT_VERSION` edges forming a chain). A forked chain means v3 has TWO outgoing `NEXT_VERSION` edges. The CURRENT pointer must choose one. The `currentVersion` denormalized property is now ambiguous -- is it 4A or 4B?

**What existing systems do:**
- **Git:** Embraces branching explicitly. Two commits with the same parent produce a branch. Merge is a separate operation that creates a merge commit. Users must resolve conflicts. Git does NOT auto-merge -- it presents conflicts to the user.
- **Automerge:** Does not use version numbers. Operations are identified by (actorId, sequence) pairs. Concurrent operations are applied in a deterministic order (by actorId tiebreak). There are no "versions" -- just a stream of operations that converge to the same state regardless of application order.
- **NextGraph:** Uses a commit DAG (explicitly a DAG, not a chain). Concurrent commits create branches in the DAG. The CRDT merge produces a single merged state automatically. The DAG preserves the full history of branches and merges.
- **AT Protocol:** Does not allow concurrent edits. Each repository has a single writer. If the `rev` field does not form a linear chain, sync fails.

**What the spec needs:** A clear statement on whether the version chain can branch. If yes: how does the CURRENT pointer resolve (pick one branch? auto-merge?), how is the branched chain stored (DAG edges, not just NEXT_VERSION?), and how does `currentVersion` as a denormalized integer work when there are multiple branch tips. If no: what prevents branching (single-writer constraint? lock? abort on conflict?).

**My recommendation:** Adopt a commit DAG model (like NextGraph and Git), not a linear version chain. Each version carries a vector clock or HLC, and each version points to one or more parent versions (not just one NEXT_VERSION). The CURRENT pointer resolves via the CRDT merge rules (LWW for property conflicts, union for non-conflicting changes). The linear chain is an optimization for the common case (no concurrent edits), not the canonical model.

---

## Scenario 4: Subgraph Boundary Conflict

**Setup:**
- Node Y is inside the sync scope of subgraph S (reachable from root R via edge type T).
- Instance A creates a new edge of type T from R to Y (making Y explicitly in scope).
- Instance B deletes the edge from R to Y (removing Y from scope).
- A and B sync.

**Analysis under the spec's rules:**

Edges have add-wins semantics. Instance A added an edge. Instance B deleted a (different) edge. Under add-wins, A's addition survives. Under edge deletion semantics (unspecified), B's deletion may or may not propagate.

**The subtle problem:** Subgraph boundaries are defined by traversal patterns (Section 2.1 of the spec, Section 4.1 of the graph-native protocol doc). A subgraph is "all Nodes reachable from X via Y edges." Adding or removing edges changes what is reachable. This means subgraph membership is an emergent property of the edge set, not an explicit membership list.

If add-wins means all added edges survive, then subgraph scope can only grow (never shrink) during sync. Once a Node is reachable via any add-wins edge, it stays reachable. Removing a Node from a subgraph requires deleting ALL edges that make it reachable, and add-wins means any concurrent edge addition to that Node undoes the removal.

**This is the "persistent reachability" problem:** In an add-wins graph CRDT, the graph can only grow in connectivity. Nodes become easier to reach over time, never harder. This is by design (preventing accidental disconnection), but it has consequences:

1. **Sync scope creep:** Every sync round potentially makes subgraphs larger (new edges make more Nodes reachable). There is no mechanism to shrink a subgraph short of explicit Node deletion.
2. **Privacy implications:** If Node Y contains sensitive data and Instance B removed it from the sync scope (by deleting the edge), Instance A can re-introduce it by creating a new edge to Y. B cannot prevent this under add-wins.
3. **Performance degradation:** Over time, subgraphs grow monotonically. Sync payloads get larger. Traversal queries return more results.

**What existing systems do:**
- **CRDTs in general (Shapiro et al.):** Add-wins is the standard for set CRDTs (OR-Set, 2P-Set). The observation is that in the absence of global coordination, add-wins is the only strategy that does not lose data. Remove-wins strategies require causal stability to be safe.
- **NextGraph:** Subgraph boundaries are not emergent -- they are explicit (per-store membership). Adding/removing members from a store is a permissioned operation, not a CRDT operation. This sidesteps the emergent boundary problem entirely.
- **Holochain:** DHT neighborhoods are determined by address-space hashing, not by graph reachability. The boundary problem does not arise.

**What the spec needs:** A distinction between structural edges (which form the graph topology and determine reachability) and metadata edges (which carry non-structural information). Add-wins may be appropriate for structural edges (preventing graph disconnection) but inappropriate for scope-boundary edges (which control what is shared with whom). The spec should also address the monotonic growth problem: how are subgraphs pruned or bounded?

---

## Scenario 5: Capability Grant Conflict

**Setup:**
- Instance A grants Capability C to Entity E.
- Instance B revokes Capability C from Entity E.
- A and B sync.

**Analysis under the spec's rules:**

The spec (Section 2.4) says capabilities are Nodes with GRANTED_TO edges. Section 2.5 says edges have add-wins semantics. A grant is an edge (Entity E <- GRANTED_TO <- CapabilityGrant). A revocation is either:

**(a) Deleting the GRANTED_TO edge.** Under add-wins, a concurrent re-grant would re-create the edge. The revocation loses.

**(b) Creating a separate REVOCATION edge/Node.** Under add-wins, both the grant and revocation edges exist. The system must resolve which takes precedence.

**The security problem:** In any capability system, revocation MUST be authoritative. If an attacker or compromised instance re-grants a revoked capability, the system must honor the revocation, not the re-grant. Add-wins semantics are the exact opposite of what capability revocation requires.

The precedent research doc (`explore-p2p-sync-precedents.md`, Section on UCAN) explicitly states: "Revocation is irreversible and propagates to all derived tokens." But the spec's add-wins edge semantics make revocation non-authoritative -- a concurrent add undoes the remove.

**What existing systems do:**
- **UCAN:** Revocation is a signed statement published to a revocation service. It is NOT a CRDT operation. Revocation is checked separately from capability validation. The UCAN system explicitly does not use CRDTs for revocation because CRDTs cannot guarantee that removes win.
- **NextGraph:** Permission changes (adding/removing editors from a store) trigger a key rotation epoch. Revocation is cryptographic -- the revoked party loses the encryption keys and literally cannot produce valid commits anymore. This is not a CRDT merge; it is a hard state transition.
- **Matrix:** Power level changes in rooms use State Resolution v2, which has explicit rules for who can change power levels. Demotions are handled by comparing the power levels of the senders, not by timestamp.

**What the spec needs:** Capabilities and permission grants MUST NOT use add-wins semantics. They need a separate conflict resolution strategy, likely one of:
1. **Remove-wins for revocations** (the inverse of add-wins, applied specifically to capability edges).
2. **Authority-based resolution** (the entity with higher authority wins; e.g., an owner's revocation beats an admin's grant).
3. **Cryptographic revocation** (as in UCAN/NextGraph, where revocation removes the ability to produce valid operations entirely).

The spec's blanket "add-wins for edges" cannot apply uniformly across all edge types. Different edge types have different semantic requirements.

---

## Scenario 6: Move Conflict (Multi-Parent Problem)

**Setup:**
- Node X has edge from Subgraph S1 (X is a child of S1).
- Instance A moves X from S1 to S2 (deletes S1->X edge, creates S2->X edge).
- Instance B moves X from S1 to S3 (deletes S1->X edge, creates S3->X edge).
- A and B sync.

**Analysis under the spec's rules:**

Both instances delete the S1->X edge. Both create a new edge. Under add-wins, both new edges survive: S2->X and S3->X. The deletion of S1->X may or may not propagate (unspecified deletion semantics).

**Result:** Node X is now a child of both S2 and S3. This may or may not be semantically valid depending on the domain:
- For a file system (where a file is in exactly one directory): this is invalid. X cannot be in two directories.
- For a tagging system (where an item can have multiple tags): this is fine.
- For a CMS page tree (where a page has one parent): this creates a confusing duplicate.

**The "move" operation is not atomic in CRDTs.** A move is composed of two sub-operations: a delete (remove old edge) and a create (add new edge). CRDTs process these independently. The delete may lose (under add-wins), the create always wins. This means "move" operations can result in:
- Node in both old and new locations (add-wins keeps old edge, new edge also created)
- Node in multiple new locations (two concurrent moves both create new edges)
- Node orphaned (if remove-wins is used for structural edges, both deletes win, and neither add propagates before the other's delete arrives)

**What existing systems do:**
- **Automerge (move operations, 2023):** Automerge introduced explicit move semantics in their data model. A move operation is a first-class CRDT operation (not decomposed into delete + create). Concurrent moves are resolved by LWW: the move with the higher timestamp wins, and the other is undone. This requires the CRDT to understand "move" as an atomic operation.
- **Tree CRDTs (Kleppmann et al., "A Highly-Available Move Operation for Replicated Trees," 2021):** This paper from Kleppmann's group at Cambridge addresses exactly this problem. The key insight: tree move operations must be treated atomically, and a cycle-detection mechanism must run at merge time. Their algorithm ensures that concurrent moves to the same node are resolved without creating cycles or duplicates. The paper demonstrates that decomposing moves into delete+create is fundamentally broken for tree-structured data.
- **CRDTree (INRIA, 2023):** An extension of tree CRDTs to arbitrary DAGs. Uses a "common ancestor linearization" strategy to resolve concurrent moves without creating invalid topologies.

**What the spec needs:** An explicit decision on whether the graph supports a "move" primitive or only "add edge" and "delete edge." If the graph must support tree-like structures (parent-child relationships where a node has exactly one parent), move MUST be a first-class CRDT operation, not decomposed into delete+create. The spec should reference the tree CRDT literature and specify how concurrent moves are resolved.

---

## Scenario 7: Schema Conflict

**Setup:**
- Instance A modifies ContentType "user" to add a required field "email."
- Instance B creates a User Node without "email."
- A and B sync.

**Analysis under the spec's rules:**

The spec says "Graph structure: schema validation on receive." This implies that B's User Node (without email) is validated against A's schema (which requires email) and rejected.

**Problems:**

1. **Causal ordering violation.** B created the User Node before A's schema change. From B's perspective, the Node was valid at creation time. Rejecting it retroactively violates B's causal history. It is like Git refusing a merge because a commit from 2 weeks ago does not pass today's linting rules.

2. **Schema propagation timing.** When A adds the "email" field, when does B learn about it? If schema changes propagate via the same CRDT sync mechanism as data, there is a race: B might receive A's new User Node (which has email) before receiving the schema change, or vice versa. The order of arrival determines whether the validation passes.

3. **Bidirectional sync deadlock.** A syncs schema to B. B now requires "email" on all Users. But B already has Users without email (created before the schema change). B's own data now violates its own schema. Does B reject its own existing data? Auto-migrate it? Leave it as a validation error?

4. **Schema as a CRDT.** If schemas are themselves graph Nodes that sync via CRDTs, then two instances can concurrently modify the same schema. Instance A adds "email" (required). Instance B adds "phone" (required). After sync, the schema requires both. Any existing Node that has neither now violates the merged schema. The schema merger creates retroactive invalidity.

**What existing systems do:**
- **Holochain (DNA validation):** Schemas (DNA) are immutable per app. If two instances have different schemas, they are running different apps and cannot sync. Schema changes require deploying a new DNA version. This completely avoids runtime schema conflicts at the cost of flexibility.
- **AT Protocol (Lexicon):** Schemas are versioned and immutable. A record is validated against the schema version specified in its collection name. Adding a required field means creating a new collection version. Old records under the old schema version remain valid.
- **Automerge:** Has no schema layer. All operations are valid at the CRDT level. Schema validation, if any, happens at the application layer after CRDT merge. This means invalid data can exist in the CRDT; the app decides what to do with it.
- **NextGraph (ShEx):** ShEx shapes are stored as RDF. Schema changes are CRDT operations like any other data change. Validation is performed by the verifier after merge. Invalid data is flagged but not automatically rejected -- the application decides how to handle it.

**What the spec needs:** A schema evolution strategy for distributed sync. My recommendation:

1. **Schemas are immutable and versioned.** A schema change creates a new version. Old Nodes are valid against their creation-time schema.
2. **Additive-only schema changes.** New fields must be optional or have defaults. Required field additions require a new schema version, not a mutation of the existing one.
3. **Schema version is part of the Node metadata.** Each Node records which schema version it was created under. Validation uses the Node's schema version, not the current global schema.
4. **Schema incompatibility halts sync for that content type.** If two instances have incompatible schemas (conflicting required fields), sync for that content type is paused and flagged for human/AI resolution. Other content types continue syncing.

---

## The Fundamental Question: Is "Per-Field LWW + Add-Wins Edges" Sufficient?

**No.** It is not sufficient for a graph database. Here is why, summarized from the seven scenarios:

| Concern | LWW + Add-Wins Handling | What Is Actually Needed |
|---------|-------------------------|------------------------|
| Node deletion | Not addressed | Tombstones + causal stability + GC strategy |
| Orphan edges | Created by design (add-wins keeps edges to deleted nodes) | Explicit referential integrity policy |
| Version chain branching | Creates invalid linear chain | Commit DAG (non-linear version history) |
| Subgraph boundary changes | Monotonic growth only (add-wins = never shrink) | Edge-type-specific conflict resolution |
| Capability revocation | Revocation loses under add-wins | Remove-wins or authority-based resolution for security edges |
| Move operations | Decomposed into delete+create = duplication | First-class move CRDT operation |
| Schema conflicts | "Validate on receive" = reject valid historical data | Schema versioning + additive-only evolution |

**The minimum complexity that guarantees convergence without data loss requires:**

1. **Per-edge-type conflict resolution policies.** Not all edges are equal. Structural edges (parent-child) need different rules than metadata edges (tags) which need different rules than security edges (capability grants). A single "add-wins" policy is insufficient.

2. **Tombstones with causal stability.** Deletion must be expressible and must propagate. The system must know when all peers have observed a deletion before garbage-collecting the tombstone.

3. **A version DAG, not a version chain.** Concurrent edits create branches. The merge algorithm must handle DAG structures, not assume linear chains.

4. **Move as an atomic CRDT operation.** For any tree-like structure in the graph (page hierarchy, directory tree, org chart), decomposing moves into delete+create produces invalid states.

5. **Schema-versioned validation.** Validation must be applied against the schema that was current when the data was created, not the schema that is current when the data is received.

6. **A separate conflict resolution track for security-critical edges.** Capability grants and revocations cannot use the same merge strategy as content edges.

---

## Research: What Existing Systems Actually Do

### Automerge and Graph/Tree CRDTs

Automerge is designed for JSON documents (nested maps, arrays, text). It does NOT natively support graph structures. References between Automerge documents are just strings (UUIDs stored as property values). There is no referential integrity, no edge semantics, no traversal.

For tree-structured data within a single document, Automerge handles move operations (since v2.0) as atomic operations, using a deterministic conflict resolution: concurrent moves to the same node are resolved by actorId comparison. But this is for trees within a single document, not for a graph database with arbitrary edge topologies.

**Key paper:** Martin Kleppmann, Dominic P. Mulligan, Victor B.F. Gomes, and Alastair R. Beresford. "A Highly-Available Move Operation for Replicated Trees." IEEE Transactions on Parallel and Distributed Systems, 2021. This paper proves that tree move operations require special handling in CRDTs and provides an algorithm that maintains tree invariants (no cycles, single parent) during concurrent moves.

### Yjs and Graph-Like Structures

Yjs operates on sequences (text), maps, and XML fragments. Like Automerge, it does not natively support graph references. Cross-document references are application-level strings. Yjs's YATA algorithm ensures convergence for concurrent insertions in sequences, but graph topology is not in scope.

For Thrum's use case (a graph database with typed nodes and edges), neither Automerge nor Yjs provides a ready-made solution. They would need to be used as building blocks for specific aspects of the system:
- Yjs for rich text fields within Nodes (collaborative editing of `config.body`)
- Automerge for structured JSON properties within Nodes (concurrent edits to `config`)
- A custom graph CRDT for the topology (Nodes, Edges, traversal patterns)

### The "JSON CRDT" Problem

JSON CRDTs (Automerge, Yjs maps) handle nested/reference data by treating each nested level as its own CRDT:
- A map is an LWW-Register per key (or a multi-value register if you want to preserve concurrent writes)
- An array is an RGA (Replicated Growable Array) that handles concurrent inserts deterministically
- Nesting composes: a map containing an array containing maps is three layers of CRDTs

The limitation: JSON CRDTs handle nesting but not references. A JSON document has no equivalent of a foreign key. Two JSON documents can refer to each other by embedding an ID string, but the CRDT has no knowledge of this reference and cannot maintain its integrity.

**For Benten Engine specifically:** The Node's `config` field is a JSON blob. It could use a JSON CRDT (Automerge) for per-field concurrent edits within `config`. But the Node's identity, its edges, and its position in the graph are structural concerns that JSON CRDTs do not address.

### Academic Work on Graph CRDTs

**Kleppmann's group at Cambridge (2021-2024):**

The most relevant academic work comes from Kleppmann's local-first group:

1. **"A Conflict-Free Replicated JSON Datatype" (2017):** Introduces a formal model for JSON CRDTs. Does not address graph references.

2. **"A Highly-Available Move Operation for Replicated Trees" (2021):** Addresses the move-conflict problem for trees. Key insight: moves must be first-class CRDT operations, not decomposed into insert+delete. Provides a provably correct algorithm.

3. **"Making CRDTs Byzantine Fault Tolerant" (2022):** Addresses the problem of malicious peers in CRDT networks. Relevant because add-wins semantics allow any peer to add edges (including malicious ones). The paper proposes validation rules that run at merge time.

4. **"Local-First Software: You Own Your Data, in spite of the Cloud" (2019):** The foundational paper on local-first architecture. Identifies graph CRDTs as an open research problem.

**Key finding from the literature:** There is NO general-purpose graph CRDT that handles arbitrary topologies with referential integrity, move operations, and schema constraints. The state of the art is:
- Tree CRDTs (solved for trees, not general graphs)
- Set CRDTs (OR-Set for vertices and edges, but no referential integrity)
- JSON CRDTs (for nested data within a vertex, but no cross-vertex references)
- Custom graph CRDTs (NextGraph's SU-Set for RDF triples, but RDF triples are simpler than property graph edges)

### Holochain's Validation Rules System

Holochain prevents invalid states after merge by requiring every operation to pass validation BEFORE it is accepted:

1. Every agent has a source chain (append-only log of their operations).
2. When an agent creates an entry or link, they include a proof that it is valid (the entry conforms to the DNA's validation rules).
3. Validation authorities (other agents in the DHT neighborhood) independently validate the proof.
4. If validation fails, the operation is rejected and a warrant is issued against the agent.

**How this prevents invalid states:**
- An agent cannot create a User without email if the DNA requires email, because validation would fail.
- An agent cannot create a link to a non-existent entry, because validation checks target existence.
- An agent cannot modify an entry they do not own, because validation checks authorship.

**The key insight for Benten Engine:** Validation happens BEFORE merge, not after. Invalid operations are never accepted into the shared state. This is fundamentally different from "validate on receive after CRDT merge" because it prevents invalid states from ever existing, rather than trying to clean them up after the fact.

**The tradeoff:** Pre-merge validation requires that validators have access to the full context needed for validation (the current graph state, the schema, the authorship chain). In a P2P system, this means validators must have a copy of the relevant data, which introduces latency and bandwidth requirements. Holochain's gossip protocol handles this but with 1-minute propagation latency (Kitsune2).

---

## Recommendations

### 1. Replace "Add-Wins for All Edges" with Per-Type Edge Policies

Define three edge conflict resolution categories:

| Category | Policy | Example Edge Types |
|----------|--------|--------------------|
| Structural | Add-wins (prevent graph disconnection) | `CONTAINS`, `CHILD_OF`, `HAS_VERSION` |
| Referential | Add-wins with orphan detection + lazy resolution | `REFERENCES`, `LINKS_TO`, `USES` |
| Security | Remove-wins (revocation is authoritative) | `GRANTED_TO`, `CAPABILITY`, `DENIED` |
| Exclusive | Move-wins (atomic move operation, LWW tiebreak) | `PARENT_OF` (tree edges where exactly one parent is required) |

### 2. Adopt a Commit DAG, Not a Version Chain

Replace the linear version chain (anchor -> v1 -> v2 -> v3) with a commit DAG (anchor -> v1 -> v2a / v2b -> v3-merged). Each version carries a vector clock. Concurrent versions create branches. Auto-merge applies field-level LWW within the version snapshot. The merged version becomes a new commit with two parents.

This is what NextGraph does and what the graph-native protocol doc hints at (Section 3.2 mentions "Option A: both v4 and v4' are created"). Make it explicit and formal.

### 3. Define Deletion as a First-Class CRDT Operation

Choose between:
- **Soft-delete versions** (a new version with `deleted: true` -- simpler, stores overhead)
- **Tombstone markers** (a lightweight deletion record separate from the version chain -- more efficient, more complex)

Either way: specify the causal stability condition for garbage collection, specify how tombstones interact with add-wins edges, and specify the maximum tombstone lifetime.

### 4. Implement Schema-Versioned Validation

Every Node carries a `schemaVersion` in its metadata. Validation on receive uses the Node's creation-time schema, not the current schema. Schema changes are additive-only (new required fields must have defaults). Incompatible schema changes create a sync boundary -- content types with incompatible schemas on two instances do not sync until resolved.

### 5. Pre-Merge Validation, Not Post-Merge Cleanup

Follow Holochain's approach: validate incoming operations BEFORE applying them to the local graph. An operation that would create an orphan edge, violate a schema constraint, or exceed a capability grant is rejected before it enters the local state. This prevents the "invalid state exists temporarily" problem that post-merge validation creates.

### 6. Move as a First-Class CRDT Operation

For any graph structure that requires tree invariants (page hierarchy, org chart, file system), implement move as an atomic CRDT operation per Kleppmann et al. (2021). Do not decompose moves into delete+create.

### 7. Add a Conflict Log

When LWW discards a value, record the discarded value in a conflict log (as the data integrity critique also recommends). The conflict log enables:
- Audit ("what was lost during sync?")
- Recovery ("restore the discarded value")
- Policy tuning ("is LWW too aggressive for this content type?")

### 8. Consider Separate CRDT Strategies per Data Tier

Following NextGraph's three-CRDT model:

| Data Tier | CRDT Strategy | Rationale |
|-----------|---------------|-----------|
| Node properties (`config` JSON) | Automerge (JSON CRDT) | Per-field merge within structured data |
| Rich text fields within config | Yjs (YATA) | Character-level merge for collaborative editing |
| Graph topology (Nodes + Edges) | Custom graph CRDT (OR-Set + per-type policies) | Referential integrity, move operations, security edges |
| Capability/security edges | Authority-based (not CRDT) | Revocation must be authoritative |

This is more complex than a single "LWW + add-wins" but it is the minimum complexity that does not silently lose data or violate structural invariants.

---

## Summary

The spec's CRDT strategy is a sentence when it needs to be a chapter. "Per-field LWW with HLC for Node properties and add-wins for Edges" is a reasonable default for a flat key-value store. For a graph database with version chains, capability enforcement, schema constraints, referential integrity, and tree-like hierarchies, it is insufficient. The seven scenarios traced above demonstrate concrete data loss, security violations, and structural corruption that the proposed strategy produces.

The good news: the precedent research (particularly `explore-p2p-sync-precedents.md` and `explore-nextgraph-deep-dive.md`) already identifies most of these problems. The recommendations in those docs are sound. The specification just needs to incorporate them rather than collapsing them into a two-sentence summary.

The honest answer to "what is the minimum complexity that guarantees convergence without data loss for graph data?" is: **per-edge-type conflict policies + commit DAG + tombstones with causal stability + atomic move operations + schema versioning + authority-based security resolution + a conflict log.** That is significantly more than LWW + add-wins. It is also significantly less than what NextGraph or Holochain implement, because Benten can make simplifying assumptions (server-authoritative for most operations, P2P sync as an opt-in layer, not a default).

---

## Sources

- Kleppmann, M., Mulligan, D.P., Gomes, V.B.F., Beresford, A.R. "A Highly-Available Move Operation for Replicated Trees." IEEE TPDS, 2021.
- Kleppmann, M., Beresford, A.R. "A Conflict-Free Replicated JSON Datatype." IEEE TPDS, 2017.
- Kleppmann, M. et al. "Making CRDTs Byzantine Fault Tolerant." PaPoC Workshop, 2022.
- Kleppmann, M. et al. "Local-First Software: You Own Your Data, in spite of the Cloud." Onward!, 2019.
- Shapiro, M. et al. "A comprehensive study of Convergent and Commutative Replicated Data Types." INRIA, 2011.
- Bieniusa, A. et al. "An Optimized Conflict-free Replicated Set." arXiv:1210.3368, 2012. (OR-Set formalization)
- Baquero, C. et al. "The Problem with Embedded CRDT Counters and a Solution." PaPoC Workshop, 2016.
- NextGraph SU-Set: NextGraph documentation on CRDT operations for SPARQL updates.
- Automerge move operations: https://automerge.org/docs/under-the-hood/move/
- Yjs internals: https://github.com/yjs/yjs/blob/main/README.md
- Holochain validation: https://developer.holochain.org/build/validation/
- UCAN revocation: https://ucan.xyz/revocation/
- All sources from `explore-p2p-sync-precedents.md` and `explore-nextgraph-deep-dive.md`
