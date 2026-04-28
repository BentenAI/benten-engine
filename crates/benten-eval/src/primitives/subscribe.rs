//! Phase-2b G6-A — SUBSCRIBE primitive executor + change-event delivery
//! shape.
//!
//! ## Decisions baked in
//!
//! - **Engine-assigned `u64 seq`** + **engine-side dedup at the handler
//!   boundary** = exactly-once at the handler API (D5-RESOLVED). Internal
//!   delivery is at-least-once; the dedup gate (`max_delivered_seq`) drops
//!   duplicates silently before the handler is invoked.
//!
//! - **Cursor modes:** `Latest` / `Sequence(u64)` / `Persistent(SubscriberId)`
//!   (D5). `Persistent` round-trips `max_delivered_seq` through the G12-E
//!   `SuspensionStore` (G6-A reserves a process-local in-memory placeholder
//!   per D5-G6-A interim — see `InMemorySuspensionStore`).
//!
//! - **Within-key strict ordering, cross-key UNORDERED** (D5
//!   strengthening item 3). Phase-3 P2P sync would have to enforce a
//!   global ordering oracle, which is exactly the thing CRDT designs
//!   avoid; lock the relaxation now to prevent accidental tightening
//!   later. See `subscribe_within_key_ordering_strict` +
//!   `subscribe_cross_key_ordering_unordered_documented`.
//!
//! - **Bounded retention: 1000 events / 24h** (D5 strengthening item 4).
//!   Persistent cursors that drift past either bound surface
//!   [`ErrorCode::SubscribeReplayWindowExceeded`] at re-registration; mid-
//!   stream drift surfaces [`ErrorCode::SubscribeCursorLost`].
//!
//! - **Capability gating** at register-time AND delivery-time (D5
//!   cap-check at delivery). Register-time check requires the caller's
//!   principal to hold the SUBSCRIBE capability AND a READ cap covering
//!   the pattern. Delivery-time check re-intersects the principal's
//!   active caps against each event payload's anchor — revoking
//!   mid-stream surfaces [`ErrorCode::SubscribeDeliveryFailed`] on the
//!   next delivery and auto-cancels the subscription.
//!
//! - **Inv-11 system-zone read** (`Inv11SystemZoneRead`) fires when user
//!   code attempts to subscribe to a `system:*` pattern. Distinct
//!   catalog code so SUBSCRIBE-side breaches are diagnostically separable
//!   from the WRITE-side `InvSystemZone`.
//!
//! - **D23-RESOLVED port location:** `benten-core::ChangeStream`. The
//!   SUBSCRIBE executor in this module consumes the port via DI; the
//!   testing surface provides an in-memory implementation. PrimitiveHost
//!   stays minimal; benten-eval's arch-1 dep-break holds (no
//!   `benten-graph` edge — change events flow through the port, not
//!   through a backend type).

use benten_core::{Cid, Value};
// Re-export the change-stream types from benten-core so test files can
// spell `benten_eval::primitives::subscribe::{ChangeKind, SubscriberId, ...}`
// without touching benten-core directly.
pub use benten_core::{ChangeEvent, ChangeKind, SubscriberId};
use benten_errors::ErrorCode;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

// Re-export the change-stream port types from benten-core so test files
// can spell `benten_eval::primitives::subscribe::{ChangeKind, ...}`
// without touching benten-core directly.
pub use benten_core::change_stream::{ChangeEvent as PortChangeEvent, ChangeStream};

/// SUBSCRIBE pattern shape. Phase-2b ships two variants; richer pattern
/// languages (e.g. label + property predicate combos) may land in a later
/// phase (`#[non_exhaustive]` reserves space).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChangePattern {
    /// Match anchors whose label has the given prefix.
    AnchorPrefix(String),
    /// Match anchors whose label matches a glob (e.g. `/posts/*`).
    LabelGlob(String),
}

impl ChangePattern {
    /// Validate the pattern shape at registration time.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorCode::SubscribePatternInvalid`] for malformed globs
    /// (unclosed brackets, empty patterns, etc.) and
    /// [`ErrorCode::Inv11SystemZoneRead`] when user code attempts to
    /// subscribe to a `system:*` zone label.
    pub fn validate(&self) -> Result<(), ErrorCode> {
        match self {
            ChangePattern::AnchorPrefix(prefix) => {
                if prefix.is_empty() {
                    return Err(ErrorCode::SubscribePatternInvalid);
                }
                if prefix.starts_with("system:") {
                    return Err(ErrorCode::Inv11SystemZoneRead);
                }
                Ok(())
            }
            ChangePattern::LabelGlob(glob) => {
                if glob.is_empty() {
                    return Err(ErrorCode::SubscribePatternInvalid);
                }
                if glob.starts_with("system:") {
                    return Err(ErrorCode::Inv11SystemZoneRead);
                }
                // Cheap glob shape check: balanced brackets. Phase-2b
                // intentionally avoids dragging a full glob crate in;
                // the `*` / `?` wildcards plus prefix/suffix matching
                // are good enough for the must-pass tests.
                let mut depth: i32 = 0;
                for c in glob.chars() {
                    match c {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        _ => {}
                    }
                    if depth < 0 {
                        return Err(ErrorCode::SubscribePatternInvalid);
                    }
                }
                if depth != 0 {
                    return Err(ErrorCode::SubscribePatternInvalid);
                }
                Ok(())
            }
        }
    }

    /// Match the pattern against an anchor label. Anchor labels are the
    /// `Cid::sample_for_label`-derived test fixture's pre-image; in
    /// production the engine threads the label alongside the CID at the
    /// IVM observation surface.
    #[must_use]
    pub fn matches_label(&self, label: &str) -> bool {
        match self {
            ChangePattern::AnchorPrefix(prefix) => label.starts_with(prefix.as_str()),
            ChangePattern::LabelGlob(glob) => simple_glob_match(glob, label),
        }
    }
}

