//! r6-err-6 / r6-err-9 regressions:
//! - `CapError::Denied` Display format uses plain `{required}` /
//!   `{entity}` (not `{:?}`), so the rendered message does NOT carry
//!   escaped quotes when both payload fields are present.
//! - `CapError::DeniedRead` carries `(required, entity)` context parallel
//!   to `CapError::Denied` so audit pipelines see the same shape on read
//!   denials as on write denials.

use benten_caps::CapError;
use benten_core::ErrorCode;

#[test]
fn denied_display_does_not_contain_escaped_quotes() {
    let err = CapError::Denied {
        required: "store:post:write".into(),
        entity: "alice".into(),
    };
    let rendered = err.to_string();
    assert!(
        !rendered.contains('"'),
        "Denied Display must not escape strings as debug-quoted: {rendered}"
    );
    assert!(
        rendered.contains("store:post:write"),
        "required must appear verbatim: {rendered}"
    );
    assert!(
        rendered.contains("alice"),
        "entity must appear verbatim: {rendered}"
    );
}

#[test]
fn denied_read_carries_structured_context() {
    let err = CapError::DeniedRead {
        required: "store:post:read".into(),
        entity: "bob".into(),
    };
    let rendered = err.to_string();
    assert!(!rendered.contains('"'));
    assert!(
        rendered.contains("store:post:read"),
        "required scope must appear: {rendered}"
    );
    assert!(rendered.contains("bob"), "entity must appear: {rendered}");
    assert_eq!(err.code(), ErrorCode::CapDeniedRead);
}

#[test]
fn denied_read_code_matches_catalog() {
    // r6-err-9: struct-variant promotion keeps the catalog code stable.
    assert_eq!(
        CapError::DeniedRead {
            required: String::new(),
            entity: String::new()
        }
        .code()
        .as_str(),
        "E_CAP_DENIED_READ"
    );
}
