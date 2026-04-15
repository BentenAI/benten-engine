# Benten Platform Design Document

**Created:** 2026-04-13
**Last Updated:** 2026-04-14 (Atrium sync marked Phase 3 committed; Gardens MVP Phase 7 committed; broader compute marketplace stays exploratory)
**Status:** WORKING DRAFT -- architecture defined, protocol details require specification before Phase 3 implementation. Sections 6-8 captured exploratory directions during vision evolution; the ones that became committed (Atriums → Phase 3, Gardens MVP → Phase 7, Credits → Phase 8) are noted inline. Full Groves and broader marketplace remain exploratory.
**Audience:** Platform architects, protocol engineers, and governance designers who need to understand the system above the engine layer.
**Related documents:** [Engine Spec](./ENGINE-SPEC.md) (Rust engine internals, 12 primitives, IVM, capabilities) | [Business Plan](./BUSINESS-PLAN.md) (economics, revenue, regulatory)

---

## 1. Vision

### 1.1 What Benten Is

Benten is a self-evaluating graph -- a platform where data and code are both Nodes and Edges, the graph evaluates itself, and every person, family, or organization can own their data, share it selectively, and fork at any time.

It is not a database. It is not a CMS. It is not a blockchain. It is the foundation of a new web where:
- Your data lives on YOUR instance
- You choose who sees what through capability grants
- Communities form, federate, fork, and compete on governance quality
- Knowledge is curated through speculative attestation markets
- AI agents are first-class citizens operating within capability boundaries
- The entire system runs as a self-evaluating graph with zero distinction between "application" and "database"

### 1.2 The Three Tiers

**Atriums** -- Peer-to-peer direct connections. Partners sharing finances, friends planning a trip, a student syncing with a school. Private, selective, bidirectional sync of chosen subgraphs. Each peer pays only for their own compute/storage.

**Digital Gardens** -- Community spaces. Like a Discord server, a Wikipedia, or a knowledge base -- but decentralized. Each member syncs the community graph locally. No central server required. Admin/moderator governance configures capabilities, moderation rules, and content policies. A Garden's character depends on its purpose -- a casual hangout, a curated knowledge base, a professional network.

**Groves** -- Governed communities. Fractal, polycentric, polyfederated governance. Voting on rules, smart contracts as operation subgraphs, formal decision-making. Sub-Groves with inherited or overridden governance. Fork-and-compete dynamics -- communities compete on governance quality.

**Technical note:** Atriums, Gardens, and Groves are configuration presets on the same underlying sync + governance mechanism. The engine does not distinguish between them at the protocol level. The difference is in the governance configuration: no formal governance (Atrium), admin-configured governance (Garden), or community-voted governance (Grove). Promotion from one tier to another is a governance configuration change, not a migration.

### 1.3 Core Principles

1. **The graph IS the runtime.** No distinction between database and application. Data and code are both Nodes and Edges. See [Engine Spec, Section 2](./ENGINE-SPEC.md) for the self-evaluating graph architecture.
2. **Capabilities, not tiers.** Operator-configured, UCAN-compatible capability grants. Same system for local modules, remote instances, AI agents.
3. **History IS the graph.** Version chains, not revision tables. Every mutation creates a version Node. Undo, audit, and time-travel are graph traversals.
4. **Content-addressed integrity.** Every version Node is hashed. Merkle trees for efficient sync. Verifiable knowledge, verifiable code, verifiable governance.
5. **Fork is a right.** Any participant can fork any subgraph at any time, keeping full history. This creates evolutionary pressure on governance.
6. **Governance is the competitive dimension.** Communities differentiate through their governance model, not their technology. The platform makes governance trivially configurable and forkable.
7. **Zero-fee transactions.** The platform currency has no transaction fees. Revenue comes from treasury interest on reserves, not from taxing users. See [Business Plan](./BUSINESS-PLAN.md) for the economic model.
8. **AI-native.** AI agents discover, operate, and reason through the same graph that humans use. The graph is self-describing and inspectable.

---

## 2. Migration from Thrum V3

### 2.1 What Carries Forward

- CMS domain code (content types, blocks, compositions, field types)
- SvelteKit web app (UI components, routes, pages)
- Module definitions (adapted to operation subgraphs)
- Test infrastructure patterns
- 3,200+ behavioral test expectations (the contracts, not the implementations)

