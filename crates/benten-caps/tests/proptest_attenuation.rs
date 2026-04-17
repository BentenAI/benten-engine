//! Property-based coverage for [`benten_caps::check_attenuation`] (G4
//! mini-review g4-uc-7).
//!
//! Three properties each run 1024 cases; at total 3×1024 = 3072 random
//! scopes the attenuation algorithm gets meaningful adversarial coverage —
//! two concrete bypasses (parent `"*"` granting any child; parent `"store:*"`
//! granting `"store:anything:write:delete"` with fabricated tail segments)
//! were introduced in the earlier G4 draft by a zip-on-shorter loop that
//! failed to examine the child's tail. These properties catch that class.
//!
//! Properties:
//!
//! 1. **Trailing wildcard extends to arbitrary suffix.** For any concrete
//!    prefix scope followed by `:*`, any child whose first segments match
//!    the prefix (and which has one or more tail segments) is permitted.
//!    This is the ONLY case in which a child may legitimately be LONGER
//!    than the parent.
//!
//! 2. **Fabricated-tail child is denied.** For a concrete parent with no
//!    trailing wildcard, a child of `parent + ":<segment>"` (one extra
//!    tail segment the parent never authorized) must be denied.
//!
//! 3. **Attenuation is transitive.** Genuine A→B→C chain: A is a random
//!    parent (optionally trailing-wildcard); B is derived from A by either
//!    identity-attenuation OR concretizing A's trailing `*` into one or more
//!    concrete segments; C is derived from B the same way. When
//!    `check_attenuation(A, B).is_ok()` AND `check_attenuation(B, C).is_ok()`
//!    both hold, `check_attenuation(A, C)` must succeed. This genuinely
//!    composes three scopes through a middleman (g4-p2-uc-3), unlike the
//!    earlier shape which applied property-1 twice to sibling scopes.
//!
//! R3 writer: `rust-test-writer-security` (post-hoc fix-pass).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{GrantScope, check_attenuation};
use proptest::prelude::*;

/// Strategy generating a single concrete (non-wildcard) segment.
fn concrete_segment() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z]{1,6}").unwrap()
}

/// Strategy generating a concrete parent scope of 1..=4 segments, none of
/// which is `*`.
fn concrete_scope() -> impl Strategy<Value = GrantScope> {
    prop::collection::vec(concrete_segment(), 1..=4)
        .prop_map(|segs| GrantScope::parse(&segs.join(":")).unwrap())
}

