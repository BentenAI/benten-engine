# benten-eval INTERNALS

A plain-English deep-dive into `crates/benten-eval/`. Audience: a fresh agent or human reviewer trying to understand what this crate is, what it owns, where its boundaries are, and what to watch for in Phase 4-Foundation / Phase 4-Meta. Companion to `docs/ARCHITECTURE.md` + `docs/ENGINE-SPEC.md` + `docs/HOW-IT-WORKS.md`.

Total source: ~16.0k LOC across 51 `.rs` files (src) + 5 benches + 135 integration test files (~17.9k LOC of tests). This is the second-largest crate in the workspace after `benten-engine` and it is the crate where the project's load-bearing architectural commitments physically live.

**Status at HEAD `8141b94` (post `phase-4-foundation-close` tag; PR #242–#250 all merged).** Substantive Phase 4-Foundation work in this crate is bounded — exactly one source change landed (PR #210 / `f3930e1`: SUBSCRIBE per-event cap-recheck enum lift; see §10 below). Phase 4-Foundation platform layer (admin UI v0 + plugin manifest schema + materializer + schema-rendering) lands in the separate `benten-platform-foundation` crate, not here. The 12 primitives + the SANDBOX 4-host-fn floor + the arch-1 dep-break are unchanged at HEAD; all 14 invariants are production-runtime LIVE (Phase 2b + Phase 3 closure).

---

## 1. What this crate does

`benten-eval` owns **the 12 operation primitives and the evaluator that walks them**. Every handler in Benten is a content-addressed subgraph of operation Nodes; this crate is the code that turns those nodes into runtime behavior. Specifically:

- It declares the executor for each of the 12 primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM) under `src/primitives/`.
- It exposes the `Evaluator` — an iterative (non-recursive) stack-machine that walks a subgraph by repeatedly dispatching to per-primitive executors via `primitives::dispatch`.
- It owns the **`PrimitiveHost` trait** — the only seam through which a primitive executor reaches storage, the capability policy, the IVM subscriber, the change-stream port, the suspension store, the wasmtime SANDBOX host, the typed-CALL dispatch table, or sibling handlers. `benten-engine` is the production implementor; `NullHost` is the test default.
- It implements the **SANDBOX wasmtime host** end-to-end at `src/sandbox/**` (manifest registry + host-fn trampolines + per-call wasmtime lifecycle + the four enforcement axes MEMORY/WALLCLOCK/FUEL/OUTPUT + Inv-4 nest-depth + escape-defense state machine).
- It owns the **TRANSFORM expression language** (parser + pure evaluator + 50+ built-ins) under `src/expr/`.
- It owns the **runtime invariant checks**: structural (1/2/3/5/6/9/10/12), multiplicative budget (8), Inv-11 system-zone reads, Inv-13 immutability declaration, Inv-14 attribution, Inv-4 SANDBOX nest-depth, Inv-7 SANDBOX output ceiling. These live at `src/invariants/**`.
- It owns the **WAIT suspension protocol shape** — the frozen `ExecutionStateEnvelope` / `ExecutionStatePayload` / `AttributionFrame` / `Frame` types at `src/exec_state.rs`, plus the `SuspensionStore` port (`src/suspension_store.rs`) that unifies WAIT metadata, envelope bytes, SUBSCRIBE cursors, and cap-snapshot persistence behind one trait.
- It owns the typed-CALL closed dispatch registry (`src/typed_call.rs`) — Phase-3's crypto / DID / UCAN / VC operations that ride the existing CALL primitive without widening the SANDBOX host-fn surface or inventing a 13th primitive.

This is the crate where **CLAUDE.md baked-in commitment #1 ("12 primitives irreducible") physically lives**. New primitives can only appear by editing the `PrimitiveKind` enum in `benten-core` and adding an executor here; the dispatcher in `src/primitives/mod.rs::dispatch` is intentionally exhaustive. It is also the home of **baked-in #4 ("not Turing-complete; SANDBOX is the escape hatch")** — the iterative evaluator with bounded iteration plus the SANDBOX subsystem are the structural enforcement of that promise.

Per `lib.rs` doc + plan §9.10: the evaluator is **architecturally isolated from `benten-graph`** (the arch-1 / phil-r1-2 dep-break). Storage failures cross the boundary only as the opaque `HostError` envelope; no `benten-graph` type appears on the public surface. This is enforced both in `Cargo.toml` (no dep) and by the dedicated arch-test files (`tests/arch_1_no_graph_dep.rs` + `tests/arch_1_no_graph_types_in_primitive_host.rs` + the CI workflow `.github/workflows/arch-1-dep-break.yml`).

---

## 2. Dependency chain

### Workspace dependencies in
- `benten-core` — the source of truth for `Subgraph`, `SubgraphBuilder`, `OperationNode`, `NodeHandle`, `PrimitiveKind`, `Cid`, `Node`, `Edge`, `Value`, `ChangeStream`, `ChangeEvent`, `SubscriberId`. Re-exported through `lib.rs` so downstream consumers spell `benten_eval::{Subgraph, OperationNode, …}` without a second import.
- `benten-errors` — the stable `ErrorCode` catalog. Every `EvalError` arm + `SandboxError` arm + `ChunkSinkError` arm + `SubscribeError` arm has a `.code()` returning a catalog discriminant so the eval → engine → napi → TS pipeline preserves typed identity.
- `benten-caps` — the `CapabilityPolicy` / `CapError` / `GrantScope::parse` / `check_attenuation` / `DEFAULT_BATCH_BOUNDARY` surface. CALL + ITERATE consult this directly; READ + WRITE consult through `PrimitiveHost::check_read_capability` / `check_capability` so the engine can interpose its real policy backend.

### External dependencies in
- `wasmtime` (native-target only via `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`) — SANDBOX runtime. Configured with `consume_fuel + epoch_interruption + max_wasm_stack`, NO `pooling-allocator`, NO `component-model` (per wsa-3). The `async` Cargo feature is on for D27 forward-compat but no async host-fn ships in 2b.
- `getrandom` (native-target only) — workspace CSPRNG decision per D-PHASE-3-11; backs the `random` host-fn.
- `toml`, `wat` (native-target only) — used by the SANDBOX fixture loader at `src/test_fixtures.rs` and the manifest TOML drift detection.
- `serde` + `serde_ipld_dagcbor` + `serde_json` — DAG-CBOR canonical encoding for ExecutionStateEnvelope, AttributionFrame, CapBundle, etc. `serde_json` only for the SUBSCRIBE test-helper payload shape.
- `blake3` — content-addressing for envelope CIDs, attribution frame CIDs, manifest CIDs.
- `thiserror` — error envelope derives.

### Crates that depend on benten-eval (consumers out)
- `benten-engine` — orchestrates the evaluator: holds the `Engine` type that implements `PrimitiveHost`, owns `engine_wait.rs` / `engine_subscribe.rs` / `engine_sync.rs` glue, threads transactions through, manages the `Outcome` boundary that wraps `Evaluator::run_with_trace_attributed`. The only consumer that drives the evaluator end-to-end.
- `bindings/napi` (transitively, through `benten-engine`) — surfaces the trace / suspend / resume / sandbox shapes through napi-rs to TypeScript.
- Several Phase-3 crates (`benten-sync`, etc.) consume the `AttributionFrame` shape but route through `benten-engine` rather than reaching into `benten-eval` directly.

### Deliberately absent
- `benten-graph` — the arch-1 dep-break. Storage is host-mediated only.
- `benten-ivm` — IVM is consulted through `PrimitiveHost::read_view` returning a generic `Value`; the evaluator does not know about IVM strategies or view shapes.
- `benten-id` — typed-CALL crypto ops route through `PrimitiveHost::dispatch_typed_call`; the actual `benten-id` calls happen on the engine side.

---

## 3. Files inventory in `src/`

### Top-level crate surface (`src/*.rs`)

- **`lib.rs`** (~1151 LOC) — the crate root. Declares the `EvalError` enum (the single typed error every primitive returns), the `InvariantViolation` enum (one variant per Inv-N), `RegistrationError` (per-invariant diagnostic context for the DX layer), `InvariantConfig` (configurable thresholds), the `Evaluator` struct + `step()` dispatch shim, the `StepResult` and `TraceStep` shapes (Step / SuspendBoundary / ResumeBoundary / BudgetExhausted), the `Outcome` proxy mirrored from `benten-engine`, the public `evaluate` / `resume` aliases, and the test-callee registry (`register_test_callee` / `lookup_test_callee`) gated behind `cfg(any(test, feature = "testing"))`. Also declares the `transform` module exposing `parse_transform` + `AstIntrospect` + `TransformParseError`. This file is the single grep target for "what catalog code does X map to" — every `EvalError::*` variant has a one-line arm in `EvalError::code()`.

