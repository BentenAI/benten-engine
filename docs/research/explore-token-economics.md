# Exploration: Token Economics for the Benten Platform

**Created:** 2026-04-11
**Purpose:** Deep exploration of token economics models for Benten's decentralized community platform -- knowledge attestation marketplace, plutocratic governance, local community tokens, and decentralized compute marketplace. Includes honest critique of each idea, precedent analysis, alternative models, and architectural recommendations.
**Status:** Research exploration (decision input)
**Dependencies:** `SPECIFICATION.md`, `explore-blockchain-assessment.md`, `explore-content-addressed-hashing.md`, `operation-vocab-p2p.md`, `critique-holochain-perspective.md`

---

## 1. Framing: What Tokens Are and Are Not

A token is a transferable, quantifiable unit tracked by a ledger. In blockchain systems, the ledger is a smart contract. In Benten, the ledger would be the graph itself -- a Token Node with version-chained balances, capability-gated transfer operations, and CRDT sync between instances.

Tokens are NOT required for:
- Access control (capabilities do this)
- Identity (DIDs do this)
- Trust (attestation graphs do this)
- Governance (signed votes do this)
- Content moderation (community norms and admin tools do this)

Tokens ARE useful when you need:
- A scarce, transferable unit of account
- Price discovery via market mechanisms
- Incentive alignment between self-interested actors
- A coordination tool where social norms are insufficient

The burden of proof is on each token proposal: **does this use case genuinely need a transferable scarce unit, or can the same outcome be achieved with capabilities, reputation, or social mechanisms?**

---

## 2. Knowledge Attestation Marketplace

### 2.1 The Idea

- Accessing knowledge in a community is FREE
- ADDING knowledge or ATTESTING to existing knowledge costs a small fee
- All fees flow EQUALLY to existing attestors of that knowledge
- This creates speculation on which knowledge is most valuable
- Early attestors of important knowledge earn more as more people attest later

### 2.2 Precedent Analysis

**Token Curated Registries (TCRs)** are the closest existing mechanism. Conceived by Simon de la Rouviere and Mike Goldin, TCRs use token staking to curate lists. Participants stake tokens to propose list entries, challengers stake to dispute, and the winning side keeps the loser's stake.

| Mechanism | How It Works | Relevance |
|---|---|---|
| Token Curated Registries | Stake to curate a list; challengers dispute entries; losers forfeit stake | Direct precedent, but designed for binary inclusion/exclusion, not continuous attestation |
| Token Bonding Curves | Price increases with supply via a mathematical formula (linear, exponential, logarithmic); as more tokens are issued the price rises; selling burns tokens and price decreases | The right mechanism for the "early attestors earn more" dynamic |
| Prediction Markets (Augur, Polymarket) | Stake on outcomes; correct predictions pay out; incorrect ones forfeit | Analogous if you model attestation as a prediction that "this knowledge will be widely attested" |
| Curation Markets (de la Rouviere) | Bonding curve tokens tied to specific curated items; early stakers earn from later stakers | Exact match for the proposed model |
| Ocean Protocol | Data tokens representing access to datasets; staking on datasets signals quality; curated datasets earn more fees | Knowledge attestation for data, similar economic model |
| Gitcoin Quadratic Funding | Community contributions amplified by matching funds; small contributions from many people amplify signal | Alternative funding model that rewards breadth of support over depth |

**Market data (2025-2026):** Bonding curve-based DEXs hold $142.7 billion in locked value -- 78% of all decentralized exchange liquidity. Over 12% of all token launches in 2025 used bonding curves. Uniswap v4 (late 2025) added custom "hooks" enabling bonding curve + limit order hybrids. L2 scaling reduced bonding curve minting/burning costs from $50+ to under $2 on zkSync and StarkNet.

However, the cautionary data from Pump.fun is instructive: in a dataset of 655,770 newly created bonding curve tokens (September-October 2025), only 4,338 (0.63%) reached their graduation threshold. The vast majority were abandoned or worthless. Speculation overwhelms utility in unregulated token markets.

### 2.3 How It Would Work in Benten

The natural Benten implementation uses the graph:

```
[KnowledgeItem Node (anchor)]
   |-- ATTESTED_BY --> [Attestation Node: { attestor: DID, fee_paid: 0.05, timestamp: HLC }]
   |-- ATTESTED_BY --> [Attestation Node: { attestor: DID, fee_paid: 0.08, timestamp: HLC }]
   |-- ATTESTED_BY --> [Attestation Node: { attestor: DID, fee_paid: 0.12, timestamp: HLC }]
   |-- HAS_CURVE --> [BondingCurve Node: { formula: 'linear', base_price: 0.01, slope: 0.001 }]
```

Each attestation:
1. Computes the current price from the bonding curve (price = f(number_of_existing_attestations))
2. Deducts the fee from the attestor's token balance
3. Distributes the fee equally to all existing attestors
4. Creates an ATTESTED_BY edge from the knowledge item to the attestation

The bonding curve ensures early attestors pay less and earn more per subsequent attestation. Late attestors pay more but the knowledge item is "proven" valuable.

### 2.4 Critique: What Goes Wrong

**Problem 1: Attestation Spam and Gaming**

If attesting to popular knowledge earns money, rational actors will:
- Create sockpuppet accounts to self-attest and collect fees from later attestors
- Attest to EVERYTHING indiscriminately (shotgun strategy: 1000 attestations, hope some become popular)
- Form attestation cartels that coordinate early-attestation on each other's content

TCRs faced exactly these problems. As the Multicoin Capital analysis documented: "The dynamics of TCRs do not necessarily incentivize high-quality curation. Rather, they incentivize token holders to act in a way that maximizes the value of their holdings and earnings." The CoinFund analysis was blunter: "Token Curated Registries don't work" -- because the fundamental assumption that curation quality maps to token value breaks down in practice.

**Mitigation approaches:**
- **Sybil resistance:** Tie attestation to verified DIDs, not just accounts. One DID = one attestation per knowledge item. But DID farming is possible.
- **Stake-weighted attestation:** Require locking tokens for a time period, not just paying a fee. If the knowledge item is later flagged as spam/misinformation, the stake is slashed. This adds skin-in-the-game for accuracy, not just speculation.
- **Decay function:** Attestation rewards decay over time if the knowledge item receives no new attestations. This penalizes low-quality attestations that fail to attract follow-on interest.
- **Social filtering:** Attestations from people you trust (your Grove/Garden members) are weighted higher than attestations from strangers. This uses the existing social graph rather than token economics alone.

