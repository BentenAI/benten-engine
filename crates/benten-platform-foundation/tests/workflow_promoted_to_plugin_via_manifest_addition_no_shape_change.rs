//! G24-D row pin — workflow ↔ plugin unification.
//!
//! Per CLAUDE.md #18 + post-R1-triage Q11: a workflow IS a plugin IS
//! a subgraph. Single shape, distinguished by scale + sharing intent
//! (manifest presence). Promoting a workflow to a plugin = adding a
//! manifest; NO fundamental shape change.
//!
//! Per R2 §5 substance discipline: substantive shape-change-not-needed
//! assertion (add manifest to workflow's existing subgraph; assert
//! subgraph CID changes MINIMALLY — only by the manifest-Node
//! addition).

mod common;

use common::manifest_fixtures::minimal_manifest;

#[test]
#[ignore = "RED-PHASE: G24-D wave wires workflow_to_plugin promotion seam; un-ignore at G24-D landing"]
fn promoting_workflow_to_plugin_via_manifest_addition_changes_cid_by_one_node_only() {
    let _manifest = minimal_manifest();

    // Future surface:
    //   workflow_to_plugin::promote(workflow_subgraph_cid: Cid,
    //     manifest: PluginManifest, engine: &mut Engine)
    //     -> Result<Cid>
    // Returns new subgraph CID with the manifest Node appended as a
    // sibling under the same root. The workflow's existing Node graph
    // is UNCHANGED.
    //
    // SUBSTANTIVE assertion (per R2 §5): diff old_cid vs new_cid; the
    // delta is exactly one new Node (the manifest); ALL existing
    // workflow Nodes retain their CIDs unchanged. FAILS-IF-NO-OP
    // because a naive implementation that re-builds the subgraph
    // would mutate child Node CIDs.
    panic!(
        "RED-PHASE: G24-D wave must wire workflow_to_plugin::promote with substantive shape-preservation invariant"
    );
}
