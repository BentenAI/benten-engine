//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — TF-11 benten-sync
//! light-client mode-b/c sub-lane. Agent **R3-B4**. RED-PHASE;
//! un-ignore at the **benten-sync light-client sub-lane** wave
//! (G-CORE-7-adjacent / Phase-3-deferred sub-lane per §3 plan).
//!
//! ## Provenance / R2-map
//!
//! - r2-test-landscape.md **TF-11** ("benten-sync light-client
//!   mode-b/c sub-lane (substrate)") + §4-C exit-criterion **C10** row
//!   + §2.B "schema-bump conformance arm is freeze-deferred per P-III".
//! - Plan `.addl/phase-4-meta/00-implementation-plan.md`
//!   "Phase-3-deferred disposition … incl. the named `benten-sync`
//!   light-client sub-lane" group def + **C10** exit criterion +
//!   **§1.A.FROZEN item 4** (the `SnapshotBlob.schema_version 1→2`
//!   bump co-scheduled into the D2 P-III decision-point) + the
//!   **§8-B-(i)** structural resolution (Ben-ACKED 2026-05-19).
//! - Covers, per TF-11: `MerkleRangeProofBackend` trait (mode-b) +
//!   mode-(c) signed-checkpoint logic (via `benten-id` PeerDid/Keypair)
//!   landing IN `benten-sync` (the "keep benten-graph thin" §8-B
//!   intent preserved); the `SnapshotBlob.schema_version 1→2` bump =
//!   an in-place `benten-graph` struct mutation (P-III — scheduled at
//!   G-CORE-9, NOT landed autonomously); the `SchemaVersion`
//!   strict-mismatch error variant IS the documented backward-compat
//!   migration path.
//!
//! ## §8-B-(i) P-III pin — EXPLICIT (Ben-ACKED 2026-05-19)
//!
//! The §8-B-(i) structural resolution: the mode-(c) `SnapshotBlob`
//! checkpoint-signature field forces an **in-place `benten-graph`
//! struct mutation** (the struct + `SNAPSHOT_BLOB_SCHEMA_VERSION:
//! u32 = 1` live in `crates/benten-graph/src/backends/
//! snapshot_blob.rs`; ZERO `SnapshotBlob` in `crates/benten-sync/
//! src/`). That `1→2` bump is a **P-III wire/on-disk format change**
//! co-scheduled into the SAME G-CORE-9 D2-freeze Ben decision-point
//! (§1.A.FROZEN item 4 / §8-F). **It is NOT landed autonomously by
//! this sub-lane.** The `MerkleRangeProofBackend` trait surface is
//! pure-trait/non-wire and stays in `benten-sync` — the P-III
//! scheduling confines to the in-place `benten-graph`
//! schema-version-bump half only.
//!
//! **The SnapshotBlob P-III pin lives in a benten-graph test, NOT
//! here.** `benten-sync` does NOT depend on `benten-graph` (documented
//! layered-dependency intent in `benten-sync` `[lib]` +
//! `tests/dependency_edges.rs`); since `SnapshotBlob` +
//! `SNAPSHOT_BLOB_SCHEMA_VERSION` structurally live in `benten-graph`
//! (the §8-B-(i) in-place mutation surface), the P-III migration-path
//! guard + the §3.5m "constant-is-still-1" tripwire are in
//! `crates/benten-graph/tests/
//! tf11_snapshot_blob_schema_version_p3_migration_path.rs` (same R3-B4
//! agent, TF-11 family, benten-graph half). Those guards are GREEN at
//! HEAD (verify-STAYS-regression), distinct from the RED mode-b/c
//! staged-pins in THIS file — flagged in the §3.6e split (see report).
//! This benten-sync file is purely the mode-b/c light-client half.
//!
//! ## Ground-truth split at HEAD `ed03729a` (R3-B4 §3.5n pass)
//!
//! - **mode-(b) `MerkleRangeProofBackend` range-proof** — STILL
//!   UNDELIVERED. `light_client.rs` doc: "Mode-(b) range-query proof
//!   + mode-(c) signed-checkpoint are OOS [Phase-3]". No
//!   `MerkleRangeProofBackend` trait / range-proof type exists. A
//!   stranded `#[ignore]` ARCHITECTURAL-ABSENCE pin
//!   (`light_client_mode_b_range_query_proof_oos_phase_3_deferred_to_phase_4`)
//!   in `light_client_distinct.rs` is the §3.6e staged-pin this lane
//!   un-ignores. RED-PHASE (would-FAIL).
//! - **mode-(c) signed-checkpoint** — STILL UNDELIVERED. No
//!   checkpoint-commitment / signature-verify / freshness surface in
//!   `light_client.rs`. Stranded `#[ignore]` sibling
//!   (`light_client_mode_c_signed_checkpoint_oos_phase_3_deferred_to_phase_4`).
//!   RED-PHASE (would-FAIL).
//! - **`SnapshotBlob` schema-version strict-mismatch reject** —
//!   LANDED + GREEN (the `SchemaVersion` variant + the
//!   `from_bytes` strict-check exist at HEAD). The P-III pin here is
//!   a **verify-stays-regression** guard that the documented
//!   backward-compat path holds; the `1→2` BUMP itself is the
//!   G-CORE-9 P-III decision, NOT this lane's work.
//!
//! ## Disjointness (R3-B4 lane vs R3-B5 / siblings — §3.5i)
//!
//! TF-11 owns ONLY the `benten-sync` **light-client** module surface
//! (`light_client` + `mst` for proof construction) + the read-only
//! `benten-graph::backends::snapshot_blob` P-III pin. It is sliced by
//! MODULE from TF-8's benten-sync sub-lane (R3-B5), which owns the
//! sync **attack-fabric** / `apply_atrium_merge` manifest-envelope-
//! recheck / thin-client surfaces. This file touches NONE of:
//! `apply_atrium_merge_manifest_envelope_recheck.rs`,
//! `attack_*`, `wire_envelope.rs`, thin-client, `manifest_envelope_
//! recheck`. TF-7's install/lifecycle file is in a DIFFERENT crate
//! (`benten-platform-foundation`).
//!
//! ## §3.6f SHAPE-not-SUBSTANCE guard (pim-18)
//!
//! The mode-b/c pins exercise the **production** light-client verify
//! surface with an OBSERVABLE consequence (an unchecked range-proof
//! is accepted / an unsigned-or-forged checkpoint verifies) that
//! WOULD-FAIL if the verification is a no-op. They are NOT
//! type-constructibility assertions.
//!
//! ## R3-brief inherited-discipline pre-flight checklist (§3.6g —
//! reproduced as LITERAL lines, NOT a §-reference; fix-6 directive)
//!
//! - [x] §3.5b HARDENED (pim-1): tests-only here; the sub-lane implementer sweeps adjacent docs (light-client mode commitment prose; ROADMAP Phase-4 deferral list) before push.
//! - [x] §3.6b + sub-rule 4 (pim-2): each pin is a PRODUCTION-ARM + OBSERVABLE-CONSEQUENCE + WOULD-FAIL pin on the SPECIFIC arm (range-proof-verify / checkpoint-sig-verify / schema-strict-reject — not an umbrella "light-client works").
//! - [x] §3.6e (pim-12): RED-PHASE staged-pins; the closing sub-lane wave un-ignores; the reviewer verifies LANDING-STATUS. The two stranded sibling `#[ignore]` ARCHITECTURAL-ABSENCE pins in `light_client_distinct.rs` (mode-b / mode-c) are NAMED here for the co-un-ignore sweep.
//! - [x] §3.6f (pim-18): SHAPE-not-SUBSTANCE — production verify call-site + substantive body + aspirational-prose-gap check.
//! - [x] §3.5g (cross-language/doc/tool rule-mirror): no ErrorCode mint here.
//! - [x] §3.5i: mini-reviewer FIRST action = tree-state-freshness vs merge-base; assert R3-B4 ⟂ R3-B5 module-disjointness.
//! - [x] §3.5j: §3.5h pre-push runs `cargo +stable clippy --workspace --all-targets -- -D warnings` + MSRV 1.95.
//! - [x] §3.6g: prior-phase pim-N reproduced as explicit lines.
//! - [x] §3.6h: no rule/codification originates here.
//! - [x] §3.6i: the R3-B4 report carries canonical `disposition` + `findings[]`.
//! - [x] §3.6j: "swept" claims validated over THIS wave's outputs.
//! - [x] §3.13: per-test locals (each pin builds its own `Mst` / `Keypair`); NO shared static under the parallel runner.
//! - [x] §3.5h: base 5-check + MANDATORY-PRE-MERGE + `jq .` + GREEN-CI-CONFIRMATION clauses.
//! - [x] §3.11: resume-into-same-worktree on kill (TF-11 is a medium lane, not the largest — §3.11 mandatory for TF-8).
//! - [x] §3.5l/§3.5m/§3.5n: combined-push verify; **§3.5m P-III — the `SnapshotBlob 1→2` bump is a P-III Ben-scheduled wire-format change, NOT an orchestrator side-effect; this lane does NOT land it**; orchestrator ground-truth every finding.
//! - [x] Iterate-to-convergence + canary-first: TF-11 is a SUBSTRATE named benten-sync sub-lane — lands AFTER the 2-canary opening pair merges; the schema-bump conformance arm is FREEZE-deferred per P-III.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(dead_code)]

