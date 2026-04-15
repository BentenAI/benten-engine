# Security Architecture Critique: Benten Engine Specification

**Reviewer:** Security & Trust Auditor
**Date:** 2026-04-11
**Scope:** `docs/SPECIFICATION.md`, `CLAUDE.md`, and supporting exploration documents (`explore-capability-system-design.md`, `explore-p2p-sync-precedents.md`, `explore-nextgraph-deep-dive.md`, `explore-wasm-sandbox-2026.md`)
**Verdict:** The specification demonstrates strong security instincts in several areas (fail-closed capabilities, engine-level enforcement, UCAN awareness) but has significant underspecification in the exact areas where security matters most: the boundaries between trusted and untrusted execution, the revocation propagation problem in P2P, Cypher injection surfaces, CRDT conflict weaponization, and memory safety assumptions in the IVM hot path.

---

## Security Score: 5 / 10

**Rationale:** The score reflects the gap between *intent* and *specification*. The security intent is consistently good (capability enforcement at the engine level, fail-closed defaults, UCAN-compatible structure). But the specification is a pre-development document that leaves the hardest security questions as open questions or defers them entirely. A 5 means: "the foundation is sound, but a naive implementation from this spec alone would ship exploitable vulnerabilities."

Breakdown:
- Capability model design: 7/10 (well-thought-out, typed, attenuable)
- Sync trust model: 3/10 (underspecified; the hard problems are deferred)
- Injection surface management: 2/10 (Cypher exposure acknowledged but unaddressed)
- Memory safety / resource exhaustion: 4/10 (Rust helps, but IVM and CRDT merge are unbounded)
- Sandboxing: 6/10 (good tech selection, but engine-sandbox boundary is unspecified)
- Cryptographic foundations: 4/10 (UCAN mentioned, no key management spec)

---

## Critical Vulnerabilities

### CRITICAL-1: Cypher Query Injection via `engine.query()`

**Location:** Specification Section 4.3 API Surface

```typescript
engine.query(cypher: string, params?: Record<string, Value>): QueryResult
```

The API accepts raw Cypher strings. The `params` parameter suggests parameterized queries are supported, but nowhere does the specification mandate their use or describe how the query parser rejects injection attempts when params are not used. The existing Thrum codebase already has Cypher injection protections (`escCypher`, `escLabel` in `@benten/auth`), but those are application-level mitigations. The engine specification does not describe:

1. Whether `engine.query()` accepts only parameterized Cypher (safe) or also string-interpolated Cypher (unsafe).
2. Whether the query parser rejects queries that attempt to modify the capability graph or version chain metadata.
3. Whether materialized view definitions (`engine.createView(name, query)`) are validated to prevent view definitions that exfiltrate data across capability boundaries.

**Attack vector:** A module with `raw:cypher` capability (or a compromised verified module) constructs a Cypher query that reads or modifies CapabilityGrant nodes, effectively escalating its own privileges. Since capabilities are "first-class Nodes in the graph" (Section 2.4), the same query language that reads user data can also read and potentially mutate the capability graph.

**Severity:** Critical. This is the engine's equivalent of SQL injection against the `pg_roles` table.

**Recommendation:** The specification must define:
- A mandatory query validation layer that rejects mutations to system-labeled nodes (CapabilityGrant, VersionChain, Anchor, TraversalPattern) from non-platform contexts.
- Whether `engine.query()` ONLY accepts parameterized queries (params map), or also raw string queries. If raw strings are supported, the parser must reject string concatenation patterns and enforce parameterization.
- A system-node label namespace (e.g., `__benten:*`) that is immutable via Cypher and only writable through dedicated engine APIs.

---

### CRITICAL-2: Capability Graph Is Queryable and Mutable via the Same Primitives as User Data

**Location:** Specification Section 2.4

> Capabilities are first-class Nodes in the graph: CapabilityGrant Node, GRANTED_TO edge.

The specification stores the authorization model (who can do what) in the same graph as the data being protected (user content, module data). This is architecturally elegant but creates a self-referential security problem: the enforcement mechanism is stored in the same substrate it is supposed to protect.

