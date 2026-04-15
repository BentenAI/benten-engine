# R2 Test Landscape — Phase 1

**Pipeline stage:** R2 (single-agent lead session — `benten-test-landscape-analyst`).
**Plan under test:** `.addl/phase-1/00-implementation-plan.md` (post-R1 triage addendum).
**R1 triage:** `.addl/phase-1/r1-triage.md` (61 findings dispositioned, 6 named compromises, 4 criticals fixed).
**Consumers:** 5 parallel R3 test-writer agents (`rust-test-writer-unit`, `rust-test-writer-edge-cases`, `rust-test-writer-security`, `rust-test-writer-performance`, `qa-expert`).
**Contract:** every row is mechanically verifiable at R3 write-time. No hand-waving.

---

## 1. Executive Summary

### Test-artifact totals

| Category | Count | Source of truth |
|---|---|---|
| Unit tests (Rust) | **201** | §2 coverage matrix per public API |
| Property-based tests (proptest) | **14** | §3 proptest harness |
| Integration tests (cross-crate) | **18** | §4 integration scenarios |
| Criterion benchmarks | **14** | §5 benchmarks (10 gates + 4 informational) |
| End-to-end (TS/Vitest) | **11** | §6 napi + DSL + scaffolder |
| Security-class tests | **22** | §7 security (11 R1-named + 11 TRANSFORM grammar + napi B8) |
| Cross-cutting CI harnesses | **9** | §8 CI (multi-arch, MSRV, WASM, drift, determinism, supply-chain × 4) |
| **Total test artifacts** | **289** | across all categories |

### R3 partition summary (file count ownership)

| R3 agent | Test files owned | Category mix |
|---|---|---|
| `rust-test-writer-unit` | **34** | Unit + proptest harness |
| `rust-test-writer-edge-cases` | **22** | Edge/error/boundary |
| `rust-test-writer-security` | **14** | Security-class + TRANSFORM rejection + napi input |
| `rust-test-writer-performance` | **10** | Criterion + perf CI gates |
| `qa-expert` (integration) | **13** | Cross-crate + Vitest end-to-end + 6 exit criteria |
| **Total files** | **93** | disjoint; no file owned by two agents |

### Estimated Rust test files per crate

| Crate | Unit | Edge | Security | Perf | Integration | Total |
|---|---|---|---|---|---|---|
| `benten-core` | 7 | 4 | 0 | 2 | 1 | 14 |
| `benten-graph` | 8 | 5 | 1 | 4 | 2 | 20 |
| `benten-ivm` | 6 | 3 | 0 | 2 | 2 | 13 |
| `benten-caps` | 4 | 2 | 3 | 0 | 1 | 10 |
| `benten-eval` | 7 | 5 | 8 | 1 | 3 | 24 |
| `benten-engine` | 2 | 3 | 2 | 1 | 3 | 11 |
| `bindings/napi` | 0 | 0 | 0 | 0 | 1 | 1 |
| **Total** | **34** | **22** | **14** | **10** | **13** | **93** |

Plus 9 CI harnesses in `.github/workflows/ci.yml` and 2 Vitest scaffolds (`bindings/napi/index.test.ts`, `packages/engine/src/*.test.ts`, `tools/create-benten-app/test/scaffolder.test.ts`).

---

## 2. Coverage Matrix by Crate

Columns: **Name · Test types required · Specific assertions · R3 owner · Dep · Error codes fired**.

### 2.1 `benten-core`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| `Value::Null / Bool / Int / Text / Bytes / List / Map` variants (C1) | Unit × 7 | Each variant encodes to expected DAG-CBOR major type; round-trips via `Node::from_cbor_bytes(node.to_cbor_bytes())` equals self | unit | — | — |
| `Value::Float(f64)` (C3, G1-A) | Unit × 5 | `Float(NaN)` → `Err(CoreError::FloatNan)` at serialization; `Float(+INF)` / `Float(-INF)` → `Err(FloatNonFinite)`; `Float(1.0)` encodes as shortest-form; `Float(0.0) == Float(-0.0)` in CID terms after canonicalization (DAG-CBOR normalizes); `Float` wrapped in `Value::Map` round-trips | unit | C1 | `E_VALUE_FLOAT_NAN`, `E_VALUE_FLOAT_NONFINITE` |
| `Value::Float` proptest (C3) | Proptest | `prop_value_float_bits_stable`: for 100k random `f64` bit patterns rejected-NaN-and-nonfinite filtered, `Value::Float(x)` encodes and decodes equal under DAG-CBOR with the same CID | unit | C3 | — |
| `Node::new(labels, properties)` (C1) | Unit × 3 | Empty-label construction accepted; ordering of labels preserved; duplicate label entries preserved (labels are a list not a set) | unit | C1 | — |
| `Node::cid()` deterministic (C1, C4) | Unit × 4 | Spike fixtures pass; `canonical_test_node().cid()? == "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda"`; two Nodes with structurally equal content produce byte-identical CIDs; Node with `anchor_id` set produces same CID as without (anchor excluded per §7) | unit | C1 | — |
| `Node::cid()` proptest | Proptest (100k) | `prop_node_roundtrip_cid_stable` — for random Nodes, `hash(decode(encode(n))).cid() == n.cid()` | unit | C1 | — |
| `Edge::new(source, target, label, properties)` (C2) | Unit × 4 | Construct with empty / non-empty properties; `source_cid != target_cid` accepted; self-loop accepted (invariant 1 is a subgraph-level check, not an edge-level check); `Edge::cid()` is stable across process boundaries | unit | C1, C2 | — |
| `Edge::cid()` (C2) | Unit × 2 | Edge with identical (source, target, label) and `properties = {}` and `properties = None` produce distinct CIDs (None vs empty map is preserved per DAG-CBOR); edges with different `source_cid` produce different CIDs | unit | C2 | — |
| `edge_creation_does_not_change_endpoint_node_cids` (C2, §7) | Unit | Create Node A, get `cid_a`; create Node B, get `cid_b`; create Edge(A→B); re-hash A → `cid_a` identical; re-hash B → `cid_b` identical | unit | C1, C2 | — |
| `Edge` proptest (C2) | Proptest | `prop_edge_roundtrip_cid_stable` — 100k instances | unit | C2 | — |
| `Cid::from_str / to_string` (C1) | Unit × 4 | Round-trip of fixture string; malformed multibase → `Err(CidParse)`; wrong multicodec (not 0x71) → `Err(UnsupportedCodec)`; wrong multihash code (not 0x1e) → `Err(UnsupportedHash)` | edge | C1 | `E_CID_PARSE`, `E_CID_UNSUPPORTED_CODEC`, `E_CID_UNSUPPORTED_HASH` |
| `Cid::from_bytes` (C1) | Unit × 3 | Length < `CID_LEN` → `Err`; length > `CID_LEN` → `Err`; version byte != 0x01 → `Err(UnsupportedVersion)` | edge | C1 | `E_CID_PARSE` |
| `Anchor` type + `CURRENT` / `NEXT_VERSION` labels (C6) | Unit × 6 | `Anchor::new()` creates stable id; `append_version(anchor, v1)` sets CURRENT → v1; `append_version(anchor, v2)` updates CURRENT → v2, creates NEXT_VERSION v1→v2; `current_version(anchor) == v2`; `walk_versions(anchor)` yields [v1, v2] in order; concurrent append creates branched chain surfaced as error `E_VERSION_BRANCHED` | unit | C1, C2 | `E_VERSION_BRANCHED` |
| `version_chain_linking_does_not_change_version_node_cids` (C6, §7) | Unit | Create Version Node v1 with initial CID `c1`; append to anchor (creates NEXT_VERSION edge v0→v1); re-hash v1 → `c1` identical | unit | C2, C6 | — |
| `walk_versions` (C6) | Proptest | `prop_version_chain_linearizable` — random sequence of appends yields a total order compatible with NEXT_VERSION DAG | unit | C6 | — |
| `ErrorCode` enum (C7) | Unit × 3 | Every variant has a stable string code; `ErrorCode::from_str("E_INV_CYCLE")` round-trips; `ErrorCode::Unknown(String)` fallback accepted for drift-detector support | unit | C7 | all |
| `CoreError::code()` (C7) | Unit | Every `CoreError` variant maps to a specific `ErrorCode` matching ERROR-CATALOG.md | unit | C7 | all core codes |
| `serde_bytes_fixed` helper | Unit × 2 | Round-trip `[u8; CID_LEN]` via serde; length-mismatch on deserialize → `Err` | unit | — | — |

