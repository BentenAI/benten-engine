# Option F+ §6.2 envelope-layer-unification — 9-EYES CONSOLIDATED REGISTRY

**Branch:** `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry`
**Role:** CONSOLIDATOR (not 10th reviewer). Output organizes 9 lens reviews into a single decision-ready registry for Ben to ratify before F-full ADDL R0 plan-doc authoring.
**Date:** 2026-05-27
**Tree-state pre-flight:** worktree clean @ `2172cb6d` against `origin/main`.

**Inputs consolidated:**
- L1 1st-cryptographer NO-GO + §6.2 sketch — `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f`
- L2 2nd-opinion cryptographer CONCUR-WITH-AMENDMENTS (Am1+Am2) — `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b`
- L3 adversarial-design CONCUR-WITH-CALIBRATION (Am3-Am6 + minor Am7+Am8) — `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3`
- L4 impl-engineering CONCUR-WITH-IMPL-AMENDMENTS (IMPL-A1..A6 + IMPL-B1..B4 + LE-vs-BE) — `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering @ 4d4aae5f`
- L5 threat-model+audit-readiness AUDIT-READY-IN-DIRECTION (11 Compromise mints + 3 invariants + THREAT-MODEL.md) — `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0`
- L6 privacy/metadata-leak CONCUR-WITH-CALIBRATION + DISAGREE-LOAD-BEARING-FINAL (Am7-Am12 + Sealed-Sender slot + Compromise mint) — `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak @ 986e50bb`
- L7 cross-ecosystem-interop SOUND-BUT-§3.5s-NON-COMPLIANT (Am9 emit-discipline + Am10 DAG-CBOR + HPKE-11-KE name correction) — `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop @ 208f98bb`
- L8 wire-format-stability CONCUR-WITH-EXTENSIONS (Am9-Am16 incl. `#[non_exhaustive]` + escape codepoint + nonce-variant + FS-gap reservation + CodepointLifecycle) — `phase-4-meta-core/option-f-plus-lens-l8-wire-format-stability @ d8d3c41c`
- L9 atrium-integration CONCUR-WITH-CALIBRATION (A1-A5 + dual-CID disagreement) — `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03`
- e2r-ffull-scope-review (background) — `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`

**Authority:** ADVISORY consolidation. Final decisions rest with Ben. Downstream DISAGREE-WITH-EXPLANATION first-class per `feedback_review_finding_ground_truth_verify`. Where reviewers disagree, this document SURFACES the disagreement rather than choosing a side (§5).

**Numbering convention.** Unified amendments numbered U1..UN. Each cites original lens(es) + their original number (e.g. "L2/Am1; L4/(reaffirmed)"). Unified Compromises numbered C32..CNN starting from the next free slot after existing #31. Unified invariants Inv-16..Inv-18.

---

## §1 Executive summary (read this first)

**Top-line consolidation outcome.** All 9 lenses CONCUR that Option F+ pseudo-keypair pattern for Layer-A is **NO-GO**, and that L1's §6.2 envelope-layer-unification is the right *direction*. The aggregate amendment set across the 9 lenses (~46 raw amendments + impl-amendments + observations) consolidates into **28 unified amendments** after dedup, of which **18 are LOAD-BEARING for v1-beta-tag** (wire-format-affecting or codepoint-slot-reservation), **6 are RECOMMENDED v1-beta** (impl-engineering + doc-deliverable), and **4 are NAMED-DEFERRED** with revisit triggers.

**Unified Compromise # count.** 13 new Compromises numbered #32..#44 + 1 extension to #31 (Drop-bundle composition consequence per L5-C9). Of these, #32 (ML-KEM Decap Bernstein-Persichetti) and #43 (envelope metadata leakage) are the two most-load-bearing on external-audit posture.

**Unified invariants.** Inv-16 (envelope-layer-unification, primitive-neutral per L2/L5/L8 phrasing); Inv-17 (hybrid-cryptography-mandatory floor per L5); Inv-18 (codepoint-registry-discipline + metadata-disclosure invariant per L5+L6+L8).

**Disagreements surfaced for Ben.** 4 substantive forks (§5):
1. **Option A (oqs-rs) vs Option B (libcrux-ml-kem)** for ML-KEM impl — L4 recommends libcrux (verified secret-independence); L5 agrees on audit-readiness grounds. **CONSOLIDATOR ASSESSMENT: HIGH on libcrux.**
2. **LE-vs-BE codepoint endianness** in existing `aead.rs` — L3 mandates BE; L4 found existing code uses LE; **CONSOLIDATOR ASSESSMENT: migrate to BE pre-v1-beta-freeze; ~1 wave-day cost.**
3. **L9 dual-CID (plaintext_cid + envelope_blob_cid)** vs **prior P2P-architect F-refinement-2 (recipient_set-in-CID)** — L9 disagrees with prior on Atrium-composition + forkability grounds. **CONSOLIDATOR ASSESSMENT: L9's dual-CID is correct; F-refinement-2 conflates content-identity with authorization-identity.**
4. **L7's `HPKE-11` vs `HPKE-11-KE`** mode mapping — L7 assumes key-encryption mode (Benten wraps K_principal then K(N) bulk-encrypts); needs F-full-design cross-check. **CONSOLIDATOR ASSESSMENT: HPKE-11-KE if Layer-C wraps a CEK; HPKE-11 if HPKE bulk-encrypts directly; verify at R0 §4 design.**

Plus one consolidator-surfaced 5th: **L3/Am4 (sender-DID-in-AAD) vs L6 metadata-leak posture** — Amendment 4 STRENGTHENS authentication while STRICTLY WORSENING metadata privacy (L6 §2.4 row Am4). Resolution: keep Am4 for default codepoint + reserve Sealed-Sender additive codepoint slot per L6/U22. Both win.

**v1-beta-LOAD-BEARING subset recommendation.** 18 amendments + 5 Compromise mints + 2 invariants + 1 wire-format migration must land before v1-beta tag. ~9-10 calendar-weeks per L4 cost estimate (~35-45 wave-days). Compresses to ~7 weeks if R3/R5 implementer briefs absorb all 18 amendments upfront.

**Pattern-induction meta-findings.** 4 cross-lens patterns the individual reviewers missed (§8): (a) Am4 ↔ L6 metadata tension is structural, not incidental; (b) Inv-15 + Inv-16 are sibling invariants targeting different identifier-hazard layers (signature-bundle-CID vs envelope-AAD-shape); (c) `#[non_exhaustive]` + escape-codepoint + per-variant `aad_version` form one coherent permanence package (currently scattered across L8 Am9+Am11+Am14); (d) all 9 lenses cite Compromise #31 (forever-valid drops) as upstream of distinct downstream hazards (Decap-CCA-replay-Bernstein-Persichetti per L3; metadata-archive per L6; recipient-key-rotation per L9; FS-gap per L8) — Compromise #31 deserves consolidated cross-reference treatment.

**Open questions warranting Ben call.** 5 explicit Ben decisions surfaced in §5 + §6: (Q1) libcrux-ml-kem swap; (Q2) BE codepoint migration in aead.rs; (Q3) dual-CID vs recipient-set-in-CID per L9-vs-prior-P2P; (Q4) HPKE-11 vs HPKE-11-KE mode; (Q5) Sealed-Sender as v1-beta-DEFAULT codepoint vs v1-beta-RESERVED-FUTURE-ADDITIVE slot (L6 leans additive; aggressive-privacy stance would mint default at v1-beta).

---

## §2 Unified amendment registry (1..28)

