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

**The fix (recommended construction).** The sender produces a real **signature over a
strip/substitution/cross-context-resistant binding** of the message and recipient context,
using their DID signing key, and **carries that signature INSIDE the encrypted inner payload**
(right next to the inner sender-DID). The recipient, post-decrypt, **resolves the sender-DID
to its verifying key and cryptographically verifies the signature** — fail-closed on any
mismatch. The signature is encrypted, so the network/relay never sees it → confidentiality
(sealed-ness) is preserved. The CEK derivation, the blinded group AAD, the F-01 truncation
defense, and the existing AAD bindings are all **unchanged** and **reused as the signed
binding inputs** (the signature *covers* them, it does not replace them).

**The two BEN-DECISION points:**

- **BD-1 (Q2 — per-message signing scheme).** **RECOMMEND classical Ed25519 (~64 B/sig)** for
  the per-message authenticator, NOT the LAMPS hybrid (~3373 B/sig). Rationale + cost below;
  the headline is that a group of N members costs **N × 64 B ≈ a few KB** with Ed25519 vs
  **N × 3373 B ≈ tens-to-hundreds of KB** with LAMPS, and the per-message auth property is a
  *now*-property (an attacker forging *today* needs to break Ed25519 *today*) for which the
  classical floor under the hybrid-IDENTITY umbrella is the defensible v1-beta choice — with a
  **codepoint-dispatched signature suite** so the hybrid arm is a free additive upgrade, never
  a wire-break. **This is a real security-posture call and must be Ben-ratified.**
- **BD-2 (Q4 — wire placement).** **RECOMMEND modify the existing `0x6510`/`0x6520`/`0x6610`
  in-place to ALWAYS carry the sender signature** (authenticated-sealed-sender becomes the v1
  DEFAULT), and **delete** the unauthenticated variant — there is no defensible use-case for an
  *unauthenticated* sealed-sender drop (an unauthenticated sender is indistinguishable from a
  forgery). The freeze is **not yet tagged** (`phase-4-meta-core-close` is HELD pending Ben), so
  in-place modification is still wire-legal. **This is a freeze-shape call and must be Ben-ratified.**

**Cost summary (recommended = Ed25519 + in-place default):** +~64 B inner ciphertext per stanza
(plus the inner sig is sealed, so +~16 B AEAD overhead is already paid by the existing inner
seal — net +~64 B/stanza), one Ed25519 sign per send + one Ed25519 verify per open, no new
trust anchor, no new PKI, no new codepoint. §4.1 becomes true; `f_lc_3` becomes substantive.

---

## 1. The construction (the load-bearing design)

### 1.1 What the sender signs — the SenderAuthBinding

The sender computes a **domain-separated binding** `M_auth` and signs it with their DID
signing key. The binding MUST be:

- **strip-resistant** — you cannot remove the signature and have the envelope still verify
  (achieved by BD-2: the authenticated path is the only path; `open` requires a present,
  verifying signature);
- **substitution-resistant** — a signature minted for one (sender, recipient, message,
  envelope-position) context MUST NOT verify in any other (achieved by binding all of those
  into `M_auth`);
- **cross-context-resistant** — a sealed-sender auth signature MUST NOT be reinterpretable as
  any other Benten signature (offline DropBundle envelope-sig, UCAN-Varsig, device attestation,
  rotation attestation, …) and vice-versa (achieved by a per-surface domain-separation tag,
  exactly mirroring `envelope_sig.rs::ENVELOPE_SIG_DOMAIN`).

**`M_auth` definition (recommended):**

```text
SENDER_AUTH_DOMAIN = b"benten/layer-c/sealed-sender-origin-auth/v1"   // per-surface DS tag

M_auth = SENDER_AUTH_DOMAIN
       ‖ sig_codepoint           (u16 BE)     // BD-1 suite selector — binds the auth suite
       ‖ envelope_codepoint      (u16 BE)     // 0x6510 / 0x6520 / 0x6610 — binds the band
       ‖ lp_u32(sender_did)                   // the claimed origin — self-binding
       ‖ recipient_context                    // see below — binds WHO this is for
       ‖ body_cid                (36 B)        // self-describing CIDv1 — binds the message
       ‖ recipient_key_generation (u32 BE)    // Inv-16/U19 — binds the key epoch
       ‖ aad_digest              (32 B)        // BLAKE3 of the stanza's plaintext AAD bytes
```

