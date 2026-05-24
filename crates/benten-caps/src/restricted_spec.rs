//! `RestrictedScope` — the structured 6-dimensional sub-graph restriction
//! language for G-CORE-3b (Path (a) of the RATIFIED-S&C 2026-05-21 §R1
//! decision; Path (b) refinement-witness over opaque specs was rejected
//! as structurally unsound per Spike H+1.1).
//!
//! ## The six dimensions
//!
//! 1. **roots** — `Vec<Cid>` of entry-point Node CIDs the receiver may
//!    walk from. SUBSET narrows; SUPERSET widens.
//! 2. **edge_allowlist** — `Vec<String>` of edge-label strings the walker
//!    may traverse. SUBSET narrows; SUPERSET widens.
//! 3. **max_depth** — `u32` BFS depth bound. LOWER narrows; HIGHER widens.
//! 4. **label_allowlist** — `Vec<String>` of Node-label strings the
//!    walker may visit. SUBSET narrows; SUPERSET widens.
//! 5. **label_denylist** — `Vec<String>` of Node-label strings the walker
//!    MUST NOT visit. **INVERSE DIRECTION:** SUPERSET (more denied)
//!    narrows; SUBSET (fewer denied) widens.
//! 6. **property_equalities** — `BTreeMap<String, PropertyValue>` of
//!    property `key → required-value` equalities Node properties MUST
//!    match. SUPERSET (more constraints) narrows; SUBSET widens.
//!    Conflicting values on a shared key ⇒ NOT contained.
//!
//! ## Composition discipline
//!
//! [`RestrictedScope::contains`] is `&&`-composed across all 6 dimensions
//! (per-dim narrowing checks AND-ed together). Any dimension that widens
//! breaks the whole contains. Reflexive (`s.contains(&s) == true`) and
//! transitive (`a.contains(&b) && b.contains(&c) ⇒ a.contains(&c)`).
//!
//! ## Extension-slot discipline
//!
//! Future predicate dimensions are added as NAMED fields (e.g.
//! `time_window`, `geo_bound`, `cardinality_bound`) following Spike H+1.1
//! refinement: extension slots are NAMED, NOT opaque. An opaque
//! refinement-witness arm would be structurally unsound (the witness
//! binds the attestation, not the spec body — Spike H+1.1 §b.SEC #4).
//!
//! ## Frozen surface
//!
//! `#[non_exhaustive]` is APPLIED so future named dimensions are a
//! minor-version bump (per `.addl/phase-4-meta/00-implementation-plan.md`
//! §1.A.FROZEN item 11 META #907 sweep). Six dimensions ship at v1-beta;
//! the type IS part of the v1-frozen public surface per §1.A.FROZEN
//! item 15(b).

use std::collections::BTreeMap;

use benten_core::Cid;
use serde::{Deserialize, Serialize};

/// A value that may participate in a `property_equalities` constraint.
///
/// The set of constructable shapes is intentionally narrow at v1-beta —
/// text + integer + boolean cover every property the engine itself
/// recognises (Phase-4-Foundation schema-vocab + plugin manifest
/// `requires` halves). Future ratifications may extend the enum (it is
/// `#[non_exhaustive]`); the comparison shape stays byte-equality (no
/// numeric-coercion gymnastics — that would break the audit-clarity
/// `subtle::ConstantTimeEq` discipline the cap-system uses).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PropertyValue {
    /// Text (UTF-8 string) value.
    Text(String),
    /// Signed-integer value (i64 covers every property the engine ships
    /// today; future widening is a non-breaking enum extension).
    Integer(i64),
    /// Boolean value.
    Bool(bool),
}

impl PropertyValue {
    /// Convenience constructor for text values.
    #[must_use]
    pub fn text<S: Into<String>>(s: S) -> Self {
        Self::Text(s.into())
    }

    /// Convenience constructor for integer values.
    #[must_use]
    pub const fn integer(n: i64) -> Self {
        Self::Integer(n)
    }

    /// Convenience constructor for boolean values.
    #[must_use]
    pub const fn boolean(b: bool) -> Self {
        Self::Bool(b)
    }
}

