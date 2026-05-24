# F4 Design — Triaged Synthesis Draft v1

> **Status:** ROUND-0.5 orchestrator triage of PLANNER-A (architectural-purist) + PLANNER-B (conservative-minimal). Written 2026-05-24 ~18:30Z. To be reviewed by critics in ROUND 1; iterated to convergence per ratified F4 design pipeline.
> 
> **Synthesis principle:** take A's framing where v1-beta-freeze ergonomics favor it (drop-Option, mint discriminator code, sweep `check_write_with_audience` workspace-wide, EngineBuilder canonical constructor, sibling D-6+D-18 bundling). Take B's threat-model-right Pareto compression where A's full cascade exceeds the actual attack surface (S1 wires at apply_atrium_merge + delegate_capability only — the 2 sites where untrusted chains exist — rather than the full ~13 WRITE entry-point sweep; engine_crud Layer-1 stays with existing `CapabilityPolicy::pre_write` per Compromise #2).

## Surface table

| ID | Surface | DEFERRED row | Design decision (synthesis) |
|---|---|---|---|
| S1 | WriteBoundaryChainValidator at WRITE admission | D-1 | **HYBRID** — wire at `apply_atrium_merge` per-row (`engine.rs:1413`-adjacent) + `delegate_capability` Step 4 (`engine_caps.rs:683`). Skip ~11 other WRITE entry points; engine_crud relies on existing `CapabilityPolicy::pre_write` per Compromise #2 sub-narrative. D-1 stays open with TIGHTENED posture: "structurally enforced at every site where an untrusted chain can arrive (delegation + sync-merge); engine_crud's direct-user-write path defended by `CapabilityPolicy::pre_write` per existing posture." |
| S2 | InstallRecordReplayStore lifecycle wiring | D-2 | **A** — drop `Option<>` on `InstallPorts.install_record_replay_check`. New `noop_replay_check()` helper from `benten_platform_foundation::testing` for test fixtures. Rename `manifest_store::install_plugin` → `install_verified_record_unchecked` with SAFETY doc. ~12-fixture-site cascade is mechanical. D-2 fully closes. |
| S3a | check_install_consent | D-3-a | **A** — new Step 3c in plugin_lifecycle.rs after replay-check at :905. New `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` accessor. **Mint new `ErrorCode::PluginInstallConsentDenied`** (preserves R6-FP-A typed-discrimination invariant; CATALOG 192→193; 8-surface §3.5g mirror). D-3-a closes. |
| S3b | check_per_delegation | D-3-b | **AGREED** — single insertion at `engine_caps.rs:559` between Step 2b and Step 3. REUSE existing `PluginDelegationOutsideManifestEnvelope`. CapError → `EngineError::Cap`. D-3-b closes. |
| S3c | check_write_with_audience | D-3-c | **A swap + B FORK-A audience=None** — single-char swap at `engine.rs:1413` + workspace sweep all `policy.check_write(` → `_with_audience`. Default impl delegates to `check_write` preserving back-compat. `CapWriteContext.audience_did = None` left at sites that don't have a natural audience (per B's recommendation; audience-population deferred to G-COMP-1). D-3-c PARTIAL CLOSE; tighten DEFERRED.md narrative. |
| S4 | ProductionManifestEnvelopeRechecker + EngineBuilder + D-6 + D-18 | D-4 (+ closes D-6 + D-18) | **A** — ship `ProductionManifestEnvelopeRechecker` in `benten-platform-foundation` composing `PluginLibrary` × `UserDidRegistry` (new sealed trait) × `validate_chain_with_manifest_envelope`. `EngineBuilder::build()` is canonical production constructor wiring substantive impl. Raw `Engine::default()` keeps Noop for test/embedded posture. Bundle D-6 (handshake.rs §4.25 sync-hydrate UnresolvedDeny consumption) + D-18 (synthesized-`node-id:` rejection). D-4 + D-6 + D-18 close together. **EngineBuilder location: Path-(ii)** — foundation owns the production builder; raw `Engine::default()` stays as test/embedded posture. |

