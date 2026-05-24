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

### Row D-1 — WriteBoundaryChainValidator consumption (Engine::commit / Engine::put_node_with_context)

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

### Row D-2 — InstallRecordReplayStore lifecycle-required wiring

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

### Row D-3 — 3 §8-E CapabilityPolicy hooks consumption

- **Frozen surface (v1-beta):** `crates/benten-caps/src/policy.rs:492-557` —
  `check_install_consent` + `check_per_delegation` +
  `check_write_with_audience` — defaulted trait methods with
  signatures locked + object-safety preserved + workspace-test
  `__sealed_for_workspace_tests` re-export covers test doubles.
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

### Row D-4 — ProductionManifestEnvelopeRechecker production impl + default-builder wiring

- **Frozen surface (v1-beta):**
  `crates/benten-engine/src/manifest_envelope_recheck.rs:144` —
  `pub trait ManifestEnvelopeRechecker` port interface + the
  four-arm `ManifestEnvelopeRecheckOutcome` enum + the structural
  empty-peer-DID fail-CLOSED at `engine.rs:1462-1476` (Layer-A
  defense fires structurally before rechecker dispatch).
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

### Row D-6 — §4.25 sync-hydrate consumption of UnresolvedDeny at handshake.rs

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

### Row D-7 — §8-A Engine visibility cluster tighten + napi cascade

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

- **Frozen surface (v1-beta):** various nice-to-have hardenings
  surfaced in R1 OBS items.
- **Deferred consumption (G-COMP-1 OR Phase-4-Meta-Composing
  v1-assessment-window):**
  - `AeadEnvelope::to_wire_bytes` panics via `.expect` on nonce
    length > 255 — convert to `Result` (L1-crypto-r1-5)
  - `AuthorizationGrant.binding_sig` hardcoded `[u8; 64]` (Ed25519)
    — promote to varsig-tagged variable-length for crypto-agility
    parity with the rest of #5 framing (L17-r1-6)
  - `CryptoPolicy::require_hybrid_pq` consumer-side policy flag for
    rejecting 0x6400 classical-only envelopes (L2-MAJ-3)
  - `AuthorizationGrant.audience_pubkey` Option→non-Option promotion
    OR `AuthorizationGrant::issue_production` mandatory-bytes
    constructor (L6-r1-9)
  - SHA2_512_256 (multihash `0x1015`) + SHA3_256 (multihash `0x16`)
    pre-blessed agile-hash-fallback codepoint mint per CLAUDE.md baked-in
    #5 (L11-R2-MINOR-4). At codepoint-mint-time MUST add to
    `codepoint_table_integer_values_pinned` with hex-pin per the
    discipline established at G-CORE-9 R1 fix-pass Bundle 5.
- **v1-beta posture:** all of the above are nice-to-have; each has
  no immediate exploit at v1-beta (the audience CID IS bound via
  binding_sig; ed25519_dalek is the only signature primitive used
  for binding_sig at v1-beta so the hardcoded shape is consistent;
  classical-only construction IS cryptographically sound).
- **Anchor:** Various R1 OBS items.

### Row D-18 — L2-MAJ-1 empty-peer-DID synthesized-fallback structural hardening

- **Frozen surface (v1-beta):** the structural empty-peer-DID
  fail-CLOSED at `engine.rs:1462-1476` IS live for the literal-empty
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
  - `benten-engine`: `UserViewInputPattern` (outcome.rs:39), `TraceStep`
    (outcome.rs:362), `StreamCursor` (engine_stream.rs:150),
    `SubscribeCursor` (engine_subscribe.rs:72), `EngineViewsHandle`
    (engine_views.rs:1129), `AtriumConfig` (atrium_api.rs:64), `SyncStatus`
    (atrium_api.rs:128), plus the outcome.rs 13-pub-struct set
    (`UserViewSpec`, `UserViewSpecBuilder`, `ReadViewOptions`, `Outcome`,
    `Trace`, `TerminalError`, `BudgetExhaustedView`, `AnchorHandle`,
    `RegisterReplaceOutcome`, `HandlerPredecessors`, `DiagnosticInfo`,
    `NestedTx`)
  - `benten-ivm`: `SubgraphSpec` (subgraph_spec.rs:107), `KernelInput`
    (subgraph_spec.rs:262), `ViewState` (view.rs:157), `ViewBudget`
    (view.rs:180), `ViewQuery` (view.rs:223), `ViewResult` (view.rs:242),
    `ViewDefinition` (view.rs:387)
  - `benten-platform-foundation`: `VocabLabel` (vocab.rs:14), `VocabEdge`
    (vocab.rs:92), `Scalar` (vocab.rs:157), `RenderError` (materializer.rs:567)
  - `benten-core`: `Mode` (version_dag.rs:75)

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

