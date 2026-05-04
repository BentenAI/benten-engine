# Architecture

The shape of the engine — crates, boundaries, primitives, invariants, and how a request flows through the layers.

For plain-English orientation, start with [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md). This doc assumes you want depth.

---

## Eight crates

The Rust workspace:

```
crates/
  benten-errors/        # Stable ErrorCode discriminants. Zero Benten-crate deps.
                        # Root of the workspace dependency graph.
  benten-core/          # Node, Edge, Value. Content-addressed hashing
                        # (BLAKE3 + DAG-CBOR + CIDv1). Version-chain primitives.
  benten-graph/         # Storage (KVBackend trait + redb impl), indexes, MVCC
                        # via redb transactions.
  benten-ivm/           # Incremental View Maintenance. Subscribes to graph
                        # changes; updates materialized views. Not known to
                        # the evaluator.
  benten-caps/          # Capability grants as Nodes. Pre-write hook trait.
                        # NoAuthBackend default; GrantBackedPolicy ships alongside.
  benten-eval/          # 12 operation primitives. Iterative evaluator (explicit
                        # stack, not recursive). Structural validation (14
                        # invariants). Transaction primitive.
  benten-dsl-compiler/  # Compiles the textual handler-DSL grammar into
                        # SubgraphSpec. Phase-2b G12-B addition; routes from
                        # devserver into Engine.register_subgraph.
  benten-engine/        # Composes the above into a public API. Wires the
                        # capability hook, storage backend, IVM subscriber.

bindings/
  napi/             # Node.js bindings via napi-rs v3. Same codebase compiles
                    # to native and WASM.

packages/
  engine/           # TypeScript DSL wrapper (@benten/engine).
```

The crate graph is DAG-shaped: `benten-errors` has no Benten dependencies; every other crate imports from it for error discriminants; `benten-engine` sits at the top and is the only crate applications link against.

### Thinness test

Each crate has one responsibility. A reader can use `benten-engine` with `NoAuthBackend` and no IVM subscribers as a content-addressed embedded graph DB, with none of the capability, sync, or application machinery engaged. Features that can live outside the evaluator's main loop are moved into sibling crates.

## Bindings and tooling

Beyond the eight Rust crates, the workspace ships two ancillary trees that exist
to make the engine reachable from JavaScript and to keep developer onboarding
ten minutes from `npx` to a green test:

### `bindings/`

- `bindings/napi/` — Node.js bindings via `napi-rs` v3. The same Rust codebase
  compiles to a native dynamic library (`.node`) for desktop / server and to a
  WASM module for browser / edge runtimes. Auto-generates TypeScript `.d.ts`
  files from `#[napi]` annotations so the TS DSL never hand-writes a binding
  signature. Phase-2a surfaces here include `call_with_suspension`,
  `resume_from_bytes`, `resume_from_bytes_as`, `grant_capability`,
  `revoke_capability`, and the trace + diagnostics APIs.

### `tools/`

- `tools/create-benten-app/` — the `npx create-benten-app <name>` scaffolder.
  Drops a minimal TypeScript project (handler file + smoke test +
  `tsconfig.json`) wired to `@benten/engine` via a `file:` link to the
  workspace `packages/engine`. Closes the Phase-1 zero-config DX promise.
  Templates live under `tools/create-benten-app/template/`.
- `tools/benten-dev/` — the diagnostic CLI. The `inspect-state <path>`
  subcommand reads a DAG-CBOR `ExecutionStateEnvelope` from disk and
  pretty-prints the suspended state.
