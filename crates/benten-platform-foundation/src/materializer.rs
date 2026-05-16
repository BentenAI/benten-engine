//! Phase 4-Foundation G23-B — Materializer pipeline canary.
//!
//! ## Surface (per plan §3 G23-B + Ben D-4F-2 + D-4F-4 + D-4F-11)
//!
//! 1. **[`Materializer`] trait** — walks a [`SchemaSubgraphSpec`] emitted
//!    by [`crate::schema_compiler::compile`] under a supplied walk-principal +
//!    optional per-row cap gate. The walk routes EVERY READ through
//!    `benten_engine::Engine::read_node_as` (CLAUDE.md baked-in #18 Class
//!    B β). NEVER through `read_node` — that surface is `pub(crate)` for
//!    engine internals; the materializer is OUT-OF-CRATE.
//!
//! 2. **[`HtmlJsonMaterializer`]** — default impl emitting HTML article
//!    bytes + JSON projection bytes. Output is deterministic across runs
//!    per mat-r1-3 (canonical-bytes-stable).
//!
//! 3. **[`PlaintextMaterializer`]** — 2nd impl, ratified D-4F-11 + per
//!    arch-r1-10 (1-impl trait can hide accidental coupling; ship a 2nd
//!    impl to empirically validate output-FORMAT pluggability). Emits
//!    plaintext `field: value` lines; produces NO HTML tags.
//!
//! 4. **[`Renderer`] trait** — transport-agnostic surface per arch-r1-16
//!    NEW sub-section. Concrete `BrowserRender` default impl lives here.
//!    `TauriRenderer` lives in the sibling crate `benten-renderer-tauri` per
//!    G24-E. Future `tauri-runtime-verso` swap-readiness preserved by
//!    keeping transport concerns inside concrete impls (per br-r1-9).
//!
//! 5. **Dual-gate composition** (sec-3.5-r1-1) — the per-row gate at
//!    materialization SHARES `IvmViewReadGate` machinery per D-4F-NEW-
//!    MATERIALIZER-READ-GATE = SHARE (mat-r1-5). Materializer-view IS IVM
//!    view per D-4F-2. The delivery-layer (G14-D `on_change_as_with_cursor`)
//!    composes with the per-row gate; deny-from-either-layer wins per
//!    cap-r4-3.
//!
//! 6. **Reactive subscribe seam** — `subscribe_with_gate` attaches to
//!    `Engine::on_change_as_with_cursor` ONLY (sec-3.5-r1-9). Bare
//!    `Engine::on_change` is NEVER called from this module; pinned by
//!    `tests/materializer_pipeline_reactive_update_propagates_through_subscribe_seam.rs`
//!    grep-arm.
//!
//! 7. **Wallclock fail-closed inheritance** (sec-3.5-r1-7) — the walk
//!    inherits the engine's `UcanClockNotInjected` posture; constructing a
//!    materializer over an engine without injected clock and walking a
//!    time-bounded UCAN chain surfaces `E_UCAN_CLOCK_NOT_INJECTED` AT THE
//!    walk boundary (the materializer does not stamp `now()` itself).
//!
//! 8. **12-primitive irreducibility** (CLAUDE.md baked-in #1) — the walk
//!    dispatches ONLY existing `benten_core::PrimitiveKind` variants. The
//!    grep + runtime-trace pair pins this in
//!    `tests/materializer_walks_only_existing_12_primitives_no_extension.rs`.
//!
//! 9. **SANDBOX host-fn rejection** (sec-3.5-r1-14 + CLAUDE.md #16) — the
//!    materializer entry-point refuses any spec whose SANDBOX module
//!    references a storage-mutating host-fn (`kv:write` / `kv:delete` /
//!    edge-mutating). Surfaced as `E_MATERIALIZER_SCHEMA_MISMATCH` — the
//!    schema-compile path catches this upstream via
//!    `E_SCHEMA_SANDBOX_HOST_FN_REJECTED`; the materializer-side check is
//!    a defense-in-depth refusal for specs that bypass the schema-compile
//!    surface (e.g. hand-authored SubgraphSpec inputs).
//!
//! ## Dep direction (arch-r1-1 + arch-r1-15)
//!
//! `benten-platform-foundation` does NOT depend on `benten-engine` /
//! `benten-eval` / `benten-graph` in production. `benten-engine` is a
//! dev-dep only. To avoid a production cycle, the materializer surface is
//! parameterised over a trait [`MaterializerEngine`] that the test crate
//! adapts to `benten_engine::Engine`. Production callers (the admin UI v0
//! shell at G24-A) plug their `&Engine` into the same adapter at the
//! consumer boundary.
//!
//! This shape is the SAME pattern as `benten_caps::CapabilityPolicy` — the
//! engine plugs the trait at the boundary; the implementing crate doesn't
//! reach back into engine internals.
//!
//! ## Cap-scope mismatch defense (T1)
//!
//! The materializer's `materialize_with_gate` entry validates the spec's
//! emitted cap-scope envelope against the schema's declared `requires`. A
//! spec whose runtime composition exceeds the declared envelope is
//! REJECTED with `E_MATERIALIZER_SCHEMA_MISMATCH` BEFORE any READ fanout.
//! Negative pin at
//! `tests/materializer_rejects_subgraph_with_cap_scope_mismatch.rs`.

#![allow(
    clippy::module_name_repetitions,
    clippy::format_push_string,
    clippy::map_unwrap_or,
    clippy::collapsible_if,
    clippy::if_not_else,
    missing_docs
)]

use benten_core::{Cid, Node, PrimitiveKind, Value};
use benten_errors::ErrorCode;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;
use thiserror::Error;

use crate::schema_compiler::{CAP_SCOPE_PROPERTY_KEY, FIELD_PATH_PROPERTY_KEY, SchemaSubgraphSpec};

// ---------------------------------------------------------------------
// MaterializerEngine — the engine-side seam.
// ---------------------------------------------------------------------

/// Engine-side seam the materializer uses to fetch content.
///
/// This trait MUST be implemented by callers (typically as a thin adapter
/// around `benten_engine::Engine`). It ROUTES the materializer's READ
/// fanout through `Engine::read_node_as` per CLAUDE.md baked-in #18 Class
/// B β. Implementations that fan out to `Engine::read_node` BYPASS the
/// cap-recheck boundary and are a regression of cag-r1-9.
///
/// **Default adapter:** the dev-dep adapter in this crate's test tree
/// supplies an `EngineAdapter` that delegates to
/// `benten_engine::Engine::read_node_as`. The same adapter shape will be
/// embedded in the admin UI v0 shell at G24-A.
pub trait MaterializerEngine {
    /// Read `cid` attributed to `principal`. MUST route through the
    /// cap-rechecking entry point (i.e. `read_node_as` on the real
    /// engine), NEVER through the engine-internal `read_node`.
    ///
    /// Returns:
    /// - `Ok(Some(node))` — admitted by the engine cap-policy + Inv-11.
    /// - `Ok(None)` — cap-denied OR system-zone OR backend-miss
    ///   (symmetric None per Option C; CLAUDE.md compromise #2).
    /// - `Err(...)` — backend / engine-internal failure.
    ///
    /// # Errors
    /// Implementation-defined backend failure.
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError>;

