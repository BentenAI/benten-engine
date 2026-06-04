/*
 * addl-r3-test-writers.js — FINALIZED R3 red-phase test-writers (canary-first, single sequential canary-gated workflow).
 *
 * R3 authors the TDD RED-PHASE corpus from the R2 family catalog: a sole-upstream CANARY wave writes its tests +
 * a mini-review emits a GATE; the fan-out fires ONLY on PASS (so a bad canary halts before burning the fan-out);
 * each wave writes SELF-CONTAINED stub-shim red-phase tests (#[ignore]'d, compile-green at baseline, V2/BE/real-
 * type-shapes from first commit per M-20); per-wave substantive-pin + seam-disjointness mini-reviews; a final
 * coverage-verify confirms every R2 family is covered with zero double-implementation, and calls convergence.
 *
 * METHODOLOGY ENCODED: canary-first (gate fan-out on canary) · pim-12 (#[ignore] + stub-shim + un-ignore destination)
 *   · pim-2/§3.6f (substantive pins: real entry + observable + would-FAIL-on-no-op) · seam-disjointness (no file on
 *   >1 wave) · coverage-verify (every R2 family covered) · refuse-on-partial. (See workflow-common.js.)
 *
 * ⚠️ R3 writers WRITE files → isolation:'worktree' + the loud ABSOLUTE-PATH-FORBIDDEN escape contract (W6 lesson)
 *    + commit-before-return (auto-worktrees auto-clean uncommitted work).
 *
 * INVOKE: Workflow({ name:'addl-r3-test-writers', args:{ cfg:{...}, canary:{...}, waves:[...] } })
 *   args.cfg    = { specRef, specPath, baseBranch, landscapeRef, landscapePath, canon }
 *   args.canary = { key, brief, branch, families:'<F-IDs>' }
 *   args.waves  = [{ key, brief, branch, families:'<F-IDs>', crate }]   (disjoint-file slices)
 */

export const meta = {
  name: 'addl-r3-test-writers',
  description: 'R3: canary-first red-phase test-writers — canary + GATE -> fan-out waves -> per-wave substantive+seam mini-reviews -> coverage-verify/converge',
  phases: [{ title: 'Canary' }, { title: 'CanaryGate' }, { title: 'Fanout' }, { title: 'MiniReview' }, { title: 'Coverage' }],
}

const CFG = (args && args.cfg) || {}
const CANARY = (args && args.canary) || {}
const WAVES = (args && args.waves) || []

const COMMON = `
You author TDD RED-PHASE tests for an ADDL phase. Each test compiles GREEN at baseline behind \`#[ignore = "RED-PHASE: <F-ID> ... un-ignore at R5"]\` against a SELF-CONTAINED in-file stub-shim, and is SUBSTANTIVE (at R5 it invokes the real entry point + asserts an observable consequence + would-FAIL if the impl were a no-op). Author V2 / big-endian / real-type-shapes from the FIRST commit (M-20). Beware Rust-2024 reserved keywords.

## ISOLATION CONTRACT (you run in an auto-managed worktree)
ALL git + file ops stay INSIDE your worktree. NEVER cd to the main repo / absolute paths / \`git -C /other\`. Read sibling state via \`git show <ref>:<path>\`. Commit-before-return (uncommitted work in auto-worktrees evaporates).

## ANCHORS
- Ratified plan (the spec your pins must faithfully encode): \`git show ${CFG.specRef}:${CFG.specPath}\`.
- R2 landscape (your family list + slicing): \`git show ${CFG.landscapeRef}:${CFG.landscapePath}\`.

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }
const GATE_SCHEMA = { type: 'object', additionalProperties: false, required: ['gate', 'reasoning'], properties: {
  gate: { type: 'string', enum: ['PASS', 'FIX-NEEDED'] }, reasoning: { type: 'string' } } }
const REVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['verdict', 'reasoning'], properties: {
  verdict: { type: 'string', enum: ['APPROVE', 'FIX-NEEDED'] }, substantive_ok: { type: 'boolean' }, red_phase_ok: { type: 'boolean' }, seam_disjoint: { type: 'boolean' }, reasoning: { type: 'string' } } }

// CANARY — sole upstream; gates the fan-out
phase('Canary')
const canary = await agent(`${COMMON}\n## CANARY WAVE: ${CANARY.key}\n${CANARY.brief}\nFamilies: ${CANARY.families}. Write the red-phase tests, compile-green-behind-#[ignore], commit to \`${CANARY.branch}\`. Report branch + SHA + one line per family.`,
  { label: `r3-canary:${CANARY.key}`, phase: 'Canary', isolation: 'worktree' })
