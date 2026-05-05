//! R3-C RED-PHASE pin: Loro rich-type merge correctness (G16-B
//! wave-6b; per r2-test-landscape §2.4 G16-B + plan §3 G16-B row).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row `loro_rich_type_merge_correctness`.
//! - plan §3 G16-B row line "rich-type Loro merge for collaborative
//!   subgraph edits".
//!
//! ## What this pins
//!
//! Loro supports rich types beyond simple LWW values: Loro Lists
//! (collaborative arrays), Loro Maps (collaborative key-value), Loro
//! Text (collaborative strings with intent-preservation). Phase-3
//! G16-B integrates rich-type merge for collaborative subgraph
//! edits where multiple peers concurrently modify a node's rich
//! property values.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b lands rich-type Loro merge"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — plan §3 G16-B — rich-type merge correctness"]
fn loro_rich_type_merge_correctness() {
    // plan §3 G16-B pin. G16-B implementer wires this against the
    // chosen rich-type subset (Lists + Maps + Text TBD at
    // implementation time):
    //
    //   use benten_sync::crdt::{LoroDoc, LoroList};
    //   let doc_a = LoroDoc::new();
    //   let doc_b = LoroDoc::new();
    //   doc_a.list("comments").push_back("first").unwrap();
    //   doc_b.list("comments").push_back("alt-first").unwrap();
    //   // Concurrent writes:
    //   doc_a.merge(&doc_b).unwrap();
    //   doc_b.merge(&doc_a).unwrap();
    //   // Both writes are preserved (List CRDT semantics):
    //   let list_a: Vec<_> = doc_a.list("comments").iter().collect();
    //   let list_b: Vec<_> = doc_b.list("comments").iter().collect();
    //   assert_eq!(list_a.len(), 2);
    //   assert_eq!(list_a, list_b);  // same canonical order across peers
    //
    // OBSERVABLE consequence: rich-type Loro structures preserve
    // both concurrent edits + converge on the same canonical order
    // across peers. Defends against the failure shape where rich
    // type merges silently lose one peer's writes.
    unimplemented!("G16-B wires Loro rich-type merge correctness assertion");
}
