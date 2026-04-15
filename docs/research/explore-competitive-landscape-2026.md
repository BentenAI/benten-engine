# Exploration: Competitive Landscape Research (April 2026)

**Created:** 2026-04-13
**Status:** RESEARCH SUMMARY -- findings from deep-dive research agents across 11 decentralized/P2P projects.
**Purpose:** Inform architectural decisions before Phase 1 implementation begins.

---

## Projects Researched

1. IPFS + IPLD + Filecoin
2. Akash Network
3. NextGraph
4. Matrix Protocol
5. Holochain
6. AT Protocol / Bluesky
7. GUN.js + OrbitDB + Ceramic
8. Scuttlebutt + Nostr
9. Solid + Urbit
10. DAO Governance (Aragon, MolochDAO, Compound, Optimism, Gitcoin)
11. Cosmos / IBC

---

## Patterns to Adopt

| Pattern | Source | Application to Benten |
|---------|--------|----------------------|
| CIDv1 format for content identifiers | IPFS/IPLD | 2 extra bytes per hash, full IPLD ecosystem interop |
| Light-client verification at sync | IBC/Cosmos | Verify synced data against content-addressed root hash |
| Scoped sync channels | IBC/Cosmos | Sync agreements between specific subgraphs, not whole graphs |
| Composable labeling/moderation | AT Protocol | Add labeling layer atop capabilities for content moderation |
| MST unicity principle | AT Protocol | Deterministic tree structure from key set for sync diffing |
| Reactive ORM proxy pattern | NextGraph | Study for napi-rs TypeScript binding DX |
| Convergent encryption for dedup | NextGraph | Adopt for encrypted content-addressed storage |
| Auth-topology ordering for capability changes | Matrix | Deterministic ordering for auth/governance, LWW for content |
| Fork frequency as health metric | MolochDAO | Track governance fragmentation risk |
| Human Passport integration | Gitcoin | Consider as approved Sybil-resistance verifier |
| Tab-based micro-transaction settlement | Interbank netting | Simple periodic net settlement between trusted peers |
| Proof of Sampling verification | Hyperbolic Labs | 5-10% random re-execution, reputation as penalty |
| Intent-based compute routing | CowSwap/Anoma | User expresses intent, AI agent finds optimal execution |

---

## Principles Validated

1. **Code-as-graph is our deepest differentiator.** No project has this -- not NextGraph, not Holochain, not Ceramic, not AT Protocol.
2. **Fork-is-a-right works** (MolochDAO rage-quit, Cosmos Neutron sovereignty exit).
3. **The TypeScript DSL is existential** (Solid, Holochain, Urbit all died on DX).
4. **Treasury-backed credits model is validated** (MakerDAO, GENIUS Act signed July 2025).
5. **Four-phase DAO transition is correct** (premature decentralization is the 2024-2025 antipattern).
6. **12 primitives as kernel, everything else composed** (SSB/Nostr small-protocol philosophy).
7. **Always-on infrastructure is necessary** but can be emergent from economic incentives rather than architectural requirement.
8. **Zero-fee micro-transactions enable models no blockchain can support.**

---

## Mistakes to Avoid

1. **Don't require a new paradigm** -- RDF killed Solid, Hoon killed Urbit, Rust/WASM killed Holochain adoption
2. **Don't build from scratch where commodity works** -- Urbit: 15 years, 3K users
3. **Don't ship without query capability** -- Solid's no-query pods
4. **Don't tie identity to external infrastructure** -- Urbit/Ethereum, AT Protocol/did:plc registry
5. **Don't create economic dependencies between communities** -- Cosmos Hub value-capture failure
6. **Don't let governance centralize despite rhetoric** -- Urbit galaxies, Interchain Foundation opacity
7. **Ship usable artifacts early** -- Holochain: 8 years, zero production apps
8. **Design key recovery from day one** -- GUN, OrbitDB, Ceramic all punt on this
9. **Plan for initial sync performance** -- every project struggles here
10. **Design storage compaction as incremental** -- IPFS stop-the-world GC takes 8 hours on 600GB

---

## Per-Project Key Findings

### IPFS / IPLD / Filecoin
- Adopt CIDv1 format (2 extra bytes, full ecosystem interop)
- serde_ipld_dagcbor IS the canonical IPLD codec (already switching to it)
- IPFS mistakes: no persistence without pinning, slow DHT, stop-the-world GC, 2.5% peer retention >24h
- Filecoin's complex token economics didn't drive adoption; supply outpaced demand
- IPLD "links as data type" is clean but our explicit Edges carry metadata they can't. Keep explicit Edges.

### Akash Network
- Reverse auction compute marketplace works without blockchain
- General compute verification remains unsolved without TEE/ZK -- our phased approach is correct
- Akash needed protocol-owned compute (Starcluster) to bootstrap -- cold-start problem is real
- Lease model wrong for persistent availability; model as graph relationships instead
- BME token model (burn to mint compute credits) ties demand to usage -- similar to our credits

