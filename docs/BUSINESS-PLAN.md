# Benten Business and Economics Plan

**Created:** 2026-04-13
**Last Updated:** 2026-04-14 (Section 11 rewritten with three-pillar business model replacing earlier three-products framing; Credits promoted to Phase 8 committed)
**Status:** WORKING DRAFT -- economic model and regulatory path defined, security controls for financial operations require specification. Section 12 (2026-04-14) reflects vision evolution from pre-work.
**Audience:** Investors, advisors, business partners, and legal counsel evaluating the economic viability and regulatory compliance of the Benten platform.
**Related documents:** [Engine Spec](./ENGINE-SPEC.md) (technical foundation) | [Platform Design](./PLATFORM-DESIGN.md) (networking, governance, identity)

---

## Executive Summary

Benten is a platform for the decentralized web -- a self-evaluating graph where every person, family, or organization owns their data, shares it selectively, and forks at any time. The business model is built on a zero-fee platform currency (Benten Credits), a knowledge attestation marketplace, and a decentralized compute marketplace. Revenue comes primarily from treasury interest on credit reserves, not from taxing user transactions.

**Key economics:**
- Benten Credits are pegged 1:1 to USD (initially), with zero transaction fees within the network
- Revenue derives from investing 100% of credit reserves in Treasury bonds (~4-5% annual return)
- The attestation marketplace creates AI-consumable trust signals with community-configurable economics
- The compute marketplace enables small businesses to monetize idle capacity at near-cost
- The regulatory path progresses from stored-value product to GENIUS Act PPSI license
- The corporate structure transitions from C-Corp to DAO-governed foundation over four phases

**Market positioning:** Benten is the platform for data-sovereign communities. Unlike centralized platforms (Discord, Slack, Notion), users own their data and can fork at any time. Unlike existing decentralized platforms (Mastodon, Matrix), Benten provides a unified data model with built-in governance, economics, and AI agent support. Unlike blockchain platforms (Ethereum, Solana), Benten has zero transaction fees and runs on standard hardware.

---

## 1. The Three Tiers (User Context)

To understand the economic model, it helps to understand who uses Benten and how. For the full technical specification of these tiers, see [Platform Design, Section 1.2](./PLATFORM-DESIGN.md).

**Atriums** -- Peer-to-peer direct connections. Partners sharing finances, friends planning a trip, a student syncing with a school. Private, selective, bidirectional sync of chosen subgraphs. Each peer pays only for their own compute/storage.

**Digital Gardens** -- Community spaces. Like a Discord server, a Wikipedia, or a knowledge base -- but decentralized. Each member syncs the community graph locally. No central server required. Admin/moderator governance configures capabilities, moderation rules, and content policies.

**Groves** -- Governed communities. Fractal, polycentric, polyfederated governance. Voting on rules, smart contracts as operation subgraphs, formal decision-making. Fork-and-compete dynamics -- communities compete on governance quality.

**Economic activity by tier:**
| Tier | Typical Economic Activity | Credit Usage |
|------|--------------------------|--------------|
| Atrium | Shared expenses, personal finance | Low -- peer-to-peer transfers |
| Garden | Knowledge curation, content attestation, marketplace transactions | Medium -- attestation fees, compute marketplace |
| Grove | Governance treasury, grant funding, compute marketplace, marketplace commerce | High -- treasury management, large-scale attestation, compute rental |

---

## 2. Benten Credits (Platform Currency)

### 2.1 Mechanics

- **1 credit = $1 USD** (initially pegged)
- **Zero transaction fees** within the network
- **Mint:** User sends $1 USD -> BentenAI mints 1 credit
- **Burn:** User redeems credit -> BentenAI burns it, returns $1 USD
- **Revenue:** BentenAI invests reserves in Treasury bonds (~4-5% annual return)
- **On/off ramp:** FedNow ($0.045/tx, instant settlement)
- **Regulatory:** Stored-value product initially, GENIUS Act PPSI license when P2P distributed

### 2.2 Reserve Management

All USD received for credit minting is held in reserve and invested in US Treasury securities:

