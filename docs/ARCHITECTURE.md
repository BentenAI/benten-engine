# Architecture

The shape of the engine — crates, boundaries, primitives, invariants, and how a request flows through the layers.

For plain-English orientation, start with [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md). This doc assumes you want depth.

---

## Ten crates

The Rust workspace ships ten Rust crates plus the napi bindings + the
TypeScript DSL wrapper. The 8 → 10 crate transition completed in
Phase 3 — `benten-id` (9th, identity + claims) and `benten-sync`
(10th, sync runtime — native-only) landed and were filled in across
Phase 3's implementation cluster. The narrative below is the final
Phase-3-close shape.

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
  benten-id/            # 9th crate (Phase 3). Ed25519 keypair
                        # management with secret-key zeroization on
                        # drop, did:key DID generation (z-multibase
                        # prefix + 0xed01 multicodec per W3C spec),
                        # UCAN claim envelope + chain validation +
                        # nbf/exp time-window enforcement, VC issuance
                        # / verification with `credentialStatus`
                        # revocation, DID rotation with `superseded_by`
                        # attestation chain, MultiSigSurface trait
                        # (Ed25519SingleKey default impl + threshold
                        # extension point), and the device-DID
                        # capability-attestation surface
                        # (replay-resistant via nonce + freshness
                        # window + nonce-store). Dependency edges:
                        # `ed25519-dalek`, `ssi`, `blake3`,
                        # `serde_ipld_dagcbor`, `zeroize`, `secrecy`,
                        # `subtle`, `getrandom` only — NO edges back to
                        # `benten-graph` / `benten-eval` /
                        # `benten-engine`.
  benten-sync/          # 10th crate (Phase 3). Atrium P2P sync layer:
                        # iroh transport (loopback + relay + holepunch
                        # fallback), Loro CRDT for per-property LWW
                        # with HLC ordering, MST diff for delta
                        # computation, DID handshake protocol with
                        # mutual auth, and the `host:atrium:*`
                        # capability surface (`publish_view_result`
                        # UCAN-gated — view-result replication does NOT
                        # introduce a new trust-policy primitive).
                        # NATIVE-ONLY: the crate is excluded from
                        # `wasm32` targets so browser tabs and edge
                        # thin-compute deployments do not carry iroh /
                        # Loro in their bundles. Browser / edge
                        # surfaces sync THROUGH a full peer via
                        # authenticated thin-client protocol; they are
                        # not full Atrium peers themselves.
  benten-dsl-compiler/  # Compiles the textual handler-DSL grammar
                        # into SubgraphSpec. Phase-2b addition; routes
                        # from devserver into Engine.register_subgraph.
  benten-engine/        # Composes the above into a public API. Wires
                        # the capability hook, storage backend, IVM
                        # subscriber. Phase 3 added the Atrium DSL
                        # session-handle
                        # `engine.atrium({config}).join()`.

bindings/
  napi/             # Node.js bindings via napi-rs v3. Compiles to a
                    # native dynamic library (`.node`) for desktop /
                    # server AND to a wasm32 module for browser / edge
                    # runtimes. The wasm32 build is a thin compute
                    # surface: stateless reads against snapshot data
                    # + writes via fetch to a full peer; no Loro /
                    # iroh / direct sync state in the bundle.

packages/
  engine/           # TypeScript DSL wrapper (@benten/engine). Phase 3
                    # adds the Atrium walkthrough surface (peer connect /
                    # sync trigger / UCAN grant flow / DID resolution)
                    # over the napi surface.
