# Development Methodology — Agent-Driven Development Lifecycle (ADDL)

## Overview

All development follows a structured pipeline designed for AI agent-driven development with human oversight. The methodology was refined across 10+ phases of Thrum development and catches architectural issues before they become expensive code.

**Infrastructure:** The pipeline runs on the Benten Engine's custom agent + skill ecosystem at `.claude/` — 44 agents (34 community-vetted from VoltAgent + 10 Benten-specific) and 30 skills (8 Benten-specific + 22 from Trail of Bits). See `.claude/README.md` for the full inventory and `docs/GLOSSARY.md` for terms.

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

**Tooling:** Use the `/spike <name>` skill to scaffold a spike under the Phase 1 crates. The spike lives INSIDE the real crates (not a separate `spikes/` directory) because it is the minimal first implementation of the real code.

---

## ADDL Stage → Agent Mapping (added 2026-04-14)

Every ADDL stage dispatches specific agents from `.claude/agents/`. This table is the canonical mapping. When a required agent doesn't exist yet, it's flagged as "to-create" — create just-in-time during the phase that first needs it.

### R1 — Spec Review (5 agents, agent-team mode for peer-debate)

| Lens | Agent | Source |
|------|-------|--------|
| Architecture | `architect-reviewer` | VoltAgent |
| Correctness | `code-reviewer` | VoltAgent |
| Security | `security-auditor` | VoltAgent |
| DX / ergonomics | `dx-optimizer` | VoltAgent |
| Engine philosophy / thin-engine | **`benten-engine-philosophy`** (to-create) | Benten custom — model on the Thrum engine-philosophy but Benten-specific |

### R2 — Test Plan Synthesis (1 agent, lead session)

| Role | Agent | Source |
|------|-------|--------|
| Maps every public API to test requirements | **`benten-test-landscape-analyst`** (to-create) | Benten custom — Rust-aware, covers `#[test]` + proptest + criterion + miri coverage |

### R3 — Test Writing (5 agents, parallel subagents, TDD contract)

| Scope | Agent | Source |
|-------|-------|--------|
| Unit tests | **`rust-test-writer-unit`** (to-create) | Benten custom — Rust `#[test]` + proptest |
| Edge cases | **`rust-test-writer-edge-cases`** (to-create) | Benten custom — null inputs, overflow, concurrency |
| Security boundaries | **`rust-test-writer-security`** (to-create) | Benten custom — auth, injection, trust tier |
| Performance constraints | **`rust-test-writer-performance`** (to-create) | Benten custom — criterion benchmarks with targets |
| Integration tests | `qa-expert` | VoltAgent (may need briefing for Rust specifics) |

Supplement with Trail of Bits `/property-based-testing` skill for proptest patterns.

### R4 — Test Review (2 agents, subagents)

| Role | Agent | Source |
|------|-------|--------|
| Test quality + patterns | **`rust-test-reviewer`** (to-create) | Benten custom — reviews test files before code is written |
| Coverage analysis | `qa-expert` or **`rust-test-coverage`** (to-create) | VoltAgent or Benten custom |

### R5 — Implementation Groups (parallel `implementation-developer` agents per group)

| Role | Agent | Source |
|------|-------|--------|
| Implementation | **`rust-implementation-developer`** (to-create) | Benten custom — TDD: make R3 tests pass |
| Cargo workflow | `cargo-runner` | Benten custom |
| General Rust | `rust-engineer` | VoltAgent |
| Crate-specific guardian (dispatch based on crate) | One of the 9 Benten custom reviewers | Benten custom |
| Mini-review (after each group commits) | `code-reviewer` + crate-specific guardian | Mixed |

### R4b — Post-Implementation Test Review (2 agents, subagents)

Same as R4 but with code present — tests validated against real implementation.

### R6 — Quality Council (parallel subagents, NOT a team)

The full 14-agent council for Benten:

| # | Lens | Agent | Notes |
|---|------|-------|-------|
| 1 | Architecture | `architect-reviewer` | |
| 2 | Code quality | `code-reviewer` | |
| 3 | Security | `security-auditor` | |
| 4 | Performance | `performance-engineer` | |
| 5 | Testing | `qa-expert` | |
| 6 | Test automation | `test-automator` | |
| 7 | Resilience | `chaos-engineer` | |
| 8 | DX | `dx-optimizer` | |
| 9 | Error handling | `error-detective` | |
| 10 | Refactoring opportunities | `refactoring-specialist` | |
| 11 | Benten invariants | `benten-core-guardian` OR one of the 9 Benten customs, depending on change scope | |
| 12 | 12-primitive vocabulary | `operation-primitive-linter` | |
| 13 | Code-as-graph correctness | `code-as-graph-reviewer` | |
| 14 | Best practices 2026 | **`best-practices-2026`** (to-create or restore) | Previously a generic user-level agent; may restore or rewrite Benten-specific |

For phases touching specific domains, swap in the relevant Benten custom agent (IVM → `ivm-algorithm-b-reviewer`, caps → `ucan-capability-auditor`, CRDT → `crdt-correctness-reviewer`, etc.).

### Missing Agents (Create Just-In-Time)

The following agents are flagged "to-create" and should be written when the phase that first needs them begins:

- `benten-engine-philosophy` — Benten-specific thin-engine reviewer
- `benten-test-landscape-analyst` — Rust test landscape analysis
- `rust-test-writer-unit` — Rust unit + proptest writer
- `rust-test-writer-edge-cases` — Rust edge-case writer
- `rust-test-writer-security` — Rust security test writer
- `rust-test-writer-performance` — Rust criterion benchmark writer
- `rust-test-reviewer` — Rust test quality reviewer
- `rust-test-coverage` — Rust coverage analysis
- `rust-implementation-developer` — TDD-based Rust implementer (from test suite)
- `best-practices-2026` (Benten-flavored) — 2026 Rust + ecosystem best practices reviewer

