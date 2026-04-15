# R4 Pass-2 Triage — Phase 1

**Stage:** R4 second-pass review, after fix-pass commit `5c60c31` landed 49 of 50 first-pass findings.
**Critics:** rust-test-reviewer + rust-test-coverage + qa-expert (same 3 as first pass; fresh dispatch).
**Raw findings:** `.addl/phase-1/r4-pass2-{rust-test-reviewer,rust-test-coverage,qa-expert}.json`.

## Headline

| Critic | Pass-1 verdict | Pass-2 verdict | Critical | Major | Minor |
|---|---|---|---|---|---|
| `rust-test-reviewer` | revise (3c/11m/10m) | **pass** | 0 | 0 | 5 |
| `rust-test-coverage` | revise (0/8m/6m) | **pass** | 0 | 0 | 0 (4 documented deferrals) |
| `qa-expert` | revise (0/5m/7m) | **accept-with-minors** | 0 | 0 | 5 |
| **Net pass-2** | revise | **PASS** | **0** | **0** | **~10 live + 4 triaged-deferrals** |

Fix-pass worked. Zero critical, zero major residuals. All 3 pass-1 criticals verified fixed:
- C1 scaffolder regex → fixed (`/\bit\s*\(/g`)
- C2 TOCTOU reconciliation → fixed (both tests now assert `successful == 100`, `DEFAULT_BATCH_BOUNDARY = 100` pinned)
- C3 IVM views vacuous-pass → fixed (views 1/2/5 all have populated-specific-CID assertions)

Coverage stats improved materially:
- Public APIs: ~92/113 → **108/113 (96%)**
- Proptests landed: **7/14 → 14/14**
- Error codes fired: 21/21 (stable; E_CAP_ATTENUATION now direct, not roundtrip-only)
- Invariants with full quartet: 8/8 (diagnostic-accessor gaps closed)
- Compromises regressed: 6/6 (TOCTOU scope corrected)
- Attack vectors tested: 11/11

## Residual minor findings disposition

### Infra fix-now (pre-R5) — 1 bundled commit

**qa-p2-1 — `workspace:*` npm-compat.** Used in 2 child package.json files; npm <11 doesn't resolve it natively. **Fix-now:** replaced with `file:../../<path>` explicit relative paths. Works with npm back to 2.x, no Corepack requirement. Applied to:
- `packages/engine/package.json`: `"@benten/engine-native": "workspace:*"` → `"file:../../bindings/napi"`
- `tools/create-benten-app/package.json`: `"@benten/engine": "workspace:*"` → `"file:../../packages/engine"`

### Roll into R5 (10 items; R5 group-level polish)

- **p2-review-1** IVM views 2/5 rebuild-matches-incremental tautology → R5 G5 (views implementation time, when source-of-truth exists)
- **p2-review-2** Compromise #6 BLAKE3-128 weak grep inconsistent with #4's semantic rewrite → R5 G1 (when SECURITY-POSTURE.md gets its real content)
- **p2-review-3** `evaluator_stack.rs` CAS-conflict shape accepts both `Ok(ON_CONFLICT)` + `Err(WriteConflict)` → R5 G6 (implementer picks the shape; test tightens to match)
- **p2-qa-2** `capability_policy_grant_backed()` + `register_crud_with_grants()` composition semantics → R5 G4+G7 (wiring decision at integration time)
- **p2-qa-3** `[[bin]]` `required-features` gating speculative for Phase 1 (no external consumers); defer to when `benten-graph` publishes → Phase 2 release hardening
- **p2-qa-4** IVM lag `assert_eq!` tightening if barrier-sync is Phase 1 contract → R5 G5 (barrier semantics landed in G5)
- **p2-qa-5** Scaffolder meta-test regex false-matches `it(` in comments → R5 G8 (scaffolder refinement time; cheap fix to exclude `//.*it(` patterns)
- **cov-f1** 3 triage-deferred proptests (cap-check determinism, HLC monotonic, TRANSFORM grammar accepted-determinism) → R5 per R4 first-pass triage M16
- **cov-f2** `Engine::call_async` R1-DX-#5 barrier-race regression → R5 G7 (when call_async implementation lands)
- **cov-f3** Version-chain free-function vs module API canonicalization → R5 G1 (unification at implementation time; R4 fix narrowed per m20)

### Documented deferrals (already in r4-triage.md)

- 3 of 8 proptests deferred per M16 partial-defer (cap-check, HLC, fuzz-accepted)
- `Engine::open` vs `Engine::builder().build()` drift — m20 narrowed; R5 wires both
- Test-type projection gap within-range per M22 accept

### Known-new findings from fix-pass (feeds into R5 dispatch briefs)

Not re-raised by pass-2 critics (correctly acknowledged as R5 scope):

1. `Outcome::as_node()` missing — R5 G7 adds
2. `Anchor` top-level (`id: u64`) vs `version::Anchor` (`head: Cid`) API divergence — R5 G1 unifies
3. `RegistrationError::determinism_class()` accessor — R5 G6 adds alongside Invariant 9 enforcement
4. Exit-criterion #3 `.capability_policy_grant_backed()` + `register_crud_with_grants` overlap — R5 G4+G7 verify convergence

## Disagreements

None.

## Verdict + next action

**Pass-2 verdict: PASS.** All criticals and majors resolved; residual minors are R5-polish scope or already-triaged deferrals. Pipeline remains green.

With the `workspace:*` npm-compat fix landing in this commit, the R4 gate is closed and R5 can dispatch.

## Stats: pipeline at pass-2 close

- `cargo check --workspace --all-targets` — PASS
- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `cargo test --workspace --no-run` — PASS (392+ tests compile; runtime is TDD red)
- Canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` stable
