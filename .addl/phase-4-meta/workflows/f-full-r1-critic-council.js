export const meta = {
  name: 'f-full-r1-critic-council',
  description: 'R1 critic council on the F-full R0 plan-doc: 8 parallel lenses -> consolidated triage surface (human-in-the-loop triage)',
  phases: [
    { title: 'Council', detail: '8 parallel critic lenses review the R0' },
    { title: 'Consolidate', detail: 'dedup + per-question resolutions + triage surface' },
  ],
}

const R0 = `the F-full R0 implementation-plan doc (Phase-4-Meta-Core: encryption substrate + MembershipSet primitive). Read it via: git show 6755ea41:.addl/phase-4-meta/f-full-r0-plan.md  (1055 lines, sections §0-§12).`

const BACKGROUND = `Background docs (read via git show only as needed for your lens): M-CONS-FINAL (git show a99dd0c7:.addl/phase-4-meta/membership-set-m-cons-final.md); GN-1 graph-native (859fa51e:.addl/phase-4-meta/membership-set-gn1-graph-native-composition.md); GN-2 trajectory (61f24553:.addl/phase-4-meta/membership-set-gn2-everything-is-graph-trajectory.md); EP-1 engine-plugins (7520ae4e:.addl/phase-4-meta/ep1-rust-engine-plugins-formalization.md); the 9-eyes encryption registry (fbdfeb16:.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md); e2r-ffull (220b5aae:.addl/phase-4-meta/e2r-ffull-scope-review.md). Tracked docs (read directly): docs/INVARIANT-COVERAGE.md, docs/SECURITY-POSTURE.md, docs/ARCHITECTURE.md, docs/ENGINE-SPEC.md, docs/HOW-IT-WORKS.md.`

const RATIFIED = `RATIFIED + FIXED (do NOT re-litigate — critique the PLAN's faithfulness, soundness, completeness, and risk, not these decisions): the §6.2 codepoint-dispatched EncryptedEnvelope (Option-F+ pseudo-keypair NO-GO; SymmetricAead vault / HpkeBase[MLKEM768-X25519] drop+wraps) + Amendments 1-6; LAMPS Composite ML-DSA sig at 0x0001 (EUF-CMA-only at construction, SUF-equivalent at app-layer via Inv-15); MLKEM768-X25519 enc at 0x647A + ChaCha20-Poly1305 bulk; the X-Wing-mislabel corrective (~24 LOC); libcrux-ml-kem + Brendan McMillion hpke + keyring-core + Argon2id; Q1-Q4 + Am4 Sealed-Sender-DEFAULT + Path-A.5 (Anchor+Version+CURRENT preserved, immutable Version-Node-CIDs); the 9 ratified Ben-calls incl. BC-9 SHIP-ALL-5 RoleId (Admin/Moderator/Member/Viewer/Invitee active); {MembershipSet, Drop}; 3 orthogonal axes (Scale=EXACTLY-3 Kind / Governance=presets-on-Atrium-NOT-crypto-Kinds / Federation=SubsetRef); member=Principal (relation, nature DERIVED per Inv-22, member_type DELETED); AI-agent = derived Plugin-flavor; D6 iroh-gossip privacy; the GN graph-native wins (audit=version-chain+IVM, governance=signed-Node, the frozen-crypto rule); "Rust engine plugin" naming; compute=PeerResource composing to ZERO wire field; Inv-16..22; Compromise table (#31=LAMPS, revocation-reach=#62, #56-#61).`