**Problem 2: Knowledge Commodification**

When you price knowledge contribution, you change the motivation. Research on intrinsic vs. extrinsic motivation (Deci & Ryan, self-determination theory) consistently shows that adding financial incentives to intrinsically motivated behavior can REDUCE quality and participation. Wikipedia contributors are not paid -- and Wikipedia is the most comprehensive knowledge base in human history. Stack Overflow introduced reputation points (not financial) -- and it works. When Stack Overflow experimented with financial incentives, it altered contribution patterns in ways the community rejected.

The "knowledge is free to access, costs to attest" model creates a two-tier system: those who can afford to attest and those who cannot. The people most likely to have genuine expertise (researchers, practitioners) are not necessarily the ones with the most tokens. The people with the most tokens (early investors, speculators) may have zero domain expertise.

**Problem 3: The Speculation Trap**

The CEO's own framing acknowledges this: "this creates speculation on which knowledge is most valuable." Speculation is a double-edged sword. It creates liquidity and price discovery, but it also creates volatility, bubbles, and misallocation. In practice, financial speculation tends to dominate the utility signal in token systems.

The Pump.fun data is instructive: 99.37% of bonding curve tokens failed to reach graduation. In a knowledge attestation market, this would mean 99% of knowledge items accumulate a handful of attestations that are economically worthless, while a tiny fraction attract speculative frenzy unrelated to knowledge quality.

### 2.5 Recommendation

The knowledge attestation idea has a genuine insight at its core: **the value of knowledge is subjective and hard to measure, and market mechanisms can aggregate subjective valuations better than any central authority.** The problem is that financial markets optimize for financial returns, not knowledge quality.

**Recommended approach: Hybrid reputation + optional token layer**

| Layer | Mechanism | Purpose |
|---|---|---|
| Free tier | Reputation points (non-transferable, non-financial) | Attest to knowledge with reputation stake. Similar to Stack Overflow upvotes. No financial barrier to participation. |
| Premium tier | Token bonding curve (opt-in per community) | Communities that WANT financial attestation can enable it. The bonding curve creates price discovery for knowledge items. But this is a community choice, not a platform default. |
| Social layer | Attestation weight from trust graph | Your attestations carry more weight within communities you are a member of. A stranger's attestation is visible but not socially weighted. |

This preserves the CEO's vision (financial attestation as a coordination mechanism) while making it opt-in and layered on top of a non-financial reputation system that provides the base case.

**Benten graph implementation:**
- Knowledge items are Nodes with ATTESTED_BY edges
- Reputation attestations are free, create lightweight edges (no token transfer)
- Token attestations (when enabled) use bonding curve Nodes attached to knowledge items
- The same knowledge item can have both reputation and token attestations
- IVM materializes "total attestation weight" as a real-time view (reputation + financial, configurable weighting)

---

## 3. Plutocratic Governance: "Voting With Your Dollars"

### 3.1 The Idea

Governance voting weighted by token holdings OR by transaction volume in the community. "You vote with your dollars -- what if literally?"

### 3.2 Precedent Analysis

This is not hypothetical -- it is the dominant governance model in DeFi/DAO ecosystems, and its failure modes are extensively documented.

**Current state of token-weighted governance (2025-2026):**

| Metric | Value | Source |
|---|---|---|
| Active DAOs | 10,000+ | Industry reports |
| DAO treasuries | $22.5 billion | Governance analytics |
| Average voter participation | <10% | Multiple studies |
| Decentraland average participation | 0.79% per proposal | On-chain analysis |
| Decentraland median participation | 0.16% per proposal | On-chain analysis |
| Top 10% of holders' voting power | 76.2% | On-chain analysis |

**Vitalik Buterin's critique (2025-2026):** Buterin explicitly warned that token voting could kill the privacy model of projects like Zcash, arguing that token-weighted governance structurally favors wealthy actors and creates "governance-extractable value" -- the ability for large holders to capture governance for their own benefit at the expense of the broader community.

### 3.3 Failure Modes

**1. Whale dominance is structural, not accidental.**

Token-weighted voting mathematically guarantees that governance power concentrates with wealth. This is not a bug -- it is the definition of the mechanism. In a community of 1000 members where one member holds 51% of tokens, that member controls every vote. The other 999 members are decorative.

Real-world DAO data confirms this is not theoretical. The top 10% of token holders control 76.2% of all voting power across major DAOs. Governance decisions are determined by fewer than ten percent of eligible token holders.

**2. Vote buying and delegation markets.**

Unbundled voting rights enable sophisticated vote buying. A whale can lend tokens to a voting proxy who votes as instructed, then returns the tokens. The whale never appears in governance records. Flash loan attacks on governance -- borrowing tokens for the duration of a vote -- have been demonstrated in DeFi (Beanstalk, $182M exploit in 2022 used a flash loan to gain governance control).

**3. Voter apathy creates a power vacuum.**

When most token holders do not vote (participation rates below 10% are standard), the few who do vote have outsized influence. This rewards intensity of preference, which correlates with intensity of financial interest, not alignment with community values.

**4. Transaction-volume weighting has its own pathology.**

If voting is weighted by transaction volume, you incentivize wash trading -- making transactions with yourself to inflate your volume and governance power. This is trivially easy in a digital system and extremely hard to detect.

**5. Ethical dimension: plutocracy vs. democracy.**

The CEO frames this honestly: "this is essentially plutocratic governance." Historical and contemporary plutocracies have a consistent pattern: policy decisions favor the wealthy at the expense of broader welfare. The United States' Gilded Age, where industrial wealth translated directly to political power, is the canonical cautionary tale. The Princeton "oligarchy" study (Gilens & Page, 2014) found that economic elites' preferences had significant impact on U.S. policy, while average citizens' preferences had near-zero independent impact.

The counterargument is that in a voluntary community (unlike a nation-state), members can exit. If governance becomes extractive, members fork. This is true -- but it requires the exit option to be genuine. If the community has network effects, switching costs, or accumulated social capital, exit is costly even if technically possible.

### 3.4 Alternative Governance Mechanisms