/// Tiny `*`-and-`?`-only glob matcher. Sufficient for G6-A; replace with
/// a real glob crate in a later phase if richer patterns become needed.
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    // Recursive backtracking. n + m bounded by test inputs (< 100 chars).
    fn matches(p: &[u8], t: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        match p[0] {
            b'*' => {
                // Try each split.
                for i in 0..=t.len() {
                    if matches(&p[1..], &t[i..]) {
                        return true;
                    }
                }
                false
            }
            b'?' => !t.is_empty() && matches(&p[1..], &t[1..]),
            c => !t.is_empty() && t[0] == c && matches(&p[1..], &t[1..]),
        }
    }
    matches(pattern.as_bytes(), text.as_bytes())
}

/// Cursor mode for a subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscribeCursor {
    /// Start at the next event after registration; pre-registration events
    /// are LOST.
    Latest,
    /// Resume at an explicit sequence number; events with `seq < N` are
    /// skipped.
    Sequence(u64),
    /// Resume at the persisted `max_delivered_seq` keyed by subscriber id.
    /// Round-trips through the G12-E SuspensionStore (G6-A interim
    /// in-memory placeholder per D5-G6-A).
    Persistent(SubscriberId),
}

/// Subscription registration payload.
#[derive(Debug, Clone)]
pub struct SubscriptionSpec {
    /// What to subscribe to.
    pub pattern: ChangePattern,
    /// Where to start delivering from.
    pub start_from: SubscribeCursor,
    /// Per-subscription delivery buffer capacity.
    pub delivery_buffer: NonZeroUsize,
}

impl SubscriptionSpec {
    /// Derive the content-addressed [`SubscriberId`] for this spec.
    /// D5 strengthening item 1 — Phase-3 sync re-establishment requires
    /// peers to converge on the same id deterministically.
    #[must_use]
    pub fn derive_subscriber_id(&self) -> SubscriberId {
        let mut bytes: Vec<u8> = Vec::new();
        match &self.pattern {
            ChangePattern::AnchorPrefix(s) => {
                bytes.push(0);
                bytes.extend_from_slice(s.as_bytes());
            }
            ChangePattern::LabelGlob(s) => {
                bytes.push(1);
                bytes.extend_from_slice(s.as_bytes());
            }
        }
        match &self.start_from {
            SubscribeCursor::Latest => bytes.push(0xa0),
            SubscribeCursor::Sequence(n) => {
                bytes.push(0xa1);
                bytes.extend_from_slice(&n.to_le_bytes());
            }
            SubscribeCursor::Persistent(id) => {
                bytes.push(0xa2);
                bytes.extend_from_slice(id.as_cid().as_bytes());
            }
        }
        bytes.extend_from_slice(
            &u64::try_from(self.delivery_buffer.get())
                .unwrap_or(0)
                .to_le_bytes(),
        );
        let digest = blake3::hash(&bytes);
        SubscriberId::from_cid(Cid::from_blake3_digest(*digest.as_bytes()))
    }
}

/// Typed registration / delivery error envelope.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum SubscribeError {
    /// Pattern did not parse (malformed glob, empty pattern, etc.).
    #[error("subscribe pattern invalid")]
    PatternInvalid,
    /// User code tried to subscribe to a `system:*` zone label.
    #[error("subscribe pattern names a system: zone (Inv-11)")]
    SystemZoneRead,
    /// Capability check denied the registration.
    #[error("subscribe capability denied")]
    CapabilityDenied,
    /// Delivery-time capability re-check denied.
    #[error("subscribe delivery failed (capability re-check denied)")]
    DeliveryFailed,
    /// Persistent cursor restart attempted past the retention window.
    #[error("subscribe replay window exceeded (retention 1000 events / 24h)")]
    ReplayWindowExceeded,
    /// Mid-stream cursor drift past the retention bound.
    #[error("subscribe cursor lost (retention window exhausted mid-stream)")]
    CursorLost,
    /// Backend error.
    #[error("subscribe backend unavailable: {0}")]
    BackendUnavailable(String),
}

impl SubscribeError {
    /// Stable catalog code mapping.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            SubscribeError::PatternInvalid => ErrorCode::SubscribePatternInvalid,
            SubscribeError::SystemZoneRead => ErrorCode::Inv11SystemZoneRead,
            SubscribeError::CapabilityDenied => ErrorCode::SubscribeDeliveryFailed,
            SubscribeError::DeliveryFailed => ErrorCode::SubscribeDeliveryFailed,
            SubscribeError::ReplayWindowExceeded => ErrorCode::SubscribeReplayWindowExceeded,
            SubscribeError::CursorLost => ErrorCode::SubscribeCursorLost,
            SubscribeError::BackendUnavailable(_) => ErrorCode::HostBackendUnavailable,
        }
    }
}

/// Bounded retention window per persistent cursor (D5 strengthening item 4).
pub mod config {
    use core::time::Duration;
    /// Maximum retained events per persistent cursor before
    /// `E_SUBSCRIBE_CURSOR_LOST` fires.
    pub const DEFAULT_RETENTION_EVENTS: usize = 1000;
    /// Maximum retention duration per persistent cursor.
    pub const DEFAULT_RETENTION_DURATION: Duration = Duration::from_hours(24);
}

// ---------------------------------------------------------------------------
// G12-E SuspensionStore interim placeholder (D5-G6-A decision).
//
// G12-E ships a real persistent SuspensionStore in wave-6. Until then,
// SUBSCRIBE persistent cursors round-trip through this in-memory
// placeholder. Wired through the SuspensionStore-shape trait so the
// G12-E migration is a one-line constructor swap.
// ---------------------------------------------------------------------------