    /// Whether the engine has had a clock injected. The walk inherits the
    /// fail-closed posture per sec-3.5-r1-7: when the policy is
    /// time-bounded but no clock is injected, the walk surfaces
    /// `E_UCAN_CLOCK_NOT_INJECTED` (NOT a silent `now()` default).
    ///
    /// Default: `true` for tests / `NoAuthBackend`. Real engine adapters
    /// return whether `Engine::open_with_clock` was used.
    fn has_clock_injected(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------
// Per-row cap gate — SHARES IvmViewReadGate machinery semantically.
// ---------------------------------------------------------------------

/// Per-row cap-recheck closure — same shape as
/// `benten_engine::cap_recheck::CapRecheckFn`. Materializer-view IS IVM
/// view per D-4F-2; we reuse the cap-recheck shape (the actual
/// `IvmViewReadGate` type lives in `benten-engine` to preserve the
/// dependency direction).
///
/// Arguments:
/// - actor principal CID (the walk-principal).
/// - zone label hint (typically the schema name; passed through opaque).
/// - candidate row/node CID.
///
/// Returns `true` to admit, `false` to deny.
pub type MaterializerCapRecheck = Arc<dyn Fn(&Cid, &str, &Cid) -> bool + Send + Sync + 'static>;

/// Build an "allow-all" cap-recheck closure (used by `NoAuthBackend` /
/// tests).
#[must_use]
pub fn allow_all_cap_recheck() -> MaterializerCapRecheck {
    Arc::new(|_p: &Cid, _z: &str, _c: &Cid| true)
}

/// Build a "deny-all" cap-recheck closure.
#[must_use]
pub fn deny_all_cap_recheck() -> MaterializerCapRecheck {
    Arc::new(|_p: &Cid, _z: &str, _c: &Cid| false)
}

// ---------------------------------------------------------------------
// MaterializerError + cap-denial frame.
// ---------------------------------------------------------------------

/// Error type for the materializer walk surface.
///
/// Each variant's identity uniquely determines its typed [`ErrorCode`]
/// (surfaced via [`MaterializerError::code`]) — there is no per-variant
/// `code` field, since the variant IS the code (Qual-1 #732: a redundant
/// `code: ErrorCode` field that is structurally constant per variant is
/// duplicate state).
#[derive(Debug, Error)]
pub enum MaterializerError {
    /// Materializer's entry validation refused the spec.
    /// Surfaces [`ErrorCode::MaterializerSchemaMismatch`].
    #[error("materializer rejected SubgraphSpec at entry: {reason}")]
    SchemaMismatch {
        /// Diagnostic.
        reason: String,
    },

    /// Materializer's reactive subscribe seam failed to attach.
    /// Surfaces [`ErrorCode::MaterializerSubscribeSeamFailure`].
    #[error("materializer subscribe seam failed: pattern={pattern} reason={reason}")]
    SubscribeSeamFailure {
        /// The pattern that was being attached.
        pattern: String,
        /// Diagnostic.
        reason: String,
    },

    /// UCAN clock-not-injected inheritance per sec-3.5-r1-7.
    /// Surfaces [`ErrorCode::UcanClockNotInjected`].
    #[error("UCAN chain-walker invoked without clock injection (E_UCAN_CLOCK_NOT_INJECTED)")]
    UcanClockNotInjected,
}

impl MaterializerError {
    /// Return the typed [`ErrorCode`] this error surfaces. The variant
    /// identity alone determines the code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            MaterializerError::SchemaMismatch { .. } => ErrorCode::MaterializerSchemaMismatch,
            MaterializerError::SubscribeSeamFailure { .. } => {
                ErrorCode::MaterializerSubscribeSeamFailure
            }
            MaterializerError::UcanClockNotInjected => ErrorCode::UcanClockNotInjected,
        }
    }
}

/// Per-Node cap-denial frame surfaced in the materializer output.
///
/// When the per-row gate denies a Node during the walk, the materializer
/// returns Ok(out) (NOT Err — per ratification #7 redacted-view shape).
/// The denied Node's content is replaced by a placeholder in the output
/// bytes; this frame carries the typed code so the consumer (admin UI) can
/// render an explanation.
#[derive(Debug, Clone)]
pub struct MaterializerDenialFrame {
    /// The CID that was denied.
    pub node_cid: Cid,
    /// The walk-principal under which the denial happened.
    pub principal_cid: Cid,
    /// The cap-scope (if any) the materializer was checking.
    pub scope: Option<String>,
    /// Always [`ErrorCode::MaterializerCapDenied`].
    pub code_value: ErrorCode,
}

impl MaterializerDenialFrame {
    /// Return the typed error code carried by this denial frame.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code_value.clone()
    }
}

// ---------------------------------------------------------------------
// MaterializerWalkInputs — the bundle the trait consumes.
// ---------------------------------------------------------------------

/// Inputs to a single materializer walk.
///
/// **View identity (mat-r1-11):** the `(spec_cid, content_cid)` pair
/// uniquely identifies a materializer view. Two walks carrying the
/// SAME `(spec, content_cid)` produce the SAME canonical bytes (the
/// determinism test pins this property) — i.e., "one view per
/// schema-content-pair" per D-4F-2. Consumers that want multiple
/// views over the same content tile should pass distinct
/// `SchemaSubgraphSpec` values; consumers that want multiple views
/// over the same shape should pass distinct content CIDs.
pub struct MaterializerWalkInputs<'a, E: MaterializerEngine> {
    /// Engine seam used for content reads (`read_node_as`).
    pub engine: &'a E,
    /// Schema-emitted SubgraphSpec being walked.
    pub spec: &'a SchemaSubgraphSpec,
    /// The single content Node CID to render (post-`Engine::put_node`).
    ///
    /// The materializer reads this CID via `read_node_as(walk_principal,
    /// content_cid)` and renders the Node's property-bag against the
    /// schema-emitted field primitives.
    pub content_cid: Cid,
    /// The walk-principal (Class B β attribution).
    pub walk_principal: Cid,
    /// Per-row cap-recheck closure (the materialization-layer gate).
    pub cap_recheck: MaterializerCapRecheck,
    /// Declared `requires` envelope from the manifest / caller. The
    /// materializer rejects the spec at entry if any emitted primitive's
    /// cap-scope falls OUTSIDE this envelope per T1 (sec-r4-3).
    ///
    /// Empty Vec = allow-all (no T1 envelope check — used by tests that
    /// don't exercise the T1 arm).
    pub declared_requires: Vec<String>,
}

