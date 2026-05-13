//! G24-D row pin — plugin library subgraph + active references.
//!
//! Per CLAUDE.md #18 + D-4F-14: user's full plugin set lives as a
//! "plugin library" subgraph containing all installed versions +
//! forks. Active graph holds references to specific plugin-versions
//! currently in use. Switching active version = updating the
//! reference. Keeps old/unused versions in the library (cheap, content-
//! addressed).

mod common;

use common::manifest_fixtures::{stub_cid_one, stub_cid_two};

#[test]
#[ignore = "RED-PHASE: G24-D wave wires plugin_library subgraph; un-ignore at G24-D landing"]
fn plugin_library_subgraph_holds_all_installed_versions_active_ref_points_at_current() {
    let v1 = stub_cid_one();
    let v2 = stub_cid_two();

    // Future surface:
    //   plugin_library::install(plugin_did, manifest_cid)
    //   plugin_library::set_active(plugin_did, version_cid)
    //   plugin_library::list_versions(plugin_did) -> Vec<Cid>
    //   plugin_library::active(plugin_did) -> Cid
    //
    // SUBSTANTIVE assertion: after installing v1 then v2, list_versions
    // returns BOTH; set_active(v2) doesn't drop v1. FAILS-IF-NO-OP
    // because a naive impl that overwrites would only retain v2.
    panic!("RED-PHASE: G24-D wave must wire plugin_library with all-versions-retained invariant");
}
