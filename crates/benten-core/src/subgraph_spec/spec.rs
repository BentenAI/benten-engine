//! `Spec`, `SpecBuilder`, `SubgraphSpecRestriction`, `StructuralPath`.
//!
//! The 4-thing thin-core SubgraphSpec primitive + its builder, restricted-
//! spec predicates, and the path-as-data `StructuralPath` carrier.

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::errors::SubgraphSpecError;
use crate::{Cid, format_err};

// ---------------------------------------------------------------------------
// StructuralPath
// ---------------------------------------------------------------------------

/// The canonical BFS-arrival path for a Node within a [`Spec`] walk.
///
/// Per RATIFIED §R4 + Spike H+1.1: "BFS-order, carry canonical path in
/// AuthorizationGrant" — the path is **data**, not a contract the recipient
/// re-derives. Each `StructuralPath` is an ordered sequence of
/// edge-labels describing how BFS reached this Node from a root.
///
/// Per RATIFIED §R5 + Spike E Interpretation B: a multi-path-reachable
/// Node gets **distinct `StructuralPath`s per arrival**; the walker emits
/// the Node once per path (path-tagged-keys feature for selective-share).
///
/// Equality is the load-bearing structural property — two paths with the
/// same edge-label sequence compare equal regardless of construction
/// order; two paths with different edge-label sequences compare unequal.
/// The `Ord` implementation is byte-lex over the edge-label sequence for
/// stable deterministic BTreeSet ordering; carries no semantic meaning.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StructuralPath {
    /// Ordered edge-label sequence (root → ... → this Node).
    edge_labels: Vec<String>,
}

impl StructuralPath {
    /// Construct an empty path (the root itself; depth 0).
    #[must_use]
    pub fn root() -> Self {
        Self {
            edge_labels: Vec::new(),
        }
    }

    /// Construct a path from an ordered slice of edge labels.
    pub fn from_labels<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            edge_labels: labels.into_iter().map(Into::into).collect(),
        }
    }

    /// Public reflection on the path-as-data principle (D-4M-R4): the
    /// recipient consumes the ordered edge-label sequence directly to
    /// re-derive `K(N)` per §1.A.FROZEN item 15(f) — the path encoding is
    /// **verifiable**, not opaque.
    #[must_use]
    pub fn edge_labels(&self) -> Vec<&str> {
        self.edge_labels.iter().map(String::as_str).collect()
    }

    /// Depth of this path = number of edge-labels traversed.
    #[must_use]
    pub fn depth(&self) -> u32 {
        u32::try_from(self.edge_labels.len()).unwrap_or(u32::MAX)
    }

    /// Append an edge-label, producing a new (longer) path. (Pure;
    /// doesn't mutate `self`.)
    #[must_use]
    pub fn extended(&self, edge_label: impl Into<String>) -> Self {
        let mut labels = self.edge_labels.clone();
        labels.push(edge_label.into());
        Self {
            edge_labels: labels,
        }
    }

    /// Borrow the ordered edge-label sequence as owned `String` slice
    /// (useful for serde paths that want owned data).
    #[must_use]
    pub fn as_strings(&self) -> &[String] {
        &self.edge_labels
    }
}

// ---------------------------------------------------------------------------
// SubgraphSpecRestriction — Inclusion predicate (Path (a) per R1)
// ---------------------------------------------------------------------------

/// The `Inclusion` half of the 4-thing thin core: a restricted-spec
/// language that narrows which Nodes are emitted.
///
/// Per R1 ratification (Path (a)): the restricted-spec language is a
/// **NAMED** enum (not opaque), so the recipient can independently
/// re-evaluate the predicate. Path (b) (opaque arbitrary closure) was
/// structurally unsound per Spike H+1.1: opaque predicates can't be
/// non-emptiness-decided + can't be safely shared across trust boundaries.
///
/// **Extension slots are NAMED** (refinement #5): future variants are
/// added as additional named arms on this `non_exhaustive` enum, not as
/// an `Opaque(Box<dyn Predicate>)` escape hatch. Adding a new arm is a
/// minor version bump; downstream matchers carry `_ =>` fallback.
///
/// The current v1 surface covers what the post-RATIFIED design needs to
/// express AuthorizationGrant scopes:
///
/// - [`SubgraphSpecRestriction::Unrestricted`] — predicate matches every Node
///   reached by walking; the structural shape (Roots/Expansion/
///   Termination) alone determines membership.
/// - [`SubgraphSpecRestriction::ByLabel`] — allowlist / denylist over Node labels.
///   Constructing a Spec with overlapping allow + deny on the same label
///   surfaces typed `ConflictingLabelPredicates` at build time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SubgraphSpecRestriction {
    /// No additional inclusion filter; every walker-reached Node passes.
    Unrestricted,
    /// Label-based allowlist + denylist (overlap rejected at build time).
    ByLabel {
        /// Labels that, when present on a Node, qualify it for inclusion.
        /// Empty = no allowlist (all labels qualify).
        allow: BTreeSet<String>,
        /// Labels that, when present on a Node, exclude it from inclusion.
        deny: BTreeSet<String>,
    },
}