- **Frozen surface (v1-beta):** the obsolete `Strategy::C` arm name, the wire string `E_VIEW_STRATEGY_C_RESERVED`, the variant `ViewStrategyCReserved`, the TS class `EViewStrategyCReserved`, and the absence of explicit `E_DSL_PARSE_FAILED` / `E_DSL_UNKNOWN_PRIMITIVE` / `E_DSL_MISSING_RESPOND` ErrorCodes all freeze at v1-beta. The cargo-public-api baselines at `docs/public-api/benten-errors.txt:188` + `docs/public-api/benten-engine.txt:976,977,2329,2330` lock the obsolete `ViewStrategyCReserved` name; per Bundle 10 Fork 3 the cargo-public-api workflow is required-failing so the rename WINDOW is the G-CORE-9 freeze wave OR a deliberate post-v1-beta SemVer break.
- **Deferred consumption (G-COMP-1 destination):** atomic 4-surface rename per §3.5g:
  1. Rust enum `EngineError::ViewStrategyCReserved` → `EngineError::ViewStrategyReserved` (`crates/benten-engine/src/error.rs` + format-string at `engine_views.rs:695-699` already returns `Strategy::Reserved`)
  2. Wire string `E_VIEW_STRATEGY_C_RESERVED` → `E_VIEW_STRATEGY_RESERVED` (`crates/benten-errors/src/lib.rs` 4 sites: variant + wire string + Display arm + parse arm)
  3. TS class `EViewStrategyCReserved` → `EViewStrategyReserved` (`packages/engine/src/errors.generated.ts` 3 sites; docstring already says "Strategy::Reserved" — cross-language drift on SAME code path per §3.5g item 1)
  4. ERROR-CATALOG.md:533+727 + cargo-public-api baselines `docs/public-api/benten-errors.txt:188` + `docs/public-api/benten-engine.txt:976,977,2329,2330` (5 baseline cites) + `crates/benten-errors/tests/stable_shape.rs:112+682+1149` regenerate

  AND mint 3 new DSL ErrorCodes per L9-DSL-MAJOR-1 closure:
  5. `E_DSL_PARSE_FAILED` — mints from existing `CompileError::Parse`
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
  trybuild test at V1-FROZEN-INTERFACE.md:717 that does not exist as a
  separate file; this row plugs the named-destination phantom per HARD
  RULE 12 clause-(b).
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

- **Frozen surface (v1-beta):** the `benten-crypto-suite` crate is
  item-6-locked at V1-FROZEN-INTERFACE.md (codepoint table + public
  surface frozen at G-CORE-9). The INTERNALS.md doc has no v1-beta
  signature impact; it is internal architecture-record only.
- **Deferred consumption (Phase-4-Meta-Composing OR G-COMP-1
  destination):** author `crates/benten-crypto-suite/INTERNALS.md`
  following the structure of `crates/benten-caps/INTERNALS.md` covering
  codepoint table + typed-reject dispatch pattern + SwapMatrix umbrella
  + 5 named constructors + C11b safety gate + X-Wing vendored combiner
  provenance + AeadEnvelope/GrantKeyMaterial/AeadKeyMaterial
  type-collision-resolution name discipline.
- **v1-beta posture:** missing-but-deferred-not-blocking-tag; the
  crate's rustdoc + the V1-FROZEN-INTERFACE.md item 6 + the lib.rs
  module docstring carry the load-bearing architecture narrative at
  v1-beta. INTERNALS.md is the post-v1-beta architecture-record
  augmentation.
- **Anchor:** spec item 6 + V1-FROZEN-INTERFACE.md item 15.d + the
  rename pair at #1344 row 7 (GrantKeyMaterial / AeadKeyMaterial) +
  L18-r1-5 + L18-r2-3.

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

## Update discipline

This document updates via PR:
- When G-COMP-1 wave closes any deferred row → strike-through here
  + add a `CLOSED-at-#<PR>` annotation; do NOT delete the row
  (forensic context retained per pim-13 / §3.12).
- When G-CORE-9 R2-Rn rounds surface additional deferred items →
  append rows; maintain destination naming discipline.
- When Ben rebuts a Fork (Fork 1 / Fork 2 / Fork 3) → mark the
  affected rows as REOPENED-IN-SCOPE + cite the rebuttal PR.
