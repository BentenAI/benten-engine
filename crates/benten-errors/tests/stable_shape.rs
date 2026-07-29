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

use std::str::FromStr;

use benten_errors::ErrorCode;

/// Builds `ALL_CATALOG_VARIANTS` from the shared roster.
///
/// The roster body — all 201 identifiers plus the running per-wave ledger that
/// documents every count bump — moved to `crates/benten-errors/catalog_roster.rs.in`
/// (F-014). `src/lib.rs` includes that same file inside its own
/// `#[cfg(test)] mod catalog_roster_pin`, where it additionally expands into a
/// **wildcard-free** `match` over `ErrorCode`. That match is what makes the
/// list unable to fall behind the enum, and it can only live there: `tests/`
/// is downstream of `#[non_exhaustive]`, so a `match` HERE is forced to carry
/// a `_` arm and is exhaustive-in-name-only.
///
/// **Adding a variant:** append it to `catalog_roster.rs.in` (omitting it does
/// not compile), bump `variant_count_is_pinned` below AND
/// `catalog_roster_length_is_pinned` in `src/lib.rs`, add the `as_str` /
/// `as_static_str` / `from_str` / `routed_edge_label` arms, then document the
/// code in `docs/ERROR-CATALOG.md`.
///
/// `ErrorCode::Unknown(String)` is deliberately excluded — it's the
/// forward-compat fallback, not a catalog code.
macro_rules! catalog_roster {
    ($($variant:ident),+ $(,)?) => {
        const ALL_CATALOG_VARIANTS: &[ErrorCode] = &[$(ErrorCode::$variant),+];
    };
}

include!("../catalog_roster.rs.in");

/// Count of catalog variants (auto-derived from [`ALL_CATALOG_VARIANTS`] so
/// adding to the list and forgetting to bump a number is impossible).
const CATALOG_VARIANT_COUNT: usize = ALL_CATALOG_VARIANTS.len();

