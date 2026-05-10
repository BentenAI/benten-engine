# Phase 3 Backlog

**Status:** Consolidated list of items deferred from Phase 2 (a + b) that have a clear Phase 3 landing point. Sibling to [`phase-2-backlog.md`](./phase-2-backlog.md) (Phase-1-deferrals-targeting-Phase-2). Every item here either (a) was explicitly scoped out of Phase 2a/2b with a Phase-3 trigger, or (b) was triaged into Phase 3 during Phase 2b close because the work depends on a Phase-3 surface that doesn't exist yet.

**Phase 3 scope anchor:** [`docs/FULL-ROADMAP.md`](../FULL-ROADMAP.md) §"Phase 3: P2P Sync — Atriums Ship Here." Phase 3 brings: peer-to-peer Atrium connections via iroh, CRDT merges via Loro, identity via Ed25519 / DID / VC, the durable UCAN backend in `benten-id`. Phase 3 also closes Compromises #9-#10 (already structurally closed in 2b R5 G12-E + 8c-subscribe), #N+8 / #N+9 (browser persistence + cross-browser determinism), and several engine-internal asymmetries the 2b R5 close surfaced.

**v1 milestone framing:** Phase 3 close is the natural pause-and-assess point for "Benten Engine v1." See [CLAUDE.md §15 + memory](../../CLAUDE.md) — at Phase 3 close we evaluate what (if anything) of Phases 4-8 needs to fold into the v1 boundary before external positioning. v1 shippability is NOT pre-decided; the assessment trigger is Phase-3-close.

---

## 1. Storage / backend

### 1.1 PHASE-3-BUNDLE-1 — Engine genericism over GraphBackend → drop redb from wasm32-unknown-unknown bundle

**Phase 2b state:** `crates/benten-engine/src/engine.rs:472` declares `pub(crate) backend: Arc<RedbBackend>` — `Engine`'s backend field type is unconditionally `RedbBackend` on every target (native + wasm32-wasip1 + wasm32-unknown-unknown). `crates/benten-engine/src/engine_snapshot.rs:19` is explicit: "The Engine is hard-bound to `benten_graph::RedbBackend` in 2b." This forces redb-4.1.0 into the wasm32-unknown-unknown browser bundle even though the napi `wasm_browser.rs` design intent says "in lieu of a redb backend" — bundle is ~150-200KB gzipped of redb (24 distinct `redb-4.1.0/src/...` debug paths visible in the wasm string table).

