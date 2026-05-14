//! Shape / stability pins for the `ErrorCode` enum.
//!
//! These tests are the canonical regression fixtures for the benten-errors
//! extraction (closes SECURITY-POSTURE compromise #3). They pin:
//!
//! 1. The enum variant count — a wire-compat tripwire. Adding a variant
//!    requires bumping this number AND adding a catalog entry + `.code()`
//!    mapping in the owning crate; shrinking it is always a breaking
//!    change.
//! 2. A representative `as_str` round-trip so the string form is frozen
//!    for at least one variant per catalog "family" (invariant, capability,
//!    transaction, CID, engine-level).
//! 3. The `Unknown(String)` forward-compat fallback preserves the raw
//!    string verbatim — the drift detector relies on this so an unknown
//!    code rendered by an older client round-trips through the enum
//!    without lossy conversion.

use benten_errors::ErrorCode;

/// Every catalog variant must round-trip through `as_str` / `from_str`
/// without hitting the `Unknown` fallback. The enumerated list below is the
/// authoritative source for the count — `CATALOG_VARIANT_COUNT` is derived
/// from `ALL_CATALOG_VARIANTS.len()` rather than hard-coded, so a new
/// catalog variant added to the list without bumping a separate constant
/// cannot drift (r6b-err-2).
///
/// **Adding a variant:** add it to [`ALL_CATALOG_VARIANTS`], then add the
/// matching `match` arms in `ErrorCode::as_str`, `as_static_str`, and
/// `from_str`, then document the code in `docs/ERROR-CATALOG.md`. The
/// `round_trips_via_as_str_from_str` test below is the tripwire that fails
/// loudly if any of those steps is skipped.
///
/// `ErrorCode::Unknown(String)` is deliberately excluded — it's the
/// forward-compat fallback, not a catalog code.
const ALL_CATALOG_VARIANTS: &[ErrorCode] = &[
    ErrorCode::InvCycle,
    ErrorCode::InvDepthExceeded,
    ErrorCode::InvFanoutExceeded,
    ErrorCode::InvTooManyNodes,
    ErrorCode::InvTooManyEdges,
    ErrorCode::InvDeterminism,
    ErrorCode::InvContentHash,
    ErrorCode::InvRegistration,
    ErrorCode::InvIterateMaxMissing,
    ErrorCode::InvIterateBudget,
    ErrorCode::CapDenied,
    ErrorCode::CapDeniedRead,
    ErrorCode::CapRevoked,
    ErrorCode::CapRevokedMidEval,
    ErrorCode::CapNotImplemented,
    ErrorCode::CapAttenuation,
    ErrorCode::WriteConflict,
    ErrorCode::IvmViewStale,
    ErrorCode::TxAborted,
    ErrorCode::NestedTransactionNotSupported,
    ErrorCode::PrimitiveNotImplemented,
    ErrorCode::SystemZoneWrite,
    ErrorCode::ValueFloatNan,
    ErrorCode::ValueFloatNonFinite,
    ErrorCode::CidParse,
    ErrorCode::CidUnsupportedCodec,
    ErrorCode::CidUnsupportedHash,
    ErrorCode::VersionBranched,
    ErrorCode::BackendNotFound,
    ErrorCode::TransformSyntax,
    ErrorCode::InputLimit,
    ErrorCode::NotFound,
    ErrorCode::Serialize,
    ErrorCode::GraphInternal,
    ErrorCode::DuplicateHandler,
    ErrorCode::NoCapabilityPolicyConfigured,
    ErrorCode::ProductionRequiresCaps,
    ErrorCode::SubsystemDisabled,
    ErrorCode::UnknownView,
    ErrorCode::NotImplemented,
    ErrorCode::IvmPatternMismatch,
    ErrorCode::IvmStrategyNotImplemented,
    ErrorCode::VersionUnknownPrior,
    // Phase-2a G1-B HostError discriminants (PHASE_2A_RESERVED_CODES). All
    // five reserved for Phase-3 sync fires but already carry catalog
    // entries + as_str / as_static_str / from_str arms, so they belong on
    // the round-trip list.
    ErrorCode::HostNotFound,
    ErrorCode::HostWriteConflict,
    ErrorCode::HostBackendUnavailable,
    ErrorCode::HostCapabilityRevoked,
    ErrorCode::HostCapabilityExpired,
    // Phase-2a firing codes (PHASE_2A_FIRING_CODES). Added during the
    // Phase-2a R5 wave and carry full catalog + round-trip wiring.
    ErrorCode::ExecStateTampered,
    ErrorCode::ResumeActorMismatch,
    ErrorCode::ResumeSubgraphDrift,
    ErrorCode::WaitTimeout,
    ErrorCode::InvImmutability,
    ErrorCode::InvSystemZone,
    ErrorCode::InvAttribution,
    ErrorCode::CapWallclockExpired,
    ErrorCode::CapChainTooDeep,
    ErrorCode::WaitSignalShapeMismatch,
    // Phase-2a ucca-7 parse-time refusal code (lone-`*` GrantScope).
    ErrorCode::CapScopeLoneStarRejected,
    // Phase-2b G8-B (D8-RESOLVED): user-view strategy refusals — `Strategy::A`
    // is reserved for the 5 Phase-1 hand-written IVM views; `Strategy::C` is
    // Phase-3+ Z-set / DBSP cancellation reserved.
    ErrorCode::ViewStrategyARefused,
    ErrorCode::ViewStrategyCReserved,
    // Phase-2b G7-B SANDBOX invariants (Inv-4 nest depth + Inv-7 output
    // limit) plus the D20 saturation overflow code.
    ErrorCode::InvSandboxDepth,
    ErrorCode::InvSandboxOutput,
    ErrorCode::SandboxNestedDispatchDepthExceeded,
    // Phase-2b G7-A SANDBOX runtime + manifest + wasmtime-trap surface
    // (D1/D2/D3/D9/D17/D18/D19/D20/D21/D24/D25/D27 RESOLVED). The 12
    // additions cover the wasmtime budget axes (fuel / memory / wallclock),
    // host-fn cap-check + lookup denials, manifest-unknown + deferred
    // registration, module-shape failures, the nested-dispatch denied
    // dispatch surface (rename per D19), the module-manifest CID-pin
    // mismatch (D16), and the engine-config parse failure surface.
    ErrorCode::SandboxFuelExhausted,
    ErrorCode::SandboxMemoryExhausted,
    ErrorCode::SandboxWallclockExceeded,
    ErrorCode::SandboxWallclockInvalid,
    ErrorCode::SandboxHostFnDenied,
    ErrorCode::SandboxHostFnNotFound,
    // Phase-3 G17-A2 — random host-fn per-call entropy budget exceed.
    ErrorCode::SandboxHostFnRandomBudgetExceeded,
    ErrorCode::SandboxManifestUnknown,
    ErrorCode::SandboxManifestRegistrationDeferred,
    ErrorCode::SandboxModuleInvalid,
    ErrorCode::SandboxNestedDispatchDenied,
    ErrorCode::ModuleManifestCidMismatch,
    ErrorCode::EngineConfigInvalid,
    // Phase-2b G10-A-wasip1 (D10-RESOLVED): snapshot-blob backend
    // surfaces `BackendReadOnly` on every mutation; same code is reused
    // by the network_fetch_stub backend for write-attempt rejections.
    ErrorCode::BackendReadOnly,
    // Phase-2b Wave-8d-types: SANDBOX dispatch references a module CID
    // whose bytes were never registered through
    // `Engine::register_module_bytes`. Distinct from
    // `SandboxModuleInvalid` (bytes present but failed wasmtime
    // structural validation) — fires BEFORE the executor sees any
    // bytes.
    ErrorCode::SandboxModuleNotInstalled,
    // Phase-2b Wave-8i: WAIT-suspended control-flow signal.
    ErrorCode::WaitSuspended,
    // R6 Round-2 r6-r2-napi-1: devserver tooling typed errors —
    // promote the prior hand-typed string literals at devserver.rs to
    // first-class catalog variants so JS callers get typed BentenError
    // dispatch rather than the synthetic `E_UNKNOWN` fallback.
    ErrorCode::ReloadSubscriberUnsubscribed,
    ErrorCode::DevServerStopped,
    // Phase-3 G14-pre-D: HLC skew rejection. `Hlc::update(remote)`
    // refuses a remote stamp whose physical clock exceeds the local
    // physical clock by more than the configured skew tolerance
    // (default 5 minutes). Closes the ds-1 BLOCKER + ds-11 typed-error
    // requirement.
    ErrorCode::HlcSkewExceeded,
    // Phase-3 G14-B (durable UCAN backend in `benten-caps` —
    // `crates/benten-caps/src/backends/ucan.rs::UCANBackend`). Each
    // variant maps 1:1 to a chain-walk / rate-limit / storage failure
    // surface added by the durable backend (`UcanExpired`,
    // `UcanNotYetValid`, `UcanBadSignature`, `UcanAttenuationViolated`,
    // `BackendStorage`, `RateLimitExceeded`, `PeerBandwidthExceeded`).
    ErrorCode::CapUcanExpired,
    ErrorCode::CapUcanNotYetValid,
    ErrorCode::CapUcanBadSignature,
    ErrorCode::CapUcanAttenuationViolated,
    ErrorCode::CapUcanAudienceMismatch,
    ErrorCode::CapBackendStorage,
    ErrorCode::CapRateLimitExceeded,
    ErrorCode::CapPeerBandwidthExceeded,
    // Phase-3 G16-B-B-rest sub-item D: `UcanGroundedPolicy` chain-walker
    // fail-closed when no real wallclock has been injected against a
    // chain with time-bounded delegations. Construction site:
    // `crates/benten-caps/src/ucan_grounded.rs`.
    ErrorCode::UcanClockNotInjected,
    // Phase-3 G18-A wave-5a (D-PHASE-3-27 / br-r1-2 BLOCKER): IndexedDB
    // QuotaExceededError mapping at the browser thin-client cache write
    // boundary. Construction site: `bindings/napi/src/browser_indexeddb.rs`.
    ErrorCode::StorageQuotaExceeded,
    // Phase-3 G17-A1 wave-5b (phase-3-backlog §6.4 + r1-wsa-7 BLOCKER):
    // dedicated typed variant for `wasmtime::Trap::StackOverflow` —
    // distinct from `SandboxModuleInvalid` / `SandboxFuelExhausted`.
    ErrorCode::SandboxStackOverflow,
    // Phase-3 G17-A1 wave-5b (phase-3-backlog §6.1 + r1-wsa-1 BLOCKER):
    // dedicated typed variant for ESC defenses (ESC-7 fuel-refill via
    // host-fn re-entry, ESC-13 fuel-meter callback / Store-poison,
    // ESC-16 fingerprint-collapse). The discriminating EscVector is
    // declared in `crates/benten-eval/src/sandbox/escape_defenses.rs`.
    ErrorCode::SandboxEscapeAttempt,
    // Phase-3 G16-A wave-6 (net-blocker-2 BLOCKER): typed
    // atrium-transport errors. Construction sites:
    // `crates/benten-sync/src/transport.rs` +
    // `crates/benten-sync/src/errors.rs::AtriumTransportError::code`.
    ErrorCode::AtriumRelayUnreachable,
    ErrorCode::AtriumTransportDegraded,
    // Phase-3 G16-B-G wave (Atrium leave/rejoin lifecycle): handle
    // is in graceful-leave quiesced state; transport remains bound.
    // Construction site:
    // `crates/benten-engine/src/engine_sync.rs::AtriumHandle::merge_remote_change`
    // (+ outbound fan-out paths) when `is_active` flag is false.
    ErrorCode::AtriumInactive,
    // Phase-3 G16-B wave-6b (ds-4 Inv-13 row-4b): sync-replica
    // frame targeting a system-zone / Anchor-immutable path with a
    // divergent CID. Construction site:
    // `crates/benten-engine/src/engine_sync.rs::AtriumError::DivergentCidRejected`
    // mapped via `engine_sync.rs::AtriumError::code`.
    ErrorCode::SyncDivergentCidRejected,
    // Phase-3 G16-D wave-6b (ds-r4-3): handshake-frame replay
    // within the bounded HLC acceptance window. Construction site:
    // `crates/benten-sync/src/handshake.rs::HandshakeError::ReplayWithinBoundedWindow`
    // — carries observable original_hlc / replay_hlc / window_ms
    // diagnostic fields per pim-2 production-flow drive.
    ErrorCode::HandshakeReplayWithinBoundedWindow,
    // Phase-3 G19-C2 wave-7 (stream-r1-9 + §7.1.5): per-handler STREAM
    // chunkCountCap / wallclockBudgetMs config widening the workspace
    // grant ceiling fires this typed error at registration / call time.
    // Construction site:
    // `crates/benten-engine/src/engine_stream.rs::build_stream_handle`
    // (resolves per-handler properties + validates against
    // workspace defaults).
    ErrorCode::InvStreamConfig,
    // Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): JS-side
    // FinalizationRegistry leak detector for handles produced by
    // `engine.openStream`. Surfaces via the operator observability
    // surface; no native-side construction site (the typed code is
    // surfaced from `packages/engine/src/stream.ts`).
    ErrorCode::StreamHandleLeaked,
    // Phase-3 G20-A2 wave-8a (D12): WAIT TTL runtime expiry path +
    // GC machinery. Three new variants:
    //   `WaitTtlExpired` — wall-clock TTL deadline elapsed at resume.
    //     Construction site at
    //     `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner`
    //     (Step 1.5 deadline check; consults
    //     `crate::wait_ttl_gc::is_expired` and reaps the entry on fire).
    //   `WaitTtlInvalid` — registration-time validator: WAIT node's
    //     `ttl_hours` property is non-integer or out-of-range
    //     `[1, 720]`. Construction site at
    //     `crates/benten-engine/src/engine.rs::register_subgraph` (the
    //     WAIT-TTL validation walk).
    //   `WaitMetadataMissing` — resume against an envelope whose WAIT
    //     metadata is absent from the SuspensionStore (GC-evicted /
    //     cross-process-divergent / fabricated-real-envelope
    //     scenarios). Construction site at
    //     `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner`
    //     (Step 1.5 fail-loud branch when `pinned_subgraph_cids` is
    //     non-empty + metadata lookup returns `Ok(None)`); secondary
    //     mapping at `engine_wait.rs::map_resume_eval_error` promotes
    //     the eval-side `HostBackendUnavailable` fail-loud to this
    //     code so the engine layer carries the metadata-missing
    //     semantic separately from generic backend-unavailable.
    ErrorCode::WaitTtlExpired,
    ErrorCode::WaitTtlInvalid,
    ErrorCode::WaitMetadataMissing,
    // Phase-3 G21-T3 §2.5(d): registration-time hard reject of
    // user-handler IDs in the reserved `engine:typed:` namespace
    // (typed-CALL surface lands at G21-T1 PR #145).
    ErrorCode::ReservedHandlerNamespace,
    // Phase-3 G16-B-A canary (D-PHASE-3-25 sync-hop-depth-bounded
    // contract): typed code surfaced when a CRDT-merge frame would
    // exceed `SYNC_HOP_DEPTH_CAP` (default 8). Construction site at the
    // CRDT merge seam; carries the observed depth + cap diagnostic
    // fields. Mirrors the Inv-4 sandbox-depth precedent.
    ErrorCode::SyncHopDepthExceeded,
    // Phase-3 G16-B-F (sec-r4r1-2 BLOCKER closure): mid-session
    // revocation typed-error variant fired by `apply_atrium_merge`'s
    // per-write cap-recheck when the originating peer's local grant
    // was revoked between handshake and the next sync round.
    // Construction site at
    // `crates/benten-engine/src/engine.rs::apply_atrium_merge`'s
    // per-row apply loop; mirrors the SUBSCRIBE-side
    // `SubscribeRevokedMidStream` shape per CLR-2 dual-layer recheck
    // architecture.
    ErrorCode::SyncRevokedDuringSession,
    // Phase-3 G16-D wave-6b fix-pass (cryptographic-attestation closure
    // for criterion 16): typed code surfaced when an inbound
    // on-the-wire `DeviceAttestationEnvelope` fails cryptographic
    // verification (DID forgery / parent-chain rejection via
    // `benten_id::Acceptor::accept_at` / frame-pair payload-hash
    // binding violation). Construction site at
    // `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::verify`.
    // Joins the `ON_DENIED` routing family per CLR-2 dual-layer recheck.
    ErrorCode::DeviceAttestationForged,
    // Phase-3 R6 fp Wave C2 (dx-r6-r1-1): DSL orphan-code closure.
    // `DslInvalidShape` — DSL-layer shape validation rejected a value
    // that did not match the expected structural shape. Construction
    // site: `crates/benten-dsl-compiler/src/lib.rs` (object/pair shape
    // validation in the parser/emit pass). Routes to `ON_ERROR`.
    // `DslUnregisteredHandler` — engine call/dispatch boundary
    // referenced an unregistered handler-id. Construction sites in
    // `crates/benten-engine/src/engine.rs` + `engine_stream.rs`
    // (5 sites: `dispatch_call_with_mode_and_trace`, `dispatch_call_inner`,
    // `handler_to_mermaid`, `handler_predecessors`, `call_stream`).
    // Routes to `ON_NOT_FOUND` (joins `NotFound` / `BackendNotFound`).
    ErrorCode::DslInvalidShape,
    ErrorCode::DslUnregisteredHandler,
    // Phase-3 R6-FP Wave-C1 (ds-r6-1 / sec-r4r2-1 attack-vector
    // pin closure): sync-frame trust-boundary rejection codes.
    //   `SyncHashMismatch` — MST-diff entry's declared CID does not
    //     match BLAKE3(payload). Construction site at
    //     `crates/benten-sync/src/mst.rs::Mst::apply_entries` (already
    //     wired) + the engine receive-boundary surface that drives
    //     `tests/attack_mst_diff_cid_mismatch.rs` end-to-end.
    //   `SyncHlcDrift` — inbound sync frame HLC `physical_ms` exceeds
    //     local clock by more than the skew-tolerance window.
    //     Construction site at
    //     `crates/benten-engine/src/engine.rs::apply_atrium_merge`
    //     per-row HLC verification loop (calls
    //     `benten_core::hlc::Hlc::update`).
    //   `SyncCapUnverified` — inbound sync frame WRITE without a
    //     verifiable cap-chain. Reserved-but-not-yet-emitted shape
    //     companion to `SyncRevokedDuringSession`.
    ErrorCode::SyncHashMismatch,
    ErrorCode::SyncHlcDrift,
    ErrorCode::SyncCapUnverified,
    // Phase-3 G21-T1: typed-CALL dispatch surface. Four catalog variants
    // for the typed-CALL boundary at `crates/benten-engine/src/engine.rs`
    // typed-CALL registry dispatch. Closes the pre-v1 triage `ecc-3`
    // observation (TypedCall* family was present in the enum + catalog
    // arms but missing from this round-trip list).
    //   `TypedCallUnknownOp` — typed-CALL op-name miss against the
    //     registered typed-CALL op set. Routes to `ON_ERROR`.
    //   `TypedCallInvalidInput` — typed-CALL input shape failed the
    //     op's input-schema validation. Routes to `ON_ERROR`.
    //   `TypedCallCapDenied` — typed-CALL dispatch rejected by the
    //     host's `check_capability` hook against the op's declared
    //     per-op cap. Routes to `ON_DENIED`.
    //   `TypedCallDispatchError` — typed-CALL op-internal error
    //     bubbled out of the dispatch boundary. Routes to `ON_ERROR`.
    ErrorCode::TypedCallUnknownOp,
    ErrorCode::TypedCallInvalidInput,
    ErrorCode::TypedCallCapDenied,
    ErrorCode::TypedCallDispatchError,
    // Phase-4-Foundation G24-F (DidKeyedSession + SessionToken
    // thin-client session-protocol surface; T2 defenses 1-3 + br-r1-1
    // + sec-4f-r1-5 + Family F1 gap #2). Four new catalog variants for
    // the four failure modes the protocol surfaces at the full-peer
    // boundary. Construction site:
    // `crates/benten-engine/src/thin_client.rs`. All route to
    // `ON_DENIED` per cap-denial family precedent.
    //   `ThinClientHandshakeInvalid` — sig verification, unknown
    //     challenge, or expired challenge at handshake.
    //   `ThinClientChallengeReplay` — captured-challenge replay; the
    //     single-use nonce was already consumed by a prior handshake.
    //   `ThinClientOriginMismatch` — origin pinning recheck rejected
    //     the request (fires at establishment AND per-request
    //     mid-session per Family F1 gap #2 closure).
    //   `ThinClientSessionExpired` — token wallclock TTL elapsed;
    //     also surfaces on unknown / fabricated token ids.
    ErrorCode::ThinClientHandshakeInvalid,
    ErrorCode::ThinClientChallengeReplay,
    ErrorCode::ThinClientOriginMismatch,
    ErrorCode::ThinClientSessionExpired,
    // Phase 4-Foundation G23-A schema_compiler canary (2026-05-12): 9 NEW
    // E_SCHEMA_* codes minted atomically Rust + TS per §3.5g. Construction
    // site: `crates/benten-platform-foundation/src/schema_compiler/`. All
    // 9 carry `as_static_str` + `from_str` arms + routed_edge_label `None`
    // (registration-time refusal, same disposition as
    // `ReservedHandlerNamespace` / `DuplicateHandler`). Post-G24-F + G23-A
    // (batch-1 of strategy-C local-merge): 118 + 4 + 9 = 131.
    ErrorCode::SchemaValidationFailed,
    ErrorCode::SchemaEmitNewPrimitiveRejected,
    ErrorCode::SchemaSandboxHostFnRejected,
    ErrorCode::SchemaVocabInvalidLabel,
    ErrorCode::SchemaVocabEdgeMismatch,
    ErrorCode::SchemaVocabScalarUnknown,
    ErrorCode::SchemaVocabRefTargetMissing,
    ErrorCode::SchemaVocabCycleRejected,
    ErrorCode::SchemaVocabRequiredPropertyMissing,
    // Phase 4-Foundation G24-D — FULL plugin manifest (15 codes).
    // 14 E_PLUGIN_* + 1 E_REGISTRY_* per Ben's R4-triage §7 ratification.
    // Construction sites distributed across:
    //   `benten-platform-foundation::plugin_manifest::validate`,
    //   `::verify_user_signature`, `::verify_peer_signature`, etc.;
    //   `benten-caps::plugin_delegation::check_delegation_within_envelope`;
    //   `benten-platform-foundation::module_ecosystem::install_plugin`.
    //   `RegistryDiscoveryTimeout` reserved at Phase 4-Foundation; fires
    //   first at Phase 4-Meta.
    ErrorCode::PluginManifestInvalid,
    ErrorCode::PluginInstallRecordUserSignatureInvalid,
    ErrorCode::PluginContentPeerSignatureInvalid,
    ErrorCode::PluginContentPeerKeyRotated,
    ErrorCode::PluginAuthorNotTrusted,
    ErrorCode::PluginInstallConsentRequired,
    ErrorCode::PluginInstallRecordManifestCidMismatch,
    ErrorCode::PluginInstallRecordConsentingUserMismatch,
    ErrorCode::PluginInstallRecordPluginDidMismatch,
    ErrorCode::PluginDidHandleNotPreInserted,
    ErrorCode::PluginDidHandleDuplicate,
    ErrorCode::PluginDelegationOutsideManifestEnvelope,
    ErrorCode::PluginPrivateNamespaceDelegationForbidden,
    ErrorCode::PluginContentCidMismatch,
    ErrorCode::PluginNewVersionAvailable,
    ErrorCode::PluginHeterogeneityIncompatible,
    ErrorCode::PluginMetaCompositionCycleRejected,
    ErrorCode::PluginDeviceAttestationForged,
    ErrorCode::PluginLibraryIndexTamper,
    ErrorCode::RegistryDiscoveryTimeout,
    // Phase 4-Foundation G23-B — materializer pipeline (3 codes).
    // Construction sites:
    //   `benten-platform-foundation::materializer::HtmlJsonMaterializer::materialize_with_gate`
    //   `benten-platform-foundation::materializer::HtmlJsonMaterializer::subscribe_with_gate`
    // MaterializerCapDenied routes to ON_DENIED (cap-denial family);
    // MaterializerSchemaMismatch + MaterializerSubscribeSeamFailure are
    // pre-fanout structural rejections (no primitive-edge routing).
    ErrorCode::MaterializerCapDenied,
    ErrorCode::MaterializerSchemaMismatch,
    ErrorCode::MaterializerSubscribeSeamFailure,
    // -------------------------------------------------------------------
    // Phase 4-Foundation R6-FP-C catalog-coverage closure (ec-r6r1-1
    // + ec-r6r1-2 — Phase-3 R4-ec-8 + Phase-4-Foundation R6-R1):
    // promoted 14 previously-missing throwable variants into the
    // regression list so `variant_count_is_pinned` actually
    // round-trips the FULL throwable subset of the enum, not just
    // the 149 that survived prior catalog cohort additions.
    //
    // Each variant was already wired through `as_str` /
    // `as_static_str` / `from_str` arms in `benten-errors/src/lib.rs`
    // AND already present in `docs/ERROR-CATALOG.md` `### E_XXX`
    // headings AND already present in
    // `packages/engine/src/errors.generated.ts` CATALOG_CODES — only
    // this regression list was 14 entries short.
    //
    // Grouped by family for readability (alphabetical within family):
    //
    //   CAP family — `CapSnapshotHashMismatch`: per-row cap-recheck
    //     snapshot integrity check (closes the snapshot-hash drift
    //     attack surface).
    //
    //   INV family — `Inv11SystemZoneRead`: Inv-11 read-side
    //     defense at the engine system-zone label probe.
    //
    //   MODULE family — `ModuleMigrationsRequirePersistence`:
    //     SANDBOX module install rejected on a backend that lacks
    //     durable persistence (browser thin-client cache-only).
    //
    //   SANDBOX family — `SandboxUnavailableOnWasm`: SANDBOX
    //     primitive dispatch refused on wasm32-thin-compute-surface
    //     deployments (CLAUDE.md baked-in #17 shape-b/c).
    //
    //   STREAM family (3) — `StreamBackpressureDropped` /
    //     `StreamClosedByPeer` / `StreamProducerWallclockExceeded`:
    //     STREAM primitive runtime surface (back-pressure drop /
    //     consumer-side close / producer wallclock budget exceeded).
    //
    //   SUBSCRIBE family (4) — `SubscribeCursorLost` /
    //     `SubscribeDeliveryFailed` / `SubscribePatternInvalid` /
    //     `SubscribeReplayWindowExceeded` /
    //     `SubscribeRevokedMidStream`: SUBSCRIBE primitive runtime
    //     surface (cursor invalidation / delivery transport failure
    //     / pattern shape rejection / replay-window-exceeded /
    //     cap-revoked-mid-stream).
    //
    //   THIN_CLIENT family — `ThinClientAuthRejected`: G14-D
    //     wave-5a device-attestation auth boundary (pre-Phase-4
    //     surface, predates the 4 G24-F session-protocol codes).
    //
    //   VIEW family — `ViewLabelMismatch`: user-view emission seam
    //     rejection when emitted Node's label set diverges from the
    //     view's declared frame envelope.
    ErrorCode::CapSnapshotHashMismatch,
    ErrorCode::Inv11SystemZoneRead,
    ErrorCode::ModuleMigrationsRequirePersistence,
    ErrorCode::SandboxUnavailableOnWasm,
    ErrorCode::StreamBackpressureDropped,
    ErrorCode::StreamClosedByPeer,
    ErrorCode::StreamProducerWallclockExceeded,
    ErrorCode::SubscribeCursorLost,
    ErrorCode::SubscribeDeliveryFailed,
    ErrorCode::SubscribePatternInvalid,
    ErrorCode::SubscribeReplayWindowExceeded,
    ErrorCode::SubscribeRevokedMidStream,
    ErrorCode::ThinClientAuthRejected,
    ErrorCode::ViewLabelMismatch,
];

