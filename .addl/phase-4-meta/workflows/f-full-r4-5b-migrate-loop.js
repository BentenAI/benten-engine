/*
 * converging-fix-loop.js — FINALIZED autonomous review→fix→re-review loop.
 *
 * Loops until a full council round surfaces NO NEW BLOCKER/MAJOR. EVERY non-disagreed finding is
 * fixed each round regardless of severity (HARD RULE 12). Recoverability = audit-not-gate: every
 * decision lands in a committed decision-log for Ben's after-the-fact review; a wrong fix is
 * rewrite-cost, not permanent (the real freeze is Ben-gated downstream at G-CORE-9/tag).
 *
 * VALIDATED MECHANICS (from the 2026-06-02 R4-fix shakedown — see feedback_workflows_as_executable_methodology):
 *   - fix agents are READ-ONLY-to-repo reasoners that EMIT full new file content to /tmp (NOT return-body:
 *     sidesteps both the worktree-escape class AND notification-truncation).
 *   - a SINGLE integrator writes all approved /tmp files onto an integration worktree (disjoint files => no race).
 *   - compile-gate runs via BASH (zsh doesn't word-split unquoted feature strings) with ERROR-LINE detection
 *     (NOT the masked EXIT after `cargo|tail`); per-crate feature flags; watch for reserved-keyword (gen/etc, Rust 2024).
 *   - golden-hex fixes compute bytes once via a throwaway /tmp script + freeze a literal (M-20: R5 confirms vs real encoder).
 *
 * ⚠️ FIRST-RUN-WATCHED: the in-workflow integrator does its own git writes + cargo. Per our codification, WATCH
 * the first few autonomous runs (`/workflows`); intervene before the integrator commits if a fix looks wrong.
 * For maximum safety set args.mode='checkpoint' (the loop emits fixes + a fix-list and RETURNS to the orchestrator
 * to integrate + re-invoke) instead of 'autonomous' (the loop integrates in-workflow). Default = 'checkpoint'.
 *
 * METHODOLOGY ENCODED: HARD-RULE-12 · iterate-to-convergence/Q5 · extra-reflection-pass (synthesis + review-reasoning)
 *   · plain-English/decide-as-Ben · night-shift (decision-log + FLAGGED-FOR-BEN) · ground-truth+adversarial
 *   · pim-2 would-FAIL-on-revert · §3.5h+§3.6j gate · §3.14 strategy-C · pattern-induction. (See workflow-common.js.)
 *
 * INVOKE: Workflow({ name:'converging-fix-loop', args:{ cfg:{...}, lenses:[...] } })
 *   args.cfg = { corpusBranch, mainBase, specRef, specPath, orchRef, MAX_ROUNDS, BUDGET_FLOOR,
 *               integWorktree, mode:'checkpoint'|'autonomous', crates:[{name,features}], canon:'<inline canon text>' }
 *   args.lenses = [{ key, mandate }]   (the review lens-set for THIS artifact; see addl-review-council.js for R1/R4 sets)
 */

export const meta = {
  name: 'converging-fix-loop',
  description: 'Autonomous review->fix->re-review loop; fixes ALL non-disagreed findings each round; iterates until no new BLOCKER/MAJOR; emit-to-/tmp + single-writer integrate + compile-gate + decision-log',
  phases: [
    { title: 'Review' }, { title: 'Structure' }, { title: 'Verify' },
    { title: 'Synthesis' }, { title: 'Fix' }, { title: 'Integrate' }, { title: 'Gate' }, { title: 'Terminal' },
  ],
}

