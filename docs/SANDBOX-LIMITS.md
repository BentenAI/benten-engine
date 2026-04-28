# SANDBOX Limits and Enforcement

**Status:** Specification + operator guide. Phase 2b authoritative source for the SANDBOX primitive's enforcement axes, default limits, and platform availability.

**Pin sources:** Phase 2b implementation plan §3 G7-A / G7-B / G7-C; D17 (per-call instance lifecycle); D21 (severity priority); D22 (cold-start numeric); D24 (wallclock defaults).

## Why SANDBOX needs explicit limits

The SANDBOX primitive runs untrusted WebAssembly modules inside the engine's evaluator walk. A misbehaving (or adversarial) module that consumed unbounded memory, ran a tight infinite loop, blocked indefinitely on a host call, or emitted a multi-gigabyte output buffer would compromise host availability for every other handler sharing the same engine process.

The four enforcement axes below bound *every* SANDBOX call. They are not opt-in. The numeric bounds default to the values listed in §3 below; per-call DSL overrides may tighten (lower) the bound but cannot raise it past the engine-configured ceiling.

## 1. Architecture: per-call instance lifecycle (D17)

Every SANDBOX call instantiates a fresh `wasmtime::Instance` on the same shared `wasmtime::Engine`. The instance is dropped when the call returns or traps. There is no instance pool.

**Why per-call (not pooled):**
- **Determinism + isolation.** Two calls into the same handler MUST NOT observe each other's wasm-linear-memory side effects. A pool would surface that hazard the first time a module-author wrote to a global without realising the global persisted across calls.
- **Capability scoping is per-call.** The host-function manifest resolves capability grants at instance-init time. A pooled instance would have to re-validate its host-fn closure list at every checkout — at which point the "saved" allocation cost is gone.
- **The cold-start cost is bounded.** Per-call cold-start is gated by D22 thresholds (see §6). On Linux x86_64 the budget is ≤2ms p95 / ≤5ms p99. If real-workload telemetry breaches that bound, D3 reopens for an opt-in pool consideration with measured numbers — not an arbitrary regression.

A shared `wasmtime::Engine` (NOT a shared `Instance`) keeps JIT-compiled module bytes resident so the per-call cost is instantiation, not compilation.

**Implementation lives in:** `crates/benten-eval/src/primitives/sandbox.rs` (G7-A owned).

## 2. Four enforcement axes

Every SANDBOX call is bounded by four orthogonal axes. A single call can hit any one of them; the trap-callback path picks the **most severe** axis when multiple fire in the same trap (per §4 below).

| Axis | Default | DSL override | Error code |
|------|---------|--------------|------------|
| **Memory** | 64 MiB linear-memory ceiling | `memoryLimitBytes?: number` | `E_SANDBOX_MEMORY_EXHAUSTED` |
| **Wallclock** | 30,000 ms (D24) | `wallclockMs?: number` | `E_SANDBOX_WALLCLOCK_EXCEEDED` |
| **Fuel** | 1,000,000 wasmtime fuel units | `fuel?: number` | `E_SANDBOX_FUEL_EXHAUSTED` |
| **Output** | 1 MiB output buffer | `outputLimitBytes?: number` | `E_INV_SANDBOX_OUTPUT` |

The defaults are per-call. They are tuned for "small handler" workloads (sub-millisecond compute against bounded inputs); production deployments routinely tighten the wallclock and output bounds via the DSL.

**Defaults in code:**

- `fuel` = `1_000_000` (D24 + dx-r1-2b-5)
- `wallclockMs` = `30_000` (D24 — covers cold starts on macOS arm64 + Windows x86_64 within the D22 p99 envelope)
- `outputLimitBytes` = `1_048_576` (D15 trap-loudly default)
- `memoryLimitBytes` = `64 * 1024 * 1024` (engine-side default; not exposed on the DSL in Phase 2b)

## 3. Wallclock default rationale (D24)

`wallclockMs = 30_000` covers the worst-case D22 cold-start envelope (≤10ms p99 on macOS arm64 / Windows x86_64) plus a generous compute window for most handlers. The default is intentionally **higher than the fuel default's wall-clock equivalent** so a fuel-bound trap fires before the wallclock trap on a CPU-bound workload — operators see `E_SANDBOX_FUEL_EXHAUSTED` (actionable: "raise the fuel budget") instead of `E_SANDBOX_WALLCLOCK_EXCEEDED` (less actionable: "did the workload deadlock or just take longer?").

Operators with bounded-latency SLAs should tighten `wallclockMs` per-handler in the DSL. The engine-configured ceiling (a Phase-3 admin surface) caps the maximum the DSL can raise the bound to.

## 4. Severity priority (D21)

When the trap-callback path observes more than one axis breached in the same trap, it surfaces the **most severe** error code. Severity ordering, top-down:

1. **MEMORY** — `E_SANDBOX_MEMORY_EXHAUSTED`
2. **WALLCLOCK** — `E_SANDBOX_WALLCLOCK_EXCEEDED`
3. **FUEL** — `E_SANDBOX_FUEL_EXHAUSTED`
4. **OUTPUT** — `E_INV_SANDBOX_OUTPUT`