impl SubgraphSpecRestriction {
    /// Spec-vs-Spec containment over the Inclusion dimension: this
    /// restricted-spec admits a `superset` of what `other` admits.
    ///
    /// `Unrestricted.contains(any)` is always `true`; this models "no
    /// inclusion filter = matches every Node".
    #[must_use]
    pub fn contains(&self, other: &SubgraphSpecRestriction) -> bool {
        match (self, other) {
            (SubgraphSpecRestriction::Unrestricted, _) => true,
            (SubgraphSpecRestriction::ByLabel { .. }, SubgraphSpecRestriction::Unrestricted) => false,
            (
                SubgraphSpecRestriction::ByLabel {
                    allow: self_allow,
                    deny: self_deny,
                },
                SubgraphSpecRestriction::ByLabel {
                    allow: other_allow,
                    deny: other_deny,
                },
            ) => {
                // self contains other if:
                //   * self's allowlist is empty OR a superset of other's allowlist
                //   * self's denylist is empty OR a subset of other's denylist
                let allow_ok = self_allow.is_empty() || self_allow.is_superset(other_allow);
                let deny_ok = self_deny.is_subset(other_deny);
                allow_ok && deny_ok
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Spec — the 4-thing thin core
// ---------------------------------------------------------------------------

/// The Roots / Expansion / Inclusion / Termination thin-core SubgraphSpec
/// primitive. Constructed via [`Spec::builder`].
///
/// The `Spec` is **content-addressed** ([`Spec::cid`]) so that
/// `AuthorizationGrant.selector = Spec.cid` is a stable invariant per
/// Spike F; and **canonical-bytes-encoded** ([`Spec::to_canonical_bytes`])
/// so two builders that constructed the same logical Spec produce
/// byte-identical encodings.
///
/// **Walker discipline:** the post-G-CORE-3 contract is that the walker is
/// itself a [`crate::Subgraph`] shipped once in `benten-core` (see
/// [`super::walker::walker_as_subgraph`]); callers never re-implement BFS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    /// **Roots** — the entry CIDs the walker starts BFS from. Stored
    /// sorted by lex byte order for deterministic canonical-bytes.
    roots: Vec<Cid>,

    /// **Expansion** — declared edges of the Spec itself.
    /// `BTreeMap<source_cid, BTreeMap<edge_label, target_cid>>` —
    /// per-source-Cid the edge labels are unique + sorted (the lex order
    /// is the BFS tie-breaker per R4 ratification).
    edges: BTreeMap<Cid, BTreeMap<String, Cid>>,

    /// **Inclusion** — restricted-spec predicate per Path (a).
    inclusion: SubgraphSpecRestriction,

    /// **Termination** — `max_depth` bound + cycle detection. Required;
    /// builder defaults to a sensible cap if unset.
    max_depth: u32,
}

impl Spec {
    /// Default `max_depth` used by [`SpecBuilder::new`] when the caller
    /// doesn't specify one. Bounded to prevent unbounded walk.
    pub const DEFAULT_MAX_DEPTH: u32 = 64;

    /// Construct an empty [`SpecBuilder`].
    #[must_use]
    pub fn builder() -> SpecBuilder {
        SpecBuilder::new()
    }

    /// The Spec's roots (sorted, deduped, lex-byte-order).
    #[must_use]
    pub fn roots(&self) -> &[Cid] {
        &self.roots
    }

    /// The Spec's declared edges (per-source map of label → target).
    #[must_use]
    pub fn edges(&self) -> &BTreeMap<Cid, BTreeMap<String, Cid>> {
        &self.edges
    }

    /// The Spec's Inclusion predicate.
    #[must_use]
    pub fn inclusion(&self) -> &SubgraphSpecRestriction {
        &self.inclusion
    }

    /// The Spec's configured `max_depth` bound.
    #[must_use]
    pub fn max_depth(&self) -> u32 {
        self.max_depth
    }

    /// Canonical DAG-CBOR encoding. Roots + edges are pre-sorted at
    /// construction so this is a single deterministic encode pass.
    ///
    /// # Errors
    /// Returns [`SubgraphSpecError::Encode`] on DAG-CBOR encoding failure
    /// (Phase-1-parity: the workspace's `Value` / `Node` encoders surface
    /// `Serialize` similarly).
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, SubgraphSpecError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| SubgraphSpecError::Encode(format_err(&e)))
    }

    /// Content-addressed CID for this Spec — the load-bearing invariant
    /// per Spike F (Spec CID = AuthorizationGrant selector).
    ///
    /// # Errors
    /// Returns [`SubgraphSpecError::Encode`] on encoding failure.
    pub fn cid(&self) -> Result<Cid, SubgraphSpecError> {
        let bytes = self.to_canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }

    /// Spec-vs-Spec containment: `self` admits a superset of what `other`
    /// admits across all four dimensions + root-set semantics.
    ///
    /// Containment is used by combinator proptests + downstream
    /// envelope-ceiling checks (`intersect ⊆ a + intersect ⊆ b`; `a ⊆
    /// union + b ⊆ union`).
    ///
    /// The shape:
    ///   * self.roots ⊇ other.roots (self admits at least as many entry CIDs)
    ///   * self.edges ⊇ other.edges (self declares at least as many expansion edges)
    ///   * self.max_depth ≥ other.max_depth (self walks at least as deep)
    ///   * self.inclusion.contains(other.inclusion) (self's filter is at-most-as-narrow)
    #[must_use]
    pub fn contains_spec(&self, other: &Spec) -> bool {
        let self_roots: BTreeSet<&Cid> = self.roots.iter().collect();
        let other_roots: BTreeSet<&Cid> = other.roots.iter().collect();
        if !self_roots.is_superset(&other_roots) {
            return false;
        }
        if self.max_depth < other.max_depth {
            return false;
        }
        // Every (source, label, target) edge in `other` must also be in `self`.
        for (src, other_edges) in &other.edges {
            let Some(self_edges) = self.edges.get(src) else {
                return false;
            };
            for (label, target) in other_edges {
                if self_edges.get(label) != Some(target) {
                    return false;
                }
            }
        }
        if !self.inclusion.contains(&other.inclusion) {
            return false;
        }
        true
    }
}

// ---------------------------------------------------------------------------
// SpecBuilder
// ---------------------------------------------------------------------------

/// Builder for [`Spec`]. Construct via [`Spec::builder`].
///
/// All builder methods take `self` by value (owned-builder pattern) so
/// chains read naturally:
///
/// ```ignore
/// let spec = Spec::builder()
///     .with_root(cid_root)
///     .with_edge(cid_root, "child", cid_a)
///     .with_max_depth(8)
///     .build()?;
/// ```
///
/// Construction-time validation (decidable-non-emptiness contract) fires
/// at [`SpecBuilder::build`].
#[derive(Debug, Clone, Default)]
pub struct SpecBuilder {
    pub(super) roots: BTreeSet<Cid>,
    pub(super) edges: BTreeMap<Cid, BTreeMap<String, Cid>>,
    pub(super) allow_labels: BTreeSet<String>,
    pub(super) deny_labels: BTreeSet<String>,
    pub(super) max_depth: Option<u32>,
}

impl SpecBuilder {
    /// Construct an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a root CID. Multiple calls accumulate (sorted/deduped at build).
    #[must_use]
    pub fn with_root(mut self, cid: Cid) -> Self {
        self.roots.insert(cid);
        self
    }

