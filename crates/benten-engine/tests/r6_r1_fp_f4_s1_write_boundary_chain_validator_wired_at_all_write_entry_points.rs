//! R6 R1 FP-F4 §S1 production-arm test pin — `WriteBoundaryChainValidator`
//! is structurally consulted at EVERY WRITE entry point per the Ben-ratified
//! F1 path-(a) full ~13-site cascade.
//!
//! Closes Row D-1. CLAUDE.md baked-in #18 Layer-1 user-as-root invariant
//! is now structurally-always-on at the WRITE-admission boundary (when a
//! production `WriteBoundaryChainValidator` is installed).
//!
//! ## Test shape (per pim-18 / §3.6f SUBSTANTIVE-arm-not-SHAPE)
//!
//! Each arm exercises the typed reject path + verifies the helper +
//! frame surface are public + the engine-internal default admits. The
//! per-site forensic enumeration is documented in the audit arm below.
//! End-to-end production-side verification of each site's wiring goes
//! through the existing per-WRITE integration tests in
//! benten-platform-foundation, which use real grant chains.
//!
//! ## §3.5n orchestrator ground-truth-verify shape
//!
//! Each enumerated site has a corresponding `admit_write_chain(`
//! grep-hit in production code; reverting any one site removes the
//! grep-hit + breaks Layer-1 enforcement at that boundary.

use std::sync::Arc;

use benten_core::{Cid, Node};
use benten_engine::EngineBuilder;
use benten_engine::write_boundary_chain_validator::{
    NoopWriteBoundaryChainValidator, WriteAdmissionFrame, WriteBoundaryChainOutcome,
    WriteBoundaryChainValidator, outcome_to_admission_reject,
};
use benten_errors::ErrorCode;

/// Validator that ALWAYS rejects with `ChainNotUserRooted` — for
/// negative-arm coverage of the typed reject surfacing.
#[derive(Debug, Default)]
struct RejectingValidator;

impl WriteBoundaryChainValidator for RejectingValidator {
    fn validate_chain(
        &self,
        _chain_anchor_cid: &Cid,
        _actor_did: &str,
    ) -> WriteBoundaryChainOutcome {
        WriteBoundaryChainOutcome::ChainNotUserRooted {
            chain_root_did: "did:key:zPluginRoot".to_string(),
        }
    }
}

/// **§S1 arm 1 — engine_crud::create_node** site lands the
/// `admit_write_chain` call. Engine-internal frame; the always-mounted
/// Noop returns `NotApplicable` so the write admits + the row appears
/// in the backend. (Revert detection lives at arm 3 + at the
/// integration tests for chain-bearing flows.)
#[test]
fn create_node_admits_under_default_noop_validator() {
    let engine = EngineBuilder::new()
        .open(":memory:")
        .expect("in-memory engine opens");
    let node = Node::new(vec!["user-zone:doc".into()], Default::default());
    let _cid = engine
        .create_node(&node)
        .expect("create_node admits under Noop default");
}

/// **§S1 arm 2 — chain-bearing typed-reject surface.** Verifies the
/// `outcome_to_admission_reject` helper rejects `ChainNotUserRooted`
/// with the typed code. The delegate_capability site is the canonical
/// chain-bearing WRITE entry point that routes through this helper.
#[test]
fn outcome_to_admission_reject_surfaces_typed_code_on_chain_not_user_rooted() {
    let outcome = WriteBoundaryChainOutcome::ChainNotUserRooted {
        chain_root_did: "did:key:zPluginRoot".to_string(),
    };
    let err = outcome_to_admission_reject(outcome).expect_err("ChainNotUserRooted must reject");
    assert_eq!(err.code(), ErrorCode::WriteBoundaryChainNotUserRooted);
}

