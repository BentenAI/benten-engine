//! `ChangeProbe` — engine-level change-event observation handle.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). The probe holds a
//! reference to the engine's observed-events queue and drains events that
//! arrived after the probe was created.

use std::sync::Arc;

use benten_graph::ChangeEvent;

use crate::engine::EngineInner;

/// Probe for intercepting ChangeEvents in tests and in operator-side
/// consumers. Holds a reference to the engine's observed-events queue;
/// `drain` takes the events observed since the probe was created.
pub struct ChangeProbe {
    pub(crate) inner: Arc<EngineInner>,
    pub(crate) start_offset: u64,
    pub(crate) label_filter: Option<String>,
}

impl std::fmt::Debug for ChangeProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeProbe")
            .field("start_offset", &self.start_offset)
            .field("label_filter", &self.label_filter)
            .finish_non_exhaustive()
    }
}

impl ChangeProbe {
    /// Drain observed events. Call-once semantics: subsequent calls return
    /// empty unless more events have arrived in the meantime. Events observed
    /// before the probe was created are not returned — the probe's
    /// `start_offset` (captured at creation time) filters them out (fix for
    /// code-reviewer finding `g7-cr-7`).
    pub fn drain(&self) -> Vec<ChangeEvent> {
        let events = self.inner.drain_events_from(self.start_offset);
        let filter = self.label_filter.as_deref();
        if let Some(label) = filter {
            events
                .into_iter()
                .filter(|e| e.labels.iter().any(|l| l == label))
                .collect()
        } else {
            events
        }
    }
}
