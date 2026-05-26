# IETF PQ + Composite-Sig WG Comment Opportunities

**Scan date:** 2026-05-26
**Scout:** IETF-ecosystem-scout agent
**Scope:** LAMPS (spasm@), JOSE (jose@), COSE (cose@), CFRG (cfrg@), PQUIP (pqc@), OpenPGP, TLS, plus IANA registries (PKIX-Alg, JOSE-Alg, COSE-Alg)
**Discipline:** READ-ONLY reconnaissance; nothing posted; humble framing
**Benten substrate:** v1-beta ships LAMPS Composite `id-MLDSA65-Ed25519-SHA512` (OID 1.3.6.1.5.5.7.6.48, IANA early-allocated 2025-10-20); did:jwk canonical; possible Bird-of-Prey/SUF-CMA-preserving combiner pending senior cryptographer review

---

## TL;DR — High-priority window items

| Priority | WG | Item | Window | Action |
|---|---|---|---|---|
| **P0** | JOSE | Call for adoption: `draft-skokan-jose-hpke-pq-pqt-05` | **Ends 2026-05-29 (3 days)** | COMMENT-NOW: cite Benten as a JOSE/COSE consumer adopting a parallel PQ-hybrid stack via LAMPS Composite ML-DSA; quietly support adoption + flag specifically what an at-rest content-addressed engine needs (algorithm-naming stability / X25519+ML-KEM-768 + Ed25519+ML-DSA-65 default pairing parity with LAMPS) |
| **P0** | LAMPS | `draft-ietf-lamps-pq-composite-sigs-19` in RFC-Ed queue ("In Progress"); CMS sibling `-05` in RFC-Ed queue ("Blocked") | Imminent publication | COMMENT-AFTER-RFC: register Benten as an adopter once RFCs land; pre-position now via test-vector deposit (see F4a below) |
| **P1** | LAMPS | Issue #289 "Add negative test cases to the github" open since 2025-10-03 | Open | COMMENT-NOW: offer to contribute Benten's `id-MLDSA65-Ed25519-SHA512` round-trip negative cases (forged classical half / strip / swap / corrupted domain-sep prefix) — directly serves F4a "highest-leverage adopter contribution" |
| **P1** | CFRG | `draft-prabel-cfrg-suf-hybrid-sigs-01` (individual; CFRG list, not yet WG-adopted) | Open, expires Sep 2026 | COMMENT-NOW (low-stakes): note Benten is evaluating SUF-CMA-preserving combiners as a successor to the Composite ML-DSA construction; concrete adopter datapoint helps the case for WG adoption |
| **P2** | JOSE | `draft-ietf-jose-pq-composite-sigs-01` (WG-adopted Jan 2026, IANA early-review thread open) | Active | WATCH + CONDITIONAL-COMMENT: if Benten ever cares about JWS-format signatures (currently uses raw COSE-style for did:jwk-anchored signing), `ML-DSA-65-Ed25519` codepoint aligns; otherwise observation-only |
| **P2** | PQUIP | `draft-reddy-pquip-pqc-signature-migration-01` (individual; not adopted) | Open | WATCH-ONLY: migration-guidance doc; not a fit for adopter comment yet |
| **P3** | OpenPGP | `draft-ietf-openpgp-pqc-17` (Standards Track, near RFC publication, H1-2026) | Near publication | IRRELEVANT to Benten direct; informational only — confirms Ed25519+ML-DSA-65 pair as a real industry default beyond LAMPS |

---

## 1. Active adoption-calls / WG-LC moments worth weighing in on

### 1.1 JOSE — Call for adoption: `draft-skokan-jose-hpke-pq-pqt-05` (ends 2026-05-29) [P0]

