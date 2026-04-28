# Invariant Coverage — Phase 2b Close

CLAUDE.md commits to **14 invariants** governing the Benten engine.
This document tracks per-invariant enforcement state, the enforcing
crate, and the regression suite that pins it.

**Phase 2b status:** 14 of 14 invariants enforced (Inv-4 + Inv-7 went
live in Phase 2b G7-B alongside the SANDBOX runtime). The two
Phase-1 stubs are now removed.

---

## Coverage table

| # | Invariant | Phase | Enforcer | Tests |
|---|-----------|-------|----------|-------|
| 1 | DAG-ness — no cycles in operation graphs | 1 | `benten-eval::invariants::structural::validate_at_registration` (Kahn cycle detect) | `crates/benten-eval/src/invariants/structural.rs` (cycle test cluster) |
| 2 | Max operation-subgraph depth | 1 | Bounded longest-path walk + per-CALL increment | `structural.rs::depth_*` tests |
| 3 | Max fan-out per node | 1 | Edge enumeration at registration | `structural.rs::fan_out_*` tests |
| 4 | **SANDBOX nest-depth ceiling — ACTIVE (Phase 2b G7-B / D20)** | 2b | `invariants::sandbox_depth` + `AttributionFrame.sandbox_depth: u8` runtime counter (INHERITED across CALL boundaries) | `crates/benten-eval/tests/inv_4_*.rs`, `crates/benten-engine/tests/inv_4_*.rs`, `tests/sandbox_depth_inheritance_through_call.rs` |
| 5 | Max total nodes per subgraph | 1 | Node-count gate at registration | `structural.rs::node_count_*` tests |
| 6 | Max total edges per subgraph | 1 | Edge-count gate at registration | `structural.rs::edge_count_*` tests |
| 7 | **SANDBOX `output_max_bytes` range — ACTIVE (Phase 2b G7-B / D15 + D17 PRIMARY+BACKSTOP)** | 2b | `invariants::sandbox_output` + centralized trampoline counting (D17 PRIMARY) + per-handler ceiling (D15) | `crates/benten-eval/tests/inv_7_*.rs`, `crates/benten-engine/tests/inv_7_*.rs`, `tests/sandbox_output_centralized_counting.rs` |
| 8 | Multiplicative cumulative budget (CALL × ITERATE) | 2a | `invariants::budget` + `BudgetTracker` per evaluator step | `crates/benten-eval/src/invariants/budget.rs` (proptest cluster) |
| 9 | Determinism — handlers declared deterministic reject non-determinism sources | 1 (decl) / 2a (rt) | `structural::validate_at_registration` declaration check + runtime fence | `structural.rs::determinism_*` |
| 10 | Canonical byte encoding (order-independent DAG-CBOR) | 1 | `structural::canonical_bytes` order-independence proptest | `structural.rs::canonical_bytes_*` |
| 11 | System-zone reserved-prefix reject — user code cannot READ/WRITE system labels | 2a | `invariants::system_zone` (G5-B-i) + Engine::put_node_with_context dispatch | `crates/benten-engine/tests/inv_11_*.rs` |
| 12 | Aggregate validation catch-all — multi-invariant violations roll up | 1 | `RegistrationError::Invariant12Aggregate` | `structural.rs` aggregate-error tests |
| 13 | Immutability — User WRITE re-puts of an already-persisted CID fire `E_INV_IMMUTABILITY` | 2a | `invariants::immutability` + `WriteAuthority` firing matrix | `crates/benten-engine/tests/inv_13_*.rs` |
| 14 | Causal attribution — every primitive frame carries an `AttributionFrame` | 2a | `evaluator::attribution` runtime threading + `ATTRIBUTION_PROPERTY_KEY` registration check | `crates/benten-eval/tests/attribution_*.rs`, `crates/benten-engine/tests/attribution_*.rs` |

---

## Inv-4 + Inv-7 — Phase-2b activations

Phase 1 shipped Inv-4 + Inv-7 as **stubs** because the SANDBOX
primitive itself was compile-check only (Compromise #4 in
`docs/SECURITY-POSTURE.md`). Phase 2b G7-B activated both:

- **Inv-4** (SANDBOX nest-depth ceiling) gates registration AND
  runtime. Registration walks the static graph; runtime carries the
  D20 `AttributionFrame.sandbox_depth: u8` counter, which **inherits
  across CALL boundaries** so an attacker cannot bypass the ceiling
  by laundering depth through CALL chains. Nest-depth limit defaults
  to 3 (configurable via `EngineBuilder::sandbox_nest_depth_max`).
- **Inv-7** (SANDBOX `output_max_bytes` range) gates the cumulative
  per-SANDBOX-call wire-bytes from host-fn returns + module return
  values. Per D17 PRIMARY + BACKSTOP, the host trampoline does the
  centralised counting; per-handler ceiling per D15. Default ceiling
  1 MiB; `SandboxArgs.outputLimitBytes` overrides per-call.

Both invariants fire as `E_INV_SANDBOX_DEPTH` (Inv-4) and
`E_INV_SANDBOX_OUTPUT` (Inv-7) error codes — both pinned in
`docs/ERROR-CATALOG.md`.

The Phase-1 "Phase 2b" stubs that previously appeared in this table
have been removed; Inv-4 + Inv-7 are now first-class active rows.

---

## Where each invariant is enforced

```
┌────────────────────────────────────────┬─────────────────────────────────────────┐
│ Registration-time (one-shot)           │ Runtime-time (per-call, per-frame)      │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ Inv-1 DAG-ness                         │ Inv-4 sandbox_depth runtime counter     │
│ Inv-2 max depth                        │ Inv-7 sandbox_output trampoline counter │
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
```

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
