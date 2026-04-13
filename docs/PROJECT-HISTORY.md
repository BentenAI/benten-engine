# Project History and Philosophy

## Origins

Benten Engine emerged from the Thrum project — a TypeScript universal composable platform built over V3-0 through V3-5A (approximately 10 days of AI-accelerated development, ~15 packages, 3,200+ tests). During V3-5B planning, a series of architectural explorations fundamentally reshaped the vision beyond what the V3 plan anticipated.

The Thrum TypeScript codebase lives at `/Users/benwork/Documents/thrum/` and will eventually consume the Benten Engine as its runtime foundation.

## How We Got Here

### The V3 Journey (Thrum)
- **V3-0 through V3-4.5:** Built 15 TypeScript packages — engine, store-postgres, core, CMS, auth, expressions, commerce, communications, media, themes, workflows, admin, MCP, SEO, finance. Full module system with lifecycle hooks, RestrictedEventBus, trust tiers, content types, blocks, compositions, materializer pipeline.
- **V3-5A:** Added graph-native definitions — definitions (modules, content types, blocks) written to Apache AGE graph as shadow copy. Batch `syncRegistriesToGraph()` with delete-per-label + CREATE in transaction. IoC pattern for graph sync contributors.
- **V3-5A R6 Quality Council:** 14-agent review scored 7.4/10 average.
- **V3-5.5 Checkpoint:** Validated graph populated in production web app (9 Modules, 3 ContentTypes, 22 FieldDefs).

### The Architectural Evolution
During V3-5B planning, we asked: "What comes after V3?" This triggered:
- 4 communication channel exploration (events, services, store, sync)
- Capability system design (replacing 4 fixed trust tiers with UCAN-compatible grants)
- Version chain design (history as graph, not separate tables)
- P2P sync exploration (AT Protocol, Holochain, NextGraph, CRDTs)
- The question: "Should the graph be the single-layer runtime, not just a shadow copy?"
- Database spikes (Grafeo: fast but limited; PGlite+AGE: complete but single-threaded)
- The "database IS the application" paradigm exploration
- The code-as-graph insight: handlers are subgraphs of operation Nodes, not source code strings

### The Engine Decision
No existing database combined: graph + IVM + version chains + CRDT sync + capabilities + embeddable + WASM + concurrent read/write. We decided to build a custom Rust engine.

16 critics reviewed the v1 spec (avg 4.7/10). Major revisions led to v2 (avg 6.3/10). The code-as-graph paradigm, 12 operation primitives, and IVM Algorithm B were validated through prototyping and paper-prototyping.

## The User (CEO)

Ben is the CEO and co-architect of BentenAI. Key traits for working with him:

- **"Do it right, not fast."** Quality over speed. Never recommend cutting scope or shipping early.
- **Plain English first.** Explain decisions in non-technical terms before technical details.
- **Questions reshape architecture.** A casual question like "should the graph be the runtime?" changed the entire project direction. Treat every question as potentially plan-changing.
- **Catches laziness.** If you skip a review, rationalize dead code, defer something that should be fixed now, or don't fully triage findings — he will notice and push back.
- **Wants to understand before deciding.** Present options, not decisions. Let him choose direction.
- **Thinks in systems.** He sees connections between technical architecture, business model, governance, and social dynamics. Engage at that level.
- **Values thoroughness.** He'd rather have 16 critics review a spec than skip to implementation.

## Key Philosophical Principles

1. **The graph IS the runtime.** No distinction between database and application. Data and code are both Nodes.
2. **Thin engine, everything composed.** The engine provides primitives. Everything else composes on top.
3. **Not Turing complete by design.** Guaranteed termination enables static analysis, capability pre-checking, and prevents denial-of-service.
4. **Fork is a right.** Any participant can fork any subgraph at any time. This creates evolutionary pressure on governance.
5. **Governance is the competitive dimension.** Communities differentiate through how they govern, not what technology they use.
6. **Capabilities, not tiers.** The operator decides what each entity can do. Not a central authority.
7. **Content-addressed integrity everywhere.** If it can be hashed, hash it. Verification should be structural, not trust-based.
8. **Zero-fee transactions.** The platform currency has no transaction fees. Revenue comes from treasury interest.
9. **AI-native.** AI agents are first-class citizens, not bolted-on integrations.
10. **The user owns their data.** Data lives on the user's instance. Others request access. The permission relationship is inverted from current tech.

## Naming

- **Benten** — the engine and the broader platform
- **BentenAI** — the company
- **Thrum** — the TypeScript application layer (CMS, web app, modules)
- **Atriums** — peer-to-peer direct connections (intimate, private)
- **Digital Gardens** — community spaces (curated, configurable)
- **Groves** — governed communities (fractal, polycentric, fork-and-compete)
- **Benten Credits** — the platform currency (USD-pegged initially)
