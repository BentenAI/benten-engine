# Phase 1 Implementation Plan — Benten Engine

**Pipeline stage:** Pre-R1 (planning).
**Source of scope truth:** `CLAUDE.md` "Phase 1 Scope" + `docs/FULL-ROADMAP.md` Phase 1 + `docs/ENGINE-SPEC.md` §14 Phase 1 (all three reconciled 2026-04-14).
**Feeds into:** R1 spec review (5 agents), then R2 test landscape, then R3–R6.
**Current baseline:** Spike `phase-1-stack` validated (see `SPIKE-phase-1-stack-RESULTS.md`). 6-crate workspace compiles, content hashing is deterministic intra-process / cross-process / wasm-compile-check, redb backend CRUDs Nodes, napi-rs v3 bindings expose `initEngine` / `createNode` / `getNode` build-checked.

---

## 1. Executive Summary

Phase 1 ships a working, embeddable, content-addressed graph engine with a TypeScript developer surface good enough that a developer can `npx create-benten-app`, write `crud('post')`, and get a CRUD handler whose audit trail they can visualize — in under ten minutes, without knowing Rust exists. Under the hood, the engine enforces eight of the twelve operation primitives (the eight `crud` actually walks), eight of the fourteen structural invariants (the ones that don't require WASM), persists via redb with durable commits and a change-notification stream, maintains five hand-written IVM views incrementally, and runs capability checks through a pluggable hook that ships with a zero-auth default.

The four primitives and six invariants **not** shipped (WAIT, STREAM, SUBSCRIBE-as-user-op, SANDBOX; invariants 4/7/8/11/13/14) are explicit Phase 2 scope per the reconciliation. Phase 1's architectural thesis — that a thin code-as-graph engine with a pluggable capability hook and an IVM subscriber can out-compete PostgreSQL + AGE on hot-path CRUD — is testable at Phase 1 exit. Phase 2 proves the same thesis extends to sandboxed computation and reactive streams.

**Headline exit criterion:** `npm install @benten/engine && npx create-benten-app my-app && cd my-app && npm install && npm test` exits zero. The scaffolder ships a single Vitest file (`my-app/test/smoke.test.ts`) that mechanically asserts **six Phase 1 behaviors** — each `expect(...)` is an R5-verifiable gate:

1. **Registration succeeds.** `engine.registerSubgraph(crud('post'))` returns a handler id whose `actions` include `create / get / list / update / delete`; invariants 1/2/3/5/6/9/10/12 all pass; no `E_INV_*` thrown.
2. **Three creates + list reflects all three.** `engine.call('post:create', …)` × 3, then `engine.call('post:list')` returns those posts — exercises the 8-primitive evaluator + transaction primitive + View 3 (content listing) incremental maintenance + deterministic `createdAt` injection.
3. **Typed-error surface on unregistered handler.** Calling `engine.call("no-such-handler", ...)` throws an error whose `.code` is `E_DSL_UNREGISTERED_HANDLER` — exercises the typed-error contract (`mapNativeError` routes native failures through a stable `err.code` surface). *Substitution note:* the capability-denial-to-`ON_DENIED` variant this gate was originally specified against requires the Phase-2 `capability`-option on `crud()`; until that lands, the same typed-error-surface contract is verified through the unregistered-handler path. The capability-denial behavior itself is covered by `crates/benten-engine/tests/integration/` (Rust-side) and by `packages/engine/src/errors.test.ts` (TS-side) — not at the scaffolder smoke layer.
4. **Trace has non-zero per-step timing.** `engine.trace(handlerId, input)` returns a `TraceStep[]` where at least one step has `durationUs > 0` — exercises `engine.trace()` DX path and confirms the evaluator isn't shortcut-executing.
5. **Mermaid output shape.** `handler.toMermaid()` returns a string matching the `flowchart` grammar (begins with `flowchart (TD|LR|TB|BT|RL)`, contains at least one `-->` edge, at least one `[...]` node label). *Substitution note:* the original gate specified parser-based verification via the official `@mermaid-js/parser` package; that package ships `info`, `packet`, `pie`, `architecture`, `gitGraph`, `radar`, and `treemap` parsers but NOT a `flowchart` parser — so a regex-over-grammar assertion is the most honest Phase-1 gate. Browser rendering is unchanged.
6. **TS ↔ Rust CID round-trip.** Create a Node, read it back by CID, assert the re-read CID string equals the created CID string — exercises the full bindings stack + content-addressing via a deterministic roundtrip. *Substitution note:* the canonical-fixture CID (`bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`) assertion is covered at the crate level (`benten-core/tests/spike_fixture_cid_stable.rs`) and by the T9 cross-leg determinism CI job; the scaffolder smoke tests the user-observable roundtrip contract (TS → Rust → TS stays byte-identical) without pinning to the specific fixture at the user-visible layer.

Dev server with hot reload is **Phase 2** (per Rank 10). Phase 1 ships the `npm test` path because it is mechanically verifiable: R5 implementers can run it locally, CI enforces it on every PR, and the exit criterion stops being a judgment call.

---

## 2. Scope Inventory (by crate)

Legend for **Status at spike end**: `present` (real implementation shipped in spike) · `stub` (crate exists, one-line marker) · `absent` (no code).

### 2.1 `benten-core`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| C1 | `Node`, `Value` (Null/Bool/Int/Text/Bytes/List/Map), `Cid` types | present | Keep as-is; add `Edge` type (see C2) | — | Spike-validated; CID fixture pinned |
| C2 | `Edge` type (EdgeId, source Cid, target Cid, label, optional properties) | absent | Ship full type with DAG-CBOR round-trip and `Edge::cid()` (edges are independently content-addressed for integrity; inclusion in Node hash stays excluded per §7) | — | New in Phase 1 |
| C3 | `Value::Float(f64)` with NaN rejection + shortest-form encoding | absent (deferred from spike) | Ship with NaN / ±Inf rejection and proptest over `f64` bit patterns | C1 | **P1.core.float** — owned by this plan's G1 |
| C4 | Content hashing (BLAKE3 + DAG-CBOR + CIDv1) | present | Migrate from hand-rolled envelope to the `cid` + `multihash` + `multibase` crates once upstream PRs land; keep public `Cid` API unchanged. **CI must prove byte-equivalence, not assert it**: all cross-target checks (T4/T6/T9) phrase their assertions as `Node::cid()?.to_string() == <fixture>` so the migration is guarded by the existing CI matrix without rewrite. | — | Tracked in SPIKE "Next Actions" #1 and #3; migration is byte-compat, Phase 1 G1 if upstream releases, else defer to Phase 2 and keep our fork-pin |
| C5 | `proptest` harness: `prop_node_roundtrip_cid_stable` (100k instances) | absent | Ship in R3 test landscape | C1, C3 | **P1.core.proptest** |
| C6 | Version chain primitives (Anchor Node + `CURRENT` / `NEXT_VERSION` edge labels + `walk_versions`, `current_version`, `append_version` helpers) | absent | Opt-in convention. `Node::anchor_id` already reserved; promote to first-class with versioning helpers | C1, C2 | **P1.core.version-chain** — ENGINE-SPEC §6; ephemeral data must not pay versioning cost |
| C7 | Error-catalog mapping (`CoreError` variants → stable codes) | absent | Every `CoreError` variant carries an `ErrorCode` enum discriminant matching `docs/ERROR-CATALOG.md`; serialized in napi layer as `{code, message, context, fixHint}` | C1 | Triage for "every throw maps to catalog code" in Section 5 |

### 2.2 `benten-graph`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| G1 | `KVBackend` trait (get / put / delete / scan / put_batch) | present | Reshape per triage: **(a)** `type Error: std::error::Error + Send + Sync + 'static` associated type (per P1.graph.error-polymorphism); **(b)** `scan` returns `Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), Self::Error>>>` not Vec (P1.graph.scan-iterator) | C1 | **P1.graph.error-polymorphism**, **P1.graph.scan-iterator** |
| G2 | `RedbBackend` implementation | present | Split `open_existing` from `open_or_create`; default to former (P1.graph.open-vs-create); add `DurabilityMode` (Immediate / Group / Async) per SPIKE Next Actions #4 | G1 | **P1.graph.open-vs-create**, **P1.graph.durability** |
| G3 | Transaction primitive API (first-class) | absent | Closure-based API: `backend.transaction(|tx| { tx.put_node(&n); tx.put_edge(&e); Ok(()) })`. Supports heterogeneous ops (node + edge + index + change-event). Single redb write transaction under the hood. | G1, G2 | **P1.graph.transaction-primitive** (spec risk — see Section 5 #3) |
| G4 | `NodeStore` / `EdgeStore` traits with blanket impls over `KVBackend` | absent | Lift `put_node` / `get_node` off `RedbBackend`; add `put_edge` / `get_edge` / `edges_from` / `edges_to`; any `KVBackend` impl gets them for free | G1, C2 | **P1.graph.node-store-trait** |
| G5 | Indexes: label index (hash), property-value index (B-tree over sorted property pairs) | absent | Two redb tables: `label_index` (label bytes → set of CIDs) and `prop_index` ((label, prop_name, prop_value) → set of CIDs). Maintained on every put/delete via the transaction primitive. Supports `get_by_label`, `get_by_property` at O(log n). | G3, G4 | Performance target: <50µs index lookup hot-cache |
| G6 | MVCC via redb snapshot isolation | present (implicit via redb) | Document the read-transaction model; expose `engine.snapshot()` that returns an immutable reader pinned to one redb read-transaction | G2 | No new code; documentation + thin API |
| G7 | Change notification stream | absent | `ChangeEvent { cid, label, kind: Created|Updated|Deleted, tx_id }`. Emitted by the transaction primitive at commit. Subscribers register via `backend.subscribe() -> Receiver<ChangeEvent>` (tokio broadcast channel or equivalent). IVM consumes this. | G3 | See Section 5 risk #2 — channel kind is open |
| G8 | Stress tests: multi-MB round-trip, concurrent reader+writer, failure-injection atomicity | absent | Ship in R3 test landscape | G1–G3 | **P1.graph.stress-tests** |
| G9 | Doctests on `KVBackend` trait methods + `RedbBackend` | absent | Ship `# Examples` blocks on every public method | G1, G2 | **P1.graph.doctests** |

### 2.3 `benten-ivm`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| I1 | `ViewDefinition` type (view id, input pattern, output view-node label, update strategy tag) | stub | Ship. Views stored as Nodes with `system:IVMView` label; view definitions themselves content-addressed for determinism | C1, C2, G4 | System-zone label enforcement is Phase 2 (invariant 11), so for Phase 1 the `system:` prefix is a convention enforced by the IVM crate, not the eval crate |
| I2 | Change-stream subscriber | stub | Subscribes to `benten-graph` change stream (G7). Routes each `ChangeEvent` to every view whose input pattern matches `(label, property)` on the changed CID. | G7, I1 | Routing is O(views × patterns); acceptable for Phase 1 (≤50 views) |
| I3 | **View 1: Capability grants per entity** (from IVM benchmark) | stub | Hand-written incremental maintainer. On `GRANTED_TO` edge creation/deletion, update the entity → {grant_cids} map. | I1, I2 | Pattern directly from `docs/research/ivm-benchmark/RESULTS.md` |
| I4 | **View 2: Event handler dispatch table** | stub | On `SubscribesTo` edge, maintain `event_name → {handler_cids}` table | I1, I2 | Dispatch is O(1) read |
| I5 | **View 3: Content listing (paginated)** | stub | On `post`-labeled node create/update/delete, maintain sorted-by-`createdAt` list; paginated reads are O(log n + page_size) | I1, I2 | **This is the view `crud('post').list` uses** — exit-criterion load-bearing |
| I6 | **View 4: Governance inheritance** | stub | On `GovernedBy` edges, maintain effective-rules transitive closure. Depth cap = 5 hops per ENGINE-SPEC §8. | I1, I2 | Deep nesting test case |
| I7 | **View 5: Version-chain CURRENT pointer resolution** | stub | On version-chain `NEXT_VERSION` append, maintain anchor → current-version CID map. O(1) anchor → current resolution. | I1, I2, C6 | Depends on C6 landing first |
| I8 | Per-view CPU/memory budget + "stale" fallback (from ENGINE-SPEC §8) | absent | Phase 1 minimum: each view has a hardcoded max-work-per-update budget; on exceed, mark view stale and emit `E_IVM_VIEW_STALE`. Async recompute is Phase 2. | I3–I7 | Exit criterion requires the error surfaces correctly, not async recompute |
| I9 | Generalized Algorithm B | **Phase 2** | Do NOT ship in Phase 1 (per CLAUDE.md §14 reconciliation). | — | Flag in risks if scope creeps |

### 2.4 `benten-caps`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| P1 | `CapabilityPolicy` pre-write hook trait | stub | `fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError>` called by `benten-graph` transaction primitive before commit | G3 | TOCTOU check is Phase 2 (invariant 13); Phase 1 checks at commit per ENGINE-SPEC §9 |
| P2 | `CapabilityGrant` Node type + `GRANTED_TO` / `REVOKED_AT` edges | stub | Plain Nodes in `benten-core`; the crate provides typed constructors | C1, C2 | — |
| P3 | `NoAuthBackend` | stub | Ship. Default backend; `check_write` returns `Ok(())` unconditionally. Zero cost. | P1 | Must be the out-of-the-box path — Phase 1 DX requires it |
| P4 | `UCANBackend` stub | stub | Type defined + `check_write` returns `Err(CapError::NotImplemented)`. Full impl is Phase 3 `benten-id`. | P1 | Present so the trait shape is exercised against a second backend; prevents single-impl trait atrophy |
| P5 | Capability denial error wired to `E_CAP_DENIED` | absent | Every denied write emits the error-catalog code; the evaluator routes it to the subgraph's `ON_DENIED` edge | P1, C7, E3 (below) | Exit criterion load-bearing |
| P6 | `requires` property recognition | absent | When a handler Node has a `requires: "store:post:write"` property, the evaluator consults the capability policy before executing. This is the "GATE as property, not primitive" rule from CLAUDE.md decision #1. | P1, E3 | Replaces the dropped GATE primitive |

### 2.5 `benten-eval`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| E1 | Twelve primitive **types** defined (as Rust enum / structs) | stub | All twelve: READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM. WAIT/STREAM/SUBSCRIBE/SANDBOX are valid types with a defined determinism classification + typed-error-edge set so structural validation (E5) accepts them at registration time. Their executor paths return `E_PRIMITIVE_NOT_IMPLEMENTED` at call time. This prevents the regression-class where enabling Phase 2 executors requires re-registering every stored subgraph. | C1 | Error code `E_PRIMITIVE_NOT_IMPLEMENTED` to be reserved in ERROR-CATALOG pre-R1 |
| E2 | Iterative evaluator with explicit execution stack | stub | Ship per ENGINE-SPEC §5. Serializable execution state is Phase 2 (supports WAIT). | E1 | CALL-depth tracking is the non-trivial piece — see Section 5 risk #1 |
| E3 | **Eight executable primitives**: READ, WRITE, TRANSFORM, RESPOND, BRANCH, ITERATE, CALL, EMIT | stub | Each with complete semantics, typed error edges (ON_NOT_FOUND, ON_EMPTY, ON_CONFLICT, ON_DENIED, ON_ERROR, ON_LIMIT), and context binding (`$input`, `$result`, `$item`, `$index`, `$results`, `$error`) | E1, E2, G3, G4, P1, P6 | Core of Phase 1 |
| E4 | TRANSFORM expression evaluator | absent | Subset of JavaScript per ENGINE-SPEC §15 Open Question 1: arithmetic, comparison, logical, ternary, property access, array methods (map/filter/reduce/find/length/slice), object construction, string built-ins (lowercase/uppercase/truncate/substring/startsWith/endsWith), date built-ins (`now()`, `formatDate`), numeric built-ins (Math.min/max/round/abs). No closures, no `this`, no I/O. | E3 | See Section 5 risk #4 — parser choice (`jsep` port? `oxc_parser`? hand-rolled Pratt?) |
| E5 | Structural invariants 1, 2, 3, 5, 6, 9, 10, 12 enforced at registration | absent | DAG-check, depth-check, fan-out-check, node-count, edge-count, determinism classification, content-hash, registration-time validation. **All 12 primitive types pass structural validation** (WAIT/STREAM/SUBSCRIBE/SANDBOX included — they have defined determinism classes and typed error edges). Executor rejection for the 4 Phase-2 primitives happens at **call time** (`E_PRIMITIVE_NOT_IMPLEMENTED`), not registration. All invariant failures map to `E_INV_*` codes. | E1, C1 | Invariants 4, 7, 8, 11, 13, 14 are Phase 2 |
| E6 | Transaction primitive (begin/commit/rollback) exposed as first-class API on `Engine` | absent | Thin wrapper over G3. All WRITEs inside a single subgraph evaluation are atomic. Capability check at commit (ENGINE-SPEC §9). | E3, G3, P1 | See Section 5 risk #3 for the closure-vs-builder decision |
| E7 | `subgraph.toMermaid()` — serialize any operation subgraph as a Mermaid flowchart | absent | Public API on `Subgraph` (or equivalent). Pure function over the subgraph structure, no eval required. Lives behind a `diag` cargo feature, **default OFF in `benten-eval`** to preserve the thin-engine test. `benten-engine` enables the feature in its own default feature set so the DX surface still sees it. | E1 | See Section 5 risk #5; CI gate (future) fires if `src/diag/**` grows past 500 LOC |
| E8 | `engine.trace(handler, input)` — step-by-step evaluation trace with per-node timing | absent | Produces `Vec<TraceStep { node_cid, inputs, outputs, duration_us, error }>`. Stored optionally; off by default. Lives behind the same `diag` feature flag as E7 (default off in `benten-eval`, default on via `benten-engine`). | E2 | Tracing lives behind the workspace `tracing` feature flag per ENGINE-SPEC §14.5 |
| E9 | Phase 2 primitives are type-defined but return "not implemented" | — | WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX: evaluator matches them and returns a Phase 2 error code | E1 | DO NOT actually implement — flag if tempted |

### 2.6 `benten-engine`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| N1 | `Engine::open`, `Engine::create_node`, `Engine::get_node` | present | Keep | G2, C1 | Spike-shipped |
| N2 | `Engine::create_edge`, `Engine::get_edge`, `Engine::edges_from`, `Engine::edges_to`, `Engine::delete_node`, `Engine::update_node` | absent | Full CRUD surface | C2, G4 | — |
| N3 | Public API: `engine.register_subgraph(sg)`, `engine.call(handler_id, input)`, `engine.trace(handler_id, input)` | absent | The call-into-evaluator surface. Registration runs invariant checks, stores the subgraph content-addressed. | E1–E8, N1 | Load-bearing for TS DSL |
| N4 | IVM wiring: `engine` constructs an IVM subscriber and attaches it to the change stream | absent | Default-on. `Engine::builder().without_ivm()` opt-out for the thinness test. | G7, I2 | Thinness test: `Engine::builder().without_ivm().without_caps()` must still produce a usable graph DB |
| N5 | Capability-policy wiring: `Engine::builder().capability_policy(Box::new(NoAuthBackend))` | absent | `NoAuthBackend` is the builder default. | P1, P3 | — |
| N6 | `Engine::snapshot()` + `Engine::transaction(|tx| …)` public surface | absent | Delegates to G6 and G3 | G3, G6 | — |
| N7 | `Engine::grant_capability`, `Engine::create_view`, `Engine::revoke_capability` (engine-API-only paths for system-zone Nodes) | absent | Required by ENGINE-SPEC §9 system-zone protection. Phase 1 doesn't enforce invariant 11 at the evaluator yet, but engine API should be the only way to mutate capability/view Nodes anyway. | C1, P2, I1 | — |

### 2.7 `bindings/napi`

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| B1 | `initEngine`, `createNode`, `getNode` | present | Keep | N1 | Spike-shipped |
| B2 | Node.js-side smoke test (Vitest) of TS → Rust → redb → Rust → TS round-trip | absent | `bindings/napi/package.json` + `bindings/napi/index.test.ts` asserting the TS-returned CID matches Rust-side fixture `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` | B1 | SPIKE Next Actions #2 — currently build-checked only |
| B3 | Full CRUD binding surface: `createNode`, `getNode`, `updateNode`, `deleteNode`, `createEdge`, `getEdge`, `edgesFrom`, `edgesTo`, `deleteEdge` | absent | napi-rs #[napi] annotations; TS types auto-generated | B1, N2 | — |
| B4 | Subgraph registration + evaluation bindings: `registerSubgraph(serialized)`, `callHandler(id, input)`, `traceHandler(id, input)` | absent | Subgraph passed as DAG-CBOR bytes or structured JS object (TBD — see Section 5 risk #6) | N3 | — |
| B5 | IVM view read binding: `readView(view_id, query)` | absent | — | I3–I7, N3 | — |
| B6 | `@benten/engine` TypeScript wrapper over `@benten/engine-native` | absent | Wrapper provides the DSL (`subgraph`, `crud`, `read`, `write`, `transform`, `branch`, `iterate`, `call`, `respond`, `emit`), good TS types, zero-config `crud('post')` path, error-catalog typed errors, `handler.toMermaid()` / `engine.trace()` ergonomics. **Zero-config `crud('post')` must inject `createdAt: <deterministic HLC timestamp>` on WRITE if the input does not include one** — otherwise View 3 (content listing, sorted by `createdAt`) has nothing to sort by and the developer sees a cryptic "view read empty" error. Timestamp is deterministic (HLC stamped once at create, never re-computed) so re-writes don't silently reshuffle the list. Test: `crud_post_zero_config_injects_createdat_deterministically`. | B3–B5, E7, E8 | The DX-facing surface. Landing this correctly *is* the exit criterion. |
| B7 | Error-catalog integration: typed error classes (`ECapDenied`, `EInvCycle`, …) | absent | Generated from `docs/ERROR-CATALOG.md` via a small script, consumed by `@benten/engine/errors` | C7 | — |

### 2.8 Developer tooling (separate from binary crates)

| # | Deliverable | Status at spike end | Phase 1 deliverable | Dependencies | Notes / triage tags |
|---|---|---|---|---|---|
| T1 | `create-benten-app` scaffolder | absent | `npx create-benten-app my-app` produces a minimal TS project with `@benten/engine` installed, a sample handler file, and `npm run dev` / `npm run test` wired | B6 | See Section 5 risk #5 — TS-authored vs Rust-authored; lives in `tools/create-benten-app/` |
| T2 | `subgraph.toMermaid()` surfaced through DSL | absent | `handler.toMermaid()` returns a string | E7, B6 | — |
| T3 | `engine.trace(handler, input)` surfaced through DSL | absent | Returns `{steps: TraceStep[], result}` | E8, B6 | — |
| T4 | CI: native matrix (macOS arm64, macOS x86_64, Linux x86_64, Linux arm64) reproducing the canonical CID fixture | absent | **P1.ci.multi-arch** | — | Cross-arch determinism guard |
| T5 | CI: MSRV (1.85) and latest stable, both reproducing fixture | absent | **P1.ci.msrv** | — | — |
| T6 | CI: `print_canonical_cid` under `wasm32-wasip1` via wasmtime, assert CID matches | absent | **P1.ci.wasm-runtime**. Phrased as `assert Node::cid()?.to_string() == <fixture>` so the test survives the C4 `cid`-crate migration without edit. | — | Runtime WASM determinism — currently compile-check only |
| T7 | ERROR-CATALOG.md → TypeScript codegen + bidirectional drift detector | absent | **Decided pre-R1 (was ambiguous):** Rust enum is the source of truth, hand-authored in G1-A (C7). T7 reads `docs/ERROR-CATALOG.md` + `crates/benten-core/src/error_code.rs` and (a) generates `packages/engine/src/errors.ts` from the catalog, and (b) runs a drift detector in CI that fails if any of the three (catalog doc, Rust enum, TS types) diverge. Avoids the "duplicate enum" and "replace existing enum late in Phase 1" traps the critic flagged. | C7, B7 | — |
| T8 | CI: `cargo check --target wasm32-unknown-unknown -p benten-napi` | absent | Compile-check the full `bindings/napi` crate for wasm32 in CI. Proves the napi-rs v3 dual-target promise holds at the binding-crate level, not just for `benten-core`. Runtime WASM with network-fetch `KVBackend` remains Phase 2 per Rank 10. | B3–B7 | New deliverable added pre-R1 per Section 5 Rank 4.5. Low-cost Phase 1 commitment that prevents a large Phase 2 retrofit. |
| T9 | Cross-leg determinism gate | absent | CI job that depends on T4 + T5 + T6 all completing, collects the CID each produced, asserts byte-for-byte equality across all legs. Not "each matches the fixture" (which can hide drift) but "every leg produces the same bytes as every other leg." | T4, T5, T6 | New deliverable added pre-R1 per benten-core-guardian finding. Makes cross-target agreement explicit rather than transitive-through-the-fixture. |

---

## 3. Implementation Groups (R5 partition)

Eight groups, ordered so each consumes the outputs of earlier groups. Agents within a group own disjoint file sets (no write conflicts). Each group is 1–3 days of reviewed-human-equivalent work.

### G1 — Core types hardening + version chains

**Agents:** 2 × `rust-implementation-developer`
**Parallelism:** yes (different submodules of `benten-core`)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G1-A | `crates/benten-core/src/value.rs` (new), `crates/benten-core/src/error_code.rs` (new), updates to `src/lib.rs` (Value::Float, CoreError codes) | `tests/value_float_*`, `tests/error_code_roundtrip`, proptest `prop_node_roundtrip_cid_stable` |
| G1-B | `crates/benten-core/src/edge.rs` (new), `crates/benten-core/src/version.rs` (new — Anchor, `CURRENT` / `NEXT_VERSION` labels, `walk_versions`, `append_version`) | `tests/edge_roundtrip_*`, `tests/version_chain_*`, `tests/version_linearization`, `tests/edge_creation_does_not_change_endpoint_node_cids`, `tests/version_chain_linking_does_not_change_version_node_cids` (both protect the §7 "edges excluded from Node hash" invariant against the new C2 + C6 surfaces) |

**Gates next group:** G2 (needs `Edge` type), G3 (needs version chains), R3 test infrastructure for proptest.
**Deliverables closed:** C2, C3, C5, C6, C7.

### G2 — Graph storage reshape (KVBackend, RedbBackend, NodeStore/EdgeStore)

**Agents:** 2 × `rust-implementation-developer`
**Parallelism:** yes (trait surface vs backend impl)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G2-A | `crates/benten-graph/src/backend.rs` (KVBackend trait + associated Error type, Scan iterator type), `crates/benten-graph/src/store.rs` (NodeStore, EdgeStore blanket impls) | `tests/backend_error_polymorphism_*`, `tests/scan_iterator_*`, `tests/node_store_blanket`, `tests/edge_store_*` |
| G2-B | `crates/benten-graph/src/redb_backend.rs` (split open_existing / open_or_create, DurabilityMode), `crates/benten-graph/src/indexes.rs` (label_index, prop_index — populate hooks) | `tests/durability_mode_*`, `tests/redb_open_existing_vs_create`, `tests/label_index_*`, `tests/prop_index_*`, stress tests (multi-MB, concurrent reader+writer) |

**Gates next group:** G3 (transaction primitive needs `NodeStore`/`EdgeStore`), G5 (IVM change events need transaction commits).
**Deliverables closed:** G1, G2, G4, G5, G8, G9, plus triage tags P1.graph.error-polymorphism, P1.graph.scan-iterator, P1.graph.open-vs-create, P1.graph.node-store-trait, P1.graph.doctests, P1.graph.stress-tests.

### G3 — Transaction primitive + change notification stream

**Agents:** 1 × `rust-implementation-developer` + 1 × `performance-engineer` (from SPIKE Next Actions #4 pairing)
**Parallelism:** lightly — perf agent owns the durability mode bench, implementer owns the txn API

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G3-A | `crates/benten-graph/src/transaction.rs` (closure-based `backend.transaction(|tx| …)`, commits in one redb write-txn, emits ChangeEvents at commit), `crates/benten-graph/src/change.rs` (ChangeEvent type + broadcast channel) | `tests/transaction_atomicity_*`, `tests/change_event_*`, `tests/failure_injection_rollback` |
| G3-B | `crates/benten-graph/benches/durability_modes.rs` + docs on the durability-policy matrix | criterion benches produce `Immediate` / `Group` / `Async` numbers; pass if `Group` hits <500µs per write |

**Gates next group:** G5 (IVM subscribes to change stream), G6 (evaluator transactions wrap subgraph writes). Note: G4 runs in parallel with G3 — it only imports the `CapabilityPolicy` trait shape (defined in G4 itself) into G3's transaction module, not G3's code. G4 does not depend on G3.
**Deliverables closed:** G3, G7.

### G4 — Capability policy crate

**Agents:** 1 × `rust-implementation-developer` + 1 × `ucan-capability-auditor` (mini-review only, after commit)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G4-A | `crates/benten-caps/src/policy.rs` (CapabilityPolicy trait), `src/grant.rs` (CapabilityGrant type + edges), `src/noauth.rs` (NoAuthBackend), `src/ucan_stub.rs` (UCANBackend that errors), `src/error.rs` (CapError → E_CAP_DENIED / E_CAP_REVOKED / E_CAP_ATTENUATION) | `tests/noauth_permits_everything`, `tests/ucan_stub_errors_cleanly`, `tests/check_write_called_at_commit`, `tests/cap_error_codes_match_catalog` |

**Gates next group:** G6 (evaluator consults the hook for `requires` property).
**Deliverables closed:** P1, P2, P3, P4.

### G5 — IVM subscriber + five views

**View numbering legend** (avoids I-row ↔ View-number confusion): View 1 = I3 (capability grants), View 2 = I4 (event-handler dispatch), View 3 = I5 (content listing), View 4 = I6 (governance inheritance), View 5 = I7 (version-current).

**Agents:** 3 × `rust-implementation-developer` (views split between them) + 1 × `ivm-algorithm-b-reviewer` (mini-review only, after commit)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G5-A | `crates/benten-ivm/src/subscriber.rs` (change-stream loop, pattern matcher, routing), `src/view.rs` (ViewDefinition, budget tracking, stale state) | `tests/subscriber_routes_to_matching_views`, `tests/stale_marked_when_budget_exceeded` |
| G5-B | `crates/benten-ivm/src/views/capability_grants.rs` (View 1), `src/views/event_handler_dispatch.rs` (View 2), `src/views/governance_inheritance.rs` (View 4) | Per-view incremental-maintenance tests from R3 (each view has a write → read → assert cycle + a rebuild-from-scratch equivalence check) |
| G5-C | `crates/benten-ivm/src/views/content_listing.rs` (View 3 — exit criterion!), `src/views/version_current.rs` (View 5) | `tests/content_listing_paginated_*`, `tests/content_listing_incremental_update`, `tests/version_current_o1_resolution` |

**Gates next group:** G6 (eval uses View 3 for `crud('post').list`), G7 (engine wires subscriber).
**Deliverables closed:** I1, I2, I3, I4, I5, I6, I7, I8.

### G6 — Evaluator + eight primitives + invariants + TRANSFORM expressions

**Agents:** 3 × `rust-implementation-developer` + 1 × `operation-primitive-linter` (mini-review after commit) + 1 × `code-as-graph-reviewer` (mini-review after commit)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G6-A | `crates/benten-eval/src/primitives/mod.rs` (all 12 type defs), `src/primitives/{read,write,respond,emit}.rs` (the four simplest), `src/context.rs` (evaluation context: $input, $result, $item, $index, $results, $error bindings) | `tests/read_by_id_*`, `tests/read_by_query_*`, `tests/write_create_update_delete`, `tests/respond_terminal`, `tests/emit_fire_and_forget`, `tests/context_scoping` |
| G6-B | `crates/benten-eval/src/primitives/{branch,iterate,call,transform}.rs`, `src/expr/parser.rs` + `src/expr/eval.rs` (TRANSFORM expression language) | `tests/branch_multi_way`, `tests/iterate_max_required`, `tests/iterate_parallel`, `tests/call_attenuation`, `tests/call_timeout`, `tests/transform_expression_coverage` (full built-in matrix), `tests/transform_no_closures` |
| G6-C | `crates/benten-eval/src/evaluator.rs` (iterative stack evaluator, CALL-depth tracking, transaction wrap), `src/invariants.rs` (1/2/3/5/6/9/10/12 enforced at registration, each mapped to E_INV_* code), `src/diag/mermaid.rs` (toMermaid), `src/diag/trace.rs` (engine.trace) | `tests/evaluator_stack_*`, `tests/evaluator_no_recursion`, `tests/invariant_*` (one per enforced invariant, each producing correct E_INV_* code), `tests/mermaid_render_*`, `tests/trace_per_node_timing` |

**Gates next group:** G7 (engine API layer), G8 (bindings).
**Deliverables closed:** E1 (types), E2, E3, E4, E5, E7, E8, E9, P5, P6.

### G7 — Engine orchestrator + transaction primitive surface

**Agents:** 1 × `rust-implementation-developer` + 1 × `benten-engine-philosophy` reviewer (mini-review after commit; see Section 6 for to-create flag)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G7-A | `crates/benten-engine/src/builder.rs` (EngineBuilder with `without_ivm()` / `without_caps()`), `src/engine.rs` (register_subgraph, call, trace, transaction, snapshot, system-zone API), `src/wiring.rs` (IVM subscriber attachment, cap-policy injection) | `tests/thinness_no_ivm_no_caps_still_works` (the thinness test!), `tests/register_subgraph_runs_invariants`, `tests/call_handler_end_to_end`, `tests/trace_returns_steps`, `tests/grant_capability_only_via_engine_api` |

**Gates next group:** G8.
**Deliverables closed:** N1 (keep), N2, N3, N4, N5, N6, N7, E6.

### G8 — napi bindings + `@benten/engine` wrapper + scaffolder + DX tooling + CI matrix

**Agents:** 2 × `rust-implementation-developer` (for napi) + 1 × `ai-engineer` or `dx-optimizer` (for TS wrapper) + 1 × `napi-bindings-reviewer` (mini-review after commit)

| Agent | Files owned | Must-pass tests (from R3) |
|---|---|---|
| G8-A | `bindings/napi/src/{node,edge,subgraph,view,trace,error}.rs` (full binding surface), `bindings/napi/index.test.ts` (Vitest smoke + CRUD) | `tests/ts_roundtrip_cid_matches_rust_fixture`, `tests/ts_crud_full_cycle`, `tests/ts_subgraph_register_and_call`, `tests/ts_trace_contains_per_node_timings` |
| G8-B | `packages/engine/src/**` (`@benten/engine` TS wrapper: DSL, crud shorthand, zero-config path, typed errors, `toMermaid`/`trace` ergonomics, error-catalog codegen input) | Full DSL test pack from R3 (crud('post') zero-config, crud with options, all 12 DSL functions, error-catalog TS types match runtime codes) |
| G8-C | `tools/create-benten-app/**` (scaffolder — TS authored, ships as its own npm package), `.github/workflows/ci.yml` (multi-arch matrix + MSRV + wasm-runtime jobs), `scripts/codegen-errors.ts` (ERROR-CATALOG.md → TS + Rust enum codegen) | `tests/scaffolder_produces_working_project`, CI green on all matrix legs, drift-detector CI job passes |

**Gates next group:** R4b post-implementation test review, then R6 quality council.
**Deliverables closed:** B1 (keep), B2–B7, T1–T7.

---

## 4. Test Landscape (R2 input)

Seed content for the `benten-test-landscape-analyst` to expand. Organized by test type, not by group, because the test landscape agent produces one unified plan.

### 4.1 Unit tests — mandatory coverage

Every public function/method in every crate gets at least one unit test per invariant it claims. Notable must-haves:

- `benten-core`: Node hash determinism (4 cases from spike + float cases from G1-A); Edge hash determinism; **`edge_creation_does_not_change_endpoint_node_cids`** (creating an edge between two Nodes does not shift either endpoint's CID — protects the ENGINE-SPEC §7 invariant "edges excluded from Node hash"); **`version_chain_linking_does_not_change_version_node_cids`** (adding a `NEXT_VERSION` edge to a Version Node does not shift its CID — protects the same invariant against C6's version-chain work); Value conversion round-trips for every variant including Float; Cid parse/round-trip including malformed inputs; Version-chain `walk_versions` linearization + divergence.
- `benten-graph`: `KVBackend` conformance suite runnable against any implementation (so the future peer-fetch backend can reuse it); RedbBackend open-existing-vs-create; DurabilityMode matrix (Immediate/Group/Async commit-visibility ordering); index maintenance invariants; transaction atomicity under simulated failure; **multi-MB round-trip** (1MB / 10MB / 100MB blobs round-trip through `put` → `get` with BLAKE3 integrity intact — exercises redb page-boundary handling); **concurrent reader + writer** (N reader threads + 1 writer thread against the same backend; no torn reads; redb MVCC snapshot semantics hold; writers never deadlock readers).
- `benten-ivm`: each of the five views has (a) a "build from scratch matches incremental" equivalence test, (b) a stale-on-budget-exceeded test, (c) a write-read latency bound test.
- `benten-caps`: NoAuthBackend permits every op; UCAN stub cleanly errors with the right code; `requires` property triggers the hook with the correct context.
- `benten-eval`: one test per executable primitive covering the happy path + every typed error edge; invariants 1/2/3/5/6/9/10/12 each have a registration-time rejection test and a positive-case test; TRANSFORM expression language has a coverage test per built-in function and a rejection test per forbidden construct (closures, `this`, imports, prototype access); **`phase_two_primitives_pass_structural_validation`** — a subgraph containing WAIT / STREAM / SUBSCRIBE / SANDBOX nodes **passes** `register_subgraph` (types are known, determinism classification exists, DAG-ness holds, fan-out etc. all check out). Only at **call time** does the executor return `E_PRIMITIVE_NOT_IMPLEMENTED`. Prevents the regression-class where enabling Phase-2 executors requires re-registering existing stored subgraphs.
- `benten-engine`: thinness test (engine without IVM or caps is still a functional content-addressed graph DB); system-zone API exclusivity; transaction-primitive rollback on any WRITE failure.
- `bindings/napi`: every exposed function has a TS → Rust → TS round-trip test.

### 4.2 Property-based (proptest)

- `prop_node_roundtrip_cid_stable` — 100k+ instances over `Node` values, asserting hash → decode → re-hash is stable and the decoded node equals the original.
- `prop_edge_roundtrip_cid_stable` — same for Edge.
- `prop_value_json_cbor_conversion` — converting between JSON (from napi), Value, and DAG-CBOR round-trips without loss (except Floats-that-are-integers, which DAG-CBOR rejects per spec).
- `prop_version_chain_linearizable` — given a random sequence of `append_version` calls, `walk_versions` produces a total order compatible with the NEXT_VERSION DAG.
- `prop_kvbackend_put_get_delete` — against NoOp / in-memory / redb backends, asserting the KVBackend laws.
- `prop_content_listing_incremental_equivalence` — after N random writes, the incremental View 3 state equals a rebuild-from-scratch.
- `prop_transform_expression_deterministic` — TRANSFORM outputs only depend on inputs (no wall-clock or RNG leakage).

### 4.3 Integration tests (cross-crate)

- **Capability-gated CRUD round-trip**: register `crud('post')`, grant a capability, `engine.call('post:create', …)` succeeds; revoke, same call returns `E_CAP_DENIED` routed via `ON_DENIED`.
- **IVM write-propagation**: write ten posts, list via View 3, assert pagination correctness and that incremental maintenance never trailed by more than one write.
- **Transaction atomicity end-to-end**: subgraph with two WRITEs; inject failure in second; assert first rolled back via redb.
- **Version-chain CURRENT resolution**: append five versions to an anchor, assert View 5 CURRENT pointer is O(1) at every step.
- **Content-hash integrity through the stack**: `createNode` in TS → redb persistence → `getNode` in TS → re-hash in Rust → assert equal to original CID.

### 4.4 Criterion benchmarks

Targets annotated with their source: **(§14.6 direct)** = literal from ENGINE-SPEC §14.6 table; **(§14.6 derived)** = inference/decomposition from a §14.6 range; **(non-§14.6)** = informational, not a gate.

- `hash_only` — baseline, no regression from spike 892ns. **(non-§14.6 — baseline protection)**
- `get_node` — target 1–50µs hot cache (spike: 2.71µs). **(§14.6 direct — "Node lookup by ID")**
- `create_node_immediate` — target 100–500µs realistic (spike Immediate: 4ms — must drop with DurabilityMode::Group). **(§14.6 direct — "Node creation + IVM update")**
- `create_node_group_commit` — new benchmark; target <500µs. **(§14.6 derived — same range as Immediate but assumes group-commit amortization so the sub-range midpoint is the pass bar)**
- `10_node_handler_eval` — three sub-benches cover the §14.6 target:
  - `crud_post_create_dispatch`: end-to-end dispatch including the redb fsync commit. Phase-1 floor on macOS APFS is ~4ms per call (compromise #7: Group-durability collapses to Immediate). **Not CI-gated** — the §14.6 "150–300µs" headline target is not reachable in Phase 1 on dev hardware while the durability mode is Immediate. Phase-2 re-gates this after grouped/async durability lands.
  - `crud_post_list_dispatch_no_write`: isolates evaluator + dispatch + outcome-mapper overhead from fsync. Target: median < 300µs. **(§14.6 derived — evaluator-only slice of the "10-node handler" number)**
  - `crud_post_build_subgraph_only`: regression signal for the `SubgraphCache` (r6-perf-5) — measures the cache-hit path (template clone + per-call property patch + evaluator walk for a GET that resolves to None). Target: median < 10µs. **(non-§14.6 — cache hygiene)**
- `view_read_content_listing` — target <1µs for View 3 hot-cache read. **(§14.6 direct — "IVM view read (clean)" 0.04–1µs)**
- `view_incremental_maintenance` — target <50µs per write for any of the five views. **(§14.6 derived — decomposition of the 100–500µs "Node creation + IVM update" envelope: reserve ~50µs for the IVM slice and the balance for storage + hash)**
- `concurrent_writers` — benchmark the 100–1000 writes/sec single-community ceiling. **(§14.6 direct — "Concurrent writers")** — surface in CI as **informational, not a gate**.

### 4.5 TypeScript ↔ Rust end-to-end (closes SPIKE punt #2)

- Vitest file `bindings/napi/index.test.ts`: `initEngine` → `createNode` with the canonical fixture properties → assert returned CID string equals `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` → `getNode(cid)` returns byte-identical node shape.
- Vitest file `packages/engine/src/crud.test.ts`: register `crud('post')` with zero-config, create / read / list / update / delete, assert every code path.
- `packages/engine/src/trace.test.ts`: register a handler, call `engine.trace(handlerId, input)`, assert every TraceStep has non-zero `durationUs` and the step sequence matches the subgraph's topological order.
- Scaffolder test: `npx create-benten-app test-app && cd test-app && npm install && npm run test` in CI passes.

### 4.6 Cross-cutting / special harnesses

- **Cross-process determinism (D2 extension)**: SPIKE shipped intra-process + cross-process for `benten-core`. Extend to cover `benten-graph` (persisted CID round-trip across processes), and a transactional round-trip (write in process A, read in process B, assert CID).
- **Multi-arch CI matrix (P1.ci.multi-arch)**: GitHub Actions matrix `{aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu}`, each reproducing `canonical_cid.txt`.
- **MSRV CI (P1.ci.msrv)**: build at 1.85 and latest stable; fixture reproducibility check on both.
- **WASM runtime CI (P1.ci.wasm-runtime)**: compile `print_canonical_cid` to `wasm32-wasip1`, run under wasmtime, assert stdout matches fixture.
- **Error-catalog drift detector**: CI job that parses `docs/ERROR-CATALOG.md` and compares against `benten-core/src/error_code.rs` (hand-authored source of truth) and `packages/engine/src/errors.ts` (codegenned from the catalog); fail on drift.
- **Cross-leg determinism gate (T9)**: a CI job that depends on T4 (multi-arch), T5 (MSRV), and T6 (wasm-runtime) all completing, collects the CID each produced, and asserts they are **byte-for-byte equal** — not just "each matches a fixture string" but "each leg produces the same bytes as every other leg." Failure message identifies the drift leg and the divergent bytes. Makes cross-target agreement explicit rather than transitive-through-the-fixture.

---

## 5. Risk List + Open Questions (ranked)

Questions whose resolution blocks design; risks that could slip scope. Ranked roughly by leverage on the rest of Phase 1.

### Rank 1 — Evaluator architecture: CALL-depth tracking and pausable state

The iterative evaluator in ENGINE-SPEC §5 is validated in principle, but CALL recursion is the non-trivial piece. Phase 1 defers WAIT (which requires serializable execution state), but CALL already needs per-invocation frame isolation for capability attenuation (`isolated: true`). **Open:** is the frame a `Vec<ExecutionFrame>` inside the explicit stack, or a separate call-stack of contexts? The answer affects the evaluator's public shape and Phase 2's WAIT serialization format.

**Recommendation for R1 debate:** `Vec<ExecutionFrame>` where each frame owns its own context binding scope, capability attenuation, and iteration budget. Pausable serialization becomes "serialize the frame vec"; that's what Phase 2's WAIT will do. The explicit stack holds references into frames.

### Rank 2 — Change-notification stream channel type

G7 ships a channel, but whether it's `tokio::sync::broadcast` (bounded, last-value-wins on slow subscriber), `async-channel` (unbounded with back-pressure), a custom watchable inspired by iroh, or a synchronous `Vec<Box<dyn ChangeListener>>` affects:

- whether `benten-graph` depends on tokio (today it doesn't);
- whether slow IVM views can drop events (correctness bug) or back-pressure the writer (latency bug);
- how WASM builds consume the stream.

**Recommendation for R1 debate:** start with a **bounded tokio broadcast** (tokio is already in the napi crate's transitive deps via napi-rs; verify) with a configurable high-water mark that, on overflow, marks affected views as stale. This maps to the "E_IVM_VIEW_STALE" error surface we've already spec'd. For WASM, a synchronous callback-list wrapper provides the same interface without tokio.

### Rank 3 — Transaction-primitive API shape (closure vs WriteBatch builder)

ENGINE-SPEC §14 says "finalized in Phase 2 based on Phase 1 usage feedback," but Phase 1 needs *some* API. The two candidates:

- **Closure**: `engine.transaction(|tx| { tx.create_node(&n)?; tx.create_edge(&e)?; Ok(()) })`. Simpler for callers, harder to compose (can't pass a tx across async boundaries without `Send + 'static` gymnastics).
- **Builder**: `engine.write_batch().create_node(&n).create_edge(&e).commit()`. More awkward, but future WAIT-friendly because the batch is a pure data structure.

**Recommendation for R1 debate:** ship **closure** for Phase 1 — matches CLAUDE.md's "pragmatic defaults" ethos and the DX-critic priorities. Flag explicitly that Phase 2 may reshape based on whether any handler wants to pass a tx across CALL boundaries.

### Rank 4 — TRANSFORM expression parser choice

ENGINE-SPEC §15 Open Question 1 recommends a restricted JavaScript subset "compatible with the existing @benten/expressions evaluator (jsep + custom AST walker)." Options for Rust:

- Port `jsep` to Rust (hand-rolled Pratt parser over a defined grammar).
- Use `oxc_parser` (full JS parser, subset by post-validation).
- Use `logos` + `chumsky` / `winnow` for a custom small-language parser.
- Hand-roll a Pratt parser against a BNF we publish alongside Phase 1.

**Recommendation for R1 debate:** **hand-roll a Pratt parser with a published BNF**. TRANSFORM's surface is small (~25 operators + ~30 built-ins), and we want determinism + zero-dep + explicit rejection of unsupported syntax. `oxc_parser` would invite scope creep. Publish the BNF as `docs/TRANSFORM-GRAMMAR.md` as part of G6's deliverable.

### Rank 4.5 — napi-rs v3 dual-target build integration

SPIKE RESULTS confirmed napi-rs v3 compiles the Node.js binding surface without a separate wasm-bindgen path, and `benten-core` already compiles to `wasm32-unknown-unknown` (D3 compile-check). But the plan currently ships only a standalone `print_canonical_cid` WASM runtime check (T6) — it does not commit to a wasm32 compile-check for the full `bindings/napi` crate in Phase 1. **Open:** does Phase 1 prove the "same codebase" promise for the *bindings* crate, or does that wait for Phase 2 alongside the network-fetch `KVBackend` runtime?

**Recommendation for R1 debate:** commit Phase 1 to a CI step `cargo check --target wasm32-unknown-unknown -p benten-napi` (new deliverable T8 in Section 2.8). This is a cheap compile-check that proves the binding layer's dual-target shape stays intact. Full WASM runtime with network-fetch backend remains Phase 2 per Rank 10. Rationale: retrofitting the dual-target build system in Phase 2 after the binding code accretes native-only patterns is much more expensive than enforcing it now from day one.

### Rank 5 — Debug tooling crate home: `benten-eval` vs separate `benten-diag`

`subgraph.toMermaid()` and `engine.trace()` are diagnostics, not core. Putting them in `benten-eval` inflates the thin-engine crate; putting them in a `benten-diag` crate adds a seventh crate in Phase 1.

**Recommendation for R1 debate:** keep them in `benten-eval` behind a `diag` feature flag (default enabled). Promote to a crate in Phase 2 only if the `diag` surface grows beyond ~500 LOC. Rationale: seventh crate costs more in workspace complexity than the feature-flag pattern costs in build-time optionality.

### Rank 6 — Scaffolder: TS-authored vs Rust-authored

`create-benten-app` can be a Node.js CLI in `tools/create-benten-app/` (ship as an npm package, depends only on Node core) or a Rust binary shipped via `cargo install`. The former matches the developer's mental model (they already have Node); the latter gives us better template-file-handling ergonomics.

**Recommendation for R1 debate:** **TS-authored**, in `tools/create-benten-app/`. Benten's target developer has Node. Avoid making them install Rust just to bootstrap. `npx` works transparently.

### Rank 7 — ERROR-CATALOG integration: codegen now or by-hand mapping?

Every `thiserror` variant in the crates should map to an ERROR-CATALOG code. Two options:

- **Codegen**: Script parses ERROR-CATALOG.md → emits Rust enum + TS type → drift detector in CI.
- **Hand-authored**: Each crate defines its own enum with stable codes; catalog doc updated manually.

**Recommendation for R1 debate:** **codegen** (T7). One place to add a new error, two consumers stay in sync. Drift detector in CI catches manual edits.

### Rank 8 — Subgraph serialization wire format (Rust ↔ TS)

When TS calls `engine.registerSubgraph(sg)`, `sg` crosses the napi boundary. Options: structured JS object (napi-rs handles the conversion), DAG-CBOR bytes (deterministic but awkward from JS), JSON (lossy for `Value::Bytes`), or a content-addressed reference (TS hashes first, passes CID).

**Recommendation for R1 debate:** **structured JS object** for Phase 1 ergonomics; the napi binding validates and canonicalizes on arrival. Internally stored as DAG-CBOR with its CID.

### Rank 9 (lower) — Pre-existing deferrals from SPIKE "Critic Triage"

All **P1.*.*** tags in SPIKE RESULTS "Critic Triage — Deferred to Phase 1" are integrated into Section 2 above. The plan treats them as committed work, not as re-litigable decisions. Listed here so R1 reviewers can verify none slipped:

- P1.core.float → C3 (G1-A)
- P1.core.version-chain → C6 (G1-B)
- P1.core.proptest → C5 (R3)
- P1.ci.wasm-runtime → T6 (G8)
- P1.ci.multi-arch → T4 (G8)
- P1.ci.msrv → T5 (G8)
- P1.graph.error-polymorphism → G1 trait reshape (G2-A)
- P1.graph.scan-iterator → G1 trait reshape (G2-A)
- P1.graph.open-vs-create → G2-B
- P1.graph.transaction-primitive → G3-A
- P1.graph.node-store-trait → G2-A
- P1.graph.doctests → G2 (both agents)
- P1.graph.stress-tests → G2 R3 tests

### Rank 10 — Scope boundaries explicitly flagged

If any R1 reviewer or R5 implementer argues for including any of the following in Phase 1, escalate rather than silently drop:

- WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX primitive **execution** (types are defined in E1; executors are Phase 2)
- Structural invariants 4, 7, 8, 11, 13, 14
- Generalized IVM Algorithm B + per-view strategy selection A/B/C (the hand-written five views are Phase 1; generalization is Phase 2)
- wasmtime host integration for SANDBOX — including **instance pool** and **host-function manifest**
- WASM *runtime* build target (WASM *compile-check* for `benten-core` is Phase 1 per T6; a `bindings/napi` compile-check for `wasm32-unknown-unknown` is also Phase 1 per new T8 below; actual runtime with a network-fetch `KVBackend` implementation is Phase 2)
- **Capability enforcement hardened across all 12 primitives** (Phase 1 enforces on the 8 executed primitives only; tightening across WAIT/STREAM/SUBSCRIBE/SANDBOX is Phase 2 alongside their executors)
- **Paper-prototype re-validation against the revised 2026-04-14 primitive set** (original 12 validated at 2.5% SANDBOX rate; re-measurement is Phase 2)
- **Network-fetch `KVBackend` implementation** (the trait is stabilized in Phase 1; a production implementation that pulls content-addressed bytes from peers via iroh/HTTP is Phase 2)
- Module manifest format (requires-caps, provides-subgraphs, migrations) — Phase 2
- Dev server with hot reload (QUICKSTART marks it Phase 2; Phase 1 ships a `npm test` verification path instead, see Section 1 exit criterion)

---

## 6. ADDL Stage Dispatch Plan

| Stage | Agents | Output | Gate for next stage |
|---|---|---|---|
| **R1 — Spec review** (agent-team mode, peer-debate) | `architect-reviewer`, `code-reviewer`, `security-auditor`, `dx-optimizer`, **`benten-engine-philosophy`** (to-create before R1 begins) | 5 structured JSON findings at `.addl/phase-1/r1-<agent>.json` per the JSON contract in DEVELOPMENT-METHODOLOGY.md § Pattern 3 | Ben triages every finding: fix-plan / defer-doc / disagree-rationale. No "noted." Plan revised. |
| **R2 — Test landscape synthesis** (1 agent, lead session) | **`benten-test-landscape-analyst`** (to-create before R2 begins) | `.addl/phase-1/r2-test-landscape.md` — every public API in Section 2 mapped to test requirements (unit/proptest/integration/bench/end-to-end), expanding Section 4 of this plan | Ben reviews the test landscape for completeness; gates R3. |
| **R3 — Test writing** (5 parallel subagents, TDD contract) | **`rust-test-writer-unit`**, **`rust-test-writer-edge-cases`**, **`rust-test-writer-security`**, **`rust-test-writer-performance`**, `qa-expert` (integration) — all to-create as subagents before R3 begins | Real Rust test files + TS Vitest files in place but **failing** (no implementations yet). Criterion benchmarks with configured targets. Coverage-map JSON at `.addl/phase-1/r3-coverage.json` | Test suite compiles and runs; failures are expected and their count matches the scope-inventory deliverable count. |
| **R4 — Test review (pre-implementation)** (2 parallel subagents) | **`rust-test-reviewer`**, `qa-expert` (or **`rust-test-coverage`**) | Findings on test quality, patterns, coverage gaps. JSON at `.addl/phase-1/r4-<agent>.json` | Fix all findings; re-run tests; gate R5. |
| **R5 — Implementation groups** | 8 groups G1–G8 per Section 3. Each group: N parallel `rust-implementation-developer` agents + `cargo-runner` (runs `cargo fmt/clippy/nextest/doc` after each commit, structured output) + `rust-engineer` (fallback for cross-cutting idiom questions, dispatched ad-hoc when implementers hit a lifetime / trait-coherence / ownership question they can't resolve alone) + 1 mini-review agent (`code-reviewer` or the crate's guardian — `benten-core-guardian`, `ucan-capability-auditor`, `ivm-algorithm-b-reviewer`, `operation-primitive-linter`, `code-as-graph-reviewer`, `napi-bindings-reviewer`, `benten-engine-philosophy` as scope demands) | Commits per group. `cargo-runner` output at `.addl/phase-1/r5-gN-cargo.log`. Mini-review JSON at `.addl/phase-1/r5-gN-mini-review.json`. Full test suite green after each group. | Each group gates the next per Section 3 dependency ordering. Ben triages mini-review findings before advancing. |
| **R4b — Post-implementation test review** (2 parallel subagents) | Same agents as R4 | Tests re-validated against real code. Catches the "tests pass vacuously" failure mode. | Ben triages; gate R6. |
| **R6 — Quality council** (14 parallel subagents, independent reports) | **14 seats total** (matches DEVELOPMENT-METHODOLOGY.md pattern). Full council: `architect-reviewer`, `code-reviewer`, `security-auditor`, `performance-engineer`, `qa-expert`, `test-automator`, `chaos-engineer`, `dx-optimizer`, `error-detective`, `refactoring-specialist`, `operation-primitive-linter`, `code-as-graph-reviewer`, **`best-practices-2026`** (to-create before R6), and **one domain-swap slot** (slot 11 in DEV-METHODOLOGY.md). For Phase 1 the domain-swap slot is **`ivm-algorithm-b-reviewer`** (IVM crate is the biggest scope expansion); `ucan-capability-auditor` and `benten-core-guardian` ran at mini-review time during R5 so they're not duplicated at R6. `determinism-verifier` also not duplicated (already validated at SPIKE level and CI enforces cross-target byte equality via T9). | 14 JSON reports at `.addl/phase-1/r6-<agent>.json`. Orchestrator merges by `location + claim`, triages. | Ben triages every finding with fix/defer/disagree. Phase 1 declared done when no critical/major findings remain. |

### Agents to create just-in-time (per DEVELOPMENT-METHODOLOGY.md "Missing Agents")

Create these the moment the stage that first needs them begins — do NOT create speculatively:

- **Before R1:** `benten-engine-philosophy`.
- **Before R2:** `benten-test-landscape-analyst`.
- **Before R3:** `rust-test-writer-unit`, `rust-test-writer-edge-cases`, `rust-test-writer-security`, `rust-test-writer-performance`, `rust-implementation-developer`.
- **Before R4:** `rust-test-reviewer` (and `rust-test-coverage` if `qa-expert` doesn't fit).
- **Before R6:** `best-practices-2026`.

The `implementation-developer` agent is created before R5 but is used in every R5 group.

---

## 7. Sequencing + Calendar

No dates. Expressed as blocker-of relationships.

```
R1 (spec review) ─── Ben triage ──► R2 (test landscape) ─── Ben triage ──► R3 (tests written)
                                                                                   │
                                                                              R4 (test review)
                                                                                   │
                                                                           Ben triage / fixes
                                                                                   │
                                                                                   ▼
                          ┌──────────────── G1 (core types + version chains) ─────────────┐
                          │                                                                 │
                          ▼                                                                 ▼
    G2 (graph backend reshape, NodeStore/EdgeStore, indexes)             G4 (caps policy, NoAuth, UCAN stub)
                          │                                                                 │
                          ▼                                                                 │
    G3 (transaction primitive + change stream)                                              │
                          │                                                                 │
             ┌────────────┴────────────┐                                                    │
             ▼                         ▼                                                    │
    G5 (IVM + 5 views)                                                                      │
             │                                                                              │
             └───────────────────────► G6 (evaluator + 8 primitives + invariants + TRANSFORM) ◄
                                                              │
                                                              ▼
                                   G7 (engine orchestrator + public API)
                                                              │
                                                              ▼
              G8 (napi bindings + @benten/engine TS wrapper + scaffolder + CI matrix)
                                                              │
                                                              ▼
                                      R4b (post-implementation test review)
                                                              │
                                                         Ben triage
                                                              │
                                                              ▼
                                      R6 (14-agent quality council)
                                                              │
                                                       Ben final triage
                                                              │
                                                              ▼
                                               Phase 1 complete — ship crate 0.1.0
```

**Critical-path observation:** G3 (transaction + change stream) is the pinch point. G5 (IVM), G6 (evaluator), and G7 (engine) all block on it. Consider shipping G4 (caps) in parallel with G2/G3 since it only needs G1 + the trait interfaces, which land in G2-A. This is reflected in the sequencing diagram.

**Parallelism summary:**
- G1 ships first (alone).
- G2 + G4 can run in parallel (both consume G1 only).
- G3 follows G2 (needs NodeStore/EdgeStore).
- G5 runs after G3 (needs change stream).
- G6 runs after G3 + G4 + G5 (needs txn, caps hook, and at least one view for content-listing crud end-to-end).
- G7 runs after G6.
- G8 runs after G7.

**Gate expectations between stages (Ben's triage checkpoints):**

1. **Post-R1:** every finding has a disposition. No "noted." Plan revised.
2. **Post-R4:** tests compile, fail expectedly, coverage plan acknowledged.
3. **Post-each-R5-group:** full test suite green, mini-review findings all dispositioned.
4. **Post-R4b:** tests validated against real code, no vacuous passes.
5. **Post-R6:** zero critical/major findings. Deferred minor findings have phase targets in docs.

---

## Appendix A — Deliverable-to-Group Traceability

| Deliverable | Group |
|---|---|
| C1 (keep) | — |
| C2 Edge | G1-B |
| C3 Value::Float | G1-A |
| C4 cid/multihash migration | G1-A (contingent on upstream) |
| C5 proptest harness | R3 + G1-A |
| C6 version chain | G1-B |
| C7 error codes in core | G1-A |
| G1 KVBackend reshape | G2-A |
| G2 RedbBackend split + DurabilityMode | G2-B |
| G3 transaction primitive | G3-A |
| G4 NodeStore/EdgeStore | G2-A |
| G5 indexes | G2-B |
| G6 MVCC docs | G2-B |
| G7 change stream | G3-A |
| G8 stress tests | R3 + G2 |
| G9 doctests | G2-A + G2-B |
| I1 ViewDefinition | G5-A |
| I2 subscriber | G5-A |
| I3 capability grants view | G5-B |
| I4 event-handler view | G5-B |
| I5 content-listing view | G5-C |
| I6 governance view | G5-B |
| I7 version-current view | G5-C |
| I8 stale/budget | G5-A |
| I9 Generalized Algorithm B | **Phase 2 (out of Phase 1 scope — see Rank 10)** |
| P1 CapabilityPolicy | G4 |
| P2 CapabilityGrant types | G4 |
| P3 NoAuthBackend | G4 |
| P4 UCANBackend stub | G4 |
| P5 E_CAP_DENIED wiring | G6-A |
| P6 `requires` property handling | G6-A |
| E1 primitive types | G6-A |
| E2 iterative evaluator | G6-C |
| E3 8 executable primitives | G6-A + G6-B |
| E4 TRANSFORM expr language | G6-B |
| E5 invariants 1/2/3/5/6/9/10/12 | G6-C |
| E6 transaction primitive surface | G7 |
| E7 toMermaid | G6-C |
| E8 engine.trace | G6-C |
| E9 Phase-2 primitives return error | G6-A |
| N1 (keep) | — |
| N2 full CRUD | G7 |
| N3 register/call/trace API | G7 |
| N4 IVM wiring | G7 |
| N5 caps wiring | G7 |
| N6 snapshot/transaction surface | G7 |
| N7 system-zone API | G7 |
| B1 (keep) | — |
| B2 Vitest smoke | G8-A |
| B3 full CRUD bindings | G8-A |
| B4 subgraph bindings | G8-A |
| B5 view-read binding | G8-A |
| B6 @benten/engine TS wrapper | G8-B |
| B7 typed-error classes | G8-B |
| T1 scaffolder | G8-C |
| T2 toMermaid DSL surface | G8-B |
| T3 trace DSL surface | G8-B |
| T4 multi-arch CI | G8-C |
| T5 MSRV CI | G8-C |
| T6 wasm-runtime CI | G8-C |
| T7 error codegen | G8-C |
| T8 napi-rs wasm32 compile-check | G8-C |
| T9 determinism gate (cross-leg byte equality) | G8-C |
| T10 supply-chain CI (cargo-audit / cargo-deny / lockfile verify / response protocol) | G8-C |
| T11 SECURITY-POSTURE.md — NoAuthBackend semantics, production() builder guide, threat-model notes | G8-C |
| T12 TRANSFORM-GRAMMAR.md — allowlist BNF + rejected-constructs appendix | G6-B |
| B8 napi input validation (size / depth / CID shape / pre-parse invariant checks) | G8-A |
| B9 DSL source-map + error.dslLocation | G8-B |
| N5-b Engine::builder().production() — refuses NoAuthBackend, requires explicit capability policy | G7 |
| N8 Engine private write-context flag — system-label rejection at benten-graph write-path (Phase 1 Invariant-11 stopgap) | G3-A + G7 |

Every deliverable has an owner group. Every "P1.*.*" tag from SPIKE "Critic Triage" has a landing spot. Every R1-criticality finding has a mapped deliverable. Nothing is orphaned.

---

## R1 Triage Addendum (2026-04-15)

This section extends the plan with the R1 spec-review dispositions. 5 critics (architect-reviewer, code-reviewer, security-auditor, dx-optimizer, benten-engine-philosophy) returned 61 findings total across 4 critical / 27 major / 30 minor. Raw JSON at `.addl/phase-1/r1-<agent>.json`. Consolidated triage at `.addl/phase-1/r1-triage.md`. Applying all fix-now dispositions here so R2 and R3 agents see the corrected plan as ground truth.

### Scope-shape changes

**(Security Critical #1 — system-zone stopgap.)** Even though full Invariant 11 is Phase 2, Phase 1 ships a write-context privilege flag: every WRITE through `benten-graph`'s transaction primitive carries a `WriteContext { is_privileged: bool }`. Only the engine's `grant_capability` / `create_view` / `revoke_capability` / internal paths set `is_privileged = true`; every user-operation WRITE sets it false. Writes to any Node labeled with a `system:` prefix are rejected with `E_SYSTEM_ZONE_WRITE` when `is_privileged` is false. New deliverable **N8**. Test `user_operation_cannot_write_system_labeled_node`.

**(Security Critical #2 — NoAuth guardrails.)** `NoAuthBackend` remains the builder default for DX, but Phase 1 ships `Engine::builder().production()` (new deliverable **N5-b**) which returns `Err(EngineError::NoCapabilityPolicyConfigured)` unless a capability policy is explicitly set. Startup emits an info-level log line naming the backend. `SECURITY-POSTURE.md` (new deliverable **T11**) documents the default's intended scope (embedded / single-user). Test `engine_builder_production_refuses_noauth`.

**(Security Critical #3 — supply chain.)** New deliverable **T10** (lands in G8-C): `cargo-audit` + `cargo-deny` in CI on every PR (fail on HIGH/CRITICAL), `cargo-deny.toml` with license allowlist (MIT/Apache-2.0/BSD-3/CC0) + banned crates, `cargo build --locked` verification in CI, `CONTRIBUTING.md` yank-response protocol section, weekly `cargo update --dry-run` + `cargo audit` workflow opening issues on new advisories.

**(Security Critical #4 — `requires` property semantics.)** Decision: `requires` declares the **minimum** capability; the evaluator additionally checks each primitive's effective requirement at call time. A handler with `requires: "post:read"` that internally WRITEs to `admin:*` gets the WRITE denied individually even if the declared read capability is granted. Update §2.4 P6 prose accordingly. New tests: `handler_with_understated_requires_denies_excess_writes`, `handler_cannot_escalate_via_call_attenuation`. This is the correct model for Phase 6 AI-agent attack surface — declared-only is the canonical AI-handler escalation path.

**(Architect Major #1 — change-stream refactor.)** `benten-graph` exposes a `ChangeSubscriber` trait; it does NOT depend on tokio. `benten-engine` ships a default tokio-broadcast `ChangeSubscriber` impl (bounded with stale-on-overflow semantics per Rank 2). WASM builds plug a synchronous callback-list impl. This preserves the thin-engine test and honors the architect-reviewer + philosophy + security critics' convergent finding. Section 2.2 G7 updated; Section 2.6 N4 gains subscriber-impl wiring; Rank 2 recommendation ratified with this architectural placement.

**(Architect Major #2 — evaluator frame model.)** `Vec<ExecutionFrame>` + `frame_index: usize` on the stack. No self-referential borrows. Tracing-snapshot refactor to arena/IDs is a Phase 2 option if trace state needs frame lifetime > stack lifetime. Section 2.5 E2 + G6-C updated.

**(Architect Major — IVM base trait.)** Add a shared `View` trait to `benten-ivm` that all five hand-written views implement. Phase 2's generalized Algorithm B slots in as another `View` impl. Forecloses less than the plan's current "5 independent implementations" shape. Section 2.3 I1–I7 gets a shared trait; G5-A owns the trait definition.

**(Architect Major — error-catalog crate split.)** Conflating `CoreError` with the workspace-wide `ErrorCode` enum is architecturally awkward — they're different types. Move the `ErrorCode` enum to a tiny new shared crate `benten-errors` (or keep in `benten-core` if the critic's arg is weaker after implementation). Decision: **keep in `benten-core` for Phase 1** (one fewer crate; the `ErrorCode` enum only grows at phase boundaries). Revisit at Phase 2 if the coupling surfaces concretely. Named compromise.

### Error-catalog gaps (code-reviewer + security)

Add to `docs/ERROR-CATALOG.md` before G1 begins:

- `E_PRIMITIVE_NOT_IMPLEMENTED` — reserved for WAIT/STREAM/SUBSCRIBE-as-user-op/SANDBOX call-time rejection
- `E_SYSTEM_ZONE_WRITE` — Phase 1 stopgap (N8)
- `E_CAP_NOT_IMPLEMENTED` — UCANBackend-configured-in-Phase-1 operator error (distinct from `E_CAP_DENIED`)
- `E_CAP_REVOKED_MID_EVAL` — explicit Phase-1 TOCTOU window marker (distinct from `E_CAP_REVOKED` which is Phase-3 sync-revocation)
- `E_CAP_DENIED_READ` — Phase-1-ships-Option-A (leaky-but-honest); Phase-3 revisit for existence-hiding per sync threat model
- `E_INPUT_LIMIT` — napi boundary rejection (B8)
- `E_TRANSFORM_SYNTAX` — TRANSFORM parser rejection (T12 BNF)
- `E_INV_CONTENT_HASH` — maps Invariant 10 enforcement
- `E_INV_REGISTRATION` — maps Invariant 12 enforcement

Remove from `ERROR-CATALOG.md` (prematurely present; they're Phase 2):

- `E_INV_ITERATE_MAX_MISSING` / `E_INV_ITERATE_BUDGET` → Phase 2 with Invariant 8
- `E_INV_SANDBOX_NESTED` → Phase 2 with Invariant 4
- `E_INV_SYSTEM_ZONE` → Phase 2 with full Invariant 11 (the Phase 1 stopgap uses `E_SYSTEM_ZONE_WRITE` instead)

### TRANSFORM grammar (code-reviewer + security + philosophy)

- G6-B commits to publishing `docs/TRANSFORM-GRAMMAR.md` as new deliverable **T12** (BNF + rejected-constructs appendix).
- Grammar is **allowlist** — every token/AST shape that isn't explicitly in the BNF is a parse error with `E_TRANSFORM_SYNTAX`. This is stronger than a denylist for a language whose security depends on what CAN'T appear.
- Expand §4.1 benten-eval rejection tests from 4 constructs to the full class: closures, `this`, imports, prototype access (`__proto__`, `constructor`, `prototype`), tagged templates, template literals with expressions, optional-chained calls exposing prototype, computed property names resolving to `__proto__`/`constructor`/`Symbol.*`, `new` in any position, `with`, destructuring with getters, spread-into-call, comma operator.
- G6-B also ships a fuzz-test harness (`cargo test -p benten-eval -- --ignored fuzz_transform_parser`) running generated JS snippets through the TRANSFORM parser; accepted strings must evaluate deterministically.

### IVM semantics (code-reviewer + dx-optimizer)

- **Stale-reader behavior explicit:** a read on a stale view returns `Err(E_IVM_VIEW_STALE)`, not stale data. Caller can opt into "stale is fine" via a new method `engine.read_view_allow_stale(view_id, query)` if desired.
- **Exit-criterion IVM barrier:** `engine.call(handler, input)` blocks on IVM propagation by default (per-tx barrier). Asynchronous fire-and-forget caller semantics opt-in via `engine.call_async(handler, input)`. Preserves the exit criterion's determinism and fixes the dx-optimizer flakiness concern.
- **Transaction edge cases:** closure-panic-in-transaction → rollback + re-raise; commit-time capability failure → `engine.trace()` returns the partial trace up to the failed commit boundary plus a final `TraceStep::Aborted { error: E_CAP_DENIED }`.
- **Nested transactions:** not supported in Phase 1. A `transaction(|tx| tx.transaction(|inner| ...))` call returns `E_NESTED_TRANSACTION_NOT_SUPPORTED`. Phase 2 may lift; Phase 1 keeps it deterministic.

### Attribution (security major, for Phase 6 foundations)

Both `ChangeEvent` and `TraceStep` gain minimum-attribution fields in Phase 1 even though Invariant 14 enforcement is Phase 2:

- `ChangeEvent { cid, label, kind, tx_id, actor_cid: Option<Cid>, handler_cid: Option<Cid>, capability_grant_cid: Option<Cid> }`
- `TraceStep { node_cid, inputs, outputs, duration_us, error, actor_cid: Option<Cid>, capability_grant_cid_used: Option<Cid> }`
- NoAuthBackend populates `actor_cid` with `noauth:<session-uuid>` synthetic marker so the field is never silently None.

Ensures Phase 2 Invariant 14 is a tightening, not a schema migration.

### Thin-engine test expanded (philosophy major)

`EngineBuilder` gains `without_versioning()` alongside the existing `without_ivm()` and `without_caps()` — Validated Decision #8 requires ephemeral data not pay versioning cost. Test renamed to `thinness_no_ivm_no_caps_no_versioning_still_works`.

### Subgraph CID order-independence (philosophy major)

Add proptest `prop_subgraph_cid_order_independent` in §4.2: two Subgraphs constructed via different insertion orders but equivalent structure must produce identical CIDs. Subgraph canonicalization rule (Nodes sorted by Cid; Edges sorted by `(source_cid, target_cid, label)`) published as part of T12 grammar doc or a new §7 sub-section.

### Iteration budget stopgap (philosophy major)

Invariant 8 (cumulative budget) is Phase 2, but Phase 1 enforces a hardcoded `MAX_ITERATE_NEST_DEPTH = 3` at registration (Invariant 12 slot). This prevents a handler from silently unbounded-nesting ITERATEs within per-level limits. Named compromise: "Phase 1 bounds nesting structurally; Phase 2 adds cumulative-budget enforcement." Documented in §2.5 E5 and Rank 10.

### DX hardening (dx-optimizer)

- **B8 prebuilt binaries.** napi-rs per-platform prebuilts for at least (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc). CI gate on cold-install < 60s. Source-compile fallback works but warns.
- **B9 DSL source-maps.** Thrown errors carry `error.dslLocation` (file:line:col from TS authoring) + synthetic stack frame so developers see their handler code, not Rust Node CIDs. Attached at register-time via `subgraph` DSL builder.
- **`crud` injection completeness.** `crud('post')` zero-config injects `createdAt`, `updatedAt`, `id` deterministically. `createdAt` and `updatedAt` use an HLC timestamp from the `uhlc` crate (pulled forward to Phase 1 from its previous Phase-3 reservation — `createdAt` enters the Node content hash, so the timestamp must be reproducible across processes for D2 to hold). The HLC is a local `uhlc::HLC` instance owned by the `Engine`; one monotonic stamp per WRITE, never re-computed on repeated reads. `id` is a deterministic BLAKE3 of `(label, hlc_stamp, nonce)`. `authorId` is NOT auto-injected — that requires a capability context the zero-config path doesn't have.
- **Mermaid consumer UX.** `toMermaid()` has a sibling `toMermaidUrl()` that wraps output in a mermaid.live URL for direct paste.
- **Trace pretty-print.** `TraceStep[]` has a `.toString()` that renders a tree view with per-step timing; raw array still available for programmatic use.
- **Scaffolder template.** `npx create-benten-app` output ends with a "next steps" banner pointing to docs + the Phase-1 tutorial in QUICKSTART.md.
- **`fixHint` surfacing.** Error codes carry `fixHint` from ERROR-CATALOG into the thrown `Error`; `error.toString()` includes `"Hint: <fix-hint>"`.
- **Doc deliverable.** QUICKSTART.md section marked "not yet built" updated at Phase 1 exit to reflect the shipped scaffolder + `crud('post')` workflow.

### Test-landscape additions

New subsection in §4.1 — **security-class tests** (seeds rust-test-writer-security in R3):

- `napi_rejects_oversized_value_map` / `napi_rejects_deep_nested_value` / `napi_rejects_oversized_bytes` / `napi_rejects_malformed_cid` (B8)
- `transform_grammar_rejects_tagged_templates` / `transform_grammar_rejects_computed_proto_keys` / `transform_grammar_rejects_new_Function` / `transform_grammar_rejects_with_statement` / `transform_grammar_rejects_destructuring_getter` / fuzz harness
- `capability_revoked_mid_iteration_denies_subsequent_batches` (TOCTOU window test)
- `handler_with_understated_requires_denies_excess_writes`
- `handler_cannot_escalate_via_call_attenuation`
- `user_operation_cannot_write_system_labeled_node`
- `read_denied_returns_cap_denied_read` (Option-A existence-visibility)
- `ucan_stub_error_message_names_phase_and_alternative`
- `ucan_stub_error_routes_to_ON_ERROR_not_ON_DENIED`
- `supply_chain_ci_green_on_clean_lockfile`

### ADDL dispatch adjustments (code-reviewer minor)

- §6 R5 row adds `cargo-runner` + `rust-engineer` with scoped roles — already done in pre-R1, verified here.
- `tests/evaluator_stack_*` glob in G6-C expanded to named tests: `tests/evaluator_pushes_next_on_ok`, `tests/evaluator_pops_on_respond`, `tests/evaluator_follows_error_edge`, `tests/evaluator_preserves_frame_order`, `tests/evaluator_stack_overflow_is_err_not_panic`.
- G5/G6 `mod.rs` ownership: each sub-file carries its own `pub(crate) mod …` reference; no agent writes to the `mod.rs` collector. Explicitly a `G5-A owns src/views/mod.rs; G5-B and G5-C add their view files via pub statements G5-A aggregates on merge` rule.
- Exit-criterion #4 trace-order assertion narrowed to topological order (non-strict total order for handlers with BRANCH/ITERATE/CALL — the test checks that every executed step appears in a topologically-valid order, not that the sequence is unique).

### Named compromises (things we chose to NOT fully fix in Phase 1)

Each is documented explicitly rather than silently deferred.

1. **Invariant 13 TOCTOU window** — Phase 1 checks capabilities at every commit boundary and at every CALL entry. Revocation during a long ITERATE is visible only at iteration-batch boundaries (configurable, default 100 iters). Phase 2 tightens to per-operation via Invariant 13. Docs (Rank 10 + §2.4 P1) name the window size.
2. **E_CAP_DENIED_READ leaks existence** (Option A chosen). Phase 3 sync revisits with per-grant `existence_visibility: visible|hidden` option.
3. **`benten-errors` stays inside `benten-core`** for Phase 1. Revisit at Phase 2 if coupling surfaces.
4. **WASM runtime still Phase 2.** T8 is compile-check only. When runtime ships, default capability backend for browser contexts will NOT be NoAuthBackend — `BrowserOriginCapBackend` will scope writes to origin. Named for Phase 2.
5. **Per-capability write rate limits** — Phase 1 records `benten.ivm.view_stale_count{view_id}` metric; Phase 3 enforces per-peer rate limits when sync ships.
6. **BLAKE3 128-bit collision resistance assumption.** Phase 1 usage (dedup + integrity) relies only on collision resistance, not full preimage. Phase 3 UCAN capability-by-CID paths revisit; `SECURITY-POSTURE.md` documents the property.
7. **`[[bin]]` required-features gating for `benten-graph::write-canonical-and-exit`.** Phase 1 builds a small test-fixture binary under `crates/benten-graph/bin/` used by the cross-process determinism test. Currently gated with `test = false, bench = false` but not `required-features`, so `cargo build` still compiles it. Since `benten-graph` has no external consumers in Phase 1, the compile tax is zero. When `benten-graph` publishes to crates.io (Phase 2+ release), add a `test-fixtures` feature and gate the `[[bin]]` with `required-features = ["test-fixtures"]` so downstream consumers don't pay the compile cost. Tracked at `.addl/phase-1/r4-pass2-triage.md` finding qa-p2-3.

### Disagreements

None across all 61 findings.
