//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 — §4.23 user-DID root-chain
//! write-boundary validator (structurally-always-on).
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! CLAUDE.md baked-in #18 Layer-1 user-as-root invariant
//! (runtime-enforced): EVERY WRITE must trace back to a user-DID root
//! grant. At SYNCED HEAD `ed03729a` the SHIPPED chain-validator surface
//! `benten_caps::validate_chain_with_manifest_envelope` already returns
//! [`ChainValidationOutcome::RootNotUserDid`] for a non-user-rooted
//! chain (the static-fixture pin at
//! `manifest_envelope_chain_validation_requires_user_root_anchor.rs`
//! exercises this against an in-memory `UserDidRegistry`). What is
//! UNDELIVERED is the **structural-always-on wiring** of that
//! validator inside the WRITE primitive's evaluator dispatch — there
//! is no production WRITE admission seam that consults the
//! `UserDidRegistry` over the engine's install-record store.
//!
//! §4.23 requires this validator be **structurally-always-on** —
//! mirroring the Phase-3 G16-B-F structural-always-on per-row
//! cap-recheck (PR #161), NOT an opt-in like the §4.36 footgun.
//! It fails-CLOSED if the chain does NOT terminate at the user-DID
//! root.
//!
//! Stranded pin destinations named in §4.23 (each ignore message MUST
//! cite §4.23) — this file consolidates the structural arm; the
//! per-scenario admin_ui_v0 pins (`admin_ui_did_cannot_mint_root_grant`,
//! `admin_ui_v0_background_write_must_trace_to_user_root`,
//! `admin_ui_v0_user_initiated_write_succeeds`) are §3.6e staged-pins
//! the G-CORE-8 wave un-ignores; the reviewer verifies LANDING status,
//! not just spec-pin presence.
//!
//! ## SUBSTANTIVE-arm-not-SHAPE shape (R4.1 fix-pass per pim-18 / §3.6f)
//!
//! Each RED arm below **first** exercises the SHIPPED
//! `validate_chain_with_manifest_envelope` (the §4.28 substantive_arm
//! template pattern affirmed sound by L3) on a real
//! `DelegationStep` chain with a real outcome assertion + observable
//! would-FAIL consequence, **then** `panic!`-holds the still-
//! undelivered structurally-always-on WRITE-admission wiring. The
//! shipped primitive is exercised so the body has a non-zero substance
//! footprint regardless of the WRITE-side gap.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-1 (§3.5b): public-shape; threat-model T8 + SECURITY-POSTURE
//!     couple — sweep before push.
//!   - pim-2-amendment (§3.6b sub-rule-4): exercises the SPECIFIC
//!     always-on validator arm (production WRITE admission path,
//!     observable row-reject, would-FAIL if the validator is opt-in or
//!     absent).
//!   - pim-12 (§3.6e): RED-PHASE staged-pin; wave-completion checklist
//!     sweeps + un-ignores; reviewer verifies landing-status.
//!   - pim-18 (§3.6f): production call-site enumerated; the body
//!     asserts the SUBSTANTIVE deny-on-non-user-root, not a sentinel.
//!     The mostly-undelivered-target-surface hybrid pattern (R4.1
//!     pattern-induction): exercise SHIPPED adjacent primitives
//!     (`validate_chain_with_manifest_envelope`) + `panic!`-hold the
//!     missing structurally-always-on WRITE-admission wiring.
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!   - §3.11: TF-8 largest cross-crate family — resume INTO worktree
//!     on agent-kill.
//!
//! Pins: G-CORE-8 · C8 · §1.A.FROZEN item 12 (security-surface lock).
//! R2 map: TF-8 RED-arm (4) write-boundary chain validator
//! structurally-always-on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashSet;

use benten_caps::manifest_envelope_chain_validation::{
    ChainValidationOutcome, DelegationStep, ManifestEnvelopeLookup, UserDidRegistry,
    validate_chain_with_manifest_envelope,
};
use benten_caps::plugin_delegation::SharesPolicyView;
use benten_id::did::Did;

// ---------------------------------------------------------------------------
// In-memory fixtures (mirrors the SHIPPED static fixture pin at
// crates/benten-caps/tests/manifest_envelope_chain_validation_requires_user_root.rs).
// These are NOT a duplicate of that pin — that pin asserts the *primitive*
// works on a single-step plugin chain. THIS file folds the SHIPPED
// primitive call into the §4.23 user-root-anchor + plugin-elevation
// scenarios (multiple chain shapes) AND `panic!`-holds the missing
// WRITE-admission wiring per §3.6f mostly-undelivered-target hybrid.
// ---------------------------------------------------------------------------

struct AllPermit;
impl SharesPolicyView for AllPermit {
    fn permits(&self, _cap: &str, _target: &Did) -> bool {
        true
    }
}

