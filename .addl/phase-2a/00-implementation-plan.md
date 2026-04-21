# Phase 2a Implementation Plan — Benten Engine

**Pipeline stage:** Pre-R1 (planning).
**Source of scope truth:** `docs/FULL-ROADMAP.md` §Phase 2 (split 2a/2b pre-R1 on review-lens-coherence grounds) + `docs/ENGINE-SPEC.md` §14 + `docs/future/phase-2-backlog.md` (consolidated Phase-1 deferrals) + Phase-1 R7 compliance audit §16 risk call-outs. The original `Phases 1-8` roadmap splits Phase 2 into **Phase 2a** (this plan — evaluator completion, remaining invariants, arch-1 dep break, TOCTOU, Option C evaluator-path) and **Phase 2b** (scope outlined at `.addl/phase-2b/00-scope-outline.md` — SANDBOX, wasmtime, STREAM/SUBSCRIBE, Algorithm B, WASM runtime, module manifest). Phase 2a opens its own full ADDL cycle; Phase 2b opens a fresh pre-R1 after 2a ships.
**Feeds into:** pre-R1 critics → triage → R1 spec review → R2 test landscape → R3 test writing → R4 test review → R5 implementation → R4b → R6 quality council.
**Current baseline:** Phase 1 closed at HEAD `196239b` (7 crates: `benten-core`, `benten-graph`, `benten-ivm`, `benten-caps`, `benten-eval`, `benten-engine`, `benten-errors`; napi v3 bindings + `@benten/engine` TS wrapper + `create-benten-app` scaffolder; 8 of 12 primitives executable; 8 of 14 invariants enforced; 5 hand-written IVM views; 44 Phase-1 catalog codes reachable; 8 named compromises verified; R7 audit clean).

---

## 1. Executive Summary

