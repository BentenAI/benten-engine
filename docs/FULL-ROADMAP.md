# Benten Full Roadmap — From Engine to Ecosystem

## The Big Picture

```
FOUNDATION (the engine)
  └── Phase 1: Core engine (storage, indexes, MVCC, persistence, napi-rs bindings)
  └── Phase 2: Evaluator + IVM + capabilities (12 primitives, operation subgraphs)
  └── Phase 3: Sync + networking (CRDT, libp2p, Atrium P2P)

PLATFORM (the application layer on the engine)
  └── Phase 4: Migrate Thrum CMS (content types, blocks, compositions on the engine)
  └── Phase 5: Rendering pipeline (materializer as operation subgraphs, schema-driven rendering)
  └── Phase 6: Self-composing admin (admin UI configurable via graph)
  └── Phase 7: AI agent integration (MCP tools as capability-gated operation subgraphs)

COMMUNITY (the networking layer)
  └── Phase 8: Digital Gardens (community spaces, member-mesh, configurable governance)
  └── Phase 9: Groves (fractal governance, voting, fork-and-compete)
  └── Phase 10: Garden/Grove federation (polycentric, cross-community sync)

ECONOMY (the business model)
  └── Phase 11: Benten Credits (mint/burn, FedNow on/off ramp, treasury bonds)
  └── Phase 12: Knowledge attestation marketplace (speculative attestation, AI trust signals)
  └── Phase 13: Decentralized compute marketplace (idle compute rental)
  └── Phase 14: Token float (unpeg from USD — the "independence" moment)

GOVERNANCE EVOLUTION
  └── Phase 15: DAO transition (BentenAI → governed foundation)
  └── Phase 16: Governance Grove (community governs the platform itself)
```

## What Each Phase Delivers

### Foundation Phases (the Rust engine)

**Phase 1: Core Engine**
A working graph engine that can store/retrieve Nodes and Edges, persist to disk, and expose a TypeScript API. Proves the Rust stack works.

**Phase 2: Evaluator + IVM + Capabilities**
The engine can evaluate operation subgraphs (code-as-graph). IVM maintains materialized views. Capabilities enforce security at the engine level. This is when the engine becomes more than a database — it becomes a runtime.

**Phase 3: Sync + Networking**
CRDT sync between instances. libp2p for peer discovery and transport. Atriums work — two people can sync subgraphs peer-to-peer. This is when the "decentralized" part becomes real.

### Platform Phases (applications on the engine)

**Phase 4: Migrate Thrum CMS**
The existing Thrum CMS (content types, blocks, compositions, field types) runs on the Benten engine instead of PostgreSQL+AGE. The 3,200+ tests validate the migration. The web app serves pages.

**Phase 5: Rendering Pipeline**
Schema-driven rendering. New block types render without custom Svelte components. Three-tier fallback: custom component → category template → schema auto-render. Headless JSON API.

**Phase 6: Self-Composing Admin**
The admin UI is configurable by editing compositions in the graph. No code deployment to change the admin layout. The admin eats its own dog food.

**Phase 7: AI Agent Integration**
AI agents are first-class citizens. MCP tools as capability-gated operation subgraphs. Agents discover schema, create content, operate within their capability boundaries. The platform is AI-native.

### Community Phases (networking and governance)

**Phase 8: Digital Gardens**
Community spaces. Member-mesh (no central server). Admin/moderator governance. Content moderation. Public and private Gardens. Anyone can create a Garden and invite members.

**Phase 9: Groves**
Formal governance. Configurable voting mechanisms (1p1v, quadratic, conviction, liquid delegation). Governance rules as operation subgraphs. Sub-Groves with inheritance/override. Fork-and-compete dynamics.

**Phase 10: Federation**
Groves can federate with other Groves (polycentric). Multiple parent Groves. Cross-community sync. The mesh of communities becomes a network.

### Economy Phases (the business model)

**Phase 11: Benten Credits**
USD-pegged platform currency. Mint/burn via BentenAI. FedNow on/off ramp. Zero transaction fees. Treasury bond revenue for BentenAI. Regulatory compliance (stored-value → GENIUS Act PPSI).

**Phase 12: Knowledge Attestation Marketplace**
Accessing knowledge is free. Attesting costs a fee (community-configurable). Fees flow to existing attestors. Creates AI-consumable trust signals. Communities compete on knowledge quality. Speculation on which knowledge is most valuable.

**Phase 13: Compute Marketplace**
Small businesses run Benten servers. Idle compute rented out at near-cost. Benten Credits for purchases. Communities rent always-online nodes. Dramatically cheaper than centralized data centers.

**Phase 14: Token Float**
The big moment. Benten Credits unpeg from USD. The token's value is backed by real economic activity (attestations, compute, transactions). "The Benten economy is independent." Two-token model: pegged credits for payments + floating governance token.

### Governance Evolution

**Phase 15: DAO Transition**
BentenAI gradually decentralizes. Phase 1: sole operator. Phase 2: governance Grove has oversight. Phase 3: operations become operation subgraphs governed by the Grove. Phase 4: full DAO.

**Phase 16: Governance Grove**
The platform itself is governed by its community. Rule changes go through the governance Grove. BentenAI becomes a service provider within the ecosystem, not the controller of it.

## What Drives Adoption at Each Stage

| Phase | What attracts users |
|-------|-------------------|
| 4-6 (Platform) | Better CMS than Payload/Strapi. Schema-driven rendering. Self-composing admin. AI-native. |
| 7 (AI) | "The only platform where AI agents are graph-native citizens" |
| 8-9 (Community) | "Own your community. No platform risk. Fork if you disagree." |
| 10 (Federation) | "Connect your community to others without losing sovereignty." |
| 11-12 (Economy) | "Zero-fee transactions. Get paid for curating knowledge." |
| 13 (Compute) | "Run a server, earn credits. Cheaper than AWS." |
| 14-16 (Independence) | "The platform governs itself. No single company controls it." |

## The Competitive Moat

At each phase, the moat deepens:
- Phase 2: Only platform where code IS graph (inspectable, forkable, syncable)
- Phase 3: Only platform with native P2P sync
- Phase 7: Only platform where AI agents operate in the same graph as human users
- Phase 9: Only platform where governance is forkable and communities compete on governance quality
- Phase 12: Only platform with graph-native knowledge attestation for AI trust signals
- Phase 13: Only decentralized compute marketplace integrated with a full application platform

## Timeline Philosophy

"Do it right, not fast." AI-accelerated development compresses traditional timelines by 5-10x. The V3 Thrum project (15 packages, 3,200+ tests) took ~10 days. But database engines and distributed systems have non-linear complexity. Honest estimates:

- Foundation (Phases 1-3): 2-4 months
- Platform (Phases 4-7): 2-3 months
- Community (Phases 8-10): 2-3 months
- Economy (Phases 11-14): 3-6 months (regulatory timeline is the bottleneck)
- Governance Evolution (Phases 15-16): ongoing

Total to "functional decentralized platform with economy": 9-16 months.
Total to "fully community-governed": 18-30 months.
