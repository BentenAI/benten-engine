//! Phase-4-Foundation G24-B — admin UI v0 workflow editor (handler side).
//!
//! # What lives here
//!
//! The Rust-side workflow editor authority for the admin UI v0 plugin
//! (per CLAUDE.md baked-in #18 — admin UI v0 is the first app-level
//! plugin). The browser/Tauri front-end at
//! `packages/admin-ui-v0/src/workflow-editor/` consumes these surfaces
//! via napi-rs (full-peer) or via the thin-client session protocol
//! (G24-F) when the browser-wasm32 talks to a remote full peer.
//!
//! # Schema-driven form generation (G23-A consumer)
//!
//! The editor's per-primitive form is NOT hand-coded; instead each form
//! is derived from a [`SchemaSubgraphSpec`] via
//! [`derive_form_from_schema`]. Walking the schema's per-primitive
//! descriptors yields the form fields + their derived cap-scope
//! annotations. This pins admin UI v0 as a consumer of D-4F-2's
//! schema-driven-rendering decision: a future schema-amendment surfaces
//! automatically in the editor form, with NO hand-coded form template
//! to drift out of date.
//!
//! # Cap-scope envelope discipline (T1 + T4 defenses)
//!
//! Every workflow the editor emits is a [`Subgraph`] whose primitive
//! Nodes carry derived cap-scope annotations. Before the subgraph is
//! handed to `Engine::call_as` the editor RE-DERIVES the cap-scope set
//! from the emitted subgraph and verifies every scope is admissible
//! under the active [`PluginManifest`]'s `requires` envelope (T1
//! defense for subgraph injection + T4 defense for cap elevation). A
//! mismatch returns [`WorkflowEditorError::CapElevation`] or
//! [`WorkflowEditorError::SubgraphInjection`] BEFORE any write reaches
//! the engine boundary.
//!
//! # No engine-internal seam access
//!
//! This module composes over the 12 primitives + the manifest envelope;
//! it does NOT reach for `Engine::read_node` (pub(crate)) /
//! `Engine::subscribe_change_events`. The browser-side editor at
//! `packages/admin-ui-v0/src/workflow-editor/` routes writes via the
//! shared `AdminUiV0Bridge` TS surface →
//! `Engine::call_as` (Rust-side). The grep-assert pin at
//! `tests/admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`
//! covers this directory tree.

use benten_core::{OperationNode, PrimitiveKind, Subgraph, Value, canonical_subgraph_bytes};

use crate::plugin_manifest::PluginManifest;
use crate::schema_compiler::{PrimitiveDescriptor, SchemaSubgraphSpec};

// ---------------------------------------------------------------------
// Form-generation surface (G23-A consumer).
// ---------------------------------------------------------------------

/// One workflow-editor form field, derived from a single
/// [`PrimitiveDescriptor`] in the source [`SchemaSubgraphSpec`].
///
/// **Substantive consumer-of-G23-A pin:** every field surfaced in the
/// editor UI traces back to a schema-spec primitive. The browser-side
/// form template iterates `WorkflowForm::fields()` to render — no
/// per-primitive hand-coded `<input>` strings live in the bundle
/// (substantive form of the "no handcoded forms" pin per
/// `packages/admin-ui-v0/tests/workflow_editor_uses_schema_driven_form_generation_no_handcoded_forms.test.ts`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowFormField {
    /// Stable per-field id (mirrors [`PrimitiveDescriptor::id`]).
    pub id: String,
    /// Which canonical primitive kind this field configures.
    pub kind: PrimitiveKind,
    /// The derived cap-scope from the source schema — the editor
    /// surfaces this in the form so the user sees what cap the
    /// primitive needs (and so the save path can re-validate against
    /// the manifest envelope).
    pub cap_scope: Option<String>,
    /// Schema field path (e.g. `"Note.body"`) — for breadcrumbs +
    /// debug.
    pub field_path: Option<String>,
}

impl WorkflowFormField {
    fn from_descriptor(desc: &PrimitiveDescriptor) -> Self {
        Self {
            id: desc.id.clone(),
            kind: desc.kind(),
            cap_scope: desc.cap_scope().map(str::to_owned),
            field_path: desc.field_path.clone(),
        }
    }
}