impl<'a, E: MaterializerEngine> Clone for MaterializerWalkInputs<'a, E> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine,
            spec: self.spec,
            content_cid: self.content_cid,
            walk_principal: self.walk_principal,
            cap_recheck: Arc::clone(&self.cap_recheck),
            declared_requires: self.declared_requires.clone(),
        }
    }
}

// ---------------------------------------------------------------------
// MaterializerOutput — what the trait emits.
// ---------------------------------------------------------------------

/// Output bytes from a single materializer walk.
#[derive(Debug, Clone)]
pub struct MaterializerOutput {
    /// Primary-format bytes (HTML for HtmlJson; plaintext for Plaintext).
    primary: Vec<u8>,
    /// Optional secondary-format bytes (JSON for HtmlJson; empty for
    /// Plaintext).
    secondary: Vec<u8>,
    /// Cap-denial frames captured during the walk; each entry is a Node
    /// the per-row gate denied. Output bytes carry `[redacted]`
    /// placeholders for these Nodes.
    denials: Vec<MaterializerDenialFrame>,
    /// CIDs that the walk successfully materialised (post-gate-admission).
    materialized_cids: Vec<Cid>,
    /// Distinct `PrimitiveKind` variants the walk dispatched through.
    /// Surfaced for the 12-primitive-irreducibility runtime-trace pin.
    dispatched_kinds: HashSet<PrimitiveKind>,
    /// SubgraphSpec CID — for content-addressing of the output.
    spec_cid: Option<Cid>,
}

impl MaterializerOutput {
    /// Primary bytes (HTML / plaintext / etc).
    #[must_use]
    pub fn primary_bytes(&self) -> &[u8] {
        &self.primary
    }

    /// Convenience for HtmlJson — the HTML side.
    #[must_use]
    pub fn html_bytes(&self) -> &[u8] {
        &self.primary
    }

    /// Convenience for HtmlJson — the JSON projection side.
    #[must_use]
    pub fn json_bytes(&self) -> &[u8] {
        &self.secondary
    }

    /// Cap-denial frames captured during the walk.
    #[must_use]
    pub fn cap_denials(&self) -> &[MaterializerDenialFrame] {
        &self.denials
    }

    /// CIDs the walk admitted past the per-row gate. The v1 walk is
    /// single-row (`content_cid` is singular), so this holds at most
    /// one element — Phase-4-Meta IVM-view materialization (one view =
    /// N rows) is where a true multi-row return materializes.
    #[must_use]
    pub fn materialized_row_cids(&self) -> &[Cid] {
        &self.materialized_cids
    }

    /// `PrimitiveKind` variants the walk dispatched through.
    #[must_use]
    pub fn dispatched_primitive_kinds(&self) -> &HashSet<PrimitiveKind> {
        &self.dispatched_kinds
    }

    /// Stable content-addressed CID over the canonical output bytes.
    /// Used by the determinism pin (mat-r1-3).
    #[must_use]
    pub fn canonical_cid(&self) -> Cid {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.primary);
        hasher.update(&[0xff]);
        hasher.update(&self.secondary);
        let digest = hasher.finalize();
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    /// Spec CID associated with this output (the schema's emitted
    /// Subgraph CID — useful for cache invalidation).
    #[must_use]
    pub fn spec_cid(&self) -> Option<Cid> {
        self.spec_cid
    }
}

// ---------------------------------------------------------------------
// Materializer trait.
// ---------------------------------------------------------------------

/// Materializer trait — walks a [`SchemaSubgraphSpec`] under a walk-
/// principal + per-row gate, producing output bytes.
///
/// **Trait abstraction is INDEPENDENT of output format.** Two impls (HTML+JSON
/// and Plaintext) empirically validate per arch-r1-10 + cag-r1-6 that the
/// trait is not accidentally HtmlJson-specific (pinned by
/// `tests/materializer_output_backend_pluggable_two_impls_compile_and_round_trip.rs`).
pub trait Materializer: Send + Sync {
    /// Walk the spec + emit output bytes.
    ///
    /// # Errors
    /// Returns [`MaterializerError::SchemaMismatch`] if the spec's
    /// runtime cap-scope envelope exceeds the declared `requires`.
    /// Returns [`MaterializerError::UcanClockNotInjected`] when the
    /// engine has not had a clock injected.
    fn materialize_with_gate<E: MaterializerEngine>(
        &self,
        inputs: MaterializerWalkInputs<'_, E>,
    ) -> Result<MaterializerOutput, MaterializerError>;

    /// Filter row CIDs at the materialization-layer per-row gate.
    /// Mirrors `IvmViewReadGate::filter_rows`; used by the
    /// per-row-independent-of-delivery pin.
    #[must_use]
    fn filter_rows_at_materialization(
        &self,
        rows: Vec<Cid>,
        principal: &Cid,
        zone: &str,
        recheck: &MaterializerCapRecheck,
    ) -> Vec<Cid> {
        rows.into_iter()
            .filter(|cid| (recheck)(principal, zone, cid))
            .collect()
    }

    /// Dual-gate composition — applies mat-layer gate AND delivery-layer
    /// gate; deny-from-either-layer wins (cap-r4-3).
    #[must_use]
    fn dual_gate_admits(
        &self,
        cid: &Cid,
        principal: &Cid,
        zone: &str,
        mat_gate: &MaterializerCapRecheck,
        delivery_gate: &MaterializerCapRecheck,
    ) -> bool {
        (mat_gate)(principal, zone, cid) && (delivery_gate)(principal, zone, cid)
    }
}

// ---------------------------------------------------------------------
// HtmlJsonMaterializer (default impl).
// ---------------------------------------------------------------------

/// Default materializer impl emitting HTML article bytes + JSON
/// projection bytes. Output is deterministic per mat-r1-3.
#[derive(Debug, Default, Clone)]
pub struct HtmlJsonMaterializer;

