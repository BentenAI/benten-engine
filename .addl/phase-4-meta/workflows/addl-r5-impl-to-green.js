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
 *
 * AUTHORING NOTES (orchestrator, when writing the concrete invocation):
 *  - Thread the r4-triage §3.B R5-FILL carry-list into the wave briefs (real-NIST/draft-connolly KAT swap,
 *    bounded-decode family, substrate-wiring verify-notes, §3.5g ErrorCode mints).
 *  - Make the LAST wave a DOC-WAVE (Rule #6 — docs in the last group): ERROR-CATALOG + CATALOG_VARIANT_COUNT,
 *    spec retenses, missing_docs sweep — not an afterthought.
 *  - CI is the AUTHORITATIVE full-workspace gate: the workflow's per-crate FullSuite is a pre-CI proxy; take the
 *    integration branch -> PR -> CI -> NORMAL --squash merge (never auto-merge; never --admin-bypass).
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

## M-20 GOLDEN RECONCILIATION (freeze-byte safety — MANDATORY for every byte-pinning test)
The corpus goldens were computed by STUBS (M-20: frozen-vs-stub) and HAND-VERIFIED at R4 against the stub construction. R5 has the REAL production encoder for the FIRST time. For EVERY golden-hex / frozen-byte test you un-ignore: RECOMPUTE the golden via the REAL encoder and CONFIRM it byte-matches the frozen literal. If it DIFFERS, the stub may have frozen a wrong byte — UPDATE the golden to the real-encoder output AND FLAG-FOR-BEN in your report ("golden NAME: stub=… real=…; updated") so the orchestrator hand-verifies the wire-freeze. NEVER weaken the assertion to pass; reconcile to the real bytes. (This is the freeze-byte gate that caught the codepoint + body_cid issues by hand at R4.)

## §3.5g CROSS-LANGUAGE ERRORCODE MIRROR
If your wave MINTS an ErrorCode / any error variant crossing the public/napi/wire surface: atomically update ALL of (Rust variant + \`packages/engine/src/errors.generated.ts\` + \`docs/ERROR-CATALOG.md\` + \`CATALOG_VARIANT_COUNT\`) in the SAME commit. Drift between them is a FIX-NEEDED.

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }

// FAIL-SOFT for schema agents: a schema agent can crash ("subagent completed without calling StructuredOutput",
// esp. on a clean/simple result) — that killed a CONVERGED R4.6 run this session. softSchema catches it + fails to
// the SAFE side (a fallback verdict that SURFACES for the orchestrator), NEVER letting one crash kill the whole
// (expensive) R5 run. Use for every {schema} agent here.
async function softSchema(prompt, opts, fallback) {
  try { const r = await agent(prompt, opts); return r || { ...fallback, _empty: true } }
  catch (e) { log(`schema-agent ${opts.label} CRASHED (${String(e && e.message || e).slice(0, 90)}) — failing soft (orchestrator must review)`); return { ...fallback, _schemaCrashed: true } }
}

const GATE_SCHEMA = { type: 'object', additionalProperties: false, required: ['gate', 'reasoning'], properties: {
  gate: { type: 'string', enum: ['PASS', 'FIX-NEEDED'] }, branch: { type: 'string' }, sha: { type: 'string' }, reasoning: { type: 'string' } } }
const REVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['verdict', 'reasoning'], properties: {
  verdict: { type: 'string', enum: ['APPROVE', 'FIX-NEEDED'] }, substantive_ok: { type: 'boolean' }, all_green: { type: 'boolean' }, reasoning: { type: 'string' } } }

// ===== CANARY (sole upstream; gates the fan-out) =====
phase('Canary')
const canaryImpl = await agent(`${COMMON}\n## CANARY WAVE: ${CANARY.key}\n${CANARY.brief}\nYou MINT: ${CANARY.mints}. Implement it, un-ignore the canary tests, loop impl->nextest->fix until GREEN (≤${MAX_FIX} fix rounds), commit to \`${CANARY.branch}\`. Report branch + landing SHA.`,
  { label: `r5-canary:${CANARY.key}`, phase: 'Canary', isolation: 'worktree' })
phase('CanaryGate')
const canaryGate = await softSchema(`${COMMON}\nAdversarially mini-review the canary \`${CANARY.key}\` (branch \`${CANARY.branch}\`; read via git show). Verify: the minted surface matches the spec; its un-ignored tests are GREEN + substantive (would-FAIL-on-revert); no reserved-kw; commit landed. Emit GATE: PASS only if the fan-out can safely build on this surface; else FIX-NEEDED with the minimal fix.\nCANARY REPORT: ${canaryImpl}`,
  { label: 'r5-canary-gate', phase: 'CanaryGate', schema: GATE_SCHEMA },
  { gate: 'FIX-NEEDED', reasoning: 'canary-gate schema-agent crashed — HALT: orchestrator must hand-review the canary before any fan-out (fail-soft to the safe side, never auto-PASS).' })

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
    (impl, w) => softSchema(`${COMMON}\nAdversarially mini-review impl wave \`${w.key}\` (branch \`${w.branch}\`; git show). Verify: all assigned tests un-ignored + GREEN + substantive (real entry + observable + would-FAIL-on-revert; NOT weakened to pass); M-20 goldens reconciled vs the real encoder; §3.5h scoped clippy/fmt clean; no reserved-kw; commit landed. APPROVE or FIX-NEEDED.\nWAVE REPORT: ${impl}`,
      { label: `r5-review:${w.key}`, phase: 'MiniReview', schema: REVIEW_SCHEMA },
      { verdict: 'FIX-NEEDED', reasoning: 'review schema-agent crashed — orchestrator must hand-review this wave (fail-soft).' }).then(v => ({ wave: w.key, branch: w.branch, impl, review: v })))
  waveResults.push(...res.filter(Boolean))
}
const approved = waveResults.filter(w => w.review?.verdict === 'APPROVE')
const needFix = waveResults.filter(w => w.review?.verdict !== 'APPROVE')
// DROP-GUARD (dogfood-#1 fold-back): a wave whose pipeline THREW -> null -> got filtered out of waveResults
// is a DROPPED wave (agent died / PANEL-INCOMPLETE), NOT a silent success. The needFix check below can't see it
// (it's absent, not FIX-NEEDED). Count + HALT explicitly, else a dropped wave sails into a partial integrate.
const dropped = WAVES.length - waveResults.length
log(`waves: ${approved.length}/${WAVES.length} APPROVE; ${needFix.length} FIX-NEEDED; ${dropped} DROPPED`)
if (dropped > 0 || needFix.length) {
  log(`HALT before integrate — ${needFix.length} FIX-NEEDED + ${dropped} DROPPED. Do NOT integrate a PARTIAL set (a half-implemented corpus is worse than none). Orchestrator: fix-pass / re-run the missing waves.`)
  return { converged: false, stage: 'waves', status: dropped > 0 ? 'PANEL-INCOMPLETE' : 'FIX-NEEDED', dropped, approvedWaves: approved.map(w => w.wave), needFix: needFix.map(w => ({ wave: w.wave, branch: w.branch, review: w.review })), note: 'orchestrator-led wave fix-pass / re-run before integrate (partial-integrate suppressed)' }
}

