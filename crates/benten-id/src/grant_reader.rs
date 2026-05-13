//! G27-C — `benten-id` sibling `GrantReader` trait with CID-keyed
//! companion method.
//!
//! ## Why a sibling trait (and not extending `benten-caps::GrantReader`)
//!
//! Per **arch-r1-10**, `benten-id` MUST NOT depend on `benten-caps`
//! (see `crates/benten-id/tests/dependency_edges.rs`). The existing
//! reader surface lives at `benten_caps::grant_backed::GrantReader`
//! with the single scope-keyed method
//! [`has_unrevoked_grant_for_scope`]:
//! `fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError>`.
//!
//! Lifting that trait into `benten-id` would violate the layering
//! contract. Instead, this module mints a SIBLING trait — same shape
//! at the API surface plus the CID-keyed companion the §13.11
//! structural lesson surfaces as missing. The two traits coexist;
//! implementations (test fixtures, the engine's `BackendGrantReader`,
//! the UCAN-grounded backend) may implement either or both depending
//! on what their consume sites need. No method conflict can occur —
//! a single concrete type that wants to implement BOTH disambiguates
//! method calls via `<T as benten_id::grant_reader::GrantReader>::…`
//! / `<T as benten_caps::grant_backed::GrantReader>::…` UFCS.
//!
//! [`has_unrevoked_grant_for_scope`]: GrantReader::has_unrevoked_grant_for_scope
//!
//! ## The structural gap this closes (§13.11)
//!
//! Pre-G27-C the reader API is scope-string-keyed only. The
//! `revokeCapability(grantCid, actor)` PR #199 fail-OPEN root cause
//! was that callers holding a `&Cid` (the canonical content-addressed
//! handle for a grant Node) had no typed reader API to consult — they
//! had to round-trip through the engine seam, resolve the grant Node,
//! pull its `scope` property, then call back into the scope-keyed
//! reader. Any caller that skipped that round-trip (or got the scope
//! resolution wrong, as PR #199 originally did via a namespace
//! mismatch) silently fell back to "no matching scope → no
//! revocation → grant still active." That class of bug is what the
//! `has_unrevoked_grant_for_grant_cid(&Cid)` companion forecloses at
//! the trait surface: the CID is the canonical handle, no scope
//! resolution required to consult the revocation substrate.
//!
//! ## Consistency invariant between the two methods
//!
//! For any logical grant `(scope, cid)` stored in a `GrantReader`
//! implementation, the following must hold under every revocation
//! state:
//!
//! ```ignore
//! reader.has_unrevoked_grant_for_scope(scope)? ==
//! reader.has_unrevoked_grant_for_grant_cid(&cid)?
//! ```
//!
//! The consistency invariant is the load-bearing safety property:
//! without it a write that LOOKS revoked through the scope-keyed
//! reader would LOOK active through the CID-keyed reader (a
//! fail-OPEN race window between the napi revoke-by-CID seam + the
//! policy scope-string check). Implementations are responsible for
//! consulting the SAME revocation substrate from both call paths;
//! the trait does not enforce this structurally (the two methods
//! are independent dispatch sites). The
//! `crates/benten-id/tests/grant_reader_cid_keyed_companion_matches_scope_keyed_for_consistent_inputs.rs`
//! RED-PHASE pin (un-ignored at G27-C wave-time) exercises the
//! invariant on the canonical in-memory shape.

use benten_core::Cid;
use thiserror::Error;

/// Typed-error surface for `benten-id` `GrantReader` implementations.
///
/// Sibling to `benten_caps::error::CapError::Denied` at the
/// `benten-caps::GrantReader` boundary; minted fresh here because
/// `benten-id` cannot depend on `benten-caps` per arch-r1-10. The
/// variants name the two-way categorization that consume sites need:
/// the BACKEND read failed (a substrate / I/O error), or the lookup
/// surfaced a structural inconsistency that demands fail-CLOSED.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReaderError {
    /// The backing store rejected the read or returned a malformed
    /// record. Per the §13.11 structural lesson + the PR #199
    /// fail-CLOSED posture: a reader that cannot see its grant /
    /// revocation Nodes must surface the failure rather than report
    /// "no revocation found, grant is active."
    #[error("grant reader backend failed: {detail}")]
    BackendFailed {
        /// Operator-readable detail string.
        detail: String,
    },
    /// The backing store contained a record whose canonical
    /// structure is unrecognized (e.g., missing the `scope` property
    /// on a `system:CapabilityGrant` Node, or a revocation Node
    /// referencing a CID that does not resolve). Distinct from
    /// `BackendFailed` so consume sites can distinguish "I/O is
    /// broken" from "data shape on disk is unexpected".
    #[error("grant reader record malformed: {detail}")]
    RecordMalformed {
        /// Operator-readable detail string.
        detail: String,
    },
}

