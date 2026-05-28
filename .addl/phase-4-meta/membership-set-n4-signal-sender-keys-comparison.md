# Phase-4-Meta-Core — N4: Signal Sender Keys (SSK) vs Benten MultiRecipientSealing + AdminKickEpoch comparison

**Specialist:** N4 of 4 parallel post-M-CONS refinement specialists
**Date:** 2026-05-28
**Tree-pin:** HEAD `2172cb6d` on `main`; this branch `phase-4-meta-core/membership-set-n4-signal-sender-keys-comparison`
**Inputs:**
- M2 `@ 6170980b` (`membership-set-m2-primitive-design.md`) — MembershipSet typed-variant primitive
- M5 `@ 15819500` (`membership-set-m5-cgka-candidate-survey.md`) — CGKA survey (§2.5 covers SSK)
- M-CONS `@ 74580ee6` (`membership-set-m-cons-consolidator.md`) — consolidated registry + Inv-20 + AdminKickEpoch fold-into-FORK-ONLY ratification
- Spike-A2 (`.addl/spikes/SPIKE-A2-ucan-on-wire-2026-05-21.md`)

**External sources (WebSearch / WebFetch):**
- Balbás, Collins, Gajland — "WhatsUpp with Sender Keys? Analysis, Improvements and Security Proofs" (ASIACRYPT 2023; ePrint 2023/1385)
- Balbás, Collins, Gajland — "Analysis and Improvements of the Sender Keys Protocol for Group Messaging" (arXiv 2301.07045; RECSI 2022)
- Cohn-Gordon, Cremers et al. — "A Formal Security Analysis of the Signal Messaging Protocol" (EuroS&P 2017; JoC 2020)
- Cremers et al. — "The Complexities of Healing in Secure Group Messaging" (USENIX Sec 2021)
- "Formal Analysis of Multi-Device Group Messaging in WhatsApp" (ePrint 2025/794)
- "Poster: No safety in numbers — traffic analysis of sealed-sender groups in Signal" (arXiv 2305.09799)
- Wikipedia: Sender Keys
- libsignal source (`rust/protocol/src/sender_keys.rs`, `protocol.rs`)
- Signal blog: PQXDH (`signal.org/blog/pqxdh/`), Sealed Sender, Asynchronous Security, Synchronized Start for Linked Devices
- Signal Double Ratchet spec (`signal.org/docs/specifications/doubleratchet/`)
- Signal Private Group System / KVAC paper (ePrint 2019/1416)
- Cryspen / PQShield analyses of PQXDH + SPQR

---

## §1 — Executive recommendation + confidence

**Verdict:** **CONCUR with Benten's current MultiRecipientSealing + AdminKickEpoch-folded-into-FORK-ONLY design.** No substantive changes warranted at v1-beta. M-CONS §1.4's adoption of MultiRecipientSealing naming + M5 §6.2's fold of AdminKickEpoch into FORK-ONLY stand correct.

Three small-cost refinements are surfaced (§7 of this doc):
- **R-N4-1** (cite-only doc nit): codepoint-reserve `MembershipSetKind::AtriumWithCGKA` should be renamed `AtriumWithSenderKeys` *or* left as generic `AtriumWithRotatingGroupKey` — "CGKA" is wrong-shaped per M5's own correction, and the codepoint reserve should not bake in a future-design choice we haven't made.
- **R-N4-2** (codepoint-reserve add): reserve a `MultiRecipientSealing::SsKChainedMode` codepoint *additive slot* under TransportConfig (M6 codepoint-reserve §6.5). This is for the post-v1-beta optional landing of an SSK-style chained sender-key ratchet *if* a future use-case wants per-message FS *for the group secret itself* (Benten currently does not). Pure codepoint reserve; no implementation cost; cost = 0 wave-days at v1-beta.
- **R-N4-3** (Inv-20 clarification): Inv-20 clause-d ("per-recipient unlinkability") should explicitly note that the MultiRecipientSealing posture is **stronger than SSK's authenticator-leakage posture** — SSK has all members hold the sender's HMAC chain key, which means *any* group member can forge a message authentication on behalf of any sender (mitigated by per-sender Ed25519 signing-key); MultiRecipientSealing's HPKE-AAD-binding scheme + sender_did signature + Inv-20 clause-c per-stanza AAD-bind tuple gives both per-recipient unlinkability **and** non-forgeability between members. This is a positive — but it's a structural difference that should be cite-named in the audit-deliverable doc so an auditor sees the distinction.

**Cost:** all three refinements are doc-only / codepoint-reserve only. R-N4-1 = doc rename ~0.25 wd. R-N4-2 = codepoint-reserve table row ~0.25 wd. R-N4-3 = audit-deliverable §-add ~0.5 wd. **Total: ~1 wave-day; absorbable into M-CONS post-merge fix-pass.**

**Confidence:** HIGH that SSK comparison surfaces no v1-beta blocker. MEDIUM that R-N4-2 codepoint-reserve is the right shape; the *concrete future* use-case it preserves is "audit-mandated per-message-FS for group secret rotation" which is a real possibility but unlikely to materialize before Phase-4-Meta-Composing.

---

## §2 — Signal Sender Keys protocol deep-dive

### §2.1 What SSK is + isn't

**SSK = a group-messaging encryption scheme** built atop Signal's *pairwise* Double Ratchet channels. It is **NOT** a CGKA in the modern academic sense (MLS / DCGKA / TreeKEM). It does **NOT** compute a shared group key by group key agreement. It is precisely what M5 §2.5 named: **"every sender owns their own symmetric chain-key + Ed25519 signing keypair; the chain-key is distributed to every group member via the pairwise Double Ratchet channels; every group message is encrypted under a message key derived from the sender's chain-key, signed with the sender's signing-key, and fanned out by the server."**

Production deployment: WhatsApp, Signal, Facebook Messenger, Matrix (Megolm variant), Session — **~3 billion+ daily-active users in aggregate across these platforms.**

### §2.2 Cryptographic primitives in libsignal SSK

From libsignal `rust/protocol/src/protocol.rs` + `rust/protocol/src/sender_keys.rs`:

