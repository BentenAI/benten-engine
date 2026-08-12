# The self-balancing law — local rules for a shared bound

**Status: R0-INPUT.** A design record, not a plan; the build gets its own R0→R1 pipeline when scheduled. Lands beside `docs/future/bounded-resources.md` (D-107, single-owner admission) and answers the question that record deliberately left open: **what happens in the regime D-107's own revisit-iff points at** — a bound large relative to `n·q_p99` with genuinely multi-homed demand.

Every quantitative claim below came out of code that was run and is left in `/tmp/balance-sim/` for re-execution. Claims that were reasoned but not measured are marked **[reasoned]**. Substrate claims were re-verified first-hand at `e032ee05` by this pass, not carried from prior text. Unless stated, all simulation figures are at the harness default ρ=1.05, n=8, C=80, 8 seeds, and are `svc` (fraction of admissible demand served).

---

## 0. Headline

**In the regime that motivated D-107, no law beats single-owner, and D-107 stands unamended on the merits.** Five independently-designed local laws were built and measured against it. On the small, indivisible, short-patience bound (`museum`: n=3, C=12) single-owner scores **1.000** and the best law scores **0.982**; every law loses, and the deficit is routing, not allocation. On the high-rate bound (`ratelimit`, C=10,000) single-owner scores **1.000**. Where demand is stationary and everyone is up, single-owner wins every scenario by 0.003–0.019 while spending a third to a sixth of the messages.

Three further results matter more than the ranking:

1. **The apparent win was mostly a mis-specified floor.** Every law's headline "+0.05 to +0.28 over STATIC" is measured against an *even* split on scenarios whose demand weights are 1:1:2:2:4:4:8:8. Against a **demand-proportional static split — one line in the bound declaration, zero telemetry, zero messages, zero mechanism — 71–88% of that gap closes by configuration alone** (mean 70.0% over the six heterogeneous-demand scenarios; 82.8% excluding the non-stationary one). A correctly-configured static split scores **0.8970** mean against single-owner's **0.8470**. The residual adaptive advantage is **+0.030 to +0.073** per scenario — except on non-stationary demand, where it is **+0.2547**, the largest single number in the study.

2. **The best positive finding is not a balancing law.** A purely local rule that learns the busiest reachable peer from its own gossip and routes both units and customers there **matches pinned single-owner's service bit-for-bit** (uniform 0.9801 vs 0.9801; skewed 0.9769 vs 0.9769, all 8 metrics, 8 seeds) — and when the owner dies mid-run it scores **0.8154 against pinned single-owner's 0.4347**, recovering 38 of the 55 points a pinned owner loses. The recommendation this record ends on is therefore *keep single-owner admission; make ownership emergent* — not *replace single-owner with balancing*.

3. **Two of the commission's three framing premises did not survive measurement**, and killing them is the more useful result. The damping constant does **not** interpolate between escrow and single-owner (§5). The "peers cannot lie about having answered" availability estimator is **unforgeable and nearly useless** — it measures reachability, not cooperation, and reads a constant 1.000 for every attacker in every fully-online scenario (§4).

Two things gate any deployment beyond a single principal's own devices: **one non-lying peer can switch the whole mechanism off** (§6.4), and **the telemetry is a plaintext per-round consumption-and-presence beacon with no opt-out** (§6.5).

---

## 1. The law

Four rules. Each reads only state the node observes for itself. No coordinator, no election protocol, no global view, no shared counter.

### R1 — HOLD: lend what is idle above a floor you computed from your own arrivals

Each node keeps a floor

```
S_i = κ · λ̂_i · h̄_i            κ ≈ 1.5–2  (safety factor, per-bound)
```

where `λ̂_i` is *my own* measured arrival rate and `h̄_i` *my own* measured mean hold time (§3). Everything above `S_i` is lendable. On receiving a NEED I grant `min(asked, free − S_i − owed)`, split across all askers this round.

**Measured decomposition** (base-stock pass, `bs2/decomp.txt`, 8 seeds, mean over 8 scenarios):

| variant | mean svc | what it adds |
|---|---|---|
| STATIC | 0.7594 | — |
| **constant floor, zero telemetry** | **0.9270** | **+16.8 from "lend what's idle"** |
| **floor ∝ my own measured arrivals** | **0.9480** | **+2.1 from demand telemetry** |
| + quantile target + ownership equalisation + availability weighting | 0.9484 | **+0.04 — nothing** |

The entire base-stock/Robbins-Monro/apportionment apparatus contributes **+0.0004**. Its own tuner independently drove the ownership-equalisation rate to zero — i.e. switched the mechanism off — and *improved* the six held-out scenarios by +0.0077. **Ship R1 and stop; do not tune which apportionment rule sits on top of it until R1 is shipped and measured in the field.**

### R2 — ROUTE: when you cannot serve locally, forward the customer

Maintain per peer `j`: `a[j]` = EWMA(answered / broadcast), `q[j]` = EWMA(delivered / accepted), and `ĥ[j]` = j's last advertised free stock, discounted by *my own* receipt-clock staleness. Forward to `argmax_j ĥ[j] · a[j]^γ · q[j]`; if nothing clears a floor, refuse locally with `{available}` — never silently, never with a stale number presented as live (D-107's contract, unchanged).

**R2 is where the service is.** Measured on `skewed`, one law, identical traces, 8 seeds:

| | concentration OFF (β=0) | concentration MAX (β=32) |
|---|---|---|
| **forward OFF** | 0.6957 *(== STATIC, per-seed exact)* | **0.4739** |
| **forward ON** | **0.9287** (mv/r = 0.00) | 0.9759 |

Forwarding alone is **+23.3**. Concentration alone is **−22.2**. Concentration on top of forwarding is +4.7. The (β=0, forward-ON) cell reaches **94.6% of single-owner's service with a perfectly even allocation and zero units ever moved.**

### R3 — DAMP: deadband, backoff, minimum quantum

Act only when `|target − holding| > θ`; back off multiplicatively on a refused or unfilled ask; never move less than a minimum quantum. Measured (stigmergy pass, `uniform`):

| | msg/admission | units moved/round | write-txns at busiest node | svc |
|---|---|---|---|---|
| all damping | **4.15** | **3.47** | **1.86** | **0.970** |
| none of it | 11.41 | 15.53 | 4.21 | 0.960 |

**Damping cuts messages 2.7× and movement 4.5× while improving service.** The deadband and the backoff are load-bearing; the minimum quantum measured nearly inert. This is the cheapest real win in the design and there is no tradeoff to make.

### R4 — FAN OUT, and treat every advertised quantity as a CAP, never as a SCORE