Phase 2a ships the structural and debt-close half of what Phase 1 scoped out: the **`benten-eval → benten-graph` dependency break** (arch-1, the earliest-ordered work because it reshapes the evaluator's error surface every downstream group depends on); the **WAIT primitive executor** with serializable execution state on a content-addressed (DAG-CBOR + CIDv1) envelope; **four of the six remaining structural invariants** — Inv-8 multiplicative-through-CALL, Inv-11 full system-zone enforcement, Inv-13 immutability, Inv-14 structural causal attribution — plus runtime firing of Inv-8 currently held at the Phase-1 scalar-budget + nest-depth stopgap; **evaluator-path Option C** threading `check_read_capability` into the READ primitive so `crud:post:get` honours symmetric-None end-to-end; **wall-clock TOCTOU delegation** for long-running iterations (closes Compromise #1 Phase-2 item); and the **`DurabilityMode::Group` default** + **subgraph AST cache** that together make ENGINE-SPEC §14.6's 150–300µs headline target reachable on dev hardware. A per-item `missing_docs` sweep on `benten-eval` + a dev server with hot reload close the Phase-1 DX backlog.

**Phase 2b (follow-on phase with its own pre-R1 cycle)** ships the WASM + SANDBOX half: SANDBOX + wasmtime host with capability-derived host-function manifest (decision baked pre-R1 — see §9.3), STREAM and user-visible SUBSCRIBE executors, generalized IVM Algorithm B with per-view strategy selection and user-registered views, WASM runtime target via napi-rs v3, module manifest format, Inv-4 and Inv-7 (which are SANDBOX-adjacent). Full scope at `.addl/phase-2b/00-scope-outline.md`.

**Rationale for the split:** Phase 2a is well-scoped structural + debt-close work; Phase 2b has wasmtime API stability + host-function design unknown-unknowns. Running one ADDL cycle for each keeps the review-lens coherent (`code-as-graph-reviewer` + invariant lens for 2a; `wasmtime-sandbox-auditor` + `ivm-algorithm-b-reviewer` for 2b) and isolates 2b's speculative risk from 2a's shippable work.

Phase 2a explicitly does NOT ship: any SANDBOX / wasmtime / STREAM / SUBSCRIBE / Algorithm B / WASM / module-manifest content (all 2b); iroh P2P transport, CRDT merge, Merkle Search Tree diff, UCAN chain validation, DID/VC identity, device-mesh key recovery (all Phase 3); Thrum migration (Phase 4); platform features / AI assistant / Gardens / Credits (Phases 5–8). If a Phase 2a design would foreclose a Phase 2b or Phase 3 direction, the plan flags it rather than drags it in.

**Headline exit criteria (Phase 2a):** two headline gates + three bundled gates, all mechanically-verifiable, all CI-gated green on every PR.

1. **WAIT-resume determinism test** — `crates/benten-engine/tests/integration/wait_resume.rs::wait_serializes_and_resumes` registers a handler `[READ → WAIT(wait_for="external:signal") → TRANSFORM → RESPOND]`, drives it to suspension, persists the execution state via the engine's `suspend_to_bytes(handle)` API, tears down the engine, re-opens it from a fresh redb directory populated with the same data + the serialized state bytes, and resumes with `Engine::resume_from_bytes(state, signal_value)`. Assertion: the final RESPOND output is bit-identical to a reference run with no suspension; the trace sequence matches (modulo the suspend/resume boundary step). The on-disk bytes are DAG-CBOR + CIDv1 envelope (decision §9.1); a companion test asserts CID determinism across two suspend calls producing the same state.

2. **Four new invariants firing** (8 multiplicative, 11 full, 13 immutability, 14 structural causal attribution) — paired positive+negative tests for Inv-8 multiplicative-through-CALL, Inv-11 full system-zone enforcement (covers both registration-time literal-CID rejection AND run-time TRANSFORM-computed-CID rejection), Inv-13 immutability (a registered subgraph's bytes cannot be mutated; CAS attempt returns a typed error), Inv-14 structural causal attribution (every evaluation step carries an attribution triple reachable via `engine.trace()`).

Bundled gates (required green for Phase 2a close; part of the exit contract):

3. **arch-1 dep break landed cleanly** — `crates/benten-eval/Cargo.toml` no longer depends on `benten-graph`; `EvalError` surfaces storage failures through a new `HostError` / typed-code path (see §9.2); workspace `cargo tree` shows the broken edge.
4. **Option C evaluator-path threaded** — `crud:post:get` dispatched through `Engine::call` honours symmetric-None end-to-end without a separate gate at the public API. Existing `engine_diagnostics.rs::diagnose_read` surface stays; the READ primitive now consults `PrimitiveHost::check_read_capability` inline.
5. **Durability + AST-cache perf gate** — ENGINE-SPEC §14.6's 150–300µs 10-node-handler target reachable on macOS APFS once Durability::Group is default + AST cache is live. Criterion suite CI-gated (not just advisory) at the 300µs p95 threshold.

Phase 2a is mechanically complete when all five gates are green on main. (Phase 2b's headline gates — SANDBOX smoke + STREAM SSE back-pressure + wasm32-wasip1 canonical-CID roundtrip — open with its pre-R1.)

---

## 2. Scope Inventory (by crate)

Legend: `absent` (no code) · `stub` (marker / not-implemented placeholder) · `partial` (shipped for Phase-1 subset) · `present` (Phase-1-complete, Phase-2 touch may still be needed) · `reshape` (existing surface needs a breaking change).

### 2.1 `benten-core`

| # | Deliverable | Status at HEAD | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| C1 | `AnchorStore` handle for bulk version-chain ops | stub (`per-anchor Arc<Mutex<...>>` at `version.rs:~60`) | Promote to explicit `AnchorStore` collected at `Engine` scope; expose `append_version_batch` + bulk-read API; old per-anchor `Arc<Mutex>` becomes a cache entry inside it. Closes backlog §6.3. | — | **P2.core.anchor-store** — `TODO(phase-2-anchorstore)` |
| C2 | `Cid::from_str` base32-lower-nopad decoder | **CLOSED Phase-1-R7** (landed as F-R7-004 close-out at `crates/benten-core/src/lib.rs`) | Re-verify at Phase 2 open: if still closed, remove from inventory; else complete as originally spec'd | — | verify-only |
| C3 | Upstream `rust-cid` / `rust-multihash` unpins | `[patch.crates-io]` pins active | Remove pins once `multiformats/rust-cid#185` + `rust-multihash#407` release; re-run T9 cross-leg determinism. If upstream has not released by Phase 2 close, re-defer with explicit wait note. | — | **P2.core.unpin** — monitor-only |
| C4 | `Node::load_verified` read-path surface | absent (Phase-1 does subgraph-level `Subgraph::load_verified` only) | Add optional `Engine::get_node_verified(&self, cid) -> Result<Option<Node>, EngineError>` that re-hashes on read (~3–10µs BLAKE3 per call). Catalog `E_INV_CONTENT_HASH` updated to distinguish "Node read" from "Subgraph load." Closes backlog §6.2. | — | **P2.core.read-verify** |
| C5 | `Subgraph` DAG-CBOR schema adds `deterministic` field | partial (field exists in in-memory `Subgraph`; DAG-CBOR serialization drops it per r6-opl-5) | Extend the serialised schema to carry `deterministic` so Invariant 9 check on the finalised-load path sees the flag. Pre-existing Phase-1 serialised Subgraphs must migrate (one-shot rewrite on first open) — see risk §5. | — | **P2.core.subgraph-deterministic** |

### 2.2 `benten-graph`

| # | Deliverable | Status | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| G1 | `DurabilityMode::Group` default for CRUD fast-path | present (all three modes exist, Immediate is default) | Change default to `Group` for CRUD write path; keep Immediate opt-in for capability-grant writes (security-class). Validate on macOS APFS the 150–300µs headline becomes reachable. Closes backlog §9.1. | — | **P2.graph.durability-default** — R1 decision point |
| G2 | Subgraph AST cache completion | partial (`SubgraphCache` wired at `benten-engine/src/engine.rs:265`; `Engine::call` still re-parses per-call at `engine.rs:449`) | Complete the cache wire-through so `Engine::call` consults the AST cache before re-parsing. Measurable target: `crud_post_build_subgraph_only` bench median < 10µs (Phase-1 target, currently informational). Closes backlog §9.2. | G1 | **P2.graph.ast-cache** |
| G3 | Per-event capability read-check gate on `subscribe_change_events` | absent (honest limitation in Phase 1) | **DEFER to Phase 3** per `docs/SECURITY-POSTURE.md` §"Change-stream subscription bypasses capability read-checks." Phase 2 only adds a `cfg(feature = "change-stream-cap-gate")` scaffold so the hook is pluggable when Phase 3 sync needs it. | — | re-defer Phase 3 |
| G4 | Immutability enforcement at write-path (Invariant 13) | absent | Reject `put_node` when the target CID already exists unless `WriteContext::privileged` is true (engine API path for version-chain `NEXT_VERSION` append). Cache CID-existence pre-check against a bloom filter to avoid O(log n) per write. Fires `E_INV_IMMUTABILITY` (new code to be reserved pre-R1). | E1 (eval side) | **P2.graph.immutability** |
| G5 | Event log replay for durable IVM views | absent (views rebuild on open) | **DEFER to Phase 3** per backlog §5.4 — requires sync-era consistency story. Phase 2 adds rebuild-timing benchmark so we know the rebuild cost scales linearly with log size. | — | re-defer Phase 3 |

### 2.3 `benten-ivm`

| # | Deliverable | Status | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| I1 | `View` trait extracted | present (all 5 views implement the shared trait per R1-triage Architect Major) | No work | — | — |
| I2 | Generalized Algorithm B | absent | Ship `AlgorithmB<Input, Output>` impl behind `View` trait. Dependency tracker per-input-CID; per-view strategy tag `Strategy::{A, B, C}`; strategy chosen at view-registration time based on the view's access pattern declaration. Benchmark against the 5 hand-written views (must match within 20%) before replacing any of them. | I1 | **P2.ivm.algorithm-b** (long-pole, see §5) |
| I3 | Per-view strategy selection (A/B/C) | absent | Strategy A = O(1) HashMap (existing content-listing pattern); Strategy B = dependency-tracked incremental; Strategy C = sort-on-read with B-tree. Registration API: `engine.create_view(ViewSpec { strategy, ... })`. Default strategy = B. | I2 | **P2.ivm.strategy-select** |
| I4 | User-registered views (beyond the 5 built-ins) | stub (engine API `create_view` at `engine.rs:1549` has `TODO(phase-2-view-id-registry)`) | Promote to first-class: `Engine::create_view` accepts a user-authored `ViewSpec` (label pattern, projection fn, strategy tag). Persists in the `system:ivm_view` zone. Retrieval via `engine.read_view(view_id, query)`. | I2, I3 | **P2.ivm.user-views** |
| I5 | `E_IVM_PATTERN_MISMATCH` firing coupled with user-registered views | present (all 5 built-in views fire; backlog §5.2 CLOSED Phase-1) | Ensure user views inherit the pattern-mismatch contract. Test: user view queried with wrong partition shape returns `E_IVM_PATTERN_MISMATCH`. | I4 | verify-only |
| I6 | `benten.ivm.view_stale_count` metric wire-up | stub (`engine_diagnostics.rs:170-184` hard-codes `0.0`) | Wire the subscriber to tally `View::is_stale()` across all handles at metric-snapshot time. Mutex cost acceptable (metrics snapshot is called at <1Hz in production deployments per r6-perf tablet). Closes backlog §5.3. | I4 | **P2.ivm.metrics** |

### 2.4 `benten-caps`

| # | Deliverable | Status | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| P1 | Wall-clock TOCTOU delegation | stub (`TODO(phase-2-wallclock-toctou)` at `policy.rs:234`) | Extend `CapabilityPolicy` with `wallclock_refresh_ceiling() -> Duration` (default 300s). Evaluator's `ITERATE` + `CALL` refresh cadence becomes `min(iteration_count, wall_clock_seconds)`. Tightens Compromise #1. Closes backlog §7.1. | E2 (evaluator side), T1 (time source) | **P2.caps.wallclock-toctou** |
| P2 | `iterate_batch_boundary` delegation | stub (`TODO(phase-2-iterate-boundary-delegation)` at `policy.rs:225`) | Evaluator's `PrimitiveHost::iterate_batch_boundary` consults the configured `CapabilityPolicy::iterate_batch_boundary` instead of a constant. Policy override becomes load-bearing. | E2, P1 | **P2.caps.batch-delegation** |
| P3 | `GrantBackedPolicy` wired into READ primitive execute path | partial (wired at engine orchestrator public API; `TODO(phase-2-grant-backed-policy)` at `engine.rs:590`) | Thread `PrimitiveHost::check_read_capability` hook into the READ primitive's execute path at `crates/benten-eval/src/primitives/read.rs:44`. `crud:post:get` dispatched through `Engine::call` honours Option C end-to-end without a separate public-API gate. Closes backlog §4.1. | E3 | **P2.caps.option-c-path** |
| P4 | UCAN backend full impl | stub (`CapError::NotImplemented`) | **DEFER to Phase 3** per `docs/future/phase-2-backlog.md §7.2` — UCAN chain validation ships alongside Ed25519 / DID / VC in `benten-id`. Phase 2 only tightens the stub error to name Phase 3. | — | re-defer Phase 3 |

### 2.5 `benten-eval` (the biggest scope expansion)

| # | Deliverable | Status | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| E1 | `benten-eval → benten-graph` dependency break (arch-1) | present dep (`benten-eval/Cargo.toml` imports `benten-graph` for `GraphError` variant in `EvalError::Graph(#[from] GraphError)`) | Remove the dep. New `HostError` type in `benten-eval`; engine's `PrimitiveHost` impl catches storage errors at the host boundary and maps them to `HostError` or an already-stable code in `benten-errors`. 15–30 file touch per backlog §1.1. **Land before new primitive work** so Phase 2 primitives design against the stable host surface. | — | **P2.eval.arch-1** — group G1 (first) |
| E2 | Iterative evaluator state serialization | absent (stack is in-memory) | Define a serialisable `ExecutionState` shape (DAG-CBOR, content-addressed). Per R1-triage Architect Major #2 we already chose `Vec<ExecutionFrame>` + `frame_index` — Phase 2 adds `Serialize + Deserialize` derives + DAG-CBOR round-trip + CID determinism. WAIT builds on this; SANDBOX re-entrancy is excluded (no WAIT inside SANDBOX). | E1 | **P2.eval.exec-state** — R1 open question §9.1 |
| E3 | WAIT primitive executor | stub (returns `E_PRIMITIVE_NOT_IMPLEMENTED`) | Suspend current frame; persist `ExecutionState` to a new `system:wait_pending` zone keyed by the wait signal id; return a `SuspendedHandle` to the caller. Resume API: `Engine::resume(handle, signal_value) -> Outcome`. Time-source abstraction via `TimeSource` trait (default = `uhlc::HLC` wrapper, mock impl for tests). Integration with the existing HLC already used for `createdAt` injection. | E1, E2, T1 | **P2.eval.wait** |
| E4 | STREAM primitive executor | stub | Produces partial outputs via a `StreamChunk` sequence emitted through a new `ChunkSink` trait. Back-pressure: the sink advertises capacity; the executor blocks on push when capacity is exhausted (pull-based with bounded channel). napi layer bridges to Node.js ReadableStream (WinterTC ReadableStream compatible). Host-function manifest lists allowable sinks (Phase 2: in-process, SSE, WebSocket; Phase 3: P2P-gossip added). | E1, B2 (napi side) | **P2.eval.stream** — R1 open question §9.4 |
| E5 | SUBSCRIBE primitive (user-visible) | stub | Exposes the change-notification stream as a primitive user subgraphs can use. SUBSCRIBE node properties: `label_pattern: Text`, `property_filter: Option<Map>`. Emits a `StreamChunk` per matching `ChangeEvent`. Subject to the same output-size invariant as STREAM. Internally just composes the Phase-1 change-stream plumbing with STREAM's sink. | E4 (composes on top) | **P2.eval.subscribe-user** |
| E6 | SANDBOX primitive executor + wasmtime host | stub | Full wasmtime integration: module compile at registration; `Instance` pool keyed by module CID; fuel budget per-subgraph declared in operation-Node properties (`fuel: Number`, default 1M); output limit (`output_limit: Number`, default 1 MiB); host-function manifest (read-only — no mutation from inside SANDBOX); no re-entrancy (SANDBOX cannot call back into Benten primitives). Fires `E_SANDBOX_FUEL_EXHAUSTED`, `E_SANDBOX_TIMEOUT`, `E_SANDBOX_OUTPUT_LIMIT`. See R1 open question §9.5. | E1 | **P2.eval.sandbox** — long-pole, R1 design-heavy |
| E7 | Structural invariant 4 (SANDBOX nest depth) | absent | Registration-time check: count SANDBOX nodes on any path through the subgraph DAG. Reject when > `max_sandbox_nesting` (default 2). Fires `E_INV_SANDBOX_NESTED`. | E6 | **P2.eval.inv-4** |
| E8 | Structural invariant 7 (SANDBOX output ≤1MB) | absent | Run-time check inside SANDBOX executor. Already listed under E6 but surfaced here as its own invariant number for the invariant-coverage table. Fires `E_SANDBOX_OUTPUT_LIMIT`. | E6 | **P2.eval.inv-7** |
| E9 | Structural invariant 8 multiplicative | partial (scalar runtime budget + registration-time nest-depth-3 stopgap) | Replace scalar with multiplicative accounting through CALL + ITERATE. Cumulative budget computed at registration (static upper bound based on declared `max` property on every ITERATE node along every DAG path). Drop the nest-depth stopgap once multiplicative lands. Fires existing `E_INV_ITERATE_BUDGET`. | E1 | **P2.eval.inv-8** |
| E10 | Structural invariant 11 full | Phase-1 stopgap via write-path `E_SYSTEM_ZONE_WRITE` | Registration-time check: reject subgraphs whose READ / WRITE primitives target any CID with a `system:` prefix label, unless `WriteContext::privileged`. Fires new `E_INV_SYSTEM_ZONE` code (already reserved in catalog). Phase-1 write-path stopgap removed once evaluator-level enforcement green. | E1 | **P2.eval.inv-11** |
| E11 | Structural invariant 13 (immutability) | absent | At evaluator level, reject WRITE primitives that would mutate a registered subgraph node. At storage level (G4), reject the write. Double-layer: eval prevents the write attempt; storage is the backstop. Fires new `E_INV_IMMUTABILITY` code (to be reserved). | E1, G4 | **P2.eval.inv-13** |
| E12 | Structural invariant 14 (causal attribution structural) | partial (attribution captured on writes per Phase-1 `PendingOp::PutNode`; structurally present on `ChangeEvent` + `TraceStep`) | Structural enforcement: every `TraceStep` MUST carry `(actor_cid, handler_cid, capability_grant_cid_used)`; absence is a registration-time error for any primitive that does not declare its attribution source. Fires new `E_INV_ATTRIBUTION` code (to be reserved). | E1 | **P2.eval.inv-14** |
| E13 | Per-item `missing_docs` sweep | stub (~120 pub items carry `allow(missing_docs, reason="TODO(phase-2-docs): benten-eval has ~120 pub items (Subgraph builder, primitives, RegistrationError diagnostic fields, expr parser surface). Crate-root + module-root docs land Phase-1 R6; per-item sweep deferred to Phase-2 when the public surface is re-audited post-evaluator-completion.")` escape hatch at crate root `crates/benten-eval/src/lib.rs:13-15`) | Drop the `allow(missing_docs)` escape hatch; per-item docstrings on all ~120 pub items. Lands with the crate's public-surface re-audit post-evaluator-completion. Closes backlog §8.3. | E1, E3–E6 | **P2.eval.docs** |
| E14 | Paper-prototype re-validation against revised primitives | backlog (original 12 saw 2.5% SANDBOX rate; revised 12 not measured) | Re-measure against the 2026-04-14 set (SUBSCRIBE + STREAM added, VALIDATE + GATE removed). Target: SANDBOX rate < 30%. Artifact: `docs/validation/paper-prototype-handlers-phase-2.md`. | E3–E6 | **P2.eval.revalidate** |

### 2.6 `benten-engine`

| # | Deliverable | Status | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| N1 | `PrimitiveHost` surface expansion for new primitives | present (Phase-1 surface with `check_read_capability`) | Add `suspend_execution_state`, `resume_execution_state`, `acquire_sandbox_instance`, `open_chunk_sink`, `open_change_subscription` methods. Keep the thin-engine test: `Engine::builder().without_ivm().without_caps().without_versioning()` still constructs a usable graph DB. | E1, E3–E6 | **P2.engine.primitive-host** |
| N2 | Module manifest format | absent | Define + persist: `requires_caps: Vec<CapSpec>`, `provides_subgraphs: Vec<SubgraphRef>`, `migrations: Vec<MigrationSpec>`. Format TBD at R1 (see §9.6). Storage: new `system:module_manifest` zone. Public API: `Engine::install_module(manifest)`, `Engine::uninstall_module(manifest_cid)`. | N1, E10 | **P2.engine.manifest** — R1 open question §9.6 |
| N3 | WAIT / resume public API | absent | `Engine::call_with_suspension(handler_id, input) -> SuspendedOrComplete`; `Engine::suspend_to_bytes(handle) -> Vec<u8>` (DAG-CBOR); `Engine::resume_from_bytes(bytes, signal_value) -> Outcome`. | E3 | — |
| N4 | STREAM + SUBSCRIBE public API | absent | `Engine::call_streaming(handler_id, input) -> ChunkStream`. | E4, E5 | — |
| N5 | `Engine::builder().production()` evolves | present (refuses NoAuthBackend) | Extend: now also requires `SandboxPolicy` to be set (default = reject). Backlog migration: existing `.production()` callers get compile-time error, given a grace trait-impl path. | N1, E6 | **P2.engine.production** |
| N6 | Engine-internal SubgraphCache + AST cache complete wire-through | present (SubgraphCache at `engine.rs:265`, incomplete wire-through) | See G2. Engine orchestrates the cache lookup; eval is a pure consumer. | G2 | — |
| N7 | `Engine::transaction` nested-tx decision | Phase-1 rejects with `E_NESTED_TRANSACTION_NOT_SUPPORTED` | **R1 debate** — does Phase 2 lift this now that primitive set is complete, or defer? Recommendation: defer (composed subgraphs give equivalent atomicity without nested txn semantics). | — | **P2.engine.nested-tx** — R1 open question §9.7 |

### 2.7 `bindings/napi`

| # | Deliverable | Status | Phase 2 target | Dependencies | Triage tags |
|---|---|---|---|---|---|
| B1 | `wasm32-wasip1` runtime build | compile-check only (Phase-1 T8) | Full runtime build: load `bindings/napi` compiled to wasm32-wasip1 under wasmtime in CI. Exposes the binding surface with SANDBOX disabled (WASM host IS the sandbox per ENGINE-SPEC §14.6) and with a network-fetch `KVBackend` stub. | — | **P2.napi.wasm-runtime** — R1 open question §9.8 |
| B2 | STREAM napi bridge | absent | Translate `ChunkStream` ↔ Node.js ReadableStream. WinterTC-compatible. Back-pressure respects `highWaterMark`. SSE helper: `engine.callSse(handler_id, input)` wraps the ReadableStream in an SSE-formatted byte stream. | N4, E4 | **P2.napi.stream** |
| B3 | WAIT suspension/resume napi surface | absent | `engine.callWithSuspension(id, input) → {kind: "complete" | "suspended", value | handle}`; `engine.resume(handleBytes, signal) → Outcome`. Handle is a `Buffer` of DAG-CBOR bytes. | N3, E3 | **P2.napi.wait** |
| B4 | SUBSCRIBE napi surface | absent | `engine.subscribe(pattern, cb)` ergonomic wrapper on top of the STREAM bridge; returns an `UnsubscribeHandle`. | B2, E5 | **P2.napi.subscribe** |
| B5 | Network-fetch `KVBackend` stub (for WASM target) | absent | Trait-level impl that returns `Err(BackendError::NotImplemented("network-fetch KVBackend is Phase-3 scope; Phase-2 ships the trait shape with a fallback that reads from an in-memory snapshot blob passed at engine construction"))`. Browser contexts that construct the engine with a snapshot blob get read-only access to a content-addressed dataset — enough to prove the WASM target compiles + runs end-to-end. Full network-fetch lands in Phase 3 alongside iroh. | B1 | **P2.napi.netfetch-stub** |
| B6 | SANDBOX host exposure | absent | When loaded in Node.js, SANDBOX uses embedded wasmtime per E6. When loaded in wasm32-wasip1, SANDBOX is disabled (returns `E_SANDBOX_UNAVAILABLE_ON_WASM`, new code to be reserved). | E6 | **P2.napi.sandbox** |

### 2.8 `packages/engine` (TS DSL)

| # | Deliverable | Status | Phase 2 target | Dependencies |
|---|---|---|---|---|
| D1 | `wait()`, `stream()`, `subscribe()`, `sandbox()` DSL builders | TS surfaces exist at `packages/engine/src/dsl.ts` (exported from `index.ts`); executors return `E_PRIMITIVE_NOT_IMPLEMENTED` at runtime | Complete the DSL options surfaces alongside their Rust executors: `wait({ for: "external:signal" })`, `stream({ chunks: iterable })`, `subscribe({ pattern })`, `sandbox({ module: wasmBytes, fuel: 1_000_000 })`. | B2, B3, B4, B6 |
| D2 | Module manifest TS types + manifest-file validator | absent | `@benten/engine/manifest` submodule. Manifest is authored in TS, validated against the Phase-2 manifest format. | N2 |

### 2.9 `benten-errors`

| # | Deliverable | Status | Phase 2 target | Dependencies |
|---|---|---|---|---|
| X1 | New Phase-2 enum variants | Phase-1 variants present; Phase-2 codes exist in catalog but absent from enum | Add variants: `InvSandboxNested` (invariant 4), `SandboxFuelExhausted`, `SandboxTimeout`, `SandboxOutputLimit`, `SandboxUnavailableOnWasm`, `InvSystemZone` (invariant 11 full), `InvImmutability` (invariant 13), `InvAttribution` (invariant 14), `PrimitiveNotImplemented` variant stays (narrows to "intentionally-disabled" after all 12 executors ship — e.g. SANDBOX on WASM). | E6–E12 |
| X2 | T7 codegen regenerates | present | Re-run `scripts/codegen-errors.ts`; TS types pick up new variants. Drift detector in CI. | X1 |

### 2.10 Developer tooling + CI

| # | Deliverable | Status | Phase 2 target |
|---|---|---|---|
| T1 | `TimeSource` trait abstraction | absent | Used by evaluator wall-clock (P1 caps), HLC stamping, WAIT suspension timeout. Default = `uhlc::HLC`; tests inject `MockTimeSource`. |
| T2 | Dev server with hot reload | Phase-1 ships `npm test` path only (dev server explicitly deferred) | `npx benten-dev` watches handler TS files, recompiles subgraphs, re-registers, evaluates with current state on change. Closes backlog §8.1. |
| T3 | CI workflow: SANDBOX fuzz + sandbox-escape harness | absent | wasmtime fuzz tests targeting host-function allowlist boundaries; output-limit adversarial inputs; fuel-exhaustion timing attacks. New `.github/workflows/sandbox-fuzz.yml`. |
| T4 | CI workflow: wasm32-wasip1 runtime | absent | `.github/workflows/wasm-runtime.yml` runs `wasmtime --dir=.` on the napi crate's wasm32-wasip1 binary against a canonical input fixture. |
| T5 | CLAUDE.md drift lint | absent | Backlog §8.2 — CI lint greps CLAUDE.md for numeric performance claims + compares against ENGINE-SPEC §14.6 ranges. Low priority; ship if cheap. |
| T6 | ADDL reviewer composition docs update | — | Phase-2 pattern 6 writeup gets its own section in `docs/DEVELOPMENT-METHODOLOGY.md`. |

---

## 3. Implementation Groups (R5 partition)

Seven Phase 2a groups + four Phase 2b groups (preserved here for traceability; dispatch via Phase 2b's own pre-R1). Phase 2a dispatches G1, G2, G3, G4, G5-A, G5-B, G9, G11-2a. Phase 2b dispatches G6, G7, G8, G10, plus a 2b-only G11-2b wrap. Agents within a group own disjoint files. Each group is 1–3 days of reviewed-human-equivalent work. The arch-1 dep break (G1) goes FIRST per backlog §1.1 rationale: new primitive work (WAIT / STREAM / SUBSCRIBE / SANDBOX) otherwise accretes against the coupled `EvalError::Graph(#[from] GraphError)` surface and doubles the retrofit cost. Per arch-3, G5 is split into G5-A (Inv-13, storage-layer, parallel with G3) and G5-B (Inv-11 full + Inv-14 structural, serial after G3).

### G1 — arch-1 dep break + invariants module split (land first)

**Agents:** 1 × `rust-implementation-developer` + 1 × `rust-engineer` (cross-cutting refactor lens) + 1 × `architect-reviewer` (mini-review only, post-commit)
**Parallelism:** no — cross-cutting touch of 15–30 files is serial

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G1-A | `crates/benten-eval/Cargo.toml` (drop benten-graph dep), `crates/benten-eval/src/lib.rs` (new `HostError` alongside existing `EvalError`; surface via `pub use`), `crates/benten-eval/src/host.rs` (update `PrimitiveHost` to surface storage errors via `HostError`), `crates/benten-engine/src/primitive_host.rs` (map `GraphError` → `HostError` at the host boundary), `crates/benten-errors/src/lib.rs` (possibly add `ErrorCode::Host` if needed), **invariants-module structural split (pure code-move, zero semantic change for the Phase-1 invariants — see `invariants/mod.rs` + `invariants/structural.rs` + per-invariant sub-files listed below)** | All Phase-1 tests remain green; new `tests/arch_1_no_graph_dep.rs` asserting `benten-eval` has no `benten-graph` in its manifest; typed-error round-trip tests for every `HostError` variant; invariants split compiles with identical public surface (import paths preserved via `mod.rs` re-exports) |

**Invariants module split (G1 scope, zero-semantic-change refactor):**

```
crates/benten-eval/src/invariants/
  mod.rs             — public re-exports + validate_subgraph dispatch
  structural.rs      — invariants 1/2/3/5/6/9/10/12 (Phase-1 set; pure code-move from invariants.rs)
  budget.rs          — invariant 8 multiplicative (G4-A will populate this file)
  system_zone.rs     — invariant 11 full enforcement (G5-B will populate this file)
  immutability.rs    — invariant 13 (G5-A will populate this file; no evaluator dep)
  attribution.rs     — invariant 14 structural (G5-B will populate this file)
```

Each sub-file is single-owner so downstream groups don't contend on `invariants.rs`. `mod.rs` is the only file with multi-group visibility and only re-exports; `validate_subgraph` dispatches to each sub-module. G1 lands the skeleton + the Phase-1 move; downstream groups (G4-A, G5-A, G5-B) fill in the Phase-2 invariant bodies in their own files.

**Gates:** G3, G4, G5, G6, G7 (all primitive-executor groups depend on the stable host surface).
**Deliverables closed:** E1.
**R1 review lens:** `architect-reviewer` (dep-graph shape), `benten-engine-philosophy` (thin-engine test must still pass — philosophy specifically calls out pre-settling abstractions the Phase-2 primitive surface will exercise).

### G2 — Durability default + AST cache completion + immutability write-path

**Agents:** 1 × `rust-implementation-developer` + 1 × `performance-engineer` (durability / bench lens)
**Parallelism:** yes (different files)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G2-A | `crates/benten-graph/src/redb_backend.rs` (change DurabilityMode default for CRUD fast-path), `crates/benten-graph/src/transaction.rs` (respect the durability tier per-write-class), `crates/benten-graph/src/immutability.rs` (new — bloom-filter CID-existence cache + reject on re-put) | `tests/durability_group_default_reachable`, `tests/capability_grant_writes_immediate`, `tests/immutability_rejects_reput`, criterion `create_node_group_commit` <500µs median |
| G2-B | `crates/benten-engine/src/engine.rs` (`subgraph_for_crud` + `dispatch_call_inner` — complete AST cache wire-through), `crates/benten-engine/benches/subgraph_cache_hit.rs` (new) | `tests/engine_call_uses_ast_cache`, criterion `crud_post_build_subgraph_only` < 10µs median |

**Gates:** G3–G7 (stable storage primitives), G4 (G4's immutability eval-check composes on G2-A).
**Deliverables closed:** G1 (durability default), G2 (AST cache), G4 (storage-side of immutability).

### G3 — WAIT + evaluator serialization + TimeSource

**Agents:** 2 × `rust-implementation-developer` + 1 × `chaos-engineer` (crash-recovery + race-condition lens, pairs naturally with serialise-then-resume) + 1 × `determinism-verifier` (mini-review: DAG-CBOR round-trip + CID stability of ExecutionState)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G3-A | `crates/benten-eval/src/exec_state.rs` (new), `crates/benten-eval/src/evaluator.rs` (**only** the top-level `run` suspend/resume machinery + new `suspend_to_bytes` / `resume_from_bytes` entry points — attribution threading is partitioned into `evaluator/attribution.rs` owned by G5-B), `crates/benten-eval/src/primitives/wait.rs` (new executor) | `tests/wait_suspends_and_resumes`, `tests/exec_state_dagcbor_roundtrip`, `tests/exec_state_cid_deterministic` (proptest 10k instances), `tests/wait_crash_midway_recovers` |
| G3-B | `crates/benten-eval/src/time_source.rs` (new — placed in `benten-eval` per phil-4 since WAIT lives here and `benten-core` stays data-shape-only), `crates/benten-engine/src/engine_wait.rs` (new — suspension-handle persistence + resume orchestration; sibling module to `engine.rs` following the Phase-1 5d-K sibling-module pattern), `packages/engine/src/dsl.ts` (extend existing `wait()` stub so the DSL surface maps to the new Rust executor), `bindings/napi/src/wait.rs` (new — napi bridge for WAIT suspension/resume) | `tests/engine_wait_end_to_end`, `tests/time_source_mock_usable`, `tests/resume_after_engine_restart` (the headline exit criterion gate 1), TS Vitest `wait.test.ts` |

**Gates:** G7 (SANDBOX can't nest WAIT; STREAM composes on exec-state shape), napi bindings.
**Deliverables closed:** E2, E3, T1, N3.
**R1 review lens:** `chaos-engineer` (crash / partial-state), `determinism-verifier` (CID stability of state bytes), `security-auditor` (persisted state = new attack surface).

### G4 — Multiplicative iteration budget + evaluator-path Option C

**Agents:** 1 × `rust-implementation-developer` + 1 × `operation-primitive-linter` (mini-review: invariant-8 coverage across the compute graph)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G4-A | `crates/benten-eval/src/invariants/budget.rs` (invariant-8 multiplicative cumulative-budget computation lives here after G1's structural split; drop nest-depth-3 stopgap), `crates/benten-eval/src/primitives/read.rs` (wire `check_read_capability` into execute path), `crates/benten-eval/src/primitives/call.rs` + `iterate.rs` (multiplicative accumulation through frames) | `tests/invariant_8_multiplicative_through_call`, `tests/invariant_8_multiplicative_through_iterate`, `tests/read_primitive_option_c_symmetric`, `tests/call_respecting_cap_on_budget` |

**Gates:** G7 (cumulative budget feeds into SANDBOX fuel-budget composition).
**Deliverables closed:** E9, P3.

### G5-A — Immutability (Invariant 13) [parallel with G3; storage-layer, no evaluator coupling]

**Agents:** 1 × `rust-implementation-developer` + 1 × `security-auditor` (mini-review: Inv-13 is security-class)
**Parallelism:** yes — G5-A runs in parallel with G3 because Inv-13 lives in the storage layer (no evaluator-side threading required; G5-A's eval-side check is a registration-time static check that does not touch the frame machinery G3 is reshaping).

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G5-A | `crates/benten-eval/src/invariants/immutability.rs` (Inv-13 registration + eval-side static reject of WRITE-to-registered-subgraph), `crates/benten-engine/src/engine_crud.rs` (wire the 4-quadrant firing matrix per §9.11: privileged + content-matches = `Ok(cid_dedup)`; privileged + differ = `E_WRITE_CONFLICT`; unprivileged + match = `E_INV_IMMUTABILITY`; unprivileged + differ = `E_INV_IMMUTABILITY`), `docs/SECURITY-POSTURE.md` (append the 4-quadrant matrix section per arch-4) | `tests/invariant_13_no_write_to_registered_subgraph`, `tests/invariant_13_privileged_content_matches_dedups`, `tests/invariant_13_privileged_content_differs_write_conflict`, `tests/invariant_13_unprivileged_fires_immutability` |

**Gates:** G5-B + G7 (SANDBOX fires Inv-4 next to Inv-13); R4b.
**Deliverables closed:** E11 (eval side; storage backstop is G2-A).

### G5-B — System-zone full (Invariant 11) + Causal attribution structural (Invariant 14) [serial after G3]

**Agents:** 2 × `rust-implementation-developer` + 1 × `security-auditor` (mini-review: both are security-class invariants) + 1 × `code-as-graph-reviewer` (mini-review: structural-invariant lens)
**Parallelism:** G5-B is serial after G3-A because Inv-14 attribution threading lives in `evaluator/attribution.rs` and composes on the frame-stack shape G3-A establishes.

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G5-B-i | `crates/benten-eval/src/invariants/system_zone.rs` (Inv-11 full — registration-time literal-CID reject + runtime TRANSFORM-computed-CID reject via a static `phf` const-compiled HashMap of system-zone prefixes per §9.10; no Node-fetch path), `crates/benten-engine/src/engine_crud.rs` (remove E_SYSTEM_ZONE_WRITE stopgap once Inv-11 live — keep stopgap as fallback for Phase-1 bytes in storage), criterion bench `invariant_11_prefix_probe` gated at <1µs/check | `tests/invariant_11_system_zone_unreachable_user_path`, `tests/invariant_11_transform_computed_cid_rejected_at_runtime`, `tests/system_zone_stopgap_and_full_coexist`, criterion gated |
| G5-B-ii | `crates/benten-eval/src/invariants/attribution.rs` (Inv-14 structural), `crates/benten-eval/src/evaluator/attribution.rs` (new sub-module — attribution threading onto every TraceStep; disjoint from G3-A's `evaluator.rs` suspend/resume surface), `crates/benten-engine/src/outcome.rs` (TraceStep schema extended) | `tests/invariant_14_attribution_every_trace_step`, `tests/invariant_14_missing_attribution_is_registration_error` |

**Gates:** G7 (SANDBOX fires Inv-4 next to Inv-11/14); R4b.
**Deliverables closed:** E10, E12.

### G6 — STREAM + SUBSCRIBE (user-visible) [PHASE 2B]

> **Deferred to Phase 2b** — opens with its own pre-R1 after Phase 2a ships. Group shape preserved below for traceability; do not dispatch during Phase 2a R5.

**Agents:** 2 × `rust-implementation-developer` + 1 × `websocket-engineer` (SSE / back-pressure lens; the agent exists at `.claude/agents/websocket-engineer.md`) + 1 × `code-reviewer` (mini-review)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G6-A | `crates/benten-eval/src/chunk_sink.rs` (new trait), `crates/benten-eval/src/primitives/stream.rs` (new executor), `crates/benten-eval/src/primitives/subscribe.rs` (new executor — composes change-stream + chunk_sink) | `tests/stream_backpressure_engages`, `tests/stream_chunk_sequence`, `tests/subscribe_user_visible_routes_events`, `tests/subscribe_capability_gated` |
| G6-B | `crates/benten-engine/src/engine_stream.rs` (new), `bindings/napi/src/stream.rs` (new), `packages/engine/src/stream.ts` (new — DSL `stream()` + `subscribe()` builders) | `tests/engine_stream_end_to_end`, TS Vitest `stream.test.ts` (SSE over napi), TS `subscribe.test.ts` |

**Gates:** G7 (SANDBOX-streaming-output is a STREAM-composing primitive pattern).
**Deliverables closed:** E4, E5, N4, B2, B4, D1 (partial — stream + subscribe).

### G7 — SANDBOX + wasmtime host + Invariants 4 + 7 [PHASE 2B — longest pole]

> **Deferred to Phase 2b** — opens with its own pre-R1 after Phase 2a ships. Group shape preserved below for traceability. The §9.3 capability-derived host-function manifest decision carries forward into 2b as pre-decided architecture.

**Agents:** 3 × `rust-implementation-developer` + 1 × `wasmtime-sandbox-auditor` (mini-review + R1 review; agent exists at `.claude/agents/wasmtime-sandbox-auditor.md`) + 1 × `security-auditor` (mini-review — sandbox escape surface) + 1 × `performance-engineer` (instance-pool bench)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G7-A | `crates/benten-eval/src/sandbox/mod.rs` (new), `crates/benten-eval/src/sandbox/host_fns.rs` (host-function manifest + allowlist enforcement), `crates/benten-eval/src/sandbox/pool.rs` (instance pool keyed by module CID), `crates/benten-eval/src/primitives/sandbox.rs` (new executor) | `tests/sandbox_end_to_end`, `tests/sandbox_fuel_exhausts_routes_code`, `tests/sandbox_output_limit_routes_code`, `tests/sandbox_instance_pool_reuses`, `tests/sandbox_escape_attempts_denied` (adversarial fixture batch), `tests/sandbox_host_fn_manifest_denies_off_list` |
| G7-B | `crates/benten-eval/src/invariants.rs` (Inv-4 nest depth), fixture `.wasm` modules + build script `crates/benten-eval/tests/fixtures/build_wasm.sh` | `tests/invariant_4_sandbox_nest_depth_rejected_at_registration`, positive nest-1 + positive nest-2 + negative nest-3, `tests/fixture_wasm_hashes_stable` |
| G7-C | `crates/benten-engine/src/engine_sandbox.rs` (new — pool lifecycle + public API), `bindings/napi/src/sandbox.rs` (WASM-host detection — disables SANDBOX on wasm32-wasip1 with `E_SANDBOX_UNAVAILABLE_ON_WASM`), `packages/engine/src/sandbox.ts` (DSL `sandbox()` builder) | `tests/engine_sandbox_end_to_end`, `tests/sandbox_disabled_on_wasm32_wasip1_returns_typed_error`, `tests/pool_metrics_reuses_counted`, TS `sandbox.test.ts` |

**Gates:** R4b (longest pole); R6.
**Deliverables closed:** E6, E7, E8, B6, D1 (partial — sandbox).
**R1 review lens:** `wasmtime-sandbox-auditor`, `security-auditor`, `performance-engineer`.

### G8 — Generalized IVM Algorithm B + strategy selection + user views + metrics [PHASE 2B]

> **Deferred to Phase 2b** — opens with its own pre-R1 after Phase 2a ships. Group shape preserved below for traceability. Exception: the `view_stale_count` metric wire-up (currently hard-coded `0.0`) is a <10-line Phase-2a item absorbed into G11-A docs sweep rather than waiting for 2b.

**Agents:** 2 × `rust-implementation-developer` + 1 × `ivm-algorithm-b-reviewer` (mini-review + R1 review; agent exists) + 1 × `performance-engineer` (mini-review — Algorithm B must match hand-written within 20%)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G8-A | `crates/benten-ivm/src/algorithm_b.rs` (new — dependency tracker + per-input-CID invalidation), `crates/benten-ivm/src/strategy.rs` (new — `Strategy::{A,B,C}` tag + selection at register), `crates/benten-ivm/src/view.rs` (View trait extended to accept strategy), `crates/benten-ivm/benches/algorithm_b_vs_handwritten.rs` | `tests/algorithm_b_correctness_against_handwritten_view`, proptest `prop_algorithm_b_incremental_equals_rebuild` (100k), `tests/strategy_selection_respects_access_pattern`, criterion within 20% of hand-written baseline |
| G8-B | `crates/benten-engine/src/engine_views.rs` (`create_view` goes live — removes `TODO(phase-2-view-id-registry)` at `engine.rs:1549`) | `tests/user_registered_view_end_to_end`, `tests/user_view_pattern_mismatch_fires` (note: `view_stale_count_tallies` is owned by G11-A in Phase 2a scope; the metric wire-up is a <10-line 2a item, not 2b) |

**Gates:** R4b; R6.
**Deliverables closed:** I2, I3, I4, I5, I6.

### G9 — Wall-clock TOCTOU + iterate_batch_boundary delegation

**Agents:** 1 × `rust-implementation-developer` + 1 × `ucan-capability-auditor` (mini-review)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G9-A | `crates/benten-caps/src/policy.rs` (wall-clock + delegation), `crates/benten-eval/src/primitives/iterate.rs` + `call.rs` (consult delegated values), `crates/benten-engine/src/primitive_host.rs` (pass-through) | `tests/caps_wallclock_bound_refreshes_at_300s_default`, `tests/caps_iterate_batch_delegation_end_to_end`, `tests/long_running_transform_honors_wallclock` |

**Gates:** R4b.
**Deliverables closed:** P1, P2.

### G10 — WASM runtime build + network-fetch KVBackend stub + module manifest [PHASE 2B]

> **Deferred to Phase 2b** — opens with its own pre-R1 after Phase 2a ships. Group shape preserved below for traceability.

**Agents:** 2 × `rust-implementation-developer` + 1 × `napi-bindings-reviewer` (mini-review; agent exists) + 1 × `determinism-verifier` (mini-review — WASM reproduces canonical CID)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G10-A | `bindings/napi/src/wasm_target.rs` (cfg-gated runtime path), `crates/benten-graph/src/backends/network_fetch_stub.rs` (new), `.github/workflows/wasm-runtime.yml` (new) | `tests/wasm32_wasip1_canonical_cid_matches`, `tests/network_fetch_stub_errors_typed`, `tests/wasm_snapshot_blob_read_path` |
| G10-B | `crates/benten-engine/src/module_manifest.rs` (new), `crates/benten-engine/src/engine.rs` (`install_module`, `uninstall_module`), `packages/engine/src/manifest.ts` (new), manifest format doc `docs/MODULE-MANIFEST.md` (new) | `tests/module_install_persists_in_system_zone`, `tests/module_uninstall_respects_capability_retraction`, `tests/manifest_ts_validates_against_rust_schema` |

**Gates:** R4b; R6.
**Deliverables closed:** N2, B1, B5, D2.

### G11 — Docs sweep + devserver + `view_stale_count` wire-up + CI [PHASE 2A scope]

**Agents:** 1 × `rust-implementation-developer` + 1 × `documentation-engineer` + 1 × `dx-optimizer` (mini-review)

| Agent | Files owned | Must-pass tests |
|---|---|---|
| G11-A | `crates/benten-eval/src/**` (per-item docstring sweep for items 2a touches; drop `allow(missing_docs)` on items whose public API lands this phase — the full sweep waits for 2b), `tools/benten-dev/**` (new dev server watching handler TS files, recompiles subgraphs on change, invokes `engine.registerSubgraph` + `engine.call` against a live engine; closes backlog §8.1), `docs/DSL-SPECIFICATION.md` examples refresh for `wait()` (the one 2a executor that's newly live), `crates/benten-engine/src/engine_diagnostics.rs` (wire `view_stale_count` to tally `View::is_stale()` per subscriber view, replacing the `0.0` hard-code at `engine_diagnostics.rs:185` — this is a <10-line 2a item), `crates/benten-ivm/src/subscriber.rs` (tally helper if the subscriber side needs it), new CI workflow `.github/workflows/arch-1-dep-break.yml` (runs the workspace-wide `arch_1_dep_break_preserved` test from G1-A on every Phase-2+ PR per phil-7) | `tests/benten_eval_no_missing_docs_warnings_for_2a_items`, `tests/dev_server_hot_reloads`, `tests/view_stale_count_tallies`, `tests/arch_1_dep_break_preserved` (reads `benten-eval/Cargo.toml` at test time, asserts no `benten-graph` dep — per phil-7) |

**Gates:** R6 (docs completeness for 2a scope).
**Deliverables closed:** 2a portions of the DSL-examples refresh (wait only), `view_stale_count` metric, devserver (T2), missing_docs sweep for 2a-touched items. **Paper-prototype revalidation + full DSL examples refresh + full missing_docs sweep move to Phase 2b's G11-equivalent** because they need all 12 primitive executors live.

---

## 4. Test Landscape (R2 input)

Seeds the `benten-test-landscape-analyst` at R2. Organised by test type; R2 expands into per-file test requirements.

### 4.1 Unit tests — mandatory

Per-primitive-executor happy-path + every typed error edge for WAIT / STREAM / SUBSCRIBE / SANDBOX. Per-invariant positive + negative (both firing and permissive) for Inv-4, 7, 8 multiplicative, 11 full, 13, 14. Per-host-function-manifest entry: allowed call succeeds; off-list call denies. Per-capability-policy-shape: wall-clock + iterate-batch delegation works with both NoAuth default and GrantBackedPolicy.

### 4.2 Property-based (proptest targets)

- `prop_exec_state_dagcbor_roundtrip` — 100k runs: serialise → deserialise equals original; re-serialise-after-deserialise hashes to same CID
- `prop_sandbox_fuel_bounded` — arbitrary WASM fixture in fuel range, assert fuel strictly decreases and termination before underflow
- `prop_algorithm_b_incremental_equals_rebuild` — 100k runs: N random writes + random view shape, assert Algorithm B incremental state matches rebuild-from-scratch
- `prop_invariant_8_multiplicative_exact` — 10k runs of random CALL+ITERATE nesting, assert multiplicative cumulative budget equals sum-over-paths
- `prop_stream_backpressure_monotonic` — chunks consumed always ≤ chunks produced + buffer; backlog never unbounded
- `prop_subscribe_event_ordering` — within a single subscription, events are delivered in commit-order (Phase-2 single-engine; Phase-3 distributed is a different contract)

### 4.3 Integration tests (cross-crate) — mapped to Phase 2a's five exit gates (§1)

- **WAIT/resume end-to-end** (headline gate 1; including crash-restart case)
- **Four new invariants firing** — positive+negative pairs for Inv-8 multiplicative, Inv-11 full (both registration-time literal + runtime TRANSFORM-computed), Inv-13 immutability (4-quadrant matrix per §9.11), Inv-14 structural attribution (headline gate 2)
- **arch-1 dep break asserted** — workspace-wide `arch_1_dep_break_preserved` test reads `benten-eval/Cargo.toml` and asserts no `benten-graph` dep (bundled gate 3)
- **Option C evaluator-path symmetric-None** — `crud:post:get` dispatched through `Engine::call` honours symmetric-None end-to-end without a separate public-API gate (bundled gate 4)
- **Durability + AST-cache perf gate** — criterion `crud_post_create_dispatch_group_durability` p95 ≤ 300µs on macOS APFS; `subgraph_cache_hit` <10µs median (bundled gate 5)
- **Full invariant-14 coverage** over a 5-handler composed subgraph: every TraceStep carries attribution (contributes to headline gate 2)
- **User-registered view** incremental + pattern-mismatch — deferred to Phase 2b (G8); `view_stale_count` metric tallies is a 2a item in G11-A

### 4.4 Criterion benchmarks

- `sandbox_instance_pool_reuse_win` — must show ≥5x speedup over no-pool baseline for a 100-call workload
- `sandbox_fuel_accounting_overhead` — target ≤10% of raw WASM execution time
- `wait_suspend_resume_overhead` — target ≤50µs for the suspend-to-bytes + resume-from-bytes round-trip, excluding I/O
- `algorithm_b_vs_handwritten_content_listing` — within 20% of Phase-1 hand-written baseline
- `crud_post_create_dispatch_group_durability` — CI-gated now (Phase-1 was informational only) at ≤300µs median on macOS APFS with DurabilityMode::Group default
- `subgraph_cache_hit` — <10µs median with full AST-cache wire-through (Phase-1 informational target becomes gated)

### 4.5 Security-class tests (seed rust-test-writer-security)

- Sandbox escape attempts: direct memory access, host-function call outside manifest, fuel-free-loop, timer-based-side-channel probe, output-size-overflow attack, multiple fuel-refill attempts, re-entrancy-via-host-callback
- Invariant-4 nest depth adversarial: malformed nested subgraph trying to bypass via CALL-to-SANDBOX chain
- Invariant-11 system-zone bypass: attempt to READ a `system:capability_grant` Node via user subgraph
- Invariant-13 immutability bypass: re-registering a handler under a modified CID
- Invariant-14 attribution suppression: attempt to emit a TraceStep without `actor_cid`
- WAIT resume with corrupted state bytes: every decode failure returns a typed error, never panics
- STREAM back-pressure DoS: malicious sink that never consumes — must route to `E_STREAM_CONSUMER_STALL` after configurable timeout
- Module manifest signature forge attempt (Phase 3 ships signatures; Phase 2 reserves the slot)

### 4.6 CI workflow additions (Phase 2a)

- `.github/workflows/wait-resume-determinism.yml` — suspend-then-resume CID stability (headline gate 1)
- `.github/workflows/arch-1-dep-break.yml` — `benten-eval/Cargo.toml` never regains `benten-graph` dep (phil-7; bundled gate 3 + Phase-2+ CI guard)
- `.github/workflows/phase-2a-exit-criteria.yml` — composite job gating on gates 1–5 all green

(`sandbox-fuzz.yml` and `wasm-runtime.yml` are Phase 2b scope.)

---

## 5. Risk List + Open Questions (ranked)

### Rank 1 — wasmtime version pinning + API stability (SANDBOX long-pole)

wasmtime 35+ was validated in pre-work but the API surface shifts minor-version to minor-version (fuel API + component-model finalisation are active). G7 lands against one wasmtime version; a mid-Phase-2 bump that breaks the fuel-accounting API blocks G7 until re-paved.

**Fallback:** pin to a specific wasmtime minor (e.g. `wasmtime = "=35.0.2"`) during Phase 2; document upgrade as a Phase-2-close item in a new `phase-3-backlog.md`.
**Gate:** G7-A fuel-accounting test suite fails to compile against a new wasmtime minor.
**Owner:** G7-A + G7-C.

### Rank 2 — fuel-to-wall-clock mapping determinism

Wasmtime fuel is instruction-count; user-facing timeouts are wall-clock. Mapping is platform-dependent. Two fuel-equivalent calls can differ by 10x in wall-clock on ARM vs x86_64 vs WASM.

**Fallback:** expose fuel as the canonical budget; wall-clock is advisory. Document in `docs/SECURITY-POSTURE.md §Compromise #N+1`. No timing side-channel guarantees.
**Gate:** determinism test on fuel (must be identical across targets) vs wall-clock (must NOT be asserted identical).
**Owner:** G7 + R1 debate.

### Rank 3 — WAIT on-disk state format stability

`ExecutionState` DAG-CBOR shape becomes part of Benten's durable on-disk contract. A Phase-3 schema evolution (e.g. adding HLC fields) must round-trip old bytes. Options:

- Versioned envelope `{schema_version: u16, payload}` (recommended)
- Schema evolution via additive-only DAG-CBOR + tolerant decoder
- Never change shape — migration-only

**Fallback:** versioned envelope. CI test asserts old fixture bytes decode on new code.
**Gate:** fixture bytes at `crates/benten-eval/tests/fixtures/exec_state_v1.cbor` must decode on new code indefinitely.
**Owner:** G3-A + R1 debate (§9.1).

### Rank 4 — STREAM back-pressure across napi (Node.js ReadableStream ↔ wasmtime ↔ Phase-3-future-iroh)

Back-pressure semantics differ across the three layers. Node ReadableStream is pull-based with `highWaterMark`; wasmtime is synchronous (no native back-pressure); iroh (future) is push-based over QUIC with flow-control credits.

**Fallback:** Phase 2 implements pull-based back-pressure end-to-end. iroh integration in Phase 3 will need an adapter — flag this as a Phase-3 risk too.
**Gate:** `stream_backpressure_engages` test exercises a consumer that pauses for 100ms and asserts producer-side blocks observed.
**Owner:** G6-B + R1 debate (§9.4).

### Rank 5 — SANDBOX host-function surface shape (decided; residual risks)

Architecture settled pre-R1 — capability-derived with named-manifest DX sugar (see §9.3). The residual risks are implementation-level, not architecture-level:

- **Initial host-function set** — what ships in Phase 2b's minimum useful `host:compute:*` surface? Too thin → user-facing regressions vs hand-allowlist baseline. Too fat → attack surface + more invariant pairs to test.
- **Enforcement layer** — cap check at SANDBOX-init resolution time (cheap, happy path) OR at each host-fn invocation (stricter, catches mid-eval revocation) OR both. Dual-layer likely right: resolve-at-init as fast path, re-check at `iterate_batch_boundary`-equivalents for long-running SANDBOX calls (mirrors Compromise #1's TOCTOU refresh shape).
- **Named-manifest registry** — TOML file vs `benten-caps`-adjacent module vs inline constants.

**Fallback:** ship the minimum viable set (`host:compute:time`, `host:compute:log`, `host:random:read`) in Phase 2b G7; extend additively.
**Gate:** G7-A `sandbox_host_fn_cap_denied_when_cap_absent` + `sandbox_host_fn_cap_revoked_midcall_denied` green.
**Owner:** G7 + R1 debate (§9.3) for tactical refinements.

### Rank 6 — Evaluator-level Invariant 11 enforcement scope

"Unreachable from user operations" means the evaluator must reject CIDs that target system zones. But subgraphs can construct CIDs at runtime (via TRANSFORM on input). Full enforcement means runtime-checking every READ/WRITE target CID against a system-zone prefix table. Cost: ≥O(1) HashMap probe per primitive call.

**Fallback:** Phase-2 keeps the Phase-1 write-path stopgap as a backstop. Evaluator rejects at registration time for static-literal CIDs, at run-time for dynamically-computed CIDs. Document the dual-layer in `docs/SECURITY-POSTURE.md`.
**Gate:** both `static_system_zone_rejected_at_registration` and `dynamic_system_zone_rejected_at_runtime` green.
**Owner:** G5-A.

### Rank 7 — Immutability enforcement layer (evaluator vs storage)

Invariant 13 can fire at evaluator-level (WRITE primitive checks target is not a registered subgraph) or at storage-level (`put_node` rejects re-put). Evaluator-only is cheap but misses non-evaluator writes; storage-only is bulletproof but catches late.

**Fallback:** both layers. Evaluator rejects early for UX; storage rejects late as the security backstop.
**Gate:** each layer has its own firing test; the layers must never disagree.
**Owner:** G5-A + G2-A.

### Rank 8 — arch-1 dep break touch scope (15–30 file cross-cutting refactor)

G1 is a serial refactor with no visible user-facing change; if it slips, every downstream group accumulates against the coupled shape. Reviewer-availability risk.

**Fallback:** pair `rust-engineer` with the implementation developer up front; schedule `architect-reviewer` mini-review as a blocking gate before G3 dispatches.
**Gate:** `tests/arch_1_no_graph_dep.rs` green + full Phase-1 test suite still green + new workspace-wide `arch_1_dep_break_preserved` test (per phil-7) runs on every Phase-2+ PR via `.github/workflows/arch-1-dep-break.yml` — asserts `benten-eval/Cargo.toml` does not regain `benten-graph` as a dep.
**Owner:** G1 + G11-A (CI wire-up).

### Rank 8a — R1-escalated design questions gate G1 dispatch (arch-1 + arch-2 + phil-1/2/3/5)

Seven R1 agenda items (arch-1 ExecutionState payload shape, arch-2 HostError freeze, phil-1 Inv-14 structural-vs-policy, phil-2 Inv-11 placement, phil-3 HostError A/B/C, phil-5 Budget abstraction, arch-6 evaluator.rs partitioning) are triaged to R1 with concrete agenda; G1 dispatch cannot proceed until R1 locks HostError shape (phil-3). If R1 over-runs, downstream groups that depend on the stable host surface stall.

**Fallback:** pre-R1 triage already baked the envelope + capability-derived SANDBOX manifest decisions; R1's scope is narrower than the full agenda would suggest. Planner's nominal positions on each R1 item are recorded in §9.1, §9.2, §9.10, §9.11, §9.12 so R1 evaluates from a concrete proposal.
**Gate:** R1 exits with a decided position on each of the 7 items.
**Owner:** R1 (orchestrator + 7-seat council).

### Rank 9 — WASM target KVBackend semantics

Network-fetch KVBackend was previously spec'd as Phase 2; this plan re-defers to Phase 3 because it depends on iroh transport. But the WASM target still needs SOMETHING to read from. Options:

- Snapshot blob at engine construction (chosen): developer serialises a KV dump and passes it as `Uint8Array` to `initEngine`. Read-only, no writes
- No-storage mode: WASM engine rejects all reads
- Full iroh backend inlined (drags Phase 3 in)

**Fallback:** snapshot blob.
**Gate:** `wasm_snapshot_blob_read_path` test green.
**Owner:** G10-A + R1 debate (§9.8).

### Rank 10 — Paper-prototype re-measurement SANDBOX rate drift

E14 predicts SANDBOX rate < 30% on the revised 12. If measured rate exceeds 30%, either the primitive vocabulary is structurally insufficient (escalate to an R1 debate about a 13th primitive) or the handler selection was biased.

**Fallback:** defer the measurement-acting consequence to a Phase-2-close or Phase-3 decision. Revalidation is an artifact, not a go/no-go.
**Gate:** artifact exists + SANDBOX-rate number documented honestly.
**Owner:** G11-A.

### Rank 11 (lower) — Preexisting deferrals (verify-only)

Backlog items that closed before this plan locked: `Cid::from_str` (F-R7-004 landed in commit `a619930`), `benten-errors` crate extraction (Compromise #3, commit `d03f642`), `DSL-SPECIFICATION.md` revised-primitives rewrite (F-R7-002, commit `6cea1e2`), `ARCHITECTURE.md` seven-crate update (F-R7-003, commit `93b294c`), scaffolder-drift reconciliation in plan §1 (F-R7-001, commit `93b294c`). Upstream `rust-cid`/`rust-multihash` PR merges + workspace `[patch.crates-io]` removal remain open. At Phase-2 open, re-verify state and remove from inventory any that have landed since.

**Gate:** pre-R1 verification pass.
**Owner:** pre-R1 (orchestrator).

---

## 6. ADDL Stage Dispatch Plan

Per `docs/DEVELOPMENT-METHODOLOGY.md` Pattern 6 (Reviewer composition follows lens surface), Phase 2's lens surface is NOT Phase 1's lens surface. Phase 1 optimised for: code-as-graph thesis, content-addressing determinism, capability-policy-as-data, thin-engine philosophy. Phase 2's new lenses: wasmtime isolation correctness, wasm32-wasip1 target determinism, suspendable evaluator state (cross-cutting chaos), STREAM back-pressure, generalised IVM algorithm correctness.

| Stage | Agents | Output | Gate |
|---|---|---|---|
| **Pre-R1 critics (ran 2026-04-21)** | `benten-engine-philosophy`, `architect-reviewer`, `code-reviewer` (3 critics — composition settled on thin-engine + dep-graph + plan-coherence lenses for a planning-stage artifact) | 3 JSON findings at `.addl/phase-2a/pre-r1-<agent>.json`; triaged to `.addl/phase-2a/pre-r1-triage.md` | All three PASS_WITH_FINDINGS; 1 critical + 9 major + 17 minor — 1 critical + 7 major fixed-now; 7 majors escalated to R1 |
| **R1 spec review** (agent-team mode, peer-debate; Phase 2a lens surface — 7 seats per triage doc) | `architect-reviewer` (cross-phase coherence; carried from pre-R1), `benten-engine-philosophy` (thin-engine thesis; carried from pre-R1), `code-reviewer` (tactical correctness; carried), `security-auditor` (persisted ExecutionState is a new attack surface; TOCTOU delegation is security-class), `ucan-capability-auditor` (cross-phase Phase-3 sync / cap-chain lens for arch-1 HostError + ExecutionState attribution carry-through), `code-as-graph-reviewer` (Inv-11/13/14 enforcement is its direct domain), `dx-optimizer` (WAIT developer ergonomics + devserver hot-reload surface) | 7 JSON findings at `.addl/phase-2a/r1-<agent>.json` | Ben triages every finding |
| **R2 test landscape** | `benten-test-landscape-analyst` (1 agent, lead session) | `.addl/phase-2a/r2-test-landscape.md` — invariant targets + proptest targets + bench targets + CI workflows; seeds the 7 Phase 2a groups' test files (G1, G2, G3, G4, G5-A, G5-B, G9, G11-2a) | Ben reviews for completeness |
| **R3 test writing** (TDD contract) | 5 parallel subagents: `rust-test-writer-unit`, `rust-test-writer-edge-cases`, `rust-test-writer-security` (Inv-11/13/14 adversarial tests), `rust-test-writer-performance` (the 4 2a bench targets — no SANDBOX/Algorithm-B in 2a), `qa-expert` (integration — the 5 exit-criteria gates) | Rust test files + TS Vitest files in place, failing (no implementations) | Test suite compiles + failures match scope-inventory count |
| **R4 test review (pre-impl)** | `rust-test-reviewer`, `rust-test-coverage` | JSON at `.addl/phase-2a/r4-<agent>.json` | Fix findings; gate R5 |
| **R5 implementation** | 7 groups (G1, G2, G3, G4, G5-A, G5-B, G9, G11-2a per §3 — note G5 is split per arch-3; G6/G7/G8/G10 are Phase 2b). Per group: N × `rust-implementation-developer` + `cargo-runner` + 1–2 mini-review agents scoped to the group's lens (see per-group §3 listings) | Commits per group. `.addl/phase-2a/r5-gN-cargo.log` + `.addl/phase-2a/r5-gN-mini-<agent>.json` | Full test suite green after each group; Ben triages mini-review findings |
| **R4b post-impl test review** | `rust-test-reviewer`, `rust-test-coverage`, `qa-expert` | JSON at `.addl/phase-2a/r4b-<agent>.json` | Catches vacuous passes; gate R6 |
| **R6 quality council** (14 seats — Phase 2a lens surface; no `wasmtime-sandbox-auditor` or `ivm-algorithm-b-reviewer` since those are 2b scope) | `architect-reviewer`, `benten-engine-philosophy`, `code-reviewer`, `security-auditor`, `ucan-capability-auditor`, `code-as-graph-reviewer`, `dx-optimizer`, `chaos-engineer`, `determinism-verifier`, `qa-expert`, `test-automator`, `performance-engineer`, `refactoring-specialist`, `error-detective` | 14 JSON reports at `.addl/phase-2a/r6-<agent>.json` | Zero critical/major remaining |

### Agents to create just-in-time

No NEW agent types need creating for Phase 2a. The Phase 2a lens-surface agents all exist:

- `ucan-capability-auditor` ✓
- `code-as-graph-reviewer` ✓
- `determinism-verifier` ✓
- `chaos-engineer` ✓
- `dx-optimizer` ✓

(`wasmtime-sandbox-auditor`, `ivm-algorithm-b-reviewer`, `napi-bindings-reviewer`, `websocket-engineer` also exist — they're Phase 2b lens-surface agents, not Phase 2a.)

If Phase 2a surfaces a gap (e.g. a `primitive-executor-designer` for WAIT suspendable-state design), create only after a concrete gap is named by pre-R1 or R1 critics. **Do not create speculatively.**

### Stage parallelism (Phase 2a scope only)

- Pre-R1 + R1 + R2 strictly sequential (each gates the next).
- R3's 5 writers run in parallel.
- R5 groups (Phase 2a): G1 strictly first (dep break + invariants module split). G2 + G3 + G5-A + G9 can run in parallel after G1 (disjoint file sets; G5-A is storage-layer-only and independent of G3's evaluator reshape per arch-3). G4 depends on G3 (shares multiplicative-budget accumulation plumbing). G5-B depends on G3 (attribution threading uses the frame-stack shape). G11-2a runs last.
- G6, G7, G8, G10 are Phase 2b scope and dispatch after Phase 2a ships with their own pre-R1.
- R4b + R6 sequential after R5.

---

## 7. Sequencing + Calendar (Phase 2a scope only)

Days of work for Phase 2a (Ben's Phase 1 velocity was ~3 days actual work for ~17 days HE):

```
Pre-R1 critics         0.5 day   ◄ DONE (2026-04-21)
R1 peer-debate         0.5 day
R1 triage + plan fix   0.5 day
R2 test landscape      0.5 day
R3 test writing        1 day     (5 writers in parallel; 2a scope is narrower)
R3 consolidation       0.25 day
R4 review              0.25 day
R4 fixes               0.25 day
R5 G1 (arch-1 + inv split)  1 day   ◄ serial — longest 2a item
R5 G2 + G3 + G5-A + G9      1 day   (parallel; G5-A is storage-layer, independent)
R5 G4                       0.5 day (after G3)
R5 G5-B                     0.5 day (after G3; Inv-11 + Inv-14)
R5 G11-2a                   0.25 day
R4b                    0.25 day
R6                     0.5 day
R6 triage              0.25 day
──────────────────────────────────
Total                  ~5–6 days of human-equivalent work
```

Actual wall-clock at Phase-1 velocity ⇒ ~1.5–2 days wall-clock.

**Longest pole: G1 (arch-1 dep break + invariants module split).** No 2a group has the wasmtime-API-stability risk profile of 2b's G7. If G1 slips, every downstream 2a group queues against it (per Rank 8).

---

## 8. Scope Cut — **DECIDED pre-R1: Split 2a / 2b**

- **Phase 2a (this plan)** — evaluator completion + arch-1 + Inv 8/11/13/14 + Option C evaluator-path + wall-clock TOCTOU. Groups G1 (arch-1), G2 (durability + AST), G3 (WAIT), G4 (Inv-8 + Option C), G5 (Inv-11/13/14), G9 (wall-clock TOCTOU), G11 (docs + devserver + view_stale_count). ~5 days human-equivalent.
- **Phase 2b (follow-on; opens own pre-R1 after 2a ships)** — SANDBOX + wasmtime + generalised Algorithm B + module manifest + WASM runtime + STREAM/SUBSCRIBE. Groups G6 (STREAM/SUBSCRIBE), G7 (SANDBOX + wasmtime + Inv 4/7), G8 (Algorithm B), G10 (WASM + manifest), plus a 2b-only G11 wrap (paper-prototype revalidation + full missing_docs sweep). ~12 days human-equivalent.

**Rationale for the split:**

- **Review-lens non-overlap.** `wasmtime-sandbox-auditor` is the correct R1 + R6 lens for SANDBOX work but brings zero expertise to Invariant 13/14 structural enforcement. `code-as-graph-reviewer` is right for Inv 13/14 but doesn't sensibly review wasmtime instance-pool design. One mega-council over both surfaces dilutes each lens.
- **Blast-radius isolation.** Phase 2a's scope is well-known (debt close + invariants); Phase 2b has unknown-unknowns (wasmtime API stability, host-function design, instance-pool mechanics). Splitting lets 2a ship independent of 2b's risk profile.
- **Feedback cycle.** 2a lands → we see "did the 6 new invariants catch the attacks the Phase-1 R7 audit anticipated?" before committing to 2b's more speculative work.
- **Scope cohesion.** Arch-1 + WAIT + invariants are all evaluator-structural work. SANDBOX + wasmtime + STREAM + Algorithm B + WASM are all isolation-and-compute work. Two natural units.

**Cross-2a/2b frozen interfaces.** G3's `ExecutionState` DAG-CBOR envelope (§9.1) is the only shape G6 STREAM will compose on. Freezing its contract at 2a close prevents 2b from churning G3's work.

**Roadmap doc updates** rolled into pre-R1 triage: `docs/FULL-ROADMAP.md` Phase 2 section gets a Phase 2a + 2b split; `CLAUDE.md` status table updates; `docs/future/phase-2-backlog.md` header acknowledges the split.

---

## 9. Open design decisions for R1 to debate

### 9.1 `ExecutionState` on-disk format — **DECIDED (pre-R1): Option A (DAG-CBOR + CIDv1 envelope)**

- Option A (chosen): DAG-CBOR with versioned envelope `{schema_version: u8, payload: ExecutionState}`. Re-uses content-hash infrastructure; CID-stable; migration path by envelope bump.
- Option B: A new stable-binary shape (e.g. borsh) decoupled from content-hash infra.
- Option C: JSON for debuggability, accepting the determinism loss.

**Rationale for A baked in pre-R1:** Suspended executions cross trust boundaries in later phases and every such crossing in Benten uses content-addressing. Specifically:

- **Phase 3 sync:** Atrium replicas may observe a suspended workflow; content-addressing is how everything else replicates. A non-addressed ExecutionState is the one structural outlier.
- **Phase 6 AI assistants:** multi-turn agent workflows ARE suspended handlers. The assistant forks / branches / resumes conversation state; CID-addressing makes these deterministic and shareable.
- **Phase 7 Gardens:** approval / voting workflows suspend awaiting signals (vote quorum, review). Signatures on suspended state reference the CID; the envelope is the natural signature payload.
- **Content-hash determinism (ENGINE-SPEC §7)** requires canonical encoding — DAG-CBOR is canonical-by-default, JSON is not. Option C forces app-level canonicalization and regresses the determinism thesis.

The envelope shape is `{schema_version: u8 = 1, payload: dag-cbor-encoded ExecutionState}` rolled through the same `Node::cid` machinery as everything else. Phase 2+ can bump `schema_version` additively; resumption from an older envelope rejects if the reader doesn't understand the version (typed error, not a panic).

**What remains open for R1 to debate (arch-1 escalation):** The envelope is decided (both pre-R1 critics confirmed sound); the **payload shape is a Phase-3 / Phase-6 / Phase-7 consumer-lens R1 debate.** Specifically: (a) how the attribution triple `(actor_cid, handler_cid, capability_grant_cid_used)` carries through suspension and is reinstated on resume; (b) whether the persisted state pins the set of subgraph CIDs it references (so resume on a different Atrium replica with different registered subgraphs fails loudly rather than silently); (c) how context-binding `Value`s that reference other Nodes by CID are resolved on resume (fetch on demand vs snapshot inline). `ucan-capability-auditor` is in the R1 composition for the cap-chain-carry-through lens. See triage doc for the full agenda.

**Debuggability concern (Option C motivation) addressed separately:** `benten-dev inspect-state <path>` ships in G11 (dev-server group) as a pretty-print command. JSON-equivalent readability without paying a format cost.

### 9.2 `HostError` variant design (arch-1 outcome) — **R1 DECIDES (escalated per phil-3 + arch-2)**

What shape replaces `EvalError::Graph(#[from] GraphError)`?

- **Option A — opaque `HostError(Box<dyn StdError + Send + Sync>)` + stable `ErrorCode` classification field.** Philosophy-preferred: eval stays thin, any downstream host-error variant (Phase-3 sync hash-mismatch, HLC drift, cap-chain invalid) boxes through without eval-crate retrofit.
- **Option B — enum of known host-error variants** (NotFound / WriteConflict / BackendUnavailable / …). Architect-critic flagged this as recreating the `GraphError` coupling under a new name — arch-1's structural win is weakened if eval still pattern-matches on host-error variants.
- **Option C — `ErrorCode`-only, no Rust enum** (host errors flow purely as stable string codes). Maximum decoupling but loses typed-error ergonomics inside eval.

**R1 must decide and lock the shape before G1 proceeds.** R1 agenda must weigh Phase-3 sync error-mapping downstream (HostError is the contract every Phase-2+ primitive and every Phase-3 sync error-mapping consumes). `ucan-capability-auditor` is in the R1 composition specifically for this lens.

**Planner's nominal position (subject to R1):** A + stable ErrorCode field. Boxed shape preserves the thin-engine thesis and carries Phase-3 sync errors forward without re-opening G1.

### 9.3 SANDBOX host-function manifest shape — **DECIDED (pre-R1): capability-derived with named-manifest DX sugar**

**Architectural primitive:** host functions are identified by capability. Each host function declares its own `requires: "host:<domain>:<action>"` scope (e.g. `host:compute:time`, `host:compute:log`, `host:random:read`). At SANDBOX init the engine intersects (host-supported functions) × (caller's cap grants) and exposes only the intersection. A SANDBOX whose caller lacks `host:compute:log` cannot invoke `log`.

**DX sugar (not a second security model):** named manifests are conveniences that resolve to cap bundles. `"compute-basic"` := `host:compute:time + host:compute:log + host:random:read`. A developer can declare `sandbox({ module, manifest: "compute-basic" })` and the engine grants the bundled caps for that call. Named manifests live in a registry of well-known shapes; they're lookup-tables over caps, not a parallel permission system.

**Rationale for capability-derived baked in pre-R1:**

- **Single security model.** Benten already has one security system — UCAN-compatible capability grants with pluggable policies. A parallel hand-allowlist for SANDBOX host functions would violate the "thin engine, compose on top" thesis and require its own revocation / attenuation / TOCTOU machinery. That's the exact duplication the Phase 1 `benten-caps` extraction was meant to prevent.
- **Phase 8 Credits compute marketplace** (`docs/FULL-ROADMAP.md` Phase 8, `docs/future/compute-marketplace.md`): third parties run code on your hardware. That path REQUIRES strict, user-grantable, fine-grained host-function control. A tiered system with three levels ("pure / read-only / write") cannot express "this agent can call time + log + network:outbound-to-specific-host but nothing else." Cap-derived can.
- **Phase 6 AI assistant tool generation:** the assistant has caps; it generates a subgraph with SANDBOX; CALL isolation + attenuation narrows what the generated SANDBOX inherits. This is the already-shipped CALL semantics (Phase 1) and extends naturally to SANDBOX. Hand-allowlist can't attenuate.
- **Phase 7 Gardens moderation:** community members run modules; operators grant narrow host-function scopes per trust level. Same cap system that gates data access gates host-function access.
- **Revocation:** mid-evaluation cap revocation already exists (Compromise #1 TOCTOU refresh at commit / CALL entry / ITERATE batch boundary). Cap-derived extends this to host-function access for free. Hand-allowlist would need a separate revocation path.

**What remains open for R1 to debate:**

- The initial *set* of host functions shipped with Phase 2b (what's the minimum useful `host:compute:*` surface?)
- Default named manifests — what bundles ship ("compute-basic", "compute-network"?). This is spec work for R1, not architecture work.
- Enforcement layer — is the cap check at host-function invocation time or at SANDBOX-init resolution time? (Resolution-time intersection is cheaper, invocation-time check is stricter against mid-eval revocation. Probably both, with resolution-time as the happy path and invocation-time re-check at `iterate_batch_boundary`-equivalents for long-running SANDBOX calls.)
- Named-manifest registry shape — TOML in the workspace, or a `benten-caps`-adjacent module? R1 decision.

**Why not tiered:** tiered is a subset of capability-derived (named manifests ARE tiers with different names). The cost of going cap-derived from the start is the resolution-layer machinery; the cost of going tiered-first and migrating later is retro-fitting cap checks into every host-function call site. Do it right the first time.

**Why not hand-allowlist:** simplest, most inflexible, foreclosed by Phase 6–8. Not a live option.

### 9.4 STREAM back-pressure semantics

See Rank 4.

- Option A: pull-based, bounded channel end-to-end
- Option B: push with producer-side flow-control credits (iroh-style, forward-compat with Phase 3)
- Option C: hybrid (pull at napi, push internally)

**Planner's preference:** A for Phase 2. Phase 3 revisits when iroh integration forces the question.

### 9.5 SANDBOX fuel-to-instruction mapping

See Rank 2. Specific ratio (e.g. 1 fuel = 1 wasm instruction, or 1 fuel = 1 wasm-bytecode-basic-block) is a wasmtime-level decision that affects user-facing budget predictability.

**Planner's preference:** R1 debate — recommend defer to wasmtime-sandbox-auditor + security-auditor at R1.

### 9.6 Module manifest format

- Option A: TOML (Cargo-familiar; human-editable)
- Option B: JSON (ubiquitous; validates cleanly) — **default for Phase 2a (phil-6).** Phase 2a does not touch module manifest (manifest work is Phase 2b scope), but the default goes in now so 2b pre-R1 evaluates a decided position.
- Option C: A new Benten-native content-addressed shape (manifest IS a subgraph; manifest CID IS the module identity) — **reserved for Phase 2b R1** when the evaluator primitive surface is complete; Option C requires all 12 executors live to be coherent (manifest-as-subgraph composes on STREAM + SUBSCRIBE + SANDBOX).

**Planner's position:** Option B (JSON) for Phase 2a lock-in; Option C re-opened at 2b R1. Phase 2a's narrow scope does not exercise the manifest-as-subgraph affordance, so committing to C now would foreclose on an unproven design.

### 9.7 Engine nested-transaction lift

See N7. Does Phase-2 lift `E_NESTED_TRANSACTION_NOT_SUPPORTED`?

**Planner's preference:** defer. Composed subgraphs give atomicity without nested-txn semantics. No Phase-2 use case requires nesting.

### 9.8 WASM target KVBackend semantics

See Rank 9.

**Planner's preference:** snapshot-blob read-only. Phase 3 adds iroh.

### 9.9 Paper-prototype revalidation: single or multi-sample?

E14 re-measurement: run against the original 5 handlers (apples-to-apples) or against a new 20-handler corpus (broader)?

**Planner's preference:** both — 5-handler for apples-to-apples; 20-handler as a broader reality check. Artifact documents both numbers. (Note: E14 is Phase 2b scope since it needs all 12 executors live; recorded here for continuity with the original §9 numbering.)

### 9.10 Invariant-11 runtime check cost ceiling — **DECIDED (fix-now per arch-5)**

The §6 Rank 6 "Evaluator-level Invariant 11 enforcement scope" risk flagged TRANSFORM-computed-CID runtime checks as a potential per-primitive-call cost. G5-B commits to:

- **Implementation:** a static `phf` const-compiled `HashMap<&'static str, ()>` of system-zone prefixes (e.g. `system:capability_grant`, `system:ivm_view`, `system:wait_pending`, `system:module_manifest`). The prefix set is compile-time-known; `phf` gives amortised O(1) probe with no heap allocation.
- **No Node-fetch path.** The check is a string-prefix probe against the target CID's label, not a storage lookup. This is what makes the cost ceiling achievable.
- **Cost gate:** criterion bench `invariant_11_prefix_probe` must land at <1µs/check on dev hardware; CI-gated.
- **Owner:** G5-B-i.

Rationale for baking this: architect-critic flagged that "runtime Inv-11 check" without a concrete cost ceiling is the kind of ambiguity that grows into a perf regression downstream. Committing to a static-table probe now forecloses on the expensive-alternative path.

### 9.11 Invariant-13 4-quadrant firing matrix — **DECIDED (fix-now per arch-4)**

Inv-13 (immutability) fires in four combinations of `(privileged, content-matches)`; each quadrant has a distinct correct outcome. Committed to Phase 2a spec to eliminate the ambiguity pre-R1:

| Privileged | Content matches registered bytes | Outcome |
|---|---|---|
| yes | yes | `Ok(cid_dedup)` — write is a no-op content-addressed dedup; returns the existing CID |
| yes | no (bytes differ) | `E_WRITE_CONFLICT` — privileged write attempted to overwrite with different content; storage-layer CAS rejects; caller retries if appropriate |
| no | yes | `E_INV_IMMUTABILITY` — unprivileged re-put even of matching bytes is still a policy violation (users cannot observe dedup on system-controlled surfaces) |
| no | no | `E_INV_IMMUTABILITY` — the canonical unprivileged immutability violation |

`WriteContext::privileged` is set by the engine orchestrator when the write originates from a capability-grant-authorised version-chain `NEXT_VERSION` append (Phase-1 surface); never by a user subgraph. The 4-quadrant matrix is appended to `docs/SECURITY-POSTURE.md` as part of G5-A's deliverable set. Each quadrant has its own firing test in G5-A's must-pass list.

### 9.12 Shared `Budget` abstraction across Inv-8 / Inv-4 / Inv-7 — **R1 DECIDES (escalated per phil-5)**

Phase 2a's G4-A ships Inv-8 multiplicative-through-CALL. Phase 2b's G7-A ships Inv-4 SANDBOX nest-depth + Inv-7 SANDBOX output-size + SANDBOX fuel-budget. R1 must decide whether these share a common `Budget` trait (or similar abstract shape) or remain independent:

- **Shared:** 2a locks the abstract shape in G4-A. Pro: Phase-2b SANDBOX fuel composes on the same machinery the evaluator already walks for CALL/ITERATE. Con: abstracting before all three budgets are concrete risks a premature abstraction (Phase-1 criticism of over-abstraction before real-world usage).
- **Independent:** 2b retrofits if a common shape emerges. Pro: no premature abstraction. Con: 2b may have to reshape G4-A's work.

**R1 agenda:** which shape? If shared, what's the minimum interface? If independent, is there a lightweight "convergence point" (e.g. both report via the same trace-span kind) that gets us most of the downstream benefit without the abstract trait?

**Planner's nominal position:** defer to R1. The thin-engine philosophy pressure is toward "don't abstract until 2b reveals the shape," but determinism-verifier + architect lenses at R1 may see structural reasons to commit now.

---

## 10. Appendix — Deliverable-to-Group Traceability

| Deliverable | Group |
|---|---|
| C1 AnchorStore | G11 (covered as part of eval docs + anchor surface audit) or G3 if WAIT needs bulk anchor access — **R1 decision** |
| C2 Cid::from_str | verify at pre-R1 (closed Phase-1 per F-R7-004) |
| C3 upstream unpins | monitor-only, lands opportunistically in G2 if upstream releases |
| C4 Node::load_verified | G2-A (storage-adjacent) |
| C5 Subgraph deterministic DAG-CBOR | G5-A (invariants-adjacent) |
| G1 DurabilityMode::Group default | G2-A |
| G2 AST cache completion | G2-B |
| G3 change-stream cap gate | re-defer Phase 3 |
| G4 immutability write-path | G2-A |
| G5 event log replay | re-defer Phase 3 |
| I1 View trait | already-present Phase 1 |
| I2 Algorithm B | G8-A |
| I3 per-view strategy | G8-A |
| I4 user-registered views | G8-B |
| I5 E_IVM_PATTERN_MISMATCH on user views | G8-B |
| I6 view_stale_count metric | G8-B |
| P1 wall-clock TOCTOU | G9-A |
| P2 iterate_batch_boundary delegation | G9-A |
| P3 Option C READ primitive path | G4-A |
| P4 UCAN backend | re-defer Phase 3 |
| E1 arch-1 dep break | G1-A |
| E2 ExecutionState serialise | G3-A |
| E3 WAIT executor | G3-A |
| E4 STREAM executor | G6-A |
| E5 SUBSCRIBE executor (user-visible) | G6-A |
| E6 SANDBOX executor + wasmtime | G7-A |
| E7 Invariant 4 | G7-B |
| E8 Invariant 7 | G7-A |
| E9 Invariant 8 multiplicative | G4-A |
| E10 Invariant 11 full | G5-B (split per arch-3) |
| E11 Invariant 13 | G5-A + G2-A (split per arch-3; G5-A is storage-layer parallel with G3) |
| E12 Invariant 14 structural | G5-B (split per arch-3) |
| E13 missing_docs sweep | G11-A |
| E14 paper-prototype revalidation | G11-A |
| N1 PrimitiveHost expansion | across G3 / G6 / G7 (each new primitive extends the trait; changes coordinated) |
| N2 Module manifest | G10-B |
| N3 WAIT public API | G3-B |
| N4 STREAM + SUBSCRIBE public API | G6-B |
| N5 Engine::builder().production() evolved | G7-C (requires SandboxPolicy) |
| N6 SubgraphCache wire-through | G2-B |
| N7 nested-tx lift decision | R1 only — likely deferred |
| B1 wasm32-wasip1 runtime | G10-A |
| B2 STREAM napi bridge | G6-B |
| B3 WAIT napi surface | G3-B |
| B4 SUBSCRIBE napi surface | G6-B |
| B5 network-fetch KVBackend stub | G10-A |
| B6 SANDBOX napi + WASM disable | G7-C |
| D1 DSL builders for new primitives (+ doc examples refresh) | G3-B, G6-B, G7-C; examples wrap-up in G11-A |
| D2 Module manifest TS types | G10-B |
| X1 Phase-2 ErrorCode variants | added per group that owns the firing site (G5, G7, G3) |
| X2 T7 codegen rerun | G11-A or automatic per-group |
| T1 TimeSource trait | G3-B |
| T2 dev server | G11-A |
| T3 SANDBOX fuzz CI | G7-C |
| T4 wasm32-wasip1 CI | G10-A |
| T5 CLAUDE.md drift lint | G11 (low priority) |
| T6 DEV-METHODOLOGY updates | G11 |

Every deliverable has an owner group or a documented re-deferral. No orphans.

---

## 11. Phase-1 Deferred-Items Incorporation Map

Per-item mapping from `docs/future/phase-2-backlog.md` → Phase-2 landing.

| Backlog § | Item | Phase-2 landing | Rationale |
|---|---|---|---|
| 1.1 | arch-1 dep break | G1 (first, serial) | Land before new primitive work per backlog rationale |
| 1.2 | `transaction.rs` 711-line file split | `transaction.rs` at 711 lines — split trigger is when Phase 2 adds enough transaction machinery that it crosses ~1200 lines. Re-defer to Phase 3 if the file is still under that threshold at Phase 2 close. | Logical seam only emerges at ~1200 lines |
| 1.2 | `engine.rs` further splits | re-defer to Phase 3 | Phase-1 5d-K split already shipped 4 sibling modules |
| 2 (WAIT/STREAM/SUBSCRIBE/SANDBOX) | all four | G3 / G6 / G7 | Phase 2's core deliverable |
| 3 (Invariants 4/7/8/11/13/14) | all six | G4 / G5 / G7 | Phase 2's core deliverable |
| 4.1 | Option C evaluator-path | G4-A | Threads `check_read_capability` into READ primitive |
| 4.2 | change-stream subscribe cap-gate | re-defer to Phase 3 | Requires cross-trust-boundary story |
| 4.3 | Compromise #6 BLAKE3 PQ reconsider | re-defer to Phase N+ | Post-quantum transition not a committed requirement yet |
| 5.1 | Generalised Algorithm B | G8-A | Phase 2's IVM deliverable |
| 5.2 | `E_IVM_PATTERN_MISMATCH` firing | **CLOSED Phase-1** per R7 audit | verify-only at pre-R1 |
| 5.3 | `view_stale_count` metric | G8-B | Cheap wire-up |
| 5.4 | IVM rebuild from event log | re-defer to Phase 3 | Needs sync-era consistency story |
| 6.1 | `Cid::from_str` | **CLOSED Phase-1 R7** (F-R7-004) | verify-only |
| 6.2 | `get_node_verified` | G2-A (as C4) | Storage-adjacent |
| 6.3 | Anchor-store consolidation | G11 (docs + small API) or G3 (if WAIT needs bulk) | R1 decision |
| 6.4 | Upstream multiformats unpins | monitor-only — land in G2 opportunistically | Blocked on upstream |
| 6.5 | `E_CID_UNSUPPORTED_*` wiring | **CLOSED Phase-1** (9b45c75) | verify-only |
| 7.1 | Wall-clock + iteration TOCTOU | G9-A | |
| 7.2 | UCAN backend | re-defer Phase 3 | Ships with `benten-id` (Phase 3) |
| 8.1 | Dev server hot reload | G11-A | |
| 8.2 | CLAUDE.md numeric-claim drift lint | G11 (low priority) | |
| 8.3 | Per-item `missing_docs` sweep | G11-A (E13) | Post-evaluator-completion |
| 9.1 | macOS APFS fsync floor / Group durability default | G2-A | Headline-target-reaching |
| 9.2 | Subgraph AST cache | G2-B | Perf leak closed |
| 10 | ~180 TODO(phase-2-*) markers | Distributed per their file's owning group. During R5 each group's file-ownership maps implicitly covers its markers. G11-A does a final sweep + closes any stragglers or explicitly re-defers. | |

Every backlog item is either absorbed into a Phase-2 group, explicitly closed pre-R1, or re-deferred with rationale. Nothing is left ambiguous.

---

## Ready for R1 spec review

**Pre-R1 outcome (ran 2026-04-21):** three critics — `benten-engine-philosophy`, `architect-reviewer`, `code-reviewer` — all returned **PASS_WITH_FINDINGS** (confidence 0.78–0.82). Both baked decisions (DAG-CBOR+CIDv1 ExecutionState envelope; capability-derived SANDBOX manifest) confirmed **sound** by all three. Scope split (2a / 2b) confirmed **natural joint**. Findings total: 1 critical + 9 major + 17 minor = 27. Triage result: 1 critical + 7 major fixed-now in this revised plan; 7 majors escalated to R1 with concrete agenda; 17 minors all fixed-now. See `.addl/phase-2a/pre-r1-triage.md` for full detail.

**Scope cut decided pre-R1:** split Phase 2a / 2b on review-lens-coherence grounds. Phase 2a (this plan as trimmed) ships arch-1 dep break + WAIT + 4 new invariants (8 multiplicative + 11 full + 13 immutability + 14 structural attribution) + TOCTOU + evaluator-path Option C (~5–6 days HE); Phase 2b ships SANDBOX + wasmtime + STREAM/SUBSCRIBE + Algorithm B + WASM runtime + module manifest (~12 days HE) with its own pre-R1 opening after 2a ships.

**Load-bearing architectural decisions baked in pre-R1 (now with critic confirmation):**

1. **`ExecutionState` on-disk format = DAG-CBOR + CIDv1 envelope** (§9.1) — envelope decided; payload shape escalated to R1 (arch-1). Preserves content-addressing symmetry for Phase 3 sync, Phase 6 AI-assistant workflow forking, Phase 7 Garden approval flows.
2. **SANDBOX host-function manifest = capability-derived with named-manifest DX sugar** (§9.3) — scopes Phase 2b but the architectural call lives here since it shapes the stable Phase 2+ surface.

**R1 agenda (7 items escalated from pre-R1 triage):**

- **arch-1** — `ExecutionState` payload shape (attribution triple carry-through, pinned-subgraph-CID set, context-binding Value resolution on resume). Cross-phase consumer-lens review via `ucan-capability-auditor`. See §9.1.
- **arch-2** — `HostError` freeze at G1: does the shape anticipate Phase-3 sync error shapes (hash-mismatch, HLC drift, cap-chain invalid)? See §9.2.
- **phil-1** — Inv-14 causal attribution: subgraph-data property or evaluator-policy property? Phase 6 AI delegation chains would foreclose under a rigid 3-tuple.
- **phil-2** — Inv-11 runtime enforcement placement: `benten-eval/invariants/system_zone.rs` (current plan, G5-B) or `benten-engine/primitive_host.rs` (philosophy-preferred — preserves arch-1 thinning).
- **phil-3** — `HostError` shape: A (opaque Box + ErrorCode), B (enum), C (ErrorCode-only). R1 locks before G1. See §9.2.
- **phil-5** — Shared `Budget` abstraction across Inv-8 (2a G4-A) and Inv-4/7 SANDBOX fuel (2b G7-A). See §9.12.
- **arch-6** — `evaluator.rs` sub-module partitioning: confirm G3-A owns top-level suspend/resume, G5-B owns new `evaluator/attribution.rs`.

**R1 proposed composition (7 seats, Phase 2a lens surface):**

- `architect-reviewer` — cross-phase coherence (carried from pre-R1)
- `benten-engine-philosophy` — thin-engine thesis + cross-phase foreclosure (carried from pre-R1)
- `code-reviewer` — tactical + generic correctness baseline (carried from pre-R1)
- `security-auditor` — persisted ExecutionState is a new attack surface; TOCTOU delegation is security-class
- `ucan-capability-auditor` — Phase-3 sync / cap-chain carry-through lens for arch-1 + arch-2
- `code-as-graph-reviewer` — Inv-11/13/14 enforcement is its direct domain
- `dx-optimizer` — WAIT developer ergonomics + devserver hot-reload surface

No `wasmtime-sandbox-auditor` or `ivm-algorithm-b-reviewer` at R1 — those lenses attach to Phase 2b's R1 composition.

**Planner confidence after pre-R1 triage:** 0.88 on the plan shape. The 7 R1 agenda items are narrowly scoped and each has a nominal planner position documented in §9 for R1 to evaluate from. Soft spots remaining: (a) R1 over-run on HostError (phil-3) stalls G1 — Rank 8a; (b) the ExecutionState payload shape (arch-1) is load-bearing across Phases 3/6/7, so `ucan-capability-auditor`'s cross-phase lens is essential.
