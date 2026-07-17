# Security Proofs — the FROZEN AAD field-sets + per-stanza-LIVE decomposition (Phase-4-Meta-Core)

This document decomposes the **FROZEN AAD field-sets** for the Layer-C / MembershipSet encryption envelopes
(R0.7 §3.3 / §4.1). The governing property: **everything in the AAD is authenticated-not-encrypted** — it is
plaintext on the wire, bound by the AEAD tag, so a tamper is detected at decrypt but the bytes are visible to a
network observer. Each frozen field-set below is the v1-beta wire (G-CORE-9-frozen). Companion docs:
`docs/CRYPTO-CODEPOINTS.md` (the codepoint allocation) + `docs/THREAT-MODEL.md` (the network-observer-only
unlinkability scope this AAD binding underwrites) + `docs/SECURITY-POSTURE.md` (the named Compromises).

> **`body_cid` framing (applies to every field-set below).** `body_cid` is a **self-describing CIDv1**, multihash
> length-prefixed: `0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3 digest` (36 bytes). It is NOT a bare fixed-32-byte
> digest — baking a hash-width assumption into a frozen wire contradicts CLAUDE.md baked-in #5 (the multiformats
> framing is the permanent commitment; pre-blessed agile fallbacks = SHA-512/256 + SHA3-256). Self-describing
> CIDv1 is multihash-length-prefixed by construction ⇒ restores U3 length-injectivity for free. All integer AAD
> fields are big-endian (BE) — no `to_le_bytes` survives on any wire/AAD path (the M-19 BE sweep + the conformance
> test that pins it).

---

## §3.3 — `0x6510` single-recipient (`DROP_TO_RECIPIENT_SEALED_SENDER`) AAD

The minimal-sufficient union (Option A). 5 fields:

| Field | Width | Notes |
|---|---|---|
| `aad_version` | `0x01`, u8 | envelope AAD version |
| `codepoint` | `0x6510`, u16 BE | committed in AAD (U1) — strict-decode, no cross-variant fallback (U2) |
| `audience` | recipient DID, u32-BE length-prefixed | the only recipient identifier on the wire |
| `body_cid` | self-describing CIDv1 (36B) | the stable plaintext-CID reference |
| `recipient_key_generation` | u32 BE | key-retention generation (U19; Inv-16) |

