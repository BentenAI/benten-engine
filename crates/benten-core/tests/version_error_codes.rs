//! r6-err-11 regression: `VersionError` exposes a `.code()` method and every
//! variant maps to a non-`Unknown` catalog code.

#![allow(clippy::unwrap_used)]

use benten_core::version::VersionError;
use benten_core::{Cid, ErrorCode, Node};

fn fake_cid() -> Cid {
    // Any well-formed Node CID is fine here — the test only needs a stable
    // `Cid` value to populate the error's payload fields.
    Node::empty().cid().unwrap()
}

#[test]
fn version_branched_maps_to_catalog_code() {
    let err = VersionError::Branched {
        seen: fake_cid(),
        attempted: fake_cid(),
    };
    assert_eq!(err.code(), ErrorCode::VersionBranched);
    assert_eq!(err.code().as_str(), "E_VERSION_BRANCHED");
}

#[test]
fn version_unknown_prior_maps_to_dedicated_catalog_code() {
    let err = VersionError::UnknownPrior {
        supplied: fake_cid(),
    };
    // r6-err-11 introduced `E_VERSION_UNKNOWN_PRIOR` so `UnknownPrior` no
    // longer silently coalesces into the generic `E_NOT_FOUND` bucket.
    assert_eq!(err.code(), ErrorCode::VersionUnknownPrior);
    assert_eq!(err.code().as_str(), "E_VERSION_UNKNOWN_PRIOR");
}

#[test]
fn every_version_error_variant_has_non_unknown_code() {
    let variants = [
        VersionError::Branched {
            seen: fake_cid(),
            attempted: fake_cid(),
        },
        VersionError::UnknownPrior {
            supplied: fake_cid(),
        },
    ];
    for v in &variants {
        let code = v.code();
        assert!(
            !matches!(code, ErrorCode::Unknown(_)),
            "VersionError variant must not route to ErrorCode::Unknown; got {code:?}"
        );
        let s = code.as_str();
        assert!(
            s.starts_with("E_"),
            "code string must start with E_; got {s:?}"
        );
    }
}
