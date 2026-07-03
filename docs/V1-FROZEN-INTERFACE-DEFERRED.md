# V1-FROZEN-INTERFACE-DEFERRED — signature-frozen-consumption-deferred surfaces

> **Companion artifact to [`docs/V1-FROZEN-INTERFACE.md`](V1-FROZEN-INTERFACE.md).**
>
> This document enumerates the G-CORE-9 FREEZE wave's signature-frozen
> surfaces whose PRODUCTION CONSUMPTION is deferred to a downstream
> wave (typically G-COMP-1 within Phase-4-Meta-Composing). Per HARD
> RULE 12 clause-(b), every deferred item gets an explicit
> BELONGS-NAMED-NOW destination row here.
>
> **Authority:** G-CORE-9 R1 triage Fork 2 ratification (orchestrator
> distinctive-angle decision under night-shift stance, 2026-05-24;
> rebuttable at next morning review).
>
> **Reading-order:** consume this AFTER V1-FROZEN-INTERFACE.md when
> implementing G-COMP-1 wave consumers.
>
> **Status:** authoritative for the freeze contract's
> consumption-deferred surfaces; updates land via PR + cite-drift
> sweep.

## Revision history (selected)

- **2026-05-24** — R6 R1 FP-A: **Row D-7 + Row D-22 RETRACTED / CLOSED**
  per Ben PM ratification of HARD RULE 12 over the prior path-(b)
  defers ("do the full ~13-site cascade now"). F2 closes Row D-7
  (§8-A Engine visibility cluster tighten + napi cascade) by
  renaming + tightening the 4 methods and migrating napi to
  `read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, ...)`. F1.a-e
  closes Row D-22 (workspace `_for_test` cfg-gating sweep) with
  70+ declarations gated + 14-item EXEMPT_PUB_ITEMS allow-list +
  no-regression test pin at
  `tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs`
  + 8 cargo-public-api baseline regens. Both rows retained for
  forensic context per pim-13 / §3.12; closure annotations
  inline in each row body.

---

## Why this document exists

The G-CORE-9 FREEZE wave's contract is to lock PUBLIC SIGNATURES at
v1-beta. Several substrates landed in Phase-4-Meta-Core (the
WriteBoundaryChainValidator + InstallRecordReplayStore + 3 §8-E
CapabilityPolicy hooks + ProductionManifestEnvelopeRechecker port +
accept_atrium_share seam) with their TYPE SHAPES correctly frozen,
but their PRODUCTION CONSUMERS (the engine call sites that actually
invoke them) are part of the downstream G-COMP-1
self-composing-admin-layer build-out.

Per the G-CORE-9 R1 council's L6 lens findings, presenting these as
"frozen and live" in V1-FROZEN-INTERFACE.md narrative would be a
HARD RULE 12 clause-(b) violation: the freeze contract would
ADVERTISE security properties (Layer-1 user-root chain validation
at WRITE; Layer-3 manifest envelope at sync; cross-peer install
verification) that the v1-beta binary does not yet enforce.

The honest disposition is to distinguish:
- **signature-frozen-and-consumed-at-v1-beta** = the type shape is
  locked AND production code invokes it AND the security property
  is structurally live in the v1-beta binary.
- **signature-frozen-consumption-deferred** = the type shape is
  locked at v1-beta (so adding the consumer later is non-breaking)
  but the production consumer ships in a downstream wave with a
  NAMED destination.

This document enumerates the second class.

---

## Deferred surfaces — G-COMP-1 destinations

Each row: (i) frozen surface (where the signature locks at v1-beta),
(ii) deferred consumption (what the G-COMP-1 wave must wire),
(iii) v1-beta posture (what the binary actually enforces / does not
enforce at v1-beta), (iv) Compromise / spec anchor.

### ~~Row D-1~~ — WriteBoundaryChainValidator consumption (Engine::commit / Engine::put_node_with_context) — **CLOSED at R6 R1 FP-F4 §S1** (2026-05-24) — **SHARPENED at R6 R2 FP-B** (2026-05-25)

