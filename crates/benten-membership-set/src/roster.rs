//! EP-1 three-tier extensibility roster (F-CRATE-1) — the architecture-shape
//! pins the 15th-crate boundary observes.
//!
//! - **Tier-1 OPEN** backend seams (`{KVBackend, BlobBackend, GraphBackend,
//!   Renderer, Transport, Materializer}` — EXACTLY SIX; object-safe +
//!   conformance-tested). `DeviceAuthBackend` is NOT here.
//! - **Tier-2 SEALED** policy seams (`{CapabilityPolicy, GrantReader,
//!   DeviceAuthBackend}` — Benten-internal, no external impl).
//! - **Tier-3 ENUM-dispatch** (`benten_ivm::Strategy` is an ENUM, NOT a trait).
//! - The OPEN and SEALED tiers are **DISJOINT** (the F4-008 boundary).
//! - NO generic extension-trait + NO extension-registry (everything is a
//!   plugin; trust = compiled-in / user-root). The grep-defense pin asserts
//!   neither of those two rejected symbol shapes appears in this crate.
//!
//! `Scope` is EXACTLY-2-arm — the membership crate references the real
//! [`benten_caps::Scope`] (a 3rd arm is a HALT-AND-SURFACE).

/// The EP-1 Tier-1 OPEN backend seams (object-safe). **EXACTLY SIX** per R0.5
/// §2.7 line 264 (the m-15 GNC-4 precision edit). `DeviceAuthBackend` is
/// deliberately NOT in this roster — it is a SEALED policy seam (a sealed trait
/// cannot be an open object-safe extension point; the F4-008 boundary).
pub const TIER1_OPEN_SEAMS: [&str; 6] = [
    "KVBackend",
    "BlobBackend",
    "GraphBackend",
    "Renderer",
    "Transport",
    "Materializer",
];

/// The EP-1 Tier-2 SEALED policy seams. Benten-internal; no external crate can
/// impl these. `DeviceAuthBackend` is the #7 sealed seam.
pub const TIER2_SEALED_SEAMS: [&str; 3] = ["CapabilityPolicy", "GrantReader", "DeviceAuthBackend"];

/// `benten_ivm::Strategy` is a Tier-3 ENUM (NOT a trait seam; m-15 GNC-4 /
/// baked-in #2). The membership crate does not depend on `benten-ivm` (out of
/// its B-1 set), so this models the enum-dispatch shape locally — a Copy value
/// type dispatched by `match`, never a `dyn Strategy` trait object.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrategyShape {
    /// Algorithm-B dependency-tracked IVM.
    DependencyTracked,
    /// The reserved future-additive arm.
    Reserved,
}