### 2.2 What Gets Replaced

| Current (Thrum V3) | Replacement (Benten Engine) |
|--------------------|-----------------------------|
| In-memory registries | IVM materialized views |
| Event bus | Reactive subscriptions + EMIT primitive |
| PostgreSQL + AGE | Benten engine native storage |
| RestrictedEventBus + TIER_MATRIX | Capability enforcement |
| Trust tiers | Capability grants |
| compositions.previous_blocks | Version chains |
| content_revisions table | Version Nodes |
| module_settings | Settings as graph Nodes |

---

## 3. Networking

**Status note (2026-04-14):** Atrium-tier P2P sync is Phase 3 **committed** scope — two or more trusted peers share subgraphs. Digital Gardens MVP (community spaces beyond Atriums with admin governance) is Phase 7 committed. Full Groves (fractal/polycentric governance) remains exploratory. See [`research/explore-gardens-mvp.md`](research/explore-gardens-mvp.md) for the Gardens MVP scoping.

### 3.1 Member-Mesh Model

Communities are NOT hosted on servers. They are the distributed copies across members' instances. Each member syncs the community graph locally. There is no single point of failure or control.

- **Availability:** As long as any member is online, the community is accessible.
- **Scaling:** More members = more copies = more capacity for new syncs.
- **Cost:** Each member pays for their own storage/bandwidth. No pooled hosting.
- **Always-online nodes:** Communities that want reliability can rent a persistent node from the compute marketplace -- it's just another member that happens to be always-on.

**Honest availability assessment:** The member-mesh model provides censorship resistance and data sovereignty. It does NOT provide reliability without sufficient peer coverage.

The correct formulation: **every community needs at least one peer online at any given moment** -- not "a dedicated always-on peer." Availability is emergent from the union of member activity patterns. A community with enough members across enough time zones is naturally always-reachable without anyone running dedicated infrastructure. A 5-person community in the same timezone will experience gaps during sleeping hours; these cause temporary desync or splitting that self-heals when any member comes back online (graceful degradation, not binary failure).

For production reliability of smaller or time-zone-concentrated communities, always-on coverage can be provided by: a dedicated member choosing to run their device 24/7, opportunistic sync from old phones on postmarketOS, rented edge-node peers on the compute marketplace, or any combination. The cost model accounts for this: coverage is market-priced, not assumed free.

**New member bootstrap:** When a new member joins, they need the community graph. Any online member can serve the initial sync via Merkle tree comparison and delta transfer. For large communities, bootstrap load should be distributed across multiple serving members (round-robin or load-aware selection) to prevent bandwidth concentration on a single member.

### 3.2 Sync Protocol

Sync = exchange version Nodes newer than the peer's last known state.

**Per-subgraph sync state:** For each peer, track: last synced version, subgraph boundary (traversal pattern), capability grants, online/offline status.

**Fan-out writes:** A write to a Node in 3 sync scopes notifies all 3 peers via per-agreement outbound queues.

**Conflict resolution:**
- Node properties: per-field last-write-wins with Hybrid Logical Clocks (non-deterministic values captured in version chain, not replayed)
- Edges: add-wins with per-edge-type policies (capability revocation MUST win)
- Version chains branch into commit DAGs on concurrent edits
- Schema validation on receive
- Move = atomic CRDT operation (not decomposed to delete+create)

**Clock validation:** Receiving peers MUST reject HLC timestamps that are more than a configurable delta (default: 5 minutes) ahead of the local clock. This prevents the "monotonic clock advantage" attack where a malicious peer sets its clock far into the future to always win LWW comparisons. Timestamps beyond the tolerance threshold trigger a sync pause and clock reconciliation handshake.

**Per-peer rate limits:** The sync protocol enforces per-peer write rate limits. A peer that generates more than a configurable number of operations per time window is throttled. This prevents operation flooding attacks (including edge spam via add-wins semantics). Rate limit configuration is per-sync-agreement.

**Execution assignment:** Handlers have an `executionPolicy`: origin-only (only the instance that triggered the event), local (each instance runs independently), leader-elected (one designated instance runs, others receive results).

**Triangle convergence:** Deduplication key = (originInstance, originHLC, nodeId). Every instance forwards received changes to all agreements containing that Node.

