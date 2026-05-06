# Phase 2 Backlog

**Status (2026-05-03):** Phase 2 closed. Phase 2a closed at tag `phase-2a-close` (cb49554, 2026-04-25); Phase 2b closed at tag `phase-2b-close` (3d0f018, 2026-05-03). This document is now an audit artifact: every item below carries its closure status (CLOSED-IN-PHASE-2A / CLOSED-IN-PHASE-2B / PARTIAL-WITH-PHASE-3-CARRY / OPEN-WITH-PHASE-N+-DESTINATION). Items still open carry an explicit named destination per HARD RULE rule-12. The forward-looking surface is now [`docs/future/phase-3-backlog.md`](phase-3-backlog.md).

**Original scope distinction from `docs/future/README.md`:**
- `future/` = exploratory proposals, may or may not ship (e.g. `benten-runtime.md`, `compute-marketplace.md`).
- `phase-2-backlog.md` (this file) = committed deferrals with concrete Phase 1 references. Every item here was expected to land in Phase 2 or had a clear trigger for earlier landing.

**Phase 2 scope anchor:** `docs/FULL-ROADMAP.md` §Phase 2 — "Evaluator completion + WASM SANDBOX + remaining invariants." Phase 2 was split into Phase 2a + Phase 2b during Phase 2a pre-R1 (2026-04-21) on review-lens-coherence grounds: Phase 2a covered evaluator completion + debt close + 4 of 6 deferred invariants; Phase 2b covered SANDBOX + WASM + compute + the remaining 2 deferred invariants. Both sub-phases shipped at named tags as noted above.

---

## 1. Architectural deferrals

### 1.1 `benten-eval` → `benten-graph` dependency break (arch-1) — **CLOSED-IN-PHASE-2A**

The dep break landed in Phase 2a (G1-A / G1-B). `crates/benten-eval/Cargo.toml` no longer depends on `benten-graph`; the comment "benten-graph intentionally absent — arch-1 dep-break" pins the boundary. `EvalError` no longer surfaces `GraphError`; storage errors are caught at the `PrimitiveHost` boundary and mapped to `HostError` (`crates/benten-eval/src/host_error.rs`), which carries an opaque `Box<dyn StdError>` source so no `benten-graph` type leaks.

### 1.2 File splits not taken in Phase 1

- `crates/benten-graph/src/transaction.rs` (711 lines). Skipped because `PendingOp` + `TxGuard` + `Transaction` form a single cohesive state-machine and splitting them separates a data type from the code that produces and consumes it. Re-evaluate if Phase 2 adds enough transaction machinery (retry, MVCC read-transaction, streaming commit) that the file crosses ~1200 lines — at that size a logical seam probably emerges.
- `crates/benten-engine/src/engine.rs` was split in Phase 1 (5d-K) into engine.rs + 4 sibling modules (crud, caps, views, diagnostics). The orchestration core is still ~1600 lines dominated by `dispatch_call_inner` and `subgraph_for_crud` — Phase 2 may separate subgraph-construction from dispatch-walking once those shapes settle.

---

## 2. Deferred primitives (4 of 12) — **ALL CLOSED**

All four executors shipped at named tags. Per-primitive Phase-3-residual enhancements (per-handler tunable config, durable bytes registry, handler-id-router, stream end-to-end pins) are catalogued in [`phase-3-backlog.md`](phase-3-backlog.md) §6 + §7.

| Primitive | Closure status |
|-----------|----------------|
| `WAIT` | **CLOSED-IN-PHASE-2A.** Serializable `ExecutionStateEnvelope` (DAG-CBOR), time-source abstraction (`benten-eval::time_source` with `uhlc::HLC` backing), 4-step resume protocol with payload-CID + principal + subgraph-pin + capability re-check. Production runtime live; cross-process resume metadata persistence (`cap_snapshot_hash`) carries to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.1.4. |
| `STREAM` | **CLOSED-IN-PHASE-2B.** `BoundedSink` chunk-sink scheduler at `crates/benten-eval/src/chunk_sink.rs`; back-pressure semantics; partial-output emission via napi-rs ReadableStream bridge. Per-handler chunk-count tunables carry to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.1.5; STREAM end-to-end test pins at [`phase-3-backlog.md`](phase-3-backlog.md) §7.3.A.2. |
| `SUBSCRIBE` (user-visible op) | **CLOSED-IN-PHASE-2B.** `ActiveSubscription` at `crates/benten-eval/src/primitives/subscribe.rs`; user-visible SUBSCRIBE primitive; production-runtime change notification path. DSL-args↔eval-properties parity meta-test + handler-id-router carry to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.10. |
| `SANDBOX` | **CLOSED-IN-PHASE-2B.** Full `wasmtime` integration (v43.0.2 post CVE-2025 bump), fuel metering, per-subgraph fuel budgets, no re-entrancy, output size limits via `CountedSink`, capability-derived host-function manifest. See [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) Compromise #4 (CLOSED) for the ESC matrix; Phase-3 SANDBOX TS-bridge work catalogued at [`phase-3-backlog.md`](phase-3-backlog.md) §6.6. |

