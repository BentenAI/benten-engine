# Sealed-Sender ORIGIN-AUTHENTICATION — design (R0, mini-ADDL option B2)

> **DESIGN ONLY.** No `src` is modified by this doc. This is the R0 plan for a focused
> mini-ADDL that adds **real sender-origin authentication** to Benten's ONLINE Layer-C
> Sealed-Sender drops (`0x6510` single / `0x6520` Layer-C group / `0x6610` MembershipSet
> K_Set group) so that `docs/SECURITY-PROOFS.md §4.1`'s **"inter-member non-forgeability"**
> claim becomes TRUE — while PRESERVING sender-confidentiality (the sender identity stays
> hidden from the network/relay: still "sealed").
>
> **Base commit:** `d0ccf606` (main). **Branch:** `phase-4-meta-core/sealed-sender-auth-design`.
> **Ben ratification:** option B2 (focused mini-ADDL) ratified; this doc is its R0 input.
>
> ## ★ R0.2 REVISION — BD-1 OVERRIDDEN to FULL PQ-HYBRID + per-MESSAGE signing (2026-06-06)
>
> Ben **overrode BD-1's prior Ed25519-classical-floor recommendation.** Per CLAUDE.md
> baked-in #5 (PQ-everywhere; the classical-only-signature path is a really-built
> NON-DEFAULT downgrade, never the default), sender-origin-auth MUST be **full PQ-hybrid**
> (the LAMPS Composite `id-MLDSA65-Ed25519-SHA512` at `SigCodepoint::HYBRID_ED25519_MLDSA65
> = 0x0001`), NOT classical-only. **The naive ~53× group blow-up the prior R0 used to
> justify Ed25519 was an artifact of signing PER STANZA. It dissolves entirely once the
> signature is per MESSAGE.** This R0.2 makes **per-message hybrid signing** the recommended
> permanent v1 shape: **ONE** LAMPS-hybrid signature (~3373 B) per message regardless of
> recipient count N — carried inside the **once**-sealed body region (the body is bulk-sealed
> exactly once under one CEK on all three paths; verified in code §1.0). BD-2 is KEPT as
> Ben-ratified (in-place; authenticated-sealed-sender = v1 default; unauthenticated variant
> deleted). New in-scope work: wire the `did:key` HYBRID multicodec so the hybrid verifying
> key resolves from the sealed sender-DID (today `did:key` is Ed25519-only — the real wiring
> gap, §Q3 / FLAG-2). All §-numbers below are the R0.2 construction; the superseded R0.1
> Ed25519/per-stanza analysis is preserved at the end (§8) for forensic context.

---

## 0. TL;DR (the report)

**The gap (verified in code).** On every Sealed-Sender path the sender-DID lives as a
**plaintext field sealed under the content-encryption key (CEK)**, and the CEK is derived
**only from material every authorized sealer already holds** — no sender *secret* ever
enters the construction:

| Path | CEK derivation (verbatim from `layer_c.rs`) | Who can derive the CEK |
|---|---|---|
| `0x6510` single | `BLAKE3("benten-drop:layer-c:cek" ‖ recipient_pk ‖ sender_did ‖ aad ‖ body)` | anyone who knows `recipient_pk` (it is a public fingerprint) — **anyone** |
| `0x6520` Layer-C group | `BLAKE3("benten-drop:layer-c:group-cek" ‖ body_cid ‖ sender_did ‖ recipient_key_generation)` | **anyone** (all inputs are public/attacker-chosen) |
| `0x6610` MembershipSet group | `BLAKE3("benten-drop:membership-group-cek" ‖ K_Set ‖ sender_did)` | **every group member** (they all hold `K_Set`) |

The recipient's `open_*` recovers the inner sender-DID and the code comments call this
"post-decrypt verify" — **but there is no verify**. `open_inner` / `open_single` /
`open_membership_set_group` merely *parse* the length-prefixed sender-DID bytes out of the
decrypted inner payload. Nothing checks that those bytes were placed there by the holder of
that DID's signing key. **Therefore any sealer (and for the group paths, any co-member) can
mint a fully-valid envelope that opens cleanly and attributes itself to an ARBITRARY
`sender_did`.** This is precisely the impersonation the THREAT-MODEL "Co-recipient member"
row leaves open, and it makes the §4.1 "inter-member non-forgeability" claim **false today**.

**The fix (recommended construction — R0.2).** The sender produces a **single, real PQ-hybrid
signature** (LAMPS Composite `id-MLDSA65-Ed25519-SHA512`, `0x0001`) **over a domain-separated,
strip/substitution/cross-context-resistant binding of the BODY + the audience-set + the
envelope context** (`M_auth`, §1.1), using their DID signing key. The signature is carried
**INSIDE the once-CEK-sealed body region** (one signature for the whole send — NOT one per
stanza), so the relay/network never sees it → confidentiality (sealed-ness) is preserved. Each
recipient, post-decrypt, **resolves the sealed sender-DID to its hybrid verifying key** (via the
self-certifying `did:key` — newly hybrid-aware) and **cryptographically verifies the signature**
(both halves — Ed25519 AND ML-DSA-65 — must verify) — fail-closed on any mismatch. The CEK
derivations, the blinded group AAD, the F-01 truncation defense, and the existing AAD bindings
are all **unchanged** and **reused as signed binding inputs** (the signature *covers* them; it
does not replace them).

**The headline (per-message-vs-per-stanza — the elegant permanent shape).** The body is
**bulk-sealed exactly ONCE** under one CEK on all three paths (verified in code §1.0); the
per-recipient stanzas are **CEK-key-wrapping + a re-statement of the sealed sender-DID**, not
per-recipient *authorship*. The sender authors the **content** once. Therefore one hybrid
signature over `M_auth` — placed in the once-sealed region, bound to the audience-set
commitment — authenticates the whole send for **every** recipient at **constant cost**:

| construction | per-send wire growth | N=1 | N=50 | N=1000 |
|---|---|---|---|---|
| **R0.2 — LAMPS hybrid, per-MESSAGE (RECOMMENDED)** | **+~3373 B (CONSTANT in N)** | +3.4 KB | **+3.4 KB** | **+3.4 KB** |
| R0.1 — LAMPS hybrid, per-STANZA (rejected) | N × 3373 B | +3.4 KB | +169 KB | **+3.3 MB** |
| R0.1 — Ed25519, per-STANZA (superseded recommendation) | N × 64 B | +64 B | +3.2 KB | +64 KB |

The ~53× blow-up that drove the prior R0 to classical Ed25519 **was entirely an artifact of
per-stanza signing.** Per-message signing makes the choice cost-free: full PQ-hybrid at
constant ~3.4 KB/send is **smaller** than R0.1's *classical* Ed25519 for any N ≥ 53, and it is
in the noise relative to the N X-Wing-wrapped CEKs (~1 KB+ each) already in every group
envelope. **Full PQ-hybrid is now the strictly-better choice on cost AND on the baked-in #5
posture.** (See §1.4 soundness proof-sketch for why per-message signing is sound.)