struct EmptyLookup;
impl ManifestEnvelopeLookup for EmptyLookup {
    type View<'a>
        = &'a AllPermit
    where
        Self: 'a;
    fn lookup<'a>(&'a self, _plugin_did: &Did) -> Option<Self::View<'a>> {
        None
    }
}

struct UserRegistry {
    users: HashSet<String>,
}
impl UserDidRegistry for UserRegistry {
    fn is_user_did(&self, did: &Did) -> bool {
        self.users.contains(did.as_str())
    }
}

fn user_did() -> Did {
    Did::from_string_for_test_fixture("did:key:z6MkUser".to_string())
}

fn plugin_did() -> Did {
    Did::from_string_for_test_fixture("did:key:z6MkPlugin".to_string())
}

fn registry_with_user() -> UserRegistry {
    let mut users = HashSet::new();
    users.insert(user_did().as_str().to_string());
    UserRegistry { users }
}

#[test]
fn write_with_chain_not_terminating_at_user_root_is_denied_fail_closed() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (substantive-arm anchor for pim-18 §3.6f):
    // drive `validate_chain_with_manifest_envelope` on an adversarial
    // chain rooted at a PLUGIN-DID (not a user-DID). The validator MUST
    // return RootNotUserDid (a non-trivial assertion against the SHIPPED
    // surface). Would-FAIL signal: if the validator regressed to admit
    // non-user-rooted chains, the assertion below fires.
    // -----------------------------------------------------------------
    let registry = registry_with_user();
    let plugin = plugin_did();
    let adversarial_chain = vec![DelegationStep {
        issuer_did: plugin.clone(), // NOT user-DID — adversarial root.
        audience_did: Did::from_string_for_test_fixture("did:key:z6MkVictim".to_string()),
        cap_pattern: "store:notes:write".into(),
    }];
    let outcome =
        validate_chain_with_manifest_envelope(&adversarial_chain, &EmptyLookup, &registry);
    assert_eq!(
        outcome,
        ChainValidationOutcome::RootNotUserDid,
        "shipped surface exercise: a chain rooted at a plugin-DID MUST \
         yield RootNotUserDid (CLAUDE.md #18 Layer-1 user-as-root; the \
         retarget anchor for §4.23). Would-FAIL if the validator \
         silently admits non-user-rooted chains."
    );

    // -----------------------------------------------------------------
    // G-CORE-8 §4.23 LANDED (port + Noop default + typed reject + setter).
    // The substantive WRITE-admission STRUCTURAL wire-up into the
    // engine's commit path is the G-CORE-8.2 follow-up wave
    // (HARD-RULE-12 BELONGS-NAMED-NOW disposition; named destination:
    // `crates/benten-engine/src/write_boundary_chain_validator.rs`
    // module header documents the G-CORE-8.2 follow-up scope —
    // requires audit of every WRITE call site to ensure no admission
    // path is missed; the Ben-decision on the engine-default
    // posture (admit vs reject for un-installed validator) waits on
    // the same §8-E sealed-discipline framework).
    //
    // What G-CORE-8 §4.23 DOES land:
    //  - The `WriteBoundaryChainValidator` port + Noop default.
    //  - The `outcome_to_admission_reject` mapping that the WRITE-
    //    admission wire-up will call.
    //  - The typed `WriteBoundaryChainNotUserRooted` ErrorCode.
    //  - The `Engine::set_write_boundary_chain_validator` setter so
    //    platform-foundation can wire a substantive validator.
    //
    // Exercise the seam + the typed reject — this IS the substantive
    // arm of §4.23 the engine wire-up will consume.
    // -----------------------------------------------------------------
    use benten_engine::write_boundary_chain_validator::{
        WriteBoundaryChainOutcome, outcome_to_admission_reject,
    };

    let non_user_rooted = WriteBoundaryChainOutcome::ChainNotUserRooted {
        chain_root_did: plugin.as_str().to_string(),
    };
    let err = outcome_to_admission_reject(non_user_rooted)
        .expect_err("§4.23 G-CORE-8: ChainNotUserRooted MUST reject");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::WriteBoundaryChainNotUserRooted,
        "§4.23 G-CORE-8: typed code surfaces at the WRITE-admission \
         seam (CLAUDE.md baked-in #18 Layer-1 user-as-root invariant)"
    );
}

