# M6 — Transport-Configurability per MembershipSet for Benten Engine

**Pipeline:** MembershipSet unification specialist panel — M6 of 5 parallel specialists.
**Author:** orchestrator-dispatched p2p-networking architect on branch `phase-4-meta-core/membership-set-m6-transport-configurability`.
**Tree-state pre-flight:** main HEAD `2172cb6d`, worktree clean, branch parent = `2172cb6d`.
**Sibling specialists at dispatch:** M1a (multi-device-sync; LIVE at branch `…cataloger-m1a-multi-device-sync` @ `3618e051`), M1b (Atrium-membership-sharing; @ `1816ea60`), M1c (key-management; @ `50eb901d`). M2/M3/M4/M5 status pending; this doc mark-`M[N]-pending` where dependency exists.
**Brief framing:** broaden from "iroh-gossip codepoint-reserve-at-v1-beta" (Q3 specialist 2026-05-27 @ `23f76e24`) to "transport-configurability per MembershipSet" — does iroh-gossip replace / complement / compose with current iroh-blobs stack; what becomes per-MembershipSet config; does it enable "faster sync" for live collaboration.

---

## §1 Executive recommendation + confidence

**TL;DR** (one paragraph): Benten today uses **iroh QUIC core + iroh-blobs pairwise + MST/Loro pull-driven sync**. iroh-gossip does NOT replace what we have — it **layers on top as a latency-cut notification overlay** for the happy-path online case while iroh-blobs / MST-diff remain the durable pull substrate for offline-resync correctness. "Faster sync for live collaboration" is achievable via gossip-announce → blobs-pull pattern (Spike C measured ≤2 ms gossip RTT in 3-peer mesh; vs current ~poll-driven 0.5-30 s pull-after-merge latency depending on cadence), BUT true sub-100 ms collaborative-editing semantics (cursor-position, presence) need either gossip-presence-overlay OR a separate iroh-roq/iroh-live transport — these are POST-v1-beta scope. **Transport-configurability per MembershipSet is the right framing**: add a `MembershipSet::transport_config` field carrying `{primary: TransportKind, fallback: Option<TransportKind>, gossip_topic_id: Option<[u8;32]>}` with **defaults auto-selected from MembershipSet kind + projected size**. For v1-beta we recommend: **codepoint-reserve only** (NOT scope-include impl), with the `TransportConfig` type + wire shape frozen at v1-beta (so v1.x can add gossip without re-freezing the envelope), and **3-instance evidence ratification** for full iroh-gossip impl at Phase-4-Meta-Composing or Phase-5 (NOT v1-beta-Core). Willow Confidential Sync is correctly parked Phase-5+ Kith watch-list; iroh-roq/iroh-live are strictly post-v1-beta.

**Confidence per section:**
- §2 current stack: **HIGH** (read INTERNALS.md + crate inventory + M1a citations directly).
- §3 gossip vs blobs: **HIGH** (Spike C empirical data + WebFetch n0-computer 2026 docs).
- §4 live-collab analysis: **MEDIUM-HIGH** (sub-100 ms target is application-class specific; benchmark numbers from Spike C are 3-peer mesh, not 50-peer).
- §5 per-MembershipSet matrix: **MEDIUM** (depends on M2 primitive design; some rows are extrapolation from Spike C 3-peer + iroh-gossip's documented "few thousand peers" scaling claim).
- §6 default-selection heuristic: **MEDIUM** (UX-coupled; would benefit from M3/M4/M5 input).
- §7 v1-beta scope-vs-codepoint-reserve: **HIGH** (cost estimate grounded in Spike C "~330 LOC harness" + Phase-3 wave-sizing precedents).
- §8 Willow composition: **MEDIUM** (depends on Willow Confidential Sync spec stabilization — currently demoted Final→Proposal 2025-10-26).
- §9 iroh-roq/iroh-live composition: **HIGH-as-deferral** (clear deferral; LOW confidence on actual integration design).
- §10 final design: **MEDIUM** (recommendation shape clear; final field layout depends on M2 primitive).

---

## §2 Current Benten transport stack clarification

Per M1a §2.2 + `crates/benten-sync/INTERNALS.md` + Spike A/A2/C verification, **today** Benten uses:

### 2.1 What we have at HEAD `2172cb6d`

| Layer | Crate / file | Status | Notes |
|---|---|---|---|
| **Peer-to-peer connections** | `iroh` 1.0.0-rc.0 QUIC, `iroh-relay` fallback, `iroh-dns` discovery | LIVE | `crates/benten-sync/src/transport.rs` (~825 LOC) + `peer_discovery.rs` (~364 LOC). ALPN `b"benten/atrium/1"`. `BootstrapMode { DefaultRelay, CustomPeerList, Disabled }`. Per CLAUDE.md baked-in #17 native-only (browser tab talks thin-client over HTTPS+SSE/WS). `PeerId == iroh EndpointId == Ed25519 pubkey bytes` per crypto-minor-4 / net-minor-2. |
| **DID-mutual-auth handshake** | `handshake.rs` (~1276 LOC) + `handshake_wire.rs` (365 LOC) | LIVE (Phase-3 G16-D wave-6b) | 3-step: `Initiate` → `Respond` → `Finalise`. **Both `peer_did` AND `device_did` REQUIRED at wire** per net-blocker-4, enforced at compile time via type-state builder. Handshake-time revocation-set UNION per net-r4-r1-3 (responder seals snapshot before returning `Session`). `DEFAULT_REPLAY_WINDOW_MS = 5000`. |
| **Content-addressed bulk transfer** | NOT iroh-blobs as a runtime dep yet — current code uses custom MST + Loro CRDT byte-deltas over iroh QUIC streams (uni-stream open / write / `stopped()` ACK; 4 MiB cap) | LIVE for sync replicas; UCAN-gated `iroh-blobs` ALPN handler shape exists at `ucan_blobs_protocol.rs` + `two_cid_store::TwoCidStore` swap-point (G-CORE-3e wave-3e) | This is the **most important framing correction** — Benten does NOT yet ship iroh-blobs the way the brief framing implied. We have the *swap-point* (Flavor B per-request UCAN check ALPN handler + in-memory adapter) for the production FsStore wire-up, but the actual bulk-transfer-over-blobs production wire-up is Phase-4-Meta-Core scope, NOT Phase-3 closed. Spike A2 verified iroh-blobs works for our shape; the swap is small. |
| **"What's new since last sync" diff** | `mst.rs` (~743 LOC) + `mst_proto.rs` (~472 LOC) | LIVE | Merkle Search Tree (BTreeMap-backed; deterministic root via DAG-CBOR + BLAKE3). `MstDiffSession` two-tier queue (revocation_queue + data_queue) per net-blocker-3 invariant. `MerkleProof` is O(n) (Phase-4-Meta-Composing optimization-pin → O(log n)). Application-layer rehash defends sec-r4r2-1. |
| **CRDT per-property merge** | `crdt.rs` (~947 LOC) Loro 1.12 + `BentenHlc` LWW | LIVE (D-PHASE-3-4) | Node-property granularity. **HLC carried INSIDE the value** (packed `<physical>:<logical>:<node>:<value>` string in single root List `benten:properties`) — Loro's internal Lamport NOT trusted for LWW (load-bearing). Rich types under `benten:rich:<name>`. `winning_attribution()` returns UNION of all observed write HLC node_ids (Inv-14 device-grain). |
| **Sync runtime** | `engine_sync.rs` (~2135 LOC) + `apply_atrium_merge` row-loop with structural-always-on per-row cap-recheck | LIVE | Per-zone Loro CRDT documents. Manifest-envelope-recheck (R4b-FP-1 Seam 3). `AtriumHandle::{open, accept_invite, leave, rejoin, subscribe, apply_atrium_merge, sync_subgraph, merge_remote_change, ...}`. |

### 2.2 What is NOT implemented today

| Surface | State | Notes |
|---|---|---|
| **iroh-gossip integration** | NOT IN DEPS | Spike C @ 2026-05-21 verified the library works for our shape (Spike C `/tmp/benten-spike-C-iroh-gossip/`); no production code consumes it. Codepoint reservation at v1-beta planned per Q3 specialist `23f76e24`. |
| **iroh-docs (multi-peer KV layered on blobs+gossip)** | NOT IN DEPS | RESEARCH-atrium-transport-model-2026-05-20 §1 confirms exists at `iroh-docs 0.99.0` but we have NOT adopted. |
| **iroh-willow (Willow + Meadowcap)** | NOT IN DEPS; assessed PARKED | iroh-willow assessment 2026-05-20 found effectively parked (0 substantive 2026 commits; pinned to iroh 0.34 incompatible with our 1.0.0-rc.0; 5/14 specs implemented; only 0.0.1 published). Willow ecosystem in flux: Confidential Sync demoted Final→Proposal 2025-10-26. |
| **iroh-roq (RTP-over-QUIC) / iroh-live (media livestreaming)** | NOT IN DEPS | Identified as Spike-D candidate (deferred until after S&C-informing spikes). Real-time-media-class transport; not on v1-beta critical path. |
| **Live broadcast / gossip-style fan-out** | NOT IMPLEMENTED | We use pairwise-only handshake → pairwise CRDT delta exchange. There is no "broadcast to all Atrium members" primitive at HEAD; the closest is iterate-Atrium-members + pairwise `merge_remote_change` per peer. |
| **Real-time collaborative editing semantics** | NOT IMPLEMENTED in the "sub-100 ms cursor / presence" sense | We have eventual-convergence via Loro CRDT (correctness ✓) but no continuous-stream low-latency push (live UX ✗). |
| **`broadcast_neighbors()` / topic-overlay primitive** | NOT IMPLEMENTED | We have neither HyParView-style mesh nor a TopicId concept at HEAD. |

### 2.3 The actual sync model TODAY (plain English)

A peer P1 has a change → writes to local Loro doc (per-property HLC stamp). When P1 wants to sync to P2: pairwise handshake (3-step DID-mutual-auth) → MST diff exchange → CRDT byte-delta exchange → per-row cap-recheck on merge. **This is poll-driven / on-demand / sync-triggered, NOT broadcast-driven.** If P2 was offline during P1's write, P2 catches up on its next sync invocation. There is no "P1 broadcasts the change immediately to the whole Atrium" path.

The current latency from change-on-P1 to convergence-on-P2 is **bounded by sync invocation cadence** (the calling code in `benten-engine` calls `merge_remote_change` per peer on whatever rhythm the application sets). For an Atrium of N members, P1's change reaches everyone in N pairwise sync invocations — O(N) in the originator's bandwidth + connection count, which is exactly where gossip's broadcast amplification (Plumtree's eager-push tree) would beat us by O(log N) at the originator.

