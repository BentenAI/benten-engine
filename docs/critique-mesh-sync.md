# Critique: Mesh Sync Architecture for Multiple Peers

**Date:** 2026-04-11
**Scope:** The specification describes sync between TWO instances. This critique stress-tests that design against the real scenario: one instance syncing different subgraphs with many different peers simultaneously -- a full mesh.
**Documents reviewed:** SPECIFICATION.md, explore-p2p-sync-precedents.md, explore-sync-as-channel.md, explore-sync-as-store-layer.md, explore-sync-as-fundamental.md, explore-graph-native-protocol.md

---

## The Scenario

A student's instance syncs:
- Learning records with their school
- Health data with their doctor
- Game progress with a game server
- Social posts with friends' instances
- Work portfolio with an employer
- Family photos with family members

That is 6+ simultaneous sync relationships, each with different subgraphs, different capability scopes, different merge strategies, different online/offline patterns.

The specification's section 2.5 says: "Subgraphs sync between instances using CRDTs" and section 2.1 defines a Subgraph as "an emergent collection defined by a traversal pattern." The sync-as-store-layer exploration proposes `@benten/sync` as a module with a SyncStore wrapper, subgraph-as-Node pattern, and per-type conflict handlers.

None of these documents model what happens when a single instance is the hub of 6+ sync relationships. The two-peer model is the building block, but the building block has properties that do not compose linearly. This critique examines each of the seven questions and finds the gaps.

---

## 1. Sync State Management: Where Does Per-Peer Metadata Live?

### What the current documents propose

The sync-as-store-layer exploration proposes sync metadata as Nodes in the graph:

```
Node: { id: 'sync/learning-alice', type: 'sync-agreement', config: {
  clock: { 'school/main': 8, 'student/alice': 4 },
  boundary: { roots: ['student/alice'], follow: ['enrolled_in', ...] },
  ...
}}
```

A `SyncMetadata` table tracks per-node HLC and origin instance. A membership index (`Map<nodeId, Set<subgraphId>>`) caches which nodes belong to which subgraphs.

### How this breaks with 6+ peers

**Vector clock explosion.** With 6 peers, every sync-agreement Node's `clock` field has 7 entries (self + 6 peers). The student instance must maintain 6 separate vector clocks. When the student edits a Node that exists in 3 different sync scopes, the SyncMetadata table needs 3 rows for that single node (one per subgraph). With 1,000 synced nodes across 6 relationships, that is up to 6,000 SyncMetadata rows. The membership index holds 1,000 entries, each potentially mapping to 3+ subgraphs.

This is not a scalability crisis -- these numbers are manageable. But it reveals a structural issue: **sync state grows as O(nodes x sync_relationships), not O(nodes).**

**Clock coordination across sync scopes.** When the student updates a Node that is shared with both the school and the employer, the engine must:
1. Look up which sync agreements include this Node (membership index query)
2. Generate an HLC timestamp
3. Write SyncMetadata rows for each agreement
4. Enqueue replication events for each peer

Steps 1-4 happen on every write to a multi-scope node. The sync-as-store-layer exploration handles this for a single scope but never addresses the fan-out case.

**Recommendation: Sync-agreement Nodes as proposed are correct, but the membership index needs to be a first-class persistent structure, not just an in-memory cache.** When an instance reboots, rebuilding the index from graph traversal for 6+ agreements (each potentially covering hundreds of nodes) could take seconds. Store the index in a dedicated relational table: `(node_id, agreement_id)` with indexes on both columns. Rebuild is then a sequential scan, not a graph traversal.

### How precedent systems handle this

**Holochain:** Each DNA (application) has its own DHT network. An agent participating in 6 apps runs 6 independent DHT processes. There is zero cross-DNA state -- each DNA's gossip is fully isolated. The agent's source chain is per-DNA. This is the cleanest model: complete isolation between sync scopes, at the cost of no cross-scope queries.

**AT Protocol:** Each user has exactly one repository. There are no multiple sync scopes per user -- the entire repo syncs to relays. AT Protocol solves the mesh problem by not having it: your PDS syncs one stream to one or more relays, and relays handle fan-out. The user never manages per-peer sync state.

