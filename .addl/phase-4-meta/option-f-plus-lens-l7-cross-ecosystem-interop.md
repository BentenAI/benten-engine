# Option F+ envelope-layer-unification §6.2-with-Amendments — L7 CROSS-ECOSYSTEM-INTEROP lens review

**Branch:** `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop`
**Reviewer lens:** Senior standards-interop analyst — how does Benten's envelope look to consumers OUTSIDE Benten?
**Date:** 2026-05-27
**Inputs reviewed:**
- `origin/phase-4-meta-core/option-f-plus-pseudo-keypair-review` `@ 6d4e173f` — first cryptographer (NO-GO on F+; introduced §6.2 envelope-layer unification)
- `origin/phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review` `@ 7e900a3b` — second-opinion + Amendments 1+2 (codepoint-in-AAD; strict-decode)
- `origin/phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design` `@ 13b624c3` — adversarial red-team + Amendments 3-8 (length-injectivity; sender-DID-in-AAD; replay-window; Decap CT-mitigation; endianness pin; codepoint-registry-governance)
- `.addl/dispatch-conventions.md` §3.5s at `2768b583` (cross-ecosystem-identifier-as-content discipline)
- `docs/SECURITY-POSTURE.md` Compromise #30 (PQ unaudited; closes at v1-GM) + Compromise #31 (LAMPS Composite ML-DSA EUF-CMA-only; closed-equivalent at app layer via Inv-15)
- Primary standards: RFC 9180 (HPKE); RFC 9052 (COSE); RFC 7515/7516/7517/7518 (JOSE/JWE/JWK/JWA); RFC 9106 (Argon2); `draft-skokan-jose-hpke-pq-pqt-02`; `draft-ietf-jose-pqc-kem-05`; `draft-reddy-cose-jose-pqc-hybrid-hpke-11`; `draft-ietf-jose-hpke-encrypt-11/15`; `draft-ietf-mls-pq-ciphersuites-04`; `draft-irtf-cfrg-concrete-hybrid-kems-03`; `draft-connolly-cfrg-xwing-kem-10`
- Ecosystem specs: C2SP age v1 (`age-encryption.org/v1`); Saltpack v2 binary encryption (saltpack.org/encryption-format-v2); ATProto cryptography (`atproto.com/specs/cryptography`); multicodec table.csv at `multiformats/multicodec @ master`

---

## §1 Executive verdict + confidence

**The §6.2-with-Amendments-1-through-6 envelope as currently specified is INTERNALLY SOUND but is NOT ECOSYSTEM-INTEROP-COMPLIANT under §3.5s.** It is a perfectly serviceable **Benten-internal wire format**. It is **not** a JOSE/COSE/age/multicodec consumer-readable envelope, and the current spec text does not name how it crosses ecosystem boundaries.

Confidence on the verdict: **HIGH**. Confidence on the specific recommended amendments (§7): **MEDIUM-HIGH** — the *direction* (carry cross-ecosystem identifiers as content at each boundary; keep `codepoint: u16` as the internal dispatch number) is settled by §3.5s; the *specific* shape of the boundary-adapter envelopes (e.g. exact JWE-JSON-serialization, exact age-stanza name) is a design call with a few viable options.

**Single most load-bearing finding.** The brief itself flags it: *"the `codepoint: u16` is a Benten-internal dispatch number, NOT a JOSE/COSE/multicodec/IANA identifier."* §6.2-with-Amendments codifies an *internal* envelope but says nothing about what happens **when this envelope crosses an ecosystem boundary**. Under §3.5s, every ecosystem-boundary surface MUST carry the cross-ecosystem identifier as content. Today, no §6.2 amendment names this for **any** of the 9 ecosystems in the brief's matrix. That is a §3.5s compliance gap, not a soundness gap, but it is a real one and it is fixable by adding **Amendment 9 (cross-ecosystem-identifier-as-content)** before v1-beta wire-freeze.

**Honest framing of the gap's stakes.** The three prior reviewers correctly bounded their scope to "is this envelope internally sound?" None of them was asked the question "is this envelope what an outsider sees?" The omission is structural to lens-composition, not a flaw in any reviewer's work. This review fills the gap.

**Other major findings** (each detailed in §2–§5):

