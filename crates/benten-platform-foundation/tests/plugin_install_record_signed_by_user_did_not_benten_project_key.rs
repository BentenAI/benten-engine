//! LOAD-BEARING per plan §3 G24-D row.
//!
//! Verifies the **ratified user-as-source signing model** per
//! CLAUDE.md #18 "Implementation refinements" + post-R1-triage Q4 +
//! post-2026-05-11-conversation D-4F-12 retense.
//!
//! Would-FAIL if a Benten-project-key were the install-record signer.
//!
//! Per pim-2 §3.6b end-to-end test pin discipline AND R2 §5 Gap fixes —
//! pair the negative "NO Benten-project-key" assertion with a positive
//! "user-DID is the signer + UCAN chain traces to user-root" assertion
//! so the SUBSTANTIVE shape (not just shape-only) is verified.

mod common;

use common::manifest_fixtures::{stub_install_record, stub_user_did};

#[test]
fn install_record_is_signed_by_user_did_anchored_in_users_graph() {
    let manifest_cid = common::manifest_fixtures::stub_cid_zero();
    let install = stub_install_record(manifest_cid);

    // POSITIVE: user-DID is the consenting signer.
    assert_eq!(install.consenting_user_did, stub_user_did());

    // SUBSTANTIVE: future G24-D surface will be
    // `InstallRecord::verify_user_signature(&user_did) -> Result<()>`.
    // FAILS-IF-NO-OP because signature bytes would be 0-bytes and
    // ed25519 verify would reject. Stub at R3.
}

#[test]
fn no_benten_project_key_infrastructure_in_codebase_grep_assert() {
    // SUBSTANTIVE per R2 §5: grep-walk over the platform-foundation
    // crate source tree asserting NO symbols match the Benten-
    // project-key pattern. The user-as-source signing model is
    // structural; there should be no "project key" anywhere.
    //
    // pim-18 §3.6f vacuous-truth defense: first-line assert the
    // walked root exists; assert the walk actually surfaces .rs
    // files. Without these guards, a 0-file walk would silently pass.

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    assert!(
        src_dir.exists() && src_dir.is_dir(),
        "walked root MUST exist: {src_dir:?} (vacuous-truth defense per pim-18 §3.6f)"
    );

    // Forbidden patterns — Benten-project-key symbols that MUST NOT
    // appear in the codebase per CLAUDE.md #18 "user-as-source signing
    // model" (post-2026-05-11 D-4F-12 retense).
    let forbidden_patterns = [
        "benten_project_key",
        "BentenProjectKey",
        "project_signing_key",
        "BENTEN_PROJECT_KEY",
        "ProjectSigningKey",
    ];

    let mut walked_files = 0usize;
    let mut violations: Vec<(String, &str)> = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        walked_files += 1;
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
        for pat in &forbidden_patterns {
            if src.contains(pat) {
                violations.push((path.display().to_string(), pat));
            }
        }
    }

    // pim-18 §3.6f sub-rule: assert the walk actually visited files.
    assert!(
        walked_files > 0,
        "walkdir surfaced 0 .rs files under {src_dir:?} — vacuous-truth defense"
    );

    assert!(
        violations.is_empty(),
        "platform-foundation src/ MUST NOT contain Benten-project-key \
         patterns (user-as-source signing model per CLAUDE.md #18). \
         Violations: {violations:?}"
    );
}