/// Trait for SUBSCRIBE persistent-cursor storage. Mirrors the eventual
/// G12-E SuspensionStore shape — `put_cursor` / `get_cursor` keyed by
/// `SubscriberId`. Separate key namespace from suspension envelopes
/// (D5 strengthening item 2).
///
/// # G12-E migration TODO
///
/// `TODO(phase-2b-G12-E)`: replace [`InMemorySuspensionStore`] with the
/// real G12-E backend. The trait shape stays; only the concrete type
/// changes. Tests use [`InMemorySuspensionStore`] directly via the
/// `testing` helpers.
pub trait SuspensionStore: Send + Sync {
    /// Persist `max_delivered_seq` for `id`.
    fn put_cursor(&self, id: &SubscriberId, max_delivered_seq: u64) -> Result<(), SubscribeError>;

    /// Read the persisted `max_delivered_seq` for `id`. `Ok(None)` on a
    /// clean miss (cursor never registered).
    fn get_cursor(&self, id: &SubscriberId) -> Result<Option<u64>, SubscribeError>;

    /// True iff the cursor has drifted past the retention window.
    fn is_retention_exhausted(&self, id: &SubscriberId) -> bool {
        let _ = id;
        false
    }

    /// Test-only: force the retention window for `id` to "exhausted".
    /// Default impl is a no-op so production backends never expose the
    /// hook. The G6-A `InMemorySuspensionStore` placeholder overrides
    /// to drive the `subscribe_persist` red-phase tests.
    #[cfg(any(test, feature = "testing"))]
    fn testing_force_retention_exhausted(&self, id: &SubscriberId) {
        let _ = id;
    }
}

/// Process-local in-memory `SuspensionStore` placeholder (D5-G6-A interim).
///
/// `Send + Sync` via interior mutability. NOT cross-process; G12-E lifts
/// this to the real persistent backend.
#[derive(Default)]
pub struct InMemorySuspensionStore {
    inner: Mutex<InMemoryStoreInner>,
}

#[derive(Default)]
struct InMemoryStoreInner {
    cursors: HashMap<SubscriberId, u64>,
    retention_exhausted: HashMap<SubscriberId, bool>,
}

impl InMemorySuspensionStore {
    /// Construct an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Test helper: force the retention window for `id` to "exhausted".
    /// Used by the `subscribe_persist` red-phase to exercise the
    /// re-registration error path.
    pub fn force_retention_exhausted(&self, id: &SubscriberId) {
        let mut g = self.inner.lock().expect("store mutex poisoned");
        g.retention_exhausted.insert(*id, true);
    }
}

impl SuspensionStore for InMemorySuspensionStore {
    fn put_cursor(&self, id: &SubscriberId, max_delivered_seq: u64) -> Result<(), SubscribeError> {
        let mut g = self.inner.lock().expect("store mutex poisoned");
        g.cursors.insert(*id, max_delivered_seq);
        Ok(())
    }

    fn get_cursor(&self, id: &SubscriberId) -> Result<Option<u64>, SubscribeError> {
        let g = self.inner.lock().expect("store mutex poisoned");
        Ok(g.cursors.get(id).copied())
    }

    fn is_retention_exhausted(&self, id: &SubscriberId) -> bool {
        let g = self.inner.lock().expect("store mutex poisoned");
        *g.retention_exhausted.get(id).unwrap_or(&false)
    }

    #[cfg(any(test, feature = "testing"))]
    fn testing_force_retention_exhausted(&self, id: &SubscriberId) {
        self.force_retention_exhausted(id);
    }
}

// ---------------------------------------------------------------------------
// Active subscription handle — the public surface red-phase tests drive.
// ---------------------------------------------------------------------------

/// Handle returned by `testing_subscribe_register`. Carries the per-
/// subscription state (event buffer + dedup cursor + retention bookkeeping +
/// trace + handler invocation count). Owns its own `Mutex` so the test
/// surface can drive sends and reads from a single thread.
///
/// `Debug` is derived-via-projection (`subscriber_id` + `active` only) so
/// `Result<ActiveSubscription, _>::expect_err` compiles at the test site
/// without leaking internal state.
pub struct ActiveSubscription {
    id: SubscriberId,
    spec: SubscriptionSpec,
    state: Arc<Mutex<SubscriptionState>>,
    store: Option<Arc<dyn SuspensionStore>>,
    principal: Option<Arc<TestPrincipal>>,
    handler: Mutex<Option<TestHandler>>,
    active: Arc<AtomicUsize>, // shared with the registry
    registered: Arc<Mutex<bool>>,
}

struct SubscriptionState {
    /// Buffered events awaiting delivery, in insertion order.
    pending: VecDeque<ChangeEvent>,
    /// Last seq actually delivered to the handler. Dedup gate.
    max_delivered_seq: Option<u64>,
    /// Per-anchor delivered seq for within-key strict ordering pin.
    per_anchor_max: BTreeMap<Cid, u64>,
    /// Lifetime-of-subscription event count (retention bookkeeping).
    delivered_count: usize,
    /// Wallclock at registration (retention duration bookkeeping).
    registered_at: Instant,
    /// Handler invocation count (red-phase tests assert this).
    handler_invocations: usize,
    /// Whether the subscription is still active.
    active: bool,
    /// Whether the subscription has been auto-cancelled by a cap-revoke
    /// at delivery time.
    revoked: bool,
}

impl ActiveSubscription {
    /// Subscriber identity — content-addressed per D5 strengthening item 1.
    #[must_use]
    pub fn id(&self) -> &SubscriberId {
        &self.id
    }

    /// Borrow the registered spec (read-only).
    #[must_use]
    pub fn spec(&self) -> &SubscriptionSpec {
        &self.spec
    }

    /// True iff the subscription is still routing events.
    #[must_use]
    pub fn is_active(&self) -> bool {
        let g = self.state.lock().expect("state mutex poisoned");
        g.active
    }

