export const meta = {
  name: 'f-full-r4-test-review',
  description: 'F-full R4: 15-lens adversarial review of the R3 red-phase corpus → structure → adversarial-verify → completeness-critic → converge',
  phases: [
    { title: 'Review',       detail: '15 lenses, batched 4x, schema-free prose' },
    { title: 'Structure',    detail: '1 consolidator -> structured findings' },
    { title: 'Verify',       detail: 'adversarial refute per BLOCKER/MAJOR' },
    { title: 'Completeness', detail: 'fresh gap-hunt critic' },
    { title: 'Converge',     detail: 'final converge call; refuse on partial panel' },
  ],
}

// ---- shared anchors + canon (inlined; fresh-worktree agents cannot see gitignored .addl/CLAUDE.md except via git show) ----
const CORPUS = 'phase-4-meta-core/f-full-r3-consolidated'
const MAIN = '2172cb6d'
const R03 = 'phase-4-meta-core/f-full-r0-plan-r1fp-r03'
const ORCH = 'phase-4-meta-core/orchestration-2026-05-26'

const COMMON = `
You are ONE lens on the **F-full R4 test-review council**. R4 is the deep, adversarial review of the **R3 RED-PHASE test corpus** that gates R5 implementation. Your job: find everything WRONG, WEAK, MISSING, or MIS-FROZEN in the corpus *from your lens*, before R5 builds against it.

## READ-ONLY ACCESS CONTRACT (NON-NEGOTIABLE)
You are a read-only reviewer. You MUST NOT modify, checkout, cd, commit, branch, or stage anything. Read EVERYTHING via \`git show <ref>:<path>\` from the current repo. NEVER \`git checkout\`/\`cd\` to another branch or absolute path — that corrupts a shared working tree (a prior agent did exactly this and caused damage). Only \`git show\`, \`git ls-tree\`, \`git diff --name-only\`, \`git log\`, \`git grep <ref>\` are allowed.

## ANCHORS (read these)
- **Corpus** (the artifact under review) = branch \`${CORPUS}\` (off clean main \`${MAIN}\`).
  - Enumerate the 56 new test files: \`git diff --name-only --diff-filter=A ${MAIN} ${CORPUS} -- '*/tests/*.rs'\`
  - Read any file: \`git show ${CORPUS}:<path>\`
- **R0.3 canonical plan** (the spec the tests must faithfully pin): \`git show ${R03}:.addl/phase-4-meta/f-full-r0-plan.md\` (~1547 lines)
- **R2 test-landscape** (the 89-family catalog + coverage matrix + freeze-gating priorities): \`git show ${ORCH}:.addl/phase-4-meta/f-full-r2-test-landscape.md\`
- **R1 triage** (closed BLOCKER/MAJOR history): \`git show ${ORCH}:.addl/phase-4-meta/r1-triage.md\`

## WHAT F-FULL IS
A 4-layer encryption substrate + the MembershipSet primitive, all pre-v1-beta (Phase-4-Meta-Core freezes the v1 wire interface):
- **Layer A** = K_principal vault (symmetric AEAD-under-DAK; Argon2id; XChaCha20-Poly1305 24B nonce; vault codepoint 0x6100).
- **Layer B** = per-Node AEAD, structural-KDF key chain K(N).
- **Layer C** = encrypt-to-recipient (HPKE RFC-9180 base mode; MLKEM768-X25519 "X-Wing" @ 0x647A; multi-stanza groups; Sealed-Sender).
- **Layer D** = DAK / device-auth / remote-permission-call / multi-device key-wrap.
- **MembershipSet primitive** (everything-is-a-MembershipSet): ONE \`Principal\` (membership is a relation, not a type); EXACTLY-3 \`MembershipSetKind {Atrium, DeviceMesh, SingleDevice}\` (the only keying axis); 3 orthogonal axes (scale=keying / governance=Atrium->Garden->Grove presets NOT crypto-kinds / federation=MemberRef::SubsetRef); primitive set = {MembershipSet, Drop}; member nature (human/AI/ownership) is DERIVED never stored (Inv-22; member_type DELETED); AI-agent = derived Plugin-predicate (agents-are-plugins); compute composes from graph Nodes (ZERO new MembershipSet wire field).

## THE STAKES (why R4 is high-rigor)
~74 of the 89 families are **FREEZE-GATING** = they pin PERMANENT v1 wire bytes. A frozen-**WRONG** test is worse than no test: R5 will faithfully implement the wrong freeze and it becomes un-changeable. Scrutinize every freeze-gating byte/codepoint/AAD/length pin against R0.3 + R2.

## BLESSED CODEPOINT TABLE + BEN RULINGS (inline canon; the corpus MUST match these exactly)
- **Sealed-Sender = DEFAULT**, codepoint **0x6510**, ships at Core; abuse-control = recipient-issued delivery-tokens (**Compromise #63**). Group sends ALSO honor Sealed-Sender (**F-LC-9**: per-stanza inner-sender-DID bound INSIDE the group AAD; sender-DID NOT plaintext on group sends).
- **MembershipSet 0x6600** (relocated OUT of the MLS 0x6380/0x6390 bracket); group multi-stanza 0x6610; 0x6620 reserved.
- **X-Wing 0x647A** = REAL SHA3-256 construction \`SHA3-256(label || ss_M || ss_X || ct_X || pk_X)\` per draft-connolly-cfrg-xwing-kem (NOT HKDF-SHA256; the in-tree HEAD HKDF is the mislabel being corrected). Regen KAT vectors.
- DeviceLink 0x6310-0x631F; RemotePermission 0x6320-0x632F (+ ExecuteWorkflow reserve). Vault 0x6100.
- **Signature default = LAMPS Composite ML-DSA \`id-MLDSA65-Ed25519-SHA512\` @ 0x0001** (EUF-CMA-only at construction; SUF-equivalent at app-layer via Inv-15 3-layer decomposition). Bird-of-Prey = future-additive reserve.
- **Compromise #31 = LAMPS** (kept); revocation-reach renumbered to **#62**; #30 unaudited-PQ stays; #56 journalist-FS; #57-#61 collisions resolved; #63 Sealed-Sender abuse-control.
- Invariants: **Inv-16** envelope-unification · **Inv-17** hybrid-floor (no pure-PQ as SOLE trust path; classical is audited floor) · Inv-18 · Inv-19 · **Inv-20** (12 clauses incl. federation recursion-bound k) · **Inv-21** fork-tie-break TOTALITY (made total via forking-event Version-Node-CID discriminant) · **Inv-22** member-nature-derived-never-stored. (In-tree main has Inv-1..15 registered; Inv-16..22 are design-mints these tests pin.)
- **BC-9: all 5 RoleId ACTIVE at v1-beta** — Invitee=0, Viewer=1, Member=2, Moderator=3, Admin=4 (ordinal is AAD-keying-bound; Moderator=admin-subset, Invitee=pre-acceptance-limited).

## TDD RED-PHASE DISCIPLINE (pim-12 / §3.6f) — the bar each test must meet
Every test compiles green behind \`#[ignore = "RED-PHASE: <F-ID> ... un-ignore at R5"]\` against an in-file SELF-CONTAINED stub-shim. A SUBSTANTIVE pin (the bar) = invokes the real production entry point (at R5) + asserts an observable consequence + would-FAIL if the impl were a no-op. WEAK/INVALID = tautology (\`assert_eq!(CONST, CONST)\`), zero-assertion body, self-referential fixture (asserts an encoder against itself), or a negative arm whose falsifying case is constructed so it never actually triggers.

## KNOWN R5-FILL ITEMS (the R3 consolidator already flagged these — RE-EXAMINE THEM BLIND; decide for yourself fix-at-R4 vs genuinely-R5-fill vs worse-than-reported; do NOT just rubber-stamp)
1. W5 F-AAD-1 flagship hex-pin self-referential (calls the same encoder it asserts). 2. W5 F-CRDT-3 / F-MST-3 / F-GOSSIP-1 / F-INV21-4 pass-through/near-tautology stubs. 3. W4 crate2_b1_dep_set asserts a Cargo.toml *comment* (count 38 not 39). 4. W0 canary F-INV16-1 U3 length-injectivity (+U1, classical_0x6400, aead_lifts) pass-against-stub when intended-RED. 5. M-20: byte-pinning families must rebase onto the canary landing SHA at R5.

## OUTPUT CONTRACT
Return your findings AS YOUR FINAL MESSAGE (plain prose — do NOT call any structured-output tool). For each finding:
\`[SEVERITY] <file or "coverage"/"cross-cutting"> | <F-ID if any> | DEFECT: <one sentence> | WHY: <why it matters for freeze/R5> | DISPOSITION: <fix-now-at-R4 | R5-fill (named) | out-of-scope (reason) | disagree (cite)>\`
SEVERITY ∈ {BLOCKER, MAJOR, MINOR, OBS}. Ground EVERY finding in an actual line you read (cite file + what the code does) — no speculation. If you cannot substantiate it against the real corpus text, drop it. End with: \`LENS VERDICT: <APPROVE-FOR-R5 | FINDINGS-RAISED> — <N BLOCKER / N MAJOR / N MINOR / N OBS>\`.
`