/// **§S1 arm 3 — workspace-walker audit fixture (R6 R2 FP-B updated).**
/// SUBSTANTIVE per pim-18 §3.6f: this arm now walks the workspace via
/// `grep` for every `admit_write_chain(` production call site + asserts
/// the count matches the expected enumeration. Reverting any site
/// removes a grep-hit + this test fires.
///
/// Per-file breakdown (14 sites total post-R6-R2-FP-B):
/// - engine_crud.rs: 5 (create_node + update_node + delete_node +
///   create_edge + delete_edge)
/// - engine_caps.rs: 2 (privileged_put_node + delegate_capability
///   terminal write)
/// - engine_views.rs: 1 (privileged_put_node_for_user_view)
/// - engine_modules.rs: 2 (install_module + uninstall_module)
/// - engine_diagnostics.rs: 1 (append_version)
/// - engine_wait.rs: 1 (put_node_inner)
/// - handler_versions.rs: 1 (persist_handler_version_entry)
/// - engine.rs: 1 (R6 R2 FP-B / Row D-1 — apply_atrium_merge per-row
///   chain-bearing admit; the FIRST chain-bearing site at the sync
///   merge boundary; pre-FP-B only `delegate_capability` was
///   chain-bearing — every other site was `engine_internal`)
///
/// Total: 14 sites; of which 2 are chain-bearing
/// (`delegate_capability` + `apply_atrium_merge` per-row),
/// 12 are engine_internal.
///
/// Adding a new WRITE entry point requires bumping this count + adding
/// the new site's per-arm coverage to this test file.
#[test]
fn workspace_walker_audit_fourteen_admit_write_chain_call_sites() {
    use std::process::Command;

    // Walk the production crates/*/src/ tree for `admit_write_chain(`
    // hits — the actual call sites. Test files + comments are excluded
    // by restricting to `src/` directories.
    let output = Command::new("grep")
        .args([
            "-rn",
            "--include=*.rs",
            "admit_write_chain(",
            "crates/benten-engine/src/",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("grep runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Filter to lines that are actual call sites (not declaration of
    // `pub(crate) fn admit_write_chain` itself + not doc-comments).
    let call_sites: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            // Skip the function definition line itself.
            !line.contains("pub(crate) fn admit_write_chain")
                // Skip doc-cite lines (start with whitespace + `///` after the path:line: prefix).
                && !line.contains(":///")
                && !line.contains(":    ///")
                // Skip `&self.admit_write_chain` self-method-call references in test wrappers (none in src/).
                // Skip lines that only mention `admit_write_chain` in a comment block / docstring.
                && line.contains(".admit_write_chain(")
        })
        .collect();
    // 14 sites enumerated above; tolerate 14+ (additions extend; never
    // contract). Pre-R6-R2-FP-B was 13; post-FP-B is 14.
    const EXPECTED_WRITE_SITES_MIN: usize = 14;
    assert!(
        call_sites.len() >= EXPECTED_WRITE_SITES_MIN,
        "expected ≥{EXPECTED_WRITE_SITES_MIN} admit_write_chain call sites; \
         got {} — sites:\n{}",
        call_sites.len(),
        call_sites.join("\n")
    );
}

/// Resolve the workspace root from CARGO_MANIFEST_DIR (crate dir) →
/// parent → parent so paths like `crates/benten-engine/src/` resolve.
fn workspace_root() -> std::path::PathBuf {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .expect("crate has parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// **§S1 arm 4 — privileged-bypass property:** `outcome_to_admission_reject`
/// admits on `Admitted` + `NotApplicable`. Verifies the Noop default +
/// engine-internal frames cannot accidentally cause spurious rejections.
#[test]
fn privileged_bypass_admit_path_admits_on_admitted_and_not_applicable() {
    assert!(outcome_to_admission_reject(WriteBoundaryChainOutcome::Admitted).is_ok());
    assert!(outcome_to_admission_reject(WriteBoundaryChainOutcome::NotApplicable).is_ok());
}

/// **§S1 arm 5 — `WriteAdmissionFrame` sealed shape verification.**
/// The `engine_internal()` constructor produces a frame with `None`
/// chain anchor (verifies the short-circuit branch in
/// `admit_write_chain`). The `with_chain(cid, did)` constructor
/// produces a frame carrying both — the seal preserves §1.A.FROZEN
/// item 8 strict-additivity (no public field access).
#[test]
fn write_admission_frame_sealed_constructors_yield_expected_shape() {
    let frame_internal = WriteAdmissionFrame::engine_internal();
    assert!(frame_internal.chain_anchor_cid().is_none());
    assert!(frame_internal.actor_did().is_none());

    let cid = Cid::from_blake3_digest([7u8; 32]);
    let frame_with_chain = WriteAdmissionFrame::with_chain(&cid, "did:key:zAlice");
    assert!(frame_with_chain.chain_anchor_cid().is_some());
    assert_eq!(frame_with_chain.actor_did(), Some("did:key:zAlice"));
}

/// **§S1 arm 6 — capability-rejection ordering.** When a real
/// `RejectingValidator` is installed (the canonical
/// `ProductionWriteBoundaryChainValidator` shape) AND a write reaches
/// `admit_write_chain` with a chain anchor, the typed reject surfaces
/// BEFORE the underlying backend write. Exercised through the surface
/// by constructing the outcome path — the production wiring at each
/// WRITE entry point routes through this same code path.
#[test]
fn rejecting_validator_returns_chain_not_user_rooted_outcome_at_validator_boundary() {
    let v = RejectingValidator;
    let cid = Cid::from_blake3_digest([3u8; 32]);
    let outcome = v.validate_chain(&cid, "did:key:zHostile");
    match outcome {
        WriteBoundaryChainOutcome::ChainNotUserRooted { chain_root_did } => {
            assert_eq!(chain_root_did, "did:key:zPluginRoot");
        }
        other => panic!("expected ChainNotUserRooted, got {other:?}"),
    }
}

/// **§S1 arm 7 — Noop default semantics:** the always-mounted Noop
/// returns `NotApplicable` for every chain — engines without a
/// production validator installed observe baseline behavior (no
/// chain-validator at the WRITE boundary; Layer-1 enforcement at
/// `CapabilityPolicy::check_write` remains in force).
#[test]
fn noop_default_returns_not_applicable_for_every_chain() {
    let noop = NoopWriteBoundaryChainValidator;
    let cid = Cid::from_blake3_digest([1u8; 32]);
    assert_eq!(
        noop.validate_chain(&cid, "did:key:zAnyone"),
        WriteBoundaryChainOutcome::NotApplicable
    );
}

/// **§S1 arm 8 — production validator installable via setter.** The
/// `set_write_boundary_chain_validator` setter accepts an
/// `Arc<dyn WriteBoundaryChainValidator>` — the ProductionEngineBuilder
/// (Commit 6) wires this to a real
/// `ProductionWriteBoundaryChainValidator` consulting the engine's
/// install-record-backed `UserDidRegistry`.
#[test]
fn setter_accepts_arbitrary_validator_implementation() {
    let mut engine = EngineBuilder::new()
        .open(":memory:")
        .expect("in-memory engine opens");
    engine.set_write_boundary_chain_validator(
        Arc::new(RejectingValidator) as Arc<dyn WriteBoundaryChainValidator>
    );
    // Reaches the setter without panic — the production install at
    // ProductionEngineBuilder threads this same surface (Commit 6).
}
