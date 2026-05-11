# benten-caps — Internals

A plain-English deep-dive on the `benten-caps` crate. Companion to the public `lib.rs` docs; written for fresh agents coming into Phase 4 (Foundation + Meta) work. Treats *why* and *what's load-bearing* as first-class.

---

## 1. What this crate does

`benten-caps` is the **capability-policy layer** for the Benten engine. It owns the pre-write (and pre-read) hook trait — `CapabilityPolicy` — that the transaction primitive consults at commit time, plus the surrounding types every concrete policy backend needs: `WriteContext`, `ReadContext`, `PendingOp`, `CapabilityGrant`, `GrantScope`, the segment-wise attenuation check, the typed `CapError` family that maps 1:1 to stable codes in `docs/ERROR-CATALOG.md`, and a separate `RateLimitPolicy` trait whose plug-in shape composes alongside the capability policy. The crate ships three concrete `CapabilityPolicy` impls: `NoAuthBackend` (the zero-cost Phase-1 default that permits everything), `GrantBackedPolicy` (reads `system:CapabilityGrant` / `system:CapabilityRevocation` Nodes through a small `GrantReader` trait and denies writes whose scope has no unrevoked grant), and `UcanGroundedPolicy` (composes `GrantBackedPolicy` with `UCANBackend` proof-chain validation for the `cap:typed:*` namespace). The durable UCAN backend itself — `UCANBackend<B: GraphBackend>` — lives at `src/backends/ucan.rs` and is native-only.

Per CLAUDE.md baked-in #7, the capability system is *pluggable policy*, not a fixed UCAN implementation. The trait is the contract; backends slot in. UCAN-specific identity types (DIDs, keypairs, signed UCAN envelopes, the in-memory chain-walker) live in the `benten-id` crate; this crate consumes them at the `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` boundary because identity work is full-peer-only per baked-in #17. The thin-client wasm32 build sees only the policy trait + `NoAuthBackend` + `GrantBackedPolicy` + `LegacyUcanStubBackend` + the typed `CapError` surface.

---

## 2. Dependency chain

