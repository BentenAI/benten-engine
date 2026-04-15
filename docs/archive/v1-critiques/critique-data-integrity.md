# Data Integrity Critique -- Benten Engine Specification

**Reviewer:** Data Integrity Specialist
**Document:** `docs/SPECIFICATION.md`
**Date:** 2026-04-11
**Score: 3/10**

---

## Executive Summary

The specification describes an ambitious unified graph engine with MVCC, CRDT sync, version chains, and capability enforcement. However, from a data integrity perspective, it is critically under-specified. Nearly every mechanism that protects data consistency -- transactions, conflict resolution, referential integrity, constraint enforcement, crash recovery -- is either absent or described at a level that permits multiple incompatible interpretations. The spec reads as a feature list, not an invariant contract. A team implementing from this document would be forced to make dozens of data-integrity decisions on the fly, many of which would conflict with each other.

The score of 3 reflects that the document identifies the right categories of concern (MVCC, versioning, sync) but provides almost no formal guarantees, invariants, or failure-mode analysis for any of them.

---

## Issue 1: CURRENT Pointer Can Reference Non-Existent or Stale Versions (Version Chain Integrity)

**Severity: Critical**

Section 2.3 defines version chains as: anchor -> CURRENT edge -> latest version, with NEXT_VERSION edges forming the chain. The spec says "Undo = move CURRENT pointer back." But it never specifies:

- **Atomicity of CURRENT pointer updates.** Moving CURRENT from v3 to v4 requires: (a) creating version Node v4, (b) creating NEXT_VERSION edge v3->v4, (c) deleting the old CURRENT edge, (d) creating a new CURRENT edge to v4. If the process crashes between steps (c) and (d), the anchor has NO CURRENT pointer. Every external edge resolving "current state" via anchor->CURRENT->version will dereference into nothing.

- **Deletion of version Nodes.** The spec mentions no garbage collection or compaction strategy for version chains. If version Nodes are ever deleted (for storage reclamation, GDPR, or sync divergence), can CURRENT or NEXT_VERSION point to a deleted Node? What is the recovery procedure?

- **Concurrent CURRENT updates.** Two concurrent transactions both read CURRENT=v3 and attempt to create v4. The spec says "per-Node locking" for writers but does not specify whether the lock covers the entire anchor-plus-version-chain or just individual Nodes. If only the new version Node is locked, both transactions could create competing v4 Nodes, each with a NEXT_VERSION from v3 and a CURRENT edge from anchor, resulting in a forked chain that is not a DAG.

**Recommendation:** Define version chain mutation as an atomic operation on the anchor identity (lock anchor, create version, link chain, move CURRENT, unlock). Specify that CURRENT must always point to a valid version Node (invariant), and define what happens during recovery if this invariant is violated. Specify whether version chains can be pruned and what referential integrity checks apply.

---

## Issue 2: CRDT Sync Cannot Preserve Referential Integrity Across Instances (Orphan Edges)

**Severity: Critical**

Section 2.5 says sync exchanges version Nodes newer than the peer's latest. Section 2.1 says edges point to anchors, not specific versions. But the spec provides no mechanism to ensure that when a subgraph is synced, all referenced anchors exist on the receiving instance.

Consider: Instance A has Node X with an edge to Node Y. The subgraph being synced includes X but not Y (Y is outside the traversal pattern). Instance B receives X and its edges. The edge to Y now points to a non-existent anchor on Instance B. This is a dangling reference -- an orphan edge.

The spec says edges have "add-wins semantics" for conflict resolution, which means an edge received via sync will be retained even if its target does not exist locally. This is the opposite of referential integrity.

Further, the "fork" operation (Section 2.5: "stop syncing, keep all data") creates a snapshot of one side's data. If the fork happens while a multi-Node operation is partially synced, the forked data may contain half of a logically atomic change -- e.g., an order Node without its line items, or a capability grant without its target entity.

**Recommendation:** Define a "sync envelope" that includes transitive closure of referenced anchors for edges in the synced subgraph, or explicitly state that dangling references are allowed and specify how the engine handles edge traversal to non-existent targets (return null? error? lazy-fetch from peer?). Define fork consistency: is a fork guaranteed to be a consistent snapshot, or can it capture partial state?

---

## Issue 3: No Constraint Enforcement Mechanism (Unique, Required, Type)

**Severity: Critical**

The spec defines Nodes as having "properties: typed key-value pairs" and "labels: zero or more type tags." It mentions no mechanism for enforcing:

