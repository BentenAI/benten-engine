// G24-C wave-6b — UserViewSpec TypeScript shape.
//
// Mirrors `benten_ivm::subgraph_spec::SubgraphSpec` (the Rust-side
// canonical kernel-input shape introduced at G23-0a). The admin UI v0
// composed-view creator emits values of this shape; the bridge
// forwards them across napi/wasm to the engine, where the user-view
// branch of `Algorithm::register_subgraph` accepts them.
//
// **D-4F-2 commitment:** the composed-view creator does NOT carry an
// internal "view recomputation" module — view materialization flows
// through the generalized IVM Algorithm B kernel, NOT through a
// parallel admin-ui-internal path. This file is the schema-shaped
// view-definition value the creator surfaces to the bridge.
//
// **Cross-language rule-mirror (§3.5g):** when fields are added to the
// Rust-side `SubgraphSpec`, this file MUST be updated atomically. The
// Rust-side test pin
// `admin_ui_v0_composed_view_creator_emits_subgraph_shaped_view_via_generalized_ivm.rs`
// asserts the spec round-trips through `Algorithm::register_subgraph`
// with no shape mismatch.

/**
 * Label-pattern selector mirror — narrowed to the two shapes the
 * generalized Algorithm B kernel admits per `benten_ivm::algorithm_b`.
 */
export type LabelPattern =
  | { readonly kind: "exact"; readonly label: string }
  | { readonly kind: "anchor_prefix"; readonly prefix: string };

/**
 * Typed-output projection mirror (`TypedOutputProjection` in the Rust
 * kernel). User-defined views ALWAYS leave this unset — only the
 * canonical views 4 (governance_inheritance → `Rules`) + 5
 * (version_current → `Current`) carry non-null values; the
 * `Algorithm::register_subgraph` guard rejects a mis-declared shape
 * with `TypedOutputProjectionMismatch` per g23-0a-mr-3.
 */
export type TypedOutputProjection = null | "rules" | "current";

/**
 * Subgraph-shaped view definition the composed-view creator emits.
 *
 * This is the **kernel-input shape per D-4F-2**: a user-defined view
 * authored via the admin UI is structurally a `UserViewSpec` value
 * that the engine passes through `Algorithm::register_subgraph`. The
 * field set mirrors the Rust-side `SubgraphSpec` 1:1.
 *
 * Stability contract: additive evolution only. Removing or renaming a
 * field breaks the §3.5g cross-language rule-mirror.
 */
export interface UserViewSpec {
  /** Stable view id (user-supplied; must NOT collide with canonical ids). */
  readonly viewId: string;
  /** Label-pattern selector — narrows which Nodes contribute. */
  readonly labelPattern: LabelPattern;
  /** Output projection — fields of the source Nodes to project. */
  readonly projection: ReadonlyArray<string>;
  /**
   * Always `null` for user-defined views (only canonical views 4/5
   * carry a typed-output variant per Algorithm B's register-time guard).
   */
  readonly typedOutputProjection: TypedOutputProjection;
  /**
   * Self-reference flag (`mat-r1-13` fail-fast). Must be `false` for
   * user-defined views; the kernel rejects `true` with
   * `SelfReferentialSubgraphRejected` at register time.
   */
  readonly selfReferential: boolean;
  /**
   * Optional per-update budget cap. `null` ⇒ unbounded (kernel uses
   * saturating arithmetic); `<positive integer>` ⇒ cap.
   */
  readonly budget: number | null;
  /**
   * Anchor pattern label the creator was driven by.
   *
   * **§3.5g cross-language rule-mirror EXCEPTION — INTENTIONALLY
   * TS-side-only.** This field is **NOT** part of the Rust-side
   * `benten_ivm::subgraph_spec::SubgraphSpec` parity contract. It is
   * UX-side metadata used by the admin UI v0 composed-view creator
   * for display + traceability in the persisted view subgraph; the
   * generalized Algorithm B kernel in `benten-ivm` only consumes
   * `labelPattern`. The bridge round-trips this field back through
   * the view's persisted Node as opaque metadata.
   *
   * **Future-agent contract (closes g24c-mr-1 MINOR / §4.17):** do
   * NOT add a mirrored field to the Rust `SubgraphSpec` to "fix the
   * mirror." This deliberate asymmetry is the contract — the kernel
   * does not need this field, and adding it would broaden the
   * kernel's input surface without semantic gain. Drift-defense
   * pin `crates/benten-engine/tests/workflow_editor_cross_language_drift_defense.rs`
   * grep-asserts THIS docstring's presence so that any future
   * removal of the exception declaration also fails the parity test.
   */
  readonly anchorPatternLabel: string;
}

/**
 * Constructs a `UserViewSpec` for a user-authored view. Defaults to
 * `LabelPattern::Exact(anchorPattern)` + `selfReferential=false` +
 * unbounded budget — the safe-by-construction shape the kernel admits
 * without further fail-loud guards.
 */
export function userViewSpec(args: {
  readonly viewId: string;
  readonly anchorPattern: string;
  readonly projection: ReadonlyArray<string>;
}): UserViewSpec {
  if (args.viewId.length === 0) {
    throw new Error("userViewSpec: viewId must be non-empty");
  }
  if (args.anchorPattern.length === 0) {
    throw new Error("userViewSpec: anchorPattern must be non-empty");
  }
  if (args.projection.length === 0) {
    throw new Error(
      "userViewSpec: projection must select at least one field",
    );
  }
  // Reject collision with the canonical view ids per Rust-side
  // `SubgraphSpec::user_view`. Kept in sync with
  // `benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS`.
  const CANONICAL_VIEW_IDS = [
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
  ];
  if (CANONICAL_VIEW_IDS.includes(args.viewId)) {
    throw new Error(
      `userViewSpec: \`${args.viewId}\` is a canonical view id; pick a user-scoped id`,
    );
  }
  return {
    viewId: args.viewId,
    labelPattern: { kind: "exact", label: args.anchorPattern },
    projection: [...args.projection],
    typedOutputProjection: null,
    selfReferential: false,
    budget: null,
    anchorPatternLabel: args.anchorPattern,
  };
}
