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

### Row D-17 — `CapWriteContext` + `ReadContext` + `SuspensionOutcome` `#[non_exhaustive]` application (with ~80+ test-site cascade)

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
- **Deferred consumption (G-COMP-1 destination):** apply
  `#[non_exhaustive]` to both types + cascade through ~50+ benten-caps
  test-site direct-struct-literal constructions, migrating each to
  `Default::default()` + field-mutation pattern. Production code (in
  `benten-engine`) ALREADY uses the field-mutation pattern per
  Bundle 3 of this PR — so the migration is benten-caps tests only.
- **v1-beta posture:** at v1-beta the type shape is locked per the
  freeze contract narrative; the attribute is the documentation gap.
  Field additions are TREATED AS breaking by v1-beta engineering
  discipline per spec item 11 narrative (the structural enforcement
  via `#[non_exhaustive]` is what G-COMP-1 lights).
- **Anchor:** V1-FROZEN-INTERFACE.md item 11 table rows for
  `CapWriteContext` + `ReadContext`; L6-r1-1 G-CORE-9 R1 escalation.

### Row D-16 — V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md authorship

- **Frozen surface (v1-beta):** V1-FROZEN-INTERFACE.md item 4
  references `docs/V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md` as the
  Ben-signed P-III decision-point artifact.
- **Deferred consumption (G-COMP-1 destination OR pre-`v1-beta`
  tag):** author this doc OR (alternative ratification) rename
  references to point at `docs/V1-WIRE-FORMAT-INVENTORY.md` (which
  already serves this role; the inventory IS the P-III decision-point
  artifact). The latter is the orchestrator's preferred path
  (consistency with existing tracked-doc); requires a sweep of
  cite-drift across V1-FROZEN-INTERFACE.md.
- **v1-beta posture:** the inventory IS authored + tracked.
- **Anchor:** L17-r1-7.

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
