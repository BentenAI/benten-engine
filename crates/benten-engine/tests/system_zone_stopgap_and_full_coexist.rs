//! Edge-case tests: system-zone defence-in-depth — Phase-1 `E_SYSTEM_ZONE_WRITE`
//! storage stopgap co-exists with the Phase-2a `E_INV_SYSTEM_ZONE` Inv-11 path.
//!
//! R2 landscape §2.5.5 row "System-zone stopgap + full coexist".
//!
//! Plan §9.10 "defence in depth": Inv-11 (Phase 2a full form) lives at the
//! registration + runtime layers in `benten-eval` / `benten-engine`. The
//! storage-layer check (`E_SYSTEM_ZONE_WRITE`) from Phase 1 stays wired as a
//! belt-and-suspenders safety net. Both paths agree on which writes to deny.
//!
//! Concerns pinned:
//! - A user-zone-legal subgraph that writes a `system:*`-labelled Node fires
//!   `E_INV_SYSTEM_ZONE` at the Phase-2a path (registration-time).
//! - A bare storage-layer put bypassing the evaluator (the legacy path) still
//!   fires `E_SYSTEM_ZONE_WRITE` — the stopgap was NOT removed.
//! - Both codes exist in the catalog and have distinct string identifiers
//!   (they must not collapse to a single code).
//! - A user write that the Phase-2a check rejects is ALSO rejected at the
//!   storage layer (the two paths agree on the set of deniable writes —
//!   proves defence-in-depth).
//!
//! R3 red-phase contract: R5 (G5-B-i) lands Inv-11 at
//! registration + runtime. Tests compile; they fail because
//! `ErrorCode::InvSystemZone` does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes per R1 triage."
)]

use benten_eval::{ErrorCode, SubgraphBuilder};

fn subgraph_writes_system_labelled_node()
-> Result<benten_eval::Subgraph, benten_eval::RegistrationError> {
    let mut sb = SubgraphBuilder::new("user_writes_system_zone");
    let root = sb.read("input");
    let w = sb.write("system:internal:forbidden");
    sb.add_edge(root, w);
    sb.respond(w);
    sb.build_validated()
}

#[test]
fn phase_2a_inv_11_fires_at_registration_for_system_zone_write() {
    let err = subgraph_writes_system_labelled_node()
        .expect_err("user-subgraph writing system:* must fail Inv-11 at registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvSystemZone,
        "Phase-2a path must fire E_INV_SYSTEM_ZONE, got {:?}",
        err.code()
    );
}

#[test]
fn phase_1_storage_stopgap_still_fires_for_direct_backend_put() {
    // Bypass evaluator → go straight to RedbBackend.put_node with a Node
    // whose label is `system:*`. The Phase-1 stopgap at the storage layer
    // must still deny with E_SYSTEM_ZONE_WRITE.
    use benten_core::{Node, Value};
    use benten_graph::{NodeStore, RedbBackend, WriteAuthority, WriteContext};
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("sz.redb")).unwrap();

    let mut props = BTreeMap::new();
    props.insert("k".into(), Value::text("v"));
    let node = Node::new(vec!["system:internal:forbidden".into()], props);

    let ctx = WriteContext {
        label: "system:internal:forbidden".into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    };

    let err = backend
        .put_node_with_context(&node, &ctx)
        .expect_err("direct-put of system:* Node must be denied by storage stopgap");

    assert_eq!(
        err.code(),
        ErrorCode::SystemZoneWrite,
        "storage stopgap must still fire E_SYSTEM_ZONE_WRITE, got {:?}",
        err.code()
    );
}

#[test]
fn system_zone_codes_are_distinct_in_catalog() {
    // Catalog hygiene: the stopgap code and the Phase-2a invariant code must
    // not collapse to a single identifier.
    assert_ne!(
        ErrorCode::SystemZoneWrite.as_str(),
        ErrorCode::InvSystemZone.as_str(),
        "E_SYSTEM_ZONE_WRITE (stopgap) and E_INV_SYSTEM_ZONE (Inv-11) must differ"
    );
    assert_eq!(ErrorCode::SystemZoneWrite.as_str(), "E_SYSTEM_ZONE_WRITE");
    assert_eq!(ErrorCode::InvSystemZone.as_str(), "E_INV_SYSTEM_ZONE");
}

#[test]
fn both_paths_agree_on_deniable_set() {
    // Every system:* write the Phase-2a check rejects must also be rejected
    // by the storage stopgap (and vice versa for the tested label).
    let label = "system:audit:never_writable";

    // Phase-2a path.
    let err_phase_2a = {
        let mut sb = SubgraphBuilder::new("agreement_pin");
        let r = sb.read("x");
        let w = sb.write(label);
        sb.add_edge(r, w);
        sb.respond(w);
        sb.build_validated()
            .expect_err("registration must deny")
            .code()
    };
    assert_eq!(err_phase_2a, ErrorCode::InvSystemZone);

    // Storage-layer path.
    use benten_core::{Node, Value};
    use benten_graph::{NodeStore, RedbBackend, WriteAuthority, WriteContext};
    use std::collections::BTreeMap;
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("agree.redb")).unwrap();
    let mut props = BTreeMap::new();
    props.insert("k".into(), Value::text("v"));
    let node = Node::new(vec![label.into()], props);
    let ctx = WriteContext {
        label: label.into(),
        authority: WriteAuthority::User,
        ..WriteContext::default()
    };
    let err_storage = backend
        .put_node_with_context(&node, &ctx)
        .expect_err("storage must deny")
        .code();
    assert_eq!(err_storage, ErrorCode::SystemZoneWrite);
}
