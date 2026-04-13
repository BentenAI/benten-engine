# Benten Platform Specification v2

**Created:** 2026-04-13
**Status:** DEFINITIVE — synthesizes 45+ research documents, 16 critic reviews, and extensive architectural discussion.
**Scope:** Complete platform specification — from Rust engine primitives through P2P networking, governance, economics, and legal structure.

---

## Part 1: Vision

### 1.1 What Benten Is

Benten is a self-evaluating graph — a platform where data and code are both Nodes and Edges, the graph evaluates itself, and every person, family, or organization can own their data, share it selectively, and fork at any time.

It is not a database. It is not a CMS. It is not a blockchain. It is the foundation of a new web where:
- Your data lives on YOUR instance
- You choose who sees what through capability grants
- Communities form, federate, fork, and compete on governance quality
- Knowledge is curated through speculative attestation markets
- AI agents are first-class citizens operating within capability boundaries
- The entire system runs as a self-evaluating graph with zero distinction between "application" and "database"

### 1.2 The Three Tiers

**Atriums** — Peer-to-peer direct connections. Partners sharing finances, friends planning a trip, a student syncing with a school. Private, selective, bidirectional sync of chosen subgraphs. Each peer pays only for their own compute/storage.

**Digital Gardens** — Community spaces. Like a Discord server, a Wikipedia, or a knowledge base — but decentralized. Each member syncs the community graph locally. No central server required. Admin/moderator governance configures capabilities, moderation rules, and content policies. A Garden's character depends on its purpose — a casual hangout, a curated knowledge base, a professional network.

**Groves** — Governed communities. Fractal, polycentric, polyfederated governance. Voting on rules, smart contracts as operation subgraphs, formal decision-making. Sub-Groves with inherited or overridden governance. Fork-and-compete dynamics — communities compete on governance quality.

### 1.3 Core Principles

1. **The graph IS the runtime.** No distinction between database and application. Data and code are both Nodes and Edges.
2. **Capabilities, not tiers.** Operator-configured, UCAN-compatible capability grants. Same system for local modules, remote instances, AI agents.
3. **History IS the graph.** Version chains, not revision tables. Every mutation creates a version Node. Undo, audit, and time-travel are graph traversals.
4. **Content-addressed integrity.** Every version Node is hashed. Merkle trees for efficient sync. Verifiable knowledge, verifiable code, verifiable governance.
5. **Fork is a right.** Any participant can fork any subgraph at any time, keeping full history. This creates evolutionary pressure on governance.
6. **Governance is the competitive dimension.** Communities differentiate through their governance model, not their technology. The platform makes governance trivially configurable and forkable.
7. **Zero-fee transactions.** The platform currency has no transaction fees. Revenue comes from treasury interest on reserves, not from taxing users.
8. **AI-native.** AI agents discover, operate, and reason through the same graph that humans use. The graph is self-describing and inspectable.

---

## Part 2: The Engine

### 2.1 Architecture: A Self-Evaluating Graph

The Benten engine is a Rust-native system where code is represented AS graph structure, not stored IN graph properties. A route handler is not a string of source code — it is a subgraph of operation Nodes connected by control-flow Edges. "Executing" a handler means the engine walks the subgraph, performing each operation.

```
[RouteHandler: GET /api/posts]
    ├──[FIRST_STEP]──→ [GATE: require capability store:read:post/*]
    │                      └──[NEXT]──→ [READ: query posts where published=true]
    │                                      └──[NEXT]──→ [TRANSFORM: to JSON]
    │                                                      └──[NEXT]──→ [RESPOND: 200]
    └──[ON_DENIED]──→ [RESPOND: 403]
```

**Why code-as-graph:**
- **Inspectable.** An AI agent can read any handler by traversing its subgraph.
- **Modifiable at runtime.** Insert a caching step without recompilation.
- **Versionable.** The handler is a subgraph with a version chain. Roll back by moving the CURRENT pointer.
- **Syncable.** Installing a module = syncing its operation subgraphs. Code travels as data.
- **Statically analyzable.** The engine can pre-compute cost, check capabilities, and verify determinism BEFORE execution — because subgraphs are DAGs.

### 2.2 The 12 Operation Primitives

Every computation in Benten is composed from 12 primitive operation Node types. The vocabulary is deliberately NOT Turing complete — subgraphs are DAGs with bounded iteration. This guarantees termination, enables static analysis, and prevents denial-of-service.