**BD-1 RESOLVED (Ben override applied): full PQ-hybrid LAMPS `0x0001`, signed per-MESSAGE.**
Codepoint-dispatched (`sig_codepoint` carried in the sealed body region + bound into `M_auth`),
so the classical-only `0x0002` arm remains a really-built NON-DEFAULT swap and any future suite
is an additive upgrade — never a wire-break. No size hardcoded (`signature_byte_len_for` exists).

**BD-2 KEPT as ratified: in-place modify `0x6510`/`0x6520`/`0x6610`; authenticated-sealed-sender
= the v1 DEFAULT; the unauthenticated variant is DELETED.** (§Q4.)

**The one remaining real cost (FLAG-2, now IN-SCOPE not avoided):** `did:key` resolution is
**Ed25519-only today** (`benten_id::did::Did::resolve` dispatches only the `ED25519_MULTICODEC
[0xed,0x01]` arm; the `HYBRID_SIG_MULTICODEC` the prior R0 called "reserved" **does not exist in
the codebase** — verified). Full PQ-hybrid per-message sender-auth therefore REQUIRES a
`did:key` hybrid-multicodec wiring wave (encode + resolve a hybrid `benten_crypto_suite::sig`
public key from a sealed sender-DID). This is bounded, self-contained, reusable (it is the same
gap every other hybrid-identity flow will hit), and it is the price of doing PQ-everywhere
honestly. Scoped in §Q3 + §7.

**Cost summary (recommended = full PQ-hybrid LAMPS + per-message + in-place default):**
**+~3373 B per SEND (constant in N)** inside the once-sealed body region (the +~16 B AEAD
overhead is already paid by the existing body seal); **ONE** LAMPS-hybrid sign per send + **one**
hybrid verify per open (each recipient verifies the single body signature once); a `did:key`
hybrid-resolution wave; no new trust anchor, no new PKI, no new codepoint. §4.1 becomes true;
`f_lc_3` becomes substantive.

---

## 1. The construction (the load-bearing design — R0.2)

### 1.0 The structural fact that makes per-message signing the right shape (verified in code)

On ALL THREE paths the **body is bulk-sealed exactly ONCE** under one shared CEK; only the
**per-stanza inner payload** (carrying the length-prefixed sender-DID) and the **per-stanza
CEK-wrap** repeat per recipient. Verbatim from `crates/benten-drop/src/layer_c.rs @ d0ccf606`:

- **`0x6520` group** (`seal_group_impl`): `// One shared CEK seals the bulk body ONCE; each
  recipient gets a wrapped copy (the Q4 share-to-N efficiency property).` → `cek_aead_ciphertext`
  is one ciphertext; the loop only builds per-stanza `sealed_inner` + `wrapped_cek`.
- **`0x6610` MembershipSet group** (`seal_membership_set_group`): `// Bulk-seal the body once
  (shared AAD = aad_version + codepoint + cid).` → `body_wire` is one ciphertext; the loop only
  builds per-stanza `sealed_inner` + per-stanza `wrapped_cek`.
- **`0x6510` single** (`seal_inner`): N = 1; the body and the sender-DID are sealed in the same
  inner payload. Per-message ≡ per-stanza here, so the constant-cost property holds trivially.

**Consequence.** The thing the sender *authored* is the **body** + the *choice of audience*. The
per-stanza machinery is **key delivery**, not authorship. Authenticating the body once (bound to
the audience-set commitment + envelope context) authenticates exactly what the sender authored,
for every recipient, at O(1) cost. This is the elegant permanent shape: **sign authorship once,
deliver keys N times.**

### 1.1 What the sender signs — the `SenderAuthBinding` (`M_auth`)

The sender computes a **domain-separated binding** `M_auth` and signs it ONCE per message with
their DID signing key (the LAMPS hybrid keypair). The binding MUST be:

- **strip-resistant** — you cannot remove the signature and have the envelope still verify
  (achieved by BD-2: the authenticated path is the only path; `open` requires a present,
  verifying signature). Within the LAMPS construction, the hybrid combiner is itself
  strip-resistant (both halves cover the same `M'`; the ML-DSA half binds the Label as context —
  per the byte-faithful IETF composite in `sig.rs`).
- **substitution / re-target-resistant** — a signature minted for one (sender, audience-set,
  message, envelope/band) context MUST NOT verify in any other (achieved by binding all of those
  into `M_auth`; the **audience-set-commitment** binding is what stops a co-member re-wrapping
  the body to a new recipient set — §1.4).
- **cross-context-resistant** — a sealed-sender auth signature MUST NOT be reinterpretable as any
  other Benten signature (offline DropBundle envelope-sig, UCAN-Varsig, device attestation,
  rotation attestation, …) and vice-versa (achieved by a per-surface domain-separation tag fed
  as the **LAMPS `ctx`** + as a `M_auth` prefix, mirroring `envelope_sig.rs::ENVELOPE_SIG_DOMAIN`).

**`M_auth` definition (RECOMMENDED — R0.2, per-message):**

```text
SENDER_AUTH_DOMAIN = b"benten/layer-c/sealed-sender-origin-auth/v1"   // per-surface DS tag (also the LAMPS ctx)

M_auth = SENDER_AUTH_DOMAIN
       ‖ sig_codepoint            (u16 BE)     // BD-1 suite selector — binds the auth suite (default 0x0001)
       ‖ envelope_codepoint       (u16 BE)     // 0x6510 / 0x6520 / 0x6610 — binds the band
       ‖ lp_u32(sender_did)                    // the claimed origin — self-binding
       ‖ body_cid                 (36 B)        // self-describing CIDv1 over the body — binds the CONTENT
       ‖ audience_commitment      (per band — see below)   // binds WHO the message is for (anti-re-target)
       ‖ recipient_key_generation (u32 BE)     // Inv-16/U19 — binds the key epoch (single + 0x6520)
       ‖ stanza_count             (u32 BE)      // F-01 truncation defense, now ALSO covered by the sig
       ‖ body_aad_digest          (32 B)        // BLAKE3 of the once-sealed BODY's AEAD AAD bytes
```

