# Exploration: Distributed Compute, Storage, and Identity Vision

**Created:** 2026-04-13
**Status:** EXPLORATION -- architectural brainstorming, not committed design. Emerged from pre-work research during Phase 1 preparation.
**Context:** During dependency validation and competitive landscape research for the Benten Engine, a series of architectural insights reshaped the vision for how instances, data ownership, compute, and identity could work. This document captures those ideas for future design work.

---

## 1. The Core Insight: Your Data Has No Home

The original design assumes each user runs a persistent "instance" — a server or device that owns their data and syncs with peers. The new insight inverts this:

**Your data is encrypted, content-addressed, and distributed across the instances of people and communities you trust.** Your peers hold your ciphertext — they can't read it, they're just storage. Your "instance" is not a server — it's a runtime that materializes your graph from encrypted peer storage on demand, wherever you are.

Your trust graph IS your storage network.

### How It Works

1. You generate an ed25519 keypair (your identity)
2. Your private key is protected by biometric authentication on enrolled devices
3. Your graph data is encrypted with your key and replicated across your peers
4. When you want to access your data, you authenticate from any device
5. Authentication unlocks your private key from the device's secure enclave
6. You request your encrypted data from whichever peers are online
7. You decrypt locally, work with your data
8. Changes are encrypted and pushed back to peers

### What This Changes

- **No "personal server" to maintain.** Your data exists wherever your peers are.
- **Communities become the persistence layer.** Membership in communities = replicas of your encrypted data = higher availability. Social engagement literally makes your data more durable.
- **The availability problem dissolves.** Communities with members across time zones naturally have 24/7 coverage. No dedicated "always-on nodes" needed (though peers who want to earn credits by staying online can).
- **Access from anywhere.** Any enrolled device can pull your graph from peers, decrypt, and work.

### Influences

- **Tahoe-LAFS** ("Least Authority File Store") — encrypted data on untrusted nodes, production-proven
- **IPFS** — content-addressed routing, location-independent data identity
- **Holochain** — agent-centric, no home server, data distributed across DHT
- **NextGraph** — E2EE sync through broker nodes that can't read the data
- **SSB Dark Crystal** — social key recovery through trusted peers

---

## 2. The Unified Compute/Storage Marketplace

The original design had separate concepts: "always-on nodes" rented from a compute marketplace (Phase 13), member-mesh sync, and each member paying for their own storage. The new insight unifies everything:

**Every interaction with a peer's hardware is a micro-transaction in Benten Credits.** There is no distinction between "storing my data," "running a compute job," "serving a community website," or "being an always-on node." It's all resource usage on peers' hardware, paid with credits.

### The Economics

- Reading your own encrypted data from a peer's storage = micro-payment
- Evaluating an operation subgraph on a peer's hardware = micro-payment
- A community serving web requests via edge functions on members' devices = micro-payments to whoever handles requests
- AI inference, rendering, sync relay = compute jobs routed to best-fit hardware
- An "always-on node" = just a peer configured to accept jobs 24/7 because they want to earn

### Self-Balancing Equilibrium

