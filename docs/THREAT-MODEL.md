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
| **Network observer / untrusted relay** | an iroh relay, a passive network adversary, a storage host holding ciphertext | **NO** (but see the `body_cid` note below — low-entropy bodies are confirmable/linkable) | gossip topic (blinded), Drop envelope sizes + timing, blinded group tags — the K_Set-**KEYED** `membership_set_id_commitment` is opaque (never reveals the set-id), but the **UNKEYED** `audience_set_commitment` (`BLAKE3` over the sorted roster) hides only a **HIGH-entropy** roster: for a **guessable / low-entropy** roster (narrowed by the plaintext `member_count`) it is itself a confirmation-oracle + equality-linker (recompute `BLAKE3(sorted-roster)` to confirm a guess; identical rosters → identical tags → linkable), the audience-axis sibling of the `body_cid` residual (Row D-36; U25 per-send-salt reserve is the future linkage-half fix) — plus the plaintext **`body_cid`** (unsalted `BLAKE3(body)` CID; a confirmation-oracle + equality-linker for **low-entropy** bodies only) |
| **Untrusted host** (peers-hold-ciphertext) | a peer storing a principal's encrypted partition (Phase-7 Garden-Grove) | **NOT YET at v1-beta** — see the honesty note below (the confidentiality half of the Principal primitive is DEFERRED; per-Node AEAD is a publicly-derivable-`K_principal` STAND-IN, so a malicious host CAN read the partition plaintext); the LIVE protection is the AUTHORITY half (capability/namespace isolation) which binds only a COOPERATING engine | blob CIDs + access patterns |
| **Co-recipient member** | a member of a MembershipSet holding `K_Set` | **YES** for content they are entitled to | the member roster (recomputes the blinded commitments from the member list they hold) |
| **Admin** | a MembershipSet admin holding the audit log + `members_table` | **YES** for set content + **CAN correlate members** | full audit-log visibility (Compromise #58) |
| **Coerced device** | a device whose holder is compelled to unlock / approve | **YES** (whatever that device is entitled to) | n/a — OUT-OF-SCOPE (Compromise #33) |

**The load-bearing scope boundary.** Per-recipient unlinkability (the `0x6610` / `0x6520` group-AAD blinding) is
**network-observer-only** (see §3).

**⚠️ Untrusted-host confidentiality is NOT delivered at v1-beta (HARD honesty note — must match `docs/SECURITY-POSTURE.md`).**
Per CLAUDE.md baked-in #18 the Principal primitive has TWO isolation halves, and only ONE is live at v1-beta:

- **AUTHORITY isolation** (who is *allowed* to see/act) — capability/namespace-gating (UCAN / `CapabilityPolicy` +
  private-namespace delegation-refusal). This is **LIVE + real**, and it is the protection an untrusted host defeats:
  capabilities bind only a *cooperating* engine, so they give **zero** protection the moment a partition rests on
  hardware another principal controls.
- **CONFIDENTIALITY isolation** (who *can read the bytes*) — per-principal encryption of the storage partition. This is
  the **DEFERRED** #1301 / D-64 encryption substrate (the confidentiality half of the Principal primitive) and is
  **NOT built at v1-beta.**

Consequently, per-Node AEAD is a **publicly-derivable-`K_principal` STAND-IN** at v1-beta, **not** real untrusted-host
confidentiality: `K_principal = BLAKE3-keyed-hash(domain_tag, namespace_did)` where BOTH the `domain_tag` (the public
function-local constant `K_PRINCIPAL_DOMAIN_KEY` in `benten-graph/src/redb_backend.rs` — NOT one of the 22
`registered_domain_tags()`, consistent with Compromise #65) AND the `namespace_did` are **PUBLIC**, so `K_principal` — and thus `K(N)`
and the per-Node AEAD key — is **publicly derivable**: any party holding `(namespace_did, ciphertext_blob)` can derive
the key and decrypt. An **untrusted host CAN therefore currently read the partition plaintext.** This is disclosed
honestly + tracked as a **numbered Compromise** in `docs/SECURITY-POSTURE.md` (Compromise #65 — "wave-3e per-Node AEAD
publicly-derivable-`K_principal` confidentiality limit at v1-beta"), cross-linked to the existing "⚠️ Confidentiality
limit at this wave" disclosure in that document's per-Node-AEAD section. The **Layer-A vault** (Argon2id-derived DAK
sealing the on-disk `K_principal`/user-DID-key vault) is real at-rest protection **for the local device's own vault**,
but it is distinct from — and does NOT stand in for — the per-DID partition-confidentiality half a hostile *remote*
host would defeat.

(Contrast Tier-1: the **network-observer / relay** "sees plaintext = NO" is real + live — that is **Layer-C
encrypt-to-recipient** (`0x6610` / HPKE-wrapped CEK), which genuinely seals the wire against a passive relay. The
relay-vs-untrusted-host distinction is the whole point: sealing bytes *in transit to a chosen recipient* is live;
sealing a per-principal partition *at rest on a hostile host* is the deferred confidentiality half.)

**Deterministic-CEK confirmation-oracle (additive disclosure; GAP-2).** On the **single-recipient** Layer-C bands
(`0x6500` / `0x6510`, `benten_drop::layer_c::seal_inner`) ONLY, the content-encryption key is
deterministically derived from the plaintext, so a party that **already holds the CEK** (the sealer, or a
co-recipient that recovers it) can **confirm a guessed plaintext** — a confirmation oracle for low-entropy bodies.
The **`0x6520`** group CEK is a **fresh-random per-message value** (OS
CSPRNG, delivered only via each stanza's HPKE-wrap; no wire-derived CEK, so no CEK confirmation-oracle at all), so
the deterministic-CEK oracle above does NOT apply to it. The **`0x6610`** MembershipSet group CEK is `K_Set`-keyed
and therefore **body-deterministic** — a member already holding `K_Set` CAN confirm a guessed body for a `0x6610`
send — but that capability is **subsumed by the keyless `body_cid` oracle** below (which needs no `K_Set` at all),
so `0x6610` discloses nothing beyond `body_cid` (do NOT lump it with the fresh-random `0x6520`). The three
CEK constructions are enumerated in full at `docs/SECURITY-PROOFS.md` §4.1 (`0x6610` K_Set-keyed / `0x6520`
fresh-random) + §4.2 (`0x6510` deterministic single-recipient). Separately, the `body_cid` low-entropy disclosure
below is a property of the wire `body_cid` (not the CEK) and DOES apply across all bands.
The **ciphertext** does not extend the CEK confirmation-oracle to the **Tier-1 network observer / untrusted relay**: the bulk AEAD uses a
**fresh random nonce per send** and the CEK is HPKE-wrapped to the recipient, so the relay sees neither the CEK nor
a plaintext-equality test **in the ciphertext bytes**. **However**, the wire `body_cid` — an **unsalted**
`self_describing_cid(BLAKE3(plaintext))` emitted **plaintext** in every Layer-C AAD — DOES give the Tier-1 observer,
**for low-entropy / guessable bodies only**, (a) a confirmation oracle (guess → `BLAKE3` → compare against the wire
`body_cid`, no key material needed) and (b) a plaintext-equality linker (identical bodies carry identical
`body_cid`). The Tier-1 "sees plaintext = NO" row is unchanged (the relay still recovers no plaintext for
high-entropy bodies and never the ciphertext), but low-entropy senders must **pad/randomize at the application
layer**; a per-send `body_cid` salt is additively reservable. Full cryptographic narration:
`docs/SECURITY-PROOFS.md` §4.2; residual tracked at Compromise #43.

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

## §2b — Recipient-key SUBSTITUTION closure (GAP-KDB Shape-B / Inv-23) + the two DISTINCT open residuals

The §2 rung-4 blast-radius is a *key-COMPROMISE* axis. A DISTINCT, previously-open seam was recipient-key
*SUBSTITUTION*: nothing bound the `RecipientPublic` fed to a seal to the `audience_did` it was sealed under, so
an active attacker at the address-book boundary could hand a sender the WRONG recipient's KEM key and read
everything (the "silently trusts an honest address book" premise every §2 proof rested on).

**Recipient-key substitution — CLOSED (GAP-KDB Shape-B).** The audience `did:benten` now *commits* the recipient's
key-set (including the KEM key) by CID, and `Did::resolve_kem` recovers-and-verifies the KEM key from the DID —
a substituted recipient KEM key not committed by the audience DID fails closed and cannot be sealed to (Inv-23,
a BLAKE3-256 2nd-preimage, enforced behind the `RecipientBinding` sole-constructor typestate). The recipient side
is now symmetric to the already-self-certifying sender side. Cross-link Inv-23 (`INVARIANT-COVERAGE.md`) +
`SECURITY-PROOFS.md` §4.1 (the recipient-key premise now cites the binding, not an honest address book).

**Residual A — seal-path revocation-reach (DISTINCT from Compromise #67).** Shape-B does NOT close a separate
seal-path gap: rotating away from a compromised key does NOT stop *inbound* Drops — any sender still holding the
recipient's old key-set doc can keep sealing inbound Drops to the old key-set's KEM key (the old committed doc
stays a resolvable, self-consistent commitment for any sender who cached it). This is an OPEN
seal-path revocation-reach residual, distinct from Compromise #67 (first-contact / TOFU). Cross-link Compromise
#62 (revocation-reach; Drops forever-valid once distributed). It is disclosed here so it is never conflated with
the first-contact residual.

**Residual B — offline-first-send availability (DISTINCT from #67 AND from Residual A).** Sending a *first* Drop
to an offline / uncached / brand-new contact still needs their key-set doc (carried in-band, cached in the vault
keyed by DID, or fetched by CID over iroh-blobs); offline + uncached + never-contacted = cannot send. This is
strictly better than the pre-Shape-B state (you would otherwise hold an *unverified* RecipientPublic) and is the
same first-contact ergonomic every system has, but it is a genuine availability cost — the operational face of
Compromise #67, named separately here (distinct from both #67 and Residual A) so the three residuals are never
silently conflated.

---

## §3 — Per-recipient unlinkability scope = NETWORK-OBSERVER-ONLY (load-bearing)

The Layer-C group-AAD blinding (`0x6610` MembershipSet group + `0x6520` Layer-C multi-recipient) replaces the raw
recipient roster + raw set-id with blinded commitments (`audience_set_commitment` = BLAKE3 over the canonical
sorted DID list; `membership_set_id_commitment` = `blake3::keyed_hash(K_Set, "benten:setid:v1" ‖ id)`). This
achieves **per-recipient unlinkability** against a **network observer / untrusted relay** — the relay sees only
opaque 32-byte tags. The K_Set-**keyed** `membership_set_id_commitment` never reveals the set-id; the **unkeyed**
`audience_set_commitment` hides a **high-entropy** roster but, for a **guessable / low-entropy** roster, is
guess-confirmable (see the honest-scope note below + Row D-36).

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
recurs for a static recipient set, so a network observer can still link sends to "the same group". And because the
`audience_set_commitment` is **UNKEYED** (`BLAKE3` over the sorted roster; contrast the K_Set-keyed
`membership_set_id_commitment`), for a **guessable / low-entropy** roster (the guess space narrowed by the
plaintext `member_count`) that group is not merely "unknown": a network observer with a candidate-DID pool can
**CONFIRM** a guessed roster by recomputing `BLAKE3(sorted-roster)` and comparing — the audience-axis sibling of the
`body_cid` confirmation oracle (§1 Tier-1 note; SECURITY-PROOFS §4.2). This is a metadata-privacy caveat for
low-entropy rosters only — the AEAD confidentiality + the binding/non-forgeability properties are unaffected, and a
high-entropy roster keeps the guess space intractable. Full per-send unlinkability (salt/nonce-rotated commitments)
is **U25, CODEPOINT-RESERVE for v1-GM** (the future linkage-half fix); a keyed `audience_set_commitment` (Row D-36)
is the additive guess-confirmation-half fix — both additive over the field with no wire-break. Separately, the plaintext `body_cid` links sends that share the **same body** (a
low-entropy confirmation/equality vector — see §1 Tier-1 note + `docs/SECURITY-PROOFS.md` §4.2; a distinct axis
from recipient-linkage). Cross-link Compromise #43 (envelope-metadata leakage + `body_cid` residual); Compromise
#61 (gossip-topic blinding).

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
and the ExecuteWorkflow seal/open path is **unwired at v1-beta** — `exec_workflow_seal` / `exec_workflow_open`
(`crates/benten-engine/src/layer_d/remote_permission.rs`) have **zero production callers** (the
`0x6320..=0x632F` band-dispatch `dispatch_remote_permission_codepoint` accepts the wire shape for forward-compat,
but nothing routes an `ExecuteWorkflow` to an actual executor), so there is no executable exfiltration path
through an unbound `input_node_cids` today. **Input-READ-scope AAD/signature binding is
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
`crates/benten-crypto-suite/src/domain_registry.rs` (a 22-tag corpus via `registered_domain_tags()` plus the
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
the bounded-decode ALLOCATION-CEILING on the wire-decode `count` path (the chunked-storage-envelope
chunk-count over-run reject — `decode_encrypted_node`'s `0x01` variant caps the
attacker-declared `count` at `(bytes.len() - 42) / 9` before pre-allocating, since each chunk entry needs
≥ a 4-byte length prefix + a 5-byte minimal `AeadEnvelope` header, so a crafted 42-byte blob with
`count = u32::MAX` typed-rejects via `AeadError::ChunkCountExceedsInput` instead of forcing a multi-GB
`Vec::with_capacity`; this path is reachable PRE-AUTH via `RedbBackend::get_encrypted_node` on the
untrusted-host tier; R12 F-01). The group multi-stanza `stanza_count`, by contrast, is **NOT** an
allocation-ceiling: it is a TRUNCATION-INTEGRITY defense — `stanza_count` is bound into every stanza's AAD
(`crates/benten-drop/src/layer_c.rs`), so a relay that drops or reorders a stanza makes the recipient's
AAD-open FAIL (`delivered != stanza_count`), detecting the truncation. `stanza_count` is COMPUTED author-side
from `recipient_pubs.len()`, never wire-decoded into an allocation-sizing count, so there is no
`stanza_count` over-run to reject. Rounding out the availability-adjacent substrate defenses: the
per-Kind `wire_cost_ceiling` (Compromise #46), and the 4-MiB `recv_bytes` sync cap. The MembershipSet member
count is a strictly stronger case: it is **NEVER wire-decoded** — the `member_count` is always COMPUTED from the
recipient's own held roster (`member_dids.len()`) and bound INTO the AAD, never read from an untrusted wire field
into an allocation-sizing count, so there is no member-count over-run to reject (the `#46 wire_cost_ceiling` bounds
the AUTHOR-side roster size, not a decoder). A dedicated
availability/DoS threat model (rate-limiting, connection-flood, storage-amplification, compute-exhaustion at the
engine + transport layers) is **NAMED-DEFERRED to a later phase** — it is not a v1-beta-core crypto-freeze
concern. This note exists so a reader does NOT mistake the absence of a DoS section for a claim that DoS is
out-of-scope for the *project*; it is out-of-scope for *this crypto threat model* specifically.

**Allocation-ceiling exhaustiveness audit (scheduled — G-COMP-1).** The individual allocation-ceiling defenses
disclosed above (the `decode_encrypted_node` chunk-`count` cap et al.) are pinned at their own sites, but a
systematic *exhaustiveness* audit — sweeping EVERY wire-decoded `count`/length path to confirm each caps the
attacker-declared value before pre-allocating — is scheduled at the **G-COMP-1** wave, alongside that wave's
hex-pin sweep. Until it runs, the disclosed ceilings are the known-covered set, not a proven-complete one.

---

## Cross-references

- `docs/SECURITY-POSTURE.md` — Compromise #33 (coercion OOS), #34 (password-knowledge), #35 (device retro-decrypt),
  #42 (HPKE FS-gap), #43 (envelope-metadata), #48 (set shape-leak), #52 (fork-on-kick), #58 (insider-correlation),
  #61 (gossip-topic blinding), #62 (revocation-reach), #64 (cross-device nonce window).
- `docs/SECURITY-PROOFS.md` — the per-stanza AAD binding (the cryptographic basis for the §3 unlinkability scope).
- `docs/CRYPTO-CODEPOINTS.md` — the codepoint allocation + the §3.9 gossip-topic vs §3.10 AAD-commitment distinction.
