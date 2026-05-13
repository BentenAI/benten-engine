//! G24-D row pin — plugin library subgraph + active references.
//!
//! Per CLAUDE.md #18 + D-4F-14: user's full plugin set lives as a
//! "plugin library" subgraph containing all installed versions +
//! forks. Active graph holds references to specific plugin-versions
//! currently in use. Switching active version = updating the
//! reference. Keeps old/unused versions in the library (cheap, content-
//! addressed).

mod common;

use benten_core::Cid;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::{LibraryEntry, PluginLibrary};
use common::manifest_fixtures::minimal_manifest;

fn entry(name: &str, cid: Cid, installed_at: u64) -> LibraryEntry {
    let mut m = minimal_manifest();
    m.plugin_name = name.to_string();
    let plugin_did = Keypair::generate().public_key().to_did();
    LibraryEntry {
        manifest_cid: cid,
        manifest: m,
        plugin_did,
        installed_at_nanos: installed_at,
    }
}

#[test]
fn plugin_library_retains_all_versions_when_active_ref_switches_between_them() {
    // SUBSTANTIVE per pim-2 §3.6b + pim-2-amendment sub-rule 4: build
    // a PluginLibrary at HEAD; install v1 + v2 of the same plugin
    // name; switch active between them. ASSERT: both versions remain
    // retrievable after the switch. Would-FAIL if a naive impl
    // overwrote on set_active (only retained v2).
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2 = Cid::from_blake3_digest([2u8; 32]);

    let mut library = PluginLibrary::new();
    library.insert(entry("notes-app", v1, 100));
    library.insert(entry("notes-app", v2, 200));

    // versions_of returns BOTH entries (DAG-shape retention) sorted
    // by install timestamp.
    let versions = library.versions_of("notes-app");
    assert_eq!(versions.len(), 2, "library MUST retain both versions");
    assert_eq!(versions[0].manifest_cid, v1);
    assert_eq!(versions[1].manifest_cid, v2);

    // Set active to v1.
    library.set_active("notes-app", v1).expect("v1 active OK");
    assert_eq!(library.active("notes-app"), Some(&v1));

    // SUBSTANTIVE switch-doesn't-drop: switching active to v2 does
    // NOT remove v1. Would-FAIL if naive impl over-wrote.
    library
        .set_active("notes-app", v2)
        .expect("switch to v2 OK");
    assert_eq!(library.active("notes-app"), Some(&v2));
    // Both versions STILL retrievable post-switch.
    assert!(library.get(&v1).is_some(), "v1 retained after switch");
    assert!(library.get(&v2).is_some(), "v2 retained after switch");
    assert_eq!(library.versions_of("notes-app").len(), 2);
}

