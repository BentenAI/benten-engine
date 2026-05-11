# benten-sync — INTERNALS

Read-only crate-internals deep-dive. Plain-English. Aimed at a fresh agent who needs to understand the sync layer's shape before touching it.

---

## 1. What this crate does

`benten-sync` is the 10th workspace crate (added in Phase 3) and is the runtime for the **Atrium** — Benten's peer-mesh P2P sync surface. It owns the wires between Benten peers: how two (or more) running engines find each other, authenticate as DIDs, exchange Loro CRDT op-logs at Node-property granularity, reconcile divergent subgraphs via a Merkle Search Tree (MST) diff protocol, and let thin clients (Phase 9+ exploratory) verify single-Node inclusion without pulling the whole subgraph. It is the home of the iroh QUIC transport, the Loro integration, the MST primitive, the DID-based mutual-auth handshake, and the wire-format envelopes that ride over all of it.

It is **native-only** per CLAUDE.md baked-in #17. The full peer (laptop, phone OS app, desktop, future Benten Runtime instance) lives here; the browser tab participates by talking the D-PHASE-3-30 thin-client protocol against a full peer over HTTPS+SSE/WS, NOT by running iroh in wasm. The native-only commitment is defended by three rungs that all live in this crate (see §3 below). It implements the **member-mesh networking** posture (CLAUDE.md #9 — every Atrium needs ≥1 peer online; there is no central server) and explicitly avoids adding a 13th operation primitive (CLAUDE.md #1) — it is a transport layer consumed by the existing READ / WRITE / EMIT / SUBSCRIBE primitive arms, not a new primitive.

---

## 2. Dependency chain

**Workspace deps in (this crate consumes these):**

- `benten-id` — `Keypair` / `PublicKey` / `Signature` / `Did` / `Ucan` / `Capability` / `validate_chain_no_time_check`. The handshake signs frames with the local keypair, encodes DIDs in the wire format, validates remote UCAN chains at handshake time, and derives the `PeerId` directly from the Ed25519 public-key bytes (per crypto-minor-4).
- `benten-core` — only `hlc::BentenHlc`. Loro per-property writes carry a `BentenHlc` stamp so LWW is resolved by Benten's HLC rather than Loro's internal Lamport (load-bearing for cross-process WAIT-resume + Inv-14 device-grain attribution + revocation-vs-data ordering).
- `benten-errors` — the stable `ErrorCode` catalog. Every typed error in this crate maps to a `benten_errors::ErrorCode` variant (`AtriumRelayUnreachable`, `AtriumTransportDegraded`, `HandshakeReplayWithinBoundedWindow`).
- `serde` / `serde_ipld_dagcbor` / `serde_bytes` — DAG-CBOR canonical-bytes encoding for the handshake frame, MST diff frames, handshake payloads, and signed-bytes computation. Symmetric with CLAUDE.md #5 (BLAKE3 + DAG-CBOR + CIDv1).
- `thiserror` / `tracing` — typed errors + structured logging.
- `blake3` — direct dep for MST node hashes (the `MstCid` newtype wraps the 32-byte BLAKE3 digest). The crate notes it deliberately does NOT depend on `benten-core::Cid` even though both are BLAKE3 — the engine bridges `benten-core::Cid` ↔ `MstCid` at its consume boundary to preserve the layered direction.

**Native-only deps (cfg-gated behind `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`):**

