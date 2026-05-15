# benten-graph — Crate Internals

A plain-English, code-grounded tour of the `benten-graph` crate as it stands at
HEAD `8141b94` (Phase-4-Foundation close, post tag `phase-4-foundation-close`).
The crate body has been substantively stable since the Phase-3 close window —
the last source change was the `docs(phase-rename)` retense at `00f2784`
2026-05-11, and the last code change was `dcd1275` (class-e bug fixes,
2026-05-10) + `92bd65e` (W9-T6 verify-on-read, 2026-05-08). This is a READ-ONLY
audit — no claims here about future plans, just what the code does today and
where the design seams sit.

## 1. What this crate does

`benten-graph` is the storage layer. It is the only place where the Benten engine
talks to disk (or to any other byte-level KV store). Everything above it deals in
Nodes, Edges, Subgraphs, change events, and content-addressed CIDs — everything
below it is bytes and prefixes.

Three responsibilities live here. First, a narrow byte-level key/value waist
(`KVBackend` — get / put / delete / scan / put_batch with atomic semantics). Second,
a node + edge layer on top (`NodeStore` / `EdgeStore`) that hides the DAG-CBOR
encode/decode and owns the on-disk key schema. Third, a small but load-bearing
collection of cross-cutting concerns: a closure-based `Transaction` primitive,
MVCC snapshots, a `ChangeSubscriber` channel that IVM and friends hook into, two
multimap indexes for label and `(label, property, value)` lookups, an Inv-13
immutability cache, and a system-zone label gate. The production storage backend
is `RedbBackend` over redb v4; a `BrowserBackend` (in-RAM, wasm32 thin-client),
an `InMemoryBackend`, a read-only `SnapshotBlobBackend`, a typed-error
`NetworkFetchStubBackend`, and a redb-backed `RedbBlobBackend` for the
`system:ModuleBytes` zone also live here.

## 2. Where it sits in the dependency chain

**Workspace dependencies in:**
- `benten-core` — Cid, Node, Edge, Value, Subgraph, WriteAuthority, CoreError.
- `benten-errors` — the stable `ErrorCode` enum used by `GraphError::code()`.
- `blake3` — used directly by `store_subgraph` / `RedbBlobBackend::put_sync` to
  recompute content hashes; `benten-core::Cid::from_blake3_digest` is the wrapper.
- `serde` / `serde_ipld_dagcbor` — DAG-CBOR encoding for Nodes, Edges, Subgraphs,
  index values, and the SnapshotBlob payload.
- `thiserror` — error enum derives.

**External, native-only (cfg-gated to `not(wasm32-unknown-unknown)`):**
- `redb` v4 — the production page-store. Gated out of the browser tab build per
  CLAUDE.md baked-in #17. `wasm32-wasip1` keeps redb because WASI is native-shape.

**Dev-deps:** `tempfile`, `proptest`, `criterion`, `benten-engine`
(with `test-helpers`), `benten-caps`. The dev-cycle on `benten-engine` is safe —
only the integration tests use it.

**Workspace consumers out (depend on `benten-graph`):**
- `benten-caps` — for `WriteAuthority` re-exports / `WriteContext` interop.
- `benten-engine` — primary consumer; threads `RedbBackend` (or a generic
  `B: GraphBackend`) through every primitive host and the transaction bridge.
- `benten-ivm` — subscribes to the `ChangeSubscriber` stream to drive view
  recomputation (the trait shape lives here; the subscriber concretion lives in
  `benten-engine::change`).
- `benten-eval` is NOT a direct consumer of this crate today — the
  evaluator goes through `benten-engine` for storage access. The grep above
  matches the engine's transitive position in the dep graph.

**External consumers:** none — `benten-graph` is private workspace surface today.
The trait shapes (`KVBackend`, `GraphBackend`, `BlobBackend`) are designed for
external backends to plug in (peer-fetch, iroh-fetch, IndexedDB) but no such
crate ships from the workspace yet.

## 3. Files inventory in `src/`

### `lib.rs` (986 LOC)
Crate root + module list + crate-level re-exports.
Owns the `GraphError` enum (the canonical storage-layer error type with seven
variants and a `code()` mapping to the stable error catalog), the
`SnapshotHandle` type for redb MVCC reads, and the `WriteContext` struct that
carries `(label, is_privileged, authority)` into write paths. Also hosts a
secondary `impl RedbBackend` block of Phase-2a hooks: `create`,
`get_node_label_only` (for the Inv-11 runtime probe), `store_subgraph` /
`load_subgraph_verified`, `benchmark_helper_crud_post_create_dispatch`, plus a
collection of cfg-gated `*_for_test_impl` hooks that drive the Inv-13 5-row
matrix. **Key invariants:** `GraphError::Decode` and the `Redb(String)` variant
exist for test-fixture injection only; production error paths funnel through
`RedbSource(redb::Error)` so `std::error::Error::source()` still walks the chain.
`WriteContext::privileged_for_engine_api` is the only constructor that flips
`is_privileged = true`.

