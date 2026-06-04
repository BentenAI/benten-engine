# Phase 1 — Core Engine

**Status:** SHIPPED at tag `phase-1-close` (2026-04-21).
**Closed at:** HEAD `f69830b` ("docs(claude-md): mark Phase 1 COMPLETE").
**Canonical fixture CID:** `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` (stable across all platform/MSRV/wasm legs).

---

## § 1. Narrative journal

Phase 1 opened with the engine as an idea documented in `docs/ENGINE-SPEC.md` and `CLAUDE.md`, plus a 6-crate spike (`SPIKE-phase-1-stack`) that had validated the core determinism story: BLAKE3 over DAG-CBOR canonical encoding, CIDv1 envelope, redb as the storage backend, napi-rs v3 dual-target compilation surface (native + wasm32). The spike had answered the high-stakes "can we actually do this in Rust?" question — content hashing was deterministic intra-process, cross-process, and under wasm32-wasip1 — but had punted everything else to Phase 1 ADDL. The phase started, then, with a known-buildable foundation and a long list of unbuilt pieces: the `Edge` type wasn't designed yet, the `Float(f64)` value variant wasn't decided, the transaction primitive existed only as conceptual API sketches, the IVM crate was a one-line marker, the evaluator was absent, the capability system was a trait definition with no backends, the bindings crate exposed three functions, and the TypeScript DSL didn't exist. Phase 1's job was to turn the spike's "yes, the foundations work" into a working, embeddable, content-addressed graph engine that a developer could `npx create-benten-app` and have a CRUD handler running in under ten minutes.

The phase's defining tension was scope discipline. The original ENGINE-SPEC + roadmap had mentioned 14 primitives over time; the 2026-04-14 reconciliation that opened pre-R1 froze the canonical set at **12 primitives** — READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM — dropping VALIDATE (composable from BRANCH+TRANSFORM+RESPOND) and GATE (capability checking via the `requires` property), and adding SUBSCRIBE (reactive change notification) and STREAM (partial output with back-pressure). Phase 1 would ship **8 of those 12 as executable**: READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT. The other four — WAIT, STREAM, SUBSCRIBE-as-user-op, SANDBOX — would have their **types defined** at registration time so subgraphs could reference them, but their executors would return `E_PRIMITIVE_NOT_IMPLEMENTED` at call time. This let Phase 2 enable them by lighting up executors without re-registering every stored subgraph — a foreshadowing of the wiring discipline that would matter throughout the phase. Similarly, Phase 1 enforced **8 of 14 invariants** (1, 2, 3, 5, 6, 9, 10, 12 — the structural ones that don't need wasmtime); invariants 4 / 7 / 8 / 11 / 13 / 14 were Phase 2 with named stop-gaps (notably `MAX_ITERATE_NEST_DEPTH = 3` for invariant 8 and `E_SYSTEM_ZONE_WRITE` for invariant 11). And IVM shipped 5 hand-written views, with generalized Algorithm B explicitly Phase 2.

The pre-R1 critic round set the tone for what followed. Two critics — `code-reviewer` and `benten-core-guardian` — returned 0 critical / 5 major / 12 minor against the implementation plan. Zero overlap between their lenses (one chased plan-coherence + ADDL-dispatch; the other chased content-hash invariant protection). Every finding was dispositioned "fix now in plan." The critic surfacing of `npm run dev` (in the headline exit criterion) vs Rank-10's "dev-server-with-hot-reload is Phase 2" forced replacement with a 6-assertion `npm test` gate; the absence of a regression test for "edge creation doesn't shift endpoint Node CIDs" forced explicit naming of `edge_creation_does_not_change_endpoint_node_cids` and `version_chain_linking_does_not_change_version_node_cids`; the missing napi-rs dual-target risk became Rank 4.5 + new deliverable T8 (`cargo check --target wasm32-unknown-unknown -p benten-napi`). The pattern that emerged — pre-R1 catches plan-coherence, R1 catches architecture quality and security — survived to later phases as the "two-tier critic" approach.

R1 was where the security shape of the engine got decided. Five critics dispatched in parallel — architect-reviewer, code-reviewer, security-auditor, dx-optimizer, benten-engine-philosophy. Net **4 critical / 27 major / 30 minor / 61 total**. All 4 criticals came from security-auditor and reshaped Phase 1's posture:

- **SC1 system-zone forgery:** Phase 1 deferred Invariant 11 to Phase 2 but stored capability grants and IVM view definitions as plain Nodes; without enforcement, a user-authored handler could WRITE to `system:CapabilityGrant` and forge a grant. Mitigation: `WriteContext::is_privileged` flag at the benten-graph write-path layer + system-label rejection (became deliverable N8, error `E_SYSTEM_ZONE_WRITE`).
- **SC2 NoAuthBackend silent default:** zero-auth as builder default with no production guardrail. Mitigation: startup info-log + `Engine::builder().production()` refusing NoAuth (became N5-b) + new doc `docs/SECURITY-POSTURE.md` (T11).
- **SC3 zero supply-chain CI:** mid-phase the core2 yank had been a live supply-chain event in the project's dependency graph and the plan had shipped no controls. Mitigation: cargo-audit + cargo-deny + license allowlist + `cargo build --locked` verification + yank-response protocol (T10).
- **SC4 `requires` property declared-vs-enforce:** archetypal "declare what I need, but the engine doesn't actually check the operations I perform" bug. Ben chose option A — declared minimum + per-primitive call-time check; tests `handler_with_understated_requires_denies_excess_writes` and `handler_cannot_escalate_via_call_attenuation`.

R1 also produced three Ben-level architectural ratifications: (1) `requires` enforcement model = A (declared + actually-checked); (2) evaluator frame model = `Vec<ExecutionFrame>` + `frame_index: usize` indices, not arena+IDs; (3) change-stream placement = `ChangeSubscriber` trait in `benten-graph` (runtime-agnostic) with tokio-broadcast impl in `benten-engine`. The third was three critics convergent — architect, philosophy, security — all flagging that putting tokio in `benten-graph` would violate the dual-target T8 promise. Six **named compromises** got documented at this point, distinguishing what Phase 1 chose not to fully fix (TOCTOU window, E_CAP_DENIED_READ existence-leak, ErrorCode-stays-in-benten-core, WASM-runtime-Phase-2, per-cap-rate-limits-Phase-3, BLAKE3-128-collision-bound) from what was deferred without naming. Zero disagreements across all 61 findings.

R2 produced the test landscape — a single-doc lead-session synthesis by `benten-test-landscape-analyst` that mapped every public API to test obligations: 201 unit + 14 proptest + 18 integration + 14 criterion + 11 Vitest + 22 security-class + 9 CI = **289 total artifacts** partitioned into 93 Rust test files across 5 R3 agents (rust-test-writer-{unit, edge-cases, security, performance} + qa-expert-integration). Each compromise got an explicit grep-able regression sub-test in `compromises_regression.rs` (e.g., `compromise_1_toctou_window_bound_at_100_iter_batch`) so future phases could grep-and-delete when underlying gaps closed. R3 dispatch followed: 5 parallel writers, each landing tests in TDD red-phase against `todo!()` stubs.

