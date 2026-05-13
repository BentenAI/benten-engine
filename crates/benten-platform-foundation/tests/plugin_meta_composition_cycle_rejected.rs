//! G24-D row pin — meta-plugin composition cycle rejection.
//!
//! Per post-R1-triage Q2 ratification: meta-plugins reference sub-
//! plugins recursively. Install-time cycle detection rejects cycles
//! with `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED`.

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_platform_foundation::plugin_manifest::detect_composition_cycle;
use common::manifest_fixtures::minimal_manifest;

#[test]
fn meta_plugin_composition_cycle_rejected_with_typed_error_code() {
    // SUBSTANTIVE per pim-2 §3.6b: build a cycle A -> B -> A;
    // exercise detect_composition_cycle at HEAD; expect typed
    // PluginMetaCompositionCycleRejected. Would-FAIL if cycle-walk
    // missed (would return Ok or infinite-loop).
    //
    // Use distinct CID values so root_cid != child_cid identity is real.
    let a_cid = Cid::from_blake3_digest([0xAAu8; 32]);
    let b_cid = Cid::from_blake3_digest([0xBBu8; 32]);

    let mut a = minimal_manifest();
    a.composes_plugins = Some(vec![b_cid]);

    let mut b = minimal_manifest();
    b.composes_plugins = Some(vec![a_cid]); // cycle back to a

    // Resolver: returns b's manifest when looking up b_cid; nothing
    // else.
    let resolver = |cid: &Cid| -> Option<_> { if *cid == b_cid { Some(b.clone()) } else { None } };

    let result = detect_composition_cycle(a_cid, &a, &resolver);
    let err = result.expect_err("cycle MUST be rejected");
    assert_eq!(
        err,
        ErrorCode::PluginMetaCompositionCycleRejected,
        "cycle MUST surface typed PluginMetaCompositionCycleRejected; \
         would-FAIL if cycle-walk skipped"
    );
}

#[test]
fn meta_plugin_acyclic_composition_admitted_no_typed_error() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: non-cyclic composition
    // (A -> B -> nothing) admits. Would-FAIL if detector over-rejected.
    let a_cid = Cid::from_blake3_digest([0xAAu8; 32]);
    let b_cid = Cid::from_blake3_digest([0xBBu8; 32]);

    let mut a = minimal_manifest();
    a.composes_plugins = Some(vec![b_cid]);

    let mut b = minimal_manifest();
    b.composes_plugins = None; // leaf

    let resolver = |cid: &Cid| -> Option<_> { if *cid == b_cid { Some(b.clone()) } else { None } };

    let result = detect_composition_cycle(a_cid, &a, &resolver);
    result.expect("acyclic chain MUST admit");
}

