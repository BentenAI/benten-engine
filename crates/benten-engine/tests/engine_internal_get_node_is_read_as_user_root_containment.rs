//! #593 containment proof — engine-internal `get_node` = read-as-the-
//! engine-user-root principal, NOT an auth bypass.
//!
//! Pin source: `.addl/refinement-audit-2026-05/impl-design-COLLAPSE.md`
//! §6 (`#593` re-scope) + §7 row P6. Closes the trust-half doc/
//! containment sub-part of META #593 (the broader META — read-side
//! capability-check enum-narrowing across #534/#509/#336/#525/#499 —
//! remains open; this asserts ONLY the `get_node`/`read_node_as`
//! containment sub-part).
//!
//! Per CLAUDE.md baked-in #18 + the post-Phase-4-Foundation trust-model
//! reframe (`DECISION-RECORD-trust-model-reframe.md` §4, RATIFIED):
//! there is no such thing as an un-principal'd access. The
//! engine-internal un-attributed read is *read-as-the-engine-user-root*
//! (the trust anchor, authorised by construction); `read_node_as` is
//! *read-as-an-attenuated-principal*. The security property is upheld
//! NOT by adding a per-call check to `get_node` (that would regress hot
//! paths and is semantically wrong — the engine-internal principal IS
//! root) but by a **containment proof**: no external / untrusted /
//! plugin / language-binding call path reaches the un-attributed read
//! without going through a principal-gated seam.
//!
//! This test is a would-FAIL guard. If a new external un-attributed
//! caller is introduced — e.g. a napi re-export of the raw backend
//! read, or a napi method that calls `inner.read_node(` /
//! `backend().get_node(` directly instead of the Inv-11+policy-gated
//! `Engine::get_node` or the principal-threaded `Engine::read_node_as`
//! — one of the assertions below fails, surfacing the regression at
//! the exact boundary #593 names.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // crates/benten-engine -> workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    walk(root, &mut out);
    out
}

/// Strip `//` line comments and `/* */` block comments so doc-comments
/// (which legitimately *mention* these symbols) do not produce false
/// positives.
fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
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

/// The set of `benten-engine` source files permitted to call the
/// genuinely un-attributed raw read `self.backend(...).get_node(` /
/// `backend.get_node(`. These are the engine-internal pathways named in
/// CLAUDE.md #18 (IVM / sync / view recompute / audit / builder /
/// module-store / handler-version-chain) PLUS the two public read
/// surfaces (`engine_crud.rs` = `Engine::get_node`, `engine_wait.rs` =
/// `Engine::read_node_as`) which wrap the raw read behind the Inv-11 +
/// `policy.check_read` gate. Adding a NEW source file that calls the
/// raw backend read forces a deliberate edit to this allow-list — the
/// containment decision becomes explicit and reviewable.
const ENGINE_INTERNAL_RAW_READ_FILES: &[&str] = &[
    "builder.rs",
    "engine.rs",
    "engine_crud.rs", // Engine::get_node (Inv-11 + policy gate)
    "engine_diagnostics.rs",
    "engine_modules.rs",
    "engine_views.rs",
    "engine_wait.rs", // Engine::read_node_as (principal-threaded)
    "handler_versions.rs",
    "manifest_signing.rs",
    "primitive_host.rs",
];

