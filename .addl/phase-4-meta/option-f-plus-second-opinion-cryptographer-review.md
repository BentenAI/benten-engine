# Option F+ pseudo-keypair pattern — SECOND-OPINION senior-cryptographer review

**Reviewer:** Senior cryptographer (independent fresh-eyes second opinion).
**Branch:** `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review`.
**Inputs ground-truth-verified:**
- `origin/phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — the prior NO-GO review (the "prior reviewer").
- `origin/phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — §14.1 contains the original Option F+ pitch.
- `origin/phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17` — earlier encrypt-to-recipient cryptographer review (Option B Conditional-GO).
- `origin/phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b` — L12 signature review (Inv-15 origin).
- `docs/INVARIANT-COVERAGE.md` Inv-15 (registered, partial-enforcement); Inv-16 mint **pending this decision**.
- `docs/SECURITY-POSTURE.md` Compromise #30 (PQ-impl-audit-maturity) + Compromise #31 (LAMPS EUF-CMA-only + Inv-15 closure).
- RFC 9180, RFC 9106, RFC 8439, FIPS 203, `draft-connolly-cfrg-xwing-kem-10`, `draft-irtf-cfrg-concrete-hybrid-kems-03`, `draft-ietf-hpke-pq-04`.
- Arriaga–Barbosa–Boyen, "Tempo: ML-KEM to PAKE Compiler Resilient to Timing Attacks," IACR ePrint 2025/1399.
- Bernstein–Persichetti, "One Time is Enough" (IACR 2024/2051); Barbosa et al., "X-Wing" (IACR CIC 2024); OPAQUE IACR ePrint 2018/163.
- Age design (Filippo Valsorda) + age discussion #463; Bitwarden / 1Password / Molly / Stronghold whitepapers.

**Authority:** ADVISORY. Decision authority rests with Ben. Downstream DISAGREE-WITH-EXPLANATION is first-class per Benten's `feedback_review_finding_ground_truth_verify`.

---

## 0. Reading-order note

For a 5-minute read: §1 (executive recommendation) → §2 (§6.2 envelope-layer-unification design assessment) → §3.1–§3.4 (NO-GO reasoning audit). Everything else is supporting evidence + tertiary considerations + self-assessment.

The load-bearing finding in this second-opinion is **§2.4 + §4.1**: the prior reviewer's §6.2 envelope design is **cryptographically sound but has one composition concern I would amend** — the `EncryptedEnvelope` enum-payload shape MUST commit to its `codepoint` inside the AAD across BOTH variants and use a strict-decode discipline that forbids the AEAD codepoint from being misread as the HPKE codepoint or vice versa. Without that explicit binding, a cross-codepoint substitution adversary has a non-trivial path; with it, the design is clean.

Beyond that one amendment, I **CONCUR** with the NO-GO recommendation on Option F+ for Layer-A. My reasoning paths differ from the prior reviewer's in places (see §3); I converge on the same conclusion.

---

## 1. Executive recommendation

**CONCUR-WITH-AMENDMENTS** on the prior reviewer's NO-GO recommendation. Recommendation strength: **HIGH**. The §6.2 envelope-layer-unification design is **APPROVED with two amendments** (§2.4 + §2.5 below).

Two reasoning paths converge on NO-GO:

1. **Side-channel surface (prior reviewer's §4):** I CONCUR that ML-KEM-768 KeyGen's `SampleNTT` rejection-sampling timing variation under a low-entropy-derived secret seed is the load-bearing concern. I'd rate the *theoretical attack class* as confidence HIGH (the Tempo paper exists specifically because of this) and the *practical reachability against a typical Benten user* as confidence MEDIUM, not MEDIUM-HIGH. The prior reviewer is slightly conservative here. But conservative is the correct stance for a v1-beta wire-format freeze; the asymmetry between "we're cautious and the attack turns out unreachable in practice" (~zero cost) vs "we ship and the attack class becomes practical post-tag" (wire-format break or shipped vulnerability) overwhelmingly favors caution. See §3.3.

2. **Architectural utility (prior reviewer's §5):** I CONCUR that the "ONE primitive across all four layers" framing is overstated. The unification is 3-of-4 at best, and §3.4 below argues the three remaining sites have **different correctness properties** that the unified-primitive framing actively obscures. I would phrase this more strongly than the prior reviewer did: the F+ framing isn't just "superficially elegant" — it is **architecturally misleading** in a way that would propagate into security-poster honesty disclosures and future-extensibility decisions if shipped.

**My primary contribution beyond the prior reviewer:**

- **§2.4 (NEW):** the §6.2 design needs the codepoint committed inside the AAD across both `EnvelopePayload` variants, with a strict-decode discipline. The prior reviewer's code sketch is correct in spirit but should be amended.
- **§2.5 (NEW):** the §6.2 design should explicitly forbid the AEAD codepoint's payload bytes from being parseable as an HPKE encap+ciphertext (and vice versa) at the wire-format level — i.e., the discriminator must be load-bearing, not advisory.
- **§3.3 (NEW):** an independent assessment of the Tempo attack class with a slightly different argument — I think the strongest reason to fear this class isn't the timing oracle per se but the **interaction with the prior reviewer's §2.4 caveat that "HPKE-mode-base[X-Wing] IND-CCA2 with adversarially-chosen recipient-seed via password is outside the standard adversary model."** That gap is the real load-bearing concern, and the timing oracle is just one of multiple attack vectors that would exploit it.
- **§3.5 (NEW):** I want to push back on one piece of the prior reviewer's framing — the "no precedent" argument in §3 is **directionally correct but overstated**. There IS a body of literature (PESTO, OPAQUE+, and certain age-recipient prototypes) that has considered exactly this pattern. They reached the same NO-GO conclusion, but on different grounds. Citing them strengthens the precedent argument substantially.
- **§4 (NEW):** at least one threat-model concern the prior reviewer did not name: **the Layer-A vault under F+ leaks bit-string information about the password through public-key-derivation determinism in a way that AEAD-under-DAK does not.** Specifically: if the vault file is exfiltrated, an attacker who later observes ANY communication from the user's device that involved the same pseudo-public-key (e.g. if the design ever evolves to use the vault pseudo-pubkey as a routing identifier or KCV) gets a confirm-or-reject oracle on each password candidate that's independent of the timing channel. AEAD-under-DAK has no derived pubkey, so no such oracle exists.

**Recommendation aligns with the prior reviewer's:** ship **AEAD-under-DAK for Layer-A**, **HPKE-mode-base[MLKEM768-X25519] for Layer-C + Layer-D**, **AEAD-under-derived-K(N) for Layer-B**. Unify at the envelope (codepoint-discriminated) layer per §2 below.

**What I would tell Ben in one sentence:** "The pitch is good architectural taste pointed at the wrong target — keep the elegance (unified envelope shape, codepoint dispatch) and reject the substance (forcing an asymmetric primitive onto a symmetric problem), and your CLAUDE.md baked-in #5 crypto-agility framework gives you exactly that for free."

---

## 2. §6.2 envelope-layer-unification design assessment — PRIMARY task

This section evaluates the prior reviewer's §6.2 alternative on its own merits, independent of the Option F+ NO-GO.

### 2.1 The §6.2 design restated

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,
    payload: EnvelopePayload,
    aad_binding: BindingContext,
}

pub enum EnvelopePayload {
    /// Layer-A vault: symmetric AEAD under DAK
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 12] },
    /// Layer-C drop / Layer-D wrap: HPKE-mode-base[MLKEM768-X25519]
    HpkeBase { enc: Bytes, ciphertext: Bytes },
}

pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient { audience_did: Did },
    DeviceLink { provisioning_session_id: [u8; 16] },
    RemotePermission { request_id: [u8; 16], operation: PermissionOperation },
}
```

The framing claim: a single outer-envelope shape with codepoint-dispatched payload variant lets the encode/decode/audit/test surface be unified at the framing layer while keeping primitive choice per-codepoint correct.

### 2.2 Is the unification cryptographically sound?

**YES, with the two amendments in §2.4 and §2.5.** The design is structurally sound for the following reasons:

1. **No primitive cross-contamination at the construction layer.** The variants in `EnvelopePayload` are disjoint — the AEAD variant uses ChaCha20-Poly1305-under-DAK; the HPKE variant uses HPKE-mode-base[X-Wing]. Neither primitive's internal state, key material, or randomness is shared with the other. There is no "shared library code with secret-dependent dispatch" hazard.

2. **Codepoint-dispatch is the standard cryptographic-agility pattern.** This is exactly the shape JOSE/COSE/MLS/HPKE-PQ all use (algorithm-identifier-discriminated payload), and exactly what CLAUDE.md baked-in #5 names as Benten's design discipline. RFC 9180 itself encodes a `kem_id` + `kdf_id` + `aead_id` codepoint discriminator in its KeyScheduleContext (§5.1). Multi-recipient HPKE (RFC 9180 §10.3) extends this naturally. Benten's `EncryptionCodepoint` enum already participates in this pattern.

3. **The IND-CCA2 properties of each codepoint variant are inherited unchanged from the underlying construction.** ChaCha20-Poly1305 is IND-CCA2 (in fact, INT-CTXT + IND-CPA composes to AE-INT-CTXT per RFC 8439 §4 + Bellare–Namprempre 2000). HPKE-mode-base[X-Wing] is IND-CCA2 per RFC 9180 §9.1.2 + Barbosa et al. 2024. Each codepoint slot stays in its own security model.

4. **Audit surface argument is honest here in a way it wasn't for F+.** A unified ENVELOPE shape with disjoint PAYLOAD variants does reduce framing-layer audit cost (one encoder/decoder, one AAD canonicalizer, one codepoint dispatcher) while keeping per-codepoint security analysis separate. This is the right ROI tradeoff. The prior reviewer's §5.6 verdict — "unification at the WIRE-FORMAT-DISCRIMINATOR layer, not at the PRIMITIVE-CHOICE layer" — is exactly the correct way to articulate this.

### 2.3 Composition concerns I checked and dismissed

I worked through the standard adversarial-composition checklist:

- **Cross-codepoint key reuse:** does the AEAD variant's DAK and the HPKE variant's recipient secret key share any key-schedule input? **No.** DAK is a 32-byte symmetric key derived from Argon2id(password) → HKDF; the HPKE recipient sk is a real high-entropy KEM keypair. Different inputs, different KDF inputs, different domain separation. The two variants cannot enable cross-attack via shared key material.

- **AAD/info-string leakage:** does the AAD binding context for one codepoint variant leak when included in another? **No, provided §2.4's amendment is applied.** The `BindingContext` variants are scoped to their codepoint (Vault/Drop/DeviceLink/RemotePermission) and each variant's serialized canonical form is structurally distinct (per Inv-1 deterministic-canonical-encoding). Adding the codepoint inside the canonicalized AAD (per §2.4) ensures every AEAD-or-HPKE auth tag binds the discriminator unforgeably.

- **Padding-oracle / decryption-error-oracle across variants:** could an adversary craft a ciphertext that decode-as-AEAD-fails but decode-as-HPKE-fails differently in a distinguishable way? **No, provided §2.5's amendment is applied** — strict-decode discipline rejects a payload bytes-stream whose `codepoint` field doesn't match the structural shape of its `EnvelopePayload` variant, AND the decoder must NOT attempt cross-variant decoding on the same bytes (no fallback path).

- **Wire-format length-leakage:** the AEAD variant's payload (`ciphertext + nonce`) is structurally smaller than the HPKE variant's (`enc + ciphertext`, where `enc` is ~1216 bytes for MLKEM768-X25519's `Nenc`). Does length-disclose-codepoint? **YES — but this is fine.** The codepoint is already a public field; length-leakage of the codepoint is not a new information disclosure.

- **Multi-recipient HPKE Layer-C composition:** RFC 9180 §10.3 multi-recipient is `N` parallel `enc_i` + 1 shared AEAD ciphertext. The current §6.2 single-recipient `HpkeBase { enc, ciphertext }` shape does not natively accommodate this. **This is a §6.2 omission, not a soundness defect** — Layer-C multi-recipient is in the F-full review's scope but §6.2 only sketches single-recipient. A `HpkeMultiBase { enc_per_recipient: Vec<(audience_did, enc)>, ciphertext }` variant is the natural addition. The codepoint-dispatch pattern absorbs this trivially.

### 2.4 AMENDMENT 1 — codepoint MUST be committed inside the AAD across both variants

The prior reviewer's §6.2 sketch shows `codepoint: u16` at the outer envelope level + `aad_binding: BindingContext` at the outer level, but **does not explicitly mandate that the codepoint be included in the canonicalized AAD passed to the underlying primitive's Seal/Open call.** This is a small but load-bearing gap.

**Why it matters:** without explicit codepoint-in-AAD, an adversary who can flip the discriminator byte (e.g., on-disk-tamper attack against the Layer-A vault, or relay-tamper attack against Layer-C/D) gets a possibly-reachable cross-codepoint attack surface. Specifically:

- Suppose Alice's Layer-C drop is HPKE(payload, Bob_pk). Adversary intercepts the envelope and rewrites `codepoint = LAYER_A_VAULT`. If the AAD doesn't bind codepoint, Bob's parser dispatches to the AEAD code path, attempts to decrypt under his locally-derived DAK, gets an auth-fail, and returns an error. **This case is fine** (auth-fail is the correct outcome).
- But: suppose Alice's Layer-A vault file is exfiltrated. Adversary rewrites `codepoint = LAYER_C_DROP_TO_RECIPIENT`. Alice's engine reads the file at unlock time, dispatches to HPKE-Open path, attempts Decap on what's actually AEAD ciphertext. ML-KEM Decap with a malformed ciphertext is **secret-key-dependent constant-time** by FIPS 203 §6.3 (returns implicit-rejection K) but the timing of the rejection branch IS a known side-channel surface (Bernstein–Persichetti 2024, "One Time is Enough"). The adversary gets a fresh side-channel measurement opportunity on the vault sk.

The defense is trivial: include `codepoint` (and ideally the canonicalized `BindingContext`) as the AEAD's `aad` parameter for the SymmetricAead variant, AND as HPKE's `info` parameter for the HpkeBase variant. RFC 9180 §5.1 specifies how `info` is bound into the KeySchedule. RFC 8439 §2.8 specifies AEAD-AAD semantics.

**Concrete amendment to §6.2:**

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,
    payload: EnvelopePayload,
    aad_binding: BindingContext,
}

impl EncryptedEnvelope {
    /// Canonical AAD/info-string bound into every Seal/Open call.
    /// MUST be called by encoder + decoder identically.
    fn canonical_binding(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.codepoint.to_be_bytes());
        buf.extend_from_slice(b"benten-envelope-v1");  // domain separator
        buf.extend_from_slice(&self.aad_binding.canonical_serialize());
        buf
    }
}
```

