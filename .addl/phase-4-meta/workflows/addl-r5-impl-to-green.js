/*
 * addl-r5-impl-to-green.js — FINALIZED R5 implementation: canary-first impl waves, each looping impl→un-ignore→test→fix until GREEN.
 *
 * R5 turns the R3/R4 RED-PHASE corpus GREEN: implement real production code so the #[ignore]'d tests pass.
 * Canary-first (feedback_canary_first_parallel_implementation): the canary wave owns the API surface the
 * others consume — it runs FIRST and gates the fan-out on its merge. Each wave loops impl→un-ignore→nextest→fix
 * until its tests pass, mini-reviewed (pim-2 substantive / §3.6f). Then strategy-C integrate + full-suite.
 *
 * ⚠️ Unlike the review/fix-loops, R5 agents WRITE real code + run cargo → they use isolation:'worktree' with the
 * loud ABSOLUTE-PATH-FORBIDDEN escape contract (feedback_agent_isolation_escape_absolute_paths; the W6 lesson).
 * ≤7 implementer cap (read-only mini-reviews are cap-exempt) — scale waves accordingly. Watch disk (97% HARD-ABORT,
 * ~90% cargo-clean-idle valve). M-20: byte-pinning families rebase onto the canary's landing SHA before un-ignore.
 *
 * METHODOLOGY ENCODED: canary-first · iterate-to-convergence (per-wave impl-to-green loop) · pim-2 substantive
 *   (un-ignore = real entry + observable + would-FAIL-on-revert) · §3.5h pre-push gate · §3.14 strategy-C
 *   · mini-review-after-every-group · decide-as-Ben + decision-log. (See workflow-common.js + converging-fix-loop.js.)
 *
 * INVOKE: Workflow({ name:'addl-r5-impl-to-green', args:{ cfg:{...}, canary:{...}, waves:[...] } })
 *   args.cfg    = { corpusBranch, baseBranch, specRef, specPath, MAX_FIX_ROUNDS, crates:[{name,features}], canon }
 *   args.canary = { key, brief, branch, mints:'<the types/surface the fan-out depends on>' }
 *   args.waves  = [{ key, brief, branch, files:'<un-ignore targets>' }]   (disjoint-file slices, ≤7 per batch)
 */

export const meta = {
  name: 'addl-r5-impl-to-green',
  description: 'R5: canary-first implementation waves, each looping impl->un-ignore->test->fix until GREEN; gate fan-out on canary; strategy-C integrate + full-suite',
  phases: [
    { title: 'Canary' }, { title: 'CanaryGate' }, { title: 'Waves' }, { title: 'MiniReview' }, { title: 'Integrate' }, { title: 'FullSuite' },
  ],
}

const CFG = (args && args.cfg) || {}
const CANARY = (args && args.canary) || {}
const WAVES = (args && args.waves) || []
const MAX_FIX = CFG.MAX_FIX_ROUNDS || 3

const COMMON = `
## ISOLATION CONTRACT (NON-NEGOTIABLE — you run in an auto-managed git worktree)
ALL git ops + file reads/writes stay INSIDE your worktree (your dispatch-time \`pwd\`). NEVER cd to the main repo / absolute paths outside it / \`git -C /other/path\`. To read main-repo or sibling-branch state use \`git show <ref>:<path>\`. Violating this races other parallel implementers + corrupts the shared tree (the W6 escape).

## SCOPED PRE-FLIGHT ONLY (laptop resource discipline)
NEVER run workspace cargo (\`--workspace\`). Run scoped per-package: \`cargo test -p <crate> <features> --no-run\` / \`cargo nextest run -p <crate> <features>\` / single-crate clippy + fmt. CI is the authoritative full-workspace verifier. If asked for workspace cargo, DECLINE + surface.

## ANCHORS
- Spec of record: \`git show ${CFG.specRef}:${CFG.specPath}\`. RED-PHASE corpus base: \`${CFG.corpusBranch}\` (your tests are the #[ignore]'d pins you make pass).
- Disposition (HARD RULE 12): implement everything; only out-of-scope(reason)/belongs-named-now(named)/disagree(cite) are non-do.

## THE IMPL-TO-GREEN BAR (pim-2 / §3.6f)
For each assigned RED-PHASE test: implement the REAL production surface it pins, DELETE the in-file stub-shim, insert the real \`use ...\`, REMOVE the \`#[ignore]\`, and make it pass via \`cargo nextest run\`. The un-ignored test MUST exercise the real entry point + assert an observable consequence + would-FAIL if reverted. Beware Rust-2024 reserved keywords. Commit before returning (auto-worktrees auto-clean uncommitted work).

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }

const GATE_SCHEMA = { type: 'object', additionalProperties: false, required: ['gate', 'reasoning'], properties: {
  gate: { type: 'string', enum: ['PASS', 'FIX-NEEDED'] }, branch: { type: 'string' }, sha: { type: 'string' }, reasoning: { type: 'string' } } }
const REVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['verdict', 'reasoning'], properties: {
  verdict: { type: 'string', enum: ['APPROVE', 'FIX-NEEDED'] }, substantive_ok: { type: 'boolean' }, all_green: { type: 'boolean' }, reasoning: { type: 'string' } } }

// ===== CANARY (sole upstream; gates the fan-out) =====
phase('Canary')
const canaryImpl = await agent(`${COMMON}\n## CANARY WAVE: ${CANARY.key}\n${CANARY.brief}\nYou MINT: ${CANARY.mints}. Implement it, un-ignore the canary tests, loop impl->nextest->fix until GREEN (≤${MAX_FIX} fix rounds), commit to \`${CANARY.branch}\`. Report branch + landing SHA.`,
  { label: `r5-canary:${CANARY.key}`, phase: 'Canary', isolation: 'worktree' })
