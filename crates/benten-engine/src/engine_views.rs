//! Change-stream probe + IVM view-read surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. Houses
//! `subscribe_change_events`, the test-only probe variants,
//! `change_event_count`, and the three view-read entry points
//! (`read_view`, `read_view_with`, `read_view_strict`,
//! `read_view_allow_stale`). Every method is a plain `impl Engine` item.
//!
//! **Phase 2b G8-B addition.** [`Engine::create_user_view`] registers
//! user-defined views via [`UserViewSpec`]. The user-view path is the
//! generalized Algorithm B lane (`Strategy::B` per D8-RESOLVED). The 5
//! Phase-1 hand-written views remain on `Strategy::A` (Rust-only) and are
//! still registered through the legacy `Engine::create_view(view_id, opts)`
//! surface in [`crate::engine_caps`].

use std::collections::BTreeMap;
use std::sync::Arc;

use benten_caps::CapError;
use benten_core::{Cid, Node, Value};

use crate::change_probe::ChangeProbe;
use crate::engine::{Engine, is_known_view_id};
use crate::error::EngineError;
use crate::outcome::{Outcome, ReadViewOptions, UserViewInputPattern, UserViewSpec};

impl Engine {
    // -------- Change stream surface --------

    /// Subscribe to ChangeEvents. Returns a [`ChangeProbe`] that `drain()`s
    /// every event observed since the probe was created.
    pub fn subscribe_change_events(&self) -> ChangeProbe {
        ChangeProbe {
            inner: Arc::clone(&self.inner),
            start_offset: self
                .inner
                .event_count
                .load(std::sync::atomic::Ordering::SeqCst),
            label_filter: None,
        }
    }