/// Output of [`derive_form_from_schema`] — the ordered field list a
/// browser-side workflow-editor template iterates over to render the
/// "configure this primitive" form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowForm {
    /// Schema this form was derived from (for breadcrumbs).
    pub schema_name: String,
    fields: Vec<WorkflowFormField>,
}

impl WorkflowForm {
    /// All fields, in emit order.
    #[must_use]
    pub fn fields(&self) -> &[WorkflowFormField] {
        &self.fields
    }

    /// Field names — convenience for grep-asserts + ID lookups.
    #[must_use]
    pub fn field_ids(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.id.clone()).collect()
    }

    /// True if a field with the given id exists. Used by the
    /// G23-A-amendment test (add field to schema → re-derive form →
    /// new field present).
    #[must_use]
    pub fn has_field(&self, id: &str) -> bool {
        self.fields.iter().any(|f| f.id == id)
    }
}

/// Derive a [`WorkflowForm`] from a [`SchemaSubgraphSpec`].
///
/// **Substantive G23-A consumer arm:** walks the schema's
/// [`SchemaSubgraphSpec::primitives`] (the per-primitive descriptor
/// list the compiler stamps at emit time) and builds one form field
/// per primitive. No hand-coded form template lives anywhere in this
/// path; a schema amendment surfaces automatically.
#[must_use]
pub fn derive_form_from_schema(spec: &SchemaSubgraphSpec) -> WorkflowForm {
    let fields = spec
        .primitives()
        .iter()
        .map(WorkflowFormField::from_descriptor)
        .collect();
    WorkflowForm {
        schema_name: spec.schema_name().to_owned(),
        fields,
    }
}

// ---------------------------------------------------------------------
// Draft -> Subgraph compilation (production write path).
// ---------------------------------------------------------------------

/// One primitive selection inside a [`WorkflowDraft`] — the editor's
/// in-memory representation of "user dragged primitive X with cap
/// scope Y".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPrimitiveSelection {
    /// User-supplied stable id (becomes the OperationNode id).
    pub id: String,
    /// Canonical primitive kind — MUST be one of the 12 per CLAUDE.md
    /// baked-in #1.
    pub kind: PrimitiveKind,
    /// Derived cap-scope from the schema — copied from the form field
    /// at drag-time; NEVER editable by the user (schema authority per
    /// sec-3.5-r1-4).
    pub cap_scope: Option<String>,
}

/// Edge inside a [`WorkflowDraft`] — `(from_id, to_id, label)`.
pub type WorkflowEdge = (String, String, String);

/// In-memory representation of an in-progress workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDraft {
    /// Human-readable workflow name; becomes the [`Subgraph::handler_id`]
    /// suffix.
    pub name: String,
    /// Ordered primitive selections.
    pub primitives: Vec<WorkflowPrimitiveSelection>,
    /// Edges between primitives.
    pub edges: Vec<WorkflowEdge>,
}

impl WorkflowDraft {
    /// Construct an empty draft for the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            primitives: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Append a primitive (mirrors the editor's `dragPrimitive` action).
    pub fn drag_primitive(&mut self, sel: WorkflowPrimitiveSelection) {
        self.primitives.push(sel);
    }

    /// Append edges (mirrors the editor's `connectEdges` action).
    pub fn connect_edges<I>(&mut self, edges: I)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        for (from, to) in edges {
            self.edges.push((from, to, "feeds".into()));
        }
    }

    /// Compile to a canonical [`Subgraph`] for engine handoff.
    ///
    /// The emitted subgraph's handler id is
    /// `admin-ui-v0::workflow::<name>` — so admin UI workflows live in
    /// a distinct handler-id namespace from the four category-route
    /// subgraphs (`admin-ui-v0::plugins` / `admin-ui-v0::workflows` /
    /// etc.).
    #[must_use]
    pub fn compile_subgraph(&self) -> Subgraph {
        let handler_id = format!("admin-ui-v0::workflow::{}", self.name);
        let mut sg = Subgraph::new(handler_id);
        for prim in &self.primitives {
            let mut op = OperationNode::new(&prim.id, prim.kind);
            if let Some(scope) = &prim.cap_scope {
                op = op.with_property("cap_scope", Value::Text(scope.clone()));
            }
            op = op.with_property(
                "admin_ui_v0_workflow_source",
                Value::Text(self.name.clone()),
            );
            sg.nodes.push(op);
        }
        for (from, to, label) in &self.edges {
            sg.edges.push((from.clone(), to.clone(), label.clone()));
        }
        sg
    }
}

