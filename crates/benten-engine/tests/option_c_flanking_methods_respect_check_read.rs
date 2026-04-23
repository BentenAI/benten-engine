//! Phase 2a R3 security — Option C flanking-method bypass (atk-5 / sec-r1-5).
//!
//! R4 qa-r4-10 cross-reference: R2 §4.4 lists these under
//! `crates/benten-engine/tests/integration/option_c_flanking_methods.rs`.
//! Phase-2a keeps the per-scenario file split; this header names the R2
//! anchor so auditors can cross-walk between the landscape and the code.
//!
//! **Attack class.** Phase 1 Compromise #2 Option C gates the READ primitive
//! on `PrimitiveHost::check_read_capability` via `Engine::get_node`. But the
//! engine exposes THREE sibling content-returning methods on `PrimitiveHost`
//! that a TRANSFORM-driven handler can reach — `get_by_label`,
//! `get_by_property`, `read_view` — and none of them consult
//! `check_read_capability`. Exit gate 4 passes ("crud:post:get symmetric-
//! None") while a TRANSFORM flanks through `read_view(...)` or
//! `get_by_label(...)` and returns the very Nodes the READ primitive denies.
//!
//! **Prerequisite (attacker capability).** Attacker can author a subgraph
//! containing TRANSFORM nodes whose expressions call the host's non-READ
//! content accessors. In the Phase-1 DSL this is reachable via any handler
//! that names a non-READ primitive whose evaluator path hands off to
//! `PrimitiveHost::get_by_label` / `get_by_property` / `read_view`.
//!
//! **Attack sequence (per-method).**
//!  1. Configure a capability policy that DENIES reads for `post` label.
//!  2. Create a `post` Node via the engine-privileged path.
//!  3. Invoke the flanking method directly (`engine.backend().get_by_label`
//!     bypasses the Phase-1 policy entirely; `Engine::<PrimitiveHost>::
//!     get_by_label` does not consult `check_read_capability`).
//!  4. Assert the method returns an empty list (Option C symmetric-None:
//!     under a denying policy the flanking method returns the same shape as
//!     "no matching Nodes") rather than the populated list.
//!
//! **Impact.** Gate 4 headline `crud:post:get_symmetric_none` claim is
//! vacuous — TRANSFORM flanks around the gate.
//!
//! **Recommended mitigation.** G4-A threads `check_read_capability` into
//! every content-returning `PrimitiveHost` method. For `read_view`:
//! documented as named Compromise (IVM views are coarse-grained; per-row
//! gating is Phase-3 scope).
//!
//! **Red-phase contract.** Today, `PrimitiveHost::get_by_label` on an Engine
//! that holds a deny-reads policy returns the populated CID list — the
//! policy is never consulted on this path. The tests below invoke the
//! PrimitiveHost trait methods directly and assert empty-list outcomes. R5
//! G4-A threads the cap check, tests pass.
//!
//! Three tests: `transform_via_read_view_respects_check_read`,
//! `transform_via_get_by_label_respects_check_read`,
//! `transform_via_get_by_property_respects_check_read`.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapabilityPolicy, ReadContext, WriteContext};
use benten_core::{Node, Value};
use benten_engine::Engine;
use benten_eval::host::{PrimitiveHost, ViewQuery};
use std::collections::BTreeMap;

/// Policy that denies every read (routes to `CapError::DeniedRead`) and
/// permits every write. Models a principal holding write-only grants.
#[derive(Debug)]
struct DenyAllReads;

impl CapabilityPolicy for DenyAllReads {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Err(CapError::DeniedRead {
            required: "test:read".to_string(),
            entity: "test-entity".to_string(),
        })
    }
}

fn engine_with_deny_reads() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(DenyAllReads))
        .build()
        .unwrap();
    (engine, dir)
}