**This confirms the brief's premise**: gossip would enable a notification-fan-out the current stack lacks. But the corollary is also true — the current stack provides durable-correctness-when-offline that gossip alone cannot.

---

## §3 iroh-gossip vs iroh-blobs technical comparison

Sources: Spike C empirical (2026-05-21) + WebFetch n0-computer 2026 docs + iroh-gossip CHANGELOG / repo @ 0.99.0.

### 3.1 Property comparison matrix

| Property | iroh-gossip 0.99 | iroh-blobs 0.101 |
|---|---|---|
| **Propagation model** | Push (epidemic broadcast tree; HyParView membership + Plumtree dissemination) | Pull (request/response; receiver initiates) |
| **Latency** (3-peer local mesh, Spike C Exp 1b) | 1-2 ms per-message (200 B–3 KB payloads) over default n0 relay; sub-ms LAN-only with `--relay disabled` | Per-blob fetch latency = handshake + BLAKE3-verified streaming setup (~tens of ms; dominated by stream open + first-byte; throughput-limited after) |
| **Throughput** | Best for small messages; **4 KB DEFAULT_MAX_MESSAGE_SIZE** (configurable per-swarm); oversize silently dropped at writer per Spike C footgun | Best for kilobyte–terabyte blobs (per n0-computer Blobs docs); BLAKE3 streaming verification + interrupt/resume |
| **Dedup mechanism** | Plumtree IHAVE/IWANT at protocol layer (peers track recently-seen message IDs; lazy-push for tree repair) — **Q3 specialist 23f76e24 specifically notes this enables ~few-thousand-member Atriums vs ~50-100 with blobs-only when keyed by K_Atrium-blinded plaintext_cid** | Content-CID at content-addressing layer (BLAKE3 hash = identifier; if you have the CID you have a stable identity; duplicate-fetch elision per content-store) |
| **Reliability** | Eventual; **no message persistence; offline peers permanently miss**; ~30-35 s `NeighborDown` detection latency after hard crash (Spike C Exp 4/4b) | Deterministic-once-published; content lives in publisher's store; receiver can fetch on any later schedule as long as publisher (or another holder) is reachable |
| **Bandwidth cost** | Plumtree amplification ~5-15× raw fan-out (overhead for tree healing); cheap for small text; **DO NOT use for blob-class payloads** (use as "hey there's new stuff" notification, fetch via blobs) | Single-source unicast; receiver pays full payload bandwidth from publisher (or any chunk-source per BLAKE3 streaming); no protocol amplification |
| **Battery cost on mobile** | Higher resting (mesh maintenance / Shuffle messages @ 60 s interval / keep-alive); ~3 s relay-online cold-start | Lower resting (no protocol unless actively fetching); higher per-fetch (single connection + streaming) |
| **Connection model** | Mesh (HyParView active view + passive view; bootstrap requires ≥1 EndpointAddr to dial; topology partially shaped by bootstrap order) | Pairwise request-response; one connection per blob-source (multiple sources possible per-CID with content discovery) |
| **Per-topic membership ACL** | **NONE — whoever knows the TopicId can subscribe AND broadcast**; application-layer signing + membership-validation required (chat.rs precedent + Spike C verdict) | Pull-side caller verifies CID against expected; no concept of "broadcast-ACL" because there's no broadcast |
| **Sender authentication** | NONE at protocol layer; `delivered_from` = LAST HOP not originator; application-layer `SignedMessage { from, data, signature }` pattern (Spike C confirms canonical chat.rs example) | N/A (request initiated by receiver; content-CID hash IS the integrity guarantee) |
| **Ordering** | NO total order; NO causal order; per-peer slice of stream; each peer sees own broadcasts elided | N/A (request-response; receiver imposes any order it wants) |
| **Best-fit use cases** | Live notifications ("new revocation added"; "new change at CID X"); ephemeral presence ("alice typing"); transient signals; small-message coordination | Bulk content delivery; offline-resync; CRDT-state snapshots; sendme-class share-by-URL; UCAN-gated capability-bound transfer (G-CORE-3e shape) |