R4 ran two passes. Pass 1 returned **3 critical / 24 major / 23 minor / 50 total** (rust-test-reviewer + rust-test-coverage + qa-expert in parallel; same 3 lenses both passes). The 3 criticals: scaffolder meta-test regex `\\b` didn't escape (matched 0 occurrences); TOCTOU compromise tests had `>=149` vs `>=200` boundary inconsistency (reconciled to exact `==100` per R1's default 100-iter batch); IVM views 1/2/5 used `matches!(ViewResult::Cids(_))` only — the vacuous-pass class. The qa-expert lens caught that no `package.json` existed in `packages/engine`, `bindings/napi`, or `tools/create-benten-app` (6 Vitest files unrunnable), and that `d2_cross_process_graph.rs` admitted in its own doc comment to being drop-and-reopen in one process, not actually cross-process. Fix-pass commit `5c60c31` landed 49 of 50 items. Pass 2 returned 14 minors total, all 3 critics PASS — the convergence pattern. The R4 gate closed with 392+ tests compiling, all TDD red, ready for R5.

R5 dispatched 8 implementation groups — G1 (core hardening + version chains) → (G2 graph backend reshape + G4 capability policy in parallel) → G3 (transaction + change stream) → G5 (IVM + 5 views) → G6 (evaluator + 8 primitives + invariants + TRANSFORM) → G7 (engine orchestrator) → G8 (napi + DSL + scaffolder + CI). Each group ran 2-3 parallel `rust-implementation-developer` agents owning disjoint files, followed by mini-reviews from `code-reviewer` + the crate's guardian. The pattern was uneven by group: G1 had 1 critical from `benten-core-guardian` (content-hash invariant violation, fixed inline), G3 had **2 system-zone-bypass criticals from chaos-engineer** (system-zone guard missing on inherent put_node + put_edge paths) plus a change-stream DoS-vector concern, G4 had **2 wildcard/attenuation criticals from ucan-capability-auditor** that needed a pass-2 fix, G5 had 6 majors from ivm-algorithm-b-reviewer that all rooted in the ChangeEvent payload not carrying enough context for rebuild — fixed by widening ChangeEvent with `Option<Node>` + `edge_endpoints` + 2 new ChangeKind variants in commit `8a19445`, G6 + G7 landed clean (accept-with-minors first try; benten-eval came in at 6,158 lines clean; benten-engine/lib.rs at 1,789 lines navigable via banner-commented sections), and G8 had **4 DX criticals from dx-optimizer** (error-message ergonomics, source-map propagation, install-flow friction, scaffolder template defects) that needed addressing pre-R6.

R4b (post-implementation test re-review) was the explicit guard against the "tests pass vacuously" failure mode — same 3 R4 lenses run again, this time against R5's real implementation (453/473 Rust + 17 Vitest + 7 napi smoke). It returned mixed verdicts: rust-test-reviewer flagged 3 criticals (vacuous tests, tests passing against stub state), qa-expert flagged 2 more (exit-criterion-implementation gaps) — triggered another fix-pass before R6.

R6 ran a 12-lens quality council (architect, best-practices-2026, chaos-engineer, code-as-graph-reviewer, code-reviewer, dx-optimizer, error-detective, ivm-algorithm-b-reviewer, performance-engineer, qa-expert, refactoring-specialist, test-automator). Performance-engineer returned `FAIL_CONCERNS` with **2 perf criticals**: 2/6 §14.6 benches were stubbed (`ten_node_handler` and `view_maintenance` had bench fns but ran against shim code, not real evaluator/IVM paths). Chaos-engineer's 2 system-zone bypasses from R5-G3 carried forward as concerns to verify-fixed. The 14-agent R6 review NOTABLY MISSED the "catalogued-but-unfired error codes" pattern that R7 would later catch.

Step-5b/5c fix-passes addressed R6's findings. The notable refactoring: `benten-engine/src/lib.rs` split from 3,151 LOC → 137 LOC + 10 module files (error / builder / primitive_host / subgraph_spec / outcome / engine_diagnostics / engine_views / engine_caps / engine_crud / etc), each with cohesive content; the `benten-errors` crate was extracted from `benten-core::error_code` to satisfy Compromise #3 (ErrorCode catalog discriminants live in their own zero-Benten-dep crate, removing the cross-cutting `benten-core` edge); SECURITY-POSTURE.md got its Compromise #2 Option C closure (symmetric None on policy denial via `Engine::get_node` collapsing `CapabilityPolicy::check_read` denial onto `Ok(None)` / `Ok(vec![])` / empty-list `Outcome` — byte-identical with absence — gated by a `debug:read` capability via `engine_diagnostics::diagnose_read`).

R6b ran the redux — 13 lenses (R6's 12 + `operation-primitive-linter` which hadn't returned at R6) under the **verify-don't-trust-docs contract**. Pass-1 architect: PASS (0 findings). Chaos-engineer: pass-with-minors (0 findings) with all 4 system-zone guard sites confirmed live (RedbBackend::put_node:442, put_edge:585, Transaction::put_node, Transaction::put_edge). Performance-engineer: FAIL_CONCERNS 2 critical → both **verified closed** (ten_node_handler + view_maintenance now run real code); 4/5 majors closed; 1 residual (r6-perf-6 outbound path). Code-reviewer found that the "aspirational docstring" pattern from compromises #1 + #5 (prose-only) lived in 6 other sites at HEAD. Test-automator found 1 closure-gap (R6-TA-2) was a doc-only claim with no live workflow.

The phase had one Phase-1-only stage that didn't survive: **R7 spec-to-code-compliance audit**. The `spec-to-code-compliance` skill walked every spec document (SECURITY-POSTURE, ENGINE-SPEC, ERROR-CATALOG, CLAUDE.md decisions, plan §14.6 perf targets, QUICKSTART, DSL-SPECIFICATION, TRANSFORM-GRAMMAR, ARCHITECTURE) and verified each spec claim had a code construction site. The first run (2026-04-17, HEAD `d03f642`) found **9 gaps** including 4 catalog codes that were defined-but-unfired (`E_WRITE_CONFLICT`, `E_CID_UNSUPPORTED_CODEC`, `E_CID_UNSUPPORTED_HASH`, `E_IVM_PATTERN_MISMATCH`) — the **drift-detector verifies enum↔catalog name parity but NOT runtime reachability**, a blind spot that the 14-agent R6 had missed. The named pattern: "catalogued-but-unfired error codes." Fix-passes addressed all 4. The final R7 audit (2026-04-21, HEAD `f69830b`) verified 6 of 9 prior gaps closed end-to-end + 3 carried to Phase-2 backlog (read-path hash verification, view_stale_count metric, Cid::from_str), and found **4 new MEDIUM gaps** all in "documentation falls behind code" class: scaffolder smoke substitutes 3 of 6 plan §1 gates (capability-denial → unregistered-handler error; @mermaid-js/parser → regex over `^flowchart`; canonical fixture → roundtrip-only); DSL-SPECIFICATION.md banner says VALIDATE/GATE dropped but body still imports them; ARCHITECTURE.md says "six crates" while seven exist; CID parser fix-hint misleads about Phase-1 base32 acceptance.