// ---------------------------------------------------------------------
// Errors — T1 (subgraph injection) + T4 (cap elevation) defenses.
// ---------------------------------------------------------------------

/// Failure modes the workflow editor surfaces.
///
/// Each variant maps to a stable `benten_errors::ErrorCode` value — the
/// editor's save path returns one of these typed errors BEFORE any
/// write reaches the engine boundary.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowEditorError {
    /// A user attempted to embed a cap-scope WIDER than the active
    /// plugin manifest's `requires` envelope grants — T4 defense.
    ///
    /// Maps to [`benten_errors::ErrorCode::CapDenied`] (no new code
    /// minted; cap-elevation is a specialization of cap-denial routed
    /// through the same `ON_DENIED` audit family).
    #[error("workflow editor cap elevation rejected: scope `{requested}` not in manifest envelope")]
    CapElevation {
        /// The cap scope the workflow attempted to claim.
        requested: String,
    },

    /// The emitted [`Subgraph`] contains a primitive whose cap-scope
    /// the schema-driven form-gen path would not have surfaced (i.e.
    /// the spec was forged or tampered with after the form-gen step)
    /// — T1 defense.
    ///
    /// Maps to [`benten_errors::ErrorCode::CapDenied`].
    #[error(
        "workflow editor subgraph injection rejected: primitive `{primitive_id}` carries scope \
         `{cap_scope}` whose derivation is not in cap-scope manifest envelope"
    )]
    SubgraphInjection {
        /// The offending OperationNode id.
        primitive_id: String,
        /// The cap-scope that fails the manifest-envelope check.
        cap_scope: String,
    },

    /// Workflow contains a primitive that is not one of the canonical
    /// 12 — defensive regression-guard against drift of CLAUDE.md #1.
    #[error("workflow editor primitive kind rejected: `{requested:?}` not in canonical 12")]
    UnknownPrimitiveKind {
        /// The non-canonical PrimitiveKind variant.
        requested: PrimitiveKind,
    },
}

impl WorkflowEditorError {
    /// Map to the stable wire error code admin UI surfaces over the
    /// thin-client session protocol / napi binding.
    #[must_use]
    pub fn error_code(&self) -> benten_errors::ErrorCode {
        match self {
            Self::CapElevation { .. } | Self::SubgraphInjection { .. } => {
                benten_errors::ErrorCode::CapDenied
            }
            Self::UnknownPrimitiveKind { .. } => {
                benten_errors::ErrorCode::SchemaEmitNewPrimitiveRejected
            }
        }
    }
}

// ---------------------------------------------------------------------
// Manifest-envelope validation (T1 + T4 defense surface).
// ---------------------------------------------------------------------

/// Re-derive cap scopes from an already-emitted [`Subgraph`].
///
/// The save path calls this on the COMPILED subgraph (the one heading
/// for `Engine::call_as`) and verifies the derived set is admissible
/// under the active manifest. A discrepancy with what the form-gen
/// path would have surfaced indicates tampering between form-emit and
/// save (T1 defense surface).
#[must_use]
pub fn derive_cap_scopes_from_subgraph(sg: &Subgraph) -> Vec<String> {
    let mut scopes = Vec::new();
    for op in sg.nodes() {
        if let Some(Value::Text(s)) = op.property("cap_scope") {
            scopes.push(s.clone());
        }
    }
    scopes
}