**Matrix:** Each room has its own event DAG. A server participating in 1,000 rooms maintains 1,000 independent DAGs. Room state resolution runs independently per room. There is no cross-room state. The server's sync state is `(room_id, last_event_id)` per room -- a flat table.

**Gun.js:** No per-peer state at all. Everything is broadcast to the mesh. Every node gets every update. There is no selective sync -- the mesh is one global namespace.

### The lesson

**Every successful mesh system isolates sync state per scope.** Holochain per-DNA, Matrix per-room, AT Protocol per-repo. None of them maintain cross-scope sync state. The Benten spec's "subgraph as sync scope" is the right shape, but the documents need to be explicit: **each sync agreement is a fully independent sync domain.** No shared clocks, no shared membership, no cross-agreement coordination.

This means the sync module's internal data model should look like:

```
SyncAgreement (Node in graph)
  ├── clock: VectorClock (specific to this agreement)
  ├── boundary: SubgraphBoundary
  ├── capabilities: per-participant
  └── state: 'active' | 'paused' | 'forked'

MembershipIndex (relational table)
  (node_id, agreement_id) -- many-to-many

ChangeLog (relational table, partitioned by agreement_id)
  (id, agreement_id, hlc, operation, target, payload)

SyncMetadata (relational table)
  (node_id, agreement_id, hlc, origin_instance, local_version)
```

Partitioning the ChangeLog by agreement_id ensures that syncing with the school does not require scanning changes relevant only to the doctor.

---

## 2. Fan-Out Writes: Multiplexing Updates to Multiple Peers

### The problem precisely stated

The student edits their "professional development" Node. This Node exists in three sync scopes:
1. Shared with the school (learning records)
2. Shared with the employer (work portfolio)
3. Shared with a mentor (career guidance)

After the write, the engine must:
1. Detect that the Node belongs to 3 agreements
2. Generate change records for each agreement
3. Enqueue replication to 3 different peers
4. Apply capability checks per agreement (the employer might have read-only access to this Node)

### What the documents propose

The sync-as-store-layer exploration shows a SyncStore wrapper that checks membership on every write. But it only shows the single-agreement case. The wrapper's `createNode` pseudocode does not address fan-out.

### The architecture for fan-out

The SyncStore wrapper needs a **write interceptor that fans out to all affected agreements:**

```
Write intercepted by SyncStore:
  1. Execute the write on the inner Store (local ACID)
  2. Query membership index: which agreements include this node?
  3. For each agreement:
     a. Check caller's write capability for this node type in this agreement
     b. If permitted: create a ChangeRecord in this agreement's partition
     c. Enqueue notification to this agreement's transport
  4. Return to caller (steps 2-3 are post-commit, async)
```

**Critical design decision: Should fan-out be synchronous or asynchronous?**

- **Synchronous (in-transaction):** The write and all ChangeRecord inserts happen in one transaction. If any fail, the entire write rolls back. This provides atomicity but makes write latency proportional to the number of affected agreements. With 6 agreements, a single Node update generates 6 ChangeRecord inserts inside the transaction.

- **Asynchronous (post-commit):** The write commits first. A background process scans the WAL or a change feed, resolves membership, and creates ChangeRecords. This gives constant-time writes but introduces a window where the write has committed locally but no ChangeRecord exists yet. If the process crashes in this window, the change is lost to sync.

**Recommendation: Synchronous for correctness, with batching for performance.** The membership index lookup is an in-memory operation (O(1) per node). Writing ChangeRecords is a batch INSERT. Even with 6 agreements, this adds microseconds, not milliseconds, to the transaction. The atomicity guarantee is worth more than the marginal latency.

### How precedent systems handle fan-out

**Matrix:** When a server sends an event to a room with 100 participating servers, it fans out the event to all 100 via the Server-Server API. This is a full-mesh fan-out: every event goes to every participant. Matrix handles this through federation queues -- each destination server gets its own retry queue. The sending server does not wait for all 100 acknowledgments.

**Holochain:** No fan-out at all in the traditional sense. When an agent publishes an entry, it goes to the DHT authorities responsible for that entry's address hash. Those authorities gossip it to their neighbors. The publisher does not manage per-peer delivery -- the DHT handles distribution.

**AT Protocol:** Fan-out is the relay's job, not the PDS's job. The PDS pushes to one relay. The relay broadcasts to all consumers. The PDS does not know or care how many consumers exist.

