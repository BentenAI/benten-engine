/*
 * addl-r2-test-landscape.js — FINALIZED R2 test-landscape synthesis (multi-modal sweep + completeness critic).
 *
 * R2 turns the ratified R1 plan into the test FAMILY catalog the R3 writers slice: N discovery dimensions sweep
 * the plan in parallel (each blind to the others), a completeness critic hunts what every dimension MISSED
 * (the R2-2026-06-02 critic added 22 gap-fill families the 6 discovery agents missed), then a synthesis agent
 * dedups into a unified catalog + coverage matrix (every invariant/compromise/codepoint/NQ/exit-criterion -> its
 * family; zero uncovered) + the canary-first R3 wave-slicing + freeze-gating priorities.
 *
 * METHODOLOGY ENCODED: multi-modal-sweep (each dim blind) · completeness-critic (fresh gap-hunt) · coverage-matrix
 *   (every spec obligation -> a family) · canary-first R3 slicing · freeze-gating priority. (See workflow-common.js.)
 *
 * INVOKE: Workflow({ name:'addl-r2-test-landscape', args:{ cfg:{...}, dimensions:[...] } })
 *   args.cfg        = { specRef, specPath, orchRef, canon, r3CanaryHint }
 *   args.dimensions = [{ key, mandate }]   (the discovery lenses — e.g. crypto-envelope / sync-crdt / threat / wire-freeze / privacy / graph-native)
 */

export const meta = {
  name: 'addl-r2-test-landscape',
  description: 'R2: multi-modal discovery sweep (N dims, batched) -> completeness critic -> synthesis (family catalog + coverage matrix + canary-first R3 slicing + freeze-gating priorities)',
  phases: [{ title: 'Discovery' }, { title: 'Completeness' }, { title: 'Synthesis' }],
}

const CFG = (args && args.cfg) || {}
const DIMS = (args && args.dimensions) || []

const COMMON = `
You map the TEST LANDSCAPE for an ADDL phase: enumerate the test FAMILIES the red-phase writers (R3) will author from the ratified plan.

## READ-ONLY CONTRACT
Read ONLY via \`git show <ref>:<path>\`. NEVER checkout/cd/commit/modify. Ground every family in a specific plan obligation (cite the §/invariant/compromise/codepoint/NQ/exit-criterion).

## ANCHORS
- Ratified plan (the spec families must cover): \`git show ${CFG.specRef}:${CFG.specPath}\`.
${CFG.orchRef ? `- Prior artifacts: \`git show ${CFG.orchRef}:.addl/...\`` : ''}

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }

// 1. DISCOVERY — N dims, each blind, batched 4 (schema-free prose)
phase('Discovery')
const found = []
for (const b of chunk(DIMS, 4)) {
  found.push(...await parallel(b.map(d => () =>
    agent(`${COMMON}\n## YOUR DISCOVERY DIMENSION: ${d.key}\n${d.mandate}\nEmit a list of test FAMILIES from your dimension: per family — an F-ID, what it pins, the plan obligation it covers (cite), freeze-gating? (Y/N), rough test count, the headline assertion. Be exhaustive from YOUR angle; you are blind to the other dimensions.`,
      { label: `r2:${d.key}`, phase: 'Discovery' }))))
}
const live = found.filter(Boolean)
log(`discovery: ${live.length}/${DIMS.length} dimensions returned`)

// 2. COMPLETENESS CRITIC — what did EVERY dimension miss?
phase('Completeness')
const critic = await agent(`${COMMON}\n## COMPLETENESS CRITIC\nThe ${live.length} discovery dimensions below swept the plan. Now hunt what they ALL MISSED: walk the plan and ask — which Invariant / Compromise / codepoint / NQ-* / §-exit-criterion / threat-path / wire-field has NO test family proposed? Each gap => a NEW gap-fill family (F-ID + what it pins + the obligation + freeze-gating?). Be adversarial; the discovery agents are individually narrow.\n=== DISCOVERY ===\n${live.map((r, i) => `--- dim ${i + 1} ---\n${r}`).join('\n')}`,
  { label: 'r2-completeness-critic', phase: 'Completeness' })

// 3. SYNTHESIS — unified catalog + coverage matrix + R3 slicing
phase('Synthesis')
const synthesis = await agent(`${COMMON}\n## SYNTHESIS — the R2 test-landscape deliverable\nFrom the ${live.length} discovery dimensions + the completeness-critic gap-fills below, produce: (1) the UNIFIED FAMILY CATALOG (dedup overlapping families across dimensions; group into family-groups; per family: F-ID, what-it-pins, obligation, freeze-gating?, rough count). (2) the COVERAGE MATRIX: every plan obligation (each Invariant / Compromise / codepoint / NQ / exit-criterion) -> its covering family; assert ZERO uncovered (or list gaps). (3) the canary-first R3 WAVE-SLICING (the sole-upstream canary wave${CFG.r3CanaryHint ? ' (' + CFG.r3CanaryHint + ')' : ''} + the fan-out waves by disjoint slice; ≤7 per batch). (4) the FREEZE-GATING priority list (which families pin permanent wire bytes — author/verify first). Output as a structured markdown doc the orchestrator commits as the R2 landscape.\n=== DISCOVERY ===\n${live.map((r, i) => `--- dim ${i + 1} ---\n${r}`).join('\n')}\n=== COMPLETENESS-CRITIC GAP-FILLS ===\n${critic}`,
  { label: 'r2-synthesis', phase: 'Synthesis' })

return { dimensions: `${live.length}/${DIMS.length}`, discovery: live, critic, synthesis }
