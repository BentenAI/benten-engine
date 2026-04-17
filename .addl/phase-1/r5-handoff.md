# Handoff: Benten Engine Phase 1 — R5 Implementation Dispatch

## Who You Are

You are the orchestrator for the Benten Engine — a Rust-native graph execution engine where data and code are unified as Nodes and Edges. Code IS graph. The engine evaluates itself. This is the foundation of a decentralized platform anchored by three pillars: the engine, personal AI assistants as adoption driver, and treasury-backed credits as primary revenue.

You are NOT primarily an implementer — you coordinate specialized agents who do implementation, triage their findings, and surface decisions to the user.

## Repository

`/Users/benwork/Documents/benten-engine`

## Where We Are

Phase 1 ADDL pipeline through **R4 is complete**. You are dispatching **R5 (implementation groups G1-G8)** — the stage where real Rust code gets written to make the TDD test suite pass.

### What's done (DO NOT re-run)

| Stage | Status | Key artifacts |
|-------|--------|---------------|
| Pre-work (spike + 3 critics) | ✅ | `SPIKE-phase-1-stack-RESULTS.md`, `.addl/spike/*.json` |
| R1 spec review (5 agents, 61 findings) | ✅ | `.addl/phase-1/r1-*.json`, `r1-triage.md` |
| R2 test landscape (289 artifacts planned) | ✅ | `.addl/phase-1/r2-test-landscape.md`, `r3-coverage-stub.json` |
| R3 test writing (5 parallel agents) | ✅ | 392 tests across 95+ files + 10 benches + 6 TS Vitest + 8 CI workflows |
| R3 consolidation (API-name drift reconcile) | ✅ | Commit `89db8ad` |
| R4 test review (3 agents, 2 passes) | ✅ | `.addl/phase-1/r4-*.json`, `r4-triage.md`, `r4-pass2-triage.md` |

### Pipeline state

