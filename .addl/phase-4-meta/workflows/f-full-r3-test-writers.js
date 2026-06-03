export const meta = {
  name: 'f-full-r3-test-writers',
  description: 'R3 red-phase test-writers — single sequential workflow: canary W0 + gate -> fan-out W1-W7 -> per-wave mini-reviews -> coverage-verify',
  phases: [
    { title: 'Canary', detail: 'R3-W0 crypto-suite canary + mini-review GATE' },
    { title: 'Fanout-A', detail: 'W1 crypto-KAT / W2 Layer-C / W3 Layer-D' },
    { title: 'Fanout-B', detail: 'W4 MS-structure / W5 AAD+Inv-21 / W6 governance+audit' },
    { title: 'DocWave', detail: 'W7 doc-coupling (cap-exempt)' },
    { title: 'Reviews', detail: 'per-wave substantive-pin mini-reviews' },
    { title: 'Coverage', detail: 'full-corpus coverage + seam-disjointness verification' },
  ],
}

const SPEC = `Read your inputs first: the canonical R0.3 design via \`git show 4fe9236a:.addl/phase-4-meta/f-full-r0-plan.md\`, and the R2 test-landscape (your assigned F-IDs + their exact pin specs in §1, the coverage matrix §2, freeze-gating §4) at \`.addl/phase-4-meta/f-full-r2-test-landscape.md\` (in your worktree).`

const DISCIPLINE = `Write RED-PHASE test stubs (pim-12 §3.6e): each test compiles GREEN at baseline behind \`#[ignore = "RED-PHASE: <F-ID> — <what it pins>; un-ignore at R5"]\`, using a SELF-CONTAINED stub-shim (do NOT depend on another wave's module — each wave is independent for parallel safety; R5 replaces stubs with the real types). Every test body MUST drive a PRODUCTION call site + assert an OBSERVABLE consequence + be would-FAIL-if-no-op'd (pim-2 §3.6b + pim-18 §3.6f + §3.6f-ext) — NEVER \`assert_eq!(CONST, CONST_VAL)\`, NEVER a zero-assertion arm. Byte-pinning families author V2 + BE + \`EncryptedEnvelope\` from the first commit (M-20) — no LE/V1 bytes. Scoped pre-flight ONLY (scoped per-package \`cargo check\` that your stubs compile green; NO workspace cargo). Base = main's crate code (your worktree); the plan is pinned to 2172cb6d and the orchestrator rebases onto the canary's landing SHA at consolidation. NEVER absolute paths outside your worktree root; NO AI attribution.`

const OUT = (branch) => `Branch \`${branch}\`; write your test files to the named crate's \`tests/\` dir; commit before returning (\`git add\` your files \`&& git commit\`). Final message: branch + commit SHA + the F-IDs you covered + a 1-line-per-family summary (file::test_name -> what it pins) + any open-spec arm you flagged (referencing the gating NQ).`

const TW = (w) => `You are a senior Rust test-writer (rust-implementation-developer lens) for R3 wave **${w.key}**.\n\n${SPEC}\n\nYOUR FAMILIES: ${w.fids}\nYOUR CRATE/FILES: ${w.crate}\nFOCUS: ${w.focus}\n\n${DISCIPLINE}\n\n${OUT(w.branch)}`

// ---- WAVE 0 (canary) ----
phase('Canary')
const W0 = { key: 'W0-crypto-canary', branch: 'r3/w0-crypto-canary', crate: 'crates/benten-crypto-suite/tests/ (+ reference src/{aead,cipher_suite,codepoint}.rs)', fids: 'F-W0-1..5, F-CP-1..7, F-SM-1..3, F-KAT-3, F-INV16-1', focus: 'The SOLE-UPSTREAM canary: mints the EncryptedEnvelope + typed BindingContext + the §4.0 codepoint integer pins + V2/BE migration + the real X-Wing SHA3-256 construction (BR-3) + dispatch/typed-reject/swap-matrix. AUTHOR F-KAT-3 FIRST: investigate (read crate source + the McMillion hpke crate docs) whether `hpke` admits a custom/PQ KEM into a real RFC-9180 context (NQ-C1, freeze-gating) — pin the byte-format question.' }
const w0 = await agent(TW(W0), { label: 'w0:crypto-canary', phase: 'Canary', isolation: 'worktree' })
const w0review = await agent(
  `You are an R3 canary mini-reviewer. The W0 test-writer committed red-phase tests (its result below; \`git show\` its branch to read the actual files). VERIFY: (1) every pin is SUBSTANTIVE — drives a production call site + observable consequence + would-FAIL-if-no-op'd (pim-2/18/§3.6f-ext), NOT assert_eq!(CONST,CONST) or zero-assertion; (2) proper RED-PHASE staged-pin discipline (pim-12) — compiles green + #[ignore] with a clear un-ignore destination; (3) byte families authored V2/BE/EncryptedEnvelope, NOT LE/V1; (4) the assigned F-IDs are all present. Be strict — this canary gates the whole fan-out.\n\nEnd your review with EXACTLY ONE of these two lines: \`GATE: PASS\` (clean enough for fan-out) or \`GATE: FIX-NEEDED\` (substantive issues — list them).\n\nW0 RESULT:\n${w0}`,
  { label: 'w0:mini-review', phase: 'Canary' }
)

