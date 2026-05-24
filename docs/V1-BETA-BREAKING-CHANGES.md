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

### PR #1338 — `ManifestEnvelopeRecheckOutcome` `NotApplicable` → `UnresolvedDeny` rename (per §4.36 fail-CLOSED flip)
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
- **STATUS in this PR:** anticipated mention; the Strategy rename + DSL 3 ErrorCode mints (Bundle 4) ESCALATED per the Bundle-3 substantial cascade demonstrating the per-bundle LOC explosion; relocated to a follow-up sub-pass for orchestrator pacing. BELONGS-NAMED-NOW: G-CORE-9 R2 OR G-COMP-1 §<row>. Captured in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-15 watch-list.

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
- **Status:** **DEFERRED — not landed at v1-beta.** An initial always-on filter at engine.rs:1462 was attempted (commit `34053ed4`) then reverted (commit `3d6f4d66 — "defer L2-MAJ-1 synthesized-peer-DID hardening to G-COMP-1 Row D-18"`) because the always-on form over-fires for the default-Noop test fixtures which intentionally do not register peer-DIDs. The R2 lens (`r2-l2-adversarial-threat-model`) raised this stale claim as L2-R2-BLK-1; R2 fix-pass retenses this row to honesty.
- **What v1-beta ships:** engine.rs:1463-1477 carries an inline comment naming the deferral rationale (the proper closure couples synthesized-fallback rejection to substantive-rechecker-installed detection; gap closes at Row D-4 closure when ProductionManifestEnvelopeRechecker becomes responsible for its own per-DID resolution).
- **Why break-OK:** doc-honesty retense of an attempted-then-reverted change; no public-API impact.
- **Migration path:** none — at v1-beta the always-mounted Noop rechecker admits everything per Compromise #26 disclosure; under a substantive ProductionRechecker (G-COMP-1 deliverable per Row D-4), the rechecker is responsible for synthesized-DID rejection. See `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-18.

### G-CORE-9 R1 Bundle 11 — L2-MIN-2 empty-DID-string structural defense at chain-validator
- **What changed:** `validate_chain_with_manifest_envelope` rejects empty-DID-string root BEFORE consulting `user_registry`.
- **Why break-OK:** defense-in-depth; no public-API change.
- **Migration path:** none.

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

## How to consume this ledger

1. **Adopting v1-beta:** read Cohort 1 + 2 first (wire-format + public-API shape changes you must adapt to).
2. **Updating an internal Benten codebase from `8141b94..HEAD`:** read Cohort 3 (the G-CORE-9 R1 fix-pass additions are the freshest layer).
3. **Planning post-v1-beta work:** read Cohort 4 + V1-FROZEN-INTERFACE-DEFERRED.md.
4. **Verifying nothing else changed at the public-API:** consult `docs/public-api/<crate>.txt` baselines + the now-required-failing cargo-public-api workflow.

---

## Provenance

Authored at G-CORE-9 R1 fix-pass (this PR; 2026-05-24) per L18-r1-2/3 closure. Subsequent R2-Rn rounds verify completeness vs the `git log 8141b94..HEAD` walk.