// ===== R4.3 INLINED CONFIG (args-passing was flaky for large objects; inline like the f-full-* scripts) =====
const CFG = {
  corpusBranch: 'phase-4-meta-core/f-full-r4-fix',
  mainBase: '2172cb6d',
  specRef: 'phase-4-meta-core/f-full-r0-plan-r06',
  specPath: '.addl/phase-4-meta/f-full-r0-plan.md',
  orchRef: 'phase-4-meta-core/orchestration-2026-05-26',
  MAX_ROUNDS: 3,
  BUDGET_FLOOR: 150000,
  mode: 'autonomous',
  integWorktree: '/Users/benwork/Documents/benten-wt-fixint',
  crates: [
    { name: 'benten-crypto-suite', features: '--features testing' },
    { name: 'benten-membership-set', features: '--features testing' },
    { name: 'benten-drop', features: '--features testing' },
    { name: 'benten-engine', features: '--features test-helpers --features benten-eval/testing' },
  ],
  canon: `## F-FULL CANON — R4.5 SS-AAD MIGRATION TO R0.6 (autonomous converging-fix-loop)
- Spec of record is now R0.6 (branch phase-4-meta-core/f-full-r0-plan-r06). It records 7 Ben-RATIFIED Sealed-Sender AAD freeze decisions. The corpus (f-full-r4-fix @ 85ae77bc) predates R0.6 and still holds the OLD shape in places. YOUR JOB each round: MIGRATE the corpus to match R0.6, regenerate every affected golden-hex (M-20: compute the bytes ONCE via a throwaway script, freeze a literal), and CONVERGE (0 new BLOCKER/MAJOR).
- THE MIGRATION (what R0.6 NOW mandates vs what the corpus has):
  1. 0x6610 group per-stanza AAD (benten-membership-set f_aad_2 assemble_aad_9tuple + golden EXPECTED_AAD_HEX): BLIND two fields. Replace raw sorted_member_dids[] with audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || ...) over the canonical SORTED recipient-DID list (32 bytes; lp = u32-BE length prefix). Replace raw membership_set_id with membership_set_id_commitment = HMAC(K_Set, "benten:setid:v1" || membership_set_id) truncated 32 bytes. ADD stanza_count (u32 BE) alongside stanza_index. Canonical 11-field order per R0.6 §3.10/§4.1: { aad_version(0x01,u8), codepoint(0x6610,u16 BE), body_cid(self-describing CIDv1), member_count(u32 BE), audience_set_commitment(32B), stanza_index(u32 BE), stanza_count(u32 BE), member_key_generation(u32 BE), membership_set_id_commitment(32B), membership_set_generation(u32 BE), role_assignments_generation(u32 BE) }.
  CRITICAL: reuse the EXACT commitment constructions the corpus ALREADY uses for the §3.9 gossip topic (read the gossip / topic-blinding test and reuse its HMAC primitive + truncation + its BLAKE3 helper) — do NOT invent a different HMAC or hash.
  2. body_cid -> self-describing CIDv1 on BOTH bands: bytes 0x01 0x71 0x1e 0x20 || 32-byte BLAKE3 digest. NOT a bare 32-byte digest. Applies to 0x6510 (benten-drop f_lc_hpke DropSealedSender + f_lc_abuse SealedSenderAad + benten-crypto-suite f_inv16_1 Recipient) AND 0x6610 (f_aad_2). Regenerate every affected golden (incl. F_INV18_1_SEALED_AAD_HEX).
  3. 0x6510 single-recipient AAD = Option A union { aad_version(0x01), codepoint(0x6510,u16 BE), audience(recipient DID, u32-BE length-prefixed), body_cid(self-describing CIDv1 per #2), recipient_key_generation(u32 BE) }. Already correct EXCEPT body_cid framing (#2). Confirm + apply #2.
  4. token-binding AAD (f_lc_abuse TokenBindingAad): NO coarse_epoch (already removed); confirm + aad_version byte == 0x01.
- ALREADY-CORRECT — do NOT re-flag/re-litigate: every R4.1-R4.4 fix is verified-correct (X-Wing SHA3-256 APPEND + 6-byte XWING_LABEL pin; MemberRef+RoleId int-discriminant; AAD_VERSION 0x01; DeviceAuthBackend sealed open=6; bounded-decode flagship two-bound; audience reconciliation). SETTLED: coarse_epoch NOT on Drop wire; u16/u32 per-band widths NOT unified (separately-frozen bands); identity-hiding-not-unlinkability is accepted (full unlinkability = U25 v1-GM-reserve — do NOT add salt now).
- HONEST SCOPE: the group-AAD blinding is identity-HIDING (the commitment recurs for a static group). Recipients hold K_Set + the member list -> recompute + verify both commitments -> ALL bindings (cross-stanza substitution U17) PRESERVED; relay sees only opaque 32-byte tags.
- Substantive-pin bar (pim-2): every migrated test stays ignored behind #[ignore] + self-contained stub-shim + asserts the NEW frozen bytes + would-FAIL-on-no-op. Golden-hex = M-20 (frozen vs stub; R5 confirms vs real encoder). Beware Rust-2024 reserved keyword gen.
- Codepoints: Sealed-Sender 0x6510 DEFAULT; MembershipSet 0x6600/0x6610; Layer-C 0x6500/0x6510/0x6520; X-Wing 0x647a; vault 0x6100(24B)/0x6101(12B); sig LAMPS 0x0001.`,
}
const LENSES = [
  { key: 'ss-aad-migration-fidelity', mandate: 'Every R0.6 SS-AAD freeze decision is faithfully applied: 0x6610 group AAD has audience_set_commitment + membership_set_id_commitment (NOT raw roster/set-id) + stanza_count; 0x6510 unchanged except body_cid; body_cid is self-describing CIDv1 (0x01 0x71 0x1e 0x20 || digest) on BOTH bands; token-binding AAD has NO coarse_epoch + aad_version 0x01. Flag any file still on the OLD raw shape.', focus: 'f_aad_2, f_lc_hpke, f_lc_abuse, f_inv16_1' },
  { key: 'commitment-construction-correctness', mandate: 'audience_set_commitment = BLAKE3(0x01 || lp(did_i)...) over CANONICAL SORTED DID list; membership_set_id_commitment = HMAC(K_Set,"benten:setid:v1"||id) trunc 32B reusing the EXACT corpus §3.9 gossip-topic construction (NOT a freshly-invented HMAC). Verify it matches the corpus topic-blinding helper; check domain-sep byte / lp width / truncation.', focus: 'f_aad_2, the §3.9 gossip/topic-blinding test' },
  { key: 'golden-hex-byte-correctness', mandate: 'Every regenerated golden (EXPECTED_AAD_HEX, F_INV18_1_SEALED_AAD_HEX) is a real frozen literal computed from the NEW construction (M-20 throwaway-compute), byte-length correct (commitments 32B, CIDv1 36B), and would-FAIL-on-revert. No self-referential / tautological hex.', focus: 'all migrated goldens' },
  { key: 'wire-freeze-byte-correctness', mandate: 'All OTHER frozen bytes UNREGRESSED vs the R4.1-R4.4 fixes (X-Wing APPEND + XWING_LABEL pin, MemberRef/RoleId int-discriminant, AAD_VERSION 0x01, DeviceAuthBackend sealed open=6). The migration must not disturb them.', focus: 'f_w0, f_kat_*, f_sm, f_freeze_1' },
  { key: 'cross-file-seam-consistency', mandate: 'f_aad_2 / f_lc_hpke / f_lc_abuse / f_inv16_1 AGREE on the body_cid CIDv1 framing + the 0x6510/0x6610 field-sets; no divergent bytes for the same wire object; the Inv-16 BindingContext::Recipient unification surface matches.', focus: 'cross-cutting' },
  { key: 'substantive-pin-falsifiability', mandate: 'pim-2: every migrated test stays ignored behind #[ignore] + self-contained stub-shim + asserts the NEW frozen bytes + would-FAIL-on-no-op; no tautology introduced by the migration.', focus: 'all touched files' },
  { key: 'invariant-semantics', mandate: 'Inv-16..22 + Inv-18 paired-disclosure + Inv-20 clause-c still hold post-migration; the blinding PRESERVES cross-stanza substitution (U17) + inter-member non-forgeability; recipients recompute-and-verify modeled.', focus: 'f_inv*, f_aad_2' },
  { key: 'ruling-fidelity', mandate: 'Every one of the 7 R0.6 decisions maps to a pin; do NOT relitigate SETTLED rulings (coarse_epoch-not-on-Drop-wire; u16/u32 widths NOT unified; identity-hiding-not-unlinkability accepted, do NOT add salt). Flag any test contradicting R0.6.', focus: 'cross-cutting' },
  { key: 'threat-model-bounded-decode', mandate: 'The blinding does not weaken any binding; bounded/length-checked decode still pinned (META #629); no new unbounded-decode path; the 32B commitments + CIDv1 are length-bounded.', focus: 'f_inv16_1, f_lc_*, f_aad_2' },
  { key: 'red-phase-r5-readiness', mandate: 'pim-12: every migrated test stays ignored behind a RED-PHASE message + valid un-ignore destinations; doc-comments cite R0.6 (not R0.5/R0.3); compiles green behind #[ignore].', focus: 'all + f_aad_2' },
  { key: 'coverage-gap-audit', mandate: 'The migration is COMPLETE: no file left on the old raw-roster shape; no R0.6 obligation unpinned; no NEW gap. Grep that raw sorted_member_dids / raw membership_set_id are GONE from AAD assembly (survive only inside commitment inputs).', focus: 'r0.6 + corpus' },
  { key: 'distributed-sync', mandate: 'The §3.9 gossip-topic construction reuse is consistent (set-id commitment == the topic blinding fn applied to the set-id); CRDT/HLC/MST/Inv-21 unregressed.', focus: 'f_gossip, f_crdt, f_inv21, f_aad_2' },
]
const MAX_ROUNDS = CFG.MAX_ROUNDS || 4
const BUDGET_FLOOR = CFG.BUDGET_FLOOR || 120000
const MODE = CFG.mode || 'checkpoint'
const TMP = `/tmp/fixloop-r45b-${(CFG.corpusBranch || 'run').replace(/[^a-z0-9]/gi, '_')}`

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

