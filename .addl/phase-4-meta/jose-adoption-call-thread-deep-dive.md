# JOSE WG adoption call — `draft-skokan-jose-hpke-pq-pqt-05` — full thread deep-dive

**Scan date:** 2026-05-26
**Analyst:** IETF mailing-list-archaeology agent
**Scope:** Karen O'Donoghue's adoption-call thread + cited background threads + JOSE-archive parallel-discussion enumeration + cross-reference with Benten's other draft comments
**Discipline:** READ-ONLY; every load-bearing position cite-anchored to an archive URL + date + verbatim quote where available
**Inputs read:**
- `.addl/phase-4-meta/f3-jose-comment-draft.md` (Benten's drafted comment; on `phase-4-meta-core/orchestration-2026-05-26`)
- `.addl/phase-4-meta/comment-opportunities-ietf-pq-wgs.md` (IETF-ecosystem-scout findings; on `phase-4-meta-core/comment-opps-ietf-pq-wgs`)

---

## §1 — Executive summary

The adoption call is on a strong, uncontested path to WG adoption. Karen O'Donoghue (JOSE co-chair) opened the call on 2026-05-14 with deadline 2026-05-29. As of 2026-05-26 (3 days remaining), the thread shows **seven substantive replies, all expressing "I support adoption" with no objection, no technical concern raised, no counter-proposal, and no procedural pushback** — Filip Skokan (author/Okta), Neil Madden (independent JOSE contributor), Brian Campbell (co-author/Ping Identity), Aaron Parecki (Okta/OAuth WG), tirumal reddy (co-author/Nokia), Orie Steele (Transmute), and Michael Jones (independent / JWS+JWE co-author). The prior path-forward thread (msg07049, April 22) documented a successful cross-WG handshake with the competing COSE-side authors — Skokan + Campbell on the JOSE side, Reddy on the COSE side — agreeing to split JOSE registrations into this draft and COSE registrations into a sibling `draft-reddy-cose-hpke-pq-pqt`; co-chair Karen O'Donoghue endorsed that resolution publicly ("I'm so pleased to see this outcome"). The construction itself is non-controversial in this WG — it does NOT define a new combiner; it registers JOSE-side algorithm IDs (HPKE-8 through HPKE-13, ten codepoints total) pointing at the upstream `I-D.ietf-hpke-pq` CFRG HPKE construction. **No EUF-CMA vs SUF-CMA discussion has appeared in this thread** (that conversation lives on CFRG list around `draft-prabel-cfrg-suf-hybrid-sigs-01`, not here). **No cross-stack algorithm-naming-consistency concern has been raised** by any respondent. **No multicodec container-form discussion has appeared.** **No pushback against adoption.** Benten's drafted comment fits the thread shape cleanly: humble, adopter-data-point, supportive, cross-stack alignment ask. The only refinement worth considering is a small WG-vocabulary precision pass + a structural choice on whether to include any technical content beyond pure support (this thread's tone is markedly terse — every other reply is one sentence). **Recommendation: post the drafted comment after a small vocabulary precision pass; specifically watch the tone-vs-length balance because the rest of the thread is one-sentence support and Benten's ~190-word comment will stand out visibly — that's defensible because we're adding a concrete adopter datapoint, not just +1ing, but worth a deliberate choice rather than incidental.** **Most load-bearing findings:** (1) the path-forward cross-WG handshake is the consensus baseline this call ratifies, so any disagreement with that allocation would be the only kind of comment that opens process complexity — Benten's draft doesn't go there, good; (2) "HPKE-8-KE" naming convention (the `-KE` suffix marks Key Encryption mode vs the unsuffixed Integrated Encryption mode) is the JOSE-specific vocabulary worth noting if Benten cites these codepoints by name; (3) ML-KEM-512 is explicitly excluded per security considerations (novelty concern) — Benten's X-Wing at ML-KEM-768 is on the kept-list, supportive datapoint.

---

## §2 — Thread map: Karen O'Donoghue's adoption call

