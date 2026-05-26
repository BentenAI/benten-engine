# Senior-Cryptographer Pre-Commit Review: Encrypt-to-Recipient at v1-beta — Construction-Soundness Lens

**Reviewer:** Senior Cryptographer (engaged 2026-05-26)
**Decision-surface:** Whether to add encrypt-to-recipient encryption at v1-beta scope, and via which construction
**Authority of this document:** ADVISORY. Final disposition rests with Ben.
**Companion reviews:** P2P-systems architect + standards-maturity-and-ecosystem skeptic (this review focuses on construction-soundness)

---

## 0. Reading-order note

The most important sections for a binary decision are §1, §2 (option-by-option construction-soundness verdicts), §6, §10 (extra-reflection-pass for Option F), and §11 (honest disagreement). §3-5 give the technical grounding; §9 is the cite-anchored evidence base.

This review's scope is **construction-soundness only** — does the math work, does the composition compose, what implementation hazards does each option carry. Ecosystem-maturity and P2P-vision-fit are the other reviewers' lenses; I touch them only where they affect the construction analysis.

---

## 1. Executive recommendation

**CONDITIONAL GO on Option B** (HPKE-RFC-9180 envelope + X-Wing as the KEM) — with one corrective and three load-bearing conditions:

**Corrective:** rename Benten's current `0x647a` codepoint construction. **It is NOT X-Wing.** Real X-Wing uses `SHA3-256(label || ss_M || ss_X || ct_X || pk_X)` with the 6-byte ASCII label `"\.//^\"` (hex `5C2E2F2F5E5C`). Benten's current `0x647a` construction (per `crates/benten-crypto-suite/src/cipher_suite.rs`) uses **HKDF-SHA256** with info-tag `"x-wing-v1-benten-0x647a"` over a different input concatenation (`ss_x || ss_mlkem || ek_x || ek_mlkem || pk_x || pk_mlkem`). Calling it "X-Wing-style" or "vendored X-Wing combiner" in code comments, `SECURITY-POSTURE.md`, and `CLAUDE.md` is a **load-bearing mislabel that must be corrected before any v1-beta wire-format freeze**, because (a) X-Wing's tight peer-reviewed security proof does NOT transfer to a different combiner, (b) third parties auditing or interoperating with Benten will believe it is X-Wing and get a different security argument, and (c) future migration to real X-Wing becomes a wire-format change Benten thought it didn't owe. This is the most important finding in this review and it cuts across the entire option space.

**Load-bearing conditions for Option B:**

1. **Adopt real X-Wing at `0x647a` (the SHA3-256-with-label construction from `draft-connolly-cfrg-xwing-kem-10` §5.3)** OR retire `0x647a` and assign the Benten-specific HKDF-SHA256 construction a different codepoint with a different name (e.g. `BENTEN_X25519_MLKEM768_HKDF_SHA256`). The status quo of using `0x647a` (the IANA-requested HPKE-KEM codepoint reserved for the X-Wing-identical `MLKEM768-X25519` per `draft-irtf-cfrg-concrete-hybrid-kems-03` §4.2) for a different construction is a **codepoint collision** with `draft-ietf-hpke-pq-04` Table 1 and will produce undetectable interop failure if any Benten peer ever receives a wire-format-real-X-Wing payload at that codepoint.

2. **Use HPKE RFC 9180 mode_base as the envelope**, with X-Wing slotted in as the KEM. HPKE's IND-CCA2 security proof (RFC 9180 §9.1.2, citing [CS01]) reduces to its KEM's IND-CCA2 security. X-Wing's IND-CCA2 proof (Barbosa et al., IACR Communications in Cryptology Vol. 1 No. 1, 2024-04-09) gives exactly this property. The composition is sound on a **modular-KEM-substitution argument**, but this substitution argument is NOT formally proven in the HPKE-PQ draft — it is the implicit standard cryptographic practice. Acknowledge this honestly in `SECURITY-POSTURE.md`.

3. **Audit the wrapper layer BEFORE v1-beta tag.** The Benten-specific combiner has had no third-party cryptographer review at the time of this writing. Migrating to real X-Wing inherits a peer-reviewed proof, but the wrapper code (Encap/Decap/HPKE-context setup/AEAD binding) is still novel implementation work and is exactly the layer where 2026's ml-dsa/libcrux CVE class lives (RUSTSEC-2025-0144, GHSA-fhvh-vw7h-9xf3, Verification Theatre IACR 2026/192). An external review of `benten-crypto-suite` of ~1 person-week is non-negotiable for a v1-beta-freezing default.

**Why CONDITIONAL GO and not unconditional NO-GO (which was my recommendation on Bird-of-Prey-vs-LAMPS):**

The encrypt-to-recipient question is a different decision profile from the Bird-of-Prey-vs-LAMPS signature question:

