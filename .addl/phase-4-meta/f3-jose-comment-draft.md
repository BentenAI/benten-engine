# F3 JOSE adoption-call comment draft — `draft-skokan-jose-hpke-pq-pqt-05`

**Provenance**: drafted 2026-05-26 by orchestrator per Ben's L12 ratification (LAMPS-default + Inv-15 framework + JOSE adoption-call deadline 2026-05-29). Adoption-call deadline is in 3 days; Ben pre-ratified drafting + posting conditional on L12 settlement.

**Status**: DRAFT for Ben review + edit + post. Do NOT post without Ben sign-off. Mailing list = `jose@ietf.org` (WG mailing list).

**Thread context**: JOSE WG adoption call for `draft-skokan-jose-hpke-pq-pqt-05` (PQ + PQ/T HPKE algorithm IDs for JWE/JWS). Surfaced by IETF-PQ-WGs ecosystem scan (`.addl/phase-4-meta/comment-opportunities-ietf-pq-wgs.md`); P0 urgency. Deadline 2026-05-29 (3 days from drafting).

**Stance**: Humble adopter support. Cite Benten's concrete shipping of X-Wing-hybrid AEAD at multicodec codepoint `0x647A` + LAMPS Composite ML-DSA at v1-beta. Endorse adoption + the algorithm-naming-stability ask. Do NOT propose new technical content. Do NOT cross-link to other Benten initiatives that aren't directly on-topic.

---

## DRAFT COMMENT

> Supporting adoption of `draft-skokan-jose-hpke-pq-pqt-05` from a concrete adopter perspective. We (Benten Engine — content-addressed graph engine with decentralized identity) are about to ship at v1-beta with:
>
> - **PQ-hybrid signatures** via LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20)
> - **Hybrid AEAD** using an X-Wing-style combiner at multicodec codepoint `0x647A` (X25519 + ML-KEM-768)
>
> The JOSE-side PQ + PQ/T HPKE algorithm IDs registered by this draft give every JOSE-stack consumer (every JOSE library, WebCrypto, JWKS endpoints) the same low-friction adoption path our LAMPS-side already gets from BouncyCastle 1.80+ / OpenSSL 3.5 / OpenPGP-PQC `draft-ietf-openpgp-pqc-17` (mandates `ML-DSA-65+Ed25519`; RFC publication expected H1-2026). For content-addressed systems where signed artifacts persist long-term, the algorithm-naming stability across LAMPS / JOSE / COSE registries is exactly what enables the post-quantum-transition story without forcing every adopter to mint custom algorithm identifiers.
>
> Particularly endorsing the **PQ/T hybrid** framing — the transitional construction balances harvest-now-decrypt-later against audited-classical-security-floor in a way pure-PQ doesn't, and matches the converging direction across LAMPS Composite ML-DSA + OpenPGP-PQC + Sigstore retooling. As an adopter, the unified naming this draft establishes between JWE/JWS HPKE-PQ-PQT IDs and the sibling LAMPS / COSE algorithm IDs lowers the cross-stack integration cost meaningfully.

---

## Posting discipline (per standing posting discipline)

Before posting:
1. **Verify the draft + thread state still resolve** + deadline still 2026-05-29
2. **Verify cite-anchors** (OID `1.3.6.1.5.5.7.6.48` early-allocated 2025-10-20; multicodec codepoint `0x647A` for X-Wing; OpenPGP-PQC draft -17 H1-2026 RFC ETA)
3. **Be brief** — drafted ~190 words; resist temptation to expand
4. **Avoid grandstanding** — Benten is a 5-week-old project per L5
5. **No "we believe industry should..."** — endorse the draft + share concrete shipping evidence, don't tell WG what to do
6. **JOSE WG mailing list** — `jose@ietf.org`; subject line should follow WG convention (suggest: "Re: Call for adoption: draft-skokan-jose-hpke-pq-pqt"); reply-in-thread to the adoption-call email if discoverable in the archives
7. **No AI attribution** — author = Ben (or Benten's public maintainer identity); no Co-Authored-By trailers in posted text

Once posted, this becomes a public artifact in the JOSE WG community signalling Benten's adoption posture. FOREVER (can be edited but original date is permanent).

## Why post this (vs let deadline pass)

- Low risk: humble adopter-support comment, no architectural disagreement, no proposing new content
- Highest-leverage IETF engagement Benten can do this month — JOSE WG adoption calls are the moment WG decisions get made; concrete-adopter data points carry weight
- Demonstrates Benten is paying attention to standards process (relevant for future F3/F4a/F4 engagement credibility per L11 "IETF-vocabulary-precision" pim-N candidate)
- The adopter framing (we ship X-Wing AEAD at 0x647A + LAMPS sigs) is genuinely accurate
- Cross-stack-algorithm-naming-stability ask is genuinely Benten's interest as a multi-stack adopter

## Why NOT post this

- 3-day window means if Ben doesn't see this in time, we miss the deadline (no comment posted = "we didn't engage" signal in the WG record)
- Risk Ben judges the draft text needs revision and we run out of clock

## Recommendation

Post if Ben can review + sign-off + post-or-have-posted within 72 hours. If not, let deadline pass (low-cost; F4a + F2/F7 + other paths remain).
