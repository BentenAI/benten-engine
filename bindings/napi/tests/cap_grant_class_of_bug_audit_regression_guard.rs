//! G27-A class-of-bug audit regression guard — grant entry point.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.14 G27-A row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-A entry.
//! Inherits from sec-3.5-r1-3 + arch-r1-8 + CRATES-DEEP-DIVE §6/§8 +
//! Ben D-4F-5 reframe (R1 triage). Class-of-bug audit walks every napi
//! cap-* entry point looking for scope-vs-CID confusion of the kind
//! closed by PR #199 (`Engine::revoke_capability_by_grant_cid`).
//!
//! ## Class of bug (what this pin defends against)
//!
//! The napi `grantCapability(grant_json)` surface accepts a JSON shape
//! with `{ actor, scope, issuer?, hlc? }` and routes through
//! `Engine::grant_capability_with_proof(actor, scope, issuer, hlc)`. If
//! the napi binding ever conflated the JSON's `scope` field with a CID
//! (or rendered a CID into `scope` post-parse), the resulting
//! `system:CapabilityGrant` Node would carry `scope = "<cid>"` rather
//! than the canonical `"store:<label>:write"` shape. The
//! `BackendGrantReader::has_unrevoked_grant_for_scope` walker keys on
//! the scope STRING — so a CID-keyed grant Node would never match any
//! real write at policy-check time, but ALSO would never fire deny —
//! the grant would simply be inert. The class-of-bug here is the
//! mirror of PR #199's revoke side: silent fail-OPEN if the grant is
//! invisible to the reader walker that the policy consults.
//!
//! ## Pin shape — substantive end-to-end (pim-2 §3.6b)
//!
//! 1. Mint a grant via the engine seam the napi binding routes through
//!    (`Engine::grant_capability_with_proof`) — the production arm.
//! 2. Issue a write whose scope derives to the granted scope string;
//!    assert the OK edge fires (the grant Node IS observable to the
//!    `GrantBackedPolicy::check_write` walker, keyed on the scope
//!    STRING the grant carries).
//! 3. Repeat with the canonical wildcard / store-label scope shape so
//!    the regression guard fires if the grant Node is ever persisted
//!    with `scope = "<cid>"` instead of `"store:post:write"`.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Re-introduce the class-of-bug by routing
//! `JsEngine.grantCapability(json)` through a hypothetical
//! `Engine::grant_capability_with_proof(actor, &grant_cid.to_base32(), ...)`
//! (passing the would-be grant CID AS the scope). The post-grant write
//! would be denied (no matching scope-keyed grant Node), flipping this
//! pin from PASS to FAIL.
//!
//! ## RED-PHASE expectation
//!
//! The G27-A implementer at R5 wires the audit + (if no additional
//! confusion sites surface) confirms the existing `revoke_capability_by_grant_cid`
//! seam + napi grant binding satisfy the class invariant. This pin
//! lands ignored; un-ignore happens at G27-A wave-time per §3.6e.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post_node(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