use benten_id::keypair::Keypair;
use benten_sync::light_client::{BandwidthBudget, LightClient};
use benten_sync::mst::{Mst, MstCid, MstEntry};

// =====================================================================
// mode-(b) — MerkleRangeProofBackend range-query proof
// =====================================================================

/// **TF-11 / C10 — mode-(b) RED-PHASE.**
///
/// §3.6e staged-pin redirect: the sibling
/// `light_client_distinct.rs::
/// light_client_mode_b_range_query_proof_oos_phase_3_deferred_to_phase_4`
/// is an `#[ignore]` ARCHITECTURAL-ABSENCE pin explicitly deferred
/// "to Phase-4 Benten Platform v1". This benten-sync sub-lane IS that
/// named destination.
///
/// Production-arm intent (wired at un-ignore): a
/// `MerkleRangeProofBackend` (pure-trait, non-wire, lives in
/// `benten-sync` per §8-B "keep benten-graph thin") produces an
/// authenticated range proof over a sorted MST sub-range; the
/// light-client verifies the range proof reconstructs to the
/// published root and rejects a tampered range proof. Bandwidth
/// bounded by range size.
///
/// Would-FAIL (current HEAD): no `MerkleRangeProofBackend` trait /
/// range-proof type exists — `light_client.rs` only implements
/// mode-(a) single-CID inclusion. A range-proof verify cannot even
/// be referenced. RED.
#[test]
#[ignore = "RED-PHASE: un-ignore at the benten-sync light-client sub-lane wave (TF-11 mode-(b) MerkleRangeProofBackend — pure-trait/non-wire range-proof in benten-sync per §8-B; the named Phase-4 destination for the stranded light_client_distinct.rs mode-(b) ARCHITECTURAL-ABSENCE #[ignore] per §3.6e). r2-test-landscape TF-11 / C10."]
fn light_client_mode_b_range_proof_verifies_against_production_backend() {
    // -----------------------------------------------------------------
    // SUBSTANTIVE-arm-not-SHAPE shape (R4.1 fix-pass per pim-18 §3.6f /
    // L3 finding-5; symmetric to mode-c above):
    // exercise the SHIPPED `Mst` / `MstEntry` / `MstCid` / `MerkleProof`
    // primitives the future `MerkleRangeProofBackend` will compose on.
    // Mode-(b) is a range proof over a sub-range; the substrate is the
    // SHIPPED single-key `MerkleProof` shape — a range proof is N
    // composed proofs over sorted entries. Drive the SHIPPED
    // tamper-detection contract as the would-FAIL anchor.
    // -----------------------------------------------------------------
    let mut mst = Mst::new();
    // Build a sorted multi-entry MST (the substrate for the future
    // range-proof's sorted-sub-range guarantee).
    mst.insert(MstEntry::from_payload("/zone/a", vec![10, 20]));
    mst.insert(MstEntry::from_payload("/zone/b", vec![30, 40]));
    mst.insert(MstEntry::from_payload("/zone/c", vec![50, 60]));
    let root: MstCid = mst.root_cid();
    assert_eq!(mst.len(), 3, "MST primitive sanity: 3 entries inserted");

    // Build a SHIPPED single-key proof — the per-key primitive the
    // future range proof iterates. Mode-(a) primitive sanity: an
    // honest proof reconstructs to the published root.
    let proof_b = mst
        .merkle_proof_for("/zone/b")
        .expect("present key has a proof (mode-a primitive sanity)");
    assert_eq!(
        proof_b.reconstruct_root(),
        root,
        "shipped surface exercise (substrate for mode-b): an honest \
         single-key proof reconstructs to the published root — the \
         per-key invariant a range proof iterates over a sub-range."
    );

    // Tampered proof MUST NOT verify (the tamper-detection contract
    // the future range proof composes on: a range proof rejects if
    // ANY sibling entry was mutated). Would-FAIL signal: if the
    // SHIPPED reconstruct_root regressed to be input-agnostic, the
    // range-proof substrate is broken.
    let tampered = proof_b.with_tampered_node();
    assert_ne!(
        tampered.reconstruct_root(),
        root,
        "shipped surface exercise (substrate for mode-b): a tampered \
         proof's reconstructed root MUST differ from the published root \
         (tamper-detection contract; the same property the future \
         MerkleRangeProofBackend will enforce over sub-ranges). \
         Would-FAIL if the primitive regressed."
    );

    // Bandwidth-budget substrate (the bounded-bandwidth property the
    // future range proof must preserve): the SHIPPED `LightClient`
    // budget surface exists at HEAD; mode-(b) extends the same budget
    // semantics over a range. Sanity-check the substrate primitive.
    let lc = LightClient::with_budget(BandwidthBudget::default());
    assert_eq!(
        lc.bytes_consumed(),
        0,
        "shipped surface exercise: a fresh LightClient has zero bytes \
         consumed (the substrate the range-proof bandwidth-budget \
         assertion will compose on)."
    );

    // -----------------------------------------------------------------
    // RED-arm: `MerkleRangeProofBackend` itself is UNDELIVERED at HEAD
    // (light_client.rs doc: "Mode-(b) range-query proof OOS [Phase-3]").
    // The substrate primitives (exercised above) are correct; what is
    // missing is the range-proof TYPE + verify path that iterates them
    // over a contiguous sub-range with bounded bandwidth.
    // -----------------------------------------------------------------
    panic!(
        "RED-PHASE (benten-sync light-client sub-lane): mode-(b) \
         MerkleRangeProofBackend not built at HEAD (light_client.rs: \
         mode-(b) range-query proof OOS for Phase-3). The SHIPPED \
         single-key MerkleProof substrate is correct (exercised above: \
         honest proof reconstructs to root, tampered proof does not, \
         LightClient bandwidth-budget primitive present). This pin + \
         the stranded light_client_distinct.rs mode-(b) #[ignore] \
         ARCHITECTURAL-ABSENCE pin un-ignore together (§3.6e)."
    );
}

