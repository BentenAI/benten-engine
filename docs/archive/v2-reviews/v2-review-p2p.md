# Review: Benten Platform Specification v2 -- P2P & Sync Correctness

**Date:** 2026-04-13
**Reviewer:** Correctness & Edge Cases Reviewer
**Scope:** Does v2 address the v1 P2P critique (4/10) and the mesh sync critique? Are the new networking constructs (member-mesh, execution assignment, EMIT delivery, triangle convergence, CRDT commit DAG, Atriums/Gardens/Groves) specified with enough rigor to build from?
**Score: 5.5/10 -- v2 acknowledges every problem from v1 but solves very few of them at specification depth.**

---

## Part A: Does v2 Address the v1 P2P Critique?

The v1 critique scored 4/10 and identified 4 must-fix items (M-1 through M-4), 4 should-fix items (S-1 through S-4), and 3 consider items (C-1 through C-3). I trace each one.

### M-1: Write a Sync Protocol section
**v1 asked for:** Serialization format, delta computation algorithm, handshake sequence, session lifecycle, error handling. "2-3 pages minimum."
**v2 delivers:** Section 3.2 is 19 lines. It names CBOR (Section 2.11 standards table), names HLC for conflict resolution, names per-agreement outbound queues for fan-out, names a deduplication key for triangle convergence, and names Merkle Search Trees (standards table, Phase 2). It does NOT specify: wire format for sync messages, handshake sequence, session lifecycle, delta computation algorithm, error handling, resumption, backpressure, or acknowledgment.
**Verdict: Partially addressed.** The spec now names the right building blocks (CBOR, HLC, Merkle trees, dedup key). But "naming" is not "specifying." The v1 critique's table of 9 required sync protocol elements still has 7 missing. The gap between "we will use CBOR" and "here is the sync message schema" is where implementations fail.

### M-2: Define subgraph boundaries
**v1 asked for:** Pick an approach (traverse-based, label-based, explicit membership, hybrid), specify it, analyze failure modes.
**v2 delivers:** Section 3.2 says "subgraph boundary (traversal pattern)" in one parenthetical. No further definition.
**Verdict: Not addressed.** The mesh sync critique dedicated 4 pages to this (Section 3: Selective Sync) and concluded that traverse-based boundaries are novel, risky, and should start with explicit membership. v2 does not engage with this at all. This is the single most important design decision for sync, and it remains a parenthetical.

### M-3: Specify the trust anchor
**v1 asked for:** How root capabilities are created, how they are verified cross-instance, what happens when revoked.
**v2 delivers:** Section 2.8 says capabilities are "UCAN-compatible" and "same system for local modules, remote instances, AI agents." Section 3.3 defines a three-layer identity stack (KERI AID or did:plc for persistent identity, did:key for transport). Section 2.6 specifies BLAKE3 + Ed25519 for signatures.
**Verdict: Substantially improved but incomplete.** v2 picks the trust anchor: self-sovereign keys (did:key/Ed25519) backed by a persistent identity (KERI AID or did:plc). This is the "self-sovereign roots with optional DID verification" that v1 recommended. But the spec still does not specify: how cross-instance verification works in practice, what the UCAN chain looks like (root issuance -> attenuation -> presentation -> verification), or the revocation protocol. The identity stack is the right shape; the verification protocol is still missing.

### M-4: Separate SyncProtocol from SyncTransport
**v1 asked for:** Define the boundary between "what the engine owns" (sync logic) and "what the networking layer owns" (transport).
**v2 delivers:** Section 2.10 lists `benten-sync` as an additional crate for "CRDT merge, sync protocol." Section 3 (Networking) handles identity and the member-mesh model. Phase 3 of the build order separates "CRDT merge for version chains" from "libp2p integration."
**Verdict: Partially addressed.** The crate boundary implies a separation, but there is no interface definition. v1 asked for `SyncProtocol` (computeDelta, applyDelta, verifyCapability) and `SyncTransport` (connect, send, receive, disconnect) as explicit interfaces. v2 does not define either. The separation is implied by crate structure, not specified by API contract.