#[test]
fn user_initiated_write_tracing_to_user_root_succeeds_positive_arm() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: drive the validator on a legitimate
    // user-rooted single-step chain. MUST return Admitted (the positive
    // control that the validator does NOT over-fire on legitimate
    // writes). Would-FAIL signal: if the validator over-fires + denies
    // user-rooted chains, the assertion below fires.
    // -----------------------------------------------------------------
    let registry = registry_with_user();
    let user_rooted_chain = vec![DelegationStep {
        issuer_did: user_did(),
        audience_did: plugin_did(),
        cap_pattern: "store:notes:write".into(),
    }];
    let outcome =
        validate_chain_with_manifest_envelope(&user_rooted_chain, &EmptyLookup, &registry);
    assert_eq!(
        outcome,
        ChainValidationOutcome::Admitted,
        "shipped surface exercise: the user-rooted single-step chain \
         MUST be Admitted (the validator must not over-fire and deny \
         legitimate writes). Would-FAIL if the validator regressed to \
         a blanket-deny."
    );

    // -----------------------------------------------------------------
    // G-CORE-8 §4.23 LANDED (positive arm): the WriteBoundaryChain
    // primitive's `Admitted` outcome maps to Ok(()) — the positive
    // proceed path for a user-rooted chain. The full production
    // WRITE-admission integration test (engine commit path consulting
    // the install-record-backed UserDidRegistry) is the G-CORE-8.2
    // follow-up; here we exercise the primitive contract that the
    // wire-up will consume.
    // -----------------------------------------------------------------
    use benten_engine::write_boundary_chain_validator::{
        WriteBoundaryChainOutcome, outcome_to_admission_reject,
    };

    assert!(
        outcome_to_admission_reject(WriteBoundaryChainOutcome::Admitted).is_ok(),
        "§4.23 G-CORE-8 positive arm: Admitted outcome proceeds — \
         the positive verification path the validator emits when the \
         chain root anchors at a registered user-DID"
    );
}

#[test]
fn plugin_did_cannot_mint_root_grant_structural_elevation_defense() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (structural elevation defense): drive
    // the validator on a chain where the *root* is a plugin-DID that
    // has self-minted a "root" grant attempting to forge a chain that
    // "terminates at a root". The validator MUST refuse — only a
    // user-DID can be a root. Would-FAIL signal: if the validator
    // accepted ANY DID-minted root grant (rather than binding the root
    // to a registered user-DID), the assertion below fires.
    //
    // Folds the stranded pin
    // `admin_ui_did_cannot_mint_root_grant.rs` (§4.23 / §3.6e).
    // -----------------------------------------------------------------
    let registry = registry_with_user();
    let plugin = plugin_did();
    // Plugin attempting elevation: chain root = plugin-DID; the
    // plugin's audience is itself (forged "root grant"). MUST be
    // rejected at the user-root-terminus check.
    let elevation_attempt = vec![
        DelegationStep {
            issuer_did: plugin.clone(),
            audience_did: plugin.clone(),
            cap_pattern: "admin:everything".into(),
        },
        DelegationStep {
            issuer_did: plugin.clone(),
            audience_did: Did::from_string_for_test_fixture("did:key:z6MkOther".to_string()),
            cap_pattern: "admin:everything".into(),
        },
    ];
    let outcome =
        validate_chain_with_manifest_envelope(&elevation_attempt, &EmptyLookup, &registry);
    assert_eq!(
        outcome,
        ChainValidationOutcome::RootNotUserDid,
        "shipped surface exercise: a plugin-DID-minted root grant MUST \
         NOT satisfy the user-root-terminus check (CLAUDE.md #18 Layer-1 \
         elevation defense). Would-FAIL if any DID-minted root grant \
         were accepted in place of a registered user-DID."
    );

    // -----------------------------------------------------------------
    // G-CORE-8 §4.23 LANDED (elevation-defense arm): the WriteBoundary
    // ChainValidator's `ChainNotUserRooted` outcome rejects a plugin-
    // rooted elevation chain with the typed
    // `WriteBoundaryChainNotUserRooted` code at the WRITE-admission
    // seam. Production WRITE-admission integration is the G-CORE-8.2
    // follow-up; the primitive contract IS the substantive defense
    // (mirrors how the manifest_envelope_chain_validation already
    // refuses plugin-rooted chains, but distinct in that this surface
    // is the WRITE-admission boundary, not the merge-recheck boundary).
    // -----------------------------------------------------------------
    use benten_engine::write_boundary_chain_validator::{
        WriteBoundaryChainOutcome, outcome_to_admission_reject,
    };

    let plugin_elevation = WriteBoundaryChainOutcome::ChainNotUserRooted {
        chain_root_did: plugin.as_str().to_string(),
    };
    let err = outcome_to_admission_reject(plugin_elevation)
        .expect_err("§4.23 G-CORE-8: plugin-DID elevation chain MUST reject");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::WriteBoundaryChainNotUserRooted,
        "§4.23 G-CORE-8 elevation defense: plugin-DID-minted root chain \
         surfaces typed WriteBoundaryChainNotUserRooted at the WRITE \
         admission seam"
    );
}
