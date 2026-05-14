# `benten-ivm` — Internals deep-dive

Read-only audit doc. Plain-English walk through what this crate is, what it
contains, what it exposes, and what gaps / architectural questions it leaves
open. Pair with `CLAUDE.md` item #2 (IVM Algorithm B + per-view strategy + the
"engine names `Strategy` but not internals" boundary).

State: HEAD `c589ffe` (post Phase-4-Foundation R6-FP-4 close; tag
`phase-4-foundation-close`). Reflects G23-0a IVM kernel generalization +
G23-0b 5-view re-expression as `SubgraphSpec` consumers + `Strategy::C →
Strategy::Reserved` rename + `Projection::AllProps` placeholder removal.

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

Phase-3 generalized the kernel (G15-A): `Strategy::B` is a **single generic
algorithm** that handles arbitrary `(view_id, label_pattern, projection)`
triples for user-defined views, while transparently routing canonical view ids
through the 5 hand-written inner kernels as a fast path (per `ivm-disagree-1`
the 5 hand-written views are inner kernels OF Strategy::B, NOT separate
Strategy::A baselines — even though they each still `return Strategy::A` from
the inner-kernel `View::strategy()` for back-compat reporting; the
engine-boundary `AlgorithmBView::strategy()` always returns `Strategy::B`).

Phase-4-Foundation generalized the kernel further (G23-0a + G23-0b):
`Strategy::B` now consumes a [`SubgraphSpec`](src/subgraph_spec.rs:104)
schema-shaped view definition through
[`AlgorithmBView::register_subgraph`](src/algorithm_b.rs:936) in addition to
the G15-A triple-form surface. No new `Strategy` variant was minted — per
CLAUDE.md baked-in #2 + arch-r1-14, `Strategy::B` IS the generalized
Algorithm B; the SubgraphSpec is the kernel's input shape, not a new
algorithm. G23-0b re-expresses the 5 canonical hand-written views as
SubgraphSpec consumers via `SubgraphSpec::for_canonical_view`. The
[`Projection`](src/algorithm_b.rs:231) enum was demoted to a unit struct at
G23-0b (CRATES-DEEP-DIVE §4 closure) — the prior `AllProps` identity-variant
placeholder is gone; View 4 + View 5 typed-output shapes are declared via the
NEW [`TypedOutputProjection`](src/subgraph_spec.rs:83) enum.

The Phase-3 `Strategy::C` (Z-set / DBSP) variant was renamed to
[`Strategy::Reserved`](src/strategy.rs:66) at G23-0a per arch-r1-14 — closes
CRATES-DEEP-DIVE §4 named-but-deferred item by surfacing the
reserved-not-implemented state in the variant name itself.

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
  `ViewStrategyReservedRefused` (post-rename).
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

### `src/lib.rs` (82 LOC)

Crate root. Declares `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`,
imports `extern crate alloc;` (the crate is no_std-friendly in its core
types — only `subscriber.rs` reaches for `std::sync::Mutex` and `std::panic`).
Module tree: `algorithm_b`, `budget`, `strategy`, `subgraph_spec` (NEW G23-0a),
`subscriber`, `testing`, `view`, `views`. Re-exports the public surface at the
crate root.

