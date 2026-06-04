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
  specRef: 'phase-4-meta-core/f-full-r0-plan-r07',
  specPath: '.addl/phase-4-meta/f-full-r0-plan.md',
  orchRef: 'phase-4-meta-core/orchestration-2026-05-26',
  MAX_ROUNDS: 4,
  BUDGET_FLOOR: 150000,
  mode: 'autonomous',
  integWorktree: '/Users/benwork/Documents/benten-wt-fixint',
  crates: [
    { name: 'benten-crypto-suite', features: '--features testing' },
    { name: 'benten-membership-set', features: '--features testing' },
    { name: 'benten-drop', features: '--features testing' },
    { name: 'benten-engine', features: '--features test-helpers --features benten-eval/testing' },
  ],
  canon: `## F-FULL CANON — R4.6 FULL-PHASE RE-VERIFICATION + SS-AAD COMPLETION (spec = R0.7)
- Spec of record is now R0.7 (branch phase-4-meta-core/f-full-r0-plan-r07). The corpus (f-full-r4-fix @ f9bed98b) already has the R4.5b 0x6610 blinding migration (orchestrator hand-verified BYTE-EXACT). YOUR JOB each round: RE-VERIFY THE WHOLE CORPUS FRESH against R0.7 + complete the SS-AAD freeze + CONVERGE (0 new BLOCKER/MAJOR on the WHOLE artifact).
- FULL-PHASE RE-VERIFICATION (do this EVERY round, over the WHOLE corpus — NOT just recent changes): re-derive / re-check EVERY freeze byte, golden-hex, codepoint, and construction across the ENTIRE corpus against R0.7 FRESH — INCLUDING in files a prior round already "fixed". Do NOT assume any prior fix is correct; do NOT skip "settled" territory. (A prior round froze codepoint 0x6600 where R0.7 §3.10/§4.1 mandate 0x6610 — a slip in "settled" territory the loop must catch THIS round.)
- THE 2 KNOWN CORRECTIONS R0.7 mandates (re-verify the WHOLE corpus, then fix):
  1. CODEPOINT (benten-membership-set f_aad_2 0x6610 group AAD): the golden froze codepoint 0x6600 (MEMBERSHIP_SET_ENCRYPTION); R0.7 §3.10/§4.1 mandate 0x6610 (MEMBERSHIP_SET_GROUP_MULTI_STANZA). FIX the fixture codepoint constant (the default fixture + the non-default plaintext-sender control) 0x6600 -> 0x6610 + regenerate the golden (ONLY the 2 codepoint bytes change; the commitments/body_cid/counts are codepoint-INDEPENDENT and STAY byte-identical) + fix the "DEFAULT 0x6600" doc-comments. (The 0x6610 commitments are already byte-exact correct per orchestrator hand-verify — do NOT touch them.)
  2. 0x6520 BLINDING (NEW, R0.7): the corpus 0x6520 (LAYER_C_DROP_MULTI_RECIPIENT / HpkeMultiBase) per-stanza AAD (benten-drop f_lc_hpke F-LC-2 assembler HpkeRecipientStanza::plaintext_aad_bytes + golden F_LC_2_GROUP_STANZA_AAD_HEX) is still the PRE-BLINDING raw shape (raw recipient-DID list, bare-32 body_cid, no stanza_count; its comment says "0x6520 NOT re-opened by R0.6" — R0.7 RE-OPENS it). READ the blinded 8-field set from R0.7 §3.3/§4.1 and migrate the assembler + golden to it: { aad_version(0x01,u8), codepoint(0x6520,u16 BE), body_cid(self-describing CIDv1 36B), recipient_count(u16 BE), audience_set_commitment(32B), stanza_index(u32 BE), stanza_count(u32 BE), recipient_key_generation(u32 BE) }. Blind the raw recipient list -> audience_set_commitment = BLAKE3(0x01||lp(did_i)...) over sorted DIDs (lp=u32-BE — IDENTICAL construction to 0x6610); ADD stanza_count; body_cid bare-32 -> self-describing CIDv1. NO membership_set_id/gen/role (NOT a MembershipSet). Regenerate the golden via M-20 throwaway-compute.
- DESIGN-COHERENCE / INVARIANT-RIPPLE (the R1-equivalent check, this round): verify the 0x6520 + 0x6610 blinding does NOT break any invariant statement (Inv-16..22, U17 cross-stanza substitution, Inv-18 paired-disclosure, Inv-20 clause-c) and coheres with R0.7; the blinding MUST preserve all binding properties (recipients hold K_Set + the recipient list -> recompute + verify the commitment; relay sees only opaque 32-byte tags).
- COMMITMENT CONSTRUCTION (verify reuse): audience_set_commitment = blake3::hash(0x01 || u32-BE-lp(sorted DIDs)); set-id/topic commitment = blake3::keyed_hash(K_Set, label || id) — R0.7 clarifies "HMAC" = blake3::keyed_hash (do NOT invent HMAC-SHA256). Reuse the corpus §3.9 gossip primitive.
- OFF-LIMITS to re-DEBATE (ratified DESIGN decisions — but their IMPLEMENTATION bytes are STILL re-verified every round): coarse_epoch NOT on the Drop wire; u16/u32 per-band widths NOT unified (Layer-C recipient_count u16 vs MembershipSet member_count u32 — separately-frozen bands); identity-hiding-not-unlinkability (full unlinkability = U25 v1-GM, do NOT add salt). Everything ELSE (every byte) IS re-verifiable this round.
- Substantive-pin bar (pim-2): every migrated/verified test stays ignored behind #[ignore] + self-contained stub-shim + asserts the frozen bytes + would-FAIL-on-no-op. Golden-hex = M-20 (frozen vs stub; R5 confirms vs real encoder). Beware Rust-2024 reserved keyword gen.
- Codepoints: Sealed-Sender 0x6510 DEFAULT; Layer-C 0x6500/0x6510/0x6520; MembershipSet 0x6600(set-keying)/0x6610(group multi-stanza); X-Wing 0x647a; vault 0x6100(24B)/0x6101(12B); sig LAMPS 0x0001.`,
}
const LENSES = [
  { key: 'freeze-byte-reverification-FULL-PHASE', mandate: 'RE-VERIFY every freeze byte / golden-hex / codepoint / construction across the WHOLE corpus FRESH against R0.7 — re-derive each golden, re-check each codepoint, INCLUDING in files a prior round already fixed. Do NOT assume prior fixes are correct; do NOT skip "settled" territory. Flag ANY divergence from R0.7 (e.g. f_aad_2 codepoint 0x6600 vs the R0.7-mandated 0x6610).', focus: 'WHOLE corpus' },
  { key: 'ss-aad-completion-0x6520-codepoint', mandate: 'The 2 R0.7 corrections: (1) f_aad_2 0x6610 codepoint 0x6600 -> 0x6610 (fixture + non-default control + golden + doc-comments; the commitments STAY); (2) f_lc_hpke 0x6520 (F-LC-2) migrated to the R0.7 blinded 8-field set (audience_set_commitment replaces the raw recipient list + stanza_count + CIDv1 body_cid; NO set-id/gen/role; recipient_count u16 BE after body_cid). Flag any 0x6520 still raw or codepoint still 0x6600.', focus: 'f_aad_2, f_lc_hpke' },
  { key: 'commitment-construction-correctness', mandate: 'audience_set_commitment = blake3::hash(0x01 || u32-BE-lp(sorted DIDs)) — IDENTICAL for 0x6610 + 0x6520; set-id commitment = blake3::keyed_hash(K_Set,"benten:setid:v1"||id) reusing the §3.9 gossip primitive (R0.7: HMAC = blake3::keyed_hash, NOT HMAC-SHA256). Re-verify the construction + lp width + truncation.', focus: 'f_aad_2, f_lc_hpke, the §3.9 gossip test' },
  { key: 'golden-hex-byte-correctness', mandate: 'Every golden (EXPECTED_AAD_HEX, F_INV18_1_SEALED_AAD_HEX, F_LC_2_GROUP_STANZA_AAD_HEX, etc.) is a real frozen literal computed from the R0.7 construction (M-20 throwaway-compute), byte-length correct (commitments 32B, CIDv1 36B), codepoint correct, would-FAIL-on-revert. Re-derive each independently. No self-referential / tautological hex.', focus: 'all goldens' },
  { key: 'design-coherence-invariant-ripple', mandate: 'R1-equivalent: does the 0x6520 + 0x6610 blinding BREAK any invariant statement (Inv-16..22, U17 cross-stanza substitution, Inv-18 paired-disclosure, Inv-20 clause-c) or cohere-fail with R0.7? The blinding must PRESERVE all bindings (recipients recompute+verify the commitment; relay sees opaque tags). Flag any invariant-ripple or coherence break.', focus: 'f_inv*, f_aad_2, f_lc_hpke, R0.7' },
  { key: 'cross-file-seam-consistency', mandate: 'f_aad_2 / f_lc_hpke / f_lc_abuse / f_inv16_1 AGREE on body_cid CIDv1 framing + the audience_set_commitment construction + codepoints; no divergent bytes for the same wire object; the 0x6520 + 0x6610 group AADs are structurally parallel (recipient_count/member_count after body_cid, then the commitment).', focus: 'cross-cutting' },
  { key: 'wire-freeze-byte-correctness', mandate: 'All OTHER frozen bytes UNREGRESSED vs R4.1-R4.5 (X-Wing APPEND + XWING_LABEL pin, MemberRef/RoleId int-discriminant, AAD_VERSION 0x01, DeviceAuthBackend sealed open=6, the 0x6510 audience u32-BE, token-binding no coarse_epoch). Re-verify each; the corrections must not disturb them.', focus: 'f_w0, f_kat_*, f_sm, f_freeze_1, f_lc_abuse' },
  { key: 'substantive-pin-falsifiability', mandate: 'pim-2: every migrated/verified test stays ignored behind #[ignore] + self-contained stub-shim + asserts the frozen bytes + would-FAIL-on-no-op; no tautology introduced.', focus: 'all touched files' },
  { key: 'invariant-semantics', mandate: 'Inv-16..22 + Inv-18 paired-disclosure + Inv-20 clause-c faithfully encoded post-correction; negative arms fire; the blinding preserves cross-stanza substitution (U17) + inter-member non-forgeability.', focus: 'f_inv*, f_aad_2, f_lc_hpke' },
  { key: 'ruling-fidelity', mandate: 'Every R0.7 freeze decision maps to a faithful pin; do NOT relitigate the SETTLED DESIGN rulings (coarse_epoch-not-on-wire; u16/u32 widths NOT unified; identity-hiding-not-unlinkability) — but DO re-verify their implementation bytes. Flag any test contradicting R0.7.', focus: 'cross-cutting' },
  { key: 'threat-model-bounded-decode', mandate: 'The blinding does not weaken any binding; bounded/length-checked decode still pinned (META #629); no new unbounded-decode path; the 32B commitments + CIDv1 + the u16/u32 length fields are length-bounded.', focus: 'f_inv16_1, f_lc_*, f_aad_2' },
  { key: 'red-phase-r5-readiness', mandate: 'pim-12: every test stays ignored behind a RED-PHASE message + valid un-ignore destinations; doc-comments cite R0.7 (not R0.5/R0.6/R0.3 where stale); compiles green behind #[ignore].', focus: 'all + f_aad_2 + f_lc_hpke' },
  { key: 'coverage-gap-audit', mandate: 'The SS-AAD freeze is COMPLETE: no file left on an old raw-roster / bare-32 / wrong-codepoint shape; no R0.7 obligation unpinned. Grep that raw sorted_member_dids / raw sorted_recipient_dids / raw membership_set_id are GONE from AAD assembly (survive only inside commitment inputs); codepoint 0x6600 absent from the group-send AAD.', focus: 'R0.7 + WHOLE corpus' },
  { key: 'distributed-sync', mandate: 'The §3.9 gossip-topic construction reuse is consistent (set-id commitment == the topic blinding fn applied to the set-id); CRDT/HLC/MST/Inv-21 unregressed.', focus: 'f_gossip, f_crdt, f_inv21, f_aad_2' },
]
const MAX_ROUNDS = CFG.MAX_ROUNDS || 4
const BUDGET_FLOOR = CFG.BUDGET_FLOOR || 120000
const MODE = CFG.mode || 'checkpoint'
const TMP = `/tmp/fixloop-r46-${(CFG.corpusBranch || 'run').replace(/[^a-z0-9]/gi, '_')}`

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