### S-1: Fork semantics
**v1 asked for:** Snapshot consistency guarantees, fork point recording, re-merge after fork.
**v2 delivers:** Section 2.5 says "Fork = stop syncing, keep all version history." One sentence. Version chains "branch into commit DAGs on concurrent edits" (Section 3.2).
**Verdict: Minimally improved.** The commit DAG concept (version chains branching) is the right mechanism for fork point recording. But v1's specific questions -- snapshot consistency at fork time, mid-sync fork behavior, re-merge protocol -- are unanswered.

### S-2: Partition reconciliation
**v1 asked for:** Merge size bounds, tombstone GC, clock drift tolerance, large-delta streaming.
**v2 delivers:** Nothing explicit. Merkle Search Trees (standards table) imply efficient delta computation. No mention of tombstone GC, clock drift, or streaming.
**Verdict: Not addressed.**

### S-3: Schema evolution during sync
**v1 asked for:** Take a position: strict match, negotiation, or schema-aware merge.
**v2 delivers:** Section 3.2 says "Schema validation on receive." One clause.
**Verdict: Minimally addressed.** "Validation on receive" implies strict match (reject if schema mismatch), which is the simplest and safest position. But it is not stated explicitly, and the consequences are not explored: what happens when validation fails? Is the entire sync rejected? Just the offending Node? Is there a negotiation protocol to resolve schema differences?

### S-4: Privacy model
**v1 asked for:** Tiered encryption, search/encryption tradeoff, what "user-owned data" means.
**v2 delivers:** Nothing. The word "encryption" does not appear in the spec. Ed25519 is listed for signatures, not encryption.
**Verdict: Not addressed.** This is worse than v1, which at least had "optional E2EE." v2 has nothing.

### C-1: DID-based instance identity
**v2 delivers:** Section 3.3 specifies did:key for transport identity and KERI AID or did:plc for persistent identity.
**Verdict: Fully addressed.**

### C-2: Merkle-verified subgraph snapshots
**v2 delivers:** Section 2.6 specifies BLAKE3 hashing with Merkle trees for sync. Standards table lists Merkle Search Trees for Phase 2.
**Verdict: Fully addressed.**

### C-3: Operation log
**v2 delivers:** Version chains (Section 2.5) serve this purpose -- every mutation creates a version Node. The commit DAG IS the operation log.
**Verdict: Addressed via version chains.**

### V1 Critique Scorecard

| Item | Status | Notes |
|------|--------|-------|
| M-1 Sync protocol | Partial | Building blocks named, protocol unspecified |
| M-2 Subgraph boundaries | Not addressed | Still a parenthetical |
| M-3 Trust anchor | Substantial | Identity stack good, verification protocol missing |
| M-4 SyncProtocol/SyncTransport | Partial | Crate separation implied, no interface spec |
| S-1 Fork semantics | Minimal | Commit DAG concept present, mechanics absent |
| S-2 Partition reconciliation | Not addressed | No tombstone GC, drift, streaming |
| S-3 Schema evolution | Minimal | "Validation on receive" but no error handling |
| S-4 Privacy/encryption | Not addressed | Worse than v1 |
| C-1 DID identity | Fully addressed | Three-layer stack |
| C-2 Merkle snapshots | Fully addressed | BLAKE3 + Merkle trees |
| C-3 Operation log | Addressed | Version chains serve this purpose |

**Summary:** 3 of 11 items fully addressed (all from the "Consider" tier). 3 partially addressed. 3 not addressed at all. 2 minimally addressed. The most critical gap (M-2: subgraph boundaries) has not moved at all. The spec improved on identity and content-addressing (the crypto primitives) but did not improve on protocol mechanics (the actual sync behavior).

---

## Part B: Does v2 Address the Mesh Sync Critique?

The mesh sync critique identified 3 CRITICAL gaps, 3 HIGH gaps, and 2 MEDIUM gaps.

### CRITICAL 1: Fan-out writes
**Critique asked for:** SyncStore interceptor with per-agreement queues, synchronous ChangeRecord creation.
**v2 delivers:** "A write to a Node in 3 sync scopes notifies all 3 peers via per-agreement outbound queues." One sentence.
**Verdict: Acknowledged, not specified.** v2 says the right words ("per-agreement outbound queues") but does not specify whether fan-out is synchronous or asynchronous, how the membership index is maintained, or what the ChangeRecord structure looks like. The critique's specific recommendation (synchronous for correctness, with batching) is not addressed.

