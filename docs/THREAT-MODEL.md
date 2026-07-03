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

**Inter-member sender-origin non-forgeability (positive as-built property; B2 ORIGIN-AUTHENTICATION — ENFORCED in
code).** A **co-recipient member** (and, on the multi-recipient group path, a co-sealer) — even one holding `K_Set`
and thus able to derive the CEK and produce valid AEAD tags — **CANNOT forge the sender origin** on the online
Layer-C path: it can neither mint a send attributed to another member nor re-target another member's real body to a
recipient set that member never chose. The "Co-recipient member" / "Admin" tiers above see content they are
entitled to but get **no forgery power over attribution**. Each Sealed-Sender send carries a per-MESSAGE
LAMPS-hybrid (`id-MLDSA65-Ed25519-SHA512`) sender signature inside the once-sealed body, verified post-decrypt
against the recipient-INDEPENDENTLY-held set-state, **fail-closed** (`SenderOriginAuthFailed`); forging requires the
target's hybrid (post-quantum) signing key. Full cryptographic narration: `docs/SECURITY-PROOFS.md` §4.1
(inter-member non-forgeability decomposition + the substantive `f_lc_3_*` spoof/re-target/stale-generation/strip-PQ
defense arms).

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
   recipient** (HPKE-mode-base is structurally non-FS at the long-term-sk axis). This rung is reached **only** by
   compromising the recipient's REAL hybrid SECRET (the ML-KEM-768 decapsulation key ‖ X25519 static secret carried
   by `benten_crypto_suite::cipher_suite::RecipientSecret`); an attacker holding **only the recipient PUBLIC key**
   (`RecipientPublic`) **cannot decrypt** — the public key is unrecoverable-to-secret and a non-matching secret
   fails the CEK-unwrap closed (R9 GAP-1: the prior `[u8; 32]` public-fingerprint placeholder, which let any
   public-key holder reconstruct the secret via `sk = pk + 0x80`, is DELETED). Cross-link Compromise #42 / #48.
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

**NQ-T3-adjacent — `input_node_cids` is bound into no signed/AEAD surface (NAMED-DEFERRED → v1-GM).** The
`ExecuteWorkflow` struct (`crates/benten-engine/src/layer_d/remote_permission.rs::ExecuteWorkflow`) carries an
`input_node_cids: Vec<[u8; 32]>` field (the Node CIDs the rented workflow reads), but `constraint_aad()` binds
**only** the frozen 3-field tuple `(executor_did, max_decrypt_count, result_recipient_pubkey)` — `input_node_cids`
(and `workflow_cid`) are **NOT** bound into the AAD or any signature. Consequence: the executor's **input
READ-scope is not cryptographically bound** at v1-beta. **Disposition (no live exploit at v1-beta):** runtime
ExecuteWorkflow enforcement is post-v1-beta to begin with (NQ-T3 above); the field is carried for forward-compat
and the variant is reserved / typed-rejected at the dispatch boundary at v1-beta, so there is no executable
exfiltration path through an unbound `input_node_cids` today. **Input-READ-scope AAD/signature binding is
NAMED-DEFERRED to v1-GM** (alongside the NQ-T3 runtime no-egress enforcement it travels with); the deferral is
recorded in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`. Freezing it now would be premature because the read-scope
binding shape co-designs with the post-v1-beta runtime enforcement.

### NQ-T4 — nonce-cache spec + the cross-device replay window (RATIFIED → Compromise #64)

**Rule:** the nonce-cache is **`jti`-keyed** and retention is **≥ the full 1-hour bucket window**; the replay
rejection is keyed on the `jti` nonce, NOT any time field (a clock rewrite cannot evade it). **Durability is
provided via a SEAM, not intrinsic disk-persistence at v1-beta-core.** The `JtiNonceCache`
(`crates/benten-sync/src/handshake.rs`) is a **durable-CAS-marker + hydration seam**: it holds a
`durable_store` consumed-`jti` set (in-RAM at the type level) plus a `from_durable(...)` **hydration seam** +
`durable_snapshot()` — a restart OR a cross-device sync feeds the consumed-`jti` set back through
`from_durable`. **Per-device durability is delivered through this seam** (the engine is responsible for
persisting `durable_snapshot()` to disk and re-hydrating via `from_durable` on restart — a **caller
contract**); the accept-grant path (`benten_engine::layer_d::grant_acceptance::accept_grant`, currently
zero-production-caller) likewise takes a caller-supplied `nonce_cache: &mut HashSet<[u8;32]>`. The FULL
disk-persistence wiring (and the accept_grant nonce_cache backing) is **DEFERRED with the remote-permission /
engine-encrypt-to-recipient wiring** (`docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-64-adjacent). **Scope:**
**per-device-durable is GUARANTEED via the seam** (once the caller persists + re-hydrates, a nonce consumed on
a device cannot be replayed against that same device); **user-global is best-effort-eventual-via-sync (NOT
synchronous)** — a nonce consumed on device B is rejected on device C only after sync propagates the
consumed-`jti` set (via the same `from_durable` hydration seam). The pre-sync cross-device replay window is
DISCLOSED as a named Compromise: **Compromise #64** ("best-effort-eventual cross-device nonce-rejection
window") in `docs/SECURITY-POSTURE.md`. (Distinct mechanism: the capability-CHAIN-frame replay marker
`benten_caps::FrameReplayMarker<B: GraphBackend>` IS genuinely graph-backed-durable, but it defends inbound
sync FRAMES, not the Layer-D `jti` grant nonce.)