### `backend.rs` (380 LOC)
The narrow byte-level trait. Owns `KVBackend` (get / put / delete / scan /
put_batch + associated `Error` type), `ScanResult` (a shape-opaque newtype over
`Vec<(Vec<u8>, Vec<u8>)>` that hides whether the implementation is materialized
or streaming), `ScanIter`, `BatchOp`, and `DurabilityMode`. **Key invariants:**
`ScanResult` is *shape-opaque* — callers do not see slice semantics by default;
Phase 2+ can swap the storage for a boxed iterator without semver break.
`KVBackend::put_batch` is atomic-or-nothing. `DurabilityMode::default()` returns
`Group` since Phase-3 G13-E (was `Immediate` through Phase-2b), closing
Compromise #12 at the engine-level posture even though the redb v4 mapping still
collapses Group → Immediate.

### `store.rs` (442 LOC)
The mid-level node/edge traits and the change-event schema. Owns the key-schema
helpers (`node_key`, `edge_key`, `subgraph_key`, `edge_src_index_key`,
`edge_tgt_index_key`, plus their prefix variants), `NodeStore` and `EdgeStore`
traits, `ChangeKind` (Created / Updated / Deleted / EdgeCreated / EdgeDeleted),
`ChangeEvent` (cid, labels, kind, tx_id, optional attribution triple, optional
Node body, optional edge endpoints), and `ChangeSubscriber` (the runtime-agnostic
synchronous trait). **Key invariants:** no blanket `impl<T: KVBackend> NodeStore
for T` — the prior version of that blanket silently bypassed index maintenance
under generic dispatch, so each backend now opts in explicitly (the g2-cr-1
footgun fix). Key schema is `n:CID`, `e:CID`, `s:CID`, `es:SRC|EDGE`,
`et:TGT|EDGE` — five prefixes, no escape characters needed because CID byte
length is fixed.

### `redb_backend.rs` (1848 LOC)
The production backend. Single concrete struct (`RedbBackend`) wrapping a
`redb::Database`, the configured durability, a shared `CidExistenceCache` for
the Inv-13 fast path, a transaction-flag Mutex (nested-txn detection), an atomic
monotonic `tx_id` counter, an `Arc<Mutex<Vec<Arc<dyn ChangeSubscriber>>>>`, an
atomic `writes_committed` counter, and cfg-gated test-only bookkeeping
(`last_durability_by_label`, `test_event_log`). Holds all redb table definitions
(`NODES_TABLE`, `LABEL_INDEX_TABLE`, `PROP_INDEX_TABLE`) plus all redb-specific
plumbing. **Key invariants:** the `put_node_with_context` write path folds the
in-txn existence probe + conditional write into a SINGLE redb write transaction
to defeat the G2-A / G5-A TOCTOU race (the bloom fast-path is bypassed inside
the txn because its "absent" answer can race with uncommitted concurrent
writers); dedup early-returns drop the txn without commit and do not advance the
`writes_committed` counter (the §9.11 row-3 pure-read-dedup contract); the
inherent `put_node` / `put_edge` always run the system-zone guard before any
redb call (the chaos-engineer g3-ce-1 / g3-ce-2 fix that closed the
binding-caller bypass).

### `transaction.rs` (760 LOC)
The closure-based `Transaction` primitive. Owns `PendingOp` (the four-variant
pending-ops enum: PutNode / PutEdge / DeleteNode / DeleteEdge — each carrying
enough state to construct the post-commit ChangeEvent without re-reading the
graph), `TxGuard` (RAII guard over the backend's `tx_flag` Mutex; clears on
drop even under panic), `Transaction<'a>` (the closure handle — wraps a
`redb::WriteTransaction`, a pending-ops vec, and the active `WriteAuthority`),
and the `fan_out` function that synchronously dispatches change events to every
subscriber after a successful commit (with `catch_unwind` so a single panicking
subscriber cannot poison the commit thread). **Key invariants:** nested
`Transaction::transaction(...)` always returns
`NestedTransactionNotSupported`; closure-Err drops the txn cleanly via redb's
abort-on-drop and surfaces `TxAborted`; `delete_node` cascades to every
referencing Edge inside the same redb txn (r6b-ivm-1 — the prior version left
dangling edges); `PendingOp::PutEdge` always emits `ChangeKind::EdgeCreated`,
never `Created`, so edge-driven IVM views see the right event shape.

