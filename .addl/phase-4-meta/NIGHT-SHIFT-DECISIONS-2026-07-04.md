# NIGHT-SHIFT DECISION LOG — 2026-07-04 (Phase-4-Meta-Core R6 phase-close convergence)

**Authorization (Ben, 2026-07-04):** full night-shift stance. Proceed autonomously; make decisions AS BEN; take the extra reflection-pass on each real decision (consider the options again + the final permanent shape); keep this decision log; **do NOT stop to wait for input.** If Ben disagrees with anything in the morning, that's fine (reversibility accepted). Foundational rules still override (HARD RULE 12, ground-truth-verify §3.5n, plain-English lens, etc.). See memory `feedback_night_shift_stance` + `feedback_surface_arch_decisions_under_auth`.

**Held for Ben's morning explicit go (NOT autonomous — permanent/outward-facing milestone actions):**
1. **Merging #1382 to main** (the phase-close landing).
2. **Creating the tag `phase-4-meta-core-close`** (the permanent milestone; standing law = tag Ben-gated throughout).
Everything UP TO those (all rounds, fixes, ground-truthing, re-gates, merges into r9-base, pushes to the r9-base PR branch) proceeds autonomously.

**Convergence rules in effect (Ben-ratified 2026-07-04, memory `feedback_two_consecutive_converged_and_minor_every_round`):**
- (1) Tag needs TWO consecutive CONVERGED rounds (0 confirmed BLK/MAJ). Any NOT-CONVERGED resets the count.
- (2) Every round closes ALL minor/obs (fix-now) + names all deferrals (HARD-RULE clause-b), not just BLK/MAJ.

