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

### Row D-8 — F3 anti-replay atomic compare-and-swap (FrameReplayMarker TOCTOU)

- **Frozen surface (v1-beta):**
  `crates/benten-caps/src/chain_authority.rs:404-421` —
  `FrameReplayMarker::mark_and_check_frame` get + put non-atomic
  pair (separate redb transactions). Compromise #23 IS retensed
  to acknowledge in-window racy.
- **Deferred consumption (G-COMP-1 destination):** either (a) extend
  KVBackend trait with a typed `compare_and_insert(key, value)
  -> Result<bool, _>` method AND change `mark_and_check_frame` to
  use it, OR (b) route the marker call through
  `GraphBackend::transaction(|tx| ...)` so both the get + put run
  inside one txn (the existing transaction API supports this), OR
  (c) document a serializing per-engine lock around the
  `apply_atrium_merge` marker call. Option (b) is lowest-cost
  (~10 LOC change inside `mark_and_check_frame`).
- **v1-beta posture:** F3 anti-replay defense is racy under
  concurrent inbound apply_atrium_merge presentations of the same
  session_nonce. Compromise #23 retensed to disclose.
- **Anchor:** Compromise #23.

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

### Row D-13 — structural_kdf info-tag codepoint-binding

- **Frozen surface (v1-beta):** `aead_wrap::make_key_material_matching`
  threads attacker-controlled envelope codepoint into key newtype
  (docstring at `aead_wrap.rs:500-506` acknowledges; the natural
  ChaCha20-Poly1305 defense via K_root divergence IS structurally
  present at v1-beta).
- **Deferred consumption (G-COMP-1 destination):** extend
  `structural_kdf::derive_root` info-tag to include cipher-suite
  codepoint (e.g. `info = "root:codepoint:<le_bytes>" || root_cid`);
  ~5 LOC + golden test for backward-compat (since this CHANGES
  K_root derivation, it is a wire-format-coupled change requiring
  a backward-compat scheme or version bump per the freeze contract;
  G-COMP-1 must decide the migration shape).
- **v1-beta posture:** natural ChaCha20-Poly1305 defense via K_root
  divergence between codepoint arms IS structurally present at
  v1-beta (verified L2-MAJ-4 disposition). Codepoint-binding via
  KDF info is incidental not explicit.
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
  becomes 4-segment AAD: domain-tag || plaintext_cid ||
  chunk_index LE || total_chunks LE), closing the
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

#### Row D-15a — AAD per-chunk `total_chunks` augmentation

- **Frozen surface (v1-beta):** the as-shipped per-chunk AAD layout
  is the 2-tuple `aad_per_chunk(plaintext_cid, chunk_index)` per
  Fork 1 ratification at G-CORE-9 R1 fix-pass Bundle 11b
  (V1-WIRE-FORMAT-INVENTORY.md row 4 + V1-BETA-BREAKING-CHANGES.md
  Bundle 11b).
- **Deferred consumption (G-COMP-1 destination):** augment the AAD
  layout to a 3-tuple
  `aad_per_chunk(plaintext_cid, chunk_index, total_chunks)` to
  close the cross-chunk-truncation attack surface (chunk_index
  alone does not bind the chunk count; an attacker truncating the
  ciphertext stream after N chunks gives a valid-looking decryption
  for chunks 0..N-1 with no detection that chunks N..total_chunks-1
  are missing).
- **v1-beta posture:** the canonical-bytes pin at
  `crates/benten-graph/tests/canonical_bytes_v1_per_chunk_aead_aad.rs`
  locks the 2-tuple shape; cross-chunk-truncation is a documented
  Compromise #5 sub-case (the per-Node-CID rebinding-attack defense
  fires on tamper-AFTER-decrypt; truncation-BEFORE-decrypt is the
  uncovered arm).
- **Anchor:** Compromise #5 + V1-WIRE-FORMAT-INVENTORY.md row 4 +
  V1-BETA-BREAKING-CHANGES.md Bundle 11b Fork 1 rebuttal-window
  narrative.

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

### Row D-19 — G-CORE-9 R1 Bundle 4 ESCALATED items (Strategy::C → Reserved rename + 3 DSL ErrorCode mints)

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

## Update discipline

This document updates via PR:
- When G-COMP-1 wave closes any deferred row → strike-through here
  + add a `CLOSED-at-#<PR>` annotation; do NOT delete the row
  (forensic context retained per pim-13 / §3.12).
- When G-CORE-9 R2-Rn rounds surface additional deferred items →
  append rows; maintain destination naming discipline.
- When Ben rebuts a Fork (Fork 1 / Fork 2 / Fork 3) → mark the
  affected rows as REOPENED-IN-SCOPE + cite the rebuttal PR.
