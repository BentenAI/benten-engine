//! D-4F-16 pin — plugin-DID minted via OsRng at install, NOT via
//! HKDF / seed-derivation from user-DID.
//!
//! Un-ignored at R6-FP-BF (closes R6 R1 test-coverage-auditor tc-1 +
//! tc-2 — `plugin_did::mint` source-cite cluster). The production
//! surface `benten_id::plugin_did::mint` shipped at G24-D wave; this
//! pin exercises the OsRng-not-deterministic-seed property.

#[test]
fn plugin_did_mint_uses_os_rng_two_mints_distinct() {
    use benten_id::plugin_did::mint;

    let h1 = mint();
    let h2 = mint();
    assert_ne!(
        h1.did(),
        h2.did(),
        "two plugin_did::mint() calls MUST produce distinct DIDs; \
         identical DIDs would indicate deterministic seed derivation \
         (CLAUDE.md #18 + D-4F-16 violation)"
    );
}
