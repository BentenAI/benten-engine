# PQ Codepoint Research — Agent 3: Production Deployments + Ecosystem Precedents

**Scope:** what real, shipped systems are doing TODAY for PQ-hybrid wire-format codepoints. Investigates iroh (primary anchor; shipped 2026-05-19), TLS/messaging ecosystem (Cloudflare/Signal/Apple/MLS/Tor), libp2p/Veilid/IPFS, cryptographic libraries (rustls/aws-lc-rs/BoringSSL), and the convergence pattern across these deployments.

**Branch:** `phase-4-meta-core/pq-codepoint-research-3` off main HEAD `31d1a169`.

**Anti-overlap:** Agent 1 covers multicodec + W3C DID + private-use convention. Agent 2 covers IETF/IANA/COSE/NIST standards bodies. This agent covers what's actually deployed in production.

---

## TL;DR — what's deployed RIGHT NOW (May 2026)

| Surface | KEM Side | Sig Side | Wire-Format / Codepoint |
|---|---|---|---|
| **iroh** (2026-05-19) | X25519MLKEM768 (hybrid) | Ed25519 (classical) — "waiting for industry consensus" | TLS Named Group 0x11EC (IANA) |
| **Cloudflare prod** | X25519MLKEM768 (hybrid) — 43% of human HTTPS traffic | none — "not a single public post-quantum certificate is used" | TLS Named Group 0x11EC (IANA) |
| **Chrome/BoringSSL** | X25519MLKEM768 (hybrid; default Chrome 131+) | none in TLS | TLS Named Group 0x11EC (replaces experimental 0x6399) |
| **Signal PQXDH** (2023+) | X25519 + Kyber-1024 (hybrid) | XEdDSA (classical-only) | implementer-defined single-byte tags (NOT IANA) |
| **Apple iMessage PQ3** | Kyber-1024 + P-256 (hybrid) | ECDSA P-256 (classical-only) | Apple-internal protocol-version field |
| **MLS PQ ciphersuites** (draft-ietf-mls-pq-ciphersuites-04) | ML-KEM or hybrid | 7 of 9 ciphersuites use Ed25519/ECDSA; only TBD8/TBD9 use ML-DSA-65/87 | MLS Ciphersuite registry (TBD numeric values) |
| **MLS X-Wing** (draft-mahy-mls-xwing-00) | X-Wing (hybrid) | Ed25519 (classical) | MLS Ciphersuite (TBD) |
| **Tor proposal 355** (2026-03) | exploring ML-KEM variants | not finalized | Tor-internal HTYPE (not finalized) |
| **libp2p peer-id** | n/a (no PQ KEM at peer-id level) | RSA/Ed25519/Secp256k1/ECDSA only — NO PQ extension | Protobuf KeyType enum (0..3); no PQ values |
| **Veilid** (VLD0) | curve25519 + x25519 + XChaCha20-Poly1305 | curve25519 (classical) | "VLD0" four-char crypto-kind tag; designed for migration but VLD1/PQ NOT defined |
| **Multicodec table** | pure ML-KEM-512/768/1024 (0x120b/0x120c/0x120d; draft) | pure ML-DSA-44/65/87 (0x1210/0x1211/0x1212; draft) + SLH-DSA series (0x1220–0x122b; draft) | **NO hybrid composite codepoint registered or proposed** |
| **IETF LAMPS composite ML-DSA** (X.509/CMS/TLS draft) | n/a | 18 composite OIDs: id-MLDSA44-Ed25519-SHA512 = `1.3.6.1.5.5.7.6.39`; id-MLDSA65-Ed25519-SHA512 = `1.3.6.1.5.5.7.6.48`; etc. | OID arc `1.3.6.1.5.5.7.6.{37..54}` |
| **X-Wing** (draft-connolly-cfrg-xwing-kem-09) | hybrid KEM | n/a | HPKE KEM ID + TLS Named Group **both requested at 25722 = 0x647A** (still "(please)" — not yet IANA-permanent) |

