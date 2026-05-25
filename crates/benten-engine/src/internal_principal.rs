//! Engine-internal principal CID for un-attributed reads at production
//! boundaries that have no caller-supplied principal.
//!
//! ## Why this exists (R6 R1 FP-A, Bundle F2)
//!
//! Bundle F2 of the §8-A visibility tighten (V1-FROZEN-INTERFACE.md item 1;
//! V1-FROZEN-INTERFACE-DEFERRED.md Row D-7 closure) renames + tightens
//! `Engine::get_node` → `pub(crate) fn read_node`. The napi binding's
//! `Engine::get_node` previously called `self.inner.get_node(&parsed)` —
//! an un-attributed read with no caller principal — and must now migrate
//! to the principal-bearing `read_node_as` per CLAUDE.md baked-in #18.
//!
//! Production napi callers do not carry a caller principal at that
//! boundary (a thin-client / browser napi consumer presents a CID with
//! no UCAN-bearing actor context). Per V1-FROZEN-INTERFACE.md §1 ("the
//! `read_node_as(principal=ENGINE_INTERNAL_PRINCIPAL_CID, ...)` pattern
//! covers it without re-opening the freeze"), this module mints the
//! sentinel principal CID used in those engine-internal read sites.
//!
//! ## Construction
//!
//! `ENGINE_INTERNAL_PRINCIPAL_CID` = `Cid::from_blake3_digest(blake3(b"engine-internal-principal"))`.
//! Stable across the workspace: any process computes the same bytes from
//! the same string. The principal does NOT correspond to a real DID; it
//! is the engine-owned sentinel for un-attributed reads at the napi
//! boundary.
//!
//! ## Why NOT inside `pub mod testing`
//!
//! `pub mod testing` is `#[cfg(any(test, feature = "test-helpers"))]`-gated
//! at `crates/benten-engine/src/lib.rs`. The napi cdylib production build
//! does NOT enable `test-helpers`, so a testing-module placement would
//! make the constant unreachable from the napi binding. This module is
//! always-on.

use std::sync::LazyLock;

use benten_core::Cid;

/// Sentinel principal CID for engine-internal un-attributed reads at
/// public boundaries (napi `Engine::get_node` migrated to
/// `read_node_as(&ENGINE_INTERNAL_PRINCIPAL_CID, ...)`).
///
/// **Visibility:** `pub` so the napi binding crate (`bindings/napi`) and
/// other Benten-owned boundary crates can pass it as the principal
/// argument to `Engine::read_node_as` at production sites that lack a
/// caller-supplied principal. Surfaces in the `cargo-public-api`
/// baseline so the sentinel is locked alongside the §8-A method
/// signatures.
///
/// See module docs for rationale.
pub static ENGINE_INTERNAL_PRINCIPAL_CID: LazyLock<Cid> = LazyLock::new(|| {
    let digest = blake3::hash(b"engine-internal-principal");
    Cid::from_blake3_digest(*digest.as_bytes())
});