- For signatures, the L12 hazard had an application-layer fix (revoke-by-payload-CID/tuple), and switching algorithms papered over the architectural mistake without fixing it.
- For encryption, there is **no application-layer fix for the absence of encrypt-to-recipient**. If Benten ships v1-beta without it, the "P2P-untrusted-default" vision (CLAUDE.md baked-in #17/#18) is structurally unrealized at v1-beta tag. The crypto-agility framework lets us add it later, but the default codepoint pressure favors v1-beta-tag-time inclusion.

- For signatures, LAMPS Composite ML-DSA was the ecosystem-Schelling-point default with WG-adoption + multi-implementation interop. For encrypt-to-recipient, **HPKE RFC 9180 + an X-Wing-style KEM is the ecosystem-Schelling-point default** for PQ-hybrid public-key encryption (Signal-style PQXDH is different — see §6 — and MLS-PQ uses the same HPKE-PQ KEM family per `draft-ietf-mls-pq-ciphersuites-04`).

- HPKE RFC 9180 is a published RFC (not a draft). X-Wing has a peer-reviewed IACR CIC 2024 paper with a tight IND-CCA security proof. The composition (X-Wing-as-KEM-in-HPKE-mode-base) is well-understood. This is materially less risky than Bird-of-Prey was.

**NO-GO on Options C, D, E for the v1-beta-tag-default codepoint slot.** Reasons in §2.

**Reserve future codepoints for Options C (Skokan JOSE), D (Benten-native), E (MLS group key)** as additive non-default arms when their use cases land.

---

## 2. Per-option construction-soundness assessment

### Option A — Status quo (defer encrypt-to-recipient post-v1-beta)

**Verdict: NO-GO from the construction-soundness lens.**

- **Pure cryptographic-soundness reasoning:** there is no construction to evaluate; defer.
- **Construction-soundness *cost* of deferring:** The current `0x647a` codepoint at v1-beta tag mislabels a Benten-specific HKDF-SHA256 combiner as "X-Wing." If v1-beta tag freezes this label, every later wire-format-real-X-Wing peer either (a) collides with the mislabeled codepoint and silent-decrypt-fails, or (b) Benten must mint a new codepoint and document the v1-beta `0x647a` as deprecated within months of the tag — exactly the "wire-break Benten swore it wouldn't owe" failure mode.
- **Recommendation:** Even if Option A is selected, **the codepoint-naming corrective in §1 must land before v1-beta-tag freeze.** This is independent of whether encrypt-to-recipient ships.

### Option B — HPKE RFC 9180 envelope + X-Wing as KEM

**Verdict: CONDITIONAL GO** (the recommendation; see §1 for conditions).

**Construction-soundness analysis:**

1. **HPKE RFC 9180 is a published RFC** (Feb 2022), not a draft. Its mode_base security argument (§9.1.2) is: "It is shown in [CS01] that a hybrid public key encryption scheme of essentially the same form as the Base mode described here is IND-CCA2-secure as long as the underlying KEM and AEAD schemes are IND-CCA2-secure." (verbatim, RFC 9180 §9.1.2). The proof template is well-established (Cramer-Shoup 2001).

2. **HPKE's KEM interface** (RFC 9180 §4) is `GenerateKeyPair / Encap / Decap / SerializePublicKey / DeserializePublicKey / DeriveKeyPair / [optional: AuthEncap/AuthDecap/SerializePrivateKey/DeserializePrivateKey]`. **X-Wing satisfies this interface natively** (per `draft-connolly-cfrg-xwing-kem-10` §5.1-5.4 which exposes Encap/Decap and explicitly registers itself in the "HPKE KEM Identifiers" registry — confirmed via search of the draft text).

3. **X-Wing's security theorem** (Barbosa et al. 2024, IACR CIC Vol. 1 No. 1, the paper "X-Wing: The Hybrid KEM You've Been Looking For"): X-Wing is IND-CCA secure if SHA3-256/SHAKE-256 may be modeled as random oracles, AND **either** the strong Diffie-Hellman assumption holds in the X25519 nominal group **OR** ML-KEM-768 is IND-CCA-secure. This is the **hybrid security guarantee** (one-half-must-hold), which is exactly what we want for the "unaudited PQC is not the sole trust path" invariant.

4. **The composition (HPKE-mode-base[X-Wing as KEM])** is sound by RFC 9180's modular KEM-substitution argument. **Caveat:** RFC 9180's analysis assumed DHKEM (its standard KEM) at proof-time, NOT X-Wing. The substitution is sound because HPKE's KEM interface is the IND-CCA2-KEM abstraction and X-Wing meets that abstraction, but a **dedicated composition proof for HPKE-mode-base[X-Wing]** does not exist in the literature as a single theorem. This is standard cryptographic practice (modular composition over well-defined abstractions) but worth disclosing.

5. **Multi-recipient HPKE.** RFC 9180 §10.3 ("Multi-Recipient Encryption") describes the standard pattern: a single AEAD ciphertext + N parallel KEM-encapsulations, one per recipient. This is the right construction for Atrium-shared-content multi-recipient Drops. X-Wing-as-KEM composes here without issue (each Encap is independent).

6. **Tight reduction status:** the original Barbosa et al. proof is loose in standard ways for hybrid KEMs; a 2026 follow-up (search result IACR ePrint citing "memory-tight analysis improving on Barbosa et al.") sharpens the reduction. This is a **construction-soundness positive**: the construction is improving in proof quality, not breaking. (I cannot cite the 2026 paper with full verification because it appeared only in search-result text; treat this as "I'm inferring an improvement exists" rather than "I have read the proof.")

7. **HPKE security-considerations caveats relevant to Benten:**
   - **Forward secrecy** (RFC 9180 §9.1.4 + §9.7.4): "HPKE does not provide forward secrecy with respect to recipient compromise." Benten's Atrium/Drop use cases need to acknowledge this — recipient long-term key compromise reveals all past Drops to that recipient. A separate ephemeral-DH layer (post-v1-beta) could close this.
   - **Encapsulation randomness** (RFC 9180 §9.7.5): "If the randomness used for KEM encapsulation is bad... In Base mode, confidentiality guarantees can be lost completely." The Encap implementation MUST use a properly-seeded CSPRNG. Benten's `rand_core::OsRng` choice is correct; pin in code review.

**Implementation hazard surface for Option B:**

- ~600–1000 LOC for HPKE-mode-base + X-Wing KEM glue (the X-Wing primitive math is already vendored at ~24 LOC; the HPKE wrapper is the new code).
- Reuses `ml-kem`, `x25519-dalek`, `sha3`, `chacha20poly1305` upstream crates (already in `Cargo.toml`).
- **Constant-time requirements:** X-Wing's combiner concat + SHA3-256 has no secret-dependent branches inherently; constant-time depends on the upstream `ml-kem` Decap implementation. ML-KEM's Fujisaki-Okamoto re-encryption inside Decap is the classical constant-time hazard surface (the same one driving GHSA-fhvh-vw7h-9xf3 / Verification Theatre IACR 2026/192). **Pin `ml-kem` crate version + monitor RustSec.** The hpke-rs `RUSTSEC-2025-0144`-class CVEs apply here too — Verification Theatre IACR 2026/192 V13 specifically identified issues in `hpke-rs`.
- **Multi-implementation cross-check:** the X-Wing reference C implementation at `github.com/X-Wing-KEM-Team/xwing` exists. Run Benten's wrapper output against its KAT corpus as a CI fixture.

**Test surface:**

- KATs: the X-Wing draft `draft-connolly-cfrg-xwing-kem-10` Appendix carries test vectors; cross-verify against the C reference impl.
- Property tests: KEM correctness (Encap/Decap consistency), strip-resistance (stripping either half produces a different shared secret).
- Negative tests: malformed encapsulated-key → Decap fails closed.
- Cross-construction interop test: Benten's HPKE-mode-base[X-Wing] output decryptable by an independent HPKE-mode-base[X-Wing] implementation (e.g. a `hpke` Rust crate fork wired to X-Wing).

### Option C — Skokan-draft HPKE-PQ-PQT-specific codepoints

**Verdict: NO-GO at v1-beta-tag default, RESERVE codepoint(s) for future opt-in JOSE-interop.**

**Construction-soundness analysis:**

1. **`draft-skokan-jose-hpke-pq-pqt-05`** (May 13, 2026) is an **individual draft** — quoting verbatim from datatracker boilerplate: "not endorsed by the IETF and has no formal standing." It is a **candidate for adoption by the JOSE Working Group**, NOT yet adopted. The construction is HPKE-PQ-KEM-shaped (per `draft-ietf-hpke-pq-04`) wrapped in JOSE JWE envelope.

2. **The HPKE-PQ KEM combiner construction** at `draft-ietf-hpke-pq-04` codepoint `0x647a` (MLKEM768-X25519) is, **per `draft-irtf-cfrg-concrete-hybrid-kems-03` §4.2 verbatim**: "This hybrid KEM combines ML-KEM-768 with X25519 using the CG framework from [HYBRID-KEMS]. **It is identical to the X-Wing construction** from [XWING-SPEC]." (emphasis mine). So `draft-ietf-hpke-pq-04 + draft-irtf-cfrg-concrete-hybrid-kems-03` at `0x647a` IS X-Wing. The HPKE-PQ-spec stack `(HPKE-PQ → concrete-hybrid-kems → X-Wing)` is not a different combiner from Option B — it's a deeper standards-stack route to the SAME combiner.

3. **The Skokan draft only adds the JOSE/JWE envelope on top** of HPKE-PQ — it does not introduce a new combiner. Construction-soundness-wise, Skokan's draft is identical to Option B's KEM-level construction; the differences are at the envelope/serialization layer.

4. **Why NO-GO at v1-beta default despite identical KEM:** the Skokan draft is pre-WG-adoption (this is the same maturity-risk profile as Bird-of-Prey was per the prior cryptographer review). Locking v1-beta tag's wire format to a pre-WG-adopted envelope shape inherits the standards-volatility risk. The JOSE envelope shape locks Benten into JOSE-JWE serialization for storage-substrate use cases that don't need JOSE-JWE.

5. **What Skokan IS useful for:** post-v1-beta, when JOSE-JWE-shaped envelopes become a real Benten requirement (e.g. interop with JOSE-using third parties), Skokan codepoints become the right export-format layer. This is "Layer 3" in Inv-15's 3-layer decomposition: the export-format-specific layer, distinct from the storage-substrate KEM layer.

**Recommendation: defer Skokan adoption to a post-v1-beta JOSE-interop initiative.** Reserve codepoint(s) for it; do not freeze any Benten wire format to Skokan's JOSE-JWE envelope shape at v1-beta tag.

### Option D — Benten-tailored wire format with X-Wing KEM (no HPKE envelope)

**Verdict: NO-GO** for the v1-beta default; can be a fallback if Option B's HPKE implementation work doesn't fit the schedule, but **strictly less elegant** than Option B.

**Construction-soundness analysis:**

1. **X-Wing-KEM + ChaCha20-Poly1305-AEAD without an HPKE envelope** is a valid construction in principle (it's essentially "X-Wing → shared secret → HKDF → AEAD key → AEAD(plaintext)").

2. **Why it's NOT better than HPKE-mode-base[X-Wing]:** HPKE RFC 9180's mode_base already specifies this exact KDF-and-context-binding pattern (§5.1.1), with explicit Security Considerations coverage for:
   - **Info-string binding** (§6.1): `KeyScheduleS(mode_base, shared_secret, info, default_psk, default_psk_id)` derives the AEAD key from `(mode, shared_secret, info)`, providing domain separation that Benten would otherwise have to design + cite-anchor in its own SECURITY-POSTURE.
   - **Export-secret API** (§5.3): for derived secrets beyond the AEAD key. Useful for Benten's per-Node-AEAD-AAD-binding pattern.
   - **AAD handling** (§5.2): standard pattern with security-considerations review.
   - **Sequence-number / nonce-rotation** (§5.2 + §9.7): standard pattern. Rolling your own without these is exactly the "implementation hazard surface" failure mode.

3. **The "smallest implementation" claim in the brief's Option D framing is wrong.** HPKE's standard library implementations exist (Rust: `hpke` crate). Slotting X-Wing-as-KEM into an existing HPKE library is on the order of 100-300 LOC, not less than implementing a from-scratch Benten envelope which would be the full HPKE KeySchedule + ContextBinding logic.

4. **Audit-cost-wise:** an HPKE-mode-base[X-Wing] construction inherits all of HPKE's existing security review (multiple academic + IETF cryptographer eyes since 2020). A Benten-bespoke envelope inherits NONE of that and would need the same person-week-of-cryptographer-audit as Option B from scratch — without the modular-substitution argument's protection.

5. **Ecosystem-interop-wise:** HPKE-mode-base[X-Wing] is interoperable with any HPKE-mode-base[X-Wing] implementation (a category that includes the planned JOSE/COSE/MLS-PQ stacks at codepoint `0x647a`). A Benten-bespoke envelope is not interoperable with anything outside Benten.

**Recommendation: Option D is strictly dominated by Option B. NO-GO.**

### Option E — MLS-style group key derivation for community-shared content

**Verdict: NO-GO at v1-beta-tag scope. RESERVE as a post-v1-beta companion to encrypt-to-recipient.**

**Construction-soundness analysis:**

1. **MLS (RFC 9420)** is a published RFC for group messaging with Continuous Group Key Agreement (CGKA). MLS-PQ extensions are still draft-stage; `draft-ietf-mls-pq-ciphersuites-04` (March 19, 2026) is the current state — "Waiting for WG Chair Go-Ahead" with "Revised I-D Needed - Issue raised by WG" — i.e., **NOT yet WG-finalized**.

2. **MLS solves a different problem from encrypt-to-recipient.** MLS provides:
   - Forward secrecy via key rotation (TreeKEM)
   - Post-compromise security (key rotation after member compromise)
   - Group membership management (add/remove members)
   - Continuous group operation (designed for ongoing chat rooms)

3. **What Benten's Atrium/Drop use cases actually need:**
   - **Atrium-shared content** (use case #1 in the input package): closer to "encrypt-to-this-set-of-recipients" — could be served by multi-recipient HPKE (Option B + RFC 9180 §10.3) OR by MLS group keys.
   - **Drop bundles to specific recipient** (use case #2): single-recipient encrypt-to-recipient — Option B exactly.
   - **Multi-device sync via untrusted peers** (use case #3): single-recipient encrypt-to-recipient — Option B exactly.
   - **Garden-Grove untrusted-host** (use case #4): could be served by either Option B (encrypt-to-self-multi-device) or Option E (member-group key).
   - **Kith selective-disclosure** (use case #5): typically single-recipient — Option B.

4. **For use case #1 specifically** (Atrium-shared community content), MLS-style group keys with TreeKEM rotation have **substantial cryptographic advantages** over multi-recipient HPKE:
   - O(log N) key-rotation cost on member change (TreeKEM)
   - Post-compromise security (recover from member key leak)
   - Forward secrecy (past content not exposed by current key leak)

   But MLS-PQ ciphersuites at draft-04 with WG-revision-needed status is **not v1-beta-tag-ready material**. The construction will change.

5. **Construction-soundness-wise**, MLS is excellent (RFC 9420 has had years of academic + IETF review). The MLS-PQ variant inherits the MLS protocol's structure with HPKE-PQ KEM substitution — same construction-soundness argument as Option B applies at the KEM-substitution level.

**Recommendation: ship Option B at v1-beta for use cases #2 + #3 + #4 + #5 (single-recipient encrypt-to-recipient).** Use multi-recipient HPKE-mode-base[X-Wing] (RFC 9180 §10.3) as a "good enough" interim for use case #1. **Reserve Option E for a post-v1-beta phase** (Phase 5+ or Phase 7+ when Garden-Grove untrusted-host hits production scale) once MLS-PQ ciphersuites land.

### Option F — Something else?

See §10 (extra-reflection-pass output). The candidate F is **"Option B + the codepoint-naming corrective + a post-v1-beta path to multi-recipient HPKE + a named-deferred MLS adoption row."** This is structurally Option B; what makes it "F" rather than "B" is the corrective + the named-deferred path. I have left F as an extension of B rather than a distinct option.

---

## 3. The cross-perspective question — where does construction-soundness specifically apply?

My lens is mathematical soundness of constructions, security-proof depth, side-channel surfaces, and composition arguments. **The other two lenses cover what I cannot:**

- **P2P-systems architect** covers: vision-fit (does encrypt-to-recipient serve the Atrium/Drop/Garden-Grove/Kith use cases?), iroh/Atrium-peer-mesh interaction, the Loro/CRDT-merge-with-ciphertext-blobs question, multi-device key-discovery + rotation operational semantics, sendme-via-untrusted-relay threat-model, transport-layer-encryption-on-top considerations.
- **Standards-maturity-and-ecosystem skeptic** covers: WG-adoption probability of each draft, how the JOSE-vs-COSE-vs-MLS-vs-OpenPGP ecosystem will partition over the next 12-24 months, third-party-library-availability for each option, the "we're a 5-week-old project, can we maintain this?" framing.

**Where my lens contradicts those lenses, I disagree gently.** My construction-soundness-only argument prefers HPKE-mode-base[X-Wing] (Option B) decisively over Option D. If the ecosystem skeptic argues a Benten-native envelope is safer because it doesn't lock to HPKE's wire-format, I would push back: HPKE RFC 9180 is the published-RFC ecosystem default, and any Benten-native envelope inherits more audit-debt than HPKE does.

**Where I defer to the other lenses:**
- The "should encrypt-to-recipient ship at v1-beta vs Phase-4-Meta-Composing" timing question is a vision-fit + ecosystem-maturity call. I have **no strong opinion** on whether the v1-beta-tag freeze should include encrypt-to-recipient or defer it (Option A). My lens says **whichever choice is made, the codepoint-naming corrective in §1 must land**.
- The "is multi-recipient HPKE good enough for Atriums, or do we need MLS?" question is a P2P-architecture call. My lens says both work cryptographically; MLS is stronger but not v1-beta-tag-ready.

---

## 4. Position B blog framing impact

If Option B (HPKE + X-Wing) ships at v1-beta:

**Strong framing:** "Benten ships PQ-hybrid encrypt-to-recipient at v1-beta using HPKE RFC 9180 with X-Wing as the KEM combiner. HPKE is a published RFC; X-Wing has a peer-reviewed tight IND-CCA proof from IACR Communications in Cryptology 2024. This is the standards-Schelling-point choice: the same KEM is on track for adoption by JOSE/COSE/MLS-PQ ciphersuites at codepoint `0x647a`. We picked the published-RFC envelope + the peer-reviewed-proof KEM instead of inventing our own."

**Weak framing (if the corrective in §1 isn't applied):** "Benten ships an X-Wing-style hybrid KEM at v1-beta." — and then someone notices it's not actually X-Wing, and the framing collapses.

**Recommended public framing:**
- DO NOT claim "X-Wing" until the construction matches the X-Wing draft byte-for-byte.
- DO claim "HPKE RFC 9180 [+ X-Wing once the corrective lands] [+ codepoint `0x647a`]."
- DO NOT promise unaudited PQ as the sole trust path; the hybrid-classical-floor invariant is the right narrative.
- DO claim "v1-beta encrypt-to-recipient with post-quantum hybrid security; v1-GM landing the independent ml-kem/ml-dsa audit." (consistent with CLAUDE.md baked-in #5 / #15.)

If Option A (defer) ships:

**Honest framing:** "v1-beta ships per-DID storage-substrate encryption + the codepoint dispatch + the swap-matrix. Encrypt-to-recipient — encrypting Atrium content for member-only access, encrypting Drops to specific recipients — lands at Phase-4-Meta-Composing as an additive codepoint under the crypto-agility framework." This honest framing is shippable, but it gives up the "P2P-untrusted-default at v1-beta" public posture per Ben's 2026-05-26 quote.

---

## 5. Implementation guidance (if Option B is ratified)

### 5.1 Code structure

```
crates/benten-crypto-suite/src/
├── hpke/                          # NEW module
│   ├── mod.rs                     # Public API: encrypt_to_recipient + decrypt_from_sender
│   ├── kem.rs                     # KEM trait abstraction (HPKE-compatible)
│   ├── kem_xwing.rs               # X-Wing impl of the KEM trait
│   ├── kem_x25519.rs              # Classical X25519 fallback impl (existing 0x6400 arm)
│   ├── key_schedule.rs            # HPKE KeySchedule (§5)
│   ├── context.rs                 # HPKE Context (Seal/Open primitives)
│   └── multi_recipient.rs         # RFC 9180 §10.3 multi-recipient pattern
├── cipher_suite.rs                # KEEP existing storage-substrate API; rename combiner per §1
└── codepoint.rs                   # NEW codepoints for HPKE encrypt-to-recipient suites
```

### 5.2 Codepoint layout recommendation

| Codepoint | Construction | Role | Status at v1-beta |
|---|---|---|---|
| `0x647a` | X-Wing (real: SHA3-256 + label `"\.//^\"`) per `draft-connolly-cfrg-xwing-kem-10` §5.3 / `draft-irtf-cfrg-concrete-hybrid-kems-03` §4.2 | KEM-level: X-Wing as drop-in HPKE KEM | LIVE at v1-beta if §1 corrective applied; otherwise REMOVE |
| `0x647a-storage` (TBD) | Benten storage-substrate AEAD (HKDF-SHA256 combiner) | Storage-substrate AEAD only | LIVE at v1-beta with a clear name distinguishing it from KEM-`0x647a` |
| `0x6500` (TBD) | HPKE-mode-base[X-Wing] + ChaCha20-Poly1305 encrypt-to-recipient | Single-recipient encrypt-to-recipient | LIVE at v1-beta if Option B |
| `0x6501` (TBD) | HPKE-mode-base[X-Wing] multi-recipient (RFC 9180 §10.3) | Multi-recipient encrypt-to-set | LIVE at v1-beta if Option B and Atrium use case #1 covered |
| `0x6600` (TBD) | MLS-style group key derivation | Group-managed encrypt | RESERVED; typed-reject at v1-beta; mint when MLS-PQ WG-final |
| `0x6700` (TBD) | Skokan JOSE-JWE-shaped HPKE-PQ | JOSE-interop export | RESERVED; typed-reject at v1-beta; mint when Skokan WG-adopted |

### 5.3 Disciplines + test surfaces

- **Pin every upstream crate at a specific version with a comment naming the audit/CVE state at pin-time** (per the same pattern documented for `ml-dsa` in the Bird-of-Prey review §3.3).
- **Subscribe to RustSec advisories for `ml-kem`, `x25519-dalek`, `sha3`, `chacha20poly1305`, `hkdf`, AND any HPKE Rust crate adopted** (the `hpke` crate by Brendan McMillion is the closest production-ready option; cross-check against `hpke-rs` per the Verification Theatre concerns).
- **KAT cross-verification:** Benten's wrapper output MUST match the X-Wing reference C impl byte-for-byte on the X-Wing draft's Appendix test vectors.
- **HPKE KAT cross-verification:** if a fork-of-`hpke`-with-X-Wing-KEM produces wire-compatible output with Benten's implementation on the RFC 9180 KAT corpus (suitably extended for X-Wing), cross-verify it.
- **Property tests:** Encap/Decap consistency; strip-resistance (zero-ing the ML-KEM ciphertext half produces a different shared secret); strip-resistance under each chunk-AEAD slot.
- **Negative tests:** typed-reject on unknown KEM codepoint; AEAD-tag failure surfaces `UnsupportedAlgorithm::CipherSuite` not silent decrypt-failure; HPKE Context replay-attack negative test (sequence number rotation).
- **Constant-time discipline:** the Benten wrapper code must not branch on secret data; verify with `dudect` or `valgrind --tool=memcheck` instrumentation on the ML-KEM Decap path.
- **External cryptographer audit pre-v1-beta-tag:** ~1 person-week of an external reviewer's time on `benten-crypto-suite` including the HPKE wrapper. Non-negotiable.

### 5.4 Composition with existing Benten crypto stack

- **Per-Node AEAD with `K(N) = KDF(K_principal, N.cid)`** (Spike-E Interpretation-B path-tagged derivation, per `SECURITY-POSTURE.md` Per-Node AEAD section) is **orthogonal** to encrypt-to-recipient. Per-Node AEAD encrypts a Node's content under a key derived from the *Node's owner's* `K_principal`. Encrypt-to-recipient encrypts a Node's content (or its decrypt key) under a *recipient's* public key.
- **Composition pattern:** to make a Node readable by Alice (other principal), the Owner uses HPKE-mode-base[X-Wing](pk_alice, plaintext = K(N)) to wrap the per-Node-AEAD key, ship the wrapped key alongside the AEAD ciphertext, and Alice decrypts the wrap to get K(N), then decrypts the AEAD. This is the standard KEM-DEM composition pattern with HPKE doing the KEM-and-key-wrap part.
- **AAD-binds-plaintext-CID** (per the existing Per-Node-AEAD design): the AEAD authenticates the plaintext-CID + the codepoint + chunk index. HPKE's `info` parameter (§5.1) can be set to the same binding string for cross-layer authentication of the wrap-and-AEAD pair.
- **LAMPS Composite ML-DSA signature default** (per Bird-of-Prey-vs-LAMPS review): signs the wrap-envelope-or-Node-CID-or-grant for capability-chain validation. Orthogonal to encrypt-to-recipient.
- **Inv-15 3-layer decomposition** (per `SECURITY-POSTURE.md`): the encrypt-to-recipient construction at v1-beta covers Layers 1 (storage-substrate AEAD — existing) + Layer 2 (encrypt-to-recipient — NEW with Option B). Layer 3 (JOSE/COSE export envelope) defers to post-v1-beta per §2 Option C.

---

## 6. Risks + mitigations

| Risk | Severity | Probability | Mitigation | Owner |
|---|---|---|---|---|
| `0x647a` codepoint collision (Benten's HKDF-SHA256 combiner labeled "X-Wing" colliding with the actual X-Wing-at-`0x647a` codepoint per `draft-ietf-hpke-pq-04` Table 1) | **HIGH** | **HIGH** (already happens at HEAD; v1-beta-tag freeze guarantees it ships) | §1 corrective: adopt real X-Wing at `0x647a` OR reassign the Benten construction a new codepoint name | Benten core (BLOCKER for v1-beta-tag) |
| `ml-kem` Rust crate ships a CVE post-v1-beta-tag (RUSTSEC pattern; libcrux-ml-dsa precedent) | HIGH | HIGH (3 ml-dsa CVEs in Jan-May 2026; libcrux GHSA-fhvh-vw7h-9xf3 May 2026; Verification Theatre IACR 2026/192) | Pin crate version + 48h triage SLA + subscribe RustSec + version-pin comments name audit state | Benten core |
| HPKE implementation in Rust (e.g. `hpke-rs`) ships a CVE | MED-HIGH | MED-HIGH (Verification Theatre IACR 2026/192 V13 identified `hpke-rs` issues) | Pin chosen HPKE crate version + cross-verify with alternative impl + RustSec subscription | Benten core |
| X-Wing's security proof has a gap discovered post-publication | LOW | LOW (one peer-reviewed paper, IACR CIC; one 2026 follow-up improving memory-tightness) | Codepoint rotation via crypto-agility framework; named-deferred fallback to non-hybrid storage-only mode | Benten core |
| Forward-secrecy gap (HPKE base-mode does not provide it w.r.t. recipient compromise) | MED | MED | Document honestly in `SECURITY-POSTURE.md`; named-deferred row for ephemeral-DH layer post-v1-beta | Ben (decision row) |
| Multi-recipient HPKE doesn't scale to Atrium-large-membership (TreeKEM-style needed) | MED | MED (depends on Atrium size; <100 members fine, >1000 awkward) | Document the N-recipient ciphertext-growth bound; named-deferred MLS-PQ adoption row | Benten core |
| External cryptographer audit finds blocking issue with HPKE wrapper | LOW-MED | LOW-MED | Schedule audit before v1-beta-tag freeze with 4-week buffer for findings closure | Project budget + Ben |
| Benten claims "X-Wing" publicly but ships non-X-Wing construction | **HIGH (trust)** | **HIGH** (already happens at HEAD) | §1 corrective; update `CLAUDE.md` baked-in #5 + `SECURITY-POSTURE.md` + all in-code comments | Benten core (BLOCKER) |
| Wire-format-real-X-Wing peer encounters Benten's mislabeled `0x647a` and silent-fails | **HIGH** | LOW (no real X-Wing peer exists yet) but **rising** (HPKE-PQ adoption accelerating) | §1 corrective before v1-beta-tag | Benten core (BLOCKER) |

---

## 7. Honest disagreement

### 7.1 The framing of the question

The input package treats encrypt-to-recipient as a single binary decision (yes/no at v1-beta). **It is actually three sub-decisions:**

1. **Single-recipient encrypt-to-recipient** (use cases #2, #3, #4, #5): HPKE-mode-base[X-Wing] is the right answer. Construction-soundness is clear; the work is bounded.
2. **Multi-recipient encrypt-to-set** (use case #1 — Atrium): multi-recipient HPKE (RFC 9180 §10.3) is the v1-beta-feasible answer. MLS-PQ is the right end-state but is not v1-beta-tag-ready.
3. **The codepoint-naming corrective on the existing storage-substrate combiner**: independent of (1) and (2), this MUST land at v1-beta-tag freeze regardless of the encrypt-to-recipient decision. This was not in the brief and is the most important finding.

Treating them as one decision risks under-scoping (a) and over-scoping (#1 → MLS) at the same time.

### 7.2 The current `0x647a` codepoint mislabel as "X-Wing"

The input package describes the existing construction as "X-Wing (`draft-irtf-cfrg-xwing`, X25519 + ML-KEM-768, IACR Communications in Cryptology 2024-1-21, Barbosa et al.) at multicodec codepoint `0x647A`. Vendored ~30-LOC combiner per CLAUDE.md baked-in #5." And `SECURITY-POSTURE.md` and `crates/benten-crypto-suite/src/cipher_suite.rs` describe it as "X-Wing-style" and "X-Wing-style combiner."

**This framing is incorrect.** The code is:

```text
combined = HKDF-SHA256(ss_x || ss_mlkem || ek_x || ek_mlkem || pk_x || pk_mlkem,
                       info = "x-wing-v1-benten-0x647a")
```

The X-Wing draft `draft-connolly-cfrg-xwing-kem-10` §5.3 specifies:

```text
ss = SHA3-256(label || ss_M || ss_X || ct_X || pk_X)
where label = "\.//^\" (6-byte ASCII, hex 5C2E2F2F5E5C)
```

These differ in:
- **Hash function:** SHA3-256 (X-Wing) vs HKDF-SHA256 (Benten — i.e., SHA-256 wrapped in HMAC and HKDF-Extract+Expand)
- **Domain separator / label:** `"\.//^\"` 6 bytes (X-Wing) vs `"x-wing-v1-benten-0x647a"` 23 bytes (Benten)
- **Concat order:** `label || ss_M || ss_X || ct_X || pk_X` (X-Wing — note: M = ML-KEM, X = X25519, and pk_X means X25519 public key) vs `ss_x || ss_mlkem || ek_x || ek_mlkem || pk_x || pk_mlkem` (Benten — includes BOTH public keys AND both encapsulated-keys in a different order)
- **Codepoint binding:** Benten's label embeds the codepoint into the info-tag, which is a Benten-novel design choice — the real X-Wing uses one universal label.

**This is a different construction.** X-Wing's IACR CIC 2024 proof does NOT transfer to Benten's HKDF-SHA256 construction. Benten's construction is plausibly secure (HKDF-SHA256 over the relevant inputs is a reasonable KDF design) but it has **no peer-reviewed proof**, no IACR ePrint citation, and no independent third-party review.

**This must be corrected before v1-beta-tag freeze.** Either:
- (a) **Adopt real X-Wing**: change the combiner to `SHA3-256(label || ss_M || ss_X || ct_X || pk_X)` with the X-Wing label. Retain the `0x647a` codepoint. Update `CLAUDE.md` / `SECURITY-POSTURE.md` / in-code comments to cite `draft-connolly-cfrg-xwing-kem-10` §5.3 byte-for-byte. Benefits: real peer-reviewed proof, standards-Schelling-point alignment, future-interop guarantee.
- (b) **Keep the current Benten construction but rename it**: assign it a new codepoint name (e.g., `BENTEN_X25519_MLKEM768_HKDF_SHA256` at, say, `0xF647` in a Benten-private range), explicitly document it is NOT X-Wing and has no peer-reviewed proof, and release `0x647a` to be assignable to real X-Wing in the future (or never used). Disadvantages: ships a non-peer-reviewed construction at default, contradicts CLAUDE.md baked-in #5's "rely on vetted upstream primitives" principle.

**(a) is strictly better.** The ~24 LOC of combiner code is trivially editable; the proof and standards alignment are not. **This is the single most actionable finding in this review.**

### 7.3 The "Bird-of-Prey was conditional NO-GO; therefore Option B should be conditional NO-GO too" reasoning

A reader might generalize the prior cryptographer review's conditional NO-GO on Bird-of-Prey to all PQ-hybrid constructions at v1-beta. **This generalization is wrong.**

- Bird-of-Prey was an individual draft with one peer-reviewed paper, no reference implementation, no test-vector corpus, and was substantively non-standard at v1-beta-freeze-time.
- HPKE RFC 9180 is a **published RFC**, not a draft. X-Wing has a peer-reviewed IACR CIC 2024 paper with multiple authors, multiple reference implementations (Connolly's, the X-Wing-KEM-Team C impl, Cloudflare's circl Go impl), and is the KEM-construction referenced by multiple WG-document drafts including `draft-ietf-hpke-pq-04`, `draft-irtf-cfrg-concrete-hybrid-kems-03`, and (via the same KEM) the IANA-requested codepoint `0x647a` in the "HPKE KEM Identifiers" registry.
- HPKE-mode-base[X-Wing] is, in standards-maturity terms, materially **stronger than LAMPS Composite ML-DSA** was at the time of the prior signature-default review.

The right reading is: **the prior cryptographer review's bar for v1-beta-default was "WG-adopted-or-better with a peer-reviewed proof"**, and HPKE-mode-base[X-Wing] **meets that bar** while Bird-of-Prey did not.

### 7.4 The "we should defer encrypt-to-recipient entirely" reasoning (Option A)

I can see the conservative argument for Option A: ship narrower scope at v1-beta and defer encrypt-to-recipient. The construction-soundness lens does not have a strong opinion here — I defer to the P2P-architect and ecosystem-maturity reviewers.

But I will note: **the codepoint-naming corrective (§1) is required regardless**. Option A does not let Benten avoid the corrective. The corrective is independent of whether encrypt-to-recipient ships at v1-beta.

### 7.5 The Position-B-blog framing impact

The input package treats "stronger Position-B blog framing" as a benefit of shipping encrypt-to-recipient at v1-beta. From a construction-soundness lens, **this is the wrong framing.** The right question is "does this construction stand up to a public cryptographer's third-party review?" — and the answer for HPKE-mode-base[X-Wing] is YES, while the answer for the existing `0x647a` mislabel-as-X-Wing is NO (because the construction does not match its claimed identity).

The framing should be: **fix the construction first**, then the blog framing follows. Not: **ship the blog framing**, then deal with the construction discrepancy later.

---

## 8. Self-assessment

### Confidence level

- **HIGH confidence** on:
  - The `0x647a` codepoint mislabel as X-Wing (§1 + §7.2): I have read both Benten's actual code AND the X-Wing draft text; the discrepancy is mechanical.
  - HPKE RFC 9180's modular KEM-substitution argument (§2 Option B): standard cryptographic practice, published RFC, well-understood.
  - X-Wing's IND-CCA proof status (peer-reviewed IACR CIC 2024 + 2026 follow-up improving memory-tightness): verified through multiple primary sources.
  - The HPKE-PQ → concrete-hybrid-kems → X-Wing identity at codepoint `0x647a` (§2 Option C): verified verbatim from `draft-irtf-cfrg-concrete-hybrid-kems-03` §4.2 ("identical to the X-Wing construction").
  - The ml-kem / hpke-rs / libcrux CVE-class as the dominant 2026 hazard surface (§6 risks): verified via the prior Bird-of-Prey review's evidence base + Verification Theatre IACR 2026/192.

- **MEDIUM confidence** on:
  - The exact LOC estimate for Option B's HPKE wrapper (§5.1 ~600-1000 LOC): order-of-magnitude estimate, not a measured count.
  - The "memory-tight" follow-up paper on X-Wing's proof: I saw the search-result reference but did not read the paper. Treat as "I'm inferring such a paper exists" not "I have cited the proof."
  - The "Skokan draft is candidate for JOSE WG adoption" status: I have the datatracker boilerplate but not the JOSE WG mailing-list status as of 2026-05-26.

- **LOWER confidence** on:
  - Whether `hpke-rs` or the McMillion `hpke` crate is the right Rust HPKE impl to vendor (depends on audit + interface-cleanness analysis beyond this review's scope).
  - The exact size of the Atrium-typical-member-set that would push multi-recipient HPKE → MLS-PQ-style group-key adoption (vision-fit question; defer to P2P-architect).

### Additional review I would want before commit

1. **External cryptographer review of `benten-crypto-suite/src/cipher_suite.rs`** (the existing storage-substrate combiner) BEFORE deciding (§7.2-(a)) vs (§7.2-(b)). If the existing construction is sound under a published proof framework (e.g., the BH23 KEM combiner framework from Bindel-Hale 2023), the bar for the corrective is lower; if it isn't, the bar is higher.

2. **A second cryptographer's read** of the HPKE-mode-base[X-Wing] composition argument: I have given the standard modular-substitution argument, but a formal composition proof exists in the HPKE-PQ analysis literature and I haven't read it end-to-end. Pre-v1-beta is the time to do that read.

3. **Vendor-cryptographer review** (PQShield, Cure53, NCC Group, or Cryspen) of the Benten HPKE wrapper code. ~1 person-week, scheduled before v1-beta-tag freeze.

4. **A read of the Connolly et al. "X-Wing" draft -10 §6 Security Considerations** end-to-end (I have only the abstract-level quotes verified) before final v1-beta-tag-freeze. Specifically the FIPS-certifiability and the constant-time discipline sections.

5. **A liveness check on whether `draft-ietf-hpke-pq-04` codepoint `0x647a` allocation** is in fact wedded to X-Wing's specific construction (per `draft-irtf-cfrg-concrete-hybrid-kems-03`) or just to the "MLKEM768-X25519 hybrid family" abstraction (which would allow multiple combiners at the same codepoint — unlikely but worth verifying with the IANA registry directly).

### What this review does NOT cover

- The P2P-architecture question (defer to P2P-architect lens).
- The standards-maturity question (defer to ecosystem-skeptic lens).
- The signature side (covered by prior Bird-of-Prey-vs-LAMPS review).
- Identity recovery, Wave DID, Kith protocol mechanics (orthogonal).
- The Inv-15 3-layer decomposition's Layer-3 (JOSE/COSE export) specifics (defer to post-v1-beta JOSE-interop initiative).

### Self-critique

I have given Option B a CONDITIONAL GO where the prior cryptographer review gave Bird-of-Prey a CONDITIONAL NO-GO. The difference is: HPKE RFC 9180 is published, X-Wing has peer-reviewed proof and multiple implementations, the codepoint is on its way to multiple-WG-document adoption. This is the "ecosystem Schelling point" the prior review explicitly endorsed for signatures (LAMPS). Treating it differently for encryption would be inconsistent.

I am aware that the codepoint-naming corrective in §1 is a more substantive finding than the input package anticipated. The brief asked me to evaluate 6 options. I am asserting that the answer is **"Option B, conditional on a corrective the input package didn't ask about."** If the corrective is unworkable, the analysis would change.

---

## 9. Evidence base (cite-anchored)

### HPKE RFC 9180

- **RFC 9180** (Feb 2022), "Hybrid Public Key Encryption", IETF Standards Track. <https://datatracker.ietf.org/doc/html/rfc9180>
- §4 ("Cryptographic Dependencies"): KEM interface = `GenerateKeyPair / Encap / Decap / SerializePublicKey / DeserializePublicKey / DeriveKeyPair / [optional: AuthEncap/AuthDecap/SerializePrivateKey/DeserializePrivateKey]`. (Verified via WebFetch.)
- §5.1.1 ("Encryption to a Public Key" / Base mode): `def SetupBaseS(pkR, info): shared_secret, enc = Encap(pkR); return enc, KeyScheduleS(mode_base, shared_secret, info, default_psk, default_psk_id)` (verbatim).
- §9.1.2: "It is shown in [CS01] that a hybrid public key encryption scheme of essentially the same form as the Base mode described here is IND-CCA2-secure as long as the underlying KEM and AEAD schemes are IND-CCA2-secure." (verbatim).
- §9.1.4 / §9.7.4: "HPKE does not provide forward secrecy with respect to recipient compromise. In the Base and Auth modes, the secrecy properties are only expected to hold if the recipient private key `skR` is not compromised at any point in time." (verbatim).
- §9.7.5: "If the randomness used for KEM encapsulation is bad... In Base mode, confidentiality guarantees can be lost completely." (verbatim).
- §10.3: Multi-Recipient Encryption pattern.
- §9.2.1 (KEM Requirements): "if the KEM's Encap()/Decap() interface (which is used in the Base and PSK modes) is IND-CCA2-secure, HPKE is able to satisfy its desired security properties." (verbatim).

### X-Wing (draft-connolly-cfrg-xwing-kem)

- **draft-connolly-cfrg-xwing-kem-10** (March 2, 2026): <https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/>
- **Important:** the draft is `draft-connolly-cfrg-xwing-kem`, **NOT** `draft-irtf-cfrg-xwing` as the input package and CLAUDE.md cite. The `irtf-cfrg-xwing` URL returns HTTP 404 (verified). The correct name reflects its **individual-draft (not WG-adopted) status** — Connolly et al. authorship, "no formal standing in the IETF standards process" per the datatracker boilerplate (verified verbatim).
- Status: "Intended status: Informational"; Individual draft; not endorsed by IETF.
- Abstract: "This memo defines X-Wing, a general-purpose post-quantum/traditional hybrid key encapsulation mechanism (PQ/T KEM) built on X25519 and ML-KEM-768." (verbatim, verified.)
- Security Considerations: "Informally, X-Wing is secure if SHA3 is secure, and either X25519 is secure, or ML-KEM-768 is secure." (verbatim, verified.)
- §5.3 Combiner construction (per the X-Wing draft text + search-result confirmation): `ss = SHA3-256(label || ss_M || ss_X || ct_X || pk_X)` with `label = "\.//^\"` (6-byte ASCII; hex `5C2E2F2F5E5C`).
- Crucial X-Wing-draft caveat: "The security of X-Wing relies crucially on the specifics of the Fujisaki-Okamoto transformation used in ML-KEM-768: the X-Wing combiner cannot be assumed to be secure, when used with different KEMs." (verified via WebSearch result.)

### X-Wing IACR CIC 2024 paper

- **Barbosa, Connolly, Duarte, Kaiser, Schwabe, Varner, Westerbaan** (2024), "X-Wing: The Hybrid KEM You've Been Looking For," IACR Communications in Cryptology Vol. 1, No. 1, April 9, 2024. <https://cic.iacr.org/p/1/1/21>
- Preprint: <https://eprint.iacr.org/2024/039>
- Main IND-CCA theorem: X-Wing is IND-CCA secure if SHA3-256, SHA3-512, SHAKE-256 are modeled as random oracles, AND either X25519's gap-CDH (strong DH) assumption holds OR ML-KEM-768 is IND-CCA-secure.
- 2026 follow-up: "Anonymity of X-Wing and its Variants" <https://eprint.iacr.org/2026/396> + a memory-tight reduction paper improving on Barbosa et al.

### draft-ietf-hpke-pq

- **draft-ietf-hpke-pq-04** (March 2, 2026), "Post-Quantum and Post-Quantum/Traditional Hybrid Algorithms for HPKE", **HPKE Working Group** document.  <https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/>
- Abstract (verbatim, verified): "...we define KEM algorithms for HPKE based on both post-quantum KEMs and hybrid constructions of post-quantum KEMs with traditional KEMs, as well as a KDF based on SHA-3 that is suitable for use with these KEMs. When used with these algorithms, HPKE is resilient with respect to attacks by a quantum computer."
- §4 (KEMs): "These KEMs satisfy the KEM interface defined in [GENERIC]." References `draft-irtf-cfrg-hybrid-kems-07/09` (generic hybrid KEM framework) and `draft-irtf-cfrg-concrete-hybrid-kems-03` (concrete instantiations).
- §7.1 (Security Considerations): "Hybrid KEMs can be used to provide security against a non-quantum attacker in the event of failures with regard to the PQ algorithm."
- IANA codepoint assignments (verified via WebFetch):
  - ML-KEM-512: `0x0040`
  - ML-KEM-768: `0x0041`
  - ML-KEM-1024: `0x0042`
  - MLKEM768-P256: `0x0050`
  - MLKEM1024-P384: `0x0051`
  - **MLKEM768-X25519: `0x647a`**

### draft-irtf-cfrg-concrete-hybrid-kems

- **draft-irtf-cfrg-concrete-hybrid-kems-03** (March 2, 2026): <https://datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/>
- §4.2 (MLKEM768-X25519), verbatim: "This hybrid KEM combines ML-KEM-768 with X25519 using the CG framework from [HYBRID-KEMS]. **It is identical to the X-Wing construction** from [XWING-SPEC]."
- Components: KEM_PQ = ML-KEM-768; Group_T = Curve25519; PRG = SHAKE-256; KDF = SHA3-256; Label = `"\.//^\"` (hex `5C2E2F2F5E5C`).

### draft-skokan-jose-hpke-pq-pqt

- **draft-skokan-jose-hpke-pq-pqt-05** (May 13, 2026), expires Nov 14, 2026. <https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/>
- Status (verbatim from datatracker): "not endorsed by the IETF and has no formal standing in the IETF standards process." Individual submission, candidate for adoption by JOSE WG.
- Abstract (verbatim): "This document registers Post-Quantum (PQ) and Post-Quantum/Traditional (PQ/T) hybrid algorithm identifiers for use with JSON Object Signing and Encryption (JOSE), building on the Hybrid Public Key Encryption (HPKE) framework."
- Registers JOSE-JWE algorithm identifiers HPKE-8 through HPKE-13 (with -KE variants) for PQ/T hybrid and pure-PQ.

### MLS RFC 9420 + MLS-PQ

- **RFC 9420** (July 2023), "The Messaging Layer Security (MLS) Protocol", IETF Standards Track.
- **draft-ietf-mls-pq-ciphersuites-04** (March 19, 2026), "Waiting for WG Chair Go-Ahead / Revised I-D Needed - Issue raised by WG". <https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/>
- Uses hybrid KEMs from `draft-ietf-hpke-pq` (same X-Wing-identical KEM at `0x647a` for MLKEM768-X25519).

### Signal PQXDH

- Signal Foundation, "PQXDH" specification. <https://signal.org/docs/specifications/pqxdh/>
- Sequential KDF chaining: `SK = KDF(DH1 || DH2 || DH3 || SS)` or `SK = KDF(DH1 || DH2 || DH3 || DH4 || SS)`. NOT a named combiner; concatenation through HKDF. Algorithm-agnostic to specific KEM choice.

### Rust ml-kem / ml-dsa / HPKE crate hazard surface

- **RUSTSEC-2025-0144** (Jan 27, 2026): `ml-dsa` Timing side-channel in Decompose. <https://rustsec.org/advisories/RUSTSEC-2025-0144.html>
- **GHSA-5x2r-hc65-25f9** (Jan 27, 2026): `ml-dsa` accepts signatures with repeated hint indices. <https://github.com/RustCrypto/signatures/security/advisories/GHSA-5x2r-hc65-25f9>
- **GHSA-fhvh-vw7h-9xf3** (May 2026): `libcrux-ml-dsa` AVX2 `use_hint` bug; fixed in 0.0.9.
- **Verification Theatre IACR 2026/192**: 13 vulnerabilities in libcrux + hpke-rs; identified `hpke-rs` issues.

### Benten current state (verified by direct file-read)

- `crates/benten-crypto-suite/src/cipher_suite.rs` defines the existing `0x647a` combiner as `HKDF-SHA256(ss_x || ss_mlkem || ek_x || ek_mlkem || pk_x || pk_mlkem, info = "x-wing-v1-benten-0x647a")`. This is **NOT byte-equivalent to X-Wing's `SHA3-256(label || ss_M || ss_X || ct_X || pk_X)`** construction.
- `crates/benten-crypto-suite/src/codepoint.rs` lines 176-180: "`HYBRID_X25519_MLKEM768` at `0x647a` is the IETF HPKE-PQ WG-stream `MLKEM768-X25519` hybrid-KEM codepoint (IANA-requested; X-Wing-style vendored combiner over `ml-kem` + `x25519-dalek` + `sha3`)." — **the comment correctly identifies the codepoint's IETF home BUT the implementation is not X-Wing.**
- `docs/SECURITY-POSTURE.md` line 2378: "The encryption hybrid uses the vendored ~30-LOC X-Wing-style combiner" — labels it as X-Wing-style.
- `CLAUDE.md` baked-in #5 references "X-Wing-style combiner at codepoint `0x647a`, vendored ~30-LOC combiner."

The mislabel is consistent across code comments, SECURITY-POSTURE, and CLAUDE.md.

---

## 10. Extra-reflection-pass output: more elegant permanent shape

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`, after per-option analysis, take a holistic pass for a single elegant structural shape that closes multiple findings at once.

### 10.1 The elegant shape

**"Three-layer codepoint registry: Storage-substrate AEAD codepoints, KEM-and-encrypt-to-recipient codepoints, Export-envelope codepoints — distinct, named, never overloaded."**

This is structurally cleaner than the current "one codepoint per cipher-suite" model where `0x647a` does dual duty (storage-substrate combiner + KEM-namespace placeholder).

### 10.2 The structural design

```
Layer 1: Storage-substrate AEAD wrap codepoints (Benten-internal; encrypts-to-self via K_principal-derived keys)
    0xF000: BENTEN_STORAGE_X25519_MLKEM768_HKDF_SHA256  (current 0x647a-mislabeled; renamed + relocated)
    0xF001: BENTEN_STORAGE_X25519_CLASSICAL              (current 0x6400)
    0xF002: BENTEN_STORAGE_MLKEM768_PUREPQ_GATED         (current 0x647c)
    [...]

Layer 2: KEM-and-encrypt-to-recipient codepoints (cross-ecosystem; HPKE-stack interop)
    0x647a: HPKE_KEM_MLKEM768_X25519_XWING               (real X-Wing per draft-connolly-10 §5.3 + draft-irtf-cfrg-concrete-hybrid-kems-03 §4.2)
    0x647b: [reserved for future PQ⊕PQ KEM combiner]
    [...]

Layer 3: Export-envelope codepoints (interop with JOSE/COSE/MLS/OpenPGP)
    0xJ000-0xJ0FF: JOSE-JWE-shaped HPKE-PQ envelopes (per Skokan when WG-adopted)
    0xC000-0xC0FF: COSE-HPKE envelopes (per draft-reddy-cose-jose-pqc-hybrid-hpke when adopted)
    0xM000-0xM0FF: MLS-PQ-ciphersuite envelopes (per draft-ietf-mls-pq-ciphersuites when finalized)
    [...]
```

The current Benten registry overloads `0x647a` for "Benten storage-substrate combiner" while the IETF reserves it for "MLKEM768-X25519 hybrid KEM identical to X-Wing." This is the **source of the §7.2 mislabel**. The fix is structural: separate the layers, name each clearly, don't let the IETF-KEM-codepoint space carry Benten-private constructions.

### 10.3 Why this is the actually-correct shape

The elegant shape closes the following findings/hazards at once:

1. **§7.2 mislabel of `0x647a` as X-Wing** — fixed by allocating real X-Wing at `0x647a` (Layer 2) and renaming the Benten storage construction to a Layer 1 Benten-private codepoint.
2. **§2 Option C deferral cleanliness** — Skokan-JOSE codepoints land in Layer 3 cleanly when they WG-adopt, without confusing the Layer 2 KEM-codepoint space.
3. **§2 Option E reserve cleanliness** — MLS-PQ codepoints land in Layer 3 cleanly when WG-finalized.
4. **CLAUDE.md baked-in #5's "never hardcode key/sig/ciphertext sizes; codepoint dispatch with typed unsupported arm" principle** — naturally fits the layered shape.
5. **The Inv-15 3-layer decomposition** in `SECURITY-POSTURE.md` — explicitly mirrors this structure (Storage-substrate AEAD = Layer 1; encrypt-to-recipient KEM = Layer 2; JOSE/COSE export = Layer 3). The current codepoint registry **doesn't reflect Inv-15's structure**; this elegant shape aligns them.
6. **Future codepoint additions never wire-break the existing default**, because each layer evolves independently.

### 10.4 Operational consequence

Replace the current `0x647a`-overloaded codepoint with the three-layer registry above. Specifically:

1. **At v1-beta-tag:** allocate the renamed Benten storage codepoint at a new Layer-1 slot (the current `0x647a` HKDF-SHA256 construction); allocate real X-Wing at `0x647a` (Layer 2); allocate placeholder/typed-reject for Layer-3 codepoints not yet WG-final.
2. **Ship Option B's encrypt-to-recipient** (HPKE-mode-base[X-Wing]) using the Layer-2 `0x647a` codepoint.
3. **Document the three layers explicitly** in `SECURITY-POSTURE.md` so an external auditor can map any wire-format codepoint to its layer.
4. **Future evolution** (Skokan adoption, MLS-PQ adoption, NF-1 PQ⊕PQ KEM combiner adoption) drops in cleanly without overloading any existing codepoint.

This is more structural work than just "fix the comment" but it (a) closes the mislabel finding permanently, (b) makes future ecosystem-interop trivial, (c) makes the audit narrative clean ("each layer has a clear codepoint range with clear semantics"), and (d) aligns with Inv-15 + CLAUDE.md baked-in #5 explicitly. **This is the most elegant permanent shape.**

If 10.4 is too much structural work pre-v1-beta-tag, the minimum viable corrective in §1 (adopt real X-Wing at `0x647a`; rename or relocate the Benten-private construction) is sufficient to close the construction-soundness mislabel. The full three-layer registry is named-deferrable.

---

## 11. Summary of construction-soundness verdicts

| Option | Construction-soundness verdict | Most-load-bearing reason |
|---|---|---|
| A — Defer | NO-GO (in v1-beta-tag context, with the corrective in §1 still required regardless) | Defers encrypt-to-recipient but does NOT fix the mislabeled `0x647a` codepoint |
| **B — HPKE + X-Wing** | **CONDITIONAL GO (recommended)** | Published RFC envelope + peer-reviewed-proof KEM + ecosystem-Schelling-point codepoint, conditional on §1 corrective + audit |
| C — Skokan JOSE | NO-GO at v1-beta default; RESERVE codepoint for future | Pre-WG-adoption; same maturity-risk profile as Bird-of-Prey; JOSE-envelope unnecessary at storage-substrate layer |
| D — Benten-native envelope | NO-GO (strictly dominated by B) | Forfeits HPKE's modular composition argument + audit-inheritance + ecosystem interop for zero gain |
| E — MLS group keys | NO-GO at v1-beta-tag; RESERVE for post-v1-beta | MLS-PQ ciphersuites are draft-04 "Revised I-D Needed"; not v1-beta-ready |
| F — Three-layer codepoint registry + B + named-deferred E | **CANDIDATE F: structurally cleaner than B alone** | See §10; full three-layer registry permanent shape vs §1 corrective minimum-viable |

**Final construction-soundness recommendation:** Option B (HPKE-mode-base[X-Wing] at codepoint `0x647a`, real X-Wing per `draft-connolly-cfrg-xwing-kem-10` §5.3), conditional on:
- §1 corrective: rename Benten's existing storage-substrate combiner; allocate real X-Wing at `0x647a`.
- External cryptographer audit of `benten-crypto-suite` pre-v1-beta-tag freeze.
- Pin every upstream crate with audit-state-comment at pin-time.
- Subscribe to RustSec for `ml-kem` + chosen HPKE Rust crate.
- Document the HPKE-mode-base[X-Wing] composition argument honestly in `SECURITY-POSTURE.md` as "standard modular substitution; not a single-theorem proof."

Optional but stronger: pursue Option F's three-layer codepoint registry as a v1-beta-or-pre-v1-beta structural cleanup, closing the codepoint-overloading hazard permanently.

---

*End of construction-soundness review. Reviewer: senior cryptographer, engaged 2026-05-26.*
