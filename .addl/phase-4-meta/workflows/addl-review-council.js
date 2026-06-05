/*
 * addl-review-council.js — FINALIZED generic adversarial review council with a SELF-COMPOSING lens set.
 *
 * One reusable shape behind every ADDL review tier that gates+mutates an artifact and iterates to convergence
 * (rule 9 / Q5): LENS-ARCHITECT (Phase 0) composes the lens set for THIS artifact → N lenses → structure →
 * adversarial-verify → completeness-critic → converge (refuse-on-partial). ITERATE ACROSS INVOCATIONS until 0 BLK/MAJ.
 *
 * ── THE LENS-ARCHITECT (Phase 0) — Pattern 6 made structural ──────────────────────────────────────────────
 * Instead of a hand-authored fixed lens set, a meta-reviewer reflects on the assigned work and COMPOSES the
 * comprehensive lens set = the applicable STANDARD lenses (baseline catalog, always considered) + BESPOKE lenses
 * unique to this artifact's surface. A second architect-reviewer (extra-reflection-pass) hunts a missed/redundant
 * lens. This PROPERLY GATES convergence: the set is complete by construction up-front, and the completeness-critic
 * at the end checks "did the architect miss a lens?" → any gap is a finding that feeds the next round's architect.
 * So you cannot falsely converge because the lens that would have caught the bug was never in the set.
 *
 * METHODOLOGY ENCODED: Pattern-6 (auto-composed lenses-by-surface) · extra-reflection-pass (architect-reviewer)
 *   · iterate-to-convergence/Q5 · adversarial-verify · completeness-critic (also lens-gap detector) · ground-truth
 *   · refuse-on-partial-panel · HARD-RULE-12 · plain-English converge call. (See workflow-common.js + README.)
 *
 * INVOKE: Workflow({ name:'addl-review-council', args:{ cfg:{...} } })
 *   args.cfg = { tier:'R1'|'R4'|'R6', artifactRef, artifactDesc, enumerate:'<cmd>', specRef, specPath, orchRef,
 *               framing, canon,
 *               standardCatalog:'<baseline lens list for this review-type>' (optional; default per tier below),
 *               mustInclude:['<lens keys that MUST be in the set>'] (optional),
 *               priorMissedLenses:'<lenses the prior round\'s completeness-critic flagged as missing>' (optional),
 *               targetLensCount: <int> (optional soft target; architect right-sizes, not pads) }
 *   args.lenses (optional) = [{key,mandate,focus}]  — if provided, SKIPS the architect and uses these verbatim.
 */

export const meta = {
  name: 'addl-review-council',
  description: 'Generic review council with self-composing lens set: lens-architect (Pattern-6) -> N lenses (batched 4) -> structure -> adversarial-verify -> completeness-critic (also lens-gap) -> converge; refuse on partial panel',
  phases: [{ title: 'LensArchitecture' }, { title: 'Review' }, { title: 'Structure' }, { title: 'Verify' }, { title: 'Completeness' }, { title: 'Converge' }],
}

const CFG = (args && args.cfg) || {}

// Baseline STANDARD lens catalogs (the always-consider set the architect selects from + extends). Override via cfg.standardCatalog.
const DEFAULT_CATALOG = {
  R1: 'spec-coherence/internal-consistency · security+threat-model · architecture-fit (matches baked-in commitments) · completeness/gaps vs requirements · feasibility/sequencing · invariant-preservation · cross-cutting-consistency · ruling/decision-fidelity · forward-class-of-bug (does the design foreclose bug classes)',
  R4: 'substantive-pin&falsifiability (would-FAIL-on-no-op) · RED-PHASE/TDD discipline (pim-12) · coverage/adversarial-gap-audit · determinism/flakiness/test-isolation/CI-realism · wire-freeze byte-correctness · encoding/serialization · invariant-semantics · capability/authority · threat-model/adversary+bounded-decode · cross-wave-seam consistency · cross-language/TS-mirror · ruling-fidelity (spec→test)',
  R4b: 'substantive-pin · new-surfaces coverage · cross-lang-mirror · named-destinations/no-phantom-deferrals · regression vs prior phase · would-FAIL-on-revert',
  R6: 'spec-to-code compliance (pim-13 R7) · all-R1/R4 standard lenses · doc-coupling/cite-drift · regression · pattern-induction (unnamed cross-cutting) · invariant-coverage at HEAD · threat-model · public-surface/SemVer · convergence-readiness',
}

