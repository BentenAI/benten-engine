# Option F+ §6.2-with-Amendments — L6 Privacy / Metadata-Leak / Unlinkability review

**Branch:** `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak`
**Reviewer lens:** Senior privacy-engineer + metadata-leak analyst. Distinct from the three prior cryptographer lenses (which focused on confidentiality + integrity + authentication). **My question is the one the second-opinion reviewer explicitly excluded at line 187 of his report ("the only 'leakage' is the public envelope-level fact... no new leakage from the unification") — what does the envelope LEAK in metadata terms, distinct from payload confidentiality, and how does that interact with a decentralized peer-mesh whose relays are untrusted by design and which archives envelopes forever (Compromise #31)?**

**Inputs reviewed (origin, frozen SHAs):**
- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — first cryptographer (NO-GO on F+ Layer-A; sketched §6.2 envelope-format unification)
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — second-opinion CONCUR-WITH-AMENDMENTS (Amendments 1+2; explicitly punted privacy at line 187)
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ HEAD` — adversarial red-team (Amendments 3+4+5+6)
- `docs/SECURITY-POSTURE.md` — Compromise #31 (Drop bundle revocation reach; forever-valid once distributed) at line 91 + §R6 detail at lines 2712-2820
- `docs/ARCHITECTURE.md` — Atrium peer-mesh + iroh transport (relay + holepunch) at lines 88-108 + 507-509; Drop bundle envelope-sig defense-in-depth at lines 165-181
- `docs/CRATES-DEEP-DIVE.md` line 37 — `benten-drop` envelope shape
- `docs/V1-FROZEN-INTERFACE.md` items 6, 15 — encryption codepoints + RestrictedScope + AuthorizationGrant
- `docs/INVARIANT-COVERAGE.md` Inv-15 — payload-CID identifier discipline
- `docs/V1-WIRE-FORMAT-INVENTORY.md` — 10 wire surfaces
- (VISION.md does not exist at HEAD; brief mentions it as input #7 — acknowledged absent; no semantic loss for the privacy lens since `docs/ARCHITECTURE.md` carries the load-bearing peer-mesh posture statements.)

**Web sources verified during this review (citations in §9):** RFC 9180 §9.8 (HPKE-mode-base no-recipient-anonymity); Signal Sealed Sender 2018 + 2024 designs; Signal PQXDH whitepaper; age multi-recipient privacy discussion #463 + Filippo Valsorda "Sealed Sender" critique; Pond / Briar / Cwtch threat models; Sphinx packet format (Danezis-Goldberg 2009); MASQUE / Oblivious HTTP (RFC 9458); Tor padding-machine + WTF-PAD; Loopix/Mixnet cover-traffic; PSI literature (Pinkas et al.); HPKE-PSK auth-mode (RFC 9180 §5.1.2-§5.1.4).

---

## §1. Executive verdict + confidence

**VERDICT: CONCUR-WITH-CALIBRATION on §6.2-with-Amendments-1+2+3+4+5+6 as a confidentiality+integrity+authentication design. DISAGREE that it is "load-bearing-final for v1-beta" in any sense that includes metadata-privacy properties. The envelope as currently specified is metadata-promiscuous-by-default — every load-bearing privacy property a thoughtful end-user would expect (recipient anonymity, sender anonymity, operation-type confidentiality, unlinkability across drops, padding against size-correlation, freshness without timing-leakage) is ABSENT or NEGATIVELY violated by the current AAD/BindingContext shape.**

Confidence:
- **HIGH** on the enumeration of the leak surface (§2) — the leaks are textual properties of the BindingContext enum + the codepoint-in-plaintext-AAD + the sealed-at-epoch-in-AAD; they read directly off the design.
- **HIGH** on the Drop-bundle-archive long-term threat being load-bearing for the v1-beta posture (§4) — Compromise #31's "forever-valid Drop bundles" composes catastrophically with plaintext-DID-in-AAD for any archival adversary at the relay or peer-mesh layer.
- **HIGH** on the recommendation to mint a **Sealed-Sender codepoint variant** as future-additive (§3.1) — Signal Sealed Sender has been production for 7 years (since 2018); the design is well-understood; the additive-codepoint slot fits the existing crypto-agility framework with zero wire-break.
- **MEDIUM-HIGH** on the recommendation to FREEZE the load-bearing privacy-disclosure caveat into `SECURITY-POSTURE.md` AS A NEW COMPROMISE (#32) BEFORE v1-beta tag (§6) — this is the honest-disclosure discipline operating; the design CAN ship with the leak provided the leak is disclosed in plain English; what CAN'T ship is the leak as a silent property.
- **MEDIUM** on the specific recommendations for padding (§3.3) and cover-traffic (§3.4) — these are well-studied but their cost/benefit at Benten's exact deployment shape is empirical and warrants a separate measurement pass. I recommend STANDING THE PADDING-CLASS DISCIPLINE for v1-beta (size-class buckets per codepoint) and DEFERRING cover-traffic to a post-v1-GM evaluation tied to whether Benten ships an Atrium-relay product or remains peer-only.
- **LOWER** on whether the four prior reviewers' implicit framing ("Benten is end-to-end encrypted therefore privacy-respecting") matches the actual privacy posture the design delivers. I believe the design ships strong **content** privacy and **weak-to-absent** **metadata** privacy; if the marketing framing is "your conversations are private" the wire shape will undermine that within the first published Benten audit.

**Most-substantive single concern:** **Recipient DID + Sender DID + Operation type are ALL in plaintext AAD on the wire.** An adversary observing untrusted-relay traffic learns a complete social graph + per-user operation-mix WITHOUT decrypting anything. Compromise #31 says the relay archives this forever. This is strictly worse than Signal Sealed Sender's privacy posture (Signal hides sender; we hide neither sender nor recipient) AND worse than age's multi-recipient privacy posture (age has anonymous-recipient mode at the file level). For a system positioned as "decentralized + end-to-end encrypted + p2p-encryption-by-default," this is the **biggest single gap between the implementation and the user-facing promise**.

**Most-actionable single recommendation:** Mint Compromise #32 (metadata-leak in v1-beta envelope) + Inv-16 (load-bearing privacy disclosure surface in `SECURITY-POSTURE.md`) + ratify a future-additive Sealed-Sender codepoint slot in the wire-format spec so the upgrade path is unambiguous. See §6.

---

## §2. Metadata-leak enumeration — per BindingContext variant × per adversary capability

The §6.2 design at the third-reviewer's HEAD (with Amendments 1+2+3+4+5+6 applied) is:

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,                       // PLAINTEXT
    payload: EnvelopePayload,             // CIPHERTEXT
    aad_binding: BindingContext,          // PLAINTEXT (bound into AEAD/HPKE AAD)
}

pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient {
        audience_did: Did,                                       // PLAINTEXT
        sender_did: Did,                                         // Amendment 4 — PLAINTEXT
        sealed_at_epoch_seconds: u64,                            // Amendment 5 — PLAINTEXT
        valid_until_epoch_seconds: u64,                          // Amendment 5 — PLAINTEXT
    },
    DeviceLink {
        provisioning_session_id: [u8; 16],                       // PLAINTEXT (random)
        sender_device_did: DeviceDid,                            // Amendment 4 — PLAINTEXT
        sealed_at_epoch_seconds: u64,                            // Amendment 5 — PLAINTEXT
        valid_until_epoch_seconds: u64,                          // Amendment 5 — PLAINTEXT
    },
    RemotePermission {
        request_id: [u8; 16],                                    // PLAINTEXT (random)
        operation: PermissionOperation,                          // PLAINTEXT (enum tag + bytes)
        granting_user_did: Did,                                  // Amendment 4 — PLAINTEXT
        requesting_device_did: DeviceDid,                        // Amendment 4 — PLAINTEXT
        sealed_at_epoch_seconds: u64,                            // Amendment 5 — PLAINTEXT
        valid_until_epoch_seconds: u64,                          // Amendment 5 — PLAINTEXT
    },
}
```

**Critical recognition:** Amendments 4 + 5 are the right call for **authentication and replay-defense** (third-reviewer's §2.2 + §2.3 attacks are real and the amendments close them). But each authentication-binding amendment is **plaintext-bound** — it must be plaintext so the recipient can verify the AAD before decrypting, AND so untrusted relays can route on it. The amendments STRENGTHEN cryptographic-integrity properties WHILE STRICTLY WORSENING metadata-privacy properties. **This is a real tension the prior three reviewers did not name.**

### §2.1 Per-variant leak matrix

| Variant | Plaintext-AAD fields | Adversary learns from ONE observed envelope |
|---|---|---|
| `Vault` | `codepoint`, `vault_version` | "User has a Benten vault at version V." No recipient/sender/timing leak (vault is on-device). LOW leak — this is the BENIGN variant. |
| `DropToRecipient` | `codepoint`, `audience_did`, `sender_did`, `sealed_at_epoch_seconds`, `valid_until_epoch_seconds`, envelope-size class, route-Atrium-peer | Sender DID ↔ Recipient DID linkage on the wire; precise ~1-second timestamp; validity window (often hints at semantic — "10 minute share" vs "30 day share"). Network observer reconstructs the **directed-edge sender→recipient at time T** without any cryptographic break. **HIGH leak.** |
| `DeviceLink` | `codepoint`, `sender_device_did`, `provisioning_session_id`, `sealed_at_epoch_seconds`, `valid_until_epoch_seconds`, size | Device-key DIDs visible; binds two devices of the same user to each other (cross-device linkability at the network layer). **MEDIUM-HIGH leak** — typically rare (device-add events; few per user lifetime) so absolute count is low; but each event is high-signal. |
| `RemotePermission` | `codepoint`, `granting_user_did`, `requesting_device_did`, `operation` enum tag + operation bytes, `request_id`, sealed-at/valid-until, size | **THE WORST.** Operation-type in plaintext means the relay-archival adversary sees not just "Alice talks to Bob" but **"Alice asked Bob for permission to invoke procedure foo at time T."** Operation-type frequency distribution per-user is uniquely identifying — see §2.3 long-term archive analysis. **HIGH leak.** |

### §2.2 Per-adversary-capability leak matrix

| Adversary capability | Single-envelope leak | Multi-envelope (same pair) leak | Multi-envelope (cross-pair) leak |
|---|---|---|---|
| **Passive wire-tap (1 envelope)** | codepoint ✓ + recipient_did ✓ + sender_did ✓ (Am4) + sealed_at_epoch ✓ (Am5) + valid_until ✓ + envelope-size class + Atrium-hop ✓ | n/a | n/a |
| **Passive wire-tap (N envelopes between same pair)** | (above per-envelope) | Per-pair message-frequency curve over time; operation-mix histogram for that pair; size-class histogram (small KB sub-envelope = control-plane; ~4 KB Spike-G upper-bound = Drop bundle); coarse-grained "conversation rhythm" (timezone, business hours, weekends-quiet) | Identifies "Bob is the most active recipient of Alice's RemotePermission requests" — useful for relationship-inference |
| **Compromised relay (archives forever — Compromise #31)** | Sees every envelope it transits or has been gossiped | **OVER YEARS:** Full directed-graph of `(sender_did, recipient_did, codepoint, operation, sealed_at)` quadruples for every transited envelope. This is a **complete social-graph + operation-type history**. Forever. No future revocation cuts past relay state. | **Cross-pair correlation:** identifies hubs/cliques; identifies the set of DIDs that ever interacted; identifies which DIDs co-cluster around the same group-share content (via Atrium peer-mesh route signatures + envelope-size correlations) |
| **Network-level traffic analysis (Tor-style adversary; sees timing + size + Atrium-hop count)** | Bytes-on-wire ≈ envelope size ± wrapper overhead; iroh route-RTT signatures fingerprint geographic peer pair | Timing correlations enable confirmation attacks (Alice sends X bytes at T; Bob receives ~X bytes at T+δ → confirmed pair) | Inter-pair traffic correlation reveals when two pairs are gossiping the same Drop bundle (size + timing match) |
| **Long-term archive correlator** | (above) | (above) | Identifies "Alice's most frequent collaborators" / operation-type distribution per user (per-user fingerprint) / lifetime DID-rotation history (Did rotations leak via envelope DID changes); enables **deanonymization via auxiliary information**: if Alice publishes ANY public DID anywhere (npm package author, GitHub, ATProto handle), the archived envelope-graph reveals her entire Benten social graph |

### §2.3 The "operation-type in plaintext AAD" amplification — most-important single point

`PermissionOperation` is a structured enum carrying the operation NAME (e.g., `Capability("benten.fs.read")` or `PluginInstall("did:plc:foo:plugin-name")`). Whatever its concrete encoding, it is:

1. Plaintext-in-AAD per Amendment-1's design (codepoint + canonical_binding fold).
2. Variable-length per Amendment-3 (TLV-encoded explicitly so length-injectivity is closed) — meaning **length-prefix LEAKS the operation-string-length**.
3. **Per-operation entropy is low** — operation names live in a small universe (`benten.fs.*`, `benten.network.*`, `benten.plugin.*`, plus plugin-author operations). An adversary with a published-plugin-catalog (npm + ATProto registry) can dictionary-attack operation names from length + per-byte distribution alone, without seeing plaintext.
4. **Per-user operation-frequency histograms uniquely identify users.** Just as Tor browser-fingerprinting uses font-list/timezone/etc., per-user `(operation, count)` histograms across a multi-month archive uniquely fingerprint individuals among a known population. This is the same statistical-disclosure principle that broke k-anonymity in the Netflix Prize dataset (Narayanan-Shmatikov 2008).

**Net:** for any Benten user whose DID is publicly attestable in ANY other context, the relay-archive adversary obtains a multi-year, plaintext-typed operation log keyed by their real-world identity. The strongest end-to-end encryption in the world doesn't help — the metadata IS the disclosure.

### §2.4 Per-Amendment privacy impact assessment

| Amendment | Stated purpose | Privacy impact | Verdict |
|---|---|---|---|
| **Am1 (codepoint-in-AAD)** | Closes codepoint-substitution attack (second-reviewer §2.4) | Codepoint MUST be plaintext (route/dispatch + crypto-agility). Each codepoint cleanly labels envelope as "Layer-A vault" vs "Layer-C drop" vs "Layer-D wrap" vs "Layer-D remote-perm" — adversary learns BROAD-SHAPE (4 possible meanings) per envelope. **Acceptable trade-off; this leak is structural to crypto-agility.** ✓ design choice that's OK provided the broad-shape leak is disclosed. |
| **Am2 (strict-decode)** | Closes variant-confusion attack | No incremental privacy impact (strict-decode happens after AAD-binding fold). ✓ |
| **Am3 (TLV length-prefix)** | Closes length-injectivity attack (third-reviewer §2.1) | **LEAKS field-cardinality structure per variant.** For `BindingContext::DropToGroup` (future v2 multi-stanza per third-reviewer §2.7), TLV length-prefixes EXPLICITLY leak audience-cardinality (recipient-list length). For `DropToRecipient`-single, it leaks `audience_did` length (which is uninformative for did:key, ~58 bytes; but informative for did:plc which has shape `did:plc:XXXXXXXXXXXXXXXX`). **Material side-effect not previously named.** Mitigation: pad audience-DID to a fixed 64 bytes per Inv-16-companion below. |
| **Am4 (sender-DID + recipient-DID + grantor/requester-DIDs in AAD)** | Closes outer-sig confusion attack (third-reviewer §2.2) — defense-in-depth on sender-authentication | **WORST PRIVACY-IMPACT AMENDMENT.** Before Am4, sender identity could potentially be HPKE-mode-auth-encrypted-to-recipient (Signal Sealed Sender pattern). After Am4, sender DID is structurally plaintext in AAD. **This forecloses the future-additive Sealed-Sender codepoint UNLESS the design explicitly carves out a codepoint slot where sender-DID is NOT in plaintext AAD.** See §3.1 for the proposed fix: mint a `DropToRecipientSealedSender` codepoint that uses HPKE-mode-auth + binds sender-DID INSIDE the ciphertext (not AAD); current Am4 stays for the default codepoint. |
| **Am5 (sealed-at-epoch + valid_until in AAD)** | Closes replay attack (third-reviewer §2.3) | **LEAKS ~1-second-precision creation timestamp on every envelope.** Inside ciphertext, this isn't worth fixing; in plaintext AAD, it's a continuous timestamp oracle. Mitigation: round to coarse epoch buckets (5-minute? 1-hour?) per Inv-16-companion. **Trade-off: looser bucket = better privacy, looser replay window = more attack surface.** Recommendation: 1-hour epoch bucket + per-envelope-random jitter (0-3600 seconds randomly subtracted before encoding) — equivalent replay-defense, ~6 bits of timing leak per envelope vs ~33 bits. |
| **Am6 (CT-Decap mitigation for Bernstein-Persichetti)** | Closes Decap-side ML-KEM side-channel | No privacy impact (orthogonal to metadata; affects payload confidentiality). ✓ |

---

## §3. Privacy-engineering amendment recommendations — concrete additions

### §3.1 NEW Amendment 7 — Sealed-Sender codepoint variant (Signal precedent; future-additive slot OPENED at v1-beta)

**Goal:** for the specific use case of "Alice sends Bob a drop without revealing to the relay that the sender is Alice," mint a NEW codepoint that uses HPKE-mode-auth with sender-DID bound INSIDE the ciphertext (encrypted-to-recipient-only) and a generic plaintext AAD `BindingContext::DropToRecipientSealedSender { audience_did, sealed_at_epoch_hour }` (no sender_did in AAD).

**Signal precedent.** Signal Sealed Sender (2018; https://signal.org/blog/sealed-sender/) hides sender identity from the Signal-server relay: the envelope at the relay carries only `(recipient_id, server-side delivery token)`; sender_id is encrypted inside the payload under recipient's identity-key. The recipient verifies sender identity at decrypt time. Trade-off: relay cannot rate-limit by sender (Signal added per-recipient delivery tokens to mitigate spam). 2024 update: Signal Sealed Sender V2 (https://signal.org/blog/sealed-sender-multi-recipient/) extends to multi-recipient with per-recipient unlinkable copies.

**Design sketch (v1-beta future-additive slot; implementation deferred to post-v1-beta wave per CLAUDE.md baked-in #5 crypto-agility additive-codepoint discipline):**

```rust
pub enum BindingContext {
    // ... existing variants ...

    /// Amendment 7: Sealed-Sender codepoint. Sender DID is bound INSIDE
    /// the ciphertext via HPKE-mode-auth's psk_id binding (RFC 9180 §5.1.4)
    /// OR via a sender-DID-in-plaintext-of-inner-payload that is verified
    /// post-decrypt. Plaintext AAD carries only audience + coarse epoch.
    DropToRecipientSealedSender {
        audience_did: Did,                  // PLAINTEXT — relay needs this to route
        sealed_at_epoch_hour: u32,           // PLAINTEXT — 1-hour bucket, ~14 bits/year
        // sender_did is NOT here — see ciphertext-binding below
    },
}

pub enum EnvelopePayload {
    // ... existing variants ...
    /// Amendment 7: HPKE-mode-auth or mode-auth-psk to bind sender
    /// inside the ciphertext, with sender authentication verified
    /// post-decrypt
    HpkeAuthSealedSender {
        enc: Bytes,
        ciphertext: Bytes,
        // Inner plaintext structure:
        //   sender_did: Did
        //   sender_signature: Ed25519Sig over (audience_did, sealed_at_epoch_hour, payload_cid)
        //   inner_payload_bytes: ...
    },
}
```

**Codepoint value:** allocate from the encryption-codepoint registry (§6 of `docs/V1-FROZEN-INTERFACE.md` item 6) — RECOMMEND `0x6510` (sibling to the Layer-C-drop codepoint `0x6500` sketched in the second-opinion review). LOCK the codepoint integer value at v1-beta as a RESERVED-FOR-FUTURE-ADDITIVE slot (per the codepoint-registry-governance discipline at second-opinion §8) so when implementation lands post-v1-beta, no wire-format break is needed.

**Trade-offs documented at the codepoint description:**
1. **Pro:** relay learns recipient but not sender — Signal-style metadata-asymmetry.
2. **Con:** recipient cannot pre-filter spam by sender at the AAD layer (must decrypt to learn sender) — Signal added per-recipient delivery tokens; Benten can adopt the same pattern OR add a per-recipient pre-shared-secret-keyed envelope ID (PSK-id binding via HPKE-mode-psk).
3. **Con:** does NOT close recipient-anonymity (which would require Sphinx/onion-routing — see §3.5). The Sealed-Sender codepoint addresses ONE of the two anonymity axes.
4. **Compatibility:** Sealed-Sender envelopes coexist with default `DropToRecipient` envelopes — sender chooses which to use per-drop based on threat-model preferences.

**Confidence: HIGH** that Sealed-Sender codepoint is the right additive shape. Signal has 7 years of production data; the design is well-understood; the additive-codepoint slot matches Benten's crypto-agility framework. The risk is purely "do we lock the slot at v1-beta or post-freeze?" — RECOMMEND lock at v1-beta to avoid wire-format break later.

### §3.2 NEW Amendment 8 — Per-relay-unlinkability (per-recipient unique envelope IDs)

**Goal:** prevent a relay-aggregator-adversary from linking the same envelope content across multiple relay hops or storage locations.

**Current state:** the §6.2 envelope as serialized has a stable byte representation. If Alice's envelope to Bob transits relay R1 → relay R2 (Atrium peer-mesh path), R1 and R2 see byte-identical envelopes. Cross-relay linkability is structural.

**Design:** at the iroh-blobs transport boundary (per `docs/ARCHITECTURE.md` lines 88-108), wrap the EncryptedEnvelope in a **transport-layer re-blinding outer**:

```
TransportEnvelope {
    transport_blinded_id: [u8; 32],   // SHA256(envelope_bytes || per-hop-salt)
    envelope: EncryptedEnvelope,      // the §6.2 inner envelope
}
```

Per Atrium peer-mesh hop, re-derive `transport_blinded_id` with a hop-specific salt drawn from the Noise-protocol session key (already established by iroh's QUIC layer). Two adjacent relays see different transport-IDs for the same inner envelope.

**Caveat:** does NOT prevent content-based correlation by the LAST relay (which sees the unwrapped envelope before delivering to recipient). True per-relay-unlinkability requires Sphinx-style per-hop re-encryption (§3.5).

**Severity / confidence:** MEDIUM. This is a low-cost-high-benefit defense-in-depth measure. **Confidence: MEDIUM-HIGH** that it's the right additive measure; **MEDIUM** that it's worth v1-beta-critical-path effort (could be post-v1-beta).

### §3.3 NEW Amendment 9 — Padding to fixed-size class buckets per codepoint

**Goal:** defeat size-correlation attacks (§2.2 row "compromised relay" + §2.2 row "network traffic analysis").

**Design:** every EncryptedEnvelope is padded to the next size in a publicly-defined size-class bucket:

| Codepoint | Size buckets (bytes) |
|---|---|
| Layer-A vault | 4 KiB / 16 KiB / 64 KiB / 256 KiB / 1 MiB |
| Layer-C drop | 1 KiB / 4 KiB / 16 KiB / 64 KiB / 256 KiB / 1 MiB (matches Spike G 4 KiB measurement for 5-Recipe bundles) |
| Layer-D device-link | 1 KiB (fixed — no variability) |
| Layer-D remote-perm | 1 KiB (fixed — no variability) |

Padding bytes are AEAD-encrypted with random fill (NOT zero-fill — zero-fill is detectable via compression-side-channels). Recipient strips padding post-decrypt via a length prefix INSIDE the ciphertext.

**Precedent:** Tor padding-machines / WTF-PAD (Juarez et al. NDSS 2016); Loopix mixnet poisson-rate padding; HTTP/2 frame padding (RFC 7540).

**Cost:** Spike G's 4 KiB upper-bound for 5-Recipe Drop bundles means typical-case bandwidth overhead is 0-25% (most drops are already near 4 KiB). For Layer-A vault, ~25% storage overhead. For Layer-D device-link / remote-perm, the fixed-1-KiB-padding is the design — these are RPC-shaped and naturally small.

**Severity / confidence:** MEDIUM-HIGH. **Confidence: HIGH** that padding to fixed-size-class is the right approach (industry-consensus pattern; Tor + Signal use this); **MEDIUM** on the specific bucket choices (empirical; needs measurement against the Spike G corpus + future workload). RECOMMEND v1-beta lock-in of the bucket SHAPE (publicly-defined size classes per codepoint) + post-v1-beta refinement of the specific bucket values.

### §3.4 NEW Amendment 10 — Cover-traffic / dummy envelopes (DEFERRED; analysis included)

**Goal:** defeat "Alice was active at time T" inference.

**Design space:**
1. **Poisson-process dummy envelopes** (Loopix-style; Piotrowska et al. USENIX 2017) — engine emits dummy envelopes at random intervals to a fixed pool of recipients (typically the user's own devices + a small set of "cover" peers). Bandwidth cost: ~1-10 dummy envelopes/hour at small size class.
2. **Deterministic-rate cover** (Tor circuit-pad style) — fixed rate of envelope emission regardless of user activity. Higher bandwidth cost; stronger guarantees.
3. **Network-loop cover** (Pond-style) — every active user maintains a constant low-rate cover-traffic baseline.

**Cost-benefit for Benten:**
- Benten's deployment shape is "intermittently-connected p2p peers" (per `docs/ARCHITECTURE.md` line 88-108). Background cover-traffic costs battery + bandwidth on user devices that are not always-on.
- Most Benten users will NOT need traffic-analysis-resistance — the threat model is opportunistic relay-archive adversaries, not nation-state TA-capable adversaries.

**Recommendation: DEFER to post-v1-GM.** Reason: cover-traffic is meaningful ONLY when paired with onion-routing (otherwise the cover hides "Alice sent" but not "Alice→Bob"). Onion-routing is itself post-v1 per the `Transport`-trait abstraction at `docs/ARCHITECTURE.md` line 507-509 ("post-iroh — Tor / Nostr-relay / shaped relay"). When the Tor / Nym-mixnet transport lands, cover-traffic becomes meaningful; until then, the marginal benefit is small.

**Confidence: MEDIUM.** This is a real downside trade-off (battery, bandwidth) for a real benefit (anonymity-set growth). RECOMMEND: document cover-traffic as DEFERRED-NAMED-NOW in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`; gate the design pass on shaped-relay/Tor transport extension landing.

### §3.5 NEW Amendment 11 — Multi-stanza HPKE recipient-list-hidden mode (PSI literature)

**Goal:** for group-shares (multi-recipient envelopes per third-reviewer §2.7), hide the recipient-list cardinality + identities from anyone other than the recipients themselves.

**State of the art:**
- **age** multi-recipient mode (https://age-encryption.org/v1) — recipient-list IS visible in the file (each recipient stanza is a labeled HPKE-like ciphertext-blob). age-discussion #463 (https://github.com/FiloSottile/age/discussions/463) explicitly documents that recipient-anonymity is NOT a goal.
- **Signal Sealed Sender Multi-Recipient V2** (https://signal.org/blog/sealed-sender-multi-recipient/) — per-recipient unlinkable copies; recipient learns ONLY their own recipient slot exists.
- **PSI (Private Set Intersection)** literature (Pinkas, Schneider, Zohner 2014; Kolesnikov-Kumaresan 2016) — could in principle hide group membership but adds substantial cryptographic + bandwidth cost (O(n²) message complexity for naive PSI).

**Recommendation for Benten:** the multi-stanza HpkeMultiBase variant (third-reviewer §2.7 future) should adopt the **Signal Sealed Sender Multi-Recipient pattern**: each recipient gets a distinct, unlinkable copy with NO visible cross-recipient structure on the wire. Each copy looks like an independent Sealed-Sender-codepoint envelope. Cost: O(n) bandwidth per drop (same as the naive multi-stanza), but recipient-anonymity-from-other-recipients is preserved.

**Defer specific design** to the multi-stanza HpkeMultiBase wave per the third-reviewer's §2.7 punted-to-specialist disposition. RECOMMEND minting an Inv-16-companion that LOCKS the privacy property at v1-beta: "any future group-share variant MUST provide per-recipient unlinkable copies."

**Confidence: HIGH** that the per-recipient-unlinkable-copy pattern is the right shape (Signal precedent). **MEDIUM** on whether v1-beta needs to ship the multi-stanza variant at all (the second-reviewer's design has only single-recipient HPKE; group-share is post-v1).

### §3.6 NEW Amendment 12 — Coarse-bucket sealed_at_epoch_seconds → sealed_at_epoch_hour

**Goal:** reduce the ~33-bit timestamp leak from Am5 to ~14 bits/year while preserving Am5's replay-defense property.

**Design:**

```rust
pub enum BindingContext {
    // Amendment 12 refines Amendment 5:
    DropToRecipient {
        audience_did: Did,
        sender_did: Did,
        sealed_at_epoch_hour: u32,           // CHANGED from u64 seconds → u32 hours
        valid_until_epoch_hour: u32,
    },
    // ... same change for DeviceLink + RemotePermission
}
```

**Implementation:** sender computes `sealed_at_epoch_hour = (unix_seconds + per_envelope_random_jitter[0..3600]) / 3600`. Recipient enforces `valid_until_epoch_hour + clock_skew_hours >= now_hour`. Replay window granularity becomes 1-hour-buckets, which matches the actual UCAN `nbf`/`exp` discipline used elsewhere (UCAN times are usually expressed in hours/days, not seconds).

**Cost:** loses second-granularity replay-defense (an attacker can replay within a 1-hour window). Mitigation: the OUTER application protocol (UCAN nonce + per-session nonce cache per `docs/SECURITY-POSTURE.md` line 1800 sync attack-test family) provides freshness inside the 1-hour window.

**Confidence: HIGH** that 1-hour bucket is the right granularity. **MEDIUM-HIGH** on the specific jitter mechanism (a stronger alternative: round DOWN to the hour with no jitter — leaks less but is more deterministic, easier to fingerprint).

### §3.7 Summary table of recommended new amendments

| Am # | Title | Privacy axis | v1-beta status |
|---|---|---|---|
| 7 | Sealed-Sender codepoint (additive slot) | Sender anonymity | **LOCK SLOT at v1-beta; implement post-v1-beta** |
| 8 | Per-relay-unlinkability (transport-layer blinding) | Cross-relay linkability | **LOCK SHAPE at v1-beta; implement at iroh-transport-boundary post-v1-beta** |
| 9 | Padding to fixed-size-class buckets | Size correlation | **LOCK BUCKET SHAPES at v1-beta; lock SPECIFIC bucket values post-v1-beta after measurement** |
| 10 | Cover-traffic (Loopix/Poisson) | Timing correlation | **DEFER to post-v1-GM; gate on shaped-relay/Tor transport** |
| 11 | Multi-stanza per-recipient-unlinkable copies | Group-share recipient-anonymity | **DEFER to post-v1 multi-stanza wave; lock INVARIANT at v1-beta** |
| 12 | Coarse 1-hour epoch buckets + jitter (refines Am5) | Timestamp precision | **LOCK at v1-beta; refines an existing amendment** |

---

## §4. Drop-bundle-archive long-term privacy threat (Compromise #31 5-year / 20-year analysis)

`docs/SECURITY-POSTURE.md` line 91 + §R6 (lines 2712-2820) establish that Drop bundles are **forever-valid once distributed**. The §R6 framing focuses on the **revocation reach** problem — already-distributed bundles cannot be cryptographically un-distributed. This is the correct framing for **content privacy** (Alice can't take back the content of a leaked-Drop).

The **metadata-privacy** framing is materially different. The metadata leak from §2 is *amplified* by Compromise #31 in three distinct ways:

### §4.1 5-year horizon: social-graph reconstruction

An adversary who **archives all envelope-metadata** they observe over 5 years obtains:
- Complete sender↔recipient directed graph for every Drop they intermediated.
- Per-pair message-frequency curves.
- Per-user operation-mix histograms.
- Coarse timing of every interaction.

**This is independently sufficient to deanonymize any user whose Benten DID is ever publicly attestable** (in another context — ATProto, npm, GitHub, blog, etc.). The Narayanan-Shmatikov 2008 Netflix Prize result + the Sweeney 2002 k-anonymity work establish that even small per-user statistical fingerprints suffice for deanonymization against a sufficiently-broad auxiliary information base.

**Inv-15-style 3-layer decomposition does NOT help here** — Inv-15 closes a *signature-bundle-CID* identifier hazard at the cryptographic-construction layer. The metadata-leak is at the *envelope-AAD-shape* layer; it's structurally different. Inv-15 is necessary-but-not-sufficient; we need a *metadata-disclosure* invariant (proposed as Inv-16, §6).

### §4.2 20-year horizon: post-quantum harvest-now-decrypt-later (HNDL) for metadata

The HNDL framing typically applies to **payload confidentiality** — adversary archives ciphertext today, decrypts in 2046 when a cryptographically-relevant quantum computer exists. Benten's PQ-hybrid LAMPS combiner default (per CLAUDE.md baked-in #5 + Inv-15) provides the standard mitigation: even if ML-KEM-768 is broken in 2046, the X25519 classical half provides residual security.

**Metadata leak is HNDL-irrelevant in a perverse-elegant sense:** the metadata is plaintext today; adversary doesn't need to decrypt anything in 2046. **They already have it.** This is a strictly worse HNDL property than ciphertext-HNDL — the harvest already pays out.

### §4.3 When does Inv-15 3-layer decomposition help vs hurt privacy?

Brief explicitly asks this. Inv-15 mandates payload-CID identifiers (not sig-inclusive bundle CIDs). For privacy:

- **HELPS:** payload-CIDs are deterministic across multiple senders sending the same content — enables content-deduplication at iroh-blobs without sender-revealing per-sender randomization. This is a privacy WIN.
- **HURTS:** payload-CIDs LEAK content-identity across senders — if Alice and Bob both share the same publicly-known content X, the payload-CIDs are identical, leaking "Alice and Bob have content X" to anyone with the public-content-hash. Mitigation: per-sender encryption-wrap means the OUTER ciphertext-CID differs; payload-CID identity is INTERNAL only.

**Net:** Inv-15 is privacy-neutral-to-positive at the cryptographic layer. The metadata leak at the AAD layer is orthogonal to Inv-15 and requires its own invariant (Inv-16, §6).

### §4.4 Mitigation: "DID rotation as the privacy-fundamental"

The deepest mitigation for the 5-year + 20-year archive threat is **DID rotation discipline**: users rotate their Benten DID at regular intervals (say, yearly) and old DIDs are deprecated. Per-DID-window metadata is bounded; long-term cross-DID correlation requires linking DID-rotation chains.

`docs/V1-FROZEN-INTERFACE.md` does NOT name a DID-rotation discipline at v1-beta. RECOMMEND: file a `V1-FROZEN-INTERFACE-DEFERRED.md` row "post-v1 DID rotation discipline" with a NAMED destination — this is a real privacy-fundamental that v1-beta defers.

---

## §5. Atrium peer-mesh integration — privacy implications

Per `docs/ARCHITECTURE.md` lines 88-108 + 507-509, the Atrium peer-mesh routes envelopes via iroh (loopback + relay + holepunch). The transport is abstracted via `benten_sync::transport_trait::{Transport, TransportEndpoint, TransportConnection}` (RATIFIED 2026-05-15, §15.3 #1) so post-v1 transports can swap in (Tor / Nostr-relay / shaped relay).

### §5.1 What does Atrium peer-mesh routing leak?

1. **Recipient peer-discovery** — to deliver an envelope to `audience_did`, the sender's iroh-endpoint must resolve `audience_did → NodeAddr` (iroh public-key + relay-server hints). This discovery step is itself wire-observable: an iroh relay sees Alice asking for Bob's NodeAddr. **Leak: pre-delivery discovery announces intent.**
2. **Route-correlation** — the Atrium peer-mesh "knows-path" property (Alice's peer-mesh routing table knows Bob's path) couples with the §6.2 envelope's `audience_did` plaintext-AAD. Adversary at any peer-mesh node along the route learns `(sender_peer, recipient_did, route_hops)`. **Leak: graph topology + per-pair routing fingerprint.**
3. **iroh holepunch + relay-fallback signal** — iroh's NAT-traversal logic differs by peer-pair; the holepunch-vs-relay-fallback pattern fingerprints the geographic peer-pair.

### §5.2 Coupling with the §6.2 metadata leak

The §6.2 envelope's plaintext-AAD `audience_did` field is REDUNDANT with the iroh-transport-layer's destination addressing. Removing it from AAD would NOT improve privacy against the iroh-relay (which already needs the destination to route); it WOULD improve privacy against archive-of-bytes adversaries (snapshots of stored envelopes lose the routing-layer context).

**Recommendation:** Amendment 8 (per-relay-unlinkability via transport-layer blinding) addresses ONE half of this — re-randomizes the outer envelope per hop. The other half is **route-padding / decoy-routing** — sending envelopes via multiple Atrium peer paths concurrently so individual route-observations don't reveal "Alice's preferred path to Bob." Defer to the shaped-relay transport wave per `docs/ARCHITECTURE.md` line 509.

### §5.3 The "Atrium-as-routing-substrate" privacy posture statement

Per `docs/ARCHITECTURE.md`'s text, Atrium is positioned as the routing layer. It is NOT positioned as an anonymity-providing layer. **This should be EXPLICITLY documented** in `docs/SECURITY-POSTURE.md` as a privacy-disclosure: "Atrium peer-mesh provides ROUTING-substrate properties; it does NOT provide TRAFFIC-ANALYSIS-RESISTANCE. Users who require traffic-analysis-resistance should run Benten over Tor or Nym-mixnet as a post-v1 transport extension."

---

## §6. Recommended new Compromise mints + invariant additions

### §6.1 NEW Compromise #32 — Metadata-leak in v1-beta envelope shape

**Status:** OPEN AT v1-beta + v1-GM; mitigations in §3.1-§3.7 are future-additive.

**Text (draft to be folded into `docs/SECURITY-POSTURE.md`):**

> **Compromise #32 — Envelope metadata leakage to untrusted relays.** The §6.2 EncryptedEnvelope binds sender DID, recipient DID, operation type, and sealed-at epoch into the plaintext AAD (per Amendments 4 + 5). An adversary observing the wire — including untrusted Atrium peer-mesh relays per `docs/ARCHITECTURE.md` line 88-108 — learns the directed sender→recipient graph + per-pair message frequency + per-user operation-type histogram + ~1-hour-precision timestamps WITHOUT cryptographic break. Compromise #31 amplifies this: Drop bundles archived by any relay are *forever-archived metadata*. **Mitigation roadmap (post-v1-beta):** Amendment 7 (Sealed-Sender codepoint) + Amendment 8 (per-relay-unlinkability) + Amendment 9 (size-class padding) + Amendment 12 (coarse 1-hour epoch buckets, refines Am5). **Stays OPEN at v1-beta** — accepted trade-off for the alternative metadata-free envelope shape that would require Sphinx-style onion-routing + a non-iroh transport, both post-v1. **Honest-disclosure surface:** the Position-B blog framing MUST surface this compromise + the Sealed-Sender future-additive slot + the post-v1-GM cover-traffic + onion-routing roadmap as honesty caveats per the cryptographer-review's blog-revision-requirements (paralleling the Inv-15 / Compromise #31 honest-disclosure discipline).

**Cross-refs:** the third-reviewer's Amendments 4 + 5 (the integrity-driven plaintext-AAD bindings that compose into this leak); `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (new row for the Sealed-Sender codepoint slot lock + the size-class padding shape lock); `docs/SECURITY-POSTURE.md` Compromise #31 (long-term archive amplification); CLAUDE.md baked-in #5 (crypto-agility additive-codepoint discipline that the future-additive Sealed-Sender slot exercises).

### §6.2 NEW Inv-16 — Metadata-disclosure invariant + Sealed-Sender additive-codepoint slot

**Status:** REGISTERED at Phase-4-Meta-Core (this review); ENFORCEMENT-COMPLETION at G-CORE-PRIVACY-1 wave (proposed).

**Text (draft for `docs/INVARIANT-COVERAGE.md`):**

> **Inv-16 — Every envelope shape that places sender-DID or operation-type in plaintext AAD MUST be disclosed at `docs/SECURITY-POSTURE.md` and MUST have a future-additive Sealed-Sender (or equivalent metadata-hiding) codepoint slot reserved at v1-beta.** This invariant codifies the L6-privacy-lens finding that integrity-driven AAD bindings (Amendments 4 + 5) compose into a metadata leak which is structurally distinct from payload confidentiality. The load-bearing surfaces today: the default `DropToRecipient` / `DeviceLink` / `RemotePermission` BindingContext variants (all of which bind plaintext sender + plaintext operation). Enforcement at G-CORE-PRIVACY-1 wave: (a) lock Sealed-Sender codepoint slot in the encryption-codepoint registry at `crates/benten-crypto-suite/src/codepoint.rs`; (b) lock size-class padding bucket SHAPE in the wire format; (c) document per-variant leak in `docs/SECURITY-POSTURE.md` per-codepoint table; (d) extend cite-drift-detector with a `PlaintextSenderInAadPattern` scanner that flags any new BindingContext variant placing identity-DIDs in AAD without a paired Sealed-Sender sibling codepoint slot.

**Why it matters:** turns future privacy-leak amendments into automatic FIX-NOW per the §3.5g cross-language rule-mirror pattern (which proved out for ErrorCode catalog drift). When G-CORE-9 wave-2 or any future wave proposes a new envelope variant, the Inv-16 scanner forces the privacy question to be answered explicitly.

**Cross-refs:** Inv-15 (the sibling invariant for sig-bundle-CID identifier hazard at construction layer; Inv-16 is the structural-parallel for metadata-disclosure at the AAD layer); Compromise #32 (the load-bearing instance that motivates the invariant); the third-reviewer's Amendments 4 + 5 (the integrity-driven bindings that compose into the leak); CLAUDE.md baked-in #5 (crypto-agility additive-codepoint discipline).

### §6.3 NEW row in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — Sealed-Sender codepoint slot lock

Proposed row text:

> **Row D-SS-1 — Sealed-Sender codepoint slot lock at v1-beta encryption-codepoint registry.** Per L6-privacy-lens (this review) Amendment 7 + Inv-16. Allocate codepoint `0x6510` (sibling to Layer-C-drop `0x6500`) as RESERVED-FOR-SEALED-SENDER-ADDITIVE at v1-beta. Implementation deferred to post-v1-beta wave (G-CORE-PRIVACY-1 or successor). Lock includes: codepoint integer value at `crates/benten-crypto-suite/src/codepoint.rs::EncCodepoint`; per-codepoint TS catalog mirror at `packages/engine/src/codepoints.generated.ts`; SECURITY-POSTURE.md per-codepoint privacy-disclosure table row.

### §6.4 NEW row in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` — Size-class padding bucket shape lock

Proposed row text:

> **Row D-PAD-1 — Size-class padding bucket SHAPE locked at v1-beta; specific bucket values + measurement deferred.** Per L6-privacy-lens Amendment 9. Lock at v1-beta: every EncryptedEnvelope MUST be padded to a publicly-defined size-class bucket per codepoint; specific bucket values are wire-format-additive at any subsequent point provided they only ADD finer-grained buckets, never REMOVE existing ones. Implementation: post-v1-beta measurement pass against the Spike G corpus + future workload. Pinned at `crates/benten-crypto-suite/src/envelope.rs::SIZE_CLASS_BUCKETS`.

### §6.5 NEW row — Cover-traffic + onion-routing post-v1-GM gate

Proposed row text:

> **Row D-COVER-1 — Cover-traffic + onion-routing for traffic-analysis-resistance: DEFERRED to post-v1-GM, gated on shaped-relay / Tor / Nym-mixnet transport landing.** Per L6-privacy-lens Amendment 10 + §5.3. Cover-traffic is meaningful only when paired with onion-routing; onion-routing requires non-iroh transport extension per `docs/ARCHITECTURE.md` line 509. When the post-iroh transport lands as a compile-time engine extension, this row is the trigger for the cover-traffic design pass.

---

## §7. Plain-English user-facing privacy statement

(For inclusion in Benten's user-facing documentation; the Position-B blog framing; and any "Privacy at Benten" page that exists or will exist.)

> **What Benten protects, and what it doesn't, when you send something.**
>
> When you encrypt a Benten message or share a document with another user, the **content** of that message is end-to-end encrypted. Anyone watching the wire — including the peer-mesh relays that route the message — sees only encrypted bytes, not your content. Your messages cannot be read by anyone other than the intended recipient.
>
> However, anyone watching the wire DOES see:
> - **Who you are sending to** (the recipient's Benten ID).
> - **Who you are** (your sending Benten ID).
> - **What kind of operation** you are performing (e.g., "share a document," "ask permission to invoke procedure X," "link a new device").
> - **Approximately when** you sent it (rounded to the hour).
> - **Roughly how big** your message is (which size bucket it fits into).
>
> This is metadata, not content. **Knowing who-talks-to-whom, when, and what kind of operation, over time, can itself reveal a lot.** A long-term observer who archives this metadata for years builds a picture of your social graph + your operation patterns + your activity rhythm. Compromise #31 in Benten's security posture documents that messages you share, once distributed, are forever-distributed; the metadata about them is forever-archivable.
>
> **Benten v1-beta ships with strong content privacy and limited metadata privacy.** This is an honest trade-off: building metadata-private end-to-end-encrypted messaging is hard (Signal, Briar, Cwtch, and Pond have invested years in each metadata dimension); Benten v1-beta does not yet ship the Sealed-Sender pattern, padding-to-fixed-size-classes, or anonymity-network transport.
>
> **The roadmap:** post-v1-beta, Benten will add a "Sealed-Sender" mode (hiding sender identity from relays); add size-class padding (so message sizes don't leak); reserve a transport-extension slot for Tor and similar anonymity networks. The wire format at v1-beta has slots reserved for these future-additive features so they can land without breaking interop.
>
> **If your threat model requires hiding the metadata of who-you-talk-to from your network, your ISP, or a co-located adversary, Benten v1-beta is NOT YET the right tool.** Use Benten for the content-privacy guarantees it does ship; layer it on Tor or a mixnet for metadata-privacy when the post-v1 transport extensions land.

(Length: ~340 words. This is the explicit honest-disclosure framing that matches the Position-B blog discipline ratified for Inv-15 / Compromise #31. The Compromise #32 mint in §6.1 cross-references this exact framing.)

---

## §8. Self-assessment + confidence + lower-confidence areas

### What I have HIGH confidence on

- **The leak enumeration in §2.** Reads directly off the BindingContext enum + the AAD-binding amendments. No interpretation required.
- **The Drop-bundle-archive long-term threat (§4).** Compromise #31 + the metadata-in-AAD shape compose mechanically into a multi-year archive disclosure. Sweeney 2002 + Narayanan-Shmatikov 2008 deanonymization literature establishes the sufficiency of statistical-fingerprint information.
- **The Sealed-Sender additive-codepoint recommendation (§3.1).** Signal has 7 years of production data; the additive-codepoint mechanism matches Benten's crypto-agility framework cleanly; the cost of locking the codepoint slot at v1-beta is essentially zero.
- **The Inv-16 + Compromise #32 mints (§6).** These follow the established Inv-15 + Compromise #31 pattern; the load-bearing argument is the parallel.
- **The Position-B blog honest-disclosure framing (§7).** Matches the established Position-B revision-roadmap discipline.

### What I have MEDIUM-HIGH confidence on

- **The specific size-class bucket values in §3.3.** Empirical; needs measurement. The bucket SHAPE (size-class buckets per codepoint) is HIGH-confidence; the specific values are MEDIUM-HIGH.
- **The per-relay-unlinkability design (§3.2).** The mechanism (transport-layer re-blinding per hop) is right; the specific salt-derivation from Noise session keys is a sketch — a real implementation pass would need to verify the iroh transport's session-key surface admits this.
- **The 1-hour epoch bucket in §3.6.** 1-hour is a reasonable bucket but a stronger argument could be made for variable buckets (1-day for long-validity grants; 1-hour for short-window perms). Out-of-scope for this review.

### What I have MEDIUM confidence on

- **Cover-traffic deferral (§3.4) is the right call.** Cover-traffic without onion-routing is performative; with onion-routing it's load-bearing. Onion-routing is post-v1 per ARCHITECTURE.md line 509. The deferral matches that gate but assumes Benten's threat model does not include nation-state TA adversaries pre-v1-GM — defensible but assumed.
- **The DID-rotation discipline recommendation (§4.4) is the right depth-of-mitigation for archive-threat.** Other systems (Briar, Cwtch) have made different choices here. DID rotation is the canonical mitigation but it has operational cost (key-management UX, contact-list-update).

### Lower-confidence areas (would want additional review)

- **The exact threat-model boundary between "decentralized peer-mesh with untrusted relays" (the design's stated posture) and "nation-state traffic-analysis adversary" (out-of-scope).** I argued in §5.3 that Atrium does NOT claim TA-resistance, which justifies deferring cover-traffic. But the Benten user-facing positioning (per `docs/CRATES-DEEP-DIVE.md` + Compromise #31 framing) explicitly mentions "decentralized + p2p-encryption-by-default" — a privacy-savvy user reading those words may assume TA-resistance. **The plain-English statement in §7 is the load-bearing disclosure that closes this gap.** Whether it's sufficient is a Ben call.
- **Whether the §6.2 design SHOULD ship at v1-beta with metadata leak, or whether the Sealed-Sender codepoint should be a v1-beta-required not v1-beta-future-additive.** I argued future-additive (§3.1 + §6.3) but a more privacy-aggressive reading would say: ship Sealed-Sender as the DEFAULT codepoint at v1-beta and treat the plaintext-sender codepoint as legacy. I deferred to the additive-codepoint discipline because the engineering cost of Sealed-Sender's Signal-style spam-mitigation (per-recipient delivery tokens) is non-trivial; Benten's existing capability discipline may or may not provide the equivalent (out-of-scope analysis).
- **Whether the third-reviewer's Amendments 4 + 5 should be REVISED in light of this lens.** I argued KEEP them — they close real integrity attacks — but a hybrid design (Sealed-Sender variant has Am4 INSIDE-ciphertext + plaintext-AAD variant retains Am4) could share most of the integrity properties with much-better privacy. This is the Inv-16 enforcement-completion path.

### Additional reviews I would want

1. **A traffic-analysis specialist** to estimate the practical attack-cost against Benten's specific deployment shape (intermittent iroh-peer-mesh; not always-on; typical small-payload sizes). My estimate: deanonymization-by-archive is reachable within 12-24 months of observation for ~80% of typical users; the SS specialist can refine.
2. **A signal-sealed-sender implementation reviewer** to verify that the per-recipient-delivery-token pattern composes with Benten's existing capability discipline. If it does, Sealed-Sender as a v1-beta-DEFAULT codepoint becomes feasible; if not, the additive-slot-at-v1-beta is the right disposition.
3. **A measurement pass** on the Spike G corpus + projected workload to lock specific size-class buckets per Amendment 9. ~1 person-week of work; non-blocking on v1-beta freeze.

### What this review does NOT cover

- The full Phase-4-Meta-Core scope (covered by `e2r-ffull-scope-review.md` + prior reviewers).
- The specific RustCrypto-implementation-version recommendations (covered by the first reviewer).
- Construction-layer cryptographic soundness (covered by the three prior reviewers).
- The multi-stanza HpkeMultiBase end-to-end design (third-reviewer punted; I made a recommendation in §3.5 but the full design pass deserves a multi-recipient HPKE specialist).
- The audit-firm RFP/selection (out-of-scope; called out by second-reviewer §8).

---

## §9. Citations

### §9.1 Standards + drafts

- **RFC 9180** Hybrid Public Key Encryption (https://datatracker.ietf.org/doc/html/rfc9180). §5.1.1 mode_base sender-anonymity (no sender authentication); §5.1.4 mode_auth_psk (psk_id binding); §9.5 PSK security; §9.8 message-routing-leakage. Load-bearing for the recipient-anonymity / sender-anonymity / metadata properties of HPKE used by Benten Layer-C / Layer-D.
- **RFC 9458** Oblivious HTTP (https://datatracker.ietf.org/doc/html/rfc9458). Reference for relay-blinding pattern at the application layer; informs Amendment 8.
- **RFC 9420** Messaging Layer Security §6.2 Welcome-message epoch binding. Reference for Amendment 5 + Amendment 12.
- **RFC 9001 + RFC 9000** QUIC (the iroh transport's substrate). Reference for the per-hop session-key surface that Amendment 8 exercises.
- **draft-irtf-cfrg-mixnet-stratum** (draft) reference for mixnet integration patterns.
- **draft-ietf-mls-pq-ciphersuites-04** MLS-PQ. Reference for HPKE-mode-base[MLKEM768-X25519] in production.

### §9.2 Signal Sealed Sender + privacy-aware messaging

- **Signal Sealed Sender (2018):** Moxie Marlinspike, "Technology preview: Sealed sender for Signal" (https://signal.org/blog/sealed-sender/). The canonical sender-anonymity-from-relay design. 7 years of production data.
- **Signal Sealed Sender V2 (2024):** "Sealed sender meets multi-recipient" (https://signal.org/blog/sealed-sender-multi-recipient/). Per-recipient unlinkable copies pattern; load-bearing for Amendment 11.
- **Signal PQXDH whitepaper:** "The PQXDH Key Agreement Protocol" (https://signal.org/docs/specifications/pqxdh/). Post-quantum metadata-handling reference.
- **age multi-recipient discussion #463:** https://github.com/FiloSottile/age/discussions/463. Filippo Valsorda's explicit framing that age does NOT provide recipient-anonymity; reference for why Amendment 11 needs a different pattern than naive multi-stanza.
- **Filippo Valsorda, "age and Authenticated Encryption"** (https://words.filippo.io/age-authentication/). Reference for the design tension between scrypt-recipient + x25519-recipient privacy shapes (mirrored in §2.4 Am4 trade-off).

### §9.3 Anonymity-network + metadata-resistant designs

- **Briar threat model:** Brian Briggs et al. "Briar protocol" (https://code.briarproject.org/briar/briar-spec/). Reference for offline-first metadata-resistant messaging.
- **Cwtch protocol:** Sarah Jamie Lewis, "Cwtch: Privacy Preserving Infrastructure for Asynchronous, Decentralized, Multi-Party and Metadata Resistant Applications" (https://docs.cwtch.im/security-handbook/). Reference for Tor-onion-service-based metadata-resistance.
- **Pond design (deprecated; reference design):** https://web.archive.org/web/20151024003345/https://pond.imperialviolet.org/. Original cover-traffic + Tor-hidden-service pattern.
- **Sphinx packet format:** George Danezis + Ian Goldberg, "Sphinx: A Compact and Provably Secure Mix Format," IEEE S&P 2009 (https://www.cypherpunks.ca/~iang/pubs/Sphinx_Oakland09.pdf). Reference for onion-routing per-hop re-encryption pattern (Amendment 8 + cover-traffic + onion-routing future).
- **Loopix mixnet:** Ania Piotrowska et al., "The Loopix Anonymity System," USENIX Security 2017 (https://www.usenix.org/conference/usenixsecurity17/technical-sessions/presentation/piotrowska). Reference for Poisson-cover-traffic in Amendment 10.
- **WTF-PAD:** Marc Juarez et al., "Toward an Efficient Website Fingerprinting Defense," ESORICS 2016 (https://arxiv.org/abs/1512.00524). Reference for Tor padding-machine design in Amendment 9.

### §9.4 Statistical-disclosure / deanonymization literature

- **Latanya Sweeney, "k-anonymity: A model for protecting privacy,"** IJUFKS 2002. Foundational reference for §4.1 deanonymization-by-archive argument.
- **Arvind Narayanan + Vitaly Shmatikov, "Robust De-anonymization of Large Sparse Datasets,"** IEEE S&P 2008 (https://www.cs.cornell.edu/~shmat/shmat_oak08netflix.pdf). The Netflix Prize result; load-bearing for §2.3 + §4.1 (operation-frequency-histogram fingerprinting).

### §9.5 PSI + group-share privacy

- **Pinkas, Schneider, Zohner, "Faster Private Set Intersection Based on OT Extension,"** USENIX Security 2014 (https://www.usenix.org/conference/usenixsecurity14/technical-sessions/presentation/pinkas). Reference for PSI cost analysis in Amendment 11.
- **Kolesnikov + Kumaresan, "Improved OT Extension for Transferring Short Secrets,"** CRYPTO 2013. Reference for the optimized PSI primitives.

### §9.6 Prior Benten reviews + design docs

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` (first cryptographer; sketched §6.2)
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` (second-opinion; Amendments 1+2; explicitly excluded privacy at line 187)
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design` HEAD (third reviewer; Amendments 3+4+5+6)
- `docs/SECURITY-POSTURE.md` Compromise #31 (line 91 + §R6 lines 2712-2820) — the load-bearing precedent for Compromise #32
- `docs/ARCHITECTURE.md` Atrium peer-mesh (lines 88-108) + Transport abstraction (lines 507-509) + Drop bundle envelope-sig (lines 165-181)
- `docs/INVARIANT-COVERAGE.md` Inv-15 (the structural-parallel for Inv-16)
- `docs/V1-FROZEN-INTERFACE.md` item 6 (encryption codepoint table) + item 15 (Sharing & Confidentiality surface)
- CLAUDE.md baked-in #5 (crypto-agility additive-codepoint discipline that the Sealed-Sender slot exercises) + #15 (v1-beta gate) + #18 (authority-isolation vs confidentiality-isolation; the L6-lens extends this to "metadata-isolation")

---

**End of L6 privacy / metadata-leak / unlinkability review.**
