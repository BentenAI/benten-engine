# Benten ADDL pipeline — finalized workflow library

The ADDL pipeline as **executable methodology**: each stage is a reusable, hardened Claude Code Workflow that
*structurally encodes* our rules so they can't be skipped under turn-budget pressure (see memory
[[feedback_workflows_as_executable_methodology]]). Finalized 2026-06-03 after the R4-fix shakedown validated the
integration mechanics.

## Stage → workflow map

| ADDL stage | Workflow | Shape |
|---|---|---|
| **R1** plan critic-council | `addl-review-council.js` (tier R1) | N lenses → structure → adversarial-verify → completeness-critic → converge. Iterate across invocations (R1.1 → fix plan → R1.2) until 0 BLK/MAJ. |
| **R2** test-landscape | `addl-r2-test-landscape.js` | N discovery dims (each blind) → completeness-critic (hunts what all missed) → synthesis (family catalog + coverage matrix + canary-first R3 slicing + freeze-gating priorities). |
| **R3** red-phase test-writers | `addl-r3-test-writers.js` | Canary + GATE (fan-out fires only on PASS) → fan-out waves → per-wave substantive+seam mini-reviews → coverage-verify/converge. |
| **R4 / R4b** test-corpus review | `addl-review-council.js` (tier R4) | Same council as R1, lens-set = test-quality + wire-freeze + crypto + invariant + coverage + ruling-fidelity. Iterate to convergence. |
| **R4-fix / R6-fix / any fix round** | `converging-fix-loop.js` | Autonomous review→fix→re-review→loop until no new BLK/MAJ; fixes ALL non-disagreed findings (HARD-RULE-12); decision-log = audit-not-gate. |
| **R5** implementation | `addl-r5-impl-to-green.js` | Canary-first impl waves, each looping impl→un-ignore→test→fix until GREEN; gate fan-out on canary; strategy-C integrate + full-suite. |
| **R6** phase-close council | `addl-review-council.js` (tier R6) | Full council every round (Q5); iterate until 0 substantive; pattern-induction. Wrap with `converging-fix-loop.js` for the fix rounds. |
| *(shared)* | `workflow-common.js` | Canonical preamble + codification map — INLINE it at authoring/invoke time (the sandbox has no imports). |

## Two iteration models

- **Across-invocation (orchestrator in the loop):** run `addl-review-council` → orchestrator triages + fixes the
  artifact → run it again on the edited artifact → repeat. The orchestrator gates each fix. Use for review tiers
  where fixes are high-judgment and you want eyes on each round (R1, R4, R6).
- **In-workflow autonomous (`converging-fix-loop`):** the loop reviews → fixes ALL non-disagreed findings →
  integrates → compile-gates → re-reviews → loops. `args.mode`:
  - `'checkpoint'` (default, SAFE): emits fixes to `/tmp` + returns a fix-list; the orchestrator integrates +
    compile-gates + re-invokes. This is the validated shakedown shape.
  - `'autonomous'`: an in-workflow integrator does its own git writes + cargo. ⚠️ **WATCH the first few runs**
    (`/workflows`) — this is the fragile part the shakedown kept orchestrator-side; recoverability is audit-not-gate
    (decision-log + downstream gates + Ben-gated freeze), not unrecoverable.

## Invocation

`Workflow({ name: '<workflow>', args: { cfg: {...}, lenses|dimensions|waves: [...] } })`

The machinery is generic; the phase-specific config (anchors, lens/dimension/wave sets, inline canon) goes in
`args`. Copy the matching `f-full-*.js` concrete instance as a worked example of the args shape. **Inline the
project canon** (codepoint table, rulings, invariants) into `args.cfg.canon` — fresh-worktree agents can't see
gitignored `.addl`/`CLAUDE.md` except via `git show`.

## Hard-won operational disciplines (baked into every workflow)

- **Schema-free prose for review/discovery agents** (forced StructuredOutput silently fails); **schema for
  consolidators** (single agent, reliable).
- **Batch parallel into sub-waves of ≤4** (a 6-8 burst trips the transient server-side rate-limit).
- **Refuse-on-partial-panel** — a rate-limited silence is not an APPROVE (verify-by-artifact).
- **Fix agents emit full file content to `/tmp`** (read-only-to-repo); a **single writer integrates** (sidesteps
  worktree-escape AND notification-truncation). Impl/test-writer agents that must write code use
  `isolation:'worktree'` + the loud ABSOLUTE-PATH-FORBIDDEN escape contract + commit-before-return.
- **Compile-gate via BASH** (zsh doesn't word-split unquoted `$features`), detect green/red by **error-line
  presence** (NOT the masked EXIT after `cargo | tail`), per-crate feature flags, beware Rust-2024 reserved
  keywords (`gen`/etc).
- **Golden-hex fixes**: compute bytes once via a throwaway `/tmp` script + freeze a literal (M-20: R5 confirms-or-
  updates vs the real encoder).
- **Opus-only** for council/agent work; `budget.remaining()` guard + `MAX_ROUNDS` backstop; resume returns CACHED
  (fresh run to re-run failed agents).
- **Harvest each run's auto-saved script** (session-dir `…/workflows/scripts/<name>-<runId>.js` is session-scoped)
  into this durable library at launch; record Task ID + Run ID + script path in the handoff for compact-survival.

## Reuse mechanics

- **Invoke-by-name:** `.claude/workflows/<name>.js` → `Workflow({name})` (gitignored here, functional).
- **Durable/tracked:** this dir (`.addl/phase-4-meta/workflows/`, force-added).
- **Not codified (human-gated):** the tag/freeze decision; "is this architectural direction right"; the first
  autonomous-integration run.

## Concrete F-full instances (worked examples / historical record)

`f-full-r1-critic-council.js` · `f-full-r1-2b-convergence.js` · `f-full-r2-test-landscape.js` ·
`f-full-r3-test-writers.js` · `f-full-r4-review.js` · `f-full-r4-2-convergence.js` — the as-run scripts behind the
F-full Phase-4-Meta-Core pipeline (R1 CONVERGED → R2 → R3 CONVERGED → R4 NOT-CONVERGED → R4-fix shakedown → R4.2).