const QUESTIONS = `The R0 seeds these 9 R1 questions (assess the ones in YOUR scope; reference others if they touch your lens):
- R1-Q-1: Layer-B per-Node AEAD vs Layer-A DAK-vault at-rest overlap -- complementary not duplicative? (orch my-pred: KEEP both)
- R1-Q-2: Sealed-Sender = DEFAULT (Ben ratified) -- confirm the wire-format default + the #43 metadata posture implications.
- R1-Q-3: the "unified envelope" (vault encrypted to a DAK-derived pseudo-pubkey) was NO-GO'd; per-layer-distinct (Layer-A=symmetric AEAD) stands; the unified idea (§14.1) stays NAMED-deferred -- confirm no contradiction.
- R1-Q-4 (SHARPENED): (a) DAG-CBOR outer-framing load-bearing? (b) Coarse 1-HOUR EPOCH BUCKETS -- VERIFY the privacy-metadata timestamp is DECOUPLED from EVERY functional timestamp (HLC ordering, UCAN nbf/exp validity, key-retention, the Inv-21 fork-tie-break, replay defense); confirm replay rides on GENERATION COUNTERS not time-precision; and assess whether 1 HOUR is the right granularity on the privacy-vs-precision-vs-forensics curve.
- R1-Q-5: does the audit-event version-chain SUBSTRATE ship at v1-beta-Core (cheap re-use) while audit query-tooling defers to Composing? (my-pred: yes)
- R1-Q-6: headline timeline = the honest ~7-15 week framing (Ben settled), not the optimistic ~4-5 week agent-dispatch-only number.
- R1-Q-7: Garden/Grove governance = signed-config-Node CONTENT not reserved wire-codepoints -- holds vs any future cross-engine moderation-policy interop?
- R1-Q-8: mint the 3 discretionary compromises (#56 journalist-FS, #59 KEM-key-confirm, #61 fingerprint-leak) at v1-beta for audit-readiness? (my-pred: mint all)
- R1-Q-9: define the Moderator + Invitee permission-set SEMANTICS at Core (UCAN templates gating ops) even though governance WORKFLOWS land in Phase-4-Meta-Composing? (my-pred: yes)`

const OUTPUT = `Ground-truth-verify any R0 claim about code/invariants/compromises against the tree at HEAD 2172cb6d (git show / grep) -- flag mismatches. HARD RULE 12 disposition on every finding (FIX-NOW, or OUT-OF-SCOPE / BELONGS-NAMED-NOW-with-destination / DISAGREE-WITH-EXPLANATION).

Output your review as a markdown block (this final message IS the deliverable -- do NOT call any structured-output tool):
## LENS: <your lens>
**Verdict:** APPROVE-FOR-R2 / APPROVE-WITH-FIXES / NEEDS-REVISION + confidence.
**Findings:** for each -- [ID] [SEVERITY: BLOCKER/MAJOR/MINOR/OBS] Title | R0 location | the issue | recommendation | disposition.
**R1-question resolutions:** for each question in your scope -- your ruling + reasoning (confirm/refine/refute the orch my-pred).
**New questions for R2/R3:** anything the plan leaves genuinely open.`

phase('Council')