impl HtmlJsonMaterializer {
    /// Attach a reactive subscribe seam through
    /// `Engine::on_change_as_with_cursor`. Pattern + cursor are
    /// transport-agnostic; the engine adapter routes to the real
    /// engine surface (NEVER `on_change`).
    ///
    /// # Errors
    /// Returns [`MaterializerError::SubscribeSeamFailure`] if the pattern
    /// is empty (matches `Engine::on_change_as_with_cursor` pattern-
    /// invalid guard).
    pub fn subscribe_with_gate(
        &self,
        pattern: &str,
    ) -> Result<SubscribeAttachToken, MaterializerError> {
        // The materializer routes ONLY through on_change_as_with_cursor
        // (not the bare unauthenticated cursor) per sec-3.5-r1-9. The
        // attach call site here is the seam that consumers wire to the
        // real engine; the grep-assert pin verifies zero bare on_change
        // call sites at the engine surface mentioned in this file.
        //
        // NOTE: the actual `engine.on_change_as_with_cursor(...)` call
        // happens at the consumer boundary (`SubscribeAttachToken`
        // routes to the engine surface), to preserve the dep direction
        // commitment. The trait surface here pins the seam shape.
        if pattern.is_empty() {
            return Err(MaterializerError::SubscribeSeamFailure {
                pattern: String::new(),
                reason: "pattern must be a non-empty event-name glob".into(),
            });
        }
        Ok(SubscribeAttachToken {
            pattern: pattern.to_string(),
        })
    }
}

impl Materializer for HtmlJsonMaterializer {
    fn materialize_with_gate<E: MaterializerEngine>(
        &self,
        inputs: MaterializerWalkInputs<'_, E>,
    ) -> Result<MaterializerOutput, MaterializerError> {
        materialize_html_json(inputs)
    }
}

// ---------------------------------------------------------------------
// PlaintextMaterializer (arch-r1-10 + D-4F-11 pluggability validation).
// ---------------------------------------------------------------------

/// 2nd materializer impl — plaintext output. Per arch-r1-10 + cag-r1-6
/// the existence of a 2nd impl empirically validates output-FORMAT
/// pluggability: this impl produces NO HTML tags, proving the trait is
/// not accidentally HtmlJson-specific.
#[derive(Debug, Default, Clone)]
pub struct PlaintextMaterializer;

impl Materializer for PlaintextMaterializer {
    fn materialize_with_gate<E: MaterializerEngine>(
        &self,
        inputs: MaterializerWalkInputs<'_, E>,
    ) -> Result<MaterializerOutput, MaterializerError> {
        materialize_plaintext(inputs)
    }
}

// ---------------------------------------------------------------------
// Renderer trait abstraction (arch-r1-16 NEW sub-section).
// ---------------------------------------------------------------------

/// Transport-agnostic renderer trait per arch-r1-16. Concrete impls
/// carry transport concerns (browser-wasm32 fetch, Tauri IPC, etc.); the
/// trait surface does NOT name any transport.
///
/// `tauri-runtime-verso` swap-readiness preserved (br-r1-9): swap targets
/// implement the same trait against the same MaterializerOutput shape;
/// trait surface DOES NOT contain transport-specific methods.
pub trait Renderer: Send + Sync {
    /// Render a materializer output into the renderer's transport.
    ///
    /// # Errors
    /// Implementation-defined transport failure.
    fn render(&self, output: &MaterializerOutput) -> Result<(), RenderError>;

    /// Renderer identity tag — used in tests + diagnostics to confirm
    /// which backend is wired (BrowserRender / TauriRenderer / etc.).
    fn backend_name(&self) -> &'static str;
}

/// Renderer error type — opaque to keep transport concerns inside
/// concrete impls.
#[derive(Debug, Error)]
pub enum RenderError {
    /// Renderer transport failure.
    #[error("renderer transport failure: {0}")]
    Transport(String),
}

/// Default `Renderer` impl for the browser-wasm32 shape (b) deployment.
///
/// Per CLAUDE.md #17 deployment-shape (b): browser tab loads the
/// wasm32-unknown-unknown bundle; reads-against-snapshot; writes via
/// fetch to a full peer. This default impl is a no-op stub at G23-B; the
/// admin UI v0 shell at G24-A fills the DOM render path. Used here to
/// validate the trait is pluggable + the swap-target shape (TauriRenderer
/// in `benten-renderer-tauri`) compiles against the same surface.
#[derive(Debug, Default, Clone)]
pub struct BrowserRender;

impl Renderer for BrowserRender {
    fn render(&self, _output: &MaterializerOutput) -> Result<(), RenderError> {
        // G23-B stub. The admin UI v0 shell at G24-A fills the DOM-mount
        // logic; this default impl satisfies trait coherence + the
        // arch-r1-16 doc-test surface assertion (no transport methods).
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "browser-wasm32"
    }
}

// ---------------------------------------------------------------------
// Internal walk machinery.
// ---------------------------------------------------------------------

/// Format selector — internal.
#[derive(Debug, Clone, Copy)]
enum FormatBackend {
    HtmlJson,
    Plaintext,
}

/// Subscribe seam attach token. The actual `on_change_as_with_cursor`
/// call happens at the consumer boundary (admin UI v0 shell at G24-A)
/// using this token's pattern; the materializer-side seam is the trait
/// surface lock.
#[derive(Debug, Clone)]
pub struct SubscribeAttachToken {
    /// Pattern to be subscribed against; consumer passes this to
    /// `Engine::on_change_as_with_cursor(pattern, cursor, callback, actor)`.
    pub pattern: String,
}

fn extract_first_cap_scope(spec: &SchemaSubgraphSpec) -> Option<String> {
    spec.primitives()
        .iter()
        .find_map(|p| p.cap_scope().map(str::to_string))
}

// ---------------------------------------------------------------------
// Format-specific render functions (called by trait impls).
// ---------------------------------------------------------------------

impl HtmlJsonMaterializer {
    /// Render an admitted Node to HTML + JSON projection bytes.
    fn render_html_json(spec: &SchemaSubgraphSpec, node: &Node) -> (Vec<u8>, Vec<u8>) {
        let schema_class = spec.schema_name().to_ascii_lowercase();
        let mut html = String::new();
        let _ = write!(html, "<article class=\"benten-{schema_class}\">");
        // Walk emitted READ primitives in stable order. For each one
        // whose `field_path` resolves to a Node property, render a
        // field div.
        for op in spec.as_subgraph().nodes() {
            if op.kind != PrimitiveKind::Read {
                continue;
            }
            let Some(field_path) = op.property(FIELD_PATH_PROPERTY_KEY) else {
                continue;
            };
            let Value::Text(field_name) = field_path else {
                continue;
            };
            // schema_compiler emits field_path as "SchemaName.field"
            // — extract the trailing component to look up on the Node.
            let field_key = field_name
                .rsplit_once('.')
                .map_or(field_name.as_str(), |(_, f)| f);
            if let Some(val) = node.properties.get(field_key) {
                let rendered = render_value_html(val);
                let _ = write!(
                    html,
                    "<div class=\"benten-field-{field_key}\">{rendered}</div>"
                );
            }
        }
        html.push_str("</article>");
        let json = json_projection_for_node(spec, node);
        (html.into_bytes(), json.into_bytes())
    }
}