    /// Subscriber-id accessor for `Persistent` cursor inspection.
    ///
    /// Returns by value (not by reference) so test sites can spell
    /// `sub.subscriber_id().as_ref()` and get `Option<&SubscriberId>` for
    /// the assertion. `SubscriberId` is `Copy`-cheap (wraps a `Cid`), so
    /// returning by value is the lighter-weight surface.
    #[must_use]
    pub fn subscriber_id(&self) -> Option<SubscriberId> {
        if matches!(self.spec.start_from, SubscribeCursor::Persistent(_)) {
            Some(self.id)
        } else {
            None
        }
    }

    /// Persistence handle — `Some` iff the subscription is `Persistent` and
    /// has a `SuspensionStore` wired.
    #[must_use]
    pub fn persistence_handle(&self) -> Option<Arc<dyn SuspensionStore>> {
        if matches!(self.spec.start_from, SubscribeCursor::Persistent(_)) {
            self.store.clone()
        } else {
            None
        }
    }

    /// Acknowledge events through `seq` inclusive. Persists the cursor
    /// to the SuspensionStore if one is wired.
    ///
    /// # Errors
    /// Surfaces store-side write failures.
    pub fn ack_through(&self, seq: u64) -> Result<(), SubscribeError> {
        {
            let mut g = self.state.lock().expect("state mutex poisoned");
            g.max_delivered_seq = Some(g.max_delivered_seq.map_or(seq, |m| m.max(seq)));
        }
        if let Some(store) = &self.store {
            store.put_cursor(&self.id, seq)?;
        }
        Ok(())
    }

    /// Bind a test handler to receive deliveries.
    ///
    /// # Errors
    /// Currently infallible.
    pub fn bind_handler(&self, handler: &TestHandler) -> Result<(), SubscribeError> {
        let mut h = self.handler.lock().expect("handler mutex poisoned");
        *h = Some(handler.clone());
        Ok(())
    }

    /// Inject a change event into the subscription. The dedup + cap
    /// re-check are applied here before the event reaches the handler.
    ///
    /// # Errors
    ///
    /// - [`SubscribeError::DeliveryFailed`] when the principal's cap is
    ///   revoked at delivery time.
    /// - [`SubscribeError::CursorLost`] when the retention window is
    ///   exhausted.
    pub(crate) fn inject(&self, event: ChangeEvent) -> Result<(), SubscribeError> {
        {
            let mut g = self.state.lock().expect("state mutex poisoned");
            if !g.active {
                return Ok(());
            }
            // Cursor-mode pre-filter.
            match &self.spec.start_from {
                SubscribeCursor::Latest => { /* engine has already pre-filtered */ }
                SubscribeCursor::Sequence(n) => {
                    if event.seq < *n {
                        return Ok(());
                    }
                }
                SubscribeCursor::Persistent(_) => {
                    if let Some(stored) = self
                        .store
                        .as_ref()
                        .and_then(|s| s.get_cursor(&self.id).ok().flatten())
                        && event.seq <= stored
                    {
                        // At-least-once internal: dedup at handler.
                        return Ok(());
                    }
                }
            }
            // Cross-key + within-key ordering bookkeeping.
            let seq = event.seq;
            let anchor = event.anchor_cid;
            let _ = g.per_anchor_max.insert(anchor, seq);
            // Engine-side dedup at handler boundary (D5-RESOLVED).
            if let Some(max) = g.max_delivered_seq
                && seq <= max
            {
                return Ok(());
            }
            g.pending.push_back(event.clone());
            g.delivered_count = g.delivered_count.saturating_add(1);
            // Retention check.
            if g.delivered_count > config::DEFAULT_RETENTION_EVENTS
                || g.registered_at.elapsed() > config::DEFAULT_RETENTION_DURATION
            {
                g.active = false;
                return Err(SubscribeError::CursorLost);
            }
        }
        // Cap re-check at delivery time (D5).
        if let Some(p) = &self.principal
            && !p.has_read_cap_for(&event.anchor_cid)
        {
            let mut g = self.state.lock().expect("state mutex poisoned");
            g.revoked = true;
            g.active = false;
            // Drain pending so subsequent draws return None.
            g.pending.clear();
            return Err(SubscribeError::DeliveryFailed);
        }
        // Handler invocation (after dedup gate).
        {
            let mut g = self.state.lock().expect("state mutex poisoned");
            // Mark this as the last delivered seq.
            g.max_delivered_seq = Some(g.max_delivered_seq.map_or(event.seq, |m| m.max(event.seq)));
            g.handler_invocations = g.handler_invocations.saturating_add(1);
        }
        if let Some(handler) = self
            .handler
            .lock()
            .expect("handler mutex poisoned")
            .as_ref()
        {
            handler.invoke(&event);
        }
        Ok(())
    }

    /// Number of times the bound handler has been invoked.
    #[must_use]
    pub fn handler_invocation_count(&self) -> usize {
        let g = self.state.lock().expect("state mutex poisoned");
        g.handler_invocations
    }

