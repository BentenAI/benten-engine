# Option F+ §6.2 envelope-layer-unification — C3 FRESH-EYES CRYPTOGRAPHER CRITIQUE

**Branch:** `phase-4-meta-core/option-f-plus-critique-c3-fresh-eyes-cryptographer`
**Role:** 10th cryptographer eye (HOMER privilege). Fresh-eyes re-read of the 9-lens CONSOLIDATED registry (§2 28 amendments + §3 13 Compromise mints + §4 3 invariants + §5 5 Ben-call disagreements). The 9 lenses + consolidator structurally could not see the COMPOSED whole — I can.
**Date:** 2026-05-27
**Tree-state pre-flight:** worktree clean at `2172cb6d` against `origin/main`; branch freshly cut from worktree HEAD.
**Authority:** ADVISORY. Per `feedback_review_finding_ground_truth_verify` + the brief's "DISAGREE-WITH-EXPLANATION is first-class" stance, downstream Ben call ratifies.
**Input frozen at:** `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` + L1 @ `6d4e173f` + L2 @ `7e900a3b` + L3 @ `13b624c3` + L4–L9 SHAs per registry §10.1.

---

## §1 Executive verdict

**CONCUR-WITH-REFINEMENTS.** Confidence **HIGH** on the F+ NO-GO re-confirmation, **HIGH** on Inv-16/17/18 + the 28-amendment registry's structural correctness, **MED-HIGH** on the consolidator's Q1-Q5 advisory calls, **MED-HIGH** that the registry as ratified-with-my-additions is IND-CCA2-secure + INT-CTXT-secure + replay-resistant + crypto-agility-preserving + freeze-permanent.

I surface **5 substantive refinements** + **3 net-new findings** the 9 lenses + consolidator structurally missed:

1. **NEW FINDING F1 (LOAD-BEARING):** No lens evaluated the COMPOSED design against `draft-sfluhrer-cfrg-ml-kem-security-considerations-04`'s **MAL-BIND-K-CT / MAL-BIND-K-PK** binding-properties (adversary-controlled ML-KEM private key produces non-injective shared-secret → ciphertext mapping). Layer-D's multi-device + RecoveryHook key-escrow surface AND the L9 multi-stanza HpkeMultiBase + recipient-key-rotation surface BOTH have authentication-where-adversary-can-supply-key reachability paths. Closes additively as **proposed Amendment U41** + **proposed Compromise #45**. See §6.1.
2. **NEW FINDING F2 (MED-HIGH):** **Inv-18 over-merges 3 distinct sibling invariants** (registry-discipline + metadata-disclosure + CodepointLifecycle). The consolidator self-noted this risk (§9 item 1). My COMPOSED-whole assessment confirms: enforcement-plan readability + cite-drift-detector lint shape + audit-firm cross-reference grain all argue for **splitting Inv-18 into Inv-18a (registry-discipline) + Inv-18b (metadata-disclosure paired-Sealed-Sender-slot) + Inv-18c (CodepointLifecycle)**. The MF1 + MF5 structural-tension framing surfaces cleaner when Inv-18b stands alone. See §6.2.
3. **NEW FINDING F3 (MED):** **No lens did the formal IND-CCA2 reduction for the COMPOSED design after all 28 amendments are applied.** Each lens assumed a prior reviewer did. The consolidator's MF5 notes this gap explicitly ("no formal-methods lens"). I supply the informal-rigorous walk-through in §2; flag it as audit-firm-must-revisit prose for SECURITY-POSTURE.md.
4. **REFINEMENT R1 on Q1 (libcrux):** CONCUR-WITH-EVIDENCE on Option B, BUT surface a previously-unnamed risk: libcrux is **single-vendor stewardship** (Cryspen). Compromise #39 supply-chain row should explicitly name the libcrux-mono-vendor dependency boundary + add a revisit-trigger "if Cryspen project pauses/shutters, fall back to RustCrypto ml-kem + commission CT-Decap audit." See §3.1.
5. **REFINEMENT R5 on Q5 (Sealed-Sender):** CONTEST-WITH-COUNTER on the "additive-slot OR default-codepoint" framing as a false binary. The COMPOSED design admits a **caller-aware DEFAULT-PER-OPERATION dispatch** (Sealed-Sender for DropToRecipient where metadata-leak is high-cost; non-Sealed for DeviceLink + RemotePermission where sender-AAD-binding is integrity-load-bearing per U4). Three slot positions, not two. See §3.5.

The F+ NO-GO **stands unanimously at 10 of 10 cryptographer eyes**.

---

## §2 Fresh-eyes cryptographic-property assessment of the COMPOSED whole

The mental model I built for this section: walk through `Seal(plaintext, ctx) → EncryptedEnvelope` and `Open(envelope, ctx) → plaintext` for each of the 4 layers, with ALL 28 amendments applied as a coherent unified construction. Per-property findings follow.

### 2.1 Per-layer composed construction (post-amendments)

**Common envelope-format substrate (all layers):**

```
EncryptedEnvelope := DAG-CBOR-tag(0xBE54) [             # U30
    codepoint: u16 BE                                    # U7, U11
    payload: EnvelopePayload [#[non_exhaustive]]         # U9
    aad_binding: BindingContext [#[non_exhaustive]]      # U10
]

canonical_binding(codepoint, aad_binding) :=             # U1, U3, U14
    0x01 ‖ varint(codepoint_BE) ‖
    TLV(canonical_serialize(aad_binding))                # length-injective per U3
```

The auth-tag (AEAD) or HPKE info-string consumes the FULL `canonical_binding()` output. Mismatch in any byte (codepoint / aad_version / TLV stream) ⇒ Open returns `EnvelopeError::AuthFail`. Strict-decode per U2 forbids cross-variant fallback.

