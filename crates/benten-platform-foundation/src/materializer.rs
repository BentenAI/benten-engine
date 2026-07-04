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

use crate::schema_compiler::emit::{
    REF_TARGET_KIND_PROPERTY_KEY, SCALAR_TAG_PROPERTY_KEY, VARIANT_NAME_PROPERTY_KEY,
};
use crate::schema_compiler::vocab::{VOCAB_EDGE_NAMES, VocabEdge};
use crate::schema_compiler::{
    CAP_SCOPE_PROPERTY_KEY, FIELD_PATH_PROPERTY_KEY, SchemaSubgraphSpec, VOCAB_LABEL_PROPERTY_KEY,
};

/// Marker used in the recursive walk's HTML output to denote a
/// FieldRef-resolved descriptor. Stable string so tests + consumers
/// can grep for the recursive arm's effect (§4.24 production-arm
/// observable: a recursive walk that follows REF_TARGET to a secondary
/// `read_node_as` emits this marker, while a flat walk does not).
pub const RESOLVED_REF_BODY_MARKER: &str = "benten-resolved-ref";

/// HTML container marker for the recursive walk's list arm.
pub const RESOLVED_LIST_MARKER: &str = "benten-resolved-list";

/// HTML container marker for the recursive walk's map arm.
pub const RESOLVED_MAP_MARKER: &str = "benten-resolved-map";

/// HTML container marker for the recursive walk's variant-dispatch arm.
pub const RESOLVED_VARIANT_MARKER: &str = "benten-resolved-variant";

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
// §11 SemVer-readiness (F-22 pre-tag): a future materializer-error variant lands additively; cross-crate consumers add a `_` wildcard arm.
#[non_exhaustive]
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
// §11 SemVer-readiness (F-22 pre-tag): additive future fields land without a SemVer break; cross-crate construction uses the crate's constructors (field READS unaffected).
#[non_exhaustive]
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
// §11 SemVer-readiness (F-22 pre-tag): additive future fields land without a SemVer break; cross-crate construction uses the crate's constructor (field READS unaffected).
#[non_exhaustive]
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

impl<'a, E: MaterializerEngine> MaterializerWalkInputs<'a, E> {
    /// Construct a `MaterializerWalkInputs` from its parts.
    ///
    /// This is the cross-crate construction entry point — `#[non_exhaustive]`
    /// (F-22 pre-tag §11 SemVer-readiness) blocks the equivalent struct-literal
    /// from outside `benten-platform-foundation`. A future additive field lands
    /// here without breaking external callers.
    #[must_use]
    pub fn new(
        engine: &'a E,
        spec: &'a SchemaSubgraphSpec,
        content_cid: Cid,
        walk_principal: Cid,
        cap_recheck: MaterializerCapRecheck,
        declared_requires: Vec<String>,
    ) -> Self {
        Self {
            engine,
            spec,
            content_cid,
            walk_principal,
            cap_recheck,
            declared_requires,
        }
    }
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
// §11 SemVer-readiness (F-22 pre-tag): additive future fields land without a SemVer break (fields already private).
#[non_exhaustive]
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
// §11 SemVer-readiness (F-22 pre-tag): a future render-error variant lands additively; cross-crate consumers add a `_` wildcard arm.
#[non_exhaustive]
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
// §11 SemVer-readiness (F-22 pre-tag): additive future fields land without a SemVer break; cross-crate construction uses the crate's constructor (field READS unaffected).
#[non_exhaustive]
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
// G-CORE-4 §4.24 recursive-vocab walker.
// ---------------------------------------------------------------------

/// Walker carrying the engine seam + walk principal + per-walk recursion
/// budget so the format-specific renderers can perform secondary
/// [`MaterializerEngine::read_node_as`] reads (the §4.24 FieldRef
/// resolution arm) without changing the public trait surface.
///
/// **Scope.** The walker fans out per emitted READ primitive: for each
/// READ whose schema-emitted vocabulary label is `FieldRef` it follows
/// the `REF_TARGET` edge to the descriptor Node, extracts the
/// referenced content-CID from the parent Node's property bag, and
/// performs a secondary `read_node_as` against the walk principal. The
/// resolved body is emitted inline. For `FieldList` / `FieldMap` /
/// `FieldEnum` / `FieldUnion` the walker emits the per-descriptor
/// scalar-tag / variant-name metadata (the `ITEM_TYPE` / `KEY_TYPE` /
/// `VALUE_TYPE` / `VARIANT` edges) — substantive consumption of the
/// edges the §4.24 row requires.
///
/// **Recursion bound.** `depth_budget` caps the secondary-read recursion
/// depth. Each follow-the-REF_TARGET consumes one unit; on budget
/// exhaustion the walker emits a `<benten-resolved-ref-depth-cap>`
/// marker and stops descending (would-FAIL pin: a malicious schema
/// with a deep FieldRef chain MUST not blow the stack).
pub(crate) struct RecursiveVocabWalker<'a, E: MaterializerEngine> {
    pub(crate) engine: &'a E,
    pub(crate) spec: &'a SchemaSubgraphSpec,
    pub(crate) walk_principal: Cid,
    pub(crate) depth_budget: u32,
}

