//! Change-stream probe + IVM view-read surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. Houses
//! `subscribe_change_events`, the test-only probe variants,
//! `change_event_count`, and the three view-read entry points
//! (`read_view`, `read_view_with`, `read_view_strict`,
//! `read_view_allow_stale`). Every method is a plain `impl Engine` item.
//!
//! **Phase 2b G8-B addition.** [`Engine::register_user_view`] registers
//! user-defined views via [`UserViewSpec`]. The user-view path runs under
//! `Strategy::B` per D8-RESOLVED; `Strategy::A` user-view registration is
//! refused with a typed error.
//!
//! **Phase-3 G15-A generalization.** The Phase-2b "user-defined view IDs
//! hit a `ContentListingView` fallback" disclaimer is RETIRED. Non-canonical
//! view ids now route through [`benten_ivm::Algorithm::register`]'s generic
//! kernel keyed on `(label_pattern, projection)` per `D-PHASE-3-28
//! RESOLVED`. The 5 hand-written canonical views remain as inner kernels of
//! Strategy::B per `ivm-disagree-1`; they are reachable via the legacy
//! `(view_id, ViewCreateOptions)` overload in [`crate::engine_caps`].

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

    /// Wave-8h audit-gap fix — subscribe to EMIT events.
    ///
    /// Standalone EMIT primitives (handlers using EMIT without a
    /// backing WRITE) publish their `(channel, payload)` pair through
    /// the engine's dedicated EMIT broadcast. Storage events continue
    /// to flow through [`Self::subscribe_change_events`]; the two
    /// channels are independent because [`benten_graph::ChangeEvent`]
    /// is keyed on a Node CID + commit-time fields that EMIT events
    /// don't carry. See `crate::emit_broadcast` module docs.
    ///
    /// The closure runs synchronously on whatever thread the EMIT
    /// primitive was dispatched from. Callbacks that need async
    /// dispatch should enqueue onto their own channel.
    pub fn subscribe_emit_events<F>(&self, f: F)
    where
        F: Fn(&crate::emit_broadcast::EmitEvent) + Send + Sync + 'static,
    {
        self.inner.emit_broadcast.subscribe_fn(f);
    }

    /// R6FP-Group-1 (r6-mpc-2 engine half) — subscribe to EMIT events
    /// AND return an [`crate::emit_broadcast::EmitSubscription`]
    /// handle. Mirrors the SUBSCRIBE [`Self::on_change`] handle pattern
    /// so the napi binding (`EmitSubscriptionJs`) can carry a
    /// JS-visible lifecycle the consumer can `unsubscribe()` / drop.
    /// The handle's Drop is idempotent with `unsubscribe()` and flips
    /// an active-flag the publish path consults — subsequent emits
    /// skip the handler's callback once the handle is gone.
    pub fn subscribe_emit_events_with_handle<F>(
        &self,
        f: F,
    ) -> crate::emit_broadcast::EmitSubscription
    where
        F: Fn(&crate::emit_broadcast::EmitEvent) + Send + Sync + 'static,
    {
        self.inner.emit_broadcast.subscribe_with_handle(f)
    }

    /// Wave-8h audit-gap fix — subscriber count for the EMIT broadcast.
    /// Used by tests asserting registration + by operator tooling
    /// surfacing emit-channel observability.
    #[must_use]
    pub fn emit_subscriber_count(&self) -> usize {
        self.inner.emit_broadcast.subscriber_count()
    }

    /// Wave-8h audit-gap fix #3 — query the [`benten_ivm::Strategy`]
    /// of a registered IVM view. Returns `None` when no view with
    /// `view_id` is registered (or when IVM is disabled via
    /// `.without_ivm()`).
    ///
    /// Used by the IVM-B integration test to assert that a user view
    /// registered via [`Self::create_user_view`] with the default
    /// `Strategy::B` actually runs through [`benten_ivm::algorithm_b::AlgorithmBView`]
    /// at runtime — the audit surfaced that the prior code unconditionally
    /// fell back to `ContentListingView` (which reports `Strategy::A`).
    #[must_use]
    pub fn view_strategy(&self, view_id: &str) -> Option<benten_ivm::Strategy> {
        self.ivm.as_ref().and_then(|ivm| ivm.view_strategy(view_id))
    }

    // -------- Per-row READ gate (Compromise #11 closure, G15-A) --------

    /// Materialize a registered IVM view's row CIDs filtered through the
    /// G15-A per-row READ gate (Compromise #11 closure).
    ///
    /// Returns `Ok(None)` when no view with `view_id` is registered.
    /// Returns `Ok(Some(cids))` for the row CIDs the actor (`gate`) is
    /// permitted to READ — fewer than the unfiltered row count when the
    /// actor's cap-set excludes any row, equal to the unfiltered count
    /// when every row is permitted.
    ///
    /// The materialization-time gate fires SEPARATELY from G14-D's
    /// delivery-time gate at SUBSCRIBE per `ivm-major-2`; both layers
    /// compose at `crates/benten-engine/tests/ivm_view_subscribe_compose.rs`.
    ///
    /// # Errors
    ///
    /// - [`EngineError::SubsystemDisabled`] when IVM is disabled
    ///   (`.without_ivm()`).
    /// - [`EngineError::IvmViewStale`] when the view is stale and the
    ///   gate is not configured for relaxed reads (current shape: always
    ///   strict — relaxed-mode gate is a Phase-3+ extension).
    pub fn materialize_view_with_gate(
        &self,
        view_id: &str,
        gate: &crate::ivm_view_read_gate::IvmViewReadGate,
    ) -> Result<Option<Vec<Cid>>, EngineError> {
        if !self.ivm_enabled {
            return Err(EngineError::SubsystemDisabled { subsystem: "ivm" });
        }
        let normalized = view_id.strip_prefix("system:ivm:").unwrap_or(view_id);
        let Some(ivm) = self.ivm.as_ref() else {
            return Ok(None);
        };
        let query = benten_ivm::ViewQuery::default();
        let Some(read_result) = ivm.read_view(normalized, &query) else {
            return Ok(None);
        };
        let view_result = match read_result {
            Ok(vr) => vr,
            Err(benten_ivm::ViewError::Stale { .. }) => {
                return Err(EngineError::IvmViewStale {
                    view_id: view_id.to_string(),
                });
            }
            Err(_) => {
                // Pattern-mismatch / budget: empty list (no useful answer
                // for this query shape; matches the non-gated read path).
                return Ok(Some(Vec::new()));
            }
        };
        let unfiltered: Vec<Cid> = match view_result {
            benten_ivm::ViewResult::Cids(cids) => cids,
            benten_ivm::ViewResult::Current(Some(cid)) => vec![cid],
            benten_ivm::ViewResult::Current(None) | benten_ivm::ViewResult::Rules(_) => Vec::new(),
        };
        Ok(Some(gate.filter_rows(unfiltered)))
    }

    // -------- View reads (IVM) --------

    /// Strict read of an IVM view.
    ///
    /// Returns typed errors for the unknown-view, no-IVM, and stale paths.
    /// The healthy-view path delegates to
    /// [`benten_ivm::Subscriber::read_view`] and projects the registered
    /// view's current state into `Outcome.list` (R6FP-tail NEW-1
    /// wire-through; pre-NEW-1 this branch returned `Vec::new()`). See
    /// [`Self::read_view_with`] for the projection details + the
    /// `ReadViewOptions` knob set.
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
    ///
    /// # R6FP-tail (NEW-1) wire-through
    ///
    /// The healthy-view path delegates to
    /// [`benten_ivm::Subscriber::read_view`] and projects the returned
    /// [`benten_ivm::ViewResult`] into the Outcome:
    ///
    /// - [`benten_ivm::ViewResult::Cids`] — each CID is hydrated through
    ///   the backend (`get_node`) and the resulting Nodes populate
    ///   `Outcome.list`. Order is preserved as the view returns it.
    /// - [`benten_ivm::ViewResult::Current`] — version-chain pointer; if
    ///   `Some(cid)` the corresponding Node is hydrated and emitted as a
    ///   single-element list. `None` produces an empty list.
    /// - [`benten_ivm::ViewResult::Rules`] — governance rules map; encoded
    ///   into a synthetic Node carrying the rule keys/values as
    ///   properties so consumers reading `Outcome.list` see one Node
    ///   per view. (Test-only views consuming this shape can downcast
    ///   via the property bag.)
    ///
    /// Pre-NEW-1 the healthy-view branch returned `Vec::new()`; subgraphs
    /// composing READ_VIEW against a registered view saw empty results
    /// regardless of contents. The wire-through follows the projection
    /// pattern used by `primitive_host::resolve_list_via_view_or_backend`
    /// for Phase-1 content-listing reads, lifted here so user views are
    /// observable through the public engine API.
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
        // other view ids pass through unchanged. r6-r3-ivm-2 BOUNDED
        // disclosure: this prefix-only heuristic means the 4 hardcoded-
        // label canonical views + ALL user-defined views bypass the
        // `check_read` hook on the view-level read path. Bounded by the
        // Phase-2b cap-policy backends (NoAuth / GrantBacked); see
        // `docs/SECURITY-POSTURE.md` Compromise #11 sub-block "Phase-2b
        // R6 Round-3 surfacing — read_view_with view-id-prefix
        // heuristic" for the full disclosure + Phase-3 lift plan.
        let label_hint = view_id
            .strip_prefix("content_listing_")
            .or_else(|| view_id.strip_prefix("system:ivm:content_listing_"))
            .unwrap_or("")
            .to_string();
        if let Some(policy) = self.policy.as_deref()
            && !label_hint.is_empty()
        {
            let ctx = benten_caps::ReadContext {
                label: label_hint.clone(),
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
                    // R6FP-tail (NEW-1) wire-through — project the
                    // view's last-known-good snapshot via
                    // `Subscriber::read_view_allow_stale` instead of
                    // returning the pre-NEW-1 empty-list stub.
                    let label_for_query = if label_hint.is_empty() {
                        None
                    } else {
                        Some(label_hint.clone())
                    };
                    let query = benten_ivm::ViewQuery {
                        label: label_for_query,
                        limit: None,
                        offset: None,
                        ..Default::default()
                    };
                    project_view_read_to_outcome(
                        self,
                        ivm.read_view_allow_stale(normalized, &query),
                        view_id,
                        true,
                    )
                } else {
                    Err(EngineError::IvmViewStale {
                        view_id: view_id.to_string(),
                    })
                };
            }
            // R6FP-tail (NEW-1) wire-through — delegate to the live
            // subscriber + project the ViewResult into Outcome.list.
            //
            // The pre-NEW-1 stub returned `Vec::new()` here regardless of
            // view contents; subgraphs composing READ_VIEW against a
            // registered view silently saw empty results. The fix routes
            // through `Subscriber::read_view` (the canonical read entry
            // point already used by `resolve_list_via_view_or_backend`)
            // and lifts the three `ViewResult` shapes (`Cids` / `Current`
            // / `Rules`) into hydrated `Vec<Node>`.
            //
            // R6-R3 r6-r3-ivm-4: relaxed-mode fast-path TOCTOU close.
            // Between the `view_is_stale` check above and the read below
            // the view's mutex is released; the view can flip stale in
            // that window. Pre-fix the relaxed path called `read_view`
            // (strict) here, which would surface `Err(Stale)` projecting
            // to an empty-list (per the relaxed-projection arm in
            // `project_view_read_to_outcome`). Post-fix the relaxed
            // caller uses `read_view_allow_stale` so the projection
            // honors its docstring contract: "relaxed reads see
            // last-known-good data rather than the pre-NEW-1 empty
            // stub."
            let label_for_query = if label_hint.is_empty() {
                None
            } else {
                Some(label_hint.clone())
            };
            let query = benten_ivm::ViewQuery {
                label: label_for_query,
                limit: None,
                offset: None,
                ..Default::default()
            };
            let view_result = if opts.allow_stale {
                ivm.read_view_allow_stale(normalized, &query)
            } else {
                ivm.read_view(normalized, &query)
            };
            return project_view_read_to_outcome(self, view_result, view_id, opts.allow_stale);
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
    /// overload in the private `engine_caps` module.
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
    /// - [`EngineError::ViewLabelMismatch`] (R6-R3 r6-r3-ivm-1) when the
    ///   spec id matches one of the four canonical view ids whose
    ///   hand-written dispatch arm has hardcoded `input_pattern_label`
    ///   semantics (`capability_grants`, `version_current`,
    ///   `event_dispatch`, `governance_inheritance`) AND the supplied
    ///   `Label(...)` disagrees with the hardcoded label. Mirrors the
    ///   TS-DSL pre-napi rejection in
    ///   `packages/engine/src/views.ts::validateUserViewSpec` so direct
    ///   Rust callers + napi consumers that bypass the TS validator hit
    ///   the same fail-loud boundary rather than the silent-discard
    ///   foot-gun.
    /// - Backend errors from the underlying privileged Node write.
    pub fn register_user_view(&self, spec: UserViewSpec) -> Result<Cid, EngineError> {
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

        // R6-R3 r6-r3-ivm-1: canonical-id-vs-mismatched-label fail-loud.
        // Four canonical view ids (`capability_grants`, `version_current`,
        // `event_dispatch`, `governance_inheritance`) drive
        // `AlgorithmBView::for_id` arms whose hardcoded `input_pattern_label`
        // ignores the caller-supplied label. Pre-fix the engine silently
        // accepted the mismatch + the caller would observe a view filtered
        // on the WRONG label (the `register_view` step coerced the supplied
        // label into the catalog but the runtime dispatch arm bypassed it).
        // The TS-DSL `validateUserViewSpec` (`packages/engine/src/views.ts`)
        // mirrors this rejection at the pre-napi-boundary; this engine-side
        // guard is the authoritative boundary for direct Rust callers and
        // napi consumers that bypass the TS validator. Surfaced as
        // `E_VIEW_LABEL_MISMATCH` (catalog).
        if let UserViewInputPattern::Label(supplied_label) = spec.input_pattern()
            && let Some(hardcoded) = benten_ivm::algorithm_b::hardcoded_label_for_id(spec.id())
            && hardcoded != supplied_label.as_str()
        {
            return Err(EngineError::ViewLabelMismatch {
                view_id: spec.id().to_string(),
                expected_label: hardcoded.to_string(),
                got_label: supplied_label.clone(),
            });
        }

        if !self.ivm_enabled {
            return Err(EngineError::SubsystemDisabled { subsystem: "ivm" });
        }

        // Phase-3 G15-A: derive the kernel-side `LabelPattern` from the
        // spec's `UserViewInputPattern`. AnchorPrefix is now genuinely
        // a prefix selector (the Phase-2b silent-coerce-to-Label-equality
        // stub is RETIRED). The persisted Node still carries the
        // `input_pattern_label` string (= the pattern's stable shape per
        // `LabelPattern::as_label_str`) plus a sibling `input_pattern_kind`
        // discriminating Label vs AnchorPrefix on the persisted side.
        let label_pattern = match spec.input_pattern() {
            UserViewInputPattern::Label(l) => benten_ivm::LabelPattern::Exact(l.clone()),
            UserViewInputPattern::AnchorPrefix(prefix) => {
                benten_ivm::LabelPattern::AnchorPrefix(prefix.clone())
            }
        };
        let input_pattern_label = Some(label_pattern.as_label_str().to_string());

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

        // Phase-3 G15-A: register a live view instance with the IVM
        // subscriber via [`benten_ivm::Algorithm::register`]. The kernel
        // routes through the internal Strategy::A vs Strategy::B
        // dispatch router ([`benten_ivm::dispatch_for`]):
        //
        // - canonical view ids → inner kernel is one of the 5 hand-written
        //   Phase-1 views (per `ivm-disagree-1` they are inner kernels of
        //   Strategy::B, NOT Strategy::A baselines).
        // - non-canonical view ids → inner kernel is the generic
        //   `(label_pattern, projection)`-keyed kernel per `D-PHASE-3-28
        //   RESOLVED`.
        //
        // The Phase-2b silent fallback to `ContentListingView` for
        // non-canonical ids is RETIRED — non-canonical ids no longer
        // observe label-equality semantics for AnchorPrefix patterns.
        // We dedupe by view id so re-registering the same id is a no-op
        // at the subscriber level.
        if let Some(ivm) = self.ivm.as_ref() {
            let already_registered = ivm.view_ids().iter().any(|id| id == spec.id());
            if !already_registered {
                match benten_ivm::Algorithm::register(
                    spec.id(),
                    label_pattern.clone(),
                    benten_ivm::Projection::all_props(),
                ) {
                    Ok(view) => {
                        ivm.register_view(Box::new(view));
                    }
                    Err(benten_ivm::AlgorithmError::ViewLabelMismatch {
                        view_id,
                        expected_label,
                        ..
                    }) => {
                        // Defense-in-depth: the kernel-side fail-loud
                        // mirror should never fire here because the
                        // engine-side mismatch guard above already
                        // rejected this case. Surface the mismatch
                        // through the same engine error so callers see
                        // a single typed boundary (catalog code
                        // `E_VIEW_LABEL_MISMATCH`).
                        return Err(EngineError::ViewLabelMismatch {
                            view_id,
                            expected_label,
                            got_label: label_pattern.as_label_str().to_string(),
                        });
                    }
                }
            }
        }

        Ok(cid)
    }

    /// R6FP-Group-1 (r6-arch-2) deprecation alias —
    /// [`Self::create_user_view`] was renamed to
    /// [`Self::register_user_view`] to align with the engine's
    /// `register_*` lifecycle verb (matches `register_subgraph`,
    /// `register_subgraph_replace`, `register_subgraph_aggregate`,
    /// `register_crud`, `register_module_bytes`). The alias is held
    /// alive through one transition window so the TS-side rename in
    /// Group 2 does not break the build; the alias will be removed in
    /// a follow-up.
    ///
    /// # Errors
    ///
    /// See [`Self::register_user_view`].
    #[deprecated(
        note = "renamed to register_user_view (R6FP-G1 r6-arch-2); will be removed in a follow-up"
    )]
    pub fn create_user_view(&self, spec: UserViewSpec) -> Result<Cid, EngineError> {
        self.register_user_view(spec)
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

/// R6FP-tail (NEW-1) wire-through helper — project the live subscriber's
/// `read_view` outcome into a [`Outcome`] populated with hydrated Nodes.
///
/// Mirrors the projection pattern used by
/// [`crate::primitive_host::resolve_list_via_view_or_backend`] but lifts
/// it to all three [`benten_ivm::ViewResult`] variants so user-views with
/// `Current` / `Rules` shapes also produce observable Outcome lists.
///
/// - `None` (view not registered under this id at the subscriber): falls
///   back to the empty-list shape so the caller's downstream
///   canonical-id-whitelist branch can still fire (preserves the
///   pre-NEW-1 contract that handler tests pre-dated explicit user-view
///   registration).
/// - `Some(Err(ViewError::Stale))` (view became stale between
///   `view_is_stale` and `read_view`): converts to the same `IvmViewStale`
///   path the early stale check produces; relaxed reads collapse to empty
///   last-known-good.
/// - `Some(Err(other))` (pattern-mismatch / budget): treated as empty
///   list — the view honestly did not have a useful answer for this
///   query shape; the caller can re-issue with a different query.
fn project_view_read_to_outcome(
    engine: &Engine,
    result: Option<Result<benten_ivm::ViewResult, benten_ivm::ViewError>>,
    view_id: &str,
    allow_stale: bool,
) -> Result<Outcome, EngineError> {
    use benten_ivm::{ViewError, ViewResult};
    let backend = engine.backend();
    match result {
        None => Ok(Outcome {
            list: Some(Vec::new()),
            ..Outcome::default()
        }),
        Some(Ok(ViewResult::Cids(cids))) => {
            let mut out = Vec::with_capacity(cids.len());
            for cid in cids {
                if let Ok(Some(node)) = backend.get_node(&cid) {
                    out.push(node);
                }
            }
            Ok(Outcome {
                list: Some(out),
                ..Outcome::default()
            })
        }
        Some(Ok(ViewResult::Current(maybe_cid))) => {
            let mut out = Vec::new();
            if let Some(cid) = maybe_cid
                && let Ok(Some(node)) = backend.get_node(&cid)
            {
                out.push(node);
            }
            Ok(Outcome {
                list: Some(out),
                ..Outcome::default()
            })
        }
        Some(Ok(ViewResult::Rules(rules))) => {
            // Encode the rules map as a synthetic Node so consumers
            // walking `Outcome.list` see one Node carrying the rule
            // keys/values as properties. Phase-3 generalization may
            // introduce a typed Outcome.rules slot; until then the
            // synthetic-Node projection is the lowest-friction path
            // that matches the rest of the API surface (which is
            // Vec<Node>).
            let synthetic = Node::new(vec!["system:ivm:Rules".into()], rules.into_iter().collect());
            Ok(Outcome {
                list: Some(vec![synthetic]),
                ..Outcome::default()
            })
        }
        Some(Err(ViewError::Stale { .. })) => {
            if allow_stale {
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
        Some(Err(_)) => Ok(Outcome {
            list: Some(Vec::new()),
            ..Outcome::default()
        }),
    }
}

// `is_known_view_id` consultation note (Phase 2b G8-B partially closed
// the prior user-view-registration TODO that lived in engine.rs):
// user-view registration via [`Engine::create_user_view`] adds the
// user's view id into the IVM subscriber's `view_ids()` set. The
// `read_view_with` path above already consults the live subscriber
// FIRST (see the `if let Some(ivm) = ...` block) and only falls back
// to the canonical-id whitelist when the subscriber has no live view
// for the id — so user-registered views are observable through
// `read_view*` immediately after `create_user_view` returns.
//
// The user-view-registration TODO that previously lived in engine.rs
// ("replace with a per-view definition registration pulled from
// benten-ivm") has been deleted; the registration path is now
// subscriber-driven for the dynamic-registration path. The static
// whitelist of canonical ids remains for the 5 hand-written views
// which do not auto-register a live subscriber on engine open. Full
// removal of the whitelist waits on G8-A's Algorithm B port to expose
// a definition-registry method on the subscriber.
