# Option F+ §6.2 envelope-layer-unification — C4 PROCESS-DISCIPLINE CRITIQUE

**Role.** C4 critique-round member (sibling of C1 elegant-shape + C2 composability + C3 fresh-eyes + C5 formal-methods). Distinct lens: **what does this 9-eyes-panel + consolidator reveal about Benten's broader review-process discipline?**

**Authority.** ADVISORY. Consolidator output and prior critique-round memos are evaluated, not deferred-to. Process recommendations land as MEMORY/§-rule candidates with full text + revisit-trigger.

**Branch.** `phase-4-meta-core/option-f-plus-critique-c4-process-discipline`
**Tree-state pre-flight.** Clean against `origin/main` HEAD `2172cb6d`; isolated agent worktree; absolute-paths forbidden outside `${WORKTREE_ROOT}`.
**Date.** 2026-05-27.
**Input pin.** Consolidated registry @ `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry` SHA `fbdfeb16`.

---

## §1 Executive verdict + confidence

**Top-line.** The 9-eyes-panel + ADVISORY consolidator + critique-round is the right shape for an F-full-class architectural decision (foundational invariant mints + wire-format-freeze + load-bearing-Compromise-mints) — but several elements of the process were structurally over-invested, several were structurally under-invested, and the *standing-pattern codification* lags the actual practice by a generation.

**Cost is justified for THIS decision class.** ~15 agent-dispatches × ~1-2 weeks wall-clock for a v1-beta-wire-format-freezing + 3-invariant-minting + 13-Compromise-minting + 22-load-bearing-amendment decision is at-or-below the cost of *one* externally-discovered v1-beta-wire-format-breaking issue post-tag. The asymmetry is correct. **Confidence HIGH.**

**The shape is RIGHT but the standing-pattern naming is MISSING.** Right now this looks like a one-off heroic effort. It should look like the EXECUTION of a named, repeatable, codified standing pattern ("F+-class-decision protocol" or "N-eyes-consolidator+critique" or equivalent). Future architectural-decision-class work will RE-DERIVE this shape from scratch unless codified. **Confidence HIGH.**

