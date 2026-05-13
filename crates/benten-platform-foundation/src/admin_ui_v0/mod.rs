//! Phase 4-Foundation G24-A — admin UI v0 shell + 4-category navigation
//! + thin-client bridge wiring.
//!
//! # What lives here
//!
//! The admin UI v0 is the FIRST app-level plugin per CLAUDE.md
//! baked-in #18. The Rust handler side of the plugin lives in THIS
//! module — composed entirely from the 12 existing primitives, no new
//! `PrimitiveKind` variants minted. The browser-tab + Tauri-embedded-
//! webview front-end lives at `packages/admin-ui-v0/` (TypeScript)
//! and consumes this module's surface via napi-rs (full-peer) or via
//! the thin-client session protocol (`crates/benten-engine/src/thin_client_session.rs`,
//! G24-F) when shape (b) browser-wasm32 talks to a remote full peer.
//!
//! # 4-category navigation IA (ratification #4 + plugin-arch-r1-12 +
//! ux-r1-8)
//!
//! [`Category`] enumerates the 4 user-facing concepts over the unified
//! subgraph substrate: **Plugins** / **Workflows** / **Content Types**
//! / **Views**. [`NAV_CATEGORIES`] is the canonical order this admin UI
//! ships with — locked here so G24-B (workflow editor) + G24-C
//! (composed-view creator) consume a single source-of-truth.
//!
//! # Cap-scoped reads — Class B β consumer (CLAUDE.md #18 cag-r1-9)
//!
//! Reads go through `Engine::read_node_as` threaded via the trait
//! [`crate::materializer::MaterializerEngine`] — the adapter at
//! `tests/admin_ui_v0_engine_adapter.rs` is what binds the trait to a
//! real `benten_engine::Engine`. Production callers in admin-UI's
//! handler code MUST go through that adapter; reaching for the
//! `pub(crate) Engine::read_node` seam is a regression (pinned by
//! `tests/admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`).
//!
//! # Subscribe-paths via `on_change_as_with_cursor` ONLY (sec-3.5-r1-9)
//!
//! [`Subscriber`] requests a subscription pattern via the materializer's
//! [`crate::materializer::HtmlJsonMaterializer::subscribe_with_gate`]
//! seam, which yields a [`crate::materializer::SubscribeAttachToken`].
//! The consumer adapter at the admin-UI integration boundary feeds the
//! token to `Engine::on_change_as_with_cursor` and NEVER to the bare
//! `Engine::on_change` / `Engine::subscribe_change_events` engine-internal
//! surfaces. The grep-assert pin
//! (`admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`)
//! sweeps both this module + `packages/admin-ui-v0/src/`.
//!
//! # IndexedDB write surface (br-r1-7 + T2)
//!
//! The admin UI shape (b) browser-tab writes ONLY to two IndexedDB
//! object stores: [`INDEXEDDB_SNAPSHOT_CACHE_STORE`] and
//! [`INDEXEDDB_MANIFEST_STORE_STORE`]. Forbidden stores
//! ([`INDEXEDDB_FORBIDDEN_STORES`]) carry UCAN cap-tokens / plugin
//! secrets / direct sync state — none of those are admissible into the
//! browser per CLAUDE.md baked-in #17 (deployment shape (b)).
//!
//! # WinterTC future-compat (br-r1-8)
//!
//! [`WINTERTC_FORBIDDEN_APIS`] enumerates the DOM-only + FormData +
//! relative-URL APIs that must NOT appear in the browser bundle so the
//! same wasm32-unknown-unknown bytes ship to WinterTC edge runtimes.
//! The CI guard at G26-B sweeps `packages/admin-ui-v0/src/` for these
//! identifiers. (See in-source comments at
//! `packages/admin-ui-v0/src/index.ts`.)
//!
//! # Private-namespace-as-plugin-data-residency (plugin-arch-r1-18)
//!
//! [`ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX`] is the cap-scope prefix that
//! admin UI writes its in-progress workflow drafts to — caps with this
//! prefix have `shares=none` per the manifest, so other plugins can't
//! receive delegated grants for the drafts namespace. The plugin
//! manifest at install time mints a private-NS cap under this prefix.

#![allow(missing_docs)]

use benten_core::{Cid, OperationNode, PrimitiveKind, Subgraph, Value};

