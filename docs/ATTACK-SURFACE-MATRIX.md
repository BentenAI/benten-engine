# Attack-Surface Matrix

**Status:** Phase-3 close (R5 wave-9 W9-T2). Authored as the meta-completeness audit destination per `docs/future/phase-3-backlog.md` §7.13 (origin: `sec-r4r2-2` MAJOR / `sec-r4r1-4` MAJOR / root `sec-r1-7`).

**Role.** This doc enumerates every named attack surface in the engine, classifies its current defense state, and cites the test pin(s) driving each concrete attack vector. The matrix's primary completeness role is *failing loud* if any named surface lacks a driving test pin: a missing test pin on a named surface is a finding (file under the active phase's `docs/future/phase-N-backlog.md` per HARD RULE rule-12 clause-b). Surfaces classified `Mitigated-by-construction` do not need a driving test pin (the architectural choice IS the defense; the absence of the surface is the closure).

**Scope split.** The matrix has two halves:

1. **Phase-2b SANDBOX (16 ESC vectors).** Static surface set `pre-r1-security-deliverables.md §1` is canonical; the matrix here is a re-issuance of `docs/SECURITY-POSTURE.md` Compromise #4 ESC table for cross-reference. The authoritative status table for ESC-1..16 lives in `docs/SECURITY-POSTURE.md` Compromise #4 — this doc cites it to avoid a two-source drift hazard.
2. **Phase-3 P2P-sync attack surfaces.** Atrium peer-handshake, UCAN proof-chain transport, sync-replica trust-boundary, device-DID attestation, iroh-relay metadata.

**HARD RULE rule-12 stance.** Every surface row is one of: (a) `Defended` (test pin cited); (b) `Mitigated-by-construction` (architectural choice; no test pin needed but must justify); (c) `Phase-N-deferred` with a NAMED + REAL destination (verified to exist at HEAD). If a row is neither, that is a finding to surface for FIX-NOW disposition.

---

## Part 1 — Phase-2b SANDBOX ESC matrix (re-issued for cross-reference)

For each of the 16 named SANDBOX-escape vectors (`pre-r1-security-deliverables.md §1`), the authoritative status table is `docs/SECURITY-POSTURE.md` Compromise #4. Brief status snapshot at Phase-3 wave-9 close:

