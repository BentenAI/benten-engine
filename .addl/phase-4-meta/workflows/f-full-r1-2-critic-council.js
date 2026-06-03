export const meta = {
  name: 'f-full-r1-2-critic-council',
  description: 'R1.2 convergence round: 8 lenses re-review the REVISED R0.2 (verify fixes + catch regressions) -> consolidated convergence read',
  phases: [
    { title: 'Council', detail: '8 lenses re-review the revised R0.2' },
    { title: 'Consolidate', detail: 'convergence read: did R1 converge (0 BLOCKER/MAJOR)?' },
  ],
}

const R0 = `the REVISED F-full R0.2 plan-doc (Phase-4-Meta-Core: encryption substrate + MembershipSet primitive). Read it via: git show 477529d0:.addl/phase-4-meta/f-full-r0-plan.md  (1547 lines, sections §0-§13; §13 is the R1-fix-pass changelog).`

const R12 = `THIS IS R1.2 — a CONVERGENCE re-review. The R0 was revised to address the R1.1 triage. Your R1.1 lens findings (and all lenses') are in your worktree at .addl/phase-4-meta/r1-triage.md; the revised R0's §13 changelog maps each finding -> disposition.

YOUR JOB: for your lens's scope, (1) VERIFY each R1.1 concern is ADEQUATELY closed in the revised R0 (don't just trust §13 — check the actual edited sections); (2) flag any R1.1 finding NOT adequately closed; (3) flag any NEW substantive issue the revision INTRODUCED (the doc grew ~50%). Be efficient — this is verification, not a fresh from-scratch review.

RATIFIED — do NOT re-open (these are Ben-ruled / blessed): Sealed-Sender = DEFAULT (FREEZE+SHIP-at-Core at 0x6510); Compromise #31 = LAMPS-keeps-it (revocation -> #62); X-Wing = real SHA3-256 construction at 0x647A; the blessed codepoint table (MembershipSet relocated to 0x6600; MLS keeps 0x6380/0x6390; CRYPTO-CODEPOINTS.md authored at R2); and Inv-15 IS registered in-tree (the R1.1 M-15 "Inv-15 not registered" finding was FALSE — ground-truth-confirmed: INVARIANT-COVERAGE.md table row 15 + header "15 invariants"). Do not raise these again.`

const OUTPUT = `Ground-truth-verify any R0 claim you doubt against HEAD 2172cb6d (git show / grep). HARD RULE 12 disposition on every finding.

Output as a markdown block (final message IS the deliverable; do NOT call any structured-output tool):
## LENS: <your lens>
**Verdict:** APPROVE-FOR-R2 (your scope is converged, 0 open BLOCKER/MAJOR) / APPROVE-WITH-FIXES / NEEDS-REVISION + confidence.
**R1.1 closure check:** for each R1.1 finding in your scope -> CLOSED / PARTIALLY-CLOSED / NOT-CLOSED + 1-line evidence.
**New findings (if any):** [ID] [SEVERITY] Title | R0.2 location | issue | recommendation | disposition.
**Convergence call:** is your lens's scope ready to advance to R2?`

phase('Council')

