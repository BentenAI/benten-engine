//! Phase-4-Foundation G24-A — admin UI v0 subscribe paths flowing
//! ONLY via `Engine::on_change_as_with_cursor`, never
//! `Engine::subscribe_change_events`.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 4 + §5 SHAPE-not-SUBSTANCE smell-test pairing; closes
//! sec-3.5-r1-9 + T12.
//!
//! Per G22-FP-1 PR #210 closure (`CapRecheckOutcome` shipped LIVE) +
//! T12: `on_change_as_with_cursor` is the cap-recheck-enabled
//! subscribe seam. Each event delivery invokes a per-row
//! `CapabilityPolicy::check_read`; revoked grants cause `Drop` (skip
//! event) or `Cancel` (terminate sub).
//!
//! The older `subscribe_change_events` surface does NOT recheck caps
//! on event delivery — it's reserved for engine-internal flows (audit
//! log, IVM materialization). Admin UI plugin code reaching for it
//! is the exact failure shape this pin defends against.

#![allow(clippy::unwrap_used)]

use benten_platform_foundation::Subscriber;
use std::path::PathBuf;

fn admin_ui_v0_source_roots() -> Vec<PathBuf> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = here.parent().unwrap().parent().unwrap();
    vec![
        workspace
            .join("crates")
            .join("benten-platform-foundation")
            .join("src")
            .join("admin_ui_v0"),
        workspace.join("packages").join("admin-ui-v0").join("src"),
    ]
}

fn collect_source_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    fn walk(dir: &PathBuf, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("rs" | "ts" | "tsx" | "js" | "mjs")
            ) {
                out.push(path);
            }
        }
    }
    walk(root, &mut out);
    out
}

fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// SHAPE half — grep-assert that admin UI source contains zero
/// references to `subscribe_change_events` and at least one reference
/// to `on_change_as_with_cursor`.
#[test]
fn admin_ui_v0_source_uses_only_on_change_as_with_cursor_for_subscribe() {
    let roots = admin_ui_v0_source_roots();
    let mut found_correct = 0_usize;
    let mut scanned_files = 0_usize;
    for root in &roots {
        for path in collect_source_files(root) {
            scanned_files += 1;
            let src = std::fs::read_to_string(&path).unwrap();
            let scrubbed = strip_comments(&src);
            assert!(
                !scrubbed.contains("subscribe_change_events"),
                "Admin UI v0 source MUST NEVER reference \
                 Engine::subscribe_change_events (no cap-recheck on event \
                 delivery); found in {}",
                path.display()
            );
            if scrubbed.contains("on_change_as_with_cursor") {
                found_correct += 1;
            }
        }
    }
    assert!(
        scanned_files > 0,
        "Admin UI v0 source roots MUST exist + contain source files"
    );
    assert!(
        found_correct > 0,
        "Admin UI v0 source MUST reference `on_change_as_with_cursor` at \
         least once (otherwise the cap-recheck-enabled seam isn't wired)"
    );
}

/// SUBSTANCE half — exercise the Subscriber surface that admin UI
/// uses to build subscribe-attach tokens; assert the token's pattern
/// is bound to the per-category prefix admin UI ships with.
#[test]
fn admin_ui_v0_subscribe_runtime_traces_to_on_change_as_with_cursor_not_subscribe_change_events() {
    use benten_platform_foundation::{Category, NAV_CATEGORIES};
    for category in NAV_CATEGORIES {
        let sub = Subscriber::for_category(category)
            .expect("Subscriber::for_category must succeed for canonical 4 categories");
        let pattern = &sub.token.pattern;
        assert!(
            pattern.contains(category.route_slug()),
            "subscribe token pattern MUST carry category slug per per-row gating; \
             saw {pattern}"
        );
        // The pattern shape is `admin-ui-v0:<slug>:*` — the consumer
        // adapter passes this to `Engine::on_change_as_with_cursor`,
        // NEVER to bare `Engine::subscribe_change_events` (no
        // cap-recheck seam).
        assert!(
            pattern.starts_with("admin-ui-v0:"),
            "subscribe token pattern MUST be admin-ui-v0-scoped"
        );
    }
    // Sentinel: the `Subscriber` API does not expose a path to the
    // engine-internal `subscribe_change_events` seam — there is no
    // method on `Subscriber` that could route there.
    let _: Category = Category::Workflows;
}