### The lesson

**Benten should adopt the Matrix-style queue model for fan-out.** Each sync agreement gets its own outbound queue. Writes that affect multiple agreements post to multiple queues. Each queue manages its own delivery, retry, and backpressure independently. The writer does not wait for remote acknowledgment -- only for local ChangeRecord persistence.

---

## 3. Selective Sync: Filtering What Gets Sent

### The problem

The student shares learning records with the school but NOT health data. The school syncs with the student's instance. How does the engine filter what gets sent?

### What the documents propose

The sync-as-channel exploration proposes capability-scoped access rules:

```typescript
sync.share('learning/alice', 'student/alice', {
  read: ['grade', 'assignment', 'course'],
  write: ['assignment_submission', 'study_note'],
  readEdges: ['enrolled_in', 'scored_on'],
  writeEdges: ['submitted', 'annotated'],
});
```

The sync-as-store-layer proposes traverse-based subgraph boundaries:

```typescript
interface SubgraphBoundary {
  roots: string[];
  followEdgeTypes: string[];
  maxDepth: number;
}
```

### Where this gets complicated

**Structural separation vs. capability filtering are two different mechanisms, and the documents conflate them.**

Structural separation (the boundary) defines WHAT NODES are in scope. This is resolved at agreement creation time: traverse from the roots, following specified edge types, up to maxDepth. The resulting set of node IDs is the subgraph.

Capability filtering defines WHAT OPERATIONS are permitted on nodes that are in scope. The school can read grades but not write them. The student can create study notes but not grades.

These must be evaluated at different times:
- **Boundary evaluation** happens when the agreement is created, and when the boundary is re-evaluated (new nodes added to the graph might become reachable).
- **Capability filtering** happens on every sync push/pull, per-operation.

**The missing piece: dynamic boundary evolution.** The student enrolls in a new course. That course's Nodes are now reachable from the student's root via `enrolled_in` edges. Are they automatically part of the synced subgraph? If boundaries are traverse-based, yes. But the school has not been told about these new nodes. The membership index needs to be updated, and the school needs to receive the initial state of the new nodes.

**Recommendation: Boundaries should be evaluated lazily on each sync cycle, not eagerly at agreement creation.** The sync push/pull operation should:
1. Re-evaluate the boundary (traverse from roots)
2. Diff against the previous membership snapshot
3. Include newly-reachable nodes as "created" in the delta
4. Mark no-longer-reachable nodes as "removed" from scope (not deleted -- they still exist locally)

This matches how AT Protocol handles new records: the relay discovers them on the next crawl, not at registration time.

### How precedent systems handle selective sync

**Holochain:** Selectivity is at the DNA level. Each DNA defines its own entry types and validation rules. An agent cannot "partially" participate in a DNA. Either you join the network (and sync everything it publishes to your arc) or you don't. Sub-DNA selectivity does not exist.

**AT Protocol:** Collection-scoped sync (introduced in Sync 1.1, January 2026). A consumer can subscribe to a specific collection (e.g., `app.bsky.feed.post`) rather than the entire repository. This is the AT Protocol equivalent of "sync this content type but not that one."

**Matrix:** Room membership is all-or-nothing. If you are in a room, you see all events. There is no per-event-type filtering at the federation level. Clients can filter on the client-server API, but servers exchange everything.

**Git:** Partial clone and sparse checkout (introduced in Git 2.25+). A client can request a subset of the repository based on path patterns. The server filters the packfile to include only matching objects. This is the closest to Benten's traverse-based boundaries.

### The lesson

**No precedent system does traverse-based selective sync well.** Holochain and Matrix are all-or-nothing per scope. AT Protocol is collection-scoped (flat, not graph). Git has sparse checkout but it is path-based, not graph-based.

Benten's traverse-based boundaries are novel. This is both a strength (more expressive than any precedent) and a risk (no battle-tested implementation to learn from). The key mitigation: **start with explicit membership (a fixed set of node IDs per agreement) and add traverse-based evaluation as an optimization later.** Explicit membership is simple, predictable, and easy to debug. Traverse-based boundaries are elegant but introduce edge cases around dynamic reachability that the spec has not addressed.

---

## 4. Conflicting Sync Scopes: Multi-Writer on Shared Data

### The problem

