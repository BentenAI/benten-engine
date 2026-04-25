# Architecture

The shape of the engine — crates, boundaries, primitives, invariants, and how a request flows through the layers.

For plain-English orientation, start with [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md). This doc assumes you want depth.

---

## Seven crates

The Rust workspace:

```
crates/
  benten-errors/    # Stable ErrorCode discriminants. Zero Benten-crate deps.
                    # Root of the workspace dependency graph.
  benten-core/      # Node, Edge, Value. Content-addressed hashing
                    # (BLAKE3 + DAG-CBOR + CIDv1). Version-chain primitives.
  benten-graph/     # Storage (KVBackend trait + redb impl), indexes, MVCC
                    # via redb transactions.
  benten-ivm/       # Incremental View Maintenance. Subscribes to graph
                    # changes; updates materialized views. Not known to
                    # the evaluator.
  benten-caps/      # Capability grants as Nodes. Pre-write hook trait.
                    # NoAuthBackend default; GrantBackedPolicy ships alongside.
  benten-eval/      # 12 operation primitives. Iterative evaluator (explicit
                    # stack, not recursive). Structural validation (14
                    # invariants). Transaction primitive.
  benten-engine/    # Composes the above into a public API. Wires the
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

Beyond the seven Rust crates, the workspace ships two ancillary trees that exist
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
- `tools/benten-dev/` — the Phase-2a developer-server tool. Watches handler
  source files, hot-reloads registered subgraphs, and preserves capability
  grants + in-flight evaluations across reload. Carries an `inspect-state`
  subcommand that pretty-prints a serialized `ExecutionStateEnvelope`.
  Currently runs its own minimal in-memory handler registry + grant table
  rather than threading through `Engine::register_subgraph` — see
  ["Devserver divergence (Phase-2a posture)"](#devserver-divergence-phase-2a-posture)
  below for the rationale and the Phase-2b cutover plan.

## The 12 operation primitives

```
READ       WRITE       TRANSFORM       BRANCH       ITERATE       WAIT
CALL       RESPOND     EMIT            SANDBOX      SUBSCRIBE     STREAM
```

Each primitive is an Operation Node kind. The evaluator dispatches on `PrimitiveKind` to a per-primitive executor in `crates/benten-eval/src/primitives/`.

| # | Primitive | Purpose | Live |
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

Subgraphs containing Phase-2 primitives pass structural validation at Phase 1 (so Phase-1 and Phase-2 graphs are binary-compatible and round-trip through storage). Phase-2 executors return `E_PRIMITIVE_NOT_IMPLEMENTED` at call time until they land.

## The 14 structural invariants

Validated at registration time or fired at runtime, depending on invariant:

1. **Entry/exit well-formedness.**
2. **Max operation-subgraph depth.**
3. **Max fan-out per Node.**
4. **SANDBOX nest depth limit.** (Phase 2b)
5. **Max total Nodes per subgraph.**
6. **Max total Edges per subgraph.**
7. **SANDBOX fuel budget.** (Phase 2b)
8. **Iteration budget — multiplicative through CALL and ITERATE nesting.** (Phase 2a)
9. **Type-safety of Value flows.**
10. **Reachability from entry.**
11. **System-zone Nodes are unreachable from user subgraphs.** (Phase 2a runtime enforcement)
12. **Terminal Node presence.**
13. **Immutability — registered subgraphs are not rewritable.** (Phase 2a 5-row firing matrix)
14. **Causal attribution — every evaluation step carries a principal / handler / grant chain.** (Phase 2a threading)

Invariants 1–3, 5–6, 9–10, 12 ship in Phase 1. Invariants 8, 11, 13, 14 ship in Phase 2a. Invariants 4, 7 ship in Phase 2b with SANDBOX.

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
4. Reads flow without capability checks by default. Phase 2a extends check-read to the content path for evaluator-driven reads under the grant-backed policy.

### Write (transactional)

1. Same entry flow.
2. The transaction primitive wraps all WRITEs: begin → operations → commit (or rollback).
3. Each WRITE hits the capability hook. Denial aborts the transaction; no ChangeEvents fire.
4. On commit: content is hashed into CIDs, version chains advance (where enabled), the audit sequence advances, ChangeEvents fire to the IVM subscriber.
5. IVM updates materialized views whose subscription patterns match the changes.

### Suspend and resume (Phase 2a)

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

`benten-ivm` subscribes to ChangeEvents from the storage layer and keeps views current. Phase 1 ships five hand-written views covering the hot paths: capability resolution, content listings, change-event fan-out, principal resolution, and view-staleness tallies. Phase 2b generalizes this into Algorithm B (dependency-tracked incremental maintenance) with per-view strategy selection so applications can register their own views.

The evaluator does not know IVM exists. Views are materialized Nodes; reads hit them via the normal read path.

## Capabilities

`benten-caps` defines a pre-write hook trait:

```rust
fn check_write(&self, ctx: &WriteContext) -> Result<Decision, CapError>;
```

The engine's default is `NoAuthBackend` — allows everything, zero overhead — so embedded single-user deployments don't pay for capability machinery.

A `GrantBackedPolicy` ships alongside: grants are Nodes with `GRANTED_TO` edges, attenuation is verified along the delegation chain, revocation is a Node write. Phase 2a adds TOCTOU refresh at five points (transaction commit, CALL entry, every N iterations of ITERATE, WAIT resume, wall-clock boundary) with a dual monotonic + HLC clock source. Phase 3 adds UCAN as another policy backend.

## Determinism

The canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` is stable across x86_64 Linux / macOS / Windows and ARM64 macOS. The `.github/workflows/determinism.yml` workflow computes it on every PR; drift is a merge blocker.

