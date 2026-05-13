//! Phase-4-Foundation G24-A — admin UI v0 shell renders the
//! 4-category navigation IA (ratification #4 + ux-r1-8).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 1 (LOAD-BEARING substantive); closes ratification #4 + ux-r1-8 +
//! plugin-arch-r1-12 + Family F1 gap-#6.
//!
//! Per ratification #4 (D-4F-4 admin UI v0 IA): the navigation surface
//! is **4 categories** — Plugins / Workflows / Content Types / Views.
//! The canonical Rust-side source of truth is
//! [`benten_platform_foundation::NAV_CATEGORIES`].
//!
//! ## Substantive shape
//!
//! Three production-runtime checks (pim-18 §3.6f):
//!
//! 1. **The 4 categories carry the canonical labels in the canonical
//!    order** — Rust-side constant matches the locked IA.
//! 2. **The admin UI v0 subgraph composes one route per category** —
//!    each category's subgraph carries a category-tag on every
//!    OperationNode so a walker can attribute primitives back to
//!    their route.
//! 3. **Each route's READ primitive carries a category-scoped cap-scope
//!    annotation** — the admin UI plugin's manifest grants
//!    `read:admin-ui-v0:<slug>` caps; this assertion locks the seam.

#![allow(clippy::unwrap_used)]

use benten_core::{PrimitiveKind, Value};
use benten_platform_foundation::{
    Category, NAV_CATEGORIES, build_admin_ui_v0_subgraph, build_category_route_subgraph,
};

#[test]
fn admin_ui_v0_shell_renders_4_category_navigation_substantively() {
    // (1) The 4 categories carry the canonical labels in canonical
    // order per ratification #4.
    let labels: Vec<&str> = NAV_CATEGORIES.iter().map(|c| c.label()).collect();
    assert_eq!(
        labels,
        vec!["Plugins", "Workflows", "Content Types", "Views"],
        "Admin UI v0 navigation MUST expose 4 categories in canonical order \
         per D-4F-4 ratification #4"
    );

    // (2) The admin UI v0 subgraph composes one route per category.
    let sg = build_admin_ui_v0_subgraph();
    let mut categories_seen: Vec<String> = sg
        .nodes()
        .iter()
        .filter_map(|op| op.property("admin_ui_v0_category"))
        .filter_map(|v| match v {
            Value::Text(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    categories_seen.sort();
    categories_seen.dedup();
    let mut expected: Vec<String> = NAV_CATEGORIES.iter().map(|c| c.label().into()).collect();
    expected.sort();
    assert_eq!(
        categories_seen, expected,
        "Admin UI v0 subgraph MUST carry one route per category"
    );

    // (3) Each route's READ primitive carries a category-scoped cap-scope.
    for category in NAV_CATEGORIES {
        let route = build_category_route_subgraph(category);
        let read = route
            .nodes()
            .iter()
            .find(|op| op.kind == PrimitiveKind::Read)
            .expect("category route MUST have a READ primitive");
        let scope = read
            .property("cap_scope")
            .expect("READ primitive MUST carry cap_scope annotation");
        let Value::Text(scope) = scope else {
            panic!("cap_scope MUST be Text")
        };
        assert!(
            scope.contains(category.route_slug()),
            "cap_scope MUST be category-slug-scoped; saw `{scope}` for category {:?}",
            category,
        );
        assert!(
            scope.starts_with("read:"),
            "cap_scope MUST start with `read:` (read action discrimination); saw `{scope}`"
        );
    }
}

#[test]
fn admin_ui_v0_per_category_route_slug_is_kebab_case() {
    // route_slug is the URL surface admin UI v0 routes through; it
    // MUST be kebab-case so the multi-word "Content Types" label
    // survives routing as `content-types`.
    assert_eq!(Category::Plugins.route_slug(), "plugins");
    assert_eq!(Category::Workflows.route_slug(), "workflows");
    assert_eq!(Category::ContentTypes.route_slug(), "content-types");
    assert_eq!(Category::Views.route_slug(), "views");
}