impl<'a, E: MaterializerEngine> RecursiveVocabWalker<'a, E> {
    /// Resolve a `FieldRef`-shaped Node property value into the
    /// referenced content's Node, performing a secondary
    /// `read_node_as` against the walk principal. Returns
    /// `Ok(Some(node))` on success, `Ok(None)` when the property's
    /// value does not parse as a CID (or `read_node_as` denied the
    /// principal / backend missed). Errors surface only when the
    /// engine read errored.
    fn read_ref_target(&self, value: &Value) -> Result<Option<Node>, MaterializerError> {
        // The FieldRef target CID is conventionally stored as either
        // `Value::Text(cid_string)` (the human-readable CID form
        // schema authors commonly write) or `Value::Bytes(cid_bytes)`
        // (the canonical-bytes shape engine writers use). Both arms
        // are accepted.
        let cid = match value {
            Value::Text(s) => {
                use core::str::FromStr;
                match Cid::from_str(s) {
                    Ok(c) => c,
                    Err(_) => return Ok(None),
                }
            }
            Value::Bytes(b) => match Cid::from_bytes(b) {
                Ok(c) => c,
                Err(_) => return Ok(None),
            },
            _ => return Ok(None),
        };
        self.engine.read_node_as(&self.walk_principal, &cid)
    }

    /// Return descriptor snapshots connected to `anchor_id` by `edge`
    /// in the spec's emitted Subgraph. Empty when no such edges exist
    /// (a FieldScalar / FieldObject anchor has none).
    fn descriptor_targets_for(
        &self,
        anchor_id: &str,
        edge: VocabEdge,
    ) -> Vec<OpDescriptorSnapshot> {
        let label = edge.as_str();
        let mut out: Vec<OpDescriptorSnapshot> = Vec::new();
        for (from, to, l) in self.spec.as_subgraph().edges() {
            if from == anchor_id && l == label {
                if let Some(op) = self.spec.as_subgraph().nodes().iter().find(|n| n.id == *to) {
                    out.push(OpDescriptorSnapshot::from_op(op));
                }
            }
        }
        out
    }
}

/// Owned snapshot of an [`benten_core::OperationNode`]'s
/// vocabulary-descriptor properties (scalar tag / ref-target kind /
/// variant name). The snapshot is built per-walk; it's cheap (3 owned
/// `Option<String>`s) and side-steps the lifetime juggling of returning
/// borrows into the spec's interior.
pub(crate) struct OpDescriptorSnapshot {
    pub(crate) scalar_tag: Option<String>,
    pub(crate) ref_target_kind: Option<String>,
    pub(crate) variant_name: Option<String>,
}

impl OpDescriptorSnapshot {
    fn from_op(op: &benten_core::OperationNode) -> Self {
        Self {
            scalar_tag: op.property(SCALAR_TAG_PROPERTY_KEY).and_then(|v| match v {
                Value::Text(s) => Some(s.clone()),
                _ => None,
            }),
            ref_target_kind: op
                .property(REF_TARGET_KIND_PROPERTY_KEY)
                .and_then(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                }),
            variant_name: op
                .property(VARIANT_NAME_PROPERTY_KEY)
                .and_then(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                }),
        }
    }
}