| Reserve Component | Allocation | Purpose |
|---|---|---|
| Short-term T-bills (< 1 year) | 70% | Liquidity for redemptions |
| Medium-term T-notes (1-3 years) | 20% | Yield optimization |
| Operating cash | 10% | Immediate redemption buffer |

**Revenue calculation (illustrative):**
| Credits Outstanding | Reserve Balance | Annual Treasury Yield | Annual Revenue |
|---|---|---|---|
| $1M | $1M | 4.5% | $45,000 |
| $10M | $10M | 4.5% | $450,000 |
| $100M | $100M | 4.5% | $4,500,000 |
| $1B | $1B | 4.5% | $45,000,000 |

Revenue scales linearly with adoption. The model is sustainable at any scale -- there is no minimum viable size.

### 2.3 Security Controls

The mint/burn mechanism is the highest-risk financial component. Required controls:

**Mint authority:**
- Multi-signature requirement: all mint operations require M-of-N signatures from designated key holders (minimum 2-of-3)
- Per-period rate limits: maximum credits minted per 24-hour window (configurable, starting conservative)
- Separation of functions: investment management (Treasury bonds) is handled by a separate entity/process from mint/burn operations
- Audit trail: every mint and burn operation is logged with timestamp, amount, counterparty, and authorization chain

**Burn/redemption security:**
- Cooling period: burn operations above a configurable threshold (e.g., $10,000) require a 24-hour cooling period before USD is released
- Fraud detection: velocity checks on burn requests (multiple large burns from the same account in a short window trigger review)
- FedNow settlement: FedNow transactions are instant and irrevocable -- no chargeback. The cooling period on large burns provides a window for fraud review before irrevocable settlement

**Reserve attestation:**
- Real-time reserve monitoring (not monthly): automated verification that credit supply equals reserve balance
- Monthly third-party attestation (per GENIUS Act requirements when applicable)
- Contingency plan for FedNow rail suspension: maintain relationships with multiple banking partners; if primary rails are suspended, redemptions continue through backup channels with potentially longer settlement times

---

## 3. Knowledge Attestation Marketplace

### 3.1 How It Works

- Accessing knowledge is FREE
- Attesting to knowledge costs a fee (set by community governance)
- Fees flow to existing attestors (distribution set by community)
- Creates AI-consumable trust signals: "This fact attested by N people at $X in community Y"
- Each community calibrates its own economics -- casual communities have low fees, professional communities have higher stakes
- Attestations are Edges in the graph (ATTESTED_BY), content-hashed, with cost/timestamp properties
- IVM (see [Engine Spec, Section 8](./ENGINE-SPEC.md)) materializes attestation value per knowledge Node (O(1) read for AI)

### 3.2 Community-Configurable Economics

| Parameter | Who Controls | Examples |
|---|---|---|
| Attestation fee | Community governance | Free (casual wiki), $0.10 (news), $50 (professional certification) |
| Fee distribution | Community governance | Equal split, proportional to attestation order, weighted by attestor reputation |
| Minimum attestors for "established" status | Community governance | 1 (personal blog), 10 (news), 100 (encyclopedia) |
| Attestation decay | Community governance | None (permanent), 1 year (news), 5 years (professional) |

### 3.3 Sybil Resistance

The attestation marketplace creates economic incentives for Sybil attacks (creating fake identities to be among the first attestors and collect fees from subsequent attestors). Mitigations:

**Proof-of-stake attestation:** Attestors risk losing their stake if the attestation is successfully disputed within a challenge period. The stake is proportional to the attestation fee. Disputed attestations go through a community governance process (voting, designated reviewers, or algorithmic reputation scoring, depending on community configuration).

**KYC-gated attestation:** Communities can require a minimum identity verification level for attestors. See Section 6 (Identity Verification) for the verification marketplace.

**Reputation scoring:** Attestor weight (influence on the "established" status of a knowledge node) is proportional to the attestor's history: length of membership, prior attestation accuracy (measured by dispute rate), and community standing. New accounts have low attestation weight regardless of how many credits they spend.

---

## 4. Decentralized Compute Marketplace

