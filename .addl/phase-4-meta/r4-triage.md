# F-full R4 test-review — triage (NOT-CONVERGED)

Panel: 15/15 · adversarial-verify: 22 CONFIRMED / 9 PARTIAL / 1 REFUTED

## Converge call

I'll verify panel integrity and the BLOCKER/MAJOR findings before issuing a convergence call.

**PANEL INTEGRITY CHECK:** The brief states 15 of 15 lenses returned, and the inputs contain a full 56-finding structured corpus plus adversarial-verify verdicts plus a completeness-critic gap-hunt. The lens-attribution strings (R1–R15) confirm all 15 lenses are represented across findings. Panel is complete — I may certify if findings clear.

Now let me assess the CONFIRMED BLOCKER/MAJOR set. The decision rule: convergence requires zero CONFIRMED BLOCKER and zero CONFIRMED MAJOR. Let me tally the adversarial verdicts.

**CONFIRMED BLOCKERs (from adversarial verify):** F4-001, F4-002, F4-003, F4-004, F4-005, F4-006 — all six verdicts = CONFIRMED at BLOCKER. Plus the completeness critic raises a fresh BLOCKER (Drop-no-K_Set property, no family at all).

That alone forecloses convergence. But I must still produce the full triage so R4-fix and R5-fill carry-lists are actionable. Let me work through every finding, honoring the adversarial verdicts (which override the original severity where they correct it) and the HARD RULE 12 disposition discipline.

---

# F-FULL R4 CONVERGENCE CONSOLIDATION

## 1. CONVERGENCE CALL: **NOT-CONVERGED**

Panel integrity: **15/15 lenses returned** (verified — all R1–R15 lens-attributions present; structured corpus + adversarial-verify verdicts + completeness-critic gap-hunt all delivered). Panel is NOT incomplete.

Blocking set after adversarial verification:
- **7 CONFIRMED BLOCKERs** — F4-001, F4-002, F4-003, F4-004, F4-005, F4-006 (all six BLOCKER findings upheld at BLOCKER on blind re-verify), **plus** the completeness critic's fresh **Drop-no-K_Set BLOCKER** (no family exists anywhere for the central confidentiality invariant of a frozen primitive).
- **17 CONFIRMED MAJORs** — F4-007, F4-009, F4-010, F4-011, F4-012, F4-014, F4-015, F4-017, F4-019, F4-020, F4-024, F4-025, F4-026, F4-028, F4-029, F4-031(→OBS, see below), plus completeness-critic MAJORs (F-SM-2 no-encryption arm, F-CP-2 hand-list scanner, F-LD-6 NQ-T2-pinned-open-question, and the re-confirmed F-INV16-1 U3 / F-AAD-2 sort-order which overlap structured findings).

Convergence requires zero CONFIRMED BLOCKER **and** zero CONFIRMED MAJOR. We have 8 blockers and ~16 majors after de-dup. **The corpus needs an R3.2/R4-fix pass before it can advance to R5.**

The good news the panel itself surfaced (F4-053, F4-056): the defects are **localized, surgical, single-arm rewrites within otherwise-sound files** — not a re-architecture. The team demonstrably knows the substantive bar (the W0 canary header is the standard). This is a fix-pass, not a redo.

---

## 2. TRIAGE TABLE

Severity column = final severity after adversarial verification (verdict-corrected where the verifier downgraded). REFUTED findings dropped from blocking but retained with disposition. Every row carries a HARD-RULE-12-valid disposition.

