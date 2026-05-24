# F4 Design — META-CRITIC Round 2 (convergence verifier)

> **Role:** ROUND 2 META-CRITIC for the F4 design pipeline. NOT a fresh adversarial critic — the "did synthesis v2 actually fold ROUND 1 findings cleanly" verifier per CLAUDE.md rule 9 iterate-to-convergence + memory `feedback_iterate_critical_reviews_to_convergence`.
>
> **Inputs verified:**
> - v2 synthesis: `origin/phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v2.md` @ `3f92e5ed` (172 lines)
> - v1 synthesis: `…/F4-DESIGN-TRIAGED-DRAFT-v1.md` @ `3f92e5ed` (91 lines)
> - CRITIC-1: `origin/phase-4-meta-core/r6-r1-f4-design-critic-1:.addl/phase-4-meta/F4-DESIGN-CRITIC-1-security-threat-model.md` @ `7078bf91` (344 lines)
> - CRITIC-2: `origin/phase-4-meta-core/r6-r1-f4-design-critic-2:.addl/phase-4-meta/F4-DESIGN-CRITIC-2-api-sealing-dep-direction.md` @ `a794e8b2` (422 lines)
> - PLANNER-A + PLANNER-B (staging) for F8 provenance check
>
> **Independent ground-truth verifications performed (per `feedback_review_finding_ground_truth_verify`):**
> - `git show origin/main:crates/benten-engine/src/engine_crud.rs` → confirmed `create_node`/`update_node`/`delete_node`/`create_edge`/`delete_edge` have NO `check_write` or `pre_write` call; only Inv-11 system-zone-label probe + direct `backend.transaction(|tx| tx.put_node(...))` or `backend.put_edge(...)`. **CRITIC-1 BLOCKER C1 confirmed factually correct.**
> - `git show origin/main:docs/V1-FROZEN-INTERFACE-DEFERRED.md | grep -cE '^### Row D-'` → 21 rows; max row is D-21; no D-22/D-23/D-24 entries. **CRITIC-1 HARD-ESCALATE F8 confirmed factually correct; v2 Δ11 drop is correct.**
> - `git show origin/main:crates/benten-caps/src/manifest_envelope_chain_validation.rs | grep 'pub trait'` → `pub trait UserDidRegistry` exists at line 139. **CRITIC-2 F-2.1 confirmed factually correct; v2 Δ5 reuse is correct.**
> - `git show origin/main:crates/benten-engine/src/engine.rs` lines 1395-1425 → at apply_atrium_merge :1413, `peer_actor_cid` IS available pre-check_write; `peer_did` is only resolved INSIDE the error branch (line 1417-1422). **A subtle Δ4 concern surfaces — see Section 4.**
>
> **Disposition (preview):** **ITERATE** — v2 closes 16 of 19 v1-FIX-NOWs cleanly + 1 with new concern, but the Δ4 `peer_did as audience_did` decision introduces a fresh MAJOR semantic question that needs Ben's call or synthesis v3 resolution before R5 dispatch.

---

## Section 1 — FIX-NOW closure audit

**Convention:** "Δn" cites synthesis v2's labeled deltas. "—" means "not addressed in v2 because absorbed by a coarser Δ."

### CRITIC-1 FIX-NOWs (8 total: FIX-1…FIX-8)