#[test]
fn plugin_library_set_active_rejects_unknown_cid_with_typed_error() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: set_active for a CID not
    // in the library surfaces typed PluginManifestInvalid. Would-FAIL
    // if impl silently set a dangling reference.
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let stranger = Cid::from_blake3_digest([0xEEu8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(entry("notes-app", v1, 100));

    let err = library
        .set_active("notes-app", stranger)
        .expect_err("dangling active ref MUST be rejected");
    assert_eq!(err, benten_errors::ErrorCode::PluginManifestInvalid);
    // No active set as side effect.
    assert!(library.active("notes-app").is_none());
}

#[test]
fn plugin_library_is_real_subgraph_with_anchor_version_nodes_and_canonical_primitives() {
    // **R6-FP-D substantive pin (cag-ux-r6-r1-1 + cag-ux-r6-r1-2
    // closure)**.
    //
    // pim-2 §3.6b end-to-end test pin: the library is now a real
    // `benten_core::Subgraph`, NOT a flat HashMap. ASSERT the
    // STRUCTURE of `as_subgraph()`:
    //   1. handler_id = "plugin-library"
    //   2. library_root Node exists
    //   3. One `anchor::<plugin_name>` Node per distinct plugin name
    //   4. One `version::<cid>` Node per installed CID
    //   5. library_root → anchor edges use ITEM_TYPE label (the
    //      schema-vocabulary canonical edge for container-of-element)
    //   6. anchor → version edges use VERSION_OF label
    //   7. anchor → active-version edge uses CURRENT label (per
    //      CLAUDE.md #18 + ratification #2 per-device-local CURRENT
    //      pointer)
    //   8. Every Node is `PrimitiveKind::Read` (CLAUDE.md #1: no new
    //      primitive variant minted for the library substrate).
    //   9. The per-name `Anchor` is real (Phase-1 Cid-head-threaded
    //      surface from benten_core::version).
    //
    // Would-FAIL if the lift was a no-op (library remained a
    // HashMap-backed metadata store), if a new `PrimitiveKind::Library`
    // variant was minted, or if the Anchor+Version Node pattern wasn't
    // wired through.
    use benten_core::PrimitiveKind;
    use benten_platform_foundation::plugin_library::{
        EDGE_CURRENT, EDGE_LIBRARY_ANCHOR, EDGE_VERSION_OF, HANDLER_ID_PLUGIN_LIBRARY,
        NODE_ID_LIBRARY_ROOT, anchor_node_id, version_node_id,
    };

    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2 = Cid::from_blake3_digest([2u8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(entry("notes-app", v1, 100));
    library.insert(entry("notes-app", v2, 200));
    library.set_active("notes-app", v1).unwrap();

    let sg = library.as_subgraph();

    // (1) handler_id stable.
    assert_eq!(
        sg.handler_id(),
        HANDLER_ID_PLUGIN_LIBRARY,
        "library subgraph MUST carry foundation-owned handler_id"
    );

    // (2) library_root present.
    assert!(
        sg.nodes().iter().any(|n| n.id == NODE_ID_LIBRARY_ROOT),
        "library_root structural Node MUST be present"
    );

    // (3) + (4) anchor + version nodes present.
    let anchor_id = anchor_node_id("notes-app");
    let v1_id = version_node_id(&v1);
    let v2_id = version_node_id(&v2);
    assert!(sg.nodes().iter().any(|n| n.id == anchor_id), "anchor node");
    assert!(sg.nodes().iter().any(|n| n.id == v1_id), "v1 version node");
    assert!(sg.nodes().iter().any(|n| n.id == v2_id), "v2 version node");

    // (5) library_root → anchor edge uses ITEM_TYPE.
    assert!(
        sg.edges().iter().any(|(f, t, l)| f == NODE_ID_LIBRARY_ROOT
            && t == &anchor_id
            && l == EDGE_LIBRARY_ANCHOR),
        "library_root → anchor MUST use ITEM_TYPE label"
    );

    // (6) anchor → version edges use VERSION_OF.
    for vid in [&v1_id, &v2_id] {
        assert!(
            sg.edges()
                .iter()
                .any(|(f, t, l)| f == &anchor_id && t == vid && l == EDGE_VERSION_OF),
            "anchor → {vid} MUST use VERSION_OF label"
        );
    }

    // (7) anchor → CURRENT (active = v1).
    assert!(
        sg.edges()
            .iter()
            .any(|(f, t, l)| f == &anchor_id && t == &v1_id && l == EDGE_CURRENT),
        "anchor → active version MUST use CURRENT label"
    );

    // Switching active updates the CURRENT edge.
    library.set_active("notes-app", v2).unwrap();
    let sg2 = library.as_subgraph();
    assert!(
        sg2.edges()
            .iter()
            .any(|(f, t, l)| f == &anchor_id && t == &v2_id && l == EDGE_CURRENT),
        "CURRENT edge MUST update to v2 after set_active"
    );
    assert!(
        !sg2.edges()
            .iter()
            .any(|(f, t, l)| f == &anchor_id && t == &v1_id && l == EDGE_CURRENT),
        "stale CURRENT edge MUST be dropped"
    );

    // (8) Every Node MUST be Read kind — no new primitive variant.
    for n in sg2.nodes() {
        assert!(
            matches!(n.primitive_kind(), PrimitiveKind::Read),
            "plugin-library Node MUST be PrimitiveKind::Read; \
             would-FAIL if a new variant was minted. Got: {:?} at {}",
            n.primitive_kind(),
            n.id
        );
    }

    // (9) Phase-1 Anchor surface is real. walk_mainline returns the
    // content-CID-chained sequence (not wall-clock-keyed).
    let mainline = library.walk_mainline("notes-app");
    assert!(
        mainline.contains(&v1),
        "walk_mainline MUST include v1 (root or chain entry)"
    );
    assert!(
        mainline.contains(&v2),
        "walk_mainline MUST include v2 (appended via append_version)"
    );
    assert!(
        library.anchor("notes-app").is_some(),
        "Phase-1 Anchor MUST be present for any installed plugin"
    );
}

#[test]
fn plugin_library_remove_clears_active_ref_when_dropped_was_active() {
    // SUBSTANTIVE per pim-2 §3.6b: uninstall path (remove) clears the
    // active ref when it pointed at the dropped CID. Would-FAIL if
    // impl left a dangling active ref.
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2 = Cid::from_blake3_digest([2u8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(entry("notes-app", v1, 100));
    library.insert(entry("notes-app", v2, 200));
    library.set_active("notes-app", v2).unwrap();

    library.remove(&v2);
    assert!(
        library.active("notes-app").is_none(),
        "active ref MUST clear when dropped CID was active"
    );
    // v1 still in the library.
    assert!(library.get(&v1).is_some());
}