const FINDINGS_SCHEMA = { type: 'object', additionalProperties: false, required: ['panel_returned', 'panel_expected', 'findings'], properties: {
  panel_returned: { type: 'integer' }, panel_expected: { type: 'integer' },
  findings: { type: 'array', items: { type: 'object', additionalProperties: false, required: ['id', 'severity', 'file', 'claim', 'real', 'disposition'], properties: {
    id: { type: 'string' }, severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
    lens: { type: 'string' }, file: { type: 'string' }, family: { type: 'string' }, claim: { type: 'string' },
    why_matters: { type: 'string' }, real: { type: 'boolean' },
    disposition: { type: 'string', enum: ['fix', 'disagree', 'out-of-scope', 'belongs-named-now'] }, disposition_reason: { type: 'string' } } } },
  summary: { type: 'string' } } }

const FIXREVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['file', 'verdict', 'closes_findings', 'compiles_shape_ok', 'no_regression', 'reasoning'], properties: {
  file: { type: 'string' }, verdict: { type: 'string', enum: ['APPROVE', 'REVISE', 'REJECT'] },
  closes_findings: { type: 'boolean' }, compiles_shape_ok: { type: 'boolean' }, no_regression: { type: 'boolean' },
  would_fail_on_revert: { type: 'boolean' }, reasoning: { type: 'string' }, required_revision: { type: 'string' } } }

