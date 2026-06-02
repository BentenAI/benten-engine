//! F-AUDIT-4 (R3-W6 gov-audit) — `audit_log_query` is GRAPH-NATIVE: it
//! composes the 12 primitives + an `audit:<set_id>:*` RestrictedScope, with
//! NO new frozen op signature. The frozen op-surface = EXACTLY the 12
//! `PrimitiveKind` variants (`crates/benten-core/src/subgraph.rs:69-93`);
//! the "24 ops" framing is RETRACTED (NQ-W3, M-17, CLAUDE.md #1).
//!
//! Pin sources (F-full R2 test-landscape §1 Group 11 row F-AUDIT-4; merges
//! K2-M17 + T-G4 + GNI-17, NQ-W3):
//!   - R0.3 plan §1.5 M-17, §4.2, NQ-W3, CLAUDE.md baked-in #1.
//!   - Clone-shape:
//!     `crates/benten-core/tests/tf3w_walker_is_a_subgraph_no_new_primitive_kind.rs`
//!     (12-variant canonical-tag round-trip; the "no 13th primitive"
//!     property the audit query MUST also obey).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! The W6 `audit_log_query` composition surface does not exist at this SHA.
//! Self-contained stub-shim compiles green; bodies `unimplemented!()`.
//! W6 R5 implementer:
//!   1. DELETE `mset_w6_audit_query_stub`,
//!   2. INSERT `use benten_membership_set::audit::audit_log_query_composition;`,
//!   3. UN-IGNORE the stub-driven tests,
//!   4. Verify green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext)
//!
//! Two arms: (1) the REAL `PrimitiveKind` 12-variant canonical-tag
//! round-trip + EXACTLY-12 count — `#[test]` (green now), the frozen
//! op-surface=12 pin (NQ-W3) the audit query must not grow; (2) the
//! stub-driven `audit_log_query` composition routes through READ + an
//! `audit:<set_id>:*` RestrictedScope and is NOT itself a `PrimitiveKind`
//! variant (it mints no 13th primitive).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use benten_core::PrimitiveKind;

// =====================================================================
// RED-PHASE stub-shim — DELETE at W6 implementation; replace with:
//     use benten_membership_set::audit::{
//         AuditQueryComposition, audit_log_query_composition,
//     };
// =====================================================================
mod mset_w6_audit_query_stub {
    //! Local stub matching the intended W6 `audit_log_query` composition
    //! surface. Bodies `unimplemented!()`.

    /// The composition the `audit_log_query` resolves to: the ordered set
    /// of canonical PrimitiveKind tags it walks + the RestrictedScope
    /// string it gates on. Real impl returns the actual composition the
    /// engine evaluator walks.
    #[derive(Clone, Debug)]
    pub struct AuditQueryComposition {
        /// Canonical tags of the primitives composing the query
        /// (e.g. READ + BRANCH + RESPOND). Each MUST be one of the 12.
        pub primitive_tags: Vec<&'static str>,
        /// The `audit:<set_id>:*` scope the query gates on.
        pub gating_scope: String,
        /// Whether `audit_log_query` is itself a NEW PrimitiveKind variant
        /// (real impl: `false` — it composes existing primitives).
        pub is_a_new_primitive_kind_variant: bool,
    }

    /// W6 stub: resolve the graph-native `audit_log_query` composition for
    /// a given set. Real impl composes READ over the audit version-chain +
    /// scope gate, NO new frozen op.
    pub fn audit_log_query_composition(_set_id: &[u8; 32]) -> AuditQueryComposition {
        unimplemented!(
            "W6 stub — R5 replaces this module with \
             `use benten_membership_set::audit::audit_log_query_composition;`; \
             real impl composes the 12 primitives + audit:<set_id>:* scope, \
             minting NO 13th PrimitiveKind"
        )
    }
}

use mset_w6_audit_query_stub::{AuditQueryComposition, audit_log_query_composition};