- **`evaluator.rs`** (~430 LOC) — the iterative walker. `Evaluator::run` / `run_with_budget` / `run_with_trace` / `run_with_trace_attributed` / `run_with_trace_attributed_capturing_with_budget` are the public entry points; `run_inner` is the private impl. Builds an adjacency map keyed by `(from_node_id, edge_label)`, finds the entry node (the unique one with no incoming edges), loops over `step()` calls, emits `TraceStep` rows when collecting, and short-circuits with `EvalError::Invariant(IterateBudget)` when `steps >= budget`. The `BudgetExhausted` trace-row emission for STREAM backpressure errors lives here. Recursion is banned per Validated Design Decision #4 — depth is enforced by `max_stack_depth` plus the registration-time Inv-2 check.

- **`context.rs`** (~243 LOC) — `EvalContext`: the scoped binding stack the evaluator threads through CALL boundaries. Frames hold `BTreeMap<String, Value>`; canonical names are `$input` / `$result` / `$item` / `$index` / `$results` / `$error`. Also carries the optional injected `TimeSource` (Phase-2a) and the injected `SuspensionStore` (Phase-2b G12-E). `pop_scope()` refuses to drop the last frame (the handler-top frame holds `$input` + `$result`).

- **`host.rs`** (~360 LOC) — the **`PrimitiveHost` trait**. This is the load-bearing seam: every host-managed operation a primitive executor reaches for routes through here. Methods: `read_node` / `get_by_label` / `get_by_property` / `put_node` / `put_edge` / `delete_node` / `delete_edge` / `call_handler` / `emit_event` / `check_capability` / `read_view` / `check_read_capability` / `suspension_store` / `elapsed_ms` / `suspending_principal` / `execute_sandbox` / `dispatch_typed_call` / `cached_transform_ast` / `iterate_batch_boundary`. The `ViewQuery` shape is defined locally so the evaluator stays ignorant of `benten-ivm`'s real `ViewQuery`. Also declares `NullHost` — the test default that misses on reads, ignores emits, surfaces `Backend("...unsupported")` on writes, and returns `PrimitiveNotImplemented` for SANDBOX.

- **`host_error.rs`** (~101 LOC) — `HostError`: the host-boundary error envelope. Three fields — stable `code: ErrorCode` (on wire), opaque `source: Box<dyn StdError>` (NEVER on wire, sec-r1-6 / atk-6 contract), optional `context: String` (on wire). Carries `to_wire_bytes` / `from_wire_bytes` (Phase-2a placeholder shape; a Phase-3 DAG-CBOR upgrade is a TODO). The arch-1 dep-break compromise lives here — instead of `EvalError::Graph(GraphError)` we have `EvalError::Host(HostError)`.

- **`exec_state.rs`** (~410 LOC) — the **frozen WAIT suspension shape** per plan §9.1. `AttributionFrame` carries `(actor_cid, handler_cid, capability_grant_cid)` plus Phase-2b `sandbox_depth: u8` (Inv-4 carrier per D20-RESOLVED) plus three Phase-3 G16-B sync-boundary fields (`peer_did_set` / `device_did` / `sync_hop_depth`). The DAG-CBOR canonicalisation uses `skip_serializing_if = "Option::is_none"` so Phase-2a-default frames produce the pinned schema-fixture CID `bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`; any non-default value produces a distinct CID (additive-extension discipline). `Frame` is the per-frame stack snapshot (just `{tag: String}` in 2a, will elaborate in 2b/3). `ExecutionStatePayload` carries the attribution chain + pinned subgraph CIDs + context binding snapshots + resumption principal CID + frame stack + frame index. `ExecutionStateEnvelope` wraps the payload with `schema_version: u8` and pre-computed `payload_cid: Cid`; the 4-step resume protocol verifies each boundary CID-by-CID. `SYNC_HOP_DEPTH_CAP = 8` lives here.

- **`suspension_store.rs`** (~575 LOC) — the **unified `SuspensionStore` trait** (G12-E generalisation). One port covers three previously-parallel surfaces: WAIT metadata (deadline + signal-shape + TTL by envelope CID), envelope bytes (the persisted ExecutionStateEnvelope by payload CID), SUBSCRIBE persistent cursors (max_delivered_seq by SubscriberId), plus the G14-D wave-5a `CapSnapshot` side-table (UCAN-proof-chain hash + historical-policy metadata by envelope CID). Implementations namespace their key prefixes to avoid Cid-collision between wait-metadata and envelope-bytes. `InMemorySuspensionStore` is the reference impl shipped here; the redb-backed impl lives in `benten-engine`. `SuspensionKey` is the typed key shape for `delete()`. `WaitMetadata` is the side-table value (suspend_elapsed_ms / timeout_ms / signal_shape / is_duration / ttl_hours / suspend_wallclock_ms).

- **`chunk_sink.rs`** (~905 LOC) — STREAM's typed transport. `Chunk { seq, bytes, final_chunk }`, `SendOutcome` (Accepted / BackpressureCredit / Closed), `ChunkSinkError` (typed error envelope, `#[non_exhaustive]`), `ChunkSink` trait (`send` / `try_send` / `close`), `BoundedSink` + `ChunkSource` (the concrete pair returned by `make_chunk_sink`), `ChunkProducer` trait + `ChunkProducerConfig` + `spawn_chunk_producer`. Default capacity 16 (D4-RESOLVED); zero capacity rejected at the type level via `NonZeroUsize`. Lossless is default; lossy is opt-in via `make_chunk_sink_lossy`. Optional producer wallclock budget via `make_chunk_sink_with_wallclock`. The trace-preservation pattern documented here is the one the evaluator uses to emit `TraceStep::BudgetExhausted { budget_type: "stream_backpressure", ... }` BEFORE propagating the typed error.

- **`typed_call.rs`** (~597 LOC) — the Phase-3 G21-T1 typed-CALL closed registry. Declares `TYPED_CALL_PREFIX = "engine:typed:"` and the `TypedCallOp` enum with 10 variants (Ed25519Sign / Ed25519Verify / KeypairGenerate / KeypairFromSeed / Blake3Hash / MultibaseEncode / MultibaseDecode / DidResolve / UcanValidateChain / VcVerify). Each variant carries an input-validation arm + a per-op `required_cap()` returning a `cap:typed:<group>` string. The actual dispatch happens in `benten-engine` via `PrimitiveHost::dispatch_typed_call`; this file is the closed enumeration + the input-shape gate. Phase-3 CLAUDE.md #16 closure: crypto ops fit CALL, not SANDBOX host-fn surface.

- **`subgraph_ext.rs`** (~283 LOC) — extension traits for the relocated `Subgraph` / `SubgraphBuilder` / `NodeHandle` types (G12-C-cont moved them to `benten-core`). `SubgraphBuilderExt` exposes `build_validated` / `build_validated_with_max_depth` / `build_validated_aggregate_all` / `wait_signal_typed`. `SubgraphExt` exposes `validate` / `cumulative_budget_for_root_for_test` / `to_mermaid` / `load_verified`. Both traits are sealed via a private marker so downstream crates cannot add their own impls. Necessary because Rust forbids inherent impls on foreign types and arch-1 forbids `benten-core` depending on `benten-eval` (which owns the invariants).

- **`testing.rs`** (~360 LOC) — `benten_eval::testing::testing_*` helpers gated behind `cfg(any(test, feature = "testing"))`. Re-exports chunk-sink + SUBSCRIBE test helpers so red-phase tests can drive STREAM / SUBSCRIBE primitives without a full engine stack.

- **`test_fixtures.rs`** (~211 LOC) — Phase-3 G17-B SANDBOX `.wat`/`.wasm` fixture loader. `load_fixture(name)` prefers committed `.wasm` bytes under `tests/fixtures/sandbox/<name>.wasm`, falls back to `wat::parse_file` on the source `.wat`. Subdir addressing via `escape/<name>`. Cross-platform CID-stability story: committed bytes round-trip byte-identically; the fallback is exact-version pinned (`wat = "=1.248.0"` in the workspace).

- **`time_source.rs`** (~219 LOC) — `TimeSource` trait (HLC stamps) + `MonotonicSource` trait (monotonic clock). `HlcTimeSource` + `InstantMonotonicSource` are the production defaults (Phase-2a placeholder — real `uhlc::HLC` wire-up is a Phase-3 TODO). `MockTimeSource` + `MockMonotonicSource` are test surfaces gated under the `testing` feature; they expose `frozen_at_epoch` / `at_epoch` / `advance` / `rewind_by` so the wallclock-refresh-TOCTOU tests can drive adversarial clock manipulation deterministically.

### `src/primitives/` — one file per primitive

- **`mod.rs`** (~135 LOC) — the dispatcher. `pub fn dispatch(op, host)` is the single intentionally-exhaustive `match` over `PrimitiveKind`. STREAM routes to the eval-side executor (which is a deliberate `PrimitiveNotImplemented` loud-fail because production STREAM dispatch goes through `engine.call_stream`, not `engine.call`). SANDBOX routes through `host.execute_sandbox` (NullHost returns `PrimitiveNotImplemented`; the engine impl threads the manifest + grant + module bytes through `crate::sandbox::execute`). WAIT in this dispatcher (the "regular walk path") consults `host.suspension_store()` + `host.elapsed_ms()` + `host.suspending_principal()` and either completes inline or short-circuits with `EvalError::WaitSuspended { handle }`.