**Phase 2b interim** (wave-8j-ci-cleanup, PR #59): added `[profile.release-wasm]` (opt-z + lto=fat + panic=abort) + `wasm-opt -Oz` post-process step to `wasm-browser.yml`, bumped the wasm-r1-7 cap from 500KB → 600KB. Empirical 2b ceiling with these knobs: 499,195 bytes gzipped — 805 bytes under the original 500KB but with no realistic headroom. Bumping to 600KB gives ~115KB headroom that absorbs realistic 2b residuals + napi-rs evolutions.

**Phase 3 target:** Make `Engine` generic over a new `GraphBackend` umbrella trait that captures `KVBackend + NodeStore + EdgeStore + transaction() + register_subscriber() + snapshot() + put_node_with_context()`. Ship `BrowserBackend` (in-RAM `BTreeMap` keyed graph store; no B-tree page store, no transaction tracker, no savepoints) for `wasm32-unknown-unknown`. Re-tighten the `wasm-r1-7` cap from 600KB to ~350KB (the original spirit of the cap — "engine + DSL + snapshot-blob backend, NO SANDBOX, NO wasmtime, NO redb").

**Why deferred:** The "trait already in tree" framing (per HANDOFF-2026-04-29-morning §4 row 1) understates the work by ~3-4×. Real scope is **800-1,500 LOC production + 200-400 LOC test, 18-30 files, ~20-40 implementer hours, 2-3 sessions, sub-track-splittable.** The `KVBackend` trait covers only ~5 of ~15 backend methods Engine actually consumes; `Transaction` is built directly on `redb::WriteTransaction` (not a trait); `InMemoryBackend` explicitly disclaims `NodeStore`/`EdgeStore`/transactions/subscribers and is **not** the BrowserBackend in disguise. The umbrella-trait extraction + fresh BrowserBackend impl + Engine generics cascade is the realistic shape.

**Why Phase 3 is the natural home:**
- `engine_snapshot.rs:31` already calls out "Phase 3 replaces the tempdir hydration with a direct `SnapshotBlobBackend` wired in *once the Engine is generic over its backend*." Phase 3's `SnapshotBlobBackend` direct-wire NEEDS this work; bundling them re-uses the same umbrella-trait design pass.
- Phase 3's CRDT / iroh layer interacts with the backend through `KVBackend` already; pulling in genericism alongside introduces fewer integration surfaces than doing genericism in 2b and CRDT in 3.

**Trigger for earlier landing:** If the 600KB cap is breached mid-Phase-2b-close (no big-feature wave-9 work is planned per the bisect, so probability is low). Otherwise lands as part of Phase 3's first wave alongside `SnapshotBlobBackend` direct-wire.

**Full scoping plan:** `.addl/phase-2b/wave-8j-backend-genericism-scoping-plan.md` (gitignored) — includes surface inventory (~17 Class-A + ~30 Class-B sites across 8 files), trait surface review, public API impact (additive via type-alias `Engine = EngineGeneric<RedbBackend>` sugar), `BrowserBackend` impl sketch (~700-1000 LOC fresh code), wasm32 wiring strategy (Cargo feature `browser-backend`), risk surface, and a Track 1/2/3 dispatch brief skeleton.

**Bisect detail:** `.addl/phase-2b/wave-8j-wasm-browser-bundle-bisect.md` (gitignored).

**Touch size estimate:** **~800-1,500 LOC production + 200-400 LOC test. 18-30 files. 20-40 implementer hours, multi-session. Risk surface: medium** — public-API additive via type-alias sugar; cross-process determinism unaffected (CIDs computed over canonical Node bytes, not backend state); 558+ existing tests largely compile unchanged.

**Generics-vs-dyn design call (R6-R3 r6-r3-arch-6).** `KVBackend::Error` is an associated type (`crates/benten-graph/src/backend.rs:292-298`) → `Arc<dyn KVBackend>` does NOT work without a `Box<dyn StdError>`-erasure wrapper at the trait boundary. A Phase-3 implementer attempting the dyn-route will hit the Error-type wall and need to convert the trait — a substantive design decision. The recommended shape is `Engine<B: GraphBackend>` generic-cascade (16+ impl blocks in `benten-engine` + napi-binding generic-erasure at the cdylib boundary): preserves the typed-error surface end-to-end, costs the impl-block cascade. The alternative shape adds associated-error-erasure to `KVBackend` itself (e.g. via `type Error = Box<dyn StdError + Send + Sync + 'static>`): smaller blast radius at the impl-block level but compresses backend-specific error context into a string at the trait boundary, breaking the `EngineError::Graph(GraphError)` typed pass-through that currently flows from `RedbBackend` → `EngineError`. The Phase-3 plan MUST close this design call BEFORE implementation begins; orchestrator-side context for that decision is captured at `.addl/phase-2b/wave-8j-backend-genericism-scoping-plan.md` (gitignored). Currently Engine uses `Arc<dyn ...>` only for 4 trait surfaces (`SuspensionStore`, `MonotonicSource`, `TimeSource`, `CapabilityPolicy` via `Box<dyn>`) — none of which carry an associated error type.

### 1.2 SnapshotBlobBackend direct-wire (engine_snapshot tempdir hydration cleanup)

**Phase 2b state:** `crates/benten-engine/src/engine_snapshot.rs:223` opened a `RedbBackend` against a tempdir to hydrate snapshot bytes — works but an architectural smell ("tempdir-as-backend").

**Phase 3 G13-D wave-3 (PARTIAL CLOSE):** `SnapshotBlobBackend` now satisfies the full `GraphBackend` umbrella trait (`NodeStore` + `EdgeStore` + the snapshot/transaction/subscriber/put-with-context surface) in `crates/benten-graph/src/backends/snapshot_blob.rs`. `Engine::from_snapshot_blob` no longer creates a tempdir — hydration goes into `RedbBackend::open_in_memory()` so the function never touches the filesystem. Pinned at `crates/benten-graph/tests/snapshot_blob_backend.rs::snapshot_blob_backend_impls_graph_backend_read_path` + `snapshot_blob_backend_write_path_returns_read_only_error` + `crates/benten-engine/tests/snapshot_no_tempdir.rs::from_snapshot_blob_no_tempdir_in_path`.

**Why partial:** The full direct-wire to `EngineGeneric<SnapshotBlobBackend>` (no in-memory redb hop) requires lifting every `impl Engine` method (≈ 10 modules: `engine_crud.rs`, `engine_modules.rs`, `engine_subscribe.rs`, `engine_views.rs`, `engine_caps.rs`, `engine_sandbox.rs`, `engine_stream.rs`, `engine_wait.rs`, `engine_diagnostics.rs`, `primitive_host.rs::PrimitiveHost`) into `impl<B: GraphBackend> EngineGeneric<B>` form. That structural lift is out of G13-D's scope-real-15 budget (~100-200 LOC) and lands as §1.2-followup below per HARD RULE BELONGS-NAMED-NOW.

**Touch size landed at G13-D:** ~140 LOC graph-side (NodeStore + EdgeStore + GraphBackend impls + unit-marker types) + ~10 LOC engine-side (tempdir → open_in_memory swap + registry retire) + ~155 LOC test-side (3 must-pass pins).

### 1.2-followup EngineGeneric method-cascade lift for SnapshotBlobBackend (G13-D BELONGS-NAMED-NOW carry)

**Source:** Discovered during G13-D wave-3 implementation (R5 wave-3, 2026-05-05). Tracks the engine-side method generic-cascade lift that G13-D's scope-real-15 budget could not deliver.

**Phase 3 G13-D state (post-wave-3):** `Engine::from_snapshot_blob` returns `Engine = EngineGeneric<RedbBackend>` over an in-memory redb backend. The snapshot blob is consumed via `RedbBackend::open_in_memory()` + `put_node` replay rather than via direct `EngineGeneric<SnapshotBlobBackend>` construction. The type-level read-only contract is therefore enforced at the user-facing engine surface (`engine_crud.rs`, `primitive_host.rs::check_not_read_only_snapshot`) per the existing `is_read_only_snapshot()` flag, not at the backend's typed write surface.

**Phase 3 follow-up target:** `Engine::from_snapshot_blob(bytes) -> Result<EngineGeneric<SnapshotBlobBackend>, EngineError>`. The snapshot-blob bytes drive a `SnapshotBlobBackend` directly with no redb hop. Achieves:
- Truly zero filesystem touch (already true at G13-D) AND zero in-memory redb allocator pressure.
- Type-level rejection of writes (the backend's `put_node_with_context` surfaces `BackendReadOnly`), so the engine-side `is_read_only_snapshot` flag becomes redundant for snapshot-blob-typed engines.
- Full direct-wire matching the G13 PHASE-3-BUNDLE-1 architectural intent.

**Required structural lift:**
- `impl Engine` blocks in 10 modules → `impl<B: GraphBackend> EngineGeneric<B>` (engine_crud, engine_modules, engine_subscribe, engine_views, engine_caps, engine_sandbox, engine_stream, engine_wait, engine_diagnostics, primitive_host).
- Each module's body uses the redb-specific closure-based `self.backend.transaction(|tx| ...)` execution surface — those call sites need to migrate to the umbrella `GraphBackend::transaction()` handle (currently a unit marker; the lift may evolve the handle into a borrowing runner per `arch-r1-6` recommendation).
- `EngineBuilder` becomes generic over `B: GraphBackend` (currently hard-bound to `RedbBackend`); separate `from_redb` / `from_snapshot_blob` / `from_browser` constructors handle the per-backend distinct construction shapes.
- Existing tests in `crates/benten-engine/tests/integration/snapshot_blob_round_trip.rs` (4 LIVE tests, including `snapshot_blob_rejects_delete_via_dispatch_handler` that exercises `dst.call(...)` against the snapshot-blob engine) must continue passing — the lift is a refactor, not a behavior change.

**Why deferred from G13-D:** Scope-real-15 sized G13-D at ~100-200 LOC total (re-sized from ~50-100 to add test coverage). The structural lift is a multi-thousand-LOC + 10-module touch with high cross-wave-blocking risk. Better landed as a dedicated wave with its own R1 + mini-review.

**Touch size estimate:** ~1,500-3,000 LOC engine-side method lift + ~300-600 LOC builder genericism + ~200-400 LOC test updates (cascade through call sites that name `Engine`/`EngineGeneric<RedbBackend>`).

**Suggested wave:** new G13-F or fold into G14-pre/G15-pre depending on which wave-4 work needs `EngineGeneric<B>` cascade for non-redb backends first. The G14-B durable UCAN-backend work also threads `<B: GraphBackend>` so coordination there is natural.

**Cross-ref:** Phase-3 plan §3 G13-D row scope-real-15 line. CLAUDE.md baked-in §17 deployment-shapes commitment (the in-memory redb hop is a temporary convenience for the full-peer path; thin compute surfaces will need this lift to avoid carrying any redb at all on `wasm32-unknown-unknown`).

**RATIFIED 2026-05-05 by Ben (post-W3 surface batch):** alias-based pragmatic-genericism (Engine = EngineGeneric<RedbBackend> + cfg-gated alias arms for browser-backend) is sufficient for Phase-3 close. The full impl-block cascade lift (~1500-3000 LOC) is **deferred to v1-assessment-window** (post-Phase-3-close per CLAUDE.md §15 + memory `feedback_v1_milestone_gate`). Rationale: every Phase-3 actual consumer (BrowserBackend in-RAM thin-client cache per #17, SnapshotBlobBackend read-only via in-memory-redb hop, SyncReplica native-only per #17 baked-in) is satisfied by the alias-based shape — no real Phase-3 consumer needs the full method-cascade. The cascade only matters when alternate durable backends (DynamoDB, PG, S3-stored mmap) materialize as Phase 9+ exploratory destinations. **Standing rule for R5 wave-4+ implementer briefs (G14 / G15 / G18):** preserve the alias-based shape; apply same DISAGREE-WITH-EXPLANATION + BELONGS-NAMED-NOW disposition pointing here when the brief literally calls for impl-block cascade of `<B: GraphBackend>` over redb-specific methods.

### 1.3 Arc<dyn KVBackend> migration (in-memory backend pivot from wave-5)

**Phase 2b state:** Wave-5 (G10-A wasip1) produced an `InMemoryBackend` impl of `KVBackend` (orchestrator-direct PR #38), but `Engine` is hard-bound to `Arc<RedbBackend>` so the in-memory impl is unused in production. HANDOFF-2026-04-29-morning §4 row 1: "Document tech debt, defer Arc<dyn KVBackend> refactor to Phase 3."

**Phase 3 target:** Item 1.1 is the actual structural fix; this is the same item by another name. Folded into PHASE-3-BUNDLE-1.

### 1.4 Compromise #17 durable module-bytes registry — CLOSED at Phase-3 G14-C wave-4b

**Closure narrative:** durable `RedbBlobBackend` + CID-validating entry point landed at G14-C wave-4b (PR #110 commit `6003ed0`). `Engine::register_module_bytes` at `crates/benten-engine/src/engine.rs:1612` now BLAKE3-mismatch-rejects; durable backing at `crates/benten-graph/src/backends/blob_backend.rs:94` (`RedbBlobBackend::new`); engine open path rehydrates active set via the `system:ModuleManifest` zone + persisted blobs. SECURITY-POSTURE.md retensed accordingly.

**Phase 2b state (historical):** Wave-8 shipped an in-memory module-bytes registry (`crates/benten-engine/src/engine.rs::register_module_bytes` plus the in-memory active set in `engine_modules.rs::install_module`). Two structural compromises rolled up under Compromise #17 in `docs/SECURITY-POSTURE.md`:
- (a) **Non-validating registration API** — `register_module_bytes` does NOT verify the supplied CID matches `blake3(bytes)`. Validation fires lazily at SANDBOX dispatch when wasmtime parses bytes (`Module::new(&engine, &bytes)` → `E_SANDBOX_MODULE_INVALID`). Wave-8j-cleanup added a 3-LOC `debug_assert` for dev-build fail-fast, but release builds still trust the caller-supplied CID.
- (b) **Process-local lifetime** — registered bytes are not durable across `Engine::open` cycles. The `system:ModuleManifest` zone persists module *manifests* but the actual wasm bytes blob is in-memory only; a process restart drops them.

**Phase 3 target:**
- (a) **CID verification at registration** — durable BlobBackend that computes BLAKE3 + persists bytes by CID. The `register_module_bytes` API can either (i) become CID-validating end-to-end (BLAKE3-on-input + reject mismatch) or (ii) remain caller-supplied-CID with explicit "CID-as-attribution" semantics — Phase 3 picks per UCAN integration design.
- (b) **Durable module-bytes blob store** — Phase 3's `BlobBackend` (likely in `benten-id` or `benten-graph`) keys bytes by CID + persists across restart. `Engine::open` rehydrates the active set from the persisted manifest zone + blob backend.

**Why Phase 3:** The durable BlobBackend requires the GraphBackend umbrella trait (PHASE-3-BUNDLE-1, §1.1) so that `Engine<B>` can carry a generic `Arc<B>` to native (RedbBackend + on-disk blob) vs. browser (BrowserBackend + IndexedDB blob, see §4.1) contexts. Earlier landing isn't possible without §1.1.

**Touch size:** ~150-300 LOC. Runs alongside §4.1 IndexedDB persistence on the browser side.

### 1.5 Compromise #18 durable handler-version chain — CLOSED at Phase-3 G14-C wave-4b

**Closure narrative:** durable `system:HandlerVersion` zone + extensible canonical-bytes encoding (per arch-r1-4 / D-C) landed at G14-C wave-4b (PR #110 commit `6003ed0`). The version chain is now graph-encoded as Anchor + Version Nodes in the system zone (loaded at `crates/benten-engine/src/engine.rs:1448` engine-open hydration); the in-memory `BTreeMap` at `engine.rs:351` is a CACHE, durable backing is the zone Nodes. SECURITY-POSTURE.md retensed accordingly.

**Phase 2b state (historical):** Wave-8f's `register_subgraph_replace` built a handler-version chain in memory: an in-RAM `HashMap<handler_id, VersionChain>` where each chain held (anchor_cid, current_version_cid, predecessor_cid, chain_depth). Per-PR audit-class differentiation from #17 (in-memory module bytes) — distinct concerns: #17 is content-bytes; #18 is graph-encoded version metadata. Documented in `docs/SECURITY-POSTURE.md` Compromise #18.

**Phase 3 target:** Lift to the canonical Phase-1-shipped `core::version::Anchor` + Version-Node-chain pattern, persisted via the new GraphBackend umbrella trait. The chain becomes a real graph subtree (Anchor Node + Version Nodes + CURRENT pointer) rather than a side-table HashMap.

**Why Phase 3:** Same dependency as §1.4 — needs the umbrella trait (PHASE-3-BUNDLE-1). The version chain itself IS graph-encoded by design; lifting it to durable backing is mechanical once Engine is backend-generic.

**Touch size:** ~100-200 LOC engine-side + whatever the graph schema for the chain requires. Can land in the same wave as §1.1 + §1.4.

---

### 1.6 `get_node_verified` read-path hash check — CLOSED at Phase-3 W9-T6 PR #142 (closure-shape-divergence)

**Closure narrative:** Phase-1 R7 spec-to-code-compliance audit named the gap as "Phase 2 deliverable." Phase-1 retrospective claimed this was deferred to Phase 2 and never landed; **2026-05-08 cross-phase orphan-rescue audit incorrectly carried it forward** based on that retrospective claim. Verification at HEAD `7a6c36a`: W9-T6 PR #142 (commit `92bd65e`) PROMOTED base `RedbBackend::get_node` to verify-on-read at `crates/benten-graph/src/redb_backend.rs:690-705` (calls `Node::load_verified`); test pin at `crates/benten-graph/tests/get_node_verifies_content_hash_on_read.rs`.

**Closure shape diverged from Phase-1 framing:** Phase-1 framing was "add an optional `_verified` variant API; default `get_node` stays unverified-for-speed." Actual closure shape: base `get_node` was promoted to verify-on-read (no separate `_verified` variant). Defense-in-depth applies uniformly at every read; no trust-boundary callsite migration required because every read is now verified.

**Cross-reference:** `docs/history/PHASE-1.md §4.2` (original Phase-1 deferral); 2026-05-08 cross-phase retrospective audit memo (orphan-rescue trigger; the audit was based on a stale Phase-1 retrospective claim that the work never landed — verification at HEAD found W9-T6 had already closed it).

---

## 2. Capability / identity

### 2.1 Durable UCAN backend in `benten-id` — CLOSED at Phase-3 G14-B wave-4b (PR #109 commit `496e144`)

**Phase 2b state:** `benten-caps::UCANBackend` is a `CapError::NotImplemented` stub. Phase 1's `phase-2-backlog.md` §7 already names this. Phase 2b's wave-8c-subscribe-infra adds the SUBSCRIBE delivery-time cap-recheck closure that hooks into the (in-memory) grant store — Phase 3 lifts the grant store to durable backing.

**Phase 3 target:** Full UCAN chain validation + delegation; durable grant store backed by the new graph backend (whichever PHASE-3-BUNDLE-1 produces); `benten-id` crate ships Ed25519 / DID / VC alongside.

**Phase 3 R5 wave-4b status (G14-B LANDED):** `crates/benten-caps/src/backends/ucan.rs::UCANBackend<B: GraphBackend>` ships the durable backend at wave-4b. Composes `benten_id::ucan::validate_chain_at` (in-memory chain-walk + signature + nbf/exp at every link) with a content-CID-keyed durable revocation lookup. Mini-review fix-pass adds `validate_chain_for_audience_at` pinning CLR-2 audience-binding at the durable seam + typed `CapError::UcanAudienceMismatch` for cross-atrium replay routing.

**ssi re-evaluation pointer (G16 Atrium-handshake):** the durable backend uses the hand-rolled internal `benten_id::ucan::Ucan` format. Adding `ssi` as a dep for external Benten producer-interop is deferred to G16 — the Atrium-handshake wave names it as its forward-compat axis if external producer-interop becomes required. No external Benten producers exist at G14-B (closed-loop). Per HARD RULE rule-12 clause-b: this section is the named destination for the ssi BELONGS-ELSEWHERE deferral; the destination receives the entry NOW (this paragraph). G16 then either (a) lights up ssi at the Atrium-handshake codec boundary, or (b) re-defers with a fresh named destination + reason.

**Source:** [`phase-2-backlog.md`](./phase-2-backlog.md) §7.1, §7.2, §7.3, §7.4 — all carry forward to Phase 3 verbatim.

### 2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation at G16 Atrium handshake

**G14-A1 wave-4a state (2026-05-06 R5):** the canary ships an internal-format UCAN with hand-rolled DAG-CBOR canonical-bytes signature input. This is wire-format-compatible with the project's own consumers (Phase-3 Atrium handshake at G16, capability backend at G14-B). External UCAN spec v0.10 wire-format compatibility (interop with non-Benten UCAN producers) was deferred at G14-A1 mini-review per HARD RULE rule-12 disposition (b) — `ssi` dep dropped from canary.

**G14-A2 wave-4a' (2026-05-06):** VC v1.1-INSPIRED hand-rolled surface (NOT W3C JSON-LD wire-format-compatible) shipped without `ssi`. Same deferral logic: external W3C VC consumers can't verify these credentials without a translation layer, but no Phase-3 internal consumer needs that interop.

**G14-B wave-4b (2026-05-06):** Durable UCAN backend confirmed hand-rolled internal format is sufficient — no external Benten producers exist; the durable layer just persists the same hand-rolled envelope shape G14-A1 already produces. **Named-destination shifted from G14-B → G16 Atrium handshake.**

**Phase 3 G16 re-evaluation point:** when G16 lands the iroh + Loro Atrium peer-to-peer handshake protocol, dispatch a `cryptography-reviewer` agent to assess whether external producer-interop (UCAN spec v0.10 + W3C VC v1.1 JSON-LD) is required. The decision turns on whether Atrium peers from outside Benten (e.g., third-party UCAN issuers, external VC providers) need to participate. If yes, re-introduce `ssi` (or alternative spec-compliant lib) as a translation layer at the Atrium boundary. If no (Phase-3 stays internal-only), preserve hand-rolled.

**Source:** `.addl/phase-3/r5-w4a-g14-a1-mini-review.json` + `.addl/phase-3/r5-w4astar-g14-a2-mini-review.json` `agent_dispositions_assessment.ssi_dropped` fields; G14-B PR #109 module docstring confirmation; this followup section captures the cumulative shift.

### 2.2 SUBSCRIBE delivery-time cap-recheck threading on durable grants (F6) — CLOSED at Phase-3 G14-D wave-5a (PR #115 commit `4d3f688`) + Phase-3 G21-T3 PR #147 partial-revoke

**Closure narrative:** F6 SUBSCRIBE per-subscriber filtering wired at G14-D wave-5a; `Engine::subscribe_change_events` consults a per-subscriber read-cap-coverage closure on each delivery (`crates/benten-engine/src/engine_subscribe.rs::on_change_with_cap_recheck` at lines 301-340; CLR-2 / cap-major-2 dual-layer recheck per audience-binding). Partial-revoke pin (specific grant revoked while actor still active) closed at G21-T3 PR #147 commit `f9147d9`. Replaces the boolean `revoked_actors` set; cross-trust-boundary replicas filter at delivery rather than registration. **§3.2 Per-subscriber filtering is the same item by another name** — see consolidation at §3.2.

**Source (historical):** [`phase-2-backlog.md`](./phase-2-backlog.md) §7.4. Cross-refs `.addl/phase-2b/wave-8-brief.md` §8d-narrative F6.

### 2.3 napi-UCAN-wireup — route `PolicyKind::Ucan` through the durable `UCANBackend` — CLOSED at Phase-3 G21-T2 PR #148 commit `7a6c36a`

**Origin (G20-B mini-review, 2026-05-07):** the doc-engineer + dx-optimizer mini-reviewers on PR #143 (G20-B docs sweep) flagged a doc-vs-code drift cluster: §2.1 above shipped the durable `UCANBackend<B>` at G14-B wave-4b, but the napi binding at `bindings/napi/src/lib.rs::PolicyKind::Ucan` STILL routes to the legacy Phase-1 `benten_caps::UcanBackend` stub (returns `CapError::NotImplemented` / `E_CAP_NOT_IMPLEMENTED` on every `check_write`). End-to-end Atrium examples (`packages/engine/examples/atrium-*.ts`, `ucan-grant-flow.ts`, `did-resolution.ts`) compile + import cleanly against the documented public surface but surface `E_CAP_NOT_IMPLEMENTED` at the first WRITE through the napi layer. The Rust-side durable backend is untouched + correct; the gap is at the binding seam only.

**Findings consolidated here per HARD RULE rule-12 clause-b (named destination receives the entry NOW):**
- `g20b-dx-1` — napi binding routes `PolicyKind::Ucan` to the legacy stub at `bindings/napi/src/lib.rs::Engine::open_with_policy` (the `PolicyKind::Ucan` match arm wires `benten_caps::UcanBackend` rather than the durable `benten_caps::UCANBackend`); the variant docstring at `bindings/napi/src/policy.rs::PolicyKind::Ucan` ("Phase-3 G14-B durable UCAN-grounded grants") was retensed in G20-B but the wire-up was not.
- `g20b-dx-4` — `parse_grant_json` (`bindings/napi/src/policy.rs::parse_grant_json`) reads only `actor` + `scope`; the `issuer` + `hlc` fields documented on `CapabilityGrant` (`packages/engine/src/types.ts:389-392` "Optional issuer CID (Phase-3 UCAN grounding — ignored in Phase 1)") are silently dropped before reaching any backend. Even after the stub→durable rewire, the parser still drops these fields; widening it is part of this entry.
- `g20b-dx-5` — `atrium-sync-trigger.ts` subscribe-callback shim drift (TS-side records the closure; G16-B reconciliation drains real change events). The subscribe wireup half belongs at §3.2 / §7.10 (handler-id-router + per-subscriber filtering); the UCAN-cap-recheck half (delivery-time cap recheck per F6 / §2.2 above) lights up once the durable backend is reachable from the napi surface.
- `g20b-dx-7` — `packages/engine/src/types.ts:389-392` CapabilityGrant docstrings still say "ignored in Phase 1"; retense to past-tense + named-fields-flow-through narrative once G21 T2 lands.
- `g20b-dx-8` — `packages/engine/src/engine.ts:271-272` `PolicyKind.Ucan` JSDoc still says "Phase-3 UCAN stub. Opens but surfaces E_CAP_NOT_IMPLEMENTED at check time" — currently HONEST under stub state; flip to durable narrative once G21 T2 lands.

**Phase 3 target (G21 T2 scope, per `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md` NS-T49):**
- (a) Construct `benten_caps::UCANBackend<B>` (note casing — durable, NOT the stub `UcanBackend`) at `bindings/napi/src/lib.rs::Engine::open_with_policy::PolicyKind::Ucan` arm. The constructor takes a `GraphBackend` reference + `PublisherRegistry` + clock; mirror the `capability_policy_grant_backed()` builder helper shape so the napi adapter does not need to thread the ref soup itself.
- (b) Widen `bindings/napi/src/policy.rs::parse_grant_json` to also read `issuer` (DID string → CID) + `hlc` (numeric stamp). Thread these to `engine.grantCapability(...)` through the napi `grant_capability` adapter so they reach the durable backend's chain-walker.
- (c) Retense `bindings/napi/src/policy.rs:6-7` module doc + `:26-28` variant doc (currently honest for the stub; flip to durable narrative).
- (d) Retense `packages/engine/src/types.ts::CapabilityGrant` `issuer` + `hlc` JSDoc to "consumed by the durable UCAN backend" (drop "ignored in Phase 1").
- (e) Retense `packages/engine/src/engine.ts:271-272` `PolicyKind.Ucan` JSDoc to durable narrative.
- (f) Flip `packages/engine/test/atrium_examples.test.ts` run-invocation pins (added at G20-B-MR fix-pass) from "expect stub failure shape" to "expect successful run() outcome" — this is the GREEN-phase signal that the runtime end-to-end half is real per pim-2 §3.6b.
- (g) End-to-end integration test `bindings/napi/tests/ucan_round_trip.rs` (or equivalent) walking `openWithPolicy(Ucan) → grantCapability → callAs (succeeds) → revokeCapability → callAs (fails E_CAP_DENIED)` end-to-end, with the durable backend genuinely consulted.
- (h) Retense `docs/QUICKSTART.md` Atrium walkthrough `Note (Phase-3-close honest state)` callout — drop the callout (currently warns about the stub) once the wireup lands.

**Touch size:** ~80-150 LOC napi binding (constructor + parser widening + module/variant doc retense) + ~30-50 LOC TS docstring retense + ~50-80 LOC integration test + 4 example test pin flips. Risk surface: low (constructor wiring + parser widening are mechanical; the durable backend is already shipped + tested at the Rust layer).

**Why Phase-3 R5 (NOT Phase-3 close):** the durable `UCANBackend<B>` shipped at G14-B wave-4b but napi binding wireup landed in a separate gate (G21) that absorbs the napi adapter half. NS-T49 (HANDOFF) records the scope expansion: G21 was originally the typed-CALL redirect entry; the napi-UCAN-wireup is FOLDED INTO G21 T2 as a sibling task. The G20-B fix-pass that produced this entry is orchestrator-direct (doc-side admission of the gap + test strengthening to assert the stub-state shape); the actual code-side wireup is owned by the G21 T2 implementer.

**Cross-references:**
- §2.1 Durable UCAN backend in `benten-id` (the durable Rust backend; CLOSED at G14-B).
- §2.2 SUBSCRIBE delivery-time cap-recheck threading on durable grants (F6) — once §2.3 lands, the SUBSCRIBE cap-recheck path threads through the durable grant-store via napi.
- `bindings/napi/src/lib.rs::Engine::open_with_policy` `PolicyKind::Ucan` match arm — the literal surface to rewire.
- `bindings/napi/src/policy.rs::parse_grant_json` — the JSON parser to widen.
- `crates/benten-caps/src/backends/ucan.rs::UCANBackend` — the durable backend (already shipped).
- `packages/engine/test/atrium_examples.test.ts` — run-invocation pins to flip GREEN.
- `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md` NS-T49 — the G21 T2 expanded scope.
- `.addl/phase-3/mini-review-pr-143-g20-b-dx-optimizer.json` (g20b-dx-1, g20b-dx-4, g20b-dx-7, g20b-dx-8) — origin findings.
- `.addl/phase-3/mini-review-pr-143-g20-b-doc-engineer.json` (doc-2) — symbol-cite drift sibling.

#### 2.3 (i) — `WriteContext` audience + clock threading for arbitrary-scope UCAN proof-chain enforcement (G21-T2 fp-mini-review BLOCKER-2 partial-deferral)

**Origin (G21-T2 fp-mini-review, 2026-05-08, post-PR-#148):** the fp-mini-review BLOCKER-2 sec-finding flagged that `EngineBuilder::capability_policy_ucan_durable` was a verbatim alias for `capability_policy_grant_backed` — UCAN proof-chain validation NEVER fired under `PolicyKind::Ucan`. The PR #148 fix-pass closes this for **typed-CALL `cap:typed:*` capabilities** by composing [`benten_caps::UcanGroundedPolicy`] which wraps `GrantBackedPolicy` + `UCANBackend` proof-chain validation + the [`benten_caps::typed_cap_for_ucan_claim`] mapping table.

**What's deferred here (named-destination + named-timing per HARD RULE clause-b):** per-write proof-chain enforcement for ARBITRARY scope-strings (e.g. `store:post:write`, `zone:user:read`, etc.) — the wider lift requires three coupled threading axes:

1. **`WriteContext::actor_hint`-as-DID propagation.** Today `actor_hint` carries an opaque `Cid` (Phase-1 principal); the chain-walker requires a `&Did` for `validate_chain_for_audience_at`. Threading the actor's DID through the CRUD write path + the SUBSCRIBE delivery-time recheck closure is a multi-crate touch (`benten-graph::WriteContext`, `benten-eval::PrimitiveHost::check_capability`, `benten-engine::primitive_host` cap-gate, all `benten-engine/src/engine_*.rs` privileged-write paths).
2. **`WriteContext::now`-as-real-clock injection.** Today the `UcanGroundedPolicy::now_secs` defaults to `0` (epoch start) so present-day fixtures with `nbf=0`+positive-`exp` accept; tests inject custom values via `with_now_for_test`. Production needs `WriteContext::now_secs: u64` populated by the engine's `TimeSource` at every write-check entry. This is the same threading axis as Phase-2a G9-A's wall-clock-refresh cadence work but at the policy-hook surface (vs the per-iteration boundary).
3. **Multi-token chain reference in WriteContext.** Today `UcanGroundedPolicy::iter_installed_proofs` treats each persisted token as a singleton chain (`std::slice::from_ref(proof)`). Multi-token delegation chains need either (a) per-actor chain assembly via parent-CID indexes in the durable store, OR (b) `WriteContext::ucan_chain_ref: Option<Cid>` carrying the leaf-CID with the durable store reconstructing the chain.

**Phase 3 target (re-named 2026-05-08 retense pass):** **v1-assessment-window** per CLAUDE.md item #15. Original "post-G21-T2-close follow-on wave (T3/T4 or sibling cleanup)" became phantom when T3 PR #147 + T4 PR #146 merged without absorbing this scope. Cross-references §10 Compromise registry forward-revisit (the v1-assessment-window destination registry). Each axis (DID propagation / clock injection / multi-token chain reference) is independently scoped; (1) and (2) compose; (3) is a chain-walker extension. The decision shape at v1-assessment is whether arbitrary-scope-string proof-chain enforcement is v1-shippable-blocking — it composes on top of the typed-CALL `cap:typed:*` proof-chain enforcement that DID land at G21-T2.

**Touch size:** ~150-300 LOC across `benten-graph::WriteContext` + `benten-eval::PrimitiveHost` + `benten-engine` cap-gate sites + UcanGroundedPolicy slow-path widening + integration test pins for arbitrary-scope proof-chain enforcement.

**Cross-references:**
- `crates/benten-caps/src/ucan_grounded.rs::UcanGroundedPolicy::typed_cap_permitted_by_proof` — the slow path that today only fires for `cap:typed:*`; this work widens it to arbitrary scope strings with audience binding.
- `crates/benten-caps/src/ucan_grounded.rs::DEFAULT_NOW_SECS` — the epoch-0 fallback that goes away once `WriteContext::now_secs` is populated.
- `crates/benten-engine/src/primitive_host.rs::check_capability` — the policy-hook entry that threads `WriteContext` (currently builds it with `label`-only).
- `crates/benten-engine/tests/typed_call_ucan_grounded.rs` — the BLOCKER-2 partial-closure pin set (3 pins covering typed-cap proof-chain validation under `capability_policy_ucan_durable`).
- `bindings/napi/test/typed_call_napi_cap_gate.test.ts` — the BLOCKER-1 napi-entry-cap-gate companion.

#### 2.3 (ii) — `iterate_batch_boundary` + `wallclock_refresh_ceiling` evaluator-delegation runtime-arm instrumentation (cap-r4-8 carry; G16-B-B-rest BELONGS-NAMED-NOW)

**Origin (R4 cap-system lens; cap-r4-8 / cap-minor-8 closure):** the `crates/benten-caps/src/policy.rs::CapabilityPolicy` trait declares `iterate_batch_boundary(&self) -> usize` + `wallclock_refresh_ceiling(&self) -> Duration` policy hooks (cap-minor-8 R4 finding). Phase-1 + Phase-2a shipped the trait surface; Phase-2a's R6 named cap-minor-8 closure as "G14-B durable UCAN backend wave; bundled with policy delegation tests". G14-B PR #109 (commit `496e144`, 2026-04-30) shipped the durable backend but did NOT thread the policy hooks through to the evaluator runtime arm. The 2 RED-PHASE pins at `crates/benten-caps/tests/wallclock_delegation.rs::policy_iterate_batch_boundary_evaluator_delegation_observable_in_runtime_arm` + `policy_wallclock_refresh_ceiling_evaluator_delegation_observable_in_runtime_arm` carry the closure shape but stayed `#[ignore]`'d through wave-5b/c/wave-7/wave-8a/wave-9 + G16-B/B-prime/B-D/B-rest waves, with a stale rationale referencing G14-B.

**G16-B-B-rest (PR #158, 2026-05-09) disposition:** OUT-OF-SCOPE for the cap+crypto un-ignore wave (the engine-level instrumentation is its own architectural lift; sub-item A budget was 5 R4-FP-file un-ignores at ~50-150 LOC ceiling). NAMED-NOW destination filed here per HARD RULE rule-12 clause-(b) so the stale-rationale-no-destination + phantom-destination anti-pattern doesn't perpetuate.

**What's deferred here (NAMED-NOW destination + named-timing per HARD RULE clause-b):** engine-level evaluator-metrics surface that exposes the per-iteration / per-wallclock-tick refresh-count consumption-points, so the 2 RED-PHASE pins can drive against an observable runtime metric:

1. **`Engine::run_iterate_subgraph_with_metrics(subgraph: &Subgraph) -> Result<IterateMetrics, EngineError>`** — runs an ITERATE-heavy subgraph + returns a metrics struct exposing `refresh_count` (number of times the policy's `iterate_batch_boundary` was consulted). The current `policy.rs:281-326` TODO documents the wire-up gap: the evaluator's iterate_batch loop does NOT today consult `policy.iterate_batch_boundary()` — the loop uses a hardcoded constant.

2. **`Engine::run_call_with_metrics_for_duration(subgraph: &Subgraph, duration: Duration) -> Result<CallMetrics, EngineError>`** — drives a long-running CALL + returns a metrics struct exposing `wallclock_refresh_count` (number of times `policy.wallclock_refresh_ceiling()` triggered). Sibling TODO at `policy.rs:281-326`: the evaluator's CALL slow-path does NOT today consult `policy.wallclock_refresh_ceiling()`.

3. **Un-ignore both pins.** Once items (1) + (2) land, the assertions `assert_eq!(metrics.refresh_count, 100 / 5)` (override = 5; 100 iters → 20 refreshes) + `assert_eq!(metrics.wallclock_refresh_count, 90 / 30)` (override = 30s; 90s call → 3 refreshes) drive end-to-end + each pin gains the source-cite assertion that the `policy.rs:281-326` TODO is closed.

**Phase 3 target (NAMED at G16-B-B-rest 2026-05-09):** **v1-assessment-window** per CLAUDE.md item #15. The cap-r4-8 instrumentation is part of the broader v1-shippable cap-policy delegation surface; whether it's pre-tag closure-blocking depends on (a) whether v1 multi-tenant deployments need policy-driven iterate-batch / wallclock refresh tuning + (b) whether Compromise #1 TOCTOU window bound at iterate boundary (tracked at §10.1) gets revisited together (the 2 hooks compose: policy-driven iterate boundary + cap-snapshot refresh at boundary).

**Touch size:** ~80-150 LOC engine-side (`run_iterate_subgraph_with_metrics` + `run_call_with_metrics_for_duration` runtime entry-points + `IterateMetrics`/`CallMetrics` struct + threading to evaluator) + ~10 LOC test pin un-ignore + ~6 LOC source-cite TODO close at `policy.rs:281-326`.

**Cross-references:**
- `crates/benten-caps/src/policy.rs::CapabilityPolicy::iterate_batch_boundary` + `::wallclock_refresh_ceiling` — the policy hooks today (default-impl returns hardcoded constants; engine evaluator does NOT consult the trait method)
- `crates/benten-caps/tests/wallclock_delegation.rs` — the 2 cap-r4-8 RED-PHASE pins (un-ignore body + the rationale string updated 2026-05-09 G16-B-B-rest fix-pass per HARD RULE clause-b)
- `crates/benten-caps/src/policy.rs:281-326` — the TODO comment block flagging the engine-side wire-up gap
- §10.1 Compromise #1 TOCTOU window bound — the v1-assessment-window peer item that may compose (iterate-batch-boundary cap-recheck cadence is a shared concern)

#### 2.3 (iii) — W3C did:key external interop cross-vector verification (G16-B-B-rest cryptography MINOR carry)

**Origin (G16-B-B-rest cryptography mini-review, 2026-05-09):** sub-item C added 3 W3C did:key "test vectors" + a fail-closed multicodec pin (`UnknownMulticodec(0x00, 0x00)`) at `crates/benten-id/tests/did_key.rs`. The added vectors are derived by feeding hex pubkeys through the canonical W3C did:key v1.0 pipeline and then asserting the encoder's output matches itself — meaning a systematic encoder bug would round-trip with itself. The pre-existing G14-A2 deferral specifically named cross-implementation interop as the unmet criterion.

**G16-B-B-rest disposition:** the encoder-output pinning DOES catch encoding-step drift (round-trip with re-encode) but does NOT catch encoder bugs that deviate from W3C reference implementations. The cryptographic-discipline distinction matters for v1-shippable cross-protocol UCAN interop: another DID-resolving stack (ssi crate, did-key.rs, kepler.xyz, etc.) accepting Benten-emitted did:keys requires byte-equality with externally-published reference vectors, NOT just encoder self-consistency.

**Phase 3 target:** **v1-assessment-window** per CLAUDE.md item #15. Lift options:

1. **Cross-check against `did-key.rs` crate's exposed test fixtures.** Add a dev-dep + import their pinned test vectors; assert byte-equality against Benten encoder output. Catches divergence from the reference impl.
2. **Inline externally-published W3C reference vectors verbatim** (not derived). Source: `https://w3c-ccg.github.io/did-method-key/test-vectors/` (verify availability + license at v1-assessment-window time). Catches encoder drift independent of any specific reference impl.
3. **Both** — defends against encoder bugs (option 1) AND drift from W3C spec (option 2).

**Touch size:** ~30-80 LOC test-fixture inline + dev-dep update if option 1 chosen.

**Cross-references:**
- `crates/benten-id/tests/did_key.rs` — the 3 encoder-output-pinned vectors + the fail-closed `UnknownMulticodec` pin landed at PR #158
- `crates/benten-id/src/did.rs` — the encoder + decoder; the multibase z-prefix + 0xed01 multicodec discriminator
- G14-A2 cryptography mini-review (`.addl/phase-3/r5-w4a-g14-a1-mini-review.json`) — the original cross-implementation interop deferral

### 2.5 typed-CALL fp-mini-review residuals (G21-T1 sec-minor-2/3/4 + corr-minor-3)

**Origin (G21-T1 fp-mini-review, 2026-05-08):** the security mini-review on PR #145 surfaced 4 MAJORs (closed end-to-end at PR #145 fix-pass) plus 4 minors. The MAJORs are closed; the four named carries below land in a follow-up wave so the canary-scope PR remains tight.

**Carry items:**

**(a) typed-CALL secret-byte zeroize discipline (sec-minor-2).** `build_seed_envelope` in `crates/benten-engine/src/typed_call_dispatch.rs` constructs a `Vec<u8>` carrying a 32-byte seed; the Vec is dropped naturally but is NOT zeroized-on-drop. The `keypair_generate` / `keypair_from_seed` ops also surface raw seed bytes inside a `Value::Bytes` wrapper that has no zeroize discipline. Phase 3 target: introduce a `zeroize::Zeroizing<Vec<u8>>` wrapper at the `build_seed_envelope` site + audit the per-op `Value::Bytes` outputs for whether wrapping at the dispatch boundary is feasible without breaking the public Value contract. `zeroize` is already a workspace dep (`Cargo.toml` line 367) + `benten-id` consumes it; threading into `benten-engine` is a one-line `Cargo.toml` add. Touch size: ~30-60 LOC (wrapper + 2-3 callsite swaps + a "zeroize-discipline-pin" memory-pattern test).

**(b) `did_resolve` op DID-method validation (sec-minor-3).** `crates/benten-engine/src/typed_call_dispatch.rs::did_resolve` hardcodes `method: "key"` in the output map but the input string is not parsed for its DID method. Phase-3+ `did:web:`, `did:plc:` etc. would silently produce a wrong `method` field. Phase 3 target: parse the method dynamically from the DID prefix (the segment between `did:` and the next `:`); if `did:key:`, route through current resolver; non-`did:key:` methods either route through future resolvers OR reject with a typed error. Touch size: ~20-40 LOC + 2-3 input-shape pins.

**(c) `cap:typed:*` namespace consumer-side mapping (sec-minor-4).** The 8 typed-CALL caps (`cap:typed:crypto-sign` / `cap:typed:crypto-verify` / `cap:typed:crypto-keygen` / `cap:typed:hash` / `cap:typed:codec` / `cap:typed:did-resolve` / `cap:typed:ucan-validate` / `cap:typed:vc-verify`) are STRUCTURALLY declared at `TypedCallOp::required_cap()` but `crates/benten-caps/src/backends/ucan.rs::UCANBackend` does not yet have a policy-mapping table that says "this UCAN claim string corresponds to typed-CALL cap X." Under `NoAuthBackend` all typed caps are permitted (canary-scope intent); under UCAN the cap-deny-by-default behavior surfaces because no UCAN claim grants a `cap:typed:*` capability. Phase 3 target: add a UCANBackend → typed-cap mapping function so a UCAN claim like `Capability::new("typed:crypto", "sign")` grants `cap:typed:crypto-sign` (or whatever cap-grammar Phase-3 sync settles on). Touch size: ~50-100 LOC + per-op mapping pin.

**(d) Reserved `engine:typed:` handler-id namespace registration reject (corr-minor-3).** `Engine::register_subgraph` does not currently reject a handler whose `handler_id` starts with `engine:typed:`. The eval-side dispatch fork pre-empts user-handler routing for the prefix, so user registration is effectively dead code (currently pinned at `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_namespace_pre_empts_user_handler_registry_for_unknown_op`), but a hard registration-time REJECT would surface the user-error sooner. Phase 3 target: add an `EngineError::ReservedHandlerNamespace { handler_id }` variant + corresponding `ErrorCode::ReservedHandlerNamespace` (4-surface §3.5g atomic update across `benten-errors` lib + JS adapter regen + ERROR-CATALOG row + TS bindings). Touch size: ~80-120 LOC across the 4 ErrorCode surfaces + 1 register_subgraph guard + 1 pin.

**(e) `Value::SensitiveBytes` discriminant for typed-CALL secret-byte zeroize discipline (G21-T2 fp-mini-review MAJOR-6, deferred-half).** PR #148 fp-mini-review MAJOR-6 closure took option (b) — renamed `Keypair::secret_bytes_for_test` → `secret_bytes_unprotected` so the lack of zeroize-on-drop on the returned `[u8; 32]` is explicit at every call site. The proper option (a) — introducing a `Value::SensitiveBytes(Zeroizing<Vec<u8>>)` discriminant on the `benten_core::Value` enum — is deferred here because it is a cross-crate touch on every `Value` consumer (matchers in `benten-eval`, `benten-engine`, every primitive impl, every napi marshaller, the DSL compiler's value-shape builder, the IVM-view query path). Phase 3 target: add the discriminant on `Value`, route typed-CALL secret-byte outputs through it, audit `napi-rs` marshalling at the JS boundary so the bytes flow into a JS `Uint8Array` without first materializing as a non-Zeroizing intermediate. Touch size: ~150-250 LOC across the 6+ Value-consumer crates + new variant on `benten-eval::EvalError::TypedCallInvalidInput` shape audit if `SensitiveBytes` flows in to dispatch input. Cross-cuts §2.5 (a) which scoped the simpler `build_seed_envelope` `Zeroizing<Vec<u8>>` wrap (already landed at G21-T1 fix-pass).

**Phase 3 target:** post-G21-T1 follow-on (T2/T3/T4 or a sibling cleanup wave). Each item is independently scoped; (c) couples to the napi-UCAN-wireup G21-T2 work in §2.3; (e) is named at G21-T2 fp-mini-review.

**Touch size:** ~330-570 LOC total across (a)-(e).

#### 2.5 (f) — G21-T2 fp-mini-review DX residuals cluster

**Origin (G21-T2 fp-mini-review, 2026-05-08):** the fp-mini-review surfaced 4 minor DX-class findings disposed as SKIP-here-with-named-destination per HARD RULE clause-b. None block the load-bearing security closures (BLOCKERs 1-3 + MAJORs 4-7); each is named here NOW so a future DX cleanup wave can pick up.

**Carry items (4 minors, follow-up wave):**

1. **DX-1: cargo dx items** — observed during fp-mini-review across the fix-pass commit set; symptom is workspace-level cargo command surface ergonomics. Surface: workspace `Cargo.toml` + `xtask` if added; touch ≤30 LOC.

2. **DX-2: display rendering improvement** — the `format!("{err:?}")` debug-rendering patterns in the new typed-CALL pin set produce verbose output. A `Display` impl on the typed `EngineError` variants would render more readably. Surface: `crates/benten-engine/src/error.rs::EngineError::Other` Display arm; touch ≤30 LOC.

3. **DX-3: `ucan_validate_chain` test fragility** — the existing `ucan_validate_chain_returns_*` pins in `crates/benten-engine/tests/typed_call_engine_dispatch.rs` build chains via the `Ucan::builder` + manual `now`-fixture composition; a test fixture helper that bundles the pattern would reduce duplication + clarify intent. Surface: a new `tests/common/ucan_fixtures.rs` shared helper module; touch ≤80 LOC.

4. **DX-4: multi-Atrium handle dedup** — `bindings/napi/src/atrium.rs::JsAtrium::from_engine` constructs a fresh `AtriumHandleState` per call; multiple calls with the same `atriumId` produce distinct handles routing to the same logical atrium per Ben's D1 ratification (intentional). However, the `AtriumHandleState::declared_attestations` registry is per-handle, so attestations declared on handle A are NOT visible on handle B for the same atrium. Phase 3 target: hoist the per-atrium registries to an engine-level table keyed on `atriumId`. Surface: `bindings/napi/src/atrium.rs` + new engine-level registry on `Engine`; touch ~50-80 LOC.

**Phase 3 target:** post-G21-T2-close DX cleanup wave. Independently scoped from §2.5 (a)-(e) above; lands in a separate sibling PR.

**Touch size:** ~190-220 LOC total across the 4 minors.

**Cross-references:**
- `bindings/napi/src/atrium.rs::JsAtrium::from_engine` — DX-4 surface.
- `crates/benten-engine/src/error.rs::EngineError::Other` — DX-2 surface.
- `crates/benten-engine/tests/typed_call_engine_dispatch.rs::ucan_validate_chain_returns_true_for_well_formed_chain` (and the 3 other `ucan_validate_chain_returns_*` siblings in the same file) — DX-3 surface.

**Cross-references:**
- `crates/benten-engine/src/typed_call_dispatch.rs` — sec-minor-2 + sec-minor-3 surfaces.
- `crates/benten-eval/src/typed_call.rs::TypedCallOp::required_cap` — sec-minor-4 cap-name source.
- `crates/benten-caps/src/backends/ucan.rs::UCANBackend` — sec-minor-4 destination.
- `crates/benten-engine/src/engine.rs::register_subgraph` — corr-minor-3 destination.

---

### 2.4 `phase_2b_landed` feature gate retirement in benten-core / benten-ivm / benten-engine / benten-errors (audit-3-mr-1-extended) — CLOSED at Pre-R4b orchestrator-direct fix-pass batch (PR #144 commit bd87cde)

**Origin (G20-B audit-3 mini-review, 2026-05-07):** G20-B v3 closed the `phase_2b_landed` feature gate in `benten-eval` + `benten-dsl-compiler` + `benten-dev` (Cargo.toml entries deleted + `#![cfg]` gates stripped from 23+ test files; commit message documents the narrow scope). The audit-3 reviewer extended scope flagged that the feature gate STILL exists in the other four crates: `benten-core`, `benten-ivm`, `benten-engine`, `benten-errors`. These were intentionally out-of-scope at G20-B v3 to keep the wave's audit-3 narrowly scoped. The residual is named here per HARD RULE rule-12 clause-b — the destination receives the entry NOW, not "later".

**Phase 3 target (orchestrator-direct fix-pass batch, scheduled pre-R4b dispatch per HANDOFF NS-T49):**
- (a) Audit each of `crates/benten-core`, `crates/benten-ivm`, `crates/benten-engine`, `crates/benten-errors` Cargo.toml for `phase_2b_landed` feature entries — delete.
- (b) Strip every `#![cfg(feature = "phase_2b_landed")]` / `#[cfg(feature = "phase_2b_landed")]` + adjacent `#[cfg(not(feature = "phase_2b_landed"))]` gate from `tests/**/*.rs` + `src/**/*.rs` in those four crates.
- (c) `cargo +stable check --workspace --all-targets` to confirm no stale references remain (the gate name itself should not appear in the workspace anywhere post-batch).

**Touch size:** ~30-50 LOC mechanical (4 Cargo.toml edits + N test-file gate strips). Risk surface: low (the gate is technical debt post-Phase-2b-close; the gate name no longer carries semantic meaning).

**Why Phase-3 R5 (NOT G20-B v3):** G20-B's commit-message-documented narrow scope kept v3 targeted. The audit-3-mr-1-extended finding is the right shape to fold into a sibling orchestrator-direct fix-pass batch — the batch is scheduled in HANDOFF NS-T49 PRE-R4b dispatch so R4b reviewers see a clean tree.

**Cross-references:**
- `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md` NS-T49 — the orchestrator-direct fix-pass batch destination.
- `.addl/phase-3/mini-review-pr-143-g20-b-doc-engineer.json` doc-4 — origin disposition flag (HARD RULE clause-b destination-naming).
- G20-B v3 commit narrative — narrow-scope rationale for the partial close.

---

## 3. Networking / sync

### 3.1 Atriums (P2P direct connections via iroh + Loro CRDTs)

**Phase 2b state:** Phase 2b is single-process. No networking surface.

**Phase 3 target:** This IS Phase 3's headline scope per FULL-ROADMAP.md. Iroh (peer-to-peer transport) + Loro (CRDT for collaborative graph merges) + ed25519-dalek + ssi (Ed25519 / DID / VC for identity).

**Phase-3 R5 landing state (2026-05-08):** structural surface partially landed across G14-D (per-subscriber filtering F6) + G16 wave (iroh transport canary + Loro per-zone CRDT registry + benten-sync 10th crate native-only) + G21-T2 (Engine.atrium factory + JsAtrium delegation closing audit-6-2 BLOCKER) + G21-T3 (DeviceAttestation engine-side recording + RED-PHASE pin for on-the-wire emission). **Substantive end-to-end multi-peer iroh sync remains in scope for Phase-3-close per Ben ratification 2026-05-08.** Work named at §3.1-followup below.

**Source:** [`docs/VISION.md`](../VISION.md) "Atriums (Phase 3 committed) — peer-to-peer direct connections."

### 3.1-followup Substantive end-to-end multi-peer iroh sync — CLOSED at G16-B-E (2026-05-09)

**Origin:** R4b distributed-systems lens flagged that the iroh transport + multi-peer end-to-end sync test pins remained RED-PHASE at HEAD `7a6c36a`. Plan §1 exit-criterion 1 mandated "Two full peer instances of `benten-engine` ... sync a shared subgraph bidirectionally over iroh transport" + the named end-to-end pins.

**Closure:** G16-B-E wave landed substantive end-to-end iroh transport between full-peer `benten-engine` instances + drove the canonical pins green. Phase-3 exit-criteria 1 + 15 closed end-to-end.

**What landed (G16-B-E commits on `wave-g16-b-e`):**

- **Sub-item D — receiver-side ChangeEvent fan-out** (`crates/benten-engine/src/engine_diagnostics.rs::Engine::append_version`): routed the new-Version-Node put through `backend.transaction(|tx| tx.put_node(node))` rather than the inherent `backend.put_node`. The transactional path is the one that fans `ChangeEvent`s out to registered subscribers — without it `apply_atrium_merge`'s receiver-side `subscribe_change_events` ChangeProbe would observe ZERO events on a successful Loro merge.
- **Sub-items A + B + C — iroh-transport substantive multi-peer pins:**
  - `crates/benten-engine/tests/atrium_g16_b_e_substantive_e2e.rs::three_peer_loro_convergence_via_iroh_transport_concurrent_writes` (3-peer iroh-transport convergence; distinct from the pre-G16-B-E `atrium_lifecycle.rs::three_peer_loro_convergence_under_concurrent_writes` which exercised only `merge_remote_change` direct calls).
  - `crates/benten-engine/tests/atrium_g16_b_e_substantive_e2e.rs::apply_atrium_merge_advances_anchor_chain_and_drains_change_events_on_receiver` (receiver-side Sub-item-D pin asserting ChangeProbe drains the merge-Version CID).
  - `tests/integration/atrium_two_peer.rs::atrium_two_peer_bidirectional_sync` — exit-criterion 1 LOAD-BEARING; un-ignored.
  - `tests/integration/atrium_three_peer.rs::atrium_three_peer_loro_convergence_under_concurrent_writes` — exit-criterion 15 LOAD-BEARING + C-10; un-ignored.
- **Sub-item E — asymmetric reachability typed-error pin:**
  - `crates/benten-sync/tests/atrium_partial_partition.rs::atrium_partial_partition_asymmetric_reachability_observable_state_explicit` un-ignored against the iroh transport surface directly. Surfaces `AtriumTransportError` mapped to `benten_errors::ErrorCode::AtriumTransportDegraded` per net-blocker-2 + net-major-3.
  - `crates/benten-engine/tests/atrium_g16_b_e_substantive_e2e.rs::atrium_partial_partition_asymmetric_reachability_observable_state_explicit` engine-side companion (works alongside the prior A↔B leg + asserts post-partial-partition the A↔B path remains functional).

**Items still RED-PHASE / scoped elsewhere:**

- `tests/integration/atrium_two_process.rs::atrium_two_process_bidirectional_sync_end_to_end` — scope-real-22 cross-process variant, scoped to G16-D wave-6b (handshake protocol body + cross-process driver are coupled).
- `tests/integration/atrium_two_device.rs::atrium_two_device_same_identity_selective_zone_sync` — exit-criterion 16 multi-device-same-identity. **CLOSED 2026-05-09 G16-D wave-6b + fix-pass** — un-ignored + GREEN with REAL signed device attestations. Two engines under same actor_cid + distinct device_cids + REAL `benten_id::Keypair`-derived `did:key:z<base58>` device-DIDs + parent-signed `DeviceAttestation` envelopes sync bidirectionally over real iroh transport. The on-the-wire `DeviceAttestationEnvelope` is V2 (signed; payload-hash bound; session-nonce replay-defended; verified at receive via composition of `benten_id::Acceptor::accept_at` + envelope-signature check + constant-time BLAKE3 payload-hash). Receiver-side AttributionFrame.device_did reflects the ORIGINATING device per Inv-14 device-grain attribution — now LOAD-BEARING under adversarial-peer assumptions (forgery / replay / frame-pair-swap all reject with `E_DEVICE_ATTESTATION_FORGED`; pinned end-to-end at the 4 sibling test fns in the same file). Cap-denial halves (heterogeneous-cap-envelope per-device write filter — phone CANNOT write to /zone/notes) carry to **§6.12 item 9 below** (NEW — heterogeneous-cap-envelope per-device write filter at sync-replica boundary; the unblocking dependency — verified device-DID-attestation envelope — landed at this wave's fix-pass).
- `tests/integration/atrium_three_peer.rs::atrium_three_peer_concurrent_writes_under_partial_revoke_with_offline_reconnect_converges` — ds-r4-1 Byzantine-class proptest; pairs with G14-D wave-6b cap-recheck-at-delivery + G16-C MST-diff-on-reconnect drain ordering surfaces.
- `crates/benten-sync/tests/transport_loopback.rs::iroh_transport_relay_fallback_when_holepunch_fails` + `iroh_transport_holepunch_smoke` — scope-real-10 CI-conditional gating; G16-D wave-6b un-ignores once iroh test-fixture (synthetic NAT + relay endpoint) wires per pim-4 §3.10 wave-paired closure.

**Cross-references:**
- ds-r4b-1 finding in `.addl/phase-3/r4b-distributed-systems.json` (BLOCKER closed at G16-B-E)
- `.addl/phase-3/00-implementation-plan.md §1` exit criteria 1 ✅ + 15 ✅ + 16 ✅ FULL CLOSURE (multi-device-same-identity CLOSED at G16-D wave-6b + fix-pass — signed on-the-wire device-DID-attestation envelope V2 landed; cryptographic-attestation closure for criterion 16 ratified by Ben 2026-05-09; cap-denial halves of the original test pin steps 3+6 carry to **§6.12 item 9** newly registered below — the heterogeneous-cap-envelope per-device write filter at sync-replica boundary, which keys its filter on the verified device-DID-attestation envelope this wave's fix-pass landed). See SECURITY-POSTURE.md Compromise #23 for the full cryptographic closure narrative.
- `.addl/phase-3/WAVE-G16-B-E-BRIEF.md` (wave brief; spec sources)
- `.addl/phase-3/HANDOFF-2026-05-03-phase-3-kickoff.md` NS-T64 + NS-T65 (canary scope clarification 2026-05-08)

### 3.3 napi `Acceptor` extension surface — revocation list + expected-parent gate

**Source:** R6-FP Wave A Sub-A2 BELONGS-NAMED-NOW per HARD RULE rule-12.

**Phase-3-close state:** `bindings/napi/src/atrium.rs::JsAtrium::set_acceptor` exposes the freshness-window-only Acceptor ctor (constructed via `benten_id::device_attestation::Acceptor::new(FreshnessPolicy::seconds(window))`). Two further Acceptor ctor surfaces exist Rust-side that the napi boundary does NOT yet expose:

1. **`Acceptor::new_with_revocations(freshness, Vec<DeviceRevocation>)`** — pre-populated revocation list. JS callers cannot today install custom revocation rosters at the napi boundary; the per-Acceptor revocation surface is therefore Rust-only. Composing this requires a `JsDeviceRevocation` napi class (mirrors `benten_id::device_attestation::DeviceRevocation::issue` with parent-keypair signing).
2. **`Acceptor::with_parent_lookup(expected_parent: Did)`** — gates inbound attestations on issuer-DID equality with a configured expected parent. JS callers cannot today install expected-parent gating; the surface is Rust-only.

**Phase-N target:** add `JsDeviceRevocation` napi class + `JsAtrium::set_acceptor_with_revocations(freshness_window_secs, revocations: Vec<&JsDeviceRevocation>)` + `JsAtrium::set_acceptor_with_parent_lookup(parent_did: String)` napi methods. Round-trip TS pin asserting JS-installed revocation roster rejects pre-revoked device-DIDs at handshake.

**Phase carrier:** post-Phase-3 (no Phase-3 carrier; out of v1-milestone-gate critical path per CLAUDE.md baked-in #15 — full peers can fall back to engine-direct configuration during the test runner setup).

### 3.2 Per-subscriber filtering on the change-event stream — CLOSED at Phase-3 G14-D wave-5a (PR #115 commit `4d3f688`) + Phase-3 G21-T3 PR #147 partial-revoke

**Closure narrative:** F6 SUBSCRIBE per-subscriber filtering is the same item by another name as §2.2 above; both were named separately during planning (one in phase-2-backlog.md §1 carry list, one in phase-3-backlog.md §2.2). Closed at G14-D wave-5a + G21-T3 partial-revoke; full closure narrative + file:line cites at §2.2 above. Cross-trust-boundary subscribers filter at delivery time per CLR-2 dual-layer recheck.

**Source (historical):** [`phase-2-backlog.md`](./phase-2-backlog.md) §1, [`docs/SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) §"Change-stream subscription bypasses capability read-checks."

---

## 4. Browser / wasm32-unknown-unknown

### 4.1 Compromise #19 — IndexedDB-backed persistent module-manifest store

**Renumbering note:** previously labeled "Compromise #N+8" before R6FP Group 3 globalized the numbering to match `docs/SECURITY-POSTURE.md` (#1-#21).

**Phase 2b state:** `bindings/napi/src/wasm_browser.rs::BrowserManifestStore::is_persistent` returns `false`. Module manifests are in-memory only on the browser target. Compromise #19 in `docs/SECURITY-POSTURE.md`.

**Phase 3 target:** IndexedDB-backed persistent manifest store. Pairs with PHASE-3-BUNDLE-1 (BrowserBackend) since both are browser-target persistence work — likely a single Phase-3 wave covers both.

**Phase 3 G18-A wave-5a state (PARTIAL).** Schema + handler scaffolding landed at G18-A (`bindings/napi/src/browser_indexeddb.rs` + `bindings/napi/src/browser_blob_store.rs`); wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures` plumbing deferred to §4.3 G18-A-followup wave below. `BrowserManifestStore::is_persistent()` + `IndexedDbBlobBackend::is_persistent()` HONESTLY stay `false` until that wave wires per the honest-disclosure principle Compromise #19 originally articulated.

### 4.2 Compromise #20 — Cross-browser determinism CI cadence

**Renumbering note:** previously labeled "Compromise #N+9" before R6FP Group 3 globalized the numbering to match `docs/SECURITY-POSTURE.md` (#1-#21).

**Phase 2b state:** Per-browser engine bytecode + JIT non-determinism makes per-PR cross-browser CID pinning premature. The cross-browser determinism job in `wasm-browser.yml` is gated on `release` events + `workflow_dispatch` only. Per-PR CI runs the bundle build + size cap + single-browser smoke without pinning a fixture CID across engines.

**Phase 3 target:** Engine-side determinism work that closes the compromise; flip the cross-browser job to per-PR cadence. Source: `docs/future/phase-2-backlog.md` §10.2.

**Phase 3 G18-A wave-5a state (PARTIAL).** Workflow + matrix cell structure landed at G18-A (`.github/workflows/cross-browser-determinism.yml`); fixture bodies deferred to §4.3 G18-A-followup wave below.

### 4.3 G18-A-followup — IndexedDB integration + Playwright fixture authoring

**Named destination for two G18-A wave-5a Q3 IFF-clause deferrals** (per HARD RULE rule-12 clause-b — destination NAMED + receiving the entries NOW). Closes the BLOCKER finding `g18a-mr-1` from PR #114 mini-review and the PARTIAL-CLOSURE narrative carry on Compromise #19 + #20 in `docs/SECURITY-POSTURE.md`.

**Two coupled work items (single follow-up wave or split — TBD at dispatch time):**

**(a) wasm32 IndexedDB plumbing — `web-sys` / `js-sys` / `wasm-bindgen-futures` wire-up.** Phase-3 G18-A landed the IndexedDB schema + handler scaffolding at `bindings/napi/src/browser_indexeddb.rs` (schema-version constant, object-store names, `on_upgrade_needed` chain walker, `on_version_change` handler shape, `map_dom_exception_to_error_code`, `INDEXEDDB_DATABASE_NAME`) but the wasm32 arms of `apply_migration_step` + `close_database` are stubs with comments-only. G18-A-followup wires the wasm32 arms to actual `IDBDatabase.open` / `IDBObjectStore.put` / `IDBObjectStore.get` / `IDBDatabase.close` calls via `web-sys` + `wasm-bindgen-futures::JsFuture` adapters. Adds `web-sys` (with required feature flags for `IdbDatabase`, `IdbFactory`, `IdbObjectStore`, `IdbRequest`, `IdbVersionChangeEvent`), `js-sys`, `wasm-bindgen-futures` deps under `[target.'cfg(target_arch = "wasm32")'.dependencies]` in `bindings/napi/Cargo.toml` (the existing `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` cascade pattern is preserved INTACT). Once wired:
- `BrowserManifestStore::is_persistent()` flips `false → true` on wasm32 (gated on runtime IDB-open success — returns `false` on native).
- `IndexedDbBlobBackend::is_persistent()` flips `false → true` on wasm32 (same gate).
- `BrowserManifestStore::open_indexed_db(...)` constructor lands as the production browser-target manifest-store entry point (the existing `new()` stays for tests + non-browser dev hosts).
- `Engine::open_with_browser_blob_backend(...)` constructor lands wiring `IndexedDbBlobBackend` into the engine snapshot-cache surface via the `BlobBackend` trait surface locked at G13-pre-B.

Estimated touch size: ~200-400 LOC across `browser_indexeddb.rs` (wasm32 arms) + `browser_blob_store.rs` (wasm32 arms + `is_persistent` cfg-gating) + `wasm_browser.rs` (`is_persistent` cfg-gating + `open_indexed_db` constructor) + `bindings/napi/Cargo.toml` (wasm32-only dep additions) + 2-3 new integration tests. Bundle-size impact: estimated ~30-80 KB raw / ~10-25 KB gzipped (web-sys feature flags are conservative; only `IdbDatabase` family symbols added) — keeps the wasm-r1-7 ≤600 KB cap honest.

**(b) Playwright fixture authoring for `cross-browser-determinism.yml` matrix cells.** Phase-3 G18-A landed the workflow + matrix cell structure at `.github/workflows/cross-browser-determinism.yml`; every cell currently emits `::warning::...harness fixture not yet wired (G18-A-followup)`. G18-A-followup authors the fixture bodies that drive each cell to real assertions per pim-2 §3.6b end-to-end test pin requirement. The 11 fixture bodies (per the 11 `::warning::` emit sites in the workflow):

1. `node_envelope` canonical-bytes — load bundle in browser, encode a canonical Node envelope, assert byte-identity against native canonical fixture (`bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`).
2. `handler-version-chain` — encode a handler-version-chain entry in browser, assert byte-identity.
3. `AttributionFrame-with-DID` — encode an AttributionFrame with device DID in browser, assert byte-identity.
4. `canonical-fixture-corpus` — load the canonical fixture corpus in browser, assert CID match.
5. `BLAKE3-byte-identity` — drive a BLAKE3 hash through the browser SIMD path + non-SIMD path, assert byte-identity with native.
6. `Ed25519-signature-byte-identity` — sign a fixed message in browser, assert signature byte-identity with native (deterministic-signing path).
7. `floating-point-canonicalization` — exercise NaN bit-pattern + denormal + round-to-even DSL eval cases, assert canonical-bytes match.
8. `IndexedDB schema_migration_round_trip` — call `IDBFactory.open` with `version=1`, populate, then call with `version=2` to trigger `onupgradeneeded`, assert chain-walker fired in correct order + no data loss.
9. `IndexedDB no_data_loss 1000_key sweep` — populate 1000 keys at v1, upgrade to v2, assert all 1000 keys still readable.
10. `QuotaExceededError → E_STORAGE_QUOTA_EXCEEDED typed-error mapping` — write an oversized blob to IndexedDB until quota fires, assert the error surfaces as `BentenError(code=E_STORAGE_QUOTA_EXCEEDED)`.
11. `cid_pin three-browser equivalence reduce step` — collect CID outputs from chromium / gecko / webkit cells, cross-check identity in a reduce job.

Estimated touch size: ~300-600 LOC of test infrastructure across `bindings/napi/tests/playwright/` (NEW dir) + `package.json` Playwright dep additions + `playwright.config.ts` + 11 fixture spec files. Each fixture body is ~30-60 LOC (load bundle / set up IDB / drive assertion / report exit-code). Workflow-side changes: replace each `::warning::...harness fixture not yet wired` echo with the actual `npx playwright test --grep "<fixture-name>"` invocation gated on the fixture spec file existing. The Rust-side workflow-pin tests at `bindings/napi/tests/cross_browser_determinism_workflow_pins.rs` get re-shaped to assert the fixture INVOCATIONS (not the warning emits) are present in the YAML.

**Acceptance criteria for closing Compromise #19 + #20 fully (status `CLOSED` not `PARTIALLY CLOSED`):**

- `BrowserManifestStore::is_persistent()` returns `true` on wasm32 builds (gated on runtime IDB-open success).
- `IndexedDbBlobBackend::is_persistent()` returns `true` on wasm32 builds (same gate).
- All 11 Playwright matrix cells in `cross-browser-determinism.yml` execute real assertions (no `::warning::...harness fixture not yet wired` emits remain).
- The matrix workflow's GitHub Actions job-summary shows assertion pass/fail per cell (not just structural success).
- A regression that breaks canonical-bytes determinism in the wasm32 bundle would FAIL the matrix workflow per pim-2 §3.6b.

**Why deferred from G18-A.** The schema + handler scaffolding + workflow + matrix structure are the LARGER surface that lets the full closure work be split cleanly. Wiring the wasm32 IDB plumbing + authoring 11 Playwright fixture bodies + adding `web-sys` deps in one wave would have crossed the implementer-agent sweet-spot LOC budget (~400-800) by ~2x. Splitting at the scaffolding boundary lets each half land cleanly with its own mini-review pass.

**Touch size:** ~500-1000 LOC total (a-half ~200-400 + b-half ~300-600).

### 4.4 Bundle-content audit pins — CLOSED at Phase-3 R6 fix-pass Wave B

**Origin:** R4b architecture / wasm-bundle lens (Phase-3 G16-B-D pre-dispatch brief sub-item D, 2026-05-09) flagged that the wasm32-unknown-unknown bundle MUST be auditable to NOT contain (per CLAUDE.md baked-in #17 — full peer vs thin compute surface):
- iroh transport bytes (full peer–only)
- Loro CRDT bytes (full peer–only)
- redb backend bytes (full peer–only)
- SANDBOX runtime / wasmtime bytes (full peer–only)

**Status:** **CLOSED** at Phase-3 R6 fix-pass Wave B per R6 R1 br-r6-r1-1 BLOCKER + ds-r6-2 MAJOR convergence-council ratification. The closure landed all three concrete fix-shape clauses:

- **(a) `wasm-browser.yml` bundle-content audit step.** Extended the existing `bundle-and-smoke` job with a `Bundle-content audit (forbidden symbols per CLAUDE.md baked-in #17)` step that runs `wasm-objdump -x` against the produced `.wasm` artifact + greps for the 4 forbidden crate prefixes (`loro` / `iroh` / `redb` / `wasmtime`) + asserts ZERO matches. The step installs `wabt` (WebAssembly Binary Toolkit) for `wasm-objdump`. A regression that pulled any of the 4 forbidden crate prefixes into the wasm32 bundle fails CI immediately with the matched-symbol output.
- **(b) Rust-side workflow-pin test.** `bindings/napi/tests/wasm_bundle_content.rs` un-ignored with workflow-pin bodies that assert the audit step exists + cites the 4 forbidden prefixes (5 tests: `loro_not_in_browser_bundle_per_baked_in_17`, `iroh_not_in_browser_bundle_per_baked_in_17`, `redb_not_in_browser_bundle_per_baked_in_17`, `wasmtime_not_in_browser_bundle_per_baked_in_17`, `bundle_content_audit_step_asserts_zero_forbidden_symbols`). Follows the `cross_browser_determinism_workflow_pins.rs` pattern.
- **(c) Documentation cross-reference.** `docs/SECURITY-POSTURE.md` Compromise #19 + #20 narratives reference §4.3 (G18-A-followup); this §4.4 entry retensed to CLOSED.

**Defense-in-depth shape (now LIVE).** Three rungs defend CLAUDE.md baked-in #17 thin-client surface commitment:
1. **Source-side cfg-gating** — `crates/benten-sync/src/lib.rs` `compile_error!` for `target_arch = "wasm32"` + Cargo.toml `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`. Pinned by `crates/benten-sync/tests/wasm32_excluded.rs::benten_sync_does_not_compile_for_wasm32_unknown_unknown_per_thin_client_commitment`.
2. **Cargo feature-graph closure** — no transitive activation of full-peer crates from the browser-bundle root. Pinned by `bindings/napi/tests/feature_graph_closure_no_test_helpers_in_production.rs` (also closes §10.6 v1-window destination — see below).
3. **Built-bundle symbol-section audit** — `wasm-objdump -x` + forbidden-symbol grep in `wasm-browser.yml`. Pinned by `bindings/napi/tests/wasm_bundle_content.rs`.

**Companion closure (ds-r6-3 MAJOR — at-build-time wasm32-refusal CI cell).** `.github/workflows/wasm-checks.yml` `benten-sync-refuses-wasm32` job runs `cargo check --target wasm32-unknown-unknown -p benten-sync` + asserts the build FAILS with `compile_error!` macro firing + classifier verifies the failure cites `compile_error!` / `baked-in #17` / `target_arch` (not an unrelated dep-graph break). Pinned by `crates/benten-sync/tests/wasm32_excluded.rs::benten_sync_wasm32_refusal_pinned_in_ci_workflow_per_ds_r6_3_closure`.

---

## 5. IVM Algorithm B maturity

### 5.1 Drift-detector + non-canonical-view generalization — **CLOSED at G15-A / G15-B / W9-T1**

**Closure shape (Phase-3 G15-A + G15-B + R5 wave-9 W9-T1):**
- (a) **Algorithm B drift-detector** — `crates/benten-ivm/tests/algorithm_b_drift_detector.rs` ships 5 proptest pins driving the merged G15-A `Algorithm::register` surface end-to-end (1 000 cases each, ~25s wallclock total under MSRV 1.95): `prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern` (headline drift detector — incremental vs from-scratch parity), `prop_budget_trip_state_propagation_consistent`, `prop_rebuild_after_stale_returns_view_to_fresh`, `prop_drift_detector_observes_label_pattern_extension`, `prop_drift_detector_reports_one_path_errors_other_succeeds`. The structured-diff helper at `crates/benten-ivm/tests/common.rs::structured_diff` reports drift with row counts + canonical-bytes comparison.
- (b) **Non-canonical view generalization** — `crates/benten-ivm/src/algorithm_b.rs::GenericKernel` is the load-bearing generic kernel for non-canonical view ids per `D-PHASE-3-28 RESOLVED`. `Algorithm::register(view_id, label_pattern, projection)` instantiates `GenericKernel` for non-canonical ids; canonical ids route through the matching hand-written inner kernel via `for_id`. The Phase-2b `ContentListingView` silent-fallback is RETIRED — non-canonical user-defined ids no longer coerce to label-equality semantics for AnchorPrefix patterns.
- W9-T1 hardening: `Algorithm::register_with_budget` lifts the per-update budget knob into the registration surface (closes `5.1-followup-e`); the kernel-side AnchorPrefix-on-canonical-id guard (closes `5.1-followup-c`) fails loud at registration with `AlgorithmError::CanonicalIdAnchorPrefixRefused`, mirrored at the engine boundary as `EngineError::ViewLabelMismatch`.

**Cross-references:**
- `crates/benten-ivm/src/algorithm_b.rs::AlgorithmBView` (Strategy::B wrapper handling either inner kernel)
- `crates/benten-ivm/src/algorithm_b.rs::GenericKernel` (non-canonical inner kernel)
- `crates/benten-ivm/tests/algorithm_b_drift_detector.rs` (5 proptest pins)
- `crates/benten-ivm/tests/common.rs::structured_diff` (drift-reporting helper)
- `crates/benten-engine/src/engine_views.rs::register_user_view` (engine-side dispatch through `Algorithm::register`)

**Residual followups:** `5.1-followup-a` (rebuild event-replay seam — bundles with §1.2 SnapshotBlobBackend), `5.1-followup-b` (edge-traversal-keyed selector type), `5.1-followup-d` (canonical-fast-path 1.20x perf-gate rework). All carry their own named-destination dispositions per HARD RULE rule-12.

#### 5.1-followup-a GenericKernel rebuild without event-replay seam (g15a-mr-major-3 carry)

**G15-A state:** `crates/benten-ivm/src/algorithm_b.rs::GenericKernel::rebuild()` clears `entries` + resets the stale flag fresh (lines ~326-335). The docstring acknowledges the gap: `"Phase-3+ event-replay rebuild wires the snapshot store; until then `rebuild` clears + resets fresh so a previously stale-tripped view is observably re-armed."` Consequence: when a user view trips stale mid-stream (BudgetExceeded / external `mark_stale`), `rebuild()` produces an EMPTY view with no rebuild-from-source path. Subsequent `read()` returns `Ok(ViewResult::Cids(vec![]))` — observable as "view exists but is empty," indistinguishable from "view exists + has no matching rows." The 5 hand-written canonical kernels share the same gap by Phase-2b precedent, but they materialize fixed system-zone surfaces whose rebuild semantics are bounded; the generic kernel exposes the gap on user-defined views, which is exactly the surface where rebuild matters most.

**Phase 3 target (lands at G15-B / G15-C with the event-replay surface):** wire `GenericKernel::rebuild()` to the snapshot-store / event-replay seam so a stale-tripped view re-materializes from the durable backend. Two coupling points:
- The SnapshotBlobBackend hop named in §1.2 produces the rewind seam.
- The drift-detector proptest harness (§5.1 (a)) is the verification surface for `rebuild() ≡ from-scratch` parity post-replay.

**Why Phase 3:** The Phase-2b precedent (the 5 hand-written kernels' rebuild = clear + flip fresh) was acceptable when their rebuild scope was bounded by system-zone fixity; the generic-kernel surface lifts that bound. This is the named destination per HARD RULE rule-12 disposition (b) BELONGS-ELSEWHERE-SPECIFICALLY for `g15a-mr-major-3`.

**Touch size:** ~50-100 LOC (rebuild seam) + ~100-150 LOC tests; bundles cleanly with §5.1 (a) (drift-detector exercises the rebuild equivalence).

#### 5.1-followup-b Edge-traversal-keyed user views (g15a-mr-minor-6 carry)

**G15-A state:** `crates/benten-ivm/src/algorithm_b.rs::GenericKernel::update`'s `ChangeKind::EdgeCreated | ChangeKind::EdgeDeleted` arm silently drops edge events (lines ~307-308). Comment: `"Edge events do not affect Node-keyed views."` Correct for the Phase-3 G15-A scope (`(view_id, label_pattern, projection)` triples are Node-label-keyed); future user views needing edge-traversal-keyed semantics (e.g. `"all posts authored by an actor"`, `"all messages in a thread"`) cannot be built on the generic kernel as it stands. Consistent with Phase-1 hand-written views (also Node-label-keyed), but worth a named destination so the constraint surfaces when downstream user views need it.

**Phase 3 target:** introduce a sibling selector type (working name: `EdgeKeyedSelector`, materializing as a `LabelPattern` extension or a parallel `Selector::Edge { from_label, edge_label, to_label }` shape — design left to the wave's plan-pass) that consumes `ChangeKind::EdgeCreated` / `EdgeDeleted` events. Compose with §5.1's generic-kernel core so user views can declare a Node-keyed OR edge-traversal-keyed input pattern at registration. Bundles with §5.1 + §5.2 in the same Phase-3 IVM wave; the edge-keyed lift is a third axis alongside `LabelPattern::Exact` + `LabelPattern::AnchorPrefix`.

**Why Phase 3:** the edge-traversal extension shares the same surface-completeness §5.1 needs — a registration shape rich enough to express edge-keyed selection requires the generalization itself to exist. Named destination per HARD RULE rule-12 disposition (b) for `g15a-mr-minor-6`.

**Touch size:** ~150-300 LOC (selector type + GenericKernel edge-event arm) + ~100-200 LOC tests; bundles with §5.1.

#### 5.1-followup-c Tighten canonical-id-vs-AnchorPrefix fail-loud guard — **CLOSED at W9-T1**

**Closure shape (Phase-3 R5 wave-9 W9-T1):** `crates/benten-ivm/src/algorithm_b.rs::Algorithm::register_inner` now fires `AlgorithmError::CanonicalIdAnchorPrefixRefused` BEFORE the existing `ViewLabelMismatch` guard whenever `(canonical_id, AnchorPrefix(_))` is registered — regardless of whether the supplied prefix would match the canonical hardcoded label. The doc-vs-code-strength gap (`AnchorPrefix("")` silently matched-everything) is closed: AnchorPrefix is canonical-id-incompatible at the kernel boundary AND at the engine boundary (`crates/benten-engine/src/engine_views.rs::register_user_view` mirrors the kernel-side guard, surfacing `EngineError::ViewLabelMismatch` with `expected_label = canonical hardcoded label` + `got_label = "AnchorPrefix(<prefix>)"`). Catalog code `E_VIEW_LABEL_MISMATCH` is reused (a canonical id requiring an Exact label IS a label mismatch when the supplied pattern is a prefix selector).

**Cross-references:**
- `crates/benten-ivm/src/algorithm_b.rs::AlgorithmError::CanonicalIdAnchorPrefixRefused` (new variant, W9-T1)
- `crates/benten-ivm/src/algorithm_b.rs::Algorithm::register_inner` (guard #1 fires on AnchorPrefix discriminator BEFORE label-match check)
- `crates/benten-ivm/src/algorithm_b.rs::tests::register_canonical_view_with_anchor_prefix_refused_even_when_prefix_matches` (kernel-level pin)
- `crates/benten-engine/tests/user_view_canonical_id_anchor_prefix_refused.rs` (4 engine-level end-to-end pins covering all 5 canonical ids — empty prefix, non-empty prefix, content_listing edge case, non-canonical sanity).

#### 5.1-followup-d Canonical-fast-path perf-gate rework — release-profile-gated or criterion-companion-test (PR #121 dev-profile-flake carry)

**State at PR #121 (2026-05-06):** `crates/benten-ivm/tests/algorithm_b_general.rs::algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline` is the load-bearing canonical-fast-path-not-collapsed gate. The original 1.50x ceiling tripped on slow CI runners under dev-profile cargo test (PR #116 macos-arm64@stable + PR #120 macos-arm64@1.95.0 both observed 1.4-1.7x ratios); PR #121 first attempted to silence with `#[ignore]` (BLOCKED at mini-review per pim-2 §3.6b — claimed criterion bench was the load-bearing gate, but the criterion bench at `crates/benten-ivm/benches/algorithm_b_canonical.rs` only produces measurement-only estimates with NO firing 1.20x assertion + NO CI lane that reads them). PR #121 was reworked to keep the in-test gate firing but loosen the ceiling to 2.00x — preserves "canonical fast-path has not collapsed" protection (3x+ regressions still trip) while surviving slow-CI noise.

**Phase 3 target:** re-tighten the gate to the 1.20x release-profile target via either (a) wiring a criterion-companion test that reads `target/criterion/.../estimates.json` + asserts ratio ≤ 1.20 (the pre-`g15a-mr-major-2` shape, but with FAIL-on-missing-estimates instead of silent-Ok) plus a CI lane that runs `cargo bench` so the estimates exist; OR (b) gating the in-test 1.20x assertion behind `#[cfg(not(debug_assertions))]` so dev-profile runs skip but release-profile runs (which CI gains via `cargo test --release` lane) enforce. Option (a) bundles with §5.1 (a) drift-detector since both want a release-profile bench surface. Option (b) is simpler but requires a separate CI lane.

**Why Phase 3:** the dev-profile vs release-profile divergence is intrinsic to `cargo test` running unoptimized; tightening to 1.20x in dev-profile is unsound. The 2.00x interim ceiling is meaningful protection (catches dispatch-router collapse, capability-table-bypass, kernel-instantiation regression) but not the headline 20% bound the canonical-fast-path commitment cites. The rework lifts the gate back to the documented 1.20x without dev-profile flake.

**Touch size:** option (a) ~50-100 LOC test + ~10-20 LOC CI workflow change; option (b) ~5-10 LOC test attribute + ~10-20 LOC CI workflow change. Bundles with §5.1 / §5.1-followup-a (drift-detector + rebuild) since the same harness consumes the release-profile measurement.

**Cross-references:**
- `crates/benten-ivm/tests/algorithm_b_general.rs::algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline` (current 2.00x interim site)
- `crates/benten-ivm/benches/algorithm_b_canonical.rs` (criterion bench surface for the companion-test rework)
- `.github/workflows/bench-threshold-drift.yml` (existing bench lane, currently `informational` — would gain enforcement via either rework path)
- PR #121 mini-review (`r5-pr121-mini-review.json`) carrying the BLOCKER → fix-pass narrative.

#### 5.1-followup-e Budget knob on `Algorithm::register` for user-view per-update budgets — **CLOSED at W9-T1**

**Closure shape (Phase-3 R5 wave-9 W9-T1):** `crates/benten-ivm/src/algorithm_b.rs::Algorithm::register_with_budget(view_id, label_pattern, projection, budget)` is the new budget-aware registration surface. `GenericKernel` gains a `BudgetTracker` field (default constructor unchanged: `u64::MAX` unbounded sentinel); the per-event `update` path consumes one budget unit per matching write (Created/Updated whose first label matches OR Deleted whose CID was previously admitted). Canonical-id registrations route through the new sibling `AlgorithmBView::for_id_with_budget`, which forwards the budget into the matching canonical kernel's `with_budget_for_testing` constructor (the `_for_testing` suffix is preserved on the inner constructors as the Phase-1 source-of-truth shape; the user-facing path is `register_with_budget`). `is_stale` returns the OR of (kernel-level mark_stale, budget-tracker-level BudgetExceeded); `rebuild` restores the budget cap + clears stale across both sources.

**Residual:** `content_listing` with a non-`"post"` label + budget falls back to unbounded (the Phase-1 `ContentListingView::with_budget_for_testing` constructor hard-codes label `"post"`). Lifting the canonical constructor to `(label, budget)` requires touching the 5 hand-written kernels' construction shape; intentionally NOT taken in W9-T1 (the W9-T1 close is the load-bearing `Algorithm::register_with_budget` surface lift; the inner-kernel constructor lift is a smaller follow-on bundling with §5.1-followup-a's per-view state-machine work).

**Cross-references:**
- `crates/benten-ivm/src/algorithm_b.rs::Algorithm::register_with_budget` (new surface)
- `crates/benten-ivm/src/algorithm_b.rs::AlgorithmBView::for_id_with_budget` (canonical-routing sibling)
- `crates/benten-ivm/src/algorithm_b.rs::GenericKernel::with_budget` (kernel-level constructor)
- `crates/benten-ivm/src/algorithm_b.rs::tests::register_with_budget_*` (4 unit pins covering: cap trip + canonical forwarding + guard inheritance + u64::MAX = unbounded shape).

### 5.2 AnchorPrefix selector lift in user-view registration — **CLOSED at G15-A**

**Closure shape (Phase-3 G15-A):** `crates/benten-engine/src/engine_views.rs::register_user_view` no longer silent-coerces `UserViewInputPattern::AnchorPrefix` → label-equality. The Phase-3 G15-A landing routes the variant into `benten_ivm::LabelPattern::AnchorPrefix(prefix)`, which `GenericKernel::first_label_matches` consumes via the genuine `starts_with` semantic (`crates/benten-ivm/src/algorithm_b.rs::LabelPattern::matches`). The persisted view-definition Node carries both `input_pattern_label` (the prefix string) AND a sibling `input_pattern_kind` discriminator (`"label"` vs `"anchor_prefix"`) so future readers can disambiguate without re-parsing the pattern surface.

**W9-T1 hardening:** canonical view ids + AnchorPrefix is REFUSED at the engine boundary (closes `5.1-followup-c`); the AnchorPrefix selector is therefore strictly a non-canonical-id surface, matching its semantic intent (prefix selection is incompatible with canonical kernels' Exact-label hardcoded shape).

**Cross-references:**
- `crates/benten-engine/src/outcome.rs::UserViewInputPattern::AnchorPrefix`
- `crates/benten-engine/src/engine_views.rs::register_user_view` (genuine prefix routing post-G15-A)
- `crates/benten-ivm/src/algorithm_b.rs::LabelPattern::AnchorPrefix` (kernel-side selector)
- `crates/benten-ivm/src/algorithm_b.rs::tests::generic_kernel_anchor_prefix_pattern_drives_correct_subset` (kernel-level prefix semantic pin)
- `crates/benten-engine/tests/user_view_canonical_id_anchor_prefix_refused.rs::register_user_view_non_canonical_id_with_anchor_prefix_succeeds` (engine-level non-canonical AnchorPrefix sanity).

---

## 6. SANDBOX runtime maturity

### 6.0 D10 read-only-snapshot enforcement at SANDBOX kv:write boundary — OOS per CLAUDE.md baked-in #16 (conditional forward-pointer)

**Disposition (retensed 2026-05-08):** OUT-OF-SCOPE per CLAUDE.md baked-in #16. SANDBOX modules do NOT duplicate other primitives' capabilities — `kv:write` / `kv:delete` / edge-mutating host-fns are explicitly NOT engine concerns; they would be parallel-write-pathways that bypass the WRITE primitive's capability gating + Inv-13 firing matrix + IVM materialization seam. Future agent proposals to add `kv:write` / `kv:delete` must be rejected with reference to baked-in #16 unless the proposal also re-opens commitment #1 (12 primitives are irreducible) — those decisions move together.

**Phase 2b state (historical):** R6-R3 r6-r3-arch-2 forward-pointer. Phase 2b's SANDBOX host-fn surface is read-only at the storage layer: `crates/benten-eval/src/sandbox/host_fns.rs::default_host_fns` ships ONLY `time`, `log`, `kv:read` (+ `random` per Phase-3 G17-A2 closure of Compromise #16). There is no `kv:write` host-fn; therefore a Phase-2b/3 SANDBOX module CANNOT bypass D10 read-only-snapshot contract via host_fns — there is no surface to bypass. PR #68 wired `is_read_only_snapshot()` enforcement at `crates/benten-engine/src/primitive_host.rs::put_node`; R6-R3 r6-r3-arch-1 fix-pass extended the same enforcement to `delete_node` via the shared `check_not_read_only_snapshot(op_name)` helper. Both checks fire on the dispatch-through-handler path that `engine.call(handler, ':...', ...)` exercises.

**Conditional forward-pointer (preserved):** if a future phase ships `kv:write` / `kv:delete` / edge-mutating host-fns (which would require re-opening baked-in #16 + commitment #1), the read-only-snapshot enforcement MUST live AT the host-fn dispatch boundary in addition to `PrimitiveHost::put_node` / `delete_node`. The architecturally-cheapest closure is to either (a) route every storage-mutating host-fn through `PrimitiveHost::put_node` / `delete_node` so the existing helper fires, OR (b) have each host-fn closure independently invoke `Engine::is_read_only_snapshot()` before the backend call. **No work to do today.**

### 6.1 ESC-16 fingerprint-collapse complete defense — CLOSED at Phase-3 wave-5c

**Phase 2b state (historical):** Wave-8b wired the SANDBOX runtime (Store + Linker + Instance + fuel/memory/wallclock + epoch ticker). 9 of 16 ESC vectors fully fire typed errors. ESC-16 (wallclock fingerprint collapse) had a `.wat` fixture committed but R6 wasmtime-sandbox-auditor (`r6-wsa-3`) flagged that the test bypassed it with an inline shape that didn't exercise the fingerprint-collapse property end-to-end. Wave-8j-cleanup didn't address this.

**Phase-3 closure (wave-5c, ratified 2026-05-07):** ESC-16 fingerprint-collapse is now FULLY WIRED end-to-end. The closure narrative + 6-task implementation + production-call-site audit live in §6.1-followup below. The end-to-end pin at `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs::esc_16_runtime_arm_fires_after_threshold_time_host_fn_calls` drives `Sandbox::execute` through real `wasmtime::Module` + `Instance::call` and asserts observable typed-error firing per pim-2 §3.6b. The below-threshold pin (`esc_16_silent_below_threshold_two_time_calls_pass`) proves the defense is silent on legitimate use. SECURITY-POSTURE.md ESC-16 row updated to "Fully wired end-to-end" at Phase-3 wave-5c. r1-wsa-4 MAJOR closed end-to-end.

#### 6.1-followup ESC runtime-arm wiring — **CLOSED at wave-5c** (recall of G17-A1's claimed BLOCKER closure)

**State at G17-A1 wave-5b (PR #117):** the SCAFFOLDING for ESC-7 / ESC-13 / ESC-16 (and ESC-9) defenses landed — `EscVector` enum + `SandboxError::EscapeAttempt` typed variant + `run_esc7_check` / `run_esc13_check` / `run_esc16_check` defense entry points + `EscDefenseState` per-call carrier + cfg-gated `crates/benten-eval/src/sandbox/testing_helpers.rs` SURFACE. The `Trap::StackOverflow` → `SandboxError::StackOverflow` arm at `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` was genuinely production-wired (closed r1-wsa-7).

**SHAPE-not-SUBSTANCE finding (PR #117 mini-review 2026-05-06, 4th-of-4 wave recurrence extending wave-5a's G14-C / G15-A / G18-A pattern):** the ESC-7/13/16 defense entry points had ZERO production call sites. `EscDefenseState` was constructed only in tests. `EscapeAttemptMarker` was constructed only inside `#[cfg(test)]`. `SandboxStoreData` had no `esc_defense_state` field. The `time` host-fn never called `fingerprint::record_wallclock_write`. Integration tests under `crates/benten-eval/tests/sandbox_esc_*.rs` audited the helpers against synthetic state — they would still have passed if the entire production trampoline were stripped. Per pim-2 §3.6b: NOT load-bearing closure for r1-wsa-1 BLOCKER (ESC-7 + ESC-13) or r1-wsa-4 MAJOR (ESC-16). r1-wsa-3 (ESC-9 cap-recheck) was the same shape — see §6.3 above.

**Wave-5c closure (this entry):** all 6 wave-5c tasks landed end-to-end. Production call sites verifiable via `grep 'EscapeAttemptMarker\|run_all_checks' crates/benten-eval/src/` — every match outside `#[cfg(test)]` is now a production code path (host-fn trampoline + `Store::call` panic-catcher + boundary check). The runtime arms are wired into `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check` + the engine override at `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (constructs the engine-backed `LiveCapCheck` callable as a closure capturing `Arc<Mutex<HashSet<Cid>>>` cloned from the engine's revoked-actors set + the dispatching actor CID). End-to-end test pins drive `Sandbox::execute` through real `wasmtime::Module` + `Instance::call` and assert observable typed-error firing per pim-2 §3.6b — `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs` (7 tests covering ESC-7 / ESC-9 / ESC-13 / ESC-16 + recovery-path + below-threshold + production-equivalent legitimate-call). r1-wsa-1 BLOCKER + r1-wsa-3 MAJOR + r1-wsa-4 MAJOR all CLOSED end-to-end at wave-5c.

**Wave-5c implementation (CLOSED):**

1. **`SandboxStoreData` field add — DONE.** `esc_defense_state: EscDefenseState` + `tainted_addresses: Vec<WallclockTaintedAddress>` + `live_cap_check: Option<LiveCapCheck>` fields landed in `crates/benten-eval/src/primitives/sandbox.rs::SandboxStoreData`. Threaded through `SandboxStoreData::new` constructor + every host-fn trampoline `Caller::data_mut()` site.

2. **`time` host-fn fingerprint side-table population — DONE.** Each `time` invocation (a) calls `record_wallclock_write` recording a fresh per-call address (synthetic 8-byte stride starting at 0x1000), (b) appends to `tainted_addresses`, and (c) calls `read_collapse_state` on the same address (semantically: the host writes the wallclock value AND the guest reads it on receive). The 3rd `time` call within a single SANDBOX dispatch trips `FINGERPRINT_COLLAPSE_THRESHOLD = 3`.

3. **Host-fn boundary `run_all_checks` invocation — DONE.** Each `time` / `log` / `kv:read` trampoline invokes `run_all_checks(&caller.data().esc_defense_state)` BEFORE returning to guest wasm. On `Err(SandboxError::EscapeAttempt(_))`, the trampoline returns `wasmtime::Error::from(EscapeAttemptMarker { vector, reason })` — `trap_to_typed::map_call_error` unwraps to the typed `SandboxError::EscapeAttempt` variant on the host side. *(`random` host-fn is OUT-OF-SCOPE for wave-5c per the brief — does not exist on this branch; lives on PR #118; orchestrator can rebase post-#118 if needed.)*

4. **Panic-catcher around fuel-meter callback (ESC-13) — DONE.** A `std::panic::catch_unwind(AssertUnwindSafe(...))` wrapper around `func.call` in `execute_with_live_cap_check` catches host-side panics (fuel-meter callback OR any panicking host-fn closure); the wrapper sets `esc_defense_state.fuel_meter_callback_trapped = true` + returns the typed `SandboxError::EscapeAttempt(Esc13StorePoison)` directly. The (potentially-poisoned) `Store` is dropped on return per D3-RESOLVED per-call lifecycle; the next SANDBOX call gets a fresh `Store` (recovery-path pin: `esc_13_recovery_path_next_call_fresh_store_no_poison_leak`).

5. **`live_cap_check` callback through-thread (ESC-9) — DONE.** New `pub type LiveCapCheck = Arc<dyn Fn(&str) -> bool + Send + Sync>` in `crates/benten-eval/src/primitives/sandbox.rs`. The trampoline `cap_check` helper consults the callback for `PerCall` cadence (cadence (a) per r4-r1-wsa-4 — once per host-fn entry, not per loop iteration). Engine override at `primitive_host.rs::execute_sandbox` constructs the engine-backed callable; `EngineInner::revoked_actors_for_subscribe` was promoted from `Mutex` to `Arc<Mutex<...>>` so the callback can clone the Arc + observe live revocations. New entry point `execute_with_live_cap_check(...)` accepts `Option<LiveCapCheck>`; legacy `execute(...)` delegates with `None` (Phase-2b grant-caps-snapshot fallback). Bundles with §6.3 D18 live-cap-check callback wire-through (same surface; §6.3 also CLOSED at wave-5c).

6. **End-to-end test pins — DONE.** `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs` carries 7 active end-to-end tests (NOT `#[ignore]`'d — load-bearing closure per pim-2 §3.6b). Each test drives `Sandbox::execute` through a real production trampoline + asserts the typed `SandboxError::EscapeAttempt` (or `HostFnDenied` for ESC-9) routes through `map_call_error` end-to-end. The pre-existing SHAPE pins under `tests/sandbox_esc_{7,13,16}.rs` + `tests/sandbox_capability_check_per_call_after_revoke.rs` + `tests/sandbox_esc_9.rs` retained (they document the SCAFFOLDING + SURFACE invariants); the e2e pins supersede them as the load-bearing closure tests for r1-wsa-1 / r1-wsa-3 / r1-wsa-4. The pre-existing test `sandbox_escape_attempts_denied.rs::sandbox_escape_wallclock_fingerprint_via_time_coarsened` updated to assert the new defense fires (1000-call loop trips ESC-16 at the 3rd call); narrative documents the wave-5c behavior change.

**Touch size (actual wave-5c, measured against PR #117 base):** see PR diff (~+800 LOC implementer + tests + narrative; well within the planned 400-700 + 150-300 + 30-50 envelope).

**Cross-references:**
- PR #117 G17-A1 mini-review: `.addl/phase-3/r5-w5b-g17-a1-mini-review.json`
- §6.3 D18 live-cap-check callback wire-through (CLOSED at wave-5c via the same `LiveCapCheck` callback infrastructure)
- §6.4 Dedicated `E_SANDBOX_STACK_OVERFLOW` (closed at G17-A1 — runtime-wired piece)
- pim-N codification candidate: SHAPE-not-SUBSTANCE 4th-of-4 wave recurrence (G14-C / G15-A / G18-A / G17-A1) hits the 3+-recurrence threshold per `feedback_3_plus_recurrence_deep_sweep`. Recommend codifying as pim-18 §3.6e at next dispatch-conventions amendment: implementer briefs MUST include explicit "production call site enumeration" pre-flight item. Wave-5c PR followed this discipline pre-emptively (see the §6.1-followup brief). The wave-5c PR's pre-merge grep audit (`grep -rn 'EscapeAttemptMarker\b\|run_all_checks' crates/benten-eval/src/ | grep -v 'cfg(test)'` returns matches in `crates/benten-eval/src/primitives/sandbox.rs` host-fn trampolines, not test contexts) is the load-bearing pin against the SHAPE-not-SUBSTANCE shape.

### 6.4 Dedicated `E_SANDBOX_STACK_OVERFLOW` typed variant — CLOSED at Phase-3 G17-A1 wave-5b

**Phase 2b state (historical):** R6 wasmtime-sandbox-auditor `r6-wsa-8` flagged: `Trap::StackOverflow` from wasmtime currently folds into `E_SANDBOX_MODULE_INVALID` reason string in `crates/benten-eval/src/sandbox/trap_to_typed.rs`. R6-FP Group 1 (PR #62) narrowed the reason string within the existing variant (interim disposition); the agent offered to land a dedicated `E_SANDBOX_STACK_OVERFLOW` typed variant as a follow-up, estimated as ~20-site cascade across drift detector + catalog tables + narrative docs (~50-150 LOC).

**Phase-3 closure (G17-A1 wave-5b):** dedicated `ErrorCode::SandboxStackOverflow` variant + `E_SANDBOX_STACK_OVERFLOW` catalog code minted in `crates/benten-errors/src/lib.rs` + atomic 4-surface update per dispatch-conventions §3.5g (lib.rs enum + as_str arm + `stable_shape.rs::ALL_CATALOG_VARIANTS` + `docs/ERROR-CATALOG.md` E_SANDBOX_STACK_OVERFLOW section + `packages/engine/src/errors.generated.ts::ESandboxStackOverflow` regenerated). Routing arm at `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error`: `wasmtime::Trap::StackOverflow` → `SandboxError::StackOverflow { max_wasm_stack }` typed variant in `crates/benten-eval/src/primitives/sandbox.rs`. `bindings/napi/src/error_envelope.rs::engine_err_envelope_json` surfaces `err.code()` so the typed `E_SANDBOX_STACK_OVERFLOW` propagates through the JSON envelope without per-variant special-case. SECURITY-POSTURE.md ESC-5 row updated to cite the dedicated variant. End-to-end + cascade-completeness test pins at `crates/benten-eval/tests/sandbox_stack_overflow.rs::sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant` + `..._traps_via_dedicated_variant`. r1-wsa-7 BLOCKER + r6-wsa-8 BELONGS-NAMED-NOW deferral both retired.

### 6.3 D18 live-cap-check callback wire-through (ESC-9 cap-revoke mid-call) — **CLOSED at wave-5c**

**Phase 2b state:** R6 wasmtime-sandbox-auditor `r6-wsa-2` flagged: `live_cap_check` callback was dead surface; D18 PerCall cap-recheck functionally degraded to PerBoundary in production. ESC-9 (cap-revoke mid-call TOCTOU between cap-grant and cap-use) was undefended at runtime — only the cap-snapshot at SANDBOX entry was consulted. R6-FP Group 1 (PR #62) opted for BELONGS-NAMED-NOW disposition with an in-code `TODO(PHASE-3-BUNDLE-D18-live-cap-check)` marker because the structural lift is >100 LOC + bundles cleanly with Phase-3 grant-store work.

**Wave-5c closure (this entry):** the `live_cap_check` callback is now wired end-to-end via the new `pub type LiveCapCheck = Arc<dyn Fn(&str) -> bool + Send + Sync>` in `crates/benten-eval/src/primitives/sandbox.rs`:
- (a) **DONE.** `Arc<Mutex<HashSet<Cid>>>` cloned from `EngineInner::revoked_actors_for_subscribe` (promoted from raw `Mutex<HashSet<Cid>>` to `Arc<Mutex<HashSet<Cid>>>` at wave-5c) is captured by the production callback. The callback also captures the dispatching actor CID + a snapshot of `grant_caps` for the cap-string check.
- (b) **DONE.** The trampoline `cap_check` helper in `crates/benten-eval/src/primitives/sandbox.rs::cap_check` consults `data.live_cap_check` first when present (PerCall cadence), falling back to `data.live_caps` snapshot only when the callback is `None` (legacy callsites).
- (c) **DONE.** `crates/benten-eval/tests/sandbox_capability_check_per_call_after_revoke.rs::sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent` already un-ignored at G17-A1 wave-5b; supplementary end-to-end pin at `crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs::esc_9_runtime_arm_fires_via_live_cap_check_revoke_mid_call` drives `Sandbox::execute` with a live-callback that flips revocation between the first and second `kv_read` invocations + asserts `SandboxError::HostFnDenied` fires on call #2.
- (d) **DONE.** SECURITY-POSTURE.md ESC-9 entry updated from "Wired-defense + simulation pin green at the §7.3.A.7 helper SURFACE level" → "Fully wired (end-to-end pin against `Sandbox::execute` driving `kv_read` twice with mid-call revoke)".

**Touch size (actual):** ~50 LOC engine-side override at `crates/benten-engine/src/primitive_host.rs::execute_sandbox` constructing the callback; ~30 LOC in `EngineInner` for the `Arc` promotion + `revoked_actors_arc()` accessor; ~120 LOC end-to-end test pin. Within the ~80-150 LOC plan.

### 6.2 D26 .wasm-bytes-shipping per fixture — CLOSED at Phase-3 G17-B (wave-5b)

**Phase 2b state:** ESC-1..16 fixtures live as `.wat` source compiled at test time (`wat::parse_str(...)`). D26 design intent calls for shipping pre-built `.wasm` bytes per fixture so cross-platform determinism + canonical CID pinning can apply. R6 wasmtime-sandbox-auditor (`r6-wsa-5`) flagged this gap; wave-8b ran out of budget before completing the tooling.

**Phase 3 closure (G17-B wave-5b):**
1. **`tools/bench-wat-rebake/`** regenerator binary — compiles every `.wat` under `crates/benten-eval/tests/fixtures/sandbox/**/*.wat` to its committed `.wasm` sibling using the workspace-locked exact-version `wat` crate (`=1.248.0` per `[workspace.dependencies] wat`). Invoked via `cargo bench-wat-rebake` alias in `.cargo/config.toml`. `--check` mode reports drift without writing.
2. **`crates/benten-eval/build.rs`** — emits `cargo:rerun-if-changed=` directives so a `.wat` source edit retriggers `tests/fixture_wasm_hashes_stable` + `tests/d26_wasm_present` drift detectors.
3. **`crates/benten-eval/src/test_fixtures.rs`** — runtime fixture loader (`load_fixture(stem)`) prefers committed `.wasm` if present + valid; falls back to `wat::parse_file` only when `.wasm` absent (fresh-checkout case). Both branches compile via the same workspace-locked `wat` crate, so the bytes round-trip is closed.
4. **r4-r1-wsa-9 single-tool recalibration:** workspace pins `wat = "=1.248.0"` exact-version (no `^`/`~`/bare matchers); `wasm-tools` REJECTED as a parallel dep. The legacy `scripts/build_wasm.sh` (which invoked the host `wabt` binary) is superseded — its output bytes can drift from the `wat` crate's output even on semantically-equivalent modules.
5. **`tests/fixture_wasm_hashes_stable::PINNED_FIXTURES`** updated for `depth_nest_2`, `depth_nest_3_negative`, `output_overflow_2048` (the three fixtures whose canonical bytes shifted from `wabt`'s output to `wat` crate's output during the recalibration); 14 new committed `.wasm` fixtures landed under `escape/`.

**Cross-platform CID stability** is now defended by three layers: (a) workspace exact-version pin; (b) committed `.wasm` bytes (loader prefers these); (c) per-fixture BLAKE3 drift detector. The new AArch64 SANDBOX runtime CI cell (§6.7) verifies the same fixture CIDs resolve identically on Apple Silicon.

**Touch size (actual):** ~600 LOC across tooling crate + build.rs + loader module + workflow + test pins (within G17-B plan ceiling 200-400 LOC + 50% reserve = 600).

### 6.6 TS-side SANDBOX named-manifest resolution + module-bytes registration API — CLOSED IN G17-C WAVE-5b (3 deliverables landed; 24th p/c drift acceptance criterion landed)

**Phase-3 G17-C wave-5b RESOLUTION:** All three coupled deliverables landed at G17-C wave-5b (commit `dc0284c` PR #119) plus the 24th p/c drift acceptance criterion (camelCase/snake_case translation):
- **Deliverable 1 (registration-time SANDBOX manifest validation) LANDED** at `crates/benten-engine/src/engine.rs::Engine::validate_sandbox_manifest_names` (cfg-gated NOT-wasm32; called from `register_subgraph` after the eval-side validate). The function resolves both shapes (`manifest` property OR colon-joined `module: "<manifest>:<entry>"`) against `manifest_registry_known_names()` and surfaces `EngineError::SandboxManifestUnknown` on miss. `manifest_registry()` overlay was extended to dual-key entries by both `entry.name` AND `<manifest_name>:<entry.name>` so dispatch + registration agree on lookup shape.
- **Deliverable 2 (TS-side `engine.registerModuleBytes` napi method) LANDED** at `bindings/napi/src/lib.rs::Engine::register_module_bytes` (`#[napi(js_name = "registerModuleBytes")]`). Wires through to `InnerEngine::register_module_bytes(cid, bytes)`. The TS surface lives at `packages/engine/src/engine.ts::Engine::registerModuleBytes`. Sibling napi surfaces `installModule` / `uninstallModule` / `computeManifestCid` were wired at the same wave-5b cluster.
- **Deliverable 3 (sandbox.test.ts post-`registerModuleBytes` greens) LANDED** at `packages/engine/test/sandbox.test.ts` and `packages/engine/test/install_module.test.ts`. The 3 pre-G17-C `.skip`'d tests are now `it(...)` running through production flow (DSL → installModule → registerSubgraph → call). The un-skip is structurally pinned by `packages/engine/test/sandbox_handler_args.test.ts::"sandbox.test.ts existing .skip'd tests re-pinned to production-flow shape"`.
- **24th p/c drift acceptance criterion LANDED** at `packages/engine/src/dsl.ts::translateSandboxArgs` (mirrors PR #76 `translateWaitArgs` precedent). Translates `wallclockMs` → `wallclock_ms` and `outputLimitBytes` → `output_limit` (DROPS `Bytes` per r4-r1-wsa-1 BLOCKER recalibration verifying canonical `op.properties.get("output_limit")` reader at `crates/benten-engine/src/primitive_host.rs::execute_sandbox`). Applied at both `SubgraphBuilder.sandbox()` + `CaseBuilder.sandbox()` call sites. End-to-end pin at `crates/benten-eval/tests/sandbox_handler_args.rs::sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`. TS-side meta-pin at `packages/engine/test/sandbox_handler_args.test.ts` (4 tests, including one structural pin against `.skip` regression).

**Phase 2b state (preserved for narrative):** The Rust-side named-manifest registry (`benten_eval::sandbox::ManifestRegistry` + `Engine::manifest_registry()` projection) keyed CapBundle entries by `entry.name` (e.g. `"identity"`), NOT by the colon-joined `"<manifestName>:<entryName>"` shape the TS DSL surface advertised. Wave-8h wired the registry projection but the resolution-from-DSL-shape half was missing on the TS bridge — `register_subgraph` did NOT validate at registration time that a SANDBOX node's `module: "<m>:<e>"` resolved to an installed manifest entry, AND there was NO TS-side `engine.registerModuleBytes(cid, bytes)` API to register the actual wasm bytes (Rust had `Engine::register_module_bytes`, napi-unexposed). Three TS vitest pins were authored RED-PHASE expecting this resolution + registration plumbing:

- `packages/engine/test/install_module.test.ts::"engine.uninstallModule(cid) clean release"` — expects `registerSubgraph` to REJECT with `E_SANDBOX_MANIFEST_UNKNOWN` after uninstall.
- `packages/engine/test/sandbox.test.ts::"compose SANDBOX inside a handler subgraph"` — expects `engine.call(...)` to return `result.ok=true` (which requires real wasm bytes registered + name-resolution at registration).
- `packages/engine/test/sandbox.test.ts::"E_INV_SANDBOX_OUTPUT fires on output > limit (D15 trap-loudly)"` — same shape as above; expects an actual wasmtime-driven oversize emission.

The vitest cluster fix-pass (PR linked from `.addl/phase-2b/r6-r2-fp-vitest-cluster-*`) converted these three pins to `.skip` with a destination-here named-NOW per HARD RULE (rule #12, foundational `feedback_no_defer_HARD_RULE`).

**Phase 3 deliverables (3 coupled, all LANDED at G17-C wave-5b):**
1. **Registration-time SANDBOX manifest validation LANDED.** `Engine::register_subgraph` walks the spec for SANDBOX nodes; for each, parses `module` as either `(a)` a bare base32 CID or `(b)` a `"<manifestName>:<entryName>"` lookup. Branch `(b)` is rejected with `ErrorCode::SandboxManifestUnknown` (catalog code `E_SANDBOX_MANIFEST_UNKNOWN`) when the name does not resolve through `installed_modules`. `manifest_registry()` was extended to also key entries by the colon-joined name so dispatch + register paths agree on the lookup shape. See `crates/benten-engine/src/engine.rs::Engine::validate_sandbox_manifest_names`.
2. **TS-side `engine.registerModuleBytes(cid, bytes)` napi method LANDED.** Wires through to `InnerEngine::register_module_bytes(cid, bytes)` so TS callers can ship a real wasm bytes payload. Pairs with §1.4 (Compromise #17 durable module-bytes registry) — the durable backing is what makes this useful end-to-end. See `bindings/napi/src/lib.rs::Engine::register_module_bytes` + `packages/engine/src/engine.ts::Engine::registerModuleBytes`.
3. **Sandbox.test.ts post-`registerModuleBytes` greens LANDED.** The three previously-`.skip`'d tests are now production-flow shape: install manifest → register module bytes → register subgraph → call → assert outcome. The fixture wasm bytes shipped per §6.2 (D26 `.wasm`-bytes-shipping per fixture, closed at G17-B wave-5b). Structural defense against `.skip` regression: `packages/engine/test/sandbox_handler_args.test.ts::"sandbox.test.ts existing .skip'd tests re-pinned to production-flow shape"`.

**Why Phase 3 (preserved for narrative):** All three deliverables composed with already-Phase-3-bundled work — §1.4 (durable module-bytes registry, the natural home for the `registerModuleBytes` API) + §6.2 (`.wasm`-bytes shipping for fixture distribution) + the named-manifest registry's eventual sync-replica replication shape. Landing the TS bridge standalone in Phase 2b would have required re-shaping when the durable backing arrived.

**Touch size (as authored):** ~150-300 LOC (engine-side validation walk + napi wiring + 3 test re-pins). Risk surface: low — additive surface; existing handlers without SANDBOX or with bare-CID `module` strings continued to work unchanged.

**Acceptance criterion (added 2026-05-03 R6-R5-narrow producer-consumer-deep-sweep `r6-r5-narrow-pcds-1` — 24th p/c drift instance) — LANDED at G17-C wave-5b:** the Phase-3 implementer wiring SANDBOX runtime resolved the camelCase/snake_case casing drift between the TS DSL surface and Rust eval-side property reads. Specifically: `packages/engine/src/types.ts` declares `wallclockMs: number` + `outputLimitBytes: number` — camelCase TS-idiomatic. The DSL writers at `packages/engine/src/dsl.ts::SubgraphBuilder::sandbox` + `packages/engine/src/dsl.ts::CaseBuilder::sandbox` previously spread `{ ...args }` raw, no translation. The Rust DSL Compiler test at `crates/benten-dsl-compiler/src/lib.rs::permuted_keys_yield_identical_canonical_bytes` writes `wallclock_ms` + `output_limit` (snake_case) in its fixture handler text. The eval-side reader at `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (the per-handler property override block reading `wallclock_ms` + `output_limit` from `op.properties`) reads snake_case. The drift was INERT pre-G17-C (DSL→runtime SANDBOX path structurally gated on §6.6 deliverable 1; defaults matched silently-dropped values per `SandboxConfig::default()`). With deliverable 1 LANDED, a `sandbox({ wallclockMs: 5000 })` per-handler tuning override would be silently ignored WITHOUT the translation. **Fix shape (LANDED):** mirrors the WAIT translation precedent (R6-R5-FP PR #76, dsl.ts::translateWaitArgs) — `translateSandboxArgs` camelCase→snake_case translates `wallclockMs` → `wallclock_ms` + `outputLimitBytes` → `output_limit` at the DSL spread sites, preserving the public `SandboxArgs` interface unchanged. Recalibration at R4-FP per `r4-r1-wsa-1` BLOCKER: canonical eval-side property is `output_limit` (DROPS `Bytes`), NOT `output_limit_bytes`. End-to-end test pins at `crates/benten-eval/tests/sandbox_handler_args.rs` (Rust observable end) + `packages/engine/test/sandbox_handler_args.test.ts` (TS-side meta-pin, 4 tests). Cross-references: `.addl/phase-2b/r6-r5-narrow-pcds.json` (origin); `crates/benten-eval/tests/sandbox_wallclock.rs` (existing `#[ignore]` retained or un-ignored per its own carry — see §7.3).

### 6.5 RedbSuspensionStore retention-window override — CLOSED at Phase-3 G17-A2 wave-5b

**Closure shape (Phase-3 G17-A2 wave-5b):** `RedbSuspensionStore` now overrides `is_retention_exhausted` per the Phase-3 target shape — durable per-subscriber metadata (`PersistedCursorMeta { registered_at_unix_secs, delivered_count }`) lives under the new `sm:<sub_cid>` redb prefix; `put_cursor` lazy-stamps `registered_at_unix_secs` on first put + increments `delivered_count` per put. The retention-window override itself persists durably under the singleton `sr:retention_window` key (`PersistedRetentionWindow { window_ms }`); `RedbSuspensionStore::set_retention_window(Duration)` writes it; `RedbSuspensionStore::retention_window()` reads it. New regression pins at `crates/benten-engine/tests/redb_suspension_in_process.rs` cover both correctness (single-process round-trip) + durability (override persists across `RedbSuspensionStore::open` re-open per r1-wsa-10). A `RedbSuspensionStore::open(path)` convenience constructor opens (or creates) a redb file for store-only deployments.

**Phase 2b state (historical, pre-G17-A2):** The `SuspensionStore::is_retention_exhausted` trait method enforces the SUBSCRIBE persistent-cursor retention window (1000-events / 24h). The in-memory test impl overrides correctly; the production `RedbSuspensionStore` used the trait default `false` and tracked `delivered_count` + `registered_at` in process-local memory. Consequence: a cross-process re-subscribe past the retention window did NOT surface `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED` because the counters reset on process boot. R6 Round-2 security-auditor (`r6-r2-sec-2`) reissued the Round-1 `r6-sec-4` open finding under HARD-RULE — destination must EXIST + receive entry NOW. Disclosure landed in `docs/SECURITY-POSTURE.md` Compromise #9 closure narrative at the same time as this entry.

**Why Phase 3:** The retention bookkeeping side-table shape composes with the durable grant-store + per-event read-cap-coverage work (§2.2 + `phase-2-backlog.md` §7.4). Landing it standalone in Phase 2b would require re-shaping the side-table when grant-store lands.

**Touch size:** ~50-60 LOC + 1 regression test pin.

### 6.7 AArch64 SANDBOX runtime CI cell (Apple Silicon test execution) — CLOSED at Phase-3 G17-B (wave-5b)

**Phase 2b state:** T4 multi-arch coverage was framed as `cargo check --target aarch64-apple-darwin` (compile-only) but the dedicated `multi-arch-cargo-check.yml` workflow was never authored at Phase 2b — the cite at §6.7 r6-r3-wsa-1 named the file as the destination knowing it would be authored at Phase 3 G17-B. Apple Silicon SANDBOX runtime behaviour (sigaltstack handler, 16-byte stack alignment + max_wasm_stack interaction with M-series memory model, epoch-deadline thread fairness on the heterogeneous E/P core scheduler) was uncovered at runtime CI.

**Phase 3 closure (G17-B wave-5b):** `.github/workflows/multi-arch-cargo-check.yml` authored with two job tiers:
1. **`cargo-check-multi-arch`** — compile-only T4 across Linux x86_64 / Linux arm64 / macOS arm64 (macos-14) / macOS x86_64 (macos-15-intel).
2. **`aarch64-sandbox-runtime`** — runs `cargo nextest run -p benten-eval --target aarch64-apple-darwin --test sandbox_basic --test sandbox_escape_attempts_denied --test sandbox_severity_priority` on `macos-14` (Apple M1).

The Rust-side anchor test `crates/benten-eval/tests/sandbox_severity_priority_g17_b_anchor.rs::aarch64_sandbox_runtime_ci_cell_green` greps the workflow YAML for the invocation shape per r4-r1-wsa-7 (cargo nextest run + flag-position `--target` + per-test `--test` flag-position) so a refactor that drifts the shape fails the anchor BEFORE reaching the CI cell. Pairs with the SANDBOX runtime maturity cluster (§6.1 ESC-16 + §6.4 SandboxStackExhausted) since AArch64-specific surfacing of stack-overflow / fingerprint-collapse defects is most likely to come from this cell.

**Touch size (actual):** ~95 LOC YAML + ~80 LOC anchor-test rewrite (within plan ceiling).

### 6.8 SANDBOX kv:write read-only-snapshot enforcement seam (folded into §6.0)

**R6-R4 r6-r4-doc-3 dedupe.** §6.0 and this section both named `r6-r3-arch-2` and described the same SANDBOX kv:write read-only-snapshot enforcement seam (PR #70 Group C accidentally created two parallel entries during the R6-R3 docs+cite-precision fix-pass). The canonical content lives at §6.0 above. This stub is preserved (rather than removed) so any in-tree cite of `phase-3-backlog.md §6.8` continues to resolve to the same Phase-3 forward-pointer rather than 404; the wording at §6.0 is the authoritative version.

### 6.9 benten-dev `inspect-state` thin-CLI front-door — CLOSED at Phase-3 G20-A3 wave-8a

**Phase 2b state:** The Rust-side pretty-printer entry point at `tools/benten-dev/src/inspect_state.rs::pretty_print_envelope_bytes` IS shipped, but the wrapping `node bin/benten-dev.mjs` thin-CLI front-door for `benten-dev inspect-state <path>` is not yet shipped. R6 Round 3 stale-deferrals-deep-sweep (`r6-r3-sd-5`) flagged that `tools/benten-dev/test/inspect_state_pretty_prints.test.ts` (1 `describe.skip` + 3 `it.skip`) cited "Phase-2c item" as the destination, but Phase 2c is NOT a defined phase in `docs/FULL-ROADMAP.md` — HARD RULE clause-(b) violation (destination doesn't exist; "Phase 2c" appears informally as a deferred-bucket label in security-posture/error-catalog/host-functions for the deferred `random` host-fn but isn't a real plan-doc / roadmap entry). This entry is the populated destination.

**Phase 3 target:** Ship the `node bin/benten-dev.mjs` thin-CLI front-door wrapping the existing Rust-side pretty-printer. Wire `benten-dev inspect-state <path>` to read the suspended ExecutionState envelope bytes from `<path>` and pretty-print via `pretty_print_envelope_bytes`. Un-skip the 1 describe + 3 it tests in `inspect_state_pretty_prints.test.ts`.

**Why Phase 3:** The benten-dev thin-CLI surface is part of the broader Phase-3 DX hardening pass; the Rust-side entry point is shipped, so the test bodies pin the public-facing surface that lands in Phase 3 hygiene. Bundles cleanly with the rest of the Phase-3 first-wave CI-hygiene cluster (§7.3.A).

**Touch size:** ~30-50 LOC TS CLI wrapper + the 4 test un-skips.

**Closure shape (Phase-3 G20-A3 wave-8a):** `tools/benten-dev/bin/benten-dev.mjs` thin-CLI front-door committed (~115 LOC node ESM). The wrapper resolves the compiled `target/{release,debug}/benten-dev` Cargo binary (or PATH-installed binary) via `child_process.spawnSync` and forwards `inspect-state <path>` arguments verbatim — the canonical-bytes parsing stays in Rust (one source of truth) per the §6.9 commitment. `--help`, `-h`, `--version`, `-V` flags supported. The 1 `describe.skip` + 3 `it.skip` blocks in `tools/benten-dev/test/inspect_state_pretty_prints.test.ts` un-skipped.

### 6.11 G14-C inline base64 → workspace `data-encoding` dep (g14-c-mr-6 follow-up) — CLOSED at Phase-3 R5 wave-9 W9-T5

**Closure shape (Phase-3 R5 wave-9 W9-T5):** Workspace-level `data-encoding = "2"` (default-features off + `alloc`) declared in `Cargo.toml [workspace.dependencies]`; `crates/benten-engine/Cargo.toml` consumes via `data-encoding = { workspace = true }`. The ~80 LOC inline base64 encoder/decoder in `crates/benten-engine/src/manifest_signing.rs` (and its ~30 LOC mirror in `crates/benten-engine/tests/manifest_signing.rs`) are deleted; production call sites at `sign_manifest` and `decode_signature` now invoke `data_encoding::BASE64.encode(...)` / `data_encoding::BASE64.decode(b64.as_bytes())` directly. Round-trip equivalence for the RFC 4648 known-vector set pinned by `crates/benten-engine/src/manifest_signing.rs::tests::base64_round_trip_known_vectors_via_data_encoding`. Manifest-signing integration suite (12 tests at `crates/benten-engine/tests/manifest_signing.rs`) green; existing on-disk manifests round-trip identically (RFC 4648 alphabet + padding match the prior inline impl).

**Phase 3 G14-C state (historical, pre-W9-T5):** `crates/benten-engine/src/manifest_signing.rs:619-682` shipped an inline ~80 LOC base64 encoder/decoder used by `sign_manifest` / `decode_signature`. Cargo.lock confirmed `data-encoding` (and `base64ct`) were already in the dependency graph transitively. The inline implementation was functionally safe (length-checked the 64-byte signature, no panic on malformed input, no information leak in error paths beyond non-secret invalid-char position) but duplicated well-vetted alternatives in the workspace.

**Touch size (actual at closure):** ~110 LOC drop, ~14 LOC add (call-site swaps + workspace-dep entry + retained known-vector roundtrip test against the new dep).

### 6.10 `random` host-fn deferral — workspace CSPRNG framework choice — CLOSED at Phase-3 G17-A2 wave-5b

**Closure shape (Phase-3 G17-A2 wave-5b):** The workspace CSPRNG decision landed at R1 (D-PHASE-3-11 RESOLVED-at-R1) = `getrandom` direct (NOT `rand` ecosystem; NOT a deterministic seed). G17-A2 wires `random` into the codegen-default surface alongside `time` / `log` / `kv:read` (cap-string `host:random:read`, 4-segment shape mirroring `kv:read`; `cap_recheck = per_call`); the trampoline at `crates/benten-eval/src/primitives/sandbox.rs::register_default_host_fns` invokes `getrandom::getrandom` to fill the guest buffer. Per-call entropy budget defaults to **4096 bytes** (per r1-wsa-8); a manifest may override via the additive optional `host_fns.random.budget_bytes_per_call` field on `ModuleManifest`. Budget overrun fires the typed `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` variant (routed `ON_DENIED`). The validate-time deferral guard + `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` const are RETIRED; `crates/benten-eval/tests/sandbox_host_fn_random_deferred.rs` is deleted; new green-phase regression guards live at `crates/benten-eval/tests/random_host_fn.rs`. Compromise #16 → CLOSED at Phase-3 G17-A2 in `docs/SECURITY-POSTURE.md`. `host-functions.toml` now declares `[host_fn.random]` IMPLEMENTED.

**Phase 2b state (historical, pre-G17-A2):** D1 + sec-pre-r1-06 §2.3 deferred the `random` host-fn at Phase-2b open: shipping `random` before the workspace-wide CSPRNG framework decision was made would commit the engine to a CSPRNG choice (or trait-shape) that hasn't been audited across the rest of the runtime. The deferral was originally labeled "Phase 2c" across ~25 surfaces (security-posture, error-catalog, host-functions toml + docs, quickstart, runtime sandbox.rs error message, multiple test contracts, primitive_host docstrings, error variant doc). "Phase 2c" is NOT a defined phase in `docs/FULL-ROADMAP.md` — HARD RULE clause-(b) violation (same shape as §6.9; the random host-fn is the larger sibling of the inspect-state CLI deferral that was already retensed). This entry was the populated destination for the entire Phase-2c-labeled `random` cluster, now CLOSED via the G17-A2 wave-5b PR.

**Operator-runtime contract (closed shape, post-G17-A2):** A SANDBOX module that imports `random` (cap-string `host:random:read`) succeeds at validate-time and dispatches through the codegen-default trampoline that fills the guest buffer from `getrandom::getrandom`. Per-call entropy requests above 4096 bytes (or above the per-manifest override `host_fns.random.budget_bytes_per_call`) fire the typed `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` (`ON_DENIED` family). The pre-G17-A2 validate-time deferral guard at `crates/benten-eval/src/primitives/sandbox.rs::execute` (the `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` arm) is RETIRED; the regression-pin file `crates/benten-eval/tests/sandbox_host_fn_random_deferred.rs` is DELETED. New green-phase regression guards live at `crates/benten-eval/tests/random_host_fn.rs`.

**Phase 3 closure (DONE at G17-A2 wave-5b):** (1) ✅ workspace CSPRNG decision = `getrandom` direct (D-PHASE-3-11 RESOLVED-at-R1; not `rand` ecosystem; not deterministic-seed-via-attribution which would require the broader replay-context plumbing — kept available as a future widening if Phase-4+ replay surfaces need it). (2) ✅ `random` wired through the trampoline with constant-time cap-policy check (sec-r1-3). (3) ✅ validate-time deferral guard dropped. (4) ✅ deferred-test file deleted; new green-phase regression file at `random_host_fn.rs`. (5) ✅ `host-functions.toml` marks `random` IMPLEMENTED. (6) ✅ doc sweep across SECURITY-POSTURE.md Compromise #16 + ERROR-CATALOG.md (new `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` entry + retensed `E_SANDBOX_HOST_FN_NOT_FOUND` body) + QUICKSTART.md (random-host-fn-available retense).

**Why this landed in Phase 3 (rather than holding for replay/AttributionFrame seeding):** The "thought-through entropy story" the original deferral named was scoped at G17-A2 to "non-replay-deterministic CSPRNG with capability-gated budget" — sufficient for the Phase-3 target surface. Replay-determinism via attribution-frame seeding remains an OPEN Phase-4+ widening if the replay surface surfaces it as a real requirement (named at `docs/FULL-ROADMAP.md` Phase 4+ rather than as a Phase-3 deferral). The Phase-3 closure does not commit the engine to a CSPRNG seam Phase-4+ would have to redo; the additive `host_fns.random.budget_bytes_per_call` field per r1-wsa-8 is the per-manifest hook a future replay-deterministic seam would extend.

**Touch size (actual at closure):** ~600 LOC across `crates/benten-eval/{src,tests}/` + `crates/benten-engine/{src,tests}/` + `bindings/napi/` + `docs/{SECURITY-POSTURE,ERROR-CATALOG,HOST-FUNCTIONS,SANDBOX-LIMITS,QUICKSTART}.md` + `host-functions.toml`.

---

### 6.12 G16-B post-canary residuals (Version Node mint + e2e composition observer + device-CID production threading)

**Status:** OPEN — NAMED-NOW destination for items deferred from G16-B canary (PR `wave-g16-b-a-canary`). Three residuals tracked here; each is structural-surface-adjacent work the canary's parallel-3 wave (G16-B-B/C/D) consumes.

**Origin:** G16-B canary (wave-g16-b-a-canary) brief at `.addl/phase-3/WAVE-G16-B-CANARY-BRIEF.md` lands the structural surfaces (AttributionFrame field extensions; cap_snapshot_hash 4-input signature; WriteContext/ReadContext device_cid; AtriumError::SyncHopDepthExceeded variant; `merge_remote_change_with_hop_depth` returning a `SyncMergeAttribution` seed). Three downstream surfaces pinned-RED at canary scope; each needs a Phase-1-backbone surface (anchor store / chunk-bypass / write-path engine threading) the canary explicitly defers.

**Items:**

1. **Version Node mint after Loro merge (`AtriumHandle` → `Engine` engine-side glue).** The G16-B canary surfaces the `SyncMergeAttribution` seed (peer node-ids + new sync_hop_depth) at `merge_remote_change_with_hop_depth`. The engine-side wave-6b implementer wires the mint of a new Version Node via the existing Anchor + Version + CURRENT pattern (per arch-r1-4 D-C HYBRID). Pre-G16-B-B `Engine::create_anchor` / `append_version` are Phase-1 stubs (`E_NOT_IMPLEMENTED`); G16-B post-canary wires the anchor store + the engine-side merge callback. RED-PHASE test pins to un-ignore at this wave (in `crates/benten-engine/tests/loro_version_chain.rs` + `sync_replica_attribution.rs`): `merge_loro_change_creates_versioned_anchor` + `sync_replica_write_attribution_carries_device_did_alongside_parent` + `sync_replica_attribution_frame_sync_hop_depth_bounded_with_e_sync_hop_depth_exceeded` (incoming hop-depth currently always 0 at the merge entry; G16-D wave-6b handshake protocol surfaces the carrier). **CLOSED 2026-05-08 G16-B-prime** — `Engine::create_anchor` / `append_version` / `read_current_version` / `walk_versions` real impls landed (in-memory anchor store keyed by name on `EngineInner`); `Engine::apply_atrium_merge` orchestrates SyncMergeAttribution → trust-store DID resolution → AttributionFrame stamping → Version Node mint via append_version → CURRENT advance. All 3 RED-PHASE pins un-ignored + GREEN at HEAD; the multi-Anchor cross-Atrium variant + SUBSCRIBE-from-merge variant remain DEFERRED (named inline at `loro_version_chain.rs` to G16-B-D and G16-D wave-6b respectively).

2. **Compromise #11 deepest e2e composition pin (chunk-bypass observer).** `crates/benten-engine/tests/ivm_view_subscribe_compose.rs::compromise_11_both_gates_compose_observable_delivery_end_to_end` was RED-PHASE pending an engine-side test surface that exposes ChangeEvent.anchor_cid directly to test callbacks. **CLOSED 2026-05-09 G16-B-D** via Option (a): added `Engine::testing_subscribe_observable_change_events(pattern, observer, actor, cap_recheck) -> Subscription` cfg-gated under `cfg(any(test, feature = "test-helpers"))` in `crates/benten-engine/src/engine_subscribe.rs`. Helper bypasses the chunk-encoding bridge by passing eval-side `ChangeEvent` directly to the observer closure while reusing the same `DeliveryCapRecheck` bridge as the production `on_change_with_cap_recheck`. Pin asserts the load-bearing dual-gate composition (mat-admit ∩ delivery-admit = {row_A}; mat-deny but delivery-admit = {row_B}; mat-admit but delivery-deny = {row_C}; end-to-end intersection = {row_A} only). The integration test file is registered with `required-features = ["test-helpers"]` in `Cargo.toml` so the helper symbol is present at link time (mirrors `sandbox_metrics` discipline).

3. **Production-runtime threading of `device_cid` through engine WriteContext construction sites.** G16-B canary lands the structural `WriteContext.device_cid` + `ReadContext.device_cid` fields; production engine write-path call sites (`engine_diagnostics.rs::transaction commit hook`, `primitive_host.rs::WriteContext synthesis`) populate the field at construction time when a device-DID-attestation is present. **CLOSED 2026-05-08 G16-B-prime** — `Engine::set_device_cid` + `Engine::device_cid` round-trip surface + 2-callsite threading at `engine_diagnostics.rs::transaction` commit hook + `primitive_host.rs::check_capability` cap-check arm landed. Runtime-arm pin lives at `crates/benten-engine/tests/device_cid_runtime_arm.rs::capability_policy_per_device_cid_dispatch_observable_in_runtime_arm` (was previously a RED-PHASE pin in `crates/benten-caps/tests/device_dispatch.rs`; moved to engine-side tests because the production-runtime arm requires the `benten-engine` dep). **EXTENDED 2026-05-08 G16-B-prime fp** — read-side consumer audit closed: `device_cid` now threads through 6 additional ReadContext sites (`engine_views.rs:read_view_with`, `engine_diagnostics.rs:diagnose_read` 2 sites, `engine_crud.rs:get_node` + `read_denied_for_cid`, `primitive_host.rs:check_read_capability` dual-shape branch) + 1 additional WriteContext site (`engine_wait.rs:WAIT-resume cap-recheck`). Read-path runtime-arm pin extended at `device_cid_runtime_arm.rs::capability_policy_per_device_cid_dispatch_observable_on_read_path`.

4. **AttributionFrame.actor_cid decoupling from device_cid at sync-merge boundary (cap-g16bp-1).** G16-B-prime's `apply_atrium_merge` initially conflated AttributionFrame.actor_cid with device_cid — preserving Phase-3 single-user single-device behavior but losing the principal/device split that Phase-4+ AI-agent + handler-attribution flows depend on. **CLOSED 2026-05-08 G16-B-prime fp** (Ben's RATIFIED Option A, decouple-now): added `EngineInner.actor_cid: Mutex<Option<Cid>>` slot + `Engine::set_actor_cid` setter + `Engine::effective_actor_cid` accessor (falls back to `device_cid` when unset). `apply_atrium_merge` now sources `AttributionFrame.actor_cid` from `effective_actor_cid()`; legacy callers see no behavior change (single-device fallback). Test pin at `sync_replica_attribution.rs::sync_replica_explicit_actor_cid_decouples_from_device_cid` asserts the explicit-set path AND the regression-guard against future re-conflation.

5. **D-PHASE-3-19a pinned-CID rebake — AttributionFrame.device_did convention shift `device-cid:<hex>` → `did:key:<resolved>`.** G16-B-prime's `apply_atrium_merge` populates `AttributionFrame.device_did` with the synthetic-string convention `device-cid:<hex(device_cid)>` pre-G16-D-handshake-protocol-body landing. Once the production trust-store promotes device-CID → resolved `did:key:<…>` form (G16-D wave-6b), the convention shift will mutate the canonical bytes of every minted AttributionFrame at sync-merge boundaries. **NAMED-NOW destination for the D-PHASE-3-19a pinned-CID rebake cohort entry.** Touch site: G16-D wave-6b implementer adds the `device-cid:` → `did:key:` translation at `engine.rs:apply_atrium_merge` + rebakes any pinned-CID fixtures asserting the legacy `device-cid:<hex>` shape (currently: `sync_replica_attribution_carries_device_did_alongside_parent` constructs the expected frame with `device-cid:<hex>`; `sync_replica_explicit_actor_cid_decouples_from_device_cid` does the same). Both tests + their pinned `expected.cid()` fixtures are part of the rebake cohort. **PARTIAL CLOSURE 2026-05-09 G16-D wave-6b** — on-the-wire device-DID-attestation envelope flow landed (`DeviceAttestationEnvelope` v1 emitted/parsed in `sync_subgraph` + `accept_sync_subgraph`; `AtriumHandle::set_local_device_did` setter; `SyncMergeAttribution.remote_device_did` carrier; `apply_atrium_merge` prefers the wire envelope's `device_did` when present). The two existing pinned-CID fixtures STILL pass with the legacy `device-cid:<hex>` shape because they bypass the wire envelope (call `apply_atrium_merge` directly without a sync_subgraph round-trip) — i.e. the FALLBACK PATH remains unshifted, so no rebake of those fixtures is required at this wave. **REMAINING OPEN:** trust-store DID-resolution promotion of locally-bound device-CIDs to `did:key:<…>` form (the producer side of the convention shift). When that lands, `AtriumHandle::set_local_device_did` callers will pass `did:key:<…>` form, which the receiver-side AttributionFrame will then carry verbatim — at which point any new pinned-CID fixtures asserting the wire-envelope path use `did:key:<…>` form natively. The legacy fallback stays `device-cid:<hex>` as the documented "no envelope received" carrier. New pinned CID at `tests/integration/atrium_two_device.rs::atrium_two_device_same_identity_selective_zone_sync` GREEN-PHASE asserts the wire-envelope path with `did:key:zAlice{Laptop,Phone}Device` form — this pin is the load-bearing exit-criterion-16 closure.

6. **sec-r4r1-2 BLOCKER — sync-replica WRITE per-write cap-recheck-at-delivery (mirrors SUBSCRIBE side per CLR-2) — CLOSED 2026-05-09 G16-B-F (PR #161).** Two RED-PHASE pins at `crates/benten-engine/tests/sync_replica_attribution.rs`: `sync_replica_write_cap_recheck_at_delivery_against_local_grant_store` + `sync_replica_write_after_local_grant_revoke_post_handshake_rejected_with_e_sync_revoked_during_session` un-ignored end-to-end against the production `Engine::apply_atrium_merge` per-row loop.

   **Closure shape (Ben's ratified Option (a) — structural-always-on per-row cap-recheck inside `apply_atrium_merge`):**
   - (a) Wire per-row cap-recheck into `Engine::apply_atrium_merge`'s post-merge loop (NOT a parallel `consume_sync_replica_message` entry-point per Ben's "err on the side of security generally and then open safe QOL paths later" framing). Every row's `(peer_actor_cid, zone)` pair is checked against an in-memory `revoked_actor_zone_pairs` set + the engine's `CapabilityPolicy::check_write` hook. Recheck fires BEFORE Version Node mint — single revoked row vetoes whole merge atomically (CURRENT does not advance).
   - (b) Added `EngineError::SyncRevokedDuringSession { peer_did, zone, cid }` variant + `ErrorCode::SyncRevokedDuringSession` (4-surface atomic update per §3.5g: ErrorCode enum / ALL_CATALOG_VARIANTS array / `docs/ERROR-CATALOG.md` / `errors.generated.ts`).
   - (c) Added `Engine::caps()` returning `EngineCapsHandle` with `install_proof(CapProof)` / `revoke(...)` for grant-mutation at the policy layer (in-memory mirror of durable system:CapabilityRevocation node-write path; bridge-only shape pre-G14-B durable identity backend).
   - (d) Added `Engine::sync_replica_cap_recheck_calls()` AtomicU64 counter; increments per-row at the recheck site (not per-batch / not per-merge — per-row is what cap-r4-3 / r4b-cap-4 want).
   - (e) Un-ignored both pins: pin (a) asserts counter advance per-row; pin (b) asserts typed-error rejection + ON_DENIED edge-routing + Anchor CURRENT non-advance + `E_SYNC_REVOKED_DURING_SESSION` catalog code.

   **Bridge-shape carries (named here for v1-window / G14-B durable promotion lockstep migration):**
   - **Multi-peer-merge actor-cid resolution narrows to `peer_node_ids.first()`** today (g16bf-cap-1) — non-load-bearing for two-peer canary; multi-peer-mesh wave (post-G16-B-E iroh-substantive) needs to widen this to per-row peer attribution from the Loro op-log. Acceptance criterion for multi-peer-mesh wave.
   - **`blake3::hash(did.as_bytes())` actor_cid bridge** replicated in 3 sites (install_proof / revoke / per-row recheck) without shared helper (g16bf-cap-3) — G14-B durable identity backend promotion will need lockstep migration. Acceptance criterion: extract a shared helper at G14-B promotion + verify all 3 sites use it.

   **Convergence verdict:** sec-r4r1-2 BLOCKER CLOSED end-to-end (the SUBSCRIBE side has 3 cap-recheck pins; the symmetric sync-replica WRITE side now has 2 un-ignored pins driving production runtime — both halves landed pre-tag per CLR-2 dual-layer recheck). Total touch ~390 LOC at PR #161. Pre-flight 8/8 GREEN.

**Touch estimate:** Item (1) ~300-500 LOC engine-side anchor store wiring + ~150 LOC merge callback. Item (2) ~50-150 LOC test surface + un-ignore. Item (3) ~30-80 LOC threading + un-ignore. Item (4) ~20 LOC API + threading + 1 test. Item (5) ~10-20 LOC translation + ~6 LOC pinned-CID fixture rebake. Combined: ~500-800 LOC across G16-B-B/C/D parallel wave.

**Convergence target:** All three test pins un-ignored + GREEN-PHASE before R6 phase-close convergence council. The structural surfaces landed at canary mean wave-6b wiring does NOT need to refactor existing types — purely additive.

7. **Atrium `leave()` + `rejoin()` API surface (R4b dist-systems sub-item C carry).** Pre-G16-B-D the only Atrium teardown surface is `AtriumHandle::close(self)` which consumes the handle; there is NO `leave()` (non-consuming graceful tear-down) NOR `rejoin()` (idempotent re-establish on the same handle). The R4b dist-systems lens flagged that an Atrium peer leaving + rejoining MUST: (1) drop outbound subscription state cleanly (no orphaned ChangeEvent fan-out); (2) on rejoin, reconcile state via Loro CRDT merge (idempotent re-merge from zero state); (3) preserve causal-history continuity in `AttributionFrame.peer_did_set` across the leave-rejoin window. **NAMED-NOW destination** per HARD RULE rule-12 clause-(b). Concrete fix-shape:
   - (a) Add `AtriumHandle::leave(&self)` — non-consuming graceful teardown that flips an `is_active` flag + drops outbound subscription registrations + leaves the iroh endpoint open for `rejoin()`.
   - (b) Add `AtriumHandle::rejoin(&self) -> AtriumResult<()>` — re-establishes the connection on the same handle; idempotent if called when already active.
   - (c) Test pin at `crates/benten-engine/tests/atrium_leave_rejoin.rs` (NEW; fn `peer_leave_then_rejoin_reconciles_state_via_loro_merge`) (or extend `engine_sync.rs::tests` mod): close + re-open the underlying iroh endpoint; export Loro state from peer B; apply at peer A AFTER rejoin; assert (i) the post-rejoin `apply_atrium_merge` succeeds, (ii) the merged Version's `AttributionFrame.peer_did_set` includes the originating peer-DIDs from the pre-leave window (continuity), (iii) re-applying the same bytes is idempotent (CRDT replay safety).
   - **Touch size:** ~80-150 LOC engine-side (`engine_sync.rs::leave` + `rejoin` + bookkeeping flag) + ~80 LOC test pin. **Convergence target:** Phase-3-close-blocking iff R4b dist-systems lens names this as a CLOSURE-PHASE-3 BLOCKER. **CLOSED at wave-g16-b-g** (PR pending) — Ben ratified Phase-3-close-blocking 2026-05-09. Landed: `AtriumHandle::leave(&self)` + `AtriumHandle::rejoin(&self)` + `AtriumHandle::is_active()` accessor + `AtriumInner::is_active` AtomicBool flag + `ensure_active()` private helper gating `sync_subgraph` / `sync_subgraph_over` / `accept_sync_subgraph` / `merge_remote_change_with_hop_depth` on the lifecycle flag (rejects with `AtriumError::InvalidState` when inactive). End-to-end test pin at `crates/benten-engine/tests/atrium_leave_rejoin.rs` (`peer_leave_then_rejoin_reconciles_state_via_loro_merge`) drives the full leave → mid-leave-merge-refusal → peer A keeps writing → rejoin → post-rejoin apply_atrium_merge → re-apply-idempotent cycle through the engine apex orchestrator. The handle's iroh endpoint + per-zone Loro docs + trust-store + device-attestation table ALL survive across the leave-rejoin window so AttributionFrame.peer_did_set continuity is preserved via the surviving trust-store (asserted via the resolved-DID-shape check post-rejoin). Outbound subscription registry drop-on-leave is currently a no-op because the engine_sync.rs surface does not carry an in-flight subscription registry today (the eval-side `ON_CHANGE_REGISTRY` is process-scoped per §6.12 item 8); when item 8 option-(b) refactor lands, the `leave()` site drops the handle's outbound entries from that registry.

8. **`ON_CHANGE_REGISTRY` process-scoped global causes parallel-test cross-talk on shared labels (G16-B-D ds-mr-2 INFORMATIONAL).** The eval-side change-event registry at `crates/benten-eval/src/primitives/subscribe.rs::ON_CHANGE_REGISTRY` is a `LazyLock<Mutex<HashMap<...>>>` shared across the whole test binary. When two parallel tests register subscriptions with the same `pattern` label, sibling engine instances' writes can deliver events for OTHER engines' subscriptions; the F6 `is_actor_active` cap-recheck dual-layer guard then auto-cancels the foreign subscription per D5 (correct security behavior, but produces test-order-dependent false negatives). Surfaced 2026-05-09 by G16-B-D dist-systems mini-review (`r5-w-g16-b-d-mini-review-distributed-systems.json::ds-mr-2`); implementer worked around inline by using locally-unique label `compromise11post` for the Compromise #11 deepest pin. Pim-N candidate validated as REAL by correctness lens (`r5-w-g16-b-d-mini-review-correctness.json`).
   - **v1-assessment question:** is the right shape (a) test-discipline (mandate locally-unique labels in any test using SUBSCRIBE; codify in dispatch-conventions) OR (b) architectural change (registry scoped to engine-instance handle rather than process-global)? Option (a) is cheap; option (b) eliminates the surface entirely + matches Phase-3 multi-instance semantics (each engine should have its own subscription registry as a v1 narrative, not a process-shared state cookie).
   - **Touch size if option (b) lands:** ~50-100 LOC architectural refactor (registry → `EngineInner` field + accessor; existing tests un-affected if accessor is workflow-compatible).
   - **OPEN — v1-assessment-window candidate** (this is a soft Phase-3 closure question; option (a) at minimum can land at R6 phase-close convergence council as a dispatch-conventions discipline ratification).

9. **Heterogeneous-cap-envelope per-device write filter at sync-replica boundary (registered NEW 2026-05-09 G16-D wave-6b fix-pass).** **NAMED-NOW destination** per HARD RULE rule-12 clause-(b) for the cap-denial halves of plan §1 exit-criterion 16 RED-PHASE pin (steps 3 + 6) — phone (with `holds_zones=Specific(["/zone/notifications"])` envelope) MUST be denied write to `/zone/notes` even though both devices share the same actor_cid. Closes the §3.1 phantom-destination violation surfaced at the post-PR-#163 correctness mini-review (g16d6b-corr-1 BLOCKER): the prior carry narrative pointed to a non-existent "G14-D heterogeneous-cap-envelope wave" without a registered §X.Y destination. This entry is the registered destination.

   **Unblocking dependency LANDED:** G16-D wave-6b fix-pass shipped the cryptographically-verified on-the-wire device-DID-attestation envelope (V2 signed shape; receiver-side `DeviceAttestationEnvelope::verify` + `Acceptor::accept_at` + payload-hash binding). The envelope's embedded `attestation.envelope: CapabilityEnvelope` carries the device's declared `holds_zones: ZoneScope` field — the per-zone write filter consults this surface verbatim.

   **Concrete fix-shape:**
   - (a) Extend `Engine::apply_atrium_merge`'s per-row cap-recheck loop (§6.12 item 6 sec-r4r1-2 closure shape) to ALSO consult the originating device's `attestation.envelope.holds_zones` against the `zone` argument. Reject with a typed `EngineError::SyncDeviceEnvelopeRejected { device_did, zone, holds_zones_summary }` mapping to a NEW `ErrorCode::SyncDeviceEnvelopeRejected` (`E_SYNC_DEVICE_ENVELOPE_REJECTED`, `ON_DENIED` routing) when the device's declared envelope does not include the merge zone.
   - (b) Persist the verified attestation per zone-merge so the apply_atrium_merge loop can consult `holds_zones` without re-decoding. Reuse the existing `last_received_remote_device_did: Mutex<BTreeMap<String, Option<String>>>` pattern; promote it to carry the verified attestation envelope (or just the resolved `holds_zones` enum, scoped narrowly).
   - (c) Test pin in `tests/integration/atrium_two_device.rs` (NEW fn `phone_envelope_specific_zones_rejects_write_to_unlisted_zone`): laptop = `ZoneScope::Full`, phone = `ZoneScope::Specific(vec!["/zone/notifications".into()])`; phone attempts write to `/zone/notes`; receiver-side `apply_atrium_merge` rejects with `E_SYNC_DEVICE_ENVELOPE_REJECTED`. Pinned-CID `expected_cid` for the rejection-Anchor-non-advance assertion (no Version Node minted on rejection).
   - (d) `§3.5g` 4-surface atomic update for the new ErrorCode variant (CATALOG_VARIANT_COUNT 109 → 110).

   **Touch size:** ~80-120 LOC engine-side (per-row envelope-zone filter + ErrorCode + persistence) + ~80-100 LOC test pin + ~30 LOC `ERROR-CATALOG.md` + 4-surface atomic-update.

   **Convergence target:** v1-assessment-window candidate. The G16-D wave-6b fix-pass cryptographic closure makes this filter implementable (the unblocking dependency landed), but criterion 16 ✅ FULL closure is decoupled from this filter per the cryptographic-attestation-only scope Ben ratified 2026-05-09. Item 9 stays OPEN until v1-window assessment determines whether heterogeneous-cap-envelope filtering is v1-blocking.

   **Origin:** Phase-3-fp post-PR-#163 correctness mini-review BLOCKER g16d6b-corr-1 (phantom-destination); cross-corroborated by cryptography lens g16d6b-crypto-1 disposition rationale (cap-recheck only bounds the ATTACK, doesn't close DID-forgery itself; envelope-filter is the cap-side complement to the cryptography-side closure that landed at this fix-pass). The two halves move together architecturally even if they ship in different waves.

   **Adjacent carry — silent-unsigned-emission setter-ordering trap (ds-fp-mr-g16dw6b-MINOR-2).** A peer that calls `AtriumHandle::sync_subgraph` BEFORE configuring `set_local_device_attestation` + `set_local_device_keypair` emits a V2 envelope with `attestation = None` (the legacy unsigned shape) silently — the handle has no precondition gate to refuse-or-warn on the unconfigured-attestation path. NOT criterion-16-closure-blocking (the receiver-side `verify` correctly handles the legacy `None` shape per the documented backward-compat contract; cryptographic closure for configured peers holds), but a configuration-trap surface for production deployments. Closure shape (in lockstep with the heterogeneous-cap-envelope filter above): add a `require_signed_envelope` configuration on `AtriumHandle` that, when set, causes `sync_subgraph` to return `AtriumError::InvalidState` if `set_local_device_attestation` or `set_local_device_keypair` are absent. Same v1-assessment-window scope as the per-device write filter — both are operator-deployment hardening surfaces that the wire envelope's V2 cryptographic foundation makes implementable.

---

## 7. Observability + diagnostic completeness

### 7.1 SANDBOX execution metrics propagation (Compromise #17 reinforcement)

**Phase 2b state:** R6 metadata-producer-vs-consumer audit (`r6-mpc-3`) + R6 napi-bindings (`r6-napi-3`) + R6 dx-optimizer (`r6-dx-10`) — three lenses converged — flagged: `Engine::describe_sandbox_node` claims `fuel_consumed_high_water` + `last_invocation_ms` metrics, but `SandboxResult.fuel_consumed` + `output_consumed` are dropped at `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (the post-execute return-shape extraction in the second `execute_sandbox` definition: only `output` propagates back to the engine wrapper). The diagnostic surface always returns `Err(Unknown)`; TS surface synthesizes hardcoded defaults client-side. Wave-8j R6-FP landed the doc-fix variant (honest "unknown" route + Compromise #17 reinforcement narrative); full metric-propagation deferred here.

**Phase 3 target:** Thread `fuel_consumed` + `output_consumed` + `last_invocation_ms` through the engine wrapper into a per-node high-water tracker on the `SandboxNodeState` side-table. Surface via `describe_sandbox_node` returning real values. ~150 LOC + 1 regression test pin per metric.

**Why Phase 3:** Side-table schema for `SandboxNodeState` is Phase-3-shaped (durable across restart implies the GraphBackend umbrella trait, §1.1). Metrics-in-RAM-only without §1.1 would land then need re-shape immediately when §1.1 does.

**Touch size:** ~150-200 LOC.

**Phase-3 G19-C2 wave-7 landing state (2026-05-07):** parts (a) RAM-only per-handler high-water tracker + (b) napi `describeSandboxNode` cfg-gated bridge + (c) TS-side `Engine.describeSandboxNode` consuming real numeric values landed. Per stream-r1-8: high-water values are PER-INVOCATION updates against the high-water mark within a single Engine instance; the cross-process WAIT-resume envelope does NOT carry in-flight SANDBOX metrics across the suspend boundary (a fresh `Engine::open` starts with an empty metrics map by design). Persistent durable cross-restart metrics are still §1.1 GraphBackend umbrella trait shaped — RAM-only tracker is the Phase-3 minimum-viable; durable promotion follows §1.1 / §1.2 Snapshot direct-wire.

### 7.1.1 SnapshotBlobBackend metric-propagation entry (cross-ref §1.2)

**Phase 2b state:** §7.1 above describes SANDBOX execution-metrics propagation (`fuel_consumed`/`output_consumed` propagation through engine wrapper into a per-node high-water tracker). The SnapshotBlobBackend direct-wire (§1.2) is the structural unblocker because the per-node side-table that holds those metrics lives in the GraphBackend umbrella trait the genericism unlocks. R6-FP Group 2 PR #61 docstrings cite this entry by name (`packages/engine/src/engine.ts:1567` — the `public async describeSandboxNode(...)` JSDoc; class-method, so cited by line — and `bindings/napi/src/sandbox.rs:108-119` comment block).

**Phase 3 target:** Same as §7.1 — this is a re-naming to match what Group 2's TS docstrings cite. SANDBOX metric-propagation lands together with SnapshotBlobBackend direct-wire (§1.2) because both want the same per-call-state side-table on the new backend trait.

**Touch size:** Folded into §7.1 + §1.2; no separate budget.

### 7.1.2 openStream FinalizationRegistry leak detector + requiresExplicitClose accessor

**Phase 2b state:** R6-FP Group 2 picked the honest-downgrade path for r6-stream-1: `openStream`'s `requires_explicit_close` lifecycle is enforced server-side but NOT exposed at the JS surface. `engine.callStream` and `engine.openStream` are functionally indistinguishable from the JS caller's perspective today. The honest disclosure lives in the `public openStream(...)` JSDoc at `packages/engine/src/engine.ts:1796` (class-method on `Engine`; cited by line because the detector's TS class-method discovery is limited to `function`/`class`/`interface`/`namespace` declaration shapes).

**Phase 3 target:** Wire two pieces:
- (a) **napi accessor `requiresExplicitClose()` on StreamHandle** — exposes the server-side flag to JS.
- (b) **TS-side `FinalizationRegistry` leak detector** — fires `E_STREAM_HANDLE_LEAKED` when a handle held by GC carries the flag set + was never explicitly closed.

Together they realize the cr-r4b-10 closure-narrative claim that `E_STREAM_HANDLE_LEAKED` fires on a leaked open-stream handle.

**Touch size:** ~30-40 LOC napi + ~20-30 LOC TS + 1 leak-detector test. Risk surface: low (purely additive observer; no production-runtime semantics change).

**Phase-3 G19-C2 wave-7 landing state (2026-05-07):** parts (a) + (b) landed. The 4 master scenarios from stream-r1-4 are pinned:
- (a) handler-returns-no-close + GC: gated on `typeof globalThis.gc === "function"` (requires `--expose-gc`); fires through the FinalizationRegistry callback.
- (b) handler-throws-no-close + GC: same `--expose-gc` gate; fires through the same callback path.
- (c) natural-completion negative pin: deterministic — the iterator's `return()` path disarms the bookkeeping flag so `for-await ... break` / drain-to-end does NOT fire `E_STREAM_HANDLE_LEAKED`.
- (d) `Engine.shutdown()` drain: deterministic — walks the engine's open-explicit-close-handle set, fires shutdown-drain leak events for each, then closes the wrapper.

**Sub-mechanism — GC-pressure-timeout polling fallback (§7.1.2.1):** for environments without `--expose-gc` (Node default + most browser-target runtimes), the FinalizationRegistry callback timing is non-deterministic. A bounded-retry polling fallback that fires the leak event on a configurable timeout (default ~5s) is a follow-up. Touch size: ~50-100 LOC TS + 2-3 test scenarios. Lands in a Phase-3 narrow-iter cycle (post-G19-C2 close); the 4 master scenarios are sufficient for §7.1.2's load-bearing observable-consequence contract per pim-2 §3.6b.

### 7.1.3 UserView.snapshot() + onUpdate() runtime materialization — CLOSED at Phase-3 G19-C1 + G19-C1-fp wave-7

**Phase 2b state:** G8-B (PR #28) shipped engine + DSL surface for user-registered IVM views. The TS-side `UserView` type is registered + dispatchable today, but the JS-observable runtime accessors (`view.snapshot()` returning current materialized state + `view.onUpdate()` returning an async iterator of incremental deltas) were red-phase-deferred. R6-FP Group 2 PR #61 `packages/engine/test/views.test.ts:32-50` `.skip` rationale names this entry as the destination.

**Phase 3 target:** Implement the two runtime accessors:
- (a) `view.snapshot(): Promise<T[]>` — returns current materialized rows from the IVM-maintained side-table; consults the canonical view registry's read-path.
- (b) `view.onUpdate(): AsyncIterableIterator<ViewDelta<T>>` — yields incremental deltas as ChangeEvents commit + Algorithm B maintains the view; consumed via `for await`.

**Why deferred:** The runtime-materialization path consumes the same per-view side-table that §7.1 metric propagation needs (lives in the GraphBackend umbrella trait). Phase-3's IVM Algorithm B generalization (§5.1) is the natural bundling site.

**Touch size:** ~150-250 LOC engine + napi + TS + 3-5 regression tests. Risk surface: medium (introduces a new public API).

**Phase-3 G19-C1 + G19-C1-fp wave-7 LANDING STATUS:** (a) + (b) LANDED — `Engine::user_view_snapshot` + `Engine::user_view_on_update` (engine_views.rs) + napi `userViewSnapshot` / `userViewDrainUpdates` / `userViewChangeOffset` accessors + TS-side `view.snapshot()` AsyncIterable + `view.onUpdate(): AsyncIterableIterator<ViewDelta>` polling iterator wired through `UserViewRuntimeShim` in `packages/engine/src/views.ts::buildOnUpdateIterator`. The G19-C1-fp follow-up sub-wave lifted `view.onUpdate` from the prior callback shape (`onUpdate(cb) -> UserViewSubscription`) to the AsyncIterableIterator shape per the original (b) target — clean break since pre-Phase-3 surfaces aren't in customer hands. The `UserViewSubscription` interface was dropped; cancellation is now via `iterator.return()` (or `for-await ... break`). Tests `user_view_snapshot_returns_current_materialized_rows` (Rust) + the TS-side end-to-end pins in `packages/engine/test/views.test.ts` ("graceful-fallback" + "cancellation via iterator.return()" + "ViewDelta { kind: 'change', payload } shape" + "native-binding fault closes iterator cleanly") GREEN. The polling cadence (25ms) is tunable post-Algorithm-B-port if back-pressure surfaces.

### 7.1.4 WAIT TTL TS DSL + suspend/resume DX surface widening (post-G12-E)

**Phase 2b state:** G12-E (PR #43, #57) shipped the engine-side WAIT envelope + suspension store. The TS-side DSL helpers for WAIT-TTL (declarative time-bounded waits with auto-resume on TTL expiry) + ergonomic suspend/resume call shapes are red-phase-deferred. R6-FP Group 2 PR #61 `packages/engine/test/wait_ttl.test.ts:34-36` `.skip` rationale names this entry as the destination.

**Phase 3 target:** Widen the TS DSL with:
- (a) `subgraph(...).waitWithTtl(signal, { ttlMs })` builder method — declarative TTL on a WAIT primitive.
- (b) `engine.callWithSuspension(handler, args)` returning `{ kind: 'suspended', handle, stateCid, signalName } | { kind: 'complete', result }` — already partially landed (Round-2 Instance 12 wired stateCid + signalName); Phase 3 adds the matching `engine.resumeWithMeta(handle, { signal, payload })` ergonomic wrapper.
- (c) `engine.testingAdvanceWaitClock(ms)` testing helper — currently the test file references it but the napi binding doesn't expose it.

**Why deferred:** The `testingAdvanceWaitClock` helper requires test-mode mock-clock plumbing that crosses the napi boundary; Phase-3's broader engine clock-injection work bundles cleanly.

**Touch size:** ~80-150 LOC TS surface + ~30-50 LOC napi binding + 5-7 regression tests + DSL spec doc updates.

**Phase-3 G19-C1 wave-7 LANDING STATUS:** (a) + (b) entry-point + (c) LANDED — `subgraph(...).waitWithTtl({ signal, ttlMs })` (+ positional overload) on `SubgraphBuilder` + `CaseBuilder` (dsl.ts); `Engine.resumeWithMeta(envelope, signal)` ergonomic wrapper carrying `ResumeWithMetaResult` discriminated union (engine.ts); `Engine::testingAdvanceWaitClock(deltaMs)` napi `#[napi]` method test-helpers feature-gated + `benten_napi::testing::testing_advance_wait_clock` rlib free function. The (b) `resumeWithMeta` body currently always resolves to `{ kind: "complete", outcome }` — the discriminated `kind: "suspended"` arm exists in the public type contract for forward-compat with cross-process re-suspension (post-D12 wiring). The (c) `testing_advance_wait_clock` body is a forward-compatible no-op until D12's `MockMonotonicSource` injection plumbing lands. Tests `wait_ttl_dsl_subgraph_builder_round_trip` (+ overload + reject pins) + `engine_resume_with_meta_ergonomic_wrapper` + `testing_advance_wait_clock_napi_binding_present` GREEN.

### 7.1.5 STREAM ESC defenses per-handler configurability (per-handler chunk-count + wallclock-budget)

**Phase 2b state:** R6 Round 1 streaming-systems lens (`r6-stream-5`) flagged that the STREAM primitive's ESC defenses (chunk-count cap + per-call wallclock budget) are workspace-global constants today rather than per-handler-tunable knobs. R1 disposition was BELONGS-ELSEWHERE-NAMED to "phase-2-backlog.md §10.4 (or new §10.5 STREAM widening)"; R6 Round 3 streaming-systems-redux (`r6-r3-stream-OOS-2`) verified neither destination was populated and surfaced the partial-fail of HARD RULE clause-(b). This entry is the populated destination.

**Phase 3 target:** Lift the chunk-count cap + wallclock-budget for STREAM out of workspace-global constants into per-handler `SubgraphSpec.primitives` properties (mirrors the SANDBOX `wallclock_ms` / `output_max_bytes` per-handler-knob shape per D24/D15). Wire the per-handler reads through the STREAM executor at primitive-entry; surface validation failures as registration-time `E_INV_STREAM_CONFIG` typed-error if the configured values exceed capability-grant ceilings.

**Why Phase 3:** Pairs with Phase-3 STREAM/SUBSCRIBE end-to-end work in §7.3.A.2 (test bodies pinning the configurability surface) + the broader per-handler knob taxonomy that SANDBOX already established (so STREAM lands as the second instance of a now-codified pattern rather than as a one-off knob set).

**Touch size:** ~50-80 LOC eval-side per-handler config read + ~20 LOC registration-time validation + ~30-50 LOC test pins.

### 7.2 BentenError.context full structured-field coverage — CLOSED in Phase 3 G19-B (R5 wave-7)

**Phase 2b state:** R6 deep producer-consumer sweep (Instance 8) flagged: every typed `EngineError` variant with structured fields drops them at the napi → TS boundary because `engine_err()` formats Display-only and `mapNativeError` extracts only the `E_*` code. Wave-8j R6-FP Groups 1+2 land the MINIMAL fix: `napi::Error::with_metadata` carries a JSON-serialized field bag for the most-load-bearing variants (`Invariant(RegistrationError)` + `ModuleManifestCidMismatch` + `IvmViewStale` + ~5 others). Full coverage of all ~20 EngineError variants + the long tail of `EvalError` deferred here.

**Phase 3 closure (G19-B R5 wave-7):** Replaced the legacy `$$benten-context$$` sentinel-suffix carrier with a JSON-shape envelope `{ code, message, fields? }` (formatter at `bindings/napi/src/error_envelope.rs::engine_err_envelope_json`; carrier at `bindings/napi/src/error.rs::engine_err`). `EngineError::context_json` is now match-exhaustive across every variant with `kind` discriminator + structured fields per variant (defends against the pim-1 doc-coupling failure shape). TS `mapNativeError` JSON-parses the envelope body via `tryParseJsonEnvelope` (Path 1 in `packages/engine/src/errors.ts::mapNativeError`); the legacy `code:` prefix carrier is preserved as Path 2 for hand-rolled napi errors that pre-date the envelope. End-to-end pins at `bindings/napi/tests/benten_error_context.rs` (3 tests; Rust side) + `packages/engine/test/errors.test.ts` (6 tests; TS side) cover representative variant shapes (flat, multi-field, Cid-bearing, marker, generic) with ALWAYS-`code` / ALWAYS-`message` / OBJECT-`fields` cross-cutting invariants.

### 7.6 CODE_TO_CTOR codegen completeness — CLOSED in Phase 3 G19-B (R5 wave-7)

**Phase 2b state:** `packages/engine/src/errors.ts::CODE_TO_CTOR` is a hand-maintained Record mapping `E_*` strings to typed BentenError subclasses. R6 Round-2 r6-r2-napi-3's Instance 8 round-trip pin (the new `install_module` CID-mismatch test in `packages/engine/test/install_module.test.ts`) surfaced that the map is missing ~28 entries that the codegen emits as classes — so napi errors carrying those codes round-trip through `mapNativeError` with `code: "E_UNKNOWN"` rather than the typed subclass. R6 Round-2 fix-pass added the specific `E_MODULE_MANIFEST_CID_MISMATCH` entry to make the Instance 8 pin pass + named this entry as the destination for the broader sync.

**Phase 3 closure (G19-B R5 wave-7):** Implemented option (a) — `scripts/codegen-errors.ts` now emits a `CODE_TO_CTOR_GENERATED` map in `packages/engine/src/errors.generated.ts` keyed on every catalog code → its typed BentenError subclass constructor; `packages/engine/src/errors.ts::resolveCtor` consults the hand-curated `CODE_TO_CTOR` first (historically-curated fast path) then falls back to `CODE_TO_CTOR_GENERATED` (long-tail coverage). The catalog-driven completeness pin lives at `crates/benten-engine/tests/code_to_ctor.rs::code_to_ctor_codegen_covers_every_error_catalog_entry` (walks `docs/ERROR-CATALOG.md` + asserts every `### E_*` entry appears in the generated map) + `code_to_ctor_no_e_unknown_fallback_for_known_code` (asserts every catalog code resolves to a typed subclass, never the synthetic `E_UNKNOWN` fallback). Drift-resistant by construction — no hand-maintained list to fall out of sync with the catalog.

### 7.7 napi-rs ThreadsafeFunction tuple-arg splat behavior — CLOSED in Phase 3 G19-B (R5 wave-7)

**Phase 2b state:** napi-rs v3's `Function<(A, B), Ret>` callback shape currently delivers the `(A, B)` tuple as a single-Array argument to the JS callback rather than splatting to 2 separate args, despite the d.ts emitting `(arg0: A, arg1: B) => Ret`. Affects both `Engine.onChange` (`(seq, payload)`) and the new `Engine.onEmit` (`(channel, payloadJson)`) callback shapes — the JS callback receives `args[0] = [a, b]` rather than `(a, b)`. The R6 Round-2 r6-r2-mpc-1 LOAD-BEARING test in `packages/engine/test/emit_subscribe.test.ts` accepts both delivery shapes via an `Array.isArray(channel)` runtime check; the pre-existing `subscribe.test.ts::LOAD-BEARING — onChange callback fires` test predates the workaround + currently fails on the same delivery shape. The napi-side wiring is correct (the engine-side EMIT broadcast publish IS firing + the TSFN IS delivering); the gap is the splat semantics on the napi-rs ↔ JS call edge.

**Phase 3 closure (G19-B R5 wave-7):** Investigation showed napi-rs v3.x already delivers `FnArgs<(A, B)>` as discrete splatted args (verified end-to-end via probe against the wave-6b napi cdylib). The pre-G19-B "single tuple-array" delivery shape belonged to an earlier napi-rs build. G19-B adopted r1-napi-4 path (b) tuned to the actual current behavior: `engine.ts::onChange / onChangeAs / onEmit` wrap the JS callback in a thin native wrapper that takes discrete args (`(seq: number, payload: Buffer)` / `(chanArg: string, payloadJson: string)`), preserves the user-callback's discrete-args contract, and adds the dx-r1-2b-4 / r6-dx-2 exception-isolation log path. The `Array.isArray(channel)` runtime tuple-detection branch in `emit_subscribe.test.ts` is retired (the splatted-args shape is the production reality). Pin coverage: `packages/engine/test/onChange_onEmit.test.ts` (3 tests — onEmit splat, onChange splat, retired-marker grep).

### 7.8 Engine.emitEvent standalone surface — wire through EmitBroadcast bus — CLOSED at Phase-3 G19-B R5 wave-7 (G19-A 50 LOC folded per scope-real-05)

**Phase 2b state:** `Engine.emitEvent(name, payload)` (TS at
`packages/engine/src/engine.ts:1228-1248`) and the matching napi
adapter at `bindings/napi/src/lib.rs::emit_event` both surface
`E_PRIMITIVE_NOT_IMPLEMENTED`. The standalone "emit a named event from
JS without a backing handler call" surface was deferred during Phase 1
when the change-stream fan-out was driven exclusively by storage
WRITEs. R6 Round-3 r6-r3-dx-5 surfaced that the original docstring
("deferred to Phase 2") violates HARD-RULE-12 vague-time-qualifier
(Phase 2 covers 2a + 2b + 2c; 2a is closed and 2b is closing). Naming
this entry as the destination + updating both docstrings with the
specific phase-3-backlog reference closes the disposition gap.

**Why this isn't done in Phase 2b:** The in-handler EMIT primitive
(`emit()` DSL builder) IS wired and routes through the EmitBroadcast
bus to EmitSubscription consumers (R6-R2-FP cluster-1, PR #66). The
standalone `Engine.emitEvent` surface needs a separate plumbing path:
threading `emit_event(channel, payload)` through the existing
EmitBroadcast publish entry without going through a handler dispatch.
Users who want standalone-event-emission today can compose a no-op
handler whose only Node is `emit(...)` and call it via
`engine.call(handler.id, "default", { channel, payload })` — friction
but not blocking.

**Phase 3 closure (G19-B R5 wave-7; G19-A 50 LOC folded per scope-real-05):**
`benten_engine::Engine::emit_event(channel, payload)` (at
`crates/benten-engine/src/engine.rs::emit_event`) now publishes
directly through the engine's `EmitBroadcast` bus (the same channel
`subscribe_emit_events` / `subscribe_emit_events_with_handle` consume);
the napi adapter at `bindings/napi/src/lib.rs::emit_event` accepts
`serde_json::Value` payload, calls `json_to_value_root` to convert to
`benten_core::Value`, and threads through. End-to-end pins:
`bindings/napi/tests/emit_event.rs::engine_emit_event_publishes_to_subscribed_on_emit_callback_end_to_end`
(rust-side; drives `testing::emit_event_round_trip` helper which
opens an in-memory engine, subscribes, publishes, and asserts the
callback fires with the payload) +
`engine_emit_event_no_longer_returns_e_primitive_not_implemented`
(rust-side; negative pin — verifies the deferred sentinel is GONE) +
`packages/engine/test/onChange_onEmit.test.ts` (TS-side end-to-end
through the napi cdylib; G19-B wave-7 onEmit callback splats args
correctly + onChange splat + retired-marker-grep tests). The `engine.ts::emitEvent` wrapper drops the "deferred"
verbiage in its docstring and surfaces the EmitBroadcast direct path.

### 7.9 TS-surface-parity sweep (Edge interface phantom `cid` + dropped `properties`; broader latent pre-Phase-2b TS-side drift) — CLOSED IN G19-D WAVE-7

**Phase-3 G19-D wave-7 RESOLUTION:** Edge interface fix landed (drop phantom `cid`; add `properties?: Record<string, JsonValue>`). Rust-side schema-parity meta-test landed at `crates/benten-engine/tests/ts_surface_parity_meta_test.rs` walking the napi `edge_to_json` producer keys against the TS Edge interface field set. Synthetic-drift fixture rejection (`tests/ts_surface_parity_meta_test_rejects_synthetic_drift_fixture`) defends against silent-no-op meta-test failure (pim-2 §3.6b). TS-side compile-time pin at `packages/engine/test/edge_interface.test.ts`. The structural defense closes the Edge phantom-cid recurrence vector at the test layer.

**Phase 2b state (preserved for narrative):** R6-R4 producer/consumer-deep-sweep-redux surfaced a pre-Phase-2b TS-surface drift candidate that is OUT-OF-SCOPE for the Phase-2b-close tag (named-destination-here per HARD RULE rule (b) + foundational `feedback_no_defer_HARD_RULE`):

- `packages/engine/src/Edge` interface (`packages/engine/src/types.ts::Edge`) declares `{ cid: string, source, target, label }` — 4 fields. The napi producer at `bindings/napi/src/edge.rs::edge_to_json` emits `{ source, target, label, properties? }` — 4 fields with TWO mismatches: (a) the TS interface declares `cid: string` but the napi producer never emits a `cid` field on the edge JSON (any TS caller reading `edge.cid` gets `undefined` at runtime); (b) the TS interface OMITS `properties` while the napi producer emits it when present (any TS caller wanting `edge.properties` hits a TS compile error).
- Origin: PR `3fc5262` `fix(dx)` from 2026-04-19 (Phase-2a R6 DX work, NOT Phase-2b). Preserved through every Phase-2b R5 wave + every R6 round (R6-R1 / R6-R2 / R6-R3 deep-sweep / R6-R3 narrow-iteration) without surfacing because the existing producer/consumer audits walked the producer-emits-field-vs-consumer-drops-field shape; the Edge case is the INVERTED shape (consumer-declares-field-vs-producer-doesn't-emit-it) which the Phase-2b-bounded sweeps did not target.
- Behavioral consequence in Phase 2b: zero packages/engine/test/ exercise either `edge.cid` or `edge.properties`, so no test fails today; but any user-code TS caller that consults `edge.cid` for content-addressing or expects `edge.properties` for graph-shape introspection silently mis-behaves.

**Phase 3 target:** A one-shot exhaustive TS-interface-vs-Rust-producer-shape sweep across `packages/engine/src/types.ts` + `bindings/napi/src/`. Mechanical procedure:

1. Enumerate every `pub struct` / serde-derived `pub enum` in `bindings/napi/src/*.rs` that flows to JS via napi.
2. For each, walk the corresponding TS interface in `packages/engine/src/types.ts` and assert field-for-field parity (modulo by-design omissions like `Node.anchor_id` per `#[serde(skip)]` on the `anchor_id` field of `crates/benten-core/src/lib.rs::Node` Phase-1 architectural decision).
3. Document each by-design asymmetry with a `// (intentionally NOT mirrored: <reason>)` line so future sweeps don't re-flag.
4. Fix all unintentional drift inline (likely `Edge.cid` removal + `Edge.properties` addition; possibly other instances surfaced by the sweep).
5. Add a Rust-side schema-parity meta-test (analogous to `manifest_schema_parity_pin.rs`) that walks the napi struct surface + asserts every public field has a TS-side counterpart by reading the dist `.d.ts` at test time, so the SAME drift cannot recur silently.

**Why Phase 3:** Out-of-scope for Phase-2b R6-R4 close (R6-R4 lens scope is post-R6-R3-FP delta, not pre-Phase-2b legacy); the broader TS-surface-parity work bundles cleanly with the Phase-3 first-wave CI-hygiene cluster (`§7.3.A`) since both surfaces want the same TS-side audit infrastructure. Out-of-the-band of Phase-2b's "21-now-bumped-to-21-or-22 producer/consumer drift instances" running tally because the legacy drifts predate the methodology.

**Cross-references:**
- `.addl/phase-2b/r6-r4-producer-consumer-deep-sweep.json` — surfacing finding (`near_findings_examined_and_dismissed.candidate.Edge interface`).
- `bindings/napi/src/edge.rs::edge_to_json` — Rust producer.
- `packages/engine/src/types.ts::Edge` — TS consumer (drift surface).
- `crates/benten-core/src/lib.rs::Node` — by-design `#[serde(skip)]` on the `anchor_id` field; precedent for documenting intentional omissions.

**Touch size:** ~80-150 LOC across `packages/engine/src/types.ts` (interface parity edits) + 1 Rust meta-test pin (~50-80 LOC) + cross-target pre-flight sweep. Risk surface: low — the additions are typed-surface widenings that existing TS callers don't depend on (zero current consumers).

### 7.10 SUBSCRIBE handler-id-router + DSL-args-vs-eval-properties parity meta-test — CLOSED IN G19-D WAVE-7 (handler-id-router seam wired in G14-D wave-5a; meta-test landed in G19-D wave-7)

**Phase-3 G19-D wave-7 RESOLUTION:** SUBSCRIBE handler-id-router seam was wired in G14-D wave-5a per seq-major-8 (`crates/benten-eval/src/primitives/subscribe.rs::execute` lines 1295-1317); G19-D wave-7 restored the corresponding TS DSL surface field (`SubscribeArgs.handler?`) + landed the LOAD-BEARING DSL-args-vs-eval-primitive-properties parity meta-test at `crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs`. The meta-test walks every `*Args` interface in `dsl.ts` against the eval-side primitive's canonical keyspace + 4 consumer projections (mermaid producer + drift-detector + ChangeEvent translation + DSL helper modules). 6 Args translators landed in dsl.ts (translateReadArgs / translateBranchArgs / translateIterateArgs / translateTransformArgs / translateCallArgs / translateRespondArgs). 5 synthetic-drift-fixture rejection meta-meta tests defend against silent-no-op meta-test failure (pim-2 §3.6b end-to-end). DSL-SPECIFICATION.md worked example for SUBSCRIBE handler-id-router published at `docs/DSL-SPECIFICATION.md`. The structural defense converges the 24-instance long-tail recurrence at the structural layer.

**Phase 2b state (preserved for narrative; as of R6-R4 narrow-iteration close):**

R6-R4 narrow-iteration producer/consumer-deep-sweep surfaced the 21st p/c drift instance: `SubscribeArgs.handler` was declared in the TS DSL and actively written to the SubgraphNode props bag (`packages/engine/src/dsl.ts` SubgraphBuilder + CaseBuilder), but the eval-side primitive at `crates/benten-eval/src/primitives/subscribe.rs::execute` reads ONLY the `pattern` property — never `handler`. PR #74's r6-r4-cr-1 fix mirrored an assumed EMIT precedent too literally; in practice neither EMIT nor SUBSCRIBE routes on a handler-id today (EMIT routes on `channel` name match; SUBSCRIBE on `pattern` match).

**Phase 2b resolution (orchestrator-direct, post-R6-R4-FP):** removed `handler?: string` from the `SubscribeArgs` interface in dsl.ts + DSL-SPECIFICATION.md + dropped the `props.handler = args.handler` write at both subscribe call sites. SUBSCRIBE today carries only `event` (mapped to eval-side `pattern`). The worked example at DSL-SPECIFICATION.md:557 cross-references this section.

**Phase 3 deliverables (LANDED at G14-D wave-5a + G19-D wave-7):**

1. **SUBSCRIBE handler-id-router LANDED** at G14-D wave-5a — the eval-side primitive at `crates/benten-eval/src/primitives/subscribe.rs::execute` carries the `handler?: string` arm that, when set, routes change-event delivery through the named handler instead of returning the raw event to the calling subgraph. (NOT mirroring EMIT — landed EMIT at `emit.rs::execute` + `EmitEvent` carry only `channel` + `payload`; subscribers route by channel-name match, not handler-id correlation. The Phase-3 SUBSCRIBE handler-id-router is a NEW routing dimension layered on top of the channel/pattern match.) The `SubscribeArgs.handler` field was restored in the TS DSL + DSL-SPECIFICATION.md worked example at G19-D wave-7.

2. **DSL-args-vs-eval-primitive-properties parity meta-test LANDED** at G19-D wave-7 (the structural fix the 4-deep-sweeps recurrence proved was needed):
   - Walks every `*Args` interface in `packages/engine/src/dsl.ts`
   - For each, finds the corresponding eval primitive at `crates/benten-eval/src/primitives/<primitive>.rs::execute`
   - Asserts every TS field that the DSL spreads to props is read by the eval primitive (and vice versa)
   - Bundled cleanly with §7.9's Rust-side schema-parity meta-test as authored (same TS-side audit infrastructure: reads the dist `.d.ts` at test time, walks the type definitions, asserts per-field correspondence against the Rust producer/consumer surface)

**Why Phase 3 (preserved for narrative):** Was out-of-scope for Phase-2b-close — handler-id-router was application-layer routing infrastructure that composed more naturally with the Phase-3 cross-actor SUBSCRIBE delivery work (sync deltas + broadcast widening). The structural meta-test belonged in the same Phase-3 first-wave CI-hygiene cluster as §7.9 + §7.3.A.

**Cross-references:**
- `.addl/phase-2b/r6-r4-narrow-iteration-producer-consumer.json` — surfacing finding (21st p/c drift)
- `.addl/phase-2b/r6-r3-fp-mr-group-b.json` — original mirror-EMIT-precedent context (PR #66 EMIT routing)
- §7.9 — sibling Rust-side schema-parity meta-test recommendation (same infrastructure can cover both)
- `crates/benten-eval/src/primitives/subscribe.rs::execute` — eval-side surface (extended at G14-D wave-5a)
- `crates/benten-eval/src/primitives/emit.rs` — EMIT today (channel/payload only, no handler-id) — handler-id-router currently SUBSCRIBE-only; cross-primitive widening to EMIT remains a future-phase concern.
- `crates/benten-engine/tests/dsl_args_vs_eval_properties_parity_meta_test.rs` — landed parity meta-test (G19-D wave-7).

**Touch size (as authored):** ~30-50 LOC handler-id-router wiring + ~20 LOC DSL restoration + ~80-150 LOC parity meta-test (combined with §7.9 sibling work, ~150-250 LOC for both meta-tests).

---

### 7.11 pim-N process-pattern ratifications — STATUS: ALL CODIFIED INLINE (only pim-6 CI-infrastructure half remains)

**Phase 2b state (closed at tag `phase-2b-close`):** R6 Rounds 3-5 pattern-induction meta-sweeps surfaced 9 process-level patterns. ALL 9 are now codified inline in `dispatch-conventions.md` as standing rules. Codification map:

| pim-N | Name | Codified inline at |
|---|---|---|
| pim-1 | Doc-lag-on-code-fix | §3.5b post-fix doc-coupling pre-flight (HARDENED 2026-05-03 with grep-symbol-verification + high-churn-surface MUST + NEW-prose-block grep) |
| pim-2 | Closed-claim with non-end-to-end test pin | §3.6b end-to-end load-bearing test pin |
| pim-3 | Round-2 lens-budget surface clustering | §3.9 R2 lens-menu correctness coverage |
| pim-4 | Wave-8 sibling-wave lock-in | §3.10 wave-pairing protocol |
| pim-5 | Mini-review verdict 'READY-TO-MERGE-WITH-X' comma-clause | §3.8 mini-review verdict shape |
| pim-6 | Cross-crate workflow blind spot (constraint-assertion side) | §3.4b cross-crate workflow-constraint exception |
| pim-7 | Stable rustdoc strict-lint blind spot | §3.5 dimension #5 (stable-doc-leg) |
| pim-8 | Mirror-precedent overshoot | §3.6c mirror-precedent overshoot guard |
| pim-9 | Incidental cites in NEW prose blocks | §3.5b point 3 promotion + point 4 NEW-prose grep |
| pim-10 | Narrow-iteration cycle as effective FP follow-up (POSITIVE) | §3.7b narrow-iteration cycle |
| pim-11 | Reviewer-assumption-of-translation-layer-without-verification | §3.6d reviewer translation-layer cite-discipline |

**Sole remaining Phase-3 residual — pim-6 CI infrastructure half:** the dispatch-conventions §3.4b sub-rule covers the per-implementer side (run workspace check when adding a constraint-assertion). The CI-infrastructure question — should drift-detector additions automatically trigger a workspace-wide regression scan in CI without orchestrator intervention? — is a Phase-3 CI-engineering decision (fits the Phase-3 plan-doc opening checklist §8 row 1). Touch size: 1 CI workflow update (~30-50 LOC) + decision on regression-scan cadence + cost.

**Cross-references:**
- `.addl/phase-2b/r6-r3-pattern-induction-meta-sweep.json` — pim-3 / pim-4 / pim-5 origin findings
- `.addl/phase-2b/r6-r4-pattern-induction-meta-sweep.json` — pim-6 / pim-7 origin findings
- `.addl/phase-2b/r6-r5-pattern-induction-meta-sweep.json` — pim-8 / pim-9 / pim-10 origin findings
- `.addl/phase-2b/r6-r5-narrow-pim-meta.json` — pim-11 origin finding
- `.addl/phase-2b/dispatch-conventions.md` — full codification (gitignored; orchestrator-side standing rules)

**Touch size for the residual:** ~30-50 LOC CI workflow + decision capture. NOT urgent (the per-implementer §3.4b discipline already covers the day-to-day case; CI infra is automation on top).

---

### 7.11b Phase-3 R4b pim-N candidates (R6-ratification queue) — **PHASE-3-CLOSE-BLOCKING (pre-tag R6 ratification)**

**Origin:** Phase-3 R4b returned 5 BLOCKERs + 11 MAJORs + 19 MINORs across 7 lenses (2026-05-08). Multiple lenses independently named pim-N candidates beyond the 11 codified in Phase-2b. Listed here as named destination for R6-ratification per HARD RULE rule-12 (so they have a real home before R6 phase-close convergence council ratifies). Final codification target: `dispatch-conventions.md §3.X` once R6 lens-corroboration confirms the pattern. **All 5 pim-N candidates listed here MUST land at R6 ratification before tag (orchestrator-direct dispatch-conventions edits + memory file authoring).**

**pim-12 candidate — R3-staged RED-PHASE pin → R5 un-ignore expected → R5 implementer skipped under wave-time pressure.**
- Independently named by 3 R4b lenses (cryptography r4b-major-1 + capability-system r4b-cap-1 + wasmtime r4b-wsa-1/2). Architect r4b-1 + dist-systems ds-r4b-1 corroborate at MINOR/BLOCKER respectively.
- 5-instance class across phases (Phase-1 R4 vacuous projection / Phase-1 R6→R7 catalogued-but-unfired error codes / Phase-2a G1-A false cargo green / Phase-2b R4b structural-vs-runtime gap / Phase-3 R4b RED-PHASE pins not un-ignored).
- Fix path: codify at §3.6e or §3.5b new sub-rule. Wave-completion checklist MUST include un-ignore audit of all RED-PHASE pins citing the wave as un-ignore target. Reviewer briefs MUST verify landing-status + production-arm-presence, not just spec-pin presence.

**pim-13 candidate — R7-equivalent spec-to-code-compliance audit at every phase-close as standing pattern.**
- Origin: Phase-1 R7 invented this discipline; Phase-1 retrospective claims it was retired post-Phase-1 because "discipline merged into R6 verify-don't-trust-docs." Phase-3 R4b cluster suggests **the discipline has lapsed** — Phase-2b R6 was multi-round lens-based, not spec-to-code-walk; Phase-3 R6 will follow Phase-2b shape unless reactivated.
- Fix path: revive R7-equivalent as standing companion to R6 phase-close convergence — walks every spec doc claim + every named compromise + every D-PHASE-N item to a code construction site at HEAD. Discipline lives in pim-13 §3.X codification + a `tools/spec-compliance-audit/` skill.

**pim-2-amendment — Per-finding granularity for closure pins AND deferral destinations.**
- Origin: Phase-3 R4b pattern-induction-meta-sweep r4b-pim-2-amendment (MAJOR forward-looking codification). 3+-recurrence threshold MET at HEAD (G21-T2 MAJOR-7 registry-clear-on-leave fix landed without per-finding pin + audit-6-1 wrong-surface coverage + audit-6-3 umbrella-section phantom-destination).
- Fix path: codify §3.6b sub-rule 4 (~30-50 LOC dispatch-conventions edit) + FIX-NOW 3 specific instances (~50-70 LOC). Total batch ~80-120 LOC.

**pim-18 candidate — SHAPE-not-SUBSTANCE 4-of-4 wave threshold.**
- Origin: Phase-3 R5 G14-C / G15-A / G18-A / G17-A1 (the "4th SHAPE-not-SUBSTANCE 4-of-4 wave threshold" recurrence). Strengthened to 10-datapoint trend at R4b pattern-induction (8 consecutive zero-incident handoffs after G16-A→G16-C streak).
- Fix path: codify §3.6e — implementer briefs MUST include "production call site enumeration" pre-flight item.

**pim-codification-feedback-loop candidate — codification of standing rule doesn't auto-propagate to in-flight agent briefs.**
- Origin: NS-T52 candidate from Phase-3 R4-close + 1 instance from G20-B fix-pass §3.5g-miss + 1 self-reference instance at R6 R1 (the R6 R1 ratification batch's in-flight R4b-residual fix-pass + sibling R6 R1 lenses operated against pre-ratification rules). Sub-threshold (1-2 → 2-3 instances; STILL sub-3-recurrence; track only).
- Fix path: when a new pim-N is codified, sweep in-flight agent briefs + send updates to active agents OR explicitly note "this codification applies to NEW dispatches only."
- **Status 2026-05-09 (R6 R1 sub-threshold-track-only verdict):** sub-3-recurrence; codify if/when 3rd recurrence fires.

**pim-cryptographic-attestation-transport-reuse candidate — wire-shape needs trust-model closure → COMPOSE existing hardened primitives, NOT introduce parallel unsigned transport.**
- Origin: G16-D wave-6b fp V2 envelope (parent-signed attestation + Acceptor + FreshnessPolicy + payload-hash binding composed at the wire boundary instead of introducing parallel unsigned transport). Named in g16d6b-fp-corr mini-review as WORTH_R6_RATIFICATION_AS_pim-13_CANDIDATE; renumbered sub-threshold-track-only at R6 R1.
- 1 instance currently. Sub-3-recurrence threshold; ratify only if 2 more instances surface in Phase-4+ networking work (heterogeneous-cap-envelope per §6.12 item 9, peer-DID rotation envelopes, handler-attestation envelopes).
- Fix path: codify §3.6 extension (or new §3.6g) when 3rd recurrence fires, naming the COMPOSE-existing-hardened-primitives-at-wire-boundary discipline.
- **Status 2026-05-09 (R6 R1 sub-threshold-track-only verdict):** track at this entry; codify if/when 3rd recurrence fires.

**pim-atomic-multi-MAJOR-fix-pass-scope-expansion candidate — N findings sharing canonical-bytes shape → atomic landing preferred over wave-splitting.**
- Origin: G16-D wave-6b fp (4 findings: DID forgery + replay + frame-pair-binding + version validation, all sharing one canonical-bytes shape). Fix-pass scope expanded from ~200-400 LOC to ~850 LOC; JUSTIFIED-NOT-GOLD-PLATING because splitting would ship an unstable intermediate V1.5 shape on disk.
- 1 instance currently. Sub-3-recurrence threshold; track at this entry.
- Fix path: codify §3.6 extension when 3rd recurrence fires, naming the atomic-landing-when-shared-canonical-bytes-shape discipline. Composes with `feedback_subtrack_sizing_heuristic` + `feedback_canary_first_parallel_implementation`.
- **Status 2026-05-09 (R6 R1 sub-threshold-track-only verdict):** track at this entry; codify if/when 3rd recurrence fires.

**fix-pass-mini-review-json-schema candidate — fix-pass mini-reviews demonstrate higher structural depth than R5 group-implementation mini-reviews.**
- Origin: g16d6b-fp-corr mini-review JSON structure (`prior_findings_closure` + `would_fail_if_no_opd` 4-of-4 axes + `section_3_5g_4_surface_atomic_update` 5-of-5 surfaces + `section_3_5b_HARDENED_post_fix_doc_coupling` + `loc_scope_validation` + `phantom_destination_check` + `criterion_X_closure_verdict` + `lens_focus_coverage`). 1-2 instances at fix-pass depth.
- Sub-3-recurrence threshold; track at this entry. Codify at dispatch-conventions §3.8 extension if/when 3rd fix-pass exhibits the same structural depth.
- **Status 2026-05-09 (R6 R1 sub-threshold-track-only verdict):** track at this entry; codify if/when 3rd recurrence fires.

**Cross-references:**
- `.addl/phase-3/r4b-pattern-induction.json` (pattern-induction-meta-sweep R4b output naming r4b-pim-2-amendment + pim-18 explicitly)
- `.addl/phase-3/r4b-{cryptography,capability-system,wasmtime-sandbox}.json` (3-lens corroboration of pim-12 candidate)
- `.addl/phase-3/r6-r1-pattern-induction.json` (R6 R1 ratification recommendations naming the 4 ratify-inline pim-Ns + 4 sub-threshold-track-only candidates)
- `docs/history/PHASE-1.md §5` (R7 origin) + `docs/history/PHASE-2b.md §5` (pim-1 through pim-11 catalog)
- Memory `feedback_pattern_induction_meta_sweep.md` (load-bearing operational tier; companion to known-pattern reduxes).

**Touch size:** orchestrator-direct dispatch-conventions §3.X edits (~150-300 LOC across pim-12 / pim-13 / pim-18 / pim-2-amendment / pim-codification-feedback-loop) at R6 ratification + memory file authoring per pim if needed.

**Status 2026-05-09 (R6 R1 ratification batch CLOSED for the 4 ratify-inline candidates):** pim-2-amendment codified at dispatch-conventions §3.6b sub-rule 4 + memory `feedback_pim_2_amendment_per_finding_granularity`; pim-12 codified at dispatch-conventions §3.6e + memory `feedback_pim_12_red_phase_staged_pin_un_ignore_discipline`; pim-13 codified at dispatch-conventions §3.12 + memory `feedback_pim_13_r7_spec_to_code_compliance_audit`; pim-18 codified at dispatch-conventions §3.6f + memory `feedback_pim_18_shape_not_substance_pre_flight`. The 4 sub-threshold-track-only candidates (pim-codification-feedback-loop + pim-cryptographic-attestation-transport-reuse + pim-atomic-multi-MAJOR-fix-pass-scope-expansion + fix-pass-mini-review-json-schema) remain at this §7.11b entry pending 3rd-recurrence fire.

---

### 7.12 Workspace-aware numeric-claim source-of-truth — CLOSED at Phase-3 R5 wave-9 W9-T4

**Closure shape (Phase-3 R5 wave-9 W9-T4, 2026-05-07):** the `crates` row in `tools/cite-drift-detector/src/lib.rs::numeric_claims_source_of_truth` is now derived dynamically from the workspace's `Cargo.toml` `members =` table via `tools/cite-drift-detector/src/lib.rs::derive_crate_count_from_workspace`. The new function reads `<root>/Cargo.toml`, parses the `[workspace] members = [...]` array, and counts entries whose path begins with `crates/`. `numeric_claims_source_of_truth_at(root)` exposes the path-aware variant; `numeric_claims_source_of_truth()` invokes it against `Path::new(".")` for callers that operate from the workspace root. Fallback when `Cargo.toml` is unparseable: returns the historical static value 10 so the lint stays operational.

**Phrase-source-of-truth scope:** `primitives` (12) + `invariants` (14) remain hardcoded — their authoritative source is documentation (CLAUDE.md baked-in commitments), not workspace structure. Only the `crates` row was suitable for workspace-derivation.

**Test coverage landed at W9-T4:**
- `derive_crate_count_synthetic_workspace_with_n_crates` — plants a synthetic 7-member Cargo.toml + asserts derivation returns 7.
- `derive_crate_count_excludes_non_crate_paths` — workspace with only `tools/`/`tests/`/`bindings/` rows derives 0.
- `derive_crate_count_returns_none_on_missing_cargo_toml` — clean `None` propagation.
- `derive_crate_count_returns_none_on_unparseable_cargo_toml` — malformed-TOML `None` propagation.
- `derive_crate_count_returns_none_when_workspace_table_missing` — single-crate Cargo.toml (no `[workspace]`) returns `None`.
- `numeric_claims_at_root_uses_derived_count` — end-to-end: synthetic workspace with 3 members makes the `crates` claim's `value` == 3.
- `numeric_claims_at_root_falls_back_to_static_when_cargo_unparseable` — confirms the 10-fallback when derivation fails.

**Touch size at W9-T4:** ~70 LOC derivation function + ~120 LOC tests + ~10 LOC rustdoc edits + 1 workspace-dep add (`toml` already in workspace deps; pulled into `tools/cite-drift-detector/Cargo.toml`).

**Origin (preserved for retrospective):** the cite-drift detector's `numeric_claims_source_of_truth()` table previously hardcoded `crates: 10` (bumped 8 → 10 at orchestrator-direct cleanup 2026-05-05 closing pim-12 NEW shape iii at the immediate-fix arm). The hardcode bump closed the false-positive flood (28 findings against correctly-10-crate Phase-3 docs); the recurrence-resistant arm — parsing `Cargo.toml` `members =` at runtime + counting `crates/*` entries dynamically — was deferred here and is now closed.

**What landing requires:**
- Replace the hardcoded `value: 10` for the `crates` claim with a derivation function that:
  1. Reads `<workspace_root>/Cargo.toml`.
  2. Parses the `[workspace] members = [...]` table.
  3. Counts entries whose path starts with `crates/` (excludes `bindings/`, `tests/`, `tools/` per the existing rustdoc-stated rule).
  4. Returns that count as the authoritative `value`.
- Decision: keep the `NumericClaim::value` field as a `u32` (current shape) and have the source-of-truth function compute the count once at startup, OR change the API so each claim's `value` can be a closure / derivation. The simpler path is option 1 (compute-once): `numeric_claims_source_of_truth()` invokes `derive_crate_count_from_workspace()` and embeds the result.
- Add a unit test that plants a synthetic `Cargo.toml` with N `crates/foo-N` rows under tempdir + asserts the derivation returns N.
- Update the `crates` rustdoc to point at the derivation as the authoritative source rather than the table comment.
- Remove the "When a Phase-3 group changes these counts, that group's brief MUST update this table" half of the rustdoc that applies to crates — the derivation makes that step unnecessary for the crates row (still applies to `primitives` + `invariants` which remain hardcoded).

**Why deferred (not done at orchestrator-direct cleanup):** the cleanup PR scope was the cross-cutting tracked-file fix. Adding a derivation function + tempdir-based unit test is ~80-150 LOC + 1 new dependency surface (TOML parsing — `cargo_toml` crate or `toml` direct) that warrants its own fix-pass review. The hardcode bump fully resolves the immediate finding (28 false positives gone); the derivation upgrade is the recurrence-resistance arm.

**Touch size:** ~80-150 LOC (derivation fn + 1 test + rustdoc edits + Cargo.toml dep add). Risk surface: low — additive, gated behind the same source-of-truth fn the existing tests already validate.

**Cross-references:**
- `.addl/phase-2b/dispatch-conventions.md::§3.5c amendment 2026-05-05 — NEW shape (iii) tools-as-meta-spec`
- `.addl/phase-3/r4-r1-pattern-induction.json` (pim-12 4th-instance + NEW shape iii origin finding)
- `tools/cite-drift-detector/src/lib.rs::numeric_claims_source_of_truth` (current hardcode site)
- `tools/cite-drift-detector/tests/numeric_claim_drift_lint_finds_known_drift_fixture.rs` (companion fixture; tracks the lint mechanism not the truth values, so the derivation upgrade is transparent to it)

---

### 7.12b Cite-drift detector coverage envelope — CLOSED at Phase-3 R5 wave-9 W9-T4

**Closure shape (Phase-3 R5 wave-9 W9-T4, 2026-05-07):** the `tools/cite-drift-detector/src/lib.rs::walk_doc_inputs` walker now covers:
- `README.md` (root)
- `docs/**/*.md`
- `.addl/**/*.md` (when present locally; gitignored — CI sees `docs/` only)
- **`crates/**/*.rs`** — fully recursive, automatically covers `crates/*/src/`, `crates/*/tests/`, and `crates/*/build.rs` (the walker matches by extension and walks the full subtree). Pinned by `walker_includes_crates_build_rs_and_tests` test.
- **`tools/**/*.rs`** — orchestrator-side tooling (e.g. `tools/bench-wat-rebake/src/lib.rs`, `tools/benten-dev/src/`); pinned by `walker_includes_tools_subtree` test. The detector's own subtree (`tools/cite-drift-detector/`) is excluded so its intentional cite-shaped fixture strings don't double-count as real cites; pinned by `walker_skips_target_subtree_under_tools` (companion test demonstrates target/ skip uniformity).
- **`.cargo/*.toml`** — alias-comment blocks (the walker's `extract_line_cites` already accepts `.toml` extension); pinned by `walker_includes_dot_cargo_config_toml` test.
- `packages/engine/src/**/*.ts` (doc-comment cites)

**Symbol-cite re-export recognition:** the cite-drift-detector's `target_text_defines_symbol` was extended at W9-T4 to recognise `pub use` re-exports (single-line `pub use <path>;` shape, single-line brace lists, and multi-line brace lists). This eliminates a class of false positives where a crate root re-exports types from sibling modules and a cite to the re-export site at `lib.rs` was previously flagged because no in-place `struct` / `enum` definition exists in the re-exporting file.

**Touch size at W9-T4:** ~25 LOC walker extensions + ~50 LOC re-export recognition function + ~80 LOC tests + rustdoc edits.

**Origin (preserved for retrospective):** G17-B PR #116 mini-review finding `g17-b-mr-cite-drift-detector-coverage-7` (BELONGS-ELSEWHERE-SPECIFICALLY → NAMED entry per HARD RULE clause-b). The detector previously caught 2 of 7 instances of the original `Cargo.toml:309` cite drift; the slipped 5 sites lived outside the tracked-path envelope (now closed). Pim-1 §3.5b BULK-APPLICATION HARDENED is the standing human-discipline defense; the cite-drift-detector is the automated companion. Both dimensions of incompleteness (human pre-flight + detector envelope) interact, so widening the envelope narrows the blast radius of any single human pre-flight miss.

---

### 7.13 Phase-3 attack-surface matrix authoring (sec-r4r2-2 / sec-r4r1-4 matrix-prose half) — CLOSED at Phase-3 R5 wave-9 W9-T2

**Origin:** R4-R2 security-auditor lens finding `sec-r4r2-2` MAJOR (escalation of R4-R1 `sec-r4r1-4` MAJOR; root R1 finding `sec-r1-7`). The R1 lens cited that "enumerating attack vectors ahead of implementation" was the discipline that gave the Phase-2b ESC matrix its structural value. Phase-3 has TWO halves to that work:

1. **Concrete-vector test pins** (test-pin-enumeration). Closed at R4-R2-FP/B via 3 RED-PHASE pins in `crates/benten-sync/tests/`:
   - `attack_loro_op_log_inv_13.rs::loro_merge_op_log_violating_inv_13_immutability_rejected_at_dispatch_not_just_at_cid_divergence`
   - `attack_mst_diff_cid_mismatch.rs::mst_diff_entry_with_cid_byte_mismatch_rejected_at_application_layer`
   - `attack_hlc_skew_revocation_ordering.rs::hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded`

2. **Matrix-prose meta-document** (the doc-level enumeration of all Phase-3 attack surfaces). DEFERRED to this entry as a Phase-3-close R6 hardening surface (NOT pre-R5 / not gating R5 implementation).

**DISAGREE-WITH-EXPLANATION rebuttal of the R1 "must land before R5" framing:** the matrix's role is meta-completeness at R6 phase-close (a checklist that every named attack surface has at least one test pin driving it), NOT the R5 implementation target itself. The Phase-2b ESC matrix's effectiveness came from per-vector test enumeration (closed by half (1) above), not from matrix-as-doc presence at R5 dispatch time. The matrix-prose document is a **completeness audit** running over R5 corpus, not a **plan input** that R5 implementers consume. Item (1) is the load-bearing deliverable for R5-time defense; item (2) is the load-bearing deliverable for R6-time completeness. The two halves are separable.

**Closure (Phase-3 R5 wave-9 W9-T2):** matrix authored as `docs/ATTACK-SURFACE-MATRIX.md` (tracked-tree destination, NOT `.addl/` per the W9-T2 brief decision — keeps the matrix visible to OSS contributors + cite-drift detector coverage). Two-part structure:

1. **Part 1 — Phase-2b SANDBOX ESC matrix** (re-issued for cross-reference; authoritative status table remains `docs/SECURITY-POSTURE.md` Compromise #4 to avoid two-source drift).
2. **Part 2 — Phase-3 P2P-sync attack surfaces** with the 6 sub-sections enumerated (§2.1 Atrium peer-handshake / §2.2 UCAN proof-chain / §2.3 sync-replica trust-boundary / §2.4 device-DID attestation / §2.5 iroh-relay metadata / §2.6 Atrium join + revocation-ordering / §2.7 iroh peer-id derivation).

For each surface row, the matrix cites the file:symbol test pin (per dispatch-conventions §3.5b HARDENED). All cited test pins were grep-verified to exist at HEAD before merge. Audit-cycle instructions for R6 phase-close completeness checks live at the doc's "Audit instructions" section. Cross-reference added to `docs/SECURITY-POSTURE.md` under the close compromise table.

**Touch size (actual):** ~280 LOC matrix doc + 10 LOC cross-ref edit in `docs/SECURITY-POSTURE.md`. Within plan envelope.

**Cross-references:**
- Phase-3 R4 R2 security lens finding `sec-r4r2-2` (origin finding + DISAGREE narrative); see `.addl/phase-3/r4-r2-security.json` (gitignored; orchestrator-tree only)
- Phase-3 R4 R1 security lens finding `sec-r4r1-4` (R1 escalation); see `.addl/phase-3/r4-r1-security.json` (gitignored; orchestrator-tree only)
- Phase-3 R1 security lens finding `sec-r1-7` (root R1 finding); see `.addl/phase-3/r1-security.json` (gitignored; orchestrator-tree only)
- Phase-3 implementation plan §6 line 852 (current implicit-deferral; replace with reference to this entry on next plan-doc edit pass); see `.addl/phase-3/00-implementation-plan.md` (gitignored; orchestrator-tree only)
- `crates/benten-sync/tests/attack_loro_op_log_inv_13.rs` + `attack_mst_diff_cid_mismatch.rs` + `attack_hlc_skew_revocation_ordering.rs` — 3 R4-R2-FP/B concrete-vector pins

---

### 7.14 WAIT TTL GC tokio-interval backstop production wiring (G20-A2 wave-8a mr-6)

**Phase-3 G20-A2 wave-8a state:** The WAIT TTL GC machinery (`crates/benten-engine/src/wait_ttl_gc.rs`) ships THREE sweep paths per the D12 hybrid-GC contract: (1) event-driven on every suspend / resume, (2) interval backstop, (3) drop-final on `Engine::drop`. Paths (1) + (3) are wired in production today via `engine_wait.rs::call_with_suspension` / `call_as_with_suspension` + `Engine::drop`. Path (2) is **DOCUMENTED but NOT PRODUCTION-WIRED** — the interval-backstop sweep is invoked only via the test-only helper `Engine::testing_run_wait_ttl_gc_pass`. A production engine that suspends one entry and then sits idle (no further suspend / resume traffic + no shutdown) leaves the expired entry in the SuspensionStore until the next suspend / resume / shutdown event.

**Why this is non-blocking for v1-foundation:** the resume-time deadline check at `engine_wait.rs::resume_from_bytes_inner` (Step 1.5) consults `wait_ttl_gc::is_expired` against persisted `WaitMetadata` and fires `E_WAIT_TTL_EXPIRED` independently of whether GC has reaped the entry yet. The deadline-on-resume check is the LOAD-BEARING correctness mechanism; the GC sweep is a STORAGE-CLEANUP mechanism. An idle engine with un-reaped expired entries does NOT permit the entries to resume successfully — they just consume disk until the next sweep fires.

**Phase-3 target:** Wire a tokio interval task at `EngineBuilder::build` (or `Engine::new`) that calls `crate::wait_ttl_gc::run_interval_tick` on a configurable cadence (default 1h per the wait_ttl_gc.rs module doc; tunable via `EngineBuilder::wait_ttl_gc_interval(Duration)`). The task must:
- Run on the engine's tokio runtime (introducing tokio as a Phase-3 engine-side runtime dependency if it isn't already present in the build path used for the engine surface).
- Hold a `Weak<EngineGeneric<B>>` so the task auto-shuts-down when the engine drops (Drop's final sweep is the authoritative shutdown path).
- Be suppressible via `EngineBuilder::wait_ttl_gc_interval(Duration::ZERO)` for test scenarios that drive the interval synchronously via `testing_run_wait_ttl_gc_pass`.

**Touch size:** ~80-150 LOC engine-side + ~30-50 LOC tests. Risk surface: low — purely additive (the GC contract today is correct without it; this entry hardens the storage-cleanup discipline for idle-engine workloads).

**Cross-references:**
- `crates/benten-engine/src/wait_ttl_gc.rs` module doc (lines 17-22): cites the production-wiring intent + names this entry as the destination.
- `crates/benten-engine/src/engine.rs::EngineGeneric::drop` + companion comment: documents the (3) drop-final sweep is wired.
- `crates/benten-engine/src/engine_wait.rs::call_with_suspension` / `call_as_with_suspension`: documents the (1) event-driven sweep is wired.
- G20-A2 wave-8a mini-review finding `g20-a2-mr-6` (origin); see `.addl/phase-3/r5-w8a-g20-a2-mini-review.json`.

---

### 7.15 WAIT TTL property-test case-count + pure-eval-layer sibling (G20-A2 wave-8a mr-7)

**Phase-3 G20-A2 wave-8a state:** `crates/benten-eval/tests/proptest_wait_ttl.rs::prop_wait_ttl_no_silent_expiry_in_resume` ships at 256 cases per iteration (~80ms each → ~25s total). The R2 spec target was 10k cases. The current proptest drives the FULL engine boundary (`Engine::builder().path().build()` per iteration); 256 cases samples ~1.6% of the actual `(ttl_hours ∈ [1, 720]) × (offset_hours ∈ [0, 2000])` input space (~1.4M pairs). The property under test (deadline-on-resume vs wall-clock advance) is correct at the eval-layer (`benten_eval::resume_with_meta` consumes `WaitMetadata` directly without engine-boundary cost); a pure-eval-layer sibling proptest could feasibly hit 10k cases at ~0.1ms per iteration (~1s total).

**Phase-3 target:** Land a sibling proptest at `crates/benten-eval/tests/proptest_wait_ttl_pure_eval.rs` (or similar) at 10k cases that:
- Fabricates `WaitMetadata` directly (no engine, no SuspensionStore, no redb).
- Calls `benten_eval::resume_with_meta(Some(meta), WaitResumeSignal::DurationElapsed, Some(now_ms))`.
- Asserts the same property: `(now_ms - suspend_wallclock_ms) >= (ttl_hours * 3_600_000)` ↔ `EvalError::Host(WaitTimeout)` fires.

The engine-boundary proptest stays at 256 cases (load-bearing for cross-process correctness — drives the persistence + resume protocol's full-stack interactions); the pure-eval-layer sibling carries the high-iteration coverage.

**Touch size:** ~50-100 LOC test source. Risk surface: low — purely additive observer.

**Cross-references:**
- `crates/benten-eval/tests/proptest_wait_ttl.rs::prop_wait_ttl_no_silent_expiry_in_resume` (256-case engine-boundary proptest; load-bearing for cross-process semantics).
- G20-A2 wave-8a mini-review finding `g20-a2-mr-7` (origin); see `.addl/phase-3/r5-w8a-g20-a2-mini-review.json`.

---

## 7.3 Wave-8j R6 residuals — test bodies need real implementations before un-ignore

**Phase 2b state:** R6 phase-close Round 1 surfaced two `#[ignore]`'d tests with stale rationales — both have empty `todo!()` bodies that REFERENCE landed work but don't actually exercise it:

- `crates/benten-engine/tests/no_dsl_compiler_dep.rs` — 2 tests asserting `benten-engine` does NOT depend on `benten-dsl-compiler` + does NOT publicly expose `register_handler_from_str`. G12-B has merged; the architectural invariant the tests pin is real but the test bodies are `todo!()`. Need ~10-15 LOC each: parse `crates/benten-engine/Cargo.toml` via `toml::from_str` and assert dep entries, plus a public-API check via `cargo public-api` snapshot or a `benten_engine::Engine` reflection.
- `crates/benten-eval/tests/sandbox_wallclock.rs::sandbox_wallclock_per_handler_override_via_subgraphspec_primitives` — empty body. Wave-8b's primitive-level `execute()` accepts a SandboxConfig directly; the engine-side wire-through reading SANDBOX node's `wallclock_ms` property landed in 8c. Test body needs to construct a SubgraphSpec with a SANDBOX op carrying a `wallclock_ms` property + assert engine.execute uses that override at dispatch (~30-50 LOC).

R6 lens findings: `r6-arch-3` (no_dsl_compiler_dep.rs) + `r6-wsa-6` (sandbox_wallclock_per_handler_override).

**Phase 3 target:** Land both test bodies as part of the first Phase-3 wave's CI-hygiene pass. Both are mechanical once the supporting infrastructure they reference is observably stable across a Phase-3 implementation cycle. Could fold into earlier work if a Phase-3 wave incidentally re-touches benten-engine's Cargo.toml dep graph or the SANDBOX wallclock dispatch path.

**Why deferred (not fixed inline at R6-FP):** test bodies are non-trivial (~60 LOC combined; Cargo.toml parsing + SubgraphSpec construction + dispatch verification) and bundle naturally with broader Phase-3 CI work. Lifting `#[ignore]` without writing the bodies would surface as a `todo!()` panic at CI run-time.

**Touch size:** ~60 LOC test source. Risk surface: low — pure invariant assertions.

---

### 7.3.A — Wave-8j R6 Round 1 deep-sweep residuals (~85 stale-rationale `#[ignore]` test bodies)

**Phase 2b state:** R6 Round 1's deep-retrospective sweep (`r6-round-1-deep-sweep-stale-deferrals.md`) found ~85 additional `#[ignore]`'d tests beyond the 11 R6 Round 1 known instances. The pattern is uniform: TDD red-phase rationales like "pending G7-A" / "Phase 2b G12-E pending" — those targets all merged through `e2b1c62`, but the test bodies stayed `todo!()`. Wave-8j-followup-stale-deferrals (this entry) bulk-rewrote rationales to point at the named subsections below; bodies stay deferred to Phase 3 per §7.3's existing framing. Each sub-section enumerates its file:line set so a Phase-3 first-wave CI-hygiene pass can pick up the entire residual cluster as a single sub-track.

#### 7.3.A.1 — Runtime SANDBOX invariant + attribution-frame test bodies (G7-A/B/C structurally landed) — **CLOSED-IN-PHASE-3-G20-A1**

**Status (2026-05-07):** **CLOSED-IN-PHASE-3-G20-A1 wave-8a.** All 21 test bodies un-ignored + driving production runtime arms (`sandbox::execute` / `execute_with_live_cap_check` / `CountedSink` / `EscDefenseState`). Bodies authored at `crates/benten-eval/tests/{invariant_4_runtime,invariant_7_runtime,sandbox_attribution,sandbox_attribution_frame_security,sandbox_nested_dispatch,sandbox_named_manifest,sandbox_named_manifest_codegen_drift,sandbox_capability_intersection_at_init,sandbox_host_fn_trampoline_count,sandbox_depth_inheritance_regression,attribution_non_regression,proptest_sandbox_fuel,proptest_sandbox_isolation,proptest_sandbox_output}.rs` + integration tests at `crates/benten-eval/tests/integration/{inv_4_call_boundary,inv_7_streaming,sandbox_wasm32_disabled}.rs` (added `tests/integration.rs` aggregator) + engine-side at `crates/benten-engine/tests/integration/{engine_sandbox,sandbox_in_crud,stream_into_sandbox,sandbox_compile_time_disabled_on_wasm32}.rs` (un-gated from `phase_2b_landed`). Engine helper `testing_make_minimal_sandbox_spec()` added at `crates/benten-engine/src/testing.rs`. Inv-4 runtime arm exercised via depth-chain construction (companion to `inv_4_runtime_arm_fires_at_max_depth`). Inv-7 trap-loudly verified end-to-end through `log` host-fn cumulative-output trampoline counting.

**Phase 2b state (historical):** G7-A (`a9758f8`) + G7-B (`097d66f`) + G7-C (`468b3ab`) all merged with the structural surfaces in place; SANDBOX runs through wasmtime per-call. The `todo!()` bodies in this cluster pinned Inv-4 (sandbox depth runtime threading) + Inv-7 (output trap-loudly) + sec-pre-r1 closure claims. These overlapped with the SECURITY-POSTURE.md "Honest disclosure — Inv-4 runtime threading is structural, not transitive" section which recorded that `AttributionFrame.sandbox_depth` was constructed but the depth-counter machinery had no production call site (closed at R6FP-Group-1; covered end-to-end by G20-A1 wave-8a un-ignores).

**Files (all `#[ignore]`d, all `todo!()` bodies):**
- `crates/benten-eval/tests/invariant_7_runtime.rs` — 2 tests (Inv-7 CountedSink trap + no-silent-truncation default)
- `crates/benten-eval/tests/invariant_4_runtime.rs` — 3 tests (depth traps, depth-inherited-across-CALL-boundary, AttributionFrame depth)
- `crates/benten-eval/tests/sandbox_attribution.rs` — 1 test (sec-pre-r1-03 attribution frame threading)
- `crates/benten-eval/tests/sandbox_attribution_frame_security.rs` — 2 tests (D20 inheritance + sec-pre-r1-13 forward-compat)
- `crates/benten-eval/tests/sandbox_nested_dispatch.rs` — 3 tests (D19 catalog rename + sec-pre-r1-08 nested SANDBOX denial + D19 calibrated async)
- `crates/benten-eval/tests/sandbox_named_manifest.rs` — 1 test (TOML codegen drift surface, exercises build.rs)
- `crates/benten-eval/tests/sandbox_named_manifest_codegen_drift.rs` — 1 test (D2 hybrid + wsa D18 cap_recheck drift)
- `crates/benten-eval/tests/sandbox_capability_intersection_at_init.rs` — 2 tests (testing_revoke_cap_mid_call helper + TOML drift detector)
- `crates/benten-eval/tests/sandbox_host_fn_trampoline_count.rs` — 2 tests (D25 trampoline accounting + bypass field default)
- `crates/benten-eval/tests/sandbox_depth_inheritance_regression.rs` — 1 test (G7-B + G7-C coordination)
- `crates/benten-eval/tests/attribution_non_regression.rs` — 1 test (sec-pre-r1-13 carry)
- `crates/benten-eval/tests/proptest_sandbox_fuel.rs` — 1 proptest (fuel monotonicity, 10k cases)
- `crates/benten-eval/tests/proptest_sandbox_isolation.rs` — 1 proptest (no-state-persists, 10k cases)
- `crates/benten-eval/tests/proptest_sandbox_output.rs` — 1 proptest (output bounded, 10k cases)
- `crates/benten-eval/tests/integration/inv_7_streaming.rs` — 1 test (Inv-7 streaming end-to-end)
- `crates/benten-eval/tests/integration/inv_4_call_boundary.rs` — 2 tests (Inv-4 cross-CALL + D20 + Inv-14 carry)
- `crates/benten-eval/tests/integration/sandbox_wasm32_disabled.rs` — 1 test (eval-side wasm32 absence pin)
- `crates/benten-engine/tests/integration/engine_sandbox.rs` — 1 test (E2E engine SANDBOX dispatch)
- `crates/benten-engine/tests/integration/sandbox_in_crud.rs` — 2 tests (SANDBOX inside CRUD + host-boundary cap-check on WRITE)
- `crates/benten-engine/tests/integration/stream_into_sandbox.rs` — 1 test (STREAM-into-SANDBOX back-pressure)
- `crates/benten-engine/tests/integration/sandbox_compile_time_disabled_on_wasm32.rs` — 2 tests (wasm32 build target half + source-grep drift detector)

**What landing each requires:** runtime depth-threading wired into `ActiveCall` so `AttributionFrame.sandbox_depth` propagates through CALL boundaries (the SECURITY-POSTURE-disclosed gap); 10k-case proptest fuel/isolation/output property pins; build.rs codegen pipeline for the host-fn manifest TOML drift detector; integration drivers that wire host-fn callbacks through the engine dispatcher.

**Touch size:** ~600-900 LOC test source. Risk surface: medium (security-pin bodies; getting the Inv-4 transitive threading right is the load-bearing claim).

#### 7.3.A.2 — STREAM/SUBSCRIBE end-to-end integration test bodies (G6-A landed)

**Status:** CLOSED-IN-PHASE-3-G20-A2 (wave-8a). Test bodies un-ignored + driving real engine surfaces (call_stream + on_change + StreamHandle::next_chunk drain loops). The closure pin `tests/stream_subscribe_end_to_end_no_residual_ignore` (in `crates/benten-engine/tests/wait_ttl_gc_machinery.rs`) walks the in-scope test-file set + asserts no Phase-3-destination `#[ignore]` rationales remain.

**Phase 2b state:** G6-A (`e13e796`) + wave-8c production-runtime wire-through (`443590f`) both landed. The eval-side STREAM/SUBSCRIBE primitives execute; the bodies below are end-to-end engine integration tests that exercise the streaming back-pressure path through napi.

**Files (all un-ignored at G20-A2 wave-8a):**
- `crates/benten-engine/tests/integration/subscribe_emit.rs` — 1 test (SUBSCRIBE registration surface; full SUBSCRIBE → EMIT chain pins live in `engine_subscribe_*` integration suite)
- `crates/benten-engine/tests/integration/stream_composition.rs` — 2 tests (STREAM-into-STREAM + STREAM-into-ITERATE; CRUD handlers route through wave-8c-stream-infra typed up-front rejection)
- `crates/benten-engine/tests/integration/engine_stream.rs` — 2 tests (`stream_persist_true_materializes_aggregate_node` + `stream_backpressure_engages`; load-bearing observable: typed Ok / Err shape + drain loop runs to completion)
- `crates/benten-engine/tests/integration/stream_napi.rs` — 1 test (napi async-iterator surface drain + close idempotency contract; the JS-side back-pressure semantics test lives at `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`)

**What landing each required:** integration drivers that exercise call_stream → handle → next_chunk drain loops + on_change registration + the StreamHandle close idempotency contract.

**Touch size:** ~150-250 LOC test source. Risk surface: low.

#### 7.3.A.3 — User-view Strategy-A/C rejection + view-registry label-hint test bodies — CLOSED at Phase-3 G20-A3 wave-8a

**Phase 2b state:** G8-B (`71dff61`) + wave-8h IVM Algorithm B production registration both landed. The view registry routes Strategy::B to AlgorithmBView for the 5 canonical view IDs; user-defined Strategy::A/C are now rejected at registration (the documented behaviour) but the test bodies are `todo!()`.

**Files:**
- `crates/benten-engine/tests/user_view_strategy_a_rejected_for_user.rs` — 2 tests (Strategy::A rejection + Strategy::C reserved-for-Phase-3 path)
- `crates/benten-engine/tests/view_id_label_hint_refactor.rs` — 2 tests (view registry-driven label hint + canonical Phase-1 view registry coverage)

**What landing each requires:** test driver that constructs view specs with user-defined strategies and asserts the registration-time rejection error code; refactor of label-hint logic from string-prefix-strip to registry lookup.

**Touch size:** ~100-150 LOC test source. Risk surface: low.

**Closure shape (Phase-3 G20-A3 wave-8a):** Both test files now drive the production entry points end-to-end (per dispatch-conventions §3.6b) — `user_view_strategy_a_rejected_for_user.rs` registers Strategy::A + Strategy::C `UserViewSpec`s through `Engine::register_user_view` and asserts the typed `EngineError::ViewStrategyARefused` / `ViewStrategyCReserved` error bodies surface the offending view_id; `view_id_label_hint_refactor.rs` registers a non-prefixed user view with `Engine::register_user_view`, installs a deny-reads-on-`post` `CapabilityPolicy`, calls `Engine::read_view_with`, and asserts the registry-driven label hint causes the cap-recheck to fire (the legacy `content_listing_` string-prefix would have leaked the rows). Production fix: `engine_views.rs::read_view_with` now consults `hardcoded_label_for_id` + `user_view_input_labels` registry before falling back to the prefix-strip.

#### 7.3.A.4 — Module-install dual-CID + summary mismatch error body — CLOSED at Phase-3 G20-A3 wave-8a

**Phase 2b state:** G10-B (`dcfc108`) merged with `Engine::install_module` + `uninstall_module` APIs. The dual-CID error narrative for CID-mismatch is partially implemented but the test body that asserts the "both expected and actual CID + summary metadata in the error" is `todo!()`.

**Files:**
- `crates/benten-engine/tests/module_install_rejects_cid_mismatch_dual_cids_in_error.rs` — 1 test (D16 dual-CID + summary in mismatch error body)

**What landing each requires:** test driver that constructs a module manifest pointing at the wrong CID, calls `Engine::install_module`, and asserts the resulting `BentenError` carries both `expected_cid` and `actual_cid` plus a structured summary field.

**Touch size:** ~30-50 LOC test source. Risk surface: low.

**Closure shape (Phase-3 G20-A3 wave-8a):** The test now drives `Engine::install_module` with a deliberately-mismatched `expected_cid` and asserts the rendered `EngineError::ModuleManifestCidMismatch` Display contains BOTH the computed CID + the expected CID + the manifest summary anchor (`<name> v<version> modules=<n> caps=<n>`) so an operator can diagnose from logs alone (per dispatch-conventions §3.6b).

#### 7.3.A.5 — Doc-drift detector test bodies — CLOSED at Phase-3 G20-A3 wave-8a

**Phase 2b state:** G12-B (`edb1f93`) + G11-2b-A (`8169807`) both merged with the docs sweep + DSL-SPECIFICATION.md finalization + SECURITY-POSTURE.md Phase-2b compromises + ARCHITECTURE.md crate-count narrative. The doc-drift detectors that read the .md files and assert structural invariants have `todo!()` bodies.

**Phase-3 state update (2026-05-05):** R3-A + R3-C landed `benten-id` + `benten-sync` canary stubs taking the workspace from eight to 10 crates; the prior eight-crate detector file was renamed + retensed to 10-crate at the orchestrator-direct cite-drift detector source-of-truth bump. Test stays `#[ignore]`'d; body-lift to executable still lands at G20-B per `tests/phase_3_workspace/architecture_md_g20b_final.rs`.

**Files:**
- `crates/benten-engine/tests/architecture_md_10_crate_count_post_phase_3_canaries.rs` — 2 tests (10-crate assertion enumerating benten-id + benten-sync + dsl-compiler + native-only flag for benten-sync, + canary-crate-dir presence pin). Renamed from `architecture_md_8_crate_count_after_dsl_compiler.rs` at orchestrator-direct cleanup 2026-05-05.
- `crates/benten-engine/tests/dsl_specification_md_finalization.rs` — 1 test (DSL-SPECIFICATION.md finalization assertions)
- `crates/benten-engine/tests/security_posture_md_phase_2b_compromises_documented.rs` — 1 test (Phase-2b compromise additions assertion)
- `crates/benten-engine/tests/error_catalog_md_drift_phase_2b.rs` — 2 tests (Phase-2b code presence + fix-hint format enforcement)
- `crates/benten-engine/tests/quickstart_md_walkthroughs_compile.rs` — 1 test (QUICKSTART.md walkthroughs compile)

**What landing each requires:** test bodies that parse the markdown via a structured-section reader and assert the documented invariants. The QUICKSTART.md walkthroughs-compile test needs a build harness that extracts code blocks and runs them through `cargo build`.

**Touch size:** ~200-300 LOC test source. Risk surface: low.

**Closure shape (Phase-3 G20-A3 wave-8a):** All five doc-drift detector tests un-ignored. Tests now actively scan `docs/ARCHITECTURE.md` (10-crate enumeration + benten-id + benten-sync + native-only + benten-dsl-compiler), `docs/DSL-SPECIFICATION.md` (front-matter `Status: FINAL` marker added at G20-A3 to satisfy the finalization assertion), `docs/SECURITY-POSTURE.md` (Phase-2b compromise documentation including module manifest + ed25519 + manifest-not-yet-subgraph + browser persistent storage + cross-browser determinism + Compromise #4 + #9 CLOSED markers), `docs/ERROR-CATALOG.md` (Phase-2b code presence + fix-hint format), and `docs/QUICKSTART.md` (≤15 LOC walkthroughs for STREAM / SUBSCRIBE / SANDBOX). Workspace-shape sanity check also un-ignored (asserts `crates/benten-id/`, `crates/benten-sync/`, `crates/benten-dsl-compiler/` directories present).

#### 7.3.A.6 — WAIT TTL runtime expiry path test bodies (G12-E landed structurally; runtime path Phase-3)

**Status:** CLOSED-IN-PHASE-3-G20-A2 (wave-8a). GC machinery PRODUCTION code landed at `crates/benten-engine/src/wait_ttl_gc.rs` (~250 LOC: event-driven sweep on suspend / resume + 1h interval backstop entry point + Engine::drop final sweep + observable `WaitTtlGcStats`). `WaitMetadata` extended with `ttl_hours` + `suspend_wallclock_ms` (forward-/backward-compatible `#[serde(default)]`); `register_subgraph` validates `ttl_hours ∈ [1, 720]` with E_WAIT_TTL_INVALID; `resume_with_meta` consults the wall-clock deadline before the in-process timeout check + fires E_WAIT_TTL_EXPIRED on expiry + reaps the entry as part of the resume hot-path event-driven sweep. ErrorCode catalog gained `WaitTtlExpired` + `WaitTtlInvalid` + `WaitMetadataMissing` (all routed to ON_ERROR per §9.x). Test helpers: `testing_advance_wait_clock` + `testing_set_wall_clock_baseline` + `testing_run_gc_interval_tick` + `testing_make_wait_spec_with_ttl_hours{,_unchecked,_default}` + `testing_inspect_wait_metadata` + `testing_suspension_store_has_wait` + `testing_make_resume_payload` + `testing_assert_outcome_complete` + `testing_make_unregistered_envelope`. Engine fields: `wait_wall_clock_override_ms` + `wait_ttl_gc_stats` + `wait_ttl_gc_event_driven_disabled` + `wait_ttl_tracked_envelopes`. GC scheduling discipline: event-driven (suspend / resume hot path; suppressible) + interval backstop (1h cadence; production wires a tokio interval, tests drive synchronously via `testing_run_wait_ttl_gc_pass`) + drop-final (Engine::drop best-effort sweep). Closure pin `tests/wait_ttl_runtime_expiry_path_gc_machinery_correct` drives 8 parallel suspended waits + advances clock 2h past 1h TTL + asserts GC reaps all + stats record reaps + post-reap entries absent.

**Phase 2b state:** G12-E (`0ac7b0a`) + wave-8i WAIT production runtime (`55b084a`) both landed. `SuspensionStore`, `resume_with_meta`, the engine clock override are all on main. The remaining gap (NOW CLOSED) was the runtime TTL **expiry path** — the deadline check + GC sweep that converts metadata into typed errors. Wave-8i wired the deadline consultation; the GC + cross-process expiry semantics + the runtime TTL surface itself were deferred to G20-A2 wave-8a.

**Files:**
- `crates/benten-errors/tests/wait_ttl_codes_present.rs` — 3 tests (WaitTtlExpired variant + WaitTtlInvalid variant + anti-rename guard)
- `crates/benten-eval/tests/wait_ttl.rs` — 5 tests (default 24h, explicit override, 0h validation, 720h max validation, E_WAIT_TTL_EXPIRED resume)
- `crates/benten-eval/tests/wait_ttl_cross_process.rs` — 2 tests (TTL persistence across `Engine::open` boundary + wall-clock-relative semantics)
- `crates/benten-eval/tests/wait_ttl_gc.rs` — 4 tests (event-driven sweep, 1h interval backstop, event-driven-disabled config, `Engine::drop` final sweep)
- `crates/benten-eval/tests/proptest_wait_ttl.rs` — 1 proptest (TTL property; depends on `testing_advance_wait_clock`)
- `crates/benten-engine/tests/integration/cross_process_wait_resume.rs` — 2 tests (on-disk SuspensionStore + wait.rs rewire; cross-process resume)
- `crates/benten-engine/tests/integration/wait_ttl_expires_via_suspension_store.rs` — 1 test (WAIT ttl_hours + GC + E_WAIT_TTL_EXPIRED end-to-end)

**What landing each requires:** the GC machinery (event-driven sweep on `suspend()` + 1h interval backstop + final sweep on `Engine::drop`); the `testing_advance_wait_clock` test helper that the proptest depends on; the cross-process resume test driver that rebuilds `Engine` against the same on-disk store.

**Touch size:** ~400-600 LOC test source + the GC machinery itself (~200-400 LOC eval-side). Risk surface: medium (GC scheduling + cross-process correctness).

#### 7.3.A.7 — Wave-8b/8c "paired" testing helpers — security-critical SANDBOX-escape pins (ESC-9/-10/-15 etc.) — **CLOSED-IN-PHASE-3-G20-A1**

**Status (2026-05-07):** **CLOSED-IN-PHASE-3-G20-A1 wave-8a.** The §7.3.A.7 helper SURFACE was shipped at G17-A1 wave-5b (per seq-minor-2); G20-A1 wave-8a un-ignored every test body that depends on the SURFACE. Bodies un-ignored at `crates/benten-eval/tests/{sandbox_host_fn_caps,sandbox_host_fn_kv_read,sandbox_capability_check_per_call_after_revoke,sandbox_escape_attempts_denied,sandbox_output}.rs` (5 file × ~9 test bodies). LOAD-BEARING security pin `sandbox_escape_helpers_no_widening_of_production_attack_surface` un-ignored at `crates/benten-eval/tests/sandbox_helpers_no_widening.rs` — audits that the helper module's file-level `#![cfg(any(test, feature = "test-helpers", feature = "testing"))]` gate is present + no `pub` items leak into the production cdylib (Phase-2a sec-r6r2-02 precedent). ESC-7/9/13/16 runtime arms exercised end-to-end via `execute_with_live_cap_check` + `TestEscAttackInjection` test-only seam (sibling pins at `tests/sandbox_esc_runtime_arms_e2e.rs` already shipped at wave-5c §6.1-followup). ESC-10/14 covered structurally — testing_call_engine_dispatch helper remains a marker (no host-fn re-enters Engine::call); testing_inject_forged_cap_claim_section + ESC-14 verified at the structural level (engine consults manifest exclusively for cap derivation, source-grep confirms no custom-section parsing).

**Phase 2b state (historical):** Wave-8b (`1f11c61`) shipped the wasmtime trampoline + per-call Store discipline; wave-8c (subscribe-infra + stream-infra + 8c-cont) all merged. The eval-side ESC-pin tests referenced testing helpers (`testing_revoke_cap_mid_call`, `testing_call_engine_dispatch`, `testing_inject_forged_cap_claim_section`, `testing_register_uncounted_host_fn`) that the rationales claimed were "paired with 8c work" but never actually shipped. **These were SECURITY-CRITICAL ESC-pin tests** — the `SECURITY-POSTURE.md` ESC matrix at §"Compromise #4" honestly disclosed ESC-9 / ESC-10 / ESC-13 as "Partial / eval-side smoke" with the helper-fn smoke tests verifying trampoline accounting; the integration-shape pins were reserved (now closed at G20-A1 wave-8a).

**Files:**
- `crates/benten-eval/tests/sandbox_host_fn_caps.rs` — 4 tests (per-call cap-recheck + per-boundary trampoline + typed-error-not-trap routing × 2)
- `crates/benten-eval/tests/sandbox_host_fn_kv_read.rs` — 1 test (mid-call revoke during kv:read)
- `crates/benten-eval/tests/sandbox_capability_check_per_call_after_revoke.rs` — 1 test (D7 hybrid + D18 per_call cap_recheck integration)
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:226` — 1 test (ESC-7 fuel-refill via host-fn re-entry)
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:266` — 1 test (ESC-9 host-fn after cap revoke)
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:290` — 1 test (ESC-10 reentrancy via host-fn)
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:349` — 1 test (ESC-13 trap in fuel callback / Store-poison)
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:377` — 1 test (ESC-14/-15 forged cap-claim section ignored — needs `testing_inject_forged_cap_claim_section`)
- `crates/benten-eval/tests/sandbox_output.rs:194` — 1 test (D17 BACKSTOP path; needs `testing_register_uncounted_host_fn`)

**What landing each requires:** four engine-layer testing helpers that mutate live cap-set / inject forged cap-claim sections / register uncounted host-fns / re-enter engine.call() from inside host-fn callbacks. Each needs a feature-gated test surface + careful threading of the engine dispatcher reachability into a host-fn callback. Cross-reference SECURITY-POSTURE.md's "Honest disclosure — Inv-4 runtime threading" + "ESC defense matrix" sections — those record the disclosure that this cluster sits behind.

**Touch size:** ~200-300 LOC eval+engine helper sources + ~200-400 LOC integration test bodies. Risk surface: HIGH (security claims; helpers must NOT widen the production attack surface).

**Cross-ref:** §6 "SANDBOX runtime maturity" — the integration-pin landing pairs with §6.1 (ESC-16 fingerprint-collapse complete defense) + §6.2 (D26 .wasm-bytes-shipping per fixture) so a Phase-3 wave can land the helper surface + the integration tests + §6 in one cycle.

#### 7.3.A.8 — wasmtime Component-Model gated SANDBOX-escape tests (wsa-3 removed feature) — RATIONALES REWRITTEN at Phase-3 G20-A3 wave-8a (HELD CUT per D-PHASE-3-6 RESOLVED-at-R1)

**Phase 2b state:** wsa-3 explicitly removed Component-Model from Phase 2b scope. The two ESC-11/-12 tests (Component-Model type-mismatch + resource-handle-forgery) are `#[cfg(feature = "component-model")]`-gated AND `#[ignore]`'d. SECURITY-POSTURE.md ESC matrix records both as "Component-model gated (2): full coverage requires wasm-component-model surface; current defense rejects via `Module::new` structural validation."

**Files:**
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:311` — ESC-11 component-type-mismatch
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:328` — ESC-12 resource-handle-forgery

**What landing each requires:** Phase 3 must first re-evaluate the Component-Model adoption decision (whether wsa-3 holds or whether wasmtime's component-model story is mature enough to re-enable the cargo feature). If Component-Model is re-enabled, both test bodies fold into §7.3.A.1's broader runtime-SANDBOX cluster.

**Touch size:** ~30-50 LOC test source (after Component-Model re-enabled). Risk surface: low (gated, opt-in).

**Cross-ref:** §6 "SANDBOX runtime maturity" + Phase-3 plan-doc opening checklist item that explicitly asks "do we re-open Component-Model?"

**Closure shape (Phase-3 G20-A3 wave-8a — HELD CUT):** D-PHASE-3-6 RESOLVED-at-R1 (2026-05-05) held the cut for Phase 3 (per CLAUDE.md key reading list). Both ESC-11 + ESC-12 test rationales rewritten at G20-A3 to point at the named destination "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA" per D-PHASE-3-16 ratification 2026-05-05. The rationale text mirrors the IFF-clause-grounded form: "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA — Component-Model held cut at Phase-3 R1 per D-PHASE-3-6 RESOLVED-at-R1; rationale rewritten per D-PHASE-3-16 named destination ratified 2026-05-05 (docs/FULL-ROADMAP.md Phase 4 entry naming wasmtime Component-Model re-evaluation). Phase-4 pre-R1 inherits this deferral when Phase 4 opens." A new structural pin lands at `crates/benten-engine/tests/component_model_phase3_decision_lands_per_d_phase_3_6.rs` enforcing both (a) the wasmtime `component-model` feature stays absent from `crates/benten-eval/Cargo.toml`, and (b) `docs/FULL-ROADMAP.md` continues to name the Phase-4 wasmtime Component-Model re-evaluation entry — IFF-clause violation surfaces immediately in CI.

#### 7.3.A.9 — Workflow-baseline + browser-bundle artifact + subscribe persistent-cursor helpers — CLOSED at Phase-3 G20-A3 wave-8a

**Phase 2b state:** Discovered during the wave-8j-followup post-edit verification sweep — additional STALE-LANDED rationales beyond the original §A.1-A.8 inventory. Three sub-clusters:

**Files (sub-cluster 9a — CI/workflow baselines partially landed):**
- `crates/benten-engine/tests/cargo_vet_policy_self_test.rs` — 1 test. cargo-vet workflow exists but `supply-chain/` baseline directory is not committed. **What landing each requires:** run `cargo vet init` + commit the baseline + ensure the workflow constrains rather than no-ops.
- `crates/benten-engine/tests/cargo_public_api_drift.rs` — 2 tests. `.github/workflows/cargo-public-api.yml` exists; `docs/public-api/` baseline directory does not. **What landing each requires:** generate per-crate public-api baseline files + commit them + cargo-public-api workflow already wired to consume.

**Files (sub-cluster 9b — browser-bundle artifact pinning):**
- `crates/benten-engine/tests/integration/browser_target_bundle_size.rs` — 2 tests. `wasm-browser.yml` workflow exists; `bindings/napi/dist/browser/` artifact directory is not committed (workflow produces it ephemerally). **What landing each requires:** commit a stable artifact path under `bindings/napi/dist/browser/` (or rewire test to produce-and-check rather than read-from-disk) + assert wasm-r1-7 cap holds.

**Files (sub-cluster 9c — subscribe persistent-cursor testing helpers):**
- `crates/benten-engine/tests/integration/suspension_store_round_trip_subscription_cursor.rs` — 1 test. `testing_register_persistent_subscriber` + `testing_emit_n_synthetic_events` helpers were promised "paired with 8c-cont engine boundary wire-through" but never shipped on main. **What landing each requires:** the two engine-layer testing helpers + the SUBSCRIBE production runtime path that drives subscribe.rs through the engine boundary for cursor-write/cursor-read round-trip.

**Touch size:** ~50-100 LOC test source for sub-cluster 9b/9c bodies + ~200-400 LOC for the helper infrastructure (sub-cluster 9c) + a small but careful run of `cargo vet init` + cargo-public-api baseline generation. Risk surface: low (mostly tooling).

**Cross-ref:** §7.3.A.7 (similar shape — testing helpers paired with closed waves but never shipped); Phase-3 plan-doc opening checklist (CI baselines + browser-bundle artifact pinning are Phase-3 first-wave hygiene items).

**Closure shape (Phase-3 G20-A3 wave-8a):** Three sub-clusters all closed end-to-end.

- *Sub-cluster 9a — CI baselines.* `supply-chain/{config.toml,audits.toml,exemptions.toml,imports.lock}` baseline directory committed (cargo-vet onboarding policy per sec-r1-5: exemption-budget = 5 max, criteria-set = `safe-to-deploy` (default) + `crypto-reviewed` (manual), quarterly review cadence; upstream vouch sources = mozilla + bytecode-alliance + google). New `.github/workflows/cargo-vet.yml` workflow runs `cargo vet check --locked` on every PR + push, surfaces results to step summary, exits 0 (informational gate per sec-r1-5 onboarding cadence). `.github/workflows/cargo-public-api.yml` STUB replaced with a real per-crate diff loop against `docs/public-api/<crate>.txt` baselines (8 existing-crate `.txt` baselines committed at G20-A3 alongside the previously-landed `benten-id.json` + `benten-sync.json` per-NEW-crate baselines from G14-A1 + G16-A). Test pin `tests/cargo_vet_exemption_budget_at_or_below_5_at_phase_3_close` enforces sec-r1-5 budget at every CI run.
- *Sub-cluster 9b — browser-bundle artifact.* `bindings/napi/dist/browser/benten_engine.wasm.gz` placeholder seed committed at G20-A3 (47 bytes — well under the wasm-r1-7 500KB cap; CI rebuilds and overwrites the seed with the production bundle on every push). `bindings/napi/dist/browser/README.md` documents the artifact-path + the placeholder-vs-production semantics. Both `wasm32_unknown_unknown_bundle_size_under_threshold` + `browser_bundle_excludes_napi_node_binary` tests un-ignored.
- *Sub-cluster 9c — subscribe persistent-cursor helpers.* `testing_register_persistent_subscriber` + `testing_emit_n_synthetic_events` lifted from wave-8g shape-only stubs to working drivers. The helpers maintain a cfg-gated `EngineInner.testing_persistent_subscribers: Vec<SubscriberId>` registry; register seeds `put_cursor(&sub_id, 0)` against the engine's `SuspensionStore`, emit advances every registered subscriber's cursor to `n` via the same `put_cursor` write entry — the `subscribe_max_delivered_seq_round_trips_via_suspension_store` round-trip pin asserts `get_cursor(&sub_id) == n` post-emit (per D5 cross-process replay-on-restart). Production SUBSCRIBE delivery path remains exercised separately by `engine_subscribe.rs::on_change_*` tests; this helper-pair's job is the SuspensionStore-cursor round-trip under caller-controlled SubscriberIds.

---

### 7.3.C — STALE-RATIONALE-NO-DESTINATION fixes (HARD-RULE compliance) — CLOSED at Phase-3 G20-A3 wave-8a

**Phase 2b state:** Eight `#[ignore]` rationales failed the HARD-RULE "named destination" test — they pointed at closed/removed waves or used phrases like "future wave" / "when a public security posture doc lands" / "when the runtime path lands." Wave-8j-followup-stale-deferrals rewrote each rationale to point at this subsection (or a more specific §7.3.A.X subsection where applicable). Bodies remain deferred to Phase 3 with the named destinations below.

**Files:**
- `crates/benten-eval/tests/proptest_exec_state_round_trip.rs:49` — Phase 2a closed at `phase-2a-close` tag; G3-A surfaces (`ExecutionStateEnvelope`, `ExecutionStatePayload`, `AttributionFrame`) DID land. **What landing each requires:** the proptest body needs to construct random states, round-trip them through DAG-CBOR encode/decode, and assert byte-identity. Phase 3 (or fold into Phase-3-plan-doc CI-hygiene pass).
- `crates/benten-engine/tests/inv_8_isolated_call_budget_bypass.rs:57` — Phase 2a closed; G4-A wired the budget-isolation reset semantics. **What landing each requires:** test body that establishes a parent CALL with budget B, makes an isolated CALL inside it that consumes B, and asserts the parent retains its remaining budget after the inner call returns.
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:311` + `:328` — Component-Model gated; covered by §7.3.A.8 above.
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:377` — covered by §7.3.A.7 above (forged-cap-claim-section helper).
- `crates/benten-eval/tests/sandbox_output.rs:194` — covered by §7.3.A.7 above (testing_register_uncounted_host_fn helper).
- `crates/benten-eval/tests/read_denial.rs:225` — references SECURITY-POSTURE.md as "internal-only as of 2026-04-23 default-untrack pass; re-enable when a public security posture doc lands." Rewritten to point at this destination: Phase 3 may evaluate whether SECURITY-POSTURE.md re-tracks-public alongside other launch-readiness docs (FULL-ROADMAP §"Phase 9+ OSS launch era"); until then test stays `#[ignore]`'d here.
- `crates/benten-eval/tests/wait_signal_shape_optional_typing.rs:98` — runtime signal-shape check not implemented in 2b. **What landing each requires:** the runtime signal-shape mismatch detection in `wait::evaluate_op` that fires `WaitSignalShapeMismatch` when an injected `signal` payload's shape doesn't match the WAIT node's declared `signal_shape` property.

**Touch size:** ~150-250 LOC test source + supporting runtime hooks. Risk surface: low-medium (one runtime check addition for wait-signal-shape).

**Closure shape (Phase-3 G20-A3 wave-8a):** Each rationale rewritten or test un-ignored per below:

- `proptest_exec_state_round_trip.rs:49` — rationale rewritten to point at "Phase 3+ anytime backlog" with reassessment cadence = v1-assessment-window per CLAUDE.md baked-in #15. Body remains deferred (the proptest is a strengthening pass over the existing structural unit-tests at `crates/benten-eval/src/exec_state.rs` which already pin round-trip equality).
- `inv_8_isolated_call_budget_bypass.rs:57` — rationale rewritten to point at "Phase 3+ anytime backlog" with same reassessment cadence. Body remains deferred (the per-call budget reset on isolated-CALL is in `primitive_host.rs::dispatch_call_inner`; the integration body lands in the next round of budget-axis property-coverage hardening).
- `sandbox_escape_attempts_denied.rs:311 + :328` (Component-Model ESC-11/-12) — covered by §7.3.A.8 above (rewritten 2026-05-05 to D-PHASE-3-16 named destination; HELD CUT per D-PHASE-3-6).
- `sandbox_escape_attempts_denied.rs:377` (forged-cap-claim-section helper) — owned by §7.3.A.7 cluster (G20-A1 wave-8a).
- `sandbox_output.rs:194` (testing_register_uncounted_host_fn helper) — owned by §7.3.A.7 cluster (G20-A1 wave-8a).
- `read_denial.rs:225` — UN-IGNORED at G20-A3. The deferral condition was "when SECURITY-POSTURE.md re-tracks-public" — that condition is satisfied (the doc is in `docs/SECURITY-POSTURE.md` committed and contains both "Option C" + `diagnose_read`/`diagnoseRead` references).
- `wait_signal_shape_optional_typing.rs:98` — RE-IGNORED at G20-A3 with re-targeted rationale (the deferral condition "runtime check addition in `wait::evaluate_op` lands Phase 3" IS satisfied — runtime shape-match wired end-to-end at `crates/benten-eval/src/primitives/wait.rs::evaluate_op_with_handler_id` SUSPEND-time + `crates/benten-eval/src/primitives/wait.rs::shapes_match` consumed by `resume_with_meta` resume-time + `SuspensionStore` round-trip). The re-ignore reason is a routed_edge_label classification mismatch: `ErrorCode::InvRegistration` is routed to `None` in `crates/benten-errors/src/lib.rs::routed_edge_label` (registration-time None group), but the test asserts `Some("ON_ERROR")`. Either the test's expected edge label needs adjustment OR the variant needs reclassification under the runtime-error group. Destination: §7.17 — bundle with the next round of routed_edge_label classification hardening.

---

## 7.5 cargo-llvm-cov coverage workflow investigation — DIAGNOSED 2026-05-07 (Phase-3 R5 wave-9 W9-T4); root cause is downstream test flake, not coverage tooling

**Phase-3 R5 wave-9 W9-T4 diagnosis (2026-05-07, post `28031fa`):** the coverage workflow ITSELF is functional — recent runs against `main` show a green/red mix tracking the underlying test suite's stability (e.g. run `25529408373` red on `0913a0d`, run `25529646063` green on `3c97f3b`, ~50% green over the last 20 runs). The historical "failing since wave-8c-stream-infra" framing was correct at the time it was written but the surface has shifted: coverage's recent reds are caused by **downstream test flakes**, not coverage-tool config drift.

**Concrete root cause for the current flake-class:** the `wait_signal_shape_defaults_untyped_accepts_any_value` test in `crates/benten-eval/tests/wait_signal_shape_optional_typing.rs` panics intermittently with `untyped shape must accept any Value, got Err(InvRegistration)`. This is a **wait-signal-shape registration validation** issue independent of coverage instrumentation — when the test passes (most of the time), coverage runs to completion and produces a green report; when the test fails, the entire `cargo llvm-cov --workspace` invocation aborts with exit 101 because llvm-cov shells out to `cargo test --tests` and propagates the test-suite exit code.

**Disposition:** the coverage workflow itself does NOT need restoration work — it is a faithful reflector of the underlying test suite. The downstream flake belongs in the test-stability bucket, not the coverage-tooling bucket. Cause-(a) from the original §7.5 framing ("the same test surface drift that affects the vitest informational workflow") is confirmed; causes (b) coverage-tool config and (c) Intel-Mac nextest timeouts are ruled out for the current flake-class.

**Forward-looking cleanup:**
- The `wait_signal_shape_defaults_untyped_accepts_any_value` flake is BELONGS-ELSEWHERE-SPECIFICALLY → file as a fresh §7.16 entry below (the wait-signal-shape registration-validation row); fixing that flake will green coverage as a side-effect.
- If a future flake-class shows up on coverage that is NOT a downstream test issue (e.g. cargo-llvm-cov action update breaks the workflow), reopen this entry with the new diagnosis.

**Touch size at the diagnosis pass (this entry):** 0 LOC tooling change. The retense itself is the deliverable.

**Cross-references:**
- `.github/workflows/coverage.yml` (workflow definition; no change needed)
- `crates/benten-eval/tests/wait_signal_shape_optional_typing.rs::wait_signal_shape_defaults_untyped_accepts_any_value` (downstream flake; queue as §7.16 below)
- Run history sample: `https://github.com/BentenAI/benten-engine/actions/workflows/coverage.yml` (~50% green over the most-recent 20 runs at diagnosis time)

---

### 7.16 wait_signal_shape_defaults_untyped_accepts_any_value flake — CLOSED 2026-05-08 (root cause: shared default_process_store across in-binary tests with colliding signal-derived envelope CIDs)

**Origin (2026-05-07):** §7.5 diagnosis traced the cargo-llvm-cov coverage workflow's intermittent reds to this single test panicking with `untyped shape must accept any Value, got Err(InvRegistration)`. The test asserts that an "untyped shape" (no `signal_shape` property set) accepts any Value at registration time, but the runtime is sometimes returning `Err(InvRegistration)` instead of `Ok(_)` for that case.

**Closure shape (2026-05-08):** root cause is **test isolation against the process-default suspension store**. The three `wait_signal_shape_*` tests all suspend on signal name `"go"`, which derives an identical envelope CID via `crates/benten-eval/src/primitives/wait.rs::placeholder_payload_for_signal` (BLAKE3-hash of the signal name). Without an injected store, `EvalContext::with_clock(...)` falls back to `crates/benten-eval/src/suspension_store.rs::default_process_store` — a process-wide `OnceLock<Arc<InMemorySuspensionStore>>` shared across the whole test binary. When `cargo test` runs the tests in parallel (the default outside the `serial-globals` nextest test-group, which `cargo-llvm-cov` does NOT honor since it shells out to plain `cargo test --tests`), one test's `WaitMetadata{signal_shape: Some(_)}` wins the last-write race against another test's `WaitMetadata{signal_shape: None}` under the SAME envelope CID, flipping the resume-time shape check intermittently.

**Fix shape:** added a `ctx_with_isolated_store()` helper in the test file that builds an `EvalContext` with a fresh per-test `InMemorySuspensionStore` injected via `with_suspension_store(...)`. The fresh store eliminates the shared key space — each test's WAIT metadata lives in its own isolated `Arc<dyn SuspensionStore>` regardless of the harness scheduling. The previously-`#[ignore]`'d `wait_signal_shape_defaults_untyped_accepts_any_value` is now active and passes 50/50 consecutive runs locally.

**Bundled in same PR — same root cause class:** `crates/benten-eval/tests/wait_timeout.rs` exhibited an identical pre-existing flake under shared-store collisions. `wait_signal_arrives_after_timeout_fires_e_wait_timeout` (timeout=100ms) and `wait_signal_arrives_before_timeout_resumes_normally` (timeout=1000ms) both suspend on signal name `"user_resumes"` → identical envelope CID → last-write race on `WaitMetadata.timeout_ms`. Reproduced ~30% red on origin/main (6/20 fail rate measured 2026-05-08). Same `ctx_with_isolated_store(clock)` helper applied; 30/30 consecutive runs pass post-fix. The brief's "bundle with any other wait-signal-shape stabilization work" guidance applies — both share the `default_process_store` + signal-name-keyed envelope-CID-collision pattern.

**Verification:** `cargo test -p benten-eval --test wait_signal_shape_optional_typing` 60/60 consecutive runs PASS (3 active + 1 still-ignored §7.17 routing-classification). `cargo test -p benten-eval --test wait_timeout` 30/30 consecutive runs PASS. Full benten-eval test surface (`cargo test -p benten-eval --tests --features benten-eval/testing`, 129 binaries) passes with no regressions.

**Cross-references:**
- `crates/benten-eval/tests/wait_signal_shape_optional_typing.rs::ctx_with_isolated_store` (per-test isolated-store helper for `signal_shape` collisions)
- `crates/benten-eval/tests/wait_timeout.rs::ctx_with_isolated_store` (per-test isolated-store helper for `timeout_ms` collisions; same closure pattern)
- `crates/benten-eval/src/primitives/wait.rs::placeholder_payload_for_signal` (the signal-name → envelope-CID derivation)
- `crates/benten-eval/src/suspension_store.rs::default_process_store` (the process-wide singleton that was the shared-state source)
- `.config/nextest.toml::serial-globals` (test-group override; still honored under nextest, but the in-test isolated store now removes the dependency on serialization scheduling for correctness)

---

## 7.4 CI lint: file:line cite drift detector — CLOSED at Phase-3 G13-pre-A (cite-drift-detector landed)

**Closure shape (Phase-3 G13-pre-A, 2026-05; envelope hardened at Phase-3 R5 wave-9 W9-T4):** the `tools/cite-drift-detector` Rust tool + its `.github/workflows/cite-drift.yml` non-blocking CI workflow ship every dimension of the original §7.4 ask:
- **`file:line` cite extraction** — `tools/cite-drift-detector/src/lib.rs::extract_line_cites` recognises `.rs`, `.ts`, `.tsx`, `.toml`, `.wat`, `.wasm`, `.md`, `.json`, `.yml`, `.yaml` extensions across `docs/**/*.md`, `crates/`, `tools/`, `packages/engine/src/`, `.cargo/config.toml`, and `.addl/` (when present locally).
- **Line-range existence check** — `tools/cite-drift-detector/src/lib.rs::check_line_cite` validates the file exists at HEAD and contains the cited line; emits `LineCiteFileMissing` / `LineCiteLineOutOfRange` findings.
- **Sentinel-anchor protection** — implemented in stronger form via §3.5b HARDENED point 3: any line cite to a high-churn surface (the `tools/cite-drift-detector/src/lib.rs::HIGH_CHURN_SURFACES` list — currently `primitive_host.rs`, `engine_views.rs`, `evaluator.rs`, `lib.rs`, `builder.rs`, `wait.rs`, `subscribe.rs`, `mermaid.ts`, `dsl.ts`) emits `LineCiteOnHighChurnSurface` with the message "use `path::symbol` form per §3.5b HARDENED point 3". This is **stronger than the original sentinel-anchor proposal** because it catches the antipattern at the cite-shape level rather than after a stale-anchor confirms drift.
- **Non-blocking PR comment** — `.github/workflows/cite-drift.yml` runs on every PR with `continue-on-error: true` + the `actions/github-script` step that posts a marker-keyed comment; promotion-to-required is tracked as `D-PHASE-3-10` in the Phase-3 implementation plan.

**Coverage envelope hardening at W9-T4:** the walker's coverage envelope was widened at Phase-3 R5 wave-9 W9-T4 (this fix-pass) to additionally cover `tools/*/src/**/*.rs` + `.cargo/*.toml` (closing §7.12b file-tree gaps). Re-export recognition was added to `target_text_defines_symbol` so cites to a crate-root `lib.rs` symbol resolve cleanly when the lib only `pub use`s that symbol from a sibling module.

**Cross-references:**
- `tools/cite-drift-detector/src/lib.rs` (the detector implementation)
- `.github/workflows/cite-drift.yml` (the CI workflow)
- `.addl/phase-2b/dispatch-conventions.md::§3.5b HARDENED point 3` (the high-churn-surface symbol-cite enforcement rule)
- §7.12 + §7.12b (this doc) — workspace-aware numeric-claim source-of-truth + file-tree-gap closure (both addressed at W9-T4)

---

### 7.19 devserver_in_flight_evaluations_complete_before_reload timing race on linux-arm64 1.95.0

**Origin (2026-05-09 PR #156 CI):** the `tools/benten-dev/tests/devserver_in_flight_completes.rs::devserver_in_flight_evaluations_complete_before_reload` test failed on the `build+test linux-arm64 @ 1.95.0` CI cell with `assertion left == right failed: in-flight eval must complete on pre-reload version. left: "v2", right: "v1"` at line 76. Sister cell `build+test linux-arm64 @ stable` was GREEN on the same commit; rerun of the failed cell on the same commit was GREEN — confirms timing flake, not regression.

**Test shape:** thread A is parked in a `slow_transform` gate; main thread reloads handler from v1 → v2; thread A is released and should resolve against its CAPTURED v1 snapshot. The race window: between release-of-thread-A and snapshot-readback, the test occasionally observes v2 instead of v1 — meaning the version-snapshot capture-vs-readback ordering is not strictly serialized on linux-arm64 1.95.0 (or the slow-transform gate doesn't fully guarantee the v1 snapshot is captured BEFORE the release fires).

**Likely root cause (not yet root-caused):** the `slow_transform` gate may release before the v1 snapshot is fully committed to the per-thread eval-state; on linux-arm64 1.95.0 with slightly different scheduler / atomic-ordering behavior than `@ stable`, the race surfaces. Affects `tools/benten-dev/src/` devserver hot-reload version-capture logic.

**Disposition:** BELONGS-NAMED-NOW per HARD RULE rule-12 clause-(b). Pre-existing timing race (not introduced by PR #155 / PR #156); confirmed flake-shape via successful rerun. Lands here as the named v1-window destination so the issue isn't ambiently dropped.

**Fix shape (when this lands):** investigate the `slow_transform` gate semantics in `tools/benten-dev/tests/devserver_in_flight_completes.rs` + the version-snapshot-capture site in `tools/benten-dev/src/`. Likely a sync-fence is missing OR the test should use a stronger barrier (e.g., explicit `Arc<Barrier>` instead of polling-based wait) to guarantee snapshot capture happens BEFORE the gate release.

**Phase target:** orchestrator-direct fix-pass batch alongside other Phase-3-close residuals OR pre-tag sweep. Low priority — flake, not blocker; rerun-on-failure cycles around it cheaply.

**Cross-references:**
- `tools/benten-dev/tests/devserver_in_flight_completes.rs` (test file; lines 60-90 are the gate-release + assert window)
- `tools/benten-dev/src/` devserver hot-reload version-capture site (specific file path TBD at fix time)
- PR #156 build+test linux-arm64 @ 1.95.0 run `25592063410` job `75131324528` (failure that surfaced this entry; rerun GREEN on same commit confirms flake)

---

### 7.18 hlc_clock_skew_within_tolerance parallel-test race (shared `static MOCK_MS`) — CLOSED 2026-05-09 (orchestrator-direct fix-pass batch)

**Origin (2026-05-09 PR #155 G16-B-prime CI):** the `cargo-llvm-cov` workflow on PR #155's fix-pass commit `ca16b37` failed with `update_within_custom_tolerance_accepts` panicking `assertion left == right failed: left: 1000000, right: 50500` at line 50. The expected value (`50_500`) matched the test's local `MOCK_MS.store(50_000, ...)` write; the actual value (`1_000_000`) matched a sibling test's write. The same fix-pass commit was previously GREEN on coverage at canary commit `76b8eba`, confirming flake — not regression.

**Root cause:** the previous test file declared a single `static MOCK_MS: AtomicU64` shared across the four `#[test]` functions in the same test binary. `cargo test` runs the four tests in parallel by default (no `serial-globals` test-group override since this binary isn't in `.config/nextest.toml`'s serial group, and `cargo-llvm-cov` shells out to plain `cargo test --tests` which doesn't honor nextest groups even when configured). One test's `MOCK_MS.store(1_000_000)` won the last-write race against another test's `MOCK_MS.store(50_000)` between the second test's store and its `hlc.update(&remote)` call. Coverage instrumentation slowed tests enough to widen the race window.

**Same root-cause class as §7.16 (CLOSED 2026-05-08 with `ctx_with_isolated_store` per-test isolation pattern).** That closure's signal-name → envelope-CID-collision shape didn't apply here, but the meta-pattern (process-shared mutable state colliding across parallel-scheduled tests in the same binary) is identical.

**Closure shape (2026-05-09):** replaced the single shared `static MOCK_MS: AtomicU64` with four per-test `static MOCK_DEFAULT/MOCK_CUSTOM/MOCK_BOUNDARY/MOCK_PAST: AtomicU64` + four bare `fn` wrappers. The `Hlc::new` API takes a `fn() -> u64` bare pointer (not `impl Fn`), so each test had to own its own `static` + free `fn` (an `Arc<AtomicU64>` + closure shape doesn't coerce to `fn`). Each test's mock-clock state now lives in its own static regardless of harness scheduling.

**Verification:** `cargo test -p benten-core --test hlc_clock_skew_within_tolerance` 60/60 consecutive runs PASS post-fix; subsequent `cargo-llvm-cov` runs on `main` should show no further `update_within_custom_tolerance_accepts` reds.

**Cross-references:**
- `crates/benten-core/tests/hlc_clock_skew_within_tolerance.rs` (per-test static decomposition; replaces the prior shared `MOCK_MS`)
- §7.16 closure pattern (per-test isolated state injection) — reference precedent
- PR #155 cargo-llvm-cov run `25591002108` job `75128549720` (failure that surfaced this entry)

---

### 7.17 routed_edge_label classification hardening (carry list)

**Origin (Pre-R4b 2026-05-08):** the `wait_signal_shape_optional_typing.rs::wait_signal_shape_mismatch_fires_typed_error_routed_on_error` test currently `#[ignore]`'s with a destination naming this row. The test asserts that a typed-shape mismatch routes to `WaitSignalShapeMismatch` rather than registration-time `None` routing when the wait signal carries a routed edge label.

**Phase 3 target:** Bundle with the next round of routed-edge-label classification hardening so the `wait_signal_shape_mismatch_fires_typed_error_routed_on_error` ignore can lift. Specifically:
- Audit the `crates/benten-errors/src/lib.rs::routed_edge_label` classification table to either reclassify `ErrorCode::InvRegistration` under the runtime-error group when fired from a WAIT-resume shape-mismatch path (so the test's `Some("ON_ERROR")` assertion holds), OR adjust the test's expected edge label to `None` to match the registration-time classification.
- Cross-check against the WAIT shape-match runtime path (`crates/benten-eval/src/primitives/wait.rs::shapes_match` consumed by `evaluate_op_with_handler_id`) to confirm the routing classification matches the operator-observable contract.
- Pin the `routed_edge_label` cases in the same test file alongside the existing `defaults_untyped_accepts_any_value` pin.

**Touch size:** ~20-40 LOC investigation + ~20-30 LOC pin (+ removal of the `#[ignore]` once the routing is confirmed).

**Phase target:** R6 phase-close convergence-round residuals (bundle with §7.16 if both surface in same round).

---

## 8. Phase 3 plan-doc opening checklist

When Phase 3 pre-R1 opens, the planning agent should:

1. Read this file end-to-end + the cross-referenced bisect/scoping plans.
2. Sequence PHASE-3-BUNDLE-1 (1.1 + 1.2 + 1.3 + 1.4 + 1.5) as one of the early waves so subsequent waves can consume the umbrella trait.
3. Sequence the durable UCAN backend (2.1) before SUBSCRIBE cap-recheck threading (2.2).
4. Bundle the IVM Algorithm B drift-detector + generalization (5.1) into a single wave; both halves want the same surface.
5. Bundle SANDBOX runtime maturity (§6) with §4.2 cross-browser determinism CI cadence promotion — shared tooling. Pair §6 with §7.3.A.7 (the security-critical SANDBOX-escape testing-helper cluster) since both want the engine-layer testing-helper surface.
6. Bundle the §7.3.A test-body residuals (~85 `#[ignore]`'d cases) into a single first-wave CI-hygiene sub-track. §7.3.A.1-A.8 are independently scoped; A.7 is highest-priority (security claims).
7. Re-evaluate the wsa-3 Component-Model removal decision (§7.3.A.8) — decide whether wasmtime's component-model story is mature enough to re-enable the cargo feature.
8. Note: per CLAUDE.md §15, Phase 3 close is the natural PAUSE-AND-ASSESS point for v1 milestone evaluation. The Phase 3 plan should list out what would and wouldn't be in scope for "v1 shippable" at the assess point — not as a binding decision but as a starting frame Ben can confirm/redirect.

---

## 9. Security-advisory carries (transitive deps; closure-tracked here)

Phase-3 sync introduces iroh + Loro CRDT, which transitively pull in dep tails carrying RUSTSEC advisories the project does not directly create. These are documented + ignored in `deny.toml::[advisories].ignore` so CI's cargo-deny job stays GREEN; this row is the named destination for closure tracking per HARD RULE rule-12 disposition (b).

**RUSTSEC-2026-0119 — hickory-dns CPU exhaustion via O(n²) name compression.**
- Transitive root: `iroh 0.98.2 -> hickory-resolver 0.26.0-beta.4 -> hickory-proto`.
- Closure plan: (a) Phase-3 R5 close iroh-version-bump cycle picks up an upstream hickory-dns fix (Dependabot fast-track per `scope-real-10`); (b) Phase-7 Garden-relay work may alternatively replace hickory-dns with a minimal in-process resolver, dropping the dep edge entirely.
- Remove the `deny.toml` ignore when iroh upstream picks up a hickory-dns fix or when we drop the dep edge.

**RUSTSEC-2026-0120 — hickory-dns NSEC3 unbounded loop on cross-zone responses.**
- Same transitive root as RUSTSEC-2026-0119 (hickory-resolver -> hickory-proto).
- Same closure path.

**RUSTSEC-2024-0436 — paste unmaintained advisory.**
- Transitive root: `iroh 0.98.2 -> netwatch 0.16.0 -> netlink-packet-core 0.8.1 -> paste 1.0.15`.
- Informational unmaintained advisory (no exploit). Possible alternatives: `pastey` (drop-in fork of paste, actively maintained) or `with_builtin_macros`.
- Closure plan: same iroh bump cycle. Phase-3 R5 close eligible to land an iroh patch that swaps `paste` for `pastey`.

**Cross-references:**
- `deny.toml::[advisories].ignore` — the active ignore entries with reason text.
- `.github/workflows/supply-chain.yml` — the CI invocation point.
- `CONTRIBUTING.md::Supply chain` — the yank-response protocol.

---

## 10. Compromise registry forward-revisit (v1-assessment-window destinations)

Per CLAUDE.md item #15 (v1-milestone-gate framing): Phases 1+2a+2b+3 minimum + post-Phase-3 PAUSE-AND-ASSESS step. The Compromises listed below are currently `Open` or `Open (architectural bound)` in `docs/SECURITY-POSTURE.md`; each has a documented architectural rationale for its current state but warrants explicit re-evaluation during the v1-assessment-window. This section is the named destination per HARD RULE rule-12 clause-(b) so each Compromise has a real home rather than drifting as "we'll think about it later."

**v1-assessment-window scope reminder:** the post-Phase-3 PAUSE-AND-ASSESS is for surfacing genuine v1-shippable unknowns + reframing Phases 4-8 into pre-v1 / post-v1 buckets if needed. The Compromises here are NOT pre-decided as "must close before v1" — they're the **input set for the assessment**, not the verdict.

### 10.1 Compromise #1 — TOCTOU window bound at CALL entry + ITERATE batch boundary

- **Phase introduced:** 1
- **Current state:** Open (bounded; documented threat model). Capability snapshot refreshes at commit + CALL entry + every `iterate_batch_boundary` (default 100 iters). Revocations between refresh points not visible to in-flight evaluations.
- **Phase-3 Compromise #11 closure** (per-row read-gate at delivery time, IVM views) reduces the TOCTOU surface significantly for read paths but does not close the original write-path TOCTOU window.
- **v1-assessment question:** v1 likely runs agentic / human-collaboration loops where "agent revoked mid-evaluation" matters. Is the 100-iter batch bound still right, or does v1 multi-tenant context want a tighter bound? Consider Phase-3 G14-D F6 SUBSCRIBE delivery-time cap-recheck pattern as the seam to extend into write paths.
- **Touch size if revisit lands:** ~50-150 LOC depending on bound shape (per-iter check vs adaptive vs unchanged-with-doc-rationale).

### 10.2 Compromise #5 — No write rate-limits; metric recorded only

- **Phase introduced:** 1
- **Current state:** Open (Phase 3+ closure target — capability-gated rate-limit-policy plug seam landed at Phase-3 G14-B but rate-limits themselves not Phase-3-scope).
- **v1-assessment question:** v1 multi-tenant or public-facing deployments need rate-limits. Is per-capability rate-limiting correct shape, or should it be per-actor / per-tenant / per-grant-CID? The G14-B plug seam is generic-enough to support any of these; the v1-assessment needs to pick.
- **Touch size if revisit lands:** ~100-300 LOC depending on rate-limit policy shape + storage backing (in-memory token-bucket vs durable counter).

### 10.3 Compromise #6 — BLAKE3 128-bit effective collision resistance

- **Phase introduced:** 1
- **Current state:** Open (architectural bound). Documentation-only stance.
- **Phase-1 commitment:** "Phase-3 UCAN-by-CID paths revisit." Phase-3 G21 typed-CALL shipped UCAN-by-CID via `ucan_validate_chain` + UCANBackend chain-walker. **Revisit narrative not landed.** Orphan-rescue surfaced 2026-05-08.
- **v1-assessment question:** Either close the commitment in `SECURITY-POSTURE.md` Compromise #6 (BLAKE3 collision resistance is genuinely sufficient for UCAN-by-CID + brief rationale) OR open a real follow-up (e.g., move to BLAKE3-256 truncated to 256-bit-effective at UCAN-by-CID call sites if collision-resistance is genuinely v1-blocking).
- **Touch size if revisit lands:** ~5-10 lines SECURITY-POSTURE.md update OR ~50-100 LOC if collision-bound widening adopted.

### 10.4 Compromise #13 — System-zone reserved-prefix rejection surface

- **Phase introduced:** 2a
- **Current state:** Open (documented; minor-3). System-zone reserved-prefix rejection lives at write-path; Inv-11 stop-gap is `WriteContext::is_privileged` flag.
- **v1-assessment question:** Phase-3 sync surfaces (Atrium replicas) introduce system-zone replication paths that need careful interaction with the reserved-prefix rejection. R4b dist-systems lens flagged Inv-13 row-4a/4b SPLIT classifier as the sync-time interaction point. Is the Phase-2a rejection surface still the right shape under multi-peer system-zone replication?
- **Touch size if revisit lands:** ~50-100 LOC if shape changes; documentation-only update if shape holds.

### 10.5 Compromise #14 — SANDBOX cold-start cost (no opt-in pool)

- **Phase introduced:** 2b
- **Current state:** Open (D3 RESOLVED — additive Phase-3 change if real-workload bottleneck).
- **v1-assessment question:** v1 paper-prototype revalidation may surface a workload that triggers cold-start as a real bottleneck. If so, the additive change is an opt-in instance pool (`engine.sandbox_pool({ size: N, idle_timeout_ms: T })`) that pre-warms wasmtime instances. Decision is whether to land it pre-v1 or accept cold-start as v1-shippable.
- **Touch size if revisit lands:** ~150-300 LOC for opt-in pool + tests.

### 10.6 napi cdylib Cargo feature-graph closure assertion — CLOSED at Phase-3 R6 fix-pass Wave B (r4-r1-wsa-3 LOAD-BEARING half of pim-2 §3.6b)

- **Phase introduced:** 3 (R4b-r2 pin authored at G17-A1 wave-5b; never un-ignored)
- **Origin:** r4b-wsa-5 (Phase-3 R4b wasmtime-sandbox lens, 2026-05-07): the prior `napi_cdylib_production_build_does_not_export_testing_helper_symbols` pin (formerly in `crates/benten-eval/tests/cfg_gating_audit.rs`, deleted 2026-05-09 in this batch) was the LOAD-BEARING half of the pim-2 §3.6b end-to-end shape per r4-r1-wsa-3. The 3 sibling source-cite pins in the same file were superseded by `crates/benten-eval/tests/sandbox_helpers_no_widening.rs` at G20-A1 wave-8a (file-level cfg gate + Cargo.toml default-off + pub-item-gating); the symbol-table-scan + Cargo feature-graph closure pin was **NOT covered** by the sibling — `sandbox_helpers_no_widening.rs` only audits source-side cfg-gating, NOT Cargo feature-graph composition that could transitively activate `benten-eval/test-helpers` from `bindings/napi`'s production feature set.
- **Status:** **CLOSED** at Phase-3 R6 fix-pass Wave B per R6 R1 br-r6-r1-3 MAJOR convergence-council ratification. The Cargo feature-graph closure pin landed at `bindings/napi/tests/feature_graph_closure_no_test_helpers_in_production.rs` (4 tests):
  - `napi_default_feature_does_not_include_test_helpers` — declarative pin: `default = [...]` does NOT directly include `test-helpers`.
  - `napi_default_feature_closure_does_not_activate_test_helpers_transitively` — LOAD-BEARING transitive-closure walk: features reachable from `bindings/napi.default` MUST NOT include `test-helpers`, `benten-eval/test-helpers`, or `benten-engine/test-helpers`. Walks the in-crate features table via the workspace `toml` dep (parse-only) + tracks both in-crate references (recurse) and cross-crate references (`<crate>/<feat>` form, recorded for assertion).
  - `napi_test_helpers_feature_only_reachable_when_explicitly_opted_in` — symmetric LIVE-PATH pin: `test-helpers` IS reachable from itself + DOES transitively activate `benten-engine/test-helpers` (so the feature is not vestigial; ensures we're testing the right thing).
  - `napi_napi_export_default_feature_closure_uses_only_production_features` — closure-purity pin: no test-only features (`in-process-test`, `test-helpers`) appear in the default closure.
- **Symbol-table-scan rung deferred (not in this closure).** The release-cdylib build + platform-conditional `nm`/`dumpbin` symbol-table-dump rung was the *harder* half of the original §10.6 framing. The Cargo feature-graph closure pin (defense-in-depth rung 2) catches every Cargo-feature-graph regression vector that the symbol-table-scan would catch; the symbol-table-scan adds value only if a release-cdylib build accidentally exports testing-helper symbols WITHOUT a feature-graph activation (i.e., a hand-written `pub` that bypasses feature gating entirely). The `crates/benten-eval/tests/sandbox_helpers_no_widening.rs` file-level cfg gate + pub-item-gating audit closes that regression vector at the source layer (defense-in-depth rung 1). The CI symbol-table-scan rung remains a v1-window-candidate hardening if release-cdylib defense-in-depth audit ratifies it as net-new value over the source + feature-graph rungs.

**Cross-references:**
- `crates/benten-eval/tests/sandbox_helpers_no_widening.rs::sandbox_escape_helpers_no_widening_of_production_attack_surface` (defense-in-depth rung 1 — file-level cfg gate + Cargo.toml default-off audit)
- `bindings/napi/tests/feature_graph_closure_no_test_helpers_in_production.rs` (defense-in-depth rung 2 — Cargo feature-graph closure walk; CLOSED at this entry)
- `bindings/napi/Cargo.toml` (the production cdylib whose feature-graph closure does NOT transitively activate `benten-eval/test-helpers`)
- Phase-2a `sec-r6r2-02` precedent (the named regression vector this pin defends)

---

## 11. Phase-1 deferred-orphan-rescue (2026-05-08 cross-phase audit)

Items surfaced during 2026-05-08 cross-phase retrospective audit as orphans (Phase-1 deferrals never landed in Phases 2a/2b/3 R5 + not previously named in any backlog). Each gets a real home here per HARD RULE rule-12.

### 11.1 `get_node_verified` read-path hash check — CLOSED at W9-T6 PR #142 (orphan-rescue audit was based on stale Phase-1 retrospective claim)

(Cross-reference; full closure narrative at §1.6 above. Retensed 2026-05-08 after triage agent verified W9-T6 had already promoted base `get_node` to verify-on-read. The 2026-05-08 orphan-rescue audit was based on a stale Phase-1 retrospective that incorrectly claimed the work was never landed.)

### 11.2 SHA-pin coverage gap audit — RESOLVED at PR #150 (2026-05-08; coverage was already 100% — orphan-rescue framing was incorrect)

- **Origin:** Phase-2a §3.1 CI hardening pass committed to SHA-pin third-party GitHub Actions for supply-chain defense. Spot-check 2026-05-08 surfaced apparent `@master` / `@stable` / `@nightly` references in `.github/workflows/*.yml`.
- **Audit outcome (PR #150 by sha-pin-sweep agent):** **SHA-pin coverage is and remains 100%.** The 2026-05-08 spot-check claim was a misread of inline version-tracking comments — every `# stable` / `# master` / `# nightly` in workflow files is an annotation describing where a SHA was resolved from, NOT an unsafe ref. Verified via `awk` extract of `uses:` directives stripping comments — zero non-SHA refs remaining. The original Phase-2a §3.1 item-3 commitment (commits `9e68f84` SHA-pin sweep + `e014653` v5 alignment, 2026-04-25) is verified ongoing; Dependabot rotates SHAs weekly via the github-actions package-ecosystem in `.github/dependabot.yml`.
- **Minor improvements landed in PR #150:** `actions/checkout` v4.3.1 → v5.0.1 alignment in `branch-protection-spec-check.yml` (matches workspace-wide pin); comment clarification in `cargo-public-api.yml` (misleading `# nightly via @nightly` comment now clarifies SHA is action code from `stable` branch + `toolchain: nightly` selects compiler). Documented exceptions for `actions/upload-artifact` / `actions/download-artifact` / `actions/setup-node` major-version straddles (Dependabot globally-ignored to avoid Monday-morning PR storms; future planned upgrade wave on a purpose-built branch).
- **Status:** RESOLVED.

### 11.3 R7-equivalent spec-to-code-compliance audit standing pattern — see §7.11b above (pim-13 candidate)

(Cross-reference; full entry at §7.11b.)

