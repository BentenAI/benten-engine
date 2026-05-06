//! Phase-3 G14-D wave-5a: handler-id-router seam (seq-major-8 LOAD-BEARING).
//!
//! Per the seq-major-8 R1 sequencing-systems lens finding, SUBSCRIBE
//! and EMIT need a typed routing seam that distinguishes "fan out to
//! every default consumer" from "route through a specific named
//! handler subgraph". Without this seam, EMIT and SUBSCRIBE produce
//! observable behavior that differs only by name; the router is what
//! makes them semantically distinct producer/consumer surfaces and
//! closes the structural cluster of producer/consumer drift findings
//! at the runtime layer (per stream-r1-2).
//!
//! ## Surface
//!
//! - [`HandlerRoute::DefaultFanOut`] — the pre-G14-D shape:
//!   broadcast to every registered consumer.
//! - [`HandlerRoute::Named`] — route the change event
//!   (SUBSCRIBE) or emit event (EMIT) THROUGH the named handler
//!   subgraph. The handler subgraph executes, and any side effects
//!   (probe writes, RESPOND values, etc.) are observably attributable
//!   to it.
//!
//! The routing decision lives at the engine layer; the eval-side
//! primitives consume an optional `handler` property on the
//! `OperationNode` to pick the variant. The eval primitive itself
//! does not invoke the handler — it leaves a pending-route hint that
//! the engine drains at the next dispatch boundary, then dispatches via
//! [`benten_eval::PrimitiveHost::call_handler`].
//!
//! ## Composes with
//!
//! - **§3.6 consumer-audit dimension** — both EMIT + SUBSCRIBE share
//!   the same routing seam; a future drift between them is structurally
//!   impossible because the variant lives in one place.
//! - **§3.6c mirror-precedent overshoot guard** — when a fix-pass
//!   mirrors EMIT's HandlerRoute precedent at a sibling primitive,
//!   the audit checks that ALL the precedent's consumers are wired.

use std::sync::Mutex;

/// Routing variant for SUBSCRIBE / EMIT producer events at the
/// engine layer. Default is [`Self::DefaultFanOut`] — the pre-G14-D
/// behavior; explicit [`Self::Named`] routes through a registered
/// handler subgraph by id.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HandlerRoute {
    /// Broadcast to every default consumer (the pre-G14-D shape).
    DefaultFanOut,
    /// Route through the named handler subgraph; only that
    /// handler's side effects fire — the default fan-out path is
    /// suppressed for this event.
    Named(String),
}

impl HandlerRoute {
    /// True iff this is a [`Self::Named`] routing decision.
    #[must_use]
    pub fn is_named(&self) -> bool {
        matches!(self, HandlerRoute::Named(_))
    }

    /// Borrow the handler id when this is [`Self::Named`].
    #[must_use]
    pub fn named_handler_id(&self) -> Option<&str> {
        match self {
            HandlerRoute::Named(id) => Some(id.as_str()),
            HandlerRoute::DefaultFanOut => None,
        }
    }
}

/// Engine-side routing log + counter surface. Production deployments
/// query [`Self::default_fan_out_count`] + [`Self::named_routes`] to
/// observe routing decisions; the test pin
/// `emit_handler_id_router_routing_observably_differs_from_default_fan_out_end_to_end`
/// asserts the difference end-to-end.
#[derive(Default)]
pub struct HandlerRouteLog {
    inner: Mutex<HandlerRouteLogInner>,
}

#[derive(Default)]
struct HandlerRouteLogInner {
    default_fan_out_count: u64,
    named_routes: Vec<(String, String)>,
}

impl std::fmt::Debug for HandlerRouteLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let g = self.inner.lock().expect("poisoned");
        f.debug_struct("HandlerRouteLog")
            .field("default_fan_out_count", &g.default_fan_out_count)
            .field("named_routes_len", &g.named_routes.len())
            .finish()
    }
}

impl HandlerRouteLog {
    /// Construct an empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a default fan-out routing decision.
    pub fn record_default_fan_out(&self) {
        let mut g = self.inner.lock().expect("poisoned");
        g.default_fan_out_count = g.default_fan_out_count.saturating_add(1);
    }

    /// Record a named-handler routing decision: `producer` is the
    /// producer name (e.g. `"emit:create_post"` or
    /// `"subscribe:/zone/posts"`), `handler_id` is the named handler
    /// the event was routed through.
    pub fn record_named(&self, producer: &str, handler_id: &str) {
        let mut g = self.inner.lock().expect("poisoned");
        g.named_routes
            .push((producer.to_string(), handler_id.to_string()));
    }

    /// Total events routed via [`HandlerRoute::DefaultFanOut`].
    #[must_use]
    pub fn default_fan_out_count(&self) -> u64 {
        let g = self.inner.lock().expect("poisoned");
        g.default_fan_out_count
    }

    /// Snapshot of every named-handler routing decision made so far,
    /// in dispatch order. `(producer, handler_id)` pairs.
    #[must_use]
    pub fn named_routes(&self) -> Vec<(String, String)> {
        let g = self.inner.lock().expect("poisoned");
        g.named_routes.clone()
    }

    /// Reset the log (test-only convenience).
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn reset(&self) {
        let mut g = self.inner.lock().expect("poisoned");
        g.default_fan_out_count = 0;
        g.named_routes.clear();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn handler_route_named_round_trip() {
        let r = HandlerRoute::Named("h_a".into());
        assert!(r.is_named());
        assert_eq!(r.named_handler_id(), Some("h_a"));
    }

    #[test]
    fn handler_route_default_is_named_false() {
        let r = HandlerRoute::DefaultFanOut;
        assert!(!r.is_named());
        assert_eq!(r.named_handler_id(), None);
    }

    #[test]
    fn handler_route_log_records_named_and_default_distinctly() {
        // stream-r1-2 LOAD-BEARING shape: routing observably differs
        // between Named + DefaultFanOut at the log surface.
        let log = HandlerRouteLog::new();
        log.record_default_fan_out();
        log.record_default_fan_out();
        log.record_named("emit:evt", "h_a");
        log.record_named("subscribe:/zone/posts", "h_b");
        assert_eq!(log.default_fan_out_count(), 2);
        assert_eq!(log.named_routes().len(), 2);
        assert_eq!(log.named_routes()[0], ("emit:evt".into(), "h_a".into()));
    }
}
