# The "at most N" spectrum — every regime, not just the museum's

**Status: R0-INPUT.** A design-space map, not a plan. Sits **above**
`docs/future/bounded-resources.md`, which stays the design of record for the museum regime and is
cited here as one region. Nothing below re-decides that answer. Written 2026-08-11 out of
`docs/future/engine-fit-and-gaps.md` §3.4, widened by Ben to cover the whole taxonomy.

**Provenance.** Six lenses, each adversarially verified at `ceb027ba`; 13 agents, 0 errors. The
pass carried its own **refutation ledger** — 21 claims that did not survive, including several of
its own lenses' headlines. **§8 records where I re-verified the pass itself and it was wrong**
(§3.5n). Claims I checked first-hand are marked `[ORCH-verified]`.

---

## 1. The design target

**One system for the museum's tills AND every other kind of decentralized balancing.** Not a
till-shaped mechanism retrofitted onto other cases. The taxonomy the map is scored against:

| band | members |
|---|---|
| **Physical / spatial** | timed-entry admission capacity · venue + seat capacity · reservation and appointment slots · camp and class enrolment caps |
| **Inventory** | finite stock, incl. **decentralized inventory across locations** · limited-edition minting (N ever) · one-time-use coupons and tokens (N = 1) |
| **Financial** | grant pools · budget pools · spend ≤ allocation |
| **Entitlement** | per-patron discount usage caps · license seats · concurrent sessions |
| **Computational** | rate limits (N per window) · quotas (storage, API calls) · connection-pool and concurrency caps |

---

## 2. The direct answer: can a node ask peers for spare rights before refusing?

**Yes — and the strongest result here is that asking and refusing are the same mechanism, not
rival designs.** Write the commit-time guard as:

```
count(rows attributed to me) < allowance(me)     ← generalizes
NOT  count(all rows) < C                          ← forks one mechanism into two
```

Under exclusivity these are the same number. Only the first has a knob:

| initial deal | what you get |
|---|---|
| **100% of C to one peer** | **single-owner** — every other peer's allowance is 0 and refuses locally |
| **C/n evenly** | **classic escrow / bounded counter** |
| **proportional to predicted demand** | the museum's measured oracle — 0.0% false rejections |

Balegas's own soundness proof *is* single-writer-per-cell, so escrow is not the opposite of
single-owner — it is the same family at a **finer grain of exclusive ownership**, with ownership
moving in the data plane instead of the control plane. A freshly-created bounded counter is
single-owner by construction.

**⇒ The engine needs ONE mechanism with an initial-deal policy and a borrow policy. Not a menu.**

Worth knowing: **Antidote's `bcounter` borrow is asynchronous** — it returns `no_permissions`
immediately and queues a request on a 100 ms timer. Ben's synchronous ask-then-decide is a genuine
improvement on the reference implementation, not a reinvention of it.

### The load-bearing caveat

**The continuum holds for the CHECK. It does not hold for the ENFORCEMENT.** At 100%-to-one-peer,
exclusivity is sound because one engine writes and redb serializes there. Below 100% there are *n*
writers, and three substrate facts break the safety proof:

1. The only cross-peer authority check is **zone-granular** (`engine.rs:1371, 1402-1408`) — coarser
   than the per-cell property the proof needs.
2. **Per-row authorship is unauthenticated.** `StampedValue` is `{value: String, hlc: HlcWire}` —
   no author field, no signature `[ORCH-verified]`. And `AtriumHandle::register_peer_did` has
   **zero production callers** — every call site is under `tests/`; the only non-test hits are its
   own definition and a doc comment `[ORCH-verified]`.
3. **The HLC skew check is one-directional** — `hlc.rs` verbatim: *"We do NOT reject stamps in the
   past — that's the normal case for a peer whose message was queued briefly"* `[ORCH-verified]`.
   Under LWW a forged **low** consumed-count with a **high** HLC wins.