- **URL:** https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/
- **Call thread:** http://www.mail-archive.com/jose@ietf.org/msg07070.html (Karen O'Donoghue, opened ~2026-05-14, ends 2026-05-29)
- **Background thread:** http://www.mail-archive.com/jose@ietf.org/msg07049.html ("PQ & PQ/T HPKE JWE path forward")
- **What it does:** Registers PQ and PQ/T hybrid algorithm identifiers for JWE built on the HPKE framework (`draft-ietf-jose-hpke-encrypt`), reusing HPKE mode machinery with PQ + PQ/T KEMs. Sibling: `draft-reddy-cose-hpke-pq-pqt` for COSE.
- **Authors:** Filip Skokan, Brian Campbell, Hannes Tschofenig, Tirumaleswar Reddy.K
- **What the call asks:** "WG members, please respond to this call for adoption with your thoughts on whether or not the JOSE working group should adopt this document. Also, please indicate your willingness to review and/or contribute to the document."
- **Relevance to Benten:** Tangential-but-real. Benten's `Subgraph` encryption substrate (#1301, Phase-4-Meta-Core) is being designed around X-Wing-style PQ-hybrid KEMs with codepoint dispatch. JOSE's HPKE-based JWE PQ codepoints will materially shape what a `did:jwk`-anchored content-addressed engine can interoperate with at the encryption layer downstream (Phase 7-8 untrusted-host / peers-hold-ciphertext). Adopter perspective: stability of algorithm-naming + ID round-tripping matters; that JOSE/COSE proceed in lockstep so a Benten message signed/encrypted on the engine side can be consumed by a JOSE-only browser surface.
- **Recommended action:** **COMMENT-NOW** — humble support + concrete adopter framing.
- **Comment-shape outline (~3-5 sentences, DO NOT POST):**

> Speaking as an early adopter who is shipping LAMPS Composite ML-DSA (`id-MLDSA65-Ed25519-SHA512`, OID 1.3.6.1.5.5.7.6.48) as the default for a content-addressed graph engine, I'd support adoption of this draft. The HPKE-PQ-PQT registration path mirrors the layering we're already using on the signature side (LAMPS-Composite at the wire level, JOSE/COSE codepoints at the application interop layer), and having JOSE/COSE register PQ + PQ/T HPKE identifiers in parallel reduces the migration overhead for adopters who need to cross protocol surfaces. One small note from the adopter perspective: keeping algorithm-naming stable across the LAMPS / JOSE / COSE silos (same component names, same suite ordering) materially reduces the implementation surface for downstream consumers — please prioritize that alignment in IANA registrations where the option exists. Happy to review subsequent drafts from the adopter angle.

### 1.2 LAMPS — `draft-ietf-lamps-cms-composite-sigs-05` Protocol Action / RFC-Ed Queue Blocked [P0 informational]