- `packages/engine-devserver/` — the napi-rs-backed `BentenDevServer`
  TypeScript wrapper that wraps a real `Engine` and exposes
  `replaceHandler` / hot-reload semantics through
  `Engine::register_subgraph_replace`. Phase 2b G12-B's
  `benten-dsl-compiler` landing collapsed the prior parallel-infrastructure
  posture; see ["Devserver path"](#devserver-path) below.

## The 12 operation primitives

```
READ       WRITE       TRANSFORM       BRANCH       ITERATE       WAIT
CALL       RESPOND     EMIT            SANDBOX      SUBSCRIBE     STREAM
```

Each primitive is an Operation Node kind. The evaluator dispatches on `PrimitiveKind` to a per-primitive executor in `crates/benten-eval/src/primitives/`.

All 12 primitives have live executors as of tag `phase-2b-close` (2026-05-03). The "Landed in" column tracks the phase that first wired the executor; every row is in production-runtime use today.

| # | Primitive | Purpose | Landed in |
|---|---|---|---|
| 1 | READ | Retrieve a Node by CID, label, or property | Phase 1 |
| 2 | WRITE | Persist a Node; version-stamps if versioning is enabled on its label | Phase 1 |
| 3 | TRANSFORM | Pure expression evaluation over Values | Phase 1 |
| 4 | BRANCH | Conditional routing (forward-only) | Phase 1 |
| 5 | ITERATE | Bounded collection processing | Phase 1 |
| 6 | WAIT | Suspend until signal or timeout; resume with DAG-CBOR state | Phase 2a |
| 7 | CALL | Invoke another registered subgraph | Phase 1 |
| 8 | RESPOND | Terminal: produce the handler's output | Phase 1 |
| 9 | EMIT | Fire-and-forget change notification | Phase 1 |
| 10 | SANDBOX | WASM computation, fuel-metered, no re-entrancy | Phase 2b |
| 11 | SUBSCRIBE | Reactive change notification (composition point for IVM) | Phase 2b |
| 12 | STREAM | Partial output with back-pressure (SSE, WebSockets, LLM tokens) | Phase 2b |

Phase-1 storage was forward-compatible with Phase-2 primitives: subgraphs containing WAIT / SANDBOX / SUBSCRIBE / STREAM Nodes passed structural validation under Phase 1 and round-tripped through storage, even though their executors were stubbed. That binary-compatibility property still holds — older serialised subgraphs continue to load — but the executor stubs are gone; every PrimitiveKind dispatch arm wires to a live runtime.

## The 14 structural invariants

Validated at registration time or fired at runtime, depending on invariant:

1. **Entry/exit well-formedness.**
2. **Max operation-subgraph depth.**
3. **Max fan-out per Node.**
4. **SANDBOX nest depth limit.** (landed Phase 2b)
5. **Max total Nodes per subgraph.**
6. **Max total Edges per subgraph.**
7. **SANDBOX fuel budget.** (landed Phase 2b)
8. **Iteration budget — multiplicative through CALL and ITERATE nesting.** (landed Phase 2a)
9. **Type-safety of Value flows.**
10. **Reachability from entry.**
11. **System-zone Nodes are unreachable from user subgraphs.** (Phase 2a registration + runtime enforcement; Phase 2b extended SUBSCRIBE-pattern validation)
12. **Terminal Node presence.**
13. **Immutability — registered subgraphs are not rewritable.** (Phase 2a 5-row firing matrix)
14. **Causal attribution — every evaluation step carries a principal / handler / grant chain.** (Phase 2a threading)

All 14 invariants are enforced as of `phase-2b-close`. Invariants 1–3, 5–6, 9–10, 12 landed in Phase 1; Invariants 8, 11, 13, 14 in Phase 2a; Invariants 4, 7 in Phase 2b alongside the SANDBOX runtime. See [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md) for per-invariant enforcer + test pins.

## How a request flows

### Read

1. Caller invokes `engine.call(handlerId, action, input)`.
2. `benten-engine` looks up the handler's subgraph CID, reads the subgraph, locates the action entry Node.
3. `benten-eval` walks from the entry Node, dispatching each Operation Node to its executor:
   - READ hits IVM views (O(1)) or falls through to `benten-graph` on a miss.
   - TRANSFORM evaluates a pure expression.
   - BRANCH routes on a condition.
   - CALL invokes another registered subgraph.
   - RESPOND terminates with the handler's output.
4. Reads flow without capability checks by default. Phase 2a extended check-read to the content path for evaluator-driven reads under the grant-backed policy.

### Write (transactional)

1. Same entry flow.
2. The transaction primitive wraps all WRITEs: begin → operations → commit (or rollback).
3. Each WRITE hits the capability hook. Denial aborts the transaction; no ChangeEvents fire.
4. On commit: content is hashed into CIDs, version chains advance (where enabled), the audit sequence advances, ChangeEvents fire to the IVM subscriber.
5. IVM updates materialized views whose subscription patterns match the changes.

### Suspend and resume (landed Phase 2a)

A WAIT primitive suspends the evaluator, produces an `ExecutionStateEnvelope` (DAG-CBOR, content-addressed), and returns a `SuspendedHandle` carrying the envelope CID. The caller persists the envelope bytes.

At resume the engine runs a 4-step protocol before continuing evaluation:

1. **Payload integrity.** The envelope is DAG-CBOR decoded; a mismatch with its declared `payload_cid` fires `E_EXEC_STATE_TAMPERED`.
2. **Principal binding.** The resumption principal is verified against the caller's claimed identity; mismatch fires `E_RESUME_ACTOR_MISMATCH`.
3. **Subgraph pin check.** Each pinned subgraph CID is re-verified against the registered-handler table; drift fires `E_RESUME_SUBGRAPH_DRIFT`.
4. **Capability re-check.** `CapabilityPolicy::check_write` is re-invoked against the persisted head-of-chain grant CID; mid-eval revocation fires `E_CAP_REVOKED_MID_EVAL`.

Only after all four steps pass does evaluation resume.

## Storage

`benten-graph` exposes a `KVBackend` trait. `RedbBackend` is the Phase-1 implementation: ACID, MVCC (concurrent readers with single writer), crash-safe via copy-on-write B-trees. A future WASM implementation will fetch content-addressed Nodes from peer storage.

redb transactions wrap every `put_node_with_context` call. The Phase-2a Inv-13 firing matrix dispatches on the write's `WriteAuthority` (User / EnginePrivileged / SyncReplica): User re-puts of an already-persisted CID fire `E_INV_IMMUTABILITY`; privileged dedup paths return `Ok(cid)` without emitting ChangeEvents or advancing the audit sequence (a named compromise — the privileged dedup is a pure read on the backend even though it passes through the write API).

Content is serialized via `serde_ipld_dagcbor` — the IPLD subset of CBOR with canonical encoding (map keys sorted, no indefinite-length forms). CIDs are v1 with multicodec `0x71` (dag-cbor) and multihash `0x1e` (BLAKE3).

## Incremental View Maintenance

`benten-ivm` subscribes to ChangeEvents from the storage layer and keeps views current. Phase 1 shipped five hand-written views covering the hot paths: capability resolution, content listings, change-event fan-out, principal resolution, and view-staleness tallies. Phase 2b production-registered Algorithm B (dependency-tracked incremental maintenance) with per-view strategy selection (`Strategy::A` / `Strategy::B`) at `Engine::create_user_view`. The Phase-2b dispatch constructs `AlgorithmBView::for_id` for the 5 canonical view IDs that `AlgorithmBView` supports natively; user-defined view IDs that declare `Strategy::B` continue to fall back to `ContentListingView` silently — generalised user-defined Algorithm B handlers are a Phase-3 lift (see [`docs/future/phase-3-backlog.md`](future/phase-3-backlog.md) §5).

The evaluator does not know IVM exists. Views are materialized Nodes; reads hit them via the normal read path.

## Capabilities

`benten-caps` defines a pre-write hook trait:

```rust
fn check_write(&self, ctx: &WriteContext) -> Result<Decision, CapError>;
```

The engine's default is `NoAuthBackend` — allows everything, zero overhead — so embedded single-user deployments don't pay for capability machinery.

A `GrantBackedPolicy` ships alongside: grants are Nodes with `GRANTED_TO` edges, attenuation is verified along the delegation chain, revocation is a Node write. Phase 2a added TOCTOU refresh at five points (transaction commit, CALL entry, every N iterations of ITERATE, WAIT resume, wall-clock boundary) with a dual monotonic + HLC clock source. Phase 3 adds UCAN as another policy backend.

## Determinism

The canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` is stable across x86_64 Linux / macOS / Windows and ARM64 macOS. The `.github/workflows/determinism.yml` workflow computes it on every PR; drift is a merge blocker.

## Devserver path

The developer surface ships in two layers as of `phase-2b-close`:

- `tools/benten-dev` is the diagnostic CLI; its `inspect-state <path>`
  subcommand reads a DAG-CBOR `ExecutionStateEnvelope` from disk and
  pretty-prints the suspended state. It does not host a runtime.
- `packages/engine-devserver` is the napi-rs-backed `BentenDevServer`
  TypeScript wrapper that wraps the real `Engine` (Phase 2b G12-B
  landed `benten-dsl-compiler` so devserver-side handler text now
  compiles to `SubgraphSpec` and registers through
  `Engine::register_subgraph` rather than running parallel in-memory
  infrastructure). Hot reload, grant preservation across reload,
  in-flight call ordering, suspension-handle survival across reload,
  and audit-sequence invariance on reload are exercised against the
  same Engine APIs production callers use.

The Phase-2a `ReloadCoordinator` / `CallGuard` concurrency machinery —
`RwLock<HandlerTable>::write()` serialising concurrent reload-bumps,
each in-flight call's `Arc<HandlerVersion>` snapshot keeping its
pre-reload version live for the duration of the call — survived the
Phase-2b cutover unchanged. That ordering shape is concurrency
coordination, not storage, and is orthogonal to whether the registry
is in-memory or engine-backed.

## The control plane is the graph

Everything the engine uses at runtime is in the graph: capability grants (`CapabilityGrant` Nodes with `GRANTED_TO` edges), registered handlers (content-addressed subgraphs, immutable once registered), IVM view definitions (ViewDefinition Nodes with a strategy property), user preferences, change-event queues. The engine does not read configuration from YAML or a config service; it reads Nodes.

---

For the paths this will take next, see [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md) "The path from here." For every error the engine surfaces, see [`ERROR-CATALOG.md`](ERROR-CATALOG.md).