- `cargo check --workspace --all-targets` — GREEN
- `cargo fmt --all -- --check` — GREEN
- `cargo clippy --workspace --all-targets -- -D warnings` — GREEN
- `cargo test --workspace --no-run` — GREEN (392+ tests compile; they're TDD red — R5 makes them green)
- Canonical fixture CID: `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` — STABLE

### Coverage at R4-gate-close

| Metric | Value |
|--------|-------|
| Public APIs covered | 108 / 113 (96%) |
| Error codes fired | 21 / 21 (100%) |
| Invariants with full test quartet | 8 / 8 |
| Named compromises with regression tests | 6 / 6 |
| R1 attack vectors with adversarial tests | 11 / 11 |
| Proptests landed | 14 / 14 |
| Exit-criterion assertions (Rust + TS) | 6 / 6 |
| CI workflows | 9 / 9 |

## Required Reading (in order)

1. **`CLAUDE.md`** — orchestrator role, current status at R5, validated decisions, Phase 1 scope, user preferences
2. **`.addl/phase-1/00-implementation-plan.md`** — especially §3 (8 implementation groups with file-ownership maps and must-pass test lists) and the **R1 Triage Addendum** (scope-shape changes, named compromises, attribution fields, security stopgaps, DX hardening, test-landscape additions)
3. **`.addl/phase-1/r4-pass2-triage.md`** — the final state of test-quality gates; 10 minor residuals mapped to their R5 groups; 4 known-new findings from the fix-pass
4. **`docs/ENGINE-SPEC.md`** §3-§11, §14 — the technical spec the implementation must match
5. **`docs/TRANSFORM-GRAMMAR.md`** — the TRANSFORM expression BNF (G6-B's parser implements this)
6. **`docs/ERROR-CATALOG.md`** — stable error codes; every Phase-1 code has a test that fires it
7. **`.addl/phase-1/r2-test-landscape.md`** — what tests exist and which R5 group makes them green

## Your Immediate Task: Dispatch R5 Group G1

R5 runs 8 groups in dependency order per the plan's §3. Each group dispatches 2-3 parallel `rust-implementation-developer` agents (agent definition at `.claude/agents/rust-implementation-developer.md`) owning disjoint file sets.

### Group dependency ordering

```
G1 (core types + version chains) → (G2, G4 in parallel) → G3 (transaction + change stream) → G5 (IVM + 5 views) → G6 (evaluator + 8 primitives + invariants + TRANSFORM) → G7 (engine orchestrator) → G8 (napi + TS wrapper + scaffolder + CI)
```

### Per-group protocol (from `docs/DEVELOPMENT-METHODOLOGY.md`)

For each group:
1. Brief N parallel `rust-implementation-developer` agents — each gets files it owns + tests it must make pass
2. After they return: run `cargo-runner` (pipeline validation)
3. Mini-review: dispatch `code-reviewer` + the crate's guardian against files changed
4. Triage every mini-review finding — fix before advancing to next group
5. Commit
6. Ben gates advancement to next group

### G1 specifics

**Scope (plan §3 G1):** benten-core hardening — Edge type, Float(f64) with NaN rejection, version-chain primitives (Anchor, CURRENT/NEXT_VERSION, walk_versions, append_version), error-code ErrorCode enum mapping. **2 parallel agents:**

- **G1-A:** `crates/benten-core/src/value.rs` (or update lib.rs's Value section), `crates/benten-core/src/error_code.rs`. Must-pass: `tests/value_float*`, `tests/error_code*`, `tests/proptests.rs::prop_node_roundtrip_cid_stable`
- **G1-B:** `crates/benten-core/src/edge.rs`, `crates/benten-core/src/version.rs`. Must-pass: `tests/edge_*`, `tests/version_*`, `tests/anchor_*`, `tests/proptests_edge_roundtrip.rs`, `tests/proptests_subgraph_order.rs`

### 10 minor residuals from R4 pass-2 (each mapped to a group)

- G1: Compromise #6 BLAKE3-128 regression strengthen; Anchor API convergence (top-level vs version::Anchor)
- G4+G7: `capability_policy_grant_backed()` + `register_crud_with_grants()` composition
- G5: IVM views 2/5 rebuild-matches-incremental; IVM lag `assert_eq!` tightening if barrier-sync
- G6: `evaluator_stack.rs` CAS-conflict shape; `RegistrationError::determinism_class()` accessor
- G7: `Outcome::as_node()` add; `Engine::call_async` barrier-race regression test
- G8: Scaffolder meta-test regex `it(` in comments; Vitest B3 surface spot-check

### 4 known-new findings from R4 fix-pass (R5 scope)

1. `Outcome::as_node()` missing — add at G7
2. Top-level `Anchor(id: u64)` vs `version::Anchor(head: Cid)` — converge at G1
3. `RegistrationError::determinism_class()` accessor — add at G6
4. Exit-criterion #3 `.capability_policy_grant_backed()` vs `register_crud_with_grants` — verify path convergence at G4+G7

### 7 named compromises (preserve, don't re-litigate)

1. Invariant 13 TOCTOU window (cap refresh at commit/CALL/batch boundaries, default batch=100)
2. E_CAP_DENIED_READ leaks existence (Option A; Phase 3 revisit)
3. ErrorCode enum stays in benten-core (Phase 2 may extract)
4. WASM runtime Phase 2 (T8 is compile-check only)
5. Per-capability write rate limits (Phase 1 records metrics; Phase 3 enforces)
6. BLAKE3 128-bit collision resistance (documented in SECURITY-POSTURE.md)
7. `[[bin]]` required-features gating deferred to Phase 2 crates.io publish

### 3 deferred proptests (R5 add-ons, not pre-R5 blockers)

- `prop_capability_check_deterministic` — G4
- `prop_hlc_monotonic` — G5 or G7 (wherever HLC integration lands)
- `prop_transform_grammar_fuzz_accepted_deterministic` — G6

## Agent Ecosystem You Have

- **44 agents** at `.claude/agents/` — all exist and are usable. Key for R5:
  - `rust-implementation-developer` — the per-group implementer
  - `cargo-runner` — pipeline validation
  - `code-reviewer` — mini-review quality lens
  - Crate-specific guardians: `benten-core-guardian`, `ivm-algorithm-b-reviewer`, `ucan-capability-auditor`, `operation-primitive-linter`, `code-as-graph-reviewer`, `determinism-verifier`, `benten-engine-philosophy`, `napi-bindings-reviewer`
- **Subagent permissions:** `.claude/settings.json` has broad non-destructive allow-list (cargo, git, gh, filesystem) + explicit deny-list (destructive git, package publish, repo-level ops). Subagents can commit their own slices.
- **Skills:** `/spike`, `/commit-rust` (no Co-Authored-By per policy), `/review-crate`, `/bench`, `/clippy-strict`, `/invariant-check`, `/addl-phase`, `/critic`

## The User (Ben)

- CEO and co-architect of BentenAI. Philosophy: "do it right, not fast."
- **No AI attribution on ANY commit** — not public, not internal. The `/commit-rust` skill enforces this.
- Questions reshape architecture — pause and think rather than answering reflexively.
- Present options with tradeoffs, not conclusions.
- GitHub: personal `Benten-Ben`, org `BentenAI`. Fork `BentenAI/rust-cid` with upstream PR #185 (approved, awaiting maintainer merge).

## Upstream PR Watch (passive)

- `multiformats/rust-cid#185` — APPROVED by athola, awaiting maintainer merge
- `multiformats/rust-multihash#407` — cwlittle's PR, changes-requested nit

When both merge + release, remove `[patch.crates-io]` entries from `Cargo.toml`.

## If You Have Questions

Ask Ben to relay a message to the previous orchestrator (me). I have the full context of every architectural decision, every critic finding, every triage disposition, and every named compromise. I can clarify anything about:

- Why specific R1 findings were dispositioned the way they were
- The 7 named compromises and their scope
- The R4 fix-pass decisions (what was fixed vs narrowed vs deferred)
- The evaluator architecture decisions (frame indices, change-stream trait placement, `requires` enforcement model)
- The dependency situation (core2 yank, BentenAI/rust-cid fork, no_std_io2 migration)
- The Phase 1 "tight middle" scope reconciliation (8 primitives, 8 invariants, 5 IVM views, transaction primitive)

## Start

Read `CLAUDE.md` and `.addl/phase-1/00-implementation-plan.md` §3. Confirm with Ben that R5 G1 dispatch is the next action. Then brief and dispatch 2 `rust-implementation-developer` agents for G1 per the group partition above.
