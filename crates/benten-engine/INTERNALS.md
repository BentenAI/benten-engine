# benten-engine — Internals

Plain-English deep-dive of the `benten-engine` crate, the orchestrator that composes Benten's storage, evaluator, IVM, capability, identity, and sync layers into a single public API. Audience: a developer landing in this crate fresh and trying to find the load-bearing seams. Snapshot HEAD: post-Phase-3-close pre-v1 cleanup window (~`5cab780`).

---

## 1. What this crate does

`benten-engine` is the orchestrator at the top of the crate stack. It does not implement primitives, storage, evaluation, capabilities, IVM, or sync itself — those live in `benten-eval`, `benten-graph`, `benten-caps`, `benten-ivm`, `benten-id`, and `benten-sync` respectively. What this crate does is **compose** them: it owns the public `Engine` struct, threads policy + clocks + suspension stores into builds, holds the call-stack metadata that lets the evaluator look up the active actor + handler, owns the change-broadcast tap that bridges storage commits into IVM and into ad-hoc subscribers, owns the WAIT / STREAM / SUBSCRIBE engine-side surfaces (subscriptions + cursors + cap-rechecks), owns the privileged system-zone writes (capability grants, IVM views, module manifests, handler-version anchors), owns the Atrium peer-to-peer sync session handle, owns the typed-CALL dispatch into `benten-id` crypto, and owns the napi-facing public API.

Everything a TypeScript caller can do to a Benten engine ultimately lands on a method here.

---

## 2. Dependency chain

