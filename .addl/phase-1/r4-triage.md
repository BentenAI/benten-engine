# R4 Triage — Phase 1 Test Review (Pre-Implementation)

**Stage:** R4 (test review before R5 implementation). Tests compile against `todo!()` stubs.
**Critics:** rust-test-reviewer (quality), rust-test-coverage (completeness), qa-expert (integration). 3 independent lenses, dispatched in parallel. Retry run after an Anthropic API 500 outage wiped the first attempt.
**Raw findings:** `.addl/phase-1/r4-{rust-test-reviewer,rust-test-coverage,qa-expert}.json`.

## Headline

| Critic | Verdict | Critical | Major | Minor | Total |
|---|---|---|---|---|---|
| `rust-test-reviewer` | revise | 3 | 11 | 10 | 24 |
| `rust-test-coverage` | revise | 0 | 8 | 6 | 14 |
| `qa-expert` | revise | 0 | 5 | 7 | 12 |
| **Net** | **revise** | **3** | **24** | **23** | **50** |

All 3 converge on revise. Zero block. Zero architectural disagreements. Recurring themes across critics (vacuous-pass class, TS infra missing, proptest shortfall, CRUD gaps, API drift the consolidator missed) → confirmed-valid.

## Triage disposition table

### Critical findings (3 — all rust-test-reviewer, all fix-now)

| # | Area | Disposition | Fix |
|---|---|---|---|
| C1 | Scaffolder meta-test regex broken (`/\\bit\\s*\\(/g` → 0 matches) | **Fix-now.** | Change `/\\bit\\s*\\(/g` to `/\bit\s*\(/g` in `tools/create-benten-app/test/scaffolder.test.ts:50`. Verify the assertion `toBe(6)` now counts 6 `it(` occurrences in the generated `my-app/test/smoke.test.ts`. |
| C2 | TOCTOU compromise test assertions conflict (`>=149` vs `>=200`) | **Fix-now (reconcile scope).** | R1 triage says: cap refresh at commit / CALL entry / ITERATE batch boundaries, default batch=100. So the compromise's scope is: iterations 1-100 succeed (cap snapshot held); iteration 101 fails (new batch re-reads revoked cap). Rewrite both tests: `tests/integration/compromises_regression.rs` + `crates/benten-caps/tests/toctou_iteration.rs` both assert exactly `successful_write_count == 100` and `write_101_returns_E_CAP_REVOKED_MID_EVAL`. |
| C3 | Vacuous pass in 3 IVM view test suites (only `matches!(ViewResult::Cids(_))`) | **Fix-now.** | Rewrite `view1_capability_grants.rs`, `view2_event_dispatch.rs`, `view5_version_current.rs` to assert specific CIDs / counts. Each test now: (a) populates known input, (b) runs update, (c) reads view, (d) asserts specific CID set + length, not just variant discriminant. |

### Major findings (24) — all fix-now except 2 deferrals

