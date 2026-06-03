/*
 * addl-review-council.js — FINALIZED generic adversarial review council (one ROUND of R1 / R4 / R4b / R6).
 *
 * The single reusable shape behind every ADDL review tier that gates+mutates an artifact and iterates to
 * convergence (rule 9 / Q5): N lenses → structure → adversarial-verify (refute BLK/MAJ) → completeness-critic
 * → converge (refuse-on-partial-panel). ITERATE ACROSS INVOCATIONS: run it → triage/fix the artifact → run it
 * again on the EDITED artifact → until a full round returns 0 BLOCKER/MAJOR. (To auto-iterate review+FIX in one
 * run, use converging-fix-loop.js instead; this is the review-only round.)
 *
 * Used for: R1 (plan critic-council) · R4/R4b (test-corpus review) · R6 (phase-close council). The lens SET +
 * artifact + framing are passed via args; the machinery is identical. (extra-reflection-pass applied to the
 * library itself: one council, not four.)
 *
 * METHODOLOGY ENCODED: Pattern-6 (lenses by surface) · iterate-to-convergence/Q5 (full council every round)
 *   · adversarial-verify (refute false positives) · completeness-critic (fresh gap-hunt) · ground-truth-verify
 *   · refuse-on-partial-panel · HARD-RULE-12 dispositions · plain-English converge call. (See workflow-common.js.)
 *
 * INVOKE: Workflow({ name:'addl-review-council', args:{ cfg:{...}, lenses:[...] } })
 *   args.cfg    = { tier:'R1'|'R4'|'R6', artifactRef, artifactDesc, enumerate:'<cmd to list files>', specRef,
 *                  specPath, orchRef, framing:'<what this round is + any known-correct do-not-reflag>', canon }
 *   args.lenses = [{ key, mandate, focus }]   (Pattern-6 lens set for THIS artifact)
 */

export const meta = {
  name: 'addl-review-council',
  description: 'Generic adversarial review council (R1/R4/R4b/R6): N lenses (batched 4) -> structure -> adversarial-verify -> completeness-critic -> converge; refuse on partial panel',
  phases: [{ title: 'Review' }, { title: 'Structure' }, { title: 'Verify' }, { title: 'Completeness' }, { title: 'Converge' }],
}

const CFG = (args && args.cfg) || {}
const LENSES = (args && args.lenses) || []

const COMMON = `
You are ONE lens on the **${CFG.tier || 'ADDL'} review council**. ${CFG.framing || 'Find everything WRONG, WEAK, MISSING, or MIS-SPECIFIED in the artifact from your lens, before it advances.'}

## READ-ONLY CONTRACT (NON-NEGOTIABLE)
Read ONLY via \`git show <ref>:<path>\`. NEVER checkout/cd/commit/branch/modify (the W6 escape corrupts a shared tree). First action: state the ref+SHA you review + assert it matches the brief. Ground EVERY finding in a real line you read; if you cannot substantiate it, DROP it (§3.5n).

## ANCHORS
- Artifact under review: ${CFG.artifactDesc} — ref \`${CFG.artifactRef}\`. Enumerate: \`${CFG.enumerate || 'git show ' + CFG.artifactRef + ':' + (CFG.specPath || '')}\`. Read: \`git show ${CFG.artifactRef}:<path>\`.
- Spec of record: \`git show ${CFG.specRef}:${CFG.specPath}\`.
${CFG.orchRef ? `- Prior-round triage / landscape (if any): \`git show ${CFG.orchRef}:.addl/...\`` : ''}

${CFG.canon || ''}

## OUTPUT CONTRACT
Return findings AS YOUR FINAL MESSAGE in prose (no structured-output tool). Per finding: \`[SEVERITY] <file/area> | <id> | DEFECT | WHY (for the gate/freeze) | DISPOSITION (fix-now | belongs-named-now(doc) | out-of-scope(reason) | disagree(cite))\`. SEVERITY ∈ {BLOCKER, MAJOR, MINOR, OBS}. End: \`LENS VERDICT: <APPROVE | FINDINGS-RAISED> — N BLK / N MAJ / N MIN / N OBS\`.
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }

const FINDINGS_SCHEMA = { type: 'object', additionalProperties: false, required: ['panel_returned', 'panel_expected', 'findings'], properties: {
  panel_returned: { type: 'integer' }, panel_expected: { type: 'integer' },
  findings: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['id', 'severity', 'file', 'claim', 'recommended_disposition'], properties: {
    id: { type: 'string' }, severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
    lens: { type: 'string' }, file: { type: 'string' }, family: { type: 'string' }, claim: { type: 'string' },
    why_matters: { type: 'string' }, recommended_disposition: { type: 'string' } } } }, summary: { type: 'string' } } }
const VERDICT_SCHEMA = { type: 'object', additionalProperties: false, required: ['finding_id', 'verdict', 'corrected_severity', 'reasoning'], properties: {
  finding_id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PARTIAL'] },
  corrected_severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS', 'NONE'] }, reasoning: { type: 'string' } } }

// 1. REVIEW — N lenses, batched 4 (rate-limit), schema-free prose
phase('Review')
const reports = []
for (const b of chunk(LENSES, 4)) {
  reports.push(...await parallel(b.map(L => () =>
    agent(`${COMMON}\n## YOUR LENS: ${L.key}\n**Focus:** ${L.focus || 'the whole artifact'}\n**Mandate:** ${L.mandate}`,
      { label: `${CFG.tier || 'R'}:${L.key}`, phase: 'Review' }))))
}
const live = reports.filter(Boolean)
log(`council: ${live.length}/${LENSES.length} lenses returned`)

