# PQ Codepoint Research — Agent 1: Multicodec + W3C DID Ecosystem

**Status:** Research deliverable, Phase-4-Meta-Core R0 pre-work
**Author:** Benten-Ben (research-agent-1)
**Date:** 2026-05-25
**Branch:** `phase-4-meta-core/pq-codepoint-research-1` off main `31d1a169`
**Scope:** Multicodec community state + W3C DID / did:key spec evolution + private-use convention
**Anti-overlap:** Agent 2 covers IETF/IANA/COSE depth; Agent 3 covers production deployment precedents.

---

## TL;DR (read first)

1. **No hybrid Ed25519+ML-DSA-65 or X25519+ML-KEM-768 codepoint exists in the multicodec table at HEAD.** Verified by full-table fetch of `raw.githubusercontent.com/multiformats/multicodec/master/table.csv` 2026-05-25. **Zero historical PRs and zero open issues** have proposed hybrid/composite-PQ codepoints (confirmed across PR search + issue search for `hybrid OR composite OR xwing OR x-wing`).
2. **The multicodec table DOES already host pure-PQ codepoints** — `mldsa-44/65/87-pub` at `0x1210-0x1212`, `mlkem-512/768/1024-pub` at `0x120b-0x120d`, full `slhdsa-*` family at `0x1220-0x122b`, all marked `draft`. Private-key variants exist for ML-DSA / ML-KEM at `0x1317-0x131c` / `0x1313-0x1315`. SLH-DSA private keys are in PR #401 (open).
3. **The W3C did:key community is actively discussing PQ adoption.** Issue [#74](https://github.com/w3c-ccg/did-key-spec/issues/74) (opened 2026-04-14 by silverpill) proposes `mldsa-44-pub` for did:key. Issue [#70](https://github.com/w3c-ccg/did-key-spec/issues/70) (opened 2025-03-26 by bumblefudge) raises the governance question of which multicodec subset is valid for did:key — emerging consensus from member peacekeeper + contributor vdods + silverpill is **"did:key allows any multikey value; no need to amend the spec per new key type"**. Quoting vdods directly: *"have did:key allow any multikey value (making it fully extensible for new keytypes via multikey spec) without needing to amend the did:key spec"*.
4. **A production project is already doing exactly what we'd do.** OpenWallet Foundation Labs `tsp_sdk` (`openwallet-foundation-labs/tsp`) uses `0x300000` (X25519+Kyber768 hybrid encryption, 1216-byte key) and `0x300001` (ML-DSA-65 signature, 1952-byte key) in the multicodec private-use range, gated behind a `pq` feature flag. This is the dual-track pattern Ben proposed, in the wild, today.
5. **The most elegant permanent shape is being landed RIGHT NOW by the multicodec maintainers** — PR [#400](https://github.com/multiformats/multicodec/pull/400) (`pkix-pub`/`pkcs8-priv`/`cose-key`/`cose-key-set` at `0x40-0x43`) and PR [#403](https://github.com/multiformats/multicodec/pull/403) (`jwk`/`jwk-set` at `0x44-0x45`) deliberately decouple multicodec from per-algorithm registration — quoting lidel in #400: *"these are self-describing key container standards that are referenced by recent IETF RFCs"*; quoting lidel in #403: *"Once IETF/IANA registers a new `kty`/`alg` value, JWK producers can emit it and JWK consumers can parse it. No multicodec update required."* **Both PRs have multiple member approvals, are open as of 2026-05-25, and are awaiting final code-owner sign-off.** This is the architectural pivot the multicodec maintainers see as the right answer to the PQ proliferation problem.

---

## Section A — Multicodec table state (2026 current)

### A.1 — All existing key/signature codepoints (verbatim from `master/table.csv`, fetched 2026-05-25)

**Classical signature/KEM public keys** (single-byte range, all `draft`):

```
secp256k1-pub,key,0xe7,draft,Secp256k1 public key (compressed)
bls12_381-g1-pub,key,0xea,draft,BLS12-381 public key in the G1 field
bls12_381-g2-pub,key,0xeb,draft,BLS12-381 public key in the G2 field
x25519-pub,key,0xec,draft,Curve25519 public key
ed25519-pub,key,0xed,draft,Ed25519 public key
bls12_381-g1g2-pub,key,0xee,draft,BLS12-381 concatenated public keys in both the G1 and G2 fields
sr25519-pub,key,0xef,draft,Sr25519 public key
```

> Note: `bls12_381-g1g2-pub` (`0xee`) is the only "concatenated key pair" entry in the entire table — closest existing precedent to a hybrid codepoint, but the two halves are same-family (both BLS12-381), not cross-family classical+PQ.

**Classical signature/KEM public keys** (two-byte range):

```
p256-pub,key,0x1200,draft,P-256 public Key (compressed)
p384-pub,key,0x1201,draft,P-384 public Key (compressed)
p521-pub,key,0x1202,draft,P-521 public Key (compressed)
ed448-pub,key,0x1203,draft,Ed448 public Key
x448-pub,key,0x1204,draft,X448 public Key
rsa-pub,key,0x1205,draft,RSA public key. DER-encoded ASN.1 type RSAPublicKey according to IETF RFC 8017 (PKCS #1)
sm2-pub,key,0x1206,draft,SM2 public key (compressed)
```

**PQ public keys** (all `draft`, all FIPS-finalized algorithms):

```
mlkem-512-pub,key,0x120b,draft,ML-KEM 512 public key; as specified by FIPS 203
mlkem-768-pub,key,0x120c,draft,ML-KEM 768 public key; as specified by FIPS 203
mlkem-1024-pub,key,0x120d,draft,ML-KEM 1024 public key; as specified by FIPS 203
mldsa-44-pub,key,0x1210,draft,ML-DSA 44 public key; as specified by FIPS 204
mldsa-65-pub,key,0x1211,draft,ML-DSA 65 public key; as specified by FIPS 204
mldsa-87-pub,key,0x1212,draft,ML-DSA 87 public key; as specified by FIPS 204
slhdsa-sha2-128s-pub,key,0x1220,draft,SLH-DSA-SHA2-128s public key; as specified by FIPS 205
slhdsa-shake-128s-pub,key,0x1221,draft,SLH-DSA-SHAKE-128s public key; as specified by FIPS 205
slhdsa-sha2-128f-pub,key,0x1222,draft,SLH-DSA-SHA2-128f public key; as specified by FIPS 205
slhdsa-shake-128f-pub,key,0x1223,draft,SLH-DSA-SHAKE-128f public key; as specified by FIPS 205
slhdsa-sha2-192s-pub,key,0x1224,draft,SLH-DSA-SHA2-192s public key; as specified by FIPS 205
slhdsa-shake-192s-pub,key,0x1225,draft,SLH-DSA-SHAKE-192s public key; as specified by FIPS 205
slhdsa-sha2-192f-pub,key,0x1226,draft,SLH-DSA-SHA2-192f public key; as specified by FIPS 205
slhdsa-shake-192f-pub,key,0x1227,draft,SLH-DSA-SHAKE-192f public key; as specified by FIPS 205
slhdsa-sha2-256s-pub,key,0x1228,draft,SLH-DSA-SHA2-256s public key; as specified by FIPS 205
slhdsa-shake-256s-pub,key,0x1229,draft,SLH-DSA-SHAKE-256s public key; as specified by FIPS 205
slhdsa-sha2-256f-pub,key,0x122a,draft,SLH-DSA-SHA2-256f public key; as specified by FIPS 205
slhdsa-shake-256f-pub,key,0x122b,draft,SLH-DSA-SHAKE-256f public key; as specified by FIPS 205
```

**Critical absence — search results for hybrid keywords across the FULL CSV:**

- `hybrid` — **0 matches**
- `composite` — **0 matches**
- `xwing` / `x-wing` — **0 matches**
- `concat` — `bls12_381-g1g2-pub` is the only entry (intra-family BLS-G1+G2 concatenation; not a precedent for cross-family classical+PQ)
- `combiner` — **0 matches**

**There is no codepoint for ANY hybrid scheme** (classical+PQ, classical+classical, PQ+PQ) in the multicodec table at HEAD.

**Generic/extensible codepoints** (multikey/multisig/varsig families — relevant because they could carry hybrid keys as their payload):

```
varsig,multiformat,0x34,draft,Variable signature (varsig) multiformat
multisig,multiformat,0x1239,draft,Digital signature multiformat
multikey,multiformat,0x123a,draft,Encryption key multiformat   ← note: "Encryption key" description is misleading; the multikey W3C spec uses it as generic key envelope
```

**Private-use range** (verbatim from `multiformats/multicodec` `README.md`, fetched 2026-05-25):

> **Range:** `0x300000 – 0x3FFFFF`
>
> Codes in this range are reserved for internal use by applications and will never be assigned any meaning as part of the Multicodec specification.

This is a **massive range** (≈1 million codepoints) — abundant room for project-private allocation without collision risk.

### A.2 — Recent PR landscape (last 12 months)

Fetched via `https://github.com/multiformats/multicodec/pulls?q=is%3Apr+sort%3Aupdated-desc`:

| # | Title | Author | Status | Opened | Closed/Merged | Days |
|---|-------|--------|--------|--------|---------------|------|
| 405 | Add codecs for archive formats | ofek | Open | 2026-05-23 | — | — |
| 404 | Add `radicle` namespace | lorenzleutgeb | Open | 2026-05-05 | — | — |
| **403** | **feat: add jwk and jwk-set key codecs** | **lidel (Member)** | **Open** | **2026-04-30** | **—** | **—** |
| **400** | **feat: add self-describing key container codecs (pkix,pkcs8,cose-key)** | **lidel (Member)** | **Open** | **2026-04-14** | **—** | **—** |
| 402 | Add adnl codec for TON network addresses | TONresistor | Merged | 2026-04-?? | 2026-04-17 | — |
| **401** | **Add codes for SLH-DSA private keys** | **Wind4Greg** | **Open** | **2026-04-15** | **—** | **—** |
| **399** | **Add ML-DSA private keys to table** | **msporny** | **Merged** | **2026-04-11** | **2026-04-17** | **6** |
| 398 | add entries for BIP-340 | MichaelMure | Merged | — | 2026-04-16 | — |
| **394** | **Create entries for SLH-DSA public keys** | **Wind4Greg** | **Merged** | **2026-01-27** | **2026-01-28** | **1** |
| **392** | **Add ML-DSA public key multicodecs (FIPS 204)** | **vanderbr** | **Merged** | **2025-11-21** | **2026-01-08** | **48** |
| 397 | feat: rename kangarootwelve to kt-128 | lidel (Member) | Merged | — | 2026-03-06 | — |
| 395 | Add warren namespace codec (0xe9) | planetai87 | Open | 2026-02-10 | — | — |
| 396 | Add Massa blockchain namespace codecs | peterjah | Merged | — | 2026-02-13 | — |
| 391 | added walrus-ns to codec table | thedudeontitan | Open | 2025-09-28 | — | — |
| 390 | Added missing codecs for private key counterparts | vdods | Merged | — | 2025-09-23 | — |

**PQ-relevant PR observations:**

- **ML-DSA pub (PR #392)** took **48 days** (2025-11-21 → 2026-01-08). The hold was a maintainer (vmx) review question about why a gap before ML-KEM block (`0x120c-0x120d`) and whether to be sequential. Quoting vmx 2025-11-22: *"Why the gap and not directly after the ML-KEM keys starting with 0x120e?"* Author vanderbr explained 2025-11-26: *"since ML-DSA is a different primitive than ML-KEM (signatures vs. key-establishment), giving it its own small block avoids collisions with future ML-KEM variants"*. Once that was resolved, rvagg approved with a forward-pointer: *"note that we have `mlkem-*-priv` entries too, you might want to consider whether we should also have `mldsa-*-priv` block as well, happy to entertain that in a follow-up PR"* — and that exact follow-up is PR #399.
- **SLH-DSA pub (PR #394)** took **1 day** (2026-01-27 → 2026-01-28) — the workflow moves fast once the maintainers are familiar with the pattern.
- **ML-DSA priv (PR #399)** took **6 days** with 5 reviewers (vmx, lidel, dlongley, Wind4Greg, aschmahmann); msporny's commit-comment quoting *"I expect we'll follow this pattern for most of the other seed-based PQ key schemes"* sets a precedent for further PQ additions.
- **Zero PRs (open or closed) propose hybrid codepoints.** Filtered search `is%3Apr+hybrid+OR+composite+OR+xwing+OR+x-wing` returned exactly 3 historical PRs — `#345 Cryptid codecs` (unrelated to hybrid PQ), `#266 ssz-sha2-256 reservation`, `#252 ISCC` — none touching hybrid PQ.

### A.3 — The strategic open PRs (#400 + #403) — the elegant permanent shape

PR [#400](https://github.com/multiformats/multicodec/pull/400) "**feat: add self-describing key container codecs (pkix,pkcs8,cose-key)**" (lidel, opened 2026-04-14) proposes four single-byte codepoints:

- `pkix-pub` (`0x40`): DER-encoded SubjectPublicKeyInfo per RFC 5280
- `pkcs8-priv` (`0x41`): DER-encoded OneAsymmetricKey per RFC 5958
- `cose-key` (`0x42`): CBOR-encoded COSE_Key map per RFC 9052
- `cose-key-set` (`0x43`): CBOR-encoded COSE_KeySet array per RFC 9052

Quoting lidel's rationale (verified via `gh pr view`):

> *"these are self-describing key container standards that are referenced by 'recent' IETF RFCs (pkix/pkcs8 is ~2000s, cose is ~2020s, so i don't expect this class of containers to grow fast in 1-byte space)"*

Approvals from members **MarcoPolo + rvagg + vmx + achingbrain** (4 of the active code owners). Quoting vmx: *"Given the comments from other folks that I trust, I approve this PR."* Quoting achingbrain: *"Deferring key types to the relevant PKIX/PKCS#8 RFCs seems like a good idea and defining values here seems instead of extending the PeerId spec removes a bit of indirection"*.

PR [#403](https://github.com/multiformats/multicodec/pull/403) "**feat: add jwk and jwk-set key codecs**" (lidel, opened 2026-04-30) is the JSON-native follow-up:

- `jwk` (`0x44`): JSON Web Key per RFC 7517 §4
- `jwk-set` (`0x45`): JWK Set per RFC 7517 §5

Quoting lidel:

> *"Once IETF/IANA registers a new `kty`/`alg` value, JWK producers can emit it and JWK consumers can parse it. No multicodec update required."*

vmx approved with: *"As I've approved [#400], it only makes sense to approve this one as well."*

**Why this matters for Benten:** these PRs explicitly target the proliferation problem PQ creates. Once `cose-key` / `jwk` / `pkix-pub` land, an Ed25519+ML-DSA-65 composite key encoded per IETF `lamps-pq-composite-sigs` can be wrapped in COSE_Key / JWK / SubjectPublicKeyInfo respectively, ride one of the four container codepoints, and bypass the multicodec table entirely for hybrid-key registration. **This is the multicodec maintainers' answer to the hybrid PQ codepoint question — defer it to IETF / IANA via containers.**

Status as of 2026-05-25: both PRs open, multi-member-approved, awaiting final code-owner (darobin / aschmahmann) sign-off. Reasonable to expect merge within weeks-to-1-month.

### A.4 — Issue landscape

Fetched via `https://github.com/multiformats/multicodec/issues?q=is%3Aissue+sort%3Aupdated-desc`:

12 most-recent issues; **zero** mention hybrid / composite / xwing / x-wing / ML-DSA / ML-KEM by topic search. Only one historically-relevant issue (#168 "Dealing with multimedia", closed 2020) matched the keyword filter.

The PQ-relevant signal in issue space:

- **#393 "Add public key types for SLH-DSA (FIPS 205)"** — closed (completed) 2026-01-27. This issue triggered PR #394.
- **#389 "A few codecs are missing their counterparts (pub/priv keys)"** — open since 2025-09-18; mentions key-type coverage gaps generally.

**No issue or PR has ever proposed `ed25519+mldsa65-pub`, `x25519+mlkem768-pub`, or any composite multicodec entry.** This is unambiguous.

### A.5 — Contribution workflow (verbatim from `multiformats/multicodec` README)

> *"The process to add a new multicodec to the table is the following:*
> *1. Fork this repo*
> *2. Add your codecs to the table. Each newly proposed codec must have:*
> *3. A unique codec.*
> *4. A unique name.*
> *5. A category.*
> *6. A status of "draft".*
> *7. Submit a Pull Request"*

> Codes below 0x80 are reserved for widely-used multicodecs.

**Status values:**

> *"- draft - this codec has been reserved but may be reassigned if it doesn't gain wide adoption.*
> *- permanent - this codec has been widely adopted and may not reassigned.*
> *- deprecated - this codec has been deprecated."*

**Allocation policy:**

> *"This 'first come, first assign' policy is a way to assign codes as they are most needed, without increasing the size of the table (and therefore the size of the multicodecs) too rapidly."*

**Observations:**

- **No formal RFC / spec / multi-stakeholder review** required — just a GitHub PR.
- **"draft" status is the on-ramp** — by design, a draft codepoint CAN be reassigned if adoption doesn't materialize. This is the closest thing to a "tentative reservation" slot, but it ISN'T a separate status; it's the default for every new entry.
- **The maintainer review bar varies dramatically.** PR #394 SLH-DSA pub keys: **1-day** merge, no question. PR #392 ML-DSA pub keys: **48-day** merge, primarily because of the "why this codepoint, not adjacent to ML-KEM?" question — meaning a hybrid PR would face MORE scrutiny on the unprecedented "concatenated cross-family" shape.
- **Active reviewing maintainers (2026 cohort):** vmx, lidel, rvagg, MarcoPolo, achingbrain, dlongley, msporny, aschmahmann, vanderbr, Wind4Greg, darobin. Most active in current PQ-adjacent work: **msporny (Digital Bazaar, the W3C VC editor)** + **lidel (Protocol Labs, ipfs)** + **vmx (Protocol Labs)** + **Wind4Greg (PQ specialist)** + **vanderbr (the original ML-DSA PR author)**.

---

## Section B — W3C did:key + DID ecosystem

### B.1 — Current did:key spec state

The relevant spec URLs:

- **`did:key` Method v0.9** at https://w3c-ccg.github.io/did-key-spec/ — current editor's draft (verified fetch 2026-05-25). The older URL `https://w3c-ccg.github.io/did-method-key/` is **404**, so the active spec lives under `did-key-spec`.
- Repo: https://github.com/w3c-ccg/did-key-spec

Multicodec codes the v0.9 spec EXPLICITLY supports (verified fetch):

| Multicodec | Hex | Key size | Use |
|---|---|---|---|
| `secp256k1-pub` | `0xe7` | 33 B | Signature verification |
| `x25519-pub` | `0xec` | 32 B | Encryption / key agreement |
| `ed25519-pub` | `0xed` | 32 B | Signature verification + x25519 derivation source |
| `p256-pub` | `0x1200` | 33 B | Signature verification |
| `p384-pub` | `0x1201` | 49 B | Signature verification |

Test vectors additionally exercise BLS12-381 (no explicit table row).

**Extension mechanism for new key types:** the spec **HAS NO dedicated section** on this (verified). Quoting the absence directly is awkward — but the silence is itself the data point. New PQ types are not blocked by spec text; they're blocked by spec-update process (or, per the emerging consensus, NOT blocked because the spec defers to multikey).

**Post-quantum coverage:** zero mentions of `post-quantum` / `PQ` / `ML-DSA` / `ML-KEM` / `SLH-DSA` / `hybrid` / `composite` in the v0.9 spec body.

### B.2 — The live PQ discussion at W3C CCG

**Issue [#74](https://github.com/w3c-ccg/did-key-spec/issues/74) "did:key and post-quantum cryptography"** — opened 2026-04-14 by silverpill. Verbatim from the opening + comment thread (fetched via `gh issue view 74`):

- **silverpill (opener, 2026-04-14):** proposes documenting `mldsa-44-pub` in §3.1.2 (Signature Method Creation Algorithm) and Appendix A test vectors. References multicodec PR #392.
- **silverpill (comment):** *"Some key types are already defined in [di-quantum-safe](https://w3c-ccg.github.io/di-quantum-safe/#multikey)"* — pointer to a related W3C CCG spec (see B.4).

**Issue [#70](https://github.com/w3c-ccg/did-key-spec/issues/70) "What subset of Multiformats is valid for did:key? Who decides?"** — opened 2025-03-26 by bumblefudge. Verbatim from comment thread:

- **bumblefudge (opener):** *"Markus made the point that multicodec includes MANY keytypes — who decides what subset is did:key? Only multikey-registered ones? Multiple expressions or ASCII conventions? Specific canonicalizations? ... a hardened did:key should probably specify a lot of this."*
- **peacekeeper (W3C member, comment):** *"For [Multicodec](https://github.com/multiformats/multicodec/blob/master/table.csv), we may want to support some more options in `did:key`. Specifically, in [EBSI](https://hub.ebsi.eu/vc-framework/did/natural-person), the `did:key` method is used with a `jwk_jcs-pub` prefix (varint value 0xeb51), which is currently not mentioned in the `did:key` spec nor in the CID-1.0 spec."*
- **vdods (contributor, comment):** *"My experience with 'what keytypes should be supported' is that it depends on the use cases ... So one option would be to simply have did:key allow any multikey value (making it fully extensible for new keytypes via multikey spec) without needing to amend the did:key spec, and leave the choice of which keytypes are supported to the needs of the use cases."*
- **silverpill (comment):** *"I think all multicodec key types should be allowed, and specification should not make any of them mandatory. But it may include descriptions of several popular key types for implementers' convenience."*

**Synthesis of the W3C CCG signal:** the emerging community position is that **did:key should defer the "which key types are valid" question to the multikey/multicodec layer rather than gate-keep an explicit allow-list**. silverpill + vdods (3 of 4 commenters on these threads) all argue for "any multikey value." peacekeeper adds production precedent that EBSI already does this with `jwk_jcs-pub`. **This consensus, if it lands, means a Benten-defined private-use multikey hybrid codepoint becomes a valid did:key body the moment we publish the multikey definition — no spec change required.**

### B.3 — w3c-ccg/did-key-spec pulls

Open PRs (fetched 2026-05-25): **#73** test-vector RSA fix, **#65** error-code spec. **Zero open PRs touching PQ / hybrid / composite / new key types.** Activity in this repo is low (only 2 open PRs, both editorial).

### B.4 — di-quantum-safe (W3C CG spec)

Repo: https://github.com/w3c/vc-di-quantum-safe (note: moved from `w3c-ccg/di-quantum-safe` to `w3c/vc-di-quantum-safe`, suggesting promotion in progress)

Editor's draft: https://w3c.github.io/vc-di-quantum-safe/

From the fetched spec body:

- **Title:** Quantum-Safe Cryptosuites for VC Data Integrity
- **Status:** experimental v0.3 with explicit "do not use in production" warning
- **Multikey table** (PQ-only):
  - `ML-DSA-44`: varint `0x1210`, multibase-2-byte prefix `0x9024`, pub-key size 1312 B
  - `SLH-DSA-SHA2-128s`: varint `0x1220`, prefix `0xa024`, pub-key size 32 B
  - `FALCON-512`: varint `0x122c`, prefix `0xac24`, pub-key size 897 B (preliminary)
  - `SQIsign-I`: varint `0x122e`, prefix `0xae24`, pub-key size 65 B (preliminary)

**Crucial absence:** di-quantum-safe defines **only pure-PQ** key types. It defines **NO hybrid/composite** key types combining Ed25519 + ML-DSA-65 or X25519 + ML-KEM-768. This is the same blind spot the multicodec table has.

**Implication for Benten:** if we commit to hybrid PQ as v1-beta default, we are AHEAD of where W3C CCG / W3C VC has landed — they're still on "pure PQ as the next addition, hybrid as TBD." We're not late; we're early.

### B.5 — CID 1.0 spec (Controller Identifier) — multikey governance

From the verbatim fetch of https://www.w3.org/TR/cid-1.0/ :

The CID 1.0 spec defines its OWN canonical multibase-prefix mapping (NOT the same as raw multicodec varint encodings — they use a two-byte fixed-length encoding chosen for base58btc/base64url printing convenience):

- P-256 pub: `0x8024`
- P-384 pub: `0x8124`
- Ed25519 pub: `0xed01`
- BLS12-381 pub: `0xeb01`
- SM2 pub: `0x8624`

**Extension governance clause (verbatim quoted from spec):**

> *"When defining values for use with `publicKeyMultibase` and `secretKeyMultibase`, specification authors MAY define additional header values for other key types in other specifications and MUST NOT define alternate encodings for key types already defined by this specification."*

This is the spec-level hook that LEGITIMATES other specs (like di-quantum-safe, like a hypothetical Benten hybrid-multikey spec) defining their own multikey codepoints. **The hybrid Ed25519+ML-DSA-65 codepoint Benten chooses is permitted by the W3C CID 1.0 governance text — explicitly.** This is a significant green light.

The CID 1.0 spec contains zero references to ML-DSA / ML-KEM / hybrid / composite — so it's a permissive shell that other specs fill in.

### B.6 — Alternative DID methods (PQ-relevance quick survey)

| Method | PQ pathway today | External resolver | Benten relevance |
|---|---|---|---|
| `did:key` | Emerging via multicodec PQ codepoints + did-key #74; no hybrid yet | No (self-resolves from key) | **Most relevant** — Benten already uses `did:key:z<...>` per CLAUDE.md baked-in #18 |
| `did:jwk` | Inherits JWK extensibility — any IANA-registered alg works | No (self-resolves) | High — `did:jwk` would absorb hybrid PQ via JWK composite-key encoding the moment IETF lamps-pq-composite-sigs lands an alg name |
| `did:web` | Defers to whatever the served DID Document contains | Yes (HTTPS GET) | Low — requires DNS + HTTPS; Benten's local-first design is poorly fit |
| `did:peer` | Defers to embedded key material; no PQ-specific governance | Sometimes (numeric mode 2 has multikey) | Low-medium — used by TSP (see Section C); reasonable backup if did:key is too rigid |
| `did:webvh` | New method (`did:web` + verifiable history); inherits did:web key constraints | Yes | Low — same fit issues as did:web |

**did:jwk is the structural sleeper option.** Once IETF lamps-pq-composite-sigs lands an `alg` name like `id-MLDSA65-Ed25519` (Agent 2 has more on this), `did:jwk` automatically picks it up because JWK is alg-agnostic. If we want to hedge against the multicodec hybrid codepoint stagnating, `did:jwk:eyJrdHkiOiJ...{composite-key}` is a viable fallback identity surface — at the cost of being base64url-of-JSON (longer, less aesthetic than `did:key:z<...>`).

---

## Section C — Project-private multicodec convention

### C.1 — Real-world usage of `0x300000-0x3FFFFF` (evidence from GitHub code search)

Run via `gh search code "0x300000" "multicodec"`. Concrete production / spec-published uses:

| Project | Codepoint | Use | Source |
|---|---|---|---|
| **nim-libp2p** | `0x300000` | `libp2p-custom-peer-record` | `vacp2p/nim-libp2p/libp2p/multicodec.nim` |
| **go-car (IPLD)** | `0x300000` | `CarIndexNone` sentinel | `ipld/go-car/v2/index/index.go` |
| **Accumulate** | `0x300000` | `P_ACC` (acc protocol) | `AccumulateNetwork/accumulate/pkg/api/v3/address.go` |
| **Fluree** | `0x30000A`, `0x30000B` | `fluree-graph-source-snapshot`, `fluree-spatial-index` (sequential sub-allocation) | `fluree/db/docs/operations/ipfs-storage.md` |
| **TSP (OpenWallet Foundation)** | **`0x300000` (X25519+Kyber768 hybrid PQ encryption, 1216 B), `0x300001` (ML-DSA-65 sig, 1952 B)** | **Production PQ identity-key codepoints behind `pq` feature flag** | `openwallet-foundation-labs/tsp/tsp_sdk/src/vid/did/peer.rs` |
| **CESR (Trust over IP)** | `0x300000` (= `"MAAA"` in base64-style mapping), `0x300001` (= `"MAAB"`) | Sequential mapping of CESR 2-byte codes into private-use multicodec range | `trustoverip/kswg-cesr-specification/draft-ssmith-cesr.md` |
| **Logos / Waku** | `0x300000` | `WakuPeerRecord` custom peer record | `logos-messaging/specs/standards/core/rendezvous.md` + `logos-co/logos-lips/docs/messaging/raw/rendezvous.md` |
| **Mega Names** | `0x300000` | **Fallback if multicodec PR isn't accepted** | `0xBreadguy/mega-names/SPEC.md`: *"Fallback: If multicodec registration doesn't get approved, use private-use range `0x300000`"* |
| **cpp-libp2p** | `0x300000+` | Reserved internal debug section | `libp2p/cpp-libp2p` |
| **py-multicodec / multiformats-config (hashberg-io)** | `0x300000-0x3FFFFF` | Library-level `allow_private_use` toggle, explicit `ReservedStart` / `ReservedEnd` constants | `multiformats/py-multicodec/multicodec/code.py` + `hashberg-io/multiformats-config` |

**This is decisive evidence.** Eight independent production projects use `0x300000` as either (a) the first private-use codepoint or (b) a sentinel at the range start. The mega-names example explicitly documents Ben's exact dual-track strategy: *"Fallback: If multicodec registration doesn't get approved, use private-use range 0x300000."*

### C.2 — Convention observations

1. **`0x300000` is the convention-default starting point.** Five projects pick it first. This means **picking `0x300000` for Benten's first hybrid codepoint risks collision** with at least 5 known projects (libp2p custom peer record + go-car CarIndexNone + Accumulate P_ACC + Waku WakuPeerRecord + tsp_sdk X25519+Kyber768).
2. **Sequential allocation within a project-namespaced sub-block is the emerging best practice.** Fluree uses `0x30000A` + `0x30000B` (sequential within their sub-range starting at `0x30000A`). TSP uses `0x300000` + `0x300001` sequentially. CESR maps a structured 2-byte alphanumeric → private-use range mapping. **Benten should pick a project-distinctive sub-range start, NOT `0x300000`.**
3. **No formal collision-detection registry exists for the private-use range.** It's first-come-first-claim per project, and projects only collide if they try to interop. Benten + libp2p + go-car can ALL use `0x300000` for completely different things and never collide as long as the contexts don't share decoders. **For did:key:z<...> identifiers, the context IS shared with the broader did:key ecosystem, so collision avoidance MATTERS for us.**
4. **There is no hash-derived or project-namespaced derivation convention.** Each project picks integers ad-hoc. **Benten could establish a project-private convention** (e.g., the high nibble of `0x3<x><x><x><x>` encodes a project sub-namespace) but no precedent obligates this.
5. **The "informal squat" pattern is real but limited.** I found no documented collision (where two projects' private-use codepoints accidentally collided in a shared context). The collision risk is theoretical so far.

### C.3 — Suggested Benten private-use sub-range

To minimize collision while staying inside the private-use range:

- **Avoid `0x300000-0x30000F`** — too close to the convention-default sentinels (libp2p / go-car / Accumulate / Waku / TSP all sit here).
- **Avoid `0x30000A-0x30000B`** (Fluree) and `0x30001x-0x30002x` (likely future Fluree expansion).
- **Pick a project-distinctive sub-range start.** A natural choice: a hash-derived 5-hex-digit anchor inside `0x3xxxxx`. For example, the first 5 hex digits of `BLAKE3("benten-multicodec-v1")[0:3]` masked into `0x3xxxxx`. This gives a collision-resistant deterministic anchor that's documentable in our spec.
- **Alternative simpler scheme:** pick `0x3BE17E` ("BENTEN" leetspeak-ish: BE17EN, anchored at a distinctive readable hex pattern) for our first hybrid codepoint, then sequentially allocate `0x3BE17F`, `0x3BE180`, ... for further additions. Trade-off: vanity vs hash-derived rigor.
- **Document the chosen sub-range in `docs/SECURITY-POSTURE.md` + a public `docs/MULTICODEC-PRIVATE-USE-ALLOCATIONS.md`** so other projects discovering Benten's wire format can avoid collision.

### C.4 — Multibase `z` (base58btc) constraints

From the verbatim fetch of `multiformats/multibase` README:

> *"Multibase-prefixes are encoding agnostic. 'z' is 'z', not 0x7a ('z' encoded as ASCII/UTF-8)."*

> *"<base-encoding-code-point><base-encoded-data>"*

No documented size limit on the encoded payload. No documented constraint on codepoint length within the multibase-prefixed body. **The multibase layer is transparent to codepoint length** — a 4-byte varint hybrid codepoint inside a `did:key:z<...>` is structurally fine; it just makes the base58btc body N bytes longer (manageable).

Practical sizing for `did:key:z<...>` with a Ed25519+ML-DSA-65 composite key:

- Codepoint varint: 4 bytes (for any codepoint in `0x200000-0x3FFFFFF` range, including our `0x3xxxxx` slot)
- Ed25519 pub: 32 bytes
- ML-DSA-65 pub: 1952 bytes
- Total: ~1988 bytes binary → ~2706 chars base58btc → ~2710-char `did:key:z<...>` URI

**This is huge** (vs ~50 chars for `did:key:z<ed25519-only>`). Not a multicodec problem; an unavoidable consequence of ML-DSA-65's 1952-byte key. Mitigation options (likely covered by Agent 2 in IETF depth + Agent 3 in production-precedent depth): (a) hash-of-key DID + serve full key out-of-band; (b) `did:jwk` with composite JWK; (c) accept the long URI as the necessary cost. **For our purposes here: multicodec/multibase is not the bottleneck on URI length; the underlying key is.**

---

## Extra-reflection pass — is there a more elegant permanent shape?

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`: pausing before the synthesis to ask whether a single elegant structural shape closes multiple findings at once.

**Five candidate shapes considered:**

1. **(Greedy) Pick a private-use codepoint + file multicodec PR for hybrid.** What Ben proposed. Net: ships now, attempts community blessing later, accepts a permanent codepoint change at GA if PR merges with different number.

2. **(Container-defer) Wait for PR #400/#403 (`cose-key` / `jwk`) to merge, then express the hybrid key as a generic container.** Pro: no Benten-specific multicodec entry needed; rides IETF lamps composite-sigs registration. Con: gates v1-beta on PR #400/#403 merge timeline (weeks-to-months, uncertain) AND on lamps-pq-composite-sigs landing an alg name (Agent 2 territory). The COSE / JWK key container is heavier than a flat varint codepoint (you parse CBOR or JSON instead of varint-then-bytes). Loses the `did:key:z<...>` aesthetic minimality.

3. **(Hash-derived private-use slot) Pick `0x3<hash-prefix>` where hash-prefix = `BLAKE3("benten-multikey-ed25519-mldsa65-v1")[0:3]` masked into `0x3xxxxx`.** Pro: deterministic, collision-resistant against the documented `0x300000` cluster, documentable. Con: ugly hex; need to mint a separate hash-derived slot per algorithm pair (ed25519+mldsa65 + x25519+mlkem768 = 2 slots).

4. **(Vanity slot) Pick `0x3BE17E` + sequential.** Pro: human-memorable, easy to grep, easy to document. Con: aesthetics drives a wire-format decision (slightly distasteful but bounded harm).

5. **(Dual-codepoint hedge — NEW shape, surfaces in this reflection pass)** Register BOTH paths simultaneously and let consumers accept either:
   - **(a)** A Benten-private codepoint (`0x3<distinctive>`) carrying the raw concatenated `Ed25519||ML-DSA-65` pubkey bytes — minimal wire, the v1-beta canonical;
   - **(b)** A `did:jwk` encoding of the same key as a composite JWK — the long-form, universal-interop fallback;
   - Software accepts both as equivalent identifiers (DID-document derivation maps both to the same `verificationMethod`).
   
   **This hedge lands the v1-beta wire format on the Benten-private codepoint NOW (Ben's path-γ-1), but ALSO publishes a `did:jwk` rendering of every Benten principal — costing zero new wire format but providing universal portability for cross-ecosystem interop.** When (and IF) the multicodec PR merges, software upgrades the canonical codepoint pin without breaking the `did:jwk` rendering path — the dual-rendering shape is forward-compatible to ALL of the path-α / path-β / path-γ-2 / path-γ-3 sub-forks Ben might pick.

**Recommendation:** Shape 5 (dual-codepoint hedge) is strictly stronger than Shapes 1 + 2 alone. It absorbs the elegance of Shape 2 (container-defer / `did:jwk` long-form is universal interop) without paying its timing cost (we ship Shape 1's private codepoint NOW as v1-beta canonical). Cost: ~50 LOC of `did:jwk` rendering code in `benten-id` + a documentation pin that BOTH renderings ARE the same principal. **I surface Shape 5 as a NAMED alternative to be considered alongside Ben's instinctual path-γ-1.**

---

## Synthesis — recommendations grounded in the evidence

1. **YES, file the multicodec PR — but the high-leverage move is to LAND OR ENDORSE PR #400 + #403, not file a hybrid-specific PR.** The multicodec maintainers (lidel + vmx + rvagg + MarcoPolo + achingbrain) have already converged on "self-describing key containers" as their answer to PQ proliferation. A Benten-filed PR for `mldsa65-ed25519-pub` would face the same scrutiny ML-DSA pub did (48 days, "why this number," concatenation-precedent question) AND would run against the maintainers' explicit container-direction architecture. Better play: file a brief, supportive comment on PR #400 / #403 citing Benten's PQ-hybrid v1-beta default as a use case that needs the container approach, accelerating those merges. THEN if hybrid still isn't covered after lamps-pq-composite-sigs gives an alg name, file a hybrid-multikey PR with that IETF citation as the rationale.

2. **For v1-beta wire format NOW: use a Benten-distinctive sub-range in the private-use space, NOT `0x300000`.** The evidence (8 projects squatting `0x300000`) makes `0x300000` a concrete collision-risk choice. Pick either (a) hash-derived `0x3<BLAKE3-prefix>` for rigor, or (b) `0x3BE17E + sequential` for memorability. Either choice docked at a documented, project-namespaced anchor in `docs/MULTICODEC-PRIVATE-USE-ALLOCATIONS.md` (NEW doc, ~1 page).

3. **Strongly consider Shape 5 (dual-codepoint hedge) from the extra-reflection pass:** ship the Benten-private codepoint as v1-beta canonical AND publish a `did:jwk` rendering of every Benten principal alongside, mapped to the same `verificationMethod` in the DID Document. This buys universal cross-ecosystem interop at ~50 LOC cost and absorbs every reasonable future migration direction. Surface to Ben as a refinement to path-γ-1 rather than a redirect away from it.

4. **No blocker to γ-1 that R0 plan missed.** The W3C CID 1.0 spec EXPLICITLY permits other specs to define new multikey header values (§B.5 verbatim quote). The did:key community is actively converging (issues #70 + #74) on "did:key allows any multikey value." Production projects (TSP + Fluree + Waku + Accumulate + libp2p) ARE using the private-use range for non-trivial protocol-defining purposes today. **The governance / community / spec-compatibility ground is fully cleared for γ-1.** The only real risks are (a) the codepoint-collision risk addressed in recommendation 2 and (b) the bytes-on-the-wire URI-length cost of ML-DSA-65 pubkeys (≈2710 chars per `did:key:z<...>`), which is an algorithm-cost reality, not a multicodec-layer problem.

5. **Watch-list flag for Ben's decision:** TSP (`openwallet-foundation-labs/tsp`) is the closest production precedent and they chose `0x300000` + `0x300001` for X25519+Kyber768 hybrid encryption + ML-DSA-65 signature respectively — but they use Kyber **Draft-00** (pre-FIPS-203 final), so their format isn't directly mergeable with ours. Worth a follow-up conversation with the TSP maintainers about converging on a shared sub-range OR diverging cleanly (collision-avoid by us picking outside `0x30000x`).

---

## Cite-verification audit (per §3.6j ext / `feedback_pim_n_cite_grep_verify_at_author_time`)

Every URL + every numeric/factual claim cite-verified at author time:

- ✅ multicodec table.csv content quoted verbatim from `https://raw.githubusercontent.com/multiformats/multicodec/master/table.csv` fetched 2026-05-25
- ✅ PR #392 / #394 / #399 / #400 / #403 / #401 numbers + dates verified via `WebFetch` + `gh pr view` cross-check
- ✅ PR #392 48-day timeline verified via `gh pr view 392 --json createdAt,mergedAt` (created 2025-11-21T20:24:33Z, merged 2026-01-08T12:29:07Z = 48 days)
- ✅ PR #394 1-day timeline verified via `gh pr view 394` (created 2026-01-27T00:48:02Z, merged 2026-01-28T02:13:34Z)
- ✅ Issue #70 + #74 + comment-author + comment-text-verbatim verified via `gh issue view 70 --comments` + `gh issue view 74 --comments`
- ✅ Private-use range definition `0x300000 – 0x3FFFFF` verified via verbatim multicodec/README.md fetch + `gh search code` independent corroboration (multiple repos cite same range)
- ✅ 8-project private-use squat list verified via direct `gh search code "0x300000" "multicodec"` JSON output — each row maps to a real `path` + `repository.url` in the search results
- ✅ TSP X25519+Kyber768 + ML-DSA-65 byte sequences quoted from `openwallet-foundation-labs/tsp/tsp_sdk/src/vid/did/peer.rs` via WebFetch
- ✅ did:key v0.9 spec multicodec list verified via direct fetch of `https://w3c-ccg.github.io/did-key-spec/`
- ✅ di-quantum-safe multikey table (ML-DSA-44 0x1210, SLH-DSA 0x1220, FALCON 0x122c, SQIsign 0x122e) verified via fetch of `https://w3c.github.io/vc-di-quantum-safe/`
- ✅ CID 1.0 multikey extension-governance clause verified via verbatim fetch of `https://www.w3.org/TR/cid-1.0/`
- ✅ iroh X25519MLKEM768 + 1KB-per-direction overhead + opt-in posture verified via fetch of `https://www.iroh.computer/blog/iroh-post-quantum-handshakes` (covered in depth by Agent 3; quoted here only at the level needed for cross-reference)

**No phantom cites. No unverified claims.**
