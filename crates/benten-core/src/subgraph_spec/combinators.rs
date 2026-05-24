//! `intersect / union / filter` combinators over [`Spec`].
//!
//! Structural-containment-preserving (proptest verified at
//! `tf3w_combinators_intersect_union_filter_proptest.rs`):
//!
//! - `intersect(a, b)` ⊆ a + ⊆ b  (narrows)
//! - a ⊆ union(a, b) + b ⊆ union(a, b)  (widens)
//! - filter(a, p) ⊆ a  (narrows; p can only constrain)
//!
//! Commutativity + associativity hold on canonical bytes.

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use super::errors::SubgraphSpecError;
use super::spec::{Spec, SpecBuilder, SubgraphSpecRestriction};
use crate::Cid;

/// `intersect(a, b)` — produce a [`Spec`] contained by both `a` and `b`.
///
/// Semantics:
///   * roots = a.roots ∩ b.roots
///   * edges = the (source, label, target) triples present in BOTH specs
///   * max_depth = min(a.max_depth, b.max_depth)
///   * inclusion = the narrower restricted-spec (allowlist intersect;
///     denylist union)
///
/// If the result has no roots or its inclusion-predicate becomes
/// structurally empty (allow ∩ deny non-empty over the merged label set),
/// returns [`SubgraphSpecError::EmptyIntersection`].
///
/// # Errors
/// See variant docs.
pub fn intersect(a: &Spec, b: &Spec) -> Result<Spec, SubgraphSpecError> {
    let a_roots: BTreeSet<Cid> = a.roots().iter().copied().collect();
    let b_roots: BTreeSet<Cid> = b.roots().iter().copied().collect();
    let roots_intersect: BTreeSet<Cid> = a_roots.intersection(&b_roots).copied().collect();

    if roots_intersect.is_empty() {
        return Err(SubgraphSpecError::EmptyIntersection);
    }

    let max_depth = core::cmp::min(a.max_depth(), b.max_depth());

    // Inclusion: intersect allow lists (narrower); union deny lists (stricter).
    let (allow, deny) = intersect_inclusions(a.inclusion(), b.inclusion())?;

    // Build-time conflict check: by the inputs each having passed their
    // own validation, this should be impossible. Defensive check only.
    if let Some(conflict) = allow.intersection(&deny).next() {
        return Err(SubgraphSpecError::ConflictingLabelPredicates {
            label: conflict.clone(),
        });
    }

    // Edges: intersection of declared edges.
    let mut builder = SpecBuilder::new().with_max_depth(max_depth);
    for r in &roots_intersect {
        builder = builder.with_root(*r);
    }
    for (src, edges_a) in a.edges() {
        if let Some(edges_b) = b.edges().get(src) {
            for (label, target) in edges_a {
                if edges_b.get(label) == Some(target) {
                    builder = builder.with_edge(*src, label.clone(), *target);
                }
            }
        }
    }
    if !allow.is_empty() {
        let allow_vec: Vec<String> = allow.iter().cloned().collect();
        builder = builder.with_label_allowlist(allow_vec);
    }
    if !deny.is_empty() {
        let deny_vec: Vec<String> = deny.iter().cloned().collect();
        builder = builder.with_label_denylist(deny_vec);
    }
    builder.build()
}

/// `union(a, b)` — produce a [`Spec`] containing both `a` and `b`.
///
/// Semantics:
///   * roots = a.roots ∪ b.roots
///   * edges = the union of (source, label, target) triples
///     (collisions: same `(source, label)` with different targets is
///     resolved last-write-wins via the underlying builder; in
///     well-formed inputs this should not occur)
///   * max_depth = max(a.max_depth, b.max_depth)
///   * inclusion = the wider restricted-spec (allow union; deny intersect)
///
/// # Errors
/// Returns [`SubgraphSpecError::ConflictingLabelPredicates`] if the
/// widened allow/deny sets overlap (should be impossible for well-formed
/// inputs).
pub fn union(a: &Spec, b: &Spec) -> Result<Spec, SubgraphSpecError> {
    let mut builder =
        SpecBuilder::new().with_max_depth(core::cmp::max(a.max_depth(), b.max_depth()));

    // Roots union.
    for r in a.roots().iter().chain(b.roots().iter()) {
        builder = builder.with_root(*r);
    }

    // Edges union.
    for (src, edges) in a.edges().iter().chain(b.edges().iter()) {
        for (label, target) in edges {
            builder = builder.with_edge(*src, label.clone(), *target);
        }
    }

    // Inclusion: widen.
    let (allow, deny) = union_inclusions(a.inclusion(), b.inclusion());
    if !allow.is_empty() {
        let allow_vec: Vec<String> = allow.iter().cloned().collect();
        builder = builder.with_label_allowlist(allow_vec);
    }
    if !deny.is_empty() {
        let deny_vec: Vec<String> = deny.iter().cloned().collect();
        builder = builder.with_label_denylist(deny_vec);
    }

    builder.build()
}