### `indexes.rs` (90 LOC)
Label and property-value index plumbing. Two `MultimapTableDefinition`s
(`LABEL_INDEX_TABLE`, `PROP_INDEX_TABLE`) plus three crate-private helpers:
`value_index_bytes` (DAG-CBOR-encode a `Value` for use as an index key
component), `property_index_key` (length-prefixed pack of
`label || prop_name || value_bytes`), and `cid_from_index_bytes`. **Key
invariants:** value encoding uses the same `serde_ipld_dagcbor` encoder Nodes
and Edges use, so `Value::Int(10)` and `Value::Text("10")` are
distinguishable in the index; the property-index key is length-prefixed in u32
big-endian so `(label="Po", prop="stviews")` and `(label="Post", prop="views")`
cannot collide. Cfg-gated to non-`wasm32-unknown-unknown` because the tables
are redb-typed.

### `immutability.rs` (454 LOC)
The Inv-13 fast-path cache. Owns `BloomFilter` (a pedagogically-simple double-
hashing bloom keyed on the BLAKE3 digest portion of the CID; no external hash
dep, ~10ns probe), `CidExistenceCache` (wraps the bloom + a forced-collision
one-shot + a forced-positive set + an optional `warmed` HashSet under test cfg),
and `DEFAULT_FALSE_POSITIVE_RATE` (1/10000). **Key invariants:** `may_contain`
takes `&mut self` because the one-shot collision flag must clear after firing;
`may_contain_peek` is the non-mutating variant the warmness assertions use;
the fast-path contract is "false means definitely absent, true means run the
authoritative redb read." The `warmed` set is cfg-gated and bounded at
`WARMED_CAP = 100_000` so a long-running integration process cannot grow it
unboundedly (G11-A capture).

### `in_memory_backend.rs` (229 LOC)
A pure-trait in-memory `KVBackend` backed by `Mutex<BTreeMap>`. Used as the
proof-of-equivalence reference for the redb impl (the
`tests/in_memory_backend_equiv_to_redb.rs` property test asserts identical
behavior on the trait surface). NOT a `NodeStore` / `EdgeStore` — those traits
carry index maintenance and change-event publishing that are RedbBackend-specific
concerns (the `:memory:` engine path uses redb's own `InMemoryBackend` page
store so it retains full RedbBackend semantics; this file is the bare-trait
reference impl for future non-redb consumers).

### `mutex_ext.rs` (88 LOC)
Two extension traits, `MutexExt::lock_recover` and `RwLockExt::{read,write}_recover`,
that collapse the workspace-wide
`.lock().unwrap_or_else(|e| e.into_inner())` idiom (35 callsites pre-r6-ref).
Project posture: lock poisoning is not a panic path; recover with the inner
guard and keep going.

### `prefix_helpers.rs` (42 LOC)
A single function, `next_prefix`, that computes the lexicographic successor of a
byte prefix (returns `None` for all-`0xff` so callers do an unbounded `prefix..`
scan instead). Promoted out of `redb_backend.rs` so `InMemoryBackend` and
`BrowserBackend` could share it without dragging redb into the wasm32 build.

### `graph_backend.rs` (394 LOC)
The umbrella `GraphBackend` trait, introduced at G13-A as the canary for the
generic-cascade direction. Composes `KVBackend + NodeStore + EdgeStore` plus
four new associated items: `type Snapshot` (constrained to `Send + Sync +
'static` so the engine can hold a snapshot across `.await` boundaries), `type
Error` (the typed-error shape the engine boundary erases to `Box<dyn Error>`),
`type Transaction` (an owned marker — `RedbTransactionRunner` /
`BrowserTransactionRunner`), and the four required methods (`transaction`,
`register_subscriber`, `snapshot`, `put_node_with_context`). **Key invariant:**
the trait is intentionally NOT object-safe (associated types preclude `dyn`).
Engine consumes it through the generic-cascade direction
(`Engine<B: GraphBackend>`); the test
`engine_does_not_reference_dyn_graph_backend_at_engine_boundary` pins that.

### `browser_backend.rs` (667 LOC)
The wasm32-unknown-unknown thin-client cache. In-RAM `Mutex<BTreeMap>` keyed
under the same `n:CID` / `e:CID` / `es:` / `et:` schema as `RedbBackend`. Owns
`BrowserBackend`, `BrowserSnapshot` (an owned `BTreeMap` clone — independent of
subsequent live writes per the `br-r4-r1-1` contract), and
`BrowserTransactionRunner` (a unit marker). Feature-gated behind
`browser-backend` on the crate, NOT default. **Key invariants per CLAUDE.md
baked-in #17:** no transactions, no real subscribers (`register_subscriber` is
a silent no-op), no sync state, no IndexedDB persistence (that's the separate
G18-A surface). `put_node_with_context` bypasses cap-recheck at the cache
layer (the upstream G14-D subscription already filtered events per grant) but
the system-zone label gate is preserved as a defense-in-depth check against a
buggy subscription.

### `backends/mod.rs` + four submodules
- `backends/blob_backend_trait.rs` (119 LOC) — the `BlobBackend` trait scaffold.
  Three methods (`get`, `put`, `is_persistent`) returning `impl Future + Send`
  per D-PHASE-3-7 (browser-target async compatibility). Associated `type Error`.
  Not object-safe (RPITIT + assoc type). Generic-cascade direction.