**Attack vectors:**
1. **Privilege escalation via graph mutation:** If any operation path allows writing to CapabilityGrant nodes without going through `engine.grantCapability()`, an attacker can forge capability grants.
2. **Capability exfiltration via traversal:** A module with graph traversal rights (even read-only) can traverse to CapabilityGrant nodes for other modules, learning exactly what permissions they have. This is information disclosure that enables targeted attacks.
3. **IVM poisoning:** If a materialized view is defined over capability nodes, and the IVM update path for that view has a race condition with a concurrent capability revocation, the view may serve stale (pre-revocation) capability data.

**Severity:** Critical. The specification acknowledges this architecture but does not describe the access control boundary between data nodes and system nodes.

**Recommendation:** Define an explicit two-zone model:
- **System zone:** CapabilityGrant, Anchor, VersionMeta, TraversalPattern, and IVM metadata nodes. Only writable through dedicated engine APIs. Not traversable from user-zone queries unless explicitly projected.
- **User zone:** All other nodes. Subject to capability checks.
- The Cypher parser/planner must enforce zone boundaries at the query planning stage, not at runtime (defense in depth against race conditions).

---

### CRITICAL-3: CRDT Sync Conflict Resolution Is Weaponizable

**Location:** Specification Section 2.5, P2P Sync Precedents document

The specification describes:
- Node properties: per-field last-write-wins with Hybrid Logical Clocks
- Edges: add-wins semantics
- Graph structure: schema validation on receive

**Attack vector: Malicious peer exploiting LWW with clock manipulation**

Hybrid Logical Clocks (HLC) combine physical time with logical counters. A malicious peer can:
1. Set its physical clock far into the future (e.g., year 2099).
2. Generate HLC timestamps that will always win any LWW comparison.
3. Overwrite any property on any synced node, because its timestamp will be "newer" than any legitimate write.
4. The receiving instance has no way to reject this without additional validation, because the HLC values are self-attested.

This is the "monotonic clock advantage" attack documented in CRDT research. It applies to any LWW scheme that uses timestamps from untrusted peers.

**Attack vector: Edge spam via add-wins semantics**

Add-wins means deletes are always trumped by concurrent adds. A malicious peer can:
1. Create thousands of edges in the shared subgraph.
2. The legitimate peer deletes them.
3. The malicious peer re-adds them during the same sync window.
4. Add-wins ensures the edges persist, because concurrent add > concurrent delete.
5. This fills the legitimate peer's graph with garbage edges that cannot be permanently removed as long as the malicious peer is syncing.

**Severity:** Critical for any deployment with P2P sync enabled. These are fundamental protocol-level attacks, not implementation bugs.

**Recommendation:**
- **Clock validation:** Receiving peers must reject HLC timestamps that are more than a configurable delta (e.g., 5 minutes) ahead of their own clock. This is standard practice in distributed systems (NTP uses a similar bound).
- **Rate limiting per peer:** The sync protocol must enforce per-peer write rate limits. A peer that generates more than N operations per time window is throttled or disconnected.
- **Tombstone-wins mode for edges:** The specification should allow per-subgraph configuration of conflict resolution strategy. For shared subgraphs where data integrity matters more than availability, "remove-wins" or "tombstone-wins" should be an option alongside "add-wins."
- **Peer reputation:** Consider a lightweight trust score per peer. Peers whose operations are frequently rejected or reverted accumulate negative reputation. Low-reputation peers are deprioritized or disconnected.

---

### CRITICAL-4: UCAN Revocation Does Not Propagate in a Partition-Tolerant P2P Network

**Location:** Capability System Design Section 9, P2P Sync Precedents

The specification describes revocation as "synchronous for new operations, eventual for in-flight operations" and references cascade revocation via `revokeGrantCascade()`. This works for local module capabilities but breaks down in P2P:

1. Instance A revokes Instance B's sync capability.
2. Instance A updates its local capability store immediately.
3. Instance C, which received a delegated (attenuated) grant from B, has no way to learn about the revocation unless A broadcasts it.
4. If A and C are not directly connected (they communicate via B), the revocation message cannot reach C.
5. C continues to operate with a capability derived from a revoked grant.

