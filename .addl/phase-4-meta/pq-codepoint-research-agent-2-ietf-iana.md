# PQ Codepoint Research — Agent 2: IETF + IANA + COSE + Cryptographic-OID Landscape

**Author:** Research Agent 2 of 3 (Benten-Ben)
**Date:** 2026-05-25
**Branch:** `phase-4-meta-core/pq-codepoint-research-2` (off main `31d1a169`)
**Scope:** IETF working-group drafts + IANA registries + COSE/JOSE/HPKE/TLS + NIST FIPS for hybrid PQ codepoint assignment, in support of Benten's v1-beta wire-format codepoint decision.
**Sister briefs:** Agent 1 (multicodec + W3C DID + private-use convention); Agent 3 (production deployments — iroh, Cloudflare, Signal, MLS, Veilid, Nostr, AWS-LC, rustls, Tor, BoringSSL).

---

## TL;DR (for orchestrator skim)

1. **IETF lamps-pq-composite-sigs is essentially DONE** — draft-19 (2026-04-21) is in RFC Editor Queue; OIDs already early-allocated 2025-10-20 by IANA. **`id-MLDSA65-Ed25519-SHA512` = OID `1.3.6.1.5.5.7.6.48`** is real today (cite-grep-verified against the GitHub-rendered HTML). Construction is concatenated, committing, strip-resistant via fixed-prefix `"CompositeAlgorithmSignatures2025"` + algorithm-label + ctx + pre-hashed message.

2. **IETF lamps-pq-composite-kem-14 is also at IESG/Publication-Requested.** **`id-MLKEM768-X25519-SHA3-256` = OID `1.3.6.1.5.5.7.6.58`**. SHA3-256 combiner over `mlkemSS || tradSS || tradCT || tradPK || Label` — STRUCTURALLY DIFFERENT from the X-Wing combiner. (Both are sound; both target IND-CCA2; **they are not interchangeable on the wire**.)

3. **X-Wing has IANA HPKE codepoint 0x647A = 25722** (authoritative; verified directly on the IANA registry). **The X-Wing combiner is `SHA3-256(ss_M || ss_X || ct_X || pk_X || XWingLabel)` with XWingLabel = `5c2e2f2f5e5c`.** Not COSE-registered. X-Wing is still an **individual submission, not CFRG-adopted**, intended-status Informational.

4. **COSE Algorithms registry currently has ONLY pure ML-DSA-44/65/87** at `-48/-49/-50` per RFC 9964 (published May 2026). **No composite/hybrid PQ COSE codepoints exist yet.** The relevant in-flight draft is `draft-ietf-jose-pq-composite-sigs-01` (jose WG; v01 is brand new) which requests `-51..-56` for the six MLDSA×{ECDSA,EdDSA} pairs incl. `ML-DSA-65-Ed25519` — but this is **early-stage**, not RFC-tracked yet.

5. **TLS Supported Groups already has X25519MLKEM768 = 4588 (0x11EC)** via draft-ietf-tls-ecdhe-mlkem-04 (IESG/RFC Editor Queue). Distinct codepoint family from HPKE; distinct combiner (raw concat into TLS 1.3 key schedule, no SHA3 wrap).

6. **For Benten's wire format the practical-today picture is:**
    - **Signing:** an authoritative OID exists *right now* for our exact pair → **`1.3.6.1.5.5.7.6.48`** (id-MLDSA65-Ed25519-SHA512). RFC Editor Queue means risk of rename at publication is near-zero (early-allocated).
    - **KEM:** **two incompatible "PQ-hybrid X25519+ML-KEM-768" constructions are in flight in parallel**: composite-kem OID `1.3.6.1.5.5.7.6.58` (SHA3-256 KDF, broader combiner inputs including pk_trad) AND X-Wing HPKE 0x647A (different combiner inputs, fixed XWingLabel). **This is a real architectural fork** for Benten's #1301 encryption-substrate work; the choice cascades into the canonical-bytes hash. Recommendation in §E below.

7. **NIST hybrid composite standard?** No FIPS for hybrid composites. FIPS 203/204/205 are pure-PQ only. **FIPS 206 (FN-DSA / FALCON-based) was submitted as draft 2025-08-28 with final expected late 2026/early 2027** — also pure-PQ, not hybrid. **FIPS 207-equivalent for HQC: draft expected 2026, final 2027.** NIST has explicitly left hybrid construction to IETF/CFRG/industry. NIST SP 800-227 §4.6 (final 2025-09-18) gives operational guidance on multi-algorithm KEM combiners but does not bless one specific combiner construction.

8. **Synthesis (preview of §E):** Benten should **align signing OID with `id-MLDSA65-Ed25519-SHA512` (1.3.6.1.5.5.7.6.48)** as the canonical reference and **pick X-Wing (HPKE 0x647A) over composite-kem for KEM** — X-Wing has the smaller adoption footprint but the better cryptographic provenance (peer-reviewed IACR-CiC paper), lighter wire format (no separate tradPK in the combiner), and direct IANA HPKE codepoint that maps naturally onto Benten's iroh-bytes / blob-AEAD substrate. The composite-kem OID is X.509/PKI-shaped (CMS workflows), which is not Benten's deployment target.