Keep the law's total ask `T`, but spread it across *every* peer that answered rather than the top-ranked one, and use the advertised `free_now` only to cap what you ask of each peer — never to rank them. This is the security property (§6.4) and it is a shim, not a rewrite. Measured cost when nobody is attacking: **−0.0011 to +0.0012** on four of five laws (two are *better* with it). Measured benefit under one sink attacker: **0.6923 → 0.8964** (GOSSIP-AGG), **0.6923 → 0.9036** (BASESTOCK).

The one law it does not suit is pairwise-equalisation diffusion, which pays 1.8 benign points because its damping arithmetic assumes one counterparty at a time. **R4 is a constraint on which allocation arithmetic is admissible**, not an add-on: choose an allocation rule that fans out natively. The one law in the study that was immune to four simultaneous sinks out of eight (0.9673 → 0.9700) was immune *because* it split its ask by construction — structural luck, and now a requirement.

### The regime selector, stated honestly

The four rules are one law, but the operating point is not inferable from local observation. **[reasoned, with measured support]** The per-bound declaration must carry the concentration exponent, and the evidence says the right value is often zero:

- **Small C, indivisible units, short patience** (the museum: C=12, n=3, p99 party size 9) → **do not balance**. D-107's own crossover rule `C > 3·n·q_p99` reads `12 > 81` = FALSE for the real museum. Every law loses here and the mechanism is exact: any local allocation fragments the seats and idles a fragment while another node turns a walk-up away.
- **Stationary, known demand, everyone up** → **write down the split**. Adaptive advantage at perfect uptime is **+0.0036**, and at skew ≥ 8 it is **negative** (−0.0057) — single-owner wins.
- **Intermittent nodes** → the region opens the instant nodes go offline and is widest at uptime ≈ 0.90 (**+0.055**), narrowing again toward uptime 0.30.
- **Non-stationary demand** → the only large win. `shifting` (weights reverse at r200): best static 0.7223, best adaptive 0.9770.

A single law-chosen exponent was measured to be worse than no tuning at all: training the concentration exponent on `skewed` — the scenario the whole design is motivated by — produces a law scoring **0.906** across the matrix against **0.931** for shipping the untuned default. The one adaptive-exponent mechanism anybody built never reached the best fixed value and was worst where the gap was widest.

---

## 2. Telemetry — the exact payload

The measurements support a payload that is close to empty. Every field that carries a *self-reported quantity used as a score* was measured either inert or attackable.