| # | Source | Area | Disposition | Fix |
|---|---|---|---|---|
| M1 | qa | TS infra missing (no package.json) | **Fix-now.** | Create `bindings/napi/package.json`, `packages/engine/package.json`, `tools/create-benten-app/package.json`. Each declares `@benten/engine` workspace-link, vitest dev-dep, and (for scaffolder) `@mermaid-js/parser` dev-dep. Root `package.json` + workspaces declaration if absent. |
| M2 | qa | Exit-criterion #2 never asserts deterministic `createdAt` | **Fix-now.** | `tests/integration/exit_criteria_all_six.rs::exit_2` + `packages/engine/src/crud.test.ts` both gain explicit assertion: assert the three posts' `createdAt` values form a strictly-increasing sequence AND re-reading the same post returns the same `createdAt` (stamped-once property). |
| M3 | qa | `d2_cross_process_graph.rs` doesn't span processes | **Fix-now.** | Rewrite to spawn a child via `std::process::Command` calling `cargo run --example write_and_exit` (producing a fixture), then re-read in parent. If `cargo run` is too heavy, use a compiled binary produced by `build.rs` + `CARGO_BIN_EXE_` env var. Either way: actual PID separation. |
| M4 | qa | Scaffolder npm-test path has no workspace-link | **Fix-now (with M1).** | Root `package.json` declares a workspaces array including `bindings/napi`, `packages/engine`, `tools/create-benten-app`. Scaffolder template uses `file:../../packages/engine` until `@benten/engine` publishes to npm. CI uses `npm install --workspaces` from root. |
| M5 | qa + reviewer | `incremental_view_lag_never_exceeds_one_write` tautologically true | **Fix-now.** | Replace `observed + 1 >= expected` with strict `expected - observed <= 1` + `observed > 0` (ensures IVM actually ran). Naming it so doesn't change the math — the fix is the stricter assertion plus a liveness check. |
| M6 | reviewer | MVCC snapshot test name/body mismatch | **Fix-now.** | `mvcc_snapshot.rs::snapshot_reader_sees_point_in_time` must: (a) open a `snapshot()`, (b) write to backend concurrently, (c) assert reader still sees pre-write state, (d) drop snapshot, (e) assert reader now sees post-write state. |
| M7 | reviewer | IVM view3_content_listing proptest only 5 cases + discriminant-only assertion | **Fix-now.** | Widen case range to `0usize..256`, change assertion to compare actual `Vec<Cid>` payloads between incremental and from-scratch rebuild. |
| M8 | reviewer | `iterate_max_and_nest_depth`'s `_typecheck_iterate_max_is_u64` fn never called | **Fix-now.** | Delete the no-op fn. The type is already checked by the compiler when other tests construct `SubgraphBuilder::iterate(max)`; a no-run test adds zero value. |
| M9 | reviewer | `option_a_existence_leak_is_documented_compromise` tautological | **Fix-now.** | Replace the self-comparison with: (a) assert `ErrorCode::CapDeniedRead != ErrorCode::NotFound` (distinct codes), (b) assert `CapDeniedRead.as_str()` is the exact catalog-documented string `"E_CAP_DENIED_READ"`, (c) assert `SECURITY-POSTURE.md` documents this as Option A. |
| M10 | reviewer | `value_float.rs` vs `float_nan_inf.rs` duplicate coverage with divergent contracts | **Fix-now.** | Merge: `value_float.rs` stays as the canonical location; `float_nan_inf.rs` becomes a one-line `//! Moved to value_float.rs; see there.` doc-only file OR delete it if no test name is uniquely covered there. Reconcile divergent contracts in favor of the R1-agreed NaN + ±Inf rejection. |
| M11 | reviewer | index.test.ts 32MB allocation + sleep-based pattern | **Fix-now.** | Reduce to 1.5× the `B8 bytes_len` limit (~1.5MB if limit is 1MB). Replace sleeps with explicit event primitives (or deterministic polling with small timeout). |
| M12 | reviewer | view1/view2 `update_with_event_ok` / `rebuild_from_scratch_ok` only assert `.unwrap()` succeeds | **Fix-now (folds into C3).** | Covered by the C3 IVM-view-vacuous-pass fix; those tests now assert state post-operation. |
| M13 | reviewer | `durability_modes.rs` bench falls through to default `RedbBackend::open` | **Fix-now.** | Add a compile-time sentinel: each bench body contains `todo!("durability_modes::<MODE>: R5 must wire DurabilityMode::{Group,Async}")` — the fallback-to-default behavior is explicitly a failure mode, not a silent path. Bench thus panics at R5 time until wired. |
| M14 | reviewer | NoAuthBackend / WriteContext API drift across 3 files | **Fix-now.** | Canonical forms (per consolidator): `NoAuthBackend::new()` constructor, `WriteContext { is_privileged: bool, principal: Option<EntityId> }`. Rewrite all 3 files (`noauth.rs`, `noauth_proptest.rs`, `production_refuses_noauth.rs`) to use the canonical forms. |
| M15 | reviewer | Compromise #4 (WASM Phase 2) scan uses string-grep on filenames | **Fix-now.** | Rewrite `compromise_4_wasm_runtime_is_phase_2` to: (a) assert `benten-napi`'s Cargo.toml does NOT declare `wasmtime` as a runtime dep, (b) assert `.github/workflows/wasm-checks.yml` exists and contains `cargo check --target wasm32-unknown-unknown`, (c) assert no workflow invokes `cargo test --target wasm32-wasip1` (runtime testing is Phase 2). Semantic, not literal-string. |
| M16 | coverage | 8 missing proptests (50% of landscape projection) | **Fix-now 5 / defer 3.** | Fix-now: `prop_subgraph_cid_order_independent` (R1 philosophy), `prop_edge_roundtrip_cid_stable`, `prop_kvbackend_put_get_delete`, `prop_value_json_cbor_conversion`, `prop_transform_expression_deterministic`. Defer to Phase 1 R5: `prop_capability_check_deterministic`, `prop_hlc_monotonic`, `prop_transform_grammar_fuzz_accepted_deterministic` (these can land during implementation, not pre-implementation — they each require substantial impl-side harness). |
| M17 | coverage | CRUD surface untested (Engine::update_node, delete_node, edge methods, B3 napi layer) | **Fix-now.** | Add test stubs for each method. Tests are TDD red until R5 G7 + G8 land. Files: `crates/benten-engine/tests/engine_crud.rs` (Rust), `bindings/napi/index.test.ts` (TS tier). |
| M18 | coverage | Engine methods added during integration scaffolding have no direct unit tests | **Fix-now.** | Add direct tests for `Engine::register_subgraph`, `call`, `trace`, `transaction`, `snapshot`, `grant_capability`, `create_view`, `revoke_capability` in `crates/benten-engine/tests/engine_api_surface.rs`. |
| M19 | coverage | RedbBackend::get_by_label / get_by_property untested | **Fix-now.** | Add `crates/benten-graph/tests/indexes.rs` with rejection + positive + boundary tests for label and property indexes. |
| M20 | coverage | IVM views 1/2/5 missing per-view 3-category coverage | **Fix-now (folds into C3).** | C3 rewrites these 3 files; each now has (a) build-from-scratch-matches-incremental, (b) stale-on-budget, (c) write-read latency bound. |
| M21 | coverage | Version-chain free-function vs method API drift | **Fix-now.** | C6 reserves `walk_versions`, `current_version`, `append_version`, `Anchor`. Make the `version` submodule the canonical export; top-level `Anchor` becomes an alias `pub use version::Anchor;`. Tests use the submodule. |
| M22 | coverage | Test-type category under-projection vs R2 landscape | **Defer (accept).** | R2 projected 201 unit / 14 proptest / 18 integration / 14 criterion / 11 Vitest / 22 security / 9 CI. R3 delivered higher unit count (agents expanded to triplets). Proptest shortfall is already M16. Other categories are within 20% band. Accept and document. |
| M23 | reviewer | `concurrent_reader_writer_soak` 2s wall-clock flaky on slow CI | **Fix-now.** | Reduce to `SOAK_DURATION: Duration::from_millis(500)` + increase thread count from 4 to 8 so per-second assertions are stable. Add `#[ignore]` tag gated by `CI_FULL_SOAK=1` env var for longer-form runs. |
| M24 | reviewer | `transform_grammar_rejections.rs` hardcoded byte offsets | **Fix-now.** | Helper computes expected offset from the input string via `.find("<token>")` rather than hardcoded integer, so parser error-message-format refactors don't break every rejection test. |

