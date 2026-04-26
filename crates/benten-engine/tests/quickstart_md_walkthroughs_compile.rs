//! Phase 2b R4-FP B-4 — `docs/QUICKSTART.md` ≤15 LOC walkthrough drift
//! detector (Rust-side companion to the TS Vitest `quickstart.test.ts`).
//!
//! TDD red-phase. Pin source: dx-optimizer R1 secondary finding
//! (dx-r1-2b-1 — QUICKSTART must carry STREAM/SUBSCRIBE/SANDBOX
//! walkthroughs each ≤15 LOC) + R2 §7 row (`quickstart.test.ts`
//! Vitest gates compilation + execution; this Rust-side test verifies
//! the doc-side LOC budget is honored).
//!
//! The Vitest companion (B-2 owned per dispatch split) actually
//! compiles + runs the example code blocks. This Rust-side test is
//! the structural drift detector: parses `docs/QUICKSTART.md` for
//! `STREAM` / `SUBSCRIBE` / `SANDBOX` walkthrough fenced code blocks
//! and asserts each is at most 15 LOC of TypeScript.
//!
//! Owned by R3-E (CI workflow tests row); test landed by R4-FP B-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

const WALKTHROUGH_KEYWORDS: &[&str] = &["STREAM", "SUBSCRIBE", "SANDBOX"];
const MAX_LOC: usize = 15;

/// Returns the LOC count of each walkthrough's first fenced code
/// block ranged after the keyword section header. Walks the doc
/// linearly: when a `## STREAM` / `## SUBSCRIBE` / `## SANDBOX`
/// header appears, the next ```ts / ```typescript / ```js fenced
/// block is the walkthrough; counts non-blank, non-comment lines
/// inside the fence.
///
/// Returns `(keyword, loc_count)` tuples for each found walkthrough;
/// missing walkthroughs are absent from the returned vec.
fn parse_walkthrough_loc(md_src: &str) -> Vec<(&'static str, usize)> {
    let mut results = Vec::new();
    let lines: Vec<&str> = md_src.lines().collect();

    for keyword in WALKTHROUGH_KEYWORDS {
        // Find header line containing the keyword (case-sensitive on
        // the keyword; flexible on header level).
        let header_idx = lines
            .iter()
            .position(|l| l.starts_with('#') && l.contains(keyword));
        let Some(start) = header_idx else { continue };

        // Find the next fenced block after the header.
        let mut fence_open: Option<usize> = None;
        for (i, line) in lines[start + 1..].iter().enumerate() {
            let actual_idx = start + 1 + i;
            let trimmed = line.trim_start();
            // Stop scanning if we hit the next section header
            // (avoids reading past into another walkthrough).
            if i > 0 && line.starts_with('#') {
                break;
            }
            if trimmed.starts_with("```ts")
                || trimmed.starts_with("```typescript")
                || trimmed.starts_with("```js")
                || trimmed.starts_with("```javascript")
            {
                fence_open = Some(actual_idx);
                break;
            }
        }
        let Some(fence_start) = fence_open else {
            continue;
        };

        // Find closing fence.
        let mut loc: usize = 0;
        for line in lines[fence_start + 1..].iter() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                break;
            }
            // Skip blank + comment-only lines (matches Vitest's
            // "compiled LOC" semantics).
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                continue;
            }
            loc += 1;
        }

        results.push((*keyword, loc));
    }

    results
}

/// `quickstart_md_walkthroughs_compile` — dx-r1-2b-1 (Rust-side
/// structural companion to Vitest body). Verifies every primitive
/// walkthrough is at most 15 LOC.
#[test]
#[ignore = "Phase 2b G11-2b-A pending — QUICKSTART.md walkthroughs unimplemented"]
fn quickstart_md_walkthroughs_under_15_loc() {
    let root = workspace_root();
    let doc_path = root.join("docs/QUICKSTART.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/QUICKSTART.md not found at {} ({}); this is a \
             load-bearing Phase-1 doc per CLAUDE.md key-reading list.",
            doc_path.display(),
            e
        );
    });

    let walkthroughs = parse_walkthrough_loc(&doc_src);

    // All three keywords MUST have a walkthrough (G11-2b-A files-owned
    // per plan §3 G11-2b-A — extends QUICKSTART with STREAM / SUBSCRIBE
    // / SANDBOX walkthroughs).
    let found_keywords: std::collections::BTreeSet<_> =
        walkthroughs.iter().map(|(k, _)| *k).collect();
    let missing: Vec<&&str> = WALKTHROUGH_KEYWORDS
        .iter()
        .filter(|k| !found_keywords.contains(*k))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/QUICKSTART.md missing walkthroughs for: {:?}. G11-2b-A \
         doc sweep MUST add a `## <PRIMITIVE>` section + ```ts fenced \
         code block per primitive (STREAM / SUBSCRIBE / SANDBOX) per \
         dx-r1-2b-1.",
        missing
    );

    let too_long: Vec<_> = walkthroughs
        .iter()
        .filter(|(_, loc)| *loc > MAX_LOC)
        .collect();
    assert!(
        too_long.is_empty(),
        "docs/QUICKSTART.md walkthroughs exceed {}-LOC budget per \
         dx-r1-2b-1: {:?} (LOC counts non-blank non-comment lines). \
         The 15-LOC budget is what makes the QUICKSTART trustworthy as \
         a 10-minute DX path.",
        MAX_LOC,
        too_long
    );
}
