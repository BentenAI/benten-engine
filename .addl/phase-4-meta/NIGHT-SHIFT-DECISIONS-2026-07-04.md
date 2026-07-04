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

<!-- append D-N entries as autonomous decisions arise in R16/R17/fixes -->