This is the well-known **revocation propagation problem** in capability-based systems. UCAN's revocation specification (https://ucan.xyz/revocation/) describes a revocation record that must be checked by validators, but the specification does not describe:
- How revocation records are distributed across the P2P network.
- What happens when a peer is offline during revocation and comes back online with a stale grant.
- Whether revocation records are themselves synced as graph nodes (creating a dependency: the revocation mechanism depends on the sync mechanism that is subject to revocation).

**Severity:** Critical for P2P deployments. An uninstalled or banned module's capabilities persist on disconnected peers indefinitely.

**Recommendation:**
- **Short-lived grants with renewal:** Instead of long-lived grants that require explicit revocation, use short-lived grants (e.g., 1-hour TTL) that require periodic renewal. Revocation becomes "stop renewing" rather than "broadcast revocation." This bounds the window of stale capability to the TTL.
- **Revocation as a first-class sync primitive:** Revocation records must be prioritized in the sync protocol (delivered before data operations). The engine specification should define revocation sync as a separate, higher-priority channel.
- **Offline revocation buffer:** When a peer comes online, the first sync action must be a revocation check before any data operations are processed.

---

### CRITICAL-5: IVM Resource Exhaustion -- Materialized Views as Denial-of-Service Vector

**Location:** Specification Section 2.2, API Surface Section 4.3

```typescript
engine.createView(name: string, query: string): ViewId
```

Any entity with the capability to create materialized views can define a view over an expensive traversal pattern. The IVM system then maintains this view incrementally on every write. If the view definition involves a cartesian product or unbounded traversal, every single write to the graph triggers an expensive IVM update.

**Attack vector:**
1. A module with view-creation capability defines: `MATCH (a)-[*1..100]->(b) RETURN count(*)` (unbounded traversal depth).
2. Every node/edge creation triggers the IVM to recompute how this view is affected.
3. Write throughput drops to near-zero because the IVM maintenance cost dominates.
4. This is a denial-of-service attack via resource exhaustion, not a data integrity attack.

The specification's performance targets (<0.01ms for IVM reads) assume well-behaved view definitions. There is no discussion of:
- Maximum traversal depth in view definitions.
- Maximum number of materialized views.
- Cost estimation or query planning for view maintenance operations.
- Circuit breakers for views whose maintenance cost exceeds a threshold.

**Severity:** High. Any module with view-creation capability can degrade the entire engine's performance.

**Recommendation:**
- **View cost estimation:** Before creating a view, the query planner must estimate the maintenance cost (how many nodes/edges could trigger an update, and how expensive each update is). Reject views above a cost threshold.
- **View definition restrictions:** Limit traversal depth in view definitions (e.g., max 5 hops). Reject cartesian products and unbounded patterns.
- **Per-view resource budget:** Each materialized view gets a CPU/memory budget for its incremental update. If a single write triggers an update that exceeds the budget, the view is marked as "stale" and recomputed asynchronously rather than blocking the write path.
- **Capability-gated view creation:** View creation should be a separate capability (`ivm:createView`) that is NOT included in the community preset.

---

## Additional Vulnerabilities (HIGH Severity)

### HIGH-1: No Specification of Sandbox-Engine Boundary for WASM Code Execution

The specification states (Section 5): "Code sandboxing -- @sebastianwessel/quickjs handles this (via engine API)." The exploration document recommends QuickJS-in-WASM with capability-gated host functions. But the specification does not describe:

- Which engine APIs are exposed as host functions to sandboxed code.
- How the sandbox's capability grant maps to a specific set of host functions.
- Whether sandboxed code can create nodes/edges directly or must go through a mediated API.
- What happens if sandboxed code triggers an IVM update that affects the sandbox's own view (re-entrancy).

**Risk:** Without an explicit boundary specification, the first implementation will either expose too much (sandbox escape via engine API) or too little (sandboxed plugins are useless). The host function surface is the attack surface.

