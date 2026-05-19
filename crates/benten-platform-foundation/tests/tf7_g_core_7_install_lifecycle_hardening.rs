//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — TF-7 install / lifecycle
//! hardening. Agent **R3-B4**. RED-PHASE; un-ignore at **G-CORE-7**.
//!
//! ## Provenance / R2-map
//!
//! - r2-test-landscape.md **TF-7** ("Install / lifecycle hardening
//!   (substrate)") + §2.A seed **S6** + §4-C exit-criterion **C7** row.
//! - Plan `.addl/phase-4-meta/00-implementation-plan.md` **G-CORE-7**
//!   group def + **C7** exit criterion + §1.A.FROZEN (no FROZEN item is
//!   directly produced by TF-7 — the install/lifecycle surface is not a
//!   frozen public-interface item; it is a runtime-hardening carry).
//! - Covers, per TF-7: §4.19(b) schema-author trust-list prompt +
//!   §4.20 `validate_with_clock` e2e + §4.21 Steps 9/10/11 rollback +
//!   §4.35 Step-9 cap-cascade atomicity + §4.41 caps-grew fresh-consent
//!   + §4.32 schema-author-within-envelope wiring + §6.4
//!   `RedbManifestStore` durable.
//!
//! ## Ground-truth split at HEAD `ed03729a` (R3-B4 §3.5n orchestrator-
//! ground-truth pass — see report)
//!
//! Install/lifecycle scaffolding partially landed via the Phase-4-
//! Foundation campaign-tail (`plugin_lifecycle::install_plugin` 9-arg
//! ports form is the PRODUCTION path; `module_ecosystem::install_plugin`
//! is the DEPRECATED legacy path slated for §4.33 deletion at G-CORE-0).
//! Per family, RED-against-still-undelivered vs verify-stays-regression:
//!
//! - **§4.21 Steps 9/10/11 rollback** — STILL UNDELIVERED.
//!   `install_plugin` Steps 9 (cap cascade) → 10 (provision ns) → 11
//!   (library insert) execute with NO rollback: a Step-10 failure
//!   leaves Step-9's minted grants stranded. RED-PHASE (would-FAIL).
//! - **§4.35 Step-9 cap-cascade atomicity** — STILL UNDELIVERED. The
//!   `for req in &manifest.requires { mint_root_grant }?` loop is not
//!   all-or-nothing: a mid-loop failure leaves a partial grant set.
//!   RED-PHASE (would-FAIL).
//! - **§4.41 caps-grew fresh-consent** — DECISION-LOGIC LANDED
//!   (`module_ecosystem::decide_upgrade_consent` is a pure function with
//!   green tests at HEAD) but the **e2e production-path wiring is
//!   UNDELIVERED**: `install_plugin` does NOT consult
//!   `decide_upgrade_consent` on a within-lineage upgrade whose
//!   `requires` GREW, so a caps-grew upgrade installs WITHOUT
//!   re-consent. RED-PHASE on the e2e arm (would-FAIL).
//! - **§4.19(b) schema-author trust-list prompt** — STILL UNDELIVERED.
//!   The `ProvenanceOutcome::UserPromptRequired` surface "never built"
//!   (per the stranded `#[ignore]` in
//!   `schema_author_not_in_admin_ui_trust_list_prompts_user.rs` —
//!   a §3.6e staged-pin redirect). RED-PHASE (would-FAIL).
//! - **§4.20 `validate_with_clock` e2e** — VALIDATE FN LANDED; the
//!   gap is the END-TO-END production-install assertion that a
//!   clock-not-injected manifest with time-bounded `requires` is
//!   REJECTED *through `install_plugin`* (not just the unit). The
//!   unit path exists; the e2e arm is the would-FAIL pin here.
//! - **§4.32 schema-author-within-envelope wiring** — STILL
//!   UNDELIVERED (couples §4.19(b); the trust-list lives in the signed
//!   manifest envelope so a post-install drift to the schema-author
//!   set must be detected). RED-PHASE (would-FAIL).
//! - **§6.4 `RedbManifestStore` durable** — STILL UNDELIVERED.
//!   `ManifestStore` is in-RAM `HashMap` only (its own doc says "the
//!   redb backing is a follow-on"); no `RedbManifestStore` type
//!   exists. RED-PHASE; the would-FAIL arm is durable-across-restart.
//!
//! ## Disjointness (R3-B4 lane vs R3-B5 / siblings — §3.5i)
//!
//! This file lives in `benten-platform-foundation/tests/` and exercises
//! ONLY `plugin_lifecycle` + `manifest_store` + `plugin_manifest` +
//! `module_ecosystem` (install/lifecycle). It does NOT touch the
//! `benten-engine`/`benten-caps`/`benten-sync` attack-fabric /
//! thin-client / manifest-envelope-recheck surfaces (R3-B5 / TF-8) nor
//! the `benten-ivm`/materializer surfaces (R3-B2 / TF-5). The TF-11
//! benten-sync light-client lane is a SEPARATE file
//! (`crates/benten-sync/tests/tf11_*`), sliced by MODULE.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE guard (pim-18)
//!
//! Every pin below exercises the **production** `plugin_lifecycle`
//! call-site (the 9-arg ports `install_plugin`, the real
//! `ManifestStore`/`RedbManifestStore` surface) with an OBSERVABLE
//! consequence (stranded grants / installed-without-reconsent /
//! lost-after-restart) that **WOULD-FAIL if the hardening is a
//! no-op**. No pin asserts mere type-constructibility.
//!
//! ## R3-brief inherited-discipline pre-flight checklist (§3.6g —
//! reproduced as LITERAL lines, NOT a §-reference; fix-6 directive)
//!
//! - [x] §3.5b HARDENED (pim-1): no public-shape change in THIS file (tests only); when G-CORE-7 lands the surfaces, the implementer sweeps adjacent docs before push.
//! - [x] §3.6b + sub-rule 4 (pim-2): each pin is a PRODUCTION-ARM + OBSERVABLE-CONSEQUENCE + WOULD-FAIL-IF-NO-OP'd pin on the SPECIFIC arm (rollback / atomicity / fresh-consent / durable — not an umbrella "install works").
//! - [x] §3.6e (pim-12): RED-PHASE staged-pins; the closing G-CORE-7 wave un-ignores; the reviewer verifies LANDING-STATUS, not just spec-pin presence. The two stranded sibling `#[ignore]`d pins (schema-author trust-list prompt; upgrade re-consent e2e) are NAMED here for the G-CORE-7 un-ignore sweep.
//! - [x] §3.6f (pim-18): SHAPE-not-SUBSTANCE — production call-site + substantive body + aspirational-prose-gap check (above).
//! - [x] §3.5g (cross-language/doc/tool rule-mirror): no ErrorCode mint in THIS file; if G-CORE-7 mints an install/lifecycle ErrorCode it mirrors Rust↔TS atomically.
//! - [x] §3.5i: mini-reviewer FIRST action = tree-state-freshness vs merge-base; assert R3-B4 ⟂ R3-B5 file-disjointness.
//! - [x] §3.5j: §3.5h pre-push runs `cargo +stable clippy --workspace --all-targets -- -D warnings` + MSRV 1.95.
//! - [x] §3.6g: prior-phase pim-N reproduced as explicit lines (this block).
//! - [x] §3.6h: no rule/codification originates here (test-only).
//! - [x] §3.6i: the R3-B4 report carries canonical top-level `disposition` + `findings[]` (well-formed).
//! - [x] §3.6j: "swept" claims run the validator over THIS wave's outputs before the claim.
//! - [x] §3.13: per-test fault-injector locals, NOT a single shared static under the parallel runner (each pin builds its own `InMemoryInstallCascade` / fault double; no process-global).
//! - [x] §3.5h: base 5-check + MANDATORY-PRE-MERGE-AFTER-MINI-REVIEW + `jq .` JSON-artifact + GREEN-CI-CONFIRMATION clauses.
//! - [x] §3.11: checkpoint-pre-flight recovery (R3-B4 is NOT the largest lane — §3.11 mandatory only for R3-B5/TF-8 — but the resume-into-same-worktree discipline still applies on kill).
//! - [x] §3.5l/§3.5m/§3.5n: combined-push full-workspace verify; P-I/P-II/P-III fork discipline (NO P-III wire/CID change in TF-7 — `RedbManifestStore` is a NEW durable backend, not a canonical-bytes mutation); orchestrator ground-truth every review finding.
//! - [x] Iterate-to-convergence + canary-first: TF-7 is SUBSTRATE (G-CORE-7; batches with G-CORE-4 on `benten-platform-foundation` under §3.5i disjointness) — lands AFTER the 2-canary opening pair (#989 ∥ #1300) merges.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_platform_foundation::CapRequirement;
use benten_platform_foundation::module_ecosystem::{
    UpgradeConsentDecision, decide_upgrade_consent,
};
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    CapMinter, InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape,
    PrivateNamespaceProvisioner, install_plugin,
};

