//! G16-B wave-6b LANDED — Loro rich-type merge correctness pin.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row `loro_rich_type_merge_correctness`.
//! - plan §3 G16-B row line "rich-type Loro merge for collaborative
//!   subgraph edits".
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `LoroDoc::list` API + asserts OBSERVABLE
//! behavioral consequence (concurrent inserts both preserved across
//! peers after bidirectional merge).

#![allow(clippy::unwrap_used)]

use benten_sync::crdt::LoroDoc;

#[test]
fn loro_rich_type_merge_correctness() {
    let doc_a = LoroDoc::new();
    let doc_b = LoroDoc::new();
    doc_a.list("comments").insert(0, "first").unwrap();
    doc_b.list("comments").insert(0, "alt-first").unwrap();
    // Concurrent writes — bidirectional merge.
    doc_a.merge(&doc_b).unwrap();
    doc_b.merge(&doc_a).unwrap();
    // Both writes are preserved (Loro List CRDT semantics).
    assert_eq!(doc_a.list("comments").len(), 2);
    assert_eq!(doc_b.list("comments").len(), 2);
}
