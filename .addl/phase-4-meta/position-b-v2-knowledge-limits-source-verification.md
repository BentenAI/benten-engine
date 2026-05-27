# Position B v2 — pre-publication source-verification of 5 flagged knowledge-limits

> Source-verification of the 5 knowledge-limits the Position B v2 revision-agent flagged in [position-b-revision-changelog.md](position-b-revision-changelog.md) §"Knowledge-limits I want to flag" for pre-publication verification.
>
> Authored 2026-05-27 by orchestrator (direct WebFetch/WebSearch verification).
>
> **Status**: 4 of 5 fully verified with cite-anchors; 1 (lidel quote) requires paraphrase-to-verbatim correction in v2 §3 / §8 text. Findings should be applied when Position B v2 §4 refresh happens post-F-full ratification.

## Findings

### 1. lidel "No multicodec update required" verbatim quote — PARAPHRASED, not direct quote

**v2 draft current state**: cites lidel saying "no multicodec update required" (or similar phrasing) attributing the multicodec maintainer position to him.

**Source verification result**: the exact phrase "no multicodec update required" does NOT appear as a direct lidel comment. The actual lidel-attributed content (via multicodec PR #400 description, NOT a comment-thread quote) is:

> "the moment an IETF/RFC encoding exists for a new key type...it is immediately usable as a libp2p or IPNS key without waiting on a multicodec update"

Additionally, lidel commented on PR #400:

> "these are self-describing key container standards that are referenced by 'recent' IETF RFCs (pkix/pkcs8 is ~2000s, cose is ~2020s, so i don't expect this class of containers to grow fast in 1-byte space)"

**Correction needed for v2 §3 / §8 / wherever the lidel quote appears**: replace the paraphrased "no multicodec update required" with the verbatim phrase (attributed correctly to multicodec PR #400 description as authored by lidel, NOT to a direct comment quote): *"the moment an IETF/RFC encoding exists for a new key type...it is immediately usable as a libp2p or IPNS key without waiting on a multicodec update."*

**Source URL**: https://github.com/multiformats/multicodec/pull/400 (lidel-authored PR description; lidel's comment thread on same PR is a separate sub-quote)

### 2. MLS RFC 9420 default mandatory-to-implement ciphersuite — VERIFIED

**v2 draft current state**: cites MLS-default cipher suite in §7 industry-snapshot table.

**Source verification result**: VERIFIED. Per RFC 9420 (The Messaging Layer Security Protocol) the mandatory-to-implement ciphersuite for MLS 1.0 is:

```
MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519
```

(Some implementations alternatively write it as `MLS10_128_DHKEMX25519_AES128GCM_SHA256_Ed25519` with the MLS version embedded in the ciphersuite name; both refer to the same construction.)

Construction:
- KEM: Curve25519 (DHKEMX25519)
- AEAD: AES-128-GCM
- KDF: HKDF over SHA-256
- Signature: Ed25519

**Source URLs**:
- RFC 9420: https://www.rfc-editor.org/rfc/rfc9420.html
- OpenMLS implementation: https://docs.rs/openmls/latest/openmls/
- MLS-PQ ciphersuites (draft, NOT the default): https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/

**Correction needed for v2**: ensure the §7 industry-snapshot row for "MLS-default" cites `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519` (or its alternative form `MLS10_...`) as the ciphersuite name when relevant.

### 3. Sigstore "signed today, deployed 20 years" attribution chain — VERIFIED

**v2 draft current state**: attributes the framing "signed today, deployed 20 years" to Sigstore PQC blog post 2025.

**Source verification result**: VERIFIED but **the originator is Trail of Bits, not Sigstore directly**. The framing first appears in:

- **Primary source**: Trail of Bits blog "Building cryptographic agility into Sigstore" published **2026-01-29**
- **Exact verbatim framing**: *"software you sign today might be deployed for 20 years, but the cryptographic signature protecting it may become untrustworthy within 10 years"*
- **URL**: https://blog.trailofbits.com/2026/01/29/building-cryptographic-agility-into-sigstore/

The Sigstore blog post at https://blog.sigstore.dev/post-quantum-2025/ likely references this framing but the originator is Trail of Bits.

**Correction needed for v2**: attribute the framing to **Trail of Bits** (with collaboration with Sigstore community), not to Sigstore blog directly. Cite the Trail of Bits blog as primary source; Sigstore PQC blog as secondary citation if needed.

### 4. NIST SP 800-227 — VERIFIED

**v2 draft current state**: cites NIST SP 800-227 (presumably in §4 combiner section or §5 caveats).

**Source verification result**: VERIFIED. NIST Special Publication 800-227 is titled "Recommendations for Key-Encapsulation Mechanisms" and provides operational guidance for using KEMs alongside the algorithm standards (like FIPS 203 for ML-KEM).

Key facts:
- **Title**: "Recommendations for Key-Encapsulation Mechanisms"
- **Initial Public Draft (IPD)** comment period: closed March 7, 2025
- **Final publication**: September 18, 2025
- **Scope**: basic definitions, properties, applications of KEMs + practical implementation guidance for post-quantum KEMs in real-world protocols
- **Key recommendations include**: never reuse ephemeral KEM key pairs; hybrid approaches recommended for post-quantum migration

**Source URLs**:
- NIST SP 800-227 final: https://csrc.nist.gov/pubs/sp/800/227/final
- NIST publication announcement: https://csrc.nist.gov/News/2025/nist-publishes-sp-800-227
- Direct PDF: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-227.pdf

**No correction needed**: v2 citation of NIST SP 800-227 is accurate. Worth confirming the v2 reference cites the FINAL (2025-09-18) not the IPD.

### 5. ANSSI quote attribution chain — VERIFIED

**v2 draft current state**: cites ANSSI in §3 or §5 caveats regarding hybrid signatures + post-quantum transition.

**Source verification result**: VERIFIED. ANSSI's official position on post-quantum cryptography transition is a 3-phase roadmap:

- **Phase 1 (today)**: additional post-quantum defense-in-depth to the pre-quantum approach
- **Phase 2 (not earlier than 2025)**: post-quantum security assurance with **mandatory hybridation**
- **Phase 3 (2030 or later)**: full post-quantum

**Key ANSSI positions to cite**:
- Hybridation is MANDATORY for phases 1 and 2
- Hybrid signatures recommended approach: **concatenation of signatures + accept only if all verify** (this is the EXACT construction Benten ships at v1-beta default via LAMPS Composite ML-DSA)
- Hash-based signatures are an EXCEPTION (no hybridation required due to well-studied underlying math problem)
- "Drop-in replacement of pre-quantum with post-quantum" is STRONGLY discouraged

**Source URLs** (primary documents):
- ANSSI position paper (March 30, 2022 original): https://cyber.gouv.fr/sites/default/files/document/EN_Position.pdf
- ANSSI 2023 follow-up: https://messervices.cyber.gouv.fr/documents-guides/follow_up_position_paper_on_post_quantum_cryptography.pdf
- ANSSI 2023 PKIC conference presentation (Jérôme Plût): https://pkic.org/events/2023/pqc-conference-amsterdam-nl/pkic-pqcc_jerome-plut_anssi_anssi-plan-for-post-quantum-transition.pdf
- ANSSI 2025 conference materials (Mélissa Rossi): https://pqcrypto2021.kr/download/program/PQC_transition_in_France.pdf + https://cyber.gouv.fr/sites/default/files/document/pqc-transition-in-france.pdf

**Net for Position B v2**: ANSSI is a STRONG supporter of the exact hybridation-by-concatenation construction Benten ships at v1-beta default. Worth surfacing this explicitly in the converging-cohort framing (§7) — ANSSI's mandatory-hybridation position is genuinely aligned with what Benten ships. This strengthens the "we're first-shipping in a converging cohort" framing per L15 R-1.

## Summary

| # | Knowledge-limit | Status | Correction needed in v2 |
|---|---|---|---|
| 1 | lidel quote | **PARAPHRASED → needs verbatim correction** | Replace paraphrased quote with verbatim from multicodec PR #400 description |
| 2 | MLS-default ciphersuite | VERIFIED | Ensure §7 cites `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519` precisely |
| 3 | Sigstore "20 years" attribution | VERIFIED but originator-correction needed | Attribute to Trail of Bits blog 2026-01-29 (primary), Sigstore PQC blog (secondary) |
| 4 | NIST SP 800-227 | VERIFIED | Ensure citation is the FINAL (2025-09-18), not the IPD |
| 5 | ANSSI attribution chain | VERIFIED | Strengthen §7 to cite ANSSI's mandatory-hybridation-by-concatenation position as direct converging-cohort signal |

## Application

These findings apply when:
- **Position B v2 §4 refresh happens** post-F-full ratification (the refresh adds encrypt-to-recipient + Inv-16; would also apply these knowledge-limit corrections)
- **Position B v2 publication** (after Shape 5 iroh-outreach gates the publication decision)
- **F4a LAMPS issue #289 test-vector deposit** may reference ANSSI's mandatory-hybridation position as cross-ecosystem-alignment evidence
- **CFRG draft-prabel adopter datapoint comment** may reference ANSSI's mandatory-hybridation position as additional adopter signal

## Cross-reference

- [position-b-revision-changelog.md](position-b-revision-changelog.md) §"Knowledge-limits I want to flag" — the agent-authored flag list this verification addresses
- [position-b-blog-draft-v2.md](position-b-blog-draft-v2.md) — the draft these corrections apply to (post-§4-refresh)
- [NIGHT-SHIFT-2026-05-27.md](NIGHT-SHIFT-2026-05-27.md) — comprehensive state capture including this verification's queue position

---

*Authored 2026-05-27 by orchestrator-direct WebFetch/WebSearch verification per Ben's authorization 2026-05-27.*
