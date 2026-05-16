# benten-platform-foundation — Internals

A plain-English, code-grounded tour of the `benten-platform-foundation` crate — the 11th workspace crate per Ben D-4F-2 ratification, and the substantive home of the **v1 platform-shippable surface** per CLAUDE.md baked-in #15. Read-only audit. Audience: a developer landing in this crate fresh and trying to find the load-bearing seams. Snapshot: post `phase-4-foundation-close` tag (main HEAD `8141b94`; PR #242–#250 all merged).

---

## 1. What this crate does

`benten-platform-foundation` is where Phase 4-Foundation's **platform layer** lives. It is intentionally broad per Ben's R1 ratification arch-r1-8 ("the v1 platform-shippable surface"). Where `benten-engine` is the orchestrator that walks subgraphs, this crate is the orchestrator's **application substrate** — the schema compiler that lets users describe their content types, the materializer pipeline that renders them, the plugin manifest schema that lets people share workflows + admin extensions, and the admin UI v0 itself (the first app-level plugin per CLAUDE.md baked-in #18).

Four substantive surfaces live here, each one a Phase-4-Foundation wave:

1. **`schema_compiler`** (G23-A) — schemas-as-subgraphs-of-primitive-typed-field-Nodes parser using the D-4F-NEW-TYPED-FIELD-NODE-VOCAB ratification: 8 labels (`SchemaRoot` / `FieldScalar` / `FieldObject` / `FieldList` / `FieldMap` / `FieldRef` / `FieldEnum` / `FieldUnion`), 5 labeled edges (`ITEM_TYPE` / `KEY_TYPE` / `VALUE_TYPE` / `REF_TARGET` / `VARIANT`; object-to-field is implicit-via-recursion — no `FIELD` edge label was minted at R6-FP-3), 8 scalars (text / int / float / bool / bytes / bytes-cid / timestamp-hlc / null per `benten_core::Value`), 4 mandatory field properties (`name` / `required` / `default` / `scope` — `scope` is **schema-derived per sec-3.5-r1-4**, NOT user-supplied).
2. **`materializer`** (G23-B) — the `Materializer` trait + `HtmlJsonMaterializer` default impl + `PlaintextMaterializer` 2nd impl (arch-r1-10 output-format-pluggability validation per cag-r1-6) + the `Renderer` transport-abstraction trait + `BrowserRender` default impl. TauriRenderer lives in the sibling crate `benten-renderer-tauri` per G24-E + CLAUDE.md #19 (engine-extension category).
3. **`plugin_manifest` + `plugin_lifecycle` + `plugin_library` + `manifest_store` + `module_ecosystem`** (G24-D + G24-D-FP-1 + G24-D-FP-2) — the FULL plugin manifest schema per CLAUDE.md baked-in #18 four-identity-concepts model, plus install/uninstall/upgrade lifecycle, plus the durable plugin-library subgraph, plus the verify-on-every-load manifest store, plus the install-flow orchestration.
4. **`admin_ui_v0`** (G24-A + G24-B + G24-C) — admin UI v0 shell with 4-category navigation (Plugins / Workflows / Content Types / Views per ratification #4) + the workflow editor (G24-B) + the composed-view-creator handler-side surface. First app-level plugin per CLAUDE.md baked-in #18.

Two thinner surfaces round it out: **`workflow_to_plugin`** (D-4F-14 workflow ↔ plugin unification — promote a workflow to a plugin by attaching a manifest) and **`registry`** (Phase-4-Meta-reserved decentralized-registry trait shapes; zero production call sites at Phase-4-Foundation per ratification #3).

The crate is **not** the engine — it sits *atop* the engine via trait ports (`MaterializerEngine`, `CapRevoker`, `PrivateNamespaceTeardown`, `SubscriptionRegistry`) so the production dep direction stays one-way (foundation → core / errors / id; foundation does NOT depend on graph / eval / engine in production builds).

---

## 2. Dependency chain

**Workspace deps (in):** `benten-core` (Subgraph + SubgraphSpec + PrimitiveKind + Value + OperationNode + Cid + Anchor + DagVersionChain + canonical_subgraph_bytes), `benten-errors` (ErrorCode catalog), `benten-id` (Did + RotationLog + PluginDidStore + plugin_did::mint per CLAUDE.md #18 four-identity-concepts).

**External deps:** `blake3` (content-CID computation for plugin manifests), `serde` (`derive`), `serde_ipld_dagcbor` (canonical-bytes manifest encoding), `serde_json` (G23-A JSON-Schema ingest dialect parser — workspace-canonical JSON parser only), `ed25519-dalek` (peer-DID signature verification), `thiserror`.

**Dev deps:** `benten-engine` (with `test-helpers`), `benten-caps` (G23-A `RecordingCapPolicy` + G24-D substantive plugin-delegation surface), `walkdir` 2.5 (R4-FP-4 substantive grep-walk pin), `proptest` (G23-B determinism property test).

**Consumers (out):** today, only the integration tests in this crate's own `tests/` tree. At Phase-4-Foundation close, the `bindings/napi` cdylib exposes the admin UI surface; admin UI v0 napi shell, future Phase-4-Foundation handlers, and the eventual Phase 4-Meta self-composing admin UI all consume this crate.

**Crates explicitly NOT reached (arch-r1-1 + arch-r1-15):** `benten-eval`, `benten-graph`, `benten-engine` (production). The production dep direction is pinned by the arch-test `tests/arch_n_benten_platform_foundation_dep_direction.rs`. The crate uses trait ports + the `MaterializerEngine` adapter (test-side at `tests/common/admin_ui_v0_engine_adapter.rs`) to plug `benten_engine::Engine` in at the consumer boundary without dragging the engine into this crate's `lib.rs` dep graph.

**Features:** none. The crate is single-target; the SANDBOX surface is engine-side, sync state is engine-side, and the renderer-transport split lives in the sibling `benten-renderer-tauri` crate.

---

## 3. Files inventory in `src/` (~5.0k LOC across 12 files + 2 submodule trees)

### 3a. Crate root + plugin manifest core

- **`lib.rs`** (127 LOC) — module declarations + public re-exports + the four-surface-orientation crate-level doc. `#![allow(dead_code, clippy::needless_pass_by_value, missing_docs)]` — `missing_docs` allowance is intentional during the Phase-4-Foundation pre-tag window; tightens at phase close. Re-exports for the four substantive surfaces flow through here.

- **`plugin_manifest.rs`** (890 LOC) — the **FULL plugin manifest schema** per CLAUDE.md baked-in #18. Owns `PluginManifest` (the manifest body with `plugin_name` / `content_cid` / `peer_did` / `peer_signature` / `requires` / `shares` / optional `renderer_config` / optional `composes_plugins` / optional `accepts_content` / optional `requires_schema_authors` / optional `requires_plugin_authors`), `CapRequirement`, `SharesPolicy` + `SharesPolicyDefault` (`None` / `Any` / `Matching` with the R6-FP-A sec-r6r1-7 hardening that requires `Any` be paired with an explicit `rules` vector), `SharesRule`, `SharesTarget` (`Any` / `PluginDid(Did)` — the prior dead `PluginAuthor` variant was removed at R6-FP-A sec-r6r1-8), `RendererConfig` + `RendererBackend`, `InstallRecord` (with `consenting_user_did` omit-by-design per r2-cp-5: signer recovered via pubkey-verify, not literal-bytes — cross-DID substitution requires forging the user signature), `ValidationOutcome` + `RotatedKeyWarning` for the RotationLog-aware validation surface, `ContentAddressed` trait shim, `MANIFEST_CLOCK_NOT_INJECTED_SENTINEL = 0` (mirrors `benten-caps::ucan_grounded::DEFAULT_NOW_SECS = 0` for a single mental model of "engine built without clock injection"), `detect_composition_cycle` (one `PrimitiveKind::Read` Node per visited manifest CID + `COMPOSES`-labelled edges per reference; DFS visited-on-stack — no new `PrimitiveKind` variant minted per CLAUDE.md #1), `sign_manifest`. Two distinct serializations live on the type and the rustdoc names both: `signing_payload()` (zeroes `content_cid` + `peer_signature` — chicken-and-egg) and `to_canonical_bytes()` (preserves both — round-trippable manifest bytes). The validate seam splits two ways: `validate_with_rotation_log` (consults `benten_id::did_rotation::RotationLog`; rotation surfaces as `ValidationOutcome::ValidWithWarning` per D-4F-12 — NOT hard-reject) and `validate_with_clock` (Phase-4-Foundation R4b-FP-1 Seam 2; threads clock injection at the load boundary per D-4F-15 transparent-clock-injection ratification; fires `E_UCAN_CLOCK_NOT_INJECTED` on sentinel + time-bounded requirement).

- **`plugin_lifecycle.rs`** (1098 LOC) — install/uninstall lifecycle implementation. Owns `install_plugin` (the substantive G24-D wave-7 install path that runs Layer-1 + Layer-2 + Layer-3 consent gates per CLAUDE.md #18; supersedes the deprecated `module_ecosystem::install_plugin` precursor), `uninstall_plugin` (the FIVE-step cascade per CLAUDE.md baked-in #18 + threat-model §T10/T12: held-caps revoke → downstream-delegations cascade-revoke → live-subscription termination → private-namespace teardown → library-entry removal + plugin-DID revoke), `UninstallOutcome` (per-step observable counters), three trait **ports** the engine adapter implements (`CapRevoker` with `revoke_grants_with_audience` + `cascade_revoke_grants_with_issuer`; `PrivateNamespaceTeardown::delete_private_namespace_for`; `SubscriptionRegistry::terminate_subscriptions_for` + `active_subscription_count`), `UninstallContext` (the three-port bundle), `discover_new_version` (PULL-not-PUSH new-version notification per plugin-arch-r1-13 — consults a `DagVersionChain` against the library), and `InMemoryUninstallCascade` (the substantive in-memory default consumed by every G24-D-FP-1 RED-PHASE pin; mirrors a Phase-3 UCAN grant shape via `InMemoryGrant` + `RevocationLogEntry` without depending on `benten-caps`). The cascade is ORDERED so in-flight reads cannot race past the cap-revoke — once step 1 fires, the Phase-3 G16-B-F per-row recheck surfaces `E_CAP_REVOKED` immediately on subsequent reads.

- **`plugin_library.rs`** (481 LOC) — `PluginLibrary` per CLAUDE.md #18 "library subgraph" framing. R6-FP-D lifted this from a `HashMap<Cid, LibraryEntry>` projection to a **real `benten_core::Subgraph`** anchored at `HANDLER_ID_PLUGIN_LIBRARY = "plugin-library"`. Three node kinds: a `library_root` Read node (the subgraph anchor); one `anchor::<plugin_name>` Read node per plugin-name (Phase-1 [`benten_core::version::Anchor`] companion); one `version::<cid>` Read node per installed CID. Three edge labels: `EDGE_LIBRARY_ANCHOR` (`ITEM_TYPE` from the schema-vocabulary — same-language `§3.5g` mirror via `VocabEdge::ItemType.as_str()` so future renames cascade), `EDGE_VERSION_OF = "VERSION_OF"`, `EDGE_CURRENT = "CURRENT"`. The shape preserves CLAUDE.md #18's anchor + Version Node DAG-shape (forks supported by allowing multiple appends from the same prior head — the `VersionError::Branched` arm is a non-error here, with the Version Node still landing in the subgraph for retention). Companion structures: `BTreeMap<Cid, LibraryEntry>` as the O(1) projection; `HashMap<String, Anchor>` as the Phase-1 chain index; `HashMap<String, Cid>` as the per-device-local CURRENT projection. Every mutator keeps **subgraph + anchor + entries + active in lockstep**.

- **`manifest_store.rs`** (249 LOC) — `ManifestStore` durable surface with **verify-on-EVERY-load** defense per threat-model §T5a. Stores raw DAG-CBOR install-record bytes (NOT decoded structs) so byte-mutation between install and re-load is detected; decode-then-re-encode would smooth over the attack. Three verify points per defense-step-1: engine boot, per-plugin load on first access, per-Atrium-merge boundary. Carries `DriftNotification` so user-notification sinks observe T5a drift events. Production redb backing is parallel to `PluginLibrary`'s durable half — at G24-D-FP-1 wave the in-memory shape is canonical; the redb integration is a Phase-4-Foundation backlog item per `docs/future/phase-4-backlog.md §4.11`.

- **`module_ecosystem.rs`** (310 LOC) — install/uninstall/upgrade/share/discover orchestrator on top of `plugin_manifest` + `plugin_library` + `benten_id::plugin_did`. Hosts `InstallResult` (the entry + minted `PluginDidHandle` bundle), `InstallerShape` (`FullPeer` / `ThinClient` heterogeneity classifier per CLAUDE.md #17 + ds-r1-8). **Important deprecation**: the legacy `install_plugin` + `install_plugin_persisting_did` functions here are **DEPRECATED canary precursors** that BYPASS Layer-2 + Layer-3 consent gates. They verify content-CID + peer-DID signature + heterogeneity + composition cycle ONLY. All production installs go through `plugin_lifecycle::install_plugin`. The legacy entries remain for three pre-R4b-FP-1 integration tests still consuming them for content-CID-mismatch / peer-signature-substitution / heterogeneity coverage.

- **`workflow_to_plugin.rs`** (93 LOC) — `WorkflowHandle` + `promote_workflow_to_plugin` per CLAUDE.md baked-in #18 D-4F-14 unification. A workflow IS a plugin IS a subgraph; the only shape-change is attaching a `PluginManifest`. The returned manifest has `peer_signature` empty (caller signs after computing `content_cid`); the workflow's `subgraph_cid` is seeded but then OVERWRITTEN by `compute_content_cid()` — the manifest body IS the plugin identity post-promotion.

- **`registry.rs`** (99 LOC) — **Phase-4-Meta-reserved** decentralized-registry trait shapes per ratification #3. `RegistryEntry`, `DiscoveryQuery` (3 variants: `ByAuthor` / `ByName` / `AcceptingContent`), `DiscoveryResult`, the `Registry` trait. `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode minted at G24-D wave but has no firing site until Phase 4-Meta. Zero production call sites at Phase-4-Foundation; pinned by `tests/registry_phase_4_meta_reserved_no_production_callsites.rs`.

### 3b. `schema_compiler/` (G23-A; 6 files, ~1.7k LOC)

- **`schema_compiler/mod.rs`** (243 LOC) — top-level `compile(bytes: &[u8]) -> Result<SchemaSubgraphSpec, SchemaCompileError>` entry point. The pipeline is: `parse_schema_json` → `validate_vocab` → `validate_no_sandbox_storage_mutation` → `validate_no_unconstrained_emit_respond` → `detect_field_ref_cycle` → `emit_subgraph` → **defensive `assert_canonical_primitive_kind` regression-guard** that walks every emitted Node and refuses anything outside the canonical 12 `PrimitiveKind` variants (the wildcard arm fires `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED` rather than letting a future 13th variant silently emit — CLAUDE.md #1 12-primitive-irreducibility tripwire). Three property-bag keys consumed by the materializer: `CAP_SCOPE_PROPERTY_KEY = "cap_scope"`, `FIELD_PATH_PROPERTY_KEY = "schema_field_path"`, `VOCAB_LABEL_PROPERTY_KEY = "schema_vocab_label"`. `SCHEMA_COMPILER_PROPERTY_KEYS` re-exports them as a grep-able const slice. `derive_scope(action, schema_name, field_path)` is the schema-derived cap-scope generator per sec-3.5-r1-4 — `scope` is NEVER taken from user input even if the schema JSON supplied one (load-bearing for the authorization-side-channel defense).

- **`schema_compiler/vocab.rs`** (240 LOC) — typed Rust mirrors of the 8 labels + 5 labeled edges + 8 scalars + 4 mandatory field properties. `VocabLabel` / `VocabEdge` / `Scalar` enums with `as_str()` / `from_str()` round-trips; `VOCAB_LABEL_NAMES` / `VOCAB_EDGE_NAMES` / `SCALAR_NAMES` / `VOCAB_REQUIRED_FIELD_PROPS` const slices for grep-asserts. Object-to-field is implicit-via-recursion in the emitter — no `FIELD` edge label minted, per `docs/SCHEMA-DRIVEN-RENDERING.md §2.2`.

- **`schema_compiler/parse.rs`** (694 LOC) — JSON-Schema dialect parser. `ParsedSchema` IR (kept `pub(crate)` — not part of the public surface; the API is just `compile → SchemaSubgraphSpec`). Validation passes live here: `validate_vocab` (label outside 8-set rejected), `validate_no_sandbox_storage_mutation` (per CLAUDE.md #16: SANDBOX refs requesting `kv:write` / `kv:delete` / `edges:add` / `edges:remove` rejected with `E_SCHEMA_SANDBOX_HOST_FN_REJECTED`), `validate_no_unconstrained_emit_respond` (closes the unconstrained-EMIT-target gap), `detect_field_ref_cycle` (rejects FieldRef cycles with `E_SCHEMA_VOCAB_CYCLE_REJECTED`).

- **`schema_compiler/emit.rs`** (538 LOC) — the subgraph emitter. Walks the `ParsedSchema` and emits READ / TRANSFORM / RESPOND / SUBSCRIBE primitives per field-access path, each carrying its derived cap-scope under `CAP_SCOPE_PROPERTY_KEY` + its field-path under `FIELD_PATH_PROPERTY_KEY` + the originating vocab label under `VOCAB_LABEL_PROPERTY_KEY`. Hosts `PrimitiveDescriptor` (per-primitive descriptor the materializer + workflow-editor consume; carries `id` / `kind()` / `cap_scope()` / `field_path`). NO new `PrimitiveKind` variants are minted — the test pin `tests/schema_compiler_emits_subgraph_with_no_new_primitive_kind_variants.rs` is load-bearing.

- **`schema_compiler/spec.rs`** (97 LOC) — `SchemaSubgraphSpec` content-addressed wrapper around a `benten_core::Subgraph`. `as_subgraph()` for canonical-bytes derivation + `into_subgraph()` for engine handoff. The crucial architectural property (pinned by `tests/schema_compiler_routes_through_existing_register_subgraph_surface_no_new_engine_method.rs` + `tests/schema_compiler_does_not_widen_register_subgraph_signature.rs`): the engine's `IntoSubgraphSpec for benten_eval::Subgraph` impl means `engine.register_subgraph(spec)` works WITHOUT widening the engine API or introducing a parallel registration surface — arch-r1-15.

- **`schema_compiler/error.rs`** (190 LOC) — `SchemaCompileError` with the nine variants the G23-A wave mints: `ValidationFailed`, `EmitNewPrimitiveRejected`, `SandboxHostFnRejected`, `VocabInvalidLabel`, `VocabEdgeMismatch`, `VocabScalarUnknown`, `VocabRefTargetMissing`, `VocabCycleRejected`, `VocabRequiredPropertyMissing`. Each maps to a stable `ErrorCode` (the nine `E_SCHEMA_*` codes minted at G23-A atomic Rust+TS per `§3.5g`).

- **`schema_compiler/ingest_dialect.rs`** (67 LOC) — the ingest-dialect surface skeleton. G23-A wave-4 ships the JSON-Schema dialect as the canary; future dialects (TS DSL, Python, etc.) plug in here per the dialect-pluggability framing in the crate-level docs. Engine-side parse locus per schema-r1-3 (browser may submit either canonical-bytes or dialect-source-bytes; T1 defense composes here).

### 3c. `materializer.rs` (G23-B; 1379 LOC, the largest file)

The materializer pipeline + Renderer abstraction in a single file. Eight load-bearing surfaces:

1. **`MaterializerEngine` trait** — the engine-side seam. `read_node_as(principal, cid) -> Result<Option<Node>, MaterializerError>` (routes through `Engine::read_node_as` per CLAUDE.md baked-in #18 Class B β; NEVER `read_node` — that's `pub(crate)` for engine internals) + `has_clock_injected()` (default `true`; real engine adapters return whether `Engine::open_with_clock` was used — fail-closed posture per sec-3.5-r1-7).

2. **`MaterializerCapRecheck` type alias** — `Arc<dyn Fn(&Cid, &str, &Cid) -> bool + Send + Sync + 'static>`. Same shape as `benten_engine::cap_recheck::CapRecheckFn`; materializer-view IS IVM view per D-4F-2, so the cap-recheck shape is reused. `allow_all_cap_recheck()` + `deny_all_cap_recheck()` helpers.

3. **`MaterializerError`** — three-variant error enum (`SchemaMismatch`, `SubscribeSeamFailure`, `UcanClockNotInjected` + a `code()` accessor). Each variant's identity uniquely determines its typed code (no per-variant redundant `code` field — Qual-1 #732; the dead `Backend` / `Other` variants were removed — Hyg-1 #312, forward-fire-only per §3.6h when a real adapter-error construction site lands). Catches the `E_MATERIALIZER_CAP_DENIED` / `E_MATERIALIZER_SCHEMA_MISMATCH` / `E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE` G23-B ErrorCodes + the `E_UCAN_CLOCK_NOT_INJECTED` Phase-3 inheritance.

4. **`MaterializerDenialFrame`** — per-Node cap-denial carrier surfaced in the redacted-view output per ratification #7 (denied Nodes return `Ok(out)`, not `Err`; the output bytes carry `[redacted]` placeholders + the frame carries the typed code for consumer rendering).

5. **`MaterializerWalkInputs`** — the input bundle: `engine` + `spec` + `content_cid` + `walk_principal` + `cap_recheck` + `declared_requires` (manifest envelope; empty Vec = allow-all). View identity per mat-r1-11 is the `(spec_cid, content_cid)` pair — two walks with the same pair produce the same canonical bytes (determinism test pin).

6. **`MaterializerOutput`** — `primary` + `secondary` bytes + `denials` + `materialized_cids` + `dispatched_kinds` (`HashSet<PrimitiveKind>` for the 12-primitive runtime-trace pin) + `spec_cid`. `canonical_cid()` BLAKE3s `(primary || 0xFF || secondary)` for the mat-r1-3 determinism pin.

7. **`Materializer` trait** — the polymorphic walk surface. Required: `materialize_with_gate<E: MaterializerEngine>`. Defaulted: `filter_rows_at_materialization` (per-row gate filter) + `dual_gate_admits` (mat-gate AND delivery-gate; deny-from-either wins per cap-r4-3). Two impls: `HtmlJsonMaterializer` (default) + `PlaintextMaterializer` (arch-r1-10 + D-4F-11 pluggability validation — empirical proof the trait isn't HtmlJson-specific).

8. **`Renderer` trait + `BrowserRender` default impl + `RenderError`** — transport-agnostic per arch-r1-16. `render(&self, output: &MaterializerOutput) -> Result<(), RenderError>` + `backend_name() -> &'static str`. `BrowserRender` is the wasm32-unknown-unknown shape (b) deployment default; `TauriRenderer` lives in the sibling crate per G24-E + CLAUDE.md #17 + #19. `tauri-runtime-verso` swap-readiness preserved per br-r1-9 — the trait surface names no transport methods. **At G23-B `BrowserRender::render` is a no-op stub**: it satisfies trait coherence + the arch-r1-16 doc-test assertion; the admin UI v0 shell at G24-A fills the DOM-mount logic via the napi bridge.

Internal walk machinery: `FormatBackend` enum (private), `SubscribeAttachToken` (the seam token; the actual `Engine::on_change_as_with_cursor` call happens at the consumer boundary to preserve dep direction), `extract_first_cap_scope`, a single shared recursive `Value`-tree walker `render_value<R: ValueRender>` with three leaf-rule impls (`HtmlRender` / `PlaintextRender` / `JsonRender`) — Qual-1 #730 collapsed the 4 near-identical hand-rolled walkers into one walker + three rule impls so a future `Value` scalar mint touches one place — exposed via the `render_value_html` / `render_value_plaintext` / `value_to_json` thin wrappers, plus `html_escape` / `json_escape` / `json_projection_for_node` / `json_projection_redacted` (the redacted projection emits per-field `null` + the scope array + `"redacted":true`), and the two format-specific `render_html_json` / `render_plaintext` per-Node renderers as private impl blocks. The materialization-layer per-row cap gate is invoked once for the authoritative content-CID decision and its bool is consumed (the prior discarded-bool per-primitive fan-out was observability-theater — removed per Safe-1 #527 / Qual-1 #702; per-primitive cap-scope is enforced upstream by the T1 envelope check + schema-compile `derive_scope`). The walk dispatches ONLY existing `PrimitiveKind` variants — the 12-primitive irreducibility runtime-trace pin (`tests/materializer_walks_only_existing_12_primitives_no_extension.rs`) is exhaustively-matched at walk time.

### 3d. `admin_ui_v0/` (G24-A + G24-B; 2 files, ~1.3k LOC)

- **`admin_ui_v0/mod.rs`** (537 LOC) — the admin UI v0 handler-side shell. Hosts `Category` (4 variants `Plugins` / `Workflows` / `ContentTypes` / `Views` per ratification #4 + plugin-arch-r1-12 + ux-r1-8; `label()` + `route_slug()` accessors), `NAV_CATEGORIES` (the canonical order locked here), `INDEXEDDB_SNAPSHOT_CACHE_STORE` + `INDEXEDDB_MANIFEST_STORE_STORE` (the ONLY two stores admin UI v0 shape (b) writes to per br-r1-7 + T2), `INDEXEDDB_FORBIDDEN_STORES` (the 10-entry forbidden list: `caps` / `cap_tokens` / `ucan` / `ucan_tokens` / `secrets` / `private_namespace` / `plugin_secrets` / `sync_state` / `loro_state` / `iroh_state`), `WINTERTC_FORBIDDEN_APIS` (DOM-only + FormData + relative-URL fetch patterns the CI guard at G26-B sweeps for in `packages/admin-ui-v0/src/`), `ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX = "private:admin-ui-v0"` (plugin-arch-r1-18 plugin-data-residency surface), `ADMIN_UI_V0_CLASS_B_BETA_READ_SEAM = "MaterializerEngine::read_node_as"` (grep-asserted), `ADMIN_UI_V0_SUBSCRIBE_SEAM = "on_change_as_with_cursor"` (sec-3.5-r1-9 grep-asserted). Two subgraph builders: `build_category_route_subgraph(category)` (per-category READ + TRANSFORM + RESPOND triple) and `build_admin_ui_v0_subgraph()` (the composite — 4 categories × 3 primitives = 12 nodes; every node carries its `admin_ui_v0_category` tag for walk-time traceability). Materializer consumer wiring: `render_category_content` + `render_category_content_allow_all` (G23-B §4.13 mr-3 + mr-5 — admin UI v0's content-render delegates to the materializer, not a bespoke `renderProperty`). `Subscriber::for_category` derives per-category subscribe patterns (`admin-ui-v0:<slug>:*`).

- **`admin_ui_v0/workflow_editor.rs`** (737 LOC) — G24-B workflow editor handler side. `WorkflowFormField` (one form field per `PrimitiveDescriptor` from a `SchemaSubgraphSpec`; carries id + kind + cap_scope + field_path), `WorkflowForm` (the full schema-driven form), `WorkflowPrimitiveSelection`, `WorkflowDraft`, `WorkflowEdge`, `WorkflowEditorError`. Three substantive surfaces: `derive_form_from_schema` (the schema-driven form generator — admin UI's per-primitive form is NOT hand-coded; future schema amendments cascade automatically), `compile_draft_within_manifest_envelope` (T1 + T4 defense — re-derives the cap-scope set from the emitted subgraph + verifies admissibility under the active `PluginManifest`'s `requires` envelope BEFORE the subgraph reaches `Engine::call_as`), `validate_subgraph_within_manifest_envelope` (the cap-elevation gate; surfaces `CapElevation` / `SubgraphInjection` errors), `derive_cap_scopes_from_subgraph`, `workflow_content_hash` (canonical-bytes hash via `benten_core::canonical_subgraph_bytes`).

---

## 4. Public API surface

### 4a. Schema compiler entry

- `schema_compiler::compile(bytes: &[u8]) -> Result<SchemaSubgraphSpec, SchemaCompileError>` — top-level entry. Routes JSON-Schema dialect bytes through parse + 4 validation passes + emit + the 12-primitive regression-guard.
- `schema_compiler::derive_scope(action, schema_name, field_path) -> String` — schema-derived cap-scope helper; load-bearing for sec-3.5-r1-4 (user-supplied scope NEVER taken).
- `SchemaSubgraphSpec::as_subgraph()` / `SchemaSubgraphSpec::into_subgraph()` — engine-handoff via existing `register_subgraph` surface; no new engine method needed.
- `VocabLabel` / `VocabEdge` / `Scalar` enums + their `VOCAB_*_NAMES` / `SCALAR_NAMES` const-slice grep-asserts + `VOCAB_REQUIRED_FIELD_PROPS`.
- `PrimitiveDescriptor` (consumed by materializer + workflow editor).
- `SCHEMA_COMPILER_PROPERTY_KEYS` const slice — 3 property-bag keys emitted nodes carry.

### 4b. Materializer + Renderer

- `Materializer::materialize_with_gate<E: MaterializerEngine>` — the polymorphic walk entry; required method.
- `materialize_html_json` / `materialize_plaintext` — free-function entry points; trait impls delegate.
- `HtmlJsonMaterializer` / `PlaintextMaterializer` — the two `Materializer` impls.
- `HtmlJsonMaterializer::subscribe_with_gate(pattern) -> Result<SubscribeAttachToken, MaterializerError>` — the seam token for the consumer-side `Engine::on_change_as_with_cursor` attach (the token's pattern feeds the engine surface at the consumer boundary).
- `Renderer::render(&self, output)` + `Renderer::backend_name(&self)` — the transport-abstraction surface.
- `BrowserRender` — default `Renderer` impl for deployment shape (b).
- `MaterializerEngine` trait — adapter surface; production callers (admin UI v0 + integration tests) implement against `benten_engine::Engine::read_node_as`.
- `MaterializerCapRecheck` + `allow_all_cap_recheck()` / `deny_all_cap_recheck()`.
- `MaterializerWalkInputs` + `MaterializerOutput` + `MaterializerDenialFrame`.
- `MaterializerError` (three variants + `code()` accessor).
- `RenderError` — `Transport(String)` opaque variant.

### 4c. Plugin manifest schema (G24-D + G24-D-FP-2)

- `PluginManifest` — the manifest body. Methods:
    - `validate()` — structural.
    - `validate_with_clock(now_secs)` — engine-injection-time validation seam per R4b-FP-1 + D-4F-15.
    - `validate_with_rotation_log(rotation_log)` — RotationLog-aware validation; surfaces `ValidationOutcome::ValidWithWarning` on rotation per D-4F-12.
    - `compute_content_cid()` — content-addressing.
    - `verify_content_cid_matches()` / `verify_peer_signature()` / `signing_payload()`.
    - `to_canonical_bytes()` — round-trippable manifest bytes (distinct from signing_payload).
    - `requires_sandbox_exec()` + `declares_time_bounded()`.
- `CapRequirement` (`is_private_namespace()`), `SharesPolicy::none()` / `SharesPolicy::permits_delegation`, `SharesPolicyDefault` (`None` / `Any` / `Matching` — `Any` requires explicit rules per R6-FP-A sec-r6r1-7 hardening), `SharesRule::matches`, `SharesTarget` (`Any` / `PluginDid(Did)` — `PluginAuthor` removed at sec-r6r1-8).
- `RendererConfig` + `RendererBackend` (`BrowserWasm32` / `TauriEmbeddedWebview` / `Other(String)`).
- `InstallRecord` + signing payload helpers; `consenting_user_did` omit-by-design per r2-cp-5.
- `ValidationOutcome` + `RotatedKeyWarning`.
- `ContentAddressed` trait shim.
- `detect_composition_cycle` — structural DFS using `benten_core::Subgraph`-shape; no new `PrimitiveKind`.
- `MANIFEST_CLOCK_NOT_INJECTED_SENTINEL = 0` — mirrors `benten-caps::ucan_grounded::DEFAULT_NOW_SECS = 0`.
- `sign_manifest` — signing helper.

### 4d. Plugin lifecycle (G24-D + G24-D-FP-1)

- `plugin_lifecycle::install_plugin` — the substantive install path (Layer-1 + Layer-2 + Layer-3 consent gates).
- `plugin_lifecycle::uninstall_plugin<R, P, S>` — the five-step cascade.
- `plugin_lifecycle::discover_new_version` — PULL-not-PUSH new-version detection.
- `plugin_lifecycle::plugin_did_for_entry` — convenience lookup.
- `UninstallOutcome` (six observable counters).
- `CapRevoker` / `PrivateNamespaceTeardown` / `SubscriptionRegistry` trait ports.
- `UninstallContext` bundle.
- `InMemoryUninstallCascade` + `InMemoryGrant` + `RevocationLogEntry` — substantive in-memory test default.
- `NewVersionDiscoveryOutcome`.

### 4e. Plugin library + manifest store

- `PluginLibrary::new` / `insert` / `remove` / `get` / `cids` / `entries` / `len` / `is_empty` / `set_active` / `active` / `versions_of` / `walk_mainline` / `anchor` / `as_subgraph`.
- `LibraryEntry`.
- `anchor_node_id(name)` + `version_node_id(cid)` — canonical id constructors.
- Const surface: `NODE_ID_LIBRARY_ROOT`, `HANDLER_ID_PLUGIN_LIBRARY`, `EDGE_LIBRARY_ANCHOR` (mirrors `VocabEdge::ItemType.as_str()`), `EDGE_VERSION_OF`, `EDGE_CURRENT`, `PROP_ANCHOR_PLUGIN_NAME`, `PROP_VERSION_MANIFEST_CID`, `PROP_VERSION_PLUGIN_DID`, `PROP_VERSION_INSTALLED_AT_NANOS`.
- `ManifestStore::new` / `install_plugin` / `load` / `notifications` / `clear` — verify-on-every-load durable surface.
- `DriftNotification::is_install_record_drift_warning`.

### 4f. Module ecosystem + workflow ↔ plugin

- `module_ecosystem::install_plugin` / `install_plugin_persisting_did` (DEPRECATED canary precursors; production goes through `plugin_lifecycle::install_plugin`).
- `InstallResult` + `InstallerShape`.
- `workflow_to_plugin::promote_workflow_to_plugin(workflow, peer_did, requires, shares)` — manifest minting.
- `workflow_to_plugin::is_promoted_to_plugin(cid, library_lookup)` — workflow vs plugin classifier.
- `WorkflowHandle`.

### 4g. Admin UI v0

- `Category` enum + `Category::label()` + `Category::route_slug()`.
- `NAV_CATEGORIES` const array.
- `build_admin_ui_v0_subgraph()` + `build_category_route_subgraph(category)`.
- `render_category_content<E: MaterializerEngine>` + `render_category_content_allow_all<E>`.
- `Subscriber::for_category(category)`.
- IndexedDB + WinterTC + private-namespace + seam-name consts (5 named above).
- Workflow editor: `derive_form_from_schema` / `compile_draft_within_manifest_envelope` / `validate_subgraph_within_manifest_envelope` / `derive_cap_scopes_from_subgraph` / `workflow_content_hash`.
- `WorkflowForm` / `WorkflowFormField` / `WorkflowPrimitiveSelection` / `WorkflowDraft` / `WorkflowEdge` / `WorkflowEditorError`.

### 4h. Registry (Phase-4-Meta-reserved)

- `Registry` trait + `RegistryEntry` / `DiscoveryQuery` / `DiscoveryResult`.
- `timeout_error_code()` — surfaces the reserved `E_REGISTRY_DISCOVERY_TIMEOUT` code.

---

## 5. ErrorCodes minted by this crate

Atomic Rust + TS per `§3.5g` cross-language rule-mirror discipline.

### G23-A schema_compiler (9 codes)

`E_SCHEMA_VALIDATION_FAILED` · `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED` · `E_SCHEMA_SANDBOX_HOST_FN_REJECTED` · `E_SCHEMA_VOCAB_INVALID_LABEL` · `E_SCHEMA_VOCAB_EDGE_MISMATCH` · `E_SCHEMA_VOCAB_SCALAR_UNKNOWN` · `E_SCHEMA_VOCAB_REF_TARGET_MISSING` · `E_SCHEMA_VOCAB_CYCLE_REJECTED` · `E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING`

### G23-B materializer (3 codes)

`E_MATERIALIZER_CAP_DENIED` · `E_MATERIALIZER_SCHEMA_MISMATCH` · `E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE`

### G24-D plugin manifest + lifecycle (15 codes total, atomic per §3.5g; subset)

Plugin-manifest envelope: `E_PLUGIN_MANIFEST_INVALID` · `E_PLUGIN_CONTENT_CID_MISMATCH` · `E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID` · `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` · `E_PLUGIN_HETEROGENEITY_INCOMPATIBLE` · `E_PLUGIN_COMPOSITION_CYCLE` · `E_PLUGIN_DELEGATION_OUTSIDE_ENVELOPE` · `E_PLUGIN_DEVICE_ATTESTATION_FORGED` (renamed per Ben Q ratification #3; preserves `E_PLUGIN_*` family prefix) · `E_PLUGIN_NEW_VERSION_AVAILABLE` · `E_REGISTRY_DISCOVERY_TIMEOUT` (Phase-4-Meta-reserved). The full list lives in `benten-errors/src/codes.rs` + mirrored in `packages/engine/src/errors.generated.ts`.

CATALOG_VARIANT_COUNT math per Ben ratification #2: 27 minted across G23-A + G23-B + G24-D / 10 absorbed / 17 net new (118 → 135 at Phase-4-Foundation R5 close).

---

## 6. Tests inventory (83 integration tests in `tests/`)

The test surface is unusually broad for a single crate, reflecting the crate's substantive scope per arch-r1-8. Notable groupings:

- **Architecture pins:** `arch_n_benten_platform_foundation_dep_direction.rs` (one-way dep direction at production), `materializer_walks_only_existing_12_primitives_no_extension.rs` + `schema_compiler_emits_subgraph_with_no_new_primitive_kind_variants.rs` + `schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension.rs` (CLAUDE.md #1 12-primitive irreducibility), `schema_compiler_does_not_widen_register_subgraph_signature.rs` + `schema_compiler_routes_through_existing_register_subgraph_surface_no_new_engine_method.rs` (arch-r1-15 no-engine-API-widening), `materializer_uses_read_node_as_only_never_read_node.rs` (CLAUDE.md #18 Class B β grep pin).

- **Schema compiler (G23-A) substantive:** the canonical Note fixture pin (`schema_compiler_emits_valid_subgraph_spec_for_canonical_note_type.rs`), the 4 typed-edge emission pins (item-type / key-type-value-type / ref-target / variant), the cap-scope annotation pair (presence + routability), the SANDBOX-host-fn rejection, the FieldRef-cycle rejection, the over-range field-types rejection, the unconstrained-EMIT-RESPOND rejection, the JSON-Schema-dialect parser pin, the `prop_schema_compile_is_idempotent_arbitrary_schemas.rs` proptest.

- **Materializer (G23-B) substantive:** canonical-bytes determinism, dual-gate composition (mat × delivery; deny-from-either wins), capability-denial returns redacted view, walks composed subgraph to HTML output, fires cap-policy at each primitive boundary (the substance arm for cap-policy threading), without-clock-injection surfaces `E_UCAN_CLOCK_NOT_INJECTED`, rejects subgraph with cap-scope mismatch, rejects subgraph with unregistered SANDBOX host-fn, two-impl pluggability round-trip (`materializer_output_backend_pluggable_two_impls_compile_and_round_trip.rs`), SUBSCRIBE seam pattern validation, per-row gate independent of delivery, `mat_deny_wins_composition`, `materializer_defense_in_depth_rejects_banned_sandbox_host_fn_for_handcoded_spec.rs`, `prop_materializer_idempotent_for_static_schemas.rs`.

- **Plugin manifest (G24-D) trust-model pins** — `plugin_did_is_ucan_audience_handle_not_attested_sub_identity.rs` (negative grep walks `crates/benten-id/src/` for forbidden device-DID-attestation patterns; uses `walkdir`), `plugin_content_carries_peer_did_signature_for_provenance.rs`, `plugin_content_cid_mismatch_rejected_on_receive.rs`, `plugin_cross_plugin_reference_uses_content_cid_not_author_did.rs`, `plugin_install_consent_required_at_install_time.rs`, `plugin_install_record_signed_by_user_did_not_benten_project_key.rs`, `plugin_manifest_signature_replay_with_different_nonce_rejected.rs`, `plugin_manifest_substitution_at_install_rejected.rs`, `plugin_manifest_full_round_trip.rs`, `plugin_manifest_load_clock_injection_transparent_at_engine_surface.rs` (R4b-FP-1 Seam 2 pin), `plugin_manifest_post_install_drift_detected.rs` (verify-on-every-load defense), `plugin_meta_composition_cycle_rejected.rs`, `plugin_private_namespace_cap_no_cross_plugin_delegation.rs`, `plugin_manifest_runtime_delegation_within_envelope.rs` + `plugin_manifest_runtime_delegation_outside_envelope_denied.rs`.

- **Plugin uninstall cascade (G24-D-FP-1):** five tests aligned 1:1 with the cascade steps — `plugin_uninstall_revokes_held_caps.rs`, `plugin_uninstall_revokes_all_delegated_caps.rs`, `plugin_uninstall_terminates_subscriptions.rs`, `plugin_uninstall_clears_private_namespace_data.rs`, `plugin_uninstall_cascade_revokes_delegated_caps.rs`.

- **Plugin upgrade + versioning (G24-D):** `plugin_upgrade_re_consent_required_on_shares_widening.rs`, `plugin_upgrade_rejects_version_downgrade.rs`, `plugin_upgrade_requires_same_author_did.rs`, `plugin_version_chain_dag_shape_supports_branches_and_forks.rs`, `plugin_new_version_available_notification.rs` (PULL-not-PUSH), `plugin_pull_not_push_no_manifest_schema_version_field.rs` (D-4F-13).

- **Library subgraph (R6-FP-D lift):** `plugin_library_subgraph_holds_all_versions_active_graph_references_current.rs`.

- **Admin UI v0 (G24-A + G24-B):** `admin_ui_v0_public_surface_presence_pins.rs`, `admin_ui_v0_atrium_share_unattested_peer_rejected.rs`, `admin_ui_v0_install_as_signed_plugin_across_two_atrium_peers.rs`, `admin_ui_v0_install_rejects_substituted_bundle_via_peer_did_signature.rs`, `admin_ui_v0_materializer_reactive_update_propagates_through_engine_on_change_as_with_cursor.rs` (substantive subscribe-seam end-to-end).

- **Cap-scope derivation:** `cap_scope_derivation_rejects_user_supplied_scope.rs` (sec-3.5-r1-4 authoritative pin — user-supplied `scope` is silently discarded).

- **Author rotation + signature attacks:** `plugin_manifest_author_key_rotation_round_trip.rs`, `plugin_provenance_rotated_key_surfaces_warning_via_rotation_log.rs`, `schema_author_not_in_admin_ui_trust_list_prompts_user.rs`, `schema_author_rotation_race_replay_rejected.rs`, `schema_with_forged_author_signature_rejected.rs`.

- **Heterogeneity:** `plugin_heterogeneity_incompatible.rs` (CLAUDE.md #17 + ds-r1-8 — `host:sandbox:exec` on thin-client install rejected).

- **R6-FP-A cluster pin:** `r6fp_a_plugin_trust_blocker_closures.rs` (the R6 R1 BLOCKER closure consolidated batch).

- **ErrorCode mirror pins:** `error_catalog_mints_3_g23_b_error_codes.rs`, `error_catalog_mints_9_g23_a_error_codes.rs`, `plugin_error_codes_atomic_rust_ts_mirror_pin.rs`.

- **End-to-end substantive pipeline:** `g24d_substantive_pipeline.rs` (full G24-D install → consent → render flow), `materializer_dual_gate_pim_2_end_to_end_would_fail_if_no_op.rs` (pim-2 end-to-end discipline — would FAIL if no-op'd).

- **Promotion + classifier:** `workflow_promoted_to_plugin_via_manifest_addition_no_shape_change.rs`.

- **Reserved surface pins:** `registry_phase_4_meta_reserved_no_production_callsites.rs`.

Shared test fixtures live in `tests/common/`: `admin_ui_v0_engine_adapter.rs` (the `MaterializerEngine` impl over `benten_engine::Engine` — the production-equivalent binding; admin UI v0 + future Phase-4-Foundation handlers embed the same shape at the napi consumer boundary), `manifest_fixtures.rs`, `materializer_fixtures.rs`, `schema_fixtures.rs`.

---

## 7. Trait-port discipline + dep-direction posture

The most architecturally distinctive shape in this crate is the **trait-port pattern** repeated across surfaces. Production-side `benten-platform-foundation` does NOT depend on `benten-eval` / `benten-graph` / `benten-engine`. Wherever this crate needs to do something the engine knows how to do (read a node, revoke a cap, terminate a subscription, tear down a namespace), the surface declares a trait + the engine-side adapter implements it at the consumer boundary.

Four trait ports live in the crate:

1. `MaterializerEngine` (in `materializer.rs`) — engine-side read seam for materializer walks. Adapter at `tests/common/admin_ui_v0_engine_adapter.rs` forwards to `Engine::read_node_as`.
2. `CapRevoker` (in `plugin_lifecycle.rs`) — engine-side adapter wires to `Engine::revoke_capability_by_grant_cid` (PR #199) iterated over the cap-store's `audience` / `issuer` indexes.
3. `PrivateNamespaceTeardown` (in `plugin_lifecycle.rs`) — engine-side adapter walks the storage backend for rows scope-prefixed `private:<plugin_did>:` and deletes each.
4. `SubscriptionRegistry` (in `plugin_lifecycle.rs`) — engine-side adapter walks the subscription registry for `subscriber_did == plugin_did` handles + terminates each.

Each port has an `InMemoryUninstallCascade`-style substantive default for in-crate testing. This is the same pattern `benten-caps::CapabilityPolicy` uses (the engine plugs the trait at the boundary; the implementing crate doesn't reach back into engine internals).

The dep-direction discipline is pinned by `tests/arch_n_benten_platform_foundation_dep_direction.rs`. Future agent proposals to add `benten-graph` / `benten-eval` / `benten-engine` as production deps must be rejected with reference to arch-r1-1 + arch-r1-15.

---

## 8. Thin-engine + composable-platform philosophy check

This crate is where the **v1 platform-shippable surface** meets the engine, so let me name what I see at HEAD:

- **Schema-driven everything.** The workflow editor's form is derived from the schema, not hand-coded. The materializer's walk is derived from the schema's emitted primitives, not hard-coded HtmlJson logic. The plugin library is a real `Subgraph`, not a side-table. The admin UI subgraph is composed from the 12 primitives, not a synthetic admin-UI primitive. Every surface that COULD be hand-coded is instead derived from a typed substrate — the schema vocab, the manifest envelope, the 12 primitives, the cap-scope derivation. If you find yourself reaching for a hand-coded form template, a bespoke render function, or a parallel registration surface, stop and ask whether you're re-doing what the substrate already gives you.

- **12-primitive irreducibility is structural, not aspirational.** Three pins enforce it at this crate's boundary: the `schema_compiler::mod.rs` `assert_canonical_primitive_kind` regression-guard (fires `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED` rather than letting a future 13th variant silently emit); the materializer's exhaustive-match walker (fires at runtime if any non-12 variant appears); the admin UI v0 subgraph builder + workflow editor's cap-scope re-derivation. CLAUDE.md baked-in #1 + #16 are kept structurally honest here.

- **Trust model is layered + signed end-to-end.** The four-identity-concepts model (content-CID + peer-DID signature + plugin-DID + user-DID) is what makes Phase-4-Foundation a real platform: plugins are shareable subgraphs (CLAUDE.md #18), every cap-chain traces back to user-DID-issued root grants (Layer-1), install-time manifest envelopes are signed (Layer-2), runtime delegation is policy-bounded (Layer-3). The chain validator lives at `benten-caps::manifest_envelope_chain_validation`; the validation is wired through `plugin_lifecycle::install_plugin`.

- **Three deployment shapes preserved.** `BrowserRender` lives in this crate (shape b); `TauriRenderer` lives in the sibling `benten-renderer-tauri` (shape c per CLAUDE.md #17 + #19); the engine remains shape (a). The `Renderer` trait abstraction has no transport methods so `tauri-runtime-verso` swap-readiness is preserved per br-r1-9.

- **What's NOT here.** SANDBOX runtime (engine-side, native-only). IVM machinery (`benten-ivm`). Storage backends (`benten-graph`). Capability policy backends (`benten-caps`). Identity primitives (`benten-id`). Sync runtime (`benten-sync`). Engine evaluator (`benten-eval` + `benten-engine`). This crate ASSEMBLES those layers into a platform surface; it does not re-implement them.

- **What's reserved.** Decentralized registry (Phase-4-Meta). Full Atrium-substrate publish/subscribe (Phase-4-Meta — the trait shapes are minted at G24-D + the timeout ErrorCode reserved). Self-composing admin meta-circular work (Phase-4-Meta). Plugin author-DID-based targeting in `SharesTarget` (rejected at R6-FP-A sec-r6r1-8; mint a NEW typed variant THEN if resolver-lookup wiring is added).

---

## 9. Open notes / known limitations

- `missing_docs` is currently `#![allow(...)]` at crate root. Tightening before tag is a phase-close housekeeping item.
- `module_ecosystem::install_plugin` legacy path is DEPRECATED but retained for three pre-R4b-FP-1 integration tests. Migration to `plugin_lifecycle::install_plugin` is in flight.
- `ManifestStore` redb persistence is in-memory canonical at G24-D-FP-1; durable backing tracked at `docs/future/phase-4-backlog.md §4.11`.
- `BrowserRender::render` is a no-op stub at G23-B; the admin UI v0 shell at G24-A fills the DOM-mount logic via the napi bridge.
- Registry trait shapes exist + the `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode is reserved; the substrate ships at Phase-4-Meta.
- Per-device-local active reference in `PluginLibrary` is in-memory only at G24-D wave; production persistence parallels `ManifestStore`'s redb backing.
- 2 plugin-manifest seam build issues + 2 enhancement seams that surfaced at R4b L1 are tracked at `docs/future/phase-4-meta.md §4.19`.

---

*Audit doc; not normative. The canonical state is the code at HEAD plus `docs/PLUGIN-MANIFEST.md` + `docs/SCHEMA-DRIVEN-RENDERING.md` + `docs/ADMIN-UI.md`. CLAUDE.md baked-in #1 (12 primitives) + #15 (v1 platform-shippable gate) + #17 (3 deployment shapes) + #18 (4-identity plugin trust model) + #19 (engine extensions vs app-level plugins) frame everything this crate exists to do.*
