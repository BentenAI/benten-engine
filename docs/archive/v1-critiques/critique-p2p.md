# Critique: Benten Engine Specification -- P2P & Decentralization Readiness

**Date:** 2026-04-11
**Reviewer:** Engine Philosophy Guardian
**Scope:** Does the specification actually deliver on the decentralized vision that justifies building a custom engine?
**Score: 4/10 -- The decentralized vision is the raison d'etre, but the spec hand-waves on every hard problem.**

---

## The Central Tension

The specification opens with a clear value proposition: "Every person, family, or organization runs their own instance. Data is owned by the user. Instances sync subgraphs bidirectionally. Either party can fork at any time." This is the entire justification for building a custom engine instead of using PostgreSQL+AGE, which already works for the CMS use case.

Yet when you read the spec for the mechanisms that deliver this vision, you find:

- Section 2.5 (CRDT Sync) is 11 lines long
- Section 2.3 (Version Chains) mentions "Sync = exchange version Nodes" as a throwaway line
- Section 2.4 (Capabilities) describes local enforcement but not cross-instance verification
- Section 5 explicitly lists "P2P networking" as something the engine does NOT do
- Open Question 3 literally asks "Should CRDT sync be part of the engine or a separate crate?"

The specification is thorough on the local-instance story (IVM, version chains, concurrency, Cypher queries) and vague on the decentralized story. If decentralization is the reason this engine exists, that ratio should be inverted.

---

## Question-by-Question Analysis

### 1. Is the sync protocol specified enough?

**Verdict: No. It is hand-waving.**

The specification says:

> Sync = exchange version Nodes newer than the peer's latest

This is one sentence. A sync protocol needs to specify:

| Required Element | Spec Coverage | Status |
|---|---|---|
| Wire format (how are Nodes serialized for transport?) | None | Missing |
| Discovery (how do instances find each other?) | None (deferred to "Yggdrasil") | Missing |
| Handshake (how do two instances establish a sync session?) | None | Missing |
| Delta computation (how do you determine what the peer is missing?) | "Exchange version Nodes newer than the peer's latest" | One sentence |
| Ordering (causal? total? per-Node?) | None | Missing |
| Batching (one Node at a time? Chunks? Whole subgraph?) | None | Missing |
| Resumption (what if sync is interrupted mid-stream?) | None | Missing |
| Backpressure (what if one side is slower?) | None | Missing |
| Acknowledgment (how does the sender know the receiver applied the changes?) | None | Missing |

**What the research says should be here:** The P2P precedents document (Section "Recommended Architecture") proposes a three-layer model (Trust & Identity, Sync Protocol, Module Merge Logic) with specific primitives at each layer. The sync-as-channel exploration defines `SyncDelta`, `VectorClock`, `SubgraphBoundary`, and `MergeResult` types. The NextGraph deep dive documents commit DAGs with causal ordering. None of this made it into the engine spec.

**For a first release, you need at minimum:**
1. A serialization format for Nodes and Edges (CBOR, MessagePack, or Protocol Buffers -- pick one)
2. A delta computation algorithm (vector clocks or Merkle trees -- pick one)
3. A transport abstraction (even if the only implementation is "direct WebSocket")
4. A defined handshake sequence (exchange clocks, compute delta, stream changes, acknowledge)

Without these, "CRDT sync" is an aspiration, not a specification.

### 2. Capability revocation across instances

**Verdict: Unaddressed. This is a critical gap.**

The specification says capabilities are "first-class Nodes in the graph" with "GRANTED_TO edges" and are "UCAN-compatible." But it does not address revocation propagation.

**The specific scenario:** Instance A grants Instance B a `read` capability on subgraph S. Instance B syncs subgraph S. Instance A revokes the capability. What happens?

The spec says nothing. The research documents identify this problem but do not solve it:

- The P2P precedents document notes UCAN revocation requires "publishing a signed revocation record" that "validators must check before accepting a UCAN"
- The NextGraph deep dive describes key rotation ("new epoch for the overlay") as the revocation mechanism, which is heavyweight