**The two most important convergence signals:**
1. **KEM side:** the entire deployment landscape has converged on X25519MLKEM768 with IANA TLS Named Group **0x11EC** (4588). Cloudflare, Chrome, iroh, OpenSSL 3.5+, JDK 24+ — all on the same codepoint.
2. **Signature side:** nearly EVERY deployed PQ-hybrid system keeps signatures CLASSICAL (Ed25519 or ECDSA). Hybrid SIGNATURES are spec-paper (LAMPS composite ML-DSA OIDs) — not deployed. iroh, Signal, Apple, Cloudflare, MLS-PQ-default, X-Wing-MLS all confirm.

---

## Section A — Iroh deep-read (PRIMARY ANCHOR)

### A.1 The 2026-05-19 PQ post

Source: https://www.iroh.computer/blog/iroh-post-quantum-handshakes

**Ciphersuite chosen:** `X25519MLKEM768` — a hybrid key exchange combining X25519 and ML-KEM-768.

**Wire-format codepoint:** the iroh post itself does NOT publish the numeric codepoint, but references the IETF draft `draft-ietf-tls-ecdhe-mlkem`. That draft assigns:

| Group | Decimal | Hex | Recommended | DTLS |
|---|---|---|---|---|
| **X25519MLKEM768** | **4588** | **0x11EC** | N | Y |
| SecP256r1MLKEM768 | 4587 | 0x11EB | N | Y |
| SecP384r1MLKEM1024 | 4589 | 0x11ED | N | Y |

Source: `https://www.ietf.org/archive/id/draft-ietf-tls-ecdhe-mlkem-04.html` §7 IANA Considerations. The draft also obsoletes the experimental codepoints `X25519Kyber768Draft00` (25497) and `SecP256r1Kyber768Draft00` (25498), marking them "Recommended: D".

**Negotiation mechanism:** iroh uses rustls' `prefer-post-quantum` feature flag (rustls 0.23.22+) and the `tls-aws-lc-rs` provider. The flag name signals preference, not enforcement — clients without PQ support negotiate down to classical X25519.

**Signature posture (the load-bearing quote):**

> **"We are waiting for an industry consensus to emerge."**

Extended:

> "post-quantum signature algorithms are [the subject of active research]… All current options have [significant downsides]… So there is no industry consensus yet which one to use."

iroh is shipping **PQ-KEM + classical Ed25519 signatures** for v1. They explicitly defer the signature side.

**Rationale for X25519MLKEM768 vs P-256+ML-KEM-768:** the hybrid framing — "The session key is safe as long as *at least one* of the two primitives holds — so if ML-KEM turns out to be broken, X25519 still protects you." X25519 is the workspace-standard EC primitive (matches libp2p/Noise/most modern P2P); aligning with what the broader ecosystem uses.

### A.2 Architectural placement in iroh

Iroh uses **noq** (`n0/noq`, a Quinn fork) for QUIC transport. The PQ key exchange rides inside the TLS 1.3 handshake that QUIC carries — same codepoint plumbing as TLS-over-TCP. The iroh `EndpointId` IS an `ed25519_dalek::VerifyingKey` (per Spike H findings already in CLAUDE.md §2026-05-21 night block) — so peer identity stays Ed25519 even as the transport-key-exchange goes PQ-hybrid.

This is the **load-bearing pattern** for Benten alignment consideration: the wire transport gets PQ-hybrid for forward secrecy / harvest-now-decrypt-later, but the peer-identity-DID stays classical for v1-beta. Benten's case is structurally identical (peer-identity `did:key` at Atrium layer; encryption envelope at content layer).

### A.3 What's missing from iroh's posture

Iroh's blog post does NOT discuss:
- Encryption AT REST (their model assumes ephemeral session keys)
- Content-addressed signature schemes (peer-identity-only Ed25519, no content-sig story)
- Multi-device key wrap with PQ-hybrid (iroh assumes single-device-per-EndpointId)

These gaps are EXACTLY where Benten's S&C work (per CLAUDE.md G-CORE-3 ratification) extends beyond iroh — and where Benten cannot copy iroh's choices wholesale, because iroh hasn't made them.

---

## Section B — Decentralized-identity + content-addressed ecosystem

### B.1 libp2p — frozen at classical

Source: https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md

```
enum KeyType {
    RSA = 0;
    Ed25519 = 1;
    Secp256k1 = 2;
    ECDSA = 3;
}
```

**Zero PQ entries. Zero reserved-for-PQ slots. Zero in-flight proposals discoverable via web search.** libp2p's peer-id spec is the original "decentralized peer identity" precedent and has not budged on PQ in the public spec repo. The libp2p ecosystem implicitly waits for the broader IETF/W3C direction.

