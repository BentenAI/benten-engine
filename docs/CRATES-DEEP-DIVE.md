# Crates Deep-Dive — Workspace-level Synthesis

A plain-English walk across the 10 workspace crates. This doc reads the per-crate `INTERNALS.md` deep-dives as primary sources and synthesizes patterns at the workspace level — what's working well, what's leaking, what's elegant, what could be more so, and what the open architectural questions are.

Audience: a fresh contributor or AI agent who has read `CLAUDE.md`'s "Architectural Decisions Baked In" items (#1–#19), `docs/VISION.md`, `docs/ARCHITECTURE.md`, and `docs/HOW-IT-WORKS.md`, and now wants the cross-crate picture before touching code.

Read this in order. § 1 paints the workspace; § 2 maps the dep graph; §§ 3–4 surface what's holding clean and what's leaking; § 5 names the big open architectural questions; § 6 consolidates concrete v1-gate candidates; § 7 catalogs "fine but could be more elegant" candidates; § 8 assesses Phase-3.5 / Phase-4 readiness; § 9 lists the open questions for Ben.

Source citations point at the per-crate `INTERNALS.md` documents under `crates/<crate>/INTERNALS.md` and the actual code paths they reference.

---

## § 1. Workspace at a glance

Ten crates. Roughly stacked from foundational types at the bottom to the orchestrator at the top, with two sibling layers (capabilities, IVM) feeding into the engine and one Phase-3 surface (sync) sitting alongside.

**`benten-errors`** — The single, frozen list of every error the engine is allowed to emit. 118 named variants today; the strings are the contract that TypeScript classes, operator dashboards, and audit pipelines route on. The crate is `no_std + alloc` with zero workspace deps, no `thiserror`, no `strum` — deliberately the irreducible nucleus that every other crate can depend on without pulling Benten machinery along. It earns its keep by being the engine's "what compositions are illegal" vocabulary; growing the catalog is how the engine teaches itself new ways to say no. (`crates/benten-errors/INTERNALS.md`.)

**`benten-core`** — The four shapes everything else agrees on: `Value` (DAG-CBOR-compatible value), `Node` (labelled, propertied, content-addressed graph node), `Edge` (content-addressed directed edge), `Cid` (CIDv1 newtype). Plus content hashing (BLAKE3 over canonical DAG-CBOR), two coexisting version-chain shapes, a Hybrid Logical Clock, and the `Subgraph` / `SubgraphBuilder` pair that gives a handler-as-graph its content-addressed identity. `no_std + alloc`, depends only on `benten-errors`. The hash contract is single-sourced here: every content-addressed type routes through the same encode path. The crate also enforces the arch-1 invariant — it does not, will not, and cannot depend on `benten-eval` (defended by a CI workflow + a unit test).

**`benten-graph`** — The storage layer. Only crate that talks to disk. Owns a narrow byte-level `KVBackend` trait (5 methods), a node/edge layer (`NodeStore` / `EdgeStore`), a closure-based `Transaction` primitive, MVCC snapshots, a `ChangeSubscriber` port that IVM hooks into, two on-disk indexes for label / property lookups, and the Inv-13 immutability machinery. Production backend is `RedbBackend` over redb v4 (native-only); `BrowserBackend` is the wasm32 thin-client cache; `SnapshotBlobBackend` and `NetworkFetchStubBackend` round out the trait family. The narrow byte-waist is genuinely respected; the policy that the engine layers above is partially baked into the storage layer (see § 4).

**`benten-ivm`** — Incremental View Maintenance. Subscribes to graph change events and maintains indexed materialized answers to questions the engine wants to ask cheaply later ("which grants apply to entity X?", "what's the head of version chain Y?", "next 20 posts by createdAt"). Five hand-written canonical views back invariants I3–I7; a generalized Algorithm B kernel handles arbitrary user-defined `(view_id, label_pattern, projection)` triples. Strictly downward dependencies: `benten-graph` and `benten-core` feed it; nothing reaches back. The engine names `Strategy` as the dispatch type but the algorithm internals stay opaque — this is the CLAUDE.md item #2 boundary, sharpened at Phase-3 R6-R3.

**`benten-caps`** — The capability-policy layer. Owns the `CapabilityPolicy` pre-write / pre-read hook trait the transaction primitive consults at commit time, plus `WriteContext` / `ReadContext` / `PendingOp` / `CapabilityGrant` / `GrantScope` / segment-wise attenuation. Ships three concrete policies: `NoAuthBackend` (zero-cost default), `GrantBackedPolicy` (Phase-2 grant-store), `UcanGroundedPolicy` (Phase-3 composes grant-backed with durable UCAN chain validation). The durable `UCANBackend` lives here too (native-only). The trait is principal-agnostic; UCAN-specific identity types stay over the `benten-id` line. Three independent trait surfaces stay separate: `CapabilityPolicy`, `GrantReader`, `RateLimitPolicy` — by design, not accident.

**`benten-eval`** — The 12 operation primitives and the iterative evaluator that walks them. Owns the `PrimitiveHost` trait (the single seam through which a primitive executor reaches storage / caps / IVM / etc), the wasmtime SANDBOX host end-to-end, the TRANSFORM expression language (parser + pure evaluator + 50+ built-ins), the runtime invariant checks (Inv-1 through Inv-14), the frozen WAIT suspension protocol (`ExecutionStateEnvelope` / `AttributionFrame` / `Frame`), and the typed-CALL closed dispatch registry. This is the crate where CLAUDE.md commitments #1 (12 primitives irreducible), #4 (not Turing-complete), and #16 (SANDBOX is the escape hatch) physically live. The arch-1 dep-break is enforced here too: `benten-eval` does not depend on `benten-graph`; storage failures cross as opaque `HostError` envelopes.

**`benten-id`** — Cryptographic identity primitives. Three rings: inner (Ed25519 keypair with `Zeroize` hygiene; `did:key` W3C method encode/decode; UCAN claim envelopes + chain-walk with nbf/exp/attenuation/audience binding); middle (Verifiable Credentials over DAG-CBOR + Ed25519; DID rotation attestations; signed device-DID capability envelopes); outer (the `Acceptor` runtime gate for device-DID attestations — freshness + nonce-store replay defense + parent-DID pin). Phase-3 G14-A1 / A2 + G16-D wave-6b body. The crate forbids depending on `benten-graph`, `benten-engine`, `benten-eval`, `benten-caps`, `benten-ivm`, `benten-sync`, `benten-dsl-compiler` — pinned by a unit test (`tests/dependency_edges.rs`).

**`benten-sync`** — The 10th workspace crate (Phase 3 addition). Runtime for the Atrium peer-mesh sync surface: iroh QUIC transport, Loro CRDT at Node-property granularity, Merkle Search Tree for subgraph diff, DID-based mutual-auth handshake, signed thin-client inclusion proofs. Native-only per CLAUDE.md baked-in #17, defended by a three-rung barrier (compile-error macro + cfg-gated Cargo deps + a CI cell). The dep direction is engine → sync; `benten-sync` may not depend on `benten-engine` or `benten-eval` (pinned by a test).

**`benten-dsl-compiler`** — A runt of a crate, deliberately. Takes a short DSL string of operation-primitive calls chained with `->` and emits a canonical `benten_core::Subgraph` that the engine knows how to load. Added in Phase-2b to give the devserver a Rust-side compiler that doesn't drag in `benten-eval` or `benten-graph`. Whole crate is ~895 LOC in a single `lib.rs`; depends only on `benten-core` + `benten-errors` + `thiserror`. The four-public-item discipline is intact; arch-N tests scan its Cargo.toml directly to forbid additional engine-shape deps.