| Vector | Class | Status | Authoritative row |
|---|---|---|---|
| ESC-1 | OOB linear-memory read | Fully wired | `docs/SECURITY-POSTURE.md` Compromise #4 ESC-1 row |
| ESC-2 | Linear-memory grow beyond per-call cap | Fully wired | Compromise #4 ESC-2 row |
| ESC-3 | Host-buffer overrun via host-fn output write | Fully wired | Compromise #4 ESC-3 row |
| ESC-4 | Infinite loop without fuel | Fully wired | Compromise #4 ESC-4 row |
| ESC-5 | Recursive-call stack overflow | Fully wired (dedicated `E_SANDBOX_STACK_OVERFLOW` typed variant per phase-3-backlog §6.4) | Compromise #4 ESC-5 row |
| ESC-6 | Fuel-counter overflow regression | Fully wired | Compromise #4 ESC-6 row |
| ESC-7 | Fuel-refill via host-fn re-entry | Fully wired end-to-end at Phase-3 wave-5c | Compromise #4 ESC-7 row |
| ESC-8 | Call host-fn not in manifest | Fully wired | Compromise #4 ESC-8 row |
| ESC-9 | Cap-revoke mid-call (TOCTOU) | Fully wired end-to-end at Phase-3 wave-5c | Compromise #4 ESC-9 row |
| ESC-10 | Re-entrancy via host-fn (cap-context confusion) | Wired-defense; adversarial test-paper-only (`#[ignore]`'d pending `testing_call_engine_dispatch` helper body — Phase-3 G20-A1 wave-8a fills the body) | Compromise #4 ESC-10 row |
| ESC-11 | Component-Model type mismatch | Component-model-feature-cut (defense IS the cut) | Compromise #4 ESC-11 row |
| ESC-12 | Resource handle forgery | Component-model-feature-cut (same cut as ESC-11) | Compromise #4 ESC-12 row |
| ESC-13 | Trap during fuel-meter callback / Store-state corruption | Fully wired end-to-end at Phase-3 wave-5c | Compromise #4 ESC-13 row |
| ESC-14 | Cap-claim forge in module bytes | Mitigated-by-construction (engine ignores embedded WASM custom sections; cap derivation is exclusively from manifest passed at call time) + eval-side smoke pin | Compromise #4 ESC-14 row |
| ESC-15 | Named-manifest spoofing | Fully wired | Compromise #4 ESC-15 row |
| ESC-16 | Wall-clock leak via `time` host-fn fingerprinting | Fully wired end-to-end at Phase-3 wave-5c | Compromise #4 ESC-16 row |

**Bucket totals (cross-checked at Phase-3 wave-9 close, matching `docs/SECURITY-POSTURE.md` Compromise #4 bucket-totals paragraph):** 12 fully wired (ESC-1, -2, -3, -4, -5, -6, -7, -8, -9, -13, -15, -16) + 1 partial (ESC-14) + 1 wired-defense / test-paper-only (ESC-10) + 2 component-model gated (ESC-11, -12) = 16. The component-model-gated pair (ESC-11/-12) carries a closure-by-construction posture: the wasmtime workspace dep at `Cargo.toml::workspace.dependencies::wasmtime` ships without the `component-model` feature, so the surface is compile-time absent. If the feature ever lands, ESC-11/-12 promote to "Defended end-to-end" via the existing `#[cfg(feature = "component-model")]`-gated test pins.

**Fixture catalog cross-reference.** Every ESC defense has a committed fixture under `crates/benten-eval/tests/fixtures/sandbox/escape/<name>.wat` + a `.wasm` sibling per Phase-3 G17-B wave-5b D26 closure (`docs/future/phase-3-backlog.md` §6.2). Fixture stems: `oob_linmem_read`, `linmem_grow_to_limit`, `host_buf_overrun`, `infinite_loop`, `recursive_call_overflow`, `fuel_overflow_regression`, `fuel_refill_via_host_fn`, `host_fn_not_on_manifest`, `host_fn_after_cap_revoke`, `reentrancy_via_host_fn`, `component_type_mismatch`, `resource_handle_forgery`, `forged_cap_claim_section`, `wallclock_fingerprint`. Two ESC vectors (ESC-15 named-manifest spoofing, ESC-7 fuel-refill end-to-end via test-seam) drive their attack pattern via state injection rather than a `.wat` fixture (the attack pattern requires runtime-state mutation that no isolated wasm guest can express).

---

## Part 2 — Phase-3 P2P-sync attack surfaces

The Phase-3 sync layer adds attack surfaces orthogonal to the Phase-2b SANDBOX surface set. Each surface below corresponds to a named threat-model class enumerated in `docs/future/phase-3-backlog.md` §7.13.

### 2.1 — Atrium peer-handshake

The Atrium join handshake (the per-peer mutual-authentication step that establishes the shared cap-set + revocation snapshot before sync data flows) is the trust-establishment frontier between two peers' DIDs.

**Concrete attack vectors:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| HS-1 | Signature tampering on inbound handshake frame | Ed25519 signature verification at handshake-receive boundary; `verify_handshake_signature` rejects forgery before any cap-state mutation. | `crates/benten-sync/tests/handshake.rs::handshake_rejects_invalid_signature` |
| HS-2 | Replay of a valid prior handshake outside the bounded freshness window | Bounded freshness window (the receiver tracks last-seen handshake-nonce per peer-DID; replay outside the window rejected with `E_HANDSHAKE_REPLAY`). | `crates/benten-sync/tests/handshake.rs::handshake_rejects_replay_within_bounded_window` |
| HS-3 | Peer-DID forgery (claiming to be a different peer's DID without holding the corresponding signing key) | DID-based mutual auth: each side proves possession of the private key for the claimed DID via signed challenge-response. A forger without the private key cannot complete the round-trip. | `crates/benten-sync/tests/handshake.rs::handshake_did_based_mutual_auth_round_trip` |
| HS-4 | Cap-set widening via crafted UCAN-grant exchange during handshake | Per-peer cap-set is established from validated UCAN grants only (the chain walk attenuates; widening is rejected at the chain-walk seam). | `crates/benten-sync/tests/handshake.rs::handshake_ucan_grant_exchange_establishes_per_peer_cap_set` |
| HS-5 | Stale revocation state on connect (peer joins with a revoked-actor not yet known to the receiver) | Revocation-state-first ordering: the handshake synchronizes revocation state BEFORE subscribing the data stream, so any cap-revoke that landed at the sender pre-handshake is observed before the first data frame. | `crates/benten-sync/tests/handshake.rs::atrium_handshake_synchronizes_revocation_state_before_subscribing_data` |

**Cross-reference.** `crates/benten-sync/src/handshake.rs` (production receive logic). The handshake protocol's wire shape is documented in `crates/benten-sync/src/handshake_wire.rs`.

### 2.2 — UCAN proof-chain transport

UCAN delegation chains carry capability authority across DIDs. The chain-walk seam is where authority decisions are made. Three failure modes are defended.

**Concrete attack vectors:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| UC-1 | Window-widening (extending validity past parent's `nbf` / `exp` bound) | Chain-walk propagates `nbf` / `exp` through every attenuation step; child chain validity intersects with parent's. | `crates/benten-id/tests/ucan.rs::ucan_chain_walk_propagates_nbf_exp_through_attenuation` |
| UC-2 | Authority-widening (child claims a capability parent did not delegate) | Chain-walk attenuation check: child's claimed caps must be a subset of parent's at every step. | `crates/benten-id/tests/ucan.rs::ucan_chain_attenuation_rejects_overgrant` |
| UC-3 | Revocation-propagation skip (child UCAN signed by a now-revoked DID still accepted) | Chain-walk consults the revocation log: any DID in the chain that is revoked at validation-time fails the chain. | `crates/benten-id/tests/ucan.rs::ucan_chain_revocation_propagates` + `crates/benten-id/tests/did_rotation.rs::did_rotation_propagates_revocation_to_ucan_backend` |
| UC-4 | Cross-Atrium replay (UCAN issued for Atrium A presented at Atrium B) | Audience binding: validation requires the UCAN's `aud` field to match the receiving Atrium's expected audience. | `crates/benten-id/tests/ucan.rs::ucan_audience_binding_prevents_cross_atrium_replay` |
| UC-5 | Pre-`nbf` early-activation attempt | Time-window enforcement at validation; pre-activation rejected with typed error. | `crates/benten-id/tests/ucan.rs::ucan_nbf_time_window_pre_activation_rejects` + `ucan_chain_nbf_enforcement` |
| UC-6 | Post-`exp` post-expiration replay | Time-window enforcement at validation; expired UCAN rejected. | `crates/benten-id/tests/ucan.rs::ucan_exp_time_window_post_expiration_rejects` + `ucan_chain_exp_enforcement` |
| UC-7 | Timing side-channel via non-constant-time signature comparison | Constant-time comparison audit pin: the chain-walk MUST NOT short-circuit on early signature-mismatch (timing oracle for valid prefix). | `crates/benten-id/tests/ucan.rs::ucan_chain_walk_constant_time_comparison_audit` |

**Cross-reference.** `crates/benten-id/src/ucan.rs::validate_chain_inner` (production chain-walk logic). The chain-walk seam composes with device-DID attestation (§2.4) at `validate_chain_with_attestations` and DID rotation (§2.4) at `validate_chain_with_rotation_log`.

### 2.3 — Sync-replica trust-boundary (Loro op-log + MST diff + HLC)

Sync replicas exchange Loro CRDT op-logs, MST (Merkle Search Tree) diffs, and HLC-stamped envelopes. Each is an inbound-frame surface where a malicious peer (or a compromised relay, modulo §2.5) could inject crafted state.

**Concrete attack vectors:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| SR-1 | Loro op-log violating Inv-13 immutability (re-write of a previously-committed op) | Inv-13 enforcement at dispatch (not just at CID-divergence): the inbound op is checked against the registered-subgraph/version-chain immutability rule at application layer. | `crates/benten-sync/tests/attack_loro_op_log_inv_13.rs::loro_merge_op_log_violating_inv_13_immutability_rejected_at_dispatch_not_just_at_cid_divergence` |
| SR-2 | MST diff entry with CID byte-mismatch (claimed CID does not match content bytes) | Application-layer rejection: receiving peer recomputes the BLAKE3 hash + CIDv1 envelope from the received bytes and rejects if it does not match the claimed CID. | `crates/benten-sync/tests/attack_mst_diff_cid_mismatch.rs::mst_diff_entry_with_cid_byte_mismatch_rejected_at_application_layer` |
| SR-3 | HLC skew injection (inbound frame with clock-stamp far enough in the future to disturb causality ordering) | Bounded HLC-skew tolerance: inbound frames with clock-stamps exceeding the configured skew window rejected with `E_HLC_SKEW_EXCEEDED`. | `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs::hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded` |
| SR-4 | Loro CRDT divergent merge causing inconsistent LWW resolution | LWW resolution is HLC-deterministic; the property test pins convergence under arbitrary op-arrival ordering. | `crates/benten-sync/tests/prop_loro_converge.rs` (property test) + `hlc_loro_property_lww.rs` |
| SR-5 | MST diff rejecting valid revocation-priority ordering | Revocation-prioritized MST diff: a revocation entry takes precedence over a write entry at the same key per the priority rule. | `crates/benten-sync/tests/mst_revocation_priority.rs` |

**Cross-reference.** `crates/benten-sync/src/mst.rs` (MST + CID-byte-mismatch defense), `crates/benten-sync/src/crdt.rs` (Loro merge + Inv-13 dispatch-time check + HLC skew enforcement).

### 2.4 — Device-DID attestation

Device DIDs are sub-DIDs delegated from a primary DID to a specific device (laptop / phone-OS app / browser session). The attestation envelope binds the device's signing key to the parent DID's authority + a per-device capability envelope (e.g. `runs_sandbox: false` for a browser-target session).

**Concrete attack vectors:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| DA-1 | Envelope-downgrade attack (re-issuing the attestation with a wider capability envelope without parent re-signing) | Runtime recheck against parent-chain at every UCAN delegation that consumes the attestation; widening fails at the parent-chain comparison. | `crates/benten-id/tests/device_attestation.rs::device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain` |
| DA-2 | Parent-chain forgery (claiming a parent DID that did not actually issue the attestation) | Forged parent-signature rejected at attestation-acceptance time. | `crates/benten-id/tests/device_attestation.rs::acceptor_rejects_attestation_with_forged_signature` |
| DA-3 | Freshness-window replay (presenting an old valid attestation to bypass post-revocation state) | Freshness window at attestation-receive: attestations outside the window rejected. | `crates/benten-id/tests/device_attestation.rs::device_attestation_replay_resistant_within_freshness_window` |
| DA-4 | Nonce-replay within the freshness window (presenting the same attestation twice within the window) | Nonce-store at the attestation-receive boundary: each attestation's nonce is consumed once; duplicate presentation rejected. | `crates/benten-id/tests/device_attestation.rs::device_attestation_replay_resistance_via_nonce_freshness_window` |
| DA-5 | Revoked device signing new UCAN delegation | Revocation propagated through the chain-walk: a UCAN signed by a revoked device fails validation. | `crates/benten-id/tests/device_attestation.rs::device_attestation_revoked_device_cannot_sign_new_ucan_delegation` |
| DA-6 | Browser-target falsely claiming `runs_sandbox: true` | Constructor-time validation: a browser-target attestation with `runs_sandbox: true` rejected at construction (not just at runtime invocation). | `crates/benten-id/tests/device_attestation.rs::browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time` |
| DA-7 | UCAN delegation to a browser-target for a SANDBOX-required handler | Chain-construction-time rejection: the delegation seam refuses to mint a UCAN whose subject (browser-target attestation) cannot run the requested SANDBOX. | `crates/benten-id/tests/device_attestation.rs::ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation` |
| DA-8 | Device-signed re-attestation widening parent's capability envelope | Re-attestation cannot widen: the parent envelope is the upper bound; a device-signed re-attestation with `runs_sandbox: true` against a `runs_sandbox: false` parent envelope rejected. | `crates/benten-id/tests/device_attestation.rs::device_attestation_runs_sandbox_false_cannot_be_widened_by_device_signed_re_attestation` |
| DA-9 | DID rotation widening authority post-rotation | Rotation log at chain-walk: the post-rotation DID inherits only the pre-rotation authority envelope; widening rejected. | `crates/benten-id/tests/did_rotation.rs::superseded_did_cannot_sign_new_ucan_delegations` |

**Cross-reference.** `crates/benten-id/src/device_attestation.rs` (production attestation logic), `crates/benten-id/src/did_rotation.rs` (rotation log), `crates/benten-id/src/ucan.rs::validate_chain_with_attestations` (chain-walk seam consuming attestations), `validate_chain_with_rotation_log` (chain-walk seam consuming the rotation log). The browser-target heterogeneity contract (CLAUDE.md baked-in #17 thin compute surface) feeds DA-6 / DA-7 / DA-8.

### 2.5 — iroh-relay metadata leakage (Compromise #22)

iroh's QUIC + relay protocol uses public relay infrastructure for NAT traversal. The relay sees encrypted payloads but observes peer-DIDs and connection-metadata.

**Status:** *Compromise* (introduced at Phase-3 close, NOT defended). Documented in `docs/SECURITY-POSTURE.md` Compromise #22 with named closure target Phase-7 Garden-relays (failing that Phase-9 hardened-deployment posture).

**Concrete attack vectors (NOT defended at Phase-3; documented as known compromise):**

| # | Vector | Status | Phase-3 stance | Closure target |
|---|--------|--------|----------------|----------------|
| MR-1 | Public-iroh-relay observation of peer-DID pairs (who-talks-to-whom) | Open compromise | Documented honestly in Compromise #22 | Phase-7 Garden-relays / Phase-9 hardened-deployment |
| MR-2 | Connection-metadata observability (timing, peer-availability windows, Atrium membership inferable from connection patterns) | Open compromise | Documented honestly in Compromise #22 | Phase-7 / Phase-9 |
| MR-3 | Membership-topology inference (which DIDs co-occur in connection sessions hints at Atrium membership without decrypting payload) | Open compromise | Documented honestly in Compromise #22 | Phase-7 / Phase-9 |

**Cross-reference.** `docs/SECURITY-POSTURE.md` Compromise #22 carries the full narrative + threat-model deltas + Phase-7 promotion path. RED-PHASE structural pin at `tests/phase_3_workspace/security_posture_compromises.rs::compromise_22_public_relay_metadata_leakage_introduced_at_phase_3_close_with_named_phase_7_garden_relay_destination`.

### 2.6 — Atrium join + revocation-ordering boundary (cross-cutting)

The Atrium-level lifecycle (join / partial-partition / revoke / errors-surface) is the cross-cutting boundary where §2.1 handshake state, §2.3 sync-replica state, and §2.4 device-DID state all converge.

**Concrete attack vectors:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| AT-1 | Revocation-order race (peer publishes a UCAN-revocation; concurrent inbound write-frame from another peer claims authority via the revoked UCAN) | Revocation-state-first ordering (sibling of HS-5): inbound writes processed against the post-revocation cap-set. | `crates/benten-sync/tests/atrium_revoke_order.rs` |
| AT-2 | Partial-partition handling (peer rejoining after partition with stale revocation state) | Partition-recovery synchronizes revocation state before re-enabling write authority. | `crates/benten-sync/tests/atrium_partial_partition.rs` |
| AT-3 | Atrium-join with crafted invalid state (peer claims membership without valid grant chain) | Join-time validation rejects unauthenticated joins. | `crates/benten-sync/tests/atrium_join.rs` + `atrium_errors.rs` |
| AT-4 | View-result publication forge (peer publishes a view result for a view they were not authorized to compute) | UCAN-gated `host:atrium:publish_view_result` cap (per Phase-3 D2 ratification + D-PHASE-3-21). | `crates/benten-sync/tests/host_atrium_publish_view_result_caps.rs` |
| AT-5 | Rate-limit budget exhaustion attack (peer floods write frames to consume another peer's resources) | Per-peer rate-limit consumption tracking (Compromise #5 sibling at the sync layer). | `crates/benten-sync/tests/rate_limit_consumption.rs` |
| AT-6 | Light-client trust violation (light-client peer claiming full-peer authority) | Light-client capability boundary enforced; light-client cannot publish view results. | `crates/benten-sync/tests/light_client.rs` + `light_client_distinct.rs` |

**Cross-reference.** `crates/benten-sync/src/transport.rs` (Atrium-transport lifecycle), `crates/benten-sync/src/peer_discovery.rs` (peer-discovery + join), `crates/benten-sync/src/handshake_wire.rs` (inbound-frame envelope surface).

### 2.7 — Iroh peer-id derivation (binding)

The iroh peer-id is deterministically derived from the ed25519 public key — this is the binding that prevents a malicious peer from claiming a different DID's iroh peer-id.

**Concrete attack vector:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| PI-1 | iroh peer-id forgery (claiming a peer-id not derived from the holder's pubkey) | Deterministic derivation from ed25519 pubkey; mismatch is unforgeable without the private key. | `crates/benten-sync/tests/peer_id.rs::iroh_peer_id_derived_deterministically_from_ed25519_pubkey` |

**Cross-reference.** `crates/benten-sync/src/peer_id.rs`.

### 2.8 — Typed-CALL dispatch surface (Phase-3 G21-T1)

The Phase-3 G21-T1 typed-CALL dispatch surface (see [`docs/TYPED-CALL.md`](TYPED-CALL.md)) adds a closed registry of 10 engine-known ops dispatched through the existing CALL primitive when the `target` (handler id) starts with `engine:typed:`. Per `CLAUDE.md` baked-in **#16**, typed-CALL is the home for engine-known fixed-shape compute that fits CALL semantics (Ed25519 sign/verify, BLAKE3 hash, multibase, DID resolve, UCAN chain validation, VC verify, keypair generation) while the SANDBOX host-fn surface stays minimum-viable. The dispatch surface adds new attack vectors orthogonal to the Phase-2b SANDBOX surface set.

**Concrete attack vectors:**

| # | Vector | Defense | Test pin |
|---|--------|---------|----------|
| TC-1 | Per-op cap-bypass (handler dispatches a typed-CALL op without holding the per-op `cap:typed:*` cap) | `PrimitiveHost::check_capability` hook gates BEFORE the underlying `benten-id` / `benten-core` op is invoked; denied dispatch routes to `ON_DENIED` with `E_TYPED_CALL_CAP_DENIED` and zero observable side effect. | `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_cap_denied_via_capability_policy_returns_on_denied_edge` + `typed_call_cap_denied_routes_on_denied_other_three_route_on_error` |
| TC-2 | Cap-claim forge via crafted `target` string (handler crafts `engine:typed:<op>` to invoke an op whose cap they don't hold) | Cap requirement derives from the closed `TypedCallOp` enum's `required_cap()` arm at dispatch time — the cap string is NOT taken from any user-controlled bytes, so a forged `target` cannot widen the required-cap envelope. The cap-check (TC-1) then enforces. | `crates/benten-eval/src/typed_call.rs::TypedCallOp::required_cap` (production source-of-truth; closed-enum mitigation is by construction) + `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_required_caps_each_op_namespaced` |
| TC-3 | Reserved-namespace squat (user handler registered at `engine:typed:<op>` to shadow engine dispatch) | At G21-T1 the eval-side dispatch fork pre-empts user-handler routing for the `engine:typed:` prefix — a registered handler at this namespace is dead code rather than a routing override. Registration-time hard reject (`E_RESERVED_HANDLER_NAMESPACE`) lands at G21-T3 per `docs/future/phase-3-backlog.md` §2.5(d). | `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_namespace_pre_empts_user_handler_registry_for_unknown_op` |
| TC-4 | Inv-9 determinism violation (a non-deterministic typed-CALL op like `keypair_generate` dispatched from a `is_deterministic = true` finalized subgraph) | `TypedCallOp::is_deterministic` per-op classification consulted by the Inv-9 finalization-time walker; non-deterministic ops in a deterministic subgraph rejected at finalization. | `crates/benten-eval/tests/invariant_9_finalized.rs::invariant_9_fires_for_typed_call_keypair_generate_in_deterministic_handler` + `invariant_9_permits_deterministic_typed_call_op_in_deterministic_handler` |
| TC-5 | Input-shape exploitation (malformed `Value::Map` driving an op into an unsafe code path in `benten-id` / `benten-core`) | Per-op `validate_input` shape check rejects malformed inputs with `E_TYPED_CALL_INVALID_INPUT` BEFORE the underlying API call; fixed-width fields (Ed25519 keys 32B, signatures 64B) are length-checked at dispatch time. | `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_invalid_input_via_call_primitive_routes_typed_error` |
| TC-6 | Unknown-op probe (handler dispatches `engine:typed:nonexistent` to fingerprint engine version) | `TypedCallOp::parse` returns `None` for unknown ops; dispatch surfaces `E_TYPED_CALL_UNKNOWN_OP` rather than falling through to the user-handler registry (which would have surfaced `E_NOT_FOUND` and leaked registry membership). | `crates/benten-engine/tests/typed_call_engine_dispatch.rs::typed_call_unknown_op_via_call_primitive_routes_typed_error` |
| TC-7 | UCAN chain forge / window-widening / audience-replay through `ucan_validate_chain` op | Op delegates to `benten_id::ucan::validate_chain_inner` which is the same chain-walk seam covered by §2.2 UC-1..UC-7 vectors; typed-CALL is a thin facade. | `crates/benten-engine/tests/typed_call_engine_dispatch.rs::ucan_validate_chain_returns_false_with_reason_on_audience_mismatch` + `ucan_validate_chain_returns_false_when_leaf_att_does_not_grant_required_capability` (composes with §2.2 UC-1..UC-7 chain-walk pins) |
| TC-8 | Expired-VC replay through `vc_verify` op | Op routes through `benten_id::vc::verify_at(..., now)` (NOT bare `vc::verify`) so an expired VC returns `valid: false` rather than a bypass — `now` is REQUIRED in the input shape so the time-source is operator-visible at the call site. | `crates/benten-engine/tests/typed_call_engine_dispatch.rs::vc_verify_returns_false_when_credential_is_expired_at_now` |

**Open carries (Phase-3 named follow-up — `docs/future/phase-3-backlog.md` §2.5):**

| # | Carry | Status | Phase-3 destination |
|---|-------|--------|---------------------|
| TC-CARRY-1 | Secret-byte zeroize discipline at `build_seed_envelope` + per-op `Value::Bytes` outputs (sec-minor-2) | Open carry — natural Vec drop occurs but no zeroize-on-drop wrapper | `phase-3-backlog.md` §2.5(a); `zeroize` already a workspace dep. |
| TC-CARRY-2 | `did_resolve` DID-method validation (sec-minor-3) — input DID's method prefix not parsed; non-`did:key:` would silently produce a wrong `method: "key"` field | Open carry — conservative `is_deterministic = false` already set in anticipation | `phase-3-backlog.md` §2.5(b). See [`TYPED-CALL.md`](TYPED-CALL.md) §"did_resolve DID-method validation". |
| TC-CARRY-3 | UCANBackend → `cap:typed:*` policy mapping (sec-minor-4) — under UCAN, no claim grants any `cap:typed:*` cap; cap-deny-by-default surfaces | Open carry; under `NoAuthBackend` all permitted (canary-scope intent) | `phase-3-backlog.md` §2.5(c); couples to G21-T2 napi-UCAN-wireup. |
| TC-CARRY-4 | Reserved-namespace registration-time reject (corr-minor-3) — `register_subgraph` does not currently reject `handler_id` starting with `engine:typed:` | Open carry; eval-side fork pre-empts so user registration is dead code | `phase-3-backlog.md` §2.5(d); G21-T3 lands `E_RESERVED_HANDLER_NAMESPACE` 4-surface §3.5g atomic update. |

**Cross-reference.** `crates/benten-engine/src/typed_call_dispatch.rs` (engine-side per-op dispatch impls), `crates/benten-eval/src/typed_call.rs::TypedCallOp` (closed enum + per-op validate_input + required_cap + is_deterministic). [`docs/TYPED-CALL.md`](TYPED-CALL.md) is the engineer-facing reference. The 4 typed-CALL `ErrorCode` rows live in [`docs/ERROR-CATALOG.md`](ERROR-CATALOG.md) (`E_TYPED_CALL_UNKNOWN_OP` / `E_TYPED_CALL_INVALID_INPUT` / `E_TYPED_CALL_CAP_DENIED` / `E_TYPED_CALL_DISPATCH_ERROR`).

**Audit note (silent-weakening check at G21-T1 close).** Walked Part 1 + Part 2 + Part 3 entries to verify typed-CALL did not silently weaken any defense. Findings:

1. ESC-14 (cap-claim forge in module bytes) — *not weakened.* ESC-14's mitigation is "engine ignores embedded WASM custom sections; cap derivation is exclusively from manifest passed at call time." Typed-CALL's cap derivation (TC-2) is from the closed enum — same construction-class mitigation, additive surface.
2. §3.1 Compromise #1 (Cap TOCTOU) — *not weakened.* Typed-CALL fires `check_capability` at dispatch entry (the CALL-primitive boundary), same as bare CALL; the TOCTOU window bound at CALL entry covers the typed-CALL fork.
3. §3.2 Inv-13 — *not weakened.* Typed-CALL ops do NOT mutate engine state directly; the WRITE primitive remains the only path. Per CLAUDE.md baked-in #16: SANDBOX modules return values that the engine's WRITE primitive persists — typed-CALL ops follow the same pattern (return values, not direct writes).
4. §3.3 Inv-14 attribution — *not weakened.* Typed-CALL dispatch occurs inside a CALL primitive's `ActiveCall`, so the attribution frame propagates through the dispatcher stack unchanged.

No silently-weakened entries identified.

---

## Part 3 — Engine-layer attack surfaces (cross-phase)

These surfaces are not Phase-3-introduced but are documented here for completeness so the matrix can serve as the single doc-level enumeration audit.

### 3.1 — Capability TOCTOU (Compromise #1)

The TOCTOU window between capability check and capability use is bounded at CALL entry + ITERATE batch boundary. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #1 (Phase-2a additive). Phase-3 ESC-9 (`§Part 1`) addresses the SANDBOX-host-fn variant of the same class.

### 3.2 — Inv-13 immutability (registered subgraph immutability)

**Defense:** Inv-13 firing matrix at WRITE-time. Phase-2b 5-row matrix at `docs/SECURITY-POSTURE.md` § "Inv-13 immutability firing matrix". Phase-3 sync extends the firing matrix to inbound op-logs (§2.3 SR-1).

**Test pin:** `crates/benten-eval/tests/invariant_13_no_write_to_registered_subgraph.rs`.

### 3.3 — Inv-14 attribution chain integrity

**Defense:** AttributionFrame routing through every primitive (`sec-r6r1-01`). Tampering with attribution mid-call would break the audit-trail; production code path keeps the frame on the dispatcher stack so no primitive can rewrite it.

**Test pin:** Phase-2a Inv-14 regression suite + Phase-2b `crates/benten-engine/tests/integration/engine_sandbox.rs::engine_sandbox_end_to_end_via_dsl_composition_only` (SANDBOX inheriting parent attribution).

### 3.4 — Browser-target persistent storage (Compromise #19 / #20)

**Status:** Partially closed at Phase-3 G18-A wave-5a. `docs/SECURITY-POSTURE.md` Compromise #19 (browser persistent storage threat model) + Compromise #20 (cross-browser determinism CI cadence) carry the narrative + remaining gap.

### 3.5 — Module manifest signing (Compromise #21)

**Defense:** Closed at Phase-3 G14-C wave-4b. Signed module manifests + verification on install. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #21.

**Test pin:** `crates/benten-engine/tests/manifest_signing.rs` + `crates/benten-engine/tests/manifest_temporal_binding.rs` + `crates/benten-engine/tests/module_manifest_signature_field_reserved.rs`.

### 3.6 — Module-bytes durability (Compromise #17 / #18)

**Defense:** Closed at Phase-3 G14-C wave-4b. Durable module-bytes + handler-version chain in `redb`-backed stores. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #17 + Compromise #18.

### 3.7 — Read-capability re-verification at resume (Compromise #10)

**Defense:** Closed at Phase-2b G12-E (cross-process metadata arm) + Phase-3 G14-D wave-5a (engine-side asymmetry arm). Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #10.

### 3.8 — IVM views read-gate granularity (Compromise #11)

**Defense:** Closed at Phase-3 G15-A wave-5a (sec-r1-5 closure). Per-edge read-cap check at IVM-view-result-publication time. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #11.

### 3.9 — Symmetric-None transport posture (Compromise #2)

**Defense:** Closed via the diagnostic capability (Option C) approach. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #2.

### 3.10 — No engine-ingress write rate-limits (Compromise #5)

**Defense:** Mitigated-by-recording (engine-side WRITE counter metric exists; no rate-limit enforcement). Per-call SANDBOX bounds (fuel / wallclock / memory / output) + UCAN cap-set time-windows defend SANDBOX + cap-grounded write paths; engine-ingress (transport / napi / DSL) writes have no per-source rate limit. **Status:** Open architectural; **Revisit at v1-window** per `docs/future/phase-3-backlog.md` §10.2. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #5.

### 3.11 — BLAKE3 128-bit effective collision resistance (Compromise #6)

**Defense:** Mitigated-by-construction (multihash `0x1e` BLAKE3-256 produces 128-bit effective collision resistance via Wagner's birthday bound; treated as the architectural floor for content-addressing identity). The 128-bit floor is sufficient for the personal-AI-assistants threat model targeted by Phases 1-3; cross-Atrium adversarial collision-search is bounded by Phase-3 cap-policy + UCAN-chain attribution. **Status:** Open architectural bound; **Revisit at v1-window** per `docs/future/phase-3-backlog.md` §10.3. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #6.

### 3.12 — System-zone reserved-prefix DX rejection surface (Compromise #13)

**Defense:** Mitigated-by-DX-only (no security gap). Inv-11 enforces system-zone reserved-prefix rejection at the WRITE registration boundary; the open compromise is about DX surfacing of the rejection (typed-error message clarity, not enforcement gap). Reserved-prefix tampering is blocked at Inv-11 + Inv-13 row-4b sync-receive divergent-CID classifier. **Status:** Open DX gap; **Revisit at v1-window** per `docs/future/phase-3-backlog.md` §10.4. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #13.

### 3.13 — SANDBOX cold-start cost (no opt-in pool) (Compromise #14)

**Defense:** Performance-only / no security gap. SANDBOX cold-start latency (D22 thresholds in `docs/SANDBOX-LIMITS.md`) is performance-budget concern; no security primitive depends on instance-pooling. **Status:** Open performance; **Revisit at v1-window** per `docs/future/phase-3-backlog.md` §10.5. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #14.

### 3.14 — `register_runtime` Phase-8 deferral (Compromise #15)

**Defense:** Phase-8-deferred named destination (marketplace). The reserved-with-deferred-error API surface returns `EvalError::SubsystemDisabled` at registration; no runtime path exposes uninitialized state. **Status:** Deferred to Phase 8 (marketplace). Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #15.

### 3.15 — iroh-relay public metadata leakage (Compromise #22)

**Defense:** Phase-7-deferred named destination (Garden-relay closure path). Honestly disclosed at Phase-3 close; cross-referenced at §2.5 above (MR-1..MR-3 attack vectors). **Status:** Open; **Revisit at v1-window** per `docs/future/phase-3-backlog.md` §10 (Phase-7 Garden-relays primary closure path; Phase-9 hardened-deployment fallback). Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #22; matrix §2.5.

### 3.16 — Wire device-attestation envelope cryptographic closure (Compromise #23)

**Defense:** **CLOSED at Phase-3 G16-D wave-6b fix-pass** (cryptographic shape; operator-deployment `FreshnessPolicy` override REQUIRED for production). DeviceAttestationEnvelope V2 binds Ed25519 envelope-signature + parent-chain Acceptor::accept_at + BLAKE3 payload-hash binding. Cross-referenced at §2.4 (DA-1..DA-9 attack vectors) + §2.5 cross-refs. Three defense modes: (a) DID forgery rejection via envelope-signature; (b) replay rejection via Acceptor freshness window + nonce store; (c) frame-pair binding via constant-time `BLAKE3(payload) == envelope.payload_hash` check. Operator-deployment residual: production deployments must override `Engine::set_acceptor` with concrete time-bound `FreshnessPolicy` BEFORE participating in adversarial sync. Cross-reference: `docs/SECURITY-POSTURE.md` Compromise #23; matrix §2.4 + §2.5.

---

## Audit instructions (R6 phase-close completeness check)

When this matrix is consumed at Phase-3 R6 phase-close (or any later phase-close):

1. **Walk every row.** For each row, verify the cited test pin still exists at HEAD via `grep -n '<test_fn>' <cited_path>`. If grep returns empty: the row is a finding (per pim-16 §3.5e disposition-verification at HEAD).
2. **Walk every "Defended" claim.** For each fully-wired claim, verify the production defense site exists at HEAD. The matrix cites file:symbol-level destinations; bare line cites are forbidden per §3.5b HARDENED point 3 high-churn surface rule.
3. **Walk every "Mitigated-by-construction" claim.** Verify the named structural choice still holds (e.g. ESC-14's "engine ignores embedded WASM custom sections" — is there code that consumes custom sections for cap purposes? If yes, the mitigation is broken.)
4. **Walk every "Phase-N-deferred" claim.** Verify the destination doc/section actually exists at HEAD and contains the matrix's claimed entry. Per HARD RULE rule-12 clause-b: phantom destinations are forbidden.
5. **Cross-check against `docs/SECURITY-POSTURE.md` named compromises.** Any compromise without a row in this matrix that names its closure-status is a finding.

The completeness-audit cycle is the matrix's primary load-bearing role. Per `docs/future/phase-3-backlog.md` §7.13: *"the matrix's role is meta-completeness at R6 phase-close (a checklist that every named attack surface has at least one test pin driving it), NOT the R5 implementation target itself."*

---

## Cross-references

- **`docs/SECURITY-POSTURE.md`** — authoritative compromise narrative + closure status (this doc cross-references; SECURITY-POSTURE is single-source-of-truth for the per-compromise prose).
- **`docs/SANDBOX-LIMITS.md`** — runtime limits feeding the SANDBOX defenses (fuel / wallclock / memory / output bounds; cold-start tier).
- **`docs/HOST-FUNCTIONS.md`** — host-fn surface (the subset capability-derived per `host-functions.toml`).
- **`docs/INVARIANT-COVERAGE.md`** — Inv-1..14 enforcement state (engine-layer invariants feeding into §3.2 + §3.3 above).
- **`docs/ERROR-CATALOG.md`** — typed error catalog (every defense surfaces a typed error; the catalog is the operator-UX surface).
- **`crates/benten-eval/tests/sandbox_escape_attempts_denied.rs`** — Phase-2b ESC-1..16 test corpus (authoritative test pins for Part 1).
- **`crates/benten-eval/tests/sandbox_esc_runtime_arms_e2e.rs`** — Phase-3 wave-5c end-to-end ESC runtime-arm pins (ESC-7 / -9 / -13 / -16 fully-wired closure).
- **`crates/benten-eval/tests/sandbox_stack_overflow.rs`** — Phase-3 G17-A1 wave-5b ESC-5 dedicated typed variant pins.
- **`docs/future/phase-3-backlog.md` §7.13** — origin entry for this matrix authoring task.

---

*Authored Phase-3 R5 wave-9 W9-T2 closing `docs/future/phase-3-backlog.md` §7.13 sec-r4r2-2 / sec-r4r1-4. Status `FINAL` for the Phase-3-close completeness audit; future phase-close cycles re-issue with new rows for any phase-N-introduced attack surface (audit cycle instructions above).*
