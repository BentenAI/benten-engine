//! Five hand-written Phase 1 IVM views. Each lives in its own submodule
//! under `src/views/` and is re-exported from `views::*` so tests can reach
//! it either way:
//!
//! ```text
//! use benten_ivm::views::CapabilityGrantsView;
//! use benten_ivm::views::capability_grants::CapabilityGrantsView;
//! ```
//!
//! Normalized from the earlier `#[path]`-based composition to a proper
//! `src/views/mod.rs` after all five view files landed (r6-min R-minor-12 /
//! g5-cr-1).

pub mod capability_grants;
pub mod content_listing;
pub mod event_handler_dispatch;
pub mod governance_inheritance;
pub mod version_current;

pub use capability_grants::CapabilityGrantsView;
pub use content_listing::ContentListingView;
pub use event_handler_dispatch::{EventDispatchView, EventHandlerDispatchView};
pub use governance_inheritance::GovernanceInheritanceView;
pub use version_current::VersionCurrentView;