- **`read.rs`** (~206 LOC) — READ executor. Three shapes: `target_cid: Bytes` (production path via `host.read_node`), `target_cid: Text` (legacy fixture path), `query_kind + label` (by-label list via `host.get_by_label`). Capability denial collapses to `ON_NOT_FOUND` / `ON_EMPTY` (the symmetric-None contract per Option C / sec-r1-5) so an unauthorised reader cannot distinguish denial from a missing CID.

- **`write.rs`** (~179 LOC) — WRITE executor. Modes: `create` / `update` / `delete` / `delete_missing` / `cas` / `test_inject_failure`. Cap denial routes `ON_DENIED`; CAS version mismatch routes `ON_CONFLICT` (not `Err` — R4 triage m3: conflicts are routed, never errored).

- **`transform.rs`** (~117 LOC) — TRANSFORM executor. Hot path consults `PrimitiveHost::cached_transform_ast` for a pre-parsed `Expr` (Phase-3 G19-E / phase-2-backlog §9.2 closure); fresh-parse path falls back to `expr::parser::parse`. Registration-time parse via `invariants::validate_transform_expressions` gives the fail-fast guarantee. Runtime failures route `ON_ERROR`; parse failures `Err(EvalError::TransformSyntax)`.

- **`branch.rs`** (~132 LOC) — BRANCH executor. Three shapes: binary (`condition_value: Bool`), multi-way (`match_value: Text` + optional `cases: List`), conditional-list (`conditions: List<{label, condition_value}>`). Routes via `"true"` / `"false"` / matched-label / `ON_DEFAULT`.

- **`iterate.rs`** (~145 LOC) — ITERATE executor. Named Compromise #1: capability re-check fires at `host.iterate_batch_boundary()` cadence (default 100). Entry refresh at iteration 0 always fires. `max` is required at registration (Inv-9 IterateMaxMissing); runtime `items_len > max` routes `ON_LIMIT`. Phase-1 contract is the cap-refresh cadence + edge routing; per-iteration body dispatch is Phase-2 scope.

- **`call.rs`** (~253 LOC) — CALL executor. Compromise #1 closure (CALL-entry cap refresh) runs first. Then attenuation via `benten_caps::check_attenuation(parent_scope, child_scope)`. Timeout check via `elapsed_ms > timeout_ms`. Phase-3 G21-T1 typed-CALL fork: if `target` starts with `engine:typed:`, route to `execute_typed_call` (which calls `host.dispatch_typed_call`). Otherwise dispatch through `host.call_handler(target, call_op, input)`.

- **`emit.rs`** (~86 LOC) — EMIT executor. Default fire-and-forget through `host.emit_event`. G14-D wave-5a handler-id-router seam: if `handler: Text(id)` is present, route through the named handler via `host.call_handler` instead of the default fan-out (observably different trace per stream-r1-2 LOAD-BEARING).

- **`respond.rs`** (~50 LOC) — RESPOND executor. Terminal leaf. Returns `StepResult { next: None, edge_label: "terminal", output: Map { status, body } }`. The evaluator's `run_inner` keys off `"terminal"` to break out of the walk loop.