- `recipient_context` is **the same value the AAD already binds**, per band:
  - `0x6510` single: `lp_u32(audience_did)`.
  - `0x6520` Layer-C group: `audience_set_commitment (32 B)` + `stanza_index (u32 BE)` +
    `stanza_count (u32 BE)`.
  - `0x6610` MembershipSet group: `audience_set_commitment (32 B)` + `membership_set_id_commitment (32 B)`
    + `member_key_generation` + `membership_set_generation` + `role_assignments_generation`
    + `stanza_index` + `stanza_count`.
- `aad_digest = BLAKE3(stanza.plaintext_aad_bytes())`. **This is the elegant composition
  lever:** instead of re-enumerating every AAD field into `M_auth` (and risking drift from the
  frozen AAD field-set), we bind the *digest of the already-canonical, already-frozen,
  already-blinded AAD bytes*. Anything the AAD binds — codepoint, body-CID, blinded roster,
  stanza index/count, all the generations — is transitively covered by the signature, with
  **zero** new wire-field-set drift risk and **zero** weakening of the F-01 truncation defense
  (the signature now *additionally* covers `stanza_count`, so an attacker who rewrites the
  count breaks both the AEAD tag AND the origin-auth verify). The explicit fields above
  (`sig_codepoint`, `envelope_codepoint`, `sender_did`, `recipient_context`, `body_cid`,
  `recipient_key_generation`) are listed for **defense-in-depth + readability**; they are a
  superset-pin that does not depend on the AAD-digest alone.

> **Note — why `M_auth` binds the recipient context explicitly even though it also binds
> `aad_digest`:** belt-and-suspenders. If a future wave ever changes which fields the AAD
> carries, the explicit binds keep `M_auth` self-describing. The cost is a few dozen bytes
> hashed at sign/verify time — negligible.

### 1.2 Where the signature lives — INSIDE the inner sealed payload

The inner payload that the CEK seals **today** is:

```text
inner = lp_u32(sender_did) ‖ body          // single
inner = lp_u32(sender_did)                 // group (body is the shared bulk seal)
```

The **only change** is to append the signature to the inner payload **before** it is sealed
under the CEK:

```text
inner_v2 = lp_u32(sender_did) ‖ sig_codepoint(u16 BE) ‖ lp_u32(sender_sig) ‖ body   // single
inner_v2 = lp_u32(sender_did) ‖ sig_codepoint(u16 BE) ‖ lp_u32(sender_sig)          // group
```

Because `inner_v2` is bulk-sealed under the CEK (ChaCha20-Poly1305) with the stanza AAD as
the AEAD AAD, the signature is:

- **confidential** — the relay sees only ciphertext; the sender-DID AND its signature are both
  inside the seal. Sealed-ness is preserved end-to-end (Compromise #43-improving holds; the
  on-wire AAD field-set is **unchanged** — `sealed_aad::aad_field_set()` stays
  `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`).
- **tamper-evident at two layers** — the AEAD tag catches inner-byte tampering (existing
  defense); the origin-auth verify catches a *validly-sealed-but-wrong-signer* envelope (the
  NEW defense — the one the AEAD tag structurally cannot catch, because a co-member can
  produce a valid AEAD tag).

`sig_codepoint` inside `inner_v2` lets the recipient dispatch the verify to the right suite
(BD-1) and is itself bound into `M_auth`, so an attacker cannot downgrade the auth suite
(e.g. swap a hybrid-coded sig for a classical-coded one) without breaking the verify.

### 1.3 Seal-side flow (per stanza)

1. Build the stanza plaintext AAD exactly as today (unchanged — blinded group AAD, F-01
   `stanza_count`, etc.).