phase('CanaryGate')
const canaryGate = await agent(`${COMMON}\nAdversarially mini-review the canary \`${CANARY.key}\` (branch \`${CANARY.branch}\`; read via git show). Verify: the minted surface matches the spec; its un-ignored tests are GREEN + substantive (would-FAIL-on-revert); no reserved-kw; commit landed. Emit GATE: PASS only if the fan-out can safely build on this surface; else FIX-NEEDED with the minimal fix.\nCANARY REPORT: ${canaryImpl}`,
  { label: 'r5-canary-gate', phase: 'CanaryGate', schema: GATE_SCHEMA })

if (canaryGate?.gate !== 'PASS') {
  log(`CANARY GATE: FIX-NEEDED — halting before fan-out. ${canaryGate?.reasoning || ''}`)
  return { converged: false, stage: 'canary', canaryGate, note: 'orchestrator-led canary fix-pass + re-run' }
}
log(`CANARY GATE: PASS — fanning out ${WAVES.length} waves (rebase byte-pinning families onto the canary SHA; M-20)`)

// ===== FAN-OUT WAVES (≤7 per batch) — each loops impl->un-ignore->test->fix until green, then mini-review =====
phase('Waves')
const waveResults = []
for (const batch of chunk(WAVES, 7)) {
  const res = await pipeline(batch,
    w => agent(`${COMMON}\n## IMPL WAVE: ${w.key}\n${w.brief}\nUn-ignore targets: ${w.files}. Rebase any byte-pinning family onto the canary landing SHA first (M-20). Implement, un-ignore, loop impl->nextest->fix until GREEN (≤${MAX_FIX} rounds), commit to \`${w.branch}\`. Report branch + SHA + which tests un-ignored-green.`,
      { label: `r5:${w.key}`, phase: 'Waves', isolation: 'worktree' }),
    (impl, w) => agent(`${COMMON}\nAdversarially mini-review impl wave \`${w.key}\` (branch \`${w.branch}\`; git show). Verify: all assigned tests un-ignored + GREEN + substantive (real entry + observable + would-FAIL-on-revert; NOT weakened to pass); §3.5h scoped clippy/fmt clean; no reserved-kw; commit landed. APPROVE or FIX-NEEDED.\nWAVE REPORT: ${impl}`,
      { label: `r5-review:${w.key}`, phase: 'MiniReview', schema: REVIEW_SCHEMA }).then(v => ({ wave: w.key, branch: w.branch, impl, review: v })))
  waveResults.push(...res.filter(Boolean))
}
const approved = waveResults.filter(w => w.review?.verdict === 'APPROVE')
log(`waves: ${approved.length}/${WAVES.length} APPROVE`)

// ===== INTEGRATE (strategy-C) + FULL-SUITE =====
phase('Integrate')
const integ = await agent(`${COMMON}\nIntegrator: strategy-C consolidate the canary \`${CANARY.branch}\` + the ${approved.length} approved wave branches (${JSON.stringify(approved.map(w => w.branch))}) onto an R5 integration branch off \`${CFG.baseBranch}\` (sequential, upstream/canary-first; resolve disjoint-file unions). Commit. Report the integration SHA + any conflicts.`,
  { label: 'r5-integrate', phase: 'Integrate' })
phase('FullSuite')
const suite = await agent(`${COMMON}\nRun the per-crate test suites (NOT workspace) on the R5 integration branch: ${JSON.stringify(CFG.crates || [])} via \`cargo nextest run -p <crate> <features>\` (bash; report GREEN/RED by failure-count, not masked EXIT). Confirm ZERO #[ignore]'d RED-PHASE pins remain for the implemented families (every one un-ignored + green). Report pass/fail per crate + any residual ignored pins.\nINTEGRATOR: ${integ}`,
  { label: 'r5-full-suite', phase: 'FullSuite' })

return { converged: canaryGate.gate === 'PASS' && approved.length === WAVES.length, canaryGate, waves: waveResults.map(w => ({ wave: w.wave, verdict: w.review?.verdict })), integ, suite }
