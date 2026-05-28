# Cataloger M1b — Atrium membership + sharing existing state + plans

**Branch:** `phase-4-meta-core/membership-set-cataloger-m1b-atrium-membership-sharing`
**Worktree root:** `/Users/benwork/Documents/benten-engine/.claude/worktrees/agent-aa96098fe572afdcc`
**Tree-state pre-flight:** worktree clean at `2172cb6d` = `origin/main` HEAD. Branched.
**Cataloger role:** M1b of {M1a multi-device-sync, M1b Atrium-membership-sharing = THIS, M1c key-management}. Input for downstream **MembershipSet unification specialist panel** assessing whether `Atrium + multi-device + single-device` collapse to one primitive.
**Inputs ref-pinned at:**
- HEAD `2172cb6d` for `crates/`, `bindings/`, `docs/`
- Branch commits via `git show <sha>:<path>` for the unmerged Phase-4-Meta planning corpus:
  - `1670aa03` — L9 atrium-integration lens
  - `fbdfeb16` — option-f-plus-9-eyes-consolidated-registry
  - `23f76e24` — q3-revisit-option-d-community-lens (K_Atrium-blinded plaintext_cid)
  - `e90900b4` — atrium-policy-credential-validity-design
  - `5f50a028` — Path-A vs Path-B specialist review (CURRENT origin/main HEAD parent)
- HEAD-tracked: `docs/HOW-IT-WORKS.md`, `docs/ARCHITECTURE.md`, `docs/V1-FROZEN-INTERFACE.md`, `docs/V1-FROZEN-INTERFACE-DEFERRED.md`, `docs/SECURITY-POSTURE.md`, `docs/GLOSSARY.md`, `docs/CRATES-DEEP-DIVE.md`, `docs/PLUGIN-MANIFEST.md`, `docs/ADMIN-UI.md`, `docs/future/phase-4-backlog.md`, `docs/future/kith-decentralized-identity.md`, `docs/history/PHASE-{1,2a,2b,3,4-FOUNDATION}.md`
- Code-substantive reads: `crates/benten-engine/src/engine_sync.rs` (atrium core), `bindings/napi/src/atrium.rs` (napi surface), `crates/benten-engine/tests/atrium_{g16_b_e_substantive_e2e, leave_rejoin, lifecycle, handle_last_received_remote_device_did_direct}.rs`, `crates/benten-sync/tests/atrium_{revoke_order, errors, partial_partition, join}.rs`, `crates/benten-sync/tests/host_atrium_publish_view_result_caps.rs`, `crates/benten-caps/INTERNALS.md`, `crates/benten-drop/INTERNALS.md` + `src/lib.rs`, `bindings/napi/tests/cap_device_did_revocation_resolved_scope_regression_guard.rs`, `crates/benten-engine/tests/common/admin_ui_v0_harness.rs`

**Scope discipline.** Atrium membership + sharing only. Multi-device-pairing/handshake-internals → M1a-flagged; K_principal/DAK/X-Wing derivation → M1c-flagged. Where the surfaces intersect (e.g. device-DID attestation envelope), I describe the Atrium-side composition and flag the upstream key-derivation as M1c.

**Provenance honesty.** The brief named `RATIFIED-sharing-and-confidentiality-2026-05-21.md`, `DESIGN-sharing-and-confidentiality-2026-05-21.md`, `SESSION-2026-05-20-to-2026-05-21-substrate-wave-and-willow-pivot.md`, `00-implementation-plan.md`, `NIGHT-SHIFT-2026-05-27.md`, `docs/future/phase-4-backlog.md §S&C`, plus 10 spike READMEs. Of these:
- **`docs/future/phase-4-backlog.md` §3.10 IS at HEAD** + carries the full ratified-S&C scope summary (see §1.3 quote).
- **`RATIFIED-S&C`, `DESIGN-S&C`, `SESSION-substrate-wave`, `00-implementation-plan.md`, `NIGHT-SHIFT-2026-05-27.md`, the 10 spike-READMEs were verified ABSENT from HEAD and from every reachable git commit.** They exist as working-tree-only files in the main repo. Per Inv-15 / cite-drift discipline I quote ONLY: (a) verbatim-quoted prose surviving in `docs/SECURITY-POSTURE.md` Compromise #31 + `docs/V1-FROZEN-INTERFACE.md` §15 (which paraphrase the RATIFIED-S&C text faithfully), (b) the in-tree `crates/benten-drop/INTERNALS.md` (which is the substantive landing of RATIFIED-S&C §R6 / §R3), (c) the Phase-4-backlog §3.10 narrative (which is the authoritative HEAD record of the 6 ratifications + 8 spike-derived refinements), and (d) the AtriumPolicy design doc + L9 lens + 9-eyes registry + Q3 Option D doc reachable via `git show <sha>:<path>` (these are committed but unmerged).

---

## §1 Executive summary

### §1.1 Three-bullet headline

1. **What ships at HEAD.** Atrium is partially-implemented: at the engine API level it's a SINGLE primitive (`AtriumHandle`) that does (i) sync over iroh QUIC + Loro CRDT + MST diff, (ii) device-DID attestation envelope-presentation at handshake, (iii) per-row cap-recheck at `apply_atrium_merge`, (iv) `leave/rejoin` lifecycle (issue #6.12 item 7). It does NOT yet ship: multi-recipient sealing, Drop-bundle authoring, SubgraphSpec-keyed UCAN scope, K_principal/K(N) confidentiality, AtriumPolicy, admin roles, kick/invite primitives. The `benten-drop` crate (14th workspace crate) ships the Drop-bundle CBOR-on-disk format with single-`audience: Did` per bundle.
2. **What's planned (Phase-4-Meta-Core).** The S&C ratification (2026-05-21, per `docs/future/phase-4-backlog.md` §3.10) locks the 7-thing S&C stack: SubgraphSpec primitive + structured UCAN Scope (EXACTLY-2-arm `Hashes | RestrictedSelector`) + AuthorizationGrant ONE-signed-artifact + UCAN-gated iroh-blobs two-CID seam + Drop Format + walker-as-Subgraph fractal pin + revocation-reach acknowledged (Compromise #31 forever-valid Drops). The 9-eyes consolidated registry adds 28 unified amendments (18 v1-beta-LOAD-BEARING) on top, including the L9-A1 multi-recipient stanza, L9-A2 dual-CID plaintext-vs-envelope, L9-A3 recipient-key-generation, L9-A4 K_principal-generation, L9-A5 ExecuteWorkflow scope. AtriumPolicy (per-Atrium-configurable credential-validity) is a Wave-G v1-beta-CODEPOINT-RESERVE + Phase-4-Meta-Composing-impl.
3. **MembershipSet-shape collapsibility — TENTATIVE STRONG-YES.** Of the 9 Atrium operations I cataloged (§9), 7 cleanly fit a MembershipSet shape (add/remove/list/sync/replicate/revoke/fork). Two operations (`register_zone` for sync-scope; `set_envelope_freshness_window` for replay-defense) are orthogonal cross-cutting controls that should NOT be folded into the unification (they exist independently of who's-in-the-set). Concrete flags for downstream specialists in §9.

### §1.2 The 11-paragraph Atrium-state-of-the-world for downstream

**(a) Atrium-as-primitive.** Per `docs/GLOSSARY.md:23`, an Atrium is "The Phase-3 P2P-sync social unit: a per-user (or per-group) trust boundary inside which member devices sync content + capability state through iroh transport + Loro CRDT merge + MST diff. Each Atrium has a member set of DIDs (one per principal) and a device set (multiple devices per principal)." Note the dual-grain: principal-DIDs (logical members) + device-DIDs (per-principal devices). This dual-grain is what M1a + M1b + M1c MUST unify or explicitly NOT-unify.

**(b) Atrium identity at HEAD.** An Atrium is named by `atrium_id: String` at the napi-handle level (`bindings/napi/src/atrium.rs:109`) — but the engine-side `AtriumConfig` carries NO `atrium_id` field (`AtriumConfig { mode }` only; see #1187 closure in napi atrium rustdoc lines 340-347). At HEAD, atrium-identity is JS/TS-side bookkeeping; the engine-side primitive is a per-handle iroh `Endpoint` + per-zone Loro CRDT + trust-store. This is a known-gap (cited in `docs/V1-FROZEN-INTERFACE.md:884` `Atrium + atrium_*` TS-side surface as LOCKED).

**(c) The 12 engine-side public methods of `AtriumHandle`** (`crates/benten-engine/src/engine_sync.rs`):
   1. `open(config) -> AtriumHandle` — bind iroh `Endpoint`; mint a fresh keypair (or use `open_with_keypair(config, keypair)` for caller-supplied).
   2. `register_zone(zone: &str)` — declare a sync-scope namespace.
   3. `sync_subgraph(zone, peer_addr).await` — dialer side of pair-sync.
   4. `accept_sync_subgraph(zone).await -> Connection` — accepter side.
   5. `merge_remote_change(zone, bytes).await` — apply Loro CRDT update bytes.
   6. `merge_remote_change_with_hop_depth(zone, bytes, hop_depth)` — sync-hop-bounded variant (Inv-14 carrier).
   7. `register_peer_did(peer_node_id, did)` / `resolve_peer_dids(set)` — trust-store add + resolve.
   8. `set_local_device_did(Option<String>)` / `local_device_did()` — bind/clear local device-DID.
   9. `set_local_device_keypair(Option<Keypair>)` / `set_local_device_attestation(Option<DeviceAttestation>)` — bind keypair + attestation for outbound device-DID envelope-signing (V2 signed envelope shape).
   10. `register_device_attestation(envelope).await` — record an attestation envelope from a peer (handshake-time presentation).
   11. `set_envelope_freshness_window(window_secs)` — replay-defense tunable (J8 envelope-ceiling under COLLAPSE P3; replaces the deleted `set_acceptor`).
   12. `leave().await` / `rejoin().await` / `is_active()` — non-consuming lifecycle (Phase-3 §6.12 item 7); flips an `is_active` flag but RETAINS the iroh endpoint + trust-store + Loro state.

   **No `kick_member`, `invite_member`, `create_atrium`, `accept_invitation`, `share_subgraph`, `publish_view_result` (Rust)** at HEAD. Trust-roster management is the napi `JsAtrium::trust_peer` / `revoke_peer` in-memory state (described in napi rustdoc as "delegates to engine-side when peer-mgmt API lands"). The `host:atrium:publish_view_result` UCAN cap is named in `crates/benten-sync/tests/host_atrium_publish_view_result_caps.rs` but all 5 test bodies are `#[ignore]`'d pointing to phase-3-backlog §7.3.D — the capability scope IS frozen at v1-beta but its production callers don't yet exist.

**(d) `apply_atrium_merge` is the substantive Atrium-write seam at HEAD** (`crates/benten-engine/src/engine.rs::apply_atrium_merge`). It (i) merges Loro CRDT update bytes into a registered zone's per-zone document, (ii) resolves contributing peer-DIDs against the trust-store, (iii) constructs an `AttributionFrame` with `peer_did_set` + `sync_hop_depth` slots, (iv) mints a new Version Node on the receiver-side anchor chain, (v) fires ChangeEvents to the IVM subscriber, (vi) **per-row cap-recheck against current revocation state** (`sec-r4r1-2 BLOCKER` closure per `docs/SECURITY-POSTURE.md:191`; the structural-always-on shape per Ben's Option-(a) ratification "err on the side of security generally and then open safe QOL paths later"). A revoked row vetoes the whole merge atomically — typed `E_SYNC_REVOKED_DURING_SESSION`. This is the most-substantive Atrium-side defense at HEAD.

**(e) Atrium membership-add is implicit at HEAD.** A new member's DID enters the Atrium's effective set when an existing member calls `register_peer_did(node_id, did)`. There is NO handshake-time consent step; trust is unilaterally-asserted by the registrar. Symmetrically, an Atrium "leaves" by `revoke_peer(did)` at the napi level (in-memory only) — there's no signed cross-Atrium revocation propagation today (the device-mesh exploration brief-edits + `net-blocker-3` + `crypto-major-6` pins at `crates/benten-sync/tests/atrium_revoke_order.rs` are ALL `#[ignore]`'d for "G16-B post-canary residuals → v1-assessment-window"). This is the load-bearing gap M1b surfaces for the MembershipSet panel: **at HEAD, membership-set additions and removals are unilateral local-engine ops, not consensual cross-Atrium operations.**