**`benten-engine`** — The orchestrator at the top. Composes storage, evaluator, IVM, capability, identity, and sync into a single public API (`Engine`). Owns the `Engine` struct, the builder pipeline, the active-call metadata that lets the evaluator look up the actor + handler, the change-broadcast tap that bridges storage commits into IVM and ad-hoc subscribers, the WAIT / STREAM / SUBSCRIBE engine-side surfaces, the privileged system-zone writes, the Atrium peer-to-peer session handle, the typed-CALL dispatch into `benten-id` crypto, and the napi-facing public API. Every TypeScript caller's eventual ground truth is a method here. Largest crate: ~25.6k LOC across 40 files.

---

## § 2. The dependency graph

```
                              benten-engine        (orchestrator)
                             ╱  │  │  │  │  ╲
                            ╱   │  │  │  │   ╲
              benten-eval ──┘   │  │  │  │    └── benten-sync (native-only)
                  │   │         │  │  │  │           │
                  │   └── benten-caps              benten-id ── (native-only)
                  │           │   │                   │
                  │           │   └── benten-graph    │
                  │           │           │           │
                  │           │           │           │
              benten-ivm─────────benten-graph         │
                  │               │                   │
                  └────── benten-core ────────────────┘
                              │
                        benten-errors

   benten-dsl-compiler ── benten-core   (independent leaf; consumed by devserver,
                                         deliberately NOT by benten-engine)
```

**Bottom layer — foundational shapes.** `benten-errors` is the absolute root (zero deps). `benten-core` is the second floor (depends only on `benten-errors`). Both are `no_std + alloc`. Together they pin the contracts every other crate agrees on: which strings are valid error codes, what canonical bytes a Node hashes to.

**Middle layer — storage, IVM, capabilities, identity.** `benten-graph` adds redb + DAG-CBOR encoders on top of core. `benten-ivm` subscribes to `benten-graph`'s change stream. `benten-caps` reaches into `benten-graph` for the durable UCAN backend's KV store and into `benten-id` (native-only) for chain-walk validation. `benten-id` itself is a flat leaf within this layer — it cannot depend on the others (defended by a unit test that walks its own Cargo.toml).

**Upper-middle — evaluator.** `benten-eval` depends on `benten-core`, `benten-errors`, and `benten-caps`. It deliberately does NOT depend on `benten-graph` — the arch-1 dep-break. Storage failures cross the eval/graph seam only as opaque `HostError` envelopes. This is the load-bearing architectural commitment that keeps the evaluator "ignorant of storage" and lets `PrimitiveHost` be the only seam through which storage state reaches the evaluator. Enforced by the unit test `benten_core_no_eval_dep.rs` AND a dedicated CI workflow.

**Sync layer — sibling to engine.** `benten-sync` (Phase 3) depends on `benten-id`, `benten-core`, and `benten-errors`. It deliberately does NOT depend on `benten-engine` or `benten-eval` — the layering goes engine → sync, never the reverse. Native-only.

**Top — engine.** `benten-engine` depends on every other workspace crate except `benten-dsl-compiler` (which is also intentionally absent — the engine consumes a pre-built `SubgraphSpec` shape, not source). The engine is the only crate that holds the full composition; everything else is a building block.

**Independent leaf — DSL compiler.** `benten-dsl-compiler` depends only on `benten-core` + `benten-errors`. Its arch-N tests forbid dependencies on `benten-eval`, `benten-graph`, `benten-engine`. The compiler exists for the devserver, not for the engine runtime path.

The shape is intentional and defended in depth. Every workspace crate that could plausibly invert a dep carries a unit test that scans `Cargo.toml` directly to forbid the inversion. The arch-1 dep-break (`benten-core` ↛ `benten-eval`; `benten-eval` ↛ `benten-graph`) is the load-bearing pair; the arch-r1-10 / arch-r1-11 pins on `benten-id` and `benten-sync` extend the same discipline into Phase-3.

---

## § 3. Philosophy adherence — what's working well

Reading the per-crate audits together, several themes recur as load-bearing wins.

### Composition-over-extension