/// `filter(spec, narrow_fn)` — produce a [`Spec`] narrower than `spec` by
/// applying a builder-narrowing closure.
///
/// The closure receives a [`SpecBuilder`] pre-seeded with `spec`'s
/// content and may only **narrow** (deny additional labels, lower
/// `max_depth`, remove roots — though removal is not currently supported
/// at the builder surface). The pin proptest verifies
/// `filter(a) ⊆ a` for all `a`.
///
/// # Errors
/// Surfaces any error from the closure's final `.build()`.
pub fn filter<F>(spec: &Spec, narrow: F) -> Result<Spec, SubgraphSpecError>
where
    F: FnOnce(SpecBuilder) -> SpecBuilder,
{
    let mut builder = SpecBuilder::new().with_max_depth(spec.max_depth());
    for r in spec.roots() {
        builder = builder.with_root(*r);
    }
    for (src, edges) in spec.edges() {
        for (label, target) in edges {
            builder = builder.with_edge(*src, label.clone(), *target);
        }
    }
    // Re-apply the input's inclusion predicate.
    match spec.inclusion() {
        SubgraphSpecRestriction::Unrestricted => {}
        SubgraphSpecRestriction::ByLabel { allow, deny } => {
            if !allow.is_empty() {
                let v: Vec<String> = allow.iter().cloned().collect();
                builder = builder.with_label_allowlist(v);
            }
            if !deny.is_empty() {
                let v: Vec<String> = deny.iter().cloned().collect();
                builder = builder.with_label_denylist(v);
            }
        }
    }

    // Apply the narrowing closure. The closure may introduce a label
    // into `deny` that was already in `allow` (the canonical "filter
    // narrows by adding to deny" idiom); since deny dominates allow
    // semantically (deny excludes regardless of allowlist membership),
    // we rewrite the narrowed builder to remove from `allow` any label
    // newly present in `deny`. This preserves the
    // ConflictingLabelPredicates build-time guard for genuinely-
    // contradictory user input while letting `filter` honor its
    // "narrowing-only" contract.
    let mut narrowed = narrow(builder);
    let conflicts: Vec<String> = narrowed
        .allow_labels
        .intersection(&narrowed.deny_labels)
        .cloned()
        .collect();
    for c in conflicts {
        narrowed.allow_labels.remove(&c);
    }
    narrowed.build()
}

// ---------------------------------------------------------------------------
// Helpers for inclusion-dim intersect / union.
// ---------------------------------------------------------------------------

/// Returns `Ok((allow, deny))` on success; `Err(EmptyIntersection)` when
/// both inputs supply a non-empty allowlist but their intersection is
/// empty — semantically "no labels are allowed by both sides at once",
/// which collapses to the empty Spec.
fn intersect_inclusions(
    a: &SubgraphSpecRestriction,
    b: &SubgraphSpecRestriction,
) -> Result<(BTreeSet<String>, BTreeSet<String>), SubgraphSpecError> {
    match (a, b) {
        (SubgraphSpecRestriction::Unrestricted, SubgraphSpecRestriction::Unrestricted) => {
            Ok((BTreeSet::new(), BTreeSet::new()))
        }
        (
            SubgraphSpecRestriction::Unrestricted,
            SubgraphSpecRestriction::ByLabel { allow, deny },
        )
        | (
            SubgraphSpecRestriction::ByLabel { allow, deny },
            SubgraphSpecRestriction::Unrestricted,
        ) => Ok((allow.clone(), deny.clone())),
        (
            SubgraphSpecRestriction::ByLabel {
                allow: a_allow,
                deny: a_deny,
            },
            SubgraphSpecRestriction::ByLabel {
                allow: b_allow,
                deny: b_deny,
            },
        ) => {
            // Narrower allowlist: if both non-empty, intersection;
            // if one is empty (unrestricted-on-that-axis), take the other.
            // If both non-empty AND intersection is empty, the intersect
            // collapses to the empty Spec.
            let allow = match (a_allow.is_empty(), b_allow.is_empty()) {
                (true, true) => BTreeSet::new(),
                (true, false) => b_allow.clone(),
                (false, true) => a_allow.clone(),
                (false, false) => {
                    let inter: BTreeSet<String> = a_allow.intersection(b_allow).cloned().collect();
                    if inter.is_empty() {
                        return Err(SubgraphSpecError::EmptyIntersection);
                    }
                    inter
                }
            };
            let deny: BTreeSet<String> = a_deny.union(b_deny).cloned().collect();
            // After deny-union, if every label in `allow` is also in `deny`
            // the Spec admits nothing — collapse to EmptyIntersection.
            if !allow.is_empty() && allow.is_subset(&deny) {
                return Err(SubgraphSpecError::EmptyIntersection);
            }
            Ok((allow, deny))
        }
    }
}

