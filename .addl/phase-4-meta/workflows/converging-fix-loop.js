/*
 * converging-fix-loop.js — FINALIZED autonomous review→fix→re-review loop with a SELF-COMPOSING lens set.
 *
 * Loops until a full council round surfaces NO NEW BLOCKER/MAJOR. EVERY non-disagreed finding is fixed each round
 * regardless of severity (HARD RULE 12). Recoverability = audit-not-gate: every decision lands in a committed
 * decision-log for Ben's after-the-fact review; a wrong fix is rewrite-cost, not permanent (the real freeze is
 * Ben-gated downstream at G-CORE-9/tag).
 *
 * ── LENS-ARCHITECT (Phase 0) + per-round COMPLETENESS-CRITIC (lens-gap detector) — Pattern 6 made structural ──
 * The lens set is NOT hand-inlined. A meta-reviewer (Phase 0) reflects on THIS corpus + spec and COMPOSES the lens
 * set (standard catalog + bespoke-for-this-corpus + must-include + prior-missed), refined by an architect-reviewer
 * (extra-reflection-pass). EACH round also runs a completeness-critic that hunts what the panel missed AND whether
 * a whole LENS is missing: its fresh BLK/MAJ gaps join the fix-list, and its missed-lenses GROW the panel for the
 * next round (and block convergence). So you cannot falsely converge because the lens that would have caught the
 * bug was never in the set. (Same lens-architecture as addl-review-council.js.) Pass args.lenses to skip the
 * architect and use a verbatim set.
 *
 * VALIDATED MECHANICS (2026-06-02 R4-fix shakedown — feedback_workflows_as_executable_methodology):
 *   - fix agents are READ-ONLY-to-repo reasoners that EMIT full new file content to /tmp (sidesteps both the
 *     worktree-escape class AND notification-truncation);
 *   - a SINGLE integrator writes all approved /tmp files onto an integration worktree (disjoint files => no race);
 *   - compile-gate runs via BASH (zsh doesn't word-split unquoted feature strings) with ERROR-LINE detection
 *     (NOT the masked EXIT after `cargo|tail`); per-crate feature flags; watch reserved keywords (gen/etc, Rust 2024);
 *   - golden-hex fixes compute bytes once via a throwaway /tmp script + freeze a literal (M-20: R5 confirms vs encoder).
 *
 * ⚠️ FIRST-RUN-WATCHED: in `autonomous` mode the in-workflow integrator does its own git writes + cargo. Per our
 * codification, WATCH the first few autonomous runs (`/workflows`); intervene before the integrator commits if a fix
 * looks wrong. `checkpoint` mode (default) emits fixes + a fix-list and RETURNS to the orchestrator to integrate +
 * compile-gate + re-invoke. Switch to autonomous once the mechanics are proven (validated end-to-end F-full R4.3→R4.4).
 *
 * METHODOLOGY ENCODED: lens-architect/Pattern-6 · completeness-critic-as-lens-gap · HARD-RULE-12 (fix-all) ·
 *   iterate-to-convergence/Q5 · extra-reflection-pass (synthesis + review-reasoning) · plain-English/decide-as-Ben ·
 *   night-shift (decision-log + FLAGGED-FOR-BEN) · ground-truth + adversarial · pim-2 would-FAIL-on-revert ·
 *   §3.5h+§3.6j gate · §3.14 strategy-C · pattern-induction. (See workflow-common.js + README.)
 *
 * INVOKE: Workflow({ name:'converging-fix-loop', args:{ cfg:{...} } })  — args.lenses OPTIONAL (architect composes if omitted)
 *   args.cfg = { corpusBranch, mainBase, specRef, specPath, orchRef, MAX_ROUNDS, BUDGET_FLOOR, integWorktree,
 *               mode:'checkpoint'|'autonomous', crates:[{name,features}], canon:'<inline canon>',
 *               artifactDesc, standardCatalog (optional; default below), mustInclude:[], priorMissedLenses, convergenceBar }
 *   args.lenses (optional) = [{ key, mandate, focus }]   — provide to SKIP the architect (verbatim set).
 */