**Three substantive process-discipline gaps surfaced by C4 (not by consolidator's own MF5 self-critique).**
1. **Sequential-vs-parallel cryptographer composition was implicit, not designed.** L1→L2→L3 ran sequentially (each reading the prior); L4-L9 ran parallel. The sequential cryptographer chain was load-bearing for getting Am1-Am8 right; the parallel non-cryptographer fan-out was load-bearing for coverage. This sequencing choice is INVISIBLE in the consolidator's framing — codify it.
2. **Consolidator-ADVISORY-not-AUTHORITATIVE is correct but the boundary is fuzzy.** §5 disagreement-matrix consolidator recommendations bleed into authoritative-feeling by §6 v1-beta-LOAD-BEARING categorization. Per `feedback_review_finding_ground_truth_verify`, downstream DISAGREE is first-class — but a clearer "verdicts vs consolidator-leans vs surfaced-fork-for-Ben" tri-state would sharpen this.
3. **No "decision-economics retrospective" is scheduled.** After Ben ratifies, did the 13-month metadata-leak analysis (L6) save more wave-days post-v1-beta than it cost? No mechanism captures this. Without a retrospective trigger, the 9-eyes-pattern cannot be tuned for the *next* F+-class decision.

**Confidence summary.** §2 catalog HIGH (derivable from consolidated registry origin-citations). §3 Pattern 6 effectiveness MED-HIGH (some judgment calls about counterfactual smaller panels). §4 consolidator value-add HIGH (direct evidence in registry). §5 critique-round shape MED-HIGH (self-reference + sibling-critique-rounds-in-flight). §6 codifications HIGH on shape, MED-HIGH on exact memo text (Ben to refine). §7 broader assessment MED-HIGH (single-datapoint reasoning). §8 recommendations HIGH on direction.

---

## §2 9-eyes-panel coverage catalog

For each lens: classes-of-finding contributed + unique-value-add vs redundancy-signal vs gap-signal. Derived from the consolidated registry's "Origin" and "reaffirmed" rows + consolidator MF1-MF5.

### L1 — 1st cryptographer NO-GO + §6.2 sketch (`6d4e173f`)

**Unique value-add.**
- Diagnosed the original Option F+ pseudo-keypair pattern as structurally broken (NO-GO verdict) — this is the *generative* finding without which none of the rest follows. No other lens was positioned to render this verdict.
- Proposed §6.2 envelope-layer-unification as the right *direction* (not just "Option F+ is broken; come back with a new design"). Generated the substrate the other 8 lenses critique.
- Provided the precedent table (Bitwarden/1Password/Molly/age scrypt-recipient/Stronghold/OPAQUE) that L4's L1 §3 reference inherits.

**Redundancy.** None — L1 is generative; nothing to be redundant with at this stage.

**Gap.** L1 named the formal-methods coverage gap (HPKE-mode-base[X-Wing] IND-CCA2 proof under adversarially-chosen-recipient-seed model) but did not dispatch a lens to close it. Consolidator MF5 reaffirms.

**Process-discipline observation.** L1 = "GENERATIVE-NO-GO" lens; this is structurally a distinct shape from "review existing substrate" lenses. Codification candidate: a generative-NO-GO lens MUST be sequenced before any reviewing lenses can run; treating L1 as "first of 9 parallel lenses" would have produced 8 wasted parallel dispatches.

### L2 — 2nd-opinion cryptographer CONCUR-WITH-AMENDMENTS (`7e900a3b`)

**Unique value-add.**
- Am1 (codepoint MUST be committed inside canonicalized AAD/info-string) — load-bearing foundation that ALL subsequent amendments depend on. Without Am1, U3 (canonical TLV) + U4 (sender-DID-in-AAD) + U7 (BE-codepoint) all rest on sand.
- Am2 (strict-decode discipline; codepoint determines variant; no cross-variant fallback) — foundational. Without Am2, U9/U10 `#[non_exhaustive]` permanence package collapses.
- Primitive-neutral invariant phrasing for Inv-16 — selected by consolidator as the canonical phrasing. The cryptographic literacy to phrase an invariant primitive-neutrally is itself the value-add.

**Redundancy.** Am1+Am2 are reaffirmed by L3+L4+L8+L9 (4 downstream lenses cite them). This is STRENGTH-SIGNAL not over-investment — Am1+Am2 are load-bearing foundation; cross-lens confirmation matters for foundation-class amendments.

**Gap.** L2 noted "gap in multi-recipient stanza composition" (§2.3 row 5 per consolidator) but did not design the closure — that fell to L9 4 hours later. Was the gap discovery sufficient or should L2 have proposed initial shape? Process-discipline judgment: gap-discovery + dispatch-to-specialist is correct; cryptographer should not also design Atrium-composition.

**Process-discipline observation.** L2 = "FOUNDATION-AMENDMENT" lens; structurally must run AFTER L1 (because it amends L1's sketch) but BEFORE the parallel fan-out (because the parallel lenses cite L2 amendments). This 3-stage cryptographer sequencing (NO-GO → FOUNDATION-AMENDMENT → ADVERSARIAL-CALIBRATION) is a STANDING PATTERN worth naming.

### L3 — adversarial-design CONCUR-WITH-CALIBRATION (`13b624c3`)

**Unique value-add.**
- Am3 (canonical-serialization length-injectivity via TLV) with *concrete collision demonstrated* in §2.1 — the only lens to produce a constructive break-vector. This is the highest-leverage finding shape: a concrete collision proof closes the entire JOSE/JWT alg-confusion class.
- Am4 (sender-DID-in-AAD) — defense-in-depth against AAD-belief-drift; later flagged by L6 as metadata-tension trigger (consolidator's surfaced 5th disagreement).
- Am5 (replay-window: sealed-at + valid-until epoch) — adversarial-thinking class finding.
- Am6 (Bernstein-Persichetti ML-KEM Decap CT-mitigation + Compromise #32 mint) — only L3 named B-P specifically; L4 + L5 reaffirmed in the impl + audit-readiness frames. The "named adversary published in IACR 2024/2051" framing is L3's adversarial-literacy value-add.

**Redundancy.** Am3 reaffirmed by L4/L5/L7/L8 (4 lenses) → STRENGTH-SIGNAL. Am4 reaffirmed by L5 + L9 + TENSION-FLAGGED by L6 → the right kind of cross-lens engagement (concur + tension surfaced).

**Gap.** L3 named multi-stanza HPKE cross-substitution as deferred (§2.7); L9 picked it up. Same gap-discovery-then-specialist-dispatch pattern as L2.

**Process-discipline observation.** L3 = "ADVERSARIAL-CALIBRATION" lens; closes the cryptographer 3-chain. **Was 3 cryptographers the right count?** Evidence says YES because:
- L1 = generative-NO-GO (irreducible)
- L2 = foundation-amendment (provides Am1+Am2 substrate everything else depends on)
- L3 = adversarial-calibration + concrete-collision-proof (provides break-vector)

Cutting to 2 would lose either Am3 concrete-collision (if L3 cut) or Inv-16 primitive-neutral phrasing (if L2 cut). Either loss costs ≥1 Compromise-mint or invariant strength. Cutting to 1 collapses the entire foundation. Going to 4+ would have diminishing-return: L4's IMPL-A1 already reaffirmed B-P at the crate level; a 4th cryptographer would re-confirm what L4 already mechanically closed. **3 cryptographers was the structural minimum AND the structural maximum.**

### L4 — impl-engineering CONCUR-WITH-IMPL-AMENDMENTS (`4d4aae5f`)

**Unique value-add.**
- IMPL-A1 (libcrux-ml-kem) — mechanically closes L3/Am6 B-P at the crate level. Closes a load-bearing-Compromise-mint via crate selection, not via wire-format change. Highest-leverage impl-finding shape.
- IMPL-A2 (XChaCha20-Poly1305) — surfaced Layer-A/Layer-B nonce-reuse hazard that no cryptographer named directly (cryptographers focused on Layer-C/D). The "primitive choice per layer based on key-reuse-frequency" is impl-literacy.
- IMPL-A3 (single-source `canonical_binding()` via NAPI-RS) — operationalizes §3.5g cross-language rule-mirror discipline; without IMPL-A3, U33 would be a doc-only commitment.
- IMPL-A4 (opaque-handle pattern for NAPI secrets) — the V8-GC-vs-Rust-zeroize compose-failure is a class-of-bug only impl-engineering surfaces.
- IMPL-A5 (`wasm_js` getrandom feature config) — runtime-panic-on-first-OsRng-call is invisible to cryptographers; only impl-engineering catches.
- IMPL-A6 (tokio cancel-safety) — async-context AEAD failure mode.
- IMPL-B1 (existing aead.rs uses LE; surfaces §5 Q2 conflict) — *the only lens that read the existing code*; cryptographers reasoned from RFC conventions without verifying existing impl. Massive value-add.
- LOC + wave-day cost estimate (~2730 LOC + ~4200 test LOC + ~35-45 wave-days) — feeds §6 v1-beta scope feasibility.

**Redundancy.** IMPL-A1 reaffirms L3/Am6 + L5 audit-readiness — STRENGTH-SIGNAL at the crate-selection axis.

**Gap.** L4 did not evaluate `oqs-rs` (Option A) as a fallback — consolidator §5 Q1 notes this. L4 picked Option B (libcrux) but didn't construct the counterfactual. If Ben rejects libcrux for any reason (license, supply-chain concern), the decision-tree has no L4-evaluated fallback.

**Process-discipline observation.** L4 = "IMPL-ENGINEERING" lens; the *only* lens that read existing code. This is a load-bearing distinct lens. Codification candidate: any architectural decision touching existing code MUST dispatch an "existing-code-readiness" lens; cryptographer/abstract lenses systematically miss LE-vs-BE and similar existing-code conflicts.

### L5 — threat-model + audit-readiness AUDIT-READY-IN-DIRECTION (`3f27f8e0`)

**Unique value-add.**
- 11 Compromise mints (#33-#44 minus #42-#43 which originated elsewhere) — single largest contribution by Compromise count. Compromises #33-#41 cover the entire OUT-OF-SCOPE adversary class (coercion, password-knowledge, compromised-device, RAM-residency, TEE-absence, physical side-channels, supply-chain, reproducible-builds, cross-device-sync) — without L5 these would surface piecemeal during external audit at 10×-100× cost.
- 3 invariants (Inv-L5-2 hybrid-floor; Inv-L5-3 codepoint-registry; partial Inv-L5-1) — Inv-17 hybrid-floor is L5-unique.
- 25-row threat-model matrix (T-01..T-25 IN/OUT/PARTIAL) — single artifact saves ~1 person-week of audit-time per L5's own estimate.
- THREAT-MODEL.md skeleton — the audit-firm-recognized shape (modeled on age + Obsidian-Sync + Common-Criteria-ST).
- Bernstein-Persichetti reaffirmation at audit-readiness frame (AF-27).

**Redundancy.** Compromise #32 reaffirmation overlaps L3; #42 FS-gap reaffirmation overlaps L8; #31 extension overlaps L9. All STRENGTH-SIGNAL — Compromise mints benefit from cross-lens confirmation because audit-firms cross-check.

**Gap.** L5 enumerated regulatory regimes (eIDAS-3, German critical-infrastructure, French regulated-financial) as OUT-OF-SCOPE without designing closure paths. Consolidator MF5 names this as a gap-signal: if Benten's customer base shifts, this becomes a BLOCKER not honest-disclosure. L5 should have proposed an EXPLICIT revisit-trigger for each named regulatory regime, not just enumerated them.

**Process-discipline observation.** L5 = "AUDIT-READINESS" lens; load-bearing on a unique axis (turning the work product into something an external audit firm can consume in audit-time-budget). This is structurally distinct from threat-modeling — and L5's combined-role (threat-model + audit-readiness) is borderline overloaded. Codification candidate: at v1-beta-FREEZE-class decisions, threat-modeling and audit-readiness may warrant SEPARATE lenses (current L5 served well but the combined scope risks under-investing one side).

### L6 — privacy / metadata-leak CONCUR-WITH-CALIBRATION + DISAGREE-LOAD-BEARING-FINAL (`986e50bb`)

**Unique value-add.**
- Am7 Sealed-Sender additive codepoint slot (`DROP_TO_RECIPIENT_SEALED_SENDER = 0x6510`) — the single highest-leverage privacy amendment; locks v1-beta wire-format-evolution path for Signal-pattern metadata-hiding without bloating v1-beta scope.
- Am8 per-relay-unlinkability transport-layer re-blinding.
- Am9 padding to fixed-size-class buckets per codepoint.
- Am10 cover-traffic NAMED-DEFERRED (with explicit revisit-trigger on shaped-relay-transport-landing).
- Am11 multi-stanza per-recipient-unlinkable copies invariant.
- Am12 coarse 1-hour epoch buckets (refines U5; reduces 33-bit timestamp leak to 14-bits/year).
- Compromise #43 (envelope metadata leakage) mint — the load-bearing privacy-class Compromise.
- Inv-16-metadata clause (consolidated into Inv-18(c)) — pairing-discipline invariant requiring every plaintext-sender-AAD variant to have a Sealed-Sender sibling slot.
- *Surfaced the Am4 tension* (sender-DID-in-AAD STRICTLY WORSENS metadata privacy) — consolidator §5 Q-extra. This is METADATA-PRIVACY-VS-AUTHENTICATION as a structural tension, not incidental clash (consolidator MF1).
- DISAGREE-LOAD-BEARING-FINAL stance on default-codepoint-Sealed-Sender vs additive-slot — surfaced as §5 Q5; the *only* lens to issue a DISAGREE-class stance (others CONCUR-WITH-CALIBRATION).

**Redundancy.** None — L6's privacy lens is structurally distinct from every other lens; nothing to be redundant with.

**Gap.** L6 did not propose a UX-affordance closure for the Sealed-Sender opt-in (when default codepoint is metadata-promiscuous and Sealed-Sender requires explicit caller action; the "secure-by-default" framing may be misleading to users). Consolidator MF5 names this as a gap-signal. L6 implicitly assumed UX-coupling was out-of-scope; this assumption deserves explicit critique.

**Process-discipline observation.** L6 = "PRIVACY / METADATA-LEAK" lens. **Was this lens worth its weight?** Evidence YES: 6 amendments + 1 Compromise + 1 invariant clause + 1 surfaced tension. Without L6, U22 Sealed-Sender codepoint slot would NOT have been reserved at v1-beta — and adding it post-v1-beta would be a wire-format-break. **L6 alone justified ≥10 wave-days post-v1-beta cost-avoidance.** This is the highest-cost-avoidance ratio of any single lens.

### L7 — cross-ecosystem-interop SOUND-BUT-§3.5s-NON-COMPLIANT (`208f98bb`)

**Unique value-add.**
- Am9 cross-ecosystem-identifier-as-content emit-discipline (§3.5s operationalization) — operationalizes a standing dispatch-convention rule.
- Am10 DAG-CBOR outer framing with Benten-private CBOR-tag — promoted by consolidator to LOAD-BEARING.
- HPKE-11 vs HPKE-11-KE mode mapping correction — surfaced as §5 Q4.
- Mapping table proposal: codepoint → (LAMPS OID, JOSE alg-name, COSE alg-id, multicodec key-code, age stanza-name).

**Redundancy.** Am9 reaffirms §3.5s discipline (already codified); not new but operationalizes. STRENGTH-SIGNAL.

**Gap.** L7 stopped at RECOMMENDED for DAG-CBOR rather than LOAD-BEARING. Consolidator promoted. The lens's own stop-at-RECOMMENDED stance is defensible (deferring to Benten's adoption posture) but the lens should have either (a) committed to a recommendation OR (b) explicitly named the decision as a Ben-call. Stopping at RECOMMENDED-ambiguously creates downstream load on the consolidator.

**Process-discipline observation.** L7 = "CROSS-ECOSYSTEM-INTEROP" lens. **Was this lens worth its weight?** Evidence YES but at LOWER ratio than L6: 2 load-bearing amendments + 1 codepoint mapping table + 1 mode-mapping correction. The §3.5s-discipline operationalization alone is sufficient justification. **Lower-ratio not over-investment** — cross-ecosystem-interop concerns are wire-format-affecting and frozen post-v1-beta; this is the only window to surface them.

### L8 — wire-format-stability CONCUR-WITH-EXTENSIONS (`d8d3c41c`)

**Unique value-add.**
- Am9 `EnvelopePayload` `#[non_exhaustive]` + typed-reject unknown variants.
- Am10 `BindingContext` `#[non_exhaustive]`.
- Am11 escape codepoint `0xFFFF` + experimental range `0xFE00..0xFFFE` + EnvelopeShape codepoint axis (4th axis).
- Am12 nonce-length-variant discrimination.
- Am13 FS-gap honest-disclosure (Compromise #42) + reserve MLS-PQ/CGKA/Bird-of-Prey codepoint brackets at v1-beta.
- Am14 `aad_version: u8` byte + TLV tag `0xFF` extended-canonicalization marker.
- Am15 `Did` multikey canonical-serialization + `Did::Unknown` typed-rejection.
- Am16 CodepointLifecycle typed-state — closes V1-FROZEN-INTERFACE.md item 6 "supported FOREVER" tension with future cryptanalytic breakthroughs.
- Promoted Am8 codepoint-registry to LOAD-BEARING.

**Redundancy.** Am14 `aad_version` extends L2/Am1; Am9 `#[non_exhaustive]` extends L2/Am2. STRENGTH-SIGNAL extension-pattern.

**Gap.** L8 introduced 8 amendments individually but consolidator's MF3 noted they form ONE coherent permanence-package (variant/codepoint/canonicalization 3-axis). L8 missed this consolidation opportunity at the lens level. Process-discipline lesson: wire-format-stability lens should explicitly check the 3-axis permanence package as a unit.

**Process-discipline observation.** L8 = "WIRE-FORMAT-STABILITY" lens. Highest amendment count per lens (8 amendments + 1 promotion). Permanence-class amendments dominate v1-beta-LOAD-BEARING categorization (8 of 22 LOAD-BEARING amendments originate from L8). **L8 was the single most-productive lens by amendment-count.** Without L8, v1-beta would freeze with non-permanent shapes that cost wire-format-break to extend post-v1-beta. **L8 alone justified ≥20 wave-days post-v1-beta cost-avoidance.**

### L9 — atrium-integration CONCUR-WITH-CALIBRATION (`1670aa03`)

**Unique value-add.**
- A1 HpkeMultiBase variant + cross-stanza substitution defense — closes the L3-named multi-stanza gap.
- A2 dual-CID model (plaintext_cid + envelope_blob_cid) — surfaced as §5 Q3 disagreement with prior-P2P-architect F-refinement-2.
- A3 recipient_key_generation binding — composes with Compromise #31 extension.
- A4 K_principal-generation tracking + Atrium-replicated KPrincipalRotation node.
- A5 ExecuteWorkflow variant for hyper-scaling rented-compute use case.
- *Disagreement with prior P2P-architect* on dual-CID vs recipient-set-in-CID — the only lens to directly CONTRADICT a prior architectural decision. This is high-stakes disagreement; consolidator concurred with L9.

**Redundancy.** Minimal — Atrium-integration concerns are structurally distinct from every other lens.

**Gap.** L9 did not stress-test Atrium-CRDT-conflict-resolution at the K_principal-rotation-log layer (what happens when two devices independently rotate concurrent generations?). The KPrincipalRotation Node is Atrium-replicated, so CRDT-conflict-resolution semantics matter. Consolidator MF5 lens-composition self-critique notes this kind of specialist gap.

**Process-discipline observation.** L9 = "ATRIUM-INTEGRATION" lens. The dual-CID disagreement is high-stakes — without L9, the consolidator would have inherited F-refinement-2's recipient-set-in-CID conflation. **L9 alone may have avoided a forkability-break + graph-reference-CID-mutation hazard.**

### Cross-lens gap signals (findings NO lens surfaced)

Consolidator MF5 names 4 gaps:
1. **No formal-methods lens** for HPKE-mode-base[X-Wing] IND-CCA2 proof under adversarially-chosen-recipient-seed model.
2. **No regulatory-jurisdiction-specific lens** for EU eIDAS-3 / German / French regimes.
3. **No UX-affordance lens** for Sealed-Sender opt-in framing.
4. **No multi-stanza-HPKE specialist lens** — gap to dispatch at R3/R5 implementation time.

**C4 adds 3 additional gaps:**
5. **No performance/wire-size lens.** L6 padding (U24) has a wire-size cost (~10-20% inflation depending on bucket choice); L7 DAG-CBOR (U30) has ~5% cost. No lens analyzed cumulative wire-size cost vs alternatives. For a P2P system over iroh-blobs, wire-size matters.
6. **No Atrium-CRDT-conflict-resolution lens.** As noted in L9 gap above — K_principal-rotation-log + recipient-key-generation tracking have concurrency semantics that no lens evaluated.
7. **No backwards-compatibility-migration lens.** Existing G-CORE-3a aead.rs uses LE codepoint; §5 Q2 surfaces the conflict; but no lens designed the migration path (data-at-rest re-encryption strategy; user-facing migration UX; rollback safety).

---

## §3 Pattern 6 effectiveness assessment

Per `feedback_reviewer_composition`: Pattern 6 picks reviewers by what can go wrong with THIS work, not fixed review-size.

### Was the composition right for the lens-surface?

**Yes, with caveats.** The 9-lens composition (3 cryptographer + 6 distinct lens surfaces) covers the central lens-surface for an F+-class encryption-substrate decision. The §2 catalog evidence is that every lens contributed unique value beyond reasonable doubt. **No lens was a clear redundant or under-contributor.** This is a strong Pattern-6 datapoint.

### Specific sub-questions from the brief

**3 cryptographers — right count?**
- **YES** per §2 L3 process-discipline observation. The 3-cryptographer chain (L1 generative-NO-GO → L2 foundation-amendment → L3 adversarial-calibration) is structural-minimum AND structural-maximum. Cutting to 2 loses ≥1 Compromise-mint or invariant strength. Going to 4+ has diminishing return because L4 IMPL-A1 mechanically reaffirmed at the crate level.
- **CONFIDENCE HIGH.**

**No formal-methods lens — real gap?**
- **YES for v1-GM; NO for v1-beta.** Per L5: informal-rigorous-prose is acceptable for v1-beta; external audit at v1-GM may force the issue. The gap is structural but the deferral is reasonable. However: **codify the deferral as NAMED-DEFERRED with explicit revisit-trigger** ("first external audit firm engaged for v1-GM" or "regulatory-jurisdiction-shift triggers formal-proof requirement").
- **Codification candidate per §6.**

**No perf/wire-size lens — real gap?**
- **YES, moderate severity.** U24 padding has ~10-20% wire-size cost depending on bucket choice; U30 DAG-CBOR ~5%; cumulative iroh-blobs transport overhead unanalyzed. For a P2P content-addressed system, wire-size matters more than for a single-server centralized system. Pattern-6 should add a perf/wire-size lens for wire-format-freeze-class decisions.
- **Codification candidate per §6.**

**No UX-coupling lens — real gap?**
- **YES, moderate-to-high severity.** §5 Q5 Sealed-Sender opt-in framing is fundamentally a UX question (when is "secure by default" misleading?). The crypto lens-surface cannot decide this. Pattern-6 should add a UX-coupling lens for any decision affecting user-visible security framing.
- **Codification candidate per §6.**

**No regulatory-compliance-deep-dive lens — real gap?**
- **PARTIAL severity.** L5 covered jurisdictions at enumeration level; deeper analysis is OUT-OF-SCOPE at Benten's current customer-base. If Benten's customer base shifts to EU/German/French regulated industries, this becomes BLOCKER. The deferral is reasonable; the codification gap is naming explicit revisit-triggers per regime.
- **Codification candidate per §6 (revisit-trigger discipline, not standing Pattern-6 lens).**

**No Atrium-CRDT-conflict-resolution lens — real gap?**
- **YES, moderate-to-high severity for Atrium-touching decisions.** L9 covered Atrium-integration but not CRDT-conflict-resolution at the rotation-log layer. Two devices rotating K_principal concurrently produce divergent generation counters; how Atrium resolves matters for U20 correctness.
- **Codification candidate per §6.**

### New standing lenses worth adding to Pattern 6

For *future* architectural-decision-class reviews:
- **Existing-code-readiness lens** (L4 implicitly served; codify as explicit lens) — verify reasoning matches existing code state, catch LE-vs-BE-class conflicts.
- **Permanence-3-axis-package lens** (L8 produced 8 amendments that consolidator MF3 noted should be ONE package) — explicitly check variant/codepoint/canonicalization 3-axis as a unit.
- **Perf/wire-size lens** for wire-format-freeze decisions.
- **UX-coupling lens** for user-visible security framing decisions.
- **CRDT-conflict-resolution lens** for Atrium-touching decisions.
- **Backwards-compat-migration lens** for decisions touching frozen-or-near-frozen interfaces.

Pattern-6 doesn't need ALL of these every time — it picks per lens-surface. But the catalog should expand so future orchestrators don't omit them by oversight.

---

## §4 Consolidator value-add assessment

### Did the consolidator add NEW value or just re-organize?

**Both, with the NEW-value contribution being higher than the structural framing suggests.**

**Re-organization value (mechanical):**
- 46 raw amendments → 28 unified amendments (38% deduplication).
- 11+1+1+1 Compromise mints → 13 unified Compromises + #31 extension (correct dedup of overlapping mints).
- 5 invariant proposals (Inv-L5-2 + Inv-L5-3 + L6-Inv-16-metadata + L2-Inv-16 + L8-Am16-CodepointLifecycle) → 3 unified invariants (Inv-16, Inv-17, Inv-18).
- 4 brief-named disagreements + 1 ml-kem-crate disagreement surfaced by L4/L5 → 5 disagreement-matrix rows.

**NEW value (structural):**
- §5 Q-extra (consolidator-surfaced 5th disagreement): Am4 sender-DID-in-AAD tension with L6 metadata-leak posture. *No individual lens framed this as a fork.* L6 named the tension; consolidator framed as a disagreement-with-resolution (keep U4 for default codepoint + reserve U22 Sealed-Sender for metadata-privacy-preferring callers). **This is genuine NEW analysis, not re-organization.**
- §6 v1-beta-LOAD-BEARING subset (22 + 6 + 8 + 2 categorization with per-amendment Disposition assignment). Individual lenses noted severity; consolidator did the bucket-categorization work that no individual lens did at registry-level.
- §8 MF1-MF5 pattern-induction meta-findings. Especially MF1 (sender-auth vs metadata-privacy STRUCTURAL TENSION not incidental clash) and MF4 (Compromise #31 cascading downstream-hazards across L3+L6+L8+L9) are *cross-lens patterns no individual lens could see*. MF5 (self-critique on coverage gaps) is rare consolidator-discipline.
- §7 R0 skeleton + §8 wave decomposition. Forward-looking work product that consolidates 9 lenses into a downstream plan; this is consolidator-as-bridge-to-execution.
- Inv-18 three-way merge (L5/L6/L8 sibling invariants). Consolidator's judgment call to merge vs split — surfaced as confidence caveat in §9 self-assessment. This is genuine architectural-judgment value-add.

**Verdict.** Consolidator is ~40% mechanical-organization + ~60% NEW-analysis. The 60% is concentrated in §5 Q-extra + §6 bucket-categorization + §8 MF1-MF5 + Inv-18 merge. **Consolidator was high-value-add.** **Confidence HIGH.**

### Was the renumbering choice (single 1-N space) right?

**YES.** Single 1-N unified namespace (U1..U28) is structurally correct because:
1. Each unified amendment cross-references multiple lens origins (U1 cites L2/Am1 + L3/Am1 + L4 + L8/Am14 + L9 + L7/Am9 = 6 origins). Per-lens preservation would force readers to chase 6 origin numbers per amendment.
2. Downstream R0 plan-doc + R3/R5 implementer briefs reference unified U#; per-lens preservation would force translation.
3. Disposition (LOAD-BEARING/CODEPOINT-RESERVE/DEFER/NAMED-DEFER) is per-unified-amendment, not per-lens-origin.
4. Per-lens preservation in the citation rows (Origin + reaffirmed lines) is sufficient for traceability.

**Trade-off accepted:** Unified numbering buries lens-attribution to one-line cite. **Acceptable** because lens reviews remain available at their frozen SHAs for audit-trail.

### Did the 4 (5) pattern-induction meta-findings reveal anything an N+1 lens reviewer couldn't?

**YES for MF1 + MF4; PARTIAL for MF2 + MF3; YES for MF5 self-critique.**

- **MF1 (sender-auth vs metadata-privacy STRUCTURAL TENSION).** No individual lens could see this because L3 produced Am4 + L6 noted the tension *separately*; only a consolidator reading both could synthesize "STRUCTURAL not incidental" + propose paired-sibling-codepoint resolution.
- **MF2 (Inv-15/Inv-16 sibling invariants at different identifier-hazard layers).** Requires knowledge of Benten's existing Inv-15 at registry-pattern level; no individual lens framed at this generality.
- **MF3 (3 permanence amendments form coherent package).** L8 *could* have surfaced this but didn't (L8 produced as separate amendments); consolidator's outside-view caught the package shape.
- **MF4 (Compromise #31 cascading downstream-hazards across ≥4 lenses).** STRUCTURALLY impossible for any single lens to see — requires reading all 4 lenses simultaneously. **The single highest-value pattern-induction finding.** Recommended SECURITY-POSTURE.md downstream-hazard-tree action is concrete and high-leverage.
- **MF5 (lens-composition incomplete coverage self-critique).** Rare consolidator-discipline — naming coverage gaps the consolidator itself didn't close. Per `feedback_extra_reflection_pass_for_elegant_permanent_shape` discipline this is the right shape.

**Could an N+1 lens reviewer have surfaced MF1+MF4?** Possibly MF1 if the N+1 lens had cross-lens-tension as its explicit charter. MF4 requires reading all N lenses simultaneously — that's structurally a consolidator role, not an N+1 lens role. **The 4 meta-findings vindicate the consolidator-not-just-N+1-lens framing.**

### Was consolidator's ADVISORY-not-AUTHORITATIVE shape right?

**YES, with sharpening opportunity.** Per `feedback_review_finding_ground_truth_verify` downstream DISAGREE is first-class; consolidator's ADVISORY framing preserves Ben's authoritative-call on the 5 disagreement-matrix rows + 2 consolidator-leans (U28 + U30) + Inv-18 merge call.

**Sharpening opportunity (C4 finding):** The boundary between "consolidator-mechanical-organization" (authoritative) and "consolidator-architectural-judgment" (ADVISORY) is fuzzy in §6 v1-beta-LOAD-BEARING categorization. Each row's bucket-assignment is a JUDGMENT CALL that consolidator made without explicit Ben-call-marking. Codification candidate: consolidator outputs should TRI-STATE rows — (a) MECHANICAL (no judgment), (b) JUDGMENT-CONSOLIDATOR-LEANED (downstream DISAGREE first-class), (c) FORK-FOR-BEN (no consolidator stance).

### Did consolidator surface enough disagreements + alternatives?

**MOSTLY YES.** Surfaced 5 disagreements + 4 consolidator-leans (U22 additive-vs-default, U28 LOAD-BEARING-vs-DEFER, U30 LOAD-BEARING-promotion, Inv-18 merge-vs-split). C4 would add:
- **Q6 (C4-proposed):** L6's stance on default-codepoint-Sealed-Sender (DISAGREE-LOAD-BEARING-FINAL per L6) was reduced to §5 Q5 MED-leaning-L6. L6's DISAGREE-LOAD-BEARING-FINAL is a STRONGER stance than the consolidator's MED-leaning-L6 framing. Consolidator might be UNDERWEIGHTING L6's DISAGREE-class verdict — this should be explicit.

---

## §5 Critique-round shape assessment (including yourself)

### Is "critique-of-consolidation" a standing pattern worth codifying?

**YES, INDEPENDENT of any single critique-round member.** The structural argument:
1. Consolidator output is a single point-of-failure if not critiqued. ADVISORY framing helps but doesn't catch consolidator-internal blind-spots.
2. Pattern-induction meta-findings (MF1-MF5) emerge from cross-lens reading; critique-round produces *cross-consolidator* meta-findings (e.g., "consolidator MF5 names gaps but doesn't propose codification" = C4 critique).
3. The critique-round catches WHAT THE CONSOLIDATOR MISSED + WHAT THE CONSOLIDATOR OVER-SYNTHESIZED — two distinct failure modes.

### Should critique-round be standard practice AFTER any consolidation of N-reviewer panels?

**NO — only for FOUNDATIONAL-DECISION-CLASS panels** (invariant mints + wire-format-freeze + Compromise mints affecting v1-beta-tag + similar load-bearing-permanent decisions). For routine consolidations (e.g., R6 mini-review batching, fix-pass merges), critique-round is over-investment. The decision-class threshold should be explicit.

**Codification candidate:** "F+-class decision protocol" = generative-NO-GO lens + foundation-amendment lens + adversarial-calibration lens + parallel specialist fan-out + ADVISORY consolidator + critique-round. Applied for: Inv-class mints, wire-format-freeze decisions, v1-beta/v1-GM-tag-gating decisions. NOT applied for: routine fix-passes, batch-merges, sub-wave decisions.

### Are the 5 critique lenses (elegant-shape + composability + fresh-eyes + process-discipline = C4 + formal-methods) the right composition?

**MOSTLY YES.** Per the 5 critique roles:
- **C1 elegant-shape** — catches greedy-sum-of-N vs single-elegant-structural-shape. Already memo'd per `feedback_extra_reflection_pass_for_elegant_permanent_shape`. STRONG fit.
- **C2 composability** — catches cross-amendment composability failures (3rd-reviewer residual concern class). STRONG fit.
- **C3 fresh-eyes** — catches what per-lens reviewers structurally couldn't see (whole-system-shape critique). STRONG fit.
- **C4 process-discipline (this lens)** — catches process-level findings (composition adequacy + standing-pattern codification + cost/benefit). DISTINCT from other critique lenses. STRONG fit.
- **C5 formal-methods** — closes the formal-methods coverage gap consolidator MF5 named. STRONG fit if narrowly scoped to formal-tractability assessment; over-investment if expanded to full proof attempt at this stage.

### What would C4 add or remove?

**ADD:**
- **C6 — Cost-benefit retrospective trigger lens.** No critique-round member is positioned to ask "did this 15-agent panel save more wave-days than it cost?" except retrospectively. Codification: schedule a retrospective at v1-beta-tag (or 6-months-from-this-decision, whichever first) to evaluate the panel's cost-avoidance vs cost-paid. Without this, the F+-class-decision protocol cannot be TUNED for the next decision.

**DO NOT ADD:**
- C7 Atrium-CRDT-specialist or C8 perf-lens — these are gaps in the *original 9-lens panel*, not the critique-round. Adding them to critique-round is the wrong layer; they should be added to Pattern-6 lens-surface catalog instead (per §3 codification candidates).

**REMOVE:** Nothing. The 5 critique lenses are minimal-non-redundant.

---

## §6 Pattern-induction findings for codification

Codification candidates with full memo / §-rule text + revisit-trigger.

### Candidate-1 — `feedback_n_eyes_consolidator_critique_round_for_foundational_decisions.md`

**Full memo text:**
> **N-eyes + ADVISORY-consolidator + critique-round for foundational architectural decisions.** When a decision is FOUNDATIONAL-CLASS (invariant mint OR wire-format-freeze OR Compromise mint affecting v1-beta-tag OR v1-beta/v1-GM-tag-gating decision OR similar load-bearing-permanent decision), the standing pattern is:
> 1. **Generative-NO-GO lens** (1 agent, MUST run FIRST sequentially; reviews the proposed substrate + renders GO/NO-GO verdict + proposes alternative direction if NO-GO).
> 2. **Foundation-amendment lens** (1 agent, runs AFTER NO-GO clears; provides Am-foundation amendments that subsequent specialists depend on).
> 3. **Adversarial-calibration lens** (1 agent, runs AFTER foundation-amendment; adversarial review + concrete break-vectors + Compromise mints).
> 4. **Parallel specialist fan-out** (N agents, run in parallel AFTER the 3-cryptographer chain — impl-engineering + threat-model/audit-readiness + privacy + cross-ecosystem-interop + wire-format-stability + integration-specialist + per lens-surface needs).
> 5. **ADVISORY consolidator** (1 agent; consolidates N+3 lens reviews into unified amendment/Compromise/invariant registry + bucket-categorization + disagreement-matrix + pattern-induction meta-findings + R0 skeleton). NOT a 10th reviewer.
> 6. **Critique-round** (3-5 agents per critique-lens-surface; minimal-non-redundant set = elegant-shape + composability + fresh-eyes + process-discipline + formal-methods).
> 7. **Retrospective trigger** (scheduled at v1-beta-tag or +6 months, whichever first; evaluates cost-avoidance vs cost-paid).
>
> NOT applied for: routine fix-passes, batch-merges, sub-wave decisions, mini-reviews.
> Decision-class threshold: any decision that mints an invariant OR freezes a wire format OR locks a Compromise at v1-beta tag.
> **Worked-example reference:** Phase-4-Meta-Core F+ §6.2 envelope-layer-unification (2026-05-27); consolidated registry `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`.
> **Revisit-trigger:** every F+-class-decision retrospective updates this memo with cost-asymmetry data; if 3 consecutive retrospectives show panel was over-invested, reduce specialist fan-out count; if 3 show under-invested, expand.
> **Companion memos:** `feedback_reviewer_composition` (Pattern 6 lens-surface composition); `feedback_extra_reflection_pass_for_elegant_permanent_shape` (C1 elegant-shape lens); `feedback_review_finding_ground_truth_verify` (ADVISORY-not-AUTHORITATIVE rationale).

### Candidate-2 — Extension to `feedback_reviewer_composition` (Pattern 6) — standing lens-surface catalog

**Full memo extension text:**
> **Pattern-6 standing lens-surface catalog.** For architectural-decision-class reviews, the orchestrator picks reviewers by lens-surface (not fixed N). Standing lens-surface catalog (expanded 2026-05-27 from F+ §6.2 critique):
> - **Cryptographic-substance** — generative-NO-GO + foundation-amendment + adversarial-calibration (3-chain when load-bearing; pair-or-single for sub-decisions).
> - **Impl-engineering** — crate-selection + cross-language-binding + async-safety + platform-config + LOC/wave-day cost estimation + EXISTING-CODE-READINESS (the only-lens-that-reads-existing-code role; catches LE-vs-BE-class conflicts).
> - **Threat-model + audit-readiness** — 25-row adversary-class matrix + Compromise mints + audit-firm-recognized doc shapes. Combined-role borderline-overloaded for v1-beta-FREEZE-class; consider splitting threat-model from audit-readiness for largest decisions.
> - **Privacy / metadata-leak** — distinct from threat-model; covers data-shadow + size-class + timestamp-precision + unlinkability.
> - **Cross-ecosystem-interop** — JWE/COSE/age/multicodec/did:jwk/LAMPS-OID emit-discipline + cross-ecosystem-identifier mapping per §3.5s.
> - **Wire-format-stability / permanence** — `#[non_exhaustive]` + escape codepoint + canonicalization-version + 3-axis permanence-package check (variant/codepoint/canonicalization as a UNIT not piecemeal).
> - **Atrium-integration / CRDT-conflict-resolution** — multi-recipient stanza composition + dual-CID + key-generation tracking + CRDT-conflict-resolution at rotation-log layers (split from atrium-integration when rotation-log Atrium-replicated).
> - **Perf / wire-size** (NEW per C4 §3) — cumulative wire-size cost analysis for wire-format-freeze decisions; padding-overhead; envelope-framing overhead; encoding-format cost.
> - **UX-coupling** (NEW per C4 §3) — user-visible security framing decisions ("secure by default" vs opt-in); when UX framing affects threat-model interpretation.
> - **Backwards-compat-migration** (NEW per C4 §3) — data-at-rest re-encryption + user-facing migration UX + rollback safety for decisions touching frozen-or-near-frozen interfaces.
> - **Formal-methods-tractability** (NEW per consolidator MF5) — tractability assessment for IND-CCA2 / injectivity / similar proofs under bounded shapes; not the proof itself (that's a separate effort) but the tractability triage.
> - **Regulatory-jurisdiction-revisit-trigger** (NEW per consolidator MF5) — explicit revisit-trigger per named regime when OUT-OF-SCOPE deferral could become BLOCKER post-customer-base-shift.
>
> Orchestrator picks per lens-surface; not all lenses every decision. The catalog grows; never shrinks (each addition is evidence-of-prior-miss).
> **Revisit-trigger:** every F+-class-decision retrospective evaluates whether a new lens-surface deserves addition.

### Candidate-3 — `feedback_consolidator_tri_state_disposition.md`

**Full memo text:**
> **Consolidator output should tri-state per-row dispositions.** When an ADVISORY consolidator produces a unified registry from N-lens reviews, each row's disposition should be explicitly TRI-STATED:
> - **MECHANICAL** — no consolidator judgment (e.g., "this amendment = direct copy of L3/Am4 with no merge; reaffirmed by L5/L9"). Downstream consumes as-is.
> - **JUDGMENT-CONSOLIDATOR-LEANED** — consolidator made a judgment call (e.g., bucket-categorization; severity-promotion; merge-vs-split). Downstream DISAGREE first-class per `feedback_review_finding_ground_truth_verify`. Consolidator MUST surface the alternative considered + reason for lean.
> - **FORK-FOR-BEN** — no consolidator stance; explicit Ben-decision-needed (e.g., §5 Q1-Q5 disagreement-matrix rows). Consolidator surfaces options + may recommend, but does NOT lean.
> Default disposition in absence of explicit annotation should be MECHANICAL (most-conservative interpretation).
> **Why this matters.** Current Phase-4-Meta-Core consolidator (`fbdfeb16`) blends MECHANICAL + JUDGMENT-CONSOLIDATOR-LEANED in §6 bucket-categorization without explicit annotation. Reader cannot distinguish "this row is mechanically correct" from "this row is consolidator's lean". DISAGREE-class downstream review of a MECHANICAL row is noise; DISAGREE-class downstream review of a JUDGMENT row is first-class.
> **Worked-example.** Phase-4-Meta-Core F+ §6.2 consolidation §6 bucket-categorization (22 + 6 + 8 + 2). U28 + U30 are explicit-leans (already annotated). The remaining 22 LOAD-BEARING + 6 CODEPOINT-RESERVE rows are mostly MECHANICAL but interspersed without annotation.
> **Revisit-trigger:** next F+-class-decision consolidation reviews this discipline; if consolidator output ships without tri-state, mini-review flags as MAJOR finding.

### Candidate-4 — `feedback_pattern_induction_meta_finding_as_consolidator_output.md`

**Full memo text:**
> **Pattern-induction meta-findings as standing consolidator output.** Every N-lens-panel consolidator MUST produce a pattern-induction meta-findings section (consolidator MF1..MFN per F+ §6.2 precedent). 5 categories proven productive in the F+ §6.2 case:
> 1. **Cross-lens STRUCTURAL TENSION** — when two valid amendments compose only via carve-out (e.g., MF1 sender-auth vs metadata-privacy → paired-sibling-codepoint resolution). NOT incidental clash; structural tension.
> 2. **Sibling-invariants at different layers** — when an existing invariant + a new invariant target same hazard-class at different layers (e.g., MF2 Inv-15 + Inv-16 sibling at signature-construction vs confidentiality-construction).
> 3. **Coherent-package amendments** — when N separately-numbered amendments form ONE coherent package (e.g., MF3 3-axis permanence package). Codify as package-check at lens-level for future.
> 4. **Cascading-Compromise-dependencies** — when an existing Compromise is upstream of N downstream-Compromise-hazards across ≥3 lenses (e.g., MF4 #31 → #32 + #43 + #42 + recipient-rotation). Document as downstream-hazard-tree.
> 5. **Lens-composition coverage gaps self-critique** — explicit naming of what the N-lens panel did NOT cover + why deferral is acceptable + revisit-triggers (e.g., MF5 formal-methods + regulatory + UX-affordance + multi-stanza-HPKE specialist gaps).
> **Why this matters.** Cross-lens patterns are structurally invisible to individual lenses; they emerge only at consolidation. Without explicit codification, consolidators omit this section under time-pressure. MF1+MF4 in particular were the highest-value-add outputs of the F+ §6.2 consolidator.
> **Worked-example.** Phase-4-Meta-Core F+ §6.2 §8 MF1-MF5.
> **Revisit-trigger:** every F+-class-decision consolidation MUST produce ≥3 meta-findings; if fewer surface, critique-round flags as MAJOR.

### Candidate-5 — Extension to existing critique-round elegant-shape memo

**Full memo extension text (to `feedback_extra_reflection_pass_for_elegant_permanent_shape`):**
> **Companion critique-round lenses (codified 2026-05-27 from F+ §6.2 critique-round).** Elegant-shape critique (C1) is one of a minimal-non-redundant 5-lens critique-round for F+-class decisions:
> - **C1 elegant-shape** (this memo) — greedy-sum-of-N vs single-elegant-structural-shape.
> - **C2 composability** — cross-amendment composability; covers 3rd-reviewer residual concern class.
> - **C3 fresh-eyes** — whole-system-shape critique; covers what per-lens reviewers structurally couldn't see.
> - **C4 process-discipline** — composition adequacy + standing-pattern codification + cost/benefit + new codifications.
> - **C5 formal-methods** — closes formal-methods coverage gap consolidator MF5 typically names.
> All 5 run AFTER consolidator output lands; in parallel; ADVISORY; converge via per-critique-finding triage by Ben or designated triager. Each may propose codification candidates (Candidate-1..Candidate-N per critique-member).
> **Revisit-trigger:** if critique-round identifies a 6th non-redundant lens, expand.

### Candidate-6 — `feedback_3_chain_cryptographer_sequencing.md`

**Full memo text:**
> **3-chain cryptographer sequencing for cryptographic-substance decisions.** When a decision touches cryptographic substance load-bearing-ly (encryption substrate; new primitive choice; invariant mint at confidentiality layer), the cryptographer composition is a SEQUENTIAL 3-chain:
> 1. **Generative-NO-GO** (L1-role). Reviews proposed substrate; renders GO/NO-GO verdict; proposes alternative direction if NO-GO. MUST run FIRST. If GO, L2 + L3 can run parallel; if NO-GO, repeat L1 with new direction before continuing.
> 2. **Foundation-amendment** (L2-role). Reads L1 output; provides Am-foundation amendments (codepoint-binding, strict-decode, primitive-neutral invariant phrasing) that all downstream specialists depend on.
> 3. **Adversarial-calibration** (L3-role). Reads L1 + L2 outputs; adversarial review with concrete break-vectors + Compromise mints + replay/length-injectivity/sender-binding amendments.
> **Why sequential not parallel.** L2's amendments depend on L1's substrate; L3's break-vectors depend on L2's foundation. Parallel cryptographer dispatch produces 3 lenses that don't compose (each reasons from a different baseline). Sequential dispatch produces 3 lenses that compose into a single coherent foundation.
> **Why not 2 (skip L2).** Loses primitive-neutral invariant phrasing (L2's value-add per F+ §6.2). Loses Am1+Am2 foundation that L8 + L9 + L7 reaffirm/extend.
> **Why not 4+ (add L3a).** Diminishing return; L4 IMPL-A1 mechanically reaffirms B-P at crate-selection level; a 4th cryptographer re-confirms what impl-engineering closes mechanically.
> **Decision-class threshold.** Apply for: new encryption substrate; new primitive choice; invariant mint at confidentiality-construction layer. Do NOT apply for: routine crypto-impl-bug fix; standard crate-version-bump; sub-spec-level cryptographic clarification.
> **Worked-example.** Phase-4-Meta-Core F+ §6.2 L1 + L2 + L3 (3-chain sequencing; timing evidence L1@13:23 → L2@14:09 → L3@14:42 sequential, then L4-L9 parallel @15:00-15:06).
> **Revisit-trigger:** if a future cryptographic-substance decision is BLOCKED by missing-lens after running 3-chain, expand. If a future decision finds 3-chain over-invested for sub-decisions, formalize a 1-or-2-chain shorter pattern for sub-decisions.

### Candidate-7 — `dispatch-conventions §3.5t` candidate — orchestrator retrospective trigger

**§-rule text:**
> **§3.5t — Foundational-decision-class retrospective trigger.** Every decision dispatched through the F+-class-decision protocol (per `feedback_n_eyes_consolidator_critique_round_for_foundational_decisions`) MUST schedule a retrospective at the earlier of: (a) v1-beta-tag landing, (b) +6 months from consolidation merge, (c) first externally-discovered finding that the F+-class panel missed. Retrospective output:
> - Cost-paid: agent-dispatches + wave-days + wall-clock + Ben-decision-time.
> - Cost-avoided: count + estimated-cost of issues the panel surfaced that would otherwise have surfaced post-tag or in external audit.
> - Process-tuning: amendments to lens-surface catalog (§-Candidate-2); amendments to F+-class-decision protocol (§-Candidate-1); amendments to consolidator output discipline (§-Candidate-3).
> Without a retrospective, the F+-class-decision protocol cannot be tuned. Codification at §3.5t makes this a hard discipline not orchestrator's-optional.
> **Worked-example pending.** Phase-4-Meta-Core F+ §6.2 retrospective scheduled at v1-beta-tag.

### Candidate-8 — `dispatch-conventions §3.6k` candidate — backwards-compat-migration lens for frozen-or-near-frozen interfaces

**§-rule text:**
> **§3.6k — Backwards-compat-migration lens MANDATORY for decisions touching frozen-or-near-frozen interfaces.** Any architectural decision affecting an interface that is already at v1-beta-FREEZE or within ~1 wave-month of v1-beta-FREEZE MUST dispatch a backwards-compat-migration lens. Lens covers: (a) data-at-rest re-encryption strategy; (b) user-facing migration UX; (c) rollback safety; (d) wire-format-version bump discipline (e.g., `ENVELOPE_FORMAT_VERSION_V1 → V2`); (e) golden-vector regeneration. Without this lens, decisions like F+ §5 Q2 (LE-vs-BE codepoint endianness migration in existing aead.rs) get surfaced as conflicts but not designed-with-closure-path.
> **Worked-example.** F+ §6.2 §5 Q2 surfaced LE-vs-BE conflict but consolidator only said "~1 wave-day cost; bump format-version; regen golden vectors" — no closure-path for in-flight encrypted-at-rest data, no rollback strategy.

---

## §7 Broader review-process discipline assessment

### Is the cost (~15 agent-dispatches + ~1-2 weeks wall-clock) justified?

**YES, with high confidence.**

The cost-asymmetry argument:
- **Cost paid:** 15 agent-dispatches × ~$X compute + ~1-2 weeks Ben-orchestration-time + ~1-2 weeks wall-clock.
- **Cost avoided (lower bound):** Even ONE externally-discovered v1-beta-wire-format-breaking issue post-tag costs ≥30-60 wave-days (migration + user-data + UX + rollback). The panel surfaced ≥5 such-class issues (LE-vs-BE codepoint, multi-stanza substitution, B-P Decap CT, FS-gap, metadata-leak Sealed-Sender slot). Even if 4 are over-conservative, 1 catch = ~50× ROI.
- **Cost avoided (upper bound):** External audit firm engagement at v1-GM typically charges $50-200K/week. Pre-audit deliverable production (THREAT-MODEL.md + AUDIT-SCOPE-STATEMENT.md + Compromise mints + Inv coverage) per L5 saves ≥1 person-week per audit-firm-week. ~5-10× ROI on audit-spend alone.

**Verdict: NOT over-investment for FOUNDATIONAL-DECISION-CLASS.** Confidence HIGH.

### Is the AI-orchestration discipline producing better outcomes than shorter chains would?

**MOSTLY YES, with one structural concern.**

Evidence FOR:
- Per-lens isolation:worktree + ground-truth-verify discipline prevented stale-view findings (consolidator's tree-state pre-flight at `2172cb6d` is one datapoint).
- Plain-English-with-prediction discipline surfaced disagreement-matrix in consumable form (§5 Q1-Q5).
- Commit-before-return ensured all 9 lens outputs landed as recoverable artifacts at frozen SHAs (citations §10.1).
- HARD-RULE-no-deferral discipline forced consolidator to either close, name-defer-with-trigger, or surface-as-disagreement — no phantom-deferrals.

Structural concern:
- **Single-orchestrator-bottleneck.** 15 agents converged on Ben's orchestration time. Each disagreement-matrix row + each consolidator-lean + each critique-round finding requires Ben-call. For a single human at AI-tempo, 15 agents producing ~50 actionable decision-points in 2 weeks is at-or-near saturation. Failure mode: Ben triages superficially under time-pressure → consolidator-leans become defaults without genuine evaluation.

**Mitigation candidate:** explicit per-decision-class TRIAGER-OF-RECORD (Ben for FOUNDATIONAL; designated-orchestrator-as-Ben for sub-class). Surfaces saturation-risk explicitly.

### Right-size review chain for different architectural-decision-class?

**YES, decision-class-explicit sizing.** Proposed taxonomy:

| Decision class | Lens count | Critique-round | Consolidator | Example |
|---|---|---|---|---|
| **F+-class (foundational baked-in)** | 9 (3 crypto + 6 specialist) | 5 (C1-C5) | YES ADVISORY | F+ §6.2 envelope-layer-unification |
| **Wire-format-freeze (v1-beta-tag-gating)** | 5-7 (1 crypto + 4-6 specialist) | 3 (elegant + composability + fresh-eyes) | YES ADVISORY | G-CORE-3a aead.rs codepoint format |
| **Invariant mint (sub-foundational)** | 4-5 (1 crypto + 3-4 specialist) | 2 (elegant + fresh-eyes) | YES ADVISORY | Inv-15 signature-bundle-CID |
| **Impl-detail (within frozen substrate)** | 2-3 (impl + adversarial) | 1 (elegant) optional | NO (single-author-with-mini-review) | Specific KDF-context-string addition |
| **Sub-wave fix-pass** | 1-2 (per `feedback_parallelize_fixpasses`) | NO | NO | Single Compromise refinement |

**Codification candidate:** add this taxonomy to Candidate-1 memo.

### What would change if Benten were 6 months from now with 5 contributors?

**Substantively yes.** Key shifts:
- **Lens reviewers can be human-or-AI hybrid.** Currently AI-only out of necessity (1-contributor); with 5 contributors, specialists may be human (e.g., a human cryptographer for L1; AI for L4 + L7).
- **Consolidator role becomes one human's responsibility.** Removes single-orchestrator-bottleneck.
- **Critique-round can be human-only.** Process-discipline + elegant-shape critique require less specialist depth than substantive lenses.
- **Retrospective trigger becomes self-organizing.** A 5-contributor team has cross-checking dynamics that 1-contributor lacks.
- **Memory/standing-rule discipline matters MORE not less.** Without explicit codification, 5 contributors will re-derive process from scratch divergently. The F+-class-decision protocol becomes shared-vocabulary.

**Process-discipline implication.** The codifications in §6 are MORE valuable for a multi-contributor future than for the current single-contributor present. Codify NOW while the practice is fresh.

---

## §8 Recommended new memories + §-rules + Pattern 6 standing-lenses

Summary of §6 codification candidates, with priority + ownership.

### High-priority (codify within next sprint)
1. **Candidate-1** — `feedback_n_eyes_consolidator_critique_round_for_foundational_decisions.md` — names the standing pattern. **HIGHEST PRIORITY.** Without this memo, the F+ §6.2 precedent is one-off-heroic instead of repeatable-standing-pattern.
2. **Candidate-3** — `feedback_consolidator_tri_state_disposition.md` — tri-state MECHANICAL / JUDGMENT-LEANED / FORK-FOR-BEN. Sharpens consolidator outputs going forward.
3. **Candidate-4** — `feedback_pattern_induction_meta_finding_as_consolidator_output.md` — codifies the 5 categories of pattern-induction meta-findings (cross-lens TENSION + sibling-invariants + coherent-package + cascading-Compromise + coverage-gaps-self-critique).
4. **Candidate-7** — `dispatch-conventions §3.5t` — foundational-decision-class retrospective trigger. **MANDATORY for tuning.**

### Medium-priority (codify within next 2 sprints)
5. **Candidate-2** — Extension to `feedback_reviewer_composition` Pattern 6 lens-surface catalog (add: existing-code-readiness, permanence-3-axis-package, perf/wire-size, UX-coupling, CRDT-conflict-resolution, backwards-compat-migration, formal-methods-tractability, regulatory-jurisdiction-revisit-trigger).
6. **Candidate-5** — Extension to `feedback_extra_reflection_pass_for_elegant_permanent_shape` (add C1-C5 critique-round companion).
7. **Candidate-6** — `feedback_3_chain_cryptographer_sequencing.md` — sequential L1→L2→L3 chain discipline.
8. **Candidate-8** — `dispatch-conventions §3.6k` — backwards-compat-migration lens MANDATORY for frozen-or-near-frozen interfaces.

### Decision-class taxonomy
9. Add decision-class-explicit sizing taxonomy (§7) to Candidate-1 memo as appendix.

### Action items consolidator-MF4 already named
10. **SECURITY-POSTURE.md downstream-hazard-tree** — show Compromise #31's downstream-hazard cascade (#32 + #43 + #42 + recipient-rotation). Per consolidator MF4 action. C4 reaffirms.

---

## §9 Self-assessment + confidence

### What I did
1. Tree-state pre-flight on isolated agent worktree (`2172cb6d`).
2. Read full consolidated registry (939 lines @ `fbdfeb16`).
3. Did NOT reach into in-flight C1 elegant-shape + C2 composability sibling worktrees (locked + sibling-agent-occupied).
4. Verified all 9 lens-review SHAs resolve + noted timing pattern (L1→L2→L3 sequential, L4-L9 parallel).
5. Catalogued unique-value + redundancy + gap per lens (§2) using consolidated registry origin citations.
6. Evaluated Pattern 6 effectiveness (§3) including counterfactual reasoning for 3-cryptographer count.
7. Evaluated consolidator value-add (§4) distinguishing mechanical-organization vs NEW-analysis vs forward-skeleton.
8. Evaluated critique-round shape (§5) including self-reference + C4's own role.
9. Drafted 8 codification candidates (§6) with full memo/§-rule text + revisit-trigger.
10. Broader review-process discipline assessment (§7) with cost-asymmetry argument + 5-contributor counterfactual.

### Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §1 Executive verdict | HIGH on cost-justification + shape-RIGHT-but-naming-MISSING; MED-HIGH on 3-gap-claim novelty | Sequential-vs-parallel observation is C4-original; consolidator-tri-state is C4-original; retrospective-trigger is C4-original |
| §2 9-eyes catalog | HIGH | Derivable from consolidated-registry origin-citations + timing evidence + amendment-count |
| §3 Pattern 6 effectiveness | MED-HIGH | Counterfactual smaller-panel reasoning involves judgment; 3-cryptographer-count is HIGH-confidence; new-lens recommendations are MED-HIGH |
| §4 Consolidator value-add | HIGH | Direct evidence in consolidated-registry; MF1-MF5 vindicate consolidator-not-just-N+1-lens framing |
| §5 Critique-round shape | MED-HIGH | Self-reference + sibling-critique-rounds-in-flight; C4-add-C6 (cost-benefit retrospective) is MED novelty |
| §6 Codification candidates | HIGH on direction; MED-HIGH on exact memo text | Ben to refine wording; 8 candidates produce coherent set |
| §7 Broader discipline | MED-HIGH | Single-datapoint reasoning; 5-contributor counterfactual is speculative |
| §8 Recommendations | HIGH | Direct distillation of §6 |

### What I could be wrong about

1. **3-cryptographer-count = structural-min-and-max** — strong claim. A 4th cryptographer (e.g., a libcrux specialist) might have caught issues I'm not seeing. The counterfactual is unfalsifiable from single-datapoint.
2. **C6 cost-benefit retrospective lens** — proposed addition. Could overlap with §3.5t retrospective-trigger §-rule; might be redundant. Ben call.
3. **Pattern-6 standing lens-surface catalog expansions (7 new lens-surfaces)** — risk of catalog-bloat. Counter: catalog grows-never-shrinks; orchestrator picks per-decision; bloat-risk-low.
4. **Consolidator tri-state disposition** — proposed memo. Might be over-engineering for routine consolidations. Counter: only applied to F+-class consolidations per Candidate-1 threshold.
5. **Decision-class taxonomy (§7)** — proposed sizing per class. The boundaries (e.g., between Wire-format-freeze and Invariant-mint) are fuzzy; orchestrator may struggle to classify ambiguous decisions. Counter: HARD-RULE-no-deferral + plain-English-with-prediction discipline already handles edge-cases.

### Lower-confidence areas (honest disclosure)

- I did NOT read full L1-L9 lens-review texts individually; I derived per-lens unique-value-add from consolidator origin-citations + reaffirmed-lines. A direct re-read might surface findings the consolidator did not preserve in the registry (consolidator §9 explicitly notes "Some 'observation O1..O8' rows in L9 and similar in L8 are summarized but not preserved as separate registry entries.").
- I did NOT inspect C1 elegant-shape + C2 composability sibling-critique outputs (in-flight; worktree-locked). My critique-round-shape assessment (§5) reasons from charter not from actual output.
- Cost-asymmetry argument (§7) uses round numbers; actual cost-avoided lower-bound depends on whether one of the panel's 5+ load-bearing catches would have surfaced via other channels (e.g., post-tag external audit) at lower cost than I estimate.
- Retrospective trigger §3.5t and Candidate-7 — no prior precedent in Benten; novel discipline; may need iteration.

### What this critique does NOT cover

- Per brief: this is C4 process-discipline lens; I do NOT evaluate cryptographic substance (L1+L2+L3 are authoritative); I do NOT evaluate impl-engineering (L4); I do NOT evaluate audit-readiness (L5); I do NOT evaluate privacy (L6); I do NOT evaluate cross-ecosystem-interop (L7); I do NOT evaluate wire-format-stability (L8); I do NOT evaluate Atrium-integration (L9); I do NOT evaluate consolidator amendment-merge correctness at substance-level (consolidator §9 self-assessment is preserved).
- I do NOT propose new amendments or Compromises or invariants beyond what the consolidated registry contains.
- I do NOT evaluate C1 elegant-shape critique's specific findings (sibling critique-round; locked worktrees).

---

## §10 Citations

### §10.1 Primary input
- Consolidated registry: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` (939 lines).

### §10.2 Lens reviews (referenced via consolidator origin-citations; SHAs verified)
- L1 (`6d4e173f` Wed May 27 13:23:58 -0600) — generative-NO-GO + §6.2 sketch.
- L2 (`7e900a3b` Wed May 27 14:09:01 -0600) — foundation-amendment + Inv-16 primitive-neutral phrasing.
- L3 (`13b624c3` Wed May 27 14:42:46 -0600) — adversarial-calibration + Am3-Am6 + Compromise #32 (B-P).
- L4 (`4d4aae5f` Wed May 27 15:00:50 -0600) — impl-engineering + IMPL-A1-A6 + IMPL-B1-B4 + LE-vs-BE.
- L5 (`3f27f8e0` Wed May 27 15:03:49 -0600) — threat-model + audit-readiness + 11 Compromise mints + 3 invariants + THREAT-MODEL.md skeleton.
- L6 (`986e50bb` Wed May 27 15:01:36 -0600) — privacy + metadata-leak + Am7-Am12 + Sealed-Sender slot + Compromise #43.
- L7 (`208f98bb` Wed May 27 15:04:29 -0600) — cross-ecosystem-interop + Am9-Am10 + HPKE-11/-KE.
- L8 (`d8d3c41c` Wed May 27 15:02:58 -0600) — wire-format-stability + Am9-Am16 + CodepointLifecycle.
- L9 (`1670aa03` Wed May 27 15:06:08 -0600) — atrium-integration + A1-A5 + dual-CID disagreement.

### §10.3 Sibling critique-round (in-flight; not read; named-only)
- C1 elegant-shape — `phase-4-meta-core/option-f-plus-critique-c1-elegant-shape` (locked, sibling-occupied).
- C2 composability — `phase-4-meta-core/option-f-plus-critique-c2-composability` (locked, sibling-occupied).
- C3 fresh-eyes — branch existence presumed per brief.
- C5 formal-methods — branch existence presumed per brief.

### §10.4 Benten process discipline references
- `feedback_reviewer_composition` (Pattern 6 — base for §3 and Candidate-2 extension).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` (C1 critique-round member memo; base for Candidate-5 extension).
- `feedback_review_finding_ground_truth_verify` (ADVISORY-not-AUTHORITATIVE rationale; basis for Candidate-3 tri-state discipline).
- `feedback_pattern_induction_meta_sweep` (precedent for consolidator MF1-MF5 + Candidate-4 codification).
- `feedback_iterate_critical_reviews_to_convergence` (convergence discipline; relevant to critique-round termination).
- `feedback_canary_first_parallel_implementation` (precedent for sequential-then-parallel sequencing per Candidate-6).
- `feedback_no_defer_HARD_RULE` (HARD RULE 12; basis for forced-disposition discipline at consolidator level).
- `feedback_plain_english_surfaces` (basis for disagreement-matrix consumable shape).
- `feedback_phase_close_final_council_full` (Q5 amendment; basis for full-council convergence; relevant to critique-round full-coverage discipline).
- `feedback_pim_12_red_phase_staged_pin_un_ignore_discipline` (precedent for codified standing patterns).
- `.addl/dispatch-conventions.md` §3.5 (orchestrator pre-flight; basis for Candidate-7 §3.5t addition).
- `.addl/dispatch-conventions.md` §3.6 (review/fix-pass discipline; basis for Candidate-8 §3.6k addition).

### §10.5 External standards referenced
(Inherited from consolidated registry §10.3 + §10.4; not re-cited here; see consolidated registry for full external bibliography.)

---

**End of C4 process-discipline critique.**