**Message amplification mitigation:** In a fully-connected N-member community, naive transitive forwarding produces O(N^2) messages per write. To control amplification: connect GossipSub (in the standards table) to the forwarding model. GossipSub limits message propagation to a configurable fanout parameter (default: 6) while ensuring eventual delivery to all members. For communities above ~50 members, GossipSub replaces direct peer-to-peer forwarding.

### 3.3 Sync Protocol Detail (To Be Specified)

The building blocks are chosen (CBOR serialization, HLC ordering, Merkle Search Trees for delta computation, BLAKE3 for integrity, per-agreement queues for fan-out, dedup key for triangle convergence). Before Phase 3 implementation, a full sync protocol specification must define:

| Element | Status | Notes |
|---------|--------|-------|
| Wire format (message schema) | Building block chosen (CBOR) | Need: message type definitions, field layouts |
| Handshake sequence | Not specified | Need: capability presentation, clock reconciliation, scope negotiation |
| Delta computation algorithm | Building block chosen (Merkle Search Trees) | Need: tree construction rules, comparison protocol |
| Session lifecycle | Not specified | Need: initiate, delta exchange, acknowledge, terminate, error states |
| Resumption after interruption | Not specified | Need: checkpoint mechanism, partial delta recovery |
| Backpressure | Not specified | Need: flow control for large deltas, chunk size configuration |
| Schema validation failure handling | "Validation on receive" stated | Need: reject entire sync vs. quarantine individual nodes, sender notification |

**SyncProtocol / SyncTransport separation:** The engine owns sync logic (delta computation, conflict resolution, capability verification, merge application). External crates own transport (connection management, NAT traversal, peer discovery). The boundary is defined by two interfaces:

- `SyncProtocol`: `computeDelta()`, `applyDelta()`, `verifyCapability()`, `recordForkPoint()` -- implemented by the engine
- `SyncTransport`: `connect()`, `send()`, `receive()`, `disconnect()` -- implemented by transport adapters (WebSocket, libp2p, Yggdrasil)

### 3.4 Subgraph Boundaries

**Current status:** Subgraph boundaries are described as "traversal patterns" but the formal definition is not yet specified. This is the single most important design decision for sync and must be resolved before Phase 3.

**Design space:** Traverse-based boundaries (root nodes + edge types + max depth), label-based boundaries (all nodes with a given label), explicit membership (manually curated node sets), or a hybrid. Traverse-based is the most natural for a graph engine but introduces risks: unexpected reachability, ambiguous membership, and expensive boundary evaluation.

**Recommended approach:** Start with explicit membership (a sync agreement lists specific anchor nodes) with traverse-based expansion as an opt-in feature. Explicit membership is simpler to reason about, cheaper to evaluate, and avoids the "reachability surprise" problem where a new edge inadvertently adds nodes to a sync scope.

**Dangling references:** When syncing a subgraph that references nodes outside the sync boundary, edges whose targets are outside the scope are included as "stub" anchor nodes (identity only, no version data). These stubs are tagged as "unresolved" and lazily fetched on traversal if the requesting instance has a sync agreement covering the target node. This prevents orphan edges while avoiding forced inclusion of unwanted data.

### 3.5 Identity

Three-layer identity stack:

| Layer | Purpose | Technology |
|-------|---------|------------|
| Persistent Identity | Survives key rotation | KERI AID or did:plc |
| Transport Identity | Authenticates on transport | did:key (Ed25519) |
| Transport Address | Network reachability | libp2p multiaddr (or Yggdrasil IPv6, optional) |

**Trust anchor:** Each engine instance generates an Ed25519 key pair on first boot. This key pair is the root of trust for the instance. Cross-instance trust is established by exchanging public keys directly (QR code, shared secret, manual verification) or via DID resolution (did:web, did:key). This is the self-sovereign roots model -- each instance is its own root of trust.

**UCAN chain verification:** When Instance B receives a capability grant from Instance A, it verifies: (1) the grant is signed by A's Ed25519 key (signature verification), (2) A had authority to issue the grant (delegation chain walk -- each UCAN in the chain is verified back to a self-issued root), (3) the grant has not been revoked (revocation check against the local revocation list + sync-received revocation records).

**Decentralized verification:** Identity verification is a marketplace of attestations via W3C Verifiable Credentials. KYC providers, communities, organizations, and individuals all issue credentials. Communities decide which verifiers they trust. The same mechanism handles regulatory KYC, professional credentials, social vouching, and community membership.