> **STATUS: CLOSED (sharpened).** Per Ben PM-ratified F1 path-(a) full
> ~13-site cascade ("if we're going to want to do them all eventually,
> then I say do the full ~13-site cascade now"), the
> `WriteBoundaryChainValidator` consumption is now structurally-
> always-on at **14** WRITE entry points (engine_crud × 5 + engine_caps
> × 2 + engine_views × 1 + engine_modules × 2 + engine_diagnostics × 1
> + engine_wait × 1 + handler_versions × 1 + **R6 R2 FP-B: apply_atrium_merge
> per-row chain-bearing × 1**) via the new `Engine::admit_write_chain`
> helper + sealed `WriteAdmissionFrame`.
>
> **R6 R2 FP-B (L2-R2-MAJOR-1 closure):** pre-FP-B 13 of 13 sites
> passed `WriteAdmissionFrame::engine_internal()`; the
> `delegate_capability` site was the **only** chain-bearing caller.
> Inbound-sync per-row writes routed only through `append_version`
> (engine_internal frame), so the WriteBoundaryChainValidator never
> observed the peer-DID at row admission. Post-FP-B the
> `apply_atrium_merge` per-row loop presents a
> `WriteAdmissionFrame::with_chain(peer_actor_cid, peer_did)` frame,
> closing the asymmetry where outbound writes were chain-walked but
> inbound sync rows were not. Layer-1 user-as-root invariant is
> structurally enforced at every WRITE admission when a production
> validator is installed; 2 of 14 sites are chain-bearing
> (delegate_capability + apply_atrium_merge per-row); 12 are
> engine-internal frame. Row retained for forensic context per pim-13
> / §3.12.

### Row D-1 (FORENSIC) — WriteBoundaryChainValidator consumption (Engine::commit / Engine::put_node_with_context)

- **Frozen surface (v1-beta):**
  `crates/benten-engine/src/write_boundary_chain_validator.rs` —
  `pub trait WriteBoundaryChainValidator { fn validate_chain(...) ->
  WriteBoundaryChainOutcome; }` + `pub enum WriteBoundaryChainOutcome
  { NotApplicable, AdmittedRoot, RejectedRoot { ... } }` (with
  `#[non_exhaustive]` per Bundle 3) + `NoopWriteBoundaryChainValidator`
  default returning `NotApplicable` for every chain.
- **Deferred consumption (G-COMP-1 destination):** wire
  `validate_chain` consultation at every `Engine::commit` /
  `Engine::put_node_with_context` admission call site (audit ~10-15
  WRITE entry points; precedent: Phase-3 G16-B-F structural-always-on
  per-row cap-recheck wave-pull). Estimated LOC: ~200-400 + ~10-15
  test pins.
- **v1-beta posture:** the substrate exists; the always-mounted Noop
  returns `NotApplicable` (admit) at every chain. Layer-1 user-as-root
  enforcement at the WRITE admission boundary is NOT live at v1-beta
  binary — the runtime enforcement is the Layer-1
  `CapabilityPolicy::pre_write` check (already live via Compromise
  #2 sync-replica sub-narrative).
- **Anchor:** CLAUDE.md baked-in #18 Layer-1 user-as-root invariant;
  spec V1-FROZEN-INTERFACE.md item 8.

### ~~Row D-2~~ — InstallRecordReplayStore lifecycle-required wiring — **CLOSED at R6 R1 FP-F4 §S2** (2026-05-24)

> **STATUS: CLOSED.** `InstallPorts.install_record_replay_check`
> drops `Option<&mut Fn>` for `&mut Fn` — every install caller MUST
> supply a substantive closure now. Test fixtures wire
> `benten_platform_foundation::testing::noop_replay_check()`;
> production callers wire
> `engine.install_record_replay_store().record_and_check`.
> `manifest_store::install_plugin` renamed to
> `install_verified_record_unchecked` with `#[deprecated]` +
> `#[doc(hidden)]` to steer callers to the full
> `plugin_lifecycle::install_plugin` path.

### Row D-2 (FORENSIC) — InstallRecordReplayStore lifecycle-required wiring

- **Frozen surface (v1-beta):**
  `crates/benten-engine/src/install_record_replay.rs` —
  `InstallRecordReplayStore` atomic record-and-check primitive +
  `PluginInstallRecordAlreadyApplied` typed ErrorCode + the
  `parallel_presentation_serialized_one_admits_one_rejects` test
  primitive. Frozen at v1-beta.
- **Deferred consumption (G-COMP-1 destination):** wire
  `install_record_replay_check: Some(Box::new(move |hash|
  engine.install_record_replay_store().record_and_check(hash)))` at
  the canonical install_plugin entrypoint(s); convert the
  `InstallPorts.install_record_replay_check` port from
  `Option<&mut InstallRecordReplayCheckFn>` to required
  `&mut InstallRecordReplayCheckFn` per the structural-always-on
  discipline. Also: gate `manifest_store::install_plugin` to
  require the replay check OR doc-pin as internal-only.
- **v1-beta posture:** the substrate exists + is correctly tested;
  zero production callers supply `Some(closure)` at HEAD. The TOCTOU
  defense (parallel-install-of-same-bytes) is NOT live in shipped
  binaries.
- **Anchor:** spec V1-FROZEN-INTERFACE.md item 12;
  `crates/benten-engine/src/install_record_replay.rs:33` module-doc
  per-process invariant.

### ~~Row D-3~~ — 3 §8-E CapabilityPolicy hooks consumption — **PARTIAL CLOSED at R6 R1 FP-F4 §S3a / §S3b / §S3c** (2026-05-24)

> **STATUS: PARTIAL CLOSED.**
> - **Row D-3-a CLOSED** at §S3a: `InstallConsentPolicy` trait in
>   `benten_platform_foundation::install_consent` + threaded through
>   `InstallPorts.policy` + consumed at `plugin_lifecycle::install_plugin`
>   step 3c with typed `ErrorCode::PluginInstallConsentDenied` reject.
>   CATALOG_VARIANT_COUNT 192→193.
> - **Row D-3-b CLOSED** at §S3b: `EngineCapsHandle::delegate_capability`
>   consults `CapabilityPolicy::check_per_delegation` between Step 2b
>   (shares-policy resolver) and Step 3 (effective scope) with typed
>   `ErrorCode::PluginPerDelegationDenied` reject.
>   CATALOG_VARIANT_COUNT 193→194.
> - **Row D-3-c PARTIAL CLOSED** at §S3c per Δv3-2: the 4 production
>   `policy.check_write(&ctx)` sites all switched to
>   `policy.check_write_with_audience(&ctx)`. The audience-aware
>   enrichment seam IS wired (default delegates to `check_write`);
>   `audience_did` stays `None` at sweep sites per Δv3-2 (peer_did
>   at apply_atrium_merge is transport-principal NOT cap-target).
>   The populate-side at delegate_capability defers to G-COMP-1 +
>   the existing Layer-3 `check_per_delegation` wiring covers
>   delegate-runtime audience-discrimination needs.

### Row D-3 (FORENSIC) — 3 §8-E CapabilityPolicy hooks consumption

- **Frozen surface (v1-beta):**
  `crates/benten-caps/src/policy.rs::CapabilityPolicy::{check_install_consent, check_per_delegation, check_write_with_audience}`
  — defaulted trait methods with signatures locked + object-safety
  preserved + workspace-test `__sealed_for_workspace_tests` re-export
  covers test doubles. (Path-symbol cite per pim-1 / §3.5b HARDENED
  point 3; the previous file:line cite at policy.rs:492-557 understated
  by ~14 lines as the file grew through G-CORE-9 fix-pass cycles.)
- **Deferred consumption (G-COMP-1 destination):** wire at the three
  semantically-canonical call sites:
  - `check_install_consent` → at the install-pipeline admission
    (plugin_lifecycle.rs step 3b alongside replay-check)
  - `check_per_delegation` → at the cross-plugin delegation runtime
    boundary inside `delegate_capability` (engine_caps.rs:541
    alongside the shares_policy_resolver call)
  - `check_write_with_audience` → at `apply_atrium_merge`'s per-row
    `check_write` call (engine.rs:1413 — switch unconditionally;
    default impl delegates to `check_write` preserving back-compat).
- **v1-beta posture:** trait signatures + sealed-discipline are
  structurally locked. Workspace grep finds ZERO production call
  sites for any of the three hooks at HEAD. A future custom
  CapabilityPolicy author overriding any of the three will be
  silently ignored.
- **Anchor:** CLAUDE.md #18 trust model Layer-2 + Layer-3; spec item 8.

### ~~Row D-4~~ — ProductionManifestEnvelopeRechecker production impl + default-builder wiring — **CLOSED at R6 R1 FP-F4 §S4** (2026-05-24)

> **STATUS: CLOSED.** `ProductionManifestEnvelopeRechecker` substantive
> impl shipped at `crates/benten-engine/src/production_manifest_envelope_rechecker.rs`
> + `ProductionEngineBuilder` (per CRITIC-2 F-2.2 rename) shipped at
> `crates/benten-engine/src/production_engine_builder.rs` as the
> canonical production constructor that wires the substantive
> rechecker post-build. At v1-beta the load-bearing addition is the
> synthesized-fallback hardening (Row D-18 coupling); full
> PluginLibrary-driven chain walk is the G-COMP-1 deliverable per
> the substantive-rechecker-installed-detection-couple narrative.

### Row D-4 (FORENSIC) — ProductionManifestEnvelopeRechecker production impl + default-builder wiring

- **Frozen surface (v1-beta):**
  `crates/benten-engine/src/manifest_envelope_recheck.rs::ManifestEnvelopeRechecker`
  — `pub trait ManifestEnvelopeRechecker` port interface + the
  four-arm `ManifestEnvelopeRecheckOutcome` enum + the structural
  empty-peer-DID fail-CLOSED at
  `crates/benten-engine/src/engine.rs::Engine::apply_atrium_merge`
  (Layer-A defense fires structurally before rechecker dispatch).
  (Path-symbol cite per pim-1 / §3.5b HARDENED point 3.)
- **Deferred consumption (G-COMP-1 destination):** ship
  `ProductionManifestEnvelopeRechecker` consuming `PluginLibrary` +
  `UserDidRegistry` + invoking
  `manifest_envelope_chain_validation::validate_chain_with_manifest_envelope`;
  default-builder wires `ProductionManifestEnvelopeRechecker` in
  place of `NoopManifestEnvelopeRechecker`.
- **v1-beta posture:** the always-installed Noop returns
  `NotApplicable` (admit) for resolvable peer-DIDs. Layer-A
  empty/sentinel-DID short-circuit IS live. The per-resolvable-DID
  substantive recheck is NOT live in shipped binaries.
- **Anchor:** Compromise #26 in SECURITY-POSTURE.md (retensed in
  Bundle 2); spec item 12.

### Row D-5 — accept_atrium_share cross-peer install seam

- **Frozen surface (v1-beta):** NONE — the function is named in
  V1-FROZEN-INTERFACE.md item 12 but does not exist as a public
  surface at HEAD. The frozen-doc cite has been removed at G-CORE-9
  (Bundle 2 retense).
- **Deferred consumption (G-COMP-1 destination):** ship
  `pub fn accept_atrium_share(...)` at
  `crates/benten-platform-foundation/src/plugin_lifecycle.rs` closing
  G24-D-FP-1 follow-up wave; commented-out test bodies at
  `crates/benten-platform-foundation/tests/admin_ui_v0_atrium_share_unattested_peer_rejected.rs:42-107`
  + `crates/benten-platform-foundation/tests/admin_ui_v0_install_as_signed_plugin_across_two_atrium_peers.rs:24`
  re-enable post-wire-up.
- **v1-beta posture:** cross-peer install verification is NOT live.
  The platform-foundation install pipeline at v1-beta consumes
  plugins through user-DID-signed install records ONLY (no cross-peer
  ingest).
- **Anchor:** Compromise #26 retense; CLAUDE.md #18 Layer-2.

### ~~Row D-6~~ — §4.25 sync-hydrate consumption of UnresolvedDeny at handshake.rs — **CLOSED at R6 R1 FP-F4 §S4** (2026-05-24) — **WIRED at R6 R2 FP-B** (2026-05-25)

> **STATUS: CLOSED + WIRED.** `crates/benten-sync/src/handshake.rs::sync_hydrate_consume_recheck_outcome`
> minted at R6 R1 FP-F4 §S4 as the §4.25 sync-hydrate handshake-time
> consumption surface. **R6 R2 FP-B (L2-R2-MAJOR-6 closure):** the
> helper was minted but had ZERO production callers (verified by
> §3.5n grep 2026-05-25). Post-FP-B the merge boundary at
> `apply_atrium_merge` per-row routes the recheck outcome through
> `sync_hydrate_consume_recheck_outcome` (forensic parity arm — the
> typed-error decision still surfaces via the engine-side
> `outcome_to_row_reject`; the hydrate consumer is the parity-with-
> handshake observability arm). The new
> `manifest_envelope_recheck::outcome_to_error_code` projection helper
> bridges the two surfaces' shapes. The
> `g_core_8_manifest_envelope_recheck_fail_closed_flip_4_36.rs:300-314`
> named-pin destination is now wired (the §4.36 merge-time + §4.25
> hydrate-time both consume the shared primitive; per-row consultation
> now exercises both).

### Row D-6 (FORENSIC) — §4.25 sync-hydrate consumption of UnresolvedDeny at handshake.rs

- **Frozen surface (v1-beta):** the SHARED primitive
  (`ManifestEnvelopeRecheckOutcome::UnresolvedDeny` +
  `outcome_to_row_reject`) is frozen.
- **Deferred consumption (G-COMP-1 destination):** wire UnresolvedDeny
  short-circuit at `crates/benten-sync/src/handshake.rs` per the
  test-comment named pin at
  `crates/benten-engine/tests/g_core_8_manifest_envelope_recheck_fail_closed_flip_4_36.rs:300-314`.
- **v1-beta posture:** the §4.36 merge half is structurally enforced;
  the §4.25 sync-hydrate half is NOT (the same UnresolvedDeny
  primitive is available + the consumer wire-up is the deferred half).
- **Anchor:** spec item 12 (retensed in Bundle 2).

### ~~Row D-7~~ — §8-A Engine visibility cluster tighten + napi cascade — **CLOSED at R6 R1 FP-A Bundle F2** (2026-05-24)

> **STATUS: RETRACTED / CLOSED.** Per Ben 2026-05-24 PM ratification of
> HARD RULE 12 over the prior path-(b) defer ("if we're going to want
> to do them all eventually, then I say do the full ~13-site cascade
> now"), the §8-A visibility tighten + napi cascade LANDED at R6 R1
> FP-A Bundle F2 — the four methods are now `pub(crate)` with their
> v1-GM target names + napi migrated to
> `read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, ...)` +
> test-helper re-exports at `crate::testing` preserve sibling-crate
> integration tests. Row retained for forensic context per
> pim-13 / §3.12.

- **Frozen surface (v1-beta):** NONE TIGHTENED at v1-beta — per the
  G-CORE-9 R1 triage L2-BLK-1 escalation, the §8-A tighten cascades
  through 75+ call sites across the workspace AND breaks the napi
  binding's public `get_node` / `put_node` methods at
  `bindings/napi/src/lib.rs::Engine::{get_node, put_node}`. Per HARD RULE 12 clause-(b)
  the disposition is BELONGS-NAMED-NOW here.
- **Deferred consumption (G-COMP-1 destination):**
  - Rename `Engine::get_node` → `Engine::read_node` (pub→pub(crate))
  - Rename `Engine::get_node_label_only` → `Engine::read_node_label_only` (pub→pub(crate))
  - Tighten `Engine::put_node` → `pub(crate)`
  - DELETE `Engine::resolve_subgraph_cid_for_test` from public surface; relocate to a `testing` module
  - Cascade through workspace test sites (~75 call sites)
  - Cascade through napi binding (replace `engine.get_node` /
    `engine.put_node` direct calls with `engine.read_node_as(principal, cid)` + transaction-based puts;
    design call: ENGINE_INTERNAL_PRINCIPAL_CID const for sites needing
    un-attributed reads at the napi layer)
  - Add the no-regression test pin
    `crates/benten-engine/tests/g_core_9_engine_no_direct_cap_mutation.rs`
  - Regenerate `docs/public-api/benten-engine.txt` baseline
- **v1-beta posture:** all four methods remain `pub fn` at v1-beta.
  External callers (and the napi binding) can call `Engine::get_node`
  directly without the principal-bearing capability gate. The
  CLAUDE.md baked-in #18 plugin-trust model's read-side enforcement
  recommendation is "use `read_node_as(principal, cid)`" but is
  NOT structurally enforced at v1-beta binary; per discipline,
  callers SHOULD route through `read_node_as`.
- **Anchor:** spec item 1; V1-FROZEN-INTERFACE-BUILD-BACKLOG.md row
  1.a/1.b/1.c; CLAUDE.md baked-in #18.

### ~~Row D-8 — F3 anti-replay atomic compare-and-swap (FrameReplayMarker TOCTOU)~~ **CLOSED** at R6 R2 batch-A Item 8 (Cohort 8)

- **Closure:** Option (a) chosen — `KVBackend::compare_and_insert`
  added with a default non-atomic impl (preserves behavior for
  non-transactional backends) + a txn-atomic override on
  `RedbBackend`. `FrameReplayMarker::mark_and_check_frame` routes
  through the new primitive. On the redb-backed backend the get +
  insert + commit run inside a SINGLE redb write transaction; write-
  txn exclusivity (only ONE write-txn open per-handle at a time) gates
  concurrent CAS attempts. At most ONE concurrent caller admits.
- **Test pin:**
  `crates/benten-caps/tests/tf_d8_frame_replay_marker_cas_atomic_under_concurrent_inbound.rs`
  — 16-thread race against the same nonce; asserts exactly 1
  first-observer + 15 replay-rejected.
- **Anchor:** Compromise #23 (closure narrative updated at
  SECURITY-POSTURE.md).

### Row D-9 — wire-format hex-pinned byte-pin tests sweep (6 of 8 deferred)

- **Frozen surface (v1-beta):** existing roundtrip + constant-position
  + format-version-byte-position pins per L11 lens substantively
  cover byte stability. The codepoint-integer-value pin lands in
  Bundle 5 of this PR (G-CORE-9 fix-pass). The per-chunk AAD layout
  pin lands in Bundle 5 of this PR (locks the as-shipped 2-arg
  layout per Fork 1).
- **Deferred consumption (G-COMP-1 destination):** ship the
  remaining 6 hex-pinned byte-pin tests for: SnapshotBlob v2,
  per-chunk AEAD canonical hex, UCAN-Varsig v1 header, AuthorizationGrant
  CBOR, Drop bundle CBOR, encryption-envelope per codepoint
  (5-codepoint table), signature-envelope per codepoint
  (4-codepoint table). Each at
  `crates/<crate>/tests/canonical_bytes_v1_<surface>.rs`.
- **v1-beta posture:** roundtrip + constant-position + format-version
  pins enforce byte stability + format-version discrimination.
  Hex-pinned-bytes regression-defense is partial.
- **Anchor:** spec §4; V1-FROZEN-INTERFACE.md item 4.

### Row D-10 — §15.j live-per-request resolver-evaluation test pin

- **Frozen surface (v1-beta):** the §15.j commitment (RATIFIED-S&C
  R5) is documented at V1-FROZEN-INTERFACE.md:1617-1619.
- **Deferred consumption (G-COMP-1 destination):** ship the
  verification-mechanism test pin per V1-FROZEN-INTERFACE.md:1617-1619
  shape: write a Node into scope, issue a UCAN, write a second Node
  into scope, call walk_share_scope as recipient, assert BOTH Nodes
  are enumerated. ~50 LOC at
  `crates/benten-engine/tests/g_core_9_share_scope_live_per_request.rs`.
- **v1-beta posture:** the live-per-request property is structurally
  in place (resolver IS called per request); the regression-defense
  test pin is absent.
- **Anchor:** spec §15.j.

### Row D-11 — walk_share_scope_as principal-bearing additive overload

- **Frozen surface (v1-beta):** `Engine::walk_share_scope` is
  principal-unbearing at v1-beta. The principal-bearing
  `walk_share_scope_as` is named ADDITIVE Composing-time enhancement
  at `engine_share_scope.rs:46-49`.
- **Deferred consumption (G-COMP-1 destination):** add
  `pub(crate) fn walk_share_scope_as(&self, principal: &Cid, ...)`
  + thread through napi as a principal-bearing variant for
  recipient-side enumeration per RATIFIED-S&C §R4 + 15(h).
- **v1-beta posture:** the principal-bearing recipient enumeration
  for path-tagged-key derivation is unavailable; the unbearing
  walk is used by all consumers.
- **Anchor:** RATIFIED-S&C §R4; spec §15(h).

### Row D-12 — DSL filesystem-exercising tests

- **Frozen surface (v1-beta):** `pub fn compile_file` is locked at
  v1-beta but has no substantive filesystem-exercising test (only
  `compile_str` is directly tested; the documented metadata-size
  short-circuit at `benten_dsl_compiler::lib` is dead-letter-tested).
- **Deferred consumption (G-COMP-1 destination):** ship a
  filesystem-exercising test that creates temp files, calls
  `compile_file`, asserts the Io error class fires on missing/oversized
  files.
- **v1-beta posture:** compile_file shape is locked; test coverage
  for the file-handling arms is sparse.
- **Anchor:** L9-DSL-MINOR-2.

### ~~Row D-13 — structural_kdf info-tag codepoint-binding~~ **CLOSED** at R6 R2 batch-A Item 7 (Cohort 8)

- **Closure:** `structural_kdf::derive_root` extended to take
  `cipher_suite_codepoint: u16` AND fold it into the HKDF info-tag
  (`info = "root:codepoint:" || codepoint_le_bytes || root_cid`).
  Cross-codepoint key reuse class structurally closed: same
  `(K_principal, root_cid)` inputs derived under different codepoints
  produce different K_root values. Wire-format-coupled (K_root feeds
  downstream AEAD wrap; pre-Item-7 K_root values not byte-compatible
  with post-Item-7); landed under P-III no-users-yet override per
  Cohort 8 entry.
- **Test pin:** `crates/benten-crypto-suite/src/structural_kdf.rs::tests::derive_root_distinguishes_cipher_suite_codepoints`
  — same `(K_principal, root_cid)` derived under `0x647a` / `0x6400`
  / `0x647b` MUST produce 3 distinct K_root values.
- **Anchor:** L2-MAJ-4.

### Row D-14 — Recursive cargo invocation test hygiene

- **Frozen surface (v1-beta):**
  `tests/phase_3_workspace/missing_docs_workspace.rs::full_missing_docs_sweep_no_warnings_workspace_wide`
  spawns `cargo doc --workspace --no-deps` from within a `cargo
  nextest` test runner.
- **Deferred consumption (G-COMP-1 destination):** rework to read
  the previously-captured `cargo doc` output from a CI-step
  artifact OR convert to a build.rs-only check OR remove (the
  alternate ci.yml workflow covers the missing_docs gate).
- **v1-beta posture:** the recursive-invocation footgun is present;
  the alternate ci.yml workflow covers the missing_docs gate (belt
  + suspenders).
- **Anchor:** L12-MIN-1.

### Row D-15 — Post-v1-beta hardening watch-list

- **R6 R1 fix-pass revision-history (R6 R1 Bundle F3):** the prior
  G-CORE-9 R1 triage Fork 1 disposition ("retract doc claim from
  3-tuple AAD to 2-tuple as-shipped + defer `total_chunks` defense
  to G-COMP-1") was **RETRACTED** at R6 R1 fix-pass. The CODE was
  revised to bind `total_chunks` per the spec text (3-tuple AAD
  layout: `aad_per_chunk(plaintext_cid, chunk_index, total_chunks)`
  becomes 4-segment AAD: domain-tag `benten-aead:chunk:` ||
  plaintext_cid || chunk_index (u64 BE) || total_chunks (u32 BE);
  big-endian per M-19, migrated from the earlier LE encoding at
  F-full Wave-0), closing the
  cross-chunk-truncation attack at v1-beta. Wire-format pin updated
  at `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs`;
  behavioral truncation/inflation pins added at
  `crates/benten-graph/src/aead_wrap.rs::tests`. See V1-FROZEN-INTERFACE.md
  per-chunk-AEAD wire layout entry for the post-retraction freeze contract.
- **Frozen surface (v1-beta):** various nice-to-have hardenings
  surfaced in R1 OBS items. Each named sub-row below has its own
  destination (G-COMP-1 vs Phase-4-Meta-Composing vs post-audit) and
  its own anchor.
- **Cross-cutting v1-beta posture:** all of the named sub-rows below
  are nice-to-have; each has no immediate exploit at v1-beta (the
  audience CID IS bound via binding_sig; ed25519_dalek is the only
  signature primitive used for binding_sig at v1-beta so the
  hardcoded shape is consistent; classical-only construction IS
  cryptographically sound; the per-chunk AEAD layout IS canonical-bytes
  pinned via the existing AAD layout test).
- **Anchor (overall row):** Various R1 OBS items + R4b L4 R1 fix-pass
  letter-suffix anchor promotion (L4-R4b-MIN-2 + L4-R4b-MIN-1
  closure 2026-05-24).

#### Row D-15a — AAD per-chunk `total_chunks` augmentation — ✅ CLOSED at v1-beta (SUPERSEDED by Row D-15 retraction)

- **CLOSED:** this deferral is RETRACTED — the `total_chunks` defense
  is BUILT at v1-beta (see Row D-15 above). The per-chunk AAD is the
  4-segment `aad_per_chunk(plaintext_cid, chunk_index, total_chunks)`
  binding (`benten-aead:chunk: || plaintext_cid || chunk_index (u64 BE)
  || total_chunks (u32 BE)`); big-endian per M-19. The
  cross-chunk-truncation attack surface (an attacker truncating the
  ciphertext stream after N chunks giving a valid-looking decryption
  for chunks 0..N-1 with no detection that chunks N..total_chunks-1
  are missing) is closed in-band.
- **v1-beta posture:** the canonical-bytes pin at
  `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs`
  + the behavioral truncation/inflation pins at
  `crates/benten-graph/src/aead_wrap.rs::tests` lock the 4-segment
  shape and exercise the truncation-before-decrypt arm.
- **Anchor:** Row D-15 retraction (R6 R1 fix-pass Bundle F3) +
  V1-WIRE-FORMAT-INVENTORY.md per-chunk AAD row.

#### Row D-15b — `CryptoPolicy::require_hybrid_pq` consumer-side flag

- **Frozen surface (v1-beta):** the consumer-side `CryptoPolicy`
  trait does not carry a `require_hybrid_pq` arm at v1-beta;
  classical-only `0x6400` envelopes are admitted by the SwapMatrix
  dispatch even when a deployment posture requires hybrid-PQ.
- **Deferred consumption (Phase-4-Meta-Composing v1-assessment-window):**
  add `CryptoPolicy::require_hybrid_pq -> bool` (defaulted false
  for v1-beta backward-compat); the SwapMatrix dispatch checks the
  flag pre-resolve + rejects classical-only codepoints with the
  existing `SwapMatrixError::Unsupported(UnsupportedAlgorithm::...)`
  typed-reject.
- **v1-beta posture:** classical-only is the deliberately-non-default
  swappable arm per the crypto-agility contract (#5); a deployment
  posture requiring hybrid-PQ has no policy-flag surface at v1-beta
  but can be enforced by config (disable the classical SwapMatrix
  arm registration).
- **Anchor:** L2-MAJ-3 G-CORE-9 R1 finding.

#### Row D-15c — RETRACTED at R6 R2 fix-pass (R6-R2-FP-A)

- **RETRACTED 2026-05-25 (R6 R2 Bundle R6-R2-FP-A).** The original
  Row D-15c rationale claimed "a cooperating attacker who forges
  audience_pubkey still cannot pass binding_sig verification" — this
  was **FALSE at the live ARM 2/ARM 5 split** in
  `UcanBlobsHandler::validate_request_for_connection`. Verified via
  §3.5n orchestrator ground-truth: ARM 2 (audience-binding) compared
  the connection EndpointId to `grant.audience_pubkey` (the
  post-sign-mutable field), and ARM 5 (binding-sig verification)
  re-constructed the binding-message using `grant.audience_binding`
  (the CID of the issue-time audience pubkey) — which an attacker
  could leave UNCHANGED while mutating `audience_pubkey` to their own
  pubkey. The pre-fix attack: Eve obtains a grant for Bob, mutates
  `audience_pubkey` to her own pubkey, connects with her own iroh
  EndpointId; ARM 2 admits (eve == eve), ARM 5 verifies (bob's
  audience_binding still bound), iroh-blobs serves the bytes to Eve.
  Access-theft, NOT just attribution-forgery.
- **Closed by:** [R6-R2-FP-A] folds `audience_pubkey` into the
  binding-message (6-segment layout under `BINDING_SIG_DOMAIN v3`
  bumped from `v2`); pin
  `crates/benten-caps/tests/tf3b_audience_substitution_post_sign_rejected.rs`
  exercises the substantive arm + the would-FAIL-on-revert was
  verified (3/3 tests FAIL when binding_message ignores
  audience_pubkey).
- **Cross-confirming findings closed:** L2-R2-BLOCKER-1 + L3-r2-1 +
  L13-MAJ-2 + L17-r2-MAJOR-1 + L4-MAJ + L1-MAJ-1 (6-lens cross-
  confirmation).

#### Row D-15d — `AeadEnvelope::to_wire_bytes` nonce-panic → Result

- **Frozen surface (v1-beta):**
  `crates/benten-crypto-suite/src/aead.rs::AeadEnvelope::to_wire_bytes`
  panics via `.expect` on nonce length > 255 — the format-version
  byte budgets nonce length to u8.
- **Deferred consumption (G-COMP-1 destination):** convert to
  `Result<Vec<u8>, AeadEnvelopeError>` with a typed
  `AeadEnvelopeError::NonceTooLarge(usize)` arm; the call sites
  bubble the typed error through the existing `?` chain.
- **v1-beta posture:** the panic IS reachable only with attacker-
  controlled nonce-length input + the SwapMatrix dispatch enforces
  per-cipher-suite max nonce (≤16 bytes for the v1-beta default
  ChaCha20-Poly1305 + ≤12 for AES-256-GCM swap-arm) — the panic
  arm is structurally unreachable at v1-beta default config.
- **Anchor:** L1-crypto-r1-5 G-CORE-9 R1 finding.

#### Row D-15e — `AuthorizationGrant.binding_sig` hardcoded `[u8; 64]` → varsig-tagged

- **Frozen surface (v1-beta):** binding_sig at
  `crates/benten-caps/src/authorization_grant.rs::AuthorizationGrant`
  is hardcoded `[u8; 64]` (Ed25519 fixed-size); the rest of #5
  framing carries multiformats codepoint-dispatch via varsig.
- **Deferred consumption (G-CORE-PQ-WIRE wave — see Row D-26):**
  promote to varsig-tagged variable-length to admit ML-DSA-65 (3293
  bytes) + future hybrid signatures (Ed25519⊕ML-DSA-65 = 3357 bytes
  concatenated) under the same binding_sig shape. The classical
  half is preserved via the codepoint-dispatch fall-through.
  Re-homed from "post-audit + Phase-4-Meta-Composing" to the
  G-CORE-PQ-WIRE wave per Ben 2026-05-24 PM "do everything now"
  ratification (the structural sub-fork on HOW to carry hybrid
  pubkeys is the same for binding_sig as for the 3 sites in Row D-26;
  bundling them in one wave is the do-it-all-properly path).
- **v1-beta posture:** ed25519_dalek is the only signature primitive
  used for binding_sig at v1-beta so the hardcoded shape is
  consistent. The audit (NF-2 / C-GM-AUDIT) lands BEFORE v1-GM;
  the varsig promotion couples to the audit-result decision on
  whether to ship binding_sig as hybrid-by-default at v1-GM.
- **Anchor:** L17-r1-6 G-CORE-9 R1 finding + #5 crypto-agility
  contract + NF-2 / C-GM-AUDIT v1-GM gate + Row D-26 wave-bundling
  ratification 2026-05-24 PM.

#### Row D-15-RETRACTED — SHA hashcodepoint pre-blessed agile-hash mint

- ~~SHA2_512_256 (multihash `0x1015`) + SHA3_256 (multihash `0x16`)
  pre-blessed agile-hash-fallback codepoint mint~~ — **RETRACTED at
  G-CORE-9 R3 fix-pass (L11-R3-MAJOR-2 closure)**: both `HashCodepoint`
  variants ALREADY EXIST at HEAD (minted at commit `ae69c339` G-CORE-2,
  well before this FREEZE wave) AND are declared PERMANENT at
  V1-FROZEN-INTERFACE.md item 6.2 codepoint table. The hex-pin landed
  at `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs::codepoint_table_integer_values_pinned`
  at G-CORE-9 R3 fix-pass (per HARD RULE 12 — pin must land NOW, not
  predicated on a future codepoint-mint that already happened). The
  original L11-R2-MINOR-4 closure-evidence was mis-stated.

### ~~Row D-18~~ — L2-MAJ-1 empty-peer-DID synthesized-fallback structural hardening — **CLOSED at R6 R1 FP-F4 §S4 (rechecker layer, 2026-05-24) + R6-R2 FP Item 9 (engine substrate layer, 2026-05-25)**

> **STATUS: CLOSED end-to-end.**
>
> **R6 R1 FP-F4 §S4 (2026-05-24) — rechecker layer:**
> `is_synthesized_node_id(did_str: &str) -> bool` helper minted at
> `crates/benten-engine/src/manifest_envelope_recheck.rs` per Δv3-10.
> The `ProductionManifestEnvelopeRechecker` consults this helper +
> returns `UnresolvedDeny` when the peer-DID is the `node-id:N`
> synthesized-fallback shape. The substantive-rechecker-installed-
> detection-couple narrative is preserved: the Noop default continues
> to admit (NotApplicable) so default-Noop test fixtures don't
> over-fire; only the substantive rechecker hardens.
>
> **R6-R2 FP Item 9 (2026-05-25) — engine substrate layer:**
> structural defense-in-depth lift: `ManifestEnvelopeRechecker` gains
> a `fn is_substantive(&self) -> bool` default method (default `true`;
> Noop overrides to `false`). `Engine::apply_atrium_merge`'s per-row
> loop now short-circuits with `ManifestEnvelopeRecheckUnresolvedDeny`
> when both `rechecker.is_substantive()` AND
> `is_synthesized_node_id(peer_did_str)` hold — BEFORE consulting the
> rechecker. This makes the synthesized-fallback reject the LOAD-BEARING
> defense at the engine substrate (CLAUDE.md #18 Layer-3
> structural-always-on), so a faulty production rechecker impl that
> admits `node-id:N` is no longer reachable on this code path.
> Regression-guard at `crates/benten-engine/tests/r6_r2_fp_item_9_d18_substantive_rechecker_detection_couple.rs`
> exercises a faulty-admit-all substantive rechecker + asserts the
> engine substrate rejects regardless.

### Row D-18 (FORENSIC) — L2-MAJ-1 empty-peer-DID synthesized-fallback structural hardening

- **Frozen surface (v1-beta):** the structural empty-peer-DID
  fail-CLOSED at `crates/benten-engine/src/engine.rs::apply_atrium_merge` (the `ManifestEnvelopeRecheckUnresolvedDeny` arm; symbol-form per §3.5b HARDENED point 3) IS live for the literal-empty
  peer_node_ids case. The synthesized-fallback (`resolve_peer_dids`
  emits `node-id:N` string for unregistered peer_node_ids) ADMITS
  at v1-beta via the always-mounted Noop rechecker (NotApplicable).
- **Deferred consumption (G-COMP-1 destination):** add a hardening
  layer that rejects `node-id:`-prefixed synthesized DIDs as
  unresolvable WHEN a substantive (non-Noop) rechecker is installed.
  The naïve always-on filter (initial L2-MAJ-1 fix attempted in this
  PR but reverted) over-fires for the default-Noop test fixtures
  which intentionally don't register peer-DIDs. The proper closure
  couples synthesized-fallback rejection to substantive-rechecker
  detection.
- **v1-beta posture:** L2-MAJ-1 attack vector (adversarial peer
  presenting an unmapped `node-id:N` DID) is admit-only if the
  rechecker is Noop (admit-everything anyway); under a substantive
  ProductionRechecker (G-COMP-1 deliverable per Row D-4), the
  rechecker is responsible for its own per-DID resolution including
  rejecting synthesized DIDs. So the gap closes at Row D-4 closure;
  Row D-18 is the substrate-level defense-in-depth follow-up.
- **Anchor:** L2-MAJ-1 G-CORE-9 R1 finding + Compromise #26 NOT-live
  v1-beta posture.

### Row D-17 — `CapWriteContext` + `ReadContext` + `SuspensionOutcome` + lens-scoped pub-type extension `#[non_exhaustive]` application (with cascade)

> **STATUS (2026-05-25):** the 3 NAMED types in the row title — `CapWriteContext`, `ReadContext`, `SuspensionOutcome` — **CLOSED at R6-R2 FP Item 6**. The attribute is applied at the type sites (`crates/benten-caps/src/policy.rs::CapWriteContext` + `crates/benten-caps/src/policy.rs::ReadContext` + `crates/benten-engine/src/engine_wait.rs::SuspensionOutcome`). The cross-crate cascade migrated all `~6` production sites in `benten-engine` to the `default()` + field-mutation pattern; all `~13` benten-caps integration-test sites mechanically converted; `bindings/napi/src/wait.rs` + `crates/benten-eval/benches/wait_suspend_resume_latency.rs` gained wildcard arms. `ReadContext::by_label_and_cid(label, cid, device_cid)` constructor minted at `crates/benten-caps/src/policy.rs` to handle the typed dual-shape case from `primitive_host::check_read_capability`. Audit-test deferral comments at `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs` lifted; the SuspensionOutcome arm-coverage pin now exercises the `_` wildcard guard. cargo-public-api baselines `docs/public-api/benten-caps.txt` + `docs/public-api/benten-engine.txt` regenerated.
>
> **REMAINING (G-COMP-1 destination):** the R2 EXTENSION lens-scoped pub-type set (~40+ types across `benten-engine` outcome.rs + `benten-ivm` view + `benten-platform-foundation` materializer + `benten-core` Subgraph cluster) — these were NOT closed at Item 6 (item scope was the 3 NAMED types per the title; lifting the EXTENSION set would balloon cascade ~10×). The R2 EXTENSION set carries its own per-class carve-outs documented inline below (e.g., `benten-ivm` view-instance + kernel-internal surface = "no `#[non_exhaustive]` cascade at v1-beta to preserve cargo-public-api baseline shape").

- **Frozen surface (v1-beta):** spec V1-FROZEN-INTERFACE.md item 11
  table row enumerates `CapWriteContext` + `ReadContext` +
  `SuspensionOutcome` as APPLY candidates; the attribute itself is
  NOT applied at v1-beta. The type shape is locked (additions ARE
  breaking per the freeze contract); the attribute is the missing
  piece. The non_exhaustive cascade for `SuspensionOutcome` hits ~32
  workspace match-site arms; for `CapWriteContext` hits ~50+ test
  direct-struct-literal sites; for `ReadContext` hits ~30+ similar
  sites. Total cascade ~80+ files in the benten-caps + benten-engine
  test families.

  **G-CORE-9 R2 EXTENSION (L8-R2-MAJOR-CARRY-2 closure):** the R1-enumerated
  remaining pub types not yet covered by any DEFERRED row are added here:
  - `benten-engine`: `UserViewInputPattern` (outcome.rs), `TraceStep`
    (outcome.rs), `StreamCursor` (engine_stream.rs),
    `SubscribeCursor` (engine_subscribe.rs), `EngineViewsHandle`
    (`benten_engine::engine_views::EngineViewsHandle`), `AtriumConfig`
    (atrium_api.rs), `SyncStatus`
    (atrium_api.rs), plus the outcome.rs 13-pub-struct set
    (`UserViewSpec`, `UserViewSpecBuilder`, `ReadViewOptions`, `Outcome`,
    `Trace`, `TerminalError`, `BudgetExhaustedView`, `AnchorHandle`,
    `RegisterReplaceOutcome`, `HandlerPredecessors`, `DiagnosticInfo`,
    `NestedTx`)
  - `benten-ivm`: `SubgraphSpec` (subgraph_spec.rs:107), `KernelInput`
    (subgraph_spec.rs:262), `ViewState` (view.rs:157), `ViewBudget`
    (view.rs:180), `ViewQuery` (view.rs:223), `ViewResult` (view.rs:242),
    `ViewDefinition` (view.rs:387), `LabelPattern` (algorithm_b.rs:375)
    [L8-r3-MIN-3 closure: IVM kernel pattern-selector surface;
    within-crate exhaustive matches at algorithm_b.rs:403-404 + :416
    unaffected]
  - `benten-platform-foundation`: `VocabLabel` (vocab.rs:14), `VocabEdge`
    (vocab.rs:92), `Scalar` (vocab.rs:157), `RenderError` (materializer.rs:567),
    `MaterializerError` (materializer.rs:209), `MaterializerDenialFrame`,
    `MaterializerWalkInputs`, `MaterializerOutput`, `SubscribeAttachToken`
    [L8-r3-MIN-1 + L8-r4-MIN-2 closure: materializer-walk return-type +
    materializer-public-struct cluster; in-crate exhaustive matches preserved
    when `#[non_exhaustive]` is added since out-of-crate consumers add
    wildcard arm]
  - `benten-core`: `Mode` (version_dag.rs:75), `VersionError`
    (version.rs:105), `VersionDagError` (version_chain.rs:52), `Anchor`,
    `DagVersionChain`, `VersionDag`, `Subgraph` (wire-bytes-bearing per
    `canonical_subgraph_bytes`; apply per Path-b R2.8 wire-bytes precedent
    at the next D-17-targeted fix-pass — pim-N candidate per
    L8-r4-OBS-1 same-file/same-namespace-sweep recurrence), `SubgraphBuilder`,
    `NodeHandle` [L8-r3-MIN-2 + L8-r4-MIN-1 + L8-r4-MIN-3 closure: Version
    DAG container types + Subgraph public surface]
  - `benten-ivm` view-instance + kernel-internal surface: `Subscriber`,
    `CanonicalViewEntry`, `AlgorithmBView`, `Projection`, `EffectiveRules`,
    plus 5 view-instance structs [L8-r4-MIN-4 closure: per-file pub-item
    sweep of benten-ivm; D-17 enumeration extension only — Path-a per
    spec; no `#[non_exhaustive]` cascade at v1-beta to preserve cargo-public-api
    baseline shape]

  **R6-R2 EXTENSION (F-04 / F-05 closure — §11 ↔ §16 reconciliation):** the
  15th crate `benten-membership-set` shipped at F-full with a §16 freeze
  section in V1-FROZEN-INTERFACE.md but its pub surface was NEVER added to the
  §11 `#[non_exhaustive]` sweep table (V1-FROZEN-INTERFACE.md item 11
  enumerated must-apply table covers the 14 prior crates only). This row
  carries the §16 pub-surface non_exhaustive disposition (the
  `crates/benten-membership-set/src/lib.rs` pub-use set; verified at HEAD):
  - **ALREADY `#[non_exhaustive]` at HEAD (KEEP):** `MembershipSetError`
    (`crates/benten-membership-set/src/error.rs:31`).
  - **DELIBERATE CARVE-OUTS (DO NOT APPLY — frozen-cardinality, already
    documented in V1-FROZEN-INTERFACE.md §16):** `MembershipSetKind`
    (`src/kind.rs:26` — EXACTLY-3 `VARIANT_COUNT == 3`; a 4th arm is a §15.c
    HALT-AND-SURFACE, NOT a wildcard) + `RoleId` (`src/member.rs` — 5-value
    ordinal all-active per Inv-20 clause-j; the `u8` ordinal is keying-AAD-bound
    golden-vector-pinned).
  - **REMAINING §11-sweep candidates (APPLY-or-documented-carve-out at the
    D-17-targeted fix-pass; verified MISSING `#[non_exhaustive]` at HEAD):**
    `RequestedReserveKind` (`src/kind.rs:76`), `KindDispatchError`
    (`src/kind.rs:85`), `MemberRef` (`src/member.rs:97` enum), `MemberNature`
    (`src/member.rs:191`), `MemberEntry` (`src/member.rs:135`), `MembersTable`
    (`src/member.rs:248`), `Did` (`src/member.rs:36`), `Hlc`
    (`src/member.rs:56`), `SigPubKey` (`src/member.rs:83`), `MembershipSet`
    (`src/set.rs:35`). Note the `MemberEntry` / `MembersTable` field ORDER is
    the canonical CBOR-key-sorted layout (load-bearing per §16) — the
    `#[non_exhaustive]` decision is orthogonal to field-order (it gates only
    additive future fields/variants). The R6-R2 CODE half of this work is
    finding **F-03** (Agent-C §11 sweep on the F-full crates); F-04 / F-05
    here is the DEFERRED-ledger ENUMERATION half so the §16 surface is
    registry-discoverable against the §3.12 R7-equivalent audit walk + the
    workspace-walker enhancement (also deferred in this row).
  - **Anchor (R6-R2 extension):** R6-R2 council findings F-04 + F-05
    (§11 ↔ §16 non_exhaustive carry); V1-FROZEN-INTERFACE.md §16 MembershipSet
    freeze section + item 11 enumerated must-apply table.

  **R10-council EXTENSION (F-02 / F-11 closure — crypto-suite + drop §11 gaps
  CLOSED at v1-beta, NOT deferred):** the R10 non_exhaustive freeze-hygiene sweep
  found + CLOSED the following gaps in-round (applied at HEAD, not carried):
  - **APPLIED `#[non_exhaustive]` (§11 SemVer-readiness doc-block on each):**
    `benten-crypto-suite::swap_matrix::{SwapKeypair, SwapPublicKey,
    SwapRecipientKeypair, SwapRecipientPublic, SwapRecipientSecret}` (the 5 Swap
    matrix enums — each grows with a new sig/enc arm) +
    `benten-crypto-suite::discharge::DischargeDisposition` +
    `benten-drop::layer_c::EncryptedEnvelope` (the Inv-16 codepoint-dispatched
    envelope). Integration-test consumers of `EncryptedEnvelope` gained
    fail-closed `_` arms; the 5 `Swap*` enums have no external match sites.
  - **DELIBERATE CARVE-OUT registered (DO NOT APPLY):**
    `benten-drop::layer_c::BindingContext` — the CLOSED 2-variant single-recipient
    drop set (`DropPlaintextSender` = `0x6500` / `DropSealedSender` = `0x6510`).
    Wire-keying (one codepoint per variant); exhaustive-by-design mirrors
    `MembershipSetKind`/`RoleId`. NO `#[non_exhaustive]`, NO `_` arm — the
    exhaustive 2-arm match HALT-AND-SURFACEs any wire decision. Registered in the
    V1-FROZEN-INTERFACE.md §11 carve-out set.
  - **Audit arm-coverage pins:**
    `crates/benten-crypto-suite/tests/g_core_9_non_exhaustive_audit_crypto_suite.rs`
    (6 pins) +
    `crates/benten-drop/tests/g_core_9_non_exhaustive_audit_drop.rs`
    (`EncryptedEnvelope` non_exhaustive pin + `BindingContext`
    exhaustive-by-design pin).
  - **Anchor (R10 extension):** R10-council findings F-02 + F-11;
    V1-FROZEN-INTERFACE.md item 11 must-apply table + carve-out set.
  - **NOTE — the D-17 non_exhaustive-in-Composing SemVer question (R10 F-22)** is
    SURFACED-TO-BEN separately (Ben-gated) and is NOT dispositioned in this row.

  **Wire-bytes-load-bearing types CLOSED AT G-CORE-9 R2 (NOT deferred):**
  `TypedOutputProjection` + `KernelOutput` in `benten-ivm/src/subgraph_spec.rs`
  carry the attribute at v1-beta — the 1-byte arm-discriminator at
  `algorithm_b.rs:1507-1523` makes them wire-format-bearing and they were
  not deferrable; closure landed via Bundle R2.8 with 5-test-site cascade
  fix (`view_2 / view_4 / view_5 round_trip + view_4 / view_5 shape_pin`).

  Audit-test workspace-walker enhancement: V1-FROZEN-INTERFACE.md:951-957
  describes the audit test as walking every pub enum/struct workspace-wide;
  the shipped test enumerates ~10 named types only. Workspace-walker
  implementation (consume cargo-public-api JSON output OR syn-based AST
  walker OR rustdoc-json walk) deferred to G-COMP-1 as part of this row.
- **Deferred consumption (G-COMP-1 destination):** apply
  `#[non_exhaustive]` to each type + cascade through test-site
  direct-struct-literal constructions + cross-crate match sites,
  migrating each to `Default::default()` + field-mutation pattern OR
  adding the wildcard arm. Production code (in `benten-engine`) ALREADY
  uses the field-mutation pattern per Bundle 3 of the R1 PR. Also implement
  the workspace-walker enhancement for the audit-test verification mechanism.
- **v1-beta posture:** at v1-beta the type shape is locked per the
  freeze contract narrative; the attribute is the documentation gap.
  Field additions are TREATED AS breaking by v1-beta engineering
  discipline per spec item 11 narrative (the structural enforcement
  via `#[non_exhaustive]` is what G-COMP-1 lights).
- **Anchor:** V1-FROZEN-INTERFACE.md item 11 table rows for
  `CapWriteContext` + `ReadContext`; L6-r1-1 G-CORE-9 R1 escalation;
  L8-R2-MAJOR-CARRY-2 + L8-R2-MINOR-CARRY-1 G-CORE-9 R2 extensions.

### ~~Row D-19~~ — G-CORE-9 R1 Bundle 4 ESCALATED items (Strategy::C → Reserved rename + 3 DSL ErrorCode mints) — **CLOSED at R6-R2-FP-integration-redo Group C / Cohort 8 (2026-05-25)**

**Status: CLOSED.** Landed at the R6-R2-batch-c-dsl-catalog sub-branch of the R6-R2-FP-integration-redo PR (2026-05-25) per the v1-beta-freeze-window auto-WIRE-NOW discipline. The atomic 4-surface §3.5g rename + 3 first-class catalog mints all shipped in a single commit; CATALOG_VARIANT_COUNT 194 → 197; cargo-public-api + ts-public-api baselines regenerated; drift-detect baseline removed the 3 grandfathered `CompileError::{Parse,Semantic,Build}` lines per §3.5g item 6 amendment closure. See `docs/V1-BETA-BREAKING-CHANGES.md` Cohort 8 for the full migration enumeration; row body retained below for forensic context.

**Original (pre-closure) row body:**

- **Frozen surface (v1-beta):** the obsolete `Strategy::C` arm name, the wire string `E_VIEW_STRATEGY_C_RESERVED`, the variant `ViewStrategyCReserved`, the TS class `EViewStrategyCReserved`, and the absence of explicit `E_DSL_PARSE_FAILED` / `E_DSL_UNKNOWN_PRIMITIVE` / `E_DSL_MISSING_RESPOND` ErrorCodes all freeze at v1-beta. The cargo-public-api baselines at `docs/public-api/benten-errors.txt` (`pub benten_errors::ErrorCode::ViewStrategyCReserved`) + `docs/public-api/benten-engine.txt` (`pub benten_engine::error::EngineError::ViewStrategyCReserved` + `pub benten_engine::EngineError::ViewStrategyCReserved` re-export) lock the obsolete `ViewStrategyCReserved` name; per Bundle 10 Fork 3 the cargo-public-api workflow is required-failing so the rename WINDOW is the G-CORE-9 freeze wave OR a deliberate post-v1-beta SemVer break. (Path-symbol cites per pim-1 / §3.5b HARDENED point 3; previous numeric line cites at benten-errors.txt:188 + benten-engine.txt:976,977,2329,2330 had drifted uniformly off-by-one to 187 / 975,976,2328,2329 post baseline regeneration.)
- **Deferred consumption (G-COMP-1 destination):** atomic 4-surface rename per §3.5g:
  1. Rust enum `EngineError::ViewStrategyCReserved` → `EngineError::ViewStrategyReserved` (`crates/benten-engine/src/error.rs` + format-string in `benten_engine::engine_views` already returns `Strategy::Reserved`)
  2. Wire string `E_VIEW_STRATEGY_C_RESERVED` → `E_VIEW_STRATEGY_RESERVED` (`crates/benten-errors/src/lib.rs` 4 sites: variant + wire string + Display arm + parse arm)
  3. TS class `EViewStrategyCReserved` → `EViewStrategyReserved` (`packages/engine/src/errors.generated.ts` 3 sites; docstring already says "Strategy::Reserved" — cross-language drift on SAME code path per §3.5g item 1)
  4. `docs/ERROR-CATALOG.md` (E_VIEW_STRATEGY_C_RESERVED entry + table) + cargo-public-api baselines `docs/public-api/benten-errors.txt` (`pub benten_errors::ErrorCode::ViewStrategyCReserved`) + `docs/public-api/benten-engine.txt` (`pub benten_engine::error::EngineError::ViewStrategyCReserved` + `pub benten_engine::EngineError::ViewStrategyCReserved` re-export) + `crates/benten-errors/tests/stable_shape.rs::{variant_count_is_pinned, all_throwable_codes_round_trip, ...}` regenerate. (Path-symbol cites per pim-1 / §3.5b HARDENED point 3.)

  AND mint 3 new DSL ErrorCodes per L9-DSL-MAJOR-1 closure:
  5. `E_DSL_PARSE_ERROR` — REUSES existing `pub const E_DSL_PARSE_ERROR` at `crates/benten-dsl-compiler/src/lib.rs::E_DSL_PARSE_ERROR` (already in use as the `Diagnostic.error_code` field value at 6+ production construction sites); G-COMP-1 deliverable = `ErrorCode::DslParseError` enum variant + `EDslParseError` TS class mirror (the wire string is unchanged). Per L9-r3-MIN-1 name-collision closure (the prior `E_DSL_PARSE_FAILED` naming would have left the existing pub const orphaned).
  6. `E_DSL_UNKNOWN_PRIMITIVE` — mints from existing `CompileError::Semantic`
  7. `E_DSL_MISSING_RESPOND` — mints from existing `CompileError::Semantic` sub-case

  CATALOG_VARIANT_COUNT delta: 192 → 195 (3 new mints; Strategy rename is a rename not a mint).

  Remove the corresponding drift-detect baseline grandfathered lines from `scripts/drift-detect-error-variant-mirror-baseline.txt` for `CompileError::Parse`/`Semantic`/`Build` per the §3.5g item 6 amendment closure.
- **v1-beta posture:** the obsolete `Strategy::C` naming + the 3 ungranted DSL ErrorCodes ride into v1-beta wire bytes. No immediate exploit (the variant works correctly; the names are stale). The rename window IS specifically the G-CORE-9 freeze wave OR G-COMP-1 (any later is a SemVer break post-v1-beta tag).
- **Anchor:** L8-MAJOR-1 + L9-DSL-MAJOR-1 + V1-BETA-BREAKING-CHANGES.md:152-156 Bundle 4 ESCALATED entry. Resolves the L8-R2-MAJOR-CARRY-1 / L9-r2-MIN-2 / L12-R2-MIN-1 phantom-destination cross-confirmed pattern (R2 council finding).

### Row D-20 — L6-r1-3 trybuild compile-fail regression backstop for the CapabilityPolicy hard-seal

- **Frozen surface (v1-beta):** the hard-seal MECHANISM itself IS structurally
  enforced by rustc on every workspace build. `pub(crate) mod sealed { pub trait
  Sealed {} }` + `pub trait CapabilityPolicy: sealed::Sealed + ...` at
  `crates/benten-caps/src/policy.rs:50-64` means an external
  `impl CapabilityPolicy for SomeExternalType` cannot reach the private
  `Sealed` supertrait and fails to compile. Workspace-test opt-in is via
  the `#[cfg(feature = "testing")] #[doc(hidden)] pub mod __sealed_for_workspace_tests`
  re-export. The seal is real at v1-beta.
- **Deferred consumption (G-COMP-1 destination):** add `trybuild` dev-dep +
  ship `crates/benten-caps/tests/compile_fail/external_cap_policy_impl.rs`
  (~30 LOC test fixture + .stderr file) as the explicit negative-arm
  regression test backstop. Update V1-FROZEN-INTERFACE.md item 8
  verification-mechanism bullet to cite the actual test path.
- **v1-beta posture:** the hard-seal MECHANISM is structurally enforced by
  rustc (verified by the absence of any external `impl CapabilityPolicy`
  passing the workspace build at HEAD); only the explicit negative-arm
  regression test fixture is deferred. The freeze contract advertises a
  trybuild test at V1-FROZEN-INTERFACE.md §8 "What 'frozen' means here"
  subsection ('Explicit negative-arm trybuild' bullet) that does not
  exist as a separate file; this row plugs the named-destination
  phantom per HARD RULE 12 clause-(b). (Path-anchor cite per pim-1 /
  §3.5b HARDENED point 3; the previous V1-FROZEN-INTERFACE.md:717 cite
  has drifted to line 742 as the doc grew through G-CORE-9 fix-pass
  cycles — the subsection-anchor reference is line-stable.)
- **Anchor:** L6-r1-3 G-CORE-9 R1 finding (no triage disposition recorded);
  L6-r2-1 G-CORE-9 R2 finding ratifying the deferral per Fork 2 doc-tighten
  precedent.

### Row D-16 — V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md authorship

- **Frozen surface (v1-beta):** V1-FROZEN-INTERFACE.md item 4
  references `docs/V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md` as the
  Ben-signed P-III decision-point artifact.
- **Deferred consumption (G-COMP-1 destination OR pre-`v1-beta`
  tag):** ratified path (b) at G-CORE-9 R2 per L18-r2-4 disposition —
  the `docs/V1-WIRE-FORMAT-INVENTORY.md` doc IS the Ben-decision
  deliverable (rename, or attach a sign-off appendix to the inventory).
  Path (b) collapses the distinction — Ben signs off on the inventory's
  P-III sign-off block (already present at `V1-WIRE-FORMAT-INVENTORY.md`
  §"P-III Ben decision-point") rather than authoring a separate doc.
- **v1-beta posture:** the inventory IS authored + tracked; the Ben
  sign-off path is the inventory's own §"P-III Ben decision-point"
  section. V1-FROZEN-INTERFACE.md item 4 references the inventory + the
  build-backlog row 8.f acknowledges the inventory IS the Ben-decision
  deliverable.
- **Anchor:** L17-r1-7 + L18-r2-4.

### Row D-21 — `crates/benten-crypto-suite/INTERNALS.md` authorship

- **CLOSED 2026-05-24 at R6-FP-D L16-MAJOR-1 close.** The
  `crates/benten-crypto-suite/INTERNALS.md` file was authored at
  R6-FP-D (this PR) following the
  `crates/benten-platform-foundation/INTERNALS.md` structural template;
  the row remains here in the DEFERRED ledger for archaeology
  (commit history grep-able). The companion `crates/benten-drop/INTERNALS.md`
  was also authored at the same time (closes the implicit follow-up
  for the Drop crate).
- **Frozen surface (v1-beta):** the `benten-crypto-suite` crate is
  item-6-locked at V1-FROZEN-INTERFACE.md (codepoint table + public
  surface frozen at G-CORE-9). The INTERNALS.md doc has no v1-beta
  signature impact; it is internal architecture-record only — the
  authorship at R6-FP-D is doc-coupling completeness, not a freeze
  contract change.
- **Anchor:** spec item 6 + V1-FROZEN-INTERFACE.md item 15.d + the
  rename pair at #1344 row 7 (GrantKeyMaterial / AeadKeyMaterial) +
  L18-r1-5 + L18-r2-3 + R6-FP-D L16-MAJOR-1.

---

### Row D-25 — V1-BETA-BREAKING-CHANGES.md ledger completion sweep

- **Frozen surface (v1-beta):** none — this row is a doc-completion
  obligation, not a code-surface change. The v1-beta wire bytes + the
  v1-beta public API are wholly set; what is deferred is the
  *enumeration audit-trail* in the breaking-changes ledger.
- **Deferred consumption (G-COMP-1 destination):** author per-PR
  Cohort 2 rows in `docs/V1-BETA-BREAKING-CHANGES.md` for the ~13
  substrate-canary + Strategy-C-consolidation PRs identified at the
  R6 R1 L18 phase-wide lens:
  - **Substrate canaries (8 PRs, ~+321 pub surface):** #1319 G-CORE-3a
    CANARY (KeyMaterial + AeadEnvelope + structural_kdf + X-Wing wrap);
    #1323 G-CORE-3d (per-Node AEAD + two-CID map + per-chunk AEAD
    ≥64 KiB); #1324 G-CORE-3b (RestrictedSpec + AuthorizationGrant +
    Scope + chain validator — the original mint; rename row already
    enumerated at #1344); #1307 G-CORE-2 (signature-agility integration
    crate mint, +101 pub); #1309 G-CORE-5 (D3 VersionDag unification);
    #1311 G-CORE-4 (D1 CanonicalViews + IVM 5-arm + materializer walk
    + §4.6 vocab); #1312 G-CORE-7 (install-lifecycle hardening across
    6 §4.x backlog rows); #1325 Strategy-C wave-1 batch (DSL chunk-1 +
    G-CORE-3w walker + G-CORE-10 Option-C + G-CORE-6a verify-pass).
  - **DSL chunk asymmetry (1 PR):** #1326 DSL chunk-2 (closes #663 +
    #760 + #929 + #931 + #934); the chunk-3 #1339 row is already
    enumerated at Cohort 2 line 138 — chunks 1+2 are the asymmetric gap.
  - **Wave-1/2 Strategy-C (1 PR landed; 1 abandoned):** #1235 (MERGED;
    incl. benten-graph trait shape change — verbatim break-OK signal).
    Note: #1237 was CLOSED-not-merged (not part of the v1-beta ledger
    landing-set; the underlying scope was absorbed into the broader
    Strategy-C drain batches enumerated below). Earlier R6-R2-FP-C-pre
    drafts erroneously cited #1237 as if merged; corrected per
    R6-R2-FP-C cite-grep-verify discipline (§3.6j extension).
  - **#707-trust-subset (1 PR):** #1251 (device-revocation/recheck
    parallel pipes collapse; precursor to the #1271 chain-validation
    seam consolidation row already enumerated).
  - **Strategy-C drain batches (6 PRs, net -14 pub surface from
    deletions):** #1261 + #1262 + #1269 + #1277 + #1282 + #1290 —
    can roll up into 1 row "Strategy-C refinement-audit drain — net
    -14 pub-surface across 6 batches" with per-batch PR-cite list.