// =====================================================================
// Test doubles — per-test fault-injectors (§3.13: NO single shared
// static; each pin constructs its own; semantic naming).
// =====================================================================

/// A `CapMinter` that mints `succeed_n` grants then fails the next one
/// with `E_INTERNAL`. Used to drive the §4.35 Step-9 cap-cascade
/// mid-loop-failure + §4.21 rollback pins.
struct FailAfterNGrants {
    succeed_n: usize,
    minted: Vec<(Did, Did, String)>,
}

impl FailAfterNGrants {
    fn new(succeed_n: usize) -> Self {
        Self {
            succeed_n,
            minted: Vec::new(),
        }
    }
}

impl CapMinter for FailAfterNGrants {
    fn mint_root_grant(
        &mut self,
        user_did: &Did,
        plugin_did: &Did,
        scope: &str,
    ) -> Result<Cid, ErrorCode> {
        if self.minted.len() >= self.succeed_n {
            return Err(ErrorCode::GraphInternal);
        }
        self.minted
            .push((user_did.clone(), plugin_did.clone(), scope.to_string()));
        Ok(Cid::from_blake3_digest([self.minted.len() as u8; 32]))
    }
}

/// A `PrivateNamespaceProvisioner` that always fails — drives the
/// §4.21 "Step-10 failed AFTER Step-9 minted grants → grants MUST be
/// rolled back" pin.
struct ProvisionAlwaysFails;