### CRITICAL 2: Multi-writer conflict resolution
**Critique asked for:** Move resolution policy to Node type definitions, not per-agreement. Add deterministic tiebreaker for irreconcilable conflicts.
**v2 delivers:** "Node properties: per-field last-write-wins with Hybrid Logical Clocks" and "Edges: add-wins with per-edge-type policies." Also: "capability revocation MUST win."
**Verdict: Partially addressed.** The conflict policy is stated at the Node/Edge level (not per-agreement), which is the right anchor. The critique's specific concern -- what happens when 3 writers modify the same Node through different agreements -- is answered implicitly by LWW (highest HLC wins regardless of which agreement carried the change). But the spec does not acknowledge the three-way scenario or explain why LWW is sufficient. The "per-edge-type policies" clause creates ambiguity: who defines these policies? Per Node type? Per agreement? Per community governance? The critique recommended deterministic tiebreaker as fallback; HLC + instance ID hash is implied but not stated.

### CRITICAL 3: Triangle convergence
**Critique asked for:** Origin-tagged transitive forwarding with deduplication.
**v2 delivers:** "Deduplication key = (originInstance, originHLC, nodeId). Every instance forwards received changes to all agreements containing that Node."
**Verdict: Addressed at specification depth.** This is the clearest improvement in v2. The dedup key is explicit, the forwarding rule is stated, and the behavior matches the critique's recommendation (Matrix-model). The one gap: there is no mention of dedup storage (the critique recommended a unique constraint on the ChangeLog table) or garbage collection of dedup records.

### HIGH 1: Per-peer sync state management
**Critique asked for:** Persist membership index as relational table, partition ChangeLog by agreement.
**v2 delivers:** "Per-subgraph sync state: For each peer, track: last synced version, subgraph boundary, capability grants, online/offline status."
**Verdict: Acknowledged, not specified.** v2 lists what to track but not how or where. The critique's specific data model (SyncAgreement Node + MembershipIndex table + partitioned ChangeLog + SyncMetadata table) is not present.

### HIGH 2: Selective sync (dynamic boundary evolution)
**Critique asked for:** Start with explicit membership, lazy boundary re-evaluation.
**v2 delivers:** "Subgraph boundary (traversal pattern)" -- same parenthetical as before.
**Verdict: Not addressed.**

### HIGH 3: Peer lifecycle protocol
**Critique asked for:** invite, accept, update-boundary, update-capabilities, fork, rejoin lifecycle messages.
**v2 delivers:** Nothing explicit. Fork is mentioned. No handshake, invitation, or lifecycle protocol.
**Verdict: Not addressed.**

### MEDIUM 1: Offline accumulation / bulk sync
**Critique asked for:** Two-tier sync (delta for short disconnections, snapshot for long). Metadata-first for binary data.
**v2 delivers:** Merkle Search Trees (standards table) imply efficient delta computation. No mention of snapshot mode, thresholds, or binary data handling.
**Verdict: Not addressed.**

### MEDIUM 2: Bandwidth priority per sync scope
**Critique asked for:** Per-agreement transport configuration with priority levels.
**v2 delivers:** Nothing.
**Verdict: Not addressed.**

---

## Part C: New Constructs Assessment

### C-1: Member-Mesh Model (Section 3.1)

The member-mesh model states: communities are distributed copies across members' instances, no central server, availability depends on at least one member being online.

**Issue: The availability claim is misleading for small communities.** "As long as any member is online" is true in the abstract. In practice, a 5-person Digital Garden where members are in the same timezone has zero availability for 8 hours per night. The spec acknowledges this ("rent a persistent node from the compute marketplace") but frames it as optional. For any community that expects reliability -- which is all of them -- an always-on node is effectively mandatory. This means the member-mesh model degrades to "every serious community needs a server, but we call it a member." The cost model ("each member pays for their own storage/bandwidth") hides the reality that someone pays for the always-on node. The spec should be honest: the member-mesh model provides censorship resistance and data sovereignty; it does NOT provide reliability without infrastructure. Calling a rented server "just another member" is technically accurate but economically misleading.

