//! F-073 ratchet: no `#[ignore]` arm may be homed at a destination that has
//! already shipped.
//!
//! ## Why this exists
//!
//! HARD RULE rule-12 clause-(b) allows a finding to be deferred only to a
//! NAMED destination that EXISTS and RECEIVES the entry. The failure mode this
//! file guards is narrower and nastier: a destination that existed when the
//! cite was written and then **shipped without firing**, leaving the cite
//! pointing at a closed phase. The arm still reads like a plan; it is now a
//! phantom.
//!
//! This has now happened twice on the same lineage:
//!
//! - `phase-3-backlog §7.3.D` named "the next Phase-3-close orchestrator-direct
//!   fix-pass batch". Phase 3 shipped at `phase-3-close` without it firing.
//! - `phase-4-backlog §4.29` was minted to sweep that cluster "at the
//!   Phase-4-Foundation pre-tag wave". Phase-4-Foundation shipped at
//!   `phase-4-foundation-close` 2026-05-14 without THAT firing either.
//!
//! `§4.168` (F-073 direct residuals) and `§4.169` (the §7.3.D inventory) are
//! the live successor rows. `§4.29` is SUPERSEDED and must never again be
//! written into a `Destination:` clause — re-pointing at it is precisely how
//! this defect launders itself into a third phase.
//!
//! ## What this file does and does NOT claim
//!
//! This is a **record-honesty ratchet**, not a coverage claim. It does not
//! assert that any deferred work is done, that any ignored arm is closeable, or
//! that the inventory is small. It asserts three narrow, falsifiable things:
//! the dead row is not used as a destination, shipped-destination cites carry a
//! live receiving row, and the known-stale inventory does not GROW.
//!
//! The subject under test is source *text* — the `#[ignore]` reason strings
//! themselves. That is the artifact whose honesty is at stake, so scanning it
//! is the direct assertion, not a grep standing in for a behaviour.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories that never host first-party `#[ignore]` arms. `.claude` is on
/// the list because agent worktrees nest full checkouts under
/// `.claude/worktrees/<name>/`; without it a scan run inside a worktree would
/// double-count sibling checkouts.
const SKIP_DIRS: [&str; 4] = ["target", ".git", "node_modules", ".claude"];

/// This file's own source contains every needle below as a literal. Scanning it
/// would make the guard flag itself. Skipping by exact file name is
/// deliberate: if the file is ever renamed, the guard starts matching its own
/// needles and fails LOUDLY rather than silently going vacuous.
const SELF_FILE: &str = "stale_ignore_destination_ratchet.rs";

/// Destinations that have already shipped. An arm citing one of these is only
/// acceptable if it ALSO names the live receiving row.
const SHIPPED_DESTINATIONS: [&str; 3] = ["Phase-4-Foundation pre-tag", "G26-A", "G26-B wave-10"];

/// The live receiving row for the F-073 direct residuals.
const LIVE_ROW: &str = "§4.168";

/// The SUPERSEDED row. Never valid inside a `Destination:` clause.
const DEAD_ROW: &str = "§4.29";

/// The marker carried by the Phase-3 stale-rationale cluster.
const D73_MARKER: &str = "phase-3-backlog §7.3.D";

/// Ceiling for the `§7.3.D` inventory, measured at r9-base `df0c8287`.
///
/// This is a RATCHET, not a census: the count may fall as arms are genuinely
/// closed, but it must never rise. `§4.169` records the same ceiling, and the
/// doc-coupling arm below fails if the two drift apart.
const D73_CEILING: usize = 84;

/// Anti-vacuous floor. If path resolution breaks or `SKIP_DIRS` grows too
/// greedy, the walk returns nothing and every arm below passes on an empty
/// set — the exact shape the frozen-bytes corpus floor exists to stop. 120 is
/// deliberately well under the 161 attributes present at r9-base so ordinary
/// un-ignoring never trips it.
const MIN_ARMS_SCANNED: usize = 120;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(&manifest_dir)
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

/// One `#[ignore ...]` attribute, flattened to a single line.
struct Arm {
    at: String,
    reason: String,
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if !SKIP_DIRS.contains(&name.as_str()) {
                collect_rs(&path, out);
            }
        } else if std::path::Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            && name != SELF_FILE
        {
            out.push(path);
        }
    }
}

/// Flatten every `#[ignore ...]` attribute in the tree.
///
/// Only lines whose trimmed form STARTS with `#[ignore` count: the tree carries
/// ~241 `//!` doc-comment mentions of the token, and counting those would make
/// the inventory meaningless. Verified at r9-base: zero
/// `#[cfg_attr(..., ignore)]` conditional forms exist, so the plain-attribute
/// scan is complete.
fn scan(root: &Path) -> Vec<Arm> {
    let mut files = Vec::new();
    collect_rs(root, &mut files);
    files.sort();

    let mut arms = Vec::new();
    for path in files {
        let Ok(body) = fs::read_to_string(&path) else {
            continue;
        };
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with("#[ignore") {
                continue;
            }
            // A reason string is routinely wrapped across several lines with
            // trailing backslashes. Accumulate until the brackets balance.
            let mut buf = (*line).to_string();
            let mut j = i;
            while buf.matches('[').count() > buf.matches(']').count() && j + 1 < lines.len() {
                j += 1;
                buf.push(' ');
                buf.push_str(lines[j].trim());
            }
            arms.push(Arm {
                at: format!(
                    "{}:{}",
                    path.strip_prefix(root).unwrap_or(&path).display(),
                    i + 1
                ),
                reason: buf.split_whitespace().collect::<Vec<_>>().join(" "),
            });
        }
    }
    arms
}

