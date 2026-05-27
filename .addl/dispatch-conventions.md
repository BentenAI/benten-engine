# Phase-2b Dispatch Conventions — branch-per-agent CI-as-verifier

<!-- cite-drift-exempt-file: historical-narrative

This document carries phase-bound ratification records (added 2026-04-29, added
2026-05-03 R6-R5, added 2026-05-09 R6 R6-final, etc.) whose phase-N artifact
cites (`.addl/phase-N/<file>.json`) are intentional historical pointers to
frozen forensic records. Phase-1/2a/2b artifact directories may have been
archived to `.addl/_archive/<phase>/` post-tag, making the cite targets
"missing at HEAD" — but the cite intent is "the ratification origin lived
THERE at the time of ratification," not "navigate here in the current tree."

Living rule text in this file uses path/symbol form for current-tree cites
(per §3.5b HARDENED point 3); historical ratification-origin cites use this
file-level exemption per R6-R2-FP-OD 2026-05-25 elegance-pass ratification +
`feedback_extra_reflection_pass_for_elegant_permanent_shape`.
-->

Standing rules for every Phase-2b agent brief. Briefs reference this doc with one line: "Apply [dispatch-conventions](./dispatch-conventions.md)." Specific items (owned files, must-pass tests, deviations from these standing rules) go in the scoped brief body.

**Replaces** Phase-2a `dispatch-conventions.md` Option-C hybrid (agent ran local workspace cargo + orchestrator pushed to main). Phase-2b agents push their own branches; CI runs per-branch via PR triggers; orchestrator gates merge on CI-green.

**Predecessor reading carried forward:** `.addl/phase-2a/dispatch-conventions.md` (many sections survive unchanged); `.addl/phase-2a/r5-decisions-log.md` D13 + D13.4 (Option-C hybrid history; this doc supersedes).

**Pre-G12-A blocker checklist** (METH-3): this doc must exist; the Bucket-3 workflow YAML expansion (METH-1) wiring `pull_request: branches: [main]` triggers across the required-check fleet must be merged; `gh auth status` must succeed from agent worktrees (METH-11).

---

## 1. Discipline (non-negotiable — carried from Phase-2a §1)

- **No `git stash` / `git stash pop` ever.** Phase-2a saw three self-disclosed stash violations across G3-B-cont / G5-B-i / G5-B-i-fix-pass despite explicit bans. The rule stands. If you hit a build error outside your scope, READ the file, report the diagnosis in your report — do NOT relocate tree state.
- **No force-push / no amend** on shared branches. Create NEW commits on top of HEAD on your feature branch. **Exception:** orchestrator MAY `git push --force-with-lease` to a feature branch when rebasing onto main pre-merge (METH-7); agents MUST NOT. Note: `gh pr merge --rebase` IS the merge strategy (§7) — that is rebase-by-GitHub at merge time, not the forbidden local-`git rebase` of shared branches.
- **No AI attribution.** Author `Benten-Ben <ben@benten.ai>`. No `Co-Authored-By: Claude`, no "Generated with Claude Code", no Anthropic / Claude mentions in commit bodies, code comments, PR titles, or PR bodies.
- **Don't bluff in deviations reports.** The orchestrator verifies `git show --stat` against your claimed file list.
- **Trust your own writes.** "File modified by user or linter" system-reminders are informational, not reversions. `git diff` confirms if you need to check.

---

## 2. Branch-per-agent procedure (REPLACES Phase-2a §2)

Operationalizes plan §6.1 (and addresses METH-1, METH-2, METH-5, METH-9, METH-10, METH-12).

### 2.1 Worktree + branch creation

Each agent gets a fresh worktree on a fresh branch. Branch name is **orchestrator-assigned in the dispatch brief** — agents do not invent names.

```
git worktree add -b phase-2b/<group>/<agent-slug> ../benten-wt-<group>-<agent-slug> main
cd ../benten-wt-<group>-<agent-slug>
gh auth status        # smoke-check at onboarding (METH-11)
```

If `gh auth status` reports unauthenticated, halt + report in deviations; do NOT attempt other auth paths (§6 + METH-11).

**Branch-namespace conventions** (METH-15):
- `phase-2b/<group>/<agent-slug>` — dispatched-agent work (default).
- `phase-2b/orchestrator/<topic>` — orchestrator-initiated cross-cutting work (e.g. workflow trigger expansion).
- `phase-2b/<group>/<agent-slug>/fix-pass-N` — OPTIONAL if a fix-pass scope is large enough to warrant its own branch; default is to land fix-passes on the same branch.

### 2.2 Commit on the agent branch ONLY

Per-commit conventions: Conventional Commits + ADDL provenance + no AI attribution (see §10). Commits land on `phase-2b/<group>/<agent-slug>` ONLY — never `git push origin main`, never directly attempt `git checkout main`. Branch protection enforces this regardless (`enforce_admins: true` in `.github/branch-protection.yml`).

### 2.3 Push + draft PR open (one paired action)

```
git push -u origin phase-2b/<group>/<agent-slug>
gh pr create --draft --base main \
  --head phase-2b/<group>/<agent-slug> \
  --title "<group>: <one-line summary>" \
  --body "$(cat <<'EOF'
Dispatch brief: <link to brief doc or inline summary>
Scope: <files-owned summary>
Must-pass tests: <list from brief>
EOF
)"
```

Draft prevents accidental merge. PR open is what triggers required-check workflows (METH-1: existing fleet uses `pull_request: branches: [main]` triggers; agent push without PR triggers nothing).

### 2.4 Return structured-JSON result

Agent reports in its structured-JSON output:
- `branch_name` — the full ref name.
- `pr_number` — from `gh pr create` output.
- `commit_shas` — list of SHAs landed on the branch.
- Per the §11 report template.

### 2.5 CI validation (orchestrator-side)

CI fires on PR open + every push to the PR's head branch via the existing `pull_request: branches: [main]` triggers (assumes Bucket-3 workflow YAML expansion is merged). Orchestrator monitors:

```
gh pr checks <pr-number>           # snapshot of all check states
gh run watch <run-id>              # for live monitoring
gh api repos/BentenAI/benten-engine/actions/jobs/<job-id>/logs 2>&1 | tail -100
```

**Required-check set** (must all be `success` to consider the PR validated): `ci.yml`, `determinism.yml`, `phase-2a-exit-criteria.yml` (rolls forward as `phase-2b-exit-criteria.yml` later in 2b), `drift-detect.yml`, `supply-chain.yml`, plus Phase-2b additions per `.addl/phase-2b/00-implementation-plan.md` §3.1 as they land (`wasm-conformance.yml`, `cargo-public-api`, `HOST-FUNCTIONS.md` drift detector).

### 2.6 Pre-merge gates

Orchestrator merges only when ALL hold:
1. CI green on the PR's current head SHA.
2. Sync mini-review complete with merge-as-is OR fix-pass landed (§4).
3. If main has advanced since the PR's head was pushed, branch is rebased on main + re-CI'd green (METH-7).

Then:
```
gh pr ready <pr-number>                     # flip from draft to ready
gh pr merge <pr-number> --rebase --delete-branch
```

`--rebase` preserves per-agent commit narrative (METH-9) and satisfies branch-protection `required_linear_history: true`. `--squash` is reserved for branches with genuine scaffold/wip noise — exceptional, not default.

### 2.7 Merge order across siblings (METH-10)

