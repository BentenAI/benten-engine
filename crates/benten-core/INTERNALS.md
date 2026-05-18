# benten-core — Internals

A plain-English deep-dive into the foundational crate of the Benten Engine. Read this when you need to understand what `benten-core` owns, how it sits in the workspace, and what philosophical lines it draws around itself. Audience: contributors who already know Rust but are new to the crate.

Current as of HEAD `8141b94` (post `phase-4-foundation-close` tag).

---

## 1. What this crate does (in two paragraphs)

`benten-core` is the foundation of the Benten type system. It defines the four shapes everything else in the workspace agrees on — `Value` (the DAG-CBOR-compatible value type), `Node` (a labelled, propertied content-addressed graph node), `Edge` (a content-addressed directed edge between two Node CIDs), and `Cid` (a thin CIDv1 newtype). On top of those, it provides content-addressed hashing (BLAKE3 over DAG-CBOR with canonical key sort), three coexisting version-chain shapes (a thin `u64`-id anchor for the simple case, a Cid-head-threaded linear anchor that detects concurrent forks, and a DAG-shaped chain added in Phase-4-Foundation G24-D to support branches + merges per the plugin-library subgraph design), a Hybrid Logical Clock (HLC) for Phase-3 sync, a `Subgraph` + `SubgraphBuilder` pair that represents a handler-as-graph and produces a deterministic CID for it, and the `ChangeStream` port that `benten-eval`'s SUBSCRIBE primitive consumes.

The crate is intentionally `#![no_std]` (with `extern crate alloc`). It forbids `unsafe_code`, warns on `missing_docs`, and depends on no other Benten crate except `benten-errors` (the workspace error-catalog). It pulls in nothing storage-shaped, nothing evaluator-shaped, nothing capability-shaped, and nothing networking-shaped. Every other Benten crate either depends on `benten-core` or depends on a crate that does — making `benten-core` the bottom of the workspace dependency graph and the place where the hash contract lives single-sourced. This is Compromise #3 (zero-Benten-deps root of dep graph) operationalised.

---

## 2. Where it sits in the dependency chain

### Inputs (workspace)
- `benten-errors` — the only Benten crate `benten-core` depends on. Provides the stable `ErrorCode` enum that `CoreError::code()`, `VersionError::code()`, and `VersionDagError::code()` map into.

### Inputs (external)
- `blake3` (pure feature) — content hashing.
- `serde_ipld_dagcbor` — canonical DAG-CBOR encode/decode with length-first key sort.
- `serde` + `serde_bytes` — the data-model glue.
- `spin` — `no_std`-compatible `Mutex` + `Lazy`. Used by the `u64`-id anchor table, the Cid-head anchor's per-anchor chain, and the HLC's `last_emitted` cell.
- `thiserror` — ergonomic `Display`/`Error` derives for `CoreError`, `VersionError`, and `VersionDagError`.

Dev-deps (`proptest`, `criterion`, `serde_json`) are target-gated off `wasm32-*` so the wasm32-wasip1 example build doesn't drag in `wait-timeout` (which lacks a wasi backend).

### Workspace consumers (out)
Nearly every other crate. Direct `path = "../benten-core"` entries appear in:
- `benten-caps`, `benten-graph`, `benten-ivm`, `benten-eval`, `benten-engine`, `benten-dsl-compiler`, `benten-sync`, `benten-id`, and the Phase-4-Foundation `benten-platform-foundation` crate.

### External consumers
- `bindings/napi` (napi-rs v3) wraps `Cid`, `Node`, `Value` for TypeScript.
- The `cite-drift-detector` tool consumes `Cid` literals indirectly through the canonical fixture.

### Strict architectural rule (arch-1)
`benten-core` MUST NOT depend on `benten-eval`. Enforced two ways:
- Cargo-toml grep in `tests/benten_core_no_eval_dep.rs` (rejects even dev-dependencies).
- A CI workflow (`.github/workflows/arch-1-dep-break.yml`) running the same check.

This is not soft style — it is the invariant that lets the crate stay foundational. If you find yourself needing to import an evaluator type into `benten-core`, the answer is almost always to push the consumer up to `benten-eval` instead, or to define an abstract trait in `benten-core` that `benten-eval` implements.

---

## 3. Files inventory in `src/`

There are eight `.rs` files. Total source size is roughly ~3,750 lines.