- `backends/blob_backend.rs` (344 LOC, native-only) — `RedbBlobBackend`, the
  concrete redb-native impl. Stores blobs as `system:ModuleBytes` Nodes (label +
  `blob_cid: Text` + `blob_bytes: Bytes` properties) through
  `put_node_with_context(privileged_for_engine_api())`. Defense-in-depth
  recomputes `BLAKE3(bytes)` and rejects `CidMismatch` at the put boundary.
  Closes Compromise #17 (in-memory module-bytes registry).
- `backends/snapshot_blob.rs` (564 LOC) — `SnapshotBlobBackend`, a read-only
  `KVBackend` over a canonical DAG-CBOR `SnapshotBlob` payload
  (schema_version=1, anchor_cid, nodes:BTreeMap<Cid,Vec<u8>>, system_zone_index).
  Writes surface `BackendReadOnly`. Used for Phase-3 sync handoff
  (peer A exports, peer B imports). Implements `GraphBackend` too with
  `SnapshotBlobSnapshotHandle` + `SnapshotBlobTransactionRunner` unit markers.
- `backends/network_fetch_stub.rs` (228 LOC) — `NetworkFetchStubBackend`, a
  typed-error-only stub reserving the trait shape for the Phase-3
  iroh-fetch impl. Every operation returns a typed error
  (`Phase3DeferredFetch` for reads, `BackendReadOnly` for writes) so any
  call site fails loud rather than degrading silently.

## 4. Public API surface

**The `KVBackend` trait** (`backend.rs`) is the narrow waist. Five methods —
`get(&self, &[u8]) -> Result<Option<Vec<u8>>, Self::Error>`,
`put(&self, &[u8], &[u8]) -> Result<(), Self::Error>`,
`delete(&self, &[u8]) -> Result<(), Self::Error>`,
`scan(&self, &[u8]) -> Result<ScanResult, Self::Error>`,
`put_batch(&self, &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error>`. Self::Error
is constrained to `std::error::Error + Send + Sync + 'static` (the
P1.graph.error-polymorphism deliverable). Required supertrait: `Send + Sync`.

**The `NodeStore` and `EdgeStore` traits** (`store.rs`) sit above `KVBackend`
conceptually but are implemented per-backend (no blanket impl). `NodeStore`
exposes `put_node`, `get_node`, `delete_node`, and `get_node_label_only`
(default impl decodes the full Node and projects the first label; the redb
impl overrides for the Inv-11 fast-path). `EdgeStore` exposes `put_edge`,
`get_edge`, `delete_edge`, `edges_from(source)`, `edges_to(target)`.

**The `ChangeSubscriber` trait** (`store.rs`) — a single synchronous method
`on_change(&self, event: &ChangeEvent)` — is the IVM/observability seam. No
async dependency, no tokio surface. Subscribers register through
`RedbBackend::register_subscriber(Arc<dyn ChangeSubscriber>)`. The
`ChangeEvent` struct carries cid + full labels vec + kind + tx_id + optional
attribution triple (actor / handler / capability-grant) + optional Node body
+ optional edge endpoints.

**The `GraphBackend` umbrella trait** (`graph_backend.rs`) composes
`KVBackend + NodeStore + EdgeStore` plus the four extra methods
(`transaction()`, `register_subscriber()`, `snapshot()`,
`put_node_with_context()`) and three associated types. Not object-safe by
construction.

**The transaction primitive** is the closure-based
`RedbBackend::transaction(|tx: &mut Transaction| -> Result<R, GraphError>)`.
Inside the closure, callers use `tx.put_node`, `tx.put_node_with_attribution`,
`tx.put_edge`, `tx.delete_node`, `tx.delete_edge`. The closure's `Ok`
commits the redb txn and fans events to every subscriber; the closure's `Err`
or panic aborts via redb's drop-on-uncommitted behavior. Nested
`tx.transaction(...)` always returns `NestedTransactionNotSupported`.

**MVCC snapshots:** `RedbBackend::snapshot() -> SnapshotHandle` returns an owned
handle that captures a `redb::ReadTransaction`. `SnapshotHandle::get_node(&Cid)`
and `SnapshotHandle::scan_label(&str)` read through it; concurrent writes are
invisible until the handle drops. `BrowserBackend::snapshot()` returns a
`BrowserSnapshot` (owned `BTreeMap` clone — independent of subsequent writes
by the same contract).

**Index reads:** `RedbBackend::get_by_label(&str) -> Vec<Cid>` and
`RedbBackend::get_by_property(&str, &str, &Value) -> Vec<Cid>`. Both are
scoped by label; cross-label property queries are out of scope today.

**Subgraph storage:** `RedbBackend::store_subgraph(&Subgraph) -> Cid` and
`RedbBackend::load_subgraph_verified(&Cid) -> Option<Subgraph>`. Hash-first
verification — bytes are BLAKE3'd against the supplied CID before any decode
attempt, so a tamper surfaces as `E_INV_CONTENT_HASH` not a confusing
serialize error.

