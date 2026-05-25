# feedback_pim_n_regression_guard_substantive_arm

**Status:** STAGED — orchestrator move this file to
`~/.claude/projects/-Users-benwork-Documents-benten-engine/memory/feedback_pim_n_regression_guard_substantive_arm.md`
during R6-R2-FP-D consolidation.

**Ratified:** 2026-05-25 (Phase-4-Meta-Core R6 R2 FP-D)
**Origin:** L2-R6-R2-MAJOR-2 + L9-r6r2-MINOR-3 + L13-MAJ-1 + L13-MIN-4 + L13-MIN-5 (16 instances across one wave's regression test family)
**Codification:** `.addl/dispatch-conventions.md §3.6f` sub-rule extension (a)-(e)
**See-also:** `[[feedback_pim_18_shape_not_substance_pre_flight]]` (parent — original §3.6f), `[[feedback_end_to_end_test_pin_for_closed_claims]]` (pim-2 / §3.6b — wave's substantive closure arms), `[[feedback_pim_12_red_phase_staged_pin_un_ignore_discipline]]` (pim-12 / §3.6e — RED-PHASE pin un-ignore)

## The rule (one-line)

Regression-guard tests minted to pin a wave's CLOSED findings against
future revert MUST invoke a production entry point, assert an observable
consequence, demonstrate would-FAIL-on-revert in the commit body, and
NEVER use `assert_eq!(CONST, CONST_VAL)` walker-shape or zero-assertion
`#[test]` arms.

## Why this needs its own pim-N entry

Pim-18 §3.6f original framing was a SHAPE-not-SUBSTANCE pre-flight for
ALL new test pins generally. Pim-2 §3.6b targets the wave's substantive
closure arms (the test that exercises the bug-being-closed end-to-end).
This new pim-N narrows specifically to the **regression-guard test
family** — the supplementary pins minted alongside the substantive
closure arms whose job is forward-protection-against-revert.

In R6 R2, the F4 wave's substrate-consumer-wire-in landed 6 SHAPE-not-
SUBSTANCE regression-guards uniformly across S1+S2+S3a+S3b+S3c+S4 — a
uniform-failure pattern across a single wave's regression test family.
The mini-review APPROVE flow did not catch this because each individual
test compiled and passed; only deep-verification at R6-R2 surfaced the
class. The pim-N strengthening narrows the SHAPE-not-SUBSTANCE rule
specifically to regression-guards (where the revert-detection contract
is load-bearing) so the antipattern is forbidden, not merely cautioned.

## Four anti-patterns this rule rejects

1. **Trait-isolation:** `policy.check_per_delegation(...)` called
   directly on a hand-crafted `CapabilityPolicy` impl, NOT through
   `engine.caps().delegate_capability(...)`. Reverting the engine-side
   wire-in (engine_caps.rs:572-590) leaves the trait-direct test
   passing — the production boundary is structurally untested.

2. **Sentinel-presence:** `assert_eq!(ErrorCode::PluginPerDelegationDenied.as_str(), "E_PLUGIN_PER_DELEGATION_DENIED")`
   asserts the const exists with expected name. Reverting the wire-in
   does NOT change the const's value, so this assertion stays green
   on revert.

3. **Const-tautology workspace-walker:**
   ```rust
   const EXPECTED_WRITE_SITES: usize = 13;
   assert_eq!(EXPECTED_WRITE_SITES, 13);
   ```
   The lhs IS the const; the rhs IS its declared literal. No
   possible no-op causes this to fail. The substantive shape is a
   real source-walker that counts production occurrences via
   `fs::read_dir` + `fs::read_to_string` (precedent:
   `tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs`).

4. **Zero-assertion `#[test]` forensic anchors:**
   ```rust
   #[test]
   fn ucan_grounded_excluded_per_delta_v3_7() {
       // Comment-only forensic anchor; no assertion body.
   }
   ```
   Registers as PASS-by-vacuity in nextest output. Either convert
   to module-level `//!` documentation prose (preferred) OR write a
   substantive assertion that exercises the load-bearing semantic
   (e.g. source-scan asserting `ucan_grounded.rs` policy calls are
   `#[cfg(test)]`-gated).

## The would-FAIL-on-revert demonstration discipline

Each rewritten regression-guard test in this PR was verified via:

```bash
cp <production-source>.rs /tmp/<source>.rs.bak
# Mutate production source (delete wire-in / rename method / etc.)
cargo nextest run -p <crate> --test <name>
# Observe FAIL output
cp /tmp/<source>.rs.bak <production-source>.rs
git status  # verify clean restore
```

The PR commit body MUST cite the specific mutation + the FAIL
diagnostic, so future reviewers can replay the demonstration.

## Trigger conditions (when to apply this rule)

- Any wave that mints **regression-guard tests** to pin a CLOSED
  finding against future revert (the typical FP-cycle output).
- 3+ regression-guards in the same wave's test family — sets the
  expectation that the pattern (not the individual case) is the
  thing being audited.
- Any test name containing `_audit` / `_wired_at_` / `_consults_` /
  `_routes_through_` / similar revert-detection semantic — these
  names PROMISE forward-protection; the body must deliver.

## Failure mode this prevents

A future agent reverts a production wire-in (e.g. accidentally during
a refactor) → regression-guard tests stay green (because they were
trait-isolated / sentinel-presence / tautology / empty-body) → CI
green → reverted-wire-in lands → security regression in production.
Pim-18 §3.6f original framing reduced incidence; this strengthening
makes the regression-guard test family the SPECIFIC target of the
rule (where the forward-protection contract is the test's reason for
existing).

## Cross-link map

```
feedback_pim_18_shape_not_substance_pre_flight  ← parent (§3.6f original)
       └─ feedback_pim_n_regression_guard_substantive_arm  ← THIS (§3.6f sub-rule)
       └─ feedback_end_to_end_test_pin_for_closed_claims    ← sibling (§3.6b pim-2)
       └─ feedback_pim_12_red_phase_staged_pin_un_ignore_discipline  ← sibling (§3.6e pim-12)
```

The sibling rules cover different parts of the wave's test envelope:
pim-2 = the wave's substantive closure arms; pim-12 = un-ignoring
RED-PHASE pins; THIS = the wave's regression-guards. Together they
form the 3-track test-substance discipline.