const seen = new Set(); const decisionLog = []; let round = 1, converged = false, lastFixList = []

while (!converged && round <= MAX_ROUNDS && (!budget.total || budget.remaining() > BUDGET_FLOOR)) {
  log(`=== ROUND ${round} === (spent ${Math.round(budget.spent() / 1000)}k; mode=${MODE})`)

  // 1. REVIEW — full council, batched 4 (Q5)
  phase('Review')
  const reports = []
  for (const b of chunk(LENSES, 4)) {
    reports.push(...await parallel(b.map(L => () =>
      agent(`${COMMON}\n## YOUR LENS: ${L.key}\n${L.mandate}\nAdversarially review the corpus from this lens. Cite real lines. Emit findings: [SEVERITY] file | F-ID | DEFECT | WHY | suggested-disposition. End: LENS VERDICT.`,
        { label: `r${round}:${L.key}`, phase: 'Review' }))))
  }
  const live = reports.filter(Boolean)

  // 2. STRUCTURE + VALIDATE — dedup, adversarially set real?, classify disposition
  phase('Structure')
  const structured = await agent(`${COMMON}\nStructuring+validation consolidator. From the ${live.length} lens reports below, extract EVERY distinct finding, dedup (keep highest severity), and for each set \`real\` by checking it against the actual corpus (default real=false if unsubstantiated) + classify \`disposition\` per HARD RULE 12. panel_returned=${live.length}, panel_expected=${LENSES.length}.\n=== REPORTS ===\n${live.map((r, i) => `--- ${i + 1} ---\n${r}`).join('\n')}`,
    { label: `r${round}-structure`, phase: 'Structure', schema: FINDINGS_SCHEMA })

  const all = structured?.findings || []
  const toFix = all.filter(f => f.real && f.disposition === 'fix' && !seen.has(keyOf(f)))
  const newBlkMaj = toFix.filter(f => f.severity === 'BLOCKER' || f.severity === 'MAJOR')
  decisionLog.push({ round, kind: 'triage', panel: `${live.length}/${LENSES.length}`, findings: all })
  log(`round ${round}: ${all.length} findings; ${toFix.length} fresh-to-fix (${newBlkMaj.length} BLK/MAJ)`)

  // CONVERGENCE: full panel returned AND no NEW BLOCKER/MAJOR
  if (live.length < LENSES.length) { log(`PANEL-INCOMPLETE ${live.length}/${LENSES.length} — not certifying`); }
  else if (round > 1 && newBlkMaj.length === 0) { converged = true }
  if (toFix.length === 0) { converged = true; break }

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
    g => agent(`${COMMON}\nReason AS BEN. Fix ALL findings for \`${g.file}\` (read via \`git show ${CFG.corpusBranch}:${g.file}\`). Consider the synthesis. (1) OPTIONS, (2) reasoned RECOMMENDATION as Ben (substantive-pin bar: real entry + observable consequence + would-FAIL-on-no-op; keep #[ignore]'d + self-contained stub-shim; golden-hex = compute once via a throwaway script + freeze a literal). Beware Rust-2024 reserved keywords (gen/etc). WRITE the FULL new file content to \`${cluster}/${g.file}\` (mkdir -p; /tmp only). high-uncertainty => best-effort + note FLAG-FOR-BEN.\nSYNTHESIS: ${synthesis}\nFINDINGS: ${JSON.stringify(g.findings, null, 1)}\nReturn a one-paragraph manifest: options/recommendation/better-shape/flag_for_ben.`,
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
    return { mode: 'checkpoint', round, converged: false, emittedDir: cluster, approved: approved.map(a => ({ file: a.file, ids: a.findings.map(f => f.id) })), needsOrchestratorIntegration: true, decisionLog }
  }
  phase('Integrate')
  const integ = await agent(`${COMMON}\nYou are the SINGLE integrator (only writer). In the integration worktree \`${CFG.integWorktree}\` (checked out on \`${CFG.corpusBranch}\`), for each APPROVED file copy \`${cluster}/<path>\` over the repo file, \`git add\`, then ONE commit "test(fixloop r${round}): apply ${approved.length} reviewed fixes" (§3.14). Then COMPILE-GATE via BASH (NOT zsh): for each crate run \`cargo test -p <crate> <features> --no-run\` and report GREEN/RED by ERROR-LINE presence (never the masked EXIT after a pipe). Crates+features: ${JSON.stringify(CFG.crates || [])}. Report commit SHA + per-crate green/red + any error text.\nAPPROVED: ${JSON.stringify(approved.map(a => a.file))}`,
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
const patternInduction = await agent(`${COMMON}\nPattern-induction meta-sweep over the decision log (${decisionLog.length} entries). Hunt UNNAMED cross-cutting patterns (>=3 recurrence => pim-N candidate); any class-of-bug the loop kept re-fixing (deeper root cause); §3.6h: if you propose a rule naming an origin instance, it must close that origin same-landing. Return a prose list.\nLOG: ${JSON.stringify(decisionLog.map(d => ({ round: d.round, kind: d.kind, n: (d.findings || d.approved || []).length })), null, 1)}`,
  { label: 'pattern-induction', phase: 'Terminal' })

return { mode: MODE, converged, rounds: round - 1, decisionLog, patternInduction, lastFixList }
