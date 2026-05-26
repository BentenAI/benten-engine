# Standards-Maturity-and-Ecosystem-Skeptic Review: Encrypt-to-Recipient at v1-beta

**Reviewer:** Standards-and-Ecosystem Skeptic (engaged 2026-05-26)
**Decision-surface:** Whether to add encrypt-to-recipient at v1-beta, and via which of 6 enumerated options
**Authority of this document:** ADVISORY. Final disposition rests with Ben.
**Lens:** Standards-process maturity, WG-adoption-risk, ecosystem-alignment, adoption-timing-risk asymmetry. The construction-soundness lens (senior cryptographer) and the architecture-fit lens (P2P architect) cover the other halves; I focus where my lens specifically adds signal.

---

## 0. Reading-order note

This review is long because the decision affects v1-beta wire-format freeze. The TL;DR sits in §1. The most important sections for a binary decision are §1, §2, §6, §9, §11. The standards-process state-of-the-art is in §3-5; the per-option assessment is in §2; the elegant Option F shape is in §11.

---

## 1. Executive recommendation

**CONDITIONAL GO** on adding encrypt-to-recipient at v1-beta, but **CONDITIONAL NO-GO on Options A, C, D, and E as primary v1-beta default**. **Recommended path: Option B (HPKE-RFC-9180 envelope) with a deliberate variant — use the RG-blessed concrete combiner, NOT a Benten-private X-Wing-as-KEM combination.**

The specific construction I recommend:

**Option B' (refined): HPKE-RFC-9180 single-shot mode (`SealBase`/`OpenBase`) with KEM = `MLKEM768-X25519` per `draft-irtf-cfrg-concrete-hybrid-kems-03` (an IRTF-CFRG-ADOPTED Research Group document; same construction X-Wing names but with the RG-Schelling-point KEM-id), AEAD = ChaCha20-Poly1305, KDF = HKDF-SHA256. Codepoint = the same value `draft-ietf-hpke-pq` will register for HPKE KEM `MLKEM768-X25519` once it advances to IANA early-allocation; if not yet available, mint a Benten-internal codepoint and migrate via the additive-codepoints crypto-agility framework when the IANA allocation lands.**

