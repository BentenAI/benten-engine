# `benten-ivm` — Internals deep-dive

Read-only audit doc. Plain-English walk through what this crate is, what it
contains, what it exposes, and what gaps / architectural questions it leaves
open. Pair with `CLAUDE.md` item #2 (IVM Algorithm B + per-view strategy + the
"engine names `Strategy` but not internals" boundary).

---

## 1. What this crate does

`benten-ivm` is the engine's **Incremental View Maintenance** layer. Whenever
something gets written to the graph (a node created, an edge created, a node
deleted), the storage layer fans a `ChangeEvent` out to a `Subscriber`, which
hands the event to every registered `View`. Each view maintains its own little
indexed answer to a question the engine wants to be able to ask cheaply later
("which capability grants apply to entity X?", "what's the current head of
version chain Y?", "give me the next 20 posts sorted by createdAt"). The views
are kept up to date by replaying changes incrementally — none of them re-scan
the whole graph on each read.

Architecturally the crate sits as a **subscriber, not an engine-internal
feature**. `benten-ivm` depends on `benten-graph` (for the `ChangeSubscriber`
trait + `ChangeEvent` shape). `benten-graph` does NOT depend on `benten-ivm`.
The engine names exactly one type from this crate as part of its public surface
— `benten_ivm::Strategy` — and treats the rest as inner machinery the engine
can hold by `Arc` and forward calls into, but does not understand. This is the
CLAUDE.md item #2 boundary, sharpened at Phase-3 R6-R3 (`r6-r3-arch-8`).

Phase-3 also generalized the kernel: `Strategy::B` is now a **single generic
algorithm** that handles arbitrary `(view_id, label_pattern, projection)`
triples for user-defined views, while transparently routing canonical view ids
through the 5 hand-written inner kernels as a fast path (per `ivm-disagree-1`
the 5 hand-written views are inner kernels OF Strategy::B, NOT separate
Strategy::A baselines — even though they each still `return Strategy::A` from
`View::strategy()` for back-compat reporting).

---

## 2. Dependency chain

**Inbound workspace deps (`Cargo.toml`):**
- `benten-core` (path dep) — `Node`, `Edge`, `Cid`, `Value`, `CoreError`.
- `benten-errors` (workspace dep) — the `ErrorCode` stable catalog used by
  `ViewError::code()`.
- `benten-graph` (path dep) — supplies `ChangeEvent`, `ChangeKind`, and the
  `ChangeSubscriber` trait this crate implements.
- `thiserror` (workspace) — error derives.
- `serde_json` (workspace) — used ONLY by `testing::criterion_estimates_mean_ns`
  to parse Criterion JSON output for the bench gate. Not on the runtime path.

**Inbound dev-deps:** `tempfile`, `proptest`, `criterion`, `blake3`.

**Direction of dependency:** strictly downward. `benten-ivm` reaches into
`benten-graph` and `benten-core`. Nothing in those crates reaches back. This is
load-bearing for CLAUDE.md item #2 — the evaluator (which lives in
`benten-eval` / `benten-engine`) is "deliberately ignorant of IVM"; IVM
subscribes to events the graph emits, never the reverse.

**Outbound consumers (workspace-wide grep):**
- `benten-engine` — the primary consumer. Holds `Option<Arc<Subscriber>>` on
  the engine struct; constructs canonical hand-written views in
  `engine_caps.rs` + `engine.rs`; routes user views through
  `Algorithm::register` in `engine_views.rs`. Surfaces `Engine::view_strategy`,
  `read_view_*`, `register_user_view`, `EngineError::ViewStrategyARefused` /
  `ViewStrategyCRefused`.
- `benten-eval/src/host.rs` — has IVM references (likely for the SANDBOX host
  surface that exposes view reads).
- `benten-sync/tests/host_atrium_publish_view_result_caps.rs` — sync test
  exercising view-result publishing.

Crates that explicitly do NOT depend on `benten-ivm`: `benten-core`,
`benten-graph`, `benten-errors`, `benten-caps`, `benten-id`,
`benten-dsl-compiler`. This boundary is intentional and matches CLAUDE.md
item #2.

---

## 3. Files inventory in `src/`

### `src/lib.rs` (80 LOC)

Crate root. Declares `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`,
imports `extern crate alloc;` (the crate is no_std-friendly in its core
types — only `subscriber.rs` reaches for `std::sync::Mutex` and `std::panic`).
Module tree: `algorithm_b`, `budget`, `strategy`, `subscriber`, `testing`,
`view`, `views`. Re-exports the public surface at the crate root.

