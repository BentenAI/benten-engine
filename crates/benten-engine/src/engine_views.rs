//! Change-stream probe + IVM view-read surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. Houses
//! `subscribe_change_events`, the test-only probe variants,
//! `change_event_count`, and the three view-read entry points
//! (`read_view`, `read_view_with`, `read_view_strict`,
//! `read_view_allow_stale`). Every method is a plain `impl Engine` item.

use std::sync::Arc;

use benten_caps::CapError;

use crate::change_probe::ChangeProbe;
use crate::engine::{Engine, is_known_view_id};
use crate::error::EngineError;
use crate::outcome::{Outcome, ReadViewOptions};

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
}