1. **No native ecosystem can parse §6.2 bytes today.** Not JOSE, not COSE, not age, not Saltpack, not multicodec, not ATProto, not IPLD, not OpenPGP, not HPKE-vanilla. Every one of the 9 ecosystems requires a **Benten-side adapter** that re-emits the envelope into the ecosystem's native shape carrying the ecosystem's native identifier as content. (See §2 matrix.)
2. **The `codepoint: u16` field is correctly named per §3.5s** (it's an internal dispatch number, not a JOSE alg name) **but the design must also commit that this codepoint is NEVER serialized to ecosystem-boundary outputs in lieu of the cross-ecosystem identifier.** Add an explicit clause to Amendment 1 / Amendment 2.
3. **Wire-format-recognizability is currently ZERO.** No magic bytes, no version prefix, no multicodec wrapping. An outsider with the raw bytes cannot even identify this as "an encrypted envelope" — let alone "HPKE-PQ." Recommend a 4-byte magic + 1-byte version prefix at the framing layer. (§4)
4. **The JOSE-ecosystem-aligned name for Benten's hybrid HPKE construction is `HPKE-11` (integrated) / `HPKE-11-KE` (key encryption)** per `draft-skokan-jose-hpke-pq-pqt-02` and `draft-reddy-cose-jose-pqc-hybrid-hpke-11`. The brief's strawman string `HPKE-Base-X25519-MLKEM768-ChaCha20Poly1305` is NOT a registered JOSE name; the registered name is the integer-suffix form `HPKE-11`. This is a §3.5q (IETF-vocabulary-precision) corollary. (§3)
5. **The multicodec ecosystem path is via `mlkem-768-pub = 0x120c` + `jwk_jcs-pub = 0xeb51`, NOT `cose-key 0x42` or `jwk 0x44`.** The brief's "PRs #400/#403" citation appears stale or anticipatory; the live multicodec table at HEAD contains the `jwk_jcs-*` and `mlkem-*` entries (all in draft status) but NO `cose-key` entry. Public-key surfaces should carry `mlkem-768-pub` + matching classical-pair codepoint; envelope-content surfaces have no current multicodec container codepoint and would need a Benten-private wrapper unless a future codepoint mint lands. (§3)
6. **Forward-additivity for MLS-PQ + OpenPGP-PQC is structurally fine** — §6.2's codepoint-dispatch shape absorbs new codepoints additively. The Amendment 9 cross-ecosystem-identifier-as-content discipline absorbs future ecosystem identifiers additively too. No structural blockers; only a discipline of adding adapter-emitters per ecosystem as they mature. (§5)
7. **Position B blog §4 should NOT name Benten's primitive as "X-Wing"** in cross-ecosystem-facing prose — use `MLKEM768-X25519` (CFRG-research-group-adopted name per `draft-irtf-cfrg-concrete-hybrid-kems-03`) for the KEM and `HPKE-11` (JOSE-registered identifier) for the full construction when explaining to JOSE-stack readers. (§6)

---

## §2 Per-ecosystem interop assessment matrix

For each ecosystem: can a non-Benten consumer **parse** (read bytes into structured form) / **verify** (validate auth/MAC/sig) / **re-emit** (produce a byte-compatible output) Benten's §6.2 envelope? And what wrapper bridges the gap?

| Ecosystem | Parse? | Verify? | Re-emit? | What's missing | Wrapper / converter to bridge |
|---|---|---|---|---|---|
| **HPKE-RFC-9180 vanilla** (any consumer reading HPKE bytes) | **NO** | NO | NO | RFC 9180 §10 explicitly **does NOT prescribe a wire format**; "applications that adopt HPKE must therefore specify an unambiguous encoding mechanism." There is no "HPKE wire format" to interop against. Each application defines its own (TLS-ECH does, MLS does, JOSE-HPKE does, COSE-HPKE does — all different). | N/A. There is no vanilla HPKE consumer to interop with. Benten interops with HPKE *transitively* via JOSE-HPKE or COSE-HPKE. |
| **JOSE / JWE / JWK** (JS WebCrypto, panva/jose, jose-rs, ory/hydra) | **NO** as §6.2 bytes; **YES** via Benten-side JWE adapter | NO directly; YES via adapter | NO directly; YES via adapter | §6.2 emits Benten-internal binary. JOSE consumers expect a JWE Compact Serialization (`header.encrypted_key.iv.ciphertext.tag`) or JWE JSON Serialization with `protected` header containing `"alg": "HPKE-11-KE"` + `"enc": "..."` + `"ek": base64url(enc)` per `draft-ietf-jose-hpke-encrypt-11` §4. The Benten `codepoint = 0x647A` is invisible/wrong-shape. | **Adapter: `benten-jose-emit`** crate. Given an internal `EncryptedEnvelope { codepoint, payload: HpkeBase{..}, aad_binding }`, emit a JWE JSON Serialization with `protected: {"alg": "HPKE-11-KE", "enc": "A256GCM", "ek": "<base64url(enc)>"}` + `ciphertext: "<base64url>"` + `tag: "<base64url>"`. The reverse direction (JWE → Benten) is also straightforward. Benten's codepoint `0x647A` maps deterministically to `HPKE-11-KE`. |
| **COSE** (rustler/cose, IoT stack, COSE_Encrypt tag 96) | **NO** as §6.2 bytes; YES via adapter | NO directly; YES via adapter | NO directly; YES via adapter | COSE_Encrypt is CBOR-tagged (tag 96) with a `protected` map containing `1 = alg` (e.g. `TBD13` for HPKE-11-KE per reddy-cose-jose-pqc-hybrid-hpke), `unprotected` map, and `ciphertext` bstr. Benten's binary is not CBOR-tagged + has no COSE alg integer. | **Adapter: `benten-cose-emit`** crate. Encode `protected = {1: TBD13_or_assigned_integer}` + `unprotected` (kid + ek-encapsulated-key) + `ciphertext` + COSE_recipient[] per RFC 9052 §5. COSE integer alg-IDs for HPKE-MLKEM768-X25519 are TBD (placeholders TBD12/TBD13 in the draft); MUST track IANA assignment + mirror in Benten's codepoint→alg-name table. |
| **age** (age-rs, rage) | **NO** as §6.2 bytes; YES via adapter producing age stanza | NO directly; HMAC-SHA256 header MAC needs re-issuance under age key | NO directly; YES via adapter | age v1 uses ASCII `age-encryption.org/v1` magic line + recipient stanzas `-> <TYPE> <ARGS...>` + base64-no-padding body wrapped at 64 cols + final `--- <base64-HMAC>` MAC line. Benten's binary is not even ASCII-armored. age v1 *already* defines `mlkem768x25519` recipient stanza per C2SP age.md — Benten could in principle hit binary-compat with age if it emitted an age file as the cross-ecosystem-emit form. | **Adapter: `benten-age-emit`** crate. For Layer-C drop to a recipient, emit `age-encryption.org/v1\n-> mlkem768x25519 <base64-enc>\n<base64-body>\n--- <base64-HMAC>`. The age-MLKEM768X25519 KEM-share is the same `enc` Benten already produces (with potential format-encoding differences — needs cross-validation against age-rs test vectors). |
| **Saltpack** (Keybase, kbnm) | **NO** as §6.2 bytes; YES via adapter | NO directly; YES via adapter | NO directly; YES via adapter | Saltpack v2 binary is MessagePack-encoded with mode = 0 for encryption + NaCl public-key authenticated encryption (X25519-Salsa20-Poly1305). It has **no MLKEM768 hybrid mode and no HPKE mode** at this writing — Saltpack remains classical-only. Benten cannot emit a Saltpack-compatible v2 message that carries Benten's PQ-hybrid security level. | **Adapter: `benten-saltpack-emit`** crate would have to **downgrade** to classical-only X25519 to interop with current Saltpack readers — which **violates Benten's PQ-default reframe per Compromise #30**. RECOMMEND: do not implement Saltpack-emit before Saltpack ships a PQ-hybrid mode. Document as "Saltpack interop deferred pending Saltpack-PQ" in V1-FROZEN-INTERFACE-DEFERRED.md. |
| **multicodec** (libp2p key import; did:key consumers) | **PARTIAL** — public-key bytes parse via `mlkem-768-pub = 0x120c` (draft) | N/A (multicodec is key-codec, not envelope-codec) | **PARTIAL** — for the public-key side; NOT for envelope-content | The multicodec table contains **no envelope/content codepoint** for HPKE / encrypted-content / COSE_Encrypt-wrapped at HEAD. It contains: `mlkem-768-pub = 0x120c` (draft), `mldsa-65-pub = 0x1211` (draft), `jwk_jcs-pub = 0xeb51` (draft), `jwk_jcs-priv = 0x1316` (draft). The brief's claim of `cose-key 0x42` and `jwk 0x44` does **not** match the live table; no such rows exist. (Possibly stale citation; possibly anticipating future PRs that didn't land.) | **Adapter for public-key surfaces only.** Benten's hybrid public key (X25519-half + MLKEM768-half) can be carried as a multicodec-prefixed bytestring using `mlkem-768-pub = 0x120c` for the PQ half. The classical X25519 half uses existing `x25519-pub = 0xec` (existing in table). Benten would emit `concat(varint(0x120c) || mlkem768_pub_bytes, varint(0xec) || x25519_pub_bytes)` or a Benten-private multicodec for the combined hybrid. **No multicodec envelope codepoint exists** so the envelope itself stays Benten-private until a future codepoint mints. |
| **ATProto** (Bluesky / PLC) | **NO** as §6.2 bytes; **N/A** — ATProto does not support encryption | NO (signature only); ATProto signs DAG-CBOR with P-256 or k256 + ECDSA-low-S | NO | ATProto `cryptography` spec explicitly says *"data encryption is not used directly in the protocol, though many components use secure network transport based on TLS."* ATProto's did:key multikey carries P-256 / k256 only — not MLKEM768. **There is no ATProto encrypted-envelope to interop against.** Bluesky discussion #121 ("Encryption for private content") remains open / unresolved as of 2026. | **Not applicable** for envelopes. For *signed-content* interop with ATProto, Benten's Inv-15 3-layer decomposition + LAMPS-OID Composite ML-DSA path already addresses the signature surface. Drop encryption is a Benten-private feature with no ATProto counterpart to bridge to. |
| **IPLD / IPFS** (encrypted-content blocks; CAR encoding) | **PARTIAL** — IPLD/IPFS treats encrypted blocks as opaque bytestrings addressable by CID | N/A (no auth/sig at IPLD layer for ciphertext) | YES — Benten envelopes can be CID-addressed and stored in IPLD-block-style if Benten chooses | IPLD is content-addressed; encrypted bytestrings can be stored as raw blocks (codec `0x55 = raw`) with their CIDs. The bytestring's internal structure is opaque to IPLD. | **Adapter: trivial** — wrap Benten's `EncryptedEnvelope` serialized bytes as IPLD raw-block at content-codec `0x55`. CID is `multihash(serialized_envelope)`. **This is the natural Benten path forward for at-rest storage** and aligns with Compromise #31's existing CID-as-payload pattern. Benten already uses BLAKE3 CIDs (Inv-5 / NF-4); IPLD interop is free once raw-codec is named. |
| **OpenPGP / sequoia / GnuPG** | **NO** as §6.2 bytes | NO | NO | OpenPGP-PQC stabilization is still in flight (`crypto-refresh` LWG); current GnuPG / sequoia / RFC 9580 do **not** speak MLKEM768. OpenPGP packet format is fundamentally different (variable-length packets, packet-tag headers, no codepoint-as-discriminator dispatch). | **No bridge currently.** Document as "OpenPGP-PQC interop deferred pending OpenPGP-PQC stabilization" in V1-FROZEN-INTERFACE-DEFERRED.md. Benten's vault use case (encrypt-to-self) has no OpenPGP counterpart anyway (OpenPGP is encrypt-to-recipient by design); only Layer-C / Layer-D would conceivably bridge, and not until PQC support lands upstream. |

**Summary of the matrix.** Across the 9 ecosystems: **0 of 9 can parse §6.2's raw bytes directly.** All 9 require Benten-side adapter code to bridge — **which is fine and expected; this is exactly what §3.5s describes**. The §6.2 envelope is a Benten-internal wire format; ecosystem interop happens at adapter boundaries. The discipline §3.5s codifies is: at each adapter boundary, the cross-ecosystem identifier (LAMPS OID / JOSE alg-name / multicodec codepoint / age stanza name) MUST appear as content in the adapter output.

**The §6.2 design therefore does not need to BE a JOSE/COSE/age envelope.** It needs to be **convertible to** each of them at adapter boundaries, with the right cross-ecosystem identifier emitted as content at each boundary. That convertibility is a property of having clean `codepoint → ecosystem-identifier` mapping tables — which §6.2 currently does **not** include.

---

## §3 §3.5s compliance evaluation per ecosystem boundary

§3.5s requires: "Wire-format envelopes that cross ecosystem boundaries... MUST carry the cross-ecosystem identifier AS CONTENT inside the envelope where consumers of that ecosystem will recognize it. The Benten-internal `SigCodepoint` / `CipherSuiteCodepoint` / etc. dispatch numbers stay hot-path-dispatch-only and MUST NEVER appear at ecosystem-boundary surfaces in lieu of the cross-ecosystem identifier."

Evaluation against the four boundary cases the brief enumerates:

### 3.1 JOSE / JWE consumer boundary