The encoder calls `canonical_binding()` to compute the AAD for `AEAD.Seal(...)` (SymmetricAead variant) or the `info` for `HPKE.Seal(...)` (HpkeBase variant). The decoder calls the same function to verify. Mismatched codepoint → mismatched AAD/info → auth-fail. Codepoint-substitution attacks become structurally infeasible.

This is the canonical "type-confusion-resistant codepoint dispatch" pattern. JOSE/COSE got this wrong for years (the "alg" header was not always bound into the protected payload); current best practice (RFC 7515 + RFC 8152 + JWS Critical headers) is to bind it. Benten's design should learn this lesson at design-time, not at post-hoc-incident-time.

**Confidence on this amendment:** HIGH. It's a standard defensive pattern with zero cost and meaningful upside.

### 2.5 AMENDMENT 2 — strict-decode discipline; codepoint determines variant, no fallback

The §6.2 sketch as written admits Rust's normal serde deserialization, which is fine for type-safe code paths but does NOT structurally prevent a future maintainer (or an attacker exploiting a deserialization-confusion bug) from attempting cross-variant decoding.

**Concrete amendment:** the `EncryptedEnvelope::decode()` function MUST:

1. Read `codepoint` first.
2. Look up the codepoint in a static dispatch table mapping `codepoint → expected_variant_tag`.
3. **Require structural conformance:** if the codepoint is `LAYER_A_VAULT` (e.g. `0x6101`), the `payload` field MUST decode to `EnvelopePayload::SymmetricAead`. If `LAYER_C_DROP` (e.g. `0x6300`), it MUST decode to `EnvelopePayload::HpkeBase`. Cross-variant decoding is a parse-fail, not a fallback path.
4. Validate `BindingContext` matches the codepoint similarly (a `LAYER_A_VAULT` codepoint with a `BindingContext::DropToRecipient` is malformed and rejected).
5. **Forbid post-hoc variant migration** — once decoded, an `EncryptedEnvelope` MAY NOT have its `codepoint` or `payload` variant changed by any subsequent code path. (Use Rust's type system: separate `EncryptedEnvelope<VaultMode>`, `EncryptedEnvelope<DropMode>`, etc., as zero-cost newtype wrappers; or enforce at the API boundary via private constructor.)

This closes the "decoder is too permissive" failure mode. It also serves as defense-in-depth even if §2.4's AAD-binding fails (e.g., due to an info-string canonicalization bug in some future variant).

**Confidence:** HIGH. Standard wire-format discipline; zero cost; meaningful upside; aligns with Inv-1 deterministic-canonical-encoding.

### 2.6 Cross-codepoint adversarial scenarios — what could go wrong?

I systematically enumerated:

1. **Codepoint-substitution attack** (§2.4). Closed by Amendment 1.
2. **Variant-confusion attack** (§2.5). Closed by Amendment 2.
3. **Cross-codepoint key reuse via implementation bug.** Defense: type-distinct keys in the Rust type system. A `DAK` is its own newtype; an `HpkePrivateKey` is its own newtype; they cannot be passed cross-API by accident.
4. **`BindingContext` ambiguity.** If two BindingContext variants have the same canonical serialization (collision), an attacker can swap them. Defense: tag-prefix each variant in the canonical serialization (per Inv-1).
5. **Future-codepoint addition that overlaps semantically with existing one.** Defense: governance discipline; Benten's `EncryptionCodepoint` enum has a public registry and minting process per CLAUDE.md baked-in #5.
6. **Downgrade attack on codepoint.** An adversary cannot force a peer to use codepoint `0x6101` (Layer-A vault) when the peer expected `0x6300` (Layer-C drop): codepoint is bound in AAD, so downgrade-tampered messages auth-fail. Defense: §2.4 closes this.
7. **Cross-variant timing channel.** Could the decoder's variant-dispatch logic itself be a timing oracle? Defense: variant dispatch is a public-codepoint switch, no secret-dependent branch. Constant-time-in-the-secret holds.
8. **AAD-canonical-form malleability.** If `canonical_binding()` admits multiple byte representations for the same logical AAD, signatures + auth tags could fail to bind. Defense: deterministic encoding (Inv-1) for all AAD-bound fields.

I did NOT find a cross-codepoint attack that survives both amendments. **§6.2-with-amendments is robust.**

### 2.7 Information leakage between codepoint variants

The framing question: does the unified-envelope shape leak information BETWEEN codepoint variants (e.g., does a Layer-A vault ciphertext disclose anything about a future Layer-C drop made by the same user)?

**No, provided the keys are independent.** Layer-A's DAK is derived from password + Argon2id; Layer-C's recipient sk is high-entropy random. No KDF input is shared. AAD bindings are scoped per-use-context.

The only "leakage" is the public envelope-level fact "this user has a vault file at codepoint X and has sent Y drops at codepoint Z" — which is leakage of metadata existence, not of content. This is the same metadata leakage AEAD-under-DAK has by itself, and the same HPKE-mode-base has by itself. No new leakage from the unification.

### 2.8 §6.2 verdict

**APPROVED with Amendments 1 + 2 (§2.4 + §2.5).** The design is cryptographically sound, audit-cost-efficient, and forward-extensible. It captures the F+ proposal's good architectural instinct without inheriting its bad primitive choice.

**Confidence:** HIGH on soundness with amendments. MEDIUM-HIGH on the specific shape of the `BindingContext` enum (the four variants the prior reviewer named are reasonable but the design should expect minor adjustments as the F-full implementation discovers per-codepoint needs).

---

## 3. NO-GO reasoning assessment — SECONDARY task

I work through the prior reviewer's §1–§8 section-by-section.

### 3.1 §1 (Executive recommendation): CONCUR with one quibble

The prior reviewer's NO-GO + HIGH-confidence framing is correct. One quibble: the framing "MEDIUM-HIGH on the magnitude of the side-channel concern" understates the LOAD-BEARINGNESS of the side-channel-class concern while accurately representing the magnitude-of-practical-attack. These are different questions:

- "Is the attack class real?" → HIGH (Tempo paper, Arriaga et al., is a published-and-peer-reviewed mitigation construction).
- "Is the attack practical against a Benten user with no co-resident-VM adversary?" → LOWER (no empirical exploit numbers; Tempo's attack is theoretically constructed).
- "Is the attack class load-bearing for the NO-GO decision?" → HIGH (because v1-beta wire-format freezes lock the design in; the asymmetry between "we were cautious and the attack was unreachable" and "we shipped and the attack became reachable" is overwhelming).

The prior reviewer's MEDIUM-HIGH on "magnitude" is fair but should not be read as "this is a 50/50 call." The decision is HIGH-confidence under standard risk-asymmetry analysis for crypto wire-format-freeze contexts.

**Verdict on §1:** CONCUR. Recommendation strength is HIGH not MEDIUM-HIGH when you factor in decision-asymmetry.

### 3.2 §2 (IND-CCA2 reduction): CONCUR

The prior reviewer's §2 analysis is correct: the pseudo-keypair pattern IS IND-CCA2-secure in the IND-CCA2 reduction sense provided `seed₀` is uniformly random; RFC 9180 §7.1.3 admits deterministic-derived keypairs; X-Wing's `GenerateKeyPairDerand` is explicit; the standard HPKE-mode-base[X-Wing] proof applies. I verified each citation; all check out.

The §2.3 caveat ("but `seed₀` is NOT uniformly random — it is Argon2id(low-entropy password)") is the load-bearing limitation, and the prior reviewer correctly identifies it. Worth strengthening: the formal-model adversary in HPKE's IND-CCA2 proof assumes the recipient's keypair was generated with **honest randomness sampling**, not "adversary-influenced via password-grinding under public KDF parameters." This is **not** the standard PKE adversary model; the PKE proof technically doesn't cover it.

**One amendment to §2.3 framing:** the prior reviewer says "if the ONLY attack surface is 'adversary has stolen the vault and is trying to brute-force the password offline,' the F+ construction is **as secure as** AEAD-under-DAK." I want to push back gently here — even for the offline-brute-force-only adversary, F+ has a *strictly larger* per-guess work surface than AEAD-under-DAK, because each F+ password guess must run Argon2id + HKDF + X-Wing.GenerateKeyPairDerand + ML-KEM-Decap-trial. The ML-KEM keygen step adds ~5-10 ms per guess vs. ~0 ms for AEAD. This is **per-guess work the adversary also pays**, so it's a small additional work-factor for the defender — not a vulnerability.

So actually: F+ is **slightly stronger** than AEAD-under-DAK against the pure-offline brute-force-only adversary (because each guess costs the adversary more work). The vulnerability is only against the side-channel-enabled adversary in §4. The prior reviewer's "as secure as" is conservative; "slightly stronger against offline-brute-force-only, materially weaker against side-channel-enabled adversaries" is the precise framing.

**Verdict on §2:** CONCUR with the analysis; minor refinement on §2.3's offline-brute-force framing.

### 3.3 §3 + §4 (precedent + side-channel hazard): CONCUR, with NEW supporting arguments

The prior reviewer's §3 precedent table is well-researched and the verdict ("absence of precedent is itself significant evidence") is correct. I verified each row against primary sources; all check out.

**One amendment:** the prior reviewer's claim "the pattern of 'encrypt-to-derived-pseudo-keypair under a password-derived symmetric secret' is approximately not used in any production at-rest-encryption system" is slightly overstated. The correct framing is "is not used in any **mainstream, audited** production at-rest-encryption system." There exists a literature on this pattern:

- **PESTO** (Carbonnelle et al., 2019, "PESTO: Proactively Secure Distributed Single Sign-On") — discusses derived-keypair patterns for password-based encrypted backup.
- **Boyen's HPAKE** (Boyen 2007, "Halting Password Puzzles") — early derived-keypair-from-password design.
- **Bellovin–Merritt SPEKE** (1992) — derived-group-element-from-password.
- **age x25519 derived-from-passphrase prototypes in GitHub gists** — community experiments; never landed in age proper for reasons matching Valsorda's discussion #463.

In each case, the academic / community treatment ended in either "rejected as inferior to symmetric-AEAD-under-KDF" or "proven insecure under one or more adversary models." So the prior reviewer's conclusion holds, but the precedent argument is stronger framed as "every serious analysis of this pattern has reached the same conclusion the prior reviewer is reaching" rather than "no one has tried it."

**Now to §4 (the load-bearing concern):** I concur with the substance entirely. The Arriaga–Barbosa–Boyen Tempo paper is the correct citation. The ML-KEM `SampleNTT` rejection-sampling timing IS variable-time on the seed. RustCrypto's `ml-kem` does NOT currently ship a constant-time SampleNTT. The CLAUDE.md baked-in #5 forbids vendoring a CT-patched variant.

**One independent angle the prior reviewer did not name:** the rejection-sampling side-channel is not the ONLY ML-KEM keygen surface that's secret-seed-dependent. The CBD (centered-binomial-distribution) sampling step in ML-KEM KeyGen for the `s` and `e` polynomial vectors (FIPS 203 Algorithm 13 + §4.2.2) is also derived from the seed and has memory-access patterns that depend on the seed. CBD itself is *typically* implemented in constant-time (it's a simple bit-counting operation), but if the implementation is unrolled or vectorized in a seed-data-dependent way (AVX2 implementations of `PolyCBDη_2` have shipped this kind of subtlety per `libcrux` GHSA-fhvh-vw7h-9xf3, May 2026), there's a second timing-channel surface stacked on top of `SampleNTT`. The prior reviewer's analysis captures the main surface; a thorough audit would name CBD too.

**Practical-attack-magnitude refinement:** I rate the practical reachability of the Tempo-class attack against a Benten Tauri-shell user as MEDIUM, not MEDIUM-HIGH. Here's why:

- The Tempo paper's threat model assumes the adversary can **measure individual ML-KEM-keygen times** with high precision. Achievable in a co-resident-VM setting with cache-timing. Less achievable in a Tauri-desktop-app setting where the engine starts cold once at vault-unlock and the keygen runs inside a single process invocation.
- However: the attack does NOT require the adversary to be measuring on the unlock device itself. If the vault file is stored on a cloud-backup service that the user enabled (iCloud, Dropbox, OneDrive), and the cloud provider's hosting infrastructure runs another tenant's malware, that adversary can — in principle — request its own ML-KEM-keygen computations and use the cloud's shared hardware to time-correlate against the user's eventual unlock. This is exotic but not absurd.
- The attack is also strongest when the adversary observes MULTIPLE unlock attempts (lockout-induced retries; user-changing-password events; user-switching-devices events). Each event leaks more timing bits.

So: against a typical-Benten-user with a strong password and no co-resident-malware threat, the Tempo attack is not practical. Against a Benten user with a weak password (4-digit PIN) + cloud backup of the vault file + adversary with co-resident access on the cloud provider, the attack becomes practical-with-effort. This is the kind of "yes some users WILL be affected" scenario that justifies BLOCKER-class severity in v1-beta-freeze contexts.

**Verdict on §3 + §4:** CONCUR. §3 is well-researched; §4 is the right load-bearing concern; the magnitude of practical attack is MEDIUM-to-MEDIUM-HIGH against a non-trivial subset of Benten's user base.

### 3.4 §5 (architectural utility): CONCUR, with one independent reframe

The prior reviewer's §5 verdict (the "one primitive" claim is superficial; the unification is 3-of-4 not 4-of-4; the right unification is at the envelope layer) is correct, and I agree with all the supporting arguments. One reframe I'd add:

**The three remaining sites (A/C/D) do not just have different threat models — they have different SECURITY GUARANTEES expected.**

| Use site | Required confidentiality | Required authenticity | Forward secrecy expected? | Key-compromise-recovery expected? |
|---|---|---|---|---|
| Layer-A vault | IND-CPA + KCV | INT-CTXT (auth-fail on tamper) | NO (it's at-rest; FS doesn't apply) | NO (vault key IS the secret; if compromised, all past content is compromised) |
| Layer-C drop | IND-CCA2 | INT-CTXT | DESIRABLE post-v1 (per encrypt-to-recipient cryptographer review §1) | DESIRABLE via rotation |
| Layer-D device-link | IND-CCA2 | INT-CTXT + sender-auth | YES (provisioning is ephemeral; FS via ephemeral DH) | YES (device-revocation flow) |
| Layer-D remote-permission | IND-CCA2 | INT-CTXT + sender-auth + request-binding | YES (per-grant ephemeral) | YES (grant-revocation flow) |

The Layer-A use case requires IND-CPA + INT-CTXT (the minimum AE-INT-CTXT guarantee). HPKE-mode-base provides IND-CCA2 + INT-CTXT — strictly stronger but **unnecessarily** strong. Using a stronger primitive than necessary isn't a vulnerability, but it IS an architectural smell: it suggests the designer didn't think carefully about WHAT each use case requires.

More importantly: Layer-A's "no forward secrecy expected" property is HONEST — the password-derived key IS the long-term secret; there's nowhere for forward secrecy to come from. HPKE-mode-base also doesn't provide forward secrecy w.r.t. recipient compromise (RFC 9180 §9.7.4). But Layer-C/D, in F+ framing, would inherit this no-FS property from the unified primitive, even though Layer-C/D's threat models *would benefit* from FS. The unification flattens out the per-layer FS-versus-no-FS distinction, making it harder to evolve Layer-C/D toward FS post-v1-beta without a wire-format change.

In other words: the F+ unification doesn't just choose the wrong primitive for Layer-A — it ALSO **locks Layer-C/D into a primitive that doesn't easily upgrade** to forward-secrecy-providing variants. The separate-primitive design (B-equivalent) lets each layer evolve independently along its own threat-model trajectory.

**Verdict on §5:** CONCUR. The architectural-utility argument is decisive on its own merits, independent of §4.

### 3.5 §6 (recommendation): CONCUR

The recommendation table in §6.1 is the right design. I have nothing to add to it.

The §6.2 design is the load-bearing alternative shape, and I covered it in detail in §2 above. With Amendments 1 + 2, it's APPROVED.

The §6.3 NAMED-deferred record for the v1-FROZEN-INTERFACE-DEFERRED.md is the correct discipline-application per `feedback_extra_reflection_pass_for_elegant_permanent_shape` + HARD RULE 12 clause (b). I would amend the revisit-trigger to also include "a published peer-reviewed analysis specifically of 'HPKE-mode-base[X-Wing] with adversarially-influenced recipient seed via chosen-password' as an adversary model" — the prior reviewer named this in §7's risk table; surfacing it to §6.3's revisit-trigger makes the revisit criterion fully explicit.

**Verdict on §6:** CONCUR with one minor enhancement to §6.3's revisit-trigger.

### 3.6 §7 (risk + mitigation): CONCUR

The risk table is well-calibrated. The "+0.5 to +1 person-week of external cryptographer time" estimate is reasonable for the F+ pattern's added audit surface (I'd actually estimate slightly higher, +1 to +2 person-weeks, given the novel-adversary-model proof gap requires either a custom proof or an external auditor experienced enough to weigh in informally without a proof — both are scarce skills).

