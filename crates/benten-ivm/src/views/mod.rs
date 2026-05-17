//! Five hand-written Phase-1 IVM view inner-kernels. Each lives in its own
//! submodule under `src/views/` and is re-exported from `views::*` so tests
//! can reach it either way:
//!
//! ```text
//! use benten_ivm::views::CapabilityGrantsView;
//! use benten_ivm::views::capability_grants::CapabilityGrantsView;
//! ```
//!
//! Normalized from the earlier `#[path]`-based composition to a proper
//! `src/views/mod.rs` after all five view files landed (r6-min R-minor-12 /
//! g5-cr-1).
//!
//! ## Phase-3 G15-A re-categorisation (per `ivm-disagree-1`)
//!
//! The 5 hand-written views below are **inner kernels of [`crate::Strategy::B`]**
//! — NOT [`crate::Strategy::A`] baselines. Phase-2b shipped them with
//! `View::strategy()` returning `Strategy::A` because the trait default was
//! "hand-written incremental maintenance"; G15-A keeps that per-view
//! reporting (the views are still hand-written incremental maintainers
//! internally) but re-categorises them at the kernel-dispatch level: they
//! are the **canonical fast-path inner kernels** [`crate::AlgorithmBView`]
//! invokes when the dispatch router classifies a view-id as canonical via
//! [`crate::dispatch_for`] (which returns `Strategy::A` as the canonical
//! fast-path *classification*, not the engine-boundary strategy of the
//! resulting view).
//!
//! At the engine boundary [`crate::AlgorithmBView`]'s `strategy()` impl
//! always returns `Strategy::B` — a registered view is "the Algorithm B
//! wrapper", whether its inner kernel is one of the canonical 5 below or
//! [`crate::algorithm_b::Algorithm`]'s generic kernel. The G15-A canonical
//! fast-path-preservation gate (`tests/algorithm_b_general.rs::
//! algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline`)
//! measures wallclock against a **Strategy::B baseline** per
//! `ivm-disagree-1`, NOT against a separate Strategy::A handwritten
//! baseline.

pub mod capability_grants;
pub mod content_listing;
pub mod event_handler_dispatch;
pub mod governance_inheritance;
pub mod version_current;

pub use capability_grants::CapabilityGrantsView;
pub use content_listing::ContentListingView;
pub use event_handler_dispatch::EventDispatchView;
pub use governance_inheritance::GovernanceInheritanceView;
pub use version_current::VersionCurrentView;