**Errors:** `GraphError` is the canonical error type for `RedbBackend` (and
the type `CoreError` flows into). Seven variants today, `#[non_exhaustive]`.
Stable error codes via `GraphError::code() -> ErrorCode`. `BackendNotFound`
redacts the absolute path in its Display form (r6-err-7) so user-facing
strings don't leak `$HOME`.

## 5. Tests inventory (48 integration tests)

Most-load-bearing groups:

- **KVBackend conformance (`kvbackend_conformance.rs`, `proptests_kvbackend_laws.rs`,
  `scan_iterator.rs`, `scan_zero_hit.rs`):** the byte-level get/put/delete/scan
  contract, including a property test of the three algebraic laws every
  implementation must obey.
- **Backend equivalence (`in_memory_backend_equiv_to_redb.rs`):** property test
  that `InMemoryBackend` and `RedbBackend` produce byte-for-byte identical
  results on arbitrary op sequences.
- **Inv-13 immutability matrix (`inv_13_matrix.rs`,
  `inv_13_bloom_false_positive.rs`, `inv_13_toctou_race.rs`,
  `inv_13_dedup_does_not_emit_changeevent.rs`,
  `inv_13_dedup_path_does_not_advance_audit_sequence.rs`,
  `immutability_rejects_reput.rs`):** the §9.11 5-row dispatch matrix end-to-end,
  including the TOCTOU race closure, the bloom false-positive fallback, and
  the dedup-is-a-pure-read contract.
- **Transaction (`transaction_atomicity.rs`, `nested_tx_rejected.rs`,
  `failure_injection_rollback.rs`):** atomicity under Err/panic, nested-txn
  rejection.
- **System zone (`system_zone.rs`, `system_zone_inherent_put_blocks_user.rs`,
  `system_zone_inherent_put_edge_blocks_user.rs`):** R1 SC1 — the
  `"system:"` label gate on both Node and Edge writes including the
  binding-caller bypass closure.
- **Indexes (`label_prop_index.rs`, `indexes.rs`, `indexes_float_zero_parity.rs`,
  `indexes_idempotent_put_node_twice.rs`):** label + property-value index
  maintenance under puts, deletes, idempotent reputs, and Float(0.0)/Float(-0.0)
  canonicalization.
- **MVCC + snapshot (`mvcc_snapshot.rs`):** reader-stable view across
  concurrent writes.
- **Durability (`durability_default.rs`, `durability_group_enum_preserved.rs`,
  `capability_grant_writes_immediate.rs`,
  `crud_fast_path_apfs_timing_within_target.rs`):** the G13-E posture
  flip + the capability-grant-Immediate override.
- **GraphBackend trait pins (`graph_backend_trait.rs`,
  `blob_backend_*` family):** the G13-A umbrella + the G13-pre-B blob-backend
  scaffold's object-safety + browser-target compatibility properties.
- **Browser backend (`browser_backend.rs`):** the thin-client cache contract
  including snapshot independence, no-op subscriber registration, cap-recheck
  bypass, and system-zone gate preservation.
- **Cross-process (`d2_cross_process_graph.rs`):** spawns the
  `write-canonical-and-exit` bin in a fresh PID + asserts the parent reads
  back the same CID. Pins content-addressing determinism end-to-end.
- **Cascade delete (`cascade_edge_delete.rs`):** Node delete cascades to every
  referencing edge inside the same redb txn (r6b-ivm-1 regression).
- **Cache eviction (`unbounded_cache_eviction.rs`):** the G11-A unbounded-cache
  capture — `CidExistenceCache::warmed` and friends are bounded.
- **Verify-on-read (`get_node_verifies_content_hash_on_read.rs`):** the W9-T6
  Phase-3 wave-9 closure — `RedbBackend::get_node` recomputes BLAKE3 over the
  stored bytes and fires `E_INV_CONTENT_HASH` on tamper.

Plus a long tail of single-finding regression pins (`subgraph_load_verified_migration.rs`,
`graph_error_hygiene.rs`, `concurrent_reader_writer_soak.rs`,
`open_existing_vs_create.rs`, `change_event.rs`, `change_subscriber_trait.rs`,
`backend_error_polymorphism.rs`, `network_fetch_stub.rs`,
`snapshot_blob_backend.rs`, `snapshot_blob_kvbackend.rs`,
`get_node_label_only.rs`, `security_posture_compromise_12_marked_closed.rs`,
`node_edge_store_blanket.rs`).

## 6. Benches inventory (6 files + README)

Six criterion benches live in `benches/`, all with `harness = false` and the
`bench = false` lib flag set on the crate so `cargo bench --workspace` passes
criterion CLI flags through cleanly.

