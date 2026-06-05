/*
 * addl-r5-impl-to-green.js — FINALIZED R5 implementation: canary-first impl waves, each looping impl→un-ignore→test→fix until GREEN.
 *
 * R5 turns the R3/R4 RED-PHASE corpus GREEN: implement real production code so the #[ignore]'d tests pass.
 * Canary-first (feedback_canary_first_parallel_implementation): the canary wave owns the API surface the
 * others consume — it runs FIRST and gates the fan-out on its merge. Each wave loops impl→un-ignore→nextest→fix
 * until its tests pass, mini-reviewed (pim-2 substantive / §3.6f). Then strategy-C integrate + full-suite.
 *
 * ⚠️ Unlike the review/fix-loops, R5 agents WRITE real code + run cargo → they use isolation:'worktree' with the
 * loud ABSOLUTE-PATH-FORBIDDEN escape contract (feedback_agent_isolation_escape_absolute_paths; the W6 lesson).
 * ≤7 implementer cap (read-only mini-reviews are cap-exempt) — scale waves accordingly. Watch disk (97% HARD-ABORT,
 * ~90% cargo-clean-idle valve). M-20: byte-pinning families rebase onto the canary's landing SHA before un-ignore.
 *
 * METHODOLOGY ENCODED: canary-first · iterate-to-convergence (per-wave impl-to-green loop) · pim-2 substantive
 *   (un-ignore = real entry + observable + would-FAIL-on-revert) · §3.5h pre-push gate · §3.14 strategy-C
 *   · mini-review-after-every-group · decide-as-Ben + decision-log. (See workflow-common.js + converging-fix-loop.js.)
 *
 * INVOKE: Workflow({ name:'addl-r5-impl-to-green', args:{ cfg:{...}, canary:{...}, waves:[...] } })
 *   args.cfg    = { corpusBranch, baseBranch, specRef, specPath, MAX_FIX_ROUNDS, crates:[{name,features}], canon }
 *   args.canary = { key, brief, branch, mints:'<the types/surface the fan-out depends on>' }
 *   args.waves  = [{ key, brief, branch, files:'<un-ignore targets>' }]   (disjoint-file slices, ≤7 per batch)
 *
 * AUTHORING NOTES (orchestrator, when writing the concrete invocation):
 *  - Thread the r4-triage §3.B R5-FILL carry-list into the wave briefs (real-NIST/draft-connolly KAT swap,
 *    bounded-decode family, substrate-wiring verify-notes, §3.5g ErrorCode mints).
 *  - Make the LAST wave a DOC-WAVE (Rule #6 — docs in the last group): ERROR-CATALOG + CATALOG_VARIANT_COUNT,
 *    spec retenses, missing_docs sweep — not an afterthought.
 *  - CI is the AUTHORITATIVE full-workspace gate: the workflow's per-crate FullSuite is a pre-CI proxy; take the
 *    integration branch -> PR -> CI -> NORMAL --squash merge (never auto-merge; never --admin-bypass).
 */

export const meta = {
  name: 'f-full-r5-tier1',
  description: 'F-full R5 TIER-1: canary (benten-crypto-suite substrate) + 3 disjoint waves (drop ∥ engine ∥ ms-canary); impl->un-ignore->test->fix to GREEN; gate fan-out on canary; strategy-C integrate. TIER-2 ms-sync + TIER-3 doc dispatched after.',
  phases: [
    { title: 'Canary' }, { title: 'CanaryGate' }, { title: 'Waves' }, { title: 'MiniReview' }, { title: 'Integrate' }, { title: 'FullSuite' },
  ],
}

// ════════════════════════════════════════════════════════════════════════════════════════════════════
// INLINED CONFIG for the F-full R5 TIER-1 run (per-run script; the reusable template reads args.{cfg,canary,waves}).
// This invocation = CANARY (benten-crypto-suite substrate) + the 3 genuinely-independent TIER-1 waves
// (w-drop ∥ w-engine ∥ w-ms-canary, disjoint crates). The dependent tail (TIER-2 w-ms-sync → TIER-3 w-doc)
// is dispatched by the orchestrator AFTER this integrates (ms-sync consumes ms-canary's types; w-doc needs all).
// Corpus base `f-full-r5-base @ a964107b` = R4-converged red-phase corpus + the F-full freeze record (folded in).
// ════════════════════════════════════════════════════════════════════════════════════════════════════
const RESET = 'git fetch origin && git reset --hard a964107b'                                         // canary: onto the R5 base (corpus + freeze record)
const RESET_CANARY = 'git fetch origin && git reset --hard origin/phase-4-meta-core/f-full-r5-canary-crypto' // fan-out: onto the MERGED CANARY (base + crypto-suite substrate the use-statements need)