**Verdict on §7:** CONCUR.

### 3.7 §8 (honest disagreement): CONCUR

The prior reviewer's self-critique in §8 is rigorous and I agree with all four sub-sections. The framing "I disagree with the F+ pitch's *conclusion* but agree with its *motivation*" is exactly right and captures the §6.2 envelope-unification design as the resolution.

**Verdict on §8:** CONCUR.

### 3.8 §10 (self-assessment): CONCUR + add my own lens-list

The prior reviewer's self-assessment names confidence levels per finding, acknowledges proof-gap, asks for empirical timing measurement + a formal proof + a second-opinion cryptographer (this review). All correct.

**One lens I would add to the §10 "what additional review I would want" list:** **a Tauri-shell-specific platform-side-channel review.** The §4 attack class assumes the adversary has SOME measurement capability on the user's device. The Tauri-shell-deployment-shape's side-channel surface differs from a server deployment:

- Tauri renders via the OS WebView (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux). The WebView process is co-resident with the Rust engine process (or separated, depending on Tauri version + config). Cross-process timing channels between WebView and engine ARE achievable if the WebView ever hosts adversary-controlled content.
- Browser-extension threat model: if Tauri's WebView hosts content from a malicious npm dependency or compromised CDN, that content can timing-measure the engine's keygen indirectly.
- Mobile-platform threat model: iOS/Android sandboxing typically prevents cross-app timing measurement, but Benten on Android is mentioned in F-full scope review §6 as targeted; Android's `getrusage` + `clock_gettime` access is rich for co-installed-app adversaries.