- **`get_create_node`** — gated against §14.6: `get_node/hot_cache` (1–50 µs
  target), `get_node_batch_100/hot_cache_same_cid` (<50 µs amortized),
  `create_node_immediate/default_durability` (100–500 µs).
- **`durability_modes`** — informational. Immediate vs. Group vs. Async at
  single-write, batch-100, and sustained-throughput layers. Demonstrates
  the Group→Immediate collapse on redb v4 (group ≈ immediate is expected).
  Documents the macOS APFS fsync floor (~4-13ms regardless of payload).
- **`concurrent_writers`** — informational. 1/2/4/8/16 writer threads against
  one backend; characterizes redb's single-writer-lock contention curve.
- **`multi_mb_roundtrip`** — informational. 1MB / 10MB / 100MB Node round-trips
  to catch CBOR-encoder O(n²) regressions or redb page-store cliffs.
- **`crud_post_create_dispatch_group_durability`** — Phase-2a descope-witness
  bench; routes through `benchmark_helper_crud_post_create_dispatch_impl` so
  the Group→Immediate redb mapping is observable through the production
  `put_node_with_context` path.
- **`mvcc_read_latency`** — informational. MVCC read latency under 0 / 1 / 4 /
  16 concurrent writers. Not a gate.

The `benches/README.md` is unusually thorough and worth reading before tweaking
any of the numbers — it documents Phase-1 storage-layer characteristics that
shape every result.

## 7. Thin-engine + composable-graph philosophy check

The storage layer is where the philosophy meets disk, so let me name what I see
honestly — well-respected places first, then frictions.

**Where the trait waist is respected well.** `KVBackend` is genuinely narrow:
five methods, all byte-shaped, with a per-backend associated `Error` type so
no backend has to lie through a redb-shaped variant. The
`P1.graph.error-polymorphism` deliverable is real — `InMemoryBackend`,
`SnapshotBlobBackend`, and `NetworkFetchStubBackend` all have their own
typed-error enums. `ScanResult` is genuinely opaque — the field is
crate-private, the only public accessors are `.len()`, `.is_empty()`,
`.as_slice()`, `.iter()`, and the `IntoIterator` impl, and the `Deref<Target=[..]>`
that the spike had is gone. The `ChangeSubscriber` trait carries no async-runtime
dependency at all — `benten-graph` does not depend on tokio, and the channel
concretion lives in `benten-engine::change`. That is the right place. The
`BlobBackend` trait scaffold is small and uses `impl Future + Send` (RPITIT)
deliberately so the IndexedDB browser variant can satisfy it without dragging
tokio into the wasm bundle.

**Where redb specifics intentionally leak.** `GraphError` has a
`RedbSource(redb::Error)` variant directly. `SnapshotHandle` carries an
`Option<redb::ReadTransaction>` field. The `Transaction` struct wraps a
`redb::WriteTransaction` directly. None of these is hidden behind another trait.
The architectural decision is explicit: `RedbBackend` IS the production
concrete; the trait surface (`GraphBackend`) exists for the browser thin-client
and the snapshot-blob handoff, not for runtime polymorphism via `dyn`. So
"redb-flavored" struct internals are fine because consumers go through the
inherent `RedbBackend` methods or through generic-cascade `<B: GraphBackend>`.
The friction would only matter if someone tried to write a Postgres/sled
backend that wanted to ride on `Transaction` directly — they would need a
parallel transaction type. The current shape commits to "transactions are a
per-backend inherent surface; the umbrella exposes a marker." That's a real
boundary choice, not an accidental leak.

**Where the philosophy is bent: index types are baked in at the storage layer.**
The two indexes (`LABEL_INDEX_TABLE`, `PROP_INDEX_TABLE`) are hard-coded
redb tables. `get_by_label` and `get_by_property` are inherent on
`RedbBackend`. The `BrowserBackend` does not maintain these indexes at all
(it has the `es:` / `et:` edge indexes but not label/property — see
`browser_backend.rs:316-345`). This is a real friction with the
"application-layer composition" philosophy from CLAUDE.md — these indexes
look like the kind of thing an application's IVM views could maintain
declaratively on top of the change stream, not something the storage layer
should bake in. Phase-1 baked them in because the engine's privileged write
paths (capability grants, version chains) need O(1) label-keyed reads at
runtime; an IVM-view-based replacement would have to be live before any of
those engine paths boot. Worth naming, not necessarily worth changing.

**Where the philosophy is bent: `put_node_with_context` is special-cased and
load-bearing.** The `WriteAuthority` enum on `WriteContext` drives three things
inside `put_node_with_context`: (1) the system-zone label gate, (2) the redb
durability tier (EnginePrivileged forces Immediate, SyncReplica forces None,
User honors the configured), (3) the Inv-13 5-row dispatch matrix (User-reput
→ E_INV_IMMUTABILITY; EnginePrivileged/SyncReplica reput → silent dedup).
None of this composes from KVBackend primitives — it's the engine's
capability/invariant policy stretched into the storage layer. The right
abstraction would be a pre-write hook trait (`CapabilityPolicy`) at the engine
layer; today the storage layer carries the policy directly. The TODO at
`lib.rs:797` ("phase-3 — write-authority/is_privileged coherence") flags
that the two axes can drift; that's the live tension.