Green PRs merge in **declared dispatch order** (the order the brief specified). Preserves semantic ordering even if CI returns out-of-order (G12-C lands before G12-D even if G12-D's CI returns first).

**Predecessor-stall escalation:** if a predecessor PR's CI exceeds 2× the calibrated CI timing (current Phase-2a baseline ~7 min → 14-min stall threshold) while siblings are green, orchestrator inspects the predecessor:
- `in_progress` due to slow runner → wait one more cycle.
- `failed` → dispatch fix-pass to predecessor + merge siblings out of declared order IF semantic ordering allows (logging the deviation in `r5-decisions-log.md`).
- Semantic ordering forbids out-of-order (e.g. G12-C must precede G12-D) → orchestrator blocks siblings + dispatches fix-pass.

### 2.8 Red-branch handling (METH-12)

Orchestrator picks ONE path per red-branch event — never both concurrently:

- **Inline fix (<50 LOC, orchestrator-owned).** Orchestrator checks out the feature branch in its own main worktree (`git fetch && git checkout phase-2b/<group>/<agent-slug>`), edits, commits, pushes, returns to main. Default to this path; keeps the agent worktree purely agent-owned.
- **Targeted fix-pass dispatch (>50 LOC or scope-bounded correctness).** Single agent dispatched against the same branch (re-enters the persisted worktree if still present, OR a fresh worktree on the same branch).

### 2.9 Rebase-on-main + re-CI before merge (METH-7)

If main advances between agent push and merge, the orchestrator rebases the feature branch on main + force-pushes-with-lease + waits for re-CI:

```
git checkout phase-2b/<group>/<agent-slug>
git pull --rebase origin main
git push --force-with-lease origin phase-2b/<group>/<agent-slug>
# CI re-runs automatically on the push; wait per §2.5
```

This catches workspace-wide regressions that a per-branch CI run against an older main would miss (the multi-branch CI gap METH-7 names).

#### 2.9.1 Force-with-lease safety pre-steps (added 2026-04-29 wave-8)

User-level `--force*` is denied except when the orchestrator runs through the **5-step safety check below** AND uses `--force-with-lease` (NEVER `--force` alone). Background: force-push is destructive in 3 scenarios — pushing to a shared/protected branch, racing an unfetched remote update, or losing local uncommitted work. The lease check guards against scenario 2 ONLY if the lease is fresh; the other two need explicit pre-steps.

**MUST run all 5 before any `git push --force-with-lease`:**

```bash
# 1. Branch identity gate — never force-push main/master
BRANCH=$(git rev-parse --abbrev-ref HEAD)
case "$BRANCH" in
  main|master|trunk) echo "REFUSED: $BRANCH is protected; never force-push" >&2; exit 1 ;;
  phase-2b/*|phase-*) echo "OK: feature branch $BRANCH" ;;
  *) echo "Unfamiliar namespace $BRANCH — abort or surface to Ben" >&2; exit 1 ;;
esac

# 2. Fresh fetch — the lease is only as good as the freshness of what
#    we last fetched. Always fetch immediately before force-push.
git fetch origin

# 3. Working tree clean — force-pushing with uncommitted changes risks
#    orphaning work that was about-to-be-committed.
test -z "$(git status --porcelain)" || { echo "REFUSED: working tree has uncommitted changes" >&2; exit 1; }

# 4. Diff inspection — confirm what's about to push is what I expect.
#    Visual sanity check; not auto-asserted because rebases legitimately
#    rewrite history.
git log @{u}..HEAD --oneline
echo "^ commits about to be force-pushed; confirm intentional"

# 5. Force-with-lease (NEVER plain --force). The lease-check refuses
#    if origin's tip has moved beyond what step 2 fetched.
git push --force-with-lease origin "$BRANCH"
```

**Fallback:** if the 5-step check fails OR the divergence is too large to reason about safely (e.g. local has 10 commits the remote doesn't, the remote has 3 commits we don't, a rebase-cycle attempt would be error-prone), use the **fresh-worktree workaround** instead:

```bash
git worktree remove ../benten-wt-<slug> --force         # local content recoverable from remote
git worktree add ../benten-wt-<slug> origin/<branch> --track -b <branch>-tracking
# apply intended changes on top of the fresh tracking branch
git push origin HEAD:<branch>                            # plain push, no force needed
```

This is non-destructive (no `--force` involved) and was used successfully in wave-8e (`b2df2b6..0210988` msrv.yml fix) when the local rebased copies diverged from origin and force-push was disallowed. Slower but bulletproof.

**Never bypass:** if the 5-step check feels like overkill for a "quick" push, that's a signal to USE the fresh-worktree workaround instead. The check exists because force-push lost work in past projects; the cost of running it is ~10 seconds.

### 2.10 Cleanup

After successful merge, orchestrator removes the agent worktree:

```
git worktree remove ../benten-wt-<group>-<agent-slug> --force
```

Or via the `ExitWorktree` tool. Branch already deleted by `gh pr merge --delete-branch`.

---

## 3. No local cargo for agents

Same pattern as Phase-2a Option-C hybrid (workspace cargo skipped; agent does scoped pre-flight only; CI is authoritative). Phase-2b refines the pre-flight to OPTIONAL pre-push smoke (the dispatch brief does not require it).

**Pre-flight smoke (OPTIONAL, agent's discretion):**
```
cargo check -p <your-primary-crate> --all-targets --features benten-eval/testing
cargo clippy -p <your-primary-crate> --all-targets --features benten-eval/testing -- -D warnings
cargo fmt --all --check
```

If you run pre-flight, paste exit codes + last 5 lines of each in your report.

**Exception — targeted unit-test smoke for a specific correctness concern:** `cargo test --lib -p <your-crate> --features benten-eval/testing` is fine when verifying a specific fix before commit. Use sparingly.

**FORBIDDEN (workspace cargo / integration tests hang the sandbox):**
- `cargo check --workspace` / `cargo clippy --workspace`
- `cargo test --workspace` / `cargo nextest run --workspace`
- Any `cargo test` of integration binaries (Phase-2a sandbox-hang precedent).

**Set `timeout: 600000` (10 min) on any cargo command via Bash.** Do NOT use `run_in_background: true` + polling loops — wasteful and unnecessary. If a cargo command doesn't finish in 10 min, halt and report.

CI workflows (per §2.5) are the authoritative verification — they catch every compile / clippy / fmt / test issue a workspace local check would catch, faster (~7 min full CI) and on cleaner runners.

**Agent's standing instruction (added 2026-04-27 after wave-4 OOM):** if a dispatch brief asks for workspace cargo (`cargo check --workspace`, `cargo clippy --workspace`, `cargo nextest run --workspace`, `cargo doc --workspace`) — DECLINE and run scoped pre-flight only per the OPTIONAL list above, then surface to the orchestrator that the brief over-prescribed. Wave-4 OOM crashes were partly caused by 7 agents each running workspace clippy + nextest in parallel; the policy in §3 is load-bearing for laptop resource budget. If the orchestrator insists workspace cargo IS needed for a specific verification reason, document the reason in the brief; otherwise scoped is the default.

**Phase-3 RED-PHASE compile-state expectation (added 2026-05-05 R4-close):** main `e548b58` (post all R4-FP + R2-FP merges) contains ~365 RED-PHASE test pins from R3 corpus + R4 R2-FP additions that compile-FAIL by design — they reference symbols (`benten_engine::testing::*`, `engine.registerUserView`, `Engine::consume_sync_replica_*`, etc.) that R5 implementers will introduce wave-by-wave. Confirmed pre-existing compile failures spot-checked across R2-FP cycle: `call_stream_as_partial_revoke_cancels_stream` / `wait_production_runtime_routing` / `wallclock_refresh_uses_monotonic_only` / `register_subgraph_replace` / `system_zone_api_exclusivity` / `immutability_rejects_reput` / `capability_grant_writes_immediate` / `inv_11_transform_constructed_cid_adversarial` / `ast_cache_invalidation` / `integration::stream_napi`.

**Why this matters for R5 implementer briefs:** the per-package scoped pre-flight discipline (§3) means each implementer's `cargo check -p <their-crate>` SUCCEEDS (their crate doesn't reference yet-unwired symbols); the workspace `cargo check --workspace --all-targets` FAILS until the matching R5 wave un-ignores + wires the production arm. CI's `coverage.yml` flake (`cargo-llvm-cov`) is informational + has been accepted as such per CLAUDE.md status table for many rounds; do NOT mistake it (or the broader workspace compile failures) for regressions. Each R5 wave's mini-review verifies that the wave's PRs un-ignore the corresponding RED-PHASE pins + the post-merge state has fewer compile-failure pins, monotonically.

**The R5 dispatch standing rule:** every R5 implementer brief MUST include this paragraph: "Pre-existing RED-PHASE compile failures on main are expected per R3 corpus + R4 R2-FP carries (CLAUDE.md §13 + dispatch-conventions §3 RED-PHASE expectation). Run scoped per-package check + clippy + fmt; CI workspace failures pre-existing are NOT regressions; surface only NEW failures introduced by your diff."

### 3.1 Shared compile cache via sccache (added 2026-04-27)

`~/.cargo/config.toml` configures `rustc-wrapper = "/opt/homebrew/bin/sccache"` workspace-wide. Every `cargo` invocation across all worktrees pulls from + populates a shared `~/Library/Caches/Mozilla.sccache` cache (20 GB limit). wasmtime + napi-rs + criterion + heavy transitive deps compile ONCE and are reused across `wt-g6-a` / `wt-g7-a` / `wt-g7-b` / etc. Agents do not need to configure anything — sccache is transparent.

Operational notes:
- `sccache --show-stats` shows hit/miss rate (good first hit rate per session is ~30-50% as cache warms; subsequent ~80%+).
- `sccache --zero-stats` resets counters between waves if you want a clean per-wave measurement.
- `sccache --start-server` is automatic on first invocation; if cache appears stuck, restart with `sccache --stop-server` then any cargo command.
- Cache lives in `~/Library/Caches/Mozilla.sccache` and counts toward macOS volume disk usage.

### 3.2 Cargo-clean-idle discipline (added 2026-04-27)

When a worktree is between fix-pass cycles (e.g. mini-review in flight, expected idle for >30 min), the orchestrator may run `cargo clean` in that worktree to free `target/` disk. Next pre-flight rebuilds (~2-3 min cold via sccache; subsequent worktree builds reuse compiled artifacts via sccache so the rebuild is a recompile-only step, not a from-scratch). Combined with sccache, this keeps idle worktrees at ~50 MB rather than 1-4 GB.

When NOT to cargo-clean:
- Worktree has uncommitted in-progress work (the agent will need to re-run pre-flight from clean state, slower).
- Active mini-review-fix-pass cycle ongoing (agent is rebuilding incrementally; cargo-clean would lose the warm cache).
- Worktree is the orchestrator's main worktree at `/Users/benwork/Documents/benten-engine` (always-warm; takes minutes to rebuild from scratch).

**Audit trigger:** at every wave boundary, run `du -sh ../benten-wt-* ~/Library/Caches/Mozilla.sccache` + `df -h /Users` to surface disk pressure. If `df` shows >85% used, cargo-clean idle worktrees pre-emptively.

### 3.3 Reviewer pre-flight tree-state assertion (added 2026-04-29 night-shift R4b)

**MANDATORY for every reviewer / auditor agent brief.** The night-shift R4b first-batch surfaced 3 of 8 reviewers running against a stale orchestrator working tree (HEAD `78ff8d3`, pre-wave-4) while main was at `8169807` (wave-7 close), producing high-confidence false negatives ("doc doesn't exist", "wasmtime not in workspace", "ModuleManifest production type doesn't exist"). Agents using `git show origin/main:...` would have caught the drift; agents reading from the orchestrator working tree did not.

Every reviewer brief MUST include this pre-flight verbatim near the top:

```bash
cd /Users/benwork/Documents/benten-engine
git fetch origin main
git rev-parse HEAD
# MUST output: <expected SHA>
git status --short
# MUST be empty (no uncommitted changes); if not empty, STOP and report
```

The brief MUST include the expected SHA. If `git rev-parse HEAD` does not match OR `git status --short` is non-empty, the reviewer MUST abort the audit and report — not proceed. The reviewer should also state the audited SHA at the top of their summary so the orchestrator can confirm cross-reviewer consistency.

This is paired with a discipline on the orchestrator side: the orchestrator MUST sync its own working tree to `origin/main` after every PR merge (not just the agent worktrees). The orchestrator tree at `/Users/benwork/Documents/benten-engine` is itself a worktree + accretes scaffolding across waves; if it isn't reset, agents reading from it see a hybrid of pre-wave + wave-N + leftover-debris state.

**Cross-reference:** memory `feedback_reviewer_pre_flight_tree_state` carries the post-mortem narrative + the 3-of-8 false-negative incident.

### 3.5 Cross-target / cross-toolchain pre-flight (added 2026-04-29 wave-8 close)

**MANDATORY for every implementer brief.** Wave-8b accumulated 5 orchestrator-direct CI fix-passes (rustfmt edition, 1.95 clippy `manual_checked_ops`, wasm32 cfg-gate E0433, runner disk pressure, drift detector construction site) because the brief didn't explicitly enumerate these checks. Wave-8c/d-types/f delivered cleanly FIRST TRY after the briefs added explicit callouts.

The 4 dimensions an agent's local pre-flight CANNOT catch by default:

```bash
# 1. Stable rustfmt — CI's rustfmt may collapse multi-line sigs the agent's
#    local rustfmt accepts. Pin "+stable" explicitly; do NOT rely on default.
cargo +stable fmt --all --check

# 2. Rust 1.95 clippy — new lints (manual_checked_ops, duration_suboptimal_units)
#    stabilized in 1.95; agents on 1.94.x defaults won't see them.
cargo +1.95 clippy -p <each-touched-package> --all-targets --features <flags> -- -D warnings

# 3. wasm32 target — engine surfaces referencing wasmtime/native deps need
#    cfg(not(target_arch = "wasm32")) gates + wasm32 stubs returning typed
#    errors. Local agents often don't have the target installed.
rustup target add wasm32-unknown-unknown
cargo +1.95 check --target wasm32-unknown-unknown -p <each-touched-package>

# 4. Drift detector — new ErrorCode variants MUST have a non-test construction
#    site in crates/*/src/. Adding the variant + ERROR-CATALOG.md row alone
#    fails the drift detector unless reachable from production code.
node --import tsx scripts/codegen-errors.ts  # regen TS bindings
# Verify the new variant is constructed somewhere in crates/*/src/ (not just tests).

# 5. Stable rustdoc strict-lint (added 2026-05-03 R6-R5 ratification of pim-7).
#    `cargo +1.95 doc` may pass while `cargo +stable doc` fails because some
#    `-D warnings` rustdoc lints only fire on the latest toolchain. The
#    cargo-doc CI cell uses `+stable`, but per-package pre-flight at #2 above
#    uses 1.95 (the MSRV cell). Add the stable-doc leg explicitly.
cargo +stable doc -p <each-touched-package> --no-deps
```

**Brief template:** state explicitly which of the 5 dimensions apply (e.g. "wasm32 build N/A — touches only napi cdylib which is wasm32-cut" is acceptable; silence is not).

#### 3.5h Orchestrator-direct workspace pre-push verification (added 2026-05-08 G21-T2 close, pim-N candidate)

The dimensions above are MANDATORY for AGENT pre-flight. They're scoped per-package because parallel agents OOM the laptop on workspace-wide cargo (cap=7 with sccache + cargo-clean-idle holds the line on RAM only at scoped pre-flight; workspace cargo from N agents thrashes).

**The orchestrator does NOT have this constraint** — only one orchestrator process runs at a time. So before pushing ANY fix-pass commit (orchestrator-direct OR verification of an agent-landed branch), the orchestrator runs:

```bash
cargo clippy --workspace --all-targets -- -D warnings   # workspace clippy
cargo doc --workspace --no-deps                          # rustdoc broken-link
cargo deny check                                         # supply-chain
cargo +stable fmt --all -- --check                       # format (pin +stable to match dimension #1)
cargo run -p cite-drift-detector --quiet -- . --all     # cite drift
node --import tsx scripts/codegen-errors.ts             # regen TS bindings; verify clean diff
```

~60-90s on warm sccache; ~3-4min cold.

**Why now:** G21-T2 wave (2026-05-08) burned **5 CI cycles × 15min = 75 minutes** on a single PR — clippy `.err().expect()` + 2 path-style rustdoc cites in different files. ALL would have been caught in ONE local 60s pre-push run.

**MANDATORY-PRE-MERGE-AFTER-MINI-REVIEW-APPROVE clause (added 2026-05-09 R6 R6-final ratification of pim-cite-drift-fp1-recurrence after 4-instance recurrence at HEAD):** the §3.5h workspace pre-push gate is also MANDATORY before merging ANY fp1-PASS wave that has APPROVED mini-review. The mini-review verdict APPROVE / Merge-as-is checks SUBSTANCE-of-the-fix (per-finding closure verification + would-FAIL-if-no-op'd 4-axis check + SHAPE-not-SUBSTANCE assertions per §3.6f) but does NOT exercise §3.5h workspace clippy + cargo doc + cargo deny + `cargo +stable fmt --all -- --check` + cite-drift-detector + codegen-regen. The 4 confirmed recurrences of "mini-reviewer APPROVE → orchestrator skips §3.5h → CI surfaces N follow-up batches": (a) Wave-D fp2 `99a330d` cite-drift §7.20; (b) Wave-C2 fp `273475a` 6 cite-drift findings; (c) Wave-C1 fp2 `a2e6da6` 3 cite-drift findings + ATTACK-SURFACE-MATRIX symbol typo; (d) PR #171 fp-4/5/6 cluster (codegen drift + COMPOUND_STEM_EXPANSIONS Rust-mirror gap + stable rustfmt). All would have been caught by §3.5h locally in 60-90s warm. Codification: §3.5h becomes MANDATORY-BEFORE-MERGE on every fp1-PASS wave with APPROVED mini-review, NOT just orchestrator-direct fix-pass commits. Composes naturally with §3.5b HARDENED (post-fix doc-coupling) + §3.6b sub-rule 4 (per-finding granularity) + §3.6f (SHAPE-not-SUBSTANCE pre-flight).

**EXCEPTION clause for per-AGENT pre-push promotion (added 2026-05-09 R6 R6-final ratification of 3plus-r6-final-2):** gate-steps 4 (`cargo +stable fmt --all -- --check`), 5 (`cargo run -p cite-drift-detector --quiet -- . --all`), AND the codegen-regen step (`node --import tsx scripts/codegen-errors.ts`; verify clean diff after) MUST also be run by IMPLEMENTER agents at PR pre-push. They are fast (<10s combined), do not run workspace cargo, do not OOM under parallel agents. The other gates (workspace clippy, cargo doc --workspace, cargo deny) remain orchestrator-only because of the parallelism-cap-on-workspace-cargo rationale (§3 + cap=7).

**Mini-review brief rubric extension (added 2026-05-09 R6 R6-final ratification):** mini-review brief templates MUST include 3 standing checklist items: "(i) cite-drift detector clean; (ii) `node --import tsx scripts/codegen-errors.ts` clean diff; (iii) `cargo +stable fmt --all -- --check` clean." Mini-reviewer cannot return APPROVE without all three checks attested.

**Grep-the-pattern, not just the cite:** when CI flags an instance, grep the pattern across the worktree before pushing. `.err().expect()` flagged once → grep `\.err()$` → catches all instances. `[\`benten_other::Symbol\`]` flagged once → grep `\[\`benten_[a-z_]*::` across non-touched files.

**Cost-benefit:** 60-90s × N pre-pushes vs 15min × M CI cycles. Break-even at M=1 — even one avoided CI cycle pays for the discipline.

**JSON-artifact validation amendment (added 2026-05-13 Phase-4-Foundation R6 R3 ratification, folded under §3.5h instead of new pim-N codification):** when a wave touches structured JSON artifacts (lens JSONs, mini-review JSONs, ADDL pipeline outputs, manifest files), pre-push MUST run `jq . <artifact>` on each touched JSON file to verify well-formed structure. R6 R3 pattern-induction-meta-sweep `r6r3-meta-1` surfaced one instance: R6 R2 produced `r6-r2-plugin-arch-cap-policy-reviewer.json` with malformed structure (object/array mismatch at line 120) which orchestrator triage could have silently skipped findings on; fixed inline at R6-FP-2 but the discipline gap remained. Extension shape (NOT a new pim-N — folds under §3.5h's existing pre-push umbrella):

```bash
# For any *.json artifact touched by the PR:
git diff --name-only | grep -E '\.json$' | xargs -I {} sh -c 'jq . {} > /dev/null || echo "BAD JSON: {}"'
```

This composes with the existing §3.5h workspace pre-push 5-check. Cost: <1s per JSON file. Rationale: defense-in-depth — §3.5h covers Rust/TS source validation; JSON-artifact validation extends the same shape to structured-data PR artifacts.

**GREEN-CI-CONFIRMATION substrate clause (added 2026-05-13 Phase-4-Foundation R6 R3 ratification, folded under §3.5h MANDATORY-PRE-MERGE clause):** before admin-merge bypass per §3.14 Strategy-C batch-merge, orchestrator MUST verify that any NEW CI failures introduced by the PR are NOT regression vs the pre-existing main-side baseline. Compare PR CI failure list against `gh run list --branch main --limit 3 --json conclusion` — bypass is valid ONLY if all PR failures are pre-existing-on-main (same failure shape on the most recent N main runs). R6 R3 pattern-induction-meta-sweep `r6r3-meta-2` surfaced one instance: Wave-E admin-shell-e2e was admin-merged at PR #240 batch WITHOUT this confirmation; the rustls CryptoProvider fix landed as the admin-merge fix (commit `85ecb69`) — class-of-bug regression of pim-cite-drift-fp1-recurrence pattern (fix-pass after "approval" rather than before). Folded under §3.5h's MANDATORY-PRE-MERGE clause (the §3.14 batch-merge gate inherits this requirement).

**Cross-references:**

- memory `feedback_orchestrator_workspace_pre_push` — full rationale + recurrence log.
- memory `feedback_rustdoc_path_style_cite_brackets` — companion: cross-crate path-style cites use backticks-only NOT brackets, 4 recurrences in Phase-3 R5.
- memory `feedback_pim_cite_drift_fp1_recurrence` — R6-final ratification of the 4-instance MANDATORY-PRE-MERGE-AFTER-MINI-REVIEW-APPROVE clause; cross-references §3.6f SHAPE-not-SUBSTANCE + §3.5b HARDENED post-fix-doc-coupling.

**Cross-references:**
- memory `feedback_agent_local_preflight_blind_spots` carries the full rationale + 5-incident log from wave-8b.
- `.addl/phase-2b/r6-r4-pattern-induction-meta-sweep.json` — `r6-r4-pim-7` origin (Stable rustdoc strict-lint blind spot).

### 3.5i Mini-reviewer rebase-staleness pre-flight (added 2026-05-13 Phase-4-Foundation R6 R1 pim-N meta-sweep Candidate B ratification)

**MANDATORY for every mini-reviewer brief**: the agent's FIRST action MUST be a tree-state-freshness check against the target branch's merge-base with `origin/main`. If the branch is stale by 3+ commits (or the merge-base is older than the latest parallel sibling implementer's HEAD), the mini-reviewer SHOULD flag "rebase-staleness" as a top-level disposition and recommend rebase-before-merge.

**3-instance recurrence (Phase-4-Foundation R5):**
1. **G23-0a BLOCKER** — implementer forked at `0b453f0`; G27-A/B/C landed during dispatch making the sibling 3-PRs-stale; `gh pr create` exposed alarming "silently-deletes" diffs.
2. **G24-D BLOCKER** — same shape; mini-reviewer caught the rebase-staleness only after diff-inspection.
3. **G23-A OBS-paired** — flagged but not promoted because the diff happened to be content-disjoint.

**Why §3.5h doesn't catch this**: §3.5h workspace pre-push gates run against the branch's WORKING TREE, not against the branch's relationship to `origin/main`. A branch can pass §3.5h fully while being 5 commits behind main. The mini-review verdict is the right gate to surface staleness because the reviewer is the FIRST consumer who would see merge-base drift.

**Brief-template addition:** every mini-reviewer brief must include:

```
**FIRST ACTION (§3.5i rebase-staleness pre-flight):**
1. `git fetch origin && git merge-base origin/main <branch>` — record SHA
2. `git log --oneline origin/main ^<branch> | head -10` — list commits-ahead-of-merge-base
3. If >3 commits ahead OR if there's a parallel sibling implementer's
   HEAD newer than this branch's merge-base: flag "rebase-staleness"
   as a top-level finding + recommend `git rebase origin/main` before
   merge (orchestrator handles + dispatches mini-reviewer again on
   rebased HEAD).
```

**Cross-references:**
- memory `feedback_pim_n_mini_reviewer_rebase_staleness` — full pattern catalog + 3-instance log.
- `.addl/phase-4-foundation/r6-r1-pim-n-meta-sweep.json` — Candidate B origin.
- §3.5h MANDATORY-PRE-MERGE-AFTER-MINI-REVIEW-APPROVE — companion (covers lint/cite drift; this rule covers merge-base drift).

### 3.5j Stable-Rust clippy gate as §3.5h workspace pre-push addition (added 2026-05-13 Phase-4-Foundation R6 R2 methodology-critic solo lens 4-instance recurrence ratification)

**MANDATORY addition to the §3.5h pre-push 5-check**: orchestrator (+ implementer agents preferred) MUST also run `cargo +stable clippy --workspace --all-targets --features benten-eval/testing,benten-engine/test-helpers,benten-graph/testing -- -D warnings` IN ADDITION to the standard `cargo +1.95 clippy --workspace --all-targets --features benten-eval/testing,benten-engine/test-helpers,benten-graph/testing -- -D warnings` (MSRV compatibility check). The `--features` flag set mirrors `ci.yml:141` + §3.5l — without it, both gates surface 11+ E0432/E0433/E0560/E0599 compile errors against feature-gated `pub mod testing` consumers, leading reviewers to false-RED diagnosis (G-CORE-9 R3 L12-R3-MIN-2 ratification, 2026-05-24). Stable Rust ships clippy lints that MSRV 1.95 lacks; these fire silently in CI but pass locally on MSRV-only checks. The added pre-flight cost is ~30 seconds warm.

**4-instance recurrence on Strategy-C batch PR #240 (single multi-commit PR cycle):**
1. `clippy::manual_contains` on `R4b-FP-3 admin_ui_v0_public_surface_presence_pins.rs` (caught at PR #239 retroactively).
2. `clippy::too_many_lines` on R6-FP-A `r6fp_a_plugin_trust_blocker_closures.rs` adversarial test (146 lines / 100).
3. `clippy::no_effect_underscore_binding` on R6-FP-A `NoopManifestEnvelopeRechecker` structural constructibility check.
4. `clippy::drop` on Copy type (same Wave-A test, follow-up after fixing #3 with a different shape).

Each cycle cost ~9-15min CI wall + a fix-push commit; collapses to 0 if `+stable clippy` runs as the canonical pre-flight gate locally.

**Why MSRV 1.95 doesn't catch these:** rustc + clippy stabilize lints monotonically across stable releases; some lints exist on the stable channel but not on the MSRV-pinned older channel. The §3.5h workspace pre-push gate today implicitly assumes "MSRV clippy clean = stable clippy clean," which is empirically false.

**Composes with:** §3.5h workspace pre-push gate (this is an additive sub-rule); §3.5b HARDENED post-fix doc-coupling; §3.5g cross-language rule-mirror; §3.5i rebase-staleness pre-flight.

### 3.5l Combined/consolidated-branch FULL-WORKSPACE pre-push verify — scoped/per-crate verification is structurally insufficient for cross-lane branches (added 2026-05-16 refinement-audit-2026-05 ratification; Ben-approved decision-queue item #7)

**RULE.** Before pushing ANY branch that combines work from ≥2 lanes/crates (Strategy-C batch branches, consolidated/mega batches, cross-lane fix-passes, anything built via `git merge <other-branch>`), the builder/orchestrator MUST run the **FULL-WORKSPACE** CI-equivalent gate locally before push — NOT scoped/per-crate (`-p <crate>`) checks:
- `cargo build --workspace --all-targets --locked --features benten-eval/testing,benten-engine/test-helpers,benten-graph/testing`
- `cargo +stable fmt --all -- --check`
- `cargo +stable clippy --workspace --all-targets --features <same> -- -D warnings`
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` (broken-intra-doc-links is `-D`; only runs on stable CI legs)
- `cargo nextest run` with the CI profile + feature set
- the cite-drift detector

**Why (the load-bearing evidence — refinement-audit-2026-05):** scope-clean reviews + scoped per-crate pre-flight (`cargo check -p X`) structurally CANNOT see cross-crate consumers. This produced REAL regressions in **six** scope-clean-APPROVE'd lanes (#1244/#1245/#1248/#1249/#1255/#1256 — cross-crate consumers, source-scraping pins) AND, most expensively, PR #1262 (the campaign-final consolidated batch) bounced **four** full-CI cycles — 2 nextest fails → cite-drift → stale-test → `cargo doc` broken-link — because each ~50-min full-CI run only surfaced the NEXT gate the scoped/local verify missed. A single full-workspace local gate before push collapses those N serial CI cycles into 1. The cite/non_exhaustive/rustdoc/cross-crate-consumer failures are exactly the class invisible to `cargo check -p <crate>` but caught by `--workspace --all-targets` + the stable-only doc/clippy/fmt steps.

**Two coupled sub-disciplines (both mandatory on combined branches):**
1. **Full-workspace pre-push verify** (the 6 checks above) — not scoped.
2. **Cascade-triage before merge** — when a combined/lane branch's CI fails, FIRST classify via authoritative Jobs-API `conclusion` (cancelled=infra vs failure=real) and reproduce-don't-assume the real root cause (the #1262 rustdoc cause was NOT the orchestrator's hypothesis — the agent reproduced and found the true cause); cross-crate-consumer cascade is the default suspicion for uniform multi-leg `build+test` failures.

**Cost:** one full-workspace build/test pass (~minutes warm with sccache) per combined-branch push. Trivially cheaper than N×~50-min CI bounce-cycles + the orchestrator-attention thrash they cause.

**Composes with:** §3.5h MANDATORY-PRE-MERGE 5-check (this generalizes it from scoped→full-workspace specifically for combined branches); §3.5j stable-clippy gate (subsumed here for combined branches); §3.5i rebase-staleness; §3.5b doc-coupling; the authoritative-set-diff closure-verify discipline (python set-difference, never `comm`-over-`sort -n`, never sequential-gh-loop — the #1235/238-false-alarm lesson).

**Cross-references:**
- memory `feedback_pim_n_stable_clippy_gate` — full rationale + 4-instance log.
- `.addl/phase-4-foundation/r6-r2-methodology-critic-solo.json` — origin lens for the 4-instance recurrence catalog.

### 3.5b Post-fix doc-coupling pre-flight (added 2026-05-02 R6 R3 ratification of pim-1)

**MANDATORY for every implementer brief that changes any public-shape struct, signature, error variant, primitive runtime arm, or other surface that has prose anywhere in the repo.** R6 Round 3's pattern-induction meta-sweep found 7+ recurrences of "code-fix wave silently leaves stale prose in 3-5 .md files" across Phase 2b. Each fix-pass closed the surfaced finding but the agent's diff-scoping didn't sweep adjacent docs that referenced the same surface — so subsequent reviewers caught the doc-vs-code drift as a NEW finding, dispositioned fix-now, and the cycle repeated. Two notable examples: PR #66 fixed Inv-4 narrative across 3 docs but missed `QUICKSTART.md` + `PAPER-PROTOTYPE-REVALIDATION.md` (caught by R6-R3 doc-engineer-redux as `r6-r3-doc-1`); PR #68 added 25 lines to `primitive_host.rs` and invalidated 6 cite sites across SECURITY-POSTURE / INVARIANT-COVERAGE / ERROR-CATALOG (caught as `r6-r3-doc-2`).

**The pre-flight:** before pushing, the agent runs a documentation grep scoped to every public-shape change in the diff:

```bash
# For each public-shape change (struct, signature, error variant, primitive arm, etc):
grep -rn "<symbol-name>" docs/ .addl/ README.md crates/*/src/lib.rs packages/engine/src/

# For each file:line cite that may have shifted due to insertions/deletions:
# (the cite-precision deep-sweep tooling at scripts/check-cite-precision.sh is canonical)
```

**For each match found:** verify the prose still describes the post-change reality. If stale, update inline as part of the same PR (NOT as a follow-up). If the prose intentionally describes the pre-change state for historical narrative, leave it but add a "(pre-<change-id>)" qualifier so the next sweep recognizes it as deliberate.

**Brief-template addition:** every implementer brief that touches a public surface MUST include "Doc-coupling pre-flight per §3.5b: list every prose surface that referenced the changed symbol; verify each is updated or explicitly justified." If the agent reports zero matches, that's an acceptable answer — but the grep MUST have been run.

**Composes with §3.6:** consumer-audit (§3.6) catches code-side drift across the 4 surfaces (eval / engine / napi / TS). Doc-coupling (§3.5b) catches prose-side drift across docs / .addl / README. Both run on every public-shape change.

**§3.5b hardening (added 2026-05-03 R6-R4 ratification of pim-1 recurrence):** the original §3.5b discipline catches OLD cite drift (verifies prose still describes post-change reality) but does NOT catch NEW-cite-wrong-symbol or NEW-cite-phantom-symbol shapes. R6-R4 doc-engineer-redux found 2 MAJOR instances of the latter inside PR #70's own cite-precision migration (cited `engine.rs::dispatch_call_inner` when the actual read site was `dispatch_call_with_mode_and_trace`; cited phantom `primitive_host.rs::route_read_with`). For every `path::symbol` cite ADDED or CHANGED:

1. **`grep -n 'fn <symbol>' <path>` MUST return a hit.** If the grep returns zero hits, the cite is phantom — surface to orchestrator before push.
2. **Verify the cited construct actually appears in the named fn body** (not in a caller, not at file-scope outside the fn). If the cited construct lives in the caller, cite the caller instead.
3. **For high-churn surfaces (`primitive_host.rs`, `engine_views.rs`, `evaluator.rs`, `lib.rs`, `builder.rs`, `wait.rs`, `subscribe.rs`, `mermaid.ts`, `dsl.ts`), symbol cites are MANDATORY (not "preferred").** Promoted from "prefer" to "MUST" 2026-05-03 R6-R5 cite-precision-deep-sweep-redux ratification of `r6-r5-cp-1` (4th cluster-E recurrence — a bare *evaluator.rs* line cite at *sandbox.rs* should have migrated to symbol form during R6-R4 but stayed bare) + `r6-r5-cp-2` (a bare *lib.rs* line cite in NEW §7.9 prose). The "prefer" framing left agents room to leave bare line cites against high-churn surfaces; promotion to MUST closes the loophole.

4. **For every cite in NEW prose blocks (not just deliberately-changed cites), run points 1+2.** Added 2026-05-03 R6-R5 ratification of pim-9 (incidental cites in NEW prose blocks). The agent's interpretation of "ADDED or CHANGED" was "cite I targeted" rather than "every cite in the diff including ones I incidentally wrote in NEW prose." Confirmed via 3 instances closed in same R6-R5 cycle (PR #74's §7.9 NEW backlog entry phantom + PR #74's dsl.ts NEW comment block phantom + R6-R5 streaming `dispatch_primitive` phantom in test-file new prose). New rule: grep-symbol-verify EVERY `path::symbol` cite that appears in any line you added/edited, regardless of whether you "consciously" added it or it landed as part of broader prose. **EXTENSION (added 2026-05-09 R6 R6-final ratification of 3plus-r6-final-1 carry-over after 122+ closed substance instances across the cluster):** the same post-fix doc-coupling pre-flight applies to **`#[ignore = "..."]` directives + `unimplemented!("...")` + TODO comments + module-level `//!` rustdoc rationale strings** that name a wave / destination / phase / merged gate. For each such rationale string in your diff: verify the named target has not landed; if it has landed, retense to the current correct destination NOW (NOT as a follow-up). Sweep includes test-helper module-level docstrings — corr-r6-r2-1 hit 3 sibling test files (`benten-sync/tests/{graph_encoded_state.rs,transport_loopback.rs}` + `benten-engine/tests/loro_version_chain.rs`) all citing the same merged gate. Cumulative count after PR #171 batch-2 corr-r6-r2-1 (6 more instances) on top of Wave-E PR #168 (116+) = 122+ stale-rationale instances closed across 3 rounds while the codification HARDENING was repeatedly re-recommended without landing. The rule is now LANDED.

5. **Crate-prefix-cite discipline (added 2026-05-05 R5-W1 ratification of 3rd recurrence).** When a doc-comment INSIDE a crate (`crates/<X>/src/<file>.rs` or `bindings/napi/src/<file>.rs` etc) cites a path like `tests/<file>::<symbol>`, the cite-drift detector resolves the path RELATIVE TO WORKSPACE ROOT — bare `tests/...` resolves to `<workspace-root>/tests/...` not the crate-relative `crates/<X>/tests/...`. **MUST always use the full workspace-relative path** including `crates/<X>/`, `bindings/napi/`, `packages/engine/` etc prefix. Recurrence shape: PR #95 R3-D had 9 line-cite drift findings of this shape; PR #97 R2-FP-B had 3 stale `_napi` cites + bare `tests/` cites; PR #102 R5-W1 G13-A had 2 bare `tests/` cites in inline-mod-tests doc comments. **Brief-template addition:** every implementer brief MUST cite the rule `for any cite in code or prose, use the full workspace-relative path (crates/<X>/, bindings/napi/, packages/engine/, etc); bare relative paths fail cite-drift detector at CI even when local crate test path resolution works in the editor or with rust-analyzer`.

**Cross-references:**
- memory `feedback_post_fix_doc_coupling_preflight` — full rationale + recurrence log
- `.addl/phase-2b/r6-r3-pattern-induction-meta-sweep.json` — pim-1 origin finding
- `.addl/phase-2b/r6-r3-doc-engineer-redux.json` — `r6-r3-doc-1` + `r6-r3-doc-2` instances
- `.addl/phase-2b/r6-r4-doc-engineer-redux.json` — `r6-r4-doc-1` + `r6-r4-doc-2` (wrong-symbol + phantom-symbol — origin of the §3.5b hardening)
- `.addl/phase-2b/r6-r4-cite-precision-deep-sweep.json` — `r6-r4-cp-1` (3rd recurrence of `:901` family — origin of the high-churn-surface symbol-cite preference)
- `.addl/phase-2b/r6-r5-cite-precision-deep-sweep.json` — `r6-r5-cp-1` + `r6-r5-cp-2` (origin of point 3 promotion + high-churn-surface list expansion)
- `.addl/phase-2b/r6-r5-pattern-induction-meta-sweep.json` — `r6-r5-pim-9` (incidental-cites-in-NEW-prose origin of point 4)
- `.addl/phase-2b/r6-r5-streaming-systems-redux.json` — `r6-r5-stream-1` (phantom `dispatch_primitive` instance of pim-9)

### 3.5c Orchestrator-self-doc-coupling pre-flight (added 2026-05-04 Phase-3 R1 ratification of pim-12)

**§3.5b applies to IMPLEMENTER agents who push PRs. §3.5c is the ORCHESTRATOR-shape sibling: when the orchestrator does direct integration edits to a doc, the same doc-coupling discipline applies to the orchestrator's own diff.**

Origin: Phase-3 R1 pattern-induction-meta-sweep (2026-05-04) flagged 3+-recurrence in this single session of orchestrator-direct integration edits leaving stale prose:
- STALE-PROSE-1: plan-header carve-out at `.addl/phase-3/00-implementation-plan.md:10` still said "DEFERRED-PENDING-EXPLORATION" after the disposition was integrated as RESOLVED.
- STALE-PROSE-2: TOC entry at `:24` referenced "DEFERRED-PENDING-EXPLORATION callouts" after §7 OOS subsection was renamed.
- 12-deferred-orphans-cluster (commit `56cde79` retroactively): orphan cleanup rewrote `Cargo.toml` + 2 source files but missed 25+ "Phase 2c" phantom destination instances — orchestrator's own directed-edit pass didn't sweep adjacent narrative.

**The pre-flight (orchestrator-applied to own diff before declaring integration done):**

```bash
# For each named symbol / disposition / wave / D-list-item edited in this pass:
grep -rn "<edited-symbol>" .addl/phase-N/ docs/ CLAUDE.md  # Or scoped to the doc family changed.

# Specifically: scan for stale narrative referring to the PRE-edit state.
# Common stale-prose shapes: TOC entries, plan-header carve-outs, status-table cross-refs,
# RESOLVED-but-still-says-DEFERRED, count drift ("23 D-PHASE-3-* items" when count is now 26),
# placeholder strings ("D-PHASE-3-RESOLVED-at-R1" without a number).
```

**For each stale-prose match:** update inline as part of the same integration pass. If stale prose intentionally describes the pre-edit state for historical narrative, add a "(pre-<edit-id>)" qualifier.

**Why orchestrator-shape needs its own codification:** §3.5b targets implementer agents who scope their diff narrowly (single-feature PR). Orchestrator integration edits are inherently broader (cross-section consistency in a single doc). The failure mode is the same shape but the trigger surface is different — implementers can be reminded by their brief; orchestrator must self-remind.

**When to apply:** every orchestrator-direct integration pass into a tracked doc (plan revision, HANDOFF update, FULL-ROADMAP amendment, CLAUDE.md edit, dispatch-conventions edit). Run the grep before declaring "integration done" to user.

**Cross-references:**
- §3.5b — implementer-shape sibling (carries forward the same discipline; this is the orchestrator-shape extension).
- `.addl/phase-3/r1-pattern-induction.json` — pim-12 origin finding.
- `.addl/phase-3/plan-final-consistency-check.json` — caught STALE-PROSE-1 + STALE-PROSE-2 immediately after the orchestrator's first integration pass; this codification prevents the recurrence.

**§3.5c amendment 2026-05-05 — NEW shape (iii) tools-as-meta-spec (4th-instance threshold MET).** Phase-3 R4 large-council Round-1 pattern-induction-meta-sweep flagged a 4th instance of pim-12 with a NEW shape: orchestrator-built tooling that hardcodes a workspace fact in its source-of-truth, where the workspace fact then drifts. Concrete: the historical *cite-drift-detector lib.rs* `EXPECTED_CRATES` constant hardcoded `crates: 8` as the source-of-truth for numeric-claim-drift detection; R3-A + R3-C added `benten-id` + `benten-sync` to workspace `members` post-merge, taking the workspace to 10 crates; the detector then produced 28 false-positive `numeric-claim-drift` findings against Phase-3 docs that correctly said `10-crate`. The detector's own rustdoc had even called out this exact recurrence shape preemptively.

**The amendment:** when orchestrator builds tooling that consumes workspace facts (crate count, primitive count, invariant count, etc.) as its source-of-truth, the source-of-truth derivation MUST be workspace-aware (read from canonical source at runtime; e.g., parse `Cargo.toml` `members =` list) rather than hardcoded. Hardcoded facts in tooling are themselves stale-prose-shaped — they drift the moment the workspace changes.

**Detection check (added to §3.5c pre-flight):** when editing tooling source under `tools/`, `crates/*-detector/`, or any orchestrator-built validator, grep for `// source-of-truth` / hardcoded workspace numbers and verify they're derived rather than literal. If literal, lift to runtime derivation.

**Cross-references for shape (iii):**
- `.addl/phase-3/r4-r1-pattern-induction.json` — origin finding for shape (iii) + 4th-instance evidence.
- *tools/cite-drift-detector/src/lib.rs* (`EXPECTED_CRATES` constant; historical Phase-3 site of the recurrence — since refactored to runtime workspace-derivation).

### 3.5d Pattern-induction lens FIX-NOW routing (added 2026-05-05 Phase-3 R4-R1 ratification of pim-15)

**When a pattern-induction lens (or any review lens) identifies a gap in test coverage / disposition / cross-cutting concern, the gap MUST be routed FIX-NOW into the relevant R3 partition's brief, NOT held in a triage backlog for orchestrator-only consumption.**

Origin: Phase-3 R4 large-council Round-1 pattern-induction-lens (2026-05-05) accurately predicted 3 zero-pin gaps that orchestrator triage could have caught earlier had FIX-NOW routing been the default:
- **CLR-1 D-PHASE-3-21** zero R3 pins for user-view replication semantics (caught by ivm-correctness + pattern-induction lens; surfaced too late to land in R3 corpus first time);
- **Compromise #22 phantom-destination** zero R3 pins for peer-DID metadata-leakage (3-lens cluster: networking + security + pattern-induction);
- **D-PHASE-3-26 rate-limits** zero R3 pins (D-F resolution didn't propagate to R3 partition briefs).

Each gap was independently flagged by pattern-induction lens; each was a phantom-destination shape (HARD RULE rule-12 clause-b violation: lens findings deferred-to-triage instead of FIX-NOW-routed-to-partition).

**The discipline:** when a pattern-induction (or any cross-cutting) lens finding identifies a partition-level coverage gap, the orchestrator triage step is FIX-NOW-route to the named partition's R4-FP (or equivalent) brief — not "carry to next round" / "note for triage" / "orchestrator-direct cross-cutting list." If the partition is closed (PR merged), the routing target is the partition's next round of work; if no next round is planned, the partition needs one.

**Cross-references:**
- memory `feedback_no_defer_HARD_RULE` — clause-b NAMED + REAL destination requirement.
- `.addl/phase-3/r4-r1-pattern-induction.json` — pim-15 origin finding.
- `.addl/phase-2b/HANDOFF-2026-05-03-phase-2b-close.md` Compromise #22 lineage.

### 3.5e Disposition-verification at HEAD (added 2026-05-05 Phase-3 R4-R2 ratification of pim-16)

**When triage closes a finding by naming a plan-row edit OR a cross-partition deferral OR an "orchestrator-direct cross-cutting list" entry, the closure MUST be verified at HEAD before being marked APPLIED — verify that the named test-pin/symbol/file/section actually appears at HEAD, not just that the plan-row text was updated to *describe* it.**

Origin: Phase-3 R4 large-council Round 2 (2026-05-05) pattern-induction lens flagged pim-16-CANDIDATE: 3+-recurrence MET across 4 R4-R1 lenses (browser-runtime + test-coverage-completeness + code-as-graph + doc-engineer) with 22+ instances of "plan-row says yes; test-pin doesn't exist at HEAD" — concretely: ivm-r4-r2-1 BLOCKER (PR #92 mini-review claimed BELONGS-ELSEWHERE-NAMED-NOW routing to "R3-D napi territory" but PR #95 R3-D R4-FP commit `a4bd49e` did NOT land the napi pin; verified by `find bindings/napi/tests` + grep returning zero matches at HEAD).

Pim-16 names a triage-verification asymmetry distinct from pim-12 (orchestrator-self-doc-coupling pre-flight) and pim-15 (lens-FIX-NOW-routing): pim-16 catches the gap between "the plan-row was updated" and "the test-pin / symbol / artifact actually exists at HEAD."

**The discipline (orchestrator-applied at every triage close):**

For each finding marked CLOSED with disposition (b) BELONGS-ELSEWHERE-NAMED-NOW or (c) DISAGREE-WITH-EXPLANATION:
1. **Cite the destination at the file:symbol granularity.** Not "R3-D territory." Concretely: `bindings/napi/tests/<name>.rs::<test_fn>` or `crates/<crate>/src/<file>.rs::<symbol>` or `docs/<doc>.md::§<section>`.
2. **Verify the destination exists at HEAD.** Run `grep -n "<symbol>" <path>` or `find <directory> -name <filename>` BEFORE marking APPLIED in the triage doc / HANDOFF / mini-review JSON. If the destination doesn't yet exist, the disposition is NOT closed — it's deferred + the verification step IS the next concrete action.
3. **Plan-row edits ≠ test-pin landings.** When the disposition is "added X to plan-row Y," verify that test-pins/symbols named IN that plan-row also appear at HEAD; the plan-row is a description of intent, not the closure itself.
4. **Cross-partition handoffs need closure-loop verification.** When R3-X says "this finding will be picked up by R3-Y" or "by orchestrator-direct," verify post-merge that R3-Y's PR DID pick it up; if it didn't, the disposition is unfilled + needs a fresh fix-pass.

**Why this needs codification beyond §3.5b/§3.5d:**
- §3.5b post-fix doc-coupling pre-flight catches stale-prose drift INSIDE the doc family changed by the fix-pass.
- §3.5d FIX-NOW routing catches lens findings being held in triage instead of routed.
- §3.5e catches the failure mode where routing happened (per §3.5d) and the brief was authored, but the IMPLEMENTER agent didn't pick up that finding inside their PR — and the orchestrator marked APPLIED based on the brief shape rather than the merged-PR contents.

**Detection workflow at triage close:**

```bash
# For each finding marked CLOSED with named destination:
# (a) Plain grep at the cited file:symbol granularity:
grep -n "<test_fn>" <cited_path>
# (b) Or find for new-file destinations:
find <directory> -name "<filename>"
# (c) Or git log filter for cross-partition handoffs:
git log --oneline <partition_PR_sha> -- <cited_path>
```

If any of (a)/(b)/(c) returns empty for an APPLIED-marked finding, the disposition is unfilled — re-route to fresh fix-pass before declaring the round closed.

**Cross-references:**
- `.addl/phase-3/r4-r2-pattern-induction.json` — pim-16-CANDIDATE origin finding (3+-recurrence MET, 22+ instances).
- `.addl/phase-3/r4-r2-ivm-correctness.json` — load-bearing pim-16 BLOCKER instance (r4-r2-ivm-1 napi phantom-destination).
- §3.5b — implementer-shape sibling for doc-coupling.
- §3.5d (pim-15) — routing sibling.

### 3.5f Orchestrator-DEFERRAL-to-gitignored-destination handshake (added 2026-05-05 Phase-3 R4-R3 ratification of pim-17)

**When an implementer brief asks an agent to make a change in a gitignored destination (`.addl/`, `dispatch-conventions.md` under `.addl/phase-2b/`, gitignored docs), the brief MUST acknowledge the agent CANNOT make that edit from their worktree, AND the orchestrator MUST capture the deferral in a tracked-by-orchestrator destination + apply it BEFORE declaring the round/wave closed.**

Origin: Phase-3 R4 large-council Round 3 (2026-05-05) pattern-induction lens flagged pim-17-CANDIDATE: 3+-recurrence MET with 6 instances of the orchestrator-DEFERRAL-to-gitignored-destination handshake shape across the R4 R2-FP cycle:

1. PR #96 (orchestrator-direct cross-cutting cleanup) precedent for `.addl/` cite-form fixup — agent surfaced 4 deferred cites, orchestrator applied
2. Commits `1ba6fee` + `4fdc757` (R2-FP-B PR #97) — 5 fresh `.addl/` symbol cites added then immediately fixup-PR'd (literal repeat of #96 shape; cite-drift detector caught at CI)
3. R2-FP-A `r4-r2-ivm-9` plan §3 G15 reviewer roster addition — agent surfaced as DEFERRAL, orchestrator applied directly to `.addl/phase-3/00-implementation-plan.md:441`
4. R2-FP-E `R3-CPC-6` r2-test-landscape.md §13 ownership table retense — agent surfaced as DEFERRAL, orchestrator applied directly
5. R2-FP-E pim-16 §3.5e codification itself — agent surfaced as DEFERRAL, orchestrator applied directly
6. R2-FP-C `ds-r4r2-7` plan-row addition (G14-D ↔ G15-A shared-trait callout) — agent surfaced as DEFERRAL, orchestrator applied directly

**Why pim-17 is structurally distinct from pim-12 / pim-15 / pim-16:**
- pim-12 (§3.5c): orchestrator-self-doc-coupling pre-flight inside a tracked doc — catches stale-prose drift inside the doc family changed.
- pim-15 (§3.5d): lens-FIX-NOW-routing — catches lens findings being held in triage instead of routed to a partition's brief.
- pim-16 (§3.5e): disposition-verification at HEAD — catches plan-row-edit ≠ test-pin-landed asymmetry on the tracked-tree side.
- **pim-17 (§3.5f): gitignored-destination handshake** — covers the brief-authoring discipline + orchestrator-application discipline for items where the destination IS gitignored, so the implementer agent CANNOT close the loop themselves.

**Failure mode pim-17 prevents:**
- Agent writes "DEFERRAL: orchestrator-direct edit to `.addl/phase-3/00-implementation-plan.md:441` to add streaming-systems reviewer to G15 roster"
- Orchestrator marks the round closed without checking whether they actually applied it
- The `.addl/` file lives only in main worktree (gitignored), so CI doesn't catch the missing edit
- Months later, an R5 wave reads the plan-row, sees the (missing) reviewer roster entry isn't there, and dispatches without the streaming-systems lens — surface coverage gap

**The discipline (orchestrator + brief co-applied):**

**Brief side (when authoring an implementer brief):**
- For each scope item, identify whether the destination is **tracked** (lands in PR diff) or **gitignored** (cannot land in PR diff).
- For gitignored-destination items: brief MUST explicitly note "DEFER to orchestrator-direct main-worktree edit" + name the file path + cite the line/symbol level destination.
- Agents CANNOT silently skip gitignored items — they MUST surface in PR body's "Deferrals" table.

**Orchestrator side (when triaging round closure):**
- Maintain an orchestrator-tracked TODO list of every gitignored-destination DEFERRAL surfaced.
- BEFORE declaring round/wave closed: walk the list + verify each gitignored edit is applied in main worktree.
- The verification grep is on disk in main worktree (since gitignored content isn't in CI): `grep -n "<symbol>" <gitignored-file>`.
- HANDOFF doc captures all gitignored deferrals applied in the round (so post-compact agent can verify).

**Why this can't be automated by tooling:**
- Cite-drift detector won't catch a missing `.addl/` edit — it only validates tracked-tree cites.
- CI won't catch missing `.addl/` content — gitignored.
- Only the orchestrator's own discipline + HANDOFF tracking can close the loop.

**Cross-references:**
- `.addl/phase-3/r4-r3-pattern-induction.json` — pim-17 origin finding (6-instance threshold MET in R4 R2-FP cycle).
- `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md` UPDATE 2026-05-05 sections — example of orchestrator-tracked TODO list capturing all 6 gitignored deferrals applied at R4 close.
- §3.5b — tracked-tree implementer-shape sibling.
- §3.5e (pim-16) — tracked-tree disposition-verification sibling.

### 3.5g Cross-language rule-mirror atomic-update discipline (RENAMED 2026-05-09 R6 R6-final ratification of pim-N-cross-language-rule-mirror; original "ErrorCode catalog 4-surface" is the FIRST instance of a broader pattern)

**Generalization (added 2026-05-09 R6 R6-final):** when both a TypeScript surface and a Rust surface encode the SAME class-naming-rule / DSL-syntax / wire-format / AST shape / codegen-helper table, an edit to one side MUST atomically update the other side or its drift-defense surface (parity test / generation regen / mirror-test). Active mirrors at HEAD:

1. **ErrorCode catalog 4-surface** (origin instance — see below).
2. **`COMPOUND_STEM_EXPANSIONS` table** at `scripts/codegen-errors.ts` (TS-side) ↔ `crates/benten-engine/tests/code_to_ctor.rs:118+` (Rust-side test that implements the same lowercase-fallback rule). Edits to either side MUST sweep the other; PR #171 fp-5 `e6aea5d` was an instance of the Rust-mirror gap (TS added 2-entry expansion table; Rust test panicked on `EValueFloatNonfinite`/`EValueFloatNonFinite` mismatch).
3. **Type-name cross-doc mirror** (added 2026-05-13 Phase-4-Foundation R6 R1 ratification — L14 R7-spec-compliance lens caught `TauriRender` vs `TauriRenderer` 8-cite drift across `docs/ADMIN-UI.md` + `docs/ARCHITECTURE.md` + materializer comments after Wave-E shipped struct `TauriRenderer`): for any `pub` type whose name is referenced across ≥2 docs OR cited in test/comment narratives, an edit to the Rust struct name MUST sweep all cross-doc cite sites in the same PR. Drift-defense surface: doc-coupling pre-flight per §3.5b + cite-drift sentinel test (already catches stale path-style cites; extends naturally to type-name claims). Single-character renames count.
4. **Cross-tool config mirror — same security/lint discipline split across separate config files** (added 2026-05-13 Phase-4-Foundation R6 R2 methodology-critic solo lens ratification — R6-FP-E Wave-E added 14 new RUSTSEC ignores to `deny.toml` for `cargo-deny` (bringing total to 18) but missed mirroring the cargo-audit step in `.github/workflows/supply-chain.yml` + missed 1 advisory `RUSTSEC-2024-0429` glib that surfaced only via cargo-audit's separate scan, requiring follow-up commit `2b96091` that brought parity to 19 entries each side; cargo-audit reads its own config + ignore-list separate from cargo-deny). For any rule encoded in TWO separate same-language config files (deny.toml + CI workflow audit invocation; rustfmt.toml + .editorconfig; clippy.toml + Cargo.toml `[lints]`; etc.), edits to one MUST atomically update the other. Drift-defense surface: pre-push gate runs BOTH tools against the SAME ignore-list source-of-truth (e.g. CI workflow generates `--ignore` flags from deny.toml).
5. **Cohort-mint subset-closure mirror — per-cohort ErrorCode mint REQUIRES a dedicated subset-closure test** (added 2026-05-16 ST-ERRORS lane ratification of Fwd-2 #1019; 3+-recurrence threshold MET — G23-A / G23-B / G24-D each shipped this shape ad-hoc without formal codification, per `feedback_3_plus_recurrence_deep_sweep`). When a wave mints a NAMED COHORT of ≥2 related `ErrorCode` variants (a feature family: thin-client / schema / plugin / materializer / future Phase-4-Meta `E_SANDBOX_COMPONENT_*` / `E_LIGHT_CLIENT_*` / identity-recovery codes), the SAME PR MUST land a dedicated subset-closure test file at `crates/benten-errors/tests/error_codes_<cohort>_subset_closure.rs` that: (i) round-trips every cohort variant via `from_str` ↔ `as_static_str`; (ii) mirrors the cohort's typed-error registry array (`G2X_X_ERROR_CODES` / `<COHORT>_ERROR_CODES` const) at `crates/benten-platform-foundation/tests/common/<cohort>_fixtures.rs` so the Rust enum cannot drift from the test-side registry consumed by downstream atomic-mint pins; (iii) documents any cohort-internal renames in the test-file header (precedent: `E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE` → `E_PLUGIN_DEVICE_ATTESTATION_FORGED`). One-cohort-per-file: a new feature family gets its OWN closure file, NOT bundling into a prior cohort's file (the G27 `E_REGISTRY_DISCOVERY_TIMEOUT`-into-G24-D bundling was the early-warning cohort-boundary slippage that triggered this ratification). Drift-defense surface: the subset-closure test itself (it fails if the enum drifts from the fixtures registry); this rule makes its CREATION mandatory rather than ad-hoc-rediscovered by each cohort author reading prior cohort files. Cross-reference: INTERNALS.md `crates/benten-errors/INTERNALS.md §5` enumerates the 3 origin instances; memory `feedback_pim_cross_language_rule_mirror` carries the active-mirrors registry.
6. **Public/wire-crossing error variant first-class mirror** (added 2026-05-24 Ben-ratification generalizing the F-3 finding on PR #1339 G-CORE-DSL chunk-3 — `CompileError::Backend(String)` was added as first-class catalog entry but `CompileError::Io(std::io::Error)` was NOT; post-PR napi `mapNativeError` fell to `E_UNKNOWN` synthetic for DSL Io errors, materially crossing the wire as an untyped surface). **Rule**: for any Rust error type `T` (typically named `*Error`) where variants cross a public/napi/wire surface, EVERY such variant MUST have a corresponding `benten_errors::ErrorCode` enum entry + full 4-surface mirror (Rust enum + `as_static_str` + `from_str` + `routed_edge_label` + TS class + `CODE_TO_CTOR_GENERATED` + ERROR-CATALOG.md narrative + CATALOG_VARIANT_COUNT bump). **"Crosses a public/wire surface" means**: variant is constructed in a `pub` fn that's part of the crate's public API; OR constructed in a napi binding return path; OR serialized into on-disk/on-wire format; OR Display output ships to clients via deployed binaries. **Doesn't count** (variant can stay non-first-class): only constructed in `#[cfg(test)]` code; `#[doc(hidden)]` and explicitly excluded from semver; wrapped at the napi boundary into a SEPARATE first-class ErrorCode (then the WRAPPER is the catalog entry). **Enforcement**: extends `scripts/drift-detect.ts` with a sibling scanner — for each `pub enum *Error` in `crates/*/src/`, for each variant constructed outside `#[cfg(test)]` code, assert there exists a mapped ErrorCode entry with full mirror. Named exceptions via catalog annotations (`reachability: ignore`, `non-first-class: <reason>`). **Drift-defense surface**: the new scanner runs as part of `npm run drift:errors` + fails CI if any variant lacks the mirror. **Why this beats discretionary decisions**: pre-amendment, every new error variant required ad-hoc judgment "is this worth a first-class ErrorCode + mirror?"; post-amendment, the answer is automatic ("yes if it crosses a public/wire surface; named exception otherwise"). Sibling memory: `feedback_pub_error_variant_first_class_mirror` carries full rationale.
7. **Future shapes** (any DSL syntax mirror, any AST mirror, any wire-format mirror) are subject to the same discipline as added.

**Abstract rule:** for any cross-language symmetry where both Rust and TS encode the same rule, every edit MUST identify both sides + sweep both sides + verify the drift-defense surface (parity test / generation regen / mirror-test) clean before push.

**Cross-references for this generalization:**

- `.addl/phase-3/r6-final-3+-recurrence-deep-sweep.json` — finding `3plus-r6-final-3` ratification origin (3 instances: ErrorCode catalog + PR #171 fp-4 codegen-regen-forgotten + PR #171 fp-5 COMPOUND_STEM_EXPANSIONS Rust-mirror gap).
- memory `feedback_pim_cross_language_rule_mirror` — full rationale + active-mirrors registry.

---

**ORIGINAL DISCIPLINE (FIRST INSTANCE — ErrorCode catalog 4-surface, added 2026-05-08 Phase-3 R5 wave-8a ratification of pim-N-catalog-drift):**

**MANDATORY when adding, removing, or renaming any `ErrorCode` variant in `crates/benten-errors/src/lib.rs`, OR when mutating any `Thrown at:` / `fixHint` / message-template wording in `docs/ERROR-CATALOG.md` (these fields cascade to the generated TS via the codegen).** Four surfaces must move in the SAME PR — none can lag:

1. **`crates/benten-errors/src/lib.rs`** — the `ErrorCode` enum variant + its `as_str` arm (and any classification helpers like `routed_edge_label`).
2. **`crates/benten-errors/tests/stable_shape.rs`** — `ALL_CATALOG_VARIANTS` array + `CATALOG_VARIANT_COUNT` literal.
3. **`docs/ERROR-CATALOG.md`** — narrative entry (firing condition / wire shape / consumer-side contract).
4. **`packages/engine/src/errors.generated.ts`** — regenerated via the codegen step (`pnpm --filter @bentenai/engine generate-errors` or equivalent); the TS class hierarchy must reflect the new variant.

**Confirmed recurrences (≥3+ threshold MET):**

1. **G16-B** (R5 wave-6b) — added iroh transport variants; #2 + #4 lagged, mini-review caught.
2. **G16-D** (R5 wave-6b) — added handshake-protocol variants; #3 lagged, mini-review caught.
3. **G19-C2** (R5 wave-7) — added STREAM ESC variants; #2 + #4 lagged, mini-review caught.
4. **G20-A2** (R5 wave-8a) — added `WaitTtlExpired` + `WaitTtlInvalid` + `WaitMetadataMissing`; mini-review v1 NEEDS-FIX-PASS BLOCKER mr-1 (catalog 4-surface drift); fix-pass at `193088b` closed.
5. **W9-T6** (R5 wave-9) — wording-only mutation: updated `E_INV_CONTENT_HASH` `Thrown at:` line + `fixHint` in catalog without regenerating `errors.generated.ts`. T7 detector caught it post-rebase. Fix-pass at `edcbd2f` closed by re-running codegen. **This instance promoted the trigger condition from "variant-add only" to "variant-add OR catalog-prose-mutation"** — the codegen reads catalog wording (`Thrown at` + `fixHint`) and stamps it onto the generated TS class, so prose changes cascade.

**The discipline (brief + agent + mini-reviewer co-applied):**

**Brief side (when authoring an implementer brief that adds an ErrorCode variant):**
- Brief MUST enumerate all four surfaces explicitly in the scope-list.
- Brief MUST require the agent to run the codegen step locally + commit the regenerated `errors.generated.ts` (do NOT rely on CI to regenerate).
- Brief MUST require the agent to bump `CATALOG_VARIANT_COUNT` by `+N` where `N` = number of variants added (or `-N` for removed).

**Agent side (pre-flight before push):**
- After each new variant: grep all four surfaces for the variant string. All four must be touched.
- Run `cargo test -p benten-errors --test stable_shape` locally — must pass.
- Run codegen + `git status` shows `errors.generated.ts` modified — commit it in the SAME commit as the variant.
- HARD RULE rule-12 disposition for unfinished surfaces: NEVER "next PR" / "follow-up" — all four MUST be in this PR.

**Mini-reviewer side (mandatory check):**
- Mini-review brief includes a standing rubric line: "If this PR touches `ErrorCode` enum, verify all 4 surfaces moved atomically. Missing any = BLOCKER."
- Spot-check: grep each variant in all 4 files; assert presence.

**Why this can't be automated by tooling alone:**
- The "Error Catalog Drift Detector (T7)" CI check catches docs ↔ Rust enum ↔ TS types parity but runs AFTER push — it's a backstop, not a pre-flight gate. Pre-flight catches it before CI burn.
- ErrorCode mutations are routine enough across phases that the per-brief enumeration discipline is the right cost.

**Cross-references:**
- §3.5b HARDENED — sibling tracked-tree post-fix doc-coupling sweep (general shape; this is its specialization for the ErrorCode surface).
- §3.6c (pim-8) — mirror-precedent overshoot guard sibling (over-applies a pattern; this is the opposite shape: under-applies a 4-surface obligation).
- T7 CI workflow `.github/workflows/error-catalog-drift.yml` — backstop tooling.

### 3.4 Cross-crate field-cascade exception (added 2026-04-29 wave-8e)

**MANDATORY one-time workspace check when adding a public struct field used as a literal initializer.** SCOPED per-package pre-flight (§3) compiles only the agent's primary crate's lib — it does NOT compile literal-init sites in OTHER crates' tests + benches. Adding a new field to a `pub struct` cascades to every literal-init site `Struct { field: ... }` across the workspace; missing the cascade means CI breaks downstream PRs that pull in the rebase.

**Confirmed instances (Phase 2b):** D-NS-OBS-1 (sandbox_depth on AttributionFrame), D-NS-26 (PR #30 cascade re-occurrence), G8-A (Strategy on ViewDefinition). Pattern recurred ≥3 times in R5 wave-4 alone.

**The exception:** if your dispatch adds a new field to a `pub struct` that is used as a literal initializer ANYWHERE in the workspace, run ONCE before push:

```bash
cargo check --workspace --all-targets
```

Fix any cross-crate cascade sites in the same commit. This is the explicit one-time exception to §3's workspace-cargo prohibition. The single workspace `check` is bounded enough not to blow the agent's RAM budget; workspace `clippy` / `nextest` / `doc` remain forbidden.

**Detection rule:** if your change touches a struct definition AND any of the following are true, the workspace check is REQUIRED:
- The struct is `pub`
- The struct has 1+ public fields used in literal `Struct { ... }` initializers (vs. builder pattern via `Struct::new(...)`)
- The struct is referenced from other crates (`grep -r "Struct {" --include='*.rs'` spans crate boundaries)

For non-additive changes (renaming or removing fields), the same exception applies and the cascade is even riskier — every literal-init site needs updating, not just adding a default.

**Cross-reference:** memory `feedback_cross_crate_field_cascade` carries the full pattern + 3-incident log.

### 3.4b Cross-crate workflow-constraint exception (added 2026-05-03 R6-R5 ratification of pim-6)

**MANDATORY one-time workspace check when adding any code that asserts an architectural / workflow-level constraint spanning crate boundaries.** §3.4 covers struct-field cascade (data-shape change cascading to literal-init sites). §3.4b covers the analogous workflow-level case: a constraint-assertion (CI script asserting "crate A doesn't depend on crate B" / drift-detector spanning multiple `crates/*/src` / public-API surface check across the workspace) added in one crate may regress as soon as another crate's code shifts — and per-crate scoped pre-flight (§3) cannot detect that.

**Confirmed instance (Phase 2b R6-R4):** drift detector for `ErrorCode` variants spans `crates/benten-errors/src/` definition + `crates/*/src/` construction sites — the per-crate scoped check catches the variant addition but not the production-construction-site requirement (caught as r6-r4-pim-6).

**The exception:** if your dispatch adds (or modifies) a constraint-assertion that reads source files across crate boundaries, run ONCE before push the same workspace check as §3.4:

```bash
cargo check --workspace --all-targets
node --import tsx scripts/codegen-errors.ts  # or whichever drift-detector script applies
# Verify the new constraint passes against the full workspace HEAD.
```

**Detection rule:** §3.4b applies if your change touches:
- `scripts/codegen-*.ts` or any drift-detector / API-surface-check tooling
- A CI workflow that asserts cross-crate properties (cargo-public-api, cargo-vet, dep-tree assertions)
- A test that walks multiple `crates/*/src/` paths

**Phase-3-pre-R1 carry (CI infrastructure):** the question of whether drift-detector additions should automatically trigger a workspace-wide regression scan in CI is a Phase-3 CI-infrastructure decision; remains in `phase-3-backlog §7.11` as a residual.

**Cross-reference:** `.addl/phase-2b/r6-r4-pattern-induction-meta-sweep.json` — `r6-r4-pim-6` origin finding.

### 3.6 Consumer-audit dimension (added 2026-04-29 R6 phase-close ratification)

**MANDATORY for every implementer brief that changes a public-shape struct or signature.** Wave-8 accumulated **8 confirmed instances** of "metadata correctly assembled at producer; production API surface doesn't read it" — 4 caught by sync mini-review during the wave (8b SANDBOX, 8c-subscribe-infra, 8i-wait fp1, 8i-wait fp2), a 5th caught by R6 metadata-producer-vs-consumer lens (`r6-mpc-1` wave-8i fp2 incompleteness), and 3 more (Instances 6/8/10) caught by the deep retrospective sweep. The pattern is structural enough that a standing brief-template addition is needed.

**The pattern:** a metadata field is added to a producing surface (suspension store entry, broadcast channel event, attribution frame, side-table state, struct return type, error variant carrying structured fields). The producing site is correctly written. ONE consumer (typically eval-side) IS wired and read by tests. OTHER consumers (engine API wrapper, napi binding, TS surface, error mapping) are NOT wired — but the agent's tests don't catch it because they only exercise the eval-side consumer.

**The 4 canonical consumer surfaces:**
1. **eval-side** (`crates/benten-eval/src/...`) — usually correctly wired (closest to producer)
2. **engine API wrapper** (`crates/benten-engine/src/engine_*.rs`) — frequently drifts; the public Rust surface
3. **napi binding** (`bindings/napi/src/...`) — frequently drifts; the cross-language boundary
4. **TS surface** (`packages/engine/src/...`) — frequently drifts; the user-facing JS API
5. (also count: error mapping at `bindings/napi/src/error.rs::engine_err` + `packages/engine/src/errors.ts::mapNativeError`, which is the same drift pattern applied to typed-error context-field surfacing)

**Brief-template addition** — every implementer brief that touches a public-shape struct or signature MUST require the agent to include a consumer-audit table in the post-implementation report:

```markdown
## Consumer-audit table

For each public-shape change in this PR:

### <Struct or method name>
| Consumer surface | Wired? | Notes |
|---|---|---|
| eval-side | Y | <file:line where it reads the field> |
| engine API | Y/N | <file:line OR explicit "N/A — this layer doesn't surface this field"> |
| napi binding | Y/N/NA | <as above> |
| TS surface | Y/N/NA | <as above> |
| Error mapping (if struct is in EngineError variant) | Y/N/NA | <`engine_err` + `mapNativeError` round-trip status> |
```

**For each "N" or "NA":** justify explicitly. If "N" with no justification, the agent flagged a potential drift without resolving it — orchestrator triages as either (a) wire-it-now in this PR, (b) BELONGS-ELSEWHERE-NAMED-NOW to a specific Phase-3 entry that EXISTS, or (c) DISAGREE-WITH-EXPLANATION.

**Implementer self-check:** before pushing, the implementer runs through every public-shape change in the diff and asks "did I check all 4-5 surfaces?" If a consumer was MISSED (not deliberately N/A), the implementer surfaces the gap to orchestrator before push rather than landing a half-fix that recurs the wave-8 pattern.

**R6 standing-rule check:** any future R6 phase-close council MUST include a metadata-producer-vs-consumer lens (and ideally a deep retrospective sweep at the BLOCKER-finding-rate threshold of 5+). The wave-8 retrospective sweep found 7 instances on top of the time-bounded R6 lens's 5 — convergence is approached but not achieved by lens-bounded review.

**Cross-references:**
- memory `feedback_synchronous_mini_review.md` § "Wave-8 added a SHAPE-3 sub-pattern" (originating pattern doc + 4-instance log)
- `.addl/phase-2b/r6-metadata-producer-vs-consumer.json` (R6 lens dispatch)
- `.addl/phase-2b/r6-round-2-deep-producer-consumer-sweep.md` (deep retrospective; ratifies the standing rule)
- `.addl/phase-2b/r6-round-1-triage.md` § Pattern shape clarification

### 3.6d Reviewer translation-layer cite-discipline (added 2026-05-03 R6-R5 ratification of pim-11)

**MANDATORY for read-only reviewers (R1 lenses, R6 council, deep-sweeps, mini-reviewers, narrow-iter agents) when issuing a NO-DRIFT verdict that depends on an assumed translation layer.** R6-R5 producer/consumer-deep-sweep surfaced the 23rd p/c drift instance (`r6-r5-pcds-2`: WAIT duration translation runtime correctness break) by walking the full producer→consumer chain. Both the prior R6-R4 deep-sweep AND R6-R4 narrow-iteration BOTH classified `WaitArgs` as "NO-DRIFT — verified" by ASSUMING a napi translation layer existed without grepping for it. The translation layer never existed; DSL-built duration-WAIT silently misrouted in production runtime for the entire window between PR #74 and PR #76.

**The rule:** when a reviewer's verdict on producer/consumer drift hinges on the existence of a translation layer (napi auto-translation, serde rename, `#[serde(rename = "...")]`, custom Visitor, JSON property mapping, etc.):

1. **Cite the file:line where translation occurs.** "Translation layer assumed at the napi boundary" is NOT a verification. "Translation occurs at `bindings/napi/src/node.rs:80::json_to_props`" IS.
2. **Verify the cited construct actually performs the translation** — a function named `json_to_props` could be a verbatim conversion (no key rename) or a translating one. Read the body, not the name.
3. **If you cannot locate the translation, the verdict is "translation-layer assumed" not "verified" — escalate to fix-now drift instance.** The reviewer-side analog of pim-8's mirror-precedent overshoot: assuming a precedent / structural support exists when it doesn't.

**Brief-template addition for read-only reviewers:** verdicts MUST distinguish "translation-layer-CITED-AND-VERIFIED" from "translation-layer-ASSUMED" (the latter is not a clean verdict; surface as a flag).

**Composes with §3.6 + §3.6c:** §3.6 catches drift across surfaces from the implementer side; §3.6c catches the overshoot when a fix mirrors a precedent's shape; §3.6d catches the reviewer-side analog when a verdict assumes a translation layer that was never verified.

**Cross-references:**
- `.addl/phase-2b/r6-r5-producer-consumer-deep-sweep.json` — `r6-r5-pcds-2` origin finding (23rd p/c drift instance)
- `.addl/phase-2b/r6-r5-narrow-pim-meta.json` — pim-11 ratification recommendation

### 3.6c Mirror-precedent overshoot guard (added 2026-05-03 R6-R5 ratification of pim-8)

**MANDATORY when a fix-pass intends to "follow precedent X" or "mirror the shape of sibling primitive Y."** R6-R5 pim-8 named the failure mode after PR #74's r6-r4-cr-1 fix mirrored the EMIT precedent at SubgraphBuilder.subscribe + introduced the 21st producer/consumer drift instance in the same PR that closed the 19th. The agent reasoned "follow EMIT shape" without §3.6 consumer-auditing every field of the SubscribeArgs interface they were ACTIVATING writes for — the result wrote a `handler` arm that the eval-side never reads.

**The rule:** when a brief instructs "mirror precedent X" or the agent independently chooses to copy a sibling primitive's shape:
1. Identify the precedent surface concretely (file:symbol of the producer + file:symbol of every consumer).
2. Run §3.6 consumer-audit on the FULL precedent shape — confirm precedent's consumers are all wired BEFORE assuming the precedent is sound to mirror.
3. For the new-arm shape, run §3.6 consumer-audit on EVERY field of the interface being activated for writes, NOT just the field the brief named.
4. If the precedent itself has unwired consumers (precedent = phantom-precedent), surface to orchestrator BEFORE mirroring — the right shape may be to fix the precedent instead.

**Composes with §3.6:** §3.6 catches drift across surfaces for any field change. §3.6c specifically catches the over-mirror failure mode where an agent activates writes to fields outside the brief scope on the assumption that "the precedent works the same way."

**Cross-references:**
- `.addl/phase-2b/r6-r5-pattern-induction-meta-sweep.json` — `r6-r5-pim-8` origin finding (Mirror-precedent overshoot)
- `.addl/phase-2b/r6-r4-narrow-iteration-producer-consumer.json` — Instance 21 (SubscribeArgs.handler) the recurrence that surfaced pim-8

### 3.6b End-to-end load-bearing test pin requirement for closed-claim PRs (added 2026-05-02 R6 R3 ratification of pim-2)

**MANDATORY for every implementer brief that closes a finding by claiming a runtime arm is now wired.** R6 Round 3's pattern-induction meta-sweep found 7 recurrences across Phase 2b of "fix-pass adds the runtime arm + a sentinel-presence test that pins the wire is *constructed*, but no test actually drives end-to-end through the new arm." All 3 R6 BLOCKERs were instances of this pattern: `r6-mpc-1` (`Engine::resume_with_meta` 3 metadata branches), Instance 6 (multi-label SUBSCRIBE delivery), NEW-1 (`Engine::read_view*`). Each defect hid for ~3 weeks because the sentinel test passed while the production path was inert or wrong. R6-R3 found two more instances of the same shape: `r6-r3-arch-1` (PR #68 `put_node` defended; `delete_node` undefended — sentinel test only exercised put-direction) and `r6-r3-ivm-1` (PR #61 closed TS DSL fail-loud but Rust + napi + integration test still on silent-accept; the closing test was sentinel-presence not end-to-end).

**The pin requirement:** when a fix-pass claims to wire a runtime arm, the PR MUST include at least one test that:

1. Drives the production-grade entry point (e.g. `engine.call(...)`, `engine.onChange(...)`, the napi binding's user-facing method) — NOT a `testing_*` helper that bypasses the arm under test.
2. Asserts an observable behavioral consequence of the arm firing — NOT just that a symbol is present, a method is callable, or a sentinel object is constructed.
3. Would FAIL if the arm were silently no-op'd back to its pre-fix behavior.

**Sentinel-presence tests (e.g. `assert!(method_exists)`, `expect(typed.constructor.name).toBe("X")`) are still useful** as scaffolding pins, but they do NOT satisfy this requirement on their own. Every closed-claim PR needs at least one load-bearing end-to-end test on top of any sentinel pins.

**Brief-template addition:** every implementer brief that closes a runtime-arm finding MUST include "End-to-end test pin per §3.6b: <test file:line that exercises production entry point + asserts observable behavioral consequence>." If the agent reports the test exists but is `#[ignore]`'d / `.skip`'d behind a Phase-3-deferred helper, the brief MUST surface that to orchestrator before push (HARD-RULE-style disposition: fix-now wire the helper, OR BELONGS-ELSEWHERE-NAMED-NOW with the destination identified, OR DISAGREE-WITH-EXPLANATION; "skip the e2e test" is not valid).

**Composes with §3.6 + §3.7:** consumer-audit catches drift across surfaces; 3+-recurrence catches patterns at the orchestrator triage level; end-to-end-pin catches half-fixes at the implementer-PR level. All three run.

**Cross-references:**
- memory `feedback_end_to_end_test_pin_for_closed_claims` — full rationale + 7-recurrence log
- `.addl/phase-2b/r6-r3-pattern-induction-meta-sweep.json` — pim-2 origin finding
- `.addl/phase-2b/r6-r3-architect-reviewer-redux.json` — `r6-r3-arch-1` instance
- `.addl/phase-2b/r6-r3-ivm-correctness-redux.json` — `r6-r3-ivm-1` instance

**§3.6b amendment 2026-05-05 — TS-canary defensive-throw extension (pim-2-ts-canary; ratified at Phase-3 R4-R1).** The end-to-end pin requirement applies to TypeScript RED-PHASE pins as well. Rust uses `unimplemented!()` / `panic!` to make the failure mode loud when an `#[ignore]`'d test is un-ignored before the implementer wires the runtime arm. TypeScript's `it.skip(...)` body has no analog enforcement: a skipped test with an empty body silently passes when un-skipped. The TS analog is a defensive `throw new Error("RED-PHASE: <wave/finding/implementer-step>")` body that fires loudly when the un-skip happens before the wire. R3-E PR #94's `tdd-major-1` finding closure added 27 such throw-bodies across 6 R3-E TS files (`dsl_args_drift` 7 + `edge_interface` 2 + `errors` 3 + `onChange_onEmit` 3 + `stream_leak` 5 + `atrium_examples` 5) — adopted as the canonical TS-canary shape for all Phase-3 + later TS RED-PHASE pins.

**TS-canary template:**

```ts
it.skip("RED-PHASE: <wave-N> — <observable consequence>", async () => {
  // Implementer wires <runtime arm> at <wave>; observable consequence:
  // <what the assertion would check once the arm is wired>.
  throw new Error(
    "RED-PHASE: <wave-N> authors <runtime arm> + drops .skip + un-comments assertions",
  );
});
```

**Why both sides need this:** end-to-end pins (the §3.6b primary requirement) drive observable consequences through production entry points; TS-canary throws are the secondary enforcement that makes the un-skip-before-wire failure mode loud (the same shape pim-2 catches in Rust via `unimplemented!()`). Both matter; neither substitutes for the other.

**Cross-references for ts-canary extension:**
- `.addl/phase-3/r4-r1-tdd-redphase.json` — origin finding `tdd-major-1` (defensive throw-body discipline).
- PR #94 (R4-FP R3-E partition fix-pass) — 27 throw-bodies landed as canonical precedent.

**§3.6b sub-rule 4 — Per-finding granularity for closure pins AND deferral destinations (added 2026-05-09 Phase-3 R6 R1 ratification of pim-2-amendment).** The §3.6b primary requirement (end-to-end pin) and the HARD RULE clause-(b) named-destination requirement BOTH operate at PER-FINDING granularity, not umbrella-section granularity. R6 R1 pattern-induction met the 3+-recurrence threshold at HEAD `0b8d6c5` with three concrete instances:

- **G21-T2 MAJOR-7** — registry-clear-on-leave fix landed without a per-finding pin exercising the specific `leave → rejoin → callbacks-cleared` behavioral surface. The closure cited the umbrella subscribe test rather than the specific runtime arm under the finding.
- **NS-T52 update 2 (audit-6-1)** — closure pin cited the wrong surface (a sibling test that exercised an adjacent code path, not the one the finding named).
- **audit-6-3 umbrella-section phantom-destination** — deferral cited "phase-3-backlog §3.1" (the umbrella section) rather than a specific row receiving the entry. The umbrella section existed; the row didn't.

**The amendment, in two clauses:**

1. **Closure-pin granularity (extends §3.6b primary requirement).** When closing a finding by claiming a runtime arm is wired, the cited end-to-end pin MUST exercise the SPECIFIC behavioral surface named in THAT finding — not a sibling surface, not the umbrella feature, not "the test file as a whole." If finding F closes by wiring arm A, the pin MUST drive arm A directly with assertion content tied to A's observable consequence. A sibling-surface test does not satisfy the requirement even if both surfaces were broken by the same root cause; close finding-A and finding-B both with their own pins.
2. **Deferral-destination granularity (extends HARD RULE clause-(b)).** When deferring a finding via BELONGS-NAMED-NOW, the destination MUST be a specific row / specific list-item / specific bullet in the destination doc — not the umbrella section. Cite by §-number AND row label (e.g. "phase-3-backlog §3.1.X — on-the-wire emission of audit-6-3" rather than "phase-3-backlog §3.1"). The orchestrator MUST verify the row exists at the cited address before dispatching the closing commit; if the row doesn't exist, the orchestrator authors it NOW (per HARD RULE clause-(b) "destination receives entry NOW").

**Why per-finding granularity matters:** umbrella citations make verification ambiguous. A reviewer reading "tested in subscribe.test.ts" for finding F can't cheaply confirm F's specific arm was exercised; the closure-quality drops to sentinel-presence-equivalent. Per-finding granularity restores the load-bearing property: someone reading the closure citation can grep the test body in O(1) and confirm the finding's named surface is exercised.

**Composes with §3.5b HARDENED + §3.6b primary requirement + §3.5e disposition-verification:** §3.5b sweeps adjacent docs; §3.6b primary requires end-to-end pin shape; §3.5e verifies disposition at HEAD; §3.6b sub-rule 4 verifies the cite-target is the SPECIFIC finding's surface, not its neighborhood.

**Cross-references for sub-rule 4:**
- memory `feedback_pim_2_amendment_per_finding_granularity` — full rationale + 3-instance log.
- `.addl/phase-3/r6-r1-pattern-induction.json` — R6 R1 ratification recommendation (pim-2-amendment).
- `.addl/phase-3/r4b-pattern-induction.json` — origin 3+-recurrence finding (G21-T2 MAJOR-7 + NS-T52 update 2 + audit-6-3 umbrella-section).

### 3.6e RED-PHASE staged pin → un-ignore wave-time-pressure skip (added 2026-05-09 Phase-3 R6 R1 ratification of pim-12)

**MANDATORY when a wave is staged with RED-PHASE pins (`#[ignore]`'d Rust tests / `it.skip(...)` TS tests / `#[cfg(unimplemented_*)]` cfg-gated stubs / `unimplemented!()` panicking arms / TS-canary defensive-throw bodies per §3.6b ts-canary extension) and a downstream wave is named as the un-ignore / un-skip / un-cfg target.** R6 R1 pattern-induction met the 3+-recurrence threshold across phases with a 5-instance class:

| Phase | Instance |
|---|---|
| Phase-1 R4 | Vacuous projection (test asserted shape that always holds; never exercised the projection arm) |
| Phase-1 R6→R7 | Catalogued-but-unfired error codes (variants exposed; no production path raised them) |
| Phase-2a G1-A | False-cargo-green (`#[ignore]` body covered the test's only assertion path) |
| Phase-2b R4b | Structural-vs-runtime gap (test pinned wire-shape; no runtime arm drove the wire) |
| Phase-3 R4b | RED-PHASE pins not un-ignored at named target wave (3-lens corroboration: cryptography r4b-major-1 + capability-system r4b-cap-1 + wasmtime r4b-wsa-1/2) |

**The recurring failure mode:** wave A stages a RED-PHASE pin citing wave B as the un-ignore target. Wave B ships under time pressure; the implementer focuses on the wave-B scope and never sweeps wave-A's RED-PHASE pins. The pin survives un-touched into the phase-close lens set, where a reviewer catches it ("test exists but is `#[ignore]`'d behind helper that DID land at wave B") — except by the time a reviewer catches it, the wave-B claim of "scope complete" has already shipped through mini-review and merged.

**The discipline, in two parts:**

1. **Wave-completion checklist MUST include un-ignore audit.** Before any wave-completion mini-review, the orchestrator (or implementer brief checklist) runs an explicit grep across the workspace for RED-PHASE pins that cite THIS wave as the un-ignore target:
   ```bash
   # Rust: ignored tests citing the wave
   grep -rn "#\[ignore\b" crates/ packages/ tests/ --include='*.rs' | grep -i "wave-<N>\|<wave-name>"
   # TypeScript: skipped tests citing the wave
   grep -rn "it\.skip\|describe\.skip" packages/ --include='*.ts' --include='*.test.ts' | grep -i "wave-<N>\|<wave-name>"
   # cfg-gated stubs citing the wave
   grep -rn "#\[cfg(unimplemented_" crates/ --include='*.rs' | grep -i "wave-<N>\|<wave-name>"
   # TS-canary throw bodies citing the wave (per §3.6b ts-canary extension)
   grep -rn "RED-PHASE: <wave-N>\|RED-PHASE: wave-<N>" packages/ --include='*.ts' --include='*.test.ts'
   ```
   Each match MUST be EITHER (a) un-ignored / un-skipped / un-cfg'd with the production arm wired in this wave, OR (b) explicitly re-targeted to a NEW named destination wave with HARD RULE clause-(b) destination-receives-entry-NOW landing in the phase-3-backlog (or active phase plan), OR (c) DISAGREE-WITH-EXPLANATION articulated in the wave-completion mini-review JSON. "Wave time pressure" is NEVER a valid disposition.

2. **Reviewer briefs MUST verify landing-status + production-arm-presence, not just spec-pin presence.** When a reviewer's lens covers a surface with RED-PHASE pins citing a previously-merged wave: the reviewer MUST grep for the RED-PHASE pins citing that wave's number, AND verify each has been un-ignored / un-skipped / un-cfg'd at the cited target wave's HEAD. A pin that survives past its cited target wave with an inert body is a finding (severity per impact: BLOCKER if the cited wave's claim was "runtime arm wired" + production-path is dormant; MAJOR otherwise).

**Composes with §3.6b sub-rule 4:** the per-finding closure-pin granularity rule says the pin must exercise the specific behavioral surface; §3.6e says the pin must EXIST as un-ignored / un-skipped / un-cfg'd at the wave that closes the finding. Both run together — pim-2 closure-pins target SPECIFIC arms; pim-12 closure-pins must be ACTIVE arms.

**Cross-references:**
- memory `feedback_pim_12_red_phase_staged_pin_un_ignore_discipline` — full 5-instance log + recovery protocol.
- `.addl/phase-3/r4b-cryptography.json` — r4b-major-1 (RED-PHASE pin not un-ignored).
- `.addl/phase-3/r4b-capability-system.json` — r4b-cap-1 (sibling instance).
- `.addl/phase-3/r4b-wasmtime-sandbox.json` — r4b-wsa-1 + r4b-wsa-2 (sibling instances).
- `.addl/phase-3/r6-r1-pattern-induction.json` — R6 R1 ratification recommendation.

### 3.6f SHAPE-not-SUBSTANCE pre-flight (added 2026-05-09 Phase-3 R6 R1 ratification of pim-18)

**MANDATORY for every implementer brief that adds, exposes, or wires a public symbol claimed to satisfy a finding's runtime-arm contract.** R6 R1 pattern-induction confirmed the 10-wave datapoint trend per r4b-pattern-induction: 3 instances caught POST-HOC at G16-A wave-5b (pre-mandate); 8 instances caught PRE-PUSH at G16-C (first mandated wave); 8 consecutive zero-incident waves followed (G16-D / G16-B / G19-B / G19-D / G19-C1 / G19-C2 / G19-C1-fp / G19-E). This is the strongest single positive-trend datapoint in the Phase-3 R5 record. Phase-1 R7's "aspirational prose but dead code" anti-pattern is the lineage; pim-18 is its codified pre-flight defense.

**The distinction the rule enforces:**

- **SHAPE check (insufficient on its own):** symbol exists, is exported, signature compiles, callable at the public surface, has a docstring, has a sentinel-presence test, appears in `pub use` re-exports, mentioned in the changelog. SHAPE asks: "is the symbol *there*?"
- **SUBSTANCE check (load-bearing):** the symbol's body substantively realizes the contract the finding named. SUBSTANCE asks: "does calling the symbol at a real consumer site cause the observable behavior the finding requires?"

Implementer reports of "test exists" or "API is wired" or "the method is callable" are SHAPE assertions. They DO NOT satisfy the runtime-arm-wired closure claim on their own. The implementer brief MUST surface SUBSTANCE-level verification.

**The pre-flight, in three steps:**

1. **Production call site enumeration.** Before push, the implementer enumerates EVERY production call site for the new/changed symbol — `grep -rn "<symbol>\b" crates/ packages/ bindings/ --include='*.rs' --include='*.ts'`, filtered for non-test files. For each call site, confirm the call substantively exercises the finding's contract (not "the symbol is referenced in a comment" / "the symbol appears in a `pub use` line" / "the symbol is in a test fixture").
2. **Body-of-test substantive check.** For each test pin cited in the closure, read the test body end-to-end and confirm: (a) the production entry point is invoked (per §3.6b primary requirement); (b) the assertion checks an observable consequence tied to the finding's contract (per §3.6b sub-rule 4); (c) commenting out the new code in the production path would FAIL the test. SHAPE-only tests that pass even when the new code is no-op'd do not count.
3. **Aspirational-prose gap check.** Grep the changelog / NS-T entry / commit message / PR body for verbs that imply runtime behavior ("wires", "implements", "enforces", "validates", "rejects", "emits", "delivers") — for each such claim, the corresponding production arm MUST exist + the test MUST exercise it. If the prose claims behavior the code doesn't substantively realize, fix the code OR weaken the prose to match what's actually there.

**Why this matters across phases:** Phase-1 R7 caught "aspirational prose but dead code" repeatedly across the Phase-1 corpus; the discipline merged into R6 verify-don't-trust-docs but lapsed at the per-implementer pre-flight level. Phase-3 R5's G16-A wave-5b had 3 SHAPE-not-SUBSTANCE instances caught only post-hoc; G16-C onward landed clean. The pre-flight is the difference; codifying it locks in the gain.

**Composes with §3.5b HARDENED + §3.6 + §3.6b + §3.6b sub-rule 4 + §3.6e:** §3.5b sweeps adjacent docs; §3.6 audits consumer drift; §3.6b primary requires end-to-end pin shape; §3.6b sub-rule 4 requires per-finding cite granularity; §3.6e ensures un-ignore happens at the named target wave; §3.6f confirms the symbol's BODY substantively realizes the contract beyond the SHAPE-level checks the other rules already enforce. All six run together; they cover the public-API-surface drift triangle from independent angles.

**Cross-references:**
- memory `feedback_pim_18_shape_not_substance_pre_flight` — full rationale + 11-wave datapoint log.
- `.addl/phase-3/r4b-pattern-induction.json` — 10-wave-trend origin.
- `.addl/phase-3/r6-r1-pattern-induction.json` — R6 R1 ratification recommendation.

#### 3.6f sub-rule extension — regression-guard substantive-arm contract (added 2026-05-25 R6-R2-FP-D ratification; 16-instance recurrence at L2+L9+L13)

**MANDATORY for every regression-guard test minted to pin a wave's CLOSED findings against future revert.** The 6 F4 §S1/§S2/§S3a/§S3b/§S3c/§S4 regression-guards (L2-R6-R2-MAJOR-2) + L9-r6r2-MINOR-3 self-equality tautology + L13-MAJ-1 18-of-34 trait-isolated F4 tests + L13-MIN-4 const-tautology workspace-walkers + L13-MIN-5 empty-body forensic-anchor `#[test]` arms = 16-instance recurrence within ONE wave's regression-test family establishes the strengthening threshold.

Every regression-guard test for an FP-cycle MUST:

- **(a) Invoke a production entry point** (the public-API surface that the original bug manifested at). Acceptable production entry points: `Engine::install_plugin(...)` / `Engine::delegate_capability(...)` / `Engine::admit_write_chain(...)` / `Engine::apply_atrium_merge(...)` / `Engine::call_as(...)` / equivalent named API surface. NOT acceptable: direct `policy.<hook>(...)` calls / trait-isolated method calls / mock-engine builder calls without driving through the production seam.
- **(b) Assert an observable consequence**: a database state change (row appears / row absent) / a typed error fire (`Err(ErrorCode::X)`) / an event broadcast (counter on subscriber increments) / a panic / a cap-cascade residue (`library.is_empty()`). NOT acceptable: assertion that a const equals itself / assertion that a method returned `Ok(())` without checking what changed / assertion that a trait method "is callable" without exercising production flow.
- **(c) Demonstrate would-FAIL-on-revert in the commit body** via `git stash` + `cargo nextest run -p <crate> --test <name>` showing FAIL pre-fix. The commit body line MUST cite the production-side mutation that the revert simulates (e.g. "commenting out `policy.check_install_consent(...)` block at `plugin_lifecycle.rs:967-973` → counter=0").
- **(d) NEVER use `assert_eq!(CONST, CONST_VAL)` walker shape.** The const-tautology anti-pattern (L13-MIN-4 origin instances at S1 arm 3 + S3c arm 3) is structurally non-failing — the comparison is between two compile-time-equal sides. Replace with a real `fs::read_dir` workspace-walker that counts production consumer call-sites or scans for the expected source pattern.
- **(e) NEVER ship zero-assertion `#[test]` arms.** The empty-body forensic-anchor anti-pattern (L13-MIN-5 origin instances at S3c arm 5 + S3a arm 6 + S3a `install_consent_hook_fires_before_cap_cascade_documented`) provides no regression-fire signal. Replace with a substantive ordering pin via observable side-effect (e.g. deny at Step N → assert later-step residue absent).

**Recurrence threshold**: this sub-rule extension was ratified after 4+-recurrence WITHIN A SINGLE WAVE (the F4 regression-test family). Future patterns at ≥3-recurrence in similar regression-test-family shapes trigger same-type strengthening codifications.

**Cross-references:**
- memory `feedback_pim_n_regression_guard_substantive_arm` — full rationale + 16-instance origin table.
- §3.6f parent — the SHAPE-not-SUBSTANCE pre-flight this strengthens.
- §3.6b sub-rule 4 — per-finding granularity (companion).
- §3.6h ratification-must-close-origin — R6-R2-FP-D closed all 16 origin instances in the same PR per §3.6h.

### 3.6g Prior-phase pim-N codifications as explicit pre-flight checklist in next-phase R3/R5 briefs (added 2026-05-13 Phase-4-Foundation R6 R1 pim-N meta-sweep Candidate A ratification)

**MANDATORY for every R3 / R5 implementer brief in a NEW phase after a phase-close ratification round** (or any phase that inherits pim-N rules from a predecessor): prior-phase pim-N codifications MUST be reproduced as EXPLICIT pre-flight checklist lines in the brief body — NOT as memory-references (e.g. "see memory `feedback_pim_X`") or as `.addl/dispatch-conventions.md §3.X` cross-references alone.

**5-instance recurrence (Phase-4-Foundation R3-R5):** Phase-2b ratified pim-12 / §3.5g / pim-18 / pim-2-amendment at phase-close. Phase-4-Foundation R3-R5 saw **first-phase recurrence of EACH at original-identification density**:

1. **G24-D BLOCKER** — pim-12 §3.6e RED-PHASE staged-pin un-ignore discipline missed (RED-PHASE-BODY novel pin status invention).
2. **G23-B BLOCKER** — pim-12 §3.6e missed (staged-pin un-ignore wave-completion sweep).
3. **G24-A MAJOR** — pim-12 §3.6e missed (sister phantom-destination clusters).
4. **G23-A MAJOR** — pim-12 §3.6e missed (G23-A wave-4b phantom citations left after the BLOCKER fix-pass).
5. **G24-B MAJOR** — pim-18 §3.6f SHAPE-not-SUBSTANCE missed (degenerate same-struct double-hash test).

R4 triage §6 explicitly named this pattern. **Memory-reference alone does not transfer across phases at the implementer-brief level** — the agent reads the brief end-to-end + executes; they do NOT cross-reference `.addl/dispatch-conventions.md` mid-task. The codification surface that DOES carry across phases is the BRIEF itself.

**Brief-template addition:** every R3 / R5 brief at the start of a new phase MUST include a section like:

```
## Inherited pim-N pre-flight checklist (from prior phases)

Before push, verify per-finding:
- [ ] pim-1 §3.5b HARDENED — post-fix doc-coupling sweep applied
- [ ] pim-2 §3.6b — every closure pin carries PRODUCTION-ARM + OBSERVABLE-CONSEQUENCE + WOULD-FAIL-IF-NO-OP'd
- [ ] pim-12 §3.6e — every RED-PHASE staged-pin citing THIS wave is un-ignored substantively (verify by grep across crates/*/tests/)
- [ ] pim-18 §3.6f — test bodies drive real production code paths (NOT stubs, NOT mocks, NOT assert!(true))
- [ ] §3.5g — any new ErrorCode minted atomic Rust + TS + ERROR-CATALOG.md + ALL_CATALOG_VARIANTS
- [ ] §3.5h MANDATORY 5-check before push: fmt + scoped check + scoped clippy -D warnings + scoped doc + cite-drift
```

Plus any phase-specific additions (e.g. for Phase-4-Foundation: 4-identity-concepts separation per CLAUDE.md #18; for Phase 5+: TBD).

**Why this works:** the agent reads the brief sequentially before starting work; a top-of-brief checklist is unambiguous + actionable + executable. Memory-references work for orchestrator-side reasoning but not for implementer execution. This composes with §3.5b/c/g/h/i + §3.6b/c/d/e/f as the workspace's "pim-N catalog at implementer-touch-point."

**Cross-references:**
- memory `feedback_pim_n_prior_phase_explicit_preflight` — full rationale + 5-instance recurrence catalog.
- `.addl/phase-4-foundation/r6-r1-pim-n-meta-sweep.json` — Candidate A origin (highest-impact meta-pattern).
- `.addl/phase-4-foundation/r4-triage.md` §6 — R4 triage finding that first surfaced the cross-phase memory-reference inadequacy.

### 3.6h Rule-ratification-against-drift mandatory-close clause (added 2026-05-13 Phase-4-Foundation R6 R3 ratification)

**MANDATORY when a new pim-N or §-codification names specific drift instance(s) as its origin or its empirical evidence:** the same PR/wave that lands the rule MUST close (or DEFER-NAMED-NOW with a real backlog destination per HARD RULE clause-(b)) the origin instance(s). Otherwise the rule's ratification is decoupled from its origin — engineers reading the rule + accepting it as "live" while the origin remains unfixed produces a credibility gap: rules are "live" but enforcement-against-the-origin is silently postponed.

**Recurrence evidence at HEAD `6e10aea` (Phase-4-Foundation R6 R3 ratification):**

| # | Instance | Status at original-ratification |
|---|---|---|
| 1 | §3.5g #3 type-name cross-doc mirror ratified naming TauriRender vs TauriRenderer 8-cite drift as origin | 7 cite sites UNFIXED at original-ratification; closed retroactively at R6-FP-2 |
| 2 | §3.5g #4 cross-tool config mirror ratified naming `cargo-audit ↔ deny.toml` drift as origin | Drift CLOSED inline at ratification (counterexample — partial honor) |
| 3 | §3.5h MANDATORY-PRE-MERGE precedent ratified naming 4 cite-drift fix-pass instances as origin | 2 of 4 closed inline; 2 already-closed before ratification |
| 4 | §3.6g prior-phase pim-N pre-flight checklist ratified naming 5 Phase-4-Foundation R3-R5 cases as origin | Originating waves already-shipped — rule fires forward only; no follow-on sweep |
| 5 | meth-r6-r3-1 verdict-vs-disposition schema drift across 24/32 R6 JSONs (R3 finding) | §4.30 codified rule for legacy artifacts but `§3.6c` brief-template NEVER updated at HEAD — same anti-pattern internal to dispatch-conventions itself |
| 6 | R6-FP-2 closed PLUGIN-MANIFEST.md `private_namespace_policy.rs` phantom but missed sibling docs (`docs/GLOSSARY.md:111` + `docs/history/PHASE-4-FOUNDATION.md:98`) | Sibling-doc phantoms persisted at HEAD until R6-FP-3 |

**Pattern:** when a rule codifies discipline for shape X, the SAME PR landing the rule must demonstrate the rule applied to every known instance of X. "Defer the originating drift to next wave" is the failure mode this rule names.

**How to apply:**

1. **At ratification time:** the orchestrator listing the rule MUST also enumerate ALL known instances of the rule's target shape. For each instance, the SAME PR either (a) closes it inline (preferred), or (b) DEFER-NAMED-NOW with a real `phase-N-backlog.md §X` destination carrying acceptance criteria per HARD RULE clause-(b).
2. **At review time** (mini-reviewer + R6-lens-reviewer): when verifying a pim-N codification's presence, the reviewer ALSO verifies the origin instance(s) are closed-or-named.
3. **Defense surface:** §3.6h composes with HARD RULE rule-12 (clause-(a)/(b)/(c) only) — defer-without-destination is already forbidden; this rule extends to NEW rules' origin instances.

**Sharpening (added 2026-05-13 R6-FP-4 per r6r4-pi-1 pattern-induction MINOR):**
- **"Already-closed-before-ratification" is STRICTLY STRONGER than same-PR-closure** — when the origin drift was closed at an earlier wave + the codification merely names that earlier closure as evidence, the rule is satisfied (origin is closed; ratification documents an existing pattern). Instance 2 (`§3.5g #4`, drift closed at commit `2b96091` before R6 R2 ratification) + instance 3 partial (`§3.5h MANDATORY` precedent, 2 of 4 already-closed before R6 R6-final ratification) demonstrate this stronger form.
- **Forward-fire-only exemption** — when a codification's "origin instances" are wave-class patterns that already-shipped + the rule's role is to prevent FUTURE recurrence (e.g. `§3.6g` prior-phase pim-N pre-flight, where the 5-instance origin recurrence in Phase-4-Foundation R3-R5 is structurally not closable retroactively — those waves shipped without the rule), the same-PR-closure obligation is exempt; rule fires forward-only. The codification SHOULD note this exemption explicitly in its origin table so readers don't expect retroactive closure.

**Composes with:** HARD RULE rule-12 + §3.5b HARDENED (post-shape-change doc-coupling sweep) + §3.6e (RED-PHASE staged-pin un-ignore at the closing wave) + §3.6f (SHAPE-not-SUBSTANCE pre-flight) + §3.6g (prior-phase pim-N checklist).

**Cross-references:**
- memory `feedback_pim_n_ratification_must_close_origin` — full rationale + recurrence catalog.
- `.addl/phase-4-foundation/r6-r2-pim-n-r7-spec-compliance.json` finding `r6r2-new-1` — initial proposal at R6 R2 (deferred per Ben's "1 clear + 2 marginal — weak" judgment).
- `.addl/phase-4-foundation/r6-r3-pattern-induction-meta-sweep.json` — R6 R3 validation of the deferred candidate's recurrence shape.

### 3.6i Review / lens / mini-review JSON schema discipline (added 2026-05-13 Phase-4-Foundation R6 R3 ratification — closes §4.30 destination cite)

**MANDATORY structured top-level shape for all R1 lens JSONs, R6-Rn lens JSONs, R6-FP mini-review JSONs, and any other ADDL-pipeline review artifact:**

```json
{
  "lens": "<lens-name>",            // string, identifies the lens persona
  "round": "R6-R3" | "R6-R2" | ...,  // string, round identifier
  "head": "<git-sha>",              // string, target HEAD verified by agent
  "phase": "4-Foundation",           // string, phase identifier
  "disposition": "...",              // string — REQUIRED top-level disposition (e.g. CONVERGED / APPROVE / APPROVE-WITH-FIXES / BLOCKER-cluster)
  "method": "...",                   // string, methodology summary
  "findings": [                      // REQUIRED array; may be empty
    {
      "id": "<lens-prefix-rN-finding-N>",
      "severity": "BLOCKER" | "MAJOR" | "MINOR" | "OBS",
      "category": "...",
      "evidence": "...",             // include file:line citations
      "disposition": "FIX-NOW" | "BELONGS-NAMED-NOW:phase-N-backlog.md §X" | "DISAGREE-WITH-EXPLANATION:...",
      "fix": "..."                   // when disposition is FIX-NOW
    }
  ],
  "candidate_pim_n": [...],          // optional array of new pim-N candidates
  "summary": "..."                   // optional human-readable wrap-up
}
```

**Key requirements:**
- `disposition` is the canonical top-level field name (NOT `verdict`). Legacy artifacts using `verdict` are swept at pre-tag.
- `findings[]` is REQUIRED (may be empty array).
- Per-finding `disposition` MUST be one of HARD RULE 12's three valid shapes: FIX-NOW / BELONGS-NAMED-NOW with specific destination / DISAGREE-WITH-EXPLANATION.
- Output file MUST validate as well-formed JSON (`jq .` returns 0) — see §3.5h JSON-artifact validation amendment.

**Brief-template mandate:** every reviewer/lens brief (R1, R2, R6-Rn, R4b, etc.) MUST explicitly cite this schema with field names + reference §3.6i for the canonical spec. Briefs that say "output structured JSON" without naming the field schema produce drift; the schema must be in the brief body.

**Why codified now:** R6 R3 methodology-critic `meth-r6-r3-1` (MAJOR) found that 24 of 32 R6 lens reports used `verdict` only; 5 used `disposition`; 3 lacked both. §4.30 (Phase-4-Foundation pre-tag backlog row) named the legacy-artifact sweep AND named `§3.6c brief-template` as the live-discipline destination — but §3.6c is "Mirror-precedent overshoot guard" (unrelated). The brief-template guidance never landed; the schema drift compounded across 14 mini-reviews + 17 R6-R3 JSONs. Folded under §3.6i + §4.30 retargeted to cite §3.6i; legacy artifacts swept inline at R6-FP-3.

**Composes with:** §3.5h MANDATORY-PRE-MERGE 5-check + §3.5h JSON-artifact validation amendment + HARD RULE 12.

**Cross-references:**
- `docs/future/phase-4-backlog.md §4.30` — original Phase-4-Foundation pre-tag named-destination (now CLOSED at R6-FP-3).
- `.addl/phase-4-foundation/r6-r3-methodology-critic.json` finding `meth-r6-r3-1` — origin instance.
- `.addl/phase-4-foundation/r6-r3-pattern-induction-meta-sweep.json` finding `r6r3-meta-4` — independent corroboration.

### 3.6j Sweep-completeness self-verify discipline (added 2026-05-14 Phase-4-Foundation R6 R7 ratification — promotes §4.57 watch-list candidate after 4th-instance trigger fired)

**MANDATORY for orchestrator + implementer agents when claiming a sweep is COMPLETE** in commit body, PR description, mini-review output, or status update: BEFORE writing the claim, run the actual validation tool that defines the completeness criterion against the round's own output scope (not just the prior-state baseline that motivated the sweep).

**The rule:** "I swept X" claims require running the sweep's validator-tool over (a) the prior-state baseline AND (b) the wave's own newly-produced artifacts, then asserting zero residuals on BOTH before the claim is written. Sweep tooling that grepped for a literal pattern (e.g. `"verdict":`) MUST run against the semantic criterion (e.g. top-level `disposition` field present per §3.6i schema), not just the literal pattern.

**Brief-template mandate.** Reviewer/lens/mini-reviewer briefs (R1, R2, R6-Rn, R4b, mini-review, etc.) that produce JSON artifacts MUST instruct agents to author their output with canonical top-level `disposition` field at author-time (per §3.6i schema) — eliminates the orchestrator-catchup cycle where a downstream sweep must add the field post-hoc. The brief template clause: *"Your output JSON MUST carry top-level `disposition: \"<value>\"` as the first or second field after `lens`. Acceptable values: CONVERGED / APPROVE / APPROVE-FOR-TAG / APPROVE-WITH-FIXES / BLOCKER-cluster / APPROVE-WITH-N-MAJOR-AND-... etc. — but the FIELD MUST be top-level. Do NOT use `verdict`."*

**Why codified now:** 4-instance recurrence across the Phase-4-Foundation R6 cycle:
1. R6-FP-3 §3.6i verdict→disposition sweep claimed 32-file complete but R6-FP-4 found 2 R4 residuals.
2. R6-FP-4 doc-cite 11-site sweep claimed complete but R6 R5 found 3 residuals (sdr-r6-r4-1).
3. R6-FP-5 §3.6i 49-JSON sweep claimed complete but R6 R6 found 4 JSONs lacking top-level disposition (meth-r6-r6-1 + r6r6-pi-1 cross-lens confirmed).
4. R6-FP-6 commit body claimed "79/79 R6 JSONs §3.6i conformant" but R6 R7 found the same 4 R6 R6 lens JSONs still lacking top-level `disposition` (r6r7-r7-1 + r6r7-meth-1 + doc-r6-r7-1 + arch-r6-r7-1 + r6r7-pi-1 = 5-lens cross-confirmation).

Threshold semantics: §3.7's 3+-recurrence rule was MET at instance 3 (DEFER → watch-list per §4.57); 4th instance closes the debate. The pattern-induction lens at R6 R7 fired its own promotion criterion explicitly.

**Pattern shape.** Sweep tooling defined completeness against the PAST state (the legacy artifacts that motivated the sweep) but not against the round's own outputs. Each instance is the orchestrator's own §3.6h failure mode applied to its own scope — the rule fires AT the sweep producer.

**Composes with:** §3.5h MANDATORY-PRE-MERGE 5-check (JSON-artifact validation `jq .`) + §3.6h Rule-ratification-against-drift mandatory-close (sibling family — §3.6h is "rule names origin"; §3.6j is "claim names scope") + §3.6i JSON schema discipline (the most-common substrate for §3.6j violations) + HARD RULE 12 BELONGS-NAMED-NOW (no implicit deferral of residuals).

**Cross-references:**
- `docs/future/phase-4-backlog.md §4.57` — original watch-list entry (CLOSED at R6-FP-7 ratification).
- `.addl/phase-4-foundation/r6-r7-pattern-induction-meta-sweep.json` finding `r6r7-pi-1` — promotion-trigger instance.
- `.addl/phase-4-foundation/r6-r7-r7-spec-compliance.json` finding `r6r7-r7-1` — 4th-instance cross-confirmation.
- `.addl/phase-4-foundation/r6-r7-methodology-critic.json` finding `r6r7-meth-1` — 4th-instance independent confirmation.

#### 3.6j sub-rule extension — cite-grep-verify at author-time (added 2026-05-25 R6-R2-FP-C ratification; ~40-instance recurrence at L11+L14+L16+L17+L18)

**MANDATORY for every author of any markdown / dispatch-conventions / INTERNALS / spec / backlog edit that adds or modifies a cite.** The existing §3.6j sweep-completeness self-verify discipline names the post-sweep tool-output validation step. Author-time CITE verification was an implicit corollary that was repeatedly skipped, producing the L11 (7 phantom byte-pin globs) + L14 (3 phantom file paths + 8 line-cites) + L16 (16 phantom workspace-wide globs + 4 INTERNALS body) + L17 (3 line-cites) + L18 (6 PR-cites — though §3.5n verify demoted 5 of 6) cluster = ~40 instance recurrence. Codification makes the discipline load-bearing at the author surface.

Every `.md` cite to a `.rs` / `.ts` / `.tsx` / `.toml` / `.wat` / `.json` / `.yml` source file, every `path::symbol` cite, every `#NNNN` PR-cite, and every `path/to/glob_*.rs` test-file glob MUST be verified by the AUTHOR with the relevant tool **before commit**:

- **File-path cites**: `ls <path>` / `fs.existsSync(<path>)` / `git ls-files | grep <path>`.
- **Line-number cites**: read the cited file at the cited line + match against the cite's narrative. If the cited surface is on the §3.5b HARDENED point 3 high-churn-surface list (`primitive_host.rs`, `engine_views.rs`, `evaluator.rs`, `lib.rs`, `builder.rs`, `wait.rs`, `subscribe.rs`, `mermaid.ts`, `dsl.ts`), MUST be promoted to `path::symbol` form (line numbers drift on every refactor).
- **Symbol cites**: `grep -nE "fn <symbol>|struct <symbol>|enum <symbol>|const <symbol>" <cited-file>`.
- **PR cites**: `gh pr view <N> --json mergedAt,state,baseRefName`; if PR is `CLOSED` with `mergedAt: null`, it's phantom — retract or move to "considered but not merged" narrative.
- **Glob cites**: expand the glob with `python -c "import glob; print(glob.glob('<pattern>'))"`; if zero files match, rename to the actual file OR (if forward-looking) annotate `<!-- cite-drift-exempt: <reason> -->` adjacent to the cite.

**Sibling-diff-walk sub-rule.** When adding a file-path cite, scan adjacent cites in the same section for cross-cite drift — adjacent cites often share the same shape and were edited in the same batch by the same author at the same time, so post-fix-pass drift tends to cluster.

**Enforcement.** The `tools/cite-drift-detector/` (extended at R6-R2-FP-C to cover the cite classes above with new `LineCiteGlobNoMatch` + `PrCiteClosedNotMerged` + `PrCiteNotFound` finding kinds + `--json` canonical-schema output) is the §3.5h pre-push gate that fails CI on broken cites. Self-test under `tools/cite-drift-detector/tests/glob_cite_phantom_detection_self_test.rs` verifies the scanner detects known phantoms (per §3.6j sweep-completeness — the validator validates itself).

**Cross-references:**
- memory `feedback_pim_n_cite_grep_verify_at_author_time` — full rationale + ~40-instance origin enumeration.
- §3.6j parent — sweep-completeness self-verify (this is its author-time corollary).
- §3.5b HARDENED — post-fix doc-coupling pre-flight (companion: §3.5b sweeps adjacent docs; §3.6j-ext sub-rule (i) verifies cite-pointing).
- §3.5n orchestrator-ground-truth-verify — companion: §3.5n verifies FINDINGS; §3.6j-ext sub-rule (i) verifies CITES (both at write-time).
- §3.5h MANDATORY-PRE-MERGE — the cite-drift-detector runs as part of the §3.5h gate sequence.

### 3.6k Iterate-to-convergence generalized to R1 + R4 + R4b (added 2026-05-19 Ben ratification — generalizes the Q5/R6 phase-close convergence cadence one tier up)

**Source-of-truth (read, do not re-derive):** memory `feedback_iterate_critical_reviews_to_convergence.md` (Ben-ratified 2026-05-19) + CLAUDE.md Non-Negotiable Process Rule 9 sub-bullet. This §-clause is the dispatch-context codification.

**The rule:** the iterate-until-convergence discipline is NOT R6-only. It applies to **every ADDL review tier that (a) gates an artifact, (b) whose triage mutates that artifact, and (c) is not re-reviewed by a downstream converging tier**. By that 3-part inclusion test the converging tiers are **R1 (plan/spec) + R4 + R4b (test suite) + R6 (phase-close)**. **Pre-work critics and per-group R5 mini-reviews remain SINGLE-PASS** — they are *feeders* whose findings are re-caught by a downstream converging tier (pre-work → R1-converges; mini-review → R4b/R6-converges); iterating them stacks redundant convergence loops for defense-in-depth that already exists. For any *new* review touchpoint, apply the 3-part inclusion test to decide whether it iterates.

**Termination mechanics — strict Q5 reused VERBATIM** (see `feedback_phase_close_final_council_full.md` + the R6 Q5 cadence block this §-catalog already governs): every round is the FULL appropriate council (never narrowed; same Pattern-6 lens composition; inverted/reconciliation posture where scope/forks are ratified-not-relitigated); only **BLOCKER/MAJOR** count as non-converged (MINOR/OBS never block); the pattern-induction meta-sweep + 3+-recurrence deep-sweep ride additively each round; the loop terminates ONLY when a full round returns 0 substantive findings — then (and only then) the tier advances.

**How to apply:** at R1, R4, R4b — after triage applies its FIX-NOW edits to the artifact, **re-dispatch the FULL council against the EDITED artifact**, triage that round, repeat; advance only when a full round = 0 substantive. Pre-work + per-group mini-reviews stay single-pass.

**Why:** triage at R1/R4/R4b *mutates* the artifact (plan R0.x→R0.(x+1); the test suite) and a single pass **never re-reviews the post-fix state** — the exact gap Q5 closes at R6, one tier up. Front-loaded convergence at R1/R4/R4b *shortens* the R6 cycle because defects don't accumulate to phase-close. Pairs with §3.5n orchestrator-ground-truth (the cross-check that lets convergence actually catch propagated/post-review drift) + the pattern-induction meta-sweep (additive each round).

**Composes with:** the R6 Q5 cadence (this is its generalization one tier up — same termination mechanics) + §3.5n cross-check + §3.7 3+-recurrence deep-sweep (rides each convergence round) + HARD RULE 12 (every round's findings get a real disposition, no float).

### 3.7 3+-recurrence triggers deep retrospective sweep (added 2026-04-29 R6 phase-close ratification)

**MANDATORY for orchestrator triage** at every R6 phase-close council + post-impl review. When ANY single failure shape fires 3+ times across the phase or R6 Round, the orchestrator MUST dispatch a deep retrospective sweep agent (read-only, cap-EXEMPT, ~90-180 min budget) targeted at THAT shape across the whole codebase — not just the per-lens findings.

**Why:** Empirically validated 2026-04-29 R6 phase-close: time-bounded R6 lenses caught ~50% of recurring-pattern instances on the same baseline. The metadata-producer-vs-consumer dedicated R6 lens caught 5+1; the deep retrospective sweep on the SAME baseline (broader scope, longer budget) caught 7 MORE — including a 2nd BLOCKER. If the deep sweep weren't dispatched, that 2nd BLOCKER (Instance 6 multi-label SUBSCRIBE delivery loss) would have shipped through phase-2b-close as a behavioral defect.

**Threshold = 3, not 5.** Lower bar than initial framing (which proposed 5+). Audits are cheap (single read-only agent dispatch) vs missed BLOCKERs are expensive (silent behavioral defects, recurring fix-passes within the same wave).

**Identifying recurrence:** classify findings by SHAPE not by lens or file. Examples from R6 Round 1 (which validated this rule):

| Shape | Round-1 instance count | Deep-sweep dispatched? |
|---|---|---|
| Producer/consumer drift (consumer drops a field producer wrote) | 13 (5+1+7) | YES — caught Instance 6 BLOCKER |
| Stale `#[ignore]`/`.skip` w/ landed deferral target | 11+85 | YES — caught ~85 missed instances |
| Cite-precision drift (stale line cites or wrong cross-refs in docs) | 6+7 | YES — caught 4 HIGH severity drifts |
| Engine-side claims symmetry but only eval-side wired | 4+4 | YES — caught NEW-1 BLOCKER |

**How to apply:** at every R6 council:
1. After all R1 lenses return, classify findings by SHAPE.
2. Count per shape. For each shape with count ≥3: dispatch a deep retrospective sweep targeted at THAT shape.
3. Bias toward thoroughness over speed in the deep sweep — they're cheap, catch ~50% of additional instances.
4. Fix-now everything the deep sweep finds (per HARD RULE rule-1; deep-sweep findings get the same disposition as R1 lens findings).

**Composes with §3.6:** the consumer-audit dimension prevents producer/consumer drift at the source (per-implementer brief check). §3.7 catches drift that already shipped before §3.6 was in place. Both run.

**Cross-references:**
- memory `feedback_3_plus_recurrence_deep_sweep.md` — full rule + threshold rationale
- `.addl/phase-2b/r6-round-1-triage.md` § Pattern shape clarification — Round-1 enumeration of shapes
- `.addl/phase-2b/r6-round-1-deep-sweep-{stale-deferrals,cite-precision,engine-eval-asymmetry}.md` — the 3 sweeps that validated the rule

### 3.7b Narrow-iteration cycle as effective FP follow-up (added 2026-05-03 R6-R5 ratification of pim-10)

**RECOMMENDED process-shape for every R6 phase-close cycle.** R6-R5 pim-10 named this positive pattern after R6-R4-narrow caught 3 fix-now items the R6-R4-FP mini-review missed; resolved by PR #75 in <1 hr. Multi-tier defense working as designed:

```
R6 Round N (full council) → fix-pass (R6-RN-FP) → mini-review → merge
                                                     ↓ if any post-merge concern surfaces
                                                  narrow iteration round (re-dispatch only the lenses
                                                  whose surfaces the FP touched, NOT full council) →
                                                  if any new findings → bundle into narrow-FP cycle →
                                                  R6 Round N+1 = full council redux
```

**Why this works:** mini-review is read-only single-agent (catches structural issues but limited surface). FP-mini-review-merge cycle has a window between merge + next-round full-council where post-merge regressions can hide. Narrow-iteration agents (~5-10 min each, cap-EXEMPT) close that window without paying the full-council cost. Trade-off: extra ~1 hr of orchestrator-time per phase-close cycle; in exchange, catches recurrences before they ship to next phase.

**When to use it:** every R6 phase-close cycle, between R6-RN-FP merge and R6-R(N+1) full council. Skip only if R6-RN-FP was a single-LOC docs fix with no behavioral surface.

**Composes with §3.7:** §3.7 catches PATTERN recurrences (3+-instances of same shape). §3.7b catches POINT recurrences (specific findings re-emerging post-FP-merge, possibly via incidental side-effects).

**Cross-references:**
- `.addl/phase-2b/r6-r5-pattern-induction-meta-sweep.json` — `r6-r5-pim-10` ratification finding (Narrow-iteration cycle as effective FP follow-up — POSITIVE process-shape)
- `.addl/phase-2b/r6-r4-narrow-iteration-producer-consumer.json` — narrow-iter caught Instance 21 (SubscribeArgs.handler) the R6-R4-FP mini-review missed
- `.addl/phase-2b/r6-r4-narrow-iteration-cite-precision.json` + `r6-r4-narrow-iteration-closure-verification-audit.json` — sibling narrow-iter dispatches (the 3 narrow-iter agents that caught 3 fix-now items)

### 3.8 Mini-review verdict shape (added 2026-05-03 R6-R5 ratification of pim-5)

**MANDATORY verdict format for read-only mini-reviewers.** Mini-review verdicts MUST be one of exactly 2 shapes:

- **READY-TO-MERGE** — every yes/no question yes; no fix-now items.
- **NEEDS-FIX-PASS** — at least one no; explicit list of fix-now items orchestrator must address before merge.

**FORBIDDEN:** the "READY-TO-MERGE-WITH-X" comma-clause variant. R6-R3 pim-5 named this failure mode: agents wrote verdicts like "READY-TO-MERGE pending the cite-precision drift fix at line 184" — the comma-clause hides fix-now items behind a green-sounding verdict, and orchestrator triages "WITH-X" as merge-now + carry-X-to-next-PR. Per HARD RULE rule-1, every fix-now item BLOCKS merge unless dispositioned (a) OUT-OF-SCOPE, (b) BELONGS-ELSEWHERE-NAMED-NOW with destination receiving entry NOW, or (c) DISAGREE-WITH-EXPLANATION. The comma-clause shape silently smuggles "carry to next" past the HARD RULE.

**Brief-template for read-only mini-reviewers:** the dispatch brief MUST require the agent to use exactly one of the 2 verdict shapes — no comma-clauses, no "READY-WITH-X-residual" variants, no "MERGE-NOW-AND-FIX-X-IN-FOLLOWUP" variants. If in doubt, use NEEDS-FIX-PASS + list X as a fix-now item; orchestrator can disposition with full context.

**Cross-references:**
- `.addl/phase-2b/r6-r3-pattern-induction-meta-sweep.json` — `r6-r3-pim-5` origin finding (Mini-review verdict 'READY-TO-MERGE-WITH-X')
- HARD RULE rule-1 in CLAUDE.md §"Non-Negotiable Process Rules" — the foundational rule this codification protects

### 3.9 R2 lens-menu correctness coverage (added 2026-05-03 R6-R5 ratification of pim-3)

**MANDATORY for every R2 dispatch.** R6-R3 pim-3 named the failure mode: R6 Round 2 dispatched 5 lenses (architect / doc-engineer / dx-optimizer / metadata-mpc / pim-meta-sweep); ALL 5 found docs-or-architecture findings; ZERO found correctness findings. Cause: the traditional R2 lens menu over-indexes on docs/arch surfaces while the R1 correctness lenses (code-reviewer / determinism / ivm / security / wasmtime) carry the correctness load.

**The rule:** every R2 dispatch MUST include AT LEAST one of the following correctness lenses:
- code-reviewer (or code-as-graph)
- determinism-verifier
- ivm-correctness
- security-auditor
- wasmtime-sandbox-auditor (when SANDBOX surfaces are in the wave)
- streaming-systems (when STREAM/SUBSCRIBE/EMIT/WAIT surfaces are in the wave)

**Acceptable alternative:** explicitly mark the R2 round as "docs+arch deep-pass" in the dispatch brief, in which case the correctness lenses are deferred to R3 / R4 / R6. This makes the lens-budget allocation deliberate rather than accidental.

**Brief-template addition:** every R2 dispatch brief MUST state either (a) "correctness lens included: <name>" or (b) "deliberate docs+arch deep-pass — correctness coverage deferred to <named round>".

**Composes with Pattern 6** (reviewer composition follows lens surface): Pattern 6 says pick lenses by what can go wrong with THIS work, not fixed review-size. §3.9 enforces correctness coverage doesn't accidentally fall off the lens-allocation budget at the R2 stage.

**Cross-reference:** `.addl/phase-2b/r6-r3-pattern-induction-meta-sweep.json` — `r6-r3-pim-3` origin finding (Round-2 lens-budget surface clustering).

### 3.10 Wave-pairing protocol (added 2026-05-03 R6-R5 ratification of pim-4)

**MANDATORY whenever two waves are dispatched as "paired siblings" with a claim that wave-A's coverage relies on wave-B's coverage (or vice versa).** R6-R3 pim-4 named the failure mode: mini-review rationales claimed "paired with sibling wave X" coverage that the sibling wave didn't actually do. Cause: parallel-dispatched waves assume each other's coverage without explicit hand-off contracts; orchestrator merged both on the strength of cross-pairing claims that nothing verified.

**The rule when waves are dispatched as paired siblings:**
1. **Both briefs MUST cross-reference each other explicitly.** Wave-A's brief states "wave-B covers <named surface X>"; wave-B's brief states "wave-A covers <named surface Y>". Neither brief is silent on the pairing.
2. **Both briefs MUST list the named coverage that the sibling is expected to provide.** "Sibling wave covers tests" is NOT named coverage; "sibling wave covers `crates/benten-eval/tests/wait_signal_shape_optional_typing.rs` invariant pin" IS.
3. **At merge time, the orchestrator MUST verify the claimed pairing closes the named coverage** — by reading both PRs' diffs against the claimed-coverage list. If a claim is unfulfilled, the wave is NOT merge-ready.

**Brief-template addition for paired waves:** include a "Pairing claims" section that names every cross-coverage assumption + which sibling-wave file:line satisfies each.

**Composes with §3.6 + §3.6c + §3.6d:** §3.6 catches drift across surfaces; §3.6c catches mirror-precedent overshoot; §3.6d catches assumed-translation-layer; §3.10 catches the analogous failure mode where TWO waves cross-reference each other's claims without verification.

**Cross-reference:** `.addl/phase-2b/r6-r3-pattern-induction-meta-sweep.json` — `r6-r3-pim-4` origin finding (Wave-8 'paired with sibling wave' lock-in).

### 3.11 Checkpoint-pre-flight-discipline for API-limit / mid-flight-stall recovery (added 2026-05-07 Phase-3 R5 wave-7 ratification of pim-19)

**MANDATORY when an implementer agent stalls mid-flight (weekly API limit / stream-watchdog timeout / process kill / network partition) leaving uncommitted partial work in its worktree.** R5 wave-7 named the failure mode at 4 instances (G19-B/G19-C1/G19-C2/G19-D all hit the same weekly-API-limit stall on first dispatch). The salvage-and-resume pattern is meaningfully reusable + the discipline-detail (checkpoint-commit annotation + resume-brief shape) is non-obvious enough that future agents shouldn't have to rediscover it from scratch.

**The recovery protocol when an agent stalls with uncommitted work in its worktree:**

1. **DO NOT discard the work.** The agent's partial implementation typically represents 25-80% of the scope (G19-B/C1/C2 all reached ~70-80%; G19-D reached ~25-30% with the LOAD-BEARING work still pending). Restart-from-scratch wastes that effort.
2. **Land a checkpoint commit per worktree, locally only (do NOT push).** Commit message annotation:
   ```
   checkpoint(<wave>): partial implementation pre-<root-cause>-recovery

   Salvage commit. Agents stalled at <reason> when <root-cause> hit;
   this preserves the ~<%> implementation as a rollback point. Resume
   agent will pick up from this state and complete the remaining scope
   per the original brief + plan §<N> row.

   NO LINT / NO PRE-FLIGHT — this is a recovery snapshot, NOT a finished
   unit of work. The follow-up resume commit will land the pre-flight
   discipline (cargo fmt + clippy + scoped test + SHAPE-not-SUBSTANCE +
   doc-coupling sweep + cite-drift + HARD RULE rule-12 dispositions)
   once the agent completes scope.
   ```
3. **Re-dispatch a fresh agent INTO the SAME worktree** with a "RESUME-AND-COMPLETE" brief that:
   - Names the worktree path + branch + checkpoint SHA explicitly.
   - References the original brief verbatim (the plan §N row is the scope contract).
   - Asks the agent to inspect `git show --stat <checkpoint-sha>` + read each modified file end-to-end FIRST.
   - Requires a `gap_analysis_vs_original_brief` field in the structured-JSON return (each owned-surface item: COMPLETE-IN-CHECKPOINT / NEEDS-COMPLETION / NEEDS-REWORK / DECISION-REQUIRED).
   - Lands NEW commits on top of the checkpoint (NOT amend the checkpoint — preserves the rollback boundary).
   - Pre-flight + SHAPE-not-SUBSTANCE + doc-coupling sweep + HARD RULE rule-12 dispositions on the FINAL state (not the checkpoint).
4. **The orchestrator MUST verify the resume agent's gap-analysis honestly identifies what's done vs missing** — don't trust "everything was already done" claims without spot-checking.

**Why "do NOT push the checkpoint":** the partial implementation typically has lint failures + missing tests + incomplete error handling. Pushing it triggers CI cycles on a known-broken state — wasting CI minutes + adding noise to the PR's history. The push happens with the resume commit on top, by which point pre-flight is clean.

**Composes with §3.5b HARDENED:** the resume agent's post-fix doc-coupling sweep should specifically check for stale prose introduced by the partial work (e.g., a docstring referencing an unfinished symbol).

**Cross-reference:** R5 wave-7 NS-T9..T11 entries in `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md` — origin precedent (4 instances; weekly-API-limit reset; checkpoints at 71d12bf / 8720b96 / 0cdcaf1 / 64fc1a8; resume commits at a55c24c / 737fa47 / 2fa003f / 49a7f29). Same pattern applies for non-API-limit stalls (process kill / network partition / OOM) — the discipline-detail is the same.

### 3.12 R7-equivalent spec-to-code-compliance audit at every phase-close (added 2026-05-09 Phase-3 R6 R1 ratification of pim-13)

**MANDATORY for every phase-close R6 final convergence round.** The R6 final round per `feedback_phase_close_final_council_full` runs the full Round-1 lens set against the pre-tag HEAD; §3.12 adds a STANDING COMPANION lens that walks every spec doc claim + every named compromise + every D-PHASE-N decision-log item to a code construction site at HEAD. This is the Phase-1 R7 discipline ("aspirational prose but dead code") revived as a recurring pattern, not a one-off Phase-1 audit.

**Origin and gap-detection:** Phase-1 R7 invented this discipline; the Phase-1 retrospective claimed it was retired post-Phase-1 because "the discipline merged into R6 verify-don't-trust-docs." Phase-3 R4b found the discipline has lapsed in practice — Phase-2b R6 was multi-round lens-based (not spec-to-code-walk); Phase-3 R6 will follow the Phase-2b shape unless §3.12 reactivates it explicitly. The lapse is consequential because lens-based reviewers sample the codebase by their lens's surface; a spec claim that no lens's surface covers can hide indefinitely. The R7-equivalent walk is an EXHAUSTIVE traversal of the spec corpus, not a sample.

**The audit shape:** every phase-close R6 final round dispatches a `r7-spec-to-code-compliance` lens (read-only, cap-EXEMPT, ~120-180 min budget, parallel with the rest of the full-council lens set). The lens's brief contract:

1. **Walk every claim in every spec doc** at `docs/*.md` (VISION, ARCHITECTURE, ENGINE-SPEC, HOW-IT-WORKS, FULL-ROADMAP, SECURITY-POSTURE, ERROR-CATALOG, INVARIANT-COVERAGE, MODULE-MANIFEST, SANDBOX-LIMITS, GLOSSARY, TYPED-CALL, ATTACK-SURFACE-MATRIX, plus phase-specific docs). For each claim of runtime / behavioral / structural property, locate a code construction site at HEAD that substantively realizes the claim. Findings = (claim, expected-construction-site-shape, actual-construction-site-state).
2. **Walk every named Compromise** in `docs/SECURITY-POSTURE.md`. For each Compromise, verify (a) the closure state matches the prose; (b) the residual disclosure prose matches the actual production-default behavior of the composed primitives.
3. **Walk every D-PHASE-N item** in the active phase's decisions log (`r5-decisions-log.md` or equivalent). For each D-entry that names a code-side commitment, verify the commitment lands in code at HEAD — not just in a commit message or a NS-T entry.
4. **Walk every named Invariant** in `docs/INVARIANT-COVERAGE.md`. For each invariant, verify the enforcement state matches the prose AND the invariant's named firing site is reachable from a production code path.

**Per-finding output shape:** structured-JSON entry per non-conformance:
```
{
  "claim_locus": "<doc:section> (e.g. VISION.md §3 'Atriums sync via iroh + Loro')",
  "expected_construction_site": "<crate::module::symbol or doc reference>",
  "actual_state_at_HEAD": "<what's there: missing / inert / partial / wrong-surface>",
  "severity": "BLOCKER | MAJOR | MINOR | NEGATIVE-CONFIRMATION (claim verified)",
  "disposition": "<HARD RULE rule-12 disposition>"
}
```

**Why this is a STANDING pattern, not phase-specific:** Phase-1 R7's findings were not Phase-1-only failure modes. The class of failure ("aspirational prose accumulates faster than code"; "compromise prose drifts from primitive defaults"; "decision-log items get one mention then never land") is a feature of long-lived spec-heavy codebases. Every phase risks accumulating the same shape; every phase-close should sweep for it.

**Companion to §3.7 + `feedback_phase_close_final_council_full`:** §3.7's deep retrospective sweep targets a SHAPE that fired 3+ times (reactive). §3.12 walks the spec corpus EXHAUSTIVELY (proactive, regardless of whether a shape has fired yet). Both run at every phase-close.

**Skill-creation deferred to phase-3-close work item.** A `tools/spec-compliance-audit/` skill that automates parts of the walk (e.g., extracts all "MUST" / "SHALL" / "is enforced by" claims from spec docs + matches against `grep`-able code patterns) is a Phase-3-close-or-later implementation; not pre-tag-blocking. The MANUAL §3.12 lens dispatch is the load-bearing part; the skill is automation gravy.

**Cross-references:**
- memory `feedback_pim_13_r7_spec_to_code_compliance_audit` — full rationale + Phase-1 R7 origin + revival precedent.
- `feedback_phase_close_final_council_full` — composition partner (full Round-1 council).
- `.addl/phase-3/r4b-pattern-induction.json` — gap-detection finding (Phase-1 R7 discipline lapsed).
- `.addl/phase-3/r6-r1-pattern-induction.json` — R6 R1 ratification recommendation.
- `docs/history/PHASE-1.md §5` — original R7 audit precedent (frozen retrospective; reference only).

### 3.13 Test-isolation: per-test static decomposition for process-scoped shared state (added 2026-05-09 R6 R6-final ratification of pim-N-test-isolation-process-scoped-shared-state)

**MANDATORY when a test module / proptest module uses process-scoped shared state (`static MOCK_*: AtomicU64`, mutex-guarded global, lazy-init singleton) to inject a clock / RNG / config into a primitive-under-test.** Single-static designs are race-fragile under parallel test execution AND under coverage-instrumentation slowdown (cargo-llvm-cov widens race windows substantially); each test that resets the static can lose its reset to a sibling test's reset that lands between the reset and the first observation.

**The rule:** decompose ONE shared `static` into N per-test statics, one per `#[test]` fn that depends on the state, with a semantic name that names the test fn. Each test's first action is `MOCK_*.store(N, SeqCst)` to re-pin starting state — defends against any prior-test residual mutation (the per-test static structurally precludes cross-test contention, but the explicit reset is belt-and-braces).

**Confirmed instances at HEAD post-ratification (6 instances; 3+-recurrence threshold MET):**

1. R6 R1 hlc-r6-r1-3 inline `mod tests` at `crates/benten-core/src/hlc.rs:380+` (4 instances of `MOCK_TIME_MS` shared-static contention pre-decomposition).
2. R6 R1 hlc-r6-r1-4 proptest sibling at `crates/benten-core/tests/prop_hlc_monotonic.rs` (2 instances of `MOCK_PROP_TIME_MS` shared-static contention pre-decomposition).
3. PR #171 batch-2 r6-r2-hlc-1+2 (commit `2d8e4b5`) closed all 6 substance instances — `crates/benten-core/src/hlc.rs:380+` now carries 7 per-test statics + 7 bare-fn wrappers; `crates/benten-core/tests/prop_hlc_monotonic.rs` carries 2 per-proptest statics + 2 bare-fn wrappers + structural-defense narrative.
4. PR #168 cargo-llvm-cov flake (`left:2000/right:10000` on `now_bumps_logical_when_physical_clock_stalls`) is the runtime-incident exemplar — root-caused by shared-static cross-test contention; post-decomposition structurally preclusive, not retry-defended.

**The shape (brief + agent + reviewer co-applied):**

**Brief side (when authoring an implementer brief that touches a test module / proptest module with shared `static` injection):**
- Brief MUST identify whether shared `static` is in play.
- If yes: brief MANDATES per-test decomposition with semantic naming convention.
- Naming convention: `MOCK_<TEST_NAME_FRAGMENT>_<UNIT>` (e.g. `MOCK_NOW_ADV_TIME_MS` for `now_advances_when_physical_clock_advances`).
- Each per-test static gets a paired bare `fn <test_name>_clock(&self) -> u64` wrapper that closes over only that static (not closures — closures over `&'static AtomicU64` defeat the structural isolation by making the static reference fungible across tests).

**Agent side:**
- Decompose the single shared `static` into N per-test statics with semantic names matching the `#[test]` fn each guards.
- Each test's first action: `MOCK_*.store(STARTING_VALUE, SeqCst)`.
- Replace any closure-based mock-time-injector with bare `fn` wrappers that reference exactly ONE per-test static.
- Add a module-level comment cross-referencing the sibling site (inline `mod tests` ↔ `tests/prop_*.rs`) explaining the structural-defense posture.

**Reviewer side:**
- Mini-reviewer MUST verify the decomposition is structural (per-test static), not just runtime-isolated (mutex-per-test). Mutex-per-test still allows cross-test contention if any test forgets to lock; per-test statics make cross-test contention compile-impossible.

**Composes with §3.6e (RED-PHASE staged-pin → un-ignore discipline):** if a test was `#[ignore]`'d under the old shared-static design (because of flake), the un-ignore path requires the per-test static decomposition first. The R6 R1 hlc-r6-r1-3+4 cluster + PR #168 cargo-llvm-cov flake are the load-bearing exemplar.

**Cross-references:**
- memory `feedback_pim_test_isolation_process_scoped_shared_state` — full rationale + 6-instance recurrence log + naming-convention examples.
- `.addl/phase-3/r6-r1-hlc-clocks.json` — R6 R1 hlc-r6-r1-3 + hlc-r6-r1-4 origin findings.
- `.addl/phase-3/r6-final-3+-recurrence-deep-sweep.json` — pim_n_ratification_recommendations entry naming this rule.
- §3.6e — RED-PHASE staged-pin discipline composes with the un-ignore path.

---

## 4. Sync mini-review timing — pre-merge default (METH-6)

**Default: pre-merge against the green PR's branch.** Preserves the Phase-2a invariant that the orchestrator never merges unreviewed code to main, and keeps fix-passes branch-local (cleaner main history).

Mini-reviewer dispatched against the PR's branch state:
```
gh pr checkout <pr-number>     # in the reviewer's worktree
# reviewer reads + reports findings
```

Orchestrator merges only after BOTH:
- CI green on the PR head, AND
- Mini-review either merge-as-is OR fix-pass landed + new CI green.

Cost: a fix-pass re-trigger costs one CI cycle (~7 min) but buys clean main history. Per CLAUDE.md Pattern 1 (do-it-right-not-fast), this trade is correct.

**Post-merge against main is reserved** for retroactive checks (e.g. a new lens added mid-phase reviewing already-shipped surface, or R6 quality-council reviewing previously-merged R5 work).

---

## 5. Concurrency budgeting — purpose-conditional (METH-8)

GitHub Actions free-tier caps: 20 concurrent Linux jobs / 5 macOS / 5 Windows. Required-check matrix consumes ~7 Linux jobs per branch push.

- **Substantive R5 implementation groups (G6 / G7 / G8 agents):** cap at **2 parallel agents per wave** (~14 Linux jobs concurrent — no queueing; tight iteration cycles).
- **R6 surface-grouped fix-passes (per `feedback_parallelize_fixpasses` 3-4 buckets — Rust-correctness / docs / CI / TS+napi):** cap at **3 parallel agents per wave** (~21 Linux jobs — marginal queueing on the 4th matrix leg, ~5 min cost; parallel-dispatch wall-clock win exceeds the queueing cost).
- **Reviewer-only dispatches** (no branch push, no CI cost): unbounded — Pattern 6 reviewer composition follows lens surface, not concurrency limits.

Math: `4 parallel × 7 jobs = 28 > 20-job cap` (queueing). `3 × 7 = 21` (marginal). `2 × 7 = 14` (no queueing). Bumpable as the matrix is leaned (matrix-leaning is a separate Phase-2b improvement; tracked in `r5-decisions-log.md` D-entries when it lands).

Prior reference: see Phase-2a `r5-decisions-log.md` D13.4 (the original Phase-2b ADDL redesign decision; not §5 D13 in the plan, which is a forward-referencing typo).

---

## 6. Auth audit (METH-11)

- Agents inherit the `gh` CLI auth of the orchestrator's sandbox (currently `Benten-Ben`).
- **Standing rule:** at agent onboarding (start of dispatch), run `gh auth status` smoke check. If unauthenticated, halt + report in deviations; do NOT attempt other auth paths (no token-from-env experiments, no `gh auth login` interactive flows).
- If push 403s after `gh auth status` reports authenticated, escalate as deviation — likely a branch-protection scope mismatch (admin token may be needed for protected-branch operations; orchestrator decides path forward, not the agent).
- **Default push authority resolution:** worktree-direct (agents push their own branches via inherited `gh` auth). Fallback (D13 Option B): orchestrator-proxy push — agent commits locally, returns SHAs, orchestrator pushes from its own session.

---

## 7. Merge-on-green playbook

| Scenario | Procedure |
|---|---|
| Single-branch clean merge | `gh pr ready <pr>` → `gh pr checks <pr>` (confirm all required green) → `gh pr merge <pr> --rebase --delete-branch` |
| Multi-branch stacked merge (all green) | Merge in declared dispatch order. After each merge, rebase later branches on main + force-push-with-lease + wait for re-CI green (§2.9) before merging next |
| Fix-pass on red branch | Dispatch single agent against same branch (or orchestrator inline-fix per §2.8); CI re-runs on push; merge when green per default flow |
| Predecessor stalled, sibling green | Per §2.7 escalation: 14-min threshold → inspect predecessor → wait / fix-pass / merge-out-of-order with deviation log |
| Abort + reset | `gh pr close <pr>`; `git push origin --delete phase-2b/<group>/<agent-slug>`; remove worktree; re-dispatch with corrected scope |

---

## 8. Retrospective hook — two checkpoints (METH-14)

Capture observations in `.addl/phase-2b/r5-decisions-log.md` (analogue to Phase-2a's r5-decisions-log).

- **Checkpoint A:** immediately after G12-A dry-run completes (procedure-only iteration before G6/G7/G8 fan-out — captures lessons from a single-agent canary). One D-entry.
- **Checkpoint B:** after first 2-3 multi-agent waves (captures parallel-agent + concurrency-budget lessons). Subsequent D-entries per wave close.

Specific data to capture: time-to-CI-green per branch, time-to-merge after CI-green, fix-pass frequency, queueing observed, any procedure friction.

---

## 9. Completed-group ownership (DO-NOT-TOUCH frozen-file list)

Carried forward from Phase-2a §4 — Phase-2b inherits all those frozen surfaces (G1-A through G5-B-ii are landed and CID-pinned). Plus Phase-2b adds entries as groups close.

**Phase-2a frozen surfaces** (do not touch without explicit brief authorization — repeated from `.addl/phase-2a/dispatch-conventions.md` §4 for reference; that doc remains authoritative for Phase-2a-era detail):

- **G1-A / G1-B:** `benten-eval/src/host_error.rs`, `benten-eval/src/invariants/structural.rs` arch-1-section, `benten-errors/src/lib.rs` HostError variants.
- **G3-A:** `benten-eval/src/exec_state.rs` envelope shape (FROZEN).
- **G3-B / G3-B-cont:** `benten-engine/src/engine_wait.rs` resume-protocol shape, `bindings/napi/src/wait.rs` surface, `packages/engine/src/dsl.ts` WAIT DSL shape, `benten-eval/src/primitives/wait.rs` body. (G12-E will modify the metadata-store layer below this surface — the resume-protocol shape itself remains frozen.)
- **G9-A:** `benten-caps/src/grant_reader.rs` batch method, `benten-eval/src/time_source.rs` trait shape.
- **G2-A / G2-B:** `benten-graph/src/{redb_backend.rs, transaction.rs, immutability.rs}` core shapes, `benten-engine/src/subgraph_cache.rs` key shape.
- **G4-A:** `benten-eval/src/invariants/budget.rs`, `benten-eval/src/evaluator/budget.rs` helper shape, `benten-caps/src/grant.rs` ucca-7/8.
- **G5-A:** `benten-graph/src/redb_backend.rs::put_node_with_context` 5-row matrix body.
- **G5-B-i:** `benten-engine/src/primitive_host.rs` Inv-11 runtime block, `benten-engine/src/system_zones.rs` prefix list, `benten-graph/src/store.rs::NodeStore::get_node_label_only` surface.
- **G5-B-ii:** `benten-eval/src/invariants/attribution.rs`, `benten-eval/src/evaluator/attribution.rs`, `benten-eval/src/evaluator/attribution_schema_fixture.rs` CID pin.

**Phase-2b frozen surfaces:** initially empty. Each Phase-2b group close adds entries here in a follow-up edit (the closing group's mini-review report includes a "frozen surfaces to add" section).

If your brief requires touching one of these files, the brief will explicitly authorize the surface + reason. Absent explicit authorization: out of scope; report as deviation per §11.

---

## 10. Commit format (carried from Phase-2a §5)

- Conventional Commits: `<type>(<scope>): <summary>`. Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`, `build`.
- Title ≤ 70 chars. Body explains WHY + lists per-file scope. Include ADDL provenance (e.g. `feat(benten-eval): R6 G7-A SANDBOX core + named-manifest codegen`).
- **No AI attribution.** Body ends with the last line of the narrative.
- **Do NOT push to `main`.** Push to your feature branch only (§2.3).

---

## 11. Report template

```
## Pre-flight (OPTIONAL — only if run)
cargo check -p <crate>: <exit> / <last 5 lines>
cargo clippy -p <crate>: <exit> / <last 5 lines>
cargo fmt: <exit>

## Branch + PR
Branch: phase-2b/<group>/<agent-slug>
PR: #<number> (https://github.com/BentenAI/benten-engine/pull/<number>)
Commit SHAs: <list>

## Named deviations from brief
<each one with Rule-1 disposition: fix-now / defer-with-doc-target / disagree>

## Dead-code sweep — items removed
<list>

## Files touched (final — verified via git show --stat)
<list>

## Must-pass tests — per-test status
<name>: green / red / deferred with reason

## Notes for orchestrator
<anything that needs human decision — including any auth-status anomalies, push errors, or worktree-cleanup items>
```

---

## 12. Brief brevity expectation

Briefs reference this doc with one line: "Apply [dispatch-conventions](./dispatch-conventions.md)." Specific items, owned files, must-pass tests, branch name (orchestrator-assigned), and any deviations from these standing rules go in the scoped brief body. Don't re-spell pre-flight / no-stash / commit format / no-AI-attribution / branch-naming / merge-procedure rules — they're here.

---

## 13. Procedure failure modes (NEW for Phase 2b)

Concrete halt-and-report triggers. Agents do NOT improvise around these.

| Failure | Diagnosis | Resolution |
|---|---|---|
| `gh auth status` reports unauthenticated at onboarding | Auth scope didn't propagate to agent worktree | Halt + report; orchestrator decides D13 fallback (Option B proxy-push) |
| `git push -u origin <branch>` 403s despite `gh auth status` ok | Likely branch-protection scope mismatch on a protected-branch operation | Halt + report; orchestrator escalates to Ben (admin-token decision) |
| `gh pr create` fails with "base branch missing" or "PAT scope" | Base ref or token scope bug | Halt + report; orchestrator diagnoses |
| Required-check workflow doesn't trigger on PR open | Workflow YAML missing `pull_request: branches: [main]` trigger (METH-1 + METH-13) | Land the Bucket-3 workflow YAML expansion fix-pass first; then re-push the agent branch |
| Merge conflict on `gh pr merge --rebase` | Branch diverged from main during dispatch | Per §2.9: orchestrator rebases + force-pushes-with-lease + waits for re-CI; if non-trivial, halt + dispatch fix-pass |
| CI runner concurrency exhausted (queue grows past 5 min) | Hit free-tier matrix cap | Wait for capacity; reduce per-wave parallelism temporarily (drop from §5 cap by 1 for the next wave) |
| Worktree command fails because path already exists | Prior dispatch didn't clean up | `git worktree remove --force <path>` then retry |
| Agent attempts `git push origin main` directly | Hard violation — should never happen; branch protection blocks regardless | Report immediately; escalate as a procedure-discipline gap |

---

### 3.14 Strategy-C batch-merge for accumulated wave-PRs (added 2026-05-13 Phase-4-Foundation R5 ratification of `feedback_batch_merge_strategy_c`)

**MANDATORY when ≥3 wave-PRs accumulate unmerged simultaneously** AND **mini-reviews have cleared substance**. Strategy-C is the CI-throughput discipline that collapses N sequential CI cycles into ⌈N/2⌉-⌈N/3⌉ batched cycles by local-merging waves into 2-3-wave batches + single PR per batch.

**Why mandatory** (per Ben ratification 2026-05-13 Q2): without Strategy-C, the GitHub Actions queue serializes 11 long CI workflows × N PRs × ~30 min each = ~5.5N hours per cluster. Empirically validated by 5 batches landing 15 waves via 5 CI cycles in this Phase 4-Foundation R5 (would have been ~22 hr/4-PR cluster sequentially).

**Trigger threshold:** ≥3 wave-PRs open simultaneously where mini-reviews have cleared substance.

**Sizing rule:** 2-3 waves per batch (sweet spot). >3 makes cross-wave conflict reconciliation unwieldy + larger blast radius if CI fails. =2 is the safer fallback. Respect dependency graphs — if wave Y consumes wave X's API, either put them in the same batch OR put X in batch-N and Y in batch-(N+1) so Y rebases onto post-X main cleanly.

**Process:**
1. Create local integration branch off `origin/main` HEAD (`r5/batch-N-<phase-tag>` or equivalent).
2. Cherry-pick each wave's commits in dependency-free order (preserves per-wave commit attribution via rebase-merge).
3. Resolve cross-wave conflicts ONCE in one local pass — usually concentrated in ErrorCode catalog (`stable_shape.rs` + `lib.rs` + `errors.generated.ts` + `ERROR-CATALOG.md`), Cargo.toml dep declarations, Cargo.lock, `phase-N-backlog.md` section numbering.
4. Run orchestrator-direct §3.5h pre-flight on combined tree (fmt + scoped check + scoped clippy + scoped nextest + cite-drift).
5. **MANDATORY: regenerate `packages/engine/src/errors.generated.ts`** via `npx tsx scripts/codegen-errors.ts` if any wave minted new ErrorCodes. The required-10 CI check "docs ↔ Rust enum ↔ TS types parity" re-runs codegen + diffs against the committed file; even when `npx tsx scripts/drift-detect.ts` PASSES locally (three-way agreement), the CHECKED-IN .ts may still be stale after a cherry-pick.
6. Push as single PR; close per-wave PRs as superseded + cancel their queued workflow runs (saves ~75% of queue thrash).
7. Admin-merge with **rebase-and-merge** (NOT squash) — preserves per-wave commits on main; commit messages carry per-wave attribution.
8. Each batch's downstream waves cascade-dispatch off the batch merge.

**Conflict-resolution patterns to expect** (per `feedback_batch_merge_strategy_c.md` for the full table):
- **CATALOG_VARIANT_COUNT**: collapse the two `assert_eq!` to one with combined count; keep BOTH waves' comment blocks.
- **ErrorCode enum + as_str/from_str + ON_DENIED + TS classes + ERROR-CATALOG.md**: strip conflict markers; keep both ADD blocks; regenerate TS.
- **Cargo.toml dep duplicates**: collapse to superset (e.g. `serde = { workspace = true, features = ["derive"] }` is superset of plain `serde = { workspace = true }`).
- **Cargo.lock**: `rm Cargo.lock` + let `cargo check` regenerate.
- **`phase-N-backlog.md` section numbering**: renumber the later wave's sections + atomically grep + update test-file ignore-message §-references.
- **`-X ours` strategy** for stylistic comment-only conflicts CAN drop substantive renames from the incoming wave; spot-check + `git checkout origin/<wave-branch> -- <file>` for files where the incoming wave's content is load-bearing.

**Don't batch R6 fp:** R6 phase-close convergence depends on iterating rounds with clean attribution per fix; batching obscures which fix closed which finding. Strategy-C is for R5 implementation + R4b-FP only.

**See also:** Companion memory `feedback_batch_merge_strategy_c.md` carries the empirical case-study (5 batches in Phase 4-Foundation R5; wall-clock saved 6-15 hr per batched cluster).

---

### 3.15 Phase-close archive sweep + retrospective composition (added 2026-05-14 Phase-4-Foundation close codification)

**MANDATORY post-phase-close workflow** after the `phase-<N>-close` tag lands. Consumes the phase's `.addl/<phase>/` working artifacts, archives them to `.addl/_archive/<phase>/`, and composes the canonical `docs/history/PHASE-<N>.md` retrospective.

**Step 1 — Partition sizing.** Pick the agent shape based on `.addl/<phase>/` content size:
- **< 50 files / < 250k tokens**: single-agent (Phase 1 + Phase 2a precedent).
- **50-200 files / ~250k-1M tokens**: 3-partition (Phase 2b + Phase 3 + Phase 4-Foundation precedent).
- **> 200 files / > 1M tokens**: 4-5 partition.

For Opus 4.7 1M agents: each partition agent should land in the 200-400k token range. Anything tighter risks summarize-and-discard discipline degradation.

**Step 2 — Partition criteria.** Pick the cut that produces a coherent narrative per agent:
- **Chronological by round** (Phase-4-Foundation precedent): pre-R6 era / R6 R1-R4 / R6 R5-R8.
- **By theme** (alternative): foundation+decisions / implementation / quality-council.
- **By round-set + working artifacts** (earlier-phase precedent): per-round per-lens.

**Step 3 — Per-partition agent brief shape:**
1. Read every file in your partition end-to-end.
2. Move each file via plain `mv` (NOT `git mv`; `.addl/` is gitignored) to `.addl/_archive/<phase>/`.
3. Write `.addl/<phase>/_partition-<X>-summary.md` mirroring the consolidator's 6-section target template (see Step 5).
4. Reference prior retrospectives (`PHASE-1.md / PHASE-2a.md / PHASE-2b.md / PHASE-3.md`) as prose-style + density anchors.

**Step 4 — Consolidator (orchestrator) role:**
1. Read all N partition summaries.
2. Read prior retrospectives for 6-section template + prose density.
3. Read context: CLAUDE.md status header + baked-in commitments + current backlogs.
4. Write `docs/history/PHASE-<N>.md` matching the 6-section template. Chunked write (Write-then-Edit per section) for write-limit avoidance if needed.
5. Archive the partition summaries themselves to `_archive/<phase>/` after consolidation.

**Step 5 — Canonical 6-section retrospective template** (mirror PHASE-1/2a/2b/3.md):
1. **Status header** — tag SHA + close date + convergence stats + workspace state + canonical fixture CID
2. **§1 Narrative journal** — ~80-200 lines dense continuous prose; sub-sections match THAT phase's actual arc; cite PRs + SHAs + finding-IDs
3. **§2 Changelog** — engine surface / compromises / invariants / test coverage milestones / tooling+CI / docs / ErrorCodes
4. **§3 Key takeaways** — what this phase was fundamentally about / 3-5 hardest problems / 2-4 surprises / 1-3 phase-defining decisions
5. **§4 Backlog / compromises / incomplete work** — carried-in / deferred-out / compromises landed this phase
6. **§5 Process lessons / pim-N catalog** — ratified pim-N + amendments + drift catalog + cross-cutting failure shapes
7. **§6 Decisions baked in / architectural commitments** — new CLAUDE.md additions + re-affirmations

**Step 6 — Follow-up cleanup sweep** (added 2026-05-14 after the Phase-4-Foundation close caught era-mismatch leftovers that earlier phase-close sweeps missed). After the canonical archive + retrospective land, single agent sweeps:
- `.addl/<earlier-phase-era-directories>/` (e.g. `.addl/phase-3-doc-review/`, `.addl/phase-3-test-review/`, `.addl/spike/`, `.addl/proposals/`) — Phase-1/2/3-era artifacts that escaped prior phase-close cleanups.
- Working-area duplicates (e.g. `.addl/_archive-extraction/` from Phase-3-close staging) — verify duplicate-vs-unique-methodology-reference before deleting.
- Empty residual subdirs + nested-`.addl/` artifacts from partition agents using relative paths.

Cleanup-sweep agent cross-checks each file against the relevant retrospective + backlogs — surfacing any genuinely-unaddressed findings as recommended amendments. Most leftovers are `ALREADY-ADDRESSED-IN-FLIGHT-PRs` per original triage; rare unaddressed findings are the load-bearing output.

**Don't archive:**
- `.addl/dispatch-conventions.md` — actively load-bearing standing rules
- `.addl/phase-<N>/` for any in-progress OR scheduled-next-but-not-opened phase
- Most-recent HANDOFF for any open phase — load-bearing for fresh-agent re-orient

**Empirical case study — Phase-4-Foundation close (2026-05-14):**
- 200 files / ~1.1M tokens / 3 partitions × 66-68 files each
- Each partition agent produced 275-337 line summary
- Consolidator (orchestrator) wrote 382-line retrospective in single Write call
- Follow-up cleanup sweep handled ~40 leftover files from Phase-1 spike + Phase-3 doc/test review + working-area duplicates
- Zero unaddressed findings surfaced — all content already absorbed into main + retrospectives via prior PR work

**See also:**
- memory `feedback_phase_close_archive_sweep.md` — full pattern + partition criteria + canonical template
- memory `feedback_phase_close_final_council_full.md` — Q5 cadence that governs the R6 rounds whose artifacts this sweep consumes
- `docs/history/PHASE-1.md / PHASE-2a.md / PHASE-2b.md / PHASE-3.md / PHASE-4-FOUNDATION.md` — prior retrospective precedents
- `.addl/_archive/archive-extraction-process/` — preserved methodology reference (stage-1 extraction outputs that produced PHASE-{1,2a,2b,3}.md)

---

*Last edited 2026-05-14 (Phase-4-Foundation close codification): added §3.15 Phase-close archive sweep + retrospective composition pattern (promoted from `feedback_phase_close_archive_sweep.md`).*

*Prior edit 2026-05-13 (Phase-4-Foundation R4b-FP-3 + Ben Q2 ratification): added §3.14 Strategy-C batch-merge codification (promoted from `feedback_batch_merge_strategy_c.md`).*

*Prior edit 2026-05-03 (Phase-2b-close + Ben morning-review): consolidated all 11 pim-N codifications inline (§3.4b cross-crate workflow-constraint exception per pim-6; §3.5 dimension #5 stable-doc-leg per pim-7; §3.5b HARDENED post-fix doc-coupling per pim-1+9; §3.6b end-to-end test pin per pim-2; §3.6c mirror-precedent overshoot guard per pim-8; §3.6d reviewer translation-layer cite-discipline per pim-11; §3.7b narrow-iteration cycle per pim-10; §3.8 mini-review verdict shape per pim-5; §3.9 R2 lens-menu correctness coverage per pim-3; §3.10 wave-pairing protocol per pim-4). Originally edited 2026-04-25 alongside `.addl/phase-2b/00-implementation-plan.md` §6 per pre-R1 methodology critic findings METH-1 through METH-16; subsequent edits as the frozen-surface list (§9) + R6 ratifications accumulated. Subsection numbering is additive-by-discovery (3.5 → 3.5b → 3.4 → 3.4b → 3.6 → 3.6d → 3.6c → 3.6b → 3.7 → 3.7b → 3.8 → 3.9 → 3.10) — semantic clusters are §3.4-* (cross-crate cascade), §3.5-* (cross-target/doc-coupling), §3.6-* (consumer-audit + test-pin + mirror-precedent + translation-layer), §3.7-* (recurrence + narrow-iter), §3.8-3.10 (mini-review + R2-lens + wave-pairing).*

---

## §3.5m — Fork-disposition standing principles (Ben-ratified 2026-05-17, refinement-audit-2026-05)

Authoritative detail + per-fork register + the 3 ratified Phase-4-Meta design decisions: **`.addl/refinement-audit-2026-05/RATIFIED-decisions-2026-05-17.md`**. Summary (apply as DECIDED policy — these fork-shapes are no longer "surface, don't decide"):

- **P-I — commit to the elegant permanent shape; full-migrate now; NO dual-track.** Fallible-vs-infallible / rename-for-consistency / API-shape forks: pick the correct *permanent* shape (current + plausible-future needs), fully migrate now (agent-economics makes call-site migration cheap). NEVER add-alongside-and-defer or leave a low-priority cleanup ticket — that deferred dual-track IS the debt this campaign kills.
- **P-II — cross-crate mechanical sweeps = ONE orchestrator-serialized workspace pass post-COLLAPSE**, not per-lane. A correct fix cascading across COLLAPSE-owned crates (engine/caps/napi/id) batches into a single workspace sweep when the spine frees those crates.
- **P-III — wire/CID/on-disk-format changes are NEVER autonomous, NEVER pre-v1-incidental.** Canonical-bytes/CID/on-disk-envelope changes are versioned v1-wire-freeze decisions Ben makes; never a refactor side-effect. (Orchestrator latitude here may widen later — not yet.)

Composes with: agent-economics-prefer-thorough-cleanup memory; HARD RULE 12 (these principles sharpen *which* fix = the proper permanent one); §3.5l (mega-batch full-workspace verify still mandatory on the resulting migrations).

## §3.5n — Review-finding ground-truth verification (Ben-discipline, 2026-05-17; 3rd-recurrence ratified)

**Mini-reviews can produce stale-view FALSE-POSITIVE MAJOR/BLOCKER/closure findings EVEN WHEN they claim `git rev-parse origin/main` pre-flight passed** (review-1268 ran vs stale 8141b948; review-1279 falsely claimed `Acceptor` still present; review-1281 falsely claimed phantom-§4.68/6-markers — all contradicted by actual origin/main). Root cause: agents fetch+rev-parse correctly but then read STALE WORKING-TREE / cached content instead of the ref.

RULE: (a) Cap-exempt review briefs MUST verify findings via **ref-pinned reads** — `git show origin/<main>:<path>` / `git grep <sha> -- <path>` — NOT working-tree reads. (b) **The ORCHESTRATOR independently ground-truth-verifies EVERY review MAJOR/BLOCKER/"must-stay-open"/phantom-destination/closure-adjudication finding via its own ref-pinned read BEFORE acting on it (HANDOFF note, fix-dispatch, or closure-set) — never propagate a review finding unverified.** A downstream agent's evidence-based DISAGREE-WITH-EXPLANATION (HARD RULE 12c) against a review finding is a first-class signal: re-verify, don't override the disagreer. Generalizes "trust-but-verify agent verdicts" → trust-but-verify REVIEW findings too. Composes with §3.5i (reviewer rebase-staleness) + the verify-don't-misreport / #1235 lesson.

---

## §3.5o — CI x86_64-Linux disk-OOM is its own discipline class (Ben-codified 2026-05-22 from PR #1312 + #1313 evidence)

**When CI hits `No space left on device (os error 28)` during cargo build OR `collect2: fatal error: ld terminated with signal 7 [Bus error]` 2× consecutively on the same ubuntu-24.04 GH-hosted x86_64 runner, that's a recurring infra issue NOT a transient flake.** Both symptoms are the same disk-OOM root-cause-class (the second is ld's mmap of intermediate files truncating when the temp dir runs out of disk mid-link). Treat as infrastructure-fix-blocker against the actual error mode; do NOT rerun blindly + do NOT default to mold linker (mold is a memory-pressure-during-link fix that writes the same output size to the same disk — it does NOT directly address disk-OOM).

**The proper permanent fix shape (validated by PR #1313 vs the pre-compact mold-direction reconsidered against the actual disk-OOM error):**

1. **`CARGO_INCREMENTAL=0`** in the workflow's `env:` block. Incremental compilation is useless on CI runners (each run gets a fresh image; per-run incremental cache is never reused) but still writes 3-5 GB of `target/*/incremental/**` — pure waste. No correctness impact.
2. **`CARGO_PROFILE_DEV_DEBUG=line-tables-only`** in the same `env:` block. Reduces debug-symbol size by ~80% (debug symbols dominate dev/test target sizes for large workspaces). Keeps file:line info for backtraces + panic messages; drops type-debug-info bulk test runtime never reads.
3. **`Free disk space (Linux runners only)`** composite step (existing local action at `.github/actions/free-disk-space`) on every Rust-compile-doing workflow — it frees ~25-30 GB of unused tooling. Audit annually; some workflows historically miss it (PR #1313 added it to wasm-conformance.yml).

**Apply at workflow-env scope** so every matrix leg inherits — macOS legs are no-ops there (~40 GB free) so no special cases needed. **mold linker is the escalation path** if the env-var approach is insufficient (the heavier fix; install via composite action; affects all linking) — but it's NOT the primary fix for the disk-OOM class.

**Recurrence-threshold rule:** 2× consecutive identical-pattern failures on the SAME runner class = recurring infra. Surface + fix at the workflow level; do NOT rerun a 3rd time hoping for transient flake. Validated by PR #1312's 3 consecutive failures + #1313's targeted fix.

**Composes with:** `feedback_strategy_c_batch_mandatory_not_optional` (each unforced sequential CI rerun = ~35 min wall-clock; the recurring-infra threshold reaches it sooner) + `feedback_arm_schedulewakeup_on_ci_wait` (when arming wakeup on a CI cycle that may hit recurring-infra failure, plan the +1 cycle wait with the appropriate `ScheduleWakeup` delay).

---

## §3.5p — Future signed-data designs MUST 3-layer-decompose (Ben-codified 2026-05-26 from L12 + cryptographer-review-of-bird-of-prey-vs-lamps + Inv-15 mint)

**Origin**: surfaced by L12 adversarial critic (combiner-soundness cryptographer) + load-bearing finding from senior cryptographer review of LAMPS Composite ML-DSA vs Bird-of-Prey for v1-beta default. Ratified by Ben "all yes across the board" 2026-05-26 alongside the Inv-15 mint + Compromise #31 + CLAUDE.md baked-in #5 sharpening.

**RULE**: Every NEW signed-data design proposal — whether a new wire envelope, new revocation pattern, new audit-trail surface, new dedupe-by-identifier scheme, new attestation envelope shape, or any other surface where signature bytes and identity intersect — MUST explicitly answer in its design doc:

1. **Identity decomposition**: what canonical bytes uniquely name "this thing"? MUST be either (a) a payload-CID (over canonical bytes that EXCLUDE the signature) OR (b) a semantic tuple (e.g. `(issuer, subject, cap, audience, validity)`). Designs where identifier = sig-bundle-CID are REJECTED on Inv-15 grounds (see `docs/INVARIANT-COVERAGE.md` "Inv-15 Phase-4-Meta-Core mint + 3-layer decomposition").
2. **Authentication decomposition**: which sig codepoint? MUST go through the codepoint-dispatch agility seam (CLAUDE.md baked-in #5). Designs that hardcode a specific algorithm at the call site are REJECTED.
3. **Revocation decomposition**: when this thing is revoked, what KEY does the revocation list use? MUST be (a) payload-CID OR (b) semantic tuple. NEVER sig-bundle-CID — the SUF-CMA gap at the construction layer (Compromise #31; LAMPS Composite ML-DSA is EUF-CMA-only per §9.2.2) admits a malleability bypass against any sig-CID-keyed revocation, so Inv-15 forbids the pattern.

**Enforcement seam**: cite-drift-detector `LoadBearingSigBundleCidPattern` scanner (planned at G-CORE-PQ-WIRE-1; flags functions matching `*_by_*_cid` whose parameter sources include signature bytes). Until the scanner ships, the discipline is enforced via design-review at PR-time: any new signature-touching surface MUST come with a design-doc Inv-15-compliance statement.

**Composes with**: `feedback_extra_reflection_pass_for_elegant_permanent_shape` (the 3-layer decomposition IS the elegant-permanent-shape the cryptographer's extra-reflection-pass surfaced; one architectural property closes the L12 SUF-CMA hazard across all current AND future signature surfaces simultaneously — strictly less code than a per-surface mitigation; forward-class-of-bug closure) + `feedback_surface_arch_decisions_under_auth` (any new signed-data design that proposes deviating from the 3-layer decomposition must SURFACE to Ben with reasoning, NOT silently ship; per the "real architectural forks" definition).

**Failure-mode-if-violated**: a signature surface that keys identity / revocation off sig-bundle-CID admits the EUF-only malleability bypass — attacker holding valid `(payload, sig)` mints `(payload, sig')` with a different CID + bypasses any CID-keyed system (revocation lists; dedupe; audit-uniqueness). The hazard is architectural-application-layer, NOT algorithm-layer; algorithm choice (LAMPS / Bird-of-Prey / draft-prabel) doesn't fix it; only the invariant does. See Compromise #31 in `docs/SECURITY-POSTURE.md` for the full attack-surface enumeration + closure mechanism.

**Cross-references**: Inv-15 in `docs/INVARIANT-COVERAGE.md`; Compromise #31 in `docs/SECURITY-POSTURE.md`; CLAUDE.md baked-in #5 Phase-4-Meta-Core 2026-05-26 sharpening; `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` (origin finding); `.addl/phase-4-meta/critic-lens-l12-combiner-soundness.json` (initial L12 discovery); `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-26.md` LATE-AFTERNOON #1 ADDENDUM (ratification record).

---

## §3.5q — IETF-vocabulary-precision when commenting on standards-body work (Ben-codified 2026-05-26 from L9 LAMPS-WG-participant critic finding)

**Origin**: L9 LAMPS-WG-participant critic surfaced that Benten's strategy-plan + draft comments used folksy terminology where IETF WG vocabulary is load-bearing for credibility. Codified 2026-05-26 as standing posting discipline alongside Inv-15 + Compromise #31.

**RULE**: When Benten comments on IETF / IRTF / W3C specs / drafts / mailing-list discussions / adoption calls / WG-LCs / IANA registrations, use precise WG-vocabulary throughout:

- **"WG document" vs "individual submission"** — `draft-ietf-*` is a WG document; `draft-<author>-*` is individual; the distinction is load-bearing (datatracker labels individual drafts "not endorsed by the IETF and has no formal standing in the IETF standards process"). Saying "draft-prabel-cfrg-suf-hybrid-sigs is an IETF draft" when it's an individual submission marks the speaker as not-WG-fluent.
- **"Adoption call" vs "WG Last Call (WGLC)"** vs "IETF Last Call (IETF-LC)" — three distinct WG-process moments; "WG-LC" is later in process than "adoption call."
- **"AUTH48"** is the specific RFC-Editor-process window where authors review the to-be-published RFC text; commenting on an adopter report DURING AUTH48 is a WG-process violation.
- **"RFC publication" vs "RFC-Editor queue ('In Progress' / 'Blocked')"** — RFC-Editor queue means submitted-to-RFC-Editor; can be Weeks-to-months from publication; "publication" is the formal moment.
- **IANA registry semantics** — "early allocation" (provisional, can be reclaimed) vs "permanent allocation" (no longer reversible); cite the actual registry + date.
- **Document state vocabulary** — "Internet-Draft (I-D)" vs "Internet Standard" vs "Proposed Standard" vs "Informational" — different document types with different process implications.

**Why it matters**: WG participants and adjacent-WG-fluent readers (the audience for Benten's standards-body comments) infer credibility partly from vocabulary precision. Folksy terminology marks Benten as adjacent-amateur — undermines the very adopter-credibility-signal the comment is meant to establish.

**Enforcement seam**: every standards-body comment Benten drafts goes through a pre-post pass against this rule. Specifically: every draft in [`.addl/phase-4-meta/f2-f7-comment-drafts.md`](.addl/phase-4-meta/f2-f7-comment-drafts.md) + [`.addl/phase-4-meta/f3-jose-comment-draft.md`](.addl/phase-4-meta/f3-jose-comment-draft.md) + [`.addl/phase-4-meta/new-comment-drafts.md`](.addl/phase-4-meta/new-comment-drafts.md) gets this lint at review time.

**Composes with**: §3.5h pre-push gate (cite-anchor verification); planning-agent-anticipation-audit pim-N candidate (L9's miss of F4a vehicle was related — domain-expert anticipation requires domain-expert vocabulary).

---

## §3.5r — Industry-folklore-vs-spec-text distinction (Ben-codified 2026-05-26 from L3 + L8 cross-critic finding)

**Origin**: L3 (iroh-maintainer) + L8 (Signal-protocol-veteran) both surfaced that Benten's strategy-plan reasoned from INDUSTRY FOLKLORE about why specific systems made specific design choices, when the actual primary-source text (system blog post, spec rationale section, etc.) said something different. L3: iroh's PQ blog reasons (a) no-HNDL-for-sigs + (b) no-industry-consensus — NOT the "ephemeral session vs persistent artifact" framing Benten's draft attributed to them. L8: PQXDH §4.8 explicitly rejected PQ identity sigs on DENIABILITY grounds — NOT the wire-size/pre-keys-signed folklore Benten assumed. Codified 2026-05-26 as standing research-discipline.

**RULE**: When Benten's framing or argument depends on attributing a specific reasoning to a specific external system ("system X chose Y because Z"), the orchestrator MUST verify Z against PRIMARY SOURCE TEXT (the system's own blog post / spec section / commit message / WG mailing-list message) BEFORE incorporating into Benten's framing.

**Why this matters**: paraphrased / from-memory / lore-based attributions are systematically wrong in ~50% of cases (L3 + L8 = 2 of the first 8 adversarial-critic returns surfaced this exact failure mode). Quoting primary-source text is the load-bearing discipline; "I read about this somewhere" is NOT verification.

**Specific patterns to watch**:
- Any sentence of the form "X chose Y because Z" where Z is paraphrased rather than quoted
- Any claim about why a P2P / messaging / standards-body system made a design choice
- Any "everyone knows..." framing about another system's design rationale
- Any blog/spec-text attribution where Benten's framing depends on the attributed reasoning being correct

**Enforcement seam**: Position B blog revision agent + Shape 5 iroh-outreach prep agent are both briefed with this rule explicitly (per `feedback_review_finding_ground_truth_verify` discipline). Future agent briefs that involve drafting public-facing technical content about other systems MUST inherit this discipline.

**Composes with**: §3.5n review-finding ground-truth-verify (this rule extends that discipline from "verify review findings against code" to "verify external-system reasoning attributions against primary sources"); `feedback_review_finding_ground_truth_verify` foundational memory; §3.5q IETF-vocabulary-precision (same family of discipline — get the externalities right before adopting their framing into Benten's work).

---

## §3.5s — Cross-ecosystem-identifier-as-content discipline (Ben-codified 2026-05-26 from cross-stack-naming discussion + JOSE-archaeology agent §7.2 X-Wing-vs-HPKE-PQ-disambiguation finding)

**Origin**: surfaced during the 15-critic synthesis + L12 cryptographer-review-of-bird-of-prey-vs-lamps + JOSE-archaeology agent return. Ben ratification 2026-05-26: *"use the already-converging cross-ecosystem identifiers (LAMPS OID 1.3.6.1.5.5.7.6.48 + JOSE/COSE alg-name MLDSA65-Ed25519 at COSE alg -55 + multicodec container approach) AS CONTENT inside our envelopes makes a lot of sense."*

**RULE**: Wire-format envelopes that cross ecosystem boundaries (DID URIs, X.509-style certs, JOSE/JWE/JWS envelopes, COSE envelopes, JWK fields, multicodec containers) MUST carry the **cross-ecosystem identifier AS CONTENT** inside the envelope where consumers of that ecosystem will recognize it. The Benten-internal `SigCodepoint` / `CipherSuiteCodepoint` / etc. dispatch numbers stay **hot-path-dispatch-only** and MUST NEVER appear at ecosystem-boundary surfaces in lieu of the cross-ecosystem identifier.

**Concrete mappings (current)**:
- **PKIX / X.509 / CMS** — algorithm-identifier carried as **LAMPS OID** (e.g. `id-MLDSA65-Ed25519-SHA512` = `1.3.6.1.5.5.7.6.48`)
- **JOSE / JWE / JWS / JWK** — algorithm-identifier carried as **JOSE/COSE algorithm name** (e.g. `MLDSA65-Ed25519` per `draft-skokan-jose-hpke-pq-pqt-05` / COSE alg `-55`)
- **Multicodec** — public keys carried via **container-form codepoints** (`cose-key` `0x42` / `jwk` `0x44` per multicodec PRs #400/#403) with the LAMPS-aligned algorithm naming **inside the container**, not via per-algorithm multicodec entries
- **did:jwk URIs** — algorithm naming in the JWK `alg` field references the JOSE/COSE registry
- **Benten internal Varsig + AEAD envelope dispatch** — `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` / `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768 = 0x647A` stay wire-format-internal-only

**Why this matters**: each ecosystem (PKIX/JOSE/COSE/multicodec/Benten-internal) uses its own dispatch shape (ASN.1 OID vs small-integer-alg-ID vs varint vs 2-byte LE) and there is **no shared registry across them**. Trying to make Benten's internal dispatch codepoint AS the cross-ecosystem identifier would either (a) require external registries to adopt Benten's codepoint (won't happen) or (b) leave Benten's outputs unrecognizable to external consumers. The right pattern: **identify the construction by its cross-ecosystem identifier as content, dispatch internally by codepoint for speed**. Different concerns at different layers.

**Failure mode if violated**: writing a Benten-internal SigCodepoint `0x0001` into a `did:jwk` URI or JWE algorithm field instead of the JOSE-registry name `MLDSA65-Ed25519` would make Benten outputs unrecognizable to JOSE-stack consumers (JWS libraries, WebCrypto, JWKS endpoints) without Benten-private codepoint knowledge. Ecosystem-fragmentation hazard per L4 critic finding; META-message-of-small-team-fragmenting-ecosystem hazard per L1.

**Enforcement seam**:
- New wire-format envelope designs MUST explicitly answer: "what cross-ecosystem identifier does this envelope carry as content?" alongside "what internal dispatch codepoint resolves the construction internally?"
- Design-review at PR-time for any envelope-shape changes
- Future cite-drift-detector extension to flag Benten-private-codepoint references in ecosystem-boundary surface code (DID-URI rendering paths, JWE/JWS envelope construction, etc.)

**Future-additive direction**: when Benten exposes a JOSE-side encrypt-to-recipient surface (Phase-4-Meta+ scope; see G-CORE-PQ-WIRE wave + the F-full architectural direction), the HPKE-PQ-PQT codepoints from `draft-skokan-jose-hpke-pq-pqt` are exactly the cross-ecosystem identifiers that envelope would carry as content. Benten's internal `CipherSuiteCodepoint` continues to be the hot-path dispatch number; the JOSE envelope contents reference the JOSE registry.

**Composes with**:
- §3.5p (3-layer decomposition for signed-data) — identity layer + authentication layer + revocation layer; cross-ecosystem identifiers live at the authentication layer when crossing ecosystem boundaries
- §3.5q (IETF-vocabulary-precision) — cross-ecosystem identifier naming should mirror the WG-precise vocabulary (e.g. "MLDSA65-Ed25519" not "X-Wing-style" when the WG-adopted name is the former)
- §3.5g cross-language rule-mirror — internal SigCodepoint dispatch must be mirrored TS/Rust; cross-ecosystem identifiers are content not dispatch so don't need the same mirror discipline
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` — the cleanest architectural shape separates DISPATCH (internal codepoints) from IDENTIFICATION (cross-ecosystem content) at each ecosystem-boundary surface

**Concrete current-state corrective triggered by this rule** (cryptographer-review finding 2026-05-26): `crates/benten-crypto-suite/src/cipher_suite.rs` references our X-Wing-style combiner as "X-Wing" but (a) the actual construction is NOT real X-Wing (mismatch on hash function + label) AND (b) the IRTF CFRG Research-Group-ADOPTED name is `MLKEM768-X25519` per `draft-irtf-cfrg-concrete-hybrid-kems-03`. The §3.5s discipline says: rename references in cross-ecosystem-boundary surfaces (DID URIs, did:jwk, multicodec metadata, public docs) to `MLKEM768-X25519`; the internal codepoint `0x647A` stays as-is (matches the IETF-reservation). Pre-v1-beta-tag-must-fix per the cryptographer-review independent of any other architectural decisions.
