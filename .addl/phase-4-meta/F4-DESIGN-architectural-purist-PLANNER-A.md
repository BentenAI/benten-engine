# F4 DESIGN — Architectural-Purist — PLANNER-A

> **Persistence note:** Planner-A was dispatched as a read-only `Plan` subagent and could not write files / commit / push. Design content delivered inline; orchestrator persisted to this path 2026-05-24 ~18:30Z. Base SHA explored: `a0b75637` (main HEAD).

## Section 1 — Enumerate the 6 surfaces

Cluster maps DEFERRED rows D-1..D-4 onto **6 consumer wire-up sites**; D-3 = 3 sub-surfaces (one per §8-E hook) per L6 lens-rubric Q8.

### S1 — WriteBoundaryChainValidator at WRITE admission (Layer-1)
**Frozen substrate:** trait + `WriteBoundaryChainOutcome { NotApplicable, Admitted, ChainNotUserRooted{chain_root_did} }` at `crates/benten-engine/src/write_boundary_chain_validator.rs`; setter at `engine.rs:2010`; Noop default at `:1935-1937`; typed reject `ErrorCode::WriteBoundaryChainNotUserRooted`.
**Consumers defaulting to admit:** ~13 WRITE entry points enumerated: engine_crud.rs:{68,157,169,175,189}, engine_caps.rs:683, engine_views.rs:976, engine_modules.rs:{259,325}, engine_diagnostics.rs:541, engine_wait.rs:{888,1070}, engine.rs:3972 (apply_atrium_merge per-row), handler_versions.rs:202.
**Would-be-wired:** at non-privileged WRITE, engine calls `validator.validate_chain(chain_anchor_cid, actor_did)` + `outcome_to_admission_reject(outcome)?`. Plugin-DID-rooted chain rejects with `WriteBoundaryChainNotUserRooted` BEFORE row lands.
**DEFERRED row:** D-1.

### S2 — InstallRecordReplayStore at install_plugin (Layer-2)
**Frozen substrate:** `InstallRecordReplayStore::record_and_check([u8;32])`; typed `PluginInstallRecordAlreadyApplied`; port `InstallPorts.install_record_replay_check: Option<&mut InstallRecordReplayCheckFn>` at `plugin_lifecycle.rs:705-722`; consumption site already shaped behind Option at `:901-905`.
**Consumers:** zero production callers pass `Some(closure)`; TOCTOU defense NOT live.
**Would-be-wired:** drop Option, every caller passes substantive closure backed by `engine.install_record_replay_store().record_and_check(payload_hash)`. Side-door at `manifest_store::install_plugin:98-111` renamed to `install_verified_record_unchecked` per CLAUDE.md #5 (no shims).
**DEFERRED row:** D-2.

### S3 — check_install_consent at install pipeline
**Frozen substrate:** `crates/benten-caps/src/policy.rs:508-514`, defaulted `Ok(())`, sealed.
**Consumers:** zero production sites; override silently ignored.
**Would-be-wired:** new Step 3c in plugin_lifecycle.rs (immediately after replay-check at :905, before Step 4): `policy.check_install_consent(&payload_hash, plugin_did_str)`. CapError → typed `PluginInstallConsentDenied` (mint NEW; see FORK A).
**DEFERRED row:** D-3-a.

### S4 — check_per_delegation at delegate_capability
**Frozen substrate:** `policy.rs:532-539`, defaulted `Ok(())`.
**Consumers:** zero production sites at `engine_caps.rs:429-558`.
**Would-be-wired:** new check between Step 2b (manifest-shares) + Step 3 (effective-scope) at `:559`. CapError → `EngineError::Cap(_)`. REUSE existing `PluginDelegationOutsideManifestEnvelope`.
**DEFERRED row:** D-3-b.