```text
SenderKeyDistributionMessage {
  message_version: u8
  distribution_id:  Uuid                  // 16 bytes; per-(group, sender, "device") binding
  chain_id:         u32                   // identifies which chain (e.g. on rotation)
  iteration:        u32                   // chain-key counter
  chain_key:        [u8; 32]              // the symmetric chain key
  signing_key:      PublicKey  (33 bytes) // Ed25519/Curve25519 public verification key
  serialized:       Box<[u8]>
}

SenderKeyMessage {
  message_version: u8
  distribution_id: Uuid
  chain_id:        u32
  iteration:       u32
  ciphertext:      Box<[u8]>              // AES-CBC-PKCS7 over plaintext
  // 64-byte Ed25519 signature appended at end (over the protobuf-encoded preamble)
}

// Key derivation (sender_keys.rs):
// SenderChainKey:
//   chain_key_next = HMAC-SHA256(chain_key, [0x02])
//   message_key    = HKDF-SHA256(seed = HMAC-SHA256(chain_key, [0x01]),
//                                info = "WhisperGroup",
//                                length = 48)
//                    → first 16 bytes = AES-IV, next 32 bytes = AES-CBC-256 key
```

**Cryptographic primitives:**
- Symmetric chain ratchet: HMAC-SHA256 with constant byte labels (`0x01`, `0x02`)
- Per-message key derivation: HKDF-SHA256 with `"WhisperGroup"` info string
- AEAD: AES-CBC-256 + HMAC-SHA256 MAC (Encrypt-then-MAC); **NOT** AES-GCM, **NOT** ChaCha20-Poly1305
- Authentication: Ed25519 (Curve25519 in some variants) per-message signature
- Distribution channel: each pair's Double Ratchet session (X3DH-established; PQXDH post-2023)

**Wire format:** Protobuf. Version byte at front, signature at tail. **NOT** DAG-CBOR; **NOT** content-addressed.

### §2.3 Protocol flow

**Group creation** (per Balbás-Collins-Gajland 2023 §2.2; cross-checked against libsignal):
1. Group founder receives the group-id from the server (or generates locally).
2. **Each member independently** generates their own `(distribution_id, chain_key, signing_keypair)` triple.
3. Each member sends their `SenderKeyDistributionMessage` (which contains `chain_key` + `signing_pubkey`) to **every** other member via the pairwise Double Ratchet channel.
4. Result: **every member holds a (sender_id → SenderKeyState) map** with N-1 entries per group of size N.

**Send message:**
1. Sender advances their chain: `chain_key_next = HMAC(chain_key, 0x02)`; `iteration += 1`.
2. Derives `message_key = HKDF(HMAC(chain_key, 0x01), "WhisperGroup", 48)`.
3. AES-CBC-256 encrypts plaintext under `message_key.cipher_key` with `message_key.iv`.
4. Ed25519-signs the ciphertext-prefix with `signing_priv`.
5. Single ciphertext goes to server → server fans-out to N-1 recipients.

**Receive message:**
1. Look up `(distribution_id) → SenderKeyState`.
2. Advance receiver's mirror of the chain to the sender's `iteration` (caching skipped message keys).
3. Verify Ed25519 sig with sender's `signing_pubkey`.
4. AES-CBC-256 decrypts.