- **v1-beta posture:** the v1-beta-tag artifact is whole; the ledger
  is incomplete-but-not-misleading (the existing 15 PRs ARE
  ratified-honestly enumerated; the gap is enumeration coverage, not
  factual error). Downstream consumers reading the ledger see the
  Cohort 5 cross-reference + can consult `git log
  phase-4-foundation-close..HEAD` + the per-PR PR-body for the
  uncovered set. The L18 lens's positive-confirmation findings
  (l18-r6-3/5/6) verify the existing entries' honesty.
- **Forward-protection (brief-template mandate, mirror of Row D-22
  sub-task 6):** every future fix-pass PR authoring brief MUST
  include as a §3.5b post-fix-doc-coupling pre-flight item:
  "If the PR introduces a public-API surface change OR a wire-format
  byte-shape change, enumerate the change in
  `docs/V1-BETA-BREAKING-CHANGES.md` as a new Cohort row (or extend
  an existing row) in the SAME PR. Failure to enumerate creates a
  same-shape recurrence vs L18 R6 R1 phase-wide enumeration gap." This
  mandate lands at R5-BRIEF-pim-checklist.md authorship (per L15-MAJOR-2
  pim-checklist consolidation) so the rule fires forward at G-COMP-1
  wave authoring time.
- **Anchor:** R6-FP-D L18-r6-1 + l18-r6-2 path-(b) closure (Ben/
  orchestrator preferred path-(b) over path-(a) full enumeration for
  cycle-budget; both are HARD RULE 12 compliant; R4b-FP commit
  8240a56c machinery proven for path-(b)). Cross-cite from
  `docs/V1-BETA-BREAKING-CHANGES.md` Cohort 5 (the cite line landed in
  this same commit).