So the *ask* is buildable. A *safe multi-writer allowance* is not: every point on the continuum
except the 100% endpoint is blocked on **cryptographic per-row authorship** — a genuinely absent
primitive, not a missing binding.

**Scope this honestly:** there is no counter CRDT and no escrow today, so this is a constraint on
the *design*, not a live exploit. What IS live is item 2's second half — peer-DID attribution falls
back to synthetic `node-id:NNN` strings in production while its own rustdoc reads as though the
handshake wires it. That belongs in the pre-tag list (§7).

### Even where it is safe, it usually loses

Measured, like-for-like (asks/booking): C≤16 camps **0.881 → 0.152** (5.8×) at n=3, **1.639** at
n=8; C 17–100 **0.197 → 0.038**; C>100 timed **0.155 → 0.020**. Single-owner is **flat in n** —
the owner either is or is not the seller. Escrow degrades as replicas multiply.

### Where asking IS right, and where it loses

**Right:** large, **divisible**, **regenerating**, genuinely multi-homed bounds — grant and budget
pools, API and storage quotas, license seats, concurrent sessions. Divisibility removes the
`q > C/n` floor entirely; regeneration dissolves escrow's worst defect, because **the epoch
boundary is the reclaim protocol nobody built.** Also: meshes with no natural always-on peer, and
adversarial partition where a designated owner is a designated target.

**Loses:** small C (nothing to lease) · N=1 (definitionally single-owner) · **multi-bound atomic
composition** (35.1% of museum enrolments touch >1 bound; P(needs ≥1 borrow) is 97.2% at n=8, and
synchronous borrow converts fail-fast into **hold-and-wait**, the classic deadlock precondition —
single-owner settles all bounds in ONE redb write transaction) · **physical stock** (reject locally
and route the customer to the peer holding the unit) · **mutually-distrusting participants** — the
counterintuitive result: **escrow is Byzantine-detect-eventually; single-owner is
Byzantine-preventable.** Leaderless is *worse* under distrust.

---

## 3. The frame: five currencies

Bailis et al. (PVLDB 8(3), 2014) — invariant confluence is necessary and sufficient for safe
coordination-free execution, and this class is **not** I-confluent (counter-decrement is their
proof 13; size-bounded set mutation proof 17). **No family removes coordination. Each relocates
it:** ① round trips · ② pre-allocation (false rejections, stranding, rebalance traffic) ③ clock
and synchrony assumptions · ④ retractions and repair · ⑤ the guarantee itself.

A family claiming none of the five is hiding one. The map is *where do you want to pay*.

---

## 4. Ben's two axes, and the eight others

**A6 — cost to move the allowance.** Free (capacity, quota, budget) vs costly-and-failable
(physical stock). Plus the asymmetry that dissolves half the pessimism about leases: **a capacity
lease can be RETURNED unilaterally** — decrementing your own allowance needs no agreement and is
the one genuinely coordination-free operation in the space. Stock cannot be returned that way, and
any borrow family must say what happens when a transfer is agreed and then fails.

**A5 — regeneration.** Never refills / event-driven / scheduled-windowed / release-driven.
**Regeneration IS the reclaim protocol nobody built** — a window reset re-deals from scratch and a
dead peer's stranded rights have a bounded lifetime of one window. Two consequences: a lease MUST
be epoch-scoped with the epoch part of its identity, or a lease from epoch N spends against
epoch N+1's pool; and windowed bounds need time agreement, which Benten deliberately fails closed
on rather than defaulting to `now()`. **Release-driven refill is the one shape that is
coordination-free in the safe direction** — raising a bound is monotone.