const LENSES = [
  { key: 'substantive-pin', title: 'Substantive-pin & falsifiability', focus: 'ALL 56 files (sample broadly; deep-read the byte/CRDT/HLC pins)', mandate:
    'Does each test meet the SUBSTANTIVE bar (real entry point + observable consequence + would-FAIL-on-no-op)? Hunt EVERY weak/tautological/zero-assertion/self-referential/dead-negative-arm pin — not just the 5 known. For each suspect, transiently reason: "if I replaced the impl with a no-op / a wrong-but-plausible impl, would this test still pass?" If yes => the pin is weak. Pay special attention to negative arms whose falsifying input is mis-constructed so it never triggers (the F-INV16-1 U3 class).' },
  { key: 'r5-readiness', title: 'RED-PHASE -> GREEN R5-readiness', focus: 'all files: #[ignore] messages, stub-shim modules, un-ignore destinations', mandate:
    'pim-12 compliance: every test #[ignore]d with a RED-PHASE message naming a VALID un-ignore destination (a real future symbol path). Is the stub-shim -> real-type swap path unambiguous? Do any NQ-gated open-spec stubs get marked correctly (resolve-at-R5-not-now)? Re-examine the 5 known R5-fill items: confirm each is genuinely R5-fill vs should-fix-now-at-R4. Flag any test that can NEVER cleanly un-ignore (e.g. asserts a shape R0.3 contradicts).' },
  { key: 'determinism-ci', title: 'Determinism, flakiness, isolation & CI-realism', focus: 'proptest/CRDT/HLC/gossip pins + wasm32/feature-gated tests', mandate:
    'Hunt flake + CI-break risk: unseeded randomness, wall-clock/time-of-day deps, ordering assumptions, network, process-scoped shared statics that race under parallel nextest + cargo-llvm-cov (§3.13), proptest case-count sanity. CI-realism: will the corpus survive wasm32-unknown-unknown + MSRV + cross-platform + the pre-existing testing/test-helpers/benten-eval-testing feature-gating? Flag any test that pins something platform/feature-specific incorrectly.' },
  { key: 'wire-freeze', title: 'Wire-freeze byte-correctness', focus: 'the ~74 freeze-gating families: f_w0, f_lb_2, f_va_1, f_aad_1/2, f_lc_*, f_cp, f_inv16', mandate:
    'For EVERY freeze-gating family, does it pin the CORRECT canonical bytes per R0.3 §6.2 (EncryptedEnvelope) + the codepoint table? V2 (not V1), big-endian integers (zero to_le_bytes), correct AAD layout + ordering, length-injective TLV, EncryptedEnvelope-not-AeadEnvelope. A frozen-WRONG byte pin is the worst outcome — verify byte-by-byte against R0.3, not against the test author\'s own prose.' },
  { key: 'codepoint-registry', title: 'Codepoint-registry integrity', focus: 'f_cp_codepoint_registry_dispatch, f_inv16, f_lc_*, f_trans_1, f_ms_*', mandate:
    'Every codepoint integer in the corpus matches the BLESSED table EXACTLY (Sealed-Sender 0x6510, MembershipSet 0x6600/0x6610/0x6620, X-Wing 0x647A, DeviceLink 0x6310-1F, RemotePermission 0x6320-2F, vault 0x6100, sig 0x0001). No collisions, no IANA/MLS-bracket intrusion (MLS keeps 0x6380/0x6390). Is there a band-collision scanner pin? Are typed-reject (unsupported-codepoint fail-closed) arms present + correct?' },
  { key: 'crypto-construction', title: 'Crypto-construction correctness', focus: 'f_w0, f_kat_1/2/3/4, f_sm_inv17, f_lb_1/3, f_va_2', mandate:
    'Highest-stakes lens. X-Wing: is it pinned as REAL SHA3-256(label||ss_M||ss_X||ct_X||pk_X) NOT HKDF? KAT vectors faithful to draft-connolly / FIPS-203 / LAMPS (or correctly flagged as synthesized-witness-pending-real-corpus)? Swap matrix bidirectional + complete? Inv-17 hybrid-floor (no pure-PQ as SOLE trust path; classical = audited floor) correctly pinned? HPKE RFC-9180 base mode? Argon2id params (m=19456,t=2,p=1)? AEAD nonce discipline (24B XChaCha20)? NQ-C1 (HPKE-KEM-extensibility) correctly pinned as a two-branch decision-fork?' },
  { key: 'encoding-serialization', title: 'Encoding / serialization correctness', focus: 'f_aad_1/2, f_lb_2, f_va_1, canonical_bytes_v1, f_nqc4_1', mandate:
    'Canonical DAG-CBOR: deterministic field ordering, CBOR byte-string vs array correctness, length-prefix injectivity (the F-AAD-1 members_table length-injective pin — is the collision pair VALID, i.e. does it actually produce equal-length-different-content that the injective encoding must distinguish? the F-INV16-1 U3 bug is this class), the all-big-endian / zero-to_le_bytes sweep (M-19), multicodec/multihash framing (did:key hybrid-pubkey, F-NQC4-1). Flag any encoding pin that is self-referential or tests the wrong injectivity property.' },
  { key: 'invariant-semantics', title: 'Invariant semantics (Inv-16..22)', focus: 'f_inv16_1, f_sm_inv17, f_lc_abuse(inv18), f_..inv19, f_inv21, f_nat_1(inv22)', mandate:
    'Does each Inv-16..22 family encode the invariant\'s REAL meaning (not a shallow proxy), and does its NEGATIVE arm actually fire on a violation? Inv-16 envelope-unification (codepoint-in-AAD, one-HPKE-primitive); Inv-17 hybrid-floor; Inv-18; Inv-19; Inv-20 (12 clauses incl. federation recursion-bound k); Inv-21 fork-tie-break TOTALITY via Version-Node-CID; Inv-22 member-nature-derived-never-stored (member_type DELETED). Cross-check each against R0.3 + INVARIANT-COVERAGE conventions.' },
  { key: 'capability-rbac', title: 'Capability / authority / UCAN / RBAC', focus: 'f_audit_*, f_ms_4_5(roleid), f_ms_6_7(moderator), f_gov_1, f_lc_*', mandate:
    'RBAC: all 5 RoleId active + correct ordinals (Invitee=0..Admin=4, BC-9); Moderator=admin-subset + Invitee=pre-acceptance-limited permission-sets defined; ordinal AAD-keying-bound. UCAN delegation templates, sealed CapabilityPolicy (the trait is sealed per #7), install-time consent, audience-binding, the authority-half (capability-gating binds a cooperating engine; confidentiality is the OTHER half). Audit: enforced-WRITE-path attribution, version-chain tamper-evidence, access-gradation (restricted-scope NOT a codepoint).' },
  { key: 'threat-model', title: 'Confidentiality / threat-model + bounded-decode (adversary lens)', focus: 'f_lc_hpke_sealed_sender, f_lc_abuse, f_va_*, f_disc_*, f_trans_1, f_ld_*', mandate:
    'Adopt the ADVERSARY view. Does the corpus actually test the threat model? Untrusted-host (peers-hold-ciphertext); harvest-now-decrypt-later (PQ-hybrid from first format version); Sealed-Sender metadata/anonymity (sender-DID not on wire, incl. F-LC-9 groups); traffic-analysis (blinded gossip topic); replay (nonce-cache); key-compromise + revocation-reach (#62: revocation cuts future serves, already-derived keys stay valid); forward-secrecy (#56). BOUNDED-DECODE: does the wire-decode pin bounded/length-checked decode (no unbounded-decode DoS, the META #629 class)? Flag any threat with NO covering pin.' },
  { key: 'distributed-sync', title: 'Distributed-systems / sync semantics', focus: 'f_crdt, f_mst, f_gossip, f_hlc, f_inv21, f_fed_1_2', mandate:
    'CRDT convergence laws (commutativity/associativity/idempotence — is the membership-set merge actually tested for all three, or a shallow proxy?); HLC admitted-vs-created + clock-skew bound; MST anti-entropy backstop; gossip transport placement + blinded-topic; Inv-21 fork-tie-break TOTALITY (smaller created_at_hlc wins, made total via Version-Node-CID — is the totality actually exercised on a real tie?); federation SubsetRef recursion-bound (Inv-20 clause-k). Flag pass-through/near-tautology convergence stubs (the F-CRDT-3/F-MST-3 class).' },
  { key: 'membershipset-fidelity', title: 'MembershipSet primitive-shape fidelity (ratified M-CONS-FINAL)', focus: 'f_ms_1..9, f_crate_1_2, f_nat_1, f_fed_1_2, f_aad_2', mandate:
    'Does the corpus pin the RATIFIED M-CONS-FINAL shape, not a stale earlier design? EXACTLY-3 MembershipSetKind (Atrium/DeviceMesh/SingleDevice) — the ONLY keying axis; members_table fusion + MemberEntry shape; derive-nature-not-store (Inv-22; member_type DELETED — flag ANY test that stores member_type/member-kind); agents-are-plugins (AI-agent = derived predicate, not a stored type); 3 orthogonal axes (governance = Atrium->Garden->Grove PRESETS not crypto-kinds; federation = MemberRef::SubsetRef); {MembershipSet, Drop} primitive set; compute-composes-to-ZERO-wire-field; the 15th-crate boundary (F-CRATE: crypto-suite/sync NEVER depend on benten-membership-set).' },
  { key: 'coverage-gap-audit', title: 'Coverage completeness / adversarial gap audit', focus: 'R2 landscape coverage matrix + the full corpus file list + f_disc_*/f_freeze_1 catch-nets', mandate:
    'Independently RE-DERIVE the coverage claim — do NOT trust the R3 consolidator\'s "89/89, zero uncovered." Walk R0.3 + R2: every Inv-16..22, every Compromise #30-#63, every BLESSED codepoint, every NQ-* (C1-C5/D1-D2/T2-T4/W2/W4/A1-A2/etc), every §9 exit-criterion — does each map to a REAL behavioral pin (not just the f_disc/f_freeze parametrized catch-net)? Flag anything covered ONLY by a catch-net that should have a dedicated behavioral family. Check NQ-gating correctness (freeze-gating-vs-v1-GM).' },
  { key: 'cross-wave-seam', title: 'Cross-wave seam & M-20 consistency', focus: 'cross-cutting: compare stub-shims across all waves for shared concepts', mandate:
    'The 8 waves authored SELF-CONTAINED in-file stub-shims independently. Do they AGREE on shared concepts — the EncryptedEnvelope layout, codepoint integers, MemberEntry/members_table shape, the V2/BE conventions, the K(N) structural-KDF? When R5 deletes the shims and inserts real types, will any inconsistency cause divergence (two waves assuming different byte layouts for the same wire object)? Verify W1-W5 byte-pinning families are mutually consistent + consistent with the W0 canary (the M-20 rebase dependency). Flag every shim-disagreement.' },
  { key: 'ruling-fidelity', title: 'Ruling-fidelity & cross-language mirror (pim-13 R7)', focus: 'cross-cutting: walk every Ben-ruling + R0.3 frozen-decision to its pin', mandate:
    'R7-style spec-to-test compliance: walk EVERY Ben-ruling (Sealed-Sender DEFAULT, #31=LAMPS + revocation->#62, X-Wing real SHA3-256, F-LC-9 group-honors-Sealed-Sender, BC-9 all-5-RoleId, derive-nature, agents-are-plugins, compute-zero-wire, codepoint table) + every R0.3 FROZEN decision — does each have a faithful, present pin? Flag any ruling with NO test or a test that contradicts it. Cross-language: do any new ErrorCodes / wire types crossing the napi/TS boundary need a TS-mirror pin (§3.5g first-class error-variant mirror)? Is that accounted for?' },
]