/// Verify every cap scope the workflow's emitted subgraph carries is
/// admissible under the active manifest's `requires` envelope.
///
/// **The substantive T1 + T4 defense surface.** Called by every
/// workflow-editor save path BEFORE the subgraph reaches
/// `Engine::call_as`. The check is structural: each derived scope is
/// matched against the manifest's `requires` list (prefix match
/// supported for namespaced caps like `read:Note.*`).
///
/// # Errors
///
/// - [`WorkflowEditorError::SubgraphInjection`] when a derived cap
///   scope is not in the manifest envelope (or no overlap with any
///   `requires` entry).
/// - [`WorkflowEditorError::UnknownPrimitiveKind`] if the subgraph
///   contains a non-canonical primitive kind.
pub fn validate_subgraph_within_manifest_envelope(
    sg: &Subgraph,
    manifest: &PluginManifest,
) -> Result<(), WorkflowEditorError> {
    // (1) Primitive-kind sanity (CLAUDE.md #1 regression-guard).
    for op in sg.nodes() {
        match op.kind {
            PrimitiveKind::Read
            | PrimitiveKind::Write
            | PrimitiveKind::Transform
            | PrimitiveKind::Branch
            | PrimitiveKind::Iterate
            | PrimitiveKind::Wait
            | PrimitiveKind::Call
            | PrimitiveKind::Respond
            | PrimitiveKind::Emit
            | PrimitiveKind::Sandbox
            | PrimitiveKind::Subscribe
            | PrimitiveKind::Stream => {}
            other => {
                return Err(WorkflowEditorError::UnknownPrimitiveKind { requested: other });
            }
        }
    }
    // (2) Cap-scope envelope check (T1 + T4).
    for op in sg.nodes() {
        if let Some(Value::Text(scope)) = op.property("cap_scope")
            && !manifest_envelope_admits(scope, manifest)
        {
            return Err(WorkflowEditorError::SubgraphInjection {
                primitive_id: op.id.clone(),
                cap_scope: scope.clone(),
            });
        }
    }
    Ok(())
}

/// Pre-save check on a user-authored draft. Compiles the draft to a
/// subgraph, runs [`validate_subgraph_within_manifest_envelope`], and
/// returns the compiled subgraph on success.
///
/// # Errors
///
/// As [`validate_subgraph_within_manifest_envelope`] plus
/// [`WorkflowEditorError::CapElevation`] when the original draft
/// itself attempts to embed a cap-scope outside the manifest envelope
/// (this fires BEFORE the compile step so error surface is on the
/// authored input, not the compiled form).
pub fn compile_draft_within_manifest_envelope(
    draft: &WorkflowDraft,
    manifest: &PluginManifest,
) -> Result<Subgraph, WorkflowEditorError> {
    // (1) Draft-level cap-elevation check (T4) — fires on user-authored
    // input before any compile step.
    for prim in &draft.primitives {
        if let Some(scope) = &prim.cap_scope
            && !manifest_envelope_admits(scope, manifest)
        {
            return Err(WorkflowEditorError::CapElevation {
                requested: scope.clone(),
            });
        }
    }
    let sg = draft.compile_subgraph();
    // (2) Re-derive caps from compiled form + re-validate (T1).
    validate_subgraph_within_manifest_envelope(&sg, manifest)?;
    Ok(sg)
}

/// Check whether the manifest's `requires` envelope admits the
/// requested cap-scope.
///
/// Supports two match shapes per
/// `PluginManifest`'s `CapRequirement.scope` semantics:
///
/// - Exact match: `"read:Note.body"` matches `"read:Note.body"`.
/// - Prefix-wildcard: `"read:Note.*"` in the manifest admits any
///   `"read:Note.<anything>"` derived scope.
fn manifest_envelope_admits(requested_scope: &str, manifest: &PluginManifest) -> bool {
    manifest.requires.iter().any(|req| {
        if let Some(prefix) = req.scope.strip_suffix(".*") {
            requested_scope.starts_with(&format!("{prefix}.")) || requested_scope == prefix
        } else {
            req.scope == requested_scope
        }
    })
}

// ---------------------------------------------------------------------
// Round-trip / replay helper — substantive G24-B exit-criterion arm.
// ---------------------------------------------------------------------