const TIER = CFG.tier || 'R'
const STD = CFG.standardCatalog || DEFAULT_CATALOG[TIER] || 'task-appropriate review dimensions'

const COMMON = `
## READ-ONLY CONTRACT (NON-NEGOTIABLE)
Read ONLY via \`git show <ref>:<path>\`. NEVER checkout/cd/commit/branch/modify (the W6 escape corrupts a shared tree). First action: state the ref+SHA you read + assert it matches the brief. Ground EVERY claim in a real line; if you cannot substantiate it, DROP it (§3.5n).

## ANCHORS
- Artifact under review: ${CFG.artifactDesc} — ref \`${CFG.artifactRef}\`. Enumerate: \`${CFG.enumerate || 'git show ' + CFG.artifactRef + ':' + (CFG.specPath || '')}\`. Read: \`git show ${CFG.artifactRef}:<path>\`.
- Spec of record: \`git show ${CFG.specRef}:${CFG.specPath}\`.
${CFG.orchRef ? `- Prior-round triage / landscape: \`git show ${CFG.orchRef}:.addl/...\`` : ''}

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }

// ── FAIL-SOFT for schema agents (retry-once → safe fallback that SURFACES) ───────────────────────────
// A {schema} agent can throw ("subagent completed without calling StructuredOutput"), esp. on a clean result.
// softSchema retries once then fails to a SAFE fallback (never silently drops a finding / refutes a real one).
async function softSchema(prompt, opts, fallback, retries = 1) {
  for (let i = 0; i <= retries; i++) {
    try { const r = await agent(prompt, opts); if (r) return r; log(`schema-agent ${opts.label} returned empty (attempt ${i + 1})`) }
    catch (e) { log(`schema-agent ${opts.label} crashed attempt ${i + 1} (${String((e && e.message) || e).slice(0, 80)})`) }
  }
  log(`schema-agent ${opts.label} failed ${retries + 1}x — failing soft (orchestrator must review)`)
  return { ...fallback, _schemaCrashed: true }
}

// ── SCHEMA-FREE LENS-ARCHITECT helpers: prose pipe-format → lenses, with STD-catalog fallback ─────────
// The architect is schema-FREE because its failure mode was a 39-min StructuredOutput-retry HANG (a try/catch
// can't shorten a hang). Prose pipe-format + tolerant parse + STD-catalog fallback guarantees a usable set.
const stdCatalogLenses = std => String(std).split('·').map(s => s.trim()).filter(Boolean).map((m, i) => ({
  key: ((m.split(/[\s/(&]/)[0] || '').toLowerCase().replace(/[^a-z0-9-]/g, '').slice(0, 24)) || `std-${i}`,
  mandate: m, focus: 'the artifact', origin: 'standard' }))
const parseLenses = text => {
  if (!text || typeof text !== 'string') return []
  const out = []
  for (const line of text.split('\n')) {
    const m = line.match(/^\s*(?:[-*\d.]+\s*)?([A-Za-z0-9][A-Za-z0-9 _\/-]{1,44}?)\s*\|\s*(standard|bespoke|must-include|prior-missed)\s*\|\s*([^|]+?)\s*\|\s*(.+?)\s*$/i)
    if (m) out.push({ key: m[1].trim().toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '').slice(0, 28), origin: m[2].toLowerCase(), focus: m[3].trim(), mandate: m[4].trim() })
  }
  return out
}
const parseBar = text => { const m = text && String(text).match(/CONVERGENCE[_ ]BAR\s*[:|]\s*(.+)/i); return m ? m[1].trim() : null }
const ARCH_FMT = `EMIT ONLY this format — NO JSON, no paragraphs: ONE LINE PER LENS, exactly\n  <key> | <origin> | <focus-files> | <mandate>\nwhere origin ∈ {standard, bespoke, must-include, prior-missed}. After all lens lines, ONE final line:\n  CONVERGENCE_BAR: <what "0 findings" must mean for this work>\nBe decisive — emit the set in one pass; do not deliberate at length.`