function briefFor(L) {
  return `${COMMON}

## YOUR LENS: ${L.title}
**Primary focus files:** ${L.focus}
**Your mandate:** ${L.mandate}

Read your focus files in full (via \`git show ${CORPUS}:<path>\`) + the relevant R0.3/R2 sections. Be adversarial and specific. Cite real lines. Then emit findings per the OUTPUT CONTRACT.`
}

function chunk(arr, n) { const out = []; for (let i = 0; i < arr.length; i += n) out.push(arr.slice(i, i + n)); return out }

const FINDINGS_SCHEMA = {
  type: 'object', additionalProperties: false,
  properties: {
    panel_lenses_returned: { type: 'integer' },
    panel_lenses_expected: { type: 'integer' },
    findings: { type: 'array', items: {
      type: 'object', additionalProperties: false,
      properties: {
        id: { type: 'string' },
        severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS'] },
        lens: { type: 'string' },
        file: { type: 'string' },
        family: { type: 'string' },
        claim: { type: 'string' },
        why_matters: { type: 'string' },
        recommended_disposition: { type: 'string' },
      }, required: ['id', 'severity', 'lens', 'file', 'claim', 'recommended_disposition'],
    } },
    summary: { type: 'string' },
  }, required: ['panel_lenses_returned', 'panel_lenses_expected', 'findings', 'summary'],
}

