//! COLLAPSE (P5) — ONE generalized envelope-ceiling (device +
//! plugin-manifest) + #1241/F2 cap-predicate completion + F3 durable
//! replay-marker. The MANDATORY §3.6b security closure-pins (pim-2 /
//! pim-18 SHAPE-not-SUBSTANCE — would-FAIL-if-reverted, mutation-
//! reasoned).
//!
//! # Charter
//!
//! Spec: `.addl/refinement-audit-2026-05/impl-design-COLLAPSE.md` §7
//! (P5) + `DECISION-RECORD-trust-model-reframe.md` §4 build-constraint
//! iii ("J8-caveat + #669-ceiling-check are ONE code path") + §4b
//! (F2 RATIFIED (a)-zone-scoped-for-v1 + #1241 cap-predicate
//! completion lands WITH P5 — ONE mechanism; F3 durable-replay-marker
//! re-home tracked P2/P5).
//!
//! P5 extends the single P2 seam so the #669 plugin-manifest ceiling
//! is a SECOND caller of the ONE factored predicate
//! [`benten_caps::envelope_ceiling_rejects_cap`] — NOT a parallel
//! pipe. These pins assert the three load-bearing properties survive
//! and are real-production-arm (not synthetic-scope) per pim-18.
//!
//! # The three §3.6b closure-pins (mutation-reasoned)
//!
//! - **(a) F2-substantive arm (#1241):** a `runs_sandbox=false`
//!   principal self-delegating `host:sandbox:exec` is rejected by the
//!   cap-resource predicate using the REAL cap.resource string (the
//!   production arm the F2 finding named — NOT a synthetic
//!   `{zone}:write` scope). MUTATION: if the predicate reverted to
//!   discriminating on a zone-write scope, the real
//!   `host:sandbox:exec` resource would pass → this FAILs.
//! - **(b) plugin-manifest ceiling violation rejected through the
//!   SAME seam:** a plugin whose signed manifest `requires` never
//!   declared `host:sandbox:*` is, for the CLAUDE.md #17 thin-shape
//!   ceiling, a `runs_sandbox=false` principal — and a chain link it
//!   issued exercising `host:sandbox:exec` rejects via the SAME
//!   `envelope_ceiling_rejects_cap` core. MUTATION: if P5 had built a
//!   parallel manifest pipe that did not consult the shared predicate
//!   (or `manifest_to_envelope_ceiling` mis-derived `runs_sandbox`),
//!   this FAILs.
//! - **(c) durable-replay-marker rejects a replayed stale frame
//!   (F3):** the second presentation of an already-observed frame
//!   nonce is rejected by the durable [`benten_caps::FrameReplayMarker`]
//!   even though the first was admitted. MUTATION: if the marker did
//!   not persist (no-op `put`, or `mark_and_check_frame` always
//!   returned `false`), the replay would be admitted → this FAILs.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_caps::chain_authority::{
    FrameReplayMarker, envelope_ceiling_first_rejected_resource, envelope_ceiling_rejects_cap,
    manifest_to_envelope_ceiling, validate_chain_with_manifest_ceiling,
};
use benten_id::device_attestation::CapabilityEnvelope;
use benten_id::did::Did;
use benten_id::errors::UcanError;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;
use benten_platform_foundation::{CapRequirement, PluginManifest, SharesPolicy};

