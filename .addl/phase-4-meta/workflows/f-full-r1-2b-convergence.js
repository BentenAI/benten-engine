export const meta = {
  name: 'f-full-r1-2b-convergence',
  description: 'R1.2 convergence (fresh, all-Opus, 2 sub-waves of 4 to dodge burst rate-limit): re-review revised R0.2',
  phases: [
    { title: 'Council-A', detail: 'crypto / p2p / threat / privacy' },
    { title: 'Council-B', detail: 'wire / graph-native / impl / standards' },
    { title: 'Consolidate', detail: 'convergence read across all 8' },
  ],
}

const R0 = `the REVISED F-full R0.2 plan-doc. Read it via: git show 477529d0:.addl/phase-4-meta/f-full-r0-plan.md  (1547 lines, §0-§13; §13 is the R1-fix-pass changelog).`

const R12 = `THIS IS R1.2 — a CONVERGENCE re-review. The R0 was revised to address the R1.1 triage (your worktree: .addl/phase-4-meta/r1-triage.md; §13 of R0.2 maps each finding -> disposition).
YOUR JOB for your scope: (1) VERIFY each R1.1 concern is ADEQUATELY closed in the revised R0 (check the actual edited sections, don't just trust §13); (2) flag any R1.1 finding NOT adequately closed; (3) flag any NEW substantive issue the revision INTRODUCED (the doc grew ~50%). Be efficient — verification, not from-scratch.
RATIFIED — do NOT re-open: Sealed-Sender=DEFAULT (FREEZE+SHIP @0x6510); Compromise #31=LAMPS (revocation->#62); X-Wing=real SHA3-256 @0x647A; the blessed codepoint table (MembershipSet->0x6600; MLS keeps 0x6380/0x6390; CRYPTO-CODEPOINTS.md authored at R2); Inv-15 IS registered in-tree (the R1.1 M-15 finding was FALSE — INVARIANT-COVERAGE.md row 15 + header "15 invariants"). Already-noted doc-only items (do not re-raise as new): NC-SI-1 (the §4.0 0x647A IANA-disjointness prose self-contradiction, folding into CRYPTO-CODEPOINTS.md).`

const OUTPUT = `Ground-truth-verify any R0 claim you doubt against HEAD 2172cb6d. HARD RULE 12 on every finding.
Output as markdown (final message IS the deliverable; NO structured-output tool):
## LENS: <your lens>
**Verdict:** APPROVE-FOR-R2 / APPROVE-WITH-FIXES / NEEDS-REVISION + confidence.
**R1.1 closure check:** each R1.1 finding in your scope -> CLOSED / PARTIAL / NOT-CLOSED + 1-line evidence.
**New findings (if any):** [ID][SEV] Title | R0.2 location | issue | recommendation | disposition.
**Convergence call:** is your scope ready for R2?`