- `audience_commitment` is **the same value the wire/AAD already carries** (NOT a re-derivation
  of the raw roster — confidentiality of the roster is preserved):
  - `0x6510` single: `lp_u32(audience_did)` (the single AAD's audience field).
  - `0x6520` Layer-C group: `audience_set_commitment (32 B)` — the BLINDED roster commitment
    already on the wire (`EncryptedEnvelope::HpkeMultiBase`).
  - `0x6610` MembershipSet group: `audience_set_commitment (32 B)` **‖** `membership_set_id_commitment
    (32 B)` **‖** `member_key_generation` **‖** `membership_set_generation` **‖**
    `role_assignments_generation` — the BLINDED set-identity commitments already in the 11-field
    group AAD.
- **`body_aad_digest` is the per-message composition lever** (the R0.2 analogue of R0.1's
  per-stanza `aad_digest`). The body is sealed ONCE under a body AAD that already binds
  `{aad_version, codepoint, body_cid}`; we bind `BLAKE3(body_aad_bytes)` so the signature
  transitively covers the body seal's frozen, canonical AAD with **zero** new wire-field-set
  drift risk. The explicit fields above (`sig_codepoint`, `envelope_codepoint`, `sender_did`,
  `body_cid`, `audience_commitment`, `recipient_key_generation`, `stanza_count`) are a
  **superset-pin** listed for defense-in-depth + readability; they do not depend on the
  AAD-digest alone.

> **Note — why `M_auth` binds `stanza_count` explicitly even at per-message granularity:** the
> F-01 truncation defense lives in the per-stanza AAD (and runs pre-decrypt), but binding
> `stanza_count` into the single body signature means a relay that rewrites the count to forge a
> smaller delivery breaks the origin-auth verify too (defense-in-depth on top of the existing
> per-stanza AEAD + the pre-decrypt count check). The per-stanza `stanza_index` is NOT in
> `M_auth` (it is intrinsically per-stanza and the single signature is shared) — `stanza_index`
> stays bound by the per-stanza AAD/AEAD exactly as today, which is sufficient for cross-stanza
> substitution (U17). See §1.4 for the proof that omitting per-stanza signing does not reopen U17.

### 1.2 Where the signature lives — INSIDE the once-sealed BODY region

The inner payloads the CEK seals **today** are:

```text
// single (0x6510): inner = lp_u32(sender_did) ‖ body          (sealed under CEK with the single AAD)
// group  (0x6520/0x6610): body  = plaintext                   (bulk-sealed ONCE under CEK with the body AAD)
//                          stanza.sealed_inner = lp_u32(sender_did)   (per-stanza, sealed under CEK with the per-stanza AAD)
```

The **R0.2 change places the single hybrid signature in the ONCE-sealed body region:**

```text
// single (0x6510): inner_v2 = lp_u32(sender_did) ‖ sig_codepoint(u16 BE) ‖ lp_u32(sender_sig) ‖ body
//                  (one inner payload; N=1; the sig is sealed with the body)

// group (0x6520/0x6610): body_v2 = sig_codepoint(u16 BE) ‖ lp_u32(sender_sig) ‖ body
//                  (the sig is PREPENDED into the ONCE-bulk-sealed body region — sealed ONE time,
//                   shared by all recipients; the per-stanza sealed_inner is UNCHANGED: still
//                   lp_u32(sender_did) only, which the per-stanza AEAD authenticates as bytes)
```

- **The sender-DID stays where it is** (single: in the inner; group: per-stanza `sealed_inner`)
  so each recipient still learns *who claims to have sent this* from the part they decrypt.
- **The signature is in the body region**, which is **decrypted by every recipient** (every
  recipient unwraps the same CEK and opens the same body). So every recipient can verify with
  exactly what they already decrypt — **no extra wire material is delivered to any recipient**,
  and the signature is delivered **once** on the wire (not per stanza).

Because `body_v2` / `inner_v2` is bulk-sealed under the CEK (ChaCha20-Poly1305) with the body
AAD as the AEAD AAD, the signature is:

- **confidential** — the relay sees only ciphertext; the sender-DID AND its signature are both
  inside the seal. Sealed-ness is preserved end-to-end; the on-wire plaintext AAD field-set is
  **unchanged** (`sealed_aad::aad_field_set()` stays `{aad_version, codepoint, audience,
  body_cid, recipient_key_generation}`; the group per-stanza 11-field AAD is **unchanged**).
- **tamper-evident at two layers** — the AEAD tag catches body/inner-byte tampering (existing
  defense); the origin-auth verify catches a *validly-sealed-but-wrong-signer* envelope (the NEW
  defense — the one the AEAD tag structurally cannot catch, because a co-member can produce a
  valid AEAD tag).

`sig_codepoint` inside the sealed region lets the recipient dispatch the verify to the right suite
(BD-1; default `0x0001` LAMPS hybrid) and is itself bound into `M_auth`, so an attacker cannot
downgrade the auth suite (e.g. swap the hybrid codepoint for the classical `0x0002`) without
breaking the verify — and `sig.rs::verify` already fail-closes a classical-suite verify of a
hybrid-coded sig with `VerifyError::CodepointMismatch` (the silent-downgrade defense is built).

### 1.3 Seal-side flow (per SEND — one signature)

1. Bulk-seal the body exactly as today is computed (compute `body_cid`, the CEK, the body AAD).
2. Compute `M_auth` (§1.1) from `{sender_did, audience_commitment (from the wire/AAD), body_cid,
   recipient_key_generation, stanza_count, body_aad_digest, sig_codepoint, envelope_codepoint}`.
3. `sender_sig = SignatureSuite::resolve_codepoint(sig_codepoint)?.sign_with_context(sender_kp,
   M_auth, ctx = SENDER_AUTH_DOMAIN)`. The default suite is `0x0001` (LAMPS hybrid); the call
   produces a real ML-DSA-65 half + a real Ed25519 half over the shared `M'`, no wire change to
   `sig.rs`. (Routing through `benten_crypto_suite::sig` per baked-in #5 — that module stays
   concat/framing glue only.)
4. `body_v2 = sig_codepoint ‖ lp(sender_sig) ‖ body` (group) **or** `inner_v2 = lp(sender_did) ‖
   sig_codepoint ‖ lp(sender_sig) ‖ body` (single).
5. Seal `body_v2` / `inner_v2` under the CEK with the body AAD (existing path; the CEK derivation
   is **unchanged**).
6. Build the N per-stanza `sealed_inner` (group) + HPKE-wrap the CEK per recipient — **unchanged**.

> **Determinism note (why the sig is NOT a CEK input).** The LAMPS hybrid ML-DSA half is
> **hedged/randomized** (`sign_randomized` via OS entropy — FIPS-recommended). The CEK
> derivations are deterministic (`BLAKE3(domain ‖ … ‖ sender_did ‖ …)`). We deliberately do NOT
> feed `sender_sig` into the CEK (it would make the CEK non-deterministic and gain nothing, since
> `M_auth` already binds everything the CEK binds). The sig sits *inside* the AEAD plaintext;
> the AEAD already authenticates it as bytes; the origin-auth verify authenticates it as origin.

### 1.4 Open-side flow (per recipient) — the post-decrypt VERIFY + the soundness proof-sketch

1. F-01 truncation check (`delivered == stanza_count`) — **unchanged, runs first, fail-closed**.
2. HPKE-unwrap the CEK; AEAD-unwrap the body region (`body_v2` / `inner_v2`) under the body AAD;
   for the group paths, also AEAD-unwrap this recipient's `sealed_inner` to recover `sender_did`
   (existing per-stanza path — catches per-stanza tampering / cross-stanza substitution / wrong
   recipient). **U17 cross-stanza substitution is still caught here** by the per-stanza AEAD over
   the per-stanza AAD (`stanza_index`/`stanza_count`/blinded context) — that defense is
   unchanged; we did not move it into the (shared) signature.
3. Parse `sig_codepoint`, `sender_sig`, `body` out of the body region; parse `sender_did` (from
   `inner` single, or from `sealed_inner` group).
4. **Resolve `sender_did` → hybrid verifying key** via `benten_id::did::Did::resolve` (newly
   hybrid-aware — §Q3) (+ optionally consult `RotationLog` — §Q3 / §4.3).
5. Re-build `M_auth` from the recovered `sender_did` + the recipient's own knowledge of the
   wire/AAD context (the recipient recomputes `body_aad_digest` from the body AAD it just
   AEAD-verified, and reads `audience_commitment` / `stanza_count` from the wire it received).
6. `SignatureSuite::resolve_codepoint(sig_codepoint)?.verify_with_context(vk, M_auth, ctx,
   &sender_sig)` → **fail-closed** (new typed error `SenderOriginAuthFailed`) on ANY mismatch.
   For the hybrid suite this requires BOTH halves to verify (the `sig.rs` hybrid arm never
   returns `Ok` after a single half).
7. Only on `Ok(())` return `(body, sender_did)`. **A caller can now trust that `sender_did`
   really sent this exact body to this exact audience-set.**

**Soundness proof-sketch — does per-message (not per-stanza) signing reopen any attack?** NO.
Walk the attacker's options (attacker = a co-member who holds `K_Set`/can derive the CEK, the
strongest sealed-sender adversary):