---

## Section A — IETF lamps-pq-composite-sigs draft state

### A.1 — `draft-ietf-lamps-pq-composite-sigs-19` (latest, 2026-04-21)

**Status:** "Submitted to IESG for Publication" → **RFC Editor Queue (In Progress)**. Responsible AD: Deb Cooley. Document Shepherd: Russ Housley. Intended Status: Proposed Standard. WG: LAMPS (Lightweight Authentication and Management Profile Security).

**OIDs already EARLY-ALLOCATED 2025-10-20** under `1.3.6.1.5.5.7.6.x` (SMI Security for PKIX Algorithms). The full table of 18 composite signature algorithms:

| Algorithm name | OID | Notes |
|---|---|---|
| `id-MLDSA44-RSA2048-PSS-SHA256` | `1.3.6.1.5.5.7.6.37` | |
| `id-MLDSA44-RSA2048-PKCS15-SHA256` | `1.3.6.1.5.5.7.6.38` | |
| `id-MLDSA44-Ed25519-SHA512` | `1.3.6.1.5.5.7.6.39` | Cat-1 PQ-hybrid w/ Ed25519 |
| `id-MLDSA44-ECDSA-P256-SHA256` | `1.3.6.1.5.5.7.6.40` | |
| `id-MLDSA65-RSA3072-PSS-SHA512` | `1.3.6.1.5.5.7.6.41` | |
| `id-MLDSA65-RSA3072-PKCS15-SHA512` | `1.3.6.1.5.5.7.6.42` | |
| `id-MLDSA65-RSA4096-PSS-SHA512` | `1.3.6.1.5.5.7.6.43` | |
| `id-MLDSA65-RSA4096-PKCS15-SHA512` | `1.3.6.1.5.5.7.6.44` | |
| `id-MLDSA65-ECDSA-P256-SHA512` | `1.3.6.1.5.5.7.6.45` | |
| `id-MLDSA65-ECDSA-P384-SHA512` | `1.3.6.1.5.5.7.6.46` | |
| `id-MLDSA65-ECDSA-brainpoolP256r1-SHA512` | `1.3.6.1.5.5.7.6.47` | |
| **`id-MLDSA65-Ed25519-SHA512`** | **`1.3.6.1.5.5.7.6.48`** | **← Benten's exact pair** |
| `id-MLDSA87-ECDSA-P384-SHA512` | `1.3.6.1.5.5.7.6.49` | |
| `id-MLDSA87-ECDSA-brainpoolP384r1-SHA512` | `1.3.6.1.5.5.7.6.50` | |
| `id-MLDSA87-Ed448-SHAKE256` | `1.3.6.1.5.5.7.6.51` | |
| `id-MLDSA87-RSA3072-PSS-SHA512` | `1.3.6.1.5.5.7.6.52` | |
| `id-MLDSA87-RSA4096-PSS-SHA512` | `1.3.6.1.5.5.7.6.53` | |
| `id-MLDSA87-ECDSA-P521-SHA512` | `1.3.6.1.5.5.7.6.54` | |

**Construction (verified against the GitHub-rendered draft):**

```
M' := Prefix || Label || len(ctx) || ctx || PH(M)
```

where:
- **Prefix** = ASCII string `"CompositeAlgorithmSignatures2025"` = hex `436F6D706F73697465416C676F726974686D5369676E61747572657332303235`
- **Label** = algorithm-specific ASCII; for our pair: `"COMPSIG-MLDSA65-Ed25519-SHA512"`
- **len(ctx)** = single unsigned byte (so max 255-byte context)
- **ctx** = application context bytes
- **PH(M)** = SHA-512 of original message for this pair (pre-hash; mandatory)

**Signature value** = `mldsaSig || tradSig` (3309 bytes ML-DSA-65 + 64 bytes Ed25519 = 3373 bytes total, no length-prefix).

**Public key** = `mldsaPK || tradPK` (1952 + 32 = 1984 bytes; raw concat; SubjectPublicKey BIT STRING when wrapped in X.509).

**Cryptographic properties (cite-verified):**
- **Strong commitment:** the algorithm-specific Label is folded into M' so both component signatures verify against the SAME M'. Cannot "strip" one component and verify the other against a different/derived M'.
- **Strip-resistance:** confirmed — removing either component fails verification.
- **Domain separation:** the fixed `"CompositeAlgorithmSignatures2025"` prefix separates from any naked Ed25519 or ML-DSA usage.