| # | Primitive | Purpose | Key Property |
|---|-----------|---------|-------------|
| 1 | **READ** | Retrieve from graph (by ID, query, materialized view) | Typed error edges: ON_NOT_FOUND, ON_EMPTY |
| 2 | **WRITE** | Mutate graph (create, update, delete, conditional CAS) | Auto version-stamp. Typed error: ON_CONFLICT, ON_DENIED |
| 3 | **TRANSFORM** | Pure data reshaping | Sandboxed expression with arithmetic, array built-ins (filter, map, sum, etc.), object construction. No I/O. |
| 4 | **BRANCH** | Conditional routing | Forward-only, no cycles |
| 5 | **ITERATE** | Bounded collection processing | Mandatory maxIterations. Optional parallel. Multiplicative cost through nesting. |
| 6 | **WAIT** | Suspend until signal/timeout | For workflows, approval patterns. Decomposes to data Nodes for cross-instance. |
| 7 | **GATE** | Custom logic escape hatch | For complex validation/transformation that can't be expressed as TRANSFORM. Capability checking via `requires` property on any Node. |
| 8 | **CALL** | Execute another subgraph | `isolated` flag for capability attenuation. Mandatory timeout. |
| 9 | **RESPOND** | Terminal: produce output | HTTP response, event result, agent action result |
| 10 | **EMIT** | Fire-and-forget notification | `deliveryMode`: local, exactly-once, broadcast |
| 11 | **SANDBOX** | WASM computation escape hatch | No re-entrancy. Fuel-metered. Time-limited. Max output 1MB. For complex computation that can't be expressed as operation Nodes. |
| 12 | **VALIDATE** | Schema + referential integrity check | Before writes, on sync receive |

**Composed from the 12 (NOT primitives):** Retry (ITERATE + BRANCH + CALL), Parallel (ITERATE with parallel flag), Compensate (BRANCH on error + CALL undo subgraph), DryRun (evaluation mode property), Audit (EMIT to audit channel), Map/Filter/Reduce (ITERATE + TRANSFORM).

### 2.3 Structural Invariants

| # | Invariant | Purpose |
|---|-----------|---------|
| 1 | Subgraphs are DAGs (no cycles) | Guarantees termination |
| 2 | Max depth: configurable per capability grant | Bounds sequential operations |
| 3 | Max fan-out per node: configurable | Prevents combinatorial explosion |
| 4 | Max SANDBOX nesting: configurable | Bounds WASM call depth |
| 5 | Max total nodes per subgraph: 4096 default | Bounds validation cost |
| 6 | Max total edges per subgraph: 8192 default | Bounds graph size |
| 7 | Max SANDBOX output: 1MB default | Prevents memory exhaustion |
| 8 | Cumulative iteration budget (multiplicative) | Prevents nested loop explosion |
| 9 | Determinism classification per operation type | Enables sync correctness |
| 10 | Content-addressed hash per subgraph | Integrity verification, deduplication |
| 11 | System-zone labels unreachable from user operations | Kernel/userspace boundary |
| 12 | Registration-time structural validation | Malformed subgraphs never execute |
| 13 | Immutable once registered (new version for changes) | Prevents TOCTOU attacks |
| 14 | Causal attribution on every evaluation | Unsuppressible audit trail |

All numeric limits are configurable per capability grant — the operator (or community governance) decides the bounds.

### 2.4 The Evaluator

An iterative graph walker with an explicit execution stack:

```rust
let mut stack = vec![entry_node];
while let Some(node) = stack.pop() {
    let result = execute_operation(node, &mut context);
    match result {
        Ok(next_nodes) => stack.extend(next_nodes),
        Err(error_edge) => stack.push(follow_error_edge(node, error_edge)),
    }
}
```

**Properties:**
- Iterative, not recursive (no stack overflow risk, can pause/resume)
- Per-node overhead: ~200-300ns (10 nodes ≈ 2-3 microseconds)
- Can serialize execution state (for WAIT — save stack, resume later)
- Can step-through debug (pop one node, execute, inspect context)
- Shared context with declared read/write keys per operation

### 2.5 Version Chains

Every versionable entity has:
- **Anchor Node** — stable identity, never changes
- **Version Nodes** — complete snapshots
- **NEXT_VERSION edges** — linking the chain (becomes a commit DAG when concurrent edits branch)
- **CURRENT pointer** — edge from anchor to latest version

External edges point to anchors, not versions. Resolving current state: one hop (anchor → CURRENT → version).

History = traverse version chain. Undo = move CURRENT pointer. Sync = exchange version Nodes. Fork = stop syncing, keep all version history.

### 2.6 Content-Addressed Hashing