const VERDICT_SCHEMA = {
  type: 'object', additionalProperties: false,
  properties: {
    finding_id: { type: 'string' },
    verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PARTIAL'] },
    corrected_severity: { type: 'string', enum: ['BLOCKER', 'MAJOR', 'MINOR', 'OBS', 'NONE'] },
    reasoning: { type: 'string' },
  }, required: ['finding_id', 'verdict', 'corrected_severity', 'reasoning'],
}

// ===================== PHASE 1: REVIEW (15 lenses, batched 4) =====================
phase('Review')
log(`R4 review council: ${LENSES.length} lenses, batched into sub-waves of 4 (rate-limit discipline)`)
const reviewReports = []
for (const batch of chunk(LENSES, 4)) {
  const res = await parallel(batch.map(L => () =>
    agent(briefFor(L), { label: `R4-lens:${L.key}`, phase: 'Review' })))
  reviewReports.push(...res)
}
const liveReports = reviewReports.filter(Boolean)
log(`Review returned ${liveReports.length}/${LENSES.length} lenses`)

// ===================== PHASE 2: STRUCTURE (1 consolidator -> structured findings) =====================
phase('Structure')
const structurePrompt = `You are the R4 structuring consolidator. Below are ${liveReports.length} lens reports (of ${LENSES.length} dispatched) reviewing the F-full R3 red-phase corpus.

Extract EVERY distinct finding into the structured schema. Dedup findings that multiple lenses raised (keep the highest severity + note the corroborating lenses in \`lens\`). Assign stable ids (F4-001, F4-002, ...). Set \`panel_lenses_returned\`=${liveReports.length} and \`panel_lenses_expected\`=${LENSES.length}. In \`recommended_disposition\` use exactly one of: fix-now-at-R4 / R5-fill / out-of-scope / disagree. Do NOT invent findings; only structure what the lenses actually reported.

=== LENS REPORTS ===
${liveReports.map((r, i) => `\n----- REPORT ${i + 1} -----\n${r}`).join('\n')}`
const structured = await agent(structurePrompt, { label: 'R4-structure', phase: 'Structure', schema: FINDINGS_SCHEMA })