**Decision framework (per Ben's instruction — apply to each autonomous decision):** (a) ground-truth the finding/situation in code myself (§3.5n); (b) synthesize the options; (c) reflect AGAIN — is there a more elegant final permanent shape? (`feedback_extra_reflection_pass_for_elegant_permanent_shape`); (d) decide as-Ben (do-it-right, do-it-now, wire-properly, no-defer-bias per `feedback_orchestrator_defer_prediction_bias`); (e) log it here; (f) proceed.

---

## Decisions

### D-1 (2026-07-04) — R15 F-01 fix shape: segment-boundary guard (CONVERGENT, executed)
UCAN attenuation `caps_match_or_subsume` prefix-confusion authority-widening. Options: (a) segment-boundary guard [textbook], (b) exact-match-only [would break legit sub-path delegation], (c) canonicalize-then-compare [heavier]. Reflection: (a) is the minimal correct fix that preserves the intended path-prefix semantics (`/zone/posts` covers `/zone/posts/foo`) while closing the sibling-confusion (`/zone/posts-secret`). Boundary chars `/` (path) + `:` (scope) match the resource grammar. DECIDED (a). Executed in e57c9a29, verified sound + would-FAIL-on-revert at the production entry. (This was pre-authorization but logged for continuity.)

### D-2 (2026-07-04) — R15 F-09: rename InnerSenderDidForged→MalformedInnerPayload (not collapse) (executed)
The self-contradictory variant (docstring said "NOT a forgery"). Options: rename vs collapse-to-AeadAuthenticationFailed. Reflection: collapse would lose the structural-decode-vs-AEAD-fail diagnostic distinction (a malformed inner payload is a different failure than a MAC failure); the variant is internal-only (no napi/wire boundary) so a rename is non-breaking. DECIDED rename. Executed e57c9a29.

### D-3 (2026-07-04) — Held for Ben: #1382 merge + tag creation
Per standing law + night-shift-surfaces-milestones: the phase-close merge-to-main + the permanent tag are held for Ben's morning explicit go, even as everything else proceeds autonomously. Rationale: both are hard-to-reverse / outward-facing milestone actions; the tag has been Ben-gated throughout the phase.

### D-4 (2026-07-04, ~01:50 EDT) — R15 fix landed → r9-base advanced → ROUND 16 launched (autonomous, within scope)
Post-compact resume. Ground-truthed the live state first (§3.5n / session-start context pass): disk 79% (healthy, well clear of OOM band), worktrees clean, SHAs matched the resume contract (r9-base @ 5d31406c, r15-fix @ e57c9a29). Actions taken autonomously (all internal/reversible — NOT the two held-for-Ben milestones):
- **Confirmed the full re-gate `b1h763goq` on e57c9a29 GREEN:** fmt/clippy/ws-check PASS · nextest-crates 2048/2049 (sole failure = the known `in_memory_observationally_equivalent_to_redb` slow-under-parallel-load proptest) · nextest-engine 954/954 · cargo-doc -D warnings PASS · cite-drift clean · §3.5g PASS.
- **Ground-truthed the redb flake:** ran it isolated (`--test-threads 1`) → **PASS at 88.9s** (< the 180s timeout that only trips under parallel load). Confirmed it is the harness-parallelism flake, not a code defect.
- **FF-merged r9-base → e57c9a29** (clean fast-forward; r15-fix was exactly one commit ahead) + **pushed origin/phase-4-meta-core/r9-base** (refreshes #1382 CI = the independent full verifier). r9-base now backs the R15-fixed freeze base.
- **Launched ROUND 16** (task `w3plnshpc` / run `wf_096a90b8-185`): fresh convergence attempt on e57c9a29, all 20 standing lenses (incl `secret-lifetime-and-memory-hygiene` + `capability-authority-ucan-layerd`) + the benten-id/benten-graph enumeration that surfaced F-01. R16 framing adds a **prefix-confusion-class sweep mandate** to the capability-authority lens (hunt EVERY raw `starts_with`/substring authority comparison, not just the fixed site).

**Two-consecutive counter = 0.** R16 is attempt 1 of the required 2. R16 CONVERGED → count=1 → R17; R17 CONVERGED → count=2 → TAG-READY (hold for Ben). Any NOT-CONVERGED resets to 0. Every round closes the FULL minor/obs tail.

### D-5 (2026-07-04, ~02:45 EDT) — ROUND 16 = CONVERGED (count=1); F-01/F-02 ex-MAJORs ground-truthed to OBS; pub(crate)-tighten DEFERRED not forced pre-tag
Council ROUND 16 (Task `w3plnshpc`, 19/19 panel) on e57c9a29 → **CONVERGED: 0 confirmed BLOCKER, 0 confirmed MAJOR.** Two findings entered as MAJOR and were adversarially refuted to OBS; I ORCH-GROUND-TRUTHED both myself (§3.5n):
- **F-01 (MAJOR→OBS):** claim was `StructuralKdfKey::from_bytes_for_test` is "frozen on the public API under a false exemption / zero production callers." FALSE premise — VERIFIED a **live production caller**: `redb_backend::put_node_with_context` (ungated `pub fn`) seal path at `redb_backend.rs:1692` calls `derive_test_seam_key_from_cid_with_namespace` → `from_bytes_for_test` (:183), gated only on the RUNTIME `ctx.namespace_did.is_some()`, NOT `#[cfg]`. cfg-gating it would break the default-feature build. This is the honestly-documented **Compromise #65** wave-3e publicly-derivable-K_principal stand-in (doc comment redb_backend.rs:150-159 + 1684-1691). Real residual = the EXEMPT_PUB_ITEMS justification COMMENT names the wrong caller (says benten-drop's bundle pipeline; benten-drop actually uses `GrantKeyMaterial::from_bytes_for_test`, not `StructuralKdfKey`). → OBS, comment fix.
- **F-02 (MAJOR→OBS):** `layer_c_and_d_share_one_hpke_primitive() -> bool { true }` (envelope.rs:159) is a tautological pin; substrate reuse is genuinely real (both layers route through `benten_crypto_suite::hpke`). Test-strength nit, zero security impact. → OBS.
- **F-04 (MINOR, CONFIRMED):** `git grep rotationlog` in benten-drop = ZERO hits, yet SECURITY-POSTURE #43 (L2910) claims sender-key rotation "enforced live by benten-id RotationLog... inside verify_m_auth." Real doc-overclaim: the self-certifying did:key path (v1-beta default; DID = key) needs no rotation lookup; RotationLog applies to rotatable methods resolved out-of-band. → doc fix.

**Extra-reflection-pass decision — DEFER the pub(crate) tighten (D-5 row), do NOT force it pre-tag.** `derive_test_seam_key_from_cid_with_namespace` is benten-graph-internal (only the in-crate integration test `tf3d` calls it via the public path); a `pub`→`pub(crate)` tighten would remove it from the frozen surface. BUT: (a) it is a freeze-surface visibility change best bundled with the #1301/#989 K_principal-store swap-in that REPLACES the whole seam (the swap "without changing this call site" per the comment); (b) the crypto-suite-side `StructuralKdfKey::from_bytes_for_test` is genuinely cross-crate-pub (called from benten-graph production) and CANNOT be tightened, so a benten-graph-only tighten wouldn't fully close the "_for_test on frozen API" smell; (c) doing a visibility+baseline-regen change pre-tag with R16 already CONVERGED risks a needless NOT-CONVERGED reset. **The naming smell resolves NATURALLY when the transitional stand-in is replaced — that is the elegant permanent shape.** Named as a deferred hardening row in `V1-FROZEN-INTERFACE-DEFERRED.md` alongside D-64/#1301.

**Rule-2 action:** dispatched ONE consolidated doc/comment/CI fix-pass agent (branch `phase-4-meta-core/r16-doc-fixpass` off e57c9a29) to close the FULL minor/obs tail — ~11 fix-nows (F-01/F-02/F-03/F-04/F-05/F-06/F-07/F-08/F-13/F-14 + GAP-A..E carve-out sentences) + name the 7 carries (F-09/F-10/F-11/F-12/F-15/F-16/F-17) + my D-5 row. ALL prose/comment/CI/doc — zero code-behavior/wire/public-API-signature change. ON RETURN: orch ground-truth every edit → re-gate → FF-merge r9-base → push → **ROUND 17** (the 2nd consecutive-CONVERGED attempt) with the 2 council-recommended added missed-lenses (`availability-dos-resource-exhaustion` + `authorization-enforcement-semantics`, both verify-the-deferral-boundary lenses).

<!-- append D-N entries as autonomous decisions arise in R16/R17/fixes -->