The employer and the school both have write access to the student's "professional development" subgraph. The employer adds a certification. The school adds a grade. Both sync back to the student's instance. How do conflicts resolve when the same data has multiple sync peers writing to it?

### What the documents propose

The sync-as-store-layer proposes layered conflict resolution:

```typescript
interface SyncPolicy {
  default: 'lww' | 'field-lww' | 'reject';
  handlers?: Record<string, ConflictHandler>;
}
```

The sync-as-channel proposes field-level merge with authority rules:

```typescript
// grade.score: school-wins (school is the authority for grades)
// grade.student_notes: alice-wins (student is the authority for notes)
// grade.tags: union (both can add tags, merged as a set)
```

### Where this breaks in the mesh case

The two-party model assumes exactly two sides to a conflict: local vs. remote. The authority rules are binary: "owner-wins" or "sharer-wins."

With three writers (student, school, employer) modifying the same Node, conflicts become three-way:

```
Student: config.skills = ['python', 'sql'] at HLC 100
School:  config.skills = ['python', 'statistics'] at HLC 102
Employer: config.skills = ['python', 'management'] at HLC 101
```

Which version wins? LWW says the school (HLC 102). But the merge-strategy says "employer-wins for certification-related fields" and "school-wins for academic fields." And `skills` is claimed by both.

**The fundamental issue: authority rules are per-agreement, but data is shared across agreements.** The student-school agreement says the school is the authority for academic data. The student-employer agreement says the employer is the authority for professional data. The `skills` field is both academic and professional.

### How precedent systems handle multi-writer conflicts

**Matrix State Resolution v2:** The most sophisticated multi-writer conflict resolution in production. When three servers fork and re-merge, the algorithm:
1. Takes the union of all state events from all branches
2. Resolves conflicts using a deterministic algorithm based on: event power level (who sent it?), origin server timestamp, and event ID hash as a tiebreaker
3. The result is identical on all servers because the algorithm is deterministic given the same inputs

The key insight: **Matrix does not use per-field merge.** It resolves at the event level. If two events conflict on the same state key, one wins entirely. This is per-Node LWW, not per-field LWW. Matrix's experience suggests that per-field merge with authority rules creates a complexity explosion that is not worth the benefit.

**Git:** Three-way merge uses the common ancestor. If both branches modified the same line, it is a conflict requiring manual resolution. No automatic authority-based resolution.

**Holochain:** Each entry has exactly one author. Two agents cannot modify the "same" entry. They create different entries that may represent the same concept. Conflict resolution is pushed entirely to the application layer.

### Recommendation

**Replace per-agreement authority rules with a single, global conflict resolution policy per Node type.**

Instead of:
```
student-school agreement: school-wins for grades
student-employer agreement: employer-wins for certifications
```

Use:
```
Node type 'grade': field 'score' resolved by: authority = school DID
Node type 'certification': field 'status' resolved by: authority = employer DID
Node type 'skills': field 'list' resolved by: union (merge all values)
```

The resolution policy lives on the Node type definition, not on the sync agreement. This way, when three peers modify the same Node, there is a single, deterministic resolution path that does not depend on which agreement the change arrived through.

**For truly irreconcilable conflicts (two authorities both claim the same field), adopt Matrix's approach: deterministic tiebreaker.** Sort by HLC, then by instance ID hash. The result is arbitrary but deterministic -- all instances converge to the same state.

---

## 5. Offline Accumulation: Bulk Sync After Reconnection

### The problem

The school's instance goes offline for a week. During that week, 500 changes accumulate on the student's instance within the school's sync scope. When the school reconnects, what does the sync look like?

### What the documents propose

The sync-as-store-layer proposes ChangeRecords with HLC ordering:

```typescript
interface ChangeRecord {
  id: string;
  subgraphId: string;
  originInstance: string;
  hlc: bigint;
  operation: 'create' | 'update' | 'delete';
  target: { kind: 'node' | 'edge' | 'record' | 'file'; ... };
  payload?: JsonValue;
}
```

Sync = exchange ChangeRecords since the last known HLC.

### How 500 accumulated changes play out

If the student's instance has accumulated 500 ChangeRecords for the school's agreement, reconnection means:

1. The school says: "My last known state is HLC X."
2. The student sends all ChangeRecords with HLC > X for this agreement.
3. The school applies them in HLC order.