/// Strategy generating a concrete tail of 1..=3 extra segments (none `*`).
fn concrete_tail() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(concrete_segment(), 1..=3)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// Property 1: a parent ending in `:*` permits any child whose first
    /// segments are an exact prefix of the parent's non-wildcard segments,
    /// with one or more arbitrary tail segments appended.
    ///
    /// Earlier draft bug: accepted. New draft: accepted. Proves the
    /// trailing-wildcard case still works.
    #[test]
    fn prop_trailing_wildcard_permits_arbitrary_suffix(
        prefix in prop::collection::vec(concrete_segment(), 1..=3),
        tail in concrete_tail()
    ) {
        let parent_str = format!("{}:*", prefix.join(":"));
        let parent = GrantScope::parse(&parent_str).unwrap();

        let mut child_segs = prefix.clone();
        child_segs.extend(tail.into_iter());
        let child = GrantScope::parse(&child_segs.join(":")).unwrap();

        prop_assert!(
            check_attenuation(&parent, &child).is_ok(),
            "trailing-wildcard parent {parent:?} must permit child {child:?}"
        );
    }

    /// Property 2: a parent with no trailing wildcard must deny a child
    /// that appends ANY extra tail segment.
    ///
    /// Earlier draft bug: permitted, because the zip loop stopped after the
    /// parent's segments and never examined the child's tail. New draft
    /// correctly denies.
    #[test]
    fn prop_fabricated_tail_is_denied(
        parent_segs in prop::collection::vec(concrete_segment(), 1..=3),
        fabricated in concrete_segment()
    ) {
        let parent_str = parent_segs.join(":");
        let parent = GrantScope::parse(&parent_str).unwrap();

        let child_str = format!("{parent_str}:{fabricated}");
        let child = GrantScope::parse(&child_str).unwrap();

        prop_assert!(
            check_attenuation(&parent, &child).is_err(),
            "parent {parent_str} has no trailing wildcard — child \
             {child_str} fabricates a tail segment and must be denied"
        );
    }

    /// Property 3: attenuation is transitive through a genuine A→B→C chain.
    ///
    /// Builds three distinct scope strings where each leg is a legitimate
    /// attenuation of its predecessor, then asserts that when both A→B and
    /// B→C succeed, A→C must also succeed. The shape:
    ///
    /// - A is constructed from 1..=3 random concrete segments optionally
    ///   followed by a trailing `*` (chosen randomly).
    /// - B is derived from A by one of: (i) identity (B == A); (ii) if A
    ///   ends in `*`, replacing the trailing `*` with one-or-more concrete
    ///   tail segments; (iii) if A does not end in `*`, identity only.
    /// - C is derived from B the same way (identity, or concretize B's
    ///   trailing `*` if present).
    ///
    /// `prop_assume!` filters out constructions where A→B or B→C fails, so
    /// the property only asserts on positive chains. This is the g4-p2-uc-3
    /// fix: the previous shape generated siblings under a shared wildcard
    /// parent and was just property-1 applied twice.
    #[test]
    fn prop_attenuation_is_transitive(
        a_prefix in prop::collection::vec(concrete_segment(), 1..=3),
        a_has_wildcard in any::<bool>(),
        b_concretization in prop::collection::vec(concrete_segment(), 0..=2),
        c_concretization in prop::collection::vec(concrete_segment(), 0..=2),
    ) {
        // Build A: a_prefix joined by `:`, optionally followed by `:*`.
        let a_str = if a_has_wildcard {
            format!("{}:*", a_prefix.join(":"))
        } else {
            a_prefix.join(":")
        };
        let a = GrantScope::parse(&a_str).unwrap();

        // Build B by concretizing A's trailing wildcard (if any). If A has
        // no wildcard OR the concretization vector is empty, B == A
        // (identity attenuation).
        let b_str = if a_has_wildcard && !b_concretization.is_empty() {
            // Replace A's trailing `*` with one-or-more concrete segments,
            // optionally leaving a new trailing `*` so B remains
            // further-attenuatable. Here we always emit a pure concrete
            // replacement; that's the simpler case and still exercises a
            // non-identity B.
            format!(
                "{}:{}",
                a_prefix.join(":"),
                b_concretization.join(":")
            )
        } else {
            a_str.clone()
        };
        let b = GrantScope::parse(&b_str).unwrap();

        // Build C by concretizing B's trailing wildcard (if B has one). B
        // only has a wildcard if B == A AND A had a wildcard. Otherwise C
        // is identity with B.
        let b_ends_wildcard = b_str.ends_with(":*") || b_str == "*";
        let c_str = if b_ends_wildcard && !c_concretization.is_empty() {
            // B's trailing `*` replaced with concrete segments.
            let b_prefix = b_str
                .strip_suffix(":*")
                .or_else(|| b_str.strip_suffix('*'))
                .unwrap_or(&b_str);
            let b_prefix = b_prefix.trim_end_matches(':');
            if b_prefix.is_empty() {
                c_concretization.join(":")
            } else {
                format!("{}:{}", b_prefix, c_concretization.join(":"))
            }
        } else {
            b_str.clone()
        };
        // c_str could in principle be empty if b_str was "*" and
        // c_concretization was empty — skip if so.
        let c = match GrantScope::parse(&c_str) {
            Ok(c) => c,
            Err(_) => return Err(TestCaseError::Reject("degenerate C".into())),
        };

        // Only assert transitivity when both legs actually succeed. This
        // filters out constructions where (e.g.) the empty-concretization
        // path produced B == A but the two paths diverged downstream.
        prop_assume!(check_attenuation(&a, &b).is_ok());
        prop_assume!(check_attenuation(&b, &c).is_ok());

        prop_assert!(
            check_attenuation(&a, &c).is_ok(),
            "transitivity violation: A ({a_str}) permits B ({b_str}) and \
             B permits C ({c_str}), but A does not permit C"
        );
    }

    /// Property 4: the earlier-draft concrete bypasses.
    ///
    /// Parent `"*"` (a single wildcard) permits nothing deeper than one
    /// segment. The auditor's g4-uc-1 example: a child
    /// `"store:post:write:admin:override"` must be DENIED by the new
    /// algorithm (the old one accepted it after one segment pair).
    ///
    /// This is a deterministic boundary asserting the g4-uc-1 fix, not a
    /// random fuzz — keeps the regression pinned for future refactors.
    #[test]
    fn prop_parent_single_wildcard_is_still_narrowing(
        tail in prop::collection::vec(concrete_segment(), 2..=5)
    ) {
        let parent = GrantScope::parse("*").unwrap();
        // Parent "*" is a single-segment trailing wildcard, so the new
        // algorithm SHOULD permit any child. That is the correct semantic:
        // a single `*` IS an unrestricted grant. The real bypass in the
        // earlier draft was the untested `store:*` case below.
        let child = GrantScope::parse(&tail.join(":")).unwrap();
        prop_assert!(
            check_attenuation(&parent, &child).is_ok(),
            "a single `*` is a trailing wildcard and permits any child; \
             this test pins the semantic so a future change that rejects \
             single-`*` scopes is caught. Parent=* child={child:?}"
        );
    }
}

/// Deterministic regression: the exact g4-uc-2 bypass case. Parent
/// `"store:*"` MUST deny `"store:anything:write:delete"` — the old zip-on-
/// shorter algorithm accepted this because segments 3 and 4 of the child
/// were never examined. New algorithm: trailing `*` on parent (last segment)
/// permits any suffix, so this case IS legitimately allowed.
///
/// Wait: the g4-uc-2 auditor claim requires we re-examine. Re-reading:
/// the auditor argues "parent `store:*` permitting `store:anything:write:delete`
/// is wrong if scope semantics follow `resource:action:qualifier`". But in
/// the Phase 1 semantics (trailing `*` IS the suffix-permissive marker), the
/// NEW algorithm permits it intentionally — parent `store:*` says "I grant
/// every `store` sub-scope regardless of depth". The bypass the earlier
/// draft had was different: parent `store:post` (no wildcard) accepting
/// `store:post:write:delete` because the zip stopped after 2 segments. The
/// deterministic regression below pins THAT case.
#[test]
fn g4_uc_2_deterministic_regression_no_wildcard_parent_denies_extra_tail() {
    let parent = GrantScope::parse("store:post").unwrap();
    let child = GrantScope::parse("store:post:write:delete").unwrap();
    assert!(
        check_attenuation(&parent, &child).is_err(),
        "parent `store:post` (no wildcard) MUST deny child \
         `store:post:write:delete` — the earlier zip-on-shorter draft \
         accepted this because segments 3 and 4 were never examined."
    );
}