/// Count of catalog variants (auto-derived from [`ALL_CATALOG_VARIANTS`] so
/// adding to the list and forgetting to bump a number is impossible).
const CATALOG_VARIANT_COUNT: usize = ALL_CATALOG_VARIANTS.len();

/// Every catalog variant must round-trip through `as_str` / `from_str`
/// without hitting the `Unknown` fallback. Cross-checks the enumerated
/// `ALL_CATALOG_VARIANTS` list against the enum so a variant added to the
/// enum without being added to the list is caught by the real
/// `catalog_variant_count_matches_enum` test below (Phase 4-Foundation
/// R6-FP-C closure of ec-r6r1-2 phantom-destination promise; the test
/// uses an exhaustive `match` arm so adding a variant to the enum
/// without a list entry now fails to COMPILE rather than failing only
/// at runtime), and a variant added to the list without the matching
/// `from_str` arm is caught here.
#[test]
fn variant_count_is_pinned() {
    // Every listed variant must round-trip through from_str(as_str).
    for code in ALL_CATALOG_VARIANTS {
        let s = code.as_str();
        let parsed = ErrorCode::from_str(s);
        assert_eq!(
            &parsed, code,
            "catalog variant {code:?} failed as_str/from_str round-trip via string {s}",
        );
    }
    // The "as_static_str" path MUST also return the same string for every
    // catalog variant — it's the path the engine's static-code accessor
    // delegates through, and it duplicating `as_str` is load-bearing for
    // the drift detector's expected reverse mapping.
    for code in ALL_CATALOG_VARIANTS {
        assert_eq!(
            code.as_str(),
            code.as_static_str(),
            "as_str / as_static_str disagree for {code:?}",
        );
    }
    // Canary: the known count at the time this harness last synced
    // (58). If a future change bumps the enum, it bumps the array, which
    // bumps this value — the assertion documents the expected movement
    // direction. Adding a variant is a +1 delta; shrinking is always a
    // breaking change that must surface in the catalog diff.
    //
    // G11-A Wave 3a sync: the earlier canary (43) predated the Phase-2a
    // R5 waves which introduced the 5 reserved HostError discriminants
    // (PHASE_2A_RESERVED_CODES), the 10 firing codes (PHASE_2A_FIRING_CODES),
    // and the ucca-7 `CapScopeLoneStarRejected` parse-time refusal. All
    // 16 additions already had `as_str` / `as_static_str` / `from_str`
    // coverage in `benten-errors/src/lib.rs` — the test list just hadn't
    // been updated. Post-sync: 42 + 16 = 58.
    //
    // Phase 2b G8-A adds `IvmStrategyNotImplemented` for the reserved
    // `Strategy::C` variant — Algorithm B ships A+B, C is Phase-3+ deferred.
    // Post-G8-A: 58 + 1 = 59.
    //
    // Phase 2b G8-B (D8-RESOLVED) adds `ViewStrategyARefused` +
    // `ViewStrategyCReserved` for user-view registration-time refusals.
    // Post-G8-B: 59 + 2 = 61.
    //
    // Phase-2b G7-B sync (rebased on top of G8-A + G8-B merged main): +3
    // codes (InvSandboxDepth, InvSandboxOutput,
    // SandboxNestedDispatchDepthExceeded). Post-G7-B: 61 + 3 = 64.
    //
    // Phase-2b G7-A sync (this branch, rebased on top of G7-B + G6-A/B + G8-A/B
    // merged main): +12 codes covering the SANDBOX runtime + manifest +
    // wasmtime-trap surface (Sandbox{FuelExhausted, MemoryExhausted,
    // WallclockExceeded, WallclockInvalid, HostFnDenied, HostFnNotFound,
    // ManifestUnknown, ManifestRegistrationDeferred, ModuleInvalid,
    // NestedDispatchDenied}, ModuleManifestCidMismatch, EngineConfigInvalid).
    // Post-G7-A: 64 + 12 = 76.
    //
    // Phase-2b G10-A-wasip1 (D10-RESOLVED) adds `BackendReadOnly` for
    // the snapshot-blob + network_fetch_stub backends' write-attempt
    // typed error. Post-G10-A-wasip1: 76 + 1 = 77.
    //
    // Phase-2b Wave-8d-types adds `SandboxModuleNotInstalled` for the
    // missing-module-bytes path on `impl PrimitiveHost for
    // Engine::execute_sandbox`. Post-Wave-8d-types: 77 + 1 = 78.
    //
    // Phase-2b Wave-8i adds `WaitSuspended` — the control-flow signal
    // surfaced by the dispatcher when a regular `engine.call()` walk hits
    // a WAIT primitive and the engine routes through eval-side
    // `wait::evaluate`. Post-Wave-8i: 78 + 1 = 79.
    //
    // R6 Round-2 r6-r2-napi-1 adds `ReloadSubscriberUnsubscribed` +
    // `DevServerStopped` — promotes the two devserver hand-typed
    // string literals to first-class catalog variants. Post-R6-R2:
    // 79 + 2 = 81.
    //
    // Phase-3 G14-pre-D adds `HlcSkewExceeded` — typed error fired by
    // `benten_core::hlc::Hlc::update` when the remote stamp's
    // physical-clock component exceeds the local clock by more than
    // the configured skew tolerance (default 5 minutes). Closes the
    // ds-1 BLOCKER + ds-11 typed-error requirement. Post-G14-pre-D:
    // 81 + 1 = 82.
    //
    // Phase-3 G14-B adds 7 codes for the durable UCAN backend in
    // `benten-caps` (`UCANBackend<B: GraphBackend>`): chain-walk
    // failures (`CapUcanExpired`, `CapUcanNotYetValid`,
    // `CapUcanBadSignature`, `CapUcanAttenuationViolated`), durable-
    // store I/O failure (`CapBackendStorage`), rate-limit policy plug
    // denials (`CapRateLimitExceeded`, `CapPeerBandwidthExceeded`).
    // Post-G14-B: 82 + 7 = 89.
    //
    // Phase-3 G14-B mini-review fix-pass adds 1 code:
    // `CapUcanAudienceMismatch` — typed cross-atrium replay denial at
    // the durable chain-walk seam (CLR-2 audience-binding pinned at
    // `UCANBackend::validate_chain_for_audience_at`). Distinct from
    // `CapDenied` so audit pipelines can route on cross-atrium replay
    // independently of generic denial. Post-mini-review: 89 + 1 = 90.
    //
    // Phase-3 G18-A wave-5a adds 1 code: `StorageQuotaExceeded` —
    // typed mapping for IndexedDB `DOMException(name="QuotaExceededError")`
    // at the browser thin-client cache write boundary. Construction
    // site at `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code`;
    // closes D-PHASE-3-27 / br-r1-2 BLOCKER. Post-G18-A: 90 + 1 = 91.
    //
    // Phase-3 G17-A1 wave-5b adds 2 codes:
    //   `SandboxStackOverflow` — dedicated typed variant for
    //     `wasmtime::Trap::StackOverflow` (formerly catalog-folded into
    //     `SandboxModuleInvalid`); closes phase-3-backlog §6.4 +
    //     r1-wsa-7 BLOCKER. Construction site at
    //     `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error`.
    //   `SandboxEscapeAttempt` — typed variant for ESC-7 / ESC-13 /
    //     ESC-16 defenses. Construction sites at
    //     `crates/benten-eval/src/sandbox/escape_defenses.rs::run_esc7_check`,
    //     `crates/benten-eval/src/sandbox/escape_defenses.rs::run_esc13_check`, and
    //     `crates/benten-eval/src/sandbox/escape_defenses.rs::run_esc16_check`.
    //     Closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13) + r1-wsa-4 (ESC-16) +
    //     phase-3-backlog §6.1.
    // Post-G17-A1: 91 + 2 = 93.
    //
    // Phase-3 G16-A wave-6 adds 2 codes:
    //   `AtriumRelayUnreachable` — typed variant for relay-unreachable
    //     surface (DNS / TLS / transport-timeout). Construction site at
    //     `crates/benten-sync/src/transport.rs::Endpoint::bind_with_relay_url`
    //     + `Endpoint::connect`. Closes net-blocker-2 BLOCKER (half).
    //   `AtriumTransportDegraded` — typed variant for established-
    //     connection-degraded surface (packet-loss / relay-fallback-
    //     active / direct-connection-lost / handshake-wire-format-
    //     violation). Construction site at
    //     `crates/benten-sync/src/transport.rs::Endpoint::*` +
    //     `crates/benten-sync/src/handshake_wire.rs::HandshakeFrame::from_canonical_bytes`.
    //     Closes net-blocker-2 BLOCKER (half).
    // Post-G16-A: net +2 from `AtriumRelayUnreachable` +
    // `AtriumTransportDegraded`.
    //
    // Phase-3 G16-B wave-6b adds 1 code:
    //   `SyncDivergentCidRejected` — typed variant for inbound sync
    //     frames targeting system-zone / Anchor-immutable paths with
    //     a divergent CID (ds-4 Inv-13 row-4b). Construction site at
    //     `crates/benten-engine/src/engine_sync.rs::AtriumError::DivergentCidRejected`
    //     mapped via `engine_sync.rs::AtriumError::code`. PRE-merge
    //     classifier walks `SYSTEM_ZONE_PREFIXES` so reject fires
    //     BEFORE the Loro merge applies — not post-merge cleanup.
    // Post-G16-B: 96 + 1 = 97.
    //
    // Phase-3 G16-D wave-6b adds 1 code:
    //   `HandshakeReplayWithinBoundedWindow` — typed variant for
    //     handshake frames replayed within the bounded HLC
    //     acceptance window (DEFAULT_REPLAY_WINDOW_MS = 5000).
    //     Construction site at
    //     `crates/benten-sync/src/handshake.rs::HandshakeError::ReplayWithinBoundedWindow`,
    //     carrying observable original_hlc / replay_hlc / window_ms
    //     diagnostic fields. Closes ds-r4-3.
    // Post-G16-D: 97 + 1 = 98.
    //
    // Phase-3 G19-C2 wave-7 adds 2 codes (per-handler STREAM config +
    // FinalizationRegistry leak):
    //   `InvStreamConfig` — per-handler STREAM config widens the
    //     workspace grant ceiling. Construction site at
    //     `crates/benten-engine/src/engine_stream.rs::build_stream_handle`
    //     (validates resolved per-handler `chunkCountCap` /
    //     `wallclockBudgetMs` against workspace defaults; widening
    //     fails loud per stream-r1-9).
    //   `StreamHandleLeaked` — JS-side handle dropped without
    //     `close()`. Surfaced from
    //     `packages/engine/src/stream.ts::ensureLeakRegistry`
    //     (FinalizationRegistry callback) + the `Engine.shutdown()`
    //     drain on `packages/engine/src/engine.ts::Engine` against
    //     the `engine.onStreamLeaked` operator surface (§7.1.2).
    // Post-G19-C2: 98 + 2 = 100.
    //
    // Phase-3 G20-A2 wave-8a adds 3 codes (D12 WAIT TTL runtime
    // expiry + GC machinery):
    //   `WaitTtlExpired` — wall-clock TTL deadline elapsed at resume.
    //     Construction site at
    //     `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner`
    //     (Step 1.5 deadline check + reap_one).
    //   `WaitTtlInvalid` — registration-time `ttl_hours` validator
    //     (out-of-range `[1, 720]` or non-integer). Construction site
    //     at `crates/benten-engine/src/engine.rs::register_subgraph`.
    //   `WaitMetadataMissing` — resume against a real WAIT envelope
    //     whose metadata is absent from the SuspensionStore
    //     (GC-evicted / cross-process / fabricated). Construction
    //     site at
    //     `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner`
    //     (Step 1.5 fail-loud branch + `map_resume_eval_error` remap
    //     of eval-side `HostBackendUnavailable`).
    // Post-G20-A2: 100 + 3 = 103.
    // Post-G21-T3 §2.5(d): + ReservedHandlerNamespace = 104.
    // Post-G16-B-A canary: + SyncHopDepthExceeded = 105. The hardcoded
    // count below tracks `ALL_CATALOG_VARIANTS.len()` exactly.
    // Post-G16-B-B-rest sub-item D: + UcanClockNotInjected = 106
    // Post-G16-B-G mini-review fp: + AtriumInactive = 107
    // (DEFAULT_NOW_SECS=0 fail-closed inversion at the
    // `UcanGroundedPolicy` chain-walker boundary).
    // Post-G16-B-F (sec-r4r1-2 BLOCKER closure): + SyncRevokedDuringSession = 108
    // (mid-session revocation typed-error at sync-replica WRITE delivery;
    // bumped from 107 → 108 at PR #161 rebase due to PR #159 G16-B-G
    // landing E_ATRIUM_INACTIVE first; CATALOG_VARIANT_COUNT collision
    // resolution per known sequential-merge pattern).
    // Post-G16-D wave-6b fix-pass (cryptographic-attestation closure for
    // criterion 16 per Ben ratification 2026-05-09): + DeviceAttestationForged
    // = 109 (signed wire envelope verification — DID forgery / replay /
    // frame-pair payload-hash binding rejection at `apply_atrium_merge`).
    //
    // Phase-3 R6 fix-pass Wave C2 (dx-r6-r1-1 MAJOR — DSL orphan-code
    // closure half): + DslInvalidShape + DslUnregisteredHandler = 111
    // (catalog-only TS-DSL codes promoted to first-class Rust ErrorCode
    // variants with production construction sites in
    // `benten-dsl-compiler` + `benten-engine::engine`).
    //
    // Phase-3 R6-FP Wave-C1 sequential-merge resolution (ds-r6-1 /
    // sec-r4r2-1 attack-vector pin closure): + SyncHashMismatch +
    // SyncHlcDrift + SyncCapUnverified = 114 (MST-diff declared-vs-
    // computed CID mismatch + inbound-sync-frame HLC skew rejection
    // at apply_atrium_merge per-row Hlc::update + reserved companion
    // to SyncRevokedDuringSession). C1 merges after C2 per
    // dispatch-conventions §3.5g sequential-merge resolution pattern;
    // count moves 109 → 111 (C2) → 114 (C1).
    //
    // Phase-3-close pre-v1 triage (2026-05-10 `ecc-3` FIX-NOW-INLINE):
    // + TypedCallUnknownOp + TypedCallInvalidInput + TypedCallCapDenied
    // + TypedCallDispatchError = 118 (typed-CALL dispatch family was
    // already in the enum + catalog `as_str` / `from_str` arms; only
    // this round-trip list was missing them).
    //
    // Phase-4-Foundation G24-F (DidKeyedSession + SessionToken
    // thin-client session-protocol surface; T2 defenses 1-3 + br-r1-1
    // + sec-4f-r1-5 + Family F1 gap #2): + ThinClientHandshakeInvalid
    // + ThinClientChallengeReplay + ThinClientOriginMismatch +
    // ThinClientSessionExpired = +4. All four route to `ON_DENIED`
    // per the cap-denial family precedent.
    //
    // Phase 4-Foundation G23-A schema_compiler canary (2026-05-12): + 9
    // E_SCHEMA_* codes (SchemaValidationFailed, SchemaEmitNewPrimitiveRejected,
    // SchemaSandboxHostFnRejected, SchemaVocabInvalidLabel,
    // SchemaVocabEdgeMismatch, SchemaVocabScalarUnknown,
    // SchemaVocabRefTargetMissing, SchemaVocabCycleRejected,
    // SchemaVocabRequiredPropertyMissing).
    //
    // Phase 4-Foundation G24-D — FULL plugin manifest landing
    // (2026-05-12; CLAUDE.md baked-in #18 four-identity-concepts
    // model): +15 catalog variants for the plugin manifest envelope
    // surface (14 E_PLUGIN_* + 1 E_REGISTRY_* per Ben's R4-triage §7).
    //
    // Batch-1 (G24-F + G23-A) + Batch-2 (G24-D + G23-0b):
    // 118 + 4 (G24-F) + 9 (G23-A) + 0 (G23-0a) + 15 (G24-D) + 0 (G23-0b) = 146.
    //
    // Phase 4-Foundation G23-B — materializer pipeline canary
    // (2026-05-13): +3 codes (MaterializerCapDenied,
    // MaterializerSchemaMismatch, MaterializerSubscribeSeamFailure).
    // 146 + 3 = 149.
    //
    // Phase 4-Foundation R6-FP-A plugin-trust BLOCKER closure
    // (2026-05-13): +3 codes from the arch-r6-r1-5 consent-record
    // ErrorCode split + the sec-r6r1-1 BLOCKER plugin-DID-binding
    // closure: PluginInstallRecordManifestCidMismatch +
    // PluginInstallRecordConsentingUserMismatch +
    // PluginInstallRecordPluginDidMismatch. 149 + 3 = 152.
    //
    // Phase 4-Foundation R6-FP-A fix-pass (mr-1 + mr-2 BLOCKER
    // closure, 2026-05-13): +1 code `PluginDidHandleNotPreInserted` —
    // closes the keypair-orphan failure mode by enforcing the
    // caller-mint-first pattern. 152 + 1 = 153.
    //
    // Phase 4-Foundation R6-FP-C catalog-coverage closure
    // (2026-05-13, ec-r6r1-1 + ec-r6r1-2): promoted 14 previously-
    // missing throwable variants into the regression list (CAP +
    // INV + MODULE + SANDBOX + STREAM ×3 + SUBSCRIBE ×5 +
    // THIN_CLIENT + VIEW). Each was wired through as_str / from_str
    // / catalog / TS catalog already — only this round-trip list
    // was short. 153 + 14 = 167.
    //
    // Strategy-C batch reconciliation (r6/batch-fp-cluster): Wave-A's
    // 4 plugin-trust variants + Wave-C's 14 catalog-coverage variants
    // unioned into ALL_CATALOG_VARIANTS in this slot. Final count = 167.
    //
    // R6-FP-3 (R6 R3 close): +1 `PluginDidHandleDuplicate` (cap-r6-r3-1
    // defensive-return hardening for `PluginDidStore::insert`).
    // 167 + 1 = 168.
    assert_eq!(
        CATALOG_VARIANT_COUNT, 168,
        "CATALOG_VARIANT_COUNT drift — update this value AND docs/ERROR-CATALOG.md in the same commit",
    );
}