**Question:** When the envelope is emitted to a JOSE/JWE consumer, does it carry alg-name `HPKE-Base-X25519-MLKEM768-ChaCha20Poly1305` per JOSE registry?

**Verdict:** **No, and the brief's strawman alg-string is also wrong.**

- §6.2 + Amendments specify a Benten-internal binary; no JWE emission path is defined.
- The correct JOSE alg-name per `draft-skokan-jose-hpke-pq-pqt-02` + `draft-reddy-cose-jose-pqc-hybrid-hpke-11` is **`HPKE-11`** (integrated encryption) or **`HPKE-11-KE`** (key encryption mode) — *not* the descriptive string in the brief.
- The brief's strawman `HPKE-Base-X25519-MLKEM768-ChaCha20Poly1305` is a **description, not a registered identifier**. JOSE algorithm registries use short opaque strings. The integer-suffix form (`HPKE-11`) IS the JOSE-WG-adopted shape.
- For Benten's pattern (`CipherSuiteCodepoint::HYBRID_X25519_MLKEM768 = 0x647A` + ChaCha20-Poly1305 bulk), the mapping is **`0x647A → HPKE-11-KE`** in key-encryption mode (since Benten uses HPKE to wrap the per-Node K, then ChaCha20-Poly1305 bulk-encrypts).

**§3.5s compliance gap:** §6.2's `codepoint: u16` field gets serialized into Benten-internal envelopes; when those envelopes cross into a JWE consumer's view, the alg-name in the JWE `protected` header MUST be `HPKE-11-KE`, NOT `0x647A`. **§6.2-with-Amendments-1-6 contains no clause forbidding the leak of `0x647A` to JWE protected headers; it must.**

### 3.2 multicodec consumer boundary

**Question:** When emitted to a multicodec consumer, does it use container-form `cose-key 0x42` or `jwk 0x44` per multicodec PR #400/#403?

**Verdict:** **The brief's premise is partly incorrect; the actual multicodec table state is different and narrower.**