| ID | Family | Final Sev | Verdict | Disposition |
|----|--------|-----------|---------|-------------|
| F4-001 | F-AAD-1 self-referential hex-pin | **BLOCKER** | CONFIRMED | **fix-now-at-R4** |
| F4-002 | F-INV16-1 U3 never-collides | **BLOCKER** | CONFIRMED | **fix-now-at-R4** |
| F4-003 | F-LC-2/9 0x6520 plaintext sender vs Sealed-Sender ruling | **BLOCKER** | CONFIRMED | **fix-now-at-R4** + surface to Ben |
| F4-004 | canonical_bytes LE-vs-BE contradictory pin | **BLOCKER** | CONFIRMED | **fix-now-at-R4** |
| F4-005 | F-W0-3 M-19 site-list under-enumeration (R0.3 + test) | **BLOCKER** | CONFIRMED | **fix-now-at-R4** (widen R0.3 §4.1 M-19 list) |
| F4-006 | F-AAD-1 vs F-MS-3 byte-incompatible MemberEntry | **BLOCKER** | CONFIRMED | **fix-now-at-R4** |
| **CC-BLK** | **Drop-no-K_Set — NO family exists** | **BLOCKER** | CONFIRMED (critic) | **R3.2 mint** (F-DROP-NO-KSET) |
| F4-007 | F-AAD-1/2 no absolute golden vector | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-008 | F-INV16-1 U1 passes-green-vs-stub | MINOR | PARTIAL (→MINOR) | **fix-now-at-R4** (hygiene) |
| F4-009 | F-W0-1/4 green-when-intended-RED arms | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-010 | F-CRDT-3 fork-identity tautology | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-011 | F-INV21-4 fixture-readback + K(V) self-equality | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-012 | F-AAD-2 pre-sorted-input sort arm | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-013 | F-GOSSIP-1 convergence-independence tautology | MINOR | PARTIAL (→MINOR) | **fix-now-at-R4** (sibling stand-in exists) |
| F4-014 | F-CRATE-2 comment-not-edge dep pin | MAJOR | CONFIRMED | **fix-now-at-R4** (parse `[dependencies]`) |
| F4-015 | F-LD-2 dropped scope/device_did + concat-vs-CBOR | MAJOR | CONFIRMED | **fix-now-at-R4** + reconcile encoding |
| F4-016 | F-LD W3 no-RED-state / f_ld_8 field-name list | MINOR | PARTIAL (→MINOR) | **R5-fill-named** (f_ld_8 serde-introspection) |
| F4-017 | F-SM-1 passes-green-vs-LIVE (no RED arm) | MAJOR | CONFIRMED | **fix-now-at-R4** (add would-FAIL arm OR re-scope as regression-guard) |
| F4-018 | F-SM-3 strip-resistance vs no-op stub | MINOR | PARTIAL (→MINOR) | **R5-fill-named** (rewire to real seal/open; tf2_f2 already covers live) |
| F4-019 | F-VA-1 invented 0x6101 codepoint | MAJOR | CONFIRMED | **fix-now-at-R4** + surface §4.0 under-spec to Ben |
| F4-020 | F-MS-7 Admin set drops role-change | MAJOR | CONFIRMED | **fix-now-at-R4** + surface Admin-set ruling to Ben |
| F4-021 | F-CRDT-1 with_cases(10_000) | MINOR | PARTIAL (→MINOR) | **fix-now-at-R4** (drop literal / nextest override) |
| F4-022 | F-MST-3 duplicate-args revocation order | MINOR | PARTIAL (→MINOR) | **R5-fill-named** (permute arrival order vs real drain path) |
| F4-023 | F-CRDT-1/2 associativity "gap" | OBS | **REFUTED** | non-blocking (F-INV21-3 owns associativity; F-INV21-2 owns same-HLC tie) — optional doc cross-ref |
| F4-024 | F-GOSSIP-2 topic no byte-layout freeze | MAJOR | CONFIRMED | **fix-now-at-R4** (hex golden-vector + BE/LE differentiator) |
| F4-025 | F-INV21-2 transitivity untested | MAJOR | CONFIRMED | **fix-now-at-R4** (3-fork transitivity arm) |
| F4-026 | F-AAD-2 aad_version unpinned + CBOR/TLV + false len-prop | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-027 | EncryptedEnvelope shape divergence across waves | OBS | PARTIAL (→OBS) | **R5-fill-named** (converge to R0.3 §4.1 shape; hermetic stubs, no R3 compile issue) |
| F4-028 | F-LC-8 token-binding AAD no byte-pin | MAJOR | CONFIRMED | **fix-now-at-R4** |
| F4-029 | F-INV18-1 residual-metadata only doc-grep | MAJOR | CONFIRMED | **fix-now-at-R4** (positive field-set enumeration) |
| F4-030 | bounded-decode / META #629 class | MAJOR | PARTIAL (→MAJOR) | **R5-fill-named** (no frozen byte at stake; add decode-side family) |
| F4-031 | F-MS-8 E_ROLE_STALE_AT_VERIFY TS-mirror | OBS | PARTIAL (→OBS) | **R5-fill-named** (§3.5g discharge at R5 mint + sweep sibling error codes) |
| F4-032 | F-MST-1 O(log n) vacuous-vs-stub | MAJOR | CONFIRMED | **R5-fill-named** (re-validate vs real benten_sync::mst) |
| F4-033 | F-MS-9 exp-bound tautology | MINOR | (struct) | **fix-now-at-R4** |
| F4-034 | F-LD-6 len()==6 self-coverage tautology | MINOR | (struct) | **fix-now-at-R4** (enumerate-and-invoke) |
| F4-035 | F-W0-1 legacy_hkdf_combine scrambled args | MINOR | (struct) | **R5-fill-named** (wiring note) |
| F4-036 | F-KAT-4 pin-c classification | MINOR | (struct) | **fix-now-at-R4** (label as regression-guard) |
| F4-037 | F-VA-4 Debug-leak all-same-byte fixture | MINOR | (struct) | **fix-now-at-R4** (distinct-byte fixture; drop const conjunct) |
| F4-038 | F-VA-1(d) vault no field-order hex-pin | MINOR | (struct) | **fix-now-at-R4** (golden hex, mirrors F-AAD-1 fix) |
| F4-039 | F-HLC-1 lex-compare under-constrained | MINOR | (struct) | **fix-now-at-R4** (tie sub-case) |
| F4-040 | F-CP-2 no injection arm + incomplete list | MINOR/MAJOR | (struct/critic) | **fix-now-at-R4** (injection arm + full §4.0 table; list-derivation R5-fill) |
| F4-041 | F-CP-1 registry completeness | MINOR | (struct) | **fix-now-at-R4** (assert every new const) |
| F4-042 | F-NAT-1 3rd divergent MemberEntry shape | MINOR | (struct) | **fix-now-at-R4** (align to R0.3 5-field) |
| F4-043 | F-DISC-1 substring disposition-class match | MINOR | (struct) | **fix-now-at-R4** (word-boundary match) |
| F4-044 | F-FED-1 intra-path cycle branch | MINOR | (struct) | **fix-now-at-R4** ([A,B,A] arm) |
| F4-045 | F-CRDT stub 2-field Hlc divergence | MINOR | (struct) | **fix-now-at-R4** (align to 3-field) |
| F4-046 | F-AAD-2 arm5 over-ignored compile-fence | MINOR | (struct) | **fix-now-at-R4** (un-ignore now) |
| F4-047 | F-KAT synthesized-witness external-corpus | OBS | (struct) | **R5-fill-named** (hard un-ignore gate: real NIST corpus) |
| F4-048 | F-FED-1 BE-claim vacuous on [u8;4] | OBS | (struct) | **R5-fill-named** (BE arm when real types land) |
| F4-049 | Inv-16..22 correct-by-construction arms | OBS | (struct) | **R5-fill-named** (verify-fails-vs-wrong-impl notes) |
| F4-050 | F-LD-4/F-NAT-2 stub weaker than real | OBS | (struct) | **R5-fill-named** (AEAD-auth-failure asserts at R5) |
| F4-051 | F-MST-2/F-HLC-2/F-LB-1 substrate wiring | OBS | (struct) | **R5-fill-named** (CID framing, no-clock-mutation, LE→BE codepoint) |
| F4-052 | M-20 rebase-onto-canary-SHA | OBS | (struct) | **R5-fill-named** (confirm rebase mandate in briefs) |
| F4-053 | corpus-wide positive baseline | OBS | (struct) | **out-of-scope** (no action; quality baseline) |
| F4-054 | Inv-20 clause b/e/g mapping spot-check | OBS | (struct) | **out-of-scope** (R4 spot-check vs R0.3 §5.1; lens disagrees w/ escalation) |
| F4-055 | F-MS-1/4 variant-count RED-phase shape | OBS | (struct) | **out-of-scope** (correct golden-vector shape; R5 reviewer note) |
| F4-056 | CI-realism / kani / audit-dedup baseline | OBS | (struct) | **out-of-scope** (no action; CI baseline) |
| CC-MAJ-SM2 | F-SM-2 no-encryption arm emits plaintext (no pin) | MAJOR | CONFIRMED (critic) | **R3.2 add** (no-encryption-arm pin) |

