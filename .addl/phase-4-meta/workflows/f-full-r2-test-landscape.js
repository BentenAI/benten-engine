export const meta = {
  name: 'f-full-r2-test-landscape',
  description: 'R2 test-landscape synthesis: 6 multi-modal test-family discovery agents (batched 3+3) + completeness critic + synthesis',
  phases: [
    { title: 'Discover-A', detail: 'crypto-envelope / membership-sync / threat-security' },
    { title: 'Discover-B', detail: 'wire-freeze / privacy-metadata / graph-native' },
    { title: 'Completeness', detail: 'what test families are MISSING?' },
    { title: 'Synthesize', detail: 'unified R2 test landscape + R3 slicing' },
  ],
}

const R0 = `the converged F-full R0.2 plan-doc (read: git show 477529d0:.addl/phase-4-meta/f-full-r0-plan.md, 1547 lines §0-§13). It is R1-CONVERGED. Your job is to discover the TEST FAMILIES that R3 test-writers will implement (TDD red-phase), NOT to re-review the plan.`

const COMMON = `For your dimension, enumerate the TEST FAMILIES. For each family give: a NAME; WHAT IT PINS (the observable property / would-FAIL-if-no-op'd); the R0 mapping (which §-section / Inv-16..22 / Compromise #56-#63 / frozen-codepoint / NQ-* question / §9 exit-criterion it covers); the RED-PHASE intent (the assertion shape); whether it is a FREEZE-GATING test (must pass before the wire freezes) vs functional vs conformance; and rough family size (n tests). Be exhaustive within your dimension — coverage is the goal; a missed family becomes an untested frozen byte. Cross-reference the R0's own §9 exit criteria + the carried NQ-* questions.

Output as a well-organized markdown section (final message IS the deliverable; NO structured-output tool). Ground-truth against HEAD 2172cb6d where you cite existing test files/patterns to reuse.`

const DIMS = [
  { key: 'crypto-envelope', wave: 'A', scope: `The 4 encryption layers (A vault / B per-Node AEAD K(N) chain / C HPKE encrypt-to-recipient / D DAK), the §6.2 codepoint-dispatched EncryptedEnvelope + Amendments 1-6, the real X-Wing SHA3-256 construction @0x647A (BR-3), LAMPS @0x0001, ChaCha20/XChaCha20-Poly1305, Argon2id, the FULL bidirectional swap matrix + typed-reject arms, KAT/golden-vector regen (Wave-0 BE migration), constant-time/side-channel surfaces, the libcrux<->RustCrypto FIPS-203 KAT (NQ-C2), and the McMillion-hpke RFC-9180-context question (NQ-C1, freeze-gating).` },
  { key: 'membership-sync-crdt', wave: 'A', scope: `MembershipSet ops + the 5-role RBAC, federation via SubsetRef (depth-4 + cycle-detect, offline-decidable KSetAcquisitionPath NQ-D3), HLC causal order + the Inv-21 fork-tie-break TOTALITY (created_at_hlc ASC + Version-Node-CID terminal discriminator; the kani convergence proof NQ-D2), Loro CRDT merge (LWW larger-HLC vs fork smaller-HLC asymmetry), sync convergence under out-of-order/duplicate delivery, iroh-gossip (gossip=liveness vs MST=convergence; GossipTransport placement NQ-D1), DUAL-CID dedup, the members_table canonical-bytes/AAD-injectivity (NQ-W4).` },
  { key: 'threat-security', wave: 'A', scope: `DAK + device-authentication, the remote-permission-call protocol (Signal-Provisioning+CTAP-2.2-inspired) + the §6.7 6-pass security mini-review gate, multi-device-key-wrap, capability/UCAN gating + role_assignments_generation-in-AAD + E_ROLE_STALE_AT_VERIFY, the RBAC permission-sets (Invitee=zero-content, Moderator strict-subset-of-Admin), the audit-log threat model + AuditAccessGradation, replay defense via the nonce-cache (intra-hour-replay-REJECTED; multi-device shared-nonce NQ-T4), ExecuteWorkflow no-egress enforcement (NQ-T3), the abuse-control / Compromise #63 (sealed-sender-default spam-mitigation), coerced-approval (#33 ext).` },
  { key: 'wire-freeze-conformance', wave: 'B', scope: `The §4 frozen-interface inventory conformance, the blessed codepoint table + intra-band non-collision (Inv-18; the CI band-collision scanner NQ-W2), canonical-bytes contract + AAD length-injectivity, additivity/forward-compat (every future capability lands additively), the EXACTLY-3 MembershipSetKind freeze + §15.c HALT-AND-SURFACE, the ENVELOPE_FORMAT_VERSION_V2 single-migration + BE-endianness (no to_le_bytes survives on any wire/AAD path), cross-ecosystem CONFORMANCE VECTORS (NQ-C3: BouncyCastle 1.80+/OpenSSL-3.5/OpenPGP-PQC <-> Benten LAMPS verifier, both directions), did:key hybrid-pubkey multicodec (NQ-C4, freeze-gating), the typed unsupported-algorithm reject arms.` },
  { key: 'privacy-metadata', wave: 'B', scope: `Sealed-Sender DEFAULT @0x6510 (sender-DID inside ciphertext; #43 posture), the COARSE 1-HOUR BUCKET decoupling (the bucket is Layer-D-only; HLC/UCAN-nbf-exp/retention/generation-counters all separate; DropToRecipient carries NO timestamps M-14), iroh-gossip D6 topic-blinding (HMAC-blinded + fork-rotation + OOB-rendezvous), DUAL-CID per-recipient unlinkability (network-observer-only scoping vs insider), the #43/#48 metadata posture, the plaintext_cid_local never-serialized pin.` },
  { key: 'graph-native-invariant', wave: 'B', scope: `The Inv-16..22 enforcement property tests (esp. Inv-21 tie-break + Inv-22 member-nature-derived-never-stored + Inv-20 clauses), audit-log-as-version-chain audit-event Nodes + IVM view + UCAN-gated + enforced-attribution-path (not bare put_node), governance-as-signed-config-Node, the GN graph-native wins, the members_table fusion, the engine-plugin trait-seam roster (the 3 openness tiers), the benten-membership-set 15th-crate boundary + the SPLIT mechanism/data halves, member-nature derivation (did:agent: + UCAN root-issuers).` },
]

