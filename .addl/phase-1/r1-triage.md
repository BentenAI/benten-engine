# R1 Triage — Phase 1 Implementation Plan

**Stage:** R1 (spec-agent peer review, after pre-R1 plan + pre-R1 triage).
**Plan under review:** `.addl/phase-1/00-implementation-plan.md` (revised post-pre-R1).
**Critics:** architect-reviewer, code-reviewer, security-auditor, dx-optimizer, benten-engine-philosophy (dispatched via general-purpose because agent registry hadn't reloaded). Independent lenses, non-overlapping by design.
**Raw findings:** `.addl/phase-1/r1-{architect-reviewer,code-reviewer,security-auditor,dx-optimizer,benten-engine-philosophy}.json`.

## Headline

| Critic | Verdict | Critical | Major | Minor | Total |
|---|---|---|---|---|---|
| `architect-reviewer` | revise | 0 | 5 | 5 | 10 |
| `code-reviewer` | revise | 0 | 8 | 9 | 17 |
| `security-auditor` | revise | **4** | 7 | 4 | 15 |
| `dx-optimizer` | revise | 0 | 4 | 8 | 12 |
| `benten-engine-philosophy` | revise | 0 | 3 | 4 | 7 |
| **Net** | **revise** | **4** | **27** | **30** | **61** |

All 5 critics converged on "revise, not block." No architectural claim is being weakened without naming. The 4 criticals all cluster on security foundations (system-zone, NoAuth default, supply-chain, `requires` semantics) and all have cheap Phase 1 mitigations that were landed in this triage.

## Decisions explicitly escalated to Ben

Before triaging, three decisions were surfaced for Ben's input:

| # | Decision | Ben's choice |
|---|---|---|
| 1 | `requires` property enforcement model: declared + actually-checked (option A) vs declared-only with Phase-2 tightening (option B) | **A** (declared + actually-checked). Fixes security-critical #4; correct model for Phase 6 AI-agent attack surface. |
| 2 | Evaluator frame model: `Vec<ExecutionFrame>` + indices (option A) vs arena + IDs (option B) | **A** (indices). Idiomatic Rust; arena is a Phase-2 option if tracing snapshots demand it. |
| 3 | Change-stream placement: trait in `benten-graph`, tokio impl in `benten-engine` (option A) vs accept tokio in `benten-graph` (option B) | **A** (trait in graph, impl in engine). Preserves thin-engine philosophy; three critics converged on this. |

## Triage table

All 61 findings dispositioned. Zero disagreements. Six named compromises (documented explicitly rather than silently deferred).

### Critical findings (4 — all security-auditor, all Phase-1-fixable)

| # | Area | Disposition | Applied where |
|---|---|---|---|
| SC1 | System-zone unenforced until Phase 2 | **Fix now (stopgap).** `WriteContext::is_privileged` flag in `benten-graph`. System-label rejection at write-path. New deliverable N8, new error `E_SYSTEM_ZONE_WRITE`, new test `user_operation_cannot_write_system_labeled_node`. | Plan §2.2 G7/G3-A, §2.6 N8, §4.1 security tests; ERROR-CATALOG.md addition |
| SC2 | NoAuthBackend silent default | **Fix now.** Startup info-log on NoAuth use; new `Engine::builder().production()` refusing NoAuth (N5-b); new doc `SECURITY-POSTURE.md` (T11). Test `engine_builder_production_refuses_noauth`. | Plan §2.6 N5/N5-b, §2.8 T11, §4.1 security tests |
| SC3 | Zero supply-chain CI | **Fix now.** New deliverable T10: cargo-audit + cargo-deny + cargo-deny.toml (MIT/Apache/BSD/CC0 allowlist) + `cargo build --locked` verify + yank-response protocol in CONTRIBUTING.md + weekly audit workflow. | Plan §2.8 T10, Appendix A row added, §4.6 cross-cutting test |
| SC4 | `requires` property attack surface | **Fix now (option A per Ben).** P6 prose clarifies: `requires` is the minimum declared; evaluator also checks each primitive's effective capability at call-time. Excess-operations denied individually. Tests `handler_with_understated_requires_denies_excess_writes`, `handler_cannot_escalate_via_call_attenuation`. | Plan §2.4 P6, §4.1 security tests |

### Major findings (27) — disposition summary

All fixed or landed as named compromises. Highlights:

- **Change-stream tokio leak** (architect + philosophy + security overlap) → **option A**: `ChangeSubscriber` trait in `benten-graph`, tokio-broadcast impl in `benten-engine`, sync-callback wrapper for WASM. Plan §2.2 G7, §2.6 N4.
- **Evaluator frame borrow trap** (architect) → **option A**: indices not references. §2.5 E2, G6-C.
- **CALL + transaction semantics unspecified** (architect + code-reviewer) → **Fix:** CALL enters a nested transaction scope; `isolated: true` means capability context is attenuated but transaction state is inherited. Explicit in §2.5 E3 + §2.6 N6.
- **IVM shared `View` trait missing** (architect) → **Fix:** add to `benten-ivm`; I1-I7 implement it. Phase 2 generalization slots in cleanly. §2.3 I1.
- **Error-catalog crate split** (architect) → **Named compromise:** keep in `benten-core` for Phase 1; revisit at Phase 2 if coupling surfaces.
- **E_PRIMITIVE_NOT_IMPLEMENTED missing + invariant codes misaligned** (code-reviewer) → **Fix:** 11 codes added; 3 codes (E_INV_SANDBOX_NESTED, E_INV_SYSTEM_ZONE, E_INV_ITERATE_BUDGET) marked "Phase: 2" with pointers to their Phase-1 stopgap codes. The codes themselves stay reserved (stable identifiers for Phase 2 use) — they just aren't fired by Phase 1 code. `E_INV_ITERATE_MAX_MISSING` stays Phase 1 (registration-time enforcement of ITERATE's required `max` property).
- **TRANSFORM grammar** (code-reviewer + security + philosophy) → **Fix:** new deliverable T12 (`docs/TRANSFORM-GRAMMAR.md`), allowlist semantics, full rejection-test class, fuzz harness.
- **IVM stale-reader + transaction edge cases** (code-reviewer + dx-optimizer) → **Fix:** explicit stale-as-error semantics, per-tx IVM barrier on `engine.call` (async opt-out via `call_async`), nested-transaction rejection with `E_NESTED_TRANSACTION_NOT_SUPPORTED`.
- **TOCTOU window** (security) → **Named compromise:** cap snapshot refreshed at commit boundaries, CALL entries, ITERATE batch boundaries (default 100 iters). Window size documented. `E_CAP_REVOKED_MID_EVAL` reserved. Phase 2 tightens via Invariant 13.
- **Read existence visibility** (security) → **Option A** (leaky-but-honest, `E_CAP_DENIED_READ`). Phase 3 revisits for sync threat model.
- **Napi input validation** (security) → **Fix:** new deliverable B8 (size + depth + CID shape + invariant pre-check). Error code `E_INPUT_LIMIT`. Tests `napi_rejects_*`.
- **TRANSFORM parser escape hatches** (security) → **Fix:** allowlist BNF + 15-class rejection tests + fuzz. Landed under T12.
- **Attribution fields** (security) → **Fix:** `ChangeEvent` + `TraceStep` gain `actor_cid`, `handler_cid`, `capability_grant_cid` fields now. NoAuth populates `noauth:<session-uuid>`. Phase 2 Invariant 14 tightens without schema migration.
- **UCAN stub error routing** (security) → **Fix:** `E_CAP_NOT_IMPLEMENTED` distinct from `E_CAP_DENIED`; routes to `ON_ERROR` not `ON_DENIED`; message names Phase 3 + NoAuth alternative.
- **Drift-detector failure mode** (security) → **Fix:** required status check, bidirectional comparison, `ErrorCode::Unknown(String)` fallback, test `drift_detector_fails_on_missing_rust_variant`.
- **Thinness test missing without_versioning** (philosophy) → **Fix:** `EngineBuilder::without_versioning()` added; test renamed `thinness_no_ivm_no_caps_no_versioning_still_works`.
- **Subgraph CID order-independence untested** (philosophy) → **Fix:** proptest `prop_subgraph_cid_order_independent`; canonicalization rule published (Nodes sorted by CID; Edges sorted by `(source_cid, target_cid, label)`).
- **Invariant 8 iteration budget unnamed compromise** (philosophy) → **Named compromise:** hardcoded `MAX_ITERATE_NEST_DEPTH = 3` registration-time stopgap; full Invariant 8 is Phase 2. Explicit in §2.5 E5 + Rank 10.
- **DX: IVM race in exit criterion** (dx) → **Fix:** per-tx IVM barrier on `engine.call` default; `call_async` opt-out.
- **DX: install latency** (dx) → **Fix:** B8 prebuilds matrix; CI gate cold-install < 60s.
- **DX: TRANSFORM + call-stack error attribution** (dx) → **Fix:** new B9 (DSL source-map + `error.dslLocation`).