**The hard sub-problems:**

**(a) Notification.** How does Instance B learn about the revocation? If it is offline, it cannot receive the notification. If it polls, how often? If there is a push mechanism, that requires a persistent connection or a relay.

**(b) Enforcement.** Instance B already has a copy of the data. Revoking the capability does not delete the data from Instance B's storage. You can stop future syncs, but you cannot un-share data. This is a fundamental property of distributed systems (you cannot take back information), but the spec should acknowledge it explicitly and define what "revocation" means in practice: "revocation stops future sync; it does not delete previously-synced data."

**(c) Derived capabilities.** If Instance B delegated a subset of its capability to Instance C (UCAN attenuation), does revoking B's capability automatically revoke C's? UCAN says yes (the chain is invalid if any link is revoked). But C needs to discover this.

**What the spec should say:**
- Revocation is eventual, not instant. Peers learn about revocations on their next sync attempt or via a revocation channel.
- Previously-synced data is not recallable. Revocation prevents future access, not past access.
- Capability tokens have a TTL (time-to-live). Even without explicit revocation, capabilities expire and must be renewed. This bounds the damage from delayed revocation.
- The engine stores a revocation list as Nodes in the graph (local). On every sync handshake, the initiator presents its capability; the receiver checks the revocation list.

### 3. Fork consistency

**Verdict: Not addressed. The word "fork" appears but the semantics are undefined.**

The specification says: "Either party can fork at any time (take their copy and diverge)." The sync-as-channel exploration defines `sync.fork()` more carefully, but none of that made it into the engine spec.

**The hard questions:**

**(a) Snapshot consistency.** When I fork, do I get a consistent snapshot (all Nodes and Edges at a single point in time) or a potentially-inconsistent partial snapshot (some Nodes at time T, others at time T+1)?

If the fork is "stop syncing and keep what you have," the answer depends on whether the last sync completed successfully. If I fork mid-sync (some Nodes received, others not), my local copy might have Edges pointing to Nodes I never received.

**(b) Fork point.** Is there a defined "fork point" in the version chain? If I fork at version V, and later I want to re-sync, can the system identify V as the point of divergence and compute a merge from there?

**(c) Fork of a fork.** If Instance B forked from Instance A, and Instance C forked from Instance B, and Instance A makes changes, can Instance C ever sync with Instance A directly? Or is C permanently limited to syncing with B?

**What the spec should say:**
- A fork creates a snapshot of the subgraph at the latest fully-synced version. The fork point is recorded as a "fork marker" Node with a reference to the version clock at the time of fork.
- Forks from mid-sync are not permitted. The system must complete or roll back the current sync before allowing a fork. (Or: mid-sync forks are permitted but the fork is marked as "potentially inconsistent" and the user is warned.)
- Fork points enable future re-merge: the system can compute the delta between the fork point and the current state of either branch to produce a merge.

### 4. Network partition tolerance

**Verdict: Mentioned only as "CRDT merge" which is insufficient.**

The specification says "per-field last-write-wins with Hybrid Logical Clocks" for Node properties and "add-wins semantics" for Edges. This handles concurrent writes. But it does not address the scale problem.

**The scenario:** Two instances sync, then go offline for a month. Both make changes. When they reconnect:

**(a) How large is the merge?** If each instance made 10,000 changes, the delta is 20,000 operations. Is this streamed? Batched? Is there a progress indicator?

**(b) Is it bounded?** The merge size is bounded by the number of changes, not the size of the subgraph. If both instances modified the same 100 Nodes, the merge is 200 operations regardless of whether the subgraph has 100 or 100,000 Nodes. But the spec does not state this.

**(c) HLC drift.** Hybrid Logical Clocks depend on wall-clock time (with a logical component for ties). If two instances have drifted wall clocks (common after a month offline), HLC values may be skewed. The spec does not address clock drift tolerance.

