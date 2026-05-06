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

    // Post-Phase-3-G17-A2 wave-5b: `random` is no longer deferred —
    // CSPRNG via `getrandom` direct + capability-gated entropy budget
    // landed in the codegen-default surface alongside `time`/`log`/
    // `kv:read` (cap-string `host:random:read`). Doc MUST document the
    // available host-fn including the per-call entropy-budget shape
    // (4096-byte default + per-manifest override at
    // `host_fns.random.budget_bytes_per_call` per r1-wsa-8) so
    // operators reading the doc see the live contract rather than the
    // retired deferral.
    assert!(
        body.contains("random") && body.contains("budget_bytes_per_call"),
        "docs/HOST-FUNCTIONS.md MUST document the available `random` \
         host-fn including the per-call entropy-budget contract \
         (`host_fns.random.budget_bytes_per_call` per r1-wsa-8). The \
         pre-G17-A2 'deferred' framing is retired; Compromise #16 \
         CLOSED at Phase-3 G17-A2 wave-5b per docs/SECURITY-POSTURE.md."
    );
}
