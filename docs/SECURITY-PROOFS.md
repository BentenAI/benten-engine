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

The group `EnvelopePayload::HpkeMultiBase` for a MembershipSet. **11 fields**, BLINDED:

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

`0x6520` is the Layer-C **multi-recipient** group send (`EnvelopePayload::HpkeMultiBase`) — **DISTINCT from the
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
- **Inter-member non-forgeability (B2 ORIGIN-AUTHENTICATION — NOW TRUE in code).** A member — even one holding
  `K_Set` and thus able to derive the CEK and produce valid AEAD tags — **cannot** mint a send attributed to
  another member, nor re-target another member's real body to a recipient set that member never chose. The AEAD
  tag alone CANNOT provide this (a co-member can produce a valid tag), so the property rests on a real signature,
  NOT on the un-authenticated sealed-inner-DID parse. Each Sealed-Sender send carries, **inside the once-sealed
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

**Inner-format domain-separation (single vs group).** The single (`benten_drop::layer_c::seal_inner`) and group
(`benten_drop::layer_c::seal_group_impl`) inner formats are domain-separated by the distinct CEK
domain-separators (`"benten-drop:layer-c:cek"` vs `"benten-drop:layer-c:group-cek"`) plus the outer per-stanza
AAD context, **NOT** by the inner payload bytes themselves: a single-format inner and a group-format inner are
sealed under independently-derived CEKs and bound to distinct AAD shapes, so neither can be reinterpreted as the
other (cross-format substitution flips the AEAD tag). The property holds in the current code; documenting it here
prevents a future inner-builder refactor (e.g. unifying or re-laying-out the inner bytes) from silently
regressing it by accidentally collapsing the CEK separator or AAD distinction.

---

## §4.2 — Deterministic-CEK confirmation-oracle property (GAP-2 honest disclosure)

**Property (additive disclosure; does NOT weaken any claim above).** The Layer-C content-encryption key (CEK) is
**deterministically derived** from the plaintext context, not freshly random:
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
**HPKE-key-wrapped to the recipient** — the relay never sees the CEK. Consequently:

- A **network observer / untrusted relay** (Tier-1; holds neither the CEK nor its derivation inputs) gains **no
  confirmation oracle and no equality test**: the random nonce makes two seals of the same plaintext produce
  distinct ciphertext bytes, and the wrapped CEK is opaque. The relay-facing confidentiality claim of §3.3 /
  §4.1 is **UNCHANGED**.
- The confirmation advantage is bounded to a party that can already reconstruct the CEK-derivation inputs
  (`recipient_pk`, `sender_did`, `aad`, and a *candidate* `body`) — i.e. the sealer, or a co-recipient holding the
  recovered CEK. For low-entropy / guessable plaintexts (short enumerable messages, known templates) such a party
  can **confirm** a guess. This is an accepted property at v1-beta: senders with low-entropy-plaintext concerns
  should pad / randomize the body (application-layer mitigation), and a future per-send CEK salt is additive over
  the field (codepoint-reserve, no wire-break) if the oracle is later judged load-bearing.

**Scope.** This is an honest disclosure of a known deterministic-encryption property, NOT a confidentiality break
against the wire adversary the threat model targets. Cross-link `docs/THREAT-MODEL.md` §1 (Tier-1 network observer
sees no plaintext) + Compromise #43 (envelope-metadata leakage) in `docs/SECURITY-POSTURE.md`.

---

## Cross-references

- `docs/CRYPTO-CODEPOINTS.md` — the codepoint allocation (`0x6510` / `0x6610` / `0x6520` rows) + the
  width-unification-REJECTED freeze note.
- `docs/THREAT-MODEL.md` — the network-observer-only unlinkability scope this per-stanza binding underwrites + the
  O-6 blast-radius ladder.
- `docs/SECURITY-POSTURE.md` — Compromise #43 (metadata leakage), #45 (MAL-BIND), #58 (insider-correlation), #59
  (KEM-key-confirmation), #61 (gossip blinding).