- **Required properties.** Can a Node with label "ContentType" exist without a "name" property? The spec doesn't say.
- **Unique constraints.** Can two Nodes both have label "User" with property `email = "foo@bar.com"`? The spec mentions no unique index or constraint mechanism.
- **Property type enforcement.** The Value type (Section in CLAUDE.md: `String | i64 | f64 | bool | Vec<Value> | HashMap<String, Value> | null`) is a sum type. Can a property's type change between versions? Can version v1 have `age: i64` and v2 have `age: "forty-two"`?
- **Edge cardinality constraints.** Can an anchor have multiple CURRENT edges? The spec implies exactly one, but provides no enforcement mechanism.
- **Label constraints.** Can a Node's labels change between versions? Can a "User" become a "Product"?

Without constraint enforcement at the engine level, every consumer must implement its own validation -- which is exactly the problem the spec says it solves ("The engine enforces capabilities at the data layer, not the application layer"). Capabilities control authorization; constraints control data shape. The spec conflates the two.

**Recommendation:** Add a constraint system to the specification: unique indexes on (label, property), required-property declarations per label, type stability rules for properties across versions, cardinality constraints on edge types, and label immutability rules. Specify how constraints interact with CRDT sync (what happens when a synced Node violates a local constraint?).

---

## Issue 4: WAL and Crash Recovery Are Unspecified

**Severity: High**

Section 4.1 lists `benten-persist` as handling "WAL, snapshots, disk storage." Section 4.3 exposes `engine.checkpoint()` for "force WAL flush." But the spec provides no details on:

- **WAL write protocol.** Is the WAL write-ahead (written before data modification) or write-behind? For ACID compliance, it must be write-ahead. The spec does not state this.
- **Recovery procedure.** After a crash, what state does the engine recover to? Last checkpoint? Last committed transaction? The spec does not say.
- **Durability guarantees.** Is `engine.checkpoint()` the only way to force durability, or are committed transactions automatically durable? If a transaction commits but the process crashes before the next checkpoint, is the transaction lost?
- **IVM recovery.** The spec says materialized views are "cached." After a crash, are they reconstructed from the WAL, recomputed from scratch, or potentially stale? If recomputed, the O(1) read guarantee is violated during recovery.
- **Version chain recovery.** If a crash occurs mid-version-chain-mutation (see Issue 1), how does the engine detect and repair the inconsistency?
- **WAL corruption.** No mention of checksums, CRC validation, or double-write buffers. If a WAL segment is partially written (torn page), can the engine detect this and recover?

**Recommendation:** Specify the WAL protocol (write-ahead, with fsync guarantees per committed transaction OR per checkpoint with documented durability window). Specify recovery procedure: replay WAL from last consistent checkpoint, rebuild IVM, validate version chain CURRENT pointers. Specify WAL integrity checks (page checksums, torn-write detection).

---

## Issue 5: MVCC Snapshot Isolation vs Serializable -- Write Skew is Not Addressed

**Severity: High**

Section 2.7 claims both "MVCC: readers see a consistent snapshot while writers modify" and "Serializable transactions for atomic multi-operation writes." These are different isolation levels with different guarantees, and the spec conflates them.