// 2. STRUCTURE — dedup, classify
phase('Structure')
const structured = await agent(`${COMMON}\nStructuring consolidator. From the ${live.length} lens reports, extract EVERY distinct finding, dedup (keep highest severity + note corroborating lenses), assign ids, classify recommended_disposition per HARD RULE 12 (fix-now / belongs-named-now / out-of-scope / disagree). panel_returned=${live.length}, panel_expected=${LENSES.length}. Do NOT invent findings.\n=== REPORTS ===\n${live.map((r, i) => `--- ${i + 1} ---\n${r}`).join('\n')}`,
  { label: 'structure', phase: 'Structure', schema: FINDINGS_SCHEMA })

// 3. ADVERSARIAL VERIFY — refute each BLOCKER/MAJOR (batched 4)
phase('Verify')
const majors = (structured?.findings || []).filter(f => f.severity === 'BLOCKER' || f.severity === 'MAJOR')
const verdicts = []
for (const b of chunk(majors, 4)) {
  const res = await parallel(b.map(f => () =>
    agent(`${COMMON}\n## ADVERSARIALLY REFUTE finding ${f.id} [${f.severity}] (lens ${f.lens}; file ${f.file}; ${f.family || ''})\nCLAIM: ${f.claim}\nGo to the actual artifact (\`git show ${CFG.artifactRef}:${f.file}\`) + spec; try to REFUTE it. Default REFUTED if you cannot independently reproduce the defect. CONFIRMED only if you reproduce it. PARTIAL if real but mis-severity.`,
      { label: `verify:${f.id}`, phase: 'Verify', schema: VERDICT_SCHEMA })))
  verdicts.push(...res.filter(Boolean))
}

// 4. COMPLETENESS CRITIC — fresh gap-hunt
phase('Completeness')
const critic = await agent(`${COMMON}\n## COMPLETENESS CRITIC (fresh, independent)\nThe council + verify just ran. What did the WHOLE panel MISS? (a) which spec/invariant/requirement has NO covering finding/coverage? (b) which adversary path / failure mode untested? (c) a whole dimension or lens uncovered? Independently spot-check the spec against the artifact. Return a prose GAP list (each: what's missing + severity + why + recommended action). If no real gap, say so explicitly.`,
  { label: 'completeness-critic', phase: 'Completeness' })

// 5. CONVERGE — refuse on partial panel
phase('Converge')
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED' || v.verdict === 'PARTIAL')
const converge = await agent(`${COMMON}\n## CONVERGENCE CONSOLIDATOR\nPANEL: ${live.length}/${LENSES.length}. **If < ${LENSES.length} returned you MUST NOT certify CONVERGED — call PANEL-INCOMPLETE + name the missing lenses (a rate-limited silence is not an APPROVE).**\nINPUTS:\n- findings (${(structured?.findings || []).length}): ${JSON.stringify(structured?.findings || [], null, 1)}\n- adversarial verdicts on BLK/MAJ: ${JSON.stringify(verdicts, null, 1)}\n- confirmed/partial after verify: ${confirmed.length}\n- completeness-critic gaps:\n${critic}\nPRODUCE: (1) CONVERGENCE CALL = CONVERGED / NOT-CONVERGED / PANEL-INCOMPLETE (CONVERGED requires full panel AND zero CONFIRMED BLOCKER AND zero CONFIRMED MAJOR; REFUTED don't count; MINOR/OBS + genuine-named-deferrals don't block but MUST be named). (2) triage table (every CONFIRMED finding + disposition per HARD-RULE-12). (3) the FIX work-list vs the named carry-list. (4) crisp summary for orchestrator + Ben.`,
  { label: 'converge', phase: 'Converge' })

return { panel: `${live.length}/${LENSES.length}`, structured, verdicts, critic, converge }