### Minor findings (30)

All fix-now dispositions (doc edits, test additions, small refactors). Full detail in the R1 Triage Addendum section of the plan file. None deferred.

## Named compromises (6)

1. **Invariant 13 TOCTOU window:** Phase 1 checks caps at commit / CALL entry / ITERATE batch boundary. Revocation during long ITERATE visible only at batch boundaries. Phase 2 tightens per-operation.
2. **`E_CAP_DENIED_READ` leaks existence** (Option A). Phase 3 sync revisits with per-grant `existence_visibility`.
3. **Error-code enum stays in `benten-core`** for Phase 1. Phase 2 may extract to `benten-errors` crate if coupling surfaces.
4. **WASM runtime still Phase 2.** T8 is compile-check only. Browser-context default backend (not NoAuthBackend) is a Phase 2 commitment.
5. **Per-capability write rate limits.** Phase 1 records `benten.ivm.view_stale_count{view_id}` metric; Phase 3 enforces per-peer rates when sync ships.
6. **BLAKE3 128-bit collision-resistance assumption.** Phase 1 usage doesn't rely on full preimage resistance. Phase 3 UCAN-by-CID paths document the assumption in `SECURITY-POSTURE.md`.

## Disagreements

None across all 61 findings.

## Post-triage deliverable inventory