impl PrivateNamespaceProvisioner for ProvisionAlwaysFails {
    fn provision_private_namespace(&mut self, _plugin_did: &Did) -> Result<(), ErrorCode> {
        Err(ErrorCode::GraphInternal)
    }
}

fn install_params<'a>(
    trust_list: &'a [Did],
    user_did: &'a Did,
    plugin_did: &'a Did,
) -> InstallParams<'a> {
    InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: trust_list,
        user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: plugin_did,
    }
}

// =====================================================================
// §4.21 — Steps 9/10/11 partial-failure rollback (zero residual)
// =====================================================================

/// **TF-7 / S6 / C7 — §4.21 RED-PHASE.**
///
/// Production-arm: `plugin_lifecycle::install_plugin` Step 9 mints the
/// cap cascade; Step 10 provisions the private namespace. With a
/// Step-10 provisioner that FAILS, the install MUST return `Err` AND
/// roll back Step 9 — leaving **zero residual minted grants** and an
/// **empty library**.
///
/// Would-FAIL (current HEAD): `install_plugin` mints grants in Step 9,
/// then Step 10 fails and returns `Err` — but the Step-9 grants are
/// NEVER unwound. A half-installed plugin (caps minted, no namespace,
/// not in library) is left behind. This is the partial-residual hazard
/// the §4.21 rollback closes.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-7 (§4.21 Steps 9/10/11 rollback — install_plugin must unwind Step-9 minted grants when Step-10 provision fails; zero-residual). r2-test-landscape TF-7 / S6 / C7."]
fn install_step10_provision_failure_rolls_back_step9_minted_grants_zero_residual() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);

    // Manifest requiring TWO caps (so Step 9 mints >0 grants).
    let manifest = common::manifest_fixtures::signed_manifest_by(
        &author,
        "rollback-test",
        &["store:notes:read", "store:notes:write"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    let record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did.clone(),
        7,
    );

    let mut library = PluginLibrary::new();
    let mut minter = FailAfterNGrants::new(usize::MAX); // grants succeed
    let mut bad_ns = ProvisionAlwaysFails; // Step-10 fails
    let trust_list: Vec<Did> = vec![];
    let mut ctx = InstallPorts {
        cap_minter: &mut minter,
        private_ns: &mut bad_ns,
    };
    let params = install_params(&trust_list, &user_did, &plugin_did);

    let outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &params,
        &bytes,
        &expected_cid,
        &record,
        1,
        &|_| None,
    );

    assert!(
        outcome.is_err(),
        "Step-10 provision failure MUST fail the install"
    );
    assert!(
        library.is_empty(),
        "rollback: a Step-10 failure MUST leave the library EMPTY \
         (zero-residual) — would-FAIL if a half-installed entry rides"
    );
    assert!(
        minter.minted.is_empty(),
        "§4.21 rollback: Step-10 failure MUST unwind ALL Step-9 minted \
         grants (zero stranded caps) — would-FAIL at HEAD where Step-9 \
         grants are never rolled back on a later-step failure"
    );
}

