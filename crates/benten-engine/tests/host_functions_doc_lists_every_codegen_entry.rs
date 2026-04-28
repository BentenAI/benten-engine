//! Phase 2b G11-2b — `docs/HOST-FUNCTIONS.md` MUST list every entry
//! the codegen TOML carries (the brief-named must-pass test).
//!
//! Companion to `host_functions_doc_drift_against_toml.rs` (TOML→MD
//! drift detector, R3-E) and `host_functions_md_drift_against_toml.rs`
//! (MD→TOML reverse, R4-FP B-4). This one enumerates the explicit
//! Phase-2b host-fn surface (`time`, `log`, `kv:read`) + named
//! manifests (`compute-basic`, `compute-with-kv`) and asserts the doc
//! body covers each of them so a future codegen entry that lands
//! without a doc section AND happens to typo the section header
//! cannot slip past the structural-only drift detectors.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn host_functions_doc_lists_every_codegen_entry() {
    let root = workspace_root();
    let doc_path = root.join("docs/HOST-FUNCTIONS.md");
    let body = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/HOST-FUNCTIONS.md MUST exist at Phase-2b close ({}); \
             error: {}. G11-2b-A owns this file per plan §3.",
            doc_path.display(),
            e
        );
    });

    // Phase-2b host-fns per host-functions.toml D1-RESOLVED. The
    // `kv:read` host-fn is keyed under the TOML-quoted form
    // `[host_fn."kv:read"]` to handle the colon character; the doc
    // mirrors that quoted form in its section header so the
    // bidirectional drift detectors agree.
    let host_fns = [
        ("time", "host_fn.time"),
        ("log", "host_fn.log"),
        ("kv:read", "host_fn.\"kv:read\""),
    ];
    for (label, header) in host_fns {
        assert!(
            body.contains(header),
            "docs/HOST-FUNCTIONS.md MUST contain a `## {header}` \
             section (host-fn `{label}`, the section-header pin the \
             bidirectional drift detectors expect). Phase-2b host-fn \
             set per host-functions.toml D1-RESOLVED."
        );
    }

    // Phase-2b named manifests per host-functions.toml D2-RESOLVED.
    let manifests = ["compute-basic", "compute-with-kv"];
    for m in manifests {
        assert!(
            body.contains(m),
            "docs/HOST-FUNCTIONS.md MUST document named manifest \
             `{m}` (Phase-2b set per host-functions.toml \
             D2-RESOLVED)."
        );
    }

    // The deferred `random` host-fn MUST be called out explicitly so
    // operators reading the doc see why the typed
    // `E_SANDBOX_HOST_FN_NOT_FOUND` fires when a module attempts the
    // call.
    assert!(
        body.contains("random") && body.contains("Phase 2c"),
        "docs/HOST-FUNCTIONS.md MUST call out that `random` is \
         deferred to Phase 2c (D1 + sec-pre-r1-06 §2.3 reasoning)."
    );
}