## Devserver divergence (Phase-2a posture)

The Phase-2a `tools/benten-dev` developer server runs **parallel infrastructure**
to the canonical engine path rather than threading through it:

| Concern | Canonical engine path | `tools/benten-dev` Phase-2a path |
|---|---|---|
| Handler registry | `Engine::register_subgraph` against the redb-backed graph | In-memory `OnceLock<Mutex<HashMap<HandlerId, Vec<HandlerVersion>>>>` |
| Handler CID | Canonical `Subgraph` DAG-CBOR + multicodec `0x71` + multihash `0x1e` | Dev-only surrogate: `BLAKE3(handler_id ‖ op ‖ source ‖ version_tag)` |
| Capability grants | `benten-caps` policy backend, persisted as `system:CapabilityGrant` Nodes | In-memory grant table, source-text-keyed |
| WAIT suspension | Canonical DAG-CBOR `ExecutionStateEnvelope` with `payload_cid` round-trip | Custom `BDEV\x01` magic-prefixed envelope wire format |

This is **intentional Phase-2a posture, not a regression.** The real DSL-text →
`SubgraphSpec` compiler is itself Phase-2b scope (see `docs/DSL-SPECIFICATION.md`),
so the devserver could not thread reloads through `Engine::register_subgraph`
without a compiler boundary that does not yet exist. The Phase-2a in-memory
shape pins the developer-facing contracts (grant preservation across reload,
in-flight call ordering, suspension-handle survival across reload, audit-sequence
invariance on reload) which is what the devserver test suite asserts.

**Phase-2b cutover:** when the DSL compiler lands, `tools/benten-dev` is
refactored to compile source → `SubgraphSpec` → `engine.register_subgraph(spec)`
and to drop both the in-memory registry and the `BDEV\x01` envelope shim.
Tracked in `.addl/phase-2b/00-scope-outline.md` §7a "Devserver → Engine routing".
The Phase-2a `ReloadCoordinator` / `CallGuard` machinery — together with
`DevServer`'s `RwLock<HandlerTable>::write()` + `Arc<HandlerVersion>`
snapshot ordering (the `RwLock` serialises concurrent reload-bumps; each
in-flight call's `Arc<HandlerVersion>` snapshot keeps the pre-reload
version live for the duration of the call) — survives the cutover. That
ordering shape is concurrency-coordination, not storage, and applies
unchanged when the real engine is wired.

## The control plane is the graph

Everything the engine uses at runtime is in the graph: capability grants (`CapabilityGrant` Nodes with `GRANTED_TO` edges), registered handlers (content-addressed subgraphs, immutable once registered), IVM view definitions (ViewDefinition Nodes with a strategy property), user preferences, change-event queues. The engine does not read configuration from YAML or a config service; it reads Nodes.

---

For the paths this will take next, see [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md) "The path from here." For every error the engine surfaces, see [`ERROR-CATALOG.md`](ERROR-CATALOG.md).