- **(a) Forge authorship (mint a body attributed to Alice).** The attacker must produce a valid
  `sender_sig` over `M_auth` (which binds `sender_did = Alice`) verifiable under Alice's hybrid
  key. They do not hold Alice's signing key → they cannot. The single body signature is the same
  proof for every recipient, so the forgery fails for all. **Closed.** (This is the §4.1
  inter-member non-forgeability property — now TRUE.)
- **(b) Re-target Alice's real body to a NEW recipient-set (re-wrap attack — the one genuinely
  new question for per-message signing).** The attacker takes Alice's legitimately-signed body
  (which they can decrypt as a co-member), derives a fresh CEK, re-seals the SAME `body_v2`
  (carrying Alice's real signature) to a different recipient set, and rebuilds the stanzas. **Does
  Alice's signature still verify for the new recipients?** NO — `M_auth` binds
  `audience_commitment` (the `audience_set_commitment` / set-identity commitments). A new
  recipient set produces a different commitment; the recipients recompute `M_auth` with the
  commitment they actually received, and Alice's signature (over the *original* audience
  commitment) fails to verify → `SenderOriginAuthFailed`. **Closed by the audience-set-commitment
  binding** — this is exactly why binding `audience_commitment` into the signature is the clean
  anti-re-target mechanism (and it is already on the wire/in the AAD, so binding it is free).
- **(c) Splice Alice's signature onto a DIFFERENT body.** `M_auth` binds `body_cid` (+
  `body_aad_digest`); a different body changes the CID; verify fails. **Closed.**
- **(d) Cross-stanza substitution (U17) / truncation (F-01).** Unchanged and still closed by the
  per-stanza AEAD + the pre-decrypt count check; `M_auth` additionally covers `stanza_count` as
  defense-in-depth. The single shared signature does NOT weaken these because they never relied
  on per-stanza *signing* — they rely on per-stanza *AEAD over per-stanza AAD*, which is untouched.
- **(e) Suite downgrade.** `sig_codepoint` is bound into `M_auth` AND `sig.rs::verify`
  fail-closes a cross-suite verify (`CodepointMismatch`). **Closed.**
- **(f) Cross-context replay (use the sig as a DropBundle/UCAN/device-attestation sig or vice
  versa).** The `SENDER_AUTH_DOMAIN` prefix + LAMPS `ctx` domain-separate this surface from every
  other Benten signature surface. **Closed.**

**The single residual soundness subtlety (the ONLY thing per-message gives up vs per-stanza, and
why it does not matter):** per-stanza signing would let a recipient prove *to a third party* that
a SPECIFIC stanza was authored for them; per-message signing proves authorship of the *body to
the audience-SET*, not to an individual stanza-position. For Benten's threat model (origin-auth +
anti-re-target + anti-forge among co-members) this is exactly the wanted property and arguably
**better** (it does not create a per-recipient non-repudiation token that could deanonymize a
specific recipient). No attack in (a)–(f) needs per-stanza granularity. **Per-message signing is
sound.** (Recorded as the headline answer to the brief.)

---

## 2. Design questions — RESOLVED

### Q1 — Construction: what is signed, how it stays sealed-yet-verifiable

**RESOLVED (see §1).** The sender signs `M_auth` ONCE per message (a domain-separated binding
over `{sender_did, audience-commitment, body_cid, recipient_key_generation, stanza_count,
body_aad_digest, sig_codepoint, envelope_codepoint}`) with the LAMPS-hybrid DID key; the
signature is carried inside the once-CEK-sealed body region and verified post-decrypt by each
recipient against the hybrid verifying key resolved from `sender_did`.

- **Composition with CEK derivation:** untouched (the sig is not a CEK input — §1.3 determinism
  note).
- **Composition with the blinded group AAD:** strengthened, not weakened. `M_auth` binds the
  blinded `audience_set_commitment` + set-identity commitments directly (they are on the wire),
  and `body_aad_digest` transitively. The signature is bound to the blinded roster without ever
  seeing the raw roster → roster confidentiality preserved.
- **Composition with the per-stanza sealed sender-DID + per-stanza AEAD:** the per-stanza
  `sealed_inner` and its AEAD-over-AAD are **unchanged** (U17 still closed there). We add ONE
  body-region signature on top; the sender-DID it authenticates is the one the recipient recovers
  from `sealed_inner`.
- **Composition with F-01 truncation defense:** strengthened (`stanza_count` now also covered by
  the signature); the pre-decrypt count check still runs first and unchanged.
- **Confidentiality (sealed-ness):** preserved. On-wire plaintext AAD field-set byte-identical to
  today; the signature is inside the ciphertext and on the wire exactly ONCE.

### Q2 — Signing scheme + cost — **BD-1 RESOLVED (Ben override = FULL PQ-HYBRID, per-message)**

**RESOLVED: full PQ-hybrid LAMPS Composite `id-MLDSA65-Ed25519-SHA512`
(`SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`), signed ONCE per message.** This is the
v1-beta sender-auth DEFAULT, dispatched via `sig_codepoint` so the classical-only `0x0002` arm
remains a really-built NON-DEFAULT swap and any future suite is an additive upgrade.

**Why this is now the strictly-better choice (the cost analysis, corrected for per-message):**

| scheme | sig size | single (`0x6510`) | group N — per-send wire growth |
|---|---|---|---|
| **LAMPS hybrid `0x0001`, per-MESSAGE (RECOMMENDED)** | 3373 B | +3373 B | **+3373 B (CONSTANT in N)** |
| LAMPS hybrid `0x0001`, per-STANZA (R0.1, rejected) | 3373 B | +3373 B | N × 3373 B (N=1000 → +3.3 MB) |
| Ed25519 `0x0002`, per-STANZA (R0.1 superseded rec) | 64 B | +64 B | N × 64 B (N=1000 → +64 KB) |