const LENSES = [
  { key: 'cryptographer', scope: `The 4 encryption layers, §6.2 codepoint-dispatched envelope, the X-Wing real-construction fix (BR-3), LAMPS, MLKEM768-X25519, AAD binding, side-channels. Verify M-4 (X-Wing LOC + real construction), M-5 (#30 added), M-6 (formal-methods/chosen-recipient-seed risk row), R1-Q-1/Q-3 closures.` },
  { key: 'p2p-systems', scope: `MembershipSet, federation/SubsetRef, HLC/CRDT/Inv-21 tie-break, sync, transport. Verify B-1 (benten-sync dep added), M-7/M-8 (Inv-21 asymmetry + totality via Version-Node-CID), M-9 (KSetAcquisitionPath frozen fields), M-10 (iroh-gossip convergence-vs-delivery analysis).` },
  { key: 'threat-model-security', scope: `Threat model, DAK/device-auth, remote-permission-call, the §6.7 mini-review gate, RBAC. Verify M-3 (ExecuteWorkflow reserve), M-11 (Invitee-zero-content + Moderator-strict-subset), M-12 (mini-review gate-owner+pass-classes), Compromise #63 (abuse-control), R1-Q-8/Q-9 closures.` },
  { key: 'privacy-metadata-leak', scope: `Sealed-Sender DEFAULT, iroh-gossip D6, DUAL-CID, and the COARSE-BUCKET / NONCE-CACHE fix. Verify M-1 (nonce-cache promoted to named v1-beta requirement + honest decoupling map + intra-hour-replay test), M-14 (DropToRecipient carries no timestamps), the #43 posture under Sealed-Sender-DEFAULT.` },
  { key: 'wire-format-freeze', scope: `The §4 frozen inventory + the blessed codepoint table, the frozen-crypto rule, additivity. Verify B-3 (one canonical table; MembershipSet relocated to 0x6600; Sealed-Sender 0x6510), M-15 (Inv-15 handling — note: Inv-15 IS registered, so verify ONLY Inv-16..22 are marked design-mints + the §5.0 baseline is correct), M-16 (D-28/29 marked NEW), M-17 (audit_log_query / frozen-crypto-rule 4th-limb), M-18/M-19/M-20 (one V2 migration + BE re-scope + Wave-0 DAG edge), m-12/m-13 (0x6400 + unified table).` },
  { key: 'graph-native-coherence', scope: `Everything-is-graph + GN wins, engine-plugin symmetry, the 15th-crate plan. Verify M-17 (audit-op vs frozen-crypto rule resolved), m-15 (the 5 GNC precision edits: tamper-evidence Option-enforcement, IVM-view-is-Rust-LOC, seam roster, AAD-opaque-bytes, MemberRef Kind-determined), R1-Q-5/Q-7 closures.` },
  { key: 'impl-feasibility-sequencing', model: 'sonnet', scope: `The §7 wave-decomposition + canary-first, crate plan, tactical picks, timeline, exit criteria. Verify the §7.3 DAG (B-1 benten-sync edge + M-20 Wave-0->canary edge), M-18/M-19 cost re-scopes, R1-Q-6 (timeline presentation inverted to honest-first), the §9 exit criteria + audit gate (NQ-A1/A2 carried).` },
  { key: 'standards-interop', model: 'sonnet', scope: `LAMPS/JOSE/HPKE/multicodec/IANA, identifier-as-content. Verify the codepoint table's IANA-disjointness assertion, M-4 (X-Wing interop-faithful at 0x647A), the carried NQ-C1 (McMillion-hpke KEM-extensibility) + NQ-C3 (cross-ecosystem conformance vectors) + NQ-C4 (did:key hybrid multicodec).` },
]

// model: cryptographer / p2p / threat / privacy / wire-format = Opus (omitted -> inherit); graph-native = Opus; impl + standards = Sonnet.
const council = (await parallel(LENSES.map(l => () => {
  const opts = { label: `lens:${l.key}`, phase: 'Council' }
  if (l.model) opts.model = l.model
  return agent(
    `You are a senior ${l.key} reviewer performing an R1.2 CONVERGENCE re-review of ${R0}\n\nYour LENS / scope: ${l.scope}\n\n${R12}\n\n${OUTPUT}`,
    opts
  )
})))
const got = council.filter(Boolean)

log(`R1.2 council complete: ${got.length}/${LENSES.length} lenses returned`)

phase('Consolidate')

const blocks = got.map((c, i) => `\n\n===== LENS ${i + 1} =====\n${c}`).join('')

const verdict = await agent(
  `You are consolidating an R1.2 CONVERGENCE re-review of the revised F-full R0.2.\n\nBelow are ${got.length} lens re-reviews. Produce a markdown convergence verdict:\n1. **CONVERGENCE CALL** — the single headline: does R1 CONVERGE (zero open BLOCKER/MAJOR across all lenses -> advance to R2) or NOT (needs R1.3)? State the deduplicated open BLOCKER/MAJOR/MINOR counts.\n2. **R1.1 closure scorecard** — of the original 3 BLOCKER + 20 MAJOR, how many are now CLOSED / PARTIAL / NOT-CLOSED per the lenses (cite which lens flags any non-closure).\n3. **New findings introduced by the revision** (if any) — by severity, with proposed HARD-RULE-12 dispositions.\n4. **If NOT converged:** the precise list of items the next R0 edit must fix (should be small).\n5. **If converged:** confirm R0.2 is R2-ready + list the named NQ-* questions carried forward to seed R2.\n\nBe honest; cite which lens said what. This final message IS the deliverable.\n\nLENS RE-REVIEWS:${blocks}`,
  { label: 'consolidate-r1-2', phase: 'Consolidate' }
)

return verdict