**Issue: New member bootstrap.** When a new member joins a 1,000-member community, they need the full community graph. Who provides it? If every member has a full copy, any online member can serve the initial sync. But serving a full graph snapshot to a new member is expensive (bandwidth, CPU for Merkle tree computation). With many concurrent joins, the serving members bear disproportionate cost. The spec does not address bootstrap load distribution or whether partial sync (fetch on demand) is supported.

### C-2: Execution Assignment Policy (Section 3.2)

The spec defines three execution policies: `origin-only`, `local`, `leader-elected`.

**This is the right taxonomy.** Origin-only for side effects that must happen once (send email), local for reads and view maintenance (each instance maintains its own materialized views), leader-elected for consensus operations (governance votes).

**Issue: Leader election is not specified.** "Leader-elected" requires a leader election protocol. Who elects? How? What happens when the leader goes offline? Is this Raft? Paxos? A simpler approach (highest-uptime member wins)? For a mesh of 5-50 members, full consensus protocols are overkill. The spec should specify the mechanism or at least scope the complexity.

**Issue: Origin-only is fragile in a mesh.** If the origin instance executes a handler and then goes offline before the result syncs, the result is lost. Other instances know the event happened (it is in the graph) but the handler's side effects (EMIT, external API call) did not propagate. The spec does not address this failure mode. Should another instance retry? How does it know the original execution failed vs. is still in progress?

### C-3: EMIT Delivery Modes (Section 2.2)

The EMIT primitive has three delivery modes: `local`, `exactly-once`, `broadcast`.

**Issue: `exactly-once` is a distributed systems lie.** In a distributed system without a central coordinator, exactly-once delivery is impossible (this is a fundamental result). You can have at-most-once or at-least-once. "Exactly-once" in practice means "at-least-once delivery with idempotent receivers." The spec should either:
1. Define exactly-once as "at-least-once + deduplication" and specify the dedup mechanism, or
2. Replace with `at-least-once` and require idempotent handlers.

The current spec presents exactly-once as a primitive property of EMIT, which misrepresents the distributed systems reality.

**Issue: `broadcast` scope is undefined.** Broadcast to whom? All members of the community? All connected peers? All agreements? The scope determines the cost. Broadcasting to a 10,000-member Grove is a very different operation than broadcasting to 3 Atrium peers.

### C-4: Triangle Convergence (Section 3.2)

As noted in Part B, this is well-specified: dedup key = (originInstance, originHLC, nodeId), transitive forwarding to all agreements containing the affected Node.

**Issue: Convergence speed is unbounded.** The spec guarantees eventual convergence (all instances will eventually have the same state) but says nothing about convergence time. In a chain topology (A -> B -> C -> D), a change from A reaches D after 3 sync rounds. If each round is 30 seconds, convergence takes 90 seconds. For a 100-member chain (unlikely but possible), convergence is 50 minutes. The spec should state whether there is a maximum convergence time guarantee and under what topology assumptions.

**Issue: Forwarding creates amplification.** In a fully-connected mesh of N members sharing the same Node, a single write produces N-1 direct deliveries. Each recipient forwards to N-2 other agreements (all agreements except the one that delivered it). Total messages: N-1 + (N-1)(N-2) = O(N^2). The dedup key prevents duplicate application but not duplicate transmission. For a 100-member community, a single write generates ~10,000 messages. The spec should address message amplification limits.

### C-5: CRDT Commit DAG for Version Chains (Section 2.5 + 3.2)

The spec says version chains "branch into commit DAGs on concurrent edits." This is the right model -- it is how Git works, how AT Protocol works, and how NextGraph works.

**Issue: DAG merge is not specified.** A commit DAG branches when two instances edit the same anchor concurrently. At some point, the branches must merge back to a single CURRENT version. Who performs the merge? When? The spec says "per-field LWW with HLC" for conflict resolution, which defines the merge semantics, but not the merge trigger. Does merge happen on sync receive? On explicit reconciliation? Automatically?