**Why this ordering (D21 RESOLVED rationale):** memory exhaustion is the most catastrophic failure mode (potential OOM-kill of the host process). Wallclock catches deadlock + livelock. Fuel catches CPU-bound runaway computation. Output is the least severe because output bound is recoverable in user code (truncate-and-retry). The ordering matches OS-level severity discipline: OOM > deadline > CPU > IO.

The trap-callback emits exactly one error code even when multiple axes breached simultaneously, so operators see the most-actionable failure mode first. The other axes' breach is recorded in the trace's `aux` field for post-hoc diagnosis without flooding the primary error path.

**Implementation lives in:** `crates/benten-eval/src/primitives/sandbox.rs` trap-callback (G7-A owned).

## 5. Browser/wasm32 availability gate (sec-pre-r1-05 + wsa-14)

The SANDBOX executor depends on `wasmtime`, which **does not target wasm32**. The engine cannot embed a nested wasm runtime when the engine itself is compiled to wasm32 for browser execution. The compile-time gate `#[cfg(not(target_arch = "wasm32"))]` removes the executor module from the wasm32 build entirely.

**Browser-build behaviour:**

- The DSL surface (`subgraph(...).sandbox(...)`) **stays present** on wasm32 builds. Authoring is still a valid use-case in browsers — the resulting subgraph ships over the wire (Phase 3 P2P sync) for execution against a Node-resident peer.
- Registration of a SANDBOX-bearing handler **succeeds** on wasm32. Registration is pure shape-validation; it doesn't execute the wasm module.
- **Invocation** of a SANDBOX-bearing handler on a wasm32 engine surfaces the typed error `E_SANDBOX_UNAVAILABLE_ON_WASM` at execution time (the moment the evaluator walk reaches the SANDBOX node), not at registration or handler-lookup time.

**Exact UX text** (wsa-14 — pinned by `tests/sandbox_unavailable_on_wasm_error_message_exact_text_pin`):

> "SANDBOX is unavailable in browser/wasm32 builds. Author handlers in browser context for execution against a Node-WASI peer (Phase 3 P2P sync — see ARCHITECTURE.md). For local development without a peer, run the engine via @benten/engine in a Node.js process."

This text is load-bearing — operators reading the error must see (a) the failure mode, (b) the architectural escape hatch (Phase 3 P2P routing to a Node peer), and (c) the local-development workaround (run the engine in Node, not the browser). Renaming or shortening the text requires the wsa-14 test to be updated in the same commit.

The companion napi-side gate `bindings/napi/src/sandbox.rs` is `#[cfg(not(target_arch = "wasm32"))]`-gated at compile time so the `sandbox_target_supported` symbol literally does not exist in a wasm32-built napi cdylib (per sec-pre-r1-05's compile-time discipline). A complementary `#[cfg(target_arch = "wasm32")]` stub returns the same `E_SANDBOX_UNAVAILABLE_ON_WASM` typed error so a caller that reaches the symbol via dynamic dispatch still sees the actionable text.

## 6. Cold-start performance bounds (D22)

| Platform | p95 | p99 |
|----------|-----|-----|
| Linux x86_64 (canonical CI) | ≤2 ms | ≤5 ms |
| macOS arm64 | ≤5 ms | ≤10 ms |
| Windows x86_64 | ≤5 ms | ≤10 ms |

These are per-call cold-start budgets — the time from `Engine::execute_sandbox_*` entry to first wasm-fn invocation. Per-platform thresholds live in `bench_thresholds.toml`; CI's bench-gate (G7-C owned) breaches escalate to the maintainer for a D3-reopen evaluation against real-workload data.

The platform skew is real: macOS arm64 page-fault overhead on the JIT path and Windows x86_64 process-creation overhead each add measurable cost. The slacker bound on those platforms is not "we don't care" — it's an explicit acknowledgement that the canonical perf number lives on Linux x86_64 and the other platforms have known multipliers.

**Bench gate lives in:** `crates/benten-eval/benches/sandbox_cold_start.rs` (G7-A owned bench source) wired into CI via `tests/sandbox_cold_start_budget_within_target` (G7-C owned regression gate).

## 7. Cross-references

- `docs/HOST-FUNCTIONS.md` — host-function manifest, capability resolution, named-manifest registry.
- `docs/SECURITY-POSTURE.md` Compromise #4 (CLOSED at G7 land) — the live SANDBOX runtime supersedes the Phase-1 compile-check-only posture.
- `docs/ERROR-CATALOG.md` — full text of every `E_SANDBOX_*` and `E_INV_SANDBOX_*` code listed above.
- `docs/ARCHITECTURE.md` — SANDBOX primitive in the 12-primitive set; per-call instance lifecycle.
- `.addl/phase-2b/00-implementation-plan.md` §3 G7 + §5 D17/D21/D22/D24 — engineering rationale.
