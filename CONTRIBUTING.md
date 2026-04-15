# Contributing to Benten Engine

First time reading the repo? Start with [`docs/VISION.md`](docs/VISION.md) and [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Prerequisites

- Rust 2024 edition. MSRV (minimum supported) is **1.85** (when edition 2024 stabilized); dev version is **1.94+** (pinned to `stable` channel in `rust-toolchain.toml`, which auto-installs the current stable on first cargo invocation). Install via `rustup`.
- Node.js 22+ and npm (for TypeScript bindings and IVM benchmark reproducibility).
- `cargo-nextest` for faster tests: `cargo install cargo-nextest`.

## Setup

```sh
git clone <repo>
cd benten-engine
# rust-toolchain.toml auto-installs the correct Rust version on first cargo command
cargo build --workspace
cargo nextest run --workspace
```

Phase 1 has not yet produced Rust crates — they will be created during the spike. Until then, `cargo` commands operate on an empty workspace.

## Development Philosophy

**"Do it right, not fast."** Quality over speed. Do not cut scope. Do not ship half-built features to hit timelines.

**Pre-work catches bugs before code.** Planning, dependency validation, critic reviews, and spikes catch architectural issues that would be expensive to fix later. Skipping pre-work has always cost more time than it saved.

**Every question is potentially plan-changing.** The project's direction has shifted several times based on good questions. Ask them. Challenge assumptions. Revisions are cheaper than wrong implementations.

## The ADDL Process

All substantive work follows Agent-Driven Development Lifecycle. Summary:

```
PRE-WORK
  Plan → 2+ critic reviews → triage every finding → revised plan

FULL ADDL (for feature phases)
  R1: 5 spec agents (architecture, correctness, security, DX, philosophy)
  R2: 1 test plan synthesis agent
  R3: 5 test writing agents (TDD: tests before code)
  R4: 2 test review agents
  R5: Implementation groups (parallel agents + commit + full tests + mini-review)
  R4b: 2 post-implementation test review agents
  R6: 14-agent quality council
```

Full details: [`docs/DEVELOPMENT-METHODOLOGY.md`](docs/DEVELOPMENT-METHODOLOGY.md).

## Non-Negotiable Process Rules

1. **After every review:** fix all fixable items, write deferrals to specific docs with phase targets, explain all disagreements. Never skip by severity.
2. **When triage says "fix now" — write the code.** Do not say "noted for implementation."
3. **Stay in the current phase.** Flag scope detours explicitly and let the user decide.
4. **Mini-review after every implementation group.** One correctness agent, properly briefed with files changed. Fix all findings before next group.
5. **No deprecated aliases or backward-compat shims.** Fresh project. Delete don't comment.
6. **Doc updates in the LAST implementation group,** not afterthought.
7. **Explain in plain English first,** technical details second.
8. **Don't combine agents.** Each agent gets its own prompt with its own scope.
9. **Full quality council for feature phases.** Only pure correctness work gets a reduced council.
10. **The user is a co-architect.** Present options, not just results.
11. **Run the full test suite after every commit.** Indirect breakage is real.
12. **Fix pre-existing issues** under ~15 minutes if found during review.

## Commits and PRs

- Write descriptive commit messages. Explain **why**, not just what.
- Keep commits atomic — one logical change per commit.
- Before committing, run:
  ```sh
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo nextest run --workspace
  ```
- CI runs the same checks plus `cargo doc` with warnings-as-errors. See `.github/workflows/ci.yml`.

## Style

- **Rust:** `rustfmt` and `clippy` with the configs at the repo root. Workspace lints are set in `Cargo.toml`; crates inherit.
- **TypeScript:** 2-space indent, double quotes, trailing commas. `.editorconfig` enforces.
- **Markdown:** 2-space indent, preserve trailing whitespace in docs (for intentional line breaks).

## Naming Conventions

- **`UPPERCASE-KEBAB.md`** — canonical specifications and contracts (VISION, ARCHITECTURE, ENGINE-SPEC, CLAUDE, README, CONTRIBUTING, GLOSSARY, DECISION-LOG).
- **`lowercase-kebab.md`** — research, exploration, critique, validation artifacts.
- **Rust:** `snake_case` for modules/functions, `UpperCamelCase` for types.
- **TypeScript:** `camelCase` for values/functions, `UpperCamelCase` for types/classes.

## Running Critics

Critics are AI agents with specific perspectives (architecture-purity, developer-experience, security-trust, composability-extensibility, etc.). They review specs or implementations and produce findings. Triage every finding:
- **Fix now:** write the fix, verify it, commit.
- **Defer:** write the deferral to a specific doc with a phase target.
- **Disagree:** explain why.

"Noted" is not an acceptable response.

## Reporting Issues

For the current pre-implementation phase: raise questions directly with Ben. Once code lands, use GitHub issues with the relevant phase label.

## Acknowledgments

Benten is a successor to the Thrum project (V3 TypeScript platform, 15 packages, 3,200+ tests). Many design decisions descend from that work. See [`docs/PROJECT-HISTORY.md`](docs/PROJECT-HISTORY.md).