| Mechanism | How It Works | Strengths | Weaknesses |
|---|---|---|---|
| **Quadratic voting** | Cost of votes increases quadratically (1 vote = 1 token, 2 votes = 4 tokens, 3 votes = 9 tokens) | Reduces whale dominance mathematically; rewards breadth of support | Sybil-vulnerable (create many accounts, each casts 1 cheap vote); identity verification needed |
| **Conviction voting** | Stake tokens on proposals over time; voting power increases with duration of commitment | Rewards long-term alignment; reduces snap decisions; discourages flash-loan attacks | Complex UX; favors patient over urgent; can be gamed by staking early on many proposals |
| **Reputation-weighted voting** | Vote weight based on contribution history, not token holdings | Aligns governance power with community contribution | Reputation systems are gameable; "who decides what counts as contribution?" |
| **One-person-one-vote** | Each verified identity gets one vote, regardless of holdings | Most democratic; prevents wealth concentration | Requires Sybil resistance (identity verification); ignores information asymmetry |
| **Delegated voting (liquid democracy)** | Delegate your vote to someone you trust; delegates can re-delegate; revocable | Expertise aggregation; low-participation members can still be represented | Delegation chains can be opaque; super-delegates accumulate power |
| **Futarchy** | "Vote on values, bet on beliefs." Governance decisions resolved by prediction markets | Harnesses market efficiency for decision-making | Complex; markets for governance outcomes are thin and manipulable; defining success metrics is hard |
| **Holographic consensus** | Proposals require an attention deposit (stake); if staked, they go to a broader vote; if not, they pass by default | Scalable; filters proposals by community attention | The staking barrier itself is a form of plutocracy |

### 3.5 Recommendation

**Do not make plutocratic voting the default governance model.** The evidence from 10,000+ DAOs is overwhelming: token-weighted voting concentrates power, depresses participation, and creates governance attack surfaces.

**Recommended approach: Configurable governance modules with sensible defaults**

```
Default:        One-person-one-vote (verified DID = one vote)
Optional:       Quadratic voting (for communities that want token-weighted influence with diminishing returns)
Optional:       Conviction voting (for communities with ongoing budgeting decisions)
Optional:       Delegated voting (for communities that want expertise aggregation)
Optional:       Reputation-weighted voting (for communities that want contribution-based governance)
Advanced:       Plutocratic voting (token-weighted, for communities that explicitly choose this model)
```

This gives communities the "voting with your dollars" option but makes it an informed, explicit choice rather than the default. The platform should clearly communicate the tradeoffs (whale dominance, low participation, vote buying risk) when a community selects plutocratic governance.

**Benten graph implementation:**

Every governance mechanism is expressible as an operation subgraph. A vote is a signed Node. The tally function is a deterministic operation Node that takes the set of vote Nodes and the governance rules (mechanism type, quorum, threshold) and produces a result Node. The mechanism type parameterizes the tally function.

```
[Governance Config Node]
   |-- USES_MECHANISM --> [Mechanism Node: { type: 'quadratic', sybil_check: 'did-verified' }]
   |-- QUORUM --> [Rule Node: { minimum_participation: 0.25 }]
   |-- THRESHOLD --> [Rule Node: { approval_ratio: 0.667 }]
```

This is the right architecture because it makes governance rules inspectable, versionable, and auditable as graph data. Changing the governance mechanism is a versioned mutation with a content hash, not a code deploy.

---

## 4. Local Community Tokens (Fractal Token Hierarchy)

### 4.1 The Idea

- Each community (Garden/Grove) can have its own local token
- The main Benten token serves as the backbone/interchange (like the dollar for international trade)
- Sub-communities can derive child tokens from parent community tokens
- Two communities doing a joint venture create a parent token both convert to
- Fractal token hierarchy mirroring the fractal governance hierarchy

### 4.2 Precedent Analysis