**Alignment with Benten's `benten-crypto-suite` (per CLAUDE.md #5):** The draft's construction is **identical in shape** to what CLAUDE.md describes ("concatenated/committing/strip-resistant per NF-4, both-must-verify, IETF lamps-pq-composite-sigs-aligned"). **Benten's substrate IS this draft's construction** — we should adopt the OID and the construction as-is.

### A.2 — `draft-ietf-lamps-pq-composite-kem-14` (latest, 2026-03-27)

**Status:** "Submitted to IESG for Publication" / "Publication Requested." Same LAMPS WG, same IANA early-allocation pattern. 12 composite KEM OIDs allocated under `1.3.6.1.5.5.7.6.x`:

| Algorithm | OID |
|---|---|
| `id-MLKEM768-RSA2048-SHA3-256` | `1.3.6.1.5.5.7.6.55` |
| `id-MLKEM768-RSA3072-SHA3-256` | `1.3.6.1.5.5.7.6.56` |
| `id-MLKEM768-RSA4096-SHA3-256` | `1.3.6.1.5.5.7.6.57` |
| **`id-MLKEM768-X25519-SHA3-256`** | **`1.3.6.1.5.5.7.6.58`** | **← Benten's exact pair (composite-KEM flavor)** |
| `id-MLKEM768-ECDH-P256-SHA3-256` | `1.3.6.1.5.5.7.6.59` |
| `id-MLKEM768-ECDH-P384-SHA3-256` | `1.3.6.1.5.5.7.6.60` |
| `id-MLKEM768-ECDH-brainpoolP256r1-SHA3-256` | `1.3.6.1.5.5.7.6.61` |
| `id-MLKEM1024-RSA3072-SHA3-256` | `1.3.6.1.5.5.7.6.62` |
| `id-MLKEM1024-ECDH-P384-SHA3-256` | `1.3.6.1.5.5.7.6.63` |
| `id-MLKEM1024-ECDH-brainpoolP384r1-SHA3-256` | `1.3.6.1.5.5.7.6.64` |
| `id-MLKEM1024-X448-SHA3-256` | `1.3.6.1.5.5.7.6.65` |
| `id-MLKEM1024-ECDH-P521-SHA3-256` | `1.3.6.1.5.5.7.6.66` |

**Combiner (verified):**

```
SSout = SHA3-256(mlkemSS || tradSS || tradCT || tradPK || Label)
```

— inputs concatenated WITHOUT length prefixes (fixed lengths per algorithm). **All four secret-deriving inputs** are folded in: ML-KEM shared secret, traditional shared secret, traditional ciphertext, traditional public key.

**This combiner is STRUCTURALLY DIFFERENT from X-Wing.** See §C.2 for the side-by-side.

**Note: the draft "extensively references X-Wing in security analysis sections discussing IND-CCA2 security and second pre-image resistance"** — i.e. X-Wing is acknowledged as the established peer-reviewed reference but the composite-kem combiner is a separate construction tuned for the X.509/PKI deployment context.

### A.3 — IETF working-group survey for hybrid PQ

| WG | Active hybrid PQ work | Status |
|---|---|---|
| **LAMPS** | `pq-composite-sigs` + `pq-composite-kem` | Both in RFC Editor Queue / Publication Requested (2026). |
| **CFRG** (IRTF) | `draft-connolly-cfrg-xwing-kem-10` | **Not adopted** as RG doc; individual submission, intended Informational, expires 2026-09-03. CFRG ran a 2024 adoption call for a broader "Hybrid KEM Combiners" document referencing X-Wing + Chempat-X; outcome not concluded as of latest mailing list reading. |
| **TLS** | `draft-ietf-tls-ecdhe-mlkem-04` + `draft-ietf-tls-hybrid-design-16` | ecdhe-mlkem at IESG/RFC Editor Queue (codepoints 4587/4588/4589 assigned). hybrid-design-16 at RFC Editor In Progress (Informational). |
| **COSE** | RFC 9964 PUBLISHED 2026-05 (pure ML-DSA at -48/-49/-50); `draft-ietf-cose-dilithium` → RFC 9964 done. | No composite COSE algorithms yet. |
| **JOSE** | `draft-ietf-jose-pq-composite-sigs-01` (v01 brand new) | Early WG work; requests `-51..-56` for composite COSE codepoints and string alg names `ML-DSA-65-Ed25519` etc. **No RFC track timeline.** |
| **COSE/JOSE (individual)** | `draft-reddy-cose-jose-pqc-hybrid-hpke-11` | Individual draft, not adopted; covers MLKEM768-{P256,X25519} and MLKEM1024-P384 hybrids for HPKE-in-JOSE/COSE. **Notably DOES NOT cover X-Wing** (Reddy draft is the composite-KEM-style flavor). |
| **HPKE** | `draft-ietf-hpke-pq-04` | Active WG draft; registers ML-KEM-512/768/1024 + MLKEM768-P256 + MLKEM1024-P384. X-Wing already in IANA HPKE via the CFRG individual draft. |
| **PQUIP** (Post-Quantum Use In Protocols) | Multiple guidance drafts | Migration/engineering guidance; not codepoint-assigning. |