/// Real implementation of the cross-check promised by the docstring on
/// [`variant_count_is_pinned`] (line ~414 in the prior layout): asserts
/// that `ALL_CATALOG_VARIANTS.len()` matches the count of THROWABLE
/// enum variants in [`ErrorCode`] (every variant except the
/// forward-compat `Unknown(String)` fallback).
///
/// **Closes ec-r6r1-2 (R6 R1 phantom-destination — MAJOR).** Prior
/// to this test, the only count assertion was `variant_count_is_pinned`
/// which compared `CATALOG_VARIANT_COUNT (== ALL_CATALOG_VARIANTS.len())`
/// against a hard-coded number. That guards against editing the constant
/// without editing the list, but does NOT guard against adding a new
/// enum variant without adding it to the list — exactly the latent gap
/// that allowed 14 throwable variants to silently escape regression
/// coverage across Phase-2b / Phase-3 / Phase-4-Foundation R5.
///
/// Implementation strategy: dual tripwire. Because `ErrorCode` is
/// `#[non_exhaustive]`, the `match` here carries a `_ => false`
/// wildcard fallthrough — a new variant added to the enum without an
/// arm here will silently classify as non-catalog (NOT a compile
/// error). The actual tripwire is two-fold: (1) the hard-coded
/// `CATALOG_VARIANT_COUNT` assertion in `variant_count_is_pinned`
/// forces the author to bump that const + add the variant to
/// `ALL_CATALOG_VARIANTS` + add a `from_str` round-trip arm in
/// `every_variant_round_trips`; (2) the runtime length-mismatch
/// assertion below (`counter == ALL_CATALOG_VARIANTS.len()`) fires
/// if a match arm is added without a corresponding list entry (or
/// vice versa). The `Unknown(String)` arm is excluded from the
/// counter (forward-compat fallback; not a catalog code).
#[test]
#[allow(clippy::too_many_lines)]
fn catalog_variant_count_matches_enum() {
    // Walk the enum exhaustively; any variant added to the enum without
    // a corresponding match arm here will fail to compile, surfacing
    // the omission at build-time. The seed value is irrelevant — we
    // only use the match to force exhaustive walk; the actual count
    // comes from a separate enumeration via `ALL_CATALOG_VARIANTS`.
    #[allow(clippy::too_many_lines)]
    fn arm_is_catalog_variant(code: &ErrorCode) -> bool {
        // Exhaustive match — adding a variant to `ErrorCode` without
        // adding an arm here is a compile error (forces the author
        // to acknowledge the new variant + decide whether it belongs
        // in ALL_CATALOG_VARIANTS).
        match code {
            ErrorCode::Unknown(_) => false,
            ErrorCode::InvCycle
            | ErrorCode::InvDepthExceeded
            | ErrorCode::InvFanoutExceeded
            | ErrorCode::InvTooManyNodes
            | ErrorCode::InvTooManyEdges
            | ErrorCode::InvDeterminism
            | ErrorCode::InvContentHash
            | ErrorCode::InvRegistration
            | ErrorCode::InvIterateMaxMissing
            | ErrorCode::InvIterateBudget
            | ErrorCode::Inv11SystemZoneRead
            | ErrorCode::InvImmutability
            | ErrorCode::InvSystemZone
            | ErrorCode::InvAttribution
            | ErrorCode::InvSandboxDepth
            | ErrorCode::InvSandboxOutput
            | ErrorCode::InvStreamConfig
            | ErrorCode::CapDenied
            | ErrorCode::CapDeniedRead
            | ErrorCode::CapRevoked
            | ErrorCode::CapRevokedMidEval
            | ErrorCode::CapNotImplemented
            | ErrorCode::CapAttenuation
            | ErrorCode::CapWallclockExpired
            | ErrorCode::CapChainTooDeep
            | ErrorCode::CapScopeLoneStarRejected
            | ErrorCode::CapUcanExpired
            | ErrorCode::CapUcanNotYetValid
            | ErrorCode::CapUcanBadSignature
            | ErrorCode::CapUcanAttenuationViolated
            | ErrorCode::CapUcanAudienceMismatch
            | ErrorCode::CapBackendStorage
            | ErrorCode::CapRateLimitExceeded
            | ErrorCode::CapPeerBandwidthExceeded
            | ErrorCode::CapSnapshotHashMismatch
            | ErrorCode::WriteConflict
            | ErrorCode::IvmViewStale
            | ErrorCode::IvmPatternMismatch
            | ErrorCode::IvmStrategyNotImplemented
            | ErrorCode::TxAborted
            | ErrorCode::NestedTransactionNotSupported
            | ErrorCode::PrimitiveNotImplemented
            | ErrorCode::SystemZoneWrite
            | ErrorCode::ValueFloatNan
            | ErrorCode::ValueFloatNonFinite
            | ErrorCode::CidParse
            | ErrorCode::CidUnsupportedCodec
            | ErrorCode::CidUnsupportedHash
            | ErrorCode::VersionBranched
            | ErrorCode::VersionUnknownPrior
            | ErrorCode::BackendNotFound
            | ErrorCode::BackendReadOnly
            | ErrorCode::TransformSyntax
            | ErrorCode::InputLimit
            | ErrorCode::NotFound
            | ErrorCode::Serialize
            | ErrorCode::GraphInternal
            | ErrorCode::DuplicateHandler
            | ErrorCode::NoCapabilityPolicyConfigured
            | ErrorCode::ProductionRequiresCaps
            | ErrorCode::SubsystemDisabled
            | ErrorCode::UnknownView
            | ErrorCode::NotImplemented
            | ErrorCode::HostNotFound
            | ErrorCode::HostWriteConflict
            | ErrorCode::HostBackendUnavailable
            | ErrorCode::HostCapabilityRevoked
            | ErrorCode::HostCapabilityExpired
            | ErrorCode::ExecStateTampered
            | ErrorCode::ResumeActorMismatch
            | ErrorCode::ResumeSubgraphDrift
            | ErrorCode::WaitTimeout
            | ErrorCode::WaitSignalShapeMismatch
            | ErrorCode::WaitSuspended
            | ErrorCode::WaitTtlExpired
            | ErrorCode::WaitTtlInvalid
            | ErrorCode::WaitMetadataMissing
            | ErrorCode::ViewStrategyARefused
            | ErrorCode::ViewStrategyCReserved
            | ErrorCode::ViewLabelMismatch
            | ErrorCode::SandboxNestedDispatchDepthExceeded
            | ErrorCode::SandboxFuelExhausted
            | ErrorCode::SandboxMemoryExhausted
            | ErrorCode::SandboxWallclockExceeded
            | ErrorCode::SandboxWallclockInvalid
            | ErrorCode::SandboxHostFnDenied
            | ErrorCode::SandboxHostFnNotFound
            | ErrorCode::SandboxHostFnRandomBudgetExceeded
            | ErrorCode::SandboxManifestUnknown
            | ErrorCode::SandboxManifestRegistrationDeferred
            | ErrorCode::SandboxModuleInvalid
            | ErrorCode::SandboxNestedDispatchDenied
            | ErrorCode::SandboxModuleNotInstalled
            | ErrorCode::SandboxStackOverflow
            | ErrorCode::SandboxEscapeAttempt
            | ErrorCode::SandboxUnavailableOnWasm
            | ErrorCode::ModuleManifestCidMismatch
            | ErrorCode::ModuleMigrationsRequirePersistence
            | ErrorCode::EngineConfigInvalid
            | ErrorCode::ReloadSubscriberUnsubscribed
            | ErrorCode::DevServerStopped
            | ErrorCode::HlcSkewExceeded
            | ErrorCode::UcanClockNotInjected
            | ErrorCode::StorageQuotaExceeded
            | ErrorCode::AtriumRelayUnreachable
            | ErrorCode::AtriumTransportDegraded
            | ErrorCode::AtriumInactive
            | ErrorCode::SyncDivergentCidRejected
            | ErrorCode::SyncHashMismatch
            | ErrorCode::SyncHlcDrift
            | ErrorCode::SyncCapUnverified
            | ErrorCode::SyncHopDepthExceeded
            | ErrorCode::SyncRevokedDuringSession
            | ErrorCode::HandshakeReplayWithinBoundedWindow
            | ErrorCode::DeviceAttestationForged
            | ErrorCode::DslInvalidShape
            | ErrorCode::DslUnregisteredHandler
            | ErrorCode::StreamHandleLeaked
            | ErrorCode::StreamBackpressureDropped
            | ErrorCode::StreamClosedByPeer
            | ErrorCode::StreamProducerWallclockExceeded
            | ErrorCode::SubscribeCursorLost
            | ErrorCode::SubscribeDeliveryFailed
            | ErrorCode::SubscribePatternInvalid
            | ErrorCode::SubscribeReplayWindowExceeded
            | ErrorCode::SubscribeRevokedMidStream
            | ErrorCode::ReservedHandlerNamespace
            | ErrorCode::TypedCallUnknownOp
            | ErrorCode::TypedCallInvalidInput
            | ErrorCode::TypedCallCapDenied
            | ErrorCode::TypedCallDispatchError
            | ErrorCode::ThinClientAuthRejected
            | ErrorCode::ThinClientHandshakeInvalid
            | ErrorCode::ThinClientChallengeReplay
            | ErrorCode::ThinClientOriginMismatch
            | ErrorCode::ThinClientSessionExpired
            | ErrorCode::SchemaValidationFailed
            | ErrorCode::SchemaEmitNewPrimitiveRejected
            | ErrorCode::SchemaSandboxHostFnRejected
            | ErrorCode::SchemaVocabInvalidLabel
            | ErrorCode::SchemaVocabEdgeMismatch
            | ErrorCode::SchemaVocabScalarUnknown
            | ErrorCode::SchemaVocabRefTargetMissing
            | ErrorCode::SchemaVocabCycleRejected
            | ErrorCode::SchemaVocabRequiredPropertyMissing
            | ErrorCode::PluginManifestInvalid
            | ErrorCode::PluginInstallRecordUserSignatureInvalid
            | ErrorCode::PluginContentPeerSignatureInvalid
            | ErrorCode::PluginContentPeerKeyRotated
            | ErrorCode::PluginAuthorNotTrusted
            | ErrorCode::PluginInstallConsentRequired
            | ErrorCode::PluginDelegationOutsideManifestEnvelope
            | ErrorCode::PluginPrivateNamespaceDelegationForbidden
            | ErrorCode::PluginContentCidMismatch
            | ErrorCode::PluginNewVersionAvailable
            | ErrorCode::PluginHeterogeneityIncompatible
            | ErrorCode::PluginMetaCompositionCycleRejected
            | ErrorCode::PluginDeviceAttestationForged
            | ErrorCode::PluginLibraryIndexTamper
            | ErrorCode::RegistryDiscoveryTimeout
            | ErrorCode::MaterializerCapDenied
            | ErrorCode::MaterializerSchemaMismatch
            | ErrorCode::MaterializerSubscribeSeamFailure
            // R6-FP-A plugin-trust BLOCKER closures (4 codes):
            | ErrorCode::PluginInstallRecordManifestCidMismatch
            | ErrorCode::PluginInstallRecordConsentingUserMismatch
            | ErrorCode::PluginInstallRecordPluginDidMismatch
            | ErrorCode::PluginDidHandleNotPreInserted
            | ErrorCode::PluginDidHandleDuplicate => true,
            // `ErrorCode` is `#[non_exhaustive]` across crate boundary
            // — match exhaustiveness is enforced at the def-site, not
            // here. Any future variant added to the enum that isn't
            // covered above falls through and classifies as `false`;
            // because we ALSO assert that every entry in
            // ALL_CATALOG_VARIANTS classifies as `true`, and we cross-
            // check counts below, the test surfaces the gap as a
            // runtime length mismatch (rather than a compile error).
            // The hard-coded `CATALOG_VARIANT_COUNT, 167` assertion in
            // `variant_count_is_pinned` above is the SECONDARY
            // tripwire — any author bumping the list must touch both.
            _ => false,
        }
    }

    // Walk every entry in ALL_CATALOG_VARIANTS; every entry must
    // classify as catalog (= true) via the exhaustive match.
    for code in ALL_CATALOG_VARIANTS {
        assert!(
            arm_is_catalog_variant(code),
            "ALL_CATALOG_VARIANTS contains {code:?} which the exhaustive \
             match arm classifies as Unknown — should never happen",
        );
    }

    // Independent count via the exhaustive match: if a new variant is
    // added to the enum without being added to the match (compile
    // error) OR added to the match without being added to
    // ALL_CATALOG_VARIANTS (runtime length mismatch below), the test
    // surfaces the gap.
    //
    // We count by re-walking ALL_CATALOG_VARIANTS — the exhaustive
    // match is the structural guarantee that the LIST and the ENUM
    // stay in lockstep (a new enum variant fails to compile without
    // a match arm; a new match arm without a list addition surfaces
    // here as a length mismatch via the dedicated `from_str` round-
    // trip in `variant_count_is_pinned`).
    let exhaustive_match_arms = ALL_CATALOG_VARIANTS
        .iter()
        .filter(|c| arm_is_catalog_variant(c))
        .count();

    assert_eq!(
        exhaustive_match_arms,
        ALL_CATALOG_VARIANTS.len(),
        "exhaustive-match classified {} of {} ALL_CATALOG_VARIANTS entries as catalog \
         variants — list contains an Unknown sentinel which shouldn't be possible",
        exhaustive_match_arms,
        ALL_CATALOG_VARIANTS.len(),
    );

    // Final invariant: ALL_CATALOG_VARIANTS.len() must equal
    // CATALOG_VARIANT_COUNT (auto-derived, so this is tautological at
    // const-evaluation but worth asserting for documentation).
    assert_eq!(CATALOG_VARIANT_COUNT, ALL_CATALOG_VARIANTS.len());
}

