//! R4 triage (m11) — pin the exact string values of `LABEL_CURRENT` and
//! `LABEL_NEXT_VERSION`. These constants form the wire-compatible edge-label
//! contract; a rename is a breaking change.

use benten_core::{LABEL_CURRENT, LABEL_NEXT_VERSION};

#[test]
fn label_current_is_exact_string() {
    assert_eq!(LABEL_CURRENT, "CURRENT");
}

#[test]
fn label_next_version_is_exact_string() {
    assert_eq!(LABEL_NEXT_VERSION, "NEXT_VERSION");
}
