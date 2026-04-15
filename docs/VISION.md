# Benten Vision

**Created:** 2026-04-14
**Last Updated:** 2026-04-14 (three-pillars framing replaces three-products; committed scope expanded to 8 phases after vision evolution)
**Status:** Consolidated vision statement reflecting validated Phase 1-8 committed scope.
**Audience:** Anyone new to the project — developers, critics, investors, AI agents joining the project.

---

## In One Sentence

Benten is the engine for the decentralized web — a Rust graph execution engine where every platform capability composes on top rather than being built in, with personal AI assistants as the first application that replace the paid software stack, funded by treasury interest on USD-pegged credits.

## In Three Sentences

Benten is a Rust graph execution engine where data and code are unified as Nodes and Edges, so every capability a modern platform needs — authentication, real-time sync, content management, AI integration, governance, economics — composes from a small set of primitives rather than being bolted on. On top of the engine we're building personal AI assistants that organize users' knowledge (PARA structure), generate the tools they need on demand, and ultimately replace the paid software stack with one each user owns and controls. The platform is funded by treasury bond interest on USD-pegged credit reserves, with a peer-to-peer compute network providing secondary revenue and driving credit utilization.

---

## The Three Pillars

Benten is one coherent project with three interdependent pillars. Each one supports the others. Miss any and the system doesn't hold up.

### Pillar 1: The Engine (What We Build)

**The engine for the decentralized web.** A Rust graph execution engine where:
- Code and data are one content-addressed structure
- Every platform capability (auth, sync, storage, compute, governance, economics) composes from 12 operation primitives
- Execution is deterministic, bounded, inspectable
- Every AI agent action produces an unforgeable audit trail

This is the foundation. No clever tricks — just a small engine that does a few things well, on top of which we compose everything else. If the engine holds, the rest of the vision is buildable. If it doesn't, nothing else matters.

### Pillar 2: The Adoption Driver (How People Come)

**Personal AI assistants that replace paid software.** Every user runs an assistant anchored to their own data, which:
- Organizes their knowledge using PARA methodology (Projects, Areas, Resources, Archives)
- Integrates natively with any tool built on the platform
- Generates new tools on demand from the engine's primitives — "I need a habit tracker" → the assistant composes one from WRITE/READ/IVM/TRANSFORM subgraphs
- Ultimately replaces the stack of paid subscriptions (Notion + Roam + ChatGPT + Zapier + countless SaaS tools) with one system each user fully owns

This is how we get users. Not "switch to our CMS" or "try our graph database" — "stop paying for ten pieces of software when one AI assistant, running on hardware you trust, can do all of it for you."

### Pillar 3: The Economic Engine (How It's Funded)

**Treasury interest on USD-pegged credits + compute network utilization.**
- Benten Credits are backed 1:1 by US Treasury bonds. Revenue comes from the interest (~4-5% annually) on reserves — scales with adoption without taxing users
- A peer-to-peer compute network provides secondary revenue and creates real utility for credits (peers rent storage/compute to each other, paid in credits)
- Zero transaction fees within the platform
- No mining, no staking, no gas — the treasury model makes all of this economically honest

Treasury interest is primary revenue because it's the cleanest business model: users don't pay us, they just use credits that we hold reserves for. Compute network is secondary because it's where credits get used, which keeps the flywheel turning.

### Why Three Pillars, Not Three Products

The previous framing ("three products — Application, Runtime, Economy") implied three separate businesses sharing a slogan. The three pillars framing is truer: there is one project, one engine, one user-facing story, and one business model. They depend on each other. The AI assistant needs the engine to be composable. The engine needs the AI assistant for adoption. The economics need the network effect that adoption creates. None of the three works alone.

---

## The Deepest Differentiator

**Code IS graph.** Every other decentralized platform we researched — NextGraph, Holochain, Ceramic, AT Protocol, IPFS — stores data and ships application logic separately. Benten unifies them. An operation subgraph is a bounded DAG of operation Nodes (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM) that the engine evaluates. This is the foundation for:

- **Provable execution.** Every handler walk is deterministic and content-addressed. Given the same inputs and capability state, the audit trail is reproducible.
- **Capability-gated mutation.** Every WRITE is checked against UCAN-compatible capability grants before it runs. AI agents cannot exceed delegated authority.
- **AI-native tool generation.** Because primitives compose freely, an AI assistant can generate new tools by composing subgraphs rather than generating opaque code.
- **Static analyzability.** Subgraphs are DAGs, so cost, fan-out, and determinism are computable before execution.
- **Non-Turing-completeness.** Guaranteed termination prevents DoS, enables optimistic verification.

## The Twelve Operation Primitives (revised 2026-04-14)

| # | Primitive | Purpose |
|---|-----------|---------|
| 1 | READ | Retrieve from graph |
| 2 | WRITE | Mutate graph |
| 3 | TRANSFORM | Pure data reshaping |
| 4 | BRANCH | Conditional routing |
| 5 | ITERATE | Bounded collection processing |
| 6 | WAIT | Suspend until signal or timeout |
| 7 | CALL | Execute another subgraph |
| 8 | RESPOND | Terminal: single output |
| 9 | EMIT | Fire-and-forget notification |
| 10 | SANDBOX | WASM computation (fuel-metered) |
| 11 | SUBSCRIBE | Reactive change notification |
| 12 | STREAM | Partial output with back-pressure |

**Changed from original 12:** Dropped VALIDATE (composes from BRANCH+TRANSFORM+RESPOND), GATE (capability via `requires` property). Added SUBSCRIBE (makes IVM composable) and STREAM (WinterTC streaming responses). Net: still 12, more principled set.