export const meta = {
  name: 'converging-fix-loop',
  description: 'Autonomous review->fix->re-review loop with self-composing lens set (lens-architect Phase-0 + per-round completeness-critic/lens-gap); fixes ALL non-disagreed findings each round; iterates until no new BLOCKER/MAJOR + no missed-lens; emit-to-/tmp + single-writer integrate + compile-gate + decision-log',
  phases: [
    { title: 'LensArchitecture' }, { title: 'Review' }, { title: 'Structure' }, { title: 'Completeness' },
    { title: 'Synthesis' }, { title: 'Fix' }, { title: 'Integrate' }, { title: 'Gate' }, { title: 'Terminal' },
  ],
}

const CFG = (args && args.cfg) || {}
const MAX_ROUNDS = CFG.MAX_ROUNDS || 4
const BUDGET_FLOOR = CFG.BUDGET_FLOOR || 120000
const MODE = CFG.mode || 'checkpoint'
const TMP = `/tmp/fixloop-${(CFG.corpusBranch || 'run').replace(/[^a-z0-9]/gi, '_')}`

// Baseline STANDARD lens catalog the architect selects-from + extends (default = the R4 test-corpus set). Override via cfg.standardCatalog.
const STD = CFG.standardCatalog || 'substantive-pin&falsifiability (would-FAIL-on-no-op) · RED-PHASE/TDD discipline (pim-12) · coverage/adversarial-gap-audit · determinism/flakiness/test-isolation/CI-realism · wire-freeze byte-correctness · encoding/serialization (DAG-CBOR/TLV/BE) · invariant-semantics · capability/authority/RBAC · threat-model/adversary + bounded-decode (META #629) · cross-wave-seam consistency · cross-language/TS-napi-mirror · ruling-fidelity (spec→test)'
const ARTIFACT = CFG.artifactDesc || `the ${CFG.corpusBranch} test corpus`

// ===== inline the FULL workflow-common.js COMMON_PREAMBLE + CFG.canon here at authoring/invoke time =====
const COMMON = `
## READ-ONLY-TO-REPO CONTRACT (NON-NEGOTIABLE)
Read the repo ONLY via \`git show <ref>:<path>\`. NEVER checkout/cd/commit/branch/modify the repo tree, NEVER absolute paths into the repo for writes. You MAY write throwaway scripts + your output ONLY under \`${TMP}/...\`. First action: state the ref+SHA you review and assert it matches the brief (tree-state pre-flight). Ground every claim in a real line; default REFUTED/NOT-REAL on uncertainty (§3.5n). Decide AS BEN (do-it-now, ideal permanent shape); genuine high-uncertainty => emit best-effort + FLAG-FOR-BEN, never halt.

## ANCHORS
- Corpus under review: branch \`${CFG.corpusBranch}\` (off \`${CFG.mainBase}\`). Enumerate: \`git diff --name-only --diff-filter=AM ${CFG.mainBase} ${CFG.corpusBranch}\`. Read: \`git show ${CFG.corpusBranch}:<path>\`.
- Spec of record: \`git show ${CFG.specRef}:${CFG.specPath}\`.
- Disposition (HARD RULE 12): FIX every finding regardless of severity UNLESS out-of-scope(reason) / belongs-named-now(name the doc; entry lands this round) / disagree-with-explanation(cite). "defer"/"minor enough" NEVER valid.

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }
const keyOf = f => `${f.file}::${f.family || ''}::${(f.claim || '').slice(0, 60)}`

const LENSSET_SCHEMA = { type: 'object', additionalProperties: false, required: ['lenses', 'rationale', 'convergence_bar'], properties: {
  lenses: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['key', 'mandate', 'focus', 'origin'], properties: {
    key: { type: 'string' }, mandate: { type: 'string' }, focus: { type: 'string' },
    origin: { type: 'string', enum: ['standard', 'bespoke', 'must-include', 'prior-missed'] } } } },
  rationale: { type: 'string' }, convergence_bar: { type: 'string' } } }

const FINDINGS_SCHEMA = { type: 'object', additionalProperties: false, required: ['panel_returned', 'panel_expected', 'findings'], properties: {
  panel_returned: { type: 'integer' }, panel_expected: { type: 'integer' },
  findings: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['id', 'severity', 'file', 'claim', 'real', 'disposition'], properties: {
    id: { type: 'string' }, severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
    lens: { type: 'string' }, file: { type: 'string' }, family: { type: 'string' }, claim: { type: 'string' },
    why_matters: { type: 'string' }, real: { type: 'boolean' },
    disposition: { type: 'string', enum: ['fix', 'disagree', 'out-of-scope', 'belongs-named-now'] }, disposition_reason: { type: 'string' } } } },
  summary: { type: 'string' } } }

// completeness-critic: fresh gap findings (panel misses) + missed-lenses (whole dimensions no lens covered)
const CRITIC_SCHEMA = { type: 'object', additionalProperties: false, required: ['gap_findings', 'missed_lenses'], properties: {
  gap_findings: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['id', 'severity', 'file', 'claim', 'disposition'], properties: {
    id: { type: 'string' }, severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
    file: { type: 'string' }, family: { type: 'string' }, claim: { type: 'string' }, why_matters: { type: 'string' },
    disposition: { type: 'string', enum: ['fix', 'disagree', 'out-of-scope', 'belongs-named-now'] }, disposition_reason: { type: 'string' } } } },
  missed_lenses: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['key', 'mandate', 'focus'], properties: {
    key: { type: 'string' }, mandate: { type: 'string' }, focus: { type: 'string' } } } },
  summary: { type: 'string' } } }

const FIXREVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['file', 'verdict', 'closes_findings', 'compiles_shape_ok', 'no_regression', 'reasoning'], properties: {
  file: { type: 'string' }, verdict: { type: 'string', enum: ['APPROVE', 'REVISE', 'REJECT'] },
  closes_findings: { type: 'boolean' }, compiles_shape_ok: { type: 'boolean' }, no_regression: { type: 'boolean' },
  would_fail_on_revert: { type: 'boolean' }, reasoning: { type: 'string' }, required_revision: { type: 'string' } } }

// ===== PHASE 0: LENS-ARCHITECT — compose the lens set for THIS corpus (Pattern 6, structural) =====
let LENSES, convergenceBar = CFG.convergenceBar || '0 confirmed BLOCKER/MAJOR + every freeze-gating obligation pinned + no missed lens'
const lensKeys = new Set()
if (args && args.lenses && args.lenses.length) {
  LENSES = args.lenses.map(l => ({ ...l, origin: l.origin || 'must-include' }))
  log(`lens set provided verbatim: ${LENSES.length} lenses (architect skipped)`)
} else {
  phase('LensArchitecture')
  const composed = await agent(`${COMMON}