const CANON = `
## F-FULL R5 STANDING CANON (applies to every wave)
- **The freeze record is ALREADY in your corpus base** (\`f-full-r5-base @ a964107b\` = R4-converged red-phase corpus + Compromise #32–#64 + Inv-16..22 + Cat-A MLKEM768-X25519 rename, folded in). Do NOT re-mint compromises/invariants.
- **X-Wing combiner freeze (do NOT regress the R4-fix ruling):** \`SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)\` — the 6-byte \`XWingLabel = 0x5c2e2f2f5e5c\` is **APPENDED** (suffix), per draft-connolly-cfrg-xwing-kem-10 §6; codepoint 0x647A. The PREPENDED form is the superseded v01-v02 order and would freeze a non-interoperable KEM.
- **BE endianness everywhere** on wire/AAD/keying paths (R0.7 §4.1 M-19 site-list). LE on any such path is a defect; the X-Wing info-tag ASCII string is NOT a flagged integer (m-1 negative).
- **#5 — NEVER fork/reimplement crypto primitives.** benten-crypto-suite is the ONLY crypto call site; wrap vetted upstreams (RustCrypto / libcrux-ml-kem / McMillion hpke). Codepoint dispatch has a typed-reject (fail-closed) unknown-algorithm arm — never a silent fallback.
- **M-20 (you have the REAL encoder for the first time):** every byte-pinning golden was frozen by STUBS + hand-verified at R4. Recompute each via the real encoder, confirm byte-match; if it differs, UPDATE to the real bytes + FLAG-FOR-BEN (which golden, old→new, why). NEVER weaken an assertion to pass — reconcile to the real bytes.
- **§3.5g:** any minted ErrorCode → atomic Rust variant + \`packages/engine/src/errors.generated.ts\` + \`docs/ERROR-CATALOG.md\` + \`CATALOG_VARIANT_COUNT\` in the SAME commit.
- **HARD RULE 12:** implement every assigned family; only out-of-scope(reason)/belongs-named-now(named destination)/disagree(cite) are valid non-do. The ONLY legit \`#[ignore]\`-stays-at-R5 = the F4-047 real-KAT hard-gate (keep ignored + FLAG-FOR-BEN if a real published corpus can't be acquired at impl-time; NEVER pass vs a sentinel + claim conformance) + the Inv-21 kani arm (proptest is the v1-beta floor).
- **Compile gate via BASH** (zsh won't word-split the feature args); GREEN = zero failures by error-line count, NOT a masked shell EXIT after a pipe. Scoped per-package only (no \`--workspace\`). Commit before returning; decision-log every golden recompute + FLAG-FOR-BEN.
`

const CFG = {
  corpusBranch: 'f-full-r5-base',
  baseBranch: 'main',
  specRef: 'phase-4-meta-core/f-full-r0-plan-r07',
  specPath: '.addl/phase-4-meta/f-full-r0-plan.md',
  MAX_FIX_ROUNDS: 3,
  crates: [
    { name: 'benten-crypto-suite', features: '--features testing' },
    { name: 'benten-membership-set', features: '--features testing' },
    { name: 'benten-drop', features: '--features testing' },
    { name: 'benten-engine', features: '--features test-helpers --features benten-eval/testing' },
  ],
  canon: CANON,
}

