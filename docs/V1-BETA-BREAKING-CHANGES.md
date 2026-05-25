# V1-Beta Breaking Changes Ledger — Phase-4-Meta-Core (8141b94..HEAD)

> **Purpose:** consolidated catalog of all break-OK changes shipped in
> the Phase-4-Meta-Core campaign (open at `8141b94` post-Phase-4-Foundation-tag
> → close at `phase-4-meta-core-close` tag). This is the v1-beta
> release-notes draft + the L18 "Net catalog of breaking changes ready
> for the v1-beta release notes?" answer.
>
> **Authority:** every entry is a Ben-ratified P-III no-users-yet
> override OR pre-v1-API-shape ratification OR
> orchestrator-night-shift-surface-arch-decision per the foundational
> memories (`feedback_surface_arch_decisions_under_auth`,
> `feedback_no_defer_HARD_RULE`, CLAUDE.md baked-in #5/#7/#15/#17/#18/#19).
>
> **Audience:** downstream consumers preparing for `v1-beta` adoption +
> the next-phase agent dispatching post-`phase-4-meta-core-close`.
>
> **Update discipline:** every PR in the merged range
> `8141b94..HEAD` whose body or commit message labels a change as
> "break-OK" / "wire-format change" / "rename" / "P-III no-users-yet
> override" / "pre-v1-API-shape" gets a row here. The G-CORE-9 R1 R2
> council verifies completeness.

---

## Cohort 1 — Wire-format byte-shape changes

### PR #1331 — SnapshotBlob v1 → v2 format-version bump
- **What changed:** SnapshotBlob on-disk format version discriminator bumped from v1 to v2.
- **Why break-OK:** P-III no-users-yet override per CLAUDE.md baked-in #5 (canonical-bytes contract).
- **Migration path:** any pre-v2 SnapshotBlob bytes are unreadable by v1-beta; tests + fixtures regenerated.
- **Anchor:** `crates/benten-graph/src/snapshot.rs` format-version constant.

### PR #1341 — `Cid::from_bytes_for_test` → `Cid::from_raw_bytes`
- **What changed:** test-helper renamed; production-API gains a non-test-suffixed entry point.
- **Why break-OK:** test-only surface had `_for_test` suffix which is a "red-flag" per V1-FROZEN-INTERFACE item 1 discipline; the rename makes the production API real (called by adversarial cross-codepoint test fixtures).
- **Migration path:** call-site rename across all consumers.

### PR #1342 — Cipher-suite codepoint `0x647c` mint (PURE_PQ_MLKEM768_ONLY)
- **What changed:** new reserved codepoint `0x647c` lights at the codepoint table; reachable only via the C11b audit-gated constructor `SwapMatrix::try_pure_pq_sole_trust_path` which returns `AuditNotLandedPurePqRejected` at v1-beta (compile-time gate).
- **Why break-OK:** additive at the codepoint table (RATIFIED-pq-default-reframe-2026-05-19); no breaking surface change.
- **Migration path:** none; reserved arm.

### PR #1342 — `ErrorCode::DslIoError` mint (closes §3.5g item 6)
- **What changed:** new ErrorCode variant + TS mirror class + drift-detect baseline grandfathering removed.
- **Why break-OK:** new variant on `#[non_exhaustive]` enum is additive per the §3.5g cross-language mirror contract.
- **Migration path:** none (additive).

---

## Cohort 2 — Public-API shape changes (renames + visibility)

### PR #1344 — Rename `RestrictedSpec` → `RestrictedScope` (caps side) + `RestrictedSpec` → `SubgraphSpecRestriction` (core side)
- **What changed:** type rename per RATIFIED-S&C 2026-05-21 §R3 (clearer semantic — "Scope" mirrors UCAN nomenclature; "SubgraphSpecRestriction" disambiguates from the cap-side Scope).
- **Why break-OK:** pre-v1-API-shape ratification per night-shift orchestrator-surface-arch-decision (Ben-ratified post-spike-sequence reflect-together 2026-05-21).
- **Migration path:** call-site rename via the post-rename re-exports at `benten_caps::{RestrictedScope, SubgraphSpecRestriction, GrantKeyMaterial}`.

### PR #1344 — Rename `KeyMaterial` → `GrantKeyMaterial` (caps side); `AeadKeyMaterial` is the crypto-suite distinct type
- **What changed:** disambiguating rename — `GrantKeyMaterial` is the cap-layer envelope shape; `AeadKeyMaterial` is the crypto-suite-internal AEAD key newtype.
- **Why break-OK:** pre-v1-API-shape ratification.
- **Migration path:** call-site rename + module-path updates.

### PR #1344 — `CapabilityPolicy` hard-seal (private `sealed::Sealed` supertrait)
- **What changed:** external `impl CapabilityPolicy for X` is now compile-error per CLAUDE.md #7 sealed-discipline refinement (Ben-ratified 2026-05-18).
- **Why break-OK:** v1-beta-locked decision per CLAUDE.md #7; workspace-test opt-in available via `testing` feature + `__sealed_for_workspace_tests::Sealed` re-export.
- **Migration path:** external policy authors cannot implement `CapabilityPolicy` directly; route through one of the provided backends (`NoAuthBackend` / `UcanGroundedPolicy` / `GrantBackedPolicy` / `LegacyUcanStubBackend`) OR delegate to the engine via `Engine::install_capability_policy` constructor pattern.

### PR #1344 — `#[non_exhaustive]` sweep across ~14 lens-scoped pub types (initial wave; completion in this G-CORE-9 R1 fix-pass)
- **What changed:** the maximalist `#[non_exhaustive]` sweep per V1-FROZEN-INTERFACE item 11; locks SemVer-additive-extension for all enumerated workspace pub types.
- **Why break-OK:** v1-beta-load-bearing per spec item 11 + §15.d.
- **Migration path:** external direct-struct-literal construction blocked; use `Default::default()` + field-mutation OR provided constructors.

### PR #1344 — `EncryptionClass` enum mint (benten-core)
- **What changed:** new pub enum at benten-core orthogonal to the cipher-suite dispatcher (outer Public/Confidential class layer; inner cipher-suite codepoint dispatches the suite).
- **Why break-OK:** pre-v1-API-shape ratification per the S&C ratified architecture.
- **Migration path:** none (additive).

### PR #1344 — `Engine::walk_share_scope` addition + Compromise #31 entry
- **What changed:** new principal-unbearing `walk_share_scope` API at engine_share_scope.rs:59 per the S&C ratified architecture; principal-bearing `walk_share_scope_as` is deferred-additive to G-COMP-1 per V1-FROZEN-INTERFACE-DEFERRED.md Row D-11.
- **Why break-OK:** pre-v1-API-shape ratification.
- **Migration path:** none (additive); the principal-bearing variant is the post-v1-beta hardening (Row D-11).

### PR #1338 — `ManifestEnvelopeRecheckOutcome` `NotApplicable` → `UnresolvedDeny` rename (per §4.36 fail-CLOSED flip) (landed in Strategy-C batch PR #1340)
- **What changed:** the rename is NOT cosmetic — it changes the semantic from "rechecker has no context, admit" to "rechecker could not resolve, fail-CLOSED". Adds 3 new `CapabilityPolicy` hooks (`check_install_consent` + `check_per_delegation` + `check_write_with_audience`) as defaulted-impl methods (object-safety preserved).
- **Why break-OK:** security-r1-2 BLOCKER closure per Phase-4-Meta-Core G-CORE-8.
- **Migration path:** custom rechecker impls must emit `UnresolvedDeny` rather than `NotApplicable` when their own internal resolution fails; custom `CapabilityPolicy` impls inherit the 3 new hooks via default-impl delegation to `check_write`.

### PR #1340 — `StreamHandle::next` sync → async migration (napi PR-B)
- **What changed:** the napi `StreamHandle.next` is now an `AsyncTask` wrapper (`NextChunkTask`) running on libuv worker per napi-rs 3 contract; the sync `next_chunk_with_timeout` is the Rust-side primitive.
- **Why break-OK:** v1-beta-locked per the napi-async-completion window; closes #1203 napi PR-B (V1-FROZEN-INTERFACE item 10).
- **Migration path:** JS consumers `await` the next-chunk promise; sync access is via direct Rust-side `StreamHandle::next_chunk_with_timeout`.

### PR #1340 — Option-C Mutex-split cancellation-stopgap (closes #652)
- **What changed:** `StreamHandle` split into `Mutex<Option<StreamHandle>>` so `close()` can opportunistically `try_lock` + cancel a `next()` that is blocked.
- **Why break-OK:** pre-v1 small hardening per CLAUDE.md baked-in #15 v1-gate-refactor.
- **Migration path:** none (transparent to JS callers).

### PR #1310 — `module_ecosystem::install_plugin*` deletion (G-CORE-0 §4.33)
- **What changed:** legacy install-path (pre-Phase-4-Foundation `module_ecosystem::install_plugin` family) deleted; canonical install pipeline is `benten_platform_foundation::plugin_lifecycle::install_plugin`.
- **Why break-OK:** legacy-path-deletion-freeze per V1-FROZEN-INTERFACE item 7 (Phase-4-Foundation R6 ratification).
- **Migration path:** consumers route through `plugin_lifecycle::install_plugin`.

### PR #1238 — `DeviceRevocation` deletion + `Acceptor` demotion (COLLAPSE P0+P1)
- **What changed:** `DeviceRevocation` no longer a distinct trust-root primitive; `Acceptor` demoted to internal helper; the device envelope is a ceiling the chain-validation seam enforces (per DECISION-RECORD §4).
- **Why break-OK:** architectural collapse per spec; SemVer break of obscure surfaces.
- **Migration path:** the consolidated chain-validation seam at `crates/benten-caps/src/manifest_envelope_chain_validation.rs::validate_chain_with_manifest_envelope` covers the prior `Acceptor` use cases.

### PR #1271 — Chain-validation seam consolidation
- **What changed:** the ONE chain-validation seam (per COLLAPSE P2) consolidates the prior fragmented per-cap chain-validation entry points.
- **Why break-OK:** architectural collapse per spec.
- **Migration path:** route through the single seam.

### PR #1276 — Unified envelope-ceiling (COLLAPSE P3)
- **What changed:** the inbound device `CapabilityEnvelope` is AND'd into the inbound writer's effective caps via the ONE seam — REPLACES the deleted `benten_id::Acceptor` trust pipe.
- **Why break-OK:** architectural collapse per DECISION-RECORD §4.
- **Migration path:** none — the engine consumes the unified ceiling internally.

### PR #1294 — `AllPermit` deletion
- **What changed:** the `AllPermit` "any cap" wildcard is deleted; cap permits must be explicit.
- **Why break-OK:** security hardening per L1 chain-validation semantics.
- **Migration path:** explicit cap-pattern enumeration.

### PR #1295 — Workspace-wide `to_canonical_bytes` rename + `Cid::from_str` shadow deletion + `WriteAuthority`/`OperationNode` non_exhaustive + `ChangeEvent::minimal` rename (P-II)
- **What changed:** P-II workspace-wide renames + #[non_exhaustive] additions per consolidation pass.
- **Why break-OK:** pre-v1-API-shape consolidation.
- **Migration path:** call-site renames + struct-literal → constructor pattern.

### PR #1299 — napi PR-A (StreamHandle scaffolding)
- **What changed:** napi `StreamHandle` class scaffolding lands (companion to #1340 PR-B which migrates `next` to async).
- **Why break-OK:** v1-beta-locked.
- **Migration path:** none (additive at v1-beta).

### PR #1304 — `WriteContext::namespace_did` add
- **What changed:** `WriteContext` gains an optional `namespace_did: Option<Did>` field per the #989 storage-partition seam (CLAUDE.md baked-in #18 multi-tenant substrate).
- **Why break-OK:** new field on `#[non_exhaustive]` struct is additive per the freeze contract.
- **Migration path:** none (additive); consumers needing per-DID partitioning populate the field.

### PR #1339 — DSL `Diagnostic` field → method + `Emit` → `Build` arm rename + `CompileError::Backend` arm
- **What changed:** chunk-3 #790/#839/#1000 closures — `Diagnostic.error_code` becomes `error_code()` method; `CompileError::Emit` renamed to `Build`; new `CompileError::Backend` arm for engine-registration failures.
- **Why break-OK:** pre-v1-API-shape per the DSL chunk-3 ratification.
- **Migration path:** call-site field-access → method-call; arm-rename in match-statements.

---

## Cohort 3 — G-CORE-9 R1 fix-pass additions (this PR)

### G-CORE-9 R1 Bundle 3 — `#[non_exhaustive]` sweep completion (~14 additional pub types)
- **What changed:** completion of the V1-FROZEN-INTERFACE item 11 maximalist sweep — adds `#[non_exhaustive]` to `CapWriteContext` + `ReadContext` + `WriteBoundaryChainOutcome` + `AtriumMode` + `SuspensionOutcome` + `DelegationResolution` + `NextChunkPoll` + `ManifestVerifyMode` + `AuthorizationGrant` + `GrantKeyMaterial` + `UcanEnvelope` + DSL `CompileError`/`CompiledSubgraph`/`CompiledPrimitive`/`Span`/`Diagnostic`.
- **Why break-OK:** V1-FROZEN-INTERFACE item 11 contract — adding post-v1-beta is breaking; adding NOW with the freeze is the cheap forward-compat affordance.
- **Migration path:** external direct-struct-literal blocked; use `Default::default()` + field-mutation pattern OR provided constructors (e.g. `GrantKeyMaterial::from_bytes_for_test` for the benten-drop case).

### G-CORE-9 R1 Bundle 4 — Strategy::C → Strategy::Reserved rename (anticipated; deferred to follow-up sub-pass per HARD RULE 12 named-destination)
- **What changed:** `Strategy::C` arm renamed to `Strategy::Reserved` to reflect the post-G-CORE-8 hidden-strategy semantic; locks-out the obsolete C name from v1-beta wire-bytes.
- **Why break-OK:** wire-format strings + ErrorCode variant name forever-locked at v1-beta freeze.
- **Migration path:** call-site rename; ErrorCode `ViewStrategyCReserved` → `ViewStrategyReserved`; format strings updated.
- **STATUS in this PR:** anticipated mention; the Strategy rename + DSL 3 ErrorCode mints (Bundle 4) ESCALATED per the Bundle-3 substantial cascade demonstrating the per-bundle LOC explosion; relocated to a follow-up sub-pass for orchestrator pacing. BELONGS-NAMED-NOW: G-CORE-9 R2 OR G-COMP-1 §<row>. **Captured in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-19** (the prior cite "Row D-15 watch-list" was the phantom-destination Pattern α the R2 council surfaced; R2 fix-pass authors the actual Row D-19).

### G-CORE-9 R1 Bundle 9 — V1-FROZEN-INTERFACE-DEFERRED.md authorship
- **What changed:** new tracked doc enumerating 16 deferred consumption surfaces per Fork 2 doc-tighten ratification.
- **Why break-OK:** doc-only; no surface change.
- **Migration path:** see DEFERRED doc for per-row G-COMP-1 destinations.

### G-CORE-9 R1 Bundle 10 — cargo-public-api workflow flip to required-failing (Fork 3)
- **What changed:** `.github/workflows/cargo-public-api.yml` removes `|| true` bypass; drift = workflow fails.
- **Why break-OK:** per spec item 9 literal — the freeze gate must bite.
- **Migration path:** PRs introducing public-API changes must update the appropriate `docs/public-api/<crate>.txt` baseline in the same PR. Branch-protection inclusion is a post-merge admin action.

### G-CORE-9 R1 Bundle 11 — AAD layout doc-retract per Fork 1
- **What changed:** V1-FROZEN-INTERFACE.md AAD claim retensed from 3-tuple to 2-tuple matching as-shipped `aead_per_chunk`; `total_chunks` defense deferred to G-COMP-1.
- **Why break-OK:** doc-only; preserves existing per-chunk byte-pin tests as the wire-format contract.
- **Migration path:** none at v1-beta; G-COMP-1 may augment.

### G-CORE-9 R1 Bundle 2 — Compromise #26 retense + V1-FROZEN-INTERFACE item 12 retense
- **What changed:** doc-tighten to distinguish v1-beta signature-frozen-and-consumed (the structural empty-peer-DID fail-CLOSED at apply_atrium_merge) vs signature-frozen-consumption-deferred (the substantive ProductionManifestEnvelopeRechecker + accept_atrium_share).
- **Why break-OK:** doc-only; preserves the seam-half live + names G-COMP-1 destinations per HARD RULE 12 clause-(b).
- **Migration path:** consumers consume the seam-half at v1-beta; substantive per-DID enforcement is G-COMP-1.

### G-CORE-9 R1 Bundle 11 — L2-MAJ-1 empty-peer-DID synthesized-fallback hardening (DEFERRED to G-COMP-1 Row D-18)
- **Status:** **DEFERRED — not landed at v1-beta.** An initial always-on filter at `crates/benten-engine/src/engine.rs::apply_atrium_merge` (symbol-form per §3.5b HARDENED point 3) was attempted (commit `34053ed4`) then reverted (commit `3d6f4d66 — "defer L2-MAJ-1 synthesized-peer-DID hardening to G-COMP-1 Row D-18"`) because the always-on form over-fires for the default-Noop test fixtures which intentionally do not register peer-DIDs. The R2 lens (`r2-l2-adversarial-threat-model`) raised this stale claim as L2-R2-BLK-1; R2 fix-pass retenses this row to honesty.
- **What v1-beta ships:** `crates/benten-engine/src/engine.rs::apply_atrium_merge` carries an inline comment naming the deferral rationale at the `ManifestEnvelopeRecheckUnresolvedDeny` arm (the proper closure couples synthesized-fallback rejection to substantive-rechecker-installed detection; gap closes at Row D-4 closure when ProductionManifestEnvelopeRechecker becomes responsible for its own per-DID resolution).
- **Why break-OK:** doc-honesty retense of an attempted-then-reverted change; no public-API impact.
- **Migration path:** none — at v1-beta the always-mounted Noop rechecker admits everything per Compromise #26 disclosure; under a substantive ProductionRechecker (G-COMP-1 deliverable per Row D-4), the rechecker is responsible for synthesized-DID rejection. See `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-18.

### G-CORE-9 R1 Bundle 11 — L2-MIN-2 empty-DID-string structural defense at chain-validator
- **What changed:** `validate_chain_with_manifest_envelope` rejects empty-DID-string root BEFORE consulting `user_registry`.
- **Why break-OK:** defense-in-depth; no public-API change.
- **Migration path:** none.

### G-CORE-9 R1 Bundle 3a — napi match-arm forward-compat wildcards on SuspensionOutcome + NextChunkPoll
- **What changed:** napi binding match-arms in `bindings/napi/src/wait.rs` (SuspensionOutcome) + `bindings/napi/src/stream.rs` (NextChunkPoll) gain a `_ => Err(...)` wildcard arm with `#[allow(unreachable_patterns)]` for forward-compat per the `#[non_exhaustive]` contract.
- **Why break-OK:** externally-observable behavior change at the napi boundary (previously exhaustive match; now has a documented forward-compat fall-through that can fire if a future Rust-side non_exhaustive variant arrives ahead of the napi binding being rebuilt). The Err string is forward-compat-named.
- **Migration path:** none for transparent paths; JS consumers calling either surface and pattern-matching on the Err message should treat the new forward-compat Err as a "rebuild required" signal.

### G-CORE-9 R1 Bundle 3b — ManifestEnvelopeRecheckOutcome FULL `#[non_exhaustive]` (Bundle 3 narrative retense)
- **What changed:** `ManifestEnvelopeRecheckOutcome` at `crates/benten-engine/src/manifest_envelope_recheck.rs:83` carries `#[non_exhaustive]` FULLY at v1-beta (the R1 lens snapshot called it "PARTIAL"; ground-truth at HEAD is FULL).
- **Why break-OK:** R1-snapshot language clarification; the attribute IS applied at v1-beta per Bundle 3 of the R1 fix-pass.
- **Migration path:** none (the substantive change rode with Bundle 3).

### G-CORE-9 R1 Bundle 11b — V1-FROZEN-INTERFACE.md AAD layout doc-retract (3-tuple → 2-tuple)
- **What changed:** the documented AAD layout retensed from a 3-tuple `(chunk_index, total_chunks, plaintext_cid)` to the as-shipped 2-tuple `(plaintext_cid, chunk_index)` per Fork 1 ratification. Cross-doc-mirrored at `docs/V1-WIRE-FORMAT-INVENTORY.md` row 4.
- **Why break-OK:** retense of a previously-documented (between-R0.5-plan and the FREEZE wave) wire-format-contract claim to match as-shipped reality. The `total_chunks` defense against cross-chunk-truncation is deferred to G-COMP-1 per V1-FROZEN-INTERFACE-DEFERRED.md Row D-15a.
- **Migration path:** none at v1-beta; consumers reading earlier R0.5/R2 drafts that referenced a 3-tuple AAD should update to the as-shipped 2-tuple. G-COMP-1 may augment.

### G-CORE-9 R2 fix-pass additions

Per L18-r3-1 closure (R3 ledger-completeness audit): the R2 fix-pass
itself shipped 2 break-OK changes that were not enumerated as dedicated
rows. Added at G-CORE-9 R3 fix-pass:

#### G-CORE-9 R2 Bundle R2.8 — TypedOutputProjection + KernelOutput `#[non_exhaustive]` (wire-bytes-load-bearing)
- **What changed:** `TypedOutputProjection` + `KernelOutput` at `crates/benten-ivm/src/subgraph_spec.rs::TypedOutputProjection` + `crates/benten-ivm/src/subgraph_spec.rs::KernelOutput` carry `#[non_exhaustive]` at v1-beta per L8-R2-MAJOR-CARRY-2 closure. Wire-bytes-load-bearing per the 1-byte arm-discriminator at `crates/benten-ivm/src/algorithm_b.rs` (view round-trip / shape-pin tests).
- **Why break-OK:** wire-bytes-load-bearing types must lock the variant-set at v1-beta freeze; adding post-v1-GM would be a SemVer break for any out-of-crate consumer matching exhaustively. The attribute lands NOW (NOT deferred to G-COMP-1 — explicitly distinguished from the Row D-17 deferred set).
- **Migration path:** within-crate exhaustive matches at `algorithm_b.rs` round-trip + 5 view_2/view_4/view_5 round-trip/shape-pin test sites are unaffected (within-defining-crate). Out-of-crate consumers add a wildcard arm.

#### G-CORE-9 R2 Bundle R2.5 — `SwapMatrixError` cipher-suite resolve-failure variant change
- **What changed:** `SwapMatrixError::CipherSuite(static)` → `SwapMatrixError::Unsupported(UnsupportedAlgorithm)` for cipher-suite resolve-failure routing. Surfaces typed-reject through the unified `UnsupportedAlgorithm` channel.
- **Why break-OK:** consumers pattern-matching on `SwapMatrixError::CipherSuite(..)` for cipher-suite resolve-failure no longer match — the new shape is `SwapMatrixError::Unsupported(UnsupportedAlgorithm::Cipher(..))`. Single-call-site change at v1-beta (no production consumers outside the swap-matrix dispatch).
- **Migration path:** consumers handling cipher-suite resolve-failure update arm `SwapMatrixError::CipherSuite(..)` → `SwapMatrixError::Unsupported(UnsupportedAlgorithm::Cipher(..))`.

---

## Cohort 4 — Wire-format DEFERRED (post-v1-beta hardening watch-list)

These are NOT shipped at v1-beta; named here for downstream-implementer visibility:

- **Row D-15a — AAD `total_chunks` augmentation** (G-COMP-1 destination)
- **Row D-13 — `structural_kdf` info-tag codepoint-binding** (G-COMP-1 destination)
- **Row D-15b — `CryptoPolicy::require_hybrid_pq` consumer-side flag** (Phase-4-Meta-Composing v1-assessment-window)
- **Row D-15c — `AuthorizationGrant.audience_pubkey` Option→non-Option promotion** (Phase-4-Meta-Composing)
- **Row D-15d — `AeadEnvelope::to_wire_bytes` nonce-panic → Result** (G-COMP-1)
- **Row D-15e — `AuthorizationGrant.binding_sig` hardcoded `[u8; 64]` → varsig-tagged** (post-audit)

See `docs/V1-FROZEN-INTERFACE-DEFERRED.md` for the full per-row enumeration.

---

## Cohort 5 — Public-API DEFERRED (post-v1-beta tighten watch-list)

These are public-API tightens / additive surfaces named-deferred to a follow-up wave; surfaced here so downstream consumers reading the consolidated breaking-changes view see them per L18-r2-2 R2 lens recommendation.

- ~~**Row D-7 — §8-A Engine visibility cluster tighten + napi cascade**~~ **CLOSED at R6 R1 FP-A Bundle F2 (2026-05-24)** — the four methods (`Engine::get_node` → `pub(crate) fn read_node`, `Engine::put_node` → `pub(crate) fn put_node_inner`, `Engine::get_node_label_only` → `pub(crate) fn read_node_label_only`, `Engine::resolve_subgraph_cid_for_test` → `pub(crate) fn resolve_subgraph_cid_inner`) all tightened at v1-beta. Napi `Engine::get_node` migrated to `read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, ...)`. ~80 sibling-crate integration tests preserved via cfg-gated test-helper re-exports in `crates/benten-engine/src/testing.rs` (no per-test migration). External callers needing un-attributed reads use `Engine::read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, cid)` (the always-on sentinel constant minted at `crates/benten-engine/src/internal_principal.rs` + re-exported at the crate root). See V1-FROZEN-INTERFACE-DEFERRED.md ~~Row D-7~~ for forensic context.
- **Row D-11 — `walk_share_scope_as` principal-bearing additive overload** (G-COMP-1 destination) — `Engine::walk_share_scope` is principal-unbearing at v1-beta; the principal-bearing variant for recipient-side path-tagged-key derivation is additive Composing-time enhancement per RATIFIED-S&C §R4.
- ~~**Row D-19 — G-CORE-9 R1 Bundle 4 ESCALATED items**~~ **CLOSED at R6-R2-FP-integration-redo Group C (Cohort 8 above; 2026-05-25)** — Strategy::C → Reserved rename + 3 DSL ErrorCode mints landed atomically as the Row D-19 G-COMP-1 wave WIRE-NOW per the v1-beta-freeze-window auto-WIRE-NOW discipline. CATALOG_VARIANT_COUNT 194 → 197. See Cohort 8 above for full enumeration; see `docs/V1-FROZEN-INTERFACE-DEFERRED.md` ~~Row D-19~~ for forensic context.
- **Row D-17 (extended) — `#[non_exhaustive]` cascade for ~12+ lens-scoped pub types** (G-COMP-1 destination) — CapWriteContext / ReadContext / SuspensionOutcome + the extended set from L8-R2-MAJOR-CARRY-2 (UserViewInputPattern / TraceStep / StreamCursor / SubscribeCursor / EngineViewsHandle / AtriumConfig / SyncStatus + the outcome.rs 13-pub-struct set + benten-ivm SubgraphSpec/KernelInput/View* + benten-platform-foundation Vocab*/Scalar/RenderError + Mode). **Wire-bytes-load-bearing types (TypedOutputProjection + KernelOutput) were CLOSED at G-CORE-9 R2 (Bundle R2.8) and are NOT deferred.**
- **Row D-20 — L6-r1-3 trybuild compile-fail regression backstop** (G-COMP-1 destination) — the CapabilityPolicy hard-seal MECHANISM IS structurally enforced by rustc at v1-beta; only the explicit negative-arm compile-fail test fixture is deferred.
- ~~**Row D-22 — workspace `pub fn .*_for_test` `#[cfg]` gating sweep**~~ **CLOSED at R6 R1 FP-A Bundle F1.a-e (2026-05-24)** — workspace cfg-gating sweep COMPLETE. 70+ `pub fn .*_for_test*` declarations across 11 crates gated under `#[cfg(any(test, feature = "testing"))]` (or `feature = "test-helpers"` for benten-engine). 14 production-shaped items remain `pub` per the EXEMPT_PUB_ITEMS allow-list at `tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs` + V1-FROZEN-INTERFACE-DEFERRED.md ~~Row D-22~~ EXEMPT section. 8 affected cargo-public-api baselines regenerated; 7 CI workflows extended `--features` lists with the 6 new `testing` chains. No-regression test pin lives at `tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs` (`no_ungated_pub_for_test_symbols_in_production_source` + `exempt_list_entries_all_exist`). See V1-FROZEN-INTERFACE-DEFERRED.md ~~Row D-22~~ for forensic context.
- **Row D-25 — V1-BETA-BREAKING-CHANGES.md ledger completion sweep** (G-COMP-1 destination per R6-FP-D path-(b) close of L18-r6-1 + L18-r6-2) — at v1-beta this ledger enumerates 15 of ~42 PRs in the phase-4-meta-core window with substantive break-OK content (36% coverage); the remaining ~13 substrate-canary + Strategy-C-consolidation PRs (#1319 G-CORE-3a + #1323 G-CORE-3d + #1324 G-CORE-3b + #1307 G-CORE-2 + #1309 G-CORE-5 + #1311 G-CORE-4 + #1312 G-CORE-7 + #1325 Strategy-C wave-1 incl DSL chunk-1 + #1326 DSL chunk-2 + #1235 Wave-1/2 incl benten-graph trait shape (note: #1237 was CLOSED-not-merged; corrected per R6-R2-FP-C cite-grep-verify discipline) + #1251 #707-trust-subset + the 6 Strategy-C drain batches #1261/#1262/#1269/#1277/#1282/#1290) carry cumulative ~+321 pub-surface additions + ~-75 pub-surface deletions left as a named-deferred completion sweep per HARD RULE 12 clause-(b). Closure: a G-COMP-1 wave authors per-PR Cohort 2 rows for the substrate canaries + 1 roll-up row for the Strategy-C drains. Forward-protection: brief-template mandate (mirror of Row D-22 sub-task 6) requires future fix-pass PRs to enumerate their own break-OK additions in the ledger as part of the §3.5b post-fix-doc-coupling pre-flight. Cross-cite: full per-PR enumeration + per-PR pub-delta evidence at `.addl/phase-4-meta/r6-l18-breaking-change.json` l18-r6-1 finding (the lens's path-(b) recommendation). The R4b-FP precedent (commit 8240a56c authoring Rows D-23 + D-24) validates the path-(b) machinery.

---

## Cohort 6 — R6 R1 FP-F4 substrate-frozen-but-consumer-unwired wiring (this PR)

This cohort lands at PR<F4> (R6 R1 FP-F4) per the F4 design pipeline's
synthesis v3 + Ben PM-ratified F1 path-(a) full ~13-site cascade. The
substrate types frozen at G-CORE-8 are now wired through their
production consumers; Rows D-1, D-2, D-3 (all three sub-rows), D-4,
D-6, D-18 in V1-FROZEN-INTERFACE-DEFERRED.md all close at this PR.

### Public-API shape changes

- **`InstallPorts.install_record_replay_check`** — drops the
  `Option<&mut Fn>` wrapper for `&mut InstallRecordReplayCheckFn`. The
  `None` arm silently disabled the §4.37 TOCTOU replay defense in
  shipped binaries; the drop forces every caller to make an explicit
  choice between substantive defense (production) and explicit no-op
  (tests). Migration: production callers wire
  `engine.install_record_replay_store().record_and_check` closure; test
  fixtures wire `benten_platform_foundation::testing::noop_replay_check()`.

- **`InstallPorts.policy: &dyn InstallConsentPolicy`** — NEW field
  threading the install-time consent policy (CRITIC-2 F-1.2 — via
  port, NOT via `Engine::capability_policy()` accessor per Class B β
  sealed-discipline). The new trait lives at
  `benten_platform_foundation::install_consent::InstallConsentPolicy`
  with admit-all (`AdmitAllInstallConsent`) + deny-all
  (`DenyAllInstallConsent`) default helpers. The dep-direction
  preserves `benten-caps → benten-platform-foundation` (the existing
  arrow); engine glue blanket-adapter for
  `CapabilityPolicy::check_install_consent` is a Phase-4-Meta-
  Composing addition.

- **`manifest_store::install_plugin` rename + deprecation** — renamed
  to `install_verified_record_unchecked` with `#[deprecated]` +
  `#[doc(hidden)]`. The 4 internal callers (drift-detection tests +
  redb roundtrip tests) are annotated `#[allow(deprecated)]` per
  intentional side-door use; production callers MUST route through
  `plugin_lifecycle::install_plugin`.

### New ErrorCode mints

- **`PluginInstallConsentDenied`** (E_PLUGIN_INSTALL_CONSENT_DENIED) —
  CLAUDE.md baked-in #18 §8-E hook #1 install-time consent denial.
  CATALOG_VARIANT_COUNT 192 → 193.
- **`PluginPerDelegationDenied`** (E_PLUGIN_PER_DELEGATION_DENIED) —
  CLAUDE.md baked-in #18 §8-E hook #2 per-delegation runtime denial.
  CATALOG_VARIANT_COUNT 193 → 194.

### New `pub` types

- **`benten_engine::write_boundary_chain_validator::WriteAdmissionFrame`**
  — sealed frame with `engine_internal()` + `with_chain(cid, did)`
  builders (private fields preserve §1.A.FROZEN item 8 strict-
  additivity).
- **`benten_engine::production_engine_builder::ProductionEngineBuilder`**
  — canonical production constructor (per CRITIC-2 F-2.2 rename; NOT
  shadowing `EngineBuilder`).
- **`benten_engine::production_manifest_envelope_rechecker::ProductionManifestEnvelopeRechecker`**
  — substantive rechecker substrate impl (Row D-4 + Row D-18 closure).
- **`benten_engine::manifest_envelope_recheck::is_synthesized_node_id`**
  — `pub fn` helper (per Δv3-10).
- **`benten_sync::handshake::sync_hydrate_consume_recheck_outcome`**
  — `pub fn` Row D-6 consumption surface.

### Migration for downstream consumers

- `InstallPorts {...}`: add `policy: &impl InstallConsentPolicy,` field
  (e.g. `policy: &benten_platform_foundation::install_consent::AdmitAllInstallConsent,`
  for non-production paths).
- `InstallPorts {...}`: replace `install_record_replay_check: None,`
  with `install_record_replay_check: &mut benten_platform_foundation::testing::noop_replay_check(),`
  (test fixtures) or a real `engine.install_record_replay_store().record_and_check`
  closure (production).
- Engine construction: production callers SHOULD migrate
  `EngineBuilder::new().open(path)` to
  `ProductionEngineBuilder::new().open(path)` to install the
  substantive rechecker automatically.

---

## Cohort 7 — R6 R2 FP-A audience-pubkey BLOCKER + DropBundle truncation (this PR)

This cohort lands at PR<R6-R2-FP-integration> (R6 R2 FP Strategy-C
consolidation re-do, off post-PR-#1351 main, 2026-05-25). Closes the
L2-R2-BLOCKER-1 audience_pubkey post-sign substitution access-theft
hazard + L4-MAJ DropBundle inter-Recipe truncation sibling. Sharpens
the previously-closed Row D-1 with the apply_atrium_merge per-row
chain-bearing site (now 14 admit_write_chain sites; 2 chain-bearing).

### Wire-format breaking change

- **`BINDING_SIG_DOMAIN` v2 → v3** at
  `crates/benten-caps/src/authorization_grant.rs::BINDING_SIG_DOMAIN`.
  Pre-FP-A the `AuthorizationGrant.audience_pubkey` field lived
  OUTSIDE the signed `binding_message` — a network adversary could
  mutate `audience_pubkey` to her own bytes, connect to a producer with
  her own iroh `EndpointId`, and the UcanBlobsHandler ARM 2
  (audience-binding) check (`conn_pubkey == eve_pubkey ==
  grant.audience_pubkey`) would MATCH while ARM 5 (binding-sig verify)
  still verified because the bound `audience_binding` = CID(bob_pubkey)
  was untouched. Silent access-theft (not just attribution-forgery).
  Post-FP-A the `audience_pubkey` is folded in as the 6th binding-
  message segment (`len_u32_le || pubkey_bytes`) AND the domain-tag
  bumps v2 → v3. Domain-separation at segment 1 means v2-signed grants
  fail re-verification under v3 (intended pre-tag behavior; no
  v2-signed grants exist on the wire yet — test fixtures only).
- **`DropBundle` per-recipe AAD extension** at
  `crates/benten-drop/src/bundle.rs::DropBundle` consumption. The
  per-`Recipe` AEAD seal now binds `aad_per_recipe` (position +
  list-length) in addition to `aad_whole_content(plaintext_cid)`. Pre-
  FP-A a malicious relay could DROP a `Recipe` from
  `DropBundle.content: Vec<EncryptedContent>` and the recipient's
  `consume_offline` iteration would still verify each remaining
  per-Recipe AEAD tag individually (the AAD committed to neither
  position nor list length) — silent delivery of a truncated bundle.
  Post-FP-A the per-Recipe AAD binds both, so any drop fails AEAD
  verification at the affected position. **No wire-format version bump
  for `DropBundle`** because the existing `DropBundleVersion` enum is
  the wire-version surface and the AAD-binding extension preserves the
  on-wire byte layout (the AAD is computed at verify-time, not
  serialized into the bundle).

### Substrate honesty (Row D-1 sharpening; no public-API shape change)

- **`apply_atrium_merge` per-row admit now CHAIN-BEARING** at
  `crates/benten-engine/src/engine.rs::apply_atrium_merge`. Pre-FP-B
  13 of 13 `admit_write_chain` sites passed
  `WriteAdmissionFrame::engine_internal()`; only `delegate_capability`
  was chain-bearing. Inbound-sync per-row writes routed through
  `append_version` (engine_internal frame), so the
  `WriteBoundaryChainValidator` never observed the peer-DID at row
  admission. Post-FP-B the `apply_atrium_merge` per-row loop presents
  `WriteAdmissionFrame::with_chain(peer_actor_cid, peer_did)`, closing
  the asymmetry (outbound writes chain-walked, inbound sync rows now
  also chain-walked). 14 admit_write_chain sites total; 2 chain-bearing
  (`delegate_capability` + `apply_atrium_merge` per-row), 12
  engine_internal. Behavior change: in deployments with a substantive
  `WriteBoundaryChainValidator` installed, inbound-sync rows with a
  peer-DID outside the user-as-root chain are now REJECTED at admission
  (previously admitted silently as `engine_internal`).

### Documentation / retract

- **`V1-FROZEN-INTERFACE-DEFERRED.md` Row D-15c retracted.** The
  earlier D-15c framing (`audience_pubkey` as "deferred wire-format
  add") was based on a misread of the v2 binding-message: D-15c
  asserted that adding `audience_pubkey` to the binding-message was a
  deferrable nicety, but the L2-R2-BLOCKER-1 cross-confirmation showed
  it was a load-bearing access-theft defense. Row D-15c is now marked
  `~~RETRACTED~~` with a pointer to this Cohort 7 entry.
- **`V1-WIRE-FORMAT-INVENTORY.md` Item 6** updated to `BINDING_SIG_DOMAIN v3`
  with the audience-pubkey binding context.
- **`SECURITY-POSTURE.md`** updated with the L18-related callouts (no
  open Compromise mints; closes via existing `BindingMismatch` +
  `BindingSigInvalid` variants).

### New ErrorCode mints

None — this cohort closes through existing `BindingMismatch` +
`BindingSigInvalid` variants. CATALOG_VARIANT_COUNT unchanged at 194.

### Migration for downstream consumers

- **Downstream re-issue grants signed under v2 must be re-signed under
  v3.** The change is shape-preserving at the byte-layout level (one
  new segment appended to the signed-payload preimage); only the
  signature changes. Test fixtures using
  `issue_with_nbf_for_test` are unaffected (the test helper signs over
  the 6-segment payload directly).
- **No code-change required** for callers that consume
  `AuthorizationGrant` through the
  `AuthorizationGrant::verify_binding` API — the helper internally
  reconstructs the 6-segment payload under the v3 domain tag.
- **`DropBundle` producers** must use the `aad_per_recipe`-aware seal
  path (which is the only path on the post-FP-A surface; the prior
  whole-content-AAD-only path was internal-only).
- **`apply_atrium_merge` callers** with a substantive
  `WriteBoundaryChainValidator` installed should audit their inbound-
  sync admission behavior — peer-DIDs outside the user-as-root chain
  are now REJECTED at admission instead of silently admitted.

### Tests added (per pim-2 §3.6b sub-rule 4 substantive)

- `crates/benten-caps/tests/tf3b_audience_substitution_post_sign_rejected.rs` (3 sub-tests; bare verify_binding API)
- `crates/benten-sync/tests/tf3e_audience_substitution_arm2_arm5_integration.rs` (2 sub-tests; full handler dispatch)
- `crates/benten-drop/tests/tf3d_inter_recipe_truncation_rejected.rs` (truncation defense)

`would-FAIL-on-revert` verified per pim-18 §3.6f: surgical revert of
`audience_pubkey` arg → `None` at both call sites caused 3/3 tf3b
tests to FAIL with `Got: Ok(())` (silent admission). Post-fix: 3/3 +
2/2 PASS.

---

## Cohort 8 — Row D-19 G-COMP-1 wave WIRE-NOW (this PR, Batch C)

This cohort lands at PR<R6-R2-FP-integration-redo> Group C
(R6-R2-batch-c-dsl-catalog sub-branch; 2026-05-25). Closes
[`V1-FROZEN-INTERFACE-DEFERRED.md`](V1-FROZEN-INTERFACE-DEFERRED.md)
**Row D-19** WIRE-NOW per the v1-beta-freeze-window auto-WIRE-NOW
discipline: the obsolete `Strategy::C` naming + the 3 ungranted DSL
ErrorCodes would otherwise ride into v1-beta wire bytes + freeze as
SemVer-breaking-post-tag-fix surfaces. Atomic 4-surface §3.5g rename
+ 3 first-class ErrorCode mints land in a single commit.

### Wire-format breaking change

- **`E_VIEW_STRATEGY_C_RESERVED` → `E_VIEW_STRATEGY_RESERVED`** at
  `crates/benten-errors/src/lib.rs::ErrorCode::ViewStrategyReserved`
  (variant `ViewStrategyCReserved` → `ViewStrategyReserved`; `as_str`
  arm + `from_str` arm both swap to the new wire string atomically).
  This is a **wire-format byte-shape breaking change** for any
  consumer pattern-matching on the on-disk / on-the-wire string
  `"E_VIEW_STRATEGY_C_RESERVED"`. The variant is a registration-time
  refusal code (`Engine::register_user_view`), so the wire surface
  is narrow: error-handling code in the napi binding + TS DSL
  validator + downstream test code that asserted on the old string.

### Public-API shape changes

- **Rust enum variant rename:** `benten_errors::ErrorCode::ViewStrategyCReserved`
  → `benten_errors::ErrorCode::ViewStrategyReserved`.
- **Rust EngineError variant rename:** `benten_engine::error::EngineError::ViewStrategyCReserved`
  → `benten_engine::error::EngineError::ViewStrategyReserved` (+ the
  `pub use` re-export `benten_engine::EngineError::ViewStrategyReserved`).
- **TS class rename:** `EViewStrategyCReserved` → `EViewStrategyReserved`
  in `packages/engine/src/errors.generated.ts` (auto-regen from
  `docs/ERROR-CATALOG.md`; cargo-public-api + ts-public-api baselines
  updated in same commit).
- **TS Strategy union widened:** `type Strategy = "A" | "B" | "C"`
  → `type Strategy = "A" | "B" | "C" | "Reserved"`. The new
  `"Reserved"` value is the canonical name; `"C"` is retained as a
  backward-compat alias (the napi parser accepts both → maps to
  `Strategy::Reserved` Rust-side).
- **`EngineError.diagnostic()` JSON `kind` rename:** `"viewStrategyCReserved"`
  → `"viewStrategyReserved"` in `error.rs::diagnostic()`. Consumers
  pattern-matching on the JSON `kind` field must update.

### New first-class ErrorCode mints (CATALOG_VARIANT_COUNT 194 → 197)

Three pre-existing DSL `pub const benten_dsl_compiler::E_DSL_*`
wire-string constants gain first-class typed catalog mirrors per
§3.5g pub-error-variant-first-class-mirror discipline. Pre-mint the
napi `mapNativeError` boundary collapsed all three to
`ErrorCode::Unknown(string)`; first-class mirrors let TS consumers
match by typed BentenError subclass (`EDslParseError` /
`EDslUnknownPrimitive` / `EDslMissingRespond`).

- **`E_DSL_PARSE_ERROR` → `ErrorCode::DslParseError`** (TS class
  `EDslParseError`). Mirror of `CompileError::Parse(Diagnostic)` at
  `crates/benten-dsl-compiler/src/lib.rs`. Wire string unchanged.
- **`E_DSL_UNKNOWN_PRIMITIVE` → `ErrorCode::DslUnknownPrimitive`** (TS
  class `EDslUnknownPrimitive`). Mirror of `CompileError::Semantic(_)`.
  Wire string unchanged.
- **`E_DSL_MISSING_RESPOND` → `ErrorCode::DslMissingRespond`** (TS
  class `EDslMissingRespond`). Mirror of the `CompileError::Build(_)`
  sub-case where the inner `Diagnostic.error_code == E_DSL_MISSING_RESPOND`
  (the other Build sub-case — `E_DSL_INVALID_SHAPE` — continues to
  route through the pre-existing `ErrorCode::DslInvalidShape`).
  Wire string unchanged.

### Drift-detect baseline removals

`scripts/drift-detect-error-variant-mirror-baseline.txt` removes
the 3 grandfathered lines `CompileError::{Parse,Semantic,Build}`
per §3.5g item 6 amendment closure — the variants now have
first-class typed mirrors, so the drift-detect scanner enforces
the rule going forward.

### Why break-OK at v1-beta

The v1-beta tag locks public-API + wire-format + ErrorCode catalog
surfaces. Deferring this rename / mint until post-v1-beta would
require a SemVer-breaking fix-pass. The atomic 4-surface §3.5g
rename + the 3 catalog mints land NOW in the freeze window per the
"obsolete names + ungranted DSL ErrorCodes ride into v1-beta wire
bytes" forensic argument (Row D-19 anchor).

### Migration path

- **Rust consumers:** rename `ErrorCode::ViewStrategyCReserved` →
  `ErrorCode::ViewStrategyReserved`; `EngineError::ViewStrategyCReserved
  { view_id }` → `EngineError::ViewStrategyReserved { view_id }`. The
  field shape (single `view_id: String`) is unchanged.
- **TS consumers:** `import { EViewStrategyCReserved }` →
  `import { EViewStrategyReserved }`. Code asserting on the wire
  string `"E_VIEW_STRATEGY_C_RESERVED"` → `"E_VIEW_STRATEGY_RESERVED"`.
  Code asserting on the JSON `kind` field `"viewStrategyCReserved"`
  → `"viewStrategyReserved"`.
- **DSL ErrorCode consumers:** existing string-keyed switches on
  `E_DSL_PARSE_ERROR` / `E_DSL_UNKNOWN_PRIMITIVE` / `E_DSL_MISSING_RESPOND`
  continue to work; the wire strings are unchanged. TS consumers
  upgrading to typed-class dispatch can now `instanceof EDslParseError`
  etc. against the typed `BentenError` subclasses surfaced by the
  napi `mapNativeError` boundary (previously these collapsed to the
  base `BentenError("E_UNKNOWN")` class).

---

## How to consume this ledger

1. **Adopting v1-beta:** read Cohort 1 + 2 first (wire-format + public-API shape changes you must adapt to).
2. **Updating an internal Benten codebase from `8141b94..HEAD`:** read Cohort 3 (the G-CORE-9 R1 fix-pass additions are the freshest layer).
3. **Planning post-v1-beta work:** read Cohort 4 + V1-FROZEN-INTERFACE-DEFERRED.md.
4. **Verifying nothing else changed at the public-API:** consult `docs/public-api/<crate>.txt` baselines + the now-required-failing cargo-public-api workflow.

---

## Provenance

Authored at G-CORE-9 R1 fix-pass (PR #1346; 2026-05-24) per L18-r1-2/3 closure. Subsequent R2-Rn rounds verify completeness vs the `git log 8141b94..HEAD` walk.

Cohort 6 landed at PR #1351 (R6 R1 FP Strategy-C consolidation; 2026-05-25).

Cohort 7 landed at PR<R6-R2-FP-integration-redo> (R6 R2 FP Strategy-C consolidation re-do off post-#1351 main; 2026-05-25) — closes the L2-R2-BLOCKER-1 audience_pubkey BLOCKER + L4-MAJ DropBundle inter-Recipe AAD sibling + Row D-1 sharpening (apply_atrium_merge per-row chain-bearing admit).

Cohort 8 PLANNED entry RESOLVED — superseded by FINAL Cohort 8 entries from batch-a + batch-b + batch-c consolidating into ONE canonical Cohort 8 at the end of the Strategy-C cascade. Per-group fragmentary "Cohort 8 landed at..." Provenance lines are temporary; final canonical Cohort 8 unification happens after batch-a + batch-b merge.

Cohort 8 Group C (batch-c) — `phase-4-meta-core/r6-r2-batch-c-dsl-catalog` sub-branch at `13faa4b1` (2026-05-25); closes Row D-19 atomic 4-surface §3.5g rename `Strategy::C` → `Strategy::Reserved` + 3 first-class DSL ErrorCode mints (`E_DSL_PARSE_ERROR` / `E_DSL_UNKNOWN_PRIMITIVE` / `E_DSL_MISSING_RESPOND`); CATALOG_VARIANT_COUNT 194 → 197.