/// Assertion 1 — every `benten-engine/src` site that calls the raw,
/// genuinely-un-attributed backend read lives in an enumerated
/// engine-internal file. A new file calling it un-listed = a new
/// un-attributed pathway that bypasses the principal seam → FAIL.
#[test]
fn raw_backend_get_node_callers_are_confined_to_enumerated_engine_internal_files() {
    let src = workspace_root().join("crates/benten-engine/src");
    let mut violations: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    let mut total_hits = 0usize;

    for path in collect_rs_files(&src) {
        scanned += 1;
        let scrubbed = strip_comments(&std::fs::read_to_string(&path).unwrap());
        // Raw read = `.backend.get_node(` or `.backend().get_node(`
        // (the un-gated path), NOT `Engine::get_node` self-calls.
        let hits = scrubbed
            .lines()
            .filter(|l| l.contains("backend.get_node(") || l.contains("backend().get_node("))
            .count();
        if hits == 0 {
            continue;
        }
        total_hits += hits;
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !ENGINE_INTERNAL_RAW_READ_FILES.contains(&fname) {
            violations.push(format!(
                "{} calls the raw un-attributed `backend.get_node(` ({} site(s)) but is NOT in \
                 the engine-internal allow-list. Either it is a legitimate engine-internal \
                 pathway (add it to ENGINE_INTERNAL_RAW_READ_FILES with rationale) or it is a \
                 new un-attributed caller that MUST route through `Engine::read_node_as` \
                 (principal-threaded) per #593 / CLAUDE.md #18.",
                path.display(),
                hits
            ));
        }
    }

    assert!(scanned > 0, "benten-engine/src must contain .rs files");
    assert!(
        total_hits > 0,
        "Expected the raw `backend.get_node(` to exist (engine-internal read pathway) — its \
         absence would make this containment guard trivially pass"
    );
    assert!(
        violations.is_empty(),
        "#593 containment violated — un-attributed read reachable from an un-enumerated \
         source file:\n{}",
        violations.join("\n")
    );
}

/// Assertion 2 — the napi binding (the actual external / language-
/// boundary surface plugin-adjacent callers reach) NEVER calls the raw
/// un-attributed read nor `read_node` directly. Every napi read MUST go
/// through the Inv-11+policy-gated `Engine::get_node` (`inner.get_node`)
/// or the principal-threaded `Engine::read_node_as`. A napi re-export
/// of the raw read is precisely the #593 bypass — this would-FAIL guard
/// catches it at introduction.
#[test]
fn napi_binding_never_reaches_the_un_attributed_read() {
    let napi_src = workspace_root().join("bindings/napi/src");
    if !napi_src.exists() {
        // napi bindings absent in this checkout shape — nothing to
        // contain. (The benten-engine-side Assertion 1 still holds.)
        return;
    }

    let mut violations: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    let mut saw_gated_get_node = false;

    for path in collect_rs_files(&napi_src) {
        scanned += 1;
        let scrubbed = strip_comments(&std::fs::read_to_string(&path).unwrap());
        for line in scrubbed.lines() {
            // Forbidden: raw backend read, or a `read_node(` method
            // call that is NOT the principal-threaded `read_node_as(`.
            if line.contains("backend.get_node(") || line.contains("backend().get_node(") {
                violations.push(format!(
                    "{}: raw backend read — `{}`",
                    path.display(),
                    line.trim()
                ));
            }
            if let Some(idx) = line.find("read_node(") {
                let prefix = &line[..idx];
                let is_method = prefix.chars().last().is_some_and(|c| c == '.' || c == ':');
                let is_read_node_as = line.contains("read_node_as(");
                if is_method && !is_read_node_as {
                    violations.push(format!(
                        "{}: un-attributed `read_node(` — must be `read_node_as(` — `{}`",
                        path.display(),
                        line.trim()
                    ));
                }
            }
            // The gated public seam the napi `get_node` is allowed to
            // use: `inner.get_node(` (= `Engine::get_node`, which
            // applies Inv-11 + policy.check_read).
            if line.contains("inner.get_node(") || line.contains(".read_node_as(") {
                saw_gated_get_node = true;
            }
        }
    }

    assert!(scanned > 0, "bindings/napi/src must contain .rs files");
    assert!(
        violations.is_empty(),
        "#593 containment violated — the napi language boundary reaches the un-attributed \
         read instead of the Inv-11+policy-gated `Engine::get_node` / principal-threaded \
         `Engine::read_node_as`:\n{}",
        violations.join("\n")
    );
    assert!(
        saw_gated_get_node,
        "Expected the napi binding to expose at least one read through the gated \
         `Engine::get_node` / `Engine::read_node_as` seam — its absence would make this \
         containment guard trivially pass"
    );
}