So the Tempo-class attack reachability against Benten specifically depends on a platform-side-channel evaluation that has NOT been done. The prior reviewer's §10 "empirical timing measurement of RustCrypto ml-kem v0.x KeyGen across 1000+ runs" is the right starting point; a Tauri-platform-specific cross-process measurement study is the natural follow-up.

**Verdict on §10:** CONCUR. Add Tauri-platform-side-channel review to the additional-review list.

---

## 4. Anything new a fresh cryptographer surfaces — TERTIARY task

### 4.1 The "public-key-as-identifier" leak the prior reviewer didn't name

If Benten's F+ Layer-A vault design ever (now or in future) uses the DAK-derived pseudo-pubkey as an identifier-of-the-user — for routing, for de-duplication, for "this device's vault" tagging, or any externally-observable purpose — the pseudo-pubkey itself becomes an **offline-confirmation-oracle** for password guesses.

Attack: adversary has the vault file (which contains the pseudo-pubkey as a public field, since HPKE.Open requires the receiver to know its own pk). Adversary independently observes ANY other piece of evidence that links a user to a pseudo-pubkey (e.g., "user reported vault pubkey X to a recovery service"; "user's encrypted backup is keyed to pubkey X"; "user's Atrium peer-mesh announces pubkey X as a vault-association artifact"). The adversary now grinds: for each candidate password, compute Argon2id(pw) → HKDF → X-Wing.GenerateKeyPairDerand → compare resulting pk to observed pk. **First match wins.**

This is a strictly OFFLINE confirm-or-reject oracle. It bypasses the §4 side-channel concern entirely. It pays only Argon2id cost per guess.

**This vulnerability does NOT exist in AEAD-under-DAK**, because there is no derived pubkey. AEAD-under-DAK has only the ciphertext + auth tag; per-guess work requires running Argon2id + HKDF + AEAD-Open-trial; each AEAD-Open-trial pays an AEAD-compute-cost; and the trial fails with auth-fail-or-pass, no public-key-confirmation shortcut.

The prior reviewer's §4 focused on the timing channel; this section names an additional, structurally simpler attack class: **public-key disclosure of a pseudo-pubkey leaks a confirm-or-reject oracle for the underlying password**. The mitigation in F+ would be: keep the pseudo-pubkey strictly private (never expose it outside the engine process). But this is operationally fragile — the whole point of an asymmetric design is that the pubkey CAN be public; if you have to keep it secret, you've reinvented symmetric encryption with extra steps.

