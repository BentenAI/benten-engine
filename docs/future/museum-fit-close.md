# Museum fit — the volume answer and the invariants answer

**Closes** `docs/future/engine-fit-and-gaps.md` §3.8 (*"the question nobody asked — does the engine
handle their VOLUME?"*) and the museum's own open item, `versai/design/benten-fit.md`
"What to check next" #1 (*"whether IVM views can express the 129 SQL invariants, or whether a
two-store answer is needed"*).

**Basis.** Two investigations plus an adversarial verification pass, all at `27b2568b`: a measured
volume harness driving the real `RedbBackend` write path at full corpus scale on an M1/8 GB laptop,
a rule-by-rule expressibility analysis over the museum's `design/spec/rules.json`, and a 26-probe
re-verification. Then an ORCH ground-truth pass over every finding that changes an answer (§3.5n).
The ORCH pass found three things the first two passes did not, and one of them is a FALSE-RECORD on
the frozen public API.

Marking throughout: **VERIFIED** = read at `27b2568b` · **MEASURED** = a number someone ran ·
**REASONED** = inference, never dressed as observation.

---

## The two answers

**1 — Volume: yes, and it is not close.** Their entire 94-month corpus — 1,735,000 nodes — lives in
**2.28 GB**, opens in **16 ms**, sits in **75–85 MB** of RAM, answers a by-CID read in **0.29 ms**,
and folds end-to-end in **30 seconds**, on an 8 GB laptop; a till writing a sale line costs **~6 ms**.
Nothing about their scale should influence this decision.

**2 — Invariants: they can run all 159 checks today, but they cannot yet *enforce* them, and the
gate is one engine feature.** Verification — the suite as they have it, detecting violations — works
now over the public read surface. Enforcement — a rule refusing the bad write before it exists, which
is strictly better than what SQL gives them — needs **binding grammar** (a handler being able to see
its own input), which gates roughly **119 of the 159**. **They do not need a second store in the
product.** They should keep Postgres through the cutover as a differential oracle with a defined
end date, and that end date is at migration parity, not later.

**The headline is not "they need a second store."** It is: *build on it; write your checks in host
code on day one; convert them to write-path refusals as binding lands.*

---

## What changed during verification — read this before quoting either report

Three corrections, each of which moves a claim from one column to another.

**(a) The enforcement claim was overstated, and the verifier was right to refute it.** Both the
"available today" framing in the invariants pass and any reply built on it are wrong: **a handler
cannot see its input at HEAD.** ORCH-verified independently at four sites — `benten-engine`'s
`engine.rs` builds `input_value = Value::Map(input.properties.clone())` and hands it to
`benten_eval::Evaluator::run`, whose `run_inner` takes it as **`_input: Value`** and never binds it;
`primitives/call.rs` passes `Node::empty()` to every callee; `primitives/branch.rs` exposes
`execute(op)` — no host, no context, reading only the operation node's own static properties;
and `EvalContext`, the `$input` container, is reached in production only by the WAIT path
(`benten_eval::evaluate`, documented as the test/wait entry), not by the general walk. The project's
own `docs/future/binding-grammar.md` says it outright, and `phase-4-backlog.md` §4.174 — written
today — already names the same site. So this is not a new gap; it is the **first quantification of
that gap against a real application's rule set.**

**(b) The one query that makes the volume story sing is unreachable by any application.** MEASURED:
`get_by_property` answers an indexed lookup over the full corpus in **0.43 ms** (144,428 hits in
542 ms; both cardinalities reproduced exactly by the verifier). VERIFIED: it exists on
`RedbBackend`, it exists on the `PrimitiveHost` trait, `benten-engine`'s `primitive_host.rs`
implements it — and **no primitive executor calls it and `Engine` exposes no forwarder**.
`primitives/read.rs` supports exactly three shapes: `target_cid`, `query_kind` + `label`, and
`query_kind: "empty"`. So the fastest number in the entire measurement set describes a surface
nobody can call. Same for `read_view`, `put_edge` and `delete_edge` on the host seam: implemented,
zero primitive callers. This is the same shape as (a) — **capability present one layer down, no
caller at the layer that needs it** — and it recurs enough to be worth naming as a pattern:
`benten-graph` is richer than `benten-eval`'s host trait, which is richer than `Engine`'s public
surface, and each step up silently drops capability.

**(c) A FALSE-RECORD on the frozen public API — see §5. It is cheap, and it ships with the tag.**

Also corrected, in the engine's favour or against, from the verification pass: RSS is **75–85 MB**,
not 130–193; the till path is **~6 ms/line**, not 4.627 (drop the three decimals — it is one
fsync-bound run); the fold-vs-random-read gap is **15×**, not 17×.

---

## 1. Volume — the evidence

MEASURED on 1,735,000 nodes shaped like a `sales_allocation` line (one label, 8 properties, money
as `Int` minor units), written through `RedbBackend::transaction` → `Transaction::put_node` — the
real path, with the in-transaction Inv-13 probe and every index insert. Durability is a genuine
fsync per commit (`DurabilityMode::Group` VERIFIED to collapse to redb `Durability::Immediate`;
grouped commit is not implemented and the backend says so at runtime).

| | |
|---|---|
| file on disk | **2.279 GB** (1,314 B/node) |
| clean open | **16 ms** (verifier: 22 ms) |
| RSS, full-corpus scan | **85 MB** |
| by-CID read, random | **0.29 ms** — 3,514/s |
| full fold, all 1.735M | **29.5–29.7 s** — ~58,000 nodes/s |
| single-node write (till line) | **~6 ms** — ~170/s |
| bulk ingest, 8 indexed props | ~4,200 s for the full corpus |
| bulk ingest, 1 indexed prop | **326 s** |

**Three things worth knowing, none of them disqualifying.**

**Ingest is index-bound, and the index is not optional.** VERIFIED: `put_node` loops every label ×
every property into `PROP_INDEX_TABLE`, unconditionally, with no opt-out. MEASURED: 8 properties vs
1 is **~13× ingest time and 2.11× disk**. Their largest single entity (~440k rows) loads in about
six minutes; the whole corpus in about seventy. One-time cost, budgetable, and reducible today by
bundling rarely-filtered fields into one `Value::Map` property.

**Read in index order, not application order.** MEASURED: the same corpus folds at 58,000/s
sequentially and 3,900/s randomly — **15×**. REASONED cause, consistent with the code: the node
table is keyed `"n:" ++ cid` (VERIFIED) and the label index stores CIDs sorted, so walking the label
index walks the node table in key order. Report shape follows: select a slice, fold the slice. A
per-till revenue report is ~3 s; whole-corpus is 30 s.

**A dirty shutdown costs a five-minute start.** MEASURED by accident: after killing the writer
mid-transaction, the next open took **300 s** with RSS peaking near the 1 GiB redb page cache; the
clean open of the same file immediately after took 16 ms. For a museum whose non-negotiable is that
the door works at 9am, that is a number to design around and one we have not characterised —
whether it scales with corpus size or with the interrupted transaction is **unmeasured**.

**Also VERIFIED and worth one line:** redb's page cache defaults to 1 GiB and `set_cache_size`
is called nowhere in `crates/`, so the ingest knee arrives at roughly the same absolute corpus size
on a 128 GB server as on this laptop. Cheapest engine-side improvement in this document: one config
field.

---

## 2. Invariants — the evidence

Their suite is **159 rules** (VERIFIED in `spec/rules.json`; plus 14 `skipped`). The "129" in
`tech-stack.md` and `benten-fit.md` is an earlier count — real, just superseded.

**The reframe that decides it.** Every rule is a `violations_sql` — a query returning offending
rows. They are detectors, run as a suite, not constraints. Their own notes admit where this bites:
`grant_drawdowns_never_exceed_pool` records that *"the sole source write path validates only the
date window… the balance function exists and the insert does not call it."* Versai does not enforce
that rule; it can only find out afterwards. So "can Benten express them" is two questions:

- **Run them (verification)** → host application code over the Engine read surface. Works today.
- **Prevent them (enforcement)** → the write path. Strictly better than SQL, and gated on binding.

The suite *shrinks* under enforcement: a rule you prevent is a rule you stop detecting, and a whole
family disappears entirely (below).

**IVM views express none of them, before or after the aggregation build — and that is the direct
answer to their #1.** VERIFIED: a user view's entire selector vocabulary is `LabelPattern::Exact` or
`AnchorPrefix`, matched against the node's **first label only**; `GenericKernel::read` ignores its
query argument; `Projection::apply` is the identity; `ViewResult` is `Cids` / `Current` / `Rules`.
No predicate, no join, no aggregate, anywhere. Adding the abelian fold of `ivm-aggregation.md`
computes a value over the same unfiltered label set — it does not add a `WHERE`. **Views are a
reporting surface. They were never the mechanism, and that is not a defect being confessed — it is
the enforcement-vs-reporting split `bounded-resources.md` already states.**

**Two further VERIFIED facts about views the adopter must be told, because "materialized view" does
not mean here what it means everywhere else:** `register_view` is `guard.push(view)` — no scan, no
backfill, so a view registered over an existing corpus returns empty and stays empty; and the
kernel's `entries: BTreeSet<Cid>` is in-memory and per-process, so it is empty again after every
restart. Register-before-migrate is therefore *not* a workaround: it survives one process lifetime.

**Per-shape verdicts** (shape counts REASONED from a regex classification, single primary shape per
rule; **94 of 159 are multi-entity**, independently recomputed twice — exact).

| Shape | n | Home | Available |
|---|---:|---|---|
| Row-local predicate | 50 | write-path BRANCH — *prevented* | after binding |
| Referential / cross-entity | 69 | write-path READ → `ON_NOT_FOUND` | after binding |
| Conservation tie-out, bounded group | ~18 | handler `sum()` over the group | after binding |
| Bound over an unbounded set | ~6 | **admission — and it needs a SUM, not a count** | after admission build |
| Uniqueness, whole-content | (of 9) | content-addressing | **free, by construction** |
| Uniqueness, composite key | (of 9) | **admission with C = 1** | after admission build |
| Non-overlap (temporal) | 1 | **predicate admission** | after admission build |
| Monotonic ordering | 2 | write-path BRANCH | after binding |
| Capacity bound | 2 | **single-owner admission** | after admission build |
| Derived-field consistency | woven | **mostly deleted with the field** | — |
| **All 159, as detectors** | 159 | host code over the read surface | **today** |

Four of these deserve their sentence:

**Uniqueness splits, and half of it is free.** `import_never_double_credits_the_live_ledger` becomes
structurally unsatisfiable — a byte-identical node is one CID. The inversion is real and must be
held alongside it: the same mechanism silently swallows two economically distinct identical rows,
which is why `ivm-aggregation.md` §2 mandates a per-row nonce. **Once rows carry a nonce, dedup
stops protecting you** and the composite-key rules (`posting_leg_is_unique`, a 5-column key) come
back as real checks.

**Derived-field rules mostly vanish rather than migrate.** `standing_is_derivable_from_commitment_history`
asserts a stored field agrees with a derivation. Their own target model already says *"SupporterStanding
is computed, never stored."* You do not migrate that rule; you delete it with the field it guarded.
The derivation still runs — `MAX(expiry) over commitments for (patron, fund)` — as a per-entity
handler `max()` over a small set. Note this is the grouped-max the v1 fold **excludes** (a monoid
without an inverse), and the exclusion costs nothing here because the set is per-entity and small.

**Conservation tie-outs are the pleasant surprise.** `posting_line_axis_closes`,
`refund_line_allocation_split_nets_to_zero_per_line`, `pledge_schedule_sums_to_ask_or_carries_a_write_off` —
these sum a *bounded* group (the legs of one line, the schedule of one pledge) against a parent
value. Tens of rows. Read parent, read children by CID, `sum()`, BRANCH. **These are the double-entry
conservation laws, they are the rules people assume need the fold kernel, and they do not.**

**Three "impossible" families are one build.** Capacity bounds, composite-key uniqueness (C = 1),
and non-overlap (a predicate instead of a count) all need the same thing: a check inside the commit's
own serialization domain, at redb's single-writer lock, below all three write-entry families — which
is exactly `docs/future/bounded-resources.md`. **This is the most decision-useful sentence in the
whole analysis: the "not expressible" column is one engine feature, not three.**

**One honest defect in the arithmetic they would rely on.** VERIFIED: `sum()` accumulates with
`i64::wrapping_add` and, if any element is a `Float`, silently returns `Float`. The wrap is
theoretical here (1.76M rows × any plausible minor-unit amount ≈ 10¹⁴ against a 9.2×10¹⁸ ceiling).
The **silent Int→Float promotion is not theoretical**, and it is the silent-numeric-wrongness class.
The mitigation is a modelling rule, not an engine change: every monetary property is `Value::Int` in
minor units at one schema-declared scale, never `Value::Float`.

---

## 3. Two modelling decisions they must make before migrating anything

Both are forced by engine facts rather than taste, both are cheap now and expensive later, and
neither depends on any unbuilt work.

**1. Write every reference twice — as a CID-valued property *and* as an edge.** The two read paths
have opposite capabilities, and this is what falls out of correction (b):

- **Handler side (enforcement, post-binding):** a handler can READ a `target_cid` and route
  `ON_NOT_FOUND`; it cannot traverse an edge (no edge read on the host seam) and cannot write one
  (`put_edge`/`delete_edge` are VERIFIED zero-caller too — `write.rs` calls only `put_node` /
  `delete_node`). → the **property** is load-bearing.
- **Host side (verification, today):** `Engine::edges_from` / `edges_to` (+ `_as`) are public and
  fast; there is **no public property lookup at all**. → the **edge** is load-bearing.

Writing both costs one extra edge per reference and removes the choice entirely. This affects 43% of
their suite. Second-order: a resolved reference on Benten is *stronger* than an FK (the target is
immutable, so the cascade/orphan-on-update class disappears) and *weaker* in one way that makes it
useful — nothing stops you writing a CID that was never stored, so the check is real rather than
vacuous. Several of their rules are currently marked WEAK for the mirror-image reason
(`occurrence_resolves_to_a_real_venue_and_offering`: *"FK… are both DECLARED in the source, so this
rule cannot fail on faithful data"*). On Benten it becomes load-bearing.

**2. Labels are their only index — choose the partition before the corpus lands.** VERIFIED: label
is the sole selector in views, in the handler READ path, and as the root of the property index key.
A label per venue / per fiscal period / per account turns a 6.2 s full-corpus walk into a scoped
one at zero engine cost. It is also the answer to the two temporal rules — put the resource in the
label (`booking:venue-42`) and the candidate set for an overlap check becomes small. Inelegant;
free; and effectively irreversible once 1.7M nodes carry the wrong labels.

---

## 4. Do they need a second store? No — and the oracle retires earlier than the verifier said

**Not in the product.** The 159 do not need Postgres to *run*; they need a query surface, and host
code over the read API is one. Two stores means two write paths, two truths, and a reconciliation
problem — adding the failure class they are leaving.

**Yes through the cutover, as a differential oracle, with an end date.** Their concern is right:
re-expression is where errors enter. The answer is not to avoid re-expressing but to make it
falsifiable — run the suite on Postgres and on Benten against the same migrated corpus and require
**byte-identical violating-row sets, rule by rule**. Any disagreement is a migration defect found at
the cheapest possible moment.

**The end date is migration parity, not binding-grammar landing** — I disagree with the verifier's
item 7 here, with reason. Once the host-code detectors reproduce Postgres row-for-row, the oracle
has discharged its job: it proved the re-expression. Keeping it alive until enforcement lands adds
nothing, because **a rule that converts from detection to prevention does not retire its detector —
the detector becomes a zero-row assertion.** That is our own falsification discipline applied to
their domain, it is stronger than a second store, and it costs one nightly run.

**One gap the zero-row assertion genuinely cannot see, and it is worth naming to them:** a
*false refusal* — enforcement that is too strict — leaves no row anywhere. Postgres would not catch
it either unless they dual-wrote, which is the failure mode we are avoiding. The right control is a
**typed rejection log with review**, which is the shape `bounded-resources.md` already specifies for
refusal (`{available, bound, owner_did}`), generalised.

**And the suite is smaller on the far side.** REASONED: ~31% row-local and much of the 43%
referential convert from detected to prevented; the derived-field family largely dissolves. Plan for
**~90–110 surviving checks**, most of them prevention.

---

## 5. What the engine must gain — with placement

`phase-4-backlog.md` §4.174 (Ben-prioritised **today**, 2026-08-12) already sequences the trio
BINDING → ADDRESSING → AGGREGATION, Composing-first, with two pre-tag reservations. **Nothing here
argues with that sequence.** What this pass adds is quantification against a real rule set, plus
four items not currently in any record.

### Pre-tag (Core) — disclosure only, no mechanism

**★ FALSE-RECORD on the frozen public API — `Engine::subscribe_with_handler`.** Its rustdoc says
*"`Named(handler_id)` routes change events through the named handler subgraph"* and *"Returns the
engine-side `Subscription` handle for `Named(_)` routes."* VERIFIED: the body checks the handler is
registered, calls `handler_route_log.record_named(...)`, and returns `Ok(())`. It does not dispatch,
and the signature is `Result<(), EngineError>` — it cannot return the handle the doc describes. The
sibling `emit_with_handler` carries an honest in-code comment saying exactly this (*"It does NOT
invoke the named handler subgraph… wires at G16-D"*), so **the codebase knows and the public doc
does not.** This is the rule-14 shape on a document that ships with the tag, and it is directly
load-bearing on §4.174's own amendment (fold-as-subgraph-triggered-on-change), whose trigger this
is. Per rule #15 the split is: **doc→code now** for the disclosure (rewrite the docstring; delete
the return-handle sentence outright — that half is simply wrong and needs no build), **code→doc
later** for the behaviour, via the §4.174 build.

**Disclosure — user views neither backfill nor persist.** `docs/HOW-IT-WORKS.md` defines a View as
*"a materialized query result kept current by subscribing to graph changes."* Nothing there is
false, and I am not calling it a false record. But "materialized view" carries backfill everywhere
else in the industry, and an adopter will read it that way — while VERIFIED behaviour is: empty at
registration over an existing corpus, and empty again after restart. One sentence, same posture
shape as the `MAX_DECODE_BYTES` clause already owed in §5 of the fit-gaps record.

**Disclosure — property indexing is unconditional.** Every property × every label, no opt-out;
MEASURED at ~13× ingest and 2.11× disk. A performance-posture clause, same shape as above.

### Composing — the builds

| Need | Record | What it unlocks for them |
|---|---|---|
| **Binding grammar** — a handler naming its input | `binding-grammar.md`, §4.170 | **~119 of 159 rules** move from detected to prevented. This is the gate. |
| **Single-owner admission** — in-transaction check at the commit boundary | `bounded-resources.md`, §4.172 | capacity **+ composite-key uniqueness (C=1) + non-overlap** — three families, one build |
| **Aggregation** — fold kernel *or* the §10 subgraph pattern | `ivm-aggregation.md`, §4.171 | reporting balances (never enforcement) |
| **`Decimal` scalar** | fit-gaps §3.2a | money scale, declared and validatable |

### Not in any record yet — the true delta this pass found

1. **Change-triggered handler dispatch does not exist.** The §4.174 amendment's "subgraph triggered
   on change" has no trigger at HEAD. Largest and nearest of the four.
2. **In-transaction SUM, not just count.** `bounded-resources.md` proposes
   `Transaction::count_by_property`. Their bound is `pool − Σ(auth_amount) ≥ 0` — a sum over a money
   column. Unit-rows make a party of 9 countable; they do not make a $50,000 grant pool countable.
   Same serial point, same placement — a wider in-transaction read, not a new mechanism.
3. **Predicate admission, not just count admission.** Non-overlap is mutual exclusion — the same
   serialization requirement with a predicate in place of a count. Same serial point again.
4. **`Engine`-level `get_by_label` / `get_by_property` (+ `_as`).** Correction (b). Additive
   (methods on an existing struct), so **not now-or-never** — say so plainly rather than inflating
   the pre-tag list. But its absence is precisely why the §3.8 answer reads worse than the
   measurements: the 0.43 ms lookup is real and nobody can call it.

**Placement recommendation: leave the sequence as §4.174 has it.** Nothing found here changes wire
format, and nothing found here justifies pulling a build into Core. The pre-tag obligation this pass
adds is three paragraphs of prose, not a mechanism.

---

## 6. Now-or-never

Strictly: **only the disclosures.** Every mechanism named above is additive — new callers on
existing seams, a wider in-transaction read, methods on existing structs, a config field. None
touches canonical bytes or a frozen shape.

1. **The `subscribe_with_handler` rustdoc correction.** A frozen API doc that describes dispatch the
   binary does not perform, and a return value the signature cannot express.
2. **The view backfill/persistence disclosure.**
3. **The unconditional-property-index posture clause.**

Anything beyond these three would be inflating the pre-tag list, and this project's own standing
rule is to resist that.

---

## 7. Open questions for Ben

1. **Does the reply go out before a rule is written end-to-end as an actual subgraph?** Neither
   investigation wrote one, and that omission is exactly how the input-binding gap survived two
   passes. It is a few hours against a decision this size, and it will either confirm the mechanism
   or find the next missing joint. **ORCH lean: write one first.** This is the strongest
   recommendation in the document.
2. **Do we tell them the enforcement date, or only the shape?** We can honestly say verification
   works today and enforcement is the trio's first item, already prioritised. We cannot honestly put
   a date on binding grammar. Saying "prioritised, no date" to an adopter with statutory obligations
   is more useful than silence and less useful than a date — Ben's call on how it is framed.
3. **Is the "1.76M rows" framing corrected publicly?** It is the whole database across 248 tables
   (124 empty), largest single entity ~440k — and we have repeated the aggregate figure in a way
   that made their scale sound like one hot table. Correcting it in the reply is a small credibility
   gain; §8 records why it was unfair.
4. **Does `Engine` gain the read forwarders before the tag anyway?** Additive either way, so this is
   taste rather than necessity — but the `_as` attribution shape is a v1-surface design decision
   (baked-in #18) and deciding it alongside the un-attributed one is cheaper than deciding it twice.
5. **Crash recovery: do we characterise it before anyone promises a morning-open SLA?** MEASURED
   once at five minutes; whether it tracks corpus size or interrupted-transaction size is unknown.
6. **Encryption-at-rest is unmeasured and could change the query answer materially.** A per-DID
   encrypted corpus cannot use the plaintext property index the same way. Given that the property
   index is the fast path, this is the largest unmeasured thing in the volume story.

---

## 8. Refutation ledger

| # | Claim | Verdict |
|---|---|---|
| R1 | **§3.8's framing — "a balance fold over 1.76M rows is O(n)… a cached O(n) scan is still O(n)"** | **UNFAIR, twice.** (i) Steady state under either aggregation shape is **O(1) per write**; the corpus is walked once, at backfill. (ii) **1,758,248 is the entire database**, not one entity — 248 tables, 124 empty, largest single entity ~440k, most five figures. §3.8 made their scale sound like one hot table. Both corrections favour the engine, and the residual worry §3.8 raised (backfill walks the corpus once) is real but MEASURED at **30 seconds**. |
| R2 | "IVM views can express the invariants" (the museum's framing, and ours by omission) | **REFUTED.** Zero of 159, before or after the fold. First-label matching with no predicate. |
| R3 | "Handlers can express row-local and referential rules today" (our own first pass) | **REFUTED** by the verifier, ORCH-confirmed at four sites. Handlers cannot see their input. Enforcement is post-binding, full stop. |
| R4 | "Verification needs a second store" | **REFUTED.** Host code over the read API is a complete query surface, and MEASURED fast. Postgres stays as a cutover oracle only. |
| R5 | "Capacity, uniqueness and non-overlap are three separate gaps" | **REFUTED.** One build at one serial point. |
| R6 | "Storage/throughput/memory are a risk at this scale" | **REFUTED.** 2.28 GB, 16 ms open, 85 MB RSS, ~6 ms/write on an 8 GB laptop. |
| R7 | "The version-chained balance is auditable across restart" (§4.174 amendment's premise) | **NOT YET TRUE.** VERIFIED: `anchor_store` is `Mutex<BTreeMap<..>>` initialised empty with no rehydration; `benten_core::version::Anchor` holds its chain in an `Arc<Mutex<Vec<..>>>`. Version *nodes* persist; the chain and its CURRENT pointer do not. **Cheaper to fix than it sounds** — `rehydrate_handler_version_chains_from_zone` is a live in-tree precedent (Compromise #18, recorded CLOSED by exactly this technique). |
| R8 | "SUBSCRIBE delivery is post-commit and asynchronous" (§4.174 amendment's other premise) | **HALF TRUE.** Post-commit: **yes**, VERIFIED — the transaction guard drops before fan-out, so the re-entrancy objection is genuinely dead. Asynchronous: **no** — `ChangeBroadcast` is *"synchronous. No tokio, no broadcast channel"* and the redb backend runs callbacks *"SYNCHRONOUSLY on the committing thread"* with a MUST-NOT-BLOCK contract. A fold in that callback adds to every sale's latency. The blocker is halved, not dodged. |

---

## 9. Verifier finding triage — all 26 probes dispositioned

**ACCEPTED (9).**

- **P2 — handlers cannot see their input. REFUTED our claim.** Accepted and independently
  re-verified at four sites. Reframes the whole enforcement answer. The single most valuable
  finding in the pass.
- **P3 — edge access is worse than stated:** `put_edge`/`delete_edge` are zero-caller too.
  Accepted; it *strengthens* the modelling recommendation and produced the "write both" rule in §3.
- **P8 — aggregate work understated:** 42 rules contain an aggregate, 38 `GROUP BY`, 23 `HAVING`,
  against ~24 aggregate-shaped rows in the table. Accepted; the table's single-primary-shape method
  is flagged in place rather than re-run.
- **P13 — RSS is 85/75 MB, not 193/130.** Accepted; corrected everywhere.
- **P14 — till path is ~6 ms, not 4.627.** Accepted, including the false-precision point.
- **P12 — the fold/random gap is 15×, not 17×.** Accepted; same phenomenon.
- **P20 — the 13× ingest ratio is extrapolated.** Accepted; labelled REASONED in §1.
- **P24 — anchor persistence is cheaper to fix than claimed** (Compromise #18 precedent). Accepted;
  it is why R7 above says "cheaper than it sounds."
- **P21 / Blocker A — no change-triggered dispatch.** Accepted, ORCH-verified, and escalated: it
  also makes the public rustdoc a FALSE-RECORD (§5), which the verifier did not reach.

**REJECTED, with reason (1, partial).**

- **"Lengthen the Postgres clock to after binding grammar lands"** (verifier's item 7).
  **Rejected.** The oracle's job is proving the re-expression, which completes at migration parity.
  Enforcement correctness is carried by the surviving detectors, which become zero-row assertions —
  a stronger control than a second store and one that does not reintroduce two write paths. The one
  thing that control genuinely misses (false refusals) is not caught by Postgres either without
  dual-writing; the right answer is a typed rejection log, and §4 says so. The rest of item 7 —
  keep the oracle, give it an end date — is accepted.

**OPEN (3).**

- **"Write one rule end-to-end as a subgraph before replying."** Not done here; escalated to Ben as
  open question #1 with an ORCH lean toward doing it. It is the correct next action and it is
  outside a synthesis pass's remit to skip *or* to perform unilaterally.
- **Crash-recovery scaling.** One 300 s observation; cause unquantified. Question #5.
- **Encryption-at-rest interaction with the property index.** Unmeasured; question #6.

**SOUND, absorbed without change (13).** Every structural spot-check (`ViewResult` variants,
`LabelPattern`, `GenericKernel::read` ignoring its query, identity `Projection`, no-backfill
`register_view`, `arith_sum` wrapping and float promotion, redb's 1 GiB default with no
`set_cache_size` caller, unconditional property indexing, `"n:" ++ cid` keying, one-transaction-per-
`create_node`, SUBSCRIBE's 1000-event/24-hour retention, the synchronous-callback docstrings,
`DurabilityMode::Group` → `Immediate` confirmed at runtime) and every museum-side number
(`rules.json` = 159 + 14, the "129" citations real, 94-of-159 multi-entity, 1,758,248 / 248 / 124 /
3,733 / 2,464, the 5-column `posting_leg_is_unique` key). Both result-set cardinalities reproduced
**exactly** (144,428 and 1,648,075) and the full fold within 1% (29.47 s vs 29.73 s) — these are
real runs against the real corpus. Harness audited as driving the real write path; `git diff
27b2568b..ec541a63` over the four crates under test is empty, so the measured binary matches the
cited source.

---

## 10. What is still unmeasured

Named so absence is not read as a pass: `Engine`-level overhead (capability checks, `read_node_as`,
the napi boundary — §1 is the storage layer only); **encryption at rest**; sync cost for 1.7M nodes
to a second peer; concurrent readers during writes; crash-recovery scaling; deletes and compaction;
server-class hardware (the argument that the knee barely moves is REASONED, not measured).

---

## 11. Receiving rows

- **`engine-fit-and-gaps.md` §3.8** — status OPEN → **ANSWERED**, both halves, with the measurements
  above and the R1 correction to its own framing recorded.
- **The museum's "what to check next" #1** — answered: views express none; host code expresses all
  159 today; enforcement is post-binding; no second store in the product.
- **New pre-tag items** — the three §6 disclosures need a receiving row in the same commit that
  records them, per the fit-gaps banner. The `subscribe_with_handler` rustdoc correction is a
  FALSE-RECORD and belongs on the W-REC freeze-record-honesty wave, not in a future-work file.
- **`phase-4-backlog.md` §4.174** — unchanged in sequence; amend with the four §5 delta items
  (change-triggered dispatch, in-transaction SUM, predicate admission, `Engine` read forwarders) and
  the R7/R8 corrections to its own amendment's two premises.