### S5 — check_write_with_audience at apply_atrium_merge + workspace sweep
**Frozen substrate:** `policy.rs:571-573`, defaults to `self.check_write(ctx)` (back-compat preserve). `CapWriteContext.audience_did: Option<...>` already frozen per item 8.
**Consumers:** `engine.rs:1413` (always `check_write`, never `_with_audience`).
**Would-be-wired:** at `:1413` switch unconditionally to `check_write_with_audience`. Architectural-purist sweep: convert ALL workspace `policy.check_write(...)` calls to `_with_audience` (engine_diagnostics.rs:78, etc.) — `check_write` becomes "internal default delegate target"; audience-bearing form is canonical seam.
**DEFERRED row:** D-3-c.

### S6 — ProductionManifestEnvelopeRechecker + EngineBuilder + D-6 + D-18
**Frozen substrate:** rechecker trait + 4-arm outcome at `manifest_envelope_recheck.rs:85-171`; Noop returns `NotApplicable`; engine slot wired at `engine.rs:1448`; per-row call at `:1493`; default-builder installs Noop at `:1923`.
**Consumers:** every production Engine WITH PluginLibrary defaults to Noop; per-DID substantive recheck NOT live.
**Would-be-wired:** ship `ProductionManifestEnvelopeRechecker` in `benten-platform-foundation` composing `PluginLibrary::manifest_envelope_for(peer_did)` × `UserDidRegistry::contains(peer_did)` × `validate_chain_with_manifest_envelope`. Default `Engine::default()` keeps Noop (test/embedded); `EngineBuilder::build()` is canonical production constructor that wires substantive impl. Includes D-6 sibling at `crates/benten-sync/src/handshake.rs` (§4.25 sync-hydrate consumption) + D-18 (synthesized-`node-id:` rejection — safe to land now that substantive rechecker is installed).
**DEFERRED rows:** D-4 + D-6 + D-18.

## Section 2 — Architectural-purist end-state design

**Unifying principle:** every Layer-N CLAUDE.md #18 trust gate gets ONE canonical port owned by trust-boundary owner (`benten-engine`), with substantive impl owned by the layer that owns relevant state (`benten-platform-foundation` for PluginLibrary/UserDidRegistry), wired via structural-always-on (not Option) seams, with EngineBuilder pattern enforcing substantive impl in production constructors + Noop reserved for explicit `Engine::for_test_no_trust_model()` callers. **Zero `Option<Arc<dyn Trait>>` in production paths; zero "wrapped behind None defaults" silent-admit posture; one structural compile-time pin per layer enforcing production constructors don't return until substantive impl is wired.**