Carries three Phase-3 `TODO`s naming work that's known-missing: dedicated
Criterion benchmarks per view against RESULTS.md §1 targets, the cascade
create→delete integration test (closed by `tests/cascade_create_delete.rs`),
and the rebuild-equivalence event-replay path (still open — Phase 1's
"rebuild" doesn't actually replay events, it just clears state; see §9).

### `src/strategy.rs` (67 LOC)

Defines the `Strategy` enum — `{ A, B, Reserved }`, closed (no
`#[non_exhaustive]`), `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`.
This is **the load-bearing engine-boundary type**: the one type from
`benten-ivm` the engine names in its own public API (`Engine::view_strategy`,
`EngineError::ViewStrategyARefused`, etc).

Variants:
- `A` — hand-written incremental maintenance. The 5 Phase-1 views each return
  `Strategy::A` from `View::strategy()`. This is the **canonical fast-path
  classification** at the dispatch level (see `algorithm_b::dispatch_for`).
  Reserved for engine-internal canonical-view shapes per arch-r1-14; the
  engine refuses Strategy::A user-view registration.
- `B` — generalized Algorithm B. Implemented by
  [`AlgorithmBView`](src/algorithm_b.rs:517). `Strategy::B` IS the generalized
  Algorithm B per CLAUDE.md baked-in #2; G23-0a generalizes the kernel to
  consume a `SubgraphSpec` without minting a new variant.
- `Reserved` — Z-set / DBSP cancellation. **Reserved-not-implemented.**
  Renamed from `Strategy::C` at G23-0a per arch-r1-14 (closes
  CRATES-DEEP-DIVE §4 named-but-deferred item by surfacing the
  reserved-not-implemented state in the variant name itself). Constructing
  one via `testing::try_construct_view_with_strategy` returns
  `ViewError::StrategyNotImplemented`. The
  `tests/strategy_c_renamed_to_reserved_grep_assert.rs` test pins that no
  `Strategy::C` references remain in source.

D8-RESOLVED is explicit: strategy is fixed at construction time, no
auto-select, no runtime adaptation, no `set_strategy` companion. Future
algorithms would land as a new enum (`StrategyV2`), not a new variant on this
one — the closed set is part of the API contract.

### `src/subgraph_spec.rs` (300 LOC) — NEW at G23-0a

Schema-shaped view-definition input for the generalized kernel. The G15-A
`(view_id, label_pattern, projection)` triple is now a *special case* of
[`SubgraphSpec`](src/subgraph_spec.rs:104).

- [`SubgraphSpec`](src/subgraph_spec.rs:104) — load-bearing record with
  `view_id`, `label_pattern`, `projection`, `typed_output_projection`,
  `self_referential`, `budget`. Canary stability contract: subsequent
  iterations MAY add fields (additive) but MUST NOT remove or rename per
  G23-0b consumer pins.
- [`SubgraphSpec::for_canonical_view(view_id)`](src/subgraph_spec.rs:150) —
  constructor for the 5 canonical ids; bakes the matching hardcoded label
  + `typed_output_projection` into the spec.
- [`SubgraphSpec::user_view(view_id, label_pattern)`](src/subgraph_spec.rs:198)
  — constructor for user-defined ids; rejects canonical ids (use
  `for_canonical_view` instead).
- Builder helpers: `with_self_reference`, `with_budget`, `with_label_pattern`.
- [`TypedOutputProjection`](src/subgraph_spec.rs:83) `{ Rules, Current }` —
  declared output shape per `mat-r1-1`. View 4 ⇒ Rules; View 5 ⇒ Current;
  every other view ⇒ `None`. Validated at register time; mismatch surfaces
  `AlgorithmError::TypedOutputProjectionMismatch`.
- [`KernelInput`](src/subgraph_spec.rs:259) `{ label, created_at,
  disambiguator }` — a single write the kernel consumes via `walk_writes`.
  Mirrors `benten_graph::ChangeEvent` minus the cross-crate dep G23-0b's
  Family B/C consumer pins don't need.
- [`KernelOutput`](src/subgraph_spec.rs:293) `{ Rows, Rules, Current }` —
  post-walk materialised output. Discriminants match the three
  [`ViewResult`](src/view.rs) variants the inner kernels emit. Canonical
  bytes via lexicographic CID sort + `\n` separator for determinism.
- [`CANONICAL_VIEW_IDS`](src/subgraph_spec.rs:68) — re-stated in this module
  so `SubgraphSpec::for_canonical_view`'s allowed-id contract is local.

D-4F-2 codification in the module docstring: the materializer view IS an IVM
view. The materializer pipeline (G23-B wave-5) registers its views through
this same Algorithm B kernel via `Algorithm::register_subgraph(spec)`. The
materializer's `Renderer` trait is the host-side output transform; the
kernel itself doesn't know it's serving a materializer.

mat-r1-13 self-reference rejection: the `self_referential` flag is inspected
at register-time BEFORE any walk — fail-fast semantics preclude partial
materialisation or walk-time-only checks. A future richer cycle-detection
pass lifts behind the same flag without breaking the canary contract.

### `src/view.rs` (430 LOC)

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
  `BudgetExceeded`, `StrategyNotImplemented { strategy, deferred_to_phase }`.
  Each variant has a stable `ErrorCode` mapping via `ViewError::code()`.
  `IvmError` is a back-compat type alias for `ViewError` because some R3
  tests named the type `IvmError`. The `StrategyNotImplemented` variant
  now carries `Strategy::Reserved` (post-rename) with
  `deferred_to_phase = "Phase 3+"`.
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
  Typed-per-view variant is named as Phase-4-Meta+ work.
- `ViewResult` — polymorphic enum: `Cids(Vec<Cid>)` for list-shape views,
  `Current(Option<Cid>)` for view 5 (version pointer), `Rules(BTreeMap)` for
  view 4 (governance rules).
- `ViewDefinition` — content-addressed; serializes as a Node with label
  `system:IVMView` carrying `view_id`, optional `input_pattern_label`, and a
  `strategy` property (stringified `"A" / "B" / "Reserved"`). G8-A's
  `g8-concern-1` closure: the `strategy` field is folded into the CID input
  so two definitions that differ only in strategy don't content-hash
  collide.

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
all hold a `BudgetTracker` by value, as does the generic kernel.

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

### `src/algorithm_b.rs` (1678 LOC, largest file)

The G15-A generalized kernel + G23-0a/G23-0b SubgraphSpec surface +
`AlgorithmBView` wrapper + internal dispatch router. Five load-bearing
concepts:

**(a) Canonical-id dispatch router.**
[`dispatch_for(view_id) -> Strategy`](src/algorithm_b.rs:155) classifies a
view-id into a strategy lane: canonical ids (the 5 hard-coded strings) →
`Strategy::A` (canonical fast-path classification); everything else →
`Strategy::B` (generic kernel lane). This router is `pub` but is **INTERNAL**
in the architectural sense — the engine refuses `Strategy::A` user-view
registration (per `ivm-major-5`), so callers don't pick this. The hand-written
5 views are not registered through user-view; they're inner kernels invoked
by `AlgorithmBView::for_id` when the dispatch table maps to them.

**(b) `LabelPattern` + `Projection`.**
[`LabelPattern::{Exact, AnchorPrefix}`](src/algorithm_b.rs:171) is the selector
surface user views consume. `Exact("post")` matches Nodes whose FIRST label
equals `"post"`; `AnchorPrefix("crud:")` matches Nodes whose first label
STARTS WITH `"crud:"`. Phase-2b's stub silently coerced `AnchorPrefix` to
label equality — G15-A wave-5a fixed that.
[`Projection`](src/algorithm_b.rs:231) is **a unit struct at G23-0b** (per
CRATES-DEEP-DIVE §4 closure — the prior `AllProps` enum-variant placeholder
was removed; `tests/projection_all_props_placeholder_removed_no_remaining_references.rs`
pins the removal). Identity transform on every matched Node. Real
projections (`PropSubset`, `Computed`) lift to an enum behind the same call
signature; View 4 + View 5 typed-output shapes now declared via
[`TypedOutputProjection`](src/subgraph_spec.rs:83) on `SubgraphSpec` (the
load-bearing typed-output declaration per `mat-r1-1`).

**(c) `AlgorithmBView` + `GenericKernel`.**
[`AlgorithmBView`](src/algorithm_b.rs:517) is the public wrapper consumers
see. Its `View::strategy()` always returns `Strategy::B` regardless of which
inner kernel it holds — "the wrapper IS Strategy::B". Inner kernel is one of:
- For canonical view ids: a `Box<dyn View>` holding one of the 5 hand-written
  views (via `for_id` or `for_id_with_budget`).
- For user-defined view ids: a `GenericKernel` (private struct) that holds
  a `BTreeSet<Cid>` of matched Node CIDs, plus a `BudgetTracker`. The
  kernel's `update` path: charge budget once per matching write, add the
  CID to the set on Created/Updated, remove on Deleted (only charge for
  observable deletes — deletes against never-admitted CIDs are free).

The wrapper carries four new G23-0b fields:
- [`typed_output_projection`](src/algorithm_b.rs:542) — declared output
  shape (`None` / `Some(Rules)` / `Some(Current)`); load-bearing at
  materialise time per `mat-r1-1` + `g23-0a-mr-3`.
- [`label_pattern`](src/algorithm_b.rs:549) — stored on the wrapper for
  walk-time label-match observable computation (works uniformly for all 5
  canonical inner kernels regardless of `ViewQuery` requirements).
- [`walk_observable: BTreeSet<Cid>`](src/algorithm_b.rs:559) — walk-time
  observable populated by `walk_writes`; stable across two registration
  paths (G15-A `register` + G23-0a `register_subgraph`) sharing the same
  `label_pattern`. The canonical canary observable that Family B/C
  round-trip pins compare.
- Wrapper construction goes through the private
  [`AlgorithmBView::assemble`](src/algorithm_b.rs:596) helper — single
  source of truth for initialising the four new fields.

**(d) Public construction surface — five entrypoints + try-alias:**
- [`for_id` / `for_id_with_budget`](src/algorithm_b.rs:626) —
  canonical-only legacy entry; errors with `PatternMismatch` for unknown
  view ids. Used by the engine's hand-written canonical-view construction
  paths.
- [`register` / `register_with_budget`](src/algorithm_b.rs:749) — G15-A
  triple-form surface. Accepts either canonical or user-defined ids,
  routes through the dispatch table. Fails loud with
  `AlgorithmError::ViewLabelMismatch` (canonical id + exact label that
  disagrees with the canonical hardcoded label) or
  `AlgorithmError::CanonicalIdAnchorPrefixRefused` (canonical id +
  `AnchorPrefix` — the canonical kernels ignore the supplied pattern, so
  admitting a prefix selector would be a doc-vs-code-strength gap; closed
  by `g15a-mr-minor-4` / W9-T1).
- [`register_subgraph(spec)`](src/algorithm_b.rs:936) — **G23-0a NEW**
  schema-shaped entry. Takes a `SubgraphSpec` value; runs three fail-fast
  guards in order: (1) `self_referential` rejection per `mat-r1-13`
  BEFORE any kernel input walk; (2) `typed_output_projection` match
  against `canonical_typed_output_projection_for(view_id)` per `mat-r1-1`
  + `g23-0a-mr-3`; (3) delegate to `register_with_budget` /
  `register` which apply the canonical-id-vs-mismatched-label +
  canonical-id-anchor-prefix-refused guards.
- [`try_register`](src/algorithm_b.rs:890) — symmetric alias for
  `register`.

**(e) Walk + materialize surface (G23-0a/G23-0b NEW):**
- [`walk_writes(&mut self, writes: &[KernelInput]) -> Result<KernelOutput,
  ViewError>`](src/algorithm_b.rs:985) — drives a sequence of
  `KernelInput` records through the per-event `View::update` path. Each
  input is converted to a transient `ChangeEvent { kind: Created, ... }`
  with a content-addressed Cid. The wrapper's `walk_observable` BTreeSet
  captures the CIDs of inputs whose first label matches the wrapper's
  `label_pattern` — INDEPENDENT of inner-kernel admission (so the 4 of 5
  canonical inner kernels whose `read` paths require a populated
  `ViewQuery` still produce a stable canary observable).
- [`materialize() -> KernelOutput`](src/algorithm_b.rs:1073) — emits the
  post-walk view state. Discriminant selected per the declared
  `typed_output_projection`: `None` ⇒ `Rows(canonical_bytes)`;
  `Some(Rules)` ⇒ `Rules(canonical_bytes)`; `Some(Current)` ⇒
  `Current(opt_bytes)` (None when walk_observable empty). Bytes via
  lexicographic CID-string sort + `\n` separators for determinism.
  Fail-loud assert at materialise: when the inner kernel's `read`
  succeeds with a variant that doesn't match the declared
  `typed_output_projection`, panics with the programmer-error context
  (the register-time `TypedOutputProjectionMismatch` gate prevents this
  in practice).
- [`materialize_full(&self) -> Vec<Cid>`](src/algorithm_b.rs:904) —
  unfiltered materialization the per-row READ gate
  (`crates/benten-engine/src/ivm_view_read_gate.rs`) then row-filters
  against the actor cap-set.

**(f) `AlgorithmError` variants (4 total, 2 NEW at G23-0a/G23-0b):**
- [`ViewLabelMismatch`](src/algorithm_b.rs:261) — canonical id + label that
  disagrees with hardcoded.
- [`CanonicalIdAnchorPrefixRefused`](src/algorithm_b.rs:280) — canonical id
  + `AnchorPrefix` selector.
- [`SelfReferentialSubgraphRejected`](src/algorithm_b.rs:297) — **NEW
  G23-0a** per `mat-r1-13` fail-fast.
- [`TypedOutputProjectionMismatch`](src/algorithm_b.rs:313) — **NEW
  G23-0b** per `mat-r1-1` + `g23-0a-mr-3`: declared typed-output projection
  doesn't match what the view-id requires.

**(g) Public helpers:**
- [`is_canonical_view_id(view_id) -> bool`](src/algorithm_b.rs:133)
- [`hardcoded_label_for_id(view_id) -> Option<&'static str>`](src/algorithm_b.rs:102)
  — returns the hardcoded label for 4 of the 5 canonical views (every one
  except `content_listing`, whose arm honors caller-supplied label). Used
  by the engine's `register_user_view` to surface
  `EngineError::ViewLabelMismatch` (catalog code `E_VIEW_LABEL_MISMATCH`)
  on disagreement.
- [`canonical_typed_output_projection_for(view_id) ->
  Option<TypedOutputProjection>`](src/algorithm_b.rs:117) — **NEW G23-0b**
  derives expected typed-output projection a canonical view emits.
  Mirrors `SubgraphSpec::for_canonical_view`'s declarations; consulted
  by `register_subgraph`'s validate-at-register gate.

`Algorithm` is a public type alias for `AlgorithmBView` matching the
`benten_ivm::algorithm_b::Algorithm::register(...)` shape the test pins use.

The file also has ~600 lines of in-file unit tests covering dispatch,
fail-loud guards, budget-trip behavior, rebuild-after-stale, generic
kernel's label-matching semantics, plus the G23-0a/G23-0b
SubgraphSpec entry + walk_writes + materialize + typed-output-projection
gate cases.

### `src/testing.rs` (118 LOC)

Test/dev-only helpers, exposed publicly (not `#[cfg(test)]`) because
consumer crates' integration tests reach into them.
- [`testing_construct_view_with_strategy(strategy) -> Box<dyn View>`](src/testing.rs:31)
  — picks one default view per strategy. Panics on `Strategy::Reserved`.
- [`try_construct_view_with_strategy(strategy) -> Result<Box<dyn View>,
  ViewError>`](src/testing.rs:46) — same, but surfaces the typed
  `StrategyNotImplemented { strategy: Strategy::Reserved, deferred_to_phase:
  "Phase 3+" }` for `Strategy::Reserved` instead of panicking.
- [`criterion_estimates_mean_ns(group, view, axis, value)`](src/testing.rs:80)
  — reads a Criterion `estimates.json` and returns the
  `mean.point_estimate` in nanoseconds. Centralized parser for the G8-A
  bench gate. Touches `std::fs`, so only available outside `no_std`
  builds.

### `src/views/mod.rs` (48 LOC)

Re-export hub for the 5 hand-written views. Notable docstring: explicitly
codifies the post-G15-A re-categorisation (the 5 are inner kernels OF
Strategy::B, NOT separate Strategy::A baselines, per `ivm-disagree-1`).
The G15-A canonical fast-path-preservation gate measures wallclock against
a **Strategy::B baseline**, NOT against a separate Strategy::A handwritten
baseline.

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

G23-0b round-trip pin at `tests/view_1_capability_grants_subgraph_spec_round_trip.rs`
asserts this view's wrapper, registered via `SubgraphSpec::for_canonical_view`,
materialises bytes-equivalent to the G15-A `Algorithm::register` path
under the same write sequence.

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
view see the state-as-of-just-before-the-trip. The
`view_3_stale_with_last_known_good_does_not_generalize_trivially_named_carry.rs`
test pin acknowledges this is a per-view feature that does NOT auto-lift
to the generic kernel.

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
not a CID list. G23-0b declares
`TypedOutputProjection::Rules` for this view via
`SubgraphSpec::for_canonical_view("governance_inheritance")`; the
`view_4_typed_output_projection_shape_pin.rs` pin asserts the declared
shape matches the inner-kernel emission.

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
views. G23-0b declares `TypedOutputProjection::Current` for this view;
the `view_5_typed_output_projection_shape_pin.rs` pin asserts the declared
shape matches the inner-kernel emission, including the `None` vs
`Some(_)` discrimination on an empty vs populated walk.

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
benten_ivm::AlgorithmError                // register-path errors (4 variants)
benten_ivm::LabelPattern                  // Exact / AnchorPrefix
benten_ivm::Projection                    // unit struct (placeholder removed G23-0b)
benten_ivm::SubgraphSpec                  // NEW G23-0a — schema-shaped view def
benten_ivm::TypedOutputProjection         // NEW G23-0b — Rules / Current
benten_ivm::KernelInput                   // NEW G23-0a — walk_writes input
benten_ivm::KernelOutput                  // NEW G23-0a — Rows / Rules / Current
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
  is_canonical_view_id, dispatch_for,
  canonical_typed_output_projection_for, ...}` — full path the engine uses.
- `benten_ivm::subgraph_spec::{SubgraphSpec, TypedOutputProjection,
  KernelInput, KernelOutput, CANONICAL_VIEW_IDS}` — G23-0a SubgraphSpec
  module.
- `benten_ivm::testing::{testing_construct_view_with_strategy,
  try_construct_view_with_strategy, criterion_estimates_mean_ns}` — test
  surface; intentionally pub (not cfg-gated) for cross-crate consumers.

**The engine-boundary contract (CLAUDE.md item #2):** the engine consumes
`Strategy`, `Subscriber`, `ViewQuery`, `ViewResult`, `ViewError`,
`Algorithm::register`, `Algorithm::register_subgraph` (post-G23-0a),
`hardcoded_label_for_id`, `LabelPattern`, `Projection`, `SubgraphSpec`,
`TypedOutputProjection`, and the 5 concrete `views::XxxView` types. It
does NOT name `View` algorithm internals (`update`/`rebuild` arithmetic),
the `GenericKernel` private struct, `dispatch_for`'s classification logic,
or `BudgetTracker`'s field layout. That's the asymmetry CLAUDE.md item #2
codifies.

---

## 5. Tests inventory (40+ files)

Trait + enum shapes:
- `view_trait.rs` — trait method existence pin.
- `view_trait_object_safety.rs` — compile-time `Box<dyn View>` regression
  pin (the `strategy()` default-method MUST not break object-safety).
- `strategy_enum_present.rs` — `Strategy::{A,B,Reserved}` exists + the 5
  views default to `A`.
- `strategy_c_reserved.rs` — Strategy::Reserved surfaces typed
  `E_IVM_STRATEGY_NOT_IMPLEMENTED`.
- `strategy_c_renamed_to_reserved_grep_assert.rs` — **NEW G23-0a** asserts
  no `Strategy::C` references remain in source (arch-r1-14 rename
  enforcement).
- `strategy_explicit_opt_in.rs` — no auto-select, no runtime adaptation
  (D8-RESOLVED pin).
- `view_definition.rs` — ViewDefinition serializes as a `system:IVMView`
  Node.
- `view_definition_cid.rs` — two definitions that differ only in
  `strategy` produce different CIDs (`g8-concern-1` closure).

Subscriber + routing:
- `subscriber_routing.rs` — fan-out reaches every registered view.
- `subscriber_panic_isolation.rs` — a panicking view MUST NOT take down
  the whole subscriber; the load-bearing `catch_unwind(
  AssertUnwindSafe(...))` wrap. Backstop against silent regression
  (`g5-p2-ivm-3`).

Per-view correctness:
- `view1_capability_grants.rs` — three-category coverage
  (build-from-scratch ≡ incremental, specific CID assertion, removal).
- `view2_event_dispatch.rs` — same three categories for View 2.
- `view3_content_listing.rs` — exit-criterion-load-bearing view
  (`crud('post').list`); plus a `.proptest-regressions` sibling.
- `view4_governance_inheritance.rs` — depth-cap edge cases (5 hops
  succeed, 6+ truncate cleanly, cycle detected separately).
- `view5_version_current.rs` — three-category coverage for View 5.

Cross-cutting behavior:
- `stale_on_budget_exceeded.rs` — every view trips its budget and surfaces
  `E_IVM_VIEW_STALE` on read.
- `view_read_allow_stale.rs` — relaxed read returns last-known-good (View
  3 is the witness — only view with a non-trivial override).
- `handwritten_views_remain_live.rs` — `g8-clarity-1`: the 5 hand-written
  views are NOT retired in Phase 2b; Algorithm B ships ADDITIVE.
  Retirement requires 3 named Phase-3+ conditions.
- `cascade_create_delete.rs` — RESULTS.md §3 cascade contract: Creates
  flow into views, then a delete cascade converges every view to empty.

Algorithm B (G15-A):
- `algorithm_b_view_correctness.rs` — for each of the 5 views, run the
  same event sequence through Strategy::A AND Strategy::B (now
  inner-kernel-of-B per `ivm-disagree-1`); assert row-equivalence on
  every observable read.
- `algorithm_b_general.rs` — G15-A generalization pins (arbitrary
  `(view_id, label_pattern, projection)` triples; AnchorPrefix
  correctness; fail-loud guards; canonical-fast-path-preservation gate
  parses bench JSON and asserts ratio ≤ 1.20).
- `algorithm_b_cross_replica.rs` — RED-PHASE pin for G15-B + G16-B
  convergence (Algorithm B + Loro CRDT cross-replica).
- `algorithm_b_within_20pct_gate.rs` — G8-A bench gate. Per-view
  thresholds from `benches/algorithm_b_thresholds.toml`.
- `algorithm_b_drift_detector.rs` — G15-B proptest harness; uses the
  `Algorithm::register` lane + the budget-aware `ContentListingView`
  lane separately because `register` doesn't surface a budget knob.
- `proptest_algorithm_b_correctness.rs` —
  `prop_algorithm_b_incremental_equals_rebuild` (10k cases originally;
  current corpus is calibrated per `r4-r2-ivm-7`).

Algorithm B (G23-0a/G23-0b SubgraphSpec generalisation — NEW):
- `ivm_algorithm_b_consumes_subgraph_spec.rs` — canary pin for
  `Algorithm::register_subgraph(spec)`: a SubgraphSpec built via
  `SubgraphSpec::for_canonical_view` materializes the same KernelOutput
  bytes as the equivalent `Algorithm::register` triple-form call.
- `ivm_generalized_kernel_no_new_strategy_variant.rs` — pins that G23-0a
  does NOT mint a `Strategy::Generalized` / `Strategy::Subgraph` variant
  (per CLAUDE.md baked-in #2 — `Strategy::B` IS the generalized algorithm).
- `subgraph_shaped_view_self_reference_rejected_at_register_time.rs` —
  mat-r1-13 fail-fast pin: `SelfReferentialSubgraphRejected` surfaces
  BEFORE any kernel input walk.
- `prop_subgraph_shaped_view_equivalent_to_handwritten_for_canonical_inputs.rs`
  — proptest cross-equivalence: G15-A triple + G23-0a SubgraphSpec
  registration paths produce identical KernelOutput bytes for the same
  write sequence (Family C cross-path equivalence guarantee).
- `view_1_capability_grants_subgraph_spec_round_trip.rs` /
  `view_2_event_dispatch_subgraph_spec_round_trip.rs` /
  `view_3_content_listing_subgraph_spec_round_trip.rs` /
  `view_4_governance_inheritance_subgraph_spec_round_trip.rs` /
  `view_5_version_current_subgraph_spec_round_trip.rs` — per-view G23-0b
  round-trip pins: SubgraphSpec-built wrapper materializes
  bytes-equivalent to the G15-A path.
- `view_4_typed_output_projection_shape_pin.rs` /
  `view_5_typed_output_projection_shape_pin.rs` — `mat-r1-1` typed-output
  declaration matches inner-kernel emission shape (Rules / Current).
- `projection_all_props_placeholder_removed_no_remaining_references.rs` —
  pins the G23-0b `Projection::AllProps` variant removal (CRATES-DEEP-DIVE
  §4 closure); fails if any source line names the removed variant.
- `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs` —
  post-SubgraphSpec-walk inner-kernel reads remain shape-equivalent to
  pre-walk reads (no internal-state contamination across registration
  paths).
- `common_kernel_canary.rs` — shared canary helper for the 5 round-trip
  pins (G23-0b restoration after batch-merge per `r5-g23-0b-batch-2-merge`).

Shared:
- `common.rs` — shared test helpers for the drift-detector + cross-replica
  pins (view materialization wrappers, structured-diff helpers,
  asymmetric path diff).

---

## 6. Benches inventory (5 files + thresholds toml)

`benches/view_maintenance.rs` — Two §14.6 gates: view-read on a hot cache
(< 1µs target) and incremental-maintenance per write (< 50µs target).
Currently `informational` because the matrix slot is one value per file
but the file mixes two case-level gates.

`benches/algorithm_b_vs_handwritten.rs` — Per-view ratio gate: Algorithm
B's wallclock must be within 20% of the hand-written baseline. Bench
order is perf-risk descending per `r1-ivm-algorithm.json`
(`content_listing` first — highest gate-risk). Companion test
`algorithm_b_within_20pct_gate.rs` parses the criterion JSON.

`benches/algorithm_b_canonical.rs` — G15-A canonical-view fast-path
preservation gate. Compares `AlgorithmBView::for_id` construction path vs
`Algorithm::register` (which dispatches through the generalized kernel) —
both should end up at the same hand-written inner kernel for canonical
ids, so the ratio should be ≤ 1.20.

`benches/ivm_generalized_kernel_hot_path_within_20_percent_of_handwritten_baseline.rs`
— **NEW G23-0a** generalized-kernel-hot-path gate. Measures the
SubgraphSpec-registration walk path against the G15-A triple-form
baseline; ratio target ≤ 1.20. Backs the G23-0a "generalisation is free
at the hot path" claim.

`benches/view_staleness_transition.rs` — Informational. Measures the cost
of transitioning a view from healthy to Stale + the already-stale
short-circuit cost.

`benches/algorithm_b_thresholds.toml` — per-view ceiling table read by
the bench gate.

---

## 7. Thin-engine + composable-graph philosophy check

IVM is the **materializer of state from event stream** — and per D-4F-2
(Phase-4-Foundation R1 triage ratification) the materializer view IS an
IVM view: the materializer pipeline registers its views through
`Algorithm::register_subgraph(spec)`. SubgraphSpec is the universal
kernel-input shape. Reading this crate through that lens surfaces several
architectural observations:

### 7.1 Well-respected: the canonical-view set is engine-special-cased, NOT user-registerable as Strategy::A

The 5 canonical view ids (`capability_grants`, `event_dispatch`,
`content_listing`, `governance_inheritance`, `version_current`) are
hard-coded in `CANONICAL_VIEW_IDS` and routed through their bespoke
hand-written kernels via `AlgorithmBView::for_id`. Users cannot register a
new view with `Strategy::A` — the engine refuses it
(`EngineError::ViewStrategyARefused`). User-registered views go through
`Strategy::B`'s `GenericKernel` exclusively, which only knows
`(label_pattern, projection)` triples (or equivalently, a
`SubgraphSpec::user_view`).

### 7.2 Well-respected: the `Strategy` enum is the only IVM type the engine names in its public API

CLAUDE.md item #2 specifically codifies this: "the engine names
`benten_ivm::Strategy` as the dispatch type but no `View` / algorithm
internals leak through". The engine now also names `SubgraphSpec` (a
canary input shape) but per arch-r1-14 G23-0a does NOT introduce a new
`Strategy` variant — the engine-boundary classification is still
{A, B, Reserved}.

### 7.3 Caveat: `dispatch_for` and `is_canonical_view_id` are `pub` but documented as INTERNAL

The doc-comment carries the intent but the visibility doesn't enforce it.
The engine boundary holds because the only call site outside the crate
is the engine's own `register_user_view`. A future re-export sweep could
narrow `dispatch_for` to `pub(crate)` once the engine's reliance is fully
audited; for now it's an honor-system boundary.

### 7.4 Projection is a unit struct (placeholder removed at G23-0b)

The pre-G23-0b `Projection` enum carried a single no-op `AllProps`
identity variant. G23-0b removed the variant (CRATES-DEEP-DIVE §4
closure) — `Projection` is now a unit struct (line 231 of
`algorithm_b.rs`). The kernel applies the identity transform on every
matched Node. View 4 (Rules) + View 5 (Current) typed-output shapes are
declared via the NEW `TypedOutputProjection` enum on `SubgraphSpec` per
`mat-r1-1`. Future real projections (`PropSubset`, `Computed`,
`Reshape`) lift to an enum shape behind the same call signature without
breaking the kernel call surface.

### 7.5 Generality gap — first-label-only matching

`GenericKernel::first_label_matches` and View 3's `update` arm both look
at `node.labels.first()` and test the pattern against that. Multi-label
Nodes get their secondary labels ignored. The `algorithm_b.rs` docstring
explicitly names this — "Matchers that need multi-label semantics belong
at a higher selector layer (named in `docs/future/phase-3-backlog.md`
§5.1-followup-b for edge-traversal-keyed views)". Right disposition.

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
trip happens only in adversarial / budget-misconfigured cases. The named
Phase-3+ event-replay work closes this.

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
`E_IVM_PATTERN_MISMATCH`. The G23-0b `walk_writes` + `materialize` path
sidesteps this by producing the canary observable from the wrapper's
`label_pattern` + `walk_observable` rather than the inner kernel's
`read(query)` — but production reads still go through `ViewQuery`.

### 7.8 Caveat: `Subscriber::on_change` uses `eprintln!` for non-fatal errors

The subscriber logs `PatternMismatch` and view-panic events via
`eprintln!` (with `#[allow(clippy::print_stderr)]` markers naming the
Phase-2 `tracing` migration). Engine-level observability is wired through
metric snapshots (`benten.ivm.view_stale_count` reads `stale_count_tally`),
so the operator-facing surface isn't dependent on the stderr sink.