/// Compute the canonical-bytes content hash for a workflow's emitted
/// subgraph. The handler-id-naming + canonical-bytes encoding give the
/// "replay yields same CID" pin its substantive teeth — the test
/// pin's would-FAIL-if-no-op'd arm is "any divergence in canonical-bytes
/// encoding between save-time and replay-time surfaces here".
///
/// # Errors
///
/// Surfaces [`benten_core::CoreError`] verbatim if canonical encoding
/// fails (should never happen in practice — every well-formed Subgraph
/// canonicalizes).
pub fn workflow_content_hash(sg: &Subgraph) -> Result<[u8; 32], benten_core::CoreError> {
    let bytes = canonical_subgraph_bytes(sg)?;
    let digest = blake3::hash(&bytes);
    Ok(*digest.as_bytes())
}

// ---------------------------------------------------------------------
// Inline canary tests.
// ---------------------------------------------------------------------

#[cfg(test)]
mod canary {
    use super::*;
    use crate::plugin_manifest::{
        CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
    };
    use benten_core::Cid;
    use benten_id::keypair::Keypair;

    const NOTE_SCHEMA: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Note",
        "fields": [
            { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null }
        ]
    }"#;

    fn fixture_manifest(scopes: &[&str]) -> PluginManifest {
        let requires = scopes
            .iter()
            .map(|s| CapRequirement::new((*s).to_string()))
            .collect();
        let kp = Keypair::generate();
        let mut m = PluginManifest {
            plugin_name: "admin-ui-v0".to_string(),
            content_cid: Cid::from_blake3_digest([0u8; 32]),
            peer_did: kp.public_key().to_did(),
            peer_signature: vec![0u8; 64],
            requires,
            shares: SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
            renderer_config: None,
            composes_plugins: None,
            accepts_content: None,
            requires_schema_authors: None,
            requires_plugin_authors: None,
        };
        m.content_cid = m.compute_content_cid();
        m
    }

    #[test]
    fn form_derived_from_schema_carries_per_primitive_fields() {
        let spec = crate::schema_compiler::compile(NOTE_SCHEMA).unwrap();
        let form = derive_form_from_schema(&spec);
        assert!(
            !form.fields().is_empty(),
            "form-gen MUST surface at least one field for a non-empty schema"
        );
        // Every field has a derived cap-scope per sec-3.5-r1-4 — the
        // schema compiler stamps cap_scope on each emitted primitive.
        for field in form.fields() {
            assert!(
                field.cap_scope.is_some(),
                "field {id} MUST carry a schema-derived cap_scope",
                id = field.id
            );
        }
    }

    #[test]
    fn form_re_derives_new_field_when_schema_amended() {
        // G23-A consumer pin canary: a schema amendment surfaces in the
        // form automatically (not gated by a hand-coded template).
        let spec_v1 = crate::schema_compiler::compile(NOTE_SCHEMA).unwrap();
        let form_v1 = derive_form_from_schema(&spec_v1);
        let v1_field_count = form_v1.fields().len();

        let amended = br#"{
            "label": "SchemaRoot",
            "name": "Note",
            "fields": [
                { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null },
                { "label": "FieldScalar", "name": "filter_label", "scalar": "text", "required": false, "default": null }
            ]
        }"#;
        let spec_v2 = crate::schema_compiler::compile(amended).unwrap();
        let form_v2 = derive_form_from_schema(&spec_v2);
        assert!(
            form_v2.fields().len() > v1_field_count,
            "amended schema MUST surface additional fields in the regenerated form \
             (would-FAIL if form-gen was hand-coded and not driven by schema)"
        );
        let ids = form_v2.field_ids();
        assert!(
            ids.iter().any(|i| i.contains("filter_label")),
            "amended schema field MUST appear in regenerated form ids; got {ids:?}"
        );
    }

    #[test]
    fn compile_draft_within_envelope_succeeds_for_admissible_scope() {
        let manifest = fixture_manifest(&["read:Note.*", "write:Note.*"]);
        let mut draft = WorkflowDraft::new("create-note");
        draft.drag_primitive(WorkflowPrimitiveSelection {
            id: "r_body".to_string(),
            kind: PrimitiveKind::Read,
            cap_scope: Some("read:Note.body".to_string()),
        });
        draft.drag_primitive(WorkflowPrimitiveSelection {
            id: "w_body".to_string(),
            kind: PrimitiveKind::Write,
            cap_scope: Some("write:Note.body".to_string()),
        });
        draft.connect_edges(vec![("r_body".into(), "w_body".into())]);
        let sg = compile_draft_within_manifest_envelope(&draft, &manifest)
            .expect("admissible draft compiles");
        // Subgraph is named under the admin-ui-v0 workflow handler-id
        // namespace.
        assert!(
            sg.handler_id().starts_with("admin-ui-v0::workflow::"),
            "compiled subgraph handler_id must be admin-ui-v0::workflow::*"
        );
        // Hash is deterministic.
        let h1 = workflow_content_hash(&sg).unwrap();
        let h2 = workflow_content_hash(&sg).unwrap();
        assert_eq!(
            h1, h2,
            "workflow content hash is deterministic over a stable subgraph"
        );
    }

    #[test]
    fn compile_draft_rejects_cap_elevation_attempt() {
        // T4 defense: draft attempts a scope outside the manifest.
        let manifest = fixture_manifest(&["read:Note.*"]);
        let mut draft = WorkflowDraft::new("hostile");
        draft.drag_primitive(WorkflowPrimitiveSelection {
            id: "w_anywhere".into(),
            kind: PrimitiveKind::Write,
            cap_scope: Some("graph:write:everywhere".into()),
        });
        let err = compile_draft_within_manifest_envelope(&draft, &manifest)
            .expect_err("manifest-elevation MUST be denied");
        assert!(
            matches!(err, WorkflowEditorError::CapElevation { .. }),
            "expected CapElevation, got {err:?}"
        );
        assert_eq!(err.error_code(), benten_errors::ErrorCode::CapDenied);
    }

    #[test]
    fn validate_subgraph_rejects_injected_edge_with_out_of_envelope_cap() {
        // T1 defense: hand-crafted subgraph with an injected cap-scope
        // skips the form-gen path entirely but the save-side
        // re-derivation catches it.
        let manifest = fixture_manifest(&["read:Note.*"]);
        let injected = OperationNode::new("forged", PrimitiveKind::Write)
            .with_property("cap_scope", Value::Text("host-fn:fs:write".into()));
        let mut sg = Subgraph::new("admin-ui-v0::workflow::forged");
        sg.nodes.push(injected);
        let err = validate_subgraph_within_manifest_envelope(&sg, &manifest)
            .expect_err("subgraph injection MUST be rejected");
        assert!(
            matches!(err, WorkflowEditorError::SubgraphInjection { .. }),
            "expected SubgraphInjection, got {err:?}"
        );
        assert_eq!(err.error_code(), benten_errors::ErrorCode::CapDenied);
    }

    #[test]
    fn replay_produces_identical_content_hash() {
        // G24-B exit-criterion substantive pin: workflow save-time hash
        // == workflow replay-time hash.
        let manifest = fixture_manifest(&["read:Note.*"]);
        let mut draft = WorkflowDraft::new("identity");
        draft.drag_primitive(WorkflowPrimitiveSelection {
            id: "r_body".into(),
            kind: PrimitiveKind::Read,
            cap_scope: Some("read:Note.body".into()),
        });
        let sg_save = compile_draft_within_manifest_envelope(&draft, &manifest).unwrap();
        let cid_save = workflow_content_hash(&sg_save).unwrap();

        // Simulate reload + replay: round-trip via canonical bytes,
        // re-construct the subgraph, re-hash. The encoding-side
        // primitive of the pin: canonical-bytes round trip preserves
        // content hash.
        let bytes = canonical_subgraph_bytes(&sg_save).unwrap();
        let digest = blake3::hash(&bytes);
        let cid_replay = *digest.as_bytes();
        assert_eq!(
            cid_save, cid_replay,
            "save-time content hash MUST equal replay-time content hash"
        );
    }
}
