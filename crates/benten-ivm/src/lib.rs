//! # benten-ivm — Incremental View Maintenance
//!
//! Phase 1 ships five hand-written IVM views:
//!
//! 1. Capability grants per entity
//! 2. Event handler dispatch
//! 3. Content listing (paginated, sorted by `createdAt`) — load-bearing for
//!    the `crud('post')` exit criterion
//! 4. Governance inheritance
//! 5. Version-chain CURRENT pointer resolution
//!
//! All views subscribe to the graph change stream from `benten-graph` and
//! maintain their state incrementally. The evaluator is deliberately ignorant
//! of IVM; IVM is a subscriber, not an engine-internal feature.
//!
//! ## Module layout
//!
//! - [`view`] — [`View`] trait, [`ViewError`], [`ViewBudget`],
//!   [`ViewDefinition`], and the shared query/result shapes.
//! - [`subscriber`] — [`Subscriber`] / [`ChangeStreamSubscriber`], the
//!   fan-out that dispatches change events to every registered view.
//! - [`views`] — the five Phase 1 hand-written view implementations.
//!
//! The Phase-2 generalized Algorithm B (per-view strategy selection, Z-set
//! cancellation, user-registered views) is deliberately out of scope —
//! Phase 1 is hand-written maintainers for five concrete views.

#![forbid(unsafe_code)]
#![allow(
    clippy::todo,
    reason = "Phase 1 leaves some per-view methods as todo!() for later groups"
)]

extern crate alloc;

pub mod subscriber;
pub mod view;

pub use subscriber::{ChangeStreamSubscriber, Subscriber};
pub use view::{
    IvmError, View, ViewBudget, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState,
};

/// Marker for the current implementation phase. Removed when the generalized
/// Algorithm B lands in Phase 2.
pub const STUB_MARKER: &str = "benten-ivm::phase-1";

// ---------------------------------------------------------------------------
// The five Phase 1 views — each in its own submodule under `src/views/`.
// The `#[path]` pattern keeps the existing `pub mod views { ... }` shape
// (inline parent) while letting each view live in its own file. A future
// mini-review can normalize to a proper `src/views/mod.rs` once all three
// G5 agents have landed; for now the `#[path]` pattern composes cleanly
// across G5-A / G5-B / G5-C without any of them touching each other's
// files.
// ---------------------------------------------------------------------------

pub mod views {
    //! Five hand-written Phase 1 IVM views. Each lives in its own submodule
    //! under `src/views/` and is re-exported from `views::*` so tests can
    //! reach it either way:
    //!
    //! ```text
    //! use benten_ivm::views::CapabilityGrantsView;
    //! use benten_ivm::views::capability_grants::CapabilityGrantsView;
    //! ```

    // G5-B: views 1, 2, 4
    #[path = "capability_grants.rs"]
    pub mod capability_grants;
    #[path = "event_handler_dispatch.rs"]
    pub mod event_handler_dispatch;
    #[path = "governance_inheritance.rs"]
    pub mod governance_inheritance;

    // G5-C: views 3, 5
    #[path = "content_listing.rs"]
    pub mod content_listing;
    #[path = "version_current.rs"]
    pub mod version_current;

    pub use capability_grants::CapabilityGrantsView;
    pub use content_listing::ContentListingView;
    pub use event_handler_dispatch::{EventDispatchView, EventHandlerDispatchView};
    pub use governance_inheritance::GovernanceInheritanceView;
    pub use version_current::VersionCurrentView;
}