const LENSES = [
  { key: 'cryptographer', scope: `The 4 encryption layers (A K_principal vault / B per-Node AEAD with the K(N) structural-KDF chain / C encrypt-to-recipient HPKE / D DAK + device-auth), the §6.2 codepoint-dispatched envelope + Amendments 1-6, the X-Wing-mislabel corrective, LAMPS (0x0001), MLKEM768-X25519 (0x647A), ChaCha20-Poly1305, Argon2id, the AAD/codepoint binding, side-channel surfaces (the ML-KEM SampleNTT timing behind the Option-F+ NO-GO), the tactical crypto-lib picks. Cryptographic SOUNDNESS of the composed whole. Owns R1-Q-1 + R1-Q-3.` },
  { key: 'p2p-systems', scope: `The MembershipSet primitive, federation via MemberRef::SubsetRef (depth-4 + cycle-detect), HLC causal ordering, Loro CRDT merge, the Inv-21 fork tie-break, sync convergence, iroh-gossip transport + D6 privacy, multi-device sync, DUAL-CID dedup. Distributed-systems correctness. Owns the HLC/ordering half of R1-Q-4 + R1-Q-5 (audit chain in the sync model).` },
  { key: 'threat-model-security', scope: `The end-to-end threat model across 4 layers + 3 Kinds, trust tiers, DAK + device-authentication, the remote-permission-call protocol (Signal-Provisioning + CTAP-2.2-inspired -- a pre-merge security mini-review is NON-NEGOTIABLE), multi-device-key-wrap, capability/UCAN gating, the audit-log threat model (insider-correlation, coerced-admin), the RBAC. Owns R1-Q-8 + R1-Q-9.` },
  { key: 'privacy-metadata-leak', scope: `Metadata-leakage + unlinkability: Sealed-Sender (Ben ratified DEFAULT -- confirm wire + #43 posture), iroh-gossip D6 topic privacy, DUAL-CID per-recipient unlinkability, and ESPECIALLY the COARSE 1-HOUR EPOCH BUCKETS -- the sharpened Q-4: verify the privacy-metadata timestamp is DECOUPLED from every functional timestamp (HLC / UCAN nbf-exp / retention / Inv-21 tie-break / replay), confirm replay rides on generation-counters not time-precision, and assess the 1-hour granularity. Owns R1-Q-2 + R1-Q-4.` },
  { key: 'wire-format-freeze', scope: `The §4 frozen-interface inventory (FREEZE vs CODEPOINT-RESERVE vs GRAPH-NATIVE vs PHASE-LATER-DEFER), the codepoint discipline + BE-endianness, and STRESS-TEST the frozen-crypto rule ("AAD-bound / gates-K_Set / verify-offline-cross-engine -> frozen; else graph") for completeness + edge cases. The canonical-bytes contract, additivity/forward-compat (does every future capability land additively?), the EXACTLY-3 Kind freeze, the G-CORE-9 freeze gate. Owns R1-Q-4(a) + R1-Q-7 + the frozen-RBAC half of R1-Q-9.` },
  { key: 'graph-native-coherence', scope: `The everything-is-graph claims + the GN wins (audit=version-chain audit-event Nodes + IVM view; membership-events=version-chain Nodes; governance/economics/member-nature = top-level graph Nodes NOT inside the sealed policy struct), the engine-plugin symmetry (graph plugin #18 / Rust engine plugin #19; the 3 openness tiers; the SPLIT crate trajectory), consistency with the 12-primitives-irreducible + code-as-graph theses, the benten-membership-set 15th-crate plan. Owns R1-Q-5 + R1-Q-7.` },
  { key: 'impl-feasibility-sequencing', scope: `The §7 wave-decomposition + canary-first sequencing (Wave-0 X-Wing+BE -> parallel canaries -> fan-out -> doc-wave -> close -> Composing), the crate plan + SPLIT model, the tactical picks (libcrux-ml-kem, McMillion hpke, keyring-core), dependency ordering, the Phase-4-Meta-Core vs Composing split, the §8 cost/timeline realism, the §9 exit criteria + v1-beta-tag gate (incl. the pre-tag external crypto audit). Owns R1-Q-6.` },
  { key: 'standards-interop', scope: `Cross-ecosystem-interop + standards-compliance: LAMPS Composite ML-DSA (OID 1.3.6.1.5.5.7.6.48, IANA early-allocation), JOSE / HPKE-RFC-9180 / COSE registries, multicodec/multihash codepoints, did:key + UCAN-Varsig, the cross-ecosystem-identifier-as-content principle (component algorithm IDs reference IANA registries -- never mint Benten algorithm numbers), and the 0x647A IETF reservation for MLKEM768-X25519. Owns R1-Q-8 from a standards lens.` },
]

const council = (await parallel(LENSES.map(l => () =>
  agent(
    `You are a senior ${l.key} reviewer performing an R1 critic-council review of ${R0}\n\nYour LENS / scope: ${l.scope}\n\n${BACKGROUND}\n\n${RATIFIED}\n\n${QUESTIONS}\n\n${OUTPUT}`,
    { label: `lens:${l.key}`, phase: 'Council' }
  )
))).filter(Boolean)

log(`Council complete: ${council.length}/${LENSES.length} lenses returned`)

phase('Consolidate')

const blocks = council.map((c, i) => `\n\n===== LENS ${i + 1} =====\n${c}`).join('')

const triage = await agent(
  `You are consolidating an R1 critic council on the F-full R0 plan-doc into a triage-ready surface for the human architects (you do NOT make the final fix/defer/disagree decisions -- you organize them).\n\nBelow are ${council.length} lens reviews. Produce a markdown R1 triage:\n1. **Convergence read** -- total BLOCKER / MAJOR / MINOR / OBS counts (deduplicated across lenses); does R1 need another round (any BLOCKER/MAJOR open)?\n2. **Deduplicated findings table** -- by severity; each with the lenses that raised it, the issue, and a PROPOSED HARD-RULE-12 disposition (FIX-NOW / DEFER-NAMED / DISAGREE) for the humans to ratify.\n3. **The 9 R1-question resolutions** -- one row per question: the council's ruling + confidence + whether it confirms/refines/refutes the orchestrator's my-pred. Flag loudly any divergence (esp. R1-Q-2 Sealed-Sender, R1-Q-4 coarse-bucket-decoupling).\n4. **The 3-5 most load-bearing items** to surface to the architects first.\n5. **New questions for R2/R3.**\n\nBe honest about lens disagreements; do not paper over them. Cite which lens said what. This final message IS the deliverable.\n\nLENS REVIEWS:${blocks}`,
  { label: 'consolidate-r1', phase: 'Consolidate' }
)

return triage