const CANARY = {
  key: 'w-canary-crypto-substrate',
  branch: 'phase-4-meta-core/f-full-r5-canary-crypto',
  mints: 'benten-crypto-suite encryption substrate: EncryptedEnvelope (M-18 lift from AeadEnvelope) + EnvelopePayload/BindingContext (#[non_exhaustive]); ENVELOPE_FORMAT_VERSION_V2 + BE serializer + bounded-decode from_wire_bytes (META #629); cipher_suite::combine_x_wing (real SHA3-256, label APPENDED 0x5c2e2f2f5e5c) + classical_combine; codepoint.rs full §4.0 table + dispatch + typed-reject + CodepointLifecycle + IANA-disjoint scanner; swap_matrix.rs full bidirectional matrix (0x647c PURE_PQ audit-gated reject); structural_kdf.rs K(N) chain (+ clause-h sibling-confinement) + Argon2id DAK; vault.rs (NEW vault.cbor DAG-CBOR, SymmetricAeadXNonce 24-B, secrecy::SecretBox, UnlockedKeyMaterial, EngineLocked typed-reject); primitives.rs (libcrux-ml-kem swap, McMillion hpke NQ-C1, FIPS-203 KAT, LAMPS sig interop); conformance::endianness::wire_path_le_survivor_count()',
  files: 'ALL 15 crates/benten-crypto-suite/tests/f_*.rs',
  brief: `You own the benten-crypto-suite encryption substrate — the production surface EVERY other wave consumes. The fan-out is GATED on your merge.
FIRST ACTION (literal): \`${RESET}\`. Your worktree is based off the orchestration branch which does NOT contain the F-full corpus; this reset brings you to the R5 base (R4-converged red-phase corpus + freeze record). Verify \`git log --oneline -1\` shows a964107b and \`ls crates/benten-crypto-suite/tests/f_*.rs\` shows 15 files. THEN implement.
SCOPE: make all 15 crates/benten-crypto-suite/tests/f_*.rs families GREEN by writing the REAL production code. Each file's module-doc header names its exact production target + un-ignore recipe (DELETE the local stub-shim module → INSERT real \`use benten_crypto_suite::…\` → un-ignore → verify GREEN → regen goldens). Follow each recipe exactly.
COMPILE GATE (bash): \`cargo nextest run -p benten-crypto-suite --features testing\` then \`… --features testing --run-ignored all\` after un-ignoring. Loop impl→un-ignore→test→fix up to 3 rounds.
FREEZE ANCHORS (do NOT regress): (1) X-Wing combiner label APPENDED 0x5c2e2f2f5e5c (canon). (2) BE everywhere — deliver \`conformance::endianness::wire_path_le_survivor_count()\` returning 0 (F-W0-3 pin consumes it); complete M-19 site-list per R0.7 §4.1 (aead.rs:165 codepoint + aead.rs:244,277 chunk/recipe + structural_kdf.rs:157 + varsig.rs:47,107 + sizes.rs:183 + swap_matrix.rs:1539,1540,1548,1550 + aead_wrap + platform-foundation).
M-20 byte-pin families: f_w0, f_inv16_1, f_va_1, f_va_2, f_lb_2, f_kat_1, f_kat_3, f_kat_4 — recompute each golden via the real encoder + confirm/UPDATE+FLAG (canon). The real X-Wing means KAT/keypair-derived goldens WILL legitimately change; log each.
R5-FILL you own: (a) F4-009/F4-030 META#629 bounded-decode — a decode-side family on EncryptedEnvelope::from_wire_bytes / U3 length-prefix: a hostile declared length (recipient_count 0xFFFF with zero stanzas) MUST typed-reject BEFORE allocation. (b) F4-040 F-CP-2 scanner — enumerate the REAL minted codepoint symbol set (registry-iterator/source-scan, NOT a hand-list) + full IANA HPKE kem/kdf/aead ranges + a baseline arm injecting a colliding const that proves the scanner fires. (c) F4-047 real KAT corpora — f_kat_1 (NIST FIPS-203), f_kat_3 (RFC-9180 HPKE), f_kat_4 (LAMPS BouncyCastle/OpenSSL) currently pin SYNTHESIZED sentinels; swap for REAL published corpora. **HARD-GATE: if a real corpus cannot be acquired at impl-time, KEEP the family #[ignore]'d with a 'R5-FILL: awaiting real <X> corpus' message + FLAG-FOR-BEN — do NOT pass vs a sentinel.** (d) F4-035 legacy-combiner args from the same real keypair. (e) clause-h: f_lb_1 negative-confinement arm (K(X)-holder cannot derive sibling K(Y)).
§3.5g on any minted ErrorCode (EngineLocked lives here — discharge if new public). HARD RULE 12. Commit before return; decision-log every golden recompute + FLAG-FOR-BEN.
**AFTER GREEN + commit: PUSH your branch — \`git push -u origin phase-4-meta-core/f-full-r5-canary-crypto\` — the fan-out waves RESET ONTO it to get your substrate, so the push is load-bearing (not optional). Report the pushed branch + landing SHA.**`,
}

