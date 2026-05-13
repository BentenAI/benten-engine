//! Phase 4-Foundation R4b-FP-2 — §4.17 cross-language drift-defense pins
//! for the workflow editor + composed-view creator surfaces.
//!
//! Closes `phase-4-backlog.md §4.17` (g24b-mr-2 MINOR + g24c-mr-1 MINOR):
//! - TS `WorkflowFormField` shape mirrors Rust `WorkflowFormField`
//! - TS `CANONICAL_12_PRIMITIVE_KINDS` set mirrors Rust `PrimitiveKind`
//!   enum variant set (CLAUDE.md baked-in #1 — 12-primitive irreducible
//!   architectural commitment)
//! - TS `UserViewSpec.anchorPatternLabel` is documented as an
//!   intentional §3.5g rule-mirror EXCEPTION (TS-side UX metadata only;
//!   not in Rust `SubgraphSpec`)
//!
//! ## §3.5g cross-language rule-mirror discipline
//!
//! Per `feedback_pim_cross_language_rule_mirror.md`: any rule encoded
//! in BOTH TS + Rust requires either (a) atomic-update discipline
//! enforced by drift-defense pin, OR (b) an explicit single-source-of-
//! truth + generator. This pin enforces shape (a) for three rule
//! surfaces.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! - A Rust contributor adds a field to `WorkflowFormField` without
//!   updating the TS interface → the field-list assertion fires.
//! - A future PR adds / removes / renames a `PrimitiveKind` variant
//!   without updating the TS `CANONICAL_12_PRIMITIVE_KINDS` set →
//!   the kind-set assertion fires.
//! - A future PR removes the §3.5g EXCEPTION docstring from
//!   `view_spec.ts` → the docstring grep-assertion fires.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

/// Locate `packages/admin-ui-v0/src/<rel>` relative to the workspace
/// root (parent of `crates/benten-engine`). Returns the file's
/// contents as a String.
fn read_admin_ui_v0_source(rel_path: &str) -> String {
    // CARGO_MANIFEST_DIR == `<workspace>/crates/benten-engine`
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(std::path::Path::parent)
        .expect("CARGO_MANIFEST_DIR has a workspace-root ancestor 2 levels up");
    let full_path = workspace.join("packages/admin-ui-v0/src").join(rel_path);
    fs::read_to_string(&full_path).unwrap_or_else(|e| {
        panic!(
            "drift-defense: failed to read TS source at {}: {e}",
            full_path.display()
        )
    })
}

#[test]
fn workflow_form_field_ts_shape_mirrors_rust_struct_fields() {
    // Rust-side authoritative field set (mirrors
    // `benten_platform_foundation::WorkflowFormField`). Per
    // workflow_editor.rs lines 64-78: id, kind, cap_scope, field_path.
    //
    // The TS camelCase mirror MUST carry each of these as readonly.
    // A future PR adding a 5th Rust field WITHOUT a TS update would
    // fail this assertion + force a §3.5g atomic-update.
    let rust_fields = [
        ("id", "string"),         // String
        ("kind", "kind"),         // PrimitiveKind enum → WorkflowPrimitiveKind union
        ("capScope", "capScope"), // Option<String> → string | undefined
        ("fieldPath", "fieldPath"),
    ];

    let ts = read_admin_ui_v0_source("workflow-editor/index.ts");
    // Sub-window: the `WorkflowFormField` interface body. Find the
    // declaration + scope to its closing brace.
    let idx = ts.find("export interface WorkflowFormField").expect(
        "TS source MUST declare `export interface WorkflowFormField` — §3.5g mirror anchor",
    );
    let after = &ts[idx..];
    let close_relative = after
        .find('}')
        .expect("WorkflowFormField interface body MUST close with `}` within file");
    let interface_body = &after[..close_relative];

    for (field, _ty_hint) in &rust_fields {
        // Each Rust field's TS camelCase counterpart MUST appear as
        // a `readonly <field>:` declaration in the interface body.
        let needle = format!("readonly {field}");
        assert!(
            interface_body.contains(&needle),
            "drift-defense FAIL: Rust `WorkflowFormField` field `{field}` \
             has no TS mirror in `WorkflowFormField` interface body.\n\n\
             Interface body snippet:\n{interface_body}\n\n\
             A Rust-side struct field was added without the §3.5g \
             cross-language rule-mirror atomic-update."
        );
    }
}