/// The structured 6-dimensional sub-graph restriction language.
///
/// See module docs for full semantics + the per-dimension narrowing
/// direction (note the INVERSE direction on `label_denylist`).
///
/// **Frozen surface — DO NOT add an opaque-refinement-witness variant**
/// (Path (b) rejected as structurally unsound per Spike H+1.1; see
/// `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
/// §R1). Future dimensions are NAMED fields added per `#[non_exhaustive]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RestrictedScope {
    /// Root Node CIDs the walker may enter from. `None` ⇒ no roots
    /// constraint (every root admissible); `Some(empty)` ⇒ no roots
    /// admissible (the empty-roots-set is the most-restrictive root
    /// constraint). Pattern mirrors the
    /// `Option<Vec<T>>` "constraint-present-vs-absent" idiom used at
    /// the cap-policy boundary.
    pub roots: Option<Vec<Cid>>,
    /// Edge-label allowlist. `None` ⇒ no constraint; `Some(set)` ⇒ only
    /// the listed labels may be traversed.
    pub edge_allowlist: Option<Vec<String>>,
    /// BFS max depth bound. `None` ⇒ unbounded (the most-permissive
    /// depth); `Some(n)` ⇒ the walker may not descend past depth `n`.
    pub max_depth: Option<u32>,
    /// Node-label allowlist.
    pub label_allowlist: Option<Vec<String>>,
    /// Node-label DENYLIST. INVERSE narrowing direction: adding entries
    /// makes the spec narrower (more labels excluded).
    pub label_denylist: Option<Vec<String>>,
    /// Property equality constraints — `key → required-value`.
    pub property_equalities: BTreeMap<String, PropertyValue>,
}

impl Default for RestrictedScope {
    fn default() -> Self {
        Self::new()
    }
}

