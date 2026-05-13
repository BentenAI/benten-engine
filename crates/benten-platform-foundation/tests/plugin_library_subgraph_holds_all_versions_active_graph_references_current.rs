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