- **`wait.rs`** (~590 LOC) — WAIT primitive surface. `SuspendedHandle` (Cid + signal name) is the opaque handle; `WaitOutcome` (Complete / Suspended) is the per-call outcome. `SignalShape` is the optional structural-typing carrier. `WaitResumeSignal` (Signal / DurationElapsed) is the resume payload. `evaluate` walks a subgraph and suspends at the first WAIT node. `evaluate_op` is the dispatcher-side entry the regular-walk path uses. Both call `evaluate_op_with_handler_id` which constructs the `ExecutionStatePayload`, builds the envelope, persists metadata + envelope through the `SuspensionStore` (G12-E), and returns `WaitOutcome::Suspended`. `resume_with_meta` is the engine-shared consumer — handles deadline check, duration-variant resume, signal-shape match. The Phase-2b G12-E missing-metadata fail-loud surfaces `EvalError::Host` with `HostBackendUnavailable` (closes Phase-2a Compromise #10).

- **`stream.rs`** (~347 LOC) — STREAM primitive surface. `StreamPersistMode` (Ephemeral / Persist) per phil-r1-1 default-ephemeral / opt-in-persist. `StreamPrimitiveSpec` + `StreamRunOutcome` + `AggregateStreamNode`. `run_stream_persist` materializes the aggregate Node (CID over chunk-byte concatenation — chunking-invariant). Concurrent + lossless test helpers (`run_lossless_stream_with_schedule`, `run_concurrent_producers`). The `execute` function itself is a deliberate `Err(PrimitiveNotImplemented(Stream))` loud-fail because the production runtime path is `engine.call_stream`, not `engine.call` (R6FP-Group-1 r6-stream-3 closure — the prior silent no-op was deceptive).

- **`subscribe.rs`** (~1817 LOC — the largest single file; grew at Phase-4-Foundation G22-FP-1-D) — SUBSCRIBE primitive surface. `ChangePattern` (AnchorPrefix / LabelGlob). `SubscribeCursor` (Latest / Sequence / Persistent). `SubscriptionSpec` (content-addressed `derive_subscriber_id`). `ActiveSubscription` handle + internal `SubscriptionState`. `register` / `register_with_store` / `register_as` — the registration paths. `publish_change_event` / `publish_change_event_with_label` / `publish_change_event_with_labels` — the fan-out path. `register_on_change` / `unregister_on_change` — the engine-side on-change callback surface. D5 strengthening commitments baked in: bounded retention (1000 events / 24h), within-key strict ordering + cross-key UNORDERED, exactly-once at handler boundary via engine-assigned `u64 seq` + dedup at delivery, Inv-11 system-zone reject for `system:*` patterns, register-time + delivery-time cap-checks. **Phase-4-Foundation G22-FP-1-D (PR #210, `f3930e1`) lifted `DeliveryCapRecheck` from `bool`/`Option` return to a typed enum** — `CapRecheckOutcome { Keep, Drop, Cancel }` (defined at `subscribe.rs:996`): `Keep` = deliver this event; `Drop` = silently elide THIS event (the per-row cap-recheck rejection path; matches `apply_atrium_merge` G16-B-F per-row recheck shape); `Cancel` = whole-subscription terminate (preserve historical semantics where the source identity gets fully revoked mid-stream). The closure type at line 1012 is now `Arc<dyn Fn(&ChangeEvent) -> CapRecheckOutcome + Send + Sync>`. Closes sec-4f-r1-1 BLOCKER (option-D ratified). `make_change_event` / `inject_event` are test helpers; `subscribe_revoked_mid_stream_count` is a metrics surface for the Phase-3 r6-r1-cap-1 closure (auto-cancel on cap-revoke surfaces `EvalError::SubscribeRevokedMidStream`).

- **`sandbox.rs`** (~1722 LOC — second-largest file) — the SANDBOX primitive executor. `SandboxConfig` (fuel / memory_bytes / wallclock_ms / output_bytes / max_nest_depth / max_wasm_stack / random_budget_bytes_per_call). `SandboxResult` (output bytes + fuel_consumed + output_consumed). `SandboxError` (#[non_exhaustive], 13 variants covering all four axes plus host-fn cap denial / not-found / manifest-unknown / module-invalid / nested-dispatch / stack-overflow / escape-attempt / module-not-installed / manifest-encode-failed). `resolve_priority` — D21 priority resolver (`MEMORY > WALLCLOCK > FUEL > OUTPUT`). `LiveCapCheck` type — the wave-5c live cap-recheck callback. `execute` / `execute_with_live_cap_check` — the two public entry points. The body sets up the per-call wasmtime `Store` + `Instance` + `Linker`, registers the host-fn trampolines, attaches the resource limiter, sets the epoch deadline, runs the module's `_start` (or named export), maps any `wasmtime::Error` through `trap_to_typed::map_call_error`, and packages a `SandboxResult`. Test-only `TestEscAttackInjection` seam (cfg-gated) drives ESC-7 / ESC-13 detection from inside the `time` host-fn trampoline. The `to_budget_exhausted_trace` helper builds the `TraceStep::BudgetExhausted` row the engine emits BEFORE propagating the typed error.

### `src/sandbox/` — the wasmtime host

- **`mod.rs`** (~83 LOC) — the SANDBOX subsystem's public face. `#![cfg(not(target_arch = "wasm32"))]` at module level. Re-exports counted_sink + escape_defenses + fingerprint + host_fns + instance + manifest + trap_to_typed + the executor surface (`SandboxConfig` / `SandboxError` / `SandboxResult` / `execute` / `execute_with_live_cap_check` / `resolve_priority` / `WALLCLOCK_*` / `MAX_WASM_STACK_DEFAULT`).

- **`manifest.rs`** (~490 LOC) — named-manifest registry (D2-RESOLVED hybrid). `CapBundle` (`caps: Vec<String>` sorted-canonical + optional `description` + Phase-3 reserved `signature: Option<ManifestSignature>`). `canonical_bytes` produces the DAG-CBOR encoding with `skip_serializing_if` on signature so the unsigned-bundle CID stays stable across the Phase-3 signed lift. `ManifestRegistry::new` pre-loads the codegen defaults (`compute-basic` = time + log; `compute-with-kv` = adds kv:read). `register_runtime` is reserved as `Err(RuntimeRegistrationDeferred)` until Phase-8 marketplace work; `from_overlay` is the wave-8h engine-install path. `ManifestRef::Named` / `ManifestRef::Inline` + `resolve(registry)`. ESC-15 closure: no permissive fall-through on unknown names. Intentionally NOT `Serialize` / `Deserialize` (det-r4b-4) so callers go through `canonical_bytes()`.

- **`host_fns.rs`** (~567 LOC) — the codegen host-fn table. `CapRecheckPolicy` (PerCall fail-secure default / PerBoundary). `HostFnBehavior` (TimeMonotonicCoarsened / LogSink / KvRead / Random — exactly four shapes per CLAUDE.md baked-in #16; storage-mutating shapes are explicitly absent and the regression test at `tests/host_fn_no_storage_mutating_per_baked_in_16.rs` defends this). `HostFnSpec` (name / requires cap-string / cap_recheck / behavior / bypass_output_budget / requires_async / description). `default_host_fns()` returns a process-shared `Arc<BTreeMap>` built once via `OnceLock` — perf-g7a-mr-2 fix (~12 allocations per call dropped). `DEFAULT_RANDOM_BUDGET_BYTES_PER_CALL = 4096`. `RESERVED_HOST_ASYNC_CAP = "host:async"` (declared, not yet used). `CapAllowlist::intersect(manifest_caps, grant_caps)` is the D7 init-snapshot intersection. `HostFnContext<'_>` is the per-invocation context the trampoline threads through every host-fn call (sink + allowlist + kv_reads_remaining + log_bytes_remaining + attribution frame + live_cap_check callback). `HostFnReturn` (Bytes / Empty / Error).

- **`instance.rs`** (~276 LOC) — per-call wasmtime lifecycle (D3-RESOLVED + wsa-20). `shared_engine()` returns the process-singleton `Engine` configured with `consume_fuel(true)` + `epoch_interruption(true)` + `max_wasm_stack(512 KiB)`. `module_for_bytes(bytes)` is the content-CID-cached module compiler — keys cache by BLAKE3 of module bytes, FIFO-evicts at `MODULE_CACHE_MAX_ENTRIES = 256`. `module_cache_size()` is a metric. The per-call `Store` + `Instance` lifecycle lives in `primitives/sandbox.rs::execute_with_live_cap_check`.

- **`counted_sink.rs`** (~242 LOC) — D17-RESOLVED Inv-7 streaming accumulator. `CountedSink` enforces `consumed + bytes.len() <= limit` BEFORE accepting bytes and traps with `SinkOverflow` carrying `OverflowPath::PrimaryStreaming` or `OverflowPath::ReturnBackstop`.

- **`epoch_ticker.rs`** (~136 LOC) — D24 wallclock-axis ticker. Process-wide daemon thread driven by `OnceLock`; ticks `shared_engine().increment_epoch()` every `EPOCH_TICK_INTERVAL = 10ms` (widened from 1ms post-wsa-w8b-3 parallel-test-run flake). `epoch_ticks_for_ms(ms)` converts a wallclock budget to a tick count.

- **`resource_limiter.rs`** (~169 LOC) — `SandboxResourceLimiter` implements `wasmtime::ResourceLimiter`. Caps `memory.grow` requests at `SandboxConfig::memory_bytes`. ESC-2 (linmem grow to limit) defense lives here. Rejection becomes a typed `SandboxError::MemoryExhausted` via `trap_to_typed`.

- **`trap_to_typed.rs`** (~430 LOC) — `map_call_error` walks the `wasmtime::Error` cause chain and surfaces the typed `SandboxError` variant. `EscapeAttemptMarker` is the typed escape-attempt signal the trampoline raises (sec-r1 D7 — host-fn cap denial routes typed-error, NOT wasmtime trap). `HostFnDenialMarker` carries `HostFnDenialKind` (CapDenied / NotFound / NestedDispatchDenied).

- **`escape_defenses.rs`** (~475 LOC) — Phase-3 G17-A1 wave-5b SURFACE for ESC-7 / ESC-13 / ESC-16. `EscVector` enum + `EscDefenseState` per-call carrier. `run_esc7_check` / `run_esc13_check` / `run_esc16_check` + `run_all_checks` are the boundary-firing defenses called by the trampoline. SURFACE in G17-A1; runtime arms wire in at wave-5c via the `SandboxStoreData` field plus host-fn boundary calls. `Trap::StackOverflow` → `SandboxError::StackOverflow` arm was the genuinely production-wired piece of G17-A1.

- **`fingerprint.rs`** (~151 LOC) — ESC-16 engine-side memory-read helper. `FINGERPRINT_COLLAPSE_THRESHOLD = 3` (conservative — 3 reads of a wallclock-correlated cell within one SANDBOX call). `record_wallclock_write` tags cells; `read_collapse_state` consults the side-table at host-fn boundaries.

- **`testing_helpers.rs`** (~413 LOC) — §7.3.A.7 SANDBOX-escape testing helpers. `cfg(any(test, feature = "test-helpers", feature = "testing"))`-gated. Four helpers: `testing_revoke_cap_mid_call` (ESC-9 driver), `testing_call_engine_dispatch` (ESC-10 driver), `testing_inject_forged_cap_claim_section` (ESC-14 driver), `testing_register_uncounted_host_fn` (ESC-7 setup). The cfg-gating discipline is load-bearing — Phase-2a sec-r6r2-02 precedent — and the audit pin `tests/sandbox_helpers_no_widening.rs` fires on every CI build if any pub item drifts out of the gate.

### `src/invariants/` — registration-time + runtime invariant checks

- **`mod.rs`** (~50 LOC) — re-export hub. Existing call sites import `benten_eval::invariants::validate_subgraph` / `validate_transform_expressions` / `canonical_subgraph_bytes` without knowing the body moved into submodules.

- **`structural.rs`** (~887 LOC) — invariants 1 (DAG-ness via DFS coloring), 2 (max depth — longest path), 3 (max fan-out), 5 (max nodes), 6 (max edges), 9 (determinism — non-deterministic primitives in `deterministic: true` handler rejected), 10 (content-hash order-independence — the canonical DAG-CBOR encoder sorts nodes and edges before encoding), 12 (registration-time catch-all aggregator). Also owns `validate_transform_expressions` which parses every TRANSFORM node's `expr` at registration time so unparseable grammar surfaces `E_TRANSFORM_SYNTAX` from `register_subgraph` rather than from `engine.call`.

- **`budget.rs`** (~732 LOC) — Invariant 8 multiplicative cumulative budget (G4-A). `MultiplicativeBudget(u64)` newtype + `DEFAULT_INV_8_BUDGET = 500_000`. The walker computes per-node factor (`ITERATE(N)` contributes `N`; `CALL { isolated: false }` is a pass-through; `CALL { isolated: true }` resets to the callee's declared bound — Code-as-graph Major #2 Option B; every other primitive contributes 1). The cumulative is the MAX across DAG paths of the per-path product, saturating at `u64::MAX`. `BudgetError` carries the typed `E_INV_ITERATE_BUDGET` code. The Phase-1 nest-depth-3 stopgap is dropped — a 10-deep nest at max=1 has cumulative 1 and must be accepted.

- **`system_zone.rs`** (~170 LOC) — Invariant 11 registration-time literal-CID walker. Walks READ + WRITE OperationNodes; if `label` or node `id` starts with `system:` it fires `InvariantViolation::SystemZone` → `E_INV_SYSTEM_ZONE`. The runtime half lives in `benten-engine/src/primitive_host.rs` and reuses the same code.

- **`immutability.rs`** (~142 LOC) — Invariant 13 registration-time declaration-layer reject of WRITE-to-registered-CID. The authoritative storage-layer enforcement lives in `benten_graph::RedbBackend::put_node_with_context` per the 5-row matrix; this is the declaration-layer affordance.

- **`attribution.rs`** (~156 LOC) — Invariant 14 structural declaration-time check. Every OperationNode in a registered subgraph MUST declare the `ATTRIBUTION_PROPERTY_KEY = "attribution"` property as `Value::Bool(true)` (Phase-2a hard-true per D12.7 Decision 2; opt-out moved to Phase-6 seeds). The runtime threader lives in `evaluator/attribution.rs`.

- **`sandbox_depth.rs`** (~414 LOC) — Invariant 4 SANDBOX nest-depth ceiling (G7-B). Two firing surfaces: registration-time static walker (longest SANDBOX-only chain along the call-graph) and `check_runtime_entry` (called from the SANDBOX executor BEFORE wasmtime instantiation). The counter rides on `AttributionFrame::sandbox_depth: u8` per D20-RESOLVED — INHERITED across CALL boundaries. `DEFAULT_MAX_SANDBOX_NEST_DEPTH = 4`. Saturation: `u8::checked_add(1).ok_or(NestedDispatchDepthExceeded)` so even `u8::MAX` cannot wrap.

- **`sandbox_output.rs`** (~278 LOC) — Invariant 7 SANDBOX cumulative-output ceiling (G7-B). `DEFAULT_MAX_SANDBOX_OUTPUT_BYTES = 16 MiB`. Single `check_admission` helper shared by both D17 paths (PRIMARY streaming sink + BACKSTOP return-value). D15 trap-loudly default — no silent truncation (the truncation-byte-position covert-channel concern per sec-pre-r1-07).

### `src/expr/` — TRANSFORM expression language

- **`mod.rs`** (~199 LOC) — `Expr` AST (Literal / Identifier / ContextBinding / Binary / Unary / Conditional / PropertyAccess / IndexAccess / Call / Array / Object / Lambda) + `BinaryOp` (Add/Sub/Mul/Div/Mod/Lt/Le/Gt/Ge/Eq/Ne/EqStrict/NeStrict/And/Or) + `UnaryOp` (Not/Neg/Pos). `Expr::uses_only_allowlisted_nodes` is the fuzz-harness allowlist property (vacuously true; the parser cannot produce other shapes).

- **`parser.rs`** (~1297 LOC) — hand-rolled Pratt / recursive-descent parser. Positive-allowlist grammar per `docs/TRANSFORM-GRAMMAR.md`. `ParseError { offset, message }` carries the byte offset of the first rejected token. Lambdas are only admitted as arguments to specific array methods (map / filter / reduce / find / findIndex / every / some / sortBy / uniqueBy / groupBy / count). Reserved-word denylist rejects `new` / `this` / `typeof` / `instanceof` / `function` / `return` / etc.

- **`eval.rs`** (~695 LOC) — pure deterministic evaluator. Walks `Expr` against `Env` binding frames; produces `Value`. No engine access, no clock, no RNG, no I/O. The determinism is load-bearing for IVM view consistency and content-hash invariance. Built-ins dispatch through `builtins::dispatch_builtin` + `builtins::dispatch_namespaced`.

- **`builtins.rs`** (~983 LOC) — 50+ pure built-ins covering arithmetic / string / array / object / coercion / number formatting. Bare (`min(a, b)`) and namespaced (`Math.min`) forms. `now()` is correctly absent (non-deterministic); `formatDate` is a Phase-3 audit TODO.

### `src/diag/` — diagnostic outputs (`diag` feature)

- **`mod.rs`** (~44 LOC) — gates `mermaid` + `trace` behind the `diag` feature. Slim builds get stub modules returning empty strings.
- **`mermaid.rs`** (~151 LOC) — `flowchart TD` renderer. WAIT outgoing edges render dashed (`-.->`) per dx-r1-9.
- **`trace.rs`** (~93 LOC) — `pretty(steps)` table renderer for CLI use.

### `src/evaluator/` — sub-modules under the evaluator

- **`attribution.rs`** (~96 LOC) — Inv-14 runtime threading. `thread_over_subgraph(subgraph, frame, host)` walks a subgraph and emits one `TraceStep::Step` per node with the `AttributionFrame` stamped in. `stamp_step` is the per-node builder; `default_frame_for_subgraph` is a deterministic frame derived from the handler id (tests use it when they don't have a principal registry).
- **`budget.rs`** (~103 LOC) — the shared budget helper consumed by both `primitives/iterate.rs` and `primitives/call.rs`. `cumulative_budget_for_subgraph` + `check_per_iteration_budget` are the two surfaces; both delegate to `invariants/budget.rs` so the validation logic stays in one place.
- **`attribution_schema_fixture.rs`** (~49 LOC) — `FIXTURE_CID = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a"`. The pinned empty-extensions `AttributionFrame` CID. Phase-6 additions must be additive; a shift in this CID fails CI.

### `build.rs` (~102 LOC)

Phase-3 G17-B build script. Walks `tests/fixtures/sandbox/**/*.wat` and emits `cargo:rerun-if-changed=` directives so a `.wat` source edit retriggers the `tests/fixture_wasm_hashes_stable` + `tests/d26_wasm_present` drift detectors. Early-exits on wasm32 (the SANDBOX surface is already cfg-cut from the wasm32 lib build). Does NOT regenerate `.wasm` itself — that's the `cargo bench-wat-rebake` regenerator binary's job.

---

## 4. Public API surface

### Crate root re-exports (from `lib.rs`)
- Types: `EvalError`, `InvariantViolation`, `RegistrationError`, `InvariantConfig`, `ExecutionFrame`, `Evaluator`, `StepResult`, `TraceStep`, `Outcome`, `SignalShape`, `SuspendedHandle`, `WaitOutcome`, `WaitResumeSignal`, `AttributionFrame`, `ExecutionStateEnvelope`, `ExecutionStatePayload`, `Frame`, `NullHost`, `PrimitiveHost`, `ViewQuery`, `HostError`, `SuspensionStore`, `InMemorySuspensionStore`, `SuspensionKey`, `WaitMetadata`, `SuspensionStoreError`, `TimeSource`, `MonotonicSource`, `HlcTimeSource`, `InstantMonotonicSource`, `MockTimeSource`, `MockMonotonicSource`, `TypedCallOp`.
- Re-exported from `benten-core` for one-stop import: `ATTRIBUTION_PROPERTY_KEY`, `NodeHandle`, `OperationNode`, `PrimitiveKind`, `Subgraph`, `SubgraphBuilder`.
- Re-exported from `benten-errors`: `ErrorCode`.
- Extension traits: `NodeHandleExt`, `SubgraphBuilderExt`, `SubgraphExt`.
- Top-level entry points: `evaluate(sg, ctx, input)` + `resume(sg, ctx, handle, signal)` (crate-root aliases that wrap `primitives::wait::evaluate` + `resume_with_meta`).
- Constants: `TYPED_CALL_PREFIX`, `limits::DEFAULT_MAX_DEPTH` / `_FANOUT` / `_NODES` / `_EDGES`.

### Per-primitive entry points (all under `crate::primitives::*::execute` and dispatched via `primitives::dispatch`)
- `read::execute(op, host)` / `write::execute(op, host)` / `transform::execute(op, host)` / `branch::execute(op)` / `iterate::execute(op, host)` / `call::execute(op, host)` / `respond::execute(op)` / `emit::execute(op, host)` / `stream::execute` (loud-fail stub) / `subscribe::execute(op, host)` / `sandbox::execute` (wrapped through host) / WAIT dispatched via `wait::evaluate_op`.

### `PrimitiveHost` trait — the single seam (~16 methods)
- Storage: `read_node`, `get_by_label`, `get_by_property`, `put_node`, `put_edge`, `delete_node`, `delete_edge`, `read_view`.
- Capability: `check_capability`, `check_read_capability`, `iterate_batch_boundary`.
- Sibling-handler dispatch: `call_handler`.
- Change-stream: `emit_event`.
- WAIT plumbing: `suspension_store`, `elapsed_ms`, `suspending_principal`.
- SANDBOX dispatch: `execute_sandbox`.
- Typed-CALL dispatch: `dispatch_typed_call`.
- AST cache: `cached_transform_ast`.

### SANDBOX surface (`crate::sandbox::*`)
- Executor: `execute(module_bytes, manifest_ref, registry, config, grant_caps, attribution)` + `execute_with_live_cap_check(..., LiveCapCheck)`.
- Config: `SandboxConfig` with the seven knobs (fuel / memory_bytes / wallclock_ms / output_bytes / max_nest_depth / max_wasm_stack / random_budget_bytes_per_call).
- Manifests: `ManifestRegistry::new` / `lookup` / `from_overlay` / `register_runtime` (Phase-8-reserved); `CapBundle::canonical_bytes` / `cid`; `ManifestRef::{Named, Inline}`.
- Host-fn table: `default_host_fns()` returns `Arc<BTreeMap<String, HostFnSpec>>`; `host_fn_names()` is the closed list `["time", "log", "kv:read", "random"]`.
- Engine + module: `shared_engine()` / `module_for_bytes(bytes)` / `module_cache_size()`.
- Errors: `SandboxError` (13 variants) + `resolve_priority(Vec<SandboxError>)` priority resolver.

### Transaction primitive

The transaction surface (`begin / commit / rollback` per Validated Design Decision #6) is NOT physically located in `benten-eval` — it lives on the `Engine` type in `benten-engine` because transactions span the cap-policy commit-time hook + the IVM materializer + the backend write batch, all of which live on the engine side. `benten-eval` participates by surfacing `EvalError::WriteConflict` for the CAS path. From the evaluator's perspective the transaction is invisible; the engine wraps the evaluator run in its own transaction boundary.

### TRANSFORM expression API
- `transform::parse_transform(input)` → `Result<AstIntrospect, TransformParseError>`.
- `expr::parser::parse(input)` → `Result<Expr, ParseError>` (crate-internal).
- `expr::eval::eval_with_namespaces(expr, env)` → `Result<Value, EvalError>` (crate-internal).

### Frozen interface contracts
- `ExecutionStateEnvelope` + `ExecutionStatePayload` + `AttributionFrame` + `Frame` — frozen at Phase-2a close per plan §8. Field additions must be additive (skip-when-default discipline preserves the pinned schema-fixture CID). Pinned by `tests/invariant_14_fixture_cid.rs` + the in-crate copy at `evaluator/attribution_schema_fixture.rs`.
- `SuspendedHandle` shape frozen at 2a close.
- `PrimitiveHost` trait — every method documented; default impls for new methods (`suspension_store`, `elapsed_ms`, `suspending_principal`, `execute_sandbox`, `dispatch_typed_call`, `cached_transform_ast`, `iterate_batch_boundary`, `check_read_capability`) so adding hooks is additive.
- 12-primitive set in `PrimitiveKind` — `#[non_exhaustive]` at the source-of-truth site (`benten-core`) for downstream version-evolution.

---

## 5. Tests inventory

Total: 135 integration test files / ~17.9k LOC. Organized into eight clusters by surface.

### Arch / dep-break (4 files)
- `arch_1_no_graph_dep.rs` / `arch_1_no_graph_dep_g6.rs` / `arch_1_no_graph_types_in_primitive_host.rs` / `no_graph_dep.rs` — assert at the source / Cargo.toml / type-signature level that `benten-graph` does not appear. Companion to the CI workflow.
- `benten_eval_no_residual_subgraph_definition.rs` — asserts the `Subgraph` / builder types moved cleanly to `benten-core` (no residual definitions left behind post-G12-C-cont).

### Per-primitive structural / contract (Phase-1 + 2a)
- `primitive_types.rs` — the 12-variant `PrimitiveKind` shape pin.
- `phase_two_primitives_structural.rs` — Phase-2 primitives (WAIT / STREAM / SUBSCRIBE / SANDBOX) round-trip structurally even when their executors are stubs.
- `primitive_read_write.rs` / `primitive_branch_respond_emit.rs` / `primitive_transform_builtins.rs` — Phase-1 executable primitives end-to-end.
- `read_denial.rs` / `read_primitive_option_c_symmetric.rs` — Option C symmetric-None capability discipline for reads.
- `requires_enforcement.rs` / `requires_property_call_time_check.rs` — CALL-entry cap check.
- `emit_handler_router.rs` — G14-D wave-5a handler-id-router seam for EMIT.

### Invariants (8 numbered + cross-cutting)
- `invariant_1_cycle.rs` / `invariant_2_depth.rs` / `invariant_3_fanout.rs` / `invariants_5_6_counts.rs` / `invariant_4_static.rs` / `invariant_4_runtime.rs` / `invariant_4_overflow.rs` / `inv_4_runtime_arm_fires_at_max_depth.rs` / `invariant_7_static.rs` / `invariant_7_runtime.rs` / `invariant_8_isolated_call.rs` / `invariant_8_isolated_call_post_boundary.rs` / `invariant_8_multiplicative.rs` / `invariant_8_nest_depth_stopgap_removed.rs` / `invariant_8_unknown_callee_rejects.rs` / `invariant_9_finalized.rs` / `invariants_9_10_12.rs` / `invariant_11_registration_time.rs` / `invariant_13_no_write_to_registered_subgraph.rs` / `invariant_14_attribution.rs` / `invariant_14_fixture_cid.rs` / `invariant_violation_to_error_code_exhaustive.rs` — comprehensive coverage including the schema-fixture CID pin + the exhaustive code-mapping. All 14 invariants are production-runtime LIVE at HEAD (Phase 2b + Phase 3 closure).
- `perf_inv_8_diamond_does_not_explode.rs` — perf regression pin for Inv-8 diamond DAG shape.
- `iterate_max_and_nest_depth.rs` — ITERATE-specific edge cases.

### SANDBOX (40+ files, the largest cluster)
- Per-axis: `sandbox_fuel.rs`, `sandbox_memory.rs`, `sandbox_wallclock.rs`, `sandbox_output.rs`, `sandbox_severity_priority.rs`, `sandbox_severity_priority_g17_b_anchor.rs`.
- Per-host-fn: `sandbox_host_fn_caps.rs`, `sandbox_host_fn_kv_read.rs`, `sandbox_host_fn_log.rs`, `sandbox_host_fn_time.rs`, `sandbox_host_fn_trampoline_count.rs`, `random_host_fn.rs`, `random_constant_time.rs`.
- ESC vectors: `sandbox_esc_7.rs` / `sandbox_esc_9.rs` / `sandbox_esc_13.rs` / `sandbox_esc_16.rs` / `sandbox_esc14_forged_cap_claim_section.rs` / `sandbox_esc_runtime_arms_e2e.rs` / `sandbox_escape_attempts_denied.rs` / `sandbox_stack_overflow.rs`.
- Manifest + capability: `sandbox_named_manifest.rs` / `sandbox_named_manifest_codegen_drift.rs` / `sandbox_capability_check_per_call_after_revoke.rs` / `sandbox_capability_intersection_at_init.rs` / `sandbox_rate_full_revalidation_g11_2b.rs` / `sandbox_rate_under_30_percent.rs`.
- Bytes / fixtures: `sandbox_d26_wasm_bytes_shipping.rs` / `d26_wasm_present.rs` / `fixture_wasm_hashes_stable.rs` / `wasm_tools_version.rs` / `register_default_host_fns_matches_codegen_table.rs`.
- Behavior: `sandbox_basic.rs` / `sandbox_attribution.rs` / `sandbox_attribution_frame_security.rs` / `sandbox_depth_inheritance_regression.rs` / `sandbox_handler_args.rs` / `sandbox_helpers_no_widening.rs` (load-bearing — audits testing-helper cfg-gating) / `sandbox_nested_dispatch.rs`.
- Architectural pins: `host_fn_no_storage_mutating_per_baked_in_16.rs` (CLAUDE.md #16 regression test) / `host_functions_toml_location.rs`.
- Proptests: `proptest_sandbox_fuel.rs` / `proptest_sandbox_isolation.rs` / `proptest_sandbox_output.rs`.

### WAIT + suspension protocol (15+ files)
- `wait_primitive_happy_path.rs` / `wait_timeout.rs` / `wait_ttl.rs` / `wait_ttl_cross_process.rs` / `wait_ttl_gc.rs` / `wait_dsl_signal_naming.rs` / `wait_signal_shape_optional_typing.rs`.
- `exec_state_dagcbor_roundtrip.rs` / `exec_state_envelope_shape.rs` / `exec_state_payload_shape.rs` / `proptest_exec_state_round_trip.rs` / `proptest_wait_ttl.rs`.
- `attribution_frame_shape.rs` / `attribution_non_regression.rs`.
- `cap_refresh_toctou.rs` / `time_source_and_monotonic_source.rs`.

### STREAM + chunk-sink (5 files)
- `chunk_sink_trait_shape.rs` / `stream_basic.rs` / `stream_lossless.rs` / `stream_lossy.rs` / `stream_per_handler.rs` / `stream_persist.rs` / `stream_producer_wallclock.rs` / `proptest_chunk_sink_conformance.rs` / `proptest_stream_lossless.rs`.

### SUBSCRIBE (10+ files)
- `subscribe_basic.rs` / `subscribe_cursor_modes.rs` / `subscribe_handler_router.rs` / `subscribe_idempotency.rs` / `subscribe_lifecycle.rs` / `subscribe_ordering.rs` / `subscribe_pattern.rs` / `subscribe_persist.rs` / `proptest_subscribe_ordering.rs` / `proptest_subscribe_pattern.rs`.
- Aggregator: `security.rs` (declares `security/subscribe_caps.rs` submodule).

### TRANSFORM grammar (4 files + proptests)
- `transform_grammar_fuzz.rs` / `transform_grammar_rejections.rs` / `proptests_transform_deterministic.rs` / `proptests_subgraph_order.rs`.

### Cross-cutting / framework
- Aggregator: `integration.rs` (declares `integration/inv_4_call_boundary.rs` + `integration/inv_7_streaming.rs` + `integration/sandbox_wasm32_disabled.rs`).
- `evaluator_stack.rs` / `host_error_shape.rs` / `g11_a_eval_wave1_minors.rs` / `trace_step_new_variants.rs` / `inv_14_sync_boundary_fields_distinct_cid.rs`.

---

## 6. Benches inventory

- **`ten_node_handler.rs`** — the §14.6 headline target (150-300µs for a mixed 10-node handler on dev hardware). Includes Phase-1-floor notes about macOS APFS fsync limits; the evaluator-only sub-bench (`list_dispatch_no_write`) lands in the sub-100µs range.
- **`multiplicative_budget_overhead.rs`** — Inv-8 walker per-boundary-check overhead. CI-gated <1µs median per §4.4-derived.
- **`wait_suspend_resume_latency.rs`** — `suspend_to_bytes` + `resume_from_bytes` round-trip excluding I/O. Plan §4.4 target 50µs; INFORMATIONAL gate in 2a.
- **`transform_expression_latency.rs`** — TRANSFORM parser cold-path latency. CI-gated <10µs median per ENGINE-SPEC §5.
- **`sandbox_cold_start.rs`** — D22-RESOLVED tiered cold-start budget (Linux x86_64: 2ms p95 / 5ms p99; macOS arm64 and Windows x86_64: 5ms p95 / 10ms p99). Loads per-platform thresholds from workspace-root `bench_thresholds.toml`.

(A sibling `sandbox_fuel.rs` skeleton was removed at pre-v1 Class-E fix-pass 2026-05-09 — empty bench body that never measured anything; per-host-fn fuel-cost measurement waits until the SANDBOX runtime matures Store+Instance+invocation lifecycle profiling.)

---

## 7. Thin-engine + composable-graph philosophy check

This is the crate where the 12-primitives-irreducible commitment (#1), the not-Turing-complete commitment (#4), the SANDBOX-min-viable commitment (#16), and the eval / engine boundary (#2 IVM thinness, arch-1 dep-break) all physically live. The audit is therefore the most consequential of any crate in the workspace.

### Strong examples — features composed from primitives, not bolted on

- **Transactions** (Validated Design Decision #6) — physically NOT here. `begin / commit / rollback` lives on `Engine` in `benten-engine`. The evaluator participates by surfacing `WriteConflict` and otherwise stays ignorant. This is the right factoring: transactions span the cap-commit-hook + IVM materializer + backend write batch, all engine concerns.
- **Typed-CALL** (`typed_call.rs`) — the textbook example of correct composition. Phase-3 needed Ed25519 + DID + UCAN + VC operations; the temptation was either (a) widen the SANDBOX host-fn surface (would violate #16) or (b) invent a 13th primitive (would violate #1). The chosen path threads typed-CALL ops through the EXISTING CALL primitive with a reserved `engine:typed:` prefix and a closed registry; the actual crypto runs in `benten-engine` (which can depend on `benten-id`) via the `PrimitiveHost::dispatch_typed_call` hook. Zero new primitives, zero new SANDBOX host-fns, full Phase-3 crypto surface. This is the pattern future "feature X needs a Y operation" requests should follow.
- **CRUD handlers** — the `crud('post')` DSL (`benten-engine` synthesises the subgraph) lowers to a READ → WRITE → RESPOND chain that this evaluator walks identically to any other handler. No special-case "CRUD path" in the evaluator. Closes Compromise #8 — `Engine::call` no longer short-circuits the evaluator for CRUD.
- **Version chains** (#8 baked-in) — opt-in `AnchorNode + VersionNode + CURRENT pointer` composition lives in `benten-core` and is consumed by handler subgraphs through plain READ + WRITE. Not a primitive. Ephemeral data doesn't pay versioning cost.
- **WAIT cross-process resume durability** — Phase-2b Compromise #10 closure happened via generalisation: the per-primitive ad-hoc surfaces (process-local WAIT registry, per-module SUBSCRIBE trait, engine-side envelope cache) all collapsed behind the single `SuspensionStore` port (`src/suspension_store.rs`). One trait, one engine wire-up, three suspension shapes survive process restart. Compositional pressure, not feature-addition pressure.
- **SANDBOX nest-depth Inv-4** — the counter physically rides on `AttributionFrame.sandbox_depth: u8` rather than on the evaluator's per-call stack. This makes the counter INHERIT across CALL boundaries (`SANDBOX → handler → SANDBOX → handler → SANDBOX` is depth 3, not three depth-1s). Composition through an existing carrier (AttributionFrame already crosses CALL for audit-trail integrity) rather than a parallel "nest depth tracker" type. D20-RESOLVED.
- **`AttributionFrame` additive extensions** (Phase-3 G16-B) — three new sync-boundary fields (`peer_did_set` / `device_did` / `sync_hop_depth`) added via `skip_serializing_if = "Option::is_none"` so the canonical Phase-2a CID stays stable when the fields are default. This is the discipline the FROZEN-SHAPE pin (`tests/invariant_14_fixture_cid.rs` + `evaluator/attribution_schema_fixture.rs`) demands; the technique scales to Phase-6 additions.

### Boundaries holding

- **arch-1 dep-break**: `Cargo.toml` has no `benten-graph` entry. The CI workflow + the four arch tests defend in depth. `HostError` is the opaque envelope; no graph type appears on the public surface.
- **IVM thinness**: `ViewQuery` is defined LOCALLY in `host.rs` so the evaluator does not import `benten_ivm::ViewQuery`. The engine maps between the two shapes at its `impl PrimitiveHost`.
- **SANDBOX host-fn surface**: closed at 4 (time + log + kv:read + random) per CLAUDE.md #16. Defended by `tests/host_fn_no_storage_mutating_per_baked_in_16.rs` regression pin. `HOST_FN_NAMES` is a `const &[&str; 4]` so adding a new name fails compilation in any code that depends on the length. There is NO `kv:write` / `kv:delete` / edge-mutating host-fn and the architectural commitment in `host_fns.rs::build_default_host_fns` doc-comment explicitly forbids adding one.
- **No Turing escape**: bounded iteration (Inv-8 multiplicative budget); DAG-only structure (Inv-1); SANDBOX is the only place arbitrary computation lives, and the SANDBOX runtime itself has four hard axes plus nest-depth plus escape-defense.

### Watch-outs

- **`SandboxConfig.testing_inject_attack`** is a cfg-gated test seam (ESC-7 / ESC-13 driver). The cfg gating discipline is load-bearing — sec-r6r2-02 precedent — and `tests/sandbox_helpers_no_widening.rs` audits every CI build. New ESC-defense work that adds production seams should follow the same pattern.
- **STREAM's `stream::execute` deliberately fail-loud** — it returns `PrimitiveNotImplemented(Stream)` because the production path is `engine.call_stream`. Prior silent no-op was deceptive (R6FP r6-stream-3 closure). If a future composition wants STREAM through `engine.call`, the correct move is to wire it through the existing `engine.call_stream` integration rather than re-resurrecting an eval-side body.
- **Class B β `Engine::read_node_as` SHIPPED at PR #184** (2026-05-10 pre-v1 cleanup). The 4 `todo!()` stubs that previously sat at `crates/benten-engine/src/engine_wait.rs:1011-1026` are CLOSED — those line numbers now host the real `get_node_label_only` / `put_node` / `read_node_as` / `resolve_subgraph_cid_for_test` bodies per CLAUDE.md baked-in #18. They are NOT in this crate, but the `PrimitiveHost::read_node` semantics this crate exposes are what the migration plugged into. New primitive executors that perform reads should call `host.read_node` + `host.check_read_capability` and let the engine impl thread the active-principal switching via `Engine::read_node_as`. The `pub` visibility of `Engine::get_node` (the un-attributed read pathway) is the only residual v1-assessment-window question — tracked at `docs/future/phase-4-backlog.md §4.43`; deferred to Phase-4-Meta API stabilization.
- **Pre-mature abstraction near zero**. The primitive executors are intentionally property-driven (`op.properties.get("...").and_then(...)`) and small. The only "framework" code is the dispatcher (single `match`) and the host trait (single seam). No primitive-base-class hierarchy, no executor-registry pattern, no codegen for executors. Good.

### No drift detected

I did not find: a new primitive sneaking in (none); SANDBOX host-fn surface expansion past min-viable (set is exactly time + log + kv:read + random; the architectural-floor doc-comment + regression test defend this); Turing-complete escape outside SANDBOX (bounded iteration + DAG-only); engine-orchestration logic leaking into eval (transactions correctly absent; engine-level installed_modules / capability backend / IVM strategies all stay on the engine side); compromises that turned into permanent shapes (Compromise #10 was generalized away; Compromise #8 was closed; Compromise #1 ITERATE-batch-boundary is still the named-compromise it was and the cadence is the right shape).

---

## 8. Phase 4-Foundation + Phase 4-Meta expectations

### Materializer pipeline
The materializer pipeline (turning declarative view definitions into IVM strategies the engine can run incrementally) is LIKELY to land either here, in a new sibling crate, or split across this crate + `benten-ivm`. Two reasons it might end up here: (a) materializers compile to subgraph specs that the existing evaluator walks (composition-through-primitives discipline); (b) the evaluator already owns the TRANSFORM expression compiler in `src/expr/`, which is the natural place to extend with a "compile expr to IVM strategy" pass. Two reasons it might NOT end up here: (a) the evaluator stays ignorant of `benten-ivm`'s internals by design (`Strategy` is named at the engine boundary but the per-strategy logic lives in `benten-ivm`); (b) materialization is conceptually a phase that happens AHEAD of evaluation, not during it. If the materializer compiles down to operation Node subgraphs the evaluator walks (rather than to runtime IVM strategy objects), Phase 4-Foundation work physically lives in `src/materializer/` here or in a new `benten-materializer` crate that imports `benten-eval` + `benten-ivm` and produces subgraph specs.

### Schema-driven rendering compiler
The Phase-4 rendering compiler (declarative schema → handler subgraph) is the same composition shape as the materializer. It emits subgraph specs; this evaluator walks them. The right question at Phase-4 design time is whether the compiler lives in a sibling crate (clean separation; depends on `benten-eval` for the subgraph types) or whether it lives in the DSL layer (`bindings/dsl` or equivalent TypeScript) and produces JSON which gets deserialised on the Rust side. Either way `benten-eval`'s public shape doesn't change.

### Admin UI v0 workflows
Per CLAUDE.md #15 (v1-milestone-gate widened 2026-05-10): v1 = Benten Platform = engine + admin UI + plugin ecosystem + decentralized self-discovered registry + UI composable from engine primitives. The "UI composable from engine primitives" part means admin UI workflows ARE subgraphs this evaluator walks. There's no UI-specific primitive; an admin form is a `READ → TRANSFORM → BRANCH → WRITE → RESPOND` chain with conditional rendering driven by view reads. The Phase-4 work touches this crate mostly through new TRANSFORM built-ins (rendering primitives), new view shapes consumed via `PrimitiveHost::read_view`, and possibly new typed-CALL ops for client-side validation. None of those expand the 12-primitive set.

### Class B β `Engine::read_node_as` interaction
CLAUDE.md #18 chose Class B β: the read pathway threads the active principal via `Engine::read_node_as(principal, cid)` (public, plugin-side caller-visible) and `Engine::read_node(cid)` (`pub(crate)`, engine internals, no permission check, no overhead on hot paths). This crate is downstream of that choice — `benten-eval`'s `PrimitiveHost::read_node(cid)` stays as the seam the evaluator uses. The active-principal switching is the engine's job. New primitive executors that perform reads should call `host.read_node` + `host.check_read_capability` and let the engine impl thread the principal-switch. The `host.suspending_principal()` accessor (already wired for WAIT) is the precedent: a plugin walk runs with the engine setting the active principal once at dispatch time; `host.read_node` consults whatever the engine has staged.

### Plugin authoring surface
Per CLAUDE.md #18 plugins are SHAREABLE SUBGRAPHS — authored against the 12 primitives + the TRANSFORM expression language + the SANDBOX host-fn surface, content-addressed, importable across Atriums. Plugin authors do not touch this crate's public surface; they author graph nodes (probably via the TypeScript DSL or a future UI builder). The `Engine` runs their subgraphs through this evaluator with the active principal switched. This crate's job is to keep the evaluator boring + deterministic + scoped to operations the plugin author actually invoked. The `attribution` frame field carries the plugin's identity through the walk; the cap-recheck hooks (`check_capability` / `check_read_capability` / `check_admission`) gate every host-bridging operation against the plugin's declared manifest envelope (resolved by the engine).

### Engine-level extensions
Per CLAUDE.md #19 engine-level extensions are Rust crates compile-time linked — "you compiled this in" is the trust model. They are NOT subject to `read_node_as` or any of the seams this crate exposes; they're inside the engine. This crate doesn't change for engine-extension work. If someone adds a new IVM strategy (engine extension), the evaluator stays ignorant — it consults `host.read_view` and gets a `Value` back.

---

## 9. Open questions / unresolved internals

- **WAIT regular-walk path's `signal_derived_placeholder` principal binding**. The Wave-8i fix-pass closed the obvious break (the regular-walk path no longer silently drops `call_as_with_suspension`'s `principal` arg), but the contract for `evaluate_op` when `principal: None` is still permissive: the envelope is keyed on signal-derived `BLAKE3(signal_name)` and `resume_from_bytes_as` will fire `E_RESUME_ACTOR_MISMATCH` against any non-trivial principal. The documented mitigation is "use `resume_from_bytes_unauthenticated`" in that path. The Phase-3 eval/engine `Outcome` unification TODO (named at `lib.rs:208`) is the lift-point.

- **TraceStep boundary-variant + attribution-threading completion**. The `TraceStep::Step` rows carry `attribution: Option<AttributionFrame>` and the runtime threader stamps them; `SuspendBoundary` / `ResumeBoundary` / `BudgetExhausted` rows do NOT carry attribution. Plan §5 names "required on every variant" as the eventual contract; the current shape-pin in `tests/inv_8_11_13_14_firing.rs` is the source of truth. Phase-3 broadens the contract.

- **`HostError::to_wire_bytes` Phase-3 DAG-CBOR upgrade**. The Phase-2a stub uses `code_str\0context_str` as the wire format; the TODO in `host_error.rs` names DAG-CBOR with a versioned envelope. Carrying.

- **`HlcTimeSource` real `uhlc::HLC` wire-up**. Phase-2a placeholder is an atomic counter; the `uhlc::HLC` integration lands with Phase-3 sync work. The trait shapes shipped; the default impl is the remaining work.

- **`build_only` / `list_dispatch_no_write` evaluator-only floor**. The 10-node bench's evaluator-only sub-benches land in the sub-100µs range but the Phase-2 grouped-commit work that would let the §14.6 150-300µs headline target be reached has not yet landed (`redb` v4 exposes only `Durability::Immediate` / `Durability::None`). Not blocking this crate — it's a backend / engine layering concern — but the bench narrative names the dependency.

- **AST cache invalidation on Phase-3 / Phase-4 handler re-registration**. The G19-E AST cache (`PrimitiveHost::cached_transform_ast`) is keyed by `(handler_cid, node_id)`; cache lookup is the responsibility of the engine impl. If Phase-4 introduces handler hot-reload or version-pinned per-call cache lookups, this crate's surface (`cached_transform_ast` returns `Option<Arc<Expr>>`) stays the same but the engine-side eviction policy will need tightening.

- **`SubscribeError::CapabilityDenied` collapses into `SubscribeDeliveryFailed` ErrorCode**. `subscribe.rs::SubscribeError::error_code` maps both `CapabilityDenied` and `DeliveryFailed` to `ErrorCode::SubscribeDeliveryFailed`. This was deliberate at the time but may be a separation candidate during a future ErrorCode split (cap-denied-at-register vs cap-denied-at-delivery are different operator stories).

- **The `phase_2b_landed` feature gate retirement audit**. Several aggregator files (`tests/integration.rs`, `tests/security.rs`) carry archaeology comments about the feature gate that protected them through Phase-2b. The gates are gone but the comments document the pattern — the audit pin at `tests/host_functions_toml_location.rs` is the live invariant on this front.

- **`HostError::source` opaque box vs a typed source enum** — the current `Box<dyn StdError + Send + Sync>` shape works but means cross-crate consumers cannot pattern-match on the source. A typed source enum behind a `#[non_exhaustive]` wrapper might land if a future consumer needs to route on the source class.

---

## 10. Phase 4-Foundation delta (HEAD `8141b94`)

Substantive Phase-4-Foundation changes to this crate are bounded — the platform layer (admin UI v0, plugin manifest schema, materializer pipeline, schema-driven rendering) lands in the sibling `benten-platform-foundation` crate, not here. The single substantive source change in this crate during Phase-4-Foundation:

- **PR #210 (`f3930e1`) — G22-FP-1-D SUBSCRIBE per-event cap-recheck enum lift.** `DeliveryCapRecheck` callback return type was changed from a `bool`/`Option<()>` shape to the typed enum `CapRecheckOutcome { Keep, Drop, Cancel }` at `crates/benten-eval/src/primitives/subscribe.rs:996-1013`. The three-way split closes sec-4f-r1-1 BLOCKER (option-D ratified): `Keep` continues the historical happy-path delivery; `Drop` skips THIS event but keeps the subscription alive (parallels the G16-B-F `apply_atrium_merge` per-row recheck shape); `Cancel` terminates the whole subscription (preserves the historical pre-Phase-4-Foundation behaviour). The dispatch arm at `subscribe.rs:1352-1376` is the consumer site; engine-side callers (`benten-engine/src/engine_subscribe.rs`) construct the closure via `CapabilityPolicy::check_read` per event.

Doc + test retense activity in this crate during Phase-4-Foundation:
- `ffd1e93` retensed stale RED-PHASE / Phase-N narratives in test + bench module headers (cluster-5 doc sweep).
- `59017a7` un-ignored test pins per pim-12 §3.6e staged-pin discipline (class-A un-ignore; closes Phase-2b/3 RED-PHASE staged-pin drift).
- `dcd1275` closed 3 standalone test/bench bugs from the per-crate review (class-E bug-fixes).
- `a9da0be` landed this INTERNALS.md (CRATES-DEEP-DIVE; 10 per-crate INTERNALS files + workspace synthesis).

No primitive added, no SANDBOX host-fn added, no invariant added, no public-API breakage. The 12-primitive set + 4-host-fn floor + 14-invariant production-runtime live state is identical pre- vs. post-Phase-4-Foundation. The crate retains its architectural posture as the home of CLAUDE.md baked-in commitments #1 (12 primitives irreducible) + #4 (not Turing complete; SANDBOX is the escape hatch) + #16 (SANDBOX min-viable host-fn surface) + arch-1 dep-break.

The R4b test-coverage findings naming `Engine::install_plugin` lifecycle + `validate_with_clock` + `apply_atrium_merge` envelope-recheck land in `benten-engine` + `benten-platform-foundation`, NOT here — they consume the SUBSCRIBE / CapRecheckOutcome surface via `PrimitiveHost`. The seven cross-cutting v1-shippable seams identified at R4b L1 are tracked in `docs/future/phase-4-backlog.md`.