/// F-AUDIT-4 (a): the frozen op-surface is EXACTLY 12 `PrimitiveKind`
/// variants, and all 12 round-trip their canonical tags. This is the
/// NQ-W3 "frozen-op=12" pin — `#[test]` (green now), load-bearing against
/// the retracted "24 ops" framing. `audit_log_query` must compose within
/// this set; it cannot grow it.
#[test]
fn frozen_op_surface_is_exactly_twelve_primitive_kinds() {
    let known: &[&str] = &[
        "READ", "WRITE", "TRANSFORM", "BRANCH", "ITERATE", "WAIT", "CALL", "RESPOND", "EMIT",
        "SANDBOX", "SUBSCRIBE", "STREAM",
    ];
    assert_eq!(
        known.len(),
        12,
        "F-AUDIT-4 (NQ-W3): the frozen op-surface is EXACTLY 12 — the '24 ops' \
         framing is RETRACTED (M-17)"
    );

    let twelve = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ];
    assert_eq!(twelve.len(), 12, "12 known PrimitiveKind variants constructed");

    for k in &twelve {
        let tag = k.canonical_tag();
        assert!(
            known.contains(&tag),
            "F-AUDIT-4: PrimitiveKind {k:?} ({tag}) MUST be one of the 12 \
             baseline kinds (rename/typo/dispatch-bug defense; CLAUDE.md #1)"
        );
    }
    let emitted: Vec<&'static str> = twelve.iter().map(|k| k.canonical_tag()).collect();
    for tag in known {
        assert!(
            emitted.contains(tag),
            "F-AUDIT-4: canonical tag {tag} MUST be emitted by one of the 12 \
             variants — the op-surface is closed at 12"
        );
    }
}

/// F-AUDIT-4 (b): `audit_log_query` is GRAPH-NATIVE — it composes ONLY the
/// 12 existing primitives (each tag ∈ the 12-set) and is NOT itself a new
/// `PrimitiveKind` variant.
///
/// would-FAIL if W6 mints a 13th primitive (e.g. a bespoke `AuditQuery`
/// op) instead of composing READ + scope.
#[test]
#[ignore = "RED-PHASE: F-AUDIT-4 — audit_log_query composes the 12 primitives, no new frozen op; un-ignore at W6 R5 (delete mset_w6_audit_query_stub; insert real `use`)"]
fn audit_log_query_composes_existing_primitives_no_new_frozen_op() {
    let twelve_tags: [&str; 12] = [
        "READ", "WRITE", "TRANSFORM", "BRANCH", "ITERATE", "WAIT", "CALL", "RESPOND", "EMIT",
        "SANDBOX", "SUBSCRIBE", "STREAM",
    ];

    let composition: AuditQueryComposition = audit_log_query_composition(&[0x51; 32]);

    assert!(
        !composition.is_a_new_primitive_kind_variant,
        "F-AUDIT-4: `audit_log_query` MUST NOT be a new PrimitiveKind variant — \
         it composes the existing 12 primitives (NQ-W3 / CLAUDE.md #1); \
         would-FAIL if W6 mints a 13th primitive"
    );
    assert!(
        !composition.primitive_tags.is_empty(),
        "F-AUDIT-4: the composition MUST name the primitives it walks (at \
         minimum a READ over the audit version-chain)"
    );
    for tag in &composition.primitive_tags {
        assert!(
            twelve_tags.contains(tag),
            "F-AUDIT-4: every primitive in the audit_log_query composition MUST \
             be one of the 12 frozen kinds; offending tag = {tag}"
        );
    }
    assert!(
        composition.primitive_tags.contains(&"READ"),
        "F-AUDIT-4: `audit_log_query` MUST route through READ (it reads the \
         audit version-chain) rather than a bespoke audit op"
    );
}

/// F-AUDIT-4 (c): the query gates on an `audit:<set_id>:*` RestrictedScope
/// (F-AUDIT-3 coupling) — the access control is a scope, never a codepoint
/// or a new op signature.
#[test]
#[ignore = "RED-PHASE: F-AUDIT-4 — audit_log_query gates on audit:<set_id>:* scope; un-ignore at W6 R5"]
fn audit_log_query_gates_on_audit_set_scope() {
    let composition: AuditQueryComposition = audit_log_query_composition(&[0x51; 32]);
    assert!(
        composition.gating_scope.starts_with("audit:"),
        "F-AUDIT-4: `audit_log_query` MUST gate on an `audit:<set_id>:*` \
         RestrictedScope (couples F-AUDIT-3); got scope = {}",
        composition.gating_scope
    );
    assert!(
        composition.gating_scope.contains(":*") || composition.gating_scope.contains(":read"),
        "F-AUDIT-4: the audit gating scope MUST be the read-side audit scope \
         family; got {}",
        composition.gating_scope
    );
}