impl PlaintextMaterializer {
    /// Render an admitted Node to plaintext bytes (one field per line).
    fn render_plaintext(spec: &SchemaSubgraphSpec, node: &Node) -> Vec<u8> {
        let mut out = String::new();
        for op in spec.as_subgraph().nodes() {
            if op.kind != PrimitiveKind::Read {
                continue;
            }
            let Some(Value::Text(field_path)) = op.property(FIELD_PATH_PROPERTY_KEY) else {
                continue;
            };
            let field_key = field_path
                .rsplit_once('.')
                .map_or(field_path.as_str(), |(_, f)| f);
            if let Some(val) = node.properties.get(field_key) {
                let rendered = render_value_plaintext(val);
                let _ = writeln!(out, "{field_key}: {rendered}");
            }
        }
        out.into_bytes()
    }
}

/// Per-format leaf-rendering rules. The recursive [`Value`]-tree walk
/// (List / Map descent) is shared across HTML / plaintext / JSON via
/// [`render_value`] — only the per-variant leaf rules + container
/// composition differ (Qual-1 #730: 4 near-identical Value walkers
/// collapsed to one walker + three rule impls). A future Phase-4-Meta
/// `Value` scalar mint (e.g. `Decimal` / `Timestamp`) touches exactly
/// one place (the `render_value` dispatch + each rule's new arm via the
/// trait), not 3-4 duplicated dispatch sites.
trait ValueRender {
    fn text(&self, s: &str) -> String;
    fn int(&self, i: i64) -> String {
        i.to_string()
    }
    fn float(&self, f: f64) -> String {
        f.to_string()
    }
    fn boolean(&self, b: bool) -> String {
        b.to_string()
    }
    fn null(&self) -> String;
    fn bytes(&self, len: usize) -> String;
    /// Join already-rendered list items into the list representation.
    fn list(&self, items: &[String]) -> String;
    /// Compose an already-rendered map of `(key, rendered_value)` pairs.
    fn map(&self, pairs: &[(String, String)]) -> String;
}

/// Shared recursive [`Value`]-tree walk. Leaf + container rules come
/// from `R`; the List/Map descent structure is written once here.
fn render_value<R: ValueRender>(v: &Value, r: &R) -> String {
    match v {
        Value::Text(s) => r.text(s),
        Value::Int(i) => r.int(*i),
        Value::Float(f) => r.float(*f),
        Value::Bool(b) => r.boolean(*b),
        Value::Null => r.null(),
        Value::Bytes(b) => r.bytes(b.len()),
        Value::List(l) => {
            let items: Vec<String> = l.iter().map(|x| render_value(x, r)).collect();
            r.list(&items)
        }
        Value::Map(m) => {
            let pairs: Vec<(String, String)> = m
                .iter()
                .map(|(k, vv)| (k.clone(), render_value(vv, r)))
                .collect();
            r.map(&pairs)
        }
    }
}

struct HtmlRender;
impl ValueRender for HtmlRender {
    fn text(&self, s: &str) -> String {
        html_escape(s)
    }
    fn null(&self) -> String {
        String::new()
    }
    fn bytes(&self, len: usize) -> String {
        format!("[bytes:{len}]")
    }
    fn list(&self, items: &[String]) -> String {
        items.join(", ")
    }
    fn map(&self, pairs: &[(String, String)]) -> String {
        let mut out = String::new();
        for (k, v) in pairs {
            let _ = write!(out, "{k}={v}");
        }
        out
    }
}

struct PlaintextRender;
impl ValueRender for PlaintextRender {
    fn text(&self, s: &str) -> String {
        s.to_string()
    }
    fn null(&self) -> String {
        "null".into()
    }
    fn bytes(&self, len: usize) -> String {
        format!("[bytes:{len}]")
    }
    fn list(&self, items: &[String]) -> String {
        items.join(", ")
    }
    fn map(&self, pairs: &[(String, String)]) -> String {
        let mut out = String::new();
        for (k, v) in pairs {
            let _ = write!(out, "{k}={v}");
        }
        out
    }
}

struct JsonRender;
impl ValueRender for JsonRender {
    fn text(&self, s: &str) -> String {
        format!("\"{}\"", json_escape(s))
    }
    fn null(&self) -> String {
        "null".into()
    }
    fn bytes(&self, len: usize) -> String {
        format!("\"[bytes:{len}]\"")
    }
    fn list(&self, items: &[String]) -> String {
        format!("[{}]", items.join(","))
    }
    fn map(&self, pairs: &[(String, String)]) -> String {
        let body: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
        format!("{{{}}}", body.join(","))
    }
}

fn render_value_html(v: &Value) -> String {
    render_value(v, &HtmlRender)
}

fn render_value_plaintext(v: &Value) -> String {
    render_value(v, &PlaintextRender)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// JSON projection for an admitted Node — emits a canonical object whose
/// keys are the schema-emitted field names + a `scope` array carrying the
/// schema-derived cap-scopes per sec-3.5-r1-4.
fn json_projection_for_node(spec: &SchemaSubgraphSpec, node: &Node) -> String {
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    let mut scopes: BTreeSet<String> = BTreeSet::new();
    for op in spec.as_subgraph().nodes() {
        if op.kind != PrimitiveKind::Read {
            continue;
        }
        let Some(Value::Text(field_path)) = op.property(FIELD_PATH_PROPERTY_KEY) else {
            continue;
        };
        let field_key = field_path
            .rsplit_once('.')
            .map(|(_, f)| f.to_string())
            .unwrap_or_else(|| field_path.clone());
        if let Some(v) = node.properties.get(&field_key) {
            fields.insert(field_key.clone(), value_to_json(v));
        }
        if let Some(Value::Text(scope)) = op.property(CAP_SCOPE_PROPERTY_KEY) {
            // Phase-4-Foundation: surfaced as lowercased schema name per
            // canonical-projection determinism.
            scopes.insert(scope.to_ascii_lowercase());
        }
    }
    let mut out = String::from("{");
    let mut first = true;
    for (k, v) in &fields {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&format!("\"{k}\":{v}"));
    }
    // Emit `"scope":[...]` — sorted, stable, schema-derived.
    if !scopes.is_empty() {
        if !first {
            out.push(',');
        }
        let scope_list: Vec<String> = scopes.iter().map(|s| format!("\"{s}\"")).collect();
        out.push_str(&format!("\"scope\":[{}]", scope_list.join(",")));
    }
    out.push('}');
    out
}

