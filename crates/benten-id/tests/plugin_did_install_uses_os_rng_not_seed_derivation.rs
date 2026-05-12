//! D-4F-16 pin — plugin-DID minted via OsRng at install, NOT via
//! HKDF / seed-derivation from user-DID.
//!
//! Per CLAUDE.md #18 + D-4F-16: "Plugin-DID minted at install — a UCAN
//! audience handle (NOT an attested sub-identity); just an identifier
//! so the user can issue UCAN caps with `audience=plugin-DID`".
//!
//! Per R2 §5 Gap fix #5 paired discipline: this OsRng pin is
//! statistically fragile alone; PAIR with the grep-assert against
//! `hkdf` / `derive_from` patterns in `crates/benten-id/src/plugin_
//! did.rs` (see companion test file
//! `plugin_did_install_no_hkdf_from_user_did_grep_assert.rs`).

#[test]
#[ignore = "RED-PHASE: G24-D wave wires plugin_did::mint(OsRng); un-ignore at G24-D landing"]
fn plugin_did_mint_uses_os_rng_two_mints_distinct() {
    // Future surface:
    //   benten_id::plugin_did::mint() -> Keypair
    // calls Keypair::generate(&mut OsRng) per crypto-major-2 baseline.
    //
    // Statistical-only assertion: two separate mints produce distinct
    // public keys with overwhelming probability (OsRng entropy).
    // FAILS-IF-NO-OP if mint were deterministic from user-DID (would
    // produce identical keys).
    panic!("RED-PHASE: G24-D wave must wire plugin_did::mint via OsRng");
}
