# CI infrastructure posture

**Status:** load-bearing reference (Phase-3 close); supersedes the
ad-hoc framing scattered across individual workflow files.

This document captures the project's posture toward CI infrastructure
choices that have security or operational consequences. It exists per
phase-2-backlog §10.2 / Phase-3 G20-B C-12 and lives alongside
`SECURITY-POSTURE.md` (named compromises) +
`INVARIANT-COVERAGE.md` (engine invariants).

---

## Hosted runners only — no self-hosted runner machines

**Posture:** every CI workflow under `.github/workflows/` runs on
GitHub-hosted runners. The standard runner is `ubuntu-24.04`;
`codeql.yml` uses `ubuntu-latest` for upstream-pinned-runner
compatibility.

**Why no self-hosted runners:**

1. **Supply-chain attack surface.** A self-hosted runner machine
   under the project's control is a privileged execution surface that
   third-party PRs can target via `pull_request_target` mistakes,
   action-pinning bypasses, or runner-environment poisoning. GitHub-
   hosted runners are ephemeral, isolated VMs the project does not
   operate; the project never holds privileged credentials on a
   long-lived host whose security depends on the project's own ops.

2. **No operational dependency on Ben's hardware.** The project is
   one developer plus AI-orchestrated agents at the time of writing
   (Phase 3 close). Running CI on Ben's laptop or a personally-owned
   box would couple developer-time productivity to runner availability
   and would create an asymmetric attack vector: the runner machine
   would have access to the project's signing keys / publishing
   credentials / supply-chain-attestation slots over time.

3. **Reproducibility.** GitHub-hosted runners are well-documented +
   pin-able by image label (`ubuntu-24.04`); the toolchain installed
   per-job is captured in workflow YAML. A self-hosted runner would
   carry residual ambient state (sccache contents, cargo registry,
   node_modules) that the workflow YAML does not capture, which makes
   intermittent failures harder to reproduce + reason about.

**When this might change:** the cross-browser-determinism workflows
already do per-target shard runs against multiple browser engines
(Chromium / Firefox / WebKit) on hosted runners; if Phase-9+ OSS-launch
work needs hardware acceleration the project does not currently use
(GPU sandboxing, ARM-only crates, or persistent sccache hot caches),
the posture would be revisited THEN, with a documented threat-model
update + branch-protection-spec change. Until that explicit decision,
hosted-runner-only is the standing rule.

---

## Runner-image pinning convention

Workflows pin to **`ubuntu-24.04`** as the default image label rather
than `ubuntu-latest`, so a GitHub-side default-image rotation does
not silently flip the build environment under the project. The one
exception is `codeql.yml`, which follows GitHub's CodeQL action's own
preferred-runner posture.

When upgrading the standard image (e.g. `ubuntu-26.04` when it
stabilizes), the upgrade lands as a single PR sweeping every
`runs-on: ubuntu-24.04` line + including a CHANGELOG-style narrative
in the PR body covering what new toolchain versions ship in the new
image (rust default, node default, etc.).

---

## Workflow-baseline coverage

The CI workflow set at Phase-3 close covers:

- **Build + test:** `ci.yml` (full nextest run + clippy + fmt),
  `coverage.yml` (cargo-llvm-cov), `msrv.yml` (MSRV gate),
  `wasm-conformance.yml` + `wasm-runtime.yml` (wasm32 target gates).
- **Supply chain:** `cargo-vet.yml` (audit-policy), `cargo-public-api.yml`
  (per-crate API drift), `supply-chain.yml` + `supply-chain-seeded-test.yml`
  (transitive-dep posture).
- **Security:** `codeql.yml` (CodeQL static analysis), `fuzz.yml`
  (cargo-fuzz harnesses), `host-error-wire-safety.yml` (DAG-CBOR wire
  safety pin).
- **Architecture:** `arch-1-dep-break.yml` (crate-graph hygiene gate),
  `cite-drift.yml` (file:line + symbol-cite drift detector +
  numeric-claim drift lint), `drift-detect.yml` (TS/Rust error code
  drift), `inv-11-system-zone-drift.yml` (system-zone drift gate).
- **Performance:** `bench.yml` (criterion baselines),
  `bench-threshold-drift.yml` (perf-regression gate),
  `bundle-size.yml` (browser-bundle-size cap).
- **Cross-process / cross-browser:** `cross-process-graph.yml`,
  `cross-browser-determinism.yml`, `determinism.yml` (canonical-CID
  determinism harness).
- **Branch protection meta:** `branch-protection-spec-check.yml`
  (declarative branch-protection-spec.json self-test).

The branch-protection spec under `.github/branch-protection-spec.json`
declaratively names the required-checks set. `branch-protection-spec-check.yml`
self-tests the spec on every PR; the spec is the single source of
truth for what blocks merge.

---

## Out of scope / pending

- **OSS publication signing infrastructure** — deferred to Phase 8/9+
  per Phase-2a §3.2 publish-readiness-pass framing. The project does
  not currently sign releases; `cargo-public-api` baselines are the
  pre-publication API stability gate, not a publication hardening
  surface.
- **Self-hosted runner adoption** — explicitly deferred per the
  posture above. Any future change reopens this document + requires
  a SECURITY-POSTURE.md compromise narrative + a branch-protection-spec
  update.

---

*Last touched: G20-B Phase-3 close (2026-05-07). Future updates: when
runner image rolls, when supply-chain workflow set expands, when
self-hosted runner posture is revisited per the explicit-decision
trigger above.*
