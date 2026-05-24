# F4 Design — Triaged Synthesis Draft v3

> **Status:** ROUND 2.5 orchestrator re-triage after META-CRITIC ROUND-2 returned **ITERATE** with 7 unclosed/partial findings + 3 NEW Δ-introduced concerns + F1 surfacing failure flagged as binding gate. Written 2026-05-24 PM.
>
> **v3 binding correction:** F1 was pre-picked path-(a) full cascade in v2 per night-shift stance. META-CRITIC rebutted: per `feedback_surface_arch_decisions_under_auth` the F1 question (CLAUDE.md #18 Layer-1 narrative + Compromise #26 contract + 3-legitimate-paths combo) is a REAL arch fork night-shift autonomy does NOT cover. **v3 explicitly defers F1 to Ben as a 3-option fork (a/b/c)** rather than predicting.
>
> Reads as incremental diff against v2 at `.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v2.md`. Per-section deltas; unchanged sections noted "(v2 stands)".
>
> Source docs (all on staging branches; force-add per `.addl/` gitignore):
> - `F4-DESIGN-TRIAGED-DRAFT-v1.md` + `v2.md` (`phase-4-meta-core/r6-r1-f4-design-staging`)
> - `F4-DESIGN-CRITIC-1-security-threat-model.md` (`phase-4-meta-core/r6-r1-f4-design-critic-1`)
> - `F4-DESIGN-CRITIC-2-api-sealing-dep-direction.md` (`phase-4-meta-core/r6-r1-f4-design-critic-2`)
> - `F4-DESIGN-META-CRITIC-ROUND-2.md` (`phase-4-meta-core/r6-r1-f4-design-meta-critic-round-2`)

## v3 deltas

### Δv3-1 — F1 OPENED AS 3-OPTION BEN-FORK (binding correction per META-CRITIC)

**v2 said:** pre-decided F1 path-(a) full ~13-site cascade per night-shift prediction.

**META-CRITIC rebuttal:** night-shift autonomy does NOT cover the F1 fork. v2 foreclosed the Pareto middle path without acknowledgment.

**v3 fix:** explicitly defer F1 to Ben. Document all 3 paths with their trade-offs:

#### F1 option (a) — Full ~13-site WRITE entry-point cascade (PLANNER-A path)
- **Scope:** wire `WriteBoundaryChainValidator::admit_write_chain(...)` at all ~13 WRITE entry points enumerated by PLANNER-A (engine_crud.rs:{68,157,169,175,189} + engine_caps.rs:683 + engine_views.rs:976 + engine_modules.rs:{259,325} + engine_diagnostics.rs:541 + engine_wait.rs:{888,1070} + engine.rs:3972 apply_atrium_merge + handler_versions.rs:202)
- **LOC for S1:** ~350
- **v1-beta posture:** "Layer-1 user-as-root structurally enforced at every WRITE admission" — claim is TRUE post-impl
- **Honesty gap closure:** complete; Compromise #26 + CLAUDE.md #18 substantively LIVE
- **Cost:** widest cascade; ~13 production sites to thread WriteAdmissionFrame through; highest test-burden

#### F1 option (b) — Hybrid 5-CRUD engine_crud wiring (META-CRITIC Pareto middle)
- **Scope:** wire validator at the 5 engine_crud user-facing CRUD APIs (create_node + update_node + delete_node + create_edge + delete_edge) + apply_atrium_merge + delegate_capability. **Skip** privileged engine-internal paths (engine_caps.rs:683 delegate-final-write + engine_views + engine_modules + engine_diagnostics + engine_wait + handler_versions) where `chain_anchor_cid = None` would be the right answer anyway.
- **LOC for S1:** ~250
- **v1-beta posture:** "Layer-1 user-as-root structurally enforced at every user-facing WRITE + sync-merge + delegation" — claim is TRUE post-impl; precise about user-facing-vs-privileged distinction
- **Honesty gap closure:** complete for the threat model (privileged writes don't carry plugin-rooted chains by construction)
- **Cost:** lower cascade than (a); cleaner semantic split (user-facing-vs-privileged); recommended by META-CRITIC as Pareto-optimal

#### F1 option (c) — Honest-disclose (no wiring at engine_crud)
- **Scope:** wire at apply_atrium_merge + delegate_capability only (v1's original hybrid). Mint **Row D-26** (not D-25 — Agent D's R6FP-D took D-25 for L18 ledger purpose) honestly disclosing "engine_crud direct-write paths do not enforce WriteBoundaryChainValidator at v1-beta; G-COMP-1 lifts to full cascade." Retense Compromise #26 substrate-only-where-it-can-be-substrate-only narrative. Retense CLAUDE.md #18 Layer-1 with "structurally enforced at sync-merge + delegation paths only at v1-beta." Delete the (incorrect) `pre_write` reference from Row D-1.
- **LOC for S1:** ~80 (just doc updates)
- **v1-beta posture:** "Layer-1 user-as-root structurally enforced at sync-merge + delegation; engine_crud direct-write is plain user-root posture (CapabilityPolicy::check_write at admission; NO chain-validation)" — claim is HONEST but narrower than CLAUDE.md #18 currently reads
- **Honesty gap closure:** via documentation tightening rather than wiring
- **Cost:** lowest LOC; widest honesty gap left open; v1-beta "Layer-1 enforced" claim becomes "Layer-1 partially enforced"

**Prediction (rebuttable):** my prediction remains path-(a) per F4 reversal pattern + "do it right not fast", but META-CRITIC's path-(b) Pareto argument is substantively strong (~100 fewer LOC + cleaner user-facing-vs-privileged distinction + same honesty closure for the threat model). **Ben to ratify (a/b/c).**

### Δv3-2 — Δ4 audience_did CATEGORY-ERROR FIX (per META-CRITIC NEW concern 1)

**v2 said:** populate `audience_did = Some(peer_did)` at apply_atrium_merge:1413 (and `Some(plugin_did)` at delegate_capability — that part is fine).

**META-CRITIC found:** `peer_did` is transport-principal; `audience_did` is cap-target-principal. Different concepts. v2 mechanically followed CRITIC-1's FIX-2 without verifying semantics.

**v3 fix:** **DO NOT** populate `audience_did = Some(peer_did)` at apply_atrium_merge — that's a category error. **DO** populate `audience_did = Some(plugin_did)` at delegate_capability — that IS the correct cap-target. For apply_atrium_merge: leave `audience_did = None` at the per-row context (the absence of audience IS the correct semantic for an inbound sync row), but **populate a NEW field `inbound_peer_did: Option<&str>`** if S3c audience-aware policy actually needs transport-principal information for forensic logging. The field is added to `CapWriteContext` ONLY if the policy hooks need it (verify by re-reading `check_write_with_audience` signature contract — if the hook can be informed purely from existing CapWriteContext fields + the chain-validation result from S1, no new field needed).

**v3 conservative position:** add NO new `CapWriteContext` field (preserves §1.A.FROZEN item 8 strict-additivity discipline). populate `audience_did = Some(plugin_did)` ONLY at delegate_capability. Tighten DEFERRED.md Row D-3-c to "audience-aware seam wired at delegate_capability per its natural audience (cap-target plugin_did); apply_atrium_merge per-row writes are inbound sync where audience-absent is the correct semantic; G-COMP-1 lifts forensic transport-principal observation if needed."

**Mitigates:** META-CRITIC NEW concern 1 + S3c "swap-that-pretends-to-wire" residual.

### Δv3-3 — Δ7 S3c production_wiring_revert SPECIFIC SHAPE (per META-CRITIC NEW concern 3)

**v2 said:** `production_wiring_revert_would_fail` test pin pattern per surface.

**META-CRITIC found:** for S3c specifically, trait-default `check_write_with_audience = check_write` masks regression — reverting `apply_atrium_merge` from `_with_audience` back to `check_write` would still pass the production-arm test because default delegates anyway.

**v3 fix:** for S3c, the test must use a custom CapabilityPolicy whose `check_write_with_audience` impl ASSERTS-PANIC-IF-CALLED + whose `check_write` impl returns Ok. Then call apply_atrium_merge; if the swap is reverted, `check_write` is called (no panic) — TEST PASSES INCORRECTLY (false-negative). To fix: invert — the custom policy's `check_write_with_audience` returns Ok normally + `check_write` panics. Then if revert happens, panic fires + test FAILS. This is the right pin shape — explicitly assert the AUDIENCE-AWARE method was called.

**Mitigates:** META-CRITIC NEW concern 3 + Δ7 S3c sufficiency.

### Δv3-4 — F-1.1 NOT-CLOSED: V1-BETA-BREAKING-CHANGES + fixture enum

**v2 said:** mentioned "Option drop is a breaking change" but did not enumerate fixtures or add breaking-change row.

**v3 fix:** add to F4 implementer brief:
- **Enumerate ALL `InstallPorts` construction sites** workspace-wide via `git grep -nE 'InstallPorts\s*\{'` — surface count + per-fixture migration scope
- **Add V1-BETA-BREAKING-CHANGES.md row**: "Cohort 6 — InstallPorts.install_record_replay_check shape change Option<&mut Fn> → &mut Fn at PR<F4>; migration: production callers wire `engine.install_record_replay_store().record_and_check` closure; test fixtures wire `benten_platform_foundation::testing::noop_replay_check()`"
- **F4 implementer pre-push gate**: ALL fixtures compile + workspace tests pass (catches missed fixtures)

**Mitigates:** CRITIC-2 F-1.1 MAJOR.

### Δv3-5 — F-1.3 NOT-CLOSED: V1-FROZEN-INTERFACE.md item 8 retense

**v2 said:** N/A.

**v3 fix:** add to F4 implementer brief: retense `docs/V1-FROZEN-INTERFACE.md` §1.A.FROZEN item 8 entries that depend on capability-policy threading model:
- Update `InstallPorts.install_record_replay_check` field-type post-Option-drop
- Add NEW field `InstallPorts.policy: &dyn CapabilityPolicy` (per Δ2 v2 fix; CRITIC-2 F-1.2)
- Document the threading model as "CapabilityPolicy threaded via InstallPorts port; NOT exposed via Engine accessor" (preserve Class B β sealed boundary narrative)
- Mark any other §1.A.FROZEN items the v3 design touches

**Mitigates:** CRITIC-2 F-1.3 MINOR.

### Δv3-6 — F-4.3 NOT-CLOSED: drift-detect + npm + ERROR-CATALOG coupling

**v2 said:** "8-surface mirror for both new mints; CATALOG 192→194."

**v3 fix:** add explicit checklist to F4 implementer brief covering each mirror surface:
- `crates/benten-errors/src/lib.rs`: add `PluginInstallConsentDenied` + `PluginPerDelegationDenied` variants (4 internal sites each: variant + as_static_str + from_str + routed_edge_label)
- `crates/benten-errors/tests/stable_shape.rs`: bump `variant_count_is_pinned` 192 → 194
- `packages/engine/src/errors.generated.ts`: add 2 TS classes + CODE_TO_CTOR entries (regenerate via script if applicable; verify pre-push)
- `docs/ERROR-CATALOG.md`: add 2 entries (192 → 194 CATALOG narrative)
- `scripts/drift-detect-error-variant-mirror.ts`: verify the new variants are detected as first-class on the next baseline regen; if a baseline file exists, regenerate
- `npm run drift:errors` pre-push: must PASS after all mirrors land
- `docs/public-api/benten-errors.txt` baseline regen
- **Coordination with Agent B**: if Agent B's `PluginUpgradeAuthorBroken` mint (L2-R6-MAJOR-1) lands first, F4 starts at CATALOG = 193 (post-Agent-B) → 195 (post-F4). If F4 lands first, Agent B rebases.

**Mitigates:** CRITIC-2 F-4.3 MAJOR.

### Δv3-7 — F-7.F4 NOT-CLOSED: S5 4-site enumeration + ucan_grounded exclude

**v2 said:** "workspace sweep `check_write` → `check_write_with_audience`; ALL sites; engine_diagnostics.rs:78 + engine.rs:1413."

**v3 fix:** F4 implementer brief enumerates explicitly:
- `git grep -nE 'policy\.check_write\b'` to discover all sites (avoid `check_write_with_audience` matches)
- Per CRITIC-2 F-7.F4: there are **4 expected sites** — engine.rs:1413 + engine_diagnostics.rs:78 + (TWO MORE per CRITIC-2 enumeration; F4 implementer to verify via grep)
- **EXCLUDE** `ucan_grounded.rs` (or wherever the ucan-grounded check lives) — that's a substrate-internal check that should NOT route through the policy hook
- Implementer must list the 4 sites explicitly in the commit body for cargo-public-api auditability

**Mitigates:** CRITIC-2 F-7.F4 MAJOR.

### Δv3-8 — F-8.3 NOT-CLOSED: 11-row checklist + D-11 + D-25 mint

**v2 said:** "Update DEFERRED.md rows D-1..D-4 + D-6 + D-18 + Compromise #26 + CLAUDE.md #18 + INTERNALS.md."

**v3 fix:** the checklist needs to enumerate 11 specific row updates per CRITIC-2 F-8.3:
1. D-1 retense (close OR mint companion row per F1 ratification)
2. D-2 CLOSE
3. D-3-a CLOSE
4. D-3-b CLOSE
5. D-3-c PARTIAL-CLOSE per Δv3-2 (audience_did at delegate_capability only)
6. D-4 CLOSE
7. D-6 CLOSE (bundled with S4)
8. D-11 retense if affected (per CRITIC-2 sibling-bundling concern)
9. D-18 CLOSE (bundled with S4)
10. D-26 MINT (only if Ben picks F1 path-(c)) OR omit
11. Cohort 6 entry in V1-BETA-BREAKING-CHANGES.md (per Δv3-4)

**Plus:** D-25 was minted by Agent D for L18 ledger purpose — F4 implementer must NOT collide (use D-26 if needed per F1 path-(c) honest-disclose). Coordinate the row numbering pre-merge.

**Mitigates:** CRITIC-2 F-8.3 MAJOR.

### Δv3-9 — FIX-6 PARTIALLY-CLOSED: `#[deprecated]` + `#[doc(hidden)]` for side-door

**v2 said:** `#[doc(hidden)]` + trybuild.

**META-CRITIC found:** `#[deprecated]` dropped from spec.

**v3 fix:** add BOTH `#[doc(hidden)]` AND `#[deprecated(note = "use plugin_lifecycle::install_plugin which threads InstallRecordReplayStore")]` to `manifest_store::install_verified_record_unchecked`. The deprecation steers callers to the right path; the doc-hidden prevents accidental external rediscovery via cargo doc. Plus trybuild as in v2.

**Mitigates:** CRITIC-1 FIX-6 MINOR fully closed.

### Δv3-10 — F-6.1 PARTIALLY-CLOSED: napi S2 closure-threading file specificity

**v2 said:** "napi binding wires substantive replay-check closure via `engine.install_record_replay_store()`."

**META-CRITIC found:** v2 did not name the specific napi file to migrate.

**v3 fix:** name the specific files in implementer brief:
- `bindings/napi/src/lib.rs` ~lines 337-339 (the install entry point currently calling `install_plugin` without supplying `Some(closure)` for replay-check)
- If a more specific install entry-point exists (e.g. `bindings/napi/src/install.rs` or similar), verify via `git grep -nE 'install_plugin' bindings/napi/src/`
- The migration wires `ports.install_record_replay_check = &mut closure` where `closure = make_engine_replay_check_closure(engine.install_record_replay_store())` (post-Option-drop per S2)

**Mitigates:** CRITIC-2 F-6.1 MAJOR fully closed.

## Forks for Ben (v3 explicit surfacing)

| Fork | v3 status | Predicted answer if asked (per night-shift stance) |
|---|---|---|
| **F1** (S1 scope) | **OPEN — 3-option fork: (a) full cascade ~350 LOC / (b) hybrid 5-CRUD ~250 LOC / (c) honest-disclose ~80 LOC**. SURFACED to Ben; v3 designs against path-(b) Pareto-middle as the default-if-no-answer per META-CRITIC recommendation; predicts Ben picks (a) or (b). | **(b) hybrid** per META-CRITIC Pareto argument |
| F2 (S2 drop-Option) | unchanged | drop |
| F3 (S3a + S3b ErrorCode mints) | unchanged | mint both |
| F4 (S5 workspace sweep) | unchanged | sweep (4 sites per Δv3-7) |
| **F5** (S5 audience_did) | **REVISED per Δv3-2** — populate at delegate_capability ONLY (cap-target plugin_did is correct audience); leave None at apply_atrium_merge (per category-correctness; inbound sync is audience-absent semantically) | populate at delegate_capability only |
| F6 (EngineBuilder) | unchanged | ProductionEngineBuilder at foundation + napi compile/test pin |
| F7 (D-6 + D-18 bundling) | unchanged | bundle with S4 |
| ~~F8~~ | **DROPPED** (D-22/D-23/D-24 phantom verified) | N/A |

## Net scope (synthesis v3)

| Surface | LOC v2 | LOC v3 | Δ |
|---|---|---|---|
| S1 | ~350 (path-a) | **VARIES BY F1** — (a) ~350 / (b) ~250 / (c) ~80 | depends on Ben |
| S2 | ~80 | ~80 + ~10 (deprecated annotation) | +10 |
| S3a | ~70 | ~70 | 0 |
| S3b | ~55 | ~55 | 0 |
| S3c | ~50 | ~30 (NO new CapWriteContext field; populate only at delegate_capability) | -20 |
| S4 + D-6 + D-18 | ~340 | ~340 | 0 |
| Δv3-4 BREAKING-CHANGES row + fixture enum | new | ~30 | +30 |
| Δv3-5 V1-FROZEN-INTERFACE.md item 8 retense | new | ~20 | +20 |
| Δv3-6 ERROR-CATALOG + drift-detect + TS regen | partial | +30 | +30 |
| Napi pin (Δ6) | ~40 | ~40 | 0 |
| Test pins (Δ7 corrected for S3c) | ~150 | ~150 | 0 |
| Docs (DEFERRED + Compromise + CLAUDE.md + INTERNALS + 11-row checklist) | ~120 | ~150 | +30 |

**Total (assuming Ben picks F1 path-(b) hybrid):** ~975 production LOC + ~1040 test LOC ≈ **~2015 total**. Still single-agent feasible.

**Total (F1 path-(a)):** ~1075 production LOC. Still single-agent feasible.

**Total (F1 path-(c)):** ~805 production LOC (no S1 wiring; just docs). Easiest.

## Implementation phasing (v3 reaffirms 3-stage sequential)

Per META-CRITIC's Δ1+Δ3 concern about wall-clock:
1. **Stage 1:** Agent A (R6FP-A cfg+visibility) lands first — F4 needs F2 visibility tighten as prerequisite (napi `put_node` becomes `pub(crate)`).
2. **Stage 2:** Agent B (R6FP-B crypto+security+wire) lands second — F4 needs CATALOG settled + `PluginUpgradeAuthorBroken` mint coordination.
3. **Stage 3:** F4 implementer dispatches — single agent, 7 commits, ~975-1075 LOC depending on F1 ratification.

**Wall-clock:** ~3 hours per stage × 3 stages = ~9 hours total. Plus mini-review + CI cycles. **Surface to Ben** that this is sequential not parallel — may want to accept the longer wall-clock or pick a different F1 path that allows F4 to start earlier (path-(c) honest-disclose has no Agent A dependency since it doesn't touch napi binding).

## Coordination notes (unchanged from v2 + additions)

- F4 implementer MUST start AFTER Agent A lands (F2 visibility tighten prerequisite for C1 BLOCKER mitigation under F1 paths a/b; not needed under path c)
- F4 implementer MUST NOT touch `plugin_manifest.rs::verify_peer_signature`, `install_record.rs::verify_user_signature`, or `module_ecosystem.rs::verify_upgrade_author_continuity` — Agent B's scope
- F4 implementer MUST coordinate CATALOG bump (192→194 for F4's 2 mints; if Agent B's mint lands first → start at 193 → 195)
- **NEW:** F4 implementer must coordinate Row numbering in DEFERRED.md (Agent D took D-25; F4 uses D-26 only if F1 path-(c))
- **NEW:** F4 implementer must ratify the v3 design with the Ben-ratified F1 path explicitly stated in the brief; if F1 unratified at dispatch time, default to path-(b) per META-CRITIC Pareto recommendation

## CRITICAL: F1 binding-gate posture

Per `feedback_surface_arch_decisions_under_auth`: F1 IS a real arch fork that overshadows v1-beta security narrative. v3 does NOT pre-pick. If Ben unavailable: default to path-(b) (META-CRITIC Pareto recommendation); if Ben picks (a) or (c) post-dispatch, F4 implementer follows actual ratification + revises scope/docs accordingly.