Every version Node is hashed using BLAKE3 with a multihash prefix for algorithm agility. What gets hashed: labels + properties (NOT anchor ID, NOT timestamps, NOT edges).

**Used for:**
- Module identity and community attestation
- Sync integrity verification (what I received = what was sent)
- Version comparison (same hash = same content, instant comparison)
- Deduplication across instances
- Governance decision anchoring (votes reference specific hashes)
- Merkle trees for efficient sync (compare roots, transfer only differences)

**Canonical serialization:** CBOR with RFC 8949 deterministic encoding.

### 2.7 Incremental View Maintenance (IVM)

The engine maintains materialized views that update incrementally on writes:

1. Define a view (a query pattern, stored as a Node)
2. Engine pre-computes the result
3. On write: engine identifies affected views and incrementally updates them
4. Read from view: O(1) — return pre-computed result

**Critical for:** Event handler resolution, capability checks, content listings, knowledge attestation values, governance rule resolution.

**The key property:** Governance resolution (deep nesting, polycentric multi-parent) is O(1) at check time because the "effective rules" view is maintained incrementally when governance changes (rare), not recomputed when rules are checked (constant).

### 2.8 Capability System

Capabilities are UCAN-compatible typed objects stored as Nodes with GRANTED_TO edges:

```
[CapabilityGrant: {domain: 'store', action: 'read', scope: 'post/*'}]
    └── GRANTED_TO ──→ [Entity: seo-module]
```

**Properties:**
- Operator-configured (not hardcoded tiers)
- Enforced at the engine level (every WRITE checks capabilities before executing)
- Attenuation: scope can only narrow, never widen (CALL with `isolated=true`)
- UCAN-compatible serialization for P2P (same grant, signed for network transport)
- Same system for modules, WASM sandboxes, remote instances, AI agents
- Multi-tenancy is just capability scopes (no separate tenant infrastructure)

**Replaces:** The 4 fixed trust tiers (platform/verified/community/untrusted) from V3. Tiers become optional presets.

### 2.9 WASM Sandbox (SANDBOX Primitive)

For computation that can't be expressed as operation Nodes, the SANDBOX primitive calls into a WASM runtime (@sebastianwessel/quickjs v3.x):

- **No re-entrancy.** The sandbox receives data, returns data. Cannot call back into the graph.
- **Fuel-metered.** Every WASM instruction costs fuel. Execution terminates when fuel runs out.
- **Memory-limited.** Configurable max heap.
- **Time-limited.** Wall-clock timeout as backstop.
- **Capability-gated.** Which host functions exist is determined by the caller's capabilities.

### 2.10 Rust Crate Structure

Revised from critic feedback (10 crates → 4-5 for V1):

```
benten-engine/
├── crates/
│   ├── benten-core/       # Node, Edge, Value types, content hashing, version chain primitives
│   ├── benten-graph/      # Graph storage, indexes (hash + B-tree), MVCC, persistence (redb), IVM
│   ├── benten-eval/       # Operation evaluator, 12 primitives, capability enforcement
│   └── benten-engine/     # Orchestrator: public API, ties crates together
├── bindings/
│   ├── napi/              # Node.js bindings (@benten/engine-native)
│   └── wasm/              # WASM bindings (@benten/engine-wasm)
└── tests/
```

Additional crates (added incrementally): `benten-sync` (CRDT merge, sync protocol), `benten-query` (Cypher/query parser, if needed beyond operation subgraphs).

### 2.11 Standards Adopted

| Standard | Purpose | Phase |
|----------|---------|-------|
| BLAKE3 | Hashing (content addressing, Merkle trees) | 1 |
| Ed25519 | Signatures (votes, attestations, UCAN, sync) | 1 |
| CBOR (RFC 8949) | Canonical serialization | 1 |
| Multihash | Algorithm-agile hash format | 1 |
| UCAN | Authorization tokens | 1 |
| did:key | Default identity | 1 |
| Hybrid Logical Clocks | Causal ordering for CRDT | 1 |
| libp2p | P2P networking (NAT traversal, DHT, GossipSub) | 2 |
| W3C Verifiable Credentials | Identity attestations | 2 |
| Merkle Search Trees | Efficient sync negotiation | 2 |

---

## Part 3: Networking

### 3.1 Member-Mesh Model

Communities are NOT hosted on servers. They are the distributed copies across members' instances. Each member syncs the community graph locally. There is no single point of failure or control.

- **Availability:** As long as any member is online, the community is accessible.
- **Scaling:** More members = more copies = more capacity for new syncs.
- **Cost:** Each member pays for their own storage/bandwidth. No pooled hosting.
- **Always-online nodes:** Communities that want reliability can rent a persistent node from the compute marketplace — it's just another member that happens to be always-on.

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