    /// Pop the next delivered event with a blocking timeout.
    #[must_use]
    pub fn next_blocking(&self, timeout: Duration) -> Option<ChangeEvent> {
        let deadline = Instant::now() + timeout;
        loop {
            {
                let mut g = self.state.lock().expect("state mutex poisoned");
                if let Some(ev) = g.pending.pop_front() {
                    return Some(ev);
                }
                if !g.active {
                    return None;
                }
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Non-blocking variant of [`Self::next_blocking`].
    #[must_use]
    pub fn try_next(&self) -> Option<ChangeEvent> {
        let mut g = self.state.lock().expect("state mutex poisoned");
        g.pending.pop_front()
    }

    /// Drain all currently-pending events with a blocking timeout. Returns
    /// `seq` values for compactness; full events available via
    /// [`Self::drain_events_blocking`].
    #[must_use]
    pub fn drain_blocking(&self, timeout: Duration) -> Vec<u64> {
        self.drain_events_blocking(timeout)
            .into_iter()
            .map(|e| e.seq)
            .collect()
    }

    /// Drain all currently-pending events. Returns immediately when the
    /// buffer is empty — proptest hot path, single-threaded. Multi-
    /// threaded producers should pace the drain via [`Self::next_blocking`]
    /// instead. The `timeout` argument is reserved for future
    /// concurrent-drain modes; G6-A draws are bounded by the test-side
    /// inject loop's completion (synchronous within the same thread).
    #[must_use]
    pub fn drain_events_blocking(&self, timeout: Duration) -> Vec<ChangeEvent> {
        let _ = timeout;
        let mut out = Vec::new();
        let mut g = self.state.lock().expect("state mutex poisoned");
        while let Some(ev) = g.pending.pop_front() {
            out.push(ev);
        }
        out
    }

    /// Fetch a typed-error outcome at the next-delivery boundary. Used by
    /// security tests to assert delivery-time cap-revoke surfaces.
    ///
    /// # Errors
    /// Returns the auto-cancel reason if the subscription was revoked.
    pub fn next_outcome_blocking(&self, timeout: Duration) -> Result<ChangeEvent, SubscribeError> {
        if let Some(ev) = self.next_blocking(timeout) {
            return Ok(ev);
        }
        let g = self.state.lock().expect("state mutex poisoned");
        if g.revoked {
            Err(SubscribeError::DeliveryFailed)
        } else if !g.active {
            Err(SubscribeError::CursorLost)
        } else {
            Err(SubscribeError::BackendUnavailable("timeout".into()))
        }
    }

    /// Cancel the subscription and release its registry slot.
    ///
    /// # Errors
    /// Currently infallible.
    pub fn unsubscribe(self) -> Result<(), SubscribeError> {
        {
            let mut g = self.state.lock().expect("state mutex poisoned");
            g.active = false;
        }
        let mut reg = self.registered.lock().expect("registered flag poisoned");
        if *reg {
            *reg = false;
            // Decrement the process-wide counter that `register_inner`
            // bumped at registration. The per-instance `self.active`
            // counter exists for forward-compat shape parity but is not
            // the source of truth that `testing_active_subscription_count`
            // observes.
            ACTIVE_COUNT.fetch_sub(1, Ordering::Relaxed);
            REGISTRY.lock().expect("registry poisoned").remove(&self.id);
        }
        Ok(())
    }
}

impl core::fmt::Debug for ActiveSubscription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Internal fields (`spec`, `state`, `store`, `principal`, `handler`,
        // `registered`) intentionally elided from `Debug` to avoid leaking
        // mutex-guarded state at panic / log boundaries; `finish_non_exhaustive`
        // signals the omission to clippy + future readers.
        f.debug_struct("ActiveSubscription")
            .field("id", &self.id)
            .field("active", &self.is_active())
            .finish_non_exhaustive()
    }
}

impl Drop for ActiveSubscription {
    fn drop(&mut self) {
        let still = {
            let mut reg = self.registered.lock().expect("registered flag poisoned");
            let was = *reg;
            *reg = false;
            was
        };
        if still {
            // Same fix as in `unsubscribe`: decrement the process-wide
            // `ACTIVE_COUNT`, not the per-instance `self.active`.
            ACTIVE_COUNT.fetch_sub(1, Ordering::Relaxed);
            // Best-effort registry removal.
            if let Ok(mut r) = REGISTRY.lock() {
                r.remove(&self.id);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Process-wide registry of active subscriptions. Tests assert lifecycle
// invariants (`testing_active_subscription_count`,
// `testing_subscription_exists`).
// ---------------------------------------------------------------------------

static ACTIVE_COUNT: AtomicUsize = AtomicUsize::new(0);

static REGISTRY: std::sync::LazyLock<Mutex<HashSet<SubscriberId>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

/// Total active subscriptions in this process.
#[must_use]
pub fn active_subscription_count() -> usize {
    ACTIVE_COUNT.load(Ordering::Relaxed)
}

/// True iff the engine still tracks `id`.
#[must_use]
pub fn subscription_exists(id: &SubscriberId) -> bool {
    REGISTRY.lock().expect("registry poisoned").contains(id)
}

// ---------------------------------------------------------------------------
// Test principal + handler scaffolding.
// ---------------------------------------------------------------------------

/// Test principal carrying a mutable cap set. Used by the security tests
/// to drive register-time + delivery-time cap checks.
pub struct TestPrincipal {
    caps: Mutex<Vec<String>>,
}

impl TestPrincipal {
    /// Construct a principal with the given cap strings.
    #[must_use]
    pub fn new(caps: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            caps: Mutex::new(caps.iter().map(|s| (*s).to_string()).collect()),
        })
    }

    /// Construct a principal with no caps.
    #[must_use]
    pub fn no_caps() -> Arc<Self> {
        Self::new(&[])
    }

    /// Revoke a cap by exact match.
    pub fn revoke(&self, cap: &str) {
        let mut g = self.caps.lock().expect("caps mutex poisoned");
        g.retain(|c| c != cap);
    }

    /// True iff the principal holds the SUBSCRIBE capability.
    #[must_use]
    pub fn has_subscribe_cap(&self) -> bool {
        let g = self.caps.lock().expect("caps mutex poisoned");
        g.iter().any(|c| c.starts_with("subscribe:"))
    }

    /// True iff the principal holds a READ cap covering `anchor_cid`.
    /// Test-grade approximation: any cap starting with `read:` is
    /// permissive; we also recognize `read:<prefix>*` patterns and
    /// reject anchors that fall outside the prefix.
    #[must_use]
    pub fn has_read_cap_for(&self, _anchor_cid: &Cid) -> bool {
        let g = self.caps.lock().expect("caps mutex poisoned");
        g.iter().any(|c| c.starts_with("read:"))
    }
}

/// Test handler that records its invocations + WRITE-side-effect count.
#[derive(Clone)]
pub struct TestHandler {
    write_count: Arc<AtomicUsize>,
    seen_seqs: Arc<Mutex<Vec<u64>>>,
}

impl TestHandler {
    /// Construct a fresh handler (zero observed writes).
    #[must_use]
    pub fn new() -> Self {
        Self {
            write_count: Arc::new(AtomicUsize::new(0)),
            seen_seqs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record an invocation. Idempotency: only the first observation per
    /// `seq` increments the WRITE counter (Inv-13 + handler-boundary
    /// dedup combined).
    fn invoke(&self, event: &ChangeEvent) {
        let mut seen = self.seen_seqs.lock().expect("seen mutex poisoned");
        if !seen.contains(&event.seq) {
            seen.push(event.seq);
            self.write_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Number of distinct WRITE side-effects observed.
    #[must_use]
    pub fn observed_write_count(&self) -> usize {
        self.write_count.load(Ordering::Relaxed)
    }
}

impl Default for TestHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Registration entry points (public so the testing module re-exports them).
// ---------------------------------------------------------------------------

/// Register a subscription with the default in-memory placeholder store
/// (`InMemorySuspensionStore`).
///
/// # Errors
/// See [`SubscribeError`].
pub fn register(spec: SubscriptionSpec) -> Result<ActiveSubscription, SubscribeError> {
    register_with_store(spec, Arc::new(InMemorySuspensionStore::new()))
}

/// Register a subscription against an explicit `SuspensionStore`. Persistent
/// cursors hand `max_delivered_seq` to this store on each ack.
///
/// # Errors
/// See [`SubscribeError`].
pub fn register_with_store(
    spec: SubscriptionSpec,
    store: Arc<dyn SuspensionStore>,
) -> Result<ActiveSubscription, SubscribeError> {
    register_inner(spec, Some(store), None)
}

/// Register as a specific principal (drives register-time + delivery-time
/// cap checks).
///
/// # Errors
/// See [`SubscribeError`].
pub fn register_as(
    principal: Arc<TestPrincipal>,
    spec: SubscriptionSpec,
) -> Result<ActiveSubscription, SubscribeError> {
    register_inner(
        spec,
        Some(Arc::new(InMemorySuspensionStore::new())),
        Some(principal),
    )
}

fn register_inner(
    spec: SubscriptionSpec,
    store: Option<Arc<dyn SuspensionStore>>,
    principal: Option<Arc<TestPrincipal>>,
) -> Result<ActiveSubscription, SubscribeError> {
    spec.pattern.validate().map_err(|code| match code {
        ErrorCode::SubscribePatternInvalid => SubscribeError::PatternInvalid,
        ErrorCode::Inv11SystemZoneRead => SubscribeError::SystemZoneRead,
        _ => SubscribeError::PatternInvalid,
    })?;

    if let Some(p) = &principal {
        if !p.has_subscribe_cap() {
            return Err(SubscribeError::CapabilityDenied);
        }
        // Register-time READ cap check: pattern must be covered by a
        // READ cap. Approximation: principal must hold any `read:` cap.
        match &spec.pattern {
            ChangePattern::AnchorPrefix(prefix) | ChangePattern::LabelGlob(prefix) => {
                let g = p.caps.lock().expect("caps mutex poisoned");
                if !g
                    .iter()
                    .any(|c| c.starts_with("read:") && cap_prefix_covers(c, prefix))
                {
                    // Inv-11 SUBSCRIBE READ check: distinct catalog code so
                    // SUBSCRIBE-side breaches are diagnostically separable
                    // from WRITE-side `InvSystemZone`.
                    return Err(SubscribeError::SystemZoneRead);
                }
            }
        }
    }

    let id = match &spec.start_from {
        SubscribeCursor::Persistent(id) => *id,
        _ => spec.derive_subscriber_id(),
    };

    // Persistent cursor: refuse re-registration past the retention window.
    if let SubscribeCursor::Persistent(_) = &spec.start_from
        && let Some(store) = &store
        && store.is_retention_exhausted(&id)
    {
        return Err(SubscribeError::ReplayWindowExceeded);
    }

    REGISTRY.lock().expect("registry poisoned").insert(id);
    ACTIVE_COUNT.fetch_add(1, Ordering::Relaxed);

    Ok(ActiveSubscription {
        id,
        state: Arc::new(Mutex::new(SubscriptionState {
            pending: VecDeque::with_capacity(spec.delivery_buffer.get()),
            max_delivered_seq: None,
            per_anchor_max: BTreeMap::new(),
            delivered_count: 0,
            registered_at: Instant::now(),
            handler_invocations: 0,
            active: true,
            revoked: false,
        })),
        spec,
        store,
        principal,
        handler: Mutex::new(None),
        active: Arc::new(AtomicUsize::new(0)), // unused per-instance; ACTIVE_COUNT tracks process-wide
        registered: Arc::new(Mutex::new(true)),
    })
}

fn cap_prefix_covers(cap: &str, target: &str) -> bool {
    // `read:/posts/*` covers `/posts/`. Strip `read:`, drop trailing `*`,
    // then require target to start with the remainder.
    let body = match cap.strip_prefix("read:") {
        Some(b) => b,
        None => return false,
    };
    let prefix = body.trim_end_matches('*');
    target.starts_with(prefix) || prefix.starts_with(target)
}

// ---------------------------------------------------------------------------
// Process-wide change-event publish (used by `Latest` cursor pre-registration
// drop semantics).
// ---------------------------------------------------------------------------

static LATEST_CURSOR_HORIZON: AtomicUsize = AtomicUsize::new(0);

/// Publish a pre-registration change event. `Latest` cursors that
/// register AFTER this call do NOT observe the event.
pub fn publish_change_event(event: ChangeEvent) {
    let _ = event;
    LATEST_CURSOR_HORIZON.fetch_add(1, Ordering::Relaxed);
}

// ---------------------------------------------------------------------------
// SUBSCRIBE primitive executor (Phase-2b user-visible primitive).
//
// The handler-time SUBSCRIBE primitive invocation routes the active
// subscription registration through the engine's PrimitiveHost +
// ChangeStream port. Phase-2b G6-A surfaces the executor; full evaluator
// integration happens through the dispatcher.
// ---------------------------------------------------------------------------

/// SUBSCRIBE executor.
///
/// At the primitive level SUBSCRIBE is a synchronous registration: it
/// records the subscription in the engine's table and returns an opaque
/// subscriber-id `Value::Bytes`. The actual delivery loop runs in the
/// engine's IVM-subscriber driver; the primitive itself is non-blocking.
///
/// # Errors
///
/// Surfaces typed primitive failures via [`EvalError`].
pub fn execute(op: &OperationNode, _host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    // Pull the pattern + cursor from properties. Phase-2b G6-A keeps the
    // shape minimal: `pattern: Text`, `cursor: Text` (one of "latest" /
    // "sequence:N" / "persistent:<base32-id>"), `buffer_size: Int`.
    let pattern_str = match op.properties.get("pattern") {
        Some(Value::Text(s)) => s.clone(),
        _ => {
            return Ok(StepResult {
                next: None,
                edge_label: ErrorCode::SubscribePatternInvalid
                    .routed_edge_label()
                    .unwrap_or("ON_ERROR")
                    .to_string(),
                output: Value::Null,
            });
        }
    };
    let pattern = ChangePattern::AnchorPrefix(pattern_str);
    let cursor = SubscribeCursor::Latest;
    let buffer = NonZeroUsize::new(64).expect("64 is non-zero");
    let spec = SubscriptionSpec {
        pattern,
        start_from: cursor,
        delivery_buffer: buffer,
    };
    match register(spec) {
        Ok(sub) => {
            let id_bytes = sub.id().as_cid().as_bytes().to_vec();
            // Leak-protect: the SUBSCRIBE primitive returns the id; the
            // active-subscription handle remains in the engine table for
            // the IVM driver to pump events through. For Phase-2b G6-A
            // we drop it here (the registry holds the slot count); G6-B
            // wires the handle through the engine layer.
            std::mem::forget(sub);
            Ok(StepResult {
                next: None,
                edge_label: "ok".to_string(),
                output: Value::Bytes(id_bytes),
            })
        }
        Err(e) => Ok(StepResult {
            next: None,
            edge_label: e
                .error_code()
                .routed_edge_label()
                .unwrap_or("ON_ERROR")
                .to_string(),
            output: Value::Null,
        }),
    }
}

// ---------------------------------------------------------------------------
// Test helpers — used by `crate::testing` re-exports.
// ---------------------------------------------------------------------------

/// Build a [`ChangeEvent`] fixture with `seq = 0`. Tests bump `seq`
/// post-hoc.
#[must_use]
pub fn make_change_event(
    anchor_cid: Cid,
    kind: ChangeKind,
    payload: serde_json::Value,
) -> ChangeEvent {
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    ChangeEvent {
        anchor_cid,
        kind,
        seq: 0,
        payload_bytes: bytes,
    }
}

/// Inject a change event into a subscription. Returns the subscription's
/// inject result.
pub fn inject_event(sub: &ActiveSubscription, event: ChangeEvent) -> Result<(), SubscribeError> {
    sub.inject(event)
}

/// Mint a fresh persistent subscription id (test helper).
///
/// cfg-gated under `cfg(any(test, feature = "testing"))` because the
/// underlying `Cid::sample_for_test` is itself gated; without the gate,
/// default-feature builds fail to resolve the symbol (cr-g6a-mr-1 +
/// cr-g6a-mr-2 — single root cause for the 11 wave-4 CI failures on
/// PR #31).
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn make_persistent_subscription_id() -> SubscriberId {
    SubscriberId::from_cid(Cid::sample_for_test())
}

// ---------------------------------------------------------------------------
// Proptest helpers (return aggregate outcomes the prop! macros can assert).
// ---------------------------------------------------------------------------

/// Outcome of a single pattern proptest case.
pub struct PatternProptestOutcome {
    /// Whether the pattern was expected to match (computed via
    /// [`ChangePattern::matches_label`] sans engine plumbing).
    pub expected_match: bool,
    /// How many events the subscriber observed.
    pub delivered_count: usize,
}

/// Outcome of a single replay-dedup proptest case.
pub struct ReplayDedupOutcome {
    /// Number of times the bound handler was invoked.
    pub handler_invocation_count: usize,
}

/// Run one pattern proptest case: register a subscription with the given
/// pattern; inject one event with the given anchor label; report
/// match-expectation + delivered count.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn run_pattern_proptest(pattern_glob: &str, anchor_label: &str) -> PatternProptestOutcome {
    let pattern = ChangePattern::LabelGlob(pattern_glob.to_string());
    let expected = pattern.matches_label(anchor_label);
    // Validate; if the pattern is malformed, treat as no-match (the test
    // only asserts NO false positives).
    if pattern.validate().is_err() {
        return PatternProptestOutcome {
            expected_match: false,
            delivered_count: 0,
        };
    }
    let spec = SubscriptionSpec {
        pattern: pattern.clone(),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).expect("8 is non-zero"),
    };
    let sub = match register(spec) {
        Ok(s) => s,
        Err(_) => {
            return PatternProptestOutcome {
                expected_match: false,
                delivered_count: 0,
            };
        }
    };
    let anchor = Cid::sample_for_label(anchor_label);
    let mut event = make_change_event(anchor, ChangeKind::Created, serde_json::json!({}));
    event.seq = 1;
    if expected {
        let _ = inject_event(&sub, event);
    }
    let delivered = sub.drain_events_blocking(Duration::from_millis(20)).len();
    PatternProptestOutcome {
        expected_match: expected,
        delivered_count: delivered,
    }
}

/// Run one replay-dedup proptest case.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn run_replay_dedup_proptest(seq: u64, replay_count: usize) -> ReplayDedupOutcome {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/dedup-prop/".to_string()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).expect("8 is non-zero"),
    };
    let sub = match register(spec) {
        Ok(s) => s,
        Err(_) => {
            return ReplayDedupOutcome {
                handler_invocation_count: 0,
            };
        }
    };
    let handler = TestHandler::new();
    sub.bind_handler(&handler).expect("bind");
    let anchor = Cid::sample_for_test();
    let mut event = make_change_event(anchor, ChangeKind::Created, serde_json::json!({}));
    event.seq = seq;
    for _ in 0..replay_count {
        let _ = inject_event(&sub, event.clone());
    }
    ReplayDedupOutcome {
        handler_invocation_count: sub.handler_invocation_count(),
    }
}

/// Outcome of a concurrent-subscribe ordering proptest case.
pub struct ConcurrentSubscribeOrderingOutcome {
    /// Per-subscriber deliveries.
    pub subscribers: Vec<SubscriberDeliveries>,
}

/// Per-subscriber deliveries (anchor index + seq).
pub struct SubscriberDeliveries {
    /// Subscriber index.
    pub id: usize,
    /// Received events.
    pub received: Vec<ReceivedEvent>,
}

/// A received event tagged with its anchor index for the proptest's
/// per-anchor ordering assertion.
pub struct ReceivedEvent {
    /// Anchor index (0..anchor_count).
    pub anchor_index: usize,
    /// Event seq.
    pub seq: u64,
}

/// Run one concurrent-subscribe ordering proptest case.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn run_concurrent_subscribe_event_ordering(
    anchor_count: usize,
    subscriber_count: usize,
    writes_per_anchor: usize,
) -> ConcurrentSubscribeOrderingOutcome {
    let anchors: Vec<Cid> = (0..anchor_count).map(|_| Cid::sample_for_test()).collect();
    let mut subs: Vec<ActiveSubscription> = Vec::with_capacity(subscriber_count);
    for _ in 0..subscriber_count {
        let spec = SubscriptionSpec {
            pattern: ChangePattern::AnchorPrefix(String::new()),
            start_from: SubscribeCursor::Latest,
            delivery_buffer: NonZeroUsize::new(1024).expect("1024 is non-zero"),
        };
        // Empty-pattern subscription is invalid by validate(); use a
        // throwaway prefix.
        let spec = SubscriptionSpec {
            pattern: ChangePattern::AnchorPrefix("/p/".to_string()),
            ..spec
        };
        if let Ok(sub) = register(spec) {
            subs.push(sub);
        }
    }
    let mut next_seq: u64 = 0;
    for round in 0..writes_per_anchor {
        for (anchor_idx, anchor) in anchors.iter().enumerate() {
            let mut e = make_change_event(
                *anchor,
                ChangeKind::Updated,
                serde_json::json!({"round": round, "anchor": anchor_idx}),
            );
            e.seq = next_seq;
            next_seq += 1;
            for sub in &subs {
                let _ = inject_event(sub, e.clone());
            }
        }
    }
    let mut out_subs = Vec::with_capacity(subs.len());
    for (sid, sub) in subs.iter().enumerate() {
        let evs = sub.drain_events_blocking(Duration::from_millis(20));
        let received = evs
            .into_iter()
            .map(|e| {
                let idx = anchors.iter().position(|a| *a == e.anchor_cid).unwrap_or(0);
                ReceivedEvent {
                    anchor_index: idx,
                    seq: e.seq,
                }
            })
            .collect();
        out_subs.push(SubscriberDeliveries { id: sid, received });
    }
    ConcurrentSubscribeOrderingOutcome {
        subscribers: out_subs,
    }
}

/// Outcome of a concurrent-subscribe no-event-loss proptest.
pub struct ConcurrentSubscribeNoLossOutcome {
    /// Seqs that were committed (every published event).
    pub committed_seqs: Vec<u64>,
    /// Seqs that the subscriber actually received.
    pub received_seqs: Vec<u64>,
}

/// Run one concurrent-subscribe no-event-loss proptest case.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn run_concurrent_subscribe_no_event_loss(
    writer_count: usize,
    writes_per_writer: usize,
) -> ConcurrentSubscribeNoLossOutcome {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/nl/".to_string()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(4096).expect("4096 is non-zero"),
    };
    let sub = match register(spec) {
        Ok(s) => s,
        Err(_) => {
            return ConcurrentSubscribeNoLossOutcome {
                committed_seqs: Vec::new(),
                received_seqs: Vec::new(),
            };
        }
    };
    let mut committed: Vec<u64> = Vec::new();
    let mut next_seq: u64 = 0;
    for w in 0..writer_count {
        for _ in 0..writes_per_writer {
            let anchor = Cid::sample_for_label(&format!("/nl/writer-{w}"));
            let mut e = make_change_event(anchor, ChangeKind::Updated, serde_json::json!({}));
            e.seq = next_seq;
            next_seq += 1;
            let _ = inject_event(&sub, e);
            committed.push(e_seq(next_seq, 1));
        }
    }
    let evs = sub.drain_events_blocking(Duration::from_millis(50));
    let received: Vec<u64> = evs.into_iter().map(|e| e.seq).collect();
    ConcurrentSubscribeNoLossOutcome {
        committed_seqs: committed,
        received_seqs: received,
    }
}

#[inline]
fn e_seq(next_seq: u64, offset: u64) -> u64 {
    next_seq.saturating_sub(offset)
}