**(f) The SubgraphSpec primitive is the planned sharing-shape primitive** (per `docs/V1-FROZEN-INTERFACE.md` §15.a + ratified S&C R1). It's a 4-thing thin core: `Roots / Expansion / Inclusion / Termination`. Defined at `crates/benten-core/src/subgraph_spec/spec.rs:190`; walker at `walker.rs:78`; BFS-order enumeration per R4. **The walker IS itself a Subgraph composed of the 12 existing primitives** (`walker_as_subgraph()` at `walker.rs:183`) — CLAUDE.md baked-in #1 12-primitive irreducibility preserved; the engine evaluator already executes SubgraphSpec walks via the universal walk loop. This is the fractal-architecture pin (observation O7 in L9).

**(g) UCAN Scope EXACTLY-two-arm enum** (`docs/V1-FROZEN-INTERFACE.md` §15.c, `crates/benten-caps/src/scope.rs:46`): `Hashes(Vec<Cid>) | RestrictedSelector(RestrictedScope)`. NOT `#[non_exhaustive]` (deliberate carve-out — third arm = HALT-AND-SURFACE-TO-BEN). `RestrictedScope` is the 6-dimension product (roots + edge-allowlist + max_depth + label-allowlist + label-denylist + property-equalities) at `crates/benten-caps/src/restricted_spec.rs:103`. Decidable containment per dimension. The OPAQUE-arm pattern (Spike H+1.1 Path b) was explicitly REJECTED + frozen-as-rejected (NP-hard satisfiability over arbitrary SubgraphSpec; non-portable across SubgraphSpec evolution).

**(h) `AuthorizationGrant` is the ONE signed wire-artifact** (`docs/V1-FROZEN-INTERFACE.md` §15.d, `crates/benten-caps/src/authorization_grant.rs`): `{ucan: UcanEnvelope, key_material: GrantKeyMaterial, binding_sig: Vec<u8>, audience_binding: Cid, issuer_verifying_key: Vec<u8>, audience_pubkey: Option<Vec<u8>>}`. Single-artifact + binding-sig-validation-FIRST ordering. The `binding_sig` covers `(ucan, key_material)` together so neither can be substituted. Consistent across the online ALPN handler (G-CORE-3e) AND the offline Drop bundle (G-CORE-3f).

