# v1 Frozen Interface Contract — Phase-4-Meta-Core deliverable (TRIAGE SYNTHESIS)

> **Status: POST-BUILD-OUT-WAVE.** Round 0.5 triage-synthesis refreshed
> at the V1-FROZEN-INTERFACE build-out wave (2026-05-23). The 8 cross-
> confirmed FIX-NOW pre-freeze build-out items from the companion
> [`docs/V1-FROZEN-INTERFACE-BUILD-BACKLOG.md`](V1-FROZEN-INTERFACE-BUILD-BACKLOG.md)
> all LANDED at this wave (rows 1-8); the BUILD-AT-FREEZE-WAVE markers
> below are updated to LANDED with concrete file:line citations.
>
> **Source drafts:** `g-core-9/planner-a-architectural-purist` @ `7ef3f279` +
> `g-core-9/planner-b-conservative-minimal` @ `f5aaebb0`.
>
> **Build-out commits** (against `origin/g-core-9/triage-synthesis`):
> - `dd12f394` row 7 (RestrictedSpec + KeyMaterial renames)
> - `5ce8bab6` row 6 (CapabilityPolicy hard-seal via Sealed supertrait)
> - `a9d2753c` row 3 (EncryptionClass enum mint)
> - `7af94d06` row 4 (Engine::walk_share_scope mint + SubgraphSpecWalkFailed ErrorCode)
> - `d2616800` row 5 (MerkleRangeProofBackend → DEFERRED to G-COMP-1 per Option A)
> - `75a1d33a` row 8 bundle (non_exhaustive sweep + wire-format inventory + Compromise #31 [revocation-reach; re-pointed to #62 at F-full per BR-2 — #31 now denotes LAMPS Composite ML-DSA])
> - `fb7c212d` row 1 (cargo-public-api baselines regenerated — 14 of 14 real)
> - `13322df4` row 2 (TS-public-api parity gate workflow + baseline)
>
> **Council-readiness statement:** the doc + as-built surface are ready
> for the 10-lens FREEZE subset iterate-to-convergence council per the
> §1.A C13 contract.

## Authority + scope

This document is the FROZEN-INTERFACE CONTRACT named by
`.addl/phase-4-meta/00-implementation-plan.md` §1.A.FROZEN (15-item spec,
lines 111-148) as the **terminal deliverable of Phase-4-Meta-Core (Exit
Criterion C13)**. It is what Phase-4-Meta-Composing builds against;
nothing in Composing may alter a frozen surface, and a genuine need to do
so is a HALT-AND-SURFACE-TO-BEN event per methodology-r1-5 (NOT an
orchestrator autonomous Core re-open).

The freeze locks at git tag `phase-4-meta-core-close` (HEAD post-G-CORE-9
merge). The release cadence beyond is `phase-4-meta-close` → `v1-beta` →
(independent `ml-dsa`/`ml-kem` audit lands per C11c / NF-2 / C-GM-AUDIT) →
`v1-GM`, per `RATIFIED-pq-default-reframe-2026-05-19.md` §1.

**Authoritative provenance (read these BEFORE proposing any freeze mutation):**

- `.addl/phase-4-meta/00-implementation-plan.md` §1.A C1-C13 + §1.A.FROZEN
  (the 15-item spec).
- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
  (item 15 sub-clauses a-j).
- `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md` (items 6 + 14
  + the audit-gates-GM clause).
- `.addl/phase-4-meta/RATIFIED-crypto-agility-2026-05-18.md` (the
  multiformats-permanent framing distinct from algorithm choice).
- `.addl/phase-4-meta/RATIFIED-prework-forks-2026-05-18.md` (items 8 + 13 —
  sealed-CapabilityPolicy + engine-owns-its-own-tokio-runtime).
- `CLAUDE.md` baked-in items 1, 5, 7 (sealed-discipline refinement), 15, 17,
  18, 19.

## Verification mechanism (workspace-wide)

The freeze is enforceable, not merely declarative. Five structural backstops
fail CI on a frozen-surface mutation:

1. **`cargo-public-api` baseline regeneration + drift test** at
   `crates/benten-engine/tests/cargo_public_api_drift.rs` against
   `docs/public-api/benten-*.{txt,json}`.
   **LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 1 (commit `fb7c212d`).**
   All 14 lib crate baselines regenerated with `cargo +nightly public-api
   --simplified -p <crate>`; total **20,559 LOC** of real public-API
   surface committed (replaces the 11-LOC G20-A3 seed stubs). The 3
   missing baselines (`benten-crypto-suite` 1609 LOC, `benten-drop` 320
   LOC, `benten-platform-foundation` 2557 LOC) ALL minted. CI workflow
   `.github/workflows/cargo-public-api.yml` expanded from 8 → 14 crates.
   The drift gate is now REAL, not a placebo.
2. **TS-side public-API parity gate (#1204)** for `@benten/engine`.
   **LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 2 (commit `13322df4`).**
   Workflow at `.github/workflows/ts-public-api.yml`; baseline at
   `packages/engine/etc/public-api.txt` (403 LOC; extract-from-.d.ts
   structural diff covering all 14 `dist/*.d.ts` files). Migration to
   `@microsoft/api-extractor` is named for v1-Composing (the workflow
   + baseline-file location ARE the migration seam — swap-in is
   contained).
3. **§3.5g cross-language rule-mirror scanners** at
   `scripts/drift-detect-error-variant-mirror.ts` (item 6 of §3.5g, the
   `feedback_pub_error_variant_first_class_mirror` ratification
   2026-05-24). Every variant of any Rust error type crossing
   public/napi/wire surfaces has a first-class `ErrorCode` entry + TS
   mirror; the scanner runs in CI and rejects asymmetric variants.
4. **Workspace `missing_docs` sweep** = `cargo nextest run -p
   phase-3-workspace-tests --test missing_docs_workspace` (per
   `feedback_workspace_missing_docs_test_invocation`). Mandatory
   pre-push + CI lane. Every `pub` item carries `///` docs at freeze;
   post-freeze additions inherit the gate.
5. **CATALOG_VARIANT_COUNT exhaustive-match dual-tripwire** at
   `crates/benten-errors/tests/stable_shape.rs::catalog_variant_count_matches_enum`.
   **CATALOG_VARIANT_COUNT = 192 at G-CORE-9 build-out wave HEAD**
   (was 191 at `ae7cd3d5`; +1 = `SubgraphSpecWalkFailed` minted at
   G-CORE-9 V1-FROZEN-INTERFACE row 4 commit `7af94d06`). Adding
   or removing an `ErrorCode` variant without updating the list fails to
   compile or fails the runtime length assertion.

> **NEW pim-N candidate REJECTED at triage** (Planner-A's
> per-`pub`-declaration `// FROZEN: re-open requires Ben sign-off`
> marker comment + CI lint scanner): the existing `cargo-public-api`
> baseline + #1204 parity gate ARE the structural backstop per spec
> methodology-r1-5. Per Planner-B's reasoning: comment-lints add
> ceremony without adding structural enforcement the two real CI gates
> don't already provide. **Recorded as candidate pim-N for future
> iteration if structural-backstop alone proves insufficient**; not
> ratified at G-CORE-9.

---

## 1. §8-A visibility cluster — FULLY LANDED at v1-beta (R6 R1 FP-A Bundle F2, 2026-05-24)

**Orchestrator distinctive-angle decision (RATIFIED 2026-05-18): Planner-A
wins — aggressive tighten.** Matches spec item 1 ratification "DECIDED
`pub`→`pub(crate)` + rename + drop `_for_test`".

**R6 R1 FP-A status (2026-05-24).** The binary-side tighten + napi cascade
**LANDED in full** at R6 R1 FP-A Bundle F2 per Ben PM ratification of
HARD RULE 12 ("if we're going to want to do them all eventually, then
I say do the full ~13-site cascade now") over the prior G-CORE-9 R1
triage L2-BLK-1 path-(b) defer. The four methods are now
`pub(crate)` with their v1-GM target names; the napi binding migrates
to `read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, ...)` per CLAUDE.md
baked-in #18; ~80 sibling-crate integration tests are preserved via
test-helper re-exports under `cfg(any(test, feature = "test-helpers"))`
in `crates/benten-engine/src/testing.rs`. Row D-7 in
`docs/V1-FROZEN-INTERFACE-DEFERRED.md` is RETRACTED / CLOSED.

**Per discipline at v1-beta**, external consumers needing principal-bearing
read use `Engine::read_node_as(principal, cid)`. Benten-owned boundary
callers (e.g. the napi binding) that lack a caller-supplied principal
use `Engine::read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, cid)` — the
always-on sentinel constant minted in
`crates/benten-engine/src/internal_principal.rs` + re-exported at the
crate root. External `benten-engine`-as-library consumers requiring
direct un-attributed engine reads (NOT routable through `read_node_as`)
must surface to Ben for sealed-trait extension per the Composing-phase
escape valve below.

**Frozen surfaces (LANDED at v1-beta via R6 R1 FP-A Bundle F2; per
RATIFIED-prework-forks-2026-05-18.md §8-A option (a)):**

- `crates/benten-engine/src/engine_crud.rs::Engine::get_node` →
  `pub(crate) fn read_node(&self, cid: &Cid) -> Result<Option<Node>,
  EngineError>` **renamed to `read_node` to remove the un-attributed
  semantic from the name surface**. The public `Engine::read_node_as`
  (at `crates/benten-engine/src/engine_wait.rs::Engine::read_node_as`)
  carries the principal-bearing semantic.
- `crates/benten-engine/src/engine_wait.rs::Engine::put_node` →
  `pub(crate) fn put_node_inner` (disambiguating rename so the cfg-gated
  test-helper re-export under `Engine::put_node` in `crate::testing`
  preserves sibling-crate integration tests).
- `crates/benten-engine/src/engine_wait.rs::Engine::get_node_label_only`
  → `pub(crate) fn read_node_label_only`
  (renamed; un-attributed label-only read; engine-internal only).
- `crates/benten-engine/src/engine_wait.rs::Engine::resolve_subgraph_cid_for_test`
  → `pub(crate) fn resolve_subgraph_cid_inner` (DELETED from the public
  surface entirely; test-helper re-export under the historical
  `_for_test` spelling lives in `crates/benten-engine/src/testing.rs`).
  Public surface MUST NOT carry `_for_test` suffixes at v1-GM (a
  `_for_test` `pub fn` is a red-flag — either real public API or
  belongs in `testing` module). **R6 R1 FP-A Bundle F1 closure (2026-05-24):**
  the workspace cfg-gating sweep is COMPLETE — 70+ `pub fn .*_for_test*`
  declarations across `benten-caps` / `benten-crypto-suite` /
  `benten-drop` / `benten-core` / `benten-sync` / `benten-graph` /
  `benten-eval` / `benten-engine` / `benten-id` /
  `benten-platform-foundation` / `tools/benten-dev` are gated under
  `#[cfg(any(test, feature = "testing"))]` (or
  `feature = "test-helpers"` for benten-engine). 14 production-shaped
  items (whose `_for_test*` name is a misleading suffix; production
  code paths consume them) remain `pub` per the EXEMPT_PUB_ITEMS table
  at
  `tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs` +
  V1-FROZEN-INTERFACE-DEFERRED.md ~~Row D-22~~ EXEMPT_PUB_ITEMS section
  (the row is RETRACTED / CLOSED; preserved for forensic context). The
  no-regression pin at the test path above blocks any future
  re-introduction of newly-public ungated `_for_test*` items. Row D-22
  closure means the v1-beta cargo-public-api baselines no longer carry
  the bulk of the `_for_test*` surfaces; renaming the 14 exempts is a
  v1-GM-target cleanup tracked separately.
- `crates/benten-engine/src/engine.rs::Engine::caps` (`fn caps(&self) ->
  &EngineCapsHandle`) stays `pub`; this is the canonical cap-mutation
  surface per the §4.69-ALREADY-SHIPPED ground-truth. No `Engine`-direct
  cap-mutation method may regress (freeze invariant; orchestrator-
  mechanical no-regression test pin per build-backlog).

**What "frozen" means here (at v1-beta, FULLY LANDED at R6 R1 FP-A
Bundle F2):**
- The four methods are `pub(crate) fn read_node` /
  `pub(crate) fn read_node_label_only` / `pub(crate) fn put_node_inner` /
  `pub(crate) fn resolve_subgraph_cid_inner`. External callers MUST go
  through `read_node_as(principal, cid)` (or
  `read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, cid)` for un-attributed
  Benten-owned-boundary reads).
- Adding NEW cap-mutation methods to `Engine` (other than via the
  `caps()` handle) is a HALT-AND-SURFACE event — `Engine::caps()` is
  the canonical cap-mutation surface at v1-beta and beyond.
- The `cargo-public-api` baseline at `docs/public-api/benten-engine.txt`
  enforces the visibility: any post-fix-pass re-`pub`-ing of the four
  methods fails the drift test.
- Behaviorally: the engine-internal callers (IVM, sync, view
  materialization, audit) keep using the un-attributed pathway with zero
  overhead — the tighten was a visibility-only change, not a behavior
  change.
- The `Engine::caps()` handle pattern is the SemVer-locked cap-mutation
  organizing principle (item 1's §4.69 sub-clause; ALREADY SHIPPED at
  `ed03729a`).

**What's NOT frozen:**
- The internal implementation behind `read_node_as` (principal-routing,
  cap-policy consultation, namespace-partition lookup) may change post-v1
  as long as the function signature + semantic contract is preserved.
- `pub(crate)` internal helpers and their `pub(super)` re-exports inside
  `benten-engine` are NOT frozen.

**Verification mechanism:**
- `cargo-public-api` baseline `docs/public-api/benten-engine.txt`
  (regenerated at R6 R1 FP-A Bundle F1.e) carries the locked `pub`
  set with the 6 §8-A surfaces tightened out and the new
  `internal_principal::ENGINE_INTERNAL_PRINCIPAL_CID` sentinel
  surface added. Re-`pub`-ing fails the
  `.github/workflows/cargo-public-api.yml` drift gate.
- **napi cascade migration LANDED at R6 R1 FP-A Bundle F2.**
  `bindings/napi/src/lib.rs::Engine::get_node` migrated to
  `self.inner.read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, &parsed)`;
  no other `engine.get_node` / `engine.put_node` direct calls remain
  on the napi production surface.
- **Test-site preservation:** ~80 sibling-crate integration tests
  that called `Engine::get_node` / `put_node` / `get_node_label_only` /
  `resolve_subgraph_cid_for_test` under their historical spellings
  continue to compile via the test-helper re-export `impl
  crate::Engine` block at the end of
  `crates/benten-engine/src/testing.rs` (cfg-gated by the existing
  `#[cfg(any(test, feature = "test-helpers"))]` attribute on
  `pub mod testing`). No per-test migration was needed.
- No-regression test pin (forthcoming Composing-phase build-out):
  `crates/benten-engine/tests/g_core_9_engine_no_direct_cap_mutation.rs`
  will use `cargo-public-api` output to assert ZERO cap-mutation
  methods on `Engine` (other than `caps()`).

**Composing-phase escape valve:**
A Composing-time discovery that genuinely needs un-attributed
`Engine::read_node` access (NOT routable through `read_node_as` for a
documented reason) is a HALT-AND-SURFACE-TO-BEN event. Likely outcome on
surfacing: the `read_node_as(principal=ENGINE_INTERNAL_PRINCIPAL_CID, ...)`
pattern covers it without re-opening the freeze.

---

## 2. Class-B-β visibility — `read_node_as` is the canonical principal-bearing read

**Frozen surfaces:**
- `crates/benten-engine/src/engine_wait.rs::Engine::read_node_as`
  — `pub fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, EngineError>` —
  the public principal-bearing read API per CLAUDE.md baked-in #18. **The
  ONLY public read pathway for non-trusted principals.**
- `crates/benten-engine/src/engine.rs::Engine::call_as` — the existing
  Phase-2a precedent for principal-attributed call dispatch; signature
  parity with `read_node_as` is the canonical mirror.
- napi MUST NOT expose `read_node_as` directly (per
  `bindings/napi/INTERNALS.md:53`); napi exposes only the un-attributed
  path's wrappers; the `_as` path flows through `call_as`.

**What "frozen" means here:**
- The `(principal: &Cid, cid: &Cid)` signature shape is locked. A
  `(principal: SomeNewType, cid: &Cid)` rewrite is a breaking change and
  requires re-opening the freeze.
- The semantic contract — "the engine consults
  `CapabilityPolicy::check_read` with the supplied `ReadContext {
  principal, target_cid, .. }` before returning the Node" — is part of
  the freeze.
- The CLAUDE.md #18 plugin-trust contract requires this exact shape:
  plugin authors author graph nodes; the evaluator is the only caller of
  `_as`. Re-naming, signature changes, or making napi expose `_as`
  directly would break #18.
- The TODO at `crates/benten-engine/src/engine_wait.rs::Engine::read_node_as` referencing Class-B-β alpha-shaped
  stubs is closed (shipped at PR #184).

**What's NOT frozen:**
- The internal routing inside `read_node_as` (principal-DID parsing,
  UCAN chain validation, namespace-partition lookup, IVM-cache
  consultation) can evolve as long as the contract holds.

**Verification mechanism:**
- `cargo-public-api` baseline locks the signature.
- `crates/benten-engine/tests/engine_read_node_as_put_node_pre_v1_closure.rs`
  + `crates/benten-engine/tests/admin_ui_v0_shell_routes_through_engine_read_node_as_for_cap_scoped_reads.rs`
  + `crates/benten-engine/tests/admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`
  (existing test family) carry the behavior pins.
- A documentation pin in `engine_wait.rs`'s rustdoc cites CLAUDE.md #18
  verbatim.

**Composing-phase escape valve:**
A new public principal-bearing READ shape (e.g. `read_subgraph_as`,
`stream_node_as`) is an ADDITIVE pub item — fine if it CO-EXISTS with
`read_node_as`. Removing or changing `read_node_as`, or exposing it over
napi directly, is a HALT-AND-SURFACE event (CLAUDE.md #18 explicit
re-open).

---

## 3. Backend-trait SemVer-locks (§4.60/4.61/4.62/4.63/4.64/4.43)

**Frozen surfaces (per §1.A.FROZEN item 3):**

| Sub-clause | Surface | Frozen shape |
|---|---|---|
| §4.60 | `crates/benten-graph/src/graph_backend.rs:238` `GraphBackend::Transaction::run<F, R>` | The closure-shaped transaction surface stays as-shipped; `run<F: FnOnce(&mut Transaction) -> R, R>` |
| §4.61 | `GraphBackend::snapshot()` + `register_subscriber()` | Both **DECIDED infallible** (`-> SnapshotHandle` and `-> ()`); fail-modes route through the typed `GraphError` channel on dependent operations, NOT through `Result` on these allocation methods. Lock as-shipped per `c4a37bb`-era baseline. |
| §4.62 | `crates/benten-graph/src/backends/blob_backend_trait.rs:120` `BlobBackend` | **DECIDED additive-default** (NOT a split). The trait carries `put_blob`/`get_blob`/`has_blob` with `Send + Sync + 'static`; future additive methods land as defaulted methods. |
| §4.63 | `crates/benten-graph/src/backend.rs:306` `KVBackend: Send + Sync` | **DECIDED sync** (NOT RPITIT). RPITIT adds 2024-edition feature-gate complexity v1-beta cannot absorb; future-Composing-async migration is an additive `AsyncKVBackend` trait. |
| §4.64 | `crates/benten-sync/src/transport_trait.rs:85` `Transport` + `TransportEndpoint` + `TransportConnection` family | `Transport` family stays in `benten-sync` per §8-B (b). The trait surface is `pub` + `Send + Sync + 'static`. **`MerkleRangeProofBackend` trait DEFERRED to G-COMP-1 per V1-FROZEN-INTERFACE row 5 outcome (commit `d2616800`)** — Option A per Planner-B; verified not-built at HEAD; freezing a phantom shape is overcommit. Tracked at `docs/future/phase-4-backlog.md §4.64` (the named-NOW destination per HARD RULE 12 clause-(b)). The §4.64 row received an explicit verify-or-defer outcome paragraph at the G-CORE-9 row 5 commit. |
| §4.43 | `WriteContext` / `ChangeEvent` / `GraphError::TxAborted` `#[non_exhaustive]` | **APPLY `#[non_exhaustive]` to all three.** `benten_graph::WriteContext` (in `crates/benten-graph/src/lib.rs`) is currently MISSING the attribute (verified HEAD). `benten_core::change_stream::ChangeEvent` ALREADY has it — KEEP. `benten_graph::GraphError::TxAborted` per-variant `#[non_exhaustive]` — APPLY defensively. The freeze MUST not ship without these. Couples to item 5 + item 11; closes atomically in the G-CORE-9 wave. |

**What "frozen" means here:**
- Trait method signatures + `Send + Sync + 'static` bounds + the
  `async`/`sync` posture per row.
- The `#[non_exhaustive]` per-row decision is structural — applying it
  post-v1 is breaking (the discipline item 11 enforces).
- The trait's CRATE PLACEMENT (graph vs sync vs caps) is part of the
  freeze: moving a trait crate-side post-v1 is a wire/type-import break.

**What's NOT frozen:**
- Default-method bodies inside each trait may evolve.
- Internal helper types referenced only inside the trait's method
  signatures (e.g. iteration-result handle types) are governed by their
  own per-type freeze decisions (item 11 sweep).
- Backend-impl-specific signatures (constructor sigs, redb-specific
  config types) — NOT in the GraphBackend trait surface; out of scope at
  v1-beta freeze.

**Verification mechanism:**
- `cargo-public-api` baselines for `benten-graph` + `benten-sync` +
  `benten-caps`. Adding/removing a non-defaulted trait method = baseline
  delta = CI failure.
- `cargo +stable clippy --workspace --all-targets -- -D warnings`
  catches `non_exhaustive` ABI-break candidates via the
  `non_exhaustive_omitted_patterns` lint.
- A test that constructs `Arc<dyn GraphBackend>` for the three shipped
  backends compile-pins the trait's object-safety.

**Composing-phase escape valve:**
A new defaulted trait method is ADDITIVE (cargo-public-api accepts; the
`#[non_exhaustive]` discipline propagates). A non-defaulted method
addition OR a signature change OR a `Send`/`Sync`/`'static` bound change
is a HALT-AND-SURFACE event.

---

## 4. D2 v1-canonical-bytes contract frozen as version 1 (P-III Ben decision-point)

**This is a scheduled Ben decision-point** (§1.A.FROZEN item 4, marker
"P-III BEN DECISION-POINT the plan SCHEDULES, never makes"). The G-CORE-9
freeze wave SURFACES the decision; Ben makes it.

**Frozen surfaces (the wire-byte lock; P-III scheduled here per §8-F):**

- `crates/benten-graph/src/backends/snapshot_blob.rs:125` —
  `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 2` (locked per §8-B-(i); already
  applied 2026-05-22 per ground-truth at HEAD; G-CORE-6b discharged the
  1→2 bump in-place per "no-users-yet" P-III override at `#1331
  ecc5111e`).
- The Phase-1 canonical Node/Edge DAG-CBOR encoding family (CIDv1 +
  BLAKE3-256 + multihash `0x1e` + multicodec `0x71`), per CLAUDE.md
  baked-in #5.
- The MerkleRangeProof v2 wire shape (§8-B mode-(b)) — bytewise locked
  including field order and CBOR encoding (IF the trait ships at G-CORE-9
  per build-backlog row 5; otherwise wire shape is named-but-deferred).
- The per-chunk AEAD wire layout — chunk_size = `IROH_BLOCK_SIZE = 16384`
  (item 15(g)) — locked at `crates/benten-crypto-suite/src/aead.rs:52`.
  AAD layout binds `(plaintext_cid: &[u8], chunk_index: u64, total_chunks: u32)`
  per `crates/benten-crypto-suite/src/aead.rs::aad_per_chunk` (4-segment
  layout: domain-tag || plaintext_cid || chunk_index LE || total_chunks LE).
  The `total_chunks` segment closes the cross-chunk-truncation attack
  (an attacker who truncates a 10-chunk ciphertext to 5 chunks cannot
  fabricate per-chunk AAD-matching tags because the seal-time AAD
  committed to `total_chunks=10`). Wire layout pinned at
  `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs::aad_per_chunk_canonical_layout_pinned`
  + behavioral truncation/inflation pins at
  `crates/benten-graph/src/aead_wrap.rs::tests::{cross_chunk_truncation_fails, cross_chunk_inflation_fails}`.
  **R6 R1 fix-pass note:** the prior G-CORE-9 R1 triage Fork 1 disposition
  (defer `total_chunks` to G-COMP-1, retain 2-tuple AAD at v1-beta) was
  RETRACTED at R6 R1 per the L1/L3 cross-confirmation cluster + the
  spec text "binds plaintext_cid + chunk_index + total_chunks"
  brought into alignment with the as-shipped code. See
  `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-15 revision-history.
- Sentinel CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`
  remains the canonical Phase-1 golden fixture and MUST round-trip
  identically under v1 canonical bytes.

**What G-CORE-9 produces here:**

1. A documented **inventory** of every wire-format-bearing surface
   (CBOR envelopes; AEAD wrap envelopes; UCAN-Varsig envelopes; DropBundle
   envelope; SnapshotBlob; TwoCidStore mapping format).
2. The current **explicit format-version discriminator** for each (e.g.
   `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 2`; `DropBundleVersion` enum at
   `crates/benten-drop/src/lib.rs`).
3. The list of surfaces that **DO NOT YET HAVE** a byte-pin test in CI
   (the wire-format pre-flight gap to close — added before tag).
4. **The Ben sign-off path** — `docs/V1-WIRE-FORMAT-INVENTORY.md` IS the
   Ben-decision deliverable per G-CORE-9 R2 L18-r2-4 ratification (path b).
   Ben signs the freeze decision via the inventory's own
   §"P-III Ben decision-point" section after reviewing the inventory + the
   §8-B-(i) reasoning. (The earlier framing proposed a separate
   `V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md` artifact; path b collapses the
   distinction — the inventory serves the dual role of enumeration + sign-off
   site. See `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-16 for the
   ratification rationale.)

**What "frozen" means here:**
- BYTEWISE: a one-bit change to any encoded value (Node, Edge,
  SnapshotBlob, MerkleRangeProof, per-chunk AEAD ciphertext, UCAN-Varsig
  v1 header, AuthorizationGrant CBOR, Drop bundle CBOR) is a P-III
  re-decision Ben must make. Not orchestrator-autonomous, not a refactor
  side-effect.
- The format-version discriminator (`schema_version: u32`) is the
  migration affordance: bumping it is the explicit re-open mechanism.
- Old codepoints / old format versions are decode-supported FOREVER per
  item 14 (never-strand-content).

**What's NOT frozen:**
- The Rust struct representation in memory (we may add `#[serde(skip)]`
  fields, change field ORDER inside the struct as long as `Serialize`
  order is locked, refactor the encoder internally).
- Newly-written content uses whatever new format the schema-bump
  introduces; no in-place migration on existing redb partitions
  (immutable content-addressed objects per item 14).
- **The wire format for any surface that does not yet have a byte-pin
  test** cannot legitimately freeze; even an inventory entry without a
  fixture is just narrative.
- **MerkleRangeProofBackend's proof bytes** — the trait is not built at
  HEAD; build-backlog row 5 verifies-or-builds before freeze.

**Verification mechanism:**
- Byte-pin tests under `tests/canonical_bytes_v1_*.rs`. **G-CORE-9 R1 Bundle 5
  PARTIAL outcome (2026-05-24):** 2 of 8 hex-pinned tests landed at v1-beta —
  (a) `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs::codepoint_table_integer_values_pinned`
  (hex-pins the 9-codepoint integer table) + (b) the per-chunk AAD layout pin
  in the same file. The remaining 6 hex-pinned tests (SnapshotBlob v2,
  per-chunk-AEAD canonical hex, UCAN-Varsig v1 header, AuthorizationGrant CBOR,
  Drop bundle CBOR, encryption envelope per codepoint, signature envelope per
  codepoint) are DEFERRED to G-COMP-1 per
  `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-9. **The 6 deferred surfaces
  remain byte-stable at v1-beta via roundtrip + constant-position +
  format-version-discriminator pins per the L11 lens substantively-covers
  finding** — full hex-pinned-bytes regression-defense is the deferred half.
- `crates/benten-graph/tests/redb_schema_version_envelope_pin.rs` +
  `crates/benten-graph/tests/in_memory_backend_equiv_to_redb.rs` +
  `crates/benten-graph/tests/kvbackend_conformance.rs` cover the redb
  on-disk format.
- **CI inventory-walking lane** — DEFERRED to G-COMP-1 per Row D-9; the
  drift-detect substrate exists (`.github/workflows/cite-drift.yml` +
  `.github/workflows/cargo-public-api.yml`) but the
  inventory-walk-asserts-byte-pin-exists discipline is the G-COMP-1
  follow-up. At v1-beta the inventory at
  `docs/V1-WIRE-FORMAT-INVENTORY.md` is authored + tracked + the Ben P-III
  decision-point sign-off path is the inventory's own §"P-III Ben
  decision-point" section.

**Composing-phase escape valve:**
ANY frozen-byte mutation is a P-III Ben decision-point — HALT-AND-
SURFACE-TO-BEN, with options + prediction per the standing surface
discipline. The mutation lands in Core re-open (a Core re-open IS
allowed; it is announced, deliberate, never silent). This is the
strongest backstop in the entire freeze contract because wire-format
breaks are non-recoverable post-`v1-GM`.

---

## 5. `WriteContext` shape frozen (#989 / G-CORE-1 canary output)

**Frozen surfaces (the full struct `benten_graph::WriteContext` in `crates/benten-graph/src/lib.rs`):**

```rust
#[non_exhaustive]  // ← LANDED at G-CORE-9 row 8c (commit 75a1d33a; couples item 11)
pub struct WriteContext {
    pub label: String,                 // primary label / system-zone prefix check
    pub is_privileged: bool,           // engine-API-only path marker
    pub authority: WriteAuthority,     // Phase-2a G2-B authority enum
    pub namespace_did: Option<Cid>,    // ← G-CORE-1 / #989 partition seam
}
```

- All four fields `pub`; `Default` impl carries `namespace_did = None`
  (the legacy un-namespaced keyspace; byte-identical to pre-#989).
- `WriteContext::with_namespace_did(self, namespace_did: Option<Cid>) -> Self` builder.
  **R6 R2 FP-B (L5-r2-MAJOR-1 closure):** the parameter is `Option<Cid>`
  (NOT bare `Cid`) — pass `None` to explicit-downgrade a previously-
  partitioned context to legacy un-namespaced keyspace, `Some(did)` to
  confine the write to the per-DID partition. The `Option<_>` shape is
  the deliberate v1-beta-frozen contract; passing `None` on a
  per-principal write site is a footgun (silently routes to the legacy
  un-partitioned keyspace). **Construction discipline:** any production
  caller threading a per-principal write MUST use `Some(principal_cid)`
  and treat `None` as the legacy/test-fixture-only shape.
- `WriteContext::namespace_did(&self) -> Option<&Cid>` accessor.
- The C1 cross-DID non-leak invariant (doc-block on `benten_graph::WriteContext` in `crates/benten-graph/src/lib.rs`) — structural: keys under per-DID prefix derived from
  `Cid::as_bytes()`; never collide with legacy `n:`/`e:`/`es:`/`et:`
  prefixes.

**`#[non_exhaustive]` LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 8c
(commit `75a1d33a`).** Applied on `benten_graph::WriteContext` (in `crates/benten-graph/src/lib.rs`).
7 test-construction sites migrated to the existing builder pattern
(`WriteContext::new(label)` + `.with_authority(...)` +
`.with_namespace_did(did)`; `WriteContext::privileged_for_engine_api()`
for engine-privileged paths) — see commit body for full file list.
Workspace builds clean; downstream construction via builder is the
v1-beta-and-forward contract.

**What "frozen" means here:**
- Field-set frozen: no removal, no rename, no type-change of any of the
  four fields.
- `Default` semantics frozen: `namespace_did = None` means the legacy
  keyspace (byte-identical to pre-#989).
- Builder + accessor names frozen.
- The cross-DID non-leak invariant is part of the type's contract; a
  Composing-time backend impl that breaks the invariant = test fails =
  HALT.

**What's NOT frozen:**
- Future additive fields land at the `#[non_exhaustive]` tail (e.g. a
  defaulted `tenant_id: Option<TenantId>` post-v1) — that's the entire
  reason for the attribute.
- The `RedbBackend::scoped(did)` constructor signature — backend-impl
  surface, not part of the trait. Covered by item 3.
- Whether other backends (BrowserBackend, SnapshotBlobBackend) implement
  scoped-views — those backends fail-closed on `Some(namespace_did)` per
  `benten_graph::GraphError::NamespacedWriteUnsupported`; the v1-beta lock is on the failure-mode shape.

**Verification mechanism:**
- `cargo-public-api` baseline `docs/public-api/benten-graph.txt`
  (regenerated at the G-CORE-9 build-out wave per build-backlog row 1).
- `crates/benten-graph/tests/tf1_989_cross_did_partition_isolation.rs`
  (G-CORE-1 canary pin; the historical `tf1_write_context_namespace_did_*.rs`
  glob family was consolidated into this single pin file post-G-CORE-1
  landing — corrected per R6-R2-FP-C cite-grep-verify discipline).
- A new no-regression test that scans `WriteContext` builder code paths
  for accidental `namespace_did = None` overrides post-write.

**Composing-phase escape valve:**
Field addition is ADDITIVE-with-`#[non_exhaustive]`; the new field must
have a default + a builder. Field removal/rename is HALT-AND-SURFACE.

---

## 6. #1300 signature + #1301 encryption boundaries frozen (PQ-hybrid DEFAULT + full swap matrix)

Per `RATIFIED-pq-default-reframe-2026-05-19.md` §1-2 + §1.A.FROZEN item 6 +
CLAUDE.md baked-in #5 (the multiformats-permanent framing).

**Frozen surfaces (LOCK THE FRAMING + CODEPOINT TABLE; algorithms behind
each codepoint = SWAPPABLE within the framing):**

1. **`benten-crypto-suite` integration-crate boundary** — the crate IS the
   seam; the public-API of `benten_crypto_suite` (the `pub use` re-export
   block in `crates/benten-crypto-suite/src/lib.rs` — `AeadEnvelope` /
   `AeadError` / `AeadKeyMaterial` / `CipherSuiteCodepoint` / `HashCodepoint`
   / `SigCodepoint` / `CryptoError` / `UnsupportedAlgorithm` / `VerifyError`
   / `HashSeam` / `HybridSignature` / `SignatureSuite` / `SuiteConfig` /
   `StructuralKdfKey` / `derive_root` / `derive_step` / the `swap_matrix`
   exports) is frozen.
   `cargo-public-api` enforces.

2. **Codepoint table integer values** — PERMANENT (never reuse a value
   for a different algorithm; the #1341 0x647b drift incident established
   this discipline in writing):

   | Surface | Constant | Value | Status at v1-beta |
   |---|---|---|---|
   | Hash | `HashCodepoint::BLAKE3` | `0x1e` | LIVE, default |
   | Hash | `HashCodepoint::SHA2_512_256` | `0x1015` | reserved fallback |
   | Hash | `HashCodepoint::SHA3_256` | `0x16` | reserved fallback |
   | Sig | `SigCodepoint::HYBRID_ED25519_MLDSA65` | `0x0001` | LIVE, **default (byte-faithful IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`, OID `1.3.6.1.5.5.7.6.48`; `draft-ietf-lamps-pq-composite-sigs-19` + test-vector commit `f0627ab3`; wire `mldsaSig(3309) ‖ tradSig(64)` = 3373 B ML-DSA-first, NO commitment trailer; both halves MUST verify)** |
   | Sig | `SigCodepoint::CLASSICAL_ED25519` | `0x0002` | LIVE, non-default downgrade |
   | Sig | `SigCodepoint::HYBRID_MLDSA65_SLHDSA` | `0x0003` | reserved swap-matrix arm (NF-1 end-state; **typed-rejected by default** at `SigCodepoint::resolve` + `SignatureSuite::resolve_codepoint` + `varsig.rs::decode_payload`; reachable only via `SwapMatrix::try_pure_pq_sole_trust_path()` audit-gated constructor per C11b safety gate; mirrors 0x647c framing) |
   | Cipher | `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` | `0x647a` | LIVE, **default (X-Wing-style combiner vendored ~30 LOC; ChaCha20-Poly1305 bulk)** |
   | Cipher | `CipherSuiteCodepoint::CLASSICAL_X25519` | `0x6400` | LIVE, non-default classical-only downgrade |
   | Cipher | `CipherSuiteCodepoint::NONE_PLAINTEXT` | `0x0000` | LIVE, non-default plaintext-partition downgrade |
   | Cipher | `CipherSuiteCodepoint::HYBRID_MLKEM768_HQC` | `0x647b` | reserved-unimplemented (NF-1 KEM end-state; FIPS 207-final build-gated) |
   | Cipher | `CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY` | `0x647c` | reserved swap-matrix arm (NOT default; pre-FREEZE bundle #1342 mint per Ben morning queue item 1; **typed-rejected by default — gated by `AUDIT_LANDED_PURE_PQ_FLAG` per the C11b safety gate**) |

3. **Swap-matrix constructors** at
   `crates/benten-crypto-suite/src/swap_matrix.rs::SwapMatrix`:
   `v1_beta_default()`, `classical_only()`, `no_encryption_public_class()`,
   `non_pq_encryption()`, `try_pure_pq_sole_trust_path() -> Result<Self,
   SwapMatrixError>`. **The `try_*` shape encodes the C11b safety gate at
   the type level** — pure-PQ cannot be a sole-trust-path config without
   an explicit `AUDIT_LANDED_PURE_PQ_FLAG = true` flip.

4. **Wire envelope structure** — the `AeadEnvelope` shape at
   `crates/benten-graph/src/aead_wrap.rs` + the `WrappedKey` wire form +
   the hybrid-sig byte-faithful IETF LAMPS composite construction
   (`id-MLDSA65-Ed25519-SHA512`: shared `M'`/ctx=Label binding, wire
   `mldsaSig(3309) ‖ tradSig(64)` = 3373 B ML-DSA-first, NO commitment
   trailer; both halves MUST verify). Byte-pinned at item 4 — note item 4's
   signature-envelope **hex**-golden is the DEFERRED half (Row D-9 →
   G-COMP-1); at v1-beta the signature wire is locked by the
   structural/size + roundtrip + constant-position + format-version pins,
   not a deterministic hex byte-golden.

5. **Multi-device key-wrap/recovery envelope SHAPE** — frozen as part of
   #1301 per item 6. Recovery PROTOCOL choice (Shamir / social / hardware
   / MLS-style) stays G-COMP-3 v1-assessment-window; the ENVELOPE SHAPE
   around the wrap is frozen here so a recovery-protocol choice doesn't
   require re-opening the freeze.

6. **Typed-reject discipline** — `UnsupportedAlgorithm::{Signature,
   CipherSuite, Hash}` error variant on every codepoint dispatcher;
   FAIL-CLOSED on unknown codepoint (NEVER silent fallback; Veilid + MLS
   + Nostr NIP-44 normative precedent; age's silent-ignore is the
   explicitly-REJECTED outlier).

7. **No-hardcoded-sizes invariant** — at v1-beta this is a v1-PRODUCTION-
   correctness property because hybrid ML-DSA/ML-KEM is the default path.
   A test pin scans the crypto-suite + every consumer crate for `[u8; 32]`
   / `[u8; 64]` etc. that should be `Vec<u8>` per the agile-size
   discipline.

8. **Codepoint-typed constructors** — `SigCodepoint` / `CipherSuiteCodepoint`
   / `HashCodepoint` are wrapper structs around `u16` (`pub struct
   SigCodepoint(pub(crate) u16)`). The `from_raw(raw: u16) -> Self`
   constructor at codepoint.rs:64 is `pub` for deserializer use, paired
   with `resolve()` → `Result<(), UnsupportedAlgorithm>` at every
   dispatch site — i.e. you can construct any codepoint but you can't
   USE one that doesn't typed-resolve. This is the C11b safety property
   and MUST be enforced end-to-end at every dispatch site (auditable
   workspace-wide).

9. **Hash-codepoint dispatch surface** — BLAKE3-256 default (multihash
   `0x1e`) + SHA-512/256 (`0x1015`) + SHA3-256 (`0x16`) as pre-blessed
   agile fallbacks.

**What "frozen" means here:**
- Codepoint table is BYTEWISE PERMANENT. Old codepoints supported
  FOREVER per item 14 (never-strand-content). New codepoints land
  additively at unused values.
- The framing (multiformats CIDv1 + multihash + multicodec + `did:key` +
  UCAN-Varsig + codepoint-dispatched suite-selector) is the PERMANENT
  commitment — algorithms behind ANY codepoint are swappable as long as
  the framing holds.
- Sizes are NEVER hardcoded. ML-DSA ~1952 B key / ~3309 B sig + ML-KEM-768
  ciphertext dimensions are exercised on the v1-beta DEFAULT path — no
  Ed25519-shaped (32 B-key / 64 B-sig) assumption survives anywhere in
  the workspace.
- Typed-reject discipline frozen.
- The hybrid construction = the byte-faithful IETF LAMPS composite
  `id-MLDSA65-Ed25519-SHA512` (shared `M'`/ctx=Label binding + both halves
  MUST verify; wire `mldsaSig(3309) ‖ tradSig(64)` = 3373 B ML-DSA-first,
  NO commitment trailer — the prior Benten-own NF-4 SHA3-256 commitment is
  dropped). Strip-resistance now rests on the shared-`M'`/ctx=Label binding
  + both-halves-required. Safety invariant: PQC is NEVER the sole trust
  path (the classical half is the audited security floor — exactly what
  makes v1-beta shippable BEFORE the independent audit lands).

**What's NOT frozen:**
- The internal Rust implementation of any algorithm behind a codepoint —
  we may bump `ml-dsa` / `ml-kem` / `x25519-dalek` / `chacha20poly1305`
  crate versions freely. The CODEPOINT is the contract; the CRATE is the
  implementation behind it. C-GM-AUDIT (NF-2) pins the audited versions
  once they land.
- The integration-crate glue logic (concat layout, HKDF info-tag binding,
  envelope serialization) is internal and may refactor as long as the
  wire bytes per codepoint stay byte-identical.
- The internal implementation of the X-Wing-style combiner — the
  ~30-LOC vendored body is replaceable (Benten owns the draft version
  per RATIFIED-PQ NF-5). Only the codepoint `0x647a` + the inputs/outputs
  are frozen.
- The independent audit DELIVERY date — that's a v1-GM gate
  (C-GM-AUDIT), not a v1-beta freeze item.

**Verification mechanism:**
- `cargo-public-api` baselines for `benten-crypto-suite` + `benten-caps`
  + `benten-graph` lock the codepoint-typed constructors.
- A test pin asserting the codepoint table integer values match the
  ratified values exactly (a single golden-file test).
- Existing `tf3a_*` + `tf4_*` tests pin typed-reject behavior.
- G-CORE-3c's conformance test corpus
  (`crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance_additional.rs`
  + `crates/benten-crypto-suite/tests/tf4_gcore3c_full_swap_matrix_strip_resistance_pure_pq_nondefault.rs`
  + `crates/benten-crypto-suite/tests/tf4_pure_pq_gated_audit_landed.rs`)
  exercises all 7 swap-matrix arms — these tests are part of the freeze
  (CI lane).
- The P2P-interop conformance lane (item 14) MUST run on every push and
  pass for v1-beta to ship.
- Item 4 byte-pin tests cover the wire envelope.

**Composing-phase escape valve:**
- Algorithm bump within a codepoint = NOT a freeze break.
- New codepoint addition = ADDITIVE; lands at unused value; old
  codepoints decode forever.
- Removing a codepoint OR repurposing one OR changing the typed-reject-
  on-unknown discipline = HALT-AND-SURFACE.
- **Re-using an existing codepoint for a different algorithm =
  HALT-AND-SURFACE-TO-BEN** (structural — the wire-format byte-pin in
  item 4 will fire if anyone tries; the #1341 0x647b incident is the
  in-tree example of this discipline working).
- `AUDIT_LANDED_PURE_PQ_FLAG` flip from `false` to `true` is a Ben
  decision-point gated by C11c / NF-2 / C-GM-AUDIT (the independent
  audit landing). Orchestrator NEVER flips it autonomously.

---

## 7. §4.33 legacy `module_ecosystem::install_plugin*` path DELETED

**Frozen surfaces:**
- `crates/benten-platform-foundation/src/module_ecosystem.rs` —
  `module_ecosystem::install_plugin*` family **REMOVED** at Core opening
  wave (G-CORE-0 via #1311; verified post-merge per HEAD: only
  `new_version_available_code` helper remains; the test file
  `tf_g_core_0_legacy_install_path_deletion_4_33.rs` carries the absence
  pin).

**What "frozen" means here:**
- DELETION, not deprecation. Per HARD RULE 12 clause-(a) + CLAUDE.md #15
  "two install paths with different security envelopes cannot coexist
  into the freeze."
- A future re-introduction is a Ben re-open (NEW pub item, distinct
  name).
- `cargo-public-api` baseline does NOT contain any
  `module_ecosystem::install_plugin*` symbol. The absence is structural.

**What's NOT frozen:**
- The replacement public install pipeline
  (`benten_platform_foundation::plugin_lifecycle::install_plugin_*`) is
  governed by items 9 + 10 + 12 (cargo-public-api + #1204 + G-CORE-8
  security-shape).

**Verification mechanism:**
- `tf_g_core_0_legacy_install_path_deletion_4_33.rs` absence pin.
- `cargo-public-api` baseline does not contain the deleted symbols.
- Sibling no-regression test pin (recommended at build-backlog) for any
  re-emergence of `install_plugin*` symbols.

**Composing-phase escape valve:**
A genuine need to re-introduce a deleted install path is a HALT-AND-
SURFACE event with strong predisposition AGAINST re-introduction (per
Compromise # documenting why deletion was chosen). A new install path
can be added IF AND ONLY IF the legacy-path-deletion discipline is
preserved (one canonical install path; per CLAUDE.md #15). Adding a
second parallel path = HALT.

---

## 8. `benten-caps` v1-API forks decided + recorded (SEALED `CapabilityPolicy`, HARD-SEAL)

**Orchestrator distinctive-angle decision: Planner-A wins — hard-seal at
G-CORE-9.** Matches spec item 8 ratification "the sealed-extension marker
concrete mechanism = a private `Sealed` supertrait in a non-pub module;
object-safety preserved". Roll the cited G-CORE-8.3 follow-up wave INTO
G-CORE-9. Pay the ~20-test-file migration cost now per
`feedback_agent_economics_prefer_thorough_cleanup`.

**Frozen surfaces (per `RATIFIED-prework-forks-2026-05-18.md` §8-E option
(a) + §1.A.FROZEN item 8):**

| Sub-fork | Decision | Action at G-CORE-9 |
|---|---|---|
| #886 `[features]` | DECIDED (already shipped) | Pin `Cargo.toml` `[features]` block exactly as-is; comment-cite. |
| #993 `CapabilityPolicy` sealed-discipline shape | DECIDED (a) SEALED per RATIFIED-PREWORK §8-E | **HARD-SEAL LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 6 (commit `5ce8bab6`).** `crates/benten-caps/src/policy.rs` `pub(crate) mod sealed { pub trait Sealed {} }` + `pub trait CapabilityPolicy: sealed::Sealed + Send + Sync`. Old `sealed_marker::SealedCapabilityPolicy` soft-seal DELETED (no shim per HARD RULE 12 + CLAUDE.md #5). Workspace-wide migration applied: 4 internal impls (NoAuthBackend, GrantBackedPolicy, LegacyUcanStubBackend, UcanGroundedPolicy<B>) + ~17 workspace test-double impls received sibling `impl Sealed` blocks via the `#[cfg(feature = "testing")] #[doc(hidden)] pub mod __sealed_for_workspace_tests` re-export. Feature pass-through: benten-engine `test-helpers` + benten-eval `testing` features enable `benten-caps/testing`. Object-safety preserved (compile-test pin at `crates/benten-engine/tests/g_core_8_capability_policy_sealed_compile_test.rs` exercises `Arc<dyn CapabilityPolicy>`). |
| 3 new Phase-4-Meta G-CORE-8 hooks (`check_install_consent` / `check_per_delegation` / `check_write_with_audience`) | DECIDED additive (defaulted trait methods + `CapWriteContext`/`ReadContext` audience field) | **Lock the new method signatures + the new field**. Object-safety preserved. **CONSUMPTION WIRED at R6 R1 FP-F4 §S3a/§S3b/§S3c (Row D-3 close, 2026-05-24):** `check_install_consent` is consulted at `plugin_lifecycle::install_plugin` step 3c via `InstallPorts.policy: &dyn InstallConsentPolicy` (the engine-side glue adapter over `CapabilityPolicy::check_install_consent` is a G-COMP-1 addition; the v1-beta install pipeline routes through the foundation-local `InstallConsentPolicy` trait per the dep-cycle constraint). `check_per_delegation` is consulted at `EngineCapsHandle::delegate_capability` between Step 2b and Step 3 with typed `PluginPerDelegationDenied` reject. `check_write_with_audience` is routed at all 4 production `policy.check_write(&ctx)` sites; the audience-aware default delegates to `check_write` so existing policies are unaffected. `audience_did` stays None at sweep sites per Δv3-2 (peer_did at apply_atrium_merge is transport-principal NOT cap-target); populate-side at delegate_capability defers to G-COMP-1 + the existing Layer-3 `check_per_delegation` wiring covers delegate-runtime audience-discrimination needs. **Net: external policy authors overriding any of the three hooks now have runtime effect at v1-beta.** |
| #1005 `actor_hint` shape | DECIDED | Lock as-shipped (the `actor_hint: Option<String>` placeholder per `crates/benten-caps/src/policy.rs::CapWriteContext::actor_hint`). Tightening to a typed principal is a v1-assessment-window v1-Composing item (named in §1.B). |
| #883b prod-dep-edge | DECIDED | Lock as-shipped. |
| #887b `check_read` default-impl | DECIDED (defaulted; pulled WITH/BEFORE G-CORE-8) | Lock at `crates/benten-caps/src/policy.rs::CapabilityPolicy::check_read` (defaulted `fn check_read(...) -> Result<(), CapError>` body; admit-all baseline per Phase-1). |
| §4.69 organizing principle | RESOLVED (a) `EngineCapsHandle`-canonical — see item 1 | Already frozen at item 1; no-regression invariant pin. |

**Additional surface freezes:**
- `crates/benten-caps/src/policy.rs::CapabilityPolicy` `pub trait CapabilityPolicy:
  sealed::Sealed + Send + Sync` (post-hard-seal).
- `CapWriteContext` + `ReadContext` + `PendingOp` (`crates/benten-caps/src/policy.rs::CapWriteContext`,
  `crates/benten-caps/src/policy.rs::ReadContext`,
  `crates/benten-caps/src/policy.rs::PendingOp`) — the cap-policy context types.
- **Apply `#[non_exhaustive]` to `CapWriteContext` + `ReadContext`** —
  **DEFERRED to G-COMP-1 per `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-17**
  (~80+ workspace test-site cascade; the production-side migration to
  `Default::default()` + field-mutation pattern IS already complete at
  v1-beta per Bundle 3 of the R1 fix-pass; the attribute landing is the
  test-cascade half).

**What "frozen" means here:**
- Trait shape (signature, defaulted-vs-required, return types) is locked.
- Sealed discipline is **HARD-ENFORCED via private supertrait** — an
  external impl FAILS to compile post-freeze, which is exactly the v1
  guarantee §8-E (a) DECIDED.
- Object-safety preserved (`Arc<dyn CapabilityPolicy>` boxing compile-
  test pin).
- All three new G-CORE-8 hooks freeze at their CURRENT defaulted
  signature.

**What's NOT frozen:**
- The `NoAuthBackend` / `GrantBackedPolicy` / `UcanGroundedPolicy` impl
  bodies may evolve (they are concrete implementations).
- Internal helper functions under `benten_caps::evaluator_delegation::*`
  are `pub(crate)` and not frozen.

**Verification mechanism:**
- `cargo-public-api` baseline `docs/public-api/benten-caps.txt`
  (regenerated at the G-CORE-9 build-out wave per build-backlog row 1).
- Compile-test pin for `Arc<dyn CapabilityPolicy>` object-safety at
  `crates/benten-engine/tests/g_core_8_capability_policy_sealed_compile_test.rs`.
- **Hard-seal mechanism is structurally enforced by rustc on every workspace
  build** (`pub(crate) mod sealed { pub trait Sealed {} }` private supertrait
  in the `sealed` module at `crates/benten-caps/src/policy.rs::sealed` (a `pub(crate) mod sealed { pub trait Sealed {} }` private supertrait); external `impl CapabilityPolicy`
  cannot reach the private `Sealed` trait and fails to compile). Workspace-test
  opt-in is via the `#[cfg(feature = "testing")] #[doc(hidden)] pub mod
  __sealed_for_workspace_tests` re-export. **Explicit negative-arm trybuild
  regression test fixture DEFERRED to G-COMP-1 per
  `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-20** — the seal MECHANISM
  is real at v1-beta; only the explicit compile-fail test fixture is
  deferred (the mechanism + the cargo-public-api baseline lock are the
  v1-beta structural defenses).
- See `docs/V1-FROZEN-INTERFACE-DEFERRED.md` for the consumption-deferred
  rows (D-3 §8-E hooks; D-17 `#[non_exhaustive]` cascade for CapWriteContext +
  ReadContext; D-20 trybuild regression backstop).

**Composing-phase escape valve:**
- New defaulted trait method = ADDITIVE; fine.
- New required trait method = HALT-AND-SURFACE (breaks every impl).
- Removing or changing a hook signature = HALT-AND-SURFACE.

---

## 9. `cargo-public-api` baselines regenerated + committed as v1 surface

**Frozen surfaces (all 14 baselines regenerated as real cargo-public-api
output at G-CORE-9 FREEZE wave build-out commit `8cc4eddd`; per L12-R3-MIN-1
closure):**
- `docs/public-api/benten-caps.txt`
- `docs/public-api/benten-core.txt`
- `docs/public-api/benten-crypto-suite.txt`
- `docs/public-api/benten-drop.txt` (Phase-4-Meta-Core crate)
- `docs/public-api/benten-dsl-compiler.txt`
- `docs/public-api/benten-engine.txt`
- `docs/public-api/benten-errors.txt`
- `docs/public-api/benten-eval.txt`
- `docs/public-api/benten-graph.txt`
- `docs/public-api/benten-id.txt`
- `docs/public-api/benten-ivm.txt`
- `docs/public-api/benten-platform-foundation.txt`
- `docs/public-api/benten-renderer-tauri.txt`
- `docs/public-api/benten-sync.txt`

**What "frozen" means here:**
- Each baseline is the AUTHORITATIVE list of every `pub` symbol the
  crate exports at `phase-4-meta-core-close`. Any post-freeze delta = CI
  failure on the drift test.
- The baseline format (`.txt` vs `.json`) per crate is locked.
- A NEW pub item post-v1 requires explicit baseline-update + manifest-
  review + Ben sign-off (the cargo-public-api gate is the freeze's
  structural backstop).

**CLOSED post G-CORE-9 build-out (commit `8cc4eddd`):** all 14 baselines
regenerated as real `cargo public-api -p <crate> --simplified --omit
blanket-impls` output and committed as the canonical v1 baseline; the
prior G20-A3 11-LOC placeholder stubs are gone. Per L12-R3-MIN-1 closure
the gate is now structurally REAL (cf. item 1 "drift gate is now REAL,
not a placebo"). Baseline LOC range at HEAD: 110-3509 across the 14
crates. See build-backlog row 1 for the regeneration procedure.

**What's NOT frozen:**
- The `cargo-public-api` tool version (carried in `Cargo.toml` dev-deps);
  tool bumps may produce slight diff in baseline serialization (the
  regeneration ritual handles this).
- The internal symbols (`pub(crate)`, `pub(super)`, private) are not in
  the baseline.

**Verification mechanism:**
- `crates/benten-engine/tests/cargo_public_api_drift.rs` runs the drift
  detection on every CI lane.
- `cargo +stable clippy --workspace --all-targets -- -D warnings`
  orthogonal catch on missing-docs / unused-pub.

**Composing-phase escape valve:**
A new pub item = baseline-update PR; reviewed against the freeze
contract; Composing may add but never remove or rename without HALT-AND-
SURFACE.

---

## 10. TS/JS `@benten/engine` public API frozen (incl. #1204 parity gate)

**Frozen surfaces:**

| Surface | Source | v1-beta lock |
|---|---|---|
| `packages/engine/src/index.ts` exports | All `export` statements at HEAD | LOCKED as-shipped at the freeze wave; commit the post-freeze `index.d.ts` |
| `packages/engine/src/engine.ts` `Engine` + `PolicyKind` | As-shipped | LOCKED |
| `packages/engine/src/errors.generated.ts` `CATALOG_CODES` | The 194-TS-class catalog at HEAD post G-CORE-9 build-out (192 Rust ErrorCode variants + `E_INV_ITERATE_NEST_DEPTH` Phase-2a-retired retained envelope + `E_UNKNOWN` forward-compat sentinel = 194; documented in ERROR-CATALOG.md "Catalog count narrative" table) | LOCKED — mirror item 8's `ErrorCode` mirror discipline; auto-generation contract frozen (regen MUST produce byte-identical file given same input) |
| `packages/engine/src/types.ts` typed-call shapes | `TypedCallInputShapes`, `TypedCallOutputShapes`, `ManifestSignature`, the `ed25519_*` / `keypair_*` / `did_resolve` arms | LOCKED — **PQ-hybrid-capable** sizes (NO hardcoded Ed25519 32B-key / 64B-sig assumption; per item 10 PQ-hybrid JS-shape widening + napi-r1-1 atomic mirror) |
| `packages/engine/src/types.ts` other interface exports | `Subgraph`, `RegisteredHandler`, `AttributionFrame`, `Trace*`, `CapabilityClaim`, `DeviceAttestation`, `CapabilityGrant`, `Edge`, `TypedCallOp`, etc. | LOCKED as-shipped |
| `packages/engine/src/index.d.ts` | The TS module declaration file; generated from napi-rs via the build pipeline | LOCKED post-regen at the freeze wave |
| `packages/engine/src/stream.ts` `StreamHandle.next()` post-PR-B | The G-CORE-10 PR-B AsyncTask migration (`#1340 05707357`) post-merge shape | LOCKED — sync→async break is ratified per CLAUDE.md #5 no-shims + the #1331 "no users yet" override |
| `packages/engine/src/atrium.ts` `Atrium` + `atrium_*` typed-call surfaces | As-shipped | LOCKED |
| `packages/engine/src/subscribe.ts` `SubscribeHandle` | As-shipped | LOCKED |
| `packages/engine/src/identity.ts` `Keypair` / `VerifiableCredential` / `DeviceAttestation` JS-side wrappers | As-shipped (PQ-hybrid sized) | LOCKED |
| `packages/engine/src/manifest.ts` `ManifestSignature` + plugin-manifest JS shapes | As-shipped (PQ-hybrid sized) | LOCKED |
| `packages/engine/src/sandbox.ts` SANDBOX JS API | As-shipped | LOCKED |
| `packages/engine/src/wait.ts` WAIT JS API | As-shipped | LOCKED |

**#1204 JS-side public-API parity gate LANDED at G-CORE-9
V1-FROZEN-INTERFACE row 2 (commit `13322df4`).** Workflow at
`.github/workflows/ts-public-api.yml`; baseline at
`packages/engine/etc/public-api.txt` (403 LOC; extract-from-.d.ts
structural diff covering all 14 `dist/*.d.ts` files). Migration to
`@microsoft/api-extractor` is named for v1-Composing (the workflow
+ baseline-file location ARE the migration seam; swap-in is
contained).

**errors.generated.ts ↔ catalog ↔ Rust `ErrorCode` parity audit
RESOLVED at G-CORE-9 V1-FROZEN-INTERFACE row 8a (investigation outcome
in commit `75a1d33a` body).** Post-build-out counts: 192 Rust ErrorCode
variants + 1 `E_INV_ITERATE_NEST_DEPTH` Phase-2a-retired retained
envelope (catalog ID retained for backward-compat string round-trip;
Rust enum has no variant) + 1 `E_UNKNOWN` forward-compat sentinel
(mirrors Rust `Unknown(String)` fallback) = 194 catalog/TS entries.
**Delta is the legitimate retained-envelope set, NOT drift**; the
drift-detect script (`npm run drift:errors`) validates this exact
pattern. Documented in ERROR-CATALOG.md's "Catalog count narrative"
table; the script reports "OK — catalog, Rust enum, and TS classes
agree" at every CI run.

**What "frozen" means here:**
- The exported TS class/type/function names are locked.
- The PQ-hybrid sizing (no Ed25519-shaped assumption) is locked at the
  type level — e.g. `keypair_publicKey: Uint8Array` with no length pin
  in the type, and runtime length-check tests at
  `packages/engine/src/manifest.test.ts` exercise the hybrid-sized
  inputs.
- The `errors.generated.ts` regen-determinism is part of the contract (a
  re-codegen produces zero diff).

**What's NOT frozen:**
- The internal Rust→napi bridging logic.
- The `bindings/napi/src/*.rs` rust source (governed by item 9
  cargo-public-api baselines for `bindings-napi` if/when that crate gets
  one; today napi is workspace-only with no cargo-public-api baseline).
- JS internal helpers (`packages/engine/src/internal/*` or similar) —
  out of scope; not exported.
- JSDoc-only changes to existing exports.
- Subpath exports other than `errors` — out of scope; current set is the
  lock.

**Verification mechanism:**
- #1204 JS-side parity gate (BUILD-AT-FREEZE-WAVE row 2).
- `scripts/drift-detect-error-variant-mirror.ts` enforces Rust↔TS error
  parity.
- `packages/engine/src/*.test.ts` carries behavior pins.
- A new test that scans `types.ts` for hardcoded `[ 0-9]+ B` size
  literals on crypto-touching types (PQ-hybrid JS-shape widening
  invariant).

**Composing-phase escape valve:**
- New TS export = ADDITIVE; baseline-update PR; reviewed against the
  freeze.
- Removing/renaming = HALT-AND-SURFACE.
- Backward-compatible additions (new optional fields on existing
  interfaces with `#[non_exhaustive]`-equivalent TS shape) can land with
  explicit regenerate + R6 review.

---

## 11. META #907 `#[non_exhaustive]` sweep — MAXIMALIST application

**Orchestrator distinctive-angle decision: Planner-A wins — maximalist
over the FULL enumerated workspace surface.** Matches spec item 11 "ONE
coherent freeze-wave over the FULL enumerated workspace surface."

**Frozen scope:**

The G-CORE-9 wave enumerates EVERY public enum + struct workspace-wide
and makes a per-item apply-or-D8-carve-out decision. Verified at HEAD:
**158 total `pub enum` across `crates/`; 148 of those carry
`#[non_exhaustive]` already (counting all types, not just enums)**. The
freeze MUST close the remaining gap.

**Architectural position: APPLY `#[non_exhaustive]` UNIVERSALLY** unless
a D8-carve-out has a documented structural reason. The cost of NOT
applying it to a type that will need to grow post-v1 is a SemVer break —
*non-recoverable post-`v1-GM`*. The cost of applying it where it's not
strictly needed is mostly a documentation/match-arm-awkwardness tax.
Lean DEFENSIVE here.

**The CARVE-OUT set** (cases where `#[non_exhaustive]` is structurally
WRONG):

- **`benten-caps::Scope`** (`crates/benten-caps/src/scope.rs:46`) —
  EXACTLY two arms per §1.A.FROZEN item 15(c); `#[non_exhaustive]` would
  defeat the exhaustive-match structural pin. **CARVE-OUT (documented;
  per the doc-block at scope.rs:40-46).**
- **`benten-core::Subgraph::PrimitiveKind`** — KEEP existing
  `#[non_exhaustive]` per Planner-B's reading: it serves as the
  DEFENSIVE guard against future-13th-primitive proposals that
  CLAUDE.md #1 rejects (the carve-out IS the application here — the
  attribute presence is the freeze defense).
- Per `benten_graph::GraphError::TxAborted` (in `crates/benten-graph/src/lib.rs`) Fwd-2 #997 / #1207 explicit
  decision NOT to apply — preserve the explicit reason at the cite.

**The enumerated must-apply set** (from §1.A.FROZEN item 11 + workspace
verification at HEAD):

| Crate | Type | Currently `#[non_exhaustive]`? | v1-beta action |
|---|---|---|---|
| `benten-engine` | `EngineError`, `engine_config::*`, `engine_sync::*` | YES | KEEP |
| `benten-engine` | `UserViewInputPattern` / `TraceStep` / `Transport` (thin_client) / `AtriumMode` / `SuspensionOutcome` / `DelegationResolution` / `NextChunkPoll` / `StreamCursor` / `SubscribeCursor` / `WriteBoundaryChainOutcome` / `ManifestEnvelopeRecheckOutcome` / `ManifestVerifyMode` | **12+ verified MISSING at HEAD** | APPLY |
| `benten-core` | `WriteAuthority`, `ChangeEvent`, `ChangeKind`, `subgraph_spec::Spec`+`SpecError`, `version_dag::*`, `Subgraph::PrimitiveKind` | YES (except `Spec` which uses private-fields-plus-builder pattern for equivalent SemVer-safety per L17-r2-1) | KEEP |
| `benten-core` | new `RestrictedSpec` enum variants (`subgraph_spec/spec.rs:126`) | TBD | APPLY |
| `benten-ivm` | `AlgorithmError` | per spec item 11 | AUDIT + APPLY |
| `benten-sync` | §4.71 5-enum cluster | per spec item 11 | AUDIT + APPLY |
| `benten-caps` | `CapError`, `RestrictedSpec`, `PendingOp` | YES | KEEP |
| `benten-caps` | `TypedCapGroup` | per spec item 11 | AUDIT + APPLY |
| `benten-caps` | `CapWriteContext` + `ReadContext` (structs) | TBD | **APPLY** (item 8 coupling) |
| `benten-caps` | **`Scope`** | NO (deliberate) | **DO NOT APPLY** — explicit carve-out per item 15(c); the EXACTLY-two-arms-by-the-type-system property IS the structural pin |
| `benten-ivm` | **`Strategy`** | NO (deliberate) | **DO NOT APPLY** — explicit carve-out per G-CORE-9 R1 L8-MAJOR-3 ratification; the 3-arm `{A, B, Reserved}` set IS load-bearing per the spec's audit-pin (item 11 documented carve-out); adding a 4th strategy is a Composing-time architectural decision, NOT a SemVer non-breaking field addition |
| `benten-graph` | `WriteContext` (struct) | NO at HEAD | **APPLY** (item 5 coupling) |
| `benten-graph` | `ChangeEvent` (re-export) | YES | KEEP |
| `benten-graph` | `GraphError` | YES | KEEP |
| `benten-graph` | `GraphError::TxAborted` (per-variant) | Unclear at HEAD | **APPLY** defensively |
| `benten-graph` | `WriteAuthority` (re-export from core) | YES | KEEP |
| `benten-graph` | per `benten_graph::GraphError::TxAborted` Fwd-2 #997/#1207 explicit no-apply | NO (explicit reason) | DO NOT APPLY |
| `benten-crypto-suite` | `UnsupportedAlgorithm` | TBD | APPLY |
| `benten-crypto-suite` | `SwapMatrixError` | TBD | APPLY |
| `benten-drop` | `DropBundleVersion`, `DropContentMode`, `DropBundleError`, `EnvelopeSigError` | TBD | APPLY each |
| `benten-renderer-tauri` | `IpcMethod` (per-method allowlist) | TBD | APPLY |
| `benten-dsl-compiler` | `CompileError`, `CompiledSubgraph`, `CompiledPrimitive`, `Diagnostic`, `Span` | YES (5/5 applied per L9-DSL-MAJOR-2 closure at G-CORE-9 R2; audit test at `crates/benten-dsl-compiler/tests/g_core_9_non_exhaustive_audit_dsl.rs` 2/2 PASS) | KEEP |
| `benten-errors` | `ErrorCode` | YES (per Phase-4-Foundation freeze) | KEEP |

**What "frozen" means here:**
- `#[non_exhaustive]` per-item decision is BAKED into the type — a
  future variant addition is a minor version bump, NOT a SemVer break.
- The carve-out list is part of the freeze (a carve-out can't be
  silently added later; an enum that ships exhaustive-by-design is
  exhaustive forever unless re-opened).
- `cargo +stable clippy --workspace --all-targets -- -D warnings`
  catches the `non_exhaustive_omitted_patterns` lint on consumers.

**What's NOT frozen:**
- The variant SET inside the enum (the whole point of `#[non_exhaustive]`
  is permitting additive future variants).
- The struct FIELD SET (same — additive future fields).

**Verification mechanism:**
- An enumerated-per-type audit test pin
  `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs` exercises
  arm-coverage matches + struct construction patterns for a curated set
  of v1-beta-load-bearing types (~10 named types covering the highest-leverage
  surfaces: WriteBoundaryChainOutcome / AtriumMode / DelegationResolution /
  ManifestVerifyMode / NextChunkPoll / Strategy carve-out + AuthorizationGrant
  constructor pattern + 2 CapWriteContext/ReadContext Default-construction
  + SuspensionOutcome D-17 deferred-arm coverage). **The workspace-wide
  walker** (consuming cargo-public-api JSON output OR a syn-based AST walker
  OR rustdoc-json walk to assert every pub enum/struct carries
  `#[non_exhaustive]` OR is in the carve-out registry) is **DEFERRED to
  G-COMP-1 per `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-17** (extended
  at G-CORE-9 R2 L8-R2-MINOR-CARRY-1 closure). At v1-beta the
  enumerated-per-type audit catches drift against the named-set; the
  workspace-walker is the regression-defense enhancement.
- `cargo-public-api` baseline catches the attribute (it's part of the
  declaration shape).

**Composing-phase escape valve:**
- New `pub enum` / `pub struct` in Composing MUST default to
  `#[non_exhaustive]` per the freeze policy; carve-out requires explicit
  registry entry + Ben sign-off.
- ADDING `#[non_exhaustive]` to a type that doesn't have it = additive +
  permitted in Composing (caveat: technically SemVer-breaking for
  external direct-struct-literal construction, so the migration path
  must be tested).
- REMOVING `#[non_exhaustive]` from a frozen item = HALT-AND-SURFACE.

---

## 12. G-CORE-8 security-surface public-shape lock

**Frozen surfaces (per §1.A.FROZEN item 12 + security-r1-2; all DECIDED +
SHIPPED per #1338 G-CORE-8 fix-pass + the wave-2 batch #1340):**

- `crates/benten-engine/src/manifest_envelope_recheck.rs::ManifestEnvelopeRecheckOutcome` `pub enum
  ManifestEnvelopeRecheckOutcome { NotApplicable, UnresolvedDeny, Admitted,
  OutsideEnvelope { offending_plugin_did, cap_pattern } }` — **all four
  variants frozen** including the post-rename `UnresolvedDeny` semantic.
  **`#[non_exhaustive]` LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 8b
  (commit `75a1d33a`)** at `crates/benten-engine/src/manifest_envelope_recheck.rs::ManifestEnvelopeRecheckOutcome` (attribute applied directly above the enum definition).
  Adding a fifth variant post-v1 is breaking; the attribute makes the
  variant-set additively extensible.
- The `Admitted` arm's structural invariant (security-r1-2 frozen):
  returned ONLY on a positively-verified envelope/chain match.
  **`outcome_to_row_reject` at `crates/benten-engine/src/manifest_envelope_recheck.rs::outcome_to_row_reject`
  MUST NOT collapse a non-positive outcome to `Ok(())`** — the
  structural property is part of the freeze.
- `pub trait ManifestEnvelopeRechecker` (`crates/benten-engine/src/manifest_envelope_recheck.rs::ManifestEnvelopeRechecker`) — the port interface; method
  signatures frozen.
- `NoopManifestEnvelopeRechecker` is the v1-beta **shipped default**
  (`crates/benten-engine/src/engine.rs::Engine::new_with_engine_caps_handle` always installs
  `Some(Arc::new(NoopManifestEnvelopeRechecker))` per the
  `manifest_envelope_rechecker: Some(Arc::new(...NoopManifestEnvelopeRechecker))` initializer). At HEAD its
  `recheck_row` impl returns `NotApplicable` for every input
  (`crates/benten-engine/src/manifest_envelope_recheck.rs::NoopManifestEnvelopeRechecker::recheck_row`); the substantive Layer-3
  defense (per-DID `PluginLibrary` + `UserDidRegistry` consult) is
  **consumption-deferred** — destination: `docs/V1-FROZEN-INTERFACE-DEFERRED.md`
  G-COMP-1 row "ProductionManifestEnvelopeRechecker production impl
  + default-builder wiring". The structural fail-CLOSED on empty/sentinel
  peer-DID at `apply_atrium_merge` IS live at v1-beta (see below);
  the per-DID substantive-recheck is the G-COMP-1 deliverable.
  Compromise #26 in SECURITY-POSTURE.md documents this v1-beta posture
  end-to-end.
- Empty/sentinel `<unresolved-peer>` peer-DID MUST deny (never admit) at
  recheck — **structurally enforced at v1-beta** in the
  `apply_atrium_merge` body at `crates/benten-engine/src/engine.rs`
  (the `resolve_peer_dids` empty-arm short-circuit returns typed
  `ManifestEnvelopeRecheckUnresolvedDeny` BEFORE rechecker dispatch). The
  §4.25 sync-hydrate path shares the SAME primitive (`UnresolvedDeny` +
  `outcome_to_row_reject`); the handshake.rs wire-up is named to
  G-COMP-1 per
  `crates/benten-engine/tests/g_core_8_manifest_envelope_recheck_fail_closed_flip_4_36.rs`.
- `accept_atrium_share` — **deferred to G-COMP-1 (G24-D-FP-1 follow-up
  wave)**; destination: `docs/V1-FROZEN-INTERFACE-DEFERRED.md` row
  "accept_atrium_share cross-peer install seam". At v1-beta the
  cross-peer plugin-install verification is NOT live; Compromise #26
  documents this. The platform-foundation install pipeline at v1-beta
  consumes plugins through user-DID-signed install records ONLY (no
  cross-peer ingest).
- Any §4.40 key-at-rest public type AUDIT + freeze whatever shipped at
  G-CORE-7 install-hardening; if absent, defer to the C1+C2 substrate
  completion (§989→§1301).

**What "frozen" means here:**
- The four `ManifestEnvelopeRecheckOutcome` variants + their semantics
  are bytewise + behaviorally locked.
- The `pub trait ManifestEnvelopeRechecker` port-interface signatures
  are locked (G-COMP-1's ProductionManifestEnvelopeRechecker will be
  an additional `impl ManifestEnvelopeRechecker` honoring this trait).
- The structural empty-peer-DID fail-CLOSED at the §4.36 merge boundary
  IS the load-bearing v1-beta security property — re-introducing
  admit-on-unresolved is a HALT-AND-SURFACE event.

**What's NOT frozen (consumption deferred to G-COMP-1):**
- The substantive `ProductionManifestEnvelopeRechecker` implementation
  (per-DID `PluginLibrary` + `UserDidRegistry` consult) — deferred per
  Compromise #26 v1-beta posture.
- The DEFAULT-builder wiring of `ProductionManifestEnvelopeRechecker`
  in place of `NoopManifestEnvelopeRechecker` — deferred to G-COMP-1.
- The `accept_atrium_share` cross-peer install seam — deferred to
  G-COMP-1 (G24-D-FP-1 wave).
- The §4.25 sync-hydrate consumption of `UnresolvedDeny` at
  `crates/benten-sync/src/handshake.rs` — deferred to G-COMP-1.

See `docs/V1-FROZEN-INTERFACE-DEFERRED.md` for the explicit G-COMP-1
destination rows + per-surface deferral rationale.

**Verification mechanism:**
- `crates/benten-engine/tests/g_core_8_manifest_envelope_recheck_*.rs`
  family carries the would-FAIL pins (incl. the
  `noop_rechecker_admit_everything_is_security_r1_1_would_fail_baseline`
  honest-name pin per mr-2 fix-up).
- `crates/benten-engine/tests/g_core_8_manifest_envelope_recheck_fail_closed_flip_4_36.rs`
  pins the `UnresolvedDeny` → `Err(...)` non-collapse.
- `cargo-public-api` baseline locks the enum + trait shape.

**Composing-phase escape valve:**
- New variant = gated by `#[non_exhaustive]` (per item 11 apply); the
  variant MUST default to denying behavior (structural fail-closed
  invariant). Adding a new variant that admits = HALT.
- Changing fail-CLOSED semantics = HALT-AND-SURFACE-WITH-STRONG-
  DEFAULT-NO.

---

## 13. Engine↔host runtime-ownership boundary frozen = bridged-dual-runtime

**Orchestrator distinctive-angle decision: KEEP const-allowlist; both
planners agreed; locked as-shipped.**

**Frozen surfaces (per `RATIFIED-prework-forks-2026-05-18.md` §8-C option
(2) + §1.A.FROZEN item 13):**

- The engine OWNS its own tokio runtime — no
  `tauri::Builder::with_runtime`, no shell-runtime-handle threading
  through the engine API.
- `crates/benten-renderer-tauri/src/lib.rs` — the renderer crate has
  ZERO `tauri`/`tokio` deps (per the swappability thesis; verified by
  compile-test pin).
- `benten_renderer_tauri::IPC_METHODS` (in `crates/benten-renderer-tauri/src/lib.rs`) `pub const IPC_METHODS:
  &[IpcMethod]` — the **explicit method-name const-allowlist.
  IPC_METHODS IS THE IPC SURFACE** — webview cannot invoke a method not
  in the const-allowlist. **Registration affordance REJECTED** per
  CLAUDE.md #19 engine-extensions-are-compile-time-linked discipline; a
  runtime-registerable IPC method bypasses the compile-time review gate.
- `crates/benten-platform-foundation/src/materializer.rs:552` `pub trait
  Renderer: Send + Sync` with `render(&MaterializerOutput) -> Result<(),
  RenderError>` + `backend_name() -> &'static str`. **Trait surface
  carries NO transport-specific methods** (compile-test pin asserts a
  Tauri runtime type can't leak through the seam).
- `crates/benten-engine/src/thin_client_bridge.rs:86` `pub struct
  ThinClientBridge` — the §4.22 thin-client bridge; principal-resolution
  semantics frozen per G-CORE-8.

**The three IPC contracts that MUST lock CONSISTENT (per deployment-r1-6):**
- (a) The #838 IPC-surface shape (the const-allowlist + explicit baseline-
  update + manifest-review gate — a deliberate T3-defense; the freeze
  preserves the const-allowlist property).
- (b) The §4.22 thin-client bridge principal-resolution (G-CORE-8).
- (c) The §8-C bridged-dual-runtime channel/IPC boundary (engine ↔ host
  shell).

**What "frozen" means here:**
- The Renderer trait method set is locked (`render` + `backend_name`
  only).
- IPC_METHODS is the AUTHORITATIVE method-name allowlist; additions
  require explicit const update + manifest-review.
- The engine's runtime ownership is structural — no API change can
  thread a host runtime through.
- The ThinClientBridge principal-resolution semantics (G-CORE-8) are
  frozen as shipped.

**What's NOT frozen:**
- Concrete Renderer impls (`BrowserRender`, `TauriRenderer`, future
  `TauriVersoRender`, `SlintRender`, etc.) are NOT part of the freeze;
  new renderer backends can land in Composing per CLAUDE.md #19
  compile-time-linked.
- The internal channel/IPC mechanism between engine + host shell may
  change (e.g. swap from native channel to lock-free queue) as long as
  the observable contract holds.
- Adding new IPC methods to `IPC_METHODS` — additive; pre-flight gate
  per the rustdoc narrative; permitted in Composing.

**Verification mechanism:**
- `crates/benten-renderer-tauri/tests/arch_n_benten_renderer_tauri_dep_direction.rs`
  no-tauri / no-tokio dep posture pin (sweeps Cargo.toml + use-statements
  across src/).
- `crates/benten-renderer-tauri/tests/ipc_allowlist_rejects_unknown_method.rs`
  + `crates/benten-renderer-tauri/tests/ipc_method_invocation_requires_manifest_cap.rs`
  + `crates/benten-renderer-tauri/tests/ipc_method_name_stability_drift_detector.rs`
  IPC allowlist pins (asserts `IPC_METHODS` is `const`, not `static mut`,
  not a dynamic registry; methods bind manifest caps; name-stability
  drift-detect runs CI-wired).
- **Runtime-handle-leak structural pin via cargo-public-api baseline** —
  the `EngineBuilder` block at `docs/public-api/benten-engine.txt:189-195`
  (`backend` / `build` / `capability_policy` / etc.) is grep-clean of
  `tauri::Runtime`, `tokio::runtime::Handle`, and `with_runtime`; any
  future PR threading one would surface as drift and fail the now-required
  cargo-public-api workflow (Bundle 10 + Fork 3 flip + Bundle R2.3
  branch-protection inclusion at `.github/workflows/cargo-public-api.yml:91-93`
  + GitHub branch-protection required_status_checks). This consolidates with
  the cargo-public-api baseline defense below; per G-CORE-9 R2 L10-r2-1
  disposition (path a).
- `cargo-public-api` baselines for `benten-renderer-tauri` (JSON format)
  and `benten-platform-foundation` (baseline regenerated at the G-CORE-9
  build-out wave per build-backlog row 1; 2556 LOC at HEAD).

**Composing-phase escape valve:**
- New IPC method = baseline-update PR; manifest-review; passes if in the
  T3-defense spirit (no privileged operations, no auth-bypass).
- Threading a shell runtime through the engine API = HARD HALT-AND-
  SURFACE (this is the §8-C explicit rejection — "the engine NEVER
  borrows the host shell's runtime").
- New Renderer trait method = HALT-AND-SURFACE (breaks every backend
  implementation).

---

## 14. P2P-interop conformance invariant frozen

**Frozen surfaces (per `RATIFIED-pq-default-reframe-2026-05-19.md` §4 +
§1.A.FROZEN item 14):**

A mandatory baseline conformance suite that EVERY peer MUST satisfy,
with four structural properties:

**(a) Mandatory baseline conformance suite.** Every claimed-Benten peer
MUST pass the suite. The suite lives at
`crates/benten-crypto-suite/tests/tf4_gcore3c_full_swap_matrix_strip_resistance_pure_pq_nondefault.rs`
+ `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance_additional.rs`
+ `crates/benten-crypto-suite/tests/tf4_codepoint_0x647c_pure_pq_mlkem_only.rs`
+ `crates/benten-crypto-suite/tests/tf4_pure_pq_gated_audit_landed.rs`. These
files cover all 7 swap-matrix arms × both wire directions at v1-beta.
(The earlier-cited `p2p_interop_conformance_*.rs` family + the standalone
`conformance_baseline.rs` did not land under those names; the `tf4_gcore3c_*`
named files ARE the substantive conformance suite per L14-MIN-2 close at
R6-FP-D 2026-05-24. Naming-drift fixed; substance intact.)

**(b) Typed-unsupported-error, NEVER silent fallback.** Unknown crypto
codepoint surfaces typed `UnsupportedAlgorithm::{Signature, CipherSuite,
Hash}` — never silent fallback. Veilid `common_crypto_kinds`-intersection
/ MLS `RequiredCapabilities`-floor / Nostr **NIP-44** explicit-MUST-
indicate-unsupported normative precedent. **Age's silent-ignore is the
explicitly-REJECTED outlier.**

**(c) No wire-break when a codepoint is added.** Additive-codepoint
discipline: a new codepoint at an unused value extends the table; old
deserializers see `UnsupportedAlgorithm` and route to the typed-reject
arm; old encoders never produce the new codepoint. Wire format is
permanent. Pinned by a test that walks `CipherSuiteCodepoint` +
`SigCodepoint` + `HashCodepoint` constant lists + asserts every value
maps to EITHER a LIVE implementation OR a typed-reject arm (never a
panic / silent / wildcard match).

**(d) Old codepoints supported FOREVER — algorithm-add never strands
previously-written content.** Stated STRONGER than MLS: Benten content
is immutable content-addressed objects, so old objects NEVER need an
MLS-style `ReInit` ceremony. Only NEW objects use the new codepoint;
structurally avoids MLS's hard migration case. Pinned narratively in
`docs/SECURITY-POSTURE.md` + by a test that decodes a historical fixture
(BLAKE3-encoded; classical Ed25519 sig from a pre-#1300 baseline) under
the post-freeze code + asserts it still verifies.

**(e) Component-ID reuse policy:** Benten reuses IANA HPKE/COSE
component IDs (KEM / AEAD / KDF) — **NEVER mints Benten component
algorithm numbers**. Benten owns ONLY the thin one-codepoint-per-suite
SELECTOR table (MLS RFC 9420 one-codepoint-per-suite model). The
selector table is Benten-owned; the algorithm IDs reference upstream
registries. Pinned by a docstring on `CipherSuiteCodepoint` citing IANA
HPKE/COSE registries.

**Rejected alternative (record + reject any future re-proposal):**
"PQ-TLS-as-envelope buys time" (Matrix's transport-relayed position).
REJECTED for Benten — Benten ciphertext rests at-rest on peer disks
(CLAUDE.md #18 / item 6 substrate); the transport-envelope argument
doesn't apply. iroh transport is classical-only/no-PQ-roadmap so HNDL
protection MUST be Benten-owned application-layer object encryption.

**What "frozen" means here:**
- The four structural properties (a)-(d) are PERMANENT invariants —
  they outlast any single algorithm choice.
- The conformance suite shape is locked.
- The IANA component-ID reuse policy is a freeze item (a future agent
  proposing to mint Benten algorithm numbers MUST be rejected with
  reference to this item).
- The typed-unsupported-never-silent-fallback discipline is enforced at
  every dispatch site workspace-wide.

**What's NOT frozen:**
- Which algorithms occupy reserved codepoints (e.g. `0x647b` HQC build-
  go triggered by FIPS 207 final).
- The internal mechanism by which the conformance suite runs (CI lane
  vs GitHub Action vs scheduled cron).
- The NF-1 PQ⊕PQ end-state arms — reserved-typed-reject codepoints;
  build-trigger is FIPS-207-final (RATIFIED-PQ NF-1); v1-beta locks the
  reservation, not the build.

**Verification mechanism:**
- Conformance baseline test (BUILD-AT-FREEZE-WAVE: validate).
- Additive-codepoint enumeration test (new; scans codepoint constant
  blocks).
- Historical-fixture decode test (new; pins old-codepoint-supported-
  forever).
- The cite-drift CI lane (item 6 mechanism).
- A standing CI lane (per §4 plan) that runs the conformance suite on
  every push.

**Composing-phase escape valve:**
- New algorithm on reserved codepoint = ADDITIVE per (c)+(d); fine.
- Re-using a codepoint for a different algorithm = HALT (the #1341
  incident already established this discipline).
- Silent fallback on unknown codepoint = HALT (typed-reject is the law).
- Wire-break = HALT (item 4 byte-pin tests fire).
- Minting Benten component algorithm number = HALT-AND-SURFACE (rejected
  outright with reference to this item).

---

## 15. Sharing & Confidentiality (S&C) public surface frozen

Per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` (the authoritative
post-spike-sequence record) + §1.A.FROZEN item 15 sub-clauses (a)-(j) +
CLAUDE.md baked-in #18 (Principal primitive + plugin trust model).

### 15.a — SubgraphSpec primitive

**Frozen surfaces:**
- `crates/benten-core/src/subgraph_spec/spec.rs:190` `pub struct Spec` —
  the 4-thing thin core (Roots / Expansion / Inclusion / Termination).
  **Equivalent SemVer-safety via private fields + builder pattern**
  (`pub fn builder() -> SpecBuilder` at `spec.rs:216`; all four fields are
  private). External direct-struct-literal construction is already blocked
  by field visibility — `#[non_exhaustive]` is NOT required for the
  SemVer-additive-field-extension property `Spec` needs. Per G-CORE-9 R2
  L17-r2-1 disposition path (b).
- `crates/benten-core/src/subgraph_spec/walker.rs:78` `pub fn walk(spec:
  &Spec) -> Result<WalkResult, SubgraphSpecError>` — the canonical
  walker.
- `crates/benten-core/src/subgraph_spec/walker.rs:183` `pub fn
  walker_as_subgraph() -> Subgraph` — the fractal-property pin (the
  walker IS a Subgraph composed of the existing 12 operation primitives;
  CLAUDE.md baked-in #1 12-primitive irreducibility PRESERVED — no new
  `PrimitiveKind` variant minted).
- `pub struct WalkResult` (`walker.rs:50`) — `enumerated: Vec<(Cid,
  StructuralPath)>` BFS-order shape per RATIFIED-S&C §R4 explicit.
- `pub struct StructuralPath` (`spec.rs:38`).
- `pub enum SubgraphSpecError` (`errors.rs:20`).
- `pub fn intersect(a: &Spec, b: &Spec)` / `pub fn union(...)` / `pub fn
  filter(...)` combinators (`combinators.rs:37, 99, 140`).

**ARCHITECTURAL CONCERN — type-name collision (orchestrator-decided per
distinctive-angle).** Two `RestrictedSpec` types existed at HEAD: (i)
`crates/benten-caps/src/restricted_spec.rs:103` (the 6-dimension product
per (b)); (ii) `crates/benten-core/src/subgraph_spec/spec.rs:126` (a
different enum). **Renames LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 7
(commit `dd12f394`):** `subgraph_spec::RestrictedSpec` →
`SubgraphSpecRestriction`; `caps::RestrictedSpec` → `RestrictedScope`.
Compatible-interpretation trap (someone imports the wrong one and
trait-bounds line up enough that it compiles but runtime is wrong) is
structurally prevented by type-level distinction. **Tentatively-decided
per night-shift stance; rebuttable at morning Ben review** (if rebutted,
revert the rename commit + take the import-confusion liability into
v1-Composing instead).

**What "frozen" means here:**
- The 4-thing structural decomposition (Roots / Expansion / Inclusion /
  Termination) is PERMANENT.
- The fractal-property invariant: `walker_as_subgraph()` is a Subgraph
  composed of the existing 12 primitives; ANY proposal to mint a new
  `PrimitiveKind::SubgraphSpec` variant is a HARD-HALT (re-opens
  CLAUDE.md #1).
- Walker enumeration order = BFS (per (h) below).

**Verification mechanism:**
- A no-13th-primitive test pin
  (`crates/benten-core/tests/tf3w_walker_is_a_subgraph_no_new_primitive_kind.rs`)
  asserts `PrimitiveKind::*` discriminant count remains 12.
- `cargo-public-api` (item 9).

**Escape valve:** a proposed 13th primitive = HALT-AND-SURFACE-TO-BEN
(CLAUDE.md #1 explicit).

### 15.b — `RestrictedScope` shape (post-rename; 6-dimension product)

**Frozen surfaces:**
- `crates/benten-caps/src/restricted_spec.rs:103` `pub struct
  RestrictedScope` (post-rename per 15.a name-collision resolution) with
  6 dimensions: roots + edge-allowlist + max_depth + label-allowlist +
  label-denylist + property-equalities.
- `#[non_exhaustive]` already APPLIED — KEEP.
- Structural `contains(&self, other: &RestrictedScope) -> bool` method —
  decidable per-dimension + composed with `&&`.
- `no-opaque-arm` decision documented in scope's rustdoc + tested.

**Extension slots** for additive predicates (Boolean OR, numeric
comparisons, has_edge, anchor-chain LIMIT, etc.) are NAMED-not-opaque —
each future extension lands as a typed slot with its own decidable
containment rule (~50-100 LOC).

**Rejected and frozen-as-rejected:** the witness-bearing OpaqueSelector
mechanism (Path b in R1). Structurally unsound per Spike H+1.1's b.SEC
#4. Future agents proposing the witness pattern as a feature MUST be
rejected with reference to this freeze item.

**What "frozen" means here:**
- The 6 dimensions + their structural-containment semantics are locked.
- The NAMED-extension-slot pattern is the canonical add-mechanism (no
  opaque arm).
- `contains()` is decidable + total (no panics, no `unimplemented!()`).

**What's NOT frozen:**
- The internal representation of each dimension (e.g. `BTreeSet` vs
  `Vec` vs `Cow`) may evolve.
- The set of named extension slots is OPEN per the additive-predicate
  pattern.

**Verification:** `cargo-public-api`; the `restricted_spec.rs`
narrative-as-cite; existing tests in `crates/benten-caps/tests/tf3b_*.rs`.

**Escape valve:** adding an OPAQUE dimension (vs a NAMED slot) = HALT
(RATIFIED-S&C R1 explicit).

### 15.c — Structured UCAN `Scope` enum — EXACTLY two arms

**Frozen surfaces:**
- `crates/benten-caps/src/scope.rs:46` `pub enum Scope` — EXACTLY TWO
  arms: `Hashes(Vec<Cid>)` + `RestrictedSelector(RestrictedScope)`
  (post-rename per 15.a).
- **NOT `#[non_exhaustive]`** (deliberate carve-out per the doc-block at
  `scope.rs:32-46`: the freeze MUST be enforced by the type system so a
  future agent proposing a third arm gets exhaustive-match compile
  failures + the cite-drift CI lane fires on the `no-opaque-arm`
  sentinel).
- A third arm CANNOT be added post-freeze without explicit re-open.
- Wire envelope typed for additive future arms via **serde-tag dispatch**
  (the two arms `Hashes(Vec<Cid>)` + `RestrictedSelector(RestrictedScope)`
  carry distinct serde tags; the dispatch is at the serde-tag layer, NOT
  via a numeric codepoint table parallel to the crypto-agility framing)
  — a future arm lands as a new serde-tag arm on this carve-out enum,
  never repurposing the existing two-arm enum. The "codepoint-dispatch"
  framing in earlier drafts overstated the machinery; the serde-tagged
  dispatch IS additive-friendly so the freeze contract is not weakened.
  Per G-CORE-9 R2 L17-r2-2 disposition path (a).

**What "frozen" means here:**
- EXACTLY two arms — structural pin via exhaustive `match` at every
  consumer site.
- The wire form of each arm (CBOR encoding) is part of item 4 (D2
  v1-canonical-bytes); bytewise locked.

**What's NOT frozen:**
- The WIRE codepoint encoding (item 4 P-III) — that locks the bytes for
  the two arms; additive wire arms are out of scope at the enum level
  (would require both Core re-open + this enum + HALT-AND-SURFACE).

**Verification:** the `tf3b_no_opaque_selector_arm_structural` test;
`cargo-public-api`; the no-opaque-arm sentinel scan in the cite-drift
CI lane.

**Escape valve:** any proposal for `OpaqueSelector` or third arm = HALT-
AND-SURFACE-TO-BEN per RATIFIED-S&C §R1 (structurally unsound per Spike
H+1.1; explicit re-open justification required).

### 15.d — `AuthorizationGrant` envelope = ONE signed artifact

**Frozen surfaces:**
- `crates/benten-caps/src/authorization_grant.rs::AuthorizationGrant` `pub struct
  AuthorizationGrant`:
  - `ucan: UcanEnvelope` (the UCAN half)
  - `key_material: GrantKeyMaterial` (the key-material half; **renamed**
    per name-collision resolution below)
  - `binding_sig: Vec<u8>` (issuer's sig over canonical
    `(ucan, key_material, audience)` bytes)
  - `audience_binding: Cid`
  - `issuer_verifying_key: Vec<u8>` (Ed25519 verifying key)
  - `audience_pubkey: Option<Vec<u8>>` (G-CORE-3e ALPN handler use)
  - The G-CORE-3e `RestrictedScope` scope field (per partial file content)
- **`#[non_exhaustive]`** — APPLY at the freeze wave (per item 11; the
  struct grew between 3b + 3e and will plausibly grow further).
- **Binding-sig-validation-FIRST ordering** semantic: validators MUST
  check `binding_sig` BEFORE consulting UCAN scope or key material
  (RATIFIED-S&C §R3 quoted explicit). Pinned by the `verify_binding`
  implementation + tests.

**ARCHITECTURAL CONCERN — second name collision (orchestrator-decided
per distinctive-angle).** Two `KeyMaterial` types at HEAD: (i)
`crates/benten-caps/src/authorization_grant.rs::GrantKeyMaterial` (the GRANT-bearing
handle carrying audience binding + paths; post-G-CORE-9 row 7 rename;
historical name `KeyMaterial`); (ii)
`crates/benten-crypto-suite/src/aead.rs::AeadKeyMaterial` (the AEAD-bearing key;
post-G-CORE-9 row 7 rename; historical name `KeyMaterial`).
Distinct semantics; same name. **Renames LANDED at G-CORE-9
V1-FROZEN-INTERFACE row 7 (commit `dd12f394`):**
`benten_caps::KeyMaterial` → `GrantKeyMaterial`;
`benten_crypto_suite::aead::KeyMaterial` → `AeadKeyMaterial`. The
compatible-interpretation risk (someone imports the wrong one and
trait-bounds line up enough that it compiles but runtime is wrong)
is structurally prevented by type-level distinction. **Tentatively-
decided per night-shift stance; rebuttable at morning Ben review**
(same provision as item 15.a above).

**Frozen semantics:**
- `binding_sig` is computed by the issuer over the CBOR-encoded
  `(ucan, key_material)` tuple, bound to the same audience.
- Validators check `binding_sig` BEFORE consulting the UCAN scope or
  the key material — the binding is the foundation.
- Consistent across online (custom-ALPN handler in G-CORE-3e) and
  offline (Drop bundle in G-CORE-3f) paths.

**What "frozen" means here:**
- Field set + CBOR encoding bytewise locked (item 4).
- Validation ORDER (binding → UCAN → keys) is part of the contract.
- The ONE-artifact-not-TWO discipline is permanent (avoids the per-
  deployment-shape ambiguity Spike H identified).

**Verification:** existing `tf3b_*` tests pin envelope shape +
binding-sig semantics; `cargo-public-api` (item 9); item 4 byte-pin.

**Escape valve:** changing the binding-sig algorithm or the validation-
ordering invariant = HALT (cryptographic).

### 15.e — Encryption-class enum

**Frozen surfaces:**

`pub enum EncryptionClass { Public, Confidential }` — **LANDED at
G-CORE-9 V1-FROZEN-INTERFACE row 3 (commit `a9d2753c`)** at
`crates/benten-core/src/encryption_class.rs:36`. Placement decision:
`benten-core` (vocabulary-of-encryption-state type, NOT a
capability-policy type; same pattern as `WriteAuthority` +
`ChangeEvent`). `EncryptionClassError` typed-reject error variant
+ `pub mod codepoint` wire table + `from_codepoint(u8) -> Result`
typed dispatch all minted.

```rust
#[non_exhaustive]
pub enum EncryptionClass {
    /// Public — content visible to any reader with the CID; no
    /// confidentiality envelope.
    Public,
    /// Confidential — content encrypted per §1301 substrate; reader
    /// must hold the appropriate `AuthorizationGrant.key_material`.
    Confidential,
    // Reserved future arms (NOT built; documented for forward planning):
    //   AnonymousGroup,   // group-keyed without revealing principal identity
    //   PrivateLocal,     // device-local-only; never sync-eligible
}
```

Reserved future variants (`AnonymousGroup`, `PrivateLocal`) typed at
v1-beta with the typed-reject pattern; explicit-add via additive enum
variants post-v1.

**What "frozen" means here:**
- The two-variant baseline (Public + Confidential) is locked.
- `#[non_exhaustive]` permits additive future variants without re-
  freezing the shape.
- Wire-form per variant is locked (CBOR + codepoint dispatch; item 4).

**Verification:** `cargo-public-api`; a new test pin for the typed-
reject on unknown class variants.

**Escape valve:** new enum variant = HALT-AND-SURFACE-TO-BEN if it
collapses the encryption-class taxonomy (RATIFIED-S&C §R6 + spec item
15(e)).

### 15.f — Two-path key-derivation contract (Interpretation B per Spike E)

**Frozen surfaces:**
- The HKDF-SHA256 `derive_step` API in
  `crates/benten-crypto-suite/src/structural_kdf.rs`:
  - `K(root) = HKDF-SHA256(K_principal, info = "root" || root_cid)`
    (line 124 `derive_root`).
  - `K(N) = HKDF-SHA256(K(predecessor), info = "step" || edge_label ||
    N.cid)` (line 141 `derive_step`).
- `StructuralKdfKey` zeroize-on-drop output type.
- **The `"step"` + `"root"` HKDF info-tags** for cross-role domain
  separation (per Spike E correction + R0.8).
- `derive_step_without_info_tag_for_test` (line 96) — test-only escape;
  pinned as `_for_test`-suffixed (cosmetic rename to `_internal` per
  item 1 visibility-cluster pattern is **BELONGS-NAMED-NOW** to
  v1-assessment-window v1-Composing; safety-orthogonal — lock the
  test-only escape AS-IS at v1-beta).
- Path-tagged keys: a Node reachable by multiple paths gets multiple
  distinct keys (feature for selective-share; envelope records which
  canonical path produced each ciphertext per (g) chunk + per (h)
  walker).
- **KDF = HKDF-SHA256 v1-beta DEFAULT** (codepoint-dispatched per
  CLAUDE.md #5; BLAKE3 stays the content-hash; future codepoint
  additions land additively per item 14).

**What "frozen" means here:**
- The HKDF info-tag convention (`"step"` for step-derivation, `"root"`
  for root) is wire-permanent (a different tag = different key =
  decryption failure).
- The corrected (per Spike E) derivation formula with the explicit
  info-tags is locked; not the literal-DESIGN-doc formula (which Spike E
  proved doesn't converge).
- Owner-derivable via owner-walks-canonical-path; recipient-derivable
  via given-K(N)-recipient-walks-edge-labels — the two-path symmetry is
  the contract.
- KDF codepoint-dispatch is in place; alternative KDFs may be added at
  new codepoints.

**What's NOT frozen:**
- The internal HKDF implementation crate version — `Cargo.lock`
  semantics.

**Verification:** `cargo-public-api`; existing `structural_kdf.rs`
tests; the `tf3a_*` integration tests.

**Escape valve:** mutating the info-tag structure ("step"/"root" →
different strings) = HALT (cryptographic; breaks downstream derivation).

### 15.g — Two-CID mapping contract + per-chunk-AEAD chunk-size constant

**Frozen surfaces:**
- `crates/benten-sync/src/two_cid_store.rs::TwoCidStore` — the wave-3e
  adapter wrapping a ciphertext-bytes backing store + the two-CID
  mapping (plaintext_cid → ciphertext_cid).
- redb-backed `plaintext_cid → ciphertext_cid` mapping table at G-CORE-3d
  (#1323) — durable.
- UCAN scopes against plaintext_cid; iroh-blobs serves ciphertext blob by
  its own hash (preserves "served-bytes-hash == requested-hash"
  invariant).
- `crates/benten-crypto-suite/src/aead.rs:52` `pub const IROH_BLOCK_SIZE:
  usize = 16 * 1024` — the load-bearing chunk-size constant aligned with
  iroh-blobs's native block size per RATIFIED-S&C §R2 (different chunk
  size = double-chunking overhead).
- 64 KiB threshold for chunked-vs-whole-AEAD heuristic (documented per
  `aead.rs:35`). Whole-content AEAD for smaller Nodes.
- AAD-binds-chunk-index semantic (preserves per-chunk decryptability
  for range-fetched slices).

**For Nodes ≥64 KiB:** per-chunk AEAD with chunk size = `IROH_BLOCK_SIZE`
+ AAD-binds-chunk-index. Whole-content AEAD for smaller Nodes.

**What "frozen" means here:**
- The `IROH_BLOCK_SIZE = 16384` constant is wire-format-load-bearing
  (different chunk size = double-chunking overhead; bytewise
  dependency).
- The 64 KiB threshold for chunked-vs-whole is part of the freeze.
- The two-CID mapping shape (plaintext_cid → ciphertext_cid) is the
  permanent storage substrate seam.

**What's NOT frozen:**
- The redb table schema-version (already accommodated via `SnapshotBlob`
  precedent for any future migration).
- The iroh-blobs upstream crate version (Benten consumes via stable
  API).
- The internal redb table layout — implementation detail behind the
  mapping API.

**Verification:** byte-pin (item 4); a test pin asserting
`IROH_BLOCK_SIZE == 16 * 1024` (golden constant).

**Escape valve:** changing the chunk size = HALT (wire-format break +
content-incompatibility).

### 15.h — SubgraphSpec walker IS a Subgraph shipped once in `benten_core`

**Frozen surfaces:**
- `crates/benten-core/src/subgraph_spec/walker.rs:78` `pub fn walk` —
  the canonical BFS-order walker.
- `crates/benten-core/src/subgraph_spec/walker.rs:183` `pub fn
  walker_as_subgraph() -> Subgraph` — the fractal-property pin.
- `WalkResult.enumerated: Vec<(Cid, StructuralPath)>` — the BFS-order
  enumeration shape (RATIFIED-S&C §R4 explicit).
- The BFS-order = canonical path contract (per R4) — recipients walk
  the same BFS the producer enumerated; canonical path carried in
  `AuthorizationGrant.key_material` per (d) + (f).
- The walker is **data-not-evaluator-extension** — no evaluator special-
  case for SubgraphSpec; preserves CLAUDE.md baked-in #1.

**Engine wrapper LANDED at G-CORE-9 V1-FROZEN-INTERFACE row 4 (commit
`7af94d06`)** at `crates/benten-engine/src/engine_share_scope.rs:59`:
`pub fn Engine::walk_share_scope(&self, spec: &Spec) -> Result<WalkResult, EngineError>`.
Wrapper delegates to the canonical `benten_core::subgraph_spec::walker::walk`
BFS enumerator (no engine-side reimplementation; preserves producer/
recipient enumeration parity per §R4). Typed reject routes through
`EngineError::Other { code: ErrorCode::SubgraphSpecWalkFailed, .. }`
(new ErrorCode minted in the same commit; CATALOG_VARIANT_COUNT 191
→ 192). End-to-end test at
`crates/benten-engine/tests/g_core_9_walk_share_scope_e2e.rs` (BFS-
order parity arm + typed-reject arm).

**What "frozen" means here:**
- The walker lives in `benten-core` (not `benten-engine` and not
  `benten-caps`); no duplication elsewhere.
- BFS enumeration order is part of the wire-bytes (path-tagged keys
  depend on it).
- The walker is data-not-evaluator-extension.

**What's NOT frozen:**
- The internal queue+visited-set implementation — opaque.

**Verification:** `cargo-public-api`; walker test pins.

**Escape valve:** evaluator special-case for SubgraphSpec = HALT
(CLAUDE.md #1).

### 15.i — Revocation reach documentation

**Frozen surfaces (DOCUMENTED-DESIGN-CONSTANT, not code-shape):**
- `docs/SECURITY-POSTURE.md` carries an explicit section "Revocation
  reach in encryption-at-rest" documenting:
  - UCAN revocation cuts future serves (cap-policy check fails for
    subsequent requests).
  - Already-derived keys remain decryptable FOREVER. Once Bob has
    derived `K(N)` for some Node, Bob can decrypt any ciphertext he
    obtains for that Node, regardless of UCAN revocation. Re-keying the
    Node requires Alice to re-encrypt + re-issue (a heavy operation).
  - Drop bundles are forever-valid once distributed. Producer has no
    callback to revoke an already-distributed Drop.
  - Mitigation: tight `nbf`/`exp` + key rotation.

**SECURITY-POSTURE Compromise #31 LANDED at G-CORE-9 V1-FROZEN-INTERFACE
row 8e (commit `75a1d33a`)** *(historical mint-event record: this revocation-reach compromise was re-pointed to **#62** at F-full per BR-2; #31 now denotes LAMPS Composite ML-DSA)* — Compromise #31 added to
`docs/SECURITY-POSTURE.md` registry table: "Revocation reach in
encryption-at-rest (already-derived keys remain decryptable; Drop
bundles forever-valid once distributed)" — OPEN ARCHITECTURAL TRADE-OFF;
MITIGATED by tight UCAN `nbf`/`exp` + key rotation; stays OPEN at
v1-beta + v1-GM (inherent to encryption-at-rest where reader holds
plaintext key). Body sections already exist at line 2492 + line 2536
of SECURITY-POSTURE.md.

**What "frozen" means here:**
- The documentation language is the v1-beta posture; users + plugins
  consume it as the security model.
- The cited mitigations (tight `nbf`/`exp` + key rotation) are the v1
  affordances — Composing may implement additional mitigations.

**Verification:** the doc presence test (some doc-coverage CI lane
asserts the section exists).

**Escape valve:** removing or contradicting the design constant in
Composing = HALT.

### 15.j — Resolver evaluation model = live-per-request

**Frozen surfaces (DOCUMENTED CONTRACT, not wire-format):**
- The resolver evaluates a SubgraphSpec against the CURRENT graph state
  on every request (NOT a frozen-snapshot semantics).
- IVM-cache-invalidation seam reuses G-CORE-4's CanonicalViews
  subscription.
- UCANs gate sub-graph SHAPES that EVOLVE (Alice's writes-since-issuance
  flow into Bob's accessible scope automatically).
- The ONLY frozen-snapshot path is the offline-Drop-bundle (Drop =
  snapshot-at-production-time by construction).

**What "frozen" means here:**
- The live-per-request semantics is the v1 contract; consumers depend
  on it.
- The IVM-cache seam interface (whatever it's called at HEAD; verify
  the CanonicalViews subscription surface) is part of the freeze.
- The Drop-bundle = frozen-snapshot dichotomy is permanent.

**What's NOT frozen:**
- The internal caching strategy (LRU / TTL / etc.).
- The CanonicalViews subscription implementation.

**Verification:** test pin demonstrating live-per-request evaluation
(write new in-scope content after UCAN issued → recipient sees it on
next request); doc-coverage CI lane.

**Escape valve:** changing live-per-request to frozen-snapshot
semantics in Composing = HALT.

---

## Composing-phase escape valve (per methodology-r1-5)

Any Composing-time discovery that would require altering a §1.A.FROZEN
surface is a **HALT-AND-SURFACE-TO-BEN event**, NOT an orchestrator-
autonomous Core re-open:

1. The orchestrator STOPS the affected Composing lane.
2. Records in the Composing R6/handoff the specific frozen surface +
   the required change + why Composing cannot proceed without it.
3. Surfaces it to Ben as a decision (plain-English + options +
   prediction per the standing surface discipline).

**Structural backstop:** the `cargo-public-api` gate + the #1204 TS-side
parity gate (BUILD-AT-FREEZE-WAVE row 2) + the per-shape byte-pin tests
+ the `#[non_exhaustive]` sweep + the cite-drift scanners + the
workspace `missing_docs` test catch silent frozen-surface mutations
structurally — the gate makes silent re-open structurally impossible,
not merely policy-forbidden.

---

## Triage decision provenance

**Distinctive-angle decisions applied at triage:**

| Spec item | Planner-A | Planner-B | Orchestrator decision | Rationale |
|---|---|---|---|---|
| §8-A visibility cluster (item 1) | Aggressive tighten + rename + DELETE `_for_test` | Conservative: keep `get_node` `pub`, soft-defer | **Planner-A wins** | Spec item 1 ratification "DECIDED `pub`→`pub(crate)` + rename + drop `_for_test`" is explicit; HARD RULE 12 → FIX-NOW; cascade cost is real but worth it pre-freeze (agent-economics: thorough cleanup) |
| §8-E sealed-CapabilityPolicy (item 8) | Hard-seal at G-CORE-9 | Soft-seal lock + G-CORE-8.3 deferred | **Planner-A wins** | Spec item 8 ratification names "private `Sealed` supertrait in a non-pub module"; post-v1 hardening is breaking; pre-freeze is the cheap correct call per `feedback_agent_economics_prefer_thorough_cleanup` |
| `#[non_exhaustive]` sweep (item 11) | Maximalist over the full 158-enum workspace surface | Defensive per-type, minimal per-variant | **Planner-A wins (maximalist)** | Spec item 11 "ONE coherent freeze-wave over the FULL enumerated workspace surface"; cost-asymmetry favors application; `Scope` carve-out preserved per item 15(c) |
| `// FROZEN:` marker comments + CI lint (NEW pim-N candidate) | Add | REJECT | **Planner-B wins** | The existing `cargo-public-api` baseline + #1204 TS parity gate ARE the structural backstop per spec methodology-r1-5; don't add ceremony beyond the spec. Noted as candidate pim-N if structural-backstop alone proves insufficient |
| IPC const-allowlist (item 13) | Keep | Keep | **Both agree** | Per CLAUDE.md #19 engine-extensions-compile-time-linked |
| `EncryptionClass` enum mint (item 15(e)) | Mint unconditionally | Conditional on §8-CC consumers | **Mint unconditionally (Planner-A direction)** | Defensive call per orchestrator: deferring leaves spec reference as a phantom destination per HARD RULE 12; mint at wave time |
| Name-collision renames (15.a + 15.d) | Rename for clarity | Accept duplication | **Rename per orchestrator distinctive-angle** | Compatible-interpretation trap (import the wrong type; trait bounds line up enough that it compiles but runtime is wrong); tentatively-decided per night-shift stance, rebuttable at morning Ben review |
| `MerkleRangeProofBackend` placement (item 3 / §4.64) | Lock above storage in `benten-sync` | NOT FROZEN; BELONGS-NAMED-NOW G-COMP-1 | **Verify-or-build per build-backlog row 5** | Currently only narrative references at HEAD; verification step decides freeze-now vs G-COMP-1 |

**Items where both planners agreed** (locked as common ground):
items 4 (P-III scheduled Ben decision-point), 5 (`WriteContext` shape +
non_exhaustive), 7 (§4.33 deletion already discharged), 9 (cargo-public-
api baselines), 12 (G-CORE-8 security-shape), 14 (P2P conformance), 15.a
(SubgraphSpec primitive), 15.b (6-dim product), 15.c (Scope two-arm
carve-out), 15.f (KDF info-tags), 15.g (chunk-size constant), 15.h
(walker placement), 15.i (revocation reach doc), 15.j (resolver live-
per-request).

---

## Provenance + cross-cites

**Triage authoring:** Round 0.5 triage-synthesis agent on
`g-core-9/triage-synthesis` branch, 2026-05-23 post-#1342 main HEAD
`ae7cd3d5` (CATALOG_VARIANT_COUNT 191; 14 workspace crates). Source
drafts: `g-core-9/planner-a-architectural-purist` @ `7ef3f279` (1411
LOC) + `g-core-9/planner-b-conservative-minimal` @ `f5aaebb0` (1290
LOC). Companion build-backlog at
[`docs/V1-FROZEN-INTERFACE-BUILD-BACKLOG.md`](V1-FROZEN-INTERFACE-BUILD-BACKLOG.md).

**Authoritative inputs (read in addition to this artifact):**
- `.addl/phase-4-meta/00-implementation-plan.md` §1.A C1-C13 +
  §1.A.FROZEN (15 items, lines 111-148).
- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
  (R1-R6 ratifications + 8 spike-derived refinements).
- `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md` (PQ-
  hybrid default + audit-gates-GM + the four landscape passes).
- `.addl/phase-4-meta/RATIFIED-crypto-agility-2026-05-18.md`
  (multiformats-permanent framing).
- `.addl/phase-4-meta/RATIFIED-prework-forks-2026-05-18.md` (§8-A
  tighten + §8-E sealed + §8-C bridged-dual-runtime).
- `CLAUDE.md` Architectural Decisions Baked In items 1, 5, 7, 15, 17,
  18, 19.

**End of triage-synthesis draft. Iterate-to-convergence council convenes
next.**