- The ~53× blow-up that drove R0.1 to classical Ed25519 **was an artifact of per-stanza
  signing.** Per-message LAMPS hybrid is **constant ~3.4 KB/send** — *smaller* than R0.1's
  classical Ed25519 for any N ≥ 53, and negligible next to the N X-Wing-wrapped CEKs (~1 KB+
  each) already in every group envelope.
- **Posture (baked-in #5):** sender-origin-auth is now full PQ-hybrid by default — the classical
  half is the audited floor; the ML-DSA-65 half closes the harvest-now-forge-later concern for a
  future quantum adversary forging *new* attributions (the property is a now-property, but
  attributing-to-Alice-after-Q-day is a real concern for durable drops, and the cost to close it
  is now zero). `classical-only 0x0002` stays a really-built swap, never the default.
- **No size hardcoded** (`SignatureSuite::signature_byte_len_for(codepoint)` already exists);
  codepoint dispatch with the typed-reject arm (`resolve_codepoint` → `UnsupportedAlgorithm`)
  satisfies Inv-15 / baked-in #5.

**Key-source sub-question (signing key identity):** the sender signs with **the LAMPS-hybrid
signing key that backs `sender_did`** — the same key `did:key:z…` resolves to (once the hybrid
multicodec is wired). No new ephemeral or per-sender key (the ephemeral-subkey alternative is
evaluated + rejected in §Q2b / FLAG-3). **The sealer must hold their DID hybrid signing key at
send time** — true for every real sender (they already sign UCANs/DropBundles with it); the only
callers that "seal as someone" without that key are the forgeries we are closing.

### Q2b — Ephemeral per-sender signing-subkey (alternative explored per the brief — REJECTED)

The brief asked to evaluate **a per-sender EPHEMERAL hybrid signing-subkey signed ONCE by the
long-term DID** (amortize the hybrid cost across many messages). Evaluated and **REJECTED** for
v1 — the per-message construction already delivers constant cost, so the indirection buys nothing
and costs real complexity:

- **It does not reduce cost.** The per-message construction is already **O(1) per send**
  (one ~3373 B sig). An ephemeral subkey would amortize the *sign-time CPU* across messages, but
  (a) one hybrid sign per send is not a bottleneck, and (b) the *wire* cost is already constant —
  the ephemeral pattern's whole value is amortizing a per-stanza cost we no longer have.
- **It RE-INTRODUCES a binding/PKI problem.** "Who vouches the ephemeral key is Alice's?" → you
  need a `long_term_DID.sign(ephemeral_pubkey)` certificate carried in the envelope, which is
  *itself* ~3373 B hybrid + the ephemeral pubkey bytes (ML-DSA-65 vk ≈ 1952 B) — so the
  first/only message in a session is **larger**, not smaller, and you have added a
  cert-freshness/rotation/replay surface (subkey lifetime, revocation of a leaked subkey, binding
  the subkey to a time-window) for zero wire savings on the per-message path.
- **It weakens forward-clarity for verification.** Recipients would verify a chain (subkey-cert
  then message-sig) instead of one direct `did:key`-resolved verify. More moving parts, more
  failure modes, no benefit.

**Verdict:** sign directly with the long-term DID key, once per message. Recorded; the ephemeral
pattern is a future option ONLY if a session-batching feature ever makes per-message signing the
bottleneck (it is not at v1).

### Q3 — Key discovery: recipient resolves the sender's HYBRID verifying key — the real wiring gap

**RESOLVED — with an in-scope `did:key` hybrid-multicodec wiring wave (the genuine cost of the
full-PQ-hybrid override).**

- `sender_did` is a `did:key:z…` string already recovered (sealed) at open. `did:key` is
  **self-certifying**: the verifying key IS the DID (W3C did-method-key — the public key is
  embedded in the identifier). **No PKI, no registry, no new trust anchor** — the DID *is* the
  key. This is the offline-DropBundle pattern (`envelope_sig::verify_envelope`), with the vk
  *derived from the sealed DID* instead of carried in a forgeable field.
- **THE GAP (verified, corrected from R0.1's "reserved" framing):** `benten_id::did::Did::resolve`
  is **Ed25519-ONLY** — it dispatches **only** the `ED25519_MULTICODEC [0xed,0x01]` arm and
  returns a `benten_id::keypair::PublicKey` (an Ed25519 `VerifyingKey` wrapper). **The
  `HYBRID_SIG_MULTICODEC` the prior R0 called "reserved-but-unwired" does NOT exist anywhere in
  the codebase** (grep-verified across `crates/` + `docs/`). `benten-id`'s `Keypair`/`PublicKey`
  are Ed25519-only; the hybrid keypair lives in a *different* type world
  (`benten_crypto_suite::sig::Keypair` / `::PublicKey`, carrying both halves). So full PQ-hybrid
  sender-auth genuinely requires a new wiring wave:
  1. **Allocate a `did:key` hybrid multicodec** for the LAMPS-composite public key (the encode of
     `sig::PublicKey` = `Ed25519 vk (32 B) ‖ ML-DSA-65 vk (1952 B)`). Use a self-describing
     multicodec prefix; **do NOT mint a Benten-private algorithm number** where an IANA/multicodec
     code exists — per baked-in #5, reference registries. (R1's cryptographer/wire lens picks the
     exact prefix; candidate: a composite-key multicodec or a Benten-namespaced
     application-multicodec if no registry entry fits — flagged for R1, not decided here.)
  2. **`Did::from_hybrid_public_key(&sig::PublicKey) -> Did`** — encode `multicodec ‖ ed_vk ‖
     mldsa_vk` as the did:key body.
  3. **`Did::resolve_hybrid(&self) -> Result<sig::PublicKey, DidError>`** (or extend `resolve` to
     a tagged enum return) — dispatch the hybrid multicodec arm, reconstruct `sig::PublicKey`
     from the two vk slices, with a typed-reject arm for unknown multicodecs (mirrors the existing
     `DidError::UnknownMulticodec`).
  4. The verify path in `layer_c.rs` calls `resolve_hybrid` (default) and feeds the
     `sig::PublicKey` to `SignatureSuite::verify_with_context`.
  - This wave is **bounded, self-contained, and reusable** — it is the SAME gap every other
    hybrid-identity flow (hybrid UCAN issuers, hybrid DropBundle issuers) will hit, so wiring it
    here pays a debt that is coming regardless. It is the honest price of PQ-everywhere.
- **Revocation / rotation:** the recipient MAY additionally consult
  `benten_id::did_rotation::RotationLog` (`is_superseded(did)` + `accept_rotation_event`) to
  reject a rotated/revoked sender-DID — reusing Phase-3 infra, no new tooling (baked-in #18
  "Revocation reuses Phase-3 infrastructure"). RECOMMEND: verify-against-resolved-key is
  MANDATORY; RotationLog consultation is an OPTIONAL caller-supplied policy arg (a recipient with
  no rotation context still gets origin-auth; one with rotation context gets origin-auth +
  freshness). Keeps `benten-drop`'s dependency surface from growing a mandatory rotation-state dep.

### Q4 — Wire placement — **BD-2 (Ben-RATIFIED, KEPT)**

**RATIFIED + KEPT: modify `0x6510` / `0x6520` / `0x6610` IN-PLACE to ALWAYS carry the sender
signature (authenticated-sealed-sender = the v1 DEFAULT), and DELETE the unauthenticated
sealed-sender variant. The explicitly-non-default plaintext-sender siblings (`0x6500` /
`0x6520`+`plaintext_sender_did` / `0x6610`+`plaintext_sender_did`) are retained but ALSO gain the
once-sealed origin-auth signature (plaintext-sender discloses WHO; it still must prove the who).**

Ben has ratified this; it is no longer an open decision. The supporting rationale (recorded):

- The freeze is **NOT yet tagged.** `docs/V1-FROZEN-INTERFACE.md` locks at git tag
  `phase-4-meta-core-close`, which is HELD pending Ben → in-place wire change is legal **now** and
  ONLY now. This is the entire reason this work is sequenced before the tag (FLAG-1 HARD
  sequencing constraint stands).
- An **unauthenticated** sealed-sender drop has **no defensible use-case** (an unauthenticated
  sender-DID is indistinguishable from a forgery). True anonymity is served by *omitting/zeroing
  the sender-DID*, NOT by an unauthenticated-but-claimed sender-DID (the worst of both).
- A new authenticated codepoint would leave §4.1 false for the default, double the conformance
  matrix, and burn a codepoint for no benefit.

**The cost of in-place:** a wire-format change to an AUTHORED-frozen surface → MUST land before
the tag and MUST update every byte-pin / golden / inventory row covering the `0x65xx`/`0x66xx`
**body region** (`V1-WIRE-FORMAT-INVENTORY.md`, `f_lc_*` goldens, `f_inv16_*`). Note the R0.2
placement (signature in the **body** region, not per-stanza `sealed_inner`) means the per-stanza
inner-payload shape for the GROUP paths is **unchanged** (still `lp_u32(sender_did)`); only the
**once-sealed body region grows** (group) / the **single inner grows** (`0x6510`). The
membership-set `f_aad_*` cross-check is AAD-only → **unaffected** (the AAD field-set does not
change). This narrows the golden churn vs R0.1's per-stanza placement.

### Q5 — Group specifics: closing the every-member-can-spoof gap + per-group cost

**RESOLVED.** Today, for `0x6520` and `0x6610`, **every co-member can spoof** (CEK derivable from
public inputs / from `K_Set` all members hold; inner sender-DID unauthenticated). The R0.2 design
closes this with **ONE** body-region signature over `M_auth` (binding the body-CID + the blinded
audience/set commitments + `stanza_count`), verified by each recipient against the hybrid key
resolved from the recovered sender-DID. A co-member who holds `K_Set` can still derive the CEK and
produce valid AEAD tags — but they **cannot produce a valid `sender_sig` for a DID whose hybrid
signing key they do not hold**, and they **cannot re-target Alice's real signed body to a new set**
(the audience-commitment binding, §1.4 (b)). **Inter-member non-forgeability now holds.**

**Per-group cost:** +`signature_byte_len_for(0x0001)` = **+3373 B per SEND total** (CONSTANT in N)
inside the once-sealed body region, plus **ONE** hybrid sign at seal and **one** hybrid verify at
open per recipient (each recipient verifies the single shared body signature once). No change to
the per-stanza `sealed_inner` shape, the CEK-wrap count, or the F-01 machinery.

### Q6 — Proof + test: how §4.1 becomes true + the substantive `f_lc_3`

**RESOLVED.** See §3 (proof restatement) and §5 (`f_lc_3` substantive-test spec, incl. the new
**re-target** pin that the per-message construction specifically demands).

---

## 3. §4.1 — restated as a TRUE claim (R0.2 — full PQ-hybrid, per-message)

> **§4.1 — Sealed-Sender ORIGIN-AUTHENTICATED property (RATIFIED, post-mini-ADDL).**
>
> EVERY Sealed-Sender send (`0x6510` / `0x6520` / `0x6610`) is, by default, **origin-
> authenticated (full PQ-hybrid) AND sender-confidential**:
>
> - **Sender-confidential (sealed):** the sender-DID lives INSIDE the ciphertext (single: in the
>   inner; group: per-stanza `sealed_inner`); the origin-auth signature lives INSIDE the
>   once-sealed body region, on the wire exactly ONCE. The on-wire plaintext AAD field-set is
>   unchanged (`{aad_version, codepoint, audience, body_cid, recipient_key_generation}`); the
>   relay/network sees neither the sender-DID nor the signature (Compromise #43-improving).
> - **Inter-member non-forgeability (NOW TRUE) — full PQ-hybrid:** a member — even one holding
>   `K_Set` and thus able to derive the CEK and produce valid AEAD tags — **cannot** mint a send
>   attributed to another member, nor re-target another member's real body to a new recipient set.
>   The body region carries `sender_sig`, a single real **LAMPS-hybrid `id-MLDSA65-Ed25519-SHA512`
>   (`0x0001`)** signature over the domain-separated binding `M_auth` (sender-DID + blinded
>   audience/set commitments + body-CID + key-generation + stanza-count + body-AAD-digest),
>   produced with the sender's DID **hybrid** signing key. Each recipient resolves the sealed
>   sender-DID to its **hybrid** verifying key (self-certifying `did:key`, hybrid-multicodec;
>   optionally RotationLog-checked) and **cryptographically verifies both halves** post-decrypt,
>   fail-closed (`SenderOriginAuthFailed`). Forging an attribution requires the target's hybrid
>   signing key (post-quantum-secure); re-targeting is blocked by the audience-commitment binding.
> - **Strip / substitution / re-target / cross-context resistance:** the authenticated path is the
>   only path (unauthenticated variant removed) → the signature cannot be stripped; the LAMPS
>   combiner is internally strip-resistant (both halves bind the shared `M'`); `M_auth` binds the
>   full body + audience + envelope/band context → a signature cannot be replayed across bodies,
>   audience-sets, envelopes, or bands; the `SENDER_AUTH_DOMAIN` tag (as `M_auth` prefix AND LAMPS
>   `ctx`) + the `sig_codepoint` binding prevent cross-surface and cross-suite reinterpretation
>   (`sig.rs::verify` fail-closes a cross-suite verify).
> - **Truncation/censorship + cross-stanza-substitution defenses (UNCHANGED + STRENGTHENED):** the
>   F-01 `stanza_count` pre-decrypt check still runs first; the per-stanza AEAD-over-AAD still
>   closes U17; `stanza_count` is now ALSO covered by the origin-auth signature.
> - **Constant cost:** the origin-auth signature is **one per send (~3373 B), independent of N**;
>   it does not scale the envelope with the recipient count.

This restatement is what the mini-ADDL must make true in code; the proof obligation is discharged
by the substantive `f_lc_3` (a real second-sealer spoof AND a real re-target attempt must be
REJECTED) plus the existing round-trip + tamper pins.

---

## 4. Notes on what is UNCHANGED (composition safety)

- **CEK derivations:** unchanged on all three paths.
- **HPKE CEK-wrap (X-Wing `0x647a`):** unchanged.
- **Per-stanza `sealed_inner` shape (group paths):** unchanged (`lp_u32(sender_did)` only) — the
  signature is in the BODY region, not per-stanza. This is a deliberate R0.2 improvement over
  R0.1 (narrower golden churn; no per-recipient non-repudiation token).
- **Plaintext AAD field-sets + `sealed_aad` + the group 11-field AAD:** unchanged — confidentiality
  posture identical, metadata-disclosure invariant (Inv-18) still satisfied.
- **Membership-set `f_02` AAD cross-check (`assemble_group_aad_local` byte-equality vs
  `benten_membership_set::aad::assemble_group_aad`):** UNAFFECTED — the AAD bytes don't change.
- **F-01 truncation defense + `dispatch_group` cross-band strict-reject:** unchanged.
- **`benten_crypto_suite::sig`:** **unchanged** — the LAMPS-hybrid `sign_with_context` /
  `verify_with_context` API already exists and is byte-faithful; this design only *calls* it.
- **Abuse-control delivery tokens (`abuse_control`, #63):** orthogonal and complementary
  (recipient-issued pre-decrypt admission gate, NOT sender authentication). Both wanted; untouched.

---

## 5. `f_lc_3` substantive-test spec (the would-FAIL-on-revert pins — R0.2)

**The defect in the existing pin.** `f_lc_3_forged_inner_sender_did_rejected` (today) only flips a
ciphertext byte → the AEAD tag catches it. That is SHAPE-not-SUBSTANCE: it tests ciphertext
integrity, NOT origin authentication. A revert of the entire origin-auth feature leaves it GREEN.
It MUST be replaced/augmented with a real second-sealer spoof AND a real re-target attempt.

**New substantive pins (must FAIL if origin-auth is reverted):**

- **`f_lc_3_second_sealer_spoof_rejected_single` (`0x6510`).**
  1. Sealer-A (holds A's HYBRID keypair) seals to recipient-R claiming `sender_did = A`. R opens →
     `Ok`, recovered sender == A, origin-auth verifies. (positive control)
  2. Sealer-B (holds B's keypair, knows `recipient_pk` — public) builds a fresh, fully-valid
     envelope to R **claiming `sender_did = A`** but signing `M_auth` with **B's** key (or
     omitting/garbaging the sig). The envelope AEAD-opens cleanly (B can derive the CEK).
  3. **ASSERT** `open_single` → `Err(SenderOriginAuthFailed)` — rejected at the origin-auth verify,
     NOT at the AEAD layer.
  - would-FAIL-on-revert: with origin-auth removed, step-3 returns `Ok((body, A))` — silent
    successful impersonation. Commit body MUST state this.

- **`f_lc_3_second_member_spoof_rejected_membership_group` (`0x6610`).** The real group threat.
  1. Members A and B both hold `K_Set`. A seals a group drop claiming sender A → members open →
     `Ok`, sender == A, origin-auth verifies. (positive control)
  2. **B derives the group CEK from `K_Set`** and builds a fresh valid `GroupSealedEnvelope`
     claiming `sender_did = A`, signing the body-region `M_auth` with **B's** key (or no/garbage
     sig). Every stanza AEAD-opens.
  3. **ASSERT** every honest recipient's `open_membership_set_group` → `Err(GroupError::
     SenderOriginAuthFailed)`.
  - would-FAIL-on-revert: without origin-auth, the co-member impersonation SUCCEEDS — the exact
    THREAT-MODEL "Co-recipient member" gap. Commit body MUST demonstrate the revert.

- **`f_lc_3_second_sealer_spoof_rejected_layer_c_group` (`0x6520`).** Same shape as `0x6610` but
  for the Layer-C group (CEK derivable from public inputs → ANY party can spoof today).

- **`f_lc_3_retarget_to_new_audience_rejected` (`0x6520` + `0x6610`) — NEW, R0.2-specific.** The
  attack the per-message construction must specifically defeat (§1.4 (b)).
  1. A legitimately seals a group drop to recipient-set S1; members of S1 open → `Ok`,
     origin-auth verifies. (positive control)
  2. A co-member B (∈ S1, holds `K_Set`/derives the CEK) takes A's **real signed `body_v2`**,
     re-seals it under a fresh CEK to a DIFFERENT recipient-set S2 (different
     `audience_set_commitment`), rebuilds stanzas, claims `sender_did = A`.
  3. **ASSERT** every recipient in S2 → `Err(SenderOriginAuthFailed)` — A's signature is over S1's
     audience-commitment; S2 recomputes `M_auth` with S2's commitment; verify fails.
  - would-FAIL-on-revert: if `M_auth` omits the audience-commitment, the re-targeted send opens as
    a valid A-attributed message to an audience A never chose. Commit body MUST state this.

- **`f_lc_3_legit_sender_still_verifies` (all bands).** Positive controls retained: a real
  sender's drop opens AND origin-verifies (the round-trip property is extended to assert
  origin-auth `Ok` — both hybrid halves — not just byte-recovery).

- **`f_lc_3_cross_context_replay_rejected`.** Take a valid `sender_sig` from one (body / audience /
  band) context and splice it into another; ASSERT `SenderOriginAuthFailed` — pins the
  substitution/cross-context resistance of `M_auth`.

- **`f_lc_3_suite_downgrade_rejected`.** Take a hybrid-coded sig, rewrite `sig_codepoint` → `0x0002`
  (or vice versa) inside the sealed region; ASSERT fail-closed (`CodepointMismatch` surfaces as
  `SenderOriginAuthFailed`) — pins that the suite cannot be silently downgraded.

These pins are the R3 red-phase corpus for the mini-ADDL; the R5 impl turns them green by landing
the construction in §1 + the `did:key` hybrid-resolution wave (§Q3).

---

## 6. FLAGs / risks for Ben

- **FLAG-1 (BD-2 freeze timing — HARD sequencing constraint, UNCHANGED).** In-place wire
  modification of `0x65xx`/`0x66xx` is legal ONLY before the `phase-4-meta-core-close` tag. This
  mini-ADDL MUST land before the freeze tag. **Confirm the tag has NOT landed before starting R5.**
  If it has, BD-2 is forced to a new-codepoint fallback and §4.1 re-scoped.
- **FLAG-2 (did:key hybrid-resolution wave — NOW IN-SCOPE, the real cost of the BD-1 override).**
  Full PQ-hybrid sender-auth REQUIRES wiring a `did:key` hybrid multicodec + `resolve_hybrid` in
  `benten-id` (today it is Ed25519-only; `HYBRID_SIG_MULTICODEC` **does not exist** — corrected
  from R0.1's "reserved" framing). Bounded, self-contained, reusable (every hybrid-identity flow
  hits the same gap). **R1 cryptographer/wire lens must pick the exact multicodec prefix** (prefer
  a registry-referenced composite-key code over a Benten-private number, per baked-in #5).
  **FLAG-FOR-BEN: this adds a benten-id wave to the mini-ADDL scope** — it is the honest price of
  PQ-everywhere; confirm acceptable (the alternative — keeping the prior Ed25519 floor — was
  explicitly overridden).
- **FLAG-3 (rejected alternative — ephemeral per-sender signing-subkey).** Evaluated per the brief
  (§Q2b) and REJECTED for v1: per-message signing already gives constant cost, so the ephemeral
  subkey buys no wire savings while re-introducing a subkey-binding/PKI/freshness surface and
  making the first message LARGER. Future option only if session-batching ever makes per-message
  signing the bottleneck (it is not at v1). Recorded.
- **FLAG-4 (sealer must hold the DID HYBRID signing key).** Origin-auth requires the sender to sign
  at send time → the sealing context must have access to the sender's DID hybrid signing key. True
  for all legitimate senders; worth confirming there are no relay-side / re-seal flows (there
  should be none — re-sealing as another principal is the forgery we close).
- **FLAG-5 (Inv-15 alignment — already satisfied by design).** Per Inv-15 / the 3-layer-decompose
  rule: identity = body CID (unchanged); authentication = the codepoint-dispatched `sender_sig`
  (this design's addition); revocation = sender-DID + RotationLog semantic tuple (Q3). No
  identifier is keyed off the signature bytes → Inv-15-clean. Record in the R5 PR.
- **FLAG-6 (golden/byte-pin churn — NARROWED by the body-region placement).** The in-place wire
  change churns the `0x65xx`/`0x66xx` **body-region** goldens + `V1-WIRE-FORMAT-INVENTORY` rows.
  Because the signature is in the body region (not per-stanza), the per-stanza inner-payload
  goldens for the GROUP paths are **unaffected** and the AAD `f_aad_*` cross-checks are
  **unaffected** — narrower churn than R0.1. Budget a goldens-regen via throwaway-compute (M-20),
  not hand-edited hex. Note the LAMPS ML-DSA half is hedged/randomized → the `sender_sig` bytes
  are NOT reproducible across runs, so the goldens must pin the **body/AAD/wire shape + a verify
  round-trip**, NOT a fixed signature hex (a fixed-hex golden over a randomized signature is
  impossible — pin structure + a sign→verify property instead).
- **FLAG-7 (per-message non-repudiation scope — informational).** Per-message signing
  authenticates authorship of the body to the audience-SET, not per-recipient-stanza. This is the
  wanted property and arguably better (no per-recipient deanonymizing token). Noted for the §4.1
  audit line so a future reader does not mistake it for a gap.

---

## 7. Mini-ADDL sequencing (what comes after this R0.2)

R0.2 (this doc) → **Ben ratifies the BD-1 override resolution (full PQ-hybrid, per-message) +
confirms the did:key hybrid-resolution wave is in-scope (FLAG-2)** → R1 (focused critic council:
cryptographer + wire-format + threat-model lenses on this construction; the cryptographer lens
also picks the did:key hybrid multicodec prefix per FLAG-2) → R2/R3 (the `f_lc_3` substantive
corpus in §5, red-phase — including the new re-target + suite-downgrade pins) → R5 (land §1 in
`layer_c.rs` + the `SenderOriginAuthFailed` typed errors + the `benten-id` did:key
hybrid-resolution wave + the `verify_with_context` calls; goldens-regen as shape+property pins per
FLAG-6) → R4b → fold the §4.1 restatement (§3) into `SECURITY-PROOFS.md` + THREAT-MODEL
"Co-recipient member" row + CRYPTO-CODEPOINTS notes + V1-WIRE-FORMAT-INVENTORY → **then** the
`phase-4-meta-core-close` freeze tag (Ben-gated) can proceed with §4.1 TRUE.

---

## 8. SUPERSEDED — R0.1 analysis (Ed25519 floor + per-stanza signing), retained for forensic context

> The R0.1 analysis below recommended **classical Ed25519 (`0x0002`, ~64 B/sig)** for the
> per-message authenticator and signed **per stanza**, justified by a ~53× group wire blow-up.
> **Ben OVERRODE this 2026-06-06:** (a) baked-in #5 PQ-everywhere requires full PQ-hybrid, not a
> classical floor; (b) the ~53× blow-up was an artifact of **per-stanza** signing — it dissolves
> entirely with **per-message** signing (the body is sealed once; §1.0), making full PQ-hybrid
> constant-cost (~3.4 KB/send, smaller than R0.1's Ed25519 for N ≥ 53). The R0.2 construction
> above (per-message LAMPS hybrid in the once-sealed body region, audience-commitment-bound) is
> the recommended permanent shape. R0.1's key points, retained:
>
> - **R0.1 BD-1:** RECOMMEND Ed25519 `0x0002` per-stanza. **SUPERSEDED** — full PQ-hybrid `0x0001`
>   per-message (R0.2 §Q2). The codepoint-dispatch + typed-reject + no-hardcoded-sizes disciplines
>   carry over verbatim.
> - **R0.1 signature placement:** inside each per-stanza `sealed_inner` (`inner_v2 = lp(sender_did)
>   ‖ sig_codepoint ‖ lp(sender_sig) [‖ body]`), N copies on the wire. **SUPERSEDED** — one copy
>   in the once-sealed body region (R0.2 §1.2). The per-stanza `sealed_inner` reverts to its
>   original `lp(sender_did)`-only shape (narrower golden churn).
> - **R0.1 `M_auth`:** bound a per-stanza `aad_digest` + per-stanza `recipient_context` (incl.
>   `stanza_index`). **REVISED** — R0.2 binds a per-message `body_aad_digest` + the
>   audience-commitment + `stanza_count` (NOT `stanza_index`, which stays a per-stanza AEAD
>   concern). The substitution/strip/cross-context resistance goals are unchanged; the re-target
>   defense is now explicit (R0.2 §1.4 (b) + the new `f_lc_3_retarget_*` pin).
> - **R0.1 Q3:** treated the hybrid did:key resolution as a "reserved-but-unwired
>   `HYBRID_SIG_MULTICODEC`" and used it as a *reason to prefer Ed25519*. **CORRECTED** — that
>   multicodec does not exist in the codebase; R0.2 makes wiring it in-scope (FLAG-2) as the
>   honest cost of the PQ-everywhere override.
> - **R0.1 FLAG-2 / FLAG-3:** the BD-1↔Q3 coupling and the ephemeral-subkey rejection are carried
>   forward (FLAG-2 now in-scope rather than avoided; FLAG-3 unchanged — ephemeral subkey still
>   rejected, now for the stronger reason that per-message signing already gives constant cost).
> - Everything R0.1 marked UNCHANGED (CEK derivations, HPKE wrap, AAD field-sets, F-01, the
>   `f_02` AAD cross-check, abuse-control orthogonality) remains UNCHANGED under R0.2.