const WAVES = [
  {
    key: 'w-drop-layer-c',
    branch: 'phase-4-meta-core/f-full-r5-drop-b',
    files: '7 production families: f_lc_hpke, f_lc_abuse_control, f_lc_dual_cid, f_drop_no_k_set, f_nqa1_1, f_nqc4_1, f_trans_1 (the 3 doc-coupling families f_disc_1/f_disc_2/f_freeze_1 are DEFERRED to the w-doc tail — leave them #[ignore]\'d, named-deferral not skip)',
    brief: `Layer-C (benten-drop) impl-to-green. FIRST ACTION (literal): \`${RESET_CANARY}\` — this resets you onto the MERGED CANARY (corpus + freeze record + the crypto-suite substrate your \`use benten_crypto_suite::…\` need). Verify the substrate is present (\`grep -rl EncryptedEnvelope crates/benten-crypto-suite/src\`) + 10 benten-drop/tests/f_*.rs present. THEN implement.
SCOPE (7 production families): make GREEN — f_lc_hpke, f_lc_abuse_control, f_lc_dual_cid, f_drop_no_k_set, f_nqa1_1, f_nqc4_1, f_trans_1. Mint \`benten_drop::layer_c::{seal_sealed_sender, seal_plaintext_sender, open_single, seal_group_multi, open_group_stanza, serialize}\` (headers name \`unimplemented!()\` stubs to replace). Sealed-Sender DEFAULT 0x6510; group 0x6520 BLINDED per-stanza AAD (8 fields, u16 widths, audience_set_commitment = BLAKE3 over sorted recipient DIDs, sealed-sender-DID INSIDE ciphertext per F-LC-9). DUAL-CID extends benten-sync::two_cid_store (envelope_blob_cid vs plaintext_cid; plaintext_cid_local never-serialized; plaintext_cid_set blinded). f_drop_no_k_set: scan the serialized DropBundlePayload, assert NO K_Set sentinel (R4-minted confidentiality invariant). f_nqc4_1 touches benten-id::did.rs (hybrid-pubkey multicodec; did:agent: alias). f_trans_1: codepoint/transport reserve (Willow/iroh-roq/iroh-live typed-reject; GossipPlusBlobs ships).
**DEFER (do NOT implement now):** f_disc_1, f_disc_2, f_freeze_1 stay #[ignore]'d — they are doc-coupling/tally families run at the w-doc TIER-3 step after the new crypto-docs land (named deferral to w-doc, HARD RULE clause-b — NOT a silent skip).
COMPILE GATE (bash): \`cargo nextest run -p benten-drop --features testing\` (+ \`--run-ignored all\` for your 7). GREEN by error-line count.
M-20 byte-pin (f_lc_hpke, f_lc_abuse_control): recompute the 0x6520 group-AAD golden + the abuse token-binding AAD golden via the real encoder; confirm/UPDATE+FLAG. NOTE: f_lc_hpke's 0x6520 AAD is a SIBLING of (not identical to) the membership 0x6610 AAD — drop=u16, membership=u32, §4.0 width-unification REJECTED; do NOT unify them.
R5-FILL: F4-030 (bounded-decode HpkeMultiBase recipient_count typed-rejects before alloc); F4-028 (token-binding AAD BE byte-pin); F4-015 (reconcile CID 36-B CIDv1 vs 32-B digest with the canary's shape). HARD RULE 12. §3.5g on any minted ErrorCode. Commit before return; decision-log.`,
  },
  {
    key: 'w-engine-layer-d',
    branch: 'phase-4-meta-core/f-full-r5-engine-b',
    files: '16 families: f_ld_1,2,3,4,6,7,8 + f_va_3 + f_audit_1,2,3,4 + f_gov_1 + f_inv19_1 + f_nat_1,2 (all crates/benten-engine/tests/f_*.rs)',
    brief: `Layer-D + audit/governance/nature (benten-engine) impl-to-green. FIRST ACTION (literal): \`${RESET_CANARY}\` — resets you onto the MERGED CANARY (corpus + freeze record + crypto-suite substrate). Verify the substrate present (\`grep -rl EncryptedEnvelope crates/benten-crypto-suite/src\`) + 16 benten-engine/tests/f_*.rs. THEN implement all 16. (You consume the canary's EncryptedEnvelope/HPKE; multi-device-wrap also reuses Layer-C HPKE — benten_drop::layer_c is a SIBLING wave not yet merged, so implement the wrap against the canary HPKE primitive directly + FLAG the eventual layer_c reuse.)
SCOPE: mint \`benten_engine::layer_d::remote_permission::{PermissionRequest, PermissionGrant, PermissionOperation (incl ExecuteWorkflow reserve)}\` (0x6320..0x632F); \`Provisioning{Offer,Payload,InnerPayload}\` device-link (0x6310..0x631F; HPKE-wrap K_principal to device B's fresh keypair); the 6-class remote-permission harness (f_ld_6); DeviceAuthBackend sealed trait + headless BENTEN_VAULT_PASSWORD; keyring-core v1.0.0 + file-vault fallback + Tauri IPC smoke; the enforced-WRITE audit-emit path (attribution triple; store.rs:468; is_actor_active-gated); audit:<set_id>:* RestrictedScope arm (NOT a 3rd Scope codepoint); GovernanceConfig{tier} top-level signed Node; K(V) type-restriction (rejects non-Version-Node/non-MembershipSet CID); Inv-22 derived-nature (no nature field; is_ai_operated from method-parse; IVM view); f_nat_2 network-observer-only unlinkability scoping.
F-LD-6 is the §9.1-4 freeze gate — carries a mandatory pre-merge security mini-review (owner = threat-model lens; clean on ALL SIX classes: replay/device-key-revocation/clock-skew/confused-deputy/UI-deception/audit-Node-binding). Surface that the mini-review is owed.
COMPILE GATE (bash): \`cargo nextest run -p benten-engine --features test-helpers --features benten-eval/testing\` (+ \`--run-ignored all\`). GREEN by error-line count.
M-20 byte-pin (f_ld_2, f_ld_3, f_ld_4, f_ld_8): recompute PermissionRequest/Grant golden CBOR, ExecuteWorkflow AAD hex, Provisioning* golden, timestamp-exclusion structural pin via real encoder; confirm/UPDATE+FLAG.
R5-FILL: F4-015 (f_ld_2 — restore scope/requesting_device_did/ephemeral_signing_key/reason; reconcile concat-vs-CBOR); F4-030 (bounded-decode PermissionRequest); F4-011 (retense the stale 'OPEN-SPEC/NQ-T2/T3 gated' docstrings — ALL RATIFIED 2026-06-02 per R0.7 §10.5/§10.6); F4-020 (f_ld_6 roll-up enumerates-and-invokes, not literal-array-len); F4-006 (R0.7 §4.1: DropToRecipient/0x6510/0x6610 carry NO coarse_epoch — implement per R0.7; if a corpus stub disagrees, FLAG-FOR-BEN); key_retention_window_secs=604800 default pin (#62) in f_ld_7/f_ld_8. HARD RULE 12. §3.5g on any minted ErrorCode. Commit before return; decision-log.`,
  },
  {
    key: 'w-ms-canary',
    branch: 'phase-4-meta-core/f-full-r5-ms-canary-b',
    files: '9 families: f_ms_1, f_ms_2, f_ms_3, f_ms_4_5, f_ms_6_7, f_ms_8_9, f_aad_1, f_aad_2, f_crate_1_2 (the MembershipSet primitive structure/RBAC/AAD surface; the 6 sync/CRDT families are TIER-2 w-ms-sync, dispatched later)',
    brief: `MembershipSet-primitive canary (benten-membership-set, the 15th crate) impl-to-green. FIRST ACTION (literal): \`${RESET_CANARY}\` — resets you onto the MERGED CANARY (corpus + freeze record + crypto-suite substrate). Verify the substrate present (\`grep -rl EncryptedEnvelope crates/benten-crypto-suite/src\`) + the 15 benten-membership-set/tests/f_*.rs. THEN implement YOUR 9 (f_ms_1, f_ms_2, f_ms_3, f_ms_4_5, f_ms_6_7, f_ms_8_9, f_aad_1, f_aad_2, f_crate_1_2). The 6 sync families (f_crdt, f_hlc_1_2, f_inv21, f_mst, f_gossip, f_fed_1_2) are TIER-2 — leave them #[ignore]'d (a later wave consumes your member/role/aad types).
SCOPE: un-comment the B-1 dependency block in Cargo.toml (benten-crypto-suite, benten-core, benten-caps, benten-id, benten-graph, benten-sync); mint \`benten_membership_set::{kind::MembershipSetKind (EXACTLY-3 — no #[non_exhaustive] wildcard; a 4th arm is a compile error), member::{MemberEntry, MembersTable, MemberRef}, role::RoleId (5-value ordinal Invitee=0…Admin=4, all-active), aad::assemble_group_aad}\`. members_table = BTreeMap<Did, MemberEntry> one-DID-one-record, ZERO nature/member_type field. AAD assembler hands OPAQUE Vec<u8> to crypto-suite (compile-fence: crypto-suite has NO reverse dep — verify Cargo.toml). RBAC: Invitee derives ZERO content; Moderator ⊊ Admin (strict); per-role UCAN ability-templates. 0x6600 set-keying / 0x6610 group multi-stanza.
**You are a SECOND-ORDER CANARY** — the TIER-2 sync wave consumes your member/role/aad types. Mint them complete + stable.
COMPILE GATE (bash): \`cargo nextest run -p benten-membership-set --features testing\` (+ \`--run-ignored all\` for your 9). GREEN by error-line count.
M-20 byte-pin (f_aad_1, f_aad_2, f_ms_3 [shares f_aad_1 golden], f_ms_4_5, f_ms_6_7): recompute the members_table canonical-CBOR golden, the 0x6600/0x6610 9/11-tuple AAD golden (u32 widths; commitments — audience_set_commitment = BLAKE3 over sorted DIDs; membership_set_id_commitment = blake3::keyed_hash(K_Set, "benten:setid:v1"‖id)), the RoleId ordinal hex, the per-role UCAN templates via the real encoder; confirm/UPDATE+FLAG. Reconcile MemberEntry repr (member_ref CBOR + role ordinal) to the ONE shape the R4 golden froze (F4-006/F4-007).
R5-FILL: **F4-031 — §3.5g mint for E_ROLE_STALE_AT_VERIFY** (f_ms_8_9): atomic Rust variant + errors.generated.ts + ERROR-CATALOG.md + CATALOG_VARIANT_COUNT in ONE commit; sweep sibling new MembershipSetError public variants. F4-014/F-CRATE-2 (f_crate_1_2 — parse [dependencies] not raw .contains(); the B-1 edges are real now). F4-040 (add benten-membership-set/testing to ALL CI --features lists — note the delta for w-doc to land). F4-020/F4-032 (f_ms_6_7 — Admin set includes role-change; real ability_template + grep-defense). F4-034 (DeviceMesh admin = user-DID typing). F4-014 K(V): membership K(V) derives via blake3::derive_key("benten-membership-set:K(V):v1", cid) — note this for the engine f_inv19_1 K(V) type-restriction to agree (cross-wave; engine owns the restriction, you own the keying). HARD RULE 12. Commit before return; decision-log goldens + the §3.5g discharge.`,
  },
]
const MAX_FIX = CFG.MAX_FIX_ROUNDS || 3

