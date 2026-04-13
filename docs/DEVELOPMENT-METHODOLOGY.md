# Development Methodology — Agent-Driven Development Lifecycle (ADDL)

## Overview

All development follows a structured pipeline designed for AI agent-driven development with human oversight. The methodology was refined across 10+ phases of Thrum development and catches architectural issues before they become expensive code.

## The Pipeline

```
PRE-WORK
  1. Planning agent → produces implementation plan
  2. 2+ critic agents → review the plan (architecture + correctness minimum)
  3. Triage → coordinator handles EVERY finding (fix / defer / disagree)
  4. Revised plan

FULL ADDL (for feature phases)
  R1:  5 spec agents (architecture, correctness, security, DX, philosophy)
  R1 triage: fix everything fixable, defer to specific docs, explain disagreements

  R2:  1 test plan synthesis agent → maps every API to test requirements

  R3:  5 test writing agents → produce actual test files (TDD — tests before code)

  R4:  2 test review agents (quality + coverage) → fix issues in test files

  R5:  Implementation groups (4-8 groups depending on scope)
       Each group:
         - N parallel agents (per the plan — do NOT combine agents)
         - Commit
         - Full test suite on real infrastructure
         - Mini-review (1 correctness agent)
         - Fix ALL findings
         - Commit

  R4b: 2 post-implementation test review agents (tests against real code)

  R6:  14-agent quality council (or targeted subset)
       Triage EVERY finding — no finding floats
```

## Non-Negotiable Process Rules

These were learned through corrections across multiple phases. Violating them wastes time.

1. **After EVERY review:** Fix ALL fixable items, write ALL deferrals to docs with phase targets, explain ALL disagreements. Never skip by severity.

2. **When triage says "fix now" — write the actual code.** Don't say "noted for implementation."

3. **Stay in the current phase.** Flag scope detours explicitly and let the user decide.

4. **Mini-review after EVERY implementation group.** One correctness agent, properly briefed with files changed. Fix all findings before next group.

5. **No deprecated aliases or backward-compat shims.** Fresh project. Delete don't comment.

6. **Doc updates in the LAST implementation group,** not afterthought.

7. **Explain in plain English first,** technical details second.

8. **Don't combine agents.** Each agent gets its own prompt with its own scope. The plan specifies agent counts for a reason.

9. **Full quality council for feature phases.** Only pure correctness work gets a reduced council.

10. **The user is a co-architect.** Present options, not just results. Their questions reshape the plan.

11. **Run the full test suite after every commit.** Not just the packages you changed. Indirect breakage is real.

12. **Fix pre-existing issues** found during review if they're under ~15 minutes. Don't leave known bugs.

## For Rust Development Specifically

The ADDL pipeline adapts for Rust:

- **R3 (test writing):** Rust tests with `#[test]`, property-based tests with `proptest`, benchmarks with `criterion`/`divan`
- **R5 (implementation):** `cargo test` after every group, `cargo clippy` for linting, `cargo bench` for performance
- **Mini-reviews:** Focus on unsafe code, ownership issues, lifetime problems, and performance regressions
- **R6 quality council:** Include a Rust-specific best-practices agent
- **Benchmarks are part of the test suite.** Performance regressions are bugs.

## Spike Pattern

Before committing to a major implementation, run a spike:
1. Write the minimal code to test the critical assumption
2. Benchmark it
3. Have critics review it
4. THEN decide whether to proceed

This pattern caught: the AGE MERGE issue (AGE doesn't support MERGE), the PGlite single-threaded limitation, the Grafeo indexing gap, and many more. Spikes are cheap. Bad architectural assumptions are expensive.

## What "Triage" Means

For every finding from every review, the coordinator must do ONE of:
- **Fix now:** Write the actual fix, verify it works, commit
- **Defer:** Write the deferral to a specific doc with a phase target and rationale
- **Disagree:** Explain why the finding is wrong or not applicable

"Noted" is not an acceptable response. "Will address later" without a doc entry is not acceptable. Every finding gets a disposition.
