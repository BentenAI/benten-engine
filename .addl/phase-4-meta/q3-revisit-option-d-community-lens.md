# Q3 revisit — Option D (K_Atrium-blinded HMAC `plaintext_cid`) — senior cryptographer + P2P-systems architect lens

**Reviewer lens:** Combined SENIOR CRYPTOGRAPHER + P2P-SYSTEMS ARCHITECT — the question lives at the intersection of (a) keyed-PRF content-fingerprint primitives (HMAC / OPRF / convergent-encryption literature; DupLESS), (b) Atrium peer-mesh forkability semantics + iroh-blobs/iroh-gossip transport composition, (c) the deferred CGKA decision.

**Reviewer task:** assess Option D as PRIMARY content-equality-oracle defense; explore iroh-gossip's per-Atrium dedup angle; re-evaluate CGKA-deferral; surface elegant alternatives; recommend a winner (D | C | D' | hybrid).

**Branch:** `phase-4-meta-core/q3-revisit-option-d-community-lens`
**Worktree:** `${WORKTREE_ROOT}`
**Tree-state pre-flight:** HEAD `2172cb6d` matches `origin/main`; tree clean; branched.
**Prior context loaded in full:** L9-atrium-integration (717 lines), 9-eyes-consolidated-registry (939 lines), C1-elegant-shape (724 lines), C2-composability (767 lines), C3-fresh-eyes-cryptographer (697 lines), C4-process-discipline (618 lines), C5-formal-methods (637 lines), docs/SECURITY-POSTURE.md, docs/INVARIANT-COVERAGE.md.
**Literature anchors:** Bellare-Keelveedhi-Ristenpart 2013 (DupLESS / message-locked encryption / brute-force attacks against convergent encryption); Wikipedia "Convergent encryption" (content-equality-oracle vulnerability formalization); Alwen-Coretti-Jost-Mularczyk 2020 (CGKA with active security); Alwen-Hartmann-Kiltz-Mularczyk 2023 (Fork-Resilient CGKA); Leitão-Pereira-Rodrigues HyParView + PlumTree (iroh-gossip's protocol substrate); Kissner-Song 2005 (PSI primitives); RFC 9180 (HPKE).

---

## §1 Executive recommendation + confidence

### Headline

**Adopt Option D as PRIMARY** with **two qualifications** that turn it from "Ben's preference assessed" into a structurally-sound shape:

1. **K_Atrium MUST be derived from a `key_id` (Atrium-creation nonce + scoped HKDF info-string), NOT from member-DID-set or from K_principal.** This ensures K_Atrium is **stable across membership changes** (matches forkability semantics; no CGKA-rotation-on-leave), but **distinct per Atrium** (so cross-Atrium dedup is broken — which is the actually-desired property; see §2.2).

2. **`plaintext_cid` MUST be split into TWO surfaces** at the U18 dual-CID layer:
   - **`plaintext_cid_local`** = `BLAKE3(canonical(DropBundlePayload))` — the *un-blinded* content-CID; **NEVER on the wire**; computed at decrypt-time by each recipient; what the user's graph references locally; what survives forks.
   - **`plaintext_cid_atrium`** = `HMAC-SHA256(K_Atrium, plaintext_cid_local)` (16-byte truncation OK; 128-bit collision is sufficient at the dedup-fingerprint surface) — the *blinded* content-fingerprint; visible to Atrium-internal peers only; **storage host never sees it without K_Atrium**.

   This is a refinement of U18 — currently U18 has ONE plaintext_cid and ONE envelope_blob_cid; the equality-oracle threat closure requires SPLITTING U18's plaintext_cid into a local-axis (graph-stable, never-on-wire) + an atrium-axis (blinded, peer-visible-only).

**Confidence on Option D as PRIMARY (with the two qualifications above):** **HIGH (~85%).** Option D defeats the storage-host equality-oracle while preserving Atrium-internal dedup, forkability, and offline-first semantics. The crypto is well-trodden (HMAC-PRF on a fixed-size BLAKE3 input is the strongest possible regime for HMAC). The two qualifications are mechanical refinements, not redesigns.

**Confidence on fallback to Option C if a future constraint surfaces:** **HIGH** — Option C is strictly within the design-envelope of Option D (just set `plaintext_cid_atrium = nil` and rely on per-recipient envelope_blob_cid dedup). Migration C → D is additive (mint a new codepoint reserve for atrium-blinded variant); no breaking change.

### Three load-bearing conclusions

1. **iroh-gossip becomes valuable under Option D in a way it ISN'T under Option C** (§3). Under C, gossip can't dedup envelopes at protocol level (each is per-recipient-distinct); gossip-spread costs scale linearly with `|members|`. Under D with `plaintext_cid_atrium` visible, gossip's PlumTree IHAVE/IWANT can dedup at the topic-substrate. **Recommendation: mint a CODEPOINT-RESERVE slot at v1-beta for `LAYER_C_ATRIUM_GOSSIP` transport-binding** (per the CLAUDE.md #5 codepoint-discipline); ship iroh-blobs as the v1-beta default per ratified S&C; iroh-gossip light-weight pilot at G-CORE-ATRIUM-SCALE-1 (post-v1-beta).

2. **The CGKA-deferral STILL HOLDS under Option D**, because Option D's K_Atrium does NOT require rotation on member-leave (the forkability semantic is exactly "past content stays with whoever already has it; future content excludes them via recipient-set exclusion"). What Option D needs is **K_Atrium DISTRIBUTION on member-JOIN**, which is **multi-stanza HPKE under U17** (already ratified) targeted at the joining member's pubkey. This is **not CGKA**; it's one-shot key-wrap. The deferred CGKA value-add (PCS / FS-across-membership-changes) remains structurally mismatched with Benten's forkability.

3. **The elegant shape Ben asked about emerges from recognizing K_Atrium isn't really an "Atrium key" — it's a `K_DedupScope` parameter** (§5 Option I). The dedup-scope can be Atrium-default but ALSO Drop-bundle-creator-chosen: per-Atrium (default; dedup across all Atrium members), per-creator (dedup only for content from one author; sender-anonymity), per-recipient (= Option C; degenerate single-member scope), or per-content-class (rotation per class). **Treat K_Atrium as one instance of a general `K_DedupScope: HMAC-PRF-key` parameter in the EncryptedEnvelope codepoint family.** This composes with the existing CLAUDE.md #5 crypto-agility doctrine and adds zero structural complexity.

---

## §2 Option D rigorous assessment (per Task 1)

### §2.1 Crypto soundness

**Threat model recap.** Untrusted 3rd-party storage host has `(plaintext_cid_blinded, envelope_blob_cid, envelope_ciphertext)` triples on disk. Host wishes to test "is `candidate_plaintext` in storage?" by computing some function of `candidate_plaintext` and comparing.

**Option D primitive:** `plaintext_cid_blinded = HMAC-SHA256(K_Atrium, BLAKE3(canonical(plaintext)))`.

**Properties (vs the stated threat):**

| Property | Assessment | Reasoning |
|---|---|---|
| **Pre-image resistance** (given `plaintext_cid_blinded`, find `plaintext`) | **STRONG** | HMAC-SHA256 is a PRF assuming SHA-256 compression is a PRF; finding pre-image requires either breaking SHA-256 pre-image (~2^256) OR brute-forcing the plaintext space — but the BLAKE3 inner hash means plaintext-space brute-force still gives the adversary nothing without K_Atrium. |
| **2nd-pre-image resistance** | **STRONG** | Same regime; the adversary cannot construct a 2nd plaintext that hashes to the same blinded CID without K_Atrium. |
| **Equality-oracle resistance** (the *actual* threat) | **STRONG** | Host computes `BLAKE3(canonical(candidate))` cheaply, but cannot compute `HMAC-SHA256(K_Atrium, ·)` without K_Atrium. Probe fails. |
| **Key-recovery resistance under observing many (plaintext_cid_blinded, plaintext_cid_blinded') pairs from same Atrium** | **STRONG** | HMAC-SHA256 has provable security as a PRF under the NMAC reduction (Bellare 2006); key-recovery from `q` query/output pairs requires distinguishing PRF from random function — best-known attack remains generic 2^256/q work-factor. **Adversary observing all envelope traffic for the Atrium's lifetime sees ≤ 10^9 blinded-CIDs (10 years × 10^8 drops); 2^30 queries vs 2^256 key-space leaves >2^200 distinguishing margin.** Comfortable headroom. |
| **Multi-target attacks (DupLESS-style adversary observes blinded-CIDs across MULTIPLE Atriums, each with distinct K_Atrium)** | **STRONG** | Cross-Atrium K_Atrium independence (per §2.2) means multi-target collision requires breaking N independent HMAC keys. Standard multi-target reductions apply. |
| **Length-extension attacks (the SHA-256-without-HMAC class)** | **N/A** | HMAC structurally defeats length-extension by design. Use the actual HMAC construction; NEVER substitute raw `SHA256(K \|\| plaintext)`. |
| **Truncation of HMAC output to 16 bytes** | **ACCEPTABLE at 128-bit collision target** | At ~10^9 blinded-CIDs, birthday bound is `2^(128/2) = 2^64` ≫ 2^30. 16-byte truncation is the same regime as Wireguard's MAC + Signal's PreKey identifier. RECOMMEND: 16-byte truncation acceptable; 32-byte full HMAC also fine if storage cost not load-bearing. |

**Note on HMAC-SHA256 vs HMAC-BLAKE3 vs HKDF-Extract.** All three are PRFs over the input. HMAC-SHA256 is the conservative audit-friendly choice (NIST SP 800-198; widely deployed; FIPS-validated). HMAC-BLAKE3 is faster and equally sound (BLAKE3 has stronger compression-function security than SHA-256 in some regimes) but less audit-anchored. **Recommend HMAC-SHA256 per the registry's L4/L5 "audit-friendly primitives" preference (matches U6 / U31 / U32 disposition).**

**Single subtle gotcha — `canonical(plaintext)` matters.** If `canonical()` is non-injective (two distinct plaintexts canonicalize identically), the adversary can craft a "shadow plaintext" with the same blinded-CID. **Mitigation:** the inner BLAKE3 already operates on canonical bytes (per U3 canonical-TLV-encoded BindingContext discipline); reuse the same canonical-encoder for plaintext_cid_local computation. **Specifically:** `plaintext_cid_local = BLAKE3(DAG-CBOR-canonical-encoded(DropBundlePayload))` per U30 (DAG-CBOR consolidator-leaned LOAD-BEARING). DAG-CBOR is canonical-by-spec (deterministic key-ordering + canonical integer encoding); injectivity is structural.

**Verdict on crypto soundness:** Option D's primitive (HMAC-SHA256(K_Atrium, BLAKE3(canonical(plaintext)))) is **cryptographically sound for the stated threat**. No NEW crypto primitive required; reuses HMAC-SHA256 already implied by Layer-B's HKDF chain (HKDF-Extract is HMAC under the hood per RFC 5869). Composes with codepoint-discipline (CLAUDE.md #5). Composes with the existing crypto-agility shape via Inv-16.

### §2.2 Cross-Atrium dedup: NOT a problem; actually-desirable

Ben asked: does Option D enable dedup across DIFFERENT Atriums when the same content is shared in both?

**Mechanical answer:** No. Different K_Atrium → different blinded CIDs for the same plaintext. Storage host with both Atriums' envelopes sees two distinct blinded-CIDs; cannot determine they reference the same plaintext.

**Is this a problem?** No — it's the **desired property**:
- If cross-Atrium dedup WORKED, an adversary in one Atrium could probe whether OTHER Atriums share the same content. That's a confidentiality leak across Atrium boundaries — exactly the kind of "lateral surveillance" Benten's per-Atrium-isolation discipline (CLAUDE.md #18 authority-isolation vs confidentiality-isolation) is designed to prevent.
- Storage savings from cross-Atrium dedup are second-order at Benten's expected scale (per-Atrium content; not the public-blob-store scenario where convergent-encryption literature was designed for).
- The forkability semantic specifically wants Atrium-A's content to be a *separate identity* from Atrium-B's content even when textually identical — content-shared-by-citation in two Atriums is intended to surface as two distinct "drop slots," one per Atrium's authorization context.

**Verdict on cross-Atrium dedup:** Loss-of-property is **the intended behavior**; not a regression vs Option C (which also doesn't cross-Atrium-dedup) and not a regression vs the original dual-CID U18 (which DID cross-Atrium-dedup, but that was an *unintentional leak* that the equality-oracle threat surfaces).

### §2.3 Within-Atrium dedup: YES by design

When Alice + Bob both store Drop bundle X in Atrium-α, both compute the same `plaintext_cid_atrium = HMAC-SHA256(K_Atrium_α, BLAKE3(X))`. Both can answer "do I have X?" against each other's storage by exchanging blinded-CIDs. iroh-blobs can address by blinded-CID at the Atrium-internal-peer-mesh layer; iroh-gossip can dedup by topic-attached blinded-CID (§3).

**Verdict:** Within-Atrium dedup preserved. This is the value-add Ben asked Option D to provide. **CONFIRMED.**

### §2.4 Storage-host probe-resistance: CONFIRMED

Untrusted 3rd-party storage host (e.g. a public iroh-relay; a cloud-blob-host; a co-tenant in a colo) does NOT have K_Atrium. Storage host with access to all stored envelopes can compute `BLAKE3(candidate_plaintext)` freely, but cannot compute `HMAC-SHA256(K_Atrium, ·)`. The probe attack reduces to brute-forcing K_Atrium, which is 2^256 work.

**One subtle attack worth naming explicitly:** if the storage host is *also* an Atrium member (insider threat — Mallory is a peer in Atrium-α AND runs the storage tier), Mallory has K_Atrium_α and CAN probe. **Mitigation:** treat Atrium-member-trust and storage-host-trust as the SAME trust boundary in the threat model; Compromise mint at v1-beta to disclose this honestly. See §5 Option G for a per-content-class scope refinement that bounds this.

**Verdict on storage-host probe-resistance:** **CONFIRMED for non-member storage hosts.** Disclose member-storage-host edge case in a new Compromise (#46-suggest); document as a property-not-an-attack (Atrium members are intentionally trusted with K_Atrium per §6).

### §2.5 What if K_Atrium leaks?

K_Atrium leak (one Atrium member's vault compromised by adversary) consequences:

- **Past blinded-CIDs disclosed:** adversary can now probe storage-host content for any plaintext they can guess. **Note:** this is FORWARD + BACKWARD dedup-fingerprint disclosure — once K_Atrium is in adversary's hands, they can probe all past + future stored content under that K_Atrium.
- **Past plaintext content NOT directly disclosed:** the adversary still needs the HPKE-Layer-C decap keys (per-recipient sk) AND the per-Node K(N) AEAD keys (derived from K_principal). K_Atrium leak does NOT decrypt content.
- **Equivalent risk class:** equivalent to "membership-fingerprint-leak" — adversary learns WHICH content is in the Atrium, not the content itself. Bounded confidentiality damage.

**Mitigation candidates:**

1. **K_Atrium rotation on-fork-only (RECOMMENDED — see §2.6 below):** scopes the leak-blast-radius to the time-window between Atrium creation and the next fork. For long-lived Atriums that never fork, K_Atrium leak is unbounded; users who care should periodically fork the Atrium (UX-affordance for "rotate Atrium dedup-key").

2. **K_Atrium rotation per epoch (e.g. yearly), distributed via new multi-stanza HPKE wrap to all current members.** This is what CGKA-LITE shape Option D'-rot looks like (§4). Cost: one extra multi-stanza HPKE wrap per epoch per member; no impact on past content (which keeps its old K_Atrium-blinded CIDs; recipients keep both old + new K_Atrium in local vault).

3. **NEVER-ROTATE, disclose-honestly:** for the v1-beta shape, simplest. Add Compromise #46-or-equivalent: "K_Atrium leak discloses Atrium-membership-fingerprint forward + backward; recovery requires Atrium-fork." Most-forkable Atriums get bounded leak naturally; rarely-forked Atriums accept long-tail leak risk explicitly. **My recommendation for v1-beta.**

### §2.6 K_Atrium derivation

**The critical design choice.** Where does K_Atrium come from?

**Three candidates:**

| Candidate | Derivation | Properties |
|---|---|---|
| **(A) Random-on-creation** | `K_Atrium = CSPRNG-32-bytes()` at Atrium-creation time; stored as opaque bytes in Atrium-creator's vault; distributed to new members via multi-stanza HPKE wrap on member-join | Simple. Stable. Independent of all other key material. Forkability-clean (fork-event mints NEW K_Atrium for child Atriums; parent unchanged). RECOMMENDED. |
| **(B) Derived from member-DID-set** | `K_Atrium = HKDF(union(sorted(member_dids)), salt="Benten/Atrium-K-v1")` | DETERMINISTIC (any member can recompute); zero distribution cost. But: changes on every membership change → re-blinds ALL past content's CIDs → DEDUP-FINGERPRINT-DISCONTINUITY on every member join/leave → defeats Option D's value-add. REJECTED. |
| **(C) Derived from creator-DID + Atrium-nonce** | `K_Atrium = HKDF(creator_principal_pubkey, Atrium_creation_nonce, info="Benten/Atrium-K-v1")` | Deterministic-from-creator-perspective; new members must receive Atrium_creation_nonce (small) plus creator's principal authority → derivable; but couples K_Atrium to creator's K_principal. If creator's K_principal rotates, K_Atrium changes; bad for stability. REJECTED. |

**Recommendation: (A) Random-on-creation.** Reuses the per-Atrium-state shape that Atrium-creator already manages (Atrium has metadata: name + creator-DID + creation-time + member-set; adding `K_Atrium` to that metadata is mechanical). Distribution is multi-stanza-HPKE per U17 — already ratified as a primitive. Forkability is clean: when Atrium forks, child Atrium's creator mints a NEW random K_Atrium for the child; the parent's K_Atrium is unchanged for ongoing parent activity.

**K_Atrium is DISTINCT from all other K-class keys:**
- `K_principal` = each user's principal vault key (Layer-A); per-user.
- `K(N) = KDF(K_principal, N.cid)` = per-Node AEAD key (Layer-B); per-user-per-node.
- `DAK` = device-authentication key (Layer-D); per-device.
- **NEW: `K_Atrium`** = per-Atrium dedup-blinding key (Layer-C-aux); per-Atrium. **Lives at the Atrium-metadata layer**, not at the user-vault layer.

**Storage:** K_Atrium is stored encrypted-under-K_principal in each member's local vault (Layer-A AEAD; reuses existing vault primitive). Distribution: at Atrium-join time, sender (existing member) HPKE-Seals(joining-member-pubkey, K_Atrium) — one-shot wrap; not a CGKA flow.

### §2.7 K_Atrium rotation policy — fork-on-rotate (RECOMMENDED)

Ben's question: rotate on Bob-leaves? Never? Sometimes? Trade-offs:

| Policy | Pro | Con |
|---|---|---|
| **Never rotate** | Simple; preserves Compromise #31 forever-valid Drop semantic (Bob who left still has K_Atrium and CAN compute past blinded-CIDs — but that's CONSISTENT with "Bob keeps past content"); zero CGKA-like infrastructure | K_Atrium leak window unbounded |
| **Rotate on every membership change** | Bounds K_Atrium-leak blast radius to one membership-epoch | Requires CGKA-like distribution; breaks forkable semantics (re-blinds CIDs for content that should remain stable for old members); HIGH cost; structural mismatch with ratified forkability |
| **Rotate on FORK ONLY** (RECOMMENDED) | Bounds K_Atrium-leak blast radius to one fork-epoch; matches forkability semantic exactly (fork = new Atrium = new K_Atrium); user has UX-affordance ("fork to rotate dedup-key"); zero CGKA infrastructure (per-fork is one-shot wrap to new member set) | Long-lived never-forked Atriums accept long-tail leak risk |

**Justification for fork-on-rotate:**

- The forkability semantic Ben ratified (2026-05-27) is: "Atrium-as-forkable not messaging-leave-forgets; when member leaves they keep copy of past content; future content excludes them via recipient-set exclusion." Under this semantic, Bob-leaves-Atrium does NOT mutate Bob's view of past content. Therefore K_Atrium SHOULD NOT rotate on Bob-leaves (would mutate past CIDs Bob can compute).
- Fork-events are explicitly a "new identity" moment. The whole point of fork is to start a new lineage; minting a new K_Atrium for the child is consistent.
- K_Atrium-leak blast radius is bounded by user UX (fork to recover); users with high-risk Atriums fork frequently, users with low-risk Atriums fork rarely. Self-tuning.
- Zero CGKA-like infrastructure: forks happen via Atrium-creation, which already has K_Atrium-mint as the standard flow.

**Verdict:** **K_Atrium ROTATES ON FORK ONLY.** Disclose long-lived-Atrium leak risk in Compromise #46 (new mint). Surface "fork to rotate dedup-key" as UX-affordance in post-v1-beta UX wave.

---

## §3 iroh-blobs vs iroh-gossip composition with Option D

Ben asks: would iroh-gossip make per-Atrium dedup more valuable + Atrium-size scaling more possible?

### §3.1 iroh-blobs under Option D — dedup story

iroh-blobs is pull-based content-addressed: recipient asks for CID; storage host stores + serves. Under U18 dual-CID, the on-wire CID is `envelope_blob_cid` (per-recipient distinct because each recipient gets a per-recipient HPKE stanza wrap).

**Under Option C (encrypted-inside-envelope plaintext_cid):**
- Storage host sees only `envelope_blob_cid` (per-recipient distinct).
- Cross-recipient dedup: **NONE** (each recipient has a different envelope blob).
- Within-recipient dedup: only if the recipient re-stores the same envelope (degenerate; trivial).
- **iroh-blobs dedup story under Option C: ZERO useful Atrium-dedup at the transport layer.** All dedup happens after decrypt at the recipient's local index over `plaintext_cid_local`.

**Under Option D (atrium-blinded `plaintext_cid_atrium` on the wire):**
- Storage host sees `envelope_blob_cid` (per-recipient distinct) + `plaintext_cid_atrium` (per-Atrium-shared).
- **Atrium-internal storage hosts (or peers acting as caches) can dedup by `plaintext_cid_atrium` at the iroh-blobs addressing layer** — but iroh-blobs addressing is BLAKE3-CID-keyed, so the dedup happens by having `plaintext_cid_atrium` *as* a transport CID alongside the BLAKE3-of-blob CID.
- **Mechanical realization:** Atrium-internal peers maintain a `plaintext_cid_atrium → envelope_blob_cid_list` index (replicated via the existing Atrium-sync mechanism). When peer-B receives a Drop bundle published by peer-A, peer-B can resolve "do I already have content for `plaintext_cid_atrium = X`?" by index lookup, skipping the blob fetch entirely.
- **Cross-recipient dedup within Atrium:** **YES** — if Alice publishes the same content to Bob and Carol in Atrium-α, the blinded `plaintext_cid_atrium` is identical, so Bob and Carol can recognize the duplicate even though their envelope_blob_cids differ.

**Storage savings estimate:** For a typical Atrium of N members with M content items each sent on average to K recipients:
- Under C: storage = N × M × K envelope-blobs (no dedup).
- Under D with index: storage = M × K envelope-blobs (per-recipient still distinct due to HPKE stanza) BUT recipients can avoid re-fetching the same plaintext-content K-1 times → bandwidth savings = (K-1)/K. **Modest but real.**

### §3.2 iroh-gossip under Option D — where the elegance lives

iroh-gossip is push-based mesh broadcast using HyParView (membership) + PlumTree (epidemic broadcast). Properties:

- **Per-message dedup at protocol level:** PlumTree's IHAVE/IWANT discipline structurally avoids redelivering messages to peers who already have them. Each peer maintains a per-topic message-ID seen-set; PlumTree's eager + lazy push split ensures single-delivery-per-peer-per-message in steady state. **This is the per-Atrium dedup Ben asked about — and it's at the protocol layer.**
- **Scaling:** HyParView maintains O(log N) active view + larger passive view; PlumTree broadcasts in O(log N) hops with bounded per-peer load. iroh-gossip's stated scaling target is "a few thousand peers" per single topic (per docs.iroh.computer/connecting/gossip). **For Atriums of ≤1000 members, gossip outscales blobs-unicast-fan-out.**
- **Per-Atrium topic:** the natural mapping is one gossip topic per Atrium. Topic-id is the addressing primitive (per C3 G7 note).
- **Carrying CIDs:** PlumTree messages carry arbitrary bytes; carrying a content-addressed envelope is mechanical.

**The elegant Option-D-shape under iroh-gossip:**

```
For Drop bundle X in Atrium-α:
  1. Sender computes:
     plaintext_cid_atrium = HMAC-SHA256(K_Atrium_α, BLAKE3(canonical(X)))
     envelope = EncryptedEnvelope { payload: HpkeMultiBase(per-recipient stanzas), ... }
     envelope_blob_cid = BLAKE3(serialized(envelope))
  2. Sender publishes via iroh-gossip on topic = atrium_α_topic_id:
     gossip_message = GossipPayload {
       message_id: plaintext_cid_atrium,      // ← PlumTree-dedup-key
       transport: BlobRef { envelope_blob_cid }, // ← pull-on-demand
     }
  3. PlumTree disseminates with single-delivery-per-peer.
  4. Each Atrium member's gossip receiver:
     a. If message_id (= plaintext_cid_atrium) already in local seen-set, IGNORE (dedup).
     b. Else, fetch envelope via iroh-blobs lookup by envelope_blob_cid.
     c. HPKE-Open the per-recipient stanza.
     d. Decode plaintext, compute plaintext_cid_local, index into local graph.
```

**Why this is structurally beautiful:**

- **PlumTree's per-topic seen-set IS the per-Atrium dedup index.** No separate index needed. The gossip protocol substrate provides the dedup-table for free.
- **gossip + blobs hybrid:** gossip pushes the *announcement* (small; ~64 bytes message-ID + envelope_blob_cid pointer); blobs handles the *payload* (large; the actual envelope). This is the canonical "push notification + pull payload" pattern — proven at scale in IPFS pubsub, GossipSub (libp2p), Scuttlebutt.
- **Atrium-size scaling:** log-N gossip-spread for the announcement; O(1) per-recipient envelope fetch from any peer that has it (no central storage host required). **For 1000-member Atriums, this is dramatically more scalable than 1000-way blobs unicast.**
- **Storage-host-probe-resistance preserved:** non-member storage hosts (relay nodes that store envelope_blob_cid → bytes) still don't have K_Atrium; gossip topic-id is per-Atrium and access-controlled at the gossip subscription layer (HyParView won't admit non-members if Atrium is closed-membership).

**Composition with Compromise #31 (forever-valid Drop bundles):**

- iroh-gossip is lossy-by-design (PlumTree's "eventually reaches most or all subscribers; no strict guarantee"). For Compromise #31's forever-valid semantic, gossip alone is INSUFFICIENT — must be supplemented with iroh-blobs durable-store fallback (storage host retains the envelope_blob_cid even if a recipient missed the gossip-announcement; recipient can poll-fetch later).
- **The hybrid (gossip for announce + blobs for durable store + pull) gives BOTH the per-Atrium dedup-at-protocol property AND the Compromise #31 forever-valid property.** Neither alone does.

**Composition with forkability:**

- On Atrium-fork: parent topic and child topic are distinct (different topic-ids → different gossip swarms). Pre-fork members continue to receive parent-topic gossip; new fork-only members receive child-topic gossip.
- Pre-fork drops remain accessible via parent-topic plus iroh-blobs durable-store (the per-Atrium archive).
- Clean.

### §3.3 iroh-gossip-vs-blobs scaling comparison under Option D

| Atrium size | Blobs-only (current ratified) | Blobs + Gossip (proposed hybrid) | Win condition |
|---|---|---|---|
| 2-5 members | Negligible difference | Negligible difference | Blobs simpler; gossip overhead not justified |
| 10-50 members | Sender does N HPKE wraps + N unicast posts | Sender does N HPKE wraps + 1 gossip publish + blobs store | Hybrid wins on sender bandwidth (gossip O(log N) vs O(N) unicast) |
| 100-1000 members | Sender unicast fan-out is bottleneck; sender bandwidth O(N) | gossip-spread is O(log N) sender-load; per-recipient single-delivery via PlumTree | Hybrid dramatically wins |
| 1000+ members | Unscalable without external relay infrastructure | Approaches gossip protocol limit (~few thousand per topic per docs) | Hybrid is the only viable path |
| 10000+ members | Infeasible | Requires multi-topic sharding or hierarchical Atrium structure | Beyond Benten's v1-beta scope; future work |

**The strategic implication:** at Benten's PROBABLE v1-beta-target Atrium size (a few to a few-hundred members per Atrium for personal/small-org use cases), iroh-blobs alone is sufficient. **But the gossip-hybrid is what makes a 1000-member Atrium FEASIBLE; without it, Benten's "Atrium peer-mesh" framing is bottlenecked at ~50-100 members per Atrium due to sender unicast fan-out.**

### §3.4 Recommendation: CODEPOINT-RESERVE gossip-transport at v1-beta; ship blobs-only at v1-beta; pilot hybrid at G-CORE-ATRIUM-SCALE-1

- **v1-beta:** ship iroh-blobs only per RATIFIED-sharing-and-confidentiality-2026-05-21. No transport change.
- **v1-beta CODEPOINT-RESERVE:** mint `LAYER_C_ATRIUM_GOSSIP_TRANSPORT_BINDING = 0x6330` (per CLAUDE.md #5 codepoint-discipline) as a transport-binding variant in `BindingContext` per U10. Field present at slot-level; impl deferred. **Cost: ~1 wave-day at v1-beta to lock the slot.**
- **Post-v1-beta wave G-CORE-ATRIUM-SCALE-1:** implement gossip-transport binding. Add iroh-gossip dependency. Pilot with one large Atrium (e.g. the Benten community Atrium itself). Validate scaling. **Estimated cost: 5-8 wave-days.**
- **Inv-19-pending (new):** when Atrium membership exceeds 50, the gossip-transport binding SHOULD be active. Below 50, blobs-unicast is sufficient. (Threshold tunable; document in THREAT-MODEL.md U40 + audit-readiness wave.)

---

## §4 CGKA-deferral re-evaluation

### §4.1 The CGKA-deferral argument re-stated

Ben ratified 2026-05-26: CGKA (continuous group key agreement; TreeKEM-style) is DEFERRED because its value-add (forward-secrecy across membership changes; post-compromise-security) does NOT match Benten's forkability semantic ("Atrium-as-forkable not messaging-leave-forgets"; member-leaves-keeps-past-content). Multi-stanza HPKE chosen for v1-beta groups instead.

### §4.2 Does Option D require CGKA-like infrastructure?

**Surface concern:** Option D requires K_Atrium-shared-key. If K_Atrium must rotate on member-leave (to bound leak blast radius), that's CGKA-adjacent territory.

**Resolution:** under the recommended **K_Atrium rotation policy of FORK-ONLY** (§2.7), K_Atrium does NOT rotate on member-leave. Therefore:

- K_Atrium **distribution** infrastructure is needed (on member-JOIN: existing-member HPKE-Seals K_Atrium to joining-member-pubkey; one-shot wrap; not CGKA).
- K_Atrium **rotation** infrastructure is NOT needed (rotation = fork = new Atrium = new K_Atrium minted from scratch; uses existing Atrium-creation flow).

**This is structurally distinct from CGKA.** CGKA's value is the *continuous* part — ongoing rotation with PCS + FS as members come and go. Option D needs neither continuous-rotation nor PCS/FS-across-membership; it needs ONE-SHOT distribution. **Multi-stanza HPKE (already ratified per U17) is the primitive.**

### §4.3 The CGKA-deferral STILL HOLDS

The original deferral argument — CGKA-FS-across-membership-changes mismatches forkability — applies identically under Option D. Option D's K_Atrium distribution is a degenerate case of CGKA (one-shot key-wrap; no rotation), which doesn't need CGKA-level infrastructure.

**Verdict:** **CGKA-deferral STANDS.** No change to the 2026-05-26 ratification needed.

### §4.4 Is there a CGKA-LITE shape worth considering?

For completeness: would a "CGKA-LITE that distributes K_Atrium on join but never rotates" be more elegant than ad-hoc multi-stanza HPKE wraps?

**Fork-Resilient CGKA (Alwen-Hartmann-Kiltz-Mularczyk 2023, eprint 2023/394)** is the closest published variant. It supports group-split + group-merge operations with cryptographic guarantees about fork-history. However:

- FR-CGKA still maintains the *continuous* property (ongoing key rotation with FS); Option D doesn't need that.
- FR-CGKA's overhead is TreeKEM-level (O(log N) per update; full crypto-state per member). Multi-stanza HPKE one-shot is O(N) per group-add but ZERO ongoing overhead.
- FR-CGKA is not yet IETF-track or production-ready; multi-stanza HPKE is RFC 9180.

**Verdict:** **Even Fork-Resilient CGKA is overkill for Option D's needs.** Multi-stanza HPKE one-shot distribution is the right primitive. Re-affirms the original CGKA-deferral.

### §4.5 Implication for K_Atrium distribution at member-join

Mechanical flow at Atrium-α member-add:

```
Existing-Member-Alice (has K_Atrium_α) admits Joining-Member-Carol:
  1. Alice retrieves K_Atrium_α from her vault (decrypt under K_principal_Alice).
  2. Alice constructs:
     EncryptedEnvelope {
       codepoint: 0x6340,  // LAYER_C_ATRIUM_KEY_DISTRIBUTION (new codepoint reserve at v1-beta)
       payload: HpkeBase {
         recipient_pubkey: Carol.kem_pubkey,
         encapped_key + ciphertext: HPKE-Seal(Carol.kem_pubkey,
                                              info=canonical_binding(
                                                codepoint=0x6340,
                                                BindingContext::AtriumKeyDistribution {
                                                  atrium_id, k_atrium_generation,
                                                  sender_did=Alice.did, recipient_did=Carol.did,
                                                  sealed_at_epoch_hour, valid_until_epoch_hour,
                                                  recipient_key_generation
                                                }),
                                              plaintext = K_Atrium_α || atrium_metadata)
       }
     }
  3. Alice posts the envelope via iroh-blobs or sendme-ticket to Carol.
  4. Carol HPKE-Opens, extracts K_Atrium_α, stores encrypted-under-K_principal_Carol in her vault.
  5. Carol can now compute plaintext_cid_atrium for Atrium-α drops.
```

**Codepoint mint at v1-beta:** `LAYER_C_ATRIUM_KEY_DISTRIBUTION = 0x6340`. Add to U10 `BindingContext` codepoint family. **Cost: ~0.5 wave-day at v1-beta.**

---

## §5 Alternative options surveyed (D' / E / F / G / H / I)

### §5.1 Option D' — member-derived K_Atrium (rejected; named for completeness)

`K_Atrium = HKDF(union(sorted(member_dids)), salt)`. Deterministic from membership; no distribution cost.

**Fatal flaw:** K_Atrium changes on every membership change → re-blinds ALL past content's plaintext_cid_atrium → past dedup-fingerprints become unrecognizable to current members. Defeats the value-add. **REJECTED.**

(A weaker variant: `K_Atrium = HKDF(creator_did + atrium_creation_nonce + salt)` is what §2.6 Candidate C is; rejected for K_principal-coupling reasons.)

### §5.2 Option E — DupLESS-style OPRF via a Benten Atrium-key-server

Use an Oblivious-PRF (OPRF) protocol where K_Atrium lives on a key-server (NOT on each member's vault). Members compute `plaintext_cid_atrium` by sending a blinded request to the key-server; key-server applies the PRF; returns blinded result; member unblinds.

**Pro:** K_Atrium never leaves the key-server; member-vault-compromise doesn't leak K_Atrium.

**Con (fatal for Benten):**
- Requires an always-online Atrium-key-server. Benten's "decentralized peer-mesh" framing explicitly does NOT have central servers.
- Defeats offline-first semantic: member-offline can't compute blinded-CID without OPRF round-trip.
- Adds a new trusted party (the key-server) — orthogonal to storage-host trust; doesn't actually reduce trust footprint.

**Verdict:** REJECTED structurally; doesn't compose with Benten's P2P-mesh + offline-first commitments.

### §5.3 Option F — full PSI for content-equality

Use Private-Set-Intersection cryptography (Kissner-Song 2005; Pinkas-Schneider-Zohner 2014) for dedup. Two members compute "do we both have this content?" with strong privacy.

**Pro:** strongest privacy properties for dedup; no shared key needed.

**Con (fatal for v1-beta):**
- Computational cost: PSI protocols are O(N) per query with substantial constants; orders of magnitude slower than HMAC.
- Requires interactive protocol between pairs of members; doesn't compose with offline-first.
- Massive impl complexity; not v1-beta-realistic.

**Verdict:** REJECTED for v1-beta. Optionally revisit at v2 if a Benten use-case emerges that needs PSI-strength privacy on dedup.

### §5.4 Option G — per-content-class K_Atrium-derivative (refinement of D, optional)

Split K_Atrium into per-content-class sub-keys: `K_Atrium_classX = HKDF(K_Atrium, class_label)`. Rotation per class.

**Use case:** Atrium has high-sensitivity content (e.g. medical notes) vs low-sensitivity (e.g. chat); the high-sensitivity class gets its own sub-key, rotated more frequently or distributed to a tighter sub-set of members.

**Pro:** finer-grained leak-blast-radius. Composes additively with Option D.

**Con:** more complex; UX-affordance for content-classification needed.

**Verdict:** **DEFERRED to v2 / v1-GM** as an additive shape. Mint codepoint reserve at v1-beta (~0.25 wave-day) for `LAYER_C_ATRIUM_CLASS_KEY_DISTRIBUTION`. Document in roadmap as "post-v1 if users want it."

### §5.5 Option H — encrypted Bloom-filter / probabilistic dedup

Storage host sees only Bloom-filter-encoded fingerprints; can answer "have I seen this fingerprint?" probabilistically without learning specific content.

**Pro:** storage-host can do crash-dedup probabilistically without K_Atrium (?).

**Con:** Bloom filters leak set-membership over time; under aggregate-traffic-analysis the Bloom-filter equivalence-class is enumerable. Doesn't actually defeat the equality-oracle threat — only obscures it. Storage cost overhead is non-trivial. Composes awkwardly with content-addressing.

**Verdict:** REJECTED — doesn't structurally defeat the equality-oracle; just adds probabilistic noise.

### §5.6 Option I — the proposed elegant shape: K_Atrium as one instance of `K_DedupScope` parameter

**The shape:** generalize K_Atrium from "the Atrium key" to a `K_DedupScope` parameter in the EncryptedEnvelope codepoint family. Per-Drop-bundle, sender chooses a dedup-scope:

| Scope | K_DedupScope | Dedup property | Use case |
|---|---|---|---|
| Per-Atrium (default) | K_Atrium | Atrium-internal dedup; per §2-§3 | Most Drops |
| Per-creator | K_creator (per-author key) | Only content from one author dedups; sender-anonymity within Atrium | Sender-anonymous publishing |
| Per-recipient (= Option C) | K_recipient | No cross-recipient dedup; degenerate single-member scope | Maximally-private 1:1 drops |
| Per-content-class (= Option G) | K_Atrium_classX | Class-scoped dedup | High-sensitivity content classes |
| Per-thread / per-document (future) | K_thread | Thread-scoped dedup | Long-running collaborative editing |

**Mechanical realization:**

- EncryptedEnvelope gets a new field: `dedup_scope_id: Option<DedupScopeId>` where `DedupScopeId = (scope_kind: u8, scope_instance: [u8; 16])`.
- `plaintext_cid_dedup = HMAC-SHA256(K_DedupScope, BLAKE3(canonical(plaintext)))` where K_DedupScope is resolved from `dedup_scope_id` via the local vault.
- `None` dedup_scope_id = Option C (no blinded CID on the wire).
- `Some(atrium_default)` = Option D.
- `Some(creator_X)` / `Some(class_Y)` = Options G / Option-I extensions.

**Pro:**
- **Elegance:** ONE primitive (HMAC-PRF on a fixed input) generalizes ALL dedup-scope variants. Each variant is a key-resolution rule, not a separate crypto construction.
- **Composes with CLAUDE.md #5 codepoint-discipline:** each scope-kind is a u8 codepoint; new scope-kinds are additive via existing crypto-agility doctrine.
- **Composes with Option C as the `None` case:** Option C is the degenerate "no dedup scope" of Option I. **Migration C → D → I is purely additive.**
- **User-facing affordance:** sender chooses per-Drop "dedup scope" via UX (default = Atrium; opt-in tighter scope for sensitive content).
- **K_Atrium becomes one well-defined scope; not a load-bearing global parameter.** Reduces semantic surface area.

**Con:**
- Slight wire-format expansion (~17 bytes per envelope: 1 byte scope_kind + 16 bytes scope_instance).
- Requires a scope-resolution table in the vault (mapping scope_instance → K_DedupScope). Mechanical.

**Verdict:** **Option I is what I recommend Ben ratify as the proper shape**, with **Option D (per-Atrium scope) as the v1-beta DEFAULT instance**. Other scope-kinds (G per-class; per-creator; per-thread) are codepoint-reserve only at v1-beta; impl deferred.

This satisfies Ben's "more elegant solution/shape through that community lens" framing by recognizing that K_Atrium is not a special primitive but an instance of a general parameter — the same way "K_principal" is just an instance of "per-user vault keys."

---

## §6 Final recommendation with justification + Q3-decision matrix per scenario

### §6.1 The winner

**Adopt Option D as the v1-beta DEFAULT instance of the more-general Option I (K_DedupScope-parameterized blinded plaintext-CID).**

Specifically:

1. **At v1-beta LOAD-BEARING:**
   - U18 dual-CID model SPLIT: `plaintext_cid_local` (BLAKE3 of canonical-plaintext; computed at decrypt-time; NEVER on wire) + `plaintext_cid_atrium` (HMAC-SHA256(K_Atrium, plaintext_cid_local); on wire; Atrium-internal-peer-visible).
   - `EncryptedEnvelope.dedup_scope_id: Option<DedupScopeId>` field minted at v1-beta. v1-beta DEFAULT is `Some(atrium_default)`; `None` is the Option-C-degenerate; other scope-kinds are CODEPOINT-RESERVE.
   - K_Atrium: random-on-creation, fork-on-rotate only.
   - K_Atrium distribution: multi-stanza HPKE at member-join (codepoint `LAYER_C_ATRIUM_KEY_DISTRIBUTION = 0x6340`).
2. **At v1-beta CODEPOINT-RESERVE:**
   - `LAYER_C_ATRIUM_GOSSIP_TRANSPORT_BINDING = 0x6330` (per §3.4).
   - `LAYER_C_ATRIUM_CLASS_KEY_DISTRIBUTION` (per §5.4 Option G).
   - Per-creator + per-thread scope-kinds (per §5.6 Option I).
3. **At v1-beta NEW Compromise mints:**
   - **Compromise #45-suggest:** K_Atrium leak discloses Atrium-membership-fingerprint forward + backward; recovery requires Atrium-fork (per §2.5 + §2.7).
   - **Compromise #46-suggest:** Atrium-member-storage-host insider-threat — peers who are also storage hosts can probe; treat Atrium-member-trust and storage-host-trust as the same trust boundary (per §2.4).
4. **At v1-beta NEW invariant:**
   - **Inv-19-suggest:** every EncryptedEnvelope with non-`None` dedup_scope_id has a corresponding scope-resolution entry in some recipient's vault; orphan envelopes (no recipient can resolve dedup_scope_id) are TYPED-REJECT at decode.
5. **Post-v1-beta wave G-CORE-ATRIUM-SCALE-1:**
   - Implement gossip-transport binding (per §3.2-§3.4).
   - Pilot with the Benten community Atrium.
6. **CGKA-deferral re-affirmed; no change** (per §4.3).

### §6.2 Why D wins over C as default

| Property | Option C | Option D | Winner |
|---|---|---|---|
| Storage-host equality-oracle defense | YES (no plaintext_cid on wire) | YES (only blinded plaintext_cid on wire) | TIE |
| Within-Atrium dedup at transport | NO | YES | D |
| iroh-gossip protocol-level dedup | NOT USEFUL (per-recipient distinct) | USEFUL (PlumTree dedup on plaintext_cid_atrium) | D |
| Atrium-size scaling beyond ~100 members | Bottlenecked by unicast fan-out | Unblocked via gossip hybrid | D |
| Composition with forkability | YES (recipient-local index) | YES (K_Atrium random-on-creation; fork = new K_Atrium) | TIE |
| Wire-format surface area | Smaller (no extra field) | +17 bytes per envelope | C (marginal) |
| K_Atrium-leak risk | N/A | NEW (bounded by Compromise #45; UX-fork-affordance mitigation) | C (marginal) |
| Future-extensibility (Option I generalization) | DEGENERATE CASE of D | RICH parameter family | D |
| Impl complexity at v1-beta | Lower | +1-2 wave-days | C (marginal) |

**Net:** D wins on the load-bearing P2P-architecture properties (gossip-dedup + scaling); C wins on three marginal properties (wire-size, leak-risk-surface, impl-cost). The marginal C-wins are quantitatively small; the D-wins are qualitatively load-bearing.

**Bonus:** because D is the parameterized superset of C (D with `dedup_scope_id = None` = C), choosing D doesn't preclude C; per-Drop choice is available.

### §6.3 Q3-decision matrix per scenario

| Scenario | Recommended scope | Notes |
|---|---|---|
| Default Atrium Drops | Option D (atrium_default) | Wins per §6.2 |
| Sender-anonymous publishing | Option I per-creator | Codepoint-reserve at v1-beta; impl post-v1 |
| High-sensitivity content (medical, financial) | Option G per-class | Codepoint-reserve at v1-beta; impl post-v1 |
| 1:1 Drop with no dedup intent | Option C (`None`) | Available at v1-beta as the degenerate scope |
| Untrusted-storage-host scenario where Atrium membership itself is sensitive | Option D **PLUS** Sealed-Sender per U22 + per-relay-unlinkability per U23 | Composite; covers metadata leakage too |
| Cross-Atrium content sharing (rare) | Option C in the second Atrium; sender re-publishes | Cross-Atrium dedup intentionally not provided per §2.2 |
| Atrium expected to grow beyond 100 members | Option D **PLUS** gossip-transport binding (post-v1-beta G-CORE-ATRIUM-SCALE-1) | Hybrid push/pull |

### §6.4 When to fall back to C

C is preferred over D in the following cases (all are 1:1 Drop scenarios where the dedup-scope value is zero):

- Single-recipient Drops with no future-recipient anticipated (no within-Atrium dedup possible).
- Drops crossing Atrium boundaries (where K_Atrium for one Atrium doesn't apply to the other).
- Drops where the sender prefers MINIMUM wire-format footprint (saves 17 bytes per envelope; matters at ultra-low-bandwidth scenarios).

**Per-Drop choice via `dedup_scope_id = None` is available; sender opts in/out per Drop.** Default is D; ergonomic UX surface for opting out exists.

---

## §7 R0 plan-doc implications + cost estimate

### §7.1 Changes to the 9-eyes consolidated registry

**Update U18 (dual-CID model):** SPLIT plaintext_cid into local + atrium axes. Registry row revision:

> **U18 — TRIPLE-CID model: `plaintext_cid_local` + `plaintext_cid_atrium` + `envelope_blob_cid`**
> - Statement: Drop bundle has THREE distinct CIDs. `plaintext_cid_local = BLAKE3(canonical(DropBundlePayload))` (NEVER on wire; recipient-local after decrypt; graph-stable; survives reseal + recipient-set evolution + cipher rotation + forks). `plaintext_cid_atrium = HMAC-SHA256(K_Atrium, plaintext_cid_local)` (blinded; on wire; Atrium-internal-peer-visible; storage-host-opaque without K_Atrium). `envelope_blob_cid = BLAKE3(canonical(EncryptedEnvelope serialized bytes))` (transport; changes on reseal; iroh-blobs addressing handle). Mapping `plaintext_cid_local → Vec<plaintext_cid_atrium>` (one local-CID may map to one atrium-CID per dedup_scope_id; usually 1:1 for default scope). Mapping `plaintext_cid_atrium → Vec<envelope_blob_cid>` (per-recipient envelopes share the same blinded plaintext-CID).

**NEW U41-suggest — dedup_scope_id parameter:**
> Statement: `EncryptedEnvelope.dedup_scope_id: Option<DedupScopeId>` where `DedupScopeId = (scope_kind: u8, scope_instance: [u8; 16])`. v1-beta scope_kinds: `0x00 = none` (Option C; no blinded plaintext_cid), `0x01 = atrium_default` (Option D; HMAC-SHA256(K_Atrium, plaintext_cid_local)). Reserved scope_kinds: `0x02 = per-creator`, `0x03 = per-class`, `0x04 = per-thread`, `0xFF = escape`.

**NEW U42-suggest — K_Atrium key management:**
> Statement: K_Atrium derivation: CSPRNG-32-bytes at Atrium-creation. Storage: per-member-vault-encrypted-under-K_principal. Distribution at member-join: HPKE-Seal under joining-member-kem-pubkey via codepoint `LAYER_C_ATRIUM_KEY_DISTRIBUTION = 0x6340`. Rotation policy: FORK-ONLY (parent K_Atrium unchanged on member-leave; child Atrium mints new K_Atrium on fork-creation). Member who has left RETAINS K_Atrium (consistent with Compromise #31 forever-valid + ratified forkability semantic).

**NEW Compromise #45-suggest:** K_Atrium leak discloses Atrium-membership-fingerprint forward + backward for the K_Atrium-epoch window; recovery via Atrium-fork.

**NEW Compromise #46-suggest:** Atrium-member-storage-host insider-threat — member-peers acting as storage tier can probe by K_Atrium; trust-boundary is Atrium-member-trust = storage-host-trust if same operator.

**NEW Inv-19-suggest:** every non-`None` dedup_scope_id maps to a vault-resolvable K_DedupScope for at least one recipient of the envelope.

**Update CGKA-deferral disposition:** re-affirm; document explicit reasoning that Option D's K_Atrium needs ONE-SHOT distribution, not CGKA-style continuous rotation.

### §7.2 Cost estimate (incremental to the registry's ~65-72 wave-day v1-beta total)

| Component | Wave-days |
|---|---|
| U18 split into triple-CID (local + atrium + blob) | ~0.75 |
| U41 dedup_scope_id field + codepoint family | ~1.0 |
| U42 K_Atrium key management + distribution flow (`LAYER_C_ATRIUM_KEY_DISTRIBUTION = 0x6340` codepoint) | ~1.5 |
| Inv-19 + Compromise #45 + Compromise #46 documentation | ~0.5 |
| THREAT-MODEL.md (U40) update for content-equality-oracle row + member-storage-host row | ~0.5 |
| Golden corpus extension for HMAC-blinded CID round-trips | ~0.5 |
| Kani injectivity proof extension for triple-CID disjoint | ~0.5 |
| **CODEPOINT-RESERVE slots** (`LAYER_C_ATRIUM_GOSSIP_TRANSPORT_BINDING`, per-class, per-creator) | ~0.5 |
| **Subtotal — INCREMENTAL** | **~5.75 wave-days** |
| **Buffer +30%** | **~1.75 wave-days** |
| **GRAND TOTAL INCREMENTAL** | **~7.5 wave-days** |

**Total v1-beta with Q3-Option-D adoption:** registry's ~65-72 + ~7.5 = **~72-80 wave-days = ~14.5-16 calendar-weeks.** Pushes against the upper bound of Ben's 7-15-week window. If tight, the Option I generalization (per-creator + per-class codepoints) can defer to v1-GM; pure Option D (just the default scope) is ~3.5 wave-days incremental.

### §7.3 Wave sequencing recommendation

- **Wave 1 (R3 implementer):** triple-CID split + dedup_scope_id field + K_Atrium random-on-creation primitive.
- **Wave 2 (R3 implementer, can parallelize):** K_Atrium distribution via `LAYER_C_ATRIUM_KEY_DISTRIBUTION` codepoint; multi-stanza HPKE flow for member-join.
- **Wave 3 (R3 implementer):** golden corpus + Kani injectivity proofs + THREAT-MODEL.md updates.
- **Wave 4 (post-v1-beta G-CORE-ATRIUM-SCALE-1):** gossip-transport binding implementation; iroh-gossip integration.

---

## §8 Self-assessment + confidence

### §8.1 What I am HIGH-confidence on (~80-90%)

- HMAC-SHA256(K_Atrium, BLAKE3(canonical(plaintext))) is cryptographically sound for the stated equality-oracle threat (literature anchor: DupLESS / message-locked encryption / Bellare-Keelveedhi-Ristenpart 2013). Standard NMAC reduction; no novel crypto.
- Cross-Atrium-dedup loss is intentional, not a regression.
- K_Atrium rotation policy = FORK-ONLY matches forkability semantic exactly; CGKA-deferral stands.
- iroh-gossip's PlumTree IHAVE/IWANT protocol provides per-message dedup at the protocol layer — proven property of the HyParView/PlumTree family (Leitão-Pereira-Rodrigues).
- Option D is the parameterized superset of Option C; choosing D doesn't preclude C; per-Drop choice via `dedup_scope_id` field.
- The triple-CID split (U18 refinement) is mechanically clean and required for the equality-oracle defense.

### §8.2 What I am MEDIUM-confidence on (~60-75%)

- The 17-byte wire-format overhead of `dedup_scope_id` is acceptable. Ben may push back on additional envelope fields; might prefer a codepoint-pair shape (one codepoint for Option-C-envelope; one for Option-D-envelope) instead of a parametric field. **Surface for ratification.** Confidence on "parametric field is better than codepoint-pair": MEDIUM-HIGH; codepoint-pair is simpler structurally and matches the existing CLAUDE.md #5 codepoint-discipline more cleanly.
- iroh-gossip scaling to "a few thousand peers per topic" per the docs is for ideal-condition synthetic benchmarks; real-world Atrium scaling under churny membership + intermittent connectivity may degrade. Pilot needed before promising 1000-member Atriums at v1-GM. Confidence on "gossip enables ~few-hundred-member Atriums": HIGH; on "gossip enables 1000+ member Atriums": MEDIUM.
- The CGKA-LITE-not-needed conclusion is robust UNDER the recommended fork-on-rotate policy. If Ben prefers tighter K_Atrium leak-blast-radius (e.g. rotate on every leave), CGKA-LITE infrastructure becomes relevant and the deferral may need revisit.

### §8.3 What I am LOW-confidence on (~40-55%; flagged for follow-up)

- Whether Option G (per-content-class K_Atrium-derivative) is genuinely useful at Benten's expected use-cases vs cargo-culted from generic enterprise-RBAC patterns. Defer to user-research; codepoint-reserve preserves the option.
- Whether `dedup_scope_id` collisions across Atriums (same scope_instance bytes randomly generated for two different Atriums) need explicit defense. At 128-bit random, birthday bound is 2^64 Atriums (unrealistic); 128-bit truncation should be safe. **But if Ben wants belt-and-suspenders, expand scope_instance to 32 bytes for ~0 wire-format cost increase relative to whole-envelope size.**
- Composition with the future Sealed-Sender variant (U22) under metadata-privacy: blinded plaintext_cid still reveals "two Drops have same content"; if metadata-privacy goal is "Drops should be unlinkable by content too," Option D doesn't deliver that. **Surface as documented limitation; Sealed-Sender + per-recipient-unlinkable (U25) is the harder shape.**

### §8.4 What this review explicitly does NOT cover

- Full IND-CCA2 reduction proof for the composed EncryptedEnvelope under the triple-CID model. (Existing C3 §2 reduction needs minor extension; ~0.5 wave-day at audit-readiness time.)
- iroh-gossip API surface vs Benten's specific needs (topic management; subscription auth; relay protocol). Dispatch iroh-specialist sub-review at R3/R5 per C3 G7.
- UX-affordances for "rotate dedup-key via fork" / "choose dedup scope per Drop" / "what does my Atrium look like to a storage host?" — UX-design wave, post-v1-beta.
- MLS-PQ specialist review of the K_Atrium-distribution multi-stanza HPKE under U17 cross-stanza substitution defense. Aligns with C3 G10 pre-existing recommendation.
- Detailed wire-format CBOR encoding of `DedupScopeId`. Pin at R0 design.

### §8.5 Net confidence on the recommendation

**HIGH (~85%)** that Option D as the v1-beta default instance of the more-general parameterized Option I is the right architectural shape. The qualifications (triple-CID split, K_Atrium fork-on-rotate, gossip codepoint-reserve, CGKA-deferral re-affirmation) are mechanical refinements grounded in the existing registry + ratified architectural commitments.

**LOW probability (~15%)** that some constraint I haven't seen (e.g. an unstated UX requirement; a regulatory requirement disclosing the K_Atrium-leak-risk surface; a deeper interaction with the L9-A3 recipient-key-retention policy) makes Option C-as-default the right call instead. In that case, Option D becomes a codepoint-reserve at v1-beta and is implemented post-v1-GM when the constraint is understood.

---

## §9 Citations

### Phase-4-meta-core internal artifacts (load-bearing)

- `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` (939 lines; consolidated registry)
- `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03` (717 lines; introduces dual-CID, lines 250-310 + 506)
- `phase-4-meta-core/option-f-plus-critique-c1-elegant-shape @ 3f5a4351` (724 lines)
- `phase-4-meta-core/option-f-plus-critique-c2-composability @ 9c548e5f` (767 lines; R-C3 plaintext_cid-collusion analysis lines 385-400)
- `phase-4-meta-core/option-f-plus-critique-c3-fresh-eyes-cryptographer @ 79c99aa5` (697 lines; G7 iroh-transport gap lines 360-400)
- `phase-4-meta-core/option-f-plus-critique-c4-process-discipline @ 6ea9718a` (618 lines)
- `phase-4-meta-core/option-f-plus-critique-c5-formal-methods @ 8e374a9d` (637 lines; CT-5 key-committing AEAD)

### Standing architectural docs

- `docs/SECURITY-POSTURE.md` — Compromise #30 (PQ-impl-audit-maturity); Compromise #31 (forever-valid Drop bundles); proposed Compromise #45 + #46 extensions per §6.1.
- `docs/INVARIANT-COVERAGE.md` — Inv-15 (3-layer signature decomposition); proposed Inv-16 (3-layer encryption decomposition); proposed Inv-19 (dedup_scope_id resolvability per §7.1).
- `CLAUDE.md` baked-in #5 (crypto-agility codepoint-dispatch); baked-in #18 (authority-isolation vs confidentiality-isolation).

### External literature anchors

- **Bellare-Keelveedhi-Ristenpart 2013, "DupLESS: Server-Aided Encryption for Deduplicated Storage."** USENIX Security 2013. <https://www.usenix.org/conference/usenixsecurity13/technical-sessions/presentation/bellare>. Eprint: <https://eprint.iacr.org/2013/429>. (Anchor for: message-locked encryption; brute-force attacks against convergent encryption; OPRF-based mitigation. Option E rationale.)
- **Bellare-Keelveedhi-Ristenpart 2013, "Message-Locked Encryption and Secure Deduplication."** EUROCRYPT 2013. (Anchor for: convergent encryption formalization + content-equality-oracle threat.)
- **Wikipedia, "Convergent encryption."** <https://en.wikipedia.org/wiki/Convergent_encryption>. (Anchor for: content-equality-oracle vulnerability statement.)
- **Alwen-Coretti-Jost-Mularczyk 2020, "Continuous Group Key Agreement with Active Security."** TCC 2020. <https://eprint.iacr.org/2020/752>. (Anchor for: CGKA formalization; PCS + FS-across-membership properties.)
- **Alwen-Hartmann-Kiltz-Mularczyk 2023, "Fork-Resilient Continuous Group Key Agreement."** <https://eprint.iacr.org/2023/394>. (Anchor for: FR-CGKA being overkill for Option D's needs per §4.4.)
- **RFC 9180 (Barnes-Bhargavan-Lipp-Wood 2022), "Hybrid Public Key Encryption."** <https://www.rfc-editor.org/rfc/rfc9180>. (Anchor for: HPKE-mode-base; cited in C3 §2; basis for multi-stanza HPKE.)
- **RFC 5869 (Krawczyk-Eronen 2010), "HKDF."** (Anchor for: HKDF-Extract = HMAC; K_Atrium primitive reuses the existing crypto-suite.)
- **NIST SP 800-198 (Krishna 2017), "HMAC Verification."** (Anchor for: HMAC-SHA256 audit-friendly primitive choice.)
- **Bellare 2006, "New Proofs for NMAC and HMAC."** CRYPTO 2006. (Anchor for: HMAC PRF-security reduction; key-recovery resistance bound per §2.1.)
- **Leitão-Pereira-Rodrigues 2007a, "HyParView: A membership protocol for reliable gossip-based broadcast."** DSN 2007. <https://asc.di.fct.unl.pt/~jleitao/pdf/dsn07-leitao.pdf>. (Anchor for: iroh-gossip membership substrate; O(log N) scaling per §3.2.)
- **Leitão-Pereira-Rodrigues 2007b, "Epidemic Broadcast Trees."** SRDS 2007. <https://asc.di.fct.unl.pt/~jleitao/pdf/srds07-leitao.pdf>. (Anchor for: PlumTree per-peer single-delivery via IHAVE/IWANT; gossip-protocol-level dedup per §3.2.)
- **iroh-gossip docs.** <https://docs.iroh.computer/connecting/gossip> — "scales to a few thousand peers" per single topic.
- **Kissner-Song 2005, "Privacy-Preserving Set Operations."** CRYPTO 2005. (Anchor for: PSI; Option F rationale.)
- **Albrecht-Bellare 2024, "Key-Committing Authenticated Encryption."** (Anchor for: C5 NEW-F2 key-committing AEAD; cross-reference per §2.1 plaintext-injectivity gotcha.)
- **MLS WG IETF, RFC 9420 (Barnes-et-al 2023), "The Messaging Layer Security (MLS) Protocol."** (Anchor for: TreeKEM; cross-reference per §4 CGKA-deferral re-evaluation.)

---

**End of Q3 revisit assessment.**
