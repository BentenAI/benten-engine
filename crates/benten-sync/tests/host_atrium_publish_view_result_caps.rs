//! R4-FP/R3-C RED-PHASE pins for `host:atrium:publish_view_result`
//! capability + 5 trust-mode patterns (Ben's D2 decision 2026-05-04).
//!
//! ## D2 (Ben's decision 2026-05-04): D-PHASE-3-21 = option (iii)
//!
//! User-view replication via content-addressed snapshots gated by a
//! UCAN capability `host:atrium:publish_view_result`. NO new
//! trust-policy primitive — trust modes EMERGE from UCAN delegation
//! patterns.
//!
//! 5 trust-mode patterns capture the spectrum of operational shapes:
//!
//!   1. Cap-required-for-publishing (positive surface)
//!   2. Consumer-side check against UCAN chain (delivery-time)
//!   3. Trust-anyone via wildcard UCAN ("publish from any audience")
//!   4. Trust-allowlist via specific UCANs (named audiences only)
//!   5. Trust-no-one via no delegation → recompute locally
//!
//! ## Cross-references
//!
//! - Plan §5 D-PHASE-3-21 (3-architecture fork; D2 selects option iii).
//! - `crates/benten-ivm/tests/algorithm_b_cross_replica.rs` —
//!   companion ivm-r4-1 BLOCKER closure (Algorithm-B-meets-Loro
//!   on the IVM side; THIS file pins the consumer's UCAN-gated
//!   acceptance of the published snapshot).
//! - CLAUDE.md baked-in #14 (capability system as pluggable policy;
//!   UCAN is one backend).
//!
//! ## RED-PHASE discipline
//!
//! All `#[ignore]`'d with rationale citing G16-B + G14-B wave-6b
//! ownership.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — host:atrium:publish_view_result cap REQUIRED for publishing snapshot. G16-B wave-6b shipped the production Atrium API surface; test body pins specific cap-required-for-publish defensive contract that needs driver authoring; un-ignore at §6.12 G16-B post-canary residuals landing (per-device write filter v1-assessment-window scope) per Wave-E rationale-only sweep."]
fn host_atrium_publish_view_result_cap_required_for_publishing() {
    // D2 trust-mode pattern 1: cap-required-for-publishing.
    //
    // The publishing peer (the one whose engine has materialized the
    // user-view + wants to share its content-addressed snapshot CID
    // with peers) MUST hold an `host:atrium:publish_view_result`
    // capability — typically obtained via UCAN delegation from the
    // atrium owner or via self-delegation of an originating root cap.
    //
    //   use benten_caps::host_caps::HostCap;
    //   use benten_sync::view_replication::PublishViewResult;
    //
    //   let mut peer_a = test_peer(peer_a_did);
    //   peer_a.atrium_join(shared_atrium()).await.unwrap();
    //
    //   // Without the cap, publishing fails:
    //   let view_cid = peer_a.materialize_and_snapshot_user_view("custom:posts").unwrap();
    //   let publish_result_no_cap = peer_a.atrium_publish_view_result(view_cid).await;
    //   match publish_result_no_cap {
    //       Err(EngineError::CapMissing { cap, .. }) => {
    //           assert_eq!(cap, "host:atrium:publish_view_result");
    //       }
    //       other => panic!("expected CapMissing, got {other:?}"),
    //   }
    //
    //   // With the cap held in the actor's effective cap-set,
    //   // publishing succeeds:
    //   peer_a.set_effective_cap_set(cap_set_with(HostCap::AtriumPublishViewResult));
    //   peer_a.atrium_publish_view_result(view_cid).await.unwrap();
    //
    // OBSERVABLE consequence: publishing the view-result snapshot CID
    // requires the cap; defends against unauthorised view-result
    // injection attack class.
    unimplemented!(
        "G16-B + G14-B wire host:atrium:publish_view_result cap-required gate at publish entry-point"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — host:atrium:publish_view_result consumer-side UCAN-chain check at delivery. G16-B wave-6b shipped Atrium delivery surface; test body pins consumer-side UCAN-chain check defensive contract; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn host_atrium_publish_view_result_cap_consumer_side_check_against_ucan_chain() {
    // D2 trust-mode pattern 2: consumer-side check.
    //
    // When peer-B receives a published view-result snapshot CID over
    // the wire, peer-B MUST verify the publisher's UCAN chain
    // delegates `host:atrium:publish_view_result` for the named
    // user-view + atrium scope BEFORE accepting the snapshot. This is
    // a delivery-time check, distinct from the publisher-side gate.
    //
    //   use benten_id::ucan::Chain;
    //
    //   let mut peer_a = test_peer(peer_a_did);
    //   let mut peer_b = test_peer(peer_b_did);
    //   peer_a.atrium_join(shared_atrium()).await.unwrap();
    //   peer_b.atrium_join(shared_atrium()).await.unwrap();
    //
    //   // peer_a publishes WITHOUT a valid UCAN chain delegating the cap:
    //   let snap_cid = peer_a.materialize_and_snapshot_user_view("custom:posts").unwrap();
    //   let bad_chain = synthesize_ucan_chain_without_publish_cap(peer_a_did);
    //   peer_a.atrium_publish_view_result_with_chain(snap_cid, bad_chain).await.unwrap();
    //
    //   // peer_b receives + REJECTS:
    //   wait_for_atrium_sync(&[&peer_a, &peer_b]).await;
    //   match peer_b.atrium_consume_published_view_result(snap_cid).await {
    //       Err(ViewReplicationError::UcanChainMissingPublishCap { audience, .. }) => {
    //           assert_eq!(audience, peer_a_did);
    //       }
    //       other => panic!("expected UcanChainMissingPublishCap, got {other:?}"),
    //   }
    //
    //   // peer_a re-publishes WITH a valid UCAN chain:
    //   let good_chain = synthesize_ucan_chain_with_publish_cap(peer_a_did);
    //   peer_a.atrium_publish_view_result_with_chain(snap_cid, good_chain).await.unwrap();
    //   wait_for_atrium_sync(&[&peer_a, &peer_b]).await;
    //   peer_b.atrium_consume_published_view_result(snap_cid).await.unwrap();
    //
    // OBSERVABLE consequence: consumer-side check fires before
    // accepting any snapshot; defends against publisher-side-only-
    // gate bypass + UCAN-chain forgery + snapshot-spoofing.
    unimplemented!(
        "G16-B + G14-B wire consumer-side UCAN-chain check on host:atrium:publish_view_result at delivery"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — host:atrium:publish_view_result trust-anyone via wildcard UCAN. G16-B wave-6b shipped Atrium API; test body pins wildcard-UCAN trust-anyone shape that needs driver authoring; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn host_atrium_publish_view_result_cap_trust_anyone_via_wildcard_ucan() {
    // D2 trust-mode pattern 3: trust-anyone (wildcard UCAN).
    //
    // The atrium owner can issue a wildcard UCAN delegating
    // `host:atrium:publish_view_result` to any audience (audience =
    // "*" or equivalent UCAN-spec wildcard). This is the "trust
    // anyone" mode — operationally rare but architecturally legitimate
    // (e.g. read-only public view-snapshots in a public garden).
    //
    // No new trust-policy primitive: trust-anyone EMERGES from a
    // UCAN with audience-wildcard. The same chain-walk consumer-side
    // check (pattern 2) accepts.
    //
    //   use benten_id::ucan::{UcanBuilder, Audience};
    //
    //   let owner_kp = test_atrium_owner_kp();
    //   let wildcard_ucan = UcanBuilder::new()
    //       .issuer(&owner_kp)
    //       .audience(Audience::Wildcard)
    //       .capability("host:atrium:publish_view_result", "*")
    //       .build();
    //
    //   // Any peer-DID may now publish a snapshot under this chain:
    //   let mut peer_random = test_peer(arbitrary_peer_did());
    //   peer_random.atrium_join(shared_atrium()).await.unwrap();
    //   let chain = vec![wildcard_ucan];
    //   let snap_cid = peer_random.materialize_and_snapshot_user_view("public:posts").unwrap();
    //   peer_random.atrium_publish_view_result_with_chain(snap_cid, chain.clone()).await.unwrap();
    //
    //   // ANY consumer accepts (the chain validates regardless of
    //   // peer_random's identity):
    //   let mut peer_consumer = test_peer(other_peer_did());
    //   peer_consumer.atrium_join(shared_atrium()).await.unwrap();
    //   wait_for_atrium_sync(&[&peer_random, &peer_consumer]).await;
    //   peer_consumer.atrium_consume_published_view_result(snap_cid).await.unwrap();
    //
    // OBSERVABLE consequence: trust-anyone mode emerges from UCAN
    // wildcard-audience delegation; no new trust-policy primitive
    // needed. Closes the "do we need a trust-policy enum?" question
    // raised in pre-D2 brainstorming with NO new primitive.
    unimplemented!(
        "G16-B + G14-B wire UCAN wildcard-audience acceptance for host:atrium:publish_view_result trust-anyone mode"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — host:atrium:publish_view_result trust-allowlist via specific UCANs. G16-B wave-6b shipped Atrium API; test body pins specific-UCAN-allowlist shape that needs driver authoring; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn host_atrium_publish_view_result_cap_trust_allowlist_via_specific_ucans() {
    // D2 trust-mode pattern 4: trust-allowlist (specific UCANs).
    //
    // The atrium owner issues UCANs with specific audience-DIDs
    // (peer_a_did, peer_c_did) — only those DIDs can publish
    // view-results. Peer_b_did's published snapshots are rejected
    // by consumer-side chain-walk because no UCAN with audience =
    // peer_b_did exists in the chain.
    //
    // This is the most common operational shape (allowlist = trusted
    // collaborators).
    //
    //   let owner_kp = test_atrium_owner_kp();
    //   let ucan_a = UcanBuilder::new()
    //       .issuer(&owner_kp)
    //       .audience(peer_a_did)
    //       .capability("host:atrium:publish_view_result", "/zone/posts")
    //       .build();
    //   let ucan_c = UcanBuilder::new()
    //       .issuer(&owner_kp)
    //       .audience(peer_c_did)
    //       .capability("host:atrium:publish_view_result", "/zone/posts")
    //       .build();
    //
    //   // peer_a + peer_c can publish; peer_b cannot:
    //   peer_a.atrium_publish_view_result_with_chain(snap_a_cid, vec![ucan_a]).await.unwrap();
    //   peer_c.atrium_publish_view_result_with_chain(snap_c_cid, vec![ucan_c]).await.unwrap();
    //
    //   // peer_b synthesizes a chain that does NOT carry an audience
    //   // matching peer_b_did:
    //   match peer_b.atrium_publish_view_result_with_chain(snap_b_cid, vec![]).await {
    //       Err(ViewReplicationError::UcanChainMissingPublishCap { .. }) => {}
    //       other => panic!("expected UcanChainMissingPublishCap for peer_b (not in allowlist), got {other:?}"),
    //   }
    //
    //   // Even if peer_b SOMEHOW publishes (e.g. forged chain), the
    //   // consumer-side check at peer_d catches the missing
    //   // peer_b-audience UCAN:
    //   wait_for_atrium_sync(&[&peer_a, &peer_b, &peer_c, &peer_d]).await;
    //   match peer_d.atrium_consume_published_view_result(snap_b_cid).await {
    //       Err(ViewReplicationError::UcanChainMissingPublishCap { .. }) => {}
    //       other => panic!("expected consumer-side rejection for peer_b allowlist miss, got {other:?}"),
    //   }
    //
    // OBSERVABLE consequence: allowlist mode emerges from
    // specific-audience UCAN delegations; non-allowlisted publishers
    // are rejected at both publisher-side gate AND consumer-side
    // chain-walk.
    unimplemented!(
        "G16-B + G14-B wire UCAN specific-audience allowlist acceptance + non-listed-DID rejection"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — host:atrium:publish_view_result trust-no-one via no delegation → recompute locally. G16-B wave-6b shipped Atrium API; test body pins trust-no-one + local-recompute fallback shape; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn host_atrium_publish_view_result_cap_trust_no_one_via_no_delegation_recompute_locally() {
    // D2 trust-mode pattern 5: trust-no-one.
    //
    // The atrium owner issues NO `host:atrium:publish_view_result`
    // delegations. ALL peers must recompute the user-view locally
    // from their own replicated graph state — never accepting a
    // snapshot from a peer. This is the highest-trust-floor mode,
    // useful for adversarial-environment atriums or sensitive
    // user-views where recomputation cost is acceptable.
    //
    // No new trust-policy primitive: trust-no-one EMERGES from the
    // ABSENCE of any UCAN delegating `host:atrium:publish_view_result`
    // — the consumer-side chain-walk simply finds zero applicable
    // chains and falls back to local materialization.
    //
    //   // atrium owner issues NO publish-view-result UCANs.
    //
    //   // All peers register the same user-view definition + each
    //   // recomputes locally:
    //   for peer in &mut peers {
    //       peer.register_user_view(&view_def).unwrap();
    //   }
    //
    //   // No peer publishes a snapshot (or if anyone tries, they're
    //   // rejected at publisher-side per pattern 1):
    //   let snap_a_cid = peers[0].materialize_and_snapshot_user_view("custom:posts").unwrap();
    //   match peers[0].atrium_publish_view_result(snap_a_cid).await {
    //       Err(EngineError::CapMissing { cap, .. }) => {
    //           assert_eq!(cap, "host:atrium:publish_view_result");
    //       }
    //       other => panic!("expected CapMissing under trust-no-one, got {other:?}"),
    //   }
    //
    //   // Each peer recomputes locally + canonical-bytes match
    //   // (deterministic recomputation under the same graph state):
    //   wait_for_atrium_convergence(&peers).await;
    //   let view_bytes: BTreeSet<_> = peers.iter()
    //       .map(|p| p.materialize_user_view(&view_def.id()).unwrap().to_canonical_bytes())
    //       .collect();
    //   assert_eq!(view_bytes.len(), 1,
    //       "trust-no-one mode: each peer recomputes locally; deterministic recomputation produces identical canonical-bytes");
    //
    //   // Recomputation is observable as a metric (no peer accepted
    //   // any external snapshot):
    //   for peer in &peers {
    //       assert_eq!(peer.consumed_published_view_result_count(), 0,
    //           "trust-no-one: zero consumed-published-snapshots");
    //       assert!(peer.locally_materialized_user_view_count() > 0,
    //           "trust-no-one: at least one local materialization");
    //   }
    //
    // OBSERVABLE consequence: trust-no-one mode emerges from absence
    // of UCAN delegation — peers recompute locally; no snapshot
    // publication happens. Closes the trust-mode spectrum at the
    // safest end. Confirms NO new trust-policy primitive is needed.
    unimplemented!(
        "G16-B + G14-B wire trust-no-one mode = absence-of-delegation = local recomputation only"
    );
}