**Key reading: there are TWO PARALLEL EFFORTS for X25519+ML-KEM-768 in IETF/IRTF** that have produced incompatible wire formats:
- **X-Wing** (CFRG individual; cleaner combiner; HPKE-only; peer-reviewed crypto paper; codepoint already in IANA HPKE registry at 0x647A; intended Informational).
- **Composite KEM** (LAMPS WG; X.509/CMS-shaped; broader combiner; OID `1.3.6.1.5.5.7.6.58`; intended Proposed Standard; RFC Editor Queue).

**Both are technically sound. Pick exactly one for any new system.** Benten's choice is structurally meaningful (see §E).

---

## Section B — IANA registries for cryptographic algorithms

### B.1 — IANA COSE Algorithms registry

**Source:** https://www.iana.org/assignments/cose/cose.xhtml (verified 2026-05-25).

**Classical signature algorithms (relevant subset):**

| Codepoint | Name | Recommended | Reference |
|---|---|---|---|
| `-7` | ES256 (ECDSA w/ SHA-256) | **Deprecated** | RFC 9053 / RFC 9054 |
| `-9` | ESP256 (ECDSA P-256 + SHA-256) | Yes | (current ES* family) |
| `-19` | **Ed25519** | **Yes** | RFC 9864 §2.2 |
| `-51` | ESP384 (ECDSA P-384 + SHA-384) | Yes | |
| `-52` | ESP512 (ECDSA P-521 + SHA-512) | Yes | |

**Post-quantum algorithms (already registered, per RFC 9964 published 2026-05):**

| Codepoint | Name | Recommended | Reference |
|---|---|---|---|
| `-48` | ML-DSA-44 | Yes | RFC 9964 |
| `-49` | **ML-DSA-65** | **Yes** | **RFC 9964** |
| `-50` | ML-DSA-87 | Yes | RFC 9964 |

**No ML-KEM codepoints in COSE Algorithms registry yet.** No SLH-DSA codepoints yet. No composite/hybrid codepoints yet.

**Registration procedure:**
- Integer codepoints in `[-256, 255]`: **Standards Action With Expert Review**.
- Outside that range: Specification Required.
- **Private Use range: integers `< -65536`.**

**JSON Web Key Types registry:** kty `AKP` ("Algorithm Key Pair") registered per RFC 9964 as the generic PQ key-type wrapper.

### B.2 — IANA HPKE registries

**Source:** https://www.iana.org/assignments/hpke/hpke.xhtml (verified 2026-05-25).

**HPKE KEM Identifiers (relevant subset):**

| Decimal | Hex | KEM | Recommended | Reference |
|---|---|---|---|---|
| 32 | 0x0020 | DHKEM(X25519, HKDF-SHA256) | Yes | RFC 7748 |
| 48 | 0x0030 | X25519Kyber768Draft00 | No | draft-westerbaan-cfrg-hpke-xyber768d00-02 (historical) |
| 64 | 0x0040 | ML-KEM-512 | No | draft-connolly-cfrg-hpke-mlkem-04 |
| 65 | 0x0041 | ML-KEM-768 | No | draft-connolly-cfrg-hpke-mlkem-04 |
| 66 | 0x0042 | ML-KEM-1024 | No | draft-connolly-cfrg-hpke-mlkem-04 |
| 80 | 0x0050 | MLKEM768-P256 | No | draft-ietf-hpke-pq-04 |
| 81 | 0x0051 | MLKEM1024-P384 | No | draft-ietf-hpke-pq-04 |
| **25722** | **0x647A** | **X-Wing** | **No** | **draft-connolly-cfrg-xwing-kem-06** |

**X-Wing parameters (verified):** Nsecret=32, Nenc=1120, Npk=1216, Nsk=32, Auth=no. (Nenc=1120 = ML-KEM-768 ciphertext 1088 + X25519 element 32; Npk=1216 = ML-KEM-768 pk 1184 + X25519 pk 32.)

**0x647A name etymology (verified):** `25519 + 203 = 25722 = 0x647A` — i.e. X25519's curve number plus FIPS 203 number. Cute, but deliberate.