    /// Declare a structural edge `from --label--> to` in the Spec's
    /// Expansion dimension. Multiple calls accumulate; declaring the
    /// same `(from, label)` twice replaces the prior target.
    #[must_use]
    pub fn with_edge(mut self, from: Cid, label: String, to: Cid) -> Self {
        self.edges.entry(from).or_default().insert(label, to);
        self
    }

    /// Set the Inclusion-dim label allowlist. Multiple calls replace the
    /// prior allowlist (last-call-wins). Accepts any iterator of types
    /// that can convert into `String`.
    #[must_use]
    pub fn with_label_allowlist<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allow_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Set the Inclusion-dim label denylist. Multiple calls replace the
    /// prior denylist (last-call-wins).
    #[must_use]
    pub fn with_label_denylist<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.deny_labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Filter-helper: deny a single label (combinator-friendly affordance).
    #[must_use]
    pub fn deny_label(mut self, label: impl Into<String>) -> Self {
        self.deny_labels.insert(label.into());
        self
    }

    /// Set the Termination-dim `max_depth` bound.
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Finalize the Spec. Runs construction-time validation:
    ///
    /// - Overlapping allowlist + denylist on the same label →
    ///   [`SubgraphSpecError::ConflictingLabelPredicates`].
    ///
    /// # Errors
    /// See variant docs.
    pub fn build(self) -> Result<Spec, SubgraphSpecError> {
        // Decidable-non-emptiness validation: no label may be in both
        // allow + deny on the same Spec (silent-no-op share-grant defense).
        if let Some(label) = self.allow_labels.intersection(&self.deny_labels).next() {
            return Err(SubgraphSpecError::ConflictingLabelPredicates {
                label: label.clone(),
            });
        }

        let mut roots: Vec<Cid> = self.roots.into_iter().collect();
        // BTreeSet → Vec already sorted by Cid's byte-lex Ord, but be
        // explicit for the canonical-bytes invariant.
        roots.sort_by_key(|c| *c.as_bytes());

        let inclusion = if self.allow_labels.is_empty() && self.deny_labels.is_empty() {
            SubgraphSpecRestriction::Unrestricted
        } else {
            SubgraphSpecRestriction::ByLabel {
                allow: self.allow_labels,
                deny: self.deny_labels,
            }
        };

        Ok(Spec {
            roots,
            edges: self.edges,
            inclusion,
            max_depth: self.max_depth.unwrap_or(Spec::DEFAULT_MAX_DEPTH),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cid_for(label: &str) -> Cid {
        let digest = blake3::hash(label.as_bytes());
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    #[test]
    fn builder_default_max_depth_is_set() {
        let spec = Spec::builder()
            .with_root(cid_for("r"))
            .build()
            .expect("builds");
        assert_eq!(spec.max_depth(), Spec::DEFAULT_MAX_DEPTH);
    }

    #[test]
    fn conflicting_label_predicates_typed_error() {
        let err = Spec::builder()
            .with_root(cid_for("r"))
            .with_label_allowlist(["X"])
            .with_label_denylist(["X"])
            .build()
            .expect_err("conflict");
        assert!(matches!(
            err,
            SubgraphSpecError::ConflictingLabelPredicates { .. }
        ));
    }

    #[test]
    fn canonical_bytes_deterministic_across_insertion_order() {
        let a = cid_for("a");
        let b = cid_for("b");
        let s1 = Spec::builder()
            .with_root(a)
            .with_root(b)
            .build()
            .expect("s1");
        let s2 = Spec::builder()
            .with_root(b)
            .with_root(a)
            .build()
            .expect("s2");
        assert_eq!(
            s1.to_canonical_bytes().unwrap(),
            s2.to_canonical_bytes().unwrap()
        );
    }

    #[test]
    fn cid_is_stable_across_invocations() {
        let s = Spec::builder()
            .with_root(cid_for("r"))
            .with_edge(cid_for("r"), "child".to_string(), cid_for("a"))
            .build()
            .expect("spec");
        let c1 = s.cid().expect("cid 1");
        let c2 = s.cid().expect("cid 2");
        assert_eq!(c1, c2);
    }

    #[test]
    fn contains_spec_reflexive() {
        let s = Spec::builder()
            .with_root(cid_for("r"))
            .build()
            .expect("spec");
        assert!(s.contains_spec(&s));
    }

    #[test]
    fn contains_spec_root_superset() {
        let a = cid_for("a");
        let b = cid_for("b");
        let bigger = Spec::builder()
            .with_root(a)
            .with_root(b)
            .build()
            .expect("bigger");
        let smaller = Spec::builder().with_root(a).build().expect("smaller");
        assert!(bigger.contains_spec(&smaller));
        assert!(!smaller.contains_spec(&bigger));
    }

    #[test]
    fn contains_spec_inclusion_unrestricted_contains_filtered() {
        let r = cid_for("r");
        let unrestricted = Spec::builder().with_root(r).build().expect("u");
        let filtered = Spec::builder()
            .with_root(r)
            .with_label_allowlist(["Recipe"])
            .build()
            .expect("f");
        assert!(unrestricted.contains_spec(&filtered));
        assert!(!filtered.contains_spec(&unrestricted));
    }

    #[test]
    fn structural_path_root_has_empty_labels() {
        let p = StructuralPath::root();
        assert!(p.edge_labels().is_empty());
        assert_eq!(p.depth(), 0);
    }

    #[test]
    fn structural_path_extended_is_pure() {
        let p = StructuralPath::root();
        let p1 = p.extended("a");
        assert_eq!(p1.edge_labels(), alloc::vec!["a"]);
        assert_eq!(p.depth(), 0, "original is unchanged");
    }
}