phase('CanaryGate')
const gate = await agent(`${COMMON}\nAdversarially mini-review the canary \`${CANARY.key}\` (branch \`${CANARY.branch}\`; git show). STRICT verify: every test compiles green behind #[ignore]; pins are SUBSTANTIVE (un-ignore => would-FAIL-on-no-op; no tautologies/self-referential-fixtures); V2/BE/real-shapes; no reserved-kw; commit landed. Emit GATE: PASS only if the fan-out can safely build on this canary's minted shapes; else FIX-NEEDED + minimal fix.\nCANARY: ${canary}`,
  { label: 'r3-canary-gate', phase: 'CanaryGate', schema: GATE_SCHEMA })

if (gate?.gate !== 'PASS') {
  log(`CANARY GATE: FIX-NEEDED — halting before fan-out. ${gate?.reasoning || ''}`)
  return { converged: false, stage: 'canary', gate, note: 'orchestrator-led canary fix-pass + re-run' }
}
log(`CANARY GATE: PASS — fanning out ${WAVES.length} waves`)

// FAN-OUT — each wave writes + a substantive/seam mini-review (≤7 per batch)
phase('Fanout')
const results = []
for (const b of chunk(WAVES, 7)) {
  const res = await pipeline(b,
    w => agent(`${COMMON}\n## R3 WAVE: ${w.key}\n${w.brief}\nFamilies: ${w.families}. Crate: ${w.crate}. Write self-contained red-phase tests, compile-green-behind-#[ignore], commit to \`${w.branch}\`. Report branch + SHA + one line per family.`,
      { label: `r3:${w.key}`, phase: 'Fanout', isolation: 'worktree' }),
    (rep, w) => agent(`${COMMON}\nAdversarially mini-review wave \`${w.key}\` (branch \`${w.branch}\`; git show). Verify: substantive_ok (pins would-FAIL-on-no-op; no tautology/self-ref-fixture/zero-assert), red_phase_ok (all #[ignore]'d w/ valid un-ignore destinations, compile green), seam_disjoint (no test file shared with another wave). APPROVE or FIX-NEEDED.\nWAVE: ${rep}`,
      { label: `r3-review:${w.key}`, phase: 'MiniReview', schema: REVIEW_SCHEMA }).then(v => ({ wave: w.key, branch: w.branch, report: rep, review: v })))
  results.push(...res.filter(Boolean))
}
const approved = results.filter(w => w.review?.verdict === 'APPROVE')
log(`fan-out: ${approved.length}/${WAVES.length} APPROVE`)

// COVERAGE-VERIFY — every R2 family covered, zero double-impl, convergence call
phase('Coverage')
const coverage = await agent(`${COMMON}\n## COVERAGE CONSOLIDATION + CONVERGENCE\nIndependently verify against the R2 landscape (\`git show ${CFG.landscapeRef}:${CFG.landscapePath}\`) + the canary/wave branches: (1) every R2 family has a covering test file (set-diff: missing? extra?); (2) every freeze-gating family present; (3) ZERO double-implementation (no test file authored on >1 branch — git set-intersection across the branches); (4) all mini-reviews APPROVE (${approved.length}/${WAVES.length} + canary). CONVERGENCE CALL = CONVERGED (all covered, zero double-impl, all APPROVE) or list the GAP/PATCH work-list. Note any R5-fill carry-items. Refuse to certify if a wave is missing.\nCANARY @ \`${CANARY.branch}\`; WAVES: ${JSON.stringify(results.map(r => ({ wave: r.wave, branch: r.branch, verdict: r.review?.verdict })))}`,
  { label: 'r3-coverage', phase: 'Coverage' })

return { gate, waves: results.map(r => ({ wave: r.wave, verdict: r.review?.verdict })), approved: approved.length, total: WAVES.length, coverage }
