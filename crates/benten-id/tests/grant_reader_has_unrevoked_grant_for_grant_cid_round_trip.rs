//! G27-C — `benten_id::grant_reader::GrantReader::has_unrevoked_grant_for_grant_cid` round-trip.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.16 G27-C row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-C entry
//! + sec-3.5-r1-3 structural lesson (R1 security-auditor).
//!
//! ## The architectural gap (sec-3.5-r1-3 structural lesson)
//!
//! `BackendGrantReader::has_unrevoked_grant_for_scope(scope: &str)` is
//! the only reader API at HEAD — it keys on scope STRINGS. The PR #199
//! production gap (revoke-by-grant-CID at the engine + napi seams) is
//! closed at the WRITE side (the engine resolves the grant's scope
//! before writing the revocation Node). But the READER side is still
//! string-keyed. Any future call site that holds a grant CID (rather
//! than a scope string) has no typed handle into the reader — it has
//! to round-trip through the engine seam.
//!
//! G27-C closes the structural gap by adding a typed
//! `has_unrevoked_grant_for_grant_cid(&Cid)` companion that consults
//! the SAME revocation substrate but keys on the grant Node's CID.
//! Per the plan, this lives in `crates/benten-id/src/grant_reader.rs`
//! (NEW module — `benten-id` cannot depend on `benten-caps` per
//! arch-r1-10, so the new trait is a sibling shape that consume
//! sites adopt at G27-C wave-time).
//!
//! ## Pin shape — round-trip
//!
//! 1. Construct an in-memory implementation of the new
//!    `benten_id::grant_reader::GrantReader` trait carrying a grant
//!    keyed on a known CID (the grant's content-addressed CID).
//! 2. Call `has_unrevoked_grant_for_grant_cid(&grant_cid)`; assert
//!    `Ok(true)`.
//! 3. Apply a revocation record keyed on the SAME CID; call again,
//!    assert `Ok(false)`.
//!
//! ## Would-FAIL-if-no-op'd (pim-2 §3.6b)
//!
//! Revert the lift; the new `benten_id::grant_reader` module doesn't
//! exist; the test fails to compile (the un-ignore target is the
//! compile step).
//!
//! ## RED-PHASE expectation
//!
//! At HEAD: `benten_id::grant_reader` does not exist. The G27-C R5
//! implementer creates the module + lifts the existing scope-keyed
//! method into the new trait + adds the CID-keyed companion. This
//! pin un-ignores at G27-C wave-time per §3.6e.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// RED-PHASE: at HEAD `benten_id::grant_reader` doesn't exist; the
// `use` line below intentionally fails to compile until G27-C lands
// the module. The implementer un-ignores this test + drops the
// `#[cfg]` gate at G27-C wave-time.

#[cfg(any())]
mod red_phase_compile_witness {
    use benten_core::Cid;
    use benten_id::grant_reader::GrantReader;
    use std::collections::HashSet;

    /// Minimal in-RAM `GrantReader` keyed on grant CIDs.
    struct InMemoryReader {
        grants: HashSet<Cid>,
        revocations: HashSet<Cid>,
    }

    impl GrantReader for InMemoryReader {
        fn has_unrevoked_grant_for_scope(
            &self,
            _scope: &str,
        ) -> Result<bool, benten_id::grant_reader::ReaderError> {
            // Existing scope-keyed shape — preserved at the lift; the
            // implementer wires this to actual scope-keyed storage.
            Ok(false)
        }

        fn has_unrevoked_grant_for_grant_cid(
            &self,
            grant_cid: &Cid,
        ) -> Result<bool, benten_id::grant_reader::ReaderError> {
            // CID-keyed companion: consults grant + revocation substrate
            // by content-addressed CID rather than by scope string.
            if self.revocations.contains(grant_cid) {
                return Ok(false);
            }
            Ok(self.grants.contains(grant_cid))
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

    #[test]
    fn round_trip() {
        let grant_cid = synthetic_cid(b"grant-1");
        let mut reader = InMemoryReader {
            grants: HashSet::new(),
            revocations: HashSet::new(),
        };
        reader.grants.insert(grant_cid.clone());

        assert!(
            reader
                .has_unrevoked_grant_for_grant_cid(&grant_cid)
                .expect("reader ok")
        );

        reader.revocations.insert(grant_cid.clone());
        assert!(
            !reader
                .has_unrevoked_grant_for_grant_cid(&grant_cid)
                .expect("reader ok post-revoke")
        );
    }
}

/// RED-PHASE outer test — fires loudly when un-ignored before the
/// `grant_reader` module exists in `benten-id`.
#[test]
#[ignore = "RED-PHASE: G27-C — un-ignore at G27-C wave-time AFTER `benten_id::grant_reader::GrantReader::has_unrevoked_grant_for_grant_cid` lands; drop the cfg(any()) gate on the inner module"]
fn benten_id_grant_reader_has_unrevoked_grant_for_grant_cid_round_trip() {
    panic!(
        "RED-PHASE: G27-C — `benten_id::grant_reader::GrantReader` module + \
         `has_unrevoked_grant_for_grant_cid(&Cid)` companion must land first; \
         then drop the cfg(any()) gate above + invoke `red_phase_compile_witness::round_trip()`."
    );
}