**Issue: DAG growth is unbounded.** If two instances edit concurrently 100 times before syncing, the DAG has 200 branches from a single fork point. Merging 200 concurrent versions into one is computationally expensive (100 pairwise field comparisons for each branch). The spec should address whether the DAG is compacted (old versions pruned after merge) and what the maximum practical branch factor is.

### C-6: Atriums / Gardens / Groves (Section 1.2)

The three-tier model is clear and well-motivated.

**Issue: The boundaries between tiers are social, not technical.** An Atrium is "private, selective, bidirectional sync." A Garden is "community space, each member syncs locally." A Grove is "governed community with voting." Technically, all three are the same thing: a set of sync agreements between instances with capability grants. The difference is only in the governance configuration (no governance, admin governance, formal governance). This is actually fine -- the tiers are user-facing categories, not engine-level distinctions. But the spec does not state this explicitly. It should say: "Atriums, Gardens, and Groves are configuration presets on the same underlying sync+governance mechanism. The engine does not distinguish between them."

**Issue: Atrium to Garden promotion.** What happens when an Atrium (2 friends) grows to 10 people? Is it still an Atrium or should it become a Garden? If the spec says "the engine does not distinguish," this is a non-issue (it is just a label). But the governance implications are real: 10 people sharing data without governance is a recipe for conflict. The spec should address whether tier promotion triggers any governance defaults.

---

## Part D: Issues Summary

### Issue 1 (CRITICAL): Subgraph boundary definition is still absent

This was M-2 in v1. It remains a parenthetical ("traversal pattern"). Without a subgraph boundary definition, sync scope is undefined, selective sync is impossible, and capability enforcement cannot be scoped. This is load-bearing for the entire networking story. The spec must define: what constitutes a subgraph boundary, how boundaries are evaluated (eagerly or lazily), how dynamic membership changes propagate, and what happens when a boundary traversal reaches Nodes the evaluator lacks capability to read.

### Issue 2 (CRITICAL): Encryption/privacy model deleted rather than improved

v1 had "optional E2EE" and the critique asked for a tiered model. v2 has nothing. The word "encryption" does not appear. For a platform whose core promise is "your data lives on YOUR instance" and "you choose who sees what," the absence of any privacy specification is a fundamental gap. At minimum, the spec needs: transport encryption (TLS or noise protocol via libp2p), at-rest encryption policy, and a statement on whether E2EE is supported per-subgraph.

### Issue 3 (HIGH): EMIT `exactly-once` is not achievable as specified

Exactly-once delivery in a distributed system without a central coordinator is impossible. The spec presents it as a primitive delivery mode. This will either be silently downgraded to at-least-once (breaking the contract) or require a central coordinator (breaking the decentralization promise). The spec should replace with `at-least-once` + idempotency requirement, or specify the deduplication/acknowledgment protocol that makes exactly-once practical.

### Issue 4 (HIGH): Leader election for `leader-elected` execution policy is unspecified

The execution assignment taxonomy is correct but "leader-elected" requires a consensus protocol. The spec names neither the algorithm nor the failure handling. For a mesh of 5-50 members, even a simple approach (longest-uptime member, with failover) needs specification: how is uptime measured across instances with no shared clock? How is leader change communicated? What is the election latency?

### Issue 5 (HIGH): Sync protocol remains at bullet-point level

v1 asked for 2-3 pages. v2 delivers 19 lines. The right building blocks are named (CBOR, HLC, Merkle trees, dedup key, per-agreement queues), but the protocol -- the sequence of messages exchanged between two instances to perform a sync -- is absent. Without this, two independent implementations will be incompatible. A sync protocol specification needs: message types (propose-sync, offer-delta, accept-delta, acknowledge, reject), message ordering constraints, error states, and resumption after interruption.

### Issue 6 (HIGH): Message amplification in transitive forwarding

In a fully-connected N-member community, a single write generates O(N^2) forwarding messages. Dedup prevents duplicate application but not duplicate transmission. For communities above ~50 members, this becomes a bandwidth problem. The spec should either: (a) define a gossip protocol (GossipSub is in the standards table but not connected to sync), or (b) define forwarding limits (e.g., a member forwards to at most K peers, relying on those K to forward further), or (c) designate specific "relay" members who handle forwarding so regular members only send to relays.