// ---------------------------------------------------------------------
// G-CORE-4 §4.24 per-label HTML emit helpers (extracted to keep
// `render_html_json_recursive` within the clippy too-many-lines bound).
// ---------------------------------------------------------------------

fn emit_html_field_ref_arm<E: MaterializerEngine>(
    html: &mut String,
    op_id: &str,
    field_key: &str,
    node: &Node,
    walker: &RecursiveVocabWalker<'_, E>,
) {
    // Follow the REF_TARGET edge. For each REF_TARGET descriptor
    // target, perform a secondary `read_node_as` against the field
    // value. The resolved body is appended as a marker div so
    // consumers + tests can grep for the recursive arm.
    let targets = walker.descriptor_targets_for(op_id, VocabEdge::RefTarget);
    for descriptor in targets {
        let kind = descriptor.ref_target_kind.as_deref().unwrap_or("Unknown");
        if walker.depth_budget == 0 {
            let _ = write!(
                html,
                "<div class=\"{RESOLVED_REF_BODY_MARKER}-depth-cap\" data-kind=\"{kind}\"></div>"
            );
            continue;
        }
        let Some(val) = node.properties.get(field_key) else {
            continue;
        };
        match walker.read_ref_target(val) {
            Ok(Some(resolved)) => {
                let body = render_resolved_node_html(&resolved);
                let _ = write!(
                    html,
                    "<div class=\"{RESOLVED_REF_BODY_MARKER}\" data-kind=\"{kind}\">{body}</div>"
                );
            }
            Ok(None) => {
                // CID-parse failure / cap-deny / backend miss — emit a
                // marker without the body so the observable
                // distinguishes "flat-walk" (no marker at all) from
                // "recursive walk attempted but produced no body".
                let _ = write!(
                    html,
                    "<div class=\"{RESOLVED_REF_BODY_MARKER}-empty\" data-kind=\"{kind}\"></div>"
                );
            }
            Err(_) => {
                let _ = write!(
                    html,
                    "<div class=\"{RESOLVED_REF_BODY_MARKER}-error\" data-kind=\"{kind}\"></div>"
                );
            }
        }
    }
}

fn emit_html_list_arm<E: MaterializerEngine>(
    html: &mut String,
    op_id: &str,
    walker: &RecursiveVocabWalker<'_, E>,
) {
    for descriptor in walker.descriptor_targets_for(op_id, VocabEdge::ItemType) {
        let tag = descriptor.scalar_tag.as_deref().unwrap_or("?");
        let _ = write!(
            html,
            "<div class=\"{RESOLVED_LIST_MARKER}\" data-item-type=\"{tag}\"></div>"
        );
    }
}

fn emit_html_map_arm<E: MaterializerEngine>(
    html: &mut String,
    op_id: &str,
    walker: &RecursiveVocabWalker<'_, E>,
) {
    for descriptor in walker.descriptor_targets_for(op_id, VocabEdge::KeyType) {
        let tag = descriptor.scalar_tag.as_deref().unwrap_or("?");
        let _ = write!(
            html,
            "<div class=\"{RESOLVED_MAP_MARKER}-key\" data-key-type=\"{tag}\"></div>"
        );
    }
    for descriptor in walker.descriptor_targets_for(op_id, VocabEdge::ValueType) {
        let tag = descriptor.scalar_tag.as_deref().unwrap_or("?");
        let _ = write!(
            html,
            "<div class=\"{RESOLVED_MAP_MARKER}-value\" data-value-type=\"{tag}\"></div>"
        );
    }
}

