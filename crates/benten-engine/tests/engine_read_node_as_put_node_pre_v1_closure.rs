//! Pre-v1 Class-B β closure pins for `docs/future/phase-3-backlog.md
//! §13.7` — the 4 `todo!()` stubs at
//! `crates/benten-engine/src/engine_wait.rs:1011-1311` are gone.
//!
//! Each test below is a SUBSTANTIVE arm-exercising pin per pim-2 §3.6b:
//! it exercises the production surface end-to-end (not just the
//! engine-internal seam) AND would FAIL if the implementation were
//! no-op'd or reverted to `todo!()`.
//!
//! 1. `put_node_writes_node_observable_via_get_node` — `Engine::put_node`
//!    actually persists a Node (round-trip via `Engine::get_node`).
//! 2. `read_node_as_returns_node_under_noauth_policy` — `read_node_as`
//!    returns the resolved Node when policy permits.
//! 3. `read_node_as_collapses_to_none_under_inv_11_system_zone` — the
//!    Inv-11 system-zone probe fires regardless of principal.
//! 4. `read_node_as_collapses_to_none_under_grant_backed_denial` — the
//!    Option-C symmetric-None contract: a caller without the read
//!    grant sees `Ok(None)`, indistinguishable from miss.
//! 5. `read_node_as_threads_principal_into_read_context` — the
//!    `actor_cid` field reaches the policy via a custom probe policy.
//! 6. `grant_read_capability_for_testing_permits_read_after_grant` —
//!    the test-helpers grant installer flips a denied read to
//!    permitted (end-to-end through `read_node_as` + grant-backed
//!    policy).

#![cfg(any(test, feature = "test-helpers"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use benten_caps::{CapError, CapabilityPolicy, ReadContext, WriteContext};
use benten_core::{Cid, Node, Value};
use benten_engine::Engine;

fn make_post(title: &str) -> Node {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

fn fresh_engine_grant_backed() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();
    (dir, engine)
}

// ---------------------------------------------------------------------------
// 1. Engine::put_node — production round-trip.
// ---------------------------------------------------------------------------

/// §13.7 (a) closure: `put_node` must actually persist the Node so a
/// subsequent `get_node` returns the same labels + properties. Pre-PR
/// state was `todo!("Phase 2a G2-A: implement engine.put_node")` —
/// this test would panic on first invocation.
#[test]
fn put_node_writes_node_observable_via_get_node() {
    let (_dir, engine) = fresh_engine();
    let node = make_post("put_node_round_trip_fixture");
    let cid = engine.put_node(&node).expect("put_node must succeed");

    let read_back = engine
        .get_node(&cid)
        .expect("get_node infallible")
        .expect("Node must be present after put_node");
    assert_eq!(read_back.labels, vec!["post".to_string()]);
    assert_eq!(
        read_back.properties.get("title"),
        Some(&Value::Text("put_node_round_trip_fixture".into())),
        "put_node must persist properties verbatim"
    );
}

// ---------------------------------------------------------------------------
// 2. Engine::read_node_as — happy path under NoAuth.
// ---------------------------------------------------------------------------

/// §13.7 (b) closure baseline: `read_node_as` returns the resolved
/// Node when the policy permits (default `NoAuthBackend` always
/// permits). Pre-PR state was `todo!("Phase 2a G4-A: Option C
/// flanking-method plumbing per sec-r1-5")`.
#[test]
fn read_node_as_returns_node_under_noauth_policy() {
    let (_dir, engine) = fresh_engine();
    let node = make_post("read_node_as_happy_path");
    let cid = engine.put_node(&node).expect("seed");
    let principal = engine
        .create_principal("test-principal-happy")
        .expect("seed principal");

    let got = engine
        .read_node_as(&principal, &cid)
        .expect("read_node_as infallible under NoAuth permit")
        .expect("NoAuth must permit so the Node is returned");
    assert_eq!(got.labels, vec!["post".to_string()]);
}

// ---------------------------------------------------------------------------
// 3. Inv-11 system-zone probe fires regardless of principal.
// ---------------------------------------------------------------------------

