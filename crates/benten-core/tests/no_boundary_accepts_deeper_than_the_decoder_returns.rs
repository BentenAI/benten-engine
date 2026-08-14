//! Every input boundary must cap nesting depth at or below the depth the
//! canonical decoder can actually return.
//!
//! ## Why this exists
//!
//! `bindings/napi/src/node.rs` carried `JSON_MAX_DEPTH = 128` as a literal
//! while `benten_core::MAX_VALUE_DECODE_DEPTH` was 64. The JSON path is the
//! LIVE production write path (`createNode` / `updateNode`), so a property bag
//! nested 65..=128 deep was **accepted, hashed and persisted — and then could
//! not be decoded on read.** Silent write-side data loss: the boundary
//! admitted a value it could never hand back.
//!
//! The fix derives the cap from the decoder's own bound. This pin stops it
//! being re-literalised, and stops a NEW boundary being added with its own
//! larger number.
//!
//! ## What this pin does and does not prove
//!
//! It is a SOURCE-PROPERTY scan, and the property genuinely is a source
//! property: "this cap is derived, not literal." It is not a behavioural
//! assertion, and it is recorded as such rather than dressed up.
//!
//! A behavioural pin cannot live here: `bindings/napi/src/node.rs` is
//! `napi-export`-gated, so it is not compiled in rlib mode, and under
//! `napi-export` the test binary needs napi cdylib symbols that only exist
//! inside a Node host process (`napi-sys` declares `napi_*` as plain
//! `extern "C"` on every non-msvc, non-wasm target). Closing that needs a
//! Node-hosted vitest assertion on the thrown `E_INPUT_LIMIT`.
//!
//! ## Would-FAIL-on-revert
//!
//! Rewrite `bindings/napi/src/node.rs`'s `JSON_MAX_DEPTH` back to a numeric
//! literal — `const JSON_MAX_DEPTH: usize = 128;` — and this test fails,
//! naming the file and the literal. Adding a NEW `*_MAX_DEPTH` const anywhere
//! under `bindings/napi/src/` with a literal value fails it too: the check is
//! over the CLASS of declaration, not one remembered identifier.

use std::path::PathBuf;

/// Repo root, from this crate's manifest dir.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/<crate> is two levels below the repo root")
        .to_path_buf()
}

/// Strip `//` and `///` line comments so a doc-comment mentioning a literal
/// depth cannot trip (or satisfy) the scan.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn no_napi_depth_cap_is_a_numeric_literal() {
    let root = repo_root();
    let napi_src = root.join("bindings/napi/src");
    assert!(
        napi_src.is_dir(),
        "napi source dir not found at {} — this scan is reading nothing, which \
         would make it vacuously green. Fix the path, do not delete the test.",
        napi_src.display()
    );

    let mut scanned_files = 0usize;
    let mut depth_consts = 0usize;
    let mut offenders: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(&napi_src).expect("read napi src dir");
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).expect("read napi source file");
        let src = strip_line_comments(&raw);
        scanned_files += 1;

        for line in src.lines() {
            let t = line.trim();
            // Any const whose name ends in `_MAX_DEPTH` — the CLASS, so a
            // newly-added boundary with a different identifier is caught too.
            if !(t.starts_with("const ") || t.starts_with("pub const "))
                || !t.contains("_MAX_DEPTH")
            {
                continue;
            }
            depth_consts += 1;
            let rhs = match t.split_once('=') {
                Some((_, r)) => r.trim().trim_end_matches(';').trim(),
                None => continue,
            };
            // A bare numeric literal (allowing `_` separators and a suffix)
            // is the defect. A derived expression naming the decoder bound
            // is the fix.
            let is_literal = rhs.chars().next().is_some_and(|c| c.is_ascii_digit());
            if is_literal {
                offenders.push(format!(
                    "{}: `{}` — depth cap is a numeric literal, not derived \
                     from `benten_core::MAX_VALUE_DECODE_DEPTH`",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                    t
                ));
            }
        }
    }

    // Vacuity floor: if the scan finds no depth consts at all it is reading
    // the wrong thing, and a green result would be meaningless.
    assert!(
        scanned_files > 0,
        "scanned zero .rs files under {} — vacuous",
        napi_src.display()
    );
    assert!(
        depth_consts > 0,
        "scanned {scanned_files} files under bindings/napi/src and found ZERO \
         `*_MAX_DEPTH` consts. Either they were renamed (update this scan) or \
         the scan is broken — a green verdict here would prove nothing."
    );

    assert!(
        offenders.is_empty(),
        "a napi input boundary caps nesting depth with a hard-coded number \
         instead of deriving it from `benten_core::MAX_VALUE_DECODE_DEPTH` \
         ({}).\n\nA cap ABOVE the decoder bound accepts values it can never \
         return: they are hashed and persisted, then fail to decode on read — \
         silent write-side data loss. A cap BELOW it is a defense that looks \
         present and is not.\n\nOffenders:\n  {}",
        benten_core::MAX_VALUE_DECODE_DEPTH,
        offenders.join("\n  ")
    );
}

#[test]
fn decoder_depth_bound_is_the_single_source_of_truth() {
    // Pins the value the scan above defers to. If this moves, every derived
    // boundary moves with it by construction — which is the point.
    assert_eq!(
        benten_core::MAX_VALUE_DECODE_DEPTH,
        64,
        "MAX_VALUE_DECODE_DEPTH changed. That is legal, but it is a decode \
         bound on attacker-supplied bytes: re-derive the DoS reasoning and \
         update `docs/SECURITY-POSTURE.md`'s B8 closure before repinning."
    );
}