/// `benten-id` sibling `GrantReader` trait.
///
/// **Sibling, not extension.** This trait coexists with
/// `benten_caps::grant_backed::GrantReader`. A concrete type may
/// implement either or both; the two traits do not conflict because
/// they live in different namespaces. Consume sites pick the trait
/// they need based on whether they hold a scope string (the existing
/// `benten-caps` API surface) or a typed `Cid` handle (the new
/// surface that closes §13.11).
///
/// Both methods consult the SAME logical revocation substrate — see
/// the module-level consistency invariant. Implementations are
/// responsible for that invariant; the trait does not enforce it
/// structurally.
///
/// # Why the two-method shape instead of `(Option<&Cid>, &str)`
///
/// A single method taking both keys (`fn has_unrevoked_grant(scope:
/// Option<&str>, cid: Option<&Cid>)`) would force every consume site
/// to construct the OTHER half — defeating the point of the lift,
/// which is to let CID-holding sites consult the reader WITHOUT
/// scope resolution. Separate methods make the CID-keyed path
/// independent of scope-string handling at the call site.
pub trait GrantReader: Send + Sync {
    /// Does the backend contain at least one unrevoked
    /// `system:CapabilityGrant` Node whose `scope` property equals
    /// `scope`?
    ///
    /// Sibling shape to
    /// `benten_caps::grant_backed::GrantReader::has_unrevoked_grant_for_scope`;
    /// preserved at the lift so consume sites that currently hold a
    /// scope string have a typed handle into the `benten-id`-layer
    /// reader without round-tripping through `benten-caps`.
    ///
    /// # Errors
    ///
    /// Returns [`ReaderError::BackendFailed`] when the backing store
    /// rejects the read or returns a malformed record; returns
    /// [`ReaderError::RecordMalformed`] when a record's shape is
    /// structurally unexpected (missing `scope` property on a grant
    /// Node, revocation pointing at a missing CID, etc.). A
    /// reader-side failure is a fail-CLOSED signal — callers MUST
    /// propagate as a denial rather than treat as "no revocation
    /// found, grant is active."
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, ReaderError>;

    /// **CID-keyed companion (G27-C — closes §13.11 structural
    /// gap).** Does the backend contain a grant Node addressed by
    /// `grant_cid` whose revocation substrate reports it unrevoked?
    ///
    /// The CID is the canonical content-addressed handle for a
    /// grant Node — preferable at every consume site that already
    /// holds a CID (the napi `revokeCapability(grantCid, actor)`
    /// seam + the engine's `Engine::revoke_capability_by_grant_cid`
    /// surface + every future lookup site that operates on
    /// content-addressed identifiers rather than scope strings).
    ///
    /// # Consistency contract
    ///
    /// For any logical grant `(scope, cid)`:
    ///
    /// ```ignore
    /// reader.has_unrevoked_grant_for_scope(scope)? ==
    /// reader.has_unrevoked_grant_for_grant_cid(&cid)?
    /// ```
    ///
    /// Must hold under every revocation state. The
    /// `grant_reader_cid_keyed_companion_matches_scope_keyed_for_consistent_inputs`
    /// integration pin exercises this on the canonical in-memory
    /// shape.
    ///
    /// # Errors
    ///
    /// Returns [`ReaderError::BackendFailed`] when the backing
    /// store rejects the read or returns a malformed record;
    /// returns [`ReaderError::RecordMalformed`] when the grant Node
    /// addressed by `grant_cid` is present but structurally
    /// unexpected. A reader-side failure is a fail-CLOSED signal —
    /// per §13.11, the original revoke-by-CID fail-OPEN was
    /// "couldn't find revocation → grant still active"; this method
    /// surfaces "couldn't query → propagate error" instead.
    fn has_unrevoked_grant_for_grant_cid(&self, grant_cid: &Cid) -> Result<bool, ReaderError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use benten_core::{Node, Value};
    use std::collections::{HashMap, HashSet};