| # | CRITIC-1 finding | v2 Δ cite | Verdict | Notes |
|---|---|---|---|---|
| FIX-1 | BLOCKER: F1 rationale false (`pre_write` doesn't exist); engine_crud gap; pick path (a) full cascade, (b) explicit wire-up to 5 CRUD APIs, or (c) honest-disclose with DEFERRED row + retense | Δ1 (S1 scope expansion to ~350 LOC; F1 path-(a) full cascade pre-decision; alternative (c) honest-disclose path also documented for Ben-reversal) + coordination note with Agent A on F2 visibility tighten | **CLOSED-WITH-NEW-CONCERN** | v2 correctly elevates to full cascade per PLANNER-A. BUT: see Section 3 — pre-decision of F1 path-(a) without surfacing to Ben is a Ben-Fork framing question. Also see Section 4 Δ1: ~350 LOC at single-agent-dispatch is at the high edge of the sweet-spot. |
| FIX-2 | MAJOR: S3c populate `audience_did = Some(peer_did)` at apply_atrium_merge:1413 + `Some(plugin_did)` at delegate_capability; retense Compromise #26 | Δ4 (populate at 2 sites: apply_atrium_merge per-row `peer_did`; delegate_capability `plugin_did`) + DEFERRED.md retense | **CLOSED-WITH-NEW-CONCERN** | The CRITIC-1 recommendation is mechanically followed. BUT: see Section 4 — semantic question whether "peer_did" (transport principal) IS "audience_did" (cap-target principal) is a category-error question that neither CRITIC-1 nor v2 surface. |
| FIX-3 | MAJOR: napi binding must construct Engine via foundation builder; add compile/test pin | Δ6 (napi pin at `bindings/napi/tests/r6_r1_fp_c_napi_engine_routes_through_production_builder.rs`) | **CLOSED-CLEAN** | Pin name + location specified. Pattern (trybuild or runtime assert) left to implementer; both work. |
| FIX-4 | MAJOR: production-arm test pins beyond substrate; would-FAIL-if-reverted pattern; sweep-walker enumerates workspace-wide | Δ7 (`production_wiring_revert_would_fail` test pin pattern; ~150 LOC across 6 surfaces) | **CLOSED-WITH-NEW-CONCERN** | The pattern is named. BUT: see Section 4 Δ7 — testability of "production-wiring-live-vs-noop" depends on whether the Engine surfaces a `current_validator()` accessor (or similar), which the v2 doesn't enumerate. The pattern is name-coined but its concrete shape is undetermined. |
| FIX-5 | MAJOR: mint `PluginPerDelegationDenied` ErrorCode at S3b for forensic-discrimination symmetry (uniform with S3a) | Δ3 (mint `PluginPerDelegationDenied`; CATALOG 192→194 cumulative both mints; both go through §3.5g 8-surface mirror) | **CLOSED-CLEAN** | Symmetric to S3a per FIX-5 spec. CATALOG arithmetic correct (192+S3a→193, +S3b→194). |
| FIX-6 | MINOR: `install_verified_record_unchecked` gets `#[doc(hidden)]` + `#[deprecated]` + trybuild pin | Δ8 (`#[doc(hidden)]` + trybuild compile-fail) | **PARTIALLY-CLOSED** | v2 omits `#[deprecated]` annotation. The CRITIC-1 spec was `#[doc(hidden)]` + `#[deprecated(note = "internal-only; bypasses TOCTOU defense — use install_plugin")]`. v2 keeps the SAFETY doc + adds `#[doc(hidden)]` but the `#[deprecated]` annotation is dropped without explanation. **Minor but trivially fixable in implementer brief.** |
| FIX-7 | MINOR: Step 3c rustdoc + INTERNALS.md install-pipeline diagram update | Δ9 (rustdoc on Step 3c documenting ordering invariant vs `verify_user_signature`; INTERNALS.md update) | **CLOSED-CLEAN** | Specified verbatim. |
| FIX-8 | MINOR: D-18 synthesized-`node-id:` discriminator spec — named helper `fn is_synthesized_node_id` + unit tests | Δ10 (`is_synthesized_node_id` named helper + 1 unit test enumerating positive + negative) | **CLOSED-CLEAN** | Helper name + test specified per CRITIC-1 brief. Reused at handshake.rs D-6 site per spec. |

**CRITIC-1 8/8 addressed; 5 CLOSED-CLEAN + 1 PARTIALLY-CLOSED + 2 CLOSED-WITH-NEW-CONCERN.**

### CRITIC-2 FIX-NOWs (11 total: F-1.1, F-1.2, F-1.3, F-2.1, F-2.2, F-4.3, F-6.1, F-6.2, F-7.F4, F-7.F8, F-8.3)

| # | CRITIC-2 finding | v2 Δ cite | Verdict | Notes |
|---|---|---|---|---|
| F-1.1 | MAJOR: V1-BETA-BREAKING-CHANGES.md Cohort 2 row for InstallPorts field-type; pre-enumerate ~12-fixture cascade sites | — (v2 doesn't explicitly cite V1-BETA-BREAKING-CHANGES.md or fixture enumeration in the Δ list) | **NOT-CLOSED** | v2's "6 docs touched" line at row 148 doesn't enumerate V1-BETA-BREAKING-CHANGES.md by name. v1's "Critical files" listed `docs/V1-FROZEN-INTERFACE-DEFERRED.md` + `docs/SECURITY-POSTURE.md` + `docs/CLAUDE.md` + INTERNALS.md but never V1-BETA-BREAKING-CHANGES.md. **Real omission.** Must add to F4 implementer brief OR escalate. |
| F-1.2 | MAJOR (BLOCKER-candidate): replace `Engine::capability_policy()` accessor with `InstallPorts.capability_policy` port threading per `*Ports` discipline | Δ2 (threads `policy: &dyn CapabilityPolicy` via `InstallPorts` port; no new engine public-API surface) | **CLOSED-CLEAN** | The fix matches CRITIC-2's recommended remediation literally (port-threading pattern). |
| F-1.3 | MINOR: V1-FROZEN-INTERFACE.md item 8 narrative retense (Consumption-deferred-to-G-COMP-1 → consumed-at-F4) | — (v2 cites "DEFERRED.md retense" but NOT V1-FROZEN-INTERFACE.md item 8 retense) | **NOT-CLOSED** | The §1.A.FROZEN spec doc carries the "Consumption-deferred-to-G-COMP-1" prose that F4 invalidates by closing Row D-3. The spec doc must retense. **Real omission.** |
| F-2.1 | MAJOR (BLOCKER-candidate): correct "new sealed `UserDidRegistry` trait" to "concrete impl of existing `benten_caps::UserDidRegistry` trait in benten-engine" | Δ5 (reuse existing `benten-caps::UserDidRegistry`; concrete impl `InstallRecordUserDidRegistry` lives ENGINE-side per data-locality) | **CLOSED-CLEAN** | Matches CRITIC-2's F-2.1 + F-3.2 verbatim. The "engine-side concrete impl" placement IS per F-3.2 DISAGREE-WITH-EXPLANATION (data-locality) — see Section 4 Δ5 for the dep-direction nuance. |
| F-2.2 | MAJOR: rename foundation-side builder to `ProductionEngineBuilder` to avoid `EngineBuilder` collision | Δ5 (rename to `ProductionEngineBuilder` or `build_production_engine` factory) | **CLOSED-CLEAN** | Specified verbatim. |
| F-4.3 | MAJOR: couple `drift-detect-error-variant-mirror` + `npm run drift:public-api` + ERROR-CATALOG.md regen to commit 3 as §3.5g atomic bundle | — (v2 doesn't explicitly cite drift-detect-error-variant-mirror or npm run drift:public-api in Δ list) | **NOT-CLOSED** | v2's commit-phasing section says "cargo-public-api regen per-PR" but doesn't enumerate the §3.5g coupling tools by name. The implementer must know to run BOTH drift-detect (Rust ↔ TS) AND cargo-public-api AND ERROR-CATALOG.md regen, AND the CATALOG_VARIANT_COUNT pin update. **Real omission.** |
| F-6.1 | MAJOR: add `bindings/napi/src/lib.rs` to "Critical files" for S2 napi install-path replay-check closure threading | — (v2's "Critical files" list inherited from v1 was not re-enumerated; the v2 doc focuses on deltas, not full file list) | **PARTIALLY-CLOSED** | v2 implicitly absorbs via Δ6 (napi-via-foundation-builder pin) but does NOT explicitly call out S2's napi closure-threading. The closure-threading is a different concern from the builder routing — without explicit closure-threading, the napi binding ships with an absent replay-check even though the builder is correct. **Implementer brief MUST add `bindings/napi/src/lib.rs` to S2 file list.** |
| F-6.2 | MAJOR: napi-binding-side `ProductionEngineBuilder` migration so napi-loaded engines get substantive substrates | Δ6 (napi pin asserts napi binding routes through `ProductionEngineBuilder::build()`) | **CLOSED-CLEAN** | Matches CRITIC-2 F-6.2 spec. The "pin" enforces the migration (test fails if napi binding regresses to raw `Engine::open`). |
| F-7.F4 | MAJOR: pre-enumerate S5 sweep targets (4 production sites: engine.rs:1413, engine_diagnostics.rs:84, engine_wait.rs:899, primitive_host.rs:613) + explicitly EXCLUDE `ucan_grounded.rs:420` | — (v2 inherits v1's "workspace sweep" framing; no explicit 4-site enumeration or ucan_grounded exclusion noted in the Δ section) | **NOT-CLOSED** | This is concrete, mechanical, would-prevent-implementer-confusion guidance that didn't make it into v2. Per `feedback_d17_same_file_sweep_recurrence` the enumeration BEFORE dispatch is the discipline. **Real omission with concrete recurrence risk.** |
| F-7.F8 | MAJOR: clarify F8's D-24 identity OR retract | Δ11 (DROP F8 entirely; D-22/D-23/D-24 phantom per CRITIC-1 HARD-ESCALATE) | **CLOSED-CLEAN** | v2 drops F8 explicitly + acknowledges the hallucination. Correct closure. |
| F-8.3 | MAJOR: 7th-commit checklist of 11 row dispositions (closes D-11 narrative + adds D-25 narrative) | — (v2 doesn't enumerate the 11-row disposition checklist; doesn't address CRITIC-2 F-1.5 D-11 bundle/conscious-deferral question; doesn't address CRITIC-2 F-2.3 D-25 NEW row creation) | **NOT-CLOSED** | This is THREE coupled CRITIC-2 sub-findings (F-1.5 D-11 + F-2.3 D-25 mint + F-8.3 11-row checklist) that v2 collapses by omission. D-11 (`walk_share_scope_as` read-side audience overload) is a real sibling-sweep candidate the synthesis should explicitly disposition (bundle, conscious-defer, OR DISAGREE). D-25 (post-F4 sealing of ManifestEnvelopeRechecker + WriteBoundaryChainValidator) is a HARD-RULE-12 BELONGS-NAMED-NOW row that needs to be minted in F4's doc-touch pass. **Real omission cluster.** |

**CRITIC-2 11/11 addressed-or-omitted: 5 CLOSED-CLEAN + 1 PARTIALLY-CLOSED + 5 NOT-CLOSED.**

### Tally
- **8 CRITIC-1 + 11 CRITIC-2 = 19 total FIX-NOWs**
- CLOSED-CLEAN: **10**
- CLOSED-WITH-NEW-CONCERN: **2**
- PARTIALLY-CLOSED: **2**
- NOT-CLOSED: **5**
- ALTERNATIVE-DISPOSITION: **0**

The 5 NOT-CLOSED items (F-1.1, F-1.3, F-4.3, F-7.F4, F-8.3) plus 2 PARTIALLY-CLOSED (FIX-6, F-6.1) constitute the ITERATE bar.

---

## Section 2 — BLOCKER + HARD-ESCALATE disposition

### CRITIC-1 BLOCKER C1 (napi put_node + engine_crud bypass composition)

**v2 mitigation:** Δ1 + Δ1's Agent-A coordination note.

**Analysis:**

The BLOCKER decomposes into two coupled sub-vulnerabilities:
- **(C1-a)** `Engine::create_node` / `update_node` / `delete_node` / `create_edge` / `delete_edge` — direct public Rust API. Bypasses BOTH the proposed S1 validator AND the existing `CapabilityPolicy::check_write` hook (which is never called from engine_crud).
- **(C1-b)** napi `put_node` mirror — same problem one layer up.

**v2's two-pronged closure:**
1. **PLANNER-A full cascade** (Δ1: ~350 LOC across ~13 sites) wires the validator at every WRITE entry point including engine_crud. This closes (C1-a) structurally.
2. **Agent A's `pub` → `pub(crate)` visibility tighten** on `Engine::put_node` (per the §8-A V1-FROZEN-INTERFACE work). After Agent A lands, napi `put_node` no longer has a direct public engine method to forward to; it MUST route through a sealed seam (which, per v2 Δ1, the seam IS the `admit_write_chain` flow).

**Verdict: CLOSED-WITH-RESIDUAL-COORDINATION-RISK.**

The architectural closure is correct. The residual:
- v2 assumes Agent A (R6FP-A) lands FIRST. If Agent A's PR slips OR is held for separate ratification, F4's implementer is blocked OR is forced to land against an untightened surface (then re-rebase). The coordination note at v2:170 names this explicitly, which is good — but it's a sequencing risk Ben should be aware of.
- v2 says "After F2 lands, napi `put_node` no longer exists as public surface. The C1 BLOCKER is PARTIALLY closed by F2 — napi must route writes through a different mechanism (likely a sealed write-port that itself calls `admit_write_chain`)." The word "likely" is hedged — the actual napi rewiring mechanism is unspecified. **Sub-concern:** the F4 implementer brief MUST coordinate with whoever is doing the napi `put_node` rewiring (presumably also part of Agent A's scope; needs confirmation).

**Disposition: CLOSED-WITH-RESIDUAL-COORDINATION-RISK** (not fully CLOSED-CLEAN). The architectural fix is sound; the sequencing + napi-rewiring-mechanism are residual concerns Ben should know about.

### CRITIC-1 HARD-ESCALATE F8 (D-22/D-23/D-24 phantom destinations)

**v2 disposition:** Δ11 (DROP F8 entirely).

**Analysis:**

I independently verified via `git show origin/main:docs/V1-FROZEN-INTERFACE-DEFERRED.md | grep -cE '^### Row D-'` → 21 rows max; no D-22/D-23/D-24. **CRITIC-1 was factually correct.**

v2's Δ11 narrative: "D-23 + D-24 references appear to have been hallucinated by my v1 synthesis (citing CRITIC-1's L6 lens-narrative reference that I evidently misattributed). v2 drops F8."

This is the correct HARD RULE 12 (b) disposition: phantom destinations cannot ride; the rule requires BELONGS-NAMED-NOW destinations to receive the entry NOW. Since D-22/D-23/D-24 don't exist, the only valid dispositions are MINT-NOW or DROP. v2 chose DROP.

**Sub-concern:** PLANNER-A's "FORK D" at line 145 references D-22/D-23/D-24 as "include D-24 (coupled to S4); D-22 + D-23 separate" — so the hallucination originated at PLANNER-A (not v1 synthesis as v2's narrative implies). PLANNER-A's FORK D was a real fork-shape that v1 synthesis attempted to disposition. The DROP is correct but the v2 narrative miscredits where the phantom originated. **Cosmetic** — does not affect closure.

**Disposition: CLOSED-CLEAN.** The drop is correct; the phantom is verified phantom; HARD RULE 12 is honored.

---

## Section 3 — Ben-Fork F1 framing audit

v2 pre-decides F1 path-(a) full cascade per night-shift stance + `feedback_surface_arch_decisions_under_auth`. Audit per the brief's 4 sub-questions:

### 3.1 Is the framing of the F1 fork (a vs b) accurate?

**Yes, but the v2 narrative collapses CRITIC-1's THREE-option fork into a TWO-option fork.** CRITIC-1 FIX-1 enumerated THREE options:
- (a) PLANNER-A full ~13-site cascade
- (b) Hybrid + explicit `check_write` wire-up to the 5 engine_crud CRUD APIs
- (c) Honest-disclose the gap: new DEFERRED row + retense Compromise #26 + retense Row D-1 + retense CLAUDE.md #18

v2's pre-decision frames it as "(a) full cascade vs (b) honest-disclose" — collapsing CRITIC-1's (b) hybrid+CRUD-wireup into (a)'s envelope or into (c)'s envelope. **The middle path (b: hybrid + CRUD-only wire-up) is a real Pareto point CRITIC-1 explicitly named** — wiring the validator at the 2 untrusted-chain sites (apply_atrium_merge + delegate_capability) AND wiring `check_write` at the 5 engine_crud CRUD APIs gives engine_crud the EXISTING `CapabilityPolicy::check_write` defense (closing the silent-bypass) WITHOUT requiring chain-anchor threading through engine_crud (which is what makes PLANNER-A's full cascade expensive). LOC-wise option (b) ≈ ~250 LOC (hybrid 180 + 5×CRUD≈70), midway between (a)'s ~350 and the original hybrid's ~180.

**Sub-concern:** v2's framing forecloses option (b) without acknowledging it. Ben may prefer (b) over (a) — it closes the silent-bypass at lower scope while preserving the chain-anchor architectural-purist endgame for G-COMP-1.

### 3.2 Is the predicted Ben answer well-grounded vs orchestrator-overproduction?

**Mixed.** The "do it right not fast" + "F4 reversal pattern" rationale is well-grounded (HARD RULE 12 a + night-shift stance + bias-to-continue), and PLANNER-A's full cascade IS the architecturally-right end-state. However:
- Ben has explicitly ratified "narrower iteration where threat-model-right answer is narrower" in Phase-2b precedent (Compromise #2 sub-narrative; sec-r4r1-2 BLOCKER closure structural-always-on per-row cap-recheck inside `apply_atrium_merge` was Option (a), not Option full-cascade-everywhere).
- The ~350 LOC scope expansion approaches sweet-spot ceiling (`feedback_subtrack_sizing_heuristic` 800-LOC; Phase-2b precedent absorbed up to ~1500-2000 LOC on well-scoped single-agent). v2's note at line 150 acknowledges this. Not a hard veto; worth Ben's eyes.

### 3.3 Does the pre-decision foreclose other reasonable paths Ben might prefer?

**Yes — option (b) hybrid+CRUD-wireup as noted in 3.1.** Also, v2's alternative narrative ("If Ben picks F1 path-(b) honest-disclose: mint Row D-25, retense Compromise #26 substrate-only narrative, retense CLAUDE.md #18 Layer-1 with 'structurally enforced at sync-merge + delegation paths only at v1-beta', delete `pre_write` reference from Row D-1") is a HONEST-DISCLOSE-ONLY path — it does NOT include the (b) middle option of "add `check_write` to engine_crud + keep validator hybrid" which would close the silent-bypass at low cost.

### 3.4 Should F1 ACTUALLY surface to Ben or is night-shift autonomy appropriate?

**MUST SURFACE.** This is a real architectural fork with three legitimate paths and a security-honesty contract on the table (CLAUDE.md #18 Layer-1 + Compromise #26 + V1-BETA-BREAKING-CHANGES.md). Per `feedback_surface_arch_decisions_under_auth`:
> "Auto Mode / 'whatever you see fit' doesn't extend to substituting orchestrator judgment for Ben's on real architectural forks."

The F1 fork qualifies because:
- It affects CLAUDE.md #18 baked-in narrative (Layer-1 "user-as-root" structural enforcement guarantee).
- It affects Compromise #26 v1-beta posture.
- It affects scope (~180 vs ~250 vs ~350 LOC).
- It has 3 legitimate paths, not 2.
- It is rebuttable per v2's own framing.

**Recommendation:** v2's F1 pre-decision is honest about being a pre-decision-AS-BEN; the F4 implementer dispatch must NOT proceed until Ben ratifies. If Ben endorses path-(a): proceed per v2. If Ben prefers path-(b) hybrid+CRUD-wireup: synthesis v3 needed. If Ben picks path-(c) honest-disclose: synthesis v3 retense per v2's alternative narrative.

---

## Section 4 — NEW substantive concerns introduced by v2

### Δ1 (~350 LOC for S1)

**Concern (MAJOR):** ~350 LOC S1 + the test corpus expansion (Δ7 ~150 LOC) + the broader S2/S3a/S3b/S3c/S4 scope = ~1105 production LOC + ~1040 test LOC ≈ ~2145 total in single-agent dispatch.

Per `feedback_subtrack_sizing_heuristic` the sweet-spot is ~800 LOC, with extension to ~1500-2000 LOC on well-scoped single-agent dispatches per Phase-2b precedent. v2 itself acknowledges this at line 150 ("Still within single-agent sweet-spot at ~1100 production LOC... the planning-pipeline-pre-validated path is wider").

**But:** ~2145 LOC IS at the edge. The 7-commit cadence + per-commit mini-review may be insufficient cadence; consider splitting into a 2-agent SEQUENTIAL dispatch where Agent F4a does S1 + S2 (the heaviest) and Agent F4b does S3a + S3b + S3c + S4 + D-6 + D-18 after F4a merges.

**Disposition:** OBS-leaning-MAJOR — Ben judgment call on splitting; doesn't BLOCK convergence but the implementer brief should explicitly note "if scope exceeds ~1500 LOC mid-implementation, the implementer is authorized to HARD-ESCALATE for a split."

### Δ2 (InstallPorts.policy port)

**Concern (OBS):** Threading `policy` via `InstallPorts` is the correct sealing-preserving pattern per CRITIC-2 F-1.2. BUT the `InstallPorts` struct already gets a field-type change from Δ S2 (drop-Option on `install_record_replay_check`). Adding a NEW field (`policy: &'a dyn CapabilityPolicy`) on top of the field-type change is TWO breaking changes to `InstallPorts` in one commit.

cargo-public-api delta for `InstallPorts` is now: (1) `install_record_replay_check` field-type change + (2) new `policy` field add. Both need V1-BETA-BREAKING-CHANGES.md Cohort 2 entries. CRITIC-2 F-1.1 already flags the cohort-row need; v2 still doesn't add V1-BETA-BREAKING-CHANGES.md to its doc-touch list (per Section 1 F-1.1 NOT-CLOSED).

**Disposition:** OBS — already captured by F-1.1 NOT-CLOSED; flag this Δ2 amplifies the omission.

### Δ3 (Second ErrorCode mint with Agent B coordination)

**Concern (MAJOR coordination):** v2 line 172: "F4 implementer MUST coordinate ErrorCode CATALOG bump with Agent B's parallel mint (`PluginUpgradeAuthorBroken`). If both land at the same CATALOG number, one must rebase. Recommended: F4 starts at CATALOG = post-Agent-B-merge."

This is a real cross-agent coordination concern. CRITIC-2 F-1.4 flagged it as OBS. v2 elevates it to "F4 starts at CATALOG = post-Agent-B-merge" — which IS a stronger constraint than F-1.4 envisioned. The orchestrator must explicitly sequence: Agent A → Agent B → F4 (3-stage sequential, not parallel).

**Disposition:** MAJOR — Δ3 plus the Δ1 Agent A coordination = THREE-stage sequential dispatch (Agent A R6FP-A → Agent B R6FP-B → F4). The orchestrator must surface this sequencing to Ben + the dispatch plan must reflect it.

### Δ4 (`audience_did = Some(peer_did)` at apply_atrium_merge:1413 + `Some(plugin_did)` at delegate_capability)

**Concern (MAJOR semantic):** **Is "peer_did" semantically equivalent to "audience_did"?**

Independent verification at `crates/benten-engine/src/engine.rs` lines 1395-1425:
- The check_write is built at line 1402 from `peer_actor_cid` (not `peer_did`). `peer_did` is only resolved on the error path (lines 1417-1422) by calling `atrium.resolve_peer_dids(&seed.peer_node_ids)`.
- Semantically, `peer_did` = the DID of the sync-merge transport principal (the OTHER peer presenting rows for merge).
- `audience_did` per `policy.rs:485-503` doc = "the principal the cap is being EXERCISED FOR / the policy enforces denial constraints against."

**These are different concepts.** In the apply_atrium_merge per-row context:
- The peer DID is the TRANSPORT principal — who's presenting the bytes.
- The cap's intended audience is the principal the cap was MINTED for — typically `actor_cid` resolved to a DID, which is what should populate `audience_did`.

Populating `audience_did = Some(peer_did)` treats the peer-as-presenter as the audience-of-cap, which is a **category error**. A custom audience-aware policy expecting to gate "this cap is being exercised for plugin-X-DID" will get peer-DID instead. **It will silently match the wrong subject.**

The correct natural-audience at apply_atrium_merge for the audience-aware seam is `peer_actor_cid`-resolved-to-DID (the principal of the WRITER on the other peer, not the transport-peer itself; in many cases these match, but in shared-account / device-attestation scenarios they don't).

For delegate_capability the `plugin_did` IS the audience (the target of the delegation) — that one is correct.

**This is a Δ4-introduced concern that didn't exist in v1 (because v1 left `audience_did = None` everywhere) and CRITIC-1 didn't catch.**

**Disposition:** MAJOR — synthesis v3 OR the F4 implementer brief MUST disambiguate:
- (a) populate `audience_did = Some(<resolve(peer_actor_cid)>)` at apply_atrium_merge (semantically correct; adds a DID-resolution call), OR
- (b) populate `audience_did = Some(peer_did)` and explicitly DOCUMENT the conflation as v1-beta-acceptable (with DEFERRED row narrative), OR
- (c) leave `audience_did = None` at apply_atrium_merge and only populate at delegate_capability where the semantics are unambiguous.

CRITIC-1 FIX-2 said "the peer_did string is already computed at line 1416" — but verification shows it's computed INSIDE the error branch, not pre-check. CRITIC-1's verification was loose; v2 mechanically followed CRITIC-1 without re-verifying.

### Δ5 (reusing benten-caps::UserDidRegistry + engine-side concrete impl)

**Concern (MINOR dep-direction):** v2 puts the concrete impl `InstallRecordUserDidRegistry` ENGINE-side (in `crates/benten-engine/src/install_record_replay.rs`). The trait lives in `benten-caps`. The S4 production `ProductionManifestEnvelopeRechecker` (foundation-side) consumes `Arc<dyn UserDidRegistry>` — meaning it receives the engine-side concrete via the trait at runtime.

CRITIC-2 F-3.2 endorses this shape: "the foundation impl needs to query the engine's UserDidRegistry. The cleanest pattern is `ProductionWriteBoundaryChainValidator` takes an `Arc<dyn UserDidRegistry>` in its constructor (where the registry trait already lives in `benten-caps`); the engine-side install-record-backed concrete `InstallRecordBackedUserDidRegistry` lives in **benten-engine** (close to the data); foundation's production validator is constructed with that registry passed in. This matches the existing port pattern."

**This is correct.** But note: foundation-side ProductionManifestEnvelopeRechecker and ProductionWriteBoundaryChainValidator are constructed by `ProductionEngineBuilder` (foundation), which is the natural place for the engine-side concrete `InstallRecordUserDidRegistry` to be instantiated and injected. The `ProductionEngineBuilder::build()` flow: construct Engine → construct InstallRecordUserDidRegistry (engine-side concrete) over the engine's install-record store → wrap as `Arc<dyn UserDidRegistry>` → inject into ProductionManifestEnvelopeRechecker + ProductionWriteBoundaryChainValidator → install both on the Engine via setters.

**Disposition:** MINOR — the dep-direction shape is correct (engine doesn't reach back into foundation); CRITIC-2's F-3.2 endorsement stands. The MINOR concern is whether `Arc<dyn UserDidRegistry>` is appropriate for an engine-side type that the engine itself owns the data for (the install-record store lives in `Engine`). Could it be `&'engine dyn UserDidRegistry`? Probably — but `Arc` is the established pattern for the other 3 trait-objects on the Engine (ManifestEnvelopeRechecker, WriteBoundaryChainValidator already Arc-wrapped). Consistency wins. NO ACTION.

### Δ7 (`production_wiring_revert_would_fail` test pin pattern)

**Concern (MAJOR testability):** The proposed pattern "explicitly constructs an Engine via `ProductionEngineBuilder::build()` then attempts the attack scenario; assertion that succeeds proves production wiring is live (not the substrate)" — this works IFF the attack scenario can be triggered through public API + the production-wired behavior is observably different from substrate-wired behavior.

For S1 (validator): production-wired validator rejects unknown-root chains with a typed error; substrate (Noop) admits. Observable difference: ErrorCode::WriteBoundaryChainValidatorRejected vs Ok. Testable. ✓

For S3a (check_install_consent): production-wired calls custom policy's check_install_consent; substrate (NoAuth) returns Ok(()). To observe difference, test must install a custom policy that returns `Err(...)` then attempt install. The test's setup itself proves wiring. Testable. ✓

For S4 (ProductionManifestEnvelopeRechecker): production-wired rejects rows whose chain falls outside known manifest envelope; Noop admits all. Testable IF the test can construct a "row whose chain falls outside known manifest envelope" — needs a UserDidRegistry that knows DID-X + a peer presenting a row signed by DID-Y. Setup is non-trivial but tractable.

For S3c (`check_write_with_audience`): per Section 4 Δ4, the audience-aware behavior depends on the policy IMPL paying attention to `audience_did`. The default trait impl ignores `audience_did` and delegates to `check_write`. So `production_wiring_revert_would_fail` test for S3c MUST install a custom policy that overrides `check_write_with_audience` and returns different verdicts based on `audience_did`. The substrate (trait-default `check_write_with_audience = check_write`) would silently match. So this test verifies the SWAP went through but NOT (per Δ4 concern) whether `audience_did` is semantically right.

**Disposition:** MAJOR — the pattern is valid for 5 of 6 surfaces; for S3c it's necessary-but-not-sufficient (per Δ4 concern). The implementer brief must spell out the S3c test pin shape explicitly OR accept the limitation.

### Δ6 + Δ8 + Δ9 + Δ10

- Δ6 (napi pin): CLEAN.
- Δ8 (`#[doc(hidden)]` + trybuild): per FIX-6 PARTIALLY-CLOSED (missing `#[deprecated]`).
- Δ9 (Step 3c rustdoc): CLEAN.
- Δ10 (is_synthesized_node_id helper): CLEAN.

### Δ11 (DROP F8)

CLEAN per Section 2.

### Δ12 (out-of-scope assignment to Agent B)

**Concern (OBS):** v2 line 130 names Agent B's scope as "L2-R6-MAJOR-1 (T10 caller wire-in) + L2-R6-MAJOR-2 (PQ-hybrid 3-site wiring)". The PQ-hybrid wiring is a substantial decision (v1-beta default per CLAUDE.md #5 reframe). If Agent B HARD-ESCALATES on the PQ-hybrid decision, F4 is unaffected per v2's framing. ✓ Correct.

**But:** v2 says "F4 implementer must NOT touch these to avoid merge conflicts with Agent B" — yet the F4's S4 production rechecker uses `validate_chain_with_manifest_envelope` from `benten-caps`, which internally calls signature-verify code. If Agent B retenses `verify_peer_signature` to PQ-hybrid mid-F4-flight, F4's tests against the substantive recipe MAY need rebase. This is a coordination risk noted but not architectural.

**Disposition:** OBS — coordination call between Agent B + F4 timing is the orchestrator's responsibility; doesn't BLOCK convergence.

---

## Section 5 — Cross-critic disagreements

**Searched for substantive disagreements where CRITIC-1 and CRITIC-2 took different positions on the same synthesis point.**

### Disagreement 1 — F1 S1 scope

- **CRITIC-1 F1 disposition:** SUBSTANTIVE REBUTTAL — rationale architecturally false; either flip to PLANNER-A full cascade OR honestly disclose the gap (FIX-1). The synthesis pre-decision rests on a false premise. → FIX-NOW; surface to Ben as a real fork.
- **CRITIC-2 F1 disposition:** APPROVE the hybrid. "The hybrid (apply_atrium_merge + delegate_capability) covers the two trust-boundary entry points where untrusted chains arrive. engine_crud's defense via existing `CapabilityPolicy::pre_write` per Compromise #2 sub-narrative is the right v1-beta posture." → ACCEPT.

**CRITIC-2 echoed the same false premise CRITIC-1 caught.** CRITIC-2's grep coverage missed the engine_crud `pre_write` non-existence claim — its lens (API/sealing/dep-direction) didn't go to ground-truth on the security claim, and it took the synthesis's "Compromise #2 sub-narrative" reference at face value. CRITIC-1's lens (security/threat-model) caught the false premise.

**Resolution in v2:** v2 took CRITIC-1's view (Δ1 path-(a) full cascade). The cross-critic disagreement WAS resolved by picking the security-lens-correct view over the API-lens-passive view. **This is the right resolution.**

**Meta-observation:** The disagreement is itself a confirmation that the F1 fork is real (one critic accepted, one rebutted). Reinforces Section 3 recommendation: F1 MUST surface to Ben.

### Disagreement 2 — F-2.1 trait location

- **CRITIC-2 F-2.1:** trait exists in benten-caps; concrete impl belongs **engine-side** per data-locality (the install-record store IS engine-side). Foundation hosts the rechecker but consumes via trait.
- **CRITIC-1 implicit:** CRITIC-1 doesn't directly disposition trait-location (its lens is security/threat-model, not dep-direction). The synthesis v1 said "user_did_registry.rs (NEW) in foundation". CRITIC-1 didn't object — implicit ACCEPT.

**Resolution in v2:** v2 took CRITIC-2's view (Δ5 engine-side concrete impl). Resolved consistently.

### Disagreement 3 — S3c audience_did population

- **CRITIC-1 FIX-2:** populate `Some(peer_did)` at apply_atrium_merge + `Some(plugin_did)` at delegate_capability + new S3b call site.
- **CRITIC-2 F-5.3:** mostly silent on S3c population; F-5.3 only notes the `check_read_with_audience` absence as structural asymmetry. CRITIC-2's F-7.F5 disposition: "APPROVE (partial-close honest)" — accepts `None` everywhere with DEFERRED narrative.

**These are different positions.** CRITIC-1 says populate at the 2 sites; CRITIC-2 says leave `None` everywhere is honest. **v2 took CRITIC-1's view (Δ4).**

**Meta-observation:** Per Section 4 Δ4 concern, CRITIC-1's view introduces a semantic-correctness concern about whether `peer_did` IS the right `audience_did`. CRITIC-2's "None everywhere is honest" view would have AVOIDED the Δ4 concern but at the cost of S3c being a swap-that-pretends-to-wire. v2 picked the activist position; needs Δ4 disambiguation.

### Total cross-critic disagreement count: 3 (1 fully resolved by v2; 1 fully resolved; 1 resolved with new concern per Δ4).

---

## Section 6 — Convergence disposition

Per CLAUDE.md rule 9 iterate-to-convergence discipline + strict Q5 termination ("only BLOCKER/MAJOR block; full round = 0 substantive findings to advance"):

### Substantive findings tally for v2

**NEW substantive concerns (MAJOR or higher) introduced by v2:**
1. **Δ4 — `peer_did as audience_did` semantic question (MAJOR).** Category-error risk; needs disambiguation in v3 or Ben call.
2. **Δ1 + Δ3 sequencing — three-stage sequential dispatch (Agent A → Agent B → F4) (MAJOR).** Coordination risk; the dispatch plan must reflect this; Ben should know about the longer wall-clock.
3. **Δ7 S3c testability gap (MAJOR).** The `production_wiring_revert_would_fail` pattern is necessary-but-not-sufficient for S3c; brief must spell out S3c test pin shape.

**NOT-CLOSED FIX-NOWs from ROUND 1 (CRITIC-2):**
4. **F-1.1 (MAJOR) — V1-BETA-BREAKING-CHANGES.md Cohort 2 row + fixture enumeration omitted.**
5. **F-1.3 (MINOR; aggregates to MAJOR with omission cluster) — V1-FROZEN-INTERFACE.md item 8 retense omitted.**
6. **F-4.3 (MAJOR) — drift-detect-error-variant-mirror + npm run drift:public-api + ERROR-CATALOG.md regen coupling omitted.**
7. **F-7.F4 (MAJOR) — S5 4-site enumeration + ucan_grounded:420 exclusion omitted.**
8. **F-8.3 (MAJOR) — 11-row disposition checklist (D-11 bundle/defer; D-25 mint) omitted.**

**PARTIALLY-CLOSED FIX-NOWs:**
9. **FIX-6 — `#[deprecated]` annotation dropped from #[doc(hidden)] + trybuild (MINOR; trivially fixable in implementer brief).**
10. **F-6.1 — napi S2 closure-threading not explicitly enumerated (MAJOR; real omission requiring implementer-brief addition).**

**CLOSED-WITH-RESIDUAL:**
11. **FIX-1 / BLOCKER C1 — closed architecturally but with Agent-A coordination + napi-rewiring-mechanism residual.**

### Total substantive MAJOR+ findings: **7-8** (depending how you count F-1.3 + F-8.3 cluster).

### Convergence verdict: **ITERATE.**

Per strict Q5: a substantive-finding-bearing round does NOT converge. v2 closed 10 of 19 v1-FIX-NOWs CLEAN, but the omission cluster (5 NOT-CLOSED + 2 PARTIALLY-CLOSED) plus 3 NEW substantive concerns means **synthesis v3 is required.** The omission cluster could plausibly be absorbed into the F4 implementer brief (since they're concrete-mechanical doc-touch + enumeration items), BUT the convergence discipline + the Δ4 semantic question + the F1 Ben-fork-surfacing requirement mean v3 should land before R5 dispatch.

**Two paths to convergence-from-here:**

**Path A (recommended): synthesis v3.** Author a small v3 (single orchestrator-direct pass; not a fresh planner agent) that:
- Resolves Δ4 audience_did semantics (pick option a / b / c per Section 4 Δ4) OR explicitly surfaces as F1.5 sub-fork to Ben alongside F1.
- Adds V1-BETA-BREAKING-CHANGES.md to doc-touch list (F-1.1).
- Adds V1-FROZEN-INTERFACE.md item 8 retense (F-1.3).
- Enumerates the §3.5g atomic-update bundle by tool name (F-4.3).
- Enumerates S5 4 sweep targets + ucan_grounded:420 exclusion (F-7.F4).
- Adds 11-row disposition checklist including D-11 disposition + D-25 mint (F-8.3).
- Adds `#[deprecated]` annotation to FIX-6 (trivial).
- Adds napi S2 closure-threading to S2's critical-files (F-6.1).
- Spells out S3c production_wiring_revert_would_fail test shape (Section 4 Δ7).
- Adds 3-stage sequential dispatch sequencing note (Agent A → Agent B → F4).
- Re-surfaces F1 as a TRUE 3-option fork (path a vs b vs c, not 2) to Ben.

ETA: ~30-45 min orchestrator-direct work; no agent dispatch needed. v3 → meta-critic-round-3 (light verifier) → CONVERGED → F4 implementer dispatch.

**Path B: skip v3; absorb omissions into F4 implementer brief directly.** Risk: the implementer brief becomes a multi-page wall-of-spec that defeats the synthesis pipeline's purpose (the brief is meant to point at a settled design, not BE the settled design). Also: F1 is a real Ben fork that cannot be sneaked into an implementer brief.

**Recommended: Path A.**

---

## Section 7 — F4 implementer brief readiness checklist

**Per the brief's contract: IF you recommend CONVERGED, list what the implementer brief needs. Since I recommend ITERATE, the checklist below is for the POST-v3 implementer brief — to be authored after v3 lands.**

### F4 implementer brief contract (POST-v3, draft):

- **Branch name:** `phase-4-meta-core/r6-r1-f4-impl` (or per orchestrator convention).
- **Read-list (implementer must consume):**
  - `.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v3.md` (TO BE AUTHORED)
  - `.addl/phase-4-meta/F4-DESIGN-CRITIC-1-security-threat-model.md`
  - `.addl/phase-4-meta/F4-DESIGN-CRITIC-2-api-sealing-dep-direction.md`
  - `.addl/phase-4-meta/F4-DESIGN-META-CRITIC-ROUND-2.md` (this doc)
  - `docs/V1-FROZEN-INTERFACE.md` (item 8 narrative)
  - `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (rows D-1 through D-21; the 11 row dispositions)
  - `docs/V1-BETA-BREAKING-CHANGES.md` (Cohort 2 + Cohort 3)
  - `docs/SECURITY-POSTURE.md` (Compromise #2 + Compromise #26)
  - `docs/CLAUDE.md` (baked-in #18)
  - `.addl/dispatch-conventions.md` (§3.5 + §3.5g + §3.5h + §3.5l + §3.6g + §3.6j + §3.12 + §3.13 + §3.14)
  - Cited memories: `feedback_d17_same_file_sweep_recurrence` + `feedback_pub_error_variant_first_class_mirror` + `feedback_pim_n_sweep_completeness_self_verify` + `feedback_pim_cross_language_rule_mirror` + `feedback_pim_n_prior_phase_explicit_preflight` + `feedback_agent_economics_prefer_thorough_cleanup` + `feedback_review_finding_ground_truth_verify`
- **Per-bundle scope + LOC budget:** as per v3 (estimated ~1100 production + ~1040 test = ~2145 total at v2 state).
- **Hard-escalate triggers (sub-forks implementer should surface mid-impl rather than pick):**
  - If `peer_did` semantic disambiguation in Δ4 wasn't resolved in v3 — HARD-ESCALATE to Ben.
  - If scope exceeds ~1500 LOC mid-implementation — HARD-ESCALATE for split (Section 4 Δ1).
  - If napi-rewiring-mechanism (post Agent A's `pub(crate)` visibility tighten) is unspecified — HARD-ESCALATE for Ben/orchestrator coordination (Section 2 BLOCKER C1 residual).
  - If `is_synthesized_node_id` test corpus uncovers additional synthesized-DID patterns beyond `node-id:` — HARD-ESCALATE (D-18 scope creep).
  - If cargo-public-api regen surfaces UNEXPECTED public-surface deltas beyond enumerated — HARD-ESCALATE.
- **Coordination notes:**
  - **3-stage sequential dispatch:** F4 MUST start AFTER Agent A (R6FP-A cfg+visibility) AND AFTER Agent B (R6FP-B crypto+security+wire CATALOG mint `PluginUpgradeAuthorBroken`). Order: Agent A → Agent B → F4.
  - F4 implementer MUST NOT touch `plugin_manifest.rs::verify_peer_signature`, `install_record.rs::verify_user_signature`, `module_ecosystem.rs::verify_upgrade_author_continuity` (Agent B scope).
  - F4 ErrorCode CATALOG starts at post-Agent-B-merge baseline; F4 mints two NEW codes after that baseline.
- **Pre-push gate (§3.5h MANDATORY):**
  - `cargo +1.95 clippy --workspace --all-targets -- -D warnings`
  - `cargo +stable clippy --workspace --all-targets -- -D warnings` (per §3.5j; sibling memory `feedback_pim_n_stable_clippy_gate`)
  - `cargo nextest run --workspace`
  - `cargo doc --workspace --no-deps`
  - `cargo deny check`
  - `cargo run -p drift-detect-error-variant-mirror`
  - `cargo run -p drift-detect-public-api` (or equivalent)
  - `npm run drift:errors`
  - `npm run drift:public-api`
  - `cargo run -p cite-drift` (or equivalent doc cite drift detector)
  - `jq .` on every touched JSON artifact (§3.5h amendment per R6-FP-3)
  - `cargo-public-api` baseline regen (3 baselines: errors + engine + foundation)
  - `cargo nextest run -p phase-3-workspace-tests --test missing_docs_workspace` (per `feedback_workspace_missing_docs_test_invocation`)
- **Mini-review trigger:** synchronous mini-review per group (per `feedback_synchronous_mini_review`); one mini-reviewer agent dispatched after F4 implementer commits + pushes; mini-reviewer pre-flight = §3.5i rebase-staleness check + §3.5n orchestrator-ground-truth-verify discipline.
- **Commit-before-return mandate (per `feedback_agent_output_must_commit_before_return`):** F4 implementer brief MUST mandate commit-before-return; agent worktrees auto-clean otherwise.
- **Strategy-C batch eligibility:** if F4 lands as 7 commits / 1 PR per v2 phasing, no Strategy-C needed. If 7 PRs (one per commit), Strategy-C consolidate per `feedback_strategy_c_batch_mandatory_not_optional`.

---

## End of META-CRITIC-ROUND-2

**Convergence verdict: ITERATE → synthesis v3 (orchestrator-direct ~30-45 min pass) → meta-critic-round-3 (light verifier; ~15 min) → CONVERGED → F4 implementer dispatch.**

**F1 MUST SURFACE to Ben as a 3-option fork (a / b / c) before F4 implementer dispatch — this is the binding gate per `feedback_surface_arch_decisions_under_auth`.**

**Δ4 audience_did semantic disambiguation MUST be resolved (either in v3 or as a Ben sub-fork F1.5) before F4 implementer dispatch.**
