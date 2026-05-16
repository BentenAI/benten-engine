//! `ChangeStream` port — the change-event observation surface that
//! `benten-eval`'s SUBSCRIBE primitive consumes via dependency injection.
//!
//! Phase-2b G6-A landed this trait per D23-RESOLVED. Two alternatives were
//! considered and explicitly ruled out:
//!
//! 1. **Extend `PrimitiveHost`** with change-stream observation methods —
//!    ruled out because the port is single-purpose (change-event observation
//!    only) and `PrimitiveHost` is already overloaded with READ / WRITE /
//!    CALL / EMIT / IVM-view / capability-recheck surfaces. Adding observer
//!    methods would entangle SUBSCRIBE evolution with every backend
//!    implementor.
//! 2. **Define inside `benten-eval` directly** — ruled out for three reasons:
//!    (a) Arch-1 dep-break discipline keeps backend-storage types out of
//!    `benten-eval`; the change-event source is a backend concern; (b) the
//!    port lives at a stable seam, so testability benefits from the same
//!    no_std-compatible home as `Cid` + `Value`; (c) Phase-3 P2P sync needs
//!    a multi-source change-event merger (local + remote peers), and a
//!    dedicated trait at the `benten-core` seam keeps the merger pluggable
//!    without churning `PrimitiveHost`.
//!
//! The trait shape is intentionally minimal: callers `subscribe` with a
//! pattern, `next_event` polls the merged stream, and `unsubscribe` releases
//! resources. Pattern parsing + cap-checking + delivery-time enforcement
//! live in `benten-eval::primitives::subscribe`; this trait is purely the
//! boundary between "change events arrive from somewhere" and "SUBSCRIBE
//! routes them at the handler API."
//!
//! `SubscriberId` is content-addressed (CID over a canonical encoding of
//! the subscription spec) per D5 strengthening item 1, so Phase-3 peers can
//! re-establish the same subscription deterministically.

use alloc::string::String;
use alloc::vec::Vec;

use crate::Cid;

/// Opaque subscriber identity.
///
/// Phase-2b G6-A — content-addressed: derived from BLAKE3 over the
/// canonical-encoded subscription spec (pattern + cursor + buffer-size).
/// Mirrors how handlers are identified and is forward-compatible with
/// Phase-3 P2P sync where peers need to recognize "the same subscription"
/// across re-registration.
///
/// `Copy` is intentional: `SubscriberId` wraps a fixed-size `Cid`, so
/// passing it around handler boundaries does not allocate. The opaque
/// newtype prevents callers from forging a synthetic id or mistaking it
/// for a generic CID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriberId(Cid);

impl SubscriberId {
    /// Construct a new subscriber id from a derived CID. Callers should
    /// produce `cid` by content-hashing the subscription spec (pattern,
    /// cursor, buffer-size) so two callers issuing the same subscription
    /// arrive at the same id.
    #[must_use]
    pub fn from_cid(cid: Cid) -> Self {
        Self(cid)
    }

    /// Borrow the underlying CID. Useful for serialization and Phase-3 sync
    /// payloads.
    #[must_use]
    pub fn as_cid(&self) -> &Cid {
        &self.0
    }
}

impl core::fmt::Display for SubscriberId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Render as the underlying CID's base32 multibase form so logs +
        // diagnostics show a single canonical spelling.
        core::fmt::Display::fmt(&self.0, f)
    }
}

/// Kind of change observed on a graph anchor.
///
/// Mirrors the three IVM mutation classes the engine emits today; Phase-3
/// sync may add `Replicated` or `Conflict` arms — `#[non_exhaustive]`
/// reserves space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ChangeKind {
    /// A new Node materialized at this anchor.
    Created,
    /// An existing Node's content changed.
    Updated,
    /// The Node was removed.
    Deleted,
}

