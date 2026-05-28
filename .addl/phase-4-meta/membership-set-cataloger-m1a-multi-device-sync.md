# M1a — Multi-Device Sync Cataloger (existing state + plans)

**Pipeline:** MembershipSet unification specialist panel; 1 of 3 parallel catalogers (sibling = M1b Atrium-membership-sharing, M1c key-management).
**Author:** orchestrator-dispatched specialist on branch `phase-4-meta-core/membership-set-cataloger-m1a-multi-device-sync`.
**Tree state at audit start:** `main` HEAD `2172cb6d`, worktree clean.
**Scope strictly:** how Benten supports one user across multiple devices today + what the plans add. Atrium-multi-user sharing and the crypto KDF chain are flagged where I see them but NOT covered in depth (M1b / M1c own those).

---

## §1 Executive Summary

Multi-device sync ALREADY exists at HEAD in a substantive, end-to-end-wired form: Ed25519 keypairs, did:key, device-DID attestation envelopes V2, per-device-grain AttributionFrame (`device_did` + `peer_did_set` + `sync_hop_depth`), HLC-strict RotationLog with replay defenses, an iroh + Loro CRDT sync runtime with handshake-time revocation-set sync, structural-always-on per-row cap-recheck in `apply_atrium_merge`, and the V2 `DeviceAttestationEnvelope` (`payload_hash` + `session_nonce` + envelope-signature + freshness window). Plan §1 exit criterion 16 ("multi-device support for single identity") was closed at Phase-3 close (PR #163, G16-D wave-6b). Phase-4-Foundation added per-device-local Loro-Map CURRENT pointer for plugin versioning.

The PLANNED scope (Phase-4-Meta-Core + Phase-4-Meta-Composing, both pre-v1-beta-tag, RATIFIED 2026-05-27 F-full) is large: a real K_principal store + DAK (Device Authentication Key) + multi-device-key-wrap WIRE + remote-permission-call WIRE + at-rest user-DID-key encryption + Device-link UX flow (QR + approval) + identity-recovery hook. The envelope SHAPE for multi-device key-wrap/recovery is in the v1-beta freeze set (`docs/V1-FROZEN-INTERFACE.md` item 15.5, frozen alongside #1301). The PROTOCOL choice (Shamir / social / hardware / MLS-style) stays G-COMP-3 v1-assessment-window.

Multi-device exhibits clear MembershipSet-shape patterns: the Atrium's "device set" (per-user) maps to "members are devices"; encrypt-to-recipient over a device-set maps to a MembershipSet with members = devices; the device-DID attestation chain maps to "membership proof signed by set-owner"; revocation = "remove member". HOWEVER, the existing code has a substantive *asymmetry*: device-DID and user-DID are NOT a single primitive (per CLAUDE.md baked-in #18 — devices are attested sub-identities of hardware; plugins are NOT — and per baked-in #17 — devices have a `CapabilityEnvelope` minimum-capability shape based on deployment shape (a)/(b)/(c)). The MembershipSet unification will need to either reconcile that asymmetry or surface it as a distinct concept.

**Confidence: high on current state (read all the cited code).** Moderate on long-horizon plans (CLAUDE.md-encoded; F-full R0 plan-doc not yet written; Phase 5-8 specifics from FULL-ROADMAP only). Some planning docs cited in the brief (`SESSION-2026-05-20-to-2026-05-21-substrate-wave-and-willow-pivot.md` / `RATIFIED-sharing-and-confidentiality-2026-05-21.md` / `option-f-plus-9-eyes-consolidated-registry.md` / `option-f-plus-lens-l9-atrium-integration.md` / `encrypt-to-recipient-review-ffull-scope.md`) are NOT on this worktree's HEAD (untracked / on other branches like `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`); referenced via git history but not read in full. See §10 for confidence breakdown.

---

## §2 Current-state code inventory (multi-device-relevant; at HEAD `2172cb6d`)

### 2.1 `crates/benten-id/` — identity primitives (LIVE)

| Surface | File | State | Notes |
|---|---|---|---|
| `Keypair { secret: SecretKey, signing, verifying }` | `src/keypair.rs:189-198` | LIVE | Ed25519 via `benten_crypto_suite::primitives::ed25519_dalek`; `SecretKey` is `Zeroize + ZeroizeOnDrop + !Clone` with redacted Debug per `crypto-blocker-1`. `Keypair::generate()` pinned to `OsRng` per `crypto-major-2`. **One keypair per device today** (no separate user-DID keypair vs device-DID keypair primitive distinction at the keypair layer). |
| `Keypair::export_seed_envelope()` / `Keypair::from_seed_bytes(bytes)` / `from_dag_cbor_envelope(bytes)` | `keypair.rs:297-337` | LIVE | DAG-CBOR `{version: u8, alg: "Ed25519", secret_bytes: Bytes(32)}` envelope per `crypto-major-5`. This is the cross-device transport shape today: a device that holds a seed envelope can re-instantiate the keypair on another device. NOT password-wrapped, NOT recipient-encrypted — that's F-full Layer-D scope. |
| `Keypair::secret_bytes_unprotected()` | `keypair.rs:283-286` | LIVE | Production caller for iroh keypair construction (`benten-sync/src/transport.rs` + `peer_discovery.rs`) — iroh `EndpointId` IS the Ed25519 verifying key bytes (Spike A2 finding; documented in `crates/benten-id/INTERNALS.md` §3). |
| `Did` (did:key) | `src/did.rs` | LIVE | `did:key:z<...>` per W3C method-key spec; multibase `z` + multicodec `0xed01`. `serde(transparent)` round-trips string form without validation; `resolve()` returns typed error. Used by `benten-sync` `HandshakeFrame` wire-format per `net-blocker-4`. |
| `DeviceAttestation` + `CapabilityEnvelope` + `envelope_widens` + 5 `issue_*` constructors | `src/device_attestation.rs` (608 LOC) | LIVE (Compromise #23 substrate) | `(device_did, parent_did, envelope, nonce, issued_at, signature)`. `CapabilityEnvelope` carries `runs_sandbox / holds_zones / online_uptime / runs_atrium_peer` (the 4-dimension hardware capability declaration matched to deployment shape (a)/(b)/(c) per CLAUDE.md #17). `issue_for_browser_target` auto-asserts minimum envelope (browser tab shape). `issue_with_authority` rejects with `EnvelopeWidening` when device claims wider authority than parent envelope (`cap-r4-7`). **POST-COLLAPSE:** the entire `Acceptor` runtime gate cluster (`Acceptor::new` / `new_with_revocations` / `with_parent_lookup` / `accept_at` / `accept`), `DeviceRevocation`, `RevocationReason`, `FreshnessPolicy`, `revocation_canonical_bytes`, and the chain-walker gate `validate_chain_with_device_revocations` are ALL DELETED per `crates/benten-id/INTERNALS.md` `(COLLAPSE P0-P3)`. Surviving: the signed `DeviceAttestation` struct + `CapabilityEnvelope` type + `envelope_widens` + `generate_fresh_nonce`. The acceptance pipe collapsed to the unified principal + envelope-ceiling model recheck at `Engine::apply_atrium_merge`. |
| `RotationLog` + `RotationAttestation` + `rotate_keypair(did, old_kp, new_kp, superseded_at)` | `src/did_rotation.rs` (276 LOC) | LIVE (extended at G24-D-FP-2) | In-RAM `Vec<RotationAttestation>` (rehydration deferred to Phase-4-Meta per `tests/rotation_log_rehydrated_at_engine_open.rs` RED-PHASE pin + `phase-4-backlog.md §4.26`). 3 composed defenses at `accept_rotation_event`: (0) authenticity-gate via `ct_signature_eq`; (1) verbatim-replay reject; (2) HLC-monotonic-strict (closes nonce-swap attack class). Signed by **OLD keypair** — proves authorization. Test pins at `crates/benten-id/tests/did_rotation.rs`. The "logical DID" survives rotation as the string itself; what rotates is the underlying keypair. |
| `PluginDidStore` + `mint() / mint_and_store() / insert()` | `src/plugin_did.rs` (229 LOC) | LIVE (G24-D) | Per CLAUDE.md #18 plugin-DID is the 3rd of 4 identity concepts (NOT an attested sub-identity; just a UCAN audience handle). One keypair per install, OsRng-minted, NO HKDF-from-user-DID derivation (asserted by 2 RED-PHASE-promoted tests). Plugin-DID is distinct from device-DID — see §6 for the 4-identity-concept tree. |
| `MultiSigSurface` trait + `Ed25519SingleKey` default impl + `ThresholdMultiSig` placeholder | `src/multi_sig.rs` (158 LOC) | LIVE (shape only) | Extension point for threshold/multi-sig identity-recovery protocols. Phase-3 ships only `Ed25519SingleKey` with `threshold()=1, participants()=1`. `ThresholdMultiSig` bodies return `MultiSigError::PostPhase3`. Per Phase-4-Foundation R1 Ben-ratification #6: identity-recovery MVP via `SelfRevocation` (NOT threshold); Kith effort (Phase 5+) is the future home of threshold-based recovery. |
| `GrantReader` sibling trait (G27-C) | `src/grant_reader.rs` (316 LOC) | LIVE | Sibling-not-extension of `benten_caps::grant_backed::GrantReader` (arch-r1-10 forbids `benten-id → benten-caps`). Two methods: scope-keyed + CID-keyed. Used by engine's `revoke_capability_by_grant_cid` + napi `revokeCapability(grantCid, actor)`. Not directly multi-device-shaped but enables CID-keyed revocation surfaces that compose with multi-device. |

### 2.2 `crates/benten-sync/` — Atrium transport + CRDT + handshake (LIVE)

| Surface | File | State | Notes |
|---|---|---|---|
| `PeerId { bytes: [u8; 32] }` | `src/peer_id.rs` | LIVE | `serde(transparent)`; **`PeerId == iroh EndpointId == Ed25519 pubkey bytes`** per `net-minor-2 + ds-8 + crypto-minor-4`. Cross-process determinism is load-bearing for multi-device peer-mesh (D-PHASE-3-25). One Ed25519 key signs both UCAN chains (engine-layer) AND QUIC TLS handshakes (transport-layer) — trust-coupling tradeoff documented; Phase 9+ may re-open. |
| `transport::Endpoint` + `transport::Connection` (iroh QUIC) | `src/transport.rs` (~825 LOC) | LIVE | One per peer-process. ALPN `b"benten/atrium/1"`. `bind_loopback / bind_loopback_with_keypair / bind_with_keypair / from_iroh_parts`. `TransportKind { Direct, Relay, Loopback }` discriminator + `TransportStatus { Healthy, Degraded, NotConnected }`. The wave-6b production-binding entry is `peer_discovery::bind_atrium_peer(kp, config)`. |
| `BootstrapMode { DefaultRelay, CustomPeerList, Disabled }` + `operator_observability_disclosure()` | `src/peer_discovery.rs` (~364 LOC) | LIVE | Operator-readable trust-boundary disclosure per `net-major-1 + sec-r1-12`. DefaultRelay → public iroh relays (Compromise #22 metadata-leakage); CustomPeerList → operator-controlled (Phase-7 Garden-relays); Disabled → no relay. |
| `HandshakeFrame { version, peer_did, device_did, peer_id, protocol_payload }` | `src/handshake_wire.rs` (365 LOC) | LIVE | **Both `peer_did` AND `device_did` REQUIRED at wire-format** per `net-blocker-4`. Enforced at COMPILE time via type-state builder (phantom-state marker structs `NoPeerDid` / `WithPeerDid` / `NoDeviceDid` / `WithDeviceDid` / `NoPeerId` / `WithPeerId`; only `(With, With, With)` has `build()`). DAG-CBOR canonical-bytes round-trip. **This is the load-bearing multi-device wire-format pin: every handshake carries device-DID end-to-end.** |
| `Handshake { initiate, respond, respond_with_window, finalise }` + `Session` + `EffectiveCapSet` + `RevocationEntry` | `src/handshake.rs` (~1276 LOC) | LIVE | 3-step DID-based mutual-auth state machine. `Initiate { audience_did, nonce: [u8;32], hlc_physical_ms, grant: Option<Ucan>, revocation_set: Vec<RevocationEntry>, signature }`. `Respond { echoed_nonce, hlc_physical_ms, grant, revocation_set, signature }`. **`Session.synchronized_revocations`** is the load-bearing handshake-time revocation-set gate per `net-r4-r1-3`: responder UNIONS initiator's advertised revocations with its own local set BEFORE returning `Session`; `subscription_open_permitted()` consults the sealed snapshot so the post-handshake SUBSCRIBE gate cannot open until the cache is seeded. **`DEFAULT_REPLAY_WINDOW_MS = 5_000`** + bounded-window HLC math at `respond`. Random nonce derived from `Keypair::generate()` pubkey bytes (avoids `rand` direct dep). `dedup_synchronized_revocations` per Safe-3 #613. |
| `MessageKind { Revocation = 0, Data = 1 }` | `handshake.rs` + `mst_proto.rs` (duplicated) | LIVE | Discriminants are load-bearing per `net-blocker-3`: revocations drain before data. `MstDiffSession` enforces a tier-stratified queue (revocation_queue + data_queue) at runtime drainer. Duplication between two files is intentional; reconciliation tracked. |
| `LoroDoc` + `StampedValue { value, hlc }` + `HlcWire` | `src/crdt.rs` (~947 LOC) | LIVE | Loro at Node-property granularity per `D-PHASE-3-4`. **HLC carries INSIDE the value** (packed `<physical>:<logical>:<node>:<value>` string in a root List at `benten:properties`) — Loro's internal Lamport is NOT trusted for LWW. Rich types under `benten:rich:<name>`. `winning_attribution()` returns the union of every observed write's HLC `node_id` (load-bearing for revocation-vs-data ordering audit). `all_writes()` surfaces `(property_key, StampedValue)` for engine-side `AttributionFrame` mint per arch-r1-4 D-C HYBRID. |
| `Mst` + `MstCid` + `MstEntry` + `MstDiff` + `MstDiffSession` | `src/mst.rs` (~743 LOC) + `mst_proto.rs` (~472 LOC) | LIVE | BTreeMap-based MST per Phase-3 G16-C. Root computation = canonical sorted `(String, MstCid)` pairs → DAG-CBOR → BLAKE3. Application-layer rehash check at `apply_entries` defends `sec-r4r2-1` (declared CID ≠ payload-CID reject). MerkleProof shape currently O(n); tree-shaped O(log n) named for Phase-4-Meta. |
| `LightClient::verify(root, path, proof)` | `src/light_client.rs` (~418 LOC) | LIVE (mode-(a) only) | Single-CID inclusion proof per `ds-r4r2-3 mode-(a)`. Mode-(b) range-query + mode-(c) signed-checkpoint are architectural-absence-pinned for Phase-4-Meta. Per CLAUDE.md #17, this is the surface a browser-tab thin-client uses to verify Node inclusion against a full-peer-published root. |
| `transport_trait::{Transport, TransportEndpoint, TransportConnection}` + `IrohTransport` | `src/transport_trait.rs` | LIVE (RATIFIED §15.3 #1 / umbrella #1176) | Abstraction boundary over iroh-concrete. Post-v1 alternate transports (Tor / Nostr-relay / shaped relay per CLAUDE.md #19) implement these traits as compile-time engine extensions. |
| `ucan_blobs_protocol` + `two_cid_store::TwoCidStore` | `src/ucan_blobs_protocol.rs` + `src/two_cid_store.rs` | LIVE (G-CORE-3e wave-3e) | UCAN-gated iroh-blobs custom-ALPN handler (Flavor B per-request UCAN check). The `TwoCidStore` is the in-memory swap-point for `iroh_blobs::FsStore`; durable plaintext→ciphertext mapping table lives at `benten-graph::two_cid_map` per V1-FROZEN-INTERFACE item 15(g). |

### 2.3 `crates/benten-engine/` — multi-device-relevant engine surface

| Surface | File:line | State | Notes |
|---|---|---|---|
| `Engine::set_device_cid(Option<Cid>)` + `device_cid` field | `src/engine.rs:339-353` + `:2017` | LIVE | The engine's own device-CID slot (`None` for non-device-attestation engines; `Some(cid)` for engines whose owner wired in a `DeviceAttestation`). `device_cid` is encoded as `device-cid:<hex>` and lands in `AttributionFrame.device_did` per `crates/benten-sync/tests/sync_replica_attribution.rs`. |
| `Engine::set_actor_cid(Option<Cid>)` + Option-A decoupling | `src/engine.rs` + `sync_replica_attribution.rs:143` | LIVE (Ben RATIFIED 2026-05-08) | Actor identity (the user / Atrium principal) decoupled from device identity. When `set_actor_cid` is called, post-merge `AttributionFrame.actor_cid` carries the EXPLICIT actor, NOT the device — defends against AI-agent / handler-attribution conflation in Phase 4+. |
| `Engine::revoked_device_dids: Mutex<HashSet<String>>` + per-device-DID revocation set | `src/engine.rs:933-939` | LIVE (G14-D wave-5a) | SUBSCRIBE subscriptions bound to a device-DID auto-cancel on the next delivery once the device-DID is added per `crypto-major-6`. **Per-device-DID revocation in this map is in-RAM**; durable propagation rides Phase-3 UCAN per-grant revocation + RotationLog. |
| `Engine::apply_atrium_merge` row-loop with structural-always-on per-row cap-recheck (G16-B-F PR #161) | `src/engine.rs:1607-1759` (env-recheck), `:1671-1706` (device-DID slot) | LIVE | **The load-bearing trust-boundary seam for multi-device merge.** Per row: (a) `policy.check_write` for cap revocation (emits `E_SYNC_REVOKED_DURING_SESSION`); (b) `ManifestEnvelopeRechecker` consultation for plugin-delegation-outside-manifest-envelope refinement (R4b-FP-1 Seam 3 extension); (c) the `device_did` slot in `AttributionFrame` reflects the DeviceAttestationEnvelope's declared `device_did` when present, else falls back to local `device_cid`. `clear_last_received_remote_device_did(zone)` per zone-merge per G16-D wave-6b to prevent device-DID inheritance across merges. |
| `AttributionFrame { actor_cid, handler_cid, capability_grant_cid, sandbox_depth, device_did: Option<String>, peer_did_set: Option<BTreeSet<String>>, sync_hop_depth }` | `benten-eval::AttributionFrame` (used at `sync_replica_attribution.rs:122-132`) | LIVE | The cross-Atrium attribution shape. `device_did` slot carries the device-grain identity end-to-end (Inv-14). `peer_did_set` is the resolved DIDs of CRDT contributors. `sync_hop_depth` increments per merge boundary (Compromise #2 sub-narrative). |
| `DeviceAttestationEnvelope V2` `(version, attestation, payload_hash, session_nonce, envelope_signature, device_did: Option<String>)` + `verify(loro_payload, freshness_window_secs, now_secs)` | `src/engine_sync.rs:311-662` | LIVE (PR #163, G16-D wave-6b — Compromise #23 SHIPPED) | V2 wire shape. 4 cryptographic defenses in `verify`: (1) envelope-signature against device-DID's resolved pubkey (DID-forgery defense); (2) embedded attestation signature against parent_did's pubkey (user-root delegation-link forgery; J-COLLAPSE simplified the chain-walk); (3) freshness gate `now - issued_at <= freshness_window_secs` (stale-frame anti-replay; J5 re-home); (4) constant-time `BLAKE3(loro_payload) == self.payload_hash` (frame-pair swap defense). All failure modes reject with `E_DEVICE_ATTESTATION_FORGED`. Returns the verified `CapabilityEnvelope` for the J8 ceiling-AND at `apply_atrium_merge`. **`attestation = None` envelopes** skip verify + return `None` (V1 legacy / test-fixture backward compat). |
| `AtriumHandle::set_local_device_attestation` + `set_local_device_did` + `register_device_attestation` + `declared_device_attestations: Mutex<BTreeMap<String, DeclaredDeviceAttestation>>` | `src/engine_sync.rs:715-789` | LIVE | Per-handle declared envelopes keyed by `device_did`; presented at handshake time (the wire-up to G16-D handshake state machine is partially in flight per the comment). |
| `AtriumHandle::register_peer_did(node_id, did)` + `peer_did_registry: Mutex<BTreeMap<u64, String>>` | `engine_sync.rs:720-733` | LIVE (G16-B-prime §6.12 #3) | Trust-store mapping CRDT peer node-id → resolved peer-DID. Populated explicitly today; G16-D handshake wires the post-handshake population. Unresolved node-ids fall back to `node-id:NNN` synthetic strings (pim-2 end-to-end pin). |
| `AtriumHandle { open, accept_invite, leave, rejoin, is_active, subscribe, register_zone, with_zone, apply_atrium_merge, sync_subgraph, accept_sync_subgraph, merge_remote_change, ... }` | `src/engine_sync.rs` (~2135 LOC total) | LIVE | The high-level multi-device-sync API. Per-zone Loro CRDT documents. Cloneable via Arc. Peer-churn lifecycle flag (`true` after open/rejoin; `false` after leave); sync-touching surfaces are gated. |
| `ChangeStream` trait + `ChangeEvent { actor_cid, handler_cid, capability_grant_cid, ... }` | `crates/benten-core/src/change_stream.rs` | LIVE (`#[non_exhaustive]`) | Multi-source change-event merger (Phase-3 `local + remote-peer` merger). `ChangeEvent` is `#[non_exhaustive]` with documented widening trajectory: multi-device / Kith attribution expected to widen further. `SubscriberId` is content-addressed for cross-peer deterministic re-establish. |

### 2.4 Tests that codify the existing multi-device-sync UX-acceptance contract

| Test | File:line | Status | Pins what |
|---|---|---|---|
| `dogfood_path_c_multi_device_sync_ux_acceptance` | `crates/benten-engine/tests/dogfood_path_c_multi_device_sync_ux_acceptance.rs` | LIVE | At G24-A: only the content-addressing convergence arm (two `Engine::create_node` on different engines for same bytes yield same CID — substrate property). **≤3s loopback latency + Devices-sub-panel last-sync-time UX BELONG to wave-9 dogfood gate**, named at `docs/future/phase-4-backlog.md §2`. |
| `dogfood_path_e_install_admin_ui_on_2nd_device_arm` | `tests/common/admin_ui_v0_dogfood.rs:201-229` | LIVE | Cross-device install precondition: admin UI v0 subgraph canonical bytes stable across builds + all 4 nav categories present. Signed-manifest-envelope path at G24-D + wave-9. |
| `sync_replica_write_attribution_carries_device_did_alongside_parent` (exploration-device-mesh GREEN-PHASE) | `crates/benten-engine/tests/sync_replica_attribution.rs:34-139` | LIVE | Inv-14 device-grain attribution: every sync-replica merge mints an AttributionFrame with BOTH parent DID + device-DID. Defends "compromised device cannot be isolated" failure shape. |
| `sync_replica_explicit_actor_cid_decouples_from_device_cid` | `sync_replica_attribution.rs:142-` | LIVE (Ben RATIFIED 2026-05-08 Option A) | When `set_actor_cid` is set, `AttributionFrame.actor_cid` carries EXPLICIT actor, NOT device — preserves Phase-4+ AI-agent / handler-attribution distinction. |
| `subscribe_subscription_path_terminated_when_device_did_revoked` | `crates/benten-engine/tests/subscribe_device_revoke.rs` | **`#[ignore]`** | `phase-3-backlog §7.3.D`; un-ignore at §2.3 (i) WriteContext threading landing (v1-assessment-window). Production revocation flow + observable subscription-state termination not yet wired end-to-end. |
| `subscribe_cap_recheck` tests | `crates/benten-engine/tests/subscribe_cap_recheck.rs` (382 LOC) | LIVE | Per-event delivery-time cap-recheck pin (`E_SUBSCRIBE_REVOKED_MID_STREAM`). Composes with the per-device-DID revocation set. |
| `did_rotate_keypair_*` pins | `crates/benten-id/tests/did_rotation.rs` | mostly LIVE (1 RED-PHASE) | `superseded_by_attestation_chain` + `preserves_did_under_canonical_bytes` + 3 `accept_rotation_event` authenticity pins (forged-sig reject / wrong-key reject / genuine accept + verbatim-replay reject). `did_rotation_propagates_revocation_to_ucan_backend` is `#[ignore]` pending Phase-4-Meta `§2.1-followup` re-evaluation. |
| `apply_atrium_merge_manifest_envelope_recheck` | `crates/benten-sync/tests/apply_atrium_merge_manifest_envelope_recheck.rs` | LIVE (R4b-FP-1 Seam 3) | Grep-walk pin asserting structural-always-on per-row cap-recheck path EXTENDS to manifest-envelope-recheck for plugin-delegation-outside-manifest-envelope. |
| `admin_ui_v0_atrium_share_bytes_dont_match_announced_cid_rejected` (T6a) + `admin_ui_v0_atrium_share_substitution_with_different_author_rejected` (T6b) | `benten-sync/tests/` | DESTINATION-REMAPPED | Cross-Atrium sync-layer hydrate-time verifier NOT yet wired. Named destination: `docs/future/phase-4-backlog.md §4.25`. |

### 2.5 What does NOT exist at HEAD (multi-device-relevant gaps)

- **Device pairing UX (QR-code + approval flow).** No code at HEAD. Named in CLAUDE.md NIGHT-SHIFT-2026-05-27 as Phase-4-Meta-Composing scope.
- **Multi-device-key-wrap WIRE.** No code at HEAD. The envelope SHAPE is in v1-beta freeze set (`docs/V1-FROZEN-INTERFACE.md` item 15.5) but `K_principal` is currently SYNTHESIZED from `namespace_did` via BLAKE3 keyed-hash at G-CORE-3e (`crates/benten-graph/src/redb_backend.rs::derive_test_seam_key_from_cid_with_namespace`) — NOT a real per-deployment K_principal store. F-full Layer-A scope.
- **Remote-permission-call WIRE.** No code at HEAD. Signal-Provisioning + CTAP-2.2 inspired protocol shape RATIFIED 2026-05-27; brief authored; not yet implemented. Ben emphatic 2026-05-27 it is REQUIRED v1-beta scope (not cuttable).
- **DAK (Device Authentication Key) substrate.** No code at HEAD. F-full Layer-D scope.
- **At-rest user-DID-key encryption.** No code at HEAD. F-full Layer-D scope. Currently `secret_bytes_unprotected()` carries the seed in cleartext through napi typed-CALL boundary + iroh keypair construction.
- **RotationLog durable rehydration at engine-open.** RED-PHASE pin at `crates/benten-id/tests/rotation_log_rehydrated_at_engine_open.rs`; couples to `phase-4-backlog.md §4.26` + `§4.20` engine-builder seam (Phase-4-Meta).
- **Real packet-loss detector** (`transport::Endpoint::simulate_packet_loss` is synthetic). `benten-sync/INTERNALS.md` §9 flags.
- **Per-peer handshake nonce-cache for replay-protection** (currently bounded-window math only). `handshake.rs` open question per `benten-sync/INTERNALS.md §9`.
- **Production-wired Acceptor freshness/nonce-store/revocation pipe.** COLLAPSE-deleted. Replaced by `DeviceAttestationEnvelope::verify` 4-defense path + user-root UCAN-grant revocation at `benten_caps::revoke`.
- **The 15 remaining sync-attack vectors** (3 of 18 landed: hlc_skew + loro_op_log_inv_13 + mst_diff_cid_mismatch). Composes with Compromises #22/#23/#25/#26 closure narratives; tracked at `docs/future/phase-4-backlog.md §4.58`.
- **MessageKind reconciliation** between `handshake.rs` + `mst_proto.rs` (duplicated definition); flagged in `benten-sync/INTERNALS.md §9`.

---

## §3 Planned-state inventory

### 3.1 Phase-4-Meta-Core (current — wire-format-affecting; ~5-6.5K LOC; pre-v1-beta-tag)

Per CLAUDE.md NIGHT-SHIFT-2026-05-27 LATE-SESSION block (F-full RATIFIED) + `docs/V1-FROZEN-INTERFACE.md` item 15.5:

| Wave | Multi-device-relevant scope | State |
|---|---|---|
| **X-Wing-mislabel corrective** (~24 LOC; INDEPENDENT) | Rename + crypto-suite cipher_suite.rs HKDF info-tag → real X-Wing SHA3-256(label \|\| ss_M \|\| ss_X \|\| ct_X \|\| pk_X) per `draft-connolly-cfrg-xwing-kem-10` §5.3. Codepoint `0x647A` is IETF-reserved. Pre-v1-beta-tag-must-fix. | PRE-DISPATCH; cited at G-CORE-PQ-WIRE-1 (Row D-26) |
| **Layer-A K_principal store** (G-CORE-3e pulled forward) | The real per-deployment K_principal secret material (#989 / #1301 substrate; per-DID secret store) replacing the synthesized BLAKE3 keyed-hash. 1-line swap at the helper's K_principal synthesis step. Multi-device implication: K_principal is what gets WRAPPED for the second device. | DESIGN COMPLETE (RATIFIED-S&C-2026-05-21); IMPL HELD pending F-full R0 plan-doc |
| **Layer-B per-Node AEAD residual** | Already partly LIVE (G-CORE-3a canary `AeadEnvelope` + `IROH_BLOCK_SIZE` chunk); residual = X-Wing-mislabel corrective + chunked-vs-whole heuristic refinement. | partially LIVE |
| **Layer-C encrypt-to-recipient** (HPKE-RFC-9180 + MLKEM768-X25519 + multi-stanza + Inv-16) | Multi-device implication: encrypt-to-second-device IS an encrypt-to-recipient call where recipient = second-device-DID. `multi-stanza-HPKE for groups` IS the MembershipSet-shape encrypt path. Combined Option F 3-cryptographer-reviewer slate CONVERGED 2026-05-27. | DESIGN COMPLETE; IMPL HELD pending F-full R0 |
| **Layer-D DAK substrate + Argon2id + at-rest K_principal/user-DID-key encrypt + multi-device-key-wrap WIRE + remote-permission-call WIRE + minimum desktop platform glue (`keyring-core` + Tauri shell smoke + file-vault fallback)** | **The core F-full multi-device scope.** DAK = Device Authentication Key; protects local engine vault. K_principal at-rest = wrapped under DAK on each device. Multi-device-key-wrap = the wire shape for handing K_principal to a 2nd device (envelope SHAPE frozen at V1-FROZEN-INTERFACE item 15.5). Remote-permission-call = device-1 user approves device-2 doing a sensitive op (Signal-Provisioning + CTAP-2.2 inspired). | DESIGN ARC RATIFIED; F-full R0 plan-doc NOT YET written; needs full ADDL pipeline (R0→R1→R2→R3→R4→R5→R4b) BEFORE R6 R3 can converge |

### 3.2 Phase-4-Meta-Composing (current — UX-coupled; ~1.3-2.4K LOC; pre-v1-beta-tag)

| Item | Multi-device-relevant scope |
|---|---|
| **Biometric layer** | Unlock DAK via Touch ID / Windows Hello / etc on each device. |
| **Stronghold optional backend** | Alternate to `keyring-core`. |
| **Device-link UX flow (QR + approval)** | **The end-to-end UX flow for "user adds 2nd device".** QR-code on device-1 carries an Atrium-join + device-attestation-issuance payload; device-2 scans, mints fresh keypair, requests parent attestation from device-1; user on device-1 approves; device-1 issues `DeviceAttestation` + multi-device-key-wrap envelope. |
| **Remote-permission-call UX flow** | **The end-to-end UX flow for "device-2 asks device-1 for permission to do X".** Sensitive op on device-2 (e.g. issue a high-stakes UCAN delegation, decrypt high-sensitivity Node) triggers a push to device-1; user on device-1 sees plain-English prompt + approves/denies. |
| **Identity-recovery `RecoveryHook` stub trait** | MVP shape; per Phase-4-Foundation R1 ratification #6 the MVP body = `SelfRevocation` attestation; threshold/Kith deferred. |

### 3.3 v1-beta-frozen surfaces touching multi-device (per `docs/V1-FROZEN-INTERFACE.md`)

| Item # | Surface | Frozen as |
|---|---|---|
| **15.5** | **Multi-device key-wrap/recovery envelope SHAPE** | Frozen as part of #1301 per item 6. Recovery PROTOCOL choice (Shamir / social / hardware / MLS-style) stays G-COMP-3 v1-assessment-window; the ENVELOPE SHAPE around the wrap is frozen so a recovery-protocol choice doesn't require re-opening the freeze. |
| 15.b (signature) | `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` (LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512`) | LIVE, default; PQ-hybrid sig that multi-device attestation chains will switch over to via G-CORE-PQ-WIRE (Row D-26). At HEAD, 4 production sites are classical-only Ed25519 (DropBundle envelope-sig; `PluginManifest::verify_peer_signature`; `InstallRecord::verify_user_signature`; `AuthorizationGrant::binding_sig`). Sub-fork α/β/γ TBD. |
| 15.b (cipher) | `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768 = 0x647a` | LIVE, default; encryption combiner for multi-device-key-wrap. |
| 15.f | Two-path key-derivation contract (HKDF-SHA256 `"step"` / `"root"` info-tags) | LIVE per Spike E; per-Node `K(N) = HKDF-SHA256(K(predecessor), info = "step" \|\| edge_label \|\| N.cid)`. Multi-device implication: 2nd device with K_principal + canonical path can derive K(N) for any Node. |
| 15.g | Two-CID mapping + `IROH_BLOCK_SIZE` chunk-size | LIVE; iroh-blobs serves ciphertext blob by its own hash; UCAN scopes against plaintext_cid. |

### 3.4 Deferred (`docs/V1-FROZEN-INTERFACE-DEFERRED.md`)

| Row | Multi-device relevance |
|---|---|
| **D-26 G-CORE-PQ-WIRE wave** | PQ-hybrid app-layer wire-in for 4 classical-only sites (DropBundle envelope-sig + plugin-manifest signatures + binding_sig). Multi-device chains will pick this up. Sub-fork α (additive sibling pubkey field per site) / β (envelope v1→v2 per site) / γ (DID-extension carries hybrid pubkey bytes). Active queued (NOT perpetually deferred). |
| **D-27 StampedValue per-row originating-grant_cid plumbing** | **Direct multi-device-attribution concern.** Currently `apply_atrium_merge` reconstructs `AttributionFrame` per merged row using LOCAL device's `effective_actor_cid` — the ORIGINATING peer's authorization-grant CID does not survive the sync hop. `StampedValue { value, hlc }` has NO per-row grant_cid slot. Multi-hop consumers see laptop's fresh-reconstructed attribution, not phone's original grant. Deferred consumption = G-COMP-1 (`StampedValueV2 { value, hlc, originating_grant_cid: Option<Cid> }`; ~600-800 LOC). Real defer rationale = semantic redesign, not Loro coord. |
| Various D-rows (D-1, D-2, D-3, D-4, D-6, D-8, D-13, D-18, D-19, D-22) | Closed at R6-R1/R6-R2 FP cycles; collectively wire the unified principal + envelope-ceiling model that the multi-device DeviceAttestationEnvelope::verify substrate already exercises. |

### 3.5 Future-phase + long-horizon multi-device scope (coordinator-requested expansion)

Per the coordinator's 2026-05-27 scope expansion: assess MembershipSet unification against the FULL ARCHITECTURAL ARC, not just v1-beta. If long-horizon plans assume different primitive shapes, surface NOW so the v1-beta wire format is correct.

**Source authority for this section:** CLAUDE.md (untracked but read at this session); `docs/VISION.md` + `docs/BUSINESS-PLAN.md` + `docs/FULL-ROADMAP.md` (untracked since `db5831ff`; read from git history at `a645ad8b`); `docs/future/kith-decentralized-identity.md` + `docs/future/phase-4-backlog.md` (TRACKED, read at HEAD); `docs/history/PHASE-3.md` + `docs/history/PHASE-4-FOUNDATION.md` (read at HEAD).

#### 3.5.1 Phase 4-Meta-Core / Composing (covered §3.1 + 3.2)

— Multi-device-key-wrap WIRE, remote-permission-call WIRE, DAK substrate, at-rest K_principal/user-DID-key encryption, device-link QR-flow, biometric unlock, identity-recovery `RecoveryHook`.

#### 3.5.2 v1-beta → v1-GM gap

The independent `ml-dsa`/`ml-kem` audit is the v1-GM gate (NF-2 / C-GM-AUDIT). Multi-device attestation chains will exercise PQ-hybrid sigs through G-CORE-PQ-WIRE before v1-beta ships; classical Ed25519 remains the audited floor (defense-in-depth). MembershipSet over a device-set must accept that its "membership signature" can be hybrid-PQ-sig at codepoint dispatch — the SHAPE must NOT hardcode Ed25519-shaped 32+64 byte fields.

#### 3.5.3 Phase 5 — First Reference Application

Per `docs/FULL-ROADMAP.md` (former Thrum-migration; now post-v1 First Reference Application per CLAUDE.md #15 v1-milestone-gate reframe). No new multi-device-shaped primitives expected; uses what Phase-4 ships.

#### 3.5.4 Phase 6 — Personal AI Assistant MVP

Per `docs/FULL-ROADMAP.md` §"Phase 6": the assistant runs LOCAL on user's Benten instance + calls out to remote LLMs. **Multi-device implication:** the assistant is itself per-device (one assistant per device, all sharing the user's Atrium state via the multi-device-sync substrate). UCAN capability grants for the assistant's authority compose with multi-device attestation (the assistant on device-1 has different cap-envelope-ceiling than on a browser thin-client device-2). Intent declaration + provenance composes with `AttributionFrame.actor_cid` (the Option-A decoupling at PR #155 was already designed for AI-agent attribution — see `sync_replica_explicit_actor_cid_decouples_from_device_cid`). **MembershipSet shape question for the panel:** is the assistant a device, a plugin, both? Per CLAUDE.md #18 it's a plugin; per CLAUDE.md #17 a device hosts it.

#### 3.5.5 Phase 7 — Digital Gardens MVP + Member-mesh replication

Per `docs/FULL-ROADMAP.md` §"Phase 7": "Gardens" = community spaces with admin governance; member-mesh replication (no central server; every member's device participates in replicating the Garden's data). **This is the clearest MembershipSet-shape unification target:** an Atrium with ≥1 user becomes a Garden with admin-configured governance; both ride the SAME sync + capability substrate (per `docs/VISION.md` §"Social Architecture" — "Three tiers of community, all configuration presets on the same underlying sync + governance mechanism"). Multi-device implication: a single user in a Garden brings N devices; the Garden's member-set is composed of (user-DID, device-DID-set) pairs. If MembershipSet is the right primitive, Garden's member-set IS a MembershipSet whose members are themselves MembershipSets (recursive composition).

#### 3.5.6 Phase 7+ — Garden-relays / Bootstrap strategy

Per `docs/FULL-ROADMAP.md` §"Phase 7": Bootstrap for new-member onboarding via Merkle diff + parallel peer serving. Operator-controlled Garden-relays (`BootstrapMode::CustomPeerList` at `benten-sync/src/peer_discovery.rs`) replace the public iroh relay (Compromise #22 metadata-leak posture). Multi-device implication: a Garden-relay is a long-lived full-peer (CLAUDE.md #17 shape (a)) that holds the Garden's encrypted shared state for offline-member catch-up. Its device-DID IS in the Garden's MembershipSet but with a distinct `runs_atrium_peer=true` capability envelope.

#### 3.5.7 Phase 8 — Decentralized plugin discovery + Benten Credits

Per `docs/FULL-ROADMAP.md` §"Phase 8" + CLAUDE.md #18 trajectory-alignment: in Phase 8, plugin discovery is P2P (no central registry). Multi-device implication: a plugin discovered on device-1's network must be re-discoverable / re-installable on device-2 with same content-addressed identity. Per Phase-4-Foundation R1 ratification #2: "per-device-local CURRENT pointer via Loro Map per Ben ratification #2" — the user's library has per-device-distinct active-versions. **This is a MembershipSet-shape question:** is the set of installed plugins a MembershipSet keyed by plugin-CID, with per-device-Loro-Map active-version pointers as projections of that set?

#### 3.5.8 Phase 5+ — Kith decentralized identity & attestation system

Per `docs/future/kith-decentralized-identity.md` (read in full): Kith extends the basic peer-DID + RotationLog with **relational attestations** ("X says Y is Z to me"), **trust-graph traversal**, **per-relationship privacy controls**, **organizational attestations** (Gardens / Groves / schools / certifying bodies), **UCAN-mediated contextual sharing**, and **filterability for proving identity in new contexts**. Couples to Phase-4-Foundation MVP rotation (`SelfRevocation`). **Multi-device implication:** Kith's "person X with N devices" naturally compose into MembershipSet — a person is a MembershipSet of devices, and a Kith attestation about a person attests over THAT MembershipSet. Trust-graph traversal across users-with-devices needs to work uniformly. **This is the strongest long-horizon argument for MembershipSet unification.**

#### 3.5.9 Phase 9+ exploratory — Full Groves / Garden federation / Knowledge attestation marketplace / Benten Runtime / `bentend` peer daemon / P2P compute marketplace

Per `docs/VISION.md` §"Exploratory / Future Scope" + `docs/FULL-ROADMAP.md` §"Exploratory Scope (Phase 9+)":
- **Groves** (governed communities with fractal/polycentric governance + fork-and-compete) — federated MembershipSets with governance metadata.
- **Garden/Grove federation** — cross-community sync; multi-device-shaped because user devices participate in MULTIPLE communities concurrently.
- **Benten Runtime** (WinterTC-compliant edge host) — a deployment shape (b) edge worker is a transient thin-compute device per CLAUDE.md #17; long-horizon multi-device fleet on a user's account.
- **`bentend` peer daemon** + **P2P compute marketplace** — long-lived peer infra; some devices in the user's fleet ARE compute peers, others are clients.

**Architectural-arc implication for MembershipSet:** the v1-beta wire format must preserve the ability to (a) treat a device as a member of multiple membership sets (user-fleet, Atriums, Gardens, future Groves), (b) attach per-member capability envelopes (CLAUDE.md #17 shapes (a)/(b)/(c)), (c) recurse (a Garden's member-set has members that are themselves user-MembershipSets-of-devices), (d) support membership proofs filterable for selective disclosure (Kith).

#### 3.5.10 BUSINESS-PLAN.md positioning

Per `docs/BUSINESS-PLAN.md` §"Atriums" (line 31): "Peer-to-peer direct connections. Partners sharing finances, friends planning a trip, a student syncing with a school. Private, selective, bidirectional sync of chosen subgraphs. Each peer pays only for their own compute/storage." — confirms the multi-device + multi-user unification is product-positioning-baked-in.

---

## §4 Multi-Device UX flow walkthrough (current state + planned)

### 4.1 User installs Benten on device-1 (single-user single-device — exists today)

1. User installs Benten engine binary (full peer per CLAUDE.md #17 shape (a)).
2. Engine opens redb backend (`Engine::open`).
3. `Engine::set_device_cid(Some(blake3(device-name-or-fingerprint)))` populates the local device-CID slot.
4. User mints user-DID via `Keypair::generate()` (OsRng); user-DID = `Did::from_public_key(kp.public_key())`.
5. `Engine::set_actor_cid(Some(actor_cid))` populates the explicit actor identity (Option-A decoupling). On single-device, actor_cid ≈ user-DID hash + device-cid; on Phase-4+ AI-agent runs, actor_cid is the agent's identity.
6. User can WRITE / READ / etc. All writes mint `AttributionFrame { actor_cid, device_did: Some("device-cid:<hex>"), peer_did_set: None, sync_hop_depth: 0 }`.

**At HEAD this works end-to-end; tested via `dogfood_path_c_multi_device_sync_arm` for the content-addressing convergence half + `sync_replica_attribution.rs` for the attribution half.**

### 4.2 User adds device-2 (the dogfood multi-device flow — PARTIALLY at HEAD, full UX in Phase-4-Meta-Composing)

**At HEAD (substrate-only):**

1. On device-2, user installs Benten engine + mints `Keypair::generate()` → device-2-DID.
2. On device-1, user constructs a `DeviceAttestation::issue(parent_kp = user_kp, device_did = device-2-DID, envelope: CapabilityEnvelope { runs_sandbox: ..., holds_zones: Full, online_uptime: AlwaysOn, runs_atrium_peer: true })`. The signature is over DAG-CBOR canonical bytes of `(device_did, parent_did, envelope, nonce, issued_at)` by the user keypair.
3. User out-of-band ships the `DeviceAttestation` + the Atrium config (peer-list / relay-mode) + the seed envelope of any caps to device-2.
4. On device-2: `Engine::open` + `AtriumHandle::set_local_device_attestation(attestation)` + `set_local_device_did(device-2-did)`.
5. Device-2 calls `AtriumHandle::open` with the same Atrium config; iroh peer-discovery finds device-1's full peer.
6. Handshake exchange: `Handshake::initiate(device-2-kp, audience_did = device-1-did, grant, revocation_set)` → device-1 responds → device-2 finalises. The `HandshakeFrame` carries BOTH `peer_did` (user-DID) AND `device_did` (device-2-DID) per net-blocker-4.
7. `Session` is established with mutually-authenticated DIDs + synchronized revocation set sealed.
8. Subsequent SUBSCRIBE / sync messages flow. Each replicated row at device-1 calls `apply_atrium_merge`: `DeviceAttestationEnvelope::verify` checks 4 defenses; emits `E_DEVICE_ATTESTATION_FORGED` on any failure. Per-row cap-recheck fires via `policy.check_write`.
9. Loro CRDT merges; `AttributionFrame` minted per row with `device_did = Some("device-cid:<hex-of-device-2-cid>")` + `peer_did_set = Some({user-DID})` + `sync_hop_depth = 1`.

**The MISSING piece for end-to-end UX (Phase-4-Meta-Composing):**

10. **K_principal multi-device-key-wrap:** for confidential subgraphs (post-#1301 substrate), device-2 needs K_principal to decrypt at-rest data. F-full Layer-D protocol: device-1 encrypts K_principal under an HPKE envelope to device-2's pubkey, attaches to the device-attestation issuance step. Wire SHAPE frozen at V1-FROZEN-INTERFACE.md item 15.5.
11. **QR-code device-link UX:** device-1 displays QR carrying (Atrium config + device-attestation-issuance-payload-with-K_principal-wrap); device-2 scans, mints keypair, sends activation request; device-1 user approves; device-1 ships the attestation + wrap.
12. **Devices sub-panel in admin UI v0** (`docs/ADMIN-UI.md §3.5`): list of user's full peers; per-device last-sync-time + per-plugin sync status; conflict indicator; "This version active on <device X>" per ratification #2.

### 4.3 User revokes device-1 from device-2 (PARTIALLY at HEAD)

**At HEAD:**

1. On device-2, user marks device-1 as revoked via `Engine::revoked_device_dids` set (add device-1-DID).
2. Any active SUBSCRIBE bound to device-1-DID auto-cancels at next delivery per `crypto-major-6`.
3. **Durable propagation:** per Phase-4-Foundation R1 ratification #6 SelfRevocation attestation MVP: an OLD-key-signed rotation attestation supersedes device-1's keypair; propagates via Atrium sync; peers reject content signed by revoked key after timestamp.
4. **`subscribe_subscription_path_terminated_when_device_did_revoked` test is `#[ignore]`** at HEAD — production revocation flow + observable subscription-state termination wired but the pin un-ignores at §2.3 (i) WriteContext threading landing (v1-assessment-window).

**The MISSING piece for stronger device-revocation (Phase-5+ Kith):**

5. **Web-of-trust assisted rotation:** rotations propagate with multi-peer attestation chains; no out-of-band needed for non-compromise rotations (Kith-§2 differentiation table).

### 4.4 User-DID rotation vs device-DID rotation

- **Logical DID survives rotation** (`did_rotate_keypair_preserves_did_under_canonical_bytes` test). The DID string is the long-lived identifier; the underlying keypair rotates.
- **Rotation event** = `RotationAttestation { previous_did, next_did, superseded_at, signature(by OLD key) }`. Signed by OLD key proves authorization. Accepted via `RotationLog::accept_rotation_event` with 3 defenses.
- **Device-DID rotation** uses the same machinery as user-DID rotation (device-DID IS a `did:key` — same type). Triggered when a device's keypair is compromised; user re-attests the new device-keypair via a fresh `DeviceAttestation`.
- **User-DID rotation:** all device-DID attestations issued under the old user-DID are still valid (the attestation signature was by the old user-DID; verifiers consult `RotationLog` to see the old user-DID is superseded but pre-rotation attestations remain valid for their issued-at window). New devices are attested by the new user-DID keypair.

---

## §5 Sync-state-resolution mechanics

### 5.1 Per-property HLC-LWW Loro

- Every property write is a flat string `"<key>\x1f<physical>:<logical>:<node>:<value>"` appended to a Loro List at container `benten:properties`.
- Reads scan the List + group by key + resolve LWW by HLC ordering (`cmp_lex` matches `BentenHlc::Ord`).
- Loro's internal Lamport is INTENTIONALLY NOT used for ordering — HLC carries inside the value (`benten-sync/INTERNALS.md` §3 / `crdt.rs`).
- 10,000-case proptest `prop_loro_converge.rs` exercises N writers / arbitrary interleavings → all writers converge to same LWW value = highest-HLC write.

### 5.2 Device-attribution

- `LoroDoc::winning_attribution()` returns the UNION of every observed write's HLC `node_id` (not just LWW winner). Load-bearing for revocation-vs-data ordering audit (a revoked peer's contributing writes are surfaced, not silently dropped).
- `LoroDoc::all_writes()` surfaces `(property_key, StampedValue)` for engine-side `AttributionFrame` mint per arch-r1-4 D-C HYBRID.
- `AttributionFrame.peer_did_set` = `BTreeSet<String>` of resolved peer-DIDs from the merge.
- `AttributionFrame.device_did` = the originating device-DID (preferred from `DeviceAttestationEnvelope.attestation.device_did` when present; fallback to local `device_cid` per L#1671-1706 in `engine.rs`).

### 5.3 The "D-C HYBRID" version-chain (referenced by L11)

Per `crates/benten-sync/INTERNALS.md` §3 + `arch-r1-4`: at merge time the engine takes the union of `all_writes()` + assembles per-property HLC-LWW resolution while preserving the full set of contributing `(peer, device, hlc)` tuples in `AttributionFrame`. This is the "D-C HYBRID" — D = deterministic content-addressed version Node; C = CRDT-LWW per-property resolution; HYBRID = the resolution path runs both axes (CRDT settles per-property; engine then mints a versioned Version Node tied to the converged property bag's canonical-bytes CID).

### 5.4 Same-key-different-CID at MST level

- `MstDiff::between` surfaces same-key-different-CID to BOTH sides; engine breaks tie via HLC LWW at G16-B.
- `Mst::apply_entries` rehash check rejects entries whose declared CID doesn't match payload bytes (sec-r4r2-1 defense).

### 5.5 Per-device-local CURRENT pointer (Phase-4-Foundation ratification #2)

For plugin/workflow/schema versioning, the active-version CURRENT pointer is per-device-local via Loro Map keyed by device-DID. **Intentional per-device variance is first-class** — different devices can be on different active versions of the same plugin. This is the strongest existing "device-as-distinct-tenant-in-a-MembershipSet" pattern in the codebase.

### 5.6 Revocation-drains-before-data invariant (net-blocker-3)

- Wire-protocol enum `MessageKind { Revocation = 0, Data = 1 }` — discriminants are load-bearing.
- Runtime drainer `MstDiffSession` has tier-stratified queue (revocation_queue + data_queue) with O(1) drain per message in tier-major order.
- Handshake-time revocation-set synchronization gate (`Session.synchronized_revocations`) is a third defense layer.

---

## §6 Device-DID + User-DID relationship

### 6.1 Hierarchical via UCAN / DeviceAttestation

- **User-DID** = trust anchor (per CLAUDE.md #18 four-identity-concepts #4).
- **Device-DID** = attested sub-identity (`DeviceAttestation::parent_did = user-DID`; `DeviceAttestation::device_did = device-keypair-DID`; signed by user-keypair).
- **Plugin-DID** = NOT an attested sub-identity (just a UCAN audience handle; one keypair per install; OsRng-minted; no parent signature).
- **Content-CID** = what a plugin IS.

This is the 4-identity-concepts model (CLAUDE.md #18 implementation refinements 2026-05-11):

```
Content-CID (what the plugin IS)
    │
    └─ Peer-DID signature on original content (provenance; who shared/authored)
        │
        └─ Plugin-DID (UCAN audience handle; minted at install)
            │
            └─ User-DID (trust anchor; signs install records)
                │
                └─ Device-DID (attested sub-identity; physical hardware the user owns)
                    │
                    └─ CapabilityEnvelope (per-shape minimum; runs_sandbox / holds_zones / online_uptime / runs_atrium_peer)
```

### 6.2 Rotation patterns

- **Logical DID survives rotation** (the DID string is the long-lived identifier; underlying keypair rotates). Test pin: `did_rotate_keypair_preserves_did_under_canonical_bytes`.
- **`RotationAttestation` signed by OLD key** proves authorization. `accept_rotation_event` has 3 defenses (authenticity / verbatim-replay / HLC-monotonic-strict).
- **User-DID rotation** vs **device-DID rotation** uses identical machinery (both `did:key`). Difference is in trust-propagation: a user-DID rotation cascades to all devices (each device-DID attestation issued by the old user-DID stays valid for issued-at-window but new devices/cap-grants need new-user-DID signatures); a device-DID rotation only invalidates that one device's attestation.
- **Recovery protocol** is open (Shamir / social / hardware / MLS-style); MVP via SelfRevocation per Phase-4-Foundation R1 #6; Kith threshold-based recovery deferred Phase 5+.

---

## §7 Key-chain pointers (cross-references to M1c scope)

**Brief mandate: don't duplicate M1c work; just name the multi-device cross-references.**

Keys present in the multi-device path:

| Key | Where | Scope |
|---|---|---|
| **User-DID keypair (`Keypair`)** | `benten_id::keypair::Keypair`; one per user; lives in engine's keypair store (currently `secret_bytes_unprotected` exposes it raw at typed-CALL boundary) | M1c — base of identity tree |
| **Device-DID keypair (`Keypair`)** | one per device; per CLAUDE.md #18 there is no separate device-keypair "type" — same `Keypair` primitive; the distinction is in the `DeviceAttestation` envelope binding it to the user-DID | M1a (multi-device path); M1c (derivation) |
| **Plugin-DID keypair (`Keypair`)** | per `PluginDidStore`; OsRng-minted; one per install; NOT derived from user-DID (`plugin_did_install_no_hkdf_from_user_did_grep_assert.rs` asserts) | M1c |
| **K_principal** | per-user (or per-namespace_did) symmetric key material for at-rest encryption; CURRENTLY synthesized via BLAKE3 keyed-hash at G-CORE-3e (`derive_test_seam_key_from_cid_with_namespace`); REAL store is F-full Layer-A | M1c |
| **K(N) per-Node derived keys** | `K(root) = HKDF-SHA256(K_principal, info = "root" \|\| root_cid)`; `K(N) = HKDF-SHA256(K(predecessor), info = "step" \|\| edge_label \|\| N.cid)` per V1-FROZEN-INTERFACE.md item 15.f | M1c |
| **DAK (Device Authentication Key)** | F-full Layer-D; protects local engine vault on each device; not yet at HEAD | M1c (primitive); M1a (multi-device wire) |
| **HPKE ephemeral keys** for encrypt-to-recipient | Per Combined Option F: HPKE-mode-base[MLKEM768-X25519] for Layer-C drop / Layer-D wraps | M1c |
| **Multi-device-key-wrap envelope** | The HPKE envelope that wraps K_principal for handoff to a 2nd device; SHAPE frozen at V1-FROZEN-INTERFACE.md item 15.5 | **M1a (this cataloger) for the SHAPE; M1c for the inner crypto** |
| **iroh TLS keys** | Same Ed25519 keypair as the engine-layer auth (one key signs both UCAN chains AND QUIC TLS); trust-coupling tradeoff documented | M1a/M1c boundary; could split in Phase 9+ |
| **`benten-crypto-suite::SignatureSuite`** + codepoint-dispatched envelopes | All signature verify dispatch threads through here per CLAUDE.md baked-in #5 only-call-site rule; v1-beta default = LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` at `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001` | M1c |

---

## §8 MembershipSet-shape pattern observations

For each multi-device operation, asking: "is this shaped like a MembershipSet operation?"

| Operation | Today's primitive | MembershipSet-shape? |
|---|---|---|
| Add 2nd device | `DeviceAttestation::issue(parent_kp = user_kp, device_did, envelope)` | **YES (clean fit).** "Add member to set where set-owner = user-DID, member = device-DID, member-metadata = `CapabilityEnvelope`". |
| Revoke device | (a) `Engine::revoked_device_dids` in-RAM set add; (b) `SelfRevocation` attestation propagated via Atrium sync (MVP); (c) future Kith web-of-trust | **YES (clean fit).** "Remove member from set"; revocation propagation = membership-state CRDT. |
| Per-device CURRENT pointer (plugin versioning) | Loro Map keyed by device-DID | **YES (projection over MembershipSet).** Per-member projection where projection-value = active-version-CID. |
| Sync handshake (`HandshakeFrame { peer_did, device_did, ... }`) | Type-state builder enforces BOTH-required | **YES.** Membership-proof at the wire: "I claim to be device-DID, attested by peer-DID, here's my chain". |
| `AttributionFrame { device_did, peer_did_set, ... }` | Per-row attribution | **YES.** Per-row provenance over the device MembershipSet. |
| Multi-device-key-wrap (F-full Layer-D) | HPKE envelope to each device-DID pubkey | **YES (clean fit).** "Encrypt-to-MembershipSet" with members = device-DIDs. Combined Option F's `multi-stanza-HPKE for groups` IS the MembershipSet encrypt path. |
| Atrium peer-set (multi-user) | `AtriumConfig` + Session.synchronized_revocations | **YES, but separately scoped — M1b's domain.** |
| Plugin install record + per-plugin-DID UCAN audience | `InstallRecord` + `PluginDidStore` | **NO — distinct concept.** Plugin-DID is NOT an attested sub-identity (CLAUDE.md #18 explicit); the install record is a user-anchored consent envelope. A MembershipSet treatment of "plugins user has installed" would be possible but is intentionally distinct from device-MembershipSet (different trust model). |
| Garden member-set (Phase 7) | NOT YET BUILT | **Recursive MembershipSet target:** a Garden's members are user-MembershipSets-of-devices. The unification panel needs to assess whether v1-beta MembershipSet shape supports this recursion. |
| Kith relational attestation graph (Phase 5+) | NOT YET BUILT | **Distinct concept (Kith-§2 differentiation table):** Kith is a trust-graph over MembershipSets, not itself a MembershipSet. But it operates ON MembershipSets (a Kith attestation is about a user-DID-MembershipSet-of-devices). |

**Where the code uses a different abstraction than MembershipSet would suggest:**

- `Engine::revoked_device_dids: Mutex<HashSet<String>>` is a raw HashSet, NOT a MembershipSet-typed surface. If MembershipSet is canonicalized, this would become a `MembershipSet::removed_members()` projection.
- `AtriumHandle::peer_did_registry: Mutex<BTreeMap<u64, String>>` (node_id → peer-DID) is a flat map, NOT a MembershipSet of peers.
- `AtriumHandle::declared_device_attestations: Mutex<BTreeMap<String, DeclaredDeviceAttestation>>` (device-DID-keyed) — implicitly a MembershipSet projection but not typed as such.
- `Session.synchronized_revocations: Vec<RevocationEntry>` — a Vec-with-dedup-pass; semantically a set-difference operation against the membership.

**Distinct device-vs-plugin-vs-user asymmetry that complicates pure MembershipSet unification (load-bearing):**

- Per CLAUDE.md #18 implementation refinements (2026-05-11) the 4 identity concepts are intentionally distinct: content-CID + peer-DID-signature + plugin-DID + user-DID. Device-DID has the SAME shape as user-DID at the keypair layer (`did:key`) but is bound by `DeviceAttestation::parent_did = user-DID`. Plugin-DID has NO attestation chain (no parent_did binding).
- Per CLAUDE.md #17, devices have a `CapabilityEnvelope` shape (`runs_sandbox / holds_zones / online_uptime / runs_atrium_peer`) tied to deployment shape (a)/(b)/(c). Plugins do NOT have this envelope shape.
- A MembershipSet unification that ignores these asymmetries risks collapsing the 4-identity-concepts model the project just spent Phase-4-Foundation R1 surfacing.

---

## §9 Open questions for downstream specialists

(Don't answer; just flag for M2-M6.)

1. **Does MembershipSet have first-class per-member capability-envelope metadata?** Devices carry `CapabilityEnvelope` (4-dimension); Atrium peers carry UCAN-grant + revocation-set. If MembershipSet is the primitive, both need a uniform metadata slot.
2. **Recursive composition:** can a MembershipSet's members themselves be MembershipSets (Garden ← users ← devices)? Phase 7 / Kith both want this; v1-beta freeze decision is now.
3. **Attestation vs membership-claim distinction.** Today `DeviceAttestation` is the membership-claim primitive (signed by set-owner). If MembershipSet unifies, what's the membership-claim type? `MembershipAttestation` over (set-id, member-id, envelope, signature-by-set-owner)?
4. **Revocation semantics across the unification.** Today user-root UCAN revocation + device-DID-revocation + plugin-uninstall + RotationLog all coexist. If MembershipSet is the canonical surface, do all 4 collapse into "remove member"? The COLLAPSE P0-P3 work already merged some of this (Compromise #23 SUPERSEDED-BY-COLLAPSE) but the surfaces remain distinct at HEAD.
5. **Multi-device-key-wrap envelope SHAPE compatibility.** The SHAPE is frozen at V1-FROZEN-INTERFACE.md item 15.5; if MembershipSet is canonicalized, does the wrap-to-device-set envelope become a wrap-to-MembershipSet envelope? The recovery PROTOCOL (Shamir / social / hardware / MLS-style) stays open at G-COMP-3, so MembershipSet should not over-commit on the protocol.
6. **Per-device-local CURRENT pointer (Loro Map per device-DID)** is the strongest existing per-device-distinct-state pattern. Does MembershipSet expose "per-member-state" as a first-class projection, or does each plugin/feature reinvent the Loro Map?
7. **`StampedValue` per-row originating-grant_cid (Row D-27).** Multi-hop attribution preservation across `apply_atrium_merge` is a known wire-format gap. If MembershipSet carries set-membership-proofs end-to-end, this composes naturally. If not, Row D-27 stays a G-COMP-1 wave.
8. **PQ-hybrid sig agility through G-CORE-PQ-WIRE (Row D-26).** 4 production sites are classical-only Ed25519 at HEAD; sub-fork α/β/γ TBD. MembershipSet's membership-proof signature MUST follow the codepoint-dispatch contract (no hardcoded 32+64 byte fields).
9. **Inv-14 device-grain attribution preservation.** Inv-14 says every cross-Atrium write carries device-grain. Does MembershipSet's wire shape preserve device-grain when an Atrium has members that are themselves MembershipSets-of-devices?
10. **Frozen-interface adjacency.** `WriteContext::namespace_did` (item 11), structural-KDF info-tags (15.f), TwoCidStore (15.g), encryption-class enum (15.e), 4-dimension CapabilityEnvelope, multi-device-key-wrap-envelope-shape (15.5) — all touch MembershipSet. The MembershipSet unification must NOT silently re-open any of these freezes.
11. **CLAUDE.md baked-in #18 four-identity-concepts.** Plugin-DID's intentional non-attestation distinction from device-DID was a load-bearing R1 ratification in Phase-4-Foundation. Does MembershipSet collapse them inadvertently?
12. **Kith trust-graph composition (Phase 5+).** Kith operates ON MembershipSets (attestations over user-MembershipSets-of-devices). The v1-beta wire format needs to support Kith attestations as additive future-codepoints without breaking; assess that the MembershipSet membership-attestation envelope is extensible per CLAUDE.md #5 crypto-agility framework.
13. **Garden / Grove federation (Phase 7-9+).** Polycentric cross-community sync where one user's devices participate in multiple communities. Does MembershipSet support member-multiplicity-across-sets?
14. **Phase 6 AI-assistant attribution.** The Option-A `actor_cid` decoupling was designed for this. If MembershipSet is canonicalized, does the AI assistant become a member of the device's local MembershipSet (the assistant-DID is attested by device-DID? or by user-DID directly?)?
15. **Confidentiality half vs authority half of the Principal primitive.** Per CLAUDE.md #18 implementation refinement (2026-05-18 §8-D framing): a "Principal" has both authority isolation (capabilities; LIVE) and confidentiality isolation (encryption; F-full Layer-A/B/D). If MembershipSet unifies, which half does it canonicalize over? Both?

---

## §10 Self-assessment + confidence + areas needing M1b/M1c integration

### 10.1 Confidence breakdown

| Section | Confidence | Why |
|---|---|---|
| §2 Current state | **High** | Read every cited file in full (benten-id INTERNALS, benten-sync INTERNALS, keypair.rs, lib.rs, handshake.rs head + handshake_wire intro, lib.rs of sync, sync_replica_attribution.rs, subscribe_device_revoke.rs, change_stream.rs, dogfood path c arm, admin_ui_v0_dogfood common). |
| §3.1-3.4 Current planned scope | **High** | CLAUDE.md NIGHT-SHIFT-2026-05-27 LATE-SESSION + V1-FROZEN-INTERFACE.md + V1-FROZEN-INTERFACE-DEFERRED.md (D-26/D-27) + phase-4-backlog.md (selected rows). |
| §3.5 Future-phase + long-horizon | **Medium-high** | Read VISION.md + BUSINESS-PLAN.md + FULL-ROADMAP.md (from git history at `a645ad8b`; these were untracked from main at `db5831ff` so they're not at HEAD) + kith-decentralized-identity.md in full + Phase-3/Phase-4-Foundation history retros. **Caveat:** these documents are 2026-04 originals; some are pre-F-full and may be partially superseded by 2026-05-27 ratifications. |
| §4 UX flow walkthrough | **High** for at-HEAD substrate; **medium** for Phase-4-Meta-Composing UX (QR-flow is named in CLAUDE.md but no protocol spec at HEAD). |
| §5 Sync-state-resolution | **High** | All from benten-sync INTERNALS + crdt.rs + arch-r1-4 references. |
| §6 Device-DID + user-DID relationship | **High** | CLAUDE.md #18 4-identity-concepts is explicit + Phase-4-Foundation retro confirms; rotation pattern is well-tested. |
| §7 Key chain | **Medium — flag M1c** | I named the cross-references but did not catalog M1c's K_principal derivation algorithm / X-Wing combiner / DAK / structural KDF — that's M1c's lane. |
| §8 MembershipSet-shape | **High** for the pattern enumeration; **medium** for the asymmetry surfacing (the 4-identity-concepts split is the load-bearing complication, and M2-M6 specialists may have a cleaner way to reconcile or carve out). |
| §9 Open questions | **High** | Surfaced 15 named questions across the spec's wire-format / freeze / trust-model surfaces. |

### 10.2 Areas needing M1b/M1c integration

**M1b (Atrium-membership-sharing) cross-references I flagged but did NOT cover:**
- The 6 G-CORE-3 ratifications (RATIFIED-S&C-2026-05-21) — covered SubgraphSelector / iroh-blobs / AuthorizationGrant / Path canonicalization / Resolver evaluation / Revocation reach. Multi-device shares these substrates but the cross-Atrium multi-user shape is M1b.
- 3 sendme deployment modes + SubgraphSpec as 4-thing thin core (Roots/Expansion/Inclusion/Termination).
- AuthorizationGrant = ONE signed artifact `{ucan, key_material, binding_sig}`.
- The L9 A1-A5 Atrium-integration items (multi-device-key-wrap is A3+A4 per the brief).
- Atrium creation / multi-user-member-add / leave-rejoin / forkability — `AtriumHandle::open / accept_invite / leave / rejoin / is_active` at `engine_sync.rs` is the surface; the multi-user-add semantics is M1b's lane.

**M1c (key-management) cross-references I flagged but did NOT cover:**
- K_principal derivation algorithm depth.
- DAK substrate (Argon2id + keyring-core + Stronghold).
- Structural KDF (`crates/benten-crypto-suite/src/structural_kdf.rs`) two-path key derivation depth.
- X-Wing combiner ~30-LOC vendored body.
- K(N) per-Node derivation chain in benten-crypto-suite.
- HPKE-RFC-9180 MLKEM768-X25519 envelope construction.
- PQ-hybrid sig combiner details (LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512`).

**Cross-cataloger seams the M-CONS consolidator will need to reconcile:**
- All 3 catalogers will surface "membership-set-shape" patterns; the consolidator picks the canonical primitive.
- The MembershipSet primitive's METADATA SHAPE per member needs to span: capability-envelope (M1a devices) + UCAN-grant (M1b users) + key-material (M1c keys). The freeze decision on this metadata shape is the v1-beta-tag-must-fix.
- Recursive composition (Garden ← users ← devices) is M1b primary but spans M1a.

### 10.3 Process discipline observed

- Tree-state pre-flight done first (per `feedback_reviewer_pre_flight_tree_state`).
- All paths absolute, no `cd` outside worktree.
- Files staged at `.tmp-cataloger/` inside worktree (NOT `/tmp/`) per the brief's isolation-escape ban.
- Documents not at HEAD (CLAUDE.md, VISION, BUSINESS-PLAN, FULL-ROADMAP) were read via `git show <sha>:<path>` for specific historic commits (untracked-but-preserved-in-history pattern); content staged to `.tmp-cataloger/` for grep/read.
- NO --admin-bypass; NO force-push; commit will happen ON this cataloger branch via NORMAL flow per the brief's mandate.

### 10.4 What the M2-M6 specialists need from me

1. The 4-identity-concepts asymmetry is the load-bearing complication for any pure MembershipSet unification — surface it FIRST in M-CONS reconciliation.
2. The v1-beta freeze list (V1-FROZEN-INTERFACE.md item 15.5 + 15.b + 15.f + 15.g + 15.e + 4-dimension CapabilityEnvelope) is what they must NOT silently re-open.
3. The long-horizon arc (Phase 5+ Kith + Phase 7 Gardens + Phase 9+ federation + Phase 6 AI-assistant attribution) all want recursive + per-member-metadata-rich MembershipSets — the v1-beta wire format must NOT foreclose.
4. The 15 open questions in §9 are the right place for the panel's decisions.

---

*End of M1a cataloger output. Total: 10 sections, ~5500 words, drawn from ~20 source files read in full + CLAUDE.md (untracked) + 4 history-only docs.*
