# MembershipSet P2 (C-3) — iroh-gossip topic-id-privacy seam

**Author:** orchestrator-dispatched SENIOR PRIVACY-ENGINEER + P2P-NETWORKING SPECIALIST on branch
`phase-4-meta-core/membership-set-p2-c3-iroh-gossip-privacy`.
**Date:** 2026-05-28.
**Brief framing:** resolve the **MCV2-C-3 contradiction** (M-C2 v2 critique `0533d0cf` §3.3)
and mint / consolidate the **Compromise #59 STRONGLY-RECOMMENDED** disclosure
surfaced by M-C3 v2 fresh-eyes cryptographer (`cac10631` §3.1).
The threat: iroh-gossip topic-ids are PUBLIC at the transport layer; a naïve
`topic_id = membership_set_id` derivation lets a passive network observer / relay
operator enumerate (topic_id, subscriber-set, publication-timing) tuples and
fingerprint every MembershipSet — defeating Inv-20 clause-d per-recipient
unlinkability at the TRANSPORT layer, even though the per-stanza HPKE envelopes
preserve it at the CRYPTOGRAPHIC layer. Worse: forks (FORK-ONLY rotation, F19
generation-CRDT-vector) mint fresh `membership_set_id`s but the
`parent_membership_set_id` chain is observable from the topic-id rotation
pattern itself, so the adversary rebuilds the fork tree purely from
topic-id history. Per Ben's Q1 SCOPE-IN ratification, Wave-MS-TRANSPORT
ships iroh-gossip impl at v1-beta, so this is no longer a Phase-5+ concern —
it must be resolved AT v1-beta-Core.

---

## §1 Executive recommendation + confidence

**TL;DR (one paragraph).** Adopt a **hybrid 3-defense composition** as the
canonical topic-id privacy substrate for `GossipPlusBlobs` and
`GossipRequiredBlobsFallback` transport variants. The 3 defenses, in
descending importance: **(D1)** HMAC-blinded topic-id —
`topic_id = HMAC-SHA256(K_Set, "benten/gossip/topic/v1" ‖ membership_set_id ‖ generation)`
truncated to 32 bytes (the iroh-gossip TopicId wire size) — only members who
hold `K_Set` can derive it, relays see a meaningless 32-byte string per
MembershipSet, and **the predecessor chain is unrecoverable from the topic-id
alone** because each new generation re-keys with the post-fork K_Set (which
already rotates FORK-ONLY per Inv-20 clause-b); **(D2)** epoch-bound rotation
piggybacks the EXISTING FORK-ONLY K_Set rotation cadence (no new rotation
machinery, no new wave-day cost beyond the derivation function itself); **(D3)**
out-of-band rendezvous — peers learn `(membership_set_id, K_Set, generation)`
ONLY via the UCAN-signed `DropBundle` admission flow (per F4 + Inv-20 clause-a
multi-stanza HPKE-Encap), NEVER via gossip discovery / DHT publication / public
rendezvous server. The composition closes MCV2-C-3 completely at the
cryptographic layer; residual disclosure narrows to (a) the subscriber-set
intersection (relay still sees WHICH peers subscribe to WHICH blinded-topic,
which is structurally inherent to any pub/sub overlay short of mixnet routing)
and (b) the relay-observable existence of *some* private group per blinded
topic, which is k-anonymous against the entire population of all Benten
MembershipSets sharing the relay. We DEFER 3 alternatives NAMED-NOW: sharded
multi-topic-per-set (heavy + bandwidth-amplifying), onion/mixnet relay layering
(post-v1-beta Phase-5+ Kith watch-list), and per-recipient unicast over QUIC
(degenerate; defeats the whole reason gossip exists). **The chosen shape adds
~0.4–0.6 wave-days to Wave-MS-TRANSPORT and closes Compromise #59 as
MITIGATED (not merely DISCLOSED).**

**Confidence breakdown:**
- §1 chosen-shape soundness (HMAC-PRF blinding + FORK-ONLY epoch rotation +
  out-of-band rendezvous): **HIGH (~88%).** Reuses the SAME HMAC-SHA256-on-
  fixed-input-keyed-by-K_Set PRF regime that Q3 Option D (`23f76e24`) already
  validated for `plaintext_cid_atrium`; the cryptographic primitive is
  identical, the threat model maps 1:1 (storage host : equality-oracle
  ↔ gossip relay : topic-fingerprint-oracle), and the keying-context
  separation via the `"benten/gossip/topic/v1"` domain-separation prefix
  defeats cross-context PRF correlation per NIST SP 800-108-style discipline.
- §1 fork-tree-unrecoverability claim: **HIGH (~85%).** When Atrium A forks
  to A' with FORK-ONLY K_Set rotation, `K_Set` changes; the HMAC-blinded
  topic-id changes to a value that has no observable cryptographic relation to
  the parent's topic-id (HMAC is a PRF, so outputs across keys are
  indistinguishable from random). The `parent_membership_set_id` chain is
  ENVELOPE-LAYER metadata (visible to members who decrypt; invisible to gossip
  relays) — the topic-id rotation alone cannot reconstruct it.
- §1 subscriber-set intersection residual: **HIGH (~92%).** This is a
  structural property of any non-mixnet pub/sub overlay (Waku 2024 analysis
  documents the same for libp2p GossipSub k-anonymity). The honest disclosure
  is correct; the bound is k-anonymous against the population of all peers
  sharing the relay.
- §1 wave-day delta (~0.4–0.6): **HIGH (~88%).** Derivation function is ~30
  LOC + ~50 LOC tests + 1 RED-PHASE pin per pim-12; integration into the
  Wave-MS-TRANSPORT `gossip_topic.rs` shape (M6 §10.2 item 13) per existing
  M6 cost-line; no new dep; no new error class beyond E_TRANSPORT_KIND.
- §1 fork-tie-break composition with N4 R-N4-3a: **MEDIUM-HIGH (~78%).**
  Concurrent forks A→A1 + A→A2 derive DIFFERENT topic-ids
  (each from its post-fork K_Set); the losing-fork's topic stays alive for
  the audit-archive but stops receiving new traffic per Inv-21
  fork-tie-break HARD partition. Confidence MEDIUM-HIGH because the
  interaction with iroh-gossip's `NeighborDown` 30-35 s detection lag
  (M6 §3.1 footgun) creates a brief window during which both A1 and A2
  topic-ids carry live traffic — survivable per F19 dedup at envelope
  layer + Inv-20 clause-e generation-CRDT-vector but adds an e2e
  pin requirement (per pim-2).
- §1 K_Set-leak residual posture: **MEDIUM (~70%).** If K_Set leaks
  (Compromise #48 class — MembershipSet-shape-leak at K_Set compromise
  threshold), the adversary CAN now recompute past blinded topic-ids and
  reconstruct the historical (topic_id, subscriber-set, timing) fingerprint.
  This is **equivalent threshold** to Compromise #48 (already disclosed); we
  do NOT introduce a NEW threshold, we extend the existing one to cover
  topic-id-history-disclosure as a clause of #48. **Recovery path is
  FORK-ONLY rotation** per existing Inv-20 clause-b discipline.

**Three load-bearing conclusions:**