**Non-testable (inspection-only):** `CID_V1`, `MULTICODEC_DAG_CBOR`, `MULTIHASH_BLAKE3`, `BLAKE3_DIGEST_LEN`, `CID_LEN` constants (private byte constants used by hashing path; covered transitively by Node::cid determinism tests).

### 2.2 `benten-graph`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| `KVBackend::get / put / delete / scan / put_batch` conformance (G1) | Unit × 5 | Runs against in-memory mock impl + RedbBackend; same suite both pass | unit | — | — |
| `KVBackend` associated `type Error` polymorphism (G1, G2-A) | Unit × 2 | `backend_error_polymorphism_mock` — custom backend with `type Error = MyErr` compiles + surfaces `MyErr` via `?`; redb backend surfaces `RedbError` | unit | G1 | — |
| `KVBackend::scan` iterator shape (G1, G2-A) | Unit × 3 | `scan_iterator_is_lazy` — consuming prefix of the iterator doesn't force full scan; `scan_iterator_empty_prefix` returns everything; `scan_iterator_prefix_bounds_range` stops at next-prefix | unit | G1 | — |
| `RedbBackend::open_existing` vs `open_or_create` (G2, G2-B) | Unit × 3 | `open_existing(missing_path)` → `Err(BackendNotFound)`; `open_or_create(missing_path)` creates; `open_existing(existing)` succeeds | edge | G2 | `E_BACKEND_NOT_FOUND` |
| `DurabilityMode::{Immediate, Group, Async}` (G2, G2-B) | Unit × 3 | Immediate: single write visible after commit; Group: batch of 100 writes returns < 500µs per write p95; Async: write returns before fsync completes | perf | G2 | — |
| `label_index` + `prop_index` (G5, G2-B) | Unit × 6 | Create Node labeled "Post" → `get_by_label("Post")` returns its CID; delete → no longer appears; update removes old property-value entry and inserts new; multiple Nodes same label returned in insertion order; O(log n) lookup time asserted via benchmark | unit | G2 | — |
| `transaction(closure)` atomicity (G3, G3-A) | Unit × 4 | All WRITEs inside closure commit atomically; panic in closure → all WRITEs rolled back (redb tx rolled back); `Err` returned from closure → rolled back; commit emits `ChangeEvent` for each write | unit | G3 | — |
| `failure_injection_rollback` (G3, G8) | Edge | Inject failure in 2nd WRITE of 3-write tx; verify 1st and 3rd are both rolled back via re-read after reopen | edge | G3 | — |
| `ChangeEvent` schema (G3-A, R1 attribution) | Unit × 3 | `ChangeEvent { cid, label, kind: Created|Updated|Deleted, tx_id, actor_cid: Option<Cid>, handler_cid: Option<Cid>, capability_grant_cid: Option<Cid> }` — serializes; NoAuth populates `actor_cid = Some(noauth:<uuid>)`; Update vs Create distinguished by prior-existence | unit | G3 | — |
| `ChangeSubscriber` trait (G7, R1 architect major #1) | Unit × 2 | `benten-graph` does NOT import tokio (verified by `cargo tree -p benten-graph | grep -v tokio`); trait shape accepts both tokio-broadcast and sync-callback impls | unit | G7 | — |
| `WriteContext { is_privileged: bool }` (N8, SC1) | Unit × 3 | Default construction → `is_privileged = false`; system-prefix label + `is_privileged = false` → `Err(E_SYSTEM_ZONE_WRITE)`; system-prefix label + `is_privileged = true` → commits | security | N8 | `E_SYSTEM_ZONE_WRITE` |
| `user_operation_cannot_write_system_labeled_node` (R1 SC1 named) | Security | User-path WRITE to label `"system:IVMView"` rejected; error code `E_SYSTEM_ZONE_WRITE`; engine-privileged path accepts same write | security | N8 | `E_SYSTEM_ZONE_WRITE` |
| `NodeStore` / `EdgeStore` blanket impls (G4, G2-A) | Unit × 4 | Blanket impl over arbitrary `KVBackend` — `put_node` / `get_node` / `put_edge` / `get_edge` / `edges_from` / `edges_to` all behave identically across in-memory mock and redb | unit | G4 | — |
| `multi_MB_roundtrip` (G8) | Perf | 1MB, 10MB, 100MB blobs round-trip through `put` → `get`; BLAKE3 integrity intact; observed time recorded per size | perf | G8 | — |
| `concurrent_reader_writer` (G8) | Perf | N=16 reader threads + 1 writer thread for 10s; no torn reads; no deadlock; redb MVCC snapshot semantics hold | perf | G8 | — |
| `MVCC snapshot isolation` (G6) | Unit × 2 | Reader A opens snapshot; writer B commits change; reader A still sees old value until they drop snapshot and re-open | unit | G6 | — |
| Doctests on `KVBackend` trait + `RedbBackend` (G9) | Unit (doc) | Every `# Examples` block in `/// ` doc comments compiles and runs green via `cargo test --doc -p benten-graph` | unit | G9 | — |

### 2.3 `benten-ivm`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| `View` shared trait (I1, R1 architect) | Unit | Trait defines `update(&mut self, event: &ChangeEvent) -> Result<(), IvmError>` + `read(&self, query) -> Result<ViewResult, IvmError>` + `rebuild(&mut self, backend: &dyn NodeStore)`; all 5 views implement it | unit | I1 | — |
| `ViewDefinition` type (I1) | Unit × 2 | Content-addressed (view def is a Node with label `system:IVMView`); round-trip serialization stable | unit | I1 | — |
| `Subscriber::route_change_event` (I2, G5-A) | Unit × 3 | `subscriber_routes_to_matching_views` — `ChangeEvent` with label "Post" routes only to views whose pattern includes "Post"; views with no pattern match not invoked; routing cost bounded | unit | I2 | — |
| View 1 Capability grants (I3) | Unit × 3 | Build from scratch equivalence; incremental matches rebuild; delete removes; write-read latency < 50µs | unit | I3 | — |
| View 2 Event handler dispatch (I4) | Unit × 3 | Same three patterns — scratch-vs-incremental, delete, perf | unit | I4 | — |
| View 3 Content listing (I5, exit-criterion) | Unit × 4 | `content_listing_paginated_all_returned`; `content_listing_incremental_update` (after 3 writes, list has all 3); `content_listing_sorted_by_createdAt`; `content_listing_delete_removes_from_view` | unit | I5 | — |
| View 3 determinism (I5) | Proptest | `prop_content_listing_incremental_equivalence` — after N random write/update/delete, view 3 state == rebuild-from-scratch | unit | I5 | — |
| View 4 Governance inheritance (I6) | Unit × 4 | Rebuild; depth cap 5 hops respected; depth > 5 → view marks stale with `E_IVM_DEPTH_LIMIT`; incremental matches scratch | edge | I6 | `E_IVM_VIEW_STALE` |
| View 5 Version CURRENT pointer (I7) | Unit × 3 | After append, current resolved O(1); after concurrent branch, resolution picks higher-HLC; stale is set if anchor→current chain broken | unit | I7 | — |
| `stale_on_budget_exceeded` (I8) | Edge × 5 | Each of 5 views: inject a high-cost event; view is marked stale; read returns `Err(E_IVM_VIEW_STALE)`; `engine.read_view_allow_stale` returns last-known-good | edge | I8 | `E_IVM_VIEW_STALE` |
| View write-read latency bound (I3-I7) | Perf × 5 | One benchmark per view: write N events, measure time-to-visible in read; p95 < 50µs | perf | I3-I7 | — |

### 2.4 `benten-caps`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| `CapabilityPolicy` trait shape (P1, G4-A) | Unit × 2 | `fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError>` signature; trait object safe (`Box<dyn CapabilityPolicy>` compiles) | unit | P1 | — |
| `NoAuthBackend::check_write` (P3, G4-A) | Unit × 3 | `noauth_permits_everything` — returns `Ok(())` for arbitrary write ctx (fuzzed with proptest across 1000 inputs); zero allocations hot path; populates `actor_cid = Some(noauth:<uuid>)` | unit | P3 | — |
| `UCANBackend` stub (P4, G4-A) | Unit × 4 | `ucan_stub_errors_cleanly` — returns `Err(CapError::NotImplemented)`; error code is `E_CAP_NOT_IMPLEMENTED` distinct from `E_CAP_DENIED`; message names Phase 3 alternative (`ucan_stub_error_message_names_phase_and_alternative`); route in evaluator goes to `ON_ERROR` not `ON_DENIED` (`ucan_stub_error_routes_to_ON_ERROR_not_ON_DENIED`) | security | P4 | `E_CAP_NOT_IMPLEMENTED` |
| `check_write_called_at_commit` (P1, P5) | Unit | Hook called at commit boundary, not per individual WRITE primitive; verified with a counting mock policy | unit | P1 | — |
| `cap_error_codes_match_catalog` (P5, C7) | Unit | Every `CapError` variant maps to the right ERROR-CATALOG code: `NotImplemented` → `E_CAP_NOT_IMPLEMENTED`, `Denied` → `E_CAP_DENIED`, `DeniedRead` → `E_CAP_DENIED_READ`, `Revoked` → `E_CAP_REVOKED_MID_EVAL`, `Attenuation` → `E_CAP_ATTENUATION` | unit | P5 | all `E_CAP_*` |
| `CapabilityGrant` Node (P2) | Unit × 2 | Type constructor emits a Node with label `CapabilityGrant`; GRANTED_TO edge has the right source/target semantics | unit | P2 | — |
| `grant_uniqueness_on_cid` (P2) | Edge | Two GrantedTo edges with same source/target/scope produce different CIDs (due to HLC timestamps being properties) — tests anti-dedupe semantic | edge | P2 | — |
| `capability_revoked_mid_iteration_denies_subsequent_batches` (R1 named) | Security | Start a 300-iter ITERATE with 100-iter batch boundaries; revoke capability after batch 1 (iter 150); iter 200+ → `E_CAP_REVOKED_MID_EVAL`; documented 100-iter TOCTOU window | security | P1 | `E_CAP_REVOKED_MID_EVAL` |

### 2.5 `benten-eval`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| 12 primitive types defined (E1, G6-A) | Unit × 12 | Each of READ / WRITE / TRANSFORM / BRANCH / ITERATE / WAIT / CALL / RESPOND / EMIT / SANDBOX / SUBSCRIBE / STREAM has a Node constructor; determinism classification; error-edge set | unit | E1 | — |
| `phase_two_primitives_pass_structural_validation` (E5, R1) | Unit × 4 | Subgraphs containing WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX register successfully (no `E_INV_*`); executor at call-time returns `E_PRIMITIVE_NOT_IMPLEMENTED` for each | unit | E1, E5 | `E_PRIMITIVE_NOT_IMPLEMENTED` |
| READ primitive happy paths (E3, G6-A) | Unit × 4 | `read_by_id_found` returns value; `read_by_id_missing` routes to `ON_NOT_FOUND`; `read_by_query_empty_result` routes to `ON_EMPTY`; `read_with_cap_denied` routes to `ON_DENIED` with `E_CAP_DENIED_READ` | unit | E3 | `E_CAP_DENIED_READ` |
| READ capability denial (P5) | Security | `read_denied_returns_cap_denied_read` — option A existence-leak semantics. Documents phase-1 compromise. | security | E3, P5 | `E_CAP_DENIED_READ` |
| WRITE primitive (E3, G6-A) | Unit × 5 | Create / update / delete / conditional-CAS success; CAS with wrong version → `ON_CONFLICT` with `E_WRITE_CONFLICT`; cap denied → `ON_DENIED` with `E_CAP_DENIED` | unit | E3 | `E_WRITE_CONFLICT`, `E_CAP_DENIED` |
| TRANSFORM primitive — expression eval (E4, G6-B) | Unit × 20 | One test per built-in: `Math.min/max/round/abs`, string `lowercase/uppercase/truncate/substring/startsWith/endsWith`, array `map/filter/reduce/find/length/slice`, `Date.now/formatDate`, object construction, arithmetic, comparison, logical, ternary, property access | unit | E4 | — |
| TRANSFORM expression determinism (E4) | Proptest | `prop_transform_expression_deterministic` — random-valid expressions produce identical output across 1000 runs; no wall-clock or RNG leakage | unit | E4 | — |
| TRANSFORM grammar rejection (T12, R1 security) — **class of 15** | Security × 15 | Each forbidden construct rejected at registration with `E_TRANSFORM_SYNTAX`: closures (`() => x`), `this`, imports, prototype access (`__proto__`, `constructor`, `prototype`), tagged templates, template literals with expressions, optional-chained method calls, computed property names resolving to `__proto__`/`constructor`/`Symbol.*`, `new` in any position, `with`, destructuring with getters, spread-into-call, comma operator, `eval`, `Function` constructor | security | E4, T12 | `E_TRANSFORM_SYNTAX` |
| TRANSFORM parser fuzz (T12, R1 security) | Security | `fuzz_transform_parser` (ignored-by-default harness) — 10k random JS-like snippets; accepted strings must parse to an AST that only uses allowlisted nodes | security | E4, T12 | `E_TRANSFORM_SYNTAX` |
| BRANCH primitive (E3, G6-B) | Unit × 3 | Binary branch routes correctly; multi-way branch picks matching case; no-case-matches routes to `ON_DEFAULT` (fall-through) | unit | E3 | — |
| ITERATE primitive (E3, G6-B) | Unit × 5 | `iterate_max_required` — no `max` property at registration → `E_INV_ITERATE_MAX_MISSING`; iterations bounded; `$item` / `$index` / `$results` bindings; parallel flag runs concurrently; budget exceeded routes `ON_LIMIT` | unit | E3 | `E_INV_ITERATE_MAX_MISSING` |
| `iterate_nest_depth_bound` (E5, R1 philosophy named compromise) | Edge | Subgraph with 4 nested ITERATE → registration rejected with `E_INV_ITERATE_NEST_DEPTH`; 3-deep nesting accepted; test comment documents the Phase-1 stopgap | edge | E5 | `E_INV_ITERATE_NEST_DEPTH` |
| CALL primitive (E3, G6-B, R1 architect) | Unit × 4 | `call_handler` executes child subgraph; `call_attenuation` — capability context is attenuated when `isolated: true`; `call_timeout` — mandatory timeout enforced; `call_isolated_true_preserves_transaction_state` (R1 architect disposition: CALL enters nested tx scope; `isolated: true` attenuates caps but inherits tx) | unit | E3 | — |
| RESPOND primitive (E3) | Unit × 2 | `respond_terminal` — evaluator halts after RESPOND; response bytes available to caller | unit | E3 | — |
| EMIT primitive (E3) | Unit × 2 | `emit_fire_and_forget` — emit doesn't block evaluator; subscribers receive message | unit | E3 | — |
| Evaluator stack model (E2, G6-C) | Unit × 5 | `evaluator_pushes_next_on_ok`; `evaluator_pops_on_respond`; `evaluator_follows_error_edge`; `evaluator_preserves_frame_order`; `evaluator_stack_overflow_is_err_not_panic` (explicit stack, no recursion — R1 architect major #2 disposition) | unit | E2 | `E_INV_DEPTH_EXCEEDED` |
| Evaluator frame model (E2, R1 option A) | Unit × 2 | `Vec<ExecutionFrame>` + `frame_index: usize` — no self-ref borrows; frame outlives instruction processing; mutation-safe | unit | E2 | — |
| Invariant 1 — DAG / cycle detection (E5) | Unit × 2 | Registration of a subgraph with a back-edge → `E_INV_CYCLE`; DAG accepted (positive case) | unit | E5 | `E_INV_CYCLE` |
| Invariant 2 — Max depth (E5) | Unit × 2 | Subgraph with depth > configured max → `E_INV_DEPTH_EXCEEDED`; at-max accepted | unit | E5 | `E_INV_DEPTH_EXCEEDED` |
| Invariant 3 — Max fan-out (E5) | Unit × 2 | Node with fan-out > configured max → `E_INV_FANOUT_EXCEEDED`; at-max accepted | unit | E5 | `E_INV_FANOUT_EXCEEDED` |
| Invariant 5 — Max total nodes (E5) | Unit × 2 | Subgraph with 4097 nodes → `E_INV_TOO_MANY_NODES`; 4096 accepted | unit | E5 | `E_INV_TOO_MANY_NODES` |
| Invariant 6 — Max total edges (E5) | Unit × 2 | Subgraph with 8193 edges → `E_INV_TOO_MANY_EDGES`; 8192 accepted | unit | E5 | `E_INV_TOO_MANY_EDGES` |
| Invariant 9 — Determinism classification (E5) | Unit × 2 | Non-deterministic primitive (EMIT audit) in deterministic context → `E_INV_DETERMINISM`; well-classified accepted | unit | E5 | `E_INV_DETERMINISM` |
| Invariant 10 — Content hash (E5) | Unit × 2 | Registered subgraph hash matches computed; mutation → `E_INV_CONTENT_HASH` | unit | E5 | `E_INV_CONTENT_HASH` |
| Invariant 12 — Registration-time validation (E5) | Unit × 2 | Registration-time catch-all for invariants firing with multiple violations returned; `violated_invariants: [1, 3]` populated | unit | E5 | `E_INV_REGISTRATION` |
| `requires` property handling (P6, G6-A, R1 SC4) | Security × 3 | `handler_with_understated_requires_denies_excess_writes` — handler declares `requires: "post:read"` but internally WRITEs admin:* → WRITE denied individually; `handler_cannot_escalate_via_call_attenuation` — sub-CALL can't exceed parent's effective caps; `requires_checked_at_primitive_not_just_declaration` | security | P6 | `E_CAP_DENIED` |
| Transaction primitive (E6, G7) | Unit × 4 | `engine.transaction(|tx| …)` commits all WRITEs atomically; closure-panic → rollback + re-raise; nested tx → `E_NESTED_TRANSACTION_NOT_SUPPORTED`; commit-time cap failure → aborted trace + `E_CAP_DENIED` | unit | E6 | `E_NESTED_TRANSACTION_NOT_SUPPORTED`, `E_TX_ABORTED`, `E_CAP_DENIED` |
| `engine.trace()` (E8) | Unit × 3 | `trace_returns_steps` — every step has `duration_us > 0`; `trace_topo_order` — step sequence in topological order (non-strict because of BRANCH/ITERATE); `trace_includes_attribution` — `actor_cid`, `capability_grant_cid_used` present | unit | E8 | — |
| `subgraph.toMermaid()` (E7) | Unit × 3 | Returns valid Mermaid `flowchart` syntax (minimal grammar check); edge labels present; node labels present | unit | E7 | — |
| `diag` feature gating (E7/E8) | Unit × 1 | `cargo test -p benten-eval --no-default-features` — toMermaid / trace modules absent from thin build | unit | E7 | — |
| Phase 2 primitives return error at call time (E9) | Unit × 4 | WAIT execution → `E_PRIMITIVE_NOT_IMPLEMENTED`; STREAM execution → same; SUBSCRIBE as user op → same; SANDBOX → same | unit | E9 | `E_PRIMITIVE_NOT_IMPLEMENTED` |
| TRANSFORM grammar BNF published (T12) | Unit (doc) | `docs/TRANSFORM-GRAMMAR.md` exists; it compiles via BNF parser self-check in a `docs-check` CI job | unit | T12 | — |

### 2.6 `benten-engine`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| `Engine::open / create_node / get_node` (N1 — keep) | Unit × 2 | Spike tests survive (`create_then_get_roundtrip`, `missing_cid_returns_none`); already landed | unit | N1 | — |
| `Engine::create_edge / get_edge / edges_from / edges_to / delete_node / update_node` (N2) | Unit × 6 | One test per API; full CRUD closed | unit | N2 | — |
| `Engine::register_subgraph` (N3) | Unit × 3 | Registration runs invariants, stores content-addressed subgraph; re-registration is a no-op (same CID); different subgraph with same ID → `E_INV_REGISTRATION` | unit | N3 | `E_INV_REGISTRATION` |
| `Engine::call` (N3) | Unit × 2 | `call_handler_end_to_end` — register crud('post'), call 'post:create', assert created Node exists; `engine.call blocks on IVM barrier by default` | unit | N3 | — |
| `Engine::call_async` (N3, R1 dx) | Unit | `call_async` opt-out doesn't block on IVM barrier; exit criterion default still uses synchronous | unit | N3 | — |
| `EngineBuilder::{without_ivm, without_caps, without_versioning}` (N4, N5, R1 philosophy) | Unit | `thinness_no_ivm_no_caps_no_versioning_still_works` — all three opted out, engine is a pure content-addressed graph DB; CRUD succeeds | integration | N4, N5 | — |
| `Engine::builder().production()` (N5-b, SC2) | Security × 2 | `engine_builder_production_refuses_noauth` — returns `Err(NoCapabilityPolicyConfigured)` if no policy set; accepts any explicit non-NoAuth policy; startup emits info-log on NoAuth use | security | N5-b | — |
| `Engine::grant_capability / create_view / revoke_capability` (N7) | Unit × 3 | `grant_capability_only_via_engine_api` — user-path WRITEs to `system:` labels denied; engine API writes privileged=true; capability revocation surfaces in change stream | unit | N7 | `E_SYSTEM_ZONE_WRITE` |
| `Engine::snapshot / transaction` (N6) | Unit × 2 | Snapshot is pinned read; transaction delegates to G3 | unit | N6 | — |
| `Engine::read_view / read_view_allow_stale` (R1 dx disposition) | Unit × 2 | Stale view returns `E_IVM_VIEW_STALE`; `_allow_stale` returns last-known-good | integration | N3 | `E_IVM_VIEW_STALE` |
| Subgraph CID order-independence (R1 philosophy) | Proptest | `prop_subgraph_cid_order_independent` — equal subgraphs constructed via different insertion orders produce identical CIDs (Nodes sorted by CID, Edges by (source_cid, target_cid, label)) | unit | N3 | — |

### 2.7 `bindings/napi`

| Name | Test types | Specific assertions | R3 owner | Dep | Codes fired |
|---|---|---|---|---|---|
| `initEngine / createNode / getNode / updateNode / deleteNode` (B3) | End-to-end | Each binding has TS → Rust → TS round-trip test | integration | B3 | — |
| `createEdge / getEdge / edgesFrom / edgesTo / deleteEdge` (B3) | End-to-end | Each binding has round-trip test | integration | B3 | — |
| `registerSubgraph / callHandler / traceHandler` (B4) | End-to-end | Register + call + trace works; serialized subgraph round-trips | integration | B4 | — |
| `readView` (B5) | End-to-end | View 3 read returns expected posts in sorted order | integration | B5 | — |
| napi input validation (B8, R1 security) — **class of 4** | Security × 4 | `napi_rejects_oversized_value_map` (>10K keys → `E_INPUT_LIMIT`); `napi_rejects_deep_nested_value` (>128 depth); `napi_rejects_oversized_bytes` (>16MB); `napi_rejects_malformed_cid` (invalid multibase) | security | B8 | `E_INPUT_LIMIT` |
| Error-catalog TS types match runtime codes (B7) | Integration | `packages/engine/src/errors.ts` generated types have parity with runtime Rust enum; drift detector green | integration | B7 | all |
| `@benten/engine` DSL (B6) | Integration × 12 | One test per DSL function (`subgraph`, `crud`, `read`, `write`, `transform`, `branch`, `iterate`, `call`, `respond`, `emit`, `toMermaid`, `trace`) | integration | B6 | — |
| `crud('post')` zero-config injection (B6, R1 dx) | Integration × 3 | `crud_post_zero_config_injects_createdAt_deterministically`; `authorId` NOT auto-injected; `crud('post').list` returns in `createdAt` order | integration | B6 | — |

---

## 3. Property-Based Tests (proptest)

| Name | Invariant | Min instances | Shrink target |
|---|---|---|---|
| `prop_node_roundtrip_cid_stable` | Hash → decode → re-hash produces byte-identical CID | 100k | Minimal Node (1 label + 1 property) that fails round-trip |
| `prop_edge_roundtrip_cid_stable` | Edge round-trip CID stable | 100k | Minimal Edge (empty props) |
| `prop_value_float_bits_stable` | `Float(x)` for every f64 bit pattern (excluding NaN/Inf) is DAG-CBOR determinism-safe | 100k | Minimal f64 producing drift |
| `prop_value_json_cbor_conversion` | JSON ↔ Value ↔ DAG-CBOR round-trips losslessly (except float-integers per spec) | 10k | Minimal Value with type confusion |
| `prop_version_chain_linearizable` | `walk_versions` linear-extends NEXT_VERSION DAG | 1k | Minimal branching sequence |
| `prop_kvbackend_put_get_delete` | KVBackend laws across mock and redb | 10k | Minimal failing key/value pair |
| `prop_content_listing_incremental_equivalence` | View 3 state = rebuild from scratch | 5k | Minimal sequence of write/update/delete |
| `prop_transform_expression_deterministic` | TRANSFORM outputs depend only on inputs | 10k | Minimal expression + input pair causing drift |
| `prop_subgraph_cid_order_independent` | Subgraph canonicalization is insertion-order invariant | 10k | Minimal subgraph whose CID drifts by order |
| `prop_transform_grammar_fuzz_accepted_deterministic` | Fuzz-accepted inputs re-evaluate deterministically | 10k | Minimal accepted-and-nondet expression |
| `prop_capability_check_deterministic` | Same write context → same policy decision | 1k | Minimal context pair producing divergence |
| `prop_hlc_monotonic` | HLC timestamps are monotonic within a process | 10k | Minimal sequence causing rollback |
| `prop_node_anchor_id_excluded_from_cid` | Setting/unsetting anchor_id never changes the Node CID | 10k | Minimal anchor_id assignment that drifts |
| `prop_noauth_returns_ok_unconditionally` | NoAuthBackend permits all | 1k | N/A (expected all-pass) |

---

## 4. Integration Tests (Cross-Crate)

| Name | Scenario | Files touched | R3 owner | Codes fired |
|---|---|---|---|---|
| `capability_gated_crud_roundtrip` | Register crud('post'); grant WRITE; create → OK; revoke; create → `E_CAP_DENIED` via `ON_DENIED`. Exit-criterion #3. | `tests/integration/caps_crud.rs` | integration | `E_CAP_DENIED` |
| `ivm_write_propagation` | Write 10 posts; list via View 3; pagination correct; incremental never trails by > 1 write. Exit-criterion #2. | `tests/integration/ivm_propagation.rs` | integration | — |
| `transaction_atomicity_end_to_end` | Subgraph with 2 WRITEs; inject failure in 2nd; assert 1st rolled back via re-read | `tests/integration/tx_atomicity.rs` | integration | `E_TX_ABORTED` |
| `version_chain_current_resolution` | Append 5 versions to anchor; View 5 CURRENT is O(1) at every step | `tests/integration/version_current.rs` | integration | — |
| `content_hash_integrity_stack` | createNode TS → redb → getNode TS → re-hash Rust → assert equal to original CID. Exit-criterion #6. | `bindings/napi/index.test.ts` | integration | — |
| `exit_criterion_registration_succeeds` (#1) | `engine.register_subgraph(crud('post'))` returns handler id; all enforced invariants pass | `tests/integration/exit_criterion_1_registration.rs` | integration | — |
| `exit_criterion_three_creates_list_returns_them` (#2) | 3x `post:create` + `post:list` returns all 3 in `createdAt` order | `tests/integration/exit_criterion_2_list.rs` | integration | — |
| `exit_criterion_cap_denial_routes_on_denied` (#3) | Revoke write cap; `post:create` returns via `ON_DENIED` with `E_CAP_DENIED` | `tests/integration/exit_criterion_3_denial.rs` | integration | `E_CAP_DENIED` |
| `exit_criterion_trace_non_zero_timing` (#4) | `engine.trace(handlerId, input)` — every step has `durationUs > 0`; topo-order holds | `tests/integration/exit_criterion_4_trace.rs` | integration | — |
| `exit_criterion_mermaid_output_parses` (#5) | `handler.toMermaid()` returns valid Mermaid flowchart | `tests/integration/exit_criterion_5_mermaid.rs` | integration | — |
| `exit_criterion_ts_rust_cid_roundtrip` (#6) | TS creates canonical node, returns CID equal to fixture | `bindings/napi/index.test.ts` | integration | — |
| `six_named_compromises_regression` | One sub-test per named compromise asserting scope. See §2.8. | `tests/integration/compromises_regression.rs` | integration | `E_CAP_REVOKED_MID_EVAL`, `E_CAP_DENIED_READ`, `E_INV_ITERATE_NEST_DEPTH` |
| `cross_process_cid_determinism_with_graph` | Write in process A, read in process B, re-hash, assert equal CID (extends spike D2 to graph layer) | `tests/integration/cross_process_graph.rs` | integration | — |
| `change_stream_routes_to_ivm_subscriber` | Write triggers ChangeEvent; IVM subscriber receives; View 3 updates | `tests/integration/change_stream.rs` | integration | — |
| `handler_cannot_write_system_zone_via_normal_api` | Full-stack assertion of N8 | `tests/integration/system_zone.rs` | integration | `E_SYSTEM_ZONE_WRITE` |
| `nested_transaction_rejected` | `transaction(|tx| tx.transaction(|inner| …))` → `E_NESTED_TRANSACTION_NOT_SUPPORTED` | `tests/integration/nested_tx.rs` | integration | `E_NESTED_TRANSACTION_NOT_SUPPORTED` |
| `stale_view_returns_error_not_stale_data` | Force View 3 stale; default `read_view` → `E_IVM_VIEW_STALE`; `read_view_allow_stale` returns last-good | `tests/integration/stale_view.rs` | integration | `E_IVM_VIEW_STALE` |
| `capability_revoked_mid_eval_surfaces_at_batch_boundary` | 300-iter handler, revoke after batch 1; iter 200 → `E_CAP_REVOKED_MID_EVAL` | `tests/integration/cap_toctou.rs` | integration | `E_CAP_REVOKED_MID_EVAL` |

### §2.8 Named-compromise regression coverage (subtests in `compromises_regression.rs`)

1. `compromise_1_toctou_window_bound_at_100_iter_batch` — revoke at iter 150; iter 200+ denied; iter 149 not yet denied (bounds the window).
2. `compromise_2_ecapdenied_read_leaks_existence` — option A documented; error message is about the capability, not existence.
3. `compromise_3_error_code_enum_in_benten_core` — asserts `ErrorCode` is imported from `benten_core`, not a separate crate (regression marker for Phase 2 extract decision).
4. `compromise_4_wasm_runtime_only_compile_check` — asserts CI has no WASM runtime test for `bindings/napi` yet (T8 compile-check only); runtime with network-fetch backend is Phase 2.
5. `compromise_5_no_write_rate_limits_but_metric_recorded` — write N times fast; no rate-limiting error; `benten.ivm.view_stale_count{view_id}` metric non-empty.
6. `compromise_6_blake3_collision_resistance_note_in_security_posture` — asserts `docs/SECURITY-POSTURE.md` contains the BLAKE3 128-bit note.

Each subtest has a comment `// Phase 1 compromise; remove when Phase X implements Y` so removal at the phase boundary is a grep-and-delete exercise.

---

## 5. Criterion Benchmarks (Performance)

Sources: **(§14.6 direct)** literal, **(§14.6 derived)** decomposed, **(non-§14.6)** informational.

| Bench | Target | Source | Gate? | R3 owner |
|---|---|---|---|---|
| `hash_only` | No regression from spike 892ns (±10%) | non-§14.6 (baseline protection) | Warning on >10% regress | perf |
| `get_node` | 1-50µs (spike 2.71µs) | §14.6 direct | Fail on > 50µs p95 | perf |
| `create_node_immediate` | 100-500µs (spike 4ms — must drop with DurabilityMode::Group) | §14.6 direct | Fail on > 10ms p95 | perf |
| `create_node_group_commit` | < 500µs p95 | §14.6 derived | Fail on > 500µs p95 | perf |
| `10_node_handler_eval` | 150-300µs (mixed handlers with 2 WRITEs + IVM) | §14.6 direct | Fail on > 300µs p95 | perf |
| `view_read_content_listing` | < 1µs p95 | §14.6 direct | Fail on > 1µs p95 | perf |
| `view_incremental_maintenance` | < 50µs per write | §14.6 derived | Fail on > 50µs p95 | perf |
| `concurrent_writers` | 100-1000 writes/sec | §14.6 direct | Informational | perf |
| `durability_mode_matrix` | Group < 500µs, Async < 100µs, Immediate documented range | §14.6 derived | Fail on Group > 500µs | perf |
| `multi_mb_roundtrip` | 1MB < 10ms, 10MB < 100ms, 100MB < 1s | non-§14.6 | Informational | perf |
| `cid_parse_from_str` | < 1µs | non-§14.6 | Informational | perf |
| `blake3_hash_node_small` | < 2µs per Node | non-§14.6 | Informational | perf |
| `transform_expression_small` | < 10µs per expression | non-§14.6 | Informational | perf |
| `cold_install_size` | npm install `@benten/engine` completes in < 60s (CI gate per R1 dx B8) | non-§14.6 | Fail on > 60s | perf |

---

## 6. TypeScript ↔ Rust End-to-End (closes SPIKE punt #2)

| Vitest file | Test | Assertion |
|---|---|---|
| `bindings/napi/index.test.ts` | `ts_rust_cid_roundtrip_matches_fixture` | `initEngine` → `createNode` with canonical fixture properties → returned CID === `"bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda"`. Exit-criterion #6. |
| `bindings/napi/index.test.ts` | `ts_full_crud_cycle` | create/get/update/delete round-trip for nodes and edges |
| `bindings/napi/index.test.ts` | `ts_subgraph_register_and_call` | register crud('post'), call 'post:create', assert change stream fired |
| `bindings/napi/index.test.ts` | `ts_trace_contains_per_node_timings` | `traceHandler` returns `TraceStep[]` with `durationUs > 0` everywhere |
| `bindings/napi/index.test.ts` | `ts_napi_input_validation_rejects_oversized` | Oversized map/list/bytes rejected with `E_INPUT_LIMIT` (B8) |
| `packages/engine/src/crud.test.ts` | `crud_post_zero_config_full_cycle` | register crud('post'), create × 3, list returns 3 in order, update one, delete one. Exit-criterion #2. |
| `packages/engine/src/crud.test.ts` | `crud_post_zero_config_injects_createdAt_deterministically` | two `post:create` calls ~1ms apart have distinct monotonic `createdAt`; HLC stamped once |
| `packages/engine/src/errors.test.ts` | `typed_error_classes_generated_from_catalog` | Every error code in `docs/ERROR-CATALOG.md` has a TS class; `fixHint` surfaced in `error.toString()` |
| `packages/engine/src/trace.test.ts` | `trace_returns_topo_ordered_steps` | handler with BRANCH/ITERATE/CALL — every step appears in a topologically valid order. Exit-criterion #4. |
| `packages/engine/src/mermaid.test.ts` | `mermaid_output_parses_as_flowchart` | `handler.toMermaid()` passes minimal Mermaid grammar check. Exit-criterion #5. |
| `tools/create-benten-app/test/scaffolder.test.ts` | `scaffolder_produces_working_project` | `npx create-benten-app test-app && cd test-app && npm install && npm test` exits 0. Headline exit criterion. |

---

## 7. Security-Class Tests

The **11 R1-named security tests** (all required per plan R1 Triage Addendum § "Test-landscape additions") plus expansion.

| Test name | R3 owner | Attack vector | File | Codes fired |
|---|---|---|---|---|
| `napi_rejects_oversized_value_map` (B8) | security | DoS via unbounded map keys | `bindings/napi/tests/input_validation.rs` | `E_INPUT_LIMIT` |
| `napi_rejects_deep_nested_value` (B8) | security | DoS via deep recursion / stack blow-up | `bindings/napi/tests/input_validation.rs` | `E_INPUT_LIMIT` |
| `napi_rejects_oversized_bytes` (B8) | security | DoS via >16MB Value::Bytes | `bindings/napi/tests/input_validation.rs` | `E_INPUT_LIMIT` |
| `napi_rejects_malformed_cid` (B8) | security | CID injection / malformed multibase | `bindings/napi/tests/input_validation.rs` | `E_INPUT_LIMIT` |
| `transform_grammar_rejects_tagged_templates` (T12) | security | Prototype pollution via tag func | `crates/benten-eval/tests/transform_grammar_rejections.rs` | `E_TRANSFORM_SYNTAX` |
| `transform_grammar_rejects_computed_proto_keys` (T12) | security | `obj[__proto__]` at runtime | same file | `E_TRANSFORM_SYNTAX` |
| `transform_grammar_rejects_new_Function` (T12) | security | Code injection via `new Function()` | same file | `E_TRANSFORM_SYNTAX` |
| `transform_grammar_rejects_with_statement` (T12) | security | Scope escape via `with` | same file | `E_TRANSFORM_SYNTAX` |
| `transform_grammar_rejects_destructuring_getter` (T12) | security | Side-effect via getter during destructure | same file | `E_TRANSFORM_SYNTAX` |
| `transform_grammar_fuzz_harness` (T12) | security | Unknown syntax shapes | same file (ignored-by-default) | `E_TRANSFORM_SYNTAX` |
| `capability_revoked_mid_iteration_denies_subsequent_batches` (R1 named) | security | TOCTOU during long ITERATE | `crates/benten-caps/tests/toctou_iteration.rs` | `E_CAP_REVOKED_MID_EVAL` |
| `handler_with_understated_requires_denies_excess_writes` (R1 SC4) | security | AI-agent escalation via understated `requires` | `crates/benten-eval/tests/requires_enforcement.rs` | `E_CAP_DENIED` |
| `handler_cannot_escalate_via_call_attenuation` (R1 SC4) | security | Privilege escalation through sub-CALL | same file | `E_CAP_DENIED` |
| `user_operation_cannot_write_system_labeled_node` (R1 SC1) | security | System-zone boundary | `crates/benten-graph/tests/system_zone.rs` | `E_SYSTEM_ZONE_WRITE` |
| `read_denied_returns_cap_denied_read` (R1 named) | security | Option-A existence-visibility | `crates/benten-eval/tests/read_denial.rs` | `E_CAP_DENIED_READ` |
| `ucan_stub_error_message_names_phase_and_alternative` (R1 named) | security | Operator misconfiguration clarity | `crates/benten-caps/tests/ucan_stub_messages.rs` | `E_CAP_NOT_IMPLEMENTED` |
| `ucan_stub_error_routes_to_ON_ERROR_not_ON_DENIED` (R1 named) | security | Error routing correctness | same file | `E_CAP_NOT_IMPLEMENTED` |
| `supply_chain_ci_green_on_clean_lockfile` (R1 SC3) | security | Supply-chain signal integrity | CI harness `.github/workflows/supply-chain.yml` | — |
| `engine_builder_production_refuses_noauth` (R1 SC2) | security | Accidentally-shipping NoAuth in prod | `crates/benten-engine/tests/production_refuses_noauth.rs` | — |
| Additional TRANSFORM rejection classes (to fill 15 classes per plan) | security | Each additional forbidden construct (closures, `this`, imports, optional-chained calls, `new`, spread-into-call, comma op, prototype access via `.__proto__`, `yield`/`async`/`await`, template literal with expressions) | `crates/benten-eval/tests/transform_grammar_rejections.rs` (continued) | `E_TRANSFORM_SYNTAX` |

**TRANSFORM grammar rejection matrix (15 classes) — explicit enumeration:**

1. closures (`() => x`) → `E_TRANSFORM_SYNTAX`
2. `this` references
3. `import` statements
4. prototype access (`__proto__`, `constructor`, `prototype`)
5. tagged templates
6. template literals with expressions
7. optional-chained method calls
8. computed property names referencing `__proto__` / `constructor` / `Symbol.*`
9. `new` in any position
10. `with` statement
11. destructuring with getter triggers
12. spread into call
13. comma operator
14. `yield` / `async` / `await` / `eval`
15. `Function` constructor / `new Function(...)` / global `Function`

Each class = one test asserting `register_subgraph(...)` returns `Err(E_TRANSFORM_SYNTAX)` with a specific error message naming the construct.

---

## 8. Cross-Cutting / CI Harnesses

| Harness | File | R3 owner | Gate policy |
|---|---|---|---|
| T4 multi-arch matrix | `.github/workflows/ci.yml` — matrix job `reproduce-fixture` × `{aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu}` | perf | Required status check; PR blocks on any leg divergence |
| T5 MSRV CI | `.github/workflows/ci.yml` — matrix job `msrv-fixture` × `{1.85, stable}` | perf | Required status check |
| T6 WASM runtime CI | `.github/workflows/ci.yml` — job `wasm-runtime-fixture` compiles `print_canonical_cid` for `wasm32-wasip1`, runs under wasmtime, asserts fixture | perf | Required status check |
| T8 napi wasm32 compile-check | `.github/workflows/ci.yml` — `cargo check --target wasm32-unknown-unknown -p benten-napi` | perf | Required status check |
| T7 Error-catalog drift detector | `.github/workflows/ci.yml` — `scripts/drift-detect-errors.ts` (bidirectional compare of doc + Rust enum + TS types) | security | Required status check; test `drift_detector_fails_on_missing_rust_variant` asserts the tool actually fails (not vacuous pass) |
| T9 Cross-leg determinism gate | `.github/workflows/ci.yml` — `determinism-gate` job depends on T4/T5/T6/T8; collects each leg's CID, asserts byte-for-byte equality (not just "each matches fixture") | perf | Required status check |
| T10 Supply-chain CI (SC3) | `.github/workflows/supply-chain.yml` — `cargo-audit` + `cargo-deny check` + `cargo build --locked` verify + weekly `cargo update --dry-run` + `cargo audit` workflow opening issues on new advisories | security | Required on main; fail on HIGH/CRITICAL |
| Cold-install gate (B8 from R1 dx) | `.github/workflows/ci.yml` — job `cold-install-latency` downloads prebuilt binary matrix, measures cold `npm install` < 60s | perf | Required status check |
| Cross-process determinism (graph extension of D2) | `.github/workflows/ci.yml` — job `graph-cross-process` runs write in process A + read in process B + assert equal CID | integration | Required status check |

**Supply-chain CI (T10) decomposition (4 required jobs):**

1. `cargo-audit` — fail on any HIGH/CRITICAL advisory
2. `cargo-deny check` — enforce license allowlist (MIT, Apache-2.0, BSD-3-Clause, CC0-1.0); banned-crate list
3. `cargo build --locked` — ensure Cargo.lock present and used
4. `weekly-advisory-scan` — cron `cargo update --dry-run` + `cargo audit`; open GitHub issue on new advisories

---

## 9. R3 Partition Recommendation

### `rust-test-writer-unit` — 34 test files

**Categories owned:** per-primitive happy paths, per-API contracts, proptest harness.

**Files:**
- `crates/benten-core/tests/value_variants.rs` (7 variants)
- `crates/benten-core/tests/value_float.rs` (C3)
- `crates/benten-core/tests/node_cid.rs` (Node::cid determinism)
- `crates/benten-core/tests/edge_cid.rs` (Edge::cid)
- `crates/benten-core/tests/edge_does_not_change_endpoint_cids.rs` (§7 invariant)
- `crates/benten-core/tests/anchor_version.rs` (C6 helpers)
- `crates/benten-core/tests/error_codes.rs` (ErrorCode enum)
- `crates/benten-core/tests/proptests.rs` (proptest harness host)
- `crates/benten-graph/tests/kvbackend_conformance.rs` (trait laws)
- `crates/benten-graph/tests/scan_iterator.rs`
- `crates/benten-graph/tests/transaction_atomicity.rs`
- `crates/benten-graph/tests/change_event.rs`
- `crates/benten-graph/tests/change_subscriber_trait.rs`
- `crates/benten-graph/tests/node_edge_store_blanket.rs`
- `crates/benten-graph/tests/mvcc_snapshot.rs`
- `crates/benten-graph/tests/label_prop_index.rs`
- `crates/benten-ivm/tests/view_trait.rs`
- `crates/benten-ivm/tests/view_definition.rs`
- `crates/benten-ivm/tests/subscriber_routing.rs`
- `crates/benten-ivm/tests/view1_capability_grants.rs`
- `crates/benten-ivm/tests/view2_event_dispatch.rs`
- `crates/benten-ivm/tests/view3_content_listing.rs`
- `crates/benten-ivm/tests/view5_version_current.rs`
- `crates/benten-caps/tests/noauth.rs`
- `crates/benten-caps/tests/capability_grant.rs`
- `crates/benten-caps/tests/policy_trait.rs`
- `crates/benten-caps/tests/error_code_mapping.rs`
- `crates/benten-eval/tests/primitive_types.rs` (E1 12 primitives)
- `crates/benten-eval/tests/primitive_read_write.rs`
- `crates/benten-eval/tests/primitive_branch_respond_emit.rs`
- `crates/benten-eval/tests/primitive_transform_builtins.rs`
- `crates/benten-eval/tests/phase_two_primitives_structural.rs`
- `crates/benten-eval/tests/evaluator_stack.rs`
- `crates/benten-engine/tests/spike_survivors.rs` (N1 regression)

### `rust-test-writer-edge-cases` — 22 test files

**Categories owned:** error edges, typed error routing, boundary conditions.

**Files:**
- `crates/benten-core/tests/cid_malformed.rs`
- `crates/benten-core/tests/float_nan_inf.rs`
- `crates/benten-core/tests/version_branched.rs`
- `crates/benten-core/tests/anchor_id_excluded_from_cid.rs`
- `crates/benten-graph/tests/open_existing_vs_create.rs`
- `crates/benten-graph/tests/failure_injection_rollback.rs`
- `crates/benten-graph/tests/nested_tx_rejected.rs`
- `crates/benten-graph/tests/scan_zero_hit.rs`
- `crates/benten-graph/tests/backend_error_polymorphism.rs`
- `crates/benten-ivm/tests/view4_governance_inheritance.rs` (depth-cap edge)
- `crates/benten-ivm/tests/stale_on_budget_exceeded.rs`
- `crates/benten-ivm/tests/view_read_allow_stale.rs`
- `crates/benten-caps/tests/grant_uniqueness_on_cid.rs`
- `crates/benten-caps/tests/check_write_called_at_commit.rs`
- `crates/benten-eval/tests/invariant_1_cycle.rs`
- `crates/benten-eval/tests/invariant_2_depth.rs`
- `crates/benten-eval/tests/invariant_3_fanout.rs`
- `crates/benten-eval/tests/invariants_5_6_counts.rs`
- `crates/benten-eval/tests/invariants_9_10_12.rs`
- `crates/benten-eval/tests/iterate_max_and_nest_depth.rs`
- `crates/benten-engine/tests/engine_builder_thinness.rs`
- `crates/benten-engine/tests/engine_read_view_stale.rs`
- `crates/benten-engine/tests/register_subgraph_failures.rs`

### `rust-test-writer-security` — 14 test files

**Categories owned:** adversarial tests from R1 security auditor, TRANSFORM grammar rejection matrix, napi input validation, `requires` property enforcement.

**Files:**
- `crates/benten-graph/tests/system_zone.rs` (N8 SC1 — `user_operation_cannot_write_system_labeled_node`)
- `crates/benten-caps/tests/ucan_stub_messages.rs` (SC2 + R1 named × 2)
- `crates/benten-caps/tests/toctou_iteration.rs` (R1 named — `capability_revoked_mid_iteration_denies_subsequent_batches`)
- `crates/benten-caps/tests/noauth_proptest.rs` (prop_noauth_returns_ok_unconditionally)
- `crates/benten-eval/tests/transform_grammar_rejections.rs` (15-class rejection matrix)
- `crates/benten-eval/tests/transform_grammar_fuzz.rs` (ignored-by-default fuzz)
- `crates/benten-eval/tests/requires_enforcement.rs` (SC4 × 3)
- `crates/benten-eval/tests/read_denial.rs` (R1 — `read_denied_returns_cap_denied_read`)
- `crates/benten-engine/tests/production_refuses_noauth.rs` (SC2 — `engine_builder_production_refuses_noauth`)
- `crates/benten-engine/tests/system_zone_api_exclusivity.rs` (N7)
- `bindings/napi/tests/input_validation.rs` (B8 × 4)
- `.github/workflows/supply-chain.yml` (SC3 — `supply_chain_ci_green_on_clean_lockfile` + 3 supply-chain jobs)
- `.github/workflows/drift-detect.yml` (T7 drift detector)
- `crates/benten-eval/tests/requires_property_call_time_check.rs` (primitive-level `requires` enforcement per R1 SC4 option A)

### `rust-test-writer-performance` — 10 test files

**Categories owned:** criterion benchmarks, §14.6 targets, concurrent writers, cold-install CI.

**Files:**
- `crates/benten-core/benches/hash_only.rs`
- `crates/benten-core/benches/cid_parse.rs`
- `crates/benten-graph/benches/get_create_node.rs` (get_node, create_node_immediate, create_node_group_commit)
- `crates/benten-graph/benches/durability_modes.rs`
- `crates/benten-graph/benches/concurrent_writers.rs`
- `crates/benten-graph/benches/multi_mb_roundtrip.rs`
- `crates/benten-graph/tests/concurrent_reader_writer_soak.rs` (stress, not bench)
- `crates/benten-ivm/benches/view_maintenance.rs` (view_read_content_listing, view_incremental_maintenance)
- `crates/benten-eval/benches/ten_node_handler.rs` (10_node_handler_eval + transform_expression_small)
- `crates/benten-engine/benches/end_to_end_create.rs` (spike-compatible baseline)

Plus CI harness entries in `.github/workflows/ci.yml` — multi-arch, MSRV, WASM runtime, napi wasm32 compile-check, cross-leg determinism gate, cold-install latency gate, cross-process determinism. These are owned by the performance writer because they encode performance + determinism gates.

### `qa-expert` (integration) — 13 test files

**Categories owned:** cross-crate scenarios, TypeScript end-to-end (Vitest), 6 exit-criterion assertions, named-compromise regression.

**Files:**
- `crates/benten-engine/tests/integration/caps_crud.rs`
- `crates/benten-engine/tests/integration/ivm_propagation.rs`
- `crates/benten-engine/tests/integration/tx_atomicity.rs`
- `crates/benten-engine/tests/integration/version_current.rs`
- `crates/benten-engine/tests/integration/change_stream.rs`
- `crates/benten-engine/tests/integration/system_zone_integration.rs`
- `crates/benten-engine/tests/integration/cap_toctou.rs`
- `crates/benten-engine/tests/integration/stale_view.rs`
- `crates/benten-engine/tests/integration/nested_tx.rs`
- `crates/benten-engine/tests/integration/exit_criteria_all_six.rs` (one sub-test per Vitest assertion, Rust-side equivalent)
- `crates/benten-engine/tests/integration/compromises_regression.rs` (6 sub-tests)
- `crates/benten-engine/tests/integration/cross_process_graph.rs` (D2 extension)
- `bindings/napi/index.test.ts` + `packages/engine/src/*.test.ts` + `tools/create-benten-app/test/scaffolder.test.ts` (TS-side Vitest — treated as one "file cluster" owned by integration agent since they're co-designed)

### Partition disjointness verification

Every file above appears in exactly one agent's list. Three deliberately-split files:
- `crates/benten-caps/tests/noauth.rs` (unit) vs `crates/benten-caps/tests/noauth_proptest.rs` (security) — different concerns, different files.
- `crates/benten-engine/tests/system_zone_api_exclusivity.rs` (security — N7 engine-API boundary) vs `crates/benten-engine/tests/integration/system_zone_integration.rs` (integration — full-stack) — same invariant, different scopes.
- `crates/benten-graph/tests/system_zone.rs` (security — N8 write-path) vs the two above — the three together cover all three layers.

### Prerequisites (per-agent)

Before R3 dispatch can begin:

| Prereq | Owner | Blocks agent |
|---|---|---|
| **T12 `docs/TRANSFORM-GRAMMAR.md` BNF** | pre-R3 doc slice | `rust-test-writer-security` (15-class rejection matrix needs the BNF to define what's "allowlisted") |
| **ERROR-CATALOG drift-detector spec** | pre-R3 doc slice (already done in R1 addendum) | `rust-test-writer-unit` (tests every code's `fixHint` must know the catalog schema) |
| **Cargo-deny + cargo-audit CI config draft** | pre-R3 infra slice (T10 scope) | `rust-test-writer-performance` (supply-chain test wiring) |
| **Mermaid BNF reference grammar** | pre-R3 doc slice (minimal subset OK) | `qa-expert` (exit-criterion #5 Mermaid parse test) |
| **HLC property-key name finalized (`createdAt` type + format)** | pre-R3 plan slice (already in R1 addendum as deterministic HLC; but key-name confirmation) | `qa-expert` (exit-criterion #2 sorting test) |

None block the R2 landscape from shipping. They block specific R3 agents' dispatch readiness.

---

## 10. Coverage-Map JSON Stub

See `.addl/phase-1/r3-coverage-stub.json` — schema per agent definition, populated skeleton with all planned counts, error-code coverage map, and per-agent file ownership.

---

## Plan-level gaps flagged for Ben before R3 dispatches

1. **Exit-criterion #5 Mermaid grammar check.** The plan says "verified by a minimal Mermaid grammar check." No Mermaid subset spec exists. Options: (a) use the `@mermaid-js/parser` npm package as a dependency, (b) hand-roll a minimal flowchart grammar, (c) settle for "does not throw when passed to mermaid.render." Recommend (a) as dep-light; flag for confirmation.
2. **HLC clock source for `crud('post')` `createdAt` injection.** R1 dx disposition says "deterministic HLC timestamp stamped once at create, never re-computed." The HLC implementation is scoped to Phase 3 (`uhlc 0.2.x`) per tech-stack table. Phase 1 needs *some* monotonic timestamp source. Options: `std::time::Instant` with process-lifetime offset, monotonic counter, lightweight HLC. Recommend monotonic counter stamped at Node creation, keyed by Anchor CID, so it survives restarts; flag for confirmation.
3. **`E_NESTED_TRANSACTION_NOT_SUPPORTED` is listed for Phase 1 but `ERROR-CATALOG.md` already has it — verify no drift between plan R1 Addendum's 9 new codes and actual catalog contents.** R2 verified the catalog has: `E_PRIMITIVE_NOT_IMPLEMENTED`, `E_SYSTEM_ZONE_WRITE`, `E_CAP_NOT_IMPLEMENTED`, `E_CAP_REVOKED_MID_EVAL`, `E_CAP_DENIED_READ`, `E_INPUT_LIMIT`, `E_TRANSFORM_SYNTAX`, `E_INV_CONTENT_HASH`, `E_INV_REGISTRATION`, and the `E_NESTED_TRANSACTION_NOT_SUPPORTED` code is also present. The 4 codes to be removed (`E_INV_ITERATE_MAX_MISSING`, `E_INV_ITERATE_BUDGET`, `E_INV_SANDBOX_NESTED`, `E_INV_SYSTEM_ZONE`) are still present in the catalog but marked with `Phase: 2` — this is **inconsistent with the triage "remove" disposition**. Recommend: leave them in the catalog as reserved-Phase-2 codes (which is what the markers already say) and tighten the triage language from "remove" to "mark Phase 2." Flag for Ben's confirmation.

**None of the three gaps blocks R2 from shipping.** The test landscape is complete against the plan-as-written; gaps would become R3 ambiguity if not resolved before R3 dispatch.

---

## Appendix — ERROR-CATALOG coverage audit

**Catalog codes (26 active Phase 1 codes):**

Registration-time invariants (11): `E_INV_CYCLE`, `E_INV_DEPTH_EXCEEDED`, `E_INV_FANOUT_EXCEEDED`, `E_INV_SANDBOX_NESTED` (Phase 2), `E_INV_TOO_MANY_NODES`, `E_INV_TOO_MANY_EDGES`, `E_INV_SYSTEM_ZONE` (Phase 2), `E_INV_DETERMINISM`, `E_INV_ITERATE_MAX_MISSING` (Phase 2), `E_INV_ITERATE_BUDGET` (Phase 2), `E_INV_ITERATE_NEST_DEPTH`, `E_INV_CONTENT_HASH`, `E_INV_REGISTRATION`.

Evaluation-time (13): `E_CAP_DENIED`, `E_CAP_DENIED_READ`, `E_CAP_REVOKED_MID_EVAL`, `E_CAP_NOT_IMPLEMENTED`, `E_CAP_REVOKED`, `E_CAP_ATTENUATION`, `E_WRITE_CONFLICT`, `E_SANDBOX_FUEL_EXHAUSTED` (Phase 2), `E_SANDBOX_TIMEOUT` (Phase 2), `E_SANDBOX_OUTPUT_LIMIT` (Phase 2), `E_IVM_VIEW_STALE`, `E_TX_ABORTED`, `E_NESTED_TRANSACTION_NOT_SUPPORTED`, `E_PRIMITIVE_NOT_IMPLEMENTED`, `E_SYSTEM_ZONE_WRITE`, `E_TRANSFORM_SYNTAX`, `E_INPUT_LIMIT`.

Sync-receive (3) — **Phase 3**: `E_SYNC_HASH_MISMATCH`, `E_SYNC_HLC_DRIFT`, `E_SYNC_CAP_UNVERIFIED` — no Phase 1 test.

TS-binding (2): `E_DSL_INVALID_SHAPE`, `E_DSL_UNREGISTERED_HANDLER`.

**Phase 1 required coverage (excluding Phase-2/3 codes):** 21 codes.

**Codes covered by at least one test in the landscape:** 21 (all).

**Coverage stat: 21/21 = 100% of Phase 1 codes have a test.**

Phase 2 codes (`E_INV_SANDBOX_NESTED`, `E_INV_ITERATE_MAX_MISSING` [full invariant-8 form], `E_INV_ITERATE_BUDGET`, `E_INV_SYSTEM_ZONE` [full-registration form], `E_SANDBOX_*`) deliberately have NO Phase 1 test — they're reserved-but-dormant. R3 does not cover them.