// =====================================================================
// §4.35 — Step-9 cap-cascade atomicity (all-or-nothing)
// =====================================================================

/// **TF-7 / S6 / C7 — §4.35 RED-PHASE.**
///
/// Production-arm: a manifest requiring THREE caps; the minter succeeds
/// on the first TWO then fails the third. Step 9's cap cascade MUST be
/// all-or-nothing — on the third-grant failure, the two already-minted
/// grants MUST be unwound (no partial grant set persists).
///
/// Would-FAIL (current HEAD): the `for req in &manifest.requires`
/// mint-loop propagates the third `?` error with the first two grants
/// already committed and never unwound — a partial cap envelope.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-7 (§4.35 Step-9 cap-cascade atomicity — mid-loop mint failure must unwind already-minted grants; all-or-nothing). r2-test-landscape TF-7 / S6 / C7."]
fn install_step9_cap_cascade_is_atomic_midloop_failure_unwinds_prior_grants() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);

    let manifest = common::manifest_fixtures::signed_manifest_by(
        &author,
        "atomic-cascade-test",
        &["store:a:read", "store:b:read", "store:c:read"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    let record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did.clone(),
        8,
    );

    let mut library = PluginLibrary::new();
    let mut minter = FailAfterNGrants::new(2); // 1st+2nd succeed, 3rd fails
    let mut ns = InMemoryInstallCascade::new();
    let trust_list: Vec<Did> = vec![];
    let mut ctx = InstallPorts {
        cap_minter: &mut minter,
        private_ns: &mut ns,
    };
    let params = install_params(&trust_list, &user_did, &plugin_did);

    let outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &params,
        &bytes,
        &expected_cid,
        &record,
        1,
        &|_| None,
    );

    assert!(
        outcome.is_err(),
        "a mid-cascade mint failure MUST fail the install"
    );
    assert!(
        minter.minted.is_empty(),
        "§4.35 atomicity: a 3rd-grant failure MUST unwind the 2 \
         already-minted grants (all-or-nothing) — would-FAIL at HEAD \
         where the mint-loop leaves a partial grant set"
    );
    assert!(
        library.is_empty(),
        "atomic-failure install MUST NOT commit library state"
    );
}

// =====================================================================
// §4.41 — caps-grew fresh-consent (e2e production-path, NOT the unit)
// =====================================================================