const COMMON = `
## ISOLATION CONTRACT (NON-NEGOTIABLE — you run in an auto-managed git worktree)
ALL git ops + file reads/writes stay INSIDE your worktree (your dispatch-time \`pwd\`). NEVER cd to the main repo / absolute paths outside it / \`git -C /other/path\`. To read main-repo or sibling-branch state use \`git show <ref>:<path>\`. Violating this races other parallel implementers + corrupts the shared tree (the W6 escape).

## SCOPED PRE-FLIGHT ONLY (laptop resource discipline)
NEVER run workspace cargo (\`--workspace\`). Run scoped per-package: \`cargo test -p <crate> <features> --no-run\` / \`cargo nextest run -p <crate> <features>\` / single-crate clippy + fmt. CI is the authoritative full-workspace verifier. If asked for workspace cargo, DECLINE + surface.

## ANCHORS
- Spec of record: \`git show ${CFG.specRef}:${CFG.specPath}\`. RED-PHASE corpus base: \`${CFG.corpusBranch}\` (your tests are the #[ignore]'d pins you make pass).
- Disposition (HARD RULE 12): implement everything; only out-of-scope(reason)/belongs-named-now(named)/disagree(cite) are non-do.

## THE IMPL-TO-GREEN BAR (pim-2 / §3.6f)
For each assigned RED-PHASE test: implement the REAL production surface it pins, DELETE the in-file stub-shim, insert the real \`use ...\`, REMOVE the \`#[ignore]\`, and make it pass via \`cargo nextest run\`. The un-ignored test MUST exercise the real entry point + assert an observable consequence + would-FAIL if reverted. Beware Rust-2024 reserved keywords. Commit before returning (auto-worktrees auto-clean uncommitted work).

## M-20 GOLDEN RECONCILIATION (freeze-byte safety — MANDATORY for every byte-pinning test)
The corpus goldens were computed by STUBS (M-20: frozen-vs-stub) and HAND-VERIFIED at R4 against the stub construction. R5 has the REAL production encoder for the FIRST time. For EVERY golden-hex / frozen-byte test you un-ignore: RECOMPUTE the golden via the REAL encoder and CONFIRM it byte-matches the frozen literal. If it DIFFERS, the stub may have frozen a wrong byte — UPDATE the golden to the real-encoder output AND FLAG-FOR-BEN in your report ("golden NAME: stub=… real=…; updated") so the orchestrator hand-verifies the wire-freeze. NEVER weaken the assertion to pass; reconcile to the real bytes. (This is the freeze-byte gate that caught the codepoint + body_cid issues by hand at R4.)

## §3.5g CROSS-LANGUAGE ERRORCODE MIRROR
If your wave MINTS an ErrorCode / any error variant crossing the public/napi/wire surface: atomically update ALL of (Rust variant + \`packages/engine/src/errors.generated.ts\` + \`docs/ERROR-CATALOG.md\` + \`CATALOG_VARIANT_COUNT\`) in the SAME commit. Drift between them is a FIX-NEEDED.

${CFG.canon || ''}
`