**NEED** (broadcast to the bound's member set; simultaneously the request, the telemetry channel, and the liveness probe):

| field | source | use |
|---|---|---|
| `bound_cid` | — | which bound |
| `units` | **self-reported** | the ask. **Bounded at the lender by R1**, never trusted. The greedy-ask attack is the one attack nobody measured (§9); the lender-side recheck is what closes it **[reasoned]** |
| *(nothing else)* | | |

**OFFER** (unicast reply):

| field | source | use |
|---|---|---|
| `free_now` | **self-reported** | a **CAP on what I will ask of you**, never a rank weight |
| `grant` | signed | the transfer authorisation if the requester accepts (§7) |
| *(no demand, no uptime, no priority, no reputation)* | | |

**Derived at the receiver, purely locally, never on the wire:**

| observable | how | forgeable? |
|---|---|---|
| `a[j]` — answered / broadcast | count | **no** — but see §4: it measures reachability, not cooperation |
| `q[j]` — delivered / accepted | count | **no**, and unlike `a[j]` it costs the peer real units to satisfy |
| staleness | my own receipt clock | **no** — never use a sender-supplied timestamp; forging it forward makes a stale offer look fresh |
| `λ̂_i`, `h̄_i` | my own arrivals | **n/a** — mine |

**The deletion that matters: no peer's declared demand appears anywhere.** The one law that apportioned directly on declared demand was measured at **10.5× share gain for a ×20 claim, fleet service 0.958 → 0.745**; at ×1000 the attacker took 78% of C and the fleet collapsed to 0.263. The one law whose floor was computed only from its own measured arrivals measured demand inflation at **1.36–1.49× gain, fleet service unharmed** — and its author's explanation is the design rule: *a liar that acquires units it does not use has free stock above its own floor and offers them back next round.* **Make every threshold a function of measured local flow and inflation of any declared quantity becomes self-limiting — no cryptography, no reputation.**

---

## 3. Demand — the estimator, its time constant, and the censoring argument

**Estimator:** `λ̂_i ← (1−α)·λ̂_i + α·(arrivals observed this interval)`, and `h̄_i` likewise over completed holds. Both are counted at *arrival*, never at *admission*.

**Time constant.** The harness rounds are dimensionless, so the calibrated statement is a ratio, not a number: `1/α` must be **short relative to the demand's drift and long relative to arrival noise**. Both bounds are measured.

- *Short enough:* on `shifting`, with weights reversing at r200 and no handover mechanism, a node goes from 0.075 to 0.263 of C within **~20 rounds**, and post-flip shares land within 0.02 of the oracle's on every node.
- *Long enough:* on `museum`, where demand is genuinely uniform, a concentration exponent of 1 **amplifies Poisson noise in equal demand into unequal targets** and costs service — 0.8610 at γ=0, 0.8691 at γ=0.25, **0.7755 at γ=1**, 0.6936 at γ=2. Every seat moved to the wrong node is an immediate walk-away.

The implied local self-test — compare the estimator's own variance against the between-peer spread, and set γ→0 when the spread is not significant — **was not built and not measured**. It is the obvious way to make the exponent per-bound-emergent rather than declared, and it is the largest cheap follow-up available. **[reasoned]**

**The censoring argument (required).** The estimator must count arrivals, not admissions. Estimating from admissions samples a **censored** variable: a node short of stock refuses customers → its measured demand falls → it is allocated less → it refuses more. That is a positive feedback loop terminating in starvation, and it is invisible in aggregate service numbers because the starving node's demand has been defined away.

Implementability is not in question — a walk-away is observed at arrival even at patience 0. **This failure mode was NOT measured**: the harness calls its arrival hook unconditionally, so every law in the study estimates uncensored by construction. **[reasoned]**

There is one measured shadow of it. Weighting a static split by *declared uptime* — the natural "give less to nodes that are often away" rule — scored **0.9285 → 0.7549** on `intermit`, worse than the unweighted proportional split, because uptime is not a proxy for demand and discounting a node's share for being absent starves it when it is present. **Availability must never enter the amount.** Confirmed independently by an availability-weight sweep in a second law: 0.935 / 0.892 / 0.822 / 0.720 / 0.624 as the weight went 0 → 0.5 → 1 → 2 → 4, monotone.

---

## 4. Availability and spawners

### 4.1 The commission's premise is true and nearly useless

*"A peer cannot lie about having answered"* — correct, and it does not buy what it looks like it buys. Measured directly, reading what the honest nodes believed at end of run on `skewed`:

| law | `a[honest peer]` | `a[SINK]` | `a[FREERIDE]` | `a[pure eavesdropper]` |
|---|---|---|---|---|
| four of five laws | 1.000 | **1.000** | **1.000** | **1.000** |

Answering costs one packet. `a[j]` therefore measures **reachability**, not cooperation, and on a fully-online bound it is a constant carrying information only about outages. A node that never asks, never gives, and only listens scores the fleet maximum.

**The correction: score delivery, not answering.** `q[j] = EWMA(delivered / accepted)` is equally local, equally unforgeable, and — decisively — **costs the attacker real units to keep high.** Keep `a[j]` for outage detection and probe restriction; make `q[j]` the term that gates who you fetch from.

### 4.2 The measurement that says availability matters was invisible to the whole panel

The simulator gates arrivals on liveness only, never on reachability: **an offline node keeps serving its own customers out of its own stock.** Measured: on `intermit`, **52.2% of demand arrives at a node that is offline at that moment, and 43.8–63.2% of all service is performed by an offline node.** Being offline costs you nothing but the ability to trade — so a share parked at a dark node is never *inaccessible*, which is the precise cost Ben's spawner rule exists to pay.

All five laws independently concluded availability-weighting earns nothing. All five were correct about the instrument and all five were measuring a mechanism whose motivating cost had been set to zero.

Flip that single choice — an arrival at an offline node re-homes to a live peer, identical arrival stream so the denominator is unchanged — and the picture inverts (`intermit`, 8 seeds; **re-run and confirmed by this pass**):

| | STATIC | SINGLE-OWNER | ORACLE | best law |
|---|---|---|---|---|
| **as shipped** | 0.7075 | **0.7019** *(last)* | 0.9615 | 0.9585 |
| **demand fails over** | 0.8071 | **0.9721** *(first, beats ORACLE)* | 0.9021 | 0.8941 |

And the spawner mechanism itself, same law, one exponent, both semantics (**re-run and confirmed**):

| availability exponent on placement | harness semantics | failover semantics | strand (failover) |
|---|---|---|---|
| `a⁰` | 0.6871 | 0.7727 | 0.205 |
| `a¹` | 0.6879 **(+0.0008)** | **0.8394 (+0.0667)** | **0.149** |

**Whether the spawner idea pays is a property of a fact about the world that nobody has measured: when a node is unreachable, does its demand walk away or move to a peer?** That is one question to the museum and it decides an 80× difference in the value of the mechanism.

### 4.3 "Park your surplus at a stable peer" is structurally dead — invert it

The only law that implemented Ben's rule as stated instrumented it: **6 pushes in 400 rounds even with every threshold gutted.** The branch requires free stock above target, and the intermittent-and-hungry nodes — exactly the ones the rule protects — are never above target. Parking requires surplus; they have none. Measured effect on service across five configurations: 0.9349 → 0.9365 on `intermit`, and *identical to four decimals* on `death` and `partition`.

**The rule inverts.** A spawner is not where you park surplus; it is **a preferred source and a preferred forwarding target**. Spawner-ness = being frequently answered-and-delivered-from, and it migrates by pure local reinforcement with **no grant, no handover, no election protocol**:

- Measured migration with no handover mechanism at all: on `shifting`, node 0 goes 0.075 → 0.263 of C within ~20 rounds of the weights reversing; node 7 drains 0.250 → 0.062.
- Measured election: the local leader-learning rule of §0(2) reaches single-owner's service exactly when healthy and **0.8154 vs 0.4347 when the owner dies.**

### 4.4 Do intermittent nodes settle on stable spawners? **Refuted at the tuned point; true only with an explicit term.**

Directly measured. Share of a flaky node's inflow drawn from the stable peers, against a null equal to the stable peers' share of total peer-uptime:

| receiver (uptime 0.25) | tuned law (β=1) | with a sharp availability discount (β=4) | null |
|---|---|---|---|
| node 6 | 0.667 | **0.874** | 0.724 |
| node 7 | 0.561 | **0.850** | 0.724 |

**At the tuned point, flaky nodes draw from stable peers *below* chance.** The reason is structural and generalises: in a demand-weighted allocation the stable peers were the *low-demand* ones, so they held least, so there was least to draw from them. "Who answered" alone does not produce the effect.

It becomes true with an explicit, sharp availability exponent on **source choice** — and under the harness's (wrong) semantics that costs 0.008 of service, while under failover semantics the same term is worth +0.0667. **Honest verdict: the effect Ben predicted is real and requires an explicit mechanism, not emergence-for-free; and the only measurement taken in the right semantics is a single probe at n=8 on one scenario. Not demonstrated at design strength.**

---

## 5. Damping — and the endpoint claim, refuted

**The parameter** is the deadband `θ` (plus multiplicative backoff and a minimum quantum), and its measured value is in §1/R3: 2.7× fewer messages, 4.5× less movement, *better* service. Nothing here is a tradeoff.

**The continuum hypothesis is false.** Four independent measurements, from four agents, in agreement:

**(a) The escrow end is exact — and reaching it means switching two things off.** A zero concentration exponent reproduces STATIC *per-seed, to 1e-12, on every seed of every scenario* — verified twice, in two laws, 32/32 cells each, with **zero units moved**. But only with forwarding also off: the same law at exponent 0 with forwarding **on** is +0.048 to +0.245 *above* STATIC on all eight scenarios. And `gini = 0` is not escrow: the harness's own perfect-knowledge even-splitter sits at gini **0.151** (uniform) / **0.439** (skewed). Gini 0 is below the noise floor of any live even-splitter — it identifies *a law that did not act*, which is not the same object as escrow.

**(b) The concentrated end is not single-owner. It is the worst configuration in the entire study.** One law driven to gini **0.875**, holdings `[80,0,0,0,0,0,0,0]` — the exact single-owner signature — scores **svc 0.0338** against single-owner's 0.9811. Add `forward_target → 0`, changing no exponent, and it becomes **bit-identical to SINGLE-OWNER on all seven metrics, all 8 seeds**. Across the five laws the concentration sweep asymptotes at gini 0.395–0.798 against the target 0.875, with service *falling* the whole way, and removing every safety cap moves it by ≤0.006.

**single-owner = concentration × forwarding. The damping constant reaches only the first factor.**

**(c) Maximum damping does not give even-spread. It gives the genesis.** One maximally-damped configuration, `mv/r = 0.00`, three starting allocations: even genesis → gini **0.000**; owner genesis → gini **0.875**; lumpy genesis → gini 0.391. The same damping setting reaches both of the supposed endpoints, selected entirely by how the bound was first split. Reproduced in three laws.

**(d) The off-diagonal corner is catastrophic.** (concentrated, never forward) = **svc 0.034** against STATIC's 0.696. Two points cannot be the ends of one line if the segment between them passes through a value 20× worse than either.

**The mechanism space is a 2×2, not a line:**

|  | never forward | always forward |
|---|---|---|
| **even allocation** | **escrow** (svc 0.696) | 0.9287 — 94.6% of single-owner, zero units moved |
| **concentrated** | **0.034 — worst measured** | **single-owner** (svc 0.977) |

Escrow and single-owner are **diagonal** corners. The damping constant interpolates between the **genesis allocation** and the **exponent's fixed point**; the configuration constant that spans both endpoints *exactly, with no mechanism at all*, is the **split constant** — how much of C sits at the edge versus the centre. Verified bit-for-bit at both ends: `k=0` == SINGLE-OWNER, `k=C/n` == STATIC.

**Why the concentrated corner is unreachable under load, measured:** at ρ=1.05 only **4.5–8.9% of C is free** at any instant; the rest is busy serving and releases back to whoever admitted it, so holdings track *realised consumption* and the exponent has second-order leverage. Drop to ρ=0.20 (81% free) and the corner is genuinely reachable (gini 0.832) — but at ρ=0.20 STATIC already scores **1.0000**. The continuum exists only where nothing discriminates and collapses exactly where balancing is for.

**This was ORCH's hypothesis and it is dead. Recording its death is the point.** The replacement statement is more useful: *the split constant selects the mechanism; the damping constant governs only how fast and how far a law departs from whatever split it was given.*

---

## 6. What the numbers actually showed

### 6.1 The one comparable table

Every write-up used a different seed count. This is all eight laws on **identical seeds 1–8**:

| svc | uniform | skewed | shifting | intermit | death | partition | museum | ratelimit | mean |
|---|---|---|---|---|---|---|---|---|---|
| STATIC (even) | 0.925 | 0.696 | 0.706 | 0.707 | 0.731 | 0.703 | 0.861 | 0.745 | 0.759 |
| **STATIC-PROP** | 0.925 | **0.928** | 0.722 | **0.929** | **0.913** | **0.931** | 0.861 | **0.968** | **0.897** |
| SINGLE-OWNER | 0.979 | 0.981 | 0.978 | 0.702 | 0.435 | 0.701 | **1.000** | **1.000** | 0.847 |
| ORACLE *(privileged)* | 0.979 | 0.981 | 0.978 | 0.961 | 0.954 | 0.976 | 1.000 | 1.000 | 0.979 |
| AIMD | 0.975 | 0.971 | 0.970 | **0.958** | 0.964 | 0.951 | 0.982 | 0.996 | **0.971** |
| DIFFUSE | 0.981 | 0.977 | 0.977 | 0.944 | 0.964 | **0.972** | 0.925 | 0.996 | 0.967 |
| BASESTOCK | **0.986** | 0.974 | 0.975 | 0.875 | **0.986** | 0.881 | 0.911 | **0.999** | 0.948 |
| STIG | 0.970 | 0.969 | 0.964 | 0.879 | 0.834 | 0.931 | 0.953 | 0.997 | 0.937 |
| GOSSIP-AGG | 0.957 | 0.961 | 0.955 | 0.935 | 0.945 | 0.944 | 0.775 | 0.984 | 0.932 |

`rem%` (admissions costing a network round trip): DIFFUSE / GOSSIP-AGG / BASESTOCK **0.0** · AIMD 23.6 · STIG 32.7 · **SINGLE-OWNER 86.8**.

### 6.2 Where the baselines won

- **`museum` — single-owner 1.000, every law loses, and it is structural.** A full constant-family sweep (cover 2/8/24, exponent 0, all thresholds) moves the best law between 0.920 and 0.935 — nothing fixes it. Erlang-B, computed independently: partitioned (3×4 seats) blocking **0.2910**; pooled (1×12) **0.1748**. With patience 0 no unit can arrive in time. **When patience is zero and the pool is small, move the customer, not the unit.**
- **`ratelimit` — single-owner 1.000.** No law reaches it.
- **Stationary scenarios** — single-owner wins `uniform`/`skewed`/`shifting` by 0.003–0.019 at **1.34–1.93 msg/admission** against the laws' 0.77–52.5.
- **Under failover semantics, single-owner wins `intermit` outright (0.9721) and beats even ORACLE**, because its owner is by construction always up.
- **STATIC-PROP beats single-owner on the mean** (0.897 vs 0.847) spending no messages at all.
- **Every no-movement baseline is immune to delivery loss.** At 20% loss the laws collapse (0.49–0.68, up to 75% of the bound destroyed) while STATIC and SINGLE-OWNER are untouched.

### 6.3 The single largest correctness finding: movement without a recovery window destroys capacity

A transfer that debits the lender first and is then dropped destroys the units. Measured across four independent laws:

| loss | no recovery | recovery window |
|---|---|---|
| 10% | 0.814 (19.2 units of 80 destroyed) | **0.957** |
| 20% | 0.487–0.569 | **0.902–0.912** |
| 30% | **0.745** | **0.962** — within 0.006 of the no-loss result |

**A transfer must carry a validity window and the lender must re-credit at `exp`.** This is a correctness precondition for any unit-moving law, not an optimisation, and it is independently the highest-value component every law identified. One further measured detail that should not be lost: **reclaim latency, not lease length, bounds loss exposure** — a 5-round window beat a 20-round window decisively (0.957 vs 0.931 at 10% loss; 0.943 vs 0.830 at the concentrated extreme).

### 6.4 Security: one non-lying peer switches the mechanism off

The **SINK** — answer every broadcast, advertise honestly, never deliver. It moves no units, forges nothing, needs no colluder, and for the offer-ranking law **needs no lie at all** (the sweep from ×1 to ×100 is flat at 0.8683 because the attacker reports its free stock honestly).

`skewed`, one attacker at the *coldest* node, 6 seeds (**re-run and confirmed by this pass**):

| law | honest svc | **SINK svc** | honest units moved/round | **SINK units moved/round** |
|---|---|---|---|---|
| GOSSIP-AGG | 0.9579 | **0.6923** *(== STATIC exactly)* | 1.80 | **0.00** |
| BASESTOCK | 0.9716 | **0.6923** *(== STATIC exactly)* | 21.52 | **0.00** |
| DIFFUSE | 0.9744 | 0.8852 | 2.50 | 0.71 |
| AIMD | 0.9691 | 0.9500 | 0.82 | 0.04 |
| STIG *(splits its ask by construction)* | 0.9673 | **0.9690** | 3.59 | 3.33 |

**Two laws are reduced to the static baseline, for the whole run, by one peer at zero cost.** And a sink is *indistinguishable from the benign race* the simulator deliberately models (a customer takes the stock between offer and accept), so it cannot be punished without punishing honest nodes.

**Second finding, methodological and worse: `svc` is not a security metric.** One attacker took **64% of a bounded resource while fleet service rose by +0.007**; another took **62% of C at +0.0026**; a cold-node attacker took **16.4× its honest twin's share at −0.0003**. On a bounded resource the attacker's objective is the **share**, not the service rate, and four of five laws hand it over with no service signal or a positive one. **Every table in this domain must report attacker-share-of-C and fleet-units-moved alongside service.**

**No shipped defence defended.** A purpose-built reciprocity gate recovered +0.0175 of a 0.1061 loss and exactly 0.0000 against two other attacks; a reinforcement trail and an availability penalty were worth **zero to three decimals**. What worked is R4 (§1): fan out, cap never score. It degrades gracefully with attacker density (0.896 / 0.826 / 0.767 / 0.713 at 1/2/3/4 sinks out of 8, against 0.6923 undefended at all four).

### 6.5 The telemetry is a fleet-wide activity beacon with no opt-out

A passive member that never broadcasts, answers `Offer(0)`, and is indistinguishable from a well-behaved peer with no surplus reconstructs, on `skewed`, 6 seeds:

| law | corr(true demand weights, inferred) | liveness accuracy | precision | free-stock error |
|---|---|---|---|---|
| five laws | **0.937 – 0.999** | 0.928 – **1.000** | **1.000** | 0.01 – 0.82 of a fair share |

There is no opt-out: **broadcasting is simultaneously the request**, so a node that stops disclosing stops being served. The audience is the whole bound — up to **32** members for an Atrium. For a project whose confidentiality thesis is *peers hold ciphertext*, this ships a plaintext, unauthenticated, per-round consumption-and-presence timeseries for every principal to every other principal. **It is not a leak in the mechanism; it is the mechanism.**

### 6.6 The engine write path, measured on the pinned redb

Executed against redb 4.1.0 (the `=` pin at `e032ee05`) at the default `Durability::Immediate` path:

- One write transaction: **4.74 ms / 211 ops/s** clean; 4.49 ms / 223 under host load.
- `begin_write` **blocks; contention becomes tail, not throughput**: aggregate flat at 203–230 ops/s across 1/2/4/8 threads, **0 errors**, max latency 11 ms → **3356 ms**.
- **D-107's falsification arm 1, executed:** count inside the write txn → admitted exactly 12 of 480 attempts against cap 12, **oversell 0**. Count in a separate read txn → admitted **19, oversell +7 (58%)**. The mechanism is sound and the misplacement oversells by 58% on a 12-seat bound.
- **The sharpening D-107 needs:** under the check as written, *a refusal costs a full write transaction* — 468 refusals consumed 2.19 s of the global single-writer lock to admit 12. A **read pre-check followed by the same in-tx confirm** admits exactly 12 with oversell 0 at **0.175 ms/attempt against 4.554 — 26× cheaper, identical soundness.** D-107 names "check twice, enforce once" as UX; at ρ≈1 it is a throughput requirement.
- **D-107's O(k) scan claim holds exactly where it is scoped and breaks where it says it will:** k=12 → 0.015 ms; k=400 → 0.060 ms; **k=10,000 → 0.992 ms mean / 3.371 ms p99**; k=100,000 → 8.693 ms. The record's "revisit-iff k ≫ 10³" now has a number. Note the harness's own high-rate scenario is C=10,000.

### 6.7 Instrument defects that bound what any of this can claim

Reported here because they bound the conclusions, not to disparage a genuinely honest, self-testing, reproducible instrument. **Every headline table in every write-up reproduced exactly** under three independent re-run audits, including this pass's own spot-checks.

1. **Offline nodes serve their own demand** (§4.2) — sets the cost of unavailability to zero and thereby decides the spawner question.
2. **A law cannot express "forward even though I could serve"** — the local-admit branch is unconditional. Half of single-owner was inexpressible, which is why four of five laws reported the endpoint unreachable as a *finding* rather than an instrument limit.
3. **The privileged baseline moves units through a back-channel** — 3.32 units/round on `skewed` at zero messages, zero write transactions, and **zero loss at any loss rate**. Its entire lead on `skewed` is the *pooled serving* privilege: its repositioning is worth exactly **0.0000** there. It is a **routing** upper bound, not an allocation one.
4. **The `death` scenario conflates three roles in one node** (hottest = pinned owner = the one that dies). Decomposed: single-owner's 0.435 collapse is **100% attributable to the owner being the node that dies and 0% to concentration**; move the death anywhere else and single-owner is the best row in the table (0.980–0.990, beating the oracle).
5. **The service denominator is not an upper bound above ρ≈1.3** (observed to 1.034) — it assumes a global FIFO discipline the laws need not obey. **At the default ρ=1.05 every headline table is safe** (max observed 0.9892); every ρ-sweep above 1.3 in any write-up is measured against a broken denominator.
6. **A wrong-guess forward is free** — up to **90.3% of forward attempts are unbilled**, understating single-owner's message cost by up to **10×** and flattering the very axis this record recommends.
7. **Conservation is a harness gift.** Capacity is conserved physical tokens moved by one debit-first primitive, asserted every round. Every law was handed for free the exact safety property escrow exists to construct. Fair to a lease's steady state; not to its edges.
8. **The built-in liar cannot test any of these laws** — it rewrites only the request payload, which no law's allocation reads. Every "robust under the liar" statement in every write-up is vacuous; only the three agents who rebuilt the attacker law-side produced meaningful security numbers.

---

## 7. What the engine must gain, at `e032ee05`

All verified first-hand at the pinned commit by this pass.

1. **A transfer record type — and `DeliveryToken` cannot be it.** `DeliveryToken { not_before, expires_at, rate_limit }` in `crates/benten-drop/src/layer_c.rs` derives **`Clone + Debug` only** (confirmed against the frozen baseline `docs/public-api/benten-drop.txt`): no `Serialize`, **no issuer, no signature, no id, no quantity**. `admit_sealed_sender` takes the already-used count as a **caller-supplied `u32`**, and a workspace grep finds no store for it anywhere. `rate_limit` counts *sends*, and the token binds one body. The `{nbf, exp}` **shape** is exactly right and the *enforcement posture* is exactly right — the enforcer counts at the point of use and the right expires by construction, with nobody reclaiming anything. **The shape is the pattern to copy; the format has to be invented.** Needs: issuer (who re-credits), quantity, unique id, `Serialize`.
2. **A nonce on every transfer row.** `put_node_with_context` in `crates/benten-graph/src/redb_backend.rs` already folds the existence probe and the conditional write into one redb transaction, so a content-addressed insert-only transfer is **idempotent by CID for free** — a re-delivered transfer lands once, which closes the duplication failure the harness never modelled. The same fact is a hazard: the CID hashes labels and properties only (baked-in #5), so two legitimately identical transfers would dedupe into one and **destroy** units. The uniqueness disambiguator D-107 §4 already mandates covers this.
3. **The in-tx count method D-107 names, plus the read pre-check** (§6.6). `crates/benten-graph/src/transaction.rs` exposes puts and deletes only — the single structural addition is confirmed.
4. **A delivery/answer observation seam.** Nothing in the tree observes peer reachability or cooperation. `UptimePolicy` in `crates/benten-id/src/device_attestation.rs` has **zero decision branches in production code** — the enum, its `Default`, one struct field, doc comments, test fixtures, and one napi string→enum parser — and `crates/benten-caps/src/chain_authority.rs` states its own non-enforcement verbatim: *"…`online_uptime`… is NOT enforced here."*
   **This is a false-record finding against D-107 itself, not against this design:** `docs/future/bounded-resources.md` §1 selects the bound owner as *"the declared `UptimePolicy::AlwaysOn` peer"* — availability **declared**, by a field nothing reads, inside a record whose own subject is enforcement. Fixable at doc level; it belongs in the pre-tag false-record family.
5. **Zone-per-bound.** Already a load-bearing REQUIREMENT in D-107 §3 for the merge-recheck's zone granularity. It independently bounds the §6.5 telemetry audience, which is the second reason to keep it.
6. **No counter CRDT, and none should be added.** `crates/benten-sync/src/crdt.rs` is `StampedValue { value, hlc }` — **no author, no signature** — and `crates/benten-core/src/hlc.rs` states verbatim *"We do NOT reject stamps in the past."* Measured: syncing a balance through this LWW discards **61.9–63.1%** of commitments on perfect clocks and **76.7%** at +10 ms skew. The law never needs it: holdings stay local and move by signed message. An unauthenticated, unsigned, past-accepting LWW merge is also the **worst-exposed and entirely unmeasured** attack surface for any law that relays second-hand telemetry (§9).
7. **A guard band.** `{nbf, exp}` is wall-clock epoch seconds; the HLC rejects only future skew. Correct operation requires a `2Δ` interval during which the units are held by nobody — a cost appearing in **no** measured result anywhere in this study, because the simulator has one global clock.
8. **The transfer settlement rule is a design fork, not a detail.** The simulator reverts a lease by **arithmetic**: the lender reads the borrower's live balance, with no reachability test and no message. That is not implementable on two machines. The implementable unilateral rule (lender re-credits on its own clock) **creates units** — measured service above the conserved bound at 1.006–1.143, with **0.4× to 43.7× of C conjured** depending on law and scenario. The conserving rule (settle only when mutually reachable, costs a message) is **mostly better** — partition **+0.2148**, intermit +0.0749 — and worse only on `death` (−0.019 to −0.040). The whole story in one line: **unilateral reclaim is correct exactly when the borrower is gone forever and creates units exactly when it is merely unreachable, and no local rule can tell those apart.**

**Nothing here is wire-format-affecting for the frozen v1 surface**; every item is additive. **[reasoned]**

---

## 8. Open questions for Ben

1. **Scope — DeviceMesh or Atrium?** *My recommendation:* ship at **DeviceMesh** (`wire_cost_ceiling` = 5, enforced in `construct`), where all peers are one principal's own devices, the holder and the enforcer are the same principal, and a defecting borrower is only stealing from itself. At **Atrium** scope (up to 32 mutually-distrusting principals) local admission means the enforcer *is* the adversary, there is no accountable ledger, one non-lying peer switches the mechanism off (§6.4), and the telemetry is a plaintext beacon (§6.5). Atrium needs a signed, epoch-scoped allowance spent at the point of enforcement — the `DeliveryToken` posture — and a **decided** leak position, before any of this goes near it.
2. **Does the museum's demand fail over?** One question to them: *when a till is down, does the customer walk away, or go to another till?* It is an 80× swing in the value of the entire spawner mechanism (§4.2), and it is the only thing that would move the museum out of D-107's answer.
3. **Should ownership be elected locally instead of declared?** Measured: identical to pinned single-owner when healthy, **+38 points** when the owner dies, no coordinator, no quorum, no protocol — and D-107's declared selection currently rests on a field nothing reads (§7.4). This is a live alternative to D-107 §3's static capability-held ownership, and it is the one I would spend the next pass on.
4. **Settlement rule: guard band or handshake?** Accept a `2Δ` window in which the units belong to nobody, or require mutual reachability to settle (one extra message, and measured **better** everywhere except a permanently-dead holder)?
5. **Is non-stationary demand a real Benten workload?** It carries the only large adaptive win in the study (+0.2547). If Benten's bounds are stationary-with-known-weights, the honest answer is a demand-proportional split in the bound declaration and no mechanism at all.
6. **Should the concentration exponent be declared per bound, or should we build the local significance self-test (§3) that would make it emergent?** As it stands the law needs a policy decision it cannot make for itself, and that is a real cost against "emergent, not directed."

---

## 9. Refutation ledger

Claims that did not survive. **Do not re-import any of these.**

| # | Claim | Status | Why |
|---|---|---|---|
| R1 | "The damping constant interpolates between single-owner and escrow; both are extremes of one local rule." *(ORCH hypothesis)* | **REFUTED** | Four independent measurements. The mechanism space is a 2×2; escrow and single-owner are diagonal corners; the off-diagonal is 20× worse than either. Damping interpolates genesis ↔ exponent fixed point. §5 |
| R2 | "Maximum damping gives even-spread." | **REFUTED** | Maximum damping gives *the genesis*: one setting yields gini 0.000 or 0.875 from even vs owner start, both at zero movement. §5(c) |
| R3 | "gini 0 identifies escrow." | **REFUTED** | A perfect-knowledge even-splitter sits at gini 0.151/0.439. Gini 0 identifies a law that did not act. §5(a) |
| R4 | "The concentration end was confirmed to reach single-owner." *(one law reported "confirmed")* | **REFUTED** | It reached gini 0.875 at **svc 0.0338** vs 0.9811. It confirmed the allocation half and refuted the serving half. §5(b) |
| R5 | "A peer cannot lie about having answered, so answering is the availability signal." | **DEMOTED** | True and nearly useless: it reads a constant 1.000 for every attacker on a fully-online bound. Replaced by delivered/accepted. §4.1 |
| R6 | "Availability weighting earns nothing" *(all five laws)* | **INSTRUMENT ARTIFACT** | Offline nodes serve their own demand, so unavailability costs nothing. Under failover semantics the same term is +0.0667 vs +0.0008. §4.2 |
| R7 | "Single-owner is not an endpoint of any local rule; it is a serving policy no allocation constant reaches" *(four laws)* | **INSTRUMENT ARTIFACT** | A law could not express "forward though I could serve." With the hook, a local rule matches single-owner bit-for-bit. §6.7(2) |
| R8 | "Intermittent nodes settle on stable spawners for free." | **REFUTED** | At the tuned point flaky nodes drew from stable peers *below* chance (0.667/0.561 vs null 0.724). Requires an explicit sharp availability exponent. §4.4 |
| R9 | "Give spawners to high-demand nodes so their provision stays accessible" (park-your-surplus form) | **REFUTED as stated** | 6 pushes in 400 rounds with thresholds gutted: the intermittent-and-hungry never have surplus. Inverts to preferred-source/forward-target. §4.3 |
| R10 | Every lease-horizon result, including "genuine interior optimum at ≈128 rounds ≈ 16× mean holding time" | **WITHDRAWN** | (a) The simulator's reclaim reads the borrower's private balance with no reachability test — not implementable; the implementable version *creates units*. (b) The cited sweep's own mean is **monotone in horizon** with its argmax at the unbounded lease; the interior optimum exists on one scenario. §6.3, §7.8 |
| R11 | "The utilisation split is the only observed, unforgeable signal" *(two laws, used as an anti-inflation defence)* | **REFUTED** | Manufactured from state the lender cannot see, delivered even when the borrower is dead (54% of one law's reclaims), and measured **inert** (identical to 4 dp with it removed). Delete it; do not carry it as a security mechanism. |
| R12 | "Robust under the liar" *(three laws)* | **VACUOUS** | The built-in liar rewrites only request payloads; no law's allocation reads them. Real law-side attackers give −1.9 to −14.7 fleet points and 2–16× share gains. §6.7(8) |
| R13 | "The baselines are immune to delivery loss because they move nothing" | **FALSE for the privileged baseline** | It moves 3.32 units/round through a back-channel and loses **zero** at any loss rate. §6.7(3) |
| R14 | "A 10× demand lie buys ~2.9× the holding" / "availability-ranked probing at f=2 gives 2.8× the local-admit rate" / "reserve decay is worth ~6 points" *(one law's prior-model claims)* | **NOT REPLICATED** | Measured 0.86–1.56×; a round-robin control ties availability ranking; reserve decay flat within 0.004. Only "the reserve itself is load-bearing" replicated. |
| R15 | "No law ever exceeds the service bound" *(harness documentation)* | **FALSE above ρ≈1.3** | Observed to 1.034; the denominator assumes a FIFO discipline the laws need not obey. True at the default ρ=1.05, so all headline tables stand. §6.7(5) |
| R16 | "The `death` result shows concentration-near-demand is dangerous" | **CONFOUNDED** | 100% attributable to the owner being the node that dies; move the death and the same baseline is the best row in the table. §6.7(4) |
| R17 | "Beats STATIC on all 8" *(all five laws, as the headline)* | **TRUE BUT MISLEADING** | 71–88% of the gap closes with a demand-proportional static split. Re-baseline or the number reads as a 25-point win when it is 3–7. §0(1) |
| R18 | "Forwarding is nearly free" | **PARTLY INSTRUMENT** | A wrong-guess forward costs nothing here; up to 90.3% of attempts unbilled. The axis this record recommends is the one whose cost the instrument does not model. §6.7(6) |
| R19 | "The harness `museum` corroborates the museum decision." | **WRONG REGIME** | It is C=12, n=3, single-seat parties; the real museum's p99 party is 9, so D-107's own rule reads `12 > 81` = FALSE. And its 1.000 is a zero-latency-forward artifact: priced at one round of latency the same baseline scores **0.383**, last place. The *conclusion* (route the customer) survives; the corroboration does not. |
| R20 | "`sim.py` is identical to session start" / four cited artifact files | **UNVERIFIABLE / STALE** | Three artifacts have no regenerating command; one stored output predates a reproducibility fix and disagrees with its own report. All numbers re-derive from the public API; the files should be deleted or stamped. |

---

## 10. Critic triage — every finding dispositioned

Per rule 1, no finding is silently dropped. **A** = accepted into this record · **R** = rejected with reason · **O** = open.

**Harness audit (does the instrument beg the question?)**

| | finding | disp. | where / why |
|---|---|---|---|
| H-1 | Offline nodes serve their own demand → cost of unavailability is zero | **A** | §4.2, §6.7(1), R6. Decides the spawner question; re-run and confirmed by this pass |
| H-2 | Single-owner endpoint inexpressible; local rule matches it with the hook | **A** | §0(2), §5(b), R7. The study's best positive finding |
| H-3 | Privileged baseline moves units via a back-channel, cost-exempt | **A** | §6.7(3), R13. It is a routing upper bound, not an allocation one |
| H-4 | `death` conflates hottest / owner / dying | **A** | §6.7(4), R16 |
| H-5 | Service denominator not a bound above ρ≈1.3 | **A** | §6.7(5), R15. Headline tables unaffected |
| H-6 | At ρ=1.05 only 4.5–8.9% of C is free — ablations measured at minimum leverage | **A** | §5. Reframes every "mechanism X is inert" result as regime-conditional |
| H-7 | Round order prevents a fetch serving a same-round arrival | **A (bounded)** | Moves two laws by ≤0.039; the museum conclusion survives |
| H-8 | Built-in liar cannot express the attack | **A** | §6.7(8), R12 |
| H-9 | Broadcast billed only to reachable peers (18–23% rebate under outage) | **A** | Flatters "the broadcast is also the liveness sensor"; direction is against this design, magnitude small |
| H-10 | Shared rng desynchronises loss runs across laws | **A (minor)** | Loss runs are unpaired; loss=0 is paired. Bounds §6.3's precision, not its direction |

**Reproducibility audit**

| | finding | disp. | where / why |
|---|---|---|---|
| F1 | "Bit-identical" ablation is false at full precision; the ablation subclass is confounded | **A** | Conclusion survives (+0.0013, inside sd); the word does not. Recorded so it is not re-cited as proof of inertness |
| F2 | Lease interior-optimum falsified by its own monotone mean | **A** | R10(b) |
| F3 | "Worse than doing nothing at 5% loss" contradicts its own table | **A** | §6.3 restated by loss level; the design conclusion is unaffected |
| F4 | Failover conclusion doesn't generalise: identical on `death`/`partition`, margin is 8.2 not 13 points | **A** | §4.2 states `intermit` only; the failover reframing changes nothing on 7 of 8 scenarios |
| F5 | Four cited artifacts unregenerable or stale | **A** | R20 |
| F6 | Two false claims in the harness's own documentation | **A** | R15 + one pre-fix artifact |
| F7 | Cost facts nobody surfaced (write-txn contention is scenario-conditional; absolute message rates invert the ranking on some bounds) | **A** | §6.1's cost line; noted that three of five laws are *dearer* on the wire than single-owner |
| F8 | The greedy **ask** is the attack nobody ran | **O** | §2 bounds it at the lender **[reasoned]**; must be built before any "not attackable" claim ships |

**Locality audit (does any law use state a peer could not observe?)**

| | finding | disp. | where / why |
|---|---|---|---|
| F-5 | **No law smuggles global state** — full object-graph audit, all three channels clean | **A (negative result)** | The laws are honestly local. The leaks are all instrument affordances |
| F-1 | Lease expiry reads the borrower's private balance; "phantom" holdings to 388% of C; no admission-veto hook so no law can implement the borrower-side discipline | **A** | §7.8, R10, R11 |
| F-2 | Wrong-guess forward is free | **A** | §6.7(6), R18 |
| F-3 | Broadcast billed only to reachable peers | **A** | = H-9 |
| F-4 | Denominator not a bound | **A** | = H-5 |
| F-6 | Built-in liar cannot test any law | **A** | = H-8 |
| F-7 | A law learns "am I online" perfectly, instantly, free | **A (declare)** | A real node roughly knows its radio state; here it is error-free and not partition-sensitive. Declared limitation |

**Byzantine audit**

| | finding | disp. | where / why |
|---|---|---|---|
| B-1 | The SINK zeroes two laws | **A** | §6.4 — gates deployment scope. Re-run and confirmed by this pass |
| B-2 | Service is not a security metric; report attacker-share-of-C | **A** | §6.4 |
| B-3 | Availability estimator is a constant | **A** | §4.1, R5 |
| B-4 | No shipped defence defends; fan-out works, with a density limit | **A** | §1/R4 |
| B-5 | Reclamation is a harness privilege; the THIEF leaves lenders asserting 131× the bound | **A** | §7.8, R10 |
| B-6 | Telemetry is a fleet-wide beacon with no opt-out | **A** | §6.5, §8.1 |
| B-7 | The trust boundary was never named — DeviceMesh vs Atrium | **A** | §8.1. The single most consequential scoping decision in the record |

**Hosting audit (can the engine host this at `e032ee05`?)**

| | finding | disp. | where / why |
|---|---|---|---|
| Hs-1 | Implementable lease reversion creates units | **A** | §7.8, R10 |
| Hs-2 | `DeliveryToken` cannot host a lease | **A** | §7.1 — withdraws "examine it before inventing a lease format"; the format must be invented |
| Hs-3 | Refusals cost a full write txn; read pre-check is 26× cheaper at identical soundness; O(k) scan holds to ~400, breaks at 10⁴ | **A** | §6.6, §7.3 — a real sharpening owed **to D-107** |
| Hs-4 | D-107 selects its owner by a field nothing reads | **A** | §7.4 — false-record family, pre-tag |
| Hs-5 | `put_node_with_context` gives idempotent-by-CID delivery free; needs a nonce or identical transfers dedupe and destroy units | **A** | §7.2 |
| Hs-6 | LWW is not this design's conservation problem | **A** | §7.6 — the measured 62% discard is the cost of the mistake of syncing a balance, which this law does not make |

**Applicable-region audit**

| | finding | disp. | where / why |
|---|---|---|---|
| Rg-1 | The floor is mis-specified; 71–88% of the gap is configuration | **A** | §0(1), §6.1, R17. Re-run and confirmed by this pass |
| Rg-2 | The region is intermittency + non-stationarity, not skew; empty at perfect uptime, negative at skew ≥ 8 | **A** | §1 regime selector |
| Rg-3 | The split constant spans both endpoints exactly; one law's "confirmed" is wrong | **A** | §5, R4 |
| Rg-4 | Priced write-txn budget: single-owner's edge dies between W=16 and W=12 — but on messages three of five laws are strictly worse | **A** | §6.1 cost line; the honest trade is write-serialisation vs wire traffic and only one side was ever priced |
| Rg-5 | The harness museum is the wrong regime; its 1.000 is a zero-latency-forward artifact | **A** | R19 |
| Rg-6 | Its own primary attack refuted — pricing the forward does *not* sink single-owner outside the museum | **A** | Recorded: single-owner's no-failure wins survive a priced forward |
| Rg-7 | Denominator + saturation at 1.0000 on two of eight scenarios | **A** | = H-5; two scenarios have zero headroom above single-owner by construction |
| Rg-8 | The instrument cannot measure the property the decision is about — conservation is given free | **A** | §6.7(7). The deepest finding: fair to a lease's steady state, not to its edges |
| Rg-9 | Tuning honesty uneven; one shipped default is measured-suboptimal | **A** | §1 regime selector — and the reason the record recommends declaring the exponent, not tuning it |

**Continuum audit**

| | finding | disp. | where / why |
|---|---|---|---|
| C-1 | The concentrated end is not single-owner; it is the worst configuration measured | **A** | §5(b), R4 |
| C-2 | Maximum damping gives the genesis | **A** | §5(c), R2 |
| C-3 | The corner is unreachable under contention, and not because of safety rails | **A** | §5 |
| C-4 | Forwarding +23.3 vs concentration −22.2 | **A** | §1/R2 |
| C-5 | "Per-bound, emergently" is unsupported by any measurement | **A** | §1 regime selector, §8.6 |
| C-6 | gini 0 is not escrow | **A** | §5(a), R3 |
| C-7 | Repro caveats: one escrow-exactness claim needs a second mechanism off; one endpoint number is on a different statistic (0.016 vs 0.155 under the harness's own definition); a large *finite* exponent crashes three laws with an overflow | **A** | §5(a) carries the qualifier; the statistic mismatch is why §5 quotes per-seed identity rather than any Gini figure; the overflow is a config-value clamp — a constant that can kill every node in the fleet |

---

## 11. What this closes, and what it does not

**Closes:** the self-balancing law's shape, its telemetry, its estimator, its damping parameter, and — negatively but firmly — the endpoint continuum. It gives D-107 two gifts: a measured sharpening of its admission check (§6.6) and a false-record finding against its own owner-selection sentence (§7.4). It sharpens D-107's recorded escrow-revisit trigger from *"revisit iff `C > 3·n·q_p99` and demand genuinely multi-homed"* to a two-part test whose second half is now empirical: **and demand actually fails over when a node is unreachable** (§4.2) — because if it does not, a share at a dark node is not stranded and there is nothing for balancing to recover.

**Does not close:** whether the mechanism is safe outside one principal's device mesh (§8.1); whether the museum's demand fails over (§8.2); whether a locally-elected owner should replace D-107's declared one (§8.3); the settlement rule (§8.4); and every one of the following, which nobody in this study measured and which should not be inferred from anything above — **wall-clock latency, message latency, clock skew and therefore every `{nbf, exp}` guard band, redb write *latency* under contention (counted, never delayed, in the simulation), duplicate delivery, correlated failure, collusion, equivocation, forged relayed telemetry over the unsigned past-accepting LWW merge, the greedy-ask attack, any `n > 32`, and any workload family other than Poisson arrivals with exponential holds.**