### 7.9 Caveat: Phase-1 budget arithmetic is per-view local, not subscriber-wide

Each view holds its own `BudgetTracker`; nothing globally throttles
across-view work. A `Subscriber` with 50 views fans every event to all 50
without per-event aggregate cost-control. Phase 1 ships this knowingly (5
views; load is trivially bounded by 5x per-event work). When user views
proliferate, the named "TODO(phase-3 — pattern-based pre-filtering
router)" in `subscriber.rs` becomes load-bearing — and per D-4F-2 the
materializer's schema-driven view registrations push this concern into
Phase-4-Meta.

### 7.10 G23-0a/G23-0b canary observable is wrapper-side, not inner-kernel-side

The G23-0b `walk_observable: BTreeSet<Cid>` lives on the
`AlgorithmBView` wrapper, NOT inside the inner kernel. This is a
deliberate choice: 4 of the 5 canonical inner kernels'  `read` paths
require a populated `ViewQuery` field (entity_cid / event_name /
anchor_id), so they can't produce a uniform default-query observable for
the canary contract. The wrapper-side observable is keyed only on the
shared `label_pattern` and works uniformly for all 5 canonical kernels +
the generic kernel. The `materialize` path consults the inner kernel's
`read` purely for SHAPE confirmation (Rules vs Cids vs Current) — the
typed-output projection gate at register-time stays load-bearing at
materialise-time per `g23-0a-mr-3`.