const chunk = (a, n) => { const o = []; for (let i = 0; i < a.length; i += n) o.push(a.slice(i, i + n)); return o }

// FAIL-SOFT for schema agents: a schema agent can crash ("subagent completed without calling StructuredOutput",
// esp. on a clean/simple result) — that killed a CONVERGED R4.6 run this session. softSchema catches it + fails to
// the SAFE side (a fallback verdict that SURFACES for the orchestrator), NEVER letting one crash kill the whole
// (expensive) R5 run. Use for every {schema} agent here.
async function softSchema(prompt, opts, fallback) {
  try { const r = await agent(prompt, opts); return r || { ...fallback, _empty: true } }
  catch (e) { log(`schema-agent ${opts.label} CRASHED (${String(e && e.message || e).slice(0, 90)}) — failing soft (orchestrator must review)`); return { ...fallback, _schemaCrashed: true } }
}

const GATE_SCHEMA = { type: 'object', additionalProperties: false, required: ['gate', 'reasoning'], properties: {
  gate: { type: 'string', enum: ['PASS', 'FIX-NEEDED'] }, branch: { type: 'string' }, sha: { type: 'string' }, reasoning: { type: 'string' } } }
const REVIEW_SCHEMA = { type: 'object', additionalProperties: false, required: ['verdict', 'reasoning'], properties: {
  verdict: { type: 'string', enum: ['APPROVE', 'FIX-NEEDED'] }, substantive_ok: { type: 'boolean' }, all_green: { type: 'boolean' }, reasoning: { type: 'string' } } }