**Problem 1: Redundant intermediate states.** If the student updated Node A 50 times during the week, the ChangeLog has 50 records for Node A. But the school only needs the final state. Sending all 50 is wasteful.

**Problem 2: Snapshot vs. operation log.** The spec proposes version chains with snapshot Nodes. Each version is a complete snapshot. For sync, sending the latest snapshot for each changed Node would be more efficient than sending 500 individual operations.

**Problem 3: Bandwidth estimation.** 500 ChangeRecords, each with a Node snapshot payload averaging 1KB = 500KB. Manageable. But if the scope includes binary data (images in family photos), the payloads could be megabytes each. 500 photo updates = potential gigabytes of sync traffic.

### How precedent systems handle offline accumulation

**AT Protocol:** The PDS stores a linear commit log. On reconnection, the relay requests all commits since the last known `rev`. Commits reference the MST diff, so only changed records are sent, not the full tree. A key optimization: if the relay has been offline for a long time, it can request a full CAR file (snapshot) instead of replaying individual commits. There is a threshold: if the diff would be larger than X% of the total repo, just send the full repo.

**Matrix:** Federation backfill. When a server reconnects, it requests events since the last known event ID. If the gap is large, the server can request state at a specific point (the room's state at the moment of the last known event), then receive all subsequent events. This is a "snapshot + delta" approach.

**Git:** `git fetch` sends a packfile containing all objects reachable from the remote's HEAD but not from the local's known refs. Git's pack-objects algorithm deduplicates and delta-compresses objects in the pack. This is extremely efficient for bulk transfer.

**Holochain:** Kitsune2's gossip protocol (as of 0.5.1) syncs the full DHT shard in approximately 1 minute for initial sync, using a multi-round gossip approach. For reconnection, the protocol compares bloom filters of known data and exchanges only the missing entries.

### Recommendation

**Implement a two-tier sync protocol:**

**Tier 1: Delta sync (normal operation).** Exchange ChangeRecords since the last known HLC. Compact redundant updates to the same Node (send only the latest state, not all intermediate operations). This handles the common case of short disconnections.

**Tier 2: Snapshot sync (long disconnection).** When the number of accumulated ChangeRecords exceeds a threshold (e.g., 1,000 changes or 1 hour of wall-clock drift), switch to snapshot mode. Export the current state of all Nodes in the subgraph boundary as a signed, content-addressed snapshot. The receiving instance diffs the snapshot against its local state and applies changes.

The threshold is a tuning parameter. AT Protocol uses a similar heuristic: Jetstream (lightweight delta) for real-time, CAR files (full snapshot) for backfill.

**For binary data (files/photos):** Metadata-first sync. The ChangeRecord includes file metadata (hash, size, mime type) but not the blob. The receiving instance can fetch blobs on demand, lazily, in the background. This is the standard approach in IPFS-based systems (pin the hash, fetch the content when needed).

---

## 6. Peer Discovery and Lifecycle

### The problem

How do I add a new sync peer? Remove one (fork)? Change what subgraph is shared with an existing peer? Is there a handshake protocol?

### What the documents propose

The sync-as-channel exploration defines `sync.share()`, `sync.accept()`, and `sync.fork()`. The sync-as-store-layer represents agreements as graph Nodes. Neither document specifies the protocol-level handshake.

### What a handshake protocol needs

**Step 1: Invitation.** Instance A creates a SyncAgreement Node locally with status `pending`. It generates a signed invitation token (UCAN) encoding:
- The agreement ID
- The subgraph boundary
- The capabilities being offered
- A public key for the invitee
- An expiration timestamp

**Step 2: Acceptance.** Instance B receives the invitation (out of band: URL, QR code, message). B verifies the UCAN chain. B creates its own SyncAgreement Node locally (mirroring A's boundary and capabilities). B sends an acceptance message to A with B's public key and endpoint.

**Step 3: Initial sync.** A evaluates the boundary, collects all Nodes in scope, creates a signed snapshot, and sends it to B. B verifies and applies the snapshot. Both instances now have matching state. The agreement's clock is initialized.

**Step 4: Ongoing sync.** Normal push/pull cycle begins.

**Modifying an existing agreement.** Instance A wants to expand the boundary (share more data with B). A proposes a boundary change by updating its local SyncAgreement Node and sending a `boundary-update` message to B. B can accept (update its own agreement) or reject. This is a two-phase commit at the protocol level.

**Removing a peer (fork).** Instance A calls `sync.fork(agreementId)`. A updates its SyncAgreement to status `forked`. A sends a `fork` notification to B. B updates its copy. No data is deleted -- both keep their current state. Future changes do not propagate.

**Revoking capabilities.** Instance A wants to reduce B's write access. A updates the capabilities in the SyncAgreement and sends a `capability-update` to B. B updates its local copy. B's SyncStore wrapper will now reject writes that exceed the new capabilities. Existing data that was written under the old capabilities remains.

### How precedent systems handle lifecycle

**AT Protocol:** Account creation on a PDS is the "join" event. Account migration (PDS-to-PDS transfer) uses a formal protocol: the old PDS signs a deactivation, the new PDS activates with the same DID. There is no "handshake" per se -- relays discover new PDS instances by crawling.

**Matrix:** Room invites are first-class protocol events. An invite event is signed by the inviting server, sent to the invitee's server, and the invitee's server decides whether to accept. Join and leave are also events in the room DAG. The protocol explicitly handles the case where a server rejects an invite.

**Holochain:** Joining a DNA network is automatic: install the DNA, connect to bootstrap servers, start gossiping. There is no invite/accept handshake. Leaving is also automatic: stop running the DNA.

### Recommendation

**Adopt Matrix's model of lifecycle events as protocol-level messages.** Specifically:

- `invite`: signed by the sharer, includes boundary + capabilities + UCAN
- `accept`: signed by the invitee, includes their endpoint + public key
- `update-boundary`: signed by either party (requires mutual consent)
- `update-capabilities`: signed by the sharer (unilateral -- the sharer controls access)
- `fork`: signed by the forking party (unilateral -- either party can fork at any time)
- `rejoin`: signed by both parties (mutual -- requires a new handshake)

Each message should be idempotent and include a monotonic sequence number per agreement to handle out-of-order delivery.

---

## 7. Bandwidth and Priority

### The problem

Syncing with the game server needs low latency (real-time). Syncing family photos can be lazy (background). Can the engine prioritize sync channels?

### What the documents propose

Nothing. The spec mentions `<10ms` for syncing 100 version Nodes (section 6, Performance Targets) but does not address priority between different sync relationships.

### The architecture for prioritized sync

**Priority is a transport-layer concern, not a sync-protocol concern.** The sync module produces ChangeRecords for each agreement. The transport layer decides when and how to deliver them.

```
SyncStore wrapper (produces ChangeRecords)
  │
  ├── Agreement: game-server (priority: realtime)
  │     └── Transport: WebSocket, persistent connection, push immediately
  │
  ├── Agreement: school (priority: normal)
  │     └── Transport: HTTP polling every 30 seconds, or WebSocket if available
  │
  ├── Agreement: family-photos (priority: background)
  │     └── Transport: HTTP batch, sync every 5 minutes, metered bandwidth
  │
  └── Agreement: employer (priority: normal)
        └── Transport: WebSocket, push on change, backpressure-aware
```

Each agreement should have a `priority` field in its configuration:

```typescript
interface SyncAgreement {
  // ... existing fields ...
  transport: {
    type: 'websocket' | 'http-poll' | 'http-batch' | 'webrtc';
    priority: 'realtime' | 'normal' | 'background';
    pollInterval?: number;     // for http-poll
    batchInterval?: number;    // for http-batch
    maxBandwidth?: number;     // bytes/sec limit for background
  };
}
```

### How precedent systems handle priority

**Matrix:** No explicit priority between rooms. All events are sent as fast as the federation queue can deliver them. However, Matrix has "lazy-loading" for room state: clients can request minimal initial state and load the rest on demand.

**AT Protocol:** The firehose is a single stream -- no per-collection priority. Jetstream consumers can filter by collection, but the relay does not prioritize one collection over another.

**Gun.js:** No priority. All data propagates at the same rate through the mesh. The DAM protocol makes no distinction between urgent and lazy data.

**Git:** No priority between remotes. `git push` is blocking; `git fetch` processes one remote at a time. Background fetch (via `maintenance`) runs at OS-level scheduling priority.

### The lesson

**No precedent system implements per-scope priority.** This is an area where Benten can differentiate. The per-agreement transport configuration is the right abstraction. The sync module should:

1. Produce ChangeRecords at the same rate regardless of priority
2. Each agreement's transport queue processes at its own rate
3. The transport layer enforces bandwidth limits, connection persistence, and delivery guarantees per priority level
4. The engine's reactive subscriptions (IVM) naturally prioritize: a game server subscribing to position data triggers immediate notification, while a photo sync triggers a batched notification

---

## 8. Consistency Across the Mesh: The Triangle Problem

### The problem

If A syncs with B and C, and B syncs with C too (triangle), do all three eventually converge? Or can A-B and A-C have different views of the same data?

### Analysis

Consider a triangle: Student (S) syncs with School (Sc) and Employer (E). School and Employer also sync with each other (through a shared "professional development" subgraph).

S edits Node X. The change propagates to Sc (via S-Sc agreement) and to E (via S-E agreement). But Sc and E also have a Sc-E agreement that includes Node X. Does Sc forward the change to E through the Sc-E agreement?

**Scenario 1: No transitive forwarding.** Sc receives the change from S. Sc does NOT forward it to E through the Sc-E agreement because Sc did not originate the change. E receives it directly from S via the S-E agreement. Both Sc and E have the same state. This works when S has direct connections to all parties.

**Scenario 2: Transitive forwarding.** Sc receives the change from S. Sc applies it locally. Because Node X is also in the Sc-E agreement, Sc forwards the change to E. E now receives the SAME change twice: once from S, once from Sc. The HLC and origin-instance fields make this detectable (same change, same origin). E deduplicates.

**Scenario 3: S is offline.** S edits Node X locally. S comes online and syncs with Sc but not E (S-E link is still down). Sc now has the change. If Sc forwards it to E (transitive), E gets the change despite S-E being down. If Sc does not forward, E is stale until S-E reconnects.

### How precedent systems handle the triangle

**Matrix:** Full mesh, with forwarding. Every event is sent to every participating server. If server A sends an event to server B, and B knows that server C is also in the room, B will also send the event to C. Events carry origin server signatures, so duplicates are trivially detected and deduplicated. Convergence is guaranteed because the state resolution algorithm is deterministic and every server eventually receives every event.

**Holochain:** No triangle problem. The DHT handles routing. If Agent A publishes an entry, the DHT authorities for that entry's address gossip it to all peers in their arc. There is no concept of A-to-B-to-C forwarding. The DHT IS the mesh.

**AT Protocol:** No triangle problem. Each repo is single-writer. Relays aggregate all repos. If two relays both consume the same PDS, they both get the same data. Convergence is guaranteed by the Merkle tree structure.

**Git:** Triangles are common (origin, upstream, fork). Git does NOT handle transitive forwarding. If you push to origin, upstream does not automatically receive your changes. You must explicitly push to each remote. This is by design: Git gives the user full control over what goes where.

### Recommendation

**Adopt the Matrix model with origin-tagged changes.**

Every ChangeRecord carries an `originInstance` and `originHlc`. When a node receives a change via sync, it checks:
1. Have I already applied a change with this `originInstance` + `originHlc`? If yes, deduplicate (skip).
2. If no, apply the change AND forward it to all other agreements that include the affected Node.

This guarantees eventual convergence in any topology (star, triangle, full mesh, chain) as long as every instance eventually connects to at least one other instance that has the change.

**Deduplication is critical.** Without it, transitive forwarding creates message storms in triangles. The deduplication key is `(originInstance, originHlc, nodeId)` -- globally unique by construction (HLCs are unique per instance).

The ChangeLog table should have a unique constraint on `(agreement_id, origin_instance, origin_hlc, node_id)` to enforce deduplication at the storage level.

**Convergence guarantee:** If all instances use the same conflict resolution policy (per Node type, deterministic), and all instances eventually receive all changes (via direct or transitive delivery), then all instances converge to the same state. This is the same guarantee Matrix provides via State Resolution v2.

---

## Summary of Findings

| Question | Spec Status | Severity | Recommendation |
|----------|-------------|----------|----------------|
| 1. Per-peer sync state | Partially addressed | **HIGH** | Persist membership index as relational table, partition ChangeLog by agreement |
| 2. Fan-out writes | Not addressed | **CRITICAL** | SyncStore interceptor with per-agreement queues, synchronous ChangeRecord creation |
| 3. Selective sync | Partially addressed | **HIGH** | Start with explicit membership, lazy boundary re-evaluation on each sync cycle |
| 4. Multi-writer conflicts | Binary model only | **CRITICAL** | Move resolution policy to Node type definitions, add deterministic tiebreaker |
| 5. Offline accumulation | Implied but unspecified | **MEDIUM** | Two-tier sync (delta for short disconnections, snapshot for long), metadata-first for binary |
| 6. Peer lifecycle | Partial (share/accept/fork) | **HIGH** | Full lifecycle protocol: invite, accept, update-boundary, update-capabilities, fork, rejoin |
| 7. Bandwidth priority | Not addressed | **MEDIUM** | Per-agreement transport configuration with priority levels |
| 8. Triangle convergence | Not addressed | **CRITICAL** | Origin-tagged transitive forwarding with deduplication |

### The Three Critical Gaps

1. **Fan-out writes (Q2):** The spec describes sync between two instances but never addresses what happens when a single write must propagate to multiple peers. Without a fan-out architecture, the system cannot function as a mesh.

2. **Multi-writer conflict resolution (Q4):** The conflict model is binary (local vs. remote). Real mesh scenarios have 3+ writers. Per-agreement authority rules create contradictions when the same data is in multiple agreements. Resolution must be per-Node-type, not per-agreement.

3. **Triangle convergence (Q8):** Without transitive forwarding and deduplication, triangles diverge. This is not an edge case -- it is the normal topology for any instance with more than 2 peers sharing overlapping data.

### Architectural Implications for the Specification

The specification's section 2.5 (CRDT Sync) needs to be expanded from the current 8-line description to address:

1. **Sync topology**: The spec should explicitly state that the engine supports arbitrary mesh topologies via per-agreement independent sync domains with transitive forwarding.

2. **Node type as conflict resolution anchor**: The spec currently says "Conflict resolution per data type" (section 2.5). This is correct but needs to be formalized: conflict resolution policies are defined on Node labels, not on sync agreements. Agreements define scope and capabilities. Node labels define merge semantics.

3. **Transport abstraction**: The spec should acknowledge that different sync relationships may use different transports with different performance characteristics. The engine provides the sync protocol; the transport is pluggable.

4. **Deduplication as a first-class concern**: The spec should define the deduplication key for changes flowing through the mesh. Without this, any topology with shared nodes and multiple paths produces duplicate application.

The two-peer model in the current documents is a valid starting point. But the mesh scenario is not a future extension -- it IS the vision ("every person, family, or organization runs their own instance"). The architecture must be mesh-native from the start, even if the implementation starts with two-peer sync.

---

## Sources

- [Holochain DHT Architecture](https://developer.holochain.org/concepts/4_dht/)
- [Holochain Kitsune2 Performance](https://blog.holochain.org/dev-pulse-148-major-performance-improvements-with-0-5/)
- [AT Protocol Federation Architecture](https://docs.bsky.app/docs/advanced-guides/federation-architecture)
- [AT Protocol Relay Sync Updates](https://docs.bsky.app/blog/relay-sync-updates)
- [AT Protocol Relay Operations](https://atproto.com/blog/relay-ops)
- [Matrix State Resolution v2](https://matrix.org/docs/older/stateres-v2/)
- [Matrix Specification](https://spec.matrix.org/latest/)
- [Matrix Project Hydra: Improving State Resolution](https://matrix.org/blog/2025/08/project-hydra-improving-state-res/)
- [Gun.js Architecture](https://github.com/amark/gun)
- [GunDB Explained](https://genosdb.com/gundb)
- [CRDT Implementation Guide 2026](https://oneuptime.com/blog/post/2026-01-30-crdt-implementation/view)
- [Cascading Complexity of Offline-First Sync](https://dev.to/biozal/the-cascading-complexity-of-offline-first-sync-why-crdts-alone-arent-enough-2gf)
- [CRDT-Based Game State Synchronization in P2P VR](https://arxiv.org/abs/2503.17826)
- [Git Sync Between Multiple Remotes](https://dev.to/knitex/how-to-sync-git-repositories-a-complete-guide-to-syncing-between-different-remote-repositories-2m0a)