/// JSON projection when the Node was denied — emit `null` for each
/// field + the scope array (so consumers can render an explanation
/// with the would-have-been-checked scopes).
fn json_projection_redacted(spec: &SchemaSubgraphSpec) -> String {
    let mut fields: Vec<String> = Vec::new();
    let mut scopes: BTreeSet<String> = BTreeSet::new();
    for op in spec.as_subgraph().nodes() {
        if op.kind != PrimitiveKind::Read {
            continue;
        }
        let Some(Value::Text(field_path)) = op.property(FIELD_PATH_PROPERTY_KEY) else {
            continue;
        };
        let field_key = field_path
            .rsplit_once('.')
            .map(|(_, f)| f.to_string())
            .unwrap_or_else(|| field_path.clone());
        fields.push(format!("\"{field_key}\":null"));
        if let Some(Value::Text(scope)) = op.property(CAP_SCOPE_PROPERTY_KEY) {
            scopes.insert(scope.to_ascii_lowercase());
        }
    }
    let mut out = String::from("{");
    out.push_str(&fields.join(","));
    if !scopes.is_empty() {
        if !fields.is_empty() {
            out.push(',');
        }
        let scope_list: Vec<String> = scopes.iter().map(|s| format!("\"{s}\"")).collect();
        out.push_str(&format!("\"scope\":[{}]", scope_list.join(",")));
        out.push_str(",\"redacted\":true");
    } else {
        if !fields.is_empty() {
            out.push(',');
        }
        out.push_str("\"redacted\":true");
    }
    out.push('}');
    out
}

fn value_to_json(v: &Value) -> String {
    render_value(v, &JsonRender)
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------
// Re-export the trait-impl format-specific renderers via the impl-by-
// dispatch shape used in `materialize_common`. Re-implement the
// `materialize_common` path to call the format-specific renderer.
// ---------------------------------------------------------------------

#[doc(hidden)]
pub fn materialize_html_json<E: MaterializerEngine>(
    inputs: MaterializerWalkInputs<'_, E>,
) -> Result<MaterializerOutput, MaterializerError> {
    materialize_format(inputs, FormatBackend::HtmlJson)
}

#[doc(hidden)]
pub fn materialize_plaintext<E: MaterializerEngine>(
    inputs: MaterializerWalkInputs<'_, E>,
) -> Result<MaterializerOutput, MaterializerError> {
    materialize_format(inputs, FormatBackend::Plaintext)
}

#[allow(
    clippy::too_many_lines,
    reason = "single-source-of-truth walk dispatch"
)]
fn materialize_format<E: MaterializerEngine>(
    inputs: MaterializerWalkInputs<'_, E>,
    fmt: FormatBackend,
) -> Result<MaterializerOutput, MaterializerError> {
    // (1) Clock fail-closed.
    if !inputs.engine.has_clock_injected() {
        return Err(MaterializerError::UcanClockNotInjected);
    }
    // (2) T1 envelope.
    if !inputs.declared_requires.is_empty() {
        let declared: BTreeSet<&str> = inputs
            .declared_requires
            .iter()
            .map(String::as_str)
            .collect();
        for op in inputs.spec.as_subgraph().nodes() {
            if let Some(Value::Text(scope)) = op.property(CAP_SCOPE_PROPERTY_KEY) {
                if !declared.contains(scope.as_str()) {
                    return Err(MaterializerError::SchemaMismatch {
                        reason: format!(
                            "primitive `{}` requires cap-scope `{}` outside declared envelope ({:?})",
                            op.id, scope, inputs.declared_requires
                        ),
                    });
                }
            }
        }
    }
    // (3) SANDBOX defense.
    for op in inputs.spec.as_subgraph().nodes() {
        if matches!(op.kind, PrimitiveKind::Sandbox) {
            if let Some(Value::Text(host_fn)) = op.property("sandbox_host_fn") {
                let banned = ["kv:write", "kv:delete", "edges:add", "edges:remove"];
                if banned.iter().any(|b| host_fn == b) {
                    return Err(MaterializerError::SchemaMismatch {
                        reason: format!(
                            "SANDBOX primitive `{}` requests storage-mutating host-fn `{}` — forbidden per CLAUDE.md baked-in #16",
                            op.id, host_fn
                        ),
                    });
                }
            }
        }
    }
    // (4) Materialization-layer per-row gate.
    //
    // Per-primitive cap-scope enforcement is performed UPSTREAM of this
    // walk: the T1 envelope check at step (2) above rejects any emitted
    // primitive whose `CAP_SCOPE_PROPERTY_KEY` falls outside the
    // declared `requires` envelope BEFORE any READ fanout, and the
    // schema-compile / workflow-editor save path
    // (`schema_compiler::derive_scope` + `validate_subgraph_within_
    // manifest_envelope`) derives + bounds per-primitive scopes before
    // a SubgraphSpec ever reaches the materializer. The previous
    // per-primitive fan-out loop discarded both the read scope and the
    // gate bool (Qual-1 #702 / Safe-1 #527 — "observability-theater
    // discarding a security-shaped bool"); it provided no production
    // enforcement and no production observability and is removed.
    //
    // The authoritative materialization-layer cap-decision for the
    // content CID is this single gate call — its bool is consumed (NOT
    // discarded); a deny collapses the walk to the redacted view below.
    let zone_hint = inputs.spec.schema_name();
    let admitted_by_gate =
        (inputs.cap_recheck)(&inputs.walk_principal, zone_hint, &inputs.content_cid);
    // (5) Engine read via read_node_as. NEVER read_node.
    let node_opt = inputs
        .engine
        .read_node_as(&inputs.walk_principal, &inputs.content_cid)?;

    let mut dispatched_kinds = HashSet::new();
    for op in inputs.spec.as_subgraph().nodes() {
        dispatched_kinds.insert(op.kind);
    }
    let mut denials = Vec::new();
    let mut materialized_cids = Vec::new();

    let node_value = if admitted_by_gate {
        if node_opt.is_some() {
            materialized_cids.push(inputs.content_cid);
            node_opt
        } else {
            // Gate admitted but engine returned None — engine-side
            // denial (Option C) or backend miss; record as denial.
            denials.push(MaterializerDenialFrame {
                node_cid: inputs.content_cid,
                principal_cid: inputs.walk_principal,
                scope: extract_first_cap_scope(inputs.spec),
                code_value: ErrorCode::MaterializerCapDenied,
            });
            None
        }
    } else {
        denials.push(MaterializerDenialFrame {
            node_cid: inputs.content_cid,
            principal_cid: inputs.walk_principal,
            scope: extract_first_cap_scope(inputs.spec),
            code_value: ErrorCode::MaterializerCapDenied,
        });
        None
    };

    let (primary, secondary) = match (node_value, fmt) {
        (Some(node), FormatBackend::HtmlJson) => {
            HtmlJsonMaterializer::render_html_json(inputs.spec, &node)
        }
        (Some(node), FormatBackend::Plaintext) => (
            PlaintextMaterializer::render_plaintext(inputs.spec, &node),
            Vec::new(),
        ),
        (None, FormatBackend::HtmlJson) => {
            let schema_class = inputs.spec.schema_name().to_ascii_lowercase();
            (
                format!(
                    "<article class=\"benten-{schema_class}\"><div class=\"benten-field-body\">[redacted]</div></article>"
                )
                .into_bytes(),
                json_projection_redacted(inputs.spec).into_bytes(),
            )
        }
        (None, FormatBackend::Plaintext) => ("body: [redacted]\n".as_bytes().to_vec(), Vec::new()),
    };

    Ok(MaterializerOutput {
        primary,
        secondary,
        denials,
        materialized_cids,
        dispatched_kinds,
        spec_cid: inputs.spec.as_subgraph().cid().ok(),
    })
}