**Where the philosophy is well-respected: in-memory vs. durable.** The
distinction is explicit and honest. `InMemoryBackend` (pure trait) is
documented as "NOT a NodeStore/EdgeStore on purpose" because those traits
imply change events + index maintenance — concerns that belong to the
engine-wired production backend. The `:memory:` engine path goes through
redb's own in-memory page store via `RedbBackend::open_in_memory()` so all
RedbBackend semantics (Inv-11, Inv-13, change events, transactions) stay
intact even without disk. `BrowserBackend` is explicit about being a
thin-client CACHE (per CLAUDE.md baked-in #17) — no transactions, no
subscribers, no sync state, no IndexedDB persistence. The doc says it
loudly and the impl matches.

**D-PHASE-3-25 + CLAUDE.md #17 implications visible.** The heterogeneity
contract is honest. `cfg(not(target_arch = "wasm32"))` cleanly excludes redb
+ `RedbBackend` + `SnapshotHandle` + the redb error sub-types from the
browser-tab build; `wasm32-wasip1` is treated as native-shape and keeps
everything. The `BrowserBackend` cap-recheck-bypass at the cache layer + the
no-op subscriber registration are both load-bearing per the spec and
genuinely match: a buggy thin-client subscription that delivered a system-zone
event without privilege would still surface `E_SYSTEM_ZONE_WRITE` at the
cache, but per-row cap-recheck on the local cache would double-fire against
the upstream filter. That's the right place to draw the line.

**Snapshot independence.** Worth calling out: `BrowserSnapshot` is an
owned-clone of the backing `BTreeMap` (per the `br-r4-r1-1` contract); the
`RedbBackend::SnapshotHandle` wraps a `redb::ReadTransaction` which redb
itself implements via Arc-counted page references. Both satisfy
`Send + Sync + 'static` per the `arch-r1-6` requirement so the engine can
hold a snapshot across `.await` points. The two snapshot impls thus look
different at the struct level but match at the contract level.

## 8. Phase 4-Meta + post-v1 expectations

Phase 4-Foundation has closed at tag `phase-4-foundation-close`. The
substantive Phase-4 platform work (admin UI v0, full plugin manifest schema,
schema-driven rendering, materializer pipeline, IVM-subgraph generalization,
decentralized registry on top of Atriums) landed in that window; the storage
layer absorbed it without trait-surface change. Phase 4-Meta + post-v1
materializer + admin-UI dogfood work will continue to read graphs heavily and
care about three things in this crate:

- **`get_node`'s verify-on-read posture.** Already wired (W9-T6). The
  ~3-10µs BLAKE3 recompute cost per `get_node` is the floor; if
  materialization needs sub-microsecond reads against trusted snapshots,
  there is no opt-out today and the doc explicitly treats the redb file as
  a system boundary. A future fast-path that skips verify-on-read for an
  active snapshot handle would need a new method (`get_node_unverified`?)
  — adding it under the snapshot's read-txn lifetime would be safe because
  the snapshot already commits to a point-in-time consistent view.
- **`get_by_label` / `get_by_property` as the materializer's index reads.**
  Both are label-scoped today and return owned `Vec<Cid>`. Schema-driven
  rendering of "all Posts whose category=Foo" would hit `get_by_property`
  directly; for cross-label queries (e.g., "everything tagged urgent")
  there is no path today and the multimap shape doesn't support it
  without a second index.
- **Owned snapshots across `.await`.** Both `SnapshotHandle` and
  `BrowserSnapshot` are `Send + Sync + 'static` so the materializer can
  hold one across await points without lifetime jiu-jitsu.

The admin UI v0 will write workflows through the engine, which means the
storage-layer touchpoints will be transactional writes through
`Transaction::put_node_with_attribution`. The attribution triple
(actor / handler / capability-grant) is already plumbed through
`PendingOp::PutNode` into `ChangeEvent`, so an audit view can render
"who created this workflow, under what capability" without additional
schema. The one open seam: every write through `Engine::transaction` enters
with `WriteAuthority::User` today (`redb_backend.rs:1534`), and the
privileged-entry-point that flips to `EnginePrivileged` is the engine-layer
concern. The storage-layer hook (`Transaction::durability_for_authority`)
is ready.

Schema-driven rendering may want a content-type-keyed index. Today
content-type lives in a Node label (`"Post"`, `"Comment"`) and is reachable
via `get_by_label`. A dedicated "content-type registry" index would be
new — see the application-layer-composition note in §7.

Plugins-as-subgraphs (CLAUDE.md baked-in #18) run through `store_subgraph` /
`load_subgraph_verified`; both are wired and the hash-first verification
posture is in place. Phase 4-Foundation's plugin manifest schema landed
above this crate (in `benten-platform-foundation::plugin_manifest`); its
storage-layer touchpoint is the `system:`-prefixed manifest Nodes which flow
through `put_node_with_context(privileged_for_engine_api())` — same path
already in production for `system:ModuleBytes` blobs via `RedbBlobBackend`.
Composition-cycle detection (per ratification of CLAUDE.md #18) walks a
structural `Subgraph` representation at install time without minting a new
`PrimitiveKind` variant. The plugin-library subgraph (`handler_id =
"plugin-library"`) extends the Phase-1 anchor + Version Node pattern over
existing `n:CID` / `e:CID` keys; no new prefix or schema change in this
crate.

CLAUDE.md baked-in #19 (engine-level Rust extensions linked at compile time)
has zero storage-layer touchpoint by construction — engine extensions plug
in above the `GraphBackend` trait surface, not below it.

## 9. Open questions / unresolved internals

Five TODOs in the source today, plus a few not-explicitly-TODO frictions
worth surfacing.

**Explicit TODOs (5 in-source; all carrying past Phase-3 close):**

- `lib.rs:796` — `WriteContext::with_authority` flips `is_privileged`
  on `EnginePrivileged` but the inverse direction (callers explicitly
  setting `is_privileged = true` separately from authority) can drift.
  The TODO text names Phase-3 as the resolution window; it remained
  live through `phase-3-close` and `phase-4-foundation-close` without
  driving a regression; the two axes are de facto co-set in every
  production caller today.
- `redb_backend.rs:266` — the in-transaction flag is per-
  `Arc<RedbBackend>`. Two distinct Arc handles opened on the same redb
  file do not coordinate at the Mutex level and fall through to redb's
  single-writer lock (which blocks rather than deadlocks — but it's a
  different observable shape). Mini-review g3-ce-7 proposed keying the
  flag on the canonical DB path via a process-wide static; carried.
- `redb_backend.rs:280` — `next_tx_id` is process-lifetime-only.
  Reopening the backend restarts the counter at 1. An IVM persistence layer
  that uses `tx_id` as a durable high-water-mark would observe a
  monotonicity violation across restart. Mini-review g3-ce-8 proposed
  persisting it into a dedicated redb table; carried.
- `transaction.rs:48, 725` — a permanently-broken subscriber drifts
  invisibly today. The `catch_unwind` keeps it from poisoning the commit
  thread but there is no dead-letter counter and no `tracing` dep on this
  crate, so repeated panics are unobservable to operators. The TODO text
  names "phase-3 — subscriber dead-letter counter" as the resolution
  window; carried past `phase-3-close`.
- `lib.rs:266` — `read_bytes_since_reset` is a no-op stub returning 0
  unconditionally because `get_node_label_only` is full-decode-then-project
  rather than a prefix-bounded read. A real prefix-bounded fast-path is
  tracked at phase-3-backlog §7.21; until that lands the test that pins
  `<= 128` bytes is also gated.

**Non-TODO frictions worth naming:**

- **`Group` durability collapses to `Immediate` at the redb mapping.** The
  enum is the engine-level posture surface (closes Compromise #12), the
  redb implementation does not honor it. Documented loudly + a one-shot
  stderr warning fires when an explicit `Group` request is observed. Until
  redb grows native batched-commit or Benten adds its own write-batching
  layer above redb, the Group vs. Immediate bench delta will be zero.
- **The TOCTOU window fix bypasses the bloom in the write path.** Inside
  `put_node_with_context`, the in-txn existence probe runs unconditionally
  against `NODES_TABLE` (`redb_backend.rs:1110-1113`), not against the
  bloom. The bloom remains useful for the non-transactional
  `probe_cid_exists` path. Worth knowing if perf profiling ever fingers
  the in-txn probe as hot.
- **The blanket `impl<T: KVBackend> NodeStore for T` is gone (g2-cr-1) but
  there is no compile-time mechanism stopping someone from re-adding it.**
  The module docstring on `store.rs:13-24` is the only defense. A
  per-backend explicit-opt-in posture is the contract; an audit catches
  drift.
- **`BlobError::CidMismatch.code()` returns
  `ErrorCode::Unknown("E_MODULE_BYTES_CID_MISMATCH")`** rather than a
  catalog-enum variant. Indicates the error catalog hasn't admitted the
  module-bytes-mismatch code yet; minor friction at the cross-language
  boundary.
- **`Transaction::transaction` ALWAYS rejects nested.** That's the Phase-1
  documented compromise. Anything that legitimately needs nested-txn
  semantics (savepoints, partial rollback inside a larger batch) has no
  path today.
- **`SystemZoneWrite` carries only the offending label, not the full Node /
  attempted authority.** Diagnostics surface "system-zone write not
  permitted from user path: {label}" but not "from which call site with
  what authority"; useful for debugging only via the chain.