const LENSES = [
  { key: 'cryptographer', scope: `The 4 encryption layers, §6.2 envelope, X-Wing real-construction (BR-3), LAMPS, MLKEM768-X25519, AAD, side-channels. Verify M-4 (X-Wing real SHA3-256 + LOC), M-5 (#30 added), M-6 (chosen-recipient-seed/formal-methods risk row), R1-Q-1/Q-3.` },
  { key: 'p2p-systems', scope: `MembershipSet, federation/SubsetRef, HLC/CRDT/Inv-21 tie-break, sync, transport, AND verify B-2 if it falls in your scope. Verify B-1 (benten-sync dep), M-7/M-8 (Inv-21 asymmetry + totality via Version-Node-CID), M-9 (KSetAcquisitionPath frozen fields), M-10 (gossip convergence-vs-delivery).` },
  { key: 'threat-model-security', scope: `Threat model, DAK/device-auth, remote-permission-call, §6.7 mini-review gate, RBAC. Verify M-3 (ExecuteWorkflow reserve), M-11 (Invitee-zero-content + Moderator-strict-subset), M-12 (mini-review owner+pass-classes), Compromise #63 (abuse-control), R1-Q-8/Q-9.` },
  { key: 'privacy-metadata-leak', scope: `Sealed-Sender DEFAULT, iroh-gossip D6, DUAL-CID, the COARSE-BUCKET/NONCE-CACHE fix. Verify M-1 (nonce-cache promoted to named v1-beta requirement + honest decoupling map + intra-hour-replay test), M-14 (DropToRecipient no timestamps), the #43 posture under Sealed-Sender-DEFAULT.` },
  { key: 'wire-format-freeze', scope: `The §4 frozen inventory + blessed codepoint table, the frozen-crypto rule, additivity, AND verify B-2 (#31 renumbering plan correctly re-derived: LAMPS gets #31 row, revocation->#62, #30 stays) if in scope. Verify M-16 (D-28/29 NEW), M-17 (audit_log_query / frozen-crypto-rule 4th-limb), M-18/M-19/M-20 (one V2 migration + BE re-scope + Wave-0 DAG edge), m-12 (0x6400). NOTE Inv-15 IS registered (M-15 closed correctly).` },
  { key: 'graph-native-coherence', scope: `Everything-is-graph + GN wins, engine-plugin symmetry, 15th-crate plan. Verify M-17 (audit-op vs frozen-crypto rule resolved), m-15 (5 GNC precision edits: tamper-evidence enforcement, IVM-view-is-Rust-LOC, seam roster, AAD-opaque-bytes, MemberRef Kind-determined), R1-Q-5/Q-7.` },
  { key: 'impl-feasibility-sequencing', scope: `§7 wave-decomposition + canary-first, crate plan, tactical picks, timeline, exit criteria. Verify the §7.3 DAG (B-1 + M-20 edges), M-18/M-19 cost re-scopes, R1-Q-6 (honest-first timeline), §9 exit criteria + audit gate.` },
  { key: 'standards-interop', scope: `LAMPS/JOSE/HPKE/multicodec/IANA, identifier-as-content. Verify the codepoint table IANA-disjointness (note NC-SI-1 already-flagged for 0x647A), M-4 (X-Wing interop-faithful), carried NQ-C1/C3/C4.` },
]

const runLens = (l) => () => agent(
  `You are a senior ${l.key} reviewer performing an R1.2 CONVERGENCE re-review of ${R0}\n\nYour LENS / scope: ${l.scope}\n\n${R12}\n\n${OUTPUT}`,
  { label: `lens:${l.key}`, phase: LENSES.indexOf(l) < 4 ? 'Council-A' : 'Council-B' }
)

phase('Council-A')
const waveA = (await parallel(LENSES.slice(0, 4).map(runLens))).filter(Boolean)
log(`Wave A: ${waveA.length}/4 returned`)

phase('Council-B')
const waveB = (await parallel(LENSES.slice(4, 8).map(runLens))).filter(Boolean)
log(`Wave B: ${waveB.length}/4 returned`)

const got = [...waveA, ...waveB]

phase('Consolidate')
const blocks = got.map((c, i) => `\n\n===== LENS ${i + 1} =====\n${c}`).join('')
const verdict = await agent(
  `You are consolidating an R1.2 CONVERGENCE re-review of the revised F-full R0.2 (all 8 lenses). ${got.length} lenses returned — if fewer than 8, DO NOT certify convergence (rate-limit silence is not APPROVE).\n\nProduce a markdown convergence verdict:\n1. **CONVERGENCE CALL** — does R1 CONVERGE (zero open BLOCKER/MAJOR across ALL 8 lenses -> advance to R2) or NOT (needs R1.3)? Deduplicated open BLOCKER/MAJOR/MINOR counts.\n2. **R1.1 closure scorecard** — of the original 3 BLOCKER + 20 MAJOR, how many CLOSED / PARTIAL / NOT-CLOSED per the lenses.\n3. **New findings introduced** (if any) — severity + HARD-RULE-12 dispositions.\n4. **If NOT converged:** the precise (small) list the next R0 edit must fix.\n5. **If converged:** confirm R2-ready + list the named NQ-* questions carried to seed R2.\n\nHonest; cite which lens. This final message IS the deliverable.\n\nLENS RE-REVIEWS:${blocks}`,
  { label: 'consolidate-r1-2b', phase: 'Consolidate' }
)
return verdict