MVCC snapshot isolation (as in PostgreSQL's default "READ COMMITTED" or "REPEATABLE READ") permits write skew anomalies. Serializable isolation prevents them but at significant cost. The spec does not clarify:

- **What isolation level do transactions provide?** Snapshot isolation? Serializable? Per-transaction configurable?
- **Write skew scenario.** Transaction T1 reads Nodes A and B, decides to update A based on B's value. Concurrently, T2 reads A and B, decides to update B based on A's value. Both commit successfully under snapshot isolation, but the combined result is inconsistent with any serial execution order. Does the engine prevent this?
- **Per-Node locking scope.** Section 6 says "Per-Node locking" for concurrent writers. If transactions lock individual Nodes, multi-Node invariants (e.g., "the sum of all account balances must be zero") cannot be enforced without explicit range locks or predicate locks. The spec mentions neither.
- **Interaction with IVM.** If a materialized view aggregates data across multiple Nodes, and two concurrent transactions each modify one Node, the view could see a state that never existed in any consistent snapshot. Does the IVM engine observe transaction boundaries?

**Recommendation:** Choose an isolation level and document it clearly. If snapshot isolation: document the write-skew risk and provide a mechanism for applications to opt into stricter isolation (e.g., SELECT FOR UPDATE equivalent). If serializable: document the performance implications and conflict-retry protocol. Specify how IVM views interact with transaction boundaries (are views updated within the transaction or after commit?).

---

## Issue 6: CRDT Conflict Resolution Loses Data Silently (Last-Write-Wins for Properties)

**Severity: High**

Section 2.5 specifies "Node properties: per-field last-write-wins with Hybrid Logical Clocks." This means that when two instances concurrently modify the same property of the same Node, one write is silently discarded. There is no merge, no conflict notification, no audit trail of the discarded value.

For a platform that emphasizes "data is owned by the user" and "instances sync subgraphs bidirectionally," silent data loss during sync is a significant integrity concern:

- A user edits a document title on their phone (offline). Another user edits the same title on desktop. After sync, one title disappears with no indication.
- An admin changes a permission scope on Instance A. A different admin changes the same scope on Instance B. After sync, one admin's change is silently overwritten.

The spec acknowledges "schema validation on receive" for graph structure but provides no validation for property-level conflicts. There is no mechanism for:
- Conflict detection (was this property modified on both sides since last sync?)
- Conflict notification (alert the user that their edit was overridden)
- Conflict resolution strategies beyond LWW (application-level merge, manual resolution, multi-value registers)

**Recommendation:** At minimum, define a conflict log: when LWW discards a value, record the discarded write in the version chain so it can be audited and potentially recovered. Better: support configurable conflict resolution strategies per property or per label (LWW for timestamps, multi-value register for titles, application-merge for rich text). Document which data types are safe under LWW and which require stronger semantics.

---

## Issue 7: IVM Consistency Under Concurrent Writes Is Undefined

**Severity: Medium**

Section 2.2 says "When a write occurs (create/update/delete Node or Edge), the engine identifies which views are affected" and incrementally updates them. But:

- **Atomicity of IVM updates.** If a single transaction modifies 5 Nodes that affect 3 views, are all 3 views updated atomically (they all reflect the transaction, or none do)? Or can a reader see a view that reflects some but not all of the transaction's changes?
- **IVM during long transactions.** If a transaction is open and has modified data but not committed, do materialized views reflect the uncommitted changes? If yes, dirty reads via views. If no, the view is stale for the duration of the transaction.
- **IVM and CRDT sync.** When a batch of Nodes arrives via sync, each one triggers IVM updates. If the batch represents a logically atomic change (e.g., an order with line items), views may briefly reflect an order without line items during the incremental sync processing.
- **IVM failure.** If the IVM update for one view fails (e.g., out of memory, bug in the view definition), does the write fail? Or does the write succeed with a stale view? The spec does not define the failure coupling between data writes and IVM updates.

**Recommendation:** Specify that IVM updates are applied atomically with the committing transaction (views are only updated when transactions commit, and all affected views are updated together). Specify the failure coupling: if IVM update fails, the transaction must fail (or the view must be marked as potentially stale and subject to recomputation). Specify how synced batches interact with IVM (batch-level IVM update, not per-Node).

---

## Issue 8: No Schema Evolution Strategy for Version Chains

**Severity: Medium**

Section 8 lists "How do we handle schema evolution (adding/removing labels, properties) during sync?" as an open question. But this is not just a sync concern -- it is a fundamental data integrity concern for version chains.

If a Content Node at v1 has properties `{title, body}` and the schema evolves to add a required `slug` property, what happens to:

- **Historical versions.** Version v1 does not have `slug`. Is it invalid? Can it still be read? Can the user roll back to v1 (which violates the current schema)?
- **Materialized views.** If a view is defined as "all Content Nodes sorted by slug," historical versions without `slug` are either excluded (data loss in the view) or fail (view broken).
- **Sync with peers.** If Instance A has evolved the schema but Instance B has not, syncing a Node from B to A may produce a Node that violates A's constraints. The spec provides no mechanism for schema version negotiation during sync.

**Recommendation:** Define a schema registry in the graph (schema definitions are themselves versioned Nodes). Each Node version records which schema version it was created under. Reads of historical versions apply the schema of their era, not the current schema. Sync negotiation includes schema version exchange. Required-property additions must include a default value (additive-only migration pattern).

---

## Issue 9: Capability Enforcement Has No Transactional Boundary Definition

**Severity: Medium**

Section 2.4 says capabilities are "Checked at every operation boundary" and "A write that violates capabilities is rejected before it reaches storage." But what constitutes an "operation boundary" within a transaction?

- If a transaction grants a capability to Entity A in step 1, then Entity A performs a write in step 2, is the capability visible within the same transaction? (Read-your-own-writes within the transaction?)
- If a capability is revoked in a concurrent transaction, does an in-flight transaction that was granted the capability before revocation continue to operate? (Snapshot isolation for capabilities?)
- Capabilities are stored as Nodes with GRANTED_TO edges. They participate in MVCC and CRDT sync. Can a capability grant arrive via sync and retroactively authorize operations that were rejected before the sync?

**Recommendation:** Define capability evaluation timing: capabilities are evaluated against the transaction's snapshot (consistent with MVCC). Capability changes within a transaction are visible to subsequent operations in the same transaction. Sync-received capability grants take effect only for future operations (no retroactive authorization).

---

## Issue 10: The "Fork" Operation Provides No Consistency Guarantee

**Severity: Medium**

Section 2.5 defines fork as "stop syncing, keep all data." This implies the forked instance retains a copy of all data it has received up to the fork point. But:

- **No snapshot isolation for fork.** If sync is in progress when fork is triggered, the forked data may include a partial sync batch. The fork does not take a consistent snapshot -- it takes whatever state exists at the moment of the call.
- **No referential integrity check on fork.** The forked data may contain edges pointing to anchors that were not yet synced (see Issue 2). After fork, these become permanent dangling references with no mechanism to resolve them (the peer is no longer connected).
- **No fork metadata.** There is no record of when the fork happened, what the last consistent sync point was, or what data may be missing. If the user later wants to re-establish sync, there is no mechanism to determine what diverged.

**Recommendation:** Define fork as creating a consistent snapshot at the last fully-committed sync point (not mid-sync). Record fork metadata as a Node in the graph (fork timestamp, last sync version, peer identity). Provide a "fork health check" that identifies any dangling references created by the fork.

---

## Summary Table

| # | Issue | Severity | Category |
|---|-------|----------|----------|
| 1 | CURRENT pointer can reference non-existent version | Critical | Version chain integrity |
| 2 | CRDT sync creates orphan edges / fork captures partial state | Critical | Referential integrity |
| 3 | No constraint enforcement (unique, required, type, cardinality) | Critical | Constraint enforcement |
| 4 | WAL and crash recovery unspecified | High | Durability / ACID |
| 5 | MVCC isolation level undefined, write skew unaddressed | High | Transaction isolation |
| 6 | LWW conflict resolution silently loses data | High | Sync integrity |
| 7 | IVM consistency under concurrent writes undefined | Medium | View consistency |
| 8 | No schema evolution strategy for version chains or sync | Medium | Schema integrity |
| 9 | Capability enforcement has no transactional boundary definition | Medium | Authorization consistency |
| 10 | Fork provides no consistency or referential integrity guarantee | Medium | Fork integrity |

---

## Comparison with Existing Thrum Codebase

The current Thrum codebase (V3-4.5) has addressed many of these concerns at the application layer:

- **Transactions:** Composition updates use explicit `BEGIN`/`COMMIT`/`ROLLBACK` with `FOR UPDATE` locks (see `packages/cms/src/db/queries/compositions.ts`). The engine spec must provide at least equivalent protection.
- **Optimistic locking:** Thrum uses `expectedVersion` with row-level version checks. The engine spec mentions per-Node locking but provides no equivalent application-level optimistic concurrency control.
- **Referential integrity:** Thrum uses FK constraints (`CASCADE`/`RESTRICT` on `composition_refs`). The engine spec has no equivalent edge constraint mechanism.
- **Revision safety:** Thrum creates revisions inside the same transaction as the update, preventing phantom revisions. The engine spec's version chain mutation atomicity is undefined.

The engine must not regress on the integrity guarantees the current codebase provides. Currently, the spec does not demonstrate awareness of these existing safeguards.

---

## Actionable Next Steps

1. **Write an invariants document** that lists every data integrity invariant the engine must maintain (e.g., "every anchor has exactly one CURRENT edge," "no edge may reference a non-existent anchor," "committed transactions survive process crash"). The specification should be derived from invariants, not the other way around.

2. **Define failure modes** for every operation (create Node, move CURRENT, sync batch, fork). For each: what can fail, what state is the engine in after failure, how does recovery work.

3. **Choose and document an isolation level.** Do not hand-wave "MVCC" and "serializable" as though they are the same thing. Pick one (or pick configurable) and document the anomalies that are possible.

4. **Design the constraint system** before the capability system. Capabilities control who can do what. Constraints control what is structurally valid. Without constraints, capabilities are guarding a door to a room with no walls.

5. **Define sync consistency boundaries.** A sync batch should be atomic (all-or-nothing). Fork should produce a consistent snapshot. Dangling references must be detected and handled (not silently tolerated).