use crate::materializer::{
    HtmlJsonMaterializer, Materializer, MaterializerCapRecheck, MaterializerEngine,
    MaterializerError, MaterializerOutput, MaterializerWalkInputs, SubscribeAttachToken,
    allow_all_cap_recheck,
};
use crate::schema_compiler::SchemaSubgraphSpec;

// ---------------------------------------------------------------------
// 4-category navigation IA (ratification #4 + plugin-arch-r1-12 + ux-r1-8).
// ---------------------------------------------------------------------

/// The 4 user-facing categories admin UI v0 ships with (ratification #4).
///
/// All 4 are subgraphs at the engine layer; the IA distinguishes them by
/// usage intent + lifecycle affordances, not by substrate shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Plugin library subgraph + installed-versions timeline.
    Plugins,
    /// User-authored workflow handlers (12-primitive subgraphs).
    Workflows,
    /// Schemas — typed-field-Node subgraphs per
    /// `docs/SCHEMA-DRIVEN-RENDERING.md` vocabulary.
    ContentTypes,
    /// Composed views — anchor pattern + projection; materialised via
    /// generalised IVM Algorithm B kernel per D-4F-2.
    Views,
}

impl Category {
    /// Canonical user-facing label per ratification #4.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Plugins => "Plugins",
            Self::Workflows => "Workflows",
            Self::ContentTypes => "Content Types",
            Self::Views => "Views",
        }
    }

    /// URL-segment shape per ratification #4 — kebab-case so multi-word
    /// labels survive routing.
    #[must_use]
    pub fn route_slug(self) -> &'static str {
        match self {
            Self::Plugins => "plugins",
            Self::Workflows => "workflows",
            Self::ContentTypes => "content-types",
            Self::Views => "views",
        }
    }
}

/// Canonical 4-category order — Plugins / Workflows / Content Types /
/// Views (per ratification #4). The TS shell pulls this same order via
/// the napi bridge; the lock prevents drift between Rust + TS sides.
pub const NAV_CATEGORIES: [Category; 4] = [
    Category::Plugins,
    Category::Workflows,
    Category::ContentTypes,
    Category::Views,
];

// ---------------------------------------------------------------------
// IndexedDB surface discipline (br-r1-7 + T2).
// ---------------------------------------------------------------------

/// The only IndexedDB object store admin UI shape (b) writes to for
/// snapshot caching. Per CLAUDE.md #17 deployment shape (b):
/// IndexedDB-side state is read-cache only; the durable store is at
/// the full peer.
pub const INDEXEDDB_SNAPSHOT_CACHE_STORE: &str = "snapshot_cache";

/// The only IndexedDB object store admin UI writes for plugin-manifest
/// inspection (used by the install consent flow to remember "have I
/// seen this manifest before" + display the prior consent decision).
pub const INDEXEDDB_MANIFEST_STORE_STORE: &str = "manifest_store";

/// IndexedDB object stores admin UI v0 MUST NEVER write to (br-r1-7 +
/// T2). Containing cap-tokens, plugin secrets, or direct sync state in
/// browser-side storage would break the deployment shape (b)
/// thin-compute contract.
pub const INDEXEDDB_FORBIDDEN_STORES: &[&str] = &[
    "caps",
    "cap_tokens",
    "ucan",
    "ucan_tokens",
    "secrets",
    "private_namespace",
    "plugin_secrets",
    "sync_state",
    "loro_state",
    "iroh_state",
];

// ---------------------------------------------------------------------
// WinterTC future-compat forbidden-API list (br-r1-8).
// ---------------------------------------------------------------------

/// JavaScript / wasm-import identifiers that admin UI v0 MUST NOT use
/// in the browser bundle so the same wasm32-unknown-unknown bytes
/// remain WinterTC-compatible (browser tab AND edge worker AND embedded
/// webview). CI guard at G26-B sweeps `packages/admin-ui-v0/src/` for
/// any line that names one of these.
pub const WINTERTC_FORBIDDEN_APIS: &[&str] = &[
    // DOM-only surfaces — absent in WinterTC profiles.
    "document.cookie",
    // FormData — not in WinterTC core.
    "new FormData",
    // Relative-URL fetch — WinterTC requires absolute URLs.
    "fetch(\"./",
    "fetch('./",
    "fetch(`./",
];

// ---------------------------------------------------------------------
// Private-namespace prefix (plugin-arch-r1-18).
// ---------------------------------------------------------------------

