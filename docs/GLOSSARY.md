# Glossary

Terms that have specific meaning in Benten. Alphabetical.

---

**Acceptor** — The Phase-3 freshness + nonce-store + revocation-list seam consumed by `DeviceAttestationEnvelope::verify`. Lives at `benten_id::Acceptor::accept_at`; composes a parent-chain signature check, a configurable replay-window (`FreshnessPolicy`), and a parent-issued nonce store. The Acceptor is one of the three cryptographic defenses against device-attestation forgery (alongside envelope-signature + payload-hash binding). Operator-deployment residual: the engine ships with `FreshnessPolicy::seconds(u64::MAX)` as a test-grade default; production deployments override via `Engine::set_acceptor` with a concrete time-bound — see [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #23.

**actor_cid** — Phase-3 attribution slot decoupled from `device_cid`. Identifies the *logical* actor (the user / Atrium principal) whose authority the write claims, independent of which device emitted the write. Pinned by `sync_replica_explicit_actor_cid_decouples_from_device_cid` (see `crates/benten-sync/tests/sync_replica_attribution.rs`). The decoupling lets multi-device users (laptop ↔ phone-OS-app ↔ desktop) carry consistent actor identity across heterogeneous devices.

**Algorithm B** — The IVM (Incremental View Maintenance) strategy Benten selects for most views: dependency-tracked incremental maintenance. Phase 1 shipped five hand-written views; Phase 2b production-registered Algorithm B with per-view strategy selection (`Strategy::A` / `Strategy::B`) at `Engine::create_user_view` for the 5 canonical view IDs `AlgorithmBView` supports natively. Phase 3 generalized Algorithm B beyond the canonical-view fallback: user-defined view IDs declaring `Strategy::B` now run a generic single-loop kernel (`benten_ivm::algorithm_b::GenericKernel`) keyed on `(label_pattern, projection)`. See [`docs/ARCHITECTURE.md`](ARCHITECTURE.md).

**Anchor (Anchor Node)** — A Node with stable identity that never changes. External edges point to anchors, not to versions. The anchor has a `CURRENT` edge to its latest Version Node. See "Version chain."

**App-level plugin** — The Phase-4 extensibility shape: a content-addressed **subgraph** of operation Nodes (handlers, materializers, SANDBOX nodes) packaged for sharing through Atrium peer groups. Each plugin has its own DID + an attenuated UCAN delegated by the user at install. The engine evaluator walks plugin subgraphs the same way it walks any handler — there is no separate plugin runtime. Trust model = three layers: user-as-root + install-time signed manifest envelope (`requires` + `shares`) + runtime UCAN delegation within manifest envelope. Contrast with "Engine extension." See [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Plugin trust model" + [`ARCHITECTURE.md`](ARCHITECTURE.md) "Plugins and engine extensions."

**Atrium** — The Phase-3 P2P-sync social unit: a per-user (or per-group) trust boundary inside which member devices sync content + capability state through iroh transport + Loro CRDT merge + MST diff. Each Atrium has a member set of DIDs (one per principal) and a device set (multiple devices per principal). The engine surface is `benten_engine::engine_sync::AtriumHandle` (per `crates/benten-engine/src/engine_sync.rs`); the underlying transport + CRDT lives in `benten-sync`. See [`ARCHITECTURE.md`](ARCHITECTURE.md) §benten-sync and [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #22 (public iroh-relay metadata leak) + Compromise #23 (wire device-attestation envelope).

**Attribution** — The Phase-2a Inv-14 contract that every executed `TraceStep` carries an `AttributionFrame` naming the actor (principal CID), handler (registered subgraph CID), and the head-of-chain capability grant CID that authorized the step. Stamped automatically by the DSL on every emitted Operation Node and threaded through the evaluator runtime; opt-out is a Phase-6 affordance, not a Phase-2a one. Missing or malformed frames fire `E_INV_ATTRIBUTION` at registration or runtime. **Phase 3 widened Inv-14** with three additive sync-boundary slots — `peer_did_set` / `device_did` / `sync_hop_depth` — populated at the CRDT merge boundary; see "device-DID," "peer_did_set," "sync_hop_depth," and [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md) §"Inv-14 Phase 3 device-grain extension."

**`benten-dsl-compiler`** — The Phase-2b crate that compiles textual handler-DSL source into a `SubgraphSpec` ready for `Engine::register_subgraph`. Sibling of `benten-engine`; depends only on `benten-core`. Used by `packages/engine-devserver` to route hot-reload edits through the canonical engine path rather than parallel infrastructure.

**BLAKE3** — The cryptographic hash function used for CID derivation. Fast, tree-hash-friendly, multi-threaded.

**CID / CIDv1** — Content Identifier version 1. IPLD standard: version byte + multicodec + multihash. Benten uses CIDv1 with multicodec `0x71` (dag-cbor) and multihash `0x1e` (BLAKE3).

**Code-as-graph** — The paradigm where application logic is represented AS graph structure, not stored IN graph properties. A handler is a subgraph of Operation Nodes connected by control-flow Edges. The engine walks the subgraph to execute it.

**Content-addressed** — A storage model where an item's identity is derived from its bytes. Identical content has identical identity; different content has different identity. Enables cryptographic verification, dedup, and peer sync without schema reconciliation.

**Capability grant chain** — The ordered delegation chain from a root grant to the leaf grant that actually authorizes a write. Phase-2a `GrantBackedPolicy` walks the chain at every refresh point; each link must attenuate (narrow scope, never widen). The head-of-chain grant CID is persisted in the WAIT `ExecutionStateEnvelope` so resume re-checks the chain at the same head it was authorized against. Chain depth is capped (default 64) — exceeding fires `E_CAP_CHAIN_TOO_DEEP`.

**CURRENT pointer** — An Edge from an Anchor Node to its latest Version Node. Atomic update moves the pointer within a storage transaction, giving versioned entities "single latest" semantics while preserving history.

**BentenHlc** — The concrete Phase-3 HLC type implementing `benten_core::hlc::Hlc`. Carries `(physical_secs, logical_counter)` and resolves causal ordering across Atrium peers under bounded skew. Exercised at the sync-replica WRITE per-row cap-recheck boundary inside `apply_atrium_merge`. See "HLC."

**DAG-CBOR** — The IPLD subset of CBOR with canonical (map-keys-sorted, no indefinite-length) encoding. The on-the-wire format for content-addressed Nodes. Implemented via `serde_ipld_dagcbor`.

**device-DID** — The device-grain attribution carrier in the Phase-3 device-heterogeneity contract. A `did:key`-shaped identifier (one per device) bound by parent-issued `DeviceAttestation` to a parent identity. Populated in `AttributionFrame.device_did` for sync-attributed and device-DID-attested writes; lets multi-device users distinguish per-device origins inside a single per-user Atrium. See "DeviceAttestation" + "DeviceAttestationEnvelope."

**DeviceAttestation** — The `benten_id::DeviceAttestation` surface — a parent-issued capability envelope binding a device's public key to the parent identity (a signed certificate of the form "parent attests this `device_did` is mine"). Verified at chain-construction time via `benten_id::Acceptor`; consumed by `DeviceAttestationEnvelope` at sync-time. See "DeviceAttestationEnvelope" and `crates/benten-id/src/device_attestation.rs`.

**DeviceAttestationEnvelope** — The Phase-3 on-the-wire envelope shape carrying signed `payload_hash` + `session_nonce` alongside the embedded `DeviceAttestation`. Defined at `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope`. The V2 wire shape is `(version, attestation, payload_hash, session_nonce, envelope_signature)`; `verify` enforces three cryptographic defenses: (1) envelope-signature against the device's resolved public key (DID forgery defense); (2) `Acceptor::accept_at` (parent-chain signature + freshness window + nonce-store replay defense); (3) constant-time `BLAKE3(payload) == envelope.payload_hash` (frame-pair binding). All three failure modes reject with `E_DEVICE_ATTESTATION_FORGED`. See [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #23 for the full closure narrative.

**Edge** — A typed directional link between two Nodes. Labels include `NEXT` (control flow), `ON_ERROR`, `ON_NOT_FOUND`, `GRANTED_TO`, `CURRENT`, etc.

**Engine extension** — A Rust crate compile-time linked into the engine binary. Distinct from "App-level plugin": rare; for custom IVM strategies, alternate transports (post-iroh — Tor / Nostr / shaped relays), alternate persistence backends (post-redb — sled / fjall / cloud-KV), custom signature schemes (post-Ed25519 — X25519 / BLS / post-quantum), performance-critical primitives that need raw Rust speed beyond SANDBOX. Trust model = "you compiled this into your engine binary" — same trust as Benten core. No UCAN, no manifest envelope, no `read_node_as` boundary; the boundary is `cargo` and code review. See [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Plugin trust model" + [`ARCHITECTURE.md`](ARCHITECTURE.md) "Plugins and engine extensions."

**ExecutionStateEnvelope** — The DAG-CBOR-serialized shape a Phase-2a WAIT primitive produces when suspending. Carries the frame stack, pinned subgraph CIDs, resumption principal, and context bindings needed to resume atomically across process boundaries. Envelope CID is content-addressed for tamper detection.

**FreshnessPolicy** — The operator-configurable replay-window for the `Acceptor`'s nonce store (`benten_id::FreshnessPolicy`). Phase-3 ships with a test-grade `FreshnessPolicy::seconds(u64::MAX)` default (no time-window pruning); production deployments MUST override via `Engine::set_acceptor` with a concrete time-bound (e.g., `FreshnessPolicy::seconds(86_400)` for a 24h replay window) BEFORE participating in adversarial sync. See [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #23 "operator-deployment residual."

**Full peer** — The Phase-3 deployment shape that IS a sync participant: native Rust on user-owned hardware (laptop / phone OS app / desktop / future Benten Runtime instances). Carries durable storage (redb), full Atrium sync participation (iroh + Loro CRDT in `benten-sync`), SANDBOX runtime (wasmtime), persistent UCAN grant store. Contrast with "Thin compute surface."

**Handler** — A registered subgraph that acts as an entry point for external calls. `crud('post')` produces a handler with five actions.

**HLC** — Hybrid Logical Clock. Monotonic timestamps combining physical and logical clocks, used for causal ordering. Relevant in Phase 3 (P2P sync) and in Phase-2a capability wall-clock revocation paths. Implemented directly in `benten-core::hlc`; rationale at the module rustdoc — `uhlc` crate evaluated and rejected for `async-std` + `no_std` + async-return-shape mismatch. See "BentenHlc" for the concrete type used at sync-replica WRITE boundaries.

**Invariant** — A structural or runtime check the engine enforces. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full 14-invariant list and their phase landing.

**iroh** — The P2P networking library (QUIC, dial-by-public-key, NAT traversal with relay fallback) used in Phase 3. Substantively wired in `benten-sync` (`crates/benten-sync/src/transport.rs`); the Phase-3 `ATRIUM_ALPN` is advertised over iroh's QUIC connection establishment. Public-iroh-relay metadata leakage is honestly disclosed at [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #22.

**Loro / Loro CRDT** — The per-property LWW + HLC-ordering CRDT used in `benten-sync` for device-to-device merge. Loro's `LoroDoc` is wrapped in `benten_sync::crdt::InnerLoroDoc`; `winning_attribution` collects the contributing peer DIDs into `peer_did_set` at merge time. Phase 3 wired Loro substantively into the production sync pipeline.

**Manifest envelope** — The bound on an app-level plugin's runtime authority, established at install time when the user consents to the plugin's signed manifest. The envelope is the union of the manifest's `requires` (caps the plugin needs) and `shares` (delegation policy for what other plugins it will hand caps to). Plugins may delegate UCANs to other plugins at runtime *if and only if* the request fits the source plugin's manifest envelope. The CapabilityPolicy backend validates the resulting cap chain at access time. See "App-level plugin," "Plugin manifest," + [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Plugin trust model."

**Module manifest** — The Phase-2b `ModuleManifest` shape (`benten-engine::module_manifest::ModuleManifest`) that declares one or more WASM modules (name + version + content CID + capability requirements + migration steps + optional signature). Installed via `engine.installModule({manifest, manifestCid})`; see [`MODULE-MANIFEST.md`](MODULE-MANIFEST.md) for the schema and [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #19 for the in-memory persistence boundary that lifts in Phase 3. Distinct from "Plugin manifest" — module manifests describe wasm bundles for SANDBOX; plugin manifests describe shareable subgraphs.

**MST / Merkle Search Tree** — The Phase-3 sync diff structure used to locate divergent subtrees between Atrium peers efficiently. Each node carries a CID over its (key, value, child-CIDs) tuple; identical CIDs prove identical subtrees, letting peers exchange only the differing branches. Wired in `benten-sync` alongside Loro CRDT merge; the MST diff yields the candidate write set that the receiver's `apply_atrium_merge` then runs through per-row cap-recheck (per Compromise #2 sync-replica sub-narrative).

**Multiplicative budget** — The Phase-2a Inv-8 cumulative iteration budget. Computed at registration time as the worst-case product of `ITERATE.max` values and non-isolated `CALL` callee bounds along any DAG path through the handler. Caps the worst-case iteration space so nested `ITERATE` + `CALL` combinations cannot trigger combinatorial explosion. `CALL { isolated: true }` resets the cumulative for the callee frame (the callee runs under its own grant's bound). Default registration-time bound: `DEFAULT_INV_8_BUDGET = 500_000`. Exceeding fires `E_INV_ITERATE_BUDGET` at registration; the runtime flat budget (`DEFAULT_ITERATION_BUDGET = 100_000`) remains as a Phase-1 backstop.

**IVM** — Incremental View Maintenance. Benten keeps materialized views up to date via change subscriptions; common reads hit them in O(1).

**`KVBackend`** — The storage trait in `benten-graph` that abstracts over the key-value store. The Phase-1 implementation is redb; a future WASM implementation will fetch content-addressed Nodes from peer storage.

**napi-rs** — The Rust-to-Node.js binding framework. v3 compiles the same codebase to native and WASM targets and auto-generates TypeScript `.d.ts` files.

**`NoAuthBackend`** — The default `benten-caps` policy: allows all writes without capability checks. Ships as the engine's default so embedded / local-only users pay no capability-system overhead.

**Node** — The basic unit of Benten storage. A Node has a label, properties (key-value pairs), and a CID derived from its bytes.

**Operation Node** — A Node representing one of the 12 operation primitives. Operation subgraphs are DAGs of Operation Nodes connected by control-flow Edges.

**Operation subgraph** — A handler represented as a DAG of Operation Nodes. Bounded (max depth, max fan-out, max Nodes, iteration budget). Deterministically evaluable. Content-hashed. Immutable once registered.

**Plugin DID** — The cryptographic identity an app-level plugin runs under. Generated at install time; distinct from the user's DID and from any other plugin's DID. Receives an attenuated UCAN delegated from the user at install and acts as the principal under which the engine evaluator walks the plugin's subgraph. Capability checks fire against the plugin's UCAN chain rather than the user's. See "App-level plugin."

**Plugin manifest** — The signed, install-time-consented declaration that bounds an app-level plugin's authority. Two halves: `requires` (caps the plugin needs to function) and `shares` (delegation policy for what other plugins are allowed to receive from this one). Both are signed by the plugin author so they cannot drift post-install. The user reviews the manifest at install and consents to the *envelope*, not to each runtime access. Schema lands in Phase 4. Distinct from "Module manifest" (which describes wasm bundles for SANDBOX). See "App-level plugin," "Manifest envelope."

**peer_did_set** — The `BTreeSet<Did>` attribution slot on `AttributionFrame`, populated at the Loro CRDT merge boundary. `Some(set)` when the frame originates from a sync merge (captures contributing peer DIDs observed via `benten_sync::crdt::LoroDoc::winning_attribution`); `None` for purely-local writes. The peer-node-id → DID resolution lives in `crates/benten-engine/src/engine_sync.rs` against the local trust store. See "Attribution."

**PublisherRegistry** — The Phase-3 durable manifest-signing publisher key registry per Compromise #21 closure. Maps publisher DIDs to Ed25519 public keys; consulted at module-manifest verification time as the fallback path when the manifest's UCAN proof-chain doesn't already attest the publisher. See [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) Compromise #21.

**redb** — The Phase-1 embedded key-value store: pure Rust, ACID, MVCC (concurrent readers with single writer), crash-safe via copy-on-write B-trees.

**sync_hop_depth** — Inv-14 device-grain extension; bounded merge-hop counter (default cap `SYNC_HOP_DEPTH_CAP = 8`, mirrors Inv-4's sandbox-depth precedent). Increments at each CRDT merge hop; the typed `ErrorCode::SyncHopDepthExceeded` fires at the merge seam when a merge would push the depth past the cap. Defends against unbounded replay across an Atrium peer mesh. See [`INVARIANT-COVERAGE.md`](INVARIANT-COVERAGE.md) §"Inv-14 Phase 3 device-grain extension."

**Thin compute surface** — The Phase-3 wasm32 deployment shape (browser tab + edge worker / WinterTC-compatible runtime) that is NOT a sync participant. Stateless reads against snapshot data; writes go via fetch to a Full peer. Excludes Loro / iroh / SANDBOX / direct sync state from the bundle. IndexedDB persistence (where target supports it) is for snapshot cache + manifest-store, NOT full sync state. Contrast with "Full peer."

**SANDBOX** — The WASM computation escape hatch (landed Phase 2b, wasmtime-backed, fuel-metered, no re-entrancy, default 1 MiB output ceiling per call).

**`serde_ipld_dagcbor`** — The CBOR serialization crate Benten uses. Deterministic by default (sorts map keys); IPLD-native.

**STREAM** — A primitive (landed Phase 2b) producing partial/ongoing output with back-pressure. For Server-Sent Events, WebSocket messages, LLM token streams, progress updates.

**Subgraph** — See "Operation subgraph."

**SUBSCRIBE** — A primitive (landed Phase 2b) providing reactive change notification. The base primitive on which IVM views, sync delta propagation, and event-driven handlers all compose.

**System zone** — The reserved namespace for engine-internal Nodes (capability grants, version-chain metadata, IVM view definitions, subscriber bookkeeping). Labels and node IDs prefixed with `system:` are off-limits to user subgraphs. Phase-2a Inv-11 enforces this at three layers: registration-time literal-CID rejection in `benten-eval::invariants::system_zone`, runtime resolved-label probing in `benten-engine::primitive_host`, and storage-layer defence-in-depth in `benten-graph::redb_backend::guard_system_zone_node`. Reads collapse to `Ok(None)` on the user-visible surface (symmetric with a backend miss); writes fire `E_INV_SYSTEM_ZONE`. System-zone Nodes are only writable through dedicated engine APIs (`engine.grantCapability`, `engine.createView`, …).

**TOCTOU** — Time-of-check-to-time-of-use. The security class where a permission check succeeds but the underlying permission changes before the protected action runs. Phase-2a hardens five TOCTOU points across capability enforcement (commit, CALL entry, ITERATE boundary, WAIT resume, wall-clock revocation ceiling).

**Transaction primitive** — An engine-provided begin/commit/rollback cycle wrapping all WRITEs in a subgraph evaluation. If any WRITE fails, all WRITEs in the transaction roll back atomically.

**Typed-CALL** — The Phase-3 dispatch surface for the 10 engine-known fixed-shape ops (ed25519_sign / ed25519_verify / keypair_generate / keypair_from_seed / blake3_hash / multibase_encode / multibase_decode / did_resolve / ucan_validate_chain / vc_verify). Reserved handler-id namespace `engine:typed:`; closed registry (no user-extensibility — see [`TYPED-CALL.md`](TYPED-CALL.md)). Each op declares required cap + determinism class + input/output shape. Consumed via the standard CALL primitive with `target = engine:typed:<op>` (Rust path goes through `<Engine as benten_eval::PrimitiveHost>::dispatch_typed_call`; `Engine::dispatch_typed_call_public` is the direct-invocation helper) or via `engine.typedCall()` sugar in the napi/TS DSL.

**UCAN** — User-Controlled Authorization Networks. Capability-based auth tokens. Phase 3 ships UCAN as a `benten-caps` policy backend alongside the default `NoAuthBackend` and the existing `GrantBackedPolicy`.

**Version chain** — Benten's opt-in history pattern: Anchor + Version Nodes + `NEXT_VERSION` edges + `CURRENT` pointer. History = traverse. Undo = move `CURRENT`. Sync (Phase 3) = exchange version Nodes. Ephemeral data does not pay versioning cost.

**WAIT** — A primitive (landed Phase 2a) that suspends execution until an external signal arrives or a duration elapses. The engine produces an `ExecutionStateEnvelope` at suspend time; resume runs a 4-step integrity + principal + pin + capability protocol before continuing.

**`wasmtime`** — The WASM runtime for SANDBOX (landed Phase 2b at v43.0.2). Rust-native, Bytecode Alliance, fuel-metered, Component Model support.
