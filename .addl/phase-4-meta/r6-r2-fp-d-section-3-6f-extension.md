# §3.6f Strengthening — R6-R2-FP-D ratification (2026-05-25)

**Status:** STAGED-FOR-ORCHESTRATOR — apply to `.addl/dispatch-conventions.md §3.6f`
during Strategy-C consolidation of PR-D's branch
`phase-4-meta-core/r6-r2-fp-d-shape-not-substance`.

**Trigger:** L2-R6-R2-MAJOR-2 (6 F4 regression-guards uniformly SHAPE-not-
SUBSTANCE) + L9-r6r2-MINOR-3 (`assert_eq!(CONST, "CONST")` tautology) +
L13-MAJ-1 (18-of-34 F4 tests trait-isolated) + L13-MIN-4 (workspace-walker
const-tautology in S1 arm 3 + S3c arm 3) + L13-MIN-5 (empty-body forensic-
anchor #[test] arms). 4+-recurrence pattern within a single wave's regression
test family establishes the strengthening threshold.

**Origin instances all CLOSED in this PR** (per §3.6h — ratification MUST
close origin; §3.6j sweep-completeness self-verify):

| Site | Pre-PR shape | Post-PR shape |
|---|---|---|
| `r6_r1_fp_f4_s1*.rs::workspace_walker_audit_thirteen_admit_write_chain_call_sites` (L100-103) | `const EXPECTED = 13; assert_eq!(EXPECTED, 13)` (tautology) | Real `fs::read_dir(crates/benten-engine/src/)` walker counting `.admit_write_chain(` consumer call-sites excluding the `pub(crate) fn` definition; revert-detection verified by commenting out one production site → count drops to 12 → FAIL |
| `r6_r1_fp_f4_s3c*.rs::workspace_walker_audit_four_check_write_with_audience_sites` (L106-109) | `const EXPECTED = 4; assert_eq!(EXPECTED, 4)` (tautology) | Real source-walker counting `.check_write_with_audience(` consumer call-sites |
| `r6_r1_fp_f4_s3c*.rs::ucan_grounded_excluded_per_delta_v3_7` (L130-134) | Empty `#[test]` body (zero-assertion forensic anchor) | Substantive: reads `crates/benten-caps/src/ucan_grounded.rs`; finds all `policy.check_write*` hits; asserts each is `#[cfg(test)]` / `#[test]` / `mod tests`-gated |
| `r6_r1_fp_f4_s3b*.rs::delegate_capability_surfaces_plugin_per_delegation_denied_code` (L69-77) | Trait-direct `assert_eq!(ErrorCode::PluginPerDelegationDenied.as_str(), "...")` | Builds real `Engine` with denying `CapabilityPolicy`; mints user-rooted grant; invokes `engine.caps().delegate_capability(...)`; asserts typed `PluginPerDelegationDenied` surfaces at production boundary + observation counter incremented. Revert-detection verified: commenting wire-in at `engine_caps.rs:572-590` → test FAILS with diagnostic naming engine_caps.rs:572-590 |
| `r6_r1_fp_f4_s3b*.rs::check_per_delegation_observes_invocation` (L80-89) | Trait-direct `policy.check_per_delegation(...)` × 2 + counter assert | Real engine + admitting policy + `delegate_capability` call; asserts counter ≥1 + observed-source/target/scope forwarding contract |
| `r6_r1_fp_f4_s3b*.rs::check_per_delegation_returns_err_on_denial` (L93-100) | Trait-direct deny + `result.is_err()` | Two engines differing only in policy deny-flag; admit returns Ok(cid); deny returns Err; both consult hook |
| `r6_r1_fp_f4_s3b*.rs::default_check_per_delegation_admits_all` (L106-115) | Trait-direct `NoAuthBackend.check_per_delegation(...).is_ok()` | Real engine with `NoAuthBackend` policy; `delegate_capability` admits end-to-end (default trait impl returns Ok) |
| `r6_r1_fp_f4_s3b*.rs::check_per_delegation_is_callable_on_arc_dyn_capability_policy` (L126-130) | Trait-direct `Arc<dyn CapabilityPolicy>.check_per_delegation(...)` | Real engine + private-namespace ordering pin: a `private:*` scope short-circuits at Step-2a with `PluginPrivateNamespaceDelegationForbidden` BEFORE the policy hook fires — reverting the ordering surfaces `PluginPerDelegationDenied` instead |
| `r6_r1_fp_f4_s3a*.rs::check_install_consent_observes_invocation_count` (L52-65) | Trait-direct `policy.check_install_consent(...)` × 2 + counter assert | Real `install_plugin` pipeline + counting policy threaded via `InstallPorts.policy`; asserts counter ≥1 + canonical signing-payload BLAKE3 forwarded (never all-zero) + `did:key:` plugin_did forwarded |
| `r6_r1_fp_f4_s3a*.rs::deny_all_install_consent_returns_typed_code` (L71-75) | Trait-direct `DenyAllInstallConsent.check_install_consent(...)` returns typed code | Real `install_plugin` with `DenyAllInstallConsent` policy; asserts pipeline returns `Err(PluginInstallConsentDenied)` end-to-end |
| `r6_r1_fp_f4_s3a*.rs::check_install_consent_receives_install_record_plugin_did_string` (L82-111) | Trait-direct custom-deny policy | Real `install_plugin` with substring-deny policy that never matches; admits + verifies forwarded plugin_did is well-formed `did:key:` |
| `r6_r1_fp_f4_s3a*.rs::admit_all_install_consent_admits_every_hash` (L117-129) | Trait-direct `AdmitAllInstallConsent.check_install_consent(...) × 2` | Real `install_plugin` with `AdmitAllInstallConsent` policy; asserts pipeline admits end-to-end |
| `r6_r1_fp_f4_s3a*.rs::install_consent_hook_fires_before_cap_cascade_documented` (L159-179) | Empty `#[test]` body (zero-assertion forensic anchor) | Substantive ordering pin via observable side-effect: deny at step 3c → `library.is_empty()` AND `cascade.minted_grants().is_empty()` AND `cascade.provisioned_count() == 0` (Step 9 cap-cascade never reached) |
| `r6_r1_fp_f4_s4*.rs::production_engine_builder_open_in_memory` (L29-33) | Builder open only (no behavioral assertion) | Builder open + direct construction of `ProductionManifestEnvelopeRechecker` + load-bearing synthesized-fallback semantic assertion (`recheck_row("node-id:42", ...) == UnresolvedDeny`) — the type the builder commits to wire in at production_engine_builder.rs:78 |
| `r6_r1_fp_f4_s2*.rs::counting_replay_check_observes_invocation_at_install_time` (L88-101) | Trait-direct closure invocation + counter assert | Real `install_plugin` pipeline + counting closure threaded via `InstallPorts.install_record_replay_check`; asserts step 3b consulted closure with canonical signing_payload BLAKE3 (never all-zero) |
| `tf_dsl_chunk3_closures.rs::dsl_839_error_code_mirror_matches_typescript_constant_literal` (L254-261) | `assert_eq!(E_DSL_BACKEND_REJECTED, "E_DSL_BACKEND_REJECTED")` (self-equality tautology) | Reads `packages/engine/src/errors.generated.ts` at test time; parses `EDslBackendRejected.code = "..."`; asserts equality with Rust const |

## Proposed §3.6f extension text

Insert the following sub-rule block at the end of the existing §3.6f
text (after the SHAPE-not-SUBSTANCE pre-flight enumeration). The 4
clauses operationalize "SHAPE vs SUBSTANCE" for **regression-guard
tests specifically** — the test family minted to pin a wave's CLOSED
findings against future revert.

```markdown
### §3.6f sub-rule: regression-guard test substantive-arm contract

Regression-guard tests for any FP-cycle (waves that close prior R*-
review findings) MUST satisfy ALL of:

(a) **Production entry point invocation.** The test body MUST invoke
the public-API surface where the original bug manifested — NOT a
trait method called directly on a hand-crafted impl. For policy
hooks: route through `engine.caps().delegate_capability(...)` /
`install_plugin(...)` / `engine.create_node(...)` / equivalent —
not `policy.check_per_delegation(...)` / `policy.check_install_consent(...)`
in isolation.

(b) **Observable consequence.** The assertion MUST observe a
behavioral consequence of the wire-in's existence — database state,
typed error fire, event broadcast, counter increment, panic. NOT a
sentinel constant value, NOT a type-level coercion check, NOT a
self-equality literal comparison.

(c) **Demonstrated would-FAIL-on-revert.** The commit body MUST
record the revert demonstration: `git stash`-equivalent the
production wire-in (or rename/comment the call-site), rerun the
test, capture the FAIL output, restore. Without this evidence the
test's substantive-arm claim is unverified.

(d) **NEVER `assert_eq!(N, N)` walker shape.** The const-tautology
anti-pattern — declaring `const EXPECTED_SITES: usize = N;` then
`assert_eq!(EXPECTED_SITES, N)` — is forbidden. If counting
production sites for a forward-protection pin, walk the source
filesystem (`fs::read_dir` / `fs::read_to_string`) and count
actual call-site occurrences. The
`tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs`
scanner is the canonical precedent for the source-walker shape.

(e) **NEVER zero-assertion `#[test]` arms.** A `#[test]` with no
assertion body registers as PASS-by-vacuity in nextest output and
inflates test counts without coverage. Either (i) convert to
module-level `//!` documentation prose (preferred when the arm is
truly forensic-anchor) or (ii) write a substantive assertion body.
A `#[test]` that asserts nothing is anti-discoverability.

The 4-recurrence pattern within a single wave's regression test
family (L2-R6-R2-MAJOR-2 + L13-MAJ-1 + L13-MIN-4 + L13-MIN-5 +
L9-r6r2-MINOR-3 in Phase-4-Meta-Core R6 R2; total 16 instances at
HEAD) establishes this strengthening as a MUST-ENFORCE clause, not
a SHOULD recommendation. Companion to pim-2 §3.6b end-to-end pin
discipline (which targets the wave's substantive closure arms) +
pim-18 §3.6f original SHAPE-not-SUBSTANCE pre-flight (which targets
ALL new test pins generally). This sub-rule narrows to the
regression-guard test family — where the prevention-of-revert
contract is load-bearing.
```

## Memory codification

A new memory file MUST be minted at
`~/.claude/projects/-Users-benwork-Documents-benten-engine/memory/feedback_pim_n_regression_guard_substantive_arm.md`
during orchestrator consolidation, with cross-links to
`[[feedback_pim_18_shape_not_substance_pre_flight]]` +
`[[feedback_end_to_end_test_pin_for_closed_claims]]` +
`[[feedback_pim_12_red_phase_staged_pin_un_ignore_discipline]]`.
Staged content at
`.addl/phase-4-meta/r6-r2-fp-d-memory-feedback_pim_n_regression_guard_substantive_arm.md`
(force-added in this PR; orchestrator moves to canonical memory dir).
MEMORY.md index gets a one-line entry under
"🟡 Load-bearing operational > Review composition + convergence"
near the existing pim-18 entry.

## Closed origins (HARD RULE 12 / §3.6h compliance)

| Lens finding | Origin sites | Closure |
|---|---|---|
| L2-R6-R2-MAJOR-2 | 6 F4 SHAPE arms | Rewritten in this PR; all 18 affected tests now substantive |
| L9-r6r2-MINOR-3 | `dsl_839_error_code_mirror_matches_typescript_constant_literal` | Rewritten to cross-file TS source scan |
| L13-MAJ-1 | 18-of-34 F4 trait-isolated tests | Rewritten across S2 + S3a + S3b + S3c + S4 to invoke production entry points |
| L13-MIN-4 | S1 arm 3 + S3c arm 3 const-tautology workspace-walkers | Rewritten to real `fs::read_dir` walkers |
| L13-MIN-5 | S3c arm 5 + S3a arm 6 empty `#[test]` arms | Rewritten to substantive assertions (S3c → source-scan; S3a → cascade-residue ordering pin) |

L13-MIN-2 audience_pubkey same-attack-class sibling is OWNED BY R6-R2-FP-A
(sibling agent) per the brief's per-agent ownership split.
L13-MIN-1 (G-CORE-8.2 workspace naming-drift) is OUT OF SCOPE for FP-D
(belongs to orchestrator-direct doc sweep per per-agent ownership
split). L13-MIN-2 (Row D-22 enumeration miscount) + L13-MIN-3 (Row D-22
sub-task 5 destination cite-drift) are doc-only deferrals; OUT OF SCOPE
for FP-D (belongs to orchestrator-direct).