### B.2 Veilid (VLD0) — designed for migration but PQ slot empty

Source: https://veilid.com/how-it-works/cryptography/

Veilid's design uses a four-byte crypto-kind tag (e.g., `"VLD0"`) prepended to all keys/signatures/hashes:

> "Cryptographic keys, signatures, and hashes are all tagged with their cryptosystem to ensure that we know exactly how they were generated and how they should be used and persisted."

> "Reading persisted data will automatically use the correct cryptosystem and will default to always writing it back using the newest/best cryptosystem... While transitioning cryptosystems, nodes can respond to other nodes using either the old system or the new one, or both."

VLD0 primitives: XChaCha20-Poly1305, curve25519 sign+DH, BLAKE3 hash, Argon2 password KDF. **All classical. No `VLD1` PQ kind defined in public docs.** The architecture is ready for PQ migration (tag-based dispatch), but the PQ kind hasn't been minted. This is a precedent FOR the "self-describing tag with reserved slot" pattern; not yet a precedent for the specific PQ codepoint values.

### B.3 IPFS / libp2p multicodec — pure PQ entries only, no hybrids

Multicodec table (`multiformats/multicodec`, master branch, grepped 2026-05-25):

**Pure ML-KEM (FIPS 203):**
- `mlkem-512-pub` = 0x120b (draft)
- `mlkem-768-pub` = 0x120c (draft)
- `mlkem-1024-pub` = 0x120d (draft)
- `mlkem-512-priv` = 0x1313 (draft)
- `mlkem-768-priv` = 0x1314 (draft)
- `mlkem-1024-priv` = 0x1315 (draft)

**Pure ML-DSA (FIPS 204):**
- `mldsa-44-pub` = 0x1210 (draft)
- `mldsa-65-pub` = 0x1211 (draft)
- `mldsa-87-pub` = 0x1212 (draft)
- `mldsa-{44,65,87}-priv` = 0x1317/0x1318/0x1319 (draft)
- `mldsa-{44,65,87}-priv-seed` = 0x131a/0x131b/0x131c (draft)

**Pure SLH-DSA (FIPS 205):** 12 entries `slhdsa-*-pub` at 0x1220–0x122b (draft).

**Hybrid entries:** **NONE.** No `ed25519-mldsa65`, `x25519-mlkem768`, `xwing`, or any composite. No discoverable open issue or PR proposing such an entry as of 2026-05-25.