---

## 3. R4-FIX WORK-LIST vs R5-FILL CARRY-LIST

### A. R4-FIX (MUST change in the corpus / R0.3 before R5 dispatch)

**Freeze-correctness BLOCKERs (the must-not-freeze-wrong cluster):**
1. **F4-001 + F4-007 + F4-038** — Freeze a real hand-transcribed canonical-DAG-CBOR **golden hex literal** for the members_table AND the vault payload field-order. Replace `expected_fixture_hex()` self-derivation with a frozen string literal. (Same fix-shape across F-AAD-1, F-VA-1(d).)
2. **F4-002** — Reconstruct the U3 collision pair so the two field-splits yield **byte-identical naive concatenations** (equal total length) → RED at baseline, GREEN only against a real length-prefixed injective encoder.
3. **F4-003** — Resolve 0x6520 Sealed-Sender contradiction: update R0.3 §3.3 to the F-LC-9 ruling (per-stanza inner-sender-DID, group sends honor Sealed-Sender), fix F-LC-2's stanza shape, and add a 0x6520 sealed-group wire-scan. **SURFACE TO BEN** (it's a Ben ruling, BR-1, not yet in the plan body).
4. **F4-004** — Delete or BE-migrate `aad_per_chunk_canonical_layout_pinned` (the live on-main LE pin) in the same Wave-0/M-19 step that flips the production encoder, so only one canonical BE pin survives.
5. **F4-005** — **Widen R0.3 §4.1 M-19 site-list** from {aead.rs:165,244,277 + aead_wrap + platform-foundation} to the full 13 sites (add structural_kdf.rs:157, varsig.rs:47/107, sizes.rs:183, swap_matrix.rs:1539/1540/1548/1550). This is a **plan-doc edit**, not just a test edit — freeze-gating keying/wire paths.
6. **F4-006 + F4-042 + F4-045** — Reconcile to **ONE canonical MemberEntry** (R0.3 §3.5 structured 5-field shape) across F-AAD-1 / F-MS-3 / F-NAT-1; align all stub Hlc to the 3-field shape.
7. **CC-BLK (Drop-no-K_Set)** — **R3.2 mint** F-DROP-NO-KSET: seal a Drop over a member subtree, byte-scan the serialized `DropBundlePayload` for the K_Set sentinel → assert absent.