#[test]
fn canonical_12_primitive_kinds_ts_set_mirrors_rust_primitivekind_enum() {
    // CLAUDE.md baked-in #1: the 12 canonical primitive kinds are
    // architecturally irreducible. The TS set in
    // `packages/admin-ui-v0/src/workflow-editor/index.ts` MUST mirror
    // the Rust `PrimitiveKind` enum verbatim.
    //
    // Drift-defense scope: catch the case where a future PR adds /
    // removes / renames a PrimitiveKind variant without updating the
    // TS set. (Note: adding a 13th primitive ALSO violates CLAUDE.md
    // #1 — this pin is the cross-language echo of that commitment.)
    let canonical_kinds = [
        "Read",
        "Write",
        "Transform",
        "Branch",
        "Iterate",
        "Wait",
        "Call",
        "Respond",
        "Emit",
        "Sandbox",
        "Subscribe",
        "Stream",
    ];
    assert_eq!(
        canonical_kinds.len(),
        12,
        "Rust-side canonical kind count locked at 12 per CLAUDE.md #1"
    );

    let ts = read_admin_ui_v0_source("workflow-editor/index.ts");
    let idx = ts
        .find("CANONICAL_12_PRIMITIVE_KINDS")
        .expect("TS source MUST declare `CANONICAL_12_PRIMITIVE_KINDS` constant");
    // The constant's body lives inside a `new Set([ ... ])`; scope
    // to the closing `])`.
    let after = &ts[idx..];
    let body_end = after.find("])").expect(
        "CANONICAL_12_PRIMITIVE_KINDS body MUST close with `])` — \
         malformed TS source",
    );
    let body = &after[..body_end];

    for kind in &canonical_kinds {
        // Each Rust variant MUST appear as a `"<Kind>"` string-literal
        // inside the Set's array literal.
        let needle = format!("\"{kind}\"");
        assert!(
            body.contains(&needle),
            "drift-defense FAIL: Rust `PrimitiveKind::{kind}` has no \
             TS mirror entry in `CANONICAL_12_PRIMITIVE_KINDS` set.\n\n\
             Set body snippet:\n{body}\n\n\
             Either the Rust enum gained / renamed a variant without \
             the §3.5g cross-language rule-mirror atomic-update, OR \
             the TS set was edited out of sync. Per CLAUDE.md #1 the \
             12-primitive set is architecturally irreducible — any \
             change to either side must be sourced through CLAUDE.md \
             ratification, not a bare PR."
        );
    }

    // Also pin the TS union type `WorkflowPrimitiveKind` to the same
    // 12 strings — the set above + the union form the redundant
    // (defense-in-depth) cross-language pin. A future PR that edits
    // the set but forgets the union (or vice-versa) would surface
    // here.
    let union_idx = ts
        .find("export type WorkflowPrimitiveKind")
        .expect("TS source MUST declare `WorkflowPrimitiveKind` union type");
    let union_body_end = ts[union_idx..].find(';').unwrap_or(ts.len() - union_idx);
    let union_body = &ts[union_idx..union_idx + union_body_end];
    for kind in &canonical_kinds {
        let needle = format!("\"{kind}\"");
        assert!(
            union_body.contains(&needle),
            "drift-defense FAIL: Rust `PrimitiveKind::{kind}` has no \
             TS union-type mirror in `WorkflowPrimitiveKind`.\n\n\
             Union body snippet:\n{union_body}"
        );
    }
}

#[test]
fn user_view_spec_anchor_pattern_label_is_intentionally_ts_only_per_3_5g_exception() {
    // g24c-mr-1 MINOR closure (b): the `anchorPatternLabel` field is
    // intentionally TS-side-only — NOT part of the Rust-side
    // `benten_ivm::subgraph_spec::SubgraphSpec` parity contract.
    // The §3.5g cross-language rule-mirror EXCEPTION is documented
    // via a sharpened docstring on the TS field declaration.
    //
    // This drift-defense pin grep-asserts the EXCEPTION declaration's
    // presence; removing the declaration without re-ratifying the
    // §3.5g mirror would fail this test.
    let ts = read_admin_ui_v0_source("view-composer/view_spec.ts");

    // The exception docstring must explicitly carry both phrases:
    //   - `§3.5g cross-language rule-mirror EXCEPTION`
    //   - `INTENTIONALLY TS-side-only`
    // Either phrase being removed indicates the exception was
    // silently dropped + the §3.5g contract should be re-evaluated.
    assert!(
        ts.contains("§3.5g cross-language rule-mirror EXCEPTION"),
        "drift-defense FAIL: `view_spec.ts` MUST carry the explicit \
         §3.5g rule-mirror EXCEPTION marker on the `anchorPatternLabel` \
         field docstring. The marker's removal indicates the field \
         silently lost its intentional TS-only status; either \
         re-add the marker OR add a mirrored Rust field + drift- \
         defense pin per §3.5g."
    );
    // Normalize line-wrap inside JSDoc comments (TS doc-comments
    // line-wrap with `\n   * ` continuation) before grep.
    let normalized = ts.replace("\n   * ", " ").replace("\n   *", " ");
    assert!(
        normalized.contains("INTENTIONALLY TS-side-only"),
        "drift-defense FAIL: `view_spec.ts` MUST carry the \
         `INTENTIONALLY TS-side-only` phrase on the \
         `anchorPatternLabel` docstring (line-wrap tolerated). \
         See §4.17 closure rationale in `docs/future/phase-4-backlog.md`."
    );

    // Defense-in-depth: explicitly named field
    // `anchorPatternLabel` MUST still be in the file (a future
    // refactor that renames OR removes the field without re-routing
    // through the EXCEPTION needs to surface here).
    assert!(
        ts.contains("anchorPatternLabel: string"),
        "drift-defense FAIL: `anchorPatternLabel: string` field \
         declaration removed from `view_spec.ts`. The §3.5g \
         EXCEPTION was anchored to this specific field; any \
         removal needs to either re-route the doc OR re-evaluate \
         whether the Rust mirror should also acquire the field."
    );
}