// ===== INTEGRATE (strategy-C) + FULL-SUITE — only reached when ALL waves APPROVE =====
phase('Integrate')
const integ = await agent(`${COMMON}\nIntegrator: strategy-C consolidate the canary \`${CANARY.branch}\` + the ${approved.length} approved wave branches (${JSON.stringify(approved.map(w => w.branch))}) onto an R5 integration branch off \`${CFG.baseBranch}\` (sequential, upstream/canary-first; resolve disjoint-file unions). Commit. Report the integration SHA + any conflicts.`,
  { label: 'r5-integrate', phase: 'Integrate' })
phase('FullSuite')
const suite = await agent(`${COMMON}\nRun the per-crate test suites (NOT workspace) on the R5 integration branch: ${JSON.stringify(CFG.crates || [])} via \`cargo nextest run -p <crate> <features>\` (bash; report GREEN/RED by failure-count, not masked EXIT). Confirm ZERO #[ignore]'d RED-PHASE pins remain for the implemented families (every one un-ignored + green). Report pass/fail per crate + any residual ignored pins.\nINTEGRATOR: ${integ}`,
  { label: 'r5-full-suite', phase: 'FullSuite' })

return { converged: canaryGate.gate === 'PASS' && approved.length === WAVES.length, canaryGate, waves: waveResults.map(w => ({ wave: w.wave, verdict: w.review?.verdict })), integ, suite }