The rest, ordered (first three *eliminate*, rest *select*): **A0 utilisation** — 0.0% of the
museum's 17–100 capacity bands ever reach 90%, so two of five bands need **no mechanism at all**
· **A1 guarantee strength**, whose sub-axis is that *a never-silently guarantee with no detector is
advisory with better branding* · **A2 counting vs identified goods** — when the N things are
distinguishable (seat 14A, serial #7), "at most N" decomposes into N independent "at most 1 on a
unique key", an existence test rather than arithmetic · **A3 reversibility × consequence**, giving
four classes (R0 costless: rate limits, quotas · R1 pre-priced: admission, seats, enrolment,
license seats · R2 clawback: inventory, mints, coupons · R3 irreversible: disbursed funds, a dose)
· **A4 divisibility** — the `q > C/n` floor is an indivisibility artifact, not a size law
· **A7 trust THEN liveness**, separated in that order · **A8 h**, the locally-servable fraction
· **A9 λ** against the serial point · **A10 multi-bound composition**, which constrains the
meta-answer: bounds that compose atomically must share a mechanism and preferably a home.

---

## 5. The leaderless regimes specifically

| you need | family | Benten today |
|---|---|---|
| large, divisible, multi-homed, regenerating | escrow + borrow | **BLOCKED** — serial point ✓, data shape ✓, discovery free, but single-writer-per-cell unenforceable |
| **identified goods** (seats, slots, serials) | **key-space / demarcation** | **EXPRESSIBLE TODAY** — `store:seat:*` attenuates to disjoint `block-A`/`block-B`. **The one leaderless family Benten can serve at v1**, and it meets the elegance bar: generalize a shipped fragment |
| reversible + must stay write-available | compensation / IPA | **BLOCKED** — merge veto is whole-merge not per-row; graph↔zone hydration absent |
| transferable / offline-redeemable | token / bearer | **PARTIAL** — Compromise #64 already prices the cross-device residual honestly |
| trustless participants | quorum / threshold | **BLOCKED** — no threshold crypto (baked-in #5 makes this an upstream-adoption decision) |
| regenerating computational bounds | probabilistic / soft | **HAVE weakly** — per-process; shipped default is *no bound* |

**A constraint that disqualifies whole regimes before mechanism comparison starts:**
`benten-sync` is compile-time-rejected on wasm32, so **only full peers can be CRDT replicas.** Any
deployment whose edge devices are browsers or webviews is single-owner by construction.

---

## 6. What v1 builds, and the one thing that is now-or-never

**v1 = single-owner**, on grounds independent of the museum's numbers: **migration asymmetry**
(owner → escrow is additive — the owner deals its bound out, literally Balegas's creation state;
escrow → owner requires reclaiming stranded rights, which no surveyed implementation does) ·
it is **the degenerate case of every other family** · multi-bound composition is free only at one
owner · Byzantine-preventable.

**Split GUARANTEE from MECHANISM.** The guarantee (`hard | never-silently | advisory`) lives in
application content, content-addressed and portable. The mechanism
(`owner | key-space | escrow | compensating | quorum | local-soft`) lives in deployment policy.
The same application runs single-owner at the museum and key-space escrow at a multi-site venue
**with no application rewrite** — mirroring `Strategy` exactly.

**The reserve — one genuine candidate, and it is not the one I expected:**

> **A per-property `mergeStrategy` slot.** Property merge is locked to LWW at the sync layer.
> Escrow needs a non-LWW rights ledger; compensation needs a converging count. If the freeze bakes
> *"properties are LWW, period"* with no annotation slot, **both leaderless families are foreclosed
> at the sync layer regardless of what the bound declaration says.** Single-owner is the one family
> that never touches the CRDT property layer — which is exactly why the museum-regime record does
> not carry this. **Reserve the SLOT, never a strategy.**

Three others assessed and **withdrawn**: the refusal payload (`ErrorCode` and `Outcome` are both
`non_exhaustive`; keep the observation that `available` means three different things across
families as a §4.172 design note) · per-row authenticated authorship (additive as a mechanism) ·
a threshold codepoint (already reserved twice and already scheduled).

**Free at v1, carry forward:** write the guard as `count(rows attributed to me) < allowance(me)`
· make the allowance a **node CID indirection**, not an inline integer · state zone-coupling in its
**writer-partition** form and price it honestly (`n` zones × `n` anchors × fork-refusing chains, not
"free") · **reserve the epoch/fencing integer** — epoch, not time, is the reclaim primitive, needing
no clock · do **not** reuse `fork_a_wins` for succession (oldest-anchor-wins is backwards for
ownership transfer) · no ALPN reservation needed. **The invariant to state:** *placement may be
computed; authority must be granted.*

---

## 7. Honest limits, and what this adds to the pre-tag list

**Tokens:** offline double-spend is the UTXO problem — unsolved without a global ledger or trusted
hardware. **Escrow:** no surveyed implementation reclaims; safely revoking from a partitioned-but-
alive peer needs agreement on who is dead, which is consensus by the back door. **Compensation:**
the merge veto is **whole-merge, not per-row**, so a returning stale-lease till does not get its
over-limit rows voided — *its whole zone stops merging, legitimate sales included*; and the ranking
clock is caller-supplied with a one-directional skew check, so the incentive **inverts** between
LWW and loss-allocation. **Single-owner:** exclusivity rests on non-syncing local grants plus
operational discipline, not on engine-enforced principal ownership (§8).

**Empirical base is n=1.** Every *quantitative* claim traces to one children's museum with 82%
single-channel demand on a LAN, and **till↔till partition frequency remains unmeasured** — the
highest-value unknown, and the only input that would move borrow-on-demand from "narrow band" to
"necessary." The *qualitative* structure does not depend on it.

**Also: overbooking-as-policy is the incumbent answer in several bands.** Airlines and hotels
deliberately sell > C to a measured, priced violation rate. Where that is acceptable, every
mechanism in this map is over-engineering.

**Pre-tag items this pass adds** (small, disclosure-shaped, fold into W-REC):

1. **`register_peer_did` has zero production callers** while its rustdoc states the handshake wires
   it in the present tense `[ORCH-verified]`. Either wire it, or retense to match — per rule 15,
   price the code fix first. Sync attribution is synthetic (`node-id:NNN`) in production today.
2. **The `mergeStrategy` slot decision** — the one genuine now-or-never above.

---

## 8. Where I re-verified the pass and it was WRONG

The survey's refutation ledger is mostly excellent, and two of its entries are themselves false.
Recorded so nobody re-imports them:

| ledger claim | ORCH verdict |
|---|---|
| *"redb's blocking single-writer lock" — **REFUTED**, it is a fail-fast typed error `[verified-here]`* | **THE REFUTATION IS WRONG.** redb 4.1.0's own doc on `Database::begin_write`, verbatim: *"Only a single write may be in progress at a time. If a write is in progress, this function will block until it completes."* `bounded-resources.md`'s blocking-lock claim stands `[ORCH-verified]` |
| *"`check_write` discards the actor `[verified-here]`"* | **TRUE ONLY FOR ONE PATH, false as stated.** `engine.rs:1407` sets `ctx.actor_cid = peer_actor_cid` on the sync-merge per-row recheck, and `engine_wait.rs:893` populates it on WAIT resumption. But `primitive_host.rs:623-625` — **the WRITE-primitive path an admission check would sit on** — builds `CapWriteContext::default()`, sets `label` and `device_cid`, and never sets `actor_cid` `[ORCH-verified]`. **The finding survives on the path that matters; its generality does not.** |

Both were marked `[verified-here]`, i.e. an agent claimed to have checked them personally. That is
the §3.5n case exactly: **a claim's confidence label is not evidence.** The genuinely useful
survivor of the second row is that on the admission path the policy sees a *scope*, not a
*principal* — so single-owner exclusivity is not engine-enforced ownership, and
`bounded-resources.md` should say so rather than imply otherwise.

Three sharper claims I confirmed at source and carry forward: `StampedValue` has no author or
signature field · `register_peer_did` has no production caller · the HLC rejects only future skew.