### S1 — Implementation shape
New `pub(crate) fn admit_write_chain(chain_anchor_cid, actor_did) -> Result<(), EngineError>` helper inside `Engine`. New sealed `WriteAdmissionFrame { chain_anchor_cid: Option<Cid>, actor_did: Option<String> }` (avoids `CapWriteContext` `#[non_exhaustive]` cascade per D-17). Highest-leverage call site = `engine_caps.rs::delegate_capability` Step 4 (delegation chains = exact CLAUDE.md #18 Layer-1 attack surface). Plus `apply_atrium_merge` per-row + privileged sites pass None/None (return Ok).
**Production impl:** `benten_platform_foundation::trust::ProductionWriteBoundaryChainValidator` using `UserDidRegistry` view + `benten_caps::chain_authority::validate_chain_with_manifest_envelope`.
**Tests:** production-arm + adversarial (T-class) + capability-rejection ordering + privileged-bypass.

### S2 — Drop Option
```rust
pub struct InstallPorts<'a, M, P> {
    pub cap_minter: &'a mut M,
    pub private_ns: &'a mut P,
    pub install_record_replay_check: &'a mut InstallRecordReplayCheckFn,  // was Option<_>
}
```
Test fixtures get `noop_replay_check()` helper from `benten_platform_foundation::testing`. Side-door: `manifest_store::install_plugin` renamed `install_verified_record_unchecked` + SAFETY doc.
**Tests:** end-to-end replay-rejected + 16-thread parallel-replay + backward-compat fresh-install + side-door trybuild doc-warning.

### S3 — check_install_consent
New Step 3c in plugin_lifecycle.rs install pipeline. Requires new `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` accessor.
**FORK A** — mint `PluginInstallConsentDenied` vs reuse `PluginInstallConsentRequired`. Recommendation: **mint** (preserves R6-FP-A typed-discrimination invariant).
**Tests:** production-arm admit + custom curated-trust-list policy denial + before-cap-cascade ordering.

### S4 — check_per_delegation
Single insertion at `engine_caps.rs:559`. REUSE `PluginDelegationOutsideManifestEnvelope`. CapError routes through `EngineError::Cap`.
**Tests:** production-arm + rate-limiting custom policy + after-private-namespace-clause ordering.

### S5 — Workspace sweep `check_write` → `check_write_with_audience`
Single-line change at `engine.rs:1413` + workspace sweep all `policy.check_write(` sites. Default impl delegates to `check_write` so back-compat exact. **FORK B** — audience_did threading model (Path-α: None at engine_crud, real at delegate_capability + apply_atrium_merge; Path-β: synthesize user-DID-root anchor; Path-γ: thread Option<&Cid> through engine_crud public API — HALT-AND-SURFACE per item 8 freeze). Recommendation: **Path-α** (matches substrate's NotApplicable design + preserves v1-frozen signatures + relies on existing `CapabilityPolicy::check_write` for engine_crud Layer-1 per Compromise #2).
**Tests:** default-delegation + audience-aware-override denial + apply_atrium_merge consults `_with_audience` (panic on direct `check_write` call).

### S6 — ProductionRechecker + EngineBuilder + siblings
New `production_manifest_envelope_rechecker.rs` in foundation + sealed `UserDidRegistry` trait. **FORK C** — EngineBuilder location (Path-(i) engine ships generic builder + foundation extends; Path-(ii) foundation owns production builder, raw `Engine::default()` = test/embedded; Path-(iii) both). Recommendation: **Path-(ii)** (single canonical production constructor in the layer owning production state). D-6 (handshake.rs §4.25) + D-18 (synthesized-node-id rejection) ship together.
**Tests:** production-arm in-envelope + adversarial outside-envelope (T8) + unresolvable-node-id (D-18) + missing-manifest + default-builder substantive verify + handshake.rs D-6 wire.

## Section 3 — Cross-cutting concerns

### Sibling sweeps (per feedback_d17_same_file_sweep_recurrence)
| Surface | Sibling-sweep targets |
|---|---|
| S1 | ~13 WRITE entry points; D-18 (substrate defense-in-depth); D-8 (F3 anti-replay TOCTOU at FrameReplayMarker) |
| S2 | manifest_store side-door rename; verify no other install entry points |
| S3 | only procedural Step 4 (verify_user_signature); the hook IS the sibling |
| S4 | only one delegate site exists |
| S5 | all `policy.check_write(` → `_with_audience` |
| S6 | D-6 (handshake.rs); D-18 (synthesized-node-id) |

**Net:** wirings naturally close **D-1 + D-2 + D-3a + D-3b + D-3c + D-4 + D-6 + D-18** (8 DEFERRED rows for 6 surfaces). D-8 architecturally orthogonal — defer.

### ErrorCode mirror obligations (§3.5g 8-surface)
| Variant | Mirror at HEAD | F4 action |
|---|---|---|
| `WriteBoundaryChainNotUserRooted` | first-class | Verify TS mirror; MINT if absent |
| `PluginInstallRecordAlreadyApplied` | first-class | Verify TS mirror |
| `PluginInstallConsentDenied` (NEW) | needs mint | Full 8-surface mirror (Rust + as_static_str + from_str + routed_edge_label + TS class + CODE_TO_CTOR + CATALOG.md + CATALOG_VARIANT_COUNT 192→193) |
| Per-delegation denial | via CapError (mirrored) | No new mint |
| `check_write_with_audience` denial | via CapError | No new mint |
| `ManifestEnvelopeRecheckUnresolvedDeny` + `PluginDelegationOutsideManifestEnvelope` | mirrored | Verify TS class names |

**Net: 0-1 new mints; CATALOG 192→192 or 193.**

### §3.6j sweep-completeness self-verify
Each surface's test pin set MUST include: production-arm + adversarial (would-FAIL-if-no-op'd) + capability-rejection (typed code reaches consumer) + sweep-completeness (workspace walker enumerates all WRITE/install/delegate sites + asserts each gated).

### Wire-format
**ZERO wire changes.** All wirings runtime-only. No Compromise #26 narrative update needed for wirings themselves (closure annotation only).

### napi
S2 napi wrapper wires substantive replay-check closure via `engine.install_record_replay_store()`. S6 napi builder extension. **No new TS surface; behavior change only.**

## Section 4 — Scope estimate

| Surface | Rust LOC | Test LOC | Files |
|---|---|---|---|
| S1 | ~350 (13 sites + helper + threading + production impl) | ~250 | 2 (production impl + audit walker) |
| S2 | ~80 (drop Option + helper + rename + fixture sweep) | ~100 | 0 |
| S3 | ~40 | ~120 | 0 |
| S4 | ~25 | ~120 | 0 |
| S5 | ~10 | ~100 | 0 |
| S6 + D-6 + D-18 | ~360 | ~300 | 2 (production rechecker + UserDidRegistry) |

**Totals:** ~865 production LOC + ~990 test LOC ≈ **~1855 total**. **4 new files.** **0-1 new ErrorCode mints.** **5 docs to touch** (DEFERRED.md + V1-FROZEN-INTERFACE.md + CLAUDE.md #18 + SECURITY-POSTURE.md #26 + INTERNALS.md).

## Section 5 — Risk register + FORKS

### Risk register
1. **WRITE-site audit coverage** — HIGH; mitigation: first impl step is exhaustive enumeration.
2. **Chain-anchor at engine_crud surfaces** — see FORK B; LOW with Path-α.
3. **Engine::capability_policy() accessor** — LOW; additive.
4. **InstallPorts Option drop** — LOW; mechanical fixture sweep (~12 sites).
5. **manifest_store rename** — LOW; pure rename.
6. **D-6 + D-18 piggybacking** — LOW; architecturally inseparable; one PR.
7. **UserDidRegistry sealing** — LOW; well-established pattern.

### FORKS FOR ORCHESTRATOR/BEN
- **FORK A** — S3 ErrorCode shape (reuse `PluginInstallConsentRequired` vs mint `PluginInstallConsentDenied`). Rec: **mint**.
- **FORK B** — S1 chain-anchor threading model (Path-α/β/γ). Rec: **Path-α**.
- **FORK C** — EngineBuilder location (i/ii/iii). Rec: **Path-(ii)** (foundation owns production builder).
- **FORK D** — Bundle D-22/D-23/D-24 with F4? Rec: include D-24 (coupled to S4); D-22 + D-23 separate.

## Section 6 — Implementation phasing

### 3-agent split
| Agent | Surfaces | LOC | Sequencing |
|---|---|---|---|
| 1 | S1 + S4 + S5 | ~700 | First |
| 2 | S2 + S3 | ~250 | After Agent 1 (S3 needs capability_policy accessor) |
| 3 | S6 + D-6 + D-18 | ~600 | After Agent 1 (EngineBuilder wires S1 validator too) |

### cargo-public-api regen cadence: per-PR
### Pre-push: drift:errors + drift:public-api + workspace test + clippy + trybuild + §3.5l mandatory pre-combined-push + §3.6j sweep-completeness self-verify

## Critical Files for Implementation
- `crates/benten-engine/src/engine.rs` — apply_atrium_merge wire + builder + setters (~4832 LOC; mods at :1413, :1448-1499, :1923-1949)
- `crates/benten-engine/src/engine_caps.rs` — delegate_capability (S1 highest-leverage + S4 hook; ~:429-558)
- `crates/benten-platform-foundation/src/plugin_lifecycle.rs` — S2 + S3 install pipeline (drop Option, add Step 3c; ~:696-722 and :885-905)
- `crates/benten-caps/src/policy.rs` — no edits; frozen substrate reference
- `crates/benten-platform-foundation/src/lib.rs` + new `production_manifest_envelope_rechecker.rs` — S6 production impl + UserDidRegistry trait + EngineBuilder extension