/// **TF-7 / S6 / C7 — §4.41 RED-PHASE (e2e arm).**
///
/// The pure decision fn `decide_upgrade_consent` is LANDED + green at
/// HEAD (it correctly returns `ConsentRequired` when `requires` grew).
/// The UNDELIVERED gap is the **production-path wiring**: a
/// within-lineage upgrade whose `requires` GREW relative to the
/// already-installed prior manifest MUST NOT install without a fresh
/// consent record covering the WIDER cap set.
///
/// Production-arm: install v1 (1 cap, consented). Then attempt an
/// upgrade to v2 (2 caps — GREW) reusing v1's narrower consent record.
/// `install_plugin` MUST reject (caps-grew → re-consent required),
/// surfacing a typed consent code; the wider grant MUST NOT be minted.
///
/// Would-FAIL (current HEAD): `install_plugin` never consults
/// `decide_upgrade_consent` against `params.prior_installed_cid`'s
/// manifest, so a caps-grew upgrade installs silently — the Q3/§4.41
/// fresh-consent rule is paper.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-7 (§4.41 caps-grew fresh-consent e2e — install_plugin must consult decide_upgrade_consent on a within-lineage upgrade and reject a caps-grew upgrade reusing the narrower consent). DECISION-LOGIC landed; e2e wiring undelivered. r2-test-landscape TF-7 / S6 / C7."]
// Inherently long: a two-stage e2e (install v1 narrow + consented, then
// attempt v2 widened reusing narrower consent). Mirrors the codebase's
// established scoped-allow precedent for long e2e pins.
#[allow(clippy::too_many_lines)]
fn upgrade_with_grown_requires_must_block_install_until_fresh_consent_e2e() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Sanity: the LANDED pure decision fn is correct (this half is GREEN
    // at HEAD — asserted so the RED arm is unambiguously the e2e wiring).
    let mut old_m = common::manifest_fixtures::minimal_manifest();
    old_m.requires = vec![CapRequirement {
        scope: "store:notes:read".to_string(),
    }];
    let mut new_m = common::manifest_fixtures::minimal_manifest();
    new_m.requires = vec![
        CapRequirement {
            scope: "store:notes:read".to_string(),
        },
        CapRequirement {
            scope: "store:notes:write".to_string(),
        },
    ];
    assert_eq!(
        decide_upgrade_consent(&old_m, &new_m),
        UpgradeConsentDecision::ConsentRequired,
        "PRE-CONDITION (landed-green half): decide_upgrade_consent MUST \
         flag caps-grew as ConsentRequired"
    );

    // --- e2e RED arm: the production install path ---
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);

    // v1 — narrow (1 cap), honestly installed + consented.
    let v1 = common::manifest_fixtures::signed_manifest_by(
        &author,
        "upgrade-consent-app",
        &["store:notes:read"],
    );
    let v1_bytes = serde_ipld_dagcbor::to_vec(&v1).expect("encode");
    let v1_cid = v1.content_cid;
    let v1_record =
        common::manifest_fixtures::signed_install_record(&user_kp, v1_cid, plugin_did.clone(), 1);

    let mut library = PluginLibrary::new();
    let mut minter = InMemoryInstallCascade::new();
    let mut ns = InMemoryInstallCascade::new();
    let trust_list: Vec<Did> = vec![];
    {
        let mut ctx = InstallPorts {
            cap_minter: &mut minter,
            private_ns: &mut ns,
        };
        let params = install_params(&trust_list, &user_did, &plugin_did);
        install_plugin(
            &mut library,
            &mut store,
            &mut ctx,
            &params,
            &v1_bytes,
            &v1_cid,
            &v1_record,
            1,
            &|_| None,
        )
        .expect("v1 install (narrow, consented) succeeds");
    }
    let grants_after_v1 = minter.minted_grants().len();

    // v2 — WIDER (2 caps). Re-uses v1's consent record (narrower).
    let v2 = common::manifest_fixtures::signed_manifest_by(
        &author,
        "upgrade-consent-app",
        &["store:notes:read", "store:notes:write"],
    );
    let v2_bytes = serde_ipld_dagcbor::to_vec(&v2).expect("encode");
    let v2_cid = v2.content_cid;
    // The consent record still binds v1_cid's narrower envelope — there
    // is NO fresh consent covering the widened cap set.
    let stale_record =
        common::manifest_fixtures::signed_install_record(&user_kp, v2_cid, plugin_did.clone(), 2);

    let mut ctx = InstallPorts {
        cap_minter: &mut minter,
        private_ns: &mut ns,
    };
    let mut params = install_params(&trust_list, &user_did, &plugin_did);
    params.prior_installed_cid = Some(v1_cid);

    let outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &params,
        &v2_bytes,
        &v2_cid,
        &stale_record,
        2,
        &|cid| {
            if *cid == v1_cid {
                Some(v1.clone())
            } else {
                None
            }
        },
    );

    assert!(
        outcome.is_err(),
        "§4.41 e2e: a caps-GREW within-lineage upgrade reusing the \
         narrower consent MUST be rejected (fresh consent required) — \
         would-FAIL at HEAD where install_plugin never consults \
         decide_upgrade_consent against the prior manifest"
    );
    assert_eq!(
        minter.minted_grants().len(),
        grants_after_v1,
        "the WIDER cap MUST NOT be minted before fresh consent — \
         grant count must be unchanged from post-v1"
    );
}