```

A workspace test pin verifies all ten crate names + the `native-only`
annotation on `benten-sync` are present in this document, so the
Phase-3-close shape described above is the durable narrative.

The crate graph is DAG-shaped:

- `benten-errors` has no Benten dependencies; every other crate imports
  from it for error discriminants.
- `benten-id` depends only on third-party crypto crates (no edges back
  to graph / eval / engine), so the identity surface is reusable
  outside the engine if needed.
- `benten-sync` depends on `benten-id` (DID + UCAN) and `benten-core`
  (Node + content-addressing) but NOT on `benten-eval` — sync is not
  allowed to walk the evaluator.
- `benten-engine` sits at the top and is the only crate applications
  link against.

### Thinness test

Each crate has one responsibility. A reader can use `benten-engine` with `NoAuthBackend` and no IVM subscribers as a content-addressed embedded graph DB, with none of the capability, sync, or application machinery engaged. Features that can live outside the evaluator's main loop are moved into sibling crates.

## Bindings and tooling

Beyond the ten Rust crates, the workspace ships two ancillary trees that exist
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
  `Engine::register_subgraph_replace`. Phase 2b landed
  `benten-dsl-compiler` and collapsed the prior parallel-infrastructure
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

`benten-ivm` subscribes to ChangeEvents from the storage layer and keeps views current. Phase 1 shipped five hand-written views covering the hot paths: capability resolution, content listings, change-event fan-out, principal resolution, and view-staleness tallies. Phase 2b production-registered Algorithm B (dependency-tracked incremental maintenance) with per-view strategy selection (`Strategy::A` / `Strategy::B`) at `Engine::create_user_view`. Phase 3 generalised Algorithm B beyond the 5 canonical view IDs: `Algorithm::register(view_id, label_pattern, projection)` (and the budget-aware sibling `Algorithm::register_with_budget`) instantiates a generic single-loop kernel (`benten_ivm::algorithm_b::GenericKernel`) for non-canonical view IDs keyed on `(label_pattern, projection)`, with the `AnchorPrefix` selector lift shipping in `register_user_view`. The drift-detector proptest harness at `crates/benten-ivm/tests/algorithm_b_drift_detector.rs` runs incremental-vs-rebuild parity end-to-end (5 pins × 1 000 cases). The `ContentListingView` silent-fallback for user-defined Strategy::B views is RETIRED.

The evaluator does not know IVM exists. Views are materialized Nodes; reads hit them via the normal read path.

## Capabilities

`benten-caps` defines a pre-write hook trait:

```rust
fn check_write(&self, ctx: &WriteContext) -> Result<Decision, CapError>;
```

The engine's default is `NoAuthBackend` — allows everything, zero overhead — so embedded single-user deployments don't pay for capability machinery.

A `GrantBackedPolicy` ships alongside: grants are Nodes with `GRANTED_TO` edges, attenuation is verified along the delegation chain, revocation is a Node write. Phase 2a added TOCTOU refresh at five points (transaction commit, CALL entry, every N iterations of ITERATE, WAIT resume, wall-clock boundary) with a dual monotonic + HLC clock source. Phase 3 landed `UCANBackend` — a durable UCAN-grant policy backend over `benten-id`'s claim envelope + chain validation surface. UCAN grants attenuate on delegation, propagate revocations, and validate `nbf`/`exp` time-windows at chain-walk time; signature comparison is constant-time via `subtle::ConstantTimeEq`. Compromise #11 (IVM views per-row read-gate) closed end-to-end in Phase 3 via the label-hint extraction + delivery-side filtering composition.

## Determinism

The canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` is stable across x86_64 Linux / macOS / Windows and ARM64 macOS. The `.github/workflows/determinism.yml` workflow computes it on every PR; drift is a merge blocker.

## Devserver path

The developer surface ships in two layers as of `phase-2b-close`:

- `tools/benten-dev` is the diagnostic CLI; its `inspect-state <path>`
  subcommand reads a DAG-CBOR `ExecutionStateEnvelope` from disk and
  pretty-prints the suspended state. It does not host a runtime.
- `packages/engine-devserver` is the napi-rs-backed `BentenDevServer`
  TypeScript wrapper that wraps the real `Engine` (Phase 2b landed
  `benten-dsl-compiler` so devserver-side handler text now compiles
  to `SubgraphSpec` and registers through `Engine::register_subgraph`
  rather than running parallel in-memory infrastructure). Hot
  reload, grant preservation across reload, in-flight call ordering,
  suspension-handle survival across reload, and audit-sequence
  invariance on reload are exercised against the same Engine APIs
  production callers use.