**Historical precedent: Complementary currencies.** Local currencies have a long history outside of crypto. The WIR Bank in Switzerland (founded 1934) operates a mutual credit system with ~60,000 member businesses. The Sardex system in Sardinia processes over EUR 50 million annually in mutual credit transactions. Timebanking (everyone's hour is worth the same) has 500+ networks globally.

A 2025 study in the Journal of Economic Behavior & Organization confirmed that mutual credit systems bolster small firm resilience, particularly in economic downturns. Complementary currencies are countercyclical by design: they fill liquidity gaps when fiat currency is scarce.

**Crypto precedent: Cosmos IBC.** The Inter-Blockchain Communication Protocol (IBC) is the closest technical precedent for inter-community token exchange. IBC allows heterogeneous chains to trustlessly transfer tokens. The Cosmos Hub uses ATOM as the backbone token, with application-specific chains (Osmosis, Akash, Stride) having their own tokens that exchange via IBC. The IBCX index token aggregates exposure to the interchain ecosystem.

**DAO precedent: SubDAO hierarchies.** The Cosmos Hub has formalized a SubDAO treasury system with ATOM at the top of the hierarchy, council DAOs below, and arbitrary organizations below that. API3 uses fractal scaling -- subDAOs that operate semi-autonomously with their own budgets but align incentives by airdropping a fraction of subDAO tokens to parent DAO stakers.

**City token precedent:** Mayors from New York City, Miami, Austin, Jackson, and Tampa Bay endorsed city-branded tokens between 2021-2023. MiamiCoin launched on the Stacks blockchain and raised over $20 million for the city treasury. However, MiamiCoin's value subsequently crashed 95%+, and most city token initiatives have stalled as of 2025, demonstrating the volatility risk of community tokens tied to speculative markets.

### 4.3 Exchange Rate Mechanisms

The fractal token idea raises the critical question: **who or what sets the exchange rate between parent and child tokens?**

| Mechanism | How It Works | Properties |
|---|---|---|
| **Fixed peg** | 1 child token = X parent tokens, set by community governance | Simple, predictable; breaks if the child economy diverges from the parent; requires reserves |
| **Automated Market Maker (AMM)** | A bonding curve or constant-product pool (Uniswap-style) sets the price based on supply/demand | Market-driven price discovery; requires liquidity provision; subject to impermanent loss |
| **Mutual credit (no exchange rate)** | Child tokens are denominated differently; exchange happens via mutual credit clearing | No speculation; no liquidity requirements; requires bilateral trust; does not scale to strangers |
| **Reserve-backed** | Child token backed by a reserve of parent tokens (like a gold standard) | Stable; limits child token supply to reserve size; requires reserve management |
| **Algorithmic** | A smart contract / operation subgraph adjusts supply to target a price (like RAI/Reflexer) | Decentralized; historically fragile (see Terra/LUNA collapse, $40B loss in 2022) |

### 4.4 Critique

**Problem 1: Liquidity fragmentation.**

Every new community token fragments liquidity. If 100 Groves each have their own token, the total market is split 100 ways. Thin markets are volatile, easy to manipulate, and provide poor price discovery. This is the exact problem the EUR solved for Europe -- local currencies mean local liquidity traps.

**Problem 2: Complexity budget.**

Running a community is already hard. Adding token economics (supply policy, exchange rate management, liquidity provision, regulatory compliance) is a massive cognitive and operational burden. Most community leaders are not economists or financial engineers. The ones who want to be are probably building the wrong kind of community.

**Problem 3: Regulatory risk.**

As of 2026, the EU's Markets in Crypto-Assets Regulation (MiCA) classifies tokens as either e-money tokens, asset-referenced tokens, or utility tokens, each with different compliance requirements. A community issuing its own token may inadvertently create a regulated financial instrument. The regulatory landscape varies by jurisdiction and is changing rapidly. Even "utility tokens" face increasing scrutiny.

**Problem 4: The joint-venture token is overengineered.**

"Two communities doing a joint venture create a parent token both convert to." This can be achieved much more simply via a shared escrow account denominated in the backbone token. The overhead of creating, distributing, and managing a new token for every inter-community collaboration is enormous relative to the coordination benefit.

### 4.5 Recommendation

**The fractal token hierarchy is architecturally elegant but practically dangerous. Simplify.**

**Recommended approach: Single backbone token + community treasuries + optional local tokens**

| Component | Purpose | Implementation |
|---|---|---|
| **Backbone token** | Platform-wide unit of account; used for compute purchases, inter-community exchange, and platform governance | Benten-native, tracked in the graph, not on a blockchain (unless the community opts for anchoring) |
| **Community treasuries** | Each Grove has a treasury denominated in the backbone token; managed by community governance | Treasury Node in the graph with capability-gated transfers |
| **Optional local tokens** | Communities that WANT local tokens can create them | Bonding curve against the backbone token (automated, no manual exchange rate management); clearly communicated as high-risk and speculative |
| **Joint ventures** | Two communities doing a joint venture create a shared treasury, not a new token | Multisig-style capability grants on a shared treasury Node |

The key insight is that most of what the fractal token hierarchy achieves can be done with **capabilities and governance on shared treasuries** without the liquidity fragmentation, complexity, and regulatory risk of issuing new tokens.

**Benten graph implementation:**

```
[Backbone Token Node: { supply: 10000000, name: 'BEN' }]
   |
   |-- TREASURY_OF --> [Grove A Treasury: { balance: 50000 }]
   |                      |-- MANAGED_BY --> [Grove A Governance Config]
   |
   |-- TREASURY_OF --> [Grove B Treasury: { balance: 30000 }]
   |
   |-- SHARED_TREASURY --> [Joint Venture Treasury: { balance: 10000 }]
   |                          |-- MANAGED_BY --> [Joint Governance Config]
   |                          |-- FUNDED_BY --> [Grove A Treasury]
   |                          |-- FUNDED_BY --> [Grove B Treasury]

[Optional: Grove A Local Token]
   |-- BONDING_CURVE --> [Curve: { reserve_token: 'BEN', formula: 'sqrt', reserve_ratio: 0.5 }]
```

---

## 5. Decentralized Compute Marketplace

### 5.1 The Idea

- Users run their own servers (small businesses get a server for their needs)
- Idle compute time is rented out at near-cost
- Dramatically cheaper than centralized data centers
- The Benten token is used for compute purchases
- This creates the economic incentive to run instances

### 5.2 Precedent Analysis: The DePIN Landscape (2025-2026)

Decentralized Physical Infrastructure Networks (DePIN) is the category term. The sector has matured significantly.

| Network | Resource | Token | Scale (2025-2026) | Key Mechanism |
|---|---|---|---|---|
| **Akash Network** | General compute + GPU | AKT | 428% YoY growth; 80%+ utilization; Burn-Mint Equilibrium (March 2026) | Reverse auction: providers bid to serve workloads; lowest bid wins. AKT burned on compute purchase (deflationary). Starcluster hybrid: protocol-owned datacenters + decentralized providers. |
| **Render Network** | GPU rendering + AI inference | RENDER | $10.57 price peak May 2025; 60,000+ GPUs integrated | Render credits purchased with RENDER; node operators earn RENDER; token burns create deflationary pressure. Dispersed.com subnet: $1.75/compute-hour for AI workloads. |
| **Filecoin** | Storage | FIL | Fast Finality (100x speed, April 2025); Proof of Data Possession (May 2025) | Proof-of-Replication (sealing-time) + Proof-of-Spacetime (ongoing); miners stake FIL as collateral; slashed for downtime/data loss. |
| **Helium** | Wireless coverage | HNT | Transitioned from LoRa to 5G/WiFi; Mobile and IoT subnets | Proof of Coverage: hotspots prove location and radio coverage via challenges and witnesses. Oracle network validates claims. |
| **Golem** | General compute | GLM | Powering AI workloads, simulations, inference | Task marketplace: requestors post compute tasks; providers bid; payments via GLM. |
| **Fluence** | Serverless compute | FLT | Launched 2025 | Cloudless computing: functions run on a decentralized network of servers. |

**Market data:** As of September 2025, CoinGecko tracked nearly 250 DePIN projects with a combined market cap above $19 billion (up from $5.2 billion one year prior). Revenues projected to surpass $150 million in 2026 across the sector.

### 5.3 The Critical Problem: Verification

The fundamental challenge of a decentralized compute marketplace is: **how do you verify that the computation was actually performed?**

Storage is easier to verify (you can challenge a provider to prove they still hold a specific file -- Filecoin's Proof-of-Spacetime). Compute is harder because:

1. **You cannot verify a computation without re-doing it.** If a provider claims to have run your workload for 10 minutes, you either trust them or re-run the workload yourself (which defeats the purpose of outsourcing).

2. **Deterministic verification works only for deterministic workloads.** If the workload is pure (same inputs always produce same outputs), you can spot-check by re-running a random subset. But many real workloads are non-deterministic (timing-dependent, random-seed-dependent, GPU-floating-point-nondeterminism).

3. **Sybil attacks on compute provision:** A provider could claim to offer 8 CPU cores but actually be a single-core machine that runs your workload 8x slower. Without benchmarking, you cannot tell.

**How DePIN projects handle this:**

- **Akash:** Relies on Kubernetes-level resource reporting + reputation. Providers register their hardware specs; workload deployers choose providers based on price and reputation. No cryptographic verification of computation itself.
- **Render:** Renders are visually verifiable (the output is an image/video). AI inference outputs can be spot-checked. This works because rendering is deterministic given the same scene + parameters.
- **Filecoin:** Proof-of-Replication (one-time) + Proof-of-Spacetime (ongoing random challenges). Miners who fail challenges lose staked FIL. This is the gold standard for verifiable storage but does not apply to general compute.
- **Helium:** Proof of Coverage uses radio-frequency challenges. Physical coverage is verified by other nearby hotspots acting as witnesses.

**Emerging solution: Optimistic computation with slashing.**

1. Provider runs the workload, posts a result and a collateral stake.
2. A random verifier re-runs a fraction of the workload.
3. If the result matches, the provider is paid and the verifier gets a small verification fee.
4. If the result diverges, the provider's stake is slashed and the workload is re-routed.

This is economically secure (cheating is expensive) but not cryptographically secure (a provider who cheats on the un-verified fraction gets away with it).

**Future solution: Zero-knowledge proofs of computation (2027+).**

ZK proofs can prove that a computation was performed correctly without revealing the computation itself. The zkVM projects (RISC Zero, SP1, Jolt) are making this increasingly practical. But proof generation is currently too slow and expensive for general-purpose compute (seconds to minutes of overhead per computation). This will improve but is not viable for 2026.

### 5.4 Benten's Specific Advantage

Benten has a unique structural advantage for a compute marketplace: **the graph is the ledger.** Unlike Akash or Render, which require blockchain transactions for every compute purchase, Benten can track compute agreements, resource usage, and payments entirely within the graph. This means:

- **No transaction fees** for compute purchases (just a graph write)
- **No blockchain latency** (compute agreements are instant)
- **Local-first** (a provider and consumer in the same Garden can trade compute without any external network)
- **CRDT sync** for cross-instance compute marketplaces (price discovery across the Benten network)
- **Capability-gated** access to compute resources (fine-grained permissions)

The Benten token is the natural payment medium for this marketplace, and the compute marketplace is the natural demand driver for the token. This is the strongest economic loop in the proposal.

### 5.5 Critique

**Problem 1: The "near-cost" promise is misleading.**

Centralized data centers achieve economies of scale that individual servers cannot match. AWS, GCP, and Azure buy hardware at volume discounts, negotiate power contracts, and amortize cooling/networking across thousands of servers. A small business running a single server in a closet has higher per-unit costs for power, cooling, networking, and maintenance. The "dramatically cheaper" claim requires careful analysis.

The honest value proposition is not lower cost -- it is **different cost structure:**
- No vendor lock-in (you own the hardware)
- No surprise bills (you know your costs upfront)
- Data sovereignty (your data stays on your hardware)
- Idle capacity monetization (you are buying the server anyway; selling idle time is incremental revenue)

Akash's real-world pricing bears this out: Akash is competitive for burst workloads and specific GPU tasks, but is not universally cheaper than cloud providers for sustained compute.

**Problem 2: Reliability and SLA.**

A small business server has no redundancy, no failover, and no SLA. If the server goes down (power outage, hardware failure, internet disruption), the compute workloads it was serving go down too. Cloud providers invest billions in reliability engineering. A decentralized marketplace of individual servers will have lower reliability per-node.

Mitigation: redundant execution (run the same workload on multiple providers), but this multiplies cost.

**Problem 3: Network effects vs. chicken-and-egg.**

A compute marketplace needs both providers AND consumers. If you launch with 10 providers and 0 consumers, providers earn nothing and leave. If you launch with 10 consumers and 0 providers, consumers cannot get compute and leave. This is the classic two-sided marketplace cold-start problem.

Benten's advantage: every Benten instance is BOTH a potential provider and consumer. Running Benten = joining the marketplace. This sidesteps the cold-start problem if the platform itself generates sufficient compute demand (e.g., AI inference, content rendering, sync operations).

### 5.6 Recommendation

**The decentralized compute marketplace is the strongest token use case because it creates genuine, non-speculative demand for the Benten token.** Every compute purchase burns or transfers tokens. Every idle-capacity sale earns tokens. The token price reflects real resource demand, not speculation.

**Recommended approach: Start with what Benten itself needs**

| Phase | Scope | Token Role |
|---|---|---|
| **Phase 1: Self-hosting incentives** | Benten instances run their own compute. No marketplace yet. | No token needed. Prove the self-hosting model works. |
| **Phase 2: Sync bandwidth marketplace** | Instances that relay CRDT sync for other instances earn tokens. This is the minimal viable marketplace -- selling bandwidth, which is easy to verify (data was or was not delivered). | Token as payment for sync relay services. Bandwidth is verifiable (ACK/NACK). |
| **Phase 3: Storage marketplace** | Instances with excess storage offer it to others. Filecoin-style proof-of-storage adapted for the Benten graph. | Token as payment for storage. Proof-of-storage is well-understood. |
| **Phase 4: General compute marketplace** | Instances sell idle CPU/GPU time. Optimistic verification with slashing. | Token as payment for compute. Requires reputation + collateral + spot-checking. |

This phased approach starts with verifiable services (bandwidth, storage) before tackling the harder verification problem (general compute).

**Benten graph implementation:**

```
[Compute Provider Node (anchor)]
   |-- OFFERS --> [Resource Listing: { cpu_cores: 4, ram_gb: 16, gpu: 'RTX 4070', price_per_hour: 0.05 }]
   |-- STAKED --> [Collateral Node: { amount: 100, locked_until: HLC }]
   |-- REPUTATION --> [Reputation Node: { completed_jobs: 247, dispute_rate: 0.02, uptime: 0.994 }]

[Compute Job Node (anchor)]
   |-- ASSIGNED_TO --> [Provider Node]
   |-- PAID_BY --> [Consumer Node]
   |-- RESULT --> [Result Node: { output_hash: 'abc123', duration_seconds: 3600, verified: true }]
   |-- VERIFICATION --> [Verification Node: { verifier: DID, matches_original: true }]
```

---

## 6. Alternative Models: What If Not Tokens?

Before committing to token economics, it is worth examining non-token alternatives that achieve similar goals.

### 6.1 Reputation-Based Systems (No Tokens)

**How it works:** Every action (contributing knowledge, providing compute, participating in governance) earns non-transferable reputation points. Reputation decays over time (use-it-or-lose-it). Governance weight, resource access, and community standing are based on reputation.

**Strengths:**
- Cannot be bought, sold, or speculated on
- Directly measures contribution
- Sybil-resistant (reputation is earned, not purchased)
- No regulatory risk (not a financial instrument)

**Weaknesses:**
- Cannot be used for inter-community exchange (non-transferable by design)
- Does not create economic incentive to run infrastructure (you need something transferable for that)
- Reputation systems can be gamed (but so can token systems)

**Verdict:** Excellent for governance and knowledge attestation. Insufficient for compute marketplace (you need a transferable unit to pay for resources).

### 6.2 Mutual Credit Systems

**How it works:** Members of a community extend credit to each other. If Alice does work for Bob, Bob's account goes negative and Alice's goes positive. The total system balance is always zero. There is no "money supply" -- credit is created by transactions.

**Real-world precedent:** The WIR Bank (Switzerland) has operated a mutual credit system since 1934 with ~60,000 member businesses. Sardex (Sardinia) processes EUR 50+ million annually.

**Strengths:**
- No speculation (there is nothing to speculate on -- credit is not a token)
- Countercyclical (works better during economic downturns, when fiat is scarce)
- No liquidity fragmentation (credit is created as needed)
- Builds trust relationships (you extend credit to people you trust)

**Weaknesses:**
- Does not scale to strangers (you will not extend credit to an unknown community)
- Negative balances are obligations -- what happens when someone leaves with a negative balance?
- No price discovery mechanism (all credit units are equal, regardless of what was exchanged)
- Cannot serve as a platform-wide backbone (requires bilateral trust)

**Verdict:** Excellent for tight-knit Gardens. Does not scale to inter-community exchange or compute marketplace.

### 6.3 Time-Based Currency

**How it works:** Everyone's hour of contribution is worth the same, regardless of what they do. One hour of coding = one hour of writing = one hour of moderation. Tracked in "time credits."

**Strengths:**
- Radically egalitarian
- Simple to understand
- Non-speculative

**Weaknesses:**
- Fails to reflect skill differentiation (one hour of a security expert's time is objectively more impactful than one hour of data entry)
- Cannot price compute resources (an hour of GPU time is not the same as an hour of CPU time)
- Empirically, time banks have struggled to scale beyond small communities

**Verdict:** Interesting as a supplementary mechanism within Gardens. Not viable as the platform economic layer.

### 6.4 Attention Economy

**How it works:** Users earn tokens by contributing attention -- reading, reviewing, responding, curating. Basic Attention Token (BAT) in the Brave browser is the canonical example (100M+ monthly active users as of 2025, 1M+ verified creators).

**Strengths:**
- Rewards contribution, not wealth
- Proven at scale (Brave/BAT)
- Aligns incentives (users are rewarded for engagement, not passive holding)

**Weaknesses:**
- Attention is gameable (bots, clickfarms)
- Reduces human attention to a commodity
- BAT's specific model requires a centralized entity (Brave) to verify attention -- incompatible with Benten's decentralized model

**Verdict:** Elements of the attention economy model (rewarding contribution) should inform Benten's reputation system. The full BAT model requires centralized verification that conflicts with Benten's architecture.

---

## 7. The Backbone Token Question

### 7.1 Is a Platform-Wide Token Necessary?

**For governance:** No. Signed votes with DID verification work without tokens.

**For knowledge attestation:** Not at the base layer. Reputation is sufficient. Token-based attestation is an optional community choice.

**For the compute marketplace:** Yes. A transferable unit of account is needed to pay for resources across community boundaries. Mutual credit does not scale to strangers. Fiat currencies require payment processors and banking relationships. A platform-native token is the simplest solution for cross-community resource exchange.

**For economic incentive to run instances:** Yes. The "sell idle compute, earn tokens" loop is the most natural incentive to grow the network. Without it, the only reason to run an instance is personal use -- which limits network growth.

**Conclusion:** A platform-wide token is justified primarily by the compute marketplace. If the compute marketplace is deferred to a later phase, the token can be deferred too. If the compute marketplace is Phase 1, the token is Phase 1.

### 7.2 Native Token vs. Existing Blockchain Token

| Option | Pros | Cons |
|---|---|---|
| **Native token (tracked in the Benten graph)** | No blockchain dependency; no gas fees; instant transfers; local-first; aligns with Benten's architecture | Not tradeable on exchanges; no external liquidity; harder to bootstrap value; regulatory gray area |
| **ERC-20 on Ethereum/L2** | External liquidity; tradeable on DEXs; established tooling; regulatory clarity (MiCA classification) | Blockchain dependency; gas fees; latency; violates "no external dependencies" principle |
| **Dual token (native + bridged)** | Best of both: native for internal use, bridged for external exchange | Complexity; bridge security risks; maintaining peg |

### 7.3 Recommendation

**Start native. Bridge later if needed.**

The Benten token should be native to the graph -- a Token Node with version-chained balances, capability-gated transfers, and CRDT sync. This aligns with every architectural principle in the specification: local-first, no external dependencies, data sovereignty, graph-native.

If the platform reaches sufficient scale that external liquidity is valuable, a bridge module can wrap the native token as an ERC-20 on an L2 (exactly like the existing anchoring module recommendation from `explore-blockchain-assessment.md`). But this is a future optimization, not a launch requirement.

**Token properties:**

| Property | Value | Rationale |
|---|---|---|
| Name | BEN (working name) | Short, memorable, matches platform name |
| Supply | Fixed or predictable inflation schedule | Fixed supply creates deflationary pressure (good for holders, bad for users who need tokens for compute). A small, predictable inflation rate (1-3% annually) funds ongoing platform development and prevents hoarding. |
| Distribution | Earned by running instances and providing resources | No ICO, no pre-mine beyond development fund. Tokens enter circulation through the compute marketplace, not through speculation. |
| Minimum viable functionality | Transfer between DIDs; pay for compute/storage/bandwidth | Start minimal. Governance weight, staking, and bonding curves are optional features added per community. |
| Regulatory posture | Utility token (access to compute resources) | Position clearly as payment for services, not an investment. Avoid any language suggesting price appreciation. |

---

## 8. Architectural Integration with Benten Engine

### 8.1 Token as Graph Primitive

In the Benten engine specification, a token is simply a specialized Node:

```
[Token Definition Node (anchor): { name: 'BEN', supply: 10000000, decimals: 8, inflation_rate: 0.02 }]
   |-- CURRENT --> [Token State v47: { total_supply: 10200000, circulating: 8500000 }]

[Balance Node (anchor): { owner: did:key:alice, token: 'BEN' }]
   |-- CURRENT --> [Balance v12: { amount: 4500, locked: 1000 }]
   |-- LOCKED_BY --> [Compute Collateral Node]
```

Token transfers are operation subgraphs:

```
[Transfer Operation: { from: did:key:alice, to: did:key:bob, amount: 50, reason: 'compute_payment' }]
   |-- DEBITS --> [Alice Balance Node]
   |-- CREDITS --> [Bob Balance Node]
   |-- AUTHORIZED_BY --> [Capability: token:transfer:BEN]
```

This is fully native -- no external system needed. The same IVM that maintains materialized views for content queries maintains real-time balance views. The same CRDT sync that replicates content replicates balances. The same capability system that controls content access controls token transfers.

### 8.2 Double-Spend Prevention

The one challenge with graph-native tokens and CRDT sync is double-spending. If Alice has 100 tokens and sends 100 to Bob on Instance A and 100 to Carol on Instance B before the instances sync, both transfers appear valid locally.

**Resolution options:**

| Approach | How It Works | Properties |
|---|---|---|
| **CRDT merge + overdraft** | Both transfers apply. Alice's balance goes to -100. Overdraft is resolved by community governance (reverse one transfer, charge a penalty, etc.). | Simple; eventual consistency; requires social/governance resolution for conflicts |
| **Global ordering for transfers** | Token transfers go through a designated "mint" instance that serializes all transfers for a given token. | Prevents double-spend; introduces a centralization point and latency; single point of failure |
| **Optimistic locking with sync** | Transfers include a "balance version" reference. On sync, conflicting transfers (referencing the same balance version) are flagged. The later transfer (by HLC) is rejected. | Eventually consistent; may reject valid transfers during network partitions |
| **Operation subgraph with reserved balance** | A transfer first "reserves" the amount (locked in the balance Node), then "commits" after sync confirms no conflict. Two-phase transfer. | Most robust; adds latency; complex UX |

**Recommendation:** Start with optimistic locking (simplest, good enough for most cases). For high-value transfers, support two-phase commits as an opt-in. Document that the system is eventually consistent and that network partitions can cause temporary balance disagreements (just like real banking systems handle this -- the solution is reconciliation, not prevention).

### 8.3 Cross-Community Token Exchange

When Grove A's local token needs to exchange with the backbone BEN token:

```
[AMM Pool Node: { token_a: 'BEN', token_b: 'GROVE_A', reserve_a: 5000, reserve_b: 10000, fee: 0.003 }]
   |-- SWAP_OPERATION --> [Swap: { input: 'BEN', amount: 100, output: 'GROVE_A', amount: 196 }]
```

The AMM pool is a graph Node with a constant-product formula (x * y = k). Swaps are operation subgraphs that update both reserves atomically. No external DEX needed. IVM materializes the current exchange rate as a real-time view.

---

## 9. Risk Matrix

| Risk | Severity | Likelihood | Mitigation |
|---|---|---|---|
| Token speculation overwhelms utility | High | High | Position as utility token; no exchange listings initially; earn-only distribution |
| Regulatory classification as security | High | Medium | Clear utility function (compute payment); no investment language; legal review per jurisdiction |
| Knowledge attestation gaming | Medium | High | Hybrid reputation + optional token; social weighting; stake-and-slash |
| Plutocratic governance capture | High | High (if enabled) | Not the default; quadratic voting as recommended alternative; clear communication of tradeoffs |
| Local token liquidity collapse | Medium | High | Optional feature; bonding curve against backbone provides automated liquidity |
| Double-spend during network partition | Medium | Low | Optimistic locking; two-phase commits for high-value transfers |
| Compute verification fraud | Medium | Medium | Start with verifiable services (bandwidth, storage); optimistic verification with slashing for compute |
| Cold-start problem (no providers, no consumers) | High | Medium | Platform itself generates compute demand (AI inference, sync, rendering); every instance is both provider and consumer |
| Token hoarding (deflationary death spiral) | Medium | Medium | Small predictable inflation; compute-only earning; no passive staking rewards |

---

## 10. Questions for the CEO

### Strategic Questions

**1. Is the token necessary for V1, or is it a future phase?**

The analysis suggests phasing: reputation and governance work without tokens. The compute marketplace -- the strongest token use case -- requires significant infrastructure (resource verification, marketplace matching, SLA enforcement) that is beyond V1 scope. Recommendation: design the token architecture now, implement it when the compute marketplace is built.

**2. Should the token be on an existing blockchain or native to Benten?**

Strong recommendation: native to Benten's graph. The blockchain assessment document already established that blockchain is an optional module, not a foundational piece. The token should follow the same principle. A bridge to Ethereum/L2 can be added later for external liquidity.

**3. How do you prevent speculation from overwhelming the community purpose?**

This is the hardest problem and there is no perfect answer. The recommended mitigations:
- Earn-only distribution (no purchase of tokens without providing resources)
- No exchange listings at launch (tokens only usable within the Benten network)
- Position as utility (payment for compute), never as investment
- Community-configurable: each Grove sets its own token policies

**4. What happens to tokens when someone forks a community?**

In the Benten model, forking means copying the graph. If the graph includes token balances, the fork includes those balances. This means a fork doubles the token supply (original + fork each have a copy). Resolution options:
- Token balances are NOT included in forks (forking gives you the content but not the economy -- you start from zero)
- Token balances ARE included but the fork's tokens are a NEW token (same initial distribution, independent economy going forward)
- Community governance decides per-fork

Recommendation: token balances are not included in forks by default. Forking is an exit from a community, and taking the community's tokens with you would be extractive. The fork starts with zero tokens and must build its own economy.

### Design Questions

**5. Should the knowledge attestation marketplace be built into the platform or be a module?**

Module. Not every community needs it. The core platform provides the reputation system; the attestation marketplace is a `@benten/attestation` module that communities opt into.

**6. Should governance mechanisms be pluggable or should the platform choose one?**

Pluggable, with a default. The default should be one-person-one-vote (simplest, most democratic). Quadratic voting, conviction voting, delegated voting, and token-weighted voting should be governance modules that communities can swap in.

**7. Should the compute marketplace use the backbone token or allow community tokens?**

Backbone token only. The compute marketplace is the inter-community coordination layer -- it needs a single unit of account. Local tokens add no value here and fragment liquidity.

---

## 11. Implementation Roadmap Recommendation

| Phase | Token Economics Scope | Dependencies |
|---|---|---|
| **Engine V1** | No tokens. Build reputation system as graph Nodes. Build governance operation subgraphs (one-person-one-vote default). | Engine core (Nodes, Edges, capabilities, IVM) |
| **P2P Tiers** | No tokens. Governance module with configurable mechanisms. Knowledge attestation with reputation (free). | Sync, DIDs, signed mutations |
| **Compute Phase 1** | Backbone token (BEN) launched as graph-native token. Sync bandwidth marketplace (first verifiable service). | Token Nodes, transfer operations, capability-gated transfers |
| **Compute Phase 2** | Storage marketplace. Proof-of-storage adapted for graph. | Proof-of-storage mechanism |
| **Compute Phase 3** | General compute marketplace. Optimistic verification with slashing. | Reputation system, collateral staking, verification protocol |
| **Community Tokens** | Optional local tokens via bonding curves against BEN. Per-community governance weight configuration. | AMM pool Nodes, bonding curve operation subgraphs |
| **External Bridge** | Optional ERC-20 bridge for external liquidity (if demand warrants). | Anchoring module, blockchain integration |

---

## 12. Summary of Recommendations

| CEO Idea | Verdict | Recommendation |
|---|---|---|
| Knowledge attestation marketplace | Core insight is sound; pure token model is fragile | Hybrid: free reputation attestation (default) + opt-in token bonding curve (per community) |
| Plutocratic voting | Extensively documented failure mode in DAOs | Not the default. Configurable governance modules. Default: one-person-one-vote. |
| Fractal local community tokens | Architecturally elegant, practically dangerous | Single backbone token + community treasuries. Optional local tokens via bonding curves, with clear risk communication. |
| Decentralized compute marketplace | Strongest token use case; genuine non-speculative demand | Build in phases: bandwidth -> storage -> compute. Token launched at compute Phase 1. |

**The unifying principle:** Tokens should be earned by contributing resources, not purchased for speculation. The Benten token is a coordination tool for a decentralized infrastructure network, not a financial instrument. Every design decision should optimize for utility and contribution, not for number-go-up.

---

## References

Research conducted April 2026. Key sources:

- [Simon de la Rouviere: Verified Curation Markets & Graduating Token Bonding Curves](https://medium.com/@simondlr/verified-curation-markets-graduating-token-bonding-curves-b3885cd1108)
- [Simon de la Rouviere: Tokens 2.0 -- Curved Token Bonding in Curation Markets](https://medium.com/@simondlr/tokens-2-0-curved-token-bonding-in-curation-markets-1764a2e0bee5)
- [Multicoin Capital: Token Curated Registries -- Features and Tradeoffs](https://multicoin.capital/2018/09/05/tcrs-features-and-tradeoffs/)
- [CoinFund: Curate This -- Token Curated Registries That Don't Work](https://blog.coinfund.io/curate-this-token-curated-registries-that-dont-work-d76370b77150)
- [Predicting the Success of New Crypto-Tokens: The Pump.fun Case (2026)](https://arxiv.org/html/2602.14860v1)
- [Crypto Governance Systems Face Major Overhaul as Token Voting Crumbles](https://thecurrencyanalytics.com/altcoins/crypto-governance-systems-face-major-overhaul-as-token-voting-crumbles-250392)
- [Vitalik Buterin Warns Token Voting Could Kill Zcash Privacy Model](https://blockonomi.com/vitalik-buterin-warns-token-voting-could-kill-zcash-privacy-model)
- [Quadratic Voting vs. Conviction Voting: Optimizing DAO Governance in 2025](https://markaicode.com/quadratic-vs-conviction-voting/)
- [How DAOs Failed to Deliver on Their Original Promise (March 2026)](https://lopetaku.medium.com/dao-governance-failures-whales-low-turnout-attacks-d1375c556384)
- [Delegated Voting in Decentralized Autonomous Organizations: A Scoping Review (2025)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2025.1598283/full)
- [Decentralized AI Compute: DePIN Tokenomics & 2026 Compute Wars](https://cryptollia.com/articles/decentralized-ai-infrastructure-race-depin-tokenomics-compute-wars-2026)
- [Complete Guide to Decentralized Cloud Computing (2026) -- Fluence](https://www.fluence.network/blog/decentralized-cloud-computing-guide/)
- [Akash Network: Why Akash -- A Primer on the First Decentralized Cloud Marketplace](https://akash.network/blog/why-akash-network-a-primer-on-the-first-decentralized-cloud-marketplace/)
- [Render Network's Decentralized Cloud Computing Model (December 2025)](https://www.ainvest.com/news/render-network-decentralized-cloud-computing-model-deep-dive-sustainable-growth-treasury-dynamics-2512/)
- [Filecoin: Introducing Proof of Data Possession (May 2025)](https://filecoin.io/blog/posts/introducing-proof-of-data-possession-pdp-verifiable-hot-storage-on-filecoin/)
- [DePIN Tokenomics (Frontiers in Blockchain, 2025)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2025.1644115/full)
- [From Community Currency to Crypto City Tokens -- Belfer Center (2025)](https://www.belfercenter.org/publication/community-currency-crypto-city-tokens-potentials-shortfalls-and-future-outlooks-new-old)
- [Blockchain for Local Communities: Token Economy Aspects (Frontiers, 2024)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2024.1426802/full)
- [The Metagovernance Trilemma Across DAOs (Frontiers, 2026)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2026.1759073/full)
- [Basic Attention Token: A 2026 Guide](https://www.bitcoin.com/get-started/what-is-basic-attention-token-bat/)
- [Decentralized Reputation Systems in 2025 -- TDeFi](https://tde.fi/founder-resource/blogs/web3-strategy/why-every-web3-founder-should-care-about-decentralized-reputation-systems-in-2025-2/)
- [API3 Fractal Scaling](https://medium.com/api3/fractal-scaling-of-api3-b3ba78c9dcb7)
- [DAO DAO -- SubDAO Infrastructure on Cosmos](https://daodao.zone/)
- [Cosmos IBC -- The Internet of Blockchains](https://cosmos.network/ibc)
- [Mutual Credit Systems -- Wikipedia](https://en.wikipedia.org/wiki/Mutual_credit)
- [Sardex Mutual Credit System -- LSE Research](https://eprints.lse.ac.uk/67135/7/Dini_From%20complimentary%20currency.pdf)
- [Mitosis University: Token Curated Registries](https://university.mitosis.org/token-curated-registries-tcr/)