// ===================== PHASE 3: ADVERSARIAL VERIFY (refute each BLOCKER/MAJOR) =====================
phase('Verify')
const majors = (structured?.findings || []).filter(f => f.severity === 'BLOCKER' || f.severity === 'MAJOR')
log(`Adversarial-verify: ${majors.length} BLOCKER/MAJOR findings to refute`)
const verdicts = []
for (const batch of chunk(majors, 4)) {
  const res = await parallel(batch.map(f => () =>
    agent(`${COMMON}

## YOUR JOB: ADVERSARIALLY REFUTE this R4 finding
A council lens raised the finding below about the F-full R3 corpus. Try to REFUTE it: go to the actual corpus file + R0.3 and check whether the finding is REAL. Default to **REFUTED** if you cannot substantiate it against the real corpus text (we reject plausible-but-wrong findings). Only CONFIRMED if you independently reproduce the defect by reading the cited code. PARTIAL if real but mis-severity.

FINDING ${f.id} [${f.severity}] (lens: ${f.lens})
file: ${f.file}  family: ${f.family || 'n/a'}
claim: ${f.claim}
why_matters: ${f.why_matters || ''}

Read the cited file via \`git show ${CORPUS}:${f.file}\` (and R0.3 if it\'s a freeze/spec claim). Return your verdict.`,
      { label: `R4-verify:${f.id}`, phase: 'Verify', schema: VERDICT_SCHEMA })))
  verdicts.push(...res.filter(Boolean))
}