    /// Test-only probe equivalent to `subscribe_change_events` — kept so
    /// integration tests written against the v1 name keep compiling.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn test_subscribe_all_change_events(&self) -> ChangeProbe {
        self.subscribe_change_events()
    }

    /// Subscribe filtered to a specific label.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn test_subscribe_change_events_matching_label(&self, label: &str) -> ChangeProbe {
        ChangeProbe {
            inner: Arc::clone(&self.inner),
            start_offset: self
                .inner
                .event_count
                .load(std::sync::atomic::Ordering::SeqCst),
            label_filter: Some(label.to_string()),
        }
    }

    /// Count of ChangeEvents emitted since the engine opened.
    #[must_use]
    pub fn change_event_count(&self) -> u64 {
        self.inner
            .event_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    // -------- View reads (IVM) --------

    /// Strict read of an IVM view. Phase 1: returns typed errors for the
    /// unknown-view, no-IVM, and stale paths; the healthy-view path routes
    /// through the evaluator-backed primitive dispatch which is Phase 2.
    pub fn read_view(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::strict())
    }

    /// Read an IVM view with explicit options.
    ///
    /// Consults the live IVM subscriber (philosophy g7-ep-2): the healthy
    /// path returns an Outcome whose `list` reflects the view's current
    /// state; strict reads of a stale view error with `E_IVM_VIEW_STALE`;
    /// relaxed reads of a stale view return the empty last-known-good.
    /// Unknown view ids error with `E_UNKNOWN_VIEW`.
    ///
    /// Option C (5d-J workstream 1): when the view id encodes a label
    /// (`content_listing_<label>`) and the policy denies a read on
    /// that label, the return collapses to an empty list — symmetric
    /// with an empty view.
    pub fn read_view_with(
        &self,
        view_id: &str,
        opts: ReadViewOptions,
    ) -> Result<Outcome, EngineError> {
        if !self.ivm_enabled {
            return Err(EngineError::SubsystemDisabled { subsystem: "ivm" });
        }
        // Derive a label from the view id for the read-gate. Only
        // content_listing_<label> views carry a Phase-1 label hint;
        // other view ids pass through unchanged.
        if let Some(policy) = self.policy.as_deref() {
            let label = view_id
                .strip_prefix("content_listing_")
                .or_else(|| view_id.strip_prefix("system:ivm:content_listing_"))
                .unwrap_or("");
            if !label.is_empty() {
                let ctx = benten_caps::ReadContext {
                    label: label.to_string(),
                    target_cid: None,
                    ..Default::default()
                };
                if let Err(CapError::DeniedRead { .. }) = policy.check_read(&ctx) {
                    return Ok(Outcome {
                        list: Some(Vec::new()),
                        ..Outcome::default()
                    });
                }
            }
        }
        // Normalize the namespaced alias `system:ivm:<id>` → `<id>`.
        let normalized = view_id.strip_prefix("system:ivm:").unwrap_or(view_id);
        // Consult the subscriber first — if a live view exists with this id,
        // route through it. Falling back to the canonical-id whitelist
        // preserves the Phase-1 contract for views that haven't been
        // create_view-registered yet but are named in R3 tests.
        if let Some(ivm) = self.ivm.as_ref()
            && let Some(is_stale) = ivm.view_is_stale(normalized)
        {
            if is_stale {
                return if opts.allow_stale {
                    Ok(Outcome {
                        list: Some(Vec::new()),
                        ..Outcome::default()
                    })
                } else {
                    Err(EngineError::IvmViewStale {
                        view_id: view_id.to_string(),
                    })
                };
            }
            // Healthy view — return empty listing (Phase 1: view's full
            // read API surface is Phase 2).
            return Ok(Outcome {
                list: Some(Vec::new()),
                ..Outcome::default()
            });
        }
        // No live view registered for this id. Phase 1 canonical whitelist
        // decides: recognized -> stale (in strict) / last-known-good empty
        // (relaxed). Unknown -> UnknownView error.
        if !is_known_view_id(view_id) {
            return Err(EngineError::UnknownView {
                view_id: view_id.to_string(),
            });
        }
        if opts.allow_stale {
            Ok(Outcome {
                list: Some(Vec::new()),
                ..Outcome::default()
            })
        } else {
            Err(EngineError::IvmViewStale {
                view_id: view_id.to_string(),
            })
        }
    }

    /// Strict view read — alias for [`Self::read_view`].
    ///
    /// Retained for source-compatibility with R3 tests that spell the strict
    /// intent explicitly. `read_view_strict(id)` is literally
    /// `read_view_with(id, ReadViewOptions::strict())` and is documented as
    /// such so operators choosing between the three names know the contract
    /// is identical (R-minor-05).
    pub fn read_view_strict(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::strict())
    }

    /// Relaxed view read — equivalent to
    /// [`Self::read_view_with`] with `ReadViewOptions::allow_stale()`.
    /// Retained for R3 test source-compatibility (R-minor-05).
    pub fn read_view_allow_stale(&self, view_id: &str) -> Result<Outcome, EngineError> {
        self.read_view_with(view_id, ReadViewOptions::allow_stale())
    }

    // -------- User-view registration (Phase 2b G8-B) --------

    /// Register a user-defined IVM view via the [`UserViewSpec`] builder.
    ///
    /// This is the Phase-2b user-view registration surface. Defaults to
    /// `Strategy::B` per D8-RESOLVED — the 5 hand-written Phase-1 views
    /// stay on `Strategy::A` (Rust-only) and continue to be registered via
    /// the legacy [`Engine::create_view`] `(view_id, ViewCreateOptions)`
    /// overload in [`crate::engine_caps`].
    ///
    /// # Errors
    ///
    /// - [`EngineError::SubsystemDisabled`] when IVM is disabled
    ///   (`.without_ivm()` engine builder).
    /// - [`EngineError::ViewStrategyARefused`] when the spec declared
    ///   `Strategy::A` (Strategy A is hand-written-IVM-only; user views
    ///   cannot claim that lane).
    /// - [`EngineError::ViewStrategyCReserved`] when the spec declared
    ///   `Strategy::C` (Z-set / DBSP cancellation reserved for Phase 3+).
    /// - Backend errors from the underlying privileged Node write.
    pub fn create_user_view(&self, spec: UserViewSpec) -> Result<Cid, EngineError> {
        // D8-RESOLVED: refuse Strategy::A + Strategy::C BEFORE writing the
        // definition Node so a refused registration leaves no on-disk
        // residue (the `system:IVMView` Node only exists for accepted
        // strategies).
        match spec.strategy() {
            benten_ivm::Strategy::A => {
                return Err(EngineError::ViewStrategyARefused {
                    view_id: spec.id().to_string(),
                });
            }
            benten_ivm::Strategy::C => {
                return Err(EngineError::ViewStrategyCReserved {
                    view_id: spec.id().to_string(),
                });
            }
            benten_ivm::Strategy::B => { /* accepted */ }
        }

        if !self.ivm_enabled {
            return Err(EngineError::SubsystemDisabled { subsystem: "ivm" });
        }

        // Derive the input-pattern label (used by the ContentListingView
        // shim until G8-A's generalized Algorithm B port lands; per the
        // §3 G8-B coordination note the user-view ingestion path stubs
        // through ContentListingView in the Label case so the registration
        // round-trip is observable end-to-end on this branch).
        //
        // ⚠️ PRE-G8-A SEMANTIC STUB: AnchorPrefix is silently coerced to
        // a Label-equality match against the prefix string (because
        // ContentListingView only knows label equality). An app that
        // declares `inputPattern: { anchorPrefix: "post" }` and then
        // reads the user view will see results filtered by `label ==
        // "post"`, NOT by anchor prefix. This is a stub bridge until
        // G8-A's per-strategy view dispatch lands (then this branch
        // swaps to the proper anchor-prefix selector). DO NOT rely on
        // AnchorPrefix semantics in tests or app code that targets the
        // pre-G8-A engine. The DSL surface (`packages/engine/src/views.ts`
        // `UserViewInputPattern` doc + `outcome.rs::UserViewInputPattern`)
        // mirrors this warning.
        let input_pattern_label = match spec.input_pattern() {
            UserViewInputPattern::Label(l) => Some(l.clone()),
            UserViewInputPattern::AnchorPrefix(prefix) => Some(prefix.clone()),
        };

        // Persist the view definition Node so the registration is content-
        // addressed + visible to Phase-3 sync. The Node carries the user
        // view's id + input-pattern label + an explicit `strategy: "B"`
        // property so the catalog encoding round-trips a future-loader's
        // Strategy enum lookup unambiguously.
        let mut def_props: BTreeMap<String, Value> = BTreeMap::new();
        def_props.insert("view_id".into(), Value::text(spec.id()));
        if let Some(label) = input_pattern_label.as_deref() {
            def_props.insert("input_pattern_label".into(), Value::text(label));
        }
        match spec.input_pattern() {
            UserViewInputPattern::Label(_) => {
                def_props.insert("input_pattern_kind".into(), Value::text("label"));
            }
            UserViewInputPattern::AnchorPrefix(_) => {
                def_props.insert("input_pattern_kind".into(), Value::text("anchor_prefix"));
            }
        }
        def_props.insert("strategy".into(), Value::text("B"));
        let def_node = Node::new(vec!["system:IVMView".into()], def_props);
        let cid = self.privileged_put_node_for_user_view(&def_node)?;

        // Register a live view instance with the IVM subscriber so future
        // change events propagate. We dedupe by view id so re-registering
        // the same id is a no-op at the subscriber level.
        if let Some(ivm) = self.ivm.as_ref() {
            let already_registered = ivm.view_ids().iter().any(|id| id == spec.id());
            if !already_registered && let Some(label) = input_pattern_label.as_deref() {
                // Phase 2b G8-B: until G8-A's generalized Algorithm B port
                // lands, the user-view runtime ingestion path stubs through
                // ContentListingView keyed on the input pattern label so
                // change events still flow to a live View. Once G8-A ships
                // the dispatch swaps to the per-strategy view constructor;
                // the registration surface (this method) does not change.
                let view = benten_ivm::views::ContentListingView::new(label);
                ivm.register_view(Box::new(view));
            }
        }

        Ok(cid)
    }

    /// Internal helper mirroring `engine_caps::Engine::privileged_put_node`
    /// so the user-view registration path can write its system-zone Node
    /// without re-exporting the helper. The two implementations are
    /// intentionally identical in body — keeping this one private to
    /// `engine_views` avoids bumping `engine_caps`'s public surface.
    fn privileged_put_node_for_user_view(&self, node: &Node) -> Result<Cid, EngineError> {
        Ok(self.backend.put_node_with_context(
            node,
            &benten_graph::WriteContext::privileged_for_engine_api(),
        )?)
    }
}

// `is_known_view_id` consultation note (Phase 2b G8-B addresses
// engine.rs:1976 TODO partially): user-view registration via
// [`Engine::create_user_view`] adds the user's view id into the IVM
// subscriber's `view_ids()` set. The `read_view_with` path above already
// consults the live subscriber FIRST (see the `if let Some(ivm) = ...`
// block) and only falls back to the canonical-id whitelist when the
// subscriber has no live view for the id — so user-registered views are
// observable through `read_view*` immediately after `create_user_view`
// returns.
//
// The TODO at engine.rs:1976 ("replace with a per-view definition
// registration pulled from benten-ivm") is now subscriber-driven for
// the dynamic-registration path; the static whitelist of canonical ids
// remains for the 5 hand-written views which do not auto-register a
// live subscriber on engine open. Full removal of the whitelist waits
// on G8-A's Algorithm B port to expose a definition-registry method on
// the subscriber.