const runDim = (d) => () => agent(
  `You are a senior test-architect doing R2 test-landscape DISCOVERY for the "${d.key}" dimension of ${R0}\n\nDimension scope: ${d.scope}\n\n${COMMON}`,
  { label: `discover:${d.key}`, phase: `Discover-${d.wave}` }
)

phase('Discover-A')
const a = (await parallel(DIMS.filter(d => d.wave === 'A').map(runDim))).filter(Boolean)
log(`Discover-A: ${a.length}/3 returned`)

phase('Discover-B')
const b = (await parallel(DIMS.filter(d => d.wave === 'B').map(runDim))).filter(Boolean)
log(`Discover-B: ${b.length}/3 returned`)

const found = [...a, ...b]
const foundBlocks = found.map((f, i) => `\n\n===== DIMENSION ${i + 1} =====\n${f}`).join('')

phase('Completeness')
const gaps = await agent(
  `You are a COMPLETENESS CRITIC for the F-full R2 test landscape. ${found.length} discovery dimensions enumerated test families (below). Read ${R0} and the discovery output, then find WHAT IS MISSING: for EVERY Inv-16..22, every Compromise #56-#63 (+ #30/#31), every FROZEN codepoint in the §4 table, every carried NQ-* question, and every §9 exit-criterion — is there at least one test family covering it? List the GAPS (item with NO family, or thinly-covered), and propose the missing family for each. Be adversarial about coverage; this is the catch-net before the wire-freeze. Final message IS the deliverable.\n\nDISCOVERY OUTPUT:${foundBlocks}`,
  { label: 'completeness-critic', phase: 'Completeness' }
)

phase('Synthesize')
const landscape = await agent(
  `You are synthesizing the F-full R2 TEST LANDSCAPE. Merge the ${found.length} discovery dimensions + the completeness critic's gap-fills into ONE unified, deduplicated test-landscape document for ${R0}\n\nProduce markdown:\n1. **Test-family catalog** — organized by family, each with: name · what it pins · R0/Inv/Compromise/codepoint/NQ mapping · FREEZE-GATING vs functional vs conformance · red-phase intent · size.\n2. **Coverage matrix** — every Inv-16..22, Compromise #56-#63, frozen-codepoint, NQ-*, and §9 exit-criterion -> the family(ies) covering it (flag any still-uncovered).\n3. **R3 test-writer slicing** — how to partition the families into N parallel R3 test-writer waves (by file/crate ownership, disjoint), with the canary-first + freeze-gating families prioritized.\n4. **Freeze-gating test priorities** — the families that MUST be green before any wire-minting canary (esp. NQ-C1/C2/C3/C4, the codepoint table, the AAD canonical-bytes).\n5. **R2->R3 open questions** carried forward.\n\nHonest about any residual coverage gap. Final message IS the deliverable.\n\nDISCOVERY:${foundBlocks}\n\nCOMPLETENESS-CRITIC GAPS:\n${gaps}`,
  { label: 'synthesize-r2', phase: 'Synthesize' }
)
return landscape