// ===================== PHASE 4: COMPLETENESS CRITIC (fresh gap-hunt) =====================
phase('Completeness')
const criticPrompt = `${COMMON}

## YOUR JOB: COMPLETENESS CRITIC (fresh, independent)
The 15-lens council + adversarial-verify just reviewed the corpus. Your job is the FRESH gap-hunt: what did the WHOLE panel likely MISS? Ask: (a) which R0.3 frozen-decision / Inv / Compromise / codepoint / NQ-* / §9-exit-criterion has NO dedicated behavioral pin (only a catch-net, or nothing)? (b) which threat-model adversary path is untested? (c) which cross-wave seam inconsistency would only surface at R5? (d) is there a whole test-FAMILY or lens the council didn\'t cover? Independently spot-check R2\'s coverage matrix against the real corpus. Return a prose list of GAPS (each: what\'s missing + severity + why + recommended family/pin to add at R3.2-or-R5). If you find no real gap, say so explicitly.`
const critic = await agent(criticPrompt, { label: 'R4-completeness-critic', phase: 'Completeness' })

// ===================== PHASE 5: CONVERGE (final call; refuse on partial panel) =====================
phase('Converge')
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED' || v.verdict === 'PARTIAL')
const convergePrompt = `You are the R4 convergence consolidator. Decide whether the F-full R3 red-phase corpus is CONVERGED (ready to advance to R5) or needs an R3.2/R4-fix pass.

PANEL INTEGRITY: ${liveReports.length} of ${LENSES.length} lenses returned. **If fewer than ${LENSES.length} returned, you MUST NOT certify CONVERGED** — a rate-limited silence is not an APPROVE (verify-by-artifact, never notification-absence). Instead call "PANEL-INCOMPLETE — re-run missing lenses."

INPUTS:
- Structured findings (${(structured?.findings || []).length}): ${JSON.stringify(structured?.findings || [], null, 1)}
- Adversarial verify verdicts on BLOCKER/MAJOR: ${JSON.stringify(verdicts, null, 1)}
- Confirmed/partial-after-verify count: ${confirmed.length}
- Completeness critic gaps:
${critic}

PRODUCE:
1. CONVERGENCE CALL: one of CONVERGED / NOT-CONVERGED / PANEL-INCOMPLETE. Convergence requires: full panel returned AND zero CONFIRMED BLOCKER AND zero CONFIRMED MAJOR (REFUTED findings don\'t count; MINOR/OBS don\'t block; completeness-critic gaps that are genuinely R5-fill don\'t block but MUST be named).
2. The triage table: every CONFIRMED finding (drop REFUTED) with final severity + disposition (fix-now-at-R4 / R5-fill-named / out-of-scope / disagree per HARD-RULE-12).
3. The R4-FIX work-list (what must change in the corpus before R5) vs the R5-FILL carry-list (named items for the R5 brief).
4. A crisp summary for the orchestrator + Ben.`
const converge = await agent(convergePrompt, { label: 'R4-converge', phase: 'Converge' })

return {
  panel: `${liveReports.length}/${LENSES.length}`,
  structured,
  verdicts,
  critic,
  converge,
}