**(d) Tombstone accumulation.** If add-wins is the Edge strategy, deletes use tombstones. After a month of changes, tombstones accumulate. Is there a garbage collection mechanism? When is it safe to discard a tombstone?

**What the spec should say:**
- Delta computation is O(changes since last sync), not O(subgraph size). This is achieved via version vectors.
- Large deltas are streamed in chunks of configurable size, with acknowledgment per chunk.
- HLC values include a configurable maximum drift tolerance (e.g., 1 hour). Clocks that have drifted beyond tolerance trigger a full re-sync rather than an incremental merge.
- Tombstones are retained for a configurable retention period (e.g., 90 days). After retention, a tombstone is compacted into the snapshot. Peers that have not synced within the retention period must perform a full snapshot sync rather than an incremental sync.

### 5. Trust in a trustless environment

**Verdict: The trust anchor is undefined. It IS turtles all the way down as specified.**

The specification says capabilities are "UCAN-compatible" and checked "at every operation boundary." But UCAN chains need a root.

**The trust chain problem:**

Instance A creates a CapabilityGrant Node: `{ domain: 'store', action: 'read', scope: 'content/*' }`. Instance B receives this via sync. Instance B needs to verify:
1. That Instance A actually created this grant (signature verification)
2. That Instance A had the authority to create this grant (delegation chain)
3. That the grant has not been revoked (revocation check)

Step 2 is the problem. Who gave Instance A the authority? If it is self-signed (Instance A is the root), then every instance is its own root of trust. This is valid for the "every person runs their own instance" model -- you are the authority over your own data. But it means Instance B must decide whether to trust Instance A's root. How?

**Options the spec should evaluate:**
1. **Self-sovereign roots.** Every instance is its own root of trust. Trust is established out-of-band (QR code exchange, shared secret, manual verification). This is the Holochain model.
2. **Well-known roots.** A set of trusted root authorities (like CAs in TLS). This centralizes trust but simplifies verification.
3. **Web of trust.** Instance B trusts Instance A because Instance C (whom B already trusts) vouched for A. This is the PGP/GPG model.
4. **DID-based.** Instances have DIDs (`did:web`, `did:key`). The DID document specifies the instance's public key. Trust is established by resolving the DID. This is the AT Protocol model.

The spec should pick one (or a hybrid) and specify it. Currently it says "UCAN-compatible" without specifying what trust model backs the UCAN chain.

**Recommendation:** Self-sovereign roots with optional DID verification. Each instance generates a key pair on first boot. Cross-instance trust is established by exchanging public keys (directly or via `did:web`). This is simple, decentralized, and compatible with upgrading to a more sophisticated trust model later.

### 6. Privacy: Should encryption be the default?

**Verdict: The spec's stance ("optional E2EE") is actually the correct pragmatic choice, but it needs to be more explicit about the tradeoffs.**

The specification says E2EE is optional. The NextGraph deep dive concluded that E2EE-everything is incompatible with server-side features (search, analytics, recommendations). The sync-as-channel document does not address encryption at all.

**The tension:** The vision statement says "data is owned by the user." User-owned data implies the user controls access. Plaintext data on a server means the server operator has access. These are in conflict.

**The nuanced position the spec should take:**

| Data Category | Default Encryption | Rationale |
|---|---|---|
| Local-only data (never synced) | At-rest encryption (transparent) | Standard database encryption. Server operator can query. |
| Synced data in transit | TLS + message-level signing | Prevents interception and tampering. |
| Synced data at rest on remote instances | Optional E2EE (per-subgraph) | The data owner decides. E2EE subgraphs cannot be server-indexed. |
| Capabilities/tokens | Always signed, never encrypted | Must be verifiable by any party in the chain. |