**Execution assignment:** Handlers have an `executionPolicy`: origin-only (only the instance that triggered the event), local (each instance runs independently), leader-elected (one designated instance runs, others receive results).

**Triangle convergence:** Deduplication key = (originInstance, originHLC, nodeId). Every instance forwards received changes to all agreements containing that Node.

### 3.3 Identity

Three-layer identity stack:

| Layer | Purpose | Technology |
|-------|---------|------------|
| Persistent Identity | Survives key rotation | KERI AID or did:plc |
| Transport Identity | Authenticates on transport | did:key (Ed25519) |
| Transport Address | Network reachability | libp2p multiaddr (or Yggdrasil IPv6, optional) |

**Decentralized verification:** Identity verification is a marketplace of attestations via W3C Verifiable Credentials. KYC providers, communities, organizations, and individuals all issue credentials. Communities decide which verifiers they trust. The same mechanism handles regulatory KYC, professional credentials, social vouching, and community membership.

---

## Part 4: Governance

### 4.1 Governance as Code

All governance rules are operation subgraphs — content-hashed, versionable, forkable, syncable. Voting mechanisms, contribution fees, moderation rules, membership criteria — all configurable Nodes in the graph.

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

Governance inheritance uses prototypal resolution (like JavaScript's prototype chain). IVM materializes "effective rules" so governance checks are O(1).

### 4.4 Polycentric Federation

A Grove can have MULTIPLE parent Groves simultaneously (DAG, not tree). Each parent's authority is domain-scoped. Conflicts between parents resolved by: explicit priority, union (strictest wins), local override, or mediation.

### 4.5 Fork-and-Compete

Forking a community = syncing its graph + modifying governance parameters + publishing. The fork inherits all content and history. Members choose which governance model they prefer. Evolutionary pressure optimizes governance — communities that govern well retain members.

---

## Part 5: Economics

### 5.1 Benten Credits (Platform Currency)

- **1 credit = $1 USD** (initially pegged)
- **Zero transaction fees** within the network
- **Mint:** User sends $1 USD → BentenAI mints 1 credit
- **Burn:** User redeems credit → BentenAI burns it, returns $1 USD
- **Revenue:** BentenAI invests reserves in Treasury bonds (~4-5% annual return)
- **On/off ramp:** FedNow ($0.045/tx, instant settlement)
- **Regulatory:** Stored-value product initially, GENIUS Act PPSI license when P2P distributed

### 5.2 Knowledge Attestation Marketplace

- Accessing knowledge is FREE
- Attesting to knowledge costs a fee (set by community governance)
- Fees flow to existing attestors (distribution set by community)
- Creates AI-consumable trust signals: "This fact attested by N people at $X in community Y"
- Each community calibrates its own economics — casual communities have low fees, professional communities have higher stakes
- Attestations are Edges in the graph (ATTESTED_BY), content-hashed, with cost/timestamp properties
- IVM materializes attestation value per knowledge Node (O(1) read for AI)

### 5.3 Decentralized Compute Marketplace

- Small businesses run Benten servers for their own needs
- Idle compute rented out at near-cost through the network
- Communities rent always-online nodes for persistent availability
- Benten Credits used for compute purchases
- Verification of computation through verifiable services (storage, bandwidth) initially, general compute later

### 5.4 Future: Floating Token + Governance Token

- **Phase 1:** USD-pegged Benten Credits (stored-value product)
- **Phase 2:** Two-token model — pegged credits for payments + floating governance token for community governance
- **Phase 3:** Credits unpeg and float (major milestone — "the Benten economy is independent")

### 5.5 Decentralized Identity Verification

- KYC/identity verification is a marketplace of verifiers
- Users choose their provider (Persona, Jumio, community vouching, professional bodies)
- Verifiable Credentials stored as Nodes in the user's graph
- Communities decide which verification levels they require
- BentenAI maintains approved verifier list for token system compliance
- Cost borne by the user, not the platform (~$2-5 for formal KYC)

---

## Part 6: Business Model

### 6.1 BentenAI as Central Bank

BentenAI operates the mint/burn mechanism for Benten Credits:
- Receives USD, mints credits (1:1)
- Invests reserves in Treasury bonds
- Revenue = interest on reserves
- Burns credits on redemption, returns USD

### 6.2 Revenue Streams

| Stream | Description | Phase |
|--------|-------------|-------|
| Treasury interest | ~4-5% on credit reserves | 1 |
| Compute marketplace commission | Small % on compute transactions | 2 |
| Premium features | Enterprise support, advanced analytics | 2 |
| API access | Developer tools, integration APIs | 2 |

### 6.3 DAO Transition

1. **Phase 1:** BentenAI is sole central bank operator
2. **Phase 2:** A governance Grove has oversight; BentenAI operates but the Grove sets policy
3. **Phase 3:** Central bank function becomes operation subgraphs governed by the Grove
4. **Phase 4:** Full DAO; BentenAI becomes a service provider, not the authority
- Note: regulated fiat on/off ramp always needs a licensed entity

### 6.4 Regulatory Path

- **Phase 1:** Stored-value/prepaid access (lighter regulation). FinCEN MSB registration.
- **Phase 2:** State money transmitter licenses in key states (or partner with licensed transmitter).
- **Phase 3:** GENIUS Act PPSI license (federal, preempts state). Requires 1:1 reserves, monthly attestation, AML/KYC.
- **Entity structure:** Delaware C-Corp → Cayman/Swiss Foundation → DAO-governed foundation.

---

## Part 7: Migration from Thrum V3

### 7.1 What Carries Forward

- CMS domain code (content types, blocks, compositions, field types)
- SvelteKit web app (UI components, routes, pages)
- Module definitions (adapted to operation subgraphs)
- Test infrastructure patterns
- 3,200+ behavioral test expectations (the contracts, not the implementations)

### 7.2 What Gets Replaced

- In-memory registries → IVM materialized views
- Event bus → Reactive subscriptions + EMIT primitive
- PostgreSQL+AGE → Benten engine native storage
- RestrictedEventBus + TIER_MATRIX → Capability enforcement
- Trust tiers → Capability grants
- compositions.previous_blocks → Version chains
- content_revisions table → Version Nodes
- module_settings → Settings as graph Nodes

### 7.3 Migration Strategy

The engine exposes a TypeScript API (via napi-rs) that implements the existing Thrum Store interface. Existing modules can run unmodified against this adapter. Migration is then incremental — each module is rewritten to use operation subgraphs at its own pace.

---

## Part 8: Build Order

### Phase 1: Core Engine
- benten-core (Node, Edge, Value, content hashing, version chains)
- benten-graph (storage, indexes, MVCC, persistence via redb)
- napi-rs bindings (TypeScript can create/read/query Nodes)
- Benchmark suite (prove <0.1ms for hot-path queries)

### Phase 2: Evaluator + Capabilities
- benten-eval (12 primitives, evaluator, structural validation)
- Capability enforcement (UCAN grants as Nodes, checked on every WRITE)
- IVM (materialized views for capabilities, event handlers, content listings)
- SANDBOX integration (@sebastianwessel/quickjs)

### Phase 3: Sync + Networking
- CRDT merge for version chains
- libp2p integration (peer discovery, GossipSub)
- Atrium sync (peer-to-peer)
- Sync protocol (delta exchange, Merkle comparison)

### Phase 4: Platform Features
- Migrate Thrum CMS domain to operation subgraphs
- Schema-driven rendering (materializer pipeline as operation subgraphs)
- Self-composing admin
- AI agent integration (MCP tools as capability-gated operation subgraphs)

### Phase 5: Governance + Economics
- Garden/Grove governance subgraphs
- Configurable voting mechanisms
- Benten Credits (mint/burn, FedNow integration)
- Knowledge attestation marketplace
- Compute marketplace

### Phase 6: Polish + Ship
- CLI (npx create-benten)
- Documentation
- Edge/serverless deployment
- Performance optimization
- Security audit

---

## Part 9: Open Questions

1. **Expression language for TRANSFORM:** What specific syntax? JavaScript subset? Custom DSL? How much of @benten/expressions carries forward?
2. **Cypher support:** Do we need a Cypher parser, or are operation subgraphs sufficient for all queries? (Existing Rust Cypher parser: open-cypher crate)
3. **libp2p vs Yggdrasil:** libp2p is mature and production-proven. Yggdrasil is alpha but offers address=identity elegance. Support both behind a transport abstraction?
4. **MVCC implementation:** Snapshot isolation vs serializable. Garbage collection strategy for old snapshots.
5. **IVM algorithm:** DBSP (Feldera) Z-set algebra vs red-green invalidation (rustc approach) vs custom.
6. **Disk-backed graph:** redb B-tree backed, LRU cache for hot Nodes, or custom storage engine?
7. **Binary size for WASM target:** PGlite is 860MB. Target for benten-engine-wasm?