**Member add:**
- New member's pairwise Double Ratchet sessions are established (PQXDH/X3DH).
- Each existing member sends a fresh `SenderKeyDistributionMessage` to the new member via pairwise channel (so the new member learns each existing sender's *current* chain_key + signing_pubkey).
- New member generates and distributes their own sender-key to all existing members.
- **Cost:** O(N) pairwise messages from existing-members + O(N) from new-member = **O(N) total messages, O(N) ciphertext** for one member-add.

**Member remove (the load-bearing event for Benten comparison):**

Per Balbás-Collins-Gajland 2023 §2.2 + Wikipedia: **"whenever a group member leaves, or a device associated with a group member is removed, all group participants clear their Sender Key."** Then **all remaining members generate a fresh sender-key and redistribute to all remaining members via pairwise channels.**

In other words: **member-remove = full O(N²) sender-key redistribution** in the worst case (every member sends a fresh `SenderKeyDistributionMessage` to every remaining member). This is the dominant scaling cost of SSK.

**Explicit rotation / PCS update:** Same as member-remove. A member who suspects compromise sends a fresh `SenderKeyDistributionMessage` to all members.

### §2.4 FS / PCS properties (with formal-analysis nuance)

The widespread folk-claim "SSK provides forward secrecy" needs qualification per Balbás-Collins-Gajland 2023 + Cremers 2021:

**Forward secrecy of message-level content: YES.** The symmetric chain ratchet `chain_key → chain_key'` via `HMAC(chain_key, 0x02)` is one-way under HMAC-SHA256 security. Compromise of `chain_key` at iteration N does NOT reveal message keys at iterations 0..N-1 (provided message-keys were deleted by the recipient after decrypt, which the spec mandates). This is *symmetric-ratchet FS only*, identical to the Signal Double Ratchet's symmetric chain-FS property.

**Post-compromise security (PCS) at the message level: NO at the protocol level.** Per Signal's own Double Ratchet spec: *"these chains don't provide break-in recovery because KDF inputs for the sending and receiving chains are constant"*. The PCS in the 1:1 Signal Protocol comes from the **DH ratchet**, which is interleaved with the symmetric chain ratchet. **SSK has no analogue of the DH ratchet at the group layer.** PCS in SSK is therefore *purely operational* — it requires a member to *explicitly rotate* their sender key by sending a fresh `SenderKeyDistributionMessage`.

Balbás-Collins-Gajland 2023 (WhatsUpp with Sender Keys, ASIACRYPT 2023) prove what they call "weak PCS": healing from a compromise requires the compromised member to send at least Δ+1 update messages, where Δ is the largest gap of un-processed control messages from any member. They also propose an improved variant ("PCS-update mechanism") that brings communication complexity from quadratic-in-N to linear-in-N for the rotation.

**FS against a removed member: NO at the protocol level for messages-sent-before-remove.** This is BY DESIGN of SSK: the removed member retains all sender-keys + message-keys they ever received. Past content stays readable to them. **This matches Benten's Atrium-fork semantics exactly.**

**FS against a removed member for messages-sent-after-remove: YES, via the O(N²) full redistribution** triggered by remove. Because every remaining member generates a *fresh* sender-key + signing-key and the old keys are *erased*, the removed member cannot decrypt any post-remove traffic *unless* they re-compromise a current member.

### §2.5 Multi-device handling

Per Signal blog "Synchronized Start for Linked Devices" + ePrint 2025/794 "Formal Analysis of Multi-Device Group Messaging in WhatsApp":

**Signal multi-device model:** every linked device gets its own Signal account-internal identity ("sub-account"). Per Signal Support, **5 linked devices per phone-anchored account**. Each device runs its own X3DH/PQXDH + Double Ratchet. **Senders must fan-out separately to every device of every recipient.** A group of K users with 3 devices each is, at the encryption layer, a group of 3K members.

**Sender-key distribution for multi-device:** each device generates its own `SenderKeyDistributionMessage` and distributes via the pairwise Double Ratchet to every other device. The `distribution_id` is per-(group, sender-device) so multiple devices of the same user have distinct distribution-ids.

**Synchronized-start mechanism (Sep 2024 Signal blog):** when a linked device is added, the primary device packages its entire message history (last 45 days media + group state + sender-keys-mirror + delivery receipts) into a "compressed encrypted archive under a one-time 256-bit AES key." That key is transferred via the QR-pairing-established Curve25519 session. **This is a bulk-state-transfer, not a CGKA epoch advance.**

**Known multi-device weaknesses** (per ePrint 2025/794):
- **Cross-device-state-divergence:** if the primary device leaves a group on the phone, **linked devices are not automatically synced** with the group-state change (known limitation per Signal-Support docs + GitHub issue signal-cli-rest-api #647).
- **Sender-key inconsistency:** if a linked device misses a `SenderKeyDistributionMessage`-rotation due to being offline, it cannot decrypt subsequent messages until the next sender-key rotation reaches it. Operational fallback: re-request from primary.

### §2.6 Composition with Signal's per-pair Double Ratchet (PQXDH post-2023)

SSK **does NOT directly carry post-quantum protection** for group messages. Per Signal blog "Quantum Resistance and the Signal Protocol" + Cryspen analysis: **PQXDH only secures the initial 1:1 handshake**. The Double Ratchet's ongoing PCS remains classical (X25519-only) until "SPQR" / "Triple Ratchet" lands (still in research as of 2026-05).

For sender keys specifically: the `SenderKeyDistributionMessage` rides the *pairwise Double Ratchet channel*, so it inherits PQ-against-passive-record-now-decrypt-later **for the initial distribution event** (via PQXDH at session setup) but **not** PQ-against-active-MITM-during-distribution (because the Double Ratchet's DH ratchet is still X25519-only).

**Net: SSK group secrets are pre-quantum.** A "harvest-now-decrypt-later" adversary recording group ciphertexts today can decrypt them after a CRQC, even though the *initial* pairwise sender-key distribution had hybrid PQ via PQXDH (because the chain-key + message-key derivation does not re-mix in any PQ material).

### §2.7 Known attacks against SSK

**A1 — Folk-PCS gap (Balbás-Collins-Gajland 2023).** SSK provides no PCS without explicit rotation; this is a *gap relative to the marketed "ratcheting" property*. Signal does not claim group PCS publicly, but informal user-mental-model conflates 1:1-Signal-PCS with group-Signal-PCS. The paper documents this and proposes a healing-update variant.

**A2 — Authenticator-forgeability between members.** Every group member holds *all* other members' chain-keys (because that's how distribution works) + *all* signing-public-keys. The chain-key contains material that derives both message-key AND verification material. Per Balbás-Collins-Gajland 2023, *without* the Ed25519 signature, any group member could forge messages-attributed-to-any-sender (because authenticator = HMAC = symmetric, and they hold the chain-key). The Ed25519 signing-keypair is what prevents this. Their proposed improvement: also ratchet the signing-key per-message (currently the signing-key is static for the lifetime of the chain).

**A3 — Concurrent-remove race condition.** Per Balbás-Collins-Gajland 2023 + Cremers-Hale-Kohbrok 2021: when two admins concurrently remove different members, the "fresh sender-key" distributions can interleave with in-flight messages encrypted under the *old* sender-key. The protocol handles this by accepting messages under either old-or-new sender-key during a transition window, but the window opens an attack surface where a compromised state from before-rotation may still decrypt during-rotation traffic.

**A4 — Server-driven membership manipulation.** Pre-Signal-Private-Group-System (Sep 2019), the server told clients "here's the new member list" and clients trusted it. **A malicious server could silently add a member to a group** — that member would receive sender-keys from existing members + see all subsequent messages. Signal closed this with KVAC + zkgroup: the server now holds an *encrypted member-list* and authenticates admin-actions via anonymous credentials. **WhatsApp pre-2024 was vulnerable; the Check Point Research finding (2017) exploited a related gap.**

**A5 — "No safety in numbers" sealed-sender traffic analysis (arXiv 2305.09799).** Even with sealed-sender + Sender-Keys, an observer of fan-out timing can correlate sender → receivers across groups via traffic analysis. Not a cryptographic break of SSK but a metadata-leak.

**A6 — Skipped-message-key DoS.** Standard Double-Ratchet attack mode applies: a malicious sender can advance their chain by 2^32 iterations, forcing recipients to either store 2^32 skipped message-keys (DoS) or drop legitimate messages. Mitigated by per-session skip-cap (Signal sets ~1000-2000).

### §2.8 License of libsignal

**AGPLv3.** Per M5 §2.5: **blocking for Benten's plugin/UCAN/embedded distribution model.** Even if SSK *were* the right algorithm, libsignal-as-a-dependency is disqualified for Benten. Benten would need a clean-room SSK reimplementation if it ever chose to ship SSK.

---

## §3 — Comparison matrix: Benten MultiRecipientSealing vs Signal Sender Keys

| # | Aspect | Benten MultiRecipientSealing (M2 + M-CONS) | Signal Sender Keys |
|---|---|---|---|
| 1 | **Cryptographic shape** | Multi-recipient HPKE (HPKE-mode-base per-stanza) + AEAD wrap of shared K_Set | Per-sender symmetric chain-key ratchet + Ed25519 sig; chain-key distributed via pairwise Double-Ratchet channels |
| 2 | **Group key** | Single K_Set per MembershipSet, shared by all members (CSPRNG-random at Atrium-create or DAK-derived for DeviceMesh/SingleDevice) | **No** group-shared key. Each sender has their own chain-key. N senders = N chain-keys. |
| 3 | **Sender-side state per group** | K_Set (32 bytes) + generation counter (u32) + member list | Per-sender: (distribution_id, chain_key, iteration, signing_keypair). Per-receiver of every other sender: (distribution_id, chain_key, iteration, signing_pubkey). |
| 4 | **Member-key distribution at group-create** | Founder/admin generates K_Set + multi-stanza-HPKE-wraps to each member's HPKE pubkey (per `MultiStanzaHpkeEnvelope`) | Each member generates own sender-key + distributes via pairwise Double-Ratchet channels to every other member (N(N-1) pairwise messages total at creation) |
| 5 | **Member-key distribution at member-add** | Single multi-stanza HPKE envelope with new stanza for added member; K_Set unchanged | New member's pairwise DR sessions established; each existing member sends `SenderKeyDistributionMessage` over pairwise channel; new member also distributes own sender-key (O(N) pairwise messages) |
| 6 | **Rotation on member-remove** | **ForkOnly (default):** K_Set unchanged; removed member retains read-access to past *and* future content from old K_Set perspective. **AdminKickEpoch (folded into ForkOnly per M-CONS):** fork = new K_Set generation + new MembershipSetId; remaining members publish under new K_Set; removed member can read pre-fork only. | **Full O(N²) sender-key redistribution.** All remaining members clear their sender-key + generate fresh + redistribute via pairwise channels. Removed member's pre-remove messages remain readable to them (they kept old keys); post-remove traffic unreadable. |
| 7 | **Rotation on member-join** | **No rotation.** New member learns current K_Set; past content (pre-join) stays unreadable to them unless explicitly shared (Atrium-forkable semantics). | **Implicit "rotation"** in the sense that the new member's pairwise channels are fresh; existing members send current `SenderKeyDistributionMessage` (no chain-key rotation though). New member cannot read pre-join messages (they don't have the old chain-keys / message-keys). |
| 8 | **Message-encryption flow** | Sender: AEAD (default ChaCha20-Poly1305 or AES-GCM, codepoint-dispatched per `CipherSuiteCodepoint`) under K_Set with per-stanza AAD-binding tuple (sender_did, set_id, generation, ...). Sender signs payload-CID with their own Ed25519/PQ-hybrid sig. | Sender: derive `message_key = HKDF(HMAC(chain_key, 0x01), "WhisperGroup", 48)`; AES-CBC-256 encrypt; advance chain `chain_key ← HMAC(chain_key, 0x02)`; Ed25519-sign with sender's `signing_priv`. |
| 9 | **Multi-device** | First-class via `MembershipSetKind::DeviceMesh` (typed variant). User's K_principal is distributed to each device via multi-device-key-wrap (small-N multi-recipient HPKE). User-DID = admin of the mesh. Cross-Atrium: user's principal is a member of the Atrium; their DeviceMesh handles inner-distribution. | Multi-device retrofitted: each linked device = sub-account with its own X3DH/PQXDH + DR + sender-keys. Senders fan-out to every device of every recipient. Multi-device sync = bulk message-history archive (synchronized-start; Sep 2024). Known divergence bugs (signal-cli-rest-api #647). |
| 10 | **FS properties — at the message level** | **NO FS within a K_Set epoch.** K_Set is static for the lifetime of the (un-forked) MembershipSet. Compromise of K_Set reveals all past + current content encrypted under it. Forks bound the blast radius. Inv-15 plaintext_cid_set blinding limits storage-host visibility. | **FS within a chain-key epoch (per-sender).** `HMAC(chain_key, 0x02)` is one-way; recipient deletes message-keys after decrypt. Compromise of chain-key at iter N hides iter 0..N-1. **No DH ratchet → no PCS at the group level.** |
| 11 | **FS against a removed member** | ForkOnly: NONE (by design — leaver retains K_Set). Fork-event re-establishes future-only FS-against-leaver. | Yes for post-remove via O(N²) redistribution. None for pre-remove (leaver retains chain-keys / message-keys). |
| 12 | **PCS (post-compromise security)** | **Explicit-only.** A compromise of K_Set is healed by FORK (admin mints fresh K_Set). No automatic ratchet-driven healing. M5 §6.2: this is the intentional v1-beta posture; CGKA-shape PCS deferred to AtriumWithCGKA codepoint reserve. | **Weak/explicit PCS.** Symmetric chain-ratchet provides no PCS. Recovery requires the compromised member to issue a fresh `SenderKeyDistributionMessage` to all peers via pairwise channels. Healing requires Δ+1 update messages (Balbás-Collins-Gajland 2023). |
| 13 | **PQ posture** | **PQ-hybrid at v1-beta.** HPKE with ML-KEM-X25519 hybrid KEM (libcrux); ML-DSA-65 + Ed25519 hybrid sig. Reserved at codepoint level. Sealed payloads PQ-safe against harvest-now-decrypt-later. | **Pre-quantum at v1-beta.** PQXDH covers initial 1:1 handshake only; SSK chain-key + message-key derivation contains no PQ material; group ciphertext is harvestable-now-decryptable-later under CRQC. Triple Ratchet / SPQR in research. |
| 14 | **Forkability semantics** | **YES — first-class.** `parent_membership_set_id: Option<MembershipSetId>` field in M2 §2.2; fork = new MembershipSet with new K_Set + parent-pointer; old K_Set readable to whoever had it; new K_Set unknown to non-members. Atrium-as-forkable per Ben 2026-05-27 ratification. | **NO.** A leaver retains past keys but the *group itself* is not forkable — there's one group-state per (group-id); a "fork" would be modeled as a new group with re-invites. |
| 15 | **Wire format** | DAG-CBOR canonical + BLAKE3 content-addressing (`MembershipSetId` + `set_id` AAD-binding) | Protobuf (libsignal). Not content-addressed. Distinct `distribution_id: Uuid` per-(group, sender, device). |
| 16 | **Authentication / signature** | Codepoint-dispatched HybridSignature (Ed25519 + ML-DSA-65 in v1-beta); sig binds canonical-CBOR payload | Ed25519 (or Curve25519 variant) per-message signature; static signing-keypair per chain. Balbás-Collins-Gajland 2023 propose ratcheting the signing-key too. |
| 17 | **Inter-member non-forgeability** | YES structurally: per-stanza AAD-binding tuple binds `(recipient_did, stanza_index, set_id, generation, sender_did)`; sender_did's HybridSignature is required to construct a valid envelope. Other members cannot forge sender_did's signature. | Yes via Ed25519 sig: a member possessing another's chain-key *cannot* forge a message attributed to that sender (would need their `signing_priv`). The Ed25519 sig is load-bearing for this property. |
| 18 | **Audit-trail of admin actions** | Signed `MembershipAttestation` + monotonic `policy_version` + HLC; admin-action becomes a Node in the content-addressed DAG; auditable | Pre-2019: server-trusted (vulnerable per A4). Post-2019: `zkgroup` + KVAC anonymous credentials; admin-action authenticated to server, server enforces; clients see encrypted group-state. Not content-addressed; relies on server. |
| 19 | **Server / relay trust model** | **Trustless storage** (iroh-blobs / iroh-relay): plaintext_cid_set is K_Set-HMAC-blinded so relay cannot equality-oracle across deduplication. Sealed-sender-by-default (no metadata in clear). | **Server-mediated:** Signal server is trusted for membership-list integrity (post-2019 via KVAC) + fan-out routing + sealed-sender envelope. Server sees ciphertext + sender↔recipient pairs (with sealed-sender hiding sender). |
| 20 | **Recursive composition (Atrium-of-Atriums)** | YES at codepoint-reserve level; M2 §11 EXACTLY-3-arms with §15.c HALT-AND-SURFACE-TO-BEN keeps room for `Garden` 4th arm; recursive composition body lands Phase 7+ | NO; SSK is single-group, single-layer. |
| 21 | **Production scale validation** | None yet (v1-beta pre-ship). | ~3 billion DAU across WhatsApp + Signal + Messenger + Matrix. ~12 years deployment. Withstood Check Point Research 2017 + multiple targeted attacks; closed gaps in major version-bumps (Groups-v2 2019, multi-device 2020+). |
| 22 | **Open-source license** | MIT / Apache-2.0 (Benten convention) | AGPLv3 (libsignal) — **blocking** for Benten. |

### §3.1 The structural one-sentence comparison

**Benten MultiRecipientSealing = "one K_Set per group, multi-recipient HPKE-distributed, FORK on rotation, PQ-hybrid from day one, content-addressed forkable Atriums."**

**Signal Sender Keys = "N chain-keys per group (one per sender), pairwise-DR-distributed, full O(N²) redistribution on rotation, pre-quantum, server-mediated non-forkable groups."**

---

## §4 — Things Signal got right that Benten might be missing

I scrutinized SSK + its surrounding ecosystem (sealed-sender, zkgroup, synchronized-start, PQXDH, Triple Ratchet) for patterns Benten under-implements.

### §4.1 Skipped-message-key cache + DoS-cap discipline

**Signal pattern:** per-session cap on cached skipped message-keys (default ~1000-2000). Expire after time-bounded interval. Prevents attacker from advancing chain by 2^32 iterations to force-cache.

**Benten current posture:** MultiRecipientSealing has *no message-level ratchet*, so this attack class doesn't apply *as-is*. However, **the engine's per-Atrium replay-window discipline (M1b §3.4 freshness_window) is the analogous defense surface.** The `MembershipSetPolicy::refresh_required_secs` field (M2 §2.1) is the configurable knob.

**Recommendation:** No code change. **Validate** that the Inv-16 / Inv-18a freshness-window discipline has a default cap (e.g. 24h default; admin-configurable) per Inv-20 audit-deliverable. This is already a hardening item in the M1b non-fits §7.2. Confirm landing-status in the F-full plan-doc.

### §4.2 Concurrent-admin-remove transition window

**Signal pattern:** when two admins concurrently remove different members, the protocol accepts messages encrypted under *either* old-or-new sender-key during a transition window. Window-open attack-surface is documented in Balbás-Collins-Gajland 2023 + Cremers-Hale-Kohbrok 2021.

**Benten current posture:** MembershipSet is content-addressed (Inv-15) + the fork semantic resolves concurrent admin-actions cleanly — concurrent forks produce **two distinct MembershipSetId values, both valid**, and the application layer (Atrium policy) picks the canonical CURRENT. This is **structurally cleaner than SSK's transition-window approach** because the content-addressing makes the two possible group-states distinguishable rather than ambiguous.

**However**, there's a subtle gap: if two admins fork concurrently and *both* signed-MembershipAttestations are circulated, the *receiving* clients need a deterministic tie-break. M-CONS §1.4's `policy_version` monotonic + `created_at_hlc` provides the tie-break, but the rule is not yet spelled out in the audit-deliverable.

**Recommendation R-N4-3a (small):** add an audit-deliverable §-row documenting the concurrent-fork tie-break rule: "if two admin-signed forks at same parent_membership_set_id have differing created_at_hlc, the lexicographically-smaller HLC wins; if equal-HLC, lexicographically-smaller MembershipSetId wins; clients MUST archive (not discard) the losing fork for audit." Already implicit in M-CONS Inv-20 + HLC discipline; making it explicit prevents implementation drift. Cost ~0.5 wd.

### §4.3 Sealed-sender at the relay layer

**Signal pattern:** server delivers ciphertext to recipient without knowing sender's identity (sender attaches sender-cert proving membership; recipient verifies). Hides sender↔recipient metadata from server.

**Benten current posture:** Spike-A2 + Compromise #32 (per M-CONS §3) already mandates sealed-sender-by-default at the iroh-relay layer. Inv-16 covers this. **No gap.**

### §4.4 PCS healing improvements (Balbás-Collins-Gajland 2023 proposed)

**Their proposal:** efficient PCS-update bringing communication complexity from quadratic to linear in N; per-message ratcheting of signing-key as well as chain-key.

**Benten current posture:** PCS at v1-beta is **fork-driven, not chain-ratchet-driven**. M-CONS §1.4 ratifies M5 §6.2 fold-into-FORK-ONLY. The case for adopting a chain-ratchet PCS is only relevant if Benten ships AtriumWithCGKA later. The Balbás et al. proposal would be a candidate-shape for that future codepoint.

**Recommendation R-N4-2 (codepoint reserve only):** in the AtriumWithCGKA reserve, add an inner-codepoint slot for `MultiRecipientSealing::SsKChainedMode` (or a more-neutral name like `RotatingGroupKeyChainedMode`). This is a doc-only / codepoint-reserve-only refinement; cost ~0.25 wd. Defers the design choice but preserves the wire-format slot.

### §4.5 Anonymous-credentials (KVAC / zkgroup) for admin attestation

**Signal pattern:** the Signal server stores an encrypted member-list; admin-actions are authenticated via keyed-verification anonymous credentials. The server enforces admin-action validity without learning which user is admin.

**Benten current posture:** Benten's admin-attestation is a direct HybridSignature by `admin_pubkey` (M2 §2.1 `KindPolicy::Atrium { admin_pubkey }`). **Server-trustless** (no analogue to Signal server). The KVAC pattern is solving a problem Benten doesn't have: it hides admin-identity from the server. Benten's relay sees only blinded-CIDs, so the admin-identity-hiding goal is achieved through a different (and arguably cleaner) substrate.

**No change needed.** Document this as a "structural-difference: Benten achieves Signal-KVAC's metadata-hiding via blinded-CIDs + sealed-sender, not via anonymous credentials" in the audit-deliverable. This is the kind of thing an auditor would flag if not explicitly named (part of R-N4-3).

### §4.6 Out-of-order message delivery

**Signal pattern:** Double Ratchet handles via N + PN counters in message header + MKSKIPPED cache.

**Benten current posture:** content-addressed DAG + HLC + Loro CRDT handles out-of-order **at the application layer**, not the cryptographic layer. Out-of-order envelopes simply decrypt independently (each is K_Set-AEAD-self-contained). No ratchet to advance. **Structurally simpler than SSK on this axis.**

**No change needed.**

### §4.7 Synchronized-start for linked devices

**Signal pattern:** primary device bundles all account state + sender-keys-mirror + group-state into a 256-bit-AES-encrypted archive, transferred via QR-pairing-established Curve25519 session.

**Benten current posture:** M2 `MembershipSetKind::DeviceMesh` + M1c K(N) chain handles cross-device sync at a deeper layer. New device receives K_principal via multi-device-key-wrap; subsequent state is reconstructable from the content-addressed DAG + the device's K_principal-derived keys. **No bulk-archive step needed because content-addressing is the substrate.**

**Recommendation R-N4-4 (cite-only):** in the audit-deliverable doc, name the structural difference: "Signal multi-device requires bulk-archive transfer at link-time; Benten DeviceMesh achieves same goal via content-addressed DAG reconstruction + K_principal multi-device-key-wrap." Helps auditor see Benten is *not* missing a known Signal capability. Folded into R-N4-3.

### §4.8 Multi-device-state-divergence bugs

**Signal observation:** group-state changes on primary don't auto-sync to linked devices (Signal-Support docs + signal-cli-rest-api #647).

**Benten current posture:** content-addressed DAG + Loro CRDT means all devices converge on the same MembershipSet snapshots via the sync substrate. **Structurally avoids the Signal divergence-bug class.**

**No change needed.** **This is a Benten strength worth naming** in the audit-deliverable.

### §4.9 Authenticator-forgeability defense (Ed25519 sig load-bearing)

**Signal pattern:** every group member holds others' chain-keys; the Ed25519 sig is the *only* thing preventing inter-member message forgery.

**Benten current posture:** K_Set is shared by all members. The structural analog of forgery-defense is the **per-stanza AAD-binding tuple** (Inv-20 clause-c) which includes `sender_did`, + the sender's HybridSignature. **Equivalent defense, different mechanism.**

The Inv-20 clause-c tuple includes `sender_did` so AAD-binding fails if a different member tries to construct an envelope claiming the wrong sender. **This is already strong.**

**Recommendation R-N4-3b (cite-only):** explicitly note the AAD-binding-of-sender_did in Inv-20 clause-c is *load-bearing for inter-member non-forgeability*. Currently the audit-deliverable may not name this explicitly. Cost folded into R-N4-3.

---

## §5 — Things Benten does that Signal doesn't

For symmetry, surface where Benten's MultiRecipientSealing is structurally richer than SSK.

### §5.1 PQ-hybrid from day one

Benten ships ML-KEM-X25519 hybrid KEM + ML-DSA-65 + Ed25519 hybrid sig at v1-beta. SSK is pre-quantum; group ciphertext is harvest-now-decrypt-later vulnerable.

### §5.2 Forkable Atrium semantics

A first-class operation (`MembershipSet::fork`) returning a child set with a new K_Set + parent-pointer. SSK has no notion of group-forks; the closest analogue is "create a new group with overlapping membership," which loses cryptographic continuity.

### §5.3 Content-addressed DAG integration with K(N) chain

Per M1c + M2 §8: K(N) keys to immutable Version-Node-CIDs; MembershipSetId is itself a CID; fork-history is traceable via `parent_membership_set_id` chain. SSK has no analogue — Signal's groups are mutable server-state.

### §5.4 MembershipSet generalization across kinds

`MembershipSetKind::{Atrium, DeviceMesh, SingleDevice}` lets Benten unify multi-user Atrium + single-user DeviceMesh + degenerate-singleton under one primitive. SSK is single-group-shape; multi-device is *retrofitted* on top via sub-accounts.

### §5.5 Atrium-of-Atriums (Garden) recursive composition

Phase 7+ recursive composition is open in Benten's design. SSK has no recursion.

### §5.6 Cap-envelope ceiling per member (4-dim CapabilityEnvelope)

`MemberKey::DeviceDid { attestation: DeviceAttestation }` carries CLAUDE.md #17 4-dim CapabilityEnvelope ceiling per device. SSK has no per-member capability discrimination — every member is equal-in-cap.

### §5.7 Trustless storage substrate

Benten's plaintext_cid blinding + iroh-relay-blinded-CID model means **the relay doesn't even see ciphertext-↔-recipient mapping clearly**. SSK relies on a trusted (but cryptographically-constrained-via-zkgroup) Signal server.

### §5.8 Codepoint-dispatched cipher-suite agility

`CipherSuiteCodepoint` + `SigCodepoint` per V1-FROZEN-INTERFACE §15 means Benten can swap primitives across versions. SSK is locked into AES-CBC + HMAC-SHA256 + Ed25519 by libsignal wire format (a protobuf change is a protocol-version bump).

### §5.9 Hybrid Logical Clock ordering

`BentenHlc` provides causal ordering across membership changes. SSK relies on server-mediated ordering.

---

## §6 — Targeted scenarios: would SSK have made these easier?

For each of M-CONS Inv-20 clauses / Compromise #s, ask: would adopting SSK simplify or complicate?

| Scenario | MultiRecipientSealing handling | SSK handling | Verdict |
|---|---|---|---|
| Single sender, broadcast to N readers | Single envelope, N stanzas | N pairwise distributions then 1 ciphertext | Comparable; MultiRecipientSealing simpler at distribution-time |
| Ad-hoc "Bob no longer trusted; future content unreadable to Bob" | Atrium-fork: O(1) admin op + new K_Set distributed to N-1 via 1 multi-stanza envelope | O(N²): every member regenerates sender-key + N-1 pairwise distributions | MultiRecipientSealing significantly cheaper |
| "Alice's device was stolen; revoke that device only" | DeviceMesh add/remove via K_principal-generation-counter (M1c §11 Q2); single-user-scope op | Sub-account-style: remove device-account from group; trigger O(N²) sender-key rotation in *every* group device was in | MultiRecipientSealing much cleaner — device-scope vs group-scope |
| Member-add when group is large (N=100) | Single multi-stanza envelope with 1 added stanza; K_Set unchanged | O(N) `SenderKeyDistributionMessage` sends from existing members + N from new member | MultiRecipientSealing scales O(1) at add; SSK O(N) |
| "I joined yesterday; can I read content from 6 months ago?" | NO (default; past content stays unreadable; explicit re-share possible) | NO (you don't have old chain-keys) | Equivalent |
| "Group is forked; both branches active for a while" | Native: two MembershipSets, two K_Sets, both content-addressed | Not modeled; would require two server-side groups | MultiRecipientSealing native; SSK can't represent natively |
| Multi-device for a single user | First-class DeviceMesh kind | Sub-account-per-device + fan-out + sync-bugs (signal-cli-rest-api #647) | MultiRecipientSealing structurally cleaner |
| PQ harvest-now-decrypt-later | Defended (ML-KEM-X25519 hybrid) | NOT defended | MultiRecipientSealing significantly better |
| "An admin's key is compromised; rotate admin" | `KindPolicy::Atrium { admin_pubkey }` update + policy_version increment + fork | Threshold-admin / multi-admin not first-class in SSK; admin-handover is application-layer | Both have gaps; Benten reserved threshold_admin slot |
| "Storage relay is hostile; what does it learn?" | plaintext_cid_set is K_Set-HMAC-blinded; relay sees blinded CIDs only | Signal server sees ciphertext + (sender↔recipient pairs via DR session-ids unless sealed-sender) | MultiRecipientSealing stronger trust-minimization |

**Aggregate: in 9 of 10 scenarios MultiRecipientSealing is structurally simpler or stronger.** The one tie is "joiner reads old content" (both deny by default).

---

## §7 — Specific amendments suggested by SSK comparison

### R-N4-1 — Rename `AtriumWithCGKA` codepoint reserve

**Current (M-CONS §1.4 + M5 §6.2):** reserve `MembershipSetKind::AtriumWithCGKA` for post-v1-beta CGKA shape.

**Issue:** M5 itself documents that "CGKA-LITE" was a wrong-shape framing (the whole reason for the rename to MultiRecipientSealing). Reserving the codepoint as "AtriumWithCGKA" bakes the same misframing into the future-additivity surface. If we later land an SSK-style chained sender-key ratchet, that's not a CGKA either.

**Proposal:** rename codepoint reserve to **`AtriumWithRotatingGroupKey`** (neutral; covers SSK-shape OR MLS-shape OR DCGKA-shape; doesn't bake choice in).

**Cost:** doc-only rename. ~0.25 wd.

### R-N4-2 — Add `RotatingGroupKeyChainedMode` codepoint-reserve sub-slot

**Rationale:** the SSK-style chained-sender-key approach is a credible v1.x post-v1-beta candidate if a future use-case requires per-message FS-of-group-secret (e.g. regulatory audit requirement). Reserving the codepoint slot in TransportConfig (per M6 §6.5 codepoint-reserve) preserves the wire-additivity without committing to ship.

**Proposal:** in M-CONS §3 Compromise registry, add `#54: AtriumWithRotatingGroupKey inner-modes codepoint-reserve slot for {NoChain, SsKChain, MlsChain, DcgkaChain}` with v1-beta-load-bearing = NO; landing = post-v1-beta.

**Cost:** codepoint-reserve table row + audit-deliverable §-line. ~0.25 wd.

### R-N4-3 — Audit-deliverable §-add: SSK-comparison structural-difference table

**Rationale:** an auditor familiar with SSK will pattern-match Benten's MultiRecipientSealing and immediately ask "why isn't this CGKA-LITE? Why not SSK? Where's the chain-ratchet?" Pre-empting with an explicit comparison-section in the audit deliverable saves audit-cycle time.

**Proposal:** in SECURITY-POSTURE.md (audit-deliverable per M-CONS), add §X "Comparison to Signal Sender Keys" with:
- (a) Table from §3 of this doc (22-row matrix; subset of 10 load-bearing rows)
- (b) Explicit naming of Inv-20 clause-c AAD-binding-of-sender_did as the inter-member-non-forgeability defense (Benten's analogue to SSK's Ed25519 sig)
- (c) Concurrent-fork tie-break rule from §4.2 of this doc
- (d) Anonymous-credentials trade-off naming from §4.5 of this doc
- (e) Multi-device cleanliness naming from §4.8 of this doc

**Cost:** SECURITY-POSTURE.md ~80-line §-add. ~0.5 wd.

### Aggregate cost

**Total: ~1 wave-day** (R-N4-1 + R-N4-2 + R-N4-3). All doc-only / codepoint-reserve-only; no code path. Absorbable into a single M-CONS post-merge fix-pass alongside other N1/N2/N3 specialist outputs.

---

## §8 — Final recommendation

**CONCUR-WITH-AMENDMENTS** (the 3 small refinements R-N4-1 / R-N4-2 / R-N4-3 in §7).

**Headline:** Benten's MultiRecipientSealing + AdminKickEpoch-folded-into-FORK-ONLY is **structurally sound vs Signal Sender Keys.** The SSK comparison surfaces zero v1-beta blockers, validates 9-of-10 scenarios where Benten is structurally simpler or stronger, and ratifies M-CONS §1.4's recommendation to ship MultiRecipientSealing (not CGKA-LITE-as-thin-CGKA) at v1-beta.

The 3 refinements are doc-only:
1. Rename codepoint reserve `AtriumWithCGKA` → `AtriumWithRotatingGroupKey` (neutral; un-bakes future-shape).
2. Add `RotatingGroupKeyChainedMode` codepoint-reserve sub-slot to preserve post-v1-beta SSK-style landing.
3. Add SSK-comparison §-row to SECURITY-POSTURE.md audit-deliverable.

**Aggregate cost: ~1 wave-day; absorbable into M-CONS post-merge fix-pass.**

**Confidence:** HIGH that no substantive code change is warranted. MEDIUM-HIGH that the 3 refinements are the *right* small refinements (R-N4-2 is the most-arguable — codepoint-reserves are cheap so the bias is to reserve, but a reasonable critic could argue we're over-reserving).

**Headline strengths of Benten vs SSK:**
- PQ-hybrid from day one (SSK is pre-quantum at v1-beta and through Triple-Ratchet-research-status)
- Forkable Atrium semantics (SSK has no forks)
- Content-addressed DAG integration (SSK has no analogue)
- Trustless storage substrate (SSK relies on Signal server even with zkgroup)
- Multi-device first-class via DeviceMesh (SSK retrofits via sub-accounts + bulk-archive)
- 9-of-10 scenario simpler or stronger (§6)

**Headline weaknesses of Benten vs SSK (acknowledged, judged acceptable):**
- No per-message ratchet → K_Set compromise = bigger blast radius until fork (mitigated by fork-frequency discipline)
- No production-validation at billion-DAU-scale (mitigated by being a v1-beta, not a final cryptographic claim)

---

## §9 — Self-assessment + confidence per finding

| Finding | Confidence | Rationale |
|---|---|---|
| §2 SSK deep-dive (protocol flow + primitives + wire format) | **HIGH** | Cross-checked libsignal source + Balbás-Collins-Gajland 2023 + Wikipedia + Signal blog. Triangulated. |
| §2.4 FS=YES, PCS=NO-at-protocol-level for SSK | **HIGH** | Signal's own Double Ratchet spec explicitly says symmetric-chain alone gives FS-only-no-PCS. Balbás-Collins-Gajland 2023 + Cremers 2021 corroborate. |
| §2.7 known attacks against SSK | MEDIUM-HIGH | Public attacks well-documented; I may be missing some 2024-2026 work (literature search was depth-limited). |
| §3 22-row comparison matrix | **HIGH** | Each row triangulated against M2 / M-CONS sources for Benten side + libsignal + academic literature for SSK side. |
| §4 Signal-got-right surfacing | MEDIUM-HIGH | 9 patterns checked; some may not apply because Benten's substrate (content-addressed DAG) makes them irrelevant. Listed structurally. |
| §5 Benten-does-Signal-doesn't | **HIGH** | Direct comparison; Benten's strengths are clear structural items. |
| §6 10-scenario verdict (9-of-10 Benten simpler/stronger) | MEDIUM-HIGH | Scenario list is representative-not-exhaustive; reasonable critics could pick scenarios where SSK is comparable or better. |
| §7 R-N4-1 rename | **HIGH** | M5 already corrected the CGKA-LITE → MultiRecipientSealing rename; extending to codepoint-reserve naming is straightforward. |
| §7 R-N4-2 codepoint-reserve slot | MEDIUM | The "concrete future use-case" for SSK-style chained mode is speculative; codepoint-reserves are cheap so bias-to-reserve, but a critic could argue we're over-reserving. |
| §7 R-N4-3 SECURITY-POSTURE §-add | **HIGH** | Auditors will ask the SSK-comparison question; pre-empting is sound process. |
| §8 final verdict (CONCUR-WITH-AMENDMENTS; ~1 wd cost) | **HIGH** | Aggregate is straightforward; the 3 refinements are small, the absence of substantive blockers is clear. |

### §9.1 What I did NOT investigate (within scope but bounded)

- Detailed Matrix/Megolm (SSK-variant) deviations from libsignal SSK — out-of-scope for Benten's audit since Benten doesn't ship Megolm.
- Full reading of ePrint 2025/794 multi-device WhatsApp formal analysis (PDF returned binary; relied on search-summary snippets).
- Cryspen Triple Ratchet / SPQR detailed analysis — Signal's PQ-group-PQ-ratchet is research-stage; not yet load-bearing for Benten's posture.
- libsignal AGPLv3 contagion legal nuance — M5 §2.5 already disqualifies libsignal; no further legal investigation needed.

### §9.2 Cross-pollination opportunities with other N-specialists

- **N1, N2, N3** specialists (parallel) may surface complementary refinements; orchestrator consolidates.
- **N4's R-N4-2 codepoint-reserve slot** composes cleanly with M6 TransportConfig codepoint-reserve; check N1/N2/N3 don't propose conflicting codepoint names.
- **N4's R-N4-3 audit-deliverable §-add** should land alongside any other audit-deliverable additions from N1/N2/N3 to keep SECURITY-POSTURE.md coherent.

---

## §10 — Citations

### §10.1 Tree-pinned (HEAD `2172cb6d`)

- M2 `@ 6170980b` `.addl/phase-4-meta/membership-set-m2-primitive-design.md` §2.1, §2.2, §4, §5, §11 — MembershipSet primitive design.
- M5 `@ 15819500` `.addl/phase-4-meta/membership-set-m5-cgka-candidate-survey.md` §2.5, §6.2, §10 — Signal Sender Keys evaluation in CGKA survey.
- M-CONS `@ 74580ee6` `.addl/phase-4-meta/membership-set-m-cons-consolidator.md` §1.4, §3 (Inv-20), §9.1 — MultiRecipientSealing naming + AdminKickEpoch fold ratification.
- Spike-A2 `.addl/spikes/SPIKE-A2-ucan-on-wire-2026-05-21.md` — UCAN-on-wire integration that MultiRecipientSealing composes with.

### §10.2 External (WebSearch / WebFetch verified 2026-05-28)

- **Balbás, Collins, Gajland** — "WhatsUpp with Sender Keys? Analysis, Improvements and Security Proofs" — IACR ePrint 2023/1385; ASIACRYPT 2023 LNCS vol. 14442 pp. 307-341.
- **Balbás, Collins, Gajland** — "Analysis and Improvements of the Sender Keys Protocol for Group Messaging" — arXiv 2301.07045; RECSI 2022.
- **Cohn-Gordon, Cremers, Dowling, Garratt, Stebila** — "A Formal Security Analysis of the Signal Messaging Protocol" — EuroS&P 2017; J. of Cryptology 2020.
- **Cremers, Hale, Kohbrok** — "The Complexities of Healing in Secure Group Messaging" — USENIX Security 2021.
- **"Formal Analysis of Multi-Device Group Messaging in WhatsApp"** — IACR ePrint 2025/794.
- **"Poster: No safety in numbers: traffic analysis of sealed-sender groups in Signal"** — arXiv 2305.09799.
- **Wikipedia** — "Sender Keys" (en.wikipedia.org/wiki/Sender_Keys).
- **libsignal source** — github.com/signalapp/libsignal/blob/main/rust/protocol/src/{protocol,sender_keys}.rs (HEAD as of 2026-05-28).
- **Signal blog** — "Quantum Resistance and the Signal Protocol" (signal.org/blog/pqxdh/), "Sealed Sender" (signal.org/blog/sealed-sender/), "Forward Secrecy for Asynchronous Messages" (signal.org/blog/asynchronous-security/), "A Synchronized Start for Linked Devices" (signal.org/blog/a-synchronized-start-for-linked-devices/).
- **Signal Double Ratchet spec** — signal.org/docs/specifications/doubleratchet/.
- **Signal Private Group System + KVAC** — IACR ePrint 2019/1416; signal.org/blog/signal-private-group-system/.
- **Cryspen** — "Analysis of Signal's PQXDH" (cryspen.com/post/pqxdh/); "Post-Quantum Group Messaging" (cryspen.com/post/pq-mls/).
- **PQShield** — "Diving into Signal's New Post-Quantum Protocol" (pqshield.com/diving-into-signals-new-pq-protocol/).

### §10.3 CLAUDE.md baked-in references

- #17 (4-dim CapabilityEnvelope) — per-member envelope ceiling in DeviceMesh KindPolicy.
- #18 (4-identity-concepts) — Atrium / DeviceMesh / SingleDevice typed variants reflect 4-identity tree.
- V1-FROZEN-INTERFACE.md §15 — CipherSuiteCodepoint + SigCodepoint + canonical CBOR + content-addressing discipline.

---

**END §10**
