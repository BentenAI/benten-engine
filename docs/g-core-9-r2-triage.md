# G-CORE-9 FREEZE iterate-to-convergence council R2 triage

> **Scope:** R2 council (10 lenses) returned **2 BLOCKER + 10 MAJOR + 18 MINOR + 17 OBSERVATION** = 47 findings against `origin/main` @ `235ad861` (the R1 fix-pass PR #1346). Per CLAUDE.md rule 9 Q5 termination: 0 BLOCKER + 0 MAJOR is the convergence criterion; this round therefore iterates one more fix-pass. **Every R2 finding dispositions as a surgical doc edit or trivial code edit** (~150-300 LOC across ~10 sub-bundles). No new architectural concerns surfaced.
>
> **Date:** 2026-05-24 (under night-shift stance continuation; orchestrator-direct dispositions).
>
> **Status:** authoritative for the `g-core-9/r2-fix-pass` PR.

---

## 1. Executive summary

**Convergence shape — monotonic R1→R2:**

| Severity | R1 baseline | R2 result | Net delta |
|---|---|---|---|
| BLOCKER | 3 | 2 | -1 (R1 3 closed; 2 fresh from fix-pass doc-coupling drift) |
| MAJOR | 30 | 10 | -20 (R1 substantively closed; 10 carries / re-surfaced) |
| MINOR | 24 | 18 | -6 |
| OBS | 17 | 17 | unchanged (positive-control + forward-looking) |
| **Total** | **74** | **47** | **-27 (~37% reduction)** |

The R1 fix-pass closed the LOAD-BEARING work substantively: 12 of 12 bundles executed; 3 R1 BLOCKERs all resolved via doc-tighten + DEFERRED.md authoring + Compromise #26 retense. The R2 residuals are **honest-record + cite-drift + sweep-completeness** gaps, NOT regressions — exactly the shape Phase-2b R1→R2 convergence taught us to expect (front-loaded-convergence per pim-13 / §3.12). All 47 R2 findings are **surgically closable** (doc edits at known file:line targets + 1 GitHub branch-protection update + 1 workflow-bash bugfix + a handful of code edits ≤30 LOC apiece).

**Three structural cross-confirmed patterns** surfaced by ≥2 R2 lenses each:

| Pattern | Confirming lenses | Origin |
|---|---|---|
| **Phantom Row D-15 cluster** — V1-BETA-BREAKING-CHANGES.md Cohort 3 says "Bundle 4 captured in Row D-15 watch-list" but Row D-15 doesn't enumerate Bundle 4 | L8-R2-MAJOR-CARRY-1, L9-r2-MIN-2, L12-R2-MIN-1, L18 corroboration | Bundle 4 escalated; named destination phantom (HARD RULE 12 clause-b violation) |
| **Fork 3 branch-protection gap** — workflow flipped to required-failing at workflow-level, but GitHub branch-protection required_status_checks not updated AND the bash `\| tee` pipeline subshell loses `DRIFT_DETECTED=1` | L12-R2-MAJ-1 + L9-r2-MAJ-1 | Bundle 10 partial closure |
| **Doc-honesty residues** — V1-FROZEN-INTERFACE.md §4 verification-mechanism narrative + §8 §8-E hooks table + Compromise #23 retense missing + BREAKING-CHANGES.md Bundle 11 stale claim | L11-R2-MAJ-1, L6-r2-2, L2-R2-BLK-1, L2-R2-BLK-2 | Bundle 7/9/11 sweep incompleteness |

**Decisions made (orchestrator-direct under night-shift stance, all rebuttable at next morning):**
- Bundle R2.1 closes L2 BLKs by retensing BREAKING-CHANGES.md Bundle 11 line 178-181 to reflect revert + retensing Compromise #23 to disclose F3 in-window racy property.
- Bundle R2.2 closes phantom-destination cluster by appending explicit rows to DEFERRED.md for Strategy rename + 3 DSL ErrorCode mints.
- Bundle R2.3 closes Fork 3 second half via gh api PATCH to required_status_checks (already verified write-access succeeded during triage).
- Bundle R2.8 applies `#[non_exhaustive]` to the two wire-bytes-load-bearing benten-ivm types (TypedOutputProjection + KernelOutput) NOW, defers the rest to DEFERRED.md Row D-17 extension.
- Bundle R2.6 chooses doc-retense path for L17-r2-1 Spec attribute (private-fields-plus-builder already provides equivalent SemVer-safety; no need to apply attribute).
- Bundle R2.5 chooses **path (a)** for L1-crypto-r2-1 (route through `SwapMatrixError::Unsupported`).

## 2. Per-BLOCKER disposition

### L2-R2-BLK-1 — V1-BETA-BREAKING-CHANGES.md Bundle 11 line 178-181 stale claim
- **Class:** Doc-vs-code drift (BREAKING-CHANGES.md narrative diverges from engine.rs reality post-revert of commit 34053ed4 by 3d6f4d66).
- **Disposition:** FIX-NOW — retense Bundle 11 entry to reflect the deferral; cite engine.rs:1463-1477 inline comment + Row D-18 destination.
- **Bundle:** R2.1.

### L2-R2-BLK-2 — Compromise #23 retense promised by Row D-8 never landed
- **Class:** Doc-vs-doc drift (DEFERRED.md asserts SECURITY-POSTURE.md retense; the retense was never written).
- **Disposition:** FIX-NOW — actually add the F3 in-window racy disclosure paragraph to Compromise #23 (cite `crates/benten-caps/src/chain_authority.rs:404-421` non-atomic get+put across separate redb txns; honest narrative; mitigation = tight nbf/exp + atomic CAS land at G-COMP-1).
- **Bundle:** R2.1.

## 3. Cross-confirmed patterns across lenses

### Pattern α — Phantom Row D-15 cluster (HARD RULE 12 clause-b violation)
- **Cross-confirmed by:** L8-R2-MAJOR-CARRY-1 + L9-r2-MIN-2 + L12-R2-MIN-1 + L18-r2-1 partial.
- **Root cause:** Bundle 4 (Strategy::C → Reserved rename + 3 DSL ErrorCode mints) ESCALATED at R1 fix-pass; V1-BETA-BREAKING-CHANGES.md:152-156 named "Row D-15 watch-list" as the destination; Row D-15 enumerates 4 OBS-derived crypto items, NOT the Strategy rename + DSL mints.
- **Disposition:** FIX-NOW Bundle R2.2 — append a new Row D-19 to DEFERRED.md naming Strategy rename + 3 DSL ErrorCode mints + cargo-public-api baseline cascade + §3.5g 4-surface mirror sweep.

### Pattern β — Fork 3 second-half gap
- **Cross-confirmed by:** L12-R2-MAJ-1 (branch-protection update) + L9-r2-MAJ-1 (bash `| tee` pipeline subshell DRIFT_DETECTED bug + DSL baseline staleness).
- **Root cause:** Bundle 10 workflow flip at file-level was executed; the branch-protection inclusion was named as separate post-merge admin action; the bash subshell variable bug was not noticed.
- **Disposition:** FIX-NOW Bundle R2.3 — (a) gh api PATCH branches/main/protection/required_status_checks/contexts to add 'API drift detector (required-failing per G-CORE-9 FREEZE)' (VERIFIED writable during triage; landed); (b) restructure the workflow to write DRIFT_DETECTED to a tempfile that the parent shell reads after the pipeline.

### Pattern γ — Doc-honesty residues sweep
- **Cross-confirmed by:** L11-R2-MAJ-1 (§4 verification-mechanism narrative un-updated) + L6-r2-2 (§8 §8-E hooks table missing DEFERRED cross-ref) + L17-r2-1 (Spec attribute claim factually wrong) + L17-r2-2 (codepoint-dispatch overstatement at 15.c) + L10-r2-1 (bullet #3 phantom compile-test cite) + L11-R2-MINOR-1/2 (inventory phantom cites un-swept) + L11-R2-MINOR-3 (tf3d test-comment gate-14 phantom).
- **Root cause:** Bundle 7 doc-coupling sweep was scoped to V1-FROZEN-INTERFACE.md primary surface; missed several adjacent surfaces (WIRE-FORMAT-INVENTORY rows, test-comment cites, §4 narrative re-scope).
- **Disposition:** Bundles R2.4 + R2.6 + R2.7 + R2.10 distribute the doc-honesty sweep across the relevant docs.

## 4. Per-MAJOR disposition

| Lens | ID | Disposition | Bundle |
|---|---|---|---|
| L1 | L1-crypto-r2-1 (diagnostic-info loss persists) | FIX-NOW — route through `SwapMatrixError::Unsupported` (path a per R2 lens) | R2.5 |
| L1 | L1-crypto-r2-2 (codepoint.rs phantom-destination) | FIX-NOW — code-fix retense `benten_crypto_suite::codepoint` module-docstring + `benten_crypto_suite` lib.rs cipher-suite section to FROZEN state | R2.5 |
| L6 | L6-r2-1 (L6-r1-3 trybuild test no disposition) | FIX-NOW — add Row D-19 (or extension) deferring trybuild compile-fail test to G-COMP-1 (path b per Fork 2 — hard-seal IS enforced by rustc, only explicit negative-arm test deferred) | R2.7 |
| L6 | L6-r2-2 (§8 §8-E hooks table missing DEFERRED cross-ref) | FIX-NOW — symmetric retense to mirror §12's signature-frozen-vs-consumption-deferred distinction | R2.7 |
| L8 | L8-R2-MAJOR-CARRY-1 (Strategy rename phantom-destination) | FIX-NOW — Pattern α closure; new Row D-19 | R2.2 |
| L8 | L8-R2-MAJOR-CARRY-2 (#[non_exhaustive] sweep partial) | FIX-NOW — apply to TypedOutputProjection + KernelOutput (wire-bytes-load-bearing NON-DEFERRABLE); defer remainder to extended Row D-17 | R2.8 |
| L9 | L9-r2-MAJ-1 (cargo-public-api baseline + bash bug) | FIX-NOW — Pattern β closure; restructure workflow + regenerate baselines (baseline regen falls outside R2 scope — workflow fix lands now; baselines deferred to G-COMP-1 row) | R2.3 |
| L11 | L11-R2-MAJ-1 (§4 verification-mechanism narrative un-updated) | FIX-NOW — retense to Bundle 5 PARTIAL + cite Row D-9 | R2.4 |
| L12 | L12-R2-MAJ-1 (branch-protection gap) | FIX-NOW — gh api PATCH (Pattern β closure; VERIFIED applied) | R2.3 |
| L18 | l18-r2-1 (3 missing Cohort 3 rows) | FIX-NOW — append 3 ledger rows | R2.10 |

## 5. MINORs + OBSs grouped

### MINORs (18)

| Lens | ID | Disposition | Bundle |
|---|---|---|---|
| L1 | L1-crypto-r2-3 (`benten_crypto_suite` lib.rs Reserved-codepoints + cipher-suite sections stale framing) | FIX-NOW — retense ~15 LOC across the 3 sub-sections | R2.5 |
| L1 | L1-crypto-r2-4 (sub-pins b + c for resolve_codepoint + UcanVarsigV1Header::decode) | FIX-NOW — extend canonical_bytes_v1 test with 2 more assertion blocks (~25 LOC) | R2.5 |
| L6 | L6-r2-3 (audit-test SuspensionOutcome + NextChunkPoll arm coverage) | FIX-NOW — add 2 audit-arm tests | R2.7 |
| L6 | L6-r2-4 (V1-FROZEN cite drift to policy.rs:179 + companion cites) | FIX-NOW — refresh cites: actor_hint 167→179; PendingOp 247→100; CapWriteContext 103→163; ReadContext 154→260 | R2.7 |
| L8 | L8-R2-MINOR-CARRY-1 (audit-test workspace-walker spec-vs-impl drift) | FIX-NOW (path b per L8) — retense V1-FROZEN.md:951-957 narrative to describe actual enumerated-per-type shape; defer workspace-walker upgrade to extended Row D-17 | R2.8 |
| L9 | L9-r2-MIN-1 (EDslIoError + EDslBackendRejected re-export gap) | FIX-NOW — 4-line edit to packages/engine/src/errors.ts | R2.9 |
| L9 | L9-r2-MIN-2 (DEFERRED.md Row D-15 phantom for Strategy + DSL mints) | FIX-NOW — Pattern α (Row D-19 close) | R2.2 |
| L9 | L9-r2-MIN-3 (audit-test 5 DSL types missing) | FIX-NOW — add 5 doc-comment audit tests | R2.9 |
| L10 | l10-r2-1 (bullet #3 phantom compile-test cite) | FIX-NOW — orchestrator-direct 4-line edit consolidating bullet #3 into cargo-public-api defense | R2.10 |
| L11 | L11-R2-MINOR-1 (4 phantom file cites in WIRE-FORMAT-INVENTORY rows 1/5/7/10) | FIX-NOW — sweep | R2.4 |
| L11 | L11-R2-MINOR-2 (Summary table row 10 stale cite) | FIX-NOW — single-line edit | R2.4 |
| L11 | L11-R2-MINOR-3 (tf3d test-comment gate-14 phantom) | FIX-NOW — single-line edit | R2.4 |
| L11 | L11-R2-MINOR-4 (SHA2_512_256 + SHA3_256 codepoint mint deferral) | BELONGS-NAMED-NOW — Row D-15 extension | R2.4 |
| L12 | L12-R2-MIN-1 (Bundle 4 named-destination drift) | FIX-NOW — Pattern α (Row D-19 close) | R2.2 |
| L17 | L17-r2-1 (Spec attribute claim factually wrong) | FIX-NOW path b — retense doc claims at 15.a:1263 + table row 916 to acknowledge private-fields-plus-builder pattern provides equivalent SemVer-safety | R2.6 |
| L17 | L17-r2-2 (codepoint-dispatch overstatement at 15.c) | FIX-NOW path a — downgrade language to "serde-tag dispatch" | R2.6 |
| L18 | l18-r2-2 (Row D-7 missing from Cohort 4 / Cohort 5) | FIX-NOW — add Cohort 5 "Public-API DEFERRED" section | R2.10 |
| L18 | l18-r2-3 (crypto-suite INTERNALS.md soft-defer) | FIX-NOW — add concrete Row to DEFERRED.md per the L18 lens recommendation | R2.10 |
| L18 | l18-r2-4 (V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md missing) | FIX-NOW path (b) per L18 lens — retense V1-FROZEN-INTERFACE.md item 4 + build-backlog row 8.f to acknowledge the inventory IS the Ben-decision deliverable | R2.10 |

### OBSs (17 — positive-control + forward-looking)

| Lens | ID | Disposition |
|---|---|---|
| L1 | L1-r2-approve-1 through -5 | OUT-OF-SCOPE: positive evidence preserved |
| L2 | L2-R2-OBS-1 (0x6400 SECURITY-POSTURE.md disclosure gap) | OUT-OF-SCOPE-DEFER — Row D-15 already covers via require_hybrid_pq item; observation-level relative to BLKs |
| L2 | L2-R2-OBS-2 (Row D-18 honest) | OUT-OF-SCOPE: positive evidence |
| L2 | L2-R2-OBS-3 (Compromise #26 retense gold-standard) | OUT-OF-SCOPE: shape reference for L2-R2-BLK-2 Compromise #23 retense |
| L6 | L6-r2-5 through -10 | OUT-OF-SCOPE: positive evidence verify-stays |
| L9 | L9-r2-OBS-1, -2 | OUT-OF-SCOPE: positive evidence |
| L11 | L11-MINOR-4 brief-drift logged | OUT-OF-SCOPE — note for R3+ |
| L12 | L12-R2-OBS-1 (§3.5j feature-flag wording amendment) | OUT-OF-SCOPE-LIGHT — orchestrator may amend dispatch-conventions.md post-R2 if pattern recurs; non-R2-blocking |
| L17 | L17-r2-3 (triage-narrative accuracy) | OUT-OF-SCOPE: orchestrator handoff note (the substantive work IS correct) |
| L17 | L17-r2-4 (Fork 2 verify-pass pass) | OUT-OF-SCOPE: positive evidence |
| L18 | l18-r2-5/6/7 | OUT-OF-SCOPE: positive confirmation + forward-looking |

---

## 6. Bundle execution plan

| # | Bundle | Touches | Status |
|---|--------|---------|--------|
| R2.1 | L2 BLOCKERs — doc-coupling drift | V1-BETA-BREAKING-CHANGES.md:178-181 + SECURITY-POSTURE.md Compromise #23 | FIX-NOW |
| R2.2 | Phantom Row D-15 cluster — Pattern α | V1-FROZEN-INTERFACE-DEFERRED.md (new Row D-19) + V1-BETA-BREAKING-CHANGES.md:152-156 cite | FIX-NOW |
| R2.3 | Fork 3 second half — Pattern β | gh api PATCH (DONE during triage) + .github/workflows/cargo-public-api.yml subshell bugfix | FIX-NOW |
| R2.4 | L11 wire-format doc retenses | V1-FROZEN-INTERFACE.md:389-401 + WIRE-FORMAT-INVENTORY 4 rows + tf3d test-comment + Row D-15 SHA-fallback addition | FIX-NOW |
| R2.5 | L1 crypto-correctness residues | `benten_crypto_suite::swap_matrix` 2 resolve-failure call sites + `unsupported_codepoint_msg_static` helper delete + `benten_crypto_suite::codepoint` module-docstring + `benten_crypto_suite` lib.rs cipher-suite section + canonical_bytes_v1 test extension | FIX-NOW |
| R2.6 | L17 §1.A.FROZEN item 15 residues | V1-FROZEN-INTERFACE.md:1263 + :916 + :1363-1366 | FIX-NOW |
| R2.7 | L6 capability + plugin-trust residues | V1-FROZEN-INTERFACE.md §8 + DEFERRED.md Row D-19 (trybuild defer) + audit-test extension + cite refresh | FIX-NOW |
| R2.8 | L8 IVM #[non_exhaustive] sweep extension | subgraph_spec.rs:86+296 (TypedOutputProjection + KernelOutput) + audit-test arm + DEFERRED.md Row D-17 extension + V1-FROZEN.md:951-957 retense | FIX-NOW |
| R2.9 | L9 DSL R2 residues | packages/engine/src/errors.ts + audit-test 5 DSL types | FIX-NOW |
| R2.10 | L10 + L18 + remaining MINORs sweep | V1-FROZEN-INTERFACE.md:1137-1139 + V1-BETA-BREAKING-CHANGES.md (3 rows + Cohort 5) + DEFERRED.md INTERNALS.md row + retense item 4 | FIX-NOW |

## 7. Expected R2→R3 convergence

Per the iterate-to-convergence Q5 amendment (CLAUDE.md rule 9; `feedback_iterate_critical_reviews_to_convergence`): R3 dispatches the full 10-lens council against the post-R2-FP state. **Expected R3 finding count: 0-2 MAJORs + ≤5 MINORs** (front-loaded-convergence shape per Phase-2b R1→R2→R3 precedent; each lens went R1=many → R2=few → R3≈0 substantive). Worst-case: a sweep-completeness miss surfaces 1 MAJOR which is closable in a single R3-FP commit. Convergence-shaped path: R3 0-substantive → APPROVE-FOR-TAG → pre-tag sweep → tag `phase-4-meta-core-close` (item 9 v1-beta freeze).

---

## Provenance

Authored at G-CORE-9 R2 fix-pass (this PR; 2026-05-24) per R2 council 10 lens JSONs. R2→R3 expected via the Q5 cadence + iterate-to-convergence amendment.
