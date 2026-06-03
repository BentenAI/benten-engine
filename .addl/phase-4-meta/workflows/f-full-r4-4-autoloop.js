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
  specRef: 'phase-4-meta-core/f-full-r0-plan-r05',
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
  canon: `## F-FULL CANON — R4.4 AUTONOMOUS CONVERGENCE (spec = R0.5; R4.3 fixes ALREADY APPLIED)
- **R4.3 fixes are already in the corpus** (commit on f-full-r4-fix): X-Wing APPENDED + XWingLabel pin (F4-002/003), f_aad_2 sealed-inner-sender on 0x6600 (F4-001), MemberRef int-discriminant (F4-007), AAD-version reconciled to one-shared 0x01 (F4-004/005 both Layer-C + Layer-D), DeviceAuthBackend sealed/open=6 (F4-008), + MAJORs/doc-sweep. **VERIFY each is correctly applied; do NOT re-flag a correctly-applied fix.**
- **4 Ben-RULINGS (2026-06-03, now RESOLVED — do NOT re-surface as open):** (1) coarse_epoch is NOT on the Drop wire (§4.1/M-14); (2) Layer-D aad_version = the ONE shared 0x01 (distinct from wire-format version); (3) Compromise #64 IS assigned (cross-device best-effort-eventual nonce window; minted in SECURITY-POSTURE via the tracked-doc cascade — NOT a corpus concern); (4) bounded-decode = flagship arm now + a named R5-carry-row (the full family is R5-fill, NOT an R4 blocker). Treat these as SETTLED; only flag a NEW issue or a fix that was applied WRONG.
- Your job each round: HUNT RESIDUAL freeze-gating defects + regressions the R4.3 fixes may have introduced; FIX every non-disagreed finding (HARD RULE 12); CONVERGE when a full round = 0 new BLOCKER/MAJOR. R4.2/R4.3 history: \`git show phase-4-meta-core/orchestration-2026-05-26:.addl/phase-4-meta/r4-2-triage.md\`.
- **X-Wing combiner (R0.5-CORRECTED, verified vs draft-connolly-cfrg-xwing-kem-10 §6):** \`SHA3-256(ss_M || ss_X || ct_X || pk_X || XWingLabel)\` — label APPENDED (XWingLabel = the 6 bytes \`0x5c2e2f2f5e5c\`, ASCII \\.//^\\ ), NOT prepended. Corpus f_w0:67 still says prepended — FIX to appended + add an explicit 6-byte XWingLabel pin (F4-002/003).
- **Sealed-Sender reaches MembershipSet 0x6600 (F4-001):** f_aad_2 still length-prefixes plaintext sender_did into the default 0x6600 AAD golden — FIX: bind the sealed-inner-sender-DID (post-decrypt-verified, NOT plaintext) like the Layer-C 0x6520 fix; regenerate EXPECTED_AAD_HEX without inline sender-DID; convert the byte-bound arm to a no-plaintext-sender wire-scan; gate any plaintext-sender arm to the explicit non-default codepoint.
- **Enum representation (R0.5 F4-007):** ALL MemberEntry enums = INTEGER discriminant in canonical-CBOR/AAD — RoleId u8 ordinal AND MemberRef u8-tagged variant (NOT text). Fix golden: member_ref as int tag, symmetric with role.
- **AAD version-prefix byte (F4-004/005):** §4.1 freezes ONE \`aad_version:u8\` prefix DISTINCT from ENVELOPE_FORMAT_VERSION_V2. Layer-C wrongly uses ENVELOPE_FORMAT_VERSION(=2) as byte 0; membership uses AAD_VERSION(=0x01). Reconcile to membership's dedicated AAD_VERSION (fix Layer-C).
- **DeviceAuthBackend tier (F4-008):** §2.7 freezes OPEN seam roster at SIX (no DeviceAuthBackend) + places it SEALED (#7). Corpus wrongly includes it in TIER1_OPEN_SEAMS(len==7). Fix to open=6, DeviceAuthBackend sealed.
- **MINOR doc-sweep:** \`_nq_*_gated\` arms carry stale NQ-OPEN-SPEC doc-comments — NQ-T2/T3/T4/C5 are RATIFIED; update doc-comments to RATIFIED (arms stay). Bump stale \`R0.3 §\` cites to R0.5.
- Codepoints: Sealed-Sender 0x6510 DEFAULT; MembershipSet 0x6600/0x6610; Layer-C 0x6500/0x6510/0x6520; X-Wing 0x647a; vault 0x6100(24B)+0x6101(12B); sig LAMPS 0x0001. Golden-hex = M-20 (frozen vs stub; R5 confirms). Substantive-pin bar (pim-2): #[ignore]'d + self-contained shim + would-FAIL-on-no-op. Beware Rust-2024 reserved kw (gen).`,
}
const LENSES = [
  { key: 'ruling-fidelity', mandate: 'Walk every R0.5 ruling to its pin: X-Wing APPENDED (F4-002), Sealed-Sender on 0x6600 (F4-001), int-discriminant enums (F4-007), AAD-version byte (F4-004), DeviceAuthBackend sealed (F4-008). Flag any test contradicting R0.5.', focus: 'cross-cutting' },
  { key: 'crypto-construction', mandate: 'X-Wing real SHA3-256 APPENDED label + the 6-byte XWingLabel pin (F4-002/003); KAT/swap-matrix/Inv-17/HPKE; no prepend survives.', focus: 'f_w0, f_kat_*, f_sm' },
  { key: 'wire-freeze-byte-correctness', mandate: 'Freeze-gating goldens pin correct canonical bytes (V2/BE/AAD/TLV) vs R0.5; AAD-version-prefix byte reconciled (F4-004/005).', focus: 'f_w0, f_lb_2, f_va_1, f_aad_*, f_lc_*' },
  { key: 'sealed-sender-confidentiality', mandate: 'Adversary: NO group-send path (0x6520 OR 0x6600) freezes plaintext sender-DID in AAD (F4-001); Sealed-Sender holds across BOTH Layer-C + MembershipSet groups.', focus: 'f_lc_hpke, f_aad_2' },
  { key: 'encoding-serialization', mandate: 'Canonical DAG-CBOR/TLV: enum int-discriminant symmetry (F4-007), length-injectivity, aad_version pin, all-BE; member_ref int tag not text.', focus: 'f_aad_*, f_ms_3, f_nat_1' },
  { key: 'substantive-pin-falsifiability', mandate: 'Every fixed test would-FAIL-on-no-op (no tautology introduced by the fix); golden literals real frozen bytes (M-20).', focus: 'all touched files' },
  { key: 'invariant-semantics', mandate: 'Inv-16..22 faithfully encoded post-fix; negative arms fire; Inv-22 derive-nature preserved.', focus: 'f_inv*, f_nat_1' },
  { key: 'seam-tier-consistency', mandate: 'Open-vs-sealed roster matches R0.5 §2.7 (open=6, DeviceAuthBackend sealed, F4-008); no Tier-1/Tier-2 contradiction.', focus: 'f_freeze_1, f_crate_1_2' },
  { key: 'cross-wave-seam', mandate: 'Stub-shims AGREE across waves on AAD-version byte, TLV width, MemberEntry shape, EncryptedEnvelope layout; no divergent bytes for the same wire object.', focus: 'cross-cutting' },
  { key: 'membership-shape-fidelity', mandate: 'MemberEntry R0.5 §3.5 5-field + int-discriminant; 3 kinds; derive-nature; M-CONS-FINAL shape.', focus: 'f_ms_*, f_aad_*, f_nat_1' },
  { key: 'capability-rbac', mandate: 'Admin=Moderator u governance; RoleId ordinals; UCAN templates; sealed Policy; audit attribution.', focus: 'f_ms_6_7, f_audit_*' },
  { key: 'distributed-sync', mandate: 'CRDT/HLC/MST/gossip/Inv-21 totality/federation bound correct post-fix; no convergence tautology re-introduced.', focus: 'f_crdt, f_inv21, f_mst, f_gossip, f_hlc, f_fed' },
  { key: 'red-phase-r5-readiness', mandate: 'pim-12: every test stays ignored behind a RED-PHASE message + valid un-ignore destinations; stale NQ-OPEN-SPEC doc-comments updated to RATIFIED; stale R0.3 cites bumped; compiles green.', focus: 'all + f_ld_3/5/6/8' },
  { key: 'bounded-decode-threat', mandate: 'Bounded/length-checked decode pinned (no unbounded-decode DoS, META #629) on envelope/AAD decode paths; the bounded-decode gap R4.2 flagged.', focus: 'f_inv16_1, f_lc_*, f_ld_2' },
  { key: 'coverage-gap-audit', mandate: 'Every R4.2 finding has a fix; no NEW gap; the 2 freeze-gating completeness gaps (XWingLabel pin, bounded-decode) closed.', focus: 'r4-2-triage + corpus' },
]
const MAX_ROUNDS = CFG.MAX_ROUNDS || 4
const BUDGET_FLOOR = CFG.BUDGET_FLOOR || 120000
const MODE = CFG.mode || 'checkpoint'
const TMP = `/tmp/fixloop-${(CFG.corpusBranch || 'run').replace(/[^a-z0-9]/gi, '_')}`

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