**Substantive-pin / falsifiability MAJORs:**
8. F4-009 — Strengthen F-W0-1 classical-consistency + F-W0-4 lift arms (no enum-literal-only pins on the SOLE upstream canary).
9. F4-010 — Re-route F-CRDT-3 fork-identity through the existing `fork_winner`/`total_order_key` stand-in.
10. F4-011 — F-INV21-4 no-absorption arm must assert on merge OUTPUT; K(V) arm must derive from a real K(V) fn.
11. F4-012 — F-AAD-2 sort arm: delete in-body `.sort()`, hand the assembler an unsorted list.
12. F4-014 — F-CRATE-2: parse `[dependencies]` not whole-file `.contains()`.
13. F4-015 — F-LD-2: restore `scope`/`requesting_device_did`/`reason`/`ephemeral_signing_key`; reconcile concat-vs-CBOR encoding.
14. F4-017 — F-SM-1: add a would-FAIL arm OR re-scope as already-LIVE regression-guard (un-ignore + move out of RED corpus).
15. F4-019 — F-VA-1: drop invented 0x6101, re-pin 12-byte sibling at EnvelopePayload-variant level under 0x6100. **SURFACE §4.0 under-spec to Ben.**
16. F4-020 — F-MS-7: restore `role-change` to Admin set. **SURFACE Admin-set ruling to Ben** (M-11 vs role-change question).
17. F4-024 — F-GOSSIP-2: hex golden-vector pinning `truncate_32(HMAC(K_Set, set_id || BE(generation)))` + explicit BE/LE differentiator + non-vacuous no-time arm.
18. F4-025 — F-INV21-2: add 3-fork transitivity arm.
19. F4-026 — F-AAD-2: add `aad_version` pin; reconcile CBOR-vs-TLV to one contract per R0.3 §4.1 across F-AAD-1/2, F-INV16-1, W0; drop the provably-false `prop_assert_eq!(len,len)`.
20. F4-028 — F-LC-8: add token-binding-AAD mutate→fail-admit arm with BE byte-layout pin.
21. F4-029 — F-INV18-1: add positive field-set enumeration arm (serialized 0x6510 AAD == exactly {audience, coarse_epoch}).
22. F4-040 + CC-MAJ — F-CP-2: add injection negative arm (push duplicate → scanner fires) + expand assigned set to full §4.0 table.
23. **CC-MAJ-SM2** — **R3.2 add** no-encryption-arm pin (selecting 0x0000 emits plaintext verbatim).

**MINOR fix-now cleanups (cheap, fold into the same pass):** F4-008, F4-013, F4-021, F4-033, F4-034, F4-036, F4-037, F4-039, F4-041, F4-043, F4-044, F4-046.

**Open-question audit (CC-MAJ F-LD-6):** Re-audit F-LD-6 class-3 (NQ-T2) + F-LD-3/NQ-T3 + F-LD-5/NQ-T4 + F-LD-8/NQ-C5 for the "pinned-a-premature-answer-to-an-open-NQ" pattern; downgrade to shape-only placeholders until R2/Ben ratifies, OR confirm ratification. **SURFACE NQ-T2 status to Ben.**

### B. R5-FILL CARRY-LIST (named items for the R5 brief — do NOT block R4 convergence)

