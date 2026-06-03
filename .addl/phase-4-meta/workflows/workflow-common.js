/*
 * workflow-common.js — CANONICAL shared preamble + methodology-codification map for ALL Benten ADDL workflows.
 *
 * ⚠️ The Workflow execution sandbox has NO imports / NO filesystem (plain JS, sandboxed). So this file is NOT
 * imported at runtime — it is the SOURCE OF TRUTH that you INLINE (copy the COMMON_PREAMBLE text into each
 * workflow's `COMMON` const at authoring time). Maintain it HERE; when it changes, re-inline into the library.
 * This is the "inline-gitignored-canon-in-briefs" discipline applied to workflows themselves.
 *
 * WHY workflows codify our methodology: a workflow turns "rules I might skip under turn-budget pressure" into
 * control flow that CANNOT be skipped. See memory feedback_workflows_as_executable_methodology.
 */

// ===== UNIVERSAL PREAMBLE — inline into every council/review/fix workflow's COMMON =====
export const COMMON_PREAMBLE = `
## READ-ONLY ACCESS CONTRACT (NON-NEGOTIABLE) — applies to all reasoning/review agents
Read everything via \`git show <ref>:<path>\`. NEVER checkout / cd / commit / branch / modify / use absolute paths outside your given scope — a prior agent did exactly that and corrupted a shared working tree (the W6 isolation-escape). Only \`git show\` / \`git ls-tree\` / \`git diff --name-only\` / \`git grep <ref>\` / \`git log\` are allowed. You EMIT text; you do not touch git state. (In fix workflows, exactly ONE integrator stage writes.)

## PRE-FLIGHT (first action)
State the ref + SHA you are reviewing and assert it matches the brief's expected ref (reviewer-pre-flight tree-state; §3.5i staleness). If you are >3 commits stale or the corpus moved, say so as a top-level finding.

## GROUND-TRUTH DISCIPLINE (§3.5n + ground-truth-verify)
Ground EVERY claim in an actual line you read (cite file + what the code does). Default to NOT-REAL / REFUTED if you cannot substantiate a finding against the real text — we reject plausible-but-wrong. Adversarial verifiers default to REFUTED on uncertainty.

## DISPOSITION (HARD RULE 12) — only 3 valid non-fix outcomes
Every finding is FIX (regardless of severity) UNLESS: (a) out-of-scope (state reason), (b) belongs-named-now (name the SPECIFIC other doc; it must receive its entry THIS round, never "later"), (c) disagree-with-explanation (cite the conflict). "defer" / "carry to next" / "minor enough" are NEVER valid.

## PANEL INTEGRITY (verify-by-artifact, never notification-absence)
A consolidator MUST NOT certify convergence on a partial panel. If fewer lenses returned than dispatched, call PANEL-INCOMPLETE and name the missing lenses — a rate-limited silence is not an APPROVE.

## BEN-FACING OUTPUT (plain-English-with-prediction)
Anything surfaced to Ben: plain situation -> concrete options -> reasoned recommendation AS BEN + prediction -> confirm/redirect. Decision-logs are per-decision in this format so Ben can audit cold in seconds.

## OUTPUT
Return findings AS YOUR FINAL MESSAGE in prose (no structured-output tool unless the brief gives a schema).
`

/* ===== CODIFICATION MAP — methodology/memory -> workflow construct =====
 * STRUCTURAL (encoded as stages/control-flow):
 *   HARD RULE 12 ............................. fix-all-non-disagreed + 4-disposition classifier
 *   iterate-to-convergence (rule 9) / Q5 ..... the loop + full-council-every-round + no-new-BLK/MAJ exit
 *   extra-reflection-pass .................... cross-finding synthesis-as-Ben + review-reasoning+better-shape
 *   plain-English + surface-arch-decisions ... brainstorm-options + reasoned-rec-AS-BEN stage; decision-log format
 *   night-shift-stance ....................... autonomy + decision-log + FLAGGED-FOR-BEN (continue, don't halt)
 *   ground-truth-verify + adversarial ........ validate stage + adversarial-verify + adversarial-review-the-fix
 *   pim-2/§3.6f substantive-arm .............. review-the-fix asserts would_fail_on_revert
 *   §3.5h + §3.6j ............................ per-round compile + lint/cite self-verify GATE
 *   §3.14 strategy-C + worktree-drop ......... integrate stage (single-writer; drop fix-worktrees on merge)
 *   §3.5g cross-language rule-mirror ......... gate check: new ErrorCode/wire-type has TS/napi mirror
 *   §3.6h ratification-must-close-origin ..... terminal: a proposed rule citing an origin closes it same-run
 *   canary-first (R5 loop variant) ........... round-0 canary; fan-out gated on its merge
 *   pattern-induction + 3+-recurrence ........ terminal pim-N candidate sweep
 *   periodic-spend-snapshot .................. per-round budget.spent() log
 *   agent-economics-prefer-thorough-cleanup .. accept-rewrite stance (audit-not-gate)
 * PREAMBLE (inline COMMON_PREAMBLE above):
 *   read-only/isolation-escape · reviewer-pre-flight tree-state · §3.5n ground-truth ·
 *   refuse-on-partial-panel · plain-English-with-prediction · commit-before-return · inline-canon
 * NOT codified (stay Ben-gated / human-watched):
 *   the tag/freeze decision · "is this architectural direction right" · the integrator's first real run
 *
 * REUSE MECHANICS:
 *   - invoke-by-name: .claude/workflows/<name>.js  ->  Workflow({name}) / inline workflow("<name>", args)
 *   - durable+tracked (since .claude/ is gitignored here): .addl/phase-4-meta/workflows/  (force-added)
 *   - every run auto-saves to the SESSION dir (.../workflows/scripts/<name>-<runId>.js) — session-scoped;
 *     HARVEST keepers into the durable library at launch (same discipline as recording Task/Run IDs).
 */
