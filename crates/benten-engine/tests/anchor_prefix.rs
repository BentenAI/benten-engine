//! G15-B (wave-5a) — `PrefixMatcher` selector type + `AnchorPrefix` lift.
//!
//! Pin sources:
//! - `r2-test-landscape` §2.3 G15-B rows
//!   `anchor_prefix_matches_prefix_not_equality` +
//!   `anchor_prefix_no_silent_label_equality_coerce`.
//! - plan §3 G15-B row line "PrefixMatcher selector type —
//!   `anchor_prefix=\"crud:\"` matches both `\"crud:post\"` and
//!   `\"crud:user\"`".
//!
//! Pre-G15-B, `Engine::register_user_view` silently coerced
//! `UserViewInputPattern::AnchorPrefix(p)` into a `ContentListingView::new(p)`
//! registration: filter by `label == p` (label-equality) NOT by
//! `label.starts_with(p)` (prefix matching).
//!
//! Post-G15-B, `AnchorPrefix` routes through the new `PrefixMatchingView`
//! (engine_views.rs) — `anchor_prefix="crud:"` materialises rows for every
//! Node whose label begins with `"crud:"` (e.g. `"crud:post"`, `"crud:user"`,
//! `"crud:comment"`) but NOT for nodes labelled `"zone:public"` or
//! `"governance:rule"`.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::{Engine, UserViewInputPattern, UserViewSpec};

/// Build an engine with IVM enabled, in a fresh tempdir-backed redb.
fn open_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// Build a Node carrying `label` as its sole label and a `text` property
/// (so the encoded body is non-empty / round-trips through the backend).
fn make_node(label: &str, text: &str) -> Node {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("text".into(), Value::text(text));
    Node::new(vec![label.to_string()], props)
}

#[test]
fn anchor_prefix_matches_prefix_not_equality() {
    // plan §3 G15-B pin. Register first, then drive five writes; three
    // labels share the `"crud:"` prefix, two don't. The view materialises
    // the three matching rows.
    let (_dir, engine) = open_engine();

    let spec = UserViewSpec::builder()
        .id("all_crud")
        .input_pattern(UserViewInputPattern::AnchorPrefix("crud:".to_string()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    engine
        .transaction(|tx| {
            tx.put_node(&make_node("crud:post", "p1"))?;
            tx.put_node(&make_node("crud:user", "u1"))?;
            tx.put_node(&make_node("crud:comment", "c1"))?;
            tx.put_node(&make_node("zone:public", "z1"))?;
            tx.put_node(&make_node("governance:rule", "r1"))?;
            Ok(())
        })
        .unwrap();

    let outcome = engine
        .read_view("all_crud")
        .expect("read_view succeeds for the registered view id");
    let rows = outcome.as_list().unwrap_or_default();

    assert_eq!(
        rows.len(),
        3,
        "anchor_prefix \"crud:\" must match the 3 crud:* writes via prefix \
         (got {} rows). Labels seen: {:?}",
        rows.len(),
        rows.iter()
            .flat_map(|n| n.labels.clone())
            .collect::<Vec<_>>(),
    );

    // Every materialised row must carry a label beginning with `"crud:"` —
    // the negative half of prefix-vs-equality.
    for n in &rows {
        assert!(
            n.labels.iter().any(|l| l.starts_with("crud:")),
            "every materialised row must carry a `crud:`-prefixed label; \
             got labels {:?}",
            n.labels
        );
    }
}

#[test]
fn anchor_prefix_no_silent_label_equality_coerce() {
    // plan §3 G15-B pin — explicitly rejects the silent coerce back to
    // label-equality. Three writes, all under labels that strictly start
    // with `"crud:"` but NONE that equal `"crud:"`. If a future refactor
    // collapsed PrefixMatcher::Prefix to PrefixMatcher::Equal, the
    // assertion would fire (zero matches under equality).
    let (_dir, engine) = open_engine();

    let spec = UserViewSpec::builder()
        .id("any_crud")
        .input_pattern(UserViewInputPattern::AnchorPrefix("crud:".to_string()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    engine
        .transaction(|tx| {
            tx.put_node(&make_node("crud:post", "p1"))?;
            tx.put_node(&make_node("crud:post", "p2"))?;
            tx.put_node(&make_node("crud:user", "u1"))?;
            Ok(())
        })
        .unwrap();

    let outcome = engine.read_view("any_crud").unwrap();
    let rows = outcome.as_list().unwrap_or_default();
    assert_eq!(
        rows.len(),
        3,
        "anchor_prefix \"crud:\" must match 3 nodes via prefix, not 0 via equality"
    );
}