**Workspace-in (the crate's own deps):**

- `benten-core` — for `Cid`, `Node`, `Value`, `CoreError`, `WriteAuthority` (re-exported through `policy::WriteAuthority` as the single canonical type across core / graph / caps).
- `benten-errors` — for the stable `ErrorCode` enum that `CapError::code()` maps into.
- `benten-graph` — for the `GraphBackend` umbrella trait + its `KVBackend` super-trait. The durable UCAN backend keys grants + revocations into the inherited KV surface (`g14b:grant:*`, `g14b:revoked:*`, `g14b:dev_revoke:*`). Promoted from dev-dep to non-dev-dep at G14-B wave-4b.
- `benten-id` (native-only) — for `Ucan`, `UcanError`, `Did`, `DeviceRevocation`, `validate_chain_at` / `validate_chain_for_audience` / `validate_chain_with_device_revocations`. The chain-walk logic lives in `benten-id`; this crate composes those entry points with durable storage + revocation lookups.

**External in:**

- `thiserror` — the typed `CapError` derive.
- `blake3` — for the BLAKE3 hash that computes the content-CID of a UCAN envelope (matching the engine's `Node::cid` scheme).
- `serde_ipld_dagcbor` — for the DAG-CBOR encode/decode of UCAN bodies + device-revocation entries into the KV store.
- `subtle` — pulled at this layer to pin the same minor version that flows through `benten-id`; any cap-side security-decision compare (signature equality, DID equality) goes through `ConstantTimeEq` per `crypto-major-4`.

**Dev-deps:** `proptest`, `tempfile`, `criterion`, and `benten-engine` (with `features = ["test-helpers"]`) for the few integration tests that need a full engine fixture (e.g. `toctou_iteration.rs`, `check_write_called_at_commit.rs`).

**Consumers out:**

- `benten-eval` consumes the trait + `WriteContext` / `ReadContext` to call `check_write` at transaction commit + `check_read` on the engine's read path.
- `benten-engine` is the composing layer: `EngineBuilder` owns the choice of which `CapabilityPolicy` to plug in (`NoAuthBackend` default, `GrantBackedPolicy` for the Phase-2b grant-reader path, `UcanGroundedPolicy` for the Phase-3 durable-UCAN path via the `capability_policy_ucan_durable` builder). The engine also implements `GrantReader` against its own `RedbBackend` and injects it as `Arc<dyn GrantReader>` into `GrantBackedPolicy::new`.
- `benten-sync` consumes the `RateLimitPolicy` plug at the Atrium boundary for per-peer bandwidth accounting.

**No cycles.** `benten-caps` sits between `benten-graph` (below it) and `benten-eval` / `benten-engine` (above it). It does not depend on `benten-eval` (which would invert the layering). Integration tests that need a full `Engine` do so as dev-deps only, and the dev-cycle is safe because test binaries link separately from the lib crate graph.

---

## 3. Files inventory in `src/`

- **`lib.rs`** (198 LOC) — crate root. Module declarations, public re-exports, and three small in-tree items: `evaluator_delegation` (a helper module the evaluator calls into for `iterate_batch_boundary` + `wallclock_refresh_ceiling`, structured so test mocks can count invocations), `HlcStampedRefreshEvent` + `emit_refresh_event_for_test` (the §9.13 dual-source refresh-event shape and a test synthesiser), the `DEFAULT_BATCH_BOUNDARY` constant (`100`), and a `testing::` back-compat re-export module that preserves the historical `benten_caps::testing::check_attenuation` import path.
- **`policy.rs`** (362 LOC) — the `CapabilityPolicy` trait, the `WriteContext` / `ReadContext` structs, the `PendingOp` enum (`PutNode` / `PutEdge` / `DeleteNode` / `DeleteEdge`, sealed behind `#[non_exhaustive]` so future SANDBOX `HostFunctionCall` variants are a minor bump), and the re-export of `WriteAuthority` from `benten-core`. Both contexts are deliberately public-field structs so a policy implementor can write a single match expression. Both carry a `device_cid: Option<Cid>` (Phase-3 G16-B canary, r4b-cap-3 closure) so heterogeneous policies can dispatch per-device under the same logical actor identity. `ReadContext` exposes two typed constructors (`by_cid_only` + `by_label_only`) so the "empty-label means CID-only" convention is no longer an unwritten rule.
- **`error.rs`** (291 LOC) — the `#[non_exhaustive]` `CapError` enum. 17 variants today: `Denied` / `DeniedRead` (both carry a `(required, entity)` payload — flattened from an earlier `DeniedDetail` split that was a hazard for audit pipelines), `Revoked`, `RevokedMidEval` (distinct so per-iteration mid-eval revocations don't conflate with cross-peer sync revocations), `NotImplemented`, `Attenuation`, `ScopeLoneStarRejected`, `ChainTooDeep { depth, limit }` (R4 tq-7 has a `chain_depth_context` accessor so tests can assert without grepping the message), `WallclockExpired`, the eight G14-B durable-UCAN variants (`UcanExpired`, `UcanNotYetValid`, `UcanBadSignature`, `UcanAttenuationViolated`, `UcanAudienceMismatch`, `BackendStorage`, `UcanClockNotInjected`, `RateLimitExceeded`, `PeerBandwidthExceeded`). `code()` maps each variant 1:1 to a `benten_errors::ErrorCode`. Display formats deliberately use `{required}` / `{entity}` not `{:?}` (r6-err-6) so JS `Error.message` doesn't carry escaped quotes.
- **`noauth.rs`** (41 LOC) — `NoAuthBackend`, a zero-sized type. `check_write` is `#[inline]` and returns `Ok(())` unconditionally; no allocations, no branches on `WriteContext` fields. Carries a `pseudo_actor_label()` returning the static `"noauth"` so change-event attribution still has something to record.
- **`attenuation.rs`** (151 LOC) — the segment-wise `check_attenuation` function plus the `AsAttenuationScope` trait that lets it accept `GrantScope`, `&GrantScope`, `&str`, or `&String`. Strict subset semantics: child longer than parent without a trailing `*` is denied; child shorter than parent is denied (must narrow, never widen); mid-scope `*` consumes one segment; trailing `*` short-circuits to permit the whole tail. The module's doc-comment names the two concrete bypasses (parent `"*"` accepting `"store:post:write:admin:override"`; parent `"store:*"` accepting `"store:anything:write:delete"`) that the earlier zip-on-shorter draft permitted — and that the `proptest_attenuation.rs` suite exercises.
- **`grant.rs`** (253 LOC) — `CapabilityGrant`, `GrantScope`, and the three string constants `CAPABILITY_GRANT_LABEL` (`"system:CapabilityGrant"`), `GRANTED_TO_LABEL` (`"GRANTED_TO"`), `REVOKED_AT_LABEL` (`"REVOKED_AT"`). `GrantScope::parse` is intentionally strict: rejects empty/whitespace-only, the lone `"*"` footgun, and any empty inner / leading / trailing segment (the `"store::write"` visual-confusion / encoding-trick surface — auditor finding g4-p2-uc-4). `CapabilityGrant` is a four-public-field struct (`grantee`, `issuer`, `scope`, `hlc_stamp`) plus an optional `ttl_hlc_duration` (ucca-8 additive shape; `None` keeps Phase-1 CIDs bit-identical via `skip_serializing_if = "Option::is_none"` semantics). `as_node()` produces the content-addressed `Node` representation with label `"system:CapabilityGrant"` — load-bearing for the namespace-aligned read path (see §7 below).
- **`grant_backed.rs`** (418 LOC) — `GrantBackedPolicy`, the `GrantReader` trait (read-only handle that answers `has_unrevoked_grant_for_scope(scope)` + the default-impl batched `has_unrevoked_grant_for_any(scopes)`), `GrantReaderConfig` (max_chain_depth = 64 default), `GrantReaderChain` (a Phase-2a test harness pinning the chain-depth bound), and a `wildcard_variants` enumerator that translates a concrete required scope into every parent-scope spelling that would attenuate to it (so the policy can match a stored `"store:post:*"` grant against a required `"store:post:write"` without forcing the reader to carry wildcard awareness). The `check_write` path is the load-bearing one: engine-privileged contexts bypass; `pending_ops` drives per-op scope derivation (`store:<label>:write`); system-zone (`"system:"` prefix) writes are skipped (they reach the policy only on a higher-level wiring bug — denying them double-fires the guard); empty pending-ops + empty label/scope is denied rather than permitted (r6-sec-8 closes the fail-open surface). `check_read` derives `store:<label>:read`, permits empty/`system:` labels, and otherwise wildcard-enumerates against the reader, returning typed `DeniedRead` (Phase-1 named compromise #2 Option A — existence leak documented on the trait method).
- **`rate_limit.rs`** (422 LOC) — `RateLimitPolicy` trait (`check_writes_per_sec` / `check_read_budget` / `check_peer_bandwidth` / `is_peer_back_pressured`), `NullRateLimitPolicy` (default no-op), and `InMemoryRateLimitPolicy` with a `Builder` for configured per-`(actor, zone)` / per-actor / per-peer budgets. 1-second sliding-window buckets via a `Mutex<RateLimitState>` and an injectable clock for tests. Per-peer back-pressure carries a sticky `saturated_in_window` flag on the bandwidth bucket so a peer remains back-pressured for the remainder of the window even after a rejected chunk leaves the bucket count untouched (see test `cross_peer_back_pressure_via_rate_limit_policy`).
- **`typed_cap_mapping.rs`** (214 LOC) — closes G21-T2 §D §2.5(c). The 8 typed-CALL caps under `cap:typed:*` (CryptoSign, CryptoVerify, CryptoKeygen, Hash, Codec, DidResolve, UcanValidate, VcVerify) are structurally declared at `benten_eval::TypedCallOp::required_cap`. This module ships the inverse: `typed_cap_for_ucan_claim(resource, ability) -> Option<TypedCapGroup>`, so the durable UCAN backend's chain-walker can translate a UCAN leaf-claim `(resource, ability)` into the matching `cap:typed:*` string and decide whether the proof grants the required capability. `NoAuthBackend` permits all typed caps by default; this table is consulted only by `UcanGroundedPolicy`.
- **`ucan_grounded.rs`** (491 LOC, native-only) — `UcanGroundedPolicy<B: GraphBackend>`, the composed policy that consults `GrantBackedPolicy` first (fast path: Phase-2b revocation-aware grant store), and on denial falls through to `UCANBackend::iter_installed_proofs` + per-proof `validate_chain_at` + leaf-claim → `cap:typed:*` mapping. Native-only because it depends on `UCANBackend`. Owns the `DEFAULT_NOW_SECS = 0` sentinel + the `chain_has_time_bounds` helper that drives the G16-B-B-rest sub-item D fail-closed inversion: a chain with any `nbf > 0` or `exp > 0` against `now_secs == 0` aborts with `CapError::UcanClockNotInjected` rather than silently fail-OPENing whenever a forged chain happens to have `nbf=0`. The composed read path defers to `GrantBackedPolicy` unchanged (typed-cap reads do not flow through the namespace).
- **`ucan_stub.rs`** (53 LOC) — `LegacyUcanStubBackend` (renamed from `UcanBackend` at G21-T2 audit-6-1 closure). Every `check_write` returns `CapError::NotImplemented { backend: "UCANBackend", lands_in_phase: 3 }`. Preserved for the legacy tests that pin the `ON_ERROR` (not `ON_DENIED`) routing contract under `tests/ucan_stub_messages.rs`. New code MUST NOT use this type — `EngineBuilder::capability_policy_ucan_durable` composes the production-grade `UCANBackend` + `GrantBackedPolicy`.
- **`backends/mod.rs`** (27 LOC) — the `cfg(not(target_arch = "wasm32"))` gate that hides the durable UCAN backend on the wasm32 thin-client target. Mirrors the `bindings/napi/Cargo.toml` G13-C / G14-A1 cfg-gating pattern.
- **`backends/ucan.rs`** (568 LOC, native-only) — `UCANBackend<B: GraphBackend>`, the durable Phase-3 UCAN backend. Module doc names the **dual durable-grant-store seam (intentional)** — the raw-KV `g14b:grant:<cid>` store this file owns coexists with the Node-encoded `system:CapabilityGrant` store the `GrantReader` consumes; they have different read-shapes (CID-keyed direct-fetch vs zone-prefix scan) and different write-paths (UCAN envelope persist vs Node-encoded grant write); a future "unify the stores" PR is rejected by reference to this paragraph. Three KV prefixes: `b"g14b:grant:"` (the UCAN body, DAG-CBOR-encoded under its content-CID), `b"g14b:revoked:"` (empty-value markers; presence == revoked), `b"g14b:dev_revoke:"` (DAG-CBOR-encoded `DeviceRevocation`, keyed by device-DID bytes). Public surface: `install_proof` / `record_grant` (`record_grant` adds the rate-limit plug call for non-privileged contexts), `revoke` / `record_revocation` / `is_revoked` / `iter_installed_proofs`, `validate_chain_at` / `validate_chain` / `validate_chain_for_audience_at` / `validate_chain_with_durable_revocations`, plus `with_rate_limit_policy` / `rate_limit_policy` / `graph_backend` / `cid_of`. `validate_chain_for_audience_at` fires audience binding *before* the time-window walk so cross-atrium replay rejects with the typed `UcanAudienceMismatch` rather than masking as `UcanExpired`. `cap_err_from_ucan` maps every `UcanError` variant to a typed `CapError`.

---

## 4. Public API surface

The crate re-exports its load-bearing items off the root so consumers write `use benten_caps::{...}` rather than chasing module paths:

- **Trait surface:** `CapabilityPolicy` (object-safe; consumers routinely box behind `dyn CapabilityPolicy`).
- **Context types:** `WriteContext`, `ReadContext`, `PendingOp`, `WriteAuthority` (re-exported from `benten-core`).
- **Concrete policies:** `NoAuthBackend`, `LegacyUcanStubBackend`, `GrantBackedPolicy`, `UcanGroundedPolicy` (native-only), `UCANBackend` (native-only, the *backend*, not a `CapabilityPolicy` itself — composed into `UcanGroundedPolicy`).
- **Grant types:** `CapabilityGrant`, `GrantScope`, the constants `CAPABILITY_GRANT_LABEL` / `GRANTED_TO_LABEL` / `REVOKED_AT_LABEL`.
- **Grant reader:** `GrantReader`, `GrantReaderChain`, `GrantReaderConfig`.
- **Attenuation:** `check_attenuation` (the function), `AsAttenuationScope` (the trait that makes it accept `&str` / `&String` / `GrantScope` / `&GrantScope`).
- **Rate limiting:** `RateLimitPolicy`, `NullRateLimitPolicy`, `InMemoryRateLimitPolicy`, `InMemoryRateLimitPolicyBuilder`.
- **Typed-cap mapping:** `TypedCapGroup`, `typed_cap_for_ucan_claim`.
- **Errors:** `CapError`.
- **Constants + helpers:** `DEFAULT_BATCH_BOUNDARY`, `HlcStampedRefreshEvent`, `emit_refresh_event_for_test`, the `evaluator_delegation` module's `iterate_batch_boundary_for` / `wallclock_refresh_ceiling_for`, and the `testing::` module (back-compat).

Three trait surfaces deliberately stay separate rather than collapsing:

1. `CapabilityPolicy` — the per-write/per-read gate.
2. `GrantReader` — the storage-shaped read handle that `GrantBackedPolicy` consults. Behind a trait so the policy does not take a direct dep on `benten-graph`; the engine implements it against `RedbBackend`.
3. `RateLimitPolicy` — pluggable rate/bandwidth budget, composed alongside `CapabilityPolicy` (not folded in) because rate-limiting and capability-checking are independently configurable concerns. `UCANBackend` carries an `Arc<dyn RateLimitPolicy>` so the same UCAN backend can opt in or out without growing another type parameter through the generic cascade.

---

## 5. Tests inventory

The `tests/` directory carries 30+ integration tests. Grouping by what each pins:

**Trait shape + object safety**
- `policy_trait.rs` — pins `check_write` signature + object-safety; exercises `NoAuthBackend` + `LegacyUcanStubBackend` through `&dyn CapabilityPolicy`.
- `check_write_called_at_commit.rs` — a `CountingPolicy` proves `check_write` fires once per commit, not once per WRITE primitive. Phase-1 TOCTOU named compromise companion.
- `noauth.rs` / `noauth_proptest.rs` / `noauth_still_permits_everything.rs` — exhaustive coverage of the zero-cost default: unchanged by `WriteAuthority`, doesn't reject on adversarial property values, doesn't leak intermittent denials.

**Grants + attenuation**
- `capability_grant.rs` — the typed `CapabilityGrant` shape: struct-literal construction requires `issuer` (the g4-cr-2 principal-confusion fix).
- `grant_uniqueness_on_cid.rs` — content-addressing contract: two byte-identical grants share a CID; any field difference produces a different CID.
- `grant_scope_lone_star.rs` — `GrantScope::parse("*")` rejects with `ScopeLoneStarRejected`; `"*:ns"` accepts.
- `grant_ttl_hlc_duration.rs` — additive-compat shape: `ttl_hlc_duration = None` preserves Phase-1 CIDs.
- `call_attenuation.rs` — segment-wise subset check; positive + negative bounds.
- `proptest_attenuation.rs` — fuzz coverage of the wildcard rules (env-driven `PROPTEST_CASES`; CI 1024, fuzz.yml nightly 10k).

**Grant-backed policy**
- `grant_backed_policy.rs` — `GrantBackedPolicy` coverage including the r6-sec-8 fail-closed default for unstructured contexts + delete-side scope derivation.
- `grant_reader_batch.rs` — `has_unrevoked_grant_for_any` batched-read contract (single backend call, not N).
- `grant_reader_max_chain_depth.rs` — the 64-frame default + boundary firing of `E_CAP_CHAIN_TOO_DEEP`.
- `resume_revocation_denies.rs` — the resume-side §9.13 refresh-point-4 contract: a revoked grant denies at resume.

**TOCTOU + wall-clock refresh**
- `toctou_iteration.rs` — Phase-1 named compromise #1: revocation observable at the next batch boundary, not retroactively, via `E_CAP_REVOKED_MID_EVAL`.
- `wallclock_delegation.rs` — G9-A trait-level delegation: `wallclock_refresh_ceiling` + `iterate_batch_boundary` consulted by the evaluator helpers.
- `wallclock_refresh_typed_error_fires.rs` — end-to-end firing edge for `CapError::WallclockExpired` (qa-r6r1-1 closure).

**Durable UCAN (native-only)**
- `ucan_backend.rs` — durable chain-walk against the redb-backed store + revocation persistence across restart + D2 D-PHASE-3-21 (`host:atrium:publish_view_result`) acceptance + attenuation + the wave-4b `no_longer_returns_not_implemented` symbol-presence pin.
- `ucan_chain_window_narrowing.rs` — R4-FP-R3-B RED-PHASE pins: child cannot widen parent's nbf/exp window; replay-time clock not issuance-time.
- `prop_ucan_window.rs` — 10k-case proptest of `validate_chain_at` time-window edges at the durable seam (defends against off-by-one drift between in-memory and durable layers).
- `device_dispatch.rs` — G16-B canary structural pins: `device_cid` field on `WriteContext` / `ReadContext`; backward-compat when None; the runtime-arm `#[ignore]`'d test that lights up in the post-canary wave.

**Rate-limit policy**
- `rate_limit_policy.rs` — per-actor writes/sec/zone; per-peer bandwidth at Atrium boundary; cross-peer back-pressure via sticky `saturated_in_window` flag.

**Error mapping + display hygiene**
- `error_code_mapping.rs` — every `CapError` variant maps to the right `ErrorCode`.
- `cap_error_display_hygiene.rs` — r6-err-6 + r6-err-9 regressions: Display does not escape-quote payload fields; `DeniedRead` carries the same `(required, entity)` shape as `Denied`.
- `ucan_stub_messages.rs` — Phase-1 `LegacyUcanStubBackend` error routes through `ON_ERROR` not `ON_DENIED`, names Phase 3 + names the `NoAuthBackend` alternative.

---

## 6. Benches inventory

One bench: `benches/wallclock_toctou_refresh.rs` (Criterion, `harness = false`).

Measures the per-iteration cost of the §9.13 dual-source TOCTOU refresh: `elapsed_check_no_refresh` (the common in-window case — one monotonic-clock read + one integer compare) and `refresh_fires_dual_source` (the 300s-boundary case — re-anchor `MonotonicSource::elapsed` + capture the HLC ride-along stamp). Gate policy is **informational** — hardware-independent nanosecond thresholds don't survive CI runner variance. The bench exists to flag a regression where the check path accidentally pulls in the full refresh on every iteration (e.g. a future change that consults the HLC unconditionally).

`[lib] bench = false` is set in `Cargo.toml` so `cargo bench --workspace` doesn't trip libtest CLI rejection of Criterion flags (same rationale as `benten-core`).

---

## 7. Thin-engine + composable-graph philosophy check

The crate's load-bearing posture is: **capability is policy, the trait is the contract, backends are pluggable** (CLAUDE.md baked-in #7). Held against that, the crate is mostly well-respected. The places where the philosophy is honoured + the places where there's friction:

### Well-respected examples

1. **Trait surface is principal-agnostic.** `WriteContext` carries `actor_cid: Option<Cid>` + `actor_hint: Option<String>` + `device_cid: Option<Cid>` + the `pending_ops` batch + the `authority: WriteAuthority` axis. No backend-specific shape is baked in. UCAN-specific principal types (DIDs, signed envelopes, attestation envelopes) stay over the `benten-id` line; this crate sees them only as opaque `Cid` / `String` / typed `Ucan` values.
2. **Object safety is preserved.** Every trait method has a concrete-return signature. No `where Self: Sized` defaults that take `self` by value. `Arc<dyn CapabilityPolicy>` / `Arc<dyn GrantReader>` / `Arc<dyn RateLimitPolicy>` boxing all work — integration tests routinely do this.
3. **GrantReader is a separate trait, not a method on `CapabilityPolicy`.** The shape of storage-read access has nothing to do with the shape of policy decisions; folding them would force every policy to think about storage. The trait split is the layering boundary that keeps `benten-caps` from taking a direct dep on `benten-graph` for the policy types themselves.
4. **RateLimitPolicy is a separate trait, not a method on `CapabilityPolicy`.** The Atrium-boundary bandwidth budget is independently configurable from the capability gate. Composing via `Arc<dyn RateLimitPolicy>` on `UCANBackend` avoids growing another generic parameter.
5. **`#[non_exhaustive]` on `PendingOp` + `CapError`.** Phase-2b SANDBOX `HostFunctionCall` + Phase-3 UCAN-specific error variants land as minor bumps; external `match` expressions must include `_ =>`. The forward-compat discipline mirrors `ErrorCode` and `GraphError`.
6. **Dual durable-grant-store seam intentionally documented.** `backends/ucan.rs` carries a load-bearing module-doc paragraph explaining why the raw-KV `g14b:grant:<cid>` store and the Node-encoded `system:CapabilityGrant` store coexist (per mini-review g14b-mr-3 disposition). The two seams have different read-shapes and different write-paths; a "unify the stores" PR is rejected by reference to this paragraph. This is the right shape because each consumer reads its own seam; they are NOT a write-mirror. This is also exactly the *kind* of decision that needs to be defended in writing, because the surface looks duplicative without the read-shape rationale.

### Frictions / surface concerns to flag

1. **`UcanGroundedPolicy` scope-string surface knows about `cap:typed:*`.** The composed-policy `check_write` literally string-prefix-checks `ctx.scope` for `"cap:typed:"` and routes to `typed_cap_permitted_by_proof` only for that namespace. The non-typed scope falls through to the underlying `GrantBackedPolicy` disposition. This is *correct* — the module doc explains that audience-binding for arbitrary scope strings requires per-actor DID propagation through `WriteContext::actor_hint` which is its own architectural lift (named at `phase-3-backlog §2.3 (i)`) — but the policy surface is currently bifurcated by a string prefix. The right cleanup at the v1 window is to thread audience DIDs end-to-end so the chain-walker can ground arbitrary scopes, not just the typed namespace, and the prefix check becomes a non-special-case routing decision.

2. **`DEFAULT_NOW_SECS = 0` sentinel is a load-bearing inversion that depends on a discipline.** The G16-B-B-rest fail-closed inversion in `UcanGroundedPolicy` keys off `now_secs == 0` AND `chain_has_time_bounds`. The sentinel works because production callers MUST inject a real wallclock via `with_now_for_test` or (eventually) `WriteContext::now` threading. Until that threading lands (`phase-3-backlog §2.3 (i)`), the sentinel is the only operator-visible signal that "no clock was injected." If a future caller picks `0` as a legitimate epoch second by accident, the fail-closed flips on. The right cleanup is `WriteContext::now` threading; the right interim defense is the typed `UcanClockNotInjected` error that surfaces the misconfiguration loudly.

3. **`GrantBackedPolicy` derives scope from label-only via `store:<label>:write`.** Phase-1 scope derivation is `format!("store:{label}:write")` — the policy hard-codes the `"store:"` prefix. This is fine for the Phase-1 CRUD zero-config path (every `crud('post')` write maps to `store:post:write`), but any future namespace (e.g. `host:atrium:publish_view_result`, `zone:posts:write`) bypasses this derivation and must arrive via the `ctx.scope` convenience field already populated. The shape works today because non-CRUD scopes are pre-populated by the engine, but the bifurcation is a smell — there's no single source-of-truth that says "for primary-label X, the required scope is Y." Phase 4-Foundation plugin manifests (CLAUDE.md baked-in #18) would make this worse: a plugin's manifest declares a `requires` cap that may not match the `store:<label>:write` shape at all. Either the policy needs to learn manifest-aware scope derivation, or the manifest-time scope must be threaded through `WriteContext::scope` at the registration boundary.

4. **§13.11 root-cause structural lesson — `BackendGrantReader::revoked_scopes` matches by scope-string equality.** The Phase 4-Foundation fix surfaced that the production gap (silent fail-OPEN on `revokeCapability(grantCid, actor)`) was structurally enabled by the policy reading revocations *by scope string* while the napi surface passed the grant CID *as* the scope. The fix added `Engine::revoke_capability_by_grant_cid` that resolves the CID to a Node and extracts the `scope: Text` property before routing through the existing `(actor, scope)` revoke path. The structural lesson on the `benten-caps` side is that `GrantReader::has_unrevoked_grant_for_scope(scope: &str)` is *correct* — the policy needs to ask "is this scope revoked?" — but the surface is silent on **how the engine builds the scope-string** for any given input. The right v1 cleanup is to make the engine-side grant/revoke API CID-shaped (the canonical handle) and treat the scope-string as derived state. Where this crate could help: a `GrantReader::has_unrevoked_grant_for_grant_cid(cid: &Cid)` companion method whose default impl resolves via the Node store + delegates to `has_unrevoked_grant_for_scope` — making the CID-keyed path the typed API even for callers that don't know the scope-string layout. This would have prevented the silent fail-OPEN at the trait surface, not just at the engine surface.

5. **`GrantBackedPolicy::wildcard_variants` is an O(2^N) enumeration.** For an N-segment required scope, the policy enumerates up to 2^N candidate parent-scope spellings and queries each through `has_unrevoked_grant_for_scope`. Bounded to N ≤ 6 (bails to `[required, "*"]` above that) but still up to 32 reader hits per write check. Cheap when N is 3 (the Phase-1 CRUD case) and the reader is in-process; harder to defend when N grows and the reader queries a remote KV. The right cleanup is for `GrantReader` to expose a wildcard-aware query the policy calls once (matching the `has_unrevoked_grant_for_any` precedent). Surfaced as a Phase-2 carry in the source comment.

6. **No durable-vs-ephemeral assumption baked in.** `CapabilityPolicy` doesn't presume the underlying store is durable. `NoAuthBackend` has no store at all; `GrantBackedPolicy` consults whatever `GrantReader` is injected (could be redb, could be in-memory `BTreeMap` for tests); `UCANBackend<B: GraphBackend>` is parametric over the backend shape. Good.

7. **No coupling to specific Atrium/sync shapes at the policy trait.** `WriteContext::device_cid` is an `Option<Cid>` — the policy can dispatch on it but doesn't require it. `WriteAuthority::SyncReplica` is a variant on the re-exported core enum, so the policy can match on "this write arrived via sync" without this crate caring about iroh / Loro / MST details. The `RateLimitPolicy::check_peer_bandwidth` surface takes `peer: &str` so the rate-limit plug doesn't need to know about iroh's `NodeId` shape either. Good — `benten-sync` is the only crate that translates from network identity to `peer: &str`.

---

## 8. Phase 4-Foundation + Phase 4-Meta expectations

### Phase 4-Foundation (Track B included the §13.11 closure)

The §13.11 namespace-mismatch fix already landed at `pre-3.5-ucan-revocation-observance-fix`: `Engine::revoke_capability_by_grant_cid` resolves the grant Node and extracts the typed `scope` property before routing through the existing `(actor, scope)` revoke path. The `BackendGrantReader::revoked_scopes` surface is unchanged — the fix lives upstream of this crate. The structural lesson (§7 item 4 above) is the carry for the v1 window: making CID-keyed revoke the canonical typed API instead of relying on string-shaped scopes.

### Phase 4-Foundation (plugin manifests + admin UI v0 + extension architecture)

Admin UI v0 will exercise the cap-grant + cap-revoke + UCAN-attenuated-delegation flows end-to-end against the durable backend — that's the production path through `EngineBuilder::capability_policy_ucan_durable` composing `UcanGroundedPolicy` + `UCANBackend` + `GrantBackedPolicy`. Where this crate's surface gets exercised:

- **Plugin manifest `requires` / `shares` halves bind to scope strings.** The Phase-4 plugin trust model (CLAUDE.md baked-in #18) is layered: user-as-root → install-time manifest envelope → runtime delegation within manifest envelope. The runtime delegation step lands on this crate's surface: each plugin gets a DID; UCAN delegations between plugins are validated through `UCANBackend::validate_chain_for_audience_at` (cross-atrium replay defense via `CapError::UcanAudienceMismatch`); the chain-walker's attenuation check enforces "child cannot widen parent." The plumbing the manifest schema needs from this crate is mostly *already shipped* — the chain-walk, attenuation, audience binding, time-window, revocation, all exist. What this crate doesn't yet have:
  - **Manifest-aware scope derivation.** See §7 item 3.
  - **Per-plugin private-namespace caps.** Plugin manifests with `shares=none` for their private namespace need the policy to refuse cross-plugin cap issuance into that namespace. Today the policy is principal-coarse; this becomes a per-plugin-DID concern in Phase 4. Probably an extension axis on `GrantBackedPolicy` (or a wrapper-policy) rather than a trait change, but the design is open.

- **Class B β `Engine::read_node_as` already shipped** (#184 in the pre-v1 cleanup window per CLAUDE.md baked-in #18). The READ-side check on the policy trait (`check_read`) is the seam `_as` calls into. `ReadContext::by_cid_only` is the typed constructor `read_node_as` uses to thread the principal-CID through the policy. The work this crate carries is making sure every concrete `CapabilityPolicy` honours the principal — `NoAuthBackend` permits unconditionally (fine), `GrantBackedPolicy` currently doesn't consult the principal on reads (the wildcard-enumeration matches any unrevoked grant for the scope), and `UcanGroundedPolicy` defers reads to `GrantBackedPolicy`. Phase 4 will need a read-side per-actor check on `GrantBackedPolicy` — likely via the same `actor_cid` field on `ReadContext` that's already a `None`-defaulted axis.

- **Engine-level extensions (CLAUDE.md baked-in #19) are out of scope for this crate.** Engine-level extensions (custom IVM strategies, alternate transports, alternate persistence backends, custom signature schemes) are compile-time linked Rust crates that don't go through `read_node_as` and don't go through `CapabilityPolicy`. They ARE the engine. No work here.

---

## 9. Open questions / unresolved internals

1. **When does `WriteContext::now` threading land?** Named at `phase-3-backlog §2.3 (i)` as the v1-assessment-window deliverable that retires the `DEFAULT_NOW_SECS = 0` sentinel. Until then `UcanGroundedPolicy` requires `with_now_for_test` injection for any chain with time bounds. Two open shape questions: does `now` go on `WriteContext` or on the policy itself? Does the evaluator inject a per-CALL `now` or a per-batch `now`?

2. **Should `GrantReader` grow a CID-keyed companion to its scope-keyed query?** Per §7 item 4 — would have prevented the §13.11 silent fail-OPEN by making the typed handle the CID rather than the scope string. Open question: does this go on the trait (default impl resolves the Node + delegates) or is it a separate `RevokedGrantReader` trait?

3. **Should `wildcard_variants` move into `GrantReader`?** Per §7 item 5 — O(2^N) enumeration on the policy side could collapse into a single wildcard-aware reader call. Open shape: does this break the "reader is opaque to wildcards" property that lets simple test fixtures implement just one method?

4. **Is there a future split between policy-with-grants and policy-with-chains?** Today `UcanGroundedPolicy` composes `GrantBackedPolicy` + `UCANBackend`. The composition is hard-coded to "grant-first, chain-second" with the `cap:typed:*` prefix as the routing key. A more layered shape might be `CapabilityPolicy` impls that can be *stacked* (an `Or<P1, P2>` combinator, or a per-scope-namespace dispatch). Open question: is the stacked shape worth the abstraction cost, or does the current "one composed policy per namespace family" hold up through Phase 4?

5. **`InMemoryRateLimitPolicy` is the only concrete `RateLimitPolicy`.** Per the trait module doc, "distributed token-bucket across Atrium peers" lands at G14-D / G16. Open question: is a single concrete plug enough through v1, or does the Atrium boundary need a distributed-counter shape before plugin-marketplace traffic patterns (Phase 6+) make in-memory accounting laughable? Probably yes for v1; flag for Phase 6 dispatch.

6. **`UCANBackend::iter_installed_proofs` silently skips decode failures.** The source comment names this as a future-hardening axis: "tighten to surfacing a typed `CapError::BackendStorage` when decode fails (would require distinguishing real corruption from forward-compat envelope shapes)." Open question: when forward-compat envelope shapes start arriving (Phase 4 plugin signing? Phase 6 cross-network UCANs?), what's the right disposition for "I can't decode this entry"? Skip-and-log? Fail-closed? Treat as revoked?

7. **`LegacyUcanStubBackend` retention timeline.** Renamed at G21-T2 to disambiguate from `UCANBackend`. The error-routing contract (`ON_ERROR` not `ON_DENIED`) is what's load-bearing; the stub itself is dead weight that survives because some Phase-1 / Phase-2a tests still import it. Open question: when can these tests be retired or migrated to the durable backend so the stub goes away?
