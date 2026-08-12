<!-- ORCH ground-truth note (2026-08-10, §3.5n): the four decisive claims below were
re-verified first-hand at bf964354+ before this record landed — (1) `Deleted` events carry
the read-before-delete pre-image (`store.rs:508-518`); (2) `append_version` moves CURRENT
in memory with no event (`engine_diagnostics.rs:592`); (3) `apply_atrium_merge` persists
the zone as ONE `"version"` node with `loro:` text props (`engine.rs:1768-1783`); (4) the
subscriber's non-budget error arm logs and leaves the view Fresh (`subscriber.rs:307-313`).
All four hold verbatim. -->

# Aggregation as IVM views — completing §3.3

**Status: R0-INPUT.** This is a design record, not a plan. Per the ADDL observance rule, the build gets its own R0→R1 pipeline when scheduled; this document is that pipeline's input, the way `binding-grammar.md` fed §3.1. Written 2026-08-10 out of the engine-fit-and-gaps investigation (`docs/future/engine-fit-and-gaps.md` §3.3), synthesized from 4 adversarially-verified lenses; conflicts resolved below with first-hand re-verification at worktree `606fdeda`.

---

## 1. The answer in a paragraph

Benten gets aggregation as a **bespoke abelian fold kernel inside benten-ivm, riding `Strategy::B`**, declared by an additive `aggregate` field on the `#[non_exhaustive]` `UserViewSpecBuilder` (`crates/benten-engine/src/outcome.rs:103-110`), returning a new additive `ViewResult::Aggregates` variant (`crates/benten-ivm/src/view.rs:248-257` is `#[non_exhaustive]` for exactly this), accumulating in **i128** and crossing the TS boundary as a **decimal string, always**. v1 folds are **Sum and Count, optionally grouped** — the group-homomorphic case, which is fully incremental over the engine's real event stream *including deletes*, because `Deleted` events carry the full pre-image (`crates/benten-graph/src/store.rs:508-518`). We adopt DBSP's two-line theory (a linear operator is its own incremental version) and reject both candidate crates; `Strategy::Reserved` stays reserved for true Z-set cancellation (Min/Max under general retraction). Every code surface this needs is additive through already-applied freeze valves — **nothing is now-or-never except disclosure sentences and one cosmetic rename window** (§5). The declaration surface is a day of work; the actual feature is the correctness contract behind it: engine-side **backfill-on-register**, **fail-closed error routing** (today's subscriber silently discards fold errors while the view reports Fresh), the **CID-uniqueness contract** on ledger rows, the **versioned-entity typed-reject**, and the honest disclosure that balance views are **per-engine at v1** because sync merges deliver zone snapshots, not per-entry nodes.

## 2. The delta model

### What the event stream actually is

`ChangeEvent` carries `cid` / full `labels` / `kind` / monotonic `tx_id` / attribution triple / `node: Option<Node>` / `edge_endpoints` (`store.rs:484-525`). `ChangeKind` has five variants (`store.rs:445-462`), of which **`Updated` is never emitted by any production path at HEAD** — it is reserved for a `PendingOp::UpdateAnchor` that was never built (`crates/benten-graph/src/transaction.rs:149-155`; two independent adversarial greps confirmed zero production constructors). Content-addressing makes every logical update a new CID ⇒ `Created` (`transaction.rs:143-147`). Fan-out is synchronous, post-commit, on the committing thread, with the transaction guard released *before* fan-out (#645, `crates/benten-graph/src/redb_backend.rs:2105-2118`) — so cross-transaction event interleaving is real and arrival order is not tx_id order.

### Is the ledger case insert-only? RESOLVED: no at the engine level, yes as schema discipline — and the difference is cheap

All four lenses converged after verification: the money *path* is insert-shaped (sync merge inserts, revocation is an INSERT of a `system:CapabilityRevocation` node per `engine_caps.rs:200-217` — the engine's own born-dead pattern), but the engine cannot promise insert-only. Three production delete routes exist: `Engine::delete_node` (napi-exposed, `bindings/napi/src/lib.rs`), `update_node` = delete+put in one tx (`engine_crud.rs:190-193`), and the evaluator's `:delete` walk route (`primitive_host.rs` → `engine.rs:4224-4226`). **This does not require DBSP machinery**: `Deleted` events carry the complete pre-delete Node captured read-before-delete (`store.rs:508-518`, materialized at `transaction.rs:562-605`), so Sum retraction is O(1) and fully informed — stronger than DBSP's usual setting, where the delta stream must be arranged to carry weights. The design: Sum/Count **handle** `Deleted` by subtraction; a view declared `insert_only` that receives a `Deleted` for an admitted CID **fails closed** (self-stale + typed error), never skips.

**Soundness precondition (survived adversarial review as the condition, not the aside):** subtract-on-`Deleted` is sound only if the matching `Created` was applied to this view instance. v1 therefore keeps a per-view **admitted-CID set** — the same state `GenericKernel` already pays (`algorithm_b.rs:576, 661-665`) — so a `Deleted` for a never-admitted row is a detected anomaly, not silent corruption. Dropping the set behind a proven backfill-watermark lifecycle is a named R1-pipeline optimization, not a v1 assumption. Corollary: delete→re-put re-emits `Created` for the same CID (`store.rs:447` — "once, ever" is false), so any backfill-handover seen-CID check must be handover-scoped, never permanent.

### Two traps on the "safe" insert path

1. **Dedup undercount.** A byte-identical re-put emits **no event** (`redb_backend.rs:1521-1547`; same matrix on the transactional path, `transaction.rs:388-428`). Exactly-once-per-CID means a Sum can never double-count — but two economically distinct, byte-identical deposits collapse to one CID and the second **vanishes silently**; a reversal-of-a-reversal is byte-identical to the original absent a nonce. Contract, stated in the aggregate's docstring and the museum guidance: **every ledger row carries a uniqueness disambiguator** (HLC timestamp / nonce property). Engine precedent exists: monotonic `createdAt` on CRUD walk-writes (`engine.rs:116, 4290`).
2. **Ordering.** Cross-tx interleave + per-event mutex application (`crates/benten-ivm/src/subscriber.rs:365-376`) means a reader can observe one leg of a double entry and events arrive in non-deterministic order. Consequence: **only commutative folds are admitted at registration**. Sum/Count qualify; any order-sensitive fold ("last-by-arrival") is refused typed; "latest" must key off data-carried order, never arrival. Tx-atomic view application has an additive future seam (`on_change_batch` defaulted method on `ChangeSubscriber`, `store.rs:597-602`) — declined for v1, named here.

### Version-chain CURRENT moves — CONFLICT RESOLVED with first-hand evidence

The delta-model lens claimed CURRENT tracking rides `NEXT_VERSION`-labeled events; one verification claimed `EdgeCreated`. Both wrong about production. I re-read `Engine::append_version` (`engine_diagnostics.rs:~560-597`) this session: it emits exactly **one `Created`** for the new Version Node via `backend.transaction(|tx| tx.put_node(node))`, then moves CURRENT in the **in-memory anchor store** (`entry.current = new_head`) — **no event fires for the pointer move**, and grep confirms no production code creates `NEXT_VERSION` edges (constant definition only; `VersionCurrentView`'s edge arms are production-dead, fed by synthetic test events). Consequence: aggregation over **versioned entities has no retraction event to key off** — every edit double-counts with nothing to correlate. **v1 typed-rejects registration of an aggregate whose input label participates in version chains** (Case B). The mechanism version-chains would need is the anchor-identity event widening that `version_current.rs:161-163` itself names — see §5 on why that feeds the §4.43 decision.

### Sync merge — CONFLICT RESOLVED: insert-only is true but insufficient

All lenses correctly reported `apply_atrium_merge` is insert-only (zero deletes in `crates/benten-sync/src/`). Two verifications found the governing deeper fact, which I re-confirmed at `engine.rs:1768-1783`: the merge persists the entire zone as **ONE node labeled `"version"` with `loro:<key>` → `Value::text(...)` string props**, via `append_version`. Remote ledger entries **never materialize as individually-labeled nodes on the receiving peer**. A label-matched balance view therefore silently excludes everything synced in. **Balance views are per-engine-local at v1, disclosed as such**; multi-peer balance requires zone hydration (per-entry materialization), a named dependency, not this design's scope. The CRDT-convergence argument (abelian fold over a convergent set converges everywhere) is algebraically true and vacuous at HEAD — the two peers' event streams are not permutations of each other.

### The fold taxonomy — the abelian carve, precisely

Delivery model: unordered-across-tx, per-event-atomic, exactly-once-per-CID, retractions-carry-bodies, no backfill primitive.

| Fold | v1? | State | Why |
|---|---|---|---|
| Count, Sum (i128 over `Value::Int`) | **yes** | O(1) + admitted-CID set | Group homomorphism: insert = add, retract = subtract the event-carried pre-image; order-free |
| Grouped balance (key → sum) | **yes** | O(#groups) + set | Per-key group fold — **the ledger view** |
| Avg | client-side | — | Sum ÷ Count, exact |
| Min/Max/Top-K | no | O(n) multiset | Monoid without inverse; retracting the extremum needs the runner-up. "Mark stale and rebuild" is **refuted**: `rebuild()` is documented clear-and-reset, not replay (`view.rs:330-342`), and the kernel has no input source — it would restart from zero. Destination: the `Strategy::Reserved` Z-set future |
| Distinct-count (CID) | already exists | O(n) | `GenericKernel`'s `BTreeSet<Cid>` is exactly this |
| Order-sensitive folds | never | — | Unsound under #645 interleave; refuse at registration |
| Float folds | never | — | Non-associative + non-deterministic order ⇒ replica-divergent "balances"; typed reject |

### The two genuinely unbuilt pieces (the real feature)

1. **Backfill.** `register_user_view` (`engine_views.rs`) does no scan; benten-ivm structurally cannot scan (no store dependency — baked-in #2, verified via `Cargo.toml`). A balance view registered over an existing ledger returns **0, confidently**. The build: engine-side backfill at registration (label-index scan → synthesize `Created`s into the view → attach live, with `tx_id` watermark closing the handover gap), plus a typed **not-backfilled refusal state** so a partially-initialized aggregate can never answer.
2. **The error channel.** Re-verified this session at `subscriber.rs:296-313`: `BudgetExceeded` → stale; **any other `ViewError` → log-and-continue, view stays Fresh**. A fold whose `update` errors typed-without-self-staling silently diverges while reporting Fresh — the worst class available. The fold's contract: **mark self stale internally before returning any error** (the `BudgetTracker` pattern). This belongs to the aggregate's update contract, not a blanket subscriber change (`PatternMismatch` is a benign signal for listing views).

## 3. The surface

**Declaration** — additive field, strategy stays B (consistent with `subgraph_spec.rs:120-121` "the engine-boundary strategy for either lane is `Strategy::B`"; `Reserved` unspent):

```rust
#[non_exhaustive] pub struct AggregateSpec { fold: Fold, group_by: Option<String> }
pub enum Fold { Sum { over: String }, Count }        // builder-constructed, additive later members
impl UserViewSpecBuilder { pub fn aggregate(self, a: AggregateSpec) -> Self }
```
```ts
engine.registerUserView({ id: "balance_by_account",
  inputPattern: { label: "ledger:entry" },
  aggregate: { fold: "sum", over: "amount_minor", groupBy: "account" } });
```

Selection is **label-only** (`UserViewInputPattern` = `Label`/`AnchorPrefix`, `outcome.rs:45-54`) — "posted vs draft" is modeled as distinct labels, a stated v1 exclusion. Note the kernel matches **first label only** (`GenericKernel::first_label_matches`) — a documented constraint on ledger label design.

**Result shape.** Internal: `ViewResult::Aggregates(BTreeMap<String, AggregateRow { value: i128, count: u64, skipped: u64 }>)`; ungrouped under a documented sentinel key; group keys must be `Value::Text` (typed reject otherwise). Engine projection: **one synthetic Node per group** (`system:ivm:AggregateRow`, props `{group, value: Text(decimal), count, skipped}`) through the existing `Outcome.list` path — `ViewQuery.limit/offset` already exist and a later `group_key` filter is additive (`view.rs:226-243`, `#[non_exhaustive]`). `skipped` makes the absence rule observable: **absent property = skip-and-count; wrong-typed property = typed fail-closed error.** Aggregates are **strict-only** (no `allow_stale`): stale currently projects to an empty list (`engine_views.rs`), and "all balances: zero rows" is itself the silent-wrong shape.

**The i128/TS answer.** `Value::Int(i64)` already crosses to JS as f64 (`bindings/napi/src/node.rs:323`; napi `serde-json` feature) — lossy above 2⁵³ today, and the sum is the value engineered to grow; i128 is inexpressible in `Value` (8 variants, cannot grow, per §3.2a). Therefore **the aggregate value crosses as a decimal string, always** — one encoding, no fits-in-Int dual-type foot-gun; the TS DSL wraps to `BigInt`. Precedent: CIDs cross as base32 strings (`lib.rs`). `read_view(view_id, _query)` needs **no signature change** (`lib.rs`).

**The mirror cascade — with one correctness item, not just ceremony.** napi fails loud on unknown spec fields (`bindings/napi/src/view.rs:61, 81-90`) — widen `KNOWN_USER_VIEW_FIELDS` additively; old *engines* reject new specs typed. **But one layer up the fail-loud claim is false**: TS `validateUserViewSpec` (`packages/engine/src/views.ts:70-126`) has no unknown-field rejection and `userViewSpecToNativeJson` (`views.ts:143-154`) copies only `id/inputPattern/strategy` — an old `@benten/engine` package handed an aggregate spec **silently strips it and registers a plain listing view**, the exact D-93 shape. TS-side unknown-field rejection ships in the same change, filed as correctness. While touching that surface, retire the adjacent FALSE-RECORD: `types.ts:1063-1065` claims `project?` is "round-tripped to the Rust side" — it is stripped (rule 14/15). ErrorCode mints (~3, full catalog + TS mirror ceremony): aggregate-type-mismatch (covers float/wrong-type/mixed-scale), group-key-not-text, view-not-backfilled.

**Graph-native composition: composes, does not foreclose.** `register_user_view` already persists every spec as a content-addressed `system:IVMView` Node (`engine_views.rs`) — the Rust API is already a Node constructor. Requirements: (a) `aggregate_*` props land on the persisted Node in the same change (spec↔Node bijection; absent props = no aggregate); (b) the anticipated future loader (`engine_views.rs`) **typed-rejects** unknown aggregate props rather than hydrating a silently non-aggregating view. A later views-as-graph-Nodes feature hydrates the same schema.

## 4. The exactness result

**§3.2a makes §3.3 exact — confirmed.** With scale schema-declared over `Value::Int`, a same-scale balance is plain i64 addition into an i128 accumulator: no decimal arithmetic in the engine, ever; an i128 cannot overflow from i64 terms before ~2⁶⁴ events; checked narrowing only at the read boundary (typed error), matching workspace precedent (`sandbox_output.rs:180-199`, `layer_c.rs:110-119` — checked/typed at boundaries, never silently wrapping data). Every surveyed production system (Kafka Streams, Materialize, event-sourcing projections) does grouped SUM exactly this way.

**Where it breaks, and the fail-closed behavior:**
- **(a) Mixed scales in one view.** `LabelPattern` selects by label, not scale (`algorithm_b.rs:379-388`), and **nothing at HEAD validates an instance `Value` against its declared `Scalar`** (`engine-fit-and-gaps.md:186-188`). v1: the spec pins `(property, expected_scale)`; this is declaration + disclosure (the same conformance trust schema-driven rendering extends), and becomes an enforced typed fail-closed check the day instance-validation lands. **Silent skip is never the answer** — a balance that omits rows is worse than one that refuses.
- **(b) f64 fields.** Refused at registration, typed. Non-associative addition under non-deterministic cross-tx order means two replicas could disagree bit-for-bit on a "balance."
- **(c) Scale migration.** Immutable rows keep their birth scale; a migration is a new schema CID. A cross-epoch view needs per-epoch declared scales with checked 10^Δ normalization in i128, or the app restates via reversal rows; **undeclared mixing is refused**.

All mismatch errors route through the self-stale channel (§2) — the view refuses, visibly; it never answers wrong while Fresh.

## 5. Now-or-never

**No pre-tag code is required to keep aggregation buildable.** Verified against the frozen baselines: `ViewResult`, `ViewQuery`, `ViewError`, `KernelOutput`, `TypedOutputProjection`, `UserViewSpec` family all `#[non_exhaustive]`; the `View` trait carries an aggregating impl unchanged; napi/TS surfaces need nothing signature-level. Owed pre-tag, all record-work:

1. **Freeze-record disclosure sentence** (adjacent to the item-11 `Strategy` carve-out row, `V1-FROZEN-INTERFACE.md:1333`), amended per verification — drafted:
   > **Views return references, not computed values, at v1-beta.** `ViewResult` is exhaustively `Cids`/`Current`/`Rules` — a v1-beta view answers *which* Nodes match, never a value computed *over* them (no sum/count/fold; a balance over append-only ledger rows is not computable by the engine at this tag). The named additive path is a fold kernel under `Strategy::B`: a new `View` impl + an additive `#[non_exhaustive]` `ViewResult` variant + additive spec fields on `UserViewSpec` — with **all aggregation configuration in spec types, never in the `Strategy` discriminant**. `Strategy` is frozen-cardinality `{A, B, Reserved}` with `Reserved` a permanent unit variant; per this row's own clause, a 4th strategy is a Composing-time architectural decision, not a SemVer field addition.
2. **`Strategy::Reserved` identifier freezes** (baseline `docs/public-api/benten-ivm.txt:183`; deliberately closed enum, `strategy.rs:53-58`). If a meaningful name (`ZSet`) is ever wanted, the rename is legal only pre-tag. Cosmetic, permanent — surfaced; **RESOLVED (Ben-delegated, 2026-08-10): KEEP `Reserved`.** Renaming to `ZSet` would bake an algorithm family into a frozen identifier — the mistake baked-in #5 exists to prevent (name the SLOT, never the algorithm; the rustdoc names the intended use and rustdoc is retensable forever). Spend nothing.
3. **`ChangeEvent`/`ChangeKind` SemVer posture — cite, don't re-mint.** The disposition already exists: `V1-FROZEN-INTERFACE.md:1336`, F-05/R12 named-deferral → `phase-4-backlog.md §4.43` (apply `#[non_exhaustive]` + migrate literals, OR accept the field-level lock with rationale). The v1 fold needs **no** `ChangeEvent` widening — Case A is complete on today's events. But this design adds a concrete future consumer to that parked decision: versioned-entity aggregation (Case B) requires the anchor-identity event widening `version_current.rs:161-163` already names, and `ChangeKind` is matched exhaustively without wildcards downstream. **CORRECTED (2026-08-10): the pre-tag urgency dissolves.** The freeze contract's Composing-phase escape valve permits ADDING `#[non_exhaustive]` post-tag, then the new variant — a legal two-step additive path. Nothing is foreclosed; §4.43 stays parked with Case B recorded as its first concrete consumer, decided before the Case-B build rather than before the tag.
4. **Honesty tail (~30 LOC, doc/string):** `crates/benten-ivm/src/testing.rs:66` still ships `deferred_to_phase: "Phase 3+"` (four phase-closes stale) with `testing.rs:54-56` + `strategy.rs:46,82` citing the *closed* #1084 as open (one test pin to update: `tests/strategy_c_reserved.rs`); `ERROR-CATALOG.md:~542` context still reads `"A" | "B" | "C"`; `strategy.rs:10-12` "the ONLY IVM type the engine names" is false at HEAD. TS `Strategy`'s dead `"C"` member lives forever (removal = narrowing) — harmless, leave it.

## 6. Prior art: adopt / port / bespoke

**Decision: bespoke fold kernel; adopt DBSP's algebra as theory; adopt no crate.** The load-bearing theory is one corollary of the DBSP paper (Budiu/McSherry/Ryzhyk/Tannen, PVLDB 2023): a linear operator is its own incremental version — Sum and Count are linear over Z-sets, so incremental maintenance is `acc ± extract(event)`. The `dbsp` crate (verified: v0.331.0, **69 runtime deps** incl. tokio/rkyv/a second allocator's sys-crate, 271 published 0.x versions on Feldera's product train) is a scheduler-owning *circuit runtime* — running it inside `ChangeSubscriber::on_change`, a synchronous must-not-block post-commit commit-thread callback (`store.rs:597-602`), to add an i128, is indefensible under baked-in #19's compile-time-trust weight test. `differential-dataflow` (0.25.1, requires the timely runtime) fails identically. The correct primary literature for the v1 scope is not DBSP at all but **Kafka Streams' adder/subtractor split and event-sourcing projections** — the industry API is literally shaped around insert-only-vs-retraction, and our retractions arrive pre-weighted.

**Revisit iff:** (a) composed multi-operator incremental queries (join→group→sum cascades) or non-invertible aggregates under general retraction become real demand → build a small bespoke **Z-set module** (~200-400 LOC, proptest-verified: commutativity/associativity/inverse round-trip) behind `Strategy::Reserved` — still never the `dbsp` crate; (b) `dbsp` ships a library-mode core (no scheduler ownership) with an API-stability contract → re-evaluate adoption of that module only.

## 7. What we tell the museum

**The balance story:** model money as append-only `ledger:entry` Nodes — immutable, each carrying amount (`Value::Int` at the schema-declared scale), account key, and a **uniqueness nonce/HLC** (two byte-identical deposits are one CID; the engine will deduplicate them silently). Corrections are reversal inserts, never edits — an edited content-addressed row is a *new* node with the old one still counted. Register a grouped-Sum view per balance; the engine backfills it at registration, maintains it incrementally on every commit, and returns exact integers at your declared scale as decimal strings. When anything is wrong — a float where an integer belongs, a mixed scale, an un-backfilled view — the view **refuses with a typed error rather than answering wrong**.

**The honest limits, plainly:** (1) views select by label only — model posted-vs-draft as separate labels; (2) **a balance view is per-engine at v1** — entries synced in from peers arrive as zone snapshots, not as ledger-labeled nodes, and will not appear in your balance until zone hydration lands (named dependency); (3) versioned/editable entities cannot be aggregated at v1 — registration refuses them; (4) **aggregation makes invariants observable, never enforced.** Balance non-negativity, capacity bounds, and uniqueness all need a write-admission check *reading* the view — that is §3.4's `executionPolicy`, which has zero implementation hits at HEAD (`engine-fit-and-gaps.md:248-249`); referential integrity is not an aggregate at all. (The "129 SQL invariants" figure appears nowhere in `engine-fit-and-gaps.md`; the honest answer is per-shape, and the enforcement half is unbuilt — we say so rather than let "balance views" read as "balance constraints.")

## 7b. Should the fold itself be graph-native? (Ben, 2026-08-12)

**The question:** back IVM aggregation — and maybe view *formation* too — with graph-native logic,
so a view's fold is a handler rather than Rust inside `benten-ivm`. Conceptually this is the
meta-circular thesis pointed at views, and it deserves a real answer rather than a reflex.

**Formation is ALREADY declarative, and that half is settled.** `UserViewSpec` is a spec — an id,
an input pattern, a `Strategy` — not code. You do not write a view, you *declare* one. What the
spec lacks is a **fold field**; the shape it would slot into already exists. So "make formation
graph-native" is largely done, and the real question is only about the fold.

**Maintenance cannot be a graph walk, and the reason is structural rather than performance
squeamishness.** Three independent blockers, any one of which is sufficient:

1. **The fold runs in a must-not-block post-commit callback.** Putting an evaluator run there
   places an arbitrary-length computation inside the write path. This is the identical objection
   that declined the `dbsp` crate in §6 — a scheduler-owning runtime in a callback — and it
   applies with more force to a full graph walk.
2. **Re-entrancy against a blocking lock.** A handler fold would issue READs through the engine
   while the engine is mid-commit, and redb's `begin_write` **blocks** rather than failing fast.
   That is a deadlock shape, not a slowdown.
3. **Cost per changed row.** A graph walk versus an `i128` add, once per delta. Over the museum's
   1.76M-row corpus the difference is not a constant factor worth arguing about.

Termination, notably, is *not* a blocker — handlers are bounded by construction, so a fold
expressed as a handler would provably halt. The problem is where it runs, not whether it stops.

**The synthesis, and it is the same split the engine uses everywhere else: the graph DECLARES,
the engine EXECUTES.** A view declares its fold as a composition drawn from a **closed set of
engine-provided abelian operations** (`sum`, `count`, `min`, `max`, grouped or not). That is
graph-native *formation* — content-addressed, inspectable, shareable, versioned — with a native
*implementation* in the hot path. Extensibility comes from adding operations to the set
(additive, post-tag-safe), never from running arbitrary code in the commit path.

**One candidate vehicle worth evaluating at build time rather than inventing a fold language:**
`TRANSFORM` already ships a primitive with its own grammar and a bounded, non-re-entrant
evaluator. If a view's projection is expressible in that grammar, the fold declaration may be
able to reuse it outright — which would make the aggregation surface a generalization of a
shipped fragment rather than a new one, matching how §3.1 and §3.4 both resolved.

**Where this leaves the arbitrary-fold idea:** not rejected, relocated. A user-defined fold that
runs *outside* the commit path — a batch recomputation, a periodic rollup, an analytical view
materialized on demand — has none of the three blockers and is a legitimate later shape. What
cannot happen is arbitrary graph logic on the incremental maintenance path.

## 8. Considered and declined

- **Adopting `dbsp` (or differential-dataflow) wholesale** — §6: total runtime-model impedance, 69-dep trust surface, 0.x churn under a permanent freeze.
- **A 4th `Strategy` variant** — foreclosed as an *additive* move by the crate's own G23-0a posture and the carve-out row's clause ("a Composing-time architectural decision, NOT a SemVer non-breaking field addition," `V1-FROZEN-INTERFACE.md:1333`); spec-side configuration is the better design regardless, and `StrategyV2` remains the sanctioned escape (`strategy.rs:55`).
- **Landing the fold under `Strategy::Reserved` now** — the rustdoc reserves it for "Z-set / DBSP cancellation" (`strategy.rs:75-83`); an insert-biased abelian fold is not that; spending the named slot on the wrong thing forecloses the honest future use.
- **Aggregation as `Projection::Computed`** — refuted by signature: `Projection::apply` is per-node `Node → Node` narrowing (`algorithm_b.rs:425-457`); a fold is cross-node accumulation and cannot inhabit it. The prior-art lens's contrary recommendation did not survive verification.
- **Computing balances in handlers** — a handler-maintained balance node is a mutable aggregate fighting content-addressing: every update mints a new CID with the old node extant, concurrent walks race on read-modify-write, double-entry discipline across handlers is unenforceable, and it bypasses the one seam (`ChangeSubscriber`) built for derived state.
- **Min/Max in v1** — non-invertible; the cheap mitigation ("stale + rebuild") is refuted because `rebuild()` is clear-and-reset (`view.rs:330-342`) and would silently restart from zero.
- **Float folds; silent skip on scale/type mismatch; order-sensitive folds** — each a silent-wrong-balance generator; all typed-refused (§2, §4).
- **Widening `ChangeEvent` for the fold** — not needed; Case A is complete on today's events; the widening question belongs to §4.43 (§5.3).
- **`on_change_batch` (tx-atomic view application) now** — real seam, additive later via defaulted trait method; declined for v1 scope; this record is its named destination.

## 9. Open questions (genuinely undecidable from code)

1. **Multi-peer balance mechanics** — fold over materialized zone state vs per-entry replication on merge. Undecidable until zone hydration is designed; whichever lands, the per-engine fold is the substrate both compose over. **Urgency note (2026-08-10): §3.4's single-owner model means ENFORCEMENT never needs the multi-peer balance — a bound-admission check reads the owner's local view, which is exactly what v1 provides. Multi-peer matters only for cross-site REPORTING, which tolerates staleness.**
2. **§4.43 exercise vs accept** (`ChangeEvent`/`ChangeKind` `#[non_exhaustive]`) — already parked for Ben; this design adds the Case-B consumer as a new argument for exercising it pre-tag (§5.3).
3. **The `Reserved` rename window** (§5.2) — cosmetic, Ben's call, closes at the tag.

## What this closes

- `engine-fit-and-gaps.md` §3.3 (benten-ivm does not aggregate) — this is the design; §3.2a's schema-scale model is confirmed as the coupling that makes it exact.
- The version-chain and sync-merge questions from the R0 brief — resolved with first-hand evidence (CURRENT moves emit no event, `engine_diagnostics.rs:~592`; merges deliver one `"version"` snapshot node, `engine.rs:1768-1783`), and turned into the two disclosed v1 scope boundaries.
- The `types.ts:1063-1065` `project?` FALSE-RECORD — flagged for retirement in the same change that touches the TS mirror (rule 14/15).
- Insight #2 from the brief, corrected to its true form: the ledger case does not need insert-only to be exact — retractions arrive pre-weighted; what it needs is the nonce contract, the backfill, and the fail-closed error channel.
