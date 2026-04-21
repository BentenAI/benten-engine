# Benten Roadmap — Committed Scope + Exploratory

**Last Updated:** 2026-04-14 (committed scope expanded to 8 phases reflecting the three pillars: engine + adoption + economics)

## The Partition

Benten's roadmap has two sections: **committed** (what we are actually building, 8 phases) and **exploratory** (proposals that earn committed status only after demand materializes).

The committed scope reflects the three-pillar vision (see [`VISION.md`](VISION.md)): ship the engine, prove it with Thrum, build the platform layer that makes it self-composing, ship the AI assistant that drives adoption, ship the Gardens that make community use real, and ship the Credits that fund the whole thing.

---

## Committed Scope (Phases 1-8)

### Phase 1: Core Engine

**The foundation.** A working Rust graph engine with TypeScript bindings that proves the code-as-graph thesis for real CRUD handlers. Scope reconciled 2026-04-14 — the shape below is what meets Phase 1's exit criteria without deferring the central architectural claims.

- 6 crates: `benten-core`, `benten-graph`, `benten-ivm`, `benten-caps`, `benten-eval`, `benten-engine`
- Node, Edge, Value types; content hashing (BLAKE3 + DAG-CBOR + CIDv1); version chains (opt-in)
- Storage via `KVBackend` trait (redb native implementation) with change-notification stream that IVM subscribes to
- Capability system as pluggable policy (`NoAuthBackend` default, UCAN backend stub)
- All 12 operation primitive *types* defined (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM)
- **Evaluator executes 8 primitives**: READ, WRITE, TRANSFORM, RESPOND, BRANCH, ITERATE, CALL, EMIT — sufficient for `crud('post')` and all IVM view maintenance. WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX ship in Phase 2.
- **Structural invariants 1-6, 9-10, 12** enforced at registration time: DAG-ness, depth, fan-out, node/edge size, determinism classification, content-hash, registration-time validation. Invariants 4, 7, 8, 11, 13, 14 ship in Phase 2 alongside the completed evaluator.
- Transaction primitive (begin/commit/rollback) exposed as first-class API
- napi-rs v3 TypeScript bindings (same codebase compiles to WASM — runtime WASM is Phase 2)
- 5 hand-written IVM views from the prototype benchmark (capability grants, event handlers, content listing, governance inheritance, version-chain CURRENT); generalized Algorithm B ships Phase 2
- Debug tooling: `subgraph.toMermaid()`, `engine.trace()` evaluation trace
- Error catalog with stable codes (spec'd in [`ERROR-CATALOG.md`](ERROR-CATALOG.md))
- `create-benten-app` scaffolder for the 10-minute DX path

**Exit criteria:** Developer can `npm install @benten/engine`, write `crud('post')`, get a working CRUD handler with audit trail visualization. All 8-primitive handler evaluations complete with causal attribution; the 5 IVM views maintain incrementally on writes; capability-gated writes route denials correctly.

### Phase 2: Evaluator Completion + WASM + SANDBOX

**Split during Phase 2a pre-R1 (2026-04-21)** into two coherent sub-phases on review-lens-coherence grounds — see `.addl/phase-2a/00-implementation-plan.md` §8. Each sub-phase runs its own full ADDL cycle.

#### Phase 2a: Evaluator completion + debt close (~5 days HE)

Structural + debt-close work with a coherent `code-as-graph-reviewer` + invariant-lens review surface.

- **`benten-eval → benten-graph` dependency break** (arch-1) — the evaluator's error surface reshapes so every downstream group builds on a clean base
- **WAIT** executor with serializable execution state on a DAG-CBOR + CIDv1 envelope (decision baked pre-R1 per §9.1 — preserves content-addressing symmetry for Phase 3 sync, Phase 6 AI workflow forking, Phase 7 Garden approval flows)
- **Four of six remaining invariants** — 8 (cumulative iteration budget multiplicative through ITERATE/CALL), 11 (system-zone full enforcement replacing Phase-1 write-path stopgap), 13 (immutability/TOCTOU), 14 (structural causal attribution per evaluation step)
- **Evaluator-path Option C** — threads `PrimitiveHost::check_read_capability` into the READ primitive so `crud:post:get` honours symmetric-None end-to-end
- **Wall-clock TOCTOU delegation** — closes the Phase-2 portion of Compromise #1
- **`DurabilityMode::Group` default** + **subgraph AST cache** — together make ENGINE-SPEC §14.6's 150–300µs 10-node-handler target reachable on macOS APFS
- `view_stale_count` metric wire-up (drops the `0.0` placeholder)
- Dev server with hot reload; per-item `missing_docs` sweep for 2a-touched items

#### Phase 2b: SANDBOX + WASM + compute (~12 days HE)

Isolation + compute work with a `wasmtime-sandbox-auditor` + `ivm-algorithm-b-reviewer` review surface. Opens pre-R1 after 2a ships. Scope frozen at `.addl/phase-2b/00-scope-outline.md`.

- **Three primitive executors** — STREAM (chunked output + back-pressure + SSE/WebSocket), SUBSCRIBE (user-visible reactive primitive), SANDBOX (wasmtime host + fuel metering + instance pool + capability-derived host-function manifest — decision baked during 2a pre-R1 §9.3)
- **Two remaining invariants** — 4 (SANDBOX nesting), 7 (SANDBOX output ≤ 1 MB)
- **Generalized IVM Algorithm B** + per-view strategy selection (A/B/C) + user-registered views beyond the 5 hand-written Phase-1 views
- **WASM build target** via napi-rs v3 with network-fetch `KVBackend` stub (snapshot-blob flavour; full iroh backend is Phase 3)
- **Module manifest format** (requires-caps, provides-subgraphs, migrations) + install/uninstall APIs
- Paper-prototype revalidation against the full 12 primitives (first measurement with all executors live — re-checks the < 30% SANDBOX-rate prediction)
- Transaction-primitive API shape finalized based on Phase 1 + 2a usage feedback

### Phase 3: P2P Sync — **Atriums Ship Here**

**Distribution.** Two or more trusted peers share subgraphs. This is the Atrium tier.

- `benten-sync` + `benten-id` crates
- iroh transport (QUIC, holepunch, relay)
- CRDT merge (per-property LWW + HLC; Loro for rich types)
- Merkle Search Tree diff for subgraph sync
- Light-client verification against content-addressed root
- DID-based identity (did:key baseline)
- Device mesh + Shamir threshold key recovery

**Exit criteria:** Two instances sync a shared subgraph bidirectionally. Key recovery works across a device mesh. An Atrium (trusted peer group) is a working social unit.

### Phase 4: Thrum Migration

**Proof of use.** Thrum CMS runs on the engine. The architecture is validated at production scope.

- CMS domain expressed as operation subgraphs (content types, blocks, compositions, field types)
- Existing Thrum modules migrate (retain module system and lifecycle hooks)
- Existing Thrum admin UI works on Benten
- 3,200+ behavioral tests pass against the Benten engine
- Performance competitive with or better than PostgreSQL + AGE baseline
- Web app renders pages; admin manages content

**Exit criteria:** Thrum's full test suite green; web app serves real pages; migration is reproducible for future modules.

### Phase 5: Platform Features

**Self-composition.** The platform layer becomes configurable via the graph.

- Schema-driven rendering (materializer pipeline as operation subgraphs — new content types render without custom components)
- Self-composing admin (admin UI layout is a graph composition that admins edit in the admin itself)
- Declarative plugin manifest format (name, version, required capabilities, provided subgraphs, migrations)
- Plugin ecosystem tooling (install/uninstall/upgrade flows)

**Exit criteria:** A third-party developer can ship a Benten module without modifying core crates. Admin UI is configurable without code changes.

### Phase 6: Personal AI Assistant MVP

**The adoption driver becomes real.** An AI assistant anchored to the user's own data that organizes their life and builds tools on demand.

- MCP (Model Context Protocol) integration — assistant calls out to LLM providers (OpenAI, Anthropic, local models)
- PARA knowledge organization (Projects, Areas, Resources, Archives as graph structures)
- On-demand tool generation — assistant composes new subgraphs from primitives to fulfill user intents ("I need a task tracker that integrates with my email")
- UCAN capability grants for the assistant's authority (spending caps, rate limits, peer-selection rules)
- Intent declaration + provenance — every agent action traces back to a user-signed intent
- Local-first execution (assistant runs on user's Benten instance, calls out to remote LLMs as needed)

**Exit criteria:** A user can talk to their assistant, have their knowledge organized PARA-style, request a tool, and have it generated and usable in minutes — all running on their own hardware with audit trails for every agent action.

### Phase 7: Digital Gardens MVP

**Community becomes real.** Beyond direct Atrium sharing — actual community spaces with admin governance.

- Garden creation flow (promote an Atrium to a Garden, or create new)
- Admin-configured governance (invitation flow, member roles, basic moderation)
- Content policies (what can be posted, how sync scope extends to new members)
- Member-mesh replication for Garden data
- Moderation tooling (content removal, member muting/banning)
- Bootstrap strategy for new-member onboarding (Merkle diff + parallel peer serving)

**Exit criteria:** A non-technical user can create a Garden, invite friends, and have a working community space with moderation. Full fractal Groves remain exploratory.

### Phase 8: Benten Credits MVP

**The economic engine turns on.** USD-pegged stable currency with treasury-backed reserves.

- Benten Credits 1:1 USD peg
- Treasury bond reserve management (70% short-term T-bills, 20% medium-term T-notes, 10% operating cash)
- FedNow on/off ramp for mint/burn
- Tab-based periodic net settlement between peers (hourly default, configurable)
- Multi-signature mint/burn with FIPS 140-3 HSMs, geographically separated signers
- Per-key rate limits, atomic mint-with-FedNow-ack
- Real-time reserve monitoring (pre-commit, not post)
- MSB registration, state money transmitter licenses as needed

**Exit criteria:** Users can buy credits with USD via FedNow, transact zero-fee within the network, redeem credits for USD. Reserves fully backed, auditable, regulatorily compliant.

---

## Exploratory Scope (Phase 9+ / Candidate Future Products)

Documented in [`future/`](future/). Each is a candidate product that earns committed status only after the committed scope ships AND real demand materializes.

### Community evolution
- **Full Groves** — fractal/polycentric governance, configurable voting mechanisms (1p1v, quadratic, conviction, liquid delegation with decay), REPLACE/EXTEND/EXEMPT override modes
- **Garden/Grove federation** — polycentric, cross-community sync, parent authority domains
- **Knowledge attestation marketplace** — speculative attestation, AI trust signals, fee distribution

### Infrastructure products
- **Benten Runtime** — WinterTC-compliant edge host, peer-distributed alternative to Cloudflare Workers — see [`future/benten-runtime.md`](future/benten-runtime.md)
- **`bentend` peer daemon** — general-purpose compute orchestration with Nomad-style pluggable drivers (containers, VMs, WASM workloads beyond ours) — see [`future/bentend-daemon.md`](future/bentend-daemon.md)
- **Peer-to-peer compute marketplace (broad)** — hardware-renting for arbitrary workloads, beyond the Benten-specific compute paid for in Phase 8 — see [`future/compute-marketplace.md`](future/compute-marketplace.md)

### Governance evolution
- **DAO transition** — four-phase shift from sole operator to community-governed foundation
- **Governance Grove** — the meta-community that governs the platform itself

---

## Revival Criteria (Exploratory → Committed)

For any `future/` proposal to move into committed scope, it must meet all four:

1. **Committed scope has shipped** and external users depend on it
2. **Concrete demand exists** for the specific feature (not inferred from architecture)
3. **A dedicated owner** (team, founder, or funded contributor) can commit to the scope
4. **The critic review** that kept it exploratory has been revisited with new information

Until then, `future/` proposals inform thinking but not engineering.

---

## Adoption Path

**Phase 1-3 (developers only):** Rust engineers and TypeScript developers interested in a new paradigm for structured, auditable, capability-gated data + logic. Research users. Niche but real.

**Phase 4-5 (developer ecosystem):** CMS developers + regulated-AI teams. Thrum on Benten is the concrete proof. Platform features make third-party module development viable. Wedge market: compliance-sensitive content workflows.

**Phase 6 (end users arrive):** Personal AI Assistant is the first user-facing product. Early adopters: people who want a self-owned alternative to Notion + ChatGPT + Zapier. The pitch shifts from technical to "you stop paying for 10 subscriptions; one system organizes everything."

**Phase 7 (community adoption):** Gardens give early adopters a way to bring friends/family/small teams onto the platform. Network effects begin.

**Phase 8 (economic flywheel):** Credits enable zero-fee transactions within the network. Peers start providing compute/storage to each other. Treasury interest scales with adoption.

**Phase 9+ (if reached):** Each exploratory proposal has its own adoption story.

---

## Timeline Philosophy

"Do it right, not fast" applies. But "do it right, not forever" also applies — Holochain's 8 years of zero production apps is the cautionary tale.

Phase 1-8 is multi-year work. The partition exists so each phase produces something usable on its own, rather than being held hostage to a full vision that keeps expanding.

Honest estimates for committed scope (AI-accelerated development):
- Phase 1-3: 8-14 months (engine + sync)
- Phase 4-5: 4-8 months (Thrum + platform features)
- Phase 6-7: 4-8 months (AI Assistant + Gardens)
- Phase 8: 6-12 months (Credits — regulatory timeline dominates)

Total committed scope: ~2-4 years to a shipped platform with engine, sync, CMS, AI assistant, community spaces, and stable currency. Exploratory scope adds years if pursued; that's why it's exploratory.
