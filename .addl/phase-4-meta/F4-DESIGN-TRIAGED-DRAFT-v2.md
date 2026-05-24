# F4 Design — Triaged Synthesis Draft v2

> **Status:** ROUND 1.5 orchestrator re-triage after CRITIC-1 (security/threat-model) + CRITIC-2 (API/sealing/dep-direction) returned. Both critics: APPROVE-WITH-FIX-NOW. CRITIC-1 also flagged 1 BLOCKER + 1 HARD-ESCALATE. Written 2026-05-24 ~PM.
>
> **Reads incremental diff against v1** at `.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v1.md`. Per-section changes called out; unchanged sections noted "(v1 stands)".
>
> **Critic source docs:**
> - `.addl/phase-4-meta/F4-DESIGN-CRITIC-1-security-threat-model.md` (branch `phase-4-meta-core/r6-r1-f4-design-critic-1` @ `7078bf91`)
> - `.addl/phase-4-meta/F4-DESIGN-CRITIC-2-api-sealing-dep-direction.md` (branch `phase-4-meta-core/r6-r1-f4-design-critic-2` @ `a794e8b2`)

## Material changes vs v1 (synthesis v2 deltas)

### Δ1 — S1 SCOPE EXPANSION + Ben-Fork SURFACING (per CRITIC-1 BLOCKER C1 + F1 SUBSTANTIVE REBUTTAL)

**v1 said:** wire validator at apply_atrium_merge + delegate_capability only; engine_crud defended by `CapabilityPolicy::pre_write` per Compromise #2.

**CRITIC-1 found:** `pre_write` method does NOT EXIST in the CapabilityPolicy trait. CRUD APIs do NOT call `check_write`. The Row D-1 line 81 claim is **architecturally false**. C1 BLOCKER: napi `put_node` composes to full Layer-1 chain-validation bypass.

**v2 PRE-DECISION (predicted Ben answer):** **F1 path-(a) full cascade per PLANNER-A** — wire `WriteBoundaryChainValidator` at all ~13 WRITE entry points enumerated by PLANNER-A. Closes the honesty gap; v1-beta ships the security guarantee CLAUDE.md #18 Layer-1 claims. **Surfacing as F1 to Ben for explicit ratification** — rebuttable but pre-picked per night-shift stance.

Alternative if Ben picks F1 path-(b) honest-disclose: mint Row D-25 ("engine_crud direct-write paths do not enforce WriteBoundaryChainValidator at v1-beta; G-COMP-1 lifts to full cascade"); retense Compromise #26 substrate-only narrative; retense CLAUDE.md #18 Layer-1 with "structurally enforced at sync-merge + delegation paths only at v1-beta"; delete `pre_write` reference from Row D-1.

**Net LOC delta:** S1 ~180 → ~350 (PLANNER-A budget).

**Coordination with Agent A (R6FP-A cfg+visibility):** Agent A is making `Engine::get_node`/`put_node` `pub(crate)`. After F2 lands, napi `put_node` no longer exists as public surface. The C1 BLOCKER is PARTIALLY closed by F2 — napi must route writes through a different mechanism (likely a sealed write-port that itself calls `admit_write_chain`). v2 brief for F4 implementer must include explicit coordination with the F2 visibility tighten.

**Mitigates:** CRITIC-1 BLOCKER C1; H1 + H2 honesty residuals.

### Δ2 — S3a CAPABILITY-POLICY THREADING (per CRITIC-2 F-1.2)

**v1 said:** new `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` accessor; install_plugin step 3c calls `policy.check_install_consent`.

**CRITIC-2 found:** the new accessor violates Class B β `EngineCapsHandle`-canonical invariant + over-exposes sealed `CapabilityPolicy`.

**v2 fix:** thread `policy: &dyn CapabilityPolicy` via `InstallPorts` port (matches existing `*Ports` discipline; same LOC cost; **no new engine public-API surface**). Install pipeline caller (foundation's install path + napi binding) supplies the policy reference. Step 3c becomes:
```rust
ports.policy.check_install_consent(&payload_hash, plugin_did_str)
    .map_err(|cap_err| ErrorCode::from(cap_err))?;
```

**Mitigates:** CRITIC-2 F-1.2 MAJOR.

### Δ3 — S3b ERRORCODE MINT FOR SYMMETRY (per CRITIC-1 FIX-5)

**v1 said:** S3b reuses existing `PluginDelegationOutsideManifestEnvelope`.

**CRITIC-1 found:** asymmetric forensic discrimination across S3a (typed `PluginInstallConsentDenied`) vs S3b (reused untyped) — closes C2 cross-surface chain.

**v2 fix:** mint **`ErrorCode::PluginPerDelegationDenied`** for S3b. CATALOG bump 192 → **194** (S3a's `PluginInstallConsentDenied` 193 + S3b's `PluginPerDelegationDenied` 194). Both go through §3.5g 8-surface mirror.

**Mitigates:** CRITIC-1 FIX-5 + CRITIC-1 C2 chain.

### Δ4 — S3c POPULATE audience_did AT NATURAL-AUDIENCE SITES (per CRITIC-1 F5 PARTIAL REBUTTAL)

**v1 said:** workspace sweep `check_write` → `check_write_with_audience`; leave `audience_did = None` everywhere (partial-close per PLANNER-B FORK-A).

**CRITIC-1 found:** S3c hook receives `audience_did = None` everywhere → audience-aware policy override silently ignored at runtime. CLAIMS-DEFEAT-BUT-DOESNT (the "swap-that-pretends-to-wire" pattern).

**v2 fix:** populate `audience_did` at **the 2 natural-audience sites**:
- `engine.rs:1413` apply_atrium_merge per-row → `audience_did = Some(peer_did)`
- `engine_caps.rs::delegate_capability` (new S3b call site) → `audience_did = Some(plugin_did)`

Other sites stay `audience_did = None` (matches absence of a natural audience). Tighten DEFERRED.md Row D-3-c narrative: "audience-aware seam wired at apply_atrium_merge + delegate_capability per their natural audiences; engine_crud + diagnostics + privileged sites have None per absence-of-audience semantics."

**Mitigates:** CRITIC-1 F5 + CRITIC-1 S3c "CLAIMS-DEFEAT-BUT-DOESNT" + closes H3 honesty residual.

### Δ5 — S4 REUSE EXISTING UserDidRegistry + RENAME BUILDER (per CRITIC-2 F-2.1 + F-2.2)

**v1 said:** mint new sealed `UserDidRegistry` trait in foundation; concrete impl foundation-side; `EngineBuilder::build()` is canonical production constructor.

**CRITIC-2 found:**
- F-2.1: `UserDidRegistry` already exists at `crates/benten-caps/src/manifest_envelope_chain_validation.rs`. Don't re-mint (type-name collision risk per `RestrictedSpec`/`RestrictedScope` rename incident).
- F-2.2: `EngineBuilder` is engine-side; foundation builder must NOT shadow it.

**v2 fixes:**
- **REUSE existing `benten-caps::UserDidRegistry`** trait. No new mint.
- **Concrete impl belongs ENGINE-side** per data-locality (install-record store IS engine-side). Add `InstallRecordUserDidRegistry` in `crates/benten-engine/src/install_record_replay.rs` (or sibling module). Foundation's production rechecker consumes the engine-provided concrete via the trait.
- **Rename foundation builder to `ProductionEngineBuilder`** (or `build_production_engine` factory function). Raw `Engine::default()` stays test/embedded.

**Mitigates:** CRITIC-2 F-2.1 + F-2.2 + S4 type-name collision risk.

### Δ6 — NAPI-VIA-FOUNDATION-BUILDER COMPILE/TEST PIN (per CRITIC-1 FIX-3)

**v2 add:** compile/test pin at `bindings/napi/tests/r6_r1_fp_c_napi_engine_routes_through_production_builder.rs` (or trybuild test) asserting napi binding's Engine constructor goes through `benten_platform_foundation::ProductionEngineBuilder::build()`, NOT raw `Engine::default()`. Prevents accidental Noop ship in release binary.

**Mitigates:** CRITIC-1 FIX-3 + F6 ACCEPT+ADD-PIN.

### Δ7 — PRODUCTION-ARM TEST PINS BEYOND SUBSTRATE (per CRITIC-1 FIX-4)

**v1 said:** test pins per surface (production-arm + adversarial + capability-rejection + sweep-completeness).

**CRITIC-1 found:** several proposed tests would pass even if production wiring was reverted (substrate tests + sweep-completeness gap). C4 chain attack.

**v2 add:** for EACH surface, add a **`production_wiring_revert_would_fail`** test pin pattern — explicitly constructs an Engine via `ProductionEngineBuilder::build()` then attempts the attack scenario; assertion that succeeds proves production wiring is live (not the substrate). Test corpus expansion ~150 LOC across the 6 surfaces.

**Mitigates:** CRITIC-1 FIX-4 + C4 chain + §3.6j sweep-completeness self-verify.

### Δ8 — S2 SIDE-DOOR `#[doc(hidden)]` + TRYBUILD (per CRITIC-1 FIX-6)

**v2 add:** `manifest_store::install_verified_record_unchecked` annotated `#[doc(hidden)]` (in addition to the SAFETY doc); trybuild compile-fail test asserting attempting to call from outside the workspace produces a doc-hidden warning. Hardens against accidental external rediscovery.

**Mitigates:** CRITIC-1 FIX-6 MINOR.

### Δ9 — S3a STEP 3c ORDERING DOC-COUPLING (per CRITIC-1 FIX-7)

**v2 add:** in `plugin_lifecycle.rs` step 3c implementation, include rustdoc comment explicitly documenting the ordering invariant: "Step 3c runs AFTER replay-check (3b) and BEFORE clock-validation (4) so that policy-routed denials see a fresh-bytes presentation but precede expensive cryptographic work." Update `crates/benten-platform-foundation/INTERNALS.md` install-pipeline ordering diagram to include step 3c.

**Mitigates:** CRITIC-1 FIX-7 MINOR.

### Δ10 — S4 D-18 DISCRIMINATOR SPECIFICITY (per CRITIC-1 FIX-8)

**v2 add:** spec the D-18 synthesized-`node-id:` rejection mechanism precisely. The `ProductionManifestEnvelopeRechecker::recheck_row` checks: `if peer_did_str.starts_with("node-id:") { return UnresolvedDeny; }` (per PLANNER-A's draft) — make this a NAMED helper `fn is_synthesized_node_id(did_str: &str) -> bool` so the discriminator is testable in isolation + can be reused at `handshake.rs` D-6 site. Add 1 unit test per `is_synthesized_node_id` enumerating positive ("node-id:42") + negative ("did:key:...") inputs.

**Mitigates:** CRITIC-1 FIX-8 MINOR + D-18 rigor.

### Δ11 — DROP F8 OR VERIFY D-22/D-23/D-24 (per CRITIC-1 HARD-ESCALATE F8)

**v1 said:** "F8 — include D-24 in F4 scope; D-22 + D-23 separate."

**CRITIC-1 found:** D-22, D-23, D-24 do NOT exist in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (verified `grep -n "^### "`). Per HARD RULE 12 clause-(b) this is a phantom destination.

**v2 fix:** **DROP F8 entirely from synthesis.** Agent A (R6FP-A) is closing F1 cfg-gating which would *create* a Row D-22 conditionally — but per Ben's F1 retract, no Row D-22 is needed. D-23 + D-24 references appear to have been hallucinated by my v1 synthesis (citing CRITIC-1's L6 lens-narrative reference that I evidently misattributed). v2 drops F8.

**Mitigates:** CRITIC-1 HARD-ESCALATE F8 — closed by drop.

### Δ12 — S7/S8/S9 COORDINATION WITH AGENT B (per CRITIC-1 missed-surfaces section)

**CRITIC-1 named:** `verify_peer_signature` + `verify_user_signature` at `plugin_manifest.rs:178` + `:597` are classical-Ed25519-only; T10-upgrade gap at L2-R6-MAJOR-1.

**v2 note:** these surfaces are **OUT OF F4 SCOPE** — they are owned by sibling agent **R6FP-B (crypto+security+wire)** as L2-R6-MAJOR-1 (T10 caller wire-in) + L2-R6-MAJOR-2 (PQ-hybrid 3-site wiring) per the original R6 R1 FP wave-1 dispatch. F4 implementer must NOT touch these to avoid merge conflicts with Agent B. If Agent B HARD-ESCALATES on L2-R6-MAJOR-2 (the PQ-hybrid wiring decision), the resulting fork is independent of F4.

**Mitigates:** CRITIC-1 missed-surfaces by explicit out-of-scope assignment.

## Net scope (synthesis v2)

| Surface | Rust LOC v1 | Rust LOC v2 | Δ | Reason |
|---|---|---|---|---|
| S1 (F1 path-a full cascade) | ~180 | **~350** | +170 | CRITIC-1 BLOCKER C1 + F1 rebuttal |
| S2 | ~80 | ~80 | 0 | unchanged (+~20 LOC for FIX-6 doc-hidden+trybuild) |
| S3a | ~70 | **~70** | 0 | shape changes (port-threaded) but same LOC |
| S3b | ~25 | **~55** | +30 | mint PluginPerDelegationDenied + 8-surface mirror |
| S3c | ~30 | **~50** | +20 | populate audience_did at 2 sites |
| S4 + D-6 + D-18 | ~360 | **~340** | -20 | reuse UserDidRegistry (no re-mint) + rename ProductionEngineBuilder + named is_synthesized_node_id helper |
| NEW Δ6 napi pin | 0 | ~40 | +40 | compile/test pin |
| NEW Δ7 production-arm test pins | (in test LOC) | +150 test LOC | +150 test | substrate-tests-don't-suffice closure |
| Doc updates (DEFERRED.md + Compromise #26 + CLAUDE.md #18 + INTERNALS.md) | ~80 | ~120 | +40 | per Δ1 alt + Δ9 INTERNALS + Δ12 v1-beta narrative honesty |

**Total v2:** ~1105 production LOC + ~1040 test LOC ≈ **~2145 total**. **5 new files** (S1 production impl + S1 audit walker + S4 production rechecker + InstallRecordUserDidRegistry + napi pin). **2 new ErrorCode mints** (`PluginInstallConsentDenied` + `PluginPerDelegationDenied`; CATALOG **192→194**). **6 docs touched** (+ INTERNALS.md additions).

Still within single-agent sweet-spot at ~1100 production LOC (sweet-spot is ~800 per `feedback_subtrack_sizing_heuristic` but the planning-pipeline-pre-validated path is wider — Phase-2b precedent absorbs up to ~1500-2000 LOC on well-scoped single-agent dispatches).

## Forks remaining for Ben (synthesis v2)

| Fork | Status | Predicted Ben answer |
|---|---|---|
| **F1** (S1 scope: full-cascade vs honest-disclose) | **NEW from CRITIC-1; SURFACED** | **path-(a) full cascade** per F4 reversal pattern + "do it right not fast"; rebuttable |
| F2 (S2 drop-Option) | unchanged | drop |
| F3 (S3a ErrorCode mint) | unchanged + extended to S3b | mint both |
| F4 (S5 workspace sweep) | unchanged | sweep |
| F5 (S5 audience_did) | REVISED per CRITIC-1 | populate at 2 natural sites + None elsewhere |
| F6 (EngineBuilder location) | REVISED per CRITIC-2 | ProductionEngineBuilder at foundation + napi compile/test pin |
| F7 (D-6 + D-18 bundling) | unchanged | bundle with S4 |
| ~~F8 (D-22/D-23/D-24)~~ | **DROPPED — phantom destinations per CRITIC-1 HARD-ESCALATE** | N/A |

## Implementation phasing (unchanged from v1; single agent, 7 commits)

Same 7-commit sequence as v1 with the v2 modifications absorbed per surface. Each commit independently runs full test suite. Mini-review after the group. `cargo-public-api` regen per-PR (CATALOG bump and InstallPorts shape change both need baseline regen).

## Coordination notes for F4 implementer dispatch
- **F4 implementer MUST start AFTER Agent A (R6FP-A cfg+visibility) lands** — Agent A's §8-A visibility tighten (`get_node`→`read_node` pub(crate)) is the prerequisite for the C1 BLOCKER mitigation. F4 wiring assumes the visibility tighten is in place.
- **F4 implementer MUST NOT touch** `plugin_manifest.rs::verify_peer_signature`, `install_record.rs::verify_user_signature`, or `module_ecosystem.rs::verify_upgrade_author_continuity` — these are Agent B's scope (L2-R6-MAJOR-1 + L2-R6-MAJOR-2).
- **F4 implementer MUST coordinate ErrorCode CATALOG bump with Agent B's parallel mint** (`PluginUpgradeAuthorBroken`). If both land at the same CATALOG number, one must rebase. Recommended: F4 starts at CATALOG = post-Agent-B-merge.