New / changed in Appendix A since the R1 pass:

| Deliverable | Group | Origin |
|---|---|---|
| N5-b `Engine::builder().production()` | G7 | Security Critical #2 |
| N8 write-context privilege flag + system-label rejection | G3-A + G7 | Security Critical #1 |
| T10 supply-chain CI | G8-C | Security Critical #3 |
| T11 `docs/SECURITY-POSTURE.md` | G8-C | Security Critical #2 |
| T12 `docs/TRANSFORM-GRAMMAR.md` | G6-B | Code-reviewer + security + philosophy |
| B8 napi input validation | G8-A | Security major |
| B9 DSL source-map + error.dslLocation | G8-B | DX major |

## Pre-R2 actions

1. Update `docs/ERROR-CATALOG.md` with the 9 additions and 4 removals named in the plan's R1 addendum.
2. Verify pipeline stays green (no Rust-code edits this pass; plan-doc + ERROR-CATALOG only).
3. Commit the R1 triage + plan revision + ERROR-CATALOG update as one slice.
4. Dispatch R2: `benten-test-landscape-analyst` agent (JIT-create before dispatch).

## Ready for R2

With the R1 addendum and these catalog updates, the plan is R2-ready. R2 (test-landscape analyst) will expand §4 into a per-public-API test plan, seeded with:

- The 8 security-class tests from the new §4.1 subsection
- The 5 critical-finding tests
- The allowlist-BNF-grammar rejection class (15+ cases)
- The attribution schema changes
- The six named compromises (each needs a regression test confirming the compromise is scoped as documented)