### Minor findings (23) — fix-now cheap ones, defer style-only

| # | Source | Area | Disposition | Fix |
|---|---|---|---|---|
| m1-m5 | reviewer | Various clarity / weak-bound minor issues | **Fix-now bundle.** | Quick tightenings: `evaluator_pops_on_respond` stricter stack-delta assertion; `phase_1_executable_subset_is_eight_primitives` use set equality not count+contains; `write_cas_wrong_version_conflict` pick one of (`Ok(ON_CONFLICT)` OR `Err(WriteConflict)`) per plan §2.5 E3 — not both; `stale_view::write_commits_even_if_ivm_is_stale` inner loop → use `retain.then_some`; `view4_governance_inheritance` cycle case now uses a distinct flag from depth-exceeded. |
| m6 | reviewer | `NoAuthBackend` construction idiom inconsistency (sub-issue of M14) | **Fix-now (folds into M14).** | — |
| m7 | reviewer | `engine_builder_thinness::lint_budget` implies enforcement but only counts subscribers | **Fix-now.** | Rename to `thin_engine_has_no_ivm_subscribers_when_disabled`. The lint-budget aspect is deferred to a future phase CI check; document in test body comment. |
| m8 | reviewer | `system_zone_integration` uses test-only subscribe helper | **Fix-now.** | Use the real `Engine::subscribe_change_events` public API rather than the `test_subscribe_change_events_matching_label` helper. |
| m9 | coverage | Invariant 5/6 missing `rejects_one_over` diagnostic-accessor tests | **Fix-now.** | Add `invariant_5_nodes_rejects_one_over_with_actual_field` and same for 6 — assert `error.nodes_actual()` and `error.nodes_max()` return the right values. |
| m10 | coverage | Invariant 9/10/12 missing positive-at-limit diagnostic tests | **Fix-now.** | Add assertions that `error.determinism_class()` / `error.cid_expected()` / `error.cid_actual()` / `error.violated_invariants()` accessors return correct values on the rejection cases. |
| m11 | coverage | `LABEL_CURRENT` + `LABEL_NEXT_VERSION` constants not asserted | **Fix-now.** | Add `crates/benten-core/tests/version_chain_label_constants.rs` asserting exact string values (`"current"`, `"next_version"` or whatever the plan commits to). |
| m12 | coverage | napi Rust-side `input_validation.rs` doesn't gate on napi-export feature | **Fix-now.** | Add `#[cfg(feature = "napi-export")]` gating + a doc comment explaining the test is linkable only with `cargo test --features in-process-test --no-default-features`. |
| m13 | coverage | E_CAP_ATTENUATION only fired in `error_code_mapping.rs` | **Fix-now.** | Add `crates/benten-caps/tests/call_attenuation.rs` firing `E_CAP_ATTENUATION` via the declared-vs-actual check on chained CALL. |
| m14 | coverage | Missing standalone MSRV workflow | **Fix-now.** | Add `.github/workflows/msrv.yml` running `cargo +1.85 build --workspace --locked` + fixture-match check. |
| m15 | coverage | `ErrorCode::from_str` catch-all `Unknown(String)` untested | **Fix-now.** | Add `error_codes.rs::unknown_error_code_preserves_string_not_panic` per R1 drift-detector finding. |
| m16 | qa | Fixture-CID canary missing from compromise_4 test | **Fix-now.** | compromise_4 regression additionally asserts the canonical CID fixture hasn't changed (`assert_eq!(canonical_test_node().cid().to_string(), FIXTURE_CID)`). |
| m17 | qa | Mermaid parse uses `??` fallback masking wrong-API failures | **Fix-now.** | Remove the `??` — if parser throws, let the exception propagate and fail the test. |
| m18 | qa | `exit_3` cap-denial test omits `.capability_policy_grant_backed()` | **Fix-now.** | Add `.capability_policy_grant_backed()` to the builder; test was silently using NoAuthBackend. |
| m19 | qa | napi test uses CommonJS `require()` under Vitest ESM | **Fix-now (with M1 package.json).** | Rewrite `index.test.ts` to use ESM `import`. |
| m20 | qa | `Engine::open` vs `Engine::builder().build()` drift | **Fix-now.** | Canonicalize: `Engine::builder().open(path)` or `Engine::open(path)` — pick one, rewrite both call sites. Recommend `Engine::builder().open(path)` for consistency with `builder().production()` pattern. |
| m21 | qa | `integration.rs` aggregator comment has causality backwards | **Fix-now (trivial).** | Fix the comment. |
| m22 | qa | `E_TX_ABORTED` pinned to `ON_ERROR` without spec confirmation | **Fix-now (spec edit).** | Add explicit line to `ENGINE-SPEC.md` §5 confirming tx aborts route via `ON_ERROR` (not a separate `ON_TX_ABORT` edge). |
| m23 | qa | `exit_3` mermaid fixture-CID canary (minor) | **Folds into m16.** | — |