The clearest example is **typed-CALL** in `benten-eval`. Phase-3 needed Ed25519, DID, UCAN, and VC operations. The temptation was either to widen the SANDBOX host-fn surface (would violate CLAUDE.md #16) or to invent a 13th primitive (would violate #1). The chosen path threads typed-CALL ops through the existing CALL primitive with a reserved `engine:typed:` prefix and a closed `TypedCallOp` registry; the actual crypto runs in `benten-engine` (which can depend on `benten-id`) via `PrimitiveHost::dispatch_typed_call`. Zero new primitives, zero new SANDBOX host-fns, full Phase-3 crypto. This is the textbook example future "feature X needs a Y operation" requests should follow.

The same pattern shows up in **CRUD handlers**: `crud('post')` lowers to a `READ → WRITE → RESPOND` chain that the evaluator walks identically to any other handler. No special-case "CRUD path" in the evaluator. Closes Compromise #8.

**Version chains** (CLAUDE.md baked-in #8) live in `benten-core` as opt-in shapes — `AnchorNode` + `VersionNode` + `CURRENT` pointer composition consumed by handler subgraphs through plain READ + WRITE. Not a primitive; ephemeral data doesn't pay versioning cost.

**WAIT cross-process resume durability** (Compromise #10 closure) happened via generalization, not feature-addition. The per-primitive ad-hoc surfaces (process-local WAIT registry, per-module SUBSCRIBE trait, engine-side envelope cache) all collapsed behind the single `SuspensionStore` port in `benten-eval`. One trait, one engine wire-up, three suspension shapes survive process restart.

### Boundary discipline

**arch-1 dep-break** (`benten-eval` ↛ `benten-graph`) is defended in depth: `Cargo.toml` carries no entry; the CI workflow + four dedicated arch tests assert the absence; `HostError` is the opaque envelope so no graph type appears on the public surface.

**IVM thinness** (CLAUDE.md #2) is defended by the engine boundary naming exactly one type (`Strategy`) from `benten-ivm` and the eval boundary defining `ViewQuery` LOCALLY in `host.rs` so the evaluator does not import `benten_ivm::ViewQuery`. The `dispatch_for` router is `pub` but documented INTERNAL; the engine's `register_user_view` is the only call site that consumes it.

**SANDBOX host-fn surface** (CLAUDE.md #16) is closed at four (time + log + kv:read + random). The `HOST_FN_NAMES` constant is a `const &[&str; 4]` so adding a new name fails compilation in any code that depends on the length. There is no `kv:write`, no `kv:delete`, no edge-mutating host-fn, and a regression test (`tests/host_fn_no_storage_mutating_per_baked_in_16.rs`) defends against future drift.

**The 3-rung wasm32 defense** in `benten-sync` is exemplary: compile-error macro at `lib.rs` + cfg-gated Cargo dep tables + CI check cell. Future agent proposals to ship Loro / iroh in a wasm32 bundle run into all three rungs. (CLAUDE.md baked-in #17.)

**`benten-id`'s `arch-r1-10` test** walks its own `Cargo.toml` programmatically (not raw grep) and forbids depending on `benten-graph`, `benten-engine`, `benten-eval`, `benten-caps`, `benten-ivm`, `benten-sync`, `benten-dsl-compiler`. Identity is foundational; the test bakes the invariant into the build.

**`benten-sync`'s reverse pin** (`dependency_edges.rs`) forbids `benten-engine` + `benten-eval`. Same discipline, opposite direction. Engine → sync is the only allowed direction.

### Fail-closed defaults

`benten-errors` uses `#[non_exhaustive]` on the `ErrorCode` enum so catalog growth is always a minor-version bump and downstream `match` expressions are forced to include `_ =>`. Same discipline on `CapError` and `PendingOp` in `benten-caps`. Same on `EvalError` and `SandboxError` in `benten-eval`. Same on `GraphError` in `benten-graph`.

**`benten-eval`'s `routed_edge_label` is exhaustively named** (no `_ =>` wildcard) in `benten-errors`. A previous wildcard had silently misrouted `WriteConflict` to `ON_ERROR` when it should have been `ON_CONFLICT`; the EH2 fix replaced it with named family groupings. Future variants will fail to compile if they don't get an explicit family assignment.

**`UcanGroundedPolicy::DEFAULT_NOW_SECS = 0` sentinel + `chain_has_time_bounds`** in `benten-caps` is a fail-closed inversion: a chain with any time bounds against `now_secs == 0` aborts with `CapError::UcanClockNotInjected` rather than silently fail-open. Closes the G16-B-B-rest sub-item D.

**`Edge::None` vs empty-map properties are CID-distinct** in `benten-core`. No `skip_serializing_if` on `Edge::properties` — the CBOR encoder emits `null` for `None` and `a0` for `Some(empty)`, a stable 1-byte difference in the hash input.

**Hash-first verification on read** (`Node::load_verified` / `Subgraph::load_verified` / `Mst::apply_entries` / `RedbBackend::get_node` per W9-T6): bytes are BLAKE3-checked against the supplied CID BEFORE any decode attempt. Tampering surfaces as `E_INV_CONTENT_HASH`, never as a confusing decode error.

### Single source of truth

The **`Subgraph` content-addressed encoding** lives in `benten-core::subgraph.rs::canonical_subgraph_bytes`. `Subgraph` / `OperationNode` / `NodeHandle` / `PrimitiveKind` deliberately do NOT impl generic `serde::Serialize` / `Deserialize` (D5 — closed in cag-mr-g12c-cont-1). A caller who tries the generic-serde shortcut gets a compile error pointing at the canonical entry points. There is no second encoding lurking that could produce a different CID.

The **`CapRecheckFn` shared scaffold** in `benten-engine::cap_recheck.rs` is the single shared-signature surface that BOTH the G14-D SUBSCRIBE delivery-time gate AND the G15-A IVM materialization-time gate compose on. Extract first; no inline-then-refactor. A test pins the no-refactor contract.

The **`HandlerRoute` enum** in `benten-engine::handler_router.rs` lives in one place. EMIT and SUBSCRIBE both consume it; future producer/consumer drift is structurally impossible because the variant lives in one place.

`benten-id`'s **`capability_satisfies_requirement`** is the single subsume relation exposed for engine-side queries. The chain-walker uses it internally; the engine asks the same question through the same function. No parallel implementation in `benten-engine` that could drift.

### Test-shape pins

Every per-crate audit names extensive **architectural-shape pins** as load-bearing: the canonical Node CID literal `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` is committed and asserted across architectures; the `AttributionFrame` schema-fixture CID is pinned; the 12-variant `PrimitiveKind` shape is pinned; the four-host-fn-name set is pinned; the `[non_exhaustive]` discipline on every catalog enum is pinned; the dep-direction forbidden-crate sets are pinned. Together these turn architectural commitments into mechanical CI failures.

---

## § 4. Philosophy adherence — what's leaking + v1-gate candidates

The same audits surface frictions where the philosophy bends or where the code is fine today but flags risk as the platform grows. Grouped by theme.

### Engine-concern leakage into the storage layer

**`benten-graph::put_node_with_context`** is special-cased and load-bearing in three ways that don't compose from KVBackend primitives:

- The system-zone label gate (refuses user-path writes to `system:*`).
- The redb durability tier (`EnginePrivileged` forces `Immediate`, `SyncReplica` forces `None`, `User` honors configured).
- The Inv-13 5-row dispatch matrix (user-reput → `E_INV_IMMUTABILITY`; engine-privileged reput → silent dedup).

This is the engine's capability/invariant policy stretched into the storage layer. The right abstraction is a pre-write hook trait (`CapabilityPolicy`) at the engine layer; today the storage layer carries the policy directly. The TODO at `crates/benten-graph/src/lib.rs:797` ("phase-3 — write-authority/is_privileged coherence") flags that the two axes can drift.

**v1-gate prominence: medium.** The shape works today; the cleanup is hygiene. Phase 4 plugin manifests will pressure this surface harder because manifest-declared `requires` caps may not match the `store:<label>:write` shape that `GrantBackedPolicy` hard-codes today.

### Storage-layer indexes that should be user-defined views

**`benten-graph`'s two redb indexes** (`LABEL_INDEX_TABLE`, `PROP_INDEX_TABLE`) are hard-coded on `RedbBackend`. `get_by_label` and `get_by_property` are inherent methods. The browser thin-client does not maintain these indexes at all. From the application-layer-composition philosophy, these look like the kind of thing an application's IVM views could maintain declaratively on top of the change stream, not something the storage layer should bake in.

The audit acknowledges the tension honestly: Phase-1 baked them in because the engine's privileged write paths (capability grants, version chains) need O(1) label-keyed reads at boot time, and an IVM-view-based replacement would have to be live before any of those engine paths boot.

**v1-gate prominence: low-medium.** Not urgent — the shape works and the bootstrap concern is real. But it's exactly the kind of "thinness slip" the v1-assessment-window per CLAUDE.md baked-in #15 is meant to consider.

### Transport / CRDT abstraction baked in

**`benten-sync` imports iroh and Loro types directly.** `transport.rs` directly imports `iroh::Endpoint`, `iroh::EndpointAddr`, `iroh::SecretKey`, `iroh::endpoint::presets`. `crdt.rs` directly imports `loro::{LoroDoc, LoroList, LoroMap, LoroValue, ExportMode}`. There is no `trait Transport` or `trait Crdt` abstraction.

CLAUDE.md baked-in #19 explicitly contemplates engine-level extensions for alternate transports (Tor / Nostr-relay / shaped relay) and alternate persistence backends. A Phase-9+ extension wanting to swap iroh for a different QUIC implementation would need either (a) iroh-compatible types or (b) public-surface refactoring. The engine-facing wrappers (`Endpoint`, `Connection`, `LoroDoc`) ARE newtypes — so a future `trait Transport` could be introduced without breaking the engine-facing API — but the seam doesn't exist yet.

**v1-gate prominence: low.** The audit assesses this honestly: Loro and iroh are plausibly settled choices for the engine's life. The risk is asymmetric: cheap to introduce a `trait Transport` later if the engine API doesn't depend on it; expensive to introduce later if it does. The engine-facing surface today preserves that optionality cleanly.

### Signature-scheme hardcoded

**`benten-id` bakes Ed25519 throughout.** The DAG-CBOR seed envelope tags `alg: "Ed25519"` and the import path returns `UnknownAlg` for anything else. `MultiSigSurface` is the cleanest extension trait, but the Phase-3 default carries Ed25519 in its type signature (`type Signature = ed25519_dalek::Signature`). The CLAUDE.md #19 trust model (engine extensions are compile-time linked) makes a heavy-lift refactor acceptable when it lands, but the post-quantum / hardware-key future would touch `Keypair` / `Did` / UCAN public surfaces broadly.

**v1-gate prominence: low.** The Phase-3 commitment was a single algorithm; flagging this so the future isn't a surprise rewrite. The cag-5 + D-PHASE-3-24 commitment is to defer identity-recovery protocol choice to the v1-assessment-window, and a `MultiSigSurface` extension trait is the named seam.

### Test-helpers leaking into public API

**`benten-errors::code(&self) -> ErrorCode`** is an identity method that exists purely so test code like `let err = ErrorCode::Foo; err.code()` compiles. The doc comment is candid that it's a "Phase 2a dx-r1-add" — it's not catalog API, it's test-shim API.

**`benten-errors`'s `PartialEq<ErrorCode> for str` + `for &str`** (plus reverse) exists for Phase-2a test ergonomics (`assert_eq!(err.code(), "E_CAP_DENIED")`). The asymmetry without rationale is mildly confusing on first read.

**`benten-core::Cid::sample_for_test` / `sample_for_label`** is gated behind `cfg(any(test, feature = "testing"))`, which is the right discipline — but the `testing` feature is part of the crate's public surface.

**`SecretKey::bytes_for_test` + `secret_bytes_unprotected` in `benten-id`** are documented escape hatches. The first is `#[doc(hidden)]` and explicitly test-only; the second is `#[must_use]` production-named but the docstring warns the caller is responsible for `Zeroizing` if the value lives past dispatch. Two production use sites are named (typed-CALL dispatch + iroh keypair construction), each with phase-3-backlog destinations.

**v1-gate prominence: low.** Cosmetic across the workspace. The right cleanup is per-shim: either move to `#[cfg(test)]` impl, document as test-helper-only, or delete and require tests to use the value directly.

### Schema / scope derivation hardcoded

**`benten-caps::GrantBackedPolicy`** derives scope from label-only via `format!("store:{label}:write")`. Phase-1 CRUD zero-config maps every `crud('post')` write to `store:post:write`; any future namespace (e.g. `host:atrium:publish_view_result`, plugin manifests) must pre-populate `ctx.scope` at the engine layer. The bifurcation is a smell — there's no single source-of-truth that says "for primary-label X, the required scope is Y."

**Phase 4 plugin manifests** (CLAUDE.md baked-in #18) make this worse: a plugin's manifest declares a `requires` cap that may not match the `store:<label>:write` shape at all. Either the policy needs to learn manifest-aware scope derivation, or the manifest-time scope must be threaded through `WriteContext::scope` at the registration boundary.

**v1-gate prominence: high.** This is a real Phase-4 prerequisite. The audit names it explicitly as the v1-gate work.

### Stale comments / unresolved TODOs

Each per-crate audit surfaces a small handful of stale comments or unresolved TODOs:

- `benten-errors::parse_cap_string` doc-comment still says "Phase 2a stub" + "Real parser lands in G4-A" despite G4-A having closed.
- `benten-core::lib.rs:711` carries `TODO(phase-3 — anchorstore + GC)` on `U64_CHAINS` unbounded growth.
- `benten-core` has two coexisting `Anchor` shapes (`u64`-id and Cid-head-threaded) with `TODO(phase-3 — version surface consolidation)` markers; R5 G7 was supposed to pick a canonical shape and didn't.
- `benten-graph::lib.rs:797` carries `TODO(phase-3 — write-authority/is_privileged coherence)`.
- `benten-graph`'s in-transaction flag is per-`Arc<RedbBackend>` (mini-review g3-ce-7 proposed keying on canonical DB path; carried).
- `benten-graph::next_tx_id` is process-lifetime-only (mini-review g3-ce-8 proposed persisting; carried).
- `benten-ivm::lib.rs` has three Phase-3 TODOs: per-view Criterion benches, cascade create→delete (now closed), and rebuild-equivalence event-replay (still open).
- `benten-eval::HostError::to_wire_bytes` is a Phase-2a placeholder; the Phase-3 DAG-CBOR upgrade is a TODO.
- `benten-eval::HlcTimeSource` is a Phase-2a atomic-counter placeholder; the `uhlc::HLC` integration is named for Phase-3 sync work.
- `benten-sync` carries `MessageKind` duplicated across `handshake.rs` + `mst_proto.rs` with a "reconcile to single source of truth" comment.
- `benten-sync::crdt.rs` flags "compact-old-writes-on-checkpoint" as a future R6-FP optimization with no concrete trigger.

**v1-gate prominence: low to medium.** Most are quick hygiene wins; the version-surface consolidation and HLC wire-up are the most prominent ("medium").

### Monolithic file growth

Three crates carry files that are past comfortable readability:

- **`benten-errors::lib.rs`** at 1666 LOC. Natural for a catalog (one variant + 3 match arms + 1 doc-comment per entry), but a future split into one module per family (`cap.rs` / `sandbox.rs` / `sync.rs`) becomes attractive past ~300 variants. The friction is that `match` exhaustiveness is what makes the enum a tripwire — splitting variants across modules requires the arms to distribute, which is awkward in Rust.
- **`benten-eval::primitives::subscribe.rs`** at 1740 LOC and **`benten-eval::primitives::sandbox.rs`** at 1722 LOC are the largest single files in the workspace's second-largest crate. SUBSCRIBE folds D5 strengthening commitments (bounded retention + within-key strict ordering + exactly-once delivery + per-event cap-recheck). SANDBOX folds the wasmtime per-call lifecycle + the 13-variant `SandboxError` + the four enforcement axes + the live cap-recheck callback. Both have a coherent story; both will benefit from a follow-up split.
- **`benten-engine::engine.rs`** at 4471 LOC is past the comfortable readable threshold. The R6 Wave-2 split already moved diagnostics + caps + views + crud + modules + transaction + snapshot out; `engine.rs` retains the dispatch core + the registration paths + the apply-atrium-merge orchestrator + the resolved-alias `impl Engine` block. The most natural next split would peel `register_subgraph*` into its own `engine_register.rs`.
- **`benten-engine::engine_sync.rs`** at 1805 LOC is the second-largest. The split point would be `engine_sync_attestation.rs` (the V2 envelope + verify path) vs the `AtriumHandle` proper.

**v1-gate prominence: low.** Hygiene, not correctness. The v1-assessment-window per CLAUDE.md baked-in #15 names "small architectural cleanups" — these files are candidate.

### `*::AllProps` placeholder for Phase-4 materializer

**`benten-ivm::Projection`** has exactly one variant (`AllProps`). The kernel applies the identity transform on every matched Node. Real projections (`Computed`, `PropSubset`, `Reshape`) are deferred. The Phase-3.5 / Phase-4 materializer will force the issue: schema-driven rendering needs richer-than-identity projections to support joins across labels, edge-traversal-keyed views, computed fields, reshape to non-Node output.

This is the single most prominent "load-bearing-yet-placeholder" surface in the workspace. The kernel architecture is ready for richer projections; the enum is the gating shape.

**v1-gate prominence: high** for v1 (Phase 4) if the materializer composes USING IVM views (Sketch A in § 5). **Low** if the materializer is a sibling subscriber (Sketch B).

### Other named surface concerns

- **`benten-ivm::ViewQuery`** is one un-typed record carrying every field any view needs (`label`, `limit`, `offset`, `anchor_id`, `entity_cid`, `event_name`). Every view ignores most of them. Pattern mismatches surface as `E_IVM_PATTERN_MISMATCH`. A typed-per-view variant is named in the docstring but hasn't landed. **v1-gate prominence: medium.**
- **`benten-ivm`'s `Strategy::C` (Z-set / DBSP)** is reserved-not-implemented forever-deferred. Either commit to a phase or rename to `Strategy::Reserved`. **v1-gate prominence: low.**
- **`benten-ivm`'s Phase-1 `rebuild()` doesn't replay events.** Every view's `rebuild` clears state and resets the budget; none replay the change-event log. The audit names this as a real correctness gap. **v1-gate prominence: medium.**
- **`benten-sync` light-client `MerkleProof` is O(n) not O(log n).** Tree-shaped path is named for Phase 4. The bandwidth-saving still holds because payload bytes aren't in the proof. **v1-gate prominence: low.**
- **`benten-sync`'s handshake nonce-cache for replay-protection.** The bounded-window math catches captured-off-wire replays older than the window; it does NOT catch a replay within the window if the nonce hasn't been cached. **v1-gate prominence: medium.**
- **`benten-caps::GrantBackedPolicy::wildcard_variants` is O(2^N).** For an N-segment scope, up to 2^N candidate parent-scope spellings are queried. Bounded to N ≤ 6. **v1-gate prominence: low.**
- **`benten-dsl-compiler::validate_shapes` only knows about SANDBOX.** Structured for appending rules, currently covers one. **v1-gate prominence: low.**
- **`benten-engine`'s generic-cascade lift** — most dispatch methods live on the resolved-alias `impl Engine` block (which uses `RedbBackend`), not on the generic `impl<B: GraphBackend>`. Named in phase-3-backlog §1.2-followup; v1-assessment-window names "engine impl-block generic-cascade lift" as a pre-tag cleanup. **v1-gate prominence: medium.**

---

## § 5. The big architectural questions

Two architectural forks surface across the per-crate audits. Both are pre-Phase-4 surface-to-Ben questions.

### Where does the materializer pipeline live, and how does it relate to IVM views?

`benten-ivm`'s audit flags three sketches explicitly:

**Sketch A — Materializer is built on top of IVM views.** A schema (content-type definition) compiles into a `ViewDefinition` (or several). The materializer registers those as user views via `Algorithm::register` and walks the change stream into them. Reads from the materializer are reads from the IVM view. Pros: reuses every piece of `benten-ivm`. `GenericKernel`'s `(label_pattern, projection)` shape already covers a non-trivial slice. `BudgetTracker`'s stale-with-last-known-good is exactly the contract a render layer wants. Cons: `Projection::AllProps` is the only variant — real schema-driven projections (joins, edge-traversal, computed fields, reshape to non-Node output) all need `Projection` to be a much richer enum.

**Sketch B — Materializer is a sibling subscriber.** The materializer is its own `impl ChangeSubscriber`, parallel to the IVM `Subscriber`, holding its own state shapes that don't fit `ViewResult`'s `Cids / Current / Rules` enum. Pros: clean separation; materializer is free to be as expressive as Phase 4 needs without contorting the IVM trait. Cons: code duplication. Both subscribers re-implement panic isolation, budget tracking, fan-out, stale-with-last-known-good. The existing `Subscriber` could be lifted to a generic shape that both consume, but that's a non-trivial refactor.

**Sketch C — Materializer uses the engine evaluator + handler subgraphs.** CLAUDE.md #18 frames plugins as "subgraphs of the engine's own operation primitives." The materializer could similarly be a subgraph that the evaluator walks on every write — a TRANSFORM node consuming the changed Node, an EMIT node broadcasting rendered output. IVM views are then orthogonal: the materializer subgraph could read FROM an IVM view as one of its inputs, but it doesn't live in `benten-ivm`. Pros: maximally aligned with CLAUDE.md #3 (code-as-graph). New materializer logic ships as new subgraphs/handlers, not new Rust crates. Plugin trust model trivially extends. Cons: read latency might be higher (evaluator walk per read vs `BTreeMap::range` on a pre-materialized index).

The `benten-engine` audit's § 8 flags D-3.5-2 picking "(b) engine-side sub-module" (parallel sibling to `engine_views.rs`), composing `IvmViewReadGate` + `cap_recheck` + `UserViewSpec`. That positioning is closest to Sketch A's lighter cousin (materializer lives engine-side, consumes IVM-view-shaped registration through `UserViewSpec`), but the question of whether `Projection` lifts to a richer enum vs. whether the materializer holds its own state shapes is still open.

### Handler-call-graph cycle detection

`benten-engine`'s audit names this in its open questions: the evaluator runs over a DAG (Inv-2 forbids cycles + Inv-7 forbids self-loops). User-registered subgraphs walk the static `SubgraphSpec` at register time and the invariant battery rejects cycles. But **at materialization time + during `register_subgraph_replace` hot-reload, no cycle detection runs over the broader graph of handler-CID → handler-CID call edges** (the `call_handler` cross-handler dispatch).

Phase-3-backlog hasn't surfaced a finding here. The current shape is: a handler-call cycle would only fail at iteration-budget exhaustion (Inv-8 multiplicative budget). That's correct in the sense of "not infinite loops" but unhelpfully late as a diagnostic — the operator sees `E_INV_ITERATE_BUDGET` rather than "you have a cycle handler-A → handler-B → handler-A".

The right shape is likely a registration-time cross-handler dependency walker that runs when any handler is registered or replaced; it would extend the existing per-subgraph cycle detection into the handler-call graph. Phase 4 (admin UI v0) is where this becomes operator-visible.

---

## § 6. v1-engine-gap candidates — consolidated list

Every concrete v1-gate candidate surfaced by the 10 per-crate audits, tabulated.

| Crate | Item | v1-gate prominence | Sketch of fix or follow-up |
|---|---|---|---|
| `benten-errors` | `parse_cap_string` stub doc-comment stale ("Phase 2a stub", G4-A landed) | low | Audit whether real parser absorbed it; if it is dead code, delete; if it remains the parser, update comment |
| `benten-errors` | `parse_cap_string` returns `Err(CapDenied)` for empty/malformed — conflates parse-failure with denial | low | Mint `E_CAP_STRING_MALFORMED` variant |
| `benten-errors` | `PartialEq<ErrorCode> for str` (bare `str`) + sibling impls — test-shim leaking into public API | low | Audit which test sites depend on which direction; drop unused half |
| `benten-errors` | `lib.rs` at 1666 LOC, monotonic-growing | low | Defer; future split into per-family modules |
| `benten-errors` | No `serde` integration | low (medium for Phase 4) | Add `serde` feature-flag (off by default) when Phase 4 plugin manifests need wire-encoded ErrorCode |
| `benten-errors` | No `#[derive(Hash)]` | low | Add if a consumer needs `HashMap` keying |
| `benten-core` | Two coexisting Anchor shapes (`u64`-id vs Cid-head-threaded) | medium | Pick canonical shape per `TODO(phase-3 — version surface consolidation)` |
| `benten-core` | `U64_CHAINS` unbounded process-global table | medium | Build caller-owned `AnchorStore` handle |
| `benten-core` | `HlcTimeSource` is a Phase-2a placeholder atomic counter | medium | Wire `uhlc::HLC` for Phase-3 sync (the trait exists; only default impl remains) |
| `benten-graph` | `WriteContext::with_authority` + `is_privileged` coherence drift | medium | Tighten per `TODO(phase-3 — write-authority/is_privileged coherence)` |
| `benten-graph` | Per-Arc transaction flag (two distinct handles on same redb file don't coordinate) | low-medium | Key on canonical DB path via process-wide static (g3-ce-7) |
| `benten-graph` | `next_tx_id` is process-lifetime-only — restart resets counter | low-medium | Persist into a dedicated redb table (g3-ce-8) |
| `benten-graph` | Broken subscriber drifts invisibly | low | Add dead-letter counter + tracing dep |
| `benten-graph` | `Group` durability collapses to `Immediate` at redb mapping | low | Tracking only — redb upstream issue |
| `benten-graph` | Storage-layer label/property indexes that could be user-defined views | low-medium | Defer to materializer / IVM-view-driven replacement (see § 5) |
| `benten-graph` | `BlobError::CidMismatch.code()` returns `Unknown("E_MODULE_BYTES_CID_MISMATCH")` | low | Admit catalog enum variant |
| `benten-graph` | `Transaction::transaction` always rejects nested | low | Savepoints / partial rollback design — Phase 4+ |
| `benten-ivm` | `Projection` has one variant (`AllProps`) — placeholder for materializer | high | Lift enum: `PropSubset`, `Computed`, `Reshape` |
| `benten-ivm` | `ViewQuery` is one un-typed record over-broad | medium | Typed-per-view variant per docstring |
| `benten-ivm` | `dispatch_for` + `is_canonical_view_id` are `pub` but documented INTERNAL | low | Narrow to `pub(crate)` after re-export sweep |
| `benten-ivm` | Phase-1 `rebuild()` doesn't replay events | medium | Phase-3+ event-replay infrastructure |
| `benten-ivm` | `Strategy::C` (Z-set / DBSP) reserved-not-implemented forever-deferred | low | Commit to phase OR rename to `Strategy::Reserved` |
| `benten-ivm` | Subscriber pattern-based pre-filtering Phase-3 TODO unmoved | low (medium at Phase 4 scale) | Becomes load-bearing when N user views >> 5 |
| `benten-ivm` | `Subscriber::on_change` uses `eprintln!` for non-fatal errors | low | `tracing` migration |
| `benten-ivm` | `for_id_with_budget` for `content_listing` non-`"post"` label silently drops budget | low | Lift constructor to accept `(label, budget)` together |
| `benten-ivm` | Budget arithmetic per-view-local, not subscriber-wide | low (medium at Phase 4 scale) | Pattern-based pre-filtering router (above) covers most of this |
| `benten-caps` | `UcanGroundedPolicy` scope-string surface uses `cap:typed:*` prefix as routing key | medium | Thread audience DIDs end-to-end so chain-walker grounds arbitrary scopes (`phase-3-backlog §2.3 (i)`) |
| `benten-caps` | `DEFAULT_NOW_SECS = 0` sentinel + clock-injection discipline | medium | `WriteContext::now` threading (`phase-3-backlog §2.3 (i)`) |
| `benten-caps` | `GrantBackedPolicy` derives scope from `store:<label>:write` only | high | Manifest-aware scope derivation for Phase 4 plugin manifests |
| `benten-caps` | `GrantReader::has_unrevoked_grant_for_scope` is scope-string-keyed (§13.11 root cause) | medium | Add CID-keyed companion method |
| `benten-caps` | `wildcard_variants` is O(2^N) | low | Wildcard-aware `GrantReader` query (single call) |
| `benten-caps` | `LegacyUcanStubBackend` retention timeline | low | Retire when Phase-1 tests migrate to durable backend |
| `benten-caps` | `InMemoryRateLimitPolicy` is the only concrete plug | low | Distributed token-bucket for Phase 6+ marketplace traffic |
| `benten-caps` | `UCANBackend::iter_installed_proofs` silently skips decode failures | low | Decide disposition when forward-compat envelope shapes arrive (Phase 4+) |
| `benten-eval` | `HostError::to_wire_bytes` Phase-2a placeholder format | low | Phase-3 DAG-CBOR versioned envelope upgrade |
| `benten-eval` | `WAIT regular-walk path's `signal_derived_placeholder` principal binding | low | Phase-3 eval/engine `Outcome` unification (`lib.rs:208` TODO) |
| `benten-eval` | `TraceStep` boundary-variant + attribution-threading completion | low | Required-on-every-variant contract |
| `benten-eval` | `subscribe.rs` at 1740 LOC + `sandbox.rs` at 1722 LOC | low | Hygiene split; defer past v1 |
| `benten-eval` | `SubscribeError::CapabilityDenied` collapses into `SubscribeDeliveryFailed` ErrorCode | low | Future cap-denied-at-register vs at-delivery split |
| `benten-id` | Ed25519 hardcoded throughout; no alternate-signature-scheme extension point | low | Future `MultiSigSurface`-shaped extension when post-quantum / hardware-key lands |
| `benten-id` | `serde_json` listed in dev-deps but possibly unused | low | `cargo machete` audit |
| `benten-id` | `SeedImportError::InvalidSecret` reserved but unreachable today | low | Document reserved status for coverage-tool clarity |
| `benten-id` | `UcanClaims::aud` is `String` not typed `Did` | low | Could tighten without breaking wire format |
| `benten-id` | `validate_chain_no_time_check` ambiguous re. nbf handling | low | Rename to `_test_only` + gate behind cfg |
| `benten-id` | No wasm32-compat CI cell on the crate itself | low | Add defensive `cargo check --target wasm32-unknown-unknown -p benten-id` |
| `benten-id` | `secret_bytes_unprotected` named uses (typed-CALL + iroh) | low | `Value::SensitiveBytes` extension (`phase-3-backlog §2.5 (e)`) |
| `benten-sync` | `MessageKind` duplicated across `handshake.rs` + `mst_proto.rs` | low | Reconcile to single source of truth |
| `benten-sync` | `crdt.rs` append-only property root List has unbounded write history | low (medium for long-lived Atriums) | Compact-old-writes-on-checkpoint |
| `benten-sync` | `MerkleProof` is O(n) not O(log n) | low | Phase-4 tree-shaped Merkle path |
| `benten-sync` | Real packet-loss detector | low | Replace `simulate_packet_loss` with sliding-window detector |
| `benten-sync` | `bind_with_relay_url` canary returns typed error rather than instantiating real relay-mode | low | Retire canary entry-point once no consumer reaches it |
| `benten-sync` | Handshake nonce-cache for replay-protection within window | medium | Add per-peer nonce-cache layer |
| `benten-sync` | `iroh::EndpointId` ↔ `PeerId` byte-equivalence not unit-pinned | low | Add byte-equivalence test |
| `benten-sync` | No `trait Transport` / `trait Crdt` abstraction for Phase-9+ engine extensions | low | Introduce later; engine-facing surface today preserves optionality |
| `benten-dsl-compiler` | `validate_shapes` covers only SANDBOX integer-typed properties | low | Append rules as shape-validation pattern grows |
| `benten-dsl-compiler` | Cargo.toml description says "~200-300 LOC, 4 public items" — drift | low | Update description |
| `benten-dsl-compiler` | No fuzz / proptest coverage of parser | low-medium | Add property test ("parses ⇒ round-trips through canonical-bytes with stable CID") |
| `benten-dsl-compiler` | `branch(...)` / `iterate(...)` predicate/body captured as opaque text | low | Phase 4+ predicate-semantics work |
| `benten-engine` | `engine.rs` at 4471 LOC | low | Hygiene split: peel `register_subgraph*` to `engine_register.rs` |
| `benten-engine` | `engine_sync.rs` at 1805 LOC | low | Split `engine_sync_attestation.rs` from `AtriumHandle` proper |
| `benten-engine` | Generic-cascade lift — most methods on resolved-alias `impl Engine` block | medium | `phase-3-backlog §1.2-followup` + v1-assessment-window per CLAUDE.md #15 |
| `benten-engine` | Handler-call-graph cycle detection absent | medium | Cross-handler dependency walker at registration time |
| `benten-engine` | EMIT-Named subgraph dispatch asymmetry between engine-side and eval-side | low | Wave-paired closure named; might still have small engine-side gap |
| `benten-engine` | `read_only_snapshot` runtime flag vs type-level mode | low | Defer; current shape is correct, type-level would cascade |
| `benten-engine` | Subgraph-cache key shape (`handler_id`, `op`, `subgraph_cid`) for materializer | low | Wait until materializer R1 |

---

## § 7. Over-engineering / inelegance / could-be-elegant candidates

Distinct from § 4 "philosophy leaks" — these are surfaces where the code works but could be simpler, more graph-native, or more in-spirit.

### Workspace-wide

**`#[non_exhaustive]` discipline** is excellent at the catalog enums (`ErrorCode`, `CapError`, `EvalError`, `GraphError`, `PendingOp`). Worth verifying the same discipline reaches every `#[non_exhaustive]` candidate (e.g. `SandboxError`, `AtriumTransportError`, `LightClientError`, `MstError`) — most are; a few might still be inferred-exhaustive.

**Per-crate `#[cfg(test)]` mock-clock patterns** in `benten-core::hlc.rs` and `benten-eval::time_source.rs` carry per-test static `AtomicU64` + fn-pointer pairs because the trait/struct signatures take bare `fn() -> u64` rather than `impl Fn` or trait objects. The audit notes this was a structural fix for a real R6-R2 flake. It's also an elegance smell: per-test statics mean adding a test means adding a static. A future refactor could lift to `Arc<dyn Fn() -> u64 + Send + Sync>` if no other crate needs `no_std` compatibility on the surface.

### `benten-errors`

**Variant ordering is roughly chronological-by-introduction, not family-grouped.** Readability suffers — finding "all cap-related variants" requires scanning the whole 1666-line file. A pre-v1 cleanup could reshuffle into family groups; the variant names + strings are stable so the reshuffle is purely internal.

### `benten-core`

**`Subgraph` fields are `pub`** (`nodes`, `edges`, `handler_id`, `deterministic`). The G12-C-cont docstring acknowledges the trade-off (eval-side invariants module was reaching into the previous `pub(crate)` siblings and converting to accessors-everywhere would cascade across ~2000 LOC). Accessor methods exist alongside. Defensible; flagging because the dual-surface (fields AND accessors) is mildly redundant.

**`SubgraphBuilder` knows about Inv-14** by stamping `attribution: true` on every emitted `OperationNode` by default. The `ATTRIBUTION_PROPERTY_KEY` constant is core-side because the eval-side builder previously needed the string. A soft boundary leak — Inv-14 is an evaluator concern that benten-core's builder defaults a property for. Documented trade-off; could be either accepted as the canonical attribution surface or moved to an eval-side trait extension.

### `benten-graph`

**The `KVBackend::ScanResult` is shape-opaque** which is exactly right. Worth pointing at as a model: the field is crate-private; the only public accessors are `.len()`, `.is_empty()`, `.as_slice()`, `.iter()`, and `IntoIterator`; the `Deref<Target=[..]>` that the spike had is gone. This pattern (shape-opaque newtype with a small accessor set) generalizes well; future "consider exposing a Vec<...>" decisions should consider whether the shape-opaque variant is structurally cleaner.

### `benten-ivm`

**`Subscriber::view_count`, `view_ids`, `view_strategy(view_id)`, `view_is_stale(view_id)`** are introspection accessors. Together they're a small but real public surface. A future engine-side metrics aggregation (the `benten.ivm.*` metric namespace) might prefer one `ivm_metrics_snapshot()` returning a `BTreeMap<String, f64>` matching the engine's `metrics_snapshot` shape; that would be more graph-native (state as data, not as four accessor calls).

### `benten-caps`

**Dual durable-grant-store seam** (raw-KV `g14b:grant:<cid>` store vs Node-encoded `system:CapabilityGrant` store) is documented in the module doc and defended by reference. It's the right shape — they have different read-shapes and write-paths — but it's also exactly the kind of "looks duplicative without the rationale" surface that future contributors will want to unify. The module-doc paragraph is the defense; worth pointing at as a model of how to write a defense.

**`UCANBackend::iter_installed_proofs` silently skips decode failures.** Trade-off between forward-compat tolerance and audit visibility. A future shape with `iter_installed_proofs_with_diagnostics` exposing the skip count + reason could let operator dashboards surface unrecognized envelope versions without breaking the common case.

### `benten-eval`

**`Subscriber::on_change` uses `eprintln!` for non-fatal errors** (with `#[allow(clippy::print_stderr)]` markers). Tracing migration is named since Phase 1. Mostly cosmetic.

**The 4 `todo!()` stubs at `engine_wait.rs:1011-1026`** named in the brief are CLOSED at HEAD — those line numbers now host the implementations of `get_node_label_only`, `put_node`, and `read_node_as` (per the `benten-engine` audit's § 7). Worth verifying the older `INTERNALS.md` references aren't stale by the time Phase-3.5 R5 dispatches.

### `benten-sync`

**`Endpoint::bind_with_relay_url` canary-scope arm still returns `RelayUnreachable` for the well-formed-URL case.** The wave-6b production path moved to `peer_discovery::bind_atrium_peer`. Two-arm distinguishing test keeps both arms exercised. A v1-window cleanup could fully retire the canary entry-point if no downstream consumer reaches `bind_with_relay_url` directly.

**Random nonce derived from a fresh `Keypair`** rather than `rand` direct dep. Works, is documented, but is a roundabout entropy path. A reviewer auditing entropy-source provenance has to follow the chain through `benten-id::keypair::Keypair::generate`. Either accept the indirection (the docs do) or add a direct `rand` dep — the cost is a one-line dep table change.

### `benten-dsl-compiler`

**`pub use benten_core::PrimitiveKind`** couples the DSL surface to the core enum. A new core `PrimitiveKind` variant would still compile (parser dispatch has `_ =>` → `E_DSL_UNKNOWN_PRIMITIVE`; `id_for` fallback to `"op"` prefix), but the new variant gets a generic prefix silently. Not a bug — `#[non_exhaustive]` discipline is respected — but the fallback is silent and a future agent might not notice.

### `benten-engine`

**The `Engine = EngineGeneric<RedbBackend>` type alias** means the resolved-alias `impl Engine` block carries ~all of the dispatch core; the `impl<B: GraphBackend>` block carries only constructors + a few cross-module accessors. This is more accumulated debt than design choice; the cleanest cascade requires `GraphBackend` to surface `register_module_bytes` / `get_by_label` / `get_by_property` / closure-based `transaction(|tx| ...)` paths uniformly. The v1-assessment-window names "engine impl-block generic-cascade lift" as a pre-tag cleanup. The lift would also enable the wasm32 `EngineGeneric<BrowserBackend>` to share more code with native.

**Compromise #17, #18, #21 are all CLOSED at HEAD** via durable backing. The audit names this explicitly as a load-bearing example: compromises that turned into permanent shapes (none); compromises that closed via durable persistence + rehydration (three). Future "named compromise X" entries can point at these as the disposition pattern.

---

## § 8. Phase 3.5 + Phase 4 readiness assessment

The workspace is **broadly ready** for Phase 3.5 (materializer + schema-rendering + admin UI v0) with one architectural decision and a small number of per-crate prerequisites.

### Architectural decision before Phase 3.5 R5

**Materializer-vs-IVM relationship.** Per § 5, the three sketches (A/B/C) are real architectural forks. The `benten-engine` audit's § 8 notes that D-3.5-2 is currently positioned as "(b) engine-side sub-module, sibling to `engine_views.rs`" — that's closest to Sketch A's lighter cousin. But the question of whether `Projection` lifts to a richer enum (Sketch A) vs. the materializer holds its own state shapes (Sketch B) vs. the materializer is composed from subgraphs (Sketch C) is still open. This is a Ben-level question, not an orchestrator-level decision.

### Per-crate touchpoints that need to land BEFORE Phase 3.5 R5

These are concrete prerequisites the materializer / admin UI work would surface if dispatched naively:

- **`benten-ivm::Projection` enum lift** (if Sketch A is chosen). The current `AllProps` placeholder cannot support schema-driven joins, computed fields, or non-Node reshape. The kernel architecture is ready; the enum is the gating shape. The audit names this as the "single most prominent load-bearing-yet-placeholder surface in the workspace."
- **`benten-caps::GrantBackedPolicy` scope derivation** to admit non-CRUD scopes (manifest-declared `requires` caps that don't fit `store:<label>:write`). Threaded through `WriteContext::scope` at the registration boundary.
- **Class B β `read_node_as` semantics confirmed end-to-end** (CLAUDE.md baked-in #18). Shipped at PR #184 per the current CLAUDE.md status. The audit notes the 4 `todo!()` stubs cited in the brief are CLOSED at HEAD. Worth re-verifying as Phase 3.5 R5 dispatch enters.

### Per-crate gaps that Phase 3.5 will surface

These are gaps that won't block dispatch but will become tangible inside Phase 3.5 work:

- **`benten-ivm::ViewQuery` un-typedness.** Materializer-registered views may want per-view query types; the current un-typed `ViewQuery` is fragile when N views with different field expectations register.
- **`benten-ivm::Phase-1 rebuild()` doesn't replay events.** Admin UI v0 would observe this if a materialized view trips its budget on first install and rebuilds to empty rather than to the recovered state.
- **`benten-eval`'s SUBSCRIBE pattern-based pre-filtering router** (named Phase-3 TODO, unmoved). Becomes load-bearing at "Phase 4 materializer registers 50 schemas → 50 views" scale.
- **Storage-layer indexes** (`benten-graph::get_by_label` / `get_by_property`) as the materializer's read-path entry. Cross-label property queries are out of scope today; "everything tagged urgent" has no path without a second index.

### v1-gate items that should land DURING Phase 3.5 vs deferred to Phase 4 / v1-assessment-window

**Land during Phase 3.5:**

- Materializer architectural decision (sketches A/B/C).
- `benten-ivm::Projection` lift if Sketch A.
- `benten-caps` scope derivation for non-CRUD scopes.
- Handler-call-graph cycle detection (admin UI will surface this).

**Defer to Phase 4 (plugin manifests + admin UI v0 + extension architecture):**

- Per-plugin DID + UCAN delegation surfaces in `benten-id` and `benten-caps`.
- Manifest-aware scope derivation in `benten-caps`.
- Per-plugin private-namespace caps (probably an extension axis on `GrantBackedPolicy`).
- Plugin manifest schema in `benten-core` (signed manifest envelope + `requires` / `shares` halves).

**Defer to v1-assessment-window:**

- `benten-engine` impl-block generic-cascade lift.
- `benten-core` two-Anchor-shape consolidation.
- Identity-recovery protocol choice (the `MultiSigSurface` extension surface).
- wasmtime Component-Model re-evaluation.
- `missing_docs` sweep (most crates already compliant).
- Small architectural cleanups (monolithic file splits, stale comments).

**Defer post-v1:**

- `trait Transport` / `trait Crdt` abstractions in `benten-sync` (CLAUDE.md #19 engine extensions).
- Alternate signature schemes in `benten-id` (post-quantum / hardware-key).
- `benten-ivm::Strategy::C` Z-set / DBSP implementation (or rename to `Strategy::Reserved`).

---

## § 9. Open questions for Ben

Flat list of the most-impactful unresolved decisions across all 10 crates. Roughly priority-ordered for the pre-Phase-3.5-R5 + pre-Phase-4 windows.

1. **D-3.5-2 materializer location + composition shape.** Sketch A (extend `benten-ivm`'s `Projection` enum + register materializer views as user views) vs Sketch B (sibling subscriber, new crate or new module) vs Sketch C (materializers are subgraphs the evaluator walks). The audit positions the current plan as "(b) engine-side sub-module" but the underlying composition-vs-sibling-vs-subgraph question hasn't been codified. This is the single biggest pre-Phase-3.5 decision.

2. **D-3.5-3 schema-rendering compiler location.** JSON-Schema-flavored shape vs DSL-extension in `benten-dsl-compiler`. The DSL compiler is currently 895 LOC of single-purpose handler-DSL → Subgraph compiler. Extending it to a second pipeline (schema → renderable graph) roughly doubles its scope; alternatively a `benten-schema-compiler` sibling crate keeps each compiler narrow. Which better fits the "code-as-graph" philosophy (CLAUDE.md #3)?

3. **`benten-graph` storage-layer label/property indexes as user-defined views.** Phase-1 baked them in for boot-time engine paths (capability grants, version chains). Moving them out requires the materializer / IVM-view-driven replacement to be live before engine boot. Is the storage-layer thinness slip worth resolving for v1, or is "engine-boot indexes are special" the right durable shape?

4. **`benten-sync` `trait Transport` / `trait Crdt` abstractions for CLAUDE.md #19 engine extensions.** Introduce now (low cost; preserves Phase-9+ flexibility) or defer (Loro + iroh are plausibly settled for the engine's life)? The engine-facing surface today preserves optionality cleanly — the abstraction can land later without breaking changes — but the post-Phase-3-close cleanup window may be the last cheap opportunity.

5. **`benten-id` `MultiSigSurface` extension surface for post-quantum / hardware-key futures.** Ed25519 is hardcoded throughout. CLAUDE.md #19 trust model (compile-time-linked Rust crates) makes a heavy-lift refactor acceptable when it lands. Should the trait extension point land for v1 (the cag-5 + D-PHASE-3-24 commitment is to defer the concrete protocol; the trait could ship without choosing one) or defer entirely until a concrete need lands?

6. **Handler-call-graph cycle detection at registration time.** Currently the only cycle defense is Inv-8 multiplicative budget exhaustion at runtime. Admin UI v0 will surface this. Add a registration-time cross-handler dependency walker in `benten-engine` for v1, or accept that cycles surface only as `E_INV_ITERATE_BUDGET` until Phase 4?

7. **`benten-ivm::ViewQuery` typed-per-view variant** (named in docstring, not landed despite views being stable since Phase 1). Phase 4 materializer-registered views will benefit. Land in Phase 3.5 alongside the `Projection` lift, or defer to a Phase 4 design pass?

8. **`benten-engine` impl-block generic-cascade lift.** Named in `phase-3-backlog §1.2-followup` and in the v1-assessment-window per CLAUDE.md #15. The wasm32 `EngineGeneric<BrowserBackend>` would benefit from sharing more code. Mid-Phase-3.5 (when surfaces are still in flux) vs. pre-v1-tag (when surfaces are frozen)?

9. **`benten-core` two-Anchor-shape consolidation.** `u64`-id and Cid-head-threaded both ship; R5 G7 was supposed to pick a canonical shape and didn't. Phase 4 anchor-store-with-GC work would force the issue. Resolve in Phase 3.5 or in Phase 4 alongside the anchor-store landing?

10. **`benten-ivm::Strategy::C` disposition.** Reserved-not-implemented forever-deferred. Either commit to a phase (Z-set / DBSP is a real algorithmic family) or rename to `Strategy::Reserved` with a string payload so the variant doesn't accumulate semantic weight without a plan.

---

End of synthesis.