- The live multicodec table at `master` HEAD does **not** contain `cose-key = 0x42`. It does contain `jwk_jcs-pub = 0xeb51` (draft) and `jwk_jcs-priv = 0x1316` (draft). The `0x42` and `0x44` cited in the brief are not in the table at HEAD; either the cited PRs (#400/#403) have not merged, or the codepoints in the cited PRs differ from what's at HEAD, or the brief's citation is anticipatory.
- For Benten's **public-key emission** to multicodec consumers (e.g., libp2p key import), the right shape is:
  - PQ half: `mlkem-768-pub = 0x120c` (draft) — varint-prefixed bytestring.
  - Classical half: `x25519-pub = 0xec` (stable).
  - For the **combined hybrid** public key: no multicodec codepoint exists; Benten can either (a) emit a `jwk_jcs-pub` (0xeb51) container holding a JWK with the combined key as a JSON structure, or (b) wait for / propose an MLKEM768-X25519 multicodec PR.
- For **envelope emission** (the §6.2 envelope itself going to a multicodec consumer): **no multicodec envelope/content-codec exists** for HPKE-encrypted bytes. The only path is wrap-as-raw (`raw = 0x55`) + treat the Benten envelope bytes as IPLD-opaque content addressable by CID. This is IPLD interop, not multicodec-envelope-interop, and is fine.

**§3.5s compliance posture:** for public-key surfaces (DID document `verificationMethod`s, JWKS endpoints, libp2p PeerID material), Benten MUST emit the multicodec-prefixed bytestring using the right varint codepoint per the table at HEAD (`0x120c` for ML-KEM-768 public key; `0xec` for X25519; future codepoint for the combined hybrid). The internal `CipherSuiteCodepoint = 0x647A` MUST NOT appear in DID-document or JWKS output. **§6.2-with-Amendments contains no clause specifying multicodec emission for public-key surfaces; this is currently outside §6.2's scope but is a §3.5s-relevant gap that needs to be named somewhere in the F-full design.**

### 3.3 did:jwk URI boundary

**Question:** When emitted in a did:jwk URI, does it use JWK alg-name not Benten-codepoint?

**Verdict:** **Yes, MUST. RFC 7517 §4.4 makes `alg` optional but case-sensitive ASCII; if Benten emits an `alg` field in a did:jwk-embedded JWK, that field MUST be a JOSE-registered name (e.g. `HPKE-11-KE` for the hybrid encryption key, or `MLDSA65-Ed25519` for the hybrid signature key per `draft-ietf-lamps-pq-composite-sigs-19` + Compromise #31's existing pattern).**

The `kty` field on a JWK is also relevant: for ML-KEM-768 keys, the JWK `kty` value would track `draft-ietf-jose-pqc-kem-05` (which defines `kty = AKP` or similar; the draft is still in flux on the exact kty value).

**§3.5s compliance posture:** Benten's did:jwk emission path MUST produce a JWK with `alg = "<JOSE-registered-name>"` and `kty = "<JOSE-registered-key-type>"`, never `alg = "0x647A"` or any Benten-internal codepoint shape. **§6.2-with-Amendments contains no did:jwk emission clause; this is outside §6.2's scope but is a §3.5s gap to be named in the F-full DID-resolver-emit-path design.**

### 3.4 LAMPS / X.509 / CMS consumer boundary

**Question:** When emitted to a LAMPS / X.509 / CMS consumer, does it use LAMPS OID where applicable?

**Verdict:** **For signature surfaces — YES, already.** Per Compromise #31 + existing Inv-15 design, the LAMPS OID `1.3.6.1.5.5.7.6.48` (`id-MLDSA65-Ed25519-SHA512`) is the cross-ecosystem identifier already used at LAMPS-boundary surfaces, and the internal `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` stays internal. This is the original instance of the §3.5s pattern.

**For encryption surfaces — N/A currently.** LAMPS doesn't yet define an OID for HPKE-MLKEM768-X25519-ChaCha20Poly1305 (LAMPS focuses on signature + KEM key-establishment for PKIX/CMS, not full HPKE envelope encryption). CMS-PQ envelope identifiers exist for raw ML-KEM-768 (`id-alg-ml-kem-768`) but not for the HPKE composition. **No LAMPS-side boundary applies to Benten's §6.2 encrypt envelope today.** When/if LAMPS mints an HPKE-MLKEM768 OID, Benten adds a `codepoint → OID` row to the mapping table. Forward-additive; no current gap.

### 3.5 §3.5s compliance summary

| Boundary | Has Benten emitted to this boundary today? | If emitted, does §6.2-with-Amendments comply with §3.5s? | Gap to close |
|---|---|---|---|
| JOSE/JWE/JWK | Not yet (no current emit path) | NO — `0x647A` would leak unless adapter rewrites | Add Amendment 9: codepoint→alg-name mapping table + emit-discipline clause |
| COSE | Not yet | NO — same as JOSE | Same Amendment 9 with COSE integer alg-IDs |
| multicodec (public-key surface) | YES, for did:key / libp2p PeerIDs (existing benten-crypto-suite path) | PARTIAL — internal codepoint not currently leaked to public-key surfaces, but no explicit clause forbids it | Add explicit clause in Amendment 9 or §3.5s code-anchor list |
| multicodec (envelope surface) | No envelope codec exists | N/A | No current ecosystem identifier to carry; IPLD-raw-block interop is sufficient |
| did:jwk | Not yet | NO if emit path were added today | Add Amendment 9 |
| LAMPS / X.509 / CMS (signature surface) | YES — via Compromise #31 + Inv-15 + existing LAMPS-OID emit | YES — already compliant; this is the §3.5s exemplar | None |
| LAMPS / X.509 / CMS (encryption surface) | N/A; no LAMPS-HPKE OID exists | N/A | Forward-additive when LAMPS mints |
| age v1 | Not yet | NO — would need age `mlkem768x25519` stanza emission | Add age-emit adapter (Amendment 9 + per-ecosystem adapter spec) |
| Saltpack | Not yet | N/A — Saltpack has no PQ mode | Document as deferred until Saltpack-PQ |
| ATProto | N/A — ATProto has no encryption envelope | N/A | None for encryption; signature side already aligned via Inv-15 |
| IPLD | Trivially yes (raw block) | YES — raw bytes are opaque to IPLD; no identifier leak | None |

**Overall §3.5s compliance posture:** **PARTIAL.** The signature-side path (LAMPS OID + Inv-15) is the §3.5s exemplar and is already compliant. The encryption-side path (§6.2 envelope) has **no current §3.5s clause** — the design is silent on what happens at ecosystem boundaries. This is a fillable gap, not a structural defect, and is the load-bearing recommendation in §7.

---

## §4 Wire-format-recognizability heuristics

**Does an outsider with the bytes alone recognize this as "an encrypted envelope using HPKE-PQ"? Or does it look like noise?**

Heuristics outsiders use to recognize crypto wire formats:

1. **Magic byte prefix.** PGP packets start with `0xC4` or `0x84` (packet-tag high bits); age files start with `age-encryption.org/v1\n`; Saltpack messages start with a MessagePack array prefix `0x94`/`0x95` + version tuple; CBOR objects start with a CBOR major-type byte; ASN.1/DER starts with `0x30 0x82` (SEQUENCE) or `0x30 0x81` (short-form SEQUENCE).
2. **Version prefix.** Most modern wire formats carry an explicit version (age `v1`; Saltpack version tuple `[2, 0]`; TLS record-layer version).
3. **Container/codec tag.** Multicodec varint-prefixed bytestrings; IPLD CIDs encode the content codec.
4. **Self-describing CBOR.** COSE messages are CBOR-tagged (tag 96 for COSE_Encrypt, tag 97 for COSE_Encrypt0, etc.); a CBOR-aware reader recognizes them as "this is a COSE_Encrypt structure" without decoding.

### 4.1 What §6.2-with-Amendments-1-6 looks like to an outsider

From the design as stated:

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,                       // BIG-ENDIAN on-wire (Amendment 7)
    payload: EnvelopePayload,             // codepoint-discriminated
    aad_binding: BindingContext,
}
```

The on-wire bytes (assuming a deterministic serialization per Inv-1 + canonical-tagged-length-value per Amendment 3) start with:

- Bytes 0-1: `codepoint` as big-endian u16 (e.g., `0x6101` for Layer-A vault, `0x647A` for Layer-C drop's HPKE codepoint, etc.).
- Bytes 2-N: payload + AAD-binding canonical bytes.

**Recognizability assessment:**

- **No magic byte prefix.** An outsider's first 2 bytes are `0x6101` or `0x647A` — these are not registered codec tags in any major registry (multicodec, IANA, CBOR). They look like arbitrary 16-bit numbers.
- **No version prefix.** No way to know "this is Benten envelope v1" vs "v2." Future versioning requires renegotiation of the codepoint space, which is feasible but fragile.
- **No CBOR/DAG-CBOR wrapping.** §6.2 doesn't specify the serialization format. If it's ad-hoc binary, outsiders cannot use CBOR-aware tools. If it's DAG-CBOR, the codepoint should be encoded as a CBOR field with a known key, and the outer structure should be CBOR-tagged.
- **No multicodec wrapping at the envelope level** (and there's no envelope multicodec codepoint to use).

**Verdict:** **§6.2 bytes look like noise to outsiders.** An age-rs tool, a panva/jose parser, a sequoia/PGP tool, a libp2p multicodec reader, a COSE library — none of them recognize this as "a thing." They get random-looking 16-bit codes + opaque body bytes.

### 4.2 Recommendation: add framing-layer recognizability

Add at the absolute outermost framing layer (before the §6.2 envelope body):

```
[ 4-byte ASCII magic: "BENT" ]
[ 1-byte format-version: 0x01 ]
[ 2-byte codepoint: big-endian (§6.2's existing field, now at byte offset 5) ]
[ canonical-tagged-length-value EnvelopePayload + BindingContext bytes ]
```

Justification for `"BENT"`:
- 4-byte ASCII is the standard for self-identifying binary formats (`%PDF`, `‰PNG`, `RIFF`, `MThd` for MIDI). Outsiders' file-type-detector tools (libmagic, file(1)) can be taught to recognize Benten envelopes.
- The format-version byte allows clean v1→v2 upgrade without overloading the codepoint space.
- The `codepoint` field stays at the §6.2-Amendment-1-defined position relative to the AAD-binding (Amendment 1 still mandates codepoint-in-AAD; the AAD content includes only the 2-byte codepoint + canonical-tagged-binding-context, NOT the 4-byte magic or 1-byte version — otherwise framing-layer changes would invalidate AAD binding).

Alternative: wrap the entire §6.2 envelope as **DAG-CBOR** with a CBOR-tag at the outer layer. This gets you CBOR-aware-tool recognition for free + interop with COSE-shaped consumers + content-addressability via Benten's BLAKE3 CID. Cost: ~5% size overhead + DAG-CBOR canonicalization discipline (already required by Inv-1).

**Trade-off summary.** Both options are viable. The ASCII-magic approach is lighter-weight and matches a "Benten-private wire format that doesn't pretend to be COSE" framing. The DAG-CBOR approach pays a slight cost for free integration with the broader CBOR/COSE/IPLD ecosystem. Recommend DAG-CBOR if Benten ever expects to expose envelope contents through IPLD-aware infrastructure (e.g., car-file backups, IPFS-style peer-mesh sync, which are already foreshadowed in baked-in #17/#18). Otherwise ASCII-magic is simpler.

**My recommendation: DAG-CBOR wrapping**, on the grounds that (a) Benten already uses BLAKE3 CIDs (Inv-5) which align naturally with IPLD; (b) the §3.5s discipline explicitly names "multicodec container approach" as a cross-ecosystem-aligned direction; (c) DAG-CBOR canonicalization is already in scope for Inv-1; (d) CBOR-tag 96 (COSE_Encrypt) is the closest cross-ecosystem framing for "encrypted envelope" — Benten's envelope could even use a Benten-private CBOR-tag (e.g., `0xBE54` registered in the CBOR-tag registry) to be unambiguously identifiable while keeping Benten-internal semantics.

### 4.3 What a recognizability-improved §6.2 envelope buys

- **libmagic / file(1) recognition.** Forensic / backup-tooling can identify Benten files without Benten knowledge.
- **Cross-language interop.** A future Benten TypeScript port reads the same bytes correctly via standard CBOR libraries; no Benten-private framing parser to mirror.
- **Format-versioning headroom.** v1 → v2 upgrade is a 1-byte format-version field bump, not a codepoint-space reorg.
- **Pre-decode validity check.** Outsider tools can validate "this is a Benten v1 envelope structurally" without decrypting / verifying — useful for sanity-checking backups, debugging.
- **§3.5s alignment.** A DAG-CBOR-wrapped envelope at a registered CBOR tag carries the cross-ecosystem identifier (the CBOR tag itself + a registered alg field inside) AS CONTENT, exactly per §3.5s.

---

## §5 Forward-additivity for future ecosystems

**Question:** When MLS-PQ matures, can Benten's envelope add an MLS-PQ codepoint additively without breaking v1-beta wire format? When OpenPGP-PQC stabilizes, can OpenPGP-PQC envelopes be wrapped or referenced? When draft-skokan-jose-hpke-pq-pqt advances, can Benten use those JOSE codepoints?

### 5.1 MLS-PQ (`draft-ietf-mls-pq-ciphersuites-04`)

**Verdict: structurally fine; trivially additive.**

MLS-PQ defines 5 ciphersuites: ML-KEM-768 + X25519 (hybrid), ML-KEM-768 + P-256 (hybrid), ML-KEM-1024 + P-384 (hybrid), pure ML-KEM-768, pure ML-KEM-1024. The first matches Benten's existing CipherSuiteCodepoint::HYBRID_X25519_MLKEM768 = 0x647A.

**Path forward:**
1. Benten's §6.2 codepoint-dispatch absorbs new codepoints naturally per Amendment 8 (codepoint-registry-governance). When MLS-PQ ciphersuite identifiers receive IANA codepoints, add rows to Benten's `codepoint → MLS-ciphersuite-name` mapping table.
2. Benten's envelope is **not** an MLS message; MLS group-key-encapsulation is its own protocol with TreeKEM + Welcome messages + GroupContext binding. Benten ↔ MLS interop happens at a higher protocol layer, not at the envelope layer.
3. If/when Benten ever decides to ship a group-encryption mode using MLS as the underlying group-keying protocol, it would emit MLS Welcome/Commit messages natively (separate adapter), and the §6.2 envelope at the Atrium-mesh layer would carry MLS ciphersuite IDs as content per §3.5s.

**No structural blocker.** No v1-beta wire-format breakage required.

### 5.2 OpenPGP-PQC

**Verdict: not blocked; bridging is unlikely to be load-bearing.**

OpenPGP-PQC stabilization is pending (`crypto-refresh` LWG; RFC 9580 + post-RFC-9580 PQ-PKE drafts). OpenPGP's packet-tag dispatch is fundamentally different from Benten's codepoint dispatch (variable-length packet headers, packet-type tags 1-19+, sub-packet structure inside signatures). **OpenPGP and §6.2 don't share a structural shape.**

**Path forward:** OpenPGP interop happens at a transcoder boundary — if Benten ever needs to emit/consume OpenPGP-PQC envelopes, write a dedicated `benten-openpgp-emit` adapter that translates between §6.2 codepoints + AAD-binding-context + payload-bytes and OpenPGP packet sequences + sub-packets. The §6.2 envelope stays Benten-private; OpenPGP stays OpenPGP-shape. Same pattern as §2's adapter approach for JOSE/COSE/age.

**No structural blocker.** Document as "OpenPGP-PQC bridge deferred to post-v1-beta, pending OpenPGP-PQC stabilization" in V1-FROZEN-INTERFACE-DEFERRED.md.

### 5.3 JOSE-HPKE-PQ (`draft-skokan-jose-hpke-pq-pqt` + `draft-reddy-cose-jose-pqc-hybrid-hpke`)

**Verdict: directly usable; the mapping is one-row.**

The drafts register `HPKE-11` (integrated) and `HPKE-11-KE` (key encryption) for HPKE using MLKEM768-X25519 + SHAKE256 + ChaCha20-Poly1305 — **exactly Benten's Layer-C/D primitive.**

**Path forward:**
1. Add the canonical mapping `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768 = 0x647A → JOSE alg = "HPKE-11-KE"` to Benten's `codepoint_to_jose_alg_name()` function (introduced by Amendment 9; see §7).
2. When/if those drafts get COSE integer codepoints (currently placeholders `TBD12`, `TBD13`), add the COSE row too.
3. Benten's JWE-emit adapter (per §2 matrix) uses this mapping directly. JWE protected header `{"alg": "HPKE-11-KE", ...}` becomes the cross-ecosystem identifier carried as content per §3.5s.

**No structural blocker.** The JOSE-HPKE-PQ drafts and Benten's encryption stack are co-designed at the cryptographic level; the only remaining work is the encoding-level adapter.

### 5.4 LAMPS encryption-side OIDs

**Verdict: forward-additive; not currently usable.**

LAMPS today defines OIDs for raw ML-KEM-768 KEM key-establishment and for Composite ML-DSA signatures (Benten's existing Compromise #31 path). LAMPS does **not** define an OID for HPKE-MLKEM768-X25519-ChaCha20-Poly1305 envelope encryption today, and the LAMPS WG charter focuses on PKIX/CMS use cases rather than full envelope encryption.

**Path forward:** when/if LAMPS or a sister WG (LWIG-style) mints an OID for HPKE-MLKEM768-X25519 envelope encryption, Benten's mapping table absorbs it as a new row. No current emit path; no current consumer.

### 5.5 Forward-additivity summary

| Future ecosystem | Structural blocker? | Path forward |
|---|---|---|
| MLS-PQ ciphersuites | No | Add codepoint→MLS-ciphersuite-name row when MLS-PQ codepoints land at IANA |
| OpenPGP-PQC | No | Build adapter post-v1-beta when OpenPGP-PQC stabilizes |
| JOSE-HPKE-PQ | No — directly aligned | Add codepoint→JOSE-alg-name row now (`0x647A → HPKE-11-KE`); build JWE-emit adapter when needed |
| LAMPS encryption OIDs | No | Add codepoint→LAMPS-OID row when LAMPS mints |
| Future Bird-of-Prey-class signature combiner (SUF-CMA at construction) | No — already in Benten's roadmap | Forward-additive per Compromise #31's existing roadmap |

**Conclusion: §6.2-with-Amendments + the proposed Amendment 9 (cross-ecosystem-identifier-as-content) is forward-additive for every relevant future ecosystem.** The codepoint-dispatch shape is the right invariant to lock; the ecosystem-identifier mappings live in adapter code that can change without breaking the v1-beta wire format.

---

## §6 Position B blog §4 cross-ecosystem-narrative recommendations

Position B blog §4 refresh (per the F-full + Inv-15 + Inv-16 + encrypt-to-recipient maturation) should articulate Benten's cross-ecosystem stance honestly. Recommended narrative posture:

### 6.1 Naming discipline for blog §4 prose

| Concept | Use this name in blog | NOT this name |
|---|---|---|
| Benten's hybrid PQ KEM | `MLKEM768-X25519` (CFRG-research-group-adopted per `draft-irtf-cfrg-concrete-hybrid-kems-03` §4.2) | "X-Wing" (the construction differs from `draft-connolly-cfrg-xwing-kem-10` on hash function + label per cryptographer-review-bird-of-prey-vs-lamps finding; using "X-Wing" is a §3.5s violation in cross-ecosystem-facing prose) |
| Benten's HPKE construction | `HPKE-Base-MLKEM768-X25519-SHAKE256-ChaCha20Poly1305` (full descriptive name) OR the JOSE shorthand `HPKE-11-KE` (key-encryption mode) | "Benten's encryption envelope" (too Benten-private for JOSE-stack readers) |
| Benten's hybrid signature | `MLDSA65-Ed25519` (CFRG-aligned) at the construction layer; `id-MLDSA65-Ed25519-SHA512` (LAMPS-OID-aligned) at the PKIX-emit layer | "Bird-of-Prey" (Benten-internal codename); "X-Wing-signature" (wrong construction) |
| Benten's internal dispatch codepoints | "Benten-internal hot-path dispatch numbers (e.g. `0x647A` for the hybrid KEM)" — name them as INTERNAL when introducing them | Don't lead with the codepoint as if it's the ecosystem-name |
| Cross-ecosystem identifier discipline | "Benten's wire format carries the JOSE/COSE/LAMPS/multicodec identifier as content; internal dispatch numbers stay internal" | Don't conflate codepoints with ecosystem-names |

### 6.2 Recommended §4 cross-ecosystem narrative arc

Suggested structure for blog §4 refresh (1-2 paragraphs each):

1. **"Why Benten's encryption envelope looks Benten-private but is ecosystem-interop-by-design."** Explain that §6.2 is the internal wire format optimized for Benten's at-rest peer-mesh use case (BLAKE3 CIDs, DAG-CBOR canonicalization, codepoint-dispatch for hot-path performance), and that ecosystem boundary surfaces (JWE, COSE, age, multicodec, did:jwk) are bridged by per-ecosystem adapters that emit the cross-ecosystem identifier as content. **Key sentence:** "we built one internal envelope optimized for our threat model, and we bridge to JOSE / COSE / age / multicodec / LAMPS at adapter boundaries — using each ecosystem's native identifier where its consumers expect to see it."

2. **"How we pick algorithms — convergence-cohort discipline."** Name the convergence-cohort partners (OpenPGP-PQC + Sigstore + LAMPS + JOSE-HPKE-PQ + multicodec maintainers) and articulate that Benten's algorithm choices track the cohort's converging consensus (MLKEM768-X25519 per CFRG draft; MLDSA65-Ed25519 per LAMPS Composite ML-DSA; HPKE-11/HPKE-11-KE per JOSE-HPKE-PQ). **Key sentence:** "the cryptographic community is converging on a small set of hybrid PQ primitives; we picked the ones the converging consensus is settling on, not Benten-private inventions."

3. **"Honest about the gaps."** Name Compromise #30 (unaudited PQ primitives; closes at v1-GM) and the post-v1-beta-ecosystem-bridge backlog (Saltpack-PQ pending Saltpack; OpenPGP-PQC pending stabilization; full LAMPS encryption OID pending). **Key sentence:** "we ship what's ready, name what's not, and refuse to fabricate compatibility with ecosystems that don't yet have a PQ story."

4. **"What we're NOT claiming."** Explicitly reject claims that Benten *is* a JOSE / COSE / age implementation; Benten *speaks* JOSE / COSE / age at adapter boundaries when consumers need it, but its internal envelope is its own. **Key sentence:** "our envelope is Benten-shaped because our threat model is Benten-specific — decentralized peer-mesh + at-rest replicated ciphertext on untrusted peers — and forcing a JOSE-shape on top of that would be either misleading or actively harmful."

5. **"How this connects to encrypt-to-recipient + Inv-16."** Inv-16 (the encrypt-to-recipient invariant the F-full review introduces) plugs into the §6.2 envelope at Layer-C (drop) codepoint dispatch; the Benten-side adapter for a JWE-stack consumer rewrites the §6.2 envelope into a JWE JSON Serialization carrying `alg = HPKE-11-KE` (or the integrated `HPKE-11`) without touching the underlying cryptography. **Key sentence:** "encrypt-to-recipient at the user-facing layer means dropping a §6.2 envelope onto the Atrium peer-mesh; cross-ecosystem interop means that envelope can be re-emitted as JWE when a JOSE-stack consumer needs it, with no cryptographic re-encryption."

### 6.3 What blog §4 should NOT do

- Do not lead with the §6.2 envelope diagram; lead with the ecosystem alignment story and let the envelope shape follow.
- Do not name the envelope as "X-Wing-encrypted" — that's wrong (the construction isn't true X-Wing) and §3.5s-non-compliant.
- Do not promise Saltpack interop or current OpenPGP-PQC interop; those are open backlog items, not v1-beta deliverables.
- Do not understate the §3.5s posture; the discipline is a substantive cross-ecosystem-citizenship claim and is differentiating relative to small-team-fragments-ecosystem patterns the L1 / L4 critics flag elsewhere.
- Do not over-claim that §6.2 IS COSE or IS JOSE — it isn't, and won't be; what Benten claims is *convertibility-at-adapter-boundaries-with-correct-cross-ecosystem-identifiers*.

---

## §7 Recommended amendments to §6.2 + Amendments 1-6 for interop

Two new amendments to close the §3.5s compliance gap + the wire-format-recognizability gap. Numbered to continue the existing Amendment 1-8 sequence:

### 7.1 Amendment 9 — Cross-ecosystem-identifier-as-content emit-discipline (§3.5s compliance)

**Mandate.** Every Benten path that emits a §6.2 envelope to a cross-ecosystem-boundary surface (JWE / COSE / age / multicodec / did:jwk / LAMPS-OID-emit / IPLD-block-emit) MUST translate the Benten-internal `codepoint: u16` into the matching cross-ecosystem identifier as content of the adapter output. The internal codepoint MUST NEVER appear in a cross-ecosystem-boundary output in lieu of the cross-ecosystem identifier.

**Mapping table (initial; lives in `benten-crypto-suite::codepoint::cross_ecosystem_map`).** Concrete table values (specific assignments) need final cite-verification at landing time, but the *shape* is:

```rust
pub struct CrossEcosystemIdentifiers {
    /// LAMPS / X.509 / CMS algorithm-identifier OID (signature surfaces).
    /// Sig codepoints carry this; encryption codepoints generally don't (LAMPS doesn't define encryption OIDs yet).
    pub lamps_oid: Option<&'static str>,
    /// JOSE / JWE / JWK / JWS algorithm-name (case-sensitive ASCII).
    /// e.g. "HPKE-11-KE" for HYBRID_X25519_MLKEM768 + ChaCha20-Poly1305 + key-encap mode.
    pub jose_alg_name: Option<&'static str>,
    /// COSE algorithm integer codepoint (negative for IANA-assigned values).
    /// e.g. -55 for MLDSA65-Ed25519; TBD12/TBD13 for HPKE-MLKEM768-X25519 variants (placeholders pending IANA).
    pub cose_alg_id: Option<i32>,
    /// Multicodec public-key codepoint (varint), where applicable.
    /// e.g. 0x120c for ML-KEM-768 public key; 0xec for X25519.
    pub multicodec_key_code: Option<u32>,
    /// age stanza type name, where applicable.
    /// e.g. "mlkem768x25519" for the C2SP age MLKEM768X25519 recipient stanza.
    pub age_stanza_name: Option<&'static str>,
}

impl SigCodepoint {
    pub fn cross_ecosystem(self) -> CrossEcosystemIdentifiers { /* ... */ }
}
impl CipherSuiteCodepoint {
    pub fn cross_ecosystem(self) -> CrossEcosystemIdentifiers { /* ... */ }
}
```

**Enforcement seam.** Any path that emits a JWE / COSE_Encrypt / age-file / did:jwk / multicodec-prefixed-bytestring / LAMPS-X.509-AlgorithmIdentifier MUST call `cross_ecosystem().<field>` to get the right identifier; constructing such an output by `format!("{}", codepoint)` or any codepoint-direct path is a §3.5s violation and MUST be caught at PR-review time. Future cite-drift-detector extension (per §3.5s "Enforcement seam") to flag Benten-private-codepoint references in ecosystem-boundary surface code.

**Code anchors needed at landing time:**
- `benten-crypto-suite::codepoint::cross_ecosystem_map` (new module)
- `benten-drop::envelope_jwe_emit` (new module if/when JWE-emit path is needed)
- `benten-drop::envelope_cose_emit` (new module if/when COSE-emit path is needed)
- `benten-drop::envelope_age_emit` (new module if/when age-emit path is needed)
- Tests: `tests/cross_ecosystem_emit_no_codepoint_leak.rs` verifying that JWE/COSE/age outputs contain the registered identifier and never the raw Benten codepoint.

**Confidence on Amendment 9 as load-bearing:** **HIGH**. The §3.5s discipline is Ben-codified; this amendment is the concrete operationalization of §3.5s for the §6.2 envelope. Without it, §6.2 ships as "a Benten-private envelope with no cross-ecosystem story" which is a §3.5s violation by silence.

### 7.2 Amendment 10 — Wire-format-recognizability (framing layer + DAG-CBOR wrapping)

**Mandate.** §6.2 envelopes on the wire / at rest MUST be DAG-CBOR-encoded with a Benten-private CBOR-tag (recommend `0xBE54` — register at the CBOR-tag IANA registry pre-v1-beta) at the outermost framing layer. The DAG-CBOR canonicalization rules per Inv-1 apply.

**Why DAG-CBOR** (over ASCII-magic-prefix or ad-hoc-binary):
1. CBOR-aware tools (cbor-diag, panva/jose for the JOSE-HPKE-CBOR variant, COSE libraries) recognize the outer structure without Benten-private code.
2. The Benten BLAKE3 CID for a §6.2 envelope is computed over the DAG-CBOR canonical bytes, which aligns with IPLD content-addressing for free (Inv-5).
3. IPLD CAR-file backups, IPFS-style peer-mesh sync (foreshadowed by baked-in #17 / #18), and any future content-addressed-storage path get free interop.
4. DAG-CBOR canonicalization is already in Inv-1 scope; no new discipline to introduce.
5. Cross-language ports (a future TypeScript Benten client) get standard CBOR libraries for free; no Benten-private framing parser to mirror.

**Envelope shape post-Amendment-10:**

```
DAG-CBOR-tagged(0xBE54):
  CBOR map {
    "v": 1,                                          // format-version
    "c": <codepoint: u16>,                           // §6.2 + Amendment 7 BE codepoint
    "p": EnvelopePayload (codepoint-discriminated),  // §6.2 payload variant
    "b": BindingContext (codepoint-discriminated),   // §6.2 binding-context variant
  }
```

Amendment 1 (codepoint-in-AAD) still binds the codepoint into the AAD; AAD now also binds the format-version `v` field (to prevent v1↔v2 confusion attacks). Amendment 3 (TLV length-injectivity) is **subsumed** by DAG-CBOR canonical encoding (CBOR is self-describing-length per RFC 8949; the byte-injectivity property holds under deterministic encoding).

**Trade-off vs. ASCII-magic alternative.** ASCII-magic + ad-hoc-binary saves ~5% wire size + avoids CBOR-canonicalization-spec-version risk. DAG-CBOR pays that cost in exchange for ecosystem-tool recognition + CID-alignment + future-language-port-ease. **Net recommendation: DAG-CBOR is worth the cost** for a project that has already committed to BLAKE3 CIDs + Atrium peer-mesh sync (both align naturally with IPLD/DAG-CBOR).

**Confidence on Amendment 10 as load-bearing:** **MEDIUM-HIGH**. The wire-format-recognizability gap is real but is a "nice-to-have" relative to Amendment 9's §3.5s-compliance gap. If Benten ships §6.2 without Amendment 10, it's still a valid internal wire format; outsiders just have no way to recognize it. If Benten ships §6.2 without Amendment 9, it's §3.5s-non-compliant by construction. **Amendment 9 is mandatory; Amendment 10 is strongly recommended.**

### 7.3 Optional Amendment 11 — Per-ecosystem-adapter spec hooks

**Mandate (lighter-weight).** §6.2 design doc MUST name a per-ecosystem adapter list (initial set: JWE, COSE, age, IPLD-raw-block, multicodec-public-key-emit) and explicitly defer the others (Saltpack, OpenPGP-PQC) with named conditions for un-deferral.

Reason for naming this as an Amendment rather than a backlog row: the §6.2 design surface implicitly assumes "Benten talks to itself"; making the adapter list explicit forces the F-full design to surface the ecosystem-boundary surfaces at design time, which aligns with §3.5s "New wire-format envelope designs MUST explicitly answer: 'what cross-ecosystem identifier does this envelope carry as content?'"

**Confidence:** MEDIUM. This is documentation discipline, not security discipline. If the team prefers to track this in V1-FROZEN-INTERFACE-DEFERRED.md instead of as an Amendment, that's acceptable. Either way, the discipline MUST land before v1-beta tag.

### 7.4 Refinements to existing Amendments (1-8) for interop

- **Amendment 1 (codepoint-in-AAD)** — add a clause: the codepoint included in canonical-binding MUST be the Benten-internal codepoint (`0x647A` form); when the envelope is re-emitted to a JWE/COSE/age adapter, the adapter MUST translate to the cross-ecosystem identifier and bind THAT into the JWE/COSE AAD per the ecosystem's rules (not pass through the internal codepoint to the adapter output's AAD).
- **Amendment 2 (strict-decode)** — extend the codepoint→variant-tag dispatch table to ALSO reject any envelope where the format-version field `v` doesn't match a known version.
- **Amendment 3 (TLV length-injectivity)** — subsumed by DAG-CBOR canonical encoding under Amendment 10; if Amendment 10 lands, Amendment 3's hand-rolled TLV scheme is replaced by CBOR-deterministic-encoding (RFC 8949 §4.2.1 "Core Deterministic Encoding"); the byte-injectivity property holds. If Amendment 10 does NOT land, Amendment 3 stays as specified by the third reviewer.
- **Amendment 7 (codepoint endianness)** — explicitly pin "BE on Benten-internal wire" — and add that "the BE encoding is irrelevant to JWE / COSE adapter outputs because those carry the cross-ecosystem identifier as content, not the raw 16-bit codepoint."
- **Amendment 8 (codepoint-registry governance)** — extend with cross-ecosystem-mapping-table maintenance: when a new codepoint is minted, the `CrossEcosystemIdentifiers` row MUST be populated for every relevant ecosystem at the same PR (or explicitly named as `None` with reason). Atomic-update discipline per §3.5g pattern.

---

## §8 Self-assessment + confidence + lower-confidence areas

### 8.1 High-confidence findings

- **§6.2-with-Amendments-1-6 is §3.5s-non-compliant by silence** at the encryption-side ecosystem boundaries (JWE, COSE, age, did:jwk, multicodec). Discipline-violation, not soundness-violation. (HIGH confidence)
- **The JOSE alg-name for Benten's hybrid is `HPKE-11` / `HPKE-11-KE`** per `draft-skokan-jose-hpke-pq-pqt-02` and `draft-reddy-cose-jose-pqc-hybrid-hpke-11`. The brief's strawman descriptive string is not a registered identifier. (HIGH — directly verified via WebFetch on the IETF drafts)
- **No vanilla HPKE wire-format exists to interop against** — RFC 9180 §10 explicitly disclaims a wire format. Cross-ecosystem interop happens at JWE/COSE/age boundaries, not at "raw HPKE" boundaries. (HIGH — verified via RFC 9180 §10 fetch)
- **0 of 9 ecosystems can parse §6.2 bytes directly today** — all require Benten-side adapters. This is expected and is exactly what §3.5s describes; the gap is in adapter spec, not in §6.2's internal shape. (HIGH)
- **Amendment 9 (cross-ecosystem-identifier-as-content emit-discipline) is the load-bearing recommendation.** Without it, §6.2 ships with no §3.5s compliance posture. (HIGH)
- **The signature-side path (Compromise #31 + Inv-15 + LAMPS-OID-as-content) is already §3.5s-compliant** — it's the §3.5s exemplar. The encryption-side gap is what's new. (HIGH)
- **Forward-additivity for MLS-PQ / OpenPGP-PQC / JOSE-HPKE-PQ / LAMPS-encryption-OIDs is structurally fine.** §6.2's codepoint-dispatch shape absorbs new codepoints additively. (HIGH)

### 8.2 Medium-confidence findings

- **DAG-CBOR wrapping (Amendment 10) is the right framing.** The trade-off vs. ASCII-magic is real (5% size cost + canonical-form discipline); the net recommendation is DAG-CBOR but reasonable designers might disagree. (MEDIUM-HIGH)
- **multicodec table has `mlkem-768-pub = 0x120c` (draft) and NOT `cose-key = 0x42` or `jwk = 0x44` as the brief claims.** This is verified against the live multicodec/table.csv at master HEAD; the brief's "PRs #400/#403" cite appears stale or anticipatory. Direction (use multicodec for public-key surfaces) is right; specific codepoints cited in the brief are not what's in the table. (MEDIUM-HIGH — direct CSV verification)
- **CBOR-tag 0xBE54 (or any specific Benten-private CBOR-tag value) for Amendment 10** — exact tag value should be picked via IANA CBOR-tag registry consultation; the recommendation is the *direction* of using a registered CBOR-tag, not the specific value. (MEDIUM)
- **Position B blog §4 narrative recommendations** are stylistic + framing-judgment; reasonable editors will adjust phrasing. (MEDIUM)

### 8.3 Lower-confidence areas (honest disclosure)

- **COSE integer alg-IDs for HPKE-MLKEM768-X25519 are TBD** per the Reddy/Skokan drafts (placeholders TBD12/TBD13 pending IANA assignment). The exact COSE integer value Benten will use is unknown at the moment; the design must track IANA assignment and update the mapping table. (LOWER — pending IANA action)
- **age v1's `mlkem768x25519` stanza details** — I verified the stanza name exists in C2SP age.md but did not verify byte-level compatibility between age's `enc` encoding and Benten's MLKEM768-X25519 KEM-share encoding. Cross-validation against age-rs test vectors is needed before claiming age binary-compat. (LOWER — needs test-vector verification at adapter-build time)
- **JWK `kty` value for ML-KEM-768 keys** — `draft-ietf-jose-pqc-kem-05` is still in flux on whether the kty is `AKP` ("algorithm key pair" — a new kty under discussion) or something else. Benten's did:jwk emit path must track this. (LOWER — draft-stage spec)
- **Whether to recommend Amendment 10 as required vs. recommended.** My recommendation is "strongly recommended" not "required" — a Benten-private framing without CBOR wrapping is a valid choice, just one that pays an ongoing cost in ecosystem-tool recognition. Reasonable cryptographers + standards-people will weight this differently. (MEDIUM-LOWER)
- **Whether Benten should pursue Saltpack-PQ interop in the future.** Saltpack's design choices (MessagePack framing, NaCl crypto, mode=0 encryption with X25519-Salsa20-Poly1305) are at odds with Benten's PQ-hybrid + HPKE direction. Even when Saltpack adds PQ support (no firm roadmap as of 2026-05-27), Benten interop may not be worth the adapter cost. (LOWER — strategic call, not technical)

### 8.4 What this review does NOT cover

- The internal cryptographic soundness of §6.2-with-Amendments — covered by the three prior reviewers (first cryptographer, second-opinion, third-reviewer adversarial). I take their findings as established.
- The Option F+ NO-GO decision itself — covered by the first reviewer + concurred by the second. Out of scope for L7.
- Specific test-vector specifications for each adapter — those land in R3/R5 implementer briefs, not in this lens.
- The exact CBOR-tag value to register at IANA — needs registry consultation; recommendation here is directional.
- The full ecosystem-adapter-implementation work — out of scope for L7; this lens establishes the *requirements* the adapters must meet (Amendment 9 + Amendment 10 + per-ecosystem adapter spec).
- DID-method-spec interop beyond did:jwk (e.g., did:web, did:peer, did:plc) — Benten's broader DID story is covered elsewhere.
- The signature-side cross-ecosystem story beyond Compromise #31 + Inv-15 — already in flight; only mentioned here for completeness.

### 8.5 Self-critique

A more conservative reviewer might say: "the §3.5s discipline already applies; restating it as Amendment 9 is redundant." I disagree — §3.5s codifies the *project-wide rule*; Amendment 9 codifies the *concrete operationalization for the §6.2 envelope*. Without Amendment 9, every future F-full implementer has to re-derive the §6.2-specific application of §3.5s, which is exactly the kind of re-derivation that pim-N-prior-phase-pim-explicit-preflight (§3.6g) was minted to prevent. Naming it as an Amendment is the right discipline.

A more aggressive reviewer might say: "Amendment 10 (DAG-CBOR) should be required, not recommended." I considered this but stopped at "strongly recommended" because the case for DAG-CBOR over ASCII-magic-prefix is real but not overwhelming, and reasonable cryptographers might prefer the lighter-weight option. If Ben weighs the IPLD-alignment + cross-language-port argument heavily, he should promote Amendment 10 to required; if he prefers the lighter-weight path, the recommendation as written is appropriate.

A reviewer with more JOSE-WG experience might push back on my mapping of `0x647A` to `HPKE-11-KE` vs. `HPKE-11` (integrated). The distinction matters: integrated mode (`HPKE-11`) means HPKE itself bulk-encrypts the plaintext; key-encryption mode (`HPKE-11-KE`) means HPKE encrypts a content-encryption-key (CEK) which then bulk-encrypts the plaintext with a separate AEAD. Benten's actual usage at Layer-C/D — does it bulk-encrypt via HPKE directly, or does HPKE wrap a CEK that then bulk-encrypts? I assumed key-encryption mode (`HPKE-11-KE`) because Benten's pattern is "HPKE wraps K_principal for the recipient, then K_principal-derived K(N) bulk-encrypts" — that's the key-encryption shape. If Benten's Layer-C is actually integrated-encryption shape, the mapping is `HPKE-11` not `HPKE-11-KE`. This needs verification against the F-full Layer-C design at landing time. (Confidence on this specific mapping: MEDIUM-HIGH; correct shape but exact integrated-vs-KE call needs F-full-design cross-check.)

### 8.6 What additional review I would want

1. **Standards-WG / IETF JOSE-WG engagement** — Benten's `0x647A → HPKE-11-KE` mapping is correct per the drafts at HEAD, but JOSE-WG could mint a different/aligned identifier as part of future PQ-hybrid registrations. Building a relationship with JOSE-WG chairs + Skokan + Campbell pre-v1-beta-tag would let Benten flag the mapping early.
2. **age-rs test-vector cross-validation** — before Benten claims age `mlkem768x25519` interop, verify byte-level equivalence between age-rs MLKEM768X25519 stanza encoding and Benten's KEM-share encoding.
3. **CBOR-tag IANA registration** — for Amendment 10, register a Benten CBOR-tag at the IANA CBOR-tag registry pre-v1-beta-freeze; the exact tag value should match a real registration.
4. **Multicodec PR cross-reference** — the brief cites "PRs #400/#403" which don't match the live table. Either the orchestrator's citation is stale (newer PRs supersede; need to find the current state) or anticipatory (PRs proposed but not merged; Benten can choose to participate in pushing them through). Worth a quick GitHub `multiformats/multicodec` PR-list sweep at landing time.
5. **A second-opinion standards-interop reviewer** — this lens is solo; a second pass from someone with more JOSE-WG / COSE-WG operational experience would sharpen the alg-name + COSE-integer-codepoint recommendations.

---

## §9 Citations

### 9.1 IETF / RFCs / drafts

- **RFC 9180 — Hybrid Public Key Encryption** ([datatracker.ietf.org/doc/html/rfc9180](https://datatracker.ietf.org/doc/html/rfc9180)). §10 (Application Considerations): HPKE explicitly does NOT prescribe a wire format; applications define their own envelope.
- **RFC 9052 — CBOR Object Signing and Encryption (COSE): Structures and Process** ([datatracker.ietf.org/doc/rfc9052/](https://datatracker.ietf.org/doc/rfc9052/)). COSE_Encrypt = CBOR tag 96; protected/unprotected headers; algorithm identifier as integer in the protected map.
- **RFC 7517 — JSON Web Key (JWK)** ([rfc-editor.org/info/rfc7517/](https://www.rfc-editor.org/info/rfc7517/)). `alg` parameter optional, case-sensitive ASCII, references IANA JWS/JWE Algorithms registry.
- **RFC 7515 / RFC 7516 / RFC 7518** — JOSE family (JWS / JWE / JWA).
- **RFC 8949 — Concise Binary Object Representation (CBOR)**. §4.2.1 Core Deterministic Encoding (the canonical-form basis for DAG-CBOR).
- **draft-skokan-jose-hpke-pq-pqt-02** ([ietf.org/archive/id/draft-skokan-jose-hpke-pq-pqt-02.html](https://www.ietf.org/archive/id/draft-skokan-jose-hpke-pq-pqt-02.html)). Registers `HPKE-10` / `HPKE-11` / `HPKE-10-KE` / `HPKE-11-KE` for HPKE with MLKEM768-P256 / MLKEM768-X25519 + SHAKE256 + AES-256-GCM / ChaCha20-Poly1305.
- **draft-reddy-cose-jose-pqc-hybrid-hpke-11** ([datatracker.ietf.org/doc/html/draft-reddy-cose-jose-pqc-hybrid-hpke-11](https://datatracker.ietf.org/doc/html/draft-reddy-cose-jose-pqc-hybrid-hpke-11)). Post-Quantum and Hybrid KEMs for HPKE with JOSE and COSE. Confirms `HPKE-11` = MLKEM768-X25519 + SHAKE256 + ChaCha20-Poly1305.
- **draft-ietf-jose-hpke-encrypt-11/15** ([datatracker.ietf.org/doc/draft-ietf-jose-hpke-encrypt/](https://datatracker.ietf.org/doc/draft-ietf-jose-hpke-encrypt/)). Integrated vs. key-encryption modes; protected header parameters (`alg`, `enc`, `ek`, `psk_id`).
- **draft-ietf-jose-pqc-kem-05** ([datatracker.ietf.org/doc/draft-ietf-jose-pqc-kem/](https://datatracker.ietf.org/doc/draft-ietf-jose-pqc-kem/)). Defines `ML-KEM-512` / `ML-KEM-768` / `ML-KEM-1024` JOSE alg names; key-wrap variants `ML-KEM-768+A192KW` etc.
- **draft-ietf-mls-pq-ciphersuites-04** ([datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/](https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/)). 5 MLS PQ/hybrid ciphersuites; ML-KEM-768 + X25519 listed as hybrid.
- **draft-irtf-cfrg-concrete-hybrid-kems-03**. CFRG-adopted name `MLKEM768-X25519`.
- **draft-connolly-cfrg-xwing-kem-10**. X-Wing KEM definition; Benten's construction differs (see Compromise #31 cryptographer-review finding).
- **draft-ietf-lamps-pq-composite-sigs-19**. LAMPS Composite ML-DSA OID `id-MLDSA65-Ed25519-SHA512 = 1.3.6.1.5.5.7.6.48` (Benten's existing §3.5s exemplar).

### 9.2 Ecosystem specs

- **C2SP age v1 spec** ([github.com/C2SP/C2SP/blob/main/age.md](https://github.com/C2SP/C2SP/blob/main/age.md)). Magic line `age-encryption.org/v1`; stanza format; native `X25519` / `scrypt` / `mlkem768x25519` / `p256tag` / `mlkem768p256tag` recipient types.
- **Saltpack v2 encryption format** ([saltpack.org/encryption-format-v2](https://saltpack.org/encryption-format-v2)). MessagePack-framed; mode=0 encryption with NaCl X25519-Salsa20-Poly1305; no PQ mode.
- **ATProto cryptography spec** ([atproto.com/specs/cryptography](https://atproto.com/specs/cryptography)). P-256 / k256 ECDSA-low-S; did:key multikey with codecs `0x1200` (p256-pub) / `0xe7` (k256-pub); DAG-CBOR for signed payloads; **no encryption envelope** ("data encryption is not used directly in the protocol").
- **multicodec table.csv at master HEAD** ([raw.githubusercontent.com/multiformats/multicodec/master/table.csv](https://raw.githubusercontent.com/multiformats/multicodec/master/table.csv)). Verified rows: `mlkem-512-pub = 0x120b` (draft), `mlkem-768-pub = 0x120c` (draft), `mlkem-1024-pub = 0x120d` (draft), `mldsa-44-pub = 0x1210` (draft), `mldsa-65-pub = 0x1211` (draft), `mldsa-87-pub = 0x1212` (draft), `jwk_jcs-pub = 0xeb51` (draft), `jwk_jcs-priv = 0x1316` (draft). NOT found: `cose-key = 0x42`, `jwk = 0x44` (brief's claimed PR #400/#403 codepoints — not in table at HEAD).
- **OpenPGP RFC 9580** + crypto-refresh LWG drafts (OpenPGP-PQC stabilization in flight).
- **Bluesky discussion #121 — Encryption for private content** ([github.com/bluesky-social/atproto/discussions/121](https://github.com/bluesky-social/atproto/discussions/121)). ATProto encryption story remains open.

### 9.3 Benten internal references

- `.addl/dispatch-conventions.md` §3.5s (cross-ecosystem-identifier-as-content) at `2768b583` — the discipline this lens validates against.
- `docs/SECURITY-POSTURE.md` Compromise #30 (unaudited PQ; closes at v1-GM) + Compromise #31 (LAMPS Composite ML-DSA EUF-CMA-only; closed-equivalent at app layer via Inv-15).
- `docs/INVARIANT-COVERAGE.md` Inv-15 (3-layer decomposition, Phase-4-Meta-Core mint).
- `.addl/phase-4-meta/option-f-plus-pseudo-keypair-review.md` at `6d4e173f` — first reviewer; introduced §6.2 envelope-layer unification.
- `.addl/phase-4-meta/option-f-plus-second-opinion-cryptographer-review.md` at `7e900a3b` — second-opinion; Amendments 1+2.
- `.addl/phase-4-meta/option-f-plus-third-reviewer-adversarial-design.md` at `13b624c3` — third-reviewer; Amendments 3-8.
- CLAUDE.md baked-in #5 (crypto-agility + never-fork-primitives); baked-in #15 (v1-beta gate); baked-in #17 (transport-shape); baked-in #18 (peers-hold-ciphertext / untrusted-host).
- `crates/benten-crypto-suite/src/codepoint.rs::CipherSuiteCodepoint::HYBRID_X25519_MLKEM768 = 0x647A` — the encryption codepoint that maps to JOSE `HPKE-11-KE`.
- `crates/benten-crypto-suite/src/codepoint.rs::SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` — the signature codepoint that maps to LAMPS OID `1.3.6.1.5.5.7.6.48` (the §3.5s exemplar).

### 9.4 Prior reviews + project context

- 3 cryptographer reviewers on F+ envelope-layer unification (above).
- e2r-ffull-scope-review (the F-full scope review feeding this Phase-4-Meta-Core wave).
- Phase-4-Meta-Core orchestration NIGHT-SHIFT-2026-05-26 / NIGHT-SHIFT-2026-05-27 (the F+ third-reviewer dispatch context).

---

*End of L7 cross-ecosystem-interop review.*