/// A change event observed by the `ChangeStream` port.
///
/// Carries the engine-assigned monotonic `seq` (load-bearing for
/// exactly-once-at-handler dedup) plus an opaque `payload_bytes` blob the
/// SUBSCRIBE delivery layer decodes per the registered handler's signal-
/// shape. The payload stays as raw bytes here so `benten-core` does not
/// need to model `Value` round-tripping for the change-stream wire path.
///
/// # Multi-label matching contract
///
/// `labels` carries the **full** label set of the affected Node at emit
/// time, not a single primary label. The SUBSCRIBE matcher walks every
/// entry and fires when ANY label matches the subscription pattern, so a
/// multi-labeled Node `["User","Admin"]` is delivered to consumers
/// matching either `User:*` or `Admin:*`. Producers MUST populate the
/// complete label set; collapsing it to one label silently drops
/// deliveries for the other labels.
///
/// # SemVer
///
/// `#[non_exhaustive]`: this struct has a documented widening trajectory
/// (the R6FP-G1 expansion 4→9 fields already happened; Phase-4-Meta
/// multi-device / Kith attribution will widen it again). External crates
/// must construct via [`ChangeEvent::legacy_minimal`] (the production
/// graph→eval bridge populates every field directly inside this
/// workspace), so a future field add is not a major-version break.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ChangeEvent {
    /// Anchor-level identity the change applies to.
    pub anchor_cid: Cid,
    /// Mutation class.
    pub kind: ChangeKind,
    /// Engine-assigned monotonic sequence. Strictly increasing per
    /// `(subscription, anchor_cid)` key per D5 within-key strict ordering.
    pub seq: u64,
    /// Opaque event payload — DAG-CBOR or JSON bytes; the SUBSCRIBE
    /// delivery layer decodes it.
    pub payload_bytes: Vec<u8>,
    /// Full label set of the affected Node at the moment the event was
    /// emitted. Empty for a delete that targeted an already-absent CID
    /// (idempotent-delete miss). Walked by the SUBSCRIBE matcher so a
    /// multi-labeled Node `["User","Admin"]` matches BOTH `User:*` and
    /// `Admin:*` glob patterns.
    pub labels: Vec<String>,
    /// Monotonically increasing transaction id assigned by the engine at
    /// commit time. Forwarded from the graph-level ChangeEvent; SUBSCRIBE
    /// consumers use it to reason about before/after ordering without
    /// wall-clock timestamps.
    pub tx_id: u64,
    /// Actor attribution. The Node CID of the actor who performed the
    /// write, when the write came through an attributed engine path;
    /// `None` for system / privileged writes.
    pub actor_cid: Option<Cid>,
    /// Handler attribution. The handler subgraph CID that issued the
    /// write, when known.
    pub handler_cid: Option<Cid>,
    /// Capability-grant attribution. The grant CID authorizing the
    /// write, when known.
    pub capability_grant_cid: Option<Cid>,
}

impl ChangeEvent {
    /// Minimal constructor: sets `labels = vec![]`, `tx_id = 0`, and the
    /// three attribution CIDs to `None`. Suitable for test-grade event
    /// fabrication; production callers (the engine's graph→eval bridge)
    /// must populate every field directly so multi-label matching and
    /// attribution are not silently lost.
    #[must_use]
    pub fn legacy_minimal(
        anchor_cid: Cid,
        kind: ChangeKind,
        seq: u64,
        payload_bytes: Vec<u8>,
    ) -> Self {
        Self {
            anchor_cid,
            kind,
            seq,
            payload_bytes,
            labels: Vec::new(),
            tx_id: 0,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
        }
    }
}

/// The change-stream port. SUBSCRIBE consumes this via DI; backends
/// implement it (the IVM subscriber, a Phase-3 peer-merger, an in-memory
/// test fixture, etc.).
///
/// Object-safe: every method takes `&mut self` or `&self` and uses
/// concrete parameter types.
pub trait ChangeStream: Send + Sync {
    /// Register a subscription. Returns the assigned subscriber id (which
    /// the caller should derive deterministically from the spec per the
    /// content-addressed identity contract).
    ///
    /// # Errors
    ///
    /// Returns the catalog code as a `String` so this trait stays free of
    /// any error-type dependency. SUBSCRIBE wraps the failure in its
    /// typed error envelope at the call site. The codes a conforming
    /// backend may return here:
    ///
    /// - `E_SUBSCRIBE_PATTERN_INVALID` — `pattern` is not a well-formed
    ///   `Label:*` glob.
    /// - `E_SUBSCRIBE_DELIVERY_FAILED` — the backend could not register
    ///   the subscription (transient channel / storage failure).
    fn subscribe(&mut self, pattern: &str, id: SubscriberId) -> Result<(), String>;

    /// Poll the next event for `id`. `Ok(None)` means "no event ready
    /// right now"; the caller decides whether to spin, block on a sibling
    /// signal, or yield. SUBSCRIBE wraps this in the cursor-aware
    /// delivery loop.
    ///
    /// # Errors
    ///
    /// Catalog code as `String` per the same rationale as `subscribe`.
    /// The codes a conforming backend may return here:
    ///
    /// - `E_SUBSCRIBE_CURSOR_LOST` — the requested cursor falls before
    ///   the retained event window.
    /// - `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED` — equivalent surface raised
    ///   at re-registration time rather than mid-stream.
    /// - `E_SUBSCRIBE_REVOKED_MID_STREAM` — the subscriber's read-coverage
    ///   capability no longer holds (per-event delivery-time cap-recheck).
    /// - `E_SUBSCRIBE_DELIVERY_FAILED` — transient delivery-channel
    ///   failure.
    fn next_event(&mut self, id: &SubscriberId) -> Result<Option<ChangeEvent>, String>;

    /// Release the subscription. Idempotent: a second unsubscribe of an
    /// already-released id returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// Catalog code as `String`. Idempotent release does not error; the
    /// only failure a conforming backend may surface is
    /// `E_SUBSCRIBE_DELIVERY_FAILED` (transient channel / storage failure
    /// while tearing down subscription resources).
    fn unsubscribe(&mut self, id: &SubscriberId) -> Result<(), String>;
}