if (/GATE:\s*FIX-NEEDED/i.test(w0review) && !/GATE:\s*PASS/i.test(w0review)) {
  log('Canary GATE: FIX-NEEDED — halting before the fan-out for an orchestrator-led canary fix-pass.')
  return { stage: 'canary-gate-blocked', w0, w0review }
}
log('Canary GATE: PASS — proceeding to the fan-out.')

// ---- FAN-OUT (after the canary gate) ----
const WAVES = [
  { key: 'W1-crypto-kat', branch: 'r3/w1-crypto-kat', wave: 'A', crate: 'crates/benten-crypto-suite/tests/ (KAT slice — disjoint test files from W0)', fids: 'F-KAT-1/2/4, F-VA-1..5, F-LB-1..3', focus: 'Vault + Layer-A/B + cross-impl KATs. External-vector families (F-KAT-1/4 FIPS-203 + LAMPS) — use a deterministic synthesized witness (per the tf4 load_fips_204_kat_vector_for_test precedent) if real vectors are unavailable at write-time; flag the real-corpus swap for R5.' },
  { key: 'W2-layer-c', branch: 'r3/w2-layer-c', wave: 'A', crate: 'crates/benten-drop/tests/ (+ benten-sync/src/two_cid_store.rs + benten-graph/src/two_cid_map.rs test homes)', fids: 'F-LC-1..9, F-INV18-1', focus: 'Layer-C HPKE encrypt-to-recipient + Sealed-Sender + DUAL-CID + abuse-control #63. **F-LC-9 (Ben-ruled): GROUP sends HONOR Sealed-Sender** — pin a per-stanza inner-sender-DID binding inside the group AAD so the sender-DID is NOT plaintext on group multi-stanza sends (0x6520/0x6610).' },
  { key: 'W3-layer-d', branch: 'r3/w3-layer-d', wave: 'A', crate: 'crates/benten-engine/tests/ (+ benten-sync/tests/ handshake+nonce-cache)', fids: 'F-LD-1..8, F-VA-3', focus: 'Layer-D DAK/device-auth/remote-permission/multi-device-wrap + the 6-class §6.7 mini-review harness + the nonce-cache (NET-NEW jti-keyed instance per the R0.3 §3.10 precision) intra-hour-replay-REJECTED. NQ-resolution-gated arms (F-LD-3 NQ-T3, F-LD-5 NQ-T4, F-LD-6 NQ-T2, F-LD-8 NQ-C5): author a red-phase stub referencing the open question if unresolved.' },
  { key: 'W4-ms-structure', branch: 'r3/w4-ms-structure', wave: 'B', crate: 'crates/benten-membership-set/ (NEW crate — scaffold a minimal Cargo.toml + src/lib.rs stub + tests/)', fids: 'F-MS-1..9, F-FED-1/2, F-CRATE-1/2', focus: 'The MembershipSet primitive (EXACTLY-3 Kind enum, members_table, 5-role RoleId with Invitee=0/Moderator-strict-subset), federation SubsetRef, the 15th-crate boundary. Scaffold the new crate minimally so your tests compile green; other MS waves add minimal stubs too (consolidation reconciles).' },
  { key: 'W5-ms-aad-inv21', branch: 'r3/w5-ms-aad-inv21', wave: 'B', crate: 'crates/benten-membership-set/tests/ (+ reuse benten-sync/tests/ harnesses; ensure a minimal crate scaffold exists for compile)', fids: 'F-AAD-1/2, F-HLC-1/2, F-INV21-1..4, F-CRDT-1..3, F-MST-1..3, F-GOSSIP-1/2', focus: 'AAD 9-tuple canonical-CBOR length-injectivity (F-AAD-1 members_table bytes — the highest untested-byte risk) + HLC + Inv-21 TOTALITY via Version-Node-CID (F-INV21-3 stands up kani infra — proptest is the v1-beta floor, kani arms #[ignore] v1-GM; do NOT block on kani standup) + CRDT/MST/gossip convergence (gossip=liveness, MST=convergence backstop).' },
  { key: 'W6-gov-audit', branch: 'r3/w6-gov-audit', wave: 'B', crate: 'crates/benten-membership-set/tests/ + crates/benten-engine/tests/ (enforced-write path)', fids: 'F-AUDIT-1..4, F-GOV-1, F-NAT-1/2, F-INV19-1', focus: 'Governance-as-signed-config-Node, audit-as-version-chain (enforced-attribution-path, not bare put_node), member-nature-derived-never-stored (Inv-22 boundary; did:agent: + UCAN root-issuers), Inv-19.' },
]

