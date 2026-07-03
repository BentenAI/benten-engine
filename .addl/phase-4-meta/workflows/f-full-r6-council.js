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

const CFG = (args && args.cfg) || {
  tier: 'R6',
  artifactRef: '76e99968',
  artifactDesc: 'Phase-4-Meta-Core at PHASE-CLOSE — the landed crypto/identity substrate that FREEZES the v1 public interface. (A) Sealed-Sender encryption: benten-crypto-suite (vault/Argon2id-DAK + secrecy; per-Node AEAD; X-Wing hybrid KEM at codepoint 0x647a with libcrux-ml-kem 0.0.9 constant-time ML-KEM-768; the codepoint registry + typed-reject; the unified KEM-DEM key-encryption) + benten-drop (Layer-C encrypt-to-recipient: 0x6510 single / 0x6520 group with blinded per-stanza AAD; DUAL-CID; + per-MESSAGE LAMPS-hybrid SENDER-ORIGIN-AUTH [B2 #1366]: inner sender signature over M_auth verified post-decrypt, fail-closed SenderOriginAuthFailed, §4.1 inter-member non-forgeability now TRUE) + benten-id (hybrid did:key resolve_hybrid via two registered-component multikeys) + benten-engine/src/layer_d (remote-permission, device-link HPKE-wrap, multi-device wrap, enforced-WRITE audit path, governance, derived member-nature). (B) Signature default 0x0001 = byte-faithful IETF LAMPS composite id-MLDSA65-Ed25519-SHA512. (C) MembershipSet primitive (benten-membership-set, the 15th crate: kind/member/role/AAD + RBAC; CRDT convergence, HLC tie-break, Inv-21 fork-tie-break, MST anti-entropy, blinded gossip topic, governance, audit, derived nature). (D) the v1-FROZEN-INTERFACE contract + Inv-15..22 + Compromise #30..64.',
  enumerate: 'git ls-tree -r --name-only 76e99968 -- crates/benten-crypto-suite crates/benten-membership-set crates/benten-drop crates/benten-engine/src/layer_d crates/benten-engine/tests docs/V1-FROZEN-INTERFACE.md docs/SECURITY-POSTURE.md docs/CRYPTO-CODEPOINTS.md docs/THREAT-MODEL.md docs/SECURITY-PROOFS.md docs/INVARIANT-COVERAGE.md docs/V1-WIRE-FORMAT-INVENTORY.md',
  specRef: '76e99968',
  specPath: 'docs/V1-FROZEN-INTERFACE.md',
  framing: 'PHASE-CLOSE convergence council - ROUND 13 (re-reviewing the phase at r9-base 76e99968 after the ROUND-12 fix wave + the F-11 fallible-seal hardening landed). ROUND 12 = full 14-lens convergence council against 76e99968; the R11 crypto fixes (MC-1 random-CEK, MC-6 vault-frame) ALL re-confirmed, and the NEW freeze-safety-of-zero-production-caller-surfaces lens (GAP-D) RAN and found 0 BLOCKER/MAJOR (the >=4 zero-caller frozen surfaces derive_kv/K(V) + Inv-21 comparator + accept_grant + layer-C-engine-use are forward-compatible-additive). ROUND 12 was NOT-CONVERGED on exactly 1 CONFIRMED MAJOR + a doc/MINOR/named-carry tail, ALL now FIXED on 76e99968 + ORCH-ground-truthed + full re-gate GREEN (1663 crypto-crate tests + 941 engine; cargo-doc -D warnings / cite-drift / §3.5g scanner clean; the sole nextest timeout is the pre-existing in_memory_observationally_equivalent_to_redb slow-under-parallel-load proptest, CONFIRMED PASS isolated at 101.5s, unrelated to any change). MC/F-01 FIX (MAJOR — bounded-decode / unbounded-allocation DoS): benten-graph/src/aead_wrap.rs decode_encrypted_node variant 0x01 read an attacker-controlled count=u32::from_be_bytes(bytes[38..42]) (<=u32::MAX) then Vec::with_capacity(count) BEFORE the per-chunk bounds guard — a 42-byte blob with count=0xFFFFFFFF forced a multi-GB alloc-abort, reachable PRE-AUTH via redb_backend.rs get_encrypted_node (raw stored bytes decoded before AEAD/integrity on the untrusted-host tier). FIX: a ceiling max_chunks=(bytes.len()-42)/9 (9 = 4-byte len-prefix + 5-byte minimal AeadEnvelope) checked BEFORE the pre-alloc, rejecting with a dedicated typed AeadError::ChunkCountExceedsInput{count,max}; regression tests chunk_count_overflow_is_typed_reject_not_alloc_abort (count=0xFFFFFFFF -> typed reject, would-FAIL-on-revert) + chunk_count_boundary_at_max_encodable; encode-side with_capacity confirmed plaintext-derived (safe); neighbor sweep clean; THREAT-MODEL §6 bounded-decode claim now accurate. F-11 HARDENING (pre-freeze, ideal-shape Ben-delegated — make the crypto seal honestly fallible on caller input rather than panic): the 0x6520 group seal encodes recipient_count as a wire-frozen u16 (§4.0 by-band width; the 0x6610 MembershipSet band uses u32 — width-unification is a REJECTED freeze record, UNCHANGED), and sealing >65535 recipients previously PANICKED in plaintext_aad_bytes. FIX: a named pub const MAX_LAYER_C_GROUP_RECIPIENTS=u16::MAX(65535) + a typed LayerCError::RecipientCountExceedsBandWidth{count,max} (LayerCError is #[non_exhaustive] + §3.5g-baselined, no ErrorCode/TS mirror) + a validate_group_roster_len helper called as the FIRST statement of seal_group_impl; seal_group_impl/seal_group_multi/seal_group_multi_plaintext_sender now return Result<EncryptedEnvelope,LayerCError> (14 test callers unwrap; ZERO production callers); plaintext_aad_bytes stays infallible (its .expect is now an unreachable defense referencing the const + seal-entry guarantee); unit test accepts_at_limit_and_rejects_over_limit (MAX->Ok, MAX+1->typed Err, would-FAIL-on-revert); benten-drop.txt public-api regen exact; WIRE-INVENTORY §26 F-11 note RESOLVED (typed-error + const; u16 by-band Ben-confirmed intentional, >65535-single-send out-of-scope-by-design). MINOR/OBS tail fixed: F-02 (V1-FROZEN §3 TxAborted clause-b) + F-03 (census operative live-enum-count) + F-04 (AAD_VERSION cross-crate mirror) + F-06 (stale LE docstring) + F-14 (row_window #6-vs-#60 boundary) + F-15 (self-ORed contains). NAMED-CARRY registered: F-05(§11 ChangeEvent) F-07(Inv-16 wrap_key_material seam) F-08(Row D-70 WrappedKey consolidation) F-09(Row D-71 device_link) F-11(WIRE §26 RESOLVED) F-12(Inv-22 row 22) F-13(§6.8 HashCodepoint u64) F-16(Compromise #11 pin UN-IGNORED) F-17(RED-PHASE header retense) F-19(V1-FROZEN banner pre-tag-sweep) F-25(CI-FU-1 reword). RE-VERIFY the ROUND-12 + F-11 fixes CORRECT + COMPLETE + NO new defect: (a) F-01 - the chunk-count ceiling is sound (fires before pre-alloc; no valid encoding rejected; the typed error is honest) + no OTHER attacker-controlled length-prefix drives an unbounded alloc/decode anywhere in the wire-decode paths (re-sweep aead_wrap/layer_c/membership-set/drop-bundle); (b) F-11 - the seal is genuinely fallible on the one caller-controllable cardinality bound, the validator is correct, plaintext_aad_bytes stayed infallible with an unreachable defense, no caller path broke, and making JUST this input-check fallible (while internal-invariant .expect()s stay panics) is coherent; (c) the MINOR/named-carry landings are accurate. RE-VERIFY THE WHOLE PHASE FRESH every round per feedback_full_phase_review_every_round (re-check EVERY freeze byte / golden / codepoint / invariant vs spec, NOT just the round-12-fix delta). Keep freeze-safety-of-zero-production-caller-surfaces (GAP-D, now standing — every zero-caller frozen surface must be forward-compatible-additive with its eventual Composing caller) + gap1-closure-and-public-derivable-secret-class-sweep (hunt ANY public-derivable secret) + crypto-error-oracle/failure-mode-uniformity + at-rest-format-migration/forever-decode (the vault V2 frame) + sender-origin-authentication-spoofing + as-built-doc-reconciliation + bounded-decode/unbounded-allocation (the F-01 class — sweep EVERY attacker-controlled length prefix). CONVERGENCE ARC (deeper lenses kept surfacing real defects - do NOT short-circuit): R6(content-splice BLK)->R7(CONF-1 nonce-reuse BLK)->R8(GAP-1 placeholder-key BLK)->R9(nonce-durability MAJ)->R10(2 doc MAJ)->R11(2 REAL MAJ crypto: 0x6520 public-CEK + vault-salt)->R12(1 MAJ DoS: aead_wrap unbounded-alloc)->R13(this re-review of the round-12 + F-11 fixes). The LAST review before tag phase-4-meta-core-close freezes the v1 public interface. Q5 cadence: FULL council every round; iterate ACROSS INVOCATIONS until 0 confirmed BLOCKER/MAJOR; pattern-induction findings additive. Review the LANDED implementation + tracked docs + the frozen-interface contract (NOT a red-phase test-pin review). Hunt SUBSTANTIVE defects (crypto correctness, wire-freeze byte-correctness, fail-closed gaps, bounded-decode/unbounded-alloc, error-oracle uniformity, at-rest forever-decode, freeze-safety of zero-caller surfaces, doc-versus-code drift, invariant-coverage at HEAD, deferred-honesty, SemVer/public-surface), not phantom-destinations.',
  mustInclude: ['crypto-construction-correctness','wire-freeze-byte-correctness','codepoint-registry-integrity','invariant-semantics-inv15-22','membershipset-primitive-fidelity','sender-origin-authentication-spoofing','capability-authority-ucan-layerd','confidentiality-threat-model-bounded-decode','distributed-sync-crdt-hlc-gossip','supply-chain-libcrux-cryspen-vet','public-surface-semver-freeze','spec-to-code-ruling-fidelity-pim13-r7','deferred-items-hardrule-honesty','cross-target-wasm-deployment-shapes','as-built-doc-reconciliation-prose-currency','crypto-error-oracle-failure-mode-uniformity','at-rest-format-migration-forever-decode','gap1-closure-and-public-derivable-secret-class-sweep','freeze-safety-of-zero-production-caller-surfaces'],
  targetLensCount: 23,
  priorMissedLenses: 'ROUND-8 completeness-critic found GAP-1 (the frozen recipient-keying used a public-derivable placeholder secret) as a lens-gap NO panel lens owned. Round 9 MUST include a dedicated lens that (1) VERIFIES the GAP-1 fix genuinely closed it (open takes a real RecipientSecret unrecoverable from public; the [u8;32] placeholder + wrapping_sub(0x80) + deterministic-expansion FULLY deleted with no production caller; a public-only party cannot open on all 3 codepoints; would-FAIL-on-revert) AND (2) SWEEPS FOR THE CLASS - is there ANY OTHER secret/key anywhere in the corpus that is derivable from public material, or a placeholder/deterministic-from-public key, or a secret carrying less entropy than it appears (layer_d device-link + multi-device key-wrap, swap_matrix, hpke, the vault DAK, K_principal, K_Set, any KDF whose secret input is actually public, any _for_test/deterministic keygen reachable in production)? The GAP-1 class - a secret that is not actually secret - is the deepest confidentiality failure mode; hunt it everywhere. ALSO keep the two round-8-added deep lenses (now standard): crypto-error-oracle/failure-mode-uniformity + at-rest-format-migration/forever-decode-of-prior-schema. ALSO keep the sender-origin-authentication-spoofing lens (round-2 origin missed-lens) and the as-built-doc-reconciliation lens (round-1 missed-lens) - re-verify every tracked doc narrative of what-is-built-AND-enforced versus the artifact at r9-base 76e99968, especially the NEW recipient-secrecy claims in SECURITY-PROOFS §4.1/4.2 (are they genuinely true as-built) and the DEFERRED D-64 honesty (is the engine-e2r-wiring deferral accurately NOT-a-Compromise).',
  canon: `## PROJECT CANON (inline — review agents CANNOT read gitignored .addl/CLAUDE.md; tracked docs ARE readable via: git show 76e99968:docs/...)

### Crypto-agility (CLAUDE.md baked-in #5)
Permanent commitment = the self-describing multiformats framing, NOT any one algorithm. Algorithms are codepoint-dispatched + ADDITIVE (new codepoint, never a wire-break/fork). Typed-reject on unknown crypto (fail-closed; NEVER silent fallback). NEVER fork/reimplement primitives — the suite is concat/hash/codepoint/envelope glue over vetted upstream crates. Never hardcode key/sig/ct sizes (flow from upstream type constants).

### Signature default 0x0001 (Ben decision A, 2026-06-05)
Byte-faithful IETF LAMPS composite id-MLDSA65-Ed25519-SHA512 (OID 1.3.6.1.5.5.7.6.48), draft-ietf-lamps-pq-composite-sigs-19 + test-vector commit f0627ab3. M-prime = "CompositeAlgorithmSignatures2025"(32) then "COMPSIG-MLDSA65-Ed25519-SHA512"(30) then len(ctx)u8 then ctx then SHA-512(M); ML-DSA signs M-prime with ctx=Label (NOT bare empty-ctx); Ed25519 signs M-prime; wire = mldsaSig(3309) then tradSig(64) = 3373B ML-DSA-first, NO commitment trailer; pubkey = mldsaPK(1952) then tradPK(32) = 1984B. Both halves MUST verify, fail-closed, no single-half-accept. EUF-CMA-only / Weakly-Non-Separable per draft section 9.2.2/10 (Compromise #31); SUF closed at app-layer by Inv-15. draft-19 NOT yet RFC (Prefix embeds 2025) — re-verify at RFC (agility absorbs additively).

### Encryption (NQ-C1 Branch B, Ben-ratified)
Layer-C = Benten-supplies-the-KEM (own ml-kem + x25519 + sha3 KEM-DEM under a Benten envelope at 0x647a; NOT RFC-9180 cross-stack-interop — intentional; an RFC-9180-faithful suite is future-additive). X-Wing combiner = SHA3-256(ss_M then ss_X then ct_X then pk_X then XWingLabel) with the 6-byte XWingLabel 0x5c2e2f2f5e5c APPENDED (prepend order is WRONG). K(V) derives over the FULL self-describing CIDv1 (0x01 0x71 0x1e 0x20 then 32-byte BLAKE3 = 36 bytes), NOT the bare digest. ML-KEM-768 production impl = libcrux-ml-kem 0.0.9 (hax/F-star-verified constant-time portable backend; Compromise #32 mitigated-LIVE). RustCrypto ml-kem retained as a dev-dependencies cross-impl KAT witness; FIPS-203 serialization byte-identical across impls (NQ-C2). The 13 Cryspen transitive crates = HONEST cargo-vet exemptions (budget self-test cap 5 to 18, Ben-ratified), folded into the v1-GM C-GM-AUDIT scope (Compromise #30).

### Invariants Inv-15..22 (docs/INVARIANT-COVERAGE.md is the spec)
Inv-15 = sig-bundle CIDs are NEVER load-bearing identifiers (identity = canonical-payload-CID, authentication = codepoint-dispatched-signature, revocation = semantic-tuple) — the SUF-CMA closure. Inv-16 = envelope-unification. Inv-21 = fork-tie-break totality (version-node-CID). Inv-22 = derived-member-nature.

### Named v1-GM deferrals (HARD-RULE clause-b — GENUINE, do not flag as gaps)
f_kat_1 / f_kat_4 (cross-impl/interop KAT conformance) to v1-GM; f_kat_2 runnable check-secret-independence CI-gate carries to the libcrux version that fixes the upstream 0.0.9 E0053 macro defect (the verified portable backend IS the live #32 mitigation; the mlkem-ct-check feature is the one-flag-away seam); Inv-21 kani proof arm to v1-GM (proptest surrogate = the v1-beta floor). The independent ml-dsa/ml-kem audit GATES the v1-GM tag (NF-2 / C-GM-AUDIT).

### Convergence bar + standing law
0 confirmed BLOCKER/MAJOR across the full council = R6 CONVERGED to pre-tag sweep to tag phase-4-meta-core-close (Ben-gated; HELD). NEVER --admin-bypass / force-push; NORMAL --squash only. Surface real architectural/freeze forks to Ben (do not resolve unilaterally).`,
}

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