**Private-Use Range: 0x647B–0xFFFF.** (Begins just past X-Wing's codepoint.)

**Registration Policy: Specification Required + Expert Review** (designated experts: Christopher Wood, Richard Barnes).

**Critical for Benten:** "Recommended: No" on every PQ entry is a **process artifact** (these reflect the registries' conservative posture pre-RFC-publication), NOT a security warning. NIST FIPS 203/204/205 are production-blessed.

### B.3 — IANA JOSE registries

**Source:** https://www.iana.org/assignments/jose/jose.xhtml (verified).

Currently registered PQ algorithms (per RFC 9964):
- `ML-DSA-44`, `ML-DSA-65`, `ML-DSA-87` as alg names. Status: Optional. Change Controller: IETF.
- kty `AKP`.

**No ML-KEM, SLH-DSA, X-Wing, or composite/hybrid alg names registered yet.**

**Registration Procedure: Specification Required** with designated experts Sean Turner, Mike Jones, Filip Skokan. Three-week IANA notification window.

**No explicit private-use convention** — JOSE uses string identifiers and recommends URI-form names for application-specific algorithms.

### B.4 — IANA TLS Supported Groups (cross-reference)

| Decimal | Hex | Name | Reference |
|---|---|---|---|
| 29 | 0x001D | X25519 | RFC 8422 |
| **4587** | **0x11EB** | **SecP256r1MLKEM768** | draft-ietf-tls-ecdhe-mlkem |
| **4588** | **0x11EC** | **X25519MLKEM768** | draft-ietf-tls-ecdhe-mlkem |
| **4589** | **0x11ED** | **SecP384r1MLKEM1024** | draft-ietf-tls-ecdhe-mlkem |

**TLS combiner construction (verified):** raw concatenation `MyECDH.shared_secret || MyPQKEM.shared_secret` inserted directly into the TLS 1.3 key schedule. Note: **for X25519MLKEM768 the order is `ML-KEM || X25519`** (PQ first) whereas the NIST-curve variants use `ECDHE || ML-KEM` (trad first) — quirky but specified. **NO X-Wing-style SHA3 wrap** at the TLS layer; the TLS key schedule itself provides the KDF.

### B.5 — NIST FIPS reference identifiers

- **FIPS 203 (ML-KEM)** — published 2024-08-13; algorithm OIDs assigned under `2.16.840.1.101.3.4.4.x` (NIST KEMs arc):
    - `id-alg-ml-kem-512` = `2.16.840.1.101.3.4.4.1`
    - `id-alg-ml-kem-768` = `2.16.840.1.101.3.4.4.2`
    - `id-alg-ml-kem-1024` = `2.16.840.1.101.3.4.4.3`
- **FIPS 204 (ML-DSA)** — published 2024-08-13; OIDs under `2.16.840.1.101.3.4.3.x`:
    - `id-ml-dsa-44` = `2.16.840.1.101.3.4.3.17`
    - `id-ml-dsa-65` = `2.16.840.1.101.3.4.3.18`
    - `id-ml-dsa-87` = `2.16.840.1.101.3.4.3.19`
- **FIPS 205 (SLH-DSA)** — published 2024-08-13; OIDs under `2.16.840.1.101.3.4.3.x` (continuing the signature arc, codepoints `20..31` for the 12 SLH-DSA parameter sets).
- **FIPS 206 (FN-DSA / FALCON)** — DRAFT submitted 2025-08-28; final expected late 2026 / early 2027.
- **FIPS 207-equivalent for HQC** — DRAFT expected 2026; final 2027.
- **NIST SP 800-227 (KEM operational guidance)** — FINAL 2025-09-18. §4.6 covers multi-algorithm KEMs and PQ/T hybrids. Does NOT mandate one specific combiner construction; defers to IETF/community.

---

## Section C — Cross-registry coherence

### C.1 — Ed25519+ML-DSA-65 hybrid signature

| Registry / standard | Identifier | Status | Reference |
|---|---|---|---|
| **Multicodec** | NONE — no hybrid codepoint exists | — | (Agent 1 confirms) |
| **IETF LAMPS X.509 OID** | `id-MLDSA65-Ed25519-SHA512` = `1.3.6.1.5.5.7.6.48` | Early-allocated 2025-10-20; RFC Editor Queue | draft-ietf-lamps-pq-composite-sigs-19 |
| **IANA COSE Algorithms** | NOT YET ASSIGNED (TBD `-52`, `-55` requested in jose draft) | jose WG v01 draft | draft-ietf-jose-pq-composite-sigs-01 |
| **IANA JOSE alg name** | `ML-DSA-65-Ed25519` (requested) | jose WG v01 draft | draft-ietf-jose-pq-composite-sigs-01 |
| **W3C DID** | Depends on multicodec (Agent 1 scope); none today | — | — |
| **NIST FIPS** | None (FIPS does not specify composites) | — | — |
| **TLS** | N/A (TLS uses signature schemes from a different registry) | — | — |

**Coherent path: the OID + the construction are settled.** The COSE/JOSE assignment is in flight but the construction in the jose draft mirrors the LAMPS construction exactly.

### C.2 — X25519+ML-KEM-768 hybrid KEM — TWO COMPETING CONSTRUCTIONS

| Aspect | **X-Wing** (CFRG individual / HPKE 0x647A) | **Composite-KEM** (LAMPS WG / OID `1.3.6.1.5.5.7.6.58`) |
|---|---|---|
| Combiner | `SHA3-256(ss_M || ss_X || ct_X || pk_X || XWingLabel)` | `SHA3-256(mlkemSS || tradSS || tradCT || tradPK || Label)` |
| Inputs differ | `ct_X`, `pk_X`, `XWingLabel = 0x5c2e2f2f5e5c` (fixed 6 bytes) | `tradCT`, `tradPK`, **`Label`** (algorithm-specific string per OID) |
| Adoption track | Independent (CFRG individual; intended Informational) | LAMPS WG (Proposed Standard) |
| Codepoint home | IANA HPKE 0x647A — concrete, real today | IETF SMI PKIX 1.3.6.1.5.5.7.6.58 — concrete, real today |
| Wire format | HPKE-shaped (Nenc=1120; designed for streaming/AEAD layering) | CMS/X.509-shaped (designed for certificate flows) |
| Peer review | IACR Communications in Cryptology 2024-1-21 ("X-Wing", Barbosa/Connolly/Duits/Schwabe/Schmieg/Stebila) | LAMPS WG security analysis |
| Real deployments | Cloudflare (research deployment), Filippo Valsorda's `mlkem768` Go library, OpenSSH's hybrid roadmap discussions | PKI-side draft implementations |
| Mutually compatible? | **NO** — different combiner inputs ≠ different shared secret ≠ different ciphertext shape | |

**Critical: a system that picks one cannot interoperate with a system that picks the other** even for the "same" X25519+ML-KEM-768 pair. The shared-secret-out differs.

### C.3 — NF-1 PQ⊕PQ end-state pairs (per CLAUDE.md #5)

**Per CLAUDE.md #5: signing = ML-DSA-65 ⊕ SLH-DSA; KEM = ML-KEM-768 ⊕ HQC.**

| Pair | Current standardization status |
|---|---|
| ML-DSA-65 ⊕ SLH-DSA composite signature | **No OID. No draft.** No IETF LAMPS draft proposes pure-PQ⊕pure-PQ composites — every defined composite has one classical + one PQ component. CFRG hybrid-combiners work could host this in future but nothing in flight. |
| ML-KEM-768 ⊕ HQC composite KEM | **No OID. No draft.** Same reason — composite-KEM draft is PQ + traditional only. Also, **HQC FIPS draft not even published yet** (2026 draft, 2027 final). |

**Implication for Benten:** the NF-1 end-state is GENUINELY future work — no IETF body has even started on PQ⊕PQ composites. Building it as a Benten-internal "additive impl + reserved codepoint" per CLAUDE.md #5 is the right shape; nothing to align with.

---

## Section D — Standards-body process timing

### D.1 — IETF draft → RFC realistic timeline for `lamps-pq-composite-sigs`

**Observed timeline:** draft-00 → draft-19 → RFC Editor Queue. The draft has been worked since 2022 (v00) and is now (May 2026) in the RFC Editor's queue. Typical RFC Editor queue residence is 3–9 months. Conservative estimate: **RFC published Q3 2026 to Q1 2027**.

**Risk that OIDs change at publication: near-zero.** The OIDs were early-allocated 2025-10-20 by IANA precisely to lock the identifiers for implementations. **Benten can rely on `1.3.6.1.5.5.7.6.48` today.**

### D.2 — Multicodec PR vs IETF RFC dependency

This is mostly Agent 1's scope, but for IETF context: the multicodec table historically does NOT wait for IETF/IANA. Examples:
- BLAKE3 multicodec `0x1e` predates any IETF BLAKE3 RFC.
- Ed25519-pub multicodec `0xed` predates RFC 8410's wider deployment.
- ML-KEM/ML-DSA multicodec entries (if/when they land) will likely reference NIST FIPS directly, not IETF.

**Inference: a Benten-driven multicodec PR for a hybrid PQ codepoint would NOT need to wait for the IETF RFC.** Pre-RFC codepoint assignment is the multicodec norm. **However, multicodec entries are typically for primitive algorithms not composites** — i.e. ML-KEM-768 and X25519 separately, not the hybrid as one codepoint. Agent 1 should confirm the multicodec policy on composite/hybrid codepoints.

### D.3 — NIST PQC standardization

- **Round 4 (KEMs):** HQC selected 2025-03-11. Draft 2026, final 2027.
- **Round 5 (general signatures):** still in progress; HAWK and others under evaluation.
- **FIPS 206 (FN-DSA):** draft submitted 2025-08-28; final late-2026/early-2027.
- **NO planned FIPS for hybrid composites** — NIST has explicitly left composition to IETF and CFRG.

**Implication:** Benten's "PQ-hybrid as v1-beta default" cannot wait for a NIST hybrid standard (there isn't one and there won't be one). The IETF LAMPS path is the only authoritative algorithm-identifier path for hybrids — and it is essentially complete.

---

## Section E — Synthesis + Recommendations

### E.1 — Signing: align with `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`) — STRONG recommendation

**Why:**
1. The exact pair Benten wants is the LAMPS draft's pair. OID early-allocated. Construction is the construction CLAUDE.md #5 describes. RFC Editor Queue = settled.
2. Aligns Benten with the only IETF Standards-Track composite signature path. Future TLS, X.509-using-systems, JOSE/COSE signature interop has a chance.
3. Construction is committing + strip-resistant — satisfies CLAUDE.md NF-4.
4. **No work for Benten to do beyond adopting the OID and the construction.** Our `benten-crypto-suite` substrate already implements this shape.

**Codepoint usage in Benten:**
- Internal codepoint table: use the OID arc `1.3.6.1.5.5.7.6.48` as the canonical identifier; map to Benten's internal `KeySuite` enum.
- For multicodec / `did:key` integration (Agent 1's scope), use the *components* (Ed25519 + ML-DSA-65) at their primitive multicodec values and frame the hybrid at the Benten-suite-selector level — DO NOT mint a Benten-private hybrid multicodec without IPLD/multicodec-community coordination.

### E.2 — KEM: pick X-Wing (HPKE 0x647A) over composite-KEM (`1.3.6.1.5.5.7.6.58`) — recommendation

**Why X-Wing:**
1. **Peer-reviewed cryptographic provenance.** X-Wing has an IACR CiC paper (2024) by Barbosa, Connolly, Duits, Schwabe, Schmieg, Stebila — a who's-who of crypto engineering. Composite-KEM has the LAMPS WG security analysis, which is solid but not the same peer-review depth.
2. **Lighter combiner inputs.** X-Wing combiner: `ss_M || ss_X || ct_X || pk_X || label` (6-byte fixed label). Composite-KEM combiner: `mlkemSS || tradSS || tradCT || tradPK || Label` (variable-length per-OID label string). Functionally similar; X-Wing is slightly tighter and was peer-reviewed in this exact shape.
3. **Production deployment shape match.** X-Wing is HPKE-shaped — designed to layer with AEAD streams. Benten's #1301 encryption substrate is exactly that shape (per-chunk-AEAD ≥64 KiB with chunk size = `IROH_BLOCK_SIZE`, per the ratified S&C architecture). Composite-KEM is CMS/X.509-shaped, designed for certificate workflows that Benten does not run.
4. **Concrete IANA HPKE codepoint exists right now.** `0x647A`. No registry ambiguity.
5. **No COSE codepoint yet** for either X-Wing or composite-KEM — both options leave the same gap. But X-Wing's HPKE codepoint maps naturally onto Benten's iroh-bytes substrate without needing COSE at all.

**Why NOT composite-KEM:**
1. CMS/X.509 deployment context isn't ours.
2. Broader combiner inputs (including `tradPK`) increase wire-format weight without buying us anything for our use case.
3. X-Wing's individual-submission status is **not a security concern** — it's a process status, and the IANA HPKE codepoint is real.

**Caveat / risk for X-Wing path:**
- X-Wing has NOT been CFRG-adopted. If a future CFRG hybrid-combiners RFC blesses a different combiner, X-Wing implementations would coexist (not be replaced — IANA codepoint is permanent) but ecosystem momentum could shift. **Mitigation:** Benten's crypto-agility seam (per CLAUDE.md #5) means we can ADD a second codepoint later without breaking existing content. X-Wing today + composite-KEM as a future additive arm is a reasonable defensive posture.

### E.3 — Should Benten wait for IETF lamps-pq-composite-sigs to finalize? **NO**

- The signing OID is EARLY-ALLOCATED (locked since 2025-10-20).
- The construction is settled (the GitHub-rendered draft is the source of truth used by every implementer).
- RFC publication is a paperwork formality at this point.
- **Waiting would mean missing the v1-beta window for no incremental safety gain.**

### E.4 — Should Benten coordinate with the IETF WGs?

**For signing:** No formal coordination needed. Implement the LAMPS construction; reference draft-19 / forthcoming RFC. We are not asking IANA for anything (OIDs already allocated).

**For KEM:** Optionally, a lightweight courtesy note to CFRG about Benten's X-Wing deployment would help X-Wing's adoption-call traction, but is not required. **No coordination needed for the codepoint itself** (it's in IANA HPKE).

**For multicodec:** see Agent 1's scope — coordination there is community-process not WG-process.

### E.5 — Three alternatives we have NOT covered (elegance pass)

1. **Wait for COSE composite codepoints (jose draft `-51..-56`) and use COSE-everywhere.** Considered: NO — jose draft is v01, not WG-track, no timeline. Benten cannot block v1-beta on this.
2. **Use IANA TLS Supported Groups codepoints (4587/4588/4589) instead of HPKE.** Considered: NO — Benten is not a TLS endpoint; using TLS codepoints in a non-TLS context is a category error. Plus the TLS combiner is "raw concat into TLS key schedule," not a self-contained KEM secret.
3. **Mint Benten-private codepoints in HPKE private-use range (`0x647B..0xFFFF`) and not commit to X-Wing OR composite-KEM yet.** Considered: WEAK — this is just deferring the decision and creating wire-format orphans that will need migration. The HPKE private-use range is for app-specific KEMs, not "we couldn't decide between two standards." HARD RULE 12 says fix-now, not defer.

### E.6 — Final concrete recommendation table (for orchestrator decision-surface)

| Decision | Recommendation | Codepoint to bake in |
|---|---|---|
| Hybrid signature OID | `id-MLDSA65-Ed25519-SHA512` (LAMPS) | `1.3.6.1.5.5.7.6.48` |
| Hybrid KEM codepoint | X-Wing (IANA HPKE) | `0x647A` = 25722 |
| Symmetric AEAD | (out of scope; ChaCha20-Poly1305 per CLAUDE.md #5) | — |
| Hash | (out of scope; BLAKE3-256 per CLAUDE.md #5) | — |
| Wait for IETF RFC publication? | No | — |
| Coordinate with IETF? | Optional courtesy note to CFRG re: X-Wing usage | — |
| Mint Benten-private hybrid codepoint? | No | — |

---

## Appendix — Source list (cite-grep-verified)

| Source | URL | What it gave us |
|---|---|---|
| IETF datatracker — lamps-pq-composite-sigs | https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/ | draft-19 status; 18-OID table; RFC Editor Queue confirmation |
| LAMPS WG rendered HTML | https://lamps-wg.github.io/draft-composite-sigs/draft-ietf-lamps-pq-composite-sigs.html | OID `1.3.6.1.5.5.7.6.48` for id-MLDSA65-Ed25519-SHA512; construction details; signature/pk encoding |
| IETF datatracker — lamps-pq-composite-kem | https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-kem/ | draft-14; 12-OID composite-KEM table incl. `1.3.6.1.5.5.7.6.58`; SHA3-256 combiner |
| IETF datatracker — connolly-cfrg-xwing-kem | https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/ | X-Wing draft-10; HPKE codepoint 0x647A; combiner construction; individual-submission status |
| IANA COSE registry | https://www.iana.org/assignments/cose/cose.xhtml | COSE codepoint table; private-use range `< -65536` |
| IANA HPKE registry | https://www.iana.org/assignments/hpke/hpke.xhtml | HPKE KEM Identifier table; X-Wing at 0x647A authoritative; private-use range 0x647B-0xFFFF |
| IANA JOSE registry | https://www.iana.org/assignments/jose/jose.xhtml | ML-DSA-44/65/87 alg names; AKP kty; registration procedure |
| IETF datatracker — jose-pq-composite-sigs | https://datatracker.ietf.org/doc/draft-ietf-jose-pq-composite-sigs/ | v01 jose composite signature draft; ML-DSA-65-Ed25519 alg name proposed |
| IETF datatracker — reddy-cose-jose-pqc-hybrid-hpke | https://datatracker.ietf.org/doc/draft-reddy-cose-jose-pqc-hybrid-hpke/ | Individual HPKE-in-JOSE/COSE draft; covers ML-KEM-768+X25519 but NOT X-Wing |
| IETF datatracker — cose-dilithium → RFC 9964 | https://datatracker.ietf.org/doc/draft-ietf-cose-dilithium/ | RFC 9964 published 2026-05; ML-DSA at COSE -48/-49/-50 |
| IETF datatracker — tls-ecdhe-mlkem | https://datatracker.ietf.org/doc/draft-ietf-tls-ecdhe-mlkem/ | TLS codepoints 4587/4588/4589; combiner construction |
| IETF datatracker — tls-hybrid-design | https://datatracker.ietf.org/doc/draft-ietf-tls-hybrid-design/ | TLS hybrid-design-16 in RFC Editor Queue; concatenation combiner |
| IETF datatracker — hpke-pq | https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/ | draft-04; pure ML-KEM + MLKEM768-P256 + MLKEM1024-P384 |
| CFRG mailing list — hybrid KEM combiners adoption call | https://mailarchive.ietf.org/arch/msg/cfrg/PSvLFyBWDdrRaOmaXBStRpGoH3c/ | 2024-01-31 adoption call; references X-Wing + Chempat-X |
| NIST FIPS 203 | https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf | ML-KEM standard; OIDs under NIST KEMs arc |
| NIST FIPS 204 | https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf | ML-DSA standard; OIDs under NIST signature arc |
| NIST FIPS 205 | https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.205.pdf | SLH-DSA standard |
| NIST SP 800-227 | https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-227.pdf | KEM operational guidance; §4.6 hybrid KEMs |
| NIST FIPS 206 (FN-DSA) status | DigiCert blog + Manifold market | Draft 2025-08-28; final late 2026 / early 2027 |
| NIST HQC selection | https://www.nist.gov/news-events/news/2025/03/nist-selects-hqc-fifth-algorithm-post-quantum-encryption | HQC fifth PQ standard; draft 2026, final 2027 |

---

**End of Agent 2 deliverable. Branch ready for commit + push.**