#[test]
#[allow(clippy::too_many_lines)]
fn meta_plugin_recursive_walk_uses_engine_evaluator_no_new_primitive() {
    // **R4b-FP-1 Seam 4** un-ignore — substantive composition: grep-
    // walk asserting no NEW primitive variant minted for plugin
    // composition (CLAUDE.md #1 12-primitive-irreducible) +
    // exercises install_plugin → detect_composition_cycle wiring.
    use benten_id::keypair::Keypair;
    use benten_platform_foundation::plugin_library::PluginLibrary;
    use benten_platform_foundation::plugin_lifecycle::{
        InMemoryInstallCascade, InstallContext, InstallerShape, install_plugin,
    };
    use benten_platform_foundation::plugin_manifest::{
        CapRequirement, PluginManifest, SharesPolicy, sign_manifest,
    };

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let benten_eval_src = manifest_dir
        .parent()
        .expect("workspace crates/ parent")
        .join("benten-eval")
        .join("src");

    // pim-18 §3.6f vacuous-truth defense.
    assert!(
        benten_eval_src.exists() && benten_eval_src.is_dir(),
        "walked root must exist: {benten_eval_src:?}"
    );

    let forbidden_patterns = [
        "PluginComposePrimitive",
        "MetaPluginPrimitive",
        "ComposeSubgraphPrimitive",
        "CompositionWalkPrimitive",
    ];

    let mut walked_files = 0usize;
    let mut violations: Vec<(String, &str)> = Vec::new();
    for entry in walkdir::WalkDir::new(&benten_eval_src)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
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
    assert!(walked_files > 0, "vacuous-truth defense: walked 0 files");
    assert!(
        violations.is_empty(),
        "benten-eval/src/ MUST NOT introduce new primitive variants \
         for plugin composition (CLAUDE.md #1). Violations: {violations:?}"
    );

    // Substantive: install_plugin wires detect_composition_cycle at
    // Step 6 (Seam 4). Build A→B→A cycle; exercise install_plugin;
    // expect typed PluginMetaCompositionCycleRejected.
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Build a self-loop cycle: A composes [A_final_cid]. We can use
    // a fixed-point iteration: build A with placeholder composes,
    // compute its cid, then rebuild A with composes=[its own cid].
    // After rebuild the cid changes again — we then call
    // detect_composition_cycle with the FINAL cid + a manifest whose
    // composes_plugins points at that same final cid. This is
    // legitimate test of "cycle detected at first hop" since the
    // detect_composition_cycle implementation checks `child_cid ==
    // root_cid` BEFORE recursing.
    //
    // The simpler path: install_plugin verifies content_cid matches
    // computed cid (Step 1). So we need a manifest where:
    //   manifest.content_cid == manifest.compute_content_cid()
    //   AND manifest.composes_plugins.contains(&manifest.content_cid)
    //
    // Achievable via a 2-pass: compute a candidate cid with
    // composes=[placeholder], inspect that cid, then set
    // composes=[that cid], recompute. The cid CHANGES because the
    // body differs, so iterate. In practice we don't need exact self-
    // loop — instead exercise via install_plugin → resolver returning
    // a child whose composes references the ROOT. Since detect_cycle
    // walks all paths, this is robust against the fixed-point issue.
    let placeholder = Cid::from_blake3_digest([0xFFu8; 32]);
    let mut a = PluginManifest {
        plugin_name: "meta-a".to_string(),
        content_cid: placeholder,
        peer_did: alice.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy::none(),
        renderer_config: None,
        composes_plugins: Some(vec![placeholder]), // points at b_marker (set after)
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    // Compute a stable CID with a known b_marker fixed in advance.
    let b_marker = Cid::from_blake3_digest([0xBBu8; 32]);
    a.composes_plugins = Some(vec![b_marker]);
    a.content_cid = a.compute_content_cid();
    a.peer_signature = sign_manifest(&a, &alice);
    let a_final_cid = a.content_cid;

    // B's resolver-returned shape has composes pointing back at A.
    // Since B's CID never goes through manifest validation (only A
    // does — A is the install target), B's content_cid + signature
    // don't need to match anything (detect_composition_cycle never
    // verifies them). So we can freely build B pointing at a_final_cid.
    let b = PluginManifest {
        plugin_name: "meta-b".to_string(),
        content_cid: b_marker,
        peer_did: alice.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy::none(),
        renderer_config: None,
        composes_plugins: Some(vec![a_final_cid]), // closes the cycle
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };

    let bytes = serde_ipld_dagcbor::to_vec(&a).expect("encode");
    let install_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        a_final_cid,
        benten_id::did::Did::from_string_unchecked("did:key:z6MkMetaCycleTest".to_string()),
        4,
    );

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let mut cascade_minter = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<benten_id::did::Did> = vec![];
    let mut ctx = InstallContext {
        cap_minter: &mut cascade_minter,
        private_ns: &mut private_ns,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
    };
    let b_clone = b.clone();
    let attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &bytes,
        &a_final_cid,
        &install_record,
        1,
        &|cid: &Cid| {
            if *cid == b_marker {
                Some(b_clone.clone())
            } else {
                None
            }
        },
    );
    let err = attempt.expect_err("cycle install MUST be rejected at Seam 4");
    assert_eq!(
        err,
        ErrorCode::PluginMetaCompositionCycleRejected,
        "Seam 4: install_plugin MUST wire detect_composition_cycle (Step 6); \
         would-FAIL if cycle-walk skipped"
    );
    // pim-2 §3.6b sub-rule 4: pin the FULL no-partial-state-commit
    // invariant. Step 6 (cycle detect) precedes Step 8 (DID mint) +
    // Step 9 (cap cascade) + Step 10 (private-ns provision) + Step 11
    // (library insert) — all four MUST be empty if Step 6 rejects.
    assert!(
        library.is_empty(),
        "cycle-rejected install MUST NOT commit library entry (Step 11 unreached)"
    );
    assert!(
        store.is_empty(),
        "cycle-rejected install MUST NOT persist plugin-DID (Step 8 unreached)"
    );
    assert!(
        cascade_minter.minted_grants().is_empty(),
        "cycle-rejected install MUST NOT mint cap-cascade grants (Step 9 unreached)"
    );
    assert_eq!(
        private_ns.provisioned_count(),
        0,
        "cycle-rejected install MUST NOT provision private namespace (Step 10 unreached)"
    );
}
