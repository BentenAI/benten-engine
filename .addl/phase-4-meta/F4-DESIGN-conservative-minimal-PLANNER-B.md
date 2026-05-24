# F4 DESIGN — Conservative-Minimal — PLANNER-B

> **Persistence note:** Planner-B was dispatched as a read-only `Plan` subagent and could not write files / commit / push. Design content delivered inline; orchestrator persisted to this path 2026-05-24 ~18:00Z. The "branch + HEAD SHA" return-contract fields are N/A as a result.

## Section 1 — Enumeration of the 6 surfaces

Sourced from `docs/V1-FROZEN-INTERFACE-DEFERRED.md` rows D-1 through D-4 + the two §8-E sub-rows the brief lifts inside D-3.

- **S1 = D-1 — `WriteBoundaryChainValidator` consumption.** Substrate at `crates/benten-engine/src/write_boundary_chain_validator.rs` (trait + `NoopWriteBoundaryChainValidator` default returning `NotApplicable`). Engine slot at `engine.rs:1027` + setter at `engine.rs:2010`. Consumption named for "~10–15 WRITE entry points" inside `Engine::commit` / `Engine::put_node_with_context` is NOT wired. Layer-1 user-as-root invariant per CLAUDE.md #18 is NOT structurally enforced at v1-beta binary.

- **S2 = D-2 — `InstallRecordReplayStore` lifecycle-required wiring.** Substrate at `crates/benten-engine/src/install_record_replay.rs` (atomic Mutex-guarded `record_and_check` + 4 unit tests including `parallel_presentation_serialized_one_admits_one_rejects`). Consumer port at `crates/benten-platform-foundation/src/plugin_lifecycle.rs:713,901-905` — `install_record_replay_check: Option<&'a mut InstallRecordReplayCheckFn>` with `if let Some(replay_check) = ports.install_record_replay_check.as_mut()`. ZERO production callers supply `Some(closure)` at HEAD. TOCTOU defense NOT live in shipped binaries.

- **S3a = D-3 hook A — `check_install_consent` consumption.** Defaulted trait method at `crates/benten-caps/src/policy.rs:508`. Named call site = install-pipeline admission (plugin_lifecycle.rs step 3b alongside replay-check). ZERO production callers. Override is silently ignored.

- **S3b = D-3 hook B — `check_per_delegation` consumption.** Defaulted trait method at `crates/benten-caps/src/policy.rs:532`. Named call site = `engine_caps.rs::delegate_capability` (around line 541 alongside `shares_policy_resolver` call). ZERO production callers.

- **S3c = D-3 hook C — `check_write_with_audience` consumption.** Defaulted trait method at `crates/benten-caps/src/policy.rs:571` (default delegates to `check_write` ignoring audience). Named call site = `engine.rs:1413` per-row recheck inside `apply_atrium_merge` (currently calls `policy.check_write(&ctx)`). ZERO production overrides observed; switching the call site unconditionally is back-compat-safe because of the trait default's delegation.

- **S4 = D-4 — `ProductionManifestEnvelopeRechecker` production impl + default-builder wiring.** Substrate at `crates/benten-engine/src/manifest_envelope_recheck.rs` (`ManifestEnvelopeRechecker` trait + `NoopManifestEnvelopeRechecker` default returning `NotApplicable` always). Engine slot wired at `engine.rs:1448` via `set_manifest_envelope_rechecker`. The substantive impl that consults `PluginLibrary` + `UserDidRegistry` + invokes `benten_caps::validate_chain_with_manifest_envelope` does NOT exist as a type at HEAD (verified: `grep -rn "pub struct Production(Manifest|Write).*Recker|Validator"` returns nothing in `src/`). Layer-A empty-DID short-circuit at `engine.rs:1462-1490` IS live; per-resolvable-DID substantive recheck is NOT.

## Section 2 — Conservative-minimal end-state design (per surface)

### Overarching minimum-delta principles

