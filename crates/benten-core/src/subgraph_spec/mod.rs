//! `SubgraphSpec` — the Sharing & Confidentiality (S&C) 4-thing thin core
//! primitive and its associated walker, combinators, and authoring macros.
//!
//! # Phase-4-Meta-Core G-CORE-3w
//!
//! This module is the **walker substrate** for the post-Phase-3 S&C
//! architecture ratified at
//! `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`.
//! The SubgraphSpec primitive is itself a [`crate::Subgraph`] composed of the
//! existing 12 operation primitives — CLAUDE.md baked-in #1
//! 12-primitive-irreducibility is preserved. **No new `PrimitiveKind`
//! variant is minted.** The fractal property (Spike F finding): a
//! [`Spec`] is described by Roots + Expansion + Inclusion + Termination,
//! the walker that walks it is itself a [`crate::Subgraph`] composed of
//! `READ + TRANSFORM + BRANCH + ITERATE` primitives, and the walker is
//! shipped **once** (not re-implemented per Spec).
//!
//! # Disambiguation
//!
//! This is **not** the DSL-friendly builder in `benten-engine::subgraph_spec`
//! (which sugars `Engine::register_subgraph` / `Engine::call` inputs and
//! has been on `main` since Phase-2b R6 Wave 2). The two share a name by
//! coincidence at the leaf module level; they live in different crates,
//! solve different problems, and have no shared types. The post-G-CORE-3
//! ratification is that **this** `benten_core::subgraph_spec::Spec` is the
//! Roots/Expansion/Inclusion/Termination thin-core primitive that
//! `AuthorizationGrant.selector` carries (D-4M-R1, R4) and that drives
//! per-`StructuralPath` key derivation (D-4M-R5; Spike E
//! Interpretation B).
//!
//! # The 4 thin-core dimensions
//!
//! Per R0.7 plan-doc §3 G-CORE-3 def + RATIFIED §R1 + 8 spike-derived
//! refinements:
//!
//! 1. **Roots** — the entry CIDs the walker starts BFS from.
//! 2. **Expansion** — which outgoing edges are followed at each step.
//!    Today: the Spec itself declares its edges (Spike F: data-not-contract).
//! 3. **Inclusion** — a `SubgraphSpecRestriction` predicate that narrows / filters
//!    which Nodes are emitted (Path (a) per R1 ratification: restricted-
//!    spec language only; Path (b) structurally unsound per Spike H+1.1).
//! 4. **Termination** — `max_depth` + structural cycle detection.
//!
//! # API surface
//!
//! - [`Spec`] — the 4-thing thin core; constructed via [`Spec::builder`].
//! - [`Spec::cid`] — content-addressed Spec invariant (Spike F).
//! - [`Spec::contains_spec`] — Spec-vs-Spec containment for combinators.
//! - [`Spec::to_canonical_bytes`] — canonical DAG-CBOR encoding.
//! - [`StructuralPath`] — ordered edge-label sequence; public reflection
//!   per D-4M-R4 (BFS-order, path-as-data).
//! - [`StructuralPath::edge_labels`] — the load-bearing reflection
//!   accessor that recipients consume to verify path encoding.
//! - [`SubgraphSpecError`] — typed errors (`CidMissing` /
//!   `SelfReferentialCycle` / `MaxDepthExceeded` /
//!   `ConflictingLabelPredicates` / `EmptyIntersection`).
//! - [`walker`] sub-module — `walk()` + `walker_as_subgraph()` +
//!   [`walker::WalkResult`].
//! - [`combinators`] sub-module — `intersect`, `union`, `filter`.
//! - `query!` macro — sugar that produces the same Spec as the raw
//!   builder (byte-equal canonical form per Spike F painful-raw-literal
//!   finding).
//!
//! # Restricted-spec extension slots
//!
//! [`SubgraphSpecRestriction`] uses **named-arm carriers**, not opaque payloads
//! (refinement #5). Future extensions are added as additional named
//! variants on the `non_exhaustive` enum; downstream matchers carry a
//! `_ =>` arm. **Opaque specs must materialize before sharing**
//! (refinement #4): sharing a Spec requires that all its predicates be
//! materializable so the recipient can independently re-evaluate.

extern crate alloc;

pub mod combinators;
pub mod errors;
pub mod spec;
pub mod walker;

#[doc(hidden)]
pub mod macros;

pub use combinators::{filter, intersect, union};
pub use errors::SubgraphSpecError;
pub use spec::{Spec, SpecBuilder, StructuralPath, SubgraphSpecRestriction};
pub use walker::{WalkResult, walk, walker_as_subgraph};

// Re-export the `query!` macro under this module path so call sites can
// `use benten_core::subgraph_spec::query;` (matching the other types in
// this module). `#[macro_export]` itself only places the macro at crate
// root; the alias here is the load-bearing import surface.
pub use crate::query;