## YOU ARE THE LENS ARCHITECT (meta-reviewer) for the fix-loop review of: ${ARTIFACT}
Reflect on the ACTUAL corpus + spec and COMPOSE the comprehensive review lens set (Pattern 6: reviewer composition follows lens surface). Steps:
1. SKIM the corpus (enumerate changed files + read the load-bearing ones) + the spec, so you know what this work actually IS and where it can break.
2. From the STANDARD baseline catalog — [ ${STD} ] — select the lenses that APPLY (note any N/A for this corpus, so we don't pad).
3. INVENT the BESPOKE lenses this specific corpus needs that the standard set doesn't name (its unique surface — e.g. a crypto-construction lens for a crypto corpus, a distributed-systems lens for a sync corpus, a specific ruling-fidelity lens, an unusual failure mode you can see in THIS work).
4. MUST-INCLUDE (always add, origin must-include): ${(CFG.mustInclude || []).join(', ') || '(none)'}.
5. PRIOR-MISSED (lenses a prior round/run flagged as missing — add them, origin prior-missed): ${CFG.priorMissedLenses || '(none)'}.
Right-size the set (typically 8-16; enough to cover the surface, not padded) — each lens DISTINCT (no two that would find the same thing). Per lens: key (short), mandate (what to hunt, specific to this corpus), focus (which files), origin. Then RATIONALE (why this set is comprehensive for THIS work) + CONVERGENCE_BAR (what "0 findings" must mean here).`,
    { label: 'lens-architect', phase: 'LensArchitecture', schema: LENSSET_SCHEMA })
  const refined = await agent(`${COMMON}
## ARCHITECT-REVIEWER (extra-reflection-pass) for the fix-loop lens set
A lens-architect composed the set below for: ${ARTIFACT}. Reason AS BEN: (a) is there a lens this corpus NEEDS that's MISSING (a failure mode / spec obligation / adversary angle no lens covers)? (b) any two lenses REDUNDANT (would find the same thing — merge)? (c) is the convergence_bar right + complete? Return the FINAL refined lens set (add missing, merge redundant, sharpen mandates). Distinct + right-sized.
COMPOSED SET: ${JSON.stringify(composed, null, 1)}`,
    { label: 'lens-architect-review', phase: 'LensArchitecture', schema: LENSSET_SCHEMA })
  const ls = refined || composed
  LENSES = ls.lenses
  convergenceBar = ls.convergence_bar || convergenceBar
  log(`lens-architect composed ${LENSES.length} lenses (${LENSES.filter(l => l.origin === 'bespoke').length} bespoke). Convergence bar: ${convergenceBar}`)
}
LENSES.forEach(l => lensKeys.add(l.key))

const seen = new Set(); const decisionLog = []; let round = 1, converged = false, lastFixList = []

while (!converged && round <= MAX_ROUNDS && (!budget.total || budget.remaining() > BUDGET_FLOOR)) {
  log(`=== ROUND ${round} === (spent ${Math.round(budget.spent() / 1000)}k; mode=${MODE}; ${LENSES.length} lenses)`)

  // 1. REVIEW — full (current) council, batched 4 (Q5)
  phase('Review')
  const reports = []
  for (const b of chunk(LENSES, 4)) {
    reports.push(...await parallel(b.map(L => () =>
      agent(`${COMMON}\n## YOUR LENS: ${L.key}\n**Focus:** ${L.focus || 'the corpus'}\n${L.mandate}\nAdversarially review the corpus from this lens. Cite real lines. Emit findings: [SEVERITY] file | F-ID | DEFECT | WHY | suggested-disposition. End: LENS VERDICT.`,
        { label: `r${round}:${L.key}`, phase: 'Review' }))))
  }
  const live = reports.filter(Boolean)

  // 2. STRUCTURE + VALIDATE — dedup, adversarially set real?, classify disposition
  phase('Structure')
  const structured = await agent(`${COMMON}\nStructuring+validation consolidator. From the ${live.length} lens reports below, extract EVERY distinct finding, dedup (keep highest severity), and for each set \`real\` by checking it against the actual corpus (default real=false if unsubstantiated) + classify \`disposition\` per HARD RULE 12. panel_returned=${live.length}, panel_expected=${LENSES.length}.\n=== REPORTS ===\n${live.map((r, i) => `--- ${i + 1} ---\n${r}`).join('\n')}`,
    { label: `r${round}-structure`, phase: 'Structure', schema: FINDINGS_SCHEMA })

  // 2.5 COMPLETENESS-CRITIC + LENS-GAP detector — fresh gap-hunt; missed-lenses grow the panel + gate convergence
  phase('Completeness')
  const critic = await agent(`${COMMON}\n## COMPLETENESS CRITIC (fresh, independent) + LENS-GAP detector — round ${round}\nThe ${LENSES.length}-lens panel (auto-composed by a lens-architect) + structuring just ran over this corpus. TWO jobs:\n(1) What did the panel MISS — a spec obligation / invariant / adversary path / freeze-gating byte with NO covering finding? Return each as a gap_finding (id, severity, file = the CONCRETE target file to fix (existing path OR a new path to mint), claim grounded in a real line, disposition per HARD RULE 12).\n(2) LENS-GAP: did the architect MISS A WHOLE LENS this corpus needed (a dimension no current lens examined)? Return each as a missed_lens (key, mandate, focus) — it GROWS next round's panel.\nCurrent lenses: ${JSON.stringify(LENSES.map(l => l.key))}. Convergence bar: "${convergenceBar}". Spot-check the spec vs the corpus independently. Default to empty arrays if nothing is substantiable.`,
    { label: `r${round}-completeness`, phase: 'Completeness', schema: CRITIC_SCHEMA })
  const gapFindings = (critic?.gap_findings || []).map(g => ({ ...g, lens: 'completeness-critic', real: true }))
  const newMissed = (critic?.missed_lenses || []).filter(l => l && l.key && !lensKeys.has(l.key))

  const all = (structured?.findings || []).concat(gapFindings)
  const toFix = all.filter(f => f.real && f.disposition === 'fix' && !seen.has(keyOf(f)))
  const newBlkMaj = toFix.filter(f => f.severity === 'BLOCKER' || f.severity === 'MAJOR')
  decisionLog.push({ round, kind: 'triage', panel: `${live.length}/${LENSES.length}`, findings: all, critic_gaps: gapFindings.length, missed_lenses: newMissed.map(l => l.key) })
  log(`round ${round}: ${all.length} findings (${gapFindings.length} critic-gap); ${toFix.length} fresh-to-fix (${newBlkMaj.length} BLK/MAJ); ${newMissed.length} new missed-lens`)

  // CONVERGENCE: full panel returned AND no NEW BLOCKER/MAJOR AND no NEW missed-lens (a missed lens could hide a BLK/MAJ)
  if (live.length < LENSES.length) { log(`PANEL-INCOMPLETE ${live.length}/${LENSES.length} — not certifying`) }
  else if (round > 1 && newBlkMaj.length === 0 && newMissed.length === 0) { converged = true }
  if (toFix.length === 0 && newMissed.length === 0) { converged = true; break }
  // grow the panel with the critic's missed-lenses (forces another round so the new dimension actually reviews the corpus)
  newMissed.forEach(l => { lensKeys.add(l.key); LENSES.push({ ...l, origin: 'prior-missed' }) })

  // 3. SYNTHESIS-as-Ben — elegant permanent shape across clusters (extra-reflection-pass)
  phase('Synthesis')
  const synthesis = await agent(`${COMMON}\nReason AS BEN over the ${toFix.length} findings below. Find the ELEGANT PERMANENT SHAPE: do any share a root cause / a single structural change closing a CLUSTER (strictly less churn; forward-class-of-bug closure)? Propose cluster-fixes + flag findings best fixed individually. Do NOT write code.\n${JSON.stringify(toFix, null, 1)}`,
    { label: `r${round}-synthesis`, phase: 'Synthesis' })

  // 4. PER-FILE FIX PIPELINE — group by file; brainstorm-as-Ben -> review-reasoning+better-shape -> EMIT to /tmp -> adversarial review-the-fix
  phase('Fix')
  const byFile = {}; for (const f of toFix) (byFile[f.file] ||= []).push(f)
  const groups = Object.entries(byFile).map(([file, fs]) => ({ file, findings: fs }))
  const cluster = `${TMP}/round${round}`
  const fixes = await pipeline(groups,
    g => agent(`${COMMON}\nReason AS BEN. Fix ALL findings for \`${g.file}\` (read via \`git show ${CFG.corpusBranch}:${g.file}\`; if it does not exist yet, MINT it). Consider the synthesis. (1) OPTIONS, (2) reasoned RECOMMENDATION as Ben (substantive-pin bar: real entry + observable consequence + would-FAIL-on-no-op; keep #[ignore]'d + self-contained stub-shim; golden-hex = compute once via a throwaway script + freeze a literal). Beware Rust-2024 reserved keywords (gen/etc). WRITE the FULL new file content to \`${cluster}/${g.file}\` (mkdir -p; /tmp only). high-uncertainty => best-effort + note FLAG-FOR-BEN.\nSYNTHESIS: ${synthesis}\nFINDINGS: ${JSON.stringify(g.findings, null, 1)}\nReturn a one-paragraph manifest: options/recommendation/better-shape/flag_for_ben.`,
      { label: `r${round}-fix:${g.file.split('/').pop()}`, phase: 'Fix' }),
    (rec, g) => agent(`${COMMON}\nReason AS BEN. A fixer wrote a new \`${g.file}\` to \`${cluster}/${g.file}\` (read it: \`cat ${cluster}/${g.file}\`). Review its REASONING as Ben + hunt a MORE IDEAL permanent shape (extra-reflection-pass). If you improve it, OVERWRITE \`${cluster}/${g.file}\`. Verify it meets the substantive-pin bar + closes every finding + stays #[ignore]'d + no Rust-2024 reserved-keyword ids.\nFIXER MANIFEST: ${rec}`,
      { label: `r${round}-review-reason:${g.file.split('/').pop()}`, phase: 'Fix' }),
    (_r, g) => agent(`${COMMON}\nAdversarially review the fix at \`${cluster}/${g.file}\` (diff vs \`git show ${CFG.corpusBranch}:${g.file}\`). Verify: closes_findings (${JSON.stringify(g.findings.map(f => f.id))}), compiles_shape_ok (valid Rust, still #[ignore]'d, self-contained shim, no reserved-kw), no_regression (no unrelated content lost), would_fail_on_revert (pim-2 substantive). Default REJECT if unconfirmable.`,
      { label: `r${round}-review-fix:${g.file.split('/').pop()}`, phase: 'Fix', schema: FIXREVIEW_SCHEMA }).then(v => ({ file: g.file, findings: g.findings, review: v })))
  const approved = fixes.filter(Boolean).filter(f => f.review?.verdict === 'APPROVE')
  lastFixList = fixes.filter(Boolean)
  decisionLog.push({ round, kind: 'fixes', synthesis, approved: approved.map(a => ({ file: a.file, ids: a.findings.map(f => f.id), review: a.review })) })
  log(`round ${round}: ${approved.length}/${groups.length} file-fixes APPROVED (emitted to ${cluster})`)

  // 5/6. INTEGRATE + GATE — autonomous (in-workflow) OR checkpoint (return to orchestrator)
  if (MODE === 'checkpoint') {
    log(`CHECKPOINT mode: fixes emitted to ${cluster}; returning to orchestrator to integrate + compile-gate + re-invoke (round ${round + 1})`)
    return { mode: 'checkpoint', round, converged: false, emittedDir: cluster, newBlkMaj: newBlkMaj.length,
      approved: approved.map(a => ({ file: a.file, ids: a.findings.map(f => f.id) })),
      missedLenses: newMissed.map(l => ({ key: l.key, mandate: l.mandate, focus: l.focus })),
      needsOrchestratorIntegration: true, decisionLog }
  }
  phase('Integrate')
  const integ = await agent(`${COMMON}\nYou are the SINGLE integrator (only writer). In the integration worktree \`${CFG.integWorktree}\` (checked out on \`${CFG.corpusBranch}\`), for each APPROVED file copy \`${cluster}/<path>\` over the repo file (mkdir -p any new path), \`git add\`, then ONE commit "test(fixloop r${round}): apply ${approved.length} reviewed fixes" (§3.14). Then COMPILE-GATE via BASH (NOT zsh): for each crate run \`cargo test -p <crate> <features> --no-run\` and report GREEN/RED by ERROR-LINE presence (never the masked EXIT after a pipe). Crates+features: ${JSON.stringify(CFG.crates || [])}. Report commit SHA + per-crate green/red + any error text.\nAPPROVED: ${JSON.stringify(approved.map(a => a.file))}`,
    { label: `r${round}-integrate`, phase: 'Integrate' })
  phase('Gate')
  const gate = await agent(`${COMMON}\nSelf-verify round ${round}: (a) compile-gate GREEN behind #[ignore]; (b) §3.5h scoped clippy+fmt clean on touched crates; (c) §3.6j every added cite resolves; (d) §3.5g any new ErrorCode/wire-type has a named TS/napi mirror. Report PASS or the minimal residual to fix.\nINTEGRATOR: ${integ}`,
    { label: `r${round}-gate`, phase: 'Gate' })
  decisionLog.push({ round, kind: 'integrate', integ, gate })
  toFix.forEach(f => seen.add(keyOf(f)))
  round++
}

// TERMINAL — pattern-induction + return audit trail
phase('Terminal')
const patternInduction = await agent(`${COMMON}\nPattern-induction meta-sweep over the decision log (${decisionLog.length} entries). Hunt UNNAMED cross-cutting patterns (>=3 recurrence => pim-N candidate); any class-of-bug the loop kept re-fixing (deeper root cause); any LENS the architect kept having to add (a standard-catalog gap); §3.6h: if you propose a rule naming an origin instance, it must close that origin same-landing. Return a prose list.\nLOG: ${JSON.stringify(decisionLog.map(d => ({ round: d.round, kind: d.kind, n: (d.findings || d.approved || []).length, missed: d.missed_lenses })), null, 1)}`,
  { label: 'pattern-induction', phase: 'Terminal' })

return { mode: MODE, converged, rounds: round - 1, finalLensCount: LENSES.length, decisionLog, patternInduction, lastFixList }
