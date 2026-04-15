# Broad Compute Marketplace (Future / Exploratory)

**Status:** Candidate product. Not committed. The basic Credits + tab settlement was promoted to Phase 8 committed on 2026-04-14; what remains here is the **broader marketplace** for hardware-renting of arbitrary workloads (containers, VMs, generic compute beyond Benten's own execution).

## What Moved to Committed

The foundational economic mechanism — Benten Credits with tab-based periodic net settlement between peers — is now Phase 8 committed. See [`FULL-ROADMAP.md`](../FULL-ROADMAP.md#phase-8-benten-credits-mvp) and [`BUSINESS-PLAN.md`](../BUSINESS-PLAN.md). That includes:

- Credits with 1:1 USD peg
- Treasury bond reserve management
- FedNow on/off ramp
- Multi-sig mint/burn with HSM
- Tab-based peer-to-peer settlement
- Paying for Benten-specific compute (e.g., peers hosting a Garden's always-on node)

## What Stays Exploratory

The **broader marketplace** where peers rent hardware for *arbitrary* workloads — including workloads that aren't Benten subgraphs. This requires the `bentend` daemon (see [`bentend-daemon.md`](bentend-daemon.md)) and creates an economy separate from the platform's own compute needs.

Specifically:

1. **Arbitrary workload orchestration** — containers, VMs, generic WASM, maybe bare-metal jobs. Needs `bentend` drivers for each workload type.
2. **Hardware capability advertising at granularity** — peers advertise "RTX 4090 with 24GB VRAM, 64GB DDR5, 10Gbps network, us-east" as graph Nodes. Workloads match against this.
3. **General verification** — Benten's operation subgraphs are deterministic and cheap to verify. Arbitrary containers are not. Proof of Sampling with stake/reputation works only with mitigations (see known hard problems below).
4. **Multi-tier trust for arbitrary workloads** — a regulated AI inference workload needs TEE attestation; a batch rendering job might not. Trust requirements as declarable constraints.
5. **Hierarchical settlement for large communities** — a 10,000-peer community can't maintain direct tab counters with every other member. Needs hierarchical rollup.
6. **AI agents as economic actors for bidding/pricing** — user's agent sets prices and selects peers automatically. Blocked on resolving AI agent principal confusion (see below).

## Why This Part Is Deferred

The simpler Credits layer works standalone (Phase 8) without needing the broader marketplace. Communities can pay for always-on Benten nodes, users can transfer credits, and the treasury-backed economic engine works — all without container orchestration or arbitrary compute hosting.

The broader marketplace adds:
- Another compute runtime to ship (`bentend` with its drivers)
- Security surface for arbitrary code execution on peer hardware
- AI agent autonomous economic decisions at scale (principal confusion problem)
- Regulatory complexity (interstate compute commerce, AML for payments)

None of these are blockers for Phase 8 Credits, and each needs its own design work.

## Known Hard Problems

1. **AI agent principal confusion** (from security critic). A poisoned agent with a $100/day cap can buy compute at 1000x market rate from a colluding peer. UCAN doesn't help; the caps are valid. Needs intent declarations, two-agent quorum, anomaly-triggered human-in-loop. This is THE problem the ecosystem hasn't solved.

2. **Collusion against Proof of Sampling.** Math from security critic: with 20% sampling and 10:1 penalty, expected value of cheating is positive when colluder density > ~9%. Mitigation: VRF-selected samplers from stake-weighted pool excluding transitive trust edges; re-execution in different trust tier.

3. **Gossip amplification at scale.** At 10,000 peers, a single write propagated via GossipSub creates ~60k message deliveries. Community-level settlement needs hierarchical rollup or the protocol caps at ~1000 peers without hierarchical relays.

4. **Regulatory surface for arbitrary compute.** Renting compute across state lines and internationally may trigger regulatory requirements beyond MSB licensing. A startup shipping this without legal clarity is a pre-product regulatory bomb.

## When to Revive

Conditions for moving this proposal from `future/` to `research/`:
1. Phase 8 Credits are live and have processed real transaction volume
2. `bentend` daemon has shipped (see [`bentend-daemon.md`](bentend-daemon.md))
3. AI agent principal confusion has a working mitigation pattern (not just a theory)
4. At least one community is paying for persistent availability and expressing demand for a marketplace
5. Legal/regulatory analysis covers arbitrary compute commerce

## Source Material

See `docs/research/explore-distributed-compute-vision.md` for the full thinking. See critic findings (security) for the AI agent principal confusion problem.