// =====================================================================
// mode-(c) — signed-checkpoint verification (via benten-id Keypair)
// =====================================================================

/// **TF-11 / C10 — mode-(c) RED-PHASE.**
///
/// §3.6e staged-pin redirect: the sibling
/// `light_client_distinct.rs::
/// light_client_mode_c_signed_checkpoint_oos_phase_3_deferred_to_phase_4`
/// `#[ignore]` ARCHITECTURAL-ABSENCE pin, named-destination = this
/// benten-sync sub-lane.
///
/// Production-arm intent (wired at un-ignore): a full peer signs an
/// MST-root checkpoint (root `MstCid` + HLC time) with its
/// `benten-id` `Keypair`; the light-client verifies the checkpoint
/// signature against the peer's public key/PeerDid AND enforces
/// freshness/replay defenses. A forged or stale checkpoint MUST be
/// rejected.
///
/// Would-FAIL (current HEAD): no checkpoint-commitment / signature-
/// verify / freshness surface exists in `light_client.rs` (mode-(c)
/// OOS for Phase-3). A forged checkpoint would be indistinguishable.
/// RED.
#[test]
#[ignore = "RED-PHASE: un-ignore at the benten-sync light-client sub-lane wave (TF-11 mode-(c) signed-checkpoint — full-peer-signed MST-root checkpoint verified via benten-id PeerDid/Keypair + freshness/replay defense; named Phase-4 destination for the stranded light_client_distinct.rs mode-(c) ARCHITECTURAL-ABSENCE #[ignore] per §3.6e). r2-test-landscape TF-11 / C10."]
fn light_client_mode_c_verifies_signed_checkpoint_and_rejects_forgery() {
    // Sanity: the benten-id signing primitive the mode-(c) checkpoint
    // verification will use IS available at HEAD (this half is the
    // dependency, not the gap — the gap is the checkpoint surface in
    // benten-sync that consumes it).
    let peer_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();
    let mut mst = Mst::new();
    mst.insert(MstEntry::from_payload("/zone/posts/p1", vec![1, 2, 3]));
    let root: MstCid = mst.root_cid();

    // The would-be checkpoint payload (root || hlc) the full peer
    // signs. At un-ignore the production mode-(c) surface constructs
    // + verifies this; here we only assert the signing primitive is
    // present so the RED arm is unambiguously the benten-sync
    // checkpoint surface, not a missing crypto dep.
    let mut checkpoint_payload = Vec::new();
    checkpoint_payload.extend_from_slice(root.to_hex().as_bytes());
    checkpoint_payload.extend_from_slice(&42u64.to_le_bytes()); // HLC
    let sig = peer_kp.sign(&checkpoint_payload);
    peer_kp
        .public_key()
        .verify(&checkpoint_payload, &sig)
        .expect(
            "PRE-CONDITION: benten-id Keypair sign/verify is the \
                 available primitive mode-(c) consumes (not the gap)",
        );
    // Forged signature MUST NOT verify under the peer's key (this is
    // the property the absent benten-sync checkpoint surface must
    // enforce — asserted here only as a primitive sanity).
    let forged = attacker_kp.sign(&checkpoint_payload);
    assert!(
        peer_kp
            .public_key()
            .verify(&checkpoint_payload, &forged)
            .is_err(),
        "primitive sanity: a foreign-key signature must not verify"
    );

    // Intentional RED-PHASE failure: there is NO benten-sync
    // `light_client` checkpoint-commitment / verify / freshness API
    // at HEAD. The sub-lane builds it; the un-ignored body drives the
    // PRODUCTION mode-(c) surface (signed checkpoint accepted,
    // forged checkpoint rejected, stale checkpoint rejected).
    panic!(
        "RED-PHASE (benten-sync light-client sub-lane): mode-(c) \
         signed-checkpoint surface not built at HEAD (light_client.rs: \
         mode-(c) signed checkpoint OOS for Phase-3). This pin + the \
         stranded light_client_distinct.rs mode-(c) #[ignore] \
         ARCHITECTURAL-ABSENCE pin un-ignore together (§3.6e)."
    );
}