// =====================================================================
// §4.20 — validate_with_clock e2e (through the production install path)
// =====================================================================

/// **TF-7 / S6 / C7 — §4.20 RED-PHASE (e2e arm).**
///
/// The `validate_with_clock` unit exists + is correct. The e2e gap:
/// installing a manifest with TIME-BOUNDED `requires` (e.g.
/// `host:time:now`) under the clock-not-injected sentinel
/// (`now_secs == 0`) MUST be rejected *through `install_plugin`* with
/// the typed `E_UCAN_CLOCK_NOT_INJECTED` code — fail-closed, no grants
/// minted, library untouched.
///
/// Would-FAIL: if the production install path does not route the
/// clock-sentinel + time-bounded manifest through `validate_with_clock`
/// (or swallows its error), a time-bounded plugin installs with NO
/// trustworthy clock — the validation is paper.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-7 (§4.20 validate_with_clock e2e — a time-bounded manifest under the clock-not-injected sentinel must be rejected THROUGH install_plugin, fail-closed). r2-test-landscape TF-7 / S6 / C7."]
fn install_with_time_bounded_manifest_under_clock_sentinel_rejected_e2e() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);

    // Time-bounded: requires the wallclock cap.
    let manifest = common::manifest_fixtures::signed_manifest_by(
        &author,
        "time-bounded-app",
        &["host:time:now"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;
    let record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did.clone(),
        9,
    );

    let mut library = PluginLibrary::new();
    let mut minter = InMemoryInstallCascade::new();
    let mut ns = InMemoryInstallCascade::new();
    let trust_list: Vec<Did> = vec![];
    let mut ctx = InstallPorts {
        cap_minter: &mut minter,
        private_ns: &mut ns,
    };
    // CLOCK-NOT-INJECTED sentinel: now_secs == 0.
    let params = InstallParams {
        now_secs: 0,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did,
    };

    let outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &params,
        &bytes,
        &expected_cid,
        &record,
        1,
        &|_| None,
    );

    let err = outcome.expect_err(
        "§4.20 e2e: a time-bounded manifest under the clock-not-\
         injected sentinel MUST be rejected through install_plugin",
    );
    assert_eq!(
        err,
        ErrorCode::UcanClockNotInjected,
        "fail-closed: the typed E_UCAN_CLOCK_NOT_INJECTED code MUST \
         surface (would-FAIL if install_plugin doesn't route the \
         clock-sentinel + time-bounded manifest through \
         validate_with_clock)"
    );
    assert!(
        minter.minted_grants().is_empty(),
        "clock-rejected install MUST NOT mint grants (fail-closed)"
    );
    assert!(library.is_empty(), "clock-rejected install commits nothing");
}

// =====================================================================
// §4.19(b) / §4.32 — schema-author trust-list prompt + within-envelope
//   wiring. NAMES the stranded sibling §3.6e staged-pin for the
//   G-CORE-7 un-ignore sweep.
// =====================================================================