    /// In-memory `GrantReader` impl mirroring the consistency
    /// invariant on both key shapes. Used in the inline unit tests
    /// below + by the canonical integration pins at
    /// `crates/benten-id/tests/grant_reader_*.rs`.
    struct InMemoryReader {
        /// scope-string → set of grant CIDs that hold this scope
        scope_to_cids: HashMap<String, Vec<Cid>>,
        /// grant CIDs that are revoked
        revoked_cids: HashSet<Cid>,
        /// scope strings that have any active revocation
        revoked_scopes: HashSet<String>,
    }

    impl GrantReader for InMemoryReader {
        fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, ReaderError> {
            if self.revoked_scopes.contains(scope) {
                return Ok(false);
            }
            Ok(self
                .scope_to_cids
                .get(scope)
                .is_some_and(|cids| !cids.is_empty()))
        }

        fn has_unrevoked_grant_for_grant_cid(&self, grant_cid: &Cid) -> Result<bool, ReaderError> {
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
        let node = Node::new(
            vec!["test:fixture".into()],
            std::iter::once(("seed".to_string(), Value::Bytes(seed.to_vec()))).collect(),
        );
        node.cid().unwrap()
    }

    #[test]
    fn cid_keyed_round_trip_pre_and_post_revocation() {
        let grant_cid = synthetic_cid(b"grant-1");
        let scope = "store:notes:write".to_string();
        let mut reader = InMemoryReader {
            scope_to_cids: HashMap::from([(scope.clone(), vec![grant_cid])]),
            revoked_cids: HashSet::new(),
            revoked_scopes: HashSet::new(),
        };
        assert!(
            reader
                .has_unrevoked_grant_for_grant_cid(&grant_cid)
                .expect("reader ok pre-revoke")
        );
        reader.revoked_cids.insert(grant_cid);
        reader.revoked_scopes.insert(scope.clone());
        assert!(
            !reader
                .has_unrevoked_grant_for_grant_cid(&grant_cid)
                .expect("reader ok post-revoke")
        );
    }

    #[test]
    fn scope_and_cid_keyed_paths_agree_for_consistent_inputs() {
        let cid_1 = synthetic_cid(b"grant-1");
        let scope = "store:notes:write".to_string();
        let mut reader = InMemoryReader {
            scope_to_cids: HashMap::from([(scope.clone(), vec![cid_1])]),
            revoked_cids: HashSet::new(),
            revoked_scopes: HashSet::new(),
        };
        // pre-revoke: both readers must agree on the unrevoked grant.
        assert_eq!(
            reader.has_unrevoked_grant_for_scope(&scope).unwrap(),
            reader.has_unrevoked_grant_for_grant_cid(&cid_1).unwrap()
        );
        // post-revoke: both readers must agree on the revoked grant.
        reader.revoked_cids.insert(cid_1);
        reader.revoked_scopes.insert(scope.clone());
        assert_eq!(
            reader.has_unrevoked_grant_for_scope(&scope).unwrap(),
            reader.has_unrevoked_grant_for_grant_cid(&cid_1).unwrap()
        );
    }

    #[test]
    fn unknown_scope_and_unknown_cid_both_return_false_not_error() {
        let reader = InMemoryReader {
            scope_to_cids: HashMap::new(),
            revoked_cids: HashSet::new(),
            revoked_scopes: HashSet::new(),
        };
        let cid = synthetic_cid(b"never-issued");
        // The substrate is empty; neither key yields a grant; neither
        // method should surface a typed error (consume sites would
        // mis-classify an empty substrate as a substrate failure
        // otherwise).
        assert!(
            !reader
                .has_unrevoked_grant_for_scope("store:x:write")
                .unwrap()
        );
        assert!(!reader.has_unrevoked_grant_for_grant_cid(&cid).unwrap());
    }

    #[test]
    fn reader_error_variants_round_trip_display() {
        // Defends against accidentally collapsing the typed-error
        // shape into a single Display string; consume sites pattern-
        // match on the variant (BackendFailed vs RecordMalformed)
        // for distinct disposition paths.
        let e1 = ReaderError::BackendFailed {
            detail: "redb read failed".into(),
        };
        let e2 = ReaderError::RecordMalformed {
            detail: "grant Node missing `scope` property".into(),
        };
        assert_ne!(e1, e2);
        assert!(format!("{e1}").contains("backend failed"));
        assert!(format!("{e2}").contains("malformed"));
    }
}