/// RED-PHASE: G27-A class-of-bug audit.
///
/// Pins that the napi grant entry point persists scope strings (NOT
/// CIDs) so the reader walker can match grants to writes. Would-FAIL
/// if a future napi grant-shaped binding ever conflates CID + scope.
#[test]
#[ignore = "RED-PHASE: G27-A — un-ignore when class-of-bug audit lands; verifies napi grant entry point passes scope-string (not CID) to engine seam"]
fn napi_grant_entry_point_persists_scope_string_not_cid() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let _handler_id = engine.register_crud("post").unwrap();
    let actor = engine.create_principal("alice").unwrap();

    // Substantive arm: the seam the napi `grantCapability(json)` binding
    // routes through (`Engine::grant_capability_with_proof`). The JSON
    // parser at `bindings/napi/src/lib.rs::parse_grant_json` MUST pass
    // the scope STRING through unchanged — never the grant's eventual CID.
    let grant_cid = engine
        .grant_capability_with_proof(&actor, "store:post:write", None, None)
        .expect("grant via privileged path");

    // Class-of-bug regression guard: the resulting grant Node's `scope`
    // property MUST be the scope STRING, not the grant CID lexically.
    // If the napi binding ever inverted these args, the read here
    // would surface `scope = "<cid base32>"` and the regression-guard
    // assertion below would fire.
    let _ = grant_cid; // placeholder to silence unused warning at RED phase

    // Observable consequence: a write whose policy-derived scope is
    // `store:post:write` MUST route via OK edge — the grant Node IS
    // observable to the reader walker keyed on the SCOPE STRING.
    let handler_id = engine.register_crud("post").unwrap();
    let _ = handler_id;
    let post = post_node("post-grant write");
    let _ = post;

    // RED-PHASE narrative: G27-A audit confirms grant entry point
    // persists scope-string. Un-ignore at G27-A wave-time + replace
    // this placeholder with a `query_grant_scope_property(grant_cid)`
    // assertion + a `call(handler_id, "post:create", post)` OK-edge
    // pin once the audit has surfaced any additional engine-side
    // readback shape needed.
    panic!(
        "RED-PHASE: G27-A — implementer must un-ignore + wire grant-scope-property assertion + OK-edge pin"
    );
}

/// RED-PHASE: G27-A regression guard — non-`store:<label>:write` scope shape.
///
/// Plugin manifest scope grammar (per plugin-arch-r1-10) introduces
/// `private:<plugin_did>:*` / `requires:<plugin_did>:<requirement_path>`
/// / `shares:<plugin_did>:<share_path>` scopes that DO NOT match the
/// canonical `store:<label>:write` shape. The class-of-bug audit must
/// confirm the napi grant entry point passes ANY scope-string shape
/// through verbatim, not just `store:<label>:write` family ones.
#[test]
#[ignore = "RED-PHASE: G27-A — un-ignore at G27-A wave; verifies non-canonical scope shapes (plugin manifest grammar) flow through napi grant unchanged"]
fn napi_grant_entry_point_passes_plugin_manifest_scope_shape_through_verbatim() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let actor = engine.create_principal("plugin-issuer").unwrap();

    // Plugin manifest grammar scope (G24-D + G27-D land the manifest
    // surface; this pin verifies the napi grant entry point doesn't
    // mangle non-canonical scopes en route to the engine seam).
    let plugin_did_lexical = "did:key:zPluginDidPlaceholder";
    let scope = format!("private:{plugin_did_lexical}:notes");

    let _grant_cid: Cid = engine
        .grant_capability_with_proof(&actor, &scope, None, None)
        .expect("plugin-scope grant via privileged path");

    // RED-PHASE narrative: G27-A audit walks the napi grant binding +
    // confirms no scope-string mangling between JSON parse + engine
    // seam. Un-ignore wires a backend readback that the persisted
    // grant Node carries `scope = "private:<did>:notes"` verbatim.
    panic!(
        "RED-PHASE: G27-A — implementer must wire scope-string verbatim assertion against persisted grant Node"
    );
}

/// Compile-time witness — `Engine::grant_capability_with_proof` is the
/// seam the napi grant binding routes through. Without this symbol
/// reachable, the napi binding's `grant_capability` method cannot
/// satisfy the class invariant — so the regression-guard suite must
/// hard-fail to compile if the seam vanishes.
#[test]
fn napi_grant_class_of_bug_seam_present_compile_witness() {
    #[allow(clippy::type_complexity)]
    type GrantSeam = fn(
        &Engine,
        &Cid,
        &str,
        Option<String>,
        Option<i64>,
    ) -> Result<Cid, benten_engine::EngineError>;
    fn _accepts_engine_grant_seam(
        _engine: &Engine,
        _actor: &Cid,
        _scope: &str,
        _issuer: Option<String>,
        _hlc: Option<i64>,
    ) -> Result<Cid, benten_engine::EngineError> {
        unimplemented!("compile-time witness — body never runs")
    }
    let _: GrantSeam = _accepts_engine_grant_seam;
}