This puts Benten on the exact Schelling point three IETF WGs (HPKE / JOSE / COSE / MLS) are converging toward in mid-2026, with the KEM combiner that is mathematically identical to X-Wing (Benten's existing storage substrate combiner) but named under the WG-blessed identifier — so the same combiner serves both layers, no fragmentation between storage substrate and encrypt-to-recipient.

Three load-bearing reasons:

1. **The HPKE-PQ ecosystem alignment window is open RIGHT NOW (May 2026)**: HPKE-PQ base draft (`draft-ietf-hpke-pq-04`) is WG-adopted with Standards-Track intent; HPKE-for-JOSE base (`draft-ietf-jose-hpke-encrypt-18`) is in IESG Last Call ending 2026-05-27 (literally tomorrow at write-time); JOSE PQ extension (`draft-skokan-jose-hpke-pq-pqt-05`) is in WG-adoption-call ending 2026-05-29; MLS PQ ciphersuites (`draft-ietf-mls-pq-ciphersuites-04`) is WG-adopted referencing the SAME HPKE-PQ KEM combiners; LAMPS Composite ML-KEM is at "Publication Requested" status (IESG queue). Benten can ship on the alignment Schelling point without locking to ANY pre-WG-adopted construction — by using the RG-adopted `draft-irtf-cfrg-concrete-hybrid-kems` KEM combiner inside HPKE-RFC-9180 base mode.

2. **Option B' avoids the L12-style pre-WG-adoption-risk the cryptographer review CONDITIONAL-NO-GO'd for Bird-of-Prey signatures**: every primitive in the recommended stack is either a published RFC (HPKE 9180, ChaCha20-Poly1305 RFC 8439, HKDF RFC 5869, ML-KEM-768 FIPS 203) or in an IRTF-CFRG-adopted Research Group document (`draft-irtf-cfrg-concrete-hybrid-kems`). The ONLY individual draft involved (`draft-connolly-cfrg-xwing-kem-10`) is referenced as informative — the equivalent construction is in the RG-adopted concrete-hybrid-kems draft as `MLKEM768-X25519`.

3. **The "iroh's QUIC is end-to-end encrypted, so we don't need encrypt-to-recipient" argument FAILS for Benten's actual untrusted-host threat model**: iroh's E2EE is between two CONNECTED ENDPOINTS over a QUIC channel — it provides ZERO protection when ciphertext rests on a third-party peer (Atrium store-and-forward / Garden-Grove untrusted-host / Drop bundles through intermediate peers). Benten needs application-layer encrypt-to-recipient that survives data-at-rest on untrusted infrastructure. iroh's design does not provide this; expecting transport-layer encryption to substitute is a category error. Cite-anchored in §5.7.

The remainder of this document substantiates this recommendation and provides the per-option standards-state ratings (§2), the standards-timeline analysis (§3), the ecosystem-adoption-precedent table (§4), the timing-risk-asymmetry analysis (§5), the comparison to the cryptographer-review-of-Bird-of-Prey precedent (§6), the per-option GO/NO-GO (§7), risks + mitigations (§8), honest disagreement with framing (§9), the evidence base (§10), and the Option F shape (§11).

---

## 2. Per-option standards-maturity verdict (quick table)

| Option | Construction | Standards stage | My verdict | Reason |
|---|---|---|---|---|
| A | Status quo / defer | N/A | **NO-GO** | Misses ecosystem alignment window; not "the conservative choice" — the conservative choice in mid-2026 IS to ship the WG-converging stack |
| B | HPKE-RFC-9180 + X-Wing KEM (the Benten-private combination as posed) | Mixed: HPKE = RFC, X-Wing = individual draft + RG-adopted-equivalent | **CONDITIONAL GO** with refinement to Option B' | The HPKE envelope is the right shape; the "X-Wing-as-KEM-combiner-inside-HPKE-base" framing in the brief conflates X-Wing (individual draft) with its mathematically-identical RG-adopted equivalent (`MLKEM768-X25519` in concrete-hybrid-kems) — use the RG-adopted name |
| C | Skokan-draft codepoints | Individual draft + adoption call in flight | **NO-GO at v1-beta tag** | Adopting codepoints from a draft that is IN ITS adoption call this week (ends 2026-05-29) for v1-beta tag is the literal definition of pre-WG-adoption-risk; revisit at -01 post-adoption |
| D | Benten-tailored envelope (no HPKE) | Benten-private + X-Wing KEM | **NO-GO** | Violates the ecosystem-good-citizen framing per L4 critic precedent; multicodec maintainers explicitly recommended container-form not Benten-specific envelopes; 5-week-old project minting its own envelope is the META-message-of-small-team-fragmenting-ecosystem pattern |
| E | MLS-style group key derivation (CGKA) | WG-adopted but draft-stage for PQ | **NO-GO at v1-beta** | MLS-PQ is `draft-ietf-mls-pq-ciphersuites-04` (WG-adopted, "Waiting for WG Chair Go-Ahead" with "Revised I-D Needed"); the amortized PQ combiner (`draft-ietf-mls-combiner-02`) is even less mature; full MLS CGKA is a Phase-4-Meta-Composing or Phase-5 conversation, not a v1-beta tag commitment |
| F | Option B' refined — HPKE-RFC-9180 + RG-adopted KEM combiner | All primitives are RFC or RG-adopted | **GO** | Maximum ecosystem alignment + minimum pre-WG-adoption risk + same combiner X-Wing storage substrate uses (consistency across layers) + future-additive path to JOSE/COSE/MLS via Skokan-once-adopted |

---

## 3. Current state of HPKE-PQ standards process (load-bearing for the recommendation)

The recommendation rests on a precise reading of the standards-process state in mid-May 2026. I verified each cite against datatracker primary sources; quoted boilerplate verbatim.

### 3.1 HPKE-PQ base draft

**`draft-ietf-hpke-pq-04`** ([datatracker](https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/)):

- **Revision:** -04, published 2026-03-02
- **WG-adopted:** Yes (HPKE WG, draft-ietf prefix)
- **Track:** "Intended status: Standards Track"
- **Authors:** Richard Barnes (Cisco), Deirdre Connolly (Selkie Cryptography)
- **Boilerplate (verbatim):** "This Internet-Draft is submitted in full conformance with the provisions of BCP 78 and BCP 79"
- **KEM combiners:** `MLKEM768-P256`, `MLKEM768-X25519`, `MLKEM1024-P384`, plus pure-PQ `MLKEM768` and `MLKEM1024`. Section 7 (Security Considerations): "The IND-CCA security of the hybrid constructions used in this document is established in [CONCRETE]" — relies entirely on `draft-irtf-cfrg-concrete-hybrid-kems`.

This is a published HPKE WG document on the Standards Track. Not RFC yet, but actively progressing. The construction-soundness is established by reference to the IRTF CFRG concrete-hybrid-kems work.

### 3.2 HPKE-for-JOSE base RFC (classical only)

**`draft-ietf-jose-hpke-encrypt-18`** ([datatracker](https://datatracker.ietf.org/doc/draft-ietf-jose-hpke-encrypt/)):

- **Revision:** -18, published 2026-05-25
- **Status:** "In Last Call (ends 2026-05-27)" with ARTART review completed
- **Track:** Proposed Standard
- **Algorithms:** Classical-only DHKEM (P-256, P-384, P-521, X25519, X448). PQ extension is the separate Skokan draft.

**This document will become an RFC in 2026Q3-Q4 with very high probability.** The IESG Last Call is the final-call before publication-track. SECDIR and GENART reviews are the only remaining gates.

### 3.3 Skokan PQ extension to JOSE-HPKE

**`draft-skokan-jose-hpke-pq-pqt-05`** ([datatracker](https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/)):

- **Revision:** -05, published 2026-05-13
- **Adoption status:** "Call For Adoption By WG Issued" — adoption call ends **2026-05-29 (3 days from write-time)**
- **Boilerplate (verbatim):** "This Internet-Draft is **not endorsed by the IETF** and has **no formal standing** in the IETF standards process."
- **Authors:** Filip Skokan (Okta), Brian Campbell (Ping Identity), Hannes Tschofenig (UniBw M.), Tirumaleswar Reddy.K (Nokia)
- **KEM combiners:** Uses `MLKEM768-P256`, `MLKEM768-X25519`, `MLKEM1024-P384` from `draft-ietf-hpke-pq` (NOT X-Wing-as-named)
- **Mailing list support:** 4 explicit support votes ([archive](http://www.mail-archive.com/jose@ietf.org/msg07080.html)): Filip Skokan (author), Neil Madden, Brian Campbell, Aaron Parecki. Zero opposition documented. Path-forward agreed at IETF 125 ([archive](http://www.mail-archive.com/jose@ietf.org/msg07049.html)).

**This document will very likely be adopted within days of write-time** — but **adoption is not publication**. The adoption call ending 2026-05-29 means the document becomes `draft-ietf-jose-hpke-pq-pqt-00` shortly after. From WG-adoption to IANA codepoint allocation is typically 6-18 months at the JOSE WG cadence; RFC publication 12-24 months.

**Crucial subtlety**: the Skokan draft DOES NOT use X-Wing-as-named. It uses the HPKE-PQ KEM identifiers (`MLKEM768-X25519` etc.). The construction is mathematically identical to X-Wing for the `MLKEM768-X25519` line, but the identifier is the HPKE-PQ identifier, not the X-Wing identifier. This matters because:
- The HPKE-PQ-named identifier is what JOSE/COSE/MLS will all reference
- The X-Wing-as-named identifier (multicodec `0x647A` per Benten current code) is what Benten currently mints in its storage substrate
- These are the SAME construction with DIFFERENT names

Recommended posture: use HPKE-PQ's `MLKEM768-X25519` naming convention everywhere it appears at the wire (encrypt-to-recipient), keep the X-Wing-multicodec-0x647A naming at the storage-substrate layer (since that pre-existed and was already shipped per Benten's current code), and document the equivalence in SECURITY-POSTURE.md.

### 3.4 IRTF CFRG concrete hybrid KEMs

**`draft-irtf-cfrg-concrete-hybrid-kems-03`** ([datatracker](https://datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/)):

- **Revision:** -03, published 2026-03-02
- **WG-adopted:** Yes — `draft-irtf` prefix = IRTF Research Group document
- **CFRG adoption date:** Original adoption call sent 2024-01-31 ([archive](https://mailarchive.ietf.org/arch/msg/cfrg/PSvLFyBWDdrRaOmaXBStRpGoH3c/))
- **Track:** "Intended status: Informational" via IRTF stream
- **Boilerplate (verbatim):** "This I-D is **not endorsed by the IETF** and has **no formal standing** in the IETF standards process."
- **KEM combiners:** Three concrete instantiations:
  - `MLKEM768-P256` (ML-KEM-768 + P-256, SHAKE256 PRG, SHA3-256 KDF)
  - `MLKEM768-X25519` (ML-KEM-768 + Curve25519, SHAKE256 PRG, SHA3-256 KDF) — **explicitly: "identical to the X-Wing construction"**
  - `MLKEM1024-P384`
- **Authors:** Deirdre Connolly (SandboxAQ), Richard Barnes (Cisco), Paul Grubbs (University of Michigan)

This is the **RG-Schelling-point** construction. The IRTF endorsement boilerplate disclaimer is standard for IRTF drafts and does NOT mean "no formal standing" in the way it means it for individual submissions — IRTF documents follow the IRTF Research Group process and are published as Informational RFCs. The boilerplate disclaimer is about the IETF-Standards-Track distinction, not about ecosystem-blessing.

### 3.5 X-Wing (individual draft, RG-equivalent)

**`draft-connolly-cfrg-xwing-kem-10`** ([datatracker](https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/)):

- **Revision:** -10, published 2026-03-02
- **Adoption status:** **Individual submission** — explicitly "Active Internet-Draft (individual)" with NO formal RG backing
- **Boilerplate (verbatim):** "This I-D is **not endorsed by the IETF** and has **no formal standing** in the IETF standards process."
- **CFRG adoption call:** No adoption call observed in the CFRG mailing list for the X-Wing-as-named draft. The concrete-hybrid-kems draft (which includes the X-Wing-equivalent construction) IS the RG-adopted vehicle.
- **Construction (one sentence):** X-Wing combines X25519 and ML-KEM-768 using a SHA3-256 combiner. Equivalent to `MLKEM768-X25519` in concrete-hybrid-kems.

**The key insight:** X-Wing-as-named is INDIVIDUAL but the construction is RG-blessed under a different name. Adopting X-Wing-named at v1-beta is exactly the pattern the bird-of-prey-cryptographer-review CONDITIONAL-NO-GO'd. But adopting `MLKEM768-X25519` from concrete-hybrid-kems is using the RG-blessed name. These are MATHEMATICALLY IDENTICAL constructions; the naming choice carries the standards-maturity weight.

### 3.6 IRTF CFRG generic hybrid KEM framework

**`draft-irtf-cfrg-hybrid-kems-11`** ([datatracker](https://datatracker.ietf.org/doc/draft-irtf-cfrg-hybrid-kems/)):

- **Revision:** -11, published 2026-05-07
- **RG-adopted:** Yes (`draft-irtf` prefix)
- **Track:** Informational via IRTF
- **Defines:** Four generic frameworks (UG / UK / CG / CK) — the CG (C2PRI Combiner with Nominal Group) framework is what concrete-hybrid-kems instantiates

### 3.7 MLS-PQ status

**`draft-ietf-mls-pq-ciphersuites-04`** ([datatracker](https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/)):

- **Revision:** -04, published 2026-03-18
- **WG-adopted:** Yes (MLS WG)
- **Track:** Proposed Standard
- **Status:** "Waiting for WG Chair Go-Ahead" with "Revised I-D Needed - Issue raised by WG"
- **KEM source:** "KEMs defined in [I-D.ietf-hpke-pq]" — uses the same MLKEM768-X25519 etc. constructions

**`draft-ietf-mls-combiner-02`** ([datatracker](https://datatracker.ietf.org/doc/draft-ietf-mls-combiner/)):

- **Revision:** -02, dated 2025-10-22
- **WG-adopted:** Yes (MLS WG)
- **Purpose:** Amortized PQ MLS combiner — runs traditional + PQ MLS sessions in parallel, exports secret from PQ to traditional

MLS-PQ is real but not v1-beta-ready: the ciphersuites draft has WG-raised issues; the combiner draft is at -02. MLS-PQ as the primary encrypt-to-recipient mechanism at v1-beta tag is premature.

### 3.8 LAMPS Composite ML-KEM (CMS-shaped)

**`draft-ietf-lamps-pq-composite-kem-14`** ([datatracker](https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-kem/)):

- **Revision:** -14, published 2026-03-27
- **WG-adopted:** Yes (LAMPS WG)
- **Track:** Proposed Standard
- **Status:** **"Submitted to IESG for Publication" with "Publication Requested"** — this is the IESG queue. RFC publication is imminent (months, not years).
- **IANA early-allocated OIDs:** `1.3.6.1.5.5.7.6.55` through `1.3.6.1.5.5.7.6.66` for twelve composite algorithms
- **KEM combiner:** `SHA3-256(mlkemSS || tradSS || tradCT || tradPK || Label)` — interchangeable with X-Wing's combiner under random-oracle modeling per CFRG analysis

LAMPS Composite ML-KEM is the SECOND ecosystem Schelling point (the first being HPKE-PQ). OpenPGP-PQC mandates Composite ML-KEM for PGP. BouncyCastle / OpenSSL / AWS KMS adopting it. **For Benten, this is the right shape IF the target is CMS-style envelopes (X.509 certificates, S/MIME, OpenPGP).** It is NOT the right shape if the target is JOSE/COSE-shape envelopes (which is what JWE / Web-Crypto-stack consumability points toward). The two ecosystems are converging on different KEM-COMBINER NAMES even though the constructions are mathematically equivalent.

For Benten's encrypt-to-recipient at v1-beta with web-stack consumability in mind, HPKE-PQ-named is the right naming. For PKIX/PGP-stack interop, Composite-ML-KEM-named is the right naming. The crypto-agility framework (CLAUDE.md #5) supports BOTH future-additively.

### 3.9 OpenPGP-PQC

**`draft-ietf-openpgp-pqc-17`** ([datatracker](https://datatracker.ietf.org/doc/draft-ietf-openpgp-pqc/)):

- WG-adopted, Proposed Standard, expires 2026-07-17
- Mandates ML-KEM-768+X25519 and ML-DSA-65+Ed25519 conformance per Section 1
- Composite construction using PQ + traditional

OpenPGP-PQC ships the SAME combiner Benten already ships in storage substrate. This is positive ecosystem signal — the construction is the WG-blessed Schelling point. It's just NAMED differently in JOSE (HPKE-PQ) vs. PGP (Composite-ML-KEM) vs. Benten (X-Wing-multicodec).

---

## 4. Adopter precedents — who's shipping what

Standards documents matter, but production deployments matter more. Here's who's actually shipping what in mid-2026, cite-anchored.

### 4.1 Cloudflare

- **TLS 1.3 hybrid KEM `X25519MLKEM768` (group `0x11EC`):** deployed at production scale; ~43% of Cloudflare's human-generated connections protected by mid-September 2025 per [Cloudflare State of PQ Internet 2025](https://blog.cloudflare.com/pq-2025/). Same construction as X-Wing / `MLKEM768-X25519`, different naming convention (TLS group id).
- **IPsec PQ:** standardized 2026Q1 per [InfoQ 2026-03](https://www.infoq.com/news/2026/03/cloudflare-post-quantum-ipsec/).
- **WARP client:** PQC support per [Cloudflare blog](https://blog.cloudflare.com/post-quantum-warp/).

Cloudflare is shipping the construction in TLS naming. Not in JOSE naming. Not in X-Wing naming. The construction is the same; the wire-format-identifier ecosystem is plural.

### 4.2 Apple

- **iMessage PQ3:** [Apple Security 2024-02](https://security.apple.com/blog/imessage-pq3/). Uses ML-KEM in the ratchet (Signal-double-ratchet-PQ extended). Encrypt-to-recipient is the entire point of iMessage; PQ3 is THE largest-scale encrypt-to-recipient PQ deployment as of mid-2026.
- **Formal analysis:** [Linker et al. USENIX 2025](https://www.usenix.org/conference/usenixsecurity25/presentation/linker), TAMARIN-proven.
- Crucially: PQ3 does NOT use HPKE-PQ envelope. It's Signal-protocol-derived. This is a precedent for "P2P-messaging-class encrypt-to-recipient via Signal-style protocols" not "envelope-style encrypt-to-recipient via HPKE."

### 4.3 Signal

- **PQXDH:** ML-KEM-768 + X25519 in the X3DH replacement ([Signal spec](https://signal.org/docs/specifications/pqxdh/)). Production-deployed.
- **Triple Ratchet (2025):** combines classical Double Ratchet with Sparse Post-Quantum Ratchet (SPQR).
- Signal is on the PQ envelope-style identifier `ML-KEM-768 + X25519` — same construction as Benten's X-Wing / `MLKEM768-X25519`.

### 4.4 Chrome / Firefox browser

- Both ship ML-KEM-768+X25519 TLS hybrid by default ([F5 Labs 2026](https://www.f5.com/labs/articles/threat-intelligence/the-state-of-pqc-on-the-web)).
- 8.6% of top-1M websites support hybrid PQC key exchange (mid-2026).
- **No native Web Crypto API HPKE support yet** — third-party libs like `hpke-js` provide it ([npm hpke](https://www.npmjs.com/package/hpke), [GitHub hpke-js](https://github.com/dajiaji/hpke-js)).

For Benten's wasm32-unknown-unknown thin compute surface (deployment shape b per CLAUDE.md #17), Web Crypto API does NOT natively offer HPKE — so Benten will ship its own HPKE implementation in wasm regardless. This is a minor cost (HPKE base implementation is well-understood, ~500 LOC; many reference implementations exist).

### 4.5 Nostr NIP-44

- ChaCha20 + HMAC-SHA256 with custom padding; not HPKE-shaped. Currently no PQ extension to NIP-44.
- Cure53-audited 2023.
- Precedent for "P2P-decentralized-stack rolls its own encrypt-to-recipient envelope" — Nostr explicitly did NOT use HPKE.

### 4.6 OpenPGP-PQC

- Mandates Composite ML-KEM-768+X25519 + Composite ML-DSA-65+Ed25519.
- GnuPG implementing ML-KEM-composite keys per [OpenPGP Summit 2026 minutes](https://www.openpgp.org/community/email-summit/2026/minutes/).

### 4.7 iroh (DIRECTLY ADJACENT to Benten's stack)

- **Transport encryption only.** Every connection is end-to-end encrypted over QUIC. Ed25519 endpoint identities ([iroh security-privacy docs](https://docs.iroh.computer/deployment/security-privacy)).
- **No PQ on the roadmap as of mid-2026.** Documentation references "contact n0.computer for custom cryptographic configurations" — i.e., no PQ story.
- **No encrypt-to-recipient envelope.** iroh provides "channel encrypted between two endpoints," NOT "ciphertext that can rest on untrusted-host."

This is the critical adjacency: Benten builds on iroh's QUIC transport, but iroh's E2EE story is channel-only. For Benten's Atrium / Garden-Grove / Drop / Kith use cases, Benten MUST provide its own application-layer encrypt-to-recipient — iroh's transport encryption does NOT substitute. This is non-negotiable per the threat model captured in CLAUDE.md #18.

### 4.8 The ecosystem-fragmentation summary

The KEM CONSTRUCTION is converging: ML-KEM-768 + X25519 with SHA3-256 KDF is the ecosystem Schelling point. The KEM IDENTIFIER NAMING is plural:

| Naming | Ecosystem | Where used |
|---|---|---|
| `X25519MLKEM768` group `0x11EC` | TLS | Cloudflare, Chrome, Firefox |
| `MLKEM768-X25519` | HPKE / JOSE / COSE / MLS | draft-ietf-hpke-pq + draft-skokan + draft-ietf-mls-pq-ciphersuites |
| `id-MLKEM768-X25519` OID 1.3.6.1.5.5.7.6.X | LAMPS / PKIX / CMS / OpenPGP | OpenPGP-PQC, X.509 PQ certs |
| `id-X-Wing` multicodec `0x647A` | Multicodec-native | Benten current storage substrate, X-Wing draft |
| `X-Wing` ASCII string | draft-connolly-cfrg-xwing-kem | Reference impls |

These five naming systems are all the SAME construction. Benten's adoption of one or another is an ecosystem-citizenship statement. The most natural multi-stack-interop choice for an encrypt-to-recipient envelope on a P2P stack is **HPKE-RFC-9180 with the HPKE-PQ identifier naming**, because HPKE crosses JOSE / COSE / MLS / standalone-protocol use cases, and HPKE-PQ is at the alignment Schelling point for all three.

---

## 5. Timing-risk asymmetry

This is the section where my lens specifically adds signal beyond cryptographer / P2P-architect.

### 5.1 Asymmetry framing

Two failure modes for a v1-beta wire-format-freeze decision around encrypt-to-recipient:

- **Ship-too-early failure:** lock to pre-WG-adopted construction; construction gets superseded; v1-beta corpus uses an orphan codepoint
- **Ship-too-late failure:** miss the ecosystem alignment window; ship after JOSE/COSE/MLS have all converged on a specific identifier; Benten's content is ecosystem-orphaned even though the construction is identical

The cryptographer-review-of-Bird-of-Prey CONDITIONAL-NO-GO'd on ship-too-early grounds. The standards-skeptic-of-encrypt-to-recipient should weigh BOTH risks.

### 5.2 Ship-too-late evidence (this is the under-weighted risk)

The HPKE-PQ ecosystem alignment is happening IN MAY 2026:

- HPKE-PQ base draft progressing (`-04` published 2026-03-02; WG-adopted; Standards Track)
- HPKE-for-JOSE base in IESG Last Call (ends 2026-05-27)
- Skokan JOSE-PQ extension in WG-adoption-call (ends 2026-05-29)
- MLS-PQ ciphersuites draft progressing (`-04` published 2026-03-18)
- LAMPS Composite ML-KEM in IESG publication queue
- OpenPGP-PQC progressing to RFC

**Benten's v1-beta tag at this exact moment can ship on the alignment Schelling point.** If Benten defers (Option A) by 12-24 months, the ecosystem will have published Skokan's draft as `draft-ietf-jose-hpke-pq-pqt`, LAMPS Composite ML-KEM as an RFC, MLS-PQ as an RFC; Benten's late entry will need to retrofit interop with established identifier conventions.

### 5.3 Ship-too-early evidence

The ship-too-early risk is REAL but bounded by the recommended Option B' shape:

- **HPKE-RFC-9180 base:** published RFC. Zero supersession risk.
- **MLKEM768-X25519 construction:** mathematically identical to X-Wing per concrete-hybrid-kems §"This construction is identical to the X-Wing construction." Zero construction-soundness risk relative to what Benten already ships.
- **HPKE-PQ-named identifier:** in `draft-ietf-hpke-pq-04` (WG-adopted Standards Track). Bounded supersession risk: even if the draft is revised, the `MLKEM768-X25519` KEM name has been consistent across drafts -02, -03, -04.
- **ChaCha20-Poly1305:** RFC 8439. Zero supersession risk.
- **HKDF-SHA256:** RFC 5869. Zero supersession risk.
- **ML-KEM-768:** FIPS 203 final standard. Zero supersession risk.

**Every component of Option B' is either RFC, FIPS-standard, or in a WG-adopted Standards-Track draft.** This is the lowest ship-too-early risk of any concrete encrypt-to-recipient option.

### 5.4 Asymmetric outcomes

| Scenario | Option A (defer) | Option B' (HPKE-RFC + RG-KEM) | Option C (Skokan codepoints now) |
|---|---|---|---|
| HPKE-PQ becomes RFC unchanged | Defer-decision was conservative-cautious | Benten v1-beta content interops natively | Benten v1-beta content interops natively |
| HPKE-PQ KEM combiner changes (low probability) | Defer-decision pays off | Migrate via additive codepoint (crypto-agility framework) | Migrate via additive codepoint (crypto-agility framework) |
| Skokan adoption call fails 2026-05-29 (very low probability) | Defer-decision pays off | Benten's HPKE-RFC envelope still ships (Skokan is just naming) | Benten v1-beta locked to a NON-WG-adopted draft's identifiers |
| Skokan adopted but identifiers change pre-RFC | No impact | Migrate via additive codepoint | Codepoint break — v1-beta corpus may need re-issue |
| Ecosystem converges on Composite-ML-KEM-OID naming instead of HPKE-PQ naming | No impact | Mint a second codepoint per crypto-agility (Composite-ML-KEM-OID); existing content stays valid | Mint a second codepoint per crypto-agility; existing content stays valid |

Option B' has the lowest "bad outcome" across all scenarios because the construction is the ecosystem Schelling point regardless of which naming convention wins.

### 5.5 The cryptographer-review-of-Bird-of-Prey precedent

The cryptographer review of Bird-of-Prey-vs-LAMPS CONDITIONAL-NO-GO'd Bird-of-Prey on these grounds:

> "Bird-of-Prey is pre-standards. It is one paper (EUROCRYPT 2026), one (combined) IETF individual draft, zero Rust reference implementations, zero test-vector corpora. The author is one person. The peer review is one conference cycle. v1-beta wire-format freeze is a structural commitment that locks in the choice."

Applied to encrypt-to-recipient Options:

| Option | Pre-WG-adoption risk profile | Verdict by Bird-of-Prey logic |
|---|---|---|
| A defer | Misses alignment window (failure mode different) | Re-evaluate; defer-decision has its own risks |
| B (X-Wing-as-named inside HPKE base) | X-Wing-as-named is individual draft | Reject by Bird-of-Prey logic (matches Bird-of-Prey's "individual draft + one author" pattern) |
| **B' (RG-KEM-named inside HPKE base)** | All components RG-adopted or RFC | **Passes Bird-of-Prey logic** |
| C (Skokan codepoints) | Skokan is individual draft IN its adoption call | **FAILS Bird-of-Prey logic** — exactly the pre-WG-adoption pattern |
| D (Benten-tailored) | Benten-private = max pre-standards risk | Reject by Bird-of-Prey logic |
| E (MLS CGKA) | MLS-PQ has WG issues raised | Marginal — WG-adopted but not stable |
| F (B' refined) | Same as B' | Passes |

Consistency with the Bird-of-Prey verdict requires rejecting Option C. The brief's framing of Option B ("HPKE + X-Wing-as-named") implicitly carried pre-WG-adoption risk via the X-Wing-as-named identifier; the refinement to Option B' (RG-blessed naming for the same construction) closes that risk.

### 5.6 The "Benten is 5 weeks old with 1 contributor" framing applied

The Bird-of-Prey review emphasized this framing as load-bearing. Applied here:

- A 5-week-old / 1-contributor project should NOT be doing greenfield construction design (rejecting Option D).
- A 5-week-old / 1-contributor project should NOT be the first commercial adopter of an individual-draft codepoint allocation (rejecting Option C at v1-beta).
- A 5-week-old / 1-contributor project SHOULD be using WG-blessed constructions where they exist (favoring Option B').
- A 5-week-old / 1-contributor project SHOULD be using the well-analyzed envelope shape (HPKE RFC 9180) rather than authoring its own.

### 5.7 The iroh-transport-encryption-substitute fallacy

Some reviewers might argue "iroh's QUIC channel is E2EE; you don't need encrypt-to-recipient." This is wrong for Benten's threat model:

- iroh's E2EE protects the QUIC channel between two ACTIVE PEERS.
- Benten's Atrium model has CONTENT STORED ON intermediate peers (CLAUDE.md #18 "the moment a principal's subgraph rests on hardware another principal controls, capabilities give ZERO protection").
- Drop bundles pass through intermediate peers per CLAUDE.md baked-in #17.
- Garden-Grove untrusted-host (Phase-7+) has ciphertext-at-rest.

iroh's transport encryption protects bits-on-the-wire; it does not protect bits-at-rest-on-an-untrusted-third-party-peer. The two are different security properties. The cryptographer review of Bird-of-Prey explicitly recognizes this in §1: capabilities only bind a cooperating engine.

Cite-anchor: [iroh security-privacy docs](https://docs.iroh.computer/deployment/security-privacy) — "data is encrypted on the sender's device and can only be decrypted by the intended recipient" describes a CHANNEL, not a store-and-forward primitive. The recipient is the other QUIC endpoint, NOT a future-decryptor-after-rest-on-intermediate-peer.

This argument generalizes: any "the transport handles encryption" framing fails at the moment ciphertext rests on a third-party peer. Benten's vision REQUIRES application-layer encrypt-to-recipient.

---

## 6. Comparison to the Bird-of-Prey cryptographer review

### 6.1 What the Bird-of-Prey review established

The Bird-of-Prey cryptographer-review established three load-bearing process precedents that apply here:

1. **Pre-WG-adoption draft + 1 author + 1 conference cycle = reject as v1-beta default.** Bird-of-Prey met this profile; CONDITIONAL-NO-GO.
2. **The fix-the-application-layer pattern beats the change-the-algorithm pattern when both close the named hazards.** L12's revoke-by-sig-CID hazard is fixed by app-layer revoke-by-payload-CID, not by switching to SUF-CMA-preserving construction.
3. **WG-blessed Schelling point with ecosystem alignment > marginal cryptographic improvement.** LAMPS Composite ML-DSA preferred over Bird-of-Prey for ecosystem-alignment despite the SUF-CMA gap, because the gap is closeable by application-layer hygiene.

### 6.2 Consistency check for encrypt-to-recipient decision

Applying the SAME LENS to encrypt-to-recipient:

| Bird-of-Prey lesson | Applied here |
|---|---|
| Pre-WG-adoption draft + 1 author = reject | Option C (Skokan draft) fails this; Option B' passes |
| WG-blessed Schelling point > marginal improvement | Option B' (HPKE + RG-KEM) IS the Schelling point |
| Crypto-agility framework supports future-additive codepoints | All options preserve crypto-agility per CLAUDE.md #5 |
| Recovery cost asymmetric — broken-default trust-erodes corpus | Option B' has lowest broken-default probability (all components RFC or WG-adopted Standards-Track or RG-adopted) |

The Bird-of-Prey reviewer's recommendation pattern (LAMPS at `0x0001`, SUF-CMA reserved at `0x0002`, application-layer hygiene at `0x___`) maps cleanly to:

**Encrypt-to-recipient recommendation (mine):**
- Mint codepoint for `HPKE-Base-MLKEM768-X25519-HKDF-SHA256-ChaCha20Poly1305` as v1-beta encrypt-to-recipient default
- Reserve a second codepoint for Composite-ML-KEM-OID naming (for future PKIX/CMS interop) — typed-reject at v1-beta
- Reserve a third codepoint for MLS-CGKA group key derivation (for future Atrium-shared content) — typed-reject at v1-beta
- Application-layer: define the encrypt-to-recipient envelope's interaction with per-DID storage partition + plaintext-CID-bound AAD (extending the Inv-15 3-layer-decomposition pattern)

### 6.3 Where the two reviews disagree on framing

The Bird-of-Prey review concluded "don't ship Bird-of-Prey at v1-beta; stay LAMPS." I'm concluding "DO ship encrypt-to-recipient at v1-beta; use Option B'." These are not in tension — the Bird-of-Prey review was about adopting NEW cryptographic CONSTRUCTIONS (i.e., individual-draft constructions Benten would re-author); my encrypt-to-recipient recommendation uses ONLY ESTABLISHED CONSTRUCTIONS (HPKE-RFC-9180 + RG-adopted KEM combiner + RFC-published AEAD/KDF/AES-GCM-alt).

The cryptographer reviewer would (I predict) agree with Option B' for the same reasons they preferred LAMPS Composite ML-DSA — it's the WG-blessed Schelling-point construction with ecosystem alignment.

---

## 7. Per-option detailed GO/NO-GO

### 7.1 Option A — Defer

**Verdict: NO-GO at v1-beta.**

The defer-decision treats "do nothing pre-v1-beta-tag" as the conservative choice. From the standards-process lens, this is wrong: the conservative choice in May 2026 is to ship on the WG-converging Schelling point while the ecosystem is aligning, not to wait until after alignment when Benten's late entry has to retrofit interop.

The crypto-agility framework supports future-additive codepoints, which is fine for adding NEW cipher suites. But adding encrypt-to-recipient as a NEW WIRE-FORMAT FEATURE post-v1-beta is structurally different from adding a new cipher suite — it's a new VOCABULARY OF SHAPES (different envelope, different key-exchange semantics, different AAD bindings). Even with crypto-agility, ecosystem expectations are set at v1-beta tag.

The Position B blog impact is significant: shipping v1-beta WITHOUT encrypt-to-recipient means the public stance is "we have per-DID storage partition AEAD but no encrypt-to-recipient" — which directly contradicts the "P2P-untrusted-by-default hyperscaling feature" framing Ben articulated.

### 7.2 Option B — HPKE-RFC-9180 + X-Wing-as-KEM-combiner

**Verdict: CONDITIONAL GO, refine to Option B'.**

The HPKE envelope is the right shape (RFC, ecosystem-blessed). The "X-Wing-as-KEM-combiner-inside-HPKE-base" framing in the brief conflates the X-Wing draft (individual) with its mathematically-identical RG-adopted equivalent. Use the RG-adopted naming (`MLKEM768-X25519` from concrete-hybrid-kems) inside HPKE base mode — this is Option B'.

### 7.3 Option C — Skokan-draft HPKE-PQ-PQT codepoints

**Verdict: NO-GO at v1-beta tag. REVISIT at -01 post-adoption (~2026-Q3/Q4).**

Skokan's draft is currently in its WG-adoption call (ends 2026-05-29). Adopting its codepoints AT v1-beta tag is the literal definition of pre-WG-adoption-risk. Even though adoption is highly probable (4 explicit supporters, zero opposition documented), AND the construction is identical to what Option B' would use, the IDENTIFIER VALUES could change post-adoption.

Concretely:
- Skokan-draft -05 §3: lists 9 algorithm identifiers ("HPKE-Base-MLKEM768-X25519-HKDF-SHA256-AES-128-GCM" etc.)
- Post-WG-adoption, these identifiers go through IANA early-allocation — values may stabilize, but the JOSE-WG could rename, reorder, or remove some
- v1-beta corpus signed under Skokan-draft -05 identifiers would need verifier adaptation if names change

The crypto-agility framework supports MIGRATING the identifier mapping, but the cost is real (verifier-table updates across the consumer ecosystem; documentation drift; specifier-to-codepoint mapping rebases).

Better: ship Option B' at v1-beta with Benten-internal-codepoint naming; mint a Skokan-aligned codepoint additively once Skokan reaches `draft-ietf-jose-hpke-pq-pqt-01` (~2026-Q3/Q4 estimated).

### 7.4 Option D — Benten-tailored envelope (no HPKE)

**Verdict: NO-GO. Strongly against ecosystem-citizenship discipline.**

A 5-week-old, 1-contributor project minting its own envelope shape (instead of HPKE-RFC-9180) signals:
- "We didn't trust the WG-blessed envelope shape" — META-message that erodes adopter confidence
- "We don't want to be interoperable with JOSE/COSE/MLS/standalone HPKE consumers" — ecosystem isolation
- "We're authoring our own crypto" — pattern-matches the things CLAUDE.md #5 explicitly rules out ("never fork, never reimplement, crypto primitives")

The multicodec maintainers explicitly recommended container-form not Benten-specific envelopes per the L4 critic finding earlier on the public-stance question. Option D would re-introduce that anti-pattern after specifically deciding against it.

The size argument for Option D ("smallest implementation; tightest security analysis") doesn't hold up — HPKE base mode is small (~500 LOC); X-Wing KEM is small; the AEAD wrapper is small. The total HPKE-base-MLKEM768-X25519-Chacha20 implementation is ~600-800 LOC, fully comparable to the bespoke Option D path.

### 7.5 Option E — MLS-style CGKA

**Verdict: NO-GO at v1-beta. Right concept, wrong moment.**

MLS-PQ work IS RG-blessed-and-WG-adopted in 2026:
- `draft-ietf-mls-pq-ciphersuites-04` (March 2026) WG-adopted
- `draft-ietf-mls-combiner-02` (October 2025) WG-adopted (amortized PQ combiner)

But MLS-PQ is NOT v1-beta-ready:
- ciphersuites draft has "Revised I-D Needed - Issue raised by WG" status
- combiner draft is at -02
- Benten doesn't have an MLS implementation at all yet (would be net-new substantial subsystem)
- MLS-CGKA is the right model for Atrium-shared / community-shared content; encrypt-to-recipient (Drop-to-Bob) is a different shape — MLS doesn't replace single-recipient encryption

Right placement: Phase-4-Meta-Composing or Phase-5 once MLS-PQ stabilizes AND the Atrium-shared content model is well-specified. Reserve a codepoint at v1-beta for "future-MLS-CGKA" with typed-reject.

The amortized-PQ-MLS combiner is interesting but draft-02 maturity ≠ v1-beta-ready. The benchmarking paper [IACR 2026/034](https://eprint.iacr.org/2026/034) is recent.

### 7.6 Option F (per-extra-reflection-pass)

See §11.

---

## 8. Risks + mitigations

| Risk | Severity | Probability | Mitigation | Owner |
|---|---|---|---|---|
| HPKE-PQ draft identifier values change pre-RFC publication | LOW | LOW (KEM names stable across -02/-03/-04) | Use Benten-internal codepoint at v1-beta; mint Skokan-aligned codepoint additively once adopted | Benten core |
| Skokan adoption call fails 2026-05-29 (very unlikely) | LOW | VERY LOW (4 supports, 0 opposition) | No impact on Option B' — HPKE-RFC envelope ships regardless | N/A |
| Concrete-hybrid-kems RG draft changes the X-Wing-equivalent construction | VERY LOW | VERY LOW (construction "identical to X-Wing" explicitly in -03) | No real exposure — construction is what it is | N/A |
| Ecosystem converges on Composite-ML-KEM-OID naming for JOSE consumers (unlikely) | LOW | LOW (LAMPS targets CMS; JOSE targets HPKE) | Mint additive codepoint for Composite-ML-KEM-OID; existing content stays valid | Benten core |
| MLS-PQ becomes the ecosystem winner over HPKE for P2P encrypt-to-recipient | LOW | LOW (MLS targets group messaging; HPKE targets general envelope) | Reserve codepoint; add at Phase-4-Meta-Composing or Phase-5 | Ben sequencing |
| Apple iMessage PQ3-style Signal-protocol becomes dominant for P2P-messaging encrypt-to-recipient | MED | MED (Apple-precedent strong) | HPKE envelope serves the storage / Drop / general-recipient case; Signal-protocol-style is messaging-specific | N/A |
| ml-kem Rust crate ships new vuln post-v1-beta-tag (RUSTSEC pattern like ml-dsa) | HIGH | HIGH (analogous to ml-dsa Jan-May 2026 CVE cluster) | Pin version + 48h triage SLA + subscribe RustSec | Benten core |
| Composite-ML-KEM-OID identifier naming wins PKIX/CMS ecosystem; Benten's HPKE-PQ-named v1-beta corpus needs PKIX-side bridge | LOW | LOW (PKIX/CMS and JOSE/HPKE serve different consumer classes) | Bridge layer at consumer impl-time, not wire format | Future-additive |
| ChaCha20-Poly1305 deprecated in favor of AES-256-GCM in some ecosystem | VERY LOW | VERY LOW (NCC-audited per Compromise #30 mitigation) | Crypto-agility framework supports adding AES-GCM as additive codepoint | Benten core |
| Web Crypto API native HPKE support arrives, changes Benten thin-compute-surface implementation strategy | LOW | MED-LOW (no current proposal) | No wire-format impact; just implementation simplification | Benten core |

---

## 9. Honest disagreement with the brief framing

Several places where I think the brief framing benefits from refinement:

### 9.1 "Option B = HPKE + X-Wing"

The brief frames Option B as using "X-Wing as KEM combiner inside HPKE-mode-base." X-Wing-as-named is an INDIVIDUAL draft. The mathematically-identical RG-adopted alternative is `MLKEM768-X25519` from concrete-hybrid-kems. Using the RG-adopted naming costs nothing (same construction) and gains ecosystem alignment — so Option B as-framed has unnecessary pre-WG-adoption-risk that Option B' eliminates.

The brief notes "uses the existing X-Wing combiner we ship at storage substrate (consistency across layers)" — this is partially correct: the CONSTRUCTION is the same, the NAMES differ. Benten can have X-Wing-multicodec at the storage substrate (since that's already shipped per existing code) and HPKE-PQ-named at the encrypt-to-recipient layer; document the construction-equivalence in SECURITY-POSTURE.md. This maintains "consistency at the construction level" without forcing identifier-name conflation.

### 9.2 "Trade-off: HPKE-RFC-9180 is published + analyzed; X-Wing has IACR CIC 2024 peer-reviewed tight security proof; but using X-Wing-as-KEM-inside-HPKE-mode-base is a Benten-specific combination + needs cryptographer-validation that the composition is sound"

This framing is slightly misleading. HPKE-RFC-9180 base mode is defined for any KEM satisfying the HPKE KEM interface. X-Wing satisfies the HPKE KEM interface. The composition is NOT Benten-specific — it's well-defined by HPKE's modular design. The cryptographer-validation needed is no greater than the construction-soundness validation of using X-Wing at all (which Benten already accepts at the storage substrate per Spike-I findings).

What IS Benten-specific is the IDENTIFIER REGISTRATION. HPKE-RFC-9180 base mode + X-Wing-named-KEM is not registered in any IANA HPKE registry. HPKE-RFC-9180 base mode + `MLKEM768-X25519`-named KEM IS the construction that will be registered when draft-ietf-hpke-pq becomes an RFC. Using the latter naming avoids needing a Benten-private identifier.

### 9.3 "Position B blog framing impact" — the brief asks per-option

For Option B' (my recommendation), Position B framing strengthens:

- **Strong claim:** "Benten v1-beta ships HPKE-RFC-9180 encrypt-to-recipient with the same `MLKEM768-X25519` PQ-hybrid KEM combiner that JOSE / COSE / MLS / OpenPGP-PQC are all converging on in 2026."
- **Verifiable claim:** Yes — cite-anchored to draft-ietf-hpke-pq-04, draft-ietf-mls-pq-ciphersuites-04, draft-skokan-jose-hpke-pq-pqt-05, draft-ietf-openpgp-pqc-17.
- **Vision claim:** "Benten realizes the full Principal-confidentiality story at v1-beta: per-DID storage partition AEAD + encrypt-to-recipient on the same combiner foundation. P2P-untrusted-by-default is shipped, not deferred."

For Option C, Position B is FRAGILE — would need explicit caveat about "we shipped on a draft in adoption call." This is the L4 ecosystem-fragmentation skeptic argument; not a deal-breaker but a real drag.

For Option D, Position B is DAMAGED — "we shipped our own envelope" is exactly the small-team-fragmenting-ecosystem META-message.

### 9.4 The "10x bigger scope than the planned v1-beta crypto wire-format" claim implicit in the brief

The brief implicitly treats encrypt-to-recipient as expanding v1-beta scope. From a standards-process lens, it's the OPPOSITE: encrypt-to-recipient is THE alignment point in 2026; per-DID storage partition AEAD WITHOUT encrypt-to-recipient is what's actually the ecosystem-orphaned shape. Benten's existing storage substrate is ALREADY using the X-Wing construction; adding encrypt-to-recipient at the same KEM-construction level is a NATURAL EXTENSION not a SCOPE EXPLOSION.

The actual LOC delta for HPKE-base over a Benten codebase that already ships X-Wing AEAD is approximately:
- HPKE-base envelope implementation: ~500-700 LOC
- ChaCha20-Poly1305 AEAD wiring: ~100 LOC (likely already shipped)
- HKDF-SHA256 KDF wiring: ~50 LOC (likely already shipped)
- Encrypt-to-recipient surface (Engine API + Engine internals): ~200-400 LOC
- Test surface (KAT corpus + property tests + interop): ~500-800 LOC
- SECURITY-POSTURE.md update: ~100 lines doc

Total: ~1500-2000 LOC, ~2-4 weeks at agent-dispatch tempo. Comparable to the LAMPS Composite ML-DSA wiring already planned for v1-beta.

---

## 10. Evidence base (cite-anchored)

### HPKE base RFC

- **RFC 9180:** Hybrid Public Key Encryption (HPKE), February 2022 — https://datatracker.ietf.org/doc/rfc9180/

### HPKE for JOSE base

- **draft-ietf-jose-hpke-encrypt-18** (2026-05-25): https://datatracker.ietf.org/doc/draft-ietf-jose-hpke-encrypt/
- Status: "In Last Call (ends 2026-05-27)" — IESG Last Call. Track: Proposed Standard.
- Classical-only (P-256, P-384, P-521, X25519, X448).

### HPKE-PQ extension

- **draft-ietf-hpke-pq-04** (2026-03-02): https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/
- WG-adopted. Track: "Intended status: Standards Track". Authors: Richard Barnes (Cisco), Deirdre Connolly (Selkie Cryptography).
- KEMs: `MLKEM768-P256`, `MLKEM768-X25519`, `MLKEM1024-P384`, pure-PQ variants.
- Section 7: "The IND-CCA security of the hybrid constructions used in this document is established in [CONCRETE]."

### Skokan JOSE-HPKE-PQ extension

- **draft-skokan-jose-hpke-pq-pqt-05** (2026-05-13): https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/
- Adoption call ends **2026-05-29**.
- Boilerplate: "This Internet-Draft is **not endorsed by the IETF** and has **no formal standing** in the IETF standards process."
- Adoption call thread: http://www.mail-archive.com/jose@ietf.org/msg07070.html
- Support thread (4 supports, 0 opposition): http://www.mail-archive.com/jose@ietf.org/msg07080.html
- Path-forward agreement at IETF 125: http://www.mail-archive.com/jose@ietf.org/msg07049.html

### COSE-HPKE-PQ extension

- **draft-reddy-cose-hpke-pq-pqt-03** (2026-05-13): https://datatracker.ietf.org/doc/draft-reddy-cose-hpke-pq-pqt/
- Individual draft.

### Concrete hybrid KEMs (IRTF CFRG)

- **draft-irtf-cfrg-concrete-hybrid-kems-03** (2026-03-02): https://datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/
- IRTF stream, RG-adopted (CFRG). Track: Informational.
- KEMs: `MLKEM768-P256`, `MLKEM768-X25519`, `MLKEM1024-P384`.
- Verbatim: "MLKEM768-X25519: A hybrid KEM composing ML-KEM-768 and Curve25519 using the CG framework... This construction is identical to the X-Wing construction."
- KDF: `SHA3-256`. PRG: SHAKE256.

### Generic hybrid KEMs (IRTF CFRG)

- **draft-irtf-cfrg-hybrid-kems-11** (2026-05-07): https://datatracker.ietf.org/doc/draft-irtf-cfrg-hybrid-kems/
- IRTF stream, RG-adopted. Track: Informational.
- Defines UG / UK / CG / CK combiner frameworks.
- CFRG adoption call thread (Jan 2024): https://mailarchive.ietf.org/arch/msg/cfrg/PSvLFyBWDdrRaOmaXBStRpGoH3c/

### X-Wing individual draft

- **draft-connolly-cfrg-xwing-kem-10** (2026-03-02): https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/
- Individual submission. Track: (None) per datatracker.
- Boilerplate: "This I-D is **not endorsed by the IETF** and has **no formal standing** in the IETF standards process."
- IACR paper: Barbosa et al., "X-Wing: The Hybrid KEM You've Been Looking For," CIC 2024-1, https://eprint.iacr.org/2024/039

### LAMPS Composite ML-KEM

- **draft-ietf-lamps-pq-composite-kem-14** (2026-03-27): https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-kem/
- WG-adopted. Track: Proposed Standard.
- Status: **"Submitted to IESG for Publication"** with **"Publication Requested"** — IESG queue.
- OIDs IANA early-allocated: 1.3.6.1.5.5.7.6.55 through .66.
- KDF combiner: `SHA3-256(mlkemSS || tradSS || tradCT || tradPK || Label)`.

### LAMPS CMS Composite KEM

- **draft-ietf-lamps-cms-composite-kem-00** (2026-02-25): https://datatracker.ietf.org/doc/draft-ietf-lamps-cms-composite-kem/

### MLS PQ ciphersuites

- **draft-ietf-mls-pq-ciphersuites-04** (2026-03-18): https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- WG-adopted. Track: Proposed Standard. Status: "Waiting for WG Chair Go-Ahead" with "Revised I-D Needed".
- KEMs: from `[I-D.ietf-hpke-pq]`.

### MLS amortized PQ combiner

- **draft-ietf-mls-combiner-02** (2025-10-22): https://datatracker.ietf.org/doc/draft-ietf-mls-combiner/
- WG-adopted. Benchmarking: https://eprint.iacr.org/2026/034

### OpenPGP-PQC

- **draft-ietf-openpgp-pqc-17** (2026): https://datatracker.ietf.org/doc/draft-ietf-openpgp-pqc/
- Mandates ML-KEM-768+X25519, ML-DSA-65+Ed25519.
- OpenPGP Email Summit 2026 minutes: https://www.openpgp.org/community/email-summit/2026/minutes/

### Apple iMessage PQ3

- Apple Security Research blog: https://security.apple.com/blog/imessage-pq3/
- Formal analysis (USENIX 2025): https://www.usenix.org/conference/usenixsecurity25/presentation/linker
- Formal model (IACR 2024/1395): https://eprint.iacr.org/2024/1395

### Signal PQXDH + Triple Ratchet

- PQXDH spec: https://signal.org/docs/specifications/pqxdh/
- Signal blog on PQXDH: https://signal.org/blog/pqxdh/
- Triple Ratchet (2025): per Signal announcements

### Cloudflare PQ deployment

- State of PQ Internet 2025: https://blog.cloudflare.com/pq-2025/
- IPsec PQ adoption (InfoQ 2026-03): https://www.infoq.com/news/2026/03/cloudflare-post-quantum-ipsec/
- WARP PQ: https://blog.cloudflare.com/post-quantum-warp/

### iroh

- Security/Privacy docs: https://docs.iroh.computer/deployment/security-privacy
- Iroh website: https://www.iroh.computer/
- GitHub: https://github.com/n0-computer/iroh
- Verbatim: "data is encrypted on the sender's device and can only be decrypted by the intended recipient" — describes a CHANNEL.

### Nostr NIP-44

- Spec: https://github.com/nostr-protocol/nips/blob/master/44.md
- NIPS.nostr.com: https://nips.nostr.com/44
- Cure53-audited 2023.

### Browser PQ adoption

- F5 Labs State of PQ on Web (2026): https://www.f5.com/labs/articles/threat-intelligence/the-state-of-pqc-on-the-web
- 8.6% of top-1M websites support hybrid PQC.
- Chrome ML-KEM switch (2024): https://thehackernews.com/2024/09/google-chrome-switches-to-ml-kem-for.html

### Web Crypto API HPKE

- W3C Web Crypto Level 2: https://w3c.github.io/webcrypto/
- hpke-js (third-party impl on Web Crypto API): https://github.com/dajiaji/hpke-js
- HPKE-on-Web-Crypto NPM: https://www.npmjs.com/package/hpke

### Cloudflare X25519MLKEM768 deployment

- TLS group `0x11EC` for `X25519MLKEM768`.
- Quote: "Cloudflare deployed a hybrid of X25519+Kyber768 on production domains"
- ~43% of human-generated connections by Sept 2025.

---

## 11. Extra-reflection-pass output: more elegant permanent shape

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`: after per-option analysis, look for a holistic structural shape closing N findings at once.

### 11.1 The elegant shape

**Three-layer-decomposition for confidentiality, isomorphic to the Inv-15 3-layer decomposition for signatures.**

The Inv-15 / Compromise #31 ratification (cite: status header, "LAMPS EUF-CMA-only at construction; SUF-equivalent at application layer via 3-layer decomposition") established the pattern:

- **Layer 1 (construction):** EUF-CMA cryptographic primitive — LAMPS Composite ML-DSA
- **Layer 2 (identity):** content-CID derived from canonical payload (not signature) — application invariant
- **Layer 3 (revocation):** revoke-by-semantic-tuple — application invariant

The elegant shape for encrypt-to-recipient:

- **Layer 1 (construction):** HPKE-RFC-9180 + `MLKEM768-X25519` KEM (RG-blessed; ecosystem-aligned)
- **Layer 2 (envelope):** canonical-encrypt-to-recipient envelope binding ciphertext to recipient-DID + content-CID-of-plaintext + sender-DID + timestamp via HPKE AAD
- **Layer 3 (application invariants):** envelope binds same per-DID-storage-partition seam as the AEAD substrate (Inv-11/C1) + plaintext-CID-bound AAD (rebinding-attack-prevention from §1.A.FROZEN item 15(g)) + recipient-rotation semantics defined at the application layer (rotation = mint new HPKE keypair, additive)

### 11.2 Why this is the actually-correct shape

The construction half (HPKE-RFC-9180 + RG-KEM) closes the standards-maturity question.
The envelope half closes the rebinding-attack / cross-chunk-truncation / cross-recipient-confusion hazards by application-layer AAD binding.
The application-invariants half closes the long-tail of "what does revocation mean" / "what does rotation mean" / "what does multi-device mean" questions without re-opening the wire format.

This three-layer decomposition is symmetric with Inv-15 — Benten now has 3-layer decomposition for BOTH signatures (Inv-15) AND encryption (Inv-16 — proposed by this review). The pattern is:

- Cryptographic primitive (Layer 1) provides ONE security property (EUF-CMA for signatures; IND-CCA for KEM)
- Application binding (Layer 2) provides additional needed properties (SUF-equivalent for signatures; recipient/content/sender binding for KEMs)
- Application invariants (Layer 3) provide higher-level properties (revoke-by-tuple for signatures; recipient-rotation-semantics for KEMs)

### 11.3 Concrete codepoint layout proposal

| Codepoint | Construction | Role | Status |
|---|---|---|---|
| `ETR-0x0001` | HPKE-Base-MLKEM768-X25519-HKDF-SHA256-ChaCha20Poly1305 | v1-beta DEFAULT encrypt-to-recipient | LIVE at v1-beta |
| `ETR-0x0002` | Reserved for Skokan-aligned identifier once `draft-ietf-jose-hpke-pq-pqt-01` | Future opt-in (Skokan-named) | Typed-reject at v1-beta; mint when Skokan-adopted |
| `ETR-0x0003` | Reserved for Composite-ML-KEM-OID naming (PKIX/CMS interop) | Future opt-in | Typed-reject at v1-beta |
| `ETR-0x0004` | Reserved for MLS-CGKA group-key for Atrium-shared content | Future opt-in | Typed-reject at v1-beta |
| `ETR-0x0005` | Reserved for classical-only X25519 fallback (interop) | Optional opt-in | LIVE at v1-beta if needed |
| `ETR-0x0006` | Reserved for NF-1 PQ⊕PQ (per CLAUDE.md #5) | Future opt-in | Typed-reject at v1-beta |

### 11.4 Why this beats greedy-per-option

Compared to greedy-per-option analysis:

- Option B' alone closes the construction question
- Layer 2 (envelope binding) closes ~5 application-binding hazards
- Layer 3 (invariants) closes ~3 application-semantics questions
- Total closure: ~9 named hazards (per the brief's #1-#9 evaluation criteria) via ONE elegant structural shape

Bonus: the 3-layer decomposition can be POSITION-B'd uniformly with the Inv-15 LAMPS decomposition — "Benten's posture on both signatures and encryption is 3-layer-decomposition: WG-blessed Layer-1 construction, application-CID-bound Layer-2 envelope, semantic-invariant Layer-3" — strong, symmetric, defensible.

### 11.5 Operational consequence

Replace the algorithm-per-option investigation with this engineering plan:

1. **Mint Inv-16 (proposed):** "Encrypt-to-recipient binds plaintext-CID + recipient-DID + sender-DID into HPKE AAD; rotation is additive; revocation is application-layer at the cap-policy layer."
2. **Mint Compromise #32 (proposed):** "v1-beta encrypt-to-recipient ships HPKE-RFC-9180 base with `MLKEM768-X25519` per concrete-hybrid-kems; Skokan-aligned naming added additively post-adoption."
3. **Update SECURITY-POSTURE.md** with the encrypt-to-recipient envelope shape + the 3-layer decomposition table.
4. **Reserve the codepoint table above** in §1.A.FROZEN.
5. **Implementation wave:** ~1500-2000 LOC over ~2-4 weeks of agent-dispatch. Cryptographer-review of the HPKE-base wrapper (~3-5 days) + KAT corpus (from RFC 9180 + concrete-hybrid-kems test vectors when published).

---

## 12. Self-assessment

### Confidence level

- **HIGH confidence** on: (a) HPKE / HPKE-PQ / JOSE-HPKE / MLS-PQ standards-process state (verified against datatracker primary sources); (b) ecosystem-adoption-precedent table (verified against deployment evidence); (c) the timing-window argument for shipping on the alignment Schelling point in May 2026; (d) Option B's "X-Wing-named vs MLKEM768-X25519-named" identifier-naming subtlety; (e) the construction-equivalence of X-Wing and `MLKEM768-X25519` in concrete-hybrid-kems.

- **MEDIUM confidence** on: (a) the exact LOC estimate for Option B' implementation; (b) the Position B blog framing impact (depends on how the blog already presents the storage substrate); (c) what Apple iMessage PQ3 / Signal Triple Ratchet specifically would inform Benten's encrypt-to-recipient design (these are messaging-class, not envelope-class); (d) the MLS-PQ timeline (depends on WG-chair-go-ahead resolving the open issue).

- **LOWER confidence** on: (a) whether Benten's existing X-Wing-multicodec-`0x647A` codepoint binding choice constrains the encrypt-to-recipient codepoint binding choice in any way I haven't accounted for; (b) whether the AAD binding shape I'm proposing in §11 (plaintext-CID + recipient-DID + sender-DID) is the right shape for the Inv-11/C1 per-DID-storage-partition seam — would need to verify against Engine source; (c) the WG-chair-go-ahead-issue at MLS-PQ — couldn't read the issue text. **Ben should verify these before acting on the specific Option F shape.**

### Additional review I would want before commit

1. **Direct source-read of Benten's existing X-Wing AEAD substrate** — confirm the construction matches `MLKEM768-X25519` exactly (per the construction-equivalence claim) and not some divergence.
2. **Cryptographer review of the HPKE-base-MLKEM768-X25519-ChaCha20 wrapper** — same shape as the LAMPS Composite ML-DSA wrapper review. ~3-5 days.
3. **A second standards-skeptic review on the question of Skokan adoption-call outcome** — if Skokan adoption fails on 2026-05-29, the timeline analysis shifts. I'm assuming adoption succeeds (4-0 mailing list support, IETF 125 path-forward agreement).
4. **A read of the MLS-PQ "Issue raised by WG"** — if the issue affects KEM identifier values, MLS naming might diverge from JOSE naming.
5. **The exact wording of the Position B blog** so I can sharpen the framing-impact analysis per-option.

### What this review does NOT cover

- The Bird-of-Prey vs LAMPS signature decision (separate review)
- The Wave DID / Kith identity-recovery question (orthogonal)
- The transport-layer encrypt-to-recipient story for iroh's WireGuard-style alternatives (out of scope)
- Encryption-at-rest as a separable concept from encrypt-to-recipient (out of scope; they share constructions but are different threat models)

### Self-critique

I'm aware that my standards-skeptic lens biases me toward "use the WG-blessed Schelling point." This is a defensible bias for a 5-week-old / 1-contributor project preparing v1-beta tag, but it's a bias. Specifically:

- I'm probably under-weighting the cost of HPKE base implementation in wasm32-unknown-unknown for the thin-compute-surface deployment shape. If the wasm bundle bloat is significant, Option D's bespoke envelope might have real cost-savings I'm dismissing.
- I'm probably under-weighting the Apple iMessage PQ3 / Signal precedent for "P2P-messaging-class encrypt-to-recipient via Signal-protocol-style ratchets." That's a different shape than HPKE envelope; for Atrium-shared content with messaging-class use cases (Ben's "P2P-untrusted-by-default hyperscaling" framing), Signal-protocol-style might be more appropriate.
- I'm probably under-weighting the Phase-4-Meta-Composing / Phase-5 timeline pressure. If Benten can ship v1-beta WITHOUT encrypt-to-recipient and add it at Phase-4-Meta-Composing-close (~6-12 months later), the marginal benefit of shipping it at v1-beta vs Phase-4-Meta-Composing-close might be smaller than my Section 5.2 evidence implies.

The recommendation in §1 should be read as: **"the ecosystem-alignment window is open in May 2026; the WG-converging Schelling-point construction is buildable on top of Benten's existing X-Wing substrate; the elegant 3-layer decomposition closes the application-layer hazards uniformly with Inv-15. So ship Option B' at v1-beta if the implementation tempo allows; defer to Phase-4-Meta-Composing-close if not. The construction choice doesn't change either way."**

---

## 13. Honest disagreement (unconditional)

Beyond §9, three places where I disagree with the framing more fundamentally:

### 13.1 The "P2P-untrusted-by-default is our hyper-scaling feature" framing

Ben's framing — "P2P is as secured/untrusted as possible should just be our default since that's very much what we're building towards as our hyper-scaling feature" — implicitly couples encrypt-to-recipient to a "hyper-scaling feature" framing. From the standards-skeptic lens, encrypt-to-recipient is NOT a hyper-scaling feature — it's a BASELINE THREAT-MODEL ASSUMPTION. The Atrium / Garden-Grove / Drop / Kith use cases REQUIRE it; they don't OPTIMIZE for it.

Framing encrypt-to-recipient as "scaling feature" risks under-investing in the Layer 2/3 application-layer disciplines (the elegant shape in §11) and over-investing in Layer 1 construction choice. The construction is straightforward (Option B'); the application binding is where Benten earns its differentiation.

### 13.2 The "v1-beta wire freeze locks default cipher suite choices" framing

The brief says "v1-beta will tag a frozen wire format that locks default cipher suite choices." This is partially correct but understates what crypto-agility per CLAUDE.md #5 actually provides. The DEFAULT at v1-beta tag is what existing implementations write; existing content stays valid forever (per "never wire-break"). But the additive-codepoints framework means ADDING a NEW default cipher suite later is structurally fine — old content uses the v1-beta default codepoint; new content uses the new codepoint. This is much weaker than "lock-in" framing implies.

So the v1-beta decision is "what does the FIRST FEW MONTHS of v1-beta content use" — not "what does Benten ever do." The risk-asymmetry analysis in §5 holds, but the framing should be calibrated to "this is the first-mover advantage / disadvantage on a multi-codepoint system, not a single-binding-choice."

### 13.3 The "Option F = something else" framing

The brief allows for Option F as a 6th option. My Option F (Option B' refined + 3-layer decomposition) is not really a 6th option — it's a refinement of Option B with an additional application-layer pattern (Inv-16). The brief's option-space implicitly treats construction choice and application-layer pattern as orthogonal axes. From the standards-skeptic lens, they're not orthogonal: the WG-blessed constructions come with implicit assumptions about application-layer bindings (HPKE AAD usage, KEM-encapsulation semantics, recipient-rotation patterns). Treating them orthogonally risks repeating the L12 finding's mistake (algorithm choice obscuring application-layer architectural decisions).

A cleaner option space would be:
- **Construction axis:** {RFC, RG-adopted, WG-adopted-draft, individual-draft, Benten-bespoke}
- **Envelope axis:** {HPKE-base, HPKE-mode-PSK, MLS-CGKA, Signal-protocol, Benten-bespoke}
- **Binding axis:** {3-layer-decomposition, ad-hoc, none}

The 3-axis lens makes the "elegant shape" of §11 obvious: pick (RG-adopted, HPKE-base, 3-layer-decomposition) and the rest follows.

---

*End of review. Reviewer: standards-and-ecosystem skeptic, engaged 2026-05-26.*