**Recommendation:** Add a section to the specification that explicitly lists:
1. The complete set of engine operations available to sandboxed code.
2. Which operations are mediated (checked against the sandbox's attenuated capability grant) vs. direct.
3. Re-entrancy rules (can a sandbox trigger IVM that triggers a reactive notification that triggers another sandbox execution?).

### HIGH-2: Version Chain Manipulation via CURRENT Pointer

The specification states: "Undo = move CURRENT pointer back." The CURRENT pointer determines which version of an entity is considered authoritative. If a module (or synced peer) can manipulate the CURRENT pointer on another entity's anchor node, it can effectively revert that entity to a previous state without the owner's consent.

**Attack vector in P2P:** Peer A has v1->v2->v3 of a content node. Peer B syncs an operation that moves the CURRENT pointer from v3 back to v1, effectively "undoing" two versions of edits. Under LWW semantics, if B's operation has a higher HLC timestamp, it wins.

**Recommendation:** CURRENT pointer mutations should be a distinct capability (`version:rollback`) that is NOT granted by default in sync contexts. Rollback operations on synced entities must require explicit consent from all sync group members or be rejected.

### HIGH-3: Reactive Subscription Information Leakage

Section 2.6 describes: "Subscribe to a query pattern: get notified when the result set changes."

If subscription management does not enforce capability boundaries, a module can subscribe to a pattern that matches nodes outside its capability scope and learn about their existence, creation, and modification -- even if it cannot read the node contents. The notification that "a new node matching pattern X was created" leaks the node's existence.

**Recommendation:** Subscription creation must be validated against the subscriber's capabilities. A module with `store:read` scoped to `commerce_*` tables should not be able to subscribe to changes on `user_*` tables. The reactive subscription system must filter notifications through the subscriber's capability scope.

### HIGH-4: No Specification of Cryptographic Primitives for P2P Identity

The specification mentions "UCAN-compatible structure" and "signed token" for P2P sync but does not specify:
- What signing algorithm (Ed25519? secp256k1?).
- What DID method (did:key? did:web? did:plc?).
- What serialization format for capability tokens (JWT? CBOR? DAG-CBOR?).
- What key storage mechanism for instance identity keys.
- What happens if a signing key is compromised (key rotation protocol).

Without these decisions, two independent implementations of the specification would produce incompatible P2P protocols. More critically, a poor choice (e.g., RSA-1024, unpadded ECDSA) would undermine the entire trust model.

**Recommendation:** The specification should either:
1. Commit to specific cryptographic primitives (e.g., Ed25519 for signing, X25519 for key agreement, BLAKE3 for hashing, DAG-CBOR for serialization), or
2. Explicitly define a cryptographic agility mechanism with a minimum security level floor.

### HIGH-5: Transaction Isolation and Capability Check TOCTOU

Section 4.3 defines both transactions and capability checks. The specification does not address time-of-check-time-of-use (TOCTOU) for capability enforcement within transactions:

1. Transaction begins. Module has `store:write` capability.
2. Module performs 3 write operations within the transaction.
3. Between operation 2 and operation 3, the operator revokes the module's `store:write` capability.
4. Does operation 3 succeed (because the transaction started when the capability was valid) or fail (because the capability was revoked mid-transaction)?

The specification says revocation is "synchronous for new operations, eventual for in-flight operations." But it does not define whether a transaction is one "operation" (all-or-nothing at commit time) or multiple operations (each checked independently).

**Recommendation:** Define the transaction-capability interaction explicitly. The recommended approach: capabilities are checked at transaction commit time, not at individual operation time. This ensures atomicity (the entire transaction reflects the capability state at commit) and avoids the TOCTOU window.

---

## OWASP Top 10:2025 Checklist

| # | Category | Status | Notes |
|---|----------|--------|-------|
| A01 | Broken Access Control | PARTIAL | Capability model is well-designed for modules. No specification for user-level access control at the engine layer (deferred to TypeScript layer). The data-zone/system-zone boundary for capability nodes is undefined (CRITICAL-2). |
| A02 | Security Misconfiguration | PARTIAL | The capability system defaults to fail-closed (no capability = no access). But the "Open Questions" section (Section 8) leaves multiple security-relevant decisions unresolved: Cypher vs. Rust-native API, SQL support, schema evolution during sync. Each unresolved question is a misconfiguration surface. |
| A03 | Software Supply Chain Failures | NOT ADDRESSED | The Rust crate dependencies (petgraph, crepe/datafrog, yrs/automerge, redb, dashmap) are listed but there is no mention of: dependency auditing, supply chain verification (cargo-vet, cargo-deny), or minimum version pinning. Given this is a security-critical runtime, supply chain integrity is essential. |
| A04 | Cryptographic Failures | UNDERSPECIFIED | UCAN "compatible" is mentioned. No cryptographic primitives are specified. No key management, rotation, or compromise recovery is described (HIGH-4). |
| A05 | Injection | VULNERABLE | Cypher query injection surface is open (CRITICAL-1). The API accepts raw Cypher strings. No parameterization mandate. No system-node protection. |
| A06 | Vulnerable and Outdated Components | NOT ADDRESSED | No dependency management or vulnerability scanning strategy described. |
| A07 | Identification and Authentication Failures | PARTIAL | Instance identity is "DID-based" per the research documents, but the specification itself does not define authentication for P2P peers. Local authentication is deferred to the TypeScript layer (Better Auth). |
| A08 | Software and Data Integrity Failures | PARTIAL | Version chains provide data integrity for individual entities. CRDT merge integrity is underspecified (CRITICAL-3: clock manipulation, edge spam). Merkle-verified snapshots are mentioned in research but not in the specification. |
| A09 | Security Logging and Monitoring Failures | NOT ADDRESSED | No mention of audit logging, security event recording, or anomaly detection. The operation log mentioned in the sync precedents document would serve this purpose, but it is not in the engine specification. |
| A10 | Mishandling of Exceptional Conditions | PARTIAL | Rust's type system (Result/Option) provides good protection against unhandled errors. But the specification does not describe: what happens when IVM update fails, what happens when CRDT merge encounters an invalid operation, what happens when a capability check encounters corrupted graph data. |

---

## Plugin / Module Trust Analysis

### What Can a Malicious Module Do?

Under the current specification (assuming the capability system from the exploration document is implemented):

| Attack | Community Module | Verified Module | Platform Module |
|--------|-----------------|-----------------|-----------------|
| Read other modules' data | Blocked (scoped store:read) | Possible (broad store:read) | Full access |
| Modify other modules' data | Blocked (no store:write) | Possible (broad store:write) | Full access |
| Escalate own capabilities | Possible via CRITICAL-2 (graph mutation) | Possible via CRITICAL-2 | N/A (already max) |
| DoS via IVM | Blocked if view creation requires capability | Possible via CRITICAL-5 | Possible |
| Exfiltrate data via events | Limited (observe only) | Possible (full event access) | Full access |
| Manipulate version history | Blocked (no version:rollback) | Underspecified | Full access |
| Forge sync operations | N/A (no sync capability) | Underspecified | Possible |
| Read capability graph | Possible if graph traversal includes system nodes | Possible | Full access |

### Key Concern: Verified Modules Have Too Much Power

The capability exploration document's "verified" preset grants `raw:sql`, `raw:cypher`, full `store:readWrite`, and full event access. A compromised or malicious verified module can do almost anything a platform module can do. The distinction between "verified" and "platform" is minimal in practice.

**Recommendation:** The "verified" preset should NOT include `raw:sql` or `raw:cypher` by default. Verified modules should use the structured Store API. Raw query access should require explicit operator opt-in per module.

---

## Tenant Isolation Audit

The specification states (Section 2.1): "Subgraph -- An emergent collection defined by a traversal pattern. Not a container."

This means tenants are NOT isolated by hard boundaries. A subgraph is a query pattern, and the same node could belong to multiple subgraphs (and therefore multiple tenants) simultaneously. Tenant isolation depends entirely on:

1. Capability scoping (tenant A's capability grant scopes to tenant A's subgraph pattern).
2. Query enforcement (queries from tenant A's context cannot traverse outside tenant A's subgraph).
3. IVM isolation (materialized views for tenant A must not include nodes from tenant B).

**Data leakage vectors:**

1. **Subgraph overlap:** If tenant A and tenant B's subgraph patterns overlap (e.g., they share a content type definition node), both tenants can read the shared node. The specification does not describe how to prevent unintentional overlap.

2. **IVM cross-contamination:** A materialized view defined by a query pattern may inadvertently match nodes from multiple tenants if the query does not include a tenant-scoping predicate. The IVM system has no built-in tenant awareness.

3. **Sync-mediated leakage:** If Instance A hosts tenant X and tenant Y, and syncs tenant X's subgraph to Instance B, the sync protocol must ensure that no nodes from tenant Y are included in the sync payload. This requires the sync mechanism to enforce subgraph boundaries at the node/edge level during export.

**Recommendation:** Add a first-class tenant isolation mechanism:
- A `tenant` label or property on every node, enforced at the engine level (not the application level).
- A query rewriter that automatically injects tenant-scoping predicates into every query from a tenant context.
- An IVM tenant filter that ensures materialized views are partitioned by tenant.

---

## Prioritized Recommendations

### P0: Must resolve before implementation begins

1. **Define the system-node protection boundary** (CRITICAL-1, CRITICAL-2). Capability nodes, version chain metadata, and IVM definitions must not be mutable via Cypher queries. Specify a system-zone label namespace and the enforcement mechanism.

2. **Mandate parameterized Cypher** (CRITICAL-1). The `engine.query()` API should reject non-parameterized queries by default. If raw Cypher is needed, require an explicit `engine.rawQuery()` with a separate, more restricted capability.

3. **Specify CRDT clock validation bounds** (CRITICAL-3). Define the maximum acceptable clock skew for HLC timestamps in sync operations. This is a one-line addition that prevents the most damaging sync attack.

### P1: Must resolve before P2P sync ships

4. **Design the revocation propagation protocol** (CRITICAL-4). Choose between short-lived grants with renewal vs. revocation broadcast. Specify the behavior for offline peers.

5. **Specify cryptographic primitives** (HIGH-4). Ed25519 + BLAKE3 + DAG-CBOR is a reasonable default. Pin these decisions to prevent interoperability fragmentation.

6. **Define per-peer rate limits and reputation** (CRITICAL-3). The sync protocol must defend against both clock manipulation and operation flooding.

### P2: Must resolve before community modules ship

7. **Specify IVM resource bounds** (CRITICAL-5). Maximum traversal depth in view definitions, per-view CPU/memory budgets, view creation as a gated capability.

8. **Define the sandbox-engine boundary** (HIGH-1). Enumerate exactly which engine APIs are exposed to sandboxed code and how capability attenuation maps to host function exposure.

9. **Remove raw:sql and raw:cypher from the "verified" preset** (Plugin Trust Analysis). These should be explicit operator opt-in.

### P3: Should resolve before production deployment

10. **Add audit logging** (OWASP A09). Every capability check, every grant/revoke, every sync handshake should produce a structured audit record.

11. **Add dependency auditing** (OWASP A03). Integrate `cargo-deny` or `cargo-vet` for supply chain verification.

12. **Define tenant isolation at the engine level** (Tenant Isolation Audit). Don't rely on application-level query construction to enforce tenant boundaries.

13. **Specify transaction-capability interaction** (HIGH-5). Capabilities should be checked at commit time, not operation time.

---

## Sources

- [OWASP Top 10:2025](https://owasp.org/Top10/2025/en/)
- [OWASP Top 10:2025 Introduction](https://owasp.org/Top10/2025/0x00_2025-Introduction/)
- [UCAN Specification](https://ucan.xyz/specification/)
- [UCAN Revocation Specification](https://ucan.xyz/revocation/)
- [OWASP Top 10 2025 Key Changes (Aikido)](https://www.aikido.dev/blog/owasp-top-10-2025-changes-for-developers)
- [OWASP Top 10 2025 What's Changed (GitLab)](https://about.gitlab.com/blog/2025-owasp-top-10-whats-changed-and-why-it-matters/)
