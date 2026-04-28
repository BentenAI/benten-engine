# Contributing

Thanks for your interest. This repo is under active development; contributions are welcome but expect some churn, especially during phase transitions.

## Before you start

- Read [`docs/HOW-IT-WORKS.md`](docs/HOW-IT-WORKS.md) for orientation.
- Read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the crate layout and invariant set.
- For large changes, open an issue first. The engine's architecture is still settling at its edges; coordination avoids wasted work.

## Prerequisites

- Rust 2024 edition. MSRV is **1.89** (bounded by `redb` 4's floor); dev version is **1.94+** (pinned to `stable` in `rust-toolchain.toml`, which auto-installs on first cargo invocation).
- Node.js 22+ and npm (for the TypeScript bindings and integration tests).
- `cargo-nextest`: `cargo install cargo-nextest`.
- **Optional — only when regenerating SANDBOX `.wasm` test fixtures:** `wabt` (`brew install wabt`, or your distro's package). The committed `.wasm` bytes under `crates/benten-eval/tests/fixtures/sandbox/` are the canonical CI input — `scripts/build_wasm.sh` (dev-only) re-derives them from sibling `.wat` sources and a CI drift detector enforces equality. You only need `wabt` installed if you're editing a `.wat` source.

## Setup

```sh
git clone https://github.com/BentenAI/benten-engine.git
cd benten-engine
cargo build --workspace
cargo nextest run --workspace
```

The test suite is designed for `cargo nextest`. See "Why nextest is load-bearing" below.

## Before you push

Run locally:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
```

CI runs the same checks plus cross-target determinism, supply-chain hygiene, invariant drift detection, and doc-build with warnings-as-errors. The determinism workflow computes the canonical fixture CID on Linux/macOS/Windows; any drift is a merge blocker.

## Commits

- Conventional commits: `<type>(<scope>): <summary>`. Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`, `build`.
- One logical change per commit. Commit messages explain **why**, not just what.
- Do not amend merged commits or force-push to `main`.

## Why nextest is load-bearing

The test suite is designed to run under `cargo nextest run`. That choice is load-bearing for correctness — not just speed.

The version-chain substrate in `benten-core/src/lib.rs` keeps two process-global states: `ANCHOR_COUNTER` (an `AtomicU64` issuing unique anchor IDs) and `U64_CHAINS` (a `spin::Mutex<BTreeMap<...>>` mapping anchor IDs to CID chains). Phase 1 deliberately scopes these to the process; a Phase-2 `AnchorStore` trait will push them behind a handle. The test suite writes to them from ~50 call sites across five integration-test files.

`cargo nextest run` runs each integration-test binary in its own process, so cross-binary interactions are naturally isolated. Plain `cargo test` (without `--test-threads=1`) runs all tests in a single process and can interleave writes across tests in the same binary; assertions that depend on a specific anchor ID or chain length drift under that runner.

`.config/nextest.toml` pins `retries = 0` in both default and CI profiles. The invariant is zero hidden flakiness — a test that fails once is investigated, not retried.

If nextest is unavailable: `cargo test --workspace --jobs 1 -- --test-threads=1` is the second-best approximation. It serializes tests inside each binary, avoiding the interleaving that perturbs the globals above.

## Supply chain

The workspace enforces supply-chain hygiene via `cargo-deny` and `.github/workflows/supply-chain.yml`:

- **License allowlist.** MIT, Apache-2.0 (with LLVM-exception), BSD-2, BSD-3, CC0, ISC, Unicode-DFS-2016, Unicode-3.0, Zlib. Anything else fails the build.
- **RUSTSEC advisories.** `cargo-deny check` fails on any advisory against the current dependency tree. A weekly scheduled job re-runs `cargo audit` and opens an issue on new advisories.
- **`cargo build --locked`.** CI fails if resolution would change. Forgot to commit a `cargo update`? CI catches it.
- **Git patches.** Every `[patch.crates-io]` git source must appear in `deny.toml`'s `allow-git` list.

If a dep is yanked mid-development, the protocol is: open a `supply-chain` + `needs-triage` issue within 24h, pin by commit SHA (fork to `BentenAI/<crate>` if the upstream needs work), file an upstream fix, block the PR until the tree is green, and remove the patch once the upstream ships a fix.

## Style

- **Rust:** `rustfmt` and `clippy` with the configs at the repo root.
- **TypeScript:** 2-space indent, double quotes, trailing commas. `.editorconfig` enforces.
- **Markdown:** 2-space indent.

## Naming conventions

- **`UPPERCASE-KEBAB.md`** — canonical specifications and contracts (README, CONTRIBUTING, SECURITY, ARCHITECTURE, HOW-IT-WORKS, ERROR-CATALOG, GLOSSARY, QUICKSTART).
- **Rust:** `snake_case` for modules and functions, `UpperCamelCase` for types.
- **TypeScript:** `camelCase` for values and functions, `UpperCamelCase` for types and classes.

## Reporting issues

Use GitHub issues. Include:

- Rust version (`rustc --version`) and platform
- Minimal reproduction if behavioral
- Relevant error codes (see [`docs/ERROR-CATALOG.md`](docs/ERROR-CATALOG.md))

For security issues, please follow [`SECURITY.md`](SECURITY.md) instead of filing a public issue.

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0, matching the repository.