- **F4-016** — f_ld_8: replace `wire_field_names()` string list with serde-introspection/golden-CBOR against real DropToRecipient struct at un-ignore.
- **F4-018** — F-SM-3: rewire to real `sign_and_seal`/`open_and_verify` (file header already instructs); tf2_f2 already provides live coverage.
- **F4-022** — F-MST-3: permute arrival order against real `mst_revocation_priority.rs` drain path.
- **F4-027** — Converge all stub EncryptedEnvelope/BindingContext shapes to R0.3 §4.1 EnvelopePayload+BindingContext (hermetic stubs; no R3 compile issue).
- **F4-030** — Add decode-side bounded-decode family (META #629 class) on F-INV16-1/F-LC-2/F-LD-2; hostile length-prefix → typed-reject pre-allocation. (No frozen byte at stake.)
- **F4-031** — Discharge §3.5g for E_ROLE_STALE_AT_VERIFY at the R5 mint (Rust variant + errors.generated.ts + ERROR-CATALOG.md + CATALOG_VARIANT_COUNT bump); sweep EngineLocked/LayerCError/AdmitError/E_ENGINE_LOCKED.
- **F4-032** — Re-validate F-MST-1 O(log n) bound against real `benten_sync::mst` (drop or keep the vacuous assertion).
- **F4-035** — F-W0-1: derive legacy_hkdf_combine args from same keypair at R5 wiring.
- **F4-040 (list half)** — F-CP-2 scanner must enumerate real minted symbols + full IANA HPKE ranges (not a literal).
- **F4-047** — KAT families: hard un-ignore gate — swap `synthesize()` for real NIST FIPS-203/204 + draft-connolly X-Wing corpora.
- **F4-048, F4-049, F4-050, F4-051, F4-052** — substrate-wiring verify-fails-vs-wrong-impl notes (CID framing, no-clock-mutation, LE→BE codepoint, AEAD-auth-failure asserts, M-20 rebase-onto-canary-SHA mandate per byte-pinning family).

### C. OUT-OF-SCOPE (no action / baseline)
- F4-023 (REFUTED — associativity owned by F-INV21-3, same-HLC tie by F-INV21-2; optional doc cross-ref only), F4-053, F4-054, F4-055, F4-056.

---

## 4. SUMMARY FOR ORCHESTRATOR + BEN

**Call: NOT-CONVERGED.** The full 15-lens R4 panel returned and adversarial verification upheld **7 of 7 BLOCKER findings at BLOCKER**, the completeness critic found an **8th BLOCKER** (the Drop primitive's core "carries no group key" confidentiality invariant has no test at all), and ~16 MAJORs survived. The corpus cannot advance to R5 yet — it needs one R3.2/R4-fix pass.

**What this is NOT:** a re-architecture. The lenses' own positive baseline (F4-053/F4-056) confirms the corpus is broadly sound — the W2 Layer-C families, the LAMPS KAT decision-fork, the real-Keypair signature arms, and the cross-file codepoint/BE consistency are exemplary. The defects are **localized single-arm rewrites** in otherwise-solid files. The dominant failure mode is one the team already knows how to fix: **freeze-gating pins that assert relative structure or self-referential constants instead of absolute frozen bytes** (the flagship members_table/AAD/vault/topic golden-vectors), plus a handful of **tautological negative arms** (fork-identity, U3 collision, pre-sorted sort-arm, comment-vs-dep-edge).

**Four items need a Ben ruling before R5 freezes them** (each is an architectural decision currently being made silently inside a test stub):
1. **0x6520 group send + Sealed-Sender** (F4-003) — R0.3 §3.3 still pins the plaintext-sender shape that contradicts your F-LC-9 ruling. The plan body was never updated.
2. **Admin UCAN ability set** (F4-020) — the test dropped `role-change` and added `moderate_content`; the R0.3 §3.6.B table self-violates M-11 (Moderator ⊄ Admin). Needs the canonical Admin set ruled.
3. **Vault 12-byte AEAD codepoint** (F4-019) — the test invented `0x6101`; R0.3 §4.0 doesn't assign it. Variant-level vs codepoint-level discrimination under 0x6100 needs a call.
4. **NQ-T2 clock-skew resolution** (CC-MAJ F4-LD-6) — a freeze-gating arm pins a specific bucket↔valid_until answer to a still-OPEN question.

**Plan-doc edit required** (not just tests): R0.3 §4.1 M-19 LE→BE migration list is under-enumerated by 4 freeze-gating keying/wire sites (F4-005) — fix the spec before R5 implements the freeze.

**Path forward:** orchestrator-led R3.2/W0 fix-pass (all R4-FIX items above, including the two R3.2 mints F-DROP-NO-KSET + no-encryption-arm) → re-run the affected lenses on the edited corpus to confirm 0 BLOCKER/0 MAJOR per the iterate-to-convergence rule → harvest → R5 canary-first waves carrying the R5-FILL list. R5-FILL items are correctly-shaped red-phase deferrals (real-corpus KAT swap, substrate-wiring verify-notes, bounded-decode family) and do not block. **Surface the 4 rulings to Ben now; hold the R3.2 fix-pass dispatch until the 0x6520/Admin-set/0x6101/NQ-T2 calls land**, since fixing those tests requires knowing the ratified answer.

## Completeness critic

This reveals another gap in F-AAD-2: arm 2 (sort-order canonical) **pre-sorts the reordered input in the test itself** (`reordered.sorted_member_dids.sort()`) before calling the assembler. So it's testing that "two already-sorted equal lists produce equal bytes" — which is trivially true and tests nothing about the assembler's canonicalization. The comment admits "this stub-shim sorts here to model that contract." The real question — does the *assembler* reject or canonicalize an *unsorted* input — is NOT tested. An implementer who forgets to sort inside the assembler would pass this arm. That's a thin spot on a freeze-gating AAD property (a malicious peer could present DIDs in a non-canonical order to produce divergent AAD).

I now have a comprehensive, ground-verified picture. F-FREEZE-1 uses a real source-tree scanner with self-exclusion and live baseline arms (good). F-NAT-1 has real derivation + grep-defense (good).

I have thoroughly examined the corpus. Let me compile my completeness-critic findings. I've verified every finding against actual corpus lines. Here is my final gap-hunt report.

---

**COMPLETENESS CRITIC — F-full R4 fresh gap-hunt**

I read the full R0.3 plan, the 95-family R2 catalog + coverage matrix, the R1 triage, and spot-checked ~20 of the 56 corpus files against the coverage claims (the high-risk freeze-gating cluster + all 5 consolidator-flagged items re-examined blind + targeted greps across the whole corpus for items the matrix marks "covered" or "no family by design"). Findings below; each grounded in a line I read.

**[BLOCKER] coverage | F-LC (Drop primitive) | DEFECT:** The Drop bundle's load-bearing confidentiality property — "a sealed sibling of the graph that **provably carries no `K_Set`**" (R0.3 §3.5 + §2.5 GN-1; `git grep` for `K_Set.*absent`/`no_k_set`/`provably.carries` across `*/tests/f_*.rs` = ZERO hits) — has NO behavioral pin anywhere in the corpus, and no F-ID in the R2 catalog. **WHY:** Drop is one of the two frozen primitives `{MembershipSet, Drop}`; a Drop that accidentally serializes a `K_Set` (or any group-key material) into the bundle leaks the entire group's keys to a one-shot non-member recipient — a total confidentiality break of the central sharing primitive. This is exactly the structural property GN-1 §2.5 asserts "provably," yet nothing tests it. The whole panel covered DUAL-CID, multi-stanza, Sealed-Sender, abuse-tokens, FS-disclosure — but not the no-`K_Set`-in-Drop invariant. **DISPOSITION:** R3.2 mint — add an F-DROP-NO-KSET family (a serialization-scan/grep arm: seal a Drop over a member's subtree, scan ALL bytes of the serialized `DropBundlePayload` for the live `K_Set` sentinel → assert absent; would-FAIL if an impl bundles the set key). Co-locate in `benten-drop/tests/` (W2).

**[MAJOR] crates/benten-crypto-suite/tests/f_inv16_1_envelope_unification.rs:172 | F-INV16-1 (U3) | DEFECT:** The U3 length-injectivity pin (a P0 freeze-gating canary family) **PASSES GREEN against the deliberately-non-injective stub** — the constructed "collision" pair is wrong. `ctx_c` (audience_did `[41 42 00 00 00]` 5 bytes + gen `0x00004344` BE 4 bytes) yields a 9-byte naive concat; `ctx_d` (audience_did `[41 42 00 00]` 4 bytes + gen `0x00434400` BE 4 bytes) yields an 8-byte naive concat. Different lengths ⇒ the stub's `assert_ne!(enc_c, enc_d)` is TRUE even with NO length prefix. **WHY:** U3 (canonical-TLV length-injective) is the flagship truncation/substitution defense for the entire envelope layer (Inv-16); a red-phase test that is GREEN at red-phase pins nothing, and at R5 the impl could ship a non-injective TLV and the test would still pass. This is worse than the consolidator's flag #4 "pass-against-stub" note — the fixture is mathematically incapable of exhibiting the collision it claims. **DISPOSITION:** fix-now-at-R4 — replace the pair with a genuine same-total-length, same-byte boundary-ambiguity collision (e.g. `["AB", 0x00434400]` vs `["AB\x00", 0x434400__]` constructed so the naive concatenations are byte-identical), so the stub's missing length-prefix actually produces equal bytes (RED) and the real TLV produces distinct bytes (GREEN at R5).

**[MAJOR] crates/benten-membership-set/tests/f_aad_2_nine_tuple_injectivity_opaque_boundary.rs:203 | F-AAD-2 | DEFECT:** The `sorted-member-DID-list` canonicalization arm pre-sorts the "reordered" input inside the test (`reordered.sorted_member_dids.sort()`) before calling the assembler, then asserts equality — testing only that "two already-sorted equal lists encode equally," which is trivially true and exercises nothing in the assembler. The actual freeze property — that the assembler itself canonicalizes (or rejects) an *unsorted* DID list — is untested. **WHY:** the `sorted-member-DID-list` is field 3 of the AAD 9-tuple (Inv-20 clause-c); a malicious or buggy peer presenting DIDs in non-canonical order must produce identical AAD or the fork diverges. An R5 impl that forgets the internal sort passes this arm. **DISPOSITION:** fix-now-at-R4 — feed the assembler an UNSORTED list and assert it produces the SAME bytes as the sorted fixture (or a typed reject), with the sort happening INSIDE `assemble_aad_9tuple`, not in the test body.

**[MAJOR] coverage | F-SM-2 (no-encryption swap arm) | DEFECT:** The no-encryption (`0x0000` / `no_encryption_public_class`) swap-matrix arm has NO pin of its security-relevant property — that selecting it actually emits **plaintext** (not silently encrypted, not refused). F-SM-2 explicitly skips its round-trip ("no ciphertext to round-trip; its codepoint pin suffices"; `f_sm_inv17_hybrid_floor_swap_matrix.rs:176`), and `git grep` for `not_encrypted`/`cleartext`/`plaintext.*not.*encrypt` = ZERO. **WHY:** the swap matrix is a §9.1-1 G-CORE-3c exit criterion; the no-encryption arm is a real built path (public-class deployment). An impl that wires no-encryption to a default-encrypt-and-discard-key, or that emits the wrong codepoint, passes every pin — yet the public-class confidentiality posture (data IS readable, by design) is the whole point of the arm. **DISPOSITION:** R3.2 add a no-encryption-arm pin (`seal(no_enc, pt)` → output contains `pt` verbatim / decodes without a key) to F-SM-2 or a sibling.

**[MAJOR] crates/benten-engine/tests/f_ld_6_remote_permission_six_class_mini_review.rs:295 | F-LD-6 class-3 / NQ-T2 | DEFECT:** The clock-skew class-3 arm pins a CONCRETE decoupling behavior (60s `valid_until` window, 90s-late → reject; bucket does not widen it) even though **NQ-T2 is OPEN** (R0.3 §10.5; R2 §5.B item-5 explicitly lists F-LD-6 as "R2-resolution-gated — cannot pin the exact assertion until the NQ resolves"). The docstring even calls it "OPEN-SPEC: gated on NQ-T2," yet the arm asserts a specific answer. **WHY:** the 6-class mini-review IS the §9.1-4 freeze gate; if Ben/R2 rules a different bucket↔`valid_until` relationship, this freeze-gating arm pins the wrong behavior and R5 faithfully implements it. **DISPOSITION:** fix-now-at-R4 OR surface — either confirm NQ-T2's resolution is already ratified (it is NOT, per the plan), or downgrade class-3 to a shape-only placeholder until R2 ratifies, matching the R2 §5.B contract. (Sibling: F-LD-3/NQ-T3, F-LD-5/NQ-T4, F-LD-8/NQ-C5 should be re-audited for the same "pinned-an-open-question" pattern.)

**[MAJOR] crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs:108 | F-CP-2 (NQ-W2 scanner) | DEFECT:** The non-collision / IANA-disjointness scanner — designated a P0 freeze-gating prerequisite — operates over a hand-coded literal list of 20 integers (`all_assigned_envelope_codepoints()`) and a "representative slice" of IANA ranges (`iana_hpke_reserved_ranges()` returns only `0x0001..=0x0003, 0x0010..=0x0021`), NOT an enumeration of the real minted symbols nor the authoritative IANA table. **WHY:** R2 §residual explicitly demands (for the analogous F-DISC-1) that such scanners "enumerate from the [source], not a literal" — F-DISC-1 was fixed to do so (it parses `SECURITY-POSTURE.md`), but F-CP-2 was NOT held to the same bar. A new codepoint minted at R5 outside the hand-list is invisible to the collision scanner; the IANA slice misses most of the HPKE registry, so a future codepoint landing in an un-listed IANA range passes. The whole point of NQ-W2's CI scanner (Inv-18) is auto-inclusion. **DISPOSITION:** fix-now-at-R4 (brief R5) — require the scanner to enumerate the real minted symbol set (a source-scan or a registry iterator) and pin the full IANA HPKE `kem_id`/`kdf_id`/`aead_id` ranges, with a baseline arm proving auto-inclusion (inject a colliding const → fires).

**[MINOR] crates/benten-membership-set/tests/f_inv21_fork_tie_break_totality_version_node_cid.rs:~430 | F-INV21-4 | DEFECT:** The "archived-not-discarded" assertion (`loser.crdt_vector.contains("l-only")`) is a near-tautology — `loser` is the unmodified input `loser_fork`, which always contains its own vector; it proves nothing about a real archive operation. **WHY:** archived-not-discarded (losing-fork retention) is an Inv-21 clause; the substantive sibling (winner does NOT absorb loser-only entries) IS real, but the retention half is hollow. **DISPOSITION:** R5-fill (named) — at R5 the real `resolve_fork` returns a distinct archived handle; assert retention via a query against the archive store, not against the unchanged input. (This is the consolidator's flag-#2 class; substantiated.)

**[MINOR] crates/benten-membership-set/tests/f_crate_1_2_ep1_roster_and_boundary.rs:276 | F-CRATE-2 | DEFECT:** `crate2_b1_dep_set_direction` asserts `cargo.contains("benten-sync")` against the membership crate's Cargo.toml where the B-1 deps are present only as COMMENTS at R3 (`# benten-sync = ...`, un-commented by the canary at R5) — so the pin matches comment text, not a real `[dependencies]` edge, and passes green at R3 verifying nothing. **WHY:** B-1 (the `benten-sync` dependency) was a R1 BLOCKER; a pin that green-passes against a comment can't catch the canary silently dropping the edge. **DISPOSITION:** R5-fill (named) — at R5 parse `[dependencies]` (not raw `.contains`) so the pin tightens once the canary un-comments; flag in the R5 brief. (Consolidator flag #3; substantiated.)

**[OBS] coverage | F-W0-2 / F-KAT-1 / F-KAT-4 / F-NQC4-1 | DEFECT:** The four external-vector families pin against SYNTHESIZED sentinels (e.g. `draft_connolly_x_wing_kat_for_fixture()` returns `[0x11;32]`), not real published corpora; at R5 if the real vectors aren't acquired, the "interop / byte-for-byte against published vectors" claims are vacuous. **WHY:** these are the cross-ecosystem CONFORMANCE guarantees (the justification for X-Wing/LAMPS interop). **DISPOSITION:** out-of-scope-for-R4 (already named in R2 §5-D as a fixture-acquisition seed + real-corpus swap at R5) — noting it for completeness so R4 does not mistake the sentinels for real KATs; the red-phase shape is correct.

**[OBS] coverage | NQ-A2 / §9.1-8 / §9.3-11/12 | DEFECT:** Several matrix cells are "no test by design / process" (assessment-window placement, R6-council, audit deliverables). These are correctly test-free, but no F-DISC-2 arm pins that the EXIT-CRITERIA DOCS name the window (R2 §5.A-2 left this open: "F-DISC-2 could carry a 'exit-criteria docs name the window' arm or is intentionally test-free"). **WHY:** a doc-coupling pin would catch a silent drift of the tag sequence. **DISPOSITION:** out-of-scope (tag-sequencing ratification, intentionally test-free per R2) — flagged so R4/Ben can decide whether F-DISC-2 gains the one-line doc-coupling arm.

**Independent coverage-matrix spot-check result:** The R2 matrix's claims of behavioral coverage hold for the families I checked (federation depth-4/cycle, #46 wire-cost ceilings, ExecuteWorkflow 3-field AAD-binding, multi-device pubkey-substitution-rejects, disable-cache replay-leak control, HLC-skew, F-LC-9 group Sealed-Sender, F-DISC-1 doc-enumeration, F-FREEZE-1 source-scanner) — these are genuine substantive pins. The misses are the **Drop-no-`K_Set` property (no family at all — BLOCKER)** and the **green-at-red-phase / pre-sorted-input / hand-list-scanner / open-question-pinned** weaknesses above (freeze-gating tests that pin nothing or pin a premature answer).

LENS VERDICT: FINDINGS-RAISED — 1 BLOCKER / 5 MAJOR / 2 MINOR / 2 OBS