**Search with encrypted data:** The spec should acknowledge that E2EE subgraphs are not server-searchable. If a user wants their data encrypted AND searchable, they need a local search index (like NextGraph's verifier). The engine could support this by maintaining IVM views on decrypted data in memory (never persisted to disk), but this is a significant feature with security implications that should be explicitly scoped.

**The honest answer:** For 95% of Thrum use cases (CMS, commerce, social), transparent server-side encryption is sufficient. E2EE is a premium feature for specific verticals (healthcare, legal, government). The spec should say this explicitly rather than leaving it ambiguous.

### 7. Does the engine need to know about P2P at all?

**Verdict: Yes, but less than the spec implies and more than the spec specifies.**

The spec currently says the engine does sync ("Section 2.5: CRDT Sync") but not networking ("Section 5: P2P networking -- Yggdrasil handles this"). This is the right conceptual split. But the boundary is drawn wrong.

**What the engine must own (sync primitives):**
- Version chains (Section 2.3 -- well specified)
- Conflict resolution semantics (Section 2.5 -- under-specified)
- Capability enforcement for sync operations (Section 2.4 -- locally specified, not cross-instance)
- Subgraph extraction (implied but not specified)
- Delta computation (not specified at all)
- Merge application (mentioned, not specified)

**What the engine must NOT own (networking):**
- Peer discovery
- Transport protocols (WebSocket, Yggdrasil, HTTP)
- NAT traversal
- Relay infrastructure
- Connection management

**What is in the gray zone:**
- Serialization format for sync messages (engine needs to define what a "sync message" contains; the transport layer decides how to encode it)
- Sync session management (the engine needs to track "I am syncing with peer X, we are at step Y"; the transport layer handles the connection)
- Backpressure/flow control (the engine needs to produce deltas at a manageable rate; the transport layer handles TCP-level flow control)

**The spec currently puts all of sync into a vague Section 2.5 and a vague API (`engine.sync(peer, subgraph)`). It should instead:**

1. Define a `SyncProtocol` interface with methods like `computeDelta()`, `applyDelta()`, `verifyCapability()`, `recordForkPoint()`
2. Define a `SyncTransport` interface with methods like `connect()`, `send()`, `receive()`, `disconnect()`
3. Specify that the engine implements `SyncProtocol`; external crates implement `SyncTransport`
4. The `engine.sync()` API is a convenience that composes the two

This separation is critical because it determines what the engine team builds (sync logic) versus what can be contributed by the community (transport adapters for WebSocket, Yggdrasil, libp2p, etc.).

---

## Cross-Cutting Concerns

### The Subgraph Definition Problem

The specification mentions subgraphs repeatedly but never defines how they are bounded. This is the single most important design decision for sync, and it is completely absent.

The sync-as-channel exploration proposes `SubgraphBoundary` with traverse-based boundaries (roots + edge types + max depth). The P2P precedents document notes that AT Protocol uses flat collections (simpler) while NextGraph uses per-store overlays (more structured).

The spec should take a position. The traverse-based approach is the most natural for a graph engine, but it has real problems (Section 9.1 of sync-as-channel: unexpected reachability, ambiguous membership, expensive evaluation). The spec should either adopt it with explicit mitigations or propose an alternative.

### The "Schema Evolution During Sync" Problem

Open Question 6 asks "How do we handle schema evolution during sync?" but does not sketch an answer. This is not a minor question. If Instance A defines a content type with fields {title, body} and Instance B defines the same content type with fields {title, body, author}, what happens when they sync?

The P2P precedents document suggests "schema-aware validation on receive" (Holochain DNA model). The spec should commit to: schema mismatch on a shared subgraph is a sync error, not a silent merge. Instances must agree on schema before syncing a content type.

### The IVM + Sync Interaction

Section 2.2 (IVM) and Section 2.5 (Sync) are specified independently, but they interact in non-obvious ways. When a sync delta arrives:
1. The engine applies the delta to storage
2. IVM must incrementally update all affected materialized views
3. Reactive subscribers must be notified

This is the same as a local write, which is good (sync changes flow through the same IVM pipeline). But the spec does not state this explicitly, and there are edge cases:
- What if the delta contains 10,000 changes? Does IVM process them one at a time, or as a batch?
- What if a materialized view query spans both synced and local-only Nodes? Is there a consistency boundary?
- What if two concurrent sync sessions modify Nodes that affect the same view?

---

## Score Justification: 4/10

| Category | Score | Weight | Rationale |
|---|---|---|---|
| Local engine story (IVM, versions, concurrency) | 8/10 | 30% | Well-specified, coherent, motivated by research |
| Sync protocol | 2/10 | 25% | One paragraph for the engine's raison d'etre |
| Capability / trust model | 3/10 | 20% | Local enforcement specified; cross-instance verification absent |
| Fork / partition semantics | 1/10 | 15% | Not specified beyond a single sentence |
| Privacy / encryption | 3/10 | 10% | "Optional E2EE" is correct but under-specified |

**Weighted score: 3.85, rounded to 4.**

The specification is a good document for building a local high-performance graph engine with IVM. It is an inadequate document for building the decentralized platform described in the vision statement. The delta between "what the spec delivers" and "what the vision promises" is where the work needs to happen.

---

## Recommendations

### Must-Fix Before Implementation Starts

**M-1. Write a Sync Protocol section.** At minimum: serialization format, delta computation algorithm, handshake sequence, session lifecycle, error handling. This does not need to be a full RFC, but it needs to be more than one paragraph. Target: 2-3 pages.

**M-2. Define subgraph boundaries.** Pick an approach (traverse-based, label-based, explicit membership, or hybrid), specify it, and analyze its failure modes. This is a load-bearing design decision that affects the engine's storage model, IVM integration, and capability scoping.

**M-3. Specify the trust anchor.** "UCAN-compatible" is not a trust model. Specify: how are root capabilities created, how are they verified cross-instance, what happens when they are revoked. Target: 1 page.

**M-4. Separate SyncProtocol from SyncTransport.** Define the boundary between "what the engine owns" and "what the networking layer owns." This determines the API surface and the crate boundaries.

### Should-Fix Before First Sync Release

**S-1. Fork semantics.** Define: what state guarantees a fork provides, whether mid-sync forks are permitted, how fork points are recorded, and how re-merge after fork works.

**S-2. Partition reconciliation.** Define: upper bounds on merge size, tombstone garbage collection, clock drift tolerance, and large-delta streaming.

**S-3. Schema evolution during sync.** Take a position: strict schema match, schema negotiation, or schema-aware merge.

**S-4. Privacy model.** Document the tiered encryption approach explicitly. Acknowledge the search/encryption tradeoff. Define what "user-owned data" means when data is stored on a server.

### Consider for V2

**C-1. DID-based instance identity.** Each engine instance gets a DID. This enables cross-instance addressing without a central registry.

**C-2. Merkle-verified subgraph snapshots.** Content-addressed snapshots for full sync and fork operations. Enables integrity verification without trusting the transport.

**C-3. Operation log.** An append-only log of all graph mutations, as recommended by the P2P precedents research. Useful for sync delta computation, audit trails, and undo/redo -- even before any cross-instance sync exists.

---

## Summary

The Benten Engine specification is strong where the engine is a local runtime (IVM, version chains, concurrency, Cypher queries). It is weak where the engine is a decentralized platform (sync, trust, fork, privacy). Since the decentralized vision is the explicit justification for building a custom engine -- "No existing tool combines: graph data model + reactive incremental view maintenance + CRDT sync + capability enforcement + embeddable + WASM + true concurrent read/write" -- the sync and trust sections need to be elevated to the same level of rigor as the IVM and version chain sections.

The research is there. The P2P precedents document, the NextGraph deep dive, and the sync-as-channel exploration collectively contain enough analysis to write a rigorous sync specification. The problem is that the specification document does not synthesize this research into concrete engineering decisions. It defers them to open questions or omits them entirely.

Build the local engine first -- that is the right sequencing. But specify the sync protocol now, even if you implement it later. The sync requirements will constrain the engine's storage model, version chain design, and capability system. Discovering those constraints after the engine is built means a rewrite.