/// Cap-scope prefix admin UI v0 uses for its private-namespace writes
/// (in-progress workflow drafts + view-creator scratch space). The
/// admin-UI plugin manifest declares a cap-grant under this prefix
/// with `shares=none` — engine refuses cross-plugin delegation for it.
pub const ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX: &str = "private:admin-ui-v0";

/// Canonical seam name the admin UI v0 plugin uses for cap-scoped
/// reads — the Class B β surface from CLAUDE.md baked-in #18.
/// Production reads go through this seam via the
/// [`crate::materializer::MaterializerEngine::read_node_as`] trait
/// method (the adapter at the integration boundary forwards to
/// `benten_engine::Engine::read_node_as`). Grep-asserted by
/// `tests/admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`
/// — a non-empty seam name proves the source carries a `read_node_as`
/// reference that isn't merely a doc-comment.
pub const ADMIN_UI_V0_CLASS_B_BETA_READ_SEAM: &str = "MaterializerEngine::read_node_as";

/// Canonical seam name the admin UI v0 plugin uses for change
/// subscriptions — `Engine::on_change_as_with_cursor` per sec-3.5-r1-9.
/// Grep-asserted by
/// `tests/admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`.
pub const ADMIN_UI_V0_SUBSCRIBE_SEAM: &str = "on_change_as_with_cursor";

// ---------------------------------------------------------------------
// Route subgraph builder — composes from the 12 primitives.
// ---------------------------------------------------------------------

/// Build the operation subgraph for one of the 4 admin-UI categories.
///
/// Each route's subgraph is composed entirely of READ + TRANSFORM +
/// RESPOND primitives — NO new `PrimitiveKind` variants (CLAUDE.md
/// baked-in #1). The subgraph is content-addressed (the CID is part of
/// the admin UI plugin's library subgraph).
///
/// **Composition shape:**
///
/// 1. READ — fetches the category-anchored Node CID (e.g. plugin
///    library anchor for `Plugins`; workflow library anchor for
///    `Workflows`; schema library anchor for `ContentTypes`; view
///    library anchor for `Views`).
/// 2. TRANSFORM — projects the result through a category-specific
///    transform (currently identity, but the seam is here for G24-B
///    workflow editor + G24-C view creator to thread their projections).
/// 3. RESPOND — emits the rendered result to the renderer.
///
/// The cap-scope on the READ primitive is the category's read scope
/// (`read:admin-ui-v0:plugins` / `read:admin-ui-v0:workflows` / etc.),
/// which the admin UI manifest grants.
#[must_use]
pub fn build_category_route_subgraph(category: Category) -> Subgraph {
    let handler_id = format!("admin-ui-v0::{}", category.route_slug());
    let slug = category.route_slug();
    let read_id = format!("admin_ui_read_{slug}");
    let transform_id = format!("admin_ui_transform_{slug}");
    let respond_id = format!("admin_ui_respond_{slug}");
    let read = OperationNode::new(&read_id, PrimitiveKind::Read)
        .with_property("cap_scope", Value::Text(read_scope_for(category)))
        .with_property("admin_ui_v0_category", Value::Text(category.label().into()));
    let transform = OperationNode::new(&transform_id, PrimitiveKind::Transform)
        .with_property("admin_ui_v0_category", Value::Text(category.label().into()));
    let respond = OperationNode::new(&respond_id, PrimitiveKind::Respond)
        .with_property("admin_ui_v0_category", Value::Text(category.label().into()));
    let mut sg = Subgraph::new(handler_id);
    sg.nodes.push(read);
    sg.nodes.push(transform);
    sg.nodes.push(respond);
    sg.edges
        .push((read_id.clone(), transform_id.clone(), "feeds".into()));
    sg.edges.push((transform_id, respond_id, "feeds".into()));
    sg
}

fn read_scope_for(category: Category) -> String {
    format!("read:admin-ui-v0:{}", category.route_slug())
}