1. **Reuse existing ErrorCodes.** Every needed typed code already exists: `WriteBoundaryChainNotUserRooted`, `PluginInstallRecordAlreadyApplied`, `ManifestEnvelopeRecheckUnresolvedDeny`, `PluginDelegationOutsideManifestEnvelope`, `PluginInstallConsentRequired`. **Zero new ErrorCode mints.**
2. **Prefer enabling the existing substrate over inventing new code paths.**
3. **Prefer test-corpus pinning the honesty gap over wide-API exposure.**
4. **Glue type lives in `benten-platform-foundation`** (dep-direction discipline).
5. **Where Path-A purist would mint a new ErrorCode, audit-test type, or "structural always-on" cascade across 75+ call sites — DON'T.** Defer to G-COMP-1.

### S1 — D-1 WriteBoundaryChainValidator

**Minimum change:** One new production type `ProductionWriteBoundaryChainValidator` at `benten-platform-foundation` (~80 LOC). Wires at the ONE path where untrusted chains arrive — `apply_atrium_merge`'s per-row inbound loop BEFORE `policy.check_write` at `engine.rs:1413`. Local engine-internal writes stay defended by `CapabilityPolicy::pre_write` per existing posture. Default-builder swap: platform-foundation builder installs the production type; bare `Engine::open` keeps Noop.

**LOC:** ~100. **Files:** 3 (foundation glue + engine.rs call site + test).
**Deferred (named):** "~10-15 local WRITE entry points" cascade stays in Row D-1 with TIGHTENED v1-beta posture text.

### S2 — D-2 InstallRecordReplayStore lifecycle-required wiring

**Minimum change:** Keep `Option<>` shape on `InstallPorts.install_record_replay_check`; change default-builder pattern so production callers ALWAYS pass `Some(closure)`. New helper `make_engine_replay_check_closure(store) -> Box<InstallRecordReplayCheckFn>` (~10 LOC). Engine holds the store + accessor (mirrors rechecker pattern). Production install_plugin call sites wire `Some(&mut closure)`.

**LOC:** ~120. **Files:** 4.
**Deferred:** Option→required conversion + manifest_store::install_plugin gate stay in Row D-2 with tightened posture.

### S3a — D-3 hook A `check_install_consent`

**Minimum change:** Add ONE call to `policy.check_install_consent(payload_hash, plugin_did_str)` in `plugin_lifecycle.rs` step 3b immediately after replay-check at line 901-905. Hook defaulted to `Ok(())`. REUSE existing `PluginInstallConsentRequired` ErrorCode.

**LOC:** ~30. **Files:** 2-3.
**Deferred:** N/A — fully closes.

### S3b — D-3 hook B `check_per_delegation`

**Minimum change:** Add ONE call to `policy.check_per_delegation(source_principal_did, plugin_did, resolved_scope)` in `engine_caps.rs::delegate_capability` after `shares_policy_resolver` at ~line 558. Default `Ok(())`. REUSE existing `PluginDelegationOutsideManifestEnvelope` ErrorCode.

**LOC:** ~25. **Files:** 2.
**Deferred:** N/A — fully closes.

### S3c — D-3 hook C `check_write_with_audience` — **FORK FOR ORCHESTRATOR/BEN**

- **FORK-A (conservative-minimal):** swap `policy.check_write(&ctx)` → `policy.check_write_with_audience(&ctx)` at `engine.rs:1413`. Default delegates to `check_write` preserving back-compat. `audience_did = None` left as-is. ~20 LOC. Partial-close: hook no longer silently ignored, but substantive audience-population deferred.
- **FORK-B:** also populate `audience_did` from resolved peer-DID. ~30 LOC. Risk: conflates "transport peer" with "cap audience" — subtle layering error.
- **FORK-C:** defer S3c entirely. Keep Row D-3's hook C deferred to G-COMP-1. Most conservative; D-3 closes 2/3.

**Recommendation: FORK-A** (partial-close with tightened DEFERRED.md narrative).

### S4 — D-4 ProductionManifestEnvelopeRechecker

**Minimum change:** New production type `ProductionManifestEnvelopeRechecker` at platform-foundation (~120 LOC). Implements `recheck_row` consulting PluginLibrary + UserDidRegistry + invoking `benten_caps::validate_chain_with_manifest_envelope`. Returns `UnresolvedDeny` honestly per §4.36. Default-builder swap installs production type when PluginLibrary present.