## Pre-decisions made by orchestrator (synthesis defaults)

Per night-shift stance + `feedback_surface_arch_decisions_under_auth`, these synthesis defaults are made AS-BEN with bias-to-continue; rebuttable at any time by Ben or by critics:

1. **Hybrid S1 scope** (compromise between A's full ~13-site cascade and B's single-site wire): the threat-model right answer. Wiring at `apply_atrium_merge` + `delegate_capability` covers every site where an untrusted chain arrives. engine_crud's direct-user-write path is already defended by `CapabilityPolicy::pre_write` per Compromise #2 sub-narrative. Net LOC: ~150-200 vs A's ~350 or B's ~100. **Forecloses A's "full structural enforcement everywhere" architectural-purist endgame; preserves it as a G-COMP-1 expansion.**

2. **Mint `PluginInstallConsentDenied`** (A's FORK A recommendation): preserves the R6-FP-A typed-discrimination invariant. CATALOG 192→193. §3.5g 8-surface mirror adds ~30 LOC of mechanical mirror work. **Forecloses B's "reuse the existing code with disambiguating message" path which would dilute the typed-discrimination invariant.**

3. **Workspace sweep `check_write` → `check_write_with_audience`** (A's S5 sweep): unifies the canonical seam. Adds ~5 LOC beyond B's single-line swap. **B's "single call site only" path leaves the seam inconsistent across the workspace.**

4. **`audience_did = None` at non-natural-audience sites** (B's FORK-A): partial-close acceptable for v1-beta. The audience-population question is deferred to G-COMP-1 with tightened DEFERRED.md narrative. **Forecloses A's FORK-B path-β/γ purist plumbing for v1-beta scope.**

5. **EngineBuilder at foundation** (A's FORK C Path-ii): single canonical production constructor in the layer that owns production state. napi binding wires through foundation. Raw `Engine::default()` stays test/embedded. **Forecloses A's Path-i generic-engine-builder + B's "existing default-builder pattern only" path.**

6. **Bundle D-6 + D-18 with S4** (A's S6 + sibling-sweep recommendation): D-6 (handshake.rs §4.25) is architecturally inseparable from D-4's production rechecker (same primitive, sibling site). D-18 (synthesized-node-id rejection) is safe-to-land only AFTER D-4's substantive rechecker is installed. **Forecloses B's "D-18 closes naturally via D-4" position (it doesn't — D-18 needs an explicit synthesized-DID rejection branch).**

## Net scope (synthesis)

| Surface | Rust LOC | Test LOC | New files |
|---|---|---|---|
| S1 (hybrid) | ~180 (2 wire sites + helper + production impl in foundation) | ~150 (production-arm + adversarial + ordering + sweep-walker) | 1 (production impl) |
| S2 | ~80 (drop Option + helper + rename + ~12 fixture sweep) | ~100 (e2e + parallel-replay + back-compat + trybuild) | 0 |
| S3a | ~70 (Step 3c + accessor + ErrorCode mint + 8-surface mirror) | ~120 (admit + denial + ordering) | 0 |
| S3b | ~25 | ~120 | 0 |
| S3c | ~30 (workspace sweep + back-compat tests) | ~100 | 0 |
| S4 + D-6 + D-18 | ~360 (production rechecker + UserDidRegistry + EngineBuilder + handshake.rs wire + D-18 branch) | ~300 (5 production pins + D-6 wire pin + D-18 pin + default-builder substantive verify) | 2 (production rechecker + user_did_registry) |
| Doc updates (DEFERRED.md retense + Compromise #26 retense + CLAUDE.md #18 + INTERNALS.md) | ~80 | 0 | 0 |

**Total:** ~825 production LOC + ~890 test LOC ≈ **~1715 total**. **3 new files. 1 new ErrorCode mint** (`PluginInstallConsentDenied`; CATALOG 192→193). **5 docs touched.**

## Forks remaining for Ben (synthesis-pre-decided but rebuttable)

The 6 numbered pre-decisions above are made AS-BEN per night-shift stance. Each is rebuttable; surfacing here for explicit ratification:

- **F1** — S1 scope (hybrid 2 sites vs full ~13 vs single site). Synthesis: **hybrid**.
- **F2** — S2 Option shape (drop vs keep). Synthesis: **drop**.
- **F3** — S3 ErrorCode (mint vs reuse). Synthesis: **mint** `PluginInstallConsentDenied`.
- **F4** — S5 sweep scope (workspace vs single-site). Synthesis: **workspace sweep**.
- **F5** — S5 audience_did (plumb vs None). Synthesis: **None at non-natural-audience sites** (partial-close).
- **F6** — S4 EngineBuilder location (engine vs foundation vs both). Synthesis: **foundation**.
- **F7** — D-6 + D-18 bundling. Synthesis: **bundle with S4**.
- **F8** — D-24 cross-wave coupling. Synthesis: **include in F4** (couples to S4 via L6 lens narrative).

## Implementation phasing

Single F4 implementer agent (within sweet-spot per `feedback_subtrack_sizing_heuristic` 800-LOC budget; ~825 production LOC; acceptable). 7 commits, one PR:

1. `feat(plugin-trust): wire ProductionWriteBoundaryChainValidator at apply_atrium_merge + delegate_capability (D-1 partial-close, hybrid)`
2. `feat(plugin-trust): drop Option on InstallPorts.install_record_replay_check + rename manifest_store side-door (D-2 close)`
3. `feat(plugin-trust): consult check_install_consent at install-pipeline step 3c + mint PluginInstallConsentDenied (D-3-a close; CATALOG 192→193 + §3.5g 8-surface mirror)`
4. `feat(plugin-trust): consult check_per_delegation in engine_caps.delegate_capability (D-3-b close)`
5. `feat(plugin-trust): sweep workspace policy.check_write → check_write_with_audience (D-3-c partial-close, FORK-A)`
6. `feat(plugin-trust): wire ProductionManifestEnvelopeRechecker via EngineBuilder + handshake.rs D-6 + D-18 (D-4 + D-6 + D-18 close)`
7. `docs(phase-4-meta-core): retense DEFERRED.md rows D-1..D-4 + D-6 + D-18 + Compromise #26 + CLAUDE.md #18 + INTERNALS.md`

Each commit independently runs full test suite. Mini-review after the group. `cargo-public-api` regen per-PR.

## Critical files
- `crates/benten-engine/src/engine.rs` — apply_atrium_merge S1 wire + S3c sweep + EngineBuilder integration (mods around :1413, :1448-1499, :1923-1949)
- `crates/benten-engine/src/engine_caps.rs` — delegate_capability S1 + S4 (~:559, :683)
- `crates/benten-platform-foundation/src/plugin_lifecycle.rs` — S2 + S3a (drop Option, add Step 3c; ~:696-722, :885-905)
- `crates/benten-platform-foundation/src/production_manifest_envelope_rechecker.rs` (NEW) — S4 production impl
- `crates/benten-platform-foundation/src/user_did_registry.rs` (NEW) — sealed trait
- `crates/benten-platform-foundation/src/engine_builder.rs` (extend or NEW) — Path-(ii)
- `crates/benten-engine/src/install_record_replay.rs` — helper for S2
- `crates/benten-sync/src/handshake.rs` — D-6 wire
- `crates/benten-errors/src/lib.rs` — `PluginInstallConsentDenied` mint (4 internal sites)
- `packages/engine/src/errors.generated.ts` — TS mirror
- `docs/ERROR-CATALOG.md` — CATALOG 192→193
- `crates/benten-errors/tests/stable_shape.rs` — variant_count_is_pinned bump
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — retense Rows D-1 (tightened-not-closed), D-2 (closed), D-3 (a-closed, b-closed, c-partial-closed), D-4 (closed), D-6 (closed), D-18 (closed)
- `docs/SECURITY-POSTURE.md` — Compromise #26 substantive update
- `docs/CLAUDE.md` (project root) — baked-in #18 retense
- INTERNALS.md per affected crate