impl RestrictedScope {
    /// Construct an empty `RestrictedScope` — every dimension unconstrained.
    /// Use the `.with_*` builder methods to add per-dimension restrictions.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            roots: None,
            edge_allowlist: None,
            max_depth: None,
            label_allowlist: None,
            label_denylist: None,
            property_equalities: BTreeMap::new(),
        }
    }

    /// Restrict the walker to entering from the given root Node CIDs.
    #[must_use]
    pub fn with_roots(mut self, roots: Vec<Cid>) -> Self {
        self.roots = Some(roots);
        self
    }

    /// G-CORE-3e convenience constructor — build a [`RestrictedScope`]
    /// whose `roots` set is the supplied list of CIDs (the ciphertext-
    /// hashes the recipient is allowed to request). Equivalent to
    /// `Self::new().with_roots(hashes)` but the `with_hashes` name
    /// reads more naturally at the wave-3e ALPN-handler call site
    /// where the CIDs are "ciphertext hashes the recipient may serve"
    /// rather than "graph entry-point roots."
    ///
    /// This is the simplest constructor for the wave-3e per-request
    /// scope-check arm; the full 6-dimensional restriction language is
    /// available via the `.with_*` builders.
    #[must_use]
    pub fn with_hashes(hashes: Vec<Cid>) -> Self {
        Self::new().with_roots(hashes)
    }

    /// Restrict the walker to traversing only the given edge labels.
    #[must_use]
    pub fn with_edge_allowlist(mut self, labels: Vec<String>) -> Self {
        self.edge_allowlist = Some(labels);
        self
    }

    /// Bound the walker's BFS descent to `depth` levels.
    #[must_use]
    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Restrict the walker to visiting only the given Node labels.
    #[must_use]
    pub fn with_label_allowlist(mut self, labels: Vec<String>) -> Self {
        self.label_allowlist = Some(labels);
        self
    }

    /// Forbid the walker from visiting the given Node labels. Inverse
    /// narrowing direction — adding entries makes the spec narrower.
    #[must_use]
    pub fn with_label_denylist(mut self, labels: Vec<String>) -> Self {
        self.label_denylist = Some(labels);
        self
    }

    /// Require Node properties to carry `key == value` to be admitted.
    /// Multiple calls accumulate (AND).
    #[must_use]
    pub fn with_property_equality(mut self, key: String, value: PropertyValue) -> Self {
        self.property_equalities.insert(key, value);
        self
    }

    /// Decidable structural containment.
    ///
    /// Returns `true` iff `other` is no-wider-than `self` along every
    /// one of the 6 dimensions. The check is AND-composed (per Spike
    /// H+1.1's composability claim): any single widening dimension
    /// breaks containment.
    ///
    /// **Reflexive** (`s.contains(&s) == true`) — required by the chain
    /// validator's no-op-step admission. **Transitive** across the
    /// 6-dim product. Decidable in `O(roots + labels + props)` — no
    /// fixpoint iteration.
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        self.roots_contains(other)
            && self.edge_allowlist_contains(other)
            && self.max_depth_contains(other)
            && self.label_allowlist_contains(other)
            && self.label_denylist_contains(other)
            && self.property_equalities_contains(other)
    }

    /// `None` (= unconstrained / most-permissive) on `self` always
    /// contains any constraint on `other`. `Some(parent)` contains
    /// `Some(child)` iff every child root is in the parent set; and
    /// `Some(parent)` does NOT contain `None(child)` (an unconstrained
    /// child WIDENS a constrained parent).
    fn roots_contains(&self, other: &Self) -> bool {
        allowlist_subset_check(self.roots.as_deref(), other.roots.as_deref())
    }

    fn edge_allowlist_contains(&self, other: &Self) -> bool {
        allowlist_subset_check(
            self.edge_allowlist.as_deref(),
            other.edge_allowlist.as_deref(),
        )
    }

    fn max_depth_contains(&self, other: &Self) -> bool {
        match (self.max_depth, other.max_depth) {
            // unconstrained-self contains any child depth.
            (None, _) => true,
            // bounded-self does NOT contain unconstrained-child (widens).
            (Some(_), None) => false,
            // bounded-both: child depth must be <= parent depth.
            (Some(parent), Some(child)) => child <= parent,
        }
    }

    fn label_allowlist_contains(&self, other: &Self) -> bool {
        allowlist_subset_check(
            self.label_allowlist.as_deref(),
            other.label_allowlist.as_deref(),
        )
    }

    /// INVERSE direction — child must DENY a SUPERSET of parent's
    /// denylist to be narrower.
    fn label_denylist_contains(&self, other: &Self) -> bool {
        match (
            self.label_denylist.as_deref(),
            other.label_denylist.as_deref(),
        ) {
            // No parent denylist ⇒ anything child denies is a narrowing.
            (None, _) => true,
            // Parent denies some labels; child denies none ⇒ widens.
            (Some(_parent), None) => false,
            // Both denylists present: every parent-denied label must also
            // be child-denied (child denies a superset).
            (Some(parent), Some(child)) => parent.iter().all(|p| child.iter().any(|c| c == p)),
        }
    }

    /// Child must require every parent property equality (superset on
    /// the equality SET) and must AGREE on every shared key (no
    /// conflicting value).
    fn property_equalities_contains(&self, other: &Self) -> bool {
        for (k, v_parent) in &self.property_equalities {
            match other.property_equalities.get(k) {
                None => return false, // child drops a parent constraint ⇒ widens
                Some(v_child) if v_child != v_parent => return false, // conflict
                Some(_) => {}
            }
        }
        true
    }
}

/// Shared "allowlist subset" helper for the `roots` /
/// `edge_allowlist` / `label_allowlist` dimensions. Pulls the
/// `None=unconstrained`-narrowing-direction logic into one place so all
/// three allowlist dimensions enforce the identical rule.
///
/// Returns `true` iff `child` is no-wider-than `parent` (child is a
/// subset of parent, or parent is unconstrained).
fn allowlist_subset_check<T: PartialEq>(parent: Option<&[T]>, child: Option<&[T]>) -> bool {
    match (parent, child) {
        // unconstrained-parent contains any child.
        (None, _) => true,
        // constrained-parent does NOT contain unconstrained-child (widens).
        (Some(_), None) => false,
        // both constrained: every child element must be in parent.
        (Some(parent), Some(child)) => child.iter().all(|c| parent.iter().any(|p| p == c)),
    }
}