- `iroh` 0.98 with `tls-ring` (QUIC transport, holepunch, relay-default).
- `loro` 1.12 (CRDT framework).
- `tokio` 1 with selected features (rt, rt-multi-thread, macros, sync, io-util, time, net).
- `bytes` 1 (iroh's Connection API consumes this).

**Workspace deps explicitly forbidden (asserted by `tests/dependency_edges.rs`):**

- NOT `benten-engine`.
- NOT `benten-eval`.

The dependency direction is **engine → sync** per arch-r1-11 + D-PHASE-3-14. The engine consumes the Atrium surface from sync (via `engine.atrium.*` API in `benten-engine`); never the reverse.

**Consumers out:**

- `benten-engine` — `engine_sync.rs` / `atrium_api.rs` consume `LoroDoc`, `Mst`, `Handshake`, `Session`, `Endpoint`, etc. The engine wraps these into the user-facing `Engine::create_atrium` / `Engine::accept_atrium_invite` / `Engine::subscribe_change_events` etc.
- Phase-3 napi-rs bindings + TS DSL — only reach `benten-sync` transitively through `benten-engine` (since the napi layer is consumed from both native + thin-client builds).

**Dev-deps:** `proptest` (10 000-case Loro + MST convergence suites) + `toml` (used by `dependency_edges.rs` to programmatically walk Cargo.toml dep tables rather than grep prose).

---

## 3. Files inventory in `src/`

The crate is organised by surface-of-the-Atrium rather than by abstraction level. Eight `pub mod`s + `lib.rs`.

### `lib.rs` (~205 lines)

The crate root. Three load-bearing things happen here:

1. **`compile_error!` gate.** A `#[cfg(target_arch = "wasm32")] compile_error!(...)` macro that fires immediately if a downstream consumer attempts to build benten-sync for wasm32. The error message names CLAUDE.md baked-in #17 + points the consumer at the thin-client surfaces (`benten-engine`'s `thin_client_subscribe.rs`, `bindings/napi/src/wasm_browser.rs`, `packages/engine/src/atrium.ts`).
2. **Cfg-gated module declarations.** Every `pub mod` is wrapped in `#[cfg(not(target_arch = "wasm32"))]` so the modules and their iroh/tokio dependencies share a single architectural seam with the Cargo.toml dep-table cfg.
3. **Module-level docs.** Long `//!` headers explain the native-only commitment, the G16-A canary vs G16-B/C/D wave-6b split (now all landed), the architectural commitments back to CLAUDE.md baked-ins #1, #3, #5, #11, #17, and the layered dep-architecture pin (arch-r1-11 / D-PHASE-3-14).

This is the **3-rung baked-in #17 defense** mentioned in CLAUDE.md: rung 1 = `compile_error!` macro at `lib.rs`; rung 2 = cfg-gated dependency tables in `Cargo.toml` so iroh/tokio/loro aren't even resolvable on wasm32; rung 3 = at-build-time CI cell `benten-sync-refuses-wasm32` in `.github/workflows/wasm-checks.yml` that runs `cargo check --target wasm32-unknown-unknown -p benten-sync` and verifies the `compile_error!` fires.

### Transport module — `transport.rs` (~825 lines)

The iroh QUIC transport core. The most load-bearing module for the "two real peers actually talk to each other" exit criterion.

Key types:

- **`Endpoint`** — wraps `iroh::Endpoint`. One per peer-process. Holds the local `PeerId`, the underlying iroh handle, and an `Arc<Mutex<TransportStatus>>` that the foreground connect/accept calls + background relay-fallback task share. Construction paths: `bind_loopback` (single-process canary, `Minimal` preset, no relay), `bind_loopback_with_keypair` (deterministic keypair variant for tests), `bind_with_keypair` (production Atrium peer), `bind_with_relay_url` (currently still in canary-scope returning `RelayUnreachable` for the canary-scope arm — the real binding has moved to `peer_discovery::bind_atrium_peer` per pim-4 §3.10 wave-paired closure), and `from_iroh_parts` (the wave-6b constructor that wraps an iroh::Endpoint built directly by `peer_discovery::bind_atrium_peer`).
- **`Connection`** — wraps `iroh::endpoint::Connection`. Carries the path-discriminator (Direct / Relay / Loopback) captured at connection-establishment time + the remote `PeerId`. Surfaces `send_bytes` (open uni-stream, write_all, finish, await `stopped()` for ACK) and `recv_bytes` (accept uni-stream, read_to_end with 4 MiB cap as defense against unbounded-allocation attacks; G16-B/D layers framed length-prefix encoding on top).
- **`TransportKind`** — Direct / Relay / Loopback discriminator.
- **`TransportStatus`** — Healthy { kind } / Degraded { reason } / NotConnected. Exposes `error_code()` mapping degraded → `ErrorCode::AtriumTransportDegraded` per net-blocker-2.
- **`ATRIUM_ALPN`** — `b"benten/atrium/1"`. iroh's accept-loop filters by this ALPN; future protocol revisions negotiate via `benten/atrium/2`.

Test fixtures inside the module: `mark_degraded` (manual flip) + `simulate_packet_loss` (>10% → degraded).

### CRDT module — `crdt.rs` (~947 lines)

Loro integration at **Node-property granularity** per D-PHASE-3-4. The whole convergence semantics live here.

Key shapes:

- **`LoroDoc`** — wraps `Arc<loro::LoroDoc>`. One per logical property bag (typically one per graph Node). Cloneable via Loro's `fork`. Default constructor + `with_peer_id(u64)` for deterministic test op-logs.
- **`StampedValue { value: String, hlc: HlcWire }`** — the wire shape for an HLC-stamped scalar property. The crate's load-bearing trick: Loro's internal Map LWW uses Loro's internal Lamport clock, which would defeat Benten's HLC-everywhere posture. The fix is to **carry the HLC inside the value** as a packed string `"<physical>:<logical>:<node>:<value>"` and never trust Loro's internal Lamport for ordering decisions.
- **`HlcWire { physical_ms, logical, node_id }`** — owned-primitive mirror of `BentenHlc` suitable for Loro's `LoroValue` encoding. `From` impls both directions. Lex comparison `cmp_lex` matches `BentenHlc::Ord`.
- **`OpLogTarget { container_name }`** — surface for the engine's Inv-13 row-4 SPLIT classifier (ds-4). Walks the doc's container roots so the engine can decide row-4a (user-data, apply via D-C version-chain pattern) vs row-4b (system-zone / Anchor-immutable, reject with `E_SYNC_DIVERGENT_CID_REJECTED`).

Storage shape, very specifically: instead of using a Loro Map keyed by property name (which has documented "concurrent container creation at the same key may overwrite" semantics on Loro 1.12), every property write is appended to a **single root List** at container-id `benten:properties`. Each entry is a flat string `"<key>\x1f<physical>:<logical>:<node>:<value>"` (Unit Separator `\x1f` between key and packed-stamped-value). Reads scan the List, group by key, and resolve LWW per HLC. Loro List's CRDT semantics preserve concurrent appends. There is a deliberately-deferred R6-FP optimisation: "compact-old-writes-on-checkpoint" to truncate stale entries once all peers observe the merge.

Rich types are exposed under `benten:rich:<name>`: `LoroDoc::list(name)` and `LoroDoc::map(name)` for collaborative subgraph edits where the LWW-per-HLC discipline doesn't apply (comments lists, tag maps).

Sync I/O surface: `merge(&LoroDoc)` (bidirectional merge for two-peer convergence), `apply_remote_update(bytes)` / `export_update()` (byte-level delta interface for the Atrium sync-replica delivery seam), `to_canonical_bytes()` / `from_canonical_bytes(bytes)` (full snapshot round-trip via Loro's `ExportMode::Snapshot`).

Attribution: `winning_attribution()` returns the union of every observed write's HLC `node_id` (not just the LWW winner) so revoked peers' contributing writes are also surfaced — load-bearing for the revocation-vs-data ordering audit per net-blocker-3. `all_writes()` surfaces `(property_key, StampedValue)` for the engine-side `AttributionFrame` mint per arch-r1-4 D-C HYBRID.

### Errors module — `errors.rs` (~197 lines)

The transport-layer typed-error surface per net-blocker-2 BLOCKER.

`AtriumTransportError` (#[non_exhaustive]) carries four variants — `RelayUnreachable`, `TransportDegraded`, `PeerConnectFailed { peer, reason, relay_side }`, `HandshakeWireFormat`. The `code()` accessor maps each variant to a stable `benten_errors::ErrorCode`:

- `RelayUnreachable` → `AtriumRelayUnreachable`
- `TransportDegraded` + `HandshakeWireFormat` → `AtriumTransportDegraded`
- `PeerConnectFailed { relay_side: true }` → `AtriumRelayUnreachable`
- `PeerConnectFailed { relay_side: false }` → `AtriumTransportDegraded`

The contract: every transport-layer failure surfaces as a typed variant carrying a stable code — NEVER as a panic, NEVER as an untyped `String` error.

### Handshake wire-format — `handshake_wire.rs` (~365 lines)

The **wire-format envelope** for the Atrium peer handshake. G16-A canary scope; SHAPE only.

`HandshakeFrame { version, peer_did, device_did, peer_id, protocol_payload: Vec<u8> }`. Per net-blocker-4 BLOCKER both `peer_did` (account identity) AND `device_did` (device identity under the account) are REQUIRED at the wire-format layer (not Optional). Multi-device support (Inv-14 device-grain attribution) depends on the device-DID being observable end-to-end through every handshake frame.

The both-required constraint is enforced at compile time by a **type-state builder**: phantom-state marker structs (`NoPeerDid` / `WithPeerDid` / `NoDeviceDid` / `WithDeviceDid` / `NoPeerId` / `WithPeerId`) walk the builder through completion states; only the `(WithPeerDid, WithDeviceDid, WithPeerId)` state has a `build()` method. A caller attempting to build with either DID missing gets a compile error, not a runtime error.

Canonical-bytes round-trip via DAG-CBOR (`to_canonical_bytes` / `from_canonical_bytes`) per CLAUDE.md #5. Malformed bytes return `AtriumTransportError::HandshakeWireFormat`.

### Handshake protocol — `handshake.rs` (~1 136 lines)

The **DID-based mutual-auth handshake state machine** that rides on `HandshakeFrame`. G16-D wave-6b body.

Three-step protocol:

1. **`Handshake::initiate(local_kp, audience_did, grant, revocation_set)`** — peer-A constructs an outbound frame addressed to peer-B's DID. The frame's `protocol_payload` carries CBOR-encoded `HandshakePayload::Initiate { audience_did, nonce: [u8;32], hlc_physical_ms, grant: Option<Ucan>, revocation_set: Vec<RevocationEntry>, signature: Vec<u8> }`. The signature is over canonical-bytes of the rest of the payload (computed by `initiate_signing_bytes`).
2. **`Handshake::respond(local_kp, initiate_frame, grant, revocation_set)`** — peer-B verifies four things: (a) audience-DID matches local DID (else `AudienceMismatch`); (b) signature against the initiator's declared pubkey (else `InvalidSignature`); (c) HLC drift is within the bounded replay window (else `ReplayWithinBoundedWindow { original_hlc, replay_hlc, window_ms }` — DEFAULT_REPLAY_WINDOW_MS = 5 000); (d) UCAN chain validates (`validate_chain_no_time_check` — wallclock-skew already bounded by the replay-window check; nbf/exp deferred to G14-D delivery-time recheck). On success returns `(response_frame, Session)`.
3. **`Handshake::finalise(local_kp, outbound_initiate_nonce, grant, revocation_set, response_frame)`** — peer-A verifies the response: nonce-echo matches the initiator's nonce (else `InvalidSignature` "cross-session injection"), signature verifies, UCAN chain validates. Returns peer-A's mirror `Session`.

`MessageKind` enum: `Revocation = 0` strictly < `Data = 1` per net-blocker-3. Discriminants are load-bearing so a `Vec::sort` / `BTreeMap` ordered by `MessageKind` drains revocations first. The same enum is defined here AND at `mst_proto.rs`; the file header notes both definitions must agree on discriminants — the file-header even calls out the future reconciliation where this local definition is deleted once a re-export from `mst_proto::MessageKind` is wired (and both definitions currently sit at `Revocation = 0` / `Data = 1` so the merged enum will be a serialised no-op).

`Session` shape: mutually-authenticated DIDs (`local_did` + `remote_did`), the two exchanged UCAN grants (`local_grant_to_remote` + `remote_grant_to_local`), and a synchronized revocation snapshot (`synchronized_revocations: Vec<RevocationEntry>`). The `synchronized_revocations` field is the load-bearing **handshake-time revocation gate** per net-r4-r1-3: the responder UNIONS the initiator's advertised revocations with its own local revocation set BEFORE returning a Session, and the resulting snapshot is consulted by `subscription_open_permitted()` so the post-handshake SUBSCRIBE gate cannot open until the cache has been seeded — closing the TOCTOU window between handshake-completion and revocation-set-snapshot.

`EffectiveCapSet { caps: Vec<Capability> }` is derived from the remote-to-local UCAN grant. Exposes `includes_cap(resource, ability)` for callers checking specific delegations. `intersection_validates_against_ucan_chain()` always returns `true` because reaching the Session means the chain validated at construction time — per g16-d-mr-3 fix-pass.

Random nonce derivation is deliberately quirky: rather than pull in `rand` as a direct dep, the module derives 32 bytes of unpredictability by generating a fresh `Keypair`, taking the public-key bytes (already-vetted entropy source in the workspace), and discarding the keypair. Header comments acknowledge the choice.

### Light-client — `light_client.rs` (~418 lines)

Per-ROADMAP-2 distinct deliverable. The thin-client verifier per ds-r4r2-3 **mode-(a) "single-CID inclusion proof" only**. Modes (b) range-query and (c) signed-checkpoint are explicitly OOS for Phase 3 (architectural-absence pins at `tests/light_client_distinct.rs`).

`LightClient { budget: BandwidthBudget, bytes_consumed: AtomicUsize }`. Stateless verifier (no mutation outside the byte counter). `verify(published_root, path, proof)` does four checks:

1. Proof size ≤ budget (else `BandwidthBudgetExceeded`). Default budget 64 KiB.
2. `proof.target_key == path` (else `ProofKeyMismatch` — defends proof-substitution where a peer returns a valid proof for a DIFFERENT key than the caller asked).
3. `proof.target_cid` appears in the sorted `(key, cid)` pairs at the declared key (else `Mst(MstError::ProofKeyAbsent)`).
4. `proof.reconstruct_root() == published_root` (else `Mst(MstError::ProofRootMismatch { expected, got })` — load-bearing tampering check).

`bytes_consumed()` accessor surfaces cumulative bytes across all verifications — used by the `bandwidth_stays_below_full_subgraph_per_roadmap_2` assertion (proves the thin client really does verify within budget rather than implicitly fetching the full subgraph).

### MST — `mst.rs` (~743 lines)

Merkle Search Tree for subgraph sync. Phase 3 G16-C wave-6b.

`MstCid` newtype wraps `[u8; 32]` raw BLAKE3 digest. Custom `serde_bytes_32` module encodes as DAG-CBOR byte-string (length-tagged); `Display` is hex. `from_bytes(bytes)` computes BLAKE3 over arbitrary bytes; `from_blake3_digest(digest)` wraps an already-computed digest. The crate notes this mirrors `benten-core::Cid` shape but is intentionally a separate type to avoid the forbidden dep on benten-core (the engine bridges them at consume-time).

`MstEntry { key: String, cid: MstCid, payload: Vec<u8> }` — `key` is a logical zone path; `cid` is declared (potentially-adversarial); `payload` is canonical bytes. `from_payload(key, payload)` is the legitimate construction path (computes CID locally). `new_with_explicit_cid_for_testing(declared, payload)` is the adversarial-construction path consumed by `attack_mst_diff_cid_mismatch.rs`.

`Mst { entries: BTreeMap<String, MstEntry> }` — a deterministic sorted map. The B-Tree shape is the structural simplification that makes the implementation tractable while preserving the load-bearing properties: deterministic root, O(log n) tree depth, content-addressing.

Root computation (`root_cid()`): canonicalises to a sorted `Vec<(String, MstCid)>` of just the key+CID pairs (NOT payload bytes — each entry's CID already commits to its payload via blake3), DAG-CBOR encodes, BLAKE3 hashes. Two MSTs with the same `(key, cid)` pairs produce the same root regardless of insertion order.

**Application-layer rehash check** at `Mst::apply_entries` — for every entry, re-hashes `payload` locally and compares byte-for-byte against the declared `cid`. On mismatch: rejects with `MstError::EntryCidByteMismatch` WITHOUT applying. This is the production-runtime path the engine's `consume_sync_replica_mst_diff` (G16-B) will plumb after handshake-attesting frame integrity. Defends against sec-r4r2-1 attack: an adversarial peer crafts a frame whose declared CID doesn't match its payload bytes.

**Merkle proof construction** at `Mst::merkle_proof_for(key) -> Option<MerkleProof>`. Returns `None` if the key is absent. The `MerkleProof { target_key, target_cid, sorted_pairs }` carries the (key, cid) of the target plus the (key, cid) pairs of every OTHER entry — sufficient for a verifier holding only the published root to reconstruct the canonical-bytes shape + recompute the root. Crate docs explicitly note this is O(n) in entry count (not the optimal O(log n) tree-shaped path); a Phase-4 optimisation replaces it. Even the O(n) shape proves the thin-client bandwidth saving because payload bytes are NOT included in the proof (saving payload-size × n bytes).

**Diff protocol**: `MstDiff::between(a, b) -> MstDiff { missing_in_a, missing_in_b }` walks the two BTreeMaps in parallel via merge. Same-key-different-CID is surfaced to BOTH sides (engine layer breaks the tie via HLC LWW at G16-B). `run_mst_diff_to_convergence(a, b)` is the round-driven convergence driver — capped at 64 rounds, currently converges in one round for the BTreeMap-flat API surface. The round-driven shape is preserved so it scales when G16-B's engine layer wraps the MST in a partial-sync cursor that exposes one tree level per round.

### MST proto — `mst_proto.rs` (~472 lines)

The wire-protocol shape for MST diff exchanges. Per net-blocker-3 BLOCKER.

`MessageKind` enum (Revocation = 0 / Data = 1) — same shape as the local definition in `handshake.rs`; both files acknowledge they will reconcile to a single source of truth.

`MstDiffMessage { kind, cid, payload }` — single message. `MstDiffMessage::data(cid, payload)` and `MstDiffMessage::revocation(cid, reason)` are kind-discriminated constructors. Canonical-bytes round-trip via DAG-CBOR.

`MstDiffFrame { version, round, messages }` — a wire frame carrying multiple messages within a round. `MST_DIFF_WIRE_VERSION = 1`. The receiver rejects mismatched versions at the wire layer.

`MstDiffSession` — the runtime drainer enforcing the **revocation-drains-before-data** invariant. Two-tier queue: `revocation_queue: Vec<MstDiffMessage>` + `data_queue: Vec<MstDiffMessage>`. The kind is consulted at enqueue time so drain is O(1) per message. `drain()` returns `revocation_queue + data_queue` (FIFO within tier, revocations as a tier-block first). The invariant holds across multiple frames + multiple rounds because the queues accumulate across `enqueue` / `enqueue_frame` calls until `drain` is requested — a tier-stratified buffer.

### Peer-id — `peer_id.rs` (~198 lines)

`PeerId { bytes: [u8; 32] }` — `#[serde(transparent)]` so DAG-CBOR encodes as a raw 32-byte byte-string. Per net-minor-2 + ds-8 + crypto-minor-4, the peer-id IS the Ed25519 public-key bytes — no hashing, no salt, no process-local randomness. Two processes given the same `PublicKey` produce byte-identical `PeerId`s.

Crucially: the same 32 bytes are reused as the **iroh `EndpointId`** (pre-iroh-0.98 `NodeId`). Module-level docs acknowledge the trust-coupling tradeoff: a single Ed25519 key signs both UCAN chains (engine-layer auth) AND QUIC TLS handshakes (transport-layer auth); compromising either layer compromises both. The alternative (separate keys per layer) was rejected at G16-A; the docs flag Phase 9+ may re-open this for a hardened-deployment posture.

Accessors: `from_public_key(pk)` / `from_bytes([u8;32])` / `as_bytes()` / `to_bytes()` / `to_canonical_bytes()` (raw bytes) / `to_dag_cbor_bytes()` (envelope round-trip) / `from_dag_cbor_bytes(bytes)` (typed `PeerIdDecodeError`).

### Peer discovery — `peer_discovery.rs` (~364 lines)

Atrium peer-discovery bootstrap. Phase-3 G16-D wave-6b. Promotes G16-A's `bind_with_relay_url` canary-scope placeholder to a real `RelayMode::Custom` binding (per pim-4 §3.10 wave-paired closure).

`BootstrapMode` enum: `DefaultRelay` (production default per D-PHASE-3-3 — public iroh relay infrastructure; metadata-leakage posture per Compromise #22) | `CustomPeerList(Vec<RelayUrl>)` (operator-controlled relays; Phase-7 Garden-relays are the canonical alternative) | `Disabled` (no relay; loopback / LAN only). Empty `CustomPeerList` falls through to `RelayMode::Disabled` rather than instantiate an empty `RelayMap` which iroh treats as misconfiguration.

`operator_observability_disclosure()` returns a static `&'static str` per mode — operator-readable trust-boundary disclosure per net-major-1 + sec-r1-12. The DefaultRelay disclosure references "Compromise #22" by name; the Disabled disclosure states "No relay-side metadata observability." Snapshot-asserted by `operator_observability_disclosure_is_stable_text` so UI consumers can rely on the exact text-prefix.

`PeerDiscoveryConfig { bootstrap }` + `bind_atrium_peer(keypair, config) -> Endpoint` — the production-binding entry point. Distinct presets per mode: `Disabled` uses `presets::Minimal` (no relay, no address-lookup), `DefaultRelay` + `CustomPeerList` ride on `presets::N0` (n0-DNS pkarr address-lookup) with the relay_mode override.

---

## 4. Public API surface

Re-exported via `pub mod` from `lib.rs`. Consumed by `benten-engine`'s `engine_sync.rs` + `atrium_api.rs` and (transitively) by the napi bindings.

**Transport surface:**

- `transport::Endpoint::bind_loopback() / bind_loopback_with_keypair(kp) / bind_with_keypair(kp)` — three bind variants.
- `transport::Endpoint::connect(remote_peer_id) / connect_to_addr(remote_addr)` — outbound.
- `transport::Endpoint::accept_next()` — inbound.
- `transport::Endpoint::transport_status() / mark_degraded(reason) / simulate_packet_loss(fraction)` — observability + test fixtures.
- `transport::Connection::send_bytes(payload) / recv_bytes() / transport_kind() / remote_peer() / close()`.
- `transport::TransportStatus` / `transport::TransportKind` / `transport::ATRIUM_ALPN`.

**Peer-discovery surface (wave-6b production entrypoint):**

- `peer_discovery::BootstrapMode` (DefaultRelay / CustomPeerList / Disabled).
- `peer_discovery::PeerDiscoveryConfig`.
- `peer_discovery::bind_atrium_peer(kp, config) -> Endpoint`.

**Handshake surface (G16-D body):**

- `handshake::Handshake::initiate(...)` / `respond(...)` / `respond_with_window(..., window_ms)` / `finalise(...)`.
- `handshake::initiate_nonce(frame)` — convenience nonce extraction for the initiator.
- `handshake::Session::local_did() / remote_did() / is_authenticated() / local_grant_to_remote() / remote_grant_to_local() / effective_cap_set() / revocation_set_synchronized() / synchronized_revocations_for_local_peer() / subscription_open_permitted()`.
- `handshake::EffectiveCapSet::caps() / includes_cap(resource, ability) / intersection_validates_against_ucan_chain()`.
- `handshake::HandshakePayload` (Initiate / Respond), `handshake::RevocationEntry`, `handshake::MessageKind`.
- `handshake::HandshakeError` + variants + `code()` → ErrorCode.
- `handshake::DEFAULT_REPLAY_WINDOW_MS`.

**Handshake wire-format surface:**

- `handshake_wire::HandshakeFrame` + type-state `HandshakeFrameBuilder`.
- `handshake_wire::HANDSHAKE_WIRE_VERSION`.
- `to_canonical_bytes()` / `from_canonical_bytes(bytes)` round-trip.

**CRDT surface:**

- `crdt::LoroDoc::new() / with_peer_id(u64) / set_property(key, value, hlc) / get_property(key) / get_stamped(key) / merge(other) / apply_remote_update(bytes) / export_update() / to_canonical_bytes() / from_canonical_bytes(bytes) / list(name) / map(name) / winning_attribution() / all_writes() / op_log_targets() / op_count()`.
- `crdt::StampedValue` / `crdt::HlcWire` / `crdt::OpLogTarget` / `crdt::hlc_lww_winner(a, b)` / `crdt::CrdtError`.

**MST surface:**

- `mst::Mst::new() / insert(entry) / len() / is_empty() / root_cid() / apply_entries(iter) / merkle_proof_for(key)`.
- `mst::MstEntry::from_payload(key, payload) / new_with_explicit_cid_for_testing(...)`.
- `mst::MstCid::from_bytes(bytes) / from_blake3_digest(digest) / to_hex()`.
- `mst::MerkleProof::reconstruct_root() / with_tampered_node() / approximate_bytes()`.
- `mst::MstDiff::between(a, b)` + `mst::run_mst_diff_to_convergence(a, b)` -> rounds.
- `mst::MstError` + variants.

**MST proto surface:**

- `mst_proto::MessageKind` + `from_u8(b)`.
- `mst_proto::MstDiffMessage::data(cid, payload) / revocation(cid, reason) / to_canonical_bytes() / from_canonical_bytes(bytes)`.
- `mst_proto::MstDiffFrame::new(round) / push(msg) / to_canonical_bytes() / from_canonical_bytes(bytes)`.
- `mst_proto::MstDiffSession::new() / enqueue(msg) / enqueue_frame(frame) / drain() / pending_revocations() / pending_data()`.
- `mst_proto::MST_DIFF_WIRE_VERSION`.

**Light-client surface (mode-(a) only):**

- `light_client::LightClient::new() / with_budget(budget) / verify(root, path, proof) / bytes_consumed()`.
- `light_client::BandwidthBudget::default() / limit_bytes(n)`.
- `light_client::VerificationResult` + `light_client::LightClientError`.

**Errors surface:**

- `errors::AtriumTransportError` + 4 variants + `code()`.
- `errors::AtriumTransportResult<T>`.

**Peer-id surface:**

- `peer_id::PeerId::from_public_key(pk) / from_bytes([u8;32]) / as_bytes() / to_bytes() / to_canonical_bytes() / to_dag_cbor_bytes() / from_dag_cbor_bytes(bytes)`.
- `peer_id::PeerIdDecodeError`.

---

## 5. Tests inventory

35 test files. Roughly grouped:

### G16-A canary surface pins (transport / wire-format / peer-id / arch)

- **`transport_loopback.rs`** (4 tests; `#[ignore]` on some) — two-Endpoint loopback round-trip via `bind_loopback` + `loopback_addr` + `connect_to_addr`; relay-fallback (`#[ignore]`'d at G16-A landing pending iroh test-fixture stabilisation); holepunch smoke (`#[ignore]`'d, CI-conditional per scope-real-10).
- **`atrium_errors.rs`** (3 tests) — typed-error contract per net-blocker-2/4. Header carries a DISAGREE-WITH-EXPLANATION on the brief's `engine.atrium_status()` pin (that engine surface doesn't exist at G16-A canary scope; engine-side pin moved to G16-B).
- **`peer_id.rs`** (1 test) — cross-process determinism of `PeerId` derivation from Ed25519 pubkey.
- **`dependency_edges.rs`** (1 test) — walks `Cargo.toml`'s dep tables programmatically (NOT raw grep, so prose comments containing forbidden names don't false-positive); asserts no `benten-engine` / `benten-eval` dep.
- **`wasm32_excluded.rs`** (2 tests) — architectural pin reading the `lib.rs` `compile_error!` macro presence + the `Cargo.toml` cfg-gated dep tables. Companion CI cell `benten-sync-refuses-wasm32` at `.github/workflows/wasm-checks.yml` runs the real `cargo check --target wasm32-unknown-unknown`.
- **`graph_encoded_state.rs`** (4 tests; some `#[ignore]`'d) — G16-A canary floor: no persistent state in benten-sync per cag-2 + cag-r4-3. Defers persistent-state shape enforcement to G16-B/C/D.

### G16-B Loro wave (CRDT)

- **`loro_lww.rs`** (2 tests) — per-property LWW round-trip + concurrent writes converge via HLC ordering.
- **`loro_rich_type.rs`** (1 test) — rich-type (LoroList) collaborative concurrent insert preservation.
- **`hlc_loro_property_lww.rs`** (1 test) — HLC explicitly carries into Loro per-property LWW (would FAIL if Loro's internal Lamport were trusted).
- **`prop_loro_converge.rs`** (1 proptest, 10 000 cases) — N writers / arbitrary interleaving → all writers converge to same LWW value = highest-HLC write.

### G16-C MST wave

- **`mst_diff.rs`** (3 tests) — two-peer convergence in O(log n) rounds; canonical fixture corpus depth 4 / branch 8 per net-major-2.
- **`mst_revocation_priority.rs`** (1 test) — MST diff drainer returns Revocation before Data under interleaved arrival.
- **`light_client.rs`** (1 test) — basic MerkleProof verification round-trip.
- **`light_client_distinct.rs`** (3 tests) — ROADMAP-2 distinct-deliverable pin + architectural-absence pins for mode-(b) range-query + mode-(c) signed-checkpoint.
- **`prop_mst.rs`** (1 proptest, 10 000 cases) — concurrent divergent writes converge to same root.

### G16-D handshake / atrium-flow wave

- **`handshake.rs`** (5 tests; some previously `#[ignore]`'d at G16-A) — DID-based mutual-auth round-trip, invalid-signature rejection, UCAN grant exchange, replay-within-bounded-window rejection, handshake-time revocation-set synchronisation gate.
- **`atrium_join.rs`** (2 tests) — atrium-join flow end-to-end + revoke-peer terminates active subscriptions.
- **`atrium_revoke_order.rs`** (4 tests; some `#[ignore]`'d) — revocation-order at reconnect: revocation MUST drain before data after offline → reconnect.
- **`atrium_partial_partition.rs`** (no `#[ignore]`) — asymmetric reachability surfaces as observable typed transport error (net-major-3).

### Adversarial / attack-fixture pins (sec-r4r2-1, R6-FP Wave-C1)

- **`attack_hlc_skew_revocation_ordering.rs`** (1 test, was `#[ignore]`'d pre-Wave-C1) — adversarial `BentenHlc { physical_ms = u64::MAX/2 }` rejected by production `Hlc::update` skew classifier.
- **`attack_loro_op_log_inv_13.rs`** (2 tests, some `#[ignore]`'d pre-Wave-C1) — Inv-13 row-4 SPLIT classifier rejects Loro op-log targets touching system-zone namespace.
- **`attack_mst_diff_cid_mismatch.rs`** (2 tests, some `#[ignore]`'d pre-Wave-C1) — `Mst::apply_entries` rehash check rejects entries whose declared CID doesn't match payload bytes.
- **`wire_envelope.rs`** (1 test, `#[ignore]`'d) — non-handshake message kinds (data sync chunks, MST diff frames, Loro updates) carry device-DID coverage at the wire level per net-r4-r1-2.

### Cross-crate coordination + caps-consumption pins

- **`host_atrium_publish_view_result_caps.rs`** (5 tests; some `#[ignore]`'d) — `host:atrium:publish_view_result` capability + 5 trust-mode patterns per D2 / D-PHASE-3-21 option (iii).
- **`rate_limit_consumption.rs`** (1 test, `#[ignore]`'d) — G14-B → G16-B coordination handoff pin: Loro merge throttle consumes the `benten-caps` rate-limit policy. Originally a 25th p/c drift instance precursor (tcc-r1-2 R4 large-council finding).

---

## 6. Benches inventory

No `benches/` directory. `Cargo.toml` carries `bench = false` on the lib target, deliberately mirroring `crates/benten-core/Cargo.toml`'s disable of the implicit libtest bench harness. Performance assertions live in proptests (10 000-case convergence under arbitrary interleavings) + in `tests/mst_diff.rs::mst_diff_convergence_o_log_n_for_corpus_with_depth_4_branch_8` (asserts the convergence-bound shape) + in `light_client_distinct.rs` (asserts bandwidth-budget assertion).

---

## 7. Thin-engine + composable-graph philosophy check

### What this crate respects well

**Layered dep direction.** `benten-sync` is architecturally downstream of `benten-engine` — it owns no engine concerns (no IVM, no evaluator state, no graph type, no Engine handle, no anchor / version chain). It exports types that the engine WRAPS into its own user-facing surface. The dependency-edges architectural pin (`tests/dependency_edges.rs`) walks Cargo.toml dep tables programmatically and forbids `benten-engine` + `benten-eval`. This is well-respected.

**3-rung baked-in #17 defense.** Compile-error macro + cfg-gated Cargo dep tables + CI wasm-check cell. Clean, well-documented, mutually reinforcing. Future agent proposals to ship Loro / iroh / direct-sync state in wasm32 bundles run into all three rungs.

**HLC-explicit LWW.** The decision to encode HLC inside the value rather than trust Loro's internal Lamport is the right call — keeps Benten's HLC-everywhere posture intact and makes cross-process WAIT-resume, Inv-14 device-grain attribution, and revocation-vs-data ordering all composable. The list-of-flat-strings storage shape is structurally simple and avoids the documented Loro Map concurrent-container-create hazard.

**Type-state builder for `HandshakeFrame`.** Both-DIDs-required is enforced at COMPILE time, not at runtime. A caller missing `peer_did` or `device_did` cannot reach `build()`. This is the right defense — Inv-14 device-grain attribution depends on the device-DID being end-to-end observable.

**Tier-stratified MstDiffSession queue.** Revocation drains before data is enforced both at the wire enum (discriminant ordering) AND at the runtime drainer (separate queues consulted at enqueue time, O(1) drain per message in tier-major order). Two-layer defense against net-blocker-3. Both layers documented + tested.

**Application-layer rehash check at `Mst::apply_entries`.** Defends sec-r4r2-1 cleanly — every adversarial entry rejects BEFORE application, without atomic-batch loss (the batch's atomic-or-partial behaviour is delegated to the engine consumer, where it belongs).

**Disclosure strings for relay modes.** `BootstrapMode::operator_observability_disclosure()` is the right place for the Compromise #22 trust-boundary disclosure surface — operator-readable text bound to the bootstrap choice, snapshot-asserted so UI consumers can lean on it.

**Structural-always-on per-row cap-recheck.** Mentioned by CLAUDE.md as living at `apply_atrium_merge` in `benten-engine`, which is the right side of the layered seam — the engine is the trust-boundary owner; sync surfaces the bytes + the typed errors. The per-row recheck (PR #161 G16-B-F closure) is engine-side and consumes the sync-side primitives (`LoroDoc::op_log_targets`, etc.) cleanly. Sync exposes the surface; engine enforces.

### Possible concerns / things to track

**iroh-specific transport baked deep.** `transport.rs` directly imports iroh types (`iroh::Endpoint`, `iroh::EndpointAddr`, `iroh::SecretKey`, `iroh::endpoint::presets`). The `Endpoint` and `Connection` newtype wrappers exist, but the iroh-shaped surface leaks through: `loopback_addr()` returns `iroh::EndpointAddr`, `connect_to_addr` consumes `EndpointAddr`, `peer_discovery::bind_atrium_peer` builds `iroh::Endpoint` directly. CLAUDE.md #19 explicitly contemplates engine-level extensions for alternate transports (Tor / Nostr-relay / shaped relay). If a Phase-9+ extension wants to swap iroh for a different QUIC implementation, the swap target would need to either (a) provide iroh-compatible types or (b) require refactoring the public surface. There is no `trait Transport` abstraction layer in this crate. **Mitigation that already exists:** `BootstrapMode` is the seam for choosing between iroh's relay modes, and a future `Transport` trait could be introduced WITHOUT changing the engine-facing API since the engine only sees `Endpoint` / `Connection` newtype methods. Worth flagging for Phase-9+ engine-extension work but not a present-day defect.

**Loro-specific shape baked into CRDT.** `crdt.rs` directly imports `loro::{LoroDoc, LoroList, LoroMap, LoroValue, ExportMode}`. The newtype `LoroDoc` wraps `Arc<loro::LoroDoc>` but the rich-type accessors (`list(name) -> loro::LoroList`, `map(name) -> loro::LoroMap`) expose Loro's types directly. A future swap to a different CRDT (Automerge, Yjs, hand-rolled) would face surface change. **Mitigation:** Loro was the chosen CRDT after the D-PHASE-3-4 decision; the choice is plausibly settled for the engine's life. The `winning_attribution` / `all_writes` / `op_log_targets` accessors are CRDT-agnostic in shape (return Benten-types not Loro-types) which is the right move at the engine-consumed boundary. Same as the iroh seam — if a `trait Crdt` abstraction is needed at Phase-9+, the engine-facing surface could absorb it without breaking changes.

**The "OpLogTarget walks deep_value root names only" approximation.** `LoroDoc::op_log_targets()` currently returns a static set of container roots (`benten:properties` + the rich `benten:rich:*` prefix names visible at the deep value) rather than a finer-grained per-op container-id walk. The crate's comment acknowledges this is sufficient for the dispatch decision today (system-zone Nodes never share their property root with user-data Nodes) but flags a finer-grained walk as a wave-6b-r6-fp surface. **The defense-in-depth from the engine side** (`apply_atrium_merge` row-loop with structural-always-on per-row cap-recheck) compensates for the approximation. Worth surfacing if Phase-4 plugin work wants finer attribution granularity.

**MerkleProof shape is O(n) not O(log n).** `mst.rs::Mst::merkle_proof_for` returns the full sorted `(key, cid)` set rather than a tree-shaped O(log n) Merkle path. Crate docs explicitly note this is a Phase-4 optimisation target. The thin-client bandwidth-saving still holds because payload bytes aren't in the proof — the saving is `payload-size × n` which dominates for any non-trivial Node. Future agent proposals for tree-shaped proofs (mode-(b) range-query) need to coordinate with the verifier surface in `light_client.rs`.

**`MessageKind` duplicated across `handshake.rs` + `mst_proto.rs`.** Both files note this is intentional (G16-D handshake-layer floor + G16-C mst-proto-layer ordering) and both pin `Revocation = 0` / `Data = 1` so the merged enum will be a serialised no-op. Reconciliation to a single source of truth is a tracked TODO in the file header. Today's risk is small (discriminants are asserted equal in tests; serde shape is identical) but the duplication is a future-drift surface.

**`Endpoint::bind_with_relay_url` returns canary-scope error rather than instantiating real relay-mode.** The wave-6b production path moved to `peer_discovery::bind_atrium_peer`; the canary entry-point retains its returning-typed-error shape per pim-4 §3.10. The two-arm distinguishing test (`bind_with_relay_url_rejects_malformed_url_with_typed_error` vs `bind_with_relay_url_well_formed_returns_canary_scope_typed_error`) keeps both arms exercised so the canary's promotion seam is not silently bypassable. Not a defect, but a future cleanup if the canary path is fully retired (Phase 9+).

**Random nonce derived from Keypair generation rather than `rand` direct dep.** `handshake.rs::random_nonce()` generates a fresh Keypair, takes its public-key bytes, and discards the keypair. The crate notes this is a deliberate choice to avoid pulling `rand` in as a direct dep. The entropy source is already-vetted (workspace-blessed) but the construction is a bit indirect. A reviewer auditing entropy-source provenance for security purposes will need to follow the chain through `benten-id::keypair::Keypair::generate`.

**Light-client `bytes_consumed` is process-cumulative, not per-verification.** `LightClient::bytes_consumed()` returns total bytes across ALL verifications since construction. The per-verification byte count is in `VerificationResult::bytes_consumed`. Test `bytes_consumed_tracks_across_verifications` verifies the cumulative behaviour. Mostly cosmetic; just a naming-vs-shape thing for downstream consumers.

### Light-client mode-(a) cleanly extends to mode-(b/c)?

The current shape:

- mode-(a) **single-CID inclusion proof**: `MerkleProof { target_key, target_cid, sorted_pairs }` + `LightClient::verify(root, path, proof)`. The proof carries the full sorted pair set.
- mode-(b) **range-query proof** (Phase 9+ per docs/future/phase-3-backlog.md §12): a tree-shaped Merkle path proving a range of keys.
- mode-(c) **signed checkpoint** (Phase 9+): a quorum-signed assertion of root state at a wall-clock point.

The seam is reasonably clean:

- `LightClient::verify` takes `proof: &MerkleProof`, which is a single struct shape. Mode-(b) would either extend `MerkleProof` (add a `range: Option<KeyRange>` field) or introduce a new proof type (`RangeProof`) verified via a parallel surface (`LightClient::verify_range`). Either path is non-breaking on the existing mode-(a) surface.
- `BandwidthBudget` already generalises (per-verification byte cap is mode-agnostic).
- `Mst::merkle_proof_for` is the construction site; a `Mst::merkle_range_proof_for(start, end)` would land here without touching the existing surface.

Mode-(c) signed-checkpoint introduces a quorum-signature surface that doesn't exist today. It would likely live in a sibling module (`checkpoint.rs`) consumed by `LightClient::verify_checkpoint` or similar. Doesn't disturb the existing mode-(a) shape.

The architectural-absence pins at `tests/light_client_distinct.rs` for modes (b) + (c) are doing the right thing: they intentionally fail to compile / cite the named destination (`docs/future/phase-3-backlog.md §12` + `docs/FULL-ROADMAP.md` Phase 4 deferred-items) so a future agent considering implementing modes (b/c) is forced through the named-destination registry. Clean.

---

## 8. Phase 4-Foundation + Phase 4-Meta expectations

**Admin UI v0 (Phase 4 v1 platform-shippable per CLAUDE.md #15).** The admin UI needs to be reachable across Atrium peers — so a user managing their personal Benten instance can pull up the same admin UI on their laptop, phone OS app, or a peer device they trust. This exercises:

- The full peer↔full peer handshake: `Handshake::initiate / respond / finalise` exchanging UCAN grants for admin-scope capabilities.
- The `BootstrapMode::CustomPeerList` Garden-relay surface so an operator who wants self-hosted relays (or no relays at all under Disabled mode for LAN-only deploys) has the option.
- The light-client mode-(a) verification path for browser-tab admin views (browser thin-clients verify Node inclusion via Merkle proof against a published root from the connected full peer).

The `operator_observability_disclosure()` string is the surface that Admin UI v0 will display to operators at first-launch so they know the Compromise #22 metadata-leakage posture before joining a relay.

**Plugin manifest install across Atriums (CLAUDE.md #18 plugin trust model).** Plugins are content-addressed subgraphs; their install crosses Atriums via the existing sync path — Loro op-log applied at the receiver, MST diff converges the published manifest CIDs, handshake's UCAN grant exchange carries the `requires` / `shares` policy halves. The architecture is set up to support this:

- The handshake's `RevocationEntry { target_peer_did, path }` carries path-glob scope, which naturally extends to plugin-namespace scoping.
- The `Session::effective_cap_set` derives caps from the remote-to-local grant, which composes with delivery-time intersection (G14-D F6 SUBSCRIBE) for runtime cap-recheck.
- Per-plugin DIDs would surface as additional `Did` entries in the handshake payload + as additional UCAN proof-chain links validated by `validate_chain_no_time_check`.

No surface in `benten-sync` needs to change to support plugin install across Atriums — the manifest schema lands in `benten-engine` (Class B β `read_node_as` + plugin runtime) per the chosen architecture; sync moves the bytes.

**v1-assessment-window items potentially touching this crate:**

- **Identity-recovery protocol choice.** A device that loses its keypair needs to recover its identity (re-attest to the account-level peer-DID from a fresh device-DID with operator approval / multi-device-attestation quorum). The handshake's device-DID field is already the seam where a recovery flow would land — a new `HandshakePayload` variant `RecoveryAttestation { lost_device_did, new_device_did, account_attestation_signature }` would compose naturally. This is currently OOS for Phase 3; the v1-assessment-window decides if it ships before tag v1 or defers to Phase 5+.
- **Engine impl-block generic-cascade lift.** Tracks across the workspace; sync's wrappers are mostly concrete types (`Endpoint`, `LoroDoc`, `Mst`), so the impact here is minimal.
- **Missing_docs sweep.** Crate uses `#![deny(missing_docs)]` so already compliant.

**Phase 4 (the platform itself).** Decentralised self-discovered registry of plugins / modules / extensions. This pulls the `BootstrapMode::CustomPeerList` surface harder (operators choosing which Garden-relays / registry-relays to trust), pulls the light-client mode-(a) verification harder (verifying registry-published manifest CIDs without downloading the full registry), and may motivate mode-(b) range-query proofs (verifying a range of registry entries by category / author / version).

---

## 9. Open questions / unresolved internals

- **When `MessageKind` reconciliation lands** (merging the duplicated `handshake.rs` + `mst_proto.rs` enums into a single source of truth) is currently flagged as a comment-level TODO. No concrete trigger / wave / phase named. Worth surfacing for Phase 4 plan.
- **Compact-old-writes-on-checkpoint** in `crdt.rs`'s append-only property root List is flagged as a future R6-FP optimisation but not currently scheduled. Without it, long-lived Atriums accumulate unbounded property-write history. The doc notes "convergence is observed-correct under the 10 000-case proptest" but doesn't say anything about long-term memory growth. Worth a Phase 4 plan slot.
- **MST proof shape is O(n).** Phase-4 tree-shaped O(log n) Merkle path is named in the docs but not in a backlog destination. Closer to a real Phase 9+ light-client extension surface.
- **Real packet-loss detector.** `transport::Endpoint::simulate_packet_loss(fraction)` is a synthetic test fixture; the comment says "G16-B/D wires real degrade-detection (packet-loss-fraction over a sliding window) once the protocol body lands." The protocol body has landed; the real detector hasn't. Worth a Phase 4 backlog item or pre-v1 cleanup task.
- **`Endpoint::bind_with_relay_url` canary-scope arm** still returns `RelayUnreachable` for the well-formed-URL case. The wave-6b production path is `peer_discovery::bind_atrium_peer`. The canary surface's two-arm test pair stays in place; full retirement of the canary entry-point would simplify the surface but requires confirming no downstream consumer reaches `bind_with_relay_url` directly. Likely a Phase 9+ cleanup unless a v1-assessment-window pass catches it.
- **Handshake nonce-cache for replay-protection.** The current `respond_with_window` check rejects frames whose HLC drift exceeds the window, but the **canonical** replay-detection mechanism — a per-peer nonce-cache recording previously-seen nonces for the window duration — is documented as "left for follow-up" by `handshake.rs`. The bounded-window math catches captured-off-wire replays older than the window; it does NOT catch a replay within the window if the nonce hasn't been cached. The ds-r4-3 pin asserts typed-error + bounded-window math; the nonce-cache mechanism is the next layer.
- **UCAN chain `nbf`/`exp` time-checks deferred to G14-D delivery-time recheck.** `validate_chain_no_time_check` is called at handshake-time. The G14-D F6 SUBSCRIBE delivery-time gate re-evaluates caps including time windows on every delivery. The composition gives defense-in-depth but spread across two crates; a fresh agent reading only `handshake.rs` might miss the time-check half. Worth a cross-crate doc-comment pointer or eventual relocation.
- **Where does the iroh `EndpointId` ↔ `PeerId` round-trip live for production?** `transport.rs::public_key_from_peer_id` constructs `iroh::EndpointId::from_bytes(p.as_bytes())`. The reverse (iroh side to `PeerId`) happens at `accept_next`'s `PeerId::from_bytes(*remote_endpoint_id.as_bytes())`. This is two lines of bytes-passing but isn't formally surfaced as a single round-trip accessor — a fresh agent extending the surface (e.g., a `Connection::remote_iroh_endpoint_id()` accessor) would need to know the byte-equivalence holds. Worth a unit-test pinning the byte-equivalence explicitly.
