# F4 Design — META-CRITIC ROUND 3 (LIGHT verifier)

> **Role:** Final convergence light-verifier per META-CRITIC ROUND-2 Path-A recommendation. NOT a fresh adversarial pass. Verifies v3 closes ROUND-2's 7 unclosed/partial + 3 NEW-concerns + F1-binding-gate cleanly; checks for v3-introduced concerns; reports CONVERGED / ITERATE / HARD-ESCALATE.
>
> **Inputs verified:**
> - v3 synthesis: `origin/phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v3.md` (220 lines)
> - v2 synthesis: same staging branch (172 lines; diff baseline)
> - ROUND-2: `origin/phase-4-meta-core/r6-r1-f4-design-meta-critic-round-2:.addl/phase-4-meta/F4-DESIGN-META-CRITIC-ROUND-2.md` (400 lines)
>
> **Light-touch ground-truth verifications (§3.5n):**
> - `git show origin/main:docs/V1-FROZEN-INTERFACE.md` line 681-758 → confirmed item 8 carries "Consumption-deferred to G-COMP-1 per Row D-3 — zero production call sites at v1-beta" prose that F4's S3a/S3c consumption invalidates. **Δv3-5 retense target verified real.**
> - `git show origin/main:docs/V1-FROZEN-INTERFACE-DEFERRED.md | grep '^### Row D-'` → 21 rows max at main. Agent D's branch `phase-4-meta-core/r6-r1-fp-d-docs-ledger-ci` adds **Row D-25 — V1-BETA-BREAKING-CHANGES.md ledger completion sweep**. **D-25 collision risk verified real; v3 correctly defers to D-26 for F4's potential mint.**
>
> Written 2026-05-24 PM.

---

## Section 1 — ROUND-2 unclosed/partial closure verification

### 5 NOT-CLOSED items

