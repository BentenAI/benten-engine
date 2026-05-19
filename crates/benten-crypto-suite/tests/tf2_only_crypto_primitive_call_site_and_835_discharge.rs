//! TF-2 pin (d) — the integration crate is the ONLY crypto-primitive
//! call site — + the #835 `from_string_unchecked` discharge
//! (verify-AND-execute, not prose).
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core Wave R3-A,
//! agent R3-A2, family TF-2 (#1300 CANARY). Pin sources:
//!   - r2-test-landscape TF-2 covers ("the integration-crate-is-the-only-
//!     crypto-primitive-call-site boundary; #835 `from_string_unchecked`
//!     discharge (verify-and-execute, not prose)") + §2.B row
//!     "#835 `from_string_unchecked` discharge = verify-and-execute".
//!   - plan G-CORE-2 def ("The cipher-suite/hash codepoint dispatch lives
//!     in THIS ONE Benten integration crate (the only crypto-primitive
//!     call site …)" + "The #835 `from_string_unchecked` discharge is
//!     VERIFY-AND-EXECUTE here (actually run the delete-or-`pub(crate)`,
//!     not merely prose)") + §4 CI "asserts the integration crate's
//!     boundary is the only crypto-primitive call site".
//!   - CLAUDE.md #5 ("the integration crate is the only crypto-primitive
//!     call site"; "Never fork, never reimplement, crypto primitives").
//!
//! # RED-PHASE STATUS (pim-12 §3.6e)
//!
//! `benten-crypto-suite` is a STUB at R3-A; the intended G-CORE-2 boundary
//! API + the post-discharge `Did` API do not exist yet → compile-but-fail
//! at the `use` line. All `#[ignore]`-staged
//! `RED-PHASE: un-ignore at G-CORE-2`.
//!
//! # pim-18 SHAPE-not-SUBSTANCE note for pin (d)
//!
//! The "only call site" pin is structurally substantive, NOT a sentinel:
//! it asserts a property over the *whole workspace source tree* (no crate
//! other than `benten-crypto-suite` directly `use`s a crypto-primitive
//! crate for sign/verify/KEM) — a workspace-scan invariant that would
//! genuinely FAIL if `benten-id`/`benten-engine`/etc. kept calling
//! `ed25519-dalek` directly after G-CORE-2 routes them through the seam.
//! It is flagged in the report as "boundary-scan invariant — substantive,
//! exercised against the production workspace tree, not a constructible-
//! type sentinel".

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// RED-PHASE failure point: intended G-CORE-2 boundary-audit surface.
use benten_crypto_suite::boundary::{CryptoPrimitiveCallSiteAudit, ForbiddenDirectUse};

/// The integration crate is the ONLY crypto-primitive call site. After
/// G-CORE-2 routes signing/verification/KEM through the seam, NO other
/// workspace crate may directly depend on / `use` a raw crypto-primitive
/// crate (`ed25519-dalek`, `ml-dsa`, `slh-dsa`, `ml-kem`, `x25519-dalek`,
/// `chacha20poly1305`) for sign/verify/KEM. would-FAIL if any
/// non-`benten-crypto-suite` crate still instantiates a primitive.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_integration_crate_is_only_crypto_primitive_call_site() {
    let audit = CryptoPrimitiveCallSiteAudit::scan_workspace();
    let offenders: Vec<ForbiddenDirectUse> = audit.direct_primitive_use_outside_suite();
    assert!(
        offenders.is_empty(),
        "the signature-agility integration crate MUST be the ONLY \
         crypto-primitive call site (crypto-agility-contract:6). \
         Crates still calling a primitive directly: {offenders:?}"
    );
    // The seam itself is permitted (it IS the call site) — sanity that
    // the audit is not vacuous.
    assert!(
        audit.suite_crate_is_the_call_site(),
        "the audit MUST recognise benten-crypto-suite AS the legitimate \
         call site (a vacuous audit that finds nothing anywhere is the \
         pim-18 SHAPE-trap for this pin)"
    );
}

/// The integration crate NEVER forks/reimplements a primitive — it only
/// wraps vetted upstream crates (concat/hash/codepoint/envelope glue).
/// would-FAIL if the seam ships a hand-rolled Ed25519/ML-DSA/ML-KEM
/// implementation instead of wrapping the RustCrypto crate.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_seam_wraps_upstream_never_reimplements() {
    let audit = CryptoPrimitiveCallSiteAudit::scan_workspace();
    assert!(
        audit.suite_only_wraps_vetted_upstream(),
        "the integration crate MUST be glue over vetted upstream RustCrypto \
         primitives — NEVER a forked/hand-rolled primitive (CLAUDE.md #5)"
    );
}

// RED-PHASE failure point: the #835 discharge changes the public `Did`
// surface. At R3-A `from_string_unchecked` is still `pub` on `benten_id`;
// after the G-CORE-2 verify-and-execute discharge it is DELETED or
// `pub(crate)`. This `use` of the post-discharge marker fails until the
// discharge actually lands (verify-AND-execute, not prose).
use benten_crypto_suite::discharge::Issue835Discharge;

/// #835 discharge is VERIFY-AND-EXECUTE: assert the unsafe
/// `Did::from_string_unchecked` escape hatch is actually gone from the
/// public API (deleted or `pub(crate)`), not merely documented as
/// "should be discharged". would-FAIL if the public escape hatch survives.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-2"]
fn tf2_issue_835_from_string_unchecked_actually_discharged() {
    // The discharge marker records the executed disposition (Deleted or
    // CratePrivate) — its mere existence requires the code change to have
    // landed, since G-CORE-2 emits it only after running the change.
    let disposition = Issue835Discharge::executed_disposition();
    assert!(
        disposition.is_deleted() || disposition.is_crate_private(),
        "#835 MUST be discharged by EXECUTION (delete or pub(crate)), not \
         prose; executed disposition was {disposition:?}"
    );
    // Structural backstop: no public test/production caller of the unsafe
    // constructor remains (the verify half of verify-and-execute).
    assert!(
        Issue835Discharge::no_public_unchecked_constructor_callers(),
        "after the #835 discharge there MUST be zero public callers of an \
         unchecked DID constructor — the verify half of verify-and-execute"
    );
}
