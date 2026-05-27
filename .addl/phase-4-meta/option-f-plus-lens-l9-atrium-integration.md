# Option F+ envelope-layer-unification §6.2-with-Amendments-1-6 — Lens L9 Atrium-integration P2P-systems review

**Branch:** `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration`
**Reviewer lens:** Senior P2P-systems architect. Distinct from the three prior cryptographer reviewers (pseudo-keypair §6.2 sketch; second-opinion CONCUR-WITH-AMENDMENTS-1+2; third adversarial-design Amendments-3+4+5+6). Three priors all focused on cryptographic envelope properties. **This lens: how does §6.2 compose with Benten's specific P2P architecture** — Atrium peer-mesh, iroh-blobs / sendme delivery, forkability semantics, sometimes-offline engines, ephemeral capability-bound remote execution.

**Authority:** ADVISORY. Final decision rests with Ben. Downstream DISAGREE-WITH-EXPLANATION first-class per `feedback_review_finding_ground_truth_verify`. I treat the three prior reviewers' conclusions (NO-GO on F+ pseudo-keypair, CONCUR on §6.2 design direction, Amendments 1-6 as load-bearing) as ground-truth and attack the **P2P-composition gaps** they explicitly did not work through.

**Inputs ground-truth-verified** (every cite below pinned to the branch+SHA shown; I `git show <branch>:<path>`-verified each):

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — first reviewer's NO-GO + §6.2 sketch (`pub struct EncryptedEnvelope { codepoint, payload, aad_binding }`).
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — second-opinion CONCUR-WITH-AMENDMENTS-1+2 (codepoint-in-AAD; strict-decode).
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — third reviewer Amendments-3+4+5+6 (TLV length-injectivity; sender-DID-in-AAD; replay-window-in-AAD; Bernstein-Persichetti CT-Decap mandate).
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — F-full scope review §6 (remote-permission wire shape); §7 (device-link wire shape); §14.1 (origin Option-F+ pitch).
- `phase-4-meta-core/encrypt-to-recipient-review-p2p-architect @ 8cfb079c` — prior P2P-architect review of the 6 options (Option F structural-split; multi-stanza Atrium fallback; Drop-CID derivation).
- `docs/ARCHITECTURE.md @ origin/phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design` — engine architecture skeleton.
- `2026-05-27` Ben framing: Atrium-as-forkable (member-leaves-keeps-past-content); hyper-scaling = rent-compute-from-peers under ephemeral capability-bound grants.
- iroh-blobs protocol doc (`docs.iroh.computer/protocols/blobs`) — BLAKE3 content-addressing + `BlobTicket` shape + incremental verification, **no encryption at blob layer**.
- Willow `willowprotocol.org/specs/sync/index.html` — confidential sync = private set intersection on namespaces + read-cap gating; **explicitly does not address payload-at-rest encryption**; selective payload delivery (eager-vs-lazy by threshold).
- RFC 9180 §5.1, §5.1.1, §9.1.2, §9.1.4, §9.7.3 — HPKE info-binding, mode-base sender-auth absence, IND-CCA2 properties, mode-base provides **no replay protection** beyond same-stream ordering.

**Stance disclosed up front:** my bias is toward finding substantive P2P-composition disagreements the cryptographer-lens reviewers structurally couldn't see. The three priors agreed §6.2-plus-Amendments is cryptographically sound at the primitive layer; I attack the **architectural composition surfaces** — what happens when these envelopes hit iroh-blobs routing, when an Atrium forks, when a recipient is 6-months-offline, when a rented compute peer executes a workflow under an ephemeral grant.

---

## 0. Reading-order note