1. **The fix is MECHANICALLY IDENTICAL to Q3 Option D's storage-host
   defense, just at the transport layer instead of the addressability layer.**
   Same primitive (HMAC-SHA256 PRF), same key (K_Set / K_Atrium are the
   same key in M-CONS-v2 per N2 absorption), same threat model class
   (passive adversary holds blinded fingerprint; cannot probe without
   key). The architectural elegance is that we are NOT introducing a new
   defense; we are EXTENDING an already-ratified defense (Option D) to a
   second surface (transport-layer topic-id fingerprint) it naturally
   covers under the SAME `K_DedupScope` parameter framing (Q3 §1 third
   conclusion: "Treat K_Atrium as one instance of a general `K_DedupScope:
   HMAC-PRF-key` parameter"). This is the **single elegant structural
   shape** demanded by `feedback_extra_reflection_pass_for_elegant_permanent_shape`
   — one parameter, two surfaces, both blinded.

2. **iroh-gossip's "no per-topic membership ACL" design choice (Spike C
   footgun + M6 §3.1 + n0-computer iroh discussion #3168) becomes a NON-
   ISSUE under blinded topic-ids**, because the topic-id itself becomes the
   shared-secret-equivalent: knowing a topic-id implies either (a) you are
   a current K_Set holder (= member), OR (b) you guessed a 256-bit
   value (= ~2^256 brute-force margin). The "authorization via knowing the
   topic" anti-pattern that the n0-computer team explicitly DISLIKED
   (discussion #3168: "we prefer keeping public keys public and authorizing
   via some other mechanism") is structurally avoided because the topic-id
   IS NOT the authorization token — admission is gated by UCAN-signed
   DropBundle that distributes K_Set via multi-stanza HPKE-Encap per Inv-20
   clause-a. Knowing the topic-id is a CONSEQUENCE of admission, not a
   SUBSTITUTE for it.

3. **The hybrid composition (D1 + D2 + D3) does not require a Phase-5+
   delay** — it is implementable AT v1-beta within the Wave-MS-TRANSPORT
   ~5–8 wave-day envelope per M-CONS-v2 §7.4, adding ~0.4–0.6 wave-days
   (derivation function + tests + 1 e2e pin). The alternative — disclose
   #59 honestly and defer mitigation to Phase-5+ — would leave Inv-20
   clause-d structurally violated at the transport layer for the v1-beta
   shipping window, which is **incompatible with Ben's Q1 ratification
   intent** (the SCOPE-IN was to ship working privacy-respecting gossip,
   not to ship gossip that breaks an unlinkability invariant). Mitigating
   at v1-beta is therefore not optional.

---

## §2 Task 1 — iroh's prior art + literature review

### §2.1 iroh-gossip topic-id model (n0-computer official)

Sources: `docs.iroh.computer/connecting/gossip` (WebFetch 2026-05-28),
`github.com/n0-computer/iroh-gossip` README + `proto` rustdoc, n0
iroh-workshop-39c3 walkthrough.

- **TopicId = 32-byte identifier**, namespace for all gossip-protocol
  messages. "Topics are separate swarms and broadcast scopes."
- **Construction guidance (n0 official):** "You can pick any value you like
  for a topic ID, but it's recommended to use a cryptographic hash of a
  meaningful string to avoid collisions with other applications, and it's
  suggested to use a stable hash function to derive topic IDs from
  human-readable strings." Example: `sha256("com.example.myapp.mytopic")`.
  **No mention of privacy.** The construction recommendation is purely a
  collision-avoidance pattern, not a confidentiality discipline.
- **Bootstrap model:** "To join a gossip topic, you need to provide a list
  of bootstrap peers that are already members of the topic." Sources of
  bootstrap NodeIDs: hardcoded lists / config / discovery services / endpoint
  tickets / public rendezvous servers.
- **Protocol layering (iroh-gossip ≥0.93):** HyParView (membership) + PlumTree
  (broadcast). HyParView maintains a partial view of nodes in the swarm
  per-topic; PlumTree's eager-push/lazy-push tree carries IHAVE / IWANT
  message-hash-only metadata (the hashes uniquely identify messages but
  not the contents — content is fetched via IWANT). Per-topic mesh
  topology emerges from the HyParView active-passive split.
- **Privacy posture in iroh docs:** the iroh `docs.iroh.computer/deployment/
  security-privacy` page covers IP-address-leakage during direct connection
  + relay metadata (relays see (NodeID, connection-list) tuples even when
  e2e content is hidden); does NOT cover topic-id privacy.

### §2.2 iroh community discussion: authorization in iroh-gossip

Source: `github.com/n0-computer/iroh/discussions/3168` (WebFetch 2026-05-28).

**Key finding for our threat model.** The n0-computer team has an
EXPLICIT preference AGAINST secrets-based authorization for gossip topics:

> Maintainers expressed reluctance toward "secrets-based authorization,"
> preferring instead to "keep the public keys public, use iroh's built-in
> client authentication and authorize the public key via some third other
> mechanism," acknowledging this remains use-case-dependent.

This is INFORMATIVE FOR OUR DESIGN, NOT BLOCKING. The n0 team's framing is
that **iroh-gossip itself should not be in the business of doing access
control** — applications layer auth on top. Our design (HMAC-blinded
topic-id keyed by K_Set distributed via UCAN-admission) respects this
framing: iroh-gossip sees a 32-byte topic-id and treats it as just-another-
topic; the cryptographic privacy guarantee lives entirely in the
benten-membership-set layer ABOVE iroh-gossip. **iroh-gossip is the
substrate; the privacy discipline is the application layer.** This aligns
with CLAUDE.md baked-in #18 application-layer-composition-as-strong-default
discipline (per `feedback_engine_primitives_vs_application_layer.md`).

### §2.3 Distributed-topic-tracker pattern (rustonbsd 2025-09)

Source: `rustonbsd.github.io/2025/09/03/distributed-topic-tracker.html`.

A community-contributed pattern uses Mainline BitTorrent DHT to publish
bootstrap-peer info for an iroh-gossip topic. **Notable for our threat
model: the DHT publication is by design PUBLIC.** This is the
ANTI-PATTERN for us — under blinded topic-id, the topic-id itself must
NEVER appear in any DHT, rendezvous server, ticket, or other public
discovery surface. The bootstrap-peer info should be exchanged via the
SAME UCAN-signed Drop / admission channel that distributes K_Set, NOT via
DHT lookup. Our design uses out-of-band rendezvous (D3) precisely to
avoid this leak vector.

### §2.4 Waku content-topic k-anonymity model

Sources: `docs.waku.org/learn/concepts/content-topics/`,
`research.logos.co/rlog/wakuv2-relay-anon/` (Vac Research, redirected
from `vac.dev`), arxiv 2207.00038 (Waku family-of-protocols paper).

Waku is the closest production analog: pubsub overlay (libp2p
GossipSub-based) with explicit privacy goals + a formal k-anonymity
analysis. Key findings:

- **Receiver anonymity via single-pubsub-topic-with-many-content-topics:**
  Waku Relay achieves k-anonymity where k = number of content-topics
  multiplexed onto a single libp2p pubsub topic. An attacker can link
  receivers to content-topics with maximum certainty 1/k.
- **Sharding caveat:** "If done wrongly, such sharding of pubsub topics
  can breach anonymity." Sharding without coordination DEGRADES
  k-anonymity by partitioning the population.
- **Explicit guidance on topic-string content:** "**Avoid PII** in topic
  names (e.g., public keys). **Use Protocol Buffers** with optional
  fields to multiplex functionality within single topics. **Leverage
  functional switching** rather than creating separate topics per feature."
- **Future work item (still open):** "Anonymous Filter Subscription" and
  "Anonymous Query" — Waku itself acknowledges that the current shape
  leaks content-topic interest to peers, and is researching mitigations.
- **AS-level / global passive observer:** Waku Relay does NOT defend
  against AS-level + multi-node coordinated attackers — sender-message
  unlinkability breaks under that adversary class. This is a
  STRUCTURAL property of non-mixnet pub/sub; matches Benten's residual
  disclosure (§6 below).

**Translation to Benten.** Waku's k-anonymity-via-multiplexing pattern
does NOT apply directly to Benten because each Benten MembershipSet
genuinely is its own broadcast scope (cannot share a "global benten-traffic"
pubsub topic without inverting the privacy guarantee). But the
TOPIC-STRING-CONTENT guidance ("avoid PII") applies directly: our
blinded topic-id contains no human-readable substring, no DID, no
plaintext membership_set_id — only HMAC-output bytes. We also adopt
Waku's **single-pubsub-topic-style framing** as a DEFERRED alternative
(§4.2 sharded multi-topic, NAMED-deferred).

### §2.5 Tor v3 onion service rendezvous (canonical prior art)

Source: `torproject.gitlab.io/torspec/rend-spec-v3.html` (WebFetch
2026-05-28).

Tor v3 is the canonical prior art for **blinded-identifier rendezvous
in adversarial overlay networks**. Three mechanics map cleanly to our
design:

1. **Blinded public key rotation per time period (default 1440 min / day).**
   Derivation: `A' = h*A` where `h = H(BLIND_STRING ‖ A ‖ s ‖ B ‖ N)` and
   `N = "key-blind" ‖ period_number ‖ period_length`. Each period yields a
   freshly-blinded key UNCORRELATED to prior periods to anyone lacking the
   credential.
2. **Descriptor lookup at HSDir uses the BLINDED key, not the onion
   address.** HSDirs cannot predict which descriptors they'll host or
   link them to specific onion addresses — they see only blinded-key-→-
   descriptor mappings.
3. **Rendezvous cookie:** client generates a random 20-byte cookie per
   connection, sends it via introduction point (encrypted); service
   reads it from INTRODUCE2 and connects to rendezvous point with the
   same cookie. **Cookie ≠ identity; cookie is a one-time linking token
   that the rendezvous point cannot interpret.**

**Translation to Benten.** Our D1 (HMAC-blind) is structurally **the same
primitive class** as Tor v3's blinded-key derivation — both transform a
stable identifier into an epoch-keyed pseudo-random identifier that ONLY
parties holding a shared secret can compute. Tor v3 uses scalar
multiplication on Ed25519 (because the underlying primitive needs to be a
public-key); we use HMAC-SHA256 (because the underlying primitive only
needs to be a PRF over a 32-byte output — symmetric is fine because
K_Set is already a per-set shared secret). Tor v3's rotation cadence is
TIME-BASED (per day); ours is FORK-BASED (per K_Set rotation = per
admin-kick / per member-leave). **Our cadence is correct for our threat
model** because (a) Benten Atriums are long-lived collaboration
substrates where daily rotation would unnecessarily fragment the
gossip-mesh topology, and (b) FORK-ONLY semantics are the existing
membership-set-mutation contract per Inv-20 clause-b — adding a
time-based rotation on top would create a SECOND rotation cadence that
violates the "one-rotation-class-per-key" elegance.

### §2.6 MLS RFC 9420 group_id metadata leak (acknowledged-class)

Sources: RFC 9420 §6 + RFC 9750 architecture document. MLS itself
acknowledges:

> MLS private message format leaks some metadata — namely the group ID
> and epoch counter. This metadata can be hidden by encrypting the
> entire outer MLS application message with a symmetric key derived from
> the other party's last known epoch.

**This is the SAME class of leak we are mitigating** — the structural
property that group-membership protocols inherently leak group-ID at
the transport layer, with the mitigation being SECOND-LAYER blinding by
re-encrypting with a key the relay doesn't have. The Benten design
applies an EQUIVALENT mitigation at the gossip-topic-id layer (the
"outer" layer in our stack), using the same K_Set that MLS would use
for the inner-message encryption. This is **convergent design** —
literature anchor for our choice.

### §2.7 Signal Sealed-Sender (the recipient-side limitation)

Sources: signal.org/blog/sealed-sender, Martiny-et-al NDSS 2021
"Improving Signal's Sealed Sender" (cs.umd.edu), Garman-Persichetti et
al arxiv 2305.09799 "No safety in numbers" on sealed-sender Signal
group traffic analysis.

**Critical finding.** Sealed-sender hides the sender_did from the relay,
but the Martiny et al NDSS 2021 work + Garman et al 2023 group-traffic-
analysis poster show that **delivery records (who receives) are
sufficient to reconstruct WHO COMMUNICATES WITH WHOM** via correlation
of delivery patterns to known social graphs.

**Translation to Benten.** This validates our **residual-disclosure
honesty** in §6: the subscriber-set intersection at the relay is a
structural disclosure that NO HMAC-blinding-of-topic-id defense can
eliminate. The blinded-topic-id closes the topic-fingerprint side of
the leak; it does NOT close the subscriber-correlation side. Honest
disclosure of the subscriber-correlation residual is the correct
posture; mitigating it would require onion/mixnet relay layering
(§4.3 NAMED-deferred).

### §2.8 Libp2p GossipSub + Episub privacy posture

Source: `github.com/libp2p/specs/tree/master/pubsub` + Raven
(`github.com/rairyx/raven`).

GossipSub itself does not provide topic-privacy primitives (consistent
with iroh-gossip's design philosophy). The Raven project demonstrates
that **Dandelion++-style two-phase routing CAN be layered on top of
GossipSub for sender-anonymity** — relevant for post-v1-beta sender-
anonymity work but NOT relevant for our topic-fingerprint defense
(Dandelion++ defends sender-IP, not topic-id).

### §2.9 Convergent-encryption / DupLESS literature (Q3 Option D anchor)

Per Q3 specialist `23f76e24` §2.1, the HMAC-PRF-blinded fingerprint
regime is well-trodden in the message-locked-encryption literature
(Bellare-Keelveedhi-Ristenpart 2013). The **storage-host equality-oracle**
threat that DupLESS defends is **structurally homomorphic** to the
**gossip-relay topic-fingerprint** threat we defend:

| Threat axis | Q3 Option D (storage host) | This doc P2-C-3 (gossip relay) |
|---|---|---|
| Attacker holds | (blinded_cid, ciphertext) tuples | (blinded_topic_id, ciphertext-stream-metadata) tuples |
| Attacker wants | "is candidate_plaintext stored?" | "is candidate_MembershipSet active?" |
| Attacker brute-force without K | 2^256 (HMAC key brute-force) | 2^256 (HMAC key brute-force) |
| Mitigation primitive | HMAC-SHA256(K_Set, BLAKE3(canonical(plaintext))) | HMAC-SHA256(K_Set, "topic-derivation-domain-tag" ‖ membership_set_id ‖ generation) |
| Cross-Atrium dedup | Lost (intended; isolation property) | Lost (intended; isolation property) |
| Key-rotation cadence | FORK-ONLY | FORK-ONLY (piggybacks D1's identical K_Set rotation) |

**The 1:1 mapping is the load-bearing insight that justifies the
"single elegant structural shape" claim of §1 — we are not building a
new defense, we are recognizing that Option D's defense IS this
defense, applied at a second surface.**

### §2.10 Recent literature on AS-level / global adversary against pubsub

Sources: arxiv 2308.02477 ("On the Inherent Anonymity of Gossiping"),
arxiv 1905.07598 ("Privacy Guarantees of Gossip Protocols in General
Networks"), arxiv 1902.07138 ("Who started this rumor? Differential
privacy guarantees of gossip protocols").

**Established result:** gossip protocols provide INHERENT source-
anonymity by structurally diffusing the originator across the swarm,
but this guarantee DEGRADES sharply under (a) AS-level passive observers,
(b) multi-node colluding observers, and (c) sufficient repetition for
statistical correlation. Differential-privacy frameworks quantify the
guarantee but do not strengthen it. **Honest disclosure of these
limits is the literature-standard posture**; our §6 residual-disclosure
section adopts this framing.

---

## §3 Task 2 — Assessment of the 6 initial defenses + 4 additional

For each defense: cryptographic soundness; composability with iroh-gossip;
wave-day cost; scalability impact; residual disclosure after applying.

### §3.1 Defense 1 — HMAC-blinded topic-id (the orchestrator's first idea)

**Construction.** `topic_id = HMAC-SHA256(K_Set, "benten/gossip/topic/v1" ‖
membership_set_id_bytes ‖ generation_le_bytes)` truncated to 32 bytes
(no truncation — HMAC-SHA256 output is already 32 bytes).

**Cryptographic soundness: STRONG.** Same primitive as Q3 Option D
(`23f76e24` §2.1). HMAC-SHA256 is a PRF under the NMAC reduction
(Bellare 2006). Pre-image / 2nd-pre-image / equality-oracle / key-recovery
resistance all STRONG. Domain-separation prefix
`"benten/gossip/topic/v1"` defeats cross-context PRF correlation per
NIST SP 800-108 framing. The inclusion of `generation_le_bytes`
(little-endian per Q2 BE-codepoint-endianness… correction: per
M-CONS-v2 the codepoints are BE but content-encoding ordering is LE for
field-internal counters — must match F1 codepoint-discipline; FLAG for
implementation review) binds the topic-id to a specific membership-set
generation so a generation-bump rotates the topic-id.

**Composability with iroh-gossip: PERFECT.** iroh-gossip consumes a 32-byte
`TopicId` and is agnostic to its derivation; n0's recommendation
("cryptographic hash of meaningful string") is already a strict subset
of our derivation. No protocol change required upstream. Per `n0/iroh`
discussion #3168 framing, application-layer derivation discipline is
the IDIOMATIC integration pattern; we are not violating any iroh
contract.

**Wave-day cost:** ~0.3 wave-day (derivation function ~30 LOC + tests
~80 LOC + cite-drift mirror in TS ~10 LOC + V1-FROZEN row + cross-language
rule-mirror per §3.5g extended).

**Scalability impact:** ZERO. Topic-id computation is a single HMAC-SHA256
evaluation per MembershipSet construction or generation-bump — sub-
microsecond cost; not on any hot path.

**Residual disclosure after applying ALONE:** Subscriber-set intersection
(WHICH peers subscribe to WHICH blinded-topic) remains visible to the
relay; topic-activity timing remains visible to the relay; the
**topic-id-history-via-K_Set-leak** residual remains (if K_Set leaks, all
past blinded topic-ids can be recomputed). **The defense is necessary
but not sufficient — needs D3 for rotation cadence guarantee and D5 for
out-of-band rendezvous to avoid topic-id leakage at admission time.**

**Verdict:** ADOPT as D1. Core of the mitigation.

### §3.2 Defense 2 — Per-epoch topic-id rotation

**Construction.** Topic-id rotates on every FORK-ONLY K_Set rotation
event per Inv-20 clause-b. Because D1's HMAC input includes
`generation_le_bytes` and K_Set rotates per-fork, this is structurally
automatic — **D2 is not a separate mechanism; it is a property of D1
combined with the existing F19 generation-CRDT-vector + Inv-20 clause-b
FORK-ONLY rotation cadence.**

**Cryptographic soundness: STRONG (free).** Each rotation produces an
HMAC output that is indistinguishable from random (and independent of
the prior epoch's HMAC output) to anyone lacking the post-rotation
K_Set. No predecessor-chain reconstruction from topic-id history alone
is possible.

**Composability:** PERFECT. Re-uses the existing FORK-ONLY rotation
cadence already required by M-CONS-v2 §1 + N3 RotationTrigger doc-enum
+ Compromise #54 (continuous-rotation deferral). No new rotation
machinery needed.

**Wave-day cost:** $0 (literally; the cost is already booked in
F19 + Inv-20 clause-b + N3 implementation).

**Scalability impact:** ZERO incremental beyond the existing
fork-cadence cost.

**Residual disclosure after applying ALONE:** Without D1, this is a no-op
(rotating a public membership_set_id pattern doesn't blind anything).
WITH D1, this fully defeats the **fork-tree-reconstruction-from-topic-id-
history** attack the M-C2 critique §3.3 specifically named.

**Verdict:** ADOPT as D2. Folds into D1 mechanically; surfaces as a
SEPARATE doc-row only to make the predecessor-chain unrecoverability
property auditor-clear.

### §3.3 Defense 3 — Multiple topics per MembershipSet (sharding)

**Construction.** Split a single MembershipSet's gossip traffic across
M topic-ids (e.g., 2–8 shards keyed by message-type or hash-prefix of
content-CID). Each shard has its own blinded topic-id.

**Cryptographic soundness:** Same as D1 per-shard; no additional
strength.

**Composability with iroh-gossip:** ACCEPTABLE but COSTLY. Each shard is
a separate HyParView swarm + separate PlumTree mesh → M× mesh-
maintenance overhead, M× NeighborDown detection latency divergence
(some shards may be partition-isolated while others remain healthy),
M× bootstrap cost per join.

**Wave-day cost:** ~1.5–2.5 wave-days (shard-selection logic + per-shard
peer-discovery + per-shard fault-handling + significantly more test
surface).

**Scalability impact:** **NEGATIVE.** Per M6 §3 Spike C empirical:
iroh-gossip's PlumTree amplification is 5-15× of raw fanout per topic;
running M topics multiplies the maintenance bandwidth M-fold without
proportional benefit. For a 200-member Atrium at M=4 shards, that's
20-60× raw fanout vs the single-topic 5-15× — net-negative bandwidth.

**Scalability rationale AGAINST sharding:** The PRIVACY benefit Waku
derives from sharding-with-coordination requires that **multiple
DIFFERENT MembershipSets** share each shard's topic-id (the k-anonymity
of "this shard has traffic from k different sets"). In Benten that
sharing is structurally impossible — each MembershipSet has its OWN
K_Set, hence its OWN blinded-topic-id; cross-set sharing would require
inter-MembershipSet K_Set agreement which violates Inv-20 clause-a +
the confidentiality-isolation discipline per CLAUDE.md baked-in #18.
**Sharding within ONE MembershipSet is just bandwidth-amplification
without k-anonymity gain.**

**Residual disclosure after applying:** Same as D1 alone, MINUS the
inter-shard correlation (relay sees activity on 4 unlinked topic-ids
instead of 1); but each shard's subscriber set is the same MembershipSet
membership, so the correlation can be recovered from
co-subscription patterns over time — only a TIME-LIMITED obfuscation.

**Verdict:** REJECT for v1-beta. **NAMED-DEFER** to post-v1-beta /
Phase-5+ contingent on (a) iroh-gossip per-topic membership ACL
landing upstream OR (b) Waku-style cross-set k-anonymity becoming
feasible under a future privacy-mixnet substrate. **NAMED-deferred
destination: `docs/future/phase-4-backlog.md` §4.XX-NEW
"Sharded multi-topic-per-MembershipSet privacy" with revisit-threshold
= "v1.x iroh-gossip ships per-topic ACL OR Phase-5+ adoption of
mixnet relay layer."**

### §3.4 Defense 4 — Onion-style relay mixing

**Construction.** Layer Tor-style onion routing or Nym-style mixnet
on top of iroh-gossip relay traffic. Each gossip message wrapped in an
onion of layered encryption; intermediate relays cannot determine
final destination topic.

**Cryptographic soundness:** STRONG per the Tor / Nym literature.

**Composability with iroh-gossip: POOR at v1-beta.** Requires
substantial protocol-stack extension (mixnet integration layer below
gossip; relay-coordination subprotocol; latency budget allocation).
Conflicts with the gossip latency profile (Spike C 1-2 ms per-message
becomes ~hundreds of ms through 3-hop mixnet) — defeats the Class B
"live feel" target that justified the gossip SCOPE-IN in the first
place.

**Wave-day cost:** ~15–30 wave-days minimum + new external dep family
(Tor or Nym SDK or hand-rolled onion).

**Scalability impact:** HEAVY. Mixnet adds 3-5× bandwidth per message
(layered encryption headers) + latency multiplier.

**Residual disclosure after applying:** Strongest defense among the
6 — closes both topic-fingerprint AND subscriber-correlation
residuals. The bound is the mixnet's anonymity-set size at peak
traffic.

**Verdict:** REJECT for v1-beta. **NAMED-DEFER** to Phase-5+ Kith
watch-list, paired with the Willow-Confidential-Sync future research
item. **NAMED-deferred destination: `docs/future/phase-5-kith-watchlist.md`
NEW row "Mixnet relay-layer for MembershipSet gossip" with revisit-
threshold = "(a) a production-grade Rust mixnet substrate matures
[Nym, Katzenpost, or successor], AND (b) Benten ships a use-case
class that justifies the latency cost [e.g., journalism source
protection per Compromise #54]."** Note: this is the proper
permanent-shape destination for **Compromise #59 long-term recovery
path beyond HMAC-blinding** — honest about the next-level threshold
mitigation.

### §3.5 Defense 5 — Out-of-band rendezvous via UCAN-signed Drop

**Construction.** Peers learn the (membership_set_id, K_Set,
generation, bootstrap-peer-list) tuple ONLY through the existing
DropBundle admission flow (per F4 + Inv-20 clause-a multi-stanza
HPKE-Encap). The blinded topic-id is computed locally from K_Set;
the bootstrap peers are listed in the DropBundle's `transport_config`
field (M6 §5.2) which is delivered alongside K_Set in the same
HPKE-sealed envelope.

**Cryptographic soundness:** Inherits from F4 + Inv-20 clause-a; no new
crypto.

**Composability with iroh-gossip: PERFECT.** This is the OPPOSITE of the
n0-discouraged "DHT-publish bootstrap-tickets-with-topics" pattern. The
UCAN-signed DropBundle is the canonical Benten admission path; we
extend it to carry the (one extra field) bootstrap-peer-list. No
iroh-gossip protocol change needed; the application provides peer-list
to `subscribe()` from local-store rather than from DHT lookup.

**Wave-day cost:** ~0.2 wave-days (extending the DropBundle wire shape
+ MS-PRIMITIVE / TRANSPORT integration handover).

**Scalability impact:** ZERO. The DropBundle is already on the
admission path; adding 8-32 NodeID bytes for bootstrap-peer-list is
trivial.

**Residual disclosure after applying:** Closes the **discovery-time
topic-id leak** (no DHT publish; no public rendezvous; topic-id
becomes K_Set-derivable and known only to admitted members). Does NOT
close the **runtime subscriber-set leak** at the relay.

**Verdict:** ADOPT as D3. Mandatory complement to D1 — without D3,
even a blinded topic-id leaks via DHT publish.

### §3.6 Defense 6 — Hybrid (1+2+3+5 per orchestrator's suggestion)

**Verdict:** Adopt **modified hybrid = D1 + D2 + D3** (NOT including
D3-sharding). Sharding (orchestrator's "3") is rejected per §3.3
above. The modified hybrid is the elegant winner per §4.

### §3.7 Additional defense 7 — Per-DropBundle stable rendezvous cookie

**Construction.** Analog to Tor v3's 20-byte rendezvous cookie: at
DropBundle creation, the issuer mints a fresh random 32-byte
`rendezvous_cookie` and includes it in the sealed envelope. The
gossip topic-id derivation HMACs the cookie INSTEAD of (or IN
ADDITION TO) the membership_set_id.

**Tradeoff vs D1 alone:** Allows the topic-id to be UNLINKED from any
durable identifier — rotating the cookie post-admission would mint a
fresh topic-id even WITHOUT a K_Set rotation. **The cost:** adds a
SECOND rotation cadence (per-cookie vs per-K_Set), violating the
"one-rotation-class-per-key" elegance noted in §2.5. Also doubles the
state-management surface (admitted members must agree which
rendezvous_cookie is current).

**Verdict:** REJECT for v1-beta as over-engineered. **NAMED-deferred**
to the same destination as Defense 3 sharding — coupled to the
Phase-5+ mixnet-layer discussion (rendezvous-cookies are the natural
companion primitive to onion routing). Re-evaluate alongside D4.

### §3.8 Additional defense 8 — Multi-topic decoy noise

**Construction.** Each member subscribes to N decoy topic-ids alongside
the real one, generating background traffic on the decoys to obscure
which topic is "real."

**Cryptographic soundness:** N/A (decoy traffic is a TRAFFIC-ANALYSIS
defense, not cryptographic).

**Composability with iroh-gossip:** Bandwidth-amplifying (each member
multiplies its uplink by N+1) + breaks the n0 "topic = swarm" model
(decoy swarms are functionally fake) + degrades Class B latency.

**Wave-day cost:** ~2 wave-days (decoy generator + traffic-shaping +
mesh-participation faking).

**Verdict:** REJECT. The cost is real (bandwidth + battery) while the
benefit is marginal — a sophisticated AS-level adversary can
distinguish decoy from real traffic by correlation patterns over
sufficient observation window. NAMED-deferred-OBJECT — not deferred at
all; explicitly rejected. The proper path to AS-level-adversary
resistance is D4 mixnet layering, not decoy noise.

### §3.9 Additional defense 9 — Topic-id-as-time-anchored-Tor-v3-style

**Construction.** Adopt Tor v3's per-time-period derivation directly:
`topic_id = HMAC-SHA256(K_Set, "topic-period" ‖ floor(now / period))`
with rotation every (e.g.) 24 hours.

**Tradeoff vs FORK-ONLY rotation:** Adds time-based rotation cadence
ON TOP OF the existing FORK-ONLY cadence. Splits the gossip mesh
every 24 hours even when no membership change occurred — every member
must re-bootstrap to the new topic-id, mesh re-converges, NeighborDown
latency adds churn cost. **Net: 24-hour mesh-thrash for marginal
privacy gain.**

**Verdict:** REJECT. The FORK-ONLY cadence (D2) is correct for our
threat model; time-based rotation is a Tor-specific pattern that
fits Tor's adversary model (global passive observer of LONG-LIVED
identifiers) but does NOT fit Benten's (collaboration substrate
where churn cost matters and FORK semantics are the natural mutation
boundary). Strictly worse than D2.

### §3.10 Additional defense 10 — Selective subscription via HTTP-style range request

**Construction.** Subscribe to a topic-id PREFIX (e.g., first 16 bytes
of HMAC output is shared; last 16 bytes private). Members subscribe
to all topics matching the prefix; relays see only prefix-level
membership.

**Composability with iroh-gossip:** INCOMPATIBLE. iroh-gossip's
TopicId is a full 32-byte exact-match identifier; there is no
prefix-subscription mechanism. Would require upstream protocol
extension to iroh-gossip (a HyParView-extension supporting topic-
prefix-match).

**Verdict:** REJECT. Requires upstream protocol change; outside our
scope; conflicts with iroh-gossip's design.

### §3.11 Summary defense-comparison matrix

| Defense | Crypto soundness | iroh-gossip compose | Wave-day cost | Scale impact | Residual disclosure |
|---|---|---|---|---|---|
| **D1 HMAC-blind** | STRONG | PERFECT | ~0.3 | ZERO | subscriber-set + K_Set-leak-recompute |
| **D2 FORK-ONLY rotation** | STRONG (folds into D1) | PERFECT | $0 | ZERO | (subsumed by D1's) |
| **D3 sharding** | (n/a) | ACCEPTABLE-but-COSTLY | ~1.5–2.5 | NEGATIVE | same as D1 + time-limited correlation |
| **D4 onion/mixnet** | STRONG | POOR at v1-beta | ~15–30 + new dep | HEAVY | strongest; closes subscriber-set |
| **D5 OOB rendezvous via UCAN-Drop** | STRONG | PERFECT | ~0.2 | ZERO | discovery-time leak closed; runtime open |
| **D6 hybrid 1+2+5 (modified)** | STRONG | PERFECT | ~0.4–0.6 | ZERO | subscriber-set + K_Set-leak-recompute |
| **D7 per-Drop cookie** | STRONG (+) | PERFECT | ~0.5 | ZERO | + cookie state mgmt cost |
| **D8 decoy noise** | (n/a) | bandwidth-amplifying | ~2 | NEGATIVE | marginal; AS-level breaks anyway |
| **D9 Tor-v3 time-rotation** | STRONG | PERFECT | ~0.4 | mesh-thrash | over-rotates |
| **D10 prefix-subscribe** | STRONG | INCOMPATIBLE | n/a | n/a | n/a |

**Adoption: D6 (= D1 + D2 + D5).** D3, D4, D7 NAMED-deferred. D8, D9,
D10 rejected.

---

## §4 Task 3 — Composition concerns

### §4.1 Compose with F19 fork-on-event (predecessor-chain observability)

**Concern restated.** F19 fork-on-event mints a fresh `membership_set_id`
with parent_membership_set_id chain pointing back to predecessor. M-C2
§3.3 critique: "two forks of the same parent MembershipSet share a
deterministic-derivable predecessor; gossip-topic-name predecessor-
chain leaks the entire MembershipSet history."

**Mitigation under D6.** The `parent_membership_set_id` chain is
**ENVELOPE-LAYER metadata** — visible to members who decrypt the
envelope (per Inv-20 clause-c AAD-binding); INVISIBLE to gossip
relays (who see only the 32-byte HMAC-output topic-id). The HMAC-PRF
property guarantees that
`HMAC(K_Set_parent, parent_id) -- HMAC(K_Set_post-fork, post-fork-id)`
have NO observable correlation — they are independent PRF outputs.

**Verification.** For Atrium A→A1 fork: pre-fork K_Set is rotated
post-fork (Inv-20 clause-b FORK-ONLY); post-fork members compute
topic_id_A1 from the NEW K_Set; the relay sees topic_id_A1 as a
fresh blinded value with no traceable link to topic_id_A. **CLOSED
under D6.**

### §4.2 Compose with Compromise #48 (MembershipSet-shape-leak at K_Set compromise)

**Concern restated.** #48 already discloses: if K_Set leaks,
MembershipSet shape (member-DID-list, generation, etc.) leaks. M-C3
§3.1 #59 candidate notes #59 is a "WEAKER threshold" than #48 —
passive-network-observation vs shared-key-compromise.

**Mitigation under D6 — composition with #48 disclosure.** We
EXTEND #48 to explicitly enumerate topic-id-history-recomputability
under K_Set leak as a CLAUSE of #48 rather than minting a separate
disclosure. The recovery path (FORK-ONLY rotation to revoke past K_Set
adversary access) applies symmetrically. Without K_Set leak, the
adversary cannot brute-force past topic-ids (2^256 work-factor per
HMAC-PRF security); with K_Set leak, the existing #48 recovery path is
the correct path. **No new threshold introduced; #48 doc-amended
under D6.**

### §4.3 Compose with Inv-20 clause-d per-recipient unlinkability

**Concern restated.** Inv-20 clause-d (M-CONS-v2 §-around-line-496):
"per-recipient unlinkability." MCV2-C-3 contradiction names that
naïve topic-per-MembershipSet defeats this at the transport layer
even though cryptographic layer preserves it.

**Mitigation under D6.** Under blinded topic-id, the
**topic-id-itself** is no longer a per-recipient-linkable identifier —
all members of MembershipSet M share the SAME blinded topic-id
(it's the broadcast scope), but that topic-id is uncorrelated to
ANY external identity. The per-recipient unlinkability that Inv-20
clause-d guards (the property that the SAME content addressed to
multiple recipients is encrypted-stanza-distinctly per-recipient) is
ORTHOGONAL to the topic-id privacy — the per-stanza unlinkability
operates ABOVE the gossip layer. **What we are mitigating is a
DIFFERENT axis** — the membership-set-level fingerprint via the
topic-id — and the mitigation does not interfere with clause-d's
per-recipient axis.

**Clause-d framing under D6.** Recommend a doc-amendment to Inv-20
clause-d (or an Inv-20 clause-d-prime) that names: "per-recipient
unlinkability at the cryptographic layer; per-MembershipSet
unlinkability at the transport layer via HMAC-blinded topic-id (under
GossipPlusBlobs / GossipRequiredBlobsFallback transport configs)."
This makes the two-layer guarantee explicit + auditor-clear. **CLOSED
under D6 + recommended doc-amendment.**

### §4.4 Compose with N4 R-N4-3a fork-tie-break

**Concern restated.** N4 R-N4-3a: lexicographically-smaller HLC wins
in concurrent-fork tie-break; losing fork archived for audit.
Inv-21 candidate (M-C3 v2 F-FE-3): fork-tie-break HARD partition
boundary.

**Composition with D6.** Concurrent forks A→A1 and A→A2 each derive
their own blinded topic-ids from their respective post-fork K_Set
values. **Both topic-ids are live for the brief window between
fork-emission and tie-break-decision** — this is the same window where
the F19 generation-CRDT-vector dedup operates at envelope layer. After
tie-break: the winning fork's topic stays active; the losing fork's
topic is **frozen for audit-archive** but receives no new traffic.

**Subtle interaction with iroh-gossip 30-35 s NeighborDown lag** (M6
§3.1 footgun). The fork-tie-break decision propagates via existing
F19 + Inv-20 clause-e mechanisms (HPKE-Encap envelope at admission
layer); the gossip mesh for the LOSING fork remains structurally
alive for ~30-35 s after the last writer stops. This is **survivable**
per pim-2 substantive-arm: an e2e pin requirement covers
"after-tie-break gossip mesh quiescence within 60s." **Recommended
RED-PHASE staged-pin per pim-12:
`concurrent_fork_tiebreak_gossip_quiesces_within_60s.rs` `#[ignore]`-named,
un-ignore at Wave-MS-TRANSPORT impl.**

**CLOSED under D6 + recommended e2e pin.**

### §4.5 Compose with M-CONS-v2 §7.4 TransportConfig per MembershipSet

**Concern restated.** M6 §5.2 defined `TransportConfig` with a
`gossip_topic_id: Option<[u8; 32]>` field with semantics "None =
compute deterministically from MembershipSet root + K_Atrium-blind."
M-CONS-v2 §7.4 ships TransportConfig in `benten-engine` with Wave-MS-
TRANSPORT impl.

**Composition under D6.** The `gossip_topic_id` field semantics
become:
- **`None` (DEFAULT for Atrium / DeviceMesh under GossipPlusBlobs):**
  compute as D1 — `HMAC-SHA256(K_Set, "benten/gossip/topic/v1" ‖
  membership_set_id ‖ generation)`.
- **`Some(bytes)` (DEPLOYMENT-OVERRIDE):** explicit topic-id (e.g.
  for testing or for legacy interop). **STRICTLY DISCOURAGED in
  production** — flagged via `MembershipSetPolicy` audit field;
  `cargo clippy` lint TBD; runtime warning on subscribe.
- **`SingleDevice` Kind:** field is always `None` and unused
  (TransportKind::None).

**Wire-shape impact.** ZERO change to the M6 §5.2 type definition;
only the DEFAULT-FOR-NONE semantic shifts from "root+K_Atrium-blind"
to "K_Set-blind with epoch binding." The change is a DOC-clarification
+ implementation-detail; no V1-FROZEN row updates beyond a clarifying
sentence.

**Wire-shape impact on SubInheritPolicy.** When sub-MembershipSets
inherit transport config (`SubInheritPolicy::Inherit` /
`InheritWithOverride`), each sub-MembershipSet computes its OWN
blinded topic-id from its OWN K_Set (which differs from parent's per
N1 RestrictedScopeSet grant-time composition + per Garden recursive
shape). The inherited field is the TransportKind / fallback /
extension shape, not the topic-id itself. **DOC-clarification needed:
the `SubInheritPolicy::Inherit` semantic for the `gossip_topic_id`
field means "inherit the None-default-derivation rule, NOT inherit
the parent's literal topic-id bytes."** Recommended sentence-add to
M6 §5.2.

**CLOSED under D6 + M6 §5.2 doc-clarifications.**

### §4.6 Compose with Q3 DUAL-CID + `K_DedupScope` general framing

**Composition with K_DedupScope.** Per Q3 specialist `23f76e24` §1
third conclusion: "Treat K_Atrium as one instance of a general
`K_DedupScope: HMAC-PRF-key` parameter in the EncryptedEnvelope
codepoint family." Under M-CONS-v2 N2 absorption, K_Atrium = K_Set.
D6's HMAC-blinded topic-id is **a second instance of the SAME
parameter applied at a DIFFERENT surface (transport topic-id rather
than envelope plaintext_cid)**. This is the structural elegance:
ONE key (K_Set), TWO surfaces (DUAL-CID + topic-id), both blinded by
the same HMAC-PRF regime, both rotating FORK-ONLY.

**Recommendation:** the M-CONS-v2 doc grows a §-add (Q3-Option-D-
Companion §) generalizing "K_Set serves as the K_DedupScope parameter
for {plaintext_cid_atrium, gossip_topic_id, ...} surfaces, with each
surface keyed by a domain-separated HMAC-SHA256 derivation." This
codifies the elegance for auditor + future-extension clarity.

### §4.7 Compose with sealed-sender DEFAULT (Am4) + M-C3 #58 multi-stanza key-confirmation

**Sealed-sender composition.** Am4 sealed-sender DEFAULT hides
sender_did from intermediaries. D6 hides MembershipSet-id from
relays. The two compose orthogonally — sealed-sender operates at the
envelope-stanza layer; D6 operates at the topic-id layer. A relay
under D6 + Am4 sees: (blinded topic-id, opaque sealed-sender
envelope, timing). Strongest composition.

### §4.8 Net composition verdict

**0 of 7 composition axes block D6.** All 7 close with EITHER zero
intervention (axes 1, 6, 7) OR with the recommended doc-amendments /
pins enumerated above (axes 2, 3, 4, 5). Composition risk: LOW.

---

## §5 Task 4 — Final elegant shape + alternative-deferrals

### §5.1 Per the extra-reflection-pass-for-elegant-permanent-shape discipline

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape.md`,
after the per-defense triage table (§3.11), I take ONE EXTRA PASS
holistically: is there a SINGLE elegant structural shape closing
multiple findings at once?

**Yes.** The single elegant shape is **K_Set as universal
`K_DedupScope` PRF-key, applied at TWO surfaces under
domain-separated HMAC-SHA256 derivation, with rotation cadence
unified at FORK-ONLY per Inv-20 clause-b.**

This single shape closes:
- **Compromise #59 (topic-fingerprint at transport layer):** mitigated
  by surface-2 application of K_DedupScope.
- **MCV2-C-3 contradiction (this doc's origin):** resolved by surface-2
  application + the predecessor-chain unrecoverability follow-on.
- **Implicit composition gap between Q3 Option D and gossip layer:**
  surfaced explicitly under the unified framing.
- **Forward extensibility:** future surfaces (e.g., a per-MembershipSet
  beacon for liveness in a future iroh-roq scope) inherit the same
  K_DedupScope discipline by default.

### §5.2 Winner: D6 = D1 + D2 + D5 (modified hybrid)

**The chosen shape:**

1. **D1 — HMAC-blinded topic-id:**
   ```
   topic_id = HMAC-SHA256(K_Set,
                          "benten/gossip/topic/v1" ‖
                          membership_set_id_bytes ‖
                          generation_le_bytes_8)
   ```
   Output is exactly 32 bytes (iroh-gossip TopicId wire format).
2. **D2 — FORK-ONLY epoch rotation:** automatic via D1's input dependency
   on `K_Set` (which rotates per Inv-20 clause-b) and `generation`
   (which advances per F19 generation-CRDT-vector). No separate rotation
   machinery.
3. **D5 — Out-of-band rendezvous via UCAN-signed DropBundle:**
   bootstrap-peer-list is delivered alongside K_Set in the same HPKE-
   sealed admission envelope; NEVER published to DHT / public
   rendezvous / DropBundle public-side.

### §5.3 Justification (why D6 wins over D3, D4, D7)

- **vs D3 sharding:** D3 is bandwidth-amplifying without k-anonymity
  gain in Benten's single-set-per-shard structural constraint.
- **vs D4 onion/mixnet:** D4 is the right LONG-TERM answer for AS-level
  adversary resistance but ~15-30 wave-days + new dep family makes it
  v1-beta-incompatible. D6 closes the v1-beta-relevant residual; D4
  closes the post-v1-beta adversary class.
- **vs D7 per-Drop cookie:** D7 adds a second rotation cadence
  (per-cookie vs per-K_Set) violating the single-rotation-elegance.

### §5.4 NAMED-deferred alternatives (per HARD-RULE clause-b)

| Alternative | Destination | Revisit trigger |
|---|---|---|
| **D3 sharded multi-topic-per-set** | `docs/future/phase-4-backlog.md` §4.XX-NEW "Sharded multi-topic-per-MembershipSet privacy" | v1.x iroh-gossip ships per-topic ACL OR Phase-5+ adoption of mixnet substrate enables cross-set k-anonymity |
| **D4 onion/mixnet relay layer** | `docs/future/phase-5-kith-watchlist.md` NEW row "Mixnet relay-layer for MembershipSet gossip" | (a) production-grade Rust mixnet substrate matures [Nym / Katzenpost / successor]; (b) Benten ships a use-case justifying the latency cost [e.g., journalism source protection per #54] |
| **D7 per-Drop rendezvous cookie** | Coupled to D4 destination (same row) | Re-evaluate when D4 is on the table |

**Per HARD-RULE clause-b:** each destination is SPECIFIC (named file +
named row); each revisit-trigger is CONCRETE (named external condition).
NO "phase-N follow-up" / "defer to later" violations.

### §5.5 Explicit rejections (NOT deferred; rejected outright)

- **D8 decoy noise:** AS-level adversary breaks under sufficient
  observation; bandwidth cost is real; no path to meaningful
  improvement under future research either. REJECT.
- **D9 Tor-v3 time-period rotation:** strictly worse than D2's
  FORK-ONLY cadence for Benten's threat model (collaboration
  substrate, not high-churn anonymous-publishing). REJECT.
- **D10 prefix-subscribe:** requires iroh-gossip upstream protocol
  change; out of scope. REJECT.

---

## §6 Task 4 continued — Residual disclosure honest enumeration

Under the adopted D6 shape, the following residuals remain. Each is
DISCLOSED in the final Compromise #59 text (§8).

### §6.1 Residual R1 — Subscriber-set intersection at the relay

**What:** A passive relay operator (or any peer participating in the
gossip mesh of MembershipSet M) sees WHICH peers subscribe to WHICH
blinded topic-id. Over time, a single relay can build a
peer↔topic-id mapping table.

**Severity:** k-anonymous against the population of all Benten
MembershipSets sharing the relay. For a popular relay serving N
deployments, k ≈ N (the relay cannot distinguish "MembershipSet M is
group X" from "MembershipSet M is group Y" — only that some set
exists).

**Mitigation NOT available under D6:** This is structurally inherent
to non-mixnet pub/sub overlay (Waku 2024 analysis confirms; Signal
sealed-sender literature confirms via delivery-record correlation).
The path to mitigating this residual is D4 mixnet layering (NAMED-
deferred per §5.4).

**Honest-disclosure framing:** Compromise #59 names this residual
explicitly per Substrate-Guarantee-Disclosure class (per P14
discipline).

### §6.2 Residual R2 — Topic-id activity timing

**What:** The relay sees WHEN traffic occurs on which blinded topic-id.
Patterns may reveal MembershipSet activity rhythms (e.g., a corporate
Atrium active during business hours; a journalism collaboration active
during news cycles).

**Severity:** Pattern-statistical; bounded by D2 FORK-ONLY rotation
(rotation event partitions the timing window per-fork). Low for
forkful MembershipSets, higher for rarely-forked ones.

**Mitigation NOT available under D6:** Same class as R1 — requires
mixnet padding or constant-cover-traffic, both Phase-5+.

### §6.3 Residual R3 — Topic-id-history-disclosure under K_Set leak

**What:** If K_Set ever leaks to an adversary, that adversary can
recompute ALL past blinded topic-ids for the MembershipSet (because
HMAC is deterministic given the key + input). This unlocks the
adversary's stored relay-observation logs to retroactively label
topic-ids with MembershipSet identifiers.

**Severity:** EQUIVALENT THRESHOLD to Compromise #48 (which already
discloses MembershipSet-shape-leak under K_Set compromise). Recovery
path is FORK-ONLY rotation per Inv-20 clause-b: post-rotation
topic-ids are uncorrelated to pre-rotation ones, so the adversary's
forward-visibility ends at the K_Set-leak moment.

**Mitigation under D6:** None NEW (this is the same equivalence-class
as #48, not a new threshold). Compromise #59 text references the
#48 recovery path; #48 doc-amended to enumerate the
topic-id-history-recomputability clause.

### §6.4 Residual R4 — Bootstrap-peer-list endpoint exposure

**What:** D5's out-of-band rendezvous distributes bootstrap-peer
NodeIDs via the UCAN-signed DropBundle. The receiver, once a member,
holds the bootstrap-peer-list AND can correlate (member-DID, NodeID
they were given) — a compromised member can leak the bootstrap-peer
list externally, allowing a non-member to attempt joining the gossip
mesh (which is then refused at envelope-layer because the non-member
lacks K_Set to decrypt anything).

**Severity:** LOW. Joining the mesh without K_Set gains the
adversary only mesh-membership visibility (which is structurally
visible anyway per R1 once K_Set is known to one corrupted member).
The HMAC-blinded topic-id without K_Set is just a 32-byte string
the adversary cannot derive themselves.

**Mitigation under D6:** N/A — this is the SAME class as "compromised
member leaks K_Set" (= Compromise #48); the bootstrap-peer-list leak
is just a sub-property. Recovery: same FORK-ONLY rotation path.

### §6.5 Residual R5 — IP-address-level identity leak at relay connect

**What:** When a peer connects to the relay (per any iroh-gossip-using
config), the relay sees the peer's IP address. This is INHERENT to the
iroh-relay design (per iroh's own `docs.iroh.computer/deployment/
security-privacy` disclosure). Independent of topic-id-blinding.

**Severity:** Identity-correlation hazard; mitigatable only via
non-direct (e.g., Tor / VPN / mixnet) routing.

**Mitigation under D6:** Not in scope (this is an iroh-relay-layer
property, not a Benten-protocol property). Honest disclosure under
the existing iroh-deployment-security framing.

### §6.6 Residual disclosure summary

| Residual | What leaks | Severity | Mitigation path | v1-beta posture |
|---|---|---|---|---|
| R1 | Subscriber-set intersection at relay | k-anonymous; Waku-class | D4 mixnet (Phase-5+) | DISCLOSE in #59 |
| R2 | Topic-id activity timing | pattern-statistical; bounded by D2 rotation | D4 mixnet (Phase-5+) | DISCLOSE in #59 |
| R3 | Past topic-id-history under K_Set leak | equivalent threshold to #48 | FORK-ONLY rotation per Inv-20 b | DISCLOSE as #48 clause |
| R4 | Bootstrap-peer-list under member compromise | LOW; sub-property of #48 | FORK-ONLY rotation | DISCLOSE as #48 clause |
| R5 | IP-address at relay connect | iroh-layer; out-of-scope | non-direct routing | DEFER to iroh-deployment-doc |

**Net residual posture: HONEST + AUDIT-CLEAR + each residual has a
NAMED recovery path or NAMED-deferred mitigation destination.**

---

## §7 Task 5 — Implementation specification

### §7.1 Concrete Rust code sketch — topic-id derivation

New module: `crates/benten-membership-set/src/gossip_topic.rs` (per
M6 §10 deliverable; this doc REFINES M6's stub-shape for that file).

```rust
//! Gossip topic-id derivation for `MembershipSet` under
//! `GossipPlusBlobs` / `GossipRequiredBlobsFallback` `TransportKind`.
//!
//! Defends against MCV2-C-3 topic-fingerprint-leak at the iroh-gossip
//! transport layer (see `Compromise #59`). The topic-id is an
//! HMAC-SHA256 output keyed by `K_Set`, with a domain-separation
//! prefix and a generation binding; rotation cadence is FORK-ONLY
//! per Inv-20 clause-b.
//!
//! Cross-references:
//! - `crates/benten-membership-set/specs/MembershipSet-Spec.md`
//!   §-on-K_DedupScope (D6 framing).
//! - Q3 Option D K_Atrium-blinding precedent (`23f76e24`).
//! - `docs/foundation/COMPROMISES.md` #59 (residual disclosure).

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Domain-separation tag for gossip topic-id derivation.
///
/// MUST be string-stable + cross-language-mirrored to TS bindings
/// per §3.5g cross-language rule-mirror discipline.
pub const GOSSIP_TOPIC_DOMAIN_TAG: &[u8] = b"benten/gossip/topic/v1";

/// Compute the blinded gossip topic-id for a `MembershipSet`.
///
/// # Inputs
/// - `k_set`: the per-MembershipSet shared symmetric key (Inv-20
///   clause-a). MUST be 32 bytes (HKDF-Extract output, per Layer-B
///   HKDF chain).
/// - `membership_set_id`: the (current-generation) MembershipSet
///   identifier, canonical-bytes-encoded per F1 codepoint-discipline
///   (CID v1 + dag-cbor; BLAKE3 hash; ~38-byte typical).
/// - `generation`: the F19 generation-CRDT-vector scalar for the
///   current generation, encoded little-endian to 8 bytes (matches
///   the existing F1 field-internal LE convention).
///
/// # Output
/// The 32-byte topic-id consumable by `iroh_gossip::proto::TopicId`.
///
/// # Cryptographic properties
/// - `topic_id` is computationally indistinguishable from random to
///   any party lacking `k_set` (HMAC-SHA256 is a PRF under the
///   NMAC reduction per Bellare 2006).
/// - Rotating `k_set` (FORK-ONLY per Inv-20 clause-b) produces an
///   UNCORRELATED topic-id; the parent-fork topic-id and the
///   post-fork topic-id have NO observable cryptographic link to
///   anyone lacking BOTH the pre-rotation and post-rotation k_set.
/// - The `GOSSIP_TOPIC_DOMAIN_TAG` defeats cross-context PRF
///   correlation (NIST SP 800-108 framing); `k_set` is also used by
///   `plaintext_cid_atrium` derivation (Q3 Option D) under a DIFFERENT
///   domain tag, ensuring the two outputs are uncorrelated.
pub fn derive_gossip_topic_id(
    k_set: &[u8; 32],
    membership_set_id: &[u8],
    generation: u64,
) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(k_set)
        .expect("HMAC-SHA256 accepts any 32-byte key");
    mac.update(GOSSIP_TOPIC_DOMAIN_TAG);
    // Domain-separator/length-prefix discipline: encode |id| as u32-LE
    // before id-bytes so that distinct (id, generation) tuples never
    // produce ambiguous inputs (per RFC 9180 §4 framing).
    mac.update(&(membership_set_id.len() as u32).to_le_bytes());
    mac.update(membership_set_id);
    mac.update(&generation.to_le_bytes());
    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-key non-correlation property: rotating `k_set` produces
    /// uncorrelated topic-ids.
    #[test]
    fn fork_only_rotation_uncorrelates_topic_id() {
        let k_set_pre = [0x11; 32];
        let k_set_post = [0x22; 32];
        let ms_id_pre = b"membership-set-pre-fork";
        let ms_id_post = b"membership-set-post-fork";
        let topic_pre = derive_gossip_topic_id(&k_set_pre, ms_id_pre, 1);
        let topic_post = derive_gossip_topic_id(&k_set_post, ms_id_post, 2);
        // Heuristic non-correlation: at least 12 of 32 bytes differ.
        let diff = topic_pre.iter().zip(topic_post.iter())
            .filter(|(a, b)| a != b).count();
        assert!(diff >= 12, "post-fork topic-id correlates with pre-fork");
    }

    /// Determinism: same inputs → same topic-id (idempotence under
    /// re-derivation; recipient + sender + relay-rejoin agree).
    #[test]
    fn derivation_is_deterministic() {
        let k = [0xAB; 32];
        let id = b"deterministic-test";
        let a = derive_gossip_topic_id(&k, id, 7);
        let b = derive_gossip_topic_id(&k, id, 7);
        assert_eq!(a, b);
    }

    /// Generation-bump rotates topic-id (D2 property).
    #[test]
    fn generation_bump_rotates_topic_id() {
        let k = [0xCD; 32];
        let id = b"generation-test";
        let g1 = derive_gossip_topic_id(&k, id, 1);
        let g2 = derive_gossip_topic_id(&k, id, 2);
        assert_ne!(g1, g2);
    }

    /// Domain-separation: same key + same id under DIFFERENT domain
    /// tag (e.g. plaintext_cid_atrium) produces uncorrelated output.
    /// This is enforced by the GOSSIP_TOPIC_DOMAIN_TAG prefix being
    /// distinct from the dedup-CID domain tag.
    #[test]
    fn cross_domain_uncorrelated_to_dedup_cid() {
        let k = [0xEF; 32];
        let id = b"cross-domain-test";
        let topic = derive_gossip_topic_id(&k, id, 1);

        // Simulate plaintext_cid_atrium derivation (Q3 Option D shape).
        let mut mac = Hmac::<Sha256>::new_from_slice(&k).unwrap();
        mac.update(b"benten/dedup/cid/v1"); // DIFFERENT domain tag
        mac.update(id);
        let dedup_cid = mac.finalize().into_bytes();

        let diff = topic.iter().zip(dedup_cid.iter())
            .filter(|(a, b)| a != b).count();
        assert!(diff >= 12, "cross-domain outputs correlate");
    }
}
```

### §7.2 Integration with M6 TransportConfig + Wave-MS-TRANSPORT

**Type-level change to `TransportConfig`:** ZERO. The `gossip_topic_id:
Option<[u8; 32]>` field shape is unchanged.

**Semantic clarification to `gossip_topic_id` default-None handling.**
Edit `crates/benten-engine/src/transport_config.rs` (per M6 §10.1
item 1 deliverable):

```rust
impl TransportConfig {
    /// Resolve the effective gossip topic-id for the given
    /// MembershipSet. For `Some(explicit)`, returns the explicit
    /// bytes (DEPLOYMENT-OVERRIDE; flagged via audit field).
    /// For `None` (DEFAULT), derives via
    /// [`gossip_topic::derive_gossip_topic_id`] per Compromise #59.
    pub fn effective_topic_id(
        &self,
        ms: &MembershipSet,
    ) -> Result<[u8; 32], TransportError> {
        match self.gossip_topic_id {
            Some(bytes) => {
                // Production-discouraged path; logged at WARN per
                // `MembershipSetPolicy` audit.
                Ok(bytes)
            }
            None => Ok(derive_gossip_topic_id(
                ms.k_set(),
                ms.canonical_id_bytes(),
                ms.generation(),
            )),
        }
    }
}
```

**Out-of-band rendezvous (D5) integration:** ADD a field
`bootstrap_peers: BTreeSet<NodeId>` to the `MembershipSet` admission
flow's DropBundle envelope. Receiver stores the list locally; passes
to `iroh_gossip::subscribe()` on join. NO public-side disclosure;
NO DHT publish; NO public rendezvous-server use. Enforced via
type-state builder pattern (compile-time impossibility of constructing
a `GossipTransportContext` without a bootstrap-peer-list of length ≥1
that arrived through the UCAN-admission code-path).

**ENGINE configuration check (RED-PHASE staged-pin per pim-12):**
`crates/benten-membership-set/tests/gossip_topic_derives_with_k_set.rs`
exercises `derive_gossip_topic_id` against fixtures. The
production e2e arm: `crates/benten-engine/tests/end_to_end/
membership_set_gossip_topic_blinded.rs` (substantive arm per
pim-2 + §3.6f extension) — verifies that:
1. Two-device flow over actual iroh-gossip subscribes to the blinded
   topic-id, not the raw membership_set_id;
2. Naïvely subscribing to `BLAKE3(membership_set_id)` MISSES the
   broadcast (negative-arm; demonstrates would-FAIL-if-blinding-not-
   applied);
3. Post-fork (K_Set rotation), both sender + receiver migrate to the
   new topic-id and traffic resumes; topic-id observable via
   `iroh_gossip` admin/test API is the HMAC-blinded value, not the
   raw membership_set_id.

### §7.3 Cite-drift cross-language rule-mirror

Per §3.5g cross-language rule-mirror discipline + §3.5g amendment
2026-05-13 (Phase-4-Foundation R6 R1): the
`GOSSIP_TOPIC_DOMAIN_TAG` constant + derivation function MUST be
mirrored in TypeScript bindings (`packages/engine/src/gossip_topic.ts`)
with a cite-drift scanner entry. Pin: `benten-membership-set::
derive_gossip_topic_id` mirrors to `packages/engine/src/gossip_topic.ts::
deriveGossipTopicId` 1:1; both consume the same `GOSSIP_TOPIC_DOMAIN_TAG`
string. Tested by parity vector (a fixed input → same 32-byte output
in both implementations).

### §7.4 V1-FROZEN-INTERFACE.md row

NEW row in V1-FROZEN-INTERFACE.md §15.6 (TransportConfig section):

> | `benten_membership_set::gossip_topic::GOSSIP_TOPIC_DOMAIN_TAG` |
> Domain-separation tag for HMAC-SHA256 gossip topic-id derivation.
> Wire-stable string `"benten/gossip/topic/v1"`. Cross-language
> mirrored. Rotation requires codepoint-discipline v2 mint
> (`"benten/gossip/topic/v2"`); v1 freezes at Phase-4-Meta-Core
> v1-beta tag. |

### §7.5 Wave-day cost breakdown

| Sub-deliverable | LOC | Wave-days |
|---|---|---|
| `gossip_topic.rs` module (Rust) | ~50 | ~0.15 |
| Tests (unit + property + cross-domain) | ~100 | ~0.10 |
| TS mirror `gossip_topic.ts` + parity vector | ~60 | ~0.05 |
| `TransportConfig::effective_topic_id` impl | ~30 | ~0.05 |
| `bootstrap_peers` field on DropBundle + admission integration | ~80 | ~0.10 |
| V1-FROZEN row + cite-drift scanner + cross-lang mirror | ~20 | ~0.05 |
| End-to-end pin per pim-2 substantive-arm | ~120 | ~0.20 |
| **Total Wave-MS-TRANSPORT delta vs M-CONS-v2 §7.4 baseline** | **~460** | **~0.70** |

**Bracket: 0.5–0.9 wave-days central; per M-CONS-v2 §7.4 baseline
5–8, the new central is 5.5–8.7. Within the existing 4.5–9 bracket;
NO bracket-shift.** This is the load-bearing "implementable AT
v1-beta within the existing wave-day envelope" claim of §1.

---

## §8 Task 6 — Final Compromise #59 mint text

### §8.1 Recommended #59 text (canonical form)

> **Compromise #59 (MITIGATED — gossip topic-id passive-network
> fingerprint via HMAC-blinded topic-id + FORK-ONLY rotation +
> out-of-band rendezvous; M-CONS-v2 Q1 SCOPE-IN companion).**
>
> **Threat model:** Under `TransportKind::GossipPlusBlobs` and
> `TransportKind::GossipRequiredBlobsFallback` (Wave-MS-TRANSPORT
> SCOPE-IN per Ben Q1 ratification 2026-05-27), each MembershipSet
> publishes via an iroh-gossip topic-id. iroh-gossip topic-ids are
> PUBLIC at the transport layer (n0-computer's design intent —
> applications are expected to layer authorization on top per
> n0/iroh discussion #3168). A passive network observer / iroh-relay
> operator who observes the gossip mesh can enumerate
> (topic-id, subscriber-set, publication-timing) tuples. A naïve
> `topic_id = BLAKE3(membership_set_id)` derivation would let the
> observer link each topic-id to a stable MembershipSet identifier,
> and the `parent_membership_set_id` chain (F19 fork-on-event) would
> let the observer reconstruct the full MembershipSet fork tree
> purely from observed topic-id rotation patterns — defeating
> Inv-20 clause-d per-recipient unlinkability at the TRANSPORT layer
> even though per-stanza HPKE preserves it at the CRYPTOGRAPHIC
> layer.
>
> **Mitigation (D6 hybrid; ADOPTED at v1-beta):**
> 1. **HMAC-blinded topic-id (D1):** `topic_id = HMAC-SHA256(K_Set,
>    "benten/gossip/topic/v1" ‖ membership_set_id ‖ generation)`
>    truncated to 32 bytes. Same HMAC-PRF regime as Q3 Option D
>    K_Atrium-blinded `plaintext_cid_atrium` (`23f76e24`); same
>    cryptographic soundness (Bellare-Keelveedhi-Ristenpart 2013
>    DupLESS literature + Bellare 2006 NMAC reduction; STRONG
>    pre-image / 2nd-pre-image / equality-oracle / key-recovery
>    resistance). The relay sees an indistinguishable-from-random
>    32-byte topic-id per MembershipSet; cannot enumerate
>    membership_set_id from topic-id without K_Set (~2^256 brute-
>    force margin).
> 2. **FORK-ONLY epoch rotation (D2):** the derivation depends on
>    K_Set + generation; both rotate per Inv-20 clause-b /
>    F19 generation-CRDT-vector / N3 RotationTrigger discipline.
>    Post-fork topic-ids are uncorrelated to pre-fork topic-ids
>    (HMAC-PRF property). **The fork-tree-reconstruction attack
>    named in MCV2-C-3 is CLOSED** — the predecessor-chain is
>    envelope-layer metadata (decryptable only by K_Set holders),
>    not visible to relays observing topic-id rotation.
> 3. **Out-of-band rendezvous (D5):** bootstrap-peer NodeIDs and
>    K_Set are distributed ONLY via the UCAN-signed DropBundle
>    admission flow (per F4 + Inv-20 clause-a multi-stanza
>    HPKE-Encap). NEVER published to DHT / public rendezvous
>    server / DropBundle-public-side. Closes the
>    discovery-time topic-id leak vector.
>
> **Residual disclosure (5 residuals):**
> - **R1 — Subscriber-set intersection at relay:** any passive
>   relay sees WHICH peers subscribe to WHICH blinded topic-id;
>   k-anonymous against the population of all MembershipSets
>   sharing the relay. Structurally inherent to non-mixnet
>   pub/sub (Waku 2024 analysis + Signal sealed-sender literature
>   confirm equivalence-class). **Mitigation path:** mixnet relay
>   layer (NAMED-deferred to `docs/future/phase-5-kith-watchlist.md`
>   "Mixnet relay-layer for MembershipSet gossip"; revisit-trigger:
>   production Rust mixnet substrate matures + Benten ships a
>   use-case justifying the latency cost [e.g., journalism source
>   protection per #54]).
> - **R2 — Topic-id activity timing:** relay sees timing patterns;
>   bounded by D2 rotation. Same mitigation path as R1.
> - **R3 — Topic-id-history-disclosure under K_Set leak:** if K_Set
>   leaks, all past topic-ids for the MembershipSet become
>   recomputable by the adversary. **EQUIVALENT THRESHOLD to
>   Compromise #48** (MembershipSet-shape-leak under K_Set
>   compromise); enumerated as an additional clause of #48.
>   Recovery path is FORK-ONLY rotation per Inv-20 clause-b
>   (forward-visibility ends at K_Set-leak moment).
> - **R4 — Bootstrap-peer-list under member compromise:** a
>   compromised member can leak bootstrap-peer NodeIDs; allowed
>   join attempts fail at envelope-decryption (lack K_Set). LOW
>   severity; sub-property of #48.
> - **R5 — IP-address at relay connect:** iroh-relay-layer
>   property (per n0-computer iroh deployment doc); out of
>   Benten-protocol scope; mitigatable via Tor / VPN / mixnet
>   external routing.
>
> **Disclosure class:** Substrate-Guarantee-Disclosure per P14 +
> MITIGATED-not-merely-DISCLOSED (the D1+D2+D5 composition closes
> the v1-beta-actionable surface; only structurally-inherent
> residuals R1+R2 remain, both with NAMED-deferred mitigation
> destinations).
>
> **Composition with adjacent invariants:**
> - **Inv-20 clause-d** (per-recipient unlinkability): preserved at
>   the cryptographic layer (per-stanza HPKE); EXTENDED at the
>   transport layer via D1 topic-id blinding. Recommended
>   doc-amendment to Inv-20 clause-d adds the transport-layer
>   clause.
> - **Inv-20 clause-b** (FORK-ONLY rotation): D2 reuses the SAME
>   rotation cadence; no new rotation machinery.
> - **Inv-21 candidate** (fork-tie-break HARD partition; M-C3 v2
>   F-FE-3): concurrent forks each derive distinct topic-ids;
>   losing fork's topic frozen for audit, alive briefly during
>   tie-break decision (e2e pin per pim-2 covers the ~60s
>   quiescence requirement).
> - **Compromise #48** (MembershipSet-shape-leak): #59 R3 + R4
>   are clauses of #48; recovery path is shared.
> - **Compromise #53-narrowed** (TransportConfig codepoint-
>   reserve): #59 ADOPTED-MITIGATION extends the
>   GossipPlusBlobs codepoint-reserve to a specific
>   privacy-respecting derivation, not just a wire-shape.
> - **Q3 Option D K_Atrium-blinding** (`23f76e24`): #59 is the
>   second-surface application of the SAME K_DedupScope =
>   HMAC-PRF-key parameter; one key, two surfaces, both blinded.

### §8.2 Absorption-into-#53-narrowed: REJECTED

The orchestrator brief asked whether #59 should mint or be absorbed
into #53-narrowed. **RECOMMEND MINT (not absorb).** Reasoning:
- #53-narrowed is about WIRE-SHAPE freeze + codepoint-reserve
  semantics; it answers "what TransportConfig codepoints ship at
  v1-beta?"
- #59 is about CRYPTOGRAPHIC-DISCIPLINE-OF-TOPIC-ID-DERIVATION;
  it answers "given GossipPlusBlobs ships, how is the topic-id
  derived to preserve privacy?"
- These are ORTHOGONAL axes; absorbing would muddle the
  auditor-clarity of either.
- Companion-pair shape: #53-narrowed + #59 + (Q3 Option D
  precedent) form a 3-Compromise cluster around
  "iroh-gossip-substrate at v1-beta privacy posture" — distinct
  rows, cross-referenced via §-pointer.

### §8.3 #48 doc-amendment (R3 + R4 clauses)

EDIT to existing Compromise #48 text (insert new clauses):

> ... [existing #48 prose about MembershipSet-shape-leak under
> K_Set compromise] ...
>
> **Additional disclosure clauses under #48 (added by #59 mint
> 2026-05-28):**
> - **Clause #48-c (topic-id history):** if K_Set leaks, all past
>   gossip topic-ids derived by `derive_gossip_topic_id(K_Set, ...)`
>   become recomputable; an adversary with historical relay
>   observation logs can retroactively label observed topic-ids.
> - **Clause #48-d (bootstrap-peer-list):** if a member leaks
>   their copy of the K_Set distribution envelope, the
>   `bootstrap_peers: BTreeSet<NodeId>` field is also leaked.
>   Join attempts by non-members still fail at envelope-decrypt
>   (lack K_Set); the leak's primary cost is connection-graph
>   exposure to the adversary.
> - **Recovery path (shared with primary #48):** FORK-ONLY
>   rotation per Inv-20 clause-b. Post-rotation topic-ids are
>   uncorrelated to pre-rotation; adversary's forward-visibility
>   ends at K_Set-leak moment.

### §8.4 Inv-20 clause-d doc-amendment

EDIT to existing Inv-20 clause-d wording:

> **Inv-20 clause-d (per-recipient unlinkability):**
> per-recipient unlinkability **at the cryptographic layer** — the
> SAME content addressed to multiple recipients is encrypted-
> stanza-distinctly per-recipient, with no cross-recipient
> correlation observable to anyone lacking the per-recipient
> HPKE-Encap key. **At the transport layer (Wave-MS-TRANSPORT
> GossipPlusBlobs / GossipRequiredBlobsFallback configs):** the
> gossip topic-id is HMAC-blinded per Compromise #59 derivation,
> closing the MCV2-C-3 transport-layer-fingerprint contradiction;
> see #59 for the specific derivation, residual disclosure, and
> NAMED-deferred mitigation destinations for residuals R1/R2.

---

## §9 Open questions / Ben-call surface

### §9.1 Surface to Ben (4 calls)

Per `feedback_surface_arch_decisions_under_auth.md` and per the
plain-English-with-prediction lens — the following are real
architectural forks that benefit from Ben's confirm-or-redirect.

**Q9.1 — D6 modified-hybrid adoption vs alternatives.**
- **Plain English:** Should we ship D1 + D2 + D5 (HMAC-blind +
  FORK-ONLY rotation + UCAN-Drop rendezvous) as the v1-beta
  MITIGATION for #59, OR ship #59 as DISCLOSURE-ONLY and defer
  mitigation to Phase-5+?
- **My prediction:** **ADOPT D6 at v1-beta** (~0.7 wave-days; fits
  within existing 5-8 wave-day Wave-MS-TRANSPORT envelope; closes
  Inv-20 clause-d transport-layer violation that v1-beta SCOPE-IN
  otherwise leaves open).
- **Alternative:** Ship as DISCLOSURE-ONLY; defer to Phase-5+
  contingent on mixnet substrate maturation.

**Q9.2 — Inv-20 clause-d doc-amendment vs Inv-21 candidate mint.**
- **Plain English:** Should the transport-layer-unlinkability
  property added under D6 be an AMENDMENT to Inv-20 clause-d (as
  proposed in §8.4 above), OR a SEPARATE new invariant
  (Inv-N-transport-layer-unlinkability)?
- **My prediction:** **AMEND clause-d** (per §8.4 wording). The
  property is the SAME unlinkability axis extended to a second
  layer; minting Inv-N would create artificial separation between
  semantically-paired layers.
- **Alternative:** mint Inv-N if Ben wants explicit per-layer
  invariant boundaries.

**Q9.3 — #48 amendment vs #59 R3+R4 as #59-internal clauses.**
- **Plain English:** Should the K_Set-leak topic-id-history
  residual (R3) and bootstrap-peer-list (R4) be CLAUSES of
  Compromise #48, OR be entirely #59-internal?
- **My prediction:** **CLAUSES of #48** (per §8.3). The threshold
  (K_Set compromise) is identical; recovery path is shared; auditor
  wants ONE row to reason about for the "what does K_Set leak
  expose?" question.
- **Alternative:** keep #59-internal if Ben prefers tight residual-
  scoping per Compromise row.

**Q9.4 — DEPLOYMENT-OVERRIDE for `gossip_topic_id: Some(bytes)`
production posture.**
- **Plain English:** Should explicit `Some(bytes)` `gossip_topic_id`
  override be (a) HARD-PROHIBITED in production (deny in non-test
  builds), (b) WARN-and-allow (current §7.2 sketch), or
  (c) silently allow?
- **My prediction:** **WARN-and-allow at v1-beta** + named for
  future hardening when use-cases for the override become clear.
  Hard-prohibit would surprise legitimate-test/interop callers;
  silent-allow would erase the audit trail for an obvious
  privacy-footgun.
- **Alternative:** Hard-prohibit if Ben prefers strict invariant
  enforcement (would need `#[cfg(test)]`-gated test override).

### §9.2 NIT — orchestrator brief input asymmetry

The brief named `.addl/phase-4-meta/SESSION-2026-05-20-to-2026-05-21-
substrate-wave-and-willow-pivot.md` and
`.addl/spikes/SPIKE-C-iroh-gossip-2026-05-21.md` as inputs. Neither
exists at HEAD `2172cb6d` nor in `git log --all` on any branch I
could enumerate via `git log --all --oneline | grep -iE
"session-2026-05-2|spike-c-iroh"`. M6 §3 references "Spike C
empirical data" + cites `/tmp/benten-spike-C-iroh-gossip/` (an
ephemeral location). Substantive content I needed from those
artifacts (latency numbers, footgun enumeration, n0 chat-rs
precedent) was AVAILABLE in M6's §3 + §3.1 + §4 + §7 — those
sections appear to have ABSORBED the load-bearing Spike-C and
SESSION-2026-05-20 findings. **NIT** — surfaced for the
orchestrator's eventual record-keeping per
`feedback_review_finding_ground_truth_verify.md`; does NOT block
this output (the content was found via the M6 references).

### §9.3 Wave-MS-TRANSPORT cost re-estimate after D6

Per M-CONS-v2 §7.4 baseline (5–8 wave-days central; 4.5–9 bracket)
+ §7.5 D6 delta (~0.7 wave-days):
- **New central estimate:** ~5.5–8.7 wave-days.
- **Bracket:** stays at 4.5–9 (D6 does not widen the bracket).
- **#59 mitigation tax:** ~10% of Wave-MS-TRANSPORT cost; well
  within the "MITIGATE not merely DISCLOSE" affordability budget.

### §9.4 What this doc does NOT address

- **The actual iroh-gossip integration spike (M-CONS-v2 §7.4
  items 1–6 beyond topic-id-derivation):** out of scope; covered
  by M-CONS-v2 § 7.4 baseline + M6 § 10.1 deliverables 1–12.
- **D4 mixnet integration design:** NAMED-deferred per §5.4;
  not designed here.
- **AS-level / global passive adversary defense:** explicitly
  acknowledged as residual (R1 + R2); requires D4.
- **iroh-gossip per-topic membership ACL:** non-existent at
  upstream; not designed here. Our D5 (out-of-band rendezvous via
  UCAN-Drop) is the application-layer ACL substitute.
- **CGKA / DCGKA / MLS-PQ as an alternative substrate to FORK-ONLY:**
  per Compromise #54 deferral; out of scope for this doc.

---

## Sources

Web research consulted (2026-05-28):

- iroh-gossip official docs: [docs.iroh.computer/connecting/gossip](https://docs.iroh.computer/connecting/gossip)
- iroh-gossip repo + proto rustdoc: [github.com/n0-computer/iroh-gossip](https://github.com/n0-computer/iroh-gossip)
- n0-computer iroh authorization discussion: [github.com/n0-computer/iroh/discussions/3168](https://github.com/n0-computer/iroh/discussions/3168)
- iroh security/privacy deployment: [docs.iroh.computer/deployment/security-privacy](https://docs.iroh.computer/deployment/security-privacy)
- Distributed topic tracker pattern: [rustonbsd.github.io/2025/09/03/distributed-topic-tracker.html](https://rustonbsd.github.io/2025/09/03/distributed-topic-tracker.html)
- Waku content topics: [docs.waku.org/learn/concepts/content-topics/](https://docs.waku.org/learn/concepts/content-topics/)
- Waku Relay anonymity analysis: [research.logos.co/rlog/wakuv2-relay-anon/](https://research.logos.co/rlog/wakuv2-relay-anon/)
- Waku family-of-protocols paper: [arxiv.org/pdf/2207.00038](https://arxiv.org/pdf/2207.00038)
- Tor v3 rendezvous specification: [torproject.gitlab.io/torspec/rend-spec-v3.html](https://torproject.gitlab.io/torspec/rend-spec-v3.html)
- libp2p GossipSub / Episub specs: [github.com/libp2p/specs](https://github.com/libp2p/specs/tree/master/pubsub)
- MSC4291 Matrix room IDs as hashes: [matrix-org/matrix-spec-proposals MSC4291](https://github.com/matrix-org/matrix-spec-proposals/blob/matthew/msc4291/proposals/4291-room-ids-as-hashes.md)
- Signal Sealed Sender: [signal.org/blog/sealed-sender](https://signal.org/blog/sealed-sender/)
- Martiny et al improving Sealed Sender (NDSS 2021): [cs.umd.edu/~kaptchuk/publications/ndss21.pdf](https://www.cs.umd.edu/~kaptchuk/publications/ndss21.pdf)
- No safety in numbers (sealed-sender group traffic analysis): [arxiv.org/abs/2305.09799](https://arxiv.org/abs/2305.09799)
- Signal Private Group System (KVAC anonymous credentials): [eprint.iacr.org/2019/1416](https://eprint.iacr.org/2019/1416.pdf)
- RFC 9750 MLS architecture: [rfc-editor.org/rfc/rfc9750](https://www.rfc-editor.org/rfc/rfc9750.html)
- RFC 9420 MLS protocol: [datatracker.ietf.org/doc/html/rfc9420](https://datatracker.ietf.org/doc/html/rfc9420)
- Inherent anonymity of gossiping (arxiv 2308.02477): [arxiv.org/pdf/2308.02477](https://arxiv.org/pdf/2308.02477)
- Differential-privacy of gossip protocols (arxiv 1902.07138): [arxiv.org/pdf/1902.07138](https://arxiv.org/pdf/1902.07138)
- Privacy guarantees of gossip in general networks (arxiv 1905.07598): [arxiv.org/pdf/1905.07598](https://arxiv.org/pdf/1905.07598)

Input artifacts (Benten Engine internal; read from git objects):
- M-CONS-v2 consolidator `e62ff540` §7.4 transport-config — read.
- M-C2 v2 cross-amendment critique `0533d0cf` §3.3 MCV2-C-3 — read.
- M-C3 v2 fresh-eyes cryptographer `cac10631` §3.1 #59 candidate — read.
- M6 transport-configurability `e5046c87` §3 + §5.2 + §6.1 + §7 + §10 — read.
- Q3 revisit Option D community lens `23f76e24` §1 + §2 + §3 — read.
- SESSION-2026-05-20 substrate-wave-and-willow-pivot — **NOT FOUND** at HEAD or in `git log --all`; content absorbed by M6 §3 references; NIT'd at §9.2.
- SPIKE-C-iroh-gossip-2026-05-21 — **NOT FOUND** at `.addl/spikes/`; content absorbed by M6 §3 + §3.1 references; NIT'd at §9.2.