// ===== CANARY (sole upstream; gates the fan-out) =====
phase('Canary')
const canaryImpl = await agent(`${COMMON}\n## CANARY WAVE: ${CANARY.key}\n${CANARY.brief}\nYou MINT: ${CANARY.mints}. Implement it, un-ignore the canary tests, loop impl->nextest->fix until GREEN (≤${MAX_FIX} fix rounds), commit to \`${CANARY.branch}\`. Report branch + landing SHA.`,
  { label: `r5-canary:${CANARY.key}`, phase: 'Canary', isolation: 'worktree' })
phase('CanaryGate')
const canaryGate = await softSchema(`${COMMON}\nAdversarially mini-review the canary \`${CANARY.key}\` (branch \`${CANARY.branch}\`; read via git show). Verify: the minted surface matches the spec; its un-ignored tests are GREEN + substantive (would-FAIL-on-revert); no reserved-kw; commit landed. Emit GATE: PASS only if the fan-out can safely build on this surface; else FIX-NEEDED with the minimal fix.\nCANARY REPORT: ${canaryImpl}`,
  { label: 'r5-canary-gate', phase: 'CanaryGate', schema: GATE_SCHEMA },
  { gate: 'FIX-NEEDED', reasoning: 'canary-gate schema-agent crashed — HALT: orchestrator must hand-review the canary before any fan-out (fail-soft to the safe side, never auto-PASS).' })

