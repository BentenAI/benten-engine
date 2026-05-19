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
    Did::from_string_unchecked("did:key:z6MkUser".to_string())
}

fn plugin_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkPlugin".to_string())
}

fn registry_with_user() -> UserRegistry {
    let mut users = HashSet::new();
    users.insert(user_did().as_str().to_string());
    UserRegistry { users }
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.23 user-DID root-chain \
            write-boundary validator — structurally-always-on)"]
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
        audience_did: Did::from_string_unchecked("did:key:z6MkVictim".to_string()),
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
    // RED-arm: the structurally-always-on WRITE-admission wiring is the
    // UNDELIVERED piece. The shipped primitive answers correctly; what
    // is missing is the *WRITE evaluator dispatch* consulting it on
    // every admission against the engine's install-record store
    // (mirrors Phase-3 G16-B-F PR #161 structural-always-on per-row
    // recheck — fail-CLOSED, NOT an opt-in Noop port).
    // -----------------------------------------------------------------
    panic!(
        "§4.23 write-boundary chain validator undelivered: the SHIPPED \
         `validate_chain_with_manifest_envelope` returns RootNotUserDid \
         on the adversarial chain (exercised above), but the WRITE \
         primitive admission path does NOT re-verify the capability \
         chain terminates at a user-DID root grant on EVERY write \
         against the engine's install-record-backed UserDidRegistry. A \
         write whose chain does not trace to a user-root is currently \
         admitted by the evaluator dispatch (the shipped validator is \
         not threaded structurally-always-on). G-CORE-8 must wire the \
         structurally-always-on validator (mirrors Phase-3 G16-B-F PR \
         #161 — fail-CLOSED, NOT an opt-in Noop port)."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.23 — positive arm: \
            user-initiated write traces to user-root succeeds)"]
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
    // RED-arm: the SUBSTANTIVE positive control exercised through the
    // PRODUCTION WRITE-admission path (not just the validator) requires
    // the structurally-always-on wiring + the engine install-record-
    // backed UserDidRegistry. Pending G-CORE-8.
    // -----------------------------------------------------------------
    panic!(
        "§4.23 positive-arm undelivered: pending the structurally-\
         always-on validator at the WRITE-admission seam, the positive \
         control exercised through the production WRITE-admission path \
         (engine install-record-backed UserDidRegistry, not the in-memory \
         fixture above) cannot be exercised. Un-ignore at G-CORE-8 \
         alongside the deny arm — folds the stranded \
         admin_ui_v0_user_initiated_write_succeeds pin (§3.6e)."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.23 — plugin-DID \
            cannot mint a root grant; structural elevation defense)"]
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
            audience_did: Did::from_string_unchecked("did:key:z6MkOther".to_string()),
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
    // RED-arm: the structurally-always-on WRITE-admission wiring that
    // binds the validator's UserDidRegistry to the engine's
    // install-record-signed user-DID store is undelivered. A plugin-
    // initiated WRITE at the production admission path is not
    // currently gated by this check.
    // -----------------------------------------------------------------
    panic!(
        "§4.23 plugin-elevation defense undelivered: the SHIPPED \
         validator refuses the plugin-rooted elevation chain (exercised \
         above), but the production WRITE-admission path does not \
         consult it structurally-always-on against the engine's \
         install-record-signed user-DID registry. G-CORE-8 must assert \
         the validator binds the chain root to a registered *user-DID*, \
         not any DID-minted root grant, on EVERY write admission."
    );
}
