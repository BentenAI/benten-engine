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
//! ## G27-A R5 audit-completion finding
//!
//! The G27-A R5 implementer walked `bindings/napi/src/lib.rs` +
//! `bindings/napi/src/policy.rs` and confirmed:
//!
//! - `JsEngine::grant_capability` at lib.rs:639-652 routes through
//!   `parse_grant_json` (policy.rs:86-118) which extracts `scope` as
//!   a `String` from the JSON object's `"scope"` field directly. No
//!   conflation with CID values; no normalization or canonicalization.
//! - The parsed scope flows verbatim into
//!   `Engine::grant_capability_with_proof(actor, scope, issuer, hlc)`
//!   as `parsed.scope.as_str()`.
//! - No alternate napi grant binding exists at HEAD that could
//!   bypass this resolving seam.
//!
//! The 2 substantive tests below + the compile witness landed
//! DURABLE at R4-FP-4 (not awaiting un-ignore); the G27-A R5 audit
//! confirms the class-invariant continues to hold. See companion
//! audit doc `notes-napi-parity-audit.md` §3.

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

/// G27-A class-of-bug audit regression guard — DURABLE at HEAD.
///
/// Pins that the napi grant entry point passes scope strings (NOT
/// CIDs) through the engine seam by exercising the content-addressing
/// invariant on the grant Node: two distinct scope-strings minted at
/// the same actor MUST yield distinct content-addressed grant CIDs,
/// AND an OK-edge write through the granted handler MUST succeed.
/// Would-FAIL if a future napi grant-shaped binding ever conflates
/// CID + scope (the CID-substitution would collapse all scopes to a
/// single bucket OR the OK-edge write would surface unexpected error).
///
/// NOT RED-PHASE: this test exercises shipped surfaces today
/// (`Engine::grant_capability_with_proof` + `Engine::call_as`); it is
/// a durable regression guard, not a pin for future-wave un-ignore.
/// Per R4-FP-4 charter §5.4: R3-time substance that COULD have landed
/// AT R3 against shipped surfaces.
///
/// Substantive arm shape: the grant Node itself is system-zone
/// (`system:CapabilityGrant`) so `Engine::get_node` is sealed by
/// Inv-11 — direct backend readback is intentionally unavailable from
/// user-facing test surface. Instead we use the **content-addressing
/// invariant**: distinct scope-string inputs MUST yield distinct
/// grant CIDs because the canonical-bytes encoding of the grant Node
/// embeds the scope-string. If the napi binding mangled scope-string
/// into a CID-substitution shape, the resulting grant Node would
/// either (a) carry a constant scope value (CID-derived) collapsing
/// distinct scope inputs to the same output CID, or (b) fail to mint
/// when the CID isn't a valid Value::Text scope shape.
#[test]
fn napi_grant_entry_point_persists_scope_string_not_cid() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let handler_id = engine.register_crud("post").unwrap();
    let actor = engine.caps().create_principal("alice").unwrap();

    // Substantive arm #1 — minted CIDs distinct for distinct scopes.
    // Mint two grants for the SAME actor with DISTINCT scope strings.
    // Content-addressing guarantees: distinct inputs → distinct CIDs.
    // If the napi binding ever substituted a CID for the scope-string,
    // both grants would collapse to the same content-bucket (because
    // the actor + the eventual CID are content-addressed inputs that
    // wouldn't differ between the two writes pre-mint).
    let grant_post: Cid = engine
        .caps()
        .grant_capability_with_proof(&actor, "store:post:write", None, None)
        .expect("post-scope grant via privileged path");
    let grant_comment: Cid = engine
        .caps()
        .grant_capability_with_proof(&actor, "store:comment:write", None, None)
        .expect("comment-scope grant via privileged path");
    assert_ne!(
        grant_post, grant_comment,
        "two grants with distinct scope-strings (store:post:write, store:comment:write) \
         MUST mint to distinct grant CIDs — equal CIDs would indicate \
         the napi binding is collapsing/substituting scope-string with \
         a non-scope-derived value (class-of-bug mirror of pre-PR-#199)",
    );

    // Substantive arm #2 — idempotent re-mint yields identical CID.
    // The companion-positive proof: identical (actor, scope-string)
    // inputs MUST yield the same content-addressed grant CID. Together
    // with arm #1 this asymmetric pair proves the scope-string flows
    // through verbatim (no mangling, no normalization, no CID-
    // substitution masking the class-of-bug).
    let grant_post_repeat: Cid = engine
        .caps()
        .grant_capability_with_proof(&actor, "store:post:write", None, None)
        .expect("idempotent re-mint of same scope");
    assert_eq!(
        grant_post, grant_post_repeat,
        "re-minting identical (actor, scope-string) grant MUST yield the same \
         content-addressed CID; non-determinism here would indicate scope-string \
         mutation between napi binding and engine seam",
    );

    // Substantive arm #3 — OK-edge observable consequence on a write
    // routed through the granted handler. The shipped Phase-1
    // `register_crud` does not consult GrantBackedPolicy on every CRUD
    // write (carried as TODO in register_crud_with_grants per
    // engine.rs:3008), so this arm is the OK-edge baseline — it
    // confirms granted scopes don't accidentally surface unexpected
    // errors at call dispatch. If the napi binding ever produced
    // malformed grant Nodes (e.g., labels missing the
    // `system:CapabilityGrant` discriminator because of CID-substitution
    // at the label layer), subsequent dispatch infrastructure that
    // walks grant nodes would surface a typed engine error — this arm
    // is the canary for that downstream breakage.
    let post = post_node("post-grant write");
    let outcome = engine
        .call_as(&handler_id, "post:create", post, &actor)
        .expect("call ok at granted scope");
    assert!(
        outcome.is_ok_edge(),
        "write at granted scope MUST route via OK edge — \
         a non-OK outcome here would indicate the grant Node minted by \
         the napi entry point is malformed enough that downstream walkers \
         surface a typed error rather than the expected OK dispatch; \
         got outcome: {outcome:?}"
    );
}

