# Phase 2 Backlog

**Status:** Consolidated list of items deferred from Phase 1 that have a clear Phase 2 landing point. Not forward-guidance on abstractions the way `docs/future/` is — these are things Phase 1 explicitly scoped out, with commits and specs already citing the deferral.

**Scope distinction from `docs/future/README.md`:**
- `future/` = exploratory proposals, may or may not ship (e.g. `benten-runtime.md`, `compute-marketplace.md`).
- `phase-2-backlog.md` (this file) = committed deferrals with concrete Phase 1 references. Every item here is expected to land in Phase 2 or has a clear trigger for earlier landing.

**Phase 2 scope anchor:** `docs/FULL-ROADMAP.md` §Phase 2 — "Evaluator completion + WASM SANDBOX + remaining invariants." See `ENGINE-SPEC.md` §14 for the evaluator-side primary plan.

---

## 1. Architectural deferrals

### 1.1 `benten-eval` → `benten-graph` dependency break (arch-1)

**Phase 1 state:** `crates/benten-eval/Cargo.toml` depends on `benten-graph` because `EvalError::Graph(#[from] GraphError)` carries a `GraphError` variant. That's the sole cross-crate coupling; the evaluator otherwise talks to storage only through `PrimitiveHost`.

**Phase 2 target:** Remove the dep. `EvalError` no longer surfaces `GraphError`; the engine's `PrimitiveHost` impl catches storage errors at the host boundary and maps them to a new `HostError` type (or an already-stable code in `benten-errors`).

**Why deferred:** The `PrimitiveHost` surface isn't fully stable until Phase 2 adds WAIT / STREAM / SUBSCRIBE / SANDBOX executors. Designing `HostError` now forces guesses about the Phase 2 error surface. The thin-engine philosophy (`docs/DEVELOPMENT-METHODOLOGY.md` Pattern: thin engine, compose everything on top) explicitly cautions against pre-settling abstractions the Phase 2 primitive surface will actually exercise.

**Trigger for earlier landing:** If any Phase 2 primitive implementation needs a `GraphError` shape that the current `#[from]` doesn't carry, do the dep break first rather than growing the coupling.

**Touch size estimate:** 15–30 files. Medium risk.

### 1.2 File splits not taken in Phase 1

- `crates/benten-graph/src/transaction.rs` (711 lines). Skipped because `PendingOp` + `TxGuard` + `Transaction` form a single cohesive state-machine and splitting them separates a data type from the code that produces and consumes it. Re-evaluate if Phase 2 adds enough transaction machinery (retry, MVCC read-transaction, streaming commit) that the file crosses ~1200 lines — at that size a logical seam probably emerges.
- `crates/benten-engine/src/engine.rs` was split in Phase 1 (5d-K) into engine.rs + 4 sibling modules (crud, caps, views, diagnostics). The orchestration core is still ~1600 lines dominated by `dispatch_call_inner` and `subgraph_for_crud` — Phase 2 may separate subgraph-construction from dispatch-walking once those shapes settle.

---

## 2. Deferred primitives (4 of 12)

All primitive **types** are defined in `benten-eval` and pass structural validation; their executor paths return `E_PRIMITIVE_NOT_IMPLEMENTED`. Phase 2 wires the executors.

| Primitive | Phase 2 work |
|-----------|--------------|
| `WAIT` | Serializable execution state (so the evaluator can suspend/resume); time-source abstraction; integration with `uhlc` for deterministic re-entry |
| `STREAM` | Back-pressure semantics, partial-output emission, WinterTC-compatible ReadableStream bridge at the napi layer |
| `SUBSCRIBE` (user-visible op) | Currently only the engine-internal change-stream plumbing exists. Phase 2 exposes SUBSCRIBE as a primitive that user subgraphs can write |
| `SANDBOX` | Full `wasmtime` integration, fuel metering, per-subgraph fuel budgets, no re-entrancy, output size limits, host-function manifest. See `docs/SECURITY-POSTURE.md` Compromise #4 |

---

## 3. Deferred invariants (6 of 14)

All Phase 1 invariants (1, 2, 3, 5, 6, 9, 10, 12) are enforced at registration time. Phase 2 adds:

| Invariant | Phase 2 work |
|-----------|--------------|
| 4 — SANDBOX nest depth | Lands with SANDBOX executor |
| 7 — SANDBOX output size | Lands with SANDBOX executor |
| 8 — cumulative iteration budget (multiplicative through CALL / ITERATE) | Today a scalar budget with `E_INV_ITERATE_BUDGET` fires; Phase 2 replaces with multiplicative accounting. `E_INV_ITERATE_NEST_DEPTH` is the Phase 1 registration-time stopgap for the nesting aspect |
| 11 — system-zone labels unreachable from user operations | Phase 1 has `E_SYSTEM_ZONE_WRITE` as a stopgap enforced by the engine's put path; Phase 2 enforces this at the evaluator level so subgraphs cannot reach system-zone CIDs through any primitive |
| 13 — immutability after registration | Phase 1 registers content-addressed subgraphs but does not enforce "cannot modify a registered subgraph." Phase 2 wires the immutability invariant through the capability layer |
| 14 — causal attribution on every evaluation | Phase 1 records the `(actor, handler, grant)` triple on writes (attribution exists); Phase 2 formalises the structural requirement that every evaluation step carries causal attribution |

