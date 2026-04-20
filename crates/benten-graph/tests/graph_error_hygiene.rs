//! r6-err-3 / r6-err-7 regressions:
//! - `GraphError::BackendNotFound` Display must not leak the absolute
//!   PathBuf through to user-facing error strings.
//! - All GraphError variants must map to a real `E_*` catalog code (no
//!   lowercase `graph_internal` ad-hoc string).

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use benten_graph::{ErrorCode, GraphError};

#[test]
fn backend_not_found_display_does_not_leak_absolute_path() {
    let leaky = PathBuf::from("/Users/alice/secret/benten.redb");
    let err = GraphError::BackendNotFound { path: leaky };
    let rendered = err.to_string();
    assert!(
        !rendered.contains("/Users/alice"),
        "Display must not leak absolute path segments: {rendered}"
    );
    assert!(
        !rendered.contains("/secret"),
        "Display must not leak parent directories: {rendered}"
    );
    // The basename IS allowed — callers often need to know the filename to
    // resolve typos. The Debug impl still preserves the full path for
    // developers with log access.
    assert!(
        rendered.contains("benten.redb"),
        "Display must still surface the basename: {rendered}"
    );
}

#[test]
fn graph_error_variants_all_have_catalog_codes() {
    // Drive each variant through `.code().as_str()` and assert the result
    // is a real catalog code — uppercase `E_` prefix, no stray lowercase
    // markers like the prior `graph_internal` placeholder.
    let cases: Vec<GraphError> = vec![
        GraphError::Redb("stringified".into()),
        GraphError::Decode("decode failed".into()),
        GraphError::BackendNotFound {
            path: PathBuf::from("/tmp/x.redb"),
        },
        GraphError::SystemZoneWrite {
            label: "system:CapabilityGrant".into(),
        },
        GraphError::NestedTransactionNotSupported {},
        GraphError::TxAborted {
            reason: "test".into(),
        },
    ];
    for e in &cases {
        let code = e.code();
        assert!(
            !matches!(code, ErrorCode::Unknown(_)),
            "variant must not route to ErrorCode::Unknown: {e:?}"
        );
        let s = code.as_str();
        assert!(
            s.starts_with("E_")
                && s.chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()),
            "catalog code must be uppercase E_* form, got {s:?} for {e:?}"
        );
    }
}

#[test]
fn redb_and_decode_share_graph_internal_code() {
    // r6-err-3: the two string-payload variants route to
    // `ErrorCode::GraphInternal` / `E_GRAPH_INTERNAL`, not the prior
    // lowercase `graph_internal` placeholder.
    assert_eq!(
        GraphError::Redb("x".into()).code().as_str(),
        "E_GRAPH_INTERNAL"
    );
    assert_eq!(
        GraphError::Decode("y".into()).code().as_str(),
        "E_GRAPH_INTERNAL"
    );
}