// ---------------------------------------------------------------------
// In-memory test engine (helper for both unit tests + integration pins).
// ---------------------------------------------------------------------

/// In-memory [`MaterializerEngine`] adapter used by integration pins.
///
/// Stores `Cid → Node` mappings + an "engine-side cap policy" that can
/// deny reads for unauthorized principals (mirrors `Engine::read_node_as`
/// Option-C symmetric-None semantics).
///
/// **Test/dev fixture — NOT a stable public API.** Marked `#[doc(hidden)]`
/// per G23-B mr-6. Production consumers should wire a real engine
/// adapter at the G24-A admin-UI integration boundary that bridges
/// `MaterializerEngine` to `Engine::read_node_as`.
#[doc(hidden)]
#[derive(Default)]
pub struct InMemoryMaterializerEngine {
    nodes: std::sync::RwLock<std::collections::HashMap<Cid, Node>>,
    denied_principals: std::sync::RwLock<BTreeSet<Cid>>,
    clock_injected: bool,
}

impl InMemoryMaterializerEngine {
    /// Construct a fresh in-memory engine adapter with clock injected
    /// (default — production tests inject; the negative wallclock pin
    /// uses [`Self::without_clock`]).
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            denied_principals: Default::default(),
            clock_injected: true,
        }
    }

    /// Construct an in-memory engine adapter with NO clock injected —
    /// used by the fail-closed wallclock pin.
    #[must_use]
    pub fn without_clock() -> Self {
        Self {
            nodes: Default::default(),
            denied_principals: Default::default(),
            clock_injected: false,
        }
    }

    /// Insert a Node — returns its CID.
    pub fn put_node(&self, node: Node) -> Cid {
        let cid = node.cid().expect("Node serializes");
        self.nodes.write().unwrap().insert(cid, node);
        cid
    }

    /// Deny a principal — engine-side will return Ok(None) for any
    /// `read_node_as(principal, _)` (mirrors Engine's Option-C
    /// symmetric-None for cap-denial).
    pub fn deny_principal(&self, principal: Cid) {
        self.denied_principals.write().unwrap().insert(principal);
    }
}

impl MaterializerEngine for InMemoryMaterializerEngine {
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError> {
        if self.denied_principals.read().unwrap().contains(principal) {
            return Ok(None);
        }
        Ok(self.nodes.read().unwrap().get(cid).cloned())
    }