## Deferrals (2 total)

- **M22 test-type projection gap** — accept as within-range; R3 delivered higher unit counts by design, proptest gap tracked separately as M16.
- **M16 (partial) — 3 of 8 proptests** (cap-check determinism, HLC monotonic, TRANSFORM fuzz accepted-determinism) — require substantial impl-side harness; land during R5, not pre-implementation.

## Disagreements

None.

## Named compromise updates (post-triage)

Compromise #1 (TOCTOU) scope clarified: cap snapshot refreshed at commit / CALL entry / ITERATE batch boundary where default batch size = 100 iterations. Writes 1-100 succeed under granted cap; write 101 tests the re-read at batch 2 boundary against the revoked cap and returns `E_CAP_REVOKED_MID_EVAL`. C2 fix propagates this to both test files.

## Ready for R5 after

Execute all fix-now dispositions, verify `cargo check --workspace --all-targets` stays clean, `cargo test --workspace --no-run` stays clean, commit as a single R4-triage slice. Then:

1. Create R5 implementation-group dispatch briefs (per plan §3 G1–G8)
2. Dispatch G1 implementers in parallel
3. After G1 commits: mini-review + move to G2

## Stats after triage

- Critical findings: 3, all fix-now
- Major findings: 24, 22 fix-now + 2 partial defer
- Minor findings: 23, all fix-now
- Total fix-now items: ~49
- Total deferrals: 2 (documented scope; not re-litigated)
- Disagreements: 0