fn union_inclusions(
    a: &SubgraphSpecRestriction,
    b: &SubgraphSpecRestriction,
) -> (BTreeSet<String>, BTreeSet<String>) {
    match (a, b) {
        // Either side unrestricted widens to unrestricted.
        (SubgraphSpecRestriction::Unrestricted, _) | (_, SubgraphSpecRestriction::Unrestricted) => {
            (BTreeSet::new(), BTreeSet::new())
        }
        (
            SubgraphSpecRestriction::ByLabel {
                allow: a_allow,
                deny: a_deny,
            },
            SubgraphSpecRestriction::ByLabel {
                allow: b_allow,
                deny: b_deny,
            },
        ) => {
            // Wider allowlist: union; if either is "unrestricted-on-that-axis"
            // (empty), the union is unrestricted = empty.
            let allow = if a_allow.is_empty() || b_allow.is_empty() {
                BTreeSet::new()
            } else {
                a_allow.union(b_allow).cloned().collect()
            };
            // Stricter denylist: intersection. (Only labels denied on
            // BOTH sides remain denied in the union.)
            let deny: BTreeSet<String> = a_deny.intersection(b_deny).cloned().collect();
            (allow, deny)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cid;
    use alloc::string::ToString;

    fn cid_for(label: &str) -> Cid {
        let digest = blake3::hash(label.as_bytes());
        Cid::from_blake3_digest(*digest.as_bytes())
    }

    #[test]
    fn intersect_narrows_simple() {
        let r = cid_for("r");
        let a = Spec::builder().with_root(r).build().unwrap();
        let b = Spec::builder().with_root(r).build().unwrap();
        let ab = intersect(&a, &b).expect("intersect");
        assert!(a.contains_spec(&ab));
        assert!(b.contains_spec(&ab));
    }

    #[test]
    fn union_widens_simple() {
        let r1 = cid_for("r1");
        let r2 = cid_for("r2");
        let a = Spec::builder().with_root(r1).build().unwrap();
        let b = Spec::builder().with_root(r2).build().unwrap();
        let ab = union(&a, &b).expect("union");
        assert!(ab.contains_spec(&a));
        assert!(ab.contains_spec(&b));
    }

    #[test]
    fn intersect_disjoint_roots_empty() {
        let a = Spec::builder().with_root(cid_for("a")).build().unwrap();
        let b = Spec::builder().with_root(cid_for("b")).build().unwrap();
        let err = intersect(&a, &b).expect_err("disjoint roots");
        assert!(matches!(err, SubgraphSpecError::EmptyIntersection));
    }

    #[test]
    fn filter_narrows_simple() {
        let r = cid_for("r");
        let a = Spec::builder().with_root(r).build().unwrap();
        let filtered = filter(&a, |b| b.deny_label("X")).expect("filter");
        assert!(a.contains_spec(&filtered));
    }

    #[test]
    fn intersect_commutative_canonical_bytes() {
        let r = cid_for("r");
        let s = cid_for("s");
        let a = Spec::builder().with_root(r).with_root(s).build().unwrap();
        let b = Spec::builder().with_root(r).build().unwrap();
        let ab = intersect(&a, &b).expect("ab");
        let ba = intersect(&b, &a).expect("ba");
        assert_eq!(
            ab.to_canonical_bytes().unwrap(),
            ba.to_canonical_bytes().unwrap()
        );
    }

    #[test]
    fn intersect_with_overlapping_inclusion_narrows() {
        let r = cid_for("r");
        let a = Spec::builder()
            .with_root(r)
            .with_label_allowlist(["X", "Y"])
            .build()
            .unwrap();
        let b = Spec::builder()
            .with_root(r)
            .with_label_allowlist(["Y", "Z"])
            .build()
            .unwrap();
        let ab = intersect(&a, &b).expect("intersect");
        // ab.inclusion should have only "Y" in allowlist.
        match ab.inclusion() {
            SubgraphSpecRestriction::ByLabel { allow, .. } => {
                assert_eq!(allow.len(), 1);
                assert!(allow.contains("Y"));
            }
            _ => panic!("expected ByLabel inclusion"),
        }
    }

    #[test]
    fn union_widens_with_overlapping_inclusion() {
        let r = cid_for("r");
        let a = Spec::builder()
            .with_root(r)
            .with_label_allowlist(["X"])
            .build()
            .unwrap();
        let b = Spec::builder()
            .with_root(r)
            .with_label_allowlist(["Y"])
            .build()
            .unwrap();
        let ab = union(&a, &b).expect("union");
        // ab.inclusion has both X + Y in allowlist.
        match ab.inclusion() {
            SubgraphSpecRestriction::ByLabel { allow, .. } => {
                assert!(allow.contains("X"));
                assert!(allow.contains("Y"));
            }
            _ => panic!("expected ByLabel inclusion"),
        }
    }
}