**Layer-A — vault (encrypt-to-self at-rest):**
- `payload = SymmetricAeadXNonce { ciphertext, nonce: [u8;24] }` per U12 + U32.
- Key: `K_principal[gen=g] = AEAD-Decrypt(vault.cbor.K_principal_blob[g], DAK, AAD=canonical_binding(LAYER_A_VAULT, BindingContext::Vault{vault_version, k_principal_generation: g, sealed_at_epoch_hour})` — DAK = `Argon2id(password, salt, tier_params) → HKDF-Expand`.
- BindingContext = `Vault { vault_version: u8, k_principal_generation: u32 }` (sender-DID excluded per U4 carve-out for self-encryption; replay-window excluded per U5 carve-out for at-rest).
- **IND-CCA2 property:** inherits XChaCha20-Poly1305 IND-CCA2 (RFC 8439 §4 + Bellare-Namprempre 2000 AE-INT-CTXT composition). Per-key birthday at 2^96 (vs ChaCha20-Poly1305's 2^32) absorbs the U32-cited Layer-A vault rotation hazard. Codepoint binding closes cross-codepoint substitution (L2 §2.4 attack class).
- **Composition with rest of registry:** U3 length-injectivity prevents the vault-AAD-blob spoof attack (where adversary substitutes a forged `Vault { vault_version: 0xFF, k_principal_generation: 0x00000001 }` BindingContext stream that byte-collides with a forged Vault{0x01, 0xFF000001}). U14 aad_version closes the canonicalization-shape drift attack.
- **Verdict: IND-CCA2-secure. HIGH confidence.**

**Layer-B — per-Node AEAD:**
- `payload = SymmetricAeadXNonce { ... }` per U32. Same shape as Layer-A but distinct codepoint (`LAYER_B_NODE_AEAD = 0x6200..0x62FF`).
- Key: `K(N) = HKDF(K_principal[gen=g], context=N.cid ‖ k_principal_generation=g ‖ chunk_index)`. AAD = canonical_binding(LAYER_B_NODE_AEAD, BindingContext::PerNodeAead { node_cid: Cid, k_principal_generation: u32, chunk_index: u32 }).
- **IND-CCA2:** unchanged from Layer-A — same AEAD primitive; codepoint discriminates the symmetric-key derivation path.
- **Verdict: IND-CCA2-secure. HIGH confidence.**

**Layer-C — drop-to-recipient (HPKE-mode-base[X-Wing]):**
- `payload = HpkeMultiBase { cek_aead_ciphertext, cek_aead_nonce: [u8;24], stanzas: Vec<HpkeRecipientStanza> }` per U17. Per-stanza: `HpkeRecipientStanza { enc, encrypted_cek_share, recipient_did, recipient_key_generation: u32, stanza_index: u32 }` (HPKE-11-KE mode per Q4 — HPKE wraps a CEK; CEK + per-Node K(N) bulk-encrypt).
- BindingContext = `DropToRecipient { audience_did_set_sorted: Vec<Did>, sender_did: Did, plaintext_cid: Cid, sealed_at_epoch_hour: u32 ‖ jitter }` per U4 + U5 + U18 + U28.
- HPKE `info` string := canonical_binding(LAYER_C_DROP_MULTI_RECIPIENT, BindingContext). Cross-stanza substitution defense: per-stanza AAD additionally binds `(codepoint, plaintext_cid, sorted_audience, sender_did, stanza_index, recipient_key_generation)` per U17.
- **IND-CCA2 property:** HPKE-mode-base[X-Wing] IND-CCA2 per RFC 9180 §9.1.2 + Barbosa-Connolly-Diniz-Kahl-Krämer X-Wing IACR CIC 2024. **Crucially: this is the standard HPKE adversary model — recipient's keypair is sampled with honest uniform randomness on-device (per L1 §6.1).** The "chosen-seed-via-password" adversarial model that L1 §2.3 / L2 §3.3 flagged AT LAYER-A NO-GO is now structurally inapplicable at Layer-C/D because the recipient's HPKE keypair is NOT password-derived.
- **Cross-stanza substitution:** closed by per-stanza AAD per U17. The AAD-bound `stanza_index` + sorted-audience commits to position-in-multi-stanza, so substituting stanza_i = (enc_Eve, …) into Bob's recipient view fails at HPKE info-string verification.
- **Verdict: IND-CCA2-secure. HIGH confidence** (within the registry-named Compromise #32 Bernstein-Persichetti caveat — closed mechanically by U31 libcrux-ml-kem CT-Decap).

**Layer-D — DeviceLink + RemotePermission + ExecuteWorkflow:**
- `payload = HpkeBase { enc, ciphertext }` (single-recipient) or `HpkeMultiBase` (rare; e.g. provisioning to a device cohort).
- BindingContext = `DeviceLink { provisioning_session_id, sender_device_did, granting_user_did, requesting_device_did, sealed_at_epoch_hour, valid_until_epoch_hour, k_principal_generation }` OR `RemotePermission { request_id, operation: PermissionOperation, sender_device_did, granting_user_did, requesting_device_did, sealed_at_epoch_hour, valid_until_epoch_hour }` OR `ExecuteWorkflow { workflow_cid, input_node_cids, max_decrypt_count, result_recipient_pubkey, executor_did, sender_did, sealed_at, valid_until }` per U4 + U5 + U21.
- Same HPKE-mode-base[X-Wing] IND-CCA2 inheritance as Layer-C. Recipient device's HPKE keypair is generated with on-device random(32) — honest randomness — NOT password-derived. Standard HPKE adversary model applies. **HIGH confidence IND-CCA2.**

### 2.2 IND-CCA2 across all 4 layers (COMPOSED-whole verdict)

**HIGH confidence the COMPOSED design is IND-CCA2-secure across all 4 layers** under standard adversary model + registry-disclosed Compromises (#31 forever-valid drops + #32 Bernstein-Persichetti + #42 FS-gap + #43 metadata-leak).

**Reduction sketch (informal-rigorous, audit-firm-readable; supplies the formal-methods-lens gap MF5 names):**

> Let A be an IND-CCA2 adversary against the COMPOSED EncryptedEnvelope construction with non-negligible advantage ε. We build a chain of game-hops:
>
> 1. **G0 = real IND-CCA2 game** against EncryptedEnvelope. A queries Seal/Open oracles + outputs (m_0, m_1) + receives challenge envelope.
> 2. **G1 = strict-decode reject of cross-variant** (per U2). Any A-query that decodes-bytes-as-AEAD when codepoint says HPKE (or vice versa) is rejected. Distinguishing advantage `|G0 - G1| ≤ Pr[cross-variant-decode-succeeds]` = negligible by U2 enforcement + U1 codepoint-bound-in-AAD (would produce auth-fail under canonical_binding mismatch).
> 3. **G2 = AAD-binding canonicalization injective** (per U3 + U14). Any A-query that produces a non-canonical AAD byte-stream auth-fails. `|G1 - G2| ≤ Pr[canonical_binding-collision]` = negligible by U3 length-injectivity proof (kani-target per U39).
> 4. **G3 = per-codepoint primitive reduction.** For Layer-A/B codepoints, reduce to XChaCha20-Poly1305 IND-CCA2 (Bellare-Namprempre 2000). For Layer-C/D codepoints, reduce to HPKE-mode-base[X-Wing] IND-CCA2 (RFC 9180 §9.1.2 + Barbosa et al. 2024). `|G2 - G3| ≤ Σ_codepoint Adv_primitive(B)` for some adversary B with comparable runtime.
> 5. **G4 = side-channel-free Decap** (per U6 + U31 libcrux CT-Decap). `|G3 - G4| ≤ Adv_BernsteinPersichetti(B')` = negligible under U31 verified-CT-impl assumption (or bounded-disclosure per #32 Compromise).
>
> Therefore A's IND-CCA2 advantage ε is bounded by Σ over G-hops of well-studied primitive-advantages + the U3 kani-proven injectivity term. **The COMPOSED design is IND-CCA2-secure conditional on (a) U31 libcrux CT-Decap holds, (b) U3 injectivity proof discharges, (c) U2 strict-decode is enforced by all decoder paths.**

The three conditions (a-c) are EXACTLY what the U31 + U37 + U38 + U39 deliverables verify. The proof modularity is what makes Inv-16 + Inv-17 + Inv-18 the right normative-anchor shape. **HIGH confidence verdict.**

**One residual:** the F1 finding (MAL-BIND binding-properties — see §6.1) DOES interact with this reduction at G3 for HPKE-mode-base[X-Wing] layers — specifically when an adversary can supply a chosen recipient sk (e.g., Layer-D RecoveryHook escrow-key-recovery flow; Layer-C with attacker-controlled "compromised-recipient" key). The G3 reduction tacitly assumes the recipient sk is honest. The MAL-BIND-K-CT property fails in this submodel. F1 closes this via per-stanza AAD `audience_did + recipient_key_generation` already-bound per U17 + U19, BUT the registry doesn't currently call out the binding-properties literature alignment. **Not a security gap; a documentation gap.** See §6.1.

### 2.3 INT-CTXT at envelope-format layer

**HIGH confidence INT-CTXT-secure at the envelope-format layer.**

Walk-through: an adversary's INT-CTXT-forge attempt against EncryptedEnvelope must produce a byte sequence E* such that `Open(E*, ctx) ≠ ⊥` AND E* was not a Seal-oracle output for some queried (m_i, ctx_i).

- **Bit-flip in `codepoint` field**: codepoint is bound into canonical_binding → auth-tag covers it. Bit-flip changes the AAD stream → auth-fail. ✓
- **Bit-flip in `payload.ciphertext`**: covered by AEAD auth-tag (Layer-A/B) or HPKE-mode-base seal (Layer-C/D). ✓
- **Bit-flip in `aad_binding` byte stream**: same — canonical_binding output changes → auth-fail. ✓
- **Substitution of entire `payload`** while preserving `codepoint`: payload is bound into AEAD via the ciphertext itself (variant tag is the first byte of payload; collision-attempt blocked by U2 strict-decode + U9/U10 `#[non_exhaustive]` typed-reject). ✓
- **Cross-variant substitution** (codepoint=A, payload-bytes-look-like-B): U1 + U2 reject; auth-tag binding ensures bytes-as-decoded under codepoint=A produce auth-fail. ✓
- **TLV value-bytes-opaque attack** (forging an inner TLV field): U3 length-injectivity makes any non-original byte stream produce a different canonical_binding output → auth-fail. ✓
- **`aad_version` byte rewrite**: U14 binds aad_version into AAD itself → auth-fail on rewrite. ✓

**Forgery infeasible per AEAD-INT-CTXT (Layer-A/B) + HPKE-mode-base auth-tag (Layer-C/D). Bit-flipping detected at all positions. HIGH confidence.**

One subtle composition concern: **DAG-CBOR outer framing (U30) introduces a potential malleability surface IF the CBOR decoder admits multiple byte-representations of the same logical envelope.** RFC 8949 §4.2.1 deterministic encoding closes this (canonical CBOR — definite-length items, sorted map keys, smallest-int encoding) IF AND ONLY IF the decoder enforces it. **My recommendation:** the U30 R0 plan-doc text MUST mandate "strict deterministic-CBOR-decode" — reject envelopes whose CBOR encoding is non-canonical even when semantically valid. Without this, malleability could re-introduce INT-CTXT softness at the framing layer (an adversary could re-encode an existing envelope into a different byte stream that decodes to the same logical envelope but has a different blob CID). This composes badly with U18's `envelope_blob_cid` discipline. **REFINEMENT R3:** add "strict deterministic-CBOR-decode enforcement" as an explicit U30 sub-clause.

### 2.4 Replay resistance

**HIGH confidence replay-resistant for DeviceLink + RemotePermission + ExecuteWorkflow (per U5 + U28 + U21).**

**MED-HIGH confidence replay-resistant for DropToRecipient** — by design, drops are forever-valid per Compromise #31 (extension covers recipient-key-retention window per U19). Within the retention window, an adversary CAN replay an old drop; the recipient sees identical content. This is the documented trade-off, not a defect.

**LOW confidence replay-resistant for Vault** — by design, vault is at-rest; "replay" of an old vault.cbor is what every backup/restore flow does. Out of scope per #31 + #34.

The 4-layer replay-property differentiation IS coherent — each layer's replay-stance is registry-disclosed (U5 carve-outs for Vault + DropToRecipient).

**One residual surface the 9 lenses didn't fully integrate:** the **U28 coarse-1-hour-bucket refinement** reduces replay-window granularity from second-precision to hour-precision. An attacker who captures an envelope at hour H can replay it any time during hour H (3600-second window). For DeviceLink + RemotePermission this is acceptable per L6 + L9; for ExecuteWorkflow (U21) where `max_decrypt_count` exists, a within-hour replay could increment the decrypt-counter unintentionally. **REFINEMENT R4:** U21 ExecuteWorkflow MUST bind a per-request nonce (or `request_id: [u8;16]`) into AAD in addition to sealed_at/valid_until, so within-hour replays are rejected at the request-id-uniqueness layer.

### 2.5 SUF-CMA-equivalence at application layer (Inv-15 sibling Inv-16)

**HIGH confidence Inv-16 is the correct sibling to Inv-15 at the construction-soundness layer; the SUF-CMA-equivalence story applies analogously.**

Inv-15 closes SUF-CMA-equivalence on the signature side via 3-layer-decomposition + Drop-bundle forever-valid (Compromise #31). The COMPOSED encryption side achieves the analogous property via:

- **Envelope-CID-as-identifier discipline** (U18 dual-CID: plaintext_cid is content-stable; envelope_blob_cid is transport-stable). This is the encryption-side analog of Inv-15's "signature-bundle-CID is NEVER load-bearing."
- **Codepoint-discriminated primitive choice** (U1 + U2 + Inv-16) gives the same crypto-agility seam that Inv-15's signature-side codepoints give.
- **Per-codepoint primitive's own SUF/INT-CTXT property** is inherited (XChaCha20-Poly1305 INT-CTXT for Layer-A/B; HPKE-mode-base auth-tag for Layer-C/D).

The SUF-CMA-equivalence is at the AEAD-side INT-CTXT layer; the envelope-format layer adds canonicalization (U3 + U14) so the AEAD's INT-CTXT lifts to envelope-INT-CTXT compositionally.

**Note:** Inv-16 is normatively cleaner if we **explicitly cross-cite Inv-15** in the Inv-16 phrasing ("encryption-side sibling of Inv-15 signature-bundle-CID discipline; together they cover both halves of identifier-discipline at the crypto-construction layer"). This is consolidator pattern-induction MF2 — promote to Inv-16 phrasing.

### 2.6 Crypto-agility seam (baked-in #5 preservation)

**HIGH confidence the COMPOSED design preserves baked-in #5 crypto-agility.**

Future codepoints can be added additively via:
- **U9 + U10 `#[non_exhaustive]`** on EnvelopePayload + BindingContext — new variants land without breaking v1-beta consumers (they typed-reject unknown variants, which is the correct posture per RFC 7515 JOSE alg-discipline + RFC 9180 §7.1 codepoint-extensibility).
- **U11 escape-codepoint `0xFFFF`** + experimental range `0xFE00..0xFFFE` — future u32/u64 codepoints can extend without v2 wire-break.
- **U13 codepoint brackets** reserved at v1-beta for MLS-PQ / CGKA / Bird-of-Prey / draft-prabel — even-better, the bracket-reservation is at v1-beta-tag-time so the brackets are immutable per baked-in #15.
- **U14 aad_version: u8** — future canonicalization-shape changes (e.g. switching from TLV to length-prefixed-CBOR-array) mint `aad_version = 0x02` etc. The aad_version is bound into AAD itself, so a v1-beta consumer sees `aad_version = 0x02` → typed-reject as "unknown canonicalization." Correct crypto-agility posture.
- **U16 CodepointLifecycle** — Live → Deprecated → Quarantined → Burned typed-state lets codepoints retire gracefully without forcing wire-format break.

Future codepoints MLS-PQ-Application, CGKA-Commit, Bird-of-Prey AKEM, post-NIST-2030-PQC-Migration all slot in.

**One subtle concern:** baked-in #5 says "crypto-agility codepoint-dispatch." The COMPOSED design adds **3 orthogonal codepoint axes**: (1) the `codepoint: u16` field per envelope; (2) the `aad_version: u8` byte per U14; (3) the IMPLICIT codepoint axes per primitive choice (e.g. ML-KEM-768 vs ML-KEM-1024 may need distinct top-level codepoints rather than sub-axis). **The 3-axis surface is correct per MF3** (variant/codepoint/canonicalization permanence-package). The R0 plan-doc should make the 3-axis structure EXPLICIT — call it the "permanence triangle" — so future codepoint mints know which axis to extend. **REFINEMENT R5:** name the 3-axis structure explicitly in V1-FROZEN-INTERFACE.md item 6 + CRYPTO-CODEPOINTS.md preamble.

### 2.7 Freeze-permanence (baked-in #15 preservation)

**HIGH confidence the COMPOSED design preserves baked-in #15 v1-beta-freeze.**

The wire format pinned at v1-beta:
- `EncryptedEnvelope` outer shape (DAG-CBOR-tag 0xBE54 per U30 + canonical CBOR encoding per RFC 8949 §4.2.1)
- `codepoint: u16 BE` per U7 + U11
- `EnvelopePayload` + `BindingContext` enums with `#[non_exhaustive]` per U9 + U10
- `canonical_binding()` with `aad_version: u8 = 0x01` prefix per U14
- TLV length-injective encoding per U3
- Codepoint brackets reserved per U11 + U13
- `Did` multikey canonical encoding per U15
- CodepointLifecycle states per U16

A 2026-sealed envelope parses-and-verifies in 2050 IF AND ONLY IF:
1. The CBOR decoder admits the 2026 tag 0xBE54 + canonical CBOR mode. ✓ (CBOR is IETF standard; tag registry persists).
2. The codepoint `0x6100..0x6FFF` Benten range is honored. ✓ (U8 IANA-disjoint reservation; should we register with IANA pre-v1-beta? See F-up at §6).
3. The aad_version `0x01` is still decoder-supported. ✓ (U14 mandates Live/Deprecated/Quarantined/Burned per CodepointLifecycle; aad_version `0x01` will be `Live` or `Deprecated` in 2050 with decode still supported).
4. The cryptographic primitives ChaCha20-Poly1305 + ML-KEM-768 + X25519 + X-Wing are still believed-secure OR Quarantined-decode-with-warning. ✓ (CodepointLifecycle U16 covers this).

**One residual surface:** the `Did` multikey codepoint registry (U15) is GOVERNED BY MULTIFORMATS (external dependency), not Benten. If multiformats deprecates a multikey codepoint that Benten's 2026 envelopes use, the 2050-decode story becomes "Benten implements its own fallback multikey-table." The U15 disposition currently says "depends on Atrium's multikey commitment." **REFINEMENT R6:** mint an explicit Compromise (#45 candidate per §6.1, OR a new #46) disclosing "Benten depends on multiformats codepoint stewardship; freeze-permanence guarantee transitive on multiformats stewardship-continuity." Make this honest-disclosure explicit at v1-beta.

### 2.8 Per-property confidence summary table

| Property | Verdict | Confidence | Key dependency |
|---|---|---|---|
| Layer-A IND-CCA2 | ✓ | HIGH | XChaCha20-Poly1305 + U3 injectivity |
| Layer-B IND-CCA2 | ✓ | HIGH | Same as Layer-A; HKDF separation |
| Layer-C IND-CCA2 | ✓ | HIGH | HPKE-mode-base[X-Wing] + U31 libcrux CT-Decap |
| Layer-D IND-CCA2 | ✓ | HIGH | Same as Layer-C; recipient keypair honest-random |
| Envelope INT-CTXT | ✓ | HIGH | Per-layer AEAD/HPKE auth + U1 codepoint-binding + U3 injectivity |
| Replay-resistance DeviceLink/RemotePermission | ✓ | HIGH | U5 + U28 sealed-at/valid-until |
| Replay-resistance ExecuteWorkflow | ✓ | MED-HIGH | Needs R4 refinement (per-request-nonce) |
| Replay-resistance DropToRecipient | by-design forever-valid | LOW (disclosed) | #31 + U19 retention window |
| SUF-CMA-equivalent at app layer | ✓ | HIGH | Inv-15 + Inv-16 sibling composition |
| Crypto-agility (baked-in #5) | ✓ | HIGH | 3-axis permanence-triangle per MF3 |
| Freeze-permanence (baked-in #15) | ✓ | HIGH (transitive on CBOR + multiformats) | U30 + CodepointLifecycle |
| MAL-BIND-K-CT / K-PK | partial — see F1 | MED-HIGH | Per-stanza AAD per U17; needs explicit citation |

---

## §3 Q1-Q5 re-evaluation in FULL-REGISTRY context

### Q1 — Option A (oqs-rs) vs Option B (libcrux-ml-kem)

**Consolidator advisory:** Option B (libcrux), HIGH.

**Full-registry re-evaluation:**

Compose Q1 against U6 + U31 + U37 + U38 + U39 + #32 + #39 + Inv-17:
- **U6 + #32** mandate CT-Decap. Option B's `check-secret-independence` feature delivers this at compile-time via libcrux-secrets type-tagging — **mechanically closes the requirement**. Option A (oqs-rs) wraps liboqs C library; CT-Decap is best-effort C-code without compile-time tagging.
- **U31** picks Option B specifically; LOAD-BEARING.
- **U37 golden vectors** must include B-P regression. Either library can supply test vectors; no Q1-dependence.
- **U38 dudect CI** runs on Rust crates. libcrux integrates more naturally (pure Rust); oqs-rs needs C-build infra in dudect harness.
- **U39 kani** runs against Rust; no dependence on which crate.
- **#39 supply-chain**: libcrux is **single-vendor stewardship (Cryspen)**; oqs-rs wraps the Open Quantum Safe project (Microsoft + maintainers; broader maintainer base but C-FFI cost).
- **Inv-17 hybrid-mandatory**: both can satisfy when wrapped with X25519 half.

**My COMPOSED-whole assessment:** Option B is the right pick BUT the consolidator under-weights the **single-vendor risk**. Cryspen is a small (~10-person) commercial entity. If Cryspen pauses libcrux maintenance, Benten's PQ-impl story is at risk. The registry's #39 supply-chain Compromise should EXPLICITLY name this dependency boundary + add a revisit-trigger: "if libcrux upstream stalls >12 months OR Cryspen-corporate ceases maintenance OR a competing CT-verified ML-KEM crate matures (e.g. RustCrypto ml-kem with CT-SampleNTT lands), evaluate migration."

**Verdict on Q1: REFINE.** Confirm Option B per consolidator; **strengthen #39 with libcrux-single-vendor explicit row + revisit-trigger**. Confidence **HIGH** on Option B pick; **HIGH** on the #39 refinement.

### Q2 — LE-vs-BE codepoint endianness migration

**Consolidator advisory:** BE migration, MED-HIGH.

**Full-registry re-evaluation:**

Compose Q2 against U7 + U11 + U30 + L4/IMPL-B1's discovery that existing `aead.rs` uses LE:
- **U7** is THE wire-format-pin. LE-vs-BE is a one-way migration; deferring past v1-beta-freeze costs forever.
- **U11** reserves `0xFFFF` escape codepoint + `0xFE00..0xFFFE` experimental range — both interpretations meaningful in either endianness, but registry math (`0x6100..0x6FFF`) reads more naturally in BE.
- **U30 DAG-CBOR**: if adopted, CBOR has its own endianness rules (BE for multi-byte integers per RFC 8949). LE within CBOR would be inconsistent + jarring for CBOR-aware tooling. **U30 effectively forces BE.** This is a strong composition argument the consolidator did not surface explicitly.
- **U8 codepoint registry** rows would be more readable in BE (matches RFC 9180 / FIPS 203 / MLS / age conventions).
- **L4's "~1 wave-day cost" estimate** for migration looks correct given U37 golden vectors regenerate cheaply.

**Composition with v1-beta wave-sequencing:** the migration MUST happen on the canary wave (G-CORE-9-equivalent per §7) so all downstream Wave-B/C/D/E briefs absorb BE assumptions.

**Verdict on Q2: CONFIRM-WITH-EVIDENCE. STRENGTHEN to HIGH (was MED-HIGH).** The U30 composition forces the call. If U30 is also adopted (per consolidator's RECOMMENDED-leaning-LOAD-BEARING), Q2 becomes structurally required not advisory. Confidence **HIGH**.

### Q3 — L9 dual-CID vs prior-P2P-architect recipient-set-in-CID

**Consolidator advisory:** dual-CID, HIGH.

**Full-registry re-evaluation:**

Compose Q3 against U17 + U18 + U19 + Atrium-forkability semantic + Inv-15:
- **U18** is the dual-CID amendment itself.
- **U17 multi-stanza HpkeMultiBase** per-stanza AAD already binds sorted-audience for cross-stanza substitution defense. The defense is at the AAD layer, NOT the CID layer. F-refinement-2 conflates these.
- **U19 recipient-key-rotation generation** means recipient sets evolve over time. F-refinement-2's recipient-set-in-CID would force CID-churn on every recipient-key-rotation; dual-CID model decouples this (plaintext_cid stable; envelope_blob_cid churns).
- **Atrium forkability** (Ben-ratified 2026-05-27): forks split the user's identity-mesh. Pre-fork Drop CIDs must remain valid post-fork to preserve graph references. F-refinement-2 makes pre-fork Drops orphan-CIDs post-fork (recipient_set changes → CID changes).
- **Inv-15** signature-bundle-CID is NEVER load-bearing — the sibling principle is "encryption-side identifier-CID is content-identity, NOT authorization-identity." Dual-CID encodes this principle structurally.

**One concern I raise:** the `envelope_blob_cid` is BLAKE3 over the §6.2 EncryptedEnvelope serialized bytes. If U30 DAG-CBOR is adopted with strict-deterministic-decode (per my R3 refinement), then `envelope_blob_cid` is well-defined (one canonical byte-encoding per envelope value). Without strict-deterministic-decode, envelope_blob_cid is MALLEABLE — same logical envelope could have multiple CIDs. **The Q3 ratification depends on R3 (strict-deterministic-CBOR).** I flag this dependency as new.

**Verdict on Q3: CONFIRM-WITH-EVIDENCE.** Dual-CID per L9; confidence **HIGH**. **Add dependency note: requires R3 strict-deterministic-CBOR-decode for envelope_blob_cid to be well-defined.**

### Q4 — HPKE-11 vs HPKE-11-KE mode

**Consolidator advisory:** HPKE-11-KE (key-encryption mode), MED-HIGH.

**Full-registry re-evaluation:**

Compose Q4 against U17 + U33 + L7-cross-ecosystem + U29 cross-ecosystem-emit:
- **U17 HpkeMultiBase** with `cek_aead_ciphertext + cek_aead_nonce + stanzas` shape is structurally KEY-ENCRYPTION mode (HPKE wraps a CEK share; CEK + per-Node K(N) bulk-encrypt). The consolidator is correct.
- **L9 §3.1 quote** in the registry: "HPKE wraps K_principal for the recipient, then K_principal-derived K(N) bulk-encrypts" — unambiguously KEY-ENCRYPTION.
- **U29 cross-ecosystem-emit table**: JOSE alg-name `HPKE-11-KE` per draft-ietf-jose-hpke-encrypt-15. Cite drift if Benten emits `HPKE-11` (integrated mode) when the actual construction is `HPKE-11-KE` — would confuse JOSE consumers.
- **U33 NAPI-RS single-source canonical_binding**: no Q4 dependence.

**One nuance the consolidator missed:** the JOSE draft (`draft-ietf-jose-hpke-encrypt-15` / older `-11`) is **still evolving**. The exact alg-name might be `HPKE-11`, `HPKE-Base-P256-SHA256-AES128GCM` (full spelling), or `HPKE-Base-MLKEM768-X25519-SHA256-ChaCha20Poly1305-KE` (-KE suffix per key-encryption mode). The U29 cross-ecosystem-emit table should NOT hard-code an alg-name string at v1-beta; instead, the table entry should reference a `JOSE_ALG_REGISTRY_RESOLVE` function that maps Benten codepoint → current-JOSE-alg-name-AT-EMIT-TIME. This avoids cite-drift if the JOSE draft renames the alg-name post-v1-beta.

**Verdict on Q4: CONFIRM-WITH-EVIDENCE on KEY-ENCRYPTION mode; REFINE on alg-name encoding.** Adopt HPKE-11-KE semantics. **R7 refinement:** U29 alg-name field should be a versioned-resolver-function reference, not a hard-coded string. Confidence **HIGH** on mode call; **MED-HIGH** on R7 refinement.

### Q5 — Sealed-Sender as v1-beta-DEFAULT vs RESERVED-FUTURE-ADDITIVE slot

**Consolidator advisory:** additive (L6 disposition), MED.

**Full-registry re-evaluation:**

Compose Q5 against U4 + U17 + U22 + U25 + #43 + Inv-18b (per F2):
- **U4 sender-DID-in-AAD** is LOAD-BEARING for integrity (defense-in-depth against AAD-belief-drift). The default codepoint must bind sender-DID.
- **#43 metadata-leak** is the privacy Compromise that mandates honest disclosure regardless of Q5 choice.
- **U22 Sealed-Sender slot** carves out the alternative shape with sender-INSIDE-ciphertext.
- **U25 per-recipient-unlinkable invariant** applies when HpkeMultiBase ships; composes with both Q5 dispositions.

**The "additive vs default" framing is a FALSE BINARY.** The COMPOSED-whole admits a richer 3-position dispatch:

1. **DropToRecipient (Layer-C)**: sender-DID-in-AAD is LOAD-BEARING for integrity (per U4) BUT metadata-leak is high-cost (per #43). The Sealed-Sender additive slot per U22 is the right escape valve. **Caller chooses per-drop based on threat-model.** Default = sender-DID-in-AAD; Sealed-Sender = opt-in additive codepoint. This is the L6 disposition.
2. **DeviceLink + RemotePermission (Layer-D)**: sender-DID-in-AAD is LOAD-BEARING for integrity AND metadata-leak is LOW-cost (these flow within user's own mesh; cross-user metadata exposure is structurally absent). **Sealed-Sender is unnecessary; default = sender-DID-in-AAD.**
3. **ExecuteWorkflow (Layer-D U21)**: executor_did + sender_did are BOTH integrity-relevant (audit-trail of "who asked workflow X to run"). Sealed-Sender would defeat the audit-trail purpose. **Default = full AAD-binding; no Sealed-Sender variant needed.**

So the answer to Q5 is **operation-specific not envelope-wide**: Sealed-Sender as additive-slot at v1-beta with **scope = DropToRecipient ONLY** (not DeviceLink, not RemotePermission, not ExecuteWorkflow). The L6 disposition is correct AT THE SCOPE IT NAMES (DropToRecipient); the aggressive-privacy stance over-generalizes by proposing Sealed-Sender as a global default.

**Verdict on Q5: REFINE.** Adopt L6 additive-slot disposition; **explicitly scope the Sealed-Sender slot to DropToRecipient operation-class ONLY**. DeviceLink + RemotePermission + ExecuteWorkflow retain sender-DID-in-AAD default with no Sealed-Sender sibling. Confidence **HIGH** on the scoping; **MED-HIGH** on the consolidator's "additive vs default" framing being a false binary.

### Q-extra — consolidator-surfaced Am4 ↔ L6 tension

**Consolidator resolution:** keep U4 for default + U22 for Sealed-Sender sibling. HIGH.

**Full-registry re-evaluation:** CONCUR with consolidator. The pattern-induction MF1 ("structural tension resolved via sibling-codepoint carve-out") is the right framing. My F2 finding (split Inv-18 into Inv-18a/b/c) reinforces this — Inv-18b would be the dedicated "metadata-disclosure paired-sibling-slot discipline" invariant, which directly enforces MF1.

**No additional refinement on Q-extra.** Confidence HIGH.

---

## §4 Per-lens reviewer coverage-gap analysis

The consolidator (MF5) named 4 gaps. I extend with my COMPOSED-whole reading:

### Gap G1 — Formal IND-CCA2 reduction for the COMPOSED design (not just per-primitive)

**No lens did this walk.** Each lens cited per-primitive proofs (RFC 9180 §9 + Barbosa X-Wing + Bellare-Namprempre AE-INT-CTXT) but **none stitched them into a composed-construction proof** that includes U1 + U2 + U3 + U14 + Strict-decode.

I supplied an informal-rigorous walk in §2.2. **Recommendation:** the R0 plan-doc + SECURITY-POSTURE.md should reproduce this G0→G4 game-hop reduction as audit-firm-readable prose. ~0.5 wave-day cost.

**Status:** GAP-CLOSED-PARTIALLY by this critique §2.2; full closure = audit-firm review at external-audit time.

### Gap G2 — MAL-BIND-K-CT / MAL-BIND-K-PK binding-properties

**No lens evaluated against `draft-sfluhrer-cfrg-ml-kem-security-considerations-04`'s binding-properties.** The registry mentions only "CT-SampleNTT investigation" — a different ML-KEM concern.

draft-sfluhrer-04 establishes that ML-KEM is **MAL-BIND-K-CT and MAL-BIND-K-PK insecure** — an adversary who can construct a malformed-but-decodable private key can produce ciphertexts where the same shared secret K maps to multiple (pubkey, ciphertext) pairs. **For HPKE-mode-base[X-Wing] this is generally a NON-THREAT for KEM-as-key-exchange use** per draft-sfluhrer-04: "this is not a threat to normal uses of ML-KEM as a key exchange or public key encryption method." But it BECOMES a threat IF the protocol uses K for authentication where the adversary controls the private key.

**Reachability assessment for the COMPOSED design:**
- **Layer-C (drop-to-recipient)**: recipient is honest (it's the user's own device); their sk is honestly generated. **NOT REACHABLE.**
- **Layer-D (DeviceLink + RemotePermission)**: recipient device is honest (user's own enrolled device). **NOT REACHABLE.**
- **Layer-D (ExecuteWorkflow per U21)**: `result_recipient_pubkey: HybridKemPubKey` is supplied by the caller. **POTENTIALLY REACHABLE** if a malicious caller constructs a malformed pubkey + asks the executor to encrypt results to it. The malformed-pubkey-attack class isn't immediately exploitable (executor still produces real ciphertext; the K binding ambiguity doesn't break confidentiality of the encrypted result), BUT it surfaces in audit-trail contexts where K is later used to verify "this is the result for that request."
- **RecoveryHook escrow (per F-full scope review §7)**: recovery-key escrow CAN involve a recovery-pubkey supplied by an external party. **POTENTIALLY REACHABLE.**

**Defense in the COMPOSED design:** U17 + U19 bind `recipient_did + recipient_key_generation` into AAD, AND U18 dual-CID binds `plaintext_cid` into AAD. An adversary mounting MAL-BIND-K-CT must produce a ciphertext (cipher_1, cipher_2) decapsulating to the same K but with different bound-public-keys; per-stanza AAD-binding of (recipient_did, recipient_key_generation) limits this to "two stanzas with the SAME recipient_did + SAME key_generation but different enc/ciphertext bytes producing the same K" — which is a much narrower attack surface than the bare ML-KEM MAL-BIND class. **The per-stanza AAD-binding per U17 + U19 IS the registry's structural defense** but the registry doesn't NAME draft-sfluhrer-04's binding-properties as the literature anchor.

**Recommendation:** mint **proposed Amendment U41 (LOAD-BEARING-DOC, NOT wire-affecting)** and **proposed Compromise #45 (honest-disclosure)** — see §6.1.

**Status:** NEW FINDING F1. Cost = doc-only; ~0.5 wave-day to draft SECURITY-POSTURE.md section + cross-reference U17 + U19. NOT a wire-format change.

### Gap G3 — Chosen-seed-via-password adversary model formal treatment

The 1st reviewer (L1 §2.3) dismissed this via informal argument: "if the only attack surface is offline-brute-force, F+ is as-secure-as AEAD-under-DAK"; the 2nd reviewer (L2 §3.3) refined: "F+ slightly stronger against pure-offline-brute-force; materially weaker against side-channel-enabled adversary." Neither cited a formal-model paper that covers the adversarially-chosen-recipient-seed-via-password class.

**For the COMPOSED design this is NOT a load-bearing gap because F+ is NO-GO.** The F+ rejection moots the chosen-seed-via-password adversary model — Layer-A uses XChaCha20-Poly1305 not ML-KEM-keygen-from-seed. **HIGH confidence the gap is closed by the NO-GO.**

**One residual:** if a future ratification revisits F+ (e.g. post-2030 ML-KEM-CT-SampleNTT lands per L1 §6.3 revisit-trigger), the formal-model gap re-opens. **Document the gap explicitly in V1-FROZEN-INTERFACE-DEFERRED.md F+ revisit-trigger row** so the future-revisit doesn't re-encounter the gap blind. ~0.1 wave-day.

**Status:** GAP-CLOSED-VIA-NO-GO; document explicitly for future revisit.

### Gap G4 — Quantum-side-channel-during-decapsulation hybrid attacks

The registry covers Bernstein-Persichetti chosen-ciphertext-side-channel (#32). It does NOT cover **quantum-side-channel-during-decapsulation hybrid attacks** — e.g., if a future quantum adversary measures classical side-channels during ML-KEM Decap AND has access to a quantum oracle for the X25519 half of X-Wing, can they distinguish the hybrid combiner output?

**Reachability for v1-beta:** quantum adversary is post-2035-timeframe; classical side-channel + quantum-X25519-oracle is highly speculative. **NOT a v1-beta concern.** Worth a one-liner in #44 BSI long-term-confidentiality Compromise: "long-term-confidentiality posture also OUT-OF-SCOPE for hybrid-classical-quantum-side-channel attacks; revisit if quantum-X25519-oracle becomes practical."

**Status:** GAP-NAMED; ~0.05 wave-day to add one-liner.

### Gap G5 — draft-sfluhrer-04 alignment (other recommendations beyond MAL-BIND)

draft-sfluhrer-04 also includes recommendations on **rho transmission discipline** (don't transmit rho with the ciphertext if rho was generated from a secret; mostly relevant for KEM-as-MAC use cases — NOT Benten's use case) and **subject-identifier-binding** (similar to U17's per-stanza AAD).

The COMPOSED design IS draft-sfluhrer-04-aligned via U17 + U19 binding. **No additional finding; just document the alignment in SECURITY-POSTURE.md or CRYPTO-PARAMETERS.md.** ~0.1 wave-day.

### Gap G6 — Privacy of replay-window (Am5 timestamp leak)

L6 partially noted this; U28 refines U5 to coarse 1-hour buckets. The COMPOSED-whole reading: U28 reduces ~33-bit timestamp leak (second-precision) to ~14 bits/year (hour-precision with jitter). This is a META-FINDING the consolidator named — closed by U28 if Ben adopts the v1-beta-LOAD-BEARING consolidator lean.

**Residual:** the **`valid_until_epoch_hour` field is also bucket-reduced** under U28 logic, which means the replay-window granularity becomes 1-hour. For DeviceLink + RemotePermission this is acceptable; for ExecuteWorkflow this is INSUFFICIENT (a 1-hour replay window admits within-hour replay of execute-workflow requests, potentially exhausting `max_decrypt_count`). **R4 refinement (per §2.4) closes this** by adding per-request-nonce.

**Status:** CLOSED by U28 + R4.

### Gap G7 — Per-relay-unlinkability (U23) composition with iroh transport

U23 proposes a `TransportEnvelope { transport_blinded_id, envelope }` wrapper at the iroh-transport boundary. The L9 + L6 lenses considered the shape but didn't validate against **iroh's actual transport model** (iroh-blobs + iroh-gossip). iroh provides:
- **iroh-blobs**: BLAKE3-CID-addressed blobs. The "transport_blinded_id" would have to be the BLAKE3 of the wrapper, NOT a random per-hop salt — or else iroh-blobs can't address it.
- **iroh-gossip**: pub-sub topic-id is the addressing primitive. Per-hop salt would interact with topic-id rotation.

**The U23 design needs an iroh-specialist review before R0 plan-doc lands.** The "post-v1-beta impl at iroh-transport-boundary" disposition gives runway, but the shape-lock at v1-beta means the wire format commits to something we haven't verified against iroh's primitives.

**Recommendation:** dispatch an iroh-specialist sub-review at R3/R5 design time. Status: NEW NAMED-DEFERRED.

### Gap G8 — Three-letter-agency / nation-state adversary model

No lens explicitly handles the NSA/GCHQ/MSS-class adversary (who can compromise CA certs, can perform mass-surveillance, may have undisclosed cryptanalysis capability against X-Wing). L5 §4.4 enumerates regulatory non-compliance (eIDAS-3 / BSI / French regulated-financial) but doesn't model nation-state cryptanalysis.

**Reachability:** if NSA has unpublished cryptanalysis against ML-KEM-768, Inv-17 hybrid-mandatory protects via X25519 classical-floor (assumes NSA does NOT also have a quantum-strong-DH break against X25519, which is the BSI/ANSSI long-term-confidentiality concern that #44 already discloses). **Closed by #44 + Inv-17.**

**Residual:** add one-liner to #44 explicitly naming "nation-state adversary OUT-OF-SCOPE; hybrid-mandatory per Inv-17 is the structural defense against single-primitive-break." ~0.05 wave-day.

### Gap G9 — UX-affordance lens (consolidator named in MF5)

The consolidator named this. I add: the U22 Sealed-Sender opt-in UX **must be discoverable** — a Tauri/browser-shell user clicking "send drop to recipient" should see "this drop's metadata is visible to relays — switch to Sealed-Sender? [link to explanation]." Without this, "secure-by-default" framing is misleading.

**Status:** UX-design concern; flag for post-v1-beta UX wave.

### Gap G10 — Multi-stanza-HPKE specialist (consolidator named in MF5)

The consolidator named this. I add: U17 cross-stanza AAD binding `(codepoint, plaintext_cid, sorted_audience, sender_did, stanza_index, recipient_key_generation)` is rigorous BUT the MLS-PQ TreeKEM specialist class of review specifically stress-tests the "two recipients in audience, one is adversary-controlled" attack class. **Dispatch at R3/R5 when HpkeMultiBase variant lands.** Status: NAMED-DEFERRED-TO-R3/R5.

### Coverage gap summary

| Gap | Status | Cost to close |
|---|---|---|
| G1 — composed IND-CCA2 reduction | PARTIAL via §2.2; full at external-audit | ~0.5 day to write up |
| G2 — MAL-BIND draft-sfluhrer-04 | NEW F1; close via U41 + #45 | ~0.5 day doc |
| G3 — chosen-seed adversary formal | CLOSED-VIA-NO-GO; doc revisit-trigger | ~0.1 day doc |
| G4 — quantum-side-channel hybrid | NOT-V1-BETA; one-liner in #44 | ~0.05 day |
| G5 — draft-sfluhrer-04 other recs | ALIGNED; document alignment | ~0.1 day |
| G6 — replay-window privacy | CLOSED by U28 + R4 | (R4 in §2.4) |
| G7 — U23 iroh-transport compat | NEEDS-iroh-specialist | dispatch at R3/R5 |
| G8 — nation-state adversary | CLOSED by #44 + Inv-17; one-liner | ~0.05 day |
| G9 — UX-affordance | post-v1-beta UX wave | out of scope |
| G10 — multi-stanza HPKE specialist | NAMED-DEFERRED to R3/R5 | dispatch at R3/R5 |

Total doc-only close cost: **~1.3 wave-days**. The G7 + G10 dispatch costs are baked into R3/R5 wave planning.

---

## §5 F+ NO-GO re-confirmation

**10 of 10 cryptographer eyes CONCUR NO-GO on Option F+ pseudo-keypair for Layer-A vault.**

I add the COMPOSED-whole perspective: even setting aside L1's structural rejection (ML-KEM keygen-from-secret-seed side-channel hazard), the COMPOSED-whole design across all 28 amendments has a **structural disincentive** against F+:

1. **U32 XChaCha20-Poly1305 is LOAD-BEARING** for Layer-A per registry. XChaCha20 has 2^96 nonce-birthday and is the right primitive for the vault rotation-frequency use case. F+ at Layer-A would replace this with HPKE-mode-base[X-Wing], which has DIFFERENT (worse) properties for the vault-encrypt-to-self use case.

2. **U3 length-injectivity** is well-defined for the AEAD path. For HPKE-mode-base path, length-injectivity must compose with HPKE's `info` parameter discipline; the proof story is more elaborate. F+ at Layer-A would force the more elaborate proof for ZERO functional benefit.

3. **U4 sender-DID-in-AAD** is structurally inapplicable at Layer-A (vault is encrypt-to-self; there is no "sender" distinct from "owner"). The U4 carve-out for Vault is registry-clean. F+ at Layer-A would re-introduce ambiguity ("is the password-derived pseudo-keypair the 'sender'?") that the AEAD-under-DAK shape avoids.

4. **U18 dual-CID** at Layer-A is degenerate (vault has no peer-mesh CID story). F+ at Layer-A would shoehorn HPKE's enc/ciphertext into a vault.cbor file — extra bytes, no CID benefit.

5. **U6 + U31 + #32** Bernstein-Persichetti CT-Decap mandate. F+ at Layer-A would force EVERY vault-unlock to run ML-KEM Decap-from-password-derived-sk under adversary-watching-CPU-side-channels — the ENTIRE class of side-channel concerns that L1 §4.3 + L2 §3.3 + L3/Am6 raise. AEAD-under-DAK has NONE of these.

**The COMPOSED-whole structurally REJECTS F+ even before considering L1's KeyGen side-channel — the registry's other amendments are AEAD-shaped at Layer-A.** This is independent corroboration of the L1 NO-GO from a structural-composition angle the per-lens reviewers couldn't see.

**Verdict: F+ NO-GO RE-CONFIRMED with FRESH EYES.** Confidence **HIGH**. Document the L1 + L2 + L3 + L4 + L5 + L6 + L7 + L8 + L9 + C3 unanimous-concur in V1-FROZEN-INTERFACE-DEFERRED.md F+ revisit-trigger row.

---

## §6 Additional findings (NEW candidates)

### §6.1 Proposed Amendment U41 + Proposed Compromise #45 — MAL-BIND binding-properties alignment

**Amendment U41 (proposed LOAD-BEARING-DOC; NOT wire-affecting):**

> The COMPOSED EncryptedEnvelope design's per-stanza AAD binding (per U17 + U19) structurally defends against adversary-supplied-recipient-pubkey MAL-BIND-K-CT / MAL-BIND-K-PK attacks per `draft-sfluhrer-cfrg-ml-kem-security-considerations-04`. SECURITY-POSTURE.md MUST cite the binding-properties literature anchor + cross-reference U17 + U19 as the structural defense. CRYPTO-PARAMETERS.md MUST disclose that ML-KEM-768 is MAL-BIND-K-CT / K-PK insecure as a primitive but Benten's COMPOSED construction is not vulnerable in normal use because (a) recipients are honest (own user devices) for Layer-C/D-default flows, AND (b) per-stanza AAD-binding limits MAL-BIND attacks to within-same-recipient-key-generation (a much narrower surface).
>
> EXCEPTION: ExecuteWorkflow (U21) `result_recipient_pubkey` flow + RecoveryHook escrow-pubkey flow MAY admit attacker-supplied pubkey IF caller is malicious. These flows MUST validate `recipient_pubkey` via `validate_public_key()` per FIPS 203 + libcrux-ml-kem validation API (per U31). Without validation, MAL-BIND-K-PK attack class is reachable.

Severity: LOAD-BEARING-DOC + IMPL-AMENDMENT (validation call must be present). Wire-affecting: N. Impl-only: Y (validation call). Codepoint-reserve: N.
Disposition: v1-beta-LOAD-BEARING.
Depends-on: U17, U19, U21, U31.
Compromise/Invariant: #45 (new). Doc impact: SECURITY-POSTURE.md + CRYPTO-PARAMETERS.md + ExecuteWorkflow + RecoveryHook impl sites.
Confidence: HIGH on the structural defense being correct; MED-HIGH on practical-reachability against typical Benten user.

**Compromise #45 (proposed):**

> **ML-KEM-768 MAL-BIND-K-CT / MAL-BIND-K-PK binding-properties (PARTIAL-CLOSURE via per-stanza AAD per U17 + U19; EXCEPTION for caller-supplied-pubkey flows)**
>
> Origin: `draft-sfluhrer-cfrg-ml-kem-security-considerations-04`. C3 §6.1 finding F1.
> Statement: ML-KEM-768 as a primitive does NOT satisfy MAL-BIND-K-CT or MAL-BIND-K-PK (adversary-controlled private key can produce non-injective K → ciphertext map). For Benten's COMPOSED Layer-C drop-to-recipient + Layer-D DeviceLink + RemotePermission flows, the recipient sk is honestly generated on a user-controlled device → MAL-BIND class is NOT REACHABLE. For ExecuteWorkflow (U21) `result_recipient_pubkey` + RecoveryHook escrow-key flows where caller may supply attacker-controlled pubkey, MAL-BIND class IS REACHABLE; mitigated by `validate_public_key()` FIPS 203 calls per U41.
> Threat boundary: applies to flows where caller supplies recipient_pubkey without prior trust establishment. Does NOT apply to Layer-A/B (no ML-KEM) or Layer-C/D-default flows (recipient pubkey is from own-mesh user-device).
> Mitigation status: **STRUCTURALLY-NON-REACHABLE for v1-beta-default flows; PARTIAL via U41 validation for ExecuteWorkflow + RecoveryHook flows**.
> Wire impact: N. Doc impact: SECURITY-POSTURE.md + CRYPTO-PARAMETERS.md.

Confidence HIGH on the disclosure shape; MED-HIGH on the v1-beta scope of "non-reachable for default flows."

### §6.2 Refinement F2 — Split Inv-18 into Inv-18a/b/c

**Inv-18a** (codepoint-registry-discipline + IANA-disjoint range + CodepointLifecycle): origin L5/Inv-L5-3 + L8/Am16. Single coherent registry-governance invariant.

**Inv-18b** (metadata-disclosure paired-Sealed-Sender-slot discipline): origin L6/§6.2 Inv-16-metadata. Every envelope shape that places identity-DIDs in plaintext AAD MUST have a paired metadata-hiding additive codepoint slot reserved at v1-beta (Sealed-Sender per U22 OR equivalent). **Operation-scoped per Q5 refinement** — only Layer-C DropToRecipient operation-class is in scope.

**Inv-18c** (CodepointLifecycle typed-state): origin L8/Am16. Pure Rust-type-system + decoder discipline.

**Why split:** the three are LOGICALLY distinct enforcement surfaces. Inv-18a is registry-as-document. Inv-18b is design-time pairing discipline. Inv-18c is decoder-runtime state. Audit-firm reading + cite-drift-detector lint rule shape + enforcement-plan readability all argue for separation. The consolidator self-noted this risk in §9 item 1.

Confidence MED-HIGH on the split-vs-merge call; ratify based on Ben preference. Either works for v1-beta.

### §6.3 Refinement R3 — strict-deterministic-CBOR-decode for U30

If U30 DAG-CBOR is adopted, MUST require strict-deterministic-CBOR encoding per RFC 8949 §4.2.1. Without it:
- INT-CTXT softens at framing layer (re-encoding malleability).
- U18 `envelope_blob_cid` is MALLEABLE (same logical envelope → multiple CIDs).
- Cross-implementation reproducibility breaks (Rust + TS + future-language ports must agree on canonical encoding).

Add as explicit U30 sub-clause. Cost: ~0.1 wave-day to amend U30 text + add `serde_cbor::de::Deserializer::set_strict()`-equivalent decoder discipline. Confidence HIGH.

### §6.4 Refinement R4 — Per-request-nonce for ExecuteWorkflow

ExecuteWorkflow (U21) MUST bind `request_nonce: [u8;16]` into AAD in addition to sealed_at + valid_until + executor_did. Without this, U28's 1-hour-bucket coarsening admits within-hour replay attacks that could exhaust `max_decrypt_count`. Cost: ~0.1 wave-day (one extra AAD field). Confidence HIGH.

### §6.5 Refinement R5 — Permanence-triangle naming in CRYPTO-CODEPOINTS.md

Name the 3-axis permanence-package structure (variant / codepoint / canonicalization-version) EXPLICITLY in V1-FROZEN-INTERFACE.md item 6 + CRYPTO-CODEPOINTS.md preamble. Future codepoint mints learn which axis to extend. Cost: ~0.1 wave-day. Confidence HIGH.

### §6.6 Refinement R6 — Compromise #46 candidate (multiformats stewardship dependency)

OR amend #39: Benten's freeze-permanence story is TRANSITIVE on:
- CBOR (IETF standard; well-stewarded)
- multiformats multikey + multicodec (community-stewarded; smaller team)
- BLAKE3 (single-vendor stewarded by BLAKE3 team)
- libcrux-ml-kem (single-vendor stewarded by Cryspen per Q1 refinement R1)
- RustCrypto (community stewarded)
- McMillion `rust-hpke` (single-vendor stewarded)

Each of these is a permanence-stewardship dependency. **Honest disclosure in #39 supply-chain + new Compromise #46 stewardship-continuity is the right shape.** Cost: ~0.2 wave-day doc. Confidence HIGH.

### §6.7 Refinement R7 — U29 alg-name resolver function (not hard-coded string)

U29 cross-ecosystem-emit table entries should reference a versioned-resolver function not a hard-coded string. Avoids cite-drift if JOSE draft renames alg-name post-v1-beta. Cost: ~0.2 wave-day. Confidence MED-HIGH.

### §6.8 NEW Candidate Amendment U42 (LOW priority) — Standardized validation-error-code taxonomy at envelope decode

The COMPOSED design has multiple decode-failure modes:
- `EnvelopeError::UnknownCodepoint`
- `EnvelopeError::UnknownPayloadVariant`
- `EnvelopeError::UnknownBindingContextVariant`
- `EnvelopeError::AadCanonicalizationMismatch`
- `EnvelopeError::AuthTagFail`
- `EnvelopeError::ReplayWindowExpired`
- `EnvelopeError::RecipientKeyGenerationMismatch`
- `EnvelopeError::CrossVariantDecodeAttempted` (per U2 strict-decode)
- `EnvelopeError::MalformedCbor`
- `EnvelopeError::NonCanonicalCbor` (per R3)

**Defense-in-depth concern:** the granular error-code distinction can leak side-channel info to an adversary probing decode-paths. RFC 9180 §9.1.4 recommends "return a single uniform error" to avoid distinguishing-attack. The COMPOSED-whole consideration: granular errors are helpful for legitimate debugging; uniform-errors are helpful for adversary-resistance. **Recommendation:** internal logging gets granular error-codes; external-API surface returns uniform `EnvelopeError::DecodeFail` to callers (especially Tauri/browser/WASM boundary). U42 codifies this.

Severity: LOW. Cost: ~0.2 wave-day. Confidence MED-HIGH.

### §6.9 NEW Candidate Compromise #47 (LOW priority) — Tauri NAPI-RS marshaling boundary side-channels

V8 GC + JS-engine timing + WASM-runtime-instrumentation introduce timing surfaces that pure-Rust crypto code doesn't have. NAPI-RS opaque-handle pattern (U34) helps but doesn't eliminate. Honest-disclosure in SECURITY-POSTURE.md.

Cost: ~0.1 wave-day. Confidence MED.

---

## §7 R0 plan-doc structural advice

The consolidator's §7 R0 skeleton (16 sections) is COMPREHENSIVE but I add structural advice from COMPOSED-whole reading:

### §7.1 Add a NEW §0 Architectural commitment statement

Before §1 Architectural framing, add **§0 Architectural commitment statement** that ratifies (a) F+ NO-GO 10-of-10 cryptographer-eye concurrence, (b) §6.2 envelope-layer-unification direction, (c) the 28 amendments + 13 Compromise mints + 3 invariants as the agreed shape, (d) Ben's Q1-Q5 ratification decisions. **One page; permanent reference.**

### §7.2 Reorder §1-§6 around the "permanence triangle"

The current §1-§6 organize around layer (Layer-A/B/C/D). Re-organize around the 3-axis permanence triangle:
- **§1 Variant axis** — EnvelopePayload variants + BindingContext variants per layer
- **§2 Codepoint axis** — codepoint registry + brackets + lifecycle states
- **§3 Canonicalization axis** — canonical_binding + aad_version + TLV + CBOR

Then §4-§7 per-layer construction details. This makes the permanence-package (MF3) load-bearing structural rather than after-the-fact observation.

### §7.3 Promote §14 THREAT-MODEL.md skeleton to §2

Per L5's "single most load-bearing recommendation." Audit-firms read THREAT-MODEL.md FIRST. Putting it at §2 (right after architectural framing) signals audit-readiness as a first-class concern. Cost: re-number sections.

### §7.4 Add a NEW §8 reduction-proof section

Reproduce my §2.2 G0→G4 game-hop reduction as audit-firm-readable prose. ~1 page; closes G1 gap.

### §7.5 Add a NEW §13 MAL-BIND binding-properties section

Per F1 finding. ~0.5 page; closes G2 gap. Cross-reference U17 + U19 + U41 + #45.

### §7.6 Wave-decomposition refinements

The consolidator's §8 wave decomposition (Wave A-F) is reasonable BUT I recommend:
- **Canary wave (G-CORE-9-equivalent)** ships U1 + U2 + U3 + U7 + U11 + U14 + U30 + the new permanence-triangle V1-FROZEN-INTERFACE.md item 6 amendment. **This is the wire-format-freeze wave.** Everything downstream waits for it.
- **Wave A absorbs U4 + U5 + U9 + U10 + U28** (envelope amendments + replay-window + non_exhaustive enums).
- **Wave B absorbs U6 + U31 + U32 + U34 + U35 + U36 + U37 + U38 + U39** (impl-engineering + libcrux + XChaCha20 + NAPI + CT-validation + golden vectors + kani).
- **Wave C absorbs U17 + U18 + U19 + U22 + U25 + #43 + Inv-18b** (Layer-C drop-to-recipient + Sealed-Sender slot).
- **Wave D absorbs U21 + R4** (Layer-D ExecuteWorkflow + per-request-nonce).
- **Wave E absorbs U29 + U33 + U41 + #45** (cross-ecosystem + NAPI single-source + MAL-BIND docs).
- **Wave F absorbs U40 (THREAT-MODEL.md) + Inv-16 + Inv-17 + Inv-18a/b/c + all Compromise mints + SECURITY-POSTURE.md**.

Total ~7 waves (canary + A-F). Per `feedback_canary_first_parallel_implementation`, Waves A-F can parallelize after canary merges.

### §7.7 R3 test-landscape additions

To the consolidator's §9 test-landscape outline, add:
- **MAL-BIND adversarial test**: construct attacker-controlled-pubkey scenario for ExecuteWorkflow; verify validate_public_key rejects.
- **U18 envelope_blob_cid stability test**: verify same logical envelope → same CID under strict-deterministic-CBOR-decode (per R3).
- **U28 hour-bucket replay test**: verify within-hour replay rejected for ExecuteWorkflow (per R4).
- **U2 strict-decode test corpus**: ~24 cross-variant ciphertext samples; verify all auth-fail.
- **U17 cross-stanza substitution test corpus**: ~16 multi-stanza variants; verify Bob can't receive Carol-meant content.

### §7.8 RecoveryHook integration

The F-full scope review §7 RecoveryHook trait + my F1 finding interact. RecoveryHook escrow-pubkey flow MUST validate the escrow-pubkey via `validate_public_key()` per U41. Make this explicit in §16 RecoveryHook design.

---

## §8 Self-assessment + confidence per finding

### What I did + how I worked

1. **Tree-state pre-flight** at `2172cb6d` clean against `origin/main`. Branch cut to `phase-4-meta-core/option-f-plus-critique-c3-fresh-eyes-cryptographer`.
2. **Read consolidated registry in full** (~939 LOC) + skimmed L1 + L2 + L3 + key sections of L4-L9 via grep + targeted reads.
3. **Built mental model of COMPOSED EncryptedEnvelope** after all 28 amendments. Identified per-layer construction shape + AAD discipline + replay surface.
4. **Walked per-property assessment** (IND-CCA2 + INT-CTXT + replay + SUF-CMA + crypto-agility + freeze-permanence) for the COMPOSED design.
5. **Did a single web check** for draft-sfluhrer-04 binding-properties (turned up the MAL-BIND-K-CT / MAL-BIND-K-PK angle — drove F1 finding) + libcrux-ml-kem `check-secret-independence` status verification.
6. **Re-evaluated Q1-Q5** in full-registry context — found 3 CONFIRM + 2 REFINE.
7. **Surfaced 10 coverage gaps** (G1-G10) + closed 6 in this doc; flagged 2 for dispatch at R3/R5; 1 for post-v1-beta UX; 1 closed-via-NO-GO.
8. **Drafted F1 + F2 net-new findings** + 6 refinements (R1-R7 minus R2).
9. **Re-confirmed F+ NO-GO** with COMPOSED-whole structural-disincentive argument independent of L1's KeyGen-side-channel.
10. **Drafted R0 plan-doc structural advice** (8 sub-items).

### Per-finding confidence

| Finding | Severity | Confidence | Cost-to-close |
|---|---|---|---|
| F+ NO-GO re-confirmation | LOAD-BEARING | HIGH | already-done |
| F1 — MAL-BIND binding-properties | LOAD-BEARING-DOC | HIGH structural; MED-HIGH practical | ~0.5 day |
| F2 — split Inv-18 into a/b/c | MED-HIGH | MED-HIGH | ~0.2 day (Ben call) |
| F3 — composed IND-CCA2 reduction | NEEDED-DOC | HIGH that §2.2 covers; HIGH external-audit will require formal | ~0.5 day write-up |
| R1 — libcrux single-vendor risk | LOAD-BEARING-DOC | HIGH | ~0.2 day |
| R3 — strict-deterministic-CBOR | LOAD-BEARING (if U30) | HIGH | ~0.1 day |
| R4 — ExecuteWorkflow per-request-nonce | LOAD-BEARING | HIGH | ~0.1 day |
| R5 — permanence-triangle naming | RECOMMENDED | HIGH | ~0.1 day |
| R6 — stewardship-continuity Compromise | RECOMMENDED | HIGH | ~0.2 day |
| R7 — alg-name resolver function | RECOMMENDED | MED-HIGH | ~0.2 day |
| U42 — uniform-error-code taxonomy | LOW | MED-HIGH | ~0.2 day |
| #47 — Tauri NAPI-boundary side-channels | LOW | MED | ~0.1 day |
| Q1 (libcrux) | CONFIRM-WITH-EVIDENCE | HIGH | (R1 strengthens) |
| Q2 (BE migration) | CONFIRM strengthened HIGH | HIGH | (U30 composition forces) |
| Q3 (dual-CID) | CONFIRM-WITH-EVIDENCE | HIGH | (R3 dependency noted) |
| Q4 (HPKE-11-KE) | CONFIRM mode + REFINE alg-name | HIGH on mode; MED-HIGH on R7 | (R7) |
| Q5 (Sealed-Sender) | REFINE — operation-scoped | HIGH on scoping | (Inv-18b operationalizes) |

Total new doc-only cost (F1-F3 + R1 + R3-R7 + U42 + #47): **~2.3 wave-days**, well within v1-beta budget per consolidator's §6 ~65-72 wave-day total.

### What I could be wrong about

1. **R3 strict-deterministic-CBOR-decode REJECTING-NON-CANONICAL admit-policy** may break interop with CBOR-aware tools that emit non-canonical-but-semantically-equivalent CBOR. Mitigation: only reject at envelope-decode boundary, not at general CBOR-ingest. I have HIGH confidence on the principle; MED-HIGH on the specific reject-policy shape.
2. **F1 MAL-BIND practical-reachability for Benten's specific flows** may be lower than I estimate. The ExecuteWorkflow + RecoveryHook flows are post-v1-beta-Day-One per registry; the v1-beta-Day-One flows are all honest-recipient. So F1 is a **slot-reservation-class concern** for v1-beta (document now; validate later when ExecuteWorkflow + RecoveryHook implement).
3. **F2 split-vs-merge Inv-18 call**: I lean split; consolidator merged for compactness. Reasonable cryptographers disagree. MED-HIGH confidence either way works.
4. **R7 alg-name resolver function** may add unnecessary indirection if JOSE draft stabilizes at HPKE-11-KE name before v1-beta. MED-HIGH confidence; could go either way.
5. **U42 + #47 LOW-priority candidates** could be deferred entirely without v1-beta-load-bearing-cost. Surface for Ben call; defer-OK.

### What this critique does NOT cover

- I did NOT independently verify the registry's claim that all 9 lens reviews CONCUR F+ NO-GO at HIGH confidence — propagated.
- I did NOT enumerate every L4 + L7 + L8 + L9 individual observation; focused on amendments + Compromise mints + invariants per registry §2 + §3 + §4.
- I did NOT cross-check U30 DAG-CBOR-tag 0xBE54 against IANA CBOR-tag registry availability — propagated L7's claim that the slot is registerable pre-v1-beta.
- I did NOT do an iroh-specialist review of U23 transport-blinded-id shape — flagged for R3/R5 dispatch.
- I did NOT do an MLS-PQ TreeKEM specialist review of U17 cross-stanza substitution — flagged for R3/R5 dispatch.
- I did NOT formally machine-verify the §2.2 G0→G4 game-hop reduction — informal-rigorous prose only.

### Stance per brief

The brief stated: "ADVISORY not load-bearing. The 9 reviewers' work is HIGH-quality input but their lens-fragmentation may have caused them to miss the COMPOSED-whole properties. DISAGREE-WITH-EXPLANATION is first-class."

I have exercised DISAGREE-WITH-EXPLANATION on Q5 (operation-scoped vs envelope-wide framing). I have exercised REFINE on Q1 (single-vendor risk) + Q4 (alg-name resolver). I have CONCUR-WITH-STRENGTHENED-EVIDENCE on Q2 (U30 forces BE) + Q3 (R3 dependency noted). I have surfaced 3 NEW findings (F1 MAL-BIND, F2 Inv-18 split, F3 reduction prose) the per-lens reviewers structurally could not see.

The verdict **CONCUR-WITH-REFINEMENTS** is the right calibration. The registry is structurally correct; the refinements + new findings are additive to the registry, not replacement of it. Ben can ratify the registry + my refinements together as the F-full R0 plan-doc input.

---

## §9 Citations

### §9.1 Frozen SHAs

- Consolidated registry: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md`
- L1 1st-cryptographer: `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f`
- L2 2nd-opinion: `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b`
- L3 adversarial-design: `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3`
- L4 impl-engineering: `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering @ 4d4aae5f`
- L5 threat-model+audit-readiness: `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0`
- L6 privacy/metadata-leak: `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak @ 986e50bb`
- L7 cross-ecosystem-interop: `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop @ 208f98bb`
- L8 wire-format-stability: `phase-4-meta-core/option-f-plus-lens-l8-wire-format-stability @ d8d3c41c`
- L9 atrium-integration: `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03`
- e2r-ffull-scope: `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`

### §9.2 External standards + drafts (NEW citations beyond registry §10.3)

- [draft-sfluhrer-cfrg-ml-kem-security-considerations-04](https://datatracker.ietf.org/doc/html/draft-sfluhrer-cfrg-ml-kem-security-considerations-04) — MAL-BIND-K-CT / MAL-BIND-K-PK binding-properties (drives F1 finding).
- [draft-sfluhrer-cfrg-ml-kem-security-considerations (html)](https://sfluhrer.github.io/ml-kem-security-considerations/draft-sfluhrer-cfrg-ml-kem-security-considerations.html) — rendered HTML version.
- [draft-connolly-cfrg-hpke-mlkem-00 — ML-KEM for HPKE](https://www.ietf.org/archive/id/draft-connolly-cfrg-hpke-mlkem-00.html) — HPKE-mode-base[X-Wing] target draft.
- RFC 8949 §4.2.1 deterministic CBOR encoding (drives R3 refinement).
- RFC 9180 §9.1.4 uniform-error-response recommendation (drives U42 candidate).
- FIPS 203 §6.3 ML-KEM Decap implicit-rejection + `validate_public_key()` (drives U41 + R1).

### §9.3 libcrux-ml-kem stewardship verification

- [libcrux-ml-kem crates.io](https://crates.io/crates/libcrux-ml-kem) — published crate.
- [libcrux-ml-kem docs.rs](https://docs.rs/libcrux-ml-kem/latest/libcrux_ml_kem/) — `check-secret-independence` feature + secret-independence verification via libcrux-secrets.
- [Cryspen: Verifying Libcrux's ML-KEM](https://cryspen.com/post/ml-kem-verification/) — hax/F* verification post.
- [pq-code-package/rust-libcrux GitHub](https://github.com/pq-code-package/rust-libcrux) — pq-code-package multi-vendor stewardship layer atop Cryspen libcrux.
- [libcrux-secrets docs.rs](https://docs.rs/libcrux-secrets) — compile-time secret-independence type-tagging.

### §9.4 Benten internal references

- All registry §10.2 internal references propagate.
- CLAUDE.md baked-in #5 (crypto-agility), #15 (v1-beta interface freeze), #17 (deployment-shapes), #18 (authority-isolation vs confidentiality-isolation).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` (drove the holistic re-review pass).
- `feedback_review_finding_ground_truth_verify` (drove the DISAGREE-WITH-EXPLANATION on Q5).
- `feedback_engine_primitives_vs_application_layer` (informs the F+ NO-GO structural-disincentive § §5).
- F-full scope review §7 RecoveryHook (interacts with F1 finding per §7.8).

---

**End of C3 fresh-eyes cryptographer critique.**