**(i) `benten-drop` is the 14th workspace crate** (G-CORE-3f, PR #1340 — `crates/benten-drop/INTERNALS.md` + `src/lib.rs`). `DropBundle` is a CBOR-on-disk envelope: `{version, spec_cid, audience: Did, mode: DropContentMode, auth_grant: AuthorizationGrant, content: Vec<EncryptedContent>, envelope_sig: EnvelopeSignature}`. Two cryptographic layers (Spike G): envelope-sig over header (outer) + per-Node AEAD authentication tags (inner) — measured <12% overhead. 3 modes: Mode-1 OnlinePull (G-CORE-3e ALPN), Mode-2 OfflineDrop (this crate, SHIPPED), Mode-3 InlineTiny (deferred). The bundle is **forever-valid once distributed** per Compromise #31 — open architectural trade-off, MITIGATED-not-CLOSED by tight `nbf`/`exp` + key rotation. `DROP_BUNDLE_MAX_SIZE_BYTES = 4096` (4 KiB upper bound on 5-Recipe bundles).

**(j) The recipient-set shape is currently SINGLE: `audience: Did`** in `DropBundle`. The L9-A1 multi-recipient stanza variant (per the planned `EnvelopePayload::HpkeMultiBase`) is named in the L9 review as v1-beta-LOAD-BEARING but is NOT yet codified in §6.2 — the gap WILL surface at first integration test. The `benten-drop/INTERNALS.md` §6 "Multi-recipient bundle" explicitly names it: "current shape carries a single `audience: Did`. Multi-recipient extension (per RATIFIED-S&C future work) is an additive field landing on the next `DropBundleVersion::V2`."

**(k) Ben's 2026-05-27 forkability ratification.** "Atrium-as-forkable not messaging-leave-forgets" — member-leaves-keeps-past-content; future-content-excludes-via-recipient-set. Forks create new K_Atrium (per Q3 Option D §2.7 fork-on-rotate). This composes cleanly with HPKE-mode-base's lack-of-recipient-side-forward-secrecy (RFC 9180 §9.1.4): a member who leaves still has the per-stanza decap-keys for past Drops and can still Open them. The K_Atrium-leak blast radius is bounded by fork-frequency.

### §1.3 Reading order for downstream specialists

5-minute read: §1.2 (the 11-paragraph state of the world).
Full read: §§2-8 work the cataloging surfaces in depth; §9 is the MembershipSet-shape pattern verdict per Atrium-operation; §10 surfaces 12 open questions for M2-M6.

---

## §2 Current-state code inventory (Atrium-related code at HEAD `2172cb6d`)

### §2.1 `crates/benten-engine/src/engine_sync.rs` — the substantive Atrium primitive

**~2000 LOC; 12 public methods (enumerated §1.2(c)).**

`AtriumHandle` is `Arc<AtriumInner>`-clone-shape. The `AtriumInner` carries:
- `transport: TransportEndpoint` — the iroh-backed concrete impl (`IrohTransport` per pre-v1 ratification §15.3 #1; transport-abstraction-boundary in place so post-v1 `TorTransport` / `NostrRelayTransport` / `ShapedRelayTransport` land as engine extensions).
- `peer_keypair: Keypair` — the iroh-endpoint identity keypair.
- `zones: HashMap<String, ZoneState>` where `ZoneState` carries the per-zone Loro CRDT document + HLC tracker.
- `trust_store: HashMap<u64, String>` — peer HLC-node-id → peer-DID mapping.
- `local_device_did: Option<String>` / `local_device_keypair: Option<Keypair>` / `local_device_attestation: Option<DeviceAttestation>` — the V2 signed-device-attestation-envelope slots.
- `declared_device_attestations: Vec<DeclaredDeviceAttestation>` — handshake-time presentation cache.
- `envelope_freshness_window: AtomicU64` — replay-defense tunable.
- `is_active: AtomicBool` — the lifecycle flag (`leave` → false; `rejoin` → true).

The `DeviceAttestationEnvelope` has two wire-shape variants:
- **V1 Unsigned** — legacy; carries `device_did: Option<String>` only.
- **V2 Signed** — carries `device_did + attestation + sig over (version, attestation, payload_hash, session_nonce)`. Signed envelopes require ALL three setters (`set_local_device_did` + `set_local_device_keypair` + `set_local_device_attestation`) bound; envelope-signing uses the bound keypair via `benten-crypto-suite` per the only-call-site rule.

**Trust-store semantics.** `register_peer_did(peer_node_id, did)` writes the (HLC-node-id, peer-DID) pair into the in-memory `trust_store`. `resolve_peer_dids(set)` returns the union resolution: for each node_id in the set, returns the registered DID OR a synthetic `node-id:NNN` fallback. **This is the load-bearing membership-set primitive at HEAD** — `apply_atrium_merge` consults this map to populate `AttributionFrame.peer_did_set`.

**No durable trust-store.** The trust-store is in-memory only; it does NOT persist across engine restarts. This is a known limitation; the v1-beta substantive shape is RAM-only with the expectation that operators re-register at engine boot (acceptable for the single-process-engine v1-beta deployment model).

### §2.2 `bindings/napi/src/atrium.rs` — the napi/TS surface (PR-A AsyncTask migration LANDED)

**~1100 LOC** (R6-FP Wave A Sub-A1/A2 + META #744 PR-A complete).

The napi surface `JsAtrium` is the user-visible API for JS/TS-driven full peers (Tauri / Electron / Node-AI-assistant deployments per CLAUDE.md baked-in #17). Key shape:

- **Factory pattern.** `engine.atrium({config: AtriumConfig})` returns a `JsAtrium` handle. `JsAtrium::create(config)` is the test-only constructor; `JsAtrium::from_engine(config, engine)` is the production constructor.
- **AsyncTask-migrated mutators (9):** `join` / `leave` / `rejoin` / `declare_device_attestation` / `set_local_device_did` / `set_local_device_keypair` / `clear_local_device_keypair` / `set_local_device_attestation` / `clear_local_device_attestation`. Each returns `AsyncTask<...>` so the JS event loop is free during the underlying `block_on(iroh_op)`. Per META #744 PR-A closure (the prior sync `block_on` parked the libuv worker thread for the full iroh round-trip; with default `UV_THREADPOOL_SIZE=4`, four concurrent `await atrium.*()` calls saturated the pool).
- **In-memory trust-roster (NOT yet engine-side-delegated):** `list_peers` / `trust_peer` / `revoke_peer` / `list_declared_device_attestations`. These maintain napi-local `trusted_peers: Vec<String>` + `revoked_peers: Vec<String>` lists. The napi rustdoc explicitly names: "The trust-roster surface is in-memory today; delegates to engine-side when the broader peer-mgmt API lands." **This is a load-bearing gap for the MembershipSet specialist** — the napi trust-roster API exists but the engine-side counterpart does NOT.
- **Process-singleton tokio runtime** (`js_atrium_runtime()` static `OnceLock<Runtime>`) — multi-threaded flavor, named `"benten-js-atrium"`. Required because iroh `Endpoint`'s background tasks must survive across multiple JS-side `await` calls; a per-call runtime would drop them on `block_on` return (G21-T2 fp-mini-review MAJOR-5 closure).
- **`E_ATRIUM_NOT_JOINED` gate** (#688 fix-2) at every engine-bound setter. Surfaces the typed error pre-join rather than silently recording locally.
- **`#1187` production-mode fix.** Pre-#1187, the engine-bound `join()` unconditionally drove `AtriumConfig::for_test()` (= `AtriumMode::Loopback`) for production callers — silent no-relay-no-holepunch transport binding. Closed: engine-bound path now drives `AtriumConfig::production()`.

### §2.3 `crates/benten-engine/tests/atrium_*.rs` — substantive Atrium end-to-end pins

5 files:
- **`atrium_lifecycle.rs`** — happy-path 2-peer + 3-peer loopback convergence (direct `merge_remote_change` calls, NO transport).
- **`atrium_g16_b_e_substantive_e2e.rs`** — 3 pins (3-peer iroh-transport convergence; `apply_atrium_merge` end-to-end on receiver; asymmetric-reachability typed-error surface). The first pin is the load-bearing real-iroh-bytes convergence proof.
- **`atrium_leave_rejoin.rs`** — Phase-3 §6.12 item 7 LANDED. 1 substantive pin (`peer_leave_then_rejoin_reconciles_state_via_loro_merge`) covering: pre-leave apply succeeds; leave flips `is_active` to false; merge-attempt-while-inactive refuses with `InvalidState`; peer-A continues writing during leave window; rejoin re-enables; post-rejoin apply mints a new Version + advances CURRENT + AttributionFrame carries `peer_did_set` + `sync_hop_depth` slots; idempotent double-leave + double-rejoin; trust-store survives leave-rejoin (resolved DID is non-fallback `did:key:peer-a:*`); replay-of-identical-bytes is Inv-13-Row-1-refused (NOT silently REPLACED).
- **`atrium_handle_last_received_remote_device_did_direct.rs`** — per-zone last-received-remote-device-DID observability accessor.
- **`atriums_no_new_primitives.rs`** — the structural pin that Atriums add ZERO new primitive kinds (12-primitive-irreducibility preserved).
- **`r6_r2_batch_a_path_g_capability_grant_cid_substantive_at_apply_atrium_merge.rs`** — capability-grant-CID substantive validation at the merge seam.

### §2.4 `crates/benten-sync/tests/` — Atrium error + revocation + handshake pins

- **`atrium_errors.rs`** — 3 G16-A LANDED pins (typed `E_ATRIUM_RELAY_UNREACHABLE` / `E_ATRIUM_TRANSPORT_DEGRADED`; handshake wire-format carries peer-DID AND device-DID).
- **`atrium_revoke_order.rs`** — 4 G16-B/G16-C/G16-D `#[ignore]`'d RED-PHASE pins for revocation-before-data ordering at offline-reconnect. ALL DEFERRED to phase-3-backlog §7.3.D / §6.12 G16-B post-canary residuals → v1-assessment-window per Wave-E rationale-only sweep. **MembershipSet-specialist red-flag:** the revocation-message-kind-ordered-before-data-at-handshake contract (`net-blocker-3` BLOCKER) is unsatisfied at HEAD; a peer-B that goes offline + comes back online during a peer-A revoke + write window can briefly observe data under stale grant.
- **`atrium_partial_partition.rs`** — partial-partition (asymmetric reachability) error-surface tests.
- **`atrium_join.rs`** — atrium-join handshake pins.
- **`apply_atrium_merge_manifest_envelope_recheck.rs`** — manifest-envelope cap-recheck at merge.
- **`host_atrium_publish_view_result_caps.rs`** — 5 `#[ignore]`'d pins for `host:atrium:publish_view_result` UCAN-cap; all deferred to v1-assessment-window. **The UCAN cap is named + frozen but production callers don't yet exist.**
- **`admin_ui_v0_atrium_share_*.rs`** (3 files at `crates/benten-sync/tests/` + `crates/benten-platform-foundation/tests/`) — `admin_ui_v0_atrium_share_unattested_peer_rejected` + `admin_ui_v0_atrium_share_substitution_with_different_author_rejected` + `admin_ui_v0_atrium_share_bytes_dont_match_announced_cid_rejected`. Defense-in-depth pins for the admin-UI plugin install-via-atrium-share path.
- **`attack_hlc_skew_revocation_ordering.rs`** + **`attack_mst_diff_cid_mismatch.rs`** + **`attack_loro_op_log_inv_13.rs`** — adversarial-peer attack pins for the sync-receive seam.
- **`atrium_revoke_order.rs`** — `mst_diff_preserves_temporal_ordering_of_grants_and_revocations_relative_to_data_writes_under_offline_reconnect` is the apex pin for the interleaved-during-offline-window temporal-ordering preservation (R1 ds-9 carry; HLC-temporal-ordering of {N1, revoke, N2} apply via `into_hlc_ordered_events`). **`#[ignore]`'d at HEAD.**

### §2.5 `crates/benten-caps/` — capability + UCAN substrate

- **`crates/benten-caps/INTERNALS.md`** — 600+ LOC narrative of the capability surface. `GrantBackedPolicy` is the production trust backend; `UCANBackend` extends it with durable UCAN-grant chains over `benten-id`'s claim envelope + chain validation. Per `docs/SECURITY-POSTURE.md`, UCANs attenuate on delegation, propagate revocations, validate `nbf`/`exp` time-windows at chain-walk time; signature comparison is constant-time via `subtle::ConstantTimeEq`. COLLAPSE P2 consolidated the chain-validation + envelope-ceiling seam (per `crates/benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`).
- **`crates/benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`** — the post-COLLAPSE-P2 single chain-validation seam pin. Pre-COLLAPSE: parallel `validate_chain` + `validate_envelope_ceiling` paths. Post-COLLAPSE: ONE `chain_authority::validate` seam ANDing both.
- **`crates/benten-caps/src/authorization_grant.rs::AuthorizationGrant`** — the S&C R3 ONE-signed-artifact envelope (described §1.2(h)).
- **`crates/benten-caps/src/restricted_spec.rs::RestrictedScope`** — the 6-dimension product (described §1.2(g)).
- **`crates/benten-caps/src/scope.rs::Scope`** — the EXACTLY-2-arm enum (described §1.2(g)).
- **`crates/benten-caps/src/chain_authority.rs`** — F3 anti-replay durable replay-marker primitive (with v1-beta in-window TOCTOU race; tracked at V1-FROZEN-DEFERRED Row D-8; closure deferred to G-COMP-1).

### §2.6 `crates/benten-drop/` — the 14th crate, Drop bundle format

Described §1.2(i). The substantive surface:
- `DropBundle::seal(issuer_keypair, spec, audience, mode, auth_grant, content) -> Result<Self, DropBundleError>`
- `DropBundle::open(bytes, recipient_keymaterial) -> Result<UnwrappedBundle, DropBundleError>`
- `DropBundleVersion::V1` (+ `Synthetic` test-only arm; `#[non_exhaustive]`)
- `DropContentMode::OnlinePull | OfflineDrop` (+ `#[non_exhaustive]`; NO `InlineTiny` arm at v1-beta)
- 7 typed errors: `EnvelopeSignatureInvalid` / `VersionUnsupported` / `UnsupportedDropMode` / `BundleTooLarge` / `CborDecode` / `AeadAuthenticationFailed` / `AuthorizationGrantBindingInvalid` — all flow through workspace `benten-errors::ErrorCode`.

### §2.7 `crates/benten-engine/tests/common/admin_ui_v0_{dogfood,harness}.rs` — admin-UI substrate

The admin UI v0 (Phase-4-Foundation) ships as a content-addressed shareable subgraph + signed manifest envelope. The harness exposes:
- `AdminUiV0TestHarness::new()` — composed-engine + materializer end-to-end (G24-B-FP-1 graduation; T1 + T7 pins consume).
- `AdminUiV0TestHarness::new_thin_client_against_full_peer()` — thin-client session-protocol surface (G24-F; T2 pins consume).
- `mint_user_rooted_grant(actor_plugin_did, scope)` — user-rooted UCAN cap mint via `engine.caps().grant_capability`.
- `attempt_cross_plugin_delegation(source_grant_cid, target_plugin_did)` — exercises `Engine::delegate_capability` (T7 private-namespace delegation refusal; `E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN`).

**No admin-UI surface for atrium creation / member-add / member-kick / atrium-fork at HEAD.** The admin-UI ships with 4-category navigation IA (Plugins / Workflows / Content Types / Views) — Atrium-management is NOT a 5th category. The relevant Atrium-UX is named in `docs/future/phase-4-backlog.md` §2.1 path-(c) (multi-device sync UX) + path-(d) (revoke-cap mid-session) — both DEFERRED to wave-9 dogfood gate / Phase-4-Meta-Composing.

### §2.8 `crates/benten-platform-foundation/src/admin_ui_v0/mod.rs` — admin-UI plugin substrate

Per `docs/ARCHITECTURE.md` line 130: "admin UI v0 plugin subgraph + 4-category navigation IA". The 4 categories are Plugins / Workflows / Content Types / Views per `docs/ADMIN-UI.md`. The atrium-share path tests live at `crates/benten-platform-foundation/tests/admin_ui_v0_atrium_share_unattested_peer_rejected.rs` (3-rung defense pins).

### §2.9 `bindings/napi/tests/cap_device_did_revocation_resolved_scope_regression_guard.rs` — substrate-boundary regression guard

A G27-A class-of-bug regression guard pinning the substrate-boundary invariant: device-DID identity rotation lives on `RotationLog`; capability revocation lives on `system:CapabilityRevocation` Nodes; **the two substrates are observably disjoint**. A cap-revoke cycle does NOT alter `RotationLog` state; a device-DID rotation does NOT write a `system:CapabilityRevocation` Node keyed on the device-DID string. **MembershipSet-specialist flag:** at HEAD, the napi `Engine::revoke_capabilities_by_device_did` seam DOES NOT YET SHIP (per the test rustdoc §1). Any future device-DID-keyed-revocation surface MUST route through the substrate-aware boundary.

---

## §3 Planned-state inventory

### §3.1 Phase-4-Meta-Core wire-format-affecting (v1-beta-LOAD-BEARING)

Per `docs/future/phase-4-backlog.md` §3.10 the 6 RATIFIED-S&C decisions are scoped to Phase-4-Meta-Core (the v1-gate substrate; pre-v1-public-interface freeze). The wave decomposition (per §3.10 "Sub-wave decomposition"):

| Wave | Scope | Status at HEAD |
|------|-------|----------------|
| G-CORE-3a | crypto-suite CANARY — mints `KeyMaterial` + `AeadEnvelope` (~500-700 LOC in `benten-crypto-suite`) | SHIPPED (G-CORE-2/3a; `benten-crypto-suite` is 13th crate) |
| G-CORE-3b | `SubgraphSpec` + walker (~800-1000 LOC across `benten-core` + `benten-caps`) | SHIPPED (`crates/benten-core/src/subgraph_spec/{spec,walker}.rs`) |
| G-CORE-3w | walker-as-Subgraph fractal pin (~300-400 LOC in `benten-core`) | SHIPPED (`walker_as_subgraph()`) |
| G-CORE-3d | graph-AEAD layer (~800-1000 LOC; wraps per-Node read/write with `K(N)`) | PARTIALLY SHIPPED — `EncryptedNode` + AEAD-wrap landed; K_principal source = derived from `namespace_did` via BLAKE3 keyed-hash deterministic seam (per the §3.10 footnote "K_principal-per-DID secret-material backend (carry from this wave)") |
| G-CORE-3e | sync + iroh-blobs UCAN-gating (~600-800 LOC) — Flavor B per-request UCAN check; 6-arm validation pipeline | SHIPPED 2026-05-23 |
| G-CORE-3f | Drop Format (~500-600 LOC) | SHIPPED (`benten-drop` 14th crate) |
| G-CORE-3c | full swap matrix conformance (NOT renamed; pre-spike crypto-agility swap-matrix carrier; ~200-300 LOC) | SHIPPED at the swap-matrix wave |

**v1-beta-LOAD-BEARING-but-NOT-yet-shipped:**

- **Per the 9-eyes consolidated registry (`fbdfeb16`) — 18 of 28 unified amendments are LOAD-BEARING for v1-beta-tag** (wire-format-affecting or codepoint-slot-reservation). Below is the Atrium-side subset of those 18:

| U# | Title | Wire-affecting? | Atrium-relevance |
|----|-------|-----------------|------------------|
| **U17 / L9-A1** | Multi-recipient stanza composition rule (`EnvelopePayload::HpkeMultiBase`) | **Y** | LOAD-BEARING — current `DropBundle::audience: Did` is single-recipient; Atriums are intrinsically multi-recipient; the gap WILL surface at first Drop-bundle-to-N-Atrium-peers test |
| **U18 / L9-A2** | Envelope-CID stability under recipient-set evolution; dual-CID (plaintext_cid + envelope_blob_cid) | **Y** | LOAD-BEARING — without dual-CID, recipient-set evolution mutates Drop-CID → breaks iroh-blobs content-addressing-as-identity → breaks forkability semantic |
| **U19 / L9-A3** | Recipient-key-rotation interaction with archival envelopes (`recipient_key_generation: u32` in `BindingContext::DropToRecipient`) | **Y** | LOAD-BEARING for 6-months-offline scenario (Compromise #31 partial closure); recipient retains old HPKE decap-keys under DAK for documented grace window |
| **U20 / L9-A4** | K_principal-rotation propagation to per-Node K(N) addressing (`k_principal_generation: u32` in Layer-B AEAD AAD) | **Y** | LOAD-BEARING — without generation tracking, post-rotation engines cannot read pre-rotation Layer-B ciphertexts; constrains all of Benten's future security-incident-response |
| **U21 / L9-A5** | Ephemeral-execution-grant binding to remote-engine execution context (`PermissionOperation::ExecuteWorkflow {workflow_cid, input_node_cids, max_decrypt_count, result_recipient_pubkey, executor_did}`) | **Y** | LOAD-BEARING for hyper-scaling vision (post-v1-beta but interface freezes now); reserves codepoint slot for "rent compute" semantics |
| **C44 / Q3-Option-D** | K_Atrium-blinded plaintext_cid (`HMAC-SHA256(K_Atrium, BLAKE3(canonical(plaintext)))`) — split `plaintext_cid_local` (un-blinded, never on wire) vs `plaintext_cid_atrium` (blinded, peer-visible) | **Y** | LOAD-BEARING for storage-host equality-oracle defense; K_Atrium random-on-creation per §2.6; **fork-on-rotate** policy per §2.7 (NOT on member-leave) |
| **iroh-gossip codepoint-reserve** | `LAYER_C_ATRIUM_GOSSIP` transport-binding codepoint reserve | **Y** | LOAD-BEARING-as-reserve at v1-beta; impl at G-CORE-ATRIUM-SCALE-1 (post-v1-beta) |

### §3.2 Phase-4-Meta-Composing (UX-coupled; pre-v1-beta-tag)

- **AtriumPolicy `refresh_required` impl** (per `atrium-policy-credential-validity-design.md` D3): wire-format slot at v1-beta-CODEPOINT-RESERVE; impl at Phase-4-Meta-Composing.
- **`accept_atrium_share`** cross-peer install seam — deferred to G-COMP-1 per V1-FROZEN-DEFERRED.md Row D-5.
- **Admin-UI atrium-management 5th-category navigation** (if Ben ratifies; current is 4-category IA).
- **Self-composing admin UI meta-circular full scope** (phase-4-backlog §3.3) — atrium-management plugins editable through the admin UI itself.
- **Decentralized self-discovered registry** (phase-4-backlog §3.1) — Atrium-substrate publish/subscribe; signed + content-addressed manifest discovery; trust-graph extension; admin UI discovery affordance. Registry-trait reconsideration (Fwd-2 #1014 RATIFIED Path A): the paper-only `trait Registry { publish; discover }` was DELETED; only concrete `RegistryEntry`/`DiscoveryQuery`/`DiscoveryResult` data shapes + reserved `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode anchor remain. Phase-4-Meta task: decide registry shape against actual Phase-8 trajectory (CID-keyed announce + `#[non_exhaustive]` DiscoveryQuery + richer error vocabulary + `trait Registry` only-if-second-impl materializes).

### §3.3 Deferred to v1-assessment-window / post-v1

- **`atrium_revoke_order.rs` 4 `#[ignore]`'d pins** — phase-3-backlog §7.3.D STALE-RATIONALE sweep #2 → §6.12 G16-B post-canary residuals (v1-assessment-window).
- **`host_atrium_publish_view_result_caps.rs` 5 `#[ignore]`'d pins** — same destination.
- **Mode-3 InlineTiny Drop bundle** — additive arm per CLAUDE.md additive-codepoint discipline; `DropContentMode` is `#[non_exhaustive]` so adding the variant is non-breaking.
- **Threshold-admin (M-of-N multi-sig admin)** — per AtriumPolicy design D7 + §2.5: single-admin-DID with rotation seam at v1-beta; threshold-admin at Phase-N+1 if user demand emerges.
- **MLS-PQ-derived CGKA / Fork-Resilient CGKA** — Alwen-Hartmann-Kiltz-Mularczyk 2023 substrate. Per Q3 §1.2 conclusion 2: CGKA-deferral STILL HOLDS under Option D (Option D's K_Atrium does NOT require rotation on member-leave; forkability semantic is exactly "past content stays with whoever already has it; future content excludes them via recipient-set exclusion"). What Option D needs is K_Atrium DISTRIBUTION on member-JOIN = multi-stanza HPKE under U17 (already ratified). CGKA value-add (PCS / FS-across-membership-changes) remains structurally mismatched with Benten's forkability — deferred indefinitely.

### §3.4 Cross-cutting plans not Atrium-specific but Atrium-touching

- **Storage-partition seam** (G-CORE-1 / #989) — `WriteContext::namespace_did: Option<Cid>` per-DID storage partition (SHIPPED at Phase-4-Meta-Core; the structural hook the encryption-as-confidentiality substrate (#1301) plugs into). M1c flags as foundational for K_principal-per-DID.
- **Transaction-path namespaced-writes** (G-CORE-1 fix-pass §3.7 #1305) — transaction closures cannot scope writes to a namespace_did; named carry-over, decision deferred to cross-DID multi-write workflows wave.

### §3.5 Future-phase + long-horizon Atrium scope (M1b SCOPE-EXPANSION addendum)

**Why this section exists.** Coordinator request: the MembershipSet panel must assess unification across the full architectural arc, not just v1-beta. If long-horizon Atrium plans assume different membership-set shapes than the unified primitive, surface NOW so v1-beta wire format is correct.

**Phase 5+ (post-v1) — Kith.** Per `docs/future/kith-decentralized-identity.md` (the exploratory scope-stub). Kith is the working name (Ben TBD final name) for the decentralized identity-and-attestation substrate that would supersede the Phase-4-Foundation MVP rotation mechanism (`SelfRevocation` + out-of-band new-key trust). Differentiating features: relational attestations ("X says Y is Z to me" as first-class graph data) + trust-graph traversal + per-relationship privacy controls + organizational attestations (Gardens/Groves, schools, certifying bodies) + UCAN-mediated contextual sharing. **MembershipSet implication:** Kith would introduce a third grain BEYOND user-DID and device-DID — a **relational-attestation-DID** ("X-says-Y-is-Z") that is structurally a UCAN-edge in the graph. If unified, this is a 3-DID-grain shape (principal + device + attestation-edge). If NOT unified, Kith stays orthogonal to MembershipSet.

**Phase 7 — Gardens (community spaces).** Per `docs/SECURITY-POSTURE.md` Compromise #22 ("Gardens-protocol-controlled relays") + GLOSSARY.md "Garden" (not explicitly defined at HEAD but referenced; the social-collective scale-up of Atriums). Gardens introduce **Garden-controlled iroh relay** infrastructure where relay-operator-set is a Garden's quorum of admins rather than n0 / community. The Atrium-config surface would gain `relays: Vec<RelayDescriptor>` where `RelayDescriptor = PublicIroh | GardenRelay {garden_id, relay_did}`. **MembershipSet implication:** Garden = Atrium-of-Atriums shape. A Garden-member-set is structurally a set-of-Atriums (each member-Atrium being a unit member of the Garden). If the MembershipSet primitive is hierarchical / recursable (set-of-sets), Gardens fit cleanly; if flat, Gardens need a separate primitive.

**Phase 7-9 — Groves.** Same scope-area as Gardens; richer social structure (per Ben framing 2026-05-11 evening conversation per phase-4-backlog §3.2). Same MembershipSet implication as Gardens.

**Phase 8 — Decentralized self-discovered registry over Atriums.** Per phase-4-backlog §3.1. Atrium-substrate publish/subscribe; signed + content-addressed manifest discovery; trust-graph extension. **MembershipSet implication:** registry uses the Atrium's member-set as the discovery scope. If the MembershipSet primitive supports trust-graph traversal natively (Kith-style), discovery cleanly composes; otherwise registry needs its own trust-graph layer.

**Phase 6 — Personal AI Assistant MVP** (per `docs/HOW-IT-WORKS.md` line 146). Each AI assistant is a plugin running under the user's principal; plugins compose with Atrium membership through the plugin-DID + manifest envelope. **MembershipSet implication:** **AI-agent-as-Atrium-member is NOT named** at HEAD anywhere I searched. If a future Phase explicitly mints an AI-agent-DID as a first-class Atrium member, that's a fourth DID-grain (user + device + plugin + AI-agent). My read of the planning corpus: AI agents are plugins, NOT first-class Atrium members; the plugin-DID + manifest envelope IS the integration shape. **Flagged as M2-or-M4 specialist surface to confirm.**

**Phase 6+ — Atrium-of-Atriums / cross-Atrium federation.** NOT explicitly named in the planning corpus I read. The closest is the L9 lens §6.5.4 "Atrium-as-Subgraph fractal architecture" (observation O7): "Atrium is a Subgraph; SubgraphSpec is an Atrium-recursable primitive. §6.2 envelope is composition-neutral to this fractal architecture — the envelope encrypts payload bytes regardless of whether the payload is a leaf-Node or a Subgraph-CID-reference-bundle." **MembershipSet implication:** the fractal property suggests Atriums should be MembershipSet-shaped exactly so that Atrium-of-Atriums = set-of-MembershipSets = MembershipSet-of-MembershipSets recursively. **Cleanest unification target.**

**Phase 9+ — Garden-controlled relay infrastructure.** Per SECURITY-POSTURE.md Compromise #22 deferred-to-Phase-7/9 closure. Not Atrium-membership-set-affecting at the primitive level; relay-trust posture only.

**iroh-willow / Willow Protocol — PARKED (Phase-5+ candidate).** Per the in-tree comment-opps + L9 §6.5.3: "Willow's confidential-sync model is explicit that it doesn't address payload-at-rest encryption — it covers protocol-level set-reconciliation under access-control. Benten's chosen path is custom-Atrium-sync (not iroh-docs / not Willow), per prior architectural decisions." Willow IS currently parked + version-incompatible + spec mid-redesign (per `.addl/pq-research/RESEARCH-iroh-willow-assessment-2026-05-20.md`, which I could not read but is referenced from phase-4-backlog). **MembershipSet implication:** if Benten ever adopts Willow as a sync substrate, Willow's "selective payload delivery" (eager-vs-lazy by threshold) is **incompatible with the Compromise #31 forever-valid Drop semantic** (per L9 §6.5.3 sharp finding). Drop bundles MUST be classified as "eager" or supplemented by an out-of-band poll mechanism. Currently moot.

**Long-horizon Atrium-shape implications for the panel.**
1. **Fractal-recursable.** Atriums should be MembershipSet-of-MembershipSets so Atrium-of-Atriums (Gardens/Groves) composes recursively. The L9 fractal-architecture observation supports this.
2. **3-grain DID-hierarchy.** principal-DID (1 per user) → device-DID (N per user) is the v1-beta shape. Kith adds relational-attestation as a 3rd grain. The MembershipSet primitive must EITHER absorb all 3 grains as instances OR explicitly cleave at one of the boundaries.
3. **AI-agent-as-Atrium-member: NOT a planned distinct shape** at HEAD. AI agents are plugins; the plugin-DID surface IS the integration. Confirm with M2/M4.
4. **Forkability as the central semantic.** Whatever the unified primitive looks like, the "Atrium-as-forkable not messaging-leave-forgets" Ben-ratified 2026-05-27 semantic MUST hold. This rules out CGKA-style PCS shapes that would forward-secure-on-member-leave.
5. **Storage-host-equality-oracle defense (K_Atrium)** assumes a single K-blinding-scope per Atrium. If the unified primitive cleaves Atrium into multiple scopes (e.g. per-Garden + per-member-Atrium), the K_DedupScope generalization Ben asked about (Q3 §5 Option I) becomes load-bearing — `K_DedupScope` as a generic parameter in the EncryptedEnvelope codepoint family.

---

## §4 Atrium creation + member-add + leave-rejoin UX walkthrough

### §4.1 At HEAD — current end-to-end UX

**User-A creates a new Atrium.**
1. User-A's full peer runs `engine.atrium({atriumId: "family"})` → returns `JsAtrium` handle.
2. Awaits `atrium.join()` — drives `Engine::open_atrium(AtriumConfig::production()).await` → binds an iroh `Endpoint` with a fresh keypair (or supplied `Keypair`); per-zone Loro CRDT machinery ready.
3. **Wire-format:** new iroh `EndpointId` (= `ed25519_dalek::VerifyingKey` per the 8 spike-derived refinement #7) is published to iroh's discovery. **Key-material:** the iroh-endpoint keypair (Ed25519); no PQ-hybrid at the transport layer yet.
4. **Handshake:** none — User-A has not yet contacted anyone. The `atriumId` is local napi-bookkeeping only.

**User-A invites another user.**
1. **At HEAD: NO `invite_member` API.** The closest substrate is napi `JsAtrium::trust_peer(peer_did)` which records the DID in the in-memory trust-roster. There is NO outbound invite message; the invited user has no way to know they're invited.
2. **Realized today:** out-of-band side-channel — User-A shares the iroh `EndpointAddr` (typically via QR code / share-link). The invited User-B builds their own `JsAtrium` with the same `atriumId`, calls `join()`, then `sync_subgraph(zone, user_a_addr)` to initiate handshake. The handshake exchanges device-DID attestation envelopes (V2 signed shape if all 3 setters bound).
3. **Wire-format:** `HandshakeFrame { peer_did, device_did, peer_id }` (per `crates/benten-sync/tests/atrium_errors.rs`'s `atrium_handshake_wire_format_carries_peer_did_and_device_did` LANDED pin; type-state pattern makes it un-buildable without both DIDs).

**The invited user joins.**
1. User-B's `atrium.join()` binds iroh `Endpoint`.
2. User-B calls `atrium.sync_subgraph(zone, user_a_addr)` — the dialer side. User-A had previously called `atrium.accept_sync_subgraph(zone)` — the accepter side parked on `accept_next`.
3. **Handshake:** `HandshakeFrame` exchange. Both sides verify the other's device-DID via the embedded `DeviceAttestation` (signed by parent-DID per `benten-id::device_attestation::DeviceAttestation`).
4. **Trust-store update:** at HEAD this is MANUAL — each peer calls `register_peer_did(other_peer_hlc_node_id, other_peer_did)`. There is NO automatic trust-bootstrap. **MembershipSet-specialist flag:** this is the largest UX gap at HEAD; the trust-bootstrap MUST be automated for the v1-beta deployment (likely in Phase-4-Meta-Composing).

**Both users sync.**
1. After the handshake completes, both sides drive `sync_subgraph` rounds: MST-diff computes divergent subtrees; Loro CRDT exchanges merge bytes; `apply_atrium_merge` on each side merges + mints Version Nodes + advances anchor CURRENT pointers + fires ChangeEvents.
2. **Per-row cap-recheck** at `apply_atrium_merge` (Compromise #2 sync-replica sub-narrative). A revoked row vetoes the whole merge atomically; typed `E_SYNC_REVOKED_DURING_SESSION`.
3. **HLC-monotonic enforcement** at `crates/benten-sync/src/handshake.rs` — inbound frames with HLC below per-peer max-seen are rejected (`E_HLC_SKEW_EXCEEDED`).

**Eventually one user leaves.**
1. User-B calls `atrium.leave().await` — the non-consuming form (Phase-3 §6.12 item 7). Flips `is_active = false`. Iroh `Endpoint` stays bound; trust-store + Loro state survive.
2. Inbound merges + outbound publish/share/close-share paths return `AtriumError::InvalidState` (mapped to `E_ATRIUM_INACTIVE`) while inactive.
3. **No remote-Atrium notification.** Other peers are unaware User-B has left — they discover it only on next sync attempt (peer-unreachable surface, typed `E_ATRIUM_TRANSPORT_DEGRADED`).
4. User-B can later call `atrium.rejoin().await` to flip back to active. Trust-store + Loro state still intact → Loro CRDT replay reconciles state (assertion (ii) of the `atrium_leave_rejoin.rs` pin).

**One user kicks another (admin-kick).**
**Does NOT exist at HEAD.** The napi `JsAtrium::revoke_peer(peer_did)` adds the DID to an in-memory `revoked_peers: Vec<String>` — this is LOCAL ONLY (the kicker's view stops listing the kicked peer; the kicked peer is NOT informed; no outbound revocation message). There is NO admin-DID concept at HEAD. The 4 `#[ignore]`'d revocation-order pins (`atrium_revoke_order.rs`) describe the planned semantics (revocation-message-kind ordered before data at handshake; device-DID revocation propagates before data; MST diff preserves HLC-temporal ordering of interleaved-during-offline-window grants/revocations) but the substantive impl is deferred.

**Atrium forks.**
**Does NOT exist as a first-class primitive at HEAD.** The closest substrate is the Phase-4-Foundation D-4F-14 ratification (per `docs/GLOSSARY.md` line 67): the anchor + Version Node pattern extended to DAG-shape (anchor → v1 → {v2-mainline, v1.5-fork}; CURRENT can point at any branch tip; per-user-local version history). This is plugin-fork semantics, NOT Atrium-fork semantics. Atrium-fork semantics are ratified 2026-05-27 (Ben framing per AtriumPolicy doc §2.2 `atrium_did` field rationale) but NOT YET CODIFIED in a `Atrium::fork()` API.

### §4.2 Planned end-to-end UX (Phase-4-Meta-Core + Composing)

**User-A creates a new Atrium (planned).**
1. **Wire-format addition:** mint random `K_Atrium = CSPRNG-32-bytes()` per Q3 Option D §2.6 candidate (A); store encrypted-under-K_principal in User-A's local vault (Layer-A AEAD).
2. **Wire-format addition:** mint `AtriumPolicy::permissive_default(atrium_did, founder_pubkey)` per AtriumPolicy design D5 — semantically "no Atrium-level ceiling beyond UCAN scope's own `exp`". The Atrium's `admin_pubkey` is the founder-DID.
3. **AtriumMembership Node** (per the F-full scope review §7.2 referenced in L9): a content-addressed Node carrying `device_encryption_pubkey: HybridKemPubKey` (the X-Wing combiner output of X25519 + ML-KEM-768, per CLAUDE.md baked-in #5 hybrid floor — see M1c). This Node replicates via Atrium sync.

**User-A invites another user (planned).**
1. **Invite primitive:** `Atrium::create_invite(target_did: Did, scope: SubgraphSpec, valid_until: u64) -> InviteEnvelope`. The InviteEnvelope is a signed AuthorizationGrant carrying (a) the K_Atrium wrapped under the target's HPKE encryption pubkey (per Q3 §2.6 distribution model — one-shot multi-stanza HPKE), (b) the SubgraphSpec, (c) the validity window, (d) the inviter's signature.
2. **Out-of-band delivery:** the InviteEnvelope is a sendme-shareable blob (per L9 §2.2). Recipient discovers the invite via QR / link / iroh-gossip pub-sub on a discovery topic.
3. **Recipient acceptance:** recipient verifies the InviteEnvelope signature, decap-wraps K_Atrium, verifies the SubgraphSpec, commits acceptance to their local Atrium join state. The recipient's `Atrium::accept_invite(envelope) -> Result<()>` is the `accept_atrium_share` cross-peer install seam currently DEFERRED to G-COMP-1 (V1-FROZEN-DEFERRED.md Row D-5).

**Both users sync (planned, with K_Atrium-blinded plaintext_cid per Q3 Option D).**
1. Per-Node content is encrypted at Layer-B with `K(N) = HKDF(K_principal, info = "step" || edge_label || N.cid)` — per `docs/V1-FROZEN-INTERFACE.md` §15.f.
2. Drop-bundles for cross-peer sharing wrap K(N) under multi-stanza HPKE (U17 / L9-A1) — one stanza per Atrium-member recipient.
3. The Drop's `plaintext_cid_local` = BLAKE3(canonical(DropBundlePayload)) — un-blinded, never on wire (per L9-A2).
4. The Drop's `plaintext_cid_atrium` = HMAC-SHA256(K_Atrium, plaintext_cid_local) (16-byte truncation OK) — blinded, peer-visible.
5. iroh-blobs addresses by `envelope_blob_cid` = BLAKE3(serialized_EncryptedEnvelope_bytes).
6. iroh-gossip pub-sub on Atrium topic uses `plaintext_cid_atrium` for PlumTree IHAVE/IWANT dedup (per Q3 §3 "iroh-gossip becomes valuable under Option D").

**One user kicks another (planned, partial — admin-DID single-rotation seam per AtriumPolicy D7).**
1. The Atrium's `admin_pubkey` (per AtriumPolicy) holds the admin authority. Admin calls `Atrium::revoke_member(target_did)`.
2. **Outbound:** a signed `system:AtriumRevocation` Node replicates via Atrium sync; downstream peers' trust-stores drop the target.
3. **K_Atrium NOT rotated** on member-leave (per Q3 §2.7 fork-on-rotate; the leaver still holds K_Atrium and can compute past blinded-CIDs — but that's CONSISTENT with "Bob keeps past content").
4. **Future content excludes the leaver** via recipient-set exclusion (the multi-stanza HPKE envelope simply omits a stanza for the leaver).
5. **Threshold-admin (M-of-N multi-sig)** — DEFERRED to Phase-N+1 if user demand emerges (AtriumPolicy §2.5 + §1.4).

**Atrium forks (planned).**
1. Per Q3 §2.7: fork-event = mint a new K_Atrium for the child Atrium (zero CGKA infrastructure; one-shot wrap to the new member set). Parent's K_Atrium is unchanged for ongoing parent activity.
2. Per L9 §4.1 walk-through: pre-fork Drops are still openable by all original recipients (HPKE-mode-base lack-of-forward-secrecy on recipient-pubkey side; RFC 9180 §9.1.4). Post-fork Drops include only fork-A members in recipient_did_set.
3. Per L9 §4.1 subtle interaction with A2 dual-CID: if Alice writes a Node N referencing a pre-fork Drop's plaintext_cid AND post-fork wants the SAME plaintext_cid accessible to fork-A members who weren't in pre-fork, she **reseals** the Drop as a NEW EncryptedEnvelope with new recipient_did_set. The plaintext_cid stays stable; the envelope_blob_cid changes; the mapping plaintext_cid → Vec<envelope_blob_cid> grows by one entry.
4. **Atrium fork-event Node** — content-addressed signed Node carrying `{parent_atrium_did, fork_atrium_did, fork_at_hlc, founding_member_set}`. Replicates via existing Atrium sync mechanism.

---

## §5 SubgraphSpec primitive + UCAN-gated sharing

### §5.1 SubgraphSpec — the 4-thing primitive (FROZEN at v1-beta)

Per `docs/V1-FROZEN-INTERFACE.md` §15.a + `crates/benten-core/src/subgraph_spec/spec.rs:190`. The 4 things:

1. **Roots** — `Vec<Cid>`. The starting Node CIDs from which expansion begins.
2. **Expansion** — `ExpansionRule` declaring which edges to follow from each visited Node. E.g. "follow `NEXT` edges up to depth 5; follow `GRANTED_TO` edges always; never follow `CURRENT` edges."
3. **Inclusion** — `InclusionRule` declaring which expanded Nodes are IN the resulting set. E.g. "include Nodes with label `post`; include Nodes whose `author` property matches `did:key:zAlice...`."
4. **Termination** — `TerminationRule` declaring when expansion stops (per-path or per-spec). E.g. "stop at any Node with label `system:Boundary`; stop at depth > 10; stop on visit-count > 1000."

**Walker.** `walk(spec: &Spec) -> Result<WalkResult, SubgraphSpecError>` at `walker.rs:78`. BFS-order enumeration (per RATIFIED-S&C §R4). `WalkResult.enumerated: Vec<(Cid, StructuralPath)>` carries the canonical BFS-order list of (CID, structural-path) pairs.

**Fractal pin.** `walker_as_subgraph() -> Subgraph` at `walker.rs:183` — the walker IS a Subgraph composed of the existing 12 primitives. CLAUDE.md baked-in #1 12-primitive irreducibility PRESERVED. A proposed 13th primitive for SubgraphSpec = HARD-HALT.

**Combinators.** `intersect(a, b)` / `union(a, b)` / `filter(...)` at `combinators.rs:{37, 99, 140}`. Engine operation on the SubgraphSpecs themselves (per the "Structured UCAN scope" tentative-scope bullet in phase-4-backlog §3.10).

### §5.2 UCAN scope binding to SubgraphSpec.cid (FROZEN at v1-beta)

`crates/benten-caps/src/scope.rs:46` `pub enum Scope` — EXACTLY TWO ARMS:
- `Hashes(Vec<Cid>)` — direct CID-list scope (legacy / simple grants).
- `RestrictedSelector(RestrictedScope)` — the 6-dimension product scope (planned long-form share-shape scopes).

The serde-tag dispatch is additive-friendly (a future arm lands as a new serde-tag arm) but the Rust enum itself is NOT `#[non_exhaustive]` — a third Rust arm requires HALT-AND-SURFACE-TO-BEN per `docs/V1-FROZEN-INTERFACE.md` §15.c.

**SubgraphSpec-keyed scope.** The intended shape (per phase-4-backlog §3.10 tentative-scope and the L9-A5 ExecuteWorkflow extension): UCAN `Scope::RestrictedSelector(RestrictedScope)` where the RestrictedScope's roots-dimension carries a SubgraphSpec.cid. The recipient walks the SubgraphSpec at access-time (per RATIFIED-S&C §R5 live-per-request; G-CORE-9 §15.j frozen contract).

### §5.3 AuthorizationGrant — the ONE signed wire-artifact (FROZEN at v1-beta)

Per `docs/V1-FROZEN-INTERFACE.md` §15.d. Single struct carrying `{ucan, key_material, binding_sig, audience_binding, issuer_verifying_key, audience_pubkey}`. Validation order is **binding_sig FIRST, then UCAN scope, then key material** — the binding is the foundation. Per RATIFIED-S&C §R3 quoted explicit.

`KeyMaterial` rename collision-resolved post-G-CORE-9 row 7: `benten_caps::KeyMaterial` → `GrantKeyMaterial` (grant-bearing handle); `benten_crypto_suite::aead::KeyMaterial` → `AeadKeyMaterial` (AEAD-bearing key). Distinct semantics; same prior name. **Tentatively-decided night-shift; rebuttable at morning Ben review.**

### §5.4 Capability chain across Atrium members

The capability chain is the existing UCAN delegation chain (per `crates/benten-caps/INTERNALS.md` + `crates/benten-id`'s claim envelope + chain validation). When Alice grants Bob a UCAN, then Bob delegates to Carol, the chain is `Alice's-root → Bob's-attenuation → Carol's-attenuation`. Each link is signature-validated; attenuation is per-scope (Carol's effective scope ⊆ Bob's effective scope ⊆ Alice's scope).

**Atrium-side composition** (planned, per RATIFIED-S&C §R3 + L9 §3.2):
- Drop-bundle UCAN is **content-grant** ("you, recipient, can access subgraph X, action Y, until time Z"). Sits in DropBundlePayload plaintext. Signed by sender's user-DID.
- `BindingContext::RemotePermission` AAD-bound BindingContext is **execution-grant** ("approving device A authorizes requesting device B to perform operation X on behalf of user-DID"). Sits in §6.2 envelope AAD.
- These are different layers; L9 §3.2 names this distinction as a MEDIUM observation and recommends documentation (NOT a §6.2 amendment) so future maintainers don't conflate them.

**Per-row cap-recheck at `apply_atrium_merge`** (substrate at HEAD): the receiver's `apply_atrium_merge` consults the CURRENT cap-policy state on EVERY incoming row, not just at handshake. A revoked grant (even if Loro CRDT merge would have applied a previously-granted write) is denied at the merge seam. This is the substrate Compromise #2 sync-replica sub-narrative + the G16-B-F structural-always-on per-row cap-recheck PR #161 close.

---

## §6 Drop bundles + Atrium distribution

### §6.1 Drop bundle wire format (SHIPPED at v1-beta)

Per `crates/benten-drop/src/lib.rs` + `bundle.rs`. The on-disk CBOR structure:

```rust
pub struct DropBundle {
    pub version: DropBundleVersion,     // V1 + Synthetic test-only arm; #[non_exhaustive]
    pub spec: RestrictedScopeSpec,      // the SubgraphSpec snapshot
    pub audience: Did,                  // SINGLE recipient at v1-beta
    pub mode: DropContentMode,          // OnlinePull | OfflineDrop; #[non_exhaustive]; NO InlineTiny
    pub auth_grant: AuthorizationGrant, // {ucan, key_material, binding_sig}
    pub content: Vec<EncryptedContent>, // per-Node AEAD ciphertexts (= benten_graph::aead_wrap::EncryptedNode)
    pub envelope_sig: EnvelopeSignature,// Ed25519 sig over the bundle header
}
```

**Defense-in-depth (Spike G):** envelope-sig over `(version, spec_cid, audience, mode, auth_grant, content_root_hash, key_material_hash)` + per-Node AEAD authentication tags. Tampers in `content[i]` ciphertexts are caught at AEAD layer even if envelope-sig still verifies. <12% overhead measured.

**Construction + parse:** `DropBundle::seal(issuer_keypair, spec, audience, mode, auth_grant, content)` + `DropBundle::open(bytes, recipient_keymaterial)`.

**Size cap:** `DROP_BUNDLE_MAX_SIZE_BYTES = 4096` (Spike G measurement ~2688 bytes for 5-Recipe bundle).

**3 modes (per phase-4-backlog §3.10 8 spike-derived refinement #6):**
1. **Mode 1 — OnlinePull** — recipient pulls from a live G-CORE-3e ALPN serve endpoint. The bundle wire-shape mode discriminator names this for protocol negotiation.
2. **Mode 2 — OfflineDrop** — this crate's substantive shape. Consumer reads filesystem-only.
3. **Mode 3 — InlineTiny** (≤16 KiB inlined into share URL) — DEFERRED post-v1; attempt to construct/parse fires typed `DropBundleError::UnsupportedDropMode`.

**Revocation reach (R6 reality, Compromise #31, OPEN at v1-beta + v1-GM):** Drop bundles are **forever-valid once distributed**: UCAN revocation cuts FUTURE serves on online (G-CORE-3e) ALPN path, but already-derived keys remain decryptable. Mitigation: tight `nbf`/`exp` + periodic key rotation. Documented at `docs/SECURITY-POSTURE.md` "Revocation reach"; cross-referenced by `tf3f_revocation_reach_forever_valid_documented` pins.

### §6.2 sendme + iroh-blobs distribution

**sendme tickets** (`BlobTicket` per iroh-blobs docs): "package the file's BLAKE3 hash and our endpoint's `EndpointId` into a single copy-able string." Transport-layer identifier; encryption-agnostic.

**iroh-blobs two-CID seam** (R2 ratification): per `docs/V1-FROZEN-INTERFACE.md` §15.g. `TwoCidStore` adapter wraps ciphertext-bytes backing store + the `plaintext_cid → ciphertext_cid` mapping. UCAN scopes against `plaintext_cid`; iroh-blobs serves ciphertext-blob by its own hash (preserves "served-bytes-hash == requested-hash" invariant). Per-chunk AEAD at chunks of `IROH_BLOCK_SIZE = 16 KiB` for Nodes ≥64 KiB; whole-content AEAD below threshold. AAD binds chunk-index for replay defense.

**G-CORE-3e ALPN handler (online-pull):** wave-3e Flavor B per-request UCAN check; 6-arm validation pipeline at `crates/benten-sync/src/ucan_blobs_protocol.rs::validate_request_for_connection`. SHIPPED 2026-05-23.

### §6.3 Drop bundle composition with Atrium membership (PLANNED, per L9 §3.1)

L9 recommends flat composition: ONE outer `EncryptedEnvelope` with `HpkeMultiBase` payload (per A1), containing a CBOR-serialized DropBundlePayload as the plaintext — NOT nested envelopes.

Walk-through:
- A Drop bundle conceptually carries: (a) one or more Node CIDs + their AEAD-encrypted bodies, (b) UCAN scope assertions, (c) SubgraphSpec CIDs the recipient is granted access to, (d) sender attestation.
- The composition: serialize `(a)+(b)+(c)+(d)` into a DropBundlePayload CBOR structure; this is the *plaintext* the multi-stanza A1 envelope seals.
- Each Node body is *already* encrypted at Layer-B with `K(N) = KDF(K_principal, N.cid)` (per `docs/V1-FROZEN-INTERFACE.md` §15.f). The Drop bundle includes the **per-Node K(N) keys** wrapped under the multi-stanza CEK so recipients can derive K(N) and AEAD-Open the Layer-B ciphertexts.

**Nested envelopes rejected:** would double HPKE work (2N encaps for N recipients) + create outer envelope whose plaintext is itself a list of ciphertexts (information-theoretic redundancy) + require defining envelope-of-envelopes semantics in §6.2. Flat composition is strictly cheaper + cleaner.

### §6.4 SubgraphSpec.cid propagation through Drop bundle (PLANNED, per L9 §3.4)

SubgraphSpec.cid survives Drop bundle propagation cleanly because it's content-addressed + immutable. BUT the recipient must be able to RESOLVE SubgraphSpec.cid to a SubgraphSpec the recipient can interpret. L9 §3.4 enumerates 3 candidate composition decisions:
- (a) Alice includes the SubgraphSpec Node + its K(N) in the Drop bundle plaintext (per-Node-K-wrap pattern).
- (b) SubgraphSpec is published as Layer-C-encrypted to Bob separately.
- (c) SubgraphSpec definitions live in a globally-readable Atrium subgraph; only the contents they reference are encrypted.

L9 recommends NAMING this as a Phase-4-Meta-Composing scope item so the S&C wave decides; §6.2 envelope composes cleanly with any of the three.

### §6.5 100-Drop-bundle catch-up after 6-months-offline (PLANNED, per L9 §5.1)

User-A offline 6 months; User-B sent 100 Drop bundles. User-A returns; Atrium sync delivers 100 envelopes (with help from A1 multi-stanza if multi-recipient).

User-A's engine MUST: (a) decode + Open each envelope (independent operations; parallelizable); (b) extract the inner DropBundlePayload; (c) apply each Drop to the local CRDT.

**Amendment 5 stale-rejection variant-specific scoping (CONFIRMED correct):** Vault and DropToRecipient are out-of-scope for time-bound binding (vault is at-rest by design; drops are intentionally long-lived per Compromise #31). If a Drop's outer Layer-C envelope had `valid_until` of T_send + 1 day, Open at T_send + 6 months would otherwise reject — variant-scoping prevents this.

**CRDT conflict resolution:** multiple Drops targeting same Node-CID = standard CRDT merge case (LWW via vector clock or version-vector). §6.2 envelope is transparent. NEW concern (L9 §5.1): UCAN scope semantics make conflicting authorization claims additive (Bob holds both grants; effective access is the union). The CRDT layer must respect UCAN union-of-grants semantics. Flagged for downstream UCAN+CRDT-interaction wave.

---

## §7 Forkability semantics

### §7.1 Ben's 2026-05-27 ratification (per L9 §4.1 + Q3 §2.7 + AtriumPolicy §2.2 + this brief)

**"Atrium-as-forkable not messaging-leave-forgets."** Specifically:
- **Member-leaves-keeps-past-content** — when Bob leaves Alice's Atrium, Bob retains his stanza-decap-keys for ALL pre-leave Drops. Bob can still Open them post-leave (HPKE-mode-base lack-of-recipient-side-forward-secrecy; RFC 9180 §9.1.4 explicit).
- **Future-content-excludes-via-recipient-set** — post-leave Drops simply omit Bob's stanza from the multi-stanza HPKE envelope. Bob's parser attempts to find his stanza in the stanzas list, fails to find his recipient_did, returns "not addressed to me."
- **Forks create new K_Atrium** (per Q3 §2.7 fork-on-rotate; AtriumPolicy §2.2 `atrium_did` per-fork). Parent K_Atrium unchanged; child K_Atrium randomly minted at fork-event time; distributed to child member set via multi-stanza HPKE one-shot wrap.

### §7.2 Composition with capability delegation + revocation

Per L9 §4.1 walk-through:
- **Pre-fork content cap-grants** remain VALID for the leaver (Bob still has the UCAN; the cap-recheck against the CURRENT cap-policy on User-A's engine doesn't matter for Bob because Bob is querying his OWN engine post-leave). This is the inherent revocation-reach limit (Compromise #31).
- **Post-fork content cap-grants** are minted under a new chain whose root is the fork's founding-member-DID-set. Bob is structurally excluded.
- **Mid-leave-and-rejoin window data** — per the `atrium_leave_rejoin.rs` substantive pin: trust-store survives leave-rejoin; Loro CRDT replay reconciles state; post-rejoin merge mints a new Version Node with `peer_did_set` + `sync_hop_depth` slots; AttributionFrame continuity preserved.

### §7.3 K_principal-rotation interaction with forks

Per L9-A4 (U20): K_principal-rotation is INDEPENDENT of fork-events. A fork-event mints a new K_Atrium (per Q3 §2.7); K_principal stays the same. A K_principal-rotation event (security-driven; security-incident-response) mints a new K_principal generation; ALL of User-A's per-Node K(N) values recompute for FUTURE writes; historical Layer-B ciphertexts remain decryptable via the K_principal-generation map under DAK (per A4's `k_principal_generation: u32` in Layer-B AEAD AAD).

**MembershipSet-specialist composition observation:** if fork + K_principal-rotation are independent operations, the unified primitive must support BOTH grains independently. A unified "rotation event" that conflates them would lose the orthogonality.

### §7.4 The forkability semantic is STRICTLY INCOMPATIBLE with CGKA's PCS

Per Q3 §1.2 conclusion 2: CGKA's value-add (PCS / FS-across-membership-changes) is structurally mismatched with Benten's forkability. CGKA-style PCS would FORWARD-SECURE the K_Atrium on member-leave — which would mutate past Drop CIDs for the leaver. Ben's ratified semantic preserves the leaver's access; CGKA would NOT. CGKA-deferral STILL HOLDS under Q3 Option D.

---

## §8 Admin roles + admin actions

### §8.1 At HEAD — NO admin role

- No `admin_pubkey` field on any Atrium-related Node.
- No `system:AtriumAdmin` label or principal.
- No `Atrium::is_admin(did)` predicate.
- No admin-only-permitted action.
- The closest substrate is the existing User-DID-as-root model: every cap chain traces back to a user-issued root (per the plugin three-layer trust model + capability backend's chain validation).

### §8.2 Planned at v1-beta — single-admin-DID with rotation seam (per AtriumPolicy D7)

Per `atrium-policy-credential-validity-design.md` §1.4 + §2.5 + D7:
- **`admin_pubkey: HybridSigPubKey`** field on `AtriumPolicyPayload`. Inv-17 hybrid floor (X-Wing-style Ed25519 + ML-DSA-65 hybrid; see M1c).
- **`policy_version: u32`** monotonic ordering of policy updates; defense against admin-replay. Updates by admin MUST strictly increment.
- **Admin-rotation seam:** mint a NEW AtriumPolicy with new `admin_pubkey`, signed by OLD admin (rotation continuity); or by emergency-recovery key (out-of-scope at v1-beta — Compromise #45 candidate per AtriumPolicy §4.7).
- **What admin actions exist (planned):**
  - **Policy-update** (`AtriumPolicy` `policy_version` increment).
  - **Member-add** — sign an `InviteEnvelope` (carries K_Atrium wrapped under invitee's HPKE pubkey + SubgraphSpec + validity window).
  - **Member-kick** — sign a `system:AtriumRevocation` Node identifying the kicked-DID; replicates via Atrium sync; downstream peers' trust-stores drop the target.
  - **Revocation** — sign a `system:CapabilityRevocation` Node identifying the revoked grant CID; per Compromise #2 sync-replica per-row cap-recheck infrastructure.
  - **Fork-event** — sign an `AtriumFork` Node carrying `{parent_atrium_did, fork_atrium_did, fork_at_hlc, founding_member_set}`.

### §8.3 Threshold-admin (M-of-N multi-sig admin) — DEFERRED to Phase-N+1

Per AtriumPolicy §1.4 + §2.5: single-admin-DID at v1-beta; threshold-admin at Phase-N+1 if user demand emerges. **MembershipSet-specialist composition observation:** threshold-admin would introduce a third notion of "membership" (the admin-quorum-set, separate from the Atrium-member-set). If the unified primitive supports "sets of sets," this fits cleanly; if flat, it needs an orthogonal primitive.

### §8.4 No admin-only-permitted action at v1-beta — admin authority is signed-record-vs-not

The crucial design point per AtriumPolicy + per Ben's ratified forkability: **admin authority is the authority to mint signed Atrium-policy + signed-Atrium-revocation Nodes.** All non-admin members can still write content + grant caps within their scope; admin authority is specifically about the Atrium's policy + membership state. This is consistent with the existing User-DID-as-root cap chain model.

---

## §9 MembershipSet-shape pattern observations

For each Atrium operation at HEAD + planned, I tag with MembershipSet-fit:
- ✅ = cleanly fits a MembershipSet primitive operation.
- ⚠️ = fits with caveats (composition gap or orthogonal cross-cutting control).
- ❌ = does NOT fit MembershipSet shape (different primitive needed).

| # | Operation | At-HEAD substrate | Planned shape | MembershipSet-fit |
|---|-----------|-------------------|---------------|-------------------|
| 1 | **Atrium creation** | `AtriumHandle::open(config)` binds iroh `Endpoint` + mints trust-store | + K_Atrium random-mint + AtriumPolicy permissive-default + AtriumMembership Node | ✅ — `MembershipSet::create(creator_id, initial_set, scope)` |
| 2 | **Member-add (trust)** | `register_peer_did(node_id, did)` writes to in-memory trust-store | + InviteEnvelope (signed AuthorizationGrant carrying K_Atrium wrap) + accept_atrium_share | ✅ — `MembershipSet::add(set, member_id, evidence: InviteEnvelope)` |
| 3 | **Member-leave (self-leave)** | `leave().await` flips `is_active` flag | Same; plus optional outbound `AtriumLeave` Node | ✅ — `MembershipSet::leave(set, member_id)` |
| 4 | **Member-rejoin (self-rejoin)** | `rejoin().await` flips `is_active` flag back | Same | ✅ — `MembershipSet::rejoin(set, member_id)` |
| 5 | **Member-kick (admin-revoke)** | NOT-EXISTS (napi `revoke_peer` is local-only) | Admin-signed `system:AtriumRevocation` Node | ✅ — `MembershipSet::remove(set, member_id, admin_sig)` |
| 6 | **List members** | `list_peers()` returns trust-roster minus revoked | + signed-Node replication; durable reads | ✅ — `MembershipSet::list(set) -> Vec<Member>` |
| 7 | **Sync membership state across peers** | Loro CRDT zone-sync on the trust-store (PLANNED; not at HEAD) | AtriumMembership Node replicates via sync | ✅ — `MembershipSet::sync(set, peer_addr)` |
| 8 | **Fork** | NOT-EXISTS | `Atrium::fork(parent, founding_member_set)` mints new K_Atrium + new AtriumPolicy | ✅ — `MembershipSet::fork(parent_set, founding_member_set) -> child_set` |
| 9 | **Revoke specific capability (within Atrium)** | `revoke_capability_by_grant_cid` mints `system:CapabilityRevocation` Node | Same; replicates via Atrium sync; per-row cap-recheck at `apply_atrium_merge` | ⚠️ — capability-revocation is a SEPARATE primitive from membership-set; the two compose at the cap-chain layer |
| 10 | **Register zone (sync scope)** | `register_zone(zone: &str)` declares a sync-scope namespace | Same | ❌ — orthogonal cross-cutting control; sync scope is NOT membership |
| 11 | **Set envelope freshness window** | `set_envelope_freshness_window(secs)` tunes replay-defense | Same | ❌ — orthogonal cross-cutting control; freshness is NOT membership |
| 12 | **Set local device-DID / keypair / attestation** | 6 setters at engine-side + napi | Same | ⚠️ — device-DID grain is DISTINCT from member-DID grain; either MembershipSet absorbs both grains as instances OR explicitly cleaves at device-vs-principal boundary; see M1a |

### §9.1 The collapsibility verdict

**Of 12 Atrium operations:** 8 fit cleanly (creation, member-add, leave, rejoin, member-kick, list, sync, fork), 2 fit with caveats (revoke-cap, device-DID setters), 2 do NOT fit (register-zone, set-freshness-window).

**TENTATIVE STRONG-YES on MembershipSet collapsibility.** The Atrium primitive at the planned-v1-beta surface is structurally a MembershipSet-of-(principal-DIDs)-with-K_Atrium-and-AtriumPolicy-as-set-state. The capability-revocation operation is a separate primitive that consumes the MembershipSet but is NOT part of it. The register-zone + set-freshness-window operations are orthogonal cross-cutting controls that should remain on `AtriumHandle` without being folded.

**Cleanest collapsibility shape (M1b proposal for the panel):**
```rust
pub struct MembershipSet<M, S> {
    pub members: BTreeSet<M>,           // Member-DID set
    pub state: S,                       // Per-set state (K_Atrium, AtriumPolicy, etc.)
    pub admin: M,                       // Admin-DID
    pub parent: Option<Cid>,            // None for root sets; Some(parent_cid) for forks
}
```

Where `M = Did` (principal-DID grain) and `S = AtriumState { k_atrium, policy, ... }`. The 3 instances of MembershipSet Ben articulated:
- **Atrium** = `MembershipSet<Did, AtriumState>` where members are principal-DIDs.
- **Multi-device** (M1a) = `MembershipSet<DeviceDid, DeviceMeshState>` where members are device-DIDs, admin = parent user-DID, parent = None.
- **Single-device** (M1a) = `MembershipSet<DeviceDid, SoloDeviceState>` with `members = {single_device_did}` (degenerate singleton).

The fractal-recursable property (per L9 O7 + §3.5): a Garden = `MembershipSet<AtriumCid, GardenState>` where members are member-Atriums (set-of-sets).

### §9.2 What does NOT fit MembershipSet (red-flag observations for M4 red-team)

1. **`register_zone(zone: &str)`** — sync-scope namespace. Orthogonal to membership; multiple zones per Atrium; multiple Atriums per zone-namespace (zones are per-Atrium scoped today but namespace shape is independent). Should NOT be folded into MembershipSet.

2. **`set_envelope_freshness_window(secs)`** — replay-defense tunable. Per-Atrium-handle setting; NOT a per-member setting. Should NOT be folded.

3. **`set_local_device_did/keypair/attestation`** — these ARE device-DID-grain setters but they apply at the local engine, not at the Atrium-membership-set. They configure how THIS engine presents its device-DID outbound; they do NOT modify the membership-set. **Critical grain-disambiguation:** these belong on `AtriumHandle` (per-handle config), not on `MembershipSet` (per-set state).

4. **Capability-revocation** — `system:CapabilityRevocation` Nodes consume membership context but are NOT part of membership-set state. The cap chain + revocation graph is a SEPARATE primitive (per `crates/benten-caps/src/chain_authority.rs` post-COLLAPSE-P2 single seam). M1b proposal: keep capability-revocation as its own primitive; MembershipSet provides the addressable scope.

5. **HLC-temporal-ordering of interleaved grants/revocations** — per the `mst_diff_preserves_temporal_ordering_*` `#[ignore]`'d pin. This is a SEPARATE invariant from MembershipSet membership; it's a property of the sync layer. M1b proposal: keep as orthogonal property of the sync substrate, NOT part of MembershipSet.

6. **The L9-A2 dual-CID (plaintext_cid + envelope_blob_cid)** — this is content-identity-vs-authorization-identity. Forking the membership-set mutates the authorization-identity but NOT the content-identity. **If the unified primitive folds these together, forkability breaks.** M1b proposal: MembershipSet OWNS the authorization-identity (envelope-blob-CID); content-identity (plaintext-CID) is owned by the graph substrate independently.

---

## §10 Open questions for downstream specialists

### Q1 (M2 plan-doc author) — How is the MembershipSet primitive surfaced in the v1-beta public API?
The 28 unified amendments name 5 LOAD-BEARING Atrium-integration amendments (A1-A5) for v1-beta. Should the public API expose `MembershipSet<M, S>` as the canonical wire-shape OR keep AtriumHandle/DeviceMesh/SoloDevice as separate-but-shaped-the-same wrapper types? Authoring impact: which one of these collapses the implementation cost most without compromising forkability + storage-host-equality-oracle defense?

### Q2 (M2 plan-doc author) — Should the MembershipSet definition be in `benten-core` or in a new `benten-membership` crate?
SubgraphSpec lives in `benten-core` (per V1-FROZEN-INTERFACE §15.h: walker is data-not-evaluator-extension). MembershipSet could mirror this. But MembershipSet has admin-policy + K_Atrium concerns that may need `benten-caps` + `benten-crypto-suite` reach — a 15th workspace crate `benten-membership` may be the right home.

### Q3 (M3 specialist — fractal-architecture) — Does MembershipSet recursively compose for Atrium-of-Atriums?
Per L9 O7 the answer should be YES (Atrium-is-a-Subgraph; SubgraphSpec-is-Atrium-recursable). But the recursion's wire-format implications (Garden's K_Garden + member-Atrium-K_Atrium hierarchy; nested admin authority; cross-set revocation propagation) are NOT yet worked. Specialist needed.

### Q4 (M3 specialist — Kith composition) — How does the relational-attestation 3rd-DID-grain (Kith) compose with MembershipSet?
Per `docs/future/kith-decentralized-identity.md` Kith introduces "X says Y is Z to me" first-class relational-attestation graph. If MembershipSet is principal-DID-keyed at v1-beta, does Kith land as (a) a 3rd MembershipSet instance type, (b) a layer ABOVE MembershipSet (trust-graph evaluation that consumes MembershipSet for context), or (c) an orthogonal substrate? Plan-impact: Phase-5+ readiness of v1-beta wire format.

### Q5 (M4 red-team) — Does the proposed MembershipSet break the at-HEAD per-row cap-recheck contract at `apply_atrium_merge`?
The post-COLLAPSE-P2 single chain-validation seam (`crates/benten-caps/src/chain_authority.rs`) is the load-bearing security-defense. If unification routes capability-checks through a MembershipSet-keyed wrapper, the chain-walk must STILL terminate at user-DID-roots (per the plugin three-layer trust model). Red-team: construct an attack scenario where MembershipSet wrapping masks a chain-walk that should fail.

### Q6 (M4 red-team) — Does the K_Atrium random-on-creation + fork-on-rotate model survive multi-device-sync per M1a's grain?
The K_Atrium-distribution model assumes multi-stanza HPKE wrap to each NEW joining member. With multi-device-sync, a single user-DID has N devices; does each device need its own K_Atrium-wrap stanza, or does the user-DID-level K_Atrium-wrap suffice (with each device deriving from K_principal under DAK)? M1c likely owns the answer.

### Q7 (M4 red-team — adversary lens) — Can a malicious peer trigger MembershipSet-state divergence between honest peers via the leave-rejoin / fork interaction?
The `atrium_leave_rejoin.rs` substantive pin asserts trust-store survives leave-rejoin; per-row cap-recheck holds. But the `#[ignore]`'d revocation-order pins (`net-blocker-3` BLOCKER) suggest the revocation-ordering at offline-reconnect has unresolved gaps. Red-team: can an attacker exploit this to cause two honest peers to disagree about MembershipSet state?

### Q8 (M5 audit lens) — How does the L4 implementer-cost estimate (~35-45 wave-days for 18 LOAD-BEARING amendments) shift when factored through MembershipSet unification?
The 9-eyes consolidator estimate is for the amendments themselves; unification would shift refactor cost. Is the MembershipSet refactor cost-bounded by R3 sub-track sizing (~400-800 LOC per agent) or is it a structural orchestrator-multi-wave shape?

### Q9 (M5 audit lens) — What's the impact on `docs/V1-FROZEN-INTERFACE.md` §15 + the cargo-public-api baselines if MembershipSet lands?
V1-FROZEN §15 explicitly locks AuthorizationGrant + Scope + RestrictedScope + SubgraphSpec at v1-beta. A MembershipSet primitive that REPLACES AtriumHandle would re-cut several baselines. Plan-impact: cargo-public-api regeneration cost across 14 crates.

### Q10 (M6 dissent lens) — Is the MembershipSet collapse premature at the 5-week-old project age?
Ben's project age = 5 weeks; v1-beta in 16-20 weeks. The 12-primitive irreducibility commitment was made early + has held. Adding MembershipSet as a 13th primitive (even if 12 stays) is an architectural commitment that should survive Phase 5-8. Dissent: should this be a Phase-4-Meta-Composing decision (after v1-beta tag) rather than a Phase-4-Meta-Core decision (pre-freeze)?

### Q11 (cross-cataloger M1a) — How does multi-device-pairing (M1a scope) compose with Atrium-membership (M1b scope) at the device-DID grain?
Per `docs/GLOSSARY.md:23` Atrium has BOTH member-set-of-DIDs AND device-set-of-(device-DIDs)-per-DID. M1a + M1b need to agree on whether device-DIDs are first-class MembershipSet members (3-grain) or sub-instances of principal-DID members (2-grain with hierarchical sub-shape).

### Q12 (cross-cataloger M1c) — Does the K_Atrium random-on-creation model interact with K_principal derivation in any structural way?
Per Q3 §2.6 K_Atrium is INDEPENDENT of all other K-class keys (lives at Atrium-metadata layer, not user-vault layer). But K_Atrium STORAGE is encrypted-under-K_principal in each member's local vault. M1c likely owns the K_principal-vault interaction; M1b owns the K_Atrium-distribution + rotation semantics.

---

## §11 Self-assessment + confidence + cross-references to M1a/M1c

### §11.1 Confidence on this cataloging

**HIGH confidence (≥85%) on:**
- The Atrium current-state code inventory (§2) — read substantively at HEAD; cited file:line.
- The SubgraphSpec + AuthorizationGrant frozen shape (§5) — `docs/V1-FROZEN-INTERFACE.md` §15 is authoritative + at HEAD.
- The Drop bundle wire format (§6.1) — `crates/benten-drop` is shipped at HEAD; INTERNALS.md is comprehensive.
- The forkability semantic (§7) — Ben-ratified explicit + composed cleanly per L9 §4.1 walk-through.
- The MembershipSet-collapsibility verdict (§9) — derived from cataloged code + planning surfaces; defensible.

**MED-HIGH confidence (~70-85%) on:**
- The planned-state shape (§3) — 5 of 5 ratified-S&C decisions are at HEAD via phase-4-backlog §3.10 paraphrase, but the source-of-truth RATIFIED-S&C 2026-05-21 doc was VERIFIED ABSENT from git. I rely on SECURITY-POSTURE.md Compromise #31 + V1-FROZEN-INTERFACE.md §15 + benten-drop INTERNALS.md + phase-4-backlog §3.10 as the surviving anchors. Risk: if the RATIFIED-S&C doc has nuances NOT propagated to the 4 anchors, I missed them.
- The AtriumPolicy design (§8.2) — reachable via `git show e90900b4:.addl/phase-4-meta/atrium-policy-credential-validity-design.md`; this is a DESIGN doc not yet ratified. Ben call surface (D1-D5) per §1.2 of that doc is unresolved. My §8.2 narrative reflects the design's HIGH-confidence recommendations.
- The Q3 Option D K_Atrium-blinded plaintext_cid (§3.1 + §7.1 + §9.1) — reachable via `git show 23f76e24`; this is a senior-cryptographer + P2P-systems-architect review concluding HIGH (~85%) on Option D as PRIMARY with two qualifications. My §3.1 C44 row + §7.1 fork-on-rotate semantics + §9.1 dual-CID-MUST-not-collapse derive from this.

**MED confidence (~50-70%) on:**
- The §3.5 future-phase long-horizon Atrium scope — based on phase-4-backlog + GLOSSARY + the L9 fractal-architecture O7. Gardens/Groves/Kith/AI-agent-as-member are inferred from sparse data; risk of misreading Ben's long-horizon intent.
- The §10 open questions for M2-M6 — these are MY surfacing of unresolved decisions, not Ben-ratified questions. Specialist panels may decide some are non-issues.

**LOWER confidence (~30-50%) on:**
- The exact behavior of `apply_atrium_merge`'s receiver-side `peer_did_set` resolution under leave-rejoin (claim in §4.1 + assertion (ii) of `atrium_leave_rejoin.rs` pin). I read the test body but the engine-side impl I only inferred from rustdoc + the pin's observable assertions.
- Whether AI-agent-as-Atrium-member is a planned long-horizon shape (§3.5). I searched the planning corpus + found NO explicit plan; my read is "AI agents = plugins, not Atrium members" but Ben may have a different intent.

### §11.2 What I did NOT investigate (within-scope but I ran out of time)

- The full `apply_atrium_merge` impl in `crates/benten-engine/src/engine.rs` — I observed it via the substantive test pin's assertions but did not read the impl LOC-by-LOC.
- The `crates/benten-sync/src/handshake.rs` + `crates/benten-sync/src/handshake_wire.rs` HLC + nonce defenses in the handshake state machine.
- The 5 critique-c1..c5 docs (option-f-plus-critique-{c1,c2,c3,c4,c5}.md) — these are reachable via git show but were not load-bearing for Atrium-membership-sharing cataloging (they critique the 9-eyes registry; the Atrium-relevant amendments are already in the registry).
- The L10/L11/L12 lens reviews (perf-wire-size / CRDT / dead-code) — not Atrium-membership-sharing-specific.

### §11.3 Cross-references to M1a (multi-device-sync) and M1c (key-management)

**Flagged to M1a (multi-device-sync):**
- The 3 device-DID-grain setters (`set_local_device_did/keypair/attestation`) at §9 #12. M1a owns the device-pairing UX walkthrough.
- The `DeviceAttestation` envelope V2 signed-shape per §2.1. M1a owns the device-DID rotation + revocation propagation.
- The `register_peer_did(node_id, did)` at §1.2(c). M1b is the user-DID-grain; M1a may also use this primitive at the device-DID-grain.
- The 4 `#[ignore]`'d revocation-order pins in §2.4 — `atrium_device_did_revocation_propagates_before_data_to_offline_then_reconnect_peer` is device-DID grain; M1a should claim.
- The `bindings/napi/tests/cap_device_did_revocation_resolved_scope_regression_guard.rs` at §2.9 — substrate-boundary invariant for device-DID rotation vs cap-revoke. M1a owns.
- Question Q11 in §10 — how device-DID grain composes with principal-DID grain.

**Flagged to M1c (key-management):**
- The K_principal / K(N) per-Node AEAD derivation at §1.2(i) + §5 + §6.3. M1c owns the X-Wing combiner + the K(N) = HKDF(K_principal, ...) chain.
- The DAK (device-authentication key) per §3.5 Layer-D. M1c owns the DAK derivation.
- The K_Atrium random-on-creation per §1.2(j) + §7 — distinct from K_principal but stored encrypted-under-K_principal in member vault. **M1b owns the K_Atrium DISTRIBUTION + ROTATION semantics; M1c owns the K_principal + K_Atrium-storage-vault layer.**
- Question Q12 in §10 — how K_Atrium interacts with K_principal at the vault layer.
- The HybridSigPubKey (Inv-17 hybrid floor) for AtriumPolicy `admin_pubkey` per §8.2 — M1c owns the hybrid signature primitive.
- The U19 / L9-A3 `recipient_key_generation: u32` field in BindingContext::DropToRecipient + grace-window key-retention — M1c owns the recipient-side key-retention policy.
- The U20 / L9-A4 `k_principal_generation: u32` field in Layer-B AEAD AAD + Atrium-replicated K_principal-rotation-log — M1c owns the rotation primitive; M1b owns the Atrium-replicated-Node distribution shape.

**End of M1b catalog.** ~14,400 words. Ready for downstream MembershipSet specialist panel consumption.