**Workspace deps (in):** nearly all of them. `benten-errors`, `benten-core`, `benten-graph`, `benten-eval`, `benten-ivm`, `benten-caps` always; `benten-id` and `benten-sync` on native targets only (full-peer-only per CLAUDE.md baked-in #17). Notably **NOT** `benten-dsl-compiler` (see `tests/no_dsl_compiler_dep.rs` — engine consumes a pre-built `SubgraphSpec` shape, not source).

**External deps:** `blake3`, `serde`, `serde_ipld_dagcbor`, `serde_json`, `serde_bytes`, `thiserror`, `data-encoding`. Native-only: `toml`, `tracing`, `tempfile`, `ed25519-dalek`, `subtle`, `getrandom`, `bs58`, `zeroize`, `tokio` (rt + sync + macros), `iroh` (with `tls-ring`).

**Dev deps:** `criterion`, `proptest`, `wat`.

**Consumers (out):** `bindings/napi` (the napi-rs cdylib that powers `@benten/engine` JS/TS), every integration test in `tests/`, every bench in `benches/`, every crate's `dev-dependencies` cross-link that needs `Engine` in a fixture (with `features = ["test-helpers"]`).

**Features:**
- `default = []` (closed-by-default)
- `test-helpers` — implies `envelope-cache-test-grade` + `iteration-budget-test-grade`; lights up `crate::testing` + the `testing_*` methods sprinkled across `Engine` + `pub` accessors gated on `cfg(any(test, feature = "test-helpers"))`.
- `envelope-cache-test-grade` — retired (no-op shell kept for external consumers).
- `iteration-budget-test-grade` — gates `Engine::testing_set_iteration_budget` so the napi cdylib can opt in narrowly.
- `phase_2a_pending_apis` — gates a handful of WAIT-tests whose bodies reach into APIs wider than R3 consolidation stubs.
- `browser-backend` — flips `Engine` alias from `EngineGeneric<RedbBackend>` to `EngineGeneric<BrowserBackend>` and gates out every redb-coupled module so the wasm32-unknown-unknown thin-client target compiles.

---

## 3. Files inventory (`src/`)

The crate is large (~25.6k LOC across 40 files). Files group naturally by responsibility.

### 3a. Top-level wiring

- **`lib.rs`** (~320 LOC) — module declarations, public re-exports, feature gating. Deliberately thin after the R6 Wave-2 split; every public name flows through here so existing call sites like `benten_engine::Engine` stay stable. Contains a `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`. Has a small smoke-test module at the bottom.
- **`error.rs`** (~696 LOC) — `EngineError` variant catalog and `From`-conversions from every sibling crate's error type (`benten_graph::GraphError`, `benten_caps::CapError`, `benten_eval::EvalError`, etc.) plus the public-boundary erasure (`erase_backend_at_public_boundary`) that hides backend specifics inside `Box<dyn Error>`.
- **`outcome.rs`** (~726 LOC) — `Outcome`, `Trace`, `TraceStep`, `AnchorHandle`, `HandlerPredecessors`, `NestedTx`, `ReadViewOptions`, `RegisterReplaceOutcome`, `TerminalError`, `UserViewSpec` + builder, `UserViewInputPattern`, `ViewCreateOptions`, `BudgetExhaustedView`, `DiagnosticInfo`, `OutcomeExt`. The "response shapes" carrier crate.
- **`subgraph_spec.rs`** (~650 LOC) — the engine-side DSL builder types (`SubgraphSpec`, `SubgraphSpecBuilder`, `PrimitiveSpec`, `WriteSpec`, `IterateBody`, `GrantSubject` / `RevokeSubject` / `RevokeScope`), the `IntoSubgraphSpec` trait, the `IntoCallInput` trait. This is what `register_subgraph(spec)` accepts.
- **`system_zones.rs`** (~67 LOC) — the FROZEN `SYSTEM_ZONE_PREFIXES` constant (PascalCase labels matching HEAD). Single source-of-truth for the registration-time Inv-11 check, the runtime probe, and the storage stopgap. CI drift workflow keeps every `system:Label` literal in the codebase synced here.

### 3b. The Engine struct + builder

- **`engine.rs`** (~4471 LOC — largest file in the crate) — `EngineGeneric<B: GraphBackend>` (the actual struct), `EngineInner` (Arc'd shared state with ~25 fields, each documented for why it's there), `SubgraphCache`, the `pub type Engine = EngineGeneric<RedbBackend>` alias (or `<BrowserBackend>` under `--features browser-backend`), and the public dispatch methods (`open` / `builder` / `apply_atrium_merge` / `caps()` / `register_subgraph` / `register_subgraph_replace` / `register_crud` / `call` / `call_as` / `trace` / `handler_to_mermaid` / `handler_predecessors` / `handler_version_chain` / `dispatch_typed_call_public`). Contains the `subgraph_for_spec` + `subgraph_for_crud` walkers that materialize a runnable `Subgraph` from a registered DSL spec. Contains the `WAIT TTL GC` entry-point glue and the `Drop` impl that fires the final-sweep. The huge file size is intrinsic to the responsibility: this IS the orchestrator.
- **`builder.rs`** (~931 LOC) — `EngineBuilder` (fluent surface), `BackendGrantReader` (the `GrantReader` impl backed by `RedbBackend::get_by_label("system:CapabilityGrant")`), the `:memory:` sentinel routing, the `NOAUTH_STARTUP_LOG` constant + once-fire emission, the assembly pipeline (`build` → `assemble`) that wires the change-broadcast tap, the IVM subscriber, the broadcast→eval-subscribe bridge, the policy resolution (NoAuth / explicit / grant-backed / UCAN-grounded), the clock-source defaults, and the three rehydration passes at engine open (module manifests, module bytes, handler-version chains).

### 3c. CRUD + transactions + diagnostics (resolved-alias `impl Engine` blocks)

- **`engine_crud.rs`** (~231 LOC) — public node/edge CRUD: `create_node` / `get_node` (with the Inv-11 runtime probe + Option-C `check_read` gate) / `update_node` / `delete_node` / `create_edge` / `get_edge` / `delete_edge` / `edges_from` / `edges_to`. Every mutation refuses against a `read_only_snapshot` engine via `E_BACKEND_READ_ONLY`.
- **`engine_transaction.rs`** (~166 LOC) — `EngineTransaction<'tx, 'coll>` passed into the `.transaction(|tx| …)` closure. `create_node` / `put_node` / `delete_node` / `begin_nested`. Carries a `GraphTxLike` lifetime-elided shim so the closure can hold a borrow without the engine's full generic juggling.
- **`engine_diagnostics.rs`** (~744 LOC) — `snapshot` / `transaction` / `count_nodes_with_label` / `metrics_snapshot` / per-capability commit + denial tallies / `change_stream_capacity` / `ivm_subscriber_count` / `diagnose_read`, the anchor + version surface (`create_anchor` / `append_version` / `read_current_version` / `walk_versions` / `set_device_cid` / `device_cid` / `set_actor_cid` / `effective_actor_cid`), `schedule_revocation_at_iteration`, and `testing_insert_privileged_fixture`. `metrics_snapshot` is the canonical operator-dashboard surface.
- **`engine_caps.rs`** (~430 LOC) — `EngineCapsHandle<'eng>` + `CapProof` (Phase-3 G16-B-F), `install_proof` / `revoke` / `revoke_capability_by_grant_cid`, the legacy `create_principal` / `grant_capability` / `grant_capability_with_proof` / `revoke_capability` privileged-write paths, `create_view` (legacy), and `install_ucan_proof` for the UCAN-durable policy path.
- **`engine_views.rs`** (~1013 LOC) — view materialization (`read_view` / `read_view_with` / `read_view_strict` / `read_view_allow_stale`), user-view registration (`register_user_view` / `create_user_view`), user-view drain + snapshot (`user_view_snapshot` / `user_view_on_update` / `user_view_drain_updates_since` / `user_view_change_offset`), the IVM view-strategy accessor (`view_strategy`), `materialize_view_with_gate` (composes the G15-A per-row read gate), and the change-stream surface (`subscribe_change_events` / `test_subscribe_*` / `change_event_count` / `subscribe_emit_events*` / `emit_subscriber_count`).
- **`engine_modules.rs`** (~631 LOC) — module manifest lifecycle: `install_module` (REQUIRED `expected_cid` arg per D16-RESOLVED-FURTHER; signature verification BEFORE persistence per Compromise #21 closure), `uninstall_module`, `is_module_installed`, `active_module_capabilities`, `compute_manifest_cid`, plus the `InstalledModule` in-memory record + the durable `system:ModuleManifest` zone-Node mirror.
- **`engine_sandbox.rs`** (~285 LOC) — SANDBOX plumbing only (no top-level `Engine::sandbox(...)` — user code reaches SANDBOX exclusively via DSL composition). Exposes the `SandboxNodeDescription` diagnostic shape + `describe_sandbox_node` / `describe_sandbox_node_for_handler` (test-gated) and the `SANDBOX_UNAVAILABLE_ON_WASM_TEXT` literal. On wasm32 the methods surface `E_SANDBOX_UNAVAILABLE_ON_WASM` at execution time.
- **`engine_snapshot.rs`** (~358 LOC) — Phase-2b G10-A snapshot-blob handoff: `export_snapshot_blob` / `from_snapshot_blob` / `compute_snapshot_blob_cid`. Native + not-`browser-backend` only. A snapshot-blob engine sets `read_only_snapshot = true` and every user-mutation surfaces `E_BACKEND_READ_ONLY`.

### 3d. Primitive host + cross-language dispatch

- **`primitive_host.rs`** (~1666 LOC) — `impl PrimitiveHost for Engine` plus `ActiveCall` / `PendingHostOp` / `WALLCLOCK_REFRESH_CEILING` (5 min). This is the boundary trait the evaluator drives. The two-phase write discipline lives here: every `put_node` / `delete_node` / `put_edge` / `delete_edge` the evaluator emits during a walk lands in the active call frame's `pending_ops`; after the walk terminates, `dispatch_call_inner` opens a single backend transaction and replays the buffered ops atomically. The capability hook fires once per `Engine::call` against the fully-assembled batch (named compromise #5 — Phase-1 record-don't-enforce posture, sharpened in Phase-2/3). Also owns the `is_system_zone_label` runtime probe, the `cap_error_to_outcome` / `eval_error_to_engine_error` / `outcome_from_terminal_with_cid` / `system_zone_to_outcome` / `tx_aborted_outcome` mappers, the WAIT-resume cap-snapshot-hash recompute hook, the `cached_transform_ast` override that consults `crate::ast_cache::AstCache`, and the `execute_sandbox` override that records SANDBOX metrics into `EngineInner::sandbox_metrics`.
- **`typed_call_dispatch.rs`** (~559 LOC) — engine-side dispatch for the 10 typed-CALL ops (Ed25519 sign / verify, Keypair generate / from-seed, Blake3 hash, Multibase encode / decode, DID resolve, UCAN chain validate, VC verify). Lives here (not in `benten-eval`) because `benten-eval` cannot depend on `benten-id` per arch-r1-10. Native-only. Pure on engine state — no WRITE, no event emission, no IVM update. Reached via `<Engine as PrimitiveHost>::dispatch_typed_call`.

### 3e. WAIT / STREAM / SUBSCRIBE / EMIT surfaces

- **`engine_wait.rs`** (~1538 LOC) — the WAIT primitive engine-side surface: `call_with_suspension` / `call_as_with_suspension` / `suspend_to_bytes` / `resume_from_bytes_unauthenticated` / `resume_from_bytes_as` / `resume_with_meta`, `SuspendedHandle`, `ResumePayload`, `SuspensionOutcome`, the `HandlerRef` trait that lets callers identify a handler by either CID or stored handler-id. Also hosts the `put_node` + `read_node_as` Class-B-β surface (the load-bearing plugin-engine seam per CLAUDE.md baked-in #18 — public `_as` for non-trusted principals, `pub(crate) read_node` inside `engine_crud.rs::get_node` for engine-internal reads), plus the `get_node_label_only` fast-path probe used by the Inv-11 runtime check, the WAIT-resume cap-snapshot-hash recompute, the `fabricate_test_suspend_envelope*` test helpers, `register_wait_reference_handler`, and the bench helpers + AST-cache testing accessors.
- **`engine_stream.rs`** (~1163 LOC) — STREAM primitive engine-side surface: `call_stream` / `call_stream_as` / `open_stream` / `testing_open_stream_for_test`, `StreamHandle`, `StreamCursor`, the `STREAM_GRANT_CEILING_*` constants (1M chunks / 30s wallclock — the workspace defaults that per-handler grants must NARROW under), `active_stream_count`, the producer-bridge wiring that drives a real `benten_eval::chunk_sink::ChunkProducer` thread (not the eval-side `stream::execute` body — see the engine_stream module doc for why the eval-side is dead code on the engine path).
- **`engine_subscribe.rs`** (~773 LOC) — SUBSCRIBE primitive engine-side surface: `on_change` / `on_change_as` / `on_change_as_with_cursor` / `on_change_with_cap_recheck` / `on_change_with_cursor`, `Subscription`, `SubscribeCursor` (Latest / Sequence / Persistent), `OnChangeCallback`, the cap-recheck-at-delivery closure construction, and the `testing_subscribe_observable_change_events` helper. The handle's `Drop` impl unsubscribes automatically. D5-RESOLVED semantics: engine-assigned `u64 seq`, exactly-once at the handler API surface via dedup against `max_delivered_seq`.
- **`emit_broadcast.rs`** (~243 LOC) — `EmitBroadcast` / `EmitEvent` / `EmitSubscription`. A separate fan-out channel from `ChangeBroadcast` because EMIT events have no Node / Cid / `ChangeKind` / commit context. The Phase-2b audit-gap fix that closed the "EMIT primitive was a silent no-op" surface.
- **`change.rs`** (~204 LOC) — `ChangeBroadcast` + `ChangeCallback` alias. Stdlib-only Vec-of-Arc-callback fan-out behind a Mutex. Implements `benten_graph::ChangeSubscriber` so the backend registers it directly. Two taps land at engine assembly: one records into `EngineInner::observed_events` for `ChangeProbe::drain`; the other bridges to the eval-side SUBSCRIBE registry via `benten_eval::primitives::subscribe::publish_change_event_with_labels`.
- **`change_probe.rs`** (~49 LOC) — small handle that drains the engine's bounded observed-events queue, filtering by an optional label.
- **`handler_router.rs`** (~177 LOC) — `HandlerRoute` enum (`DefaultFanOut` / `Named(handler_id)`) plus `HandlerRouteLog`. The seq-major-8 LOAD-BEARING typed seam that lets EMIT / SUBSCRIBE distinguish "fan out to every default consumer" from "route through a specific named handler subgraph". The engine-side records the decision; the eval-side primitive consumes an optional `handler` property on the OperationNode.

### 3f. Capability + auditing seams

- **`cap_recheck.rs`** (~243 LOC) — `CapRecheckFn` type alias (`Arc<dyn Fn(...) -> bool + Send + Sync + 'static>`) — the SINGLE shared-signature surface that BOTH G14-D SUBSCRIBE delivery-time gate AND G15-A IVM materialization-time gate compose on. The "extract first; no inline-then-refactor" pattern per `seq-minor-6`. A test (`cap_recheck_helper_no_refactor_on_g14d_or_g17a1_landing.rs`) pins the no-refactor contract.
- **`cap_snapshot_hash.rs`** (~364 LOC) — `cap_snapshot_hash` pure-function derivation: BLAKE3-of-(domain-separator || actor_cid || sorted-grant-chain || sorted-revocation-set || policy-backend-identity-tag). Length-prefixed lists defend against ambiguous concatenation. Used by the WAIT-suspend envelope so resume can reject `E_CAP_SNAPSHOT_HASH_MISMATCH` when ANY of the four input dimensions has shifted.
- **`ivm_view_read_gate.rs`** (~264 LOC) — `IvmViewReadGate` — materialization-time per-row READ gate for IVM-materialized views (G15-A; closes Compromise #11 in coordination with the G14-D delivery-time gate). Composes a label-hint extractor + a `CapRecheckFn`; deny-from-either-layer wins.
- **`thin_client_subscribe.rs`** (~572 LOC) — thin-client SUBSCRIBE surface for the wasm32-unknown-unknown bundle (browser tabs as authenticated views into a full peer). `ThinClientConnection` + `ThinClientSubId` + `ThinClientMetrics` + `ThinClientError`. `connect` / `connect_unauthenticated` / `subscribe` / `try_next_event` / `delivered_count` / `revoke_device_did` / `is_device_did_revoked` / `thin_client_metrics`. F6 filter at the full-peer edge supersedes the connection-time cap-check.

### 3g. Suspension + WAIT TTL

- **`suspension_store.rs`** (~586 LOC) — `RedbSuspensionStore` — the engine-side adapter that wires `Arc<RedbBackend>` into `benten_eval::SuspensionStore`. Three reserved key prefixes (`sw:` WAIT metadata, `se:` envelope bytes, `sc:` SUBSCRIBE persistent cursor). The cross-process compromise-#10 closure.
- **`wait_ttl_gc.rs`** (~229 LOC) — three sweep paths (event-driven, interval backstop, drop-final) plus `WaitTtlGcStats`. Storage-cleanup mechanism; the resume-time deadline check at `engine_wait.rs::resume_from_bytes_inner` is the load-bearing correctness mechanism that fires `E_WAIT_TTL_EXPIRED` independently of whether GC ran.

### 3h. Atrium peer-to-peer sync (native-only)

- **`atrium_api.rs`** (~155 LOC) — public Atrium API surface: `AtriumConfig` + `AtriumMode` + `SyncStatus` + `TransportKind`. Re-exports `AtriumError` + `AtriumHandle` from `engine_sync`. Used as `engine.open_atrium(config).await`.
- **`engine_sync.rs`** (~1805 LOC — second-largest file) — `AtriumHandle` + `AtriumError` + `AtriumResult`, the `DeviceAttestationEnvelope` (signed on-the-wire device-DID attestation envelope V2 + Ed25519 sig + payload-hash binding + session-nonce replay defense — Phase-3 G16-D wave-6b cryptographic closure for criterion 16), `DeclaredDeviceAttestation`, `DeclaredCapabilityClaim`, `SyncMergeAttribution`. `open` / `open_with_keypair` / `set_local_device_did` / `set_local_device_attestation` / `set_local_device_keypair` / `set_acceptor` / `register_device_attestation` / `list_declared_device_attestations` / `register_peer_did` / `resolve_peer_dids` / `peer_id` / `loopback_addr` / `atrium_status` / `sync_subgraph` / `sync_subgraph_over` / `accept_sync_subgraph` / `register_zone` / `with_zone` / `merge_remote_change` / `merge_remote_change_with_hop_depth` / `hlc_node_id` / `local_hlc` / `inbound_hlc_skew_classifier_calls` / `is_active` / `leave` / `rejoin` / `close`. Owns the per-zone Loro CRDT documents, the iroh `Endpoint`, the local `Hlc` clock, the Inv-13 row-4 SPLIT classifier (rejects user-zone op-logs whose keys target `system:*` prefixes with `E_SYNC_DIVERGENT_CID_REJECTED`).

### 3i. Identity + crypto integrations (native-only)

- **`manifest_signing.rs`** (~872 LOC) — Compromise #21 closure: Ed25519 manifest signing wire-through. `ManifestVerifyMode` (Unsigned / Registry / UcanChain / Dual), `ManifestVerifyArgs`, `ManifestVerifyError`, `sign_manifest`, `verify_manifest_with_mode`, `PublisherRegistry` (with `add_publisher` / `lookup` / `add_publisher_unauthorized` / `add_publisher_with_ucan` / `verify_manifest_dual`), `manifest_signed_bytes`. The `system:PublisherRegistry` zone backing.
- **`handler_versions.rs`** (~302 LOC) — Compromise #18 closure: durable per-handler version chain via `system:HandlerVersion` zone Nodes carrying `handler_id` / `version_cid` / `predecessor_cid` / `seq`. Rehydrated at engine open. `HandlerVersionChain` + `make_version_node` + `handler_version_chain_with_anchor`.
- **`anchor_store.rs`** (~103 LOC) — G14-C wave-4b consolidation of the anchor-fetching ad-hoc helpers into one `AnchorStore<'a>` handle. Resolves the `cov-f3` residual.
- **`module_manifest.rs`** (~411 LOC) — `ModuleManifest` schema (D9-RESOLVED canonical DAG-CBOR), `ModuleManifestEntry`, `ManifestSummary`, `ManifestSignature`, `MigrationStep`, `ManifestError`. Single source-of-truth for the install-time CID-pin. Mirrors `packages/engine/src/types.ts` field-for-field.
- **`engine_config.rs`** (~346 LOC) — workspace-level `engine.toml` loader: `EngineConfig` + `SandboxSection` + `EngineConfigError`. Read at `Engine::open` time; overrides D24 SANDBOX wallclock defaults + ceiling. Native-only.

### 3j. Caches

- **`ast_cache.rs`** (~207 LOC) — per-handler TRANSFORM AST cache keyed on `(handler_cid, node_id)`. Populated at `register_subgraph*` time, invalidated at `register_subgraph_replace`. Consumed via the `PrimitiveHost::cached_transform_ast` override. `AstCacheStats` surfaces hit/miss counters for the `subgraph_ast_cache_full_wire_up` integration pin. Closes phase-2-backlog §9.2.

### 3k. Test surface

- **`testing.rs`** (~1061 LOC) — `iterate_write_handler`, `minimal_write_handler`, `read_handler_for`, `subject_with_no_read_grants`, `handler_declaring_read_but_writing_admin`, `handler_with_call_attenuation_escalation`, `policy_with_grants`, `counting_capability_policy`, `principal_cid`, `minimal_wait_handler`, `subgraph_bytes_for_handler`, `testing_advance_wait_clock`, plus countless helpers consumed by sibling-crate integration tests via dev-deps. Cfg-gated to `cfg(any(test, feature = "test-helpers"))` so the production cdylib doesn't ship the surface.

---

## 4. Public API surface

### 4a. `Engine` construction

- `Engine::open(path)` — convenience for `EngineBuilder::new().open(path)`. Routes the `":memory:"` sentinel to a transient redb store. The most common construction path for tests + scaffolded apps.
- `Engine::builder() -> EngineBuilder` — fluent surface (`.path` / `.backend` / `.capability_policy` / `.capability_policy_grant_backed` / `.capability_policy_ucan_durable` / `.with_policy_allowing_revocation` / `.production` / `.without_ivm` / `.without_caps` / `.without_versioning` / `.change_stream_capacity` / `.monotonic_source` / `.time_source` / `.suspension_store` / `.ucan_grounded_now_for_test` / `.with_test_ivm_budget` / `.ivm_max_work_per_update`). `.build()` performs the assembly + the three rehydration passes (module manifests, module bytes, handler-version chains).

### 4b. Code-as-graph dispatch

- `Engine::register_subgraph(spec) -> Result<String, EngineError>` — invariant battery + WAIT TTL validation + TRANSFORM-syntax fail-fast + SANDBOX manifest-name validation + reserved-handler-namespace reject + persist via `system:HandlerVersion` + in-memory swap + populate AST cache.
- `Engine::register_subgraph_replace(spec) -> Result<RegisterReplaceOutcome, EngineError>` — hot-replace shape; bumps the version chain. Carefully-ordered persist-before-swap so a crash between in-memory swap and disk leaves disk consistent.
- `Engine::register_subgraph_aggregate(spec)` — same shape with aggregate invariant collection (collects every violation into `RegistrationError::violated_invariants` rather than first-fail).
- `Engine::register_crud(label)` — zero-config `READ → RESPOND` handler with auto-registered `content_listing_<label>` IVM view.
- `Engine::register_crud_with_grants(label)` — Phase-1 alias (grant-backed routing is a Phase-2/3 policy concern, not a registration concern).
- `Engine::call(handler_id, op, input) -> Result<Outcome, EngineError>` — dispatch a handler walk through `benten_eval::Evaluator::run_with_trace` with `&self` as `PrimitiveHost`. Buffered host writes replay atomically at commit.
- `Engine::call_as(handler_id, op, input, actor)` — same but with an explicit actor CID (the capability hook binds against this).
- `Engine::call_with_revocation_at(...)` — call-shape variant whose scheduled-revocation arm is exercised by the TOCTOU harness.
- `Engine::trace(handler_id, op, input)` — side-effect-free trace mode; buffered ops are dropped instead of replayed. Returns per-step `TraceStep` records.
- `Engine::handler_to_mermaid(handler_id)` — render the registered subgraph as a Mermaid flowchart.
- `Engine::handler_predecessors(handler_id)` — predecessor adjacency map keyed by the same BLAKE3-derived per-step `node_cid` `trace` uses.
- `Engine::handler_version_chain(handler_id) -> Vec<Cid>` — newest-first version chain (Compromise #18; durable).

### 4c. The Class B β plugin-engine seam (CLAUDE.md baked-in #18, PR #184)

- `Engine::read_node_as(principal: &Cid, cid: &Cid) -> Result<Option<Node>, EngineError>` — the load-bearing read entry point for any read attributed to a non-trusted principal. Plugin authors NEVER call this directly; the evaluator threads the active principal through this surface when dispatching a plugin's read. Mirrors the `Engine::call_as` precedent. Inv-11 runtime probe + Option-C symmetric-None denial both apply.
- `Engine::put_node(node: &Node) -> Result<Cid, EngineError>` — the engine-internal write counterpart. Routes through the same backend transaction as user-facing `create_node` (so ChangeEvents + IVM materialization fire on commit) but skips the user-facing Inv-11 system-zone label rejection at the API surface. Storage-layer Inv-11 guard stays as defense-in-depth.

(The `pub(crate) read_node` half of the Class-B-β shape lives inside `engine_crud.rs::get_node` — the user-facing public read has the runtime probe + `ReadContext::actor_cid = None` posture; `read_node_as` is the public `_as` flanking entry point with `actor_cid: Some(*principal)`. The 4 `todo!()` stubs cited in the brief at `engine_wait.rs:1011-1026` are CLOSED — those line numbers now host real `put_node` / `read_node_as` / `resolve_subgraph_cid_for_test` implementations.)

### 4d. CRUD direct surfaces (in `engine_crud.rs`)

- `create_node` / `get_node` / `update_node` / `delete_node` / `create_edge` / `get_edge` / `delete_edge` / `edges_from` / `edges_to`. Each refuses against a `read_only_snapshot` engine with `E_BACKEND_READ_ONLY`.

### 4e. Privileged caps + IVM views

- `Engine::caps() -> EngineCapsHandle<'_>` — the production-equivalent grant-mutation handle: `install_proof` / `revoke` / `revoke_capability_by_grant_cid` (PR #199, UCAN revocation observance closure). Updates BOTH the durable `system:CapabilityGrant` / `system:CapabilityRevocation` zone-Node mirror AND the in-memory `(actor, zone)` revocation-pair set that `apply_atrium_merge`'s per-row cap-recheck consults.
- `Engine::create_principal(name)` / `grant_capability(actor, scope)` / `grant_capability_with_proof(...)` / `revoke_capability(actor, scope)` — the legacy direct entry points.
- `Engine::install_ucan_proof(ucan)` — UCAN-durable path; routes through `UCANBackend` + `UcanGroundedPolicy`.
- `Engine::create_view(view_id, opts)` / `register_user_view(spec)` / `create_user_view(spec)` — IVM view registration (legacy id-string form + new UserViewSpec builder form).
- `Engine::read_view*` / `user_view_snapshot` / `user_view_on_update` / `user_view_drain_updates_since` / `user_view_change_offset` — view-read + drain surfaces. `materialize_view_with_gate` composes the G15-A per-row read gate.

### 4f. Module install lifecycle

- `Engine::install_module(manifest, expected_cid, verify_args)` — D16-RESOLVED-FURTHER requires the `expected_cid` arg; signature verification runs BEFORE persistence per Compromise #21 closure.
- `Engine::uninstall_module(cid)` / `is_module_installed(cid)` / `active_module_capabilities()` / `compute_manifest_cid(manifest)`.
- `Engine::register_module_bytes(cid, bytes)` — D-PHASE-3-12 strict CID validation; persists via durable `RedbBlobBackend` + mirrors into in-memory cache. Wasm32 fires `E_SANDBOX_UNAVAILABLE_ON_WASM` immediately.
- `Engine::fetch_module_bytes(cid)`.

### 4g. WAIT / STREAM / SUBSCRIBE / EMIT

- `Engine::call_with_suspension<H: HandlerRef>(...)` / `call_as_with_suspension(...)` / `suspend_to_bytes(handle)` / `resume_from_bytes_unauthenticated(bytes)` / `resume_from_bytes_as(bytes, actor)` / `resume_with_meta(...)`.
- `Engine::call_stream<H: HandlerRef>(...)` / `call_stream_as<H: HandlerRef>(...)` / `open_stream<H: HandlerRef>(...)` / `active_stream_count()`.
- `Engine::on_change(pattern, callback)` / `on_change_as(...)` / `on_change_as_with_cursor(...)` / `on_change_with_cap_recheck(...)` / `on_change_with_cursor(...)` / `subscribe_change_events()` / `change_event_count()`.
- `Engine::emit_event(channel, payload)` / `emit_with_handler(channel, payload, route)` / `subscribe_emit_events(f)` / `subscribe_emit_events_with_handle(...)` / `subscribe_with_handler(pattern, route)` / `handler_route_log()`.

### 4h. Atrium peer-to-peer sync

- `Engine::open_atrium(config: AtriumConfig).await -> Result<AtriumHandle, AtriumError>` — opens an iroh-backed Atrium session.
- `Engine::apply_atrium_merge(atrium, anchor, zone, bytes, incoming_hop_depth).await -> Result<Cid, EngineError>` — the engine-side completion of the row-4a user-data sync path: CRDT merge → peer-DID resolution → per-row HLC skew classifier → Inv-13 row-4b per-key SPLIT defense → per-row cap-recheck (structural-always-on per Ben's RATIFIED Option (a) at sec-r4r1-2 BLOCKER closure) → AttributionFrame mint → Version Node persistence + anchor advance.
- `Engine::sync_replica_cap_recheck_calls()` — observability for the per-row cap-recheck counter.
- `AtriumHandle` exposes: zone registration, Loro `with_zone` access, `merge_remote_change*`, peer-DID registration + resolution, local-device-DID + attestation + keypair binding, acceptor injection, `sync_subgraph` / `accept_sync_subgraph`, `leave` / `rejoin` / `close`.

### 4i. Snapshot + transaction

- `Engine::snapshot()` / `Engine::transaction(|tx| …)` / `Engine::is_read_only_snapshot()`.
- `Engine::export_snapshot_blob()` / `Engine::from_snapshot_blob(bytes)` / `Engine::compute_snapshot_blob_cid(bytes)`.

### 4j. Anchors + version chain

- `Engine::create_anchor(name) -> Result<AnchorHandle>` / `Engine::append_version(anchor, node)` / `Engine::read_current_version(anchor)` / `Engine::walk_versions(anchor)`.
- `Engine::set_device_cid(opt)` / `Engine::device_cid()` / `Engine::set_actor_cid(opt)` / `Engine::effective_actor_cid()`.

### 4k. Typed-CALL crypto dispatch

- `Engine::dispatch_typed_call_public(op: TypedCallOp, input: &Value) -> Result<Value, EngineError>` — direct napi-facing path; performs the per-op cap-check (`G21-T2 fp-mini-review BLOCKER-1` closure) before invoking the dispatch arm.

### 4l. Diagnostics + metrics

- `Engine::metrics_snapshot() -> BTreeMap<String, f64>` — canonical operator-dashboard surface: `benten.writes.total`, `benten.writes.committed[.<scope>]`, `benten.writes.denied[.<scope>]`, `benten.change_stream.dropped_events`, `benten.ivm.view_stale_count`, `benten.sandbox.handler.<id>.{fuel_consumed_high_water, output_consumed_high_water, last_invocation_ms}`, `benten.subscribe.on_change_registration_count`, `benten.emit.subscriber_count`, `benten.sync_replica.cap_recheck_calls`, `benten.stream.active_count`.
- `Engine::audit_sequence()` — storage-layer commit counter (NOT the engine `writes_committed_total`).
- `Engine::count_nodes_with_label(label)` / `capability_writes_committed()` / `capability_writes_denied()` / `change_stream_capacity()` / `ivm_subscriber_count()` / `diagnose_read(cid) -> DiagnosticInfo`.

### 4m. Thin-client surface (browser-backend or wasm32)

- `ThinClientConnection::connect / connect_unauthenticated / subscribe / try_next_event / delivered_count / revoke_device_did / is_device_did_revoked` + `Engine::thin_client_metrics()` / `Engine::thin_client_publish_event(...)`.

### 4n. Top-level re-exports

`Engine`, `EngineGeneric`, `EngineBuilder`, `EngineError`, `EngineTransaction`, `EngineCapsHandle`, `CapProof`, `Outcome` + `Trace` + `TraceStep` + `OutcomeExt`, `AnchorHandle`, `HandlerPredecessors`, `NestedTx`, `ReadViewOptions`, `RegisterReplaceOutcome`, `TerminalError`, `UserViewInputPattern`, `UserViewSpec` + `UserViewSpecBuilder`, `ViewCreateOptions`, `BudgetExhaustedView`, `DiagnosticInfo`, `SubgraphSpec` + `SubgraphSpecBuilder`, `PrimitiveSpec`, `WriteSpec`, `IntoCallInput`, `IntoSubgraphSpec`, `IterateBody`, `GrantSubject`, `RevokeScope`, `RevokeSubject`, `ChangeProbe`, `EmitBroadcast` + `EmitEvent` + `EmitSubscription`, `OnChangeCallback`, `SubscribeCursor`, `Subscription`, `StreamCursor`, `StreamHandle`, `STREAM_GRANT_CEILING_CHUNK_COUNT`, `STREAM_GRANT_CEILING_WALLCLOCK_MS`, `CHANGE_STREAM_MAX_BUFFERED`, `ResumePayload`, `SuspensionOutcome`, `RedbSuspensionStore`, `WaitTtlGcStats`, `ModuleManifest` + `ModuleManifestEntry` + `ManifestError` + `ManifestSignature` + `ManifestSummary` + `MigrationStep`, `SYSTEM_ZONE_PREFIXES`, `EngineConfig` + `EngineConfigError` + `SandboxSection`, `AstCacheStats`, `NOAUTH_STARTUP_LOG`, `SandboxNodeDescription`, `SANDBOX_UNAVAILABLE_ON_WASM_TEXT`. Plus `benten_errors::ErrorCode`, `benten_eval::PrimitiveKind` + `TypedCallOp` + `TYPED_CALL_PREFIX` + `Chunk` + `ChunkSink` + `CapSnapshot` + `InMemorySuspensionStore` + `SuspensionKey` + `SuspensionStore` + `SuspensionStoreError` + `WaitMetadata`.

---

## 5. Tests inventory

The crate ships substantial test coverage — ~145 files in `tests/` plus ~45 in `tests/integration/`. Most are single-purpose pin files driven by an R3-R6 finding or BLOCKER closure. Grouped by domain:

### 5a. Engine surface + architecture pins

- **API + thinness:** `engine_api_surface.rs`, `engine_builder_thinness.rs`, `thinness_preserved_after_arch_1.rs`, `engine_generic.rs` (the cascade pin asserting no inherent `RedbBackend` outside the resolved-alias block), `engine_no_dyn_graph_backend.rs` (D-PHASE-3-1 dyn-erasure defense), `cargo_default_features.rs`, `cargo_public_api_drift.rs`, `cargo_vet_policy_self_test.rs`, `full_missing_docs_sweep_no_warnings_workspace_wide.rs`, `no_dsl_compiler_dep.rs`, `architecture_md_10_crate_count_post_phase_3_canaries.rs`, `no_unauthorized_dyn_error.rs`, `atriums_no_new_primitives.rs` (no new primitives — 12-irreducible commitment defense).
- **Error catalog + boundary:** `engine_error_boundary.rs` (typed `GraphError` survives the `Box<dyn Error>` source-chain erasure), `engine_error_codes.rs`, `error_catalog_md_drift_phase_2b.rs`, `host_error_wire_format_excludes_cids.rs`.

### 5b. CRUD + transactions + system zones

- `engine_crud.rs`, `cid_pin_handler_version_chain.rs`, `cid_pin_loro_merged_versions.rs`, `engine_transaction_policy_panic_safe.rs`, `engine_read_view_returns_view_state.rs`, `engine_read_view_stale.rs`, `prop_no_state_leak.rs`, `production_refuses_noauth.rs`, `noauth_startup_log.rs`.
- **System zones:** `inv_11_read_from_user_code_to_system_zone_denied.rs`, `inv_11_system_zone_drift_test.rs`, `inv_11_transform_constructed_cid_adversarial.rs`, `inv_11_write_system_label_rejected.rs`, `system_zone_api_exclusivity.rs`, `system_zone_stopgap_and_full_coexist.rs`, `reserved_handler_namespace_rejected.rs`.
- **Other invariants:** `inv_8_isolated_call_budget_bypass.rs`, `inv_13_dispatch.rs`, `inv_13_wait_resume_stale_pin.rs`, `invariant_coverage_doc_lists_inv_4_and_inv_7_active.rs`.

### 5c. Capabilities + UCAN

- `cap_recheck_helper_no_refactor_on_g14d_or_g17a1_landing.rs`, `cap_recheck_helper_signature_pinned.rs`, `cap_recheck_in_flight.rs`, `cap_snapshot_hash_inputs.rs`, `subscribe_cap_recheck.rs`, `subscribe_cap_recheck_concurrency.rs`, `subscribe_device_revoke.rs`, `subscribe_partial_revoke_typed_error.rs`, `call_stream_as_partial_revoke_cancels_stream.rs`.
- **UCAN:** `engine_ucan_b_alignment.rs`, `ucan_replay_audience.rs`, `revoke_capability_by_grant_cid.rs`.
- **Engine-side `read_node_as` / `put_node` closure:** `engine_read_node_as_put_node_pre_v1_closure.rs`.

### 5d. WAIT + suspension

- `engine_wait_api_shape.rs`, `wait_production_runtime_routing.rs`, `wait_resume_cross_process.rs`, `wait_resume_policy.rs`, `wait_ttl_gc_machinery.rs`, `g12_e_suspension_store_round_trips.rs`, `redb_suspension_in_process.rs`, `wallclock_refresh_ntp_slew_doesnt_skip.rs`, `wallclock_refresh_uses_monotonic_only.rs`, `resume_decode_failure_not_panic.rs`, `resume_with_missing_attribution_triple_rejects.rs`, `resume_with_revoked_grant_denies.rs`, `resume_with_substituted_principal_rejects.rs`, `resume_with_tampered_attribution_rejected.rs`, `mermaid_wait_rendering.rs`, `dsl_wait_duration_translation_pin.rs`.

### 5e. STREAM + SUBSCRIBE + EMIT

- `engine_call_returns_typed_error_on_stream_bearing_handler.rs`, `subscribe_multi_label_node_delivers_when_pattern_matches_secondary.rs`, `emit_broadcast_replicas.rs`.

### 5f. Module install + SANDBOX

- `install_module_rejects_cid_mismatch.rs`, `module_install.rs`, `module_install_rejects_cid_mismatch_dual_cids_in_error.rs`, `module_uninstall.rs`, `module_manifest_canonical.rs`, `module_manifest_doc_present.rs`, `module_manifest_signature_field_reserved.rs`, `manifest_schema_parity_pin.rs`, `manifest_signing.rs`, `manifest_temporal_binding.rs`, `manifest_unknown.rs`, `module_bytes_cid.rs`, `host_functions_doc_drift_against_toml.rs`, `host_functions_doc_lists_every_codegen_entry.rs`, `host_functions_md_drift_against_toml.rs`.
- **SANDBOX:** `sandbox_execute_via_engine_dispatch_invokes_executor.rs`, `sandbox_limits_doc_present.rs`, `sandbox_metrics.rs`, `sandbox_rate_phase_3_revalidation.rs`, `sandbox_unavailable_on_wasm_error_message_exact_text_pin.rs`, `wasm_sandbox_unavailable.rs`, `describe_sandbox_node_returns_diagnostic_shape.rs`, `primitive_host_edge_unsupported.rs`.

### 5g. Atrium + sync

- `atrium_g16_b_e_substantive_e2e.rs` (the LOAD-BEARING dual-gate deepest-e2e composition pin), `atrium_leave_rejoin.rs`, `atrium_lifecycle.rs`, `device_attestation_envelope_direct.rs`, `device_cid_runtime_arm.rs`, `sync_hop_depth_bound.rs`, `sync_inbound_hlc_skew_rejected.rs`, `sync_replica_attribution.rs`, `hlc_attribution_frame.rs`, `anchor_prefix.rs`, `anchor_store.rs`.

### 5h. IVM views + AST cache

- `ast_cache.rs`, `ast_cache_invalidation.rs`, `register_subgraph_failures.rs`, `register_subgraph_replace.rs`, `register_user_view.rs`, `ivm_read_gate.rs`, `ivm_view_subscribe_compose.rs`, `view_id_label_hint_refactor.rs`, `loro_version_chain.rs`, `handler_predecessors_real.rs`, `handler_version_chain.rs`, `transform_parse_at_registration.rs`, `subgraph_spec_primitives_widening.rs`, `user_view_canonical_id_anchor_prefix_refused.rs`, `user_view_end_to_end.rs`, `user_view_snapshot.rs`, `user_view_strategy_a_rejected_for_user.rs`, `user_view_strategy_b_default.rs`, `user_view_strategy_refusals.rs`, `strategy_enum_boundary.rs`.

### 5i. Typed-CALL + DSL parity

- `typed_call_cap_gating.rs`, `typed_call_engine_dispatch.rs`, `typed_call_ucan_grounded.rs`, `code_to_ctor.rs`, `dsl_args_vs_eval_properties_parity_meta_test.rs`, `dsl_specification_md_finalization.rs`, `ts_surface_parity_meta_test.rs`.

### 5j. Doc-coupling + spec compliance

- `error_catalog_md_drift_phase_2b.rs`, `security_posture_compromise_9_marked_closed.rs`, `security_posture_md_phase_2b_compromises_documented.rs`, `quickstart_md_walkthroughs_compile.rs`, `component_model_decision.rs`, `component_model_phase3_decision_lands_per_d_phase_3_6.rs`, `paper_prototype_revalidation_doc_present.rs`, `metrics.rs`, `write_authority_enum.rs`.

### 5k. Engine-open rebuild + R6 pins

- `engine_open_rebuilds_module_manifest_active_set_from_persisted_zone.rs`, `envelope_cache_test_grade_feature_retired.rs`, `sec_r6r1_01_inv_14_attribution_threading_preserved_under_g12_c.rs`, `sec_r6r2_02_test_helpers_gating_preserved_under_g12_c_migration.rs`, `g14_d_wave_5a_closed_claims.rs`, `g21_t3_section_d_pins.rs`, `spike_survivors.rs`, `snapshot_no_tempdir.rs`, `wasm32_wasip1_canonical_cid_matches_native.rs`, `wasm_no_redb.rs`.

### 5l. `tests/integration/`

Composition-heavy integration tests that exercise multiple engine surfaces end-to-end: `arch_1_dep_break_verified`, `browser_target_bundle_size`, `budget_exhausted_trace_emission`, `cap_toctou`, `caps_crud`, `change_stream`, `compromises_regression`, `cross_process_graph`, `cross_process_wait_resume`, `emit_event_observable_via_emit_broadcast`, `engine_sandbox`, `engine_stream`, `engine_subscribe`, `esc_subscribe_integration`, `exit_criteria_all_six`, `install_module_rejects_cid_mismatch`, `inv_8_11_13_14_firing`, `ivm_propagation`, `ivm_strategy_b_uses_algorithm_b_view`, `module_install_in_memory_only_in_browser`, `module_install_uninstall_round_trip`, `module_uninstall_releases_capabilities`, `nested_tx`, `option_c_end_to_end`, `sandbox_compile_time_disabled_on_wasm32`, `sandbox_in_crud`, `sandbox_module_not_installed_emits_typed_error`, `sandbox_named_manifest_resolves_via_install_module`, `snapshot_blob_round_trip`, `stale_view`, `stream_composition`, `stream_into_sandbox`, `stream_napi`, `subscribe_emit`, `suspension_store_round_trip_subscription_cursor`, `suspension_store_round_trip_wait_metadata`, `system_zone_integration`, `trace_no_persist`, `tx_atomicity`, `version_current`, `view_stale_count`, `wait_inside_wait_serializes_correctly`, `wait_resume_determinism`, `wait_signal_shape_optional_typing`, `wait_ttl_expires_via_suspension_store`, `wallclock_toctou_revokes_mid_iterate`, `wasip1_target_canonical_cid`, `write_authority_lift`.

The `tests/common/` directory holds shared fixtures: `mod.rs` and `ucan_fixtures.rs`.

---

## 6. Benches inventory

- **`roundtrip.rs`** — `hash_only` / `create_node` / `get_node` / `full_roundtrip`. Engine-level baseline; the actual numeric §14.6 gates live in `benten-graph/benches/get_create_node.rs`.
- **`end_to_end_create.rs`** — full composed-engine `engine.create_node` / `engine.call("post:create", ...)` path. §14.6 direct target: 100–500µs realistic, 0.1ms aspirational. CI gate: fails > 500µs, warns < 100µs.
- **`get_node_label_only_sub_1us.rs`** — Inv-11 fast-path probe. §9.10 direct: < 1 µs per lookup. CI-GATED.
- **`option_c_evaluator_path_overhead.rs`** — measures the cost of threading `PrimitiveHost::check_read_capability` through every content-returning method (sec-r1-5 flanking gap closure). Informational only; runs against NoAuth + GrantBacked variants for delta attribution. Requires `test-helpers`.
- **`subgraph_cache_hit.rs`** — cold / warm / invalidation cases against the `(handler_id, op, subgraph_cid)` cache key. Informational.
- **`subgraph_cache_hit_80_20_mixed.rs`** — 80/20 read-mostly mixed workload driving real `engine.call(...)` dispatch (G2-B fidelity-gap closure). Informational.
- **`change_event_fanout.rs`** — fan-out latency at N subscribers (1, 4, 16, 64). Informational; pins per-event CPU overhead as subscriber count scales.

All benches use `harness = false` (Criterion CLI flag pass-through).

---

## 7. Thin-engine + composable-graph philosophy check

**What's holding clean.** The big-picture composition contract holds. `benten-engine` IS the orchestrator and very little has leaked.

- Evaluator concerns stay in `benten-eval`. The engine names `PrimitiveHost` + `Evaluator` + `Subgraph` + `SubgraphBuilder` + `TypedCallOp` + the `chunk_sink` + suspension-store traits. It does NOT implement primitive bodies (the engine's `dispatch_typed_call` and `execute_sandbox` are dispatch glue, not primitive bodies — those live in `benten-eval` and `benten-eval::primitives::sandbox` / `wasmtime` respectively).
- Storage concerns stay in `benten-graph`. The engine's only direct redb touch points are the `RedbBackend::open` / `open_in_memory` calls in `builder.rs`, the `Arc<RedbBackend>`-bound `RedbSuspensionStore` adapter, the `BackendGrantReader`, and the `transaction(|tx| …)` closure-based execution path. The resolved-alias `impl Engine` blocks are the ONLY places that name `RedbBackend` inherently per the `engine_generic_cascade_no_inherent_redb_references_outside_default_alias` pin.
- IVM concerns stay in `benten-ivm`. The engine names `benten_ivm::Strategy` at the `view_strategy` accessor boundary and `benten_ivm::Subscriber` at the IVM subscriber slot (the sharpened arch-r1-3 / r6-r3-arch-8 contract). It does NOT name `View` / `Algorithm B` internals.
- Capability concerns stay in `benten-caps`. The engine wraps `CapabilityPolicy` as a trait object slot; constructs `WriteContext` + `ReadContext` at policy-consultation points; routes via `GrantBackedPolicy` + `UcanGroundedPolicy` + `NoAuthBackend` + `UCANBackend`. The capability policy IS pluggable.
- Sync concerns stay in `benten-sync`. The engine wraps `benten_sync::crdt::LoroDoc` + `benten_sync::transport::Endpoint` + `benten_sync::peer_id::PeerId` behind `AtriumHandle` but does NOT implement the CRDT or the iroh transport.
- Identity concerns stay in `benten-id`. The engine consumes `Keypair` + `PublicKey` + `Signature` + `Ucan` + `Did` + `Acceptor` and dispatches typed-CALL ops through them.
- DSL concerns stay in `benten-dsl-compiler` (and the engine deliberately does NOT depend on it — `tests/no_dsl_compiler_dep.rs` pins this).

**Class B β shape is honored.** `Engine::read_node_as(principal, cid)` is the public `_as` flanking entry; `engine_crud.rs::get_node` is the engine-internal-default read with `actor_cid: None`. No `read_node_as` from internal engine call sites; the evaluator threads the principal through `read_node_as` only when dispatching a plugin's read. No stale `todo!()` stubs at the `engine_wait.rs:1011-1026` window cited in the brief — those line numbers now host the implementations of `get_node_label_only`, `put_node`, and `read_node_as`.

**Mild flags worth noting (not blockers, just inventory).**

- The `crate::engine` module's resolved-alias `impl Engine` block (the `#[cfg(not(feature = "browser-backend"))] impl Engine` after line ~2080) is large and growing — it carries `register_module_bytes`, `register_subgraph` (both arms), `register_crud`, `call` + variants, `trace`, `handler_to_mermaid`, `handler_predecessors`, and all of the dispatch-flow helpers. Several recent additions take the form `#[cfg(not(target_arch = "wasm32"))] pub async fn ...` (Atrium surfaces inside the same `impl Engine` block). Phase-3-backlog §1.2-followup is the named destination for "impl-block cascade" (lift methods to `impl<B: GraphBackend>` where possible) — many of these methods could live on the generic block.
- `engine.rs` itself at ~4471 LOC is past the comfortable readable threshold. The R6 Wave-2 split already moved diagnostics + caps + views + crud + modules + transaction + snapshot out; `engine.rs` retains the dispatch core + the registration paths + the apply-atrium-merge orchestrator + the `EngineGeneric` struct + the resolved-alias `impl Engine` block. The most natural next split would peel `register_subgraph*` (with the canary clippy `too_many_lines` allow attributes already in place) into its own `engine_register.rs`, but this is hygiene, not correctness.
- `engine_sync.rs` at ~1805 LOC is the second-largest. Contains a substantial amount of canary scope — the on-the-wire `DeviceAttestationEnvelope` types, the handshake plumbing, and the Atrium session shape. This will continue to grow as the G16-D handshake protocol body, the G16-C MST diff sync driver, and the G14-D UCAN-grant exchange surfaces land. The split point would be `engine_sync_attestation.rs` (the V2 envelope + verify path) vs the `AtriumHandle` proper.
- The `Engine::call` family currently has `#[cfg(not(target_arch = "wasm32"))]` on `open_atrium` + `apply_atrium_merge` + `dispatch_typed_call_public` directly inside the resolved-alias `impl Engine` block. The cfg-on-method pattern is the right answer (the alternative — pulling these into their own modules — adds module-discovery cost without reducing engineer cognitive load), but it does mean a wasm32 build sees a smaller `Engine` surface than a native build. Documented + intentional.
- `read_only_snapshot` is a runtime flag rather than a type-level mode (`Engine<ReadOnly>` vs `Engine<ReadWrite>`). Every user-mutation path checks the flag and surfaces `E_BACKEND_READ_ONLY`. The defense-in-depth: the underlying `RedbBackend` is a transient tempdir-resident shape that the snapshot's CID was computed over, so a runtime check is sufficient correctness for the snapshot-blob invariant. A type-level mode would be cleaner but would cascade across every user-facing CRUD signature.
- Compromise #17 (in-memory module-bytes registry) is CLOSED at HEAD: `register_module_bytes` persists via `RedbBlobBackend` + mirrors into the in-memory cache; `rehydrate_module_bytes_from_zone` rebuilds the cache at engine open. Sibling Compromise #18 (in-memory handler-version chain) is CLOSED via the `system:HandlerVersion` zone-Node persistence + `rehydrate_handler_version_chains_from_zone`. Compromise #21 (manifest-signing wire-through) is CLOSED via `verify_manifest_with_mode` running BEFORE install-time persistence. None of these are permanent.
- The `installed_modules` / `module_bytes` / `handler_version_chain` BTreeMaps on `EngineInner` are all in-memory caches over durable backing — read-mostly, cheap to rebuild. Each documents its Compromise # + Phase-3 promotion path.
- The double-encoded HLC `system_time_ms_for_atrium_hlc()` helper at `engine_sync.rs:89` is small and stays here only because it composes with the Atrium row-4a inbound-sync-frame skew classifier — it would be a candidate for migration into `benten-core::hlc` if a sibling consumer arises.

**Well-respected examples worth pointing at as load-bearing models.**

- The Class B β shape (`Engine::read_node_as` public, `Engine::get_node` engine-internal-default via the public surface but with no principal, `pub(crate)` accessors for hot-path internal reads): clean separation of "trusted internal default" from "explicitly attributed boundary call". This is the architectural pattern the rest of the plugin-runtime work compose on.
- The `CapRecheckFn` shared scaffold at `cap_recheck.rs`: extract first, no inline-then-refactor, both consumers (G14-D delivery gate + G15-A materialization gate) cite the module directly; a no-refactor pin enforces the contract. Worth emulating for future dual-consumer seams.
- The `HandlerRoute` enum + `HandlerRouteLog` at `handler_router.rs`: one typed seam, two consumers (EMIT + SUBSCRIBE); a future drift between them is structurally impossible because the variant lives in one place. Closes the producer/consumer drift cluster at the runtime layer.
- The cap-snapshot-hash module (`cap_snapshot_hash.rs`): the algorithm is a pure function; test pins assert the exact bytes; CLR-2 + r4b-cap-2 BLOCKER closure documents both dimensions (length-prefixed lists defending against ambiguous concatenation, sorted lists making the hash order-stable). Worth pointing at as "how to ship a cryptographic invariant where the algorithm IS the spec".
- The two-phase write discipline in `primitive_host.rs` (single commit boundary, rollback-on-error parity, attribution-fidelity at buffer time): named compromise #5 explicitly + sharpened across Phase-1 → Phase-3. The clean shape buys both audit-attribution correctness AND transaction atomicity simultaneously.

---

## 8. Phase 3.5 + Phase 4 expectations

### 8a. Materializer (G23-B)

Per D-3.5-2 option (b), the schema-driven rendering materializer compiler lives as a sub-module of `benten-engine` rather than a new crate. The materializer consumes `SubgraphSpec` produced by `benten-dsl-compiler` and renders schema-driven views. The likely module location is `crates/benten-engine/src/materializer.rs` (or `materializer/mod.rs` if it grows tabular complexity) — sibling to `engine_views.rs`. It will compose `IvmViewReadGate` for per-row gating, `cap_recheck` for the closure shape, and `UserViewSpec` for the registration surface. No new public root API surface — materializer registration goes through `register_user_view` with a new `UserViewInputPattern::Materialized` variant.

### 8b. Schema-driven rendering compiler

The compiler proper lives in `benten-dsl-compiler` (or potentially a new `benten-render-compiler` crate); benten-engine receives the compiled `SubgraphSpec` shape with a materializer-flavored primitive node. The engine wires the new primitive kind into the dispatch path via the existing `subgraph_for_spec` + `primitive_host.rs` plumbing — same shape as STREAM / SUBSCRIBE / WAIT landed in Phase-2b.

### 8c. Admin UI v0 install

The admin UI is a Phase-4 platform-surface deliverable per CLAUDE.md baked-in #15 (the v1-milestone-gate refactor — v1 = Benten Platform end-to-end). It depends on `Engine`'s existing public surface — every operator-observability accessor (`metrics_snapshot`, `audit_sequence`, `change_event_count`, `capability_writes_committed`, `ivm_subscriber_count`, `diagnose_read`, `sync_replica_cap_recheck_calls`, `handler_version_chain`, `walk_versions`, `device_cid`, `effective_actor_cid`, `active_module_capabilities`, `is_module_installed`, `thin_client_metrics`) is admin-UI-facing. The UI itself is composed from engine primitives (subgraphs that READ from `system:*` zones + RENDER into UI nodes) so the engine's existing dispatch path is sufficient — no new public engine surface is gated on admin-UI delivery.

### 8d. Plugin manifest install (Phase 4)

Per CLAUDE.md baked-in #18, plugins are content-addressed shareable subgraphs. The plugin manifest schema lives alongside `ModuleManifest` (likely `crate::plugin_manifest`); install lifecycle parallels `install_module` but with the three-layer consent contract: install-time manifest review (user consents to the `requires` + `shares` envelope, both signed by the plugin author), runtime delegation within the manifest envelope (the `CapabilityPolicy` backend validates the chain at access-time), and user-as-root anchor (every capability chain traces back to a user-issued root grant). The engine surface that closes the install side is a new `Engine::install_plugin(manifest, expected_cid, verify_args)` method mirroring `install_module` — same shape, plus an Engine-level "current installed plugin set" surface for the admin UI. The `Engine::read_node_as` boundary is the load-bearing dispatch seam — the evaluator threads the active plugin's principal CID through `read_node_as` for every plugin-attributed read.

### 8e. UCAN revocation observance (PR #199 closure)

CLOSED at HEAD via `EngineCapsHandle::revoke_capability_by_grant_cid` (new method, per the source-line at `engine_caps.rs:246`). The previous gap was that revocation was observable via the durable `system:CapabilityRevocation` Node but the engine's in-memory grant store could be stale until the next backend read. The new method updates BOTH the durable revocation Node AND the in-memory `(actor, zone)` revocation pair set that `apply_atrium_merge`'s per-row cap-recheck consults — surgical revocation by grant CID.

---

## 9. Open questions / unresolved internals

- **Where does materializer live exactly?** D-3.5-2 picks "(b) engine-side sub-module"; the module name + the existing `engine_views.rs` neighbor positioning is the natural shape, but the materializer's compiler-vs-runtime split has not been mapped to a specific file. Open question for Phase-3.5 R1.
- **Cycle detection in the subgraph walker?** Currently the evaluator runs over a DAG (Inv-2 forbids cycles + Inv-7 forbids self-loops). User-registered subgraphs walk the static `SubgraphSpec` at register time and the invariant battery rejects cycles. But at materialization time + during `register_subgraph_replace` hot-reload, no cycle detection runs over the broader graph of handler-CID → handler-CID call edges (the `call_handler` cross-handler dispatch). Phase-3-backlog has not surfaced a finding here; it's worth flagging that handler-call graphs could form cycles that would only fail at iteration-budget exhaustion.
- **Generic-cascade lift (`Engine = EngineGeneric<RedbBackend>` vs full generic).** Phase-3-backlog §1.2-followup names "impl-block cascade" as the carried-forward work. At HEAD, the resolved-alias `impl Engine` block carries ~all of the dispatch core; the `impl<B: GraphBackend> EngineGeneric<B>` block carries constructors + a small number of cross-module accessors + the WAIT TTL GC entry points. The cleanest cascade requires the umbrella `GraphBackend` trait to surface `register_module_bytes` / `get_by_label` / `get_by_property` / closure-based `transaction(|tx| ...)` paths uniformly — currently those are inherent to `RedbBackend`. The v1-assessment-window per CLAUDE.md baked-in #15 includes "engine impl-block generic-cascade lift" as a named pre-tag cleanup.
- **Materializer + subgraph-cache key.** The current `SubgraphCache` keys on `(handler_id, op, subgraph_cid)`. A materializer subgraph that walks per-row would benefit from a cache keyed on something finer (likely `(view_id, row_label, row_cid)`); the existing key shape may or may not generalize. Open until materializer R1.
- **The `ast_cache` invalidation contract under future `register_subgraph_replace` semantics.** Currently the cache invalidates the OLD `handler_cid`'s entries on replace + repopulates the NEW. If Phase-4 ships per-plugin manifest-bound subgraph variants (a plugin author overrides a handler at install-time), the invalidation contract grows a new axis. Open question.
- **EMIT-Named subgraph dispatch.** Per `engine.rs:1865` — the engine-side `emit_with_handler(Named(handler_id))` records the routing decision but does NOT invoke the named subgraph; only the eval-side `benten_eval::primitives::emit::execute` Named arm invokes via `host.call_handler`. The G14-D wave-5a comment names this as a wave-paired (pim-4 §3.10) intentional asymmetry shipping at G16-D — there may still be a small gap on the engine side once the Atrium peer wave closes.

---

End of deep-dive.