---

## 8. Phase 4-Foundation + Phase 4-Meta expectations

Per D-4F-2 (ratified Phase-4-Foundation R1 triage) the materializer
pipeline IS an IVM kernel consumer: G23-B wave-5 registers materializer
views through `Algorithm::register_subgraph(spec)`. Sketch A (materializer
composes USING IVM views) is the ratified path — Sketches B (parallel
subscriber) and C (subgraphs replace incremental indices) are NOT taken
for Phase 4-Foundation.

The materializer's `Renderer` trait is the host-side output transform; the
kernel itself doesn't know it's serving a materializer. Future shape
narrowing the materializer needs (richer `Projection` enum, multi-label
matching, joins, edge-traversal-keyed views) lifts under the existing
SubgraphSpec contract without breaking the canary stability commitment.

The architectural fork named in §8 of the pre-G23-0a snapshot of this doc
("composes USING IVM" vs "alongside IVM" vs "replaces IVM with subgraph
walk") is RESOLVED at D-4F-2: composes USING IVM.

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

- **`Strategy::Reserved` is reserved-not-implemented forever-deferred.**
  Phase target named as `"Phase 3+"` in the `StrategyNotImplemented`
  payload; renamed from `Strategy::C` at G23-0a per arch-r1-14 (closes
  CRATES-DEEP-DIVE §4 named-but-deferred item). Z-set / DBSP
  cancellation is a real algorithmic family the variant gestures at, but
  the value of pre-reserving a variant for an algorithm we haven't
  designed is more API-stability-theater than concrete forward planning.

- **Subscriber pattern-based pre-filtering is named Phase-3 TODO and
  hasn't moved.** Every view sees every event with internal filter. Fine
  at 5 + a handful of user views; at "Phase 4-Foundation materializer
  registers 50 schemas → 50 views" scale this becomes a problem.

- **`Subscriber::on_change` uses `eprintln!` for non-fatal errors.**
  Documented in §7.8. `tracing` migration named since Phase 1.

- **`AlgorithmBView::for_id_with_budget` for `content_listing` with a
  non-`"post"` label silently drops the budget.** Lines ~696-708 of
  `algorithm_b.rs`: when the supplied label isn't `"post"`, the
  constructor calls `ContentListingView::new(label)` instead of
  `with_budget_for_testing(budget)`, observably losing the supplied
  budget. The docstring names the gap and points at
  `phase-3-backlog.md §5.1-followup-e residual`. The canonical
  constructor needs to be lifted to accept `(label, budget)` together.

- **`Projection` is a unit struct.** Documented in §7.4. The kernel
  surface is ready for richer projections but the struct is placeholder.
  First user view that needs anything non-identity (or the materializer
  needing reshape/compose) will force the lift to enum form.

- **`first_label_matches` ignores secondary labels.** Documented in §7.5.
  Right disposition (defer to named backlog item §5.1-followup-b); worth
  surfacing if a user-view registration in Phase 4-Foundation needs
  multi-label matching.

- **`BudgetTracker` charges 1 unit per event regardless of work.** Views
  3 (deletes, where `removed.max(1)` is charged) and 2 (deletes,
  proportional to buckets touched) tightened this; views 1/4/5 still
  charge a flat 1 per event. Likely fine — read paths are not budgeted —
  but a "delete storm against capability_grants causes every grant to
  scan the full map" pattern still charges 1 unit for what is O(n) work.
  Documented inline in the Phase-2 fix-pass notes (`g5-p2-ivm-2`).

- **G23-0b canary observable lives on the wrapper, not the inner
  kernel.** Documented in §7.10. The split is deliberate (inner kernels'
  `read` paths require populated `ViewQuery` fields); a future
  production materialise pathway that lifts at G24-A would route through
  the inner kernel's `read` (the retained-but-dead `canonicalize_rules`
  helper at `algorithm_b.rs:1188` is named for that future path).