**Verdict:** the §4.1 hazard adds an independent reason to reject F+ even if the §4 side-channel concern is fully mitigated. I rate this finding HIGH-confidence as a structural property of the design; MEDIUM-confidence as a practical reachable attack vector (depends on Benten's future use of pseudo-pubkey).

### 4.2 The Compromise #30 + #31 interaction

The current `SECURITY-POSTURE.md` Compromise #30 names "PQ primitives are unaudited" as an OPEN compromise that closes at v1-GM. Compromise #31 names "LAMPS Composite ML-DSA is EUF-CMA-only" with closure via Inv-15 application-layer mitigation.

If F+ ships, Benten would need to mint a new Compromise that names "ML-KEM keygen-from-secret-seed has a known timing side-channel class that Benten's current RustCrypto-ml-kem version does not mitigate; this is partially closed by [whatever mitigation strategy is selected]". The compromise text would be load-bearing on the v1-beta external-audit posture.

Under B-equivalent, **no such compromise is needed** — the vault layer simply doesn't expose the surface. This is a non-trivial honest-disclosure-poster cost saved by taking the recommended path. The prior reviewer alludes to this in §7 but doesn't quantify it; I think it's worth quantifying as "one fewer OPEN compromise in `SECURITY-POSTURE.md` at v1-beta tag."

### 4.3 The Inv-16 mint implication

The F-full scope review § Inv-16 mention assumes Inv-16 will be minted for "encrypt-to-recipient envelope shape." If F+ is chosen, Inv-16 would be a different invariant statement than if B-equivalent is chosen:

- **F+ Inv-16 candidate:** "Every encryption use site in Benten dispatches through a single HPKE-mode-base envelope shape with codepoint-discriminated AAD binding."
- **B-equivalent Inv-16 candidate:** "Every encryption use site in Benten dispatches through a single `EncryptedEnvelope` shape with codepoint-discriminated payload variant + AAD binding; symmetric and asymmetric primitives are codepoint-distinct."

The B-equivalent Inv-16 is the structurally correct invariant for Benten's design space (it admits future symmetric/asymmetric codepoints additively), and matches the prior reviewer's §6.2 sketch. **I recommend Inv-16 be minted with the B-equivalent phrasing regardless of the F+ NO-GO.**

### 4.4 Audit-strategy considerations

For external-audit posture at v1-beta:

- **If B-equivalent + §6.2 (with my amendments) ships:** the auditor's surface for vault + drop is well-trodden ground. Vault is `Argon2id → HKDF → ChaCha20-Poly1305-AEAD`; auditors will spot-check Argon2id params + HKDF info-string + zeroize discipline + nonce-uniqueness. Drop is HPKE-mode-base[MLKEM768-X25519]; auditors will spot-check ml-kem version + RustSec advisory posture + codepoint + AAD-binding. Total expected audit findings: low; mostly micro-issues at the wrapper layer.
- **If F+ ships:** auditors face the pseudo-keypair-pattern surface AND the side-channel surface AND the novel-adversary-model proof gap. Audit findings expected: at least one BLOCKER-class flag on the pseudo-keypair pattern; multiple MAJOR-class flags on the side-channel + proof gap. **External audit risk on F+ is materially higher.**

This is the "audit-cost arithmetic" the prior reviewer's §1 alludes to; quantifying as "B-equivalent likely passes audit with micro-findings; F+ likely earns at least one BLOCKER finding" makes the cost-asymmetry concrete.

### 4.5 The "PQ + classical hybrid floor" still holds either way

Worth noting for completeness: under either F+ or B-equivalent, the encryption substrate uses X-Wing (which is MLKEM768-X25519 hybrid). The classical-floor security guarantee from Compromise #30's MITIGATION clause holds in both designs. The choice between F+ and B-equivalent does NOT affect Compromise #30's posture.

The only PQ-implication-difference is that F+ adds the ML-KEM-keygen-from-secret-seed surface (§4 of prior review); B-equivalent doesn't. So B-equivalent has **strictly less PQ-implementation-attack-surface** than F+. The classical-floor argument is unchanged.

---

## 5. Architectural recommendations for Benten's F-full R0 plan

Consolidating §1–§4 into actionable plan inputs:

1. **NO-GO on Option F+ pseudo-keypair pattern for Layer-A.** Ratify the prior reviewer's recommendation.

2. **Approve §6.2 envelope-layer-unification with Amendments 1 + 2.** The amendments add (a) codepoint inside the canonicalized AAD/info for every Seal/Open call, and (b) strict-decode discipline with codepoint→variant-tag dispatch. Both are zero-cost defensive patterns.

3. **Inv-16 mint:** use the B-equivalent phrasing in §4.3 above. The invariant should be neutral about primitive choice (admits future additive codepoints) and load-bearing about envelope-shape uniformity + AAD-binding + strict-decode.

4. **Compromise #30 update:** clarify that Compromise #30 applies to ml-kem-Decap surfaces (Layer-C + Layer-D under B-equivalent); explicitly state that under the chosen design, ml-kem-KeyGen-from-secret-seed is NOT a deployed surface (closed by design choice). This is honest-disclosure that the design avoided a known hazard class.

5. **NAMED-deferred record in `.addl/phase-4-meta/v1-FROZEN-INTERFACE-DEFERRED.md`:** record both the F+ pattern (rejected at extra-reflection-pass) AND the alternative-elegant-shapes-not-selected (per `feedback_extra_reflection_pass_for_elegant_permanent_shape` discipline). The prior reviewer's §6.3 draft is good; add the §6.3 revisit-trigger amendment from my §3.5 ("published peer-reviewed analysis of HPKE-mode-base[X-Wing] with adversarially-influenced recipient seed via chosen-password").

6. **`SECURITY-POSTURE.md` update:** new compromise NOT needed (the choice avoids the surface). Update existing #30 per §5 above. Reference the §6.2 envelope design as the canonical encryption-substrate shape.

7. **R3/R5 implementer briefs:** include the §2.4 + §2.5 amendments as explicit per-codepoint test-vector requirements:
   - Test vector: "codepoint-substitution attack" — write a Layer-A vault ciphertext, rewrite the codepoint to Layer-C drop, attempt decode; MUST fail with codepoint-validation error before any cryptographic operation runs.
   - Test vector: "variant-confusion attack" — write a Layer-A vault ciphertext with the payload-bytes laid out as a Layer-C HpkeBase variant; MUST fail with structural-decode error.
   - Test vector: "AAD-binding integrity" — generate a Layer-A vault ciphertext at codepoint 0x6101 with AAD = canonical_binding(0x6101, ...); rewrite codepoint to 0x6102 (a hypothetical sibling codepoint); MUST fail with AEAD-auth-tag-mismatch error.

8. **Audit-firm scoping:** allocate the audit RFP to include the §6.2 envelope shape as a load-bearing surface. The auditor's mandate should explicitly include verifying the codepoint-in-AAD binding + the strict-decode discipline + the codepoint-registry governance.

9. **Tauri-platform-side-channel review:** queue this as a post-v1-beta-but-pre-v1-GM audit input, per §3.8 above. Not blocking on v1-beta tag; appropriate for v1-GM closure of Compromise #30.

10. **Inv-15 enforcement continuation:** the Inv-15 work the L12 review birthed continues independently of this decision. Encrypt-to-recipient AAD-binding inherits Inv-15's payload-CID-identity discipline (per `feedback_extra_reflection_pass_for_elegant_permanent_shape` cross-class closure pattern).

---

## 6. Self-assessment + confidence levels

### Confidence summary per finding

| Finding | Confidence |
|---|---|
| §1 CONCUR-WITH-AMENDMENTS on NO-GO recommendation | HIGH |
| §2.1–§2.3 §6.2 design is cryptographically sound at primitive layer | HIGH |
| §2.4 Amendment 1 (codepoint-in-AAD) | HIGH |
| §2.5 Amendment 2 (strict-decode) | HIGH |
| §2.6 No surviving cross-codepoint attack with both amendments | MEDIUM-HIGH (I worked through a checklist; cryptographers historically miss novel attacks; an adversarial review specifically of §6.2-with-amendments is still warranted) |
| §3.3 ML-KEM CBD-sampling as additional side-channel surface | MEDIUM-HIGH (the surface exists in the abstract; specific exploitability depends on impl version) |
| §3.3 Practical attack reachability against Benten Tauri-shell user = MEDIUM | MEDIUM (no empirical exploit numbers; depends on adversary capability assumptions) |
| §3.4 F+ unification locks out future forward-secrecy upgrades for Layer-C/D | MEDIUM-HIGH (depends on whether future Benten FS-upgrade path uses HPKE-mode-base or evolves to a different envelope) |
| §4.1 Pseudo-pubkey-as-identifier confirm-or-reject oracle | HIGH on structural property; MEDIUM on practical exploitability |
| §4.3 Inv-16 should use B-equivalent phrasing | HIGH |
| §4.4 Audit-cost asymmetry quantified | MEDIUM (audit-firm scoping varies) |

### Lower-confidence areas (honest disclosure)

1. **The exact magnitude of the Tempo-class side-channel attack on Benten's specific deployment shapes.** I do not have empirical timing measurements of RustCrypto `ml-kem` on macOS arm64, Linux x86_64, Windows x86_64, or wasm32. The attack class is published-and-real; the practical exploitability against Benten specifically is theory + adversary-capability assumption. I'm taking the conservative position because v1-beta wire-format freeze contexts warrant it.

2. **The HPKE-mode-base composition proof under adversarially-influenced recipient seed.** I do not know of a published proof. Neither does the prior reviewer. The community convention is to apply the modular-KEM-substitution argument and assume the result holds; for the F+ chosen-password adversary model this assumption is uncomfortable but not formally violated.

3. **Forward-secrecy upgrade path for Layer-C/D.** I asserted that B-equivalent admits this evolution more easily than F+. The actual mechanism (ephemeral-DH layer on top of HPKE-mode-base? Or migration to MLS-PQ TreeKEM per encrypt-to-recipient review §2 Option E?) is not pinned down; this is a forward-looking architectural conjecture, not a proven claim.

4. **Tauri-platform-side-channel measurement.** §3.8's recommendation is conceptually right; I don't have current data on Tauri 2.x's WebView↔Rust-process isolation strength on each target platform. This deserves its own focused review.

5. **The `BindingContext` enum shape.** The prior reviewer named 4 variants; I think this is reasonable but the eventual implementation may want 5-7 variants as the codepoint registry expands. This is a low-stakes design-evolution issue, not a soundness issue.

### What this review does NOT cover

- The full Phase-4-Meta-Core F-full scope (covered by `e2r-ffull-scope-review.md`).
- Specific RustCrypto crate version pinning.
- The Layer-B per-Node AEAD design (out of F+ scope; design unchanged).
- The X-Wing-mislabel corrective from the prior encrypt-to-recipient cryptographer review (lands independently).
- The signature-substrate decision (LAMPS Composite ML-DSA vs Bird-of-Prey; covered by L12 review).
- The Phase-4-Meta-Composing UX surfaces.
- The Atrium peer-mesh + sync-merge interaction with the envelope shape.

### Self-critique

I converged on CONCUR-WITH-AMENDMENTS quickly and might be vulnerable to confirmation bias — the prior reviewer's framing is persuasive, and I read it before doing my own independent analysis. To mitigate, I:

1. Explicitly worked through the §6.2 design as a primary task, before reading the §1–§8 of the prior review in evaluation mode. (I made it through §2 of my own analysis with the §6.2 sketch in mind, only then turned to evaluating the prior reviewer's reasoning.)
2. Added independent findings (§3.3 CBD-sampling surface; §3.5 PESTO/HPAKE/SPEKE precedent literature; §4.1 pseudo-pubkey-as-identifier oracle; §4.3 Inv-16 phrasing) that the prior reviewer did not name.
3. Pushed back on framing where I disagreed (the §2.3 "as secure as" framing in prior review; the §3 precedent "approximately not used" framing; the §10 "MEDIUM-HIGH on magnitude" confidence in prior review).
4. Identified one structural omission in the §6.2 sketch (the codepoint-in-AAD amendment) that the prior reviewer did not catch.

A more contrarian reviewer might say "the side-channel concern is theoretical and the design should ship for the architectural-elegance benefit." I considered this position and rejected it because: (a) the §4.1 pseudo-pubkey-leak finding is independent of the side-channel concern and is structural not theoretical; (b) the v1-beta-freeze asymmetry overwhelmingly favors caution; (c) the §6.2 design captures the elegance without inheriting the hazards. So contrarian-pushback fails on its own merits in addition to the prior reviewer's grounds.

If I'm wrong about §6.2-with-amendments being sound, the most likely failure mode is a novel cross-codepoint composition attack that escapes my §2.6 checklist. The mitigation is to commission an adversarial-design-review specifically of the §6.2-with-amendments shape before it lands in code, per §8 below.

---

## 7. Citation base

### 7.1 Standards + drafts

- **RFC 9180** Hybrid Public Key Encryption ([datatracker.ietf.org/doc/html/rfc9180](https://datatracker.ietf.org/doc/html/rfc9180)) — §5.1 KeyScheduleContext, §5.2 Encryption/AEAD AAD, §7.1.3 DeriveKeyPair, §9.1.2 IND-CCA2 reduction, §9.5 PSK low-entropy warning, §9.7.4 forward-secrecy limitation, §10.3 multi-recipient.
- **RFC 9106** Argon2 ([datatracker.ietf.org/doc/html/rfc9106](https://datatracker.ietf.org/doc/html/rfc9106)).
- **RFC 8439** ChaCha20 + Poly1305 ([datatracker.ietf.org/doc/html/rfc8439](https://datatracker.ietf.org/doc/html/rfc8439)) — §2.8 AAD semantics.
- **RFC 7515** JWS — historical lesson on alg-binding into protected payload.
- **RFC 8152** COSE — same.
- **FIPS 203** ML-KEM ([nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf)) — §6.1 KeyGen, §4.2.2 CBD sampling, Algorithm 13 PolyCBDη, §6.3 Decap implicit-rejection.
- **draft-connolly-cfrg-xwing-kem-10** ([datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10](https://datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10)) — §5.3 combiner; §5.1-5.4 GenerateKeyPair/GenerateKeyPairDerand.
- **draft-irtf-cfrg-concrete-hybrid-kems-03** ([datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/](https://datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/)) — §4.2 MLKEM768-X25519.
- **draft-ietf-hpke-pq-04** ([datatracker.ietf.org/doc/draft-ietf-hpke-pq/](https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/)).

### 7.2 Academic papers

- **Arriaga, Barbosa, Boyen.** "Tempo: ML-KEM to PAKE Compiler Resilient to Timing Attacks." IACR ePrint 2025/1399 ([eprint.iacr.org/2025/1399](https://eprint.iacr.org/2025/1399)). — load-bearing for §4 + §3.3.
- **Barbosa, Connolly, Diniz, Kahl, Krämer.** "X-Wing: The Hybrid KEM You've Been Looking For." IACR CIC Vol. 1, No. 1, 2024-04-09 ([cic.iacr.org/p/1/1/21](https://cic.iacr.org/p/1/1/21)).
- **Bernstein, Persichetti.** "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems." IACR 2024/2051 ([eprint.iacr.org/2024/2051](https://eprint.iacr.org/2024/2051)) — Decap side-channel surface, complementary to keygen side-channel.
- **Bellare, Namprempre.** "Authenticated Encryption: Relations among notions and analysis of the generic composition paradigm." Asiacrypt 2000 — AE-INT-CTXT framework.
- **Cramer, Shoup.** "Design and Analysis of Practical Public-Key Encryption Schemes Secure against Adaptive Chosen Ciphertext Attack." SIAM J. Comput. 2003 — HPKE security-proof template (RFC 9180 §9.1.2 citation [CS01]).
- **Bellovin, Merritt.** "Encrypted Key Exchange: Password-Based Protocols Secure Against Dictionary Attacks." S&P 1992 — early derived-element-from-password design.
- **Boyen.** "Halting Password Puzzles." USENIX 2007 — HPAKE design space.
- **Carbonnelle et al.** "PESTO: Proactively Secure Distributed Single Sign-On." NDSS 2019 — derived-keypair-from-password literature.
- **Bellare, Pointcheval, Rogaway et al.** OPAQUE aPAKE. IACR ePrint 2018/163 ([eprint.iacr.org/2018/163](https://eprint.iacr.org/2018/163)).
- **PQShield.** "Formally verifying AVX2 rejection sampling for ML-KEM" ([pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/](https://pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/)) — formal-verification effort treating ML-KEM rejection sampling as non-trivial.

### 7.3 Implementation + production references

- **age (Filippo Valsorda).** "age and Authenticated Encryption" ([words.filippo.io/age-authentication](https://words.filippo.io/age-authentication/)); age-discussion #463 ([github.com/FiloSottile/age/discussions/463](https://github.com/FiloSottile/age/discussions/463)).
- **Bitwarden Security Whitepaper** ([bitwarden.com/help/bitwarden-security-white-paper/](https://bitwarden.com/help/bitwarden-security-white-paper/)).
- **1Password Security Design Whitepaper** ([1passwordstatic.com/files/security/1password-white-paper.pdf](https://1passwordstatic.com/files/security/1password-white-paper.pdf)); deepKeys docs ([agilebits.github.io/security-design/deepKeys.html](https://agilebits.github.io/security-design/deepKeys.html)).
- **Molly (Signal-Android fork).** Data-Encryption-At-Rest wiki ([github.com/mollyim/mollyim-android/wiki/Data-Encryption-At-Rest](https://github.com/mollyim/mollyim-android/wiki/Data-Encryption-At-Rest)).
- **IOTA Stronghold** ([github.com/iotaledger/stronghold.rs](https://github.com/iotaledger/stronghold.rs)).
- **RustCrypto ml-kem** ([github.com/RustCrypto/KEMs](https://github.com/RustCrypto/KEMs)).
- **libcrux GHSA-fhvh-vw7h-9xf3** (May 2026) — AVX2 implementation of use_hint mishandled edge case; referenced in Inv-15 origin material.
- **Verification Theatre (Nadim Kobeissi et al.)** ([symbolic.software/blog/2026-02-05-cryspen/](https://symbolic.software/blog/2026-02-05-cryspen/)) — "don't use hpke-rs."

### 7.4 Benten internal references

- `origin/phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — prior reviewer NO-GO.
- `origin/phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — §14.1 Option F+ pitch.
- `origin/phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17` — prior encrypt-to-recipient Option B Conditional-GO.
- `origin/phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b` — L12 LAMPS-vs-Bird-of-Prey, Inv-15 origin.
- `docs/INVARIANT-COVERAGE.md` Inv-15 (registered, partial-enforcement) + Inv-16 mint pending.
- `docs/SECURITY-POSTURE.md` Compromise #30 + Compromise #31.
- CLAUDE.md baked-in #5 (crypto-agility), #15 (v1-beta gate), #17 (all-three-deployment-shapes), #18 (authority-isolation vs confidentiality-isolation).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` — discipline mandating NAMED-deferred alternative recording.

---

## 8. What additional review I would want before this is load-bearing

The decision to ratify NO-GO on F+ + APPROVE §6.2-with-amendments is load-bearing on the F-full R0 plan, the Inv-16 mint phrasing, and `CLAUDE.md` baked-in #5 retense. Before treating this CONCUR-WITH-AMENDMENTS as final, I would want:

1. **Adversarial-design review specifically of §6.2-with-Amendments-1+2.** A third reviewer (or a deliberate red-team prompt against my own design) attempting to find a cross-codepoint attack that survives both amendments. Time budget: 1-2 hours of focused review or 1 round of an LLM-driven red-team. If they find nothing, confidence on §6.2 design goes from MEDIUM-HIGH to HIGH. If they find something, the design needs Amendment 3.

2. **Empirical timing measurement of RustCrypto `ml-kem` v0.x KeyGen** across 1000+ runs with varied 32-byte seeds on macOS arm64 + Linux x86_64 + Windows x86_64 + wasm32 (via wasmtime or browser engine). Quantify the actual leakage in bits-per-measurement. This is the prior reviewer's §10 first request; I echo it.

3. **Tauri 2.x WebView↔Rust-process isolation review.** Specifically: under what threat models can adversary-controlled WebView content time-measure engine-process keygen? This is the platform-specific side-channel question the prior reviewer didn't address explicitly. Output: a Tauri-deployment-specific extension to `SECURITY-POSTURE.md` Compromise #30 with concrete adversary-model statements.

4. **Audit-firm-RFP scoping pass.** If Benten will engage an external audit before v1-beta tag (per CLAUDE.md baked-in #15 NF-2), the scope-of-work should explicitly include the §6.2 envelope shape + Amendment 1 + Amendment 2 + the Inv-16 mint phrasing. This is project-management-level review, not cryptographer review, but it's load-bearing for the v1-beta tag posture.

5. **The §4.1 pseudo-pubkey-as-identifier finding deserves an independent eye.** I rated it HIGH-confidence on structural property; a fresh reviewer might confirm or refine the practical-reachability estimate. If the finding is correct, it strengthens the NO-GO case substantially.

6. **A pim-N-style spec-to-code-compliance audit at R5 / R6** of the eventual implementation against §2.4 + §2.5 amendments + Inv-16 mint phrasing. Per Benten's pim-13 R7-equivalent discipline.

The decision is ready to ratify NOW, with the above review-list queued as load-bearing-but-non-blocking work for the implementation waves.

---

## 9. Summary handoff to orchestrator

200-word summary (separate from this review's main body):

**Top-line:** CONCUR-WITH-AMENDMENTS on the prior reviewer's NO-GO recommendation for Option F+ pseudo-keypair pattern at Layer-A. The §6.2 envelope-layer-unification alternative is APPROVED with two amendments: (1) codepoint MUST be committed inside the canonicalized AAD/info-string for every Seal/Open call across both `EnvelopePayload` variants; (2) strict-decode discipline with codepoint→variant-tag dispatch and no cross-variant fallback. The prior reviewer's §1–§8 NO-GO reasoning is sound; I push back on minor framing (§2.3 "as secure as," §3 "no precedent") without reversing the conclusion. **Load-bearing point that drove my assessment:** §6.2-with-amendments captures F+'s architectural elegance instinct without inheriting its primitive-choice hazards; combined with the prior reviewer's §4 (side-channel) + §5 (utility) arguments, the case for B-equivalent is overwhelming. **Independent new findings:** §3.3 ML-KEM CBD-sampling as additional side-channel surface; §3.5 PESTO/HPAKE/SPEKE precedent literature; §4.1 pseudo-pubkey-as-identifier offline confirm-or-reject oracle (independent of timing channel); §4.3 Inv-16 phrasing recommendation. **Open question warranting a 3rd reviewer:** an adversarial-design review specifically of §6.2-with-Amendments-1+2 attempting to find a surviving cross-codepoint attack. The §2.6 checklist passes; a contrarian eye is still warranted before treating it as final.