### Issue 7 (MEDIUM): Member-mesh availability is overstated

"As long as any member is online, the community is accessible" is true but misleading. Small communities in the same timezone have predictable multi-hour outages. The spec should quantify: for a community of N members with typical online patterns, what is the expected availability? And state clearly that always-on nodes are required for production reliability, not optional extras.

### Issue 8 (MEDIUM): DAG compaction / version chain garbage collection

Version chains grow monotonically. In a community with 1,000 Nodes edited daily for a year, the version chain for each Node could have 365+ versions. The DAG branches on every concurrent edit. Without compaction, graph storage grows without bound. The spec should define: when old versions can be pruned, how pruning interacts with sync (can a peer request a version that has been pruned?), and what the minimum retention policy is.

### Issue 9 (MEDIUM): Schema validation on receive -- failure mode unspecified

The spec says "schema validation on receive" but does not define what happens when validation fails. Is the entire sync session aborted? Is only the invalid Node rejected? Does the sender get notified? Can the receiver quarantine the Node for manual review? The failure mode determines whether schema mismatches are recoverable or catastrophic.

---

## Part E: Score Justification

| Category | v1 Score | v2 Score | Weight | Rationale |
|----------|----------|----------|--------|-----------|
| Local engine (IVM, versions, evaluator) | 8/10 | 9/10 | 25% | Code-as-graph, 12 primitives, version chains, content hashing -- significantly more rigorous |
| Sync protocol | 2/10 | 4/10 | 25% | Building blocks named, dedup key specified, but protocol mechanics still absent |
| Capability / trust model | 3/10 | 6/10 | 20% | Three-layer identity stack, UCAN adoption, capability-not-tiers. Verification protocol still missing |
| Fork / partition semantics | 1/10 | 3/10 | 15% | Commit DAG concept present, but fork mechanics / partition reconciliation absent |
| Privacy / encryption | 3/10 | 1/10 | 10% | Regressed. v1 had "optional E2EE"; v2 says nothing |
| Mesh topology (new) | N/A | 5/10 | 5% | Triangle convergence good, amplification and availability overstated |

**Weighted score: 0.25(9) + 0.25(4) + 0.20(6) + 0.15(3) + 0.10(1) + 0.05(5) = 2.25 + 1.0 + 1.2 + 0.45 + 0.1 + 0.25 = 5.25, rounded to 5.5.**

The score improved from 4 to 5.5 because v2 has a dramatically better local engine story (code-as-graph, 12 primitives, structural invariants) and improved identity/trust primitives. The sync protocol improved from hand-waving to bullet points. But bullet points are not a protocol specification, and the deletion of the encryption model is a regression. The v1 critique's core complaint -- "the spec is thorough on the local-instance story and vague on the decentralized story" -- remains true in v2, though the gap has narrowed.

---

## Recommendations

### Must-Fix (Before Phase 3 implementation)

1. **Specify subgraph boundaries.** This cannot remain a parenthetical. Define the data structure (explicit node set? traversal pattern? both?), the evaluation lifecycle (eager vs lazy), the membership change protocol, and the interaction with capabilities.

2. **Write the sync protocol.** Define message types, message ordering, handshake sequence, error states, and resumption. Even 2 pages would be a massive improvement. The building blocks are all chosen (CBOR, HLC, Merkle Search Trees, BLAKE3) -- now compose them into a protocol.

3. **Add an encryption section.** At minimum: libp2p noise protocol for transport encryption, per-instance at-rest encryption recommendation, and a statement on whether per-subgraph E2EE is planned. The current spec implies data travels between instances in the clear.

### Should-Fix (Before first sync release)

4. **Replace `exactly-once` EMIT with `at-least-once` + idempotency.** Or define the dedup mechanism that achieves the exactly-once illusion.

5. **Specify leader election for `leader-elected` execution policy.** Even a simple "longest-uptime with heartbeat failover" would suffice.

6. **Address message amplification.** Connect GossipSub (already in the standards table) to the transitive forwarding model. GossipSub is specifically designed to limit amplification in mesh topologies.

7. **Define DAG compaction.** When can old versions be pruned? How does pruning interact with sync? What is the minimum retention?
