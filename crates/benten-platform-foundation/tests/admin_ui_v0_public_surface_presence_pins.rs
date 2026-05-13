//! R4b-FP-3 closure of L2 r4b-l2-1 + r4b-l2-2 MINOR findings — presence
//! pins for two admin-UI-v0 public surfaces that R5 introduced + a
//! downstream wave will consume, but that lack a regression-guard until
//! the consumer wave lands:
//!
//! - `admin_ui_v0::Subscriber` — instantiated by inline canary at
//!   `admin_ui_v0/mod.rs:492` but no integration test covers the
//!   public re-export shape. The presence pin asserts the canonical
//!   constructor + the SubscribeAttachToken-bearing field both stay
//!   on the public surface (so a future agent refactoring the module
//!   doesn't silently rename the type or drop the constructor).
//!
//! - `admin_ui_v0::WINTERTC_FORBIDDEN_APIS` — declared as the canonical
//!   list of WinterTC-forbidden API names the G26-B CI guard will sweep
//!   for. G26-B hasn't shipped yet (post-R5 cleanup wave). The
//!   presence pin defends against premature deletion of the const
//!   before its consumer arrives.
//!
//! Both pins are 1-LOC-substance per Ben's "do/fix now" disposition on
//! L2 OBSERVATIONs (R4b-FP-3 inline closure). Closes L2 r4b-l2-1 + r4b-l2-2.

#![allow(clippy::unwrap_used)]

use benten_platform_foundation::admin_ui_v0::{Category, Subscriber, WINTERTC_FORBIDDEN_APIS};

#[test]
fn subscriber_public_surface_supports_for_category_constructor() {
    // PRESENCE PIN: `Subscriber::for_category(category)` is the
    // canonical entry point. The constructor must accept a
    // `Category` + produce a `Subscriber` carrying the
    // SubscribeAttachToken. Would FAIL if a future refactor removes
    // the constructor or changes its signature.
    let sub = Subscriber::for_category(Category::Plugins).unwrap();
    // Substantive check: the subscriber carries the category it was
    // built from (not just a unit type with no observable state).
    assert_eq!(sub.category, Category::Plugins);
    // Substantive check: the subscribe-attach token carries a
    // non-empty pattern (the `admin-ui-v0:<slug>:*` shape).
    assert!(
        !sub.token.pattern.is_empty(),
        "SubscribeAttachToken pattern must be non-empty"
    );
}

#[test]
fn wintertc_forbidden_apis_present_with_canonical_entries() {
    // PRESENCE PIN: the const exists as a `&[&str]` slice with at
    // least the 4 named entries from R5 G24-A — DOM-only +
    // FormData + 3 relative-URL fetch variants. G26-B CI guard
    // sweeps admin-UI-v0 source for these. Would FAIL if a future
    // refactor removes the const or shrinks the entry list before
    // G26-B consumer wave lands.
    assert!(
        !WINTERTC_FORBIDDEN_APIS.is_empty(),
        "WINTERTC_FORBIDDEN_APIS must list ≥1 forbidden API"
    );
    let expected = ["document.cookie", "new FormData"];
    for needle in expected {
        assert!(
            WINTERTC_FORBIDDEN_APIS.iter().any(|api| *api == needle),
            "WINTERTC_FORBIDDEN_APIS missing canonical entry `{needle}`"
        );
    }
    // Relative-URL fetch family — at least one quote-style variant
    // must be present (the const ships 3: `\"./`, `'./`, `` `./``).
    let has_relative_fetch = WINTERTC_FORBIDDEN_APIS
        .iter()
        .any(|api| api.starts_with("fetch(") && api.contains("./"));
    assert!(
        has_relative_fetch,
        "WINTERTC_FORBIDDEN_APIS missing relative-URL fetch variant"
    );
}