The Phase-2a `ReloadCoordinator` / `CallGuard` concurrency machinery —
`RwLock<HandlerTable>::write()` serialising concurrent reload-bumps,
each in-flight call's `Arc<HandlerVersion>` snapshot keeping its
pre-reload version live for the duration of the call — survived the
Phase-2b cutover unchanged. That ordering shape is concurrency
coordination, not storage, and is orthogonal to whether the registry
is in-memory or engine-backed.

## The control plane is the graph

Everything the engine uses at runtime is in the graph: capability grants (`CapabilityGrant` Nodes with `GRANTED_TO` edges), registered handlers (content-addressed subgraphs, immutable once registered), IVM view definitions (ViewDefinition Nodes with a strategy property), user preferences, change-event queues. The engine does not read configuration from YAML or a config service; it reads Nodes.

## Plugins and engine extensions

Two distinct categories of "extensibility" — different shapes, different trust models, different surfaces. Both are first-class.

### App-level plugins — subgraphs

A plugin is a **subgraph of operation Nodes** — handlers, materializers, SANDBOX nodes, READ/WRITE/etc. — packaged as content-addressed graph: importable, shareable, replicatable, and editable across Atriums. There is no separate plugin runtime; the engine evaluator walks plugin subgraphs the same way it walks any other handler.

**Identity.** Each plugin has its own DID + an attenuated UCAN delegated by the user at install. Walks of plugin subgraphs run with the plugin's principal active; capability checks fire against the plugin's UCAN chain.

**Trust model — three layers.**

1. **User-as-root.** Every cap chain traces back to a user-issued root. P2P plugin discovery (Phase 8) does not weaken this — the user-mint root is still the trust anchor.
2. **Install-time manifest.** Two halves: `requires` (caps the plugin needs) and `shares` (policy for what other plugins are allowed to receive). Both are signed by the plugin author so they cannot drift post-install. The user reviews the manifest and consents to the *envelope*, not to each runtime access.
3. **Runtime delegation within the manifest envelope.** Plugin A can delegate a UCAN to plugin B if and only if B's request fits A's manifest `shares` policy. The CapabilityPolicy backend validates the chain at access-time: chain-traces-to-user-root + each delegation step fits source policy + requested cap is within attenuation envelope.

**Engine-side surface.** The evaluator's read pathway threads the active principal through `Engine::read_node_as(principal, cid)` — the public surface for any read attributed to a non-trusted principal. Engine internals (IVM, sync, view materialization, audit) reach the unchecked storage read via `self.backend.get_node(cid)` directly — the backend field + accessor are both `pub(crate)`, so external crates physically cannot bypass the policy gate. Plugin authors do not call either path: they author graph nodes; the evaluator is the only caller of `_as`. Mirrors the existing `Engine::call_as` precedent. The four prior `todo!()` stubs in `crates/benten-engine/src/engine_wait.rs` (`put_node` + `read_node_with_policy` (renamed to `read_node_as`) + the test-only read-grant helper + the dead bench-helper sibling) landed during pre-v1 cleanup and are gone; independent of Phase-4 plugin manifest schema decisions.

**Private namespaces.** A plugin's writes go to a DID-scoped namespace whose cap is held by the plugin's DID. Manifest `shares=none` for that namespace blocks delegation; the engine refuses to issue cross-plugin caps for it. Gives plugins a sovereign space (AI agents' working memory, intermediate state) without breaking the cross-plugin sharing model — same machinery, different policy.

### Engine-level extensions — Rust crates

Compile-time linked into the engine binary. Rare. For custom IVM strategies, alternate transports (post-iroh — shaped relays, Nostr, Tor), alternate persistence backends (post-redb — sled, fjall, cloud-KV), custom signature schemes (post-Ed25519 — X25519, BLS, post-quantum), performance-critical primitives that need raw Rust speed beyond SANDBOX.

**Trust model.** "You compiled this into your engine binary." Same trust posture as Benten core itself. No UCAN, no manifest, no `read_node_as` boundary. An engine extension that wants to violate invariants can — the boundary is `cargo` and code review, not the type system.

**Audience.** People building the platform itself, not app users. The two categories are intentionally separate worlds; trust models do not transfer between them in either direction.

---

For the paths this will take next, see [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md) "The path from here." For every error the engine surfaces, see [`ERROR-CATALOG.md`](ERROR-CATALOG.md).
