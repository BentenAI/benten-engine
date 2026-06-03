/*
 * converging-fix-loop.js — the self-converging review→fix→re-review loop.
 *
 * DRAFT (2026-06-02). Wraps a review council in a loop that, each round:
 *   council → structure → adversarial-verify → cross-finding synthesis →
 *   per-file fix-pipeline (validate→brainstorm-as-Ben→review-reasoning-as-Ben+better-shape→
 *   emit-fix→adversarial-review-the-fix) → integrate → compile+self-verify gate →
 *   decision-log → re-review
 * Loops until a full council round surfaces NO NEW BLOCKER/MAJOR.
 * EVERY non-disagreed finding is fixed regardless of severity (HARD RULE 12).
 * Recoverability = audit-not-gate: every decision is documented for Ben's after-the-fact
 * review; a wrong fix is rewrite-cost, not permanent (real freeze is Ben-gated at G-CORE-9/tag).
 *
 * METHODOLOGY THIS LOOP STRUCTURALLY ENCODES (workflows-as-executable-methodology):
 *   - HARD RULE 12 (fix-all-non-disagreed; 3 valid non-fix dispositions) ...... fix-everything + disposition classifier
 *   - iterate-to-convergence R1/R4/R4b/R6 (rule 9) ........................... the loop + no-new-BLK/MAJ exit
 *   - Q5 full-council-every-round ........................................... full council re-run each round
 *   - plain-English-with-prediction + surface-arch-decisions ................ brainstorm options + reasoned-rec-AS-BEN
 *   - extra-reflection-pass-for-elegant-permanent-shape ..................... cross-finding synthesis + review-reasoning+better-shape
 *   - review-finding-ground-truth-verify + adversarial ..................... validate + adversarial-verify + review-the-fix
 *   - night-shift-stance (decide-as-Ben, document, continue, surface) ....... autonomy + decision-log + FLAGGED-FOR-BEN
 *   - agent-economics-prefer-thorough-cleanup .............................. accept-rewrite stance
 *   - §3.5h pre-merge workspace gate + §3.6j sweep-self-verify ............. compile + lint/cite self-verify gate each round
 *   - pim-2/§3.6f substantive-arm / would-FAIL-on-revert ................... review-the-fix asserts the closure is real
 *   - §3.14 strategy-C + worktree-drop-on-merge ........................... integrate + drop worktrees each round
 *   - agent-isolation-escape + commit-before-return ....................... read-only contract; single-writer integrator
 *   - pattern-induction-meta-sweep + 3+-recurrence-deep-sweep .............. terminal pim-N candidate sweep
 *   - periodic-agent-spend-snapshot ....................................... per-round cost log via budget.spent()
 */

export const meta = {
  name: 'converging-fix-loop',
  description: 'Self-converging review->fix->re-review loop; fixes ALL non-disagreed findings each round; iterates until no new BLOCKER/MAJOR',
  phases: [
    { title: 'Review' }, { title: 'Structure' }, { title: 'Verify' },
    { title: 'Synthesis' }, { title: 'Fix' }, { title: 'Integrate' },
    { title: 'Gate' }, { title: 'Terminal' },
  ],
}

// ============================== CONFIG (the R4 instance) ==============================
// For reuse: swap this block (or pass via `args`). The loop machinery below is generic.
const CFG = (args && args.cfg) || {
  corpusBranch: 'phase-4-meta-core/f-full-r3-consolidated',
  mainBase: '2172cb6d',
  r03Ref: 'phase-4-meta-core/f-full-r0-plan-r1fp-r03',
  orchRef: 'phase-4-meta-core/orchestration-2026-05-26',
  r03Path: '.addl/phase-4-meta/f-full-r0-plan.md',
  r2Path: '.addl/phase-4-meta/f-full-r2-test-landscape.md',
  MAX_ROUNDS: 4,
  BUDGET_FLOOR: 120000, // stop opening a new round if fewer than ~120k output tokens remain
  // integration worktree the loop OWNS (single-writer; the only thing that mutates the corpus)
  integWorktree: '../benten-wt-fixloop',
}