/// atk-5 flank #1: `PrimitiveHost::get_by_label`.
///
/// A TRANSFORM expression that calls the host's `get_by_label("post")` on an
/// Engine whose policy denies reads must return an empty Vec (Option C
/// symmetric-None), NOT the populated CID list. Today the method passes
/// straight through to `self.backend().get_by_label(...)` without consulting
/// the policy — the test fails.
#[test]
fn transform_via_get_by_label_respects_check_read() {
    let (engine, _dir) = engine_with_deny_reads();

    // Seed a Node via the engine-privileged path (ChangeEvent fans out to
    // indexes so `get_by_label` can find it).
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);
    let _cid = engine.create_node(&node).expect("privileged write");

    // The attack: TRANSFORM expression uses the host accessor.
    let observed = <Engine as PrimitiveHost>::get_by_label(&engine, "post")
        .expect("get_by_label must not surface EvalError under deny-read");

    assert!(
        observed.is_empty(),
        "Option C flanking (atk-5): `PrimitiveHost::get_by_label(\"post\")` \
         under a deny-reads policy MUST return an empty Vec (Option C \
         symmetric-with-not-found). Phase-1 HEAD returns {} CIDs because \
         the method passes straight through `self.backend().get_by_label` \
         without consulting `check_read_capability`. Got: {observed:?}",
        observed.len()
    );
}

/// atk-5 flank #2: `PrimitiveHost::get_by_property`.
#[test]
fn transform_via_get_by_property_respects_check_read() {
    let (engine, _dir) = engine_with_deny_reads();

    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);
    let _cid = engine.create_node(&node).expect("privileged write");

    let observed = <Engine as PrimitiveHost>::get_by_property(
        &engine,
        "post",
        "title",
        &Value::Text("hello".into()),
    )
    .expect("get_by_property must not surface EvalError under deny-read");

    assert!(
        observed.is_empty(),
        "Option C flanking (atk-5): `PrimitiveHost::get_by_property(\"post\", \
         \"title\", ...)` under a deny-reads policy MUST return an empty Vec \
         (Option C symmetric). Phase-1 HEAD returns {} CIDs. Got: {observed:?}",
        observed.len()
    );
}

/// atk-5 flank #3: `PrimitiveHost::read_view`.
///
/// IVM views are coarse-grained in 2a (named Compromise #N+2 per plan
/// §3 G4-A), so the denial shape for `read_view` under a deny-reads policy
/// is "return an empty Outcome list" rather than per-row filtering. The
/// flank here is: a TRANSFORM calls `read_view("posts")` on a view the
/// policy does not authorise and receives the populated list.
#[test]
fn transform_via_read_view_respects_check_read() {
    let (engine, _dir) = engine_with_deny_reads();

    // Once G4-A threads the cap check, invoking the flanking host method
    // under a deny-reads policy must return an empty Outcome list (Option C
    // symmetric-with-not-found). Today the method is a Phase-1 stub that
    // does not consult the policy.
    //
    // Call the trait method via its `view_id` argument (the view registry
    // is empty in this fixture; the assertion is that the DENIAL shape is
    // observed — not that a populated view exists).
    // Seed a Node so a hypothetical IVM view would include it.
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);
    let _cid = engine.create_node(&node).expect("privileged write");

    let query = ViewQuery {
        label: Some("post".to_string()),
        ..Default::default()
    };
    let observed = <Engine as PrimitiveHost>::read_view(&engine, "posts", &query);

    // Phase-2a mitigation: read_view under deny-reads returns an empty/Ok
    // shape (NOT Err with populated rows — that leaks existence). Today's
    // Phase-1 stub routes through backend without consulting the policy.
    match observed {
        Ok(value) => {
            // If the return shape is a Value::List (the IVM view row
            // envelope), assert it is empty. Any non-empty list under a
            // deny-reads policy is a flanking leak.
            if let Value::List(items) = &value {
                assert!(
                    items.is_empty(),
                    "read_view under deny-reads MUST yield an empty list \
                     (coarse-grained cap gate per Compromise #N+2). Phase-1 \
                     HEAD returns the populated list because the method \
                     does not consult check_read_capability. Got: \
                     {items:?}"
                );
            }
        }
        Err(e) => panic!(
            "read_view must not surface EvalError under a deny-reads policy \
             — that leaks existence. Got: {e:?}"
        ),
    }
}