### `lib.rs` (~948 LOC)
Crate root. Carries the module-level docs, the `WriteAuthority` enum (lifted into core during Phase-2a ucca-9 / arch-r1-2 so `benten-graph` and `benten-caps` can re-export the same type — `Copy` per R6 R2 C2-R2-6), the four multicodec/multihash constants (`CID_V1=0x01`, `MULTICODEC_DAG_CBOR=0x71`, `MULTIHASH_BLAKE3=0x1e`, `BLAKE3_DIGEST_LEN=32`, plus `CID_LEN=36`), the `Node` struct + its hash path (`to_canonical_bytes` / `cid` / `load_verified`), the `Cid` newtype + its parse paths (`from_bytes`, `<Cid as core::str::FromStr>::from_str`, `to_base32`), the rolled-by-hand base32-lower-nopad codec, the `CoreError` enum + its `code()` mapping, the `u64`-id `Anchor` surface (with its process-global `U64_CHAINS` table), the `LABEL_CURRENT` / `LABEL_NEXT_VERSION` constants, the `format_err` helper used to bridge `Display`-only errors into owned `String`s without triggering the workspace's `unwrap_used` / `expect_used` lints, and the `pub mod testing` module exposing `canonical_test_node` for cross-process determinism fixtures.

**Key invariants:** Node CID is a pure function of `(labels, properties)`; `anchor_id` is `#[serde(skip)]` and excluded by both the skip attribute and a dedicated `NodeHashView` projection (belt-and-suspenders). NaN/±Inf rejection is performed up-front by `Value::to_canonical` so failures surface as typed `CoreError::FloatNan` / `FloatNonFinite` rather than wrapped serde errors. `-0.0` is normalised to `+0.0` so the CID is stable across the sign of zero.

`WriteAuthority` (`lib.rs:103-120`) has three variants — `User` (default), `EnginePrivileged`, `SyncReplica { origin_peer: Cid }` — and is the single source-of-truth shape both `benten-graph` and `benten-caps` re-export.

### `value.rs` (~261 LOC)
The `Value` enum (eight variants: `Null`, `Bool`, `Int(i64)`, `Float(f64)`, `Text(String)`, `Bytes(Vec<u8>)`, `List(Vec<Value>)`, `Map(BTreeMap<String, Value>)`). Three convenience constructors (`text`, `unit`, `map_of`). `to_canonical` performs the float-validation + `-0.0`-normalisation walk before encode.

**Codec split** (the load-bearing design choice): `Serialize` is derived via `#[serde(untagged)]` and is safe at encode time because every variant writes a distinct CBOR major type. `Deserialize` is **hand-written** because `untagged` deserialisation collapses channels CBOR distinguishes — a small-integer array could round-trip as `Bytes` instead of `List`, and a text-string would land on `visit_str` regardless of which CBOR major type was on the wire. The `ValueVisitor` impl dispatches on the actual data-model type the decoder surfaces. Map keys are decoded as `String` to enforce DAG-CBOR's text-key restriction.

**Public exports:** `Value`.

### `edge.rs` (~133 LOC)
The `Edge` struct (`source: Cid`, `target: Cid`, `label: String`, `properties: Option<BTreeMap<String, Value>>`) + a private `EdgeHashView` serde view used for canonical bytes.

**Two non-obvious choices, both pinned by tests:**
- **Self-loops allowed.** `Edge::new(src, src, ...)` is valid; DAG-ness is enforced at the subgraph layer by `benten-eval`'s registration-time validator, not by `Edge::new`.
- **`None` vs empty-map properties are CID-distinct.** No `skip_serializing_if` on `properties` — the CBOR encoder emits `null` for `None` and `a0` for `Some(empty)`, a stable 1-byte difference in the hash input.

The endpoint Node CIDs are passed by value, never by reference, so building an Edge never disturbs the endpoint CIDs. Pinned by `tests/edge_does_not_change_endpoint_cids.rs`.

**Public exports:** `Edge`.

### `version.rs` (~189 LOC)
The Cid-head-threaded **linear** version-chain surface. Each `Anchor` owns an `Arc<spin::Mutex<Vec<(Cid, Cid)>>>` of `(prior_head, new_head)` pairs. Cloning an `Anchor` shares the chain; calling `Anchor::new` twice with the same head produces independent chains (this was the fix for a prior process-global-map design that leaked state between unrelated anchors).

`append_version(anchor, &prior_head, &new_head)` returns:
- `Ok(())` on a clean append.
- `VersionError::UnknownPrior { supplied }` if the caller named a head the anchor never observed.
- `VersionError::Branched { seen, attempted }` if a previous append already named that same prior — i.e. two writers raced on the same head.

`walk_versions` returns an `IntoIter<Cid>` from oldest to newest, starting with the root head.

**Public exports:** `version::Anchor`, `version::append_version`, `version::walk_versions`, `version::VersionError`.

### `version_chain.rs` (~327 LOC) — Phase-4-Foundation G24-D
The **DAG-shaped** version-chain surface, added in Phase-4-Foundation per CLAUDE.md baked-in #18 implementation refinement (D-4F-14): "Linear version-chain extended to support branches (forks). Anchor → v1 → {v2-mainline, v1.5-fork}; CURRENT can point at any branch tip." The linear `version.rs` shape stays for callers that don't need branches; `version_chain.rs` is what the plugin-library subgraph design depends on (one `version::<cid>` Read Node per installed CID, with fork-installs against a non-tip prior head retained in the library subgraph without the linear-chain extending past the fork point).

