//! G27-C — CID-keyed reader companion matches scope-keyed reader for
//! consistent inputs.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.16 G27-C row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-C entry.
//!
//! ## The consistency invariant
//!
//! G27-C lifts the scope-keyed `has_unrevoked_grant_for_scope` AND
//! adds the CID-keyed `has_unrevoked_grant_for_grant_cid` companion.
//! Both methods consult the SAME revocation substrate; only the key
//! shape differs. The consistency invariant: for any grant Node
//! stored with `(scope, cid)`, the two readers MUST AGREE on the
//! revocation state — `has_unrevoked_grant_for_scope(scope)` and
//! `has_unrevoked_grant_for_grant_cid(cid)` return the same bool
//! when both queries reference the same logical grant.
//!
//! Why this matters: at consume sites (engine seams; policy
//! `check_write` callers), some hold a CID handle + some hold a
//! scope string. If the two readers disagreed under any condition,
//! a write that LOOKS revoked through one reader would LOOK active
//! through the other — a fail-OPEN race window between the napi
//! revoke-by-CID seam + the policy scope-string check.
//!
//! ## Pin shape — disagreement detection
//!
//! 1. Construct an in-memory `GrantReader` carrying a grant
//!    (scope = "store:notes:write", cid = synthetic_cid_1).
//! 2. Assert
//!    `has_unrevoked_grant_for_scope("store:notes:write") ==
//!     has_unrevoked_grant_for_grant_cid(&synthetic_cid_1)`.
//! 3. Apply revocation; assert agreement still holds.
//!
//! ## RED-PHASE expectation
//!
//! As with the round-trip sister test, the `benten_id::grant_reader`
//! module does not exist at HEAD. Un-ignore at G27-C wave-time after
//! the module + trait land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// GREEN-PHASE (G27-C un-ignored 2026-05-11): `benten_id::grant_reader`
// landed at this wave; cfg(any()) gate dropped + inner test invoked
// by the outer `#[test]`. The pin asserts the consistency invariant
// between the scope-keyed + CID-keyed paths under pre-revoke + post-
// revoke states.

mod red_phase_compile_witness {
    use benten_core::Cid;
    use benten_id::grant_reader::GrantReader;
    use std::collections::HashMap;

    struct InMemoryReader {
        /// scope-string → set of grant CIDs that hold this scope
        scope_to_cids: HashMap<String, Vec<Cid>>,
        /// grant CIDs that are revoked
        revoked_cids: std::collections::HashSet<Cid>,
        /// scope strings that have any active revocation
        revoked_scopes: std::collections::HashSet<String>,
    }

    impl GrantReader for InMemoryReader {
        fn has_unrevoked_grant_for_scope(
            &self,
            scope: &str,
        ) -> Result<bool, benten_id::grant_reader::ReaderError> {
            if self.revoked_scopes.contains(scope) {
                return Ok(false);
            }
            Ok(self
                .scope_to_cids
                .get(scope)
                .is_some_and(|cids| !cids.is_empty()))
        }

        fn has_unrevoked_grant_for_grant_cid(
            &self,
            grant_cid: &Cid,
        ) -> Result<bool, benten_id::grant_reader::ReaderError> {
            if self.revoked_cids.contains(grant_cid) {
                return Ok(false);
            }
            Ok(self
                .scope_to_cids
                .values()
                .any(|cids| cids.contains(grant_cid)))
        }
    }

    fn synthetic_cid(seed: &[u8]) -> Cid {
        let node = benten_core::Node::new(
            vec!["test:fixture".into()],
            std::iter::once(("seed".to_string(), benten_core::Value::Bytes(seed.to_vec())))
                .collect(),
        );
        node.cid().unwrap()
    }

    pub fn cid_keyed_matches_scope_keyed_for_consistent_inputs() {
        let cid_1 = synthetic_cid(b"grant-1");
        let scope = "store:notes:write".to_string();
        let mut reader = InMemoryReader {
            scope_to_cids: HashMap::from([(scope.clone(), vec![cid_1])]),
            revoked_cids: Default::default(),
            revoked_scopes: Default::default(),
        };

        let scope_result = reader.has_unrevoked_grant_for_scope(&scope).unwrap();
        let cid_result = reader.has_unrevoked_grant_for_grant_cid(&cid_1).unwrap();
        assert_eq!(
            scope_result, cid_result,
            "pre-revoke: both readers must agree on the unrevoked grant"
        );
        assert!(scope_result, "pre-revoke: grant must be active");

        reader.revoked_cids.insert(cid_1);
        reader.revoked_scopes.insert(scope.clone());

        let scope_post = reader.has_unrevoked_grant_for_scope(&scope).unwrap();
        let cid_post = reader.has_unrevoked_grant_for_grant_cid(&cid_1).unwrap();
        assert_eq!(
            scope_post, cid_post,
            "post-revoke: both readers must agree on the revoked grant"
        );
        assert!(!scope_post, "post-revoke: grant must be inactive");
    }
}

/// GREEN-PHASE (G27-C un-ignored 2026-05-11): invokes the inner
/// `cid_keyed_matches_scope_keyed_for_consistent_inputs()` test that
/// exercises the consistency invariant.
#[test]
fn grant_reader_cid_keyed_companion_matches_scope_keyed_for_consistent_inputs() {
    red_phase_compile_witness::cid_keyed_matches_scope_keyed_for_consistent_inputs();
}
