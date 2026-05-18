//! Phase 4-Foundation R4b-FP-2 — G24-B EXIT CRITERION substantive
//! replay arm via real engine round-trip.
//!
//! Closes `phase-4-backlog.md §4.16` (g24b-mr-1 MAJOR):
//! the existing `replay_produces_identical_content_hash_encoding_only`
//! canary at
//! `crates/benten-platform-foundation/src/admin_ui_v0/workflow_editor.rs`
//! is a **degenerate same-struct double-hash** — both sides call
//! `canonical_subgraph_bytes(&sg_save)` on the same in-memory
//! `Subgraph`, so equality proves only that the encoder is
//! deterministic over a stable struct. It does NOT exercise an
//! encode → store → load → decode → re-encode round trip through the
//! real `benten_engine::Engine` — the substance the G24-B EXIT
//! CRITERION row demands.
//!
//! **What THIS pin establishes (per pim-2 §3.6b + pim-18 §3.6f):**
//!
//! - **PRODUCTION-ARM:** drives the real production write path
//!   `compile_draft_within_manifest_envelope` → encode to canonical
//!   bytes → `Engine::create_node` (Inv-11 + transactional ChangeEvent
//!   fan-out) under the admin-UI plugin-DID principal CID.
//! - **OBSERVABLE-CONSEQUENCE:** reads the persisted Node back via
//!   `Engine::read_node_as(admin_ui_principal, cid)` (the **Class B β
//!   seam** per CLAUDE.md baked-in #18 cag-r1-9) gated by a granted
//!   `store:AdminUiV0Workflow:read` cap; decodes the persisted bytes
//!   into a fresh `Subgraph` via `Subgraph::load_verified`;
//!   re-canonicalises + re-hashes; asserts byte-for-byte equality
//!   with the save-time hash. Also exercises `Engine::register_subgraph`
//!   so the engine's handler-version map persists the same CID.
//! - **WOULD-FAIL-IF-NO-OP'd:** if `Engine::create_node` stored a
//!   different byte sequence than `canonical_subgraph_bytes` emits
//!   (encoder ↔ decoder drift), the reloaded subgraph's CID would
//!   differ from the save-time CID; this pin fires. If
//!   `register_subgraph` recorded a CID that mismatched the
//!   `Subgraph::cid()` output, the handler-version-chain assertion
//!   fires.
//!
//! **Couples to:** §4.16 + g24b-mr-1 MAJOR; companion to the
//! encoding-only inline canary
//! `replay_produces_identical_content_hash_encoding_only`.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;

use benten_core::PrimitiveKind;
use benten_core::{Cid, Node, Subgraph, Value};
use benten_platform_foundation::admin_ui_v0::workflow_editor::{
    WorkflowDraft, WorkflowPrimitiveSelection, canonical_subgraph_bytes_for_test,
    compile_draft_within_manifest_envelope, fixture_manifest_for_test, workflow_content_hash,
};

use common::admin_ui_v0_harness::AdminUiV0TestHarness;

/// Label the test-only `Node` envelope uses to wrap the canonical
/// `Subgraph` bytes for storage. Distinct from any system-zone prefix
/// (Inv-11) and from the `Note` label `make_note_node` uses — keeps
/// this pin's grant + read isolated from the harness's pre-canned
/// `Note` fixtures.
const ADMIN_UI_WORKFLOW_NODE_LABEL: &str = "AdminUiV0Workflow";

/// Property key under which the canonical-bytes-encoded `Subgraph`
/// lives inside the persisted `Node`.
const WORKFLOW_SUBGRAPH_BYTES_PROPERTY: &str = "subgraph_bytes";