---

## 3. Deferred invariants (6 of 14) — **ALL ACTIVE**

All 14 invariants are enforced at `phase-2b-close`. See [`docs/INVARIANT-COVERAGE.md`](../INVARIANT-COVERAGE.md) as the SoT for per-invariant enforcer + test pins.

| Invariant | Closure status |
|-----------|----------------|
| 4 — SANDBOX nest depth | **CLOSED-IN-PHASE-2B.** Both arms active: registration-time `validate_registration` + runtime `AttributionFrame.sandbox_depth` threading wired at R6FP-G1 (PR #62). Adversarial integration test (`sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied`) stays `#[ignore]`'d pending the `testing_call_engine_dispatch` host-fn helper carried to [`phase-3-backlog.md`](phase-3-backlog.md) §7.3.A.7. |
| 7 — SANDBOX output size | **CLOSED-IN-PHASE-2B.** PRIMARY (`CountedSink::write` via host-fn trampoline) + BACKSTOP (`CountedSink::backstop_check` at primitive boundary). Default 1 MiB; `SandboxArgs.outputLimitBytes` overrides per-call. |
| 8 — cumulative iteration budget (multiplicative through CALL / ITERATE) | **CLOSED-IN-PHASE-2A.** `BudgetTracker` per evaluator step in `crates/benten-eval/src/invariants/budget.rs`; default registration-time bound `DEFAULT_INV_8_BUDGET = 500_000`. The Phase-1 scalar budget (`DEFAULT_ITERATION_BUDGET = 100_000`) remains as a runtime backstop. |
| 11 — system-zone labels unreachable from user operations | **CLOSED-IN-PHASE-2A; EXTENDED-IN-PHASE-2B.** Three-layer defence: registration-time literal-CID rejection in `benten-eval::invariants::system_zone`, runtime resolved-label probing in `benten-engine::primitive_host`, storage-layer guard `benten-graph::redb_backend::guard_system_zone_node`. Phase 2b G6-A added SUBSCRIBE-pattern validation (`E_INV_11_SYSTEM_ZONE_READ`). |
| 13 — immutability after registration | **CLOSED-IN-PHASE-2A.** `WriteAuthority` (User / EnginePrivileged / SyncReplica) firing matrix at `crates/benten-engine`; User re-puts of an already-persisted CID fire `E_INV_IMMUTABILITY`; privileged dedup paths return `Ok(cid)` without emitting ChangeEvents (named compromise documented in [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md)). |
| 14 — causal attribution on every evaluation | **CLOSED-IN-PHASE-2A.** `evaluator::attribution` runtime threading + `ATTRIBUTION_PROPERTY_KEY` registration check; every executed `TraceStep` carries an `AttributionFrame` naming actor + handler + head-of-chain grant CID. |

---

## 4. Security posture follow-ups

### 4.1 Option C (Compromise #2) — evaluator-path READ gating — **CLOSED-IN-PHASE-2A**

Threaded into the READ primitive's execute path; `crud:post:get` dispatched through `Engine::call` honours Option C end-to-end. See [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) §Compromise #2.

### 4.2 Change-stream subscribe bypasses `check_read` — **CARRIED-FORWARD-TO-PHASE-3**

Phase 2b wired the SUBSCRIBE delivery-time cap-recheck closure to consult the engine's `is_actor_active(&actor_cid)` flag (revoked actors stop receiving events) but per-event read-check gating is bounded by the in-process actor-active set — see [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) §Compromise #2 + §"Change-stream subscription bypasses capability read-checks." Per-subscriber cap-shape filtering (granular grant resolution at delivery time) carries to Phase 3 alongside the durable grant-store + UCAN backend lift; tracked at [`phase-3-backlog.md`](phase-3-backlog.md) §3.2.

### 4.3 Compromise #6 — BLAKE3 post-quantum reconsideration — **OPEN-WITH-PHASE-N+-DESTINATION**

Current posture stands: BLAKE3-256 → 128-bit classical collision resistance, 64-bit under Grover. Revisit if post-quantum transitions become a committed requirement. Options D (dual-hash) and the hybrid boundary-translation approach remain in [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) §"Hash algorithm choice." Long-tail destination (no specific phase target).

---

## 5. IVM deferrals

### 5.1 Generalized Algorithm B — **PARTIAL-WITH-PHASE-3-CARRY**

Phase 2b wave-8h production-registered Algorithm B at `Engine::create_user_view` for the 5 canonical view IDs `AlgorithmBView` supports natively. User-defined view IDs declaring `Strategy::B` continue to fall back to `ContentListingView` silently (bounded — the 5 canonical IDs cover all in-tree user-view patterns). Generalised user-defined Algorithm B handlers + a drift-detector for "declared B but registered ContentListingView" carry to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §5.1 + §5.2.

### 5.2 `E_IVM_PATTERN_MISMATCH` firing (spec-to-code audit §5.5) — **CLOSED-IN-PHASE-2B**

The five hand-written views in `crates/benten-ivm/src/views/*.rs` now construct `ViewError::PatternMismatch` (→ `E_IVM_PATTERN_MISMATCH`) on out-of-pattern queries rather than over-answering silently — verified across capability-grant / event-handler-dispatch / content-listing / governance-inheritance / version-current views.

### 5.3 `benten.ivm.view_stale_count` metric wiring — **CLOSED-IN-PHASE-2A**

`metrics_snapshot()` now emits a real `benten.ivm.view_stale_count` value computed via the subscriber tally — see `crates/benten-engine/src/engine_diagnostics.rs::metrics_snapshot` (the Phase-2 wire-up the comment block calls out is in production).

### 5.4 IVM rebuild from event log — **OPEN-WITH-PHASE-3-DESTINATION**

Phase 2b views still rebuild only by re-applying committed `ChangeEvent`s from engine startup. Durable view snapshots (so a crash doesn't force a full re-walk) carry to Phase 3 alongside the generalised Algorithm B lift — see [`phase-3-backlog.md`](phase-3-backlog.md) §5.

---

## 6. `benten-core` + content-addressing

### 6.1 `Cid::from_str` implementation (spec-to-code audit §5.9) — CLOSED (2026-04-20)

Landed in Phase 1 as the close-out of R7 audit finding F-R7-004: `Cid::from_str` now decodes the base32-lower-nopad multibase form (the inverse of `Cid::to_base32`) and hands the resulting bytes to `Cid::from_bytes`, so string-path callers get the same three typed failure classes (`CoreError::InvalidCid` / `CidUnsupportedCodec` / `CidUnsupportedHash`) as byte-path callers. The catalog fix-hint for `E_CID_PARSE` was updated to drop the "Phase 2 path still stubbed" caveat. Tests: `crates/benten-core/tests/cid_from_str.rs` (roundtrip + canonical fixture + prefix / alphabet / length rejections) plus a lib-level `cid_string_roundtrip` unit test.

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

### 7.1 Wall-clock + iteration TOCTOU delegation — **CLOSED-IN-PHASE-2A**

Phase 2a hardened five TOCTOU points (transaction commit, CALL entry, every N iterations of ITERATE, WAIT resume, wall-clock revocation ceiling) with a dual monotonic + HLC clock source. See [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) Compromise #1 for the spec.

### 7.2 UCAN backend — **CARRIED-FORWARD-TO-PHASE-3**

`benten-caps::UCANBackend` remains a stub. Full UCAN chain validation + delegation ships in Phase 3 `benten-id` alongside Ed25519 / DID / VC — see [`phase-3-backlog.md`](phase-3-backlog.md) §2.1.

### 7.3 Cross-process WAIT resume metadata persistence — **CARRIED-FORWARD-TO-PHASE-3**

Phase 2a's resume protocol re-checks capabilities against the current-process policy snapshot ([`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) Compromise #10 — closed at Phase 2b for in-process semantics). The cap-snapshot-hash + persisted-policy metadata that lets the resume path assert against historical state across process boundaries did not land in Phase 2b; it carries to Phase 3 alongside the durable grant-store + UCAN backend lift — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.1.4 / §2.2.

### 7.4 Durable grant-store + SUBSCRIBE delivery-time cap-recheck — **PARTIAL-WITH-PHASE-3-CARRY**

Wave-8c-subscribe-infra (Phase-2b) wired the SUBSCRIBE delivery-time cap-recheck closure to consult the engine's `is_actor_active(&actor_cid)` flag — a structural revoked-actors set the engine maintains in-process. This satisfies the D5 contract minimum (revoked actors stop receiving events) but remains bounded:

- The cap-recheck does NOT consult cap-shape grants (e.g. "actor still has `host:compute:log` grant for this anchor"); it only consults the actor-active flag.
- Deeper grant-resolution requires the `GrantBackedPolicy` to expose a SUBSCRIBE-shape grant query (i.e. "is grant G still valid for actor A against anchor X at HLC time T?") — not present in Phase-2b.
- The grant-store itself is in-memory per Phase-2b posture.

**Phase-3 lift:** when the durable grant-store lands (alongside UCAN backend, `benten-id`, Ed25519/DID/VC), the SUBSCRIBE cap-recheck closure threads the grant-shape query so a partial-revoke (e.g. specific grant revoked but actor still active) cancels the affected subscription path. Pairs with §7.1 wallclock TOCTOU work + §7.2 UCAN backend lift. See [`phase-3-backlog.md`](phase-3-backlog.md) §3.2.

---

## 8. DX / tooling

### 8.1 Dev server with hot reload — **CLOSED-IN-PHASE-2B**

`packages/engine-devserver` (`@benten/engine-devserver`) shipped wave-8f as the napi-rs-backed `BentenDevServer` TypeScript wrapper that wraps a real `Engine` and exposes `replaceHandler` / hot-reload semantics through `Engine::register_subgraph_replace`. The diagnostic CLI (`tools/benten-dev`) carries the `inspect-state <path>` subcommand for serialized `ExecutionStateEnvelope` introspection.

### 8.2 Cross-doc numeric-claim drift lint — **OPEN-WITH-PHASE-N+-DESTINATION**

Anytime / low-priority. A small CI lint that greps `docs/*.md` for numeric performance claims and compares them against `ENGINE-SPEC.md` §14.6 would close the loop; the "verify, don't trust docs" review discipline catches most occurrences in the meantime. No specific phase target.

### 8.3 Per-item `missing_docs` sweep — **OPEN-WITH-PHASE-3-DESTINATION**

The `benten-eval` `warn(missing_docs) + allow(missing_docs)` escape hatch did not get retired in Phase 2; per-item sweep across the post-Phase-2b public surface (~120+ items) carries to Phase 3 — a concrete pre-public-doc-rewrite task pairing with the DSL-SPECIFICATION public rewrite scope. Target: drop the `allow(missing_docs)` entirely.

---

## 9. Performance

### 9.1 macOS APFS fsync floor — **CLOSED-IN-PHASE-3-G13-E**

`DurabilityMode::default()` flipped from `Immediate` to `Group` at Phase-3 R5 wave-3 G13-E (engine surface posture; redb v4 backend still collapses Group → `Durability::Immediate` until upstream redb grows native batched-commit support). Benchmark CI workflow `.github/workflows/bench.yml` promoted from informational to required at the same wave + grew the APFS-relevant CRUD fast-path timing benchmarks. Closes [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) Compromise #12. Pinned by [`crates/benten-graph/tests/durability_default.rs::durability_mode_group_default_for_crud_fast_path`] (default-flip), [`crates/benten-graph/tests/security_posture_compromise_12_marked_closed`] (CLOSED marker), and [`crates/benten-graph/tests/crud_fast_path_apfs_timing_within_target`] (informational wall-clock gate; the criterion bench is the authoritative perf signal).

### 9.2 Subgraph AST cache — **PARTIAL-WITH-PHASE-3-CARRY**

The AST-cache template path lands at `Engine::call` (cache stores the static shape of the subgraph for a registered handler); per-call parsing of the subgraph itself remains. Full wire-up carries to Phase 3 — the `engine.rs` comment block at the dispatch site notes "Phase-2 completes the AST-cache wire-up" still applies.

---

## 10. CI / workflow deferrals (from 2026-04-22 maturity audit)

Source: `.addl/phase-2a/ci-maturity-audit-2026-04-22.md` + the decisions companion `.addl/phase-2a/ci-decisions-2026-04-22.md`. Reordered by Ben, 2026-04-22.

**Phase 2a CI items** (CodeQL, branch-protection-as-code, SHA-pinning) land as the Phase 2a §3.1 CI hardening pass — see `.addl/phase-2a/00-implementation-plan.md` §3.1. The 5 publication-coupled items (cargo-semver-checks, napi prebuilt publish, release-plz, SLSA attestation, SBOM) were rescoped on 2026-04-25 into §3.2 "Publish-readiness pass" and deferred to whichever phase actually publishes (provisional: Phase 8/9+ OSS launch) — see `.addl/phase-2a/00-implementation-plan.md` §3.2.

### 10.1 Phase 2b CI additions — **PARTIAL: WORKFLOWS LAND, BASELINES CARRY TO PHASE 3**

| Item | Closure status |
|---|---|
| `cargo-public-api` tracking | Workflow shipped; baseline file commit carries to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.3.A.9. |
| `cargo-vet` trust metadata | Workflow shipped; baseline data commit carries to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.3.A.9. |
| `wasm-conformance.yml` — malicious/boundary handler fixture suite | **CLOSED-IN-PHASE-2B.** Adversarial fixture suite shipped alongside SANDBOX runtime; ESC matrix (16 attempted-escape vectors) catalogued in [`SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) Compromise #4. |
| E2E tests — headless browser vitest against scaffolded handlers | Browser-bundle artifact baseline carries to Phase 3 — see [`phase-3-backlog.md`](phase-3-backlog.md) §7.3.A.9. |
| Weekly scheduled security sweep expansion (`cargo-vet check` in supply-chain.yml Monday cron) | Carries with `cargo-vet` baseline above. |

### 10.2 Phase 3 CI additions

| Item | When within 3 |
|---|---|
| Multi-peer networked CRDT test — 3+ iroh peers, assert Loro merge convergence across write patterns | 3 entry — CRDT correctness is the 3 headline invariant |
| Phase-3-specific chaos workflow (network partition simulation via `tc netem`) | 3 mid |
| Self-hosted runner posture document | 3 (don't buy capacity until a 3 test actually needs it) |
| Additional cross-target coverage (illumos, FreeBSD, linux-musl) | When actual Phase 3 users appear on those targets |

### 10.3 Phase 9+ (OSS contributor-volume era)

| Item | Trigger |
|---|---|
| Auto-labeler / PR triage bots | Issue volume arrives |
| Preview deploys for docs.benten.ai | When docs site is live |
| Auto-merge for Dependabot patch-level bumps | When contributors arrive (revisit — not now while solo) |

### 10.4 Anytime / backlog

| Item | Notes |
|---|---|
| `cargo-outdated` weekly report | Nice-to-have; ratchet if Dependabot PR volume feels noisy |
| Ratchet `PROPTEST_CASES` 1024 → 4096 nightly | Free on public runners, costs ~0 to flip |
| Ratchet `bench.yml --measurement-time` 2s → 5s | Better measurement noise floor |
| Run `mutants.yml` twice weekly instead of weekly | Free on public runners |
| `benten-eval` `expr/builtins.rs` random/timestamp surface determinism on browser-target | OOS-deferred from Phase-3 R1 br-r1-12 (browser-runtime lens; OOS-NAMED-DESTINATION per `r1-revision-triage.md`); flag to R6 phase-close determinism-verifier deep-sweep if any expr-builtin surface reachable on browser depends on a non-deterministic source. Closes phantom-destination per HARD RULE rule-12 clause-b (R4 R2 pattern-induction br-r4-r1-9). |

---

## 11. In-code `TODO(phase-2-*)` markers — **PARTIAL: STRUCTURAL ITEMS CLOSED, RESIDUALS REMAIN**

At Phase 2b close, ~63 `TODO(phase-2-*)` and ~40 `phase-2-*` markers remain across the codebase. The structural items (arch-1 dep break, primitive executors, deferred invariants, IVM pattern-mismatch firing, devserver hot reload, view_stale_count metric) are CLOSED per the sections above. The residuals are concrete implementation hints (uhlc deeper integration, host-error DAG-CBOR envelope versioning, AST-cache wire-up, anchor-store consolidation, etc.) that pair with their respective Phase-3 entries in [`phase-3-backlog.md`](phase-3-backlog.md). Re-grep at any time:

```bash
grep -rn "TODO(phase-2\|phase-2-" crates/ bindings/ packages/ tools/
```

---

## Revival / escalation

Phase 2 closed. Phase 2a closed at tag `phase-2a-close` (cb49554, 2026-04-25); Phase 2b closed at tag `phase-2b-close` (3d0f018, 2026-05-03). Items not closed by Phase 2 either rolled into [`phase-3-backlog.md`](phase-3-backlog.md) (next-phase) or are catalogued here as OPEN-WITH-PHASE-N+-DESTINATION. The forward-looking surface is now the phase-3 backlog; the Phase-3 plan-doc opening checklist lives at [`phase-3-backlog.md`](phase-3-backlog.md) §8.

See `docs/DEVELOPMENT-METHODOLOGY.md` Pattern 1 (After every review, fix or defer — never "noted") + the HARD RULE in CLAUDE.md (default disposition for any finding is FIX-NOW; only OUT-OF-SCOPE / BELONGS-NAMED-NOW / DISAGREE-WITH-EXPLANATION are valid non-fix-now dispositions).
