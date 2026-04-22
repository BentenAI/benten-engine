//! Phase 2a R3 security — Inv-11 user READ of `system:*` Node denied.
//!
//! **Attack class (sec-r1-3).** Phase 1 closed the WRITE surface
//! (`E_SYSTEM_ZONE_WRITE`) but left the READ side open: a user-authored
//! call that READs a CID resolving to a `system:CapabilityGrant` label
//! bypasses the capability-data privacy expectation. Phase 2a's Inv-11 full
//! enforcement closes both directions.
//!
//! **Prerequisite.** Attacker knows (or can enumerate) the CID of a system-
//! zone Node — for instance, a `system:CapabilityGrant` carrying another
//! principal's scope. Phase-1 `diagnose_read` + `create_principal` +
//! `grant_capability` already expose sufficient primitives to obtain a
//! known system-zone CID in-process.
//!
//! **Attack sequence.**
//!  1. Engine-privileged path creates a `system:CapabilityGrant` Node via
//!     `engine.grant_capability(alice, "store:post:write")`.
//!  2. Adversary fetches the grant Node via `engine.get_node(&cid)` on the
//!     user API path (NOT the privileged path).
//!  3. Without Inv-11 READ enforcement the privileged content is returned.
//!
//! **Impact.** Privileged capability-data readable without holding a
//! matching read grant. Compromise #2 (Option C) returns `None` symmetric
//! with not-found, but the Node IS returned on any read-allowing policy.
//!
//! **Recommended mitigation.** The READ surface (and every content-
//! returning `PrimitiveHost` method — see `option_c_flanking_methods_*`)
//! consults `is_system_zone_prefix(resolved.label)` via the engine-side
//! runtime probe (§9.10). Under a user caller, reach into `system:*` →
//! return `Ok(None)` (Option C symmetric) **even when the read policy
//! would otherwise permit** — the engine-side Inv-11 gate is stricter than
//! the user-configurable cap policy.
//!
//! **Red-phase contract.** Today (Phase 1 HEAD) `get_node(grant_cid)`
//! returns `Some(grant_node)` to any caller whose policy permits
//! (NoAuthBackend default: always permits). This test asserts the
//! mitigation-side `Ok(None)` — fails today because the Node is returned.
//! R5 G5-B-i implements runtime probe, test passes.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// atk / sec-r1-3 adversarial: the user `get_node` surface must NOT return
/// the content of a `system:*`-labeled Node even when the configured
/// capability policy permits the read. Inv-11 is an engine-side invariant,
/// stricter than the pluggable cap policy.
#[test]
fn inv_11_read_from_user_code_to_system_zone_denied() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Step 1: engine-privileged path mints a system-zone grant Node.
    let alice = engine.create_principal("alice").unwrap();
    let grant_cid = engine
        .grant_capability(&alice, "store:post:write")
        .expect("engine-privileged grant succeeds");

    // Sanity: the Node exists in the backend and carries a `system:*` label
    // (via a privileged back-channel read — NOT the user-facing probe).
    // Directly inspect via the backend-bypass path `diagnose_read` if
    // available, or just document the assumption.
    //
    // For the red-phase assertion we rely on the ATTACK: call get_node
    // (user API) and expect Inv-11 to deny.

    // Step 2: adversary invokes the user-facing get_node.
    let observed = engine
        .get_node(&grant_cid)
        .expect("get_node returns Ok; denial is via symmetric-None not Err");

    // MITIGATION: under Inv-11 + Option-C symmetric-None, reading a
    // system-labeled Node from the user API returns `None` (leak-None
    // symmetric with a backend miss) INDEPENDENTLY of the capability
    // policy's verdict. Today (no Inv-11 runtime probe) the NoAuthBackend
    // default permits the read so `Some(grant_node)` is returned.
    assert!(
        observed.is_none(),
        "Inv-11 adversarial: user-facing get_node on a system-zone CID \
         MUST return None (Option C symmetric-with-not-found). Phase-1 \
         HEAD returns Some(grant_node) because no engine-side Inv-11 \
         runtime probe fires on READ. Got: {observed:?}"
    );
}