2. Compute `M_auth` (§1.1) from `{sender_did, recipient_context, body_cid,
   recipient_key_generation, aad_digest, sig_codepoint, envelope_codepoint}`.
3. `sender_sig = SignatureSuite::resolve(sig_codepoint).sign_with_context(sender_kp, M_auth, ctx=SENDER_AUTH_DOMAIN)`.
   (Routing through `benten_crypto_suite::sig` per CLAUDE.md baked-in #5 — this module stays
   concat/framing glue only.)
4. `inner_v2 = lp(sender_did) ‖ sig_codepoint ‖ lp(sender_sig) [‖ body for single]`.
5. Seal `inner_v2` under the CEK with the stanza AAD (existing path; the CEK derivation is
   **unchanged**).
6. HPKE-wrap the CEK to the recipient (unchanged).

### 1.4 Open-side flow (per stanza) — the post-decrypt VERIFY

1. F-01 truncation check (`delivered == stanza_count`) — **unchanged, runs first, fail-closed**.
2. HPKE-unwrap the CEK; AEAD-unwrap `inner_v2` under the stanza AAD (existing path; catches
   inner tampering / substitution / re-target).
3. Parse `sender_did`, `sig_codepoint`, `sender_sig`, `[body]` out of `inner_v2`.
4. **Resolve `sender_did` → verifying key** via `benten_id::did::Did::resolve` (+ optionally
   consult `RotationLog` — see Q3 / §4.3).
5. Re-build `M_auth` from the recovered `sender_did` + the recipient's own knowledge of the
   stanza AAD/context (the recipient recomputes `aad_digest` from the AAD bytes it just
   AEAD-verified).
6. `SignatureSuite::resolve(sig_codepoint).verify_with_context(vk, M_auth, ctx, sender_sig)`
   → **fail-closed** (new typed error `SenderOriginAuthFailed`) on ANY mismatch.
7. Only on `Ok(())` return `(body, sender_did)`. **A caller can now trust that `sender_did`
   really sent this.**

---

## 2. Design questions — RESOLVED

### Q1 — Construction: what is signed, how it stays sealed-yet-verifiable

**RESOLVED (see §1).** The sender signs `M_auth` (a domain-separated binding over
`{sender_did, recipient/audience-context, body_cid, recipient_key_generation, aad_digest,
sig_codepoint, envelope_codepoint}`); the signature is carried inside the CEK-sealed inner
payload and verified post-decrypt against the verifying key resolved from `sender_did`.

- **Composition with CEK derivation:** untouched. The CEK is still
  `BLAKE3(domain ‖ … ‖ sender_did ‖ …)`; the signature is an *additional* inner field, not a
  CEK input. (We deliberately do NOT feed the signature into the CEK — that would make the CEK
  non-deterministic w.r.t. the hedged/randomized hybrid signing path and would gain nothing,
  since `M_auth` already binds everything the CEK binds.)
- **Composition with blinded group AAD:** strengthened, not weakened. `M_auth` binds
  `aad_digest`, so the signature transitively covers the blinded `audience_set_commitment` +
  `membership_set_id_commitment` — i.e. the origin-auth signature is itself bound to the
  blinded roster without ever seeing the raw roster (confidentiality of the roster preserved).
- **Composition with sealed inner sender-DID:** the inner sender-DID stays sealed; we simply
  make it **authenticated** (it was authenticated-as-bytes by the AEAD tag, but authored-by
  anyone; now it is authenticated-as-origin by the signature).
- **Composition with F-01 truncation defense:** strengthened. `stanza_count` is now covered
  by the origin-auth signature as well as the AEAD tag; the F-01 pre-decrypt count check still
  runs first and unchanged.
- **Confidentiality (sealed-ness):** preserved. The on-wire plaintext AAD field-set is
  byte-identical to today (`sealed_aad` unchanged); the signature is inside the ciphertext.

### Q2 — Signing key + cost — **BD-1 (Ben-decision)**