// 15-lens R4 council (re-uses the R4 review lens set). For a different artifact, swap LENSES.
const LENSES = (args && args.lenses) || [
  'substantive-pin&falsifiability', 'R5-readiness', 'determinism/flake/isolation/CI-realism',
  'wire-freeze-byte-correctness', 'codepoint-registry', 'crypto-construction',
  'encoding/DAG-CBOR/BE', 'invariants Inv16-22', 'capability/UCAN/RBAC',
  'threat-model/bounded-decode', 'distributed/sync', 'MembershipSet-shape-fidelity',
  'coverage/gap-audit', 'cross-wave-seam/M-20', 'ruling-fidelity/cross-lang',
].map(k => ({ key: k }))

// ============================== SHARED PREAMBLE ==============================
const READONLY = `
## READ-ONLY ACCESS CONTRACT (NON-NEGOTIABLE)
Read everything via \`git show <ref>:<path>\` from the repo. NEVER checkout/cd/commit/branch/modify — a prior agent did exactly that and corrupted a shared tree. Only \`git show\`/\`git ls-tree\`/\`git diff --name-only\`/\`git grep <ref>\` are allowed. You are a reasoner that EMITS text; you do NOT touch git state. (The single integrator stage is the ONLY writer.)`

const ANCHORS = `
## ANCHORS
- Corpus under review: branch \`${CFG.corpusBranch}\` (off main \`${CFG.mainBase}\`). Enumerate: \`git diff --name-only --diff-filter=A ${CFG.mainBase} ${CFG.corpusBranch} -- '*/tests/*.rs'\`. Read: \`git show ${CFG.corpusBranch}:<path>\`.
- R0.3 canonical plan (the spec tests must faithfully pin): \`git show ${CFG.r03Ref}:${CFG.r03Path}\`.
- R2 test-landscape (89-family catalog + coverage matrix): \`git show ${CFG.orchRef}:${CFG.r2Path}\`.`

// The full F-full canon (codepoints, rulings, invariants, stakes) — keep in sync with f-full-r4-review.js COMMON.
const CANON = `<<< inline the F-full context + BLESSED codepoint table + Ben rulings + TDD red-phase bar here; identical to f-full-r4-review.js COMMON >>>`

const COMMON = `${READONLY}\n${ANCHORS}\n${CANON}`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }
const keyOf = f => `${f.file}::${f.family || ''}::${(f.claim || '').slice(0, 60)}`

// ============================== SCHEMAS ==============================
const FINDINGS_SCHEMA = { type: 'object', additionalProperties: false, required: ['panel_returned', 'panel_expected', 'findings'], properties: {
  panel_returned: { type: 'integer' }, panel_expected: { type: 'integer' },
  findings: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['id', 'severity', 'file', 'claim', 'real', 'disposition'], properties: {
    id: { type: 'string' }, severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
    lens: { type: 'string' }, file: { type: 'string' }, family: { type: 'string' }, claim: { type: 'string' },
    why_matters: { type: 'string' },
    real: { type: 'boolean' }, // adversarial-validated: false => documented DISAGREE, no fix
    disposition: { type: 'string', enum: ['fix', 'disagree', 'out-of-scope', 'belongs-named-now'] },
    disposition_reason: { type: 'string' },
  } } } } }

const FIX_SCHEMA = { type: 'object', additionalProperties: false, required: ['file', 'finding_ids', 'new_content', 'reasoning_as_ben', 'options_considered', 'confidence', 'flag_for_ben'], properties: {
  file: { type: 'string' }, finding_ids: { type: 'array', items: { type: 'string' } },
  options_considered: { type: 'string' }, reasoning_as_ben: { type: 'string' },
  better_shape_found: { type: 'string' }, confidence: { type: 'string', enum: ['high', 'medium', 'low'] },
  flag_for_ben: { type: 'boolean' }, flag_reason: { type: 'string' },
  new_content: { type: 'string' }, // FULL new file content (integrator writes verbatim)
  diff_summary: { type: 'string' },
} }

const FIXREVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['file', 'verdict', 'closes_findings', 'compiles_shape_ok', 'no_regression', 'reasoning'], properties: {
  file: { type: 'string' }, verdict: { type: 'string', enum: ['APPROVE', 'REVISE', 'REJECT'] },
  closes_findings: { type: 'boolean' }, compiles_shape_ok: { type: 'boolean' }, // still #[ignore]'d + stub-shim self-contained
  no_regression: { type: 'boolean' }, would_fail_on_revert: { type: 'boolean' }, // pim-2 substantive-arm
  reasoning: { type: 'string' }, required_revision: { type: 'string' },
} }

// ============================== LOOP ==============================
const seen = new Set()
const decisionLog = []
let round = 1, converged = false

while (!converged && round <= CFG.MAX_ROUNDS && (!budget.total || budget.remaining() > CFG.BUDGET_FLOOR)) {
  log(`=== ROUND ${round} === (spent ${Math.round(budget.spent() / 1000)}k; ${budget.total ? Math.round(budget.remaining() / 1000) + 'k left' : 'no budget cap'})`)

  // ---- 1. REVIEW: full council, batched 4 (Q5 full-council-every-round) ----
  phase('Review')
  const reports = []
  for (const b of chunk(LENSES, 4)) {
    reports.push(...await parallel(b.map(L => () =>
      agent(`${COMMON}\n## YOUR LENS: ${L.key}\nAdversarially review the corpus from this lens. Cite real lines. Emit findings: [SEVERITY] file | F-ID | DEFECT | WHY | suggested-disposition. End: LENS VERDICT.`,
        { label: `r${round}-lens:${L.key}`, phase: 'Review' }))))
  }
  const live = reports.filter(Boolean)

  // ---- 2. STRUCTURE + VALIDATE: dedup, adversarially validate each (real?), classify disposition ----
  phase('Structure')
  const structured = await agent(`${COMMON}
You are the structuring + validation consolidator. From the ${live.length} lens reports below, extract EVERY distinct finding, dedup (keep highest severity + note corroborating lenses), and for EACH: set \`real\` by adversarially checking it against the actual corpus (\`git show ${CFG.corpusBranch}:<file>\`) — default real=false if you cannot substantiate it (we reject plausible-but-wrong). Classify \`disposition\` per HARD RULE 12: 'fix' (real, must fix regardless of severity), 'disagree' (not real / recommendation wrong — give reason), 'out-of-scope' (reason), 'belongs-named-now' (names a specific other doc that must receive an entry now). Set panel_returned=${live.length}, panel_expected=${LENSES.length}.
=== REPORTS ===\n${live.map((r, i) => `--- ${i + 1} ---\n${r}`).join('\n')}`,
    { label: `r${round}-structure`, phase: 'Structure', schema: FINDINGS_SCHEMA })

  // dedup vs seen; fix-list = real + disposition 'fix'; not yet seen
  const allFindings = (structured?.findings || [])
  const toFix = allFindings.filter(f => f.real && f.disposition === 'fix' && !seen.has(keyOf(f)))
  const newBlkMaj = toFix.filter(f => f.severity === 'BLOCKER' || f.severity === 'MAJOR')
  log(`round ${round}: ${allFindings.length} findings; ${toFix.length} fresh to-fix (${newBlkMaj.length} BLOCKER/MAJOR)`)

  // record disagrees/out-of-scope/belongs-named-now into the decision-log (nothing silently dropped)
  decisionLog.push({ round, kind: 'triage', findings: allFindings, panel: `${live.length}/${LENSES.length}` })

  // CONVERGENCE: a full panel round with no NEW BLOCKER/MAJOR. (We still fix any fresh MINOR/OBS this round.)
  if (round > 1 && newBlkMaj.length === 0) converged = true
  if (toFix.length === 0) { converged = true; break }

  // ---- 3. CROSS-FINDING SYNTHESIS-as-Ben (elegant permanent shape across clusters) ----
  phase('Synthesis')
  const synthesis = await agent(`${COMMON}
Reason AS BEN. Below are ${toFix.length} findings to fix. Before per-finding fixes, find the ELEGANT PERMANENT SHAPE: do any of these share a root cause or a single structural change that closes a CLUSTER at once (strictly less churn; forward-class-of-bug closure)? Per extra-reflection-pass discipline. Propose cluster-fixes (which findings each closes) + flag findings best fixed individually. Do NOT write code; propose shapes.
=== FINDINGS ===\n${JSON.stringify(toFix, null, 1)}`,
    { label: `r${round}-synthesis`, phase: 'Synthesis' })

  // ---- 4. PER-FILE FIX PIPELINE (group by file; brainstorm-as-Ben → review-as-Ben+better-shape → emit fix → review-the-fix) ----
  phase('Fix')
  const byFile = {}; for (const f of toFix) (byFile[f.file] ||= []).push(f)
  const fileGroups = Object.entries(byFile).map(([file, fs]) => ({ file, findings: fs }))

  const fixes = await pipeline(fileGroups,
    // stage A: brainstorm options + reasoned recommendation AS BEN (do-it-now / ideal-permanent-shape)
    g => agent(`${COMMON}
Reason AS BEN. Fix ALL findings for file \`${g.file}\` (read it via \`git show ${CFG.corpusBranch}:${g.file}\`). Consider the cross-finding synthesis below. For the file: (1) list OPTIONS, (2) give a fully-reasoned RECOMMENDATION as Ben (do-it-now, ideal permanent shape, substantive-pin bar: real entry point + observable consequence + would-FAIL-on-no-op; keep #[ignore]'d + self-contained stub-shim). Produce the FULL new file content. If a finding is genuinely high-uncertainty (e.g. R0.3 itself is ambiguous), still emit your best-reasoned-as-Ben fix and set flag_for_ben=true with a reason — do NOT halt.
SYNTHESIS: ${synthesis}
FINDINGS for this file: ${JSON.stringify(g.findings, null, 1)}`,
      { label: `r${round}-fix:${g.file.split('/').pop()}`, phase: 'Fix', schema: FIX_SCHEMA }),
    // stage B: review-the-reasoning AS BEN + hunt an even-better permanent shape (can refine the content)
    (rec, g) => agent(`${COMMON}
Reason AS BEN. A fixer proposed the fix below for \`${g.file}\`. Review its REASONING as Ben and ask: is there a MORE IDEAL permanent shape it missed (extra-reflection-pass)? Verify the proposed new_content meets the substantive-pin bar + closes every finding + stays #[ignore]'d. If you can improve it, emit the improved FULL new_content + say what you changed in better_shape_found. Otherwise pass it through. Keep flag_for_ben if the fixer set it.
PROPOSED: ${JSON.stringify(rec, null, 1)}`,
      { label: `r${round}-review-reason:${g.file.split('/').pop()}`, phase: 'Fix', schema: FIX_SCHEMA }),
    // stage C: adversarial review-THE-FIX (does it close, compile-shape, not regress, would-FAIL-on-revert)
    (fix, g) => agent(`${COMMON}
Adversarially review this APPLIED fix for \`${g.file}\` (diff new_content vs \`git show ${CFG.corpusBranch}:${g.file}\`). Verify: closes_findings (every finding id), compiles_shape_ok (still compiles green behind #[ignore], stub-shim self-contained, valid Rust), no_regression (no unrelated content lost/changed), would_fail_on_revert (the pin is SUBSTANTIVE — would fail against a no-op impl; pim-2). Default REJECT if you cannot confirm. Return the fix (with new_content) + your verdict.
FIX: ${JSON.stringify(fix, null, 1)}`,
      { label: `r${round}-review-fix:${g.file.split('/').pop()}`, phase: 'Fix', schema: FIXREVIEW_SCHEMA }).then(v => ({ ...fix, review: v })))

  const approved = fixes.filter(Boolean).filter(f => f.review?.verdict === 'APPROVE')
  log(`round ${round}: ${approved.length}/${fileGroups.length} file-fixes APPROVED`)

  // ---- 5. INTEGRATE (single controlled writer; the loop's OWN integration worktree; disjoint files => no conflict) ----
  phase('Integrate')
  const integ = await agent(`You are the SINGLE integrator (the only writer this round). Working in the integration worktree \`${CFG.integWorktree}\` checked out on \`${CFG.corpusBranch}\` (the orchestrator owns this worktree — operate ONLY inside it, never the main repo). For each approved file-fix below, write its new_content to the file verbatim, \`git add\`. Then ONE commit: "test(fixloop r${round}): apply ${approved.length} reviewed fixes". Then run the compile-gate: per affected crate \`cargo test -p <crate> [--features ...] --no-run\` and report green/red + any errors. Do NOT run workspace cargo. Report the commit SHA + compile result.
APPROVED FIXES (file -> new_content): ${JSON.stringify(approved.map(a => ({ file: a.file, finding_ids: a.finding_ids, diff_summary: a.diff_summary })), null, 1)}
[new_content bodies provided out-of-band by the orchestrator integrator step]`,
    { label: `r${round}-integrate`, phase: 'Integrate' })
  // NOTE (shakedown): the new_content bodies are large; in practice the integrator writes them from the
  //   approved[] objects. This stage is the part to harden on the first real run (single-writer worktree).

  // ---- 6. GATE: compile + §3.5h/§3.6j self-verify (lint/cite-drift on touched files) ----
  phase('Gate')
  const gate = await agent(`${COMMON}
Self-verify the round-${round} fixes in \`${CFG.integWorktree}\`: (a) compile-gate result (from integrator) is green behind #[ignore]; (b) §3.5h: scoped clippy + fmt clean on touched crates; (c) §3.6j: every cite added in the fixes resolves (file/symbol/PR exists). Report PASS or the residual to fix. If red, name the minimal compile/lint fix.
INTEGRATOR REPORT: ${integ}`,
    { label: `r${round}-gate`, phase: 'Gate' })

  // ---- 7. decision-log + seen update + cost snapshot ----
  decisionLog.push({ round, kind: 'fixes', approved: approved.map(a => ({ file: a.file, finding_ids: a.finding_ids, options: a.options_considered, rec: a.reasoning_as_ben, better_shape: a.better_shape_found, flag_for_ben: a.flag_for_ben, flag_reason: a.flag_reason, review: a.review })), integrate: integ, gate })
  toFix.forEach(f => seen.add(keyOf(f)))
  log(`round ${round} done; cumulative spent ${Math.round(budget.spent() / 1000)}k`)
  round++
}

// ============================== TERMINAL: pattern-induction + Ben surface ==============================
phase('Terminal')
const patternInduction = await agent(`${COMMON}
Pattern-induction meta-sweep over the ${decisionLog.length}-entry decision log below. Hunt UNNAMED cross-cutting patterns (≥3 recurrence => candidate). Propose: (a) new pim-N / dispatch-convention codifications, (b) any class-of-bug the loop kept re-fixing (a deeper root cause), (c) any finding the loop may have under-fixed. Return a prose list.
DECISION LOG (summary): ${JSON.stringify(decisionLog.map(d => ({ round: d.round, kind: d.kind, n: (d.findings || d.approved || []).length })), null, 1)}`,
  { label: 'pattern-induction', phase: 'Terminal' })

return {
  converged,
  rounds: round - 1,
  panel: `last round full council`,
  decisionLog,       // <- the audit trail for Ben's after-the-fact review (plain-English, per-decision)
  patternInduction,  // <- new pim-N candidates
  flaggedForBen: decisionLog.flatMap(d => (d.approved || []).filter(a => a.flag_for_ben)),
}