/// The walk must actually reach the tree.
///
/// FAILS ON THIS ONE-LINE MUTATION: add `"crates"` to `SKIP_DIRS` — the scan
/// then returns a near-empty set and this floor fires before any downstream arm
/// can pass vacuously.
#[test]
fn ignore_scan_is_not_vacuous() {
    let arms = scan(&workspace_root());
    assert!(
        arms.len() >= MIN_ARMS_SCANNED,
        "scan found only {} `#[ignore]` attributes; expected at least {}. \
         The walk is not reaching the tree, so every other arm in this file \
         would pass on an empty set.",
        arms.len(),
        MIN_ARMS_SCANNED
    );
}

/// `§4.29` is SUPERSEDED. It may still be DESCRIBED (two arms correctly
/// explain that it is the row which missed twice), but it must never be the
/// destination an arm is homed at.
///
/// FAILS ON THIS ONE-LINE MUTATION: in
/// `crates/benten-engine/tests/atriums_no_new_primitives.rs`, change the arm's
/// trailing `Destination: docs/future/phase-4-backlog.md §4.168 residual 5` to
/// `... §4.29` — i.e. re-home it on the dead row.
#[test]
fn no_arm_is_homed_at_the_superseded_row() {
    let offenders: BTreeSet<String> = scan(&workspace_root())
        .iter()
        .filter(|a| {
            a.reason
                .split_once("Destination:")
                .is_some_and(|(_, tail)| tail.contains(DEAD_ROW))
        })
        .map(|a| a.at.clone())
        .collect();

    assert!(
        offenders.is_empty(),
        "{} `#[ignore]` arm(s) name the SUPERSEDED row {DEAD_ROW} as their \
         destination: {offenders:?}. {DEAD_ROW} shipped at \
         `phase-4-foundation-close` without firing; its live successors are \
         {LIVE_ROW} (direct F-073 residuals) and §4.169 (the {D73_MARKER} \
         inventory). Re-point at a row that will actually receive the entry.",
        offenders.len()
    );
}

/// Any arm still citing a shipped destination must ALSO name the live row that
/// receives it, so the cite is a real deferral rather than a dangling one.
///
/// FAILS ON THIS ONE-LINE MUTATION: delete the trailing
/// `Destination: docs/future/phase-4-backlog.md §4.168 residual 7 (HARD RULE 12
/// clause-(b)).` sentence from the `#[ignore]` in
/// `crates/benten-engine/tests/exit_criterion_7_aggregates_6_distributed_primitive_pins.rs`.
#[test]
fn shipped_destination_cites_carry_a_live_receiving_row() {
    let offenders: BTreeSet<String> = scan(&workspace_root())
        .iter()
        .filter(|a| {
            SHIPPED_DESTINATIONS.iter().any(|s| a.reason.contains(s))
                && !a.reason.contains(LIVE_ROW)
        })
        .map(|a| a.at.clone())
        .collect();

    assert!(
        offenders.is_empty(),
        "{} `#[ignore]` arm(s) cite a destination that has already shipped \
         ({SHIPPED_DESTINATIONS:?}) without naming the live receiving row \
         {LIVE_ROW}: {offenders:?}",
        offenders.len()
    );
}

/// The `§7.3.D` cluster is inventoried at `§4.169`, not closed. This arm stops
/// the bleed: the inventory may shrink as arms are genuinely closed, but a new
/// arm may not be added to it.
///
/// FAILS ON THIS ONE-LINE MUTATION: add
/// `#[ignore = "phase-3-backlog §7.3.D — new arm"]` to any `#[test]` in the
/// workspace — the count goes to 85 and exceeds the ceiling.
#[test]
fn stale_phase_3_inventory_does_not_grow() {
    let found: Vec<String> = scan(&workspace_root())
        .iter()
        .filter(|a| a.reason.contains(D73_MARKER))
        .map(|a| a.at.clone())
        .collect();

    assert!(
        found.len() <= D73_CEILING,
        "the {D73_MARKER} inventory GREW to {} arms; the ceiling recorded at \
         r9-base is {D73_CEILING}. This cluster is inventoried at \
         `docs/future/phase-4-backlog.md §4.169` and must not take on new \
         members — its own destination already missed two phases. New arm(s) \
         beyond the ceiling: {:?}",
        found.len(),
        &found[D73_CEILING.min(found.len())..]
    );
}

/// The ceiling above is only meaningful if the doc row states the same number.
/// Without this, the constant and the row drift and the inventory becomes a
/// tally mark.
///
/// FAILS ON THIS ONE-LINE MUTATION: change `84` to `83` in the §4.169 heading
/// line of `docs/future/phase-4-backlog.md` without changing `D73_CEILING`.
#[test]
fn inventory_ceiling_matches_the_receiving_row() {
    let backlog = workspace_root().join("docs/future/phase-4-backlog.md");
    let body = fs::read_to_string(&backlog).expect("read docs/future/phase-4-backlog.md");

    let heading = body.lines().find(|l| l.starts_with("### §4.169")).expect(
        "docs/future/phase-4-backlog.md must carry a `### §4.169` row — it is the \
             receiving destination every phase-3-backlog §7.3.D arm depends on",
    );

    assert!(
        heading.contains(&D73_CEILING.to_string()),
        "the §4.169 heading does not state the inventory ceiling {D73_CEILING}: {heading:?}. \
         The row and the ratchet constant must move together or the inventory is a tally mark."
    );
}