/// Build a non-trivial WorkflowDraft: 3 primitives + 2 edges so the
/// canonical-bytes encoding is materially larger than the trivial
/// single-Read shape the inline canary uses.
fn build_substantive_workflow_draft() -> WorkflowDraft {
    let mut draft = WorkflowDraft::new("substantive-replay-fixture");
    draft.drag_primitive(WorkflowPrimitiveSelection {
        id: "r_body".to_string(),
        kind: PrimitiveKind::Read,
        cap_scope: Some("read:Note.body".to_string()),
    });
    draft.drag_primitive(WorkflowPrimitiveSelection {
        id: "t_uppercase".to_string(),
        kind: PrimitiveKind::Transform,
        cap_scope: Some("transform:Note.body".to_string()),
    });
    draft.drag_primitive(WorkflowPrimitiveSelection {
        id: "w_body".to_string(),
        kind: PrimitiveKind::Write,
        cap_scope: Some("write:Note.body".to_string()),
    });
    draft.connect_edges(vec![
        ("r_body".into(), "t_uppercase".into()),
        ("t_uppercase".into(), "w_body".into()),
    ]);
    draft
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "Single-pin substantive round-trip — the 7 numbered stages \
              (compile / encode / engine-open / register_subgraph / \
              create_node / read_node_as / decode-re-encode-re-hash) are \
              single-source-of-truth for the §4.16 G24-B EXIT CRITERION \
              arm. Splitting across helpers would obscure the test's \
              would-FAIL-if-no-op'd narrative."
)]
fn admin_ui_v0_workflow_editor_substantive_replay_via_real_engine_round_trip() {
    // ------------------------------------------------------------------
    // (1) PRODUCTION ARM — `compile_draft_within_manifest_envelope`
    //     drives the real T1 + T4 envelope defenses. We get back a
    //     compiled `Subgraph` whose primitives carry derived cap-
    //     scopes inside the manifest envelope.
    // ------------------------------------------------------------------
    let manifest = fixture_manifest_for_test(&["read:Note.*", "write:Note.*", "transform:Note.*"]);
    let draft = build_substantive_workflow_draft();
    let sg_save = compile_draft_within_manifest_envelope(&draft, &manifest)
        .expect("admissible workflow draft compiles under matching manifest envelope");

    // Save-time canonical bytes + content hash. This is the truth the
    // round-trip must preserve.
    let bytes_save =
        canonical_subgraph_bytes_for_test(&sg_save).expect("save-time canonical encoding succeeds");
    let hash_save = workflow_content_hash(&sg_save).expect("save-time content hash succeeds");
    let cid_save = sg_save.cid().expect("save-time subgraph CID computes");

    // Independent path: blake3 over canonical bytes MUST agree with
    // `workflow_content_hash` (defense-in-depth on the inline canary's
    // surface).
    let blake3_of_bytes = *blake3::hash(&bytes_save).as_bytes();
    assert_eq!(
        blake3_of_bytes, hash_save,
        "two independent BLAKE3 paths over the same canonical bytes \
         MUST agree — encoder + hasher single-source-of-truth"
    );

    // ------------------------------------------------------------------
    // (2) Engine wiring — open the AdminUiV0TestHarness's composed
    //     engine. The harness gives us a real GrantBackedPolicy engine
    //     + an admin-UI-plugin-DID principal CID (the principal the
    //     Class B β `read_node_as` walks under).
    // ------------------------------------------------------------------
    let harness = AdminUiV0TestHarness::new();
    let admin_ui_principal = harness.admin_ui_plugin_principal_cid();

    // ------------------------------------------------------------------
    // (3) PERSIST through `Engine::register_subgraph` — exercises the
    //     engine's handler-version durable persist path (one
    //     `system:HandlerVersion` zone Node written via the
    //     privileged write seam). The handler-table CID MUST match
    //     `sg_save.cid()`.
    // ------------------------------------------------------------------
    let handler_id = harness
        .engine()
        .register_subgraph(sg_save.clone())
        .expect("register_subgraph admits the compiled workflow subgraph");
    let chain = harness.engine().handler_version_chain(&handler_id);
    assert_eq!(
        chain.len(),
        1,
        "register_subgraph on fresh handler MUST leave a single-entry \
         version chain; got {} entries",
        chain.len()
    );
    assert_eq!(
        chain[0], cid_save,
        "handler-version-chain head MUST equal Subgraph::cid() — any \
         drift between engine-registered CID and the canonical-bytes \
         hash surfaces here (would-FAIL-if-no-op'd: a parallel \
         encoder hiding inside the engine would produce a different \
         CID and this assertion fires)"
    );

    // ------------------------------------------------------------------
    // (4) PERSIST a content-Node envelope carrying the canonical
    //     bytes through `Engine::create_node` (Inv-11 user-facing
    //     write path + transactional ChangeEvent fan-out). The
    //     returned CID is the address we'll round-trip through the
    //     Class B β read seam.
    // ------------------------------------------------------------------
    let mut workflow_node_props: BTreeMap<String, Value> = BTreeMap::new();
    workflow_node_props.insert(
        WORKFLOW_SUBGRAPH_BYTES_PROPERTY.to_string(),
        Value::Bytes(bytes_save.clone()),
    );
    workflow_node_props.insert(
        "save_handler_id".to_string(),
        Value::Text(handler_id.clone()),
    );
    let workflow_node = Node::new(
        vec![ADMIN_UI_WORKFLOW_NODE_LABEL.to_string()],
        workflow_node_props,
    );
    let persisted_cid: Cid = harness
        .create_test_node(&workflow_node)
        .expect("create_node persists the workflow envelope Node");

    // ------------------------------------------------------------------
    // (5) GRANT the admin-UI principal read coverage for the
    //     envelope's label, then drive a real `read_node_as` walk.
    //     This is the Class B β seam (CLAUDE.md #18 cag-r1-9) —
    //     `Engine::read_node` is `pub(crate)`; the public `_as`
    //     surface is the only path attributed reads should travel.
    // ------------------------------------------------------------------
    let read_scope = format!("store:{ADMIN_UI_WORKFLOW_NODE_LABEL}:read");
    harness
        .grant_admin_ui_read_scope(&read_scope)
        .expect("admin-UI principal granted read scope for workflow envelope label");
    let reloaded_node = harness
        .engine()
        .read_node_as(&admin_ui_principal, &persisted_cid)
        .expect("read_node_as backend call surfaces successfully")
        .expect(
            "Class B β read_node_as MUST surface the persisted Node \
             when the admin-UI principal holds the matching grant; a \
             `None` return would indicate either an Inv-11 short-circuit \
             (the envelope label is NOT a system-zone label) or a \
             cap-policy denial (the grant we just minted is for the \
             matching `store:<label>:read` derivation)",
        );
    assert_eq!(
        reloaded_node.labels.first().map(String::as_str),
        Some(ADMIN_UI_WORKFLOW_NODE_LABEL),
        "round-tripped Node preserves its primary label"
    );

    // ------------------------------------------------------------------
    // (6) OBSERVABLE CONSEQUENCE — extract the canonical-bytes
    //     envelope from the reloaded Node + drive the full
    //     decode → reconstruct → re-encode → re-hash chain.
    // ------------------------------------------------------------------
    let reloaded_bytes = match reloaded_node
        .properties
        .get(WORKFLOW_SUBGRAPH_BYTES_PROPERTY)
    {
        Some(Value::Bytes(bytes)) => bytes.clone(),
        other => panic!(
            "expected Value::Bytes at `{WORKFLOW_SUBGRAPH_BYTES_PROPERTY}` \
             on the reloaded Node; got {other:?}"
        ),
    };
    // Byte-for-byte equality on the raw envelope: the engine + redb
    // round-trip MUST preserve the canonical-bytes payload exactly.
    // A redb encoding that mutated the payload (compression / framing
    // / chunking-without-reverse-path) would surface here.
    assert_eq!(
        reloaded_bytes, bytes_save,
        "reloaded canonical-bytes envelope MUST byte-equal save-time \
         bytes — any encoder/decoder drift between create_node and \
         read_node_as surfaces here (this is the strongest \
         would-FAIL-if-no-op'd arm)"
    );

    // Decode the reloaded bytes into a fresh `Subgraph`. This walks
    // the inverse of `canonical_subgraph_bytes` — the DAG-CBOR decode
    // path the `Subgraph::load_verified` API exposes. A decoder bug
    // here would surface as a decode error.
    let sg_reloaded = Subgraph::load_verified(&reloaded_bytes)
        .expect("canonical bytes round-trip through Subgraph::load_verified");

    // Re-encode + re-hash the reloaded subgraph. This is the
    // substantive arm the §4.16 spec calls out: any drift between
    // the encoder and decoder (e.g., property-order normalisation
    // applied at encode but not at decode → re-encode) would surface
    // as a divergent hash.
    let bytes_replay = canonical_subgraph_bytes_for_test(&sg_reloaded)
        .expect("replay-time canonical encoding succeeds");
    let hash_replay =
        workflow_content_hash(&sg_reloaded).expect("replay-time content hash succeeds");
    let cid_replay = sg_reloaded
        .cid()
        .expect("replay-time subgraph CID computes");

    // Byte-for-byte AND hash-for-hash AND CID-for-CID equality.
    // Three checks at increasing levels of abstraction — any single
    // failure is a substantive correctness violation.
    assert_eq!(
        bytes_replay, bytes_save,
        "re-encoded canonical bytes MUST byte-equal save-time bytes \
         (encoder ↔ decoder round-trip preserves the byte-shape)"
    );
    assert_eq!(
        hash_replay, hash_save,
        "replay-time `workflow_content_hash` MUST equal save-time \
         hash — the G24-B EXIT CRITERION substantive arm"
    );
    assert_eq!(
        cid_replay, cid_save,
        "replay-time `Subgraph::cid()` MUST equal save-time CID — \
         catches drift between the `_content_hash` helper and the \
         `Cid::from_blake3_digest(to_canonical_bytes)` engine path"
    );

    // ------------------------------------------------------------------
    // (7) Defense-in-depth: the reloaded subgraph's handler_id +
    //     primitive-count + edge-count MUST match the saved form.
    //     This catches the failure shape where the encoder/decoder
    //     happen to produce the same hash via collision but the
    //     structural shape differs (vanishingly improbable with
    //     BLAKE3 but cheap to assert; pins canonical encoding's
    //     structural invariants).
    // ------------------------------------------------------------------
    assert_eq!(
        sg_reloaded.handler_id(),
        sg_save.handler_id(),
        "reloaded handler_id preserved"
    );
    assert_eq!(
        sg_reloaded.nodes().len(),
        sg_save.nodes().len(),
        "reloaded primitive-count preserved"
    );
    assert_eq!(
        sg_reloaded.edges().len(),
        sg_save.edges().len(),
        "reloaded edge-count preserved"
    );
}