**Thread anchor:** [`[jose] Call for adoption: draft-skokan-jose-hpke-pq-pqt-05 (Ends 2026-05-29)`](http://www.mail-archive.com/jose@ietf.org/msg07070.html) — Karen O'Donoghue via Datatracker — Thu, 14 May 2026 15:06 PDT — IETF archive: `mid:177879634223.1398005.15768994326346374720@dt-datatracker-54557f87b8-lnrkh`

The original call (auto-formatted by Datatracker; standard chair template):

> "WG members, please respond to this call for adoption with your thoughts on whether or not the JOSE working group should adopt this document. Also, please indicate your willingness to review and/or contribute to the document."

The call note also paraphrases the cross-WG handshake from the path-forward thread: "draft-skokan-jose-hpke-pq-pqt will be the draft to register PQ and PQ/T hybrid algorithm identifiers for JWE" in JOSE, while a sibling draft "will register PQ and PQ/T hybrid algorithm identifiers for CBOR" in COSE.

### Reply table

| # | Date (PDT) | Author | Affiliation | Archive URL | Verbatim position | Classification |
|---|---|---|---|---|---|---|
| R1 | 2026-05-20 00:52 | Filip Skokan | Okta; draft author | [msg07077](http://www.mail-archive.com/jose@ietf.org/msg07077.html) | "I support adoption." | supporting-adoption (author) |
| R2 | 2026-05-20 03:57 | Neil Madden | Independent JOSE contributor; author of `draft-ietf-jose-deprecate-none-rsa15` | [msg07078](http://www.mail-archive.com/jose@ietf.org/msg07078.html) | "I support adoption too." | supporting-adoption |
| R3 | 2026-05-20 04:20 | Brian Campbell | Ping Identity; draft co-author | [msg07079](http://www.mail-archive.com/jose@ietf.org/msg07079.html) | "I too support adoption." | supporting-adoption (co-author) |
| R4 | 2026-05-20 06:17 | Aaron Parecki | Okta; OAuth WG co-chair | [msg07080](http://www.mail-archive.com/jose@ietf.org/msg07080.html) | "I support adoption as well." | supporting-adoption |
| R5 | 2026-05-20 07:51 | tirumal reddy | Nokia; draft co-author + author of competing COSE draft per path-forward | [msg07081](http://www.mail-archive.com/jose@ietf.org/msg07081.html) | (Support; brief — verbatim not captured in archive snippet) | supporting-adoption (co-author) |
| R6 | 2026-05-20 14:05 | Orie (Steele) | Transmute Industries; long-time JOSE/COSE/DID contributor; DE for JWS/JWE algs | [msg07082](http://www.mail-archive.com/jose@ietf.org/msg07082.html) | "I support adoption of the draft." | supporting-adoption |
| R7 | 2026-05-26 00:02 | Michael Jones | Independent (formerly Microsoft); JWS/JWE original author + JOSE long-time contributor | [msg07087](http://www.mail-archive.com/jose@ietf.org/msg07087.html) | "I support adoption." | supporting-adoption |

**Notable absences (as of 2026-05-26 morning PDT):**
- The fourth draft co-author **Hannes Tschofenig** (UniBw München) has not yet replied in this adoption-call thread (he is a co-author and his support is presumed; not yet recorded).
- Chair **Michael P** (Michael P1, NCSC UK; identified via msg07071 IETF-126 meeting-session-request as JOSE chair) has not replied.
- Co-chair **John Bradley** has not replied.
- **Ilari Liusvaara** (heavy commenter on the parallel `draft-ietf-jose-hpke-encrypt-17` IETF-LC thread, msg07076) has not replied to the adoption call.

**Knowledge limit:** msg07081 (tirumal reddy reply) was only partially-quoted in the WebFetch retrieval — the body confirmed "expressed support" but didn't surface a verbatim string. If the verbatim phrasing is load-bearing for the comment shape, fetch the IETF-archive mirror directly before posting.

### Position distribution

- **Supporting adoption:** 7 / 7 substantive replies (100%)
- **Opposing adoption:** 0
- **Asking question / asking-for-revision:** 0
- **Neutral comment / off-topic:** 0
- **Procedural (e.g. chair nudges):** 0 beyond the original Karen call itself

**Consensus signal:** Strong, low-friction, no-dissent — chairs will almost certainly declare adoption on or shortly after 2026-05-29.

---

## §3 — Background threads directly cited or relevant

### §3.1 "PQ & PQ/T HPKE JWE path forward" thread (msg07049, April 22 2026)

[`[jose] PQ & PQ/T HPKE JWE path forward, draft-skokan-jose-hpke-pq-pqt-04 published; request for WG adoption call`](http://www.mail-archive.com/jose@ietf.org/msg07049.html) — Filip Skokan — Wed, 22 Apr 2026 03:22 PDT

This thread is **the consensus baseline the adoption call ratifies.** Skokan announces the cross-WG handshake outcome from IETF 125 (Shenzhen, March 2026):
- The previously-competing drafts (`draft-skokan-jose-hpke-pq-pqt` + Reddy's `draft-reddy-cose-jose-pqc-hybrid-hpke`) split into a JOSE-only document (this one) and a COSE-only sibling.
- Hannes Tschofenig + Tiru Reddy joined Skokan + Campbell as co-authors on the JOSE-side draft to consolidate the four contributors.
- Substantive -04 revision changes: algorithm-ID alignment between JOSE and COSE drafts; ChaCha20Poly1305 variants removed (only AES-256-GCM AEAD + SHAKE256 KDF retained); enhanced HNDL + multi-recipient security guidance; terminology aligned with RFC 9794.
- Skokan formally requested WG adoption call.

**Karen O'Donoghue reply** ([msg07050](http://www.mail-archive.com/jose@ietf.org/msg07050.html), 22 Apr 2026 06:49 PDT):

> "I'm so pleased to see this outcome. Thanks to all the authors for your contributions and your collaboration."

— this is the chair signal that the path-forward was the right answer + that an adoption call was forthcoming. The adoption call (msg07070) followed three weeks later.

### §3.2 IANA early-review collision thread (msg07032, March 18 2026)

[`[jose] [IANA #1446158] Early review: draft-ietf-jose-pq-composite-sigs-01 (IETF 125)`](http://www.mail-archive.com/jose@ietf.org/msg07032.html) — Amanda Baber (IANA) via RT — Wed, 18 Mar 2026 20:28 PDT

**NOT directly about the Skokan draft.** This thread is about a DIFFERENT draft — `draft-ietf-jose-pq-composite-sigs-01` (Prabel/Sun/Gray/Reddy; WG-adopted Jan 2026; sibling of the Skokan draft but on the signature side, not the HPKE-encryption side). The collision is on **COSE algorithm values -51, -52, -53** which were already assigned to RFC 9864 but were re-requested by the pq-composite-sigs draft. IANA proposed two resolution paths: RFC-7120 early allocation OR placeholder TBD values with descending-order language.

**Relevance to Benten's comment:** the codepoint Benten plugs into on the signature side is `ML-DSA-65-Ed25519` (requested as COSE alg value **-55**), which is NOT in the collision zone. The collision affects only the ECDSA-pairing variants. Benten's drafted comment doesn't need to reference this thread directly.

**Knowledge limit:** the resolution of the collision is not visible from the available archive view; the IANA early-review RT ticket may be the only authoritative state.

### §3.3 Prabel composite-sigs adoption call (msg06919, Jan 5 2026)

[`[jose] Call for adoption: draft-prabel-jose-pq-composite-sigs-05 (Ends 2026-01-19)`](http://www.mail-archive.com/jose@ietf.org/msg06919.html) — Karen O'Donoghue via Datatracker — Mon, 5 Jan 2026 21:13 PST

The prior precedent for how JOSE WG adoption calls run. This call closed successfully on 2026-01-19; the draft was adopted as `draft-ietf-jose-pq-composite-sigs-00` (now at -01 per §4.2). Shape of the call message is identical to the current Skokan call — same Datatracker template, same chair (Karen).

Relevance: confirms the chair-procedural pattern. The Skokan call is following a well-trod pattern that previously concluded successfully.

### §3.4 Initial draft announcement (msg06962, Feb 10 2026)

[`[jose] New I-D: draft-skokan-jose-hpke-pq-pqt (JOSE HPKE PQ & PQ/T Algorithm Registrations)`](http://www.mail-archive.com/jose@ietf.org/msg06962.html) — Filip Skokan — Tue, 10 Feb 2026 00:35 PST

Skokan + Campbell announce -00. At this point ChaCha20Poly1305 variants were still in the draft (removed in -04 per §3.1).

**Notable reply** — [msg06969](http://www.mail-archive.com/jose@ietf.org/msg06969.html), Wed, 11 Feb 2026, Michael P1 (chair, NCSC UK):

> Supports Filip's minimalist approach. Proposes starting with five core combinations pairing PQ KEMs with matching-security-level AES variants (ML-KEM-768 + AES-128-GCM; ML-KEM-768 hybrid with P256/x25519 + AES-128-GCM; ML-KEM-1024 + AES-256-GCM; ML-KEM-1024 + P384 hybrid + AES-256-GCM). Cites ETSI TR 103 967 + CHES 2024 on AES-vs-Grover analysis, arguing AES-256-GCM uniform default is unjustified when 128-bit traditional component is involved.

This was a substantive technical pushback from a chair — it's relevant because the final draft (-05) **does** uniformly use AES-256-GCM + SHAKE256, NOT the mixed-AES-128/AES-256 approach Michael P proposed. The authors took a different decision; Michael P appears to have accepted that outcome (he has not re-raised the concern in the adoption call thread). This is a useful signal: technical issues that didn't get resolved exactly as a chair proposed are not blocking adoption.

### §3.5 Cross-WG coordination thread (msg07012, March 6 2026)

[`[jose] Re: [COSE] COSE and LAKE needs draft-ietf-jose-pqc-ke (was Proposal: Use HPKE for JWE PQ/PQT straight away)`](http://www.mail-archive.com/jose@ietf.org/msg07012.html) — Filip Skokan — Fri, 6 Mar 2026 11:23 PST

Skokan reports the WG postponed adoption of PQ HPKE algs pending `draft-ietf-jose-hpke-encrypt` completion (now in IESG Last Call as of late May 2026, see §8). Anticipates post-Shenzhen adoption of one of the two competing PQ-HPKE drafts. The thread reflects coordination between JOSE / COSE / LAKE on PQ KEM specs, with 3GPP awaiting RFC-status specs for production deployment.

Relevance: confirms the adoption-call timing was sequenced behind the base HPKE-JWE draft moving forward; provides downstream-adopter context (3GPP / LAKE) that Benten can quietly reference if useful (probably not — too tangential to the Skokan thread itself).

---

## §4 — Author identification

### Filip Skokan (Okta) — primary author

- Identity provider engineer at Okta; long-time JOSE WG contributor.
- DE (Designated Expert) for JOSE algorithm registry per his self-identification in [msg07075](http://www.mail-archive.com/jose@ietf.org/msg07075.html).
- Prior drafts: `draft-ietf-oauth-rar` (OAuth Rich Authorization Requests); various JOSE drafts.
- Maintainer of `panva/jose` JavaScript JOSE library (one of the most-used in the ecosystem).
- WG role: active editor + reviewer; in the ecosystem because Okta ships JOSE-format tokens at scale.

### Brian Campbell (Ping Identity) — co-author

- Distinguished engineer at Ping Identity.
- JOSE WG co-author on multiple PQ-related drafts (`draft-ietf-jose-hpke-encrypt`, `draft-skokan-jose-hpke-pq-pqt`).
- Active on parallel HPKE-JWE IETF Last Call thread arguing for algorithm-registry conservatism (see [msg07073](http://www.mail-archive.com/jose@ietf.org/msg07073.html), [msg07074](http://www.mail-archive.com/jose@ietf.org/msg07074.html)).
- Prior work: SAML / OAuth / OIDC editor + reviewer.

### Hannes Tschofenig (UniBw M.) — co-author

- Professor at Universität der Bundeswehr München.
- Long-time IETF contributor across HPKE / CBOR / COSE / JOSE / IoT-security.
- Joined this draft post-path-forward (msg07049) consolidation.
- Has not yet replied in the adoption call thread as of 2026-05-26.

### Tirumaleswar Reddy.K (Nokia) — co-author

- Senior researcher at Nokia.
- Author of the competing-then-merged `draft-reddy-cose-jose-pqc-hybrid-hpke` (folded into Skokan's draft + sibling COSE doc).
- Co-author on multiple PQ drafts (`draft-ietf-jose-pqc-kem`, `draft-reddy-pquip-pqc-signature-migration`).
- WG role: active across COSE / JOSE / PQUIP / TLS PQ.

---

## §5 — Position B-relevance map: Benten's drafted comment vs thread state

Benten's drafted comment makes the following substantive claims; mapping each to the current thread state:

| Claim in Benten draft | What thread says | Conflict / alignment / refinement |
|---|---|---|
| **"Supporting adoption"** | 7/7 replies support; chairs ratified path-forward | **Aligned.** Benten is adding to a unanimous-supporting chorus. |
| **"PQ-hybrid signatures via LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20)"** | No thread message references the LAMPS PKIX OID by number; cross-stack signature-side context is unstated in this thread | **Refinement opportunity** — the OID is a verifiable adopter datapoint. Citing it is fine. But verify accuracy: confirm OID `1.3.6.1.5.5.7.6.48` actually maps to `id-MLDSA65-Ed25519-SHA512` against the live IANA SMI registry. Comment-opps doc states the early-allocation date as 2025-10-20; verify against PKIX-Alg registry before posting (the scout's source was draft IANA-considerations + datatracker, not direct registry HTML). |
| **"Hybrid AEAD using an X-Wing-style combiner at multicodec codepoint `0x647A` (X25519 + ML-KEM-768)"** | The Skokan draft registers HPKE-style algs (HPKE-9 = MLKEM768-X25519); the construction is per `I-D.ietf-hpke-pq`, not X-Wing-style. **These are NOT the same construction.** | **POTENTIAL VOCABULARY DRIFT — needs sharpening.** Benten's X-Wing combiner uses the X-Wing draft (Aviram/Stebila CFRG `draft-irtf-cfrg-xwing`). The HPKE-PQ draft uses a different (HPKE-native) combiner approach. Calling them both "hybrid AEAD" is correct generically, but the comment as drafted may give the impression Benten is shipping the same construction the Skokan draft registers — it isn't. **Recommended fix:** make the relationship explicit: "Benten ships an X-Wing-style combiner (`draft-irtf-cfrg-xwing`) at multicodec codepoint `0x647A` separately; the JOSE-side HPKE-PQ-PQT registrations this draft establishes will be the JOSE-stack-side substrate when Benten exposes a JWE surface." This avoids the false-equivalence read AND positions Benten as a downstream-substrate consumer rather than a same-construction implementer. |
| **"The HPKE-PQ-PQT registration path mirrors the layering we're already using on the signature side"** | Thread doesn't discuss layering; reasonable adopter framing | **Fine** but slightly verbose. The thread's tone is one-sentence-support. Verbose framing makes Benten's comment stand out — defensible but worth deliberate. |
| **"Keeping algorithm-naming stable across the LAMPS / JOSE / COSE silos"** | This concern has NOT been raised in the thread. **No respondent has flagged cross-stack naming**. The closest analog is the IANA collision thread (§3.2) but that's a different draft. | **Aligned with path-forward thread + cross-stack handshake intent**, but Benten would be introducing this concern fresh into the adoption-call thread. That's defensible (adopter perspective) — but it's slightly unusual for adoption-call replies to introduce new substantive concerns. Frame as "endorse" rather than "ask" to avoid mis-reading as a hold. |
| **"Particularly endorsing the PQ/T hybrid framing"** | Thread doesn't have anyone questioning PQ/T-vs-pure-PQ; this framing is already settled in the draft. | **Aligned but redundant.** The draft includes both pure-PQ (HPKE-12, HPKE-13) and PQ/T-hybrid (HPKE-8, HPKE-9, HPKE-10) families — the framing is already settled. Benten's endorsement adds no new signal. Could be cut for length, OR could be retained as adopter-rationale evidence. |
| **"Aligns with LAMPS Composite ML-DSA + OpenPGP-PQC + Sigstore retooling"** | Thread doesn't reference Sigstore or OpenPGP. Cross-stack examples cited by the draft itself or other respondents: HPKE registry alignment with COSE / TLS PQ context | **Slightly outside-the-thread.** Sigstore + OpenPGP examples are legitimate adopter context but Benten is the only one who brought them up. Defensible but unusual. |

**Net assessment:** The drafted comment fits the thread (humble, supportive, adopter-data-point) but has **one material vocabulary issue** worth fixing before posting (the X-Wing-vs-HPKE-PQ-construction conflation). The cross-stack ask is fine but could be sharpened to "endorse alignment" rather than "please prioritize". Length is on the upper end relative to thread tone.

### EUF-CMA / SUF-CMA scope (Benten draft does NOT mention this)

- Benten's comment-opportunities doc (§2 row 4) names CFRG `draft-prabel-cfrg-suf-hybrid-sigs-01` as a separate engagement.
- The SUF-CMA discussion is **not present in this thread** (it's a CFRG list discussion, not JOSE).
- Benten's drafted comment doesn't raise SUF-CMA scope. **Correct** — would be cross-list-bleed; out of place here.

### Multicodec container-form direction

- Benten's drafted comment cites multicodec codepoint `0x647A` as an adopter datapoint.
- The thread doesn't discuss multicodec.
- This is fine as a self-cite ("here's where we ship in our system") but should NOT be framed as an ask for the JOSE WG to do anything with multicodec. The drafted text doesn't make that mistake — good.

### Concerns / pushback against adoption

- **NONE** in the adoption call thread.
- The closest adjacent technical pushback was Michael P1 in February (§3.4) on AES-128-vs-256 + ChaCha20Poly1305-removal — both resolved before the adoption call (ChaCha20Poly1305 removed in -04; AES-256-uniform kept). No re-raise.
- Brian Campbell's parallel argument on the base HPKE-JWE IETF-LC thread (§8.1, msg07073) about removing HPKE-4-KE / HPKE-6-KE is on a DIFFERENT draft (`draft-ietf-jose-hpke-encrypt-17/18`) and is the consistent-Brian "algorithm-registry conservatism" position — it doesn't apply to this draft's PQ codepoints.

**Bottom line: Benten will NOT conflict with anyone if it posts the drafted comment with the X-Wing-vs-HPKE-PQ vocabulary fix.**

---

## §6 — Consensus assessment

**Adoption trending:** STRONGLY toward adoption. 7-of-7 substantive replies support; no objections; chairs publicly ratified the path-forward back in April; the IETF-126 meeting-session request (msg07071, May 15) was filed by the chair the day after the adoption call opened, signaling continued forward momentum.

**Specific concerns being raised that Benten's comment should address:**
- **None identified in this thread.** The drafted comment doesn't need to address an existing concern; it's adding signal, not resolving one.

**Emerging WG-vocabulary the comment should mirror:**
- **"PQ" vs "PQ/T"** — the draft + thread consistently use "PQ" for pure post-quantum and "PQ/T" for hybrid (post-quantum/traditional). Benten's draft uses "PQ-hybrid" — slightly off-register vs the WG's "PQ/T" preference. **Recommended fix:** use "PQ/T hybrid" in body text when referring to the JOSE-side construction; "PQ-hybrid" is fine for Benten-internal description.
- **"HPKE-N" and "HPKE-N-KE"** — the JOSE-side algorithm names use the form HPKE-N (Integrated) and HPKE-N-KE (Key Encryption). If Benten cites a specific codepoint, use this form.
- **"hybrid algorithm identifiers for JWE"** — the chair's framing language in msg07070. Mirroring this phrasing is good vocabulary precision.
- **"adopter perspective" / "implementer perspective"** — both are reasonable; the WG culture favors "implementer" framing for technical claims and "adopter" for downstream-deployment claims. Benten's draft uses "adopter" which is fine.

**Effective chair signal so far:**
- Karen O'Donoghue ratified path-forward (§3.1).
- Michael P1 (chair) raised technical concerns in Feb (§3.4) that were not exactly resolved as he proposed; he has not re-raised, suggesting acceptance.
- John Bradley silent.
- **Net chair signal: green light, no procedural complications.**

**IETF-vocabulary-precision check (per §3.5q) on Benten's drafted comment:**
- "Call for adoption" — Benten's draft uses this correctly.
- "WG adoption" — fine.
- The comment does NOT confuse adoption-call with WGLC (Working Group Last Call) or IETF-LC. Good.
- The comment correctly refers to the draft by its full `draft-skokan-jose-hpke-pq-pqt-05` name in the preamble. Good.
- Suggestion: when describing what the draft does, prefer "registers post-quantum (PQ) and PQ/T hybrid algorithm identifiers for JWE" (mirrors chair language) over "PQ + PQ/T HPKE algorithm IDs for JWE/JWS" (which mentions JWS — this draft does NOT register JWS-side algorithms; that's `draft-ietf-jose-pq-composite-sigs-01`). **Benten's draft does not currently say JWS — verified clean.**

---

## §7 — Recommendation for the drafted comment

### §7.1 Does the current draft fit?

**Mostly yes.** Framing (humble + adopter-data-point + supportive + cross-stack ask) fits the thread. Length is on the long side relative to thread tone (every other reply is one sentence; Benten's draft is ~190 words). Length is defensible because Benten is contributing a concrete adopter datapoint, not just +1ing — but it's worth a deliberate choice rather than incidental.

### §7.2 Required fixes before posting

1. **Fix the X-Wing-vs-HPKE-PQ-construction conflation (load-bearing).** Current draft text:
   > "Hybrid AEAD using an X-Wing-style combiner at multicodec codepoint `0x647A` (X25519 + ML-KEM-768)"
   
   This implies (or could be read as implying) that Benten's X-Wing AEAD is the same construction the Skokan draft registers. It is not — the Skokan draft uses the HPKE-PQ combiner from `I-D.ietf-hpke-pq`, which is a different construction. **Recommended replacement:**
   > "Hybrid AEAD using an X-Wing-style combiner (`draft-irtf-cfrg-xwing`) at multicodec codepoint `0x647A` (X25519 + ML-KEM-768) at the storage-substrate layer."
   
   And separately, frame the JOSE-side relevance correctly:
   > "On the JOSE-stack interop side, the HPKE-PQ-PQT algorithm registrations this draft establishes give downstream consumers (every JOSE library, WebCrypto, JWKS endpoints) the same low-friction adoption path our LAMPS-side already gets from BouncyCastle 1.80+ / OpenSSL 3.5..."
   
   This says: Benten ships X-Wing at the storage substrate AND will consume the Skokan-draft codepoints when it exposes a JOSE surface. Avoids the same-construction implication.

### §7.3 Optional polish

1. **Vocabulary precision** — replace "PQ-hybrid" with "PQ/T hybrid" where referring to the JOSE-side draft (mirrors chair language).
2. **Length trim consideration** — the "particularly endorsing the PQ/T hybrid framing" paragraph adds little signal (the framing is already settled in the draft). Could be cut for length. **Alternative kept-reason:** retain to give the post a self-contained "what we're saying" arc rather than a bare data-dump. Defensible either way.
3. **Sigstore / OpenPGP cross-reference:** retain or cut. Retain if Benten wants to signal "we're across the broader ecosystem"; cut if the comment is feeling outside-the-thread. Net: retain feels stronger.
4. **Subject line:** suggested by the draft as `"Re: Call for adoption: draft-skokan-jose-hpke-pq-pqt"` — match what `mid:177879634223...` uses, which is the full bracketed `[jose] Call for adoption: draft-skokan-jose-hpke-pq-pqt-05 (Ends 2026-05-29)`. If posting via mail client with reply-in-thread, the existing thread subject auto-fills; just don't strip the `[jose]` tag or the `(Ends 2026-05-29)` suffix.

### §7.4 Sign-off context for Ben

- The comment should be sent from Ben's personal email or the Benten public maintainer identity. The `jose@ietf.org` list does not require subscription to post a reply, but posts from non-subscribers may be moderated; subscribing first is the conservative path.
- Reply-in-thread by replying to Karen's msg07070 (or any of the 7 existing replies, e.g. Mike Jones's msg07087 as the most-recent thread anchor). The replied-to identifier preserves threading.
- Disclose-then-post: include a brief role-disclosure if useful — `Speaking as a downstream adopter (Benten Engine — `, etc. — the draft already does this. Good.

### §7.5 Net recommendation

**POST with the §7.2 fix applied. The §7.3 polish items are optional.**

If Ben cannot review + post within the 2026-05-29 window, missing the deadline is low-cost — Benten still gets its adopter signal in via post-RFC-publication adopter registration (LAMPS / JOSE / COSE), and other F-track engagement paths remain.

---

## §8 — Parallel JOSE WG discussions, 2026-Q1/Q2 active state

This section surveys the broader JOSE archive over the last 90 days (late February through May 2026) to contextualize the Skokan adoption call within the WG's current workload.

### §8.1 Active drafts + their procedural state (from datatracker `wg/jose/documents/`)

| Draft | State | Most-recent activity | Benten relevance |
|---|---|---|---|
| `draft-skokan-jose-hpke-pq-pqt-05` | **Call for adoption** (ends 2026-05-29) | -05 published 2026-05-13; adoption call thread msg07070..msg07087 | **COMMENT-NOW** (this analysis) |
| `draft-ietf-jose-hpke-encrypt-18` | **Submitted to IESG / IETF Last Call** (ended 2026-05-27) | -18 published 2026-05-24 after -17 LC concluded; substantive Brian Campbell + Ilari Liusvaara push to remove HPKE-4-KE / HPKE-6-KE | **WATCH-ONLY** — Benten doesn't ship ChaCha20Poly1305 KE; the substrate this draft establishes is what the PQ-PQT draft layers on |
| `draft-ietf-jose-deprecate-none-rsa15-04` | **WG Last Call** (closed 2026-05-20) | Neil Madden author; minor editorial revisions only; no objection at IETF 125 | **NONE** — Benten never used `none` or `RSA1_5` |
| `draft-ietf-jose-pq-composite-sigs-01` | **WG-adopted Jan 2026** (`draft-prabel-...` → `-ietf-...`); IANA early-review pending collision resolution | -01 published 2026-02-27; IANA RT #1446158 open (msg07032) | **COMMENT-OPPORTUNITY-LATER** — if Benten exposes a JWS surface; codepoint `ML-DSA-65-Ed25519` requested COSE alg -55 not in collision zone |
| `draft-ietf-jose-pqc-kem-05` | **WG document; not yet WGLC** | -05 published 2025-12-08; replaces `draft-reddy-cose-jose-pqc-kem` | **WATCH-ONLY** — overlap with Skokan draft's KEM surface but anchored as standalone JWE KEM spec, not HPKE-based |
| `draft-ietf-jose-json-proof-algorithms-13` | WG document, in development | -13 2026-03-02 | **NONE** (JSON Web Proof; selective-disclosure / BBS+; outside Benten v1-beta scope; may matter Phase-7+ Kith) |
| `draft-ietf-jose-json-proof-token-13` | WG document, in development | -13 2026-03-02 | **NONE** (same) |
| `draft-ietf-jose-json-web-proof-13` | WG document, in development | -13 2026-03-02 | **NONE** (same) |

### §8.2 Active threads enumerated

| Thread | Opened / latest | Author(s) | Topic | Classification | Benten-relevance |
|---|---|---|---|---|---|
| [Adoption call Skokan -05](http://www.mail-archive.com/jose@ietf.org/msg07070.html) | 2026-05-14 / 2026-05-26 (msg07087) | Karen O'Donoghue + 7 replies | Adoption call for HPKE-PQ-PQT draft | **active-adoption-call** | **COMMENT-OPPORTUNITY-NOW** (this analysis) |
| [WG Last Call deprecate-none-rsa15](http://www.mail-archive.com/jose@ietf.org/msg07057.html) | 2026-05-06 / 2026-05-20 | Karen O'Donoghue + Madden + several reviewers | Deprecate `none` + `RSA1_5` algs | **WGLC (closed)** | NONE |
| [IETF Last Call HPKE-JWE -17](http://www.mail-archive.com/jose@ietf.org/msg07073.html) | 2026-05-18 / 2026-05-23 | Campbell + Liusvaara + Reddy + Kyzivat ART-ART review | Substantive technical push to remove HPKE-4-KE / HPKE-6-KE before publication | **IETF-LC active (closed 2026-05-27)** | WATCH-ONLY — substrate Benten plugs into when JOSE surface is added |
| [IANA early-review pq-composite-sigs](http://www.mail-archive.com/jose@ietf.org/msg07032.html) | 2026-03-18 / open | Amanda Baber (IANA) | COSE alg -51..-53 collision with RFC 9864 | **IANA-action** | WATCH-ONLY |
| [PQ & PQ/T HPKE path forward](http://www.mail-archive.com/jose@ietf.org/msg07049.html) | 2026-04-22 / 2026-04-22 | Skokan + Karen O'Donoghue | Cross-WG handshake with Reddy's COSE draft | **cross-WG-coordination (closed)** | Context for current adoption call |
| [IETF 126 meeting session request](http://www.mail-archive.com/jose@ietf.org/msg07071.html) | 2026-05-15 / 2026-05-15 | Michael P1 (chair) | Schedule request | **chair-procedural** | NONE |
| [JOSE WG Report IETF 125](http://www.mail-archive.com/jose@ietf.org/msg07033.html) | 2026-03-19 / 2026-03-19 | Michael P1 | Post-meeting report | **chair-procedural** | NONE |
| [I-D Action hpke-encrypt-18](http://www.mail-archive.com/jose@ietf.org/msg07086.html) | 2026-05-25 / 2026-05-25 | internet-drafts (bot) | -18 publication notice | **administrative** | NONE |
| [I-D Action skokan-hpke-pq-pqt-05](datatracker only) | 2026-05-13 | internet-drafts | -05 publication | **administrative** | (anchor for adoption call) |
| [Art-ART review hpke-encrypt-17](http://www.mail-archive.com/jose@ietf.org/msg07084.html) | 2026-05-23 / 2026-05-23 | Paul Kyzivat (ART-ART reviewer) | Procedural review: "Ready for publication"; no concerns | **directorate-review** | NONE |
| [Weekly GitHub digest](http://www.mail-archive.com/jose@ietf.org/msg07085.html) | weekly | bot | WG repos activity | **administrative** | NONE |

### §8.3 Notable cross-WG signals

- **JOSE / COSE coordination is healthy.** The path-forward thread (§3.1) is a strong example of cross-WG handshake resolving competing drafts. Reddy's COSE-side sibling `draft-reddy-cose-hpke-pq-pqt` is the COSE-side companion that registers the same algorithm-IDs (CBOR side).
- **JOSE / LAMPS alignment is bidirectional.** `draft-ietf-jose-pq-composite-sigs-01` explicitly anchors on LAMPS Composite ML-DSA constructions. Benten's v1-beta default (`id-MLDSA65-Ed25519-SHA512`) is the LAMPS-side OID; the JOSE-side alg name `ML-DSA-65-Ed25519` is the published label. Cross-stack-naming-stability is observable in this alignment.
- **CFRG / JOSE separation is intentional.** SUF-CMA combiner discussion (`draft-prabel-cfrg-suf-hybrid-sigs-01`) is on CFRG list, not JOSE. The JOSE WG is operating at the codepoint-registration + JWE/JWS-format layer, not the cryptographic-construction-design layer.
- **LAKE / 3GPP downstream is waiting.** Per msg07012 (§3.5), 3GPP and LAKE WG implementers are waiting for these JOSE drafts to reach RFC status for production deployment — gives the WG external-deployment pressure to land the PQ stack cleanly.

### §8.4 Knowledge limits

- The JOSE archive view returned only 36 messages from the `skokan-jose-hpke-pq-pqt` search facet but the archive contains ~7,659 total messages across the WG; some sub-threads (e.g. specific GitHub-PR comments mirrored to the list, off-WG-list mentions) may have been missed.
- The IETF official archive at `mailarchive.ietf.org/arch/...` was only partially scraped (some search URLs returned 403); the mail-archive.com mirror was the primary data source. For load-bearing claims, both mirrors should be cross-verified.
- Interim meeting transcripts / virtual interim coordination for JOSE between IETF 125 (Shenzhen) and IETF 126 (planned) were not confirmed present.

---

## §9 — Cross-reference with Benten's other comment drafts

The orchestration branch holds ~20 comment drafts (per request context) at `.addl/phase-4-meta/new-comment-drafts.md` + `.addl/phase-4-meta/f2-f7-comment-drafts.md`. Drafts NOT directly read by this agent — cross-references below are based on the IETF-ecosystem-scout's notes in `comment-opportunities-ietf-pq-wgs.md` + the f3 draft's preamble references.

### §9.1 multicodec PRs #400 / #403 container approach (F2 drafts)

- **JOSE thread search:** No JOSE-archive thread references multicodec or content-addressing primitive discussions in the surveyed window (Feb-May 2026).
- **Cross-link:** Not applicable here. The JOSE WG operates at the JOSE-format / JWE / JWS layer; multicodec is a content-addressing layer below that. **No adjacent-thread context discovered.**

### §9.2 W3C did-method-key #70 / #74 PQ discussion (F7 drafts)

- **JOSE thread search:** No direct discussion of `did:key` or W3C DID-method-key issue numbers in the surveyed JOSE-archive window.
- **Indirect:** Orie Steele (msg07082) is a long-time DID/credentials community contributor; his support for the Skokan draft is from that ecosystem position. If Benten's F7 comment cites the Skokan adoption pattern as an analog, that's a defensible weak cross-reference.
- **Cross-link strength:** WEAK.

### §9.3 LAMPS draft-19 RFC-Editor queue state (F4a planning context)

- **JOSE thread search:** The path-forward thread (§3.1) and various co-author contexts mention LAMPS Composite alignment but don't drill into RFC-Editor queue specifics.
- **Cross-link:** The Skokan-draft adoption being unanimous is a positive signal for LAMPS-side adopter framing — confirms the JOSE-stack is on the same trajectory. **Refinement for F4a:** when LAMPS-19 publishes as RFC, Benten's adopter-registration can cite the JOSE-side parallel motion as cross-stack momentum.
- **Cross-link strength:** MODERATE (context for F4a, not a direct adjacency).

### §9.4 CFRG `draft-prabel-cfrg-suf-hybrid-sigs` SUF-CMA direction

- **JOSE thread search:** No SUF-CMA discussion in the JOSE-archive window. As noted in §5, this is intentional WG separation — CFRG handles cryptographic-construction; JOSE handles codepoint-registration.
- **Cross-link:** None direct. **Refinement:** Benten's CFRG-list datapoint comment can position the JOSE-side codepoint stability as the downstream-consumer angle — i.e. "we'd consume whichever combiner ships as RFC via the JOSE / COSE codepoints" — without conflating the two list conversations. Worth being explicit in the CFRG draft that the JOSE-side direction is settling separately.
- **Cross-link strength:** WEAK-MODERATE (useful framing distinction).

### §9.5 Adjacent observations for the broader comment-drafts review

- **Orie Steele** (msg07082, R6 in adoption-call thread) is a load-bearing connector between JOSE / COSE / DID / Sigstore communities. If Benten has any draft engaging the Sigstore / DID side, his name appears across all of them. Useful relationship-mapping datapoint.
- **Brian Campbell**'s algorithm-registry-conservatism position (msg07073, msg07074 — push to remove HPKE-4-KE / HPKE-6-KE from the base HPKE-JWE draft) is a relevant signal for ANY Benten comment that proposes adding new codepoints or expanding registries. Frame Benten's adopter-data-point comments as "stability + cross-stack alignment" not "expansion" to stay on the same wavelength.
- **Karen O'Donoghue** is the same chair across JOSE adoption calls (msg06919 Prabel + msg07070 Skokan) — useful name to know when Benten engages JOSE WG in any future context.

---

## §10 — Sources cited

Primary archive URLs (mail-archive.com mirror + IETF official archive where retrievable):

**Adoption call thread (msg07070 + 7 replies):**
- http://www.mail-archive.com/jose@ietf.org/msg07070.html — Karen O'Donoghue call
- http://www.mail-archive.com/jose@ietf.org/msg07077.html — Skokan reply
- http://www.mail-archive.com/jose@ietf.org/msg07078.html — Madden reply
- http://www.mail-archive.com/jose@ietf.org/msg07079.html — Campbell reply
- http://www.mail-archive.com/jose@ietf.org/msg07080.html — Parecki reply
- http://www.mail-archive.com/jose@ietf.org/msg07081.html — Reddy reply
- http://www.mail-archive.com/jose@ietf.org/msg07082.html — Orie Steele reply
- http://www.mail-archive.com/jose@ietf.org/msg07087.html — Mike Jones reply
- IETF mirror anchor: https://mailarchive.ietf.org/arch/msg/jose/XBAnatwSbLR-m0V3eMpb72omkF8/

**Background threads:**
- http://www.mail-archive.com/jose@ietf.org/msg07049.html — Skokan path-forward announcement
- http://www.mail-archive.com/jose@ietf.org/msg07050.html — Karen O'Donoghue path-forward endorsement
- http://www.mail-archive.com/jose@ietf.org/msg06962.html — Initial -00 draft announcement
- http://www.mail-archive.com/jose@ietf.org/msg06969.html — Michael P1 substantive technical reply on -00
- http://www.mail-archive.com/jose@ietf.org/msg07032.html — IANA early-review collision (different draft, context)
- http://www.mail-archive.com/jose@ietf.org/msg07012.html — Cross-WG COSE/LAKE coordination
- http://www.mail-archive.com/jose@ietf.org/msg06919.html — Prabel adoption-call precedent (Jan 2026)

**Parallel JOSE threads (§8):**
- http://www.mail-archive.com/jose@ietf.org/msg07073.html — Brian Campbell on HPKE-4-KE / HPKE-6-KE removal
- http://www.mail-archive.com/jose@ietf.org/msg07074.html — Campbell-Skokan IANA DE thread
- http://www.mail-archive.com/jose@ietf.org/msg07075.html — Skokan DE response
- http://www.mail-archive.com/jose@ietf.org/msg07076.html — Liusvaara on Section 10.1 + algorithm proliferation
- http://www.mail-archive.com/jose@ietf.org/msg07083.html — Reddy reply to Liusvaara
- http://www.mail-archive.com/jose@ietf.org/msg07084.html — Kyzivat ART-ART review
- http://www.mail-archive.com/jose@ietf.org/msg07086.html — I-D Action hpke-encrypt-18
- http://www.mail-archive.com/jose@ietf.org/msg07057.html — WGLC deprecate-none-rsa15
- http://www.mail-archive.com/jose@ietf.org/msg07033.html — JOSE WG Report IETF 125

**Datatracker:**
- https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/
- https://datatracker.ietf.org/group/jose/about/ (chair roster)
- https://datatracker.ietf.org/wg/jose/documents/ (active drafts state)
- https://datatracker.ietf.org/doc/draft-ietf-jose-pqc-kem/
- https://datatracker.ietf.org/doc/draft-ietf-jose-pq-composite-sigs/
- https://www.ietf.org/archive/id/draft-skokan-jose-hpke-pq-pqt-05.html (draft text, IANA-Considerations section)

---

**End of deep-dive.** Nothing posted. Recommendation: post the drafted comment after applying the §7.2 X-Wing-vs-HPKE-PQ-construction vocabulary fix; optional §7.3 polish.