**RECOMMENDATION: classical Ed25519 (`SigCodepoint::CLASSICAL_ED25519 = 0x0002`, ~64 B/sig)
for the per-message origin authenticator, dispatched via a codepoint so the LAMPS hybrid arm
is a free additive upgrade.**

**Cost analysis (the load-bearing input to the call):**

| scheme | sig size | single (`0x6510`) | group N (`0x6520`/`0x6610`) per-send wire growth |
|---|---|---|---|
| **Ed25519 (RECOMMENDED)** | 64 B | +64 B | **N × 64 B** (e.g. N=50 → +3.2 KB; N=1000 → +64 KB) |
| LAMPS hybrid `0x0001` | 3373 B | +3373 B | **N × 3373 B** (e.g. N=50 → +169 KB; N=1000 → +3.3 MB) |

The group cost is **per-stanza** because each member's stanza carries its own inner sealed
payload (the sender-DID is already per-stanza). LAMPS at group scale is a ~53× wire blow-up
and a real DoS/cost surface on large MembershipSets; Ed25519 keeps the authenticator in the
noise relative to the X-Wing-wrapped CEK already in every stanza.

**Security tradeoff (the reason this is a Ben call, not an orchestrator call):**

- Long-term **identity** is PQ-hybrid (the LAMPS composite is the `0x0001` default for
  durable, signed-once-verified-forever artifacts; this is why DropBundle offline issuer-auth,
  UCAN-Varsig, etc. are hybrid).
- Per-message **origin authentication** is a *liveness/now* property: to forge a message
  attributed to Alice, an attacker must produce a valid Ed25519 signature **at send time**.
  Harvest-now-decrypt-later does NOT apply to *authentication* (a recorded signature cannot be
  retroactively forged; the only PQ risk is a future quantum adversary forging *new* messages,
  at which point Alice's *identity key* — the thing that matters — is already protected by the
  hybrid identity layer, and per-message auth can be flipped to hybrid via the codepoint with
  zero wire-break).
- The classical Ed25519 half is the **audited floor** under the hybrid-identity umbrella —
  the same posture CLAUDE.md baked-in #5 takes for the hybrid construction generally ("the
  classical half is the audited security floor").

**Why a codepoint-dispatched suite (not hardcoded Ed25519):** Inv-15 / CLAUDE.md #5 forbid
hardcoded sizes and mandate codepoint dispatch with a typed-reject arm. `inner_v2` carries
`sig_codepoint`; the verify resolves it via `SignatureSuite::resolve_codepoint`. A deployment
or a future v1-GM wave can flip the per-message authenticator to LAMPS hybrid `0x0001` purely
additively. **No size is hardcoded** (`signature_byte_len_for(codepoint)` already exists).