### NextGraph
- Closest architectural peer. RDF/SPARQL vs our LPG. No code-as-graph, no economic layer, no IVM.
- Study their reactive ORM proxy pattern for napi-rs DX
- SU-set CRDT for graph mutations may inform Phase 3 merge strategy
- Convergent encryption for deduplication -- adopt rather than reinvent
- 2-tier broker architecture proves optional relay nodes work for availability

### Matrix
- State resolution v2 took years to get right; Project Hydra still hardening it in 2026
- Auth verification at merge time is the bottleneck, not the merge itself -- our IVM O(1) capability views address this
- Use deterministic ordering for capability/governance changes, LWW only for content
- E2EE rules: strict domain separation, verify on acceptance not use, channel binding
- Default server became de facto center (matrix.org effect) -- watch for this with BentenAI infrastructure
- P2P Matrix still not production-ready after 6+ years -- validates building P2P in from the start

### Holochain
- Eight years, zero production apps. "Build it right" became "build it forever."
- All-WASM requirement killed developer adoption. Our TypeScript DSL is the antidote.
- Integrity/coordinator zome split (what makes data valid vs what app does) maps to our subgraph immutability
- DHT provides automatic redundancy -- consider for Groves, simpler sync for Atriums
- Warranting system (flagging bad actors without centralized moderation) worth studying for Groves

### AT Protocol / Bluesky
- MST unicity (same keys always produce same tree) eliminates a class of sync bugs
- Speech/reach separation maps to our Atrium/Garden/Grove tiers
- Composable labeling is complementary to capabilities -- consider adding
- did:plc centralization is a known failure -- don't depend on any single registry
- AT Protocol repos are public by default -- our capability-gated access is superior for privacy
- A PDS stores and serves data; a Benten instance EVALUATES it -- deepest differentiator

### GUN.js + OrbitDB + Ceramic
- Common failure: DX cliff, sync latency, key management burden, browser scalability ceiling
- GUN's per-field CRDT granularity reduces unnecessary conflicts -- consider per-property resolution
- OrbitDB's coupling to IPFS created dependency churn -- sync should be a protocol layer, not storage dependency
- Ceramic streams are structurally similar to our version chains -- validates the approach
- Plan key recovery from day one -- all three projects punt on this

### Scuttlebutt + Nostr
- Absence of governance produces tyranny of the loudest, not freedom
- SSB pubs validate the need for persistent availability -- but economic incentives are better than volunteerism
- SSB's offline-first was right; UX killed it (peak 10K users)
- Nostr's simplicity is a legitimate challenge -- our DSL must hide complexity
- Append-only logs make edits/deletes painful -- our version chains chose correctly
- Small-protocol philosophy: 12 primitives as kernel, everything else composed

### Solid + Urbit
- Solid: RDF complexity killed adoption despite Tim Berners-Lee backing. TypeScript DSL is existential.
- Solid: no query API on pods made them useless. Our IVM + READ are essential from day one.
- Urbit: 15 years building from scratch, 3K users. Build from scratch only where insight demands it.
- Urbit: tying identity to Ethereum created gas costs and speculation. Our ed25519 is self-sovereign.
- Both validate: data sovereignty as principle, not feature. Identity must be infrastructure-independent.

### DAO Governance
- Voter turnout <10%, top 1% controls 90% voting power. Don't rush decentralization.
- MolochDAO rage-quit = our fork-right. Works as credible threat but causes treasury fragmentation. Track fork frequency.
- GENIUS Act signed July 2025. Our credits model is compliance-ready. $10B threshold for federal regulation.
- Quadratic voting insufficient alone -- Gitcoin uses 6 mechanisms simultaneously. Multi-mechanism governance is better.
- No DAO has implemented delegation decay at scale. UX friction is real.
- Knowledge attestation marketplace is genuinely novel -- no direct precedent.
- Governance attacks: flash-loan (irrelevant for us), multi-stage proposal manipulation (our fork-right mitigates).

### Cosmos / IBC
- "Blockchain of blockchains" maps to "graph of graphs" -- each Grove is sovereign but interoperates
- IBC light-client verification pattern: verify synced data against content-addressed root hash. We already have the infrastructure.
- Channel/port model for scoped sync -- sync agreements between specific subgraphs, not whole graphs
- Neutron exited shared security for full sovereignty -- treat hierarchy as optional, never dependency
- Hub tried value-capture and failed -- don't create economic dependencies between Groves
- Interchain Foundation opacity destroyed trust -- mandatory transparency as structural invariant

---

## Phase 3+ Rust Crate Landscape

| Need | Crate | Version | Ready? |
|------|-------|---------|--------|
| P2P transport | iroh | 0.97 | Near-1.0, simpler than libp2p |
| P2P transport (alternative) | libp2p | 0.56 | Production, heavier |
| CRDTs | Loro | ~1.10 | Production |
| Subgraph diffing | merkle-search-tree | stable | Well-tested, fuzzed |
| Signatures | ed25519-dalek | mature | Production |
| HLC timestamps | uhlc | 0.2 | Clean API |
| DID/VC identity | ssi (SpruceID) | 0.15 | Audited by Trail of Bits |
| WASM sandbox | wasmtime | 35+ | Production, fuel metering |
| Key rotation | keriox | recent | Functional |
| Content identifiers | ipld-core | stable | CID-compatible |