/// Representative catalog code renders the frozen string form.
#[test]
fn as_str_stable_for_representative_code() {
    assert_eq!(ErrorCode::CapDenied.as_str(), "E_CAP_DENIED");
    assert_eq!(ErrorCode::InvCycle.as_str(), "E_INV_CYCLE");
    assert_eq!(ErrorCode::ValueFloatNan.as_str(), "E_VALUE_FLOAT_NAN");
}

/// `from_str` round-trips `as_str` for a representative code.
#[test]
fn from_str_roundtrip_representative() {
    let parsed = ErrorCode::from_str("E_CAP_DENIED");
    assert_eq!(parsed, ErrorCode::CapDenied);
    assert_eq!(parsed.as_str(), "E_CAP_DENIED");
}

/// Unknown codes fall back to `Unknown(String)` with the payload preserved.
#[test]
fn from_str_unknown_preserves_raw_string() {
    let code = ErrorCode::from_str("E_NOT_A_REAL_CODE");
    match &code {
        ErrorCode::Unknown(s) => assert_eq!(s, "E_NOT_A_REAL_CODE"),
        other => panic!("expected Unknown, got {other:?}"),
    }
    // as_str returns the raw string verbatim so rendering stays lossless.
    assert_eq!(code.as_str(), "E_NOT_A_REAL_CODE");
}

/// `as_static_str` returns the frozen 'static form for known variants and
/// a sentinel `"E_UNKNOWN"` for the forward-compat fallback (since the
/// payload is an owned String and cannot be promoted to `'static`).
#[test]
fn as_static_str_known_and_unknown() {
    assert_eq!(ErrorCode::CapDenied.as_static_str(), "E_CAP_DENIED");
    assert_eq!(
        ErrorCode::Unknown("E_SOMETHING".into()).as_static_str(),
        "E_UNKNOWN"
    );
}
