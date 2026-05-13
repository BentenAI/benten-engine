//! G23-B GREEN: dual-gate per-row check independent of delivery
//! (LOAD-BEARING; mirrors `ivm_read_gate.rs::ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`).
//!
//! Closes r2-test-landscape §2.5 row 4 + sec-3.5-r1-1 dual-gate
//! composition pin 1 of 4.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_platform_foundation::{HtmlJsonMaterializer, Materializer, MaterializerCapRecheck};
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn materializer_per_row_gate_independent_of_delivery() {
    // 10 rows: 5 public + 5 private. Per-row gate admits ONLY public.
    let mut public = Vec::new();
    let mut private = Vec::new();
    for i in 0..5 {
        public.push(
            materializer_fixtures::make_post_row_node("public", i)
                .cid()
                .unwrap(),
        );
        private.push(
            materializer_fixtures::make_post_row_node("private", i)
                .cid()
                .unwrap(),
        );
    }
    let admitted_set: BTreeSet<_> = public.iter().copied().collect();
    let admitted_arc = Arc::new(admitted_set);
    let cap_recheck: MaterializerCapRecheck = {
        let set = Arc::clone(&admitted_arc);
        Arc::new(move |_p, _zone, cid| set.contains(cid))
    };
    let alice = materializer_fixtures::actor_principal_alice_cid();

    let mat = HtmlJsonMaterializer;
    let all_rows: Vec<_> = public.iter().chain(private.iter()).copied().collect();
    let admitted = mat.filter_rows_at_materialization(all_rows, &alice, "post", &cap_recheck);

    // Pim-2 §3.6b would-FAIL-if-no-op'd shape: exactly 5 of 10 admitted
    // (NOT 0 — gate-deny-all bypass; NOT 10 — gate-allow-all bypass).
    assert_eq!(
        admitted.len(),
        5,
        "materializer per-row gate admits exactly 5 of 10 (matches ivm_read_gate.rs shape); \
         independence: no SUBSCRIBE channel involved"
    );
    for cid in &admitted {
        assert!(
            admitted_arc.contains(cid),
            "every admitted row in public set"
        );
    }
}