Empirically validated: 5 real handlers expressed in the original 12 primitives with 2.5% SANDBOX rate. Re-validation against the revised set is a Phase 1 task. See [`validation/paper-prototype-handlers.md`](validation/paper-prototype-handlers.md).

## The Social Architecture

Three tiers of community, all configuration presets on the same underlying sync + governance mechanism:

- **Atriums** (Phase 3 committed) — peer-to-peer direct connections. Two or more trusted individuals sharing subgraphs. Ships with the P2P sync layer.
- **Digital Gardens** (Phase 7 committed) — community spaces with admin-configured governance. Invitation flows, basic moderation. Member-mesh, no central server.
- **Groves** (exploratory) — governed communities with full fractal/polycentric governance, configurable voting mechanisms, fork-and-compete dynamics.

Fork is a right. Any participant can fork any subgraph at any time, keeping full history.

## Committed Scope (Phase 1-8)

| Phase | Deliverable |
|-------|-------------|
| **1** | Core engine (6 crates, 12 primitives, capability hooks, napi-rs TypeScript bindings, scaffolder + debug tooling) |
| **2** | Evaluator completion, WASM build target, wasmtime SANDBOX with fuel metering |
| **3** | P2P sync — iroh transport, CRDT merge, Merkle Search Trees, DID identity. **Atriums ship here.** |
| **4** | Thrum CMS migration to the engine (content types, blocks, compositions, existing modules + admin, 3,200+ tests pass) |
| **5** | Platform features — schema-driven rendering, self-composing admin, declarative plugin manifest format |
| **6** | **Personal AI Assistant MVP** — MCP integration, PARA knowledge organization, on-demand tool generation |
| **7** | **Digital Gardens MVP** — community spaces with admin governance, invitation flows, basic moderation |
| **8** | **Benten Credits MVP** — USD-pegged stable currency, treasury-backed reserves, FedNow on/off ramp, tab-based settlement |

Everything beyond Phase 8 is exploratory — earned through real demand, not committed by plan.

## Exploratory / Future Scope

- **Full Groves** with fractal/polycentric governance, configurable voting mechanisms — see [`future/`](future/)
- **Garden/Grove federation** — polycentric, cross-community sync
- **Knowledge attestation marketplace** — speculative attestation, AI trust signals
- **Benten Runtime** — WinterTC-compliant edge host — see [`future/benten-runtime.md`](future/benten-runtime.md)
- **`bentend` peer daemon** — general-purpose compute orchestration — see [`future/bentend-daemon.md`](future/bentend-daemon.md)
- **Peer-to-peer compute marketplace (broad)** — hardware-renting for arbitrary workloads — see [`future/compute-marketplace.md`](future/compute-marketplace.md)
- **DAO transition** — BentenAI → governed foundation
- **Governance Grove** — community governs the platform itself

Each has a revival criterion: what needs to be true for it to earn committed status.

## What Benten Is Not

- **Not a CMS.** Thrum is the CMS; Benten is the substrate Thrum runs on. CMS is one workload among many.
- **Not a blockchain.** No global consensus, no mining, no staking at the protocol level, no gas fees.
- **Not vendor lock-in.** Open-source Rust engine, WinterTC-compatible when we expose it on edge, fork-is-a-right for everything including the platform itself.
- **Not Turing complete.** Operation subgraphs are bounded DAGs. WASM sandbox (2.5% of handler operations in paper prototypes) is the controlled escape hatch.
- **Not just another decentralized platform.** The moat is the three pillars locked together: code-as-graph engine enables AI-assisted tool generation, which drives adoption, which drives credit utilization, which funds the treasury — each reinforcing the others.
- **Not an edge runtime or general-purpose compute platform (yet).** Those are candidate exploratory products. The engine and three pillars come first.

## The North Star

A world where every person runs their own AI assistant that organizes their life, builds the tools they need on demand, and replaces the paid software stack they currently depend on — all on hardware they trust, with every action cryptographically auditable, across communities that form and fork freely, funded by a zero-fee currency whose reserves earn interest rather than extracting fees from users.

## Where to Read Next

| If you want to understand... | Read |
|-----------------------------|------|
| The 10-minute quickstart | [`QUICKSTART.md`](QUICKSTART.md) |
| How the layers compose | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| Why we're building this | [`PROJECT-HISTORY.md`](PROJECT-HISTORY.md) |
| The 12 operation primitives empirically validated | [`validation/paper-prototype-handlers.md`](validation/paper-prototype-handlers.md) |
| The Rust engine blueprint | [`ENGINE-SPEC.md`](ENGINE-SPEC.md) |
| The networking/governance/sync layer | [`PLATFORM-DESIGN.md`](PLATFORM-DESIGN.md) |
| The TypeScript developer API | [`DSL-SPECIFICATION.md`](DSL-SPECIFICATION.md) |
| The economic model | [`BUSINESS-PLAN.md`](BUSINESS-PLAN.md) |
| The 8-phase committed plan | [`FULL-ROADMAP.md`](FULL-ROADMAP.md) |
| Error codes and fixes | [`ERROR-CATALOG.md`](ERROR-CATALOG.md) |
| Exploratory / future scope | [`future/`](future/) |
| Active research feeding future decisions | [`research/`](research/) |
| Glossary of Benten-specific terms | [`GLOSSARY.md`](GLOSSARY.md) |
| The process for building this | [`DEVELOPMENT-METHODOLOGY.md`](DEVELOPMENT-METHODOLOGY.md) |
| How to contribute | [`../CONTRIBUTING.md`](../CONTRIBUTING.md) |
| AI dev instructions | [`../CLAUDE.md`](../CLAUDE.md) |