**Key-source sub-question (signing key identity):** the sender signs with **the signing key
that backs `sender_did`** — i.e. the user/principal/plugin DID's own key, the same key
`did:key:z…` resolves to. No new ephemeral or per-sender key is introduced (a per-sender
ephemeral signing key bound once would re-introduce a binding/PKI problem — "who vouches the
ephemeral key is Alice's?" — and is rejected for v1; see FLAG-3). **This means the sealer must
hold their DID signing key at send time** — true for every real sender (they already sign
UCANs / DropBundles); the only callers that "seal as someone" without that key are the very
forgeries we are closing.

> **FLAG-2 (Q3-coupled, see below):** if BD-1 picks **hybrid** (`0x0001`), Q3 key-discovery
> needs the hybrid verifying key resolvable from `sender_did` — which today's `did.rs::resolve`
> does NOT support (it is Ed25519-only). Ed25519 (recommended) resolves cleanly today. This
> coupling is a concrete reason the recommendation is Ed25519 for v1-beta.

### Q3 — Key discovery: recipient resolves the sender's verifying key — no new trust assumption

**RESOLVED — clean for the recommended Ed25519 choice; a wiring gap exists for the hybrid choice.**

- `sender_did` is a `did:key:z…` string already recovered (sealed) at open. `did:key` is
  **self-certifying**: the verifying key IS the DID (W3C did-method-key — the public key is
  embedded in the identifier). `benten_id::did::Did::resolve(&self) -> Result<PublicKey, _>`
  already exists and returns the Ed25519 `PublicKey`. **No PKI, no registry, no new trust
  anchor** — the DID *is* the key. This is exactly the offline-DropBundle pattern
  (`envelope_sig::verify_envelope(vk_bytes, sig_bytes, msg)`), just with the vk *derived from
  the sealed DID* instead of carried in a `issuer_verifying_key` field (we don't need a carrier
  field because did:key is self-certifying — and carrying a separate vk would just be a
  second forgeable field).
- **Revocation / rotation:** the recipient MAY additionally consult `benten_id::did_rotation::RotationLog`
  (`is_superseded(did)` + `accept_rotation_event`) to reject a sender-DID whose key has been
  rotated/revoked — reusing Phase-3 infrastructure, no new tooling (per CLAUDE.md #18
  "Revocation reuses Phase-3 infrastructure"). RECOMMEND: verify-against-resolved-key is
  MANDATORY; RotationLog consultation is an OPTIONAL caller-supplied policy arg (a recipient
  with no rotation context still gets origin-auth; a recipient with rotation context gets
  origin-auth + freshness). This keeps `benten-drop`'s dependency surface from growing a
  mandatory rotation-state dependency.
- **GAP for hybrid (FLAG-2):** `did.rs::resolve` only dispatches the `ED25519_MULTICODEC
  [0xed,0x01]` arm; the `HYBRID_SIG_MULTICODEC [0xef,0x01]` prefix is *reserved* but resolution
  to a hybrid `sig::PublicKey` is **not wired**. Choosing Ed25519 (BD-1 recommendation) avoids
  this; choosing hybrid would pull a `did.rs` hybrid-resolution wave into this mini-ADDL's
  scope. Recorded as a cost input to BD-1.

### Q4 — Wire placement — **BD-2 (Ben-decision)**

**RECOMMENDATION: modify `0x6510` / `0x6520` / `0x6610` IN-PLACE to ALWAYS carry the sender
signature (authenticated-sealed-sender = the v1 DEFAULT), and DELETE the unauthenticated
sealed-sender variant. KEEP the explicitly-non-default plaintext-sender siblings
(`0x6500` / `0x6520`+`plaintext_sender_did` / `0x6610`+`plaintext_sender_did`) — but those,
too, gain the inner origin-auth signature (plaintext-sender discloses WHO; it still must prove
the who).**

**Why in-place + default (not a new codepoint):**

- The freeze is **NOT yet tagged.** `docs/V1-FROZEN-INTERFACE.md` states the freeze "locks at
  git tag `phase-4-meta-core-close`", which is HELD pending Ben (per the night-shift state).
  The `0x65xx`/`0x66xx` rows are AUTHORED-frozen but the tag has not landed → in-place wire
  change is still legal **now** and ONLY now. This is the entire reason this work is sequenced
  before the tag.
- An **unauthenticated** sealed-sender drop has **no defensible use-case**: an unauthenticated
  sender-DID is indistinguishable from a forgery, so a recipient can never trust it — keeping
  the unauthenticated variant would just preserve the §4.1-false behavior behind a second
  codepoint. The "anonymous send" use-case is served by *omitting/zeroing the sender-DID*
  (true anonymity), NOT by an unauthenticated-but-claimed sender-DID (the worst of both).
- A new authenticated codepoint would (a) leave the §4.1 claim false for the default
  codepoint, (b) double the conformance/test matrix, (c) burn a codepoint for no benefit.

**The cost of in-place:** it is a wire-format change to a surface that is AUTHORED-frozen,
so it MUST land before the tag and MUST update every byte-pin / golden / inventory row that
covers the `0x65xx`/`0x66xx` inner-payload shape (`V1-WIRE-FORMAT-INVENTORY.md`,
`f_lc_*` goldens, `f_inv16_*`; the membership-set `f_aad_*` cross-check is AAD-only so it is
**unaffected** — the AAD field-set does not change; only the *inner sealed payload* grows).
This is the bounded, intentional cost of doing it pre-freeze.

> **If Ben prefers to NOT change the default wire (BD-2 = new-codepoint):** the fallback is a
> new authenticated codepoint per band (e.g. `0x6511` / `0x6521` / `0x6611` — all unused in the
> band) carrying `inner_v2`, with the existing codepoints retained but **documented as
> non-authenticated / not-for-trusted-attribution** and `§4.1` re-scoped to the new codepoints
> only. This is strictly more wire-surface + leaves a forgery-capable default; recorded as the
> non-recommended alternative.

### Q5 — Group specifics: closing the every-member-can-spoof gap + per-group cost

**RESOLVED.** Today, for `0x6520` and `0x6610`, **every co-member can spoof** because the CEK
is derivable from public inputs (`0x6520`) or from `K_Set` which all members hold (`0x6610`),
and the inner sender-DID is unauthenticated. The design closes this **per stanza,
independently** (matching the §4.1 "per-stanza-LIVE" framing): each stanza's inner payload now
carries `sender_sig` over `M_auth` (which binds *that* stanza's `stanza_index`/`stanza_count`/
blinded context), verified against the verifying key resolved from the sealed sender-DID. A
co-member who holds `K_Set` can still *derive the CEK and produce a valid AEAD tag* — but they
**cannot produce a valid `sender_sig` for a DID whose signing key they do not hold**, so their
forged stanza fails the post-decrypt origin-auth verify. **Inter-member non-forgeability now
holds.**

**Per-group cost:** +`signature_byte_len_for(sig_codepoint)` per stanza inside the seal →
**N × 64 B** (Ed25519, recommended) added to the envelope, plus N Ed25519 signs at seal and 1
Ed25519 verify at open (each recipient verifies only their own stanza). No change to the shared
bulk-body seal, the CEK wrap count, or the F-01 machinery.

### Q6 — Proof + test: how §4.1 becomes true + the substantive `f_lc_3`

**RESOLVED.** See §3 (proof restatement) and §5 (`f_lc_3` substantive-test spec).

---

## 3. §4.1 — restated as a TRUE claim

> **§4.1 — Sealed-Sender ORIGIN-AUTHENTICATED property (RATIFIED, post-mini-ADDL).**
>
> EVERY Sealed-Sender send (`0x6510` / `0x6520` / `0x6610`) is, by default, **origin-
> authenticated AND sender-confidential**:
>
> - **Sender-confidential (sealed):** the sender-DID — and its origin-auth signature — live
>   INSIDE the ciphertext, sealed under the CEK. The on-wire plaintext AAD field-set is
>   unchanged (`{aad_version, codepoint, audience, body_cid, recipient_key_generation}`); the
>   relay/network sees neither the sender-DID nor the signature (Compromise #43-improving).
> - **Inter-member non-forgeability (NOW TRUE):** a member — even one holding `K_Set` and thus
>   able to derive the CEK and produce a valid AEAD tag — **cannot** mint a stanza attributed
>   to another member. The inner payload carries `sender_sig`, a real signature (BD-1 suite)
>   over the domain-separated binding `M_auth` (sender-DID + recipient/blinded-roster context +
>   body-CID + key-generation + stanza index/count + AAD-digest), produced with the **sender's
>   DID signing key**. The recipient resolves the sealed sender-DID to its verifying key
>   (self-certifying `did:key`; optionally RotationLog-checked) and **cryptographically
>   verifies** the signature post-decrypt, fail-closed (`SenderOriginAuthFailed`). Forging an
>   attribution requires the target's signing key — which a co-member does not have. The
>   property holds **per stanza, independently**, robust to active relays.
> - **Strip / substitution / cross-context resistance:** the authenticated path is the only
>   path (the unauthenticated variant is removed), so the signature cannot be stripped; `M_auth`
>   binds the full envelope/stanza context, so a signature cannot be replayed across
>   recipients, stanzas, envelopes, or bands; the `SENDER_AUTH_DOMAIN` tag + `sig_codepoint`
>   binding prevent cross-surface and cross-suite reinterpretation.
> - **Truncation/censorship + cross-stanza-substitution defenses (UNCHANGED + STRENGTHENED):**
>   the F-01 `stanza_count` pre-decrypt check still runs first; `stanza_count` and the blinded
>   per-stanza context are now ALSO covered by the origin-auth signature.

This restatement is what the mini-ADDL must make true in code; the proof obligation is
discharged by the substantive `f_lc_3` (a real second-sealer spoof must be REJECTED) plus the
existing round-trip + tamper pins.

---

## 4. Notes on what is UNCHANGED (composition safety)

- **CEK derivations:** unchanged on all three paths.
- **HPKE CEK-wrap (X-Wing `0x647a`):** unchanged.
- **Plaintext AAD field-sets + `sealed_aad`:** unchanged — confidentiality posture identical,
  metadata-disclosure invariant (Inv-18) still satisfied by Sealed-Sender being default.
- **Membership-set `f_02` AAD cross-check (`assemble_group_aad_local` byte-equality vs
  `benten_membership_set::aad::assemble_group_aad`):** UNAFFECTED — the AAD bytes don't change;
  only the inner *sealed payload* grows.
- **F-01 truncation defense + `dispatch_group` cross-band strict-reject:** unchanged.
- **Abuse-control delivery tokens (`abuse_control`, #63):** orthogonal and complementary —
  delivery tokens are a recipient-issued *pre-decrypt admission* gate (rate-limit/spam), NOT
  sender authentication. Both are wanted; this design does not touch the token path.

---

## 5. `f_lc_3` substantive-test spec (the would-FAIL-on-revert pin)

**The defect in the existing pin.** `f_lc_3_forged_inner_sender_did_rejected` (today) only
flips a ciphertext byte → the AEAD tag catches it. That is SHAPE-not-SUBSTANCE: it tests
ciphertext integrity, NOT origin authentication. A revert of the entire origin-auth feature
leaves it GREEN. **It must be replaced/augmented with a real second-sealer spoof.**

**New substantive pins (must FAIL if origin-auth is reverted):**

- **`f_lc_3_second_sealer_spoof_rejected_single` (`0x6510`).**
  1. Sealer-A (holds A's keypair) seals to recipient-R claiming `sender_did = A`. Recipient
     opens → `Ok`, recovered sender == A. (positive control)
  2. Sealer-B (holds B's keypair, knows `recipient_pk` — which is public) constructs a fresh,
     fully-valid envelope to R **claiming `sender_did = A`** but signing `M_auth` with **B's**
     key (or omitting/garbaging the sig). The envelope AEAD-opens cleanly (B can derive the CEK).
  3. **ASSERT** recipient `open_single` returns `Err(SenderOriginAuthFailed)` — the spoof is
     rejected at the origin-auth verify, NOT at the AEAD layer.
  - would-FAIL-on-revert: with origin-auth removed, step-3 returns `Ok((body, A))` — a silent
     successful impersonation. Commit body MUST state this.

- **`f_lc_3_second_member_spoof_rejected_membership_group` (`0x6610`).** The real group threat.
  1. Members A and B both hold `K_Set`. A seals a group drop claiming sender A → members open
     → `Ok`, sender == A. (positive control)
  2. **B derives the group CEK from `K_Set`** (the actual capability every member has) and
     constructs a fresh valid `GroupSealedEnvelope` claiming `sender_did = A`, signing `M_auth`
     with **B's** key (or no/garbage sig). Every stanza AEAD-opens.
  3. **ASSERT** every honest recipient's `open_membership_set_group` returns
     `Err(GroupError::SenderOriginAuthFailed)`.
  - would-FAIL-on-revert: without origin-auth, the co-member impersonation SUCCEEDS — this is
     the exact THREAT-MODEL "Co-recipient member" gap. Commit body MUST demonstrate the revert.

- **`f_lc_3_second_sealer_spoof_rejected_layer_c_group` (`0x6520`).** Same shape as `0x6610`
  but for the Layer-C group (CEK derivable from public inputs → ANY party, not just a member,
  can spoof today).

- **`f_lc_3_legit_sender_still_verifies` (all bands).** Positive controls retained: a real
  sender's drop opens AND origin-verifies (the round-trip property `f_lc_3_recovered_inner_
  sender_did_equals_bound` is extended to assert origin-auth `Ok`, not just byte-recovery).

- **`f_lc_3_cross_context_replay_rejected`.** Take a valid `sender_sig` from one stanza/
  recipient/envelope and splice it into another (different `stanza_index` / `audience` /
  `body_cid`); ASSERT `SenderOriginAuthFailed` — pins the substitution/cross-context resistance
  of `M_auth`.

These pins are the R3 red-phase corpus for the mini-ADDL; the R5 impl turns them green by
landing the construction in §1.

---

## 6. FLAGs / risks for Ben

- **FLAG-1 (BD-2 freeze timing — HARD sequencing constraint).** In-place wire modification of
  `0x65xx`/`0x66xx` is legal ONLY before the `phase-4-meta-core-close` tag. This mini-ADDL MUST
  land before the freeze tag. If the tag has already landed when this is read, BD-2 is forced
  to the new-codepoint fallback (§Q4 alternative) and §4.1 is re-scoped to the new codepoints.
  **Confirm tag has NOT landed before starting R5.**
- **FLAG-2 (BD-1 ↔ Q3 coupling).** Picking the LAMPS hybrid per-message authenticator pulls a
  `benten_id::did::resolve` hybrid-key-resolution wave (`HYBRID_SIG_MULTICODEC` is reserved but
  unwired) into scope, AND multiplies group wire cost ~53×. The Ed25519 recommendation avoids
  both. If Ben wants PQ per-message auth at v1-beta, scope the did.rs hybrid-resolution
  explicitly.
- **FLAG-3 (rejected alternative — ephemeral per-sender signing key).** Considered and
  rejected: it re-introduces a "who vouches for the ephemeral key" binding problem (a fresh
  ephemeral key bound once would itself need a signature from the DID key — i.e. you do the DID
  signature anyway). Signing directly with the DID key is simpler and strictly stronger. Noted
  for the record.
- **FLAG-4 (sealer must hold the DID signing key).** Origin-auth requires the sender to sign at
  send time → the sealing context must have access to the sender's DID signing key. This is
  true for all legitimate senders (they already sign UCANs/bundles) but worth confirming for any
  relay-side / re-seal flows (there should be none — re-sealing as another principal is the
  forgery we're closing).
- **FLAG-5 (Inv-15 alignment — already satisfied by design).** Per Inv-15 / the dispatch-
  conventions "3-layer-decompose" rule: identity = payload/body CID (unchanged); authentication
  = the codepoint-dispatched `sender_sig` (this design's addition); revocation = sender-DID +
  RotationLog semantic tuple (Q3). No identifier is keyed off the signature bytes →
  Inv-15-clean. Record this in the R5 PR.
- **FLAG-6 (golden/byte-pin churn).** In-place wire change churns every `0x65xx`/`0x66xx`
  inner-payload golden + the `V1-WIRE-FORMAT-INVENTORY` rows. Budget a goldens-regen + inventory
  update in the R5 wave (via throwaway-compute per M-20, not hand-edited hex).

---

## 7. Mini-ADDL sequencing (what comes after this R0)

R0 (this doc) → **Ben ratifies BD-1 + BD-2** → R1 (focused critic council: cryptographer +
wire-format + threat-model lenses on this construction) → R2/R3 (the `f_lc_3` substantive
corpus in §5, red-phase) → R5 (land §1 in `layer_c.rs` + the `SenderOriginAuthFailed` typed
errors + `did`-resolve verify; goldens-regen) → R4b → fold §4.1 restatement (§3) into
`SECURITY-PROOFS.md` + THREAT-MODEL "Co-recipient member" row + CRYPTO-CODEPOINTS notes +
V1-WIRE-FORMAT-INVENTORY → **then** the `phase-4-meta-core-close` freeze tag (Ben-gated) can
proceed with §4.1 TRUE.