- **Heavy users** (lots of data, lots of compute needs) pay net — credits flow to peers
- **Light users with good hardware** (leave machine running, don't use much) earn net
- **The incentive to stay online is economic, not architectural.** You WANT your device running because it earns.
- **No free riders.** Accessing your own data on a peer costs a micro-payment. Want free access? Keep your own device online.
- **Communities self-fund availability.** Members earn by serving each other.

### Local Access Is Always Free

Your own device runs computation and accesses locally cached data with no payment. The economic layer only activates for peer-to-peer resource usage. This means bootstrapping is natural — you start with your own device and only enter the marketplace when you need resources beyond what your device provides.

### LLM Routing as Bootstrap

The system starts with pre-configured APIs to existing LLM providers (OpenAI, Anthropic, etc.) for AI features. As the peer network grows, peers with GPUs can compete on price for inference. The routing protocol is the same regardless of whether the destination is a peer or an API.

### Settlement: Bar Tabs, Not Payment Channels

Since we have zero transaction fees and trusted peers, we don't need complex payment channels. Simple periodic net settlement:

1. Each peer pair maintains a running counter ("A owes B X credits for Y operations this hour")
2. Signed micro-receipts per operation (sender, receiver, amount, operation hash)
3. Periodic netting (hourly default, configurable per community): net A→B against B→A, transfer the difference
4. Dispute window: disagree on the count? Compare receipt logs. The graph is the audit trail.
5. Trust threshold: communities can set a max unsettled balance that triggers early settlement.

For a 100-member community, that's at most 4,950 settlements per hour — trivial.

### Why Zero-Fee Credits Are Load-Bearing

This entire model depends on micro-transactions being economically viable. On any blockchain (Ethereum, Solana), gas fees dwarf the actual payment for a single Node read. Our treasury-backed credits with zero transaction fees make granularity viable that no blockchain-based system can match.

---

## 3. Hardware Heterogeneity as Feature

Every device in the trust network has different capabilities:

| Device | Strengths | Natural Role |
|--------|-----------|-------------|
| Gaming PC (idle) | GPU, high RAM | AI inference, image processing, heavy compute |
| Home NAS/server | Storage, always-on | Data persistence, sync relay |
| Laptop | Balanced | General compute, local work |
| Phone | Bandwidth, mobility | Light compute, data access, notifications |
| Raspberry Pi | Low power, always-on | Lightweight persistence, relay |
| Cloud VPS | Bandwidth, reliability | High-availability serving |

Each device advertises its capabilities and pricing. Workloads route to the cheapest peer that meets requirements (speed, latency, GPU, storage). The marketplace matches resources to needs automatically.

---

## 4. AI Agents as Economic Actors

Each user's AI agent handles economic decisions on both sides:

### As Provider
- Sets prices based on hardware capabilities, current load, and market conditions
- Adjusts pricing dynamically (AMM-like curves)
- Decides which jobs to accept based on profitability and resource impact

### As Consumer
- Routes requests to cheapest peer meeting requirements
- Expresses user preferences: "prioritize speed over cost" or "always cheapest" or "prefer trusted peers"
- Handles governance voting based on stated preferences

### The Marketplace Is Agent-to-Agent

The "marketplace" is not a UI humans browse. It's a protocol where agents negotiate in milliseconds. The human never sees the bidding. This parallels how AI agents handle governance voting — the user expresses values, the agent executes continuously.

### Trust Boundaries via UCAN

The AI agent operates within the same capability system as everything else:
- UCAN grants with spending caps, rate limits, peer-selection rules
- Governance voting scope ("vote on content moderation, escalate treasury proposals to me")
- Immediate revocation if needed
- Every decision traceable via content-addressed audit trail

### Precedent

- CowSwap's intent-based routing is production-proven (user states desired outcome, solvers find best execution)
- Aragon ships rule-based automated governance
- AI agents with self-managed wallets are live in DeFi
- MCP (Model Context Protocol) has 10,000+ production servers for agent-to-system interaction

---

## 5. Verification Without Cryptographic Proofs

### Read Verification Is Free

Content-addressed data is self-verifying. Hash the returned data, compare to CID. No sampling needed.

### Compute Verification via Proof of Sampling

For operation subgraph evaluation:
- Random re-execution at 5-10% rate on a different peer
- Compare output hashes (works because our operations are deterministic)
- Penalty-to-reward ratio makes cheating irrational: `challenge_probability * penalty > reward_for_cheating`
- With reputation as penalty (cascading through IVM to all future job routing), a 10:1 penalty ratio at 10% sampling suffices

### SANDBOX (WASM) at Higher Sampling

The Turing-complete escape hatch gets 20-30% spot-check rate. Wasmtime's fuel metering provides deterministic resource accounting.

### Reputation as IVM Materialized View

EigenTrust-derived score maintained as an IVM view (O(1) lookup). Updated on every verification outcome. Used by AI agents for peer selection. The community itself reports and discovers peer performance.

Total verification overhead: under 5% of compute.

---

## 6. Identity and Key Management

### The Architecture

1. **ed25519 keypair** is the root of trust (unchanged from current design)
2. **Biometrics unlock**, they don't derive keys. Passkeys/WebAuthn unlock the stored private key from the device's secure enclave.
3. **Device mesh** — the user's own devices hold key shares. Recovery from a lost device uses another enrolled device.
4. **Social recovery as fallback** — M-of-N trusted peers each hold a Shamir share of the private key. Only needed when ALL devices are lost.
5. **New device enrollment** — scan QR from existing device, ephemeral Diffie-Hellman key exchange, transfer encrypted private key. No Apple/Google dependency.

### Key Findings from Research

- Passkeys are authentication credentials, not encryption keys. Users and credential managers treat them as replaceable. Using them as encryption key roots is dangerous.
- Biometrics cannot derive stable crypto keys. Fuzzy extractors "provide no security for real biometric sources" (ePrint 2024).
- Synced passkeys (iCloud Keychain, Google Password Manager) introduce centralization. No decentralized passkey sync exists yet.
- Web3Auth's tKey SDK is the most battle-tested Shamir implementation (2-of-3 default).
- The unsolved problem is UX: making guardian management effortless for non-technical users. No project has nailed this yet.

---

## 7. Edge Computing and WASM

### Dual-Target Architecture

The engine compiles to two targets:

**Native Rust** (local devices, high-performance servers):
- redb for persistence
- Real threads for parallel evaluation
- Full performance

**WASM** (edge functions, browsers, peer devices serving others):
- In-memory graph with network-fetched content-addressed data
- Single-threaded evaluation (adequate for bounded DAG operations)
- Storage abstraction fetches from peers instead of local filesystem
- 2-5MB binary size

### Why This Fits

Operation subgraphs are bounded DAGs with guaranteed termination — perfect for edge function execution limits. The evaluator is stateless — read from graph, walk subgraph, write results. Content-addressed data is cacheable at any edge location. The WASM instance doesn't need local persistence because it fetches encrypted data from the peer network.

### Platform Options

- **Fastly Compute** — WASM-native, microsecond cold starts
- **Cloudflare Workers** — V8-based with Durable Objects (SQLite persistence)
- **WasmEdge** — targets phones/IoT (Android, OpenHarmony)
- **Spin (Fermyon/Akamai)** — WASM-native app framework
- **napi-rs v3** — compiles to wasm32-wasip1-threads, single codebase for native + WASM

### Threading Limitation

WASI Preview 2 has no threading. WASM instances handle one evaluation at a time. Adequate for edge functions and personal devices. Heavy-compute peers in the marketplace would run native Rust, not WASM.

---

## 8. What This Means for Phase 1

These ideas do NOT change Phase 1 scope. Phase 1 builds the graph engine with local persistence (redb), which is needed regardless of the deployment model. The storage abstraction layer (redb for native, network-fetch for WASM) is a Phase 2+ concern. The compute marketplace is Phase 5+.

What Phase 1 should keep in mind:
- **Design clean trait boundaries** for storage, so swapping redb for a network-fetch backend is possible later
- **Content-addressed hashing (CIDv1 format)** is even more important in this model — it's the routing key for peer data access
- **The evaluator should be stateless** — no implicit dependency on local storage state during subgraph evaluation
- **WASM compilation target** matters more than we initially thought — test it early in the spike

---

## 8.5. The Benten Runtime as an Infrastructure Layer

**Core insight:** Cloudflare Workers isn't magical technology -- it's V8 isolates with a specific billing model. Rather than deploying Benten communities INSIDE Cloudflare's proprietary runtime, we build our own WinterTC-compliant runtime that anyone can run. This creates a peer-distributed alternative to edge platforms.

### The Standard: WinterTC (TC55)

WinterCG became an Ecma Technical Committee (TC55) in December 2024. The Minimum Common API defines a browser-aligned surface (fetch, Request/Response, ReadableStream, crypto.subtle, structuredClone, URL, WHATWG streams). Node, Deno, Bun, Workers, Vercel, and Netlify all implement it by 2026.

Building to WinterTC means portability across all major edge platforms -- no vendor lock-in for the stateless parts of our stack.

### Three Products from One Engine

| Layer | Product | Customer |
|-------|---------|----------|
| **Application** | Benten communities, CMS, AI platform | End users, organizations |
| **Runtime** | Benten Runtime (WinterTC + Benten engine) | Application deployers, peer hosts |
| **Economy** | Benten Credits + compute marketplace | Both of the above |

All three use the same graph, capability system, and primitives. Code-as-graph means we're building one coherent system that manifests as multiple products.

### The Cloudflare Pattern Validates Community-as-Coordinator

Cloudflare acquired PartyKit in April 2024. Its core idea -- one Durable Object per room/entity -- became the canonical "stateful serverless" pattern. Applied to Benten: **one stateful coordinator per community** is a directly validated mapping.

Durable Object capabilities (for reference):
- Single-threaded actor, pinned to a geographic location
- SQLite-backed storage (10GB at GA)
- WebSocket Hibernation (long-lived connections without constant compute billing)
- 5-minute CPU time per request, 128MB memory
- NOT geo-replicated -- one location per community, chosen by first caller

**Critical limitation:** Only Cloudflare has mature stateful edge primitives. Deno KV is closest; Fastly/Vercel/Netlify have none natively. For vendor-neutral deployment, communities run on peer hardware through our runtime; for managed hosting, Cloudflare is currently the only full-featured option.

### What the Benten Runtime Does

A WASM host that provides:
- WinterTC-compliant API surface (fetch, streams, crypto, etc.)
- Content-addressed fetch integration (pull encrypted data from peer network)
- Operation subgraph evaluator (the engine compiled to WASM)
- Capability enforcement at the runtime level
- Metering/billing integration (reports to compute marketplace)
- Peer discovery (find other Benten Runtime nodes)
- Stateful coordinator primitive (community-scoped, SQLite-backed, comparable to DO)

### Foundation Options

Don't build from scratch. Fork or integrate with:
- **Deno** -- open-source, MIT-licensed, WinterTC-compliant, has Deno KV for storage
- **Bun** -- permissive license, fast, WinterTC-compliant
- **workerd** -- Cloudflare's open-source Workers runtime (MIT-licensed)
- **wasmtime** -- for WASM execution inside the runtime
- **SQLite** -- for per-community storage

---

## 8.6. General-Purpose Compute: `bentend`

**Core insight:** Akash needed K8s because it exposes a generic cloud API ("deploy any container"). Benten exposes a graph execution API -- workloads are first-class graph Nodes with capabilities. This means containers and VMs become just additional drivers in our existing SANDBOX-style escape hatch model.

### Don't Build a Distro. Build a Peer Daemon.

`bentend` is a single Rust daemon installable on any Linux (Debian, NixOS, Alpine, Ubuntu). It composes existing commodity runtimes rather than reinventing a Linux distribution.

| Layer | Component |
|-------|-----------|
| Base OS | User's choice -- provide NixOS module + Debian package |
| Container runtime | containerd + runc (lighter than Docker) |
| VM runtime | cloud-hypervisor or firecracker (microVMs, fast boot) |
| WASM runtime | wasmtime (already our SANDBOX choice) |
| Local orchestrator | Nomad (pluggable drivers) or custom thin scheduler |
| Storage | redb for graph state; bind-mount host directories for workloads |
| Control plane | The Benten graph itself |
| Networking | iroh for peer transport; WireGuard for inter-workload mesh |
| Marketplace | Bid/ask as subgraphs; UCAN capabilities gate job acceptance |

### Why Nomad Over Kubernetes

Nomad's pluggable task drivers (docker, podman, exec, qemu, firecracker, WASM) fit Benten's workload model cleanly. K8s assumes trusted nodes with mutual network access -- wrong model for P2P. K8s is overkill; Nomad is the philosophical fit.

### The Phased Path

1. **Phase 1-3:** Graph engine + DSL + sync protocol. `bentend` doesn't exist yet.
2. **Phase 5+:** `bentend` ships with WASM-only drivers (uses existing SANDBOX). Peers can host Benten communities but nothing else.
3. **Phase 7+:** Container driver added (containerd). Peers can now host containerized workloads. General-purpose compute emerges.
4. **Phase 9+:** VM driver added (firecracker). Heavy workloads, isolation, regulated use cases.
5. **Optional future:** "Benten Appliance" -- NixOS-based immutable image for non-technical users dedicating hardware.

### Resources as Graph Nodes

A peer's hardware advertisement is a graph Node:
- CPU cores, GPU (model + VRAM), RAM, storage (type + capacity), network bandwidth
- Pricing per resource type (market-discovered via AI agent)
- Availability schedule, thermal constraints
- Reputation score (EigenTrust-derived, IVM materialized view)
- Trust tier memberships (Atrium/Garden/Grove memberships, TEE attestations, VCs)

---

## 8.7. Mobile Devices: Clients, Not Providers

Mobile volunteer compute has repeatedly failed (Folding@home mobile shut down 2024, BOINC mobile marginal). The 2026 reality:
- iOS background execution: ~30 seconds, BGProcessingTask up to a few minutes (opportunistic, throttled)
- Android: WorkManager ~10 min windows, aggressive bucket demotion, Doze mode kills sockets
- **Google Play banned on-device crypto mining Oct 29, 2025.** Apple banned entirely. "Earn compute" framing gets apps delisted.
- On-device AI (Apple Intelligence, Gemini Nano, MLC-LLM) serves the local app only -- cannot serve inference to others

### Realistic Roles

**Primary phone (App Store distributed):**
- Consumer/client only
- Local subgraph read/write, local AI inference for owner
- Key custody in Secure Enclave
- Opportunistic sync when foregrounded or charging+WiFi
- **Frame as "sync," never "contribute" or "earn"**

**Old/retired phones (postmarketOS, LineageOS, Ubuntu Touch):**
- First-class full peers
- Legitimate always-on home server tier
- No App Store, no background limits
- On-brand narrative: e-waste reduction + data sovereignty

### Architectural Implication

**Every community needs at least one peer online at any given moment** -- but the specific peer doesn't have to be stable. Availability is emergent from the union of members' activity patterns. A community with enough members across enough time zones is naturally always-reachable without anyone running dedicated infrastructure. Gaps cause temporary desync or splitting, which self-heals when any member comes back online. This is the important distinction: we don't require "an always-on peer," we require that coverage exist at any moment. Coverage can be provided by member activity, by opportunistic sync from old phones on postmarketOS, by rented edge-node peers, or by anyone choosing to host -- or any combination. Mobile-only communities with all members in the same time zone will experience gaps; this is a graceful degradation, not a binary failure.

---

## 8.8. Trust Tiers as Composable Primitives

**Core insight:** Trust isn't a hierarchy -- it's four orthogonal primitives that workloads declare requirements across.

### The Four Primitives

1. **Cryptographic identity gating** (Tailnet Lock, Yggdrasil, PGP WoT) -- binary, social. Atriums use this: family members sign each other's keys.
2. **Reputation-weighted routing** (EigenTrust, staking + slashing) -- numeric scores. Open marketplace uses this.
3. **TEE remote attestation** (Intel TDX, AMD SEV-SNP, NVIDIA GPU TEE) -- hardware substitutes for social trust. Lets low-rep peers handle high-sensitivity work.
4. **Verifiable Credentials + Soulbound Tokens** -- cryptographic claims for KYC, insurance, jurisdiction, community membership.

### Workload Trust Declaration

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

The scheduler (user's AI agent) filters peer set by intersection. Maps cleanly to our capability-Node pattern -- trust requirements become edges from the workload subgraph.

### Key Nuances

- **E2EE does NOT eliminate trust requirements.** Even encrypted workloads leak execution metadata, can be withheld/delayed, can be selectively DoS'd. Trust tier still matters for liveness.
- **TEEs are a real shift in 2026** (NVIDIA GPU TEEs close the AI workload gap). But treat TEE-attested-open-peer as equivalent to Garden-tier, not Atrium-tier -- vendor-key compromise is catastrophic and correlated.
- **Pricing is reputation-weighted.** Higher trust = higher price. Market surfaces the tradeoff rather than hardcoding it.
- **One substrate, opt-in sub-markets** (the Tor/Filecoin Plus pattern). Not tiers as hierarchy -- mechanisms as filters.

### Trusted Third-Party Providers Resolve the Trust Tension

Some communities will form around providing high-reliability compute:
- "We verify our members' hardware, stake credits as insurance, offer SLAs"
- Compete with Cloudflare on service and trust guarantees
- Governance-as-code means their verification processes, audit procedures, and insurance models are operation subgraphs
- The fractal community structure naturally accommodates this -- a compute-provider Grove is just another Grove

---

## 9. Comparison to Existing Projects

No project combines all these elements:

| Feature | Benten | IPFS | Akash | Holochain | AT Protocol |
|---------|--------|------|-------|-----------|-------------|
| Content-addressed data | Yes | Yes | No | Yes | Yes |
| Encrypted peer storage | Yes | No | No | Partial | No |
| Compute marketplace | Yes | No | Yes (blockchain) | No | No |
| Zero-fee micro-transactions | Yes | No | No (gas) | No | No |
| AI agent economic actors | Yes | No | No | No | No |
| Code-as-graph execution | Yes | No | No | No (all WASM) | No |
| Bounded computation | Yes | N/A | No | No | N/A |
| Self-verifying data | Yes | Yes | No | Yes | Yes |
| Deterministic evaluation | Yes | N/A | N/A | Partial | N/A |
| Forkable governance | Yes | No | No | No | No |

The closest precedent for individual pieces: Tahoe-LAFS (encrypted untrusted storage), Akash (compute marketplace), CowSwap (intent-based routing), Proof of Sampling (probabilistic verification), EigenTrust (reputation). The unification is novel.

---

## 10. Open Questions

1. **Community-level economic policies.** An Atrium between family members probably sets internal micro-payments to zero. A Grove might have a treasury that subsidizes member usage. How configurable should economic parameters be per community?

2. **Peer churn and re-replication.** If peers go offline permanently, replica count decreases. How does the system detect this and re-replicate to maintain durability?

3. **Cold start performance.** First access from a new device requires pulling encrypted graph data from peers. How large is the typical working set? Can Merkle tree diffing make this fast enough?

4. **Guardian UX for key recovery.** No project has made Shamir threshold recovery feel effortless to non-technical users. This is a critical design challenge.

5. **Regulatory implications.** Does a compute marketplace where AI agents autonomously transact create regulatory obligations beyond what the GENIUS Act covers?

6. **Content-addressed routing protocol.** How does a device find which peer holds a specific content hash? DHT (slow, proven), GossipSub (fast, bounded), or direct peer query (simplest for small trust networks)?

7. **Write coordination.** When multiple devices/peers hold your data and you write from one, how do writes propagate and conflicts resolve? (Version chains + HLC handle this, but the flow through encrypted peer storage needs specification.)

8. **Storage pricing.** How do peers price storage vs. compute vs. bandwidth? Separate rates? Bundled? Market-discovered?