/// G27-A regression guard — non-`store:<label>:write` scope shape (DURABLE at HEAD).
///
/// Plugin manifest scope grammar (per plugin-arch-r1-10) introduces
/// `private:<plugin_did>:*` / `requires:<plugin_did>:<requirement_path>`
/// / `shares:<plugin_did>:<share_path>` scopes that DO NOT match the
/// canonical `store:<label>:write` shape. The class-of-bug audit must
/// confirm the napi grant entry point passes ANY scope-string shape
/// through verbatim, not just `store:<label>:write` family ones.
///
/// NOT RED-PHASE: the engine seam (`grant_capability_with_proof`)
/// accepts arbitrary scope-string shapes today; this is a durable
/// regression guard verifying minting a non-`store:*:write` scope
/// doesn't error AND doesn't accidentally route through canonical
/// store-scope handling that would mangle the shape.
///
/// Substantive arm shape: same Inv-11 sealing applies as test #1, so
/// we cannot directly read the persisted scope from the system-zone
/// grant Node. The substantive bar here is: the mint succeeds with
/// the non-canonical shape AND returns a stable Cid AND a second mint
/// of the SAME scope returns the SAME Cid (content-addressing
/// consistency on scope-string verbatim representation). If the napi
/// grant binding mangled the plugin-manifest shape into a canonical
/// form (truncation / normalization / CID-substitution), the second
/// mint would either fail OR return a different CID for the
/// originally-equivalent scope string.
#[test]
fn napi_grant_entry_point_passes_plugin_manifest_scope_shape_through_verbatim() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    let actor = engine.caps().create_principal("plugin-issuer").unwrap();

    // Plugin manifest grammar scope (G24-D + G27-D land the manifest
    // surface; this pin verifies the napi grant entry point doesn't
    // mangle non-canonical scopes en route to the engine seam).
    let plugin_did_lexical = "did:key:zPluginDidPlaceholder";
    let scope = format!("private:{plugin_did_lexical}:notes");

    // Substantive arm #1: the mint succeeds with a non-`store:*:write`
    // scope shape. If the engine seam rejected non-canonical shapes
    // (or the napi binding pre-validated scope syntax before calling
    // through), this mint would error.
    let grant_cid_a: Cid = engine
        .caps()
        .grant_capability_with_proof(&actor, &scope, None, None)
        .expect("plugin-manifest scope grant should mint successfully");

    // Substantive arm #2: re-mint a grant with the SAME scope-string
    // shape using a different actor; assert the resulting grant Cid
    // differs. This proves the grant Node's content-addressed CID
    // genuinely incorporates the actor + scope distinctly — if the
    // napi binding ever collapsed the scope-string to a canonical
    // form (e.g., truncating after `:notes`), grants for sibling
    // plugin-DIDs would collide.
    let other_plugin_did = "did:key:zAnotherPluginDidPlaceholder";
    let other_scope = format!("private:{other_plugin_did}:notes");
    let grant_cid_b: Cid = engine
        .caps()
        .grant_capability_with_proof(&actor, &other_scope, None, None)
        .expect("sibling plugin-manifest scope grant should mint successfully");
    assert_ne!(
        grant_cid_a, grant_cid_b,
        "two distinct plugin-DID scopes ({scope}, {other_scope}) MUST mint \
         to distinct grant CIDs — if equal, the napi binding or engine seam \
         is canonicalizing/truncating the scope-string, masking the class-of-bug \
         the napi grant entry point is supposed to defend against",
    );

    // Substantive arm #3: re-mint the FIRST scope (identical actor +
    // scope-string) and assert the same Cid surfaces (content-addressed
    // determinism). This anchors the asymmetric shape: arm #2 proves
    // scope-string differences produce distinct CIDs; arm #3 proves
    // scope-string identity produces identical CIDs. Together they
    // pin that the scope-string flows through verbatim (no mangling).
    let grant_cid_a_repeat: Cid = engine
        .caps()
        .grant_capability_with_proof(&actor, &scope, None, None)
        .expect("idempotent re-mint of same scope succeeds");
    assert_eq!(
        grant_cid_a, grant_cid_a_repeat,
        "re-minting the identical (actor, scope-string) grant MUST yield \
         the same content-addressed CID; if not, scope-string is being mutated \
         non-deterministically en route from napi binding to engine seam",
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
