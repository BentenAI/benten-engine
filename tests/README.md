# tests/

Cross-crate integration tests and benchmarks.

Structure to be created during Phase 1:

- `integration/` — integration tests that span multiple crates (e.g., "create Node via napi-rs, retrieve via benten-graph, verify content hash")
- `benchmarks/` — workspace-level benchmarks using criterion (Phase 1 performance targets; see `docs/ENGINE-SPEC.md` Section 14)

Per-crate unit tests live inside each crate under `crates/*/src/` and `crates/*/tests/`.

Test runner: **cargo-nextest** (3x faster than `cargo test`, per-test isolation). See the CI workflow at `.github/workflows/ci.yml`.

Property-based tests use **proptest** for MVCC correctness, version chain invariants, and crash recovery.
