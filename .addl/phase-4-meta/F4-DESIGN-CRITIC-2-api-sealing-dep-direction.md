# F4 DESIGN — CRITIC-2 — API surface + sealed-discipline + dep-direction + §1.A.FROZEN compliance

> **Branch:** `phase-4-meta-core/r6-r1-f4-design-critic-2`
> **Base SHA explored:** `a0b75637` (main HEAD at review time)
> **Source staging:** `origin/phase-4-meta-core/r6-r1-f4-design-staging` (commit `05c689d9`) — contains the 3 input docs the brief references.
> **Review angle:** §1.A.FROZEN-verifier + cargo-public-api baseline reviewer + sealed-discipline + dep-direction auditor + pim-N codifier
> **Date:** 2026-05-24

---

## TL;DR — overall disposition: **APPROVE-WITH-FIX-NOW**

The triaged synthesis is fundamentally on-target on dep-direction (foundation-side glue, sealed `UserDidRegistry` in foundation, EngineBuilder Path-(ii)). However it carries **5 FIX-NOW items** that must be resolved before R5 implementer dispatch, **4 sealing/coupling DISAGREE-WITH-EXPLANATION items** rebutting specific synthesis pre-decisions, and **3 BELONGS-NAMED-NOW items** that need explicit doc destinations in the synthesis itself (not left for R5 implementer authoring).

The single highest-leverage finding: **the synthesis treats `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` as a routine additive accessor when it is in fact a new public-API surface that pierces the Class B β `Engine::caps()` handle pattern + the sealed-CapabilityPolicy boundary** (Section 2.1). Net of these 5 FIX-NOWs the synthesis is sound; absent them, the wave would re-open the freeze with an inappropriate widening.

---

## Section 1 — §1.A.FROZEN compliance audit

### 1.1 InstallPorts field-type change (S2) — FIX-NOW + DEFAULT-CALL-NEEDS-EXPLICIT-RATIFICATION

**Finding F-1.1 (MAJOR).** The synthesis pre-decides F2 = "drop Option" on `InstallPorts.install_record_replay_check` (`crates/benten-platform-foundation/src/plugin_lifecycle.rs:713`). This IS a public-API breaking change with a real cargo-public-api delta:

```
docs/public-api/benten-platform-foundation.txt:777:
  pub benten_platform_foundation::plugin_lifecycle::InstallPorts::install_record_replay_check:
    core::option::Option<&'a mut benten_platform_foundation::plugin_lifecycle::InstallRecordReplayCheckFn>
→ field type changes to: &'a mut benten_platform_foundation::plugin_lifecycle::InstallRecordReplayCheckFn
```

The synthesis correctly recognizes this requires (a) cargo-public-api baseline regen and (b) the `~12-fixture-site cascade is mechanical` posture. But it misses TWO things the §1.A.FROZEN contract requires:

1. **V1-BETA-BREAKING-CHANGES.md entry MUST land in the same PR.** The Cohort 2 ("Public-API shape changes (renames + visibility)") section currently has zero `InstallPorts` row. The current ledger has 8 rows in Cohort 2; this would be #9. The synthesis enumerates "DEFERRED.md retense" + "Compromise #26 retense" + "CLAUDE.md #18 retense" but omits V1-BETA-BREAKING-CHANGES.md. **FIX-NOW:** add ledger row to commit 7's narrative.

2. **`InstallPorts` is NOT `#[non_exhaustive]` per docs/public-api/benten-platform-foundation.txt:775**. The field-type change is therefore breaking even with `Option<>` → `&mut` (pre-existing direct struct literal sites must convert). The "mechanical fixture sweep" framing accepts this; but the synthesis doesn't actually enumerate the fixture sweep targets. **Per `feedback_d17_same_file_sweep_recurrence`,** the F4 implementer brief MUST enumerate the cascade BEFORE dispatch, or D-17 recurrence will fire on a sibling pub-item miss. **FIX-NOW:** the synthesis should enumerate the ~12 (or actual) construction sites via `grep -rn "InstallPorts\s*{" crates/ bindings/ tests/` BEFORE the wave dispatches.

**Disposition:** FIX-NOW per HARD RULE 12 (a) before R5 dispatch authoring.

### 1.2 `Engine::capability_policy()` new accessor (S3a) — FIX-NOW (the headline finding)

**Finding F-1.2 (BLOCKER candidate, downgraded to MAJOR after analysis).** The synthesis pre-decides commit 3 mints a new `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` accessor at engine.rs.

This is **NOT** a routine additive accessor. It widens the Class B β boundary in ways the synthesis does not analyze:

