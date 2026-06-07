# Threat Model — Benten encryption + MembershipSet substrate (Phase-4-Meta-Core)

This document records the v1-beta threat model for the encryption / identity / MembershipSet substrate: the
trust-tier × operation matrix, the **O-6 blast-radius ladder** (what each compromised credential or device
exposes), the **network-observer-only** scope of per-recipient unlinkability, and the NQ-T1..T4 dispositions
(threat-model R2 questions, all Ben-ratified 2026-06-02). It is a companion to `docs/SECURITY-POSTURE.md` (the
named Compromise table) + `docs/SECURITY-PROOFS.md` (the FROZEN AAD field-sets) + `docs/CRYPTO-CODEPOINTS.md`
(the codepoint allocation).

---

## §1 — Trust-tier × operation matrix

The trust tiers, from least- to most-trusted relative to a principal's plaintext:

| Tier | Who | Sees plaintext? | Sees metadata? |
|---|---|---|---|
| **Network observer / untrusted relay** | an iroh relay, a passive network adversary, a storage host holding ciphertext | **NO** | gossip topic (blinded), Drop envelope sizes + timing, blinded group tags (`audience_set_commitment`, `membership_set_id_commitment`) — **opaque 32-byte tags, NOT the roster** |
| **Untrusted host** (peers-hold-ciphertext) | a peer storing a principal's encrypted partition (Phase-7 Garden-Grove) | **NO** (per-Node AEAD + Layer-A vault seal the bytes) | blob CIDs + access patterns |
| **Co-recipient member** | a member of a MembershipSet holding `K_Set` | **YES** for content they are entitled to | the member roster (recomputes the blinded commitments from the member list they hold) |
| **Admin** | a MembershipSet admin holding the audit log + `members_table` | **YES** for set content + **CAN correlate members** | full audit-log visibility (Compromise #58) |
| **Coerced device** | a device whose holder is compelled to unlock / approve | **YES** (whatever that device is entitled to) | n/a — OUT-OF-SCOPE (Compromise #33) |

**The load-bearing scope boundary.** Per-recipient unlinkability (the `0x6610` / `0x6520` group-AAD blinding) is
**network-observer-only** (see §3). Capability-gating binds only a *cooperating* engine; on an *untrusted host*,
**encryption** (per-Node AEAD + Layer-A vault) is the load-bearing confidentiality substrate (CLAUDE.md baked-in
#18 — the confidentiality half of the Principal primitive).

**Deterministic-CEK confirmation-oracle (additive disclosure; GAP-2).** The Layer-C content-encryption key is
deterministically derived from the plaintext, so a party that **already holds the CEK** (the sealer, or a
co-recipient that recovers it) can **confirm a guessed plaintext** — a confirmation oracle for low-entropy bodies.
This does **NOT** extend to the **Tier-1 network observer / untrusted relay**: the bulk AEAD uses a **fresh random
nonce per send** and the CEK is HPKE-wrapped to the recipient, so the relay sees neither the CEK nor a
plaintext-equality test (the Tier-1 "sees plaintext = NO" row is unchanged). Full cryptographic narration:
`docs/SECURITY-PROOFS.md` §4.2.

---

## §2 — O-6 blast-radius ladder (what each compromise exposes)

The blast-radius ladder names exactly what is exposed when each credential / device / key is compromised — so the
exposure is bounded + disclosed, never silently total:

1. **K_principal vault password / DAK compromise** → the **whole principal partition** for that device (the
   Argon2id-derived DAK unlocks the Layer-A vault). This is the top of the ladder. Cross-link Compromise #34
   (password-knowledge implies full access).
2. **A single device long-term key compromise** → everything **that device** held / was entitled to, and
   retroactive decryption of content it already held (no past-content FS at v1-beta). NOT the other devices in
   the mesh. Cross-link Compromise #35 (compromised-device retro-decrypt); Compromise #42 (HPKE long-term-sk
   non-FS).
3. **A per-Node AEAD key compromise** → exactly **that Node's** plaintext (the per-Node wrap is the granularity).
   Cross-link the per-Node AEAD rebinding-attack-prevention section in `docs/SECURITY-POSTURE.md`.
4. **A recipient long-term secret key (HPKE-mode-base) compromise** → **every envelope ever sent to that
   recipient** (HPKE-mode-base is structurally non-FS at the long-term-sk axis). Cross-link Compromise #42 / #48.
5. **A MembershipSet `shared_key` (K_Set) compromise** → that **generation's set shape** (a fingerprint of who is
   in the set) + the set content for that generation; recovery is **fork-only** rotation. NOT future generations
   after a fork. Cross-link Compromise #48 (shape-leak); Compromise #52 (no-PCS-against-removed-members).

The ladder is monotone: a compromise lower on the ladder (per-Node key) exposes strictly less than one higher
(vault DAK). The defenses that bound each rung — tight UCAN `nbf`/`exp`, fork-on-kick rotation, per-Node AEAD
granularity, the typed-reject-never-silent-fallback contract — are named at each rung's cross-linked Compromise.

---

## §3 — Per-recipient unlinkability scope = NETWORK-OBSERVER-ONLY (load-bearing)

The Layer-C group-AAD blinding (`0x6610` MembershipSet group + `0x6520` Layer-C multi-recipient) replaces the raw
recipient roster + raw set-id with blinded commitments (`audience_set_commitment` = BLAKE3 over the canonical
sorted DID list; `membership_set_id_commitment` = `blake3::keyed_hash(K_Set, "benten:setid:v1" ‖ id)`). This
achieves **per-recipient unlinkability** against a **network observer / untrusted relay** — the relay sees only
opaque 32-byte tags, never the roster.

**This unlinkability is explicitly NETWORK-OBSERVER-ONLY. It is NOT admin-proof.** A member-or-admin who holds
`K_Set` + the member list **recomputes** the commitments and **CAN correlate members** — the property is scoped,
not absolute:

- A **network observer** cannot link two per-recipient stanzas to the same recipient from the wire bytes (the
  unlinkability property the design DELIVERS).
- A **malicious admin** holding the `members_table` **CAN correlate members** (the BOUNDARY — the property is
  **network-observer-only**, NOT admin-proof). This is the honest disclosure recorded as **Compromise #58**
  (audit-log insider-correlation) in `docs/SECURITY-POSTURE.md`. The threshold-admin opt-in (no single admin
  sees the full audit-log) closes the insider vector for deployments that adopt it.

Honest scope of the blinding: it achieves **identity-HIDING**, NOT full unlinkability — the same commitment
recurs for a static recipient set, so a network observer can still link sends to "the same unknown group". Full
per-send unlinkability (salt/nonce-rotated commitments) is **U25, CODEPOINT-RESERVE for v1-GM**, additive over
the field with no wire-break. Cross-link Compromise #43 (envelope-metadata leakage); Compromise #61
(gossip-topic blinding).

---

## §4 — NQ-T1..T4 dispositions (threat-model R2 questions; Ben-ratified 2026-06-02)

### NQ-T1 — audit-Node encryption + replication

The `PermissionGrant` audit-Node is **encrypted + replicated to all of the user's devices** so a malicious device
cannot grant-and-hide: the audit log is graph-native version-Node content on the enforced-WRITE path (m-15 GNC-2),
replicated via the same sync substrate as other content. A coerced/malicious device's grant is visible on the
user's other devices (subject to the cross-device sync window — see NQ-T4 / Compromise #64).

### NQ-T2 — `valid_until` clock vs the 1-hour metadata bucket (RATIFIED)

The 1-hour metadata bucket (round-down) and the `valid_until` enforcement clock are **orthogonal,
separately-encoded fields**. **Rule:** `valid_until` is encoded at **full 1-second granularity** and enforced
**STRICTLY** — `present > valid_until → reject`, with **NO grace / skew window**; the coarse 1-hour bucket is
**never consulted for expiry**. A coarsened bucket therefore cannot widen the coercion / replay window (it does
not touch the enforcement clock). (The "≥1-year grace" at §3.3 is the recipient-side Drop key-retention window —
Compromise #62 — NOT a `valid_until` grace.)

### NQ-T3 — `ExecuteWorkflow` no-egress enforcement (RATIFIED)

The frozen 3-field `ExecuteWorkflow` AAD tuple `(executor_did, max_decrypt_count, result_recipient_pubkey)` is
**SUFFICIENT to express** the no-egress / bounded-decrypt constraint. **Rule:** the AAD-bound 3-field constraint
is frozen at v1-beta; **runtime enforcement** (that a rented executor cannot exfiltrate plaintext beyond
`result_recipient_pubkey` via EMIT/WRITE) **stays post-v1-beta — it is NOT freeze-gating** (M-3). Freezing the AAD
scope now is what makes the post-v1-beta enforcement non-wire-breaking.

### NQ-T4 — nonce-cache spec + the cross-device replay window (RATIFIED → Compromise #64)

**Rule:** the nonce-cache is **`jti`-keyed**, **durable** (survives engine restart — persisted, not RAM-only), and
retention is **≥ the full 1-hour bucket window**. **Scope:** **per-device-durable is GUARANTEED** (a nonce
consumed on a device cannot be replayed against that same device); **user-global is best-effort-eventual-via-sync
(NOT synchronous)** — a nonce consumed on device B is rejected on device C only after sync propagates the cache
entry. The pre-sync cross-device replay window is DISCLOSED as a named Compromise: **Compromise #64**
("best-effort-eventual cross-device nonce-rejection window") in `docs/SECURITY-POSTURE.md`.

**Nonce-cache is orthogonal to the metadata bucket (NQ-C5).** The epoch bucket
(`(raw_unix_secs / 3600) * 3600` — round-DOWN, NO jitter, deterministic) is for metadata privacy; the nonce-cache
is for replay defense. They are tuned independently — the nonce-cache window does NOT need widening to track the
bucket. The nonce-cache spec narration (the §3.10 reference) lives here in NQ-T4; the §3.9 unlabelled gossip-topic
derivation (`blake3::keyed_hash(K_Set, set_id ‖ BE(generation))`) is the SEPARATE construction documented in
`docs/CRYPTO-CODEPOINTS.md` (§3.9 vs §3.10).

---

## Cross-references

- `docs/SECURITY-POSTURE.md` — Compromise #33 (coercion OOS), #34 (password-knowledge), #35 (device retro-decrypt),
  #42 (HPKE FS-gap), #43 (envelope-metadata), #48 (set shape-leak), #52 (fork-on-kick), #58 (insider-correlation),
  #61 (gossip-topic blinding), #62 (revocation-reach), #64 (cross-device nonce window).
- `docs/SECURITY-PROOFS.md` — the per-stanza AAD binding (the cryptographic basis for the §3 unlinkability scope).
- `docs/CRYPTO-CODEPOINTS.md` — the codepoint allocation + the §3.9 gossip-topic vs §3.10 AAD-commitment distinction.