The closing texture: a clean Phase 1 ship at `phase-1-close` 2026-04-21 with the canonical fixture CID stable, 7 crates landed, 8 of 12 primitives executable, 8 of 14 invariants enforced, 5 hand-written IVM views maintaining incrementally, pluggable capability policy with NoAuthBackend default + `production()` builder guard, napi-rs TS bindings exposing 23 Engine methods all wrapped in `@benten/engine`, 36 error codes in catalog / 36 TS classes / 34 Rust enum variants, 558 tests passing, `npx create-benten-app && npm test` green on a fresh laptop, and 4 medium documentation drifts to address before Phase 2 opens. Phase 1 is the foundation everything else builds on.

---

## § 2. Changelog

**Engine surface — new primitives + crates + APIs:**
- 7 crates landed: `benten-core`, `benten-graph`, `benten-ivm`, `benten-caps`, `benten-eval`, `benten-engine`, `benten-errors` (extracted late-phase to satisfy Compromise #3)
- 8 of 12 primitives executable: READ / WRITE / TRANSFORM / BRANCH / ITERATE / CALL / RESPOND / EMIT
- 4 deferred primitives type-defined at registration; executor returns `E_PRIMITIVE_NOT_IMPLEMENTED` at call time: WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX
- `Edge` type with 4-field DAG-CBOR round-trip + `Edge::cid()` content-addressing (excluded from Node hash per ENGINE-SPEC §7)
- `Value::Float(f64)` with NaN + ±Inf rejection at serialize time
- Version-chain primitives: `Anchor` + `CURRENT` / `NEXT_VERSION` edge labels + `walk_versions` + `current_version` + `append_version` (opt-in; ephemeral data doesn't pay versioning cost — thinness defended by `EngineBuilder::without_versioning()`)
- `Engine` orchestrator API: `register_subgraph` + `call` + `call_async` + `trace` + `transaction` + `snapshot` + `grant_capability` + `create_view` + `revoke_capability` + `subscribe_change_events`
- `EngineBuilder` opt-outs: `without_ivm()` / `without_caps()` / `without_versioning()` / `production()` (refuses NoAuthBackend)
- `KVBackend` trait with associated `type Error` polymorphism + `scan` returning `Box<dyn Iterator<...>>` instead of `Vec`
- `RedbBackend` with split `open_existing` vs `open_or_create`; `DurabilityMode::{Immediate, Group, Async}`
- `NodeStore` + `EdgeStore` blanket impls over `KVBackend`
- Closure-based `transaction(|tx| ...)` API (closure panics: rolled back via `catch_unwind` + re-raised; nested: `E_NESTED_TRANSACTION_NOT_SUPPORTED`)
- `ChangeSubscriber` trait in `benten-graph` (runtime-agnostic) with tokio-broadcast `ChangeProbe` impl in `benten-engine`
- `ChangeEvent { cid, label, kind, tx_id, actor_cid: Option<Cid>, handler_cid: Option<Cid>, capability_grant_cid: Option<Cid> }` — Phase-1 attribution scaffolding so Phase-2 Invariant-14 is a tightening, not a schema migration
- `CapabilityPolicy` pluggable trait + `NoAuthBackend` default + `UCANBackend` stub (`Err(NotImplemented { phase: Phase3, alternative: NoAuthBackend })`); `cap_grant_backed` policy backend
- 5 hand-written IVM views: capability-grants / event-handler-dispatch / content-listing / governance-inheritance / version-current — all implementing shared `View` trait so Phase-2 generalized Algorithm B retrofits without rewrite
- TRANSFORM expression language: hand-rolled Pratt parser + `docs/TRANSFORM-GRAMMAR.md` BNF + 30+ built-ins (Math, string, array, Date, object construction)
- `subgraph.toMermaid()` + `engine.trace()` diagnostic surfaces (behind `diag` feature flag, default OFF in `benten-eval`, ON via `benten-engine`)
- napi-rs v3 binding surface: 23 Engine methods exposed; `@benten/engine` TypeScript wrapper with crud DSL + zero-config `crud('post')` + typed errors + `error.dslLocation` source-map + `toMermaidUrl()` helper
- napi input validation: `JSON_MAX_BYTES = 1MiB` rejecting with `E_INPUT_LIMIT`
- `tools/create-benten-app` scaffolder: TS-authored, ships its own npm package, produces `my-app` with `npm test` + `npm run build` + `npm run dev` green on fresh clone

**Compromises closed:**
- **Compromise #1 (TOCTOU)** — refresh at commit / CALL entry / ITERATE-batch-boundary (default 100 iters); writes 1-100 succeed under granted cap, write 101 returns `E_CAP_REVOKED_MID_EVAL`
- **Compromise #2 (E_CAP_DENIED_READ existence-leak)** — Option C closure: symmetric `None` on policy denial + `engine_diagnostics::diagnose_read` gated by `debug:read` capability
- **Compromise #3 (ErrorCode crate split)** — `benten-errors` crate extracted; zero-Benten-dep root of dependency graph
- **Compromise #5 (per-cap write metrics)** — `benten.writes.committed.<scope>` + `benten.writes.denied.<scope>` keys emitted from `record_cap_write_*` + `metrics_snapshot` API
- **Compromise #7 (`[[bin]]` required-features)** — `benten-graph/Cargo.toml` `[[bin]]` gated with `required-features = ["test-fixtures"]`
- **Compromise #8 (PrimitiveHost sole dispatch)** — verified by §5.1 alignment matrix; no fast-path bypasses

**Compromises still open at phase-1-close (carried to Phase 2 backlog):**
- **Compromise #4** WASM runtime → Phase 2 (Phase 1 = compile-check only via T8)
- **Compromise #6** BLAKE3 128-bit collision bound → documentation-only stance in SECURITY-POSTURE.md

**Invariants newly enforced:**
- Inv 1 (DAG / cycle detection)
- Inv 2 (max depth)
- Inv 3 (max fan-out)
- Inv 5 (max nodes 4096)
- Inv 6 (max edges 8192)
- Inv 9 (determinism classification)
- Inv 10 (content hash per subgraph — registration + Subgraph::load_verified)
- Inv 12 (registration-time structural validation)
- **Inv 8 stop-gap:** registration-time `MAX_ITERATE_NEST_DEPTH = 3` + runtime `DEFAULT_ITERATION_BUDGET`
- **Inv 11 stop-gap:** `WriteContext::is_privileged` flag at benten-graph write-path layer + `E_SYSTEM_ZONE_WRITE` on user-path WRITEs to `system:`-prefixed labels
- **Inv 14 partial:** attribution captured on writes (`PendingOp::PutNode { actor_cid, handler_cid, capability_grant_cid }`); structural per-step attribution = Phase 2

**Test coverage milestones:**
- 558 tests passing at phase-1-close (453 Rust + 17 Vitest + 7 napi smoke + 81 across other surfaces)
- 14 proptests landed (parity with R2 projection; 3 deferred per R4 M16 partial-defer landed in R5 red-phase form)
- 21 of 21 Phase-1 error codes have firing sites in non-test production code (R7 reachability audit)
- 10 criterion benches: 4/6 §14.6 targets gated, 2 informational (concurrent_writers, multi_mb_roundtrip), 4 baseline-protection (hash_only, cid_parse, transform_expression_small, blake3_hash_node_small)
- 11 Vitest E2E tests across packages/engine + bindings/napi + scaffolder
- 9 CI workflows: multi-arch matrix (T4) + MSRV (T5) + WASM-runtime (T6) + napi-wasm32 compile-check (T8) + cross-leg determinism gate (T9) + supply-chain (T10: cargo-audit + cargo-deny + lockfile-verify + weekly-advisory-scan) + drift-detector (T7) + cold-install gate + cross-process determinism

**Tooling / CI:**
- `cargo-nextest` adopted as test runner
- `cargo-audit` + `cargo-deny` in CI, fail on HIGH/CRITICAL RUSTSEC advisories; license allowlist (MIT / Apache-2.0 / BSD-3 / CC0)
- ERROR-CATALOG drift detector (`scripts/drift-detect.ts`): bidirectional Catalog ↔ Rust enum ↔ TS types; required CI status check
- T9 cross-leg determinism gate: each platform leg's CID byte-equal to every other leg
- Cold-install latency gate: `npm install @benten/engine` < 60s on the reference runner

**Docs created:**
- `docs/SECURITY-POSTURE.md` (T11) — 8 named compromises + change-stream disclosure + napi input-limit + BLAKE3 choice
- `docs/TRANSFORM-GRAMMAR.md` (T12) — Pratt-precedence BNF + 30+ allowlisted built-ins + 38-item denylist
- `docs/ERROR-CATALOG.md` updated to 44 active Phase-1 codes + 11 Phase-deferred (Phase 2/3 reservations)
- `docs/QUICKSTART.md` post-Phase-1 update: `npx create-benten-app` + `crud('post')` zero-config workflow + Option C `diagnose_read` worked example
- `LICENSE-MIT` + `LICENSE-APACHE` + `SECURITY.md` + `dependabot.yml` (R6 best-practices-2026 closure)

**Phase 1 shipped at tag `phase-1-close` (`f69830b`, 2026-04-21).**

---

## § 3. Key takeaways — what to remember

**What this phase was fundamentally about:**
Phase 1 turned a 6-crate spike into a working content-addressed graph engine that a developer can use without knowing Rust exists. The strategic frame was: prove the architectural thesis ("a thin code-as-graph engine with pluggable capability hook + IVM subscriber can out-compete PostgreSQL+AGE on hot-path CRUD") on a tight subset (8 of 12 primitives, 8 of 14 invariants, 5 hand-written IVM views) so Phase 2 could extend without re-litigating foundations. Every Phase-1 commitment was sized so Phase 2 enables additional capability via wiring, not via rewrite or re-registration.

**The hardest problems we hit:**
- **The change-stream tokio-leak debate (R1 architect M1).** Three R1 critics convergently flagged that putting `tokio::sync::broadcast` in `benten-graph` would force tokio into every downstream crate and break the dual-target T8 promise. Resolution: define `ChangeSubscriber` trait in `benten-graph` (runtime-agnostic), tokio-broadcast impl lives in `benten-engine`, sync-callback wrapper for WASM. This required reshaping G7 and G2 mid-plan but preserved the thin-engine architecture.
- **Security-zone protection without invariant 11.** Phase 1 deferred Inv 11 to Phase 2, but Phase 1 already stored capability grants and IVM view definitions as plain Nodes. SC1 forgery attack was blocking. Phase 1 stop-gap (`WriteContext::is_privileged` flag at write-path + system-label rejection) became deliverable N8 + error `E_SYSTEM_ZONE_WRITE`. The pattern of "Phase-1 stop-gap with clear Phase-N upgrade path" was repeated for Inv 8 (`MAX_ITERATE_NEST_DEPTH = 3`) and Inv 14 (attribution scaffolding fields).
- **The "aspirational prose, dead code" failure mode.** Compromise #1 + #5 both had been **declared closed in docs** but the implementation was prose-only. The R7 spec-to-code-compliance audit caught this — and then caught 4 more catalog codes with the same shape (`E_WRITE_CONFLICT`, `E_CID_UNSUPPORTED_CODEC/HASH`, `E_IVM_PATTERN_MISMATCH` defined-but-unfired). The 14-agent R6 had missed all of them because the drift-detector verified enum↔catalog name parity, not runtime reachability. Lesson baked into every later phase's reviewer brief: "verify, don't trust docs."
- **The exit-criterion substitution drift.** Plan §1 named 6 specific gates including `@mermaid-js/parser`, capability-denial-via-`ON_DENIED`, and canonical-fixture-CID assertion. Reality: `@mermaid-js/parser` doesn't ship a flowchart parser; capability-denial requires Phase-2 DSL surface; canonical-CID is asserted at the crate level not the user-visible scaffolder. The scaffolder smoke test substituted regex / unregistered-handler-typed-error / roundtrip-only — defensible substitutions individually, but plan §1 wasn't updated to match. R7 named this F-R7-001 medium.
- **G3 chaos-engineer's 2 system-zone bypass criticals.** Even with N8 system-zone enforcement landed, `RedbBackend::put_node` and `put_edge` had inherent paths bypassing the WriteContext check. R5-G3 pass-2 fixed at all 4 sites (lines 442, 585, plus Transaction::put_node + put_edge). Pattern: critic-driven hardening at the right architectural seam.

**What surprised us:**
- The pre-R1 critic round caught **17 plan-coherence findings** that the planning agent had missed — confirming that critic dispatch is cheap relative to mid-implementation pivots.
- Five R1 critics with non-overlapping lenses returned **61 findings** with **zero disagreements** and zero deferrals — the convergence-without-conflict pattern that became Pattern 6 (reviewer composition follows lens surface) in Phase 2's methodology.
- napi-rs v3's "same codebase compiles to WASM" promise actually held — cargo check --target wasm32-unknown-unknown -p benten-napi (T8) stayed green throughout, even with substantial binding-surface accretion in G8. The dual-target verification at every PR is what kept it that way.
- The R7 spec-to-code-compliance audit caught a class of bug R6's 14-lens council had missed (catalogued-but-unfired codes). The failure mode was that each individual lens reviewer trusted that other lenses checked reachability; nobody actually grepped enum-construction-sites in production code. R7's discipline (every spec claim → at least one non-test construction site) was load-bearing.

**What this phase set up for the next phase:**
- The **Phase-2-stop-gap pattern**: every deferred invariant has a named Phase-1 stop-gap with a documented Phase-2 upgrade path. Inv 4 → SANDBOX executor returns `PrimitiveNotImplemented` (Phase 2 lights it up); Inv 8 → `MAX_ITERATE_NEST_DEPTH = 3` (Phase 2 adds multiplicative budget through CALL); Inv 11 → `E_SYSTEM_ZONE_WRITE` write-path stop-gap (Phase 2 adds full registration-time `E_INV_SYSTEM_ZONE`); Inv 13 → no stop-gap (Phase 2 immutability ratchet); Inv 14 → attribution-fields-on-writes scaffolding (Phase 2 adds per-eval-step structural).
- The **6 named compromises** with regression sub-tests in `compromises_regression.rs` — grep-and-deletable when later phases close the underlying gap. Phases 2a/2b adopted this pattern verbatim.
- The **`ChangeSubscriber` trait separation** — Phase 3 sync extends `ChangeProbe` infrastructure without changing the trait shape.
- The **`benten-errors` crate extraction** — every later crate added in Phase 2/3 (e.g., `benten-sandbox`, `benten-sync`) inherits stable `ErrorCode` discriminants without depending on `benten-core`.
- The **42-item Phase-2-backlog** with concrete remediation paths: Cid::from_str base32 decoder (~30 LOC), IVM `view_stale_count` metric wiring, `get_node_verified` read-path hash check, `benten-eval → benten-graph` dependency break (called arch-1 backlog §1.1), DSL-SPECIFICATION rewrite to revised primitives, ARCHITECTURE.md "six → seven crates" update.

**Phase-defining decisions:**
- **`requires` enforcement = option A (declared + actually-checked)** — Ben's R1-time call. Reshaped capability semantics: `requires` declares the minimum, but each primitive's effective capability is also checked at call time. Closes the AI-agent escalation path that Phase 6 will need closed.
- **System-zone Phase-1 stop-gap (N8) + NoAuthBackend `production()` guard (N5-b) + supply-chain CI (T10) + system-zone-as-Phase-1-stop-gap-via-write-context-flag pattern.** These four together transformed the security posture from "Phase-2 stop-gaps everywhere" to "Phase 1 has cheap-but-real defenses at every named attack surface." The pattern of escalating "this is Phase 2" to "this gets a Phase-1 stop-gap" was the security-auditor's biggest contribution.
- **`ChangeSubscriber` trait in `benten-graph`, tokio-broadcast in `benten-engine`** — the dependency-direction call that preserves the dual-target T8 promise + thin-engine philosophy. Three R1 critics convergent.

---

## § 4. Backlog / compromises / incomplete work

### § 4.1 Carried into this phase from earlier phases

Phase 1 was the first ADDL-pipeline phase, so there's no prior-phase backlog. Inputs came from:
- The 6-crate spike (`SPIKE-phase-1-stack-RESULTS.md`) and its critic-triage P1.*.* tags — all 13 tags integrated into Phase 1 deliverables (P1.core.float → C3, P1.core.version-chain → C6, P1.core.proptest → C5, P1.ci.wasm-runtime → T6, P1.ci.multi-arch → T4, P1.ci.msrv → T5, P1.graph.error-polymorphism → G2-A trait reshape, P1.graph.scan-iterator → G2-A, P1.graph.open-vs-create → G2-B, P1.graph.transaction-primitive → G3-A, P1.graph.node-store-trait → G2-A, P1.graph.doctests → G2 both agents, P1.graph.stress-tests → G2 R3 tests).
- The 12 Validated Design Decisions in CLAUDE.md (12 primitives / IVM Algorithm B / code-as-graph / non-Turing / BLAKE3+CIDv1 / transaction primitive / pluggable cap policy / opt-in version chains / member-mesh / TS DSL crud / three-pillar / committed-scope-Phases-1-8). All 12 verified in code at phase-close.

### § 4.2 Deferred out of this phase

**To Phase 2 (consolidated in `docs/future/phase-2-backlog.md`):**

| Item | Surface | Fix path | LOC est | Target |
|---|---|---|---|---|
| Inv 4 (SANDBOX nesting) enforcement | Registration-time check that no SANDBOX node nests inside another SANDBOX | Add to `invariants.rs`; enforce at registration; new `E_INV_SANDBOX_NESTED` firing | ~50 | Phase 2 |
| Inv 7 (max SANDBOX output) enforcement | Output-size cap on SANDBOX module returns | wasmtime fuel + post-execution byte counter | ~100 | Phase 2 |
| Inv 8 multiplicative-through-CALL budget | Currently scalar runtime budget + registration-time `MAX_ITERATE_NEST_DEPTH = 3`; full multiplicative-through-CALL deferred | Track cumulative iter-count across CALL boundaries | ~80 | Phase 2 |
| Inv 11 full registration-time enforcement | Phase-1 stop-gap is `WriteContext::is_privileged` flag + write-path system-label rejection; full registration-time `E_INV_SYSTEM_ZONE` adds variant + check | Add `InvSystemZone` enum variant; registration-time scan for system-zone targets in user subgraphs | ~40 | Phase 2 |
| Inv 13 (immutability ratchet) | Storage-layer enforcement that registered subgraphs are not mutable | redb-layer immutability after registration; `E_NESTED_TRANSACTION_NOT_SUPPORTED` covers part | ~120 | Phase 2 |
| Inv 14 structural per-step attribution | Phase-1 captures attribution on writes (`PendingOp::PutNode { actor_cid, handler_cid, capability_grant_cid }`); structural per-step is Phase 2 | Extend evaluator to thread attribution through every TraceStep | ~150 | Phase 2 |
| `Cid::from_str` base32 decoder | Phase 1 unconditionally errors with "Phase 2 deliverable"; only napi boundary accepts CID strings | Add ~30-line base32 decoder mirroring existing `Cid::to_base32` encoder | ~30 | Phase 2 (backlog §6.1) |
| `benten.ivm.view_stale_count` metric wiring | Currently hard-coded to `0.0` in `metrics_snapshot()` | Subscriber iterates `View::is_stale()` + tally | ~20 | Phase 2 (backlog §5.3) |
| `get_node_verified` read-path hash check | `RedbBackend::get_node` doesn't re-hash decoded Node against requested CID; subgraph-level via `Subgraph::load_verified` | Add optional `get_node_verified` API | ~50 | Phase 2 (backlog §6.2) |
| `benten-eval → benten-graph` dependency break (arch-1) | Coupling that complicates Phase-2 primitive expansion | Refactor before adding new primitives | ~200 | Phase 2 (backlog §1.1) |
| WASM runtime + network-fetch `KVBackend` | Compile-check only in Phase 1 (T8) | wasmtime host integration + network-fetch backend impl | ~500 | Phase 2 |
| Generalized IVM Algorithm B | 5 hand-written views ship Phase 1; generalization is Phase 2 | Slot Algorithm B as another `View` impl per shared trait | ~400 | Phase 2 |
| SANDBOX wasmtime host | Type defined; executor returns `E_PRIMITIVE_NOT_IMPLEMENTED` | wasmtime instance pool + host-function manifest | ~600 | Phase 2 |
| WAIT primitive executor | Type defined; serializable execution state needed | Phase 2 wires WAIT against `Vec<ExecutionFrame>` serialization | ~200 | Phase 2 |
| STREAM primitive executor | Type defined; back-pressure protocol needed | Phase 2 | ~200 | Phase 2 |
| SUBSCRIBE-as-user-op executor | Engine-internal change plumbing only in Phase 1 | Phase 2 lights up user-facing SUBSCRIBE | ~150 | Phase 2 |
| Capability enforcement across all 12 primitives | Phase 1 enforces on 8 executable primitives; tightening across deferred 4 = Phase 2 | Wire `requires` check into WAIT/STREAM/SUBSCRIBE/SANDBOX executors | ~50 | Phase 2 |
| Paper-prototype re-validation against revised 12-primitive vocabulary | Original validated at 2.5% SANDBOX rate against pre-2026-04-14 set | Re-run paper prototype | n/a | Phase 2 |

**To Phase 3 (UCAN / sync era):**

| Item | Reason |
|---|---|
| `UCANBackend` full impl | Phase 3 `benten-id` deliverable |
| `E_CAP_DENIED_READ` existence-leak Option A → per-grant `existence_visibility: visible|hidden` | Phase 3 sync revisits with sync threat model |
| `E_CAP_REVOKED` (sync-revocation, distinct from `E_CAP_REVOKED_MID_EVAL`) | Phase 3 sync ships revocation propagation |
| Per-capability write rate limits | Phase 1 records `benten.ivm.view_stale_count` metric; Phase 3 enforces per-peer rate |
| Member-mesh networking + iroh + Loro | Phase 3 sync (D9) |

**To Phase 8/9+ (OSS publication era):**

| Item | Reason |
|---|---|
| §3.2 Publish-readiness pass | Deferred to Phase 8/9+ since OSS publication not planned before then (per CLAUDE.md status table note) |
| `[[bin]]` `required-features` gating for `benten-graph` test-fixture binary | Speculative for Phase 1 (no external consumers); Phase 2+ release-hardening |

**Documentation drift (R7 4 mediums to address before Phase-2 kickoff):**

| Item | Spec |
|---|---|
| F-R7-001 plan §1 Gate 3/5/6 wording | Update to match shipped substitutions OR land capability-gated `crud()` DSL + canonical-fixture CID assertion in scaffolder |
| F-R7-002 DSL-SPECIFICATION.md primitive table + CRUD §4.3 examples | Rewrite to revised 12 primitives (drop VALIDATE/GATE; add SUBSCRIBE/STREAM); per banner "Phase 1 deliverable" left open |
| F-R7-003 ARCHITECTURE.md "six crates" → "seven crates" | Add `benten-errors` row + role description |
| F-R7-004 `E_CID_PARSE` catalog fix-hint | Either land base32 decoder OR clarify "Rust API is Phase-2 deliverable; Phase-1 CID input via napi boundary only" |

### § 4.3 Compromises that landed during this phase

8 named compromises documented in `docs/SECURITY-POSTURE.md`:

1. **Compromise #1 — TOCTOU window.** Phase 1 refreshes capability snapshot at three boundaries: commit, CALL entry, every `iterate_batch_boundary` (default 100 iters). Revocations between refresh points not visible to in-flight evaluations. Phase 2 Invariant 13 reduces window to zero. Closure verified at HEAD `f69830b`; tests `compromise_1_toctou_window_bound_at_100_iter_batch` + `capability_revoked_mid_iteration_denies_subsequent_batches`.
2. **Compromise #2 — `E_CAP_DENIED_READ` existence-leak.** Original Option A: leaky-but-honest (denied reads return `E_CAP_DENIED_READ`, leaking existence). Final closure (Option C, post-Step-5b/5c): symmetric `Ok(None)` on policy denial — byte-identical with absence — with `engine_diagnostics::diagnose_read` gated by `debug:read` capability for ops who genuinely need to distinguish denied-vs-not-found. Compromise re-classified from "open" to "closed" at phase-close.
3. **Compromise #3 — ErrorCode crate split.** `benten-errors` crate extracted from `benten-core` to give every Benten crate access to stable `ErrorCode` discriminants without depending on `benten-core`. Closure verified.
4. **Compromise #4 — WASM runtime Phase 2.** T8 is compile-check only via `cargo check --target wasm32-unknown-unknown -p benten-napi`. Browser-context default capability backend (NOT NoAuthBackend; will be `BrowserOriginCapBackend` scoping writes to origin) deferred to Phase 2. Open at phase-1-close per spec; Phase 2 closes.
5. **Compromise #5 — Per-capability write metrics.** Phase 1 emits `benten.writes.committed.<scope>` + `benten.writes.denied.<scope>` keys; Phase 3 enforces per-peer rate limits. Closure verified.
6. **Compromise #6 — BLAKE3 128-bit collision resistance.** Phase 1 usage (dedup + integrity) relies on collision resistance only, not full preimage. Phase 3 UCAN-by-CID paths revisit. Documentation-only; no Phase-1 code change needed. SECURITY-POSTURE.md documents the bound.
7. **Compromise #7 — `[[bin]]` `required-features` gating for test-fixture binary.** Closed: `benten-graph/Cargo.toml` gates `[[bin]]` with `required-features = ["test-fixtures"]`.
8. **Compromise #8 — PrimitiveHost sole dispatch.** Architectural invariant: engine's fast-path CRUD goes through evaluator's host trait, not direct evaluator bypass. Closure verified at HEAD via §5.1 alignment matrix.

Plus two honest-limitation disclosures (not "compromises" but documented intentional gaps):
- **Change-stream subscription bypasses capability read-checks.** `Engine::subscribe_change_events` fans out every committed `ChangeEvent` without per-event `check_read` gate. Honest limitation, fully spec'd in SECURITY-POSTURE.md. Phase 3 sync may add per-event filtering.
- **napi `JSON_MAX_BYTES = 1 MiB`.** Hard limit at the napi boundary; oversized JSON / Bytes / Text rejected with `E_INPUT_LIMIT` pre-allocation. Documented as DoS mitigation.

---

## § 5. Process lessons / Phase-1 patterns

Phase 1 was the project's first ADDL-pipeline phase, so most patterns are first-iteration shapes. Some survived to later phases verbatim; others were retired or reshaped. The named-process catalog (pim-N) wasn't formalized until Phase 2b R6 — Phase-1 patterns are pre-pim-N analogs.

**Patterns that survived to later phases:**

- **The two-tier critic round (pre-R1 + R1).** Pre-R1 catches plan-coherence (zero overlap between critics by lens design); R1 catches architecture quality + security. Phase-1's pre-R1 returned 17 findings dispositioned with zero deferrals; R1 returned 61 with zero disagreements. Phase 2a/2b retained both tiers verbatim.
- **R3 partition by lens, not by file.** 5 R3 writers — unit / edge-cases / security / performance / qa-expert-integration — own disjoint test files but each agent writes against a coherent lens. The disjoint-file contract avoided merge conflicts; the coherent-lens contract gave each agent a focused brief. Phase 2a/2b retained.
- **The two-pass R4 with fix-pass commit between.** Pass 1: critics catch test-quality + coverage + integration issues against TDD red-phase tests. Fix-pass commit. Pass 2: critics verify closure. Phase 1's pass-1→pass-2 was 50→14 findings monotonic-decrease — the convergence pattern that Phase 2b R6 formalized as 6-round monotonic convergence (R1=64 → R6=0).
- **Pattern 6 (reviewer composition follows lens surface).** Phase 1 R6 had 12 lenses; R6b added `operation-primitive-linter` because the redux against fix-pass code surfaced primitive-vocabulary checks that the original 12 didn't cover. Phases 2a/2b/3 formalized this as Pattern 6.
- **Verify-don't-trust-docs contract.** R6b applied it explicitly — every claim walked to code path or its absence. Caught the "aspirational prose, dead code" pattern at 6 sites that R6 had missed. Became a baked-in agent-brief contract for every later phase.
- **Named compromises with grep-able regression sub-tests.** `compromises_regression.rs` had one `#[test]` per compromise, each with a `// Phase X compromise; remove when Phase Y implements Z` comment. Grep-and-deletable when underlying gap closes. Phases 2a/2b adopted verbatim.
- **`docs/future/phase-N-backlog.md` for explicit deferrals.** Phase-1 created `docs/future/phase-2-backlog.md` at phase-1-close to capture all Phase-2-target items with concrete remediation paths. The pattern (named destination + concrete fix + LOC estimate + target phase) became the foundation of HARD-RULE clause-(b) BELONGS-NAMED-NOW that Ben formalized in Phase 2b.
- **Phase-1 stop-gap with documented Phase-N upgrade path.** When a deferred capability has a real attack surface, Phase 1 ships a cheap stop-gap (e.g., `WriteContext::is_privileged` for Inv 11, `MAX_ITERATE_NEST_DEPTH = 3` for Inv 8, attribution-fields for Inv 14). The stop-gap becomes the upgrade trampoline. Pattern named in Phase 2 retrospective.

**Patterns that were retired or reshaped after Phase 1:**

- **R7 spec-to-code-compliance audit as a separate stage.** Phase-1-only invention. R7 walked every spec doc and verified each spec claim had a code construction site. Caught the "catalogued-but-unfired error codes" class that R6's 14-lens council missed. Phases 2a/2b/3 did NOT have R7 stages — R7's discipline merged into the R6 quality council via the verify-don't-trust-docs contract that R6b pioneered. R7's "spec-IR → code-IR → alignment-IR" 7-phase methodology survives in the `spec-to-code-compliance` skill, available on demand.
- **Pre-implementation-plan addendum-after-the-plan pattern.** Phase 1's `00-implementation-plan.md` had the R1 Triage Addendum appended at the end — a 130-line section that listed all 17 R1 fix-now dispositions with section-by-section update guidance. Phase 2a/2b/3 abandoned this in favor of editing the plan in-place + a separate `r1-triage.md` doc. Phase 1's appended-addendum approach worked but generated mental load ("which version of §2.5 E5 is correct — the original or the addendum's update?").
- **Single-doc R3 coverage stub vs per-agent JSONs.** Phase 1 had `r3-coverage-stub.json` as the cumulative coverage map updated by all 5 R3 agents (with `r3-coverage-qa-expert.json` as a partial slice). Phase 2a/2b/3 formalized into per-agent `r3-coverage-<agent>.json` JSONs.
- **The "44 agents at .claude/agents/" project-local registry.** Phase 1 used a per-project `.claude/agents/` directory with 44 agent definitions. The handoff doc explicitly references `rust-implementation-developer` agent at `.claude/agents/rust-implementation-developer.md`. Phase 2a/2b/3 dropped this in favor of platform-managed `subagent_type` defaults — `general-purpose` carries the lens definition in the brief. Reason: project-local agent JSONs accumulated drift faster than the orchestrator could maintain them.
- **R4 pass-1 + fix-pass-commit + pass-2.** Phase 2a/2b R6 generalized to multi-round monotonic convergence (R6 R1 → R2 → R3 → ... until 0 new findings). The two-pass shape was a stepping stone.

**Failure modes named during Phase 1 (input to later phases' discipline):**

- **"Aspirational prose, dead code"** — compromises declared closed in docs but implementation prose-only. Phase-1 R7 named at 6 sites. Became the most-cited failure mode in later phases' reviewer briefs.
- **"Tests pass vacuously"** — TDD red-phase tests written against `todo!()` stubs can pass vacuously if R5 implementer reshapes APIs. Phase 1 R4b is the explicit guard. Survived as ADDL pattern.
- **"Catalogued-but-unfired error codes"** — drift-detector verifies enum↔catalog name parity but not runtime reachability. Phase-1 R7 caught 4 instances. Drift-detector hardened in Phase 2a to add reachability check.
- **"Documentation falls behind code"** — 4 R7 mediums + ARCHITECTURE.md "six crates" + DSL-SPECIFICATION VALIDATE/GATE imports + scaffolder Gate 3/5/6 substitutions. Phase 2a/2b's `feedback_post_fix_doc_coupling_preflight` (later codified as pim-1) addresses this directly.

**Cross-reference into memory dir (which Phase-1 lessons became `feedback_*.md`):**

- "Aspirational prose, dead code" → `feedback_post_fix_doc_coupling_preflight.md` (pim-1) and the verify-don't-trust-docs contract baked into reviewer briefs
- R6b's verify-walk-to-code → `feedback_reviewer_pre_flight_tree_state.md` (reviewer briefs MUST mandate `git fetch origin && git rev-parse HEAD`)
- "Tests pass vacuously" + R4b → `feedback_end_to_end_test_pin_for_closed_claims.md` (pim-2) — production runtime arm + observable consequence + would-FAIL-if-no-op'd
- Pattern 6 reviewer composition → `feedback_reviewer_composition.md`
- The 12-primitive-vocabulary-finalized + ENGINE-SPEC-as-source-of-truth → `feedback_engine_primitives_vs_application_layer.md` (push application-layer composition before engine extension)

---

## § 6. Decisions baked in / architectural commitments

The Phase-1 commitments that became permanent in the engine. CLAUDE.md captures these as the "Architectural Decisions Baked In (Do Not Re-Debate)" list. Each item below cross-references the source landing point.

1. **12 operation primitives — final canonical set.**
   - **Phase / wave:** Phase 1 pre-R1 (2026-04-14 reconciliation).
   - **Source:** Plan §2.5 E1; ENGINE-SPEC §3 finalized; CLAUDE.md decision #1.
   - **Lives:** `benten-eval/src/lib.rs` `PrimitiveKind` enum; `benten-eval/src/primitives/mod.rs` (executor dispatch); `docs/ENGINE-SPEC.md` §3.
   - **Why it matters:** Adding/removing a primitive cascades through invariant validation, evaluator dispatch, error catalog, DSL surface, IVM views, capability enforcement matrix, and every stored subgraph. Phase 2b commitment #16 (SANDBOX is for compute that doesn't fit other 11 primitives) reaffirmed irreducibility.

2. **IVM Algorithm B (dependency-tracked incremental) with per-view strategy selection.**
   - **Phase / wave:** Phase 1 plan + R5-G5 hand-written 5 views; Algorithm B itself is Phase 2.
   - **Source:** Plan §2.3 I9 + Rank 10; ENGINE-SPEC §8; CLAUDE.md decision #2.
   - **Lives:** `benten-ivm/src/view.rs` (shared `View` trait); 5 hand-written impls in `benten-ivm/src/views/`; Phase 2 generalizes by adding Algorithm B as another impl of the trait.
   - **Why it matters:** The shared `View` trait (added per R1 architect major) means Phase 2's generalization slots in as another `View` impl, not a rewrite. R1 architect-reviewer's pushback (5 hand-written without shared trait would have made Phase 2 retrofit aspirational) was the load-bearing intervention.

3. **Code-as-graph: handlers ARE subgraphs of operation Nodes, not source code strings.**
   - **Phase / wave:** Phase 1 G6 + G7.
   - **Source:** ENGINE-SPEC §3; CLAUDE.md decision #3.
   - **Lives:** `benten-eval/src/lib.rs` (`Subgraph`, `SubgraphBuilder`); `benten-engine/src/engine.rs::register_subgraph`.
   - **Why it matters:** Deepest differentiator from PostgreSQL+AGE. Every subsequent phase (Phase 2 SANDBOX, Phase 3 sync, Phase 6 AI agents) builds on this — handlers as content-addressed graph data, not source strings.

4. **Not Turing complete: DAGs only. Bounded iteration. SANDBOX is the Phase 2 escape hatch.**
   - **Phase / wave:** Phase 1 G6 (DAG via Inv 1; bounded via Inv 8 stop-gap `MAX_ITERATE_NEST_DEPTH = 3` + runtime budget).
   - **Source:** ENGINE-SPEC §4 Inv 1 + Inv 8; CLAUDE.md decision #4.
   - **Lives:** `benten-eval/src/invariants.rs` (cycle detection + iterate-nest-depth + iterate-budget); SANDBOX returns `E_PRIMITIVE_NOT_IMPLEMENTED` in Phase 1.
   - **Why it matters:** Halting-guarantee on every handler. Phase 6 AI-agent attack surface depends on this — agent-authored handlers can't infinite-loop the engine.

5. **Content-addressed hashing: BLAKE3 + DAG-CBOR + CIDv1. Multicodec `0x71` (dag-cbor); multihash `0x1e` (BLAKE3). Hashes labels + properties; NOT anchor_id, NOT timestamps, NOT edges (excluded per ENGINE-SPEC §7).**
   - **Phase / wave:** Phase 1 G1.
   - **Source:** SPIKE validation + plan §2.1 C1-C4; ENGINE-SPEC §7; CLAUDE.md decision #5.
   - **Lives:** `benten-core/src/lib.rs` BLAKE3 + CIDv1 constants; `Node::cid()` excludes `anchor_id` via `#[serde(skip)]`; `Edge::cid()` separate content-addressing (4 fields).
   - **Why it matters:** Canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` stable across all platforms, MSRVs, and wasm32-wasip1 runtime — verified by T9 cross-leg determinism gate. Phase 3 sync depends on byte-identical hashing across peer machines.

6. **Transaction primitive (begin/commit/rollback) as first-class API, not `transactional: true` property.**
   - **Phase / wave:** Phase 1 G3 + G7.
   - **Source:** Plan §2.5 E6; ENGINE-SPEC §5/§9; CLAUDE.md decision #6.
   - **Lives:** `benten-engine/src/engine.rs::transaction(|tx| ...)` closure-based API; `benten-graph/src/transaction.rs` for the underlying primitive; `E_NESTED_TRANSACTION_NOT_SUPPORTED` for nested-tx rejection.
   - **Why it matters:** Closure-based avoids the WAIT-Phase-2 retrofit cost (`transactional: true` would have required handler-Node migration). Closure-panic → `catch_unwind` → rolled-back redb tx → re-raised; Inv 13 immutability adds atop without API change.

7. **Capability system as pluggable policy. `CapabilityPolicy` pre-write hook trait with `NoAuthBackend` default + `Engine::builder().production()` guard.**
   - **Phase / wave:** Phase 1 G4 + R1 SC2 disposition (added N5-b production() guard).
   - **Source:** Plan §2.4 P1-P3 + R1 SC2; ENGINE-SPEC §9; CLAUDE.md decision #7.
   - **Lives:** `benten-caps/src/policy.rs` trait; `benten-caps/src/noauth.rs` + `grant_backed.rs` + `ucan_stub.rs`; `benten-engine/src/builder.rs::production()`.
   - **Why it matters:** UCAN is one backend (Phase 3), not the only one. Embedded / single-user / browser-origin / custom — pluggable policy doesn't lock in a sync model. NoAuth `production()` guard prevents accidental production deployments.

8. **Version chains as opt-in pattern.** Anchor Node + Version Nodes + CURRENT pointer in `benten-core`. Ephemeral data (presence, cache) does NOT pay versioning cost.
   - **Phase / wave:** Phase 1 G1-B + R1 philosophy major (`without_versioning` builder added).
   - **Source:** Plan §2.1 C6; ENGINE-SPEC §6; CLAUDE.md decision #8.
   - **Lives:** `benten-core/src/version.rs` (`Anchor`, `walk_versions`, `current_version`, `append_version`); `EngineBuilder::without_versioning()`; tests `version_chain_linking_does_not_change_version_node_cids` confirms versioning doesn't contaminate the Node hash.

9. **Member-mesh networking** (Phase 3 — D9 not verified at Phase-1 gate, expected).
   - Roadmap-level commitment; no Phase-1 code surface.

10. **TypeScript DSL with `crud('post')` zero-config shorthand.**
    - **Phase / wave:** Phase 1 G8 + R1 dx disposition (createdAt + updatedAt + id + zero-config completeness).
    - **Source:** Plan §2.7 B6; CLAUDE.md decision #10.
    - **Lives:** `packages/engine/src/index.ts`; `packages/engine/src/crud.ts` (zero-config + audit-field injection).

11. **Three-pillar positioning** (vision-level).
12. **Committed scope = Phases 1-8** (roadmap-level).

**Phase-1 R1-time additions to the baked-in list (not original to plan):**

13. **`ExecutionState` on-disk format = DAG-CBOR + CIDv1 envelope** (decision came in late Phase 1 / pre-Phase-2a planning, ratifying Phase 1's choice).
14. **SANDBOX host-function manifest = capability-derived** (decision deferred to Phase 2a planning; Phase 1 only ships SANDBOX type-defined-but-not-executed; the manifest commitment tracks back to Phase 1's spec discipline).

**Phase-1 R1-time security disposition decisions (Ben-ratifications):**

- **`requires` enforcement model = Option A** (declared + actually-checked per-primitive at call time). Source: R1 SC4 Ben-escalation. Lives at `benten-eval/src/primitives/{call,write}.rs` cap-entry + commit-time checks. Closes the AI-agent escalation path that Phase 6 will need closed.
- **Evaluator frame model = `Vec<ExecutionFrame>` + `frame_index: usize` indices, not arena+IDs.** Source: R1 architect M2 + Ben choice. Lives at `benten-eval/src/evaluator.rs`. Phase 2 WAIT-serialization shape: serialize the frame vec + the stack of `(FrameId, ContinuationPoint)`.
- **Change-stream placement = `ChangeSubscriber` trait in `benten-graph` (runtime-agnostic) + tokio-broadcast impl in `benten-engine` + sync-callback wrapper for WASM.** Source: R1 architect M1 + philosophy + security convergent + Ben choice. Lives at `benten-graph/src/subscribe.rs` (trait); `benten-engine/src/change_probe.rs` (impl). Preserves dual-target T8 promise + thin-engine philosophy.
- **`requires` is the minimum declared; per-primitive checks at call-time enforce actual capability.** Source: R1 SC4 Option A. Closes the silent-escalation attack.
- **`benten-errors` crate split (Compromise #3 closure).** Source: R1 architect major + Ben acceptance during Step-5c. ErrorCode discriminants in zero-Benten-dep crate; every other crate depends on it.

These Phase-1 commitments are non-renegotiable in later phases (CLAUDE.md "Architectural Decisions Baked In (Do Not Re-Debate)"). Phase-2 commitments (SANDBOX manifest = capability-derived, ExecutionState DAG-CBOR envelope) trace back to discipline established in Phase 1's R1 + R6 + R7 stages.