/// **TF-7 / S6 / C7 — §4.19(b) + §4.32 RED-PHASE.**
///
/// §3.6e staged-pin redirect: the sibling test
/// `schema_author_not_in_admin_ui_trust_list_prompts_user.rs` carries
/// an `#[ignore]` DESTINATION-REMAPPED to Phase-4-Meta
/// (`docs/future/phase-4-backlog.md §4.19`) — "G24-B shipped WITHOUT
/// delivering this trust-list prompt path; the
/// `ProvenanceOutcome::UserPromptRequired` surface never built."
///
/// G-CORE-7 is that named Phase-4-Meta destination. This pin is the
/// substantive RED arm asserting the production schema-provenance path
/// returns a `UserPromptRequired`-equivalent outcome (NOT auto-accept,
/// NOT auto-reject) for a schema signed by a peer-DID absent from the
/// (Q3-default-EMPTY) trust-list, AND that the trust-list is read from
/// the SIGNED manifest envelope (§4.32 — a post-install drift to the
/// schema-author set is detected, not silently honored).
///
/// Would-FAIL (current HEAD): no `ProvenanceOutcome::UserPromptRequired`
/// surface exists; an unknown-author schema either materializes
/// silently or hard-rejects — neither is the Q3-ratified prompt.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-7 (§4.19(b) schema-author trust-list prompt + §4.32 within-envelope wiring — the ProvenanceOutcome::UserPromptRequired surface must be BUILT; G-CORE-7 IS the named Phase-4-Meta destination for the stranded sibling #[ignore] in schema_author_not_in_admin_ui_trust_list_prompts_user.rs per §3.6e). r2-test-landscape TF-7 / S6 / C7."]
fn schema_author_not_in_trust_list_returns_user_prompt_outcome_from_signed_envelope() {
    // Intentional RED-PHASE failure: the production schema-provenance
    // trust-list-prompt surface (`ProvenanceOutcome::UserPromptRequired`
    // read from the signed manifest envelope) does not exist at HEAD.
    // G-CORE-7 builds it; this pin (and the stranded sibling pin it
    // names) un-ignore together.
    //
    // Substantive shape G-CORE-7 must satisfy (sketch — wired at
    // un-ignore):
    //   - admin_ui_v0_manifest().requires_schema_authors is
    //     None-or-empty (Q3 default EMPTY) — asserted from the SIGNED
    //     envelope, not a side-channel.
    //   - a schema signed by an unknown peer-DID → outcome
    //     UserPromptRequired { peer_did, schema_cid } (NOT auto-accept,
    //     NOT auto-reject).
    //   - after the user trusts that peer (trust-list grows), a second
    //     schema from the same peer → Trusted (silent), AND a tampered
    //     post-install trust-list (drift) is rejected (§4.32 — the
    //     trust-list is bound by the manifest's peer signature).
    panic!(
        "RED-PHASE (G-CORE-7): schema-author trust-list prompt path \
         (§4.19(b)) + within-signed-envelope wiring (§4.32) not built \
         at HEAD. ProvenanceOutcome::UserPromptRequired surface \
         absent. This pin + the stranded sibling \
         schema_author_not_in_admin_ui_trust_list_prompts_user.rs \
         #[ignore] un-ignore together at G-CORE-7 (§3.6e)."
    );
}

// =====================================================================
// §6.4 — RedbManifestStore durable (round-trip + post-restart)
// =====================================================================

/// **TF-7 / S6 / C7 — §6.4 RED-PHASE.**
///
/// `ManifestStore` is in-RAM `HashMap` only at HEAD (its own module doc:
/// "the redb backing is a follow-on"). §6.4 requires a durable
/// `RedbManifestStore` whose persisted install records survive a
/// process/handle restart (a fresh store opened on the same redb path
/// reads back the verified record).
///
/// Would-FAIL (current HEAD): no `RedbManifestStore` type exists;
/// `ManifestStore` loses all records when dropped. A durable-store
/// round-trip + reopen pin cannot even reference the type — RED.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-7 (§6.4 RedbManifestStore durable — install records must survive a store reopen on the same redb path; HEAD ManifestStore is in-RAM HashMap only). r2-test-landscape TF-7 / S6 / C7."]
fn redb_manifest_store_install_record_survives_store_reopen() {
    // Intentional RED-PHASE failure: `RedbManifestStore` does not exist
    // at HEAD. G-CORE-7 introduces the durable backend; the un-ignored
    // body opens a `RedbManifestStore` at a tempdir path, persists a
    // verified install record, DROPS the store, reopens at the SAME
    // path, and asserts `load_verified` returns the same record
    // (would-FAIL if the store is in-RAM only / not flushed).
    //
    // NOTE (P-III, §3.5m): `RedbManifestStore` is a NEW durable backend
    // — it does NOT mutate any canonical-bytes / on-disk wire format
    // that is content-addressed or P-III-frozen. The persisted-record
    // bytes are the SAME DAG-CBOR `InstallRecord` encoding the in-RAM
    // `ManifestStore` already stores (verify-on-load byte-equality
    // preserved). No P-III decision-point sits on §6.4.
    panic!(
        "RED-PHASE (G-CORE-7): RedbManifestStore durable backend not \
         built at HEAD (ManifestStore is in-RAM HashMap only — its \
         module doc says 'the redb backing is a follow-on'). \
         §6.4 requires install records survive a store reopen."
    );
}