Carries three Phase-3 `TODO`s naming work that's known-missing: dedicated
Criterion benchmarks per view against RESULTS.md §1 targets, the cascade
create→delete integration test (now closed by `tests/cascade_create_delete.rs`),
and the rebuild-equivalence event-replay path (still open — Phase 1's
"rebuild" doesn't actually replay events, it just clears state; see §9).

### `src/strategy.rs` (55 LOC)

Defines the `Strategy` enum — `{ A, B, C }`, closed (no `#[non_exhaustive]`),
`#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`. This is **the
load-bearing engine-boundary type**: the one type from `benten-ivm` the engine
names in its own public API (`Engine::view_strategy`, `EngineError::
ViewStrategyARefused`, etc).

Variants:
- `A` — hand-written incremental maintenance. The 5 Phase-1 views each return
  `Strategy::A` from `View::strategy()`. This is the **canonical fast-path
  classification** at the dispatch level (see `algorithm_b::dispatch_for`).
- `B` — the generalized Algorithm B kernel. Implemented by `AlgorithmBView`.
- `C` — Z-set / DBSP cancellation. **Reserved-not-implemented.** Constructing
  one returns `ViewError::StrategyNotImplemented`. The variant exists so
  exhaustive matches stay stable across phases.

D8-RESOLVED is explicit: strategy is fixed at construction time, no
auto-select, no runtime adaptation, no `set_strategy` companion. Future
algorithms would land as a new enum (`StrategyV2`), not a new variant on this
one — the closed set is part of the API contract.

### `src/view.rs` (422 LOC)

The shared `View` trait + error / state / query / result shapes + the
content-addressed `ViewDefinition`. This is where Phase 1 nailed down the
shape every view conforms to.

- `View` trait — `Send + Sync + Debug`. Methods: `update`, `read`,
  `read_allow_stale` (default delegates to `read`), `rebuild`, `id`,
  `is_stale`, `mark_stale` (default no-op), `strategy` (default
  `Strategy::A`). Object-safe by construction (`_assert_view_object_safe` is
  a compile-time check). No generic methods, no `Self: Sized` bounds — the
  subscriber stores `Box<dyn View>`.
- `ViewError` — `#[non_exhaustive]` enum with four variants: `Stale`,
  `PatternMismatch` (query shape doesn't match any maintained index),
  `BudgetExceeded`, `StrategyNotImplemented`. Each variant has a stable
  `ErrorCode` mapping via `ViewError::code()`. `IvmError` is a back-compat
  type alias for `ViewError` because some R3 tests named the type `IvmError`.
- `ViewState` — `{ Fresh, Stale }`. Stale is terminal until `rebuild`; Phase
  2+ adds async background recompute.
- `ViewBudget` — wraps `max_work_per_update: u64`. Constructor rejects 0
  (a zero-budget view is stale before any data arrives — misconfig, not a
  state). Note: the old `ViewBudget::DEFAULT = 1000` constant + `Default`
  impl was removed at Phase-2a R6 (`ivm-r6-2`) because it claimed an
  "ENGINE-SPEC §8 default" no view actually wired in. Every Phase-1 view
  constructs its `BudgetTracker` with `u64::MAX` directly.
- `ViewQuery` — single un-typed record carrying every field any view needs
  (`label`, `limit`, `offset`, `anchor_id`, `entity_cid`, `event_name`).
  Typed-per-view variant is named as Phase-2+ work.
- `ViewResult` — polymorphic enum: `Cids(Vec<Cid>)` for list-shape views,
  `Current(Option<Cid>)` for view 5 (version pointer), `Rules(BTreeMap)` for
  view 4 (governance rules).
- `ViewDefinition` — content-addressed; serializes as a Node with label
  `system:IVMView` carrying `view_id`, optional `input_pattern_label`, and a
  `strategy` property (stringified `"A" / "B" / "C"`). G8-A's `g8-concern-1`
  closure: the `strategy` field is folded into the CID input so two
  definitions that differ only in strategy don't content-hash collide.

### `src/budget.rs` (174 LOC)

`BudgetTracker` — the shared `remaining / original / stale` state machine
every Phase-1 view used to replicate inline. Extracted at Phase-2a R6
(`R-major-02`). Surface:
- `new(max)` — records `max` as both current and original cap; `u64::MAX`
  is the conventional unbounded sentinel.
- `try_consume(cost, view_id)` — saturating decrement; surfaces
  `BudgetExceeded` on the call that crosses zero, marks itself stale, and
  every subsequent call also surfaces `BudgetExceeded` (no further cost).
- `rebuild()` — restores the original cap, clears the stale flag.
- `mark_stale()` / `is_stale()` — flag setters/getters.
- `stale_error(view_id)` — convenience constructor for the
  `Err(ViewError::Stale { view_id })` boilerplate every view's read path
  carries.

`#[derive(Debug, Clone, Copy, ...)]`, no_std-friendly (the only stdlib touch
is `String::from` for the view-id strings in the error payloads). Five views
all hold a `BudgetTracker` by value.

### `src/subscriber.rs` (360 LOC)

The fan-out hub. `Subscriber` owns `std::sync::Mutex<Vec<Box<dyn View>>>` and
implements `benten_graph::ChangeSubscriber` so a `Arc<Subscriber>` is what
the engine hands `RedbBackend::register_subscriber`. The `on_change` impl
(takes `&self`) locks the mutex internally so callers don't have to.

Public surface:
- `new()` / `with_view(self, view) -> Self` (chainable builder).
- `register_view(&self, view)` — thread-safe registration on a live
  subscriber. Used by `Engine::register_user_view` post-construction.
- `view_count`, `view_ids`, `stale_count_tally`, `view_strategy(view_id)`,
  `view_is_stale(view_id)` — introspection used by the engine's
  `benten.ivm.view_stale_count` metric + the wave-8h IVM-B test assertion.
- `read_view(view_id, query)` / `read_view_allow_stale(view_id, query)` —
  named-view reads with the "view not registered" vs "view erred"
  distinction encoded as `Option<Result<..>>`.
- `route_change_event(&mut self, event)` — explicit fan-out used by tests
  that construct events by hand.

The load-bearing internal is the private free function `apply_event`. It:
1. Skips views already marked stale (cheap short-circuit).
2. Wraps `view.update(event)` in `catch_unwind(AssertUnwindSafe(...))`. A
   panicking view marks itself stale and the fan-out continues to the next
   view (closes `g5-p2-ivm-3` panic isolation). Other views never see the
   panic; the commit thread is not affected.
3. Translates `Err(BudgetExceeded(_))` into `view.mark_stale()` without
   panicking the loop.
4. Translates `Err(PatternMismatch(_))` into a stderr log via
   `log_view_error` (Phase-2-named `tracing` migration; deliberately
   discarded as non-fatal — PatternMismatch is the expected "this view
   doesn't handle this query shape" signal, not an alert).

`ChangeStreamSubscriber` is a public type alias for `Subscriber` (the G5-A
brief asked for the longer name; the shorter one survives because the R3
tests use it).

### `src/algorithm_b.rs` (1249 LOC, largest file)

The Phase-3 G15-A generalized kernel + the `AlgorithmBView` wrapper + the
internal dispatch router. Three load-bearing concepts:

**(a) Canonical-id dispatch router.** `dispatch_for(view_id) -> Strategy`
classifies a view-id into a strategy lane: canonical ids (the 5 hard-coded
strings) → `Strategy::A` (canonical fast-path classification); everything
else → `Strategy::B` (generic kernel lane). This router is `pub` but is
**INTERNAL** in the architectural sense — the engine refuses
`Strategy::A` user-view registration (per `ivm-major-5`), so callers don't
pick this. The hand-written 5 views are not registered through
user-view; they're inner kernels invoked by `AlgorithmBView::for_id` when
the dispatch table maps to them.

**(b) `LabelPattern` + `Projection`.** `LabelPattern::{Exact, AnchorPrefix}`
is the selector surface user views consume. `Exact("post")` matches Nodes
whose FIRST label equals `"post"`; `AnchorPrefix("crud:")` matches Nodes
whose first label STARTS WITH `"crud:"`. Phase-2b's stub silently coerced
`AnchorPrefix` to label equality — G15-A wave-5a fixed that. `Projection`
currently has one variant, `AllProps` (identity); future shape narrowing
(`PropSubset`, `Computed`) lifts to a richer enum without breaking the
kernel surface.

**(c) `AlgorithmBView` + `GenericKernel`.** `AlgorithmBView` is the public
wrapper consumers see. Its `View::strategy()` always returns `Strategy::B`
regardless of which inner kernel it holds — "the wrapper IS Strategy::B".
Inner kernel is one of:
- For canonical view ids: a `Box<dyn View>` holding one of the 5 hand-written
  views (via `for_id` or `for_id_with_budget`).
- For user-defined view ids: a `GenericKernel` (private struct) that holds
  a `BTreeSet<Cid>` of matched Node CIDs, plus a `BudgetTracker`. The
  kernel's `update` path: charge budget once per matching write, add the
  CID to the set on Created/Updated, remove on Deleted (only charge for
  observable deletes — deletes against never-admitted CIDs are free).

Public construction surface — three pairs:
- `for_id` / `for_id_with_budget` — canonical-only; errors with
  `PatternMismatch` for unknown view ids.
- `register` / `register_with_budget` — accepts either canonical or
  user-defined ids, routes through the dispatch table. Fails loud with
  `AlgorithmError::ViewLabelMismatch` (canonical id + exact label that
  disagrees with the canonical hardcoded label) or
  `AlgorithmError::CanonicalIdAnchorPrefixRefused` (canonical id +
  `AnchorPrefix` — the canonical kernels ignore the supplied pattern, so
  admitting a prefix selector would be a doc-vs-code-strength gap; closed
  by `g15a-mr-minor-4` / W9-T1).
- `try_register` — symmetric alias for `register`.

`Algorithm` is a public type alias for `AlgorithmBView` matching the
`benten_ivm::algorithm_b::Algorithm::register(...)` shape the test pins use.

Other public helpers:
- `is_canonical_view_id(view_id) -> bool`
- `hardcoded_label_for_id(view_id) -> Option<&'static str>` — returns the
  hardcoded label for 4 of the 5 canonical views (every one except
  `content_listing`, whose arm honors caller-supplied label). Used by the
  engine's `register_user_view` to surface
  `EngineError::ViewLabelMismatch` (catalog code `E_VIEW_LABEL_MISMATCH`)
  on disagreement.
- `materialize_full(&self) -> Vec<Cid>` — unfiltered materialization the
  per-row READ gate (`crates/benten-engine/src/ivm_view_read_gate.rs`)
  then row-filters against the actor cap-set.

The file also has ~440 lines of in-file unit tests covering dispatch,
fail-loud guards, budget-trip behavior, rebuild-after-stale, and the
generic kernel's label-matching semantics.

### `src/testing.rs` (117 LOC)

Test/dev-only helpers, exposed publicly (not `#[cfg(test)]`) because
consumer crates' integration tests reach into them.
- `testing_construct_view_with_strategy(strategy) -> Box<dyn View>` — picks
  one default view per strategy. Panics on `Strategy::C`.
- `try_construct_view_with_strategy(strategy) -> Result<...>` — same, but
  surfaces the typed `StrategyNotImplemented` error for `Strategy::C`
  instead of panicking.
- `criterion_estimates_mean_ns(group, view, axis, value)` — reads a
  Criterion `estimates.json` and returns the `mean.point_estimate` in
  nanoseconds. Centralized parser for the G8-A bench gate. Touches
  `std::fs`, so only available outside `no_std` builds.

### `src/views/mod.rs` (48 LOC)

Re-export hub for the 5 hand-written views. Notable docstring: explicitly
codifies the post-G15-A re-categorisation (the 5 are inner kernels OF
Strategy::B, NOT separate Strategy::A baselines, per `ivm-disagree-1`).

### `src/views/capability_grants.rs` (357 LOC) — View 1

Maintains `BTreeMap<Cid, BTreeSet<Cid>>` mapping entity CID → set of grant
CIDs. Watches `system:CapabilityGrant` labels (namespaced — matches
`benten_caps::grant::CAPABILITY_GRANT_LABEL`; the early-Phase-1 stub
matched bare `"CapabilityGrant"` and silently skipped every real event,
closed by `r6b-ivm-2`). Two ingress paths:
- **Node path:** prefers the Node's `grantee: Value::Bytes(cid_bytes)`
  property as entity key; falls back to event.cid for legacy identity-only
  events.
- **Edge path:** `GRANTED_TO` edges — source is grant CID, target is
  entity CID, wired into the same map.

Read API: `read_for_entity(&Cid)` (direct) plus the trait `read` which
requires `query.entity_cid` (returns `PatternMismatch` if absent —
r6b §5.5 fail-fast principle; silently empty results made queries against
non-maintained partitions look like "no grants").

### `src/views/content_listing.rs` (440 LOC) — View 3

The exit-criterion-load-bearing view. Maintains a single label's Nodes
sorted by `createdAt`, paginated. Storage: `BTreeMap<(u64, u64), Cid>`
where the composite key is `(biased_createdAt, monotonic_disambiguator)`.
Two key design decisions:
- **`u64` sort primary** (not `i64`) via the `bias_i64_to_u64` affine
  transform — fixes the `g5-ivm-10` clamp-at-i64::MAX collision. Legacy
  `tx_id` fallback flows through the same bias function so mixed streams
  order chronologically (`g5-p2-ivm-1`).
- **List semantics, not set.** Three creates with the same CID yield 3
  entries; deletes remove ALL entries with the matching CID.

Implements `read_allow_stale` non-trivially: maintains a `last_known_good:
Vec<Cid>` snapshot taken at trip-to-stale time so relaxed reads on a stale
view see the state-as-of-just-before-the-trip.

Budget arithmetic was iteratively tightened: `ivm-r6-4` made
Created/Updated consume BEFORE inserting (so the budget-tripping write
doesn't land in the index); `ivm-r6-5` mirrored that for Deleted +
propagates `try_consume` errors up to the subscriber so the
`applied += 1` counter doesn't lie about the trip case; `g5-p2-ivm-2`
matches Deleted cost to work done (`removed.max(1)` so a delete storm
backpressures via budget).

### `src/views/event_handler_dispatch.rs` (295 LOC) — View 2

Maintains `event_name → BTreeSet<handler_cid>`. Watches `SubscribesTo`
labels. Node path: extracts `subscribes_to: Value::List([Value::Text])`
property and bucketizes the handler under each event name. Edge path:
`SubscribesTo` edges bucket into a global empty-string key (`""` —
`GLOBAL_BUCKET`) because Phase-1 edge events don't carry property
payloads.

Read returns the union of the named-event bucket and the global bucket so
identity-only-legacy subscribers still resolve when the production path
partitions by name. Like View 1, `read` requires `query.event_name`
(PatternMismatch otherwise).

### `src/views/governance_inheritance.rs` (337 LOC) — View 4

Walks `GovernedBy` edges to compute transitive-closure governance rules.
Storage: `BTreeMap<Cid, Cid>` mapping child → parent (single-parent
cardinality; Phase 2 named for multi-parent extension). Depth cap
`MAX_GOVERNANCE_DEPTH = 5` per ENGINE-SPEC §8.

The `EffectiveRules` result carries TWO truncation flags: `was_truncated`
(either depth-cap OR cycle-induced stop) and `cycle_detected` (cycle stops
only). R4 triage `m5` pinned the separation — a regression that conflates
the two reasons fails the cycle test.

`ivm-r6-6` budget tightening: only charge `try_consume(1)` for events the
view actually processes (an explicit `_ =>` arm in the match skips the
consume). Before that fix, a flood of unrelated graph writes pushed this
view to Stale despite doing zero work — pure pattern-match overhead was
charging the budget.

Read on this view returns `ViewResult::Rules(BTreeMap)` (not `Cids`)
because the answer is a flat rules map projection (`depth` + `rule_count`),
not a CID list.

### `src/views/version_current.rs` (270 LOC) — View 5

Maintains `BTreeMap<u64, Cid>` mapping anchor_id → current-version CID.
Watches `NEXT_VERSION` labels. Node path: reads `node.anchor_id` (set on
version-chain Nodes per ENGINE-SPEC §6); falls back to `DEFAULT_ANCHOR_ID
= 1`. Edge path: `NEXT_VERSION` edge points from previous-head to new-head
per ENGINE-SPEC §6, so the edge's `target` is the new current.

Carries a small `AnchorRef` trait with impls for `u64`, `Cid`, `&Cid` so
`resolve(anchor)` accepts either an explicit u64 anchor id or a CID handle
(Cid handles fall through to `DEFAULT_ANCHOR_ID` — a proper Cid → anchor_id
reverse map is named for Phase 2).

`ivm-r6-7` edge-delete tightening: only rolls back `DEFAULT_ANCHOR_ID`
when the deleted edge's target matches the currently-remembered head.
Pre-r6-7 ANY `NEXT_VERSION` edge delete (including unrelated chains)
would clear the default anchor.

`ViewResult::Current(Option<Cid>)` is the read shape — unique among the 5
views.

---

## 4. Public API surface

Crate-root re-exports (`pub use`):

```
benten_ivm::Strategy                      // THE engine-boundary type
benten_ivm::View                          // trait
benten_ivm::ViewError, IvmError           // error + alias
benten_ivm::ViewState                     // Fresh / Stale
benten_ivm::ViewBudget                    // legacy budget type
benten_ivm::ViewQuery                     // un-typed query shape
benten_ivm::ViewResult                    // polymorphic result enum
benten_ivm::ViewDefinition                // content-addressed definition
benten_ivm::BudgetTracker                 // shared budget state machine
benten_ivm::Subscriber                    // fan-out hub
benten_ivm::ChangeStreamSubscriber        // alias for Subscriber
benten_ivm::Algorithm                     // alias for AlgorithmBView
benten_ivm::AlgorithmBView                // generalized kernel wrapper
benten_ivm::AlgorithmError                // register-path errors
benten_ivm::LabelPattern                  // Exact / AnchorPrefix
benten_ivm::Projection                    // AllProps (placeholder)
benten_ivm::dispatch_for                  // INTERNAL router (pub-but-internal)
benten_ivm::hardcoded_label_for_id        // canonical-label introspection
benten_ivm::is_canonical_view_id          // ditto
```

Module-paths:
- `benten_ivm::views::{CapabilityGrantsView, ContentListingView,
  EventDispatchView, EventHandlerDispatchView, GovernanceInheritanceView,
  VersionCurrentView}` — the 5 inner kernels. Engine constructs them
  directly via `XxxView::new(...)` or `with_budget_for_testing(N)`.
- `benten_ivm::algorithm_b::{Algorithm, hardcoded_label_for_id,
  is_canonical_view_id, dispatch_for, ...}` — full path the engine uses.
- `benten_ivm::testing::{testing_construct_view_with_strategy,
  try_construct_view_with_strategy, criterion_estimates_mean_ns}` — test
  surface; intentionally pub (not cfg-gated) for cross-crate consumers.

**The engine-boundary contract (CLAUDE.md item #2):** the engine consumes
`Strategy`, `Subscriber`, `ViewQuery`, `ViewResult`, `ViewError`,
`Algorithm::register`, `hardcoded_label_for_id`, `LabelPattern`,
`Projection`, and the 5 concrete `views::XxxView` types. It does NOT name
`View` algorithm internals (`update`/`rebuild` arithmetic), the
`GenericKernel` private struct, `dispatch_for`'s classification logic, or
`BudgetTracker`'s field layout. That's the asymmetry CLAUDE.md item #2
codifies.

---

## 5. Tests inventory (25 files, ~4300 LOC)

Trait + enum shapes:
- `view_trait.rs` (47 LOC) — trait method existence pin.
- `view_trait_object_safety.rs` (41 LOC) — compile-time `Box<dyn View>`
  regression pin (the `strategy()` default-method MUST not break
  object-safety).
- `strategy_enum_present.rs` (60 LOC) — `Strategy::{A,B,C}` exists + the 5
  views default to `A`.
- `strategy_c_reserved.rs` (70 LOC) — Strategy::C surfaces typed
  `E_IVM_STRATEGY_NOT_IMPLEMENTED`.
- `strategy_explicit_opt_in.rs` (72 LOC) — no auto-select, no runtime
  adaptation (D8-RESOLVED pin).
- `view_definition.rs` (59 LOC) — ViewDefinition serializes as a
  `system:IVMView` Node.
- `view_definition_cid.rs` (59 LOC) — two definitions that differ only in
  `strategy` produce different CIDs (`g8-concern-1` closure).

Subscriber + routing:
- `subscriber_routing.rs` (53 LOC) — fan-out reaches every registered view.
- `subscriber_panic_isolation.rs` (174 LOC) — a panicking view MUST NOT
  take down the whole subscriber; the load-bearing `catch_unwind(
  AssertUnwindSafe(...))` wrap. Backstop against silent regression
  (`g5-p2-ivm-3`).

Per-view correctness:
- `view1_capability_grants.rs` (198 LOC) — three-category coverage
  (build-from-scratch ≡ incremental, specific CID assertion, removal).
- `view2_event_dispatch.rs` (105 LOC) — same three categories for View 2.
- `view3_content_listing.rs` (220 LOC) — exit-criterion-load-bearing view
  (`crud('post').list`); plus a `.proptest-regressions` sibling.
- `view4_governance_inheritance.rs` (122 LOC) — depth-cap edge cases (5
  hops succeed, 6+ truncate cleanly, cycle detected separately).
- `view5_version_current.rs` (108 LOC) — three-category coverage for View 5.

Cross-cutting behavior:
- `stale_on_budget_exceeded.rs` (122 LOC) — every view trips its budget
  and surfaces `E_IVM_VIEW_STALE` on read.
- `view_read_allow_stale.rs` (89 LOC) — relaxed read returns
  last-known-good (View 3 is the witness — only view with a non-trivial
  override).
- `handwritten_views_remain_live.rs` (69 LOC) — `g8-clarity-1`: the 5
  hand-written views are NOT retired in Phase 2b; Algorithm B ships
  ADDITIVE. Retirement requires 3 named Phase-3+ conditions.
- `cascade_create_delete.rs` (264 LOC) — RESULTS.md §3 cascade contract:
  Creates flow into views, then a delete cascade converges every view to
  empty. Closes the Phase-3 marker named in `lib.rs`.

Algorithm B:
- `algorithm_b_view_correctness.rs` (388 LOC) — for each of the 5 views,
  run the same event sequence through Strategy::A AND Strategy::B (now
  inner-kernel-of-B per `ivm-disagree-1`); assert row-equivalence on every
  observable read.
- `algorithm_b_general.rs` (325 LOC) — G15-A generalization pins (arbitrary
  `(view_id, label_pattern, projection)` triples; AnchorPrefix correctness;
  fail-loud guards; canonical-fast-path-preservation gate parses bench
  JSON and asserts ratio ≤ 1.20).
- `algorithm_b_cross_replica.rs` (106 LOC) — RED-PHASE pin for G15-B +
  G16-B convergence (Algorithm B + Loro CRDT cross-replica).
- `algorithm_b_within_20pct_gate.rs` (348 LOC) — G8-A bench gate. Per-view
  thresholds from `benches/algorithm_b_thresholds.toml`.
- `algorithm_b_drift_detector.rs` (321 LOC) — G15-B proptest harness; uses
  the `Algorithm::register` lane + the budget-aware `ContentListingView`
  lane separately because `register` doesn't surface a budget knob.
- `proptest_algorithm_b_correctness.rs` (161 LOC) —
  `prop_algorithm_b_incremental_equals_rebuild` (10k cases originally;
  current corpus is calibrated per `r4-r2-ivm-7`).
- `common.rs` (760 LOC) — shared test helpers for the drift-detector +
  cross-replica pins (View materialization wrappers, structured-diff
  helpers, asymmetric path diff).

---

## 6. Benches inventory (4 files)

`benches/view_maintenance.rs` (173 LOC) — Two §14.6 gates: view-read on a
hot cache (< 1µs target) and incremental-maintenance per write (< 50µs
target). Currently `informational` because the matrix slot is one value
per file but the file mixes two case-level gates.

`benches/algorithm_b_vs_handwritten.rs` (279 LOC) — Per-view ratio gate:
Algorithm B's wallclock must be within 20% of the hand-written baseline.
Bench order is perf-risk descending per `r1-ivm-algorithm.json`
(`content_listing` first — highest gate-risk). Companion test
`algorithm_b_within_20pct_gate.rs` parses the criterion JSON.

`benches/algorithm_b_canonical.rs` (106 LOC) — G15-A canonical-view
fast-path preservation gate. Compares `AlgorithmBView::for_id`
construction path vs `Algorithm::register` (which dispatches through the
generalized kernel) — both should end up at the same hand-written inner
kernel for canonical ids, so the ratio should be ≤ 1.20. Gate is
INFORMATIONAL at G15-A landing; promoted to required at R6.

`benches/view_staleness_transition.rs` (122 LOC) — Informational. Measures
the cost of transitioning a view from healthy to Stale + the
already-stale short-circuit cost. Surface a regression if the mark-stale
path ever grows expensive.

`benches/algorithm_b_thresholds.toml` — per-view ceiling table read by
the bench gate.

---

## 7. Thin-engine + composable-graph philosophy check

IVM is the **materializer of state from event stream** — it's a close cousin
to the materializer pipeline Phase 4-Foundation / Phase 4-Meta wants to build (per
FULL-ROADMAP.md §92-98: "Schema → subgraph compiler → materializer → render
output"). Reading this crate through that lens surfaces several
architectural observations:

### 7.1 Well-respected: the canonical-view set is engine-special-cased, NOT user-registerable as Strategy::A

The 5 canonical view ids (`capability_grants`, `event_dispatch`,
`content_listing`, `governance_inheritance`, `version_current`) are
hard-coded in `CANONICAL_VIEW_IDS` and routed through their bespoke
hand-written kernels via `AlgorithmBView::for_id`. Users cannot register a
new view with `Strategy::A` — the engine refuses it
(`EngineError::ViewStrategyARefused`). User-registered views go through
`Strategy::B`'s `GenericKernel` exclusively, which only knows
`(label_pattern, projection)` triples.

This is the right shape: the 5 invariant-supporting views (I3-I7) are
genuinely engine-internal — they back capability dispatch, version-chain
CURRENT pointer resolution, governance traversal. A user view that
substituted for, say, View 1's hand-written shape would have to reproduce
the entire `system:CapabilityGrant` ingestion logic + the edge-path
GRANTED_TO logic + the engine-internal-only writes that fill `grantee`
properties. Forcing all user views through the generic kernel keeps the
"user views are read-side projections, never engine-internals" line clean.

### 7.2 Well-respected: the `Strategy` enum is the only IVM type the engine names in its public API

CLAUDE.md item #2 specifically codifies this: "the engine names
`benten_ivm::Strategy` as the dispatch type but no `View` / algorithm
internals leak through". Grepping `benten_ivm::` across the engine crate
confirms: the engine consumes `Strategy`, `Subscriber`, `View`,
`ViewQuery`, `ViewResult`, `ViewError`, `ViewDefinition`, `Algorithm`,
`LabelPattern`, `Projection`, and the 5 concrete view types. None of
those leak `View`-internal algorithm logic — `View` is a trait the engine
holds by `Box<dyn View>` via the subscriber and never calls the
arithmetic-bearing methods directly.

### 7.3 Caveat: `dispatch_for` and `is_canonical_view_id` are `pub` but documented as INTERNAL

```rust
/// INTERNAL Strategy::A vs Strategy::B dispatch router.
///
/// Per `D8-RESOLVED` the router is INTERNAL: callers do not pick the
/// strategy at the engine boundary; user-view registration always runs
/// under Strategy::B (the engine refuses Strategy::A user-view
/// registration per `ivm-major-5`).
pub fn dispatch_for(view_id: &str) -> Strategy { ... }
```

The doc-comment carries the intent but the visibility doesn't enforce it.
The engine boundary holds because the only call site outside the crate
is the engine's own `register_user_view`, which uses the helper to
validate "is this user trying to register a canonical id?". A future
re-export sweep could narrow `dispatch_for` to `pub(crate)` once the
engine's reliance is fully audited; for now it's an honor-system
boundary.

### 7.4 Generality gap for Algorithm B — projections are placeholder

`Projection::AllProps` is the only variant. The kernel applies the
identity transform on every matched Node. Real projections (a `Computed`
variant carrying a small expression tree; a `PropSubset` variant carrying
a property allow-list; a `Reshape` variant for emitting non-Node
structures) lift later. This is named in the docstring on `Projection` —
"future shape narrowing lifts to a richer enum without breaking the
kernel surface". The current shape is fine as a placeholder because the 5
canonical views all have their own bespoke projection logic in the
hand-written kernels (View 4 emits `Rules`, View 5 emits `Current`),
which sidesteps the `Projection` enum entirely.

### 7.5 Generality gap — first-label-only matching

`GenericKernel::first_label_matches` and View 3's `update` arm both look
at `node.labels.first()` and test the pattern against that. Multi-label
Nodes get their secondary labels ignored. The `algorithm_b.rs` docstring
explicitly names this — "Matchers that need multi-label semantics belong
at a higher selector layer (named in `docs/future/phase-3-backlog.md`
§5.1-followup-b for edge-traversal-keyed views)". Right disposition: don't
expand the kernel surface to handle multi-label until there's a user view
that needs it; defer to the named backlog item.

### 7.6 Generality gap — Phase 1 `rebuild()` doesn't actually replay events

Every view's `rebuild()` clears state and resets the budget. None of them
replay the change-event log to reconstruct the view from scratch. The
`lib.rs` TODO acknowledges this:

> 4 rebuild-equivalence tests in view1/2/3/5 are R3 defects — they
> construct an empty rebuilt view and assert equality with a populated
> incremental one. Fixing requires event-replay, beyond the Phase-2
> fix-pass scope.

This is a real correctness gap: a post-stale rebuild produces an empty
view, not a view recovered from the persisted change-event log. In
practice the engine's current usage doesn't hit this because the stale
trip happens only in adversarial / budget-misconfigured cases. But the
"strict reads refuse stale; rebuild restores" contract that callers
mentally model is half-truthy — rebuild restores Fresh state, but not
the data the view had before tripping. The named Phase-3+ event-replay
work closes this.

### 7.7 Generality gap — `ViewQuery` is one un-typed record

```rust
pub struct ViewQuery {
    pub label: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub anchor_id: Option<u64>,
    pub entity_cid: Option<Cid>,
    pub event_name: Option<String>,
}
```

Every view ignores most of these fields. View 1 needs `entity_cid`. View 2
needs `event_name`. View 3 honors `label`/`limit`/`offset`. View 4 needs
`entity_cid`. View 5 needs `anchor_id`. Pattern mismatches surface as
`E_IVM_PATTERN_MISMATCH`. The docstring names a "typed-per-view variant
in Phase 2 once the views themselves stabilize" — but views ARE stable now,
and the un-typed shape is a known fragility. A user view registered via
`Algorithm::register` (which always returns `ViewResult::Cids` and ignores
`query` entirely in `GenericKernel::read`) means the typing question is
even more lopsided: the 5 canonical views demand specific fields; the
generic kernel ignores all of them.

### 7.8 Caveat: `Subscriber::on_change` uses `eprintln!` for non-fatal errors

The subscriber logs `PatternMismatch` and view-panic events via
`eprintln!` (with `#[allow(clippy::print_stderr)]` markers naming the
Phase-2 `tracing` migration). Three Phase-3 markers in lib.rs and
subscriber.rs name this. The named tracking is clean; the work itself is
deferred. Engine-level observability is wired through metric snapshots
(`benten.ivm.view_stale_count` reads `stale_count_tally`), so the
operator-facing surface isn't dependent on the stderr sink.

### 7.9 Caveat: Phase-1 budget arithmetic is per-view local, not subscriber-wide

Each view holds its own `BudgetTracker`; nothing globally throttles
across-view work. A `Subscriber` with 50 views fans every event to all 50
without per-event aggregate cost-control. Phase 1 ships this knowingly (5
views; load is trivially bounded by 5x per-event work). When user views
proliferate, the named "TODO(phase-3 — pattern-based pre-filtering
router)" in `subscriber.rs` becomes load-bearing.

---

## 8. Phase 4-Foundation + Phase 4-Meta expectations

The Phase 4-Foundation materializer pipeline (per `docs/FULL-ROADMAP.md`: "Schema →
subgraph compiler → materializer → render output") will be parallel to IVM
in spirit. The architectural question is **whether the materializer
composes USING IVM views, or is a sibling system.** Three sketches:

### 8.1 Sketch A — Materializer is built on top of IVM views

A schema (content-type definition) compiles into a `ViewDefinition` (or
several). The materializer registers those as user views via
`Algorithm::register` and walks the change stream into them. Reads from
the materializer are reads from the IVM view. The materializer is "the
thing that turns schemas into view registrations + ViewResult →
render-output projections".

Pros: reuses every piece of this crate. The `GenericKernel`'s
`(label_pattern, projection)` shape already covers a non-trivial slice of
"all Nodes with label X projected through transform Y". `BudgetTracker`'s
stale-with-last-known-good is exactly the contract a render layer wants
("on failure, show the last good page"). Subscriber fan-out gives
multi-view composition for free.

Cons: `Projection::AllProps` is the only variant — real schema-driven
projections (joins across labels, edge-traversal-keyed views, computed
fields, reshape to non-Node output) ALL need `Projection` to be a much
richer enum than today. This is named in §7.4 above; the materializer
would force the issue.

### 8.2 Sketch B — Materializer is a sibling subscriber

The materializer is its own `impl ChangeSubscriber` (parallel to the IVM
`Subscriber`), holding its own state shapes (render trees, page caches,
schema-driven nested structures) that don't fit the `ViewResult` enum's
`Cids / Current / Rules` shape. The IVM crate remains the
"five-engine-internals views + arbitrary user `(label_pattern, projection)`
indices" — bounded scope, bounded API.

Pros: clean separation of concerns. Materializer is free to be as
expressive as Phase 4 needs without contorting the IVM trait. The
"materializer registers views THROUGH ivm and reads their output" pattern
is still available where it fits.

Cons: code duplication. Both subscribers would re-implement panic
isolation, budget tracking, fan-out, stale-with-last-known-good. The
existing `Subscriber` could be lifted to a generic shape that both
materializers and IVM views consume, but that's a non-trivial refactor.

### 8.3 Sketch C — Materializer uses the engine evaluator + handler subgraphs

CLAUDE.md #18 + the plugin trust model frame plugins as **subgraphs of
the engine's own operation primitives**. The materializer might similarly
be a subgraph (or set of subgraphs) that the evaluator walks on every
write — a TRANSFORM node consuming the changed Node, an EMIT node
broadcasting the rendered output, etc. IVM views are then orthogonal:
the materializer subgraph could read FROM an IVM view (`benten_ivm::
Subscriber::read_view`) as one of its inputs, but the materializer
itself doesn't live in this crate.

Pros: maximally aligned with the "code-as-graph" deep differentiator
(CLAUDE.md baked-in #3). New materializer logic ships as new
subgraphs/handlers, not new Rust crates. Plugin trust model trivially
extends to materializers.

Cons: makes the materializer's compose-time wiring more complex; the
schema-compiler has to emit subgraphs, not just `ViewDefinition`s. Read
latency might be higher (evaluator walk per read vs `BTreeMap::range` on
the view's pre-materialized index).

### 8.4 The question worth surfacing

The IVM crate is well-positioned to be a building block of the
materializer (sketch A's incremental-index half) but the Phase 4
materializer scope clearly exceeds what `Projection::AllProps`
expresses. Whether the materializer LIVES as an extension to this crate
(views grow more expressive projections) or as a SIBLING subscriber
(new crate, parallel to `benten-ivm`) is a real fork that hasn't been
codified anywhere in the docs or memory.

Sketch C lurks as the radical option — materializers are subgraphs the
evaluator walks. That would unify materialization with handlers + plugins
under "code-as-graph", but it also re-opens the question of what the
incremental-index layer is FOR if the evaluator can recompute on demand.

A Phase 4-Foundation / Phase 4-Meta pre-R1 question for Ben: does the materializer
COMPOSE USING IVM (extending `Projection`), live ALONGSIDE IVM (parallel
subscriber), or REPLACE IVM (subgraphs replace incremental indices)?

---

## 9. Open questions / unresolved internals

- **Phase-1 `rebuild()` doesn't replay events.** Documented in §7.6 above
  via the `lib.rs` TODO. Real correctness gap for the stale-then-rebuild
  contract; closure is named for Phase 3+ event-replay work.

- **`ViewQuery` is un-typed and over-broad.** Documented in §7.7. Six
  optional fields; every view ignores most. The "typed-per-view variant"
  named in the docstring hasn't landed despite views being stable since
  Phase 1.

- **`dispatch_for` + `is_canonical_view_id` are `pub` but documented
  INTERNAL.** Documented in §7.3. Honor-system boundary; could be
  narrowed to `pub(crate)` after a re-export sweep.

- **`Strategy::C` is reserved-not-implemented forever-deferred.** No
  Phase target ever named beyond the vague `"Phase 3+"`. Z-set / DBSP
  cancellation is a real algorithmic family the variant gestures at, but
  the value of pre-reserving a variant for an algorithm we haven't
  designed is more API-stability-theater than concrete forward planning.
  Either commit to a phase for `Strategy::C` or rename the variant to
  something less load-bearing (e.g. `Strategy::Reserved` with a string
  payload).

- **Subscriber pattern-based pre-filtering is named Phase-3 TODO and
  hasn't moved.** Every view sees every event with internal filter. Fine
  at 5 + a handful of user views; at "Phase 4-Foundation materializer registers 50
  schemas → 50 views" scale this becomes a problem. The TODO has been
  carried since Phase 2.

- **`Subscriber::on_change` uses `eprintln!` for non-fatal errors.**
  Documented in §7.8. `tracing` migration named since Phase 1.

- **`AlgorithmBView::for_id_with_budget` for `content_listing` with a
  non-`"post"` label silently drops the budget.** Lines 565-572 of
  `algorithm_b.rs`: when the supplied label isn't `"post"`, the
  constructor calls `ContentListingView::new(label)` instead of
  `with_budget_for_testing(budget)`, observably losing the supplied
  budget. The docstring names the gap and points at
  `phase-3-backlog.md §5.1-followup-e residual`. The canonical
  constructor needs to be lifted to accept `(label, budget)` together.

- **`Projection` has one variant.** Documented in §7.4. The kernel surface
  is ready for richer projections but the enum is placeholder. First user
  view that needs anything non-identity will force the lift.

- **`first_label_matches` ignores secondary labels.** Documented in §7.5.
  Right disposition (defer to named backlog item §5.1-followup-b); worth
  surfacing if a user-view registration in Phase 4 needs multi-label
  matching.

- **Architectural fork: materializer composition vs sibling.** Documented
  in §8 above. Not codified anywhere in docs or memory; worth a
  pre-Phase-4 surface to Ben.

- **`BudgetTracker` charges 1 unit per event regardless of work.** Views
  3 (deletes, where `removed.max(1)` is charged) and 2 (deletes,
  proportional to buckets touched) tightened this; views 1/4/5 still
  charge a flat 1 per event. Likely fine — read paths are not budgeted —
  but a "delete storm against capability_grants causes every grant to
  scan the full map" pattern still charges 1 unit for what is O(n) work.
  Documented inline in the Phase-2 fix-pass notes (`g5-p2-ivm-2`).