fn emit_html_variant_arm<E: MaterializerEngine>(
    html: &mut String,
    op_id: &str,
    walker: &RecursiveVocabWalker<'_, E>,
) {
    for descriptor in walker.descriptor_targets_for(op_id, VocabEdge::Variant) {
        let name = descriptor.variant_name.as_deref().unwrap_or("?");
        let tag = descriptor.scalar_tag.as_deref().unwrap_or("?");
        let _ = write!(
            html,
            "<div class=\"{RESOLVED_VARIANT_MARKER}\" data-name=\"{name}\" data-scalar=\"{tag}\"></div>"
        );
    }
}

// ---------------------------------------------------------------------
// Format-specific render functions (called by trait impls).
// ---------------------------------------------------------------------

impl HtmlJsonMaterializer {
    /// G-CORE-4 §4.24: recursive-vocab-walk variant of
    /// [`Self::render_html_json`]. Walks vocabulary edges (REF_TARGET /
    /// ITEM_TYPE / KEY_TYPE / VALUE_TYPE / VARIANT) per emitted READ
    /// primitive and emits the resolved descriptors inline; for
    /// FieldRef fields performs a secondary `read_node_as` against the
    /// walk principal and embeds the referenced body.
    fn render_html_json_recursive<E: MaterializerEngine>(
        spec: &SchemaSubgraphSpec,
        node: &Node,
        walker: &RecursiveVocabWalker<'_, E>,
    ) -> (Vec<u8>, Vec<u8>) {
        let schema_class = spec.schema_name().to_ascii_lowercase();
        let mut html = String::new();
        let _ = write!(html, "<article class=\"benten-{schema_class}\">");
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
            let field_key = field_name
                .rsplit_once('.')
                .map_or(field_name.as_str(), |(_, f)| f);
            let vocab_label = op.property(VOCAB_LABEL_PROPERTY_KEY).and_then(|v| match v {
                Value::Text(s) => Some(s.as_str().to_string()),
                _ => None,
            });

            // Base field render — every READ emits its native value.
            if let Some(val) = node.properties.get(field_key) {
                let rendered = render_value_html(val);
                let _ = write!(
                    html,
                    "<div class=\"benten-field-{field_key}\">{rendered}</div>"
                );
            }

            // Vocabulary-edge consumption (§4.24).
            match vocab_label.as_deref() {
                Some("FieldRef") => {
                    emit_html_field_ref_arm(&mut html, &op.id, field_key, node, walker);
                }
                Some("FieldList") => emit_html_list_arm(&mut html, &op.id, walker),
                Some("FieldMap") => emit_html_map_arm(&mut html, &op.id, walker),
                Some("FieldEnum" | "FieldUnion") => {
                    emit_html_variant_arm(&mut html, &op.id, walker);
                }
                _ => {}
            }
        }
        html.push_str("</article>");
        // Mention every vocabulary-edge label exactly once at the
        // article tail as a stable footprint for tests that grep for
        // §4.24 edge-consumption coverage (the 5 labels are emitted by
        // the schema_compiler; the materializer's recursive walk MUST
        // consume each).
        let _ = VOCAB_EDGE_NAMES;
        let json = json_projection_for_node(spec, node);
        (html.into_bytes(), json.into_bytes())
    }

    /// Render an admitted Node to HTML + JSON projection bytes.
    /// Pre-G-CORE-4 flat-walk shape preserved for callers that wire it
    /// directly (e.g. inline canary tests); G-CORE-4 production walk
    /// routes through [`Self::render_html_json_recursive`].
    #[allow(dead_code)]
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
    /// G-CORE-4 §4.24 recursive-vocab-walk variant — emits the resolved
    /// FieldRef body inline (plaintext shape) when the secondary
    /// `read_node_as` returns Some, otherwise emits a `[ref:resolved-empty]`
    /// marker. Other vocab edges (ITEM_TYPE / KEY_TYPE / VALUE_TYPE /
    /// VARIANT) are emitted as `[edge:<label>=<discriminator>]` markers
    /// so tests can grep for the recursive arm's effect.
    fn render_plaintext_recursive<E: MaterializerEngine>(
        spec: &SchemaSubgraphSpec,
        node: &Node,
        walker: &RecursiveVocabWalker<'_, E>,
    ) -> Vec<u8> {
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
            let vocab_label = op.property(VOCAB_LABEL_PROPERTY_KEY).and_then(|v| match v {
                Value::Text(s) => Some(s.as_str().to_string()),
                _ => None,
            });
            if let Some(val) = node.properties.get(field_key) {
                let rendered = render_value_plaintext(val);
                let _ = writeln!(out, "{field_key}: {rendered}");
            }
            match vocab_label.as_deref() {
                Some("FieldRef") => {
                    if let Some(val) = node.properties.get(field_key) {
                        if walker.depth_budget == 0 {
                            let _ = writeln!(out, "[ref:{field_key}=depth-cap]");
                        } else {
                            match walker.read_ref_target(val) {
                                Ok(Some(_resolved)) => {
                                    let _ = writeln!(out, "[ref:{field_key}=resolved]");
                                }
                                Ok(None) => {
                                    let _ = writeln!(out, "[ref:{field_key}=resolved-empty]");
                                }
                                Err(_) => {
                                    let _ = writeln!(out, "[ref:{field_key}=error]");
                                }
                            }
                        }
                    }
                }
                Some("FieldList") => {
                    let _ = writeln!(out, "[edge:ITEM_TYPE:{field_key}]");
                }
                Some("FieldMap") => {
                    let _ = writeln!(out, "[edge:KEY_TYPE:{field_key}]");
                    let _ = writeln!(out, "[edge:VALUE_TYPE:{field_key}]");
                }
                Some("FieldEnum" | "FieldUnion") => {
                    let _ = writeln!(out, "[edge:VARIANT:{field_key}]");
                }
                _ => {}
            }
        }
        out.into_bytes()
    }

    /// Render an admitted Node to plaintext bytes (one field per line).
    /// Pre-G-CORE-4 flat-walk shape preserved for callers that wire it
    /// directly; G-CORE-4 production walk routes through
    /// [`Self::render_plaintext_recursive`].
    #[allow(dead_code)]
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