### 3.2 The unifying pattern (Spike C verdict §6 "Recommendation (a)")

> **The natural design: revocations live in a pull-shaped store (whatever that ends up being — a content-addressed log per Atrium, a CRDT, or a Loro-backed structure). Gossip pushes revocation-event NOTIFICATIONS announcing the addition. Peers who receive the gossip event can fetch the new revocation immediately; peers who miss the gossip (offline / late join) catch up on next poll.** Gossip is the latency-cut for online peers, not the durability layer.

**Applied to Benten:**

- Writes still go to the durable layer (Loro CRDT + MST diff exchange; future: iroh-blobs FsStore for blob-class payloads).
- A small "version N just shipped at CID X" gossip announcement on a per-Atrium topic notifies online subscribers within ~ms.
- Online subscribers immediately initiate pairwise `merge_remote_change` (current path) or pull the new blob from publisher's store (post-G-CORE-3e blobs wire-up).
- Offline subscribers reconcile on wake via the durable pull (no behavior change vs today).

**This is structurally additive**, not replacing anything we have. iroh-gossip becomes one more **engine extension** under the CLAUDE.md baked-in #19 trust model (Rust extension; compile-time trust; opt-in per deployment), wired into the existing handshake-time and merge-time gates.

---

## §4 Does iroh-gossip enable "faster sync" for live collaboration?

### 4.1 The "live collaboration" target — three latency classes

To answer Ben's "faster sync (like faster sync)" question rigorously, we need to separate three application-class targets:

| Class | Target latency | Current Benten | Gossip-enabled | Other transports needed? |
|---|---|---|---|---|
| **(A) Eventual sync** ("if I make a change, eventually everyone sees it") | seconds – minutes | YES (cadence-driven; depends on calling app's sync schedule) | Better — sub-second on happy path | NO |
| **(B) "Live feel" sync** ("when I save a doc, others see it within ~1 s") | ~hundreds of ms – 1 s | NO (depends entirely on calling cadence) | YES — gossip-announce ~ms + pairwise pull on receipt ~hundreds of ms | NO |
| **(C) True real-time co-edit** ("typing in shared cursor; presence; sub-100 ms; ~typing-feel CRDT merge") | <100 ms p95 | NO | PARTIALLY — gossip can carry tiny op-broadcasts (≤4 KB ceiling); but mesh repair + bootstrap + topology issues mean p95 100 ms target is fragile | YES — likely iroh-roq for media; possibly iroh-docs or a dedicated WTP-like protocol for op-streams; or Willow's RBSR + LCMUX for op-set reconciliation |

### 4.2 What gossip-enables, specifically

Concrete proposal for Class B "live feel" sync:

1. Per-Atrium gossip topic `TopicId = BLAKE3("benten/atrium/v1/" || atrium_root_cid || K_Atrium)` (the `K_Atrium`-mixing prevents passive observers from joining only-by-knowing-public-root; per Q3 specialist Option D community lens K_Atrium-blinding precedent).
2. On every write that lands in `apply_atrium_merge`, emit a `ChangeAnnounce { actor_did, change_cid, hlc, sig }` (~150–400 B; well under 4 KB ceiling) on the topic; receiver-side `SignedMessage` envelope per Spike C convention.
3. Subscribers receive the announcement at ~1-2 ms RTT (Spike C measurement) → immediately initiate `merge_remote_change` against publisher.
4. Total wall-clock change → other-peer-converges: gossip latency (~ms) + pairwise merge (~tens to hundreds of ms depending on subgraph size) = **~100-500 ms in happy path**, vs **0.5-30+ s today** (depending on calling app's sync cadence).

This is a **substantive UX improvement**. Class B is achievable; Spike C numbers support it.

### 4.3 What gossip CANNOT enable alone

- **Class C real-time co-edit / cursor / presence.** Three obstacles:
  - 4 KB ceiling forces op-fragmentation for non-trivial document edits (Loro op log entries can be larger than 4 KB for big text inserts);
  - No per-topic membership ACL → presence-leakage to anyone who knows the TopicId (a real privacy concern Spike C flagged); K_Atrium-blinding helps but doesn't solve the post-join leakage to internal observer-members;
  - 30-35 s NeighborDown detection means "is bob online?" via NeighborUp/Down alone is too slow → need application-layer heartbeat;
  - Mesh-bootstrap is partly random → topology-shape fragility for small N;
  - p95 latency under burst / churn is application-class-dependent; Spike C 50-broadcast burst delivered all 50 at max 2 ms latency in 3-peer mesh but did not stress at 20+ peers with mixed broadcasters.
- **Real-time media (audio/video/screensharing).** Out of scope for gossip entirely — this is iroh-roq + iroh-live territory.
- **Lossless op-stream catch-up.** Gossip has no message persistence; a peer that was offline misses broadcasts permanently → the durable pull (MST/Loro/blobs) STILL has to handle eventual correctness.

### 4.4 What additional infra would Class C need?

- **iroh-live / iroh-roq:** for any media use case (presence-with-camera; voice; screen). Both NOT on v1-beta critical path.
- **Application-layer heartbeat ride-along on gossip:** ~10-30 s ping for fast presence; mitigates the 30-s NeighborDown lag. Small adoption cost.
- **`broadcast_neighbors()` (iroh-gossip sibling of `broadcast()`):** sends only to direct neighbors without triggering tree-fan-out; useful for small-N Atriums where everyone is a direct neighbor.
- **A dedicated low-latency op-stream protocol (Willow LCMUX-like):** for sub-100 ms shared-editing of CRDT op-logs. POST-v1-beta; Phase-5+ Kith watch-list item.

**Verdict on §4 question:** YES, iroh-gossip enables a substantively faster Class B "live feel" sync. NO, it does not enable Class C true real-time co-edit alone — that needs additional infra deferred to Phase-5+.

---

## §5 Transport-configurability per MembershipSet kind/shape

### 5.1 Matrix (recommendation; depends on M2 primitive design)

| MembershipSet shape | Recommended primary transport | Recommended fallback | Reasoning |
|---|---|---|---|
| **SingleDevice** (no peers; engine-local only) | `None` | `None` | No sync needed; no transport spent. |
| **DeviceMesh, 1-3 devices, low-activity** | `blobs+mst` (current default) | `None` | Small mesh; pairwise pull on cadence is fine; gossip overhead (mesh + relay-online cost) not worth it for ≤3 endpoints. |
| **DeviceMesh, 3-10 devices, collaborative-edit-priority** | `gossip+blobs` | `blobs+mst` | Broadcast notification latency matters when multiple devices may edit concurrently; gossip-announce → pairwise-pull gives Class B "live feel" sync. |
| **DeviceMesh, ≥10 devices** (unusual; user with many devices) | `gossip+blobs` | `blobs+mst` | Same as above; gossip's O(log N) broadcast amplification beats O(N) pairwise originator cost. |
| **Atrium, 2-20 members, eventual-OK** | `blobs+mst` | `None` | Simple; current code suffices; no operational complexity from gossip mesh. |
| **Atrium, 20-200 members, collaborative-edit-priority** | `gossip+blobs` | `blobs+mst` | Scaling matters: pairwise O(N) cost on originator wastes bandwidth; gossip Plumtree's 5-15× amplification is cheaper than N pairwise sends for N > ~10. |
| **Atrium, 200-2000 members** | `gossip+blobs-fallback` | `blobs+mst` | Gossip scales (Q3 specialist: "iroh-gossip enables ~few-thousand-member Atriums vs ~50-100 with blobs-only"); blobs handles offline-rejoin durably. Heartbeat ride-along recommended; 4 KB ceiling pre-check mandatory. |
| **Atrium, 2000+ members** | `gossip-required + blobs-fallback` | `blobs+mst` | Pairwise pull is genuinely impractical at this scale; gossip-required with explicit fallback. May want to negotiate raised `max_message_size` per swarm. Approaches the limit of "few thousand peers" upstream claim. |
| **Atrium-of-Atriums (Garden; recursive Kith Phase-7+)** | per-sub-Atrium config; recursive resolution | per-sub-Atrium fallback | Each leaf-Atrium picks its own transport; outer Garden routing is application-layer composition of sub-Atrium transports. |

### 5.2 Per-MembershipSet config field design (proposal pending M2)

```rust
/// Per-MembershipSet transport selection (Phase-4-Meta-Core v1-beta-frozen SHAPE; impl deferred).
#[non_exhaustive]
pub struct TransportConfig {
    /// Primary transport for this MembershipSet's sync.
    pub primary: TransportKind,
    /// Optional fallback when primary is unreachable / unsupported.
    pub fallback: Option<TransportKind>,
    /// When `primary == Gossip` or `GossipPlusBlobs`, the topic identifier.
    /// `None` = compute deterministically from MembershipSet root + K_Atrium-blind.
    pub gossip_topic_id: Option<[u8; 32]>,
    /// Sub-MembershipSet inheritance policy (relevant for Garden / Atrium-of-Atriums).
    pub sub_inherit: SubInheritPolicy,
    /// Reserved-for-extension; codepoint-dispatch space for future Willow / iroh-roq / iroh-live.
    pub extension: Option<TransportExtension>,
}

#[non_exhaustive]
pub enum TransportKind {
    /// No transport; engine-local only (SingleDevice; pre-Atrium scratch).
    None,
    /// Pairwise iroh-QUIC + MST/Loro pull (current Phase-3 default).
    BlobsPullMst,
    /// Gossip-announce notifications + pairwise pull on receipt.
    GossipPlusBlobs,
    /// Gossip required for primary path; pairwise pull only for offline-rejoin.
    GossipRequiredBlobsFallback,
    /// Reserved codepoint slot — Phase-5+ Kith watch-list (Willow / iroh-roq / iroh-live).
    Reserved(u16),
}

#[non_exhaustive]
pub enum SubInheritPolicy {
    /// Sub-MembershipSets inherit parent's TransportConfig by default.
    Inherit,
    /// Sub-MembershipSets choose own; parent's TransportConfig does NOT propagate.
    Independent,
    /// Sub-MembershipSets may override but inherit unless they do (current default recommendation).
    InheritWithOverride,
}

#[non_exhaustive]
pub enum TransportExtension {
    /// Willow Confidential Sync — Phase-5+ Kith watch-list (currently Proposal-status).
    WillowConfidential { spec_version: u8 },
    /// iroh-roq RTP-over-QUIC — real-time media stream.
    IrohRoq { rtp_profile: u8 },
    /// iroh-live media livestreaming over MoQ.
    IrohLive { moq_namespace: [u8; 32] },
}
```

**Canonical CBOR (proposal):**

```cbor
TransportConfig = {
  0: TransportKind,            ; primary
  1: TransportKind / null,     ; fallback
  2: bstr .size 32 / null,     ; gossip_topic_id
  3: SubInheritPolicy,         ; sub_inherit (uint discriminator)
  4: TransportExtension / null ; extension
}
```

This is **wire-format-affecting** — it goes in the MembershipSet's canonical bytes (whatever M2 settles on) and thus must be v1-beta-frozen if we want post-v1-beta extension without re-freezing. **HENCE the codepoint-reserve recommendation at §7.**

### 5.3 Caveat: matrix assumes M2 primitive shape

The matrix above presumes M2 lands a `MembershipSet` primitive with at-minimum: a `kind` discriminator (SingleDevice / DeviceMesh / Atrium / Garden), a `member_set: BTreeSet<Did>`, a `root_cid: Cid`, and an extensibility seam. If M2 produces a meaningfully different shape, the matrix translates row-by-row but the field-layout proposal may need to follow M2's enum naming. Flagged as **M2-dependent**; final field names await M2.

---

## §6 Default-selection heuristic

When a user creates a new MembershipSet, what defaults?

### 6.1 Decision tree (recommendation; UX-coupled; depends on M2/M3/M4/M5 feedback)

```
MembershipSet::new(kind, ...) →
  match kind {
    SingleDevice              → TransportKind::None
    DeviceMesh                → BlobsPullMst   // safe default; current Phase-3 behavior
    Atrium(meta: AtriumMeta)  → match meta.projected_size {
        Some(n) if n >= 200   → GossipPlusBlobs     // crosses scaling threshold
        Some(_)               → BlobsPullMst        // small-to-medium Atrium
        None                  => BlobsPullMst       // unknown → safe default
      }
    Garden                    → SubInheritPolicy::Inherit + per-sub config
  }
```

### 6.2 User-explicit-configurable

YES — users / deployments should be able to override via a builder pattern:

```rust
Engine::create_atrium("project-x")
    .with_transport(TransportConfig::gossip_plus_blobs())
    .with_projected_size(150)
    .build()?
```

Defaults are correctness-preserving; explicit overrides are how a deployment that knows its workload (e.g. "this is a 500-person company Atrium with active collab editing") opts up.

### 6.3 Auto-promotion / observability

A future enhancement (Phase-4-Meta-Composing or Phase-5): the engine OBSERVES per-Atrium sync workload (peer count, message rate, p95 sync lag) and SURFACES TO USER a recommendation to switch transport configuration when threshold-crossing detected. NOT a v1-beta requirement. Named for `docs/future/phase-4-backlog.md §4.XX` post-merge.

### 6.4 Phase placement

- **Default-selection LOGIC:** Phase-4-Meta-Core (frozen shape). The defaults table goes in `crates/benten-engine/src/atrium_config.rs` (new file).
- **User-explicit-override BUILDER API:** Phase-4-Meta-Composing (UX-coupled; sits in `bindings/napi/src/` + `packages/engine/src/atrium.ts`).
- **Auto-promotion OBSERVABILITY:** deferred to Phase-5 or later.

---

## §7 v1-beta wire-format implications + scope-inclusion vs codepoint-reserve recommendation

### 7.1 The fork

Two options per Ben's broadened framing:

| Option | Description | v1-beta wave-days | Phase-4-Meta-Composing wave-days | v1-beta risk |
|---|---|---|---|---|
| **A: codepoint-reserve only** (Q3 specialist's original recommendation) | Freeze `TransportConfig` enum + canonical CBOR codepoints at v1-beta; **NO production iroh-gossip wire-up**. v1.x post-v1-beta can add gossip impl without re-freezing wire shape. | ~2-3 wave-days (type + canonical-bytes + V1-FROZEN row + codepoint-dispatch + 1 RED-PHASE staged-pin) | ~6-10 wave-days (full impl: gossip-publish at apply_atrium_merge + announce-receive at merge_remote_change + 4 KB ceiling pre-check + K_Atrium topic derivation + heartbeat + tests) | LOW — wire shape frozen; impl deferred carries no v1-beta scope risk |
| **B: scope-include iroh-gossip impl at v1-beta** | Land both the wire shape AND the production iroh-gossip wire-up at v1-beta. | ~8-13 wave-days | (collapsed into A) | MEDIUM — adds Phase-4-Meta-Core scope at a stage already at 5-6.5K LOC; pulls a new external dep + new ALPN; needs its own ADDL pipeline R0→R1→R2→R3→R4→R5→R4b before R6 R3 can converge per ADDL-pipeline-full-observance (`feedback_addl_pipeline_full_observance.md`) |

### 7.2 Recommendation: Option A (codepoint-reserve only)

**Rationale (the elegant-shape pass; 4 reasons):**

1. **Phase-4-Meta-Core already at ~5-6.5K LOC** per CLAUDE.md NIGHT-SHIFT-2026-05-27 F-full ratification. Adding ~10 wave-days of iroh-gossip impl pushes the phase past the "manageable wave" threshold per phase-ordering-precision and bumps the v1-beta tag.
2. **The codepoint-reserve choice is FULLY reversible UP** (we can scope-include impl in a later v1.x without re-freezing wire shape) but the inverse is NOT (scope-including impl at v1-beta and then needing to revert is expensive). Per Strategy-C and the agent-economics discipline (`feedback_agent_economics_prefer_thorough_cleanup.md`), preferring the reversible-up choice is correct.
3. **Spike C revealed real footguns** (4 KB silent-drop ceiling; 30-s NeighborDown lag; no per-topic membership ACL; bootstrap requires out-of-band peer info; mesh topology partly random for small N) — each is mitigable but each needs its own pim-N-class discipline + test pin. Authoring those disciplines IN the v1-beta-Core scope risks a rushed Mitigation harness; pushing impl to post-v1-beta lets each discipline ratify properly.
4. **Willow Confidential Sync / iroh-roq / iroh-live cohabit the same `TransportExtension` codepoint slot.** Freezing the slot at v1-beta WITHOUT a particular impl means the slot accommodates whichever transport(s) mature first in the Phase-5+ window — which is genuinely uncertain (Willow Confidential Sync just demoted Final→Proposal in 2025-10).

### 7.3 What v1-beta MUST include (scope-include subset)

- `TransportConfig` type (just the type + canonical CBOR + V1-FROZEN-INTERFACE row).
- `TransportKind::{None, BlobsPullMst, GossipPlusBlobs, GossipRequiredBlobsFallback, Reserved(u16)}` codepoint reservation.
- `TransportExtension::{WillowConfidential, IrohRoq, IrohLive}` extension-slot codepoint reservation.
- Default-selection logic for SingleDevice / DeviceMesh / Atrium (the rows in §5.1 that pick `BlobsPullMst` today — which is the current Phase-3 behavior).
- Engine refuses to use `GossipPlusBlobs` / `Reserved` codepoints at v1-beta with `E_TRANSPORT_KIND_UNSUPPORTED` error (engine-layer typed error; named in `benten_errors::ErrorCode`).
- RED-PHASE staged-pin for the gossip path: `atrium_gossip_announce_propagates_change_within_2s.rs` `#[ignore]`-named pin, destination `docs/future/phase-4-backlog.md §4.XX-NEW`.

**Wave-day cost: ~2-3 wave-days for v1-beta-Core.**

### 7.4 What v1-beta MUST NOT include

- Actual iroh-gossip dep in benten-sync `Cargo.toml`.
- Production gossip-publish / gossip-subscribe code paths.
- Per-Atrium TopicId derivation that uses K_Atrium-blinding (this needs the K_principal store from Layer-A to land first; ordering matters).

---

## §8 Willow Confidential Sync composition (Phase-5+ Kith watch-list)

### 8.1 What Willow Confidential Sync provides

Per WebFetch willowprotocol.org/specs/confidential-sync + NLnet Willow Sync grant (deadline 2026-06-01):

> Confidential Sync lets two peers determine which namespaces and Areas therein they share an interest in, **without leaking any data that only one of them wishes to synchronise**. Peers can discover common interests **without disclosing any non-shared information to each other** through private area overlap detection.

This is the "private interest overlap detection" property the iroh-willow assessment 2026-05-20 flagged as the strongest Benten alignment — it directly defends the §8-CC threat ("an adversary who knows plugin X can verify Alice signed for CID(X) without ever decrypting Alice's content") at a transport-protocol layer rather than at the app-layer encryption layer.

### 8.2 Composition with `TransportConfig`

The `TransportExtension::WillowConfidential { spec_version: u8 }` codepoint slot accommodates a future `MembershipSet` that selects Willow Confidential Sync as primary transport. The composition pattern:

- A `MembershipSet` with `transport_config.primary = Reserved(0x01)` + `transport_config.extension = Some(WillowConfidential { spec_version: 1 })` selects Willow.
- Engine resolves `Reserved(0x01)` + `WillowConfidential` via codepoint-dispatch to the Willow runtime (loaded as a benten-engine extension under CLAUDE.md #19 compile-time trust).
- Pairwise sync within the MembershipSet now rides Willow's WTP + Confidential Sync handshake instead of HandshakeFrame → MST diff → Loro merge.

### 8.3 Status reality check

- **Willow Confidential Sync demoted Final → Proposal 2025-10-26.** Spec in flux.
- **iroh-willow effectively parked.** Pinned to iroh 0.34 (incompatible with our 1.0.0-rc.0); 5/14 specs implemented; 0.0.1 published.
- **NLnet Willow Sync grant deadline 2026-06-01** — material spec changes likely between now and then.
- **Aljoscha Meyer's Codeberg fork (`willow_rs`)** appears to be the active development locus per RESEARCH-atrium-transport-model-2026-05-20.

**Verdict:** codepoint-reserve only at v1-beta. Watch-list for Phase-5+. The `TransportExtension::WillowConfidential` codepoint slot reserves the seam without committing to a particular spec version — when (and if) Willow stabilizes and a viable Rust impl matures, we add it as a transport option without re-freezing the wire shape.

---

## §9 iroh-roq / iroh-live / real-time-media composition

### 9.1 What they are (per WebFetch iroh-live + iroh-roq repos 2026-05)

- **iroh-roq**: RTP-over-QUIC per `draft-ietf-avtcore-rtp-over-quic`. By Friedel Ziegelmayer + n0 team; MIT/Apache. **Real-time media transport over iroh.** Not adopted by Benten.
- **iroh-live**: Media livestreaming over iroh. Real-time audio/video over QUIC. Full pipeline (camera capture → encode → transport → decode → render). Transport layer = **Media over QUIC (MoQ)** — each video rendition + audio track is an independent QUIC stream (dropped video packet doesn't block audio). Peer-to-peer by default; optional relay → WebTransport for browsers. **Actively maintained in 2026** (CLI tool + Android + Raspberry Pi demos).

### 9.2 Are these relevant for Benten v1-beta?

**NO.** Three reasons:

1. **Benten core thesis is NOT a media-streaming application.** The v1-beta scope is the engine + plugin framework + Atrium-sync + UCAN-cap + cryptography. Real-time media is an application a developer could build ON TOP OF Benten if they wanted, but it's not engine-core.
2. **Spike-D explicitly deferred until after S&C-informing spikes** per `.addl/spikes/README.md`. Has not even had a spike yet.
3. **Adding iroh-roq / iroh-live deps adds substantial cross-platform complexity** (codec selection; hardware acceleration; gstreamer-vs-software pipelines; etc) that crosses Benten's stated native-only-with-thin-client-browser-surface posture in non-trivial ways.

### 9.3 Where they could fit post-v1-beta

The `TransportExtension::IrohRoq { rtp_profile: u8 }` + `TransportExtension::IrohLive { moq_namespace: [u8; 32] }` codepoint slots reserve the seam. A future Phase-7+ Garden-or-presence feature (or a third-party plugin per CLAUDE.md #18 app-level subgraph plugins with three-layer consent) could opt-in either by setting `transport_config.extension`.

**Verdict:** codepoint-reserve only at v1-beta. Strictly post-v1-beta scope for any implementation. Treat as "future-extensibility commitment" rather than "v1-beta deliverable."

---

## §10 Final transport-configurability design

### 10.1 Concrete deliverables (v1-beta scope)

| # | Deliverable | LOC est | Wave-days | Phase |
|---|---|---|---|---|
| 1 | `crates/benten-engine/src/transport_config.rs` — `TransportConfig` + `TransportKind` + `SubInheritPolicy` + `TransportExtension` types | ~150 | 0.5 | Phase-4-Meta-Core |
| 2 | Canonical-bytes round-trip via DAG-CBOR (`to_canonical_bytes` / `from_canonical_bytes`) per CLAUDE.md #5 | ~100 | 0.5 | Phase-4-Meta-Core |
| 3 | `V1-FROZEN-INTERFACE.md` row (new section under §15) | ~30 lines doc | 0.25 | Phase-4-Meta-Core |
| 4 | `MembershipSet::transport_config` field integration (M2-dependent — coordinate with M2 spec) | ~50 | 0.5 | Phase-4-Meta-Core |
| 5 | Default-selection logic + builder API stub | ~80 | 0.5 | Phase-4-Meta-Core |
| 6 | `benten_errors::ErrorCode::TransportKindUnsupported` + `code()` mapping in benten-sync `AtriumTransportError` per §3.5g cross-language rule-mirror | ~30 | 0.25 | Phase-4-Meta-Core |
| 7 | Engine refusal-to-use-unsupported-kind path + test | ~80 | 0.25 | Phase-4-Meta-Core |
| 8 | RED-PHASE staged-pin `tests/atrium_gossip_announce_propagates_change_within_2s.rs` `#[ignore]` + named destination in `docs/future/phase-4-backlog.md §4.XX-NEW` per pim-12 §3.6e | ~50 | 0.25 | Phase-4-Meta-Core |
| 9 | `crates/benten-engine/INTERNALS.md` section explaining the TransportConfig seam + extension trust model per CLAUDE.md #19 | ~80 lines doc | 0.25 | Phase-4-Meta-Core |
| 10 | DAG-CBOR canonical-bytes proptest (10K cases per Phase-3 precedent) | ~50 | 0.25 | Phase-4-Meta-Core |
| | **Total v1-beta scope** | **~700 LOC + ~110 lines docs** | **~3.5 wave-days** | Phase-4-Meta-Core |

### 10.2 Concrete deliverables (Phase-4-Meta-Composing or Phase-5 — out of v1-beta-Core scope)

| # | Deliverable | LOC est | Wave-days | Phase |
|---|---|---|---|---|
| 11 | Add `iroh-gossip = 0.99` to benten-sync deps + ALPN registration | ~50 | 0.5 | Phase-5 or post |
| 12 | `crates/benten-sync/src/gossip.rs` — gossip publish wrapped in 4-KB-pre-check + `SignedMessage` envelope | ~250 | 1 | Phase-5 or post |
| 13 | `crates/benten-sync/src/gossip_topic.rs` — per-Atrium TopicId derivation with K_Atrium-blinding | ~120 | 0.5 | Phase-5 or post (needs Layer-A K_principal first) |
| 14 | Wire-up in `engine_sync.rs::apply_atrium_merge` to emit `ChangeAnnounce` on every accepted merge | ~80 | 0.5 | Phase-5 or post |
| 15 | Wire-up in `merge_remote_change` to consume `ChangeAnnounce` + trigger pull | ~80 | 0.5 | Phase-5 or post |
| 16 | Application-layer heartbeat ride-along on gossip topic for fast presence | ~100 | 0.5 | Phase-5 or post |
| 17 | TS DSL surface for builder API override | ~80 | 0.5 | Phase-4-Meta-Composing |
| 18 | Operator-observability disclosure: gossip-topic IS leaked to all members of the topic (parallel to `BootstrapMode::operator_observability_disclosure()` precedent) | ~50 | 0.25 | Phase-4-Meta-Composing |
| 19 | Tests: 3-peer happy path; offline-rejoin durability; 4-KB-oversize-rejected; topic-ACL-membership-validation; signed-payload-verify; cap-recheck per gossip-event | ~400 | 1.5 | Phase-5 or post |
| 20 | Codepoint-dispatch for `TransportExtension::WillowConfidential` / `IrohRoq` / `IrohLive` — placeholder runtime returns `E_TRANSPORT_EXTENSION_NOT_IMPLEMENTED` | ~80 | 0.25 | Phase-4-Meta-Composing or Phase-5 |
| | **Total post-v1-beta scope** | **~1240 LOC** | **~6.0 wave-days** | Spans Phase-4-Meta-Composing + Phase-5 |

### 10.3 Codepoint-dispatch layout (v1-beta-frozen)

```
TransportKind discriminator (uint):
  0 = None
  1 = BlobsPullMst
  2 = GossipPlusBlobs
  3 = GossipRequiredBlobsFallback
  256..=u16::MAX = Reserved(u16)  ; extension space

SubInheritPolicy discriminator (uint):
  0 = Inherit
  1 = Independent
  2 = InheritWithOverride       ; recommended default

TransportExtension tag (uint major + payload):
  100 = WillowConfidential { spec_version: u8 }
  101 = IrohRoq { rtp_profile: u8 }
  102 = IrohLive { moq_namespace: [u8;32] }
  200..= reserved for future extension
```

### 10.4 Integration with V1-FROZEN-INTERFACE.md

Add a new row under §15 (likely §15.6 — coordinate naming with V1-FROZEN-INTERFACE maintainer at integration time per pim-1 doc-coupling pre-flight):

> **§15.6 Transport-configurability (`TransportConfig`)** — per-MembershipSet transport selection. Wire shape: see canonical CBOR §10.3. Defaults: SingleDevice → `None`; DeviceMesh → `BlobsPullMst`; Atrium (n<200) → `BlobsPullMst`; Atrium (n≥200) → `GossipPlusBlobs`. Engine refuses to use unsupported codepoints at v1-beta with `E_TRANSPORT_KIND_UNSUPPORTED`. Codepoint-reserve only at v1-beta; production iroh-gossip / Willow Confidential Sync / iroh-roq / iroh-live impl deferred to Phase-4-Meta-Composing / Phase-5+ per `.addl/phase-4-meta/membership-set-m6-transport-configurability.md` §10.

### 10.5 Trust model (per CLAUDE.md #18 + #19)

- The `TransportConfig` type is APPLICATION-LAYER (user-of-engine picks per MembershipSet).
- The actual transport runtimes (iroh-gossip; Willow; iroh-roq; iroh-live) are ENGINE-LEVEL Rust extensions under CLAUDE.md #19 compile-time trust — deployment chooses which transports its built engine supports.
- The TWO LAYERS compose: user picks `transport_config.primary = GossipPlusBlobs` → engine looks up the GossipPlusBlobs runtime → engine returns `E_TRANSPORT_KIND_UNSUPPORTED` if the runtime is not compiled in.
- This naturally allows: deployment-A (server with stable connectivity) builds with all transports; deployment-B (mobile-only, battery-sensitive) builds with `BlobsPullMst` only; the codepoint dispatch handles the policy.

---

## §11 Self-assessment + confidence per finding

| Finding | Confidence | Evidence basis |
|---|---|---|
| Current stack does NOT include iroh-gossip; uses pairwise iroh+MST+Loro pull | HIGH | Read `crates/benten-sync/INTERNALS.md` + M1a §2.2 inventory + Spike C confirming "no production code consumes [iroh-gossip]" |
| Current sync is poll-driven / on-demand, not broadcast-driven | HIGH | Read `engine_sync.rs` (~2135 LOC) shape + `merge_remote_change` per-peer invocation pattern in M1a §2.3 |
| iroh-gossip enables Class B "live feel" sync (~100-500 ms wall-clock) | HIGH | Spike C 2026-05-21 measured 1-2 ms gossip RTT in 3-peer mesh + 4 ms pairwise pull is conservative |
| iroh-gossip does NOT enable Class C real-time co-edit alone | MEDIUM-HIGH | Spike C surfaced 4 KB ceiling, 30-s NeighborDown lag, no membership ACL — each constrains Class C |
| Per-MembershipSet matrix is correct shape | MEDIUM | Recommendation matrix grounded in Spike C empirical + Q3 specialist "few-thousand-member" claim + iroh-gossip upstream docs; specific row thresholds (e.g. n=200) are EXTRAPOLATION not empirical and may need adjustment after M2 / M5 input |
| Codepoint-reserve at v1-beta beats scope-include impl | HIGH | Elegant-shape pass §7.2 four reasons; reversible-up property |
| `TransportConfig` field layout proposal | MEDIUM | Concrete shape proposed; final names await M2 primitive |
| Willow Confidential Sync is Phase-5+ codepoint-reserve only | HIGH | Spec demoted Final→Proposal 2025-10-26; iroh-willow parked; NLnet grant deadline 2026-06-01 |
| iroh-roq / iroh-live are post-v1-beta deferral | HIGH | Spike-D explicit deferral; not on v1-beta thesis path |
| Wave-day cost ~3.5 v1-beta + ~6.0 post-v1-beta | MEDIUM | Estimated against Phase-3 wave-sizing precedents (G16-* waves) + Spike C "~330 LOC harness" data point |
| Default-selection logic is correctness-preserving | HIGH | Defaults all map to `BlobsPullMst` (current behavior) for size-unknown / unspecified cases |

**Known limitations of this assessment:**

1. **M2 primitive design not read** — final `MembershipSet::transport_config` field name + integration shape depends on M2's MembershipSet primitive. Coordinated at M2 dispatch.
2. **M3 / M4 / M5 sibling specialists not consulted** — they may surface concerns about UX surface (M4 likely) or scaling (M5 likely) that adjust the matrix thresholds.
3. **No empirical data above 3-peer mesh for iroh-gossip in Benten context** — Spike C ran 3-peer only. Upstream "few thousand peers" claim is from iroh-gossip's own README + LambdaClass blog; not independently verified by Benten.
4. **Battery-cost claims are qualitative** — Spike C did not instrument mobile battery; the "higher resting for gossip" claim is from gossip-protocol-paper-derived reasoning + relay-online cost observation. A small follow-up spike could quantify.
5. **The 4 KB ceiling is mitigable by per-swarm config but every peer must agree network-wide** — if we picked, say, 16 KB to accommodate larger Loro op log entries, a v1.x peer joining a v1-beta-default-4-KB swarm would lose messages. This is a network-effect-coupled decision; defer to Phase-5 with clearer adoption data.
6. **The `TopicId` K_Atrium-blinding derivation depends on Layer-A K_principal store landing** — ordering constraint within Phase-4-Meta-Core: K_principal must land BEFORE gossip TopicId can be computed correctly. Naming this dependency explicitly is necessary for impl-wave sequencing.

---

## §12 Citations

### Code (HEAD `2172cb6d`)

- `crates/benten-sync/INTERNALS.md` §§1-3 — current transport stack inventory.
- `crates/benten-sync/src/transport.rs` (~825 LOC) — iroh QUIC Endpoint + Connection + TransportKind + TransportStatus + ATRIUM_ALPN.
- `crates/benten-sync/src/transport_trait.rs` — RATIFIED §15.3 #1 abstraction boundary.
- `crates/benten-sync/src/peer_discovery.rs` — `BootstrapMode::operator_observability_disclosure()` precedent.
- `crates/benten-sync/src/handshake_wire.rs` (365 LOC) — `HandshakeFrame` wire-format with both-DIDs-required type-state builder per net-blocker-4.
- `crates/benten-sync/src/handshake.rs` (~1276 LOC) — 3-step DID-mutual-auth + `Session.synchronized_revocations` per net-r4-r1-3.
- `crates/benten-sync/src/crdt.rs` (~947 LOC) — Loro per-property HLC-LWW.
- `crates/benten-sync/src/mst.rs` (~743 LOC) + `mst_proto.rs` (~472 LOC) — MST diff exchange.
- `crates/benten-sync/src/ucan_blobs_protocol.rs` + `crates/benten-sync/src/two_cid_store.rs` — UCAN-gated iroh-blobs swap-point (G-CORE-3e wave-3e).
- `crates/benten-engine/src/engine_sync.rs` (~2135 LOC) — `AtriumHandle` API + `apply_atrium_merge` row-loop.

### Sibling specialists (this panel)

- **M1a** `phase-4-meta-core/membership-set-cataloger-m1a-multi-device-sync` @ `3618e051` — multi-device-sync inventory (§§2.1-2.5 read).
- **M1b** `phase-4-meta-core/membership-set-cataloger-m1b-atrium-membership-sharing` @ `1816ea60` — Atrium-membership inventory.
- **M1c** `phase-4-meta-core/membership-set-cataloger-m1c-key-management` @ `50eb901d` — key-management inventory.
- **M2/M3/M4/M5** — pending; final integration design awaits M2 primitive shape.

### Prior specialists referenced

- `q3-revisit-option-d-community-lens` @ `23f76e24` — K_Atrium-blinded plaintext_cid + few-thousand-member Atriums claim.
- `option-f-plus-lens-l9-atrium-integration` @ `1670aa03` — multi-stanza + dual-CID + transport implications.
- `option-f-plus-lens-l11-crdt-conflict-resolution` @ `68eadd0c` — CRDT / eventual-consistency / offline-first lens.

### Spikes

- `.addl/spikes/SPIKE-C-iroh-gossip-2026-05-21.md` — empirical iroh-gossip 3-peer broadcast / peer-crash / revocation-shape / failure-detection-timing / burst-stress / no-relay-LAN.
- `.addl/spikes/SPIKE-A-sendme-blobs-2026-05-21.md` (referenced, not directly read) — sendme + iroh-blobs hands-on.
- `.addl/spikes/SPIKE-A2-ucan-on-wire-2026-05-21.md` (referenced, not directly read) — UCAN-on-wire iroh-blobs.
- `.addl/spikes/README.md` — spike methodology.

### Session logs / research

- `.addl/phase-4-meta/SESSION-2026-05-20-to-2026-05-21-substrate-wave-and-willow-pivot.md` — 3 sendme deployment modes + Willow/iroh-willow assessment + S&C scope promotion + spike sequence authorization.
- `.addl/pq-research/RESEARCH-atrium-transport-model-2026-05-20.md` — full iroh family crate inventory (iroh / iroh-base / iroh-relay / iroh-dns / iroh-gossip / iroh-blobs / iroh-docs / iroh-willow / iroh-mainline-address-lookup / iroh-smol-kv / iroh-rings) + libp2p ecosystem alternative.
- `.addl/pq-research/RESEARCH-iroh-willow-assessment-2026-05-20.md` (referenced via session log) — iroh-willow code-level audit.

### Web sources (2026-05-27)

- [iroh-gossip GitHub repo](https://github.com/n0-computer/iroh-gossip) + [CHANGELOG](https://github.com/n0-computer/iroh-gossip/blob/main/CHANGELOG.md) — version 0.99 PlumTree + HyParView + 2000-node stress test note.
- [iroh-gossip docs.rs](https://docs.rs/iroh-gossip/latest/iroh_gossip/) + [iroh.computer protocol page](https://www.iroh.computer/docs/protocols/gossip) — official API surface + scaling claim.
- [iroh-blobs docs.rs](https://docs.rs/iroh-blobs/latest/iroh_blobs/) + [iroh.computer Blobs page](https://docs.iroh.computer/protocols/blobs) — BLAKE3 streaming + content-addressing semantics.
- [iroh-docs GitHub](https://github.com/n0-computer/iroh-docs) — meta-protocol on blobs+gossip; namespace+replica model.
- [iroh-roq GitHub](https://github.com/n0-computer/iroh-roq) + [crates.io entry](https://crates.io/crates/iroh-roq) — RTP-over-QUIC per draft-ietf-avtcore-rtp-over-quic.
- [iroh-live GitHub](https://github.com/n0-computer/iroh-live) + [iroh streaming protocols](https://docs.iroh.computer/protocols/streaming) — media livestreaming over MoQ.
- [Willow Confidential Sync spec](https://willowprotocol.org/specs/confidential-sync/index.html) — private interest overlap detection.
- [NLnet Willow Sync grant](https://nlnet.nl/project/WillowSync/) — Rust impl status + 2026-06-01 deadline.
- [LambdaClass The Wisdom of Iroh](https://blog.lambdaclass.com/the-wisdom-of-iroh/) — overview of iroh-gossip 2000-node stress test claim.

### Disciplines applied

- HARD RULE 12 (no "later" disposition): all gossip-impl rows have NAMED destination (Phase-4-Meta-Composing / Phase-5; backlog row `§4.XX-NEW`); all OOS items have explicit reason.
- pim-12 §3.6e (RED-PHASE staged-pin discipline): `tests/atrium_gossip_announce_propagates_change_within_2s.rs` proposed as `#[ignore]`-named pin.
- pim-1 §3.5b (post-fix doc-coupling pre-flight): V1-FROZEN-INTERFACE.md row coordinated at integration time.
- §3.5g cross-language rule-mirror: `benten_errors::ErrorCode::TransportKindUnsupported` + TS mirror named in deliverable #6.
- Extra-reflection-pass elegant-shape: §7.2 four reasons for Option A (codepoint-reserve) constitute the elegant single-shape closing all scope-fork findings.
- ADDL-pipeline-full-observance: §7.1 Option B risk note explicitly references the rule.
- M2-dependent items flagged explicitly (final field name) for coordination.