phase('Fanout-A')
const fa = (await parallel(WAVES.filter(w => w.wave === 'A').map(w => () => agent(TW(w), { label: `tw:${w.key}`, phase: 'Fanout-A', isolation: 'worktree' })))).filter(Boolean)
log(`Fanout-A: ${fa.length}/3`)

phase('Fanout-B')
const fb = (await parallel(WAVES.filter(w => w.wave === 'B').map(w => () => agent(TW(w), { label: `tw:${w.key}`, phase: 'Fanout-B', isolation: 'worktree' })))).filter(Boolean)
log(`Fanout-B: ${fb.length}/3`)

phase('DocWave')
const W7 = { key: 'W7-doc-wave', branch: 'r3/w7-doc-wave', crate: 'doc-coupling tests (grep/registration; reuse the tf3f_revocation_reach_forever_valid_documented shape)', fids: 'F-DISC-1/2, F-FREEZE-1, F-NQA1-1, F-NQC4-1, F-TRANS-1', focus: 'The registration + disclosure-coherence catch-net (pim-13). F-DISC-1 parametrizes over the FULL docs/SECURITY-POSTURE.md row-set (NOT a hand-list, so new Compromise rows auto-include) — pins disclosure-coherence + disposition_class. F-FREEZE-1 = additive-extensibility + freeze-tally; F-NQC4-1 = did:key hybrid-pubkey multicodec; F-NQA1-1 = RecoveryHook-absent-at-Core boundary.' }
const w7 = await agent(TW(W7), { label: 'tw:W7-doc-wave', phase: 'DocWave', isolation: 'worktree' })

const waves = [...fa, ...fb, w7].filter(Boolean)

// ---- PER-WAVE MINI-REVIEWS ----
phase('Reviews')
const reviews = (await parallel(waves.map((wv, i) => () => agent(
  `You are an R3 per-wave mini-reviewer. A test-writer committed red-phase tests (result below; \`git show\` its branch to read the files). VERIFY: substantive pins (production call site + observable + would-FAIL-if-no-op'd; pim-2/18/§3.6f-ext), proper RED-PHASE discipline (pim-12), byte families on V2/BE, and SEAM-DISJOINTNESS (this wave does NOT re-implement a family the R2 single-ownership assigned to another wave — check the R2 landscape §1 [SEAM] notes at .addl/phase-4-meta/f-full-r2-test-landscape.md). Report findings + the F-IDs verified-present-and-substantive vs missing/weak. Verdict: APPROVE / FIX-NEEDED.\n\nWAVE RESULT:\n${wv}`,
  { label: `review:${i}`, phase: 'Reviews' }
)))).filter(Boolean)

// ---- COVERAGE VERIFICATION ----
phase('Coverage')
const coverage = await agent(
  `You are the R3 COVERAGE consolidator. The canary (W0) + 7 fan-out waves wrote the red-phase corpus; their summaries + the per-wave mini-reviews are below. Read the R2 landscape (\`.addl/phase-4-meta/f-full-r2-test-landscape.md\`) §1 (all ~95 F-IDs), §2 (coverage matrix), §4 (the ~74 freeze-gating families). VERIFY: (1) is EVERY F-ID pinned by some wave? (2) is EVERY freeze-gating family present? (3) every Inv-16..22 / Compromise #30-#63 / frozen-codepoint / NQ / §9 exit-criterion covered? (4) any family double-implemented across waves (seam violation)? (5) any wave flagged FIX-NEEDED by its mini-review? Produce: a coverage scorecard (covered / missing / weak / double-implemented), the precise gap-list for an R3.2 patch (should be small), and a CONVERGENCE call (is the red-phase corpus complete + clean for R4, or does R3.2 need a patch pass?). Final message IS the deliverable.\n\nW0:\n${w0}\n\nFAN-OUT WAVES:\n${waves.map((w,i)=>`--- wave ${i} ---\n${w}`).join('\n')}\n\nPER-WAVE MINI-REVIEWS:\n${reviews.map((r,i)=>`--- review ${i} ---\n${r}`).join('\n')}`,
  { label: 'coverage-verify', phase: 'Coverage' }
)

return { w0, w0review, waves, reviews, coverage }