1. **The Class B β boundary discipline** (CLAUDE.md baked-in #18) is: engine internals use `Engine::get_node(cid)` (un-attributed), the engine's PUBLIC attributed surface is `Engine::read_node_as(principal, cid)`, and capability-related operations route through `Engine::caps() -> EngineCapsHandle` at `engine.rs:1703`. The `EngineCapsHandle` is the **sole public seam** for capability operations per `engine_caps.rs:8` ("lives **exclusively** on [`EngineCapsHandle`] — the sole...").

2. **A new `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` accessor returns the BOXED POLICY by reference**, exposing the policy object to ALL external callers (including the napi binding). This breaks the `EngineCapsHandle`-canonical organizing principle (V1-FROZEN-INTERFACE.md row 1 / §4.69 RESOLVED (a)).

3. **It also leaks the sealed-`CapabilityPolicy` boundary structurally**. Per CLAUDE.md baked-in #7 + V1-FROZEN-INTERFACE.md item 8 (`pub(crate) mod sealed { pub trait Sealed {} }` hard-seal landed at G-CORE-9 commit `5ce8bab6`), an external caller of `Engine::capability_policy()` gets back an `&Arc<dyn CapabilityPolicy>` whose vtable they cannot impl-against (the seal is preserved) but whose methods they CAN call. This is a NEW public surface (external callers can call `check_install_consent` / `check_per_delegation` / `check_write_with_audience` against the engine's policy outside the engine's call orchestration) — exactly the kind of "incidental affordance leak" the sealed discipline + Class B β EngineCapsHandle pattern were designed to prevent.

**The correct shape per Class B β + sealed-discipline + dep-direction:** the install-pipeline lives in `benten-platform-foundation::plugin_lifecycle` and the consent check needs to consume an `&dyn CapabilityPolicy` (or `Option<&dyn CapabilityPolicy>`) **passed as a parameter** to `install_plugin`, threaded from the caller (who already has the engine handle). Pattern parallel: `InstallPorts.cap_minter` is already passed-in; the policy reference should be a sibling port OR an `InstallParams` scalar — NOT pulled by reach-back through a new engine accessor. This preserves the `EngineCapsHandle` sole-seam invariant + matches the existing `*Ports` / `*Params` discipline.

**Recommended remediation (DISAGREE-WITH-EXPLANATION rebuttal of F-3 sub-clause):**
- Replace `Engine::capability_policy()` accessor with a new `InstallPorts.capability_policy: Option<&'a dyn CapabilityPolicy>` field (or `&'a dyn CapabilityPolicy` since dropping Options is the synthesis's S2 direction). Threads from the napi binding's caller which already owns the engine reference; no new engine public-API surface.
- This also keeps S3a inside the dep-direction discipline (foundation reads the policy; engine doesn't expose a new accessor for it).

**Disposition:** FIX-NOW per HARD RULE 12 (a) — the synthesis's S3a shape mints a new public accessor on `Engine` that violates the EngineCapsHandle-canonical invariant + over-exposes the sealed policy. The threading-via-InstallPorts shape closes the same gap at the same LOC cost.

### 1.3 §8-E hooks' "consumption-deferred-to-G-COMP-1" narrative — FIX-NOW retense

**Finding F-1.3 (MINOR).** V1-FROZEN-INTERFACE.md item 8 lines 697 + 748-750 explicitly says "Consumption-deferred to G-COMP-1 per `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-3 — zero production call sites at v1-beta". The F4 wave will close (or partial-close) Row D-3 BEFORE G-COMP-1 — this is a SCOPE-LIFT.

The synthesis treats F4 as part of post-v1-beta consumption wave but doesn't explicitly retense item 8's prose. **FIX-NOW:** commit 7's docs scope MUST include retensing V1-FROZEN-INTERFACE.md item 8 lines 697 + 748-750 from "Consumption-deferred to G-COMP-1" → "Consumption WIRED at F4 wave at PR #<N>; Row D-3 CLOSED-AT-#<N>". The freeze contract advertised post-v1-beta consumption — F4 ships it earlier, and the spec retense honors that.

### 1.4 CATALOG_VARIANT_COUNT 192 → 193 mint (F3) — VERIFY-COUPLING

**Finding F-1.4 (OBS).** The synthesis F3 = mint `PluginInstallConsentDenied` bumping CATALOG 192 → 193. Cross-wave coordination: the orchestrator's "sibling agent B is also minting `PluginUpgradeAuthorBroken` for L2-R6-MAJOR-1" was named in the brief but no evidence of that agent in the staging branch.

If both mints land in the same wave, the count goes 192 → 194 with two atomic edits at `crates/benten-errors/tests/stable_shape.rs:1019`. The §3.5g cross-language atomic-update rule means these two mints CANNOT land in separate PRs without one of them re-triggering the §3.5g audit on the other. **VERIFY-COUPLING:** confirm with sibling agent B before R5 dispatch whether the mints are atomic-in-one-PR or strictly-sequential. If sequential, the later PR rebases over the earlier's CATALOG bump.

**Disposition:** OBS — sequencing-coordination concern; no architectural change required if the orchestrator confirms strict sequential or atomic-in-batch.

### 1.5 §15.h walk_share_scope_as / D-11 sibling coupling — DISAGREE WITH SYNTHESIS OMISSION

**Finding F-1.5 (MINOR).** The synthesis bundles D-6 + D-18 into S4 (good — they're architectural siblings to the rechecker). But Row D-11 (`walk_share_scope_as` principal-bearing additive overload, anchored at `engine_share_scope.rs:46-49`) is the natural sibling to S3c's `check_write_with_audience` swap: BOTH are audience/principal-bearing additive overloads of an existing canonical method that defaults to non-audience-bearing behavior. The same `feedback_d17_same_file_sweep_recurrence` pattern applies.

Per §3.6j sweep-completeness self-verify: if F4 ships `check_write_with_audience` workspace sweep without also closing D-11, the next council round predictably surfaces "F4 swept the workspace's audience-bearing surface but missed the read-side audience-bearing overload" as a same-pattern recurrence (the read-side counterpart). **DISPOSITION:** DISAGREE-WITH-EXPLANATION + bundle D-11 with S3c sweep at marginal cost (~30 LOC additional). If orchestrator rejects the bundle, the DEFERRED.md retense at commit 7 MUST explicitly note D-11 as F4-conscious-deferral-not-oversight (HARD RULE 12 (b)).

---

## Section 2 — Sealed-discipline audit

### 2.1 New sealed `UserDidRegistry` trait in foundation — FIX-NOW DUPLICATE-TYPE-NAME

**Finding F-2.1 (BLOCKER candidate, downgraded to MAJOR after grep).** The synthesis's S4 column says "compose `PluginLibrary` × `UserDidRegistry` (new sealed trait)". This is FACTUALLY WRONG: a `UserDidRegistry` trait already exists at `crates/benten-caps/src/manifest_envelope_chain_validation.rs` (verified by grep — `benten-caps/INTERNALS.md:93` documents it as `manifest_envelope_chain_validation::{DelegationStep, ChainAnchor, ChainValidationOutcome, ManifestEnvelopeLookup, UserDidRegistry, validate_chain_with_manifest_envelope}`).

Both planners (A line 51 + A line 79 + B line 19) reference the existing trait. The synthesis row at S4 mistakenly frames it as "new sealed trait" + the synthesis's "Critical files" enumeration adds `crates/benten-platform-foundation/src/user_did_registry.rs (NEW) — sealed trait`. **Creating a SECOND `UserDidRegistry` type in `benten-platform-foundation` is a type-name collision** (parallel to the `RestrictedSpec` / `RestrictedScope` rename incident at V1-FROZEN-INTERFACE.md item 15.a "ARCHITECTURAL CONCERN — type-name collision"). The wave would import the wrong one and trait-bounds would diverge at the wire — exactly the failure mode the rename closed.

**FIX-NOW correction:** the F4 wave does NOT mint `crates/benten-platform-foundation/src/user_did_registry.rs`. It ships the *concrete impl* of the existing `benten_caps::manifest_envelope_chain_validation::UserDidRegistry` trait (e.g. `pub struct InstallRecordBackedUserDidRegistry<'a> { engine: &'a Engine } impl UserDidRegistry for InstallRecordBackedUserDidRegistry<'_> { ... }`). The trait itself stays in benten-caps where it already lives.

Also: the trait is **NOT currently sealed**. Per V1-FROZEN-INTERFACE.md item 8 ratification, the seal pattern was applied to `CapabilityPolicy`. The `UserDidRegistry` trait at HEAD is open. If the synthesis intends to seal it as part of F4, that is a NEW §1.A.FROZEN-relevant decision — adding a `Sealed` supertrait now would lock-in pre-v1-beta. The 3 existing test-double impls at:
- `crates/benten-engine/tests/g_core_8_write_boundary_user_root_chain_validator_4_23.rs:109`
- `crates/benten-caps/tests/ucan_chain_within_manifest_envelope_admitted_regression_guard.rs:54`
- `crates/benten-caps/tests/manifest_envelope_chain_validation_within_envelope_admitted.rs:58`

would need the same `__sealed_for_workspace_tests` re-export pattern. **If F4 chooses to seal: that's a 4th hook-shape change requiring its own §1.A.FROZEN ratification + V1-BETA-BREAKING-CHANGES.md row.** If F4 chooses not-to-seal: the synthesis's "sealed trait" language is just wrong and should be corrected to "trait" + the post-v1-beta sealing is a G-COMP-1 follow-up.

**Disposition:** FIX-NOW per HARD RULE 12 (a) — synthesis row must be corrected pre-dispatch. Recommend NOT sealing in F4 (sealing is a v1-FROZEN scope-add; F4 should stay scoped to consumer-wire-up).

### 2.2 EngineBuilder Path-(ii) sealed-discipline preservation — DISAGREE-WITH-EXPLANATION partial

**Finding F-2.2 (MAJOR).** The synthesis F6 = `EngineBuilder` at foundation, Path-(ii). The brief asks: "Does `EngineBuilder` (new in foundation): does it expose internal Engine machinery inappropriately? Does it preserve the sealed-CapabilityPolicy boundary?"

The existing `EngineBuilder` lives at `crates/benten-engine/src/builder.rs:154-220` (engine-side). It already has:
- `pub fn capability_policy(mut self, p: Box<dyn CapabilityPolicy>) -> Self` (line 154)
- `pub fn capability_policy_grant_backed(mut self) -> Self` (line 171)
- `pub fn capability_policy_ucan_durable(mut self) -> Self` (line 220)

So Path-(ii) "foundation owns the production builder" actually means: foundation ships a NEW `EngineBuilder`-like factory that wraps + extends the engine-side `EngineBuilder` to additionally install the `ProductionManifestEnvelopeRechecker` + `ProductionWriteBoundaryChainValidator` + `InstallRecordReplayStore` glue. The naming collision risk is real: same-name `EngineBuilder` at two crates would import-confuse.

**FIX-NOW naming:** the foundation-side builder should be `pub fn build_production_engine(...) -> Engine` OR `pub struct ProductionEngineBuilder { inner: benten_engine::EngineBuilder, ... }` — NOT `pub struct EngineBuilder` (collision). The synthesis's "Critical files" enumeration says `crates/benten-platform-foundation/src/engine_builder.rs (extend or NEW)`; if EXTEND of an existing foundation engine_builder.rs (which doesn't currently exist per grep — synthesis is wrong here), it's a fresh foundation file. **Recommend `production_engine.rs` naming** to avoid the type-name collision pattern V1-FROZEN-INTERFACE.md item 15.a flagged.

**Sealed boundary preservation:** if the foundation builder wraps the engine builder, the existing engine builder's `pub fn capability_policy(mut self, p: Box<dyn CapabilityPolicy>)` requires the caller to construct a `Box<dyn CapabilityPolicy>` — which an EXTERNAL caller cannot do post-hard-seal (the sealed `Sealed` supertrait prevents external impls). So external callers can ONLY pass a `Box<dyn CapabilityPolicy>` they got from a benten-shipped factory (e.g. `NoAuthBackend::boxed()`, `GrantBackedPolicy::boxed_with_engine_for_test_OR_legacy(...)`). This is GOOD — the sealed boundary preserves itself transitively through the builder. ✓

**Disposition:** DISAGREE-WITH-EXPLANATION on naming only (FIX-NOW for naming; sealing IS preserved).

### 2.3 `ManifestEnvelopeRechecker` + `WriteBoundaryChainValidator` traits sealed? — VERIFY

**Finding F-2.3 (MINOR).** Neither `ManifestEnvelopeRechecker` (`crates/benten-engine/src/manifest_envelope_recheck.rs:152`) nor `WriteBoundaryChainValidator` (`crates/benten-engine/src/write_boundary_chain_validator.rs:105`) is currently sealed per grep. Both are `pub trait Foo: Send + Sync { ... }` without `: Sealed` supertrait.

The synthesis ships production impls of both in `benten-platform-foundation`. If foundation's impl is the only intended impl AND the workspace test doubles are the only other impls, sealing is the right post-v1-beta direction (mirrors §8-E CapabilityPolicy seal). **But sealing is NOT in F4 scope per the synthesis's framing** (it's a consumer-wire wave, not a substrate-tighten wave).

**Disposition:** DISAGREE-WITH-EXPLANATION on framing; sealing IS the right end-state but is OUT-OF-SCOPE for F4. **BELONGS-NAMED-NOW:** add a new Row D-25 to V1-FROZEN-INTERFACE-DEFERRED.md naming "ManifestEnvelopeRechecker + WriteBoundaryChainValidator post-F4 sealing — Phase-4-Meta-Composing G-COMP-2-or-later destination" so the deferral is HARD RULE 12 (b) honest, not phantom.

### 2.4 `InstallRecordReplayCheckFn` closure-type sealing — OBS

**Finding F-2.4 (OBS).** The closure type `pub type InstallRecordReplayCheckFn = dyn FnMut(&[u8; 32]) -> Result<(), ErrorCode>` at `plugin_lifecycle.rs:722` is NOT sealable (it's a type alias for a trait object of a fn-trait, which is structurally open). When S2 drops `Option<&'a mut _>` → `&'a mut _`, the externally-constructible-FnMut shape lets external callers wire arbitrary closures into install (e.g. their own replay-store backend).

This is INTENTIONAL — the port pattern wants external testability + alternate-backend pluggability. Just want it explicit in the synthesis that S2's drop-Option DOES expose a non-sealed plug-point. The risk surface is: an external caller passes a buggy/malicious closure that returns `Ok(())` always → replay defense silently bypassed.

**Disposition:** OBS — recommend the F4 wave add a brief docstring on `InstallPorts.install_record_replay_check` (post-Option-drop) noting the trust-on-caller-honesty contract + the canonical-closure pattern `engine.install_record_replay_store().record_and_check`. ~5 LOC.

---

## Section 3 — Dep-direction audit

### 3.1 `benten-engine` → `benten-platform-foundation` dep-direction — CORRECTLY DEV-ONLY at HEAD

**Finding F-3.1 (CONFIRMED).** Verified at `crates/benten-engine/Cargo.toml:240`:
```toml
benten-platform-foundation = { path = "../benten-platform-foundation" }  # [dev-dependencies] only
```
The dep is dev-only (per the comment at `:237-239`: "production `benten-engine` does NOT depend on `benten-platform-foundation` (preserves arch-r1-1 thinness; pinned by `crates/benten-platform-foundation/tests/arch_n_benten_platform_foundation_dep_direction.rs`)"). F4's S4 + S6 production impls live in foundation. ✓ Production dep-direction preserved.

**Verify in F4:** the `arch_n_benten_platform_foundation_dep_direction.rs` pin must continue to PASS after F4 lands. Recommend the F4 implementer brief explicitly cite this pin as a required check.

### 3.2 `ProductionWriteBoundaryChainValidator` reach-back into engine — DISAGREE-WITH-EXPLANATION

**Finding F-3.2 (MAJOR).** The synthesis hybrid S1 wires `ProductionWriteBoundaryChainValidator` at TWO engine-internal sites (apply_atrium_merge per-row + delegate_capability Step 4). The validator's data dep is the **engine's** install-record store (where user-DIDs are registered). This means:
- Foundation owns the `ProductionWriteBoundaryChainValidator` impl
- Engine owns the install-record store (`crates/benten-engine/src/install_record_replay.rs`)
- Engine's `apply_atrium_merge` calls the foundation-side validator via `Arc<dyn WriteBoundaryChainValidator>` slot (already-frozen at `engine.rs:2010` setter)

This is the correct shape — the **trait** lives in engine (consumer-of-port-side), the **production impl** lives in foundation (knows-about-domain-side), and the engine's call-site uses the trait object. No dep cycle. ✓

But: the foundation impl needs to query the engine's UserDidRegistry. The cleanest pattern is `ProductionWriteBoundaryChainValidator` takes an `Arc<dyn UserDidRegistry>` in its constructor (where the registry trait already lives in `benten-caps`); the engine-side install-record-backed concrete `InstallRecordBackedUserDidRegistry` lives in **benten-engine** (close to the data); foundation's production validator is constructed with that registry passed in. This matches the existing port pattern.

**FIX-NOW:** synthesis's "Critical files" should clarify the InstallRecordBackedUserDidRegistry impl lives in **benten-engine** (not foundation), since the install-record store IS engine-side. The synthesis's `crates/benten-platform-foundation/src/user_did_registry.rs (NEW) — sealed trait` is wrong on BOTH counts (it's not a new trait per F-2.1, and the concrete-impl belongs engine-side per data-locality).

### 3.3 `ProductionManifestEnvelopeRechecker` in foundation — CORRECT

**Finding F-3.3 (CONFIRMED).** Foundation owns `PluginLibrary` already (foundation depends on engine for the trait). Production rechecker composes PluginLibrary (foundation-side) + UserDidRegistry (engine-side concrete + caps-side trait) + the chain-validator (caps-side). Foundation hosts the rechecker. ✓

### 3.4 `EngineBuilder` Path-(ii) dep cascade — VERIFIED CORRECT

**Finding F-3.4 (CONFIRMED).** Foundation already depends on engine (per existing `arch_n_benten_platform_foundation_dep_direction.rs` pin). Foundation-side production builder reads engine + composes platform-side production glue. No new dep cycle. ✓

### 3.5 `benten-sync` D-6 handshake.rs wire — VERIFY DEP-DIRECTION

**Finding F-3.5 (OBS).** Commit 6 wires `UnresolvedDeny` short-circuit at `crates/benten-sync/src/handshake.rs`. Sync depends on engine (existing). The short-circuit fires the SHARED primitive (`ManifestEnvelopeRecheckOutcome::UnresolvedDeny` + `outcome_to_row_reject`) which already lives in engine. No new dep edge. ✓

---

## Section 4 — cargo-public-api + drift-detector regression risk

### 4.1 Baseline regens required

**Finding F-4.1 (MAJOR enumeration).** Each F4 surface has a baseline impact:

| Surface | Affected baseline(s) | Regen size |
|---|---|---|
| S1 (validator already-frozen; production impl in foundation) | `docs/public-api/benten-platform-foundation.txt` (new `ProductionWriteBoundaryChainValidator` type) | ~10-30 lines |
| S2 (drop-Option on InstallPorts field) | `docs/public-api/benten-platform-foundation.txt:777` (field-type change) + `:715` (helper add if any) | ~5 lines |
| S3a (`PluginInstallConsentDenied` mint) | `docs/public-api/benten-errors.txt` (new variant in ErrorCode enum) + `docs/public-api/benten-engine.txt` (re-export propagation) | ~3-6 lines per file |
| S3a (`Engine::capability_policy()` accessor — **IF synthesis keeps this; see F-1.2 rebuttal**) | `docs/public-api/benten-engine.txt` (new pub fn) | ~3 lines |
| S3c (no API change; default delegates) | none | 0 |
| S4 (new `ProductionManifestEnvelopeRechecker` + EngineBuilder extension) | `docs/public-api/benten-platform-foundation.txt` | ~15-30 lines |

**FIX-NOW:** synthesis's "cargo-public-api regen per-PR" line in commit phasing is fine, but the synthesis should pre-name which baselines change (so the implementer doesn't have to re-discover at PR-author time). Recommend adding a "Public-API baseline regen targets" table to the triaged draft.

### 4.2 `drift-detect-error-variant-mirror.ts` §3.5g item 6 enforcement — CONFIRMED

**Finding F-4.2 (CONFIRMED).** Verified `scripts/drift-detect-error-variant-mirror.ts` exists + the baseline at `scripts/drift-detect-error-variant-mirror-baseline.txt` has 6 grandfathered DSL variants. A new `ErrorCode::PluginInstallConsentDenied` ENUM variant requires a first-class `EPluginInstallConsentDenied` TS class mirror per `feedback_pub_error_variant_first_class_mirror.md`. The synthesis's "§3.5g 8-surface mirror" obligation list at PLANNER-A Section 3 ErrorCode-mirror-obligations table correctly enumerates 8 sites; **the F4 implementer MUST exercise the drift-detect scanner pre-push** to verify no new grandfather entry is needed.

### 4.3 `npm run drift:public-api` CI lane — CONFIRMED

**Finding F-4.3 (CONFIRMED).** Per Bundle 10 Fork 3 + L17-MIN-1 closure (commit `36988d1a`), `ts-public-api` is flipped to required-failing. The F4 cargo-public-api regen must couple to the ts-public-api regen for the napi-exposed shape changes (S2's InstallPorts is NOT napi-exposed per grep, so no ts-public-api delta from S2; S3a's new ErrorCode IS TS-side via `packages/engine/src/errors.generated.ts` mirror).

**FIX-NOW:** the synthesis's commit 3 should explicitly couple `cargo run -p drift-detect-error-variant-mirror` + `npm run drift:public-api` + ERROR-CATALOG.md regen as a §3.5g atomic-update bundle.

### 4.4 `Branch Protection Spec Check` lane — VERIFIED

**Finding F-4.4 (OBS).** No new CI workflows added by F4, so the branch-protection spec stays unchanged. ✓

---

## Section 5 — Cross-doc coupling + sibling sweep (per `feedback_d17_same_file_sweep_recurrence`)

### 5.1 `Engine::capability_policy()` siblings — N/A IF REBUTTED PER F-1.2

If F-1.2's rebuttal lands (threading policy via InstallPorts instead of new engine accessor), this section moot. If F-1.2 not rebutted: sibling Engine accessors to audit are `Engine::caps()` (already-public; OK), `Engine::shares_policy_resolver` setter (already-frozen), `Engine::install_record_replay_store()` (needed for S2 closure-construction; already-public per Bundle 9 review).

**Disposition:** N/A (couples to F-1.2 outcome).

### 5.2 `UserDidRegistry` siblings — REVERSED FRAMING

Per F-2.1 the trait already exists in benten-caps and the F4 impl is in benten-engine. Siblings: `ManifestEnvelopeLookup` (already-shipped foundation-side impl per `crates/benten-caps/INTERNALS.md:104`); `SharesPolicyResolver` (already-shipped); `ManifestEnvelopeRechecker` (F4's S4). All four are the established benten-caps-trait-with-foundation-impl pattern.

**Disposition:** PATTERN-CONFIRMED; no new sibling exposure.

### 5.3 `check_write_with_audience` siblings — MAJOR sibling miss

Per F-1.5: `walk_share_scope_as` (D-11) is the read-side audience-bearing counterpart. **BELONGS-NAMED-NOW:** either bundle into F4 OR DEFERRED.md retense D-11 as F4-conscious-deferral.

**Also:** the trait `CapabilityPolicy` has NO `check_read_with_audience` companion at HEAD per grep (only `check_read` defaulted). The synthesis sweep `check_write` → `check_write_with_audience` does NOT have a matching read-side sweep candidate because the audience-aware read hook doesn't exist as a hook. **OBS:** this is a structural asymmetry that may surface at G-COMP-1 (when the audience-aware read-side discipline lands); F4 ratification of S3c forecloses the asymmetry framing.

### 5.4 D-22 `_for_test` cfg-gating sweep — COUPLING-LOW

**Finding F-5.4 (CONFIRMED LOW).** Per R4b L6-MAJOR-1 closure (commit `753d9840`), D-22 is the 115-`_for_test`-surface cascade closed via DEFER-NAMED-NOW. F4's surfaces (InstallPorts, the new production validators, EngineBuilder extension) do NOT cite `_for_test` shapes. **No coupling concern.** The synthesis omits D-22 from its `Forks remaining` list correctly.

---

## Section 6 — napi binding implications

### 6.1 S2 `InstallPorts` Option-drop napi propagation — N/A direct

**Finding F-6.1 (CONFIRMED N/A).** grep confirms `InstallPorts` is NOT directly exposed at `bindings/napi/src/lib.rs` (the napi install path constructs `InstallPorts` internally and exposes a flattened napi facade). The Option-drop is a foundation-internal shape change; napi facade unchanged. ✓

**BUT:** the napi binding currently does NOT supply `Some(closure)` for `install_record_replay_check` (per Row D-2 v1-beta posture: "ZERO production callers supply Some(closure) at HEAD"). After S2 lands as drop-Option, the napi binding MUST be updated to wire the substantive closure via `engine.install_record_replay_store().record_and_check(payload_hash)`. **FIX-NOW:** synthesis's "Critical files" enumeration omits `bindings/napi/src/lib.rs` for S2; the napi binding's install path is the production call site that must thread the substantive closure.

### 6.2 S6 `EngineBuilder` napi migration — VERIFY CARGO-PUBLIC-API delta

**Finding F-6.2 (OBS).** napi binding's `Engine` construction at `bindings/napi/src/lib.rs:290` uses the existing engine-side `EngineBuilder::capability_policy(...)` path. If F4 ships a foundation-side production builder (per F-2.2 rename to `build_production_engine` or `ProductionEngineBuilder`), the napi binding should migrate to use it — wiring the substantive `ProductionManifestEnvelopeRechecker` + `ProductionWriteBoundaryChainValidator` + replay-check closure in the napi-exposed `Engine` constructor.

**FIX-NOW:** synthesis commit 6 should explicitly include napi-binding-side wiring of the foundation builder. Otherwise the napi binding ships with the always-Noop substrates — Compromise #26 v1-beta posture stays UNCHANGED for napi-loaded engines despite F4 closing it for engine-direct callers. The synthesis's posture-update narrative would be partial.

### 6.3 S5 napi call sites for `check_write` — VERIFY-COUPLING

**Finding F-6.3 (OBS).** grep `policy.check_write\b` at `bindings/napi/src/` returns nothing — napi doesn't call `check_write` directly. ✓ S5 sweep is engine-internal-only at the napi-edge.

---

## Section 7 — Synthesis pre-decision rebuttals (F1-F8)

### F1 — hybrid S1 scope (2 sites)

**Disposition: APPROVE.** The hybrid (apply_atrium_merge + delegate_capability) covers the two trust-boundary entry points where untrusted chains arrive. engine_crud's defense via existing `CapabilityPolicy::pre_write` per Compromise #2 sub-narrative is the right v1-beta posture. The full 13-site cascade is correct end-state but the cost-benefit ratio for v1-beta favors the hybrid.

**Caveat:** PLANNER-A's S1 introduced a `WriteAdmissionFrame { chain_anchor_cid, actor_did }` shape ("avoids `CapWriteContext` `#[non_exhaustive]` cascade per D-17") — the triaged synthesis silently drops this in favor of "wire at the 2 sites directly". If the implementer ends up needing the helper for the 2-site wiring, **the helper should be `pub(crate) fn admit_write_chain` per PLANNER-A** (not exposed beyond engine) — preserves Class B β + sealed boundary.

### F2 — drop-Option on InstallPorts

**Disposition: APPROVE-WITH-FIX-NOW (per F-1.1).** Drop-Option is the right substrate shape. The 3 fix-now items (ledger row, fixture enumeration, narrative-retense) listed in F-1.1 attach to this pre-decision.

### F3 — mint `PluginInstallConsentDenied`

**Disposition: APPROVE-WITH-FIX-NOW (per F-1.2).** Mint preserves typed-discrimination invariant (matches R6-FP-A precedent). 8-surface mirror enumeration is complete per PLANNER-A's table. The FIX-NOW item is REPLACING the `Engine::capability_policy()` accessor with the threading-via-InstallPorts pattern.

### F4 — workspace sweep `check_write` → `check_write_with_audience`

**Disposition: APPROVE-WITH-FIX-NOW (verified scope).** The actual sweep targets at HEAD: 4 production sites (`engine.rs:1413`, `engine_diagnostics.rs:84`, `engine_wait.rs:899`, `primitive_host.rs:613`). The `ucan_grounded.rs:420` site is the policy's COMPOSITION-into-inner-policy and MUST NOT be swept (it's already-inside check_write_with_audience's default impl path; sweeping would either no-op or cause subtle composition shifts). **FIX-NOW:** synthesis brief MUST enumerate these 4 sites explicitly + explicitly EXCLUDE `ucan_grounded.rs:420` from the sweep.

### F5 — `audience_did = None` at non-natural-audience sites

**Disposition: APPROVE (partial-close honest).** PLANNER-B's FORK-A and PLANNER-A's Path-α agree. The DEFERRED.md retense of D-3-c must:
- Mark D-3-c as PARTIAL-CLOSE
- Explicitly carry forward "audience_did population at the 3 production sites that don't have a natural audience at v1-beta (engine_diagnostics + engine_wait + primitive_host)" — these are deferred to G-COMP-1 OR Phase-4-Meta-Composing v1-assessment-window, NOT phantom.
- **FIX-NOW DOC TARGET:** retense `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-3 sub-clause c (not just strikethrough).

### F6 — EngineBuilder at foundation (Path-(ii))

**Disposition: APPROVE-WITH-FIX-NOW (per F-2.2).** Path-(ii) is the right shape. The FIX-NOW is the NAMING: don't shadow `EngineBuilder`; use `ProductionEngineBuilder` or `build_production_engine`.

### F7 — D-6 + D-18 bundling with S4

**Disposition: APPROVE.** Architectural-sibling bundling per PLANNER-A's Section 3 sibling-sweep table. D-6 (handshake.rs §4.25) shares the UnresolvedDeny primitive; D-18 (synthesized-`node-id:` rejection) only safe-to-land once substantive rechecker exists. Bundling is correct.

### F8 — D-24 cross-wave coupling

**Disposition: DISAGREE-WITH-EXPLANATION (synthesis omission).** The triaged draft's `Forks remaining` lists F8 = "D-24 cross-wave coupling. Synthesis: include in F4". But D-24 is not enumerated in V1-FROZEN-INTERFACE-DEFERRED.md (D-15 is the highest D-row at HEAD plus D-16..D-21; D-22 is the `_for_test` cascade; D-23 doesn't exist; D-24 is also un-named). Either:
- D-24 is a TYPO for D-22 (the `_for_test` cascade — but F-5.4 confirms no coupling), or
- D-24 is a TYPO for D-20 (trybuild compile-fail backstop — couples to S4 via the sealed `UserDidRegistry`-if-sealed question per F-2.1), or
- D-24 is a planned-but-not-yet-authored row that doesn't exist in this worktree.

**FIX-NOW:** orchestrator clarify D-24's identity OR retract F8 from the synthesis. Per HARD RULE 12 (b), a phantom-destination DEFERRED row reference is exactly what §3.6j sweep-completeness self-verify is supposed to catch pre-claim.

---

## Section 8 — Compromise + breaking-change + CATALOG audit

### 8.1 SECURITY-POSTURE.md Compromise #26 narrative retense — REQUIRED

**Finding F-8.1 (MAJOR).** Compromise #26 currently narrates the v1-beta posture as "Layer-A empty/sentinel-DID short-circuit IS live; per-resolvable-DID substantive recheck is NOT live in shipped binaries". F4 closes the second half. **FIX-NOW:** commit 7's docs scope MUST retense Compromise #26 to substantive-live (mark sub-narrative CLOSED at PR #<N>; do NOT delete forensic context per pim-13 / §3.12). Synthesis correctly enumerates this.

**Sibling Compromise to verify:** Compromise #2 sync-replica sub-narrative (engine_crud relies on `CapabilityPolicy::pre_write`). The F4 hybrid-S1 framing explicitly relies on this defense. **OBS:** no Compromise #2 retense needed — F4 doesn't change engine_crud — but commit 7's narrative SHOULD cite Compromise #2 as the explicit complementary defense for the un-swept ~11 engine_crud sites.

### 8.2 V1-BETA-BREAKING-CHANGES.md additions — REQUIRED

**Finding F-8.2 (MAJOR — already in F-1.1).** Cohort 2 needs a row for InstallPorts field-type change. Cohort 3 needs a row for the new ErrorCode mint (if mint lands). Both atomic-in-commit-7.

### 8.3 DEFERRED.md row dispositions

**Finding F-8.3 (CONFIRMED enumeration).** Per synthesis:
- D-1 — TIGHTENED-NOT-CLOSED (hybrid scope partial). ✓ explicit narrative retense required.
- D-2 — CLOSED. ✓
- D-3-a — CLOSED. ✓ (assuming F-1.2 rebuttal lands)
- D-3-b — CLOSED. ✓
- D-3-c — PARTIAL-CLOSE. ✓
- D-4 — CLOSED. ✓
- D-6 — CLOSED. ✓ (sibling-bundled with S4)
- D-18 — CLOSED. ✓ (sibling-bundled with S4)
- D-11 — STAY-OPEN per F-1.5 OR BUNDLE (recommend bundle).
- D-25 (new) — open ManifestEnvelopeRechecker + WriteBoundaryChainValidator post-F4 sealing per F-2.3.
- D-22 — NO CHANGE (F-5.4 confirms no coupling).

**FIX-NOW:** add a 7th-commit checklist of "11 row dispositions" so the implementer doesn't miss D-11 + D-25.

### 8.4 CATALOG_VARIANT_COUNT trajectory

**Finding F-8.4 (per F-1.4).** 192 → 193 (if synthesis F3 mint lands as planned). Coordination with sibling agent B's `PluginUpgradeAuthorBroken` mint required pre-dispatch.

---

## Section 9 — Disposition

### Overall: **APPROVE-WITH-FIX-NOW**

The triaged synthesis is structurally sound — correct dep-direction, correct sibling-bundling (D-6 + D-18), correct hybrid S1 scope, correct PARTIAL-CLOSE posture for D-3-c. But it carries the following FIX-NOW items that MUST be addressed before R5 implementer dispatch (per HARD RULE 12 (a)):

### FIX-NOW items (must close before R5 dispatch)
1. **F-1.1** — Add V1-BETA-BREAKING-CHANGES.md Cohort 2 row for `InstallPorts` field-type change; pre-enumerate ~12-fixture cascade sites.
2. **F-1.2** — Replace `Engine::capability_policy() -> Option<&Arc<dyn CapabilityPolicy>>` accessor with `InstallPorts.capability_policy: &'a dyn CapabilityPolicy` threading per `*Ports` discipline. Preserves Class B β + sealed boundary.
3. **F-1.3** — Add V1-FROZEN-INTERFACE.md item 8 narrative retense to commit 7 (consumption-deferred → consumed at F4).
4. **F-2.1** — Correct synthesis "new sealed UserDidRegistry trait" to "concrete `InstallRecordBackedUserDidRegistry` impl of existing `benten_caps::manifest_envelope_chain_validation::UserDidRegistry` trait in benten-engine". Type-name collision risk avoided.
5. **F-2.2** — Rename foundation-side builder to `ProductionEngineBuilder` (or `build_production_engine` factory function) to avoid `EngineBuilder` type-name collision with engine-side existing builder.
6. **F-4.3** — Couple `cargo run -p drift-detect-error-variant-mirror` + `npm run drift:public-api` + ERROR-CATALOG.md regen to commit 3 §3.5g atomic-update bundle.
7. **F-6.1** — Add `bindings/napi/src/lib.rs` to "Critical files" for S2 napi install-path replay-check closure threading.
8. **F-6.2** — Add napi-binding-side `ProductionEngineBuilder` migration to commit 6 scope so napi-loaded engines also get substantive substrates (not always-Noop).
9. **F-7.F4** — Pre-enumerate S5 sweep targets (4 production sites) + explicitly EXCLUDE `ucan_grounded.rs:420` from sweep (composition-not-wrapper).
10. **F-7.F8** — Clarify F8's "D-24" identity OR retract from synthesis.
11. **F-8.3** — Add 7th-commit 11-row disposition checklist (closes D-11 narrative + adds D-25 narrative).

### DISAGREE-WITH-EXPLANATION items (reframe but don't block)
1. **F-1.5** — Bundle D-11 (walk_share_scope_as) with S3c sweep at marginal LOC cost OR explicit conscious-deferral narrative.
2. **F-2.3** — `ManifestEnvelopeRechecker` + `WriteBoundaryChainValidator` sealing is OUT-OF-SCOPE for F4 but BELONGS-NAMED-NOW as new Row D-25.
3. **F-3.2** — Move `InstallRecordBackedUserDidRegistry` concrete-impl to benten-engine (data-locality), not foundation.
4. **F-7.F8** — Per F-7.F8 above (couples to FIX-NOW).

### OBS items (no action; surfaced for context)
- F-1.4 — CATALOG mint coordination with sibling agent B.
- F-2.4 — InstallRecordReplayCheckFn type-alias trust-on-caller-honesty doc.
- F-3.5 — sync-handshake dep-direction confirmed clean.
- F-4.4 — Branch-protection spec unchanged.
- F-5.3 — `check_read_with_audience` structural asymmetry surfaces at G-COMP-1.
- F-5.4 — D-22 coupling confirmed low.
- F-6.3 — napi `check_write` direct-call sites: none.
- F-8.1 — Compromise #2 sibling-cite recommendation for commit 7.

### Pim-N candidates surfaced (≥3-recurrence threshold)
None at this lens — the findings cluster around existing pim-N rules (§3.6j sweep-completeness; §3.5g atomic mirror; `feedback_d17_same_file_sweep_recurrence`; `feedback_pub_error_variant_first_class_mirror`). The synthesis would benefit from each of those rules being cited explicitly in the F4 implementer brief as pre-flight checklist lines per §3.6g.

---

## Appendix A — File-by-file impact summary (for implementer brief)

| File | Pre-existing role | F4 delta (post-FIX-NOW) | Cargo-public-api impact |
|---|---|---|---|
| `crates/benten-engine/src/engine.rs` | apply_atrium_merge per-row | S1 wire (+S3c sweep at :1413; preserve ucan_grounded composition) | ~3-5 lines (no new pub items per F-1.2 rebuttal) |
| `crates/benten-engine/src/engine_caps.rs` | delegate_capability | S1 wire at Step 4 + S3b hook insertion at :558 (between Step 2b + Step 3) | None (internal) |
| `crates/benten-engine/src/engine_diagnostics.rs` | check_write call site | S5 sweep at :84 | None |
| `crates/benten-engine/src/engine_wait.rs` | check_write call site | S5 sweep at :899 | None |
| `crates/benten-engine/src/primitive_host.rs` | check_write call site | S5 sweep at :613 | None |
| `crates/benten-engine/src/install_record_backed_user_did_registry.rs` (NEW per F-3.2) | concrete impl of caps trait | UserDidRegistry over install-record store | New pub struct ~5 lines |
| `crates/benten-platform-foundation/src/plugin_lifecycle.rs` | install pipeline | S2 drop-Option + S3a Step 3c (via InstallPorts.capability_policy port per F-1.2) | Field type change ~3 lines + new optional `capability_policy: &'a dyn CapabilityPolicy` ~3 lines |
| `crates/benten-platform-foundation/src/production_manifest_envelope_rechecker.rs` (NEW) | S4 production impl | composes PluginLibrary + UserDidRegistry + caps validator | New pub struct ~10-15 lines |
| `crates/benten-platform-foundation/src/production_write_boundary_chain_validator.rs` (NEW) | S1 production impl | composes UserDidRegistry + caps validator | New pub struct ~10-15 lines |
| `crates/benten-platform-foundation/src/production_engine_builder.rs` (NEW per F-2.2) | Path-(ii) production builder | wraps engine EngineBuilder + wires substantive substrates | New pub struct + factory ~15-20 lines |
| `crates/benten-sync/src/handshake.rs` | sync handshake | D-6 UnresolvedDeny short-circuit wire | None (internal) |
| `bindings/napi/src/lib.rs` (per F-6.1, F-6.2) | napi facade | S2 closure threading + S6 builder migration | TS-side errors-generated regen for S3a |
| `crates/benten-errors/src/lib.rs` | ErrorCode catalog | mint PluginInstallConsentDenied (4 sites: variant + as_static_str + from_str + routed_edge_label) | +1 variant ~4 lines |
| `packages/engine/src/errors.generated.ts` | TS mirror | regen with EPluginInstallConsentDenied class + CODE_TO_CTOR | TS-side regen |
| `docs/ERROR-CATALOG.md` | catalog narrative | new entry for PluginInstallConsentDenied | ~10 lines |
| `crates/benten-errors/tests/stable_shape.rs:1019` | CATALOG_VARIANT_COUNT | 192 → 193 | N/A |
| `docs/public-api/benten-errors.txt` | baseline | regen +PluginInstallConsentDenied | ~3 lines |
| `docs/public-api/benten-platform-foundation.txt` | baseline | regen field-type + new pub structs | ~30-50 lines |
| `docs/public-api/benten-engine.txt` | baseline | re-export regen for new ErrorCode | ~3 lines |
| `docs/V1-FROZEN-INTERFACE.md` | freeze spec | item 8 retense (consumed-at-F4) | ~10 lines |
| `docs/V1-FROZEN-INTERFACE-DEFERRED.md` | deferred ledger | Row D-1 tighten + D-2 close + D-3 close + D-4 close + D-6 close + D-11 disposition + D-18 close + D-25 NEW | ~80 lines |
| `docs/V1-BETA-BREAKING-CHANGES.md` | breaking ledger | Cohort 2 row (InstallPorts) + Cohort 3 row (ErrorCode mint) | ~15 lines |
| `docs/SECURITY-POSTURE.md` | Compromise narratives | #26 substantive-live retense | ~15 lines |
| `crates/benten-caps/INTERNALS.md` | caps internals | UserDidRegistry concrete-impl cite | ~5 lines |
| `crates/benten-engine/INTERNALS.md` | engine internals | InstallRecordBackedUserDidRegistry cite | ~5 lines |
| `crates/benten-platform-foundation/INTERNALS.md` | foundation internals | ProductionEngineBuilder + ProductionManifestEnvelopeRechecker + ProductionWriteBoundaryChainValidator cite | ~15 lines |

**Net production LOC (post-FIX-NOW):** ~825-900 LOC; ~890-950 test LOC; ~3-4 new files; ~22 files touched. Single-agent dispatch remains within sweet-spot per `feedback_subtrack_sizing_heuristic`.

---

## Appendix B — Cross-references to ratified pim-N rules

- `feedback_d17_same_file_sweep_recurrence` — fires at F-1.1, F-1.5, F-5.3
- `feedback_pub_error_variant_first_class_mirror` (§3.5g item 6) — fires at F-4.2, F-4.3
- `feedback_pim_n_sweep_completeness_self_verify` (§3.6j) — fires at F-7.F4, F-8.3
- `feedback_pim_cross_language_rule_mirror` (§3.5g items 1-5) — fires at F-4.3
- `feedback_pim_n_prior_phase_explicit_preflight` (§3.6g) — recommendation to cite all 4 above in implementer brief as pre-flight checklist lines
- `feedback_agent_economics_prefer_thorough_cleanup` — supports F-1.2 rebuttal (do the threading-via-InstallPorts correctly rather than the band-aid accessor)
- `feedback_review_finding_ground_truth_verify` — F-2.1 the synthesis's "new sealed trait" framing did not survive grep-against-HEAD; the orchestrator-ground-truth-verify discipline catches it

---

## End of CRITIC-2 review