fn manifest_with_requires(requires: Vec<&str>) -> PluginManifest {
    PluginManifest {
        plugin_name: "p5-pin".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([7u8; 32]),
        peer_did: Did::from_string_unchecked("did:key:z6MkP5Author".to_string()),
        peer_signature: vec![0u8; 64],
        requires: requires
            .into_iter()
            .map(|s| CapRequirement {
                scope: s.to_string(),
            })
            .collect(),
        shares: SharesPolicy::none(),
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// §3.6b closure-pin (a) — the **#1241 / F2 cap-predicate-complete**
/// arm. The REAL production cap.resource string (`host:sandbox:exec`),
/// NOT a synthetic `{zone}:write` scope, must be rejected for a
/// `runs_sandbox=false` principal. This is the pim-18
/// SHAPE-not-SUBSTANCE requirement: the F2 finding was that the
/// engine-side seam discriminated on `format!("{zone}:write")` (which
/// can never `starts_with("host:sandbox:")`) — the completion is that
/// the ONE shared predicate is fed the writer's ACTUAL cap.resource.
#[test]
fn collapse_p5_f2_cap_predicate_rejects_real_sandbox_resource_from_thin_principal() {
    let thin = CapabilityEnvelope {
        runs_sandbox: false,
        ..CapabilityEnvelope::default()
    };
    let full = CapabilityEnvelope {
        runs_sandbox: true,
        ..CapabilityEnvelope::default()
    };

    // The REAL production cap.resource (the F2-substantive arm) — NOT
    // a "{zone}:write" synthetic scope. If a regression reverted the
    // predicate to zone-scope discrimination, `host:sandbox:exec`
    // would pass and this FAILs (pim-2 would-FAIL-if-reverted).
    let offending = envelope_ceiling_first_rejected_resource(
        Some(&thin),
        ["store:notes:write", "host:sandbox:exec"],
    );
    assert_eq!(
        offending.as_deref(),
        Some("host:sandbox:exec"),
        "F2 #1241: a runs_sandbox=false principal self-delegating the \
         REAL host:sandbox:exec cap.resource MUST be rejected via the \
         cap-resource predicate (the production arm; NOT the synthetic \
         {{zone}}:write scope the F2 finding flagged as inert)"
    );

    // No-over-rejection: a full peer, and a non-sandbox resource, pass.
    assert!(
        envelope_ceiling_first_rejected_resource(
            Some(&full),
            ["host:sandbox:exec", "store:notes:write"]
        )
        .is_none(),
        "runs_sandbox=true principal may exercise host:sandbox:* (no \
         over-rejection)"
    );
    assert!(
        envelope_ceiling_first_rejected_resource(Some(&thin), ["store:notes:write"]).is_none(),
        "a thin principal's ordinary data-zone cap is NOT gated by the \
         runs_sandbox ceiling"
    );
    // None ceiling = nothing to AND (legacy / non-wire path).
    assert!(
        envelope_ceiling_first_rejected_resource(None, ["host:sandbox:exec"]).is_none(),
        "no verified ceiling => no rejection (legacy unsigned envelope)"
    );
}

/// §3.6b closure-pin (b) — the #669 plugin-manifest ceiling is
/// enforced through the **SAME** `envelope_ceiling_rejects_cap` core
/// as the device-envelope path (build-constraint iii: ONE code path,
/// two callers). A plugin whose signed, user-consented manifest
/// `requires` never declared `host:sandbox:*` is — for the CLAUDE.md
/// #17 thin-shape ceiling — a `runs_sandbox=false` principal; a chain
/// link IT issued exercising `host:sandbox:exec` rejects.
#[test]
fn collapse_p5_manifest_ceiling_rejects_sandbox_via_shared_predicate() {
    // The manifest→ceiling adapter routes through the ONE predicate.
    let no_sandbox_manifest = manifest_with_requires(vec!["store:notes:write"]);
    let sandbox_manifest = manifest_with_requires(vec!["host:sandbox:exec"]);

    let ceiling_no = manifest_to_envelope_ceiling(&no_sandbox_manifest);
    let ceiling_yes = manifest_to_envelope_ceiling(&sandbox_manifest);

    assert!(
        !ceiling_no.runs_sandbox,
        "a manifest that did NOT declare host:sandbox:* in `requires` \
         derives a runs_sandbox=false ceiling (the user never consented \
         to this plugin running sandbox)"
    );
    assert!(
        ceiling_yes.runs_sandbox,
        "a manifest that DID declare host:sandbox:* derives \
         runs_sandbox=true (the user consented at install)"
    );

    // The SAME predicate the device caller uses (ONE code path).
    assert!(
        envelope_ceiling_rejects_cap(&ceiling_no, "host:sandbox:exec"),
        "build-constraint iii: the manifest-derived ceiling is enforced \
         through the IDENTICAL envelope_ceiling_rejects_cap core"
    );

    // End-to-end through the manifest chain-walk (the #669 deliverable).
    let plugin_kp = Keypair::generate();
    let plugin_did = plugin_kp.public_key().to_did();
    let manifest = {
        let mut m = manifest_with_requires(vec!["store:notes:write"]);
        m.peer_did = plugin_did.clone();
        m
    };
    let leaf_aud = Keypair::generate();
    let escalating_ucan = Ucan::builder()
        .issuer(plugin_did.as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("host:sandbox:exec", "*")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&plugin_kp);

    let err = validate_chain_with_manifest_ceiling(&[escalating_ucan], &manifest, &plugin_did)
        .expect_err(
            "REGRESSION: the #669 plugin-manifest ceiling did not reject a \
             host:sandbox:exec cap from a plugin whose manifest never \
             declared it — CLAUDE.md #18 Layer-2/3 envelope leaked \
             (pim-2 §3.6b would-FAIL-if-reverted; the shared predicate \
             was bypassed or manifest_to_envelope_ceiling mis-derived).",
        );
    assert!(
        matches!(err, UcanError::DeviceEnvelopeViolated { .. }),
        "expected the shared envelope-ceiling error variant, got {err:?}"
    );

    // No over-rejection: a benign cap from the same plugin passes; a
    // sandbox-consented manifest passes the same cap.
    let benign = Ucan::builder()
        .issuer(plugin_did.as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("store:notes:write", "*")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&plugin_kp);
    validate_chain_with_manifest_ceiling(&[benign], &manifest, &plugin_did)
        .expect("a cap WITHIN the manifest envelope MUST still pass");

    let sandbox_ok_manifest = {
        let mut m = manifest_with_requires(vec!["host:sandbox:exec"]);
        m.peer_did = plugin_did.clone();
        m
    };
    let sandbox_ucan = Ucan::builder()
        .issuer(plugin_did.as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("host:sandbox:exec", "*")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&plugin_kp);
    validate_chain_with_manifest_ceiling(&[sandbox_ucan], &sandbox_ok_manifest, &plugin_did)
        .expect(
            "a plugin whose manifest DID declare host:sandbox:* (user \
             consented) MUST be allowed to exercise it (no over-rejection)",
        );
}

/// §3.6b closure-pin (c) — F3 durable replay-marker rejects a
/// replayed stale frame. The first presentation of a frame nonce is
/// admitted; the SECOND presentation of the SAME nonce is rejected by
/// the durable marker even within the freshness window (the ephemeral
/// `verify` step-(3) gate cannot catch an in-window replay).
#[test]
fn collapse_p5_f3_durable_replay_marker_rejects_replayed_frame() {
    use std::sync::Arc;
    // RedbBackend::open_in_memory() is the canonical `GraphBackend`
    // test substrate (the SAME shape `UCANBackend` durable tests use —
    // one durable grammar, build-constraint iii).
    let backend =
        Arc::new(benten_graph::RedbBackend::open_in_memory().expect("redb in-memory open"));
    let marker = FrameReplayMarker::new(Arc::clone(&backend));

    // SAFETY/WHY: This is a deliberate fixed test-fixture nonce, NOT a
    // production cryptographic value. The F3 durable-replay-marker test
    // MUST present the SAME nonce twice to assert the marker REJECTS the
    // replay — a random nonce cannot exercise the replay-detection
    // property. Production frame nonces are CSPRNG-generated
    // (`benten_sync::handshake::random_nonce` → `Keypair::generate` →
    // `SigningKey::generate(&mut OsRng)`); `FrameReplayMarker` itself
    // only CONSUMES a caller-supplied `&[u8]` and never generates a
    // nonce. There is no hard-coded nonce on any production path. CodeQL
    // false-positive on intentional replay-test methodology.
    // codeql[rust/hard-coded-cryptographic-value]
    let nonce = [0xA5u8; 32];

    // First observation: NOT a replay — the marker records it.
    let first = marker
        .mark_and_check_frame(&nonce)
        .expect("durable marker store must succeed");
    assert!(
        !first,
        "first presentation of a frame nonce is NOT a replay (admitted; \
         marker now persisted)"
    );

    // Second observation of the SAME nonce: REPLAY — rejected.
    let second = marker
        .mark_and_check_frame(&nonce)
        .expect("durable marker store must succeed");
    assert!(
        second,
        "REGRESSION: a replayed frame nonce was NOT detected — the F3 \
         durable replay-marker did not persist (Compromise #23 \
         anti-replay 'dropped' instead of 're-homed'). pim-2 §3.6b \
         would-FAIL-if-reverted: if mark_and_check_frame stopped \
         persisting (no-op put) or always returned false, the replay \
         would be silently admitted."
    );

    // Durability across a fresh marker over the SAME backend (the
    // re-home is DURABLE, not per-session-ephemeral — this is the
    // exact property the P3 freshness gate lacked).
    let marker2 = FrameReplayMarker::new(Arc::clone(&backend));
    let across = marker2
        .mark_and_check_frame(&nonce)
        .expect("durable marker store must succeed");
    assert!(
        across,
        "the replay-marker MUST be durable across marker instances over \
         the same backend (F3: durable, not ephemeral — the gap the \
         ephemeral P3 freshness gate could not close)"
    );

    // A DIFFERENT nonce is independent (no false-positive replay).
    // SAFETY/WHY: deliberate fixed test-fixture nonce (see the WHY note
    // above) — asserts a distinct nonce does NOT false-positive as a
    // replay. Not a production crypto value; production uses CSPRNG.
    // codeql[rust/hard-coded-cryptographic-value]
    let other = [0x5Au8; 32];
    assert!(
        !marker
            .mark_and_check_frame(&other)
            .expect("durable marker store must succeed"),
        "a distinct frame nonce is independent — no false replay reject"
    );
}