    fn has_clock_injected(&self) -> bool {
        self.clock_injected
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod inline_canary {
    //! Inline canary tests that exercise the materializer surface
    //! against the canonical Note fixture. Full pins live at
    //! `crates/benten-platform-foundation/tests/materializer_*.rs`.

    use super::*;
    use crate::schema_compiler::compile;

    const CANONICAL_NOTE: &[u8] = br#"{
        "label": "SchemaRoot",
        "name": "Note",
        "fields": [
            { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null },
            { "label": "FieldScalar", "name": "created_at", "scalar": "timestamp-hlc", "required": true, "default": null }
        ]
    }"#;

    fn make_note(body: &str) -> Node {
        let mut props = BTreeMap::new();
        props.insert("body".into(), Value::Text(body.into()));
        props.insert(
            "created_at".into(),
            Value::Text("2026-05-13T00:00:00Z".into()),
        );
        Node::new(vec!["Note".to_string()], props)
    }

    fn principal_cid() -> Cid {
        let mut props = BTreeMap::new();
        props.insert("name".into(), Value::Text("alice".into()));
        let n = Node::new(vec!["actor".to_string()], props);
        n.cid().unwrap()
    }

    #[test]
    fn html_json_walk_renders_admitted_node() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("the body"));
        let alice = principal_cid();
        let inputs = MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        };
        let out = materialize_html_json(inputs).unwrap();
        let html = std::str::from_utf8(out.html_bytes()).unwrap();
        assert!(html.contains("the body"));
        assert!(html.contains("benten-note"));
        assert!(out.cap_denials().is_empty());
        assert_eq!(out.materialized_row_cids().len(), 1);
    }

    #[test]
    fn plaintext_walk_emits_no_html_tags() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("hello"));
        let alice = principal_cid();
        let inputs = MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        };
        let out = materialize_plaintext(inputs).unwrap();
        let txt = std::str::from_utf8(out.primary_bytes()).unwrap();
        assert!(txt.contains("body: hello"));
        assert!(!txt.contains('<'));
        assert!(!txt.contains('>'));
    }

    #[test]
    fn gate_denial_collapses_to_redacted_view() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("secret"));
        let alice = principal_cid();
        let inputs = MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: deny_all_cap_recheck(),
            declared_requires: Vec::new(),
        };
        let out = materialize_html_json(inputs).unwrap();
        let html = std::str::from_utf8(out.html_bytes()).unwrap();
        assert!(!html.contains("secret"), "denied content MUST NOT leak");
        assert!(html.contains("[redacted]"));
        assert_eq!(out.cap_denials().len(), 1);
        assert_eq!(
            out.cap_denials()[0].code(),
            ErrorCode::MaterializerCapDenied
        );
    }

    /// Safe-1 #527 closure pin (Pattern F Bundle 5): the
    /// materialization-layer per-row gate's bool MUST be consumed, not
    /// silently swallowed. Would-FAIL if a regression re-introduced the
    /// discarded-bool fan-out (`let _ = (cap_recheck)(...)`) and routed
    /// the render off an unconditional admit: a denying gate must
    /// produce zero materialized rows + a denial frame + redacted
    /// bytes, and an admitting gate must produce exactly one row.
    #[test]
    fn per_row_gate_bool_is_consumed_not_swallowed() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("classified"));
        let alice = principal_cid();
        let denied = materialize_html_json(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: deny_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
        assert!(
            denied.materialized_row_cids().is_empty(),
            "deny gate MUST NOT materialize the row (bool was swallowed if it does)"
        );
        assert_eq!(denied.cap_denials().len(), 1);
        assert!(
            !std::str::from_utf8(denied.html_bytes())
                .unwrap()
                .contains("classified"),
            "denied content MUST NOT leak"
        );

        let admitted = materialize_html_json(MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        })
        .unwrap();
        assert_eq!(
            admitted.materialized_row_cids().len(),
            1,
            "admit gate MUST materialize exactly one row"
        );
        assert!(admitted.cap_denials().is_empty());
    }

    #[test]
    fn wallclock_fail_closed_when_no_clock_injected() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::without_clock();
        let cid = engine.put_node(make_note("body"));
        let alice = principal_cid();
        let inputs = MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        };
        let err = materialize_html_json(inputs).unwrap_err();
        assert!(matches!(err, MaterializerError::UcanClockNotInjected));
        assert_eq!(err.code(), ErrorCode::UcanClockNotInjected);
    }

    #[test]
    fn t1_envelope_violation_rejected_at_entry() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("body"));
        let alice = principal_cid();
        // Declared envelope: only `read:Note` — but the schema emitted
        // `read:Note.body` + `read:Note.created_at` etc.
        let inputs = MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: vec!["read:Note".into()],
        };
        let err = materialize_html_json(inputs).unwrap_err();
        assert!(matches!(err, MaterializerError::SchemaMismatch { .. }));
        assert_eq!(err.code(), ErrorCode::MaterializerSchemaMismatch);
    }

    #[test]
    fn determinism_across_runs() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("body"));
        let alice = principal_cid();
        let mk = || MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        };
        let out1 = materialize_html_json(mk()).unwrap();
        let out2 = materialize_html_json(mk()).unwrap();
        let out3 = materialize_html_json(mk()).unwrap();
        assert_eq!(out1.html_bytes(), out2.html_bytes());
        assert_eq!(out2.html_bytes(), out3.html_bytes());
        assert_eq!(out1.json_bytes(), out2.json_bytes());
        assert_eq!(out1.canonical_cid(), out2.canonical_cid());
    }

    #[test]
    fn dispatch_kinds_are_all_within_canonical_12() {
        let spec = compile(CANONICAL_NOTE).unwrap();
        let engine = InMemoryMaterializerEngine::new();
        let cid = engine.put_node(make_note("body"));
        let alice = principal_cid();
        let inputs = MaterializerWalkInputs {
            engine: &engine,
            spec: &spec,
            content_cid: cid,
            walk_principal: alice,
            cap_recheck: allow_all_cap_recheck(),
            declared_requires: Vec::new(),
        };
        let out = materialize_html_json(inputs).unwrap();
        assert!(!out.dispatched_primitive_kinds().is_empty());
        for k in out.dispatched_primitive_kinds() {
            match k {
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
                _ => panic!("13th primitive variant dispatched: {k:?}"),
            }
        }
    }

    #[test]
    fn subscribe_seam_attaches_via_on_change_as_with_cursor_pattern_only() {
        // Smoke: the seam shape is the trait surface; the actual
        // `Engine::on_change_as_with_cursor` call happens at the
        // consumer boundary. This test pins that the seam rejects
        // empty patterns (same shape as the engine's pattern-invalid
        // guard).
        let mat = HtmlJsonMaterializer;
        let err = mat.subscribe_with_gate("").unwrap_err();
        assert!(matches!(
            err,
            MaterializerError::SubscribeSeamFailure { .. }
        ));
        assert_eq!(err.code(), ErrorCode::MaterializerSubscribeSeamFailure);
        let token = mat.subscribe_with_gate("note:*").unwrap();
        assert_eq!(token.pattern, "note:*");
    }

    /// Defensive doc-test for the Renderer trait surface (arch-r1-16):
    /// the trait MUST NOT name a transport-specific method. This is
    /// the structural assertion that BrowserRender + TauriRenderer +
    /// future Verso/Slint/etc. impls compile against the same shape.
    #[test]
    fn renderer_trait_has_no_transport_specific_methods() {
        // The trait surface — render + backend_name — is asserted
        // structurally by the type system. This test exists as the
        // arch-r1-16 doc-test pin requested by the plan §3 G23-B row.
        let br = BrowserRender;
        assert_eq!(br.backend_name(), "browser-wasm32");
        let dummy = MaterializerOutput {
            primary: b"<article></article>".to_vec(),
            secondary: b"{}".to_vec(),
            denials: Vec::new(),
            materialized_cids: Vec::new(),
            dispatched_kinds: HashSet::new(),
            spec_cid: None,
        };
        br.render(&dummy).unwrap();
    }

    #[test]
    fn dual_gate_admits_iff_both_layers_admit() {
        let mat = HtmlJsonMaterializer;
        let cid = principal_cid(); // any CID will do
        let zone = "Note";
        let admit = allow_all_cap_recheck();
        let deny = deny_all_cap_recheck();
        assert!(mat.dual_gate_admits(&cid, &cid, zone, &admit, &admit));
        assert!(!mat.dual_gate_admits(&cid, &cid, zone, &admit, &deny));
        assert!(!mat.dual_gate_admits(&cid, &cid, zone, &deny, &admit));
        assert!(!mat.dual_gate_admits(&cid, &cid, zone, &deny, &deny));
    }

    #[test]
    fn dispatched_kinds_includes_no_new_primitive() {
        // Sanity: schema_compiler emits Read/Transform/Write/Subscribe/
        // Respond; no other variants.
        let spec = compile(CANONICAL_NOTE).unwrap();
        let mut kinds = HashSet::new();
        for op in spec.as_subgraph().nodes() {
            kinds.insert(op.kind);
        }
        for k in &kinds {
            match k {
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
                _ => panic!("schema_compiler emitted unexpected variant: {k:?}"),
            }
        }
    }
}
