//! Property-based coverage for [`benten_caps::check_attenuation`] (G4
//! mini-review g4-uc-7).
//!
//! Case count is governed by the standard `PROPTEST_CASES` env var (CI sets
//! this to 1024 on every PR; fuzz.yml bumps it to 10 000 on the nightly run;
//! local `cargo nextest run` defaults to proptest's built-in 256). The
//! earlier revision hardcoded `with_cases(1024)` which overrode the env var
//! and defeated the scale-up path — fixed as R6b-TA-4.
//!
//! At the CI budget the attenuation algorithm gets meaningful adversarial
//! coverage — two concrete bypasses (parent `"*"` granting any child; parent
//! `"store:*"` granting `"store:anything:write:delete"` with fabricated tail
//! segments) were introduced in the earlier G4 draft by a zip-on-shorter
//! loop that failed to examine the child's tail. These properties catch
//! that class.
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
    // R6b-TA-4: use the default ProptestConfig so `PROPTEST_CASES` (set by
    // CI to 1024 on PR, 10 000 on nightly fuzz.yml) actually takes effect.
    // The earlier hardcoded `with_cases(1024)` overrode the env var and
    // pinned the case count regardless of how CI dialed it up or down.
    #![proptest_config(ProptestConfig::default())]

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

    /// Property 4: lone `*` is rejected at parse time (ucca-7 semantic).
    ///
    /// Pre-ucca-7 this test pinned the earlier semantic that a single `*`
    /// segment was a trailing wildcard granting unrestricted authority.
    /// ucca-7 flipped that: `GrantScope::parse("*")` now returns
    /// `Err(CapError::ScopeLoneStarRejected)` to close the root-scope
    /// footgun. A grantor who intends "any sub-scope under `ns`" must
    /// name the namespace (`ns:*`) — the compound-wildcard form exercised
    /// below.
    ///
    /// The original test unconditionally called `GrantScope::parse("*")
    /// .unwrap()` and panicked deterministically on every draw after
    /// ucca-7 landed (mis-filed as "flaky" in D12.6; the failure was
    /// universal, not intermittent). Rewritten to pin the real
    /// post-ucca-7 contract in two halves:
    ///
    /// 1. Lone `*` MUST be rejected at parse time (`ScopeLoneStarRejected`).
    /// 2. A namespace-anchored compound wildcard (`<anchor>:*`) MUST still
    ///    permit any child starting with `<anchor>` and one-or-more tail
    ///    segments — the trailing-wildcard semantic is scoped to the
    ///    compound case only, not the lone-`*` case.
    ///
    /// Test-name preserved because the CI exclusion in
    /// `.github/workflows/ci.yml` refers to it by name.
    #[test]
    fn prop_parent_single_wildcard_is_still_narrowing(
        tail in prop::collection::vec(concrete_segment(), 2..=5)
    ) {
        // ucca-7 contract: lone `*` refused at construction.
        match GrantScope::parse("*") {
            Err(benten_caps::CapError::ScopeLoneStarRejected) => {}
            other => {
                prop_assert!(
                    false,
                    "ucca-7 contract violated: GrantScope::parse(\"*\") \
                     must return Err(ScopeLoneStarRejected), got {other:?}"
                );
            }
        }

        // Compound `<anchor>:*` still permits an arbitrary tail.
        let (anchor, rest) = tail.split_first().expect("tail is 2..=5");
        let parent_str = format!("{anchor}:*");
        let parent = GrantScope::parse(&parent_str).unwrap();
        let mut child_segs = vec![anchor.clone()];
        child_segs.extend(rest.iter().cloned());
        let child = GrantScope::parse(&child_segs.join(":")).unwrap();
        prop_assert!(
            check_attenuation(&parent, &child).is_ok(),
            "compound trailing-wildcard `{parent_str}` must still permit \
             child {child:?}; lone-`*` rejection is scoped to the \
             single-segment case only"
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

// R3 security writer — added 2026-04-21 per plan §4.2 + ucca-6 seed.
// Append-only per R2 watch-1 discipline — no edits to existing blocks above.
//
// Phase 2a R3 security — chain-depth-5 transitivity (ucca-6 seed).
//
// Extends the Phase-1 3-chain (prop_attenuation_is_transitive) to a 5-chain
// A → B → C → D → E. The 3-chain catches non-transitivity at one hop; the
// 5-chain catches "drift over depth" bugs that a 3-chain would miss — e.g.
// an algorithm that accepts N-1 legs pairwise but silently relaxes a
// constraint at the fourth hop.
//
// Per ucca-6: Phase-6 AI agent chains can be deep (user → agent → tool →
// sub-tool → ...). 5 is the minimum depth where a class of
// "accumulation-of-small-errors" bugs can appear.
//
// Case count: governed by PROPTEST_CASES (CI 1024; nightly fuzz.yml 10k).

proptest! {
    #![proptest_config(ProptestConfig::default())]

    /// ucca-6: 5-hop attenuation is transitive.
    ///
    /// Builds A → B → C → D → E where each leg is a legitimate attenuation
    /// of its predecessor via identity OR concretizing a trailing `*`. Uses
    /// `prop_assume!` to filter non-chains; asserts that when every
    /// adjacent pair is permitted, end-to-end (A, E) is also permitted.
    ///
    /// Catches non-transitivity bugs a depth-3 chain misses — e.g. an
    /// algorithm that accumulates off-by-one tail handling through
    /// multiple wildcard-concretization hops.
    #[test]
    fn chain_depth_5_transitivity(
        a_prefix in prop::collection::vec(concrete_segment(), 1..=3),
        a_has_wildcard in any::<bool>(),
        b_concretization in prop::collection::vec(concrete_segment(), 0..=2),
        c_concretization in prop::collection::vec(concrete_segment(), 0..=2),
        d_concretization in prop::collection::vec(concrete_segment(), 0..=2),
        e_concretization in prop::collection::vec(concrete_segment(), 0..=2),
    ) {
        // A: concrete prefix, optionally trailing `:*`.
        let a_str = if a_has_wildcard {
            format!("{}:*", a_prefix.join(":"))
        } else {
            a_prefix.join(":")
        };
        let a = GrantScope::parse(&a_str).unwrap();

        // Helper: if `scope_str` ends with `:*` and `concretization` is
        // non-empty, replace the trailing `*` with concrete segments,
        // optionally leaving a new trailing `:*` so the next hop can
        // further concretize. Otherwise return identity (new == old).
        let derive = |scope_str: &str, concretization: &[String]| -> String {
            if scope_str.ends_with(":*") && !concretization.is_empty() {
                let prefix = scope_str.strip_suffix(":*").unwrap_or(scope_str);
                let prefix = prefix.trim_end_matches(':');
                // Emit a NEW trailing `:*` 50% of the time (determined by
                // concretization.len() parity) so the chain can continue
                // further-attenuating; otherwise terminate in a concrete.
                // This keeps the chain live across 5 hops.
                if concretization.len() % 2 == 1 {
                    if prefix.is_empty() {
                        format!("{}:*", concretization.join(":"))
                    } else {
                        format!("{}:{}:*", prefix, concretization.join(":"))
                    }
                } else if prefix.is_empty() {
                    concretization.join(":")
                } else {
                    format!("{}:{}", prefix, concretization.join(":"))
                }
            } else if scope_str == "*" && !concretization.is_empty() {
                // Lone `*` is a trailing wildcard at depth 0.
                concretization.join(":")
            } else {
                scope_str.to_string()
            }
        };

        let b_str = derive(&a_str, &b_concretization);
        let b = match GrantScope::parse(&b_str) {
            Ok(x) => x,
            Err(_) => return Err(TestCaseError::Reject("degenerate B".into())),
        };

        let c_str = derive(&b_str, &c_concretization);
        let c = match GrantScope::parse(&c_str) {
            Ok(x) => x,
            Err(_) => return Err(TestCaseError::Reject("degenerate C".into())),
        };

        let d_str = derive(&c_str, &d_concretization);
        let d = match GrantScope::parse(&d_str) {
            Ok(x) => x,
            Err(_) => return Err(TestCaseError::Reject("degenerate D".into())),
        };

        let e_str = derive(&d_str, &e_concretization);
        let e = match GrantScope::parse(&e_str) {
            Ok(x) => x,
            Err(_) => return Err(TestCaseError::Reject("degenerate E".into())),
        };

        // Only assert transitivity when every adjacent leg permits. The
        // filter makes the property a ONE-DIRECTION implication
        // (legs-all-ok ⇒ endpoints-ok); the contrapositive falls out of
        // the generator's wildcard-only-concretization discipline.
        prop_assume!(check_attenuation(&a, &b).is_ok());
        prop_assume!(check_attenuation(&b, &c).is_ok());
        prop_assume!(check_attenuation(&c, &d).is_ok());
        prop_assume!(check_attenuation(&d, &e).is_ok());

        prop_assert!(
            check_attenuation(&a, &e).is_ok(),
            "depth-5 transitivity violation: every adjacent pair permits \
             but endpoint (A, E) does not. \
             A={a_str} B={b_str} C={c_str} D={d_str} E={e_str}"
        );
    }
}