This is the most important deployment finding: **the multicodec maintainers have minted draft codepoints for every NIST-standard pure PQ primitive, but have NOT minted hybrid composites.** This suggests the hybrid layer is expected to live OUTSIDE multicodec — either as application-level concat-of-two-multikey blobs (per Agent 1's likely framing) OR via an unminted slot Benten could reserve.

### B.4 Nostr — silent on PQ

No NIP search result found for ML-DSA / hybrid-PQ / Kyber proposals as of May 2026. NIP-44 (encryption) uses XChaCha20-Poly1305 + secp256k1; no PQ extension proposed in the indexed corpus.

---

## Section C — TLS / messaging-protocol PQ deployments

### C.1 Cloudflare (production, scale-deployed)

Source: https://blog.cloudflare.com/pq-2025/

**KEM:** X25519MLKEM768 (hybrid). Quote:
> "Like many other early adopters, we like to play it safe and deploy a **hybrid** key-agreement [combining] X25519 and ML-KEM-768."

Scale: 43% of human-generated HTTPS traffic to Cloudflare uses X25519MLKEM768 (mid-September 2025).

**Signatures:** **NONE.** Quote:
> "We are in an interesting in-between time, where a lot of Internet traffic is protected by post-quantum key agreement, but not a single public post-quantum certificate is used."

> "Unless we can get performance much closer to today's authentication, we expect the vast majority to keep post-quantum authentication disabled."

Cloudflare is exploring **Merkle Tree Certificates** as the signature-side solution (trying to reduce ML-DSA-65's 1952-byte pubkey / 3309-byte signature overhead). Not deployed.

### C.2 Signal PQXDH (production since 2023)

Source: https://signal.org/docs/specifications/pqxdh/

**KEM:** Hybrid X25519 + CRYSTALS-Kyber-1024 (NOT migrated to ML-KEM as of the public spec).

**Signatures:** **XEdDSA only — classical.** Bob's PQ prekeys are signed with his EC identity key.

**Codepoint scheme:**
> "A single-byte constant representation of *curve* followed by little-endian encoding of the u-coordinate"
> "The single-byte representation of *curve* is defined by the implementer."

PQXDH does NOT use IANA codepoints. It uses **implementer-defined single-byte tags** with one constraint: "The ranges of all encoding functions must be pairwise disjoint." This is a **proprietary private codepoint scheme** — Signal does not coordinate codes with anyone.

### C.3 Apple iMessage PQ3 (production since iOS 17.4)

Source: https://security.apple.com/blog/imessage-pq3/

**KEM:** Hybrid Kyber-1024 + P-256 ECDH. (Note: Kyber-1024 not ML-KEM-768; Apple opted for higher security level.)

**Signatures:** **ECDSA P-256 only — classical.** Quote: "iMessage continues to rely on classical cryptographic algorithms to authenticate the sender."

**Wire-format codepoints:** Apple-internal protocol-version field. Not IANA-aligned. Quote: "the supported cryptographic protocol version" is signed by the Contact Key Verification account key but the document discloses no IANA registration.

### C.4 MLS (RFC 9420) PQ ciphersuites

Source: https://www.ietf.org/archive/id/draft-ietf-mls-pq-ciphersuites-04.html

9 proposed ciphersuites with TBD codepoints:

| TBD# | Name | Sig Algo |
|---|---|---|
| TBD1 | MLS_128_MLKEM768X25519_AES128GCM_SHA256_Ed25519 | **Ed25519** |
| TBD2 | MLS_128_MLKEM768X25519_AES256GCM_SHA384_Ed25519 | **Ed25519** |
| TBD3 | MLS_128_MLKEM768P256_AES128GCM_SHA256_P256 | **ECDSA P-256** |
| TBD4 | MLS_128_MLKEM768P256_AES256GCM_SHA384_P256 | **ECDSA P-256** |
| TBD5 | MLS_192_MLKEM1024P384_AES256GCM_SHA384_P384 | **ECDSA P-384** |
| TBD6 | MLS_128_MLKEM768_AES256GCM_SHA384_P256 | **ECDSA P-256** |
| TBD7 | MLS_192_MLKEM1024_AES256GCM_SHA384_P384 | **ECDSA P-384** |
| TBD8 | MLS_192_MLKEM768_AES256GCM_SHA384_MLDSA65 | ML-DSA-65 |
| TBD9 | MLS_256_MLKEM1024_AES256GCM_SHA384_MLDSA87 | ML-DSA-87 |

**7 of 9 use CLASSICAL signatures even with PQ KEM.** Only TBD8/TBD9 use ML-DSA — and notably, they use ML-DSA *alone*, not hybrid Ed25519+ML-DSA-65. The IETF MLS working group is not minting `Ed25519+ML-DSA-65` composite ciphersuites.

The MLS X-Wing variant (draft-mahy-mls-xwing-00) is `MLS_128_XWING_AES128GCM_SHA256_Ed25519` — also Ed25519 classical.

### C.5 Tor (proposal 355, exploratory)

Source: https://spec.torproject.org/proposals/355-revisiting-pq.html

Status: **exploratory, no decision.** Quote: "We won't develop any of these handshakes fully here; this proposal is about exploring our alternatives." Tor has multiple draft handshake types (PQ-TR, PQ-KEM-1/2, PQ-KEM-DSA-1/2) under analysis. Codepoints not assigned. The earlier ntor3 hybrid (proposal 269) does NOT integrate post-quantum DSA — only KEM hybrid + Curve25519 server auth.

### C.6 WhatsApp / Matrix / Wire / Threema

No public spec found documenting PQ-hybrid wire-format choices for these systems as of May 2026.

---

## Section D — Cryptographic library codepoint use

### D.1 rustls + aws-lc-rs (the iroh stack)

Source: https://docs.rs/rustls-post-quantum/

- `MLKEM768` (pure ML-KEM) exposed
- `X25519MLKEM768` (hybrid) exposed; moved INTO rustls core in 0.23.22 (no longer requires the post-quantum crate)
- `prefer-post-quantum` feature flag prioritizes PQ in negotiation
- ML-DSA support: **experimental, opt-in via `aws-lc-rs-unstable`** — "support for three variants of the experimental ML-DSA signature algorithm" — NOT shipped in production rustls

The library codepoint values match IANA TLS Named Group registry: `X25519MLKEM768 = 0x11EC`.

### D.2 BoringSSL (Chrome / Google)

- Chrome 116+ supported X25519Kyber768 experimentally (codepoint **0x6399**, private-use TLS Named Group range)
- Chrome 131+ default-enables X25519MLKEM768 (codepoint **0x11EC**, IANA-permanent)
- The 0x6399 → 0x11EC migration is the canonical example of "experimental private code → IANA permanent code" for hybrid PQ
- ML-DSA support exists in BoringSSL for testing but is NOT deployed in production Chrome TLS

### D.3 OpenSSL 3.5+

- Native X25519MLKEM768 support via TLS Named Group **0x11EC**
- ML-DSA signature support in the design phase (`openssl/doc/designs/ml-dsa.md`)
- No production deployment of hybrid SIGNATURES

### D.4 AWS-LC / aws-lc-rs

- FIPS-140-3-validated; **first open-source crypto module to include ML-KEM in FIPS validation**
- Exposes ML-DSA as "unstable" feature
- Iroh uses aws-lc-rs as its rustls provider for PQ support

---

## Section E — Convention crystallization assessment

### E.1 KEM-side convergence: STRONG

Every deployed system that ships PQ KEM uses **X25519MLKEM768** with IANA codepoint **0x11EC** (4588). Convergence is essentially complete on the TLS/QUIC/HPKE side. The few outliers (Apple Kyber-1024, Signal Kyber-1024) chose higher security levels but the same hybrid pattern.

X-Wing (HPKE codepoint 0x647A) is a parallel construction with different IANA assignment but identical algorithmic content (X25519 + ML-KEM-768) and a different combiner (SHA3-256 vs the TLS HKDF approach). X-Wing is the "general-purpose" composability winner; X25519MLKEM768 is the "TLS-ecosystem-deployed" winner. For Benten's S&C use (Iroh blobs / sendme / non-TLS contexts), **X-Wing at 0x647A is the better cited choice** — and CLAUDE.md §2026-05-21 night block already commits to it.

### E.2 Signature-side convergence: DELIBERATE NON-CONVERGENCE

The strongest signal in the dataset: **virtually no deployed system ships PQ-hybrid SIGNATURES today.** iroh, Cloudflare, Signal, Apple, MLS-default, Tor, libp2p — all keep signatures classical Ed25519 / ECDSA. Reasons:
1. **Performance overhead is large** (ML-DSA-65 = 1952-byte pubkey / 3309-byte signature vs Ed25519's 32/64)
2. **Cert/key infrastructure not ready** (no PQ public certificates anywhere; Cloudflare's quote)
3. **Industry consensus quote** (iroh): "no industry consensus yet which one to use"
4. **Composite signature standards still in draft** (LAMPS pq-composite-sigs-16 still under WG review)

Where composite signatures DO have codepoints (LAMPS X.509 OIDs `1.3.6.1.5.5.7.6.{37..54}`), no production system in the dataset uses them. They are X.509-cert-focused and gated on the broader PKI migration that hasn't happened.

### E.3 The codepoint-assignment pattern

For X25519MLKEM768:
1. **Experimental phase:** Chrome assigns 0x6399 (TLS private-use range 0xFE00-0xFEFF wasn't used; 0x6399 from the unassigned region)
2. **Stable phase:** IANA assigns 0x11EC; Chrome migrates 0x6399 → 0x11EC

For X-Wing:
- IETF draft requests 25722 / 0x647A (`25519 + 203`) as a memorable derived number — clever mnemonic
- Status: "(please)" — pending IANA, but the value is stable across draft versions and being used in implementations (`@hpke/hybridkem-x-wing` npm package)

For hybrid SIGNATURES (Ed25519+ML-DSA-65):
- LAMPS composite-sigs gives OID 1.3.6.1.5.5.7.6.48 for `id-MLDSA65-Ed25519-SHA512`
- **NO TLS Named Group / no HPKE / no multicodec / no COSE alg assigned** for this composite
- The composite is X.509-only; it has no wire-format codepoint in any non-X.509 protocol

### E.4 What "alignment" means for Benten

Benten's wire-format use cases are:
1. **Peer-identity-DID** → did:key over an iroh `EndpointId` (currently Ed25519; ratified per CLAUDE.md to stay Ed25519 for v1-beta with future PQ-hybrid swap matrix)
2. **Content-signature on Drops / Subgraph specs** → a varsig / multikey blob (not TLS, not X.509)
3. **Encryption envelope on per-chunk AEAD** → codepoint-dispatched suite (already committed to X-Wing 0x647A)

For case 1 (peer-identity) — iroh-aligned. Stay Ed25519 v1-beta, plan PQ-hybrid swap.

For case 3 (encryption) — X-Wing at 0x647A. Aligned with CLAUDE.md commitment + matches the HPKE/MLS-X-Wing draft.

For case 2 (content-signature hybrid) — **NO existing codepoint convention to align with.** The closest precedents are:
- LAMPS X.509 OIDs (wrong wire-format; OID encoding is X.509-ASN.1)
- Pure-PQ multicodec entries (0x1211 for ML-DSA-65; not hybrid)
- Veilid VLD0 four-char tag (entirely Veilid-internal)

The honest finding is: **for hybrid Ed25519+ML-DSA-65 SIGNATURE wire-format, Benten is structurally early.** No deployed P2P-content-addressed system has shipped this. The closest analog is multicodec's draft pure-PQ entries with the gap left open for hybrids.

---

## Synthesis — recommendations grounded in deployment evidence

### S1. **Align KEM-side wholesale with the IETF/iroh convergence.**

Benten's encryption envelope is already committed to X-Wing at HPKE codepoint **0x647A** per the CLAUDE.md §2026-05-21 night-block ratification. This is the right call — X-Wing is the cited choice for non-TLS, content-addressed, HPKE-ecosystem-friendly contexts; the same combiner shape is being pulled into MLS draft-mahy-mls-xwing-00. Don't drift from 0x647A. Aligns with iroh's transport-layer X25519MLKEM768 conceptually (same algorithm content), uses the composability-friendly framing.

### S2. **Mirror iroh's signature-side posture for v1-beta: classical Ed25519 default, PQ-hybrid documented but non-default-arm.**

Every shipped P2P system (iroh, libp2p, Signal, Apple, Veilid, Cloudflare TLS) keeps signatures classical. The deployment evidence does NOT support shipping hybrid Ed25519+ML-DSA-65 as v1-beta default for peer identity / content signatures. The shipping precedent is to ship PQ-KEM today + PQ-signature later when consensus emerges.

**This may be a meaningful reframe of CLAUDE.md's current "PQ-hybrid as v1-beta DEFAULT" position on the SIGNATURE side** — the 2026-05-19 RATIFIED-pq-default-reframe sets PQ-hybrid as default for BOTH signature and encryption. The encryption-side ratification is unambiguously supported by deployment evidence (HNDL-driven; un-retrofittable; iroh-style); the signature-side default-PQ position is OUT OF STEP with every shipped P2P/messaging system as of May 2026.

Frame this as a **surface-to-Ben architectural decision** (per `feedback_surface_arch_decisions_under_auth`): does the deployment-evidence picture (no shipped P2P signs hybrid PQ today) warrant softening the signature-side default to "classical Ed25519 with non-default hybrid arm built + ready to flip" — matching iroh's stance exactly? The encryption-side commitment stays unchanged. This would NOT compromise v1-beta shippability per the existing safety invariant (the classical Ed25519 half is the audited security floor; flipping the default is a config change once industry consensus emerges).

### S3. **For the hybrid-signature wire-format codepoint, do NOT wait for a non-existent convention; mint a NAMED Benten-internal varsig codepoint and document the planned migration path.**

For case 2 (content-signature hybrid), no convention exists to align with. Benten's choices:
- (a) propose a multicodec hybrid entry (Agent 1's framing)
- (b) compose two multikey blobs at the application layer (concat) and define a Benten-internal envelope tag
- (c) defer signature-hybrid wire-format until G-CORE-9 freeze (don't build the signature default-PQ flip into the wire-format pre-freeze if S2 is accepted)

If Ben holds the signature-default-PQ position, lean toward (b) — compose two multikey blobs with a Benten-internal envelope tag (4-byte magic header similar to Veilid's "VLD0" pattern). The envelope tag stays Benten-internal until/if multicodec adds hybrid entries; then we migrate the envelope to wrap the official codepoint. This avoids the "mint a permanent codepoint before there's deployment-convergence evidence" trap.

### S4. **The 0x647A precedent for self-derived mnemonic codepoints is worth following for any Benten-internal codepoint mints.**

X-Wing uses `0x647A = 25519 + 203` (the X25519-plus-ML-KEM-derivation as a mnemonic). The IANA cert-OID-style fully-qualified-numeric approach is wrong shape for content-addressed; the X-Wing-style mnemonic-derived codepoint inside a self-describing varint envelope is the right precedent. If Benten mints internal codepoints, use the same memorable-derivation pattern.

### S5. **Document the alignment matrix in Benten's S&C ratification doc.**

Specifically capture:
- KEM = X-Wing at HPKE 0x647A (iroh-adjacent; HPKE-aligned; X-Wing-MLS-draft-aligned)
- Peer-identity sig = Ed25519 (iroh-identical; libp2p-aligned; Signal/Apple/Cloudflare-aligned)
- Content sig = Ed25519 v1-beta default if S2 accepted; if hybrid retained, Benten-internal envelope tag + named migration path (no existing convention)

The doc carries the deployment-evidence rationale so future agents can see WHY each choice was made, not just WHAT was chosen.

---

## Anti-overlap addendum

- Agent 1 owns multicodec table mechanics + private-use convention + DID method analysis. This agent's contribution: the multicodec table **has zero hybrid composite entries** (confirmed via direct table.csv grep 2026-05-25) — which means Agent 1's private-use / proposed-PR options remain the entire landscape; there is no existing hybrid composite to align to.
- Agent 2 owns IETF/IANA/COSE/NIST standards-body machinery. This agent's contribution: every shipped deployment uses IETF-stable codepoints WHERE THEY EXIST (TLS 0x11EC) and uses implementer-defined or private codepoints WHERE THEY DON'T (Signal single-byte tags; Apple version field; Veilid four-char tag). The standards-body landscape Agent 2 covers is what enables the convergence Agent 3 observes.
- This agent owns: live deployments + their codepoint choices + ecosystem-convergence pattern.

---

## Sources

- iroh PQ post (2026-05-19): https://www.iroh.computer/blog/iroh-post-quantum-handshakes
- iroh 1.0.0-rc.0: https://www.iroh.computer/blog/iroh-1-0-0-rc-0
- iroh repo: https://github.com/n0-computer/iroh
- iroh noq blog: https://www.iroh.computer/blog/noq-announcement
- TLS hybrid ECDHE-MLKEM draft: https://www.ietf.org/archive/id/draft-ietf-tls-ecdhe-mlkem-04.html
- rustls-post-quantum docs: https://docs.rs/rustls-post-quantum/latest/rustls_post_quantum/
- Cloudflare PQ 2025 state: https://blog.cloudflare.com/pq-2025/
- Signal PQXDH spec: https://signal.org/docs/specifications/pqxdh/
- Apple iMessage PQ3: https://security.apple.com/blog/imessage-pq3/
- libp2p peer-ids spec: https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md
- Veilid cryptography: https://veilid.com/how-it-works/cryptography/
- MLS PQ ciphersuites draft: https://www.ietf.org/archive/id/draft-ietf-mls-pq-ciphersuites-04.html
- MLS X-Wing draft: https://www.ietf.org/archive/id/draft-mahy-mls-xwing-00.html
- Tor proposal 355: https://spec.torproject.org/proposals/355-revisiting-pq.html
- X-Wing KEM draft 09: https://www.ietf.org/archive/id/draft-connolly-cfrg-xwing-kem-09.html
- LAMPS composite ML-DSA: https://lamps-wg.github.io/draft-composite-sigs/draft-ietf-lamps-pq-composite-sigs.html
- Multicodec table (live grep 2026-05-25): https://github.com/multiformats/multicodec/blob/master/table.csv
- UCAN varsig (go-varsig): https://github.com/ucan-wg/go-varsig
- Chrome 0x6399 → 0x11EC migration: https://chromestatus.com/feature/5257822742249472
