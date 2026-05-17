# `benten-dsl-compiler` — Internals

Plain-English deep-dive into the `benten-dsl-compiler` crate. Audience: a developer or AI agent who needs to understand what this crate is, why it exists, and what to expect when extending it. Read-only audit; no claims about Phase-4 plans beyond what is already pinned in the code or accompanying retrospective docs.

**Last refreshed against `8141b94` (Phase-4-Foundation tag-eve, 2026-05-14).** The crate is substantively unchanged since the initial deep-dive at `a9da0be` (2026-05-08 G20-B docs sweep); the only intervening edit was the Phase-3.5 → Phase-4-Foundation rename at `00f2784` (2026-05-11). Phase-4-Foundation R5+R6 work touched zero source files in this crate — the typed-CALL DSL surface added at Phase-3 G21-T2 (PR #148, `7a6c36a`) landed in the TS DSL + napi binding only, not in the Rust dsl-compiler grammar. See §8 for the deliberate placement rationale.

---

## 1. What this crate does

`benten-dsl-compiler` takes a short string of DSL text — a sequence of operation-primitive calls chained with `->` — and produces a canonical `benten_core::Subgraph` that the engine knows how to load and execute. It is the bridge between "I want to write a handler in a tiny English-shaped language" and "the engine wants a content-addressed graph of `OperationNode`s." The output round-trips: the bytes the engine consumes hash to the same CID whether the handler arrived through this compiler, the TypeScript DSL surface, or hand-built builder calls.

The crate is intentionally a runt. It was added in Phase-2b (Wave-6 / G12-B) to give the devserver a Rust-side way to compile inline DSL snippets without dragging in `benten-eval` or `benten-graph`. The whole crate is 894 LOC of `src/lib.rs` plus four integration tests totalling 287 LOC; the `Cargo.toml` description still labels it `MINIMAL-FOR-DEVSERVER ~200-300 LOC, 4 public items`, and while the LOC has grown past that estimate (mostly via R6 fp Wave C2's shape-validation pass), the four-public-item discipline is intact. The dependency direction is the load-bearing constraint: `benten-core` only, no engine or eval or graph.

---

## 2. Dependency chain

**Inbound (declared in `Cargo.toml`):**

- `benten-core` — for `Subgraph`, `OperationNode`, `Value`, and `PrimitiveKind`. The compiler's whole job is to construct values of those types from text.
- `benten-errors` — workspace-shared error infrastructure. Used minimally; thiserror does most of the local work.
- `thiserror` — derives `Error` for `CompileError`.
- No dev-dependencies. The integration tests use only `benten_core` and the crate's own public surface.

**Forbidden by `tests/arch_n_benten_dsl_compiler_dep_direction.rs`:**

- `benten-eval` — would invert the engine-uses-compiler relationship.
- `benten-graph` — same reason; compiler hands off shapes, doesn't reach into runtime graph state.
- `benten-engine` — sibling crate, not parent.

The four arch-N tests scan the `Cargo.toml` text directly (not the cargo dep graph) so the constraint is enforced at compile time and survives refactors that might add transitive paths.

**Consumers (where the compiler is called from):**

- `packages/engine-devserver/` — the original motivating consumer. The devserver compiles inline DSL strings from the dev UI into Subgraphs, registers them with the engine, and reports `Diagnostic`s back to the browser when authors typo.
- `crates/benten-eval/tests/sandbox_handler_args.rs` — does NOT depend on the crate at the cargo level (would break arch-1); it instead asserts the crate's source file exists at the expected path as an architectural drift-pin and cross-references the canonical-bytes-stability test by name.
- `crates/benten-engine/tests/no_dsl_compiler_dep.rs` — the inverse pin: asserts `benten-engine` does NOT depend on this crate. The devserver consumes the compiler directly, not via the engine.

Net shape: the compiler is a leaf consumer of `benten-core` and a sibling-not-parent of `benten-engine`. Devserver-shaped tooling consumes it. Nothing in the engine runtime path touches it.

---

## 3. Files inventory in `src/`

There is one file.

**`src/lib.rs` (894 lines)** — the entire crate. Logical sections, in order of appearance:

- **Crate-level docs (lines 1-83):** scope note, dep-direction reminder, the EBNF-shaped grammar block, and the deliberate-non-extensibility note. The grammar block is the canonical reference for what tokens the parser accepts; everything below this comment should be implementing that grammar and nothing more.
- **Public surface (lines 98-218):** `CompiledSubgraph`, `CompiledPrimitive`, `compile_str`, `compile_file`, `CompileError`, `Diagnostic`. The two functions are thin: trim-check, hand off to `Parser`, hand off to `emit`. All the complexity lives in `Parser` and `emit`.
- **Stable error codes (lines 221-236):** four `pub(crate)` `&'static str` constants. They are NOT exposed as a public enum; the constants are written through `Diagnostic::error_code`. `E_DSL_INVALID_SHAPE` is the late arrival (Phase-3 R6 fp Wave C2) mirroring the TS-side `EDslInvalidShape`.
- **AST (lines 239-254):** two `pub(crate)` structs — `HandlerAst` and `PrimitiveAst`. Never escape the crate. Properties are stored as `BTreeMap<String, Value>` so the eventual canonical-bytes encode is permutation-stable; that property is exercised by a dedicated round-trip test.
- **`Parser` (lines 256-683):** a hand-written single-pass cursor parser. Tracks byte position, 1-indexed line, 1-indexed column. The dispatch table for the 12 primitives is intentionally one big `match` (the `#[allow(clippy::too_many_lines)]` is justified inline — splitting it would scatter the same call shape across 12 single-call helpers without making the table easier to read).
- **`emit` (lines 747-797):** AST → `Subgraph`. Walks the primitive list, builds one `OperationNode` per entry, threads `next`-labeled edges between consecutive ops. Refuses to emit a handler missing `respond` (`E_DSL_MISSING_RESPOND`).
- **`validate_shapes` (lines 817-860):** the Phase-3 R6 fp Wave C2 addition. Today it only enforces SANDBOX integer-typed property names (`fuel`, `wallclock_ms`, `output_limit`). Designed as a single pass with appended rules so future shape rules don't multiply the typed-error surface. Uses the canonical eval-side snake_case names — the TS surface translates camelCase to snake_case at `packages/engine/src/dsl.ts::translateSandboxArgs` BEFORE the napi boundary, so the Rust-side validator only ever sees the canonical form.
- **`id_for` (lines 862-890):** maps `(PrimitiveKind, idx)` to a stable per-node id with a uniform 2-char prefix (`"re0"`, `"wr1"`, `"rs3"` — Read/Write/Respond). #798 scheme-(a): the prefix scheme is wire-stable (the id is hashed into canonical bytes via the `(id, kind)`-sorted node view) so it was normalized to uniform 2-char pre-v1; the prior mixed-width scheme (`r`/`w` 1-char, `wait`/`resp` 4-char) is gone. `PrimitiveKind` is `#[non_exhaustive]`, so a fallback `op` prefix is in place for variants the parser does not yet have keywords for; that fallback is unreachable today.
- **Inline `#[cfg(test)] mod inline_tests` (lines 828-930):** 9 in-crate tests covering the smoke shapes (minimal round-trip, empty source, missing-respond, unknown-primitive, unbalanced-brace, three SANDBOX shape-validation cases, plus a permutation-stable canonical-bytes property pin).

No other source files exist.

---

## 4. Public API surface

Per the Cargo.toml description the public surface is "4 public items." The literal export count is larger because the compiled-output structs are also public; the spirit of the rule (a narrow, devserver-shaped surface) is what holds.

**1. `compile_str(source: &str) -> Result<CompiledSubgraph, CompileError>`**

Compile a string. Plain English: hand the function a DSL snippet, get back a structure containing both a canonical `Subgraph` and a per-primitive declaration list for introspection. The function rejects empty input with a typed parse error and otherwise delegates to the parser and emitter.

**2. `compile_file(path: &Path) -> Result<CompiledSubgraph, CompileError>`**

Compile a file. Plain English: same as `compile_str` but reads the bytes off disk first. The only failure mode added beyond `compile_str` is `CompileError::Io`.

**3. `CompileError`**

A typed error enum with four variants — `Parse`, `Semantic`, `Emit`, `Io`. Each non-Io variant carries a `Diagnostic`. The discriminant is stable; devserver and downstream tooling switch on it without parsing prose. The `diagnostic()` helper unwraps the inner `Diagnostic` when present.

**4. `Diagnostic`**

The shape devserver renders. Five fields: a stable `error_code` string (one of four `E_DSL_*` constants), a human-readable `message`, and optional 1-indexed `line` + `column`. Implements `Display` for log output. The error_code is the load-bearing field; everything else is for humans.

**Supporting publics (also part of the surface but not in the "4 public items" count):**

- `CompiledSubgraph { subgraph: Subgraph, primitives: Vec<CompiledPrimitive> }` — the success payload.
- `CompiledPrimitive { kind: PrimitiveKind, properties: BTreeMap<String, Value> }` — one per emitted node. (The per-node stable id like `"r0"` lives on the canonical `Subgraph` node via `OperationNode::new`; it is NOT duplicated onto `CompiledPrimitive` — the formerly-present `id` field was dead data with zero read sites and was removed, refinement-audit #288.)
- `pub use benten_core::PrimitiveKind` — re-exported so devserver consumers don't need a transitive `benten-core` dependency just to pattern-match on the kind.

Everything else (`E_DSL_*` constants, `HandlerAst`, `PrimitiveAst`, the `Parser` struct, the `emit` and `validate_shapes` and `id_for` functions) is `pub(crate)` or private. That is the discipline; new exports should be challenged.

---

## 5. Tests inventory

The crate has nine inline tests in `src/lib.rs` (covered in §3) plus four integration test files in `tests/`.

**`tests/arch_n_benten_dsl_compiler_dep_direction.rs`** — the architectural-pin file. Four tests scan `Cargo.toml` directly:

- The crate does NOT depend on `benten-eval`.
- The crate does NOT depend on `benten-graph`.
- The crate does NOT depend on `benten-engine`.
- The crate DOES depend on `benten-core`.

These are the load-bearing arch-1 pins. The scan parses only `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` tables — the `description` field in `[package]` mentions the forbidden crate names so the parser must be careful not to false-positive on prose.

**`tests/dsl_compiler_emits_widened_subgraph_spec.rs`** — three tests pinning the post-G12-D widened per-primitive `BTreeMap<String, Value>` properties bag:

- WAIT's `ttl_hours` survives as `Value::Int(24)` on the emitted primitive.
- SANDBOX's `wallclock_ms` survives as `Value::Int(30000)`.
- Permuted property keys produce identical canonical bytes (the sec-pre-r1-09 carry — BTreeMap sort order is what makes this work).

**`tests/dsl_compiler_rejects_invalid_syntax_typed.rs`** — five tests pinning the typed-error contract:

- Unbalanced brace → `CompileError::Parse` with code `E_DSL_PARSE_ERROR`.
- Unknown primitive → `CompileError::Semantic` with code `E_DSL_UNKNOWN_PRIMITIVE`.
- Missing respond → `CompileError::Emit` with code `E_DSL_MISSING_RESPOND`.
- Empty source → `Parse` with no line/column.
- Syntax error on a specific line → diagnostic carries the correct 1-indexed line + column.

**`tests/dsl_compiler_round_trips_5_primitive_fixtures.rs`** — six tests pinning round-trip CID-stability for the five MINIMAL-FOR-DEVSERVER fixtures (READ, WRITE, BRANCH, TRANSFORM, CALL) plus a property-shaped pin that compiles a handler, serializes via canonical bytes, deserializes back, and asserts CID equality. This is the Inv-10 canonical-bytes collision-stability anchor for the compiler emission path (order-independent DAG-CBOR per `docs/INVARIANT-COVERAGE.md` row 10 — NOT Inv-13 immutability, which fires at engine `put_node_with_context` time, not at DSL emission).

There are no proptest suites or fuzz harnesses in this crate. Everything is example-driven.

---

## 6. Benches inventory

None. `Cargo.toml` carries `[lib] bench = false`. No `benches/` directory.

---

## 7. Thin-engine + composable-graph philosophy check

**Well-respected examples:**

- **Dep direction is enforced by source-scanning tests, not just by `Cargo.toml` discipline.** Adding `benten-eval` as a dependency would be caught at test time, not just discovered later when a cyclic dep error fires. This is the right shape — the constraint is mechanical, not aspirational. Pairs with the inverse pin in `crates/benten-engine/tests/no_dsl_compiler_dep.rs` to lock the directionality from both sides.
- **The compiler emits canonical `benten-core` types only.** No engine-internal types leak into the output. `CompiledSubgraph` exposes a `Subgraph` and a `Vec<CompiledPrimitive>`; both are made from `benten-core` building blocks. Devserver introspection can read either surface (same data, different shape).
- **`BTreeMap<String, Value>` for properties is the canonical-bytes anchor.** The choice of `BTreeMap` over `HashMap` is load-bearing — it gives the emitter deterministic property ordering for free, and `permuted_keys_yield_identical_canonical_bytes` plus `dsl_compiler_widened_emission_canonical_bytes_stable_for_permuted_prop_keys` pin that promise. Re-ordering the keys in DSL source produces identical CIDs. That is the CLAUDE.md #5 content-addressing contract delivered for free by the data structure choice.
- **The four-public-item discipline.** Per the Cargo.toml comment, the surface was budgeted at four items. Even with the R6 fp Wave C2 addition of `E_DSL_INVALID_SHAPE` plus the shape-validation pass, the public function/enum/struct count is still tight. `cargo-public-api` baselining (mentioned in the crate docs) is the mechanical defense.
- **`PrimitiveKind` is `#[non_exhaustive]` and the `id_for` fallback acknowledges this.** Future primitives (none today; CLAUDE.md #1 names 12 and the parser handles all 12) get a generic `op` prefix until the grammar catches up. That is the right shape — the compiler degrades gracefully rather than silently swallowing a new variant.
- **CLAUDE.md #10 zero-config CRUD is NOT this crate's job.** The TypeScript DSL's `crud('post')` zero-config sugar lives at `packages/engine/src/dsl.ts`. This crate's grammar is the raw primitive-chain shape; the TS DSL is the user-facing sugar. That separation is correct — Rust-side DSL exists for devserver inline compilation, not for end-user authoring.

**Drift watchlist (not flags — observations that warrant attention if the crate grows):**

- **Property-key namespace (#604 scheme-(a), pre-v1 normalized).** The emitter uses the **canonical `benten_core::SubgraphBuilder` key namespace** — `label`, `user_properties`, `body`, `predicate`, `handler`, `args`, `module`, `topic`, `pattern`, `iter_body` (no underscore prefixes). These mirror the literal keys `SubgraphBuilder` stamps for the same `PrimitiveKind` (`call_handler`→`handler`, `sandbox`→`module`, `transform`→`body`, …) so a handler authored via this Rust DSL and one authored via `SubgraphBuilder` produce **byte-identical canonical bytes / identical CID** — closing the Inv-10 cross-surface gap (#604) and the prior `_body` cross-primitive overload (#782; TRANSFORM `body` Map vs ITERATE `iter_body` Text are now distinct keys). The single source of truth is the `KEY_*` const block at the top of `lib.rs`; cross-doc/type mirror discipline (dispatch-conventions §3.5g) couples it to the `SubgraphBuilder` definitions. WAIT and SANDBOX still inline user-supplied keys directly; a user-supplied key colliding with a canonical key would overwrite depending on insertion order — not a bug today; a convention pin if the property space widens. **This is a wire-format freeze item: the key namespace was normalized pre-v1 (CID churn free now, catastrophic post-v1); pinned at `tests/canonical_bytes_v1_scheme_pin.rs`.**
- **`validate_shapes` only knows about SANDBOX.** The function is structured for appending rules but currently covers exactly one primitive's typed properties. Other primitives' typed-property constraints (e.g. WAIT's `ttl_hours` shape) flow through unchecked and would surface at the engine layer with a less-actionable error. If shape-validation becomes a regular pattern, the cost of NOT adding the rule here grows; if it stays one-off, the current shape is fine.
- **The 4-public-items count is drifting in spirit.** The literal surface today is `compile_str` + `compile_file` + `CompileError` + `Diagnostic` + `CompiledSubgraph` + `CompiledPrimitive` + the re-exported `PrimitiveKind`. The Cargo.toml description still says "4 public items"; the wording could be sharpened to "4 entry points + 2 result structs + 1 re-export" so future readers don't think the spec is being violated.
- **The crate is named `dsl-compiler` but compiles only the Phase-2b grammar.** A future agent reading just the name might assume the crate is the universal Rust-side DSL surface. It isn't — it is the devserver-shaped subset. The crate-level docstring is explicit about MINIMAL-FOR-DEVSERVER, but the name is broader than the scope. Worth keeping in mind when extension proposals land.
- **Typed-CALL DSL surface lives at the TS layer, not here.** Phase-3 G21-T2 (PR #148, `7a6c36a`) added a `typedCall({ op, inputBinding? })` DSL helper at `packages/engine/src/dsl.ts` that composes into a regular `CALL` primitive with an `engine:typed:*` target string — the Rust dsl-compiler grammar has no new keyword; a `typedCall` written in raw Rust-side DSL text would trip `E_DSL_UNKNOWN_PRIMITIVE` (verified: `parse_primitive` at `src/lib.rs:312-465` has no `typedCall` arm). The deliberate placement is per CLAUDE.md #1 (typedCall composes via existing CALL — NOT a 13th primitive) AND per this crate's MINIMAL-FOR-DEVSERVER scope: typed-CALL is end-user DSL ergonomics; raw devserver inline-compile paths can still author the same shape by writing `call('engine:typed:<op>', { ... })` directly.
- **No schema-driven-rendering shape lives here.** Per the request brief, the Phase 4-Foundation D-4F-3 option (c) proposes extending this crate for schema-rendering. As of this audit, nothing schema-rendering-shaped has landed — the crate is still strictly handler-DSL → Subgraph. The mental model "if it doesn't fit handler-DSL → Subgraph, it probably belongs in a sibling crate" is the right default until that decision is made.

No anti-patterns observed (no DSL feature creep, no engine-internal types leaking, no premature schema-rendering shapes).

---

## 8. Phase 4-Foundation + Phase 4-Meta expectations

**Phase-4-Foundation R5+R6 outcome (2026-05-13 → 2026-05-14):** this crate was NOT extended during the Phase-4-Foundation engineering window. R5's 17 waves + R4b-FP + R6-FP cluster + R6-FP-2 through R6-FP-7 touched zero source files in `benten-dsl-compiler/`. The substantive Phase-4-Foundation work (plugin manifest schema, decentralized registry, materializer scaffolding, admin-shell wiring) landed in `benten-platform-foundation` and adjacent crates; this crate sat unmodified at HEAD `8141b94`. The mental model "MINIMAL-FOR-DEVSERVER" held through the phase.

Two distinct things could happen to this crate going forward; both remain pinned in the brief, not in the code:

**(a) Schema-driven-rendering compiler — Phase 4-Foundation D-4F-3 option (c).** The plan's option (c) proposes extending `benten-dsl-compiler` to host the schema-rendering compiler. If that path is chosen the crate roughly doubles in scope: it would gain a second compilation pipeline (schema-text → some renderable graph shape), likely a second public function pair (`compile_schema_str` / `compile_schema_file`), and possibly a second AST. The dependency on `benten-core` only would still need to hold; that's the load-bearing constraint that would have to be preserved. The current `validate_shapes` single-pass pattern would generalize naturally if multi-pipeline support is added.

If the plan instead chooses a sibling crate (the alternative option in D-4F-3), this crate stays at its current scope and a `benten-schema-compiler` crate appears next to it. That is the cleaner separation; the current crate's "MINIMAL-FOR-DEVSERVER" framing would survive intact.

**(b) Phase 4-Foundation materializer consumption.** A Phase 4-Foundation materializer (whatever shape it takes) may consume `CompiledSubgraph`s emitted by this crate. The current `CompiledSubgraph` shape — canonical `Subgraph` + per-primitive declaration list — is materializer-friendly: the materializer can walk either surface depending on whether it wants canonical bytes or introspection metadata. No changes anticipated to support that consumption pattern.

No other Phase-4 expectations are encoded in this crate's code. The brief instructs not to speculate further.

---

## 9. Open questions / unresolved internals

Items worth flagging for whoever next touches this crate. None are bugs; all are points where intent could be re-confirmed.

1. **Public surface count statement drift (Cargo.toml description).** The description says "~200-300 LOC, 4 public items"; the crate is ~895 LOC and the spirit-of-the-rule public surface is the four entry points plus two result structs plus one re-export. The constraint is intact in spirit but the literal description is stale. Worth updating the description to match what is actually committed, so the discipline reads as preserved rather than violated.

2. **`validate_shapes` scope expansion.** Today the only shape-validation rule is SANDBOX integer-typed properties. The brief comment says "Future shape rules append to this single pass so the typed-error surface stays narrow." If a future agent wants to add a WAIT shape rule, a TRANSFORM shape rule, or a BRANCH predicate-shape rule, the pattern is in place; it just hasn't been exercised more than once.

3. **`branch(...)` and `iterate(...)` predicate/body capture.** Both primitives store the parenthesized expression body as opaque text (`Value::Text`) rather than parsing it. The grammar comment explicitly says "the surface evaluator pins predicate semantics in a later phase." That phase is unspecified — Phase 4 plugin work or something earlier? Not for this audit to answer, but the compiler's contract is currently "I will hand you the text; you parse it later."

4. **`HandlerAst::handler_id` and `Subgraph::handler_id` are both `String`.** No length cap, no character-set restriction, no namespace conventions enforced at the compiler level. If the engine has handler-id constraints they are enforced at registration time, not at compile time. Whether this should change is a policy question.

5. **The `pub use benten_core::PrimitiveKind` re-export couples DSL surface to core enum.** If `benten-core` ever introduces a new `PrimitiveKind` variant, the parser's dispatch match would still compile (the `other =>` arm handles unknown keywords as `E_DSL_UNKNOWN_PRIMITIVE`), but the `id_for` fallback to `"op"` would be the only path the new variant could traverse. Not a bug — the `#[non_exhaustive]` discipline is being respected — but the fallback is silent and a future agent might not notice their new primitive is being given a generic prefix.

6. **No fuzz / proptest coverage of the parser.** Every test is example-driven. The parser is hand-written and small enough that this is defensible, but it is the only attack surface where malformed text reaches Rust state machines, and a property test ("any string that parses → CompiledSubgraph round-trips through canonical-bytes with stable CID") would be cheap to add.

7. **Comment claims `pinned at compile time by tests/arch_n_benten_dsl_compiler_dep_direction.rs`.** The phrase "pinned at compile time" is slightly loose — the dep-direction tests run at test time, not compile time. If `benten-eval` were added to `[dependencies]` the workspace would still compile; the test failure would catch it on the next `cargo test` run. The pin is mechanical and reliable, just not literally compile-time.

8. **Typed-CALL grammar gap is deliberate, not an oversight.** A reader noticing that `parse_primitive` has no `typedCall` keyword while `packages/engine/src/dsl.ts` does might assume the Rust side is lagging. It isn't — typed-CALL is end-user DSL ergonomics layered on top of CALL; the canonical Subgraph the TS DSL emits has a regular `CALL` node with the canonical `handler` property = `engine:typed:<op>`, which this crate already parses via the `call('engine:typed:<op>', { ... })` form. There is no class of subgraph the TS DSL can express that this crate cannot also express by writing the call-target literal. Adding a `typedCall` keyword here would gain nothing and would couple the dsl-compiler crate to the `engine:typed:*` prefix conventions defined in `benten-eval` — which would route around the arch-1 dep-direction tests.