- Small businesses run Benten servers for their own needs
- Idle compute rented out at near-cost through the network
- Communities rent always-online nodes for persistent availability
- Benten Credits used for compute purchases
- Verification of computation through verifiable services (storage, bandwidth) initially, general compute later

### 4.1 Compute Verification

**Phase 1 (verifiable services):** Storage and bandwidth can be verified objectively (data either persists and is retrievable, or it isn't; bandwidth is measurable). Compute providers for storage and bandwidth are verified through periodic challenges: the marketplace sends a random challenge to the provider (retrieve byte range X of file Y), and the provider must respond correctly within a time limit.

**Phase 2 (general compute):** General computation cannot be verified without re-executing it (or using zero-knowledge proofs, which are computationally expensive). Phase 2 approaches include: redundant execution (2 providers execute the same task, results compared), reputation-based trust (providers build track records), and selective audit (random tasks are re-executed for verification).

**Interim risk:** In Phase 1, a compute provider offering general compute services (beyond storage/bandwidth) can accept payment and return fabricated results. The platform does not guarantee correctness of general compute in Phase 1. This limitation must be clearly communicated to marketplace participants.

---

## 5. Token Evolution

### 5.1 Phase 1: USD-Pegged Credits (Current Plan)

Benten Credits pegged 1:1 to USD. Stored-value product. Revenue from Treasury bond interest. Simple, regulatable, understandable.

### 5.2 Phase 2: Two-Token Model

- **Credits** (payment token): USD-pegged, for transactions, attestation fees, compute purchases
- **Governance token** (floating): for community governance voting weight, staking, and platform governance participation

The governance token accrues value through platform utility (governance power over platform parameters, staking rewards from attestation marketplace fees). The payment token remains stable for everyday use.

### 5.3 Phase 3: Credits Unpeg and Float

Major milestone -- "the Benten economy is independent." Credits are no longer pegged to USD. Their value reflects the economic activity of the Benten network. This transition requires:
- Sufficient economic scale (the network's internal economy is large enough to be self-sustaining)
- Robust governance (the community can manage monetary policy)
- Regulatory clarity (the legal framework supports a floating digital currency)

---

## 6. Decentralized Identity Verification

- KYC/identity verification is a marketplace of verifiers
- Users choose their provider (Persona, Jumio, community vouching, professional bodies)
- Verifiable Credentials stored as Nodes in the user's graph
- Communities decide which verification levels they require
- BentenAI maintains approved verifier list for token system compliance
- Cost borne by the user, not the platform (~$2-5 for formal KYC)

### 6.1 Trust Bootstrapping

The verifier marketplace creates a bootstrapping problem: who verifies the verifiers?

**Token system KYC:** For financial operations (mint/burn credits, compute marketplace payments), BentenAI maintains an approved verifier list. Only credentials from approved verifiers satisfy token system KYC requirements. BentenAI reviews and approves verifiers based on: regulatory compliance, audit trail quality, identity verification accuracy, and jurisdictional coverage.

**Community KYC:** For community membership, attestation, and governance, communities choose their own verifiers. This creates a two-tier system:
- Token system KYC (reliable, approved by BentenAI, required for financial operations)
- Community KYC (variable quality, chosen by each community, sufficient for non-financial operations)

Communities can choose to require token-system-level KYC for governance participation (higher assurance) or accept community-level KYC (lower barrier to entry). The choice is a governance parameter.

**Credential audience restrictions:** Verifiable Credentials include `aud` (audience) claims per the W3C VC specification. A credential issued by Verifier X for Community A specifies Community A as the audience. Community B can choose to honor it, but the credential metadata makes the cross-community usage explicit. See [Platform Design, Section 3.5](./PLATFORM-DESIGN.md) for the identity architecture.

---

## 7. Business Model

### 7.1 BentenAI as Central Bank

BentenAI operates the mint/burn mechanism for Benten Credits:
- Receives USD, mints credits (1:1)
- Invests reserves in Treasury bonds
- Revenue = interest on reserves
- Burns credits on redemption, returns USD

### 7.2 Revenue Streams

| Stream | Description | Phase | Revenue Model |
|--------|-------------|-------|---------------|
| Treasury interest | ~4-5% on credit reserves | 1 | Passive (scales with adoption) |
| Compute marketplace commission | Small % on compute transactions | 2 | Transaction-based |
| Premium features | Enterprise support, advanced analytics, SLA guarantees | 2 | Subscription |
| API access | Developer tools, integration APIs, high-volume access | 2 | Usage-based |

### 7.3 Cost Structure

| Cost Category | Phase 1 | Phase 2 | Notes |
|---|---|---|---|
| Development (engineering) | Primary cost | Ongoing | AI-accelerated, small team |
| Banking / compliance | Moderate | Growing | MSB registration, transmitter licenses, audits |
| Infrastructure | Low | Moderate | Decentralized -- users pay for their own compute |
| Customer support | Low | Growing | Community-driven support model preferred |
| Legal | Moderate | Growing | Regulatory navigation, entity structuring |

### 7.4 Competitive Positioning

| Competitor Category | Examples | Benten Advantage |
|---|---|---|
| Centralized platforms | Discord, Slack, Notion | Data sovereignty, fork rights, zero platform lock-in |
| Decentralized social | Mastodon, Matrix, Bluesky | Unified data model, built-in governance, economic layer, AI-native |
| Blockchain platforms | Ethereum, Solana | Zero transaction fees, runs on standard hardware, no mining/staking requirement |
| CMS platforms | WordPress, Strapi, Payload | Self-evaluating graph, P2P sync, governance as code |
| Knowledge platforms | Wikipedia, Stack Exchange | Economic incentives for quality (attestation marketplace), AI-consumable trust signals |

**Why not contribute to NextGraph instead?** NextGraph (EU-funded, local-first, E2EE, CRDT sync) shares approximately 80% of Benten's vision. The key differentiators: NextGraph uses RDF/SPARQL (not LPG/Cypher), which doesn't align with Thrum's existing graph model and the broader developer ecosystem. NextGraph's TypeScript ORM is tied to RDF semantics. Benten's self-evaluating graph model (code-as-graph with 12 operation primitives) has no equivalent in NextGraph. And Benten's economic layer (credits, attestation marketplace, compute marketplace) is outside NextGraph's scope. The honest assessment: contributing to NextGraph with an LPG adapter is possible but would require adapting to RDF as the core data model, which is a fundamental architectural mismatch.

---

## 8. DAO Transition

### Phase 1: BentenAI Sole Operator
BentenAI is the sole central bank operator. All monetary policy decisions (reserve allocation, rate limits, verifier approval) are made by BentenAI.

### Phase 2: Governance Oversight
A governance Grove has oversight. BentenAI continues to operate the mint/burn mechanism, but the Grove sets policy parameters (reserve allocation ratios, rate limits, verifier approval criteria). The Grove cannot directly mint or burn -- it sets the rules that BentenAI follows.

### Phase 3: Operational Subgraphs
The central bank function becomes operation subgraphs governed by the Grove. Mint/burn logic is codified as inspectable, versionable operation subgraphs (see [Engine Spec, Section 2](./ENGINE-SPEC.md) for the self-evaluating graph model). BentenAI operates the infrastructure; the Grove governs the logic.

### Phase 4: Full DAO
Full DAO governance. BentenAI becomes a service provider (infrastructure, compliance, regulatory interface), not the authority. The community governs monetary policy, verifier approval, and platform parameters through the governance mechanism described in [Platform Design, Section 4](./PLATFORM-DESIGN.md).

**Note:** The regulated fiat on/off ramp (USD <-> credits) always needs a licensed entity. Even in Phase 4, a licensed entity operates the FedNow integration and banking relationships. The DAO governs policy; the licensed entity executes within those policy bounds.

---

## 9. Regulatory Path

### 9.1 Progressive Licensing

| Phase | Regulatory Status | Requirements | Timeline |
|---|---|---|---|
| 1 | Stored-value / prepaid access product | FinCEN MSB registration, state-level exemptions | Launch |
| 2 | Money transmitter | State money transmitter licenses in key states (or partnership with licensed transmitter) | Year 1-2 |
| 3 | GENIUS Act PPSI | Federal Payment Stablecoin Issuer license (preempts state licenses) | Year 2-3+ |

### 9.2 GENIUS Act Compliance (When Applicable)

The GENIUS Act (Guiding and Establishing National Innovation for U.S. Stablecoins) defines a federal licensing framework for payment stablecoin issuers (PPSIs). Key requirements:

- **1:1 reserves** in high-quality liquid assets (Treasury securities, cash, central bank deposits)
- **Monthly reserve attestation** by an independent auditor
- **AML/KYC compliance** (BSA, FinCEN regulations)
- **Redemption rights** -- holders can redeem at par within a specified timeframe
- **Segregation of reserves** from operational funds
- **Federal oversight** (OCC for non-bank issuers, or state regulators for state-chartered issuers)

Benten's economic model is designed for GENIUS Act compliance: 100% reserves in Treasury securities, real-time reserve monitoring, KYC via the identity verification marketplace, and 1:1 redemption at par.

### 9.3 Entity Structure

**Phase 1:** Delaware C-Corp (BentenAI, Inc.)
- Standard corporate structure for fundraising and operations
- FinCEN MSB registration
- State money transmitter licenses as needed

**Phase 2:** Cayman or Swiss Foundation
- Non-profit foundation holds intellectual property and governs the protocol
- C-Corp becomes an operating entity under the foundation
- Governance token holders participate through the foundation's governance structure

**Phase 3:** DAO-Governed Foundation
- Foundation governance transitions to token-holder control
- Operating entities (including the licensed entity for fiat on/off ramp) contract with the foundation
- Regulatory relationships maintained through the licensed operating entity

---

## 10. Risk Factors

### 10.1 Economic Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Bank run (mass redemption) | Low-Medium | High | 70% short-term T-bills ensure redemption liquidity; 10% operating cash for immediate needs |
| Treasury yield decline | Medium | Medium | Revenue model works at any yield >0; diversification into additional revenue streams (Phase 2) |
| Regulatory action | Medium | High | Progressive licensing strategy; legal counsel from launch; GENIUS Act as target framework |
| FedNow rail disruption | Low | Medium | Multiple banking partnerships; backup settlement channels |
| Adoption failure | Medium | High | Credits model works at any scale; focus on community value before economic features |

### 10.2 Security Risks (Financial)

| Risk | Mitigation |
|---|---|
| Mint key compromise | Multi-signature (2-of-3 minimum), hardware security modules for key storage |
| Fraudulent burn/redemption | Cooling period on large burns, velocity-based fraud detection |
| Reserve undercollateralization | Real-time automated monitoring, monthly third-party attestation |
| Sybil attacks on attestation marketplace | Proof-of-stake attestation, reputation scoring, KYC-gated attestation for high-value communities |

### 10.3 Competitive Risks

| Risk | Assessment |
|---|---|
| NextGraph captures the data-sovereignty market | NextGraph uses RDF/SPARQL (developer adoption barrier); no economic layer; EU-grant-dependent sustainability |
| Existing platforms add P2P features | Platform economics (advertising, data monetization) are structurally incompatible with data sovereignty |
| Blockchain L2s become usable for P2P apps | Transaction fees, even small ones, create friction; blockchain platforms optimize for finance, not general-purpose communities |
| Regulatory crackdown on stablecoins | GENIUS Act provides a compliance path; progressive licensing reduces exposure |

---

## 11. The Three-Pillar Business Model (revised 2026-04-14)

Benten is one coherent business with three interdependent pillars. The previous "three products, one engine" framing implied three separate companies sharing code; critic review (2026-04-14) showed this was inaccurate and created commitment inflation. The three-pillar framing is truer to how the business actually works.

### 11.1 The Three Pillars

**Pillar 1: The Engine (what we build).** A Rust graph execution engine where every platform capability composes on top. The engine itself is not monetized directly — it's the foundation.

**Pillar 2: The Adoption Driver (how users come).** Personal AI assistants (Phase 6 committed) that organize knowledge via PARA and generate tools on demand. The pitch to end users: "stop paying for ten pieces of software; one assistant on hardware you trust does it all." The assistant itself is free (drives adoption); indirect revenue comes through credits usage.

**Pillar 3: The Economic Engine (how it's funded).** Benten Credits (Phase 8 committed) are USD-pegged, treasury-backed, zero transaction fees. Revenue is treasury bond interest (~4-5%) on reserves — scales passively with credit adoption. Secondary revenue comes from a peer-to-peer compute network (exploratory broader marketplace) that drives credit utilization.

### 11.2 How They Depend On Each Other

- **Engine → AI Assistant:** because the engine is composable (code-as-graph), the assistant can generate tools by composing primitives rather than generating opaque code. Without the engine's design, the assistant is just another chatbot.
- **AI Assistant → Credits:** the assistant drives user adoption; adopted users hold credits (to pay for peer compute, LLM inference, external services); credits require reserves; reserves earn interest.
- **Credits → Engine:** revenue from credit reserves funds engine development and the team. Without the economic layer, the project is grant-funded or unfunded.

Each pillar reinforces the others. None of the three works alone.

### 11.3 Revenue Summary

| Revenue Stream | Pillar | Status | Model |
|----------------|--------|--------|-------|
| **Treasury interest on Credits reserves** | 3 (primary) | Phase 8 committed | Passive, scales with adoption, ~4-5% of reserve balance |
| **Peer compute network utilization** | 3 (secondary) | Exploratory | Small transaction fee on marketplace, or none (just drives credit demand) |
| **Premium features / enterprise support / API access** | 1 / 2 | Phase 2+ exploratory | Subscription, usage-based, or tiered |
| **Managed hosting** (BentenAI-operated peer nodes) | 3 | Exploratory | Pay-per-use, competes with Cloudflare/etc on trust + integration |

### 11.4 Why This Model Works At Any Scale

The treasury interest model has no minimum viable size. $1M in reserves earns $40K+/year. $10M earns $400K+/year. $100M earns $4M+/year. Revenue scales linearly with adoption; there is no "we need 10M users to break even" threshold.

Credits don't extract value from users — users hold credits that Benten holds reserves for. Users' funds are protected (Treasury-backed, redeemable), and Benten earns on the time-value of the reserves. This aligns incentives: Benten succeeds by growing credit circulation, which requires making the platform useful, which requires the engine and assistant pillars to work.

### 11.5 What This Replaces

The previous three-products framing was:
- Application (communities, CMS) → now Pillar 1 (engine) and Pillar 2 (AI assistant as adoption)
- Runtime (WinterTC host, bentend) → now exploratory, not a committed product line
- Economy (credits + marketplaces) → now Pillar 3

The important simplification: BentenAI is not building three companies. It's building one company with three interlocking value streams. Investors fund one thesis, not three.

---

## 12. Open Questions (Business/Economics)

1. **Token governance design:** The Phase 2 governance token needs a detailed tokenomics design: supply schedule, distribution mechanism, staking mechanics, and governance voting weight formula. This is a separate design exercise from the platform architecture.

2. **International regulatory strategy:** The regulatory path (Section 9) focuses on US compliance. International expansion requires additional analysis: EU (MiCA regulation), UK (FCA stablecoin framework), Singapore (MAS Payment Services Act), and other jurisdictions. Each may require a separate licensed entity or partnership.

3. **Pricing for premium features and API access:** The Phase 2 revenue streams (enterprise support, analytics, API access) need pricing research. What do comparable platforms charge? What pricing model (subscription, usage-based, tiered) maximizes adoption while covering costs?

4. **Compute marketplace pricing:** How are compute prices set? Market-driven (providers set their own prices)? Algorithmic (platform suggests prices based on supply/demand)? Fixed (platform sets standard rates)? Market-driven is the most decentralized but may lead to price instability or race-to-bottom dynamics.

5. **Insurance / deposit protection:** Should Benten Credits be covered by deposit insurance (e.g., through a partnership with an FDIC-insured institution)? This would increase user trust but adds regulatory complexity and cost.
