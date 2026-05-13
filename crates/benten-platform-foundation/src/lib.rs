//! `benten-platform-foundation` — Phase 4-Foundation G23-A + G23-B + G24-D canary STUB.
//!
//! **R3 RED-PHASE landing state** (2026-05-12): empty stub crate plus the
//! [`plugin_manifest`] module exporting the stub `PluginManifest` type-shape
//! that Family G + Family F1 + Family F2 depend on. R3 test pins reference
//! symbols that DON'T EXIST YET — they compile-but-fail at the `use` line
//! (canonical RED-phase per pim-12 §3.6e). G23-A wave-4 + G23-B wave-5 +
//! G24-D wave-7 fill the substantive content.
//!
//! ## Scope (post-R5; for orientation)
//!
//! Per Ben D-4F-2 ratification (2026-05-11): this is the 11th workspace crate.
//! Hosts FOUR substantive surfaces:
//!
//! 1. **`schema_compiler`** (G23-A wave-4) — schema-as-subgraph-of-primitive-typed-field-Nodes
//!    parser using the RATIFIED VOCABULARY (D-4F-NEW-TYPED-FIELD-NODE-VOCAB):
//!    8 labels (`SchemaRoot` / `FieldScalar` / `FieldObject` / `FieldList` /
//!    `FieldMap` / `FieldRef` / `FieldEnum` / `FieldUnion`); 6 edges (`FIELD` /
//!    `ITEM_TYPE` / `KEY_TYPE` / `VALUE_TYPE` / `REF_TARGET` / `VARIANT`); 8
//!    scalars (text/int/float/bool/bytes/bytes-cid/timestamp-hlc/null per
//!    `benten-core::Value`); 4 mandatory field properties
//!    (`name`/`required`/`default`/`scope` — `scope` schema-derived per
//!    sec-3.5-r1-4 NOT user-supplied).
//!
//! 2. **`ingest_dialect`** (G23-A wave-4 sub-module) — JSON-Schema / TS DSL /
//!    Python / other dialect parsers that translate input → canonical
//!    subgraph form. Engine-side parse locus per schema-r1-3 (browser may
//!    submit either canonical-bytes or dialect-source-bytes; T1 defense
//!    composes here).
//!
//! 3. **`materializer`** (G23-B wave-5) — `Materializer` trait +
//!    `HtmlJsonMaterializer` default impl + `PlaintextMaterializer` 2nd impl
//!    (arch-r1-10 output-FORMAT pluggability validation per cag-r1-6);
//!    `Renderer` trait abstraction; BrowserRender default impl. TauriRender
//!    lives in sibling crate `benten-renderer-tauri` per G24-E NEW wave.
//!
//! 4. **`plugin_manifest`** (G24-D wave-7) — FULL plugin manifest schema +
//!    install/uninstall/upgrade lifecycle + plugin library + meta-plugin
//!    composition per CLAUDE.md #18. R3 has the stub type-shape only.
//!    G24-D-FP-1 + G24-D-FP-2 add uninstall-cascade + manifest-envelope-
//!    chain-validation seams. See `docs/PLUGIN-MANIFEST.md`.
//!
//! ## Dep direction (D-4F-2 + arch-r1-1 + arch-r1-15)
//!
//! - Depends on: `benten-core` (Subgraph + SubgraphSpec + PrimitiveKind +
//!   Value), `benten-errors` (ErrorCode), `benten-id` (Did for peer-DID +
//!   plugin-DID + user-DID per CLAUDE.md #18 four-identity-concepts model).
//! - **Must not** depend on: `benten-eval`, `benten-graph`, `benten-engine`
//!   (preserves arch-1 thinness-test). Pinned by test
//!   `tests/arch_n_benten_platform_foundation_dep_direction.rs` at R5.
//!
//! ## ErrorCodes minted at G23-A (9 codes; atomic Rust+TS per §3.5g)
//!
//! - `E_SCHEMA_VALIDATION_FAILED`
//! - `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED`
//! - `E_SCHEMA_SANDBOX_HOST_FN_REJECTED`
//! - `E_SCHEMA_VOCAB_INVALID_LABEL`
//! - `E_SCHEMA_VOCAB_EDGE_MISMATCH`
//! - `E_SCHEMA_VOCAB_SCALAR_UNKNOWN`
//! - `E_SCHEMA_VOCAB_REF_TARGET_MISSING`
//! - `E_SCHEMA_VOCAB_CYCLE_REJECTED`
//! - `E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING`
//!
//! ## ErrorCodes minted at G23-B (3 codes)
//!
//! - `E_MATERIALIZER_CAP_DENIED`
//! - `E_MATERIALIZER_SCHEMA_MISMATCH`
//! - `E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE`
//!
//! At R3 these ErrorCodes DO NOT EXIST in `benten-errors`. R3 test pins assert
//! their post-R5 presence by attempting to parse the string forms via
//! `ErrorCode::from_str` and matching against `ErrorCode::Unknown` for the
//! current RED-PHASE state. Real variants land at G23-A / G23-B with the
//! canary commit.

#![allow(dead_code, clippy::needless_pass_by_value, missing_docs)]

pub mod module_ecosystem;
pub mod plugin_lifecycle;
pub mod plugin_library;
pub mod plugin_manifest;
pub mod registry;
pub mod schema_compiler;
pub mod workflow_to_plugin;

pub use plugin_manifest::{
    CapRequirement, ContentAddressed, InstallRecord, PluginManifest, RendererBackend,
    RendererConfig, SharesPolicy, SharesPolicyDefault, SharesRule, SharesTarget, sign_manifest,
};

// G23-A schema_compiler canary public surface — the entry-point
// `schema_compiler::compile` + `SchemaSubgraphSpec` wrapper + the
// vocabulary types. Downstream G23-B materializer + G24-D plugin manifest
// + test pins consume these.
pub use schema_compiler::{
    PrimitiveDescriptor, SCHEMA_COMPILER_PROPERTY_KEYS, Scalar, SchemaCompileError,
    SchemaSubgraphSpec, VocabEdge, VocabLabel, compile as compile_schema,
};