---

## 4. Security posture follow-ups

### 4.1 Option C (Compromise #2) — evaluator-path READ gating

Phase 1 closure (5d-J) landed symmetric-`None` + `diagnose_read` at the engine orchestrator public API. `PrimitiveHost::check_read_capability` hook is wired with a permissive default. Phase 2 threads the hook into the READ primitive's execute path so `crud:post:get` dispatched through `Engine::call` honours Option C end-to-end without a separate gate at the public API. See `docs/SECURITY-POSTURE.md` §Compromise #2.

### 4.2 Change-stream subscribe bypasses `check_read`

`Engine::subscribe_change_events` fans out every committed `ChangeEvent` without a per-event read-check gate. The Engine instance itself is the security boundary in Phase 1. Phase 3 federation / sync introduces cross-trust-boundary replicas; the subscribe path gains per-subscriber filtering at that point. See `docs/SECURITY-POSTURE.md` §"Change-stream subscription bypasses capability read-checks."

### 4.3 Compromise #6 — BLAKE3 post-quantum reconsideration

Current posture: BLAKE3-256 → 128-bit classical collision resistance, 64-bit under Grover. Revisit if post-quantum transitions become a committed requirement (Phase N+). Options D (dual-hash) and the hybrid boundary-translation approach are in `docs/SECURITY-POSTURE.md` §"Hash algorithm choice."

---

## 5. IVM deferrals

### 5.1 Generalized Algorithm B

Phase 1 ships **5 hand-written IVM views** (capability-grant resolution, event-handler dispatch table, content listing, governance inheritance, version-chain CURRENT pointer). The generalized Algorithm B with per-view strategy selection (A/B/C) and user-registered views is Phase 2. See `docs/research/ivm-benchmark/RESULTS.md` for the algorithm choice.

### 5.2 `E_IVM_PATTERN_MISMATCH` firing (spec-to-code audit §5.5)

`ViewError::PatternMismatch` exists and is `.code()`-mapped but no concrete view in `benten-ivm/src/views/*.rs` constructs it. The `r5-g5-mini-ivm-algorithm-b-reviewer` flagged this; the fix was not landed in Phase 1 because the five hand-written views over-answer unmatched queries (a behaviour-visible change that would ship cleanest alongside the Phase 2 generalized view registration).

### 5.3 `benten.ivm.view_stale_count` metric wiring

`metrics_snapshot()` emits `benten.ivm.view_stale_count: 0.0` as a hard-coded placeholder. The subscriber tally (using the existing `View::is_stale()`) is a <10-line wire-up that was deferred out of Phase 1's test-flakiness budget (tallying during `metrics_snapshot` would need to iterate all view handles, which lives behind a mutex). Drop the key or wire it — cheap either way.

### 5.4 IVM rebuild from event log

Phase 1 views rebuild only by re-applying every committed `ChangeEvent` from engine startup. Phase 2 adds durable view snapshots so a crash doesn't force a full re-walk.

---

## 6. `benten-core` + content-addressing

### 6.1 `Cid::from_str` implementation (spec-to-code audit §5.9)

`Cid::from_str` unconditionally returns `CoreError::CidParse("Cid::from_str is a Phase 2 deliverable; needs multibase decoder")`. The `ERROR-CATALOG.md` `E_CID_PARSE` fix-hint says "Phase 1 accepts base32-lower-nopad multibase" which reads as "Phase 1 accepts it" — the napi boundary accepts it (via a separate path that fires `E_INPUT_LIMIT`), but Rust's `Cid::from_str` does not. Phase 2 lands the ~30-line base32 decoder (mirrors the existing `to_base32` encoder in `benten-core/src/lib.rs`).

### 6.2 `get_node_verified` read-path hash check (spec-to-code audit §5.7)

`RedbBackend::get_node` does NOT re-hash the decoded Node and compare against the requested CID; a corrupted `{key, value}` pair returns a wrong-but-decodable Node. The `ERROR-CATALOG.md` entry for `E_INV_CONTENT_HASH` says "Thrown at: Registration / read" — but read-path firing is subgraph-only (via `Subgraph::load_verified`). Phase 2: add optional `get_node_verified(&self, cid)` that re-hashes on read (~3–10 µs BLAKE3 per call), or tighten the catalog entry to distinguish "Registration / Subgraph load" from "Node read."