Do not create them speculatively. Create them when Phase 1's R3 stage (first test-writing) begins; they are Phase 1 pre-work.

---

## Orchestration Patterns (added 2026-04-14)

The 2026-04-14 multi-agent orchestration research identified specific patterns that match ADDL's stages. Implementing ADDL correctly requires using the right pattern per stage.

### Pattern 1: Agent Team (Peer-Messaging)

**When:** Agents need to **debate** each other or challenge each others' conclusions. Most appropriate for adversarial or complementary-lens review.

**Example:** R1 spec review — 5 agents with distinct lenses (architecture, correctness, security, DX, engine-philosophy) peer-debate the proposed spec.

**How:** Use the agent-teams feature (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in `~/.claude/settings.json`). Teams are managed by Claude Code at runtime; team config files are auto-generated at `~/.claude/teams/{name}/`. DO NOT hand-author team config files — they are overwritten on state update.

### Pattern 2: Parallel Subagents (Independent Reports)

**When:** Agents produce independent findings that the orchestrator merges. No peer-messaging needed; saves tokens.

**Example:** R6 quality council — 14 agents each reviewing the implementation from their own lens, producing structured JSON findings. Peer messaging would be wasteful; the orchestrator does deterministic triage on the merged output.

**How:** Launch parallel subagents via the Agent tool with `run_in_background: true`. Each agent returns a JSON finding list; orchestrator merges by `location + claim`, dedupes, triages.

### Pattern 3: Structured JSON Output Contract

Every multi-agent review stage enforces this schema per agent:

```json
{
  "agent": "<name>",
  "stage": "r1|r4|r4b|r6|critic|review-crate",
  "findings": [
    {
      "severity": "critical|major|minor",
      "area": "...",
      "location": "file:line",
      "claim": "...",
      "evidence": "...",
      "fix": "..."
    }
  ],
  "verdict": "pass|revise|block"
}
```

The orchestrator:
1. Persists raw agent output to `.addl/phase-N/<stage>-<agent>.json` (keeps orchestrator context lean)
2. Merges findings across agents by `location + claim`
3. Dedupes
4. Renders a prioritized triage table
5. Does NOT ask an LLM to synthesize 14 raw outputs — too lossy

### Pattern 4: TDD Contract (R3 → R5)

R3 test-writing agents produce REAL test files, not test plans. R5 implementation agents write code that makes R3 tests pass. The `implementation-developer` agent system prompt enforces "The tests ARE the specification."

### Pattern 5: Mini-Review After Every Implementation Group

Non-negotiable per Rule 4. One correctness agent (typically `code-reviewer` or a crate-specific guardian) briefed with files changed. Fix all findings before next group. Commit only after fixes.

### Anti-Patterns (Observed and Confirmed)

- **Hand-editing team config.json** — overwritten on next state update
- **Lead-starts-coding** — explicit countermeasure: "Wait for teammates to complete their tasks before proceeding"
- **Two teammates editing the same file** — no auto-merge; partition by file/module ownership
- **Nested teams** — teammates cannot spawn their own teams. Only the lead can. Sequence N implementation groups from the lead.
- **Stuck tasks** — teammates sometimes fail to mark completed, blocking dependents. Use `TeammateIdle` hook to enforce.
- **Session resumption does NOT restore in-process teammates** — plan for respawn

---

## Skill vs Agent Decision Framework

| Use a **skill** (user-invocable `/cmd`) when... | Use an **agent** (Claude delegates) when... |
|---|---|
| User should trigger it explicitly | Claude should delegate autonomously based on context |
| Deterministic multi-step template | Open-ended investigation with judgment |
| Same prompt reused across sessions | Isolated context window needed |
| Output is an artifact (commit, file, scaffold) | Output is a report/opinion |
| Tool set is narrow and fixed | Tool set is broad or read-only |

**Skills can launch agents.** Example: `/review-crate benten-ivm` (skill) dispatches `ivm-algorithm-b-reviewer` + `rust-engineer` + `code-reviewer` (agents). They are layered, not competing.

---

## Model Policy

| Model | Agent types | Rationale |
|-------|-------------|-----------|
| **opus** (42 agents) | All judgment-heavy work: architects, reviewers, auditors, Benten domain specialists, research, orchestration | Quality over cost per "do it right, not fast" |
| **sonnet** (2 agents) | `cargo-runner`, `operation-primitive-linter` | Mechanical pattern-match work; no architectural judgment |
| **haiku** (0 agents) | None currently | No cheap-enough-to-justify tasks identified |

---

## Tool Restrictions by Role

| Role | Tools allowed | Why |
|------|---------------|-----|
| **Read-only auditor** (security-auditor, compliance-auditor) | `Read, Grep, Glob` | Cannot modify code |
| **Domain reviewer** (Benten custom reviewers) | `Read, Grep, Glob, Bash` | Can run tests/checks; cannot write code |
| **Mechanical worker** (cargo-runner) | `Bash, Read, Grep` | Reports output; no code changes |
| **Implementer** (rust-engineer, ai-engineer) | Full R/W/E/B/G/G | Can modify code |
| **Researcher** (competitive-analyst, etc.) | + `WebFetch, WebSearch` | External information retrieval |

---

## What "Triage" Means

For every finding from every review, the coordinator must do ONE of:
- **Fix now:** Write the actual fix, verify it works, commit
- **Defer:** Write the deferral to a specific doc with a phase target and rationale
- **Disagree:** Explain why the finding is wrong or not applicable

"Noted" is not an acceptable response. "Will address later" without a doc entry is not acceptable. Every finding gets a disposition.