const LENSSET_SCHEMA = { type: 'object', additionalProperties: false, required: ['lenses', 'rationale', 'convergence_bar'], properties: {
  lenses: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['key', 'mandate', 'focus', 'origin'], properties: {
    key: { type: 'string' }, mandate: { type: 'string' }, focus: { type: 'string' },
    origin: { type: 'string', enum: ['standard', 'bespoke', 'must-include', 'prior-missed'] } } } },
  rationale: { type: 'string' }, convergence_bar: { type: 'string' } } }
const FINDINGS_SCHEMA = { type: 'object', additionalProperties: false, required: ['panel_returned', 'panel_expected', 'findings'], properties: {
  panel_returned: { type: 'integer' }, panel_expected: { type: 'integer' },
  findings: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['id', 'severity', 'file', 'claim', 'recommended_disposition'], properties: {
    id: { type: 'string' }, severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
    lens: { type: 'string' }, file: { type: 'string' }, family: { type: 'string' }, claim: { type: 'string' },
    why_matters: { type: 'string' }, recommended_disposition: { type: 'string' } } } }, summary: { type: 'string' } } }
const VERDICT_SCHEMA = { type: 'object', additionalProperties: false, required: ['finding_id', 'verdict', 'corrected_severity', 'reasoning'], properties: {
  finding_id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PARTIAL'] },
  corrected_severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS', 'NONE'] }, reasoning: { type: 'string' } } }

// ===== PHASE 0: LENS ARCHITECTURE — compose the lens set for THIS artifact (Pattern 6, structural) =====
phase('LensArchitecture')
let lensSet
if (args && args.lenses && args.lenses.length) {
  lensSet = { lenses: args.lenses.map(l => ({ ...l, origin: l.origin || 'must-include' })), rationale: 'provided verbatim via args.lenses (architect skipped)', convergence_bar: '0 confirmed BLOCKER/MAJOR' }
  log(`lens set provided verbatim: ${lensSet.lenses.length} lenses (architect skipped)`)
} else {
  // SCHEMA-FREE architect (prose pipe-format → parseLenses) — avoids the StructuredOutput retry-HANG entirely.
  const composedText = await agent(`${COMMON}
## YOU ARE THE LENS ARCHITECT (meta-reviewer) for the ${TIER} review of: ${CFG.artifactDesc}
Reflect on the ACTUAL assigned work and COMPOSE the comprehensive review lens set (Pattern 6: reviewer composition follows lens surface). Steps:
1. SKIM the artifact (enumerate + read the load-bearing parts) + the spec, so you know what this work actually IS and where it can break.
2. From the STANDARD baseline catalog for ${TIER} — [ ${STD} ] — select the lenses that APPLY (and note any that are N/A for this artifact, so we don't pad).
3. INVENT the BESPOKE lenses this specific artifact needs that the standard set doesn't name (its unique surface — e.g. a crypto-construction lens for a crypto corpus, a distributed-systems lens for a sync corpus, a specific-ruling-fidelity lens, an unusual failure mode you can see in THIS work).
4. MUST-INCLUDE (always add, mark origin must-include): ${(CFG.mustInclude || []).join(', ') || '(none)'}.
5. PRIOR-MISSED (lenses a prior round's completeness-critic flagged as missing — add them, mark origin prior-missed): ${CFG.priorMissedLenses || '(none)'}.
Right-size the set${CFG.targetLensCount ? ` (soft target ~${CFG.targetLensCount} lenses)` : ' (typically 8-16; enough to cover the surface, not padded)'} — each lens DISTINCT (no two lenses that would find the same thing).
${ARCH_FMT}`,
    { label: `${TIER}-lens-architect`, phase: 'LensArchitecture' })
  let parsed = parseLenses(composedText)
  let bar = parseBar(composedText)
  if (parsed.length >= 4) {
    // extra-reflection-pass: a second architect hunts a missed/redundant lens
    const refinedText = await agent(`${COMMON}
## ARCHITECT-REVIEWER (extra-reflection-pass) for the ${TIER} lens set
A lens-architect composed the set below for: ${CFG.artifactDesc}. Reason AS BEN: (a) is there a lens this artifact NEEDS that's MISSING (a failure mode / spec obligation / adversary angle no lens covers)? (b) any two lenses REDUNDANT (would find the same thing — merge)? (c) is the convergence_bar right + complete? Return the FINAL refined lens set (add missing, merge redundant, sharpen mandates). Keep it distinct + right-sized.
${ARCH_FMT}
COMPOSED SET (same pipe format):
${composedText}`,
      { label: `${TIER}-lens-architect-review`, phase: 'LensArchitecture' })
    const refined = parseLenses(refinedText)
    if (refined.length >= 4) { parsed = refined; bar = parseBar(refinedText) || bar }
  }
  if (parsed.length < 4) { log(`architect parse yielded ${parsed.length} lenses — falling back to STD catalog (${stdCatalogLenses(STD).length} lenses)`); parsed = stdCatalogLenses(STD) }
  lensSet = { lenses: parsed, rationale: 'schema-free architect (prose pipe-format)', convergence_bar: bar || '0 confirmed BLOCKER/MAJOR + full panel + no missed lens' }
  log(`lens-architect (schema-free) composed ${lensSet.lenses.length} lenses (${lensSet.lenses.filter(l => l.origin === 'bespoke').length} bespoke). Convergence bar: ${lensSet.convergence_bar}`)
}
const LENSES = lensSet.lenses

// ===== 1. REVIEW — composed lenses, batched 4 =====
phase('Review')
const reports = []
for (const b of chunk(LENSES, 4)) {
  reports.push(...await parallel(b.map(L => () =>
    agent(`${COMMON}\n## YOUR LENS: ${L.key}\n**Focus:** ${L.focus}\n**Mandate:** ${L.mandate}\n\nReturn findings AS YOUR FINAL MESSAGE in prose. Per finding: \`[SEVERITY] <file/area> | <id> | DEFECT | WHY | DISPOSITION (fix-now | belongs-named-now(doc) | out-of-scope(reason) | disagree(cite))\`. SEVERITY ∈ {BLOCKER,MAJOR,MINOR,OBS}. End: \`LENS VERDICT: <APPROVE|FINDINGS-RAISED> — N BLK/N MAJ/N MIN/N OBS\`.`,
      { label: `${TIER}:${L.key}`, phase: 'Review' }))))
}
const live = reports.filter(Boolean)
log(`council: ${live.length}/${LENSES.length} lenses returned`)

// ===== 2. STRUCTURE =====
phase('Structure')
const structured = await softSchema(`${COMMON}\nStructuring consolidator. From the ${live.length} lens reports, extract EVERY distinct finding, dedup (highest severity + corroborating lenses), assign ids, classify recommended_disposition per HARD RULE 12. panel_returned=${live.length}, panel_expected=${LENSES.length}. Do NOT invent findings.\n**Even if the panel raised ZERO findings, you MUST STILL call the structured-output tool with an empty findings array — never end your turn without calling it.**\n=== REPORTS ===\n${live.map((r, i) => `--- ${i + 1} ---\n${r}`).join('\n')}`,
  { label: 'structure', phase: 'Structure', schema: FINDINGS_SCHEMA },
  { findings: [], panel_returned: live.length, panel_expected: LENSES.length, summary: 'structure consolidator crashed — fail-soft EMPTY (orchestrator must hand-verify before trusting a CONVERGED call this round)' })

// ===== 3. ADVERSARIAL VERIFY (BLK/MAJ, batched 4) =====
phase('Verify')
const majors = (structured?.findings || []).filter(f => f.severity === 'BLOCKER' || f.severity === 'MAJOR')
const verdicts = []
for (const b of chunk(majors, 4)) {
  const res = await parallel(b.map(f => () =>
    softSchema(`${COMMON}\n## ADVERSARIALLY REFUTE finding ${f.id} [${f.severity}] (lens ${f.lens}; file ${f.file}; ${f.family || ''})\nCLAIM: ${f.claim}\nGo to the actual artifact + spec; try to REFUTE it. Default REFUTED if you cannot independently reproduce the defect. CONFIRMED only if you reproduce it. PARTIAL if real but mis-severity. **You MUST call the structured-output tool with your verdict; never end your turn without it.**`,
      { label: `verify:${f.id}`, phase: 'Verify', schema: VERDICT_SCHEMA },
      { finding_id: f.id, verdict: 'CONFIRMED', corrected_severity: f.severity, reasoning: 'verify agent crashed — treating as UNREFUTED/CONFIRMED (safe side: keeps the finding alive + blocks false convergence); orchestrator must hand-verify' })))
  verdicts.push(...res.filter(Boolean))
}

// ===== 4. COMPLETENESS CRITIC — fresh gap-hunt + LENS-GAP detector =====
phase('Completeness')
const critic = await agent(`${COMMON}\n## COMPLETENESS CRITIC (fresh, independent) + LENS-GAP detector\nThe ${LENSES.length}-lens council (auto-composed by a lens-architect) + verify just ran. Two jobs: (1) what did the panel MISS — which spec/invariant/obligation/adversary-path has NO covering finding? (2) **LENS-GAP**: did the lens-architect MISS A LENS this artifact needed (a whole dimension no lens examined)? Name it explicitly — it feeds the next round's architect. Independently spot-check the spec vs the artifact. Return a prose GAP list (each: what's missing + severity + why + recommended action) + an explicit MISSED-LENSES list (or "none"). The composed lens set was: ${JSON.stringify(LENSES.map(l => l.key))}.`,
  { label: 'completeness-critic', phase: 'Completeness' })

// ===== 5. CONVERGE — refuse on partial panel =====
phase('Converge')
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED' || v.verdict === 'PARTIAL')
const converge = await agent(`${COMMON}\n## CONVERGENCE CONSOLIDATOR\nPANEL: ${live.length}/${LENSES.length}. **If < ${LENSES.length} returned you MUST NOT certify CONVERGED — call PANEL-INCOMPLETE + name missing lenses.** Convergence bar for this artifact (from the lens-architect): "${lensSet.convergence_bar}".\nINPUTS:\n- findings (${(structured?.findings || []).length}): ${JSON.stringify(structured?.findings || [], null, 1)}\n- adversarial verdicts: ${JSON.stringify(verdicts, null, 1)}\n- confirmed/partial: ${confirmed.length}\n- completeness-critic (incl. MISSED-LENSES):\n${critic}\nPRODUCE: (1) CONVERGENCE CALL = CONVERGED / NOT-CONVERGED / PANEL-INCOMPLETE (CONVERGED requires full panel AND zero CONFIRMED BLOCKER AND zero CONFIRMED MAJOR AND no MISSED-LENS that could hide a BLK/MAJ; REFUTED don't count; MINOR/OBS + named-deferrals don't block but MUST be named). (2) triage table (every CONFIRMED finding + disposition). (3) FIX work-list vs named carry-list. (4) any MISSED-LENSES to add next round. (5) crisp summary for orchestrator + Ben.`,
  { label: 'converge', phase: 'Converge' })

return { tier: TIER, lensSet, panel: `${live.length}/${LENSES.length}`, structured, verdicts, critic, converge }