### 6.3 Anchor-store consolidation (cov-f3 residual)

`benten-core/src/version.rs` uses per-anchor `Arc<Mutex<...>>` for chain state. R5 G7 may prefer an explicit `AnchorStore` handle for bulk operations once the evaluator surface is stable. See the `TODO(phase-2-anchorstore)` in `version.rs`.

### 6.4 Dependency unpins

Two upstream multiformats PRs are waiting on maintainer merge:
- `multiformats/rust-cid#185` — APPROVED, awaiting merge
- `multiformats/rust-multihash#407` — changes-requested nit

When both merge and release, remove the `[patch.crates-io]` entries in the workspace `Cargo.toml`.

### 6.5 `E_CID_UNSUPPORTED_CODEC` / `E_CID_UNSUPPORTED_HASH` wiring

Phase 1 closed this in commit `9b67fc5`: `Cid::from_bytes` now distinguishes codec-byte mismatch from multihash-byte mismatch. Noted here only as a closed residual from the spec-to-code audit §5.4 for traceability.

---

## 7. Capability / UCAN (Phase 3-adjacent)

### 7.1 Wall-clock + iteration TOCTOU delegation

`crates/benten-caps/src/policy.rs` has `TODO(phase-2-iterate-boundary-delegation)` and `TODO(phase-2-wallclock-toctou)`. Phase 1 checks capabilities at commit, CALL entry, and every `iterate_batch_boundary` (n=100 default) iterations; Phase 2 threads the engine's wall-clock bound through so long-running iterations also honour time-based capability expiry. Compromise #1 in `docs/SECURITY-POSTURE.md` is the spec source.

### 7.2 UCAN backend

`benten-caps::UCANBackend` is a stub (`CapError::NotImplemented`). Full UCAN chain validation + delegation ships in Phase 3 `benten-id` alongside Ed25519 / DID / VC.

---

## 8. DX / tooling

### 8.1 Dev server with hot reload

Phase 1 ships the `npm test` path (mechanically verifiable). Dev-server with hot reload is Phase 2. See `.addl/phase-1/00-implementation-plan.md` §1 and `ENGINE-SPEC.md` Rank 10.

### 8.2 CLAUDE.md → ENGINE-SPEC numeric-claim drift lint

`spec-to-code-compliance-audit.md` §6.3 flagged that CLAUDE.md's performance targets have drifted from ENGINE-SPEC §14.6 three times. A small CI lint that greps CLAUDE.md for numeric performance claims and compares them against ENGINE-SPEC ranges would close the loop. Low priority — the `docs/DEVELOPMENT-METHODOLOGY.md` Pattern 5 (Verify, don't trust docs) review discipline catches most occurrences.

### 8.3 Per-item `missing_docs` sweep

`benten-eval` has ~120 public items with a `warn(missing_docs) + allow(missing_docs, reason="...Phase-2...")` pattern at the crate root. Crate-root + module-root docstrings landed in Phase 1 R6; per-item sweep is deferred to Phase 2 when the public surface is re-audited post-evaluator-completion. Target: drop the `allow(missing_docs)` escape hatch entirely in Phase 2.

---

## 9. Performance

### 9.1 macOS APFS fsync floor

`ENGINE-SPEC.md` §14.6 documents the ~4–13 ms Immediate-durability fsync floor on macOS APFS. Phase 1's `crud_post_create_dispatch` criterion bench is NOT CI-gated because of this. Phase 2 considers a `DurabilityMode::Group` default for the CRUD fast-path so the 150–300 µs target is reachable on dev hardware.

### 9.2 Subgraph AST cache

`Engine::call` still re-parses the subgraph per-call. Phase 2 completes the AST-cache wire-up referenced in `engine.rs:449`.

---

## 10. In-code `TODO(phase-2-*)` markers

At Phase 1 close, the codebase contains ~180 `Phase 2` / `TODO(phase-2-*)` markers across crate source. This backlog doc consolidates the structural items; finer-grained markers stay in-code for the agent that touches that area. Grep for them as needed:

```bash
grep -rn "TODO(phase-2\|phase-2-" crates/ bindings/ packages/ tools/
```

---

## Revival / escalation

When Phase 2 kicks off:

1. This doc seeds the Phase 2 pre-R1 implementation plan.
2. Items with "Trigger for earlier landing" notes get evaluated against Phase 2's primitive-landing order before the plan locks.
3. Items that turned out to be non-issues get deleted (not silently left to rot).
4. New deferrals surfaced during Phase 2 get appended to this file under `## 11. Phase 2 working residuals` (or whatever phase is live) rather than creating a new deferral file per phase.

See `docs/DEVELOPMENT-METHODOLOGY.md` Pattern 1 (After every review, fix or defer — never "noted").
