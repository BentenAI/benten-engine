//! Typed errors for `SubgraphSpec` construction + walking.
//!
//! Per the never-silent-fallback discipline (P2P-mainstream choice;
//! Veilid/MLS/Nostr-NIP-44; age silent-ignore deliberately rejected).
//! Each failure surfaces as a distinct, matchable variant; no silent
//! truncation, no silent skip.

use crate::Cid;

extern crate alloc;
use alloc::string::String;

/// Errors produced by `benten_core::subgraph_spec` construction and walking.
///
/// `#[non_exhaustive]` so future variants (e.g. additional restricted-spec
/// predicate violations) are a minor-version-bump compatible addition.
/// Downstream matchers must carry a `_ =>` fallback arm.
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum SubgraphSpecError {
    /// Walker invoked against a Spec whose root CID has no resolvable
    /// structural definition (no outgoing edges declared in the Spec, not
    /// referenced as an edge target, not explicitly registered). Fires
    /// per Spike F's never-silent-fallback discipline.
    #[error("subgraph-spec walker: CID is missing / unresolvable in spec: {0}")]
    CidMissing(Cid),

    /// Walker invoked against a Spec whose roots include the Spec's own
    /// content CID (would unbounded-recurse if walked naively). Detected
    /// at walk-time after `Spec::cid()` is computed.
    #[error("subgraph-spec walker: spec is self-referential (root includes spec's own CID)")]
    SelfReferentialCycle,

    /// Walker hit the configured `max_depth` boundary. Production-arm
    /// signal: the walker returns its enumeration truncated at the bound
    /// (no Nodes beyond `max_depth` are emitted). This variant exists for
    /// callers that elect to treat the bound as an error rather than a
    /// graceful truncation.
    #[error("subgraph-spec walker: max_depth ({max_depth}) exceeded at CID {cid}")]
    MaxDepthExceeded {
        /// The configured `max_depth` bound.
        max_depth: u32,
        /// The CID at which the bound was hit.
        cid: Cid,
    },

    /// Construction-time: a `RestrictedSpec` declares both
    /// allowlist and denylist over the SAME label, making the predicate
    /// structurally unsatisfiable (the decidable-non-emptiness contract:
    /// a Spec that admits no Nodes is a silently-no-op share-grant and
    /// must surface at construct-time, not walk-time).
    #[error(
        "subgraph-spec: conflicting label predicates (label {label:?} is in both allow + deny)"
    )]
    ConflictingLabelPredicates {
        /// The conflicting label.
        label: String,
    },

    /// Combinator: `intersect(a, b)` produced a structurally empty Spec
    /// (e.g. disjoint roots + disjoint label predicates). The combinator
    /// surfaces this as a typed variant so proptest harnesses + DSL
    /// callers can distinguish "degenerate" from "bug".
    #[error("subgraph-spec combinators: intersect of disjoint specs is empty")]
    EmptyIntersection,

    /// Internal canonical-bytes encoding error. Carries a human-readable
    /// message from `serde_ipld_dagcbor`.
    #[error("subgraph-spec canonical-bytes encode failed: {0}")]
    Encode(String),
}