**NO** stanza-index, **NO** recipient-DID-LIST, **NO** `sender_did` — the sender-DID is **sealed INSIDE the
ciphertext** (Sealed-Sender DEFAULT, BR-1 / F-LC-9). On the default path the sender-DID is therefore NOT on the
wire in plaintext (Compromise #43-improving). The non-default plaintext-sender sibling is `0x6500` (`LAYER_C_DROP`);
U4 (sender-DID-in-AAD) applies to IT.

---

## §3.3 — `0x6610` MembershipSet group per-stanza AAD (BLINDED)

The `0x6610` group per-stanza AAD for a MembershipSet — assembled by `benten_membership_set::aad::assemble_group_aad`, which binds the `benten_membership_set::codepoints::MEMBERSHIP_SET_GROUP_MULTI_STANZA` (`0x6610`) codepoint. **11 fields**, BLINDED:

| Field | Width | Notes |
|---|---|---|
| `aad_version` | `0x01`, u8 | |
| `codepoint` | `0x6610`, u16 BE | committed in AAD |
| `body_cid` | self-describing CIDv1 (36B) | |
| `member_count` | u32 BE | |
| `audience_set_commitment` | 32B | `BLAKE3(0x01 ‖ lp(did_0) ‖ lp(did_1) ‖ …)` over the **canonical SORTED** recipient-DID list (lp = u32-BE length prefix). BLINDED — replaces the prior raw roster. |
| `stanza_index` | u32 BE | this stanza's position |
| `stanza_count` | u32 BE | **truncation/censorship defense** — bound alongside `stanza_index` so an active relay cannot silently drop trailing stanzas (each survivor fails the bound count) |
| `member_key_generation` | u32 BE | per-recipient key generation |
| `membership_set_id_commitment` | 32B | `blake3::keyed_hash(K_Set, "benten:setid:v1" ‖ membership_set_id)` truncated to 32B. BLINDED — replaces the prior raw set-id. **Frozen primitive:** BLAKE3 native keyed-MAC (NOT HMAC-SHA256; `benten-membership-set` carries no hmac/sha2 dep). |
| `membership_set_generation` | u32 BE | the set generation |
| `role_assignments_generation` | u32 BE | RBAC role-assignment generation |

The **sealed-inner-sender-DID stays INSIDE the ciphertext** (NOT a plaintext AAD field; per F-LC-9). Recipients
hold `K_Set` + the member list, so they **recompute + verify** both 32-byte commitments — ALL binding properties
(cross-stanza substitution U17; inter-member non-forgeability) are PRESERVED; the relay sees only opaque 32-byte
tags. (Why blinded: the prior raw shape published the roster + raw set-id in plaintext, contradicting the project's
own §3.9 / Compromise #61 blinding posture; blinding makes the group AAD obey that rule.)

---

## §3.3 — `0x6520` `LAYER_C_DROP_MULTI_RECIPIENT` group per-stanza AAD (BLINDED)

`0x6520` is the Layer-C **multi-recipient** group send (`benten_drop::layer_c::EncryptedEnvelope::HpkeMultiBase`) — **DISTINCT from the
`0x6610` MembershipSet group; it is NOT a MembershipSet**. It carries the same recipient-roster social-graph leak
(#61-class), so it is BLINDED the same way. **8 fields:**

| Field | Width | Notes |
|---|---|---|
| `aad_version` | `0x01`, u8 | |
| `codepoint` | `0x6520`, u16 BE | committed in AAD |
| `body_cid` | self-describing CIDv1 (36B) | |
| `recipient_count` | **u16 BE** | the **Layer-C drop band width** (NOT the MembershipSet band width — the §4.0 width-unification-REJECTED note governs) |
| `audience_set_commitment` | 32B | `BLAKE3(0x01 ‖ lp(did_0) ‖ lp(did_1) ‖ …)` — IDENTICAL construction to `0x6610` |
| `stanza_index` | u32 BE | |
| `stanza_count` | u32 BE | relay-truncation/censorship defense |
| `recipient_key_generation` | u32 BE | |

**UNLIKE `0x6610`:** `0x6520` carries **NO `membership_set_id_commitment`, NO `membership_set_generation`, NO
`role_assignments_generation`** (those are MembershipSet-only fields). The sealed-inner-sender-DID stays INSIDE the
ciphertext per stanza (post-decrypt-verified; F-LC-9 / BR-1). Honest scope: identity-HIDING, not unlinkability (the
commitment recurs for a static recipient set); full per-send unlinkability = U25, CODEPOINT-RESERVE for v1-GM,
additive with no wire-break.

---

## §4.1 — Sealed-Sender property + the per-stanza-LIVE binding (the proof shape)

**Recipient-key premise (REAL, not a placeholder — R9 GAP-1).** Every claim below stands on the CEK being
HPKE-key-wrapped to a **REAL hybrid recipient key**: the seal path
(`benten_drop::layer_c::seal_sealed_sender` / `seal_group_multi`) takes a `&RecipientPublic` (`&[RecipientPublic]`
for the group) and the open path (`open_single` / `open_group_stanza`) takes a `&RecipientSecret`, both re-exported
by `benten-drop` from `benten_crypto_suite::cipher_suite`. The secret is an ML-KEM-768 decapsulation key ‖ X25519
static secret carrying genuine OS-RNG entropy, **unrecoverable from the public key**. (The pre-fix corpus base wrapped
to a `[u8; 32]` public *fingerprint* and reconstructed the "secret" from that public via `sk = pk + 0x80` — ZERO
secret entropy, so any public-key holder could recover the CEK. That placeholder derivation is **DELETED**; a
non-matching secret now yields a different X-Wing shared secret and the CEK-unwrap fails closed.) All the
confidentiality + non-forgeability properties below assume, and now genuinely have, a recipient secret that only the
intended recipient holds.

**Sealed-Sender (BR-1 / F-LC-9 — RATIFIED).** EVERY group send is Sealed-Sender by default: the inner-sender-DID is
bound **INSIDE** the sealed/encrypted part **per stanza** (HPKE inner-payload sender-DID + post-decrypt-verify),
NOT in the plaintext on-wire AAD. `sender_did` is NOT a plaintext AAD field on the default path.

**The per-stanza-LIVE decomposition (the load-bearing binding property).** Each stanza independently binds its
own AAD field-set under its own HPKE-derived AEAD key, so **each stanza independently verifies**:

- **Cross-stanza substitution defense (U17):** a relay cannot move a stanza from one envelope into another — the
  per-stanza AAD (codepoint + `body_cid` + `audience_set_commitment` + `stanza_index` + `stanza_count`) binds the
  stanza to its envelope context; substitution flips the AEAD tag.
- **Truncation / censorship defense:** `stanza_count` is bound alongside `stanza_index`, so dropping trailing
  stanzas to censor a co-recipient is detectable — each surviving stanza still names the original `stanza_count`,
  which no longer matches the delivered count.
- **Inter-member non-forgeability (B2 ORIGIN-AUTHENTICATION — NOW TRUE in code).** A member who legitimately
  holds the bulk-CEK and can produce valid AEAD tags — a `0x6610` member holding `K_Set` (which derives the
  `0x6610` CEK), OR a `0x6520` **co-recipient** who HPKE-unwraps the fresh-random group CEK from its own stanza
  (R11 MC-1) — **cannot** mint a send attributed to another member, nor re-target another member's real body to a
  recipient set that member never chose. The AEAD tag alone CANNOT provide this (a CEK-holding co-member can
  produce a valid tag), so the property rests on a real signature, NOT on the un-authenticated sealed-inner-DID
  parse. Each Sealed-Sender send carries, **inside the once-sealed
  body region** (on the wire exactly ONCE; sender-confidential), a single per-MESSAGE LAMPS-hybrid
  `id-MLDSA65-Ed25519-SHA512` (`0x0001`) signature over a domain-separated binding `M_auth`
  (`SENDER_AUTH_DOMAIN` ‖ sig/envelope codepoints ‖ sender-DID ‖ `body_cid` ‖ audience commitment ‖ key-epoch
  generations ‖ `stanza_count` ‖ body-AAD digest). Each recipient resolves the recovered sender-DID to its
  **hybrid** verifying key (self-certifying `did:key`, two-component multikey; `Did::resolve_hybrid`) and
  cryptographically verifies **both halves** post-decrypt, fail-closed (`SenderOriginAuthFailed`). **Soundness
  (F-2):** the recipient re-derives the audience commitment + the key-epoch generations from the set-state it
  INDEPENDENTLY HOLDS (its own roster / `K_Set` / held generations), NEVER the attacker-controllable wire value —
  so a re-target (re-wrap to a new set) flips the commitment and a stale-generation replay (revoked-member
  cross-generation) flips a generation word, both fail-closed. Forging an attribution requires the target's
  hybrid signing key (post-quantum-secure). The construction (`benten_drop::layer_c` seal/open + the substantive
  `f_lc_3_*` pins in `crates/benten-drop/tests/f_lc_hpke_encrypt_to_recipient_sealed_sender.rs`:
  `f_lc_3_second_sealer_spoof_rejected_single` / `f_lc_3_second_member_spoof_rejected_membership_group` /
  `f_lc_3_second_sealer_spoof_rejected_layer_c_group` / `f_lc_3_retarget_to_new_audience_rejected` /
  `f_lc_3_stale_generation_replay_rejected` / `f_lc_3_strip_pq_half_rejected_single`) makes this claim TRUE; design
  record `.addl/phase-4-meta/sealed-sender-auth-design.md`. Positive cross-records: `docs/THREAT-MODEL.md` §1 +
  `docs/SECURITY-POSTURE.md` Compromise #43 (the inter-member non-forgeability positive-property notes).

These properties hold **per stanza, independently** for the per-stanza defenses (U17 / truncation), and **per
message** for the origin-auth signature (one signature authenticates the body to the whole audience-SET at
constant cost) — which together is exactly what makes the group send robust to active relays AND to malicious
co-members. The IND-CCA2-under-adversarially-chosen-recipient-seed tractability (M-6) — relevant because
device-link / remote-permission flows admit a chosen-recipient-pubkey surface — is named as an
**external-cryptographer-audit deliverable** (§9.3 audit line; Compromise #45 / #59), NOT a unit-test "proof" in
this doc.

**DropBundle envelope-signature is INTEGRITY-only, NOT origin-authority (R21 F-10).** The outer DropBundle
`envelope_sig` (`ENVELOPE_SIG_DOMAIN`; `benten_drop::bundle::verify_envelope_signature`) verifies the header bytes
under the bundle's OWN `issuer_verifying_key` — an attacker-controllable field anchored to nothing on its own, so a
valid envelope-sig proves only header integrity, never who authored the bundle. **Origin AUTHORITY comes
exclusively from the `auth_grant`** (the ONE signed `AuthorizationGrant` / UCAN, whose issuer is cryptographically
self-bound): `consume_offline` verifies the grant binding AND requires `bundle.issuer_verifying_key ==
auth_grant.issuer_verifying_key`, so a re-authored-header + fresh-key-re-sign ("signature-by-nobody") is rejected
at the grant-anchor check (the F-INJ-2 / D-42 closure). Never treat the envelope-sig as an origin-authenticator.

**Inner-format domain-separation (single vs group).** The single
(`benten_drop::layer_c::seal_inner`, `0x6500`/`0x6510`) inner format is domain-separated from the group
(`benten_drop::layer_c::seal_group_impl`, `0x6520`) inner format by **independently-keyed CEKs plus the outer
per-stanza AAD context**, **NOT** by the inner payload bytes themselves: a single-format inner and a group-format
inner are sealed under CEKs that can never coincide — the single CEK is a body-mixing BLAKE3 derivation under the
`"benten-drop:layer-c:cek"` separator, while the group CEK is a **fresh random per-message value** (R11 MC-1, no
longer derived under `"benten-drop:layer-c:group-cek"`) — and each is bound to a distinct AAD shape, so neither
inner can be reinterpreted as the other (cross-format substitution flips the AEAD tag). The property holds in the
current code; documenting it here prevents a future inner-builder refactor (e.g. unifying or re-laying-out the
inner bytes) from silently regressing it by accidentally collapsing the CEK/AAD distinction.

**Cross-surface domain-tag registry (prefix-free) — the v1-beta structural shape.** The single-vs-group CEK
separation above is one instance of a **substrate-wide property**: every cryptographic surface that keys, signs,
or AAD-commits draws its domain tag from a **single prefix-free registry** of separators. The registry's governing
property is **mutual prefix-freedom** — for any two registered tags `a ≠ b`, neither is a byte-prefix of the
other. This is what makes cross-surface confusion structurally impossible: because no tag prefixes another, bytes
derived/authenticated under one surface's tag can never be parsed or re-keyed as another surface's input, even
under adversarial concatenation/length-extension framing. The registered surfaces span the Layer-C single/group
CEK derivations, the chunked-AEAD info strings (`benten-aead:{whole,chunk,recipe}:`), the sender-auth /
envelope-signature binding domains (`SENDER_AUTH_DOMAIN` / `ENVELOPE_SIG_DOMAIN` — see §4.1 `M_auth`), the
remote-grant / remote-request / exec-workflow AAD domains, the MembershipSet set-id (`benten:setid:v1`),
the `K(V)` / `K(N)` keying-glue contexts, the Layer-A vault AAD label
(`benten-vault:`) and DAK HKDF info-tag (`benten-dak-v1`), and the deterministic recipient-seed expansion label
(`benten-crypto-suite:recipient-seed`). **Permanence:** the prefix-free property is the
PERMANENT v1-beta commitment; the registry contents are additive (a new surface registers a new tag, which MUST
clear the prefix-free check — a colliding or prefixing tag fails the build). A workspace regression test asserts
mutual prefix-freedom over the whole registered set, so a future tag mint that would prefix an existing tag
(e.g. minting `"benten-drop:layer-c:cek-v2"` while `"benten-drop:layer-c:cek"` exists) fails CI rather than
silently opening a cross-surface confusion path. (Cross-ref: `docs/THREAT-MODEL.md` §5 T-DOMSEP / T-DOMSEP-MIT
for the threat statement. The centralizing registry CODE + its prefix-free regression test have SHIPPED at
`crates/benten-crypto-suite/src/domain_registry.rs` — a 19-tag corpus enumerated by `registered_domain_tags()`
with the `all_domain_tags_are_prefix_free` regression; this property records the structural shape that code
realizes. **The §3.9 gossip-topic derivation is deliberately NOT a registered tag** — it is
`blake3::keyed_hash(K_Set, membership_set_id ‖ BE(generation))` with NO domain-separation label (R0.7 §3.9
authoritative, golden byte-confirmed): its keyed preimage shape, not a label string, is the separator, so there
is no tag to register.)

---

## §4.2 — Deterministic-CEK confirmation-oracle property (GAP-2 honest disclosure)

**Band scope (R11 MC-1).** This deterministic-CEK property is specific to the **single-recipient** Layer-C bands
(`0x6500`/`0x6510`, `benten_drop::layer_c::seal_inner`). The **`0x6520` group** CEK is a **fresh random per-message
value** sampled from the OS CSPRNG (R11 MC-1), delivered ONLY via each stanza's HPKE-wrap — it is NOT derived from
any wire input, so the group band has **NO** CEK confirmation-oracle at all (a party without a recipient secret
cannot even recover the group CEK; see §4.1 and the `mc_1_non_recipient_cannot_recover_group_cek` pin). The prior
`0x6520` CEK derivation from PUBLIC inputs (`body_cid ‖ sender_did ‖ generation`) was a confidentiality break —
any relay guessing the sender's public DID could recompute the CEK and decrypt the group body — and is **DELETED**.
The residual `body_cid` low-entropy disclosure below applies to **all** bands (it is a property of the wire
`body_cid`, not the CEK). The deterministic-CEK confirmation-oracle below applies **directly** to the
single-recipient bands (`0x6500`/`0x6510`); the **`0x6610` MembershipSet group** CEK is `K_Set`-keyed and therefore
**body-deterministic** (UNLIKE the fresh-random `0x6520` group CEK), so a member already holding `K_Set` has the
same guess-confirmation capability for a `0x6610` send — but that capability is **strictly subsumed by the keyless
`body_cid` oracle** (which confirms a low-entropy body with NO key material at all, `K_Set` or otherwise), so it
discloses nothing beyond `body_cid`. Only `0x6520` — fresh-random CEK — has no deterministic-CEK
confirmation-oracle of any kind; do NOT lump `0x6610` in with it.

**Property (single-recipient bands; additive disclosure; does NOT weaken any claim above).** For the
single-recipient bands the content-encryption key (CEK) is **deterministically derived** from the plaintext
context, not freshly random:
`CEK = BLAKE3("benten-drop:layer-c:cek" ‖ recipient_pk ‖ sender_did ‖ aad ‖ body)` (`benten_drop::layer_c::seal_inner`).
Because the CEK is a deterministic function of the body, **a party that holds (or can recompute) the CEK can
*confirm* a guessed plaintext**: re-deriving the CEK over a candidate `body` and checking it matches the bound
key (equivalently, re-sealing the guess and comparing the recovered CEK / AEAD-decryptability) reveals whether
the guess equals the real plaintext. This is the standard **confirmation-oracle** consequence of
deterministic-key derivation — it gives a guess-checking advantage to a party who ALREADY holds the CEK-derivation
inputs (the sealer itself, or a co-recipient that legitimately recovers the CEK), it does **NOT** grant plaintext
recovery to a party who does not.

**Why this does NOT break confidentiality against the relay.** The bulk AEAD seal uses a **fresh random nonce per
send** (`ChaCha20Poly1305::generate_nonce(&mut OsRng)`, `benten_crypto_suite::aead::wrap`), and the CEK is
**HPKE-key-wrapped to the recipient's REAL hybrid public key** — the relay never sees the CEK, and the CEK can be
recovered **only** by a holder of the matching REAL hybrid recipient SECRET (ML-KEM-768 decapsulation key ‖ X25519
static secret; `benten_drop::layer_c::open_single` / `open_group_stanza` take a `&RecipientSecret`, unwrap via
`benten_crypto_suite::cipher_suite::CipherSuite::unwrap_key_material`, and **fail closed** on any non-matching
secret). The recipient secret carries genuine OS-RNG entropy
(`benten_crypto_suite::cipher_suite::CipherSuite::generate_recipient_keypair`) and is **NOT recoverable from the
recipient public key** — a public-key-only party derives a different X-Wing shared secret and the ChaCha20-Poly1305
CEK-unwrap fails closed. Consequently:

- A **network observer / untrusted relay** (Tier-1; holds neither the CEK nor its derivation inputs) gains no
  confirmation oracle or equality test **from the CIPHERTEXT**: the random nonce makes two seals of the same
  plaintext produce distinct ciphertext bytes, and the wrapped CEK is opaque. The relay-facing *ciphertext*
  confidentiality claim of §3.3 / §4.1 is **UNCHANGED**.
- **HOWEVER — `body_cid` low-entropy confirmation/equality-linkability (honest disclosure).** The wire `body_cid`
  is an **unsalted** `self_describing_cid(BLAKE3(plaintext))` (`benten_drop::layer_c::self_describing_cid` over
  `blake3::hash(&body)`) and is emitted **in plaintext** in every Layer-C AAD
  (`0x6500`/`0x6510`/`0x6520`/`0x6610`). It therefore DOES give the Tier-1 observer two capabilities that the
  ciphertext denies it, both bounded to **LOW-ENTROPY / guessable** bodies:
  (1) a **confirmation oracle** — guess a candidate `body`, compute `self_describing_cid(BLAKE3(guess))`, and
  compare against the wire `body_cid`; a match confirms the plaintext with no key material at all; and
  (2) a **plaintext-equality linker** — two sends of the **identical** body carry the **identical** `body_cid`,
  so the relay can link "same plaintext body" across sends (independent of the random-nonce ciphertext
  distinctness). For **high-entropy** bodies both capabilities are computationally infeasible (the guess space is
  intractable). **Mitigations:** senders with low-entropy-plaintext concerns should **pad / randomize the body at
  the application layer** (this also mitigates the CEK confirmation oracle above); and a **per-send `body_cid`
  salt** is additive over the field (codepoint-reserve, no wire-break per CLAUDE.md baked-in #5 crypto-agility) if
  the `body_cid` oracle is later judged load-bearing. The `body_cid`-in-AAD is deliberate — it is the
  origin-auth-binding + U3 length-injectivity anchor (`open_group_stanza` recomputes and fail-closes on mismatch);
  this disclosure is DOC-ONLY and changes no wire byte.
- The confirmation advantage is bounded to a party that can already reconstruct the CEK-derivation inputs
  (`recipient_pk`, `sender_did`, `aad`, and a *candidate* `body`) — i.e. the sealer, or a co-recipient holding the
  recovered CEK. For low-entropy / guessable plaintexts (short enumerable messages, known templates) such a party
  can **confirm** a guess. This is an accepted property at v1-beta: senders with low-entropy-plaintext concerns
  should pad / randomize the body (application-layer mitigation), and a future per-send CEK salt is additive over
  the field (codepoint-reserve, no wire-break) if the oracle is later judged load-bearing.

**Scope.** This is an honest disclosure of a known deterministic-encryption property, NOT a confidentiality break
against the wire adversary the threat model targets. Cross-link `docs/THREAT-MODEL.md` §1 (Tier-1 network observer
sees no plaintext) + Compromise #43 (envelope-metadata leakage) in `docs/SECURITY-POSTURE.md`.

**Post-decrypt failure-variant scope (R15 F-10; low-materiality; distinct axis).** The confirmation-oracle axis
above concerns a party GUESSING the plaintext. On the orthogonal *co-recipient error-classification* axis, note that
all bands collapse to a **single confidentiality-boundary failure** before any structural detail is revealed: a
party without the recipient secret cannot AEAD-open at all and gets exactly `AeadAuthenticationFailed` (no branch on
inner structure). The finer post-decrypt variants (`MalformedInnerPayload` structural-decode vs
`SenderOriginAuthFailed` wrong-signer) are reachable **only** by a party that has ALREADY AEAD-opened the inner
payload (a legitimate co-recipient / CEK holder), so they leak nothing across the confidentiality boundary — they
are diagnostic distinctions available only to a party already entitled to the plaintext, NOT a decryption oracle to
an outside adversary. This is low-materiality and distinct from the §4.2 plaintext-guessing axis.

**F-07 (R20) — the pre-decrypt `StanzaCountMismatch` check is secret-INDEPENDENT / non-oracular; narrow any absolute
"before any decrypt" wording accordingly.** The `0x6520`/`0x6610` group open path fails closed with
`StanzaCountMismatch` when `stanzas.len()` (the DELIVERED stanza count) does not equal the per-stanza-bound
`stanza_count`, BEFORE any AEAD-open (`benten_drop::layer_c` — the truncation/censorship defense comment there).
This check is a comparison of two PUBLIC, wire-visible integers (the delivered count vs the count bound in the
relay-visible AAD) — it does NOT branch on any secret, key, or plaintext, and it is reachable by any party
including the relay. So the "fail closed BEFORE any decrypt" wording is a truncation-detection statement, NOT an
oracle: the pre-decrypt check leaks nothing that the plaintext AAD does not already expose, and a relay that also
rewrites the per-stanza `stanza_count` makes the AEAD-open fail (counts are bound under the tag). Narrow any
absolute reading of "before any decrypt" to "this is a public-integer structural check, secret-independent and
non-oracular — it detects relay truncation, it is not a decryption oracle."

**F-11 (R20) — the `0x6610` group AAD length-prefix framing (the "coarsening") is secret-independent / non-oracular.**
The `0x6610` MembershipSet group per-stanza AAD binds its variable-length fields under u32-BE length prefixes
(the `audience_set_commitment` is `BLAKE3(0x01 ‖ lp(did_0) ‖ lp(did_1) ‖ …)` over the canonical SORTED recipient-DID
list, lp = u32-BE; §3.3). This length-prefix framing is a length-INJECTIVITY / domain-separation device over
PUBLIC roster material (recipient DIDs + counts already relay-visible in the plaintext AAD), computed with no
secret input — it neither derives from nor discloses any key or plaintext. It is secret-independent and
non-oracular: it exists to make the AAD parse unambiguous (U3 length-injectivity), not to hide or reveal
anything secret. Any wording implying the length-prefix width carries confidentiality significance should read
"length-injectivity framing over public roster material — secret-independent, non-oracular."

---

## Cross-references

- `docs/CRYPTO-CODEPOINTS.md` — the codepoint allocation (`0x6510` / `0x6610` / `0x6520` rows) + the
  width-unification-REJECTED freeze note.
- `docs/THREAT-MODEL.md` — the network-observer-only unlinkability scope this per-stanza binding underwrites + the
  O-6 blast-radius ladder.
- `docs/SECURITY-POSTURE.md` — Compromise #43 (metadata leakage), #45 (MAL-BIND), #58 (insider-correlation), #59
  (KEM-key-confirmation), #61 (gossip blinding).