**Nonce-cache is orthogonal to the metadata bucket (NQ-C5).** The epoch bucket
(`(raw_unix_secs / 3600) * 3600` — round-DOWN, NO jitter, deterministic) is for metadata privacy; the nonce-cache
is for replay defense. They are tuned independently — the nonce-cache window does NOT need widening to track the
bucket. The nonce-cache spec narration (the §3.10 reference) lives here in NQ-T4; the §3.9 unlabelled gossip-topic
derivation (`blake3::keyed_hash(K_Set, set_id ‖ BE(generation))`) is the SEPARATE construction documented in
`docs/CRYPTO-CODEPOINTS.md` (§3.9 vs §3.10).

---

## §5 — Cross-surface domain-separation (the prefix-free domain-tag registry)

**Threat (T-DOMSEP).** The substrate derives keys, signs binding messages, and commits AAD across **many
independent cryptographic surfaces** — the Layer-C single/group CEK derivations
(`"benten-drop:layer-c:cek"` / `"...:group-cek"` / `"benten-drop:membership-group-cek"`), the chunked-AEAD
info strings (`"benten-aead:whole:"` / `":chunk:"` / `":recipe:"`), the sender-auth and envelope-signature
binding domains (`SENDER_AUTH_DOMAIN` / `ENVELOPE_SIG_DOMAIN`), the remote-grant / remote-request / exec-workflow
AAD domains (`GRANT_DOMAIN` / `REQUEST_DOMAIN` / `EXEC_WORKFLOW_AAD_DOMAIN`), the MembershipSet set-id and
gossip-topic derivations (`"benten:setid:v1"`, the §3.9 `blake3::keyed_hash(K_Set, set_id ‖ BE(generation))`
topic), and the `K(V)` keying-glue context. If any two surface tags are **not prefix-free** — i.e. one tag is a
byte-prefix of another, or two distinct surfaces share a tag — an attacker (or an honest implementation bug)
could cause bytes authenticated/keyed for surface A to be accepted on surface B (a cross-surface
key-reuse / binding-confusion attack). Length-extension-style framing, sloppy `info`-string concatenation, or a
future tag minted as `"<existing-tag>-suffix"` are the concrete failure modes.

**Defense / v1-beta structural shape (T-DOMSEP-MIT).** All cross-surface domain tags are drawn from a **single
prefix-free domain-tag registry** (the canonical source of separators), and the registry enforces — by
construction + a workspace regression test — that **no registered tag is a byte-prefix of any other registered
tag** (mutual prefix-freedom). This makes cross-surface confusion **structurally impossible** rather than
audited-by-inspection: minting a colliding/prefixing tag fails the build. The prefix-free property is the
permanent v1-beta commitment; the registry *contents* are additive (new surfaces register new tags, which must
clear the same prefix-free check). See `docs/SECURITY-PROOFS.md` §4.1 "Cross-surface domain-tag registry
(prefix-free)" for the property statement and the §4.1 "Inner-format domain-separation (single vs group)" note for
the single-vs-group instance this generalizes. **Status:** the registry + its prefix-free regression test SHIPPED
(F-full / R6-R4 structural shape) — the centralizing registry CODE is at
`crates/benten-crypto-suite/src/domain_registry.rs` (a 19-tag corpus via `registered_domain_tags()` plus the
`all_domain_tags_are_prefix_free` regression); this row + the SECURITY-PROOFS property record the v1-beta
commitment the code realizes.

---

## §6 — Availability / DoS scope (honest disclosure — NOT a section of this crypto threat model)

**This document is a CONFIDENTIALITY / INTEGRITY / AUTHENTICITY threat model.** It deliberately has **no
availability / denial-of-service section** (R9-council F-19 names this gap honestly rather than papering over it):
the cryptographic substrate's threat surface is about who-can-read / who-can-forge / who-can-replay, not
who-can-degrade-service. Availability against a resource-exhaustion adversary is a **cross-cutting engineering
concern handled outside the crypto threat model**, and where a specific amplification vector touches the crypto
substrate it is disclosed at its own site rather than here — e.g. the `dedup_synchronized_revocations`
substrate-growth defense (`crates/benten-sync/src/handshake.rs`, adversarial duplicate-packing + rejoin-churn),
the bounded-decode ceilings on every wire-decode path (`membership_count` / stanza-count over-run rejects), the
per-Kind `wire_cost_ceiling` (Compromise #46), and the 4-MiB `recv_bytes` sync cap. A dedicated
availability/DoS threat model (rate-limiting, connection-flood, storage-amplification, compute-exhaustion at the
engine + transport layers) is **NAMED-DEFERRED to a later phase** — it is not a v1-beta-core crypto-freeze
concern. This note exists so a reader does NOT mistake the absence of a DoS section for a claim that DoS is
out-of-scope for the *project*; it is out-of-scope for *this crypto threat model* specifically.

---

## Cross-references

- `docs/SECURITY-POSTURE.md` — Compromise #33 (coercion OOS), #34 (password-knowledge), #35 (device retro-decrypt),
  #42 (HPKE FS-gap), #43 (envelope-metadata), #48 (set shape-leak), #52 (fork-on-kick), #58 (insider-correlation),
  #61 (gossip-topic blinding), #62 (revocation-reach), #64 (cross-device nonce window).
- `docs/SECURITY-PROOFS.md` — the per-stanza AAD binding (the cryptographic basis for the §3 unlinkability scope).
- `docs/CRYPTO-CODEPOINTS.md` — the codepoint allocation + the §3.9 gossip-topic vs §3.10 AAD-commitment distinction.