### Row D-23 — §4-B G-CORE-3 × G-CORE-4 SubgraphSpec live-eval + IVM CanonicalViews subscription invalidation test pin

- **Frozen surface (v1-beta):** the SubgraphSpec primitive (G-CORE-3w
  walker_as_subgraph + RestrictedScope/SubgraphSpecError mints) +
  the IVM CanonicalViews 5-arm seam (G-CORE-4) + the live-per-request
  resolver-evaluation commitment from RATIFIED-S&C R5 D-4M-R5
  ("sub-graph SHAPES that evolve, not frozen snapshots; resolver
  reuses CanonicalViews subscription") all carry frozen signatures
  at v1-beta. The composition seam — the resolver-consumer
  subscribing through IVM CanonicalViews on a SubgraphSpec scope —
  has no public-API delta beyond the already-frozen pieces.
- **Deferred consumption (G-COMP-1 destination):** ship the
  cross-wave integration test pin at
  `crates/benten-engine/tests/cross_wave_3_x_4_subgraphspec_live_eval_ivm_canonical_views.rs`
  per R2-test-landscape.md §4-B. Test shape: Alice grants Bob a UCAN
  scoped to a SubgraphSpec; Alice writes a new Recipe matching the
  spec; the IVM CanonicalViews subscription correctly emits a
  ChangeEvent that the resolver consumer hears; Bob's next request
  returns the new Recipe (live-per-request semantics, NOT frozen
  snapshot). ~150-250 LOC; the R5 G-CORE-3w + G-CORE-4 substrates
  are merged so the test substrate is fully available at HEAD.
  Couples to Row D-10 (§15.j live-per-request resolver-evaluation
  test pin) — both pins exercise the same RATIFIED-S&C R5 semantic
  from different angles (Row D-10 = walk_share_scope enumeration;
  Row D-23 = SubgraphSpec×CanonicalViews ChangeEvent propagation).
- **v1-beta posture:** the composition IS structurally available at
  v1-beta (every substrate is shipped); only the integration test
  pin is deferred. Per L1 R4b finding: tf5_431_ivm_inner_kernel_read_5arm_byte_equivalence.rs
  exercises 5-arm byte-equivalence between SubgraphSpec-routed walk
  + legacy walk (a DIFFERENT property; not the live-eval invalidation
  semantic).
- **Anchor:** R2-test-landscape.md §4-B + RATIFIED-S&C R5 D-4M-R5
  (`.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`,
  orchestrator-local) + R4b L1 finding r4b-l1-1
  (`.addl/phase-4-meta/r4b-l1-test-coverage.json` lens JSON on
  origin/phase-4-meta-core/r4b-l1-test-coverage).

### Row D-24 — §4-C G-CORE-3 × G-CORE-7 manifest-envelope ∩ UCAN-gated SubgraphSpec scope intersection test pin

- **Frozen surface (v1-beta):** G-CORE-3b chain_validator (RestrictedScope
  + ChainValidationError mints + AuthorizationGrant typed seal) +
  G-CORE-7 install-lifecycle hardening (ProductionManifestEnvelopeRechecker
  port + install-time consent) + the manifest-envelope ∩
  UCAN-SubgraphSpec scope-intersection semantic (CLAUDE.md #18
  three-layer consent: install-time envelope AND runtime UCAN AND
  chain-traces-to-user-root all must admit). All sub-pieces carry
  frozen signatures at v1-beta; no new public-API surface for the
  composition.
- **Deferred consumption (G-COMP-1 destination):** ship the
  cross-wave integration test pin at
  `crates/benten-engine/tests/cross_wave_3_x_7_manifest_envelope_intersects_ucan_scope.rs`
  per R2-test-landscape.md §4-C. Test shape (adversarial): install
  plugin P with manifest scope {A,B}; grant P a UCAN scoping {B,C};
  P's effective scope = {B} (intersection); requests for A or C
  return typed OutOfScope (E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE
  + the existing ChainValidationError surface). ~150-250 LOC; both
  G-CORE-3b chain_validator + G-CORE-7 install path are merged so
  the composition test substrate is fully available at HEAD.
  Couples to Row D-3 (3 §8-E CapabilityPolicy hooks consumption) —
  the per_delegation hook is the runtime arm of the intersection
  semantic.
- **v1-beta posture:** the substrates IS structurally available at
  v1-beta. tf3b_chain_validator_narrowing.rs covers chain-validator
  narrowing in isolation; tf7_g_core_7_install_lifecycle_hardening.rs
  covers install-time manifest semantics in isolation; the
  COMPOSITION pin (both must admit; intersection semantics) is the
  §4-C gap that Row D-24 names. Per the existing 3-layer admission
  (Layer-1 user-root + Layer-2 install-envelope + Layer-3 runtime
  UCAN) the composition semantic IS already structurally enforced
  at the chain_validator level; the integration test pin is the
  forward-protection / regression-defense surface that's deferred.
- **Anchor:** R2-test-landscape.md §4-C + CLAUDE.md baked-in #18
  trust model + R4b L1 finding r4b-l1-2.

### ~~Row D-22~~ — workspace `pub fn .*_for_test` / `_for_testing` `#[cfg]` gating sweep — **CLOSED at R6 R1 FP-A Bundle F1** (2026-05-24)

> **STATUS: RETRACTED / CLOSED.** Per Ben 2026-05-24 PM ratification of
> HARD RULE 12 over the R4b L6-MAJOR-1 path-(b) defer ("do the full
> ~13-site cascade now"), the workspace cfg-gating sweep LANDED at
> R6 R1 FP-A Bundle F1.a (cfg attributes) + F1.b (testing feature
> additions) + F1.c (CI workflow updates) + F1.d (no-regression test
> pin at `tests/phase_3_workspace/for_test_symbols_are_feature_gated.rs`
> with EXEMPT_PUB_ITEMS table) + F1.e (8 cargo-public-api baseline
> regens). 70+ `pub fn .*_for_test*` declarations cfg-gated; 14
> production-shaped items added to the EXEMPT_PUB_ITEMS allow-list.
> Row retained for forensic context per pim-13 / §3.12. The original
> deferred-consumption body below is preserved verbatim.

- **Frozen surface (v1-beta):** 115 baseline entries across 6
  cargo-public-api baselines (`docs/public-api/benten-caps.txt` 37 +
  `docs/public-api/benten-crypto-suite.txt` 52 +
  `docs/public-api/benten-drop.txt` 14 + `docs/public-api/benten-core.txt`
  8 + `docs/public-api/benten-sync.txt` 3 +
  `docs/public-api/benten-graph.txt` 1) lock the as-shipped public
  surface that carries `_for_test` / `_for_testing` constructors,
  helpers, and impls (≈84 distinct `pub fn` declarations in source
  across `benten-caps` 20 + `benten-crypto-suite` 30 +
  `benten-core` 7 + `benten-sync` 5 + `benten-drop` 7 +
  `benten-graph` 15; baseline > source count reflects re-export +
  trait-impl duplication). This freezes the *shape* (the names + the
  signatures) so a downstream `#[cfg(any(test, feature = "testing"))]`
  gating sweep is a visibility-only change, not a signature break.
- **Deferred consumption (G-COMP-1 destination):**
  1. Per-site sweep: wrap each `pub fn` / `pub const fn` declaration
     ending in `_for_test` / `_for_testing` (and the surrounding
     `impl` block where the helper is associated) in
     `#[cfg(any(test, feature = "testing"))]` — following the
     precedent at `crates/benten-core/src/lib.rs::Cid::sample_for_test`
     (`#[cfg(any(test, feature = "testing"))]`-gated per its own
     docstring; G-CORE-2 substrate cascade ratified pattern).
  2. Add `testing = []` feature to those crates currently missing
     it (`benten-crypto-suite` + `benten-drop` + `benten-sync`).
     (`benten-caps`, `benten-core`, `benten-graph` already carry
     `testing = []` (`benten-graph` chains `["benten-core/testing"]`);
     the new features chain the cross-crate fixture deps:
     `benten-drop/testing = ["benten-crypto-suite/testing",
     "benten-caps/testing", "benten-core/testing"]` etc.)
  3. Dev-deps cascade: every `[dev-dependencies]` entry of every
     consumer crate that USES a `_for_test` symbol from a sibling
     crate adds `<sibling>/testing` to its feature list. Workspace
     grep `grep -rl '_for_test\|_for_testing' crates/*/tests crates/*/benches`
     enumerates ≈205 consumer files across ~14 crates as of HEAD —
     each consumer's Cargo.toml updates the feature spec on the
     sibling dev-dep entry (the body of the test changes ZERO).
  4. Regenerate the 6 affected cargo-public-api baselines under
     `cargo +nightly public-api --simplified -p <crate> 2>/dev/null
     > docs/public-api/<crate>.txt`; the diff strips the 115
     `_for_test` / `_for_testing` lines that the production target
     no longer exposes.
  5. Add a no-regression test pin at
     `crates/phase-3-workspace-tests/tests/g_core_9_for_test_cfg_gating_audit.rs`
     that AST-walks (or grep-walks) the 6 baseline files, asserts
     ZERO occurrences of `for_test` / `for_testing` in their pub
     surface, and asserts every new `_for_test` / `_for_testing`
     declaration in any `crates/<X>/src/` carries a `#[cfg]` attribute
     matching the canonical pattern. The pin fires on the next
     baseline diff that re-introduces the suffix.
  6. Update R5-BRIEF-common.md (in `.addl/phase-4-meta/`, gitignored
     orchestrator-local) to enumerate the `_for_test` cfg-gating
     discipline as a literal pre-flight checklist line per §3.6g —
     pin "Any new `pub fn` ending in `_for_test` / `_for_testing`
     MUST carry `#[cfg(any(test, feature = "testing"))]` gating per
     V1-FROZEN-INTERFACE.md:154 + precedent
     `Cid::sample_for_test`; new declarations without the cfg
     attribute FAIL the no-regression pin from sub-task 5."
  7. Update V1-FROZEN-INTERFACE.md:154 narrative from "Public
     surface MUST NOT carry `_for_test` suffixes" to add the
     v1-beta carve-out: "Public surface MUST NOT carry `_for_test`
     suffixes at v1-GM; the v1-beta cargo-public-api baselines
     carry 115 such surfaces as a Row D-22 deferred-consumption
     debt; new declarations MUST be `#[cfg]`-gated per the
     no-regression pin at
     `crates/phase-3-workspace-tests/tests/g_core_9_for_test_cfg_gating_audit.rs`."
- **v1-beta posture:** the 115 surfaces are PRODUCTION-ABI-EXPOSED at
  v1-beta — a downstream consumer compiling against the v1-beta
  baselines CAN reach `Cid::sample_for_test`, `Scope::synthetic_for_test`,
  `KeyMaterial::generate_recipient_keypair_for_test`,
  `AuthorizationGrant::synthetic_for_test`, etc. and they all return
  semantically-valid fixtures. This is a HARD RULE 12 clause-(b)
  acknowledgement of the discipline gap surfaced at G-CORE-9 R4b L6:
  the FREEZE-time triage caught the 1-site `Engine::resolve_subgraph_cid_for_test`
  cluster (Row D-7) but missed the workspace-pattern bug; the only
  forward-protection at v1-beta is the no-regression pin (sub-task 5).
  No security property degrades — the helpers all construct valid
  fixtures with random/deterministic data; the gap is brand discipline
  / API-cleanliness, not runtime-safety.
- **Pivot rationale (R4b L6-MAJOR-1 / R4b-FP-1 pivot 2026-05-24,
  orchestrator decision under night-shift stance, rebuttable at next
  morning review):** R4b L6-MAJOR-1 named two paths: (a)
  FIX-NOW orchestrator-direct sweep (close all 115 sites + 6 baseline
  regens + ~205 consumer dev-dep updates in this PR), (b)
  DEFER-NAMED-NOW to NEW Row D-22 (this row). Path (a) hit the brief's
  hard-escalation trigger ("Bundle R4b.1 cfg-gating cascade breaks
  >30 callsites without clean fix") — the cascade touches ≈205
  consumer files. Path (b) preserves all forward-protection (the
  no-regression pin at sub-task 5 + brief-template line at sub-task 6
  block recurrence) while sequencing the per-site sweep into G-COMP-1
  alongside the existing baseline-regeneration cadence at Row D-7
  (the §8-A Engine visibility cluster). The visibility-only nature
  of the gating means the sweep is mechanical at G-COMP-1; no
  signature breaks, no API additions, no behavioral change. Path (b)
  also satisfies the L6 finding's per-finding granularity discipline
  by enumerating the count + crate breakdown + named no-regression
  pin (the pattern §3.6b sub-rule 4 sub-clause 2 prescribes for
  workspace-pattern bugs).
- **Anchor:** R4b L6-MAJOR-1 (`.addl/phase-4-meta/r4b-l6-per-finding-granularity.json`)
  + V1-FROZEN-INTERFACE.md:154 + precedent
  `crates/benten-core/src/lib.rs::Cid::sample_for_test`
  `#[cfg(any(test, feature = "testing"))]` gating pattern.

---

## Cross-cutting v1-beta posture

The v1-beta-shipped binary structurally enforces:
- 12-operation-primitive irreducibility (CLAUDE.md #1)
- Content-addressed CIDv1 + DAG-CBOR + BLAKE3 hashing (#5)
- PQ-hybrid Ed25519⊕ML-DSA-65 signature default + X25519⊕ML-KEM-768
  encryption default + ChaCha20-Poly1305 bulk (#5)
- 14-invariant production-runtime (Phase-3 R6 closure)
- Layer-1 user-as-root via `CapabilityPolicy::pre_write` (Compromise
  #2 sync-replica sub-narrative)
- Layer-3 envelope-shape recheck SEAM (Compromise #26 seam-half;
  substantive consumption deferred per Row D-4)
- Empty-peer-DID structural fail-CLOSED at apply_atrium_merge (live)
- Per-row cap-revocation recheck at apply_atrium_merge (live via
  G16-B-F PR #161)
- Sealed `CapabilityPolicy` trait + workspace-test
  `__sealed_for_workspace_tests` re-export under `testing` feature
- Atrium peer-mesh networking (Phase-3 close)
- Multi-device support via signed device-DID-attestation envelope V2
  (Phase-3 G16-D wave-6b)
- C11b pure-PQ sole-trust-path gate (the audit-gated constructor
  is structurally enforced; production reachability blocked behind
  `AUDIT_LANDED_PURE_PQ_FLAG = false`)
- Strip-resistant hybrid signature both-must-verify +
  commitment-recompute (F-1 defense)
- AAD-binds-plaintext-CID rebinding-attack defense (cipher-suite
  layer)
- X-Wing combiner binding both KEM halves + both encapsulated keys
  + both public keys for strip-resistance (#5 hybrid encryption)

The v1-beta-shipped binary does NOT structurally enforce:
- All deferred rows above (D-1 through D-16)
- Items in Compromise #26's "NOT live" enumeration

The v1-GM tag is gated on the independent ml-dsa + ml-kem audit
(NF-2 / C-GM-AUDIT) per Compromise #30. The audit-landing closes
Row D-15's audit-readiness concern.

---

### Row D-26 — G-CORE-PQ-WIRE wave: PQ-hybrid app-layer wire-in for all identity-bearing surfaces

- **Frozen surface (v1-beta):** 4 production sites currently
  classical-only Ed25519 (32-byte verifying-key bytes + 64-byte
  signature) at the app layer:
  - `crates/benten-drop/src/envelope_sig.rs::sign_envelope` +
    `verify_envelope` (DropBundle envelope signature)
  - `crates/benten-platform-foundation/src/plugin_manifest.rs::PluginManifest::verify_peer_signature`
  - `crates/benten-platform-foundation/src/plugin_manifest.rs::InstallRecord::verify_user_signature`
  - `crates/benten-caps/src/authorization_grant.rs::AuthorizationGrant::binding_sig`
    (already named at Row D-15e; re-homed to this wave per
    same-structural-sub-fork analysis)

  Plus the `benten_id::Keypair` classical-only structure (~25-30
  workspace call sites; structurally couples to whichever sub-fork
  HOW choice is adopted).

- **Deferred consumption (G-CORE-PQ-WIRE wave; Ben 2026-05-24 PM
  ratification):** wire `benten_crypto_suite::SignatureSuite`
  hybrid signing + verifying at ALL identity-bearing surfaces in a
  dedicated multi-wave initiative. Sub-fork HOW choice between:
  - **(α)** additive sibling pubkey field per site (smallest
    structural delta; preserves classical-only downgrade arm)
  - **(β)** envelope v1→v2 version bump per site (cleaner per-site
    but breaks v1 readers)
  - **(γ)** DID-extension carries hybrid pubkey bytes (cleanest
    structurally; cascades through `benten_id::Keypair`)

  The wave's R0 design pre-work selects between α/β/γ based on
  cross-site cascade analysis (see R6-R1-FP-E-HARD-ESCALATION.md
  path-analysis doc on E's branch for the LOC budgets + HEAD-verified
  cascade depths).

- **Wave naming:** `G-CORE-PQ-WIRE` — sequence with Phase-4-Meta-Core
  R6 R1 FP cycle completion → R6 iteration to strict-Q5 → then
  G-CORE-PQ-WIRE wave dispatch (pre-tag wire-format window absorbs
  the wire change; same window that absorbed Agent B's F3 AAD
  total_chunks fix at R6 R1 FP-B).

- **v1-beta posture:** the v1-beta-tag artifact ships PQ-hybrid at
  the crypto-suite SUBSTRATE layer (per CLAUDE.md #5 + RATIFIED-pq-
  default-reframe-2026-05-19); the app-layer SHIPPED state at
  v1-beta is classical-only Ed25519 for the 4 sites above. This is
  the SAME classical-floor-under-audited-security posture per CLAUDE.md
  v1-GATE addition (PQ-hybrid is non-sole-trust at app layer; the
  classical Ed25519 layer is itself the audited security floor; the
  hybrid layer is defense-in-depth + post-quantum future-proofing).
  Compromise #30 narrative honestly discloses this gap with NAMED
  destination = this G-CORE-PQ-WIRE wave (NOT G-COMP-1 as Compromise
  #30's pre-2026-05-24-PM narrative suggested).

- **Forward-protection:** per `feedback_orchestrator_defer_prediction_bias`,
  the wave dispatch MUST be sequenced + sized (not perpetually
  deferred). Wave R0 brief deadline = post R6 R1 FP consolidation +
  R6 R2 dispatch (the natural window after the current FP cycle
  settles). Per Ben 2026-05-24 PM "do everything now is really my
  default stance" — the wave is queued ACTIVE, not exploratory.

- **Anchor:** L2-R6-MAJOR-2 G-CORE-9 R1 finding (the 3 sites) +
  Row D-15e (binding_sig sub-fork) + Agent E's R6-R1-FP-E HARD-ESCALATION
  fork-analysis doc on branch `phase-4-meta-core/r6-r1-fp-e-pq-hybrid-app-layer-wire`
  at SHA `83096e39` + Ben 2026-05-24 PM "do everything now" ratification
  (`.addl/phase-4-meta/MORNING-QUEUE-2026-05-24-PM.md` + this session's
  defer-bias memo codification at `feedback_orchestrator_defer_prediction_bias.md`).

---

### Row D-27 — StampedValue per-row originating-grant_cid plumbing for multi-hop attribution preservation

- **Frozen surface (v1-beta):** `apply_atrium_merge` reconstructs
  `AttributionFrame` fresh per merged row at `crates/benten-engine/src/engine.rs:1629`
  using whichever `effective_actor_cid` the LOCAL device computes —
  the ORIGINATING peer's authorization-grant CID does not survive the
  sync hop. The `StampedValue` wire envelope (`crates/benten-sync/src/crdt.rs:174-181`)
  carries `(value, hlc)` only; it has NO per-row grant_cid slot.
  Multi-hop chain consumers therefore see the laptop's fresh-reconstructed
  attribution, not the phone's original grant.

  This is the **wire/persistence half** of the Path-G/Row-D-3-c
  AttributionFrame work. Path G (this PR cycle) substantively
  populates the EXISTING `AttributionFrame.capability_grant_cid` slot
  (`crates/benten-eval/src/exec_state.rs:77`; currently zero-Cid
  sentinel at `engine.rs:1634`) for LOCAL-origin writes via
  `WriteContext.authorizing_grant_cid` propagation — ~30-50 LOC; zero
  wire-shape change; only the semantic contract tightens. Row D-27 is
  what Path G does NOT close: the MULTI-HOP preservation across
  apply_atrium_merge boundaries.

- **Deferred consumption (G-COMP-1):** extend `StampedValue` →
  `StampedValueV2 { value, hlc, originating_grant_cid: Option<Cid> }`
  + rewire the apply_atrium_merge fresh-reconstruct pattern at
  engine.rs:1629 to PRESERVE the originating grant_cid (rather than
  overwrite with local-device attribution) + extend
  `ProductionManifestEnvelopeRechecker` (Compromise #26 / Row D-4
  destination) to walk the preserved grant_cid through the existing
  single rechecker port at engine.rs:1456-1507. ~600-800 LOC across
  benten-sync (StampedValue extension; opaque-bytes-via-LoroValue::Binary
  per crdt.rs:174-181 + 392-410 + 848 — NO Loro upstream coordination
  needed; opaque-bytes is fully Benten-controlled at the codec layer)
  + benten-engine apply_atrium_merge semantic redesign + ChainResolver
  port co-ship + ledger-row backward-compat fallback (fresh-reconstruct
  on None).

- **Real defer rationale (corrected 2026-05-25 by FINAL cross-lens
  ground-truth-verify against HEAD `31d1a169`):** the apply_atrium_merge
  fresh-reconstruct pattern is a **semantic redesign**, not a Loro
  upstream-coordination cost (the earlier path-f-wire-format-relay-lens
  framing was factually incorrect about Loro coord requirements; see
  `.addl/phase-4-meta/path-f-sync-merge-arch-FINAL-cross-lens.json`
  for the corrected analysis). The semantic redesign couples to the
  G-COMP-1 chain-walker consumer (currently
  `NoopManifestEnvelopeRechecker` admits all per Compromise #26
  PARTIAL closure) — landing Row D-27 standalone before the chain-walker
  is wired would mint a slot no consumer reads. Same cohort as the
  ProductionManifestEnvelopeRechecker substantive impl + GrantResolver
  port + Row D-4 closure.

- **Path-G/Path-E lens triangulation provenance (2026-05-25):** Path-F
  investigation surfaced 3 lens variants (F-i row-properties / F-ii
  AttributionFrame slot populate / F-iii sync envelope sibling) which
  converged after FINAL cross-lens onto **Path G = F-ii substantively
  populate the EXISTING slot at v1-beta + Row D-27 = F-iii wire envelope
  extension deferred to G-COMP-1**. 3-lens unanimous on Path G after the
  predecessor sync-merge-arch lens self-corrected on 2 factual errors:
  (1) `CapWriteContext` does NOT have `#[non_exhaustive]` at HEAD
  (doc-comment at `crates/benten-caps/src/policy.rs:151-161` shows
  G-COMP-1-deferred — Row D-17 cascade is therefore MANDATORY in the
  WIRE-NOW batch, not assumed-done); (2) Loro StampedValue is opaque
  Benten-DAG-CBOR bytes encoded as `LoroValue::Binary` (verified at
  crdt.rs:848) — not a coord blocker; real defer-rationale is the
  semantic redesign above.

- **v1-beta posture:** Path G's local-grant_cid threading at v1-beta
  closes the audit-trail half of the Row D-3-c context (LOCAL-origin
  writes have substantive `capability_grant_cid` instead of zero-Cid
  sentinel); Row D-27's multi-hop preservation closure ships at
  G-COMP-1. Row D-3-c itself remains APPROPRIATELY-PARTIAL-CLOSED per
  Delta-v3-2 (audience_did=None at apply_atrium_merge sites is
  semantically correct because peer_did is transport-principal NOT
  cap-target).

- **Anchor:** Path-F lens triangulation JSONs at
  `.addl/phase-4-meta/path-f-{sync-merge-arch,crypto-cap,wire-format-relay,sync-merge-arch-FINAL-cross-lens}-lens.json`
  + the 4 LATE-MORNING addenda in `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-25.md`
  + the 9-item WIRE-NOW batch ratified for PR #1356 (item 4 = Path G
  AttributionFrame slot population).

### Row D-28 — Compute-marketplace wire types (`PeerResource` / `ResourceKind` / `OwnerRef`) — PHASE-LATER-DEFER

- **Deferred surface:** the compute-marketplace wire types — `PeerResource`
  graph Nodes, `ResourceKind { Compute | Storage | Bandwidth | Availability }`,
  `OwnerRef { Member | Community | ThirdParty }` — are **ABSENT from the frozen
  wire at v1-beta** (F-FREEZE-1 PIN 5 asserts the absence). They are
  **PHASE-LATER-DEFER** to Phase-5+ (the compute / economics pillar).

- **Why deferred (and free to defer):** the compute marketplace is NOT
  v1-beta-gating (it adds ZERO frozen wire field — §4.4 O-8) and is
  **graph-native when activated** (`PeerResource` is an ordinary signed graph
  Node), so activating it is an additive application-layer build on the frozen
  v1 substrate, never a wire-format break.

- **Destination doc:** `docs/future/compute-marketplace.md` (R0.7 §2.8 CE-1 +
  §4.3) carries the full Phase-5+ design.

- **v1-beta posture:** absent from the frozen wire; minted as a named deferral
  here so the deferral is registry-discoverable (the §3.12 R7-equivalent audit
  walk + the F-FREEZE-1 / F-DISC-2 doc-coupling gates).

### Row D-29 — Economic-policy composition surface (`CommunityEconomicPolicy`; `economic_policy` DROPPED) — PHASE-LATER-DEFER

- **Deferred surface:** the economic-policy composition surface —
  the signed **`CommunityEconomicPolicy`** Node + the Credits-ledger-as-graph
  + UCAN-caveat composition. **PHASE-LATER-DEFER** to Phase-5+.

- **`economic_policy` reserve DROPPED (CE-1 H2):** there is **ZERO economic
  freeze hook** on `MembershipSetPolicy` — the `economic_policy: Option<opaque>`
  reserve is DROPPED because economics COMPOSES from { UCAN caveats +
  Credits-ledger-as-graph + signed `CommunityEconomicPolicy` Node } with ZERO
  MembershipSet wire field (F-FREEZE-1 PIN 1 asserts `MembershipSetPolicy`
  carries no `economic_policy` field). "local-free" is a default `local_rate`
  policy-field, not a hard engine rule.

- **Destination doc:** `docs/future/compute-marketplace.md` (§2.8 / §4.3).

- **v1-beta posture:** ZERO frozen hook; graph-native when activated; minted as
  a named deferral here for registry-discoverability.

---

### Row D-30 — `DeviceLinkError::SessionIdReplayed` production replay-store wire-in (F-full R6 R1 finding F-13)

- **Frozen surface (v1-beta):** the typed error variant
  `DeviceLinkError::SessionIdReplayed` exists at
  `crates/benten-engine/src/layer_d/device_link.rs:185` (the Signal-Provisioning
  device-link replay-defense arm).
- **Replay defense is DEFERRED — NOT exercised on the production path (R9-council
  F-06 correction).** `open_provisioning_payload`
  (`crates/benten-engine/src/layer_d/device_link.rs`) takes NO nonce /
  session-id-cache argument and NEVER consults a replay store — it can return
  `SessionIdMismatch` but never `SessionIdReplayed`. The
  `SessionIdReplayed` variant has **no production emitter at v1-beta**. The test
  `f_ld_4_model_only_session_id_replay_pin`
  (`crates/benten-engine/tests/f_ld_4_multi_device_key_wrap_provisioning.rs`)
  is a **model-only pin** — it drives a test-local `HashSet` (`consumed.insert`)
  to pin the intended reject shape; it does NOT call
  `open_provisioning_payload`, so it is NOT a production-path exercise (pim-18
  SHAPE-not-SUBSTANCE). The prior "exercised by the layer_d test suite" claim was
  inaccurate and is retracted here.
- **Deferred consumption (G-COMP-1 destination):** wire the PRODUCTION
  session-id replay store — the durable consumed-session-id set that the
  device-link verify path consults so a replayed provisioning session id fires
  `SessionIdReplayed` against real persisted state — INCLUDING adding the
  nonce/session-id-cache argument to `open_provisioning_payload` so it can emit
  `SessionIdReplayed`. The replay-defense LOGIC is typed (the variant exists);
  the production storage binding (per-engine persistent consumed-session-id set,
  pruned by a **local policy window referencing `granted_at_bucket`** — the coarse
  1-hour grant-time bucket on the offer; there is **NO `exp` field** on the
  `ProvisioningOffer` wire, so any prior "offer `exp` window" phrasing was
  inaccurate and is corrected here (R10 F-06)) + the verify-path wiring are the
  G-COMP-1 deliverable coupled to the device-link UX flow (Phase-4-Meta-Composing).
- **Offer authentication is DEFERRED device-link-UX hardening (R10 F-12).** The
  `ProvisioningOffer` (the QR B publishes: `version`, `device_b_fingerprint`,
  `provisioning_session_id`) is NOT itself authenticated at v1-beta — nothing
  signs the offer, and A's out-of-band fingerprint confirmation is the only
  binding (and its enforcement is itself deferred; see the R10 F-05 addition
  below). Offer-authentication (binding the offer to B's confirmed device
  identity so a swapped offer is rejected) is deferred device-link-UX hardening,
  co-designed with the G-COMP-1 device-link flow.
- **Active swap-before-seal MITM-substitution enforcement is DEFERRED to G-COMP-1
  (R10 F-05).** The out-of-band device fingerprint
  (`ProvisioningOffer::device_b_fingerprint` + the `fingerprint_recipient` helper,
  `crates/benten-engine/src/layer_d/device_link.rs`) is the intended defense
  against an active MITM that swaps B's pubkey in the offer BEFORE A seals. At
  v1-beta this defense is **HUMAN-FINGERPRINT-dependent and NOT wired into the
  production seal/open path**: `device_b_fingerprint` / `fingerprint_recipient`
  have **zero production consumers** (exercised only by the F-LD-4 test corpus),
  and nothing in `seal_provisioning_payload` binds B's human-confirmed identity to
  the pubkey A actually seals to. The `HpkeUnwrapFailed` recipient-confidentiality
  property (a payload sealed to B does not decrypt under any other secret) is a
  REAL but WEAKER guarantee — it rejects at OPEN time, it does NOT prevent A from
  sealing to a swapped-before-seal pubkey. `device_b_fingerprint` /
  `fingerprint_recipient` are a **RESERVED SEAM** (register-then-enforce, like Row
  D-64 / D-52) — the genuine planned surface for the deferred fingerprint MITM
  defense; retained intentionally (do NOT delete; folding the fingerprint into the
  signed bytes would be a wire change, deferred with the wiring). The §10.2-HIGH
  test (`f_ld_4_recipient_confidentiality_wrong_secret_rejects_k_principal_exfil`)
  is corrected (R10 F-05) to pin recipient-confidentiality, the property it
  actually exercises.
- **v1-beta posture:** the typed reject variant + the in-band session-id-MATCH
  check are present (a substituted payload whose inner session-id disagrees fires
  `SessionIdMismatch`); the residual is BOTH the durable replay-store binding AND
  the verify-path argument that lets `SessionIdReplayed` fire at all. No exploit
  at v1-beta on a single trusted engine (sessions are short-lived; the
  session-id-mismatch arm already rejects substituted payloads).
- **Anchor:** `crates/benten-engine/src/layer_d/device_link.rs::DeviceLinkError::SessionIdReplayed`
  + the Signal-Provisioning device-link flow (Phase-4-Meta-Composing UX wave).

---

## R6-R2 (post-F-full phase-close, round 2) NAMED-CARRY rows

> The rows below land at the R6-R2 phase-close convergence (council run
> `wf_040ac861-436`, artifact main `d0ccf606`; triage
> `.addl/phase-4-meta/R6-R2-TRIAGE-FFULL.md`). Each is a HARD-RULE clause-(b)
> deferral whose ENTRY lands NOW with a NAMED destination; the substantive
> change ships in the named downstream wave. All cites verified live at HEAD
> `35fb9ad6` (post-B2 / PR #1366) at author-time.

### Row D-31 — `privacy.rs::network_observer_can_link_stanzas` input-independent predicate → v1-GM substantive scan (R6-R2 F-14)

- **Frozen surface (v1-beta):**
  `crates/benten-membership-set/src/privacy.rs::network_observer_can_link_stanzas(stanza_a_wire, stanza_b_wire)`
  documents its two-byte-slice arguments as "load-bearing" (the predicate is a
  property of the wire form, and "the answer would have to flip to `true` if a
  recipient identifier ever leaked onto the wire"), but the body is
  **input-independent**: it `let _ = (stanza_a_wire, stanza_b_wire);` then
  unconditionally returns `false`. The function never inspects the wire bytes,
  so it cannot actually detect a recipient-identifier leak — the regression it
  is documented to guard against (`f_nat_2` arm) is asserted by the SHAPE
  (two-slice signature) not the SUBSTANCE (a real cross-stanza-linkage scan).
- **Deferred consumption (v1-GM destination):** make the predicate genuinely
  input-dependent — scan the two stanza wire byte-slices for any stable
  cross-stanza linkage marker (a shared recipient slot / non-blinded field) and
  return `true` iff such a marker is present, so a future regression where a
  recipient identifier leaks onto the wire flips the answer to `true` and the
  `f_nat_2` arm fails. Couples to the §3.6f-ext substantive-arm discipline
  (would-FAIL-on-revert).
- **v1-beta posture:** no exploit at v1-beta — the per-recipient stanza wire
  form IS blinded (the recipient slot is keyed off `K_Set`; the AEAD wrap
  exposes no recipient identifier in the clear), so the property the predicate
  DOCUMENTS holds at v1-beta by construction. The gap is test-substance, not a
  live-confidentiality hole: the predicate is a correct constant TODAY because
  the wire genuinely carries no marker, but it would NOT catch a future
  regression that introduced one.
- **Anchor:** R6-R2 council finding F-14;
  `crates/benten-membership-set/src/privacy.rs:38` +
  Inv-20 clause-d (m-7) + Compromise #58 honest-disclosure boundary.

### Row D-32 — `f_aad_2_nine_tuple_*` test-file name stale (9-tuple → BLINDED 11-field) (R6-R2 F-21)

- **Frozen surface (v1-beta):** the test file
  `crates/benten-membership-set/tests/f_aad_2_nine_tuple_injectivity_opaque_boundary.rs`
  retains the `nine_tuple` token in its FILENAME, but the `0x6610` group
  per-stanza AAD it pins is now the **BLINDED 11-field set** (the migration
  from the raw 9-tuple with a plaintext roster + raw set-id is fully documented
  in the file body, which uses `11-field` / `BLINDED 11-field` throughout). The
  filename is the sole residual `nine_tuple` reference; the test bytes + golden
  are the canonical 11-field shape (127-byte M-20 golden).
- **Deferred consumption (test-hygiene fix-pass destination):** rename the file
  to `f_aad_2_eleven_field_injectivity_opaque_boundary.rs` (or equivalent
  `11_field` token) + update the cross-cites in
  `docs/V1-FROZEN-INTERFACE.md` §16 carve-out #5 +
  `crates/benten-membership-set/src/lib.rs` / any rustdoc that names the file.
  A file rename is a git-mv (not a doc-entry-now fix), hence the carry.
- **v1-beta posture:** zero functional impact — the test is green and pins the
  correct 11-field bytes; the filename is cosmetic-stale. No wire/golden change.
- **Anchor:** R6-R2 council finding F-21;
  `crates/benten-membership-set/tests/f_aad_2_nine_tuple_injectivity_opaque_boundary.rs`
  (body documents the 9-tuple → 11-field migration) +
  `docs/V1-FROZEN-INTERFACE.md` §16 carve-out #5 cite.

### Row D-33 — `f_ld_3` "R5-FOLD-IN" forward-looking comments stale (R5 is past) (R6-R2 F-22)

- **Frozen surface (v1-beta):**
  `crates/benten-engine/tests/f_ld_3_execute_workflow_aad_binding.rs` carries
  forward-looking "R5-FOLD-IN" / "folds at R5" / "R5 un-ignores" / "R5-DESTINATION"
  comments (e.g. the module-level note at `:45` + the per-test note at
  `:205`) that describe the AAD-version-coherence fold-in as a FUTURE R5 step.
  R5 is past (the stub-shim was deleted + the real
  `benten_engine::layer_d::remote_permission` is wired live at `:81`/`:86`);
  the comments are stale forward-tense in landed-and-green territory.
- **Deferred consumption (comment-retense fix-pass destination):** retense the
  R5-prefixed comments to as-built / past-tense (the
  `pim-N`/§3.6e RED-PHASE-and-forward-looking-comment retense discipline; same
  class as R6-R1 finding F-19 which retensed stale RED-PHASE doc-comments on
  landed+green tests). Confirm the `aad_version=0x01` byte-0 prefix coherence
  claim still matches the enclosing `PermissionRequest` envelope before editing.
- **v1-beta posture:** zero functional impact — the test is live + green; only
  the explanatory comments are stale-tense.
- **Anchor:** R6-R2 council finding F-22;
  `crates/benten-engine/tests/f_ld_3_execute_workflow_aad_binding.rs:45` +
  `:205` (R5-FOLD-IN / R5-DESTINATION notes); R6-R1 F-19 retense precedent.

### Row D-34 — wasm-browser bundle forbidden-symbol blocklist coverage for F-full full-peer-only deps (R6-R2 F-23)

- **Frozen surface (v1-beta):** the browser-bundle content-audit guard in
  `.github/workflows/wasm-browser.yml` (the "Bundle-content audit (forbidden
  symbols per CLAUDE.md baked-in #17)" step) blocks the 4 full-peer-only crate
  prefixes `forbidden=( "loro" "iroh" "redb" "wasmtime" )`. The F-full wave
  added the 15th crate `benten-membership-set`, whose PRODUCTION tree carried a
  native-only `benten-sync` → iroh/loro edge until the F-02 option-(b) refactor
  moved it to a dev-dependency (so `benten-drop` is now sync-free). The
  blocklist was NOT re-audited against the F-full crate graph to confirm no new
  full-peer-only symbol prefix can leak into the thin-client browser bundle.
- **Deferred consumption (CI-hardening fix-pass destination):** re-audit the
  F-full crate graph (`benten-membership-set` / `benten-drop` / `benten-id` /
  `benten-crypto-suite` libcrux/sha3 native primitives) against the
  baked-in-#17 full-peer-vs-thin-compute boundary; extend the
  `forbidden=( ... )` array (and the `bindings/napi/tests/wasm_bundle_content.rs`
  pin) with any newly-introduced full-peer-only prefix that must never appear in
  `bindings/napi/dist/browser/benten_engine.wasm`. If the audit confirms the
  existing 4-prefix set is sufficient (no new full-peer dep reaches the browser
  bundle), record that confirmation + close this row.
- **v1-beta posture:** the existing 4-prefix guard is ACTIVE + fail-closed; the
  carry is a completeness re-audit of the F-full additions, not a known leak.
  The drop tree is already sync-free (cargo tree -e normal) per the F-02
  option-(b) refactor, so the most likely outcome is a confirmation.
- **Anchor:** R6-R2 council finding F-23;
  `.github/workflows/wasm-browser.yml` forbidden-symbol blocklist +
  `bindings/napi/tests/wasm_bundle_content.rs` + CLAUDE.md baked-in #17.

### Row D-35 — freeze-record "HEAD `84280d31`" snapshot-SHA currency sweep (R6-R2 F-26) — RE-PINNED at R6-R3 (F-33/F-67)

> **STATUS (R6-R4, re-pinned from R6-R3):** the deferred re-pin is **DONE for this round** — the snapshot-SHA cites in `docs/ERROR-CATALOG.md` (~4), `docs/V1-FROZEN-INTERFACE.md` (~4), and `docs/INVARIANT-COVERAGE.md` (~4) are re-pinned to the current freeze-record HEAD `b93b2efc` (origin/main this round; the R6-R3 round had them at `fdfda621`). The AS-BUILT claims + count narratives (199 throwable / 201 catalog) were ground-truth-re-verified at `b93b2efc` before re-pinning (the only commit advancing HEAD past `fdfda621` was #1368, which left `CATALOG_VARIANT_COUNT == 199` unchanged). The historical SHA mentions BELOW (the `2172cb6d` → `84280d31` → `fdfda621` → … provenance chain) are intentionally retained as the audit trail. The pin re-floats to the freeze-tag HEAD at the final pre-tag freeze-record sweep if HEAD advances again before the Ben-gated `phase-4-meta-core-close` tag.

- **Frozen surface (v1-beta):** three freeze-record docs cite the snapshot SHA
  `84280d31` as "HEAD" / "AS-BUILT + ENFORCED at HEAD" — `docs/ERROR-CATALOG.md`
  (catalog-count narrative, ~4 cites), `docs/V1-FROZEN-INTERFACE.md` (CATALOG
  count + TS-catalog + provenance, ~4 cites), `docs/INVARIANT-COVERAGE.md`
  (Inv-16..22 AS-BUILT header + body, ~4 cites). HEAD has since advanced
  (`84280d31` → `d0ccf606` #1365 → `35fb9ad6` #1366); the "HEAD `84280d31`"
  framing is a stale snapshot reference. The two intervening commits are
  freeze-affecting (the F-02 11-field group-AAD canonicalization + F-01
  truncation defense + the B2 sealed-sender origin-authentication), so the
  snapshot SHA should re-pin to the current freeze-record HEAD at the next
  freeze-record sweep.
  - **Inv-21 `::crdt` path sub-item — VERIFIED CORRECT, no carry:** the
    `docs/INVARIANT-COVERAGE.md` Inv-21 row cites the LARGER-HLC-wins LWW in
    `crates/benten-sync/src/crdt.rs` (the `cmp_lex != Greater` keep-rule) +
    `benten_sync::crdt::LoroDoc::winning_attribution`. Ground-truth-verified at
    HEAD `35fb9ad6`: the `cmp_lex`/`Greater` LWW keep-rule is present in
    `crdt.rs`; `LoroDoc::winning_attribution` exists; the `crdt` module is
    declared `pub mod crdt` in `crates/benten-sync/src/lib.rs` so the
    `benten_sync::crdt::…` path resolves. The `::crdt` path is NOT drifted —
    only the `84280d31` snapshot-SHA currency carries forward.
- **Deferred consumption (freeze-record sweep destination — Agent-B
  freeze-record docs / next reconcile pass):** re-pin the `84280d31`
  snapshot-SHA cites across `ERROR-CATALOG.md` / `V1-FROZEN-INTERFACE.md` /
  `INVARIANT-COVERAGE.md` to the then-current freeze-record HEAD as part of the
  next freeze-record reconcile sweep (the same class as R6-R1 F-03/F-04/F-18
  which re-pinned `2172cb6d` → `84280d31`). Routed as a CARRY (not a fix-now)
  because the canonical re-pin target is the freeze-tag HEAD, which is itself
  Ben-gated (the `phase-4-meta-core-close` tag is HOLD-Ben) — re-pinning to a
  mid-flight SHA would just drift again at tag-time. The pin should land in the
  freeze-record sweep that immediately precedes the tag.
- **v1-beta posture:** no functional impact — the AS-BUILT claims are TRUE
  (the substrate shipped + is green); only the snapshot-SHA token is stale. The
  count narratives (199 throwable / 201 catalog) remain accurate at HEAD.
- **Anchor:** R6-R2 council finding F-26; `docs/ERROR-CATALOG.md` +
  `docs/V1-FROZEN-INTERFACE.md` + `docs/INVARIANT-COVERAGE.md` `84280d31`
  cites; R6-R1 `2172cb6d` → `84280d31` re-pin precedent (#1365).

> **NOTE — `docs/SECURITY-POSTURE.md` + `docs/V1-WIRE-FORMAT-INVENTORY.md` are
> OWNED BY THE R6-R2 FREEZE-RECORD AGENT (Agent B) this wave.** Any `84280d31`
> snapshot-SHA cites in those two docs are NOT swept by Row D-35 (this row owns
> only ERROR-CATALOG / V1-FROZEN-INTERFACE / INVARIANT-COVERAGE); the Agent-B
> shard handles the freeze-record currency of SECURITY-POSTURE +
> V1-WIRE-FORMAT-INVENTORY.

---

## R6-R3 (post-F-full phase-close, round 3) NAMED-CARRY rows

> The rows below land at the R6-R3 phase-close convergence (doc-reconciliation +
> OBS shard). Each is a HARD-RULE clause-(b) deferral OR an OBS/disclosure
> observation whose ENTRY lands NOW with a NAMED destination; the substantive
> change (or the decision that none is needed) ships in the named downstream
> wave / is dispositioned by a downstream reviewer. Code-behavior + cite/comment
> OBS rows are recorded here so a later reviewer or the G-COMP-1 wave can pick
> them up; they are NOT freeze-blocking at v1-beta. All cites verified live at
> HEAD `fdfda621` at author-time. The bare `F-NN` labels are the R6-R3 council
> finding IDs carried conservatively from the council brief.

### Row D-36 — GAP-D: `audience_set_commitment` is unkeyed → membership-guess-confirmable (sharpens U25 / SECURITY-PROOFS §4.2)

- **Observation (NAMED, not fixed this round):** the Layer-C group `audience_set_commitment` (`crates/benten-drop/src/layer_c.rs`, the BLAKE3 commitment over the *sorted* recipient roster) is **unkeyed** — a party that can enumerate candidate rosters can **confirm** a guessed audience by recomputing the commitment (the recipient-INDEPENDENT-recompute soundness property that defends re-target is the same property that makes a low-entropy roster guess-confirmable). This is the audience-axis sibling of the deterministic-CEK confirmation-oracle (SECURITY-PROOFS §4.2 / GAP-2) and sharpens the U25 full-per-send-unlinkability reserve.
- **Destination:** SECURITY-PROOFS §4.2 disclosure extension + the U25 v1-GM-reserve (a keyed/blinded audience commitment is additive over the wire — codepoint-reserve, no v1-beta wire-break). A downstream reviewer decides whether to add the explicit §4.2 disclosure note OR fold under U25; recorded here so the guess-confirmability is not silent.
- **v1-beta posture:** no network-observer plaintext break (the roster is not on the wire; the commitment is opaque to an observer who cannot enumerate the candidate set); the residual is the enumerable-low-entropy-roster confirmation advantage, same class as §4.2.

### Row D-37 — GAP-E: `classical_half_for_test` / `pq_half_for_test` ungated in the frozen crypto-suite baseline (PRE-EXISTING-on-main)

- **Observation (NAMED, not fixed this round):** the frozen `cargo-public-api` baseline `docs/public-api/benten-crypto-suite.txt` carries `benten_crypto_suite::sig::HybridSignature::classical_half_for_test` + `::pq_half_for_test` (plus `SyntheticVector::ml_dsa65_for_test` + `StructuralKdfKey::from_bytes_for_test`) as ungated `pub` `_for_test` accessors — i.e. they are in the v1-beta frozen public surface, not behind `#[cfg(test)]` / a `testing` feature gate. **PRE-EXISTING-on-main** (not introduced by R6-R3).
- **Destination:** the `_for_test` visibility-cluster v1-Composing tightening pass (same class as the §15.f `derive_step_without_info_tag_for_test` → `_internal` BELONGS-NAMED-NOW item already recorded in `docs/V1-FROZEN-INTERFACE.md` §15.f). A downstream reviewer decides gate-vs-rename-vs-accept; locked AS-IS at v1-beta per the freeze (a gate/rename is itself a public-surface change requiring a baseline-update + Ben sign-off).

### Row D-38 — GAP-F: `--omit blanket-impls` regen-determinism doc vs committed crypto-suite baseline

- **Observation (NAMED, not fixed this round):** `docs/V1-FROZEN-INTERFACE.md` (~L865) + `docs/V1-FROZEN-INTERFACE-BUILD-BACKLOG.md` row 1 document the baseline regen command as `cargo public-api -p <crate> --simplified --omit blanket-impls`, but the committed `docs/public-api/benten-crypto-suite.txt` baseline **contains auto-trait blanket-impl lines** (`impl<'a> core::marker::{Freeze,Send,Sync,Unpin,RefUnwindSafe,UnwindSafe} for …`) that `--omit blanket-impls` would strip. So the committed baseline was NOT produced with `--omit blanket-impls` (or the documented command is wrong) — a regen-determinism inconsistency: a fresh `--omit blanket-impls` regen would NOT byte-match the committed file, which weakens the "regen MUST produce byte-identical file" contract claim.
- **Destination:** the freeze-record build-out / cargo-public-api regen pass (the wave that owns `docs/public-api/*.txt`). Reconcile by EITHER re-regenerating all baselines WITH the documented flag-set (and re-committing the stripped output) OR correcting the documented command to match the committed baselines (drop `--omit blanket-impls` from the doc). A code/baseline change, so NAMED-not-fixed in this doc-only shard.

### Row D-39 — R6-R3 code-behavior OBS cluster (F-26 / F-39 / F-40 / F-41 / F-43 / F-44 / F-46 / F-60 / F-61 / F-62 / F-63 / F-64)

- **Observation (NAMED, not fixed this round):** the R6-R3 council surfaced a cluster of **code-behavior** observations (finding IDs F-26, F-39, F-40, F-41, F-43, F-44, F-46, F-60, F-61, F-62, F-63, F-64). These describe runtime / construction-site behaviors (NOT doc-reconciliation), so they are out of scope for the doc-only shard that authored this row and are carried for a code-owning reviewer.
- **Destination:** the R6-R3 code-fix shard (the sibling shard that owns Rust src) / the next phase-close convergence round. Each ID is dispositioned by the code reviewer as fix-now / OUT-OF-SCOPE / NAMED-downstream per HARD RULE 12 when picked up. Recorded here so none is silently dropped between rounds (pim-N-prior-phase-explicit-preflight discipline).
- **Per-ID breakdown (12 constituent IDs; uniform disposition = OUT-OF-SCOPE for this doc shard — each is a code-behavior item owned by the sibling CODE shard / Rust src, byte-correct at HEAD, not freeze-gating):**
  - `F-26` — out-of-scope (code-behavior; CODE shard)
  - `F-39` — out-of-scope (code-behavior; CODE shard)
  - `F-40` — out-of-scope (code-behavior; CODE shard)
  - `F-41` — out-of-scope (code-behavior; CODE shard)
  - `F-43` — out-of-scope (code-behavior; CODE shard)
  - `F-44` — out-of-scope (code-behavior; CODE shard)
  - `F-46` — out-of-scope (code-behavior; CODE shard)
  - `F-60` — out-of-scope (code-behavior; CODE shard)
  - `F-61` — out-of-scope (code-behavior; CODE shard)
  - `F-62` — out-of-scope (code-behavior; CODE shard)
  - `F-63` — out-of-scope (code-behavior; CODE shard)
  - `F-64` — out-of-scope (code-behavior; CODE shard)

### Row D-40 — R6-R3 cite/comment/disclosure OBS cluster (F-10 / F-11 / F-16 / F-17 / F-20 / F-27 / F-29 / F-31 / F-32 / F-34 / F-35 / F-37 / F-38 / F-42 / F-45 / F-49 / F-50 / F-51 / F-52 / F-53 / F-54 / F-56 / F-57 / F-59 / F-65 / F-66)

- **Observation (NAMED, not fixed this round):** the R6-R3 council surfaced a cluster of **cite / comment / disclosure** observations (finding IDs F-10, F-11, F-16, F-17, F-20, F-27, F-29, F-31, F-32, F-34, F-35, F-37, F-38, F-42, F-45, F-49, F-50, F-51, F-52, F-53, F-54, F-56, F-57, F-59, F-65, F-66). These are minor cite-precision / comment-accuracy / disclosure-completeness observations that a downstream reviewer can address; they are not freeze-blocking and were below the fix-now threshold for this shard (which prioritized the MAJOR + cheap-cite fixes named in its brief).
- **Destination:** the next phase-close convergence round / the freeze-record reconcile sweep that precedes the Ben-gated tag. Each ID is dispositioned when picked up. Recorded here so the cluster survives between rounds.
- **Per-ID breakdown (26 constituent IDs; uniform disposition = named-carry — each is a cite-precision / comment-accuracy / disclosure-completeness item, byte-correct at HEAD, not freeze-gating):**
  - `F-10` — named-carry (cite/comment/disclosure)
  - `F-11` — named-carry (cite/comment/disclosure)
  - `F-16` — named-carry (cite/comment/disclosure)
  - `F-17` — named-carry (cite/comment/disclosure)
  - `F-20` — named-carry (cite/comment/disclosure)
  - `F-27` — named-carry (cite/comment/disclosure)
  - `F-29` — named-carry (cite/comment/disclosure)
  - `F-31` — named-carry (cite/comment/disclosure)
  - `F-32` — named-carry (cite/comment/disclosure)
  - `F-34` — named-carry (cite/comment/disclosure)
  - `F-35` — named-carry (cite/comment/disclosure)
  - `F-37` — named-carry (cite/comment/disclosure)
  - `F-38` — named-carry (cite/comment/disclosure)
  - `F-42` — named-carry (cite/comment/disclosure)
  - `F-45` — named-carry (cite/comment/disclosure)
  - `F-49` — named-carry (cite/comment/disclosure)
  - `F-50` — named-carry (cite/comment/disclosure)
  - `F-51` — named-carry (cite/comment/disclosure)
  - `F-52` — named-carry (cite/comment/disclosure)
  - `F-53` — named-carry (cite/comment/disclosure)
  - `F-54` — named-carry (cite/comment/disclosure)
  - `F-56` — named-carry (cite/comment/disclosure)
  - `F-57` — named-carry (cite/comment/disclosure)
  - `F-59` — named-carry (cite/comment/disclosure)
  - `F-65` — named-carry (cite/comment/disclosure)
  - `F-66` — named-carry (cite/comment/disclosure)
- **Note — F-58 = NO-ACTION:** the `repr(u8)` frozen-cardinality carve-out is correct as-built; no row needed (carried here only to record the explicit NO-ACTION disposition so it is not re-raised).

---

## R6-R4 (post-F-full phase-close, round 4) NAMED-CARRY rows

> The rows below land at the R6-R4 phase-close convergence (doc shard B — cites +
> named-carry + threat-model rows). Each is a HARD-RULE clause-(b) deferral OR an
> OBS/disclosure observation whose ENTRY lands NOW with a NAMED destination; the
> substantive change (or the decision that none is needed) ships in the named
> downstream wave / is dispositioned by a downstream reviewer. They are NOT
> freeze-blocking at v1-beta. All cites verified live at HEAD `b93b2efc` at
> author-time. The bare `C-NN` labels are the R6-R4 council finding IDs.

### Row D-41 — C-04: `ExecuteWorkflow.input_node_cids` bound into no signed/AEAD surface (input-READ-scope binding → v1-GM)

- **Observation (NAMED, not fixed this round):** `crates/benten-engine/src/layer_d/remote_permission.rs::ExecuteWorkflow` carries `input_node_cids: Vec<[u8; 32]>` but `constraint_aad()` binds only the frozen 3-field tuple `(executor_did, max_decrypt_count, result_recipient_pubkey)` — the workflow input READ-scope is NOT cryptographically bound at v1-beta. No live exploit (runtime ExecuteWorkflow enforcement is post-v1-beta per NQ-T3; the variant is reserved / typed-rejected at the dispatch boundary). See `docs/THREAT-MODEL.md` §4 NQ-T3-adjacent note.
- **Destination:** input-READ-scope AAD/signature binding → **v1-GM**, co-designed with the NQ-T3 runtime no-egress enforcement it travels with. Freezing the binding shape now is premature.

### Row D-42 — C-07: `u16` Layer-C sender-lp asymmetry freeze-note

- **Observation (NAMED, not fixed this round):** the Layer-C sender length-prefix carries a `u16`/`u32` width asymmetry that warrants an explicit freeze-note (which width is wire-locked where) so a future refactor cannot silently widen/narrow the prefix.
- **Destination:** the freeze-record reconcile sweep that precedes the Ben-gated tag / the sibling CODE shard if a width change is judged needed (no change expected — this is a note-the-asymmetry row).

### Row D-43 — C-08: plaintext-sender golden pin → folds into G-COMP-1 Row D-9

- **Observation (NAMED, not fixed this round):** the plaintext-sender (`LAYER_C_DROP = 0x6500`) golden-bytes pin is part of the deferred hex-pin cohort.
- **Destination:** **G-COMP-1, Row D-9** (the 6-deferred-hex-pin cohort) — recorded here so the plaintext-sender golden is not dropped from that cohort.

### Row D-44 — C-09: `0x6380` AAD-dispatch base note

- **Observation (NAMED, not fixed this round):** the `0x6380` AAD-dispatch band base wants an explicit base-note in the codepoint/AAD-dispatch narrative (what the band roots, why it is distinct from the neighbouring bands).
- **Destination:** the codepoint-allocation doc reconcile sweep (`docs/CRYPTO-CODEPOINTS.md`) at the pre-tag freeze-record pass.

### Row D-45 — C-12: Inv-16 `const`-true assertion → round-trip arm

- **Observation (NAMED, not fixed this round):** an Inv-16 (codepoint-dispatch) test arm asserts a `const`-true shape rather than driving a real round-trip; it should be strengthened to a substantive round-trip arm (§3.6f regression-guard-substantive-arm discipline).
- **Destination:** the sibling CODE shard / next phase-close convergence round (lives under a Rust test target — not edited by this doc shard).

### Row D-46 — C-13: F-MST-3 part-2 → v1-GM

- **Observation (NAMED, not fixed this round):** the F-MST-3 MST-conformance follow-up (part 2) is a v1-GM-scoped item.
- **Destination:** **v1-GM** MST-conformance lane.

### Row D-47 — C-19: tf3a `wasm32-unknown` → `wasm32-wasip1` header currency

- **Observation (NAMED, not fixed this round):** `crates/benten-crypto-suite/tests/tf3a_pq_hybrid_wasm32_roundtrip.rs` pin-source header prose carries stale `wasm32`-target framing; the test now runs under **wasm32-wasip1** via the `crypto-suite-wasm-roundtrip` CI job (`.github/workflows/wasm-conformance.yml`). Header currency only — the test + CI gating are correct.
- **Destination:** the comment/cite-accuracy reconcile sweep (a Rust test-file header — adjacent to the CODE shard) at the pre-tag pass.

### Row D-48 — C-20: 32-bit-`usize` drop exercise → v1-GM CI

- **Observation (NAMED, not fixed this round):** a 32-bit-`usize` drop/exercise lane (catching `usize`-width assumptions on 32-bit targets) is not in v1-beta CI.
- **Destination:** **v1-GM CI** (32-bit target lane).

### Row D-49 — C-22: V1-FROZEN item 2 `.d.ts` count `15` → `14`

- **Observation (NAMED, not fixed this round):** `docs/V1-FROZEN-INTERFACE.md` item 2 (TS public-API parity gate) says the structural diff covers "all 15 `dist/*.d.ts` files"; the baseline `packages/engine/etc/public-api.txt` covers **14** real `.d.ts` modules (atrium, dsl, engine, errors, errors.generated, identity, index, manifest, mermaid, sandbox, stream, subscribe, types, views). Stale count `15` → `14`.
- **Destination:** the freeze-record cite-precision reconcile sweep that precedes the Ben-gated tag.

### Row D-50 — C-23 / C-24 / C-25: remaining doc-staleness cluster

- **Observation (NAMED, not fixed this round):** the R6-R4 council surfaced a residual cluster of minor doc-staleness items (finding IDs C-23, C-24, C-25) — cite-precision / comment-accuracy observations below the fix-now threshold for this shard (which prioritized the MAJOR + named cite fixes in its brief).
- **Destination:** the next phase-close convergence round / the freeze-record reconcile sweep that precedes the tag. Each ID is dispositioned when picked up. Recorded here so none is silently dropped between rounds (pim-N-prior-phase-explicit-preflight discipline).
- **Per-ID breakdown (3 constituent IDs; uniform disposition = named-carry — minor doc-staleness / cite-precision / comment-accuracy, below the fix-now threshold, not freeze-gating):**
  - `C-23` — named-carry (doc-staleness / cite-precision)
  - `C-24` — named-carry (doc-staleness / cite-precision)
  - `C-25` — named-carry (doc-staleness / cite-precision)

### Row D-51 — C-26: gossip §3.9 `FLAGGED-FOR-BEN` rustdoc residue → pre-tag Ben item (UNRESOLVED, by design)

- **Observation (NAMED, not resolved this round — deliberately):** `crates/benten-membership-set/src/keying.rs::gossip_topic` rustdoc (line ~80) carries a `FLAGGED-FOR-BEN` note about the §3.9-vs-§3.10 gossip-topic derivation: §3.9 owns the gossip topic and its frozen primitive is the **no-label** `blake3::keyed_hash(K_Set, set_id ‖ BE(generation))` form (golden byte-confirmed), while §3.10's prose + the dispatch brief's M-20 line echoed the *labelled* commitment formula. This row NAMES the residue as a **pre-tag Ben item**; it does NOT resolve the §3.9-vs-§3.10 question (that is Ben's freeze-gated call — the golden + no-label form is authoritative at HEAD, but the prose discrepancy is surfaced for Ben's explicit sign-off before the tag).
- **Destination:** **Ben pre-tag freeze-decision** — the `FLAGGED-FOR-BEN` rustdoc stays until Ben rules on the §3.9/§3.10 prose reconciliation; the doc-prose clarification (the w-doc §3.10 clarification flagged in orchestration) lands with that ruling.

---

## R6-R5 (post-F-full phase-close, round 5) NAMED-CARRY rows

> The rows below land at the R6-R5 phase-close convergence (doc shard B —
> Inv-21 honest-disclosure + Composing deferral + doc-tense + named-carry). Each
> is a HARD-RULE clause-(b) deferral OR an OBS/disclosure observation whose ENTRY
> lands NOW with a NAMED destination; the substantive change (or the decision
> that none is needed) ships in the named downstream wave / is dispositioned by a
> downstream reviewer. They are NOT freeze-blocking at v1-beta. All cites verified
> live at HEAD `6d340944` at author-time. The bare `F-NN` labels are the R6-R5
> council finding IDs.

### Row D-52 — F-03: Inv-21 fork-tie-break production-merge wiring → Phase-4-Meta-Composing

- **Frozen surface (v1-beta):** the fork-tie-break COMPARATOR
  `crates/benten-membership-set/src/set.rs::crdt::{fork_total_order_key, fork_a_wins}`
  (`set.rs:194-216`) is AS-BUILT at HEAD and property-pinned (totality /
  antisymmetry / transitivity + smaller-`created_at_hlc`-wins asymmetry +
  archival-half) by the `F-INV21-*` proptest family at
  `crates/benten-membership-set/tests/f_inv21_fork_tie_break_totality_version_node_cid.rs`.
  The comparator + its proptest are FROZEN at Core.
- **Deferred consumption (Phase-4-Meta-Composing destination):** wire the
  comparator into the LIVE distributed merge path. At HEAD the comparator has
  **zero production callers** (verified by §3.5n grep 2026-06-07: the only
  non-test references are the `set.rs` definitions + their rustdoc); the live
  merge is benten-sync LWW (`crates/benten-sync/src/crdt.rs:535`,
  LARGER-HLC-wins), and the test-local `resolve_fork` in
  `f_inv21_fork_tie_break_totality_version_node_cid.rs` is the
  PRODUCTION-stand-in. The concurrent same-anchor set-creation fork scenario is
  exercised by the Composing distributed-sync path; that is where the comparator
  becomes load-bearing on the live merge.
- **v1-beta posture:** the SMALLER-`created_at_hlc`-wins set-identity-fork rule
  (Inv-21) is comparator-AS-BUILT + proptest-pinned but is NOT consumed by the
  live merge path in the shipped binary — `INVARIANT-COVERAGE.md` discloses this
  as the register-then-enforce carve-out (the same honest disclosure Inv-15
  uses). No live exploit at v1-beta: distributed concurrent same-anchor
  set-creation forks are a Composing-path scenario, not a Core single-node
  scenario; the LWW property merge over member properties (the live rule) is
  correct for its object class (`F-CRDT-3` pins the two-rule co-existence).
- **Anchor:** M-7 / M-8; Inv-21 row + the "register-then-enforce" carve-out in
  `docs/INVARIANT-COVERAGE.md`; F-03 R6-R5 council finding.

### Row D-53 — F-07..F-39: R6-R5 MINOR/OBS doc-tense / cite-currency / missing-pin / cosmetic cluster

- **Observation (NAMED, not fixed this round):** the R6-R5 phase-close council
  surfaced a residual cluster of MINOR/OBS findings (finding IDs **F-07 through
  F-39**) — doc-tense (forward-looking R5/R3-future prose where the phase is
  past), cite-currency (snapshot-SHA / line-anchor freshness below the fix-now
  threshold), missing-pin (a regression-guard pin named but not yet landed), and
  cosmetic (comment/disclosure wording) observations. **All are byte-correct at
  HEAD `6d340944`** — none is a freeze-gating defect; the wire/golden/codepoint
  bytes are correct, the findings are doc/comment/cite hygiene only. The MAJOR +
  named findings (F-01 domain-registry-prose verify, F-03 Inv-21 down-classify,
  F-04 Inv-16 typed-layer hygiene, F-05 ExecuteWorkflow result-seal doc-tense)
  were resolved in this shard; this row carries the sub-threshold remainder.
- **Destination:** the next phase-close convergence round / the freeze-record
  reconcile sweep that precedes the Ben-gated tag. Each `F-NN` ID is
  dispositioned when picked up (fixed-then-struck OR re-confirmed
  no-change-needed). Recorded here so none is silently dropped between rounds
  (pim-N-prior-phase-explicit-preflight + §3.6i no-defer discipline). Where a
  specific F-NN proves to be a code-adjacent item (e.g. a Rust test-file header
  or a missing test pin), it migrates to the sibling CODE shard / a Rust test
  target at pickup rather than being closed in this doc shard.
- **Per-ID breakdown (the `F-07..F-39` carry range = 33 constituent IDs; the
  MAJOR/named findings F-01/F-03/F-04/F-05 were RESOLVED in the R6-R5 shard and
  are NOT part of this carry. Uniform disposition = named-carry — doc-tense /
  cite-currency / missing-pin / cosmetic, byte-correct at HEAD `6d340944`, not
  freeze-gating; a code-adjacent F-NN migrates to the CODE shard / a Rust test
  target at pickup):**
  - `F-07` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-08` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-09` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-10` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-11` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-12` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-13` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-14` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-15` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-16` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-17` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-18` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-19` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-20` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-21` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-22` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-23` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-24` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-25` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-26` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-27` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-28` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-29` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-30` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-31` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-32` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-33` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-34` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-35` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-36` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-37` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-38` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)
  - `F-39` — named-carry (doc-tense / cite-currency / missing-pin / cosmetic)

### Row D-54 — R6-R6 doc/cite/OBS NAMED cluster (F-14 / F-15 / F-18 + F-19..F-37) — per-row disposition

- **Observation (NAMED, HARD-RULE clause-b; not fixed this round):** the R6-R6
  phase-close council surfaced a residual cluster of doc/cite/OBS findings —
  **F-14, F-15, F-18, and F-19 through F-37** — that fall below the fix-now
  threshold for SHARD B (which closed the two Ben-ratified freeze-forks F-04
  band-move + F-07 domain-tag registration, the MINOR cluster F-05/F-06/F-10,
  and the cheap cite/comment fixes F-11/F-12/F-13/F-16/F-17). Each carries one of
  the three valid HARD-RULE-12 dispositions — **named-carry** (doc-tense /
  cite-currency / cosmetic comment hygiene that is byte-correct at HEAD and not
  freeze-gating), **out-of-scope** (code-adjacent items owned by the sibling
  CODE shard / a Rust test target, migrated at pickup), or **disagree-with-
  explanation** (a finding the reviewer rebuts as already-correct). **All are
  byte-correct at HEAD** — none is a wire/golden/codepoint defect; the freeze
  bytes are correct and these are doc/comment/cite hygiene or code-shard items
  only.
- **F-27 / F-28 carve-out (LEAVE AS-IS — do NOT re-resolve here):** the `5 → 18`
  budget item (F-27 / F-28) is the already-tracked **Ben pre-tag ratification
  surface** — it is a Ben-gated freeze-decision held open by design, NOT a
  sub-threshold doc finding. It is intentionally not dispositioned in this row;
  it stays on the pre-tag Ben item list (the same posture as Row D-51's gossip
  §3.9 `FLAGGED-FOR-BEN` residue).
- **Destination:** the next phase-close convergence round / the freeze-record
  reconcile sweep that precedes the Ben-gated tag. Each `F-NN` ID is
  dispositioned when picked up (fixed-then-struck OR re-confirmed
  no-change-needed OR migrated to the CODE shard). Recorded here so none is
  silently dropped between rounds (§3.6i no-defer discipline +
  pim-N-prior-phase-explicit-preflight). Continues the Row D-39 / D-40 / D-53
  cluster-row precedent.
- **Anchor:** R6-R6 phase-close council; HARD-RULE clause-b; Row D-53 (R6-R5
  cluster) precedent.
- **Per-ID breakdown (constituent IDs = F-14, F-15, F-18, and F-19 through F-37.
  The SHARD-B-resolved findings F-04/F-05/F-06/F-10/F-11/F-12/F-13/F-16/F-17 are
  NOT part of this carry. Disposition class per the row prose = named-carry by
  default — doc-tense / cite-currency / cosmetic, byte-correct at HEAD; an item
  that proves code-adjacent re-classes to out-of-scope and migrates to the CODE
  shard at pickup, and a finding the reviewer rebuts re-classes to
  disagree-with-explanation. The class is fixed at pickup; it is NOT changed
  here):**
  - `F-14` — named-carry (or out-of-scope/disagree at pickup)
  - `F-15` — named-carry (or out-of-scope/disagree at pickup)
  - `F-18` — named-carry (or out-of-scope/disagree at pickup)
  - `F-19` — named-carry (or out-of-scope/disagree at pickup)
  - `F-20` — named-carry (or out-of-scope/disagree at pickup)
  - `F-21` — named-carry (or out-of-scope/disagree at pickup)
  - `F-22` — named-carry (or out-of-scope/disagree at pickup)
  - `F-23` — named-carry (or out-of-scope/disagree at pickup)
  - `F-24` — named-carry (or out-of-scope/disagree at pickup)
  - `F-25` — named-carry (or out-of-scope/disagree at pickup)
  - `F-26` — named-carry (or out-of-scope/disagree at pickup)
  - `F-27` — **CARVE-OUT (NOT dispositioned here):** the `5 → 18` exemption-budget
    Ben pre-tag ratification surface — Ben-gated freeze-decision held open by
    design; stays on the pre-tag Ben item list (see the F-27/F-28 carve-out note
    above + Row D-51).
  - `F-28` — **CARVE-OUT (NOT dispositioned here):** the `5 → 18` exemption-budget
    Ben pre-tag ratification surface — Ben-gated freeze-decision held open by
    design; stays on the pre-tag Ben item list (see the F-27/F-28 carve-out note
    above + Row D-51).
  - `F-29` — named-carry (or out-of-scope/disagree at pickup)
  - `F-30` — named-carry (or out-of-scope/disagree at pickup)
  - `F-31` — named-carry (or out-of-scope/disagree at pickup)
  - `F-32` — named-carry (or out-of-scope/disagree at pickup)
  - `F-33` — named-carry (or out-of-scope/disagree at pickup)
  - `F-34` — named-carry (or out-of-scope/disagree at pickup)
  - `F-35` — named-carry (or out-of-scope/disagree at pickup)
  - `F-36` — named-carry (or out-of-scope/disagree at pickup)
  - `F-37` — named-carry (or out-of-scope/disagree at pickup)

---

## R6-R7 (post-F-full phase-close, round 7) NAMED-CARRY rows

> The rows below land at the R6-R7 phase-close convergence (doc SHARD C — doc
> fixes + named-carry + cluster-row expansion). Each is a HARD-RULE clause-(b)
> deferral OR an OBS/disclosure observation whose ENTRY lands NOW with a NAMED
> destination; the substantive change (or the decision that none is needed)
> ships in the named downstream wave / is dispositioned by a downstream
> reviewer. None is freeze-blocking at v1-beta — all freeze (wire/golden/
> codepoint) bytes are correct at HEAD `81924331`; these are doc/comment/cite
> hygiene, test-completeness, or v1-GM-scoped items. All cites verified live at
> HEAD `81924331` at author-time.

### Row D-55 — LD-AUTH-1: `HeadlessDeviceAuth::seal_and_build` sentinel `user_did_signing_key` undisclosed (doc FLAG)

- **Observation (NAMED, not fixed this round):**
  `crates/benten-engine/src/layer_d/device_auth.rs:135` `seal_and_build` seals
  the vault with a HARDCODED sentinel `user_did_signing_key: vec![0x22u8; 64]`
  (device_auth.rs:145) — a placeholder, NOT a real DID signing key — but the
  docstring discloses only `k_principal` + `password` and does not flag that the
  signing-key slot is a sentinel. A caller could mistake the headless backend's
  unlocked key material for a usable signing key. Disposition = **named-carry →
  doc FLAG**. No live defect (the headless backend is a test/headless harness;
  the sentinel is byte-correct as-built).
- **Destination:** the freeze-record reconcile sweep that precedes the Ben-gated
  tag (docstring FLAG on `seal_and_build` disclosing the sentinel) / the sibling
  CODE shard since the docstring lives on a Rust src item — migrated at pickup.

### Row D-56 — D30-LINE-DRIFT: Row D-30 `device_link.rs` cite `:147` → `:185` (FIXED this round)

- **Observation (NAMED, FIXED this round):** Row D-30 cited
  `crates/benten-engine/src/layer_d/device_link.rs:147` for the
  `DeviceLinkError::SessionIdReplayed` variant; the variant has drifted to
  `device_link.rs:185`. Disposition = **named-carry (cite-currency)** — the cite
  in Row D-30 has been corrected to `:185` in this same shard. Recorded here so
  the drift + its fix are forensically visible. (cite-drift did not flag it: the
  file is 308 lines so `:147` still resolves to a line — content-drift, not a
  missing-line, which the detector does not catch.)
- **Destination:** CLOSED-in-this-shard (Row D-30 cite corrected). No downstream
  action required.

### Row D-57 — psf-3 / psf-4: SECURITY-PROOFS doc attribution + count precision

- **Observation (NAMED, not fixed this round):** two sub-threshold
  doc-precision findings on `docs/SECURITY-PROOFS.md` — psf-3 (an attribution /
  authorship-prose precision item) and psf-4 (a field/surface count precision
  item). Both are byte-correct cosmetic doc hygiene, not freeze-gating.
  Disposition = **named-carry (cite/attribution precision)**.
- **Destination:** the freeze-record cite-precision reconcile sweep that
  precedes the Ben-gated tag. Each dispositioned when picked up.

### Row D-58 — WFB-OBS-1: workflow-binding band-width annotation

- **Observation (NAMED, not fixed this round):** the workflow-binding (WFB)
  surface wants an explicit band-width annotation in the codepoint/AAD-dispatch
  narrative (which width the band roots, so a future refactor cannot silently
  widen/narrow it). Annotation-only — the as-built bytes are correct.
  Disposition = **named-carry (freeze-note annotation)**.
- **Destination:** the codepoint-allocation doc reconcile sweep
  (`docs/CRYPTO-CODEPOINTS.md`) at the pre-tag freeze-record pass.

### Row D-59 — xtw-1: 32-bit overflow-guard test → v1-GM

- **Observation (NAMED, not fixed this round):** a 32-bit-target overflow-guard
  test arm (catching `usize`/width-overflow assumptions on 32-bit targets) is
  not in v1-beta CI. Disposition = **named-carry → v1-GM** (sibling of Row D-48
  C-20 32-bit drop-exercise lane). No live defect on 64-bit targets.
- **Destination:** **v1-GM CI** (32-bit target lane) — co-routes with Row D-48.

### Row D-60 — RGC-TRANS1: drop-side #53 regression-guard transition upgrade

- **Observation (NAMED, not fixed this round):** the drop-side Compromise-#53
  regression guard wants an upgrade from its current arm to a substantive
  production-entry-point arm (§3.6f regression-guard-substantive-arm
  discipline). Disposition = **named-carry → out-of-scope for this doc shard**
  (lives under a Rust test target — migrated to the sibling CODE shard at
  pickup). Byte-correct at HEAD; not freeze-gating.
- **Destination:** the sibling CODE shard / next phase-close convergence round
  (Rust test target, not edited by this doc shard).

### Row D-61 — CONF-2 / gap-osp-1 / gap-osp-2: SECURITY-PROOFS §4.2 caller-trust + test-symmetry

- **Observation (NAMED, not fixed this round):** three findings on
  `docs/SECURITY-PROOFS.md` §4.2 (deterministic-CEK confirmation-oracle honest
  disclosure) — CONF-2 (sharpen the caller-trust boundary prose: who exactly
  holds the confirmation advantage vs who does not), gap-osp-1 + gap-osp-2 (a
  symmetric test arm demonstrating the oracle is bounded to a CEK-input-holder
  and grants the relay nothing). The §4.2 disclosure is already substantively
  correct; these tighten the prose + add a symmetry test. Disposition =
  **named-carry** (the CONF-2 prose sharpening is doc-shard; the gap-osp test
  arms are code-adjacent and migrate to the sibling CODE shard at pickup).
- **Destination:** the freeze-record reconcile sweep (CONF-2 §4.2 prose) +
  the sibling CODE shard / a Rust test target (gap-osp-1/2 test-symmetry arms).

### Row D-62 — F-LC3 hygiene: label-collision / loose-assert / mldsa-varint prose cluster

- **Observation (NAMED, not fixed this round):** an F-LC3 hygiene cluster —
  (a) test-label collisions (duplicate `#[test]`/section labels), (b) a
  loose-assertion arm (assert that should be sharpened to a substantive check),
  and (c) ML-DSA varint-encoding prose precision. All byte-correct at HEAD; doc/
  comment/test hygiene only, not freeze-gating. Disposition = **named-carry**
  (the mldsa-varint prose is doc-shard; the label-collision + loose-assert items
  are code-adjacent and migrate to the sibling CODE shard at pickup).
- **Destination:** the comment/cite-accuracy reconcile sweep (mldsa-varint
  prose) + the sibling CODE shard / Rust test targets (label collisions +
  loose-assert) at the pre-tag pass.

### Row D-63 — §16-FLAG discharge + HEAD-repin note

- **Observation (NAMED, not fixed this round):** the `docs/V1-FROZEN-INTERFACE.md`
  FLAG-FOR-BEN / FLAG-FOR-ORCHESTRATOR-REVIEW section (the §16-class flag) wants
  an explicit discharge note + a HEAD-repin (the snapshot SHA the flag's prose
  pins should advance to the current freeze HEAD `81924331`). Disposition =
  **named-carry → out-of-scope for THIS doc shard** — `V1-FROZEN-INTERFACE.md` is
  owned by SHARD B; this row only RECORDS the §16-FLAG discharge + HEAD-repin
  obligation so it is not silently dropped between rounds. No freeze-byte change.
- **Destination:** SHARD B (`docs/V1-FROZEN-INTERFACE.md` §16 FLAG section) / the
  freeze-record reconcile sweep that precedes the Ben-gated tag — discharge the
  flag + repin the snapshot SHA to the current freeze HEAD.

> **Anchor (R6-R7 cluster):** R6-R7 phase-close council; HARD-RULE clause-b;
> Row D-39 / D-40 / D-53 / D-54 cluster-row + named-carry precedent. The bare
> `LD-AUTH-1` / `D30-LINE-DRIFT` / `psf-N` / `WFB-OBS-1` / `xtw-1` / `RGC-TRANS1`
> / `CONF-2` / `gap-osp-N` / `F-LC3` / `§16-FLAG` labels are the R6-R7 council
> finding IDs.

### Row D-64 — Engine encrypt-to-recipient wiring → Phase-4-Meta-Composing (R9 GAP-1)

- **Frozen surface (v1-beta):** the Layer-C encrypt-to-recipient PRIMITIVE is
  COMPLETE + safe + exercisable. `benten_drop::layer_c` seal/open key off the
  REAL hybrid recipient key types (`RecipientPublic` / `RecipientSecret`,
  re-exported from `benten_crypto_suite::cipher_suite`); the placeholder
  `[u8; 32]` fingerprint + `sk = pk + 0x80` derivation is DELETED (R9 GAP-1).
  The 8 seal/open signatures + the `EncryptedEnvelope` wire are frozen (see
  `docs/V1-FROZEN-INTERFACE.md` §6 item 5a; machine-locked by
  `docs/public-api/benten-drop.txt`). The primitive is unit-exercised by the
  `f_lc_*` corpus.
- **Deferred consumption (Phase-4-Meta-Composing destination):** the
  **engine-level USE** of the frozen primitive. No engine flow CALLS
  encrypt-to-recipient at v1-beta. Composing must wire: (i) minting the
  recipient keypair as Principal identity key material (via
  `CipherSuite::generate_recipient_keypair`), (ii) vault-storing the
  `RecipientSecret` at rest (Layer-A vault seal), (iii) `benten-id` DID
  encryption-key resolution (resolve a recipient's advertised
  `RecipientPublic` from its `did:key`), and (iv) seal-side recipient-pub
  sourcing (the engine flow that hands a `&RecipientPublic` to `seal_*`).
- **v1-beta posture:** encrypt-to-recipient is a frozen, safe, standalone
  primitive with no engine caller. This is **NOT a Compromise** — the frozen
  surface is safe (real keying, fail-closed open); it is a not-yet-wired
  capability, deferred because the engine wiring co-designs with the
  Composing-phase Principal-identity + vault-storage + DID-resolution flows.
- **Anchor:** R9 GAP-1 closure; CLAUDE.md baked-in #18 (Principal
  confidentiality half); `docs/SECURITY-PROOFS.md` §4.1/§4.2 +
  `docs/THREAT-MODEL.md` §2 rung 4 (the real-keying cross-records).

---

### Row D-65 — `keyring-core` v1.0.0 OS-keychain backend + Tauri IPC bridge wiring → Phase-4-Meta-Composing (R9-council F-14)

- **Frozen surface (v1-beta):** the DAK-wrap secret-store SEAM is BUILT +
  frozen. `benten_engine::layer_d::secret_store::SecretStore` is the
  `Box<dyn SecretStore>` trait (`store` / `retrieve` / `backend_name`), the
  typed `SecretStoreError` (`NotFound` / `KeychainUnavailable`,
  `#[non_exhaustive]`), the `KeyringCoreStore` keychain backend model, the
  file-vault fallback, and the `open_dak_wrap_store(keychain_available)`
  selector whose fallback decision is EXPLICIT (typed
  `KeychainUnavailable`, never a silent data-loss). The seam SHAPE +
  fallback are the load-bearing v1-beta-core properties and are unit-pinned
  by `crates/benten-engine/tests/f_ld_7_keyring_core_file_vault_fallback.rs`.
- **Deferred consumption (Phase-4-Meta-Composing destination):** wiring the
  ACTUAL `keyring-core` v1.0.0 crate (NOT the legacy `keyring` crate — per
  CLAUDE.md 3-tactical-picks) behind the seam, plus the **Tauri IPC bridge**
  that drives this store as a `Box<dyn SecretStore>` from the embedded-webview
  shell. The `KeyringCoreStore` at HEAD models the OS-keychain backend
  behaviorally (its `available` flag models a host with/without a keychain);
  the concrete OS-keychain binding + the Tauri platform-glue are the
  Composing-phase platform tasks (they co-design with the Device-link UX +
  Remote-permission UX flows). Flagged in the module header at
  `crates/benten-engine/src/layer_d/secret_store.rs` (the "`keyring-core`
  binding (Composing-phase concern)" doc-block).
- **v1-beta posture:** the seam + the explicit-fallback decision are frozen +
  exercised; the residual is the concrete OS-keychain backend crate binding +
  the Tauri IPC transport. This is **NOT a Compromise** — a correctly-built
  seam whose concrete backend is deferred (the file-vault fallback is a real,
  safe backend at v1-beta on any host without an OS keychain).
- **Anchor:** R9-council F-14; CLAUDE.md 3-tactical-picks (`keyring-core`
  v1.0.0); `crates/benten-engine/src/layer_d/secret_store.rs::SecretStore` +
  the Device-link / Remote-permission Phase-4-Meta-Composing UX waves.

---

## R10 (post-F-full phase-close, round 10) NAMED-CARRY rows

> The rows below land at the R10 phase-close convergence council fix wave (branch
> `phase-4-meta-core/r10-council-fix`). Each is a HARD-RULE clause-(b) deferral
> whose ENTRY lands NOW with a NAMED destination; the substantive change ships in
> the named downstream wave. Cites verified live at HEAD `bbdf4b91` at author-time.

### Row D-66 — Drop `per_node_attestation` typed per-Node-signature upgrade → Phase-4-Meta-Composing (R10 F-07)

- **Frozen surface (v1-beta):** the `benten_drop::bundle::DropBundle`
  `per_node_attestation` field (`crates/benten-drop/src/bundle.rs`) is a FROZEN
  `#[serde(with = "serde_bytes")] Vec<u8>` sized-placeholder blob (130-byte
  reserved marker). The `DropBundleError::PerNodeSignatureInvalid` typed-reject
  variant is defined here but is **reserved-but-unconstructed** at v1-beta
  (register-then-enforce, like Row D-64 / D-52) — nothing emits it because the
  field carries no real signatures. The v1-beta defense-in-depth on the shipped
  Drop path is genuinely 2 layers (envelope-sig + per-Node-AEAD-tag); the field's
  size-reservation is pinned by
  `tf3f_per_node_attestation_size_overhead_under_12_percent`.
- **Deferred consumption (Phase-4-Meta-Composing destination):** upgrade the
  `per_node_attestation` field from the inert `Vec<u8>` sized placeholder to a
  real typed `Vec<Signature>` parallel to `content` (the third integrity layer =
  per-Node-signature validation), which lights up the `PerNodeSignatureInvalid`
  typed-reject emitter. This is an ADDITIVE upgrade landing within the reserved
  size budget (no wire-format surprise per the ~12% Spike G ceiling pin).
- **v1-beta posture:** an inert reserved field with zero runtime impact — the
  freeze deliberately FROZE the size-reservation shape (NOT the `Vec<Signature>`
  shape), so this is a documented reserved seam, **NOT a Compromise**. The R10
  F-07 retense corrected the field/inline docstrings from future-tense ("the
  freeze upgrades this to `Vec<Signature>`") to completed-freeze framing ("FROZEN
  as an inert reserved-size blob; the typed upgrade was DEFERRED").
- **Anchor:** R10-council F-07; `crates/benten-drop/src/bundle.rs::DropBundle::per_node_attestation`
  + `DropBundleError::PerNodeSignatureInvalid`; the G-CORE-3f Spike-G size-budget pin.

### Row D-1/D-64-adjacent note — `accept_grant` caller-contract (R10 F-15)

- **Caller-contract disclosure (v1-beta):** `benten_engine::layer_d::grant_acceptance::accept_grant`
  runs the six-class grant-acceptance pipeline over a `GrantAcceptanceContext`,
  but does **NOT** verify the `PermissionGrant.signature` and does **NOT** bind
  the `request_id`. The caller is responsible for (a) verifying the grant's
  signature at the wire layer (over `signing_bytes`) and (b) binding the
  request_id BEFORE deriving the acceptance context. This is a **caller
  contract**, not a gap in the pipeline — the pipeline's job is the six-class
  order-sensitive check-cascade (replay / revocation / clock / audience /
  UI-summary / audit-binding), with signature-verify + request_id-binding as
  wire-layer preconditions. Recorded adjacent to Row D-1 (WriteBoundaryChain
  consumption) + Row D-64 (engine encrypt-to-recipient wiring), whose
  Phase-4-Meta-Composing engine-wiring is where the accept-grant path gains its
  first production caller. **Anchor:** R10-council F-15;
  `crates/benten-engine/src/layer_d/grant_acceptance.rs::accept_grant`.

---

## R11 (post-F-full phase-close, round 11) NAMED-CARRY rows

> The rows below land at the R11 phase-close convergence council fix wave (branch
> `phase-4-meta-core/r11-council-fix`). Each is a HARD-RULE clause-(b) deferral
> whose ENTRY lands NOW with a NAMED destination; the substantive change ships in
> the named downstream wave. Cites verified live at HEAD `a3a1ef03` at author-time.

### Row D-67 — MC-11: `DropBundle::parse_cbor_bytes` uncapped decode → v1-GM decode-cap hardening (bundled with remote-permission wiring)

- **Uncapped-decode disclosure (v1-beta):** `benten_drop::bundle::DropBundle::parse_cbor_bytes`
  (`crates/benten-drop/src/bundle.rs:354`) calls `serde_ipld_dagcbor::from_slice(bytes)`
  with **NO byte-length ceiling** before decoding — an unbounded-decode surface.
  There is **no live production caller** at v1-beta (only the `tf3f_*` offline-consume
  test pins drive it — `crates/benten-drop/tests/tf3f_drop_bundle_offline_consume.rs`
  + `tf3f_no_mode3_inline_tiny_arm.rs`); the Drop offline-consume path gains its
  first production caller when the remote-permission / Drop-consume engine wiring
  lands. Consequently there is no live DoS exploit at v1-beta (no attacker-reachable
  entry point), but the decode-cap MUST be added before the surface goes live.
- **Deferred hardening (v1-GM destination):** add a `DROP_BUNDLE_MAX_SIZE_BYTES`-scoped
  length ceiling (the const already exists — `benten_drop::DROP_BUNDLE_MAX_SIZE_BYTES`)
  to `parse_cbor_bytes` (reject over-cap bytes BEFORE `from_slice`), co-scheduled
  with the remote-permission-call wiring (Row D-64 engine encrypt-to-recipient +
  Row D-65 keyring/Tauri IPC) that first exposes the Drop-consume path to untrusted
  input. Bundle with the v1-GM decode-cap hardening sweep.
- **Anchor:** R11-council MC-11; `crates/benten-drop/src/bundle.rs::DropBundle::parse_cbor_bytes`
  (:354); co-routes with Row D-64 / D-65 (remote-permission wiring).

### Row D-68 — MC-16: `f_hlc_2` no-mutation claim precision (doc-tense) → tightened this round; residual precision NAMED

- **Observation (NAMED):** the `f_hlc_2` HLC test's no-mutation claim wording was
  imprecise about exactly what invariant the test pins (the test pins that the HLC
  read/observe path performs no in-place mutation of the observed clock state, NOT
  a broader "HLC is never mutated" claim). The doc/comment wording is tightened to
  exactly what the test asserts in this same round (MC-16 doc-precision). Recorded
  here per HARD-RULE clause-(b) for forensic continuity; no downstream substantive
  change — this is a doc-tense precision fix, not a code carry.
- **Anchor:** R11-council MC-16; the `f_hlc_2` HLC no-mutation pin.

### Row D-69 — MC-6 vault salt origination → production vault-creation wiring (deferred with device-auth)

- **Caller-contract disclosure (v1-beta):** the R11 MC-6 fix makes the vault
  on-disk frame self-contained (the 16-byte Argon2id salt + params are persisted
  in the header, so `vault.cbor` bytes + password alone re-derive the DAK across
  a restart — `benten_crypto_suite::vault::serialize_vault` / `open_vault`).
  `serialize_vault` threads a **caller-supplied** `salt`; it does NOT originate
  it. At v1-beta there is **no production vault-*creation* call site** — the only
  callers are tests passing fixed-constant salts (`HeadlessDeviceAuth::seal_and_build`
  is exercised only by tests). No live weak-salt exposure exists (no production
  vault is created from a low-entropy salt because no production vault is created
  at all yet).
- **Caller contract (MUST hold when creation lands):** the production
  vault-creation wiring MUST seed the 16-byte `salt` from an OS CSPRNG
  (`OsRng` / `getrandom`), **unique per vault**. The salt+params are self-authenticating
  through the DAK derivation (tampering → wrong DAK → fail-closed `AeadFailed`),
  so they are intentionally not AEAD-AAD-covered (standard PBKDF-header posture).
- **Deferred (destination):** vault-creation-from-OS-entropy is deferred with the
  device-auth / keyring surface (co-routes with Row D-64 engine encrypt-to-recipient
  + Row D-65 keyring/Tauri IPC). When it lands, the salt-origination CSPRNG contract
  above becomes an enforced construction site.
- **Anchor:** R11-council MC-6 adversarial-review observation;
  `crates/benten-crypto-suite/src/vault.rs::serialize_vault` (caller-supplied salt);
  `crates/benten-engine/src/layer_d/device_auth.rs::HeadlessDeviceAuth::seal_and_build`.

---

## R12 (post-F-full phase-close, round 12) NAMED-CARRY rows

### Row D-70 — F-08: `WrappedKey` has 3 hand-rolled encode sites → canonical `to_wire_bytes`/`from_wire_bytes` consolidation

- **Observation (NAMED):** `benten_crypto_suite::cipher_suite::WrappedKey` (the
  Layer-C/Layer-D key-wrap wire type) carries **no canonical
  `to_wire_bytes()`/`from_wire_bytes()` method on the type itself**, so its
  length-prefixed encode form (`ek_x_len u32 BE ‖ ek_x ‖ ek_mlkem_len u32 BE ‖
  ek_mlkem ‖ aead_envelope.to_wire_bytes()`, BE per M-19) is **hand-rolled in 3
  sites**: the production `benten_drop::layer_c::encode_wrapped_key`
  (`crates/benten-drop/src/layer_c.rs:656`, called at `:778` single-recipient
  and `:1315` group), plus a duplicated in-test copy
  `layer_c::group_posture::tests::encode_wrapped` (`:1953`). Each site re-derives
  the same framing by hand; a future field-order / width / endianness edit must
  be mirrored across all copies (drift surface — currently guarded only by
  round-trip tests, not by a single canonical encoder).
- **Deferred (destination):** lift the encode/decode onto `WrappedKey` as
  canonical `to_wire_bytes(&self) -> Vec<u8>` / `from_wire_bytes(&[u8]) ->
  Option<Self>` methods in `benten-crypto-suite` (the type's home crate) and
  replace the 3 hand-rolled sites with calls to it — a **no-wire-change
  consolidation**. Deferred to the Phase-4-Meta-Composing crypto-surface
  cleanup (co-routes with Row D-71's layer_c/layer_d envelope-helper
  consolidation — same class). Not a v1-beta blocker: the wire form is frozen +
  round-trip-pinned + byte-mirrored across the seal/open paths.
- **Anchor:** R12-council F-08; `crates/benten-drop/src/layer_c.rs::encode_wrapped_key`
  (+ the `:1953` test copy); `benten_crypto_suite::cipher_suite::WrappedKey`.

### Row D-71 — F-09: Layer-C / Layer-D shared envelope-assembly helper (device_link FLAG retense)

- **Observation (NAMED, retensed):** the `device_link.rs` module docstring FLAG
  previously claimed `benten_drop::layer_c` was "a SIBLING wave not yet merged
  into this base". That premise is **stale** — `benten-drop` (the 14th crate)
  HAS landed. The FLAG is retensed in-code (`crates/benten-engine/src/layer_d/device_link.rs`
  §"HPKE reuse") to record the AS-BUILT state: the Layer-D wrap calls the
  crypto-suite HPKE primitive directly, while Layer-C assembles its envelopes
  directly against `aead::wrap` (V1-FROZEN-INTERFACE.md §6 item 4 F-07 AS-BUILT note).
- **Deferred (destination):** the Layer-C drop assembler and the Layer-D wrap
  should share ONE envelope-assembly helper rather than each routing to the
  crypto-suite primitive independently — a **no-wire-change consolidation**
  deferred to Phase-4-Meta-Composing (co-routes with Row D-70). Not a v1-beta
  blocker (both paths produce the frozen, byte-mirror-pinned wire form).
- **Anchor:** R12-council F-09; `crates/benten-engine/src/layer_d/device_link.rs`
  ("HPKE reuse" FLAG); `benten_drop::layer_c`.

---

## Update discipline

This document updates via PR:
- When G-COMP-1 wave closes any deferred row → strike-through here
  + add a `CLOSED-at-#<PR>` annotation; do NOT delete the row
  (forensic context retained per pim-13 / §3.12).
- When G-CORE-9 R2-Rn rounds surface additional deferred items →
  append rows; maintain destination naming discipline.
- When Ben rebuts a Fork (Fork 1 / Fork 2 / Fork 3) → mark the
  affected rows as REOPENED-IN-SCOPE + cite the rebuttal PR.