**Credential audience restrictions:** Verifiable Credentials should include `aud` (audience) claims to prevent credential replay across communities with different trust standards. A "trusted member" credential issued by Community A should specify Community A as the audience; Community B can choose to accept it, but the credential metadata makes cross-community presentation explicit rather than silent.

### 3.6 Fork Semantics

**Fork = consistent snapshot + governance divergence.** Forking a community creates a snapshot of the community's graph at the latest fully-synced version. The fork point is recorded as a "fork marker" Node with a reference to the version clock at the time of fork. Mid-sync forks are not permitted: the system must complete or roll back the current sync before allowing a fork.

**Fork integrity:** The forked graph includes all nodes and edges in the sync scope at the fork point. Edges referencing nodes outside the scope are included as stubs (same as the dangling reference rule in Section 3.4).

**Re-merge after fork:** Fork points enable future re-merge. The system can compute the delta between the fork point and the current state of either branch. Re-merge follows the same conflict resolution rules as normal sync (per-field LWW, add-wins for edges, capability revocation wins). Re-merge is a mutual decision -- both forks must agree to re-sync.

**Fork-and-compete:** Forking a community = syncing its graph + modifying governance parameters + publishing. The fork inherits all content and history. Members choose which governance model they prefer. Evolutionary pressure optimizes governance -- communities that govern well retain members.

### 3.7 Partition Reconciliation

**Delta computation:** Delta size is O(changes since last sync), not O(subgraph size). Version vectors track the last synced state per peer. Merkle Search Tree comparison identifies the specific nodes that differ.

**Large deltas:** When two instances reconnect after a long partition, deltas are streamed in configurable chunks with per-chunk acknowledgment. A progress indicator is available to the application layer.

**HLC drift tolerance:** Hybrid Logical Clocks include a configurable maximum drift tolerance (default: 5 minutes). Clocks drifted beyond tolerance trigger a full clock reconciliation handshake before sync proceeds. If reconciliation fails, the instances fall back to full snapshot comparison rather than incremental delta sync.

**Tombstone garbage collection:** Tombstones for deleted edges are retained for a configurable retention period (default: 90 days). After retention, tombstones are compacted into the snapshot. Peers that have not synced within the retention period must perform a full snapshot sync rather than an incremental delta sync. The retention period is configurable per sync agreement.

### 3.8 Encryption and Privacy

**The tension:** The vision states "data is owned by the user" and "you choose who sees what." User-owned data implies the user controls access. Plaintext data on any peer's storage means that peer's operator has access. These are in tension.

**The pragmatic position:** For 95% of use cases (CMS, commerce, community spaces), transparent server-side encryption is sufficient. End-to-end encryption (E2EE) is a premium feature for specific verticals (healthcare, legal, government, personal finance).

| Data Category | Default Protection | Rationale |
|---|---|---|
| Local-only data (never synced) | At-rest encryption (transparent, via OS/filesystem) | Standard. Instance operator can query. |
| Synced data in transit | libp2p noise protocol (transport encryption) + message-level Ed25519 signing | Prevents interception and tampering. |
| Synced data at rest on remote instances | Optional per-subgraph E2EE | The data owner decides. E2EE subgraphs cannot be server-indexed. |
| Capabilities / tokens | Always signed (Ed25519), never encrypted | Must be verifiable by any party in the chain. |

**Search with encrypted data:** E2EE subgraphs are not server-searchable. If a user wants their data encrypted AND searchable, they need a local search index maintained in memory (decrypted data never persisted to disk). The engine could support this via IVM views on decrypted data in memory, but this is a significant feature with security implications -- scoped to a later phase.

**Key management:** Instance identity keys (Ed25519) are stored in a platform-specific secure enclave or keychain (macOS Keychain, Linux Secret Service, Windows DPAPI). Key rotation is supported via KERI's pre-rotation mechanism: the next key is committed cryptographically before the current key is rotated. This allows peers to verify the chain of key rotations and accept the new key without manual re-verification.

---

## 4. Governance

### 4.1 Governance as Code

All governance rules are operation subgraphs -- content-hashed, versionable, forkable, syncable. Voting mechanisms, contribution fees, moderation rules, membership criteria -- all configurable Nodes in the graph. See [Engine Spec, Section 3](./ENGINE-SPEC.md) for the 12 operation primitives that governance subgraphs are composed from.