- **URL:** https://datatracker.ietf.org/doc/draft-ietf-lamps-cms-composite-sigs/
- **State (2026-05-22):** Protocol action approved + moved to RFC-Ed Queue; currently "Blocked" (publication queue entry exists; unresolved technical/editorial concerns per Last-Call review summary)
- **Companion:** `draft-ietf-lamps-pq-composite-sigs-19` (PKIX-X.509 sibling) RFC-Ed Queue "In Progress"; latest version 2026-04-21; status update 2026-05-20; OIDs early-allocated 2025-10-20
- **Relevance to Benten:** Direct. Benten v1-beta sets default to `id-MLDSA65-Ed25519-SHA512` from the PKIX draft. The CMS sibling is one layer over and matters at the CMS-encoded-content surface (not Benten's current target), but the parallel motion confirms both are landing as RFCs imminently.
- **Recommended action:** **COMMENT-AFTER-RFC-PUBLICATION** for adopter registration (see §2 row 1).
- **WATCH:** the "Blocked" state on -05 — if the unblocking decision touches the underlying combiner construction or test-vector layout, Benten's implementation should re-verify against the final RFC bytes before v1-beta tag.

### 1.3 JOSE — WG-LC: `draft-ietf-jose-deprecate-none-rsa15-04` (ended 2026-05-20) [P3]

- **URL:** https://datatracker.ietf.org/doc/draft-ietf-jose-deprecate-none-rsa15/
- **Window:** Closed 6 days ago.
- **Relevance to Benten:** None direct (Benten never used `none` or `RSA1_5`).
- **Recommended action:** **WATCH-ONLY**; confirms JOSE WG is actively pruning legacy algorithm surface — supportive of Benten's PQ-default posture.

### 1.4 COSE — WG-LC: `draft-ietf-cose-sphincs-plus-07` [P3]

- **URL:** https://datatracker.ietf.org/doc/draft-ietf-cose-sphincs-plus/
- **State:** WG-LC reference visible in JOSE archive cross-thread (May 2026); -08 published 2026-05-15
- **Relevance to Benten:** None at v1-beta (Benten ships ML-DSA-65, not SLH-DSA). NF-1 PQ⊕PQ end-state names SLH-DSA as a buildable-now-as-non-default arm; this draft's IANA codepoints (SLH-DSA-SHA2-128s, SLH-DSA-SHAKE-128s) would be the right targets if Benten ever stands up that arm.
- **Recommended action:** **WATCH-ONLY** as the SLH-DSA codepoint authority of record.

---

## 2. New comment opportunities (sorted by priority)

| # | Priority | WG / Surface | Item / URL | Open? | Concrete adopter angle | Recommended action |
|---|----------|---------|---------|---|---|---|
| 1 | **P0** | LAMPS GitHub | Issue #289 "Add negative test cases to the github" (`draft-composite-sigs`, opened 2025-10-03 by ounsworth, label `Post-WGLC`) — https://github.com/lamps-wg/draft-composite-sigs/issues/289 | OPEN | Offer Benten's `id-MLDSA65-Ed25519-SHA512` negative test vectors: (a) classical-half-only-valid (PQ-half corrupted) MUST reject, (b) PQ-half-only-valid (classical-half corrupted) MUST reject, (c) suite-swap (sigs from two valid composites concatenated) MUST reject, (d) domain-sep prefix corruption MUST reject, (e) order-swap (PQ\|\|classical vs classical\|\|PQ encoding) MUST reject. This is the **F4a** highest-leverage adopter contribution. | **COMMENT-NOW** with concrete test-vector PR offer |
| 2 | **P0** | JOSE | Adoption call `draft-skokan-jose-hpke-pq-pqt-05` (§1.1 above) | Ends 2026-05-29 | Adopter quietly-supportive + algorithm-naming-stability ask | **COMMENT-NOW** per §1.1 shape |
| 3 | **P1** | LAMPS GitHub | Issue #322 "AD Review Publication Steps" (opened 2025-12-08 by johngray-dev, label `Post-WGLC`) — https://github.com/lamps-wg/draft-composite-sigs/issues/322 | OPEN | Read-only; understand publication-track unblock criteria so Benten knows when to register as adopter | **WATCH-ONLY** until RFC ships |
| 4 | **P1** | CFRG | `draft-prabel-cfrg-suf-hybrid-sigs-01` (individual; not adopted; CFRG list; expires 2026-09-02) — https://datatracker.ietf.org/doc/draft-prabel-cfrg-suf-hybrid-sigs/ | OPEN | Benten's pending senior-cryptographer review is specifically weighing whether to switch from Composite ML-DSA's vanilla construction to a SUF-CMA-preserving combiner. A short CFRG list post noting "real adopter actively evaluating this for v1-beta default" gives the draft the kind of concrete adopter datapoint that makes WG-adoption discussions easier. | **COMMENT-NOW** at very-low-stakes (adopter datapoint, no objection, no endorsement) |
| 5 | P2 | JOSE | `draft-ietf-jose-pq-composite-sigs-01` (WG-adopted 2026-Jan; IANA early-review thread http://www.mail-archive.com/jose@ietf.org/msg07032.html flagged COSE-Alg codepoint collision: requested -51..-53 already assigned to RFC 9864; alternate -54..-56 also requested for the Ed25519/Ed448 pairings) — https://datatracker.ietf.org/doc/draft-ietf-jose-pq-composite-sigs/ | OPEN (IANA action) | If Benten ever exposes a JWS-format signature surface (currently `did:jwk` is anchored, not signed), `ML-DSA-65-Ed25519` is the algorithm-name Benten would consume. The IANA codepoint collision is being procedurally resolved; comment unnecessary unless Benten plans to ship a JOSE-side library. | **CONDITIONAL-COMMENT after-v1-beta-tag** only if a JWS surface gets added; otherwise WATCH |
| 6 | P2 | PQUIP | `draft-reddy-pquip-pqc-signature-migration-01` (individual; published 2025-10-14; Informational) — https://datatracker.ietf.org/doc/draft-reddy-pquip-pqc-signature-migration/ | OPEN, individual | Composite vs Dual vs PQC-only migration guidance. Benten is on Composite path. If this gets WG-adopted, an adopter-experience datapoint would be welcome. | **WATCH** for adoption-call; comment then |
| 7 | P2 | LAMPS | `draft-ietf-lamps-cms-composite-kem-01` (in WG-LC, expires 2026-05-06) — https://datatracker.ietf.org/doc/draft-ietf-lamps-cms-composite-kem/ | WG-LC | Sibling to CMS-Composite-Sigs but on the KEM side. Benten's #1301 encryption substrate may consume this once the encryption-as-confidentiality work lands in Phase-4-Meta-Core. | **WATCH-ONLY** until #1301 design fully ratified |
| 8 | P3 | TLS | `draft-reddy-tls-composite-mldsa-10` (individual; 2026-05-14; explicitly cites LAMPS Composite as authority) — https://datatracker.ietf.org/doc/draft-reddy-tls-composite-mldsa/ | OPEN, individual | Same composite-suites mapping into TLS 1.3. Benten doesn't operate a TLS surface directly (iroh handles transport), so direct relevance is low. Informational confirmation that LAMPS Composite is the multi-protocol authority. | **IRRELEVANT** unless Benten adds a non-iroh transport |
| 9 | P3 | OpenPGP | `draft-ietf-openpgp-pqc-17` (Standards Track, near publication, H1-2026) — https://datatracker.ietf.org/doc/draft-ietf-openpgp-pqc/ | Near RFC | Mandates the same Ed25519+ML-DSA-65 composite as Benten. Sequoia PGP ship-on-publication commitment. Industry-default validation. | **IRRELEVANT** to direct comment; useful citation when Benten documents the v1-beta default choice rationale |
| 10 | P3 | CFRG | `draft-connolly-cfrg-hybrid-sig-considerations-00` (Connolly + Schmieg / SandboxAQ + Google) — **EXPIRED 2025-05-18** — https://datatracker.ietf.org/doc/draft-connolly-cfrg-hybrid-sig-considerations/ | EXPIRED | Earlier "when to use hybrid sigs / when not to" considerations doc. Expired with no -01. Referenced in cfrg/SUF discussions as the prior-art frame. | **IRRELEVANT** for comment; useful for Benten's own internal "why hybrid-by-default" rationale |
| 11 | P3 | PQUIP | `draft-ietf-pquip-hybrid-signature-spectrums-07` (in RFC-Ed queue "In Progress"; expired 2025-12-22 as draft but submitted to IESG; informational) — https://datatracker.ietf.org/doc/draft-ietf-pquip-hybrid-signature-spectrums/ | RFC-Ed queue | Taxonomy doc on hybrid-sig design dimensions (proof composability, non-separability, etc). Benten's internal docs can cite this once published. | **WATCH** for RFC publication; cite from `docs/SECURITY-POSTURE.md` then |

---

## 3. WATCH-ONLY threads worth monitoring

- **LAMPS spasm@ archive** — https://mailarchive.ietf.org/arch/browse/spasm/ — active May 18-22 review traffic on `draft-ietf-lamps-cms-composite-sigs-05` AD review (Boucadair / Jethanandani / Inacio / Vyncke replies). Watch for any AD-DISCUSS resolutions that change the combiner construction or test-vector layout — would require Benten implementation re-verify before v1-beta tag.
- **CFRG cfrg@ archive** — https://mailarchive.ietf.org/arch/browse/cfrg/ — watch for any thread responding to Prabel's `draft-prabel-cfrg-suf-hybrid-sigs-01` (Mar 1 2026) and the older Connolly hybrid-considerations draft. If a WG-adoption-call appears, Benten should weigh in given the senior-cryptographer-review fork is open.
- **JOSE jose@ archive** — https://mailarchive.ietf.org/arch/browse/jose/ — beyond the §1.1 adoption-call, watch the IANA codepoint-collision resolution for `draft-ietf-jose-pq-composite-sigs-01` (-51..-53 collision with RFC 9864). If the authors switch to a placeholder range or request RFC-7120 early allocation, the COSE algorithm value for `ML-DSA-65-Ed25519` (currently requested as -55) may shift.
- **COSE cose@ archive** — https://mailarchive.ietf.org/arch/browse/cose/ — cross-WG references to `draft-ietf-jose-pq-composite-sigs` and `draft-ietf-cose-sphincs-plus-07` WGLC. Watch for COSE-side adoption of `draft-reddy-cose-hpke-pq-pqt` (sibling to §1.1) which has not yet hit a call for adoption per the available archive view.
- **Mike Jones thread** — "JOSE libraries supporting ML-DSA" (2026-04-22, jose@) — adoption-tracking thread; useful for Benten to add to once a v1-beta tag exists and a Rust impl can be cited.
- **PQUIP pqc@ archive** — watch for adoption-call activity on `draft-reddy-pquip-pqc-signature-migration` and any new individual drafts touching composite-vs-dual decision-trees.

---

## 4. IANA registry status snapshots

### 4.1 PKIX Algorithm Identifiers (SMI Security for PKIX Algorithms, 1.3.6.1.5.5.7.6)

- **`id-MLDSA65-Ed25519-SHA512`** at OID `1.3.6.1.5.5.7.6.48` — **early-allocated 2025-10-20** per `draft-ietf-lamps-pq-composite-sigs-19` (RFC-Ed Queue: In Progress, 2026-04-21). 18 composite algorithms early-allocated as a batch.
- Source: `draft-ietf-lamps-pq-composite-sigs-19` §IANA-Considerations + datatracker status note.
- Direct IANA registry URL: https://www.iana.org/assignments/smi-numbers/smi-numbers.xhtml#smi-numbers-1.3.6.1.5.5.7.6 (registry display did not surface the entry in our scan — confirmed via draft + datatracker, not direct registry HTML).
- **Status:** STABLE for Benten v1-beta dependence. Early-allocated codepoints are explicitly designed to be referenceable pre-RFC.

### 4.2 JOSE Algorithm Names + COSE Algorithm Values

Per `draft-ietf-jose-pq-composite-sigs-01` (2026-02-27):

| JOSE alg name | COSE alg value (requested) |
|---|---|
| ML-DSA-44-ES256 | -51 (⚠️ collision with RFC 9864) |
| ML-DSA-65-ES256 | -52 (⚠️ collision) |
| ML-DSA-87-ES384 | -53 (⚠️ collision) |
| ML-DSA-44-Ed25519 | -54 |
| **ML-DSA-65-Ed25519** | **-55** ← matches Benten's LAMPS choice |
| ML-DSA-87-Ed448 | -56 |

- IANA review state: "Expert Reviews OK" on the LAMPS side; on the JOSE side, the -51..-53 collision is being procedurally resolved (Amanda Baber thread: http://www.mail-archive.com/jose@ietf.org/msg07032.html). Options on the table: RFC-7120 early allocation OR placeholder TBD1..TBD3.
- **Status for Benten:** the codepoint Benten would care about (`ML-DSA-65-Ed25519` at COSE alg value -55) is NOT in the collision zone; should remain stable. JOSE-alg-name `ML-DSA-65-Ed25519` is the published-RFC anchor for `id-MLDSA65-Ed25519-SHA512` cross-protocol.

### 4.3 IANA hybrid-sig codepoint allocations (cross-WG)

- No central "hybrid-sig codepoint" registry; per-WG (PKIX-Alg, JOSE-Alg, COSE-Alg) registrations as above. The naming-stability ask in §1.1's comment-shape is the cross-WG alignment lever for adopters.

---

## 5. Cluster-wide pattern observations

1. **LAMPS is the multi-protocol authority of record.** Every other WG's PQ-composite work (JOSE PQ-composite-sigs, COSE post-quantum-signatures, TLS reddy-tls-composite-mldsa) cites LAMPS Composite ML-DSA as the construction source. Benten's v1-beta choice to anchor on the LAMPS draft (vs e.g. an ad-hoc combiner) is well-placed: it sits at the authoritative center of the composite-sig ecosystem.

2. **The CMS sibling moving to Proposed Standard 2026-05-22 + the PKIX -19 "In Progress" in RFC-Ed queue means the LAMPS Composite ML-DSA RFCs are weeks-to-months from publication.** Benten's "v1-beta default = LAMPS Composite" framing will shift from "shipping an early-allocated codepoint from an Internet-Draft" to "shipping a freshly-published RFC" inside Benten's likely launch window. The post-publication adopter-registration timing (§2 row 1; F4 LAMPS adopter-report fit) is the natural moment to surface Benten publicly.

3. **The SUF-CMA combiner question is live but not urgent.** Connolly's hybrid-sig-considerations draft expired May 2025 without renewal; Prabel's draft-prabel-cfrg-suf-hybrid-sigs landed -00 March 2026, -01 March 2026, individual submission, no WG-adoption call yet. The academic prior-art ("Bird of Prey: Practical Signature Combiners Preserving Strong Unforgeability", Springer/IACR 2025 — https://eprint.iacr.org/2025/1844) shows the construction is well-motivated. Benten's senior-cryptographer-review fork choosing between "ship LAMPS Composite as-shipped" vs "ship a Bird-of-Prey-class SUF-CMA-preserving combiner" maps cleanly onto this IETF state: if Benten goes with the LAMPS-as-shipped path, the SUF-CMA upgrade is a clean post-v1 swap inside the codepoint-dispatch matrix; if Benten goes early with Bird-of-Prey, the comment-shape in §2 row 4 becomes higher-leverage.

4. **JOSE HPKE PQ/PQT JWE is the next adoption-cycle wave (P0 in §1.1).** The substrate Benten's #1301 encryption-as-confidentiality plugs into at the application/protocol-boundary layer.

5. **Test-vector deposits are the highest-leverage adopter contribution** (Benten's own F4a finding). LAMPS issue #289 "Add negative test cases to the github" is the most surgical, narrowly-scoped, technically-substantive, low-political-risk vehicle for Benten to surface as a real implementer pre-v1-beta. This is the recommendation worth queueing for Ben as the single most-actionable item from this scan. If the senior-crypto review concludes "ship LAMPS as-shipped," Benten's negative test vectors directly help future implementers avoid the same pitfalls Benten will have already worked through.

6. **OpenPGP PQC mandate (`draft-ietf-openpgp-pqc-17`) is the strongest external industry-default validation** of the exact Ed25519+ML-DSA-65 composite Benten ships. Sequoia PGP committed to ship-on-publication. When Benten documents the v1-beta default-choice rationale (NF-2 / C-GM-AUDIT writeup), this is the cleanest "we are not alone" citation.

7. **PQUIP hybrid-signature-spectrums is the right anchor for Benten's `docs/SECURITY-POSTURE.md` taxonomy language** (proof composability / non-separability / backwards-compat / hybrid generality / simultaneous verification). Once it publishes as RFC (in queue "In Progress"), Benten's posture doc can lift the WG-published vocabulary instead of inventing its own.

---

## Sources cited

- LAMPS PKIX composite-sigs draft: https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/
- LAMPS CMS composite-sigs draft: https://datatracker.ietf.org/doc/draft-ietf-lamps-cms-composite-sigs/
- LAMPS WG document list: https://datatracker.ietf.org/wg/lamps/documents/
- LAMPS GitHub repo: https://github.com/lamps-wg/draft-composite-sigs
- LAMPS GitHub issue #289 (negative test cases): https://github.com/lamps-wg/draft-composite-sigs/issues/289
- LAMPS GitHub issue #322 (AD Review Publication Steps): https://github.com/lamps-wg/draft-composite-sigs/issues/322
- spasm@ list archive: https://mailarchive.ietf.org/arch/browse/spasm/
- JOSE pq-composite-sigs draft: https://datatracker.ietf.org/doc/draft-ietf-jose-pq-composite-sigs/
- JOSE WG document list: https://datatracker.ietf.org/wg/jose/documents/
- JOSE skokan-hpke-pq-pqt draft: https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/
- JOSE adoption call (Skokan -05, ends 2026-05-29): http://www.mail-archive.com/jose@ietf.org/msg07070.html
- JOSE PQ/PQT HPKE path-forward thread: http://www.mail-archive.com/jose@ietf.org/msg07049.html
- JOSE IANA early-review collision thread: http://www.mail-archive.com/jose@ietf.org/msg07032.html
- JOSE adoption call Prabel composite -05 (closed 2026-01-19): http://www.mail-archive.com/jose@ietf.org/msg06919.html
- JOSE archive index: https://mailarchive.ietf.org/arch/browse/jose/
- COSE sphincs-plus draft: https://datatracker.ietf.org/doc/draft-ietf-cose-sphincs-plus/
- CFRG SUF-hybrid-sigs draft: https://datatracker.ietf.org/doc/draft-prabel-cfrg-suf-hybrid-sigs/
- CFRG hybrid-sig-considerations (expired): https://datatracker.ietf.org/doc/draft-connolly-cfrg-hybrid-sig-considerations/
- CFRG archive index: https://mailarchive.ietf.org/arch/browse/cfrg/
- PQUIP hybrid-signature-spectrums (RFC-Ed queue): https://datatracker.ietf.org/doc/draft-ietf-pquip-hybrid-signature-spectrums/
- PQUIP pqc-signature-migration: https://datatracker.ietf.org/doc/draft-reddy-pquip-pqc-signature-migration/
- TLS composite ML-DSA (individual): https://datatracker.ietf.org/doc/draft-reddy-tls-composite-mldsa/
- OpenPGP PQC draft: https://datatracker.ietf.org/doc/draft-ietf-openpgp-pqc/
- Bird of Prey paper (academic): https://eprint.iacr.org/2025/1844
- PKIX Algorithm IANA registry: https://www.iana.org/assignments/smi-numbers/smi-numbers.xhtml#smi-numbers-1.3.6.1.5.5.7.6

---

**End of scan.** Nothing posted. Hand back to orchestrator for triage + Ben surfacing.