if (canaryGate?.gate !== 'PASS') {
  log(`CANARY GATE: FIX-NEEDED — halting before fan-out. ${canaryGate?.reasoning || ''}`)
  return { converged: false, stage: 'canary', canaryGate, note: 'orchestrator-led canary fix-pass + re-run' }
}
log(`CANARY GATE: PASS — fanning out ${WAVES.length} waves (rebase byte-pinning families onto the canary SHA; M-20)`)

// ===== FAN-OUT WAVES (≤7 per batch) — each loops impl->un-ignore->test->fix until green, then mini-review =====
phase('Waves')
const waveResults = []
for (const batch of chunk(WAVES, 7)) {
  const res = await pipeline(batch,
    w => agent(`${COMMON}\n## IMPL WAVE: ${w.key}\n${w.brief}\nUn-ignore targets: ${w.files}. Rebase any byte-pinning family onto the canary landing SHA first (M-20). Implement, un-ignore, loop impl->nextest->fix until GREEN (≤${MAX_FIX} rounds), commit to \`${w.branch}\`. Report branch + SHA + which tests un-ignored-green.`,
      { label: `r5:${w.key}`, phase: 'Waves', isolation: 'worktree' }),
    (impl, w) => softSchema(`${COMMON}\nAdversarially mini-review impl wave \`${w.key}\` (branch \`${w.branch}\`; git show). Verify: all assigned tests un-ignored + GREEN + substantive (real entry + observable + would-FAIL-on-revert; NOT weakened to pass); M-20 goldens reconciled vs the real encoder; §3.5h scoped clippy/fmt clean; no reserved-kw; commit landed. APPROVE or FIX-NEEDED.\nWAVE REPORT: ${impl}`,
      { label: `r5-review:${w.key}`, phase: 'MiniReview', schema: REVIEW_SCHEMA },
      { verdict: 'FIX-NEEDED', reasoning: 'review schema-agent crashed — orchestrator must hand-review this wave (fail-soft).' }).then(v => ({ wave: w.key, branch: w.branch, impl, review: v })))
  waveResults.push(...res.filter(Boolean))
}
const approved = waveResults.filter(w => w.review?.verdict === 'APPROVE')
const needFix = waveResults.filter(w => w.review?.verdict !== 'APPROVE')
log(`waves: ${approved.length}/${WAVES.length} APPROVE; ${needFix.length} FIX-NEEDED`)
if (needFix.length) {
  log(`HALT before integrate — ${needFix.length} wave(s) FIX-NEEDED. Do NOT integrate a PARTIAL set (a half-implemented corpus is worse than none). Orchestrator: fix-pass the flagged waves + re-run.`)
  return { converged: false, stage: 'waves', approvedWaves: approved.map(w => w.wave), needFix: needFix.map(w => ({ wave: w.wave, branch: w.branch, review: w.review })), note: 'orchestrator-led wave fix-pass + re-run before integrate (partial-integrate suppressed)' }
}

// ===== INTEGRATE (strategy-C) + FULL-SUITE — only reached when ALL waves APPROVE =====
phase('Integrate')
const integ = await agent(`${COMMON}\nIntegrator: strategy-C consolidate the canary \`${CANARY.branch}\` + the ${approved.length} approved wave branches (${JSON.stringify(approved.map(w => w.branch))}) onto an R5 integration branch off \`${CFG.baseBranch}\` (sequential, upstream/canary-first; resolve disjoint-file unions). Commit. Report the integration SHA + any conflicts.`,
  { label: 'r5-integrate', phase: 'Integrate' })
phase('FullSuite')
const suite = await agent(`${COMMON}\nRun the per-crate test suites (NOT workspace) on the R5 integration branch: ${JSON.stringify(CFG.crates || [])} via \`cargo nextest run -p <crate> <features>\` (bash; report GREEN/RED by failure-count, not masked EXIT). Confirm ZERO #[ignore]'d RED-PHASE pins remain for the implemented families (every one un-ignored + green). Report pass/fail per crate + any residual ignored pins.\nINTEGRATOR: ${integ}`,
  { label: 'r5-full-suite', phase: 'FullSuite' })

return { converged: canaryGate.gate === 'PASS' && approved.length === WAVES.length, canaryGate, waves: waveResults.map(w => ({ wave: w.wave, verdict: w.review?.verdict })), integ, suite }
