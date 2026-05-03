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

**Phase 2b state:** `crates/benten-engine/src/engine_snapshot.rs:223` opens a `RedbBackend` against a tempdir to hydrate snapshot bytes — works but an architectural smell ("tempdir-as-backend").

**Phase 3 target:** Construct `EngineGeneric<SnapshotBlobBackend>` directly from snapshot bytes; no tempdir hydration step. Read-only Engine instance for snapshot inspection.

**Why deferred:** Same dependency as 1.1 — needs Engine genericism. The Phase 3 wave should bundle 1.1 + 1.2 + the SnapshotBlobBackend trait coverage in a single design pass.

**Touch size:** ~50-100 LOC engine-side (mostly removing the tempdir code path) + whatever SnapshotBlobBackend needs to satisfy the read-only subset of the umbrella trait.

### 1.3 Arc<dyn KVBackend> migration (in-memory backend pivot from wave-5)

**Phase 2b state:** Wave-5 (G10-A wasip1) produced an `InMemoryBackend` impl of `KVBackend` (orchestrator-direct PR #38), but `Engine` is hard-bound to `Arc<RedbBackend>` so the in-memory impl is unused in production. HANDOFF-2026-04-29-morning §4 row 1: "Document tech debt, defer Arc<dyn KVBackend> refactor to Phase 3."

**Phase 3 target:** Item 1.1 is the actual structural fix; this is the same item by another name. Folded into PHASE-3-BUNDLE-1.

### 1.4 Compromise #17 durable module-bytes registry

**Phase 2b state:** Wave-8 ships an in-memory module-bytes registry (`crates/benten-engine/src/engine.rs::register_module_bytes` plus the in-memory active set in `engine_modules.rs::install_module`). Two structural compromises rolled up under Compromise #17 in `docs/SECURITY-POSTURE.md`:
- (a) **Non-validating registration API** — `register_module_bytes` does NOT verify the supplied CID matches `blake3(bytes)`. Validation fires lazily at SANDBOX dispatch when wasmtime parses bytes (`Module::new(&engine, &bytes)` → `E_SANDBOX_MODULE_INVALID`). Wave-8j-cleanup added a 3-LOC `debug_assert` for dev-build fail-fast, but release builds still trust the caller-supplied CID.
- (b) **Process-local lifetime** — registered bytes are not durable across `Engine::open` cycles. The `system:ModuleManifest` zone persists module *manifests* but the actual wasm bytes blob is in-memory only; a process restart drops them.

**Phase 3 target:**
- (a) **CID verification at registration** — durable BlobBackend that computes BLAKE3 + persists bytes by CID. The `register_module_bytes` API can either (i) become CID-validating end-to-end (BLAKE3-on-input + reject mismatch) or (ii) remain caller-supplied-CID with explicit "CID-as-attribution" semantics — Phase 3 picks per UCAN integration design.
- (b) **Durable module-bytes blob store** — Phase 3's `BlobBackend` (likely in `benten-id` or `benten-graph`) keys bytes by CID + persists across restart. `Engine::open` rehydrates the active set from the persisted manifest zone + blob backend.

**Why Phase 3:** The durable BlobBackend requires the GraphBackend umbrella trait (PHASE-3-BUNDLE-1, §1.1) so that `Engine<B>` can carry a generic `Arc<B>` to native (RedbBackend + on-disk blob) vs. browser (BrowserBackend + IndexedDB blob, see §4.1) contexts. Earlier landing isn't possible without §1.1.

**Touch size:** ~150-300 LOC. Runs alongside §4.1 IndexedDB persistence on the browser side.

### 1.5 Compromise #18 durable handler-version chain

**Phase 2b state:** Wave-8f's `register_subgraph_replace` builds a handler-version chain in memory: an in-RAM `HashMap<handler_id, VersionChain>` where each chain holds (anchor_cid, current_version_cid, predecessor_cid, chain_depth). Per-PR audit-class differentiation from #17 (in-memory module bytes) — distinct concerns: #17 is content-bytes; #18 is graph-encoded version metadata. Documented in `docs/SECURITY-POSTURE.md` Compromise #18.

**Phase 3 target:** Lift to the canonical Phase-1-shipped `core::version::Anchor` + Version-Node-chain pattern, persisted via the new GraphBackend umbrella trait. The chain becomes a real graph subtree (Anchor Node + Version Nodes + CURRENT pointer) rather than a side-table HashMap.

**Why Phase 3:** Same dependency as §1.4 — needs the umbrella trait (PHASE-3-BUNDLE-1). The version chain itself IS graph-encoded by design; lifting it to durable backing is mechanical once Engine is backend-generic.

**Touch size:** ~100-200 LOC engine-side + whatever the graph schema for the chain requires. Can land in the same wave as §1.1 + §1.4.

---

## 2. Capability / identity

### 2.1 Durable UCAN backend in `benten-id`

**Phase 2b state:** `benten-caps::UCANBackend` is a `CapError::NotImplemented` stub. Phase 1's `phase-2-backlog.md` §7 already names this. Phase 2b's wave-8c-subscribe-infra adds the SUBSCRIBE delivery-time cap-recheck closure that hooks into the (in-memory) grant store — Phase 3 lifts the grant store to durable backing.

**Phase 3 target:** Full UCAN chain validation + delegation; durable grant store backed by the new graph backend (whichever PHASE-3-BUNDLE-1 produces); `benten-id` crate ships Ed25519 / DID / VC alongside.

**Source:** [`phase-2-backlog.md`](./phase-2-backlog.md) §7.1, §7.2, §7.3, §7.4 — all carry forward to Phase 3 verbatim.

### 2.2 SUBSCRIBE delivery-time cap-recheck threading on durable grants (F6)

**Phase 2b state:** Wave-8c-subscribe-infra wired the SUBSCRIBE delivery-time cap-recheck closure (D5 invariant). The grant store is in-memory per Phase-2b posture; Phase-3 lifts it to durable backing alongside the iroh-fetch path.

**Phase 3 lift:** When the durable grant-store lands, the SUBSCRIBE cap-recheck closure threads the grant-shape query so a partial-revoke (e.g. specific grant revoked but actor still active) cancels the affected subscription path.

**Source:** [`phase-2-backlog.md`](./phase-2-backlog.md) §7.4. Cross-refs `.addl/phase-2b/wave-8-brief.md` §8d-narrative F6.

---

## 3. Networking / sync

### 3.1 Atriums (P2P direct connections via iroh + Loro CRDTs)

**Phase 2b state:** Phase 2b is single-process. No networking surface.

**Phase 3 target:** This IS Phase 3's headline scope per FULL-ROADMAP.md. Iroh (peer-to-peer transport) + Loro (CRDT for collaborative graph merges) + ed25519-dalek + ssi (Ed25519 / DID / VC for identity).

**Source:** [`docs/VISION.md`](../VISION.md) "Atriums (Phase 3 committed) — peer-to-peer direct connections."

### 3.2 Per-subscriber filtering on the change-event stream

**Phase 2b state:** `Engine::subscribe_change_events` fans out every committed `ChangeEvent` without a per-event read-check gate. The Engine instance itself is the security boundary in Phase 2b — single-trust-zone.

**Phase 3 target:** Phase 3 federation / sync introduces cross-trust-boundary replicas; the subscribe path gains per-subscriber filtering at that point.

**Source:** [`phase-2-backlog.md`](./phase-2-backlog.md) §1, [`docs/SECURITY-POSTURE.md`](../SECURITY-POSTURE.md) §"Change-stream subscription bypasses capability read-checks."

---

## 4. Browser / wasm32-unknown-unknown

### 4.1 Compromise #19 — IndexedDB-backed persistent module-manifest store

**Renumbering note:** previously labeled "Compromise #N+8" before R6FP Group 3 globalized the numbering to match `docs/SECURITY-POSTURE.md` (#1-#21).

**Phase 2b state:** `bindings/napi/src/wasm_browser.rs::BrowserManifestStore::is_persistent` returns `false`. Module manifests are in-memory only on the browser target. Compromise #19 in `docs/SECURITY-POSTURE.md`.

**Phase 3 target:** IndexedDB-backed persistent manifest store. Pairs with PHASE-3-BUNDLE-1 (BrowserBackend) since both are browser-target persistence work — likely a single Phase-3 wave covers both.

### 4.2 Compromise #20 — Cross-browser determinism CI cadence

**Renumbering note:** previously labeled "Compromise #N+9" before R6FP Group 3 globalized the numbering to match `docs/SECURITY-POSTURE.md` (#1-#21).

**Phase 2b state:** Per-browser engine bytecode + JIT non-determinism makes per-PR cross-browser CID pinning premature. The cross-browser determinism job in `wasm-browser.yml` is gated on `release` events + `workflow_dispatch` only. Per-PR CI runs the bundle build + size cap + single-browser smoke without pinning a fixture CID across engines.

**Phase 3 target:** Engine-side determinism work that closes the compromise; flip the cross-browser job to per-PR cadence. Source: `docs/future/phase-2-backlog.md` §10.2.

---

## 5. IVM Algorithm B maturity

### 5.1 Drift-detector + non-canonical-view generalization

**Phase 2b state:** Wave-8h wired Algorithm B production registration. The 5 hand-written canonical views (`CapabilityGrantsView`, `ContentListingView`, `EventDispatchView`, `GovernanceInheritanceView`, `VersionCurrentView` — see `crates/benten-ivm/src/views/mod.rs:20-24`) are pure-delegation kernels; non-canonical user-defined view IDs hit a `ContentListingView` fallback (per `docs/INVARIANT-COVERAGE.md` Algorithm B canonical-only compromise note). Wave-8j-cleanup didn't change this. The R6 ivm-correctness lens (`r6-ivm-2`, `r6-ivm-3`) flagged two gaps:
- Drift-detector for IVM canonical-view-vs-Algorithm-B equivalence is named in SECURITY-POSTURE.md:266 + INVARIANT-COVERAGE.md:133 as "on the Phase-3 backlog" but had no actual entry until this section.
- 4 of 5 canonical views silently ignore user-supplied label semantics (e.g. `version_current` + Label("post") registers as `Strategy::B` but VersionCurrentView hardcodes NEXT_VERSION). R6-R3 r6-r3-ivm-1 lands a fail-loud reject for this drift across BOTH the TS-DSL pre-napi-boundary (`packages/engine/src/views.ts::validateUserViewSpec`) AND the Rust engine boundary (`crates/benten-engine/src/engine_views.rs::register_user_view` surfacing `EngineError::ViewLabelMismatch` / catalog `E_VIEW_LABEL_MISMATCH`). Full generalization to "Algorithm B handles arbitrary user-defined label semantics" remains Phase 3.

**Phase 3 target:**
- (a) **Algorithm B drift-detector** — proptest harness that compares Algorithm B incremental updates vs from-scratch full computation across all 5 canonical views + a synthetic user-defined view. Treat divergence as a test failure with structured diff. Generalizes into the Phase-3 IVM CI lane.
- (b) **Non-canonical view generalization** — Algorithm B handles arbitrary `(view_id, label_pattern, projection)` triples; the canonical-only fallback is removed (or kept as fast-path for the 5 known views). User-defined views with custom label semantics no longer silently coerce to `ContentListingView`.

**Why Phase 3:** The drift-detector needs the same surface-completeness Algorithm B's full generalization needs — testing Algorithm B against an arbitrary label pattern requires the generalization itself to exist. Sequencing: (a) and (b) land together in a Phase-3 IVM wave.

**Touch size:** ~400-700 LOC across `crates/benten-ivm/src/` (Algorithm B kernel generalization) + ~200-400 LOC tests (proptest drift detector + per-view-pattern conformance). Risk surface: medium — the 5 canonical views' performance characteristics must be preserved at the fast-path level.

### 5.2 AnchorPrefix selector lift in user-view registration (post-G8-A)

**Phase 2b state:** R6-R3 r6-r3-arch-4 named-destination carry. `Engine::register_user_view` accepts `InputPattern { anchor_prefix: Option<String>, ... }` as part of `UserViewSpec`, but the dispatch path at `crates/benten-engine/src/engine_views.rs::register_user_view` silently coerces `anchor_prefix` → label-equality match (the AnchorPrefix variant feeds the prefix string into the same `input_pattern_label` slot the `Label` variant uses). The pre-G8-A SEMANTIC STUB doc-block at the implementation site is honest about this; the stub bridges through `ContentListingView` until G8-A's per-strategy view dispatch lands. R6 Round 1 (r6-arch-4) flagged that no Phase-3 destination doc named the carry; this entry IS the named destination.

**Phase 3 target:** lift `AnchorPrefix` to genuine prefix matching (e.g. `anchor_prefix="crud:"` matches both `"crud:post"` and `"crud:user"` via a `PrefixMatcher` selector type). Compose with §5.1 generalization so the user-view ingestion path supports per-spec view dispatch with arbitrary `(view_id, label_pattern, projection)` triples + the canonical-only fallback is removed (or kept as a fast-path).

**Why Phase 3:** the AnchorPrefix lift requires the same Algorithm B selector-richness §5.1 covers — testing prefix-not-equality semantics requires the generalized dispatch path itself to exist. Sequencing: lands together with §5.1 in the Phase-3 IVM wave.

**Touch size:** ~30-50 LOC across `engine_views.rs` (extend the matcher), `benten-ivm` subscriber wiring, plus 1 regression test exercising the prefix-not-equality case. Bundles cleanly with §5.1 (~1-2 hour incremental scope).

---

## 6. SANDBOX runtime maturity

### 6.0 D10 read-only-snapshot enforcement at the SANDBOX kv:write extension boundary (forward-pointer)

**Phase 2b state:** R6-R3 r6-r3-arch-2 forward-pointer. Phase 2b's SANDBOX host-fn surface is read-only at the storage layer: `crates/benten-eval/src/sandbox/host_fns.rs::default_host_fns` ships ONLY `time`, `log`, and `kv:read`. There is no `kv:write` host-fn; therefore a Phase-2b SANDBOX module CANNOT bypass D10 read-only-snapshot contract via host_fns — there is no surface to bypass. PR #68 wired `is_read_only_snapshot()` enforcement at `crates/benten-engine/src/primitive_host.rs::put_node`; R6-R3 r6-r3-arch-1 fix-pass extended the same enforcement to `delete_node` via the shared `check_not_read_only_snapshot(op_name)` helper. Both checks fire on the dispatch-through-handler path that `engine.call(handler, ':...', ...)` exercises.

**Phase 3 target:** when the iroh / capability-graph / federation work extends host_fns with `kv:write` (and any future `kv:delete` / edge-mutating host-fn), the read-only-snapshot enforcement MUST live AT the host-fn dispatch boundary in addition to `PrimitiveHost::put_node` / `delete_node`. The SANDBOX call site does NOT flow through the host's `put_node` / `delete_node` trait methods — it goes through the dedicated `kv:write` host-fn behavior bound directly to the wasmtime Linker. A naive wiring that proxies `kv:write` through `PrimitiveHost::put_node` would be safe; a wiring that calls `backend.put_node` directly (e.g. for performance-bypassing buffer/replay) would silently violate D10 against a `from_snapshot_blob`-backed engine.

**The architecturally-cheapest closure** is to either (a) route every storage-mutating host-fn through `PrimitiveHost::put_node` / `delete_node` so the existing helper fires, OR (b) have each host-fn closure independently invoke `Engine::is_read_only_snapshot()` before the backend call. Whichever path Phase-3 picks, the design call should be locked when `kv:write` lands so the seam doesn't reopen as a regression. Bundles cleanly with the broader §6.6 SANDBOX TS-bridge work AND §1.4 durable BlobBackend.

**Touch size:** ~5-10 LOC at the host-fn build site, plus 1 regression test asserting `kv:write` from a SANDBOX module against a `from_snapshot_blob` engine surfaces `E_BACKEND_READ_ONLY` (mirrors the Phase-2b `delete_node` regression test landed alongside r6-r3-arch-1 in the engine-side integration suite).

### 6.1 ESC-16 fingerprint-collapse complete defense

**Phase 2b state:** Wave-8b wired the SANDBOX runtime (Store + Linker + Instance + fuel/memory/wallclock + epoch ticker). 9 of 16 ESC vectors fully fire typed errors. ESC-16 (wallclock fingerprint collapse) has a `.wat` fixture committed but R6 wasmtime-sandbox-auditor (`r6-wsa-3`) flagged that the test bypasses it with an inline shape that doesn't exercise the fingerprint-collapse property end-to-end. Wave-8j-cleanup didn't address this.

**Phase 3 target:** Re-author the ESC-16 test to drive the committed `.wat` fixture through the full wasmtime pipeline + assert the fingerprint-collapse defense fires before guest-observable wallclock divergence. Engine-side memory-read helper for the assertion shape.

**Why Phase 3:** The engine-side memory-read helper needed by the assertion is a Phase-3 surface (wasm32-unknown-unknown browser-target requires a different memory-introspection path than native wasmtime; a unified helper is cleaner once §1.1 + §4.1 land).

**Touch size:** ~80-150 LOC.

### 6.4 Dedicated `E_SANDBOX_STACK_OVERFLOW` typed variant (operator-UX)

**Phase 2b state:** R6 wasmtime-sandbox-auditor `r6-wsa-8` flagged: `Trap::StackOverflow` from wasmtime currently folds into `E_SANDBOX_MODULE_INVALID` reason string in `crates/benten-eval/src/sandbox/trap_to_typed.rs`. R6-FP Group 1 (PR #62) narrowed the reason string within the existing variant (interim disposition); the agent offered to land a dedicated `E_SANDBOX_STACK_OVERFLOW` typed variant as a follow-up, estimated as ~20-site cascade across drift detector + catalog tables + narrative docs (~50-150 LOC).

**Phase 3 target:** Mint dedicated `E_SANDBOX_STACK_OVERFLOW` variant in `crates/benten-errors/src/`; update `trap_to_typed::map_call_error` priority resolver to route `Trap::StackOverflow` to it; update ERROR-CATALOG.md; update SECURITY-POSTURE.md ESC-5 matrix entry; regenerate `errors.generated.ts`; update test pin in `sandbox_escape_attempts_denied.rs:170` (`sandbox_escape_recursive_call_overflow_traps`).

**Why Phase 3 (not 2b):** Operator-UX improvement; current narrowed-reason-string interim is functionally correct (the reason text reads "stack overflow" so operators see the cause). The dedicated variant is a clean catalog-correctness win but the cascade footprint is larger than typical wave-8 tail-cleanup scope. Bundles cleanly with §6.1 ESC-16 + §6.2 D26 in a single Phase-3 SANDBOX-runtime-maturity wave.

**Touch size:** ~50-150 LOC across catalog + narrative docs + test pin.

### 6.3 D18 live-cap-check callback wire-through (ESC-9 cap-revoke mid-call)

**Phase 2b state:** R6 wasmtime-sandbox-auditor `r6-wsa-2` flagged: `live_cap_check` callback in `crates/benten-eval/src/sandbox/host_fns.rs:328-345` is dead surface; D18 PerCall cap-recheck functionally degrades to PerBoundary in production. ESC-9 (cap-revoke mid-call TOCTOU between cap-grant and cap-use) is undefended at runtime today — only the cap-snapshot at SANDBOX entry is consulted. R6-FP Group 1 (PR #62) opted for BELONGS-NAMED-NOW disposition with an in-code `TODO(PHASE-3-BUNDLE-D18-live-cap-check)` marker because the structural lift is >100 LOC + bundles cleanly with Phase-3 grant-store work.

**Phase 3 target:** Wire the callback through:
- (a) Thread `Arc<EngineInner>` + actor reference into `SandboxStoreData::new` at the engine override site so the SANDBOX inner-loop can consult engine-side state.
- (b) Replace the `Vec<String> live_caps` field with a callable that consults the engine's revoked-actors set + future grant-store (UCAN backend, §2.1) on each invocation rather than a snapshot taken at SANDBOX entry.
- (c) Un-ignore the `sandbox_capability_check_per_call_after_revoke` regression test in `crates/benten-eval/tests/sandbox_capability_check_per_call_after_revoke.rs` once the helper exists.
- (d) Update SECURITY-POSTURE.md ESC matrix entry for ESC-9 to "Fully wired" (currently honestly disclosed as "Paper-only `#[ignore]`").

**Why Phase 3:** Bundles with §2.1 (durable UCAN backend) since the grant-store integration is what gives the live-cap-check meaningful state to consult. The wire-through itself is mechanical once the grant-store surface exists.

**Touch size:** ~80-150 LOC engine-eval + 1-2 regression test pin + cross-target pre-flight. Risk surface: low (purely additive defense; no production-runtime semantics regression — current Phase-2b posture is already PerBoundary-effective).

### 6.2 D26 .wasm-bytes-shipping per fixture

**Phase 2b state:** ESC-1..16 fixtures live as `.wat` source compiled at test time (`wat::parse_str(...)`). D26 design intent calls for shipping pre-built `.wasm` bytes per fixture so cross-platform determinism + canonical CID pinning can apply. R6 wasmtime-sandbox-auditor (`r6-wsa-5`) flagged this gap; wave-8b ran out of budget before completing the tooling.

**Phase 3 target:** Build-time tooling that compiles each `.wat` fixture to `.wasm` + commits the resulting bytes alongside the source. Runtime test loader prefers the pre-built `.wasm` (with `.wat` fallback for development). Cross-platform CID pinning then verifies the same fixture serializes identically across native / wasm32-wasip1 / wasm32-unknown-unknown.

**Why Phase 3:** Bundles cleanly with §4.2 (cross-browser determinism CI cadence promotion). Both surfaces want the same tooling.

**Touch size:** ~200-300 LOC tooling + ~50 LOC per-fixture loader update.

### 6.6 TS-side SANDBOX named-manifest resolution + module-bytes registration API

**Phase 2b state:** The Rust-side named-manifest registry (`benten_eval::sandbox::ManifestRegistry` + `Engine::manifest_registry()` projection at `crates/benten-engine/src/engine_modules.rs:431`) keys CapBundle entries by `entry.name` (e.g. `"identity"`), NOT by the colon-joined `"<manifestName>:<entryName>"` shape the TS DSL surface advertises (`SandboxArgsByName.module: "echo:identity"` per `packages/engine/src/types.ts:386`). Wave-8h wired the registry projection but the resolution-from-DSL-shape half is missing on the TS bridge — `register_subgraph` does NOT validate at registration time that a SANDBOX node's `module: "<m>:<e>"` resolves to an installed manifest entry, AND there is NO TS-side `engine.registerModuleBytes(cid, bytes)` API to register the actual wasm bytes (Rust has `Engine::register_module_bytes`, napi-unexposed). Three TS vitest pins were authored RED-PHASE expecting this resolution + registration plumbing:

- `packages/engine/test/install_module.test.ts::"engine.uninstallModule(cid) clean release"` — expects `registerSubgraph` to REJECT with `E_SANDBOX_MANIFEST_UNKNOWN` after uninstall.
- `packages/engine/test/sandbox.test.ts::"compose SANDBOX inside a handler subgraph"` — expects `engine.call(...)` to return `result.ok=true` (which requires real wasm bytes registered + name-resolution at registration).
- `packages/engine/test/sandbox.test.ts::"E_INV_SANDBOX_OUTPUT fires on output > limit (D15 trap-loudly)"` — same shape as above; expects an actual wasmtime-driven oversize emission.

The vitest cluster fix-pass (PR linked from `.addl/phase-2b/r6-r2-fp-vitest-cluster-*`) converted these three pins to `.skip` with a destination-here named-NOW per HARD RULE (rule #12, foundational `feedback_no_defer_HARD_RULE`).

**Phase 3 target (3 coupled deliverables):**
1. **Registration-time SANDBOX manifest validation.** `Engine::register_subgraph` walks the spec for SANDBOX nodes; for each, parse `module` as either `(a)` a bare base32 CID or `(b)` a `"<manifestName>:<entryName>"` lookup. Branch `(b)` is rejected with `ErrorCode::SandboxManifestUnknown` (catalog code `E_SANDBOX_MANIFEST_UNKNOWN`) when the name does not resolve through `installed_modules`. Implementation note: extend `manifest_registry()` to also key entries by the colon-joined name so dispatch + register paths agree on the lookup shape.
2. **TS-side `engine.registerModuleBytes(cid, bytes)` napi method.** Wires through to `InnerEngine::register_module_bytes(cid, bytes)` (already exists Rust-side) so TS callers can ship a real wasm bytes payload. Pairs with §1.4 (Compromise #17 durable module-bytes registry) since the durable backing is what makes this useful end-to-end.
3. **Sandbox.test.ts post-`registerModuleBytes` greens.** Re-pin the three currently-`.skip`'d tests to the production-flow shape: install manifest → register module bytes → register subgraph → call → assert outcome. The fixture wasm bytes ship per §6.2 (D26 `.wasm`-bytes-shipping per fixture).

**Why Phase 3:** All three deliverables compose with already-Phase-3-bundled work — §1.4 (durable module-bytes registry, the natural home for the `registerModuleBytes` API) + §6.2 (`.wasm`-bytes shipping for fixture distribution) + the named-manifest registry's eventual sync-replica replication shape. Landing the TS bridge standalone in Phase 2b would require re-shaping when the durable backing arrives.

**Touch size:** ~150-300 LOC (engine-side validation walk + napi wiring + 3 test re-pins). Risk surface: low — additive surface; existing handlers without SANDBOX or with bare-CID `module` strings continue to work unchanged.

### 6.5 RedbSuspensionStore retention-window override

**Phase 2b state:** The `SuspensionStore::is_retention_exhausted` trait method enforces the SUBSCRIBE persistent-cursor retention window (1000-events / 24h). The in-memory test impl overrides correctly; the production `RedbSuspensionStore` uses the trait default `false` and tracks `delivered_count` + `registered_at` in process-local memory. Consequence: a cross-process re-subscribe past the retention window does NOT surface `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED` because the counters reset on process boot. R6 Round-2 security-auditor (`r6-r2-sec-2`) reissued the Round-1 `r6-sec-4` open finding under HARD-RULE — destination must EXIST + receive entry NOW. Disclosure landed in `docs/SECURITY-POSTURE.md` Compromise #9 closure narrative at the same time as this entry.

**Phase 3 target:** Override `is_retention_exhausted` in `crates/benten-engine/src/suspension_store.rs::impl SuspensionStore for RedbSuspensionStore`. Track `cursor_meta_key(sub) -> (delivered_count: u64, registered_at_unix_secs: u64)` in a redb side-table; `is_retention_exhausted` reads the side-table; `put_cursor` increments `delivered_count` + lazy-creates `registered_at` on first put. Add a round-trip-through-engine-restart regression test that asserts `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED` fires on cross-process re-subscribe past the window. Pairs with §6.3 D18 live-cap-check (both surfaces want the durable subscriber-side-table shape that grant-store work introduces).

**Why Phase 3:** The retention bookkeeping side-table shape composes with the durable grant-store + per-event read-cap-coverage work (§2.2 + `phase-2-backlog.md` §7.4). Landing it standalone in Phase 2b would require re-shaping the side-table when grant-store lands.

**Touch size:** ~50-60 LOC + 1 regression test pin.

### 6.7 AArch64 SANDBOX runtime CI cell (Apple Silicon test execution)

**Phase 2b state:** T4 multi-arch coverage (`.github/workflows/multi-arch-cargo-check.yml`) covers `cargo check --target aarch64-apple-darwin` (compile-only). Apple Silicon SANDBOX runtime behaviour (sigaltstack handler, 16-byte stack alignment + max_wasm_stack interaction with M-series memory model, epoch-deadline thread fairness on the heterogeneous E/P core scheduler) is uncovered at runtime CI. R6 Round 1 wasmtime-sandbox-auditor (`r6-wsa-11`) named `phase-2-backlog.md §10.4` as the destination; R6 Round 3 wasmtime-sandbox-auditor-redux (`r6-r3-wsa-1`) verified neither §10.4 nor any phase-3-backlog §6 sub-section actually contained the entry — HARD RULE clause-(b) violation. This entry is the populated destination.

**Phase 3 target:** Add a `runs-on: macos-latest-arm64` cell to the CI matrix running `cargo nextest run -p benten-eval --target aarch64-apple-darwin --test sandbox_basic --test sandbox_escape_attempts_denied --test sandbox_severity_priority`. Couple to the SANDBOX runtime maturity cluster (§6.1 ESC-16 + §6.4 SandboxStackExhausted) since AArch64-specific surfacing of stack-overflow / fingerprint-collapse defects is most likely to come from this cell.

**Why Phase 3:** Pairs with the broader Phase-3 CI hardening pass; isn't blocking for tag close because compile-only T4 catches the most common cross-arch breakage (type-system / target-feature drift). The runtime-specific surfacing only matters once the ecosystem starts running real workloads against AArch64 production builds.

**Touch size:** ~30-40 LOC YAML + monitor wasmtime upstream's Apple Silicon issue surface.

### 6.8 SANDBOX kv:write read-only-snapshot enforcement seam (folded into §6.0)

**R6-R4 r6-r4-doc-3 dedupe.** §6.0 and this section both named `r6-r3-arch-2` and described the same SANDBOX kv:write read-only-snapshot enforcement seam (PR #70 Group C accidentally created two parallel entries during the R6-R3 docs+cite-precision fix-pass). The canonical content lives at §6.0 above. This stub is preserved (rather than removed) so any in-tree cite of `phase-3-backlog.md §6.8` continues to resolve to the same Phase-3 forward-pointer rather than 404; the wording at §6.0 is the authoritative version.

### 6.9 benten-dev `inspect-state` thin-CLI front-door

**Phase 2b state:** The Rust-side pretty-printer entry point at `tools/benten-dev/src/inspect_state.rs::pretty_print_envelope_bytes` IS shipped, but the wrapping `node bin/benten-dev.mjs` thin-CLI front-door for `benten-dev inspect-state <path>` is not yet shipped. R6 Round 3 stale-deferrals-deep-sweep (`r6-r3-sd-5`) flagged that `tools/benten-dev/test/inspect_state_pretty_prints.test.ts` (1 `describe.skip` + 3 `it.skip`) cited "Phase-2c item" as the destination, but Phase 2c is NOT a defined phase in `docs/FULL-ROADMAP.md` — HARD RULE clause-(b) violation (destination doesn't exist; "Phase 2c" appears informally as a deferred-bucket label in security-posture/error-catalog/host-functions for the deferred `random` host-fn but isn't a real plan-doc / roadmap entry). This entry is the populated destination.

**Phase 3 target:** Ship the `node bin/benten-dev.mjs` thin-CLI front-door wrapping the existing Rust-side pretty-printer. Wire `benten-dev inspect-state <path>` to read the suspended ExecutionState envelope bytes from `<path>` and pretty-print via `pretty_print_envelope_bytes`. Un-skip the 1 describe + 3 it tests in `inspect_state_pretty_prints.test.ts`.

**Why Phase 3:** The benten-dev thin-CLI surface is part of the broader Phase-3 DX hardening pass; the Rust-side entry point is shipped, so the test bodies pin the public-facing surface that lands in Phase 3 hygiene. Bundles cleanly with the rest of the Phase-3 first-wave CI-hygiene cluster (§7.3.A).

**Touch size:** ~30-50 LOC TS CLI wrapper + the 4 test un-skips.

---

## 7. Observability + diagnostic completeness

### 7.1 SANDBOX execution metrics propagation (Compromise #17 reinforcement)

**Phase 2b state:** R6 metadata-producer-vs-consumer audit (`r6-mpc-3`) + R6 napi-bindings (`r6-napi-3`) + R6 dx-optimizer (`r6-dx-10`) — three lenses converged — flagged: `Engine::describe_sandbox_node` claims `fuel_consumed_high_water` + `last_invocation_ms` metrics, but `SandboxResult.fuel_consumed` + `output_consumed` are dropped at `crates/benten-engine/src/primitive_host.rs:899-906` (only `output` propagates). The diagnostic surface always returns `Err(Unknown)`; TS surface synthesizes hardcoded defaults client-side. Wave-8j R6-FP landed the doc-fix variant (honest "unknown" route + Compromise #17 reinforcement narrative); full metric-propagation deferred here.

**Phase 3 target:** Thread `fuel_consumed` + `output_consumed` + `last_invocation_ms` through the engine wrapper into a per-node high-water tracker on the `SandboxNodeState` side-table. Surface via `describe_sandbox_node` returning real values. ~150 LOC + 1 regression test pin per metric.

**Why Phase 3:** Side-table schema for `SandboxNodeState` is Phase-3-shaped (durable across restart implies the GraphBackend umbrella trait, §1.1). Metrics-in-RAM-only without §1.1 would land then need re-shape immediately when §1.1 does.

**Touch size:** ~150-200 LOC.

### 7.1.1 SnapshotBlobBackend metric-propagation entry (cross-ref §1.2)

**Phase 2b state:** §7.1 above describes SANDBOX execution-metrics propagation (`fuel_consumed`/`output_consumed` propagation through engine wrapper into a per-node high-water tracker). The SnapshotBlobBackend direct-wire (§1.2) is the structural unblocker because the per-node side-table that holds those metrics lives in the GraphBackend umbrella trait the genericism unlocks. R6-FP Group 2 PR #61 docstrings cite this entry by name (`packages/engine/src/engine.ts::describeSandboxNode` JSDoc + `bindings/napi/src/sandbox.rs:108-119` comment block).

**Phase 3 target:** Same as §7.1 — this is a re-naming to match what Group 2's TS docstrings cite. SANDBOX metric-propagation lands together with SnapshotBlobBackend direct-wire (§1.2) because both want the same per-call-state side-table on the new backend trait.

**Touch size:** Folded into §7.1 + §1.2; no separate budget.

### 7.1.2 openStream FinalizationRegistry leak detector + requiresExplicitClose accessor

**Phase 2b state:** R6-FP Group 2 picked the honest-downgrade path for r6-stream-1: `openStream`'s `requires_explicit_close` lifecycle is enforced server-side but NOT exposed at the JS surface. `engine.callStream` and `engine.openStream` are functionally indistinguishable from the JS caller's perspective today. The honest disclosure lives in `packages/engine/src/engine.ts::openStream` JSDoc.

**Phase 3 target:** Wire two pieces:
- (a) **napi accessor `requiresExplicitClose()` on StreamHandle** — exposes the server-side flag to JS.
- (b) **TS-side `FinalizationRegistry` leak detector** — fires `E_STREAM_HANDLE_LEAKED` when a handle held by GC carries the flag set + was never explicitly closed.

Together they realize the cr-r4b-10 closure-narrative claim that `E_STREAM_HANDLE_LEAKED` fires on a leaked open-stream handle.

**Touch size:** ~30-40 LOC napi + ~20-30 LOC TS + 1 leak-detector test. Risk surface: low (purely additive observer; no production-runtime semantics change).

### 7.1.3 UserView.snapshot() + onUpdate() runtime materialization (post-G8-B)

**Phase 2b state:** G8-B (PR #28) shipped engine + DSL surface for user-registered IVM views. The TS-side `UserView` type is registered + dispatchable today, but the JS-observable runtime accessors (`view.snapshot()` returning current materialized state + `view.onUpdate()` returning an async iterator of incremental deltas) were red-phase-deferred. R6-FP Group 2 PR #61 `packages/engine/test/views.test.ts:32-50` `.skip` rationale names this entry as the destination.

**Phase 3 target:** Implement the two runtime accessors:
- (a) `view.snapshot(): Promise<T[]>` — returns current materialized rows from the IVM-maintained side-table; consults the canonical view registry's read-path.
- (b) `view.onUpdate(): AsyncIterableIterator<ViewDelta<T>>` — yields incremental deltas as ChangeEvents commit + Algorithm B maintains the view; consumed via `for await`.

**Why deferred:** The runtime-materialization path consumes the same per-view side-table that §7.1 metric propagation needs (lives in the GraphBackend umbrella trait). Phase-3's IVM Algorithm B generalization (§5.1) is the natural bundling site.

**Touch size:** ~150-250 LOC engine + napi + TS + 3-5 regression tests. Risk surface: medium (introduces a new public API).

### 7.1.4 WAIT TTL TS DSL + suspend/resume DX surface widening (post-G12-E)

**Phase 2b state:** G12-E (PR #43, #57) shipped the engine-side WAIT envelope + suspension store. The TS-side DSL helpers for WAIT-TTL (declarative time-bounded waits with auto-resume on TTL expiry) + ergonomic suspend/resume call shapes are red-phase-deferred. R6-FP Group 2 PR #61 `packages/engine/test/wait_ttl.test.ts:34-36` `.skip` rationale names this entry as the destination.

**Phase 3 target:** Widen the TS DSL with:
- (a) `subgraph(...).waitWithTtl(signal, { ttlMs })` builder method — declarative TTL on a WAIT primitive.
- (b) `engine.callWithSuspension(handler, args)` returning `{ kind: 'suspended', handle, stateCid, signalName } | { kind: 'complete', result }` — already partially landed (Round-2 Instance 12 wired stateCid + signalName); Phase 3 adds the matching `engine.resumeWithMeta(handle, { signal, payload })` ergonomic wrapper.
- (c) `engine.testingAdvanceWaitClock(ms)` testing helper — currently the test file references it but the napi binding doesn't expose it.

**Why deferred:** The `testingAdvanceWaitClock` helper requires test-mode mock-clock plumbing that crosses the napi boundary; Phase-3's broader engine clock-injection work bundles cleanly.

**Touch size:** ~80-150 LOC TS surface + ~30-50 LOC napi binding + 5-7 regression tests + DSL spec doc updates.

### 7.1.5 STREAM ESC defenses per-handler configurability (per-handler chunk-count + wallclock-budget)

**Phase 2b state:** R6 Round 1 streaming-systems lens (`r6-stream-5`) flagged that the STREAM primitive's ESC defenses (chunk-count cap + per-call wallclock budget) are workspace-global constants today rather than per-handler-tunable knobs. R1 disposition was BELONGS-ELSEWHERE-NAMED to "phase-2-backlog.md §10.4 (or new §10.5 STREAM widening)"; R6 Round 3 streaming-systems-redux (`r6-r3-stream-OOS-2`) verified neither destination was populated and surfaced the partial-fail of HARD RULE clause-(b). This entry is the populated destination.

**Phase 3 target:** Lift the chunk-count cap + wallclock-budget for STREAM out of workspace-global constants into per-handler `SubgraphSpec.primitives` properties (mirrors the SANDBOX `wallclock_ms` / `output_max_bytes` per-handler-knob shape per D24/D15). Wire the per-handler reads through the STREAM executor at primitive-entry; surface validation failures as registration-time `E_INV_STREAM_CONFIG` typed-error if the configured values exceed capability-grant ceilings.

**Why Phase 3:** Pairs with Phase-3 STREAM/SUBSCRIBE end-to-end work in §7.3.A.2 (test bodies pinning the configurability surface) + the broader per-handler knob taxonomy that SANDBOX already established (so STREAM lands as the second instance of a now-codified pattern rather than as a one-off knob set).

**Touch size:** ~50-80 LOC eval-side per-handler config read + ~20 LOC registration-time validation + ~30-50 LOC test pins.

### 7.2 BentenError.context full structured-field coverage

**Phase 2b state:** R6 deep producer-consumer sweep (Instance 8) flagged: every typed `EngineError` variant with structured fields drops them at the napi → TS boundary because `engine_err()` formats Display-only and `mapNativeError` extracts only the `E_*` code. Wave-8j R6-FP Groups 1+2 land the MINIMAL fix: `napi::Error::with_metadata` carries a JSON-serialized field bag for the most-load-bearing variants (`Invariant(RegistrationError)` + `ModuleManifestCidMismatch` + `IvmViewStale` + ~5 others). Full coverage of all ~20 EngineError variants + the long tail of `EvalError` deferred here.

**Phase 3 target:** Replace the message-prefix-`E_*` carrier shape with a JSON-shape (`{ code, fields }`) at the napi boundary so ALL typed-error variants get structured-field surfacing automatically. Migrate `mapNativeError` to read from the JSON shape consistently. Update `errors.generated.ts` codegen to emit the structured-field interfaces per variant.

**Why Phase 3:** The migration has a coordinated breaking-change to the message-prefix contract test pins; Phase-2b-close stability favors the minimal-coverage interim. Phase 3's broader API stabilization can absorb the breaking change cleanly.

**Touch size:** ~300-400 LOC including codegen updates + test pin migration.

### 7.6 CODE_TO_CTOR codegen completeness

**Phase 2b state:** `packages/engine/src/errors.ts::CODE_TO_CTOR` is a hand-maintained Record mapping `E_*` strings to typed BentenError subclasses. R6 Round-2 r6-r2-napi-3's Instance 8 round-trip pin (the new `install_module` CID-mismatch test in `packages/engine/test/install_module.test.ts`) surfaced that the map is missing ~28 entries that the codegen emits as classes — so napi errors carrying those codes round-trip through `mapNativeError` with `code: "E_UNKNOWN"` rather than the typed subclass. R6 Round-2 fix-pass added the specific `E_MODULE_MANIFEST_CID_MISMATCH` entry to make the Instance 8 pin pass + named this entry as the destination for the broader sync.

**Phase 3 target:** Generate `CODE_TO_CTOR` from the same single-source-of-truth that powers `errors.generated.ts` (the catalog scrape that emits 98 BentenError subclasses). Either (a) emit a generated `CODE_TO_CTOR_GENERATED` in `errors.generated.ts` that the hand-maintained `CODE_TO_CTOR` extends from, or (b) replace the hand-maintained map entirely and update `mapNativeError` to read from the generated record. Add a vitest smoke test that asserts every catalog code maps to a typed BentenError subclass (no `E_UNKNOWN` fallbacks for known codes).

**Why Phase 3:** The fix is mechanical but interacts with the codegen template + drift detector. Bundling with §7.2 (BentenError.context full structured-field coverage) is natural because both are codegen-completeness lifts on the TS error surface.

**Touch size:** ~50-100 LOC codegen template update + ~10 LOC vitest smoke pin.

### 7.7 napi-rs ThreadsafeFunction tuple-arg splat behavior

**Phase 2b state:** napi-rs v3's `Function<(A, B), Ret>` callback shape currently delivers the `(A, B)` tuple as a single-Array argument to the JS callback rather than splatting to 2 separate args, despite the d.ts emitting `(arg0: A, arg1: B) => Ret`. Affects both `Engine.onChange` (`(seq, payload)`) and the new `Engine.onEmit` (`(channel, payloadJson)`) callback shapes — the JS callback receives `args[0] = [a, b]` rather than `(a, b)`. The R6 Round-2 r6-r2-mpc-1 LOAD-BEARING test in `packages/engine/test/emit_subscribe.test.ts` accepts both delivery shapes via an `Array.isArray(channel)` runtime check; the pre-existing `subscribe.test.ts::LOAD-BEARING — onChange callback fires` test predates the workaround + currently fails on the same delivery shape. The napi-side wiring is correct (the engine-side EMIT broadcast publish IS firing + the TSFN IS delivering); the gap is the splat semantics on the napi-rs ↔ JS call edge.

**Phase 3 target:** Investigate napi-rs v3 release notes for the splat-behavior change between Phase-2a and Phase-2b napi-rs upgrades. Either (a) bump napi-rs to a version with restored splat semantics + remove the in-test `Array.isArray` workaround, or (b) update the engine.ts wrapper's `napiCb = (chanArg, payloadJson) => ...` shape to take a single tuple-arg + destructure inside, and update `subscribe.test.ts::LOAD-BEARING` similarly. Pair with §7.6 (CODE_TO_CTOR codegen completeness) since both touch `errors.generated.ts` codegen + the napi binding.

**Why Phase 3:** The functional behavior (callback fires) is correct in Phase 2b; only the arg-shape ergonomics are degraded. Tightening the splat is a Phase-3 napi-rs lift that bundles cleanly with broader binding-layer cleanup.

**Touch size:** ~30-50 LOC across napi-rs upgrade + test pin updates.

### 7.8 Engine.emitEvent standalone surface — wire through EmitBroadcast bus

**Phase 2b state:** `Engine.emitEvent(name, payload)` (TS at
`packages/engine/src/engine.ts:1228-1248`) and the matching napi
adapter `emit_event` (`bindings/napi/src/lib.rs:548-562`) both surface
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

**Phase 3 target:** Thread `Engine::emit_event(channel, payload)`
directly through the EmitBroadcast bus (the same channel
`subscribe_emit_events_with_handle` consumes). Decide on the
structured-payload story (likely: accept `JsonValue` payload, route as
the `payload` field of the EmitBroadcast event). Add an end-to-end
vitest pin: `engine.onEmit(channel, cb)` → `engine.emitEvent(channel,
{...})` → callback fires with the payload.

**Why Phase 3:** Phase 2b closed in-handler EMIT + EmitSubscription
delivery; standalone JS-surface emit is a small but separate plumbing
path that bundles cleanly with the broader Phase-3 event-broadcast
widening (cross-process / cross-actor delivery).

**Touch size:** ~50 LOC implementation + ~20 LOC test pin.

### 7.9 TS-surface-parity sweep (Edge interface phantom `cid` + dropped `properties`; broader latent pre-Phase-2b TS-side drift)

**Phase 2b state:** R6-R4 producer/consumer-deep-sweep-redux surfaced a pre-Phase-2b TS-surface drift candidate that is OUT-OF-SCOPE for the Phase-2b-close tag (named-destination-here per HARD RULE rule (b) + foundational `feedback_no_defer_HARD_RULE`):

- `packages/engine/src/Edge` interface (`packages/engine/src/types.ts::Edge`) declares `{ cid: string, source, target, label }` — 4 fields. The napi producer at `bindings/napi/src/edge.rs::Edge::to_json` emits `{ source, target, label, properties? }` — 4 fields with TWO mismatches: (a) the TS interface declares `cid: string` but the napi producer never emits a `cid` field on the edge JSON (any TS caller reading `edge.cid` gets `undefined` at runtime); (b) the TS interface OMITS `properties` while the napi producer emits it when present (any TS caller wanting `edge.properties` hits a TS compile error).
- Origin: PR `3fc5262` `fix(dx)` from 2026-04-19 (Phase-2a R6 DX work, NOT Phase-2b). Preserved through every Phase-2b R5 wave + every R6 round (R6-R1 / R6-R2 / R6-R3 deep-sweep / R6-R3 narrow-iteration) without surfacing because the existing producer/consumer audits walked the producer-emits-field-vs-consumer-drops-field shape; the Edge case is the INVERTED shape (consumer-declares-field-vs-producer-doesn't-emit-it) which the Phase-2b-bounded sweeps did not target.
- Behavioral consequence in Phase 2b: zero packages/engine/test/ exercise either `edge.cid` or `edge.properties`, so no test fails today; but any user-code TS caller that consults `edge.cid` for content-addressing or expects `edge.properties` for graph-shape introspection silently mis-behaves.

**Phase 3 target:** A one-shot exhaustive TS-interface-vs-Rust-producer-shape sweep across `packages/engine/src/types.ts` + `bindings/napi/src/`. Mechanical procedure:

1. Enumerate every `pub struct` / serde-derived `pub enum` in `bindings/napi/src/*.rs` that flows to JS via napi.
2. For each, walk the corresponding TS interface in `packages/engine/src/types.ts` and assert field-for-field parity (modulo by-design omissions like `Node.anchor_id` per `#[serde(skip)]` + `crates/benten-core/src/lib.rs:162` Phase-1 architectural decision).
3. Document each by-design asymmetry with a `// (intentionally NOT mirrored: <reason>)` line so future sweeps don't re-flag.
4. Fix all unintentional drift inline (likely `Edge.cid` removal + `Edge.properties` addition; possibly other instances surfaced by the sweep).
5. Add a Rust-side schema-parity meta-test (analogous to `manifest_schema_parity_pin.rs`) that walks the napi struct surface + asserts every public field has a TS-side counterpart by reading the dist `.d.ts` at test time, so the SAME drift cannot recur silently.

**Why Phase 3:** Out-of-scope for Phase-2b R6-R4 close (R6-R4 lens scope is post-R6-R3-FP delta, not pre-Phase-2b legacy); the broader TS-surface-parity work bundles cleanly with the Phase-3 first-wave CI-hygiene cluster (`§7.3.A`) since both surfaces want the same TS-side audit infrastructure. Out-of-the-band of Phase-2b's "21-now-bumped-to-21-or-22 producer/consumer drift instances" running tally because the legacy drifts predate the methodology.

**Cross-references:**
- `.addl/phase-2b/r6-r4-producer-consumer-deep-sweep.json` — surfacing finding (`near_findings_examined_and_dismissed.candidate.Edge interface`).
- `bindings/napi/src/edge.rs::Edge::to_json` — Rust producer.
- `packages/engine/src/types.ts::Edge` — TS consumer (drift surface).
- `crates/benten-core/src/lib.rs:162` — by-design `#[serde(skip)] anchor_id` precedent for documenting intentional omissions.

**Touch size:** ~80-150 LOC across `packages/engine/src/types.ts` (interface parity edits) + 1 Rust meta-test pin (~50-80 LOC) + cross-target pre-flight sweep. Risk surface: low — the additions are typed-surface widenings that existing TS callers don't depend on (zero current consumers).

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

#### 7.3.A.1 — Runtime SANDBOX invariant + attribution-frame test bodies (G7-A/B/C structurally landed)

**Phase 2b state:** G7-A (`a9758f8`) + G7-B (`097d66f`) + G7-C (`468b3ab`) all merged with the structural surfaces in place; SANDBOX runs through wasmtime per-call. The `todo!()` bodies in this cluster pin Inv-4 (sandbox depth runtime threading) + Inv-7 (output trap-loudly) + sec-pre-r1 closure claims. These overlap with the SECURITY-POSTURE.md "Honest disclosure — Inv-4 runtime threading is structural, not transitive" section which records that `AttributionFrame.sandbox_depth` is constructed but the depth-counter machinery has no production call site.

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

**Phase 2b state:** G6-A (`e13e796`) + wave-8c production-runtime wire-through (`443590f`) both landed. The eval-side STREAM/SUBSCRIBE primitives execute; the bodies below are end-to-end engine integration tests that exercise the streaming back-pressure path through napi.

**Files:**
- `crates/benten-engine/tests/integration/subscribe_emit.rs` — 1 test (SUBSCRIBE-emits-on-EMIT-broadcast)
- `crates/benten-engine/tests/integration/stream_composition.rs` — 2 tests (STREAM-into-STREAM + STREAM-into-CALL)
- `crates/benten-engine/tests/integration/engine_stream.rs` — 2 tests (lines 356, 375; STREAM E2E shape)
- `crates/benten-engine/tests/integration/stream_napi.rs` — 1 test (napi async-iterator surface E2E)

**What landing each requires:** integration drivers that exercise the live STREAM `Stream<Item = Vec<u8>>` surface through the engine + napi boundary; back-pressure assertions that probe the chunk-sink scheduler.

**Touch size:** ~150-250 LOC test source. Risk surface: low.

#### 7.3.A.3 — User-view Strategy-A/C rejection + view-registry label-hint test bodies (G8-B landed)

**Phase 2b state:** G8-B (`71dff61`) + wave-8h IVM Algorithm B production registration both landed. The view registry routes Strategy::B to AlgorithmBView for the 5 canonical view IDs; user-defined Strategy::A/C are now rejected at registration (the documented behaviour) but the test bodies are `todo!()`.

**Files:**
- `crates/benten-engine/tests/user_view_strategy_a_rejected_for_user.rs` — 2 tests (Strategy::A rejection + Strategy::C reserved-for-Phase-3 path)
- `crates/benten-engine/tests/view_id_label_hint_refactor.rs` — 2 tests (view registry-driven label hint + canonical Phase-1 view registry coverage)

**What landing each requires:** test driver that constructs view specs with user-defined strategies and asserts the registration-time rejection error code; refactor of label-hint logic from string-prefix-strip to registry lookup.

**Touch size:** ~100-150 LOC test source. Risk surface: low.

#### 7.3.A.4 — Module-install dual-CID + summary mismatch error body (G10-B landed)

**Phase 2b state:** G10-B (`dcfc108`) merged with `Engine::install_module` + `uninstall_module` APIs. The dual-CID error narrative for CID-mismatch is partially implemented but the test body that asserts the "both expected and actual CID + summary metadata in the error" is `todo!()`.

**Files:**
- `crates/benten-engine/tests/module_install_rejects_cid_mismatch_dual_cids_in_error.rs` — 1 test (D16 dual-CID + summary in mismatch error body)

**What landing each requires:** test driver that constructs a module manifest pointing at the wrong CID, calls `Engine::install_module`, and asserts the resulting `BentenError` carries both `expected_cid` and `actual_cid` plus a structured summary field.

**Touch size:** ~30-50 LOC test source. Risk surface: low.

#### 7.3.A.5 — Doc-drift detector test bodies (G12-B + G11-2b-A landed)

**Phase 2b state:** G12-B (`edb1f93`) + G11-2b-A (`8169807`) both merged with the docs sweep + DSL-SPECIFICATION.md finalization + SECURITY-POSTURE.md Phase-2b compromises + ARCHITECTURE.md 8-crate count. The doc-drift detectors that read the .md files and assert structural invariants have `todo!()` bodies.

**Files:**
- `crates/benten-engine/tests/architecture_md_8_crate_count_after_dsl_compiler.rs` — 2 tests (8-crate assertion + benten-dsl-compiler crate creation pin)
- `crates/benten-engine/tests/dsl_specification_md_finalization.rs` — 1 test (DSL-SPECIFICATION.md finalization assertions)
- `crates/benten-engine/tests/security_posture_md_phase_2b_compromises_documented.rs` — 1 test (Phase-2b compromise additions assertion)
- `crates/benten-engine/tests/error_catalog_md_drift_phase_2b.rs` — 2 tests (Phase-2b code presence + fix-hint format enforcement)
- `crates/benten-engine/tests/quickstart_md_walkthroughs_compile.rs` — 1 test (QUICKSTART.md walkthroughs compile)

**What landing each requires:** test bodies that parse the markdown via a structured-section reader and assert the documented invariants. The QUICKSTART.md walkthroughs-compile test needs a build harness that extracts code blocks and runs them through `cargo build`.

**Touch size:** ~200-300 LOC test source. Risk surface: low.

#### 7.3.A.6 — WAIT TTL runtime expiry path test bodies (G12-E landed structurally; runtime path Phase-3)

**Phase 2b state:** G12-E (`0ac7b0a`) + wave-8i WAIT production runtime (`55b084a`) both landed. `SuspensionStore`, `resume_with_meta`, `ttl_hours` metadata, the `WaitTtlExpired` + `WaitTtlInvalid` error codes, and the engine clock override are all on main. The remaining gap is the runtime TTL **expiry path** — the deadline check + GC sweep that converts metadata into typed errors. Wave-8i wired the deadline consultation; the GC + cross-process expiry semantics are deferred.

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

#### 7.3.A.7 — Wave-8b/8c "paired" testing helpers — security-critical SANDBOX-escape pins (ESC-9/-10/-15 etc.)

**Phase 2b state:** Wave-8b (`1f11c61`) shipped the wasmtime trampoline + per-call Store discipline; wave-8c (subscribe-infra + stream-infra + 8c-cont) all merged. The eval-side ESC-pin tests reference testing helpers (`testing_revoke_cap_mid_call`, `testing_call_engine_dispatch`, `testing_inject_forged_cap_claim_section`, `testing_register_uncounted_host_fn`) that the rationales claim are "paired with 8c work" but never actually shipped. **These are SECURITY-CRITICAL ESC-pin tests** — the `SECURITY-POSTURE.md` ESC matrix at §"Compromise #4" honestly discloses ESC-9 / ESC-10 / ESC-13 as "Partial / eval-side smoke" with the helper-fn smoke tests verifying trampoline accounting; the integration-shape pins below stay reserved.

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

#### 7.3.A.8 — wasmtime Component-Model gated SANDBOX-escape tests (wsa-3 removed feature)

**Phase 2b state:** wsa-3 explicitly removed Component-Model from Phase 2b scope. The two ESC-11/-12 tests (Component-Model type-mismatch + resource-handle-forgery) are `#[cfg(feature = "component-model")]`-gated AND `#[ignore]`'d. SECURITY-POSTURE.md ESC matrix records both as "Component-model gated (2): full coverage requires wasm-component-model surface; current defense rejects via `Module::new` structural validation."

**Files:**
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:311` — ESC-11 component-type-mismatch
- `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs:328` — ESC-12 resource-handle-forgery

**What landing each requires:** Phase 3 must first re-evaluate the Component-Model adoption decision (whether wsa-3 holds or whether wasmtime's component-model story is mature enough to re-enable the cargo feature). If Component-Model is re-enabled, both test bodies fold into §7.3.A.1's broader runtime-SANDBOX cluster.

**Touch size:** ~30-50 LOC test source (after Component-Model re-enabled). Risk surface: low (gated, opt-in).

**Cross-ref:** §6 "SANDBOX runtime maturity" + Phase-3 plan-doc opening checklist item that explicitly asks "do we re-open Component-Model?"

#### 7.3.A.9 — Workflow-baseline + browser-bundle artifact + subscribe persistent-cursor helpers (post-deep-sweep additions)

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

---

### 7.3.C — STALE-RATIONALE-NO-DESTINATION fixes (HARD-RULE compliance)

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

---

## 7.5 cargo-llvm-cov coverage workflow investigation (anytime)

**Phase 2b state:** `.github/workflows/coverage.yml` runs `cargo-llvm-cov` to produce HTML + summary + lcov coverage reports. It's been failing on main since at least wave-8c-stream-infra (verified `c030fed` red). Informational by workflow design (not in `.github/branch-protection.yml` required-status-checks list). Likely caused by either (a) the same test surface drift that affects the vitest informational workflow (since coverage runs the full test matrix under instrumentation), (b) a coverage-tool config issue with the wave-8 surface additions, or (c) the same Intel-Mac timeout family fixed for nextest in PR #59 §1.

**Phase 3+ target:** Diagnose root cause + restore green. If informational stays informational (likely), no urgency; if it should graduate to required for release-readiness, this is the unblocker.

**Touch size:** ~30-100 LOC tooling/config; investigation surface unbounded.

## 7.4 CI lint: file:line cite drift detector (anytime / Phase-3+)

**Source:** R6 Round 1 cite-precision-drift deep retrospective sweep (2026-04-29) found **13 instances** of stale `file.rs:LINE` cites across docs + Rustdoc + test files. Six fixed inline by Group 3 + mini-review-#60-fix-pass + r6fp-tail-comprehensive (post-Group-1-merge). The pattern recurs because file:line cites are inherently fragile against any code edit.

**Phase 3+ target:** Add a CI lint that:
- Greps for `\.(rs|ts):\d+(-\d+)?` patterns in `docs/**/*.md` + Rust doc-comments + TS JSDoc
- For each, verifies the cited line range exists at the cite's commit
- For each, optionally verifies a sentinel anchor (e.g. function name) appears at the cited line range — protects against "file shrunk + line range now points elsewhere"
- Surfaces drift as a non-blocking PR comment (initially) or a required check (later)

**Scope:** ~150-300 LOC tooling + 1 CI workflow. Risk: low (purely additive observer).

**Why deferred:** Phase 2b focuses on engine correctness + production runtime; doc-discipline tooling competes with structural work. R6's deep-sweep methodology (3+-recurrence triggers a per-pattern audit) is sufficient interim coverage.

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