5-minute read: §1 (verdict) → §2.1 (the load-bearing finding A1 multi-recipient HpkeBase doesn't exist at v1-beta yet but Drop-bundle composition needs it) → §2.5 (Bob-rotated-his-keys-while-offline finding) → §7 (recommended amendments).

Full read: §§2-6 work through 5 task surfaces in depth; §7 consolidates 5 NEW recommended amendments (A1-A5) plus refinements to existing Amendments 4+5 for the Atrium peer-mesh threat model; §8 self-assessment + lower-confidence areas; §9 citations.

---

## 1. Executive verdict

**CONCUR-WITH-CALIBRATION** on §6.2-with-Amendments-1-6 as the right cryptographic-envelope shape. **DISAGREE that the envelope as currently scoped is composition-complete for Benten's specific P2P architecture without 5 additional Atrium-integration amendments (A1-A5).**

The three priors validated soundness *as a primitive*. The lens this review adds: **§6.2 is currently single-recipient by construction** (`EnvelopePayload::HpkeBase` is one ML-KEM-768-X25519 encap targeting one recipient pubkey). Benten's Drop-bundle and Atrium-peer-mesh use cases are inherently *multi-recipient*. The prior P2P-architect review (Option F structural-split @ 8cfb079c §2 + §6 Risk R-3) named this gap explicitly: "v1-beta uses multi-stanza single-recipient as the explicit group fallback per the age/Saltpack precedent." The §6.2 envelope-layer-unification artifact **does not yet codify the multi-stanza composition rule**, leaving Drop-bundle composition ambiguous.

**5 load-bearing Atrium-integration amendments** I propose, with severity + confidence:

1. **AMENDMENT A1 (load-bearing): multi-recipient stanza composition rule.** Drop bundles addressed to N Atrium members are composed as `EncryptedEnvelope` with one `EnvelopePayload::HpkeMultiBase { stanzas: Vec<{recipient_did, hpke_encap, wrapped_cek}> }` variant (or N parallel single-recipient envelopes sharing the same plaintext CID, with explicit binding rules between them). MUST be codified at v1-beta interface-freeze or §6.2 cannot service the most common Atrium use case. **Severity: HIGH** (current §6.2 doesn't service the central Atrium use case). **Confidence: HIGH**.

2. **AMENDMENT A2 (load-bearing): envelope-CID stability under recipient-set evolution.** Drop-bundle CID derivation MUST exclude the recipient-set HPKE stanzas from the CID (CID derives from plaintext + binding-context). Otherwise adding/removing a recipient mutates the Drop CID, breaking iroh-blobs content-addressing-as-identity. The prior P2P-architect review's F-refinement-2 (Drop-CID = `BLAKE3(plaintext || recipient_did_set || context_label)`) **conflicts with this requirement** and I disagree with that refinement on Atrium-composition grounds — see §3.3. **Severity: HIGH**. **Confidence: HIGH**.

3. **AMENDMENT A3 (load-bearing): recipient-key-rotation interaction with archival envelopes.** §6.2 envelopes have no key-rotation rebinding mechanism. If Bob's HPKE encryption pubkey rotates while a Drop bundle for him sits on an Atrium relay, Bob cannot decrypt on return. MUST specify: (a) recipient key-rotation policy (Bob keeps old decap-key under DAK for grace period; OR sender keeps Drop-bundle un-deletable in their outbox to re-encrypt under new pubkey on rotation discovery; OR a hybrid), (b) the rotation-epoch binding in `BindingContext::DropToRecipient` so a recipient with N historical keys knows which to try. **Severity: HIGH** for the 6-months-offline scenario. **Confidence: HIGH**.

4. **AMENDMENT A4 (load-bearing): K_principal-rotation propagation to per-Node K(N) addressing.** §6.2 Layer-A binds `vault_version: u8`. The current 3-layer spec is `K(N) = KDF(K_principal, N.cid)`. If K_principal rotates (e.g. on user-driven security event; or on Layer-A vault-version bump from Argon2id-param-upgrade), every K(N) recomputes. Engines reading historical Nodes need to know **which K_principal generation** to KDF from. MUST bind a `k_principal_generation: u32` field into per-Node AEAD AAD (Layer-B) and into `BindingContext::Vault` (Layer-A) — and persist the K_principal-rotation-log as an Atrium-replicated structure so other devices can resolve historic ciphertexts. **Severity: MEDIUM-HIGH** (rotation isn't v1-beta-Day-One but freezes wire format). **Confidence: HIGH**.

5. **AMENDMENT A5 (load-bearing): ephemeral-execution-grant binding to remote-engine execution context.** §6.2 `BindingContext::RemotePermission { request_id, operation }` plus Amendment-5's `valid_until_epoch_seconds` is insufficient for the hyper-scaling vision: when User-A rents compute from Peer-P to execute Workflow-W, the grant must bind **the workflow-CID, the input-Node-CID set, the result-encryption-pubkey (where to send results), and the executor's DID** — not just `Decrypt(node_cid)`. Otherwise a malicious or compromised executor can use the grant outside the intended workflow scope. MUST extend `PermissionOperation` with an `ExecuteWorkflow { workflow_cid, input_node_cids, result_recipient_pubkey, executor_did }` variant whose AAD bindings are enforced at envelope-Open time. **Severity: MEDIUM-HIGH** (hyper-scaling is post-v1-beta but the interface freezes now). **Confidence: HIGH**.

**Two MEDIUM observations** (§7.6 + §7.7): (O1) the §6.2 envelope's HPKE-mode-base choice forecloses iroh-blobs-native peer-discovery for HPKE-encryption-pubkey publication (no key-fetch protocol baked in); (O2) the Drop-bundle propagation model interacts with Willow's "selective payload delivery" in ways that need explicit reconciliation if Benten ever adopts Willow as a sync substrate.

**Cumulative verdict:** §6.2-with-Amendments-1-6 is cryptographically load-bearing-final at the primitive layer. §6.2-with-Amendments-1-6-plus-A1-A5 is composition-load-bearing-final at the Benten-P2P-architecture layer. Both sets are necessary. Without A1-A5, §6.2 ships as a single-recipient primitive in an architecture that needs multi-recipient composition — and that gap will surface at the first Drop-bundle-to-3-Atrium-peers integration test.

**Most-substantive single finding:** A1 (multi-recipient stanza composition rule) — it's named obliquely by both prior P2P-architect review (Option F multi-stanza fallback) and third reviewer (§2.7 "multi-stanza HpkeMultiBase variant…residual concern warranting another reviewer") but **never codified inside the §6.2 envelope shape**. This is the kind of "spec gestures at it; wire format doesn't lock it in" failure mode that historically created interop nightmares (cf. age vs Saltpack multi-recipient incompatibility).

---

## 2. Atrium peer-mesh routing composition

### 2.1 Envelope CID stability + iroh-blobs content-addressing

**Question:** the CID of an `EncryptedEnvelope` = BLAKE3 of (codepoint || HPKE-encap || AEAD-ciphertext || aad_binding). Is this stable + addressable through iroh-blobs?

**Finding:** Stable yes, but with a fragility that matters for Atrium composition.

iroh-blobs (`docs.iroh.computer/protocols/blobs`) is *plaintext-CID*: "All blobs within iroh are referred to by the BLAKE3 hash of its content." It performs incremental verification: "the integrity of each chunk is checked both by the sender and the receiver." It has **no encryption at the blob layer**.

This means a §6.2 EncryptedEnvelope serialized to bytes can be stored as an iroh-blob; its iroh-blob-CID will be deterministic + verifiable + chunked-streamable. **Good.**

BUT: the iroh-blob-CID is over the *encrypted bytes*. Two distinct senders encrypting the *same plaintext* to the *same recipient* produce **two different iroh-blob-CIDs** (HPKE encap is randomized). This is correct for HPKE (semantic security requires it) but means iroh-blobs deduplication of identical-plaintext-to-identical-recipient is structurally impossible. Each Drop bundle is a unique blob from iroh-blobs' perspective.

**This is fine for v1-beta** — it's the price of confidentiality. But it has a knock-on consequence for §3 Drop-bundle composition (the Drop bundle's *plaintext CID*, which IS what the user-facing graph references, is necessarily distinct from the *envelope CID* on the wire). The two CIDs must NOT be confused. See A2 below.

**Compose-cleanness verdict for §2.1:** §6.2 envelope bytes compose cleanly with iroh-blobs as raw blob payloads. The cleanness is contingent on Drop bundles separating plaintext-CID from envelope-CID semantics (A2).

### 2.2 sendme ticket-based delivery

**Question:** does sendme ticket-based delivery work with §6.2 envelope format?

**Finding:** Yes, with no envelope-level changes needed.

sendme tickets (`BlobTicket` per iroh-blobs docs) "package the file's BLAKE3 hash and our endpoint's `EndpointId` into a single copy-able string." This is a *transport-layer* identifier: it tells a recipient "here's a hash + here's a node to fetch it from." It is **agnostic to whether the blob bytes are encrypted**.

A §6.2 EncryptedEnvelope CAN be a sendme-shareable blob. The sendme ticket conveys (envelope-CID, sender-endpoint-id). The recipient fetches + verifies the envelope bytes via iroh-blobs incremental-verify; then attempts `EncryptedEnvelope::decode()` + HPKE-Open with their own decap-key.

**Caveat (NEW finding, MINOR, observation O3):** the sendme ticket itself is a *bearer credential* (anyone with the ticket can fetch the blob). For Layer-C drops, this is fine — fetching just gets you ciphertext you can't decrypt without the recipient key. **But** for a poorly-considered Layer-D RemotePermission use case where the ticket is shared on a side channel, this means an interceptor can fetch the envelope (still encrypted to the legit recipient, so HPKE-Open fails — fine) but ALSO learns *that envelope exists* (metadata leak) and *which endpoint hosts it* (peer-discovery side channel). This is L6-privacy-lens territory; flagging for awareness, not a §6.2 amendment.

**Compose-cleanness verdict for §2.2:** sendme tickets are fully compatible. No envelope-level change needed.

### 2.3 Atrium peer-discovery for HPKE encryption pubkeys (NEW finding, observation O1)

**Question:** how does Alice learn Bob's HPKE encryption pubkey to seal a Drop bundle to him?

**Finding:** §6.2 envelope is silent on this; it's a substrate question — **but the silence is itself a gap** because the answer determines whether Layer-C drops actually work in the field.

Three candidate answers, with composition consequences:

(a) **Via Atrium membership-attestation Node** — when Bob joins Alice's Atrium, his Node (the AtriumMembership CBOR per ffull-scope-review §7.2) carries `device_encryption_pubkey: HybridKemPubKey`. Alice reads Bob's pubkey from her own graph. **Pro:** content-addressed, signed, replicates via Atrium sync. **Con:** what about non-Atrium-co-member recipients? (Drops to a stranger Bob whom Alice knows only by DID — a CLAUDE.md baked-in #17 valid use case.)

(b) **Via DID resolution** — Bob's DID-document publishes his HPKE encryption pubkey via the DID method (e.g. `did:key:z6...` direct encoding for X25519; harder for hybrid ML-KEM-768-X25519 — needs a multikey-formatted DID method that supports the X-Wing combiner output). **Pro:** works for non-Atrium-co-member recipients. **Con:** locks Benten into a DID-method choice that supports hybrid PQ-KEM encoding; W3C DID core doesn't define this; the prior `phase-4-meta-core/comment-opps-multiformats-w3c-did` scan documents the multicodec assignment gap.

(c) **Via iroh-gossip publication** — Bob publishes his encryption pubkey on an iroh-gossip topic; Alice subscribes. **Pro:** real-time freshness. **Con:** liveness coupling (subscribe-window vs send-time race); iroh-gossip is not authentication-providing on its own; needs (a) or (b) as fallback.

**Recommended composition:** (a) for Atrium-internal Drops (the common case); (b) for cross-Atrium Drops (the hyper-scaling case), with (c) as an optimization for online-now-fresh-key-discovery. None of these require a §6.2 envelope change, but **the Phase-4-Meta-Core interface freeze should call out the peer-discovery substrate as a separate-but-coupled work item** so Layer-C drops aren't shipped without a way to find recipient keys.

**Compose-cleanness verdict for §2.3:** §6.2 envelope is composition-neutral on peer-discovery. **Recommend documentation** (not amendment) naming the discovery substrate as separate work — observation O1 in §7.6.

### 2.4 Relay-archive scenario: Bob offline 6 months, sk rotated in interim (NEW finding, AMENDMENT A3)

**Question:** if an Atrium relay holds an envelope for Bob who's offline for 6 months, does Bob's eventual Open succeed? What if Bob's sk rotated in interim?

**Finding:** **Currently UNDEFINED in §6.2 + Amendments 1-6.** This is one of the load-bearing gaps motivating A3.

Walk-through:
1. Alice seals a Drop bundle to Bob at time T₀ using Bob's current `device_encryption_pubkey_v1` (call it `pk_B,1`).
2. The envelope sits on an iroh-blobs relay (untrusted per CLAUDE.md baked-in #18 trust model).
3. Bob's device is offline T₀ to T₀+6 months.
4. During the offline period, Bob's device rotates its HPKE encryption key (e.g. user-driven security event; periodic rotation policy; device-key-replacement). Bob's local `device_encryption_sk_v1` may or may not be retained.
5. Bob comes back online at T₀+6 months; the relay still holds the envelope; Bob attempts `EncryptedEnvelope::decode()` + HPKE-Open.

**Two sub-cases:**

**Sub-case 5a:** Bob retained `sk_B,1` under DAK after rotating to `pk_B,2` — Open succeeds. But §6.2 doesn't TELL Bob which historical sk to try; he must brute-force-try all his retained generations. With N rotations Bob has N candidate sks; envelope has no generation hint; Bob's open-cost grows linearly with rotation history. Also: the codepoint-substitution-resistance from Amendment 1 (codepoint-in-AAD) only catches *cross-codepoint* substitution, not *cross-rotation-generation* substitution within the same codepoint. (Recall Amendment-4's third-reviewer concern about "AAD-belief drift" — the recipient's confusion about which sk to try is the dual.)

**Sub-case 5b:** Bob did NOT retain `sk_B,1` (rotation discarded the old key for forward-secrecy reasons) — Open fails forever. The Drop bundle is permanently lost despite the relay faithfully retaining it. This contradicts Compromise #31's "forever-valid Drop bundles" property.

**Mitigation — AMENDMENT A3 (load-bearing):** extend `BindingContext::DropToRecipient` with a `recipient_key_generation: u32` field:

```rust
DropToRecipient {
    audience_did: Did,
    sender_did: Did,                          // Amendment 4
    recipient_key_generation: u32,            // Amendment A3 (NEW)
}
```

AND specify a recipient-key-retention policy: **devices MUST retain old HPKE decap-keys under DAK for a documented grace window** (e.g. 1 year from rotation; configurable; key-retention is encrypted-at-rest via Layer-A vault). Devices MAY discard pre-grace-window keys; envelopes sealed to those generations are permanently unrecoverable; Compromise #31 must be amended to clarify "forever-valid" means "forever-valid within recipient's key-retention window."

The generation field lets a recipient with N historical sks dispatch directly to the right one (O(1) instead of O(N)) AND lets the recipient detect "envelope sealed to a generation I no longer have" as an EXPLICIT-FAILURE-MODE rather than a silent "Open returns garbage."

**Alternative considered + rejected:** "sender retains Drop bundles in their outbox until acked + re-encrypts under rotated recipient pubkey on key-rotation discovery." Rejected because: (a) requires sender liveness which is exactly what Drop bundles abstract away; (b) violates Compromise #31 forever-valid semantics from the sender side; (c) introduces a re-encryption attack surface (sender's outbox is a confidentiality risk).

**Citation chain.** Signal Protocol's key-rotation handling (signal.org "session-state management") + Matrix Olm's session-key-rotation + age's key-rotation discipline ("the recipient must retain old identities to decrypt old files") — all converged on "recipient retains historical decap keys, optionally with a retention window." MLS RFC 9420 §16 (epoch + group-state management) makes generation explicit. Benten is structurally in this same design space and should adopt the precedent.

**Confidence on AMENDMENT A3 as load-bearing:** **HIGH**. The 6-months-offline scenario is the central Compromise #31 use case; without A3, Compromise #31 is unimplementable under realistic recipient-key-rotation policies.

### 2.5 Multi-recipient Drop bundles (NEW finding, AMENDMENT A1)

**Question:** how does §6.2's single-recipient HpkeBase compose with multi-recipient delivery (per the F-full scope review's age/Saltpack multi-stanza pattern)?

**Finding:** **§6.2 currently has NO multi-recipient variant.** The first-reviewer §6.2 sketch shows `EnvelopePayload::HpkeBase` as one HPKE encap; multi-recipient is hand-waved as "future" in both prior cryptographer reviews. The third reviewer explicitly named this gap (§2.7: "multi-stanza HPKE recipient-confusion (cross-stanza ciphertext-substitution) — I found a plausible attack class but couldn't construct an end-to-end exploit without more wire-format detail than §6.2 currently specifies"). It is **the load-bearing P2P-composition gap**.

The Atrium use case is intrinsically multi-recipient. When Alice writes a new Node to an Atrium with N members (typical N = 2-10 for a personal-mesh, 10-100 for a team-mesh), that Node's confidentiality envelope MUST be openable by all N members. The age/Saltpack multi-stanza pattern (age.computer multi-recipient encoding; Saltpack encrypted-payload-key + per-recipient-wrap) is the well-trodden shape:

- One symmetric content-encryption-key (CEK) is randomly generated.
- The plaintext is AEAD-sealed once with CEK.
- The CEK is wrapped N times, once per recipient, via N HPKE-mode-base encaps to each recipient's pubkey.
- The envelope contains 1 AEAD ciphertext + N HPKE stanzas.

**Mitigation — AMENDMENT A1 (load-bearing): codify multi-stanza as an EnvelopePayload variant inside §6.2 NOW.**

```rust
pub enum EnvelopePayload {
    SymmetricAead { /* Layer-A vault */ },
    HpkeBase { /* single-recipient Layer-C/D */ },
    HpkeMultiBase {                                       // Amendment A1 (NEW)
        cek_aead_ciphertext: Vec<u8>,                     // ChaCha20-Poly1305 over plaintext with random CEK
        cek_aead_nonce: [u8; 12],
        stanzas: Vec<HpkeRecipientStanza>,                // one per recipient
    },
}

pub struct HpkeRecipientStanza {
    recipient_did: Did,                                   // who this stanza wraps for
    recipient_key_generation: u32,                        // Amendment A3 (NEW)
    hpke_encap: Vec<u8>,                                  // ML-KEM-768-X25519 encap
    wrapped_cek: [u8; 32 + 16],                           // HPKE-AEAD-sealed CEK with poly1305 tag
}
```

**Codepoint allocation:** mint `LAYER_C_DROP_MULTI_RECIPIENT = 0x6301` (BE-pinned per Amendment 5 of the second-opinion review) distinct from `LAYER_C_DROP_SINGLE = 0x6300`. Amendment 2's strict-decode dispatches to the correct variant.

**Cross-stanza substitution defense (closes third reviewer §2.7 residual concern):** the canonical AAD/info passed to each stanza's HPKE-Seal MUST bind:
- The codepoint (Amendment 1)
- The CEK-AEAD-ciphertext's CID (BLAKE3 of `cek_aead_ciphertext || cek_aead_nonce`) — this binds the stanza to a specific ciphertext-body so swapping stanzas across envelopes is detectable
- The full sorted recipient_did list (so a recipient knows the complete addressee set was the sender's intent — defense against silent-recipient-removal attacks)
- The sender_did (Amendment 4 extension)

This makes "Eve substitutes Bob's stanza into Carol's envelope to confuse Carol" structurally infeasible.

**Costs:** O(N) HPKE encaps at seal-time (linear in recipient count); O(1) HPKE-Open at recipient-time (recipient finds their own stanza by recipient_did + recipient_key_generation match, decap their CEK, AEAD-opens the body). This matches age/Saltpack precedent + matches the prior P2P-architect review's "multi-stanza fallback" Option-F structural-split.

**What this does NOT enable:** efficient *removed-Atrium-member* forward-secrecy. Per the prior P2P-architect review §6 R-3, removed members still hold the stanza-wrap-keys for past Drops; that's the documented limitation Layer-3b MLS-PQ-derived CGKA will resolve. **This is the forkability semantic Ben already ratified** (2026-05-27: "Atrium-as-forkable, member-leaves-keeps-past-content"). A1 doesn't close that gap; it codifies the v1-beta-shippable shape that respects the forkability semantic.

**Citation chain.** age multi-recipient encoding (`age-encryption.org/v1`); Saltpack encrypted-payload-key spec (saltpack.org/encryption-format-v2); RFC 9180 §5 (HPKE single-shot API per-recipient); the prior P2P-architect review §2.F + §6 R-3 + §5.2 ("Multi-stanza fallback test"); third reviewer §2.7 ("residual concern warranting another reviewer").

**Confidence on AMENDMENT A1 as load-bearing:** **HIGH**. Without A1, §6.2 cannot service the central Atrium-multi-recipient use case; the gap WILL surface at first integration test; codifying now (interface-freeze) is materially cheaper than retrofitting later.

---

## 3. Drop-bundle composition

### 3.1 Envelope nesting inside Drop bundle (NEW finding, observation)

**Question:** Drop bundle = "full S&C composition in CBOR-on-disk" per RATIFIED-sharing-and-confidentiality-2026-05-21. How does §6.2's envelope compose into a Drop bundle? Outer Drop-bundle envelope + inner per-recipient envelopes? Or inline?

**Finding:** **Recommended composition: ONE outer EncryptedEnvelope with HpkeMultiBase payload (per A1), containing a CBOR-serialized DropBundlePayload as the plaintext** — NOT nested envelopes.

Walk-through:
- A Drop bundle conceptually carries: (a) one or more Node CIDs + their AEAD-encrypted bodies, (b) UCAN scope assertions, (c) SubgraphSpec CIDs the recipient is granted access to, (d) sender attestation.
- The straightforward composition: serialize `(a)+(b)+(c)+(d)` into a DropBundlePayload CBOR structure; this is the *plaintext* the multi-stanza A1 envelope seals.
- Each Node body is *already* encrypted at Layer-B with `K(N) = KDF(K_principal, N.cid)`. The Drop bundle includes the **per-Node K(N) keys** wrapped under the multi-stanza CEK so recipients can derive K(N) and AEAD-Open the Layer-B ciphertexts.

**Why not nested envelopes:** nesting would mean (Drop-bundle-envelope[wrap a list of per-recipient-Drop-envelopes]). This doubles HPKE work (2N encaps for N recipients), creates an outer envelope whose plaintext is itself a list of ciphertexts (information-theoretic redundancy), and requires defining envelope-of-envelopes semantics in §6.2. The flat composition (one outer multi-stanza envelope; plaintext is the DropBundlePayload) is strictly cheaper + cleaner.

**Compose-cleanness verdict for §3.1:** flat composition works cleanly given A1. No additional amendment needed beyond A1.

### 3.2 UCAN scopes inside Drop bundle (NEW finding, MEDIUM)

**Question:** UCAN scopes inside Drop bundle: how do they compose with envelope's `BindingContext::RemotePermission`?

**Finding:** They are STRUCTURALLY DIFFERENT primitives that **shouldn't be conflated** — but the §6.2 sketch risks future maintainers conflating them.

- UCAN scopes inside a Drop bundle = **content-grants** ("you, recipient, can access subgraph X, action Y, until time Z"). They sit in the DropBundlePayload plaintext (after A1's HpkeMultiBase opens). They are signature-authenticated by sender (sender's user-DID-key signs the UCAN chain).
- `BindingContext::RemotePermission` = **execution-grants** ("approving device A authorizes requesting device B to perform operation X on behalf of user-DID"). It's the AAD binding for a §6.2 envelope whose payload is the ephemeral key material or signed UCAN delegation.

These are different *layers*:
- Drop UCAN scope = content-layer authorization sitting INSIDE an A1 envelope's plaintext
- RemotePermission BindingContext = execution-layer authorization in an A1 envelope's AAD

**Recommendation (not a §6.2 amendment, but a documentation requirement):** the Phase-4-Meta-Core interface-freeze docs MUST explicitly distinguish "content-grant UCAN (inside Drop bundle plaintext, sender-signed)" from "execution-grant Permission (in §6.2 RemotePermission BindingContext)" so future maintainers don't confuse them. A pim-N-style rule would help: any new `pub` API that takes both a `DropBundle` and a `PermissionGrant` MUST route through a wrapper that disambiguates.

**Compose-cleanness verdict for §3.2:** structurally orthogonal; works cleanly; documentation gap is the risk.

### 3.3 Drop bundle CID stability (NEW finding, AMENDMENT A2, conflicts with prior P2P-architect review F-refinement-2)

**Question:** Drop bundle CID: stable across recipient-key-rotation? Across cipher-suite-rotation? Per-recipient or per-payload?

**Finding:** **Drop bundle MUST have TWO distinct CIDs**, and §6.2 + the Drop-bundle composition spec MUST not conflate them. This is AMENDMENT A2.

The two CIDs:

(i) **plaintext CID** = `BLAKE3(canonical(DropBundlePayload))` over the post-decryption plaintext. This is the CID the user's graph references; the CID that survives recipient-set evolution, cipher-suite rotation, and re-encryption. This CID is **inside the encrypted envelope** (a recipient who opens the envelope learns the plaintext CID; observers of the encrypted blob cannot).

(ii) **envelope-blob CID** = `BLAKE3(serialized_EncryptedEnvelope_bytes)` = iroh-blob CID. This is the CID for iroh-blobs transport; it changes with every reseal (different HPKE encap randomness; different recipient set; different cipher suite); it's the addressing handle for relay-fetch.

**Why this matters:**
- The **plaintext CID** is what the user's content-addressed graph references ("Node X depends on Drop bundle plaintext CID Y"). It MUST be stable across reseals.
- The **envelope-blob CID** is what sendme tickets carry ("fetch blob Z from endpoint E"). It MUST change on every reseal to maintain semantic-security.

**Conflict with prior P2P-architect review F-refinement-2:** That review proposed `Drop-CID = BLAKE3(plaintext || recipient_did_set || context_label)` — binding the recipient set INTO the Drop CID. **I disagree on Atrium-composition grounds.** Reasoning:

1. If recipient_did_set is part of the CID, adding a new Atrium member retroactively to past Drops mutates their CIDs — breaks all graph references to those Drops.
2. The "Drop CID identifies what's in the Drop" semantic and the "Drop CID identifies who's authorized" semantic are different. The first should be stable (it's an identity/content claim); the second is dynamic (it's an authorization claim that can evolve as the Atrium evolves).
3. The forkability semantic (Atrium fork at time T; pre-fork members keep pre-fork content) requires that pre-fork Drop CIDs remain valid identifiers even after the Atrium evolves. Binding recipient_set into Drop CID makes pre-fork Drop CIDs un-referenceable post-fork.

The prior P2P-architect review's intent (defense against recipient-substitution attacks) is **better served by binding the recipient_did_set into the per-stanza AAD** (per A1 cross-stanza substitution defense), NOT into the Drop CID.

**Mitigation — AMENDMENT A2:** the Phase-4-Meta-Core spec MUST define:
- `DropBundle::plaintext_cid()` = BLAKE3 over canonicalized DropBundlePayload bytes (independent of envelope encoding)
- `DropBundle::envelope_blob_cid()` = BLAKE3 over the §6.2 EncryptedEnvelope serialized bytes (changes on reseal)
- The user's graph stores `plaintext_cid` references; the iroh-blobs transport uses `envelope_blob_cid`
- A mapping from `plaintext_cid` to `Vec<envelope_blob_cid>` (one plaintext, multiple resealings for different recipient subsets) replicates via Atrium

**Confidence on AMENDMENT A2 as load-bearing:** **HIGH**. Without A2, Drop bundles cannot survive recipient-set evolution + Atrium forks while preserving graph references.

### 3.4 SubgraphSpec.cid propagation through Drop bundle (NEW finding)

**Question:** SubgraphSpec.cid as selector in UCAN scope: how does it survive Drop bundle propagation?

**Finding:** Survives cleanly because SubgraphSpec.cid is itself content-addressed + immutable; **but** the recipient must be able to **resolve** SubgraphSpec.cid to a SubgraphSpec the recipient can interpret.

Walk-through:
- Alice's Drop bundle to Bob: "you may access SubgraphSpec.cid = `bafy...`, action: read, valid_until: T+30d." (UCAN scope inside the Drop bundle plaintext.)
- Bob receives + opens the Drop bundle (per A1). Bob now sees a SubgraphSpec.cid reference.
- Bob needs to actually fetch the SubgraphSpec definition (which is itself a Node in Alice's graph) to know which Nodes the scope grants access to.

**Composition concern:** if SubgraphSpec is itself an encrypted Node (Layer-B AEAD with K(N) = KDF(K_principal_A, SubgraphSpec.cid)), Bob can't read it without the K(N) key. Solutions:

(a) Alice includes the SubgraphSpec Node + its K(N) in the Drop bundle plaintext (per-Node-K-wrap pattern).
(b) SubgraphSpec is published as Layer-C-encrypted to Bob separately.
(c) SubgraphSpec definitions live in a globally-readable Atrium subgraph; only the contents they reference are encrypted.

The choice between (a)/(b)/(c) is a S&C-architecture design decision — but it's NOT a §6.2 envelope-shape concern. §6.2 composes cleanly with any of the three. **Recommendation: name this as a Phase-4-Meta-Composing scope item** so the S&C wave decides; §6.2 doesn't need an amendment.

**Compose-cleanness verdict for §3.4:** §6.2 composes cleanly; downstream S&C architecture decides resolution mechanism.

---

## 4. Forkability composition

### 4.1 Atrium fork mechanics + existing Drop bundles for excluded members (NEW finding)

**Question:** Alice forks her workspace at time T_fork; post-fork content goes to fork-A only (excluding Bob who was in pre-fork). How do existing Drop bundles for Bob compose?

**Finding:** Composes cleanly given A1 multi-stanza + A2 dual-CID + Ben's 2026-05-27 ratified forkability semantic.

Walk-through:
- Pre-fork: Alice's Drops include Bob in recipient_did_set (per-Drop stanza for Bob).
- Fork at T_fork: Alice's engine starts a new Atrium membership-set excluding Bob.
- Post-fork: Alice's new Drops include only fork-A members in recipient_did_set; NO stanza for Bob.

Bob's access:
- **Pre-fork content:** Bob still holds his stanza decap-keys for pre-fork Drops; Bob can still Open them. This is the forkability semantic Ben ratified ("member-leaves-keeps-past-content"). §6.2 doesn't need anything special for this — it's a direct consequence of HPKE-mode-base's lack-of-forward-secrecy on the recipient-pubkey side (RFC 9180 §9.1.4).
- **Post-fork content:** Bob can't decrypt because he's not in any post-fork stanza. The codepoint dispatching is unchanged (still LAYER_C_DROP_MULTI_RECIPIENT); Bob's parser attempts to find his stanza in the stanzas list, fails to find his recipient_did, returns "not addressed to me." Clean.

**Subtle interaction with A2 plaintext-CID stability:** if Alice writes a Node N referencing a pre-fork Drop bundle plaintext_cid, AND post-fork wants the SAME plaintext-CID accessible to fork-A members who weren't in the pre-fork recipient_set, she **reseals** the Drop bundle as a NEW EncryptedEnvelope with new recipient_did_set (= post-fork members). The plaintext_cid stays stable; the envelope_blob_cid changes. The mapping plaintext_cid → Vec<envelope_blob_cid> (per A2) grows by one entry. Clean.

**Compose-cleanness verdict for §4.1:** forkability composes cleanly given A1 + A2. No additional amendment needed.

### 4.2 Key-material rotation on fork (NEW finding, AMENDMENT A4)

**Question:** does K_principal rotate on fork? Per-Node K(N) for new content? Layer-A vault unchanged?

**Finding:** **K_principal rotation on fork is OPTIONAL but the §6.2 envelope MUST support it.** This motivates AMENDMENT A4.

Ratified Ben framing: K_principal is *identity-equivalent*, not session-ephemeral. Forking doesn't necessarily rotate K_principal — fork is an authorization-set change, not an identity change. But: a security-driven event (Bob compromised; Alice wants to ensure Bob can't decrypt FUTURE post-fork Layer-B Nodes even if Bob obtains them) requires K_principal rotation.

The current spec `K(N) = KDF(K_principal, N.cid)` means a K_principal rotation breaks ALL historical K(N) derivations. Without explicit generation tracking:
- A reading device sees a Layer-B AEAD ciphertext + Node CID; it tries `K(N) = KDF(K_principal_current, N.cid)` — fails for pre-rotation Nodes.
- The device has NO HINT which K_principal generation to use.
- Even with K_principal-history under DAK, brute-force-try-all-generations grows linearly + may silently succeed on a collision (Layer-B AEAD has a 128-bit tag, so brute-force-collision probability per attempt is 2^-128 — safe — but distinguishing "wrong generation" from "tampered ciphertext" requires the auth-tag to be the only signal, which conflates failure modes).

**Mitigation — AMENDMENT A4 (load-bearing):**

1. **Bind `k_principal_generation: u32` into Layer-B per-Node AEAD AAD.** The AAD becomes: `(codepoint || node_cid || k_principal_generation)`. This makes "which generation should I KDF from?" structurally answerable at decode-time.

2. **Bind `k_principal_generation: u32` into `BindingContext::Vault`** so vault-wrapped K_principal bundles can be versioned + co-replicated.

3. **Atrium-replicated K_principal-rotation-log.** Each rotation event = a signed Node in Alice's graph: `KPrincipalRotation { generation: u32, rotated_at: u64, rotated_by_device_did, reason: enum }`. This log replicates via Atrium so all of Alice's devices know the rotation history + can derive K_principal_N from sealed-vault for any historical N.

4. **Layer-A vault stores K_principal generations as a map `HashMap<u32, [u8; 32]>`** under DAK. On rotation, append the new generation; don't drop old ones (subject to A3-style retention policy).

**Consequence:** with A4, the engine reading any historical Layer-B ciphertext:
1. Reads `k_principal_generation` from the AAD.
2. Looks up `K_principal_gen` from sealed vault map.
3. Derives `K(N) = KDF(K_principal_gen, N.cid)`.
4. AEAD-Opens. Success or fail-closed.

No brute-force; explicit failure modes; rotation-policy-tunable retention.

**Confidence on AMENDMENT A4 as load-bearing:** **HIGH**. Without A4, K_principal rotation is structurally un-implementable while preserving readability of historical Nodes; that constrains all of Benten's future security-incident-response.

---

## 5. Offline-first + eventual consistency

### 5.1 100-Drop-bundle catch-up after 6-months-offline (NEW finding)

**Question:** User-A offline 6 months; User-B sent 100 Drop bundles. User-A returns; how does §6.2 compose with the eventual-consistency CRDT? Order of envelope-Open? Conflict resolution if multiple envelopes target same Node-CID?

**Finding:** Composes cleanly with eventual-consistency CRDT semantics with NO §6.2 changes; **but** the engine-level orchestration of "Open 100 envelopes + apply 100 CRDT operations" needs care.

Walk-through:
- User-A returns online; Atrium sync delivers 100 envelopes (with help from A1 multi-stanza if they're multi-recipient).
- User-A's engine MUST: (a) decode + Open each envelope (independent operations; can parallelize); (b) extract the inner DropBundlePayload; (c) apply each Drop to the local CRDT.

**§6.2 + Amendments composition:**
- Amendment 5's `valid_until_epoch_seconds` MATTERS HERE. If a Drop's outer Layer-C envelope had `valid_until` of T_send + 1 day (a default), Open at T_send + 6 months will REJECT per Amendment 5's decoder-MUST-refuse-stale rule. **This is wrong for Drop bundles** — Drops are intentionally long-lived per Compromise #31.
- **NEW finding:** Amendment 5's stale-rejection MUST be variant-specific. The third reviewer's Amendment 5 already excludes `Vault` and `DropToRecipient` from time-bound binding ("Vault and DropToRecipient are out-of-scope for time-bound (vault is at-rest by design; drops are intentionally long-lived per Compromise #31)"). **Confirmed: this exclusion is correct for the 6-months-offline composition.** Re-affirming Amendment 5's variant scoping is right.

**CRDT conflict resolution:**
- Multiple Drops targeting the same Node-CID = standard CRDT merge case (last-write-wins via vector clock or version-vector). §6.2 envelope is transparent to this — it carries the payload; CRDT merge is downstream of envelope-Open.
- **NEW concern:** what if envelopes contain Drops with conflicting authorization claims (Drop-A says "Bob has read access until T_1"; Drop-B says "Bob has read access until T_2")? UCAN scope semantics make these additive (Bob holds both grants; effective access is the union). The CRDT layer must respect UCAN union-of-grants semantics. **Not a §6.2 concern**; flagging for downstream UCAN+CRDT-interaction wave.

**Compose-cleanness verdict for §5.1:** §6.2 + Amendments compose cleanly. Amendment 5's variant-specific time-binding (Vault + DropToRecipient excluded) is correct AS WRITTEN. Re-affirm.

### 5.2 K_principal rotation interaction (already covered by A4 §4.2)

The §5 task description's question about "K_principal rotation + K(N) derivation + version-tagging" is the exact concern A4 addresses. K_principal rotation requires generation-tagged AAD per A4; without it, post-rotation engines cannot read pre-rotation Layer-B ciphertexts.

---

## 6. Ephemeral-capability-bound remote execution

### 6.1 Time-bound capability composing with workflow-execution duration (NEW finding)

**Question:** capability expires after grant validity; Layer-D RemotePermission `valid_until` is bound to AAD (Amendment 5); does this compose with workflow-execution duration?

**Finding:** Composes if + only if `valid_until` is set with realistic margin for executor scheduling + execution time.

Walk-through:
- User-A grants Peer-P permission to execute Workflow-W with `valid_until = T_grant + ΔT`.
- Peer-P may not start execution immediately (queue, scheduling, peer-availability). Latency Δ_schedule.
- Execution runtime Δ_exec.
- Result encryption + return to User-A: Δ_return.

If `valid_until` is too short (say 5 minutes) + a Δ_schedule of 10 minutes happens, execution fails — grant expired before use. If too long (24 hours) + Peer-P is compromised mid-window, grant is exploitable for the full window.

**Subtle composition issue:** Amendment 5's enforcement is at envelope-Open-time. If Peer-P opens the grant envelope at T_grant + 1 minute (passes), then HOLDS the decrypted ephemeral key material for hours before executing, the decrypted secrets exist outside the envelope's time-bound. **Amendment 5 doesn't enforce key-material lifecycle inside Peer-P's process**, only envelope-decode-time.

This is **structurally a Peer-P-runtime-discipline question**, not a §6.2 envelope question. But the §6.2 envelope can help by carrying execution-context bindings that constrain what Peer-P does with the decrypted material — see A5 below.

**Compose-cleanness verdict for §6.1:** Amendment 5 composes with workflow-execution duration **if** valid_until is set with operational margin. Peer-P-runtime discipline is downstream. Recommend documentation: "valid_until SHOULD include 2x expected scheduling+execution latency; users SHOULD revoke ephemeral grants on completion via Atrium-replicated revocation Node."

### 6.2 Operation-bound scope (NEW finding, AMENDMENT A5)

**Question:** capability scope is narrow (decrypt-Node-X-only / sign-UCAN-on-behalf / etc.); how does §6.2 envelope enforce this beyond the AAD-bound BindingContext?

**Finding:** **Currently UNDER-SPECIFIED.** The §6.4 ffull-scope-review's `PermissionOperation` enum is `Decrypt(node_cid) | SignUcanDelegation(scope, audience, expires_at) | RemoteUnlock`. This is the right primitive set for single-shot operations (decrypt one Node; sign one UCAN; unlock once). It's INSUFFICIENT for the hyper-scaling vision of "rent compute to execute Workflow-W."

A "rent compute" grant should bind:
- The workflow CID (executor MUST only execute this specific workflow, not arbitrary code)
- The input Node CIDs the executor may decrypt
- The result-encryption-pubkey (where executor sends results — typically back to User-A)
- The executor's DID (so the grant is non-transferable to other peers)
- The maximum number of input-Node decrypts (so the grant can't be used to bulk-exfiltrate)

**Mitigation — AMENDMENT A5 (load-bearing):** extend `PermissionOperation`:

```rust
pub enum PermissionOperation {
    Decrypt(Cid),                                         // existing
    SignUcanDelegation { scope, audience, expires_at },   // existing
    RemoteUnlock,                                         // existing
    ExecuteWorkflow {                                     // Amendment A5 (NEW)
        workflow_cid: Cid,
        input_node_cids: Vec<Cid>,                        // exhaustive list; executor may not decrypt others
        max_decrypt_count: u32,                           // upper bound; per-Node tracked
        result_recipient_pubkey: HybridKemPubKey,         // where results encrypt to
        executor_did: Did,                                // non-transferable to other peers
    },
}
```

The third-reviewer's Amendment 4 (`granting_user_did` + `requesting_device_did` in RemotePermission BindingContext) provides the OUTER grant-issuance authorization (who granted to whom). A5 provides the INNER operation-scope (what the grantee may actually do). The two compose.

**Critical enforcement requirement:** the executor's engine MUST, before performing each Node-decrypt, verify (a) the Node CID is in the grant's `input_node_cids` list, (b) the running decrypt count is under `max_decrypt_count`, (c) the workflow currently executing is `workflow_cid`. If any check fails, fail-closed.

This is **runtime-enforcement-not-envelope-cryptography** — but A5 makes the SPEC of what to enforce part of the envelope binding, so an executor that cuts corners on enforcement creates an evidence trail (the grant envelope's AAD-bound scope is auditable post-hoc).

**Confidence on AMENDMENT A5 as load-bearing:** **HIGH** for the hyper-scaling vision; **MEDIUM** for v1-beta (hyper-scaling is post-v1-beta; but the interface freezes now). Argument for HIGH inclusion-at-v1-beta: A5's `ExecuteWorkflow` variant is additive; codepoint-table dispatch fail-closes on unknown variants per Amendment 2; reserving the variant shape NOW prevents wire-format break later.

### 6.3 Result-encryption back to user (NEW finding, partially closed by A5)

**Question:** remote engine encrypts results back to user; how does it know user's HPKE pubkey?

**Finding:** **A5's `result_recipient_pubkey` closes this.** The grant envelope SENT BY USER-A to PEER-P carries User-A's HPKE encryption pubkey as part of the grant. Peer-P encrypts results via §6.2 Layer-C single-recipient envelope to that pubkey.

Subtle: User-A's pubkey may rotate between grant-issue and result-return. Recommendation: the grant carries a **specific generation** of User-A's pubkey (matching A3's `recipient_key_generation`); result envelope binds same generation; User-A retains the corresponding sk under DAK (A3 grace policy).

**Compose-cleanness verdict for §6.3:** closed by A5 + A3 composition.

### 6.4 Audit-trail of remote execution (NEW finding)

**Question:** each remote-permission-grant signs and logs; does the envelope shape preserve this?

**Finding:** Yes, with caveat about WHERE the audit trail lives.

The ffull-scope-review §6.4 specifies the PermissionGrant carries `audit_node_cid: Cid` pointing to a PermissionGrant Node Alice wrote to her own graph. §6.2 envelope is transparent to this — the audit_node_cid sits in the envelope's payload.

**NEW concern:** if Alice's audit Node is encrypted at Layer-B (`K(N) = KDF(K_principal_A, N.cid)`), Peer-P can't read it for verification purposes. This is FINE — Peer-P shouldn't need to verify Alice's audit log; Alice's own future-audit verifies it. Peer-P just needs `audit_node_cid` as a citation for "Alice claimed to have audit-logged this; if it ever matters, Alice can prove it."

For Peer-P's OWN audit (Peer-P wants to log "I executed Workflow-W for Alice on date D"), Peer-P writes its own audit Node in its own graph. Alice cannot read Peer-P's audit log (it's encrypted to Peer-P's K_principal). If dispute arises, both audit logs are independently signed + content-addressed + can be cross-correlated by `request_id` (Amendment 4-style).

**Compose-cleanness verdict for §6.4:** §6.2 preserves audit-trail composition cleanly. No amendment needed.

---

## 6.5 Senary task surfaces — anything else a P2P-systems architect surfaces

### 6.5.1 iroh-gossip composition (observation O4)

iroh-gossip is a pub-sub primitive. Benten might use it for:
- Recipient-pubkey freshness publication (per §2.3 option (c))
- K_principal-rotation-event broadcast (per A4)
- Drop-bundle-available notifications (per Drop bundle composition §3)

§6.2 envelope is composition-neutral to iroh-gossip — gossip is a transport, not a content layer. **No amendment needed.** Flag: if Benten adopts iroh-gossip for any of the above, the gossip-message format becomes a parallel wire-format-freeze concern.

### 6.5.2 iroh-docs composition (observation O5)

iroh-docs (key-value document sync) was historically the precedent for what Willow now formalizes. Benten's chosen path is custom-Atrium-sync (not iroh-docs), per the prior architectural decisions. §6.2 envelope doesn't interact with iroh-docs. Flag for documentation only.

### 6.5.3 willow_rs (parked) composition (observation O6)

Willow's confidential-sync model (`willowprotocol.org/specs/sync/index.html`) is **explicit that it doesn't address payload-at-rest encryption** — it covers protocol-level set-reconciliation under access-control. Benten's §6.2 envelope sits at a different layer (content-encryption-at-rest + in-transit). If Benten ever adopts Willow as a sync substrate, §6.2's per-payload encryption sits cleanly atop Willow's selective-payload-delivery (Willow decides WHICH payloads to ship; §6.2 envelope is the format of each).

**One sharp finding:** Willow's "selective payload delivery" (eager < threshold; lazy on request) is **incompatible with the Compromise #31 forever-valid Drop semantic IF Benten relies on Willow to fan-out Drops**. If Willow decides a Drop is "lazy" (above eager-threshold) and a recipient never explicitly requests it, the Drop never propagates. Compromise #31 requires Drops to be eventually-deliverable; Willow doesn't guarantee that.

**Recommendation:** if Benten adopts Willow, Drop bundles MUST be classified as "eager" (force-deliver to all addressed recipients) or supplemented by an out-of-band poll mechanism. Currently this is moot (Willow is parked) but flagging for future-wave context.

### 6.5.4 Atrium-as-Subgraph fractal architecture (observation O7)

Atrium is a Subgraph; SubgraphSpec is an Atrium-recursable primitive. §6.2 envelope is composition-neutral to this fractal architecture — the envelope encrypts payload bytes regardless of whether the payload is a leaf-Node or a Subgraph-CID-reference-bundle.

**One sharp finding:** the recursive composition means a Drop bundle for SubgraphSpec_X may contain references to SubgraphSpec_Y (a nested Subgraph). The recipient who receives access to SubgraphSpec_X via the Drop also needs access to SubgraphSpec_Y to actually traverse. The UCAN scope chain (per §3.2) must compose transitively. **§6.2 envelope is fine; UCAN-chain transitivity is a downstream concern.**

### 6.5.5 sendme delivery modes (observation O8)

The spike-A2 README (per task description) names 3 sendme delivery modes. Without access to the README artifact directly (not found on origin branches I searched), I infer the modes are likely: (a) direct point-to-point via QR/ticket-paste, (b) iroh-gossip-broadcast publication, (c) untrusted-relay-archival. §6.2 envelope composes with all three: the envelope bytes are mode-agnostic; the ticket/gossip-message/relay-archive carries the envelope-CID + sender-endpoint.

**One finding:** untrusted-relay-archival mode + 6-months-offline scenario amplifies the AMENDMENT A3 (recipient-key-rotation) gap. The longer the archive duration, the higher the probability that recipient's HPKE encryption pubkey has rotated. **A3 is load-bearing specifically for this mode.**

---

## 7. Recommended amendments to §6.2 + Amendments 1-6 for Atrium-integration

### 7.1 AMENDMENT A1 (load-bearing): Multi-recipient stanza composition rule

**Codify `EnvelopePayload::HpkeMultiBase` variant** with `Vec<HpkeRecipientStanza>` + cross-stanza substitution defenses (per §2.5 above). Mint `LAYER_C_DROP_MULTI_RECIPIENT` codepoint distinct from single-recipient. Confidence: HIGH.

### 7.2 AMENDMENT A2 (load-bearing): Dual-CID model for Drop bundles

**Define `plaintext_cid` (stable, graph-referenced) + `envelope_blob_cid` (transport, mutable across reseal)** as two distinct CIDs (per §3.3 above). Disagree with prior P2P-architect-review F-refinement-2 ("Drop-CID includes recipient_did_set"); recipient-substitution defense lives in per-stanza AAD per A1, not in Drop CID. Confidence: HIGH.

### 7.3 AMENDMENT A3 (load-bearing): Recipient-key-rotation generation binding

**Extend `BindingContext::DropToRecipient` with `recipient_key_generation: u32`**; specify recipient-side decap-key retention policy (1-year grace minimum, configurable, vault-encrypted). Amend Compromise #31 to clarify "forever-valid" means "within retention window." Confidence: HIGH.

### 7.4 AMENDMENT A4 (load-bearing): K_principal generation tracking

**Bind `k_principal_generation: u32` into Layer-B per-Node AEAD AAD** + into `BindingContext::Vault`. Specify Atrium-replicated KPrincipalRotation Node format. Vault stores `HashMap<u32, [u8; 32]>` under DAK. Confidence: HIGH.

### 7.5 AMENDMENT A5 (load-bearing): ExecuteWorkflow PermissionOperation variant

**Extend `PermissionOperation` enum with `ExecuteWorkflow { workflow_cid, input_node_cids, max_decrypt_count, result_recipient_pubkey, executor_did }`**. AAD-binds the execution-scope so runtime-enforcement violations are auditable. Confidence: HIGH for hyper-scaling; MEDIUM for v1-beta-day-one (but interface-freeze argues HIGH).

### 7.6 Observation O1: HPKE-encryption-pubkey discovery substrate

Name peer-discovery (DID resolution vs Atrium-membership-attestation vs iroh-gossip) as separate-but-coupled work item at Phase-4-Meta-Core. NOT a §6.2 amendment; documentation requirement. Confidence: MEDIUM (substrate choice is independent of envelope shape).

### 7.7 Observation O2: Willow composition (parked-but-documented)

If Benten ever adopts Willow, Drop bundles MUST be eager-delivery-class or supplemented with out-of-band poll to preserve Compromise #31 forever-valid semantic. NOT a §6.2 amendment; future-substrate-decision flag. Confidence: HIGH-conditional (only matters if Willow adopted).

### 7.8 Affirmation: Amendment 5's variant-specific time-binding scoping is correct

The third reviewer's Amendment 5 explicitly excludes Vault + DropToRecipient from time-bound binding ("vault is at-rest by design; drops are intentionally long-lived per Compromise #31"). **From the offline-first + Atrium-composition lens, this exclusion is correct as written.** Re-affirming, no change recommended.

### 7.9 Refinement to Amendment 4 (sender-DID binding)

Amendment 4 binds `granting_user_did` + `requesting_device_did` into RemotePermission AAD. For composition with A5's ExecuteWorkflow, the `executor_did` is structurally similar but distinct (executor ≠ requesting-device in the hyper-scaling case where a workflow scheduler dispatches to executor pool). Suggest A5's `executor_did` is a SEPARATE binding from Amendment 4's `requesting_device_did`. No conflict; clarifying overlap.

### 7.10 Cumulative §6.2 shape (post-Amendments 1-6 + A1-A5)

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,                         // BE-pinned per 2nd-opinion §2.5 / 3rd-reviewer §2.5
    payload: EnvelopePayload,
    aad_binding: BindingContext,
}

pub enum EnvelopePayload {
    SymmetricAead { /* Layer-A vault */ ciphertext: Vec<u8>, nonce: [u8; 12] },
    HpkeBase { /* single-recipient Layer-C/D */ hpke_encap: Vec<u8>, ciphertext: Vec<u8> },
    HpkeMultiBase {                                                   // A1 NEW
        cek_aead_ciphertext: Vec<u8>,
        cek_aead_nonce: [u8; 12],
        stanzas: Vec<HpkeRecipientStanza>,
    },
}

pub struct HpkeRecipientStanza {
    recipient_did: Did,
    recipient_key_generation: u32,                                    // A3 NEW
    hpke_encap: Vec<u8>,
    wrapped_cek: [u8; 48],
}

pub enum BindingContext {
    Vault {
        vault_version: u8,
        k_principal_generation: u32,                                  // A4 NEW
    },
    DropToRecipient {
        audience_did: Did,
        sender_did: Did,                                              // Amendment 4
        recipient_key_generation: u32,                                // A3 NEW
    },
    DeviceLink {
        provisioning_session_id: [u8; 16],
        sender_device_did: DeviceDid,                                 // Amendment 4
        sealed_at_epoch_seconds: u64,                                 // Amendment 5
        valid_until_epoch_seconds: u64,                               // Amendment 5
    },
    RemotePermission {
        request_id: [u8; 16],
        operation: PermissionOperation,                               // includes A5 ExecuteWorkflow
        granting_user_did: Did,                                       // Amendment 4
        requesting_device_did: DeviceDid,                             // Amendment 4
        sealed_at_epoch_seconds: u64,                                 // Amendment 5
        valid_until_epoch_seconds: u64,                               // Amendment 5
    },
}

pub enum PermissionOperation {
    Decrypt(Cid),
    SignUcanDelegation { scope: SubgraphSpecRef, audience: Did, expires_at: u64 },
    RemoteUnlock,
    ExecuteWorkflow {                                                 // A5 NEW
        workflow_cid: Cid,
        input_node_cids: Vec<Cid>,
        max_decrypt_count: u32,
        result_recipient_pubkey: HybridKemPubKey,
        executor_did: Did,
    },
}

impl EncryptedEnvelope {
    /// Per Amendment 1 + Amendment 3 + per-stanza extensions for HpkeMultiBase
    fn canonical_binding(&self, stanza_idx: Option<usize>) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"benten-envelope-v1");
        buf.extend_from_slice(&self.codepoint.to_be_bytes());
        let tlv = self.aad_binding.canonical_serialize_tlv();        // Amendment 3
        buf.extend_from_slice(&(tlv.len() as u32).to_be_bytes());
        buf.extend_from_slice(&tlv);
        if let EnvelopePayload::HpkeMultiBase { cek_aead_ciphertext, cek_aead_nonce, stanzas } = &self.payload {
            // A1 cross-stanza substitution defense
            let body_cid = blake3(cek_aead_ciphertext, cek_aead_nonce);
            buf.extend_from_slice(body_cid.as_bytes());
            let recipient_set: Vec<&Did> = stanzas.iter().map(|s| &s.recipient_did).sorted().collect();
            for did in recipient_set { buf.extend_from_slice(did.as_bytes()); }
            if let Some(idx) = stanza_idx {
                buf.extend_from_slice(&(idx as u32).to_be_bytes());
                buf.extend_from_slice(&stanzas[idx].recipient_key_generation.to_be_bytes());
            }
        }
        buf
    }
}
```

---

## 8. Self-assessment + confidence + lower-confidence areas

### 8.1 Confidence summary per finding

| Finding | Confidence |
|---|---|
| A1 multi-recipient stanza composition | HIGH |
| A2 dual-CID model (plaintext + envelope-blob) | HIGH |
| A3 recipient-key-rotation generation binding | HIGH |
| A4 K_principal-generation tracking | HIGH |
| A5 ExecuteWorkflow PermissionOperation | HIGH for hyper-scaling, MEDIUM for v1-beta-day-one |
| §2.1 envelope CID stability via iroh-blobs | HIGH |
| §2.2 sendme ticket compatibility | HIGH |
| §3.1 flat (not nested) Drop bundle composition | HIGH |
| §3.4 SubgraphSpec resolution = downstream concern | MEDIUM (depends on S&C wave's choice) |
| §4.1 forkability composes via A1+A2 | HIGH |
| §5.1 Amendment 5 variant-scoping is correct | HIGH |
| §6.1 valid_until + workflow-execution duration | MEDIUM (relies on operational discipline) |
| O1 peer-discovery substrate naming | MEDIUM |
| O2 Willow eager-vs-lazy interaction | HIGH-conditional |
| O6 Willow + Compromise #31 deliverability | HIGH-conditional |

### 8.2 Lower-confidence areas (honest disclosure)

1. **spike-A2 README on sendme delivery modes** — I could not locate this artifact in any branch under `.addl/spikes/README.md`. The task description names 3 sendme modes I inferred from context. If the actual modes differ, §6.5.5 may need revision. Confidence on §6.5.5 specifics: LOW; on the structural claim that §6.2 envelope is mode-agnostic: HIGH.

2. **RATIFIED-sharing-and-confidentiality-2026-05-21.md + SESSION-2026-05-20-to-2026-05-21-substrate-wave-and-willow-pivot.md** — neither located on origin/main or any phase-4-meta-core branch I searched. The task description summarizes their content (Drop bundle = "full S&C composition in CBOR-on-disk shareable via sendme tickets + iroh-blobs"; per-Node AEAD `K(N) = KDF(K_principal, N.cid)`; structured UCAN scopes). My review applies this summary as ground-truth. If the actual ratified docs specify different shapes (e.g. nested envelopes; per-recipient Drop CIDs), some recommendations need revision. Confidence on architectural composition claims grounded in summary: MEDIUM-HIGH; on direct cite-anchoring to RATIFIED-S&C: cannot verify.

3. **MLS-PQ Layer-3b future composition** — I treat the prior P2P-architect review's Layer-3b deferral as ratified; if MLS-PQ-derived CGKA lands earlier than expected (say WGLC by Q3-2026), the A1 multi-stanza shape may become a transitional design rather than a v1-beta-and-beyond design. Confidence on A1 as v1-beta-shippable: HIGH; on A1 as long-term: MEDIUM (post-Layer-3b, A1 becomes legacy-compat).

4. **iroh-blobs offline-holder semantics** — iroh-blobs docs do not address offline-holder availability; the question "if a relay holding Bob's envelope goes offline before Bob comes back, what happens?" is unanswered. Probably resolved by replication policy (multiple relays hold the same blob), but this is iroh-blobs-operational not §6.2-envelope. Confidence on §6.2 composition with relay-offline: MEDIUM (depends on iroh-blobs replication discipline).

5. **Atrium-fork mechanics beyond Ben's 2026-05-27 ratification** — I treat "member-leaves-keeps-past-content" as the full forkability semantic. If Atrium-fork has additional semantics I don't know (e.g. fork-A's K_principal partially-overlaps with fork-B's), A4 may need refinement. Confidence on §4 finding under Ben's ratification: HIGH; under unstated additional semantics: MEDIUM.

### 8.3 What this review does NOT cover

- **Cryptographic soundness of §6.2 primitives** — covered by 3 prior reviewers. I take their conclusions as ground-truth.
- **Privacy properties** (lens L6 territory) — observation O3 (sendme metadata leak) flagged in passing only.
- **Side-channel surface analysis** (third reviewer's Amendment 6 territory) — affirmed but not re-derived.
- **DID method choice details** (whether `did:key` supports hybrid ML-KEM-768-X25519 encoding; whether a new method like `did:benten` is needed) — flagged in O1 as substrate work.
- **Tauri-specific HPKE pubkey publication UX** — outside lens.

### 8.4 Self-critique

I may be over-fitting the multi-recipient amendment (A1) to the Atrium use case. **Counter-argument:** age + Saltpack + prior P2P-architect review §2 all converge on multi-stanza; the divergence to consider is a true CGKA primitive (Layer-3b MLS-PQ-derived) which is deferred per ratified Option-F structural-split. A1 is the v1-beta-shippable shape; if it doesn't land, §6.2 ships as a single-recipient primitive in an Atrium-multi-recipient architecture. **Standing by A1 as load-bearing.**

I may be over-stating the K_principal-rotation concern (A4). **Counter-argument:** rotation IS realistic (security incidents; Argon2id-param-upgrade; long-term-key-hygiene); without generation-tagging, rotation is structurally un-implementable while preserving historical-Node-readability; the interface freeze NOW means catching this NOW vs paying interop-break later. **Standing by A4 as load-bearing.**

I may be over-reaching on A5 ExecuteWorkflow for v1-beta. **Counter-argument:** A5 is additive-codepoint per Amendment 2's strict-decode dispatch table; reserving the variant shape costs ~zero LOC at v1-beta and preserves the hyper-scaling upgrade path; not reserving forces wire-format-break when hyper-scaling lands. **Standing by A5 as load-bearing-with-MEDIUM-on-immediate-implementation.**

---

## 9. Citations

### 9.1 Benten internal sources (ground-truth-verified)

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — `.addl/phase-4-meta/option-f-plus-pseudo-keypair-review.md` — first reviewer §6.2 sketch.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — `.addl/phase-4-meta/option-f-plus-second-opinion-cryptographer-review.md` §2.4 (Amendment 1 codepoint-in-AAD), §2.5 (Amendment 2 strict-decode), §2.6 (cross-codepoint scenarios).
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — `.addl/phase-4-meta/option-f-plus-third-reviewer-adversarial-design.md` §2.1 (Amendment 3 TLV length-injectivity), §2.2 (Amendment 4 sender-DID binding), §2.3 (Amendment 5 replay-window), §2.4 (Amendment 6 Bernstein-Persichetti CT-Decap mandate), §2.5 (endianness pinning), §2.7 (multi-stanza residual concern explicitly named).
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — `.addl/phase-4-meta/e2r-ffull-scope-review.md` §6.4 (PermissionOperation enum), §7.2 (ProvisioningPayload wire format), §14.1 (Option F+ pitch + EncryptedEnvelope sketch).
- `phase-4-meta-core/encrypt-to-recipient-review-p2p-architect @ 8cfb079c` — `.addl/phase-4-meta/encrypt-to-recipient-review-p2p-architect.md` §2.F (Option F structural-split), §6 R-3 (multi-stanza fallback known limitation), §7.F-refinement-2 (Drop-CID derivation — I disagree per A2).
- `docs/INVARIANT-COVERAGE.md` (referenced via third-reviewer cites) — Inv-15 + pending Inv-16 3-layer decomposition.
- `docs/SECURITY-POSTURE.md` — Compromise #30 (PQ-impl-audit-maturity), Compromise #31 (forever-valid Drop bundles); A3 recommends amendment of #31.
- CLAUDE.md baked-in #5 (crypto-agility + additive codepoint discipline), #17 (engine deployment shapes), #18 (authority-isolation vs confidentiality-isolation).

### 9.2 External standards + drafts

- RFC 9180 — HPKE. §5.1 (info parameter binding into KeySchedule via labeled_extract); §5.1.1 (mode_base sender-auth absence — "the most basic function of an HPKE scheme is to enable encryption to the holder of a given KEM private key" without authentication); §9.1.1 (Auth mode contrast — base mode provides no sender-auth assurance); §9.1.2 (IND-CCA2 properties); §9.1.4 (malleability mitigation via key-binding); §9.7.3 (REPLAY PROTECTION — "HPKE provides no other replay protection" beyond same-stream ordering, motivating Amendment 5's explicit time-binding for time-sensitive variants).
- RFC 8949 — CBOR (Drop bundle on-disk format).
- RFC 9420 — MLS (§5.1 sender-binding precedent for Amendment 4; §16 epoch precedent for A4 generation tracking).
- FIPS 203 — ML-KEM (§6.3 implicit-rejection precedent for Amendment 6).

### 9.3 P2P substrate references

- iroh-blobs protocol — `docs.iroh.computer/protocols/blobs` — BLAKE3 content-addressing ("All blobs within iroh are referred to by the BLAKE3 hash of its content"); BlobTicket = (hash, EndpointId); incremental verification ("integrity of each chunk is checked both by the sender and the receiver"); no encryption at blob layer (composition-relevant: §6.2 envelope-bytes-as-blob composes cleanly).
- Willow Protocol confidential-sync — `willowprotocol.org/specs/sync/index.html` — 3D range-based set-reconciliation under read-cap gating; selective payload delivery (eager < threshold; lazy on request); "does not address encryption of payloads at rest" (composition-relevant: O6 finding on Drop-bundle deliverability if Willow adopted).
- age multi-recipient encoding — age-encryption.org/v1 — multi-stanza precedent for A1.
- Saltpack encrypted-payload-key — saltpack.org/encryption-format-v2 — encrypted-payload-key + per-recipient-wrap precedent for A1.
- Signal protocol session-state management — signal.org "session-state-management" — recipient-retained historical decap key precedent for A3.

### 9.4 Academic + precedent

- Barbosa et al. on X-Wing IND-CCA2 in standard-model PQ-half + ROM-model DH-half (cited by prior P2P-architect review §8.10).
- Bernstein & Persichetti, "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems," IACR 2024/2051 (cited by third reviewer §2.4 motivating Amendment 6).

---

**End of L9 Atrium-integration review.**

**Summary handoff to orchestrator:** §6.2-with-Amendments-1-6 is cryptographically sound per the 3 prior reviewers. From the Atrium-peer-mesh / iroh-blobs / sendme / forkability / offline-first / ephemeral-execution composition lens, FIVE additional load-bearing amendments (A1-A5) are required for §6.2 to actually compose with Benten's P2P architecture without wire-format break at first integration. A1 (multi-recipient stanza) is the most-substantive single finding. A2 (dual-CID), A3 (recipient-key-rotation generation), A4 (K_principal-generation tracking), A5 (ExecuteWorkflow PermissionOperation) round out the load-bearing set. Two observations (O1 peer-discovery, O6 Willow eager-vs-lazy) are documentation requirements, not amendments. Recommend Ben ratify A1-A5 alongside Amendments 1-6 before treating §6.2 as load-bearing-final.