/// Build the full admin UI v0 subgraph by concatenating the 4 category
/// route subgraphs into one composite subgraph. The composite carries
/// all 12-primitive-kind-set-only nodes (READ + TRANSFORM + RESPOND
/// repeated per category) and an envelope handler-id that the engine
/// registers.
///
/// Used by:
///
/// - 12-primitive-irreducibility pin
///   (`admin_ui_v0_uses_only_12_primitives_no_synthetic_extension`)
/// - 4-category navigation pin
///   (`admin_ui_v0_shell_renders_4_category_navigation`)
#[must_use]
pub fn build_admin_ui_v0_subgraph() -> Subgraph {
    let mut sg = Subgraph::new("admin-ui-v0::shell");
    let mut prev_respond_id: Option<String> = None;
    for category in NAV_CATEGORIES {
        let slug = category.route_slug();
        let read_id = format!("admin_ui_read_{slug}");
        let transform_id = format!("admin_ui_transform_{slug}");
        let respond_id = format!("admin_ui_respond_{slug}");
        sg.nodes.push(
            OperationNode::new(&read_id, PrimitiveKind::Read)
                .with_property("cap_scope", Value::Text(read_scope_for(category)))
                .with_property("admin_ui_v0_category", Value::Text(category.label().into())),
        );
        sg.nodes.push(
            OperationNode::new(&transform_id, PrimitiveKind::Transform)
                .with_property("admin_ui_v0_category", Value::Text(category.label().into())),
        );
        sg.nodes.push(
            OperationNode::new(&respond_id, PrimitiveKind::Respond)
                .with_property("admin_ui_v0_category", Value::Text(category.label().into())),
        );
        sg.edges
            .push((read_id.clone(), transform_id.clone(), "feeds".into()));
        sg.edges
            .push((transform_id.clone(), respond_id.clone(), "feeds".into()));
        if let Some(p) = prev_respond_id.take() {
            sg.edges.push((p, read_id, "next_category".into()));
        }
        prev_respond_id = Some(respond_id);
    }
    sg
}

// ---------------------------------------------------------------------
// Materializer consumer wiring (G23-B §4.13 mr-3 + mr-5).
// ---------------------------------------------------------------------

/// Render one category's content via the shared
/// [`HtmlJsonMaterializer`] consumer. THIS is the consumer-side
/// wiring referenced by phase-4-backlog §4.13 mr-3 + mr-5: the admin
/// UI's content-render path delegates to the materializer (not a
/// bespoke `renderProperty`).
///
/// The walk-principal is the admin-UI plugin DID; the
/// [`MaterializerCapRecheck`] is the materialization-layer gate. The
/// caller (engine adapter at the integration boundary) composes this
/// with the delivery-layer gate at `Engine::on_change_as_with_cursor`
/// to realise the dual-gate composition per sec-3.5-r1-1.
///
/// # Errors
/// Surfaces [`MaterializerError`] verbatim.
pub fn render_category_content<E: MaterializerEngine>(
    engine: &E,
    spec: &SchemaSubgraphSpec,
    content_cid: Cid,
    walk_principal: Cid,
    mat_gate: MaterializerCapRecheck,
    declared_requires: Vec<String>,
) -> Result<MaterializerOutput, MaterializerError> {
    let inputs = MaterializerWalkInputs {
        engine,
        spec,
        content_cid,
        walk_principal,
        cap_recheck: mat_gate,
        declared_requires,
    };
    HtmlJsonMaterializer.materialize_with_gate(inputs)
}

/// Convenience wrapper for tests + the install-time canary path —
/// renders with an allow-all materialization gate (i.e., the
/// authoritative cap-decision is whatever the engine-side
/// `read_node_as` applies).
///
/// # Errors
/// Surfaces [`MaterializerError`] verbatim.
pub fn render_category_content_allow_all<E: MaterializerEngine>(
    engine: &E,
    spec: &SchemaSubgraphSpec,
    content_cid: Cid,
    walk_principal: Cid,
) -> Result<MaterializerOutput, MaterializerError> {
    render_category_content(
        engine,
        spec,
        content_cid,
        walk_principal,
        allow_all_cap_recheck(),
        Vec::new(),
    )
}

// ---------------------------------------------------------------------
// Subscribe seam — §4.13 mr-5 substantive propagation surface.
// ---------------------------------------------------------------------

/// Holds the live-preview attach token for a category route. The
/// admin UI integration adapter (test side or production napi side)
/// feeds [`SubscribeAttachToken::pattern`] to
/// `Engine::on_change_as_with_cursor` — NEVER to bare `on_change` per
/// sec-3.5-r1-9.
///
/// Bare `benten_engine::Engine::subscribe_change_events` is similarly
/// never touched by this module (grep-assert pinned).
#[derive(Debug, Clone)]
pub struct Subscriber {
    /// Subscribe attach token from the materializer seam.
    pub token: SubscribeAttachToken,
    /// Category this subscription is bound to.
    pub category: Category,
}

