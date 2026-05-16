# Architecture

The shape of the engine — crates, boundaries, primitives, invariants, and how a request flows through the layers.

For plain-English orientation, start with [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md). This doc assumes you want depth.

---

## Twelve crates (post-Phase-4-Foundation)

The Rust workspace ships twelve Rust crates plus the napi bindings + the
TypeScript DSL wrapper. The 8 → 10 crate transition completed in
Phase 3 — `benten-id` (9th, identity + claims) and `benten-sync`
(10th, sync runtime — native-only) landed and were filled in across
Phase 3's implementation cluster. **Phase 4-Foundation extended the
workspace to twelve crates** per the post-R1-triage ratification
(`r1-triage.md` §1 ratification #1):

- `benten-platform-foundation` (11th — schema-driven rendering
  compiler + materializer pipeline + plugin manifest + `Renderer` trait
  abstraction. This crate is intentionally broader than other crates
  because it's the v1 platform-shippable surface — narrower three-or-four
  separate platform crates rejected per arch-r1-8 closure).
- `benten-renderer-tauri` (12th — Tauri 2.x renderer engine extension
  per CLAUDE.md baked-in #19; compile-time linked; user trust = "you
  compiled this in"; distinct from app-level plugins which are
  subgraphs).

The narrative below is the Phase-4-Foundation-close shape.

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
                        # fallback) behind the `transport_trait::
                        # Transport` abstraction boundary (RATIFIED
                        # §15.3 #1; `IrohTransport` is the pre-v1
                        # concrete impl, post-iroh swaps land as
                        # engine extensions per CLAUDE.md #19), Loro
                        # CRDT for per-property LWW
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
                        # into benten_core::Subgraph. Phase-2b addition;
                        # routes from devserver into
                        # Engine.register_subgraph.
  benten-engine/        # Composes the above into a public API. Wires
                        # the capability hook, storage backend, IVM
                        # subscriber. Phase 3 added the Atrium DSL
                        # session-handle
                        # `engine.atrium({config}).join()`.
  benten-platform-foundation/
                        # 11th crate (Phase 4-Foundation). The v1
                        # platform-shippable surface. Hosts the
                        # schema-driven rendering compiler
                        # (typed-field-Node vocabulary → renderable
                        # SubgraphSpec), the materializer pipeline
                        # (HtmlJsonMaterializer + IVM-subgraph
                        # generalisation), the full plugin manifest
                        # surface (install-time consent + per-plugin
                        # DID + manifest envelope chain validation +
                        # private namespace caps + DAG-shape
                        # versioning), the admin UI v0 plugin
                        # subgraph + 4-category navigation IA, and
                        # the `Renderer` trait abstraction with
                        # `BrowserRender` (browser-wasm32) as default
                        # impl. Per arch-r1-8 closure, the crate is
                        # intentionally broader than other crates
                        # because every part of it composes into one
                        # platform-shippable boundary.
  benten-renderer-tauri/
                        # 12th crate (Phase 4-Foundation). Tauri 2.x
                        # renderer ENGINE EXTENSION per CLAUDE.md
                        # baked-in #19: compile-time linked Rust crate
                        # implementing the `Renderer` trait for
                        # embedded-webview deployment shape (c).
                        # Single typed `IPC_METHODS` binding slice
                        # (name + `CapRequirement`; co-located so the
                        # allowlist-to-cap-binding bijection is
                        # structural, not test-guarded) + locked CSP +
                        # gate-only `dispatch_ipc`. T3 rung-2 fails
                        # CLOSED by construction (no empty-string
                        # sentinel, no fail-OPEN fallback). In-process
                        # IPC protocol. Trust = "you compiled this in."
                        # NOT an app-level plugin subgraph.

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

A workspace test pin verifies all twelve crate names + the
`native-only` annotation on `benten-sync` are present in this document
(see `crates/benten-engine/tests/architecture_md_12_crate_count_post_phase_4_foundation_canaries.rs`),
so the Phase-4-Foundation-close shape described above is the durable
narrative.

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

Beyond the twelve Rust crates, the workspace ships two ancillary trees that exist
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
- `tools/benten-admin-shell/` — Phase-4-Foundation R6-FP-E integrator
  binary for deployment shape (c). Production caller for
  `benten-renderer-tauri`'s `TauriRenderer` + `InProcessSessionBridge`;
  composes them with the canonical admin-UI-v0 manifest envelope into
  `AdminShellState::dispatch`. Default-mode boot prints the IPC method-
  cap-binding map + locked CSP header. With the `tauri` cargo feature
  enabled, `tauri_boot::run` wires the REAL Tauri 2.x runtime through
  `Builder::default().invoke_handler(...).run(generate_context!())`;
  the webview-driven `tauri-driver` E2E at
  `tools/benten-admin-shell/tests/e2e_webview_smoke.rs` drives a real
  Tauri command-invoke roundtrip + CSP eval-block assertion via
  fantoccini-rustls WebDriver client (linux substantive, macos build-
  only smoke per upstream WKWebView limitation, Windows deferred per
  `docs/future/phase-4-backlog.md §3.6`). Default-mode substantive
  pin at `tools/benten-admin-shell/tests/e2e_admin_shell_ipc.rs`
  exercises the T3 three-rung defense + bridge resolve through 9
  happy-path + negative-arm cases. Closes R6 R6-R1 br-r6-r1-3 BOTH
  halves (path-a-FULL).
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

### Registering a user-defined view

Callers extend IVM beyond the 5 canonical views by constructing a
`UserViewSpec` and calling `Engine::register_user_view` (durable) or
`Engine::create_user_view` (transient). The public surface:

- **`UserViewSpec::builder()`** — fluent builder requiring `id` +
  `input_pattern`; `strategy` defaults to `Strategy::B`.
  `Strategy::A` is reserved for the 5 hand-written Phase-1 views and
  `Strategy::Reserved` for future algorithmic families (renamed from
  `Strategy::C` at G23-0a per arch-r1-14); both are refused at
  registration time with typed errors.
- **`UserViewInputPattern`** — two-variant selector vocabulary:
  `Label(String)` (every change event whose Node carries the
  matching label) and `AnchorPrefix(String)` (every change event
  whose anchor id starts with the given prefix). Canonical view ids
  require `Label` and are fail-loud on `AnchorPrefix`
  (`E_VIEW_LABEL_MISMATCH`).
- **Projection** lives on the kernel side at
  `benten_ivm::algorithm_b::{Algorithm::register,
  Algorithm::register_with_budget}` — the engine wraps the
  user-facing builder into a `(view_id, label_pattern, projection)`
  triple that instantiates the generic single-loop kernel
  (`GenericKernel`) for non-canonical ids.
- **Budget semantics:** `register_with_budget(view_id, pattern,
  projection, budget: u64)` caps per-update work; each matching
  write consumes one budget unit and exhaustion fires the
  budget-exhausted outcome. `budget == u64::MAX` is the
  effectively-unbounded sentinel.
- Reads route through `Engine::read_view` /
  `Engine::read_view_with(ReadViewOptions)`; strict mode fires
  `E_IVM_VIEW_STALE` when materialization is behind the latest write.

The TS DSL surface mirrors the Rust shape one-for-one via
`packages/engine/src/views.ts::validateUserViewSpec`, with napi
round-tripping the field names. The drift-detector proptest at
`crates/benten-ivm/tests/algorithm_b_drift_detector.rs` runs
incremental-vs-rebuild parity end-to-end so generalised Algorithm B
is held to the same correctness bar as the canonical kernels.

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
  to `benten_core::Subgraph` and registers through
  `Engine::register_subgraph`
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

**Transport abstraction boundary (P2P sync layer).** The alternate-transport extension point above (post-iroh — Tor / Nostr-relay / shaped relay) is named by the `benten_sync::transport_trait::{Transport, TransportEndpoint, TransportConnection}` traits (RATIFIED 2026-05-15, §15.3 #1). These abstract the connection layer the Atrium P2P sync runtime exercises (connect / accept / send-bytes / recv-bytes / status / close), routing the transport's address type through the `TransportEndpoint::Addr` associated type so no iroh-concrete type (`iroh::EndpointAddr`) appears in the trait contract. The pre-v1 concrete impl, `IrohTransport`, delegates to the existing iroh-backed `transport::Endpoint` / `transport::Connection` with zero behavioral change; the engine-facing API (`engine_sync.rs`) intentionally stays on the concrete newtypes pre-v1 (no behavioral regression) — the boundary existing pre-v1 is the load-bearing deliverable so the v1 surface contract names the abstraction rather than the concrete. A post-v1 `TorTransport` / `NostrRelayTransport` / `ShapedRelayTransport` implements the three traits as compile-time engine extensions per the trust model below. Mirrors the `Renderer`-trait swappability pattern (CLAUDE.md baked-in #17). Compile-fenced against accidental iroh-leak by `crates/benten-sync/tests/transport_trait_boundary.rs` (an iroh-free in-memory mock impl).

**Trust model.** "You compiled this into your engine binary." Same trust posture as Benten core itself. No UCAN, no manifest, no `read_node_as` boundary. An engine extension that wants to violate invariants can — the boundary is `cargo` and code review, not the type system.

**Audience.** People building the platform itself, not app users. The two categories are intentionally separate worlds; trust models do not transfer between them in either direction.

## Thin-client surface — three-rung defense

The browser wasm32 bundle commits to a thin-client posture: it holds in-RAM cache + IndexedDB snapshot data + manifest-store, but contains no `iroh` transport, no `loro` CRDT bytes, no `redb` durable backend, and no `wasmtime` SANDBOX runtime. Full peers (native Rust on user-owned hardware) are the sync participants; the browser tab reads against snapshot data and writes via fetch to a full peer. The architectural commitment is defended at three rungs so a regression at any one rung is caught by another:

1. **Source-side cfg-gating.** `crates/benten-sync/src/lib.rs` fires `compile_error!` for `target_arch = "wasm32"`, and Cargo.toml restricts the crate's dependency block to `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. A wasm32 build attempt fails at compile time, not at runtime. Pinned by `crates/benten-sync/tests/wasm32_excluded.rs`.
2. **Cargo feature-graph closure.** No transitive activation of full-peer-only crates from the browser-bundle root crate, even via shared workspace features. Pinned by `bindings/napi/tests/feature_graph_closure_no_test_helpers_in_production.rs`.
3. **Built-bundle symbol-section audit.** CI's `wasm-browser.yml` runs `wasm-objdump -x` against the produced `.wasm` artifact and asserts zero matches for the four forbidden crate prefixes (`loro`, `iroh`, `redb`, `wasmtime`). A regression that pulls any forbidden crate into the bundle fails CI immediately with the matched-symbol output. Pinned by `bindings/napi/tests/wasm_bundle_content.rs`.

The companion `wasm-checks.yml` `benten-sync-refuses-wasm32` cell additionally asserts that `cargo check --target wasm32-unknown-unknown -p benten-sync` fails with the `compile_error!` macro firing — covering the case where the source-side cfg-gating gets accidentally removed but feature-graph closure happens to still hold.

---

For the paths this will take next, see [`HOW-IT-WORKS.md`](HOW-IT-WORKS.md) "The path from here." For every error the engine surfaces, see [`ERROR-CATALOG.md`](ERROR-CATALOG.md).