### 4.2 Configurable Per-Community

Every governance parameter is a Node that communities set through their chosen meta-governance process:
- **Voting mechanism:** 1-person-1-vote, token-weighted, quadratic, conviction, liquid delegation
- **Contribution economics:** Free, small fee, scaled by impact
- **Revenue distribution:** Equal to attestors, proportional to order, flows downstream through knowledge graph
- **Moderation:** Admin-appointed, community-elected, reputation-based, AI-assisted
- **Meta-governance:** How the governance parameters themselves are changed

### 4.3 Fractal Structure

Groves contain sub-Groves. Each sub-Grove inherits parent governance with three override modes:
- **REPLACE:** Full override of a specific rule
- **EXTEND:** Add to parent rules
- **EXEMPT:** Opt out of a specific parent rule

Governance inheritance uses prototypal resolution (like JavaScript's prototype chain). IVM materializes "effective rules" so governance checks are O(1). See [Engine Spec, Section 8](./ENGINE-SPEC.md) for IVM details.

### 4.4 Polycentric Federation

A Grove can have MULTIPLE parent Groves simultaneously (DAG, not tree). Each parent's authority is domain-scoped. Conflicts between parents resolved by: explicit priority, union (strictest wins), local override, or mediation.

**Parent authority limits:** To prevent polycentric authority injection (where a parent Grove imposes restrictive rules on a child through "strictest wins"), parent authority is bounded: a parent can only impose rules within its declared authority domain. A parent with authority over "content moderation" cannot impose rules on "membership criteria." Authority domains are declared when the parent-child relationship is established and can only be narrowed (never widened) after establishment.

### 4.5 Fork-and-Compete

See Section 3.6 for fork mechanics. Fork-and-compete is the governance-level consequence: communities that govern well retain members. Communities that govern poorly lose members to better-governed forks. This creates evolutionary pressure toward good governance.

### 4.6 Governance Attack Resistance

The governance system introduces attack surfaces that must be addressed:

**Liquid delegation capture:** An attacker gradually accumulates delegations from inactive members and uses the accumulated voting power to change governance rules. Mitigation: delegation decay -- delegations expire if not explicitly renewed within a configurable period (default: 90 days). Inactive delegations automatically revert to direct voting.

**Fork-bomb confusion:** An attacker repeatedly forks a community, creating confusion about which fork is "canonical." Mitigation: fork naming conventions -- the fork with the most active members (measured by sync activity in the last 30 days) is the "canonical" fork. Forks are labeled with their lineage (parent fork, fork point, fork reason).

**Polycentric authority injection:** Addressed by parent authority limits (Section 4.4).

**Meta-governance capture:** An attacker modifies the meta-governance rules to prevent future governance changes. Mitigation: meta-governance changes require a supermajority (configurable, default: 2/3) and a mandatory cooling period (configurable, default: 7 days) during which members can fork before the change takes effect.

---

## 5. Platform Build Order

Phases 1-3 are specified in the [Engine Spec, Section 14](./ENGINE-SPEC.md). The platform-focused phases are:

### Phase 4: Platform Features
- Migrate Thrum CMS domain to operation subgraphs
- Schema-driven rendering (materializer pipeline as operation subgraphs)
- Self-composing admin
- AI agent integration (MCP tools as capability-gated operation subgraphs)

### Phase 5: Governance + Economics
- Garden/Grove governance subgraphs
- Configurable voting mechanisms
- Benten Credits (mint/burn, FedNow integration) -- see [Business Plan](./BUSINESS-PLAN.md)
- Knowledge attestation marketplace
- Compute marketplace

### Phase 6: Polish + Ship
- CLI (npx create-benten)
- Documentation
- Edge/serverless deployment
- Performance optimization
- Security audit

**Phase 1 scope boundary:** Phase 1 delivers a single-instance graph engine with persistence, operation evaluation, and capability enforcement. It does NOT include sync, governance, economics, identity federation, or attestation. These are documented here for architectural alignment but are not Phase 1 deliverables.

---

---

## 6. Distributed Storage and Compute (Exploratory, added 2026-04-14)

**Status:** These directions emerged during pre-work vision evolution. They are captured as design intent for Phase 2+ and inform Phase 1 abstraction boundaries, but are not committed specification. Full details: `docs/research/explore-distributed-compute-vision.md`.

### 6.1 Data Has No Home

The original design assumes each user runs a persistent "instance" -- a server or device that owns their data. An alternative framing: the user's data is encrypted, content-addressed, and distributed across the instances of peers and communities they trust. Peers hold ciphertext they cannot read. The user's "instance" is a runtime that materializes the graph from encrypted peer storage on demand, wherever they are.

This does not conflict with the existing architecture. Content addressing already makes data location-independent. Capability grants already define the trust topology. Version chains already handle multi-device writes. The change is in the deployment/persistence model, not the computational model.

### 6.2 Unified Compute Marketplace

Rather than having separate concepts for "always-on nodes," "compute marketplace" (Phase 13), and member-hosted storage, the exploratory model unifies them: every interaction with a peer's hardware is a micro-transaction in Benten Credits. Reads, compute jobs, community serving, and availability are all resource usage paid with Credits. This makes the always-on coverage problem an economic equilibrium rather than an infrastructure mandate.

Enablers:
- Zero-fee credits make granularity viable (no blockchain can match)
- Tab-based periodic net settlement (not payment channels)
- Proof of Sampling verification (5-10% random re-execution with reputation penalty)
- Content-addressed data is self-verifying on reads
- Bounded DAG operation subgraphs are deterministic and cheap to verify

### 6.3 Benten Runtime as Infrastructure Layer

Rather than deploying Benten communities INSIDE proprietary edge runtimes (Cloudflare Workers), the exploratory direction builds Benten's own WinterTC-compliant runtime that anyone can install. Peers become nodes in a peer-distributed edge network. Three products share one engine: Application (communities), Runtime (WinterTC host), Economy (marketplace).

Enablers:
- WinterCG became Ecma TC55 in December 2024 (real cross-platform standard)
- napi-rs v3 compiles to wasm32-wasip1-threads from the same Rust codebase
- Open-source foundations available (Deno, workerd, wasmtime)

### 6.4 `bentend` Peer Daemon for General-Purpose Compute

Rather than building a Linux distribution or reinventing Proxmox/K8s, a single Rust daemon (`bentend`) composes commodity runtimes: containerd for containers, firecracker for VMs, wasmtime for WASM, Nomad-style pluggable drivers. Workloads are graph Nodes with capabilities; the graph IS the control plane.

### 6.5 Mobile Devices' Realistic Role

Primary phones (App Store distributed): consumers and clients only. Local compute for the owner, opportunistic sync when charging+WiFi. "Earn compute" framing is forbidden by Google Play (Oct 2025) and Apple. Old/retired phones running postmarketOS/LineageOS: first-class full peers with no App Store or background restrictions.

---

## 7. Trust Tiers as Composable Primitives (Exploratory, added 2026-04-14)

**Insight:** Trust is not a hierarchy -- it is four orthogonal primitives that workloads declare requirements across.

### 7.1 The Four Primitives

1. **Cryptographic identity gating** (Tailnet Lock, PGP WoT) -- binary, social, no score. Atriums use this: family members sign each other's keys.
2. **Reputation-weighted routing** (EigenTrust, staking+slashing) -- numeric scores, appropriate for open marketplace tier.
3. **TEE remote attestation** (Intel TDX, AMD SEV-SNP, NVIDIA GPU TEE) -- hardware substitutes for social trust. Lets low-reputation peers handle high-sensitivity workloads.
4. **Verifiable Credentials + Soulbound Tokens** -- cryptographic claims for KYC, insurance, jurisdiction, community membership.

### 7.2 Workload Trust Declaration

Each workload declares requirements as a graph Node:
```
trust_requirement: {
  min_tier: atrium | garden | grove | open,
  required_attestation: [TEE vendors],
  required_credentials: [VC schemas],
  min_reputation: number,
  insurance_coverage: amount
}
```

The scheduler (typically the user's AI agent) filters the peer set by intersection. Trust requirements become edges from the workload subgraph. Pricing is reputation-weighted (higher trust = higher price), so the market surfaces tradeoffs rather than hardcoding them.

### 7.3 Key Nuances

- **E2EE does NOT eliminate trust requirements.** Encrypted workloads can still leak execution metadata, be withheld/delayed, or selectively DoS'd. Trust tier matters for liveness even when data is encrypted.
- **TEEs as one signal, not sole gate.** Vendor-key compromise is catastrophic and correlated across peers with same silicon. Treat TEE-attested-open-peer as equivalent to Garden-tier, not Atrium-tier.
- **Trusted third-party provider communities.** Groves can form around providing high-reliability compute: "we verify members' hardware, stake credits as insurance, offer SLAs." Fractal governance accommodates this naturally.

---

## 8. Identity, Key Management, and Device Independence (Exploratory, added 2026-04-14)

### 8.1 The Model

- ed25519 keypair is the root of trust (unchanged from existing design)
- Biometrics (WebAuthn/passkeys) unlock the stored private key; they do not derive keys (fuzzy extractors provide no security for real biometric sources)
- Device mesh: the user's own enrolled devices hold key shares. New device enrollment via QR + ephemeral Diffie-Hellman over local network or relay. No Apple/Google custodial dependency.
- M-of-N Shamir threshold recovery via trusted peers as fallback when all devices are lost (Web3Auth tKey SDK is the most battle-tested implementation)
- Social recovery guardians are graph Nodes with GRANTED_TO edges

### 8.2 Known Unsolved Problem: Guardian UX

Cryptography is mature. Shamir threshold recovery works mathematically. It fails in practice when guardians lose shares, change devices, or become unreachable. No project has made guardian management effortless for non-technical users. This is a UX design challenge for Benten, not a cryptographic one.

### 8.3 Why Synced Passkeys Are Incompatible

iCloud Keychain and Google Password Manager sync passkeys with E2EE, but Apple/Google become key custodians. No decentralized passkey sync exists in 2026 (FIDO Credential Exchange Protocol still in draft). For Benten's "no centralized custodian" requirement, synced passkeys are architecturally incompatible as the root of trust.

---

## 9. Open Questions (Platform-Level)

These are design decisions affecting the platform layers above the engine. Engine-level open questions are in the [Engine Spec, Section 15](./ENGINE-SPEC.md).

1. **Subgraph boundary formalization:** The recommended approach (explicit membership with opt-in traverse expansion, Section 3.4) needs formal specification: data structures, evaluation lifecycle, interaction with capabilities.

2. **Leader election protocol for leader-elected execution:** The execution assignment taxonomy (Section 3.2) is correct, but the leader election mechanism is unspecified. For meshes of 5-50 members, a simple approach (longest-uptime member with heartbeat failover) may suffice. Specify: how uptime is measured, how leader change is communicated, what the election latency is.

3. **Schema evolution during sync:** When Instance A has schema version 2 (new field) and Instance B has schema version 1, what happens? Recommended: strict schema match within a sync agreement. Schema changes require an agreement-level negotiation step. Mismatches are sync errors, not silent merges.

4. **CRDT merge strategy extensibility:** Property-level merge is currently locked to LWW. For collaborative editing (Digital Gardens as "a Wikipedia"), LWW on text properties means one editor's changes are silently discarded on concurrent edits. Recommended: add a `mergeStrategy` annotation on property schemas. Default to LWW. Support `text` (Yrs/Automerge CRDT text), `counter` (PN-Counter), and `set` (Add-Wins Set) as built-in alternatives.

5. **EMIT `exactly-once` delivery:** Exactly-once delivery in a distributed system without a central coordinator is impossible. Recommended: replace with `at-least-once` delivery + idempotency requirement on receivers. Or: define the deduplication mechanism that provides the exactly-once illusion (dedup key per EMIT, retained for a configurable window).

6. **Revocation propagation protocol:** How do revoked capabilities propagate across a P2P network? Recommended: short-lived capability grants with periodic renewal (TTL, default: 1 hour). Revocation = stop renewing. This bounds the stale-capability window to the TTL. Additionally, revocation records are prioritized in the sync protocol (delivered before data operations). When a peer comes online after being offline, the first sync action is a revocation check before any data operations.

7. **Ephemeral state:** The 12 primitives create persistent version Nodes for every WRITE. There is no mechanism for transient state (presence indicators, cursor positions, typing status). Recommended: a lightweight ephemeral channel (not persisted to the graph, not versioned) for presence and collaboration awareness. This is separate from the persistent graph and does not pollute version history.