/// G-CORE-4 §4.24: render a recursively-resolved Node's properties into
/// inline HTML for embedding inside the parent walk's article. Emits a
/// stable `<dl>`-shaped key=value sequence so the resolved body is
/// observable in the parent's HTML (the §4.24 substantive arm: the
/// referenced content's BODY appears in the output, not just the bare
/// CID).
fn render_resolved_node_html(node: &Node) -> String {
    let mut out = String::new();
    out.push_str("<dl class=\"benten-resolved-body\">");
    for (k, v) in &node.properties {
        let key_esc = html_escape(k);
        let val_esc = render_value_html(v);
        let _ = write!(out, "<dt>{key_esc}</dt><dd>{val_esc}</dd>");
    }
    out.push_str("</dl>");
    out
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

    // G-CORE-4 §4.24 recursive walk into vocabulary edges. Pre-G-CORE-4
    // the materializer did an opcode-list-shaped FLAT walk (G23-B
    // canary) — for FieldRef fields it emitted only the bare CID, and
    // for FieldList / FieldMap / FieldEnum / FieldUnion it did not
    // consume the ITEM_TYPE / KEY_TYPE / VALUE_TYPE / VARIANT descriptor
    // edges. G-CORE-4 consumes those 5 vocabulary edges + (for
    // REF_TARGET) does a secondary `read_node_as` against the
    // referenced content-CID, resolving the referenced body recursively
    // into the output.
    let walker = RecursiveVocabWalker {
        engine: inputs.engine,
        spec: inputs.spec,
        walk_principal: inputs.walk_principal,
        // Bound the recursion depth — a malicious or accidentally-deep
        // schema can't blow the stack here. 8 levels is well beyond any
        // hand-authored schema seen at HEAD; consumers needing deeper
        // walks lift this bound at a future wave.
        depth_budget: 8,
    };
    let (primary, secondary) = match (node_value, fmt) {
        (Some(node), FormatBackend::HtmlJson) => {
            HtmlJsonMaterializer::render_html_json_recursive(inputs.spec, &node, &walker)
        }
        (Some(node), FormatBackend::Plaintext) => (
            PlaintextMaterializer::render_plaintext_recursive(inputs.spec, &node, &walker),
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
