# Invariant Coverage — Phase 2b Close

CLAUDE.md commits to **14 invariants** governing the Benten engine.
This document tracks per-invariant enforcement state, the enforcing
crate, and the regression suite that pins it.

**Phase 2b status:** 14 of 14 invariants enforced. Inv-4 + Inv-7 went
ACTIVE in Phase 2b alongside the SANDBOX runtime (registration arm
landed in G7-B; runtime arm landed across waves 8b + 8h with a bounded
honest-disclosure for Inv-4 — see the "Inv-4 + Inv-7 runtime arm
status" section below). The two Phase-1 stubs are now removed.

---

## Coverage table

| # | Invariant | Phase | Enforcer | Tests |
|---|-----------|-------|----------|-------|
| 1 | DAG-ness — no cycles in operation graphs | 1 | `benten-eval::invariants::structural::validate_at_registration` (Kahn cycle detect) | `crates/benten-eval/src/invariants/structural.rs` (cycle test cluster) |
| 2 | Max operation-subgraph depth | 1 | Bounded longest-path walk + per-CALL increment | `structural.rs::depth_*` tests |
| 3 | Max fan-out per node | 1 | Edge enumeration at registration | `structural.rs::fan_out_*` tests |
| 4 | **SANDBOX nest-depth ceiling — ACTIVE (Phase 2b G7-B / wave-8b / D20; both arms wired at R6FP-G1 / PR #62)** | 2b | `invariants::sandbox_depth::validate_registration` (registration); `AttributionFrame.sandbox_depth` runtime threading in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (parent-sandbox_depth+1) + `SandboxError::NestedDispatchDepthExceeded` fires in `crates/benten-eval/src/primitives/sandbox.rs::execute` (runtime arm — both arms now active) | `crates/benten-eval/tests/inv_4_*.rs`, `crates/benten-engine/tests/inv_4_*.rs`, `tests/sandbox_depth_inheritance_through_call.rs` |
| 5 | Max total nodes per subgraph | 1 | Node-count gate at registration | `structural.rs::node_count_*` tests |
| 6 | Max total edges per subgraph | 1 | Edge-count gate at registration | `structural.rs::edge_count_*` tests |
| 7 | **SANDBOX `output_max_bytes` range — ACTIVE (Phase 2b G7-B / wave-8b / D15 + D17 PRIMARY+BACKSTOP)** | 2b | `invariants::sandbox_output::validate_registration` (registration); `CountedSink::write` (PRIMARY streaming) + `CountedSink::backstop_check` (return-value BACKSTOP), both wired through wave-8b's host-fn trampoline + primitive boundary | `crates/benten-eval/tests/inv_7_*.rs`, `crates/benten-engine/tests/inv_7_*.rs`, `crates/benten-eval/src/sandbox/counted_sink.rs` |
| 8 | Multiplicative cumulative budget (CALL × ITERATE) | 2a | `invariants::budget` + `BudgetTracker` per evaluator step | `crates/benten-eval/src/invariants/budget.rs` (proptest cluster) |
| 9 | Determinism — handlers declared deterministic reject non-determinism sources | 1 (decl) / 2a (rt) | `structural::validate_at_registration` declaration check + runtime fence | `structural.rs::determinism_*` |
| 10 | Canonical byte encoding (order-independent DAG-CBOR) | 1 | `structural::canonical_bytes` order-independence proptest | `structural.rs::canonical_bytes_*` |
| 11 | System-zone reserved-prefix reject — user code cannot READ/WRITE system labels | 2a | `invariants::system_zone` (G5-B-i) + Engine::put_node_with_context dispatch | `crates/benten-engine/tests/inv_11_*.rs` |
| 12 | Aggregate validation catch-all — multi-invariant violations roll up | 1 | `RegistrationError::Invariant12Aggregate` | `structural.rs` aggregate-error tests |
| 13 | Immutability — User WRITE re-puts of an already-persisted CID fire `E_INV_IMMUTABILITY` | 2a | `invariants::immutability` + `WriteAuthority` firing matrix | `crates/benten-engine/tests/inv_13_*.rs` |
| 14 | Causal attribution — every primitive frame carries an `AttributionFrame` | 2a | `evaluator::attribution` runtime threading + `ATTRIBUTION_PROPERTY_KEY` registration check | `crates/benten-eval/tests/attribution_*.rs`, `crates/benten-engine/tests/attribution_*.rs` |

---

## Inv-4 + Inv-7 runtime arm status (honest disclosure)

Phase 1 shipped Inv-4 + Inv-7 as **stubs** because the SANDBOX primitive
itself was compile-check only (Compromise #4 in
`docs/SECURITY-POSTURE.md`). Phase 2b G7-B added the registration-time
arms; waves 8b + 8h wired the runtime executor end-to-end. R6FP Round-1
Group-1 (PR #62, 3-lens convergent fix) closed the remaining transitive
threading gap. Both runtime arms are fully active at Phase 2b close:

- **Inv-7 (SANDBOX `output_max_bytes` range)** — **fully active at
  runtime.** The wave-8b host-fn trampoline routes every host-fn
  byte-emit through `CountedSink::write`'s `OutputCheckPath::PrimaryStreaming`
  arm; the primitive boundary runs `CountedSink::backstop_check` against
  the return value (`OutputCheckPath::ReturnBackstop`). Per D17 PRIMARY +
  BACKSTOP. Per-handler ceiling per D15. Default ceiling 1 MiB;
  `SandboxArgs.outputLimitBytes` overrides per-call. Note: the
  `invariants::sandbox_output::check_admission` helper exists and is
  unit-tested but is NOT the production firing site — `CountedSink`
  enforces the same arithmetic directly via `SinkOverflow` →
  `SandboxError::OutputOverflow` mapping. Both paths produce
  `E_INV_SANDBOX_OUTPUT` typed errors with identical context shapes.

- **Inv-4 (SANDBOX nest-depth ceiling)** — **both arms fully active at
  Phase 2b close.** (1) Registration arm: `validate_registration` walks
  the static-graph at registration time. (2) Runtime arm: R6FP-G1 (PR
  #62) wired the `AttributionFrame.sandbox_depth` threading through the
  parent `ActiveCall`. At every production SANDBOX entry,
  `crates/benten-engine/src/primitive_host.rs::execute_sandbox` mutates
  the parent frame via `frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)`;
  the dispatching `AttributionFrame` is constructed with `sandbox_depth:
  nested_depth` in both match arms of the same function so subsequent
  CALL pushes inherit. The eval-side runtime arm in
  `crates/benten-eval/src/primitives/sandbox.rs::execute` fires
  `SandboxError::NestedDispatchDepthExceeded` once `attribution.sandbox_depth
  > config.max_nest_depth`. Default `max_nest_depth = 4` admits depths
  1..=4; depth 5 fires. SANDBOX-inside-CALL-inside-SANDBOX inherits the
  parent's depth correctly. Carry-forward residual: the ESC-10
  adversarial integration test (`sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied`)
  stays `#[ignore]`'d pending the `testing_call_engine_dispatch` host-fn
  helper per `docs/future/phase-3-backlog.md` §7.3.A.7. The runtime arm
  is wired; only the adversarial-test driver is paper-only.

Both invariants fire as `E_INV_SANDBOX_DEPTH` (Inv-4) and
`E_INV_SANDBOX_OUTPUT` (Inv-7) error codes — both pinned in
`docs/ERROR-CATALOG.md`. The catalog rows now reflect the per-arm
honest disclosure.

The Phase-1 "Phase 2b" stubs that previously appeared in this table
have been removed; Inv-4 + Inv-7 are now first-class active rows.

---

## Where each invariant is enforced

```
┌────────────────────────────────────────┬─────────────────────────────────────────┐
│ Registration-time (one-shot)           │ Runtime-time (per-call, per-frame)      │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ Inv-1 DAG-ness                         │ Inv-4 sandbox_depth runtime counter     │
│ Inv-2 max depth                        │ Inv-7 sandbox_output CountedSink        │
│ Inv-3 fan-out                          │ Inv-8 BudgetTracker step gate           │
│ Inv-4 sandbox_depth declaration        │ Inv-13 WriteAuthority firing matrix     │
│ Inv-5 node count                       │ Inv-14 AttributionFrame propagation     │
│ Inv-6 edge count                       │                                         │
│ Inv-7 sandbox_output declaration       │                                         │
│ Inv-9 determinism declaration          │                                         │
│ Inv-10 canonical-bytes order-indep     │                                         │
│ Inv-11 system-zone literal-CID reject  │                                         │
│ Inv-12 aggregate roll-up               │                                         │
│ Inv-14 ATTRIBUTION_PROPERTY_KEY decl   │                                         │
└────────────────────────────────────────┴─────────────────────────────────────────┘

* Inv-4 runtime counter — fully wired at R6FP-G1 (PR #62). Both
  registration arm + runtime arm are active at Phase 2b close. See
  §"Inv-4 + Inv-7 runtime arm status" above for the wiring trace.
```

---

## IVM Algorithm B production registration (audit-gap closure note)

Wave-8h closed the IVM Algorithm B production-registration drift the
docs-vs-code audit caught: `Engine::create_user_view` previously
forced `ContentListingView` for every `Strategy::B`-declared user
view. Post-wave-8h the dispatch constructs `AlgorithmBView::for_id(spec.id())`
for the **5 canonical view IDs** that `AlgorithmBView` supports
natively (the hand-written single-loop dispatch in
`crates/benten-ivm/src/algorithm_b.rs`).

**Phase-3 G15-A + G15-B + R5 wave-9 W9-T1 closure:** the
non-canonical-view fallback is RETIRED. `Algorithm::register(view_id,
label_pattern, projection)` (and the budget-aware sibling
`Algorithm::register_with_budget`) instantiates a generic single-loop
kernel (`benten_ivm::algorithm_b::GenericKernel`) for non-canonical
view IDs keyed on `(label_pattern, projection)`. The genuine
`AnchorPrefix` selector lift (post-G15-A) ships in `register_user_view`;
the kernel-side guard refuses canonical-id + AnchorPrefix
registrations with the typed `AlgorithmError::CanonicalIdAnchorPrefixRefused`
variant (mirrored at the engine boundary as
`EngineError::ViewLabelMismatch`). The drift-detector proptest harness
at `crates/benten-ivm/tests/algorithm_b_drift_detector.rs` (5 pins,
1 000 cases each) drives the merged `Algorithm::register` surface
end-to-end + reports incremental-vs-rebuild parity.

---

## What "active" means in this table

A row is **active** iff:

1. The invariant has a typed `RegistrationError` or `ErrorCode` it
   raises on violation.
2. The invariant has at least one regression test pinning the firing
   condition.
3. The crate that owns enforcement consumes the invariant on every
   relevant code path (i.e. there is no observable code path that
   bypasses it without explicit named-compromise documentation in
   `docs/SECURITY-POSTURE.md`).

All 14 invariants meet (1) (2) (3) at Phase 2b close.