/// §13.7 (b) closure — Inv-11 enforcement: a Node whose primary label
/// lives in a system-zone prefix (e.g. `system:Principal`) MUST
/// collapse to `Ok(None)` from `read_node_as` regardless of the
/// principal, because Inv-11 is engine-side and stricter than the
/// cap policy.
///
/// Would-FAIL-if-no-op'd: if `read_node_as` skipped the
/// `is_system_zone_label` probe, a caller with a valid principal
/// would observe the `system:Principal` Node directly. The default
/// `NoAuthBackend` permits every read, so the only thing blocking
/// the leak is the engine-side Inv-11 probe.
#[test]
fn read_node_as_collapses_to_none_under_inv_11_system_zone() {
    let (_dir, engine) = fresh_engine();
    // `create_principal` writes a `system:Principal`-labeled Node
    // via the privileged path; the resulting CID is our probe target.
    let principal_cid = engine.create_principal("inv11-probe-target").expect("seed");
    // Read attributed to a different principal — Inv-11 fires
    // regardless.
    let alice = engine
        .create_principal("alice-reader")
        .expect("seed reader");

    let got = engine
        .read_node_as(&alice, &principal_cid)
        .expect("read_node_as infallible — Inv-11 collapses to None");
    assert!(
        got.is_none(),
        "Inv-11: read_node_as must NOT return a system-zone Node \
         regardless of principal; got {got:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. Option-C symmetric-None under grant-backed denial.
// ---------------------------------------------------------------------------

/// §13.7 (b) closure — Option C named compromise #2: a caller
/// without the `store:<label>:read` grant must see `Ok(None)` (NOT
/// an error), indistinguishable from a missing-CID miss.
///
/// Would-FAIL-if-no-op'd: skipping the `policy.check_read` call (or
/// returning `Ok(Some(...))` on `CapError::DeniedRead` instead of
/// `Ok(None)`) would leak the Node body to an unauthorised reader.
#[test]
fn read_node_as_collapses_to_none_under_grant_backed_denial() {
    let (_dir, engine) = fresh_engine_grant_backed();
    let node = make_post("denied_read_fixture");
    let cid = engine.put_node(&node).expect("seed");
    // No `store:post:read` grant for `alice` — denial expected.
    let alice = engine
        .create_principal("alice-no-read-grant")
        .expect("seed principal");

    let got = engine
        .read_node_as(&alice, &cid)
        .expect("Option C: denial collapses to Ok(None), not an error");
    assert!(
        got.is_none(),
        "grant-backed policy denies + read_node_as must collapse to None; \
         got {got:?}"
    );
}

// ---------------------------------------------------------------------------
// 5. Principal threading reaches the policy via actor_cid.
// ---------------------------------------------------------------------------

/// Probe policy that records the `actor_cid` it observed on every
/// `check_read` invocation. Used to pin the principal-threading
/// contract end-to-end.
#[derive(Debug, Default)]
struct RecordingPolicy {
    seen_actor: Arc<Mutex<Option<Cid>>>,
}

impl CapabilityPolicy for RecordingPolicy {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_read(&self, ctx: &ReadContext) -> Result<(), CapError> {
        if let Ok(mut slot) = self.seen_actor.lock() {
            *slot = ctx.actor_cid;
        }
        Ok(())
    }
}

/// §13.7 (b) closure — load-bearing principal threading contract:
/// the `ReadContext::actor_cid` field handed to `check_read` MUST be
/// `Some(*principal)` (NOT `None` like `Engine::get_node`'s path).
///
/// Would-FAIL-if-no-op'd: if `read_node_as` constructed the
/// `ReadContext` with `..Default::default()` (omitting the explicit
/// `actor_cid: Some(*principal)`), the recorded slot would be `None`
/// and the assertion would fire. This is the architectural
/// differentiator from `Engine::get_node` per CLAUDE.md baked-in #18.
#[test]
fn read_node_as_threads_principal_into_read_context() {
    let dir = tempfile::tempdir().unwrap();
    let policy = RecordingPolicy::default();
    let observed = Arc::clone(&policy.seen_actor);
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();
    let node = make_post("principal_thread_fixture");
    let cid = engine.put_node(&node).expect("seed");
    let alice = engine
        .create_principal("alice-principal-probe")
        .expect("seed");

    let _ = engine
        .read_node_as(&alice, &cid)
        .expect("read_node_as succeeds; we only care about the recorded actor_cid");

    let seen = observed.lock().unwrap();
    assert_eq!(
        *seen,
        Some(alice),
        "read_node_as MUST thread the caller's principal CID through \
         ReadContext::actor_cid; got {:?}",
        *seen
    );
}

// ---------------------------------------------------------------------------
// 6. grant_read_capability_for_testing flips denial to permit.
// ---------------------------------------------------------------------------

/// §13.7 (c) closure — the test-helpers grant installer must
/// actually flip a denied read to a permitted one when consumed
/// end-to-end through `read_node_as` against a `GrantBackedPolicy`.
///
/// Would-FAIL-if-no-op'd: pre-PR the function was
/// `todo!("Phase 2a G4-A: test-only read-grant path")` — calling it
/// would panic. A no-op implementation that returned a dummy CID
/// without writing a grant Node would still leave the read denied,
/// so the post-grant assertion would fire.
#[test]
fn grant_read_capability_for_testing_permits_read_after_grant() {
    let (_dir, engine) = fresh_engine_grant_backed();
    let node = make_post("grant_helper_fixture");
    let cid = engine.put_node(&node).expect("seed");

    // Before the grant: a fresh reader sees None (denied collapses
    // to None per Option C).
    let reader = engine
        .create_principal("grant-helper-reader-before")
        .expect("seed");
    let pre = engine
        .read_node_as(&reader, &cid)
        .expect("read_node_as infallible");
    assert!(
        pre.is_none(),
        "pre-grant: read must be denied (collapsed to None); got {pre:?}"
    );

    // Install the grant via the test-helpers surface.
    let grant_cid = engine
        .grant_read_capability_for_testing(&cid)
        .expect("grant install must succeed");
    // The minted grant CID must differ from the target's CID — a
    // no-op stub returning the input CID would fail this pin.
    assert_ne!(
        grant_cid, cid,
        "grant installer must mint a fresh system:CapabilityGrant CID, \
         not echo the target CID"
    );

    // After the grant: any reader principal observes the Node (the
    // grant-backed policy permits any unrevoked `store:post:read`
    // grant regardless of the specific actor CID).
    let reader_after = engine
        .create_principal("grant-helper-reader-after")
        .expect("seed");
    let post = engine
        .read_node_as(&reader_after, &cid)
        .expect("read_node_as infallible")
        .expect("post-grant: read must succeed under permitted grant");
    assert_eq!(
        post.properties.get("title"),
        Some(&Value::Text("grant_helper_fixture".into())),
        "post-grant Node must round-trip through read_node_as verbatim"
    );
}