`DagVersionChain` is a forest of `(parent, child)` edges keyed by CID with a per-anchor "tips" set tracking branches that have no descendants. Storage shape: `parents[child]` + `children[parent]` (both `BTreeMap<Cid, BTreeSet<Cid>>`), `all` (every CID in the DAG), and `current` (the per-device-local CURRENT pointer per ratification #2 of CLAUDE.md #18).

Operations:
- `add_version(parent, child)` — link `parent → child`. Multiple calls with the same `parent` create branches; multiple parents for the same `child` create a merge node. Returns `VersionDagError::UnknownParent` if parent unseen, `VersionDagError::Cycle` if `child` is already an ancestor of `parent`.
- `tips()` — all leaf CIDs (branch tips).
- `descendants(from)` — BFS walk of everything reachable from `from`.
- `is_ancestor_of(candidate, target)` / `is_descendant_of(target, candidate)` — transitive ancestry check; used for upgrade DAG-monotonicity at the plugin-install path.
- `current()` / `set_current(cid)` — read/write the local active reference; `set_current` returns `VersionDagError::UnknownCurrent` if the CID is not in the DAG.

`VersionDagError` exposes `.code()` mapping into `ErrorCode::VersionUnknownPrior` (for `UnknownParent` + `UnknownCurrent`) and `ErrorCode::VersionBranched` (for `Cycle`). These reuse existing catalog codes rather than minting new ones, matching the linear `VersionError` mapping.

**Public exports:** `version_chain::DagVersionChain`, `version_chain::VersionDagError`.

### `hlc.rs` (~583 LOC, including tests)
Hybrid Logical Clock primitives. `BentenHlc` is the 20-byte value type `(physical_ms: u64, logical: u32, node_id: u64)` with lexicographic ordering. `Hlc` is the state machine: bound to a `node_id`, takes a `fn() -> u64` physical-clock callback (deliberately a bare fn pointer to keep the surface `no_std`-compatible and to allow injection in tests), and protects its `last_emitted` cell with a `spin::Mutex`.

`Hlc::now()` follows the Kulkarni-Demirbas rule: `l' = max(last.physical_ms, physical_clock())`; if `l'` did not advance past last, bump the logical counter (saturating at `u32::MAX`); otherwise reset logical to 0. `Hlc::update(remote)` returns `CoreError::HlcSkewExceeded` (which maps to `E_HLC_SKEW_EXCEEDED`) when the remote's physical_ms exceeds local physical clock + the configured skew tolerance (5 minutes default). Local state is NOT mutated when the skew error fires.

**Why a direct implementation, not `uhlc 0.2.1`:** the crate docs spell out the rejection in detail. Short form: `uhlc::HLC::new_timestamp` is `async fn`, holding an `async_std::sync::Mutex` across the await point. That would force `benten-core` to take `async-std` (which is `std`-only) and an executor dependency for what is fundamentally a `Mutex`-protected counter bump.

**Public exports:** `BentenHlc`, `Hlc`, `PhysicalClockFn`.

### `subgraph.rs` (~1,042 LOC)
The biggest file in the crate. Defines `PrimitiveKind` (the 12 operation primitives per CLAUDE.md #1), `OperationNode`, `NodeHandle`, `Subgraph`, and `SubgraphBuilder`. Phase-2b G12-C-cont relocated these from `benten-eval` so the `Subgraph` CID is single-sourced under the crate that owns the rest of content hashing.

`canonical_subgraph_bytes` is the authoritative encoding: nodes sorted by `(id, kind_tag)`, edges by `(from, to, label)`, plus the `handler_id` and `deterministic` fields. The encoding uses a dedicated `CanonView`/`CanonNodeRef`/`CanonEdgeRef` private projection that encodes `PrimitiveKind` as a stable string tag ("READ", "WRITE", ...) rather than the auto-derived enum discriminant.

**Intentionally NOT `Serialize` / `Deserialize`** for `Subgraph`, `OperationNode`, `NodeHandle`, `PrimitiveKind` (decision D5). A generic serde derive would invite a silent SECOND encoding that doesn't match `canonical_subgraph_bytes` — calling `serde_ipld_dagcbor::to_vec(&sg)` would produce bytes whose BLAKE3 differs from `sg.cid()`. Every encode path goes through `Subgraph::to_canonical_bytes` (the redundant `to_dag_cbor`/`to_dagcbor` aliases were deleted per #807/P-II + CLAUDE.md #5); every decode path through `Subgraph::load_verified` / `from_canonical_bytes` / `load_verified_with_cid`. A caller who tries the generic-serde shortcut gets a compile error pointing at the canonical entry points.

`SubgraphBuilder` is the ergonomic constructor: `read`, `write`, `transform`, `branch`, `iterate`, `call_handler`, `sandbox`, `respond`, `emit`, `wait_signal`, `wait_signal_with_timeout`, `wait_duration`, `call_with_isolated`, plus `push_primitive` as the lowest-level escape hatch. Every node it emits is stamped with `attribution: true` (the Inv-14 default); tests that want to probe the reject path construct `OperationNode`s directly.

`build_unvalidated_for_test` finalises without running the invariant pass. The validated version lives in `benten-eval::SubgraphBuilderExt::build_validated` (an extension trait), keeping the ~2,000 LOC invariants module out of `benten-core` while preserving the arch-1 dep direction.

The Phase-4-Foundation plugin composition-cycle detector (`benten_platform_foundation::plugin_manifest::detect_composition_cycle`) builds a structural `Subgraph` over manifest-CID Read Nodes + `COMPOSES`-labelled edges and walks it via DFS — no new `PrimitiveKind` variant minted, CLAUDE.md #1 12-primitive irreducibility preserved.

**Public exports:** `ATTRIBUTION_PROPERTY_KEY`, `NodeHandle`, `OperationNode`, `PrimitiveKind`, `Subgraph`, `SubgraphBuilder`, `canonical_subgraph_bytes`.

### `change_stream.rs` (~231 LOC)
The `ChangeStream` port that `benten-eval`'s SUBSCRIBE primitive consumes via dependency injection. Defines `SubscriberId` (a content-addressed `Cid` newtype), `ChangeKind` (`Created` / `Updated` / `Deleted`, `non_exhaustive` for Phase-3 `Replicated` / `Conflict` arms), and `ChangeEvent` (the full nine-field shape: `anchor_cid`, `kind`, `seq`, `payload_bytes`, `labels`, `tx_id`, `actor_cid`, `handler_cid`, `capability_grant_cid`).

The trait is object-safe with three methods (`subscribe`, `next_event`, `unsubscribe`) returning `Result<_, String>` so it carries no error-type dependency. The decision to put this port in `benten-core` rather than `benten-eval` is recorded inline: the change-event source is a backend concern and the port must sit at the stable arch-1 seam.

**Public exports:** `ChangeEvent`, `ChangeKind`, `ChangeStream`, `SubscriberId`.

---

## 4. Public API surface (plain English)

If you're a user of `benten-core`, you mostly touch these:

**Building values.** `Value::text(...)`, `Value::Bool(...)`, `Value::Int(...)`, `Value::map_of([...])`, `Value::unit()`. Floats need to be finite and non-NaN; if you violate either, the failure surfaces at hash time as `CoreError::FloatNan` / `FloatNonFinite`, not at construction. Map keys must be UTF-8 strings.

**Building a Node.** `Node::new(labels, properties)`. To get its CID, call `node.cid()` — that walks `to_canonical_bytes()` (which canonicalises the property tree, builds the `NodeHashView`, encodes to DAG-CBOR), then BLAKE3-hashes the bytes, then wraps the digest in a `Cid`. Optional `anchor_id: Option<u64>` is stored alongside but excluded from the CID.

**Reading a Node from bytes safely.** `Node::load_verified(&cid, &bytes)` hashes the bytes FIRST, before attempting decode. A tamper that happens to corrupt the CBOR structure would otherwise surface as a decode error and mask the integrity failure. Mismatches return `CoreError::ContentHashMismatch { path: "node", expected, actual }`. `Subgraph::load_verified_with_cid` follows the same pattern with `path: "subgraph"`.

**Building an Edge.** `Edge::new(src_cid, tgt_cid, "LABEL", props)`. CID is `edge.cid()`. Self-loops allowed.

**Constructing a CID.** Three doors. `Cid::from_blake3_digest([u8; 32])` for internal mint. `Cid::from_bytes(&[u8])` for napi-boundary 36-byte buffers; distinguishes structural failures (`InvalidCid`) from protocol-mismatch (`CidUnsupportedCodec`, `CidUnsupportedHash`). `<Cid as core::str::FromStr>::from_str(&str)` / `"bafyr4i...".parse::<Cid>()` for the multibase form; only accepts the `b` prefix (base32-lower-nopad). The prior shadowing inherent `Cid::from_str` was deleted (#840/P-II; the trait impl is the single parse). Render via `to_base32()` or `Display`.

**Version chains, three shapes.**
- **Thin `u64`-id (root crate).** `Anchor::new()` allocates a fresh monotonic id. `append_version(&anchor, &node)` returns the appended CID; `current_version(&anchor)` returns the latest; `walk_versions(&anchor)` returns the oldest-first `Vec<Cid>`. No fork detection. Backed by a process-global `U64_CHAINS` `BTreeMap`.
- **Cid-head-threaded linear (`version::*`).** `version::Anchor::new(root_cid)`. Each `append_version(&anchor, &prior_head, &new_head)` declares the prior head the caller observed; concurrent appends against the same prior fork into a typed `VersionError::Branched`. Per-anchor state, no global table.
- **DAG-shaped (`version_chain::*`).** `DagVersionChain::new(root_cid)`. Each `add_version(parent, child)` builds the parent/children edge map; multiple children per parent = branches; multiple parents per child = merge. `tips()` returns leaf CIDs; `current()` / `set_current(cid)` track the per-device-local active reference. Cycle detection runs on every `add_version`. This is the surface the Phase-4-Foundation plugin-library subgraph stores against.

**HLC.** `Hlc::new(node_id, fn() -> u64)` constructs a clock. `hlc.now()` returns a strictly-greater stamp each call. `hlc.update(&remote)` advances local and returns the post-update local stamp, or errors with `HlcSkewExceeded` if the remote is too far in the future.

**Subgraph.** `SubgraphBuilder::new("handler_id")`, then `.read(...)`, `.write(...)`, `.transform(prev, _)`, `.respond(prev)`, `.declare_deterministic(true)`, etc. Finalise with `build_unvalidated_for_test()` for tests, or use `benten_eval::SubgraphBuilderExt::build_validated()` to run the invariants pass. `sg.cid()` gives the content-addressed handler identity.

**Subscribe surface.** `SubscriberId::from_cid(cid)`, `ChangeEvent { anchor_cid, kind, seq, payload_bytes, labels, tx_id, actor_cid, handler_cid, capability_grant_cid }`. Implement `ChangeStream` to provide events to SUBSCRIBE.

**Errors.** `CoreError` is `#[non_exhaustive]` and ten-variant. Every variant has a stable catalog code via `CoreError::code() -> ErrorCode`. Sibling `VersionError` + `VersionDagError` expose `.code()` symmetrically.

---

## 5. Tests inventory

30 integration tests under `tests/` (plus a `fixtures/` directory and per-test `proptest-regressions/`). Grouped by surface:

### Value / Node hashing
- `value_variants.rs` — round-trip each of the eight `Value` variants through `Node::to_canonical_bytes`.
- `value_float.rs` — NaN rejection (three bit patterns), ±Inf rejection, `-0.0` normalisation, finite extremes (`f64::MIN`, `f64::MAX`).
- `node_cid.rs` — pins the canonical CID literal `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`. The TS-Rust round-trip mirror.
- `node_load_verified.rs` — `Node::load_verified` happy path + tamper-detection.
- `anchor_id_excluded_from_cid.rs` — every variant of `Option<u64>` produces the same CID for the same content.

### Cid binary + string codec
- `cid_from_str.rs` — base32 round-trip, prefix rejection (`B`, `z`, `f`, `m`), alphabet rejection.
- `cid_malformed.rs` — length / version / codec / hash / digest-length corruption.
- `cid_from_bytes_distinguishes_codec_and_hash_errors.rs` — the r6b error-arm split: `E_CID_PARSE` vs `E_CID_UNSUPPORTED_CODEC` vs `E_CID_UNSUPPORTED_HASH` are each reachable.

### Edge
- `edge_cid.rs` — `Edge::new` + `Edge::cid`.
- `edge_does_not_change_endpoint_cids.rs` — ENGINE-SPEC §7 boundary.
- `proptests_edge_roundtrip.rs` — `(source, target, label)` determines the Edge CID, proptest-shaped.

### Version chain
- `anchor_version.rs` — `u64`-id-Anchor happy path.
- `version_branched.rs` — Cid-head linear `VersionError::Branched` / `UnknownPrior` shapes.
- `version_error_codes.rs` — `.code()` mapping for both `VersionError` variants.
- `version_chain_label_constants.rs` — pins `LABEL_CURRENT == "CURRENT"` and `LABEL_NEXT_VERSION == "NEXT_VERSION"`.

(The DAG-shape surface `version_chain::DagVersionChain` is covered by inline `#[cfg(test)] mod tests` in `src/version_chain.rs` rather than a separate integration test — linear/branch/merge/cycle/unknown-parent/set-current-unknown coverage all there.)

### Subgraph
- `subgraph_deterministic_dagcbor.rs` — round-trip preserves the `deterministic` flag.
- `subgraph_load_verified_migration.rs` — corrupted bytes → typed error, not panic.
- `subgraph_serde_fail_loud_no_generic_serialize_impl.rs` — autoref-specialisation probe verifying `Subgraph` / `OperationNode` / `NodeHandle` / `PrimitiveKind` do NOT impl generic `serde::Serialize`/`Deserialize`. Cag-mr-g12c-cont-1 / D5.
- `benten_core_subgraph_canonical_bytes_match_eval_side_production_shape.rs` — the encoding shape pins (round-trip, order independence, node/edge/handler-id/deterministic sensitivity).
- `proptests_subgraph_order.rs` — node-level proptest pinning the lower-level `Node::cid` order-independence contract. (The subgraph-level sibling lives in `benten-eval`.)

### HLC
- `hlc_clock_skew_within_tolerance.rs` — `update(remote)` accepts in-tolerance.
- `hlc_clock_skew_exceeded_fires_e_hlc_skew_exceeded.rs` — out-of-tolerance fires the typed error + maps to `E_HLC_SKEW_EXCEEDED` + does NOT mutate local state.
- `prop_hlc_monotonic.rs` — 10,000-case proptest verifying `Hlc::now()` is strictly monotonic across adversarial physical-clock schedules (advances, stalls, rewinds).

### Cross-cutting
- `proptests.rs` — `prop_node_roundtrip_cid_stable`: random Node encode→decode→re-hash yields a byte-identical CID.
- `proptests_value_json_cbor.rs` — JSON↔Value fidelity for scalar variants (collections deferred to the DSL wrapper).
- `error_codes.rs` — every `CoreError` variant maps to its stable `ErrorCode`.
- `d2_cross_process.rs` — re-computes the canonical CID and asserts byte-for-byte equality against the committed `tests/fixtures/canonical_cid.txt`. Cross-architecture coverage is via the CI matrix, not subprocess-spawning.
- `benten_core_no_eval_dep.rs` — arch-1 invariant CI gate, reads `Cargo.toml` and rejects any `benten-eval` dep.

Each test file's docs explicitly name the R2 landscape row / closure pin (e.g. "R2 landscape §2.1 row 9"), making provenance traceable. The canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` remains stable across Phases 1 → 4-Foundation; any change to it is an immediately-flagging cross-phase break.

---

## 6. Benches inventory

Two informational Criterion benches under `benches/`, both `harness = false`:

- `hash_only.rs` — content-addresses `canonical_test_node` end-to-end. Baseline-protection bench, not gated against §14.6 (engine-level throughput). Pins the spike-era ~892ns measurement. If `blake3` or `serde_ipld_dagcbor` regresses upstream, this bench is where it shows up first.
- `cid_parse.rs` — `Cid::from_bytes` validator throughput. Sits on the napi-rs boundary path (TypeScript passes 36-byte buffers) and on any future sync-protocol path where CIDs arrive over the wire. Also informational; the parse cost is already amortised into ENGINE-SPEC §14.6's "Node lookup by ID: 1-50µs" gate.

Both are warmup-1s / measurement-3s aligned with the round-trip bench in `benten-engine` for cross-bench comparability.

`Cargo.toml` sets `[lib] bench = false` to disable the implicit libtest bench harness — otherwise `cargo bench --workspace` rejects Criterion's `--measurement-time` flag with "Unrecognized option."

There's one example: `examples/print_canonical_cid.rs` (~18 LOC) — prints the canonical Node's CID to stdout. Wired into the T6 wasm32-wasip1 determinism workflow as a "cross-target hash check."

---

## 7. Thin-engine + composable-graph philosophy check

`benten-core` is the foundational layer; expect strict philosophy adherence. Substantial respect for the principles, plus three observations worth flagging.

### Well-respected examples

- **No policy in types.** `Node`, `Edge`, `Value` are pure data shapes. No capability fields, no IVM annotations, no evaluator state, no transaction handles. `WriteAuthority` lives in core because it's an enum two downstream crates need to agree on, not because core uses it.
- **Hash contract is single-sourced.** All four content-addressed types (`Node`, `Edge`, `Subgraph`, `SubscriberId`) route through the same BLAKE3 + DAG-CBOR + multicodec/multihash header construction (CLAUDE.md baked-in #5). There is no second encoding lurking — the `subgraph_serde_fail_loud_no_generic_serialize_impl.rs` test proves it for `Subgraph`/`OperationNode`/`NodeHandle`/`PrimitiveKind`, and the `NodeHashView` / `EdgeHashView` private projections defend the same boundary for Node and Edge.
- **12 primitives respected.** `PrimitiveKind` is the canonical 12-variant enum (CLAUDE.md baked-in #1). The Phase-4-Foundation plugin composition-cycle detector mints structural `Subgraph`s over CID-keyed Read Nodes + `COMPOSES`-labelled edges — no 13th primitive, no new variant.
- **No IVM logic.** Search confirms zero references to `Strategy`, `View`, or `Algorithm B` in `src/`. The `ChangeStream` trait is a port — observation-only, abstract over which crate produces events. IVM-specific machinery is `benten-ivm`'s problem.
- **No evaluator concerns.** No `dispatch_call`, no `PrimitiveHost`, no transaction handles, no `Engine` struct. `Subgraph` and `SubgraphBuilder` describe shape; execution lives in `benten-eval`/`benten-engine`. The `SubgraphBuilderExt` extension-trait pattern keeps validation eval-side while keeping the shape core-side.
- **`benten-core` MUST NOT depend on `benten-eval`** is enforced both by a unit test (`benten_core_no_eval_dep.rs`) AND a workflow (`arch-1-dep-break.yml`). Belt-and-suspenders, deliberately.
- **`#[no_std]` + `forbid(unsafe_code)` + `warn(missing_docs)`** at the crate root. The crate is portable to `wasm32-unknown-unknown` (Class B thin-compute surface per CLAUDE.md #17) and the missing_docs lint is on so every public surface has docstring coverage.
- **HLC stays out of `uhlc`'s `async-std` mire.** The decision to roll HLC directly rather than take `uhlc 0.2.1` is documented in `hlc.rs` and explicitly cites the `no_std` + sync-surface motivation. The state machine is ~150 LOC, no external deps.
- **DAG-shaped version chain composes with the linear one.** `version_chain::DagVersionChain` doesn't replace `version::Anchor`; it sits alongside, sharing the `Cid` keying + the same catalog-code mapping. Phase-4-Foundation extended the version-chain surface without breaking the linear callers.

### Observations (not findings — context for future contributors)

- **Three coexisting Anchor shapes.** `u64`-id at `lib.rs`, Cid-head linear at `version.rs`, DAG-shaped at `version_chain.rs`. The plain-English contract is: `u64`-id for cheap simple cases (no fork detection), linear-Cid-head for the rejecting-fork case, DAG for the branches-allowed case. The R5-G7 "pick a canonical shape" carry has now been overtaken by events: Phase-4-Foundation needed the DAG shape, so consolidation can't mean "delete two." A future consolidation could collapse to "DAG only with an `is_linear()` convenience" but no concrete pressure to do that yet. Carried as a v1-gate / Phase 4-Meta assessment candidate — the residual `TODO(phase-3 — version surface consolidation)` markers in `lib.rs:666` and `version.rs:13` are still present at HEAD.
- **Process-global `U64_CHAINS` table for the `u64`-id `Anchor`.** This `static spin::Lazy<spin::Mutex<BTreeMap<u64, Vec<Cid>>>>` in `lib.rs:718` is acknowledged inline as a Phase-3-deferral: it grows unbounded for the life of the process and has no `drop_anchor` / GC. Fine for Phase 1 test runs; a long-running process would want a caller-owned `AnchorStore`. The Cid-head-threaded sibling in `version.rs` already moved to per-anchor `Arc<Mutex<...>>`, and `version_chain::DagVersionChain` is value-typed (no global table). The `TODO(phase-3 — anchorstore + GC)` marker in `lib.rs:712` and the `TODO(phase-3 — anchorstore + CRDT merge)` in `version.rs:26` are both still present.
- **`Subgraph` field-pub-ness vs accessor discipline.** `Subgraph::nodes`, `edges`, `handler_id`, `deterministic` are `pub`. The G12-C-cont docstring acknowledges this is a deliberate choice — `benten-eval`'s invariants module was reaching into the previous `pub(crate)` siblings and converting to accessors-everywhere would have cascaded across ~2,000 LOC. There ARE read-only accessor methods alongside (`nodes()` / `edges()` / `handler_id()` / `is_deterministic()`), so consumers have the option. Mutation discipline is enforced at registration time by Inv-13 (`benten-graph::immutability`), not by the type system. This is a legitimate trade-off; documenting that it's a trade-off is appropriate.
- **`SubgraphBuilder` knows about Inv-14.** `push()` stamps `attribution: true` on every emitted `OperationNode` by default. The constant `ATTRIBUTION_PROPERTY_KEY` is core-side because the eval-side builder previously needed the string. This is a soft boundary leak — Inv-14 is an evaluator concern that benten-core's builder defaults a property for. The inline justification (D12.7 Decision 1) is fair: the builder is the canonical attribution-stamp surface and tests bypass the builder when probing the reject path. Worth noting as a "core knows about one invariant by name, but only as a property key string."

These are observations, not violations. The crate is in a healthy posture for a foundational layer.

---

## 8. Phase-4-Foundation impact + Phase 4-Meta expectations

Phase-4-Foundation shipped at tag `phase-4-foundation-close` and added one new module (`version_chain.rs`) per CLAUDE.md baked-in #18 implementation refinement D-4F-14. The rest of the Foundation work landed downstream — plugin manifest types live in `benten-platform-foundation`, schema-driven rendering surfaces live in the new admin-shell + render-backend crates, and the Class B β `Engine::read_node_as` shipped at PR #184 entirely engine-side. The crate's role as the bottom of the workspace stayed intact across the phase.

Phase 4-Meta is the next architectural wave that may touch this crate:

- **Plugin manifest types — where they live.** Per CLAUDE.md #18 implementation refinements, every plugin ships a signed manifest with `requires` + `shares` halves. Foundation chose to keep these in `benten-platform-foundation` (closer to where install-time validation runs) rather than lifting to `benten-core`. If Phase 4-Meta needs the manifest types referenced from `benten-caps` for the capability-policy backend AND from `benten-platform-foundation` for install, the consolidation candidate is moving the value-shaped half (the immutable serialised manifest) into `benten-core` while keeping install-time validation in `benten-platform-foundation`. Open call; no concrete pressure yet.
- **`Engine::get_node` visibility tightening.** CLAUDE.md #18 names a v1-assessment-window question: `Engine::get_node` is currently `pub` (originally intended `pub(crate)` per the initial bake-in framing). This is an engine-side decision (not core-side), but it interacts with how plugins consume Node CIDs through the public API — the `Cid` type that flows through `read_node_as(principal, cid)` is owned by `benten-core`, so any visibility change ripples through the type surface here.
- **Phase-3 version-surface consolidation TODO.** Still carried; see §7 above. With three shapes coexisting, the "pick a canonical one" framing is obsolete; a more useful framing for Phase 4-Meta might be "do we need an `AnchorStore` trait that all three shapes implement, so callers can swap underlying state strategy without retyping?"
- **`AnchorStore` / anchor GC.** The `TODO(phase-3 — anchorstore + CRDT merge)` in `version.rs` and `TODO(phase-3 — anchorstore + GC)` in `lib.rs` both anticipate a caller-owned `AnchorStore` handle. Phase-3 `benten-sync` is now CRDT-merge-shaped (Loro integration shipped); the `AnchorStore` half of the comment still needs landing. v1-gate-window candidate per CLAUDE.md #15.

None of the above involves opening up a 13th primitive or relaxing the arch-1 dep direction. The crate's role as the bottom of the workspace stays intact.

---

## 9. Open questions / unresolved internals

Four explicit `TODO`-tagged carries in source — all still present at `phase-4-foundation-close`:

- `lib.rs:666` — `TODO(phase-3 — version surface consolidation)` on the `u64`-id Anchor block. Now reframed in scope by Phase-4-Foundation's addition of `version_chain.rs` (three shapes, not two).
- `lib.rs:712` — `TODO(phase-3 — anchorstore + GC)` on `U64_CHAINS` unbounded growth.
- `version.rs:13` — second instance of the version-surface-consolidation marker.
- `version.rs:26` — `TODO(phase-3 — anchorstore + CRDT merge)` on the per-anchor `Arc<Mutex<...>>` pattern.

No `FIXME` markers in source.

A few non-obvious design choices worth flagging for fresh readers:

- **`Cid` is byte-lexicographic-`Ord` with no semantic meaning.** Docstring explicitly says this. CIDs are sortable so they can be `BTreeMap` keys; the ordering is not causal, not version-ordered, not anything-meaningful. Anyone tempted to use `cid_a < cid_b` as a "happens-before" check is mistaken.
- **`format_err` exists to bridge `Display`-only errors into owned `String`s without using `expect` on infallible `write!` calls.** Workspace lints `clippy::unwrap_used` + `clippy::expect_used` are denied at the crate level, so the helper centralises the "writing to a String can't fail but the compiler doesn't know that" workaround at one site.
- **Per-test HLC mock-clock statics.** `hlc.rs` tests (and `tests/prop_hlc_monotonic.rs`, `tests/hlc_clock_skew_*`) each own their own module-level `static AtomicU64` + `fn` pointer pair. The reason is the `Hlc::new(node_id, fn() -> u64)` signature: it takes a bare fn pointer (not `impl Fn` closure or trait object), so each test's mock physical clock must live in a static. A shared `MOCK_TIME_MS` across tests caused a real flake under `cargo-llvm-cov` parallel execution (R6 R2 finding hlc-r6-r2-1/2); the per-test-statics shape is the structural fix and pim-N test-isolation-process-scoped-shared-state (§3.13) codified it.
- **`Cid::sample_for_test` / `Cid::sample_for_label`.** Gated behind `cfg(any(test, feature = "testing"))` so production builds cannot accidentally mint synthetic CIDs. The `testing` cargo feature exists exactly for downstream crates' integration tests; `benten-eval/Cargo.toml`'s `testing = ["benten-core/testing", "dep:wat"]` activates it.
- **The `NodeHashView` / `EdgeHashView` / `CanonView` private projection pattern.** These exist to make the hash-input contract explicit and protect against `#[serde(skip)]` regressions. Even if someone accidentally removed `#[serde(skip)]` from `Node::anchor_id`, the canonical-bytes path would still go through `NodeHashView` which only knows about `labels` and `properties`. Belt-and-suspenders.
- **`DagVersionChain::current` initialised to `Some(root_cid)`.** On construction the local CURRENT pointer is the root, not `None`. This matches the "anchor IS the first version" framing — a freshly-rooted plugin install has CURRENT = root, not "no version selected." Tests probing the unknown-CURRENT error path use a fresh CID that was never `add_version`'d in.