| # | ROUND-2 finding | v3 delta cite | Verdict | Notes |
|---|---|---|---|---|
| 1 | CRITIC-2 F-1.1 (V1-BETA-BREAKING-CHANGES row + fixture enum) | Δv3-4 | **CLOSED-CLEAN** | v3 explicitly adds: (a) `git grep -nE 'InstallPorts\s*\{'` workspace fixture enumeration; (b) Cohort 6 row in V1-BETA-BREAKING-CHANGES.md with verbatim migration text; (c) pre-push gate "ALL fixtures compile + workspace tests pass." All three sub-asks named verbatim. |
| 2 | CRITIC-2 F-1.3 (V1-FROZEN-INTERFACE.md item 8 retense) | Δv3-5 | **CLOSED-CLEAN** | v3 explicitly: (a) updates `InstallPorts.install_record_replay_check` field-type per S2; (b) adds NEW `InstallPorts.policy: &dyn CapabilityPolicy` field per Δ2 (CRITIC-2 F-1.2); (c) documents threading model "via InstallPorts port; NOT exposed via Engine accessor" preserving Class B β sealed boundary; (d) catch-all "Mark any other §1.A.FROZEN items the v3 design touches." All four asks named. Ground-truth-verified: item 8 line 681-758 carries the "Consumption-deferred to G-COMP-1" prose F4 invalidates by consuming the hooks. |
| 3 | CRITIC-2 F-4.3 (drift-detect + npm + ERROR-CATALOG coupling) | Δv3-6 | **CLOSED-CLEAN** | v3 enumerates 8 specific tool/file targets: errors lib.rs 4-internal-site sweep, stable_shape.rs variant_count_is_pinned 192→194, packages/engine/src/errors.generated.ts TS classes + CODE_TO_CTOR, ERROR-CATALOG.md, drift-detect-error-variant-mirror.ts baseline regen, `npm run drift:errors` pre-push, `docs/public-api/benten-errors.txt` baseline regen, Agent B coordination clause. Tool names + file paths verbatim. |
| 4 | CRITIC-2 F-7.F4 (S5 4-site enum + ucan_grounded exclude) | Δv3-7 | **CLOSED-CLEAN** | v3 mandates: (a) `git grep -nE 'policy\.check_write\b'` discovery; (b) 4 expected sites named (engine.rs:1413 + engine_diagnostics.rs:78 + "TWO MORE per CRITIC-2 enumeration; F4 implementer to verify via grep"); (c) EXCLUDE ucan_grounded.rs explicit; (d) commit-body listing for cargo-public-api auditability. **Sub-note (OBS, NOT a NEW concern):** the literal "TWO MORE per CRITIC-2 enumeration" leaves the implementer to re-grep; this is acceptable per F4 implementer brief discovery contract since the grep is determinate + the implementer must list the 4 sites in the commit body, but the 4 sites COULD have been enumerated verbatim in v3 to fully foreclose recurrence. Per HARD RULE 12 (a) acceptable as the grep is mechanical + the discovery-gate is named. Does NOT block convergence. |
| 5 | CRITIC-2 F-8.3 (11-row checklist + D-11 + D-25 mint) | Δv3-8 | **CLOSED-CLEAN** | v3 enumerates all 11 rows with per-row disposition: D-1 retense (close OR mint companion per F1) / D-2 CLOSE / D-3-a CLOSE / D-3-b CLOSE / D-3-c PARTIAL-CLOSE per Δv3-2 / D-4 CLOSE / D-6 CLOSE / D-11 retense / D-18 CLOSE / D-26 MINT (only if F1 path-c) / Cohort 6 entry. Plus explicit "D-25 was minted by Agent D for L18 ledger purpose — F4 implementer must NOT collide (use D-26 if needed)." **D-25 collision risk acknowledged + correctly resolved to D-26 (ground-truth-verified Agent D's branch carries Row D-25 for V1-BETA-BREAKING-CHANGES.md ledger completion sweep).** |

### 2 PARTIALLY-CLOSED items

| # | ROUND-2 finding | v3 delta cite | Verdict | Notes |
|---|---|---|---|---|
| 6 | CRITIC-1 FIX-6 (`#[deprecated]` dropped from #[doc(hidden)] + trybuild) | Δv3-9 | **CLOSED-CLEAN** | v3 restores `#[deprecated(note = "use plugin_lifecycle::install_plugin which threads InstallRecordReplayStore")]` per CRITIC-1 spec + retains `#[doc(hidden)]` + trybuild. All three guards present. |
| 7 | CRITIC-2 F-6.1 (napi S2 closure-threading file specificity) | Δv3-10 | **CLOSED-CLEAN** | v3 names `bindings/napi/src/lib.rs` lines ~337-339 explicitly + provides fallback discovery via `git grep -nE 'install_plugin' bindings/napi/src/` + spells out the migration shape (`ports.install_record_replay_check = &mut closure` where `closure = make_engine_replay_check_closure(engine.install_record_replay_store())` post-Option-drop). File + line + closure-source named. |

### 3 NEW concerns from ROUND-2

| # | ROUND-2 concern | v3 delta cite | Verdict | Notes |
|---|---|---|---|---|
| 8 | Δ4 peer_did-as-audience category-error | Δv3-2 | **CLOSED-CLEAN** | v3 takes the **conservative position** (Section 4 Δ4 option c): populate `audience_did = Some(plugin_did)` ONLY at delegate_capability (cap-target semantically correct); leave None at apply_atrium_merge (absence-of-audience IS the correct semantic for an inbound sync per-row write). Adds NO new CapWriteContext field (preserves §1.A.FROZEN item 8 strict-additivity). Tightens DEFERRED Row D-3-c narrative with semantically honest framing. **This avoids the category error CRITIC-1's mechanical FIX-2 introduced AND avoids the new-field bloat option (a) would require.** See Section 2.2 below for verification this doesn't create a NEW gap. |
| 9 | Δ1+Δ3 three-stage sequential dispatch wall-clock | "Implementation phasing" section | **CLOSED-CLEAN** | v3 explicitly enumerates 3-stage sequencing (Agent A → Agent B → F4) with ~3h × 3 = ~9h wall-clock estimate + surfaces to Ben: "may want to accept the longer wall-clock or pick a different F1 path that allows F4 to start earlier (path-(c) honest-disclose has no Agent A dependency since it doesn't touch napi binding)." The sequencing is named + Ben-surfaced + tied back to F1 path choice. |
| 10 | Δ7 S3c production_wiring_revert necessary-but-not-sufficient | Δv3-3 | **CLOSED-CLEAN** | v3 inverts the test pattern: custom CapabilityPolicy whose `check_write_with_audience` returns Ok normally + whose `check_write` PANICS. If swap is reverted (apply_atrium_merge falls back to `check_write`), panic fires + test FAILS. This is the semantically-correct shape per CRITIC-1 FIX-4 intent + closes the trait-default delegation hazard ROUND-2 surfaced. See Section 2.3 below for a sub-concern on default-impl resolution. |

### 2 binding-gate items

| # | Item | v3 delta cite | Verdict | Notes |
|---|---|---|---|---|
| 11 | F1 surfacing failure (3-option fork) | Δv3-1 + Forks table + "CRITICAL" section | **CLOSED-CLEAN** | v3 explicitly DEFERS F1 to Ben as 3-option fork. All 3 paths characterized with: (a) scope sites enumerated, (b) LOC budget (~350 / ~250 / ~80), (c) v1-beta posture claim text, (d) honesty gap closure shape, (e) cost trade-off. Predicted answer is acknowledged-rebuttable ("my prediction remains path-(a)... but META-CRITIC's path-(b) Pareto argument is substantively strong"). Forks table marks F1 **OPEN**. Final "CRITICAL: F1 binding-gate posture" section reaffirms binding-gate + default-if-no-Ben-answer = path-(b) per META-CRITIC. Per `feedback_surface_arch_decisions_under_auth` discipline satisfied. See Section 3 for full F1 framing audit. |
| 12 | D-25 collision risk with Agent D's L18 row | Δv3-8 + Coordination notes | **CLOSED-CLEAN** | v3 explicitly: (a) names Agent D's D-25 use ("ledger purpose" — verified as V1-BETA-BREAKING-CHANGES.md ledger sweep on `phase-4-meta-core/r6-r1-fp-d-docs-ledger-ci`); (b) commits F4 to D-26 if F1 path-(c) selected; (c) reaffirms in Coordination notes "F4 implementer must coordinate Row numbering in DEFERRED.md (Agent D took D-25; F4 uses D-26 only if F1 path-(c))." |

---

## Section 2 — NEW substantive concerns introduced by v3 deltas

### Section 2.1 — Δv3-1 explicit 3-option F1 enumeration

**Question:** Is the framing accurate for all 3 paths? Any path with hidden cost not noted?

**Audit:**

- **Path (a) full ~13-site cascade.** Sites enumerated verbatim (engine_crud.rs 5 sites + engine_caps + engine_views + engine_modules 2 sites + engine_diagnostics + engine_wait 2 sites + engine.rs apply_atrium_merge + handler_versions.rs). LOC ~350 matches PLANNER-A budget. Posture claim ("Layer-1 user-as-root structurally enforced at every WRITE admission") is TRUE post-impl. Cost ("widest cascade; ~13 production sites to thread WriteAdmissionFrame through; highest test-burden") is accurate. **Hidden cost: 13-site WriteAdmissionFrame threading may discover sites where the chain_anchor_cid is genuinely unavailable (engine-internal writes); v3 doesn't enumerate the "what does chain_anchor_cid = None mean at engine_internal sites" semantic.** Per ROUND-2 Section 4 Δ1 this approaches the single-agent LOC ceiling (~1075 total prod LOC) — v3 acknowledges single-agent feasibility. **MINOR sub-concern, NOT a blocker; F4 implementer hard-escalate trigger for "scope exceeds ~1500 LOC" already covers.**

- **Path (b) hybrid 5-CRUD.** Scope (5 engine_crud user-facing CRUD APIs + apply_atrium_merge + delegate_capability) named. LOC ~250 reasonable. Posture claim ("Layer-1 user-as-root structurally enforced at every user-facing WRITE + sync-merge + delegation; precise about user-facing-vs-privileged distinction") is HONEST + matches what's wired. Cost characterization recommended-by-META-CRITIC-as-Pareto-optimal stands. **Hidden cost: the user-facing-vs-privileged distinction needs to be DOCUMENTED somewhere persistent (probably Compromise #26 retense + CLAUDE.md #18 sub-clause) — v3 implies this in Δv3-8 row D-1 retense but doesn't make it explicit for path-(b). MINOR — implementer brief should make this docs-touch explicit if path-(b) ratified.**

- **Path (c) honest-disclose.** Scope ("wire at apply_atrium_merge + delegate_capability only" — v1 hybrid + just doc updates). LOC ~80 (just doc updates) reasonable. Posture claim ("structurally enforced at sync-merge + delegation paths only at v1-beta") is HONEST but narrower. Cost "lowest LOC; widest honesty gap left open; v1-beta 'Layer-1 enforced' claim becomes 'Layer-1 partially enforced'" is accurate. **Hidden cost: per ROUND-2 Section 4 Δ1 path-(c) also "has no Agent A dependency since it doesn't touch napi binding" per v3's own phasing note — this is a HIDDEN BENEFIT of path-(c) (parallel-dispatchable; saves ~6h wall-clock) that v3 partially surfaces in phasing but doesn't highlight in the F1 trade-off table.** MINOR — surfacing the parallel-dispatchability of (c) more prominently would strengthen Ben's information set. Does NOT block convergence; the phasing note carries it.

**Overall framing verdict:** ACCURATE for all 3 paths; 3 MINOR sub-concerns identified but none block convergence (all are HARD RULE 12 (a)/(b) acceptable as orchestrator-handle-at-implementer-brief-time or post-Ben-ratification refinements).

### Section 2.2 — Δv3-2 audience_did category-error fix

**Question:** Does "populate at delegate_capability only" actually close S3c's honesty gap, or does it create a NEW gap (apply_atrium_merge inbound sync writes have no audience-aware check at all now)?

**Audit:**

The v3 conservative position is:
- delegate_capability call site populates `audience_did = Some(plugin_did)` (the cap-target plugin is the natural audience)
- apply_atrium_merge call site leaves `audience_did = None`
- DEFERRED Row D-3-c narrative reframed: "audience-aware seam wired at delegate_capability per its natural audience (cap-target plugin_did); apply_atrium_merge per-row writes are inbound sync where audience-absent is the correct semantic; G-COMP-1 lifts forensic transport-principal observation if needed."

**Does this close S3c's honesty gap?**

The S3c honesty contract is: "the audience-aware policy hook (`check_write_with_audience`) is wired AT SITES WHERE A POLICY AUTHOR CAN MEANINGFULLY GATE ON AUDIENCE." Per v3 conservative position:
- delegate_capability: a policy author CAN write `check_write_with_audience(ctx, audience_did)` that meaningfully gates on `audience_did = plugin_did` (e.g., "deny delegation to plugin-X"). **Honest.**
- apply_atrium_merge: a policy author CANNOT meaningfully gate on audience for inbound sync per-row writes because there IS no semantic audience (the inbound row was authored by SOME peer for SOME purpose; the cap is being exercised "for" the row-author, but the row-author is identified by the chain validator, not by a single `audience_did` field). `audience_did = None` is the SEMANTICALLY CORRECT value because the row-author is communicated by other channels (peer_actor_cid + the chain-anchor-cid the chain validator emits). **Honest.**

**Does it create a NEW gap?**

Hypothetical: "a policy author wants to gate inbound-sync per-row writes on the peer-DID transport principal." v3 says this is forensic-logging concern + deferred to G-COMP-1 via a NEW `inbound_peer_did: Option<&str>` field (conditional on hook needing it). v3 doesn't add the field at v1-beta. **This IS a gap (the audience-aware hook receives `None` at apply_atrium_merge), but it's a CORRECTLY-NAMED gap rather than a category-error-disguised-as-a-fix.** The v2 mechanical fix (populate peer_did as audience_did) would have HIDDEN the gap behind silently-wrong audience matching; v3's None-at-apply_atrium_merge surfaces the gap explicitly.

**Per HARD RULE 12 disposition:**
- (b) BELONGS-NAMED-NOW destination: Row D-3-c retense with "G-COMP-1 lifts forensic transport-principal observation if needed." Named.
- The audience-aware-policy-author who wants peer-DID gating at v1-beta gets `None` + DEFERRED row pointing at G-COMP-1.

**Verdict: CLOSED-CLEAN.** v3's conservative position closes the honesty gap WITHOUT creating a NEW gap; the gap that remains (no audience-aware peer-DID observation at apply_atrium_merge) is correctly NAMED and HARD-RULE-12-(b)-DEFERRED. **This is the right fix.**

### Section 2.3 — Δv3-3 S3c invert-the-panic-pattern test shape

**Question:** Is the test shape semantically correct? Are there gotchas with trait-default delegation?

**Audit:**

v3's test shape: custom CapabilityPolicy where `check_write_with_audience` returns Ok normally + `check_write` PANICS. Then call apply_atrium_merge; if swap is reverted (apply_atrium_merge falls back to calling `check_write`), panic fires + test FAILS.

**Semantic correctness analysis:**

In Rust trait default-impl resolution: a custom `impl CapabilityPolicy for CustomPolicy { fn check_write(...) { panic!() } fn check_write_with_audience(...) { Ok(()) } }` explicitly overrides BOTH methods. When apply_atrium_merge calls `policy.check_write_with_audience(...)`, it dispatches to the override (returns Ok). When apply_atrium_merge calls `policy.check_write(...)` (the reverted code path), it dispatches to the override (panics). This is correct dynamic dispatch.

**Potential gotchas:**

1. **What if the production code calls `check_write` THROUGH the default-impl of some OTHER hook that delegates to check_write?** v3 doesn't enumerate this. For example, if `check_write_with_audience`'s trait-default body is `fn check_write_with_audience(&self, ctx, _audience) { self.check_write(ctx) }`, and the production code (after the proposed revert) calls `check_write_with_audience` (NOT the reverted check_write), the trait-default delegates to `self.check_write()` which panics — test FAILS (correct behavior: revert detected). However, if the CUSTOM policy overrides `check_write_with_audience` to NOT delegate to check_write (per v3's test setup), then the trait-default behavior is BYPASSED — the custom's override returns Ok, no panic. **This is exactly the case v3 prescribes; correct behavior.**

2. **What if the production code calls `check_write` directly elsewhere in the apply_atrium_merge path (e.g., a sub-helper)?** The test would falsely fail (panic from an unrelated check_write call). **Sub-concern: the test setup must ensure the panic fires ONLY when the apply_atrium_merge entry-point's swap is reverted, NOT when an unrelated check_write is called downstream.** v3 doesn't specify isolation. **MINOR — F4 implementer should verify test isolation by reading the apply_atrium_merge code path + ensuring the custom policy's `check_write` panic only triggers on the swapped entry-point.**

3. **Concurrency:** if apply_atrium_merge is called concurrently with other engine operations that invoke check_write, the test could falsely fail. **OBS — single-threaded test execution is standard for these pins; not a real concern.**

**Verdict: CLOSED-CLEAN with 1 MINOR sub-concern.** The test shape is semantically correct + correctly inverts the trait-default delegation hazard ROUND-2 surfaced. Sub-concern #2 (test isolation) is an implementer-brief-time refinement, not a blocker. **F4 implementer brief should add: "S3c production_wiring_revert_would_fail test pin must isolate the custom policy's check_write panic to the apply_atrium_merge entry-point only; verify by reading the apply_atrium_merge call graph for downstream check_write invocations that would falsely trigger panic."**

### Other v3 deltas (Δv3-4 through Δv3-10)

All quick-audited; no NEW substantive concerns beyond what's covered in Section 1 above.

---

## Section 3 — F1 framing audit

Per `feedback_surface_arch_decisions_under_auth`: v3 surfaces F1 as 3-option fork (a/b/c) with predicted (b) per META-CRITIC Pareto.

### 3.1 — Are all 3 options accurately characterized with LOC + posture + cost?

| Path | Scope | LOC | Posture text | Cost | Verdict |
|---|---|---|---|---|---|
| (a) full | ~13 sites enumerated verbatim | ~350 | TRUE post-impl | "widest cascade; highest test-burden" | ACCURATE |
| (b) hybrid 5-CRUD | 5 CRUD + apply_atrium_merge + delegate_capability, skip 6 privileged | ~250 | TRUE post-impl; precise about user-facing-vs-privileged | "lower cascade; cleaner semantic split; recommended by META-CRITIC as Pareto-optimal" | ACCURATE |
| (c) honest-disclose | v1 hybrid + doc updates only | ~80 | HONEST but narrower; "Layer-1 partially enforced" | "lowest LOC; widest honesty gap left open" | ACCURATE |

All three paths carry the four required dimensions (scope + LOC + posture + cost). **No path silently advantaged.**

### 3.2 — Is the default-if-no-Ben-answer (path-b) the right default?

**Yes.** Per META-CRITIC ROUND-2 Pareto argument: path-(b) closes the silent-bypass at lower scope while preserving the chain-anchor architectural-purist endgame for G-COMP-1. Path-(a) is the architecturally-pure endpoint but expands single-agent LOC budget near ceiling. Path-(c) is the lowest-effort but leaves the widest honesty gap (CLAUDE.md #18 Layer-1 claim becomes "partially enforced" which is a narrative-degradation that needs cross-doc retense).

Path-(b) is the Pareto-optimal default because:
- Closes the silent-bypass (the C1 BLOCKER root cause)
- Preserves user-facing-vs-privileged semantic distinction (which is itself a useful design property)
- Lower LOC than (a); higher security-honesty than (c)
- Allows path-(a) lift in G-COMP-1 without retracting v1-beta posture claims

**Default verdict: APPROPRIATE.**

### 3.3 — Hidden costs?

Per Section 2.1 above: 3 MINOR sub-concerns (engine-internal chain_anchor_cid=None semantic; path-b docs-touch should be explicit; path-c parallel-dispatchability should be surfaced more prominently). None block convergence.

### 3.4 — F1 binding-gate posture

v3's "CRITICAL: F1 binding-gate posture" section + Forks table + Δv3-1 narrative collectively satisfy `feedback_surface_arch_decisions_under_auth`. The fork IS surfaced + Ben-ratification gated + default-if-no-answer named.

**F1 framing audit overall verdict: CLOSED-CLEAN.**

---

## Section 4 — Convergence disposition

Per CLAUDE.md rule 9 iterate-to-convergence + strict Q5 termination:

### Substantive findings tally for v3

**STILL-NOT-CLOSED from ROUND-2:** 0
**NEW-CONCERN-INTRODUCED by v3:** 0 substantive (all 3 sub-concerns in Section 2 are MINOR + implementer-brief-time refinements, not BLOCKER/MAJOR)
**F1 properly deferred:** YES — explicit 3-option fork, Ben-gated, default named.

### Verdict: **CONVERGED**

Per strict Q5: a round with 0 substantive (BLOCKER/MAJOR) findings + 0 new substantive concerns + binding gates satisfied → CONVERGED.

**F4 implementer dispatch authorized pending Ben F1 ratification or default-path-(b) per META-CRITIC Pareto recommendation.**

---

## Section 5 — F4 implementer brief readiness checklist (CONVERGED)

### Branch to push to
`phase-4-meta-core/r6-r1-f4-impl` (orchestrator convention).

### Reading list (implementer fetches via `git show origin/<branch>:.addl/...`)

- `phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v3.md` (PRIMARY)
- `phase-4-meta-core/r6-r1-f4-design-staging:.addl/phase-4-meta/F4-DESIGN-TRIAGED-DRAFT-v2.md` (delta baseline)
- `phase-4-meta-core/r6-r1-f4-design-critic-1:.addl/phase-4-meta/F4-DESIGN-CRITIC-1-security-threat-model.md`
- `phase-4-meta-core/r6-r1-f4-design-critic-2:.addl/phase-4-meta/F4-DESIGN-CRITIC-2-api-sealing-dep-direction.md`
- `phase-4-meta-core/r6-r1-f4-design-meta-critic-round-2:.addl/phase-4-meta/F4-DESIGN-META-CRITIC-ROUND-2.md`
- `phase-4-meta-core/r6-r1-f4-design-meta-critic-round-3:.addl/phase-4-meta/F4-DESIGN-META-CRITIC-ROUND-3-LIGHT.md` (this doc)
- `docs/V1-FROZEN-INTERFACE.md` (item 8 narrative — retense target per Δv3-5)
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (Rows D-1..D-21; D-25 reserved by Agent D; F4 mints D-26 only if F1 path-c)
- `docs/V1-BETA-BREAKING-CHANGES.md` (Cohort 6 entry per Δv3-4)
- `docs/SECURITY-POSTURE.md` (Compromise #2 + Compromise #26 retense per F1 path ratified)
- `docs/CLAUDE.md` (baked-in #18 Layer-1 narrative retense per F1 path ratified)
- `crates/benten-platform-foundation/INTERNALS.md` (install-pipeline ordering diagram per Δ9)
- `.addl/dispatch-conventions.md` §§3.5, 3.5g, 3.5h, 3.5i, 3.5j, 3.5l, 3.5n, 3.6g, 3.6h, 3.6i, 3.6j, 3.12, 3.13, 3.14
- Memory citations: `feedback_d17_same_file_sweep_recurrence`, `feedback_pub_error_variant_first_class_mirror`, `feedback_pim_n_sweep_completeness_self_verify`, `feedback_pim_cross_language_rule_mirror`, `feedback_pim_n_prior_phase_explicit_preflight`, `feedback_agent_economics_prefer_thorough_cleanup`, `feedback_review_finding_ground_truth_verify`, `feedback_agent_output_must_commit_before_return`, `feedback_synchronous_mini_review`, `feedback_pim_cite_drift_fp1_recurrence`

### Per-bundle scope (under each F1 path)

| Path | S1 LOC | Other surfaces LOC | Total prod LOC | Total prod+test |
|---|---|---|---|---|
| (a) | ~350 | ~725 | ~1075 | ~2115 |
| (b) | ~250 | ~725 | ~975 | ~2015 |
| (c) | ~80 | ~725 | ~805 | ~1845 |

All three within single-agent feasibility. Path (a) approaches ceiling.

### Hard-escalate triggers

- Δ4 `peer_did` semantic re-question by implementer (v3 conservative position is final) — HARD-ESCALATE if implementer disagrees rather than DISAGREE-WITH-EXPLANATION proceed.
- Scope exceeds ~1500 LOC mid-implementation — HARD-ESCALATE for split.
- Napi-rewiring-mechanism (post Agent A's `pub(crate)` visibility tighten on `Engine::put_node`) is unspecified at start-of-impl — HARD-ESCALATE for Ben/orchestrator coordination.
- `is_synthesized_node_id` test corpus uncovers additional synthesized-DID patterns beyond `node-id:` — HARD-ESCALATE (D-18 scope creep).
- `cargo-public-api` regen surfaces UNEXPECTED public-surface deltas beyond Δv3-4 enumeration — HARD-ESCALATE.
- S5 grep discovery uncovers MORE THAN 4 sites — HARD-ESCALATE (v3 named "4 expected sites"; >4 = scope expansion needing re-triage).
- S3c custom-policy test-isolation analysis (per Section 2.3 #2) discovers downstream `check_write` invocations in apply_atrium_merge path that would falsely-trigger panic — HARD-ESCALATE for test redesign.

### Coordination notes

- **3-stage sequential dispatch** — Agent A (R6FP-A cfg+visibility) → Agent B (R6FP-B crypto+security+wire) → F4. **F1 path-(c) exception: F4 may proceed in parallel with Agent A since path-(c) does NOT touch napi binding + does NOT require F2 visibility tighten prerequisite.**
- **F4 implementer MUST NOT touch** `plugin_manifest.rs::verify_peer_signature`, `install_record.rs::verify_user_signature`, `module_ecosystem.rs::verify_upgrade_author_continuity` (Agent B scope L2-R6-MAJOR-1 + L2-R6-MAJOR-2).
- **CATALOG bump coordination:** F4 mints 2 new ErrorCodes (`PluginInstallConsentDenied` + `PluginPerDelegationDenied`). Agent B mints `PluginUpgradeAuthorBroken`. If Agent B lands first → F4 starts at CATALOG = 193 → bumps to 195. If F4 lands first → F4 starts at 192 → bumps to 194; Agent B rebases to 195.
- **Row D-26 mint conditional on F1 path-(c)** — D-25 reserved by Agent D for V1-BETA-BREAKING-CHANGES.md ledger sweep on `phase-4-meta-core/r6-r1-fp-d-docs-ledger-ci`. Verify Agent D's branch is current pre-mint.

### Pre-push gate (§3.5h MANDATORY — enumerated)

- `cargo +1.95 clippy --workspace --all-targets -- -D warnings`
- `cargo +stable clippy --workspace --all-targets -- -D warnings` (per §3.5j + `feedback_pim_n_stable_clippy_gate`)
- `cargo nextest run --workspace`
- `cargo doc --workspace --no-deps`
- `cargo deny check`
- `cargo run -p drift-detect-error-variant-mirror` (or equivalent — find via `cargo run -p --list`)
- `npm run drift:errors`
- `npm run drift:public-api`
- `cargo run -p cite-drift` (or equivalent doc cite drift detector)
- `jq .` on every touched JSON artifact (§3.5h amendment per R6-FP-3)
- `cargo-public-api` baseline regen (3 baselines: errors + engine + foundation)
- `cargo nextest run -p phase-3-workspace-tests --test missing_docs_workspace` (per `feedback_workspace_missing_docs_test_invocation`)
- `git grep -nE 'InstallPorts\s*\{'` workspace-wide → enumerate all fixture sites + verify they compile post-Option-drop (Δv3-4)
- `git grep -nE 'policy\.check_write\b'` workspace-wide → verify exactly 4 production sites swept + ucan_grounded.rs EXCLUDED (Δv3-7)

### Mini-review trigger

- Synchronous mini-review per group per `feedback_synchronous_mini_review`
- Mini-reviewer pre-flight: §3.5i rebase-staleness check (verify F4 branch is merged with origin/main + Agent A's branch + Agent B's branch tips)
- Mini-reviewer applies §3.5n orchestrator-ground-truth-verify discipline on every MAJOR finding
- Mini-reviewer brief explicitly references this doc (`F4-DESIGN-META-CRITIC-ROUND-3-LIGHT.md`) + all 7 closure dispositions + 3 NEW-concern-closures for cross-check

### Commit-before-return + force-add discipline

- F4 implementer brief MUST mandate commit-before-return per `feedback_agent_output_must_commit_before_return` (auto-managed worktrees auto-clean if no commits land + uncommitted output files evaporate)
- All `.addl/` outputs MUST be force-added (`.addl/` is gitignored)
- F4 implementer MUST push to branch BEFORE returning — no return-without-push

### Strategy-C batch eligibility

- F4 lands as ONE PR with 7 commits per v3 phasing
- No Strategy-C needed for F4 itself
- If F4 lands AFTER Agent A + Agent B (sequential), no batch consolidation across siblings needed
- If F1 path-(c) selected + F4 lands in parallel with Agent A, consider Strategy-C batch with Agent A merge (per `feedback_strategy_c_batch_mandatory_not_optional` ≥3 PRs threshold — may not apply for 2 PRs)

---

## End of META-CRITIC-ROUND-3-LIGHT

**Convergence verdict: CONVERGED.** All 12 ROUND-2 flags + binding gates CLOSED-CLEAN by v3 (some with minor implementer-brief-time sub-concerns documented in Section 2; none BLOCKER/MAJOR).

**F4 implementer dispatch authorized** pending Ben F1 ratification OR default-path-(b) per META-CRITIC ROUND-2 Pareto recommendation.

**Section 5 brief readiness checklist** is the F4 implementer brief outline; orchestrator may author the dispatch brief directly from it.