/// Every catalog variant must round-trip through `as_str` / `from_str`
/// without hitting the `Unknown` fallback, and the roster length is pinned.
///
/// SCOPE, stated honestly: this test checks the ROSTER, not the enum. A
/// variant present in the enum and absent from the roster is invisible from
/// here — that case is caught one crate up, by the wildcard-free `match` in
/// `src/lib.rs::catalog_roster_pin`, which expands from the same
/// `catalog_roster.rs.in` this file includes. What IS caught here: a roster
/// entry whose `from_str` arm is missing, and a roster length that moved
/// without the pin moving.
///
/// MUTATION THAT MUST MAKE THIS FAIL: append any identifier to
/// `catalog_roster.rs.in` without changing the `201` below — the count
/// assertion at the end of this function fails 202 != 201. (Verified by
/// running it, 2026-07-27.)
#[test]
fn variant_count_is_pinned() {
    // Every listed variant must round-trip through from_str(as_str).
    for code in ALL_CATALOG_VARIANTS {
        let s = code.as_str();
        let parsed = ErrorCode::from_str(s);
        assert_eq!(
            parsed.as_ref(),
            Ok(code),
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
    // (RESERVED_CODES_AT_PHASE_2A_SNAPSHOT), the 10 firing codes
    // (FIRING_CODES_AT_PHASE_2A_SNAPSHOT),
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
    // #992 (refinement-audit-2026-05 wire-format cluster): +1
    // `GraphSchemaVersionMismatch` (redb on-disk schema-version envelope).
    // 168 + 1 = 169.
    // G-CORE-1 fix-pass (Phase 4-Meta-Core, #989 storage-partition
    // seam): +1 `NamespacedWriteUnsupported` (BrowserBackend / non-
    // partitioned-backend fail-closed typed-reject when ctx
    // .namespace_did = Some at the §1.A.FROZEN canary surface).
    // 169 + 1 = 170.
    // G-CORE-3a CANARY (Phase 4-Meta-Core, #1300/#1301 substrate;
    // F-3 W1 spec-gap closure): +1 `RecipientLacksKeysForSuite`
    // (X-Wing-hybrid cipher-suite fail-closed typed-reject when the
    // recipient lacks one of the hybrid-required key halves; the
    // silent-downgrade defense per CLAUDE.md baked-in #5).
    // 170 + 1 = 171.
    //
    // G-CORE-3d (Phase 4-Meta-Core, #1301; per-Node AEAD wrap layer +
    // two-CID mapping table) MERGED at 5ccdd0da: +3 codes covering the
    // three structural failure classes of the storage-layer AEAD path
    // (`AeadRebindingAttackDetected` + `TwoCidMappingNotFound` +
    // `TwoCidMappingIntegrityMismatch`). 171 + 3 = 174.
    //
    // G-CORE-3b (Phase 4-Meta-Core, RATIFIED-S&C 2026-05-21 §R1 + §R3)
    // MERGED at e89a1919: +2 — `AuthorizationGrantBindingSigInvalid`
    // (the ONE-signed-artifact binding-sig tamper-detection typed-reject
    // for the A-1 stolen-UCAN + A-2 stolen-keys + A-3 wrong-audience
    // attacks) + `ChainNarrowingViolation` (structured-`Scope` chain-
    // validator widening typed-reject across `Hashes` subset-violations
    // + `RestrictedSelector` 6-dim widenings). 174 + 2 = 176.
    //
    // G-CORE-6a (Phase 4-Meta-Core, #506 builder closure) batch-rebased
    // past 176: +1 `ValueOutOfRange` (SubgraphBuilder records over-range
    // numeric arguments as deferred errors and surfaces them at the
    // single-fallible-point `.build()` call per #506 / G-CORE-6 verify-
    // pass). 176 + 1 = 177.
    //
    // G-CORE-6b (Phase 4-Meta-Core, 2026-05-23, P-III Ben-authorized
    // autonomously per no-users-yet) rebased onto post-#1325 main:
    // +1 `SnapshotBlobSchemaVersionMismatch` — the v1→v2
    // `SnapshotBlob.schema_version` P-III bump lifts the typed
    // `SnapshotBlobError::SchemaVersion` mismatch out of the generic
    // `Serialize` family so callers can match the cross-version case
    // specifically (mirrors `GraphSchemaVersionMismatch` for the
    // snapshot-blob surface). 177 + 1 = 178.
    //
    // G-CORE-3e (Phase 4-Meta-Core, sync + iroh-blobs UCAN-gating)
    // rebased onto post-#1331 main: +3 codes —
    // `UcanBlobsRequestRejected` + `UcanBlobsRequestNotInScope` +
    // `UnresolvedPeerDeny` for the custom-ALPN per-request UCAN check
    // path. 178 + 3 = 181.
    //
    // G-CORE-3f (Phase 4-Meta-Core, NEW benten-drop crate) consolidated
    // alongside G-CORE-3e in Strategy-C wave-2 batch: +3 codes —
    // `DropBundleEnvelopeSigInvalid` + `DropBundleVersionUnsupported` +
    // `DropBundleMode3InlineRejected` for the offline Drop bundle
    // defense-in-depth contract (Spike G+H). 181 + 3 = 184.
    //
    // G-CORE-8 (Phase 4-Meta-Core, security-surface lock) consolidated
    // alongside G-CORE-3e + G-CORE-3f in Strategy-C wave-2 batch:
    // +4 codes — `ManifestEnvelopeRecheckUnresolvedDeny` (§4.36 fail-
    // CLOSED flip — typed-reject for any non-positive recheck outcome at
    // the apply_atrium_merge per-row recheck boundary; closes
    // security-r1-1 + security-r1-2 BLOCKERs) +
    // `PluginInstallRecordAlreadyApplied` (§4.37 atomic record-and-check
    // around install admission — second presentation of the same
    // install-record CID rejects before the cap-cascade runs; closes the
    // TOCTOU window on the applied-records set) +
    // `WriteBoundaryChainNotUserRooted` (§4.23 structural-always-on
    // user-DID root chain validator at the WRITE admission seam — mirrors
    // Phase-3 G16-B-F structural-always-on per-row cap-recheck) +
    // `ThinClientBridgePrincipalUnresolved` (§4.22 thin-client bridge
    // principal resolution failure — the bridge never trusts client-
    // supplied principals; resolves from the authenticated session).
    // 184 + 4 = 188.
    //
    // G-CORE-DSL chunk-3 (#839) merged via #1339: +1 `DslBackendRejected`
    // — downstream-consumer rejection at the DSL-compile boundary; closes
    // the `CompileError::Io`-variant abuse at the devserver site by giving
    // downstream consumers a typed home distinct from real file-IO
    // failures. 188 + 1 = 189.
    //
    // **Phase-4-Meta-Core G-CORE-3c terminal swap-matrix wave**: +1
    // `AuditNotLandedPurePqRejected` (the C11b safety invariant per the
    // PQ-default reframe; `SwapMatrix::try_pure_pq_sole_trust_path`
    // constructor gate fires this code when the workspace-baseline
    // `AUDIT_LANDED_PURE_PQ_FLAG` is `false` — load-bearing v1-GM-gating
    // safety invariant; named arm is what the C-GM-AUDIT CI lane greps
    // for). 189 + 1 = 190.
    //
    // **Pre-G-CORE-9-FREEZE 2026-05-24 fix-up bundle**: +1 `DslIoError`
    // — first-class mirror of the pre-existing `CompileError::Io`
    // variant (§3.5g item 6 amendment closure; closes the
    // CompileError::Io first-class-mirror gap surfaced by PR #1339
    // chunk-3 where `Backend` was added as first-class but `Io` was
    // left mapping to `E_UNKNOWN` at the napi boundary). 190 + 1 = 191.
    //
    // **G-CORE-9 V1-FROZEN-INTERFACE row 4 / §1.A.FROZEN item 15(h)**:
    // +1 `SubgraphSpecWalkFailed` — typed reject from the public
    // `Engine::walk_share_scope` consumer surface for the SubgraphSpec
    // walker (the wave-time mint of the engine wrapper at
    // `crates/benten-engine/src/engine_share_scope.rs`). 191 + 1 = 192.
    // R6 R1 FP-F4 §S3a + §S3b: +2 `PluginInstallConsentDenied` +
    // `PluginPerDelegationDenied` — the §8-E hook denial codes. 192 → 194.
    //
    // **Row D-19 G-COMP-1 wave (Phase-4-Meta-Core R6 R2 FP integration,
    // Cohort 8)**: +3 `DslParseError` + `DslUnknownPrimitive` +
    // `DslMissingRespond` — first-class catalog mirrors of the
    // pre-existing `CompileError::{Parse,Semantic,Build}` variants /
    // `pub const benten_dsl_compiler::E_DSL_*` wire-string constants.
    // The same wave performs the atomic 4-surface rename of
    // `ViewStrategyCReserved` -> `ViewStrategyReserved` (rename, not a
    // mint — does not bump the count). 194 -> 197.
    //
    // **Phase-4-Meta-Core F-full Wave w-ms-canary (R5 MembershipSet
    // primitive; F-MS-8 / F4-031)**: +1 `RoleStaleAtVerify`
    // (`E_ROLE_STALE_AT_VERIFY`) — a MembershipSet group stanza sealed under
    // a stale `role_assignments_generation` is rejected at the sealed-envelope
    // verify boundary. §3.5g atomic mint across all four surfaces. 197 -> 198.
    //
    // **Phase-4-Meta-Core F-full Wave w-gov-audit (R5 MembershipSet TIER-2;
    // F-INV19-1 / Inv-19)**: +1 `KvTargetNotImmutable`
    // (`E_KV_TARGET_NOT_IMMUTABLE`) — a K(V) (membership version-node key)
    // derivation against a mutable Anchor CID is REJECTED (Inv-19 forbids
    // binding key material to an Anchor whose CURRENT pointer moves).
    // §3.5g atomic mint across all four surfaces. 198 -> 199. (NOTE: the
    // parallel w-ms-sync wave may also mint codes; the integrator reconciles
    // the count at strategy-C integrate-time.)
    // F-INJ-2 (Phase-4-Meta-Core pre-freeze): +1 for
    // `DropBundleEnvelopeIssuerMismatch` (Drop envelope-issuer anchoring).
    // 199 -> 200.
    // GAP-KDB Shape-B (Phase-4-Meta-Core, W4 docs-invariants): +1 for
    // `RecipientKemNotCommitted` (`E_RECIPIENT_KEM_NOT_COMMITTED`) — the
    // Layer-C seal reject when the recipient KEM key is not committed by its
    // audience did:benten (Inv-23; resolve_kem / RecipientBinding 2nd-preimage
    // fail-closed). §3.5g atomic mint across all four surfaces. 200 -> 201.
    // (NOTE: the parallel GAP-KDB W2 benten-drop flagship wave uses the
    // crate-local `RecipientBindingError`, NOT a throwable — no double-mint;
    // the strategy-C integrator reconciles the count if any sibling wave also
    // mints, per the historical #1319↔#1318 collision pattern.)
    assert_eq!(
        CATALOG_VARIANT_COUNT, 201,
        "CATALOG_VARIANT_COUNT drift — update this value AND docs/ERROR-CATALOG.md in the same commit",
    );
}

// Where the enum↔roster check actually lives, and why not here.
//
// This slot used to hold `catalog_variant_count_matches_enum`, described in
// `docs/V1-FROZEN-INTERFACE.md` Backstop 5 and in `INTERNALS.md` as an
// "exhaustive-match dual tripwire" that made a missing list entry "fail to
// compile". It did neither. `ErrorCode` is `#[non_exhaustive]` and this is a
// downstream crate, so its `match` was required to carry `_ => false`; and
// its "independent count" was `ALL_CATALOG_VARIANTS.iter().filter(...).count()`
// compared against `ALL_CATALOG_VARIANTS.len()` — the list against itself.
//
// Proven inert by mutation (R6 round-#1 falsification sweep, 2026-07-27):
// adding a fully-throwable `ErrorCode` variant with a catalog string and a
// routing class, and wiring `as_static_str` + `routed_edge_label`, left 9/9
// tests in this file PASSING including the hard-coded `201`. A tautology on
// a merge-blocking lane is worse than no test, because it manufactures
// confidence — so it is deleted rather than patched.
//
// The replacement is `src/lib.rs::catalog_roster_pin`. It includes the same
// `catalog_roster.rs.in` this file does, and inside the defining crate
// `#[non_exhaustive]` is inert, so its `match` has no wildcard and a missing
// entry is `error[E0004]`. It runs on the required `frozen-bytes` lane
// through that workflow's step-5 `--lib` arm (which already selects
// `-p benten-errors`) and on every `build+test` leg.
//
// Nothing is asserted here; a test whose only content is a comment would be
// one more thing that looks like coverage.

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
    let parsed = ErrorCode::from_str("E_CAP_DENIED").expect("recognized code");
    assert_eq!(parsed, ErrorCode::CapDenied);
    assert_eq!(parsed.as_str(), "E_CAP_DENIED");
}

/// Unknown codes are a parse error (#733: fallible by design). The raw
/// string is preserved on the error so forward-compat callers can recover
/// `ErrorCode::Unknown` explicitly.
#[test]
fn from_str_unknown_is_err_and_preserves_raw_string() {
    let err = ErrorCode::from_str("E_NOT_A_REAL_CODE")
        .expect_err("unrecognized code must be a parse error");
    assert_eq!(err.as_str(), "E_NOT_A_REAL_CODE");
    // Forward-compat recovery preserves the raw string verbatim so
    // rendering stays lossless.
    let recovered = ErrorCode::from_str("E_NOT_A_REAL_CODE")
        .unwrap_or_else(|e| ErrorCode::Unknown(e.into_inner()));
    assert_eq!(recovered.as_str(), "E_NOT_A_REAL_CODE");
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

// ---------------------------------------------------------------------------
// v1-API-stabilization closure-pins (ST-ERRORS lane — Refs #1180/#1007).
//
// §3.6b behavioral guarantees: each of these would FAIL if the prior
// ST-ERRORS v1-API-stabilization work were reverted. They lock the
// pre-v1-tag public-surface contract so a future refactor cannot silently
// re-introduce the dead surfaces (#283/#286/#291) or drop the
// forward-readiness derives (#1007 items 10+11) or the snapshot-frozen
// const renames (#736) without tripping CI.
// ---------------------------------------------------------------------------

/// #1007 item 10 — `Hash` is derived so `ErrorCode` is usable as a
/// `HashMap` / `HashSet` key for Phase-5+ ErrorCode-keyed routing tables
/// WITHOUT hashing via `.as_str()`. Reverting the `Hash` derive fails to
/// compile this test (the `HashSet<ErrorCode>` instantiation requires it).
#[test]
fn errorcode_is_hashable_for_keyed_routing() {
    use std::collections::HashSet;
    let mut seen: HashSet<ErrorCode> = HashSet::new();
    seen.insert(ErrorCode::CapDenied);
    seen.insert(ErrorCode::Unknown("E_PLUGIN_FOO".into()));
    assert!(seen.contains(&ErrorCode::CapDenied));
    assert!(!seen.contains(&ErrorCode::InvCycle));
}

/// #1007 item 11 — `Ord` / `PartialOrd` are derived so consumers get a
/// deterministic catalog total-order for v1-API freeze regardless of
/// internal variant reshuffles (Fwd-2 #1039 inline-reshuffle is then
/// safe). Reverting the `Ord` derive fails to compile `.sort()`.
#[test]
fn errorcode_has_total_order_for_deterministic_iteration() {
    let mut v = vec![
        ErrorCode::WriteConflict,
        ErrorCode::CapDenied,
        ErrorCode::InvCycle,
    ];
    v.sort();
    // Derived Ord follows declaration order; the exact order is not the
    // contract — the *existence* of a stable total order is. Asserting
    // the sort is idempotent pins the guarantee without coupling to
    // declaration order.
    let mut again = v.clone();
    again.sort();
    assert_eq!(v, again);
    assert_eq!(v.len(), 3);
}

/// #736 — the Phase-2a snapshot consts are reachable under their
/// snapshot-frozen names (`*_AT_PHASE_2A_SNAPSHOT`), NOT the former
/// mis-extension-inviting `PHASE_2A_*` names. This pins the rename so a
/// future agent cannot silently re-introduce the open-scope-implying
/// names. Reverting the rename fails to resolve these paths.
#[test]
fn phase_2a_snapshot_consts_use_frozen_names() {
    use benten_errors::{FIRING_CODES_AT_PHASE_2A_SNAPSHOT, RESERVED_CODES_AT_PHASE_2A_SNAPSHOT};
    assert_eq!(FIRING_CODES_AT_PHASE_2A_SNAPSHOT.len(), 13);
    assert_eq!(RESERVED_CODES_AT_PHASE_2A_SNAPSHOT.len(), 5);
    // Every snapshot entry must still round-trip (the snapshot is a
    // subset of the live catalog, never drifting out of it).
    for code in FIRING_CODES_AT_PHASE_2A_SNAPSHOT
        .iter()
        .chain(RESERVED_CODES_AT_PHASE_2A_SNAPSHOT)
    {
        assert_eq!(ErrorCode::from_str(code.as_static_str()), Ok(code.clone()));
    }
}