Format per amendment:
- **Unified #** | **Title** | **Origin (lens/original-#)**
- **Statement** (1-2 sentences)
- **Severity** (LOAD-BEARING / RECOMMENDED / MINOR / DEFERRABLE)
- **Wire-affecting?** Y/N | **Impl-only?** Y/N | **Codepoint-reserve?** Y/N
- **Disposition** (v1-beta-LOAD-BEARING / v1-beta-CODEPOINT-RESERVE / v1-GM-DEFER / NAMED-DEFERRED)
- **Depends-on** (other U#)
- **Compromise/Invariant?** | **Doc impact**
- **Confidence** HIGH/MED-HIGH/MED/LOW

### Group A — Construction soundness (Amendments 1-2; both prior-ratified by L1+L2)

**U1 — Codepoint MUST be committed inside the canonicalized AAD/info-string for every Seal/Open call**
- Origin: L2/Am1; reaffirmed L3/Am1, L4 (canonical_binding), L8/Am14 (extends with aad_version), L9 (per-stanza extension), L7/Am9 (refinement: BE codepoint in Benten-internal AAD; ecosystem-adapter outputs translate to ecosystem identifier).
- Statement: The Benten-internal `codepoint: u16` MUST be bound into the AAD (AEAD variant) or `info` (HPKE variant) via a shared `canonical_binding()` function so cross-codepoint substitution attacks fail at auth-tag verification.
- Severity: **LOAD-BEARING**. Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: — (foundation).
- Compromise/Invariant: Inv-16 normative reference. Doc impact: `docs/CRYPTO-CODEPOINTS.md` (new), CRATES-DEEP-DIVE.md.
- Confidence: **HIGH**.

**U2 — Strict-decode discipline; codepoint determines variant; NO cross-variant fallback**
- Origin: L2/Am2; reaffirmed L3, L4, L8 (extends with format-version-also-validated), L7 (extends with cross-ecosystem-adapter version check).
- Statement: Decoder reads codepoint first, looks up in static dispatch table → expected variant tag, structurally rejects mismatched variant. No "lenient parser" fallback. Cross-variant decoding is parse-fail.
- Severity: **LOAD-BEARING**. Wire-affecting: **Y** (decoder behavior is part of wire contract). Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U1 (codepoint dispatch).
- Compromise/Invariant: Inv-16 normative reference. Doc impact: Inv-16 + INTERNALS.md.
- Confidence: **HIGH**.

### Group B — Canonical encoding + AAD shape (Amendments 3-5; L3-originated, reaffirmed by L4/L5/L8/L9)

**U3 — Canonical-serialization length-injectivity via tagged-length-value encoding**
- Origin: L3/Am3 (concrete collision demonstrated §2.1); reaffirmed L4/§2.6 (kani-target IMPL-B4), L5/§4.3 AF-14, L8/§2.9 (TLV value-bytes opaque clarification + tag-byte registry per §2.3), L7/§7.4 (subsumed by DAG-CBOR if Am10 lands).
- Statement: `BindingContext::canonical_serialize()` MUST length-prefix every variable-length field with a fixed-width prefix; the top-level `canonical_binding()` also length-prefixes the TLV body. Closes the JOSE/JWT alg-confusion class.
- Severity: **LOAD-BEARING**. Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U1.
- Compromise/Invariant: Inv-16. Doc impact: INTERNALS.md.
- Confidence: **HIGH** (concrete collision demonstrated).

**U4 — Sender-DID bound into AAD for non-Vault BindingContext variants**
- Origin: L3/Am4 (`sender_did`/`sender_device_did`/`granting_user_did`/`requesting_device_did`); reaffirmed L5/§2.1 T-20 partial-closure, L9 (composes with A5 executor_did).
- Statement: Each non-Vault BindingContext variant MUST bind the sender identity (and where applicable, grantor + requester device identities) into the AAD as a defense-in-depth measure against AAD-belief-drift attacks. Tension with L6 metadata-leak (§5 disagreement 5).
- Severity: **LOAD-BEARING** as defense-in-depth; MEDIUM as immediately-reachable exploit (outer signature layer catches naive cases). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**. NOTE: minting Sealed-Sender additive codepoint (U22) carves out an alternative shape where sender is bound INSIDE ciphertext for metadata-privacy-preferring callers.
- Depends-on: U1, U3 (TLV encodes the additional fields).
- Compromise/Invariant: Inv-16 + #43 metadata-leak disclosure (because Am4 amplifies §6.1 leak). Doc impact: THREAT-MODEL.md, KEY-LIFECYCLE.md.
- Confidence: **HIGH** structural; **MEDIUM** immediate exploit.

**U5 — Replay-window: sealed-at-epoch + valid-until-epoch bound into AAD for DeviceLink + RemotePermission**
- Origin: L3/Am5; reaffirmed L4/§2.8 (clock-skew tolerance 60s; wall-clock semantics), L5/§4.3 AF-16, L9/§5.1 (variant-scoping confirmed correct — Vault + DropToRecipient excluded), L6/U28 refinement (coarse 1-hour buckets + jitter for privacy).
- Statement: DeviceLink + RemotePermission MUST include `sealed_at_epoch_seconds: u64` + `valid_until_epoch_seconds: u64` in BindingContext; decoder MUST `now() <= valid_until + clock_skew_tolerance` check BEFORE invoking primitive. Vault + DropToRecipient EXCLUDED per design (vault is at-rest; drops are forever-valid per Compromise #31).
- Severity: **LOAD-BEARING** for DeviceLink + RemotePermission; MEDIUM-HIGH for reachability (depends on outer-layer enforcement; inner AAD = structural defense-in-depth). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**. U28 (privacy refinement to 1-hour buckets) is RECOMMENDED.
- Depends-on: U1, U3.
- Compromise/Invariant: Inv-16. Doc impact: THREAT-MODEL.md T-18.
- Confidence: **HIGH** structural; **MED-HIGH** practical.

### Group C — Side-channel mitigation (Amendment 6)

**U6 — Bernstein-Persichetti ML-KEM Decap CT-mitigation + Compromise #32 mint**
- Origin: L3/Am6; reaffirmed L4/IMPL-A1 (closes mechanically via libcrux-ml-kem `check-secret-independence`), L5/§4.5 AF-27, L8 (out-of-L8-scope; affirmed cryptographer call).
- Statement: HPKE Decap path MUST use a verified-constant-time ML-KEM impl (libcrux-ml-kem with `check-secret-independence` is the recommended primary mitigation per L4 IMPL-A1); EVEN WITH CT-Decap, mint Compromise #32 for honest disclosure (option-c "both" per L3).
- Severity: **LOAD-BEARING**. Wire-affecting: **N** (impl + disclosure). Impl-only: **Y** (crate-swap path) OR **codepoint-reserve N**.
- Disposition: **v1-beta-LOAD-BEARING** (impl) + **Compromise #32 mint at v1-beta tag**.
- Depends-on: §5 Q1 (libcrux vs RustCrypto ml-kem decision).
- Compromise/Invariant: **#32** mint. Doc impact: SECURITY-POSTURE.md + Cargo.toml.
- Confidence: **HIGH** structural; **MED-HIGH** practical reachability against typical Tauri user.

### Group D — Wire-format pinning (Amendments 7-8 + L8 expansions)

**U7 — Codepoint endianness pinned BIG-ENDIAN on-wire**
- Origin: L3/Am7; L4/IMPL-B1 found existing `aead.rs` uses LE — **conflict surfaced as §5 Q2**; L7/§7.4 (clarifies BE in Benten-internal AAD; ecosystem-adapter outputs translate).
- Statement: Multi-byte integers in §6.2 envelope on-wire MUST be big-endian (network byte order; aligns with RFC 9180 / FIPS 203 / MLS conventions). Includes the `codepoint: u16` field.
- Severity: **LOAD-BEARING** (wire-format-affecting; needs to land pre-freeze). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** + **§5 Q2 Ben call** on migrating existing `aead.rs` LE → BE (~1 wave-day; bump `ENVELOPE_FORMAT_VERSION_V1 → V2` + regen golden vectors).
- Depends-on: —.
- Compromise/Invariant: Inv-16. Doc impact: CRYPTO-CODEPOINTS.md + INTERNALS.md aead.rs commentary.
- Confidence: **HIGH** on finding; **MED-HIGH** on recommendation (depends on G-CORE-3a freeze status).

**U8 — Codepoint registry IANA-disjoint range + cite-drift scanner enforcement**
- Origin: L3/Am8 (minor); promoted to LOAD-BEARING by L8/§6 (Amendment 8 promotion); L5/Inv-L5-3, L7/§5 (codepoint→ecosystem-identifier mapping rows).
- Statement: Mint `docs/CRYPTO-CODEPOINTS.md` registry. Reserve Benten ranges explicitly + coordinate with multicodec.csv maintainers. Cite-drift-detector scanner enforces "every `pub const ... = 0x...;` in crypto-suite has a registry row".
- Severity: **LOAD-BEARING** (per L8 promotion). Wire-affecting: **partial** (no wire bytes change, but governance is permanent). Impl-only: N. Codepoint-reserve: **Y** (registry IS the reservation discipline).
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U7 (BE pinning), Inv-18.
- Compromise/Invariant: Inv-18. Doc impact: `docs/CRYPTO-CODEPOINTS.md` (new).
- Confidence: **HIGH**.

### Group E — Permanence / wire-format-evolution (L8 amendments 9-16)

**U9 — `EnvelopePayload` enum MUST be `#[non_exhaustive]` + typed-reject unknown variants**
- Origin: L8/Am9.
- Statement: `EnvelopePayload` MUST be `#[non_exhaustive]` so future variants (XChaCha20, HpkeMultiBase, HpkeAuth, HpkePsk, MlsApplication, MlsWelcome, CgkaCommit, HpkeAuthSealedSender, etc.) land additively. Unknown variant tag ⇒ typed-reject `EnvelopeError::UnknownPayloadVariant`, NOT panic, NOT silent-skip.
- Severity: **LOAD-BEARING**. Wire-affecting: **N** (Rust attribute) but interacts with consumer-side decode-discipline. Impl-only: **Y** (zero wire-byte impact at v1-beta). Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U2.
- Compromise/Invariant: Inv-16. Doc impact: lib.rs attribute + INTERNALS.md.
- Confidence: **HIGH**.

**U10 — `BindingContext` enum MUST be `#[non_exhaustive]` + same typed-reject discipline**
- Origin: L8/Am10. Reaffirmed L9 (A1 + A4 + A5 add variants/fields).
- Statement: Same as U9 for `BindingContext`. Future variants: DropToGroup, MlsGroupApplication, MlsWelcome, AtriumSync, CapabilityRevocation, SealedSender, MetadataPrivateRouting all land additively.
- Severity: **LOAD-BEARING**. Wire-affecting: **N** at v1-beta. Impl-only: Y. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U9.
- Confidence: **HIGH**.

**U11 — Reserve `0xFFFF` as extended-codepoint escape + `0xFE00..0xFFFE` as experimental range + mint explicit EnvelopeShape codepoint axis**
- Origin: L8/Am11.
- Statement: Reserve `0xFFFF` as varint escape sentinel (v1-beta typed-rejects; v2+ MAY accept) so future u32/u64 codepoints can extend without v2 wire-break. Reserve `0xFE00..0xFFFE` as experimental. Make the EnvelopeShape codepoint axis explicit (4th axis alongside Hash + Sig + Cipher) in V1-FROZEN-INTERFACE.md item 6.
- Severity: **LOAD-BEARING**. Wire-affecting: **Y** (slots permanently reserved). Impl-only: N. Codepoint-reserve: **Y**.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U7, U8.
- Compromise/Invariant: Inv-18. Doc impact: CRYPTO-CODEPOINTS.md + V1-FROZEN-INTERFACE.md item 6.
- Confidence: **HIGH**.

**U12 — Nonce-length-variant discrimination: `SymmetricAead { [u8;12] }` + reserve `SymmetricAeadXNonce { [u8;24] }` future variant**
- Origin: L8/Am12; reaffirmed L4/IMPL-A2 (XChaCha20 nonce-widening recommended for Layer-A NOW, not deferred). **Tension between L8 (reserve XNonce variant for future) and L4 (use XNonce at v1-beta)**; consolidator interprets as: ship `SymmetricAead [u8;12]` for ChaCha20-Poly1305 codepoint per L8 + simultaneously ship `SymmetricAeadXNonce [u8;24]` variant for XChaCha20-Poly1305 codepoint per L4 IMPL-A2. Both at v1-beta. The codepoint-discriminated nonce-length pattern matches HPKE's `Nn` derived-from-aead_id discipline.
- Severity: **LOAD-BEARING**. Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: **Y**.
- Disposition: **v1-beta-LOAD-BEARING** (ship both variants at v1-beta to absorb L4 IMPL-A2 nonce-reuse hazard for Layer-A).
- Depends-on: U9.
- Compromise/Invariant: Inv-16. Doc impact: CRYPTO-CODEPOINTS.md (XChaCha20 codepoint), INTERNALS.md.
- Confidence: **HIGH** (industry consensus on XChaCha20 for keys re-used across many messages).

**U13 — FS-gap honest-disclosure Compromise + reserve MLS-PQ/CGKA/Bird-of-Prey codepoint brackets at v1-beta**
- Origin: L8/Am13.
- Statement: (a) Mint Compromise (consolidated as **#39**) explicitly disclosing Layer-C HPKE-mode-base FS-gap (recipient's long-term sk decrypts forever; 2030-sk-compromise recovers 2026 envelopes). (b) Reserve codepoint brackets `0x6380..0x638F` (MLS-Application), `0x6390..0x639F` (MLS-Welcome), `0x63A0..0x63AF` (CGKA-Commit), `0x63B0..0x63BF` (Bird-of-Prey AKEM), `0x63C0..0x63CF` (draft-prabel). (c) Reserve `EnvelopePayload` + `BindingContext` variant slots for these via U9 + U10.
- Severity: **LOAD-BEARING**. Wire-affecting: **N** (no bytes at v1-beta; only registry rows). Impl-only: N. Codepoint-reserve: **Y**.
- Disposition: **v1-beta-LOAD-BEARING** (reservation is cheap; retrofit politics post-v1-beta is expensive).
- Depends-on: U8, U11.
- Compromise/Invariant: **#39 FS-gap mint**. Doc impact: SECURITY-POSTURE.md + CRYPTO-CODEPOINTS.md.
- Confidence: **HIGH**.

**U14 — `canonical_binding()` versioned via `aad_version: u8` byte; bind aad_version into the AAD itself; TLV tag `0xFF` reserved as canonicalization-extension marker**
- Origin: L8/Am14.
- Statement: First byte of canonical AAD = `aad_version` (v1-beta `0x01`). Bound into AEAD/HPKE auth-tag via being part of the AAD stream. Future canonicalization-shape changes mint `aad_version = 0x02` etc. TLV tag `0xFF` reserved as "extended canonicalization marker" for future structural extensions.
- Severity: **LOAD-BEARING**. Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U1, U3.
- Compromise/Invariant: Inv-16. Doc impact: INTERNALS.md.
- Confidence: **HIGH**.

**U15 — `Did` canonical-serialization pinned to multikey varint codepoint form + `Did::Unknown(u64 multikey_codepoint, Bytes)` typed-rejection variant**
- Origin: L8/Am15 (MINOR).
- Statement: `Did` MUST serialize as `varint(multikey_codepoint) || raw_key_bytes` per multiformats discipline. Future DID methods land as new multikey codepoints. `Did::Unknown` typed-rejection lets v1-beta readers structurally decode unknown multikeys.
- Severity: **MINOR** (depends on Atrium's multikey commitment). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** (cheap; closes future Atrium DID-method drift).
- Depends-on: U3 (TLV-encoded inside BindingContext fields).
- Compromise/Invariant: —. Doc impact: CRYPTO-CODEPOINTS.md DID-method table.
- Confidence: **MED-HIGH** (depends on Atrium did-method roadmap; if `did:key` only forever, this is overkill but harmless).

**U16 — `CodepointLifecycle { Live, Deprecated, Quarantined, Burned }` typed-state + lifecycle transition discipline**
- Origin: L8/Am16 (MINOR).
- Statement: Each codepoint row in registry carries a lifecycle state. `Deprecated` reads admit/writes refuse; `Quarantined` reads admit with warning/writes refuse; `Burned` reads + writes refused. Pre-encode + pre-decode validation enforces. Resolves V1-FROZEN-INTERFACE.md item 6's "supported FOREVER" tension with future cryptanalytic breakthroughs (e.g. ML-KEM-768 broken in 2030 → Quarantined; full break → Burned).
- Severity: **MINOR** (no codepoint is non-`Live` at v1-beta; future-discipline scaffolding). Wire-affecting: **N**. Impl-only: Y. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** (~50 LOC Rust; cheap; preserves long-term-durability story).
- Depends-on: U8.
- Compromise/Invariant: Inv-18. Doc impact: V1-FROZEN-INTERFACE.md item 6 amendment.
- Confidence: **HIGH**.

### Group F — Atrium-integration (L9 amendments A1-A5)

**U17 — Multi-recipient stanza composition: `EnvelopePayload::HpkeMultiBase` variant + cross-stanza substitution defenses**
- Origin: L9/A1 (load-bearing). Reaffirmed by L3/§2.7 (residual concern named), L2/§2.3 row 5 (gap noted), prior-P2P-architect §2.F multi-stanza fallback.
- Statement: Mint `HpkeMultiBase { cek_aead_ciphertext, cek_aead_nonce, stanzas: Vec<HpkeRecipientStanza> }` variant. Cross-stanza substitution defense: AAD per-stanza binds (codepoint, body-CID, sorted recipient-DID-list, sender_did, stanza-index, recipient_key_generation). Mint codepoint `LAYER_C_DROP_MULTI_RECIPIENT = 0x6301`.
- Severity: **LOAD-BEARING** (Atrium's central multi-recipient use case is not serviceable by §6.2 without this). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: **Y**.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U9, U11, U19 (recipient_key_generation).
- Compromise/Invariant: Inv-16. Doc impact: CRYPTO-CODEPOINTS.md, INTERNALS.md.
- Confidence: **HIGH**.

**U18 — Dual-CID model: `plaintext_cid` (graph-stable) vs `envelope_blob_cid` (transport-mutable)**
- Origin: L9/A2 (load-bearing). **DISAGREEMENT with prior-P2P-architect F-refinement-2** (which proposed `Drop-CID = BLAKE3(plaintext || recipient_did_set || context_label)`) — surfaced as **§5 Q3**.
- Statement: Drop bundle MUST have two distinct CIDs. `plaintext_cid` = BLAKE3 over canonical DropBundlePayload (stable across reseal + recipient-set evolution + cipher rotation — what the user's graph references). `envelope_blob_cid` = BLAKE3 over §6.2 EncryptedEnvelope serialized bytes (changes on reseal; iroh-blobs addressing handle). Mapping `plaintext_cid → Vec<envelope_blob_cid>` replicates via Atrium.
- Severity: **LOAD-BEARING**. Wire-affecting: **partial** (envelope shape unchanged; CID-derivation semantics are protocol-spec-affecting). Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** + **§5 Q3 Ben ratification** of L9's dual-CID over prior F-refinement-2.
- Depends-on: U17.
- Compromise/Invariant: —. Doc impact: ARCHITECTURE.md + Drop-bundle composition spec.
- Confidence: **HIGH** (consolidator concurs with L9 reasoning; F-refinement-2 conflates content-identity with authorization-identity).

**U19 — Recipient-key-rotation generation binding: `recipient_key_generation: u32` in DropToRecipient + HpkeRecipientStanza**
- Origin: L9/A3 (load-bearing).
- Statement: Bind `recipient_key_generation: u32` into `BindingContext::DropToRecipient` + per-stanza `HpkeRecipientStanza` so a recipient with N historical sks dispatches in O(1) and detects "envelope sealed to a generation I no longer have" as explicit-failure. Specify recipient-side key-retention policy (≥1-year grace, vault-encrypted). Amend Compromise #31 to clarify "forever-valid" means "within retention window" (consolidated as Compromise #31 extension per L5-C9 + L9-A3).
- Severity: **LOAD-BEARING** for 6-months-offline scenario (the central Compromise #31 use case). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U17.
- Compromise/Invariant: **Extension to existing Compromise #31** (per L5-C9 + L9). Doc impact: SECURITY-POSTURE.md + KEY-LIFECYCLE.md.
- Confidence: **HIGH**.

**U20 — K_principal-generation tracking: `k_principal_generation: u32` in Layer-B AEAD AAD + Vault BindingContext + Atrium-replicated rotation log**
- Origin: L9/A4 (load-bearing).
- Statement: Bind `k_principal_generation: u32` into Layer-B per-Node AEAD AAD and into `BindingContext::Vault`. Layer-A vault stores K_principal generations as `HashMap<u32, [u8;32]>` under DAK. Mint `KPrincipalRotation { generation, rotated_at, rotated_by_device_did, reason }` Atrium-replicated Node.
- Severity: **LOAD-BEARING** (rotation isn't v1-beta-Day-One but interface-freezes wire format). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U1, U3.
- Compromise/Invariant: Inv-16. Doc impact: KEY-LIFECYCLE.md.
- Confidence: **HIGH**.

**U21 — `PermissionOperation::ExecuteWorkflow` variant for hyper-scaling rented-compute use case**
- Origin: L9/A5 (load-bearing for hyper-scaling; MEDIUM for v1-beta-day-one but interface-freeze argues HIGH).
- Statement: Extend `PermissionOperation` enum with `ExecuteWorkflow { workflow_cid: Cid, input_node_cids: Vec<Cid>, max_decrypt_count: u32, result_recipient_pubkey: HybridKemPubKey, executor_did: Did }`. Runtime-enforcement is downstream but AAD-bound scope is auditable post-hoc.
- Severity: **LOAD-BEARING** (additive variant; reserving shape NOW prevents wire-break at hyper-scaling). Wire-affecting: **Y**. Impl-only: N. Codepoint-reserve: **Y** (variant slot).
- Disposition: **v1-beta-LOAD-BEARING** (reservation only; full impl post-v1-beta).
- Depends-on: U10 (#[non_exhaustive] on BindingContext), U4 (composes with sender-DID binding), U19 (composes with recipient_key_generation).
- Compromise/Invariant: —. Doc impact: V1-FROZEN-INTERFACE.md item 15 (AuthorizationGrant).
- Confidence: **HIGH** for shape; **MED** for v1-beta-day-one implementation depth.

### Group G — Privacy / metadata-leak (L6 amendments)

**U22 — Sealed-Sender additive codepoint slot LOCKED at v1-beta; implementation post-v1-beta**
- Origin: L6/Am7 (NEW load-bearing).
- Statement: Mint codepoint `DROP_TO_RECIPIENT_SEALED_SENDER = 0x6510` (sibling to `LAYER_C_DROP = 0x6500/0x6300`) at v1-beta registry. Variant binds sender-DID INSIDE ciphertext (via HPKE-mode-auth's psk_id or via sender-DID-in-inner-payload + post-decrypt-verify), AAD carries only audience + coarse epoch. Implementation deferred to post-v1-beta (G-CORE-PRIVACY-1 wave). The slot lock at v1-beta prevents wire-format break.
- Severity: **LOAD-BEARING** at slot-reservation (v1-beta); RECOMMENDED at implementation (v1-GM defer ok). Wire-affecting: **partial** (codepoint registry row at v1-beta; no bytes shipped). Impl-only: N. Codepoint-reserve: **Y**.
- Disposition: **v1-beta-CODEPOINT-RESERVE** + **v1-GM-DEFER for implementation**. NOTE: **§5 Q5 Ben call** on whether to mint Sealed-Sender as v1-beta-DEFAULT codepoint instead of additive slot (L6 leans additive; aggressive-privacy stance ships default at v1-beta).
- Depends-on: U8 (codepoint registry), U9 (variant slot).
- Compromise/Invariant: **#43 metadata-leak disclosure**, Inv-18 metadata-disclosure invariant. Doc impact: V1-FROZEN-INTERFACE-DEFERRED.md Row D-SS-1.
- Confidence: **HIGH** that Sealed-Sender is the right additive shape; **MED** on additive-vs-default-at-v1-beta call.

**U23 — Per-relay-unlinkability: transport-layer re-blinding outer wrapping**
- Origin: L6/Am8 (NEW).
- Statement: At iroh-blobs transport boundary, wrap EncryptedEnvelope in `TransportEnvelope { transport_blinded_id: [u8;32], envelope: EncryptedEnvelope }` with per-hop salt drawn from Noise-protocol session key. Two adjacent relays see different transport-IDs for same inner envelope.
- Severity: **MED-HIGH**. Wire-affecting: **partial** (transport-layer; outside §6.2 envelope itself). Impl-only: N at transport. Codepoint-reserve: N.
- Disposition: **v1-beta-CODEPOINT-RESERVE (shape lock)** + **post-v1-beta impl at iroh-transport-boundary**.
- Depends-on: —.
- Compromise/Invariant: #43. Doc impact: V1-FROZEN-INTERFACE-DEFERRED.md transport row.
- Confidence: **MED-HIGH** on direction; **MED** on v1-beta-critical-path effort.

**U24 — Padding to fixed-size-class buckets per codepoint**
- Origin: L6/Am9 (NEW).
- Statement: Every EncryptedEnvelope MUST pad to next size-class bucket per codepoint (Layer-A 4KiB/16KiB/64KiB/256KiB/1MiB; Layer-C 1KiB/4KiB/16KiB/64KiB/256KiB/1MiB; Layer-D fixed 1KiB). Padding bytes are AEAD-encrypted with random fill (NOT zero-fill — compression side-channel). Recipient strips via length-prefix INSIDE ciphertext.
- Severity: **MED-HIGH**. Wire-affecting: **Y** (envelope size profile is wire-observable). Impl-only: N. Codepoint-reserve: **Y** (bucket shape locked).
- Disposition: **v1-beta-CODEPOINT-RESERVE (bucket SHAPE)** + **post-v1-beta refinement of specific values after measurement**.
- Depends-on: —.
- Compromise/Invariant: #43. Doc impact: V1-FROZEN-INTERFACE-DEFERRED.md Row D-PAD-1.
- Confidence: **HIGH** on approach; **MED** on specific bucket values (needs Spike G + workload measurement).

**U25 — Multi-stanza per-recipient-unlinkable copies (Signal Sealed Sender V2 pattern); INVARIANT locked at v1-beta; impl post-v1**
- Origin: L6/Am11.
- Statement: When multi-stanza HpkeMultiBase ships, each recipient MUST get a distinct, unlinkable copy with NO visible cross-recipient structure on the wire. Each copy looks like an independent Sealed-Sender envelope.
- Severity: **MED-HIGH** (invariant-class commitment now; impl post-v1). Wire-affecting: **partial** (invariant only at v1-beta). Impl-only: N. Codepoint-reserve: N (composes with U17 + U22).
- Disposition: **v1-beta-LOAD-BEARING (as Inv-18 invariant clause)** + **v1-GM-DEFER impl**.
- Depends-on: U17, U22.
- Compromise/Invariant: Inv-18 metadata-disclosure clause. Doc impact: INVARIANT-COVERAGE.md.
- Confidence: **HIGH** that per-recipient-unlinkable is right shape.

**U26 — Cover-traffic / dummy envelopes (NAMED-DEFERRED to post-v1-GM gated on shaped-relay transport)**
- Origin: L6/Am10.
- Statement: Cover-traffic is meaningful only when paired with onion-routing. Onion-routing is post-v1 per ARCHITECTURE.md line 509. Document as NAMED-DEFERRED in V1-FROZEN-INTERFACE-DEFERRED.md Row D-COVER-1 gated on shaped-relay/Tor/Nym-mixnet transport extension landing.
- Severity: **DEFERRABLE**. Wire-affecting: N. Impl-only: N. Codepoint-reserve: N.
- Disposition: **NAMED-DEFERRED** to post-v1-GM with revisit-trigger "shaped-relay transport extension lands".
- Depends-on: post-iroh `Transport` extension.
- Compromise/Invariant: #43 disclosure. Doc impact: V1-FROZEN-INTERFACE-DEFERRED.md Row D-COVER-1.
- Confidence: **MED** (cover-traffic without onion-routing is performative).

**U27 — DID-rotation discipline (NAMED-DEFERRED to post-v1 UX wave)**
- Origin: L6/§4.4.
- Statement: Long-term metadata-archive mitigation = user DID rotation discipline. Not at v1-beta wire-format; needs UX + key-management work. Document in V1-FROZEN-INTERFACE-DEFERRED.md.
- Severity: **DEFERRABLE**. Wire-affecting: N. Impl-only: N (UX-affecting). Codepoint-reserve: N.
- Disposition: **NAMED-DEFERRED** to post-v1 UX wave.
- Depends-on: —.
- Compromise/Invariant: #43. Doc impact: V1-FROZEN-INTERFACE-DEFERRED.md.
- Confidence: **MED**.

**U28 — Coarse 1-hour epoch buckets + per-envelope-random jitter (refines U5)**
- Origin: L6/Am12.
- Statement: Refine U5: `sealed_at_epoch_hour: u32 = (unix_seconds + per_envelope_random_jitter[0..3600]) / 3600`. Reduces ~33-bit timestamp leak to ~14 bits/year. Replay window granularity becomes 1-hour-buckets; matches UCAN nbf/exp.
- Severity: **RECOMMENDED** (privacy refinement; trades second-granularity replay-defense for ~19 bits/year less metadata leak; outer UCAN nonce-cache provides freshness inside 1-hour window).
- Wire-affecting: **Y** (changes u64 → u32 hour-bucket; reduces field size). Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** if Ben weights privacy heavily; **v1-GM-DEFER** if precision-replay-defense preferred. **CONSOLIDATOR RECOMMENDATION: v1-beta-LOAD-BEARING** because the U28 refinement is privacy-positive AND maintains adequate replay-defense via outer UCAN nonce-cache.
- Depends-on: U5 (refines).
- Compromise/Invariant: Inv-16 + #43. Doc impact: INTERNALS.md.
- Confidence: **HIGH** that 1-hour bucket is right granularity; **MED-HIGH** on specific jitter mechanism.

### Group H — Cross-ecosystem interop (L7 amendments)

**U29 — Cross-ecosystem-identifier-as-content emit-discipline (§3.5s operationalization)**
- Origin: L7/Am9 (LOAD-BEARING).
- Statement: Every Benten path that emits §6.2 envelope to cross-ecosystem boundary (JWE / COSE / age / multicodec / did:jwk / LAMPS-OID / IPLD-block) MUST translate Benten-internal codepoint into the matching cross-ecosystem identifier as content. Mint `benten-crypto-suite::codepoint::cross_ecosystem_map` table with `CrossEcosystemIdentifiers { lamps_oid, jose_alg_name, cose_alg_id, multicodec_key_code, age_stanza_name }` per codepoint. Cite-drift-detector enforces no codepoint-direct ecosystem-emit.
- Severity: **LOAD-BEARING** (§3.5s discipline operationalization). Wire-affecting: **N at internal envelope**; Y at adapter outputs. Impl-only: partial. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** at the mapping-table shape; adapter implementations land as ecosystem-need-arises.
- Depends-on: U7, U8.
- Compromise/Invariant: §3.5s discipline. Doc impact: CRYPTO-CODEPOINTS.md + new module benten-crypto-suite/src/cross_ecosystem.rs.
- Confidence: **HIGH**.

**U30 — DAG-CBOR outer framing with Benten-private CBOR-tag**
- Origin: L7/Am10 (RECOMMENDED).
- Statement: §6.2 envelopes on-wire / at-rest MUST be DAG-CBOR-encoded with Benten-private CBOR-tag (recommend `0xBE54`; register at IANA CBOR-tag registry pre-v1-beta). Subsumes U3 (TLV byte-injectivity is automatic under CBOR deterministic encoding RFC 8949 §4.2.1). Free interop with CBOR-aware tools + IPLD CAR-file backups + future TypeScript port.
- Severity: **RECOMMENDED** (per L7's stop-at-recommended-not-required). Wire-affecting: **Y** if adopted (would supersede ad-hoc binary). Impl-only: N. Codepoint-reserve: N.
- Disposition: **v1-beta-RECOMMENDED — Ben call** on whether to promote to LOAD-BEARING. **CONSOLIDATOR LEAN: LOAD-BEARING** because Benten already commits to BLAKE3 CIDs + Atrium peer-mesh sync (both align naturally with IPLD/DAG-CBOR), and the 5% size cost is dominated by ecosystem-recognizability + cross-language-port-ease + CID-alignment benefits.
- Depends-on: U7.
- Compromise/Invariant: Inv-16. Doc impact: INTERNALS.md + V1-FROZEN-INTERFACE.md item 6.
- Confidence: **MED-HIGH** on direction; reasonable to ship Benten-private ad-hoc binary too.

### Group I — Impl-engineering (L4 amendments; v1-beta-LOAD-BEARING for impl quality)

**U31 — Crate selection: `libcrux-ml-kem` (verified secret-independence) NOT RustCrypto `ml-kem`**
- Origin: L4/IMPL-A1 (LOAD-BEARING). Reaffirmed L5/§4.5 AF-27 (audit-readiness).
- Statement: Use `libcrux-ml-kem` with `check-secret-independence` feature, NOT RustCrypto `ml-kem`. libcrux is formally verified for panic freedom + correctness + secret-independence via hax/F*. Closes U6 (Bernstein-Persichetti) at the crate level. Migration ~50 LOC.
- Severity: **LOAD-BEARING** (impl). Wire-affecting: **N**. Impl-only: **Y**. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** + **§5 Q1 Ben call** (Option A oqs-rs vs Option B libcrux-ml-kem; consolidator HIGH recommend Option B per L4 + L5).
- Depends-on: U6.
- Compromise/Invariant: Closes #32 mechanically. Doc impact: Cargo.toml + cipher_suite.rs.
- Confidence: **HIGH**.

**U32 — XChaCha20-Poly1305 (24-byte nonce) for Layer-A + Layer-B sites where key is re-used across messages**
- Origin: L4/IMPL-A2 (LOAD-BEARING).
- Statement: Switch from ChaCha20-Poly1305 (12-byte nonce; 2^32 birthday) to XChaCha20-Poly1305 (24-byte nonce; 2^96 birthday) for all sites where same key seals many messages (Layer-A vault re-encryptions across K_principal rotations; Layer-B per-Node re-encryptions on Node updates). HPKE-mode-base sites (Layer-C/D) unaffected (HPKE manages nonce-counter internally).
- Severity: **LOAD-BEARING** (impl). Wire-affecting: **Y** (codepoint discriminates nonce-length per U12). Impl-only: partial. Codepoint-reserve: **Y** (XChaCha20 codepoint).
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U12 (variant slot).
- Compromise/Invariant: Inv-16. Doc impact: CRYPTO-CODEPOINTS.md, INTERNALS.md.
- Confidence: **HIGH**.

**U33 — Single-source `canonical_binding()` across Rust + TS via NAPI-RS (NEVER reimplement in TS)**
- Origin: L4/IMPL-A3 (LOAD-BEARING).
- Statement: TS surface MUST call into Rust `canonical_binding()` via NAPI-RS, return `Buffer`, treat bytes as opaque. Per §3.5g cross-language rule-mirror discipline.
- Severity: **LOAD-BEARING** (impl). Wire-affecting: **N**. Impl-only: **Y**. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: U1.
- Compromise/Invariant: §3.5g enforcement. Doc impact: NAPI-RS binding + integration test.
- Confidence: **HIGH** (Benten has 5+ historical cross-language drift instances).

**U34 — Opaque-handle pattern for secrets at NAPI boundary (NEVER return raw K_principal/DAK/sk bytes to V8)**
- Origin: L4/IMPL-A4 (LOAD-BEARING).
- Statement: V8 GC + Rust zeroize don't compose. Use `#[napi] pub struct VaultHandle { inner: Arc<Mutex<VaultState>> }` pattern; return decrypted Node plaintext only (the data the user is consuming), never key material that derives further secrets.
- Severity: **LOAD-BEARING** (impl). Wire-affecting: N. Impl-only: **Y**. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: —.
- Compromise/Invariant: L5-C1 (RAM-residency) partial closure. Doc impact: NAPI-RS module.
- Confidence: **HIGH**.

**U35 — `wasm_js` getrandom feature config at SHELL crate level (not library)**
- Origin: L4/IMPL-A5 (LOAD-BEARING).
- Statement: `[target.wasm32-unknown-unknown] rustflags = ['--cfg', 'getrandom_backend="wasm_js"']` MUST be set at Benten Tauri + browser-shell crate (NOT in `benten-crypto-suite` library). Forgetting = runtime panic on first OsRng call in browser.
- Severity: **LOAD-BEARING** (impl). Wire-affecting: N. Impl-only: **Y**. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: —.
- Compromise/Invariant: —. Doc impact: CLAUDE.md baked-in #17 cross-reference + .cargo/config.toml in shell crates.
- Confidence: **HIGH**.

**U36 — Tokio cancel-safety: wrap AEAD/HPKE crypto ops in `spawn_blocking`; clippy disallowed_methods lint**
- Origin: L4/IMPL-A6 (LOAD-BEARING).
- Statement: Document in `dispatch-conventions §3.5` rule. Add clippy.toml `disallowed-methods` entry for direct AEAD calls in async context.
- Severity: **LOAD-BEARING** (impl); MED-HIGH on actual Benten reachability (depends on whether crypto ops appear in `select!` arms).
- Wire-affecting: N. Impl-only: **Y**. Codepoint-reserve: N.
- Disposition: **v1-beta-LOAD-BEARING** (clippy lint is cheap insurance).
- Depends-on: —.
- Compromise/Invariant: —. Doc impact: dispatch-conventions.md + clippy.toml.
- Confidence: **MED-HIGH**.

### Group J — Test-corpus + CT-validation + formal-methods (L4 IMPL-B + L5 deliverables)

**U37 — Golden-vector test corpus generator (~96 vectors across codepoints × amendments × primitives)**
- Origin: L4/IMPL-B2 (RECOMMENDED). Reaffirmed L5/§5.2.
- Statement: `cargo run --bin gen-crypto-golden-vectors` produces versioned JSON corpus under `crates/benten-crypto-suite/tests/golden/`. TS surface validates via NAPI-RS round-trip. ~1 wave-day.
- Severity: **RECOMMENDED**. Wire-affecting: N (test). Impl-only: Y.
- Disposition: **v1-beta-LOAD-BEARING** (audit-deliverable per L5).
- Confidence: **HIGH**.

**U38 — dudect-bencher CI integration for CT-validation (nightly; per-primitive)**
- Origin: L4/IMPL-B3 (RECOMMENDED). Reaffirmed L5/§4.2 AF-18.
- Statement: Nightly workflow runs `cargo bench --bench ct_validation`; Welch's t-test fail at p<0.01 over 10⁶ samples. Cost ~3 wave-days infra + per-primitive harnesses. MUST exclude cargo-llvm-cov instrumentation (per L4/§2.10.4).
- Severity: **RECOMMENDED**. Disposition: **v1-beta-LOAD-BEARING** (audit-deliverable; complements U31 libcrux compile-time CT verification).
- Confidence: **MED-HIGH**.

**U39 — kani injectivity proof of `canonical_serialize_tlv` (formal verification)**
- Origin: L4/IMPL-B4 (RECOMMENDED).
- Statement: kani harness proves `∀ a,b: BindingContext. a.canonical_serialize_tlv() == b.canonical_serialize_tlv() ⇒ a == b` under bounded shapes. ~2 wave-days. Closes entire U3 collision class.
- Severity: **RECOMMENDED**. Disposition: **v1-beta-LOAD-BEARING** (cheap; permanent insurance).
- Depends-on: U3.
- Confidence: **MED-HIGH** tractability; **HIGH** value.

### Group K — Audit-deliverable docs (L5 high-leverage)

**U40 — `docs/THREAT-MODEL.md` mint with §2.1 25-row IN/OUT/PARTIAL matrix + §2.2 promises-and-non-promises prose**
- Origin: L5/§1 single-most-load-bearing recommendation; §2.1 matrix + §2.2 prose.
- Statement: Mint `docs/THREAT-MODEL.md` enumerating 25 adversary classes (T-01..T-25) with explicit IN-SCOPE / OUT-OF-SCOPE / PARTIAL closures + cross-references to Compromise/Invariant rows. Section structure modeled on age + Obsidian-Sync + Common-Criteria-ST shape. ~1 day; saves ~1 person-week of audit-time per L5.
- Severity: **LOAD-BEARING** for audit-readiness. Wire-affecting: N. Impl-only: N (docs).
- Disposition: **v1-beta-LOAD-BEARING**.
- Depends-on: All amendments (cross-references).
- Compromise/Invariant: All. Doc impact: THREAT-MODEL.md (new).
- Confidence: **HIGH**.

---

## §3 Unified Compromise # registry (#32..#44 + #31 extension)

Per L5/§6 + L3/§4.4 + L6/§6.1 — 13 new Compromises + 1 extension to #31.

### #32 — ML-KEM-768 Decap chosen-ciphertext side-channel surface (Layer-C drops + Layer-D wraps)
- Origin: L3/§4.4 Compromise mint (Bernstein-Persichetti); reaffirmed L5/§6.1 + L8 (out-of-L8-scope, affirmed cryptographer call).
- Statement: HPKE-mode-base[MLKEM768-X25519] inherits the published ML-KEM Decap chosen-ciphertext side-channel surface (IACR 2024/2051). An adversary with on-disk or on-wire access to envelopes can mount chosen-ciphertext queries against recipient's ML-KEM-768 sk; per-query leakage compounds across queries.
- Threat boundary: applies to all Layer-C drops + Layer-D wraps traversing adversary-accessible media (peer-mesh; iroh transports; backup media). Does NOT apply to in-process envelope handling; does NOT apply to Layer-A (ChaCha20-Poly1305).
- Mitigation status: **CT-impl-at-v1-beta via libcrux-ml-kem U31** + honest-disclosure at v1-beta + revisit-trigger "RustCrypto ml-kem ships CT-verified Decap" OR "alternative ML-KEM impl with stronger CT-proof".
- Wire impact: **N**. Doc impact: SECURITY-POSTURE.md.

### #33 — Coercion / xkcd-538-wrench attack OUT-OF-SCOPE (operational not cryptographic)
- Origin: L5-C2.
- Statement: If user is compelled (physical/legal/duress) to reveal password, Benten provides no cryptographic protection.
- Threat boundary: out-of-scope for any cryptographic claim Benten makes.
- Mitigation status: **honest-disclosure-only**; operational mitigations (plausible-deniability vaults; offline backups; duress-passwords post-v1-GM).
- Wire impact: N. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-05.

### #34 — Password-knowledge implies full access (structural by construction)
- Origin: L5-C3.
- Statement: Adversary holding both vault.cbor file AND password has full access to all K_principal-protected content. Structural to password-encrypted-vault design.
- Threat boundary: applies to all Layer-A vault content + K_principal-derived material.
- Mitigation status: **defense-in-depth via Argon2id (~1 sec cost); strong-password UX**.
- Wire impact: N. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-06/T-19.

### #35 — Compromised-device retroactive decryption (no past-content FS at v1-beta)
- Origin: L5-C4.
- Statement: K_principal delivered to device decrypts all owned content for as long as device retains K_principal. Revocation is forward-only; CGKA epoch-ratcheting deferred to post-v1-beta.
- Threat boundary: applies to all Layer-A + Layer-B content delivered to a device before revocation.
- Mitigation status: **PARTIAL via key-rotation U20**; **FULL closure requires CGKA (post-v1-beta)**.
- Wire impact: N at v1-beta. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-07.

### #36 — RAM-residency / coredump / swap forensic-extraction OUT-OF-SCOPE
- Origin: L5-C1.
- Statement: K_principal + DAK live in process memory between unlock and shutdown. `zeroize` + `secrecy` provide best-effort; OS-level swap/coredumps/hibernation outside Benten's control.
- Mitigation status: **BEST-EFFORT via U34 opaque-handle + zeroize + secrecy**; OS-hardening is operator responsibility; TEE-bound storage post-v1-GM (#38).
- Wire impact: N. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-04.

### #37 — No TEE / sealed-enclave attestation at v1-beta + v1-GM
- Origin: L5-C5.
- Statement: Benten engine runs in user-space without TEE-attestation (SGX/SecureEnclave/TrustZone/TPM-attested-launch).
- Mitigation status: **honest-disclosure** + best-effort via U34 + future TEE-bound K_principal storage post-v1-GM.
- Wire impact: N. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-08.

### #38 — Physical-presence side-channels (DPA / SEMA / fault-injection) OUT-OF-SCOPE
- Origin: L5-C6.
- Statement: Power-analysis, EM-emanation, voltage/clock-glitching fault injection out-of-scope at v1-beta + v1-GM (HSM-class threats; not pure-software defense).
- Mitigation status: **operational physical security**; HSM-backed key-storage post-v1-GM if demand emerges.
- Wire impact: N. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-10/T-11.

### #39 — Supply-chain dependency-pinning posture (PARTIAL)
- Origin: L5-C7.
- Statement: Benten depends on RustCrypto + libcrux-ml-kem + McMillion's hpke (NOT Cryspen's hpke-rs per Verification Theatre) + zeroize + secrecy + keyring-core. Supply-chain compromise risk.
- Mitigation status: **cargo deny + RustSec monitoring + version-pinning + explicit crate choice**; reproducible-builds post-v1-GM (#40).
- Wire impact: N. Doc impact: SECURITY-POSTURE.md.

### #40 — Build-time / reproducible-builds + SLSA-3+ posture (out-of-scope post-v1-GM)
- Origin: L5-C8.
- Statement: v1-beta binaries built from GitHub Actions without reproducible-builds attestation. SLSA-3+ + Sigstore-style provenance post-v1-GM.
- Mitigation status: **honest-disclosure** + GitHub Actions logs auditable + Cargo.lock pinned.
- Wire impact: N. Doc impact: SECURITY-POSTURE.md.

### #41 — Cross-device-sync UX-vs-cryptographic boundary (compromised-device-on-mesh)
- Origin: L5-C10.
- Statement: Compromised device on user's mesh holds legitimate device-key; can perform operations device is authorized for; U4 sender-DID binding distinguishes "which device sent" but not "legitimate Bob vs compromised Bob's-device".
- Mitigation status: **device-revocation via admin-UI** + device-attestation post-v1-GM (composes with #37).
- Wire impact: N. Doc impact: SECURITY-POSTURE.md + THREAT-MODEL.md T-20.

### #42 — Layer-C FS-gap (HPKE-mode-base recipient long-term sk decrypts forever)
- Origin: L8/Am13.
- Statement: HPKE-mode-base is structurally non-FS at long-term-sk axis. 2030-sk-compromise recovers 2026 envelopes. Application-layer key rotation gives partial FS; structural FS requires CGKA/MLS-PQ (post-v1-beta per U13 codepoint reservation).
- Mitigation status: **PARTIAL via application-layer rotation** (post-v1-beta; no wire-format change); FULL closure via MLS-PQ additive codepoints reserved per U13.
- Wire impact: N at v1-beta (reservation only). Doc impact: SECURITY-POSTURE.md.

### #43 — Envelope metadata leakage to untrusted relays (V1-beta accepted trade-off)
- Origin: L6/§6.1 Compromise mint.
- Statement: §6.2 EncryptedEnvelope binds sender DID + recipient DID + operation type + sealed-at epoch into plaintext AAD (per U4 + U5). Adversary observing wire (including untrusted Atrium peer-mesh relays) learns directed sender→recipient graph + per-pair message frequency + per-user operation-type histogram + ~1-hour-precision timestamps WITHOUT cryptographic break. Compromise #31 amplifies: archived metadata is forever-archived.
- Threat boundary: applies to wire-observable surfaces; co-located adversaries; untrusted relays; long-term archive correlators.
- Mitigation roadmap: **U22 Sealed-Sender codepoint slot reserved at v1-beta** + **U23 per-relay-unlinkability** + **U24 size-class padding** + **U25 per-recipient-unlinkable multi-stanza** + **U28 coarse epoch buckets**; full closure requires Sphinx-style onion-routing + non-iroh transport (post-v1).
- Wire impact: **Y** at v1-beta (because Am4+Am5 mandate plaintext-AAD bindings). Doc impact: SECURITY-POSTURE.md + plain-English user-facing statement per L6/§7.

### #44 — Long-term-confidentiality posture (BSI TR-02102-1) OUT-OF-SCOPE
- Origin: L5-C-LTC1.
- Statement: BSI names FrodoKEM + Classic McEliece as only long-term-confidentiality-floor KEMs. ML-KEM-768 + X25519 (X-Wing) is acceptable migration-window but NOT on BSI long-term floor.
- Mitigation status: **honest-disclosure**; FrodoKEM-hybrid additive codepoint reservable post-v1-GM if regulatory demand.
- Wire impact: N. Doc impact: SECURITY-POSTURE.md.

### EXTENSION to existing Compromise #31 — Drop-bundle composition with encrypt-to-recipient
- Origin: L5-C9 + L9/A3 (recipient-key-retention window clarification).
- Extension: Explicitly document encrypt-to-recipient composition consequence — Drop bundle decryptable by recipient device forever as long as device retains ML-KEM-768 sk; recipient-side key rotation is only revocation mechanism; "forever-valid" means "forever-valid within recipient's key-retention window" (per U19 ≥1-year grace).
- Wire impact: N. Doc impact: SECURITY-POSTURE.md amendment to #31 narrative + cross-link to U19 + U22.

### Compromise dedup notes
- L3/§4.4 Bernstein-Persichetti → #32 (single mint; L5/§6.1 + L8 concur)
- L5-C9 + L9/A3 → unified as Compromise #31 extension (both target same forever-valid-but-recipient-key-retention boundary)
- L6's metadata-leak compromise → #43 (single mint; uniquely L6's contribution)
- L3 also referenced "Compromise #32 for B-P" while L6 referenced "Compromise #32 for metadata"; **CONSOLIDATOR RESOLUTION:** #32 = Bernstein-Persichetti (L3's load-bearing); #43 = metadata-leak (L6's load-bearing). L3 mint takes #32 by chronological priority.

---

## §4 Unified invariant set (Inv-16, Inv-17, Inv-18)

### Inv-16 — Envelope-layer-unification + codepoint-discriminated primitive choice + AAD-binding + strict-decode + canonical-TLV + sender-DID + replay-window
- Origin: L2/§4.3 (primitive-neutral phrasing) + L5/§6.2 (normative Amendments 1–6 reference) + L8 (extensions to permanence amendments).
- Phrasing (consolidated B-equivalent + L8 extensions):
  > Every encryption use site in Benten dispatches through a single `EncryptedEnvelope` shape with codepoint-discriminated `EnvelopePayload` variant (U9 `#[non_exhaustive]`) + codepoint-discriminated `BindingContext` variant (U10 `#[non_exhaustive]`); symmetric and asymmetric primitives are codepoint-distinct; codepoint bound in canonicalized AAD/info-string (U1) with `aad_version: u8` prefix (U14); strict-decode dispatch with no cross-variant fallback (U2); canonical-TLV-encoded BindingContext with length-injective per-field encoding (U3); non-vault variants bind sender-DID (U4); DeviceLink + RemotePermission bind sealed-at + valid-until epoch (U5; coarse-bucket per U28); HPKE-mode-base Decap is constant-time per verified ML-KEM impl (U6); multi-recipient drops use `HpkeMultiBase` variant with cross-stanza substitution defense (U17); plaintext_cid and envelope_blob_cid are distinct (U18); recipient_key_generation + k_principal_generation tracked (U19 + U20).
- Composes-with: Inv-15 (signature-bundle-CID identifier discipline — sibling at construction layer); Inv-1 (deterministic canonical encoding); Inv-5 (BLAKE3 CID).
- Enforcement plan: similar to Inv-15 — registered at Phase-4-Meta-Core mint; full enforcement-completion at G-CORE-PQ-WIRE wave shipping unified envelope. Property tests per L4 IMPL-B2 corpus + L5 §5.2 corpus.

### Inv-17 — Hybrid-cryptography-mandatory floor
- Origin: L5/Inv-L5-2.
- Phrasing:
  > Every Benten KEM use site MUST use a PQ-classical hybrid construction (X-Wing or future-additive equivalent). No pure-classical or pure-PQ KEM codepoints are mintable at v1-beta or v1-GM. Aligns with ANSSI mandatory-hybridation guidance + BSI hybrid-recommendation + NIST SP 800-227 §4.4 hybrid-combiners-with-concatenation.
- Composes-with: Inv-15, Inv-16.
- Enforcement plan: codepoint-registry discipline + cite-drift-detector scanner flags KEM-codepoint mints lacking PQ + classical halves.

### Inv-18 — Codepoint-registry-discipline + metadata-disclosure invariant + CodepointLifecycle typed-state
- Origin: L5/Inv-L5-3 (registry discipline) + L6/§6.2 Inv-16 (metadata-disclosure) + L8/Am16 (CodepointLifecycle). Consolidator merges these three sibling registry-class invariants.
- Phrasing:
  > (a) Every minted Benten cryptographic codepoint MUST appear in `docs/CRYPTO-CODEPOINTS.md` registry with: hex value (BE on-wire per U7); variant name; primitive choice; AAD-binding spec; since-version; rationale; **CodepointLifecycle state** (Live/Deprecated/Quarantined/Burned per U16); cross-ecosystem-identifier row per U29.
  > (b) Codepoint range is IANA-disjoint (`0x6100..0x6FFF` reserved for Benten envelope codepoints per U8 + U11).
  > (c) Every envelope shape that places sender-DID or operation-type in plaintext AAD MUST be disclosed at `docs/SECURITY-POSTURE.md` (per #43) AND MUST have a paired metadata-hiding additive codepoint slot reserved at v1-beta (Sealed-Sender per U22 OR equivalent).
  > (d) Lifecycle transitions: Live → Deprecated (minor advisory); Deprecated → Quarantined (partial cryptanalysis); Quarantined → Burned (full break). Wire-format decode-supported FOREVER for Live + Deprecated; Quarantined admits with explicit caller warning; Burned hard-refuses.
- Composes-with: Inv-16, Inv-17.
- Enforcement plan: cite-drift-detector scanner check on codepoint definitions vs registry rows; `PlaintextSenderInAadPattern` scanner flags new BindingContext variants placing identity-DIDs in AAD without paired Sealed-Sender sibling.

---

## §5 Disagreement matrix

5 substantive forks surfaced for Ben ratification.

### Q1 — Option A (oqs-rs) vs Option B (libcrux-ml-kem) for ML-KEM impl

| Lens | Position |
|---|---|
| **L4 IMPL-A1** | **Option B (libcrux-ml-kem)**. Verified for panic-freedom + correctness + secret-independence via hax/F*. `check-secret-independence` feature gives compile-time CT verification via libcrux-secrets type tagging. Mechanically closes L3/Am6 at crate level. Migration ~50 LOC. Aligns with CLAUDE.md baked-in #5 (not vendoring; choosing a different upstream). |
| **L5 §4.5 AF-26-30** | Concurs Option B — verified secret-independence reduces audit-time spend on Decap CT-verification by ~50%. Cryspen + pq-code-package multi-vendor stewardship is audit-firm-recognized. |
| **L1 §4.3** | Caveated "vendor-and-patch path is structurally forbidden" (baked-in #5). L4 reads this as forbidding RustCrypto-fork; choosing a vetted-different-upstream (libcrux) is NOT vendoring. |
| **L3 §4.4 option-a** | Recommended option-c (BOTH crate-impl + Compromise mint) per its load-bearing analysis — agnostic on specific crate; CT-Decap mandate is the load-bearing requirement. libcrux satisfies the mandate; RustCrypto ml-kem v0.x best-effort does not (per L4). |
| **oqs-rs (Option A)** | Not directly evaluated by any lens. oqs-rs wraps liboqs (C library); larger attack surface; FFI marshaling cost; less Rust-idiomatic; no compile-time CT-verification feature. |

**CONSOLIDATOR ASSESSMENT (HIGH):** Recommend **Option B (libcrux-ml-kem)** per L4 + L5 + Ben preference. Risk delta vs RustCrypto ml-kem: near-zero migration cost; substantial audit-readiness gain; compile-time CT verification is the kind of belt-and-suspenders defense U6 needs. Ratification suggested at R0.

### Q2 — LE-vs-BE codepoint endianness migration in `crates/benten-crypto-suite/src/aead.rs`

| Lens | Position |
|---|---|
| **L3 Am7** | Mandates **BE** codepoint encoding on-wire (matches RFC 9180 / FIPS 203 / MLS conventions). |
| **L4 IMPL-B1** | Surfaced the conflict: existing `aead.rs` line 22 documents wire format as `"bytes 2-3: cipher codepoint (LE u16; e.g. 0x647a for X-Wing-hybrid)"`. Both L2 + L3 wrote `self.codepoint.to_be_bytes()`. Neither checked existing code. **IMPL-DECISION-PENDING** flagged for Ben. Options: (a) migrate aead.rs LE → BE (~5 LOC + regen golden vectors + bump `ENVELOPE_FORMAT_VERSION_V1 → V2`; ~1 wave-day net cost); (b) amend L3/Am7 to LE (preserves existing G-CORE-3a wire format; less idiomatic). |
| **L7 §7.4** | Pin "BE on Benten-internal wire". Adapter outputs translate to ecosystem identifiers (don't carry raw 16-bit codepoint). |

**CONSOLIDATOR ASSESSMENT (MED-HIGH):** Recommend **(a) migrate aead.rs to BE pre-v1-beta-freeze**. Network-byte-order is canonical. Per L4 source-comment "G-CORE-9 freezes" suggests not-yet-frozen, so migration is cheap. Living with LE forever is a permanent oddity that creates a 2-axis cite-drift hazard. ~1 wave-day cost dominated by future-maintainer-confusion-prevention.

### Q3 — L9 dual-CID (plaintext_cid + envelope_blob_cid) vs prior-P2P-architect F-refinement-2 (recipient_set-in-CID)

| Position | Argument |
|---|---|
| **L9/A2 (LOAD-BEARING)** | Drop bundle MUST have TWO distinct CIDs: `plaintext_cid = BLAKE3(canonical(DropBundlePayload))` (stable, graph-referenced, survives reseal + recipient-set evolution + cipher rotation); `envelope_blob_cid = BLAKE3(serialized_EncryptedEnvelope)` (transport, changes on reseal). Mapping `plaintext_cid → Vec<envelope_blob_cid>` replicates via Atrium. Reasons: (1) adding new Atrium member retroactively to past Drops mutates their CIDs — breaks graph references; (2) content-identity ≠ authorization-identity semantically; (3) forkability requires pre-fork Drop CIDs remain valid post-fork. Recipient-substitution defense lives in per-stanza AAD (per U17 A1 cross-stanza defense), NOT in Drop CID. |
| **Prior P2P-architect F-refinement-2 (e2r-ffull-scope-review @ 220b5aae)** | `Drop-CID = BLAKE3(plaintext || recipient_did_set || context_label)` — binds recipient set INTO Drop CID. Original intent: defense against recipient-substitution attacks at the CID layer. |

**CONSOLIDATOR ASSESSMENT (HIGH):** Concur with **L9's dual-CID**. F-refinement-2 conflates two distinct semantic properties: content-identity (what's in the Drop — should be stable) and authorization-identity (who's authorized — dynamic). The recipient-substitution defense is structurally better served by per-stanza AAD-binding (U17) which lives in the envelope, not by mutating the content-identity CID. Forkability semantic (Ben-ratified 2026-05-27) requires pre-fork Drop CIDs survive fork events; F-refinement-2 makes this impossible. Recommend **ratify L9/A2** + amend e2r-ffull-scope-review's F-refinement-2 disposition.

### Q4 — JOSE alg-name `HPKE-11` (integrated) vs `HPKE-11-KE` (key-encryption mode) per L7

| Mode | Semantics | Maps to Benten |
|---|---|---|
| `HPKE-11` (integrated) | HPKE itself bulk-encrypts plaintext directly | If Benten Layer-C bulk-encrypts via HPKE directly |
| `HPKE-11-KE` (key-encryption) | HPKE encrypts a content-encryption-key (CEK) which then bulk-encrypts plaintext with separate AEAD | If Benten Layer-C wraps K_principal/CEK then K(N)-derived AEAD bulk-encrypts |

**L7 §8.5 self-critique** flagged this: "Benten's actual usage at Layer-C/D — does it bulk-encrypt via HPKE directly, or does HPKE wrap a CEK that then bulk-encrypts? I assumed key-encryption mode (`HPKE-11-KE`) because Benten's pattern is 'HPKE wraps K_principal for the recipient, then K_principal-derived K(N) bulk-encrypts' — that's the key-encryption shape."

**CONSOLIDATOR ASSESSMENT (MED-HIGH):** L9/§3.1 confirms the flat composition: "ONE outer EncryptedEnvelope with HpkeMultiBase payload (per U17), containing a CBOR-serialized DropBundlePayload as the plaintext... Each Node body is already encrypted at Layer-B with K(N) = KDF(K_principal, N.cid). The Drop bundle includes the per-Node K(N) keys wrapped under the multi-stanza CEK so recipients can derive K(N) and AEAD-Open the Layer-B ciphertexts." This is **key-encryption mode** — HPKE wraps the CEK; CEK + per-Node K(N) bulk-encrypt. **Recommend `HPKE-11-KE` mapping**. Verify at R0 §4 design when full Layer-C wire format is pinned.

### Q5 — Sealed-Sender as v1-beta-DEFAULT codepoint vs v1-beta-RESERVED-FUTURE-ADDITIVE slot

| Position | Argument |
|---|---|
| **L6/Am7 (RECOMMENDED additive slot)** | Mint codepoint slot at v1-beta (zero cost); implementation post-v1-beta (G-CORE-PRIVACY-1 wave). Reason: Signal's spam-mitigation pattern (per-recipient delivery tokens) is non-trivial engineering work; Benten's existing capability discipline may or may not provide equivalent. Conservative call: ship default codepoint + Sealed-Sender as opt-in additive. |
| **Aggressive-privacy stance** | Ship Sealed-Sender as v1-beta-DEFAULT codepoint; treat plaintext-sender codepoint as legacy. Reason: metadata-leak (#43) is the single biggest gap between Benten's "decentralized + e2ee + p2p-encryption-by-default" positioning and actual privacy posture. |

**CONSOLIDATOR ASSESSMENT (MED-leaning-L6):** Recommend **L6's additive-slot disposition** for v1-beta, with explicit roadmap commitment to Sealed-Sender implementation at G-CORE-PRIVACY-1 (post-v1-beta but pre-v1-GM). Reasons: (a) Sealed-Sender spam-mitigation engineering is real cost; (b) v1-beta wave is already 35-45 wave-days per L4; adding Sealed-Sender implementation expands ~5-8 more wave-days; (c) additive-slot reservation at v1-beta preserves wire-format-evolution path without bloating v1-beta scope. **Ben call** is whether the privacy-marketing posture warrants accepting the additional v1-beta scope cost.

### Q-extra (consolidator-surfaced 5th) — Am4 sender-DID-in-AAD tension with metadata privacy

| Position | Argument |
|---|---|
| **L3/Am4 LOAD-BEARING** | Binds sender-DID into AAD for defense-in-depth against AAD-belief-drift confused-deputy attacks. Real integrity property. |
| **L6/§2.4 row Am4** | Same Amendment STRICTLY WORSENS metadata privacy. Sender-DID structurally plaintext in AAD. Forecloses future-additive Sealed-Sender UNLESS design carves out alternative codepoint slot. |

**CONSOLIDATOR RESOLUTION (HIGH):** Both are right. Keep U4 (Am4) for default codepoint (`LAYER_C_DROP = 0x6300/0x6500`) per integrity property. Reserve U22 (Sealed-Sender = `0x6510`) per L6 as the metadata-privacy-preferring sibling codepoint. Caller chooses per-drop based on threat-model preference. Inv-18 enforces the paired-sibling-slot discipline so all future plaintext-sender variants automatically reserve a Sealed-Sender pair.

---

## §6 v1-beta-LOAD-BEARING subset recommendation

Per CLAUDE.md baked-in #5 (crypto-agility) + #15 (v1-beta interface freeze) + `feedback_orchestrator_defer_prediction_bias`.

### Decision rule
- **Wire-format-affecting** → MUST be v1-beta-LOAD-BEARING (pre-interface-freeze)
- **Codepoint-slot-reservation** → MUST be v1-beta-CODEPOINT-RESERVE (impl post-tag OK; slot locked)
- **Impl-only no wire impact** → v1-GM-DEFER if cost justifies; otherwise v1-beta if it's audit-deliverable
- **Pure ecosystem-contribution** → NAMED-DEFERRED with revisit-trigger

### Categorized subset (28 amendments + 13 Compromises + 3 invariants)

| Bucket | Count | Items |
|---|---|---|
| **v1-beta-LOAD-BEARING (must ship pre-tag)** | **22 amendments** | U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U14, U15, U16, U17, U18, U19, U20, U28, U29, U33 |
| **v1-beta-CODEPOINT-RESERVE (slot lock; impl deferred)** | **6 amendments** | U13 (FS-gap codepoint brackets), U21 (ExecuteWorkflow variant slot), U22 (Sealed-Sender slot), U23 (per-relay-unlinkability shape), U24 (size-class bucket shape), U25 (per-recipient-unlinkable invariant) |
| **v1-beta-LOAD-BEARING (impl-engineering + audit-deliverable)** | **8 amendments** | U30 (DAG-CBOR — consolidator-leaned), U31 (libcrux), U32 (XChaCha20), U34 (NAPI opaque-handle), U35 (wasm_js cfg), U36 (cancel-safety), U37 (golden corpus), U38 (dudect CI), U39 (kani injectivity), U40 (THREAT-MODEL.md) |
| **NAMED-DEFERRED (post-v1-GM with revisit-trigger)** | **2 amendments** | U26 (cover-traffic; gated on shaped-relay transport), U27 (DID-rotation discipline; gated on post-v1 UX wave) |
| **Compromises MINT at v1-beta** | **13 + 1 extension** | #32, #33, #34, #35, #36, #37, #38, #39, #40, #41, #42, #43, #44 + #31 extension |
| **Invariants MINT at v1-beta** | **3** | Inv-16, Inv-17, Inv-18 |

### Notes on bucket assignments
- U30 (DAG-CBOR) was L7-RECOMMENDED but consolidator leaned LOAD-BEARING per §2 confidence note. Ben call surfaced.
- U28 (coarse 1-hour epoch buckets refining U5) consolidator-leaned LOAD-BEARING for privacy; could defer to v1-GM if precision-replay preferred. Ben call surfaced.
- All Compromises mint at v1-beta (per L5's audit-readiness recommendation): honest-disclosure-NOW is structurally cheaper than retroactive-honest-disclosure-LATER.

### Cost estimate (consolidated from L4 §7 + L5 §5)

| Component | Wave-days |
|---|---|
| L4 §7 implementation LOC (~2730 LOC + ~4200 test LOC) | ~17.5 |
| L4 §7 test corpus + CT-validation infra | ~11.5 |
| L4 §7 per-platform validation | ~4.5 |
| L4 §7 documentation + audit-firm prep | ~2.0 |
| L5 §5 audit-deliverable docs (THREAT-MODEL, KEY-LIFECYCLE, CRYPTO-PARAMETERS, AUDIT-SCOPE-STATEMENT) | ~4.25 |
| L5 §5 test-corpus deliverables (KAT + negative + sender-substitution + replay + endianness + B-P regression) | ~4.15 |
| L9 A1-A5 implementation incremental | ~3-5 |
| L6 U22 + U23 + U24 reservation work | ~1-2 |
| L7 U29 + U30 implementation | ~2-4 |
| **Subtotal** | **~50-55 wave-days** |
| **Buffer +30%** | **~15-17 wave-days** |
| **GRAND TOTAL** | **~65-72 wave-days = ~13-15 calendar-weeks** |

**Fits the v1-beta 7-15 week window** if R3/R5 implementer briefs absorb all 22 LOAD-BEARING amendments + 6 CODEPOINT-RESERVE amendments + 8 impl-engineering amendments upfront. **Risk:** if amendments emerge at R5 review time, late-discovery rework adds ~2-4 weeks.

---

## §7 F-full R0 plan-doc skeleton

Skeleton outline (NOT full text) for the F-full R0 plan-doc that incorporates the consolidated registry.

### §1 Architectural framing
- B-equivalent (separate primitives per layer; not Option F+ pseudo-keypair)
- §6.2 envelope-layer-unification at the codepoint-discriminated framing layer (per Inv-16)
- Per-layer primitive choice with cited reasons: Layer-A ChaCha20/XChaCha20-Poly1305-under-DAK; Layer-B AEAD-under-K(N); Layer-C HPKE-mode-base[MLKEM768-X25519]; Layer-D HPKE-mode-base[MLKEM768-X25519]
- Cross-reference to consolidated Inv-15 + Inv-16 + Inv-17 + Inv-18

### §2 Layer-A vault design
- ChaCha20-Poly1305 vs XChaCha20-Poly1305 choice (consolidator: XChaCha20 per L4 IMPL-A2 / U32; 24-byte nonce; 2^96 birthday)
- AEAD-under-DAK derived via Argon2id + HKDF (Bitwarden/1Password/Molly/age scrypt-recipient/Stronghold/OPAQUE precedent; L1 §3 table)
- Argon2id parameter tiering per L4 §2.2.2 (Mobile/Desktop/Server tiers) + tier stored in vault metadata + AAD-bound per U4
- K_principal-generation tracking per U20

### §3 Layer-B per-Node AEAD design
- `K(N) = KDF(K_principal, N.cid, k_principal_generation)` per U20
- Chunk-index binding into AAD (existing `aead.rs` discipline + extension for U10 BindingContext::PerNodeAead variant per L4 §2.3.3)
- XChaCha20-Poly1305 per U32

### §4 Layer-C encrypt-to-recipient design
- HPKE-mode-base[MLKEM768-X25519] via McMillion `rust-hpke` (NOT Cryspen `hpke-rs` per Verification-Theatre)
- libcrux-ml-kem with `check-secret-independence` per U31 (closes U6 Bernstein-Persichetti)
- JOSE alg-name `HPKE-11-KE` per L7 + §5 Q4 (key-encryption mode)
- Multi-stanza per U17 with cross-stanza substitution defense
- Recipient-key-rotation per U19 (`recipient_key_generation: u32`; ≥1-year retention grace)
- Per-recipient-unlinkable copies per U25 invariant (Signal Sealed Sender V2 pattern)
- Sealed-Sender additive codepoint slot reserved per U22

### §5 Layer-D DAK substrate + device-link + remote-permission + multi-device-key-wrap
- HPKE-mode-base[MLKEM768-X25519] to recipient device's REAL high-entropy keypair
- Sender-device-DID + grantor + requester device-DID binding per U4
- Sealed-at + valid-until epoch per U5 (or coarse 1-hour buckets per U28)
- ExecuteWorkflow variant per U21 + L9 A5 (hyper-scaling rented-compute path)
- ProvisioningPayload wire format per e2r-ffull-scope-review §7.2

### §6 EncryptedEnvelope wire format
- DAG-CBOR outer framing with Benten-private CBOR-tag `0xBE54` per U30 (recommended; Ben call per §5 Q4)
- `EncryptedEnvelope { codepoint: u16 (BE, U7), payload: EnvelopePayload (U9 #[non_exhaustive]), aad_binding: BindingContext (U10 #[non_exhaustive]) }`
- AAD canonicalization via `canonical_binding()` with `aad_version: u8` prefix per U14 + TLV `0xFF` extended-canonicalization marker
- `canonical_serialize_tlv()` length-injectivity per U3
- Escape codepoint `0xFFFF` + experimental range `0xFE00..0xFFFE` per U11
- Codepoint-bracketed reservation per U11 + U13 (Layer-A `0x6100..0x61FF`; Layer-C `0x6300..0x63FF`; DeviceLink `0x6310..0x631F`; RemotePermission `0x6320..0x632F`; MLS-Group `0x6380..0x638F`; CGKA `0x63A0..0x63AF`; Bird-of-Prey `0x63B0..0x63BF`; revocation/lifecycle `0x6700..0x67FF`)
- `Did` multikey canonical encoding + `Did::Unknown` typed-rejection per U15
- CodepointLifecycle typed-state per U16

### §7 ml-kem crate choice
- **libcrux-ml-kem** with `check-secret-independence` feature per U31 + §5 Q1
- Migration plan from RustCrypto `ml-kem` (~50 LOC per L4); pin version `=x.y.z`
- Hax/F* verified panic-freedom + correctness + secret-independence

### §8 Wave decomposition for R5 implementation
- Per `feedback_canary_first_parallel_implementation`: identify canary wave (likely G-CORE-9 wire-format-freeze owner including U7 + U11 + U30 framing)
- Fan out parallel waves after canary merges:
  - Wave A: envelope shape + canonical_binding + TLV + amendments 1-3 (foundational)
  - Wave B: Layer-A vault + Argon2id tiering + XChaCha20 + amendments 4-5 + U20
  - Wave C: Layer-C HPKE-mode-base + libcrux + multi-stanza + amendments 17-19 + 22 + 25
  - Wave D: Layer-D device-link + remote-permission + amendments 21 + ExecuteWorkflow
  - Wave E: cross-ecosystem adapters (U29) + DAG-CBOR (U30) + golden vectors (U37) + dudect CI (U38) + kani (U39)
  - Wave F: audit-deliverable docs (U40 THREAT-MODEL.md + KEY-LIFECYCLE.md + CRYPTO-PARAMETERS.md + AUDIT-SCOPE-STATEMENT.md + SECURITY-POSTURE.md Compromise mints + INVARIANT-COVERAGE.md Inv-16/17/18 mints)

### §9 Test landscape outline for R2
- Negative tests: cross-codepoint substitution; variant confusion; sender substitution; replay-window expired; codepoint endianness LE-vs-BE; length-injectivity collision; Bernstein-Persichetti Decap timing regression
- KAT tests: RFC 9180 §B.1 + X-Wing draft test vectors + libcrux test corpus
- Golden vectors: ~96 vectors across codepoints × amendments × primitives per U37
- Property tests: AAD-binding round-trip; TLV injectivity; expiry; sender-binding
- CT-validation (nightly): dudect + ctgrind per U38
- Formal verification: kani injectivity proof per U39

### §10 LOC budget per L4
- ~2730 production LOC + ~4200 test LOC = ~6930 LOC total
- ~35-45 wave-days incremental implementation
- ~7-9 wave-days docs + audit prep
- ~10-15 wave-days buffer for late-discovery rework
- **Total ~65-72 wave-days = ~13-15 calendar-weeks** at ~5 wave-days/week

### §11 v1-beta-LOAD-BEARING subset vs v1-GM-DEFER vs NAMED-DEFERRED
- Reference §6 bucket table
- 22 LOAD-BEARING wire-affecting amendments
- 6 CODEPOINT-RESERVE amendments (slot lock at v1-beta; impl post-tag)
- 8 impl-engineering LOAD-BEARING amendments
- 2 NAMED-DEFERRED (U26 cover-traffic; U27 DID-rotation)

### §12 Compromise # mints to land at v1-beta tag
- #32 (Bernstein-Persichetti Decap CCA — L3 + L5)
- #33 (Coercion out-of-scope — L5-C2)
- #34 (Password-knowledge implies access — L5-C3)
- #35 (Compromised-device retroactive decryption — L5-C4)
- #36 (RAM-residency out-of-scope — L5-C1)
- #37 (No TEE attestation — L5-C5)
- #38 (Physical-presence side-channels out-of-scope — L5-C6)
- #39 (Supply-chain dependency-pinning — L5-C7)
- #40 (No reproducible-builds — L5-C8)
- #41 (Cross-device-sync UX boundary — L5-C10)
- #42 (Layer-C FS-gap — L8 Am13)
- #43 (Envelope metadata leakage — L6)
- #44 (BSI long-term-confidentiality out-of-scope — L5-C-LTC1)
- + EXTENSION to existing Compromise #31 per L5-C9 + L9-A3

### §13 Invariant additions
- Inv-16 envelope-layer-unification (primitive-neutral phrasing per L2; normative ref to Amendments U1-U20 per L5+L8)
- Inv-17 hybrid-cryptography-mandatory floor (per L5 Inv-L5-2)
- Inv-18 codepoint-registry-discipline + metadata-disclosure + CodepointLifecycle (consolidated L5 Inv-L5-3 + L6 Inv-16-metadata + L8 Am16)

### §14 docs/THREAT-MODEL.md skeleton per L5
- §1 Scope statement
- §2 In-scope adversary classes (T-01..T-25 per L5 §2.1 matrix)
- §3 Out-of-scope adversary classes with Compromise-mint cross-references
- §4 Cryptographic primitives table (FIPS/NIST/ANSSI conformance per primitive)
- §5 Confidentiality properties promised
- §6 Honest-disclosure section (what Benten does NOT promise)
- §7 Cross-reference table (threat-class row → mitigation site → test-corpus pin → Compromise/Invariant doc reference)

### §15 Cross-ecosystem-interop emit-discipline per L7 + U29
- Mint `benten-crypto-suite::codepoint::cross_ecosystem_map` module
- Mapping table: codepoint → (LAMPS OID, JOSE alg-name, COSE alg-id, multicodec key-code, age stanza-name)
- Cite-drift-detector enforcement (forbid codepoint-direct ecosystem-emit; require `cross_ecosystem().<field>()` translation)
- Adapter modules: benten-drop::envelope_jwe_emit, envelope_cose_emit, envelope_age_emit (post-v1-beta as ecosystem-need arises)

### §16 RecoveryHook trait per L5 + F-full scope review §7
- Recovery key escrow + RecoveryHook trait shape
- Per L5 §6 cross-references + F-full scope review §7 RecoveryHook substrate

---

## §8 Pattern-induction meta-findings (consolidator-surfaced)

### MF1 — Sender-authentication / metadata-privacy is a STRUCTURAL TENSION not an incidental clash
- L3/Am4 strengthens authentication property; L6 explicitly notes this STRICTLY WORSENS metadata privacy.
- The two amendments compose only via U22 Sealed-Sender additive codepoint pattern (caller-choice per-drop based on threat-model preference).
- **Codified in Inv-18 metadata-disclosure clause:** every envelope shape placing identity-DIDs in plaintext AAD MUST have a paired Sealed-Sender sibling codepoint slot.
- **Lesson for future privacy/security-tension reviews:** when two valid amendments conflict structurally (not just superficially), the resolution is rarely "pick one" — it's "carve out a sibling-codepoint shape for the alternative-threat-model preference" + lock the pairing-discipline as an invariant.

### MF2 — Inv-15 + Inv-16 are sibling invariants at DIFFERENT identifier-hazard layers
- Inv-15: payload-CID-as-identifier discipline at signature-construction layer (signature-bundle-CID is NEVER load-bearing).
- Inv-16: envelope-layer-unification + codepoint-discriminated primitive + AAD-binding at confidentiality-construction layer.
- Both target identifier-hazards but at different layers. L5 + L6 + L9 explicitly note "Inv-15 doesn't help here; we need Inv-16."
- **Lesson:** "invariant discipline" generalizes across cryptographic layers; future Inv-N mints should cite-pattern against Inv-15 + Inv-16 as siblings.

### MF3 — Three permanence amendments form a coherent package
- L8 introduced U9 (`#[non_exhaustive]`) + U11 (escape codepoint) + U14 (`aad_version: u8`) as separate amendments. They form ONE coherent permanence-package: (a) enum extensibility (U9/U10); (b) codepoint-axis extensibility (U11); (c) canonicalization-shape extensibility (U14). Each closes a different permanence concern; together they cover the full v1-beta-freeze permanence surface.
- **Lesson:** wire-format-freeze permanence reviews should explicitly cover the 3-axis surface (variant/codepoint/canonicalization) as a package check, not piecemeal.

### MF4 — Compromise #31 (forever-valid drops) is upstream of distinct downstream hazards in ≥4 lenses
- L3/Am6: Bernstein-Persichetti Decap CCA queries scale linearly with replay opportunity.
- L6/§4: long-term metadata-archive correlator.
- L9/A3: recipient-key-rotation interaction (envelopes orphaned).
- L8/Am13: structural FS-gap (long-term-sk decrypts forever).
- All four name Compromise #31 as upstream. The downstream hazards are distinct but rooted in the same "forever-valid drops" property.
- **Lesson:** Compromise rows can have CASCADING dependency-tree of downstream Compromise rows; the consolidated registry should cross-link these (e.g. #32, #35, #43, #42 all cross-link to #31). The registry should support a "downstream-hazards-from" backlink view.
- **Action:** add cross-reference section to SECURITY-POSTURE.md showing #31's downstream-hazard tree.

### MF5 — Lens-composition incomplete coverage even with 9 lenses (consolidator self-critique)
- 9 lenses + e2r scope review = comprehensive but NOT exhaustive. Gaps consolidator notes:
  - **No formal-methods lens** evaluating tractability of HPKE-mode-base[X-Wing] IND-CCA2 proof under adversarially-chosen-recipient-seed model (L1/L2 named the gap; no lens dispatched to close it). For v1-beta this is acceptable per L5 (informal-rigorous prose; not formal-proof-machine-checked). For v1-GM external audit may force the issue.
  - **No regulatory-jurisdiction-specific lens** for EU eIDAS-3 / German critical-infrastructure / French regulated-financial. L5 enumerates these as OUT-OF-SCOPE; if Benten's target customer base shifts, this becomes a BLOCKER not honest-disclosure.
  - **No UX-affordance lens** for how end-users will understand the Sealed-Sender opt-in (when default-codepoint is metadata-promiscuous and Sealed-Sender requires explicit user action; the "secure-by-default" framing may be misleading).
  - **No multi-stanza-HPKE specialist lens** for cross-stanza ciphertext-substitution (L3 named gap; L9 designed defense but no specialist confirmed). Should dispatch at R3/R5 design time when HpkeMultiBase variant lands per L3 §2.7 deferral.

---

## §9 Self-assessment (consolidator)

### What I did + how I worked
1. Tree-state pre-flight on worktree branch (clean against `2172cb6d`).
2. Fetched all 9 lens reviews + e2r scope-review into worktree scratch directory via `git show` (5,389 total LOC).
3. Read L1 + L2 in full; read L3-L9 in offset+limit chunks targeting key sections (executive verdict + amendments + compromise mints + invariants + self-assessment).
4. Built unified amendment registry by walking each lens's amendment list, identifying overlaps (e.g., L8/Am14 extends L2/Am1 + L3/Am3), and assigning unified numbers U1-U28.
5. Built unified Compromise registry by reconciling L3/§4.4 #32 + L5/§6.1 11 mints + L6/§6.1 metadata mint into #32-#44 + #31 extension.
6. Built unified invariant set by reconciling L2/§4.3 Inv-16 + L5 Inv-L5-2 + Inv-L5-3 + L6 Inv-16-metadata + L8 Am16 CodepointLifecycle into Inv-16, Inv-17, Inv-18.
7. Built disagreement matrix from 3 brief-named + 1 consolidator-surfaced + 1 ml-kem-crate-choice surfaced by L4/L5.
8. Built v1-beta-LOAD-BEARING subset via 4-bucket categorization per CLAUDE.md baked-in #5 + #15.
9. Drafted R0 skeleton + pattern-induction meta-findings.

### Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §2 Unified amendments 1-28 | **HIGH** | Each amendment traceable to origin lens + original number; dedup decisions documented. |
| §3 Unified Compromise registry | **HIGH** on count + structure; **MED-HIGH** on exact narrative text | Mints follow established L3 + L5 + L6 prose; Ben review of exact wording recommended. |
| §4 Unified invariants Inv-16/17/18 | **HIGH** on consolidation; **MED-HIGH** on Inv-18 merge of registry+metadata+lifecycle three siblings | The 3-way merge of L5+L6+L8 sibling invariants into single Inv-18 is the most-consolidator-judgment-heavy call. |
| §5 Disagreement matrix | **HIGH** on identification; **MED-HIGH** on consolidator recommendations (Ben final call) | Surfaced 5 forks (3 brief-named + 1 ml-kem-crate + 1 consolidator-surfaced). |
| §6 v1-beta-LOAD-BEARING subset | **HIGH** on categorization rule; **MED-HIGH** on specific assignments | U28 (coarse epoch) + U30 (DAG-CBOR) consolidator-leaned LOAD-BEARING; could shift if Ben prefers leaner v1-beta scope. |
| §7 R0 skeleton | **HIGH** on shape; **MED-HIGH** on §8 wave decomposition (depends on canary identification) | Skeleton is outline only; R0 author fills in. |
| §8 Pattern-induction meta-findings | **MED-HIGH** | 5 cross-lens patterns; MF5 self-critique is consolidator-disclosure of coverage gaps. |

### What I could be wrong about
1. **Inv-18 merging L5/L6/L8 sibling invariants may be over-consolidated.** Splitting Inv-18 into Inv-18a (registry-discipline) + Inv-18b (metadata-disclosure) + Inv-18c (CodepointLifecycle) might match the lens-origin-cleanliness better. Consolidator merged for compactness; Ben call.
2. **§5 Q5 (Sealed-Sender as v1-beta-default-codepoint)** — I leaned with L6 (additive slot) but a privacy-aggressive reading could ratify default-at-v1-beta. The cost asymmetry favors additive (~5-8 wave-days less); the marketing-posture asymmetry may favor default.
3. **U30 DAG-CBOR LOAD-BEARING promotion** — L7 stopped at RECOMMENDED; I leaned LOAD-BEARING per BLAKE3-CID + Atrium-sync alignment argument. Reasonable cryptographers may disagree. Ben call.
4. **U21 ExecuteWorkflow v1-beta inclusion** — L9 rated MED for v1-beta-day-one (slot-reservation-only); some readings might defer entirely. Consolidator leaned v1-beta-LOAD-BEARING for the slot lock (~zero cost) because hyper-scaling wire-break post-v1-beta is expensive.
5. **Cost estimate (~65-72 wave-days)** — propagated from L4 + L5 + incremental L9/L6/L7 adds. Could be off by ±20%; v1-beta 7-15 week window tolerates this.

### Lower-confidence areas (honest disclosure)
- I did NOT independently verify L7's `0xeb51`/`0x120c` multicodec table claims against the live table at HEAD; I propagated L7's verification.
- I did NOT cross-check L4's libcrux-ml-kem API compatibility claim vs RustCrypto ml-kem; I propagated L4's "API shape similar" estimate.
- I did NOT enumerate every L4/L5/L6/L7/L8/L9 minor observation; consolidator focused on amendments + Compromise mints + invariants per brief task scope. Some "observation O1..O8" rows in L9 and similar in L8 are summarized but not preserved as separate registry entries.
- The §7 R0 skeleton wave decomposition is illustrative not load-bearing; R0 author should re-evaluate canary identification.

### What this consolidation does NOT cover
- Per brief: "You are NOT a 10th reviewer. You are a CONSOLIDATOR." No new amendments added beyond consolidating the 9 lenses' work.
- Did NOT re-do construction soundness analysis (L1 + L2 + L3 are authoritative).
- Did NOT re-do impl-engineering analysis (L4 is authoritative).
- Did NOT re-do audit-readiness analysis (L5 is authoritative).
- Did NOT re-do privacy analysis (L6 is authoritative).
- Did NOT re-do cross-ecosystem analysis (L7 is authoritative).
- Did NOT re-do permanence analysis (L8 is authoritative).
- Did NOT re-do Atrium-integration analysis (L9 is authoritative).

---

## §10 Citations

### §10.1 Lens reviews (frozen SHAs)
- L1: `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — `.addl/phase-4-meta/option-f-plus-pseudo-keypair-review.md`
- L2: `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — `.addl/phase-4-meta/option-f-plus-second-opinion-cryptographer-review.md`
- L3: `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — `.addl/phase-4-meta/option-f-plus-third-reviewer-adversarial-design.md`
- L4: `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering @ 4d4aae5f` — `.addl/phase-4-meta/option-f-plus-lens-l4-impl-engineering.md`
- L5: `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0` — `.addl/phase-4-meta/option-f-plus-lens-l5-threat-model-audit-readiness.md`
- L6: `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak @ 986e50bb` — `.addl/phase-4-meta/option-f-plus-lens-l6-privacy-metadata-leak.md`
- L7: `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop @ 208f98bb` — `.addl/phase-4-meta/option-f-plus-lens-l7-cross-ecosystem-interop.md`
- L8: `phase-4-meta-core/option-f-plus-lens-l8-wire-format-stability @ d8d3c41c` — `.addl/phase-4-meta/option-f-plus-lens-l8-wire-format-stability.md`
- L9: `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03` — `.addl/phase-4-meta/option-f-plus-lens-l9-atrium-integration.md`
- e2r scope review: `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — `.addl/phase-4-meta/e2r-ffull-scope-review.md`

### §10.2 Benten internal references
- `docs/INVARIANT-COVERAGE.md` Inv-15 (registered, partial-enforcement); Inv-16/17/18 pending this consolidation.
- `docs/SECURITY-POSTURE.md` Compromise #6 + #30 + #31; #32..#44 pending mint per §3.
- `docs/V1-FROZEN-INTERFACE.md` + `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (item 6 + future Row D-SS-1 + D-PAD-1 + D-COVER-1).
- CLAUDE.md baked-in #5 (crypto-agility codepoint-dispatch); #15 (v1-beta + v1-GM gates); #17 (deployment-shapes); #18 (authority-isolation vs confidentiality-isolation).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` discipline.
- `feedback_review_finding_ground_truth_verify` discipline.
- `feedback_pim_cross_language_rule_mirror` §3.5g (load-bearing for U33).
- `feedback_canary_first_parallel_implementation` (load-bearing for §7 §8 wave decomposition).
- `feedback_orchestrator_defer_prediction_bias` (load-bearing for §6 do-it-now defaults).
- `.addl/dispatch-conventions.md` §3.5s (cross-ecosystem-identifier-as-content discipline; load-bearing for U29).

### §10.3 External standards + drafts (consolidated from 9 lenses)
- RFC 9180 HPKE; RFC 9106 Argon2; RFC 8439 ChaCha20-Poly1305; RFC 8949 CBOR; RFC 9420 MLS; RFC 7515 JWS; RFC 8152 COSE; RFC 9001 + 9000 QUIC; RFC 9458 OHTTP; RFC 7517 JWK.
- FIPS 203 ML-KEM; FIPS 140-3 CMVP.
- draft-connolly-cfrg-xwing-kem-10; draft-irtf-cfrg-concrete-hybrid-kems-03; draft-ietf-hpke-pq-04; draft-ietf-mls-pq-ciphersuites-04; draft-skokan-jose-hpke-pq-pqt-02; draft-reddy-cose-jose-pqc-hybrid-hpke-11; draft-ietf-jose-hpke-encrypt-11/15; draft-ietf-jose-pqc-kem-05; draft-ietf-lamps-pq-composite-sigs-19; draft-sfluhrer-cfrg-ml-kem-security-considerations-04.
- NIST SP 800-227 (final, September 2025).
- ANSSI Follow-Up Position Paper on PQC (2023); BSI TR-02102-1 + Migration to PQC; FedRAMP Crypto Module Policy v1.1.0.

### §10.4 Academic + production references
- Arriaga-Barbosa-Boyen "Tempo" IACR ePrint 2025/1399.
- Bernstein-Persichetti "One Time is Enough" IACR 2024/2051.
- Barbosa-Connolly-Diniz-Kahl-Krämer X-Wing IACR CIC 2024.
- Bellare-Pointcheval-Rogaway OPAQUE IACR 2018/163.
- PQShield "Formally verifying AVX2 rejection sampling for ML-KEM".
- libcrux / hax / Cryspen ML-KEM verification.
- Signal Sealed Sender 2018 + V2 2024; Signal PQXDH.
- age (Filippo Valsorda) + age discussion #463; Saltpack v2; Bitwarden + 1Password + Molly + Stronghold whitepapers; ATProto cryptography spec; multicodec table.csv at master HEAD.
- Trail of Bits Sigstore agility post; Cure53/ToB Obsidian Sync audits; Verification Theatre on hpke-rs.
- Narayanan-Shmatikov 2008 (deanonymization); Sweeney 2002 (k-anonymity).

---

**End of 9-eyes consolidated registry.**