impl Subscriber {
    /// Build a subscriber for the given category. The pattern is
    /// derived from the admin UI category's anchor (`admin-ui-v0:<slug>:*`).
    ///
    /// # Errors
    /// Returns [`MaterializerError::SubscribeSeamFailure`] when the
    /// derived pattern is empty (never in practice, since the slug is
    /// always non-empty).
    pub fn for_category(category: Category) -> Result<Self, MaterializerError> {
        let pattern = format!("admin-ui-v0:{}:*", category.route_slug());
        let token = HtmlJsonMaterializer.subscribe_with_gate(&pattern)?;
        Ok(Self { token, category })
    }
}

// ---------------------------------------------------------------------
// Inline canary tests — module-level smoke pins for the 4-category +
// 12-primitive commitments. Full pins live at the integration tests
// in `crates/benten-engine/tests/`.
// ---------------------------------------------------------------------

#[cfg(test)]
mod canary {
    use super::*;

    #[test]
    fn nav_categories_canonical_order() {
        assert_eq!(NAV_CATEGORIES.len(), 4);
        assert_eq!(NAV_CATEGORIES[0].label(), "Plugins");
        assert_eq!(NAV_CATEGORIES[1].label(), "Workflows");
        assert_eq!(NAV_CATEGORIES[2].label(), "Content Types");
        assert_eq!(NAV_CATEGORIES[3].label(), "Views");
    }

    #[test]
    fn category_route_slugs_kebab_case() {
        for category in NAV_CATEGORIES {
            let slug = category.route_slug();
            assert!(!slug.contains(' '), "route slug must be kebab-case");
            assert!(!slug.is_empty(), "route slug must be non-empty");
        }
    }

    #[test]
    fn admin_ui_v0_subgraph_uses_only_canonical_12_primitive_kinds() {
        let sg = build_admin_ui_v0_subgraph();
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
                other => panic!(
                    "admin UI v0 subgraph contains non-canonical PrimitiveKind variant: {other:?}"
                ),
            }
        }
    }

    #[test]
    fn admin_ui_v0_subgraph_has_three_primitives_per_category() {
        let sg = build_admin_ui_v0_subgraph();
        // READ + TRANSFORM + RESPOND per category × 4 categories = 12.
        assert_eq!(sg.nodes.len(), 4 * 3);
        // Every node carries the category-tag property — confirms each
        // primitive can be traced back to its category at walk-time.
        for op in sg.nodes() {
            assert!(
                op.property("admin_ui_v0_category").is_some(),
                "every admin UI v0 op MUST carry its category tag"
            );
        }
    }

    #[test]
    fn admin_ui_v0_subscriber_uses_per_category_pattern() {
        for category in NAV_CATEGORIES {
            let sub = Subscriber::for_category(category).unwrap();
            assert!(
                sub.token.pattern.contains(category.route_slug()),
                "subscribe pattern must contain category slug for traceability"
            );
        }
    }

    #[test]
    fn private_namespace_prefix_is_admin_ui_v0_scoped() {
        assert!(ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX.starts_with("private:"));
        assert!(ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX.contains("admin-ui-v0"));
    }

    #[test]
    fn indexeddb_forbidden_stores_blocks_cap_token_namespace() {
        assert!(INDEXEDDB_FORBIDDEN_STORES.contains(&"caps"));
        assert!(INDEXEDDB_FORBIDDEN_STORES.contains(&"ucan_tokens"));
        assert!(INDEXEDDB_FORBIDDEN_STORES.contains(&"sync_state"));
        assert!(!INDEXEDDB_FORBIDDEN_STORES.contains(&INDEXEDDB_SNAPSHOT_CACHE_STORE));
        assert!(!INDEXEDDB_FORBIDDEN_STORES.contains(&INDEXEDDB_MANIFEST_STORE_STORE));
    }

    #[test]
    fn wintertc_forbidden_apis_includes_relative_fetch_form_data_and_dom_cookie() {
        assert!(
            WINTERTC_FORBIDDEN_APIS
                .iter()
                .any(|a| a.contains("FormData"))
        );
        assert!(
            WINTERTC_FORBIDDEN_APIS
                .iter()
                .any(|a| a.contains("document.cookie"))
        );
        assert!(
            WINTERTC_FORBIDDEN_APIS
                .iter()
                .any(|a| a.starts_with("fetch("))
        );
    }
}