**LOC:** ~200. **Files:** 3.
**Deferred:** Row D-18 (synthesized-fallback hardening) naturally closes via D-4. UPDATE D-18 to CLOSED.

## Section 3 — Trade-offs explicit (vs PLANNER-A purist)

| Surface | Conservative-minimal GIVES UP | Acceptable in v1-beta? | Upgrade path G-COMP-1 |
|---|---|---|---|
| **S1 (D-1)** | Skips 10-15 local WRITE entry points cascade | YES — local paths defended by pre_write | Mechanical audit + lift |
| **S2 (D-2)** | Keeps Option<> shape | YES — production-callers-always-Some | Option→required + fixture cascade |
| **S3a (D-3-A)** | Identical to purist | YES | N/A |
| **S3b (D-3-B)** | Identical to purist | YES | N/A |
| **S3c (D-3-C)** | **FORK** — see above | Depends on Ben's call | Plumb audience_did correctly |
| **S4 (D-4)** | Skips workspace-walker audit-test | YES — D-17 has own row | N/A for D-4 |

## Section 4 — Cross-cutting concerns

- **Wire-format:** ZERO changes.
- **ErrorCode catalog:** ZERO new mints (single biggest delta vs purist).
- **Cap policy back-compat:** every change exploits trait defaults.
- **Sealed-trait discipline:** unchanged.
- **napi binding:** verify ~5-10 LOC delta during impl.
- **Browser-backend cfg:** no new wasm32 implications.
- **Tests:** ~6 new test files (~250 LOC).
- **Docs:** Update DEFERRED.md rows D-1/2/3/4 + SECURITY-POSTURE.md #26 + CLAUDE.md #18. Doc updates in LAST group per Rule #6.

## Section 5 — Scope estimate

| Surface | LOC |
|---|---|
| S1 D-1 | ~100 |
| S2 D-2 | ~120 |
| S3a | ~30 |
| S3b | ~25 |
| S3c (FORK-A) | ~20 |
| S4 D-4 | ~200 |
| Doc updates | ~80 |
| **Total** | **~575 LOC** |

Well under 800-LOC single-agent sweet-spot. Range ~555-635 across S3c forks.

## Section 6 — Forks for orchestrator triage

- **F1** — S3c audience-population (FORK-A vs FORK-B vs FORK-C). Recommendation: FORK-A.
- **F2** — S2 Option→required vs Option-kept. Recommendation: keep Option<>.
- **F3** — ErrorCode forensic-discrimination (mint discriminator codes vs reuse). Recommendation: reuse + defer split to G-COMP-1.

## Section 7 — Implementation phasing

Single-agent dispatch, single bundle. ~575 LOC well within capacity.

**Commit sequence (one branch, 7 commits, one PR):**
1. `feat(plugin-trust): wire ProductionWriteBoundaryChainValidator at apply_atrium_merge (D-1 close)`
2. `feat(plugin-trust): wire InstallRecordReplayStore through production install pipeline (D-2 partial close)`
3. `feat(plugin-trust): consult check_install_consent at install-pipeline step 3b (D-3-A close)`
4. `feat(plugin-trust): consult check_per_delegation in engine_caps.delegate_capability (D-3-B close)`
5. `feat(plugin-trust): switch apply_atrium_merge per-row policy call to check_write_with_audience (D-3-C partial close, FORK-A)`
6. `feat(plugin-trust): wire ProductionManifestEnvelopeRechecker via default-builder (D-4 close)`
7. `docs(phase-4-meta-core): close + tighten DEFERRED.md rows D-1..D-4 + retense Compromise #26 + CLAUDE.md #18`

## Critical Files for Implementation

- `crates/benten-engine/src/engine.rs` — call-site wiring for S1 (~line 1430), S3c (~line 1413), EngineInner field/accessor for S2
- `crates/benten-platform-foundation/src/plugin_lifecycle.rs` — call-site wiring for S2 + S3a; host for new Production glue types for S1 + S4
- `crates/benten-engine/src/engine_caps.rs` — call-site wiring for S3b at `delegate_capability` ~line 558
- `crates/benten-engine/src/install_record_replay.rs` — add `make_engine_replay_check_closure` helper
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — strike-through + tighten Rows D-1/2/3/4 (D-18 closes via D-4)
