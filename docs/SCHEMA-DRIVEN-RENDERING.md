# Schema-Driven Rendering

The engineer-facing reference for Phase 4-Foundation's schema-driven rendering surface — schemas as content-addressed graph Nodes, the canonical typed-field-Node vocabulary that schema-subgraphs are built from, the materializer pipeline that walks them, and the ingest-dialect workflows that translate input → canonical form.

Cross-references: design decision points D-4F-3a (schemas as graph Nodes) + D-4F-3b (schemas-as-subgraphs of primitive typed-field Nodes) + D-4F-NEW-TYPED-FIELD-NODE-VOCAB (ratified vocabulary post-R1-triage) + D-4F-NEW-MATERIALIZER-READ-GATE (SHARE `IvmViewReadGate`) — all at `.addl/phase-4-foundation/00-implementation-plan.md §5`.

---

## §1. Schemas as graph Nodes

A schema is a **content-addressed subgraph of primitive typed-field Nodes** (D-4F-3a + D-4F-3b ratified). Each schema gets a CID derived from its bytes; users can share schemas across Atrium peers; receivers verify content-addressing on receive (Phase-3 W9-T6 inheritance).

This decision rules out:
- Schemas as Rust type-system constructs (incompatible with shareable-schema framing — schemas need to travel between peers as data, not code).
- Schemas as opaque string payloads (would lose graph-walkability + content-addressing benefits).

It commits to: schemas have the same shape as any other Phase-1 graph — a DAG of Nodes connected by typed Edges. The engine evaluator can walk a schema-subgraph the same way it walks any other subgraph.

---

## §2. Vocabulary (RATIFIED post-R1-triage)

Per D-4F-NEW-TYPED-FIELD-NODE-VOCAB post-R1-triage resolution + Ben Q9 correction 2026-05-11 (vocabulary design = R1/R2 not R5).

### §2.1 Eight labels

| Label | Purpose |
|---|---|
| `SchemaRoot` | Root container of a schema-subgraph; one per schema |
| `FieldScalar` | A primitive value field (text, int, bool, etc. — see §2.3 scalars) |
| `FieldObject` | A nested object field (composes recursively over schemas) |
| `FieldList` | An ordered collection of values |
| `FieldMap` | A keyed collection of values (added post-R1-triage; was missing in 6-label draft) |
| `FieldRef` | A cross-content reference, content-CID-keyed |
| `FieldEnum` | An enumerated choice between named variants |
| `FieldUnion` | A tagged union of variant types |

### §2.2 Six edges

| Edge | Connects | Purpose |
|---|---|---|
| `FIELD` | `SchemaRoot` / `FieldObject` → `Field*` | Object-to-field relationship |
| `ITEM_TYPE` | `FieldList` / `FieldMap` → `Field*` | Element type of a collection |
| `KEY_TYPE` | `FieldMap` → `FieldScalar` | Key type of a map |
| `VALUE_TYPE` | `FieldMap` → `Field*` | Value type of a map |
| `REF_TARGET` | `FieldRef` → (content CID) | Reference resolution (matches D-4F-14 cross-plugin content-CID shape) |
| `VARIANT` | `FieldEnum` / `FieldUnion` → `Field*` | Variant type of an enum or union |

### §2.3 Eight scalars

Derived from `benten_core::Value` variants (per schema-r1-4 + cag-r1-1). Scalar-to-`Value` mapping is many-to-one: `bytes-cid` + `timestamp-hlc` are *interpretations layered over* `Value::Bytes` + `Value::Int` respectively, not distinct `Value` variants. The canonical `Value` enum has 8 variants total: `Null` / `Bool` / `Int(i64)` / `Float(f64)` / `Text(String)` / `Bytes` / `List` / `Map`.

| Scalar | `Value` variant | Notes |
|---|---|---|
| `text` | `Value::Text` | UTF-8 string |
| `int` | `Value::Int` | 64-bit signed integer |
| `float` | `Value::Float` | 64-bit float |
| `bool` | `Value::Bool` | True/false |
| `bytes` | `Value::Bytes` | Raw byte array |
| `bytes-cid` | `Value::Bytes` (CID-interpreted) | Content identifier; bytes carry multibase-encoded CID. NOT a distinct `Value` variant. |
| `timestamp-hlc` | `Value::Int` (HLC-interpreted) | Hybrid Logical Clock timestamp; int carries the HLC ticks. NOT a distinct `Value` variant. |
| `null` | `Value::Null` | Explicit null |

### §2.4 Four mandatory field properties

Every emitted `Field*` Node carries 4 properties, split into 3 user-supplied + 1 compiler-derived:

#### User-supplied (3)

| Property | Type | Notes |
|---|---|---|
| `name` | `text` | Human-readable field name (used in admin UI form generation) |
| `required` | `bool` | Whether this field must be present on instances |
| `default` | value-or-null | Default value if omitted |

#### Compiler-derived (1)

| Property | Type | Notes |
|---|---|---|
| `scope` | `text` | Schema-derived cap-scope (sec-3.5-r1-4) — NEVER user-supplied. Computed by `derive_scope(action, schema_name, field_path)` at emit time |

`scope` is the load-bearing security property: schema author supplies 3 properties, the compiler synthesizes scope from the field path (`action:SchemaName.field_path` form). If the input JSON has a `scope` key, the parser **silently discards it** (see `schema_compiler::parse::parse_field`); the emitter synthesizes its own. This is the structural defense against subgraph-injection attacks via schemas (threat-model T1) — a user-supplied scope would be an authorization-side-channel, letting the schema author decide their own cap-policy namespace.

### §2.5 Composability invariant

Every vocabulary label maps to a composition over the existing 12 primitives via the schema-compiler. **No new `PrimitiveKind` variants are minted** (per cag-r1-1; honoring CLAUDE.md baked-in #1). The schema-compiler emits `SubgraphSpec`s wiring READ / WRITE / TRANSFORM / RESPOND / SUBSCRIBE primitive Nodes with `requires` cap-scope annotations per primitive Node.

Test pin: `crates/benten-platform-foundation/tests/schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension.rs`.

---

## §3. Reserved vocabulary extensions (Phase 4-Meta)

Reserved for Phase 4-Meta (per schema-r1-10; named at plan §7 OOS):

- `FieldTuple` — fixed-arity ordered collection
- `FieldRange` — bounded continuous range
- `FieldHLCRange` — HLC-typed temporal range
- `FieldCidSet` — set of content-CIDs

Phase 4-Foundation does NOT mint these. The vocabulary is intentionally minimal at v0; extensions land when Phase 4-Meta plugin ecosystem use-cases drive them.

---

## §4. Ingest dialects

Ingest dialects (JSON-Schema / TS DSL / Python / arbitrary text DSL) parse user-authored schema definitions into canonical typed-field-Node subgraphs. Per schema-r1-3 + post-R1-triage decision:

**Parse locus = engine-side**, NOT browser-side. Ingest dialect parsers live at `crates/benten-platform-foundation/src/ingest_dialect/`. Browser submits either:
- Canonical-bytes (already a `SubgraphSpec`-shaped DAG-CBOR document), OR
- Dialect-source-bytes (a JSON-Schema string, a TS DSL fragment, etc.) which the engine parses.

This decision composes with threat-model §3 T1 (subgraph injection): all schema-validation runs at engine-side authoritative gate; browser-side validation (if present) is best-effort feedback only.

Ingest dialects are **admin-plugin-owned workflows** — they can be added/replaced by ecosystem participants without requiring engine releases. Phase 4-Foundation ships an initial JSON-Schema dialect as a reference workflow.

---

## §5. Schema validation locus

Per schema-r1-6:

| Locus | Authority | Purpose |
|---|---|---|
| Browser-side | Best-effort UX feedback | Inline form validation; immediate user feedback |
| Engine-side (pre-WRITE gate) | Authoritative | Threat-model T1 defense; refuses subgraphs that violate vocabulary or cap-scope-derivation rules |

Threat-model §3 T1 defense composes: schema-compiler asserts cap-scope at emission time; materializer asserts again at walk time (defense-in-depth).

---

## §6. DAG-shape schema evolution

Per schema-r1-7 + D-4F-14: schema CID change = new Version Node on schema anchor (extends the Phase-1 anchor + Version Node pattern). Old content instances reference old schema CID (immutable); schema-evolution = pull-not-push (same as plugin updates per D-4F-13/14).

Cross-plugin/schema references use **content-CID, not author-DID** — `accepts_content: [hash, ...]` on plugin manifests names the schema CIDs the plugin can consume. Rotating a schema author's peer-DID does NOT invalidate downstream references (deliberate consequence of D-4F-12's content-CID-keying choice).

---

## §7. Materializer pipeline

The materializer is the runtime that walks a composed subgraph and produces output bytes. Per Ben D-4F-2 ratification: **materializer view IS an IVM view** — there is no separate materializer pipeline distinct from IVM. The 5 canonical IVM views get re-expressed as subgraph definitions (G23-0a + G23-0b); user-registered views + materializer views all use the same machinery.

### §7.1 Trait surface

`Materializer::materialize_with_gate(spec, content_cid, walk_principal, recheck_fn) -> Bytes`. NO new `PrimitiveKind` enum variant added — materializer is NOT a new primitive kind, it composes from existing 12 primitives.

### §7.2 Where the walk happens

Per Ben D-4F-2: the materializer view IS an IVM view, walking the same `SubgraphSpec` subgraph with existing engine evaluator dispatch on READ / TRANSFORM / SUBSCRIBE / RESPOND primitive Nodes.

**G23-B canary scope (Phase-4-Foundation):** the materializer iterates the emitted SubgraphSpec primitives directly, dispatching reads against a single content_cid via `Engine::read_node_as`. Recursive composition (walking `FieldRef::REF_TARGET` content into its referenced Node; nested `FieldObject` sub-field composition; `FieldList` / `FieldMap` element-level resolution at materialize time) is **named** in `docs/future/phase-4-backlog.md` §4.24 (Phase-4-Meta — admin UI v0 nested-form rendering driver). The opcode-list-shaped walk at G23-B is the v1-platform-shippable substrate; the recursive walk lands when the admin UI workflow editor needs nested-form rendering.

The vocabulary-edge wiring (`ITEM_TYPE` / `KEY_TYPE` / `VALUE_TYPE` / `REF_TARGET` / `VARIANT` per §2.2) IS shipped at R6-FP — the SubgraphSpec carries the edges so that the recursive walk has structure to consume when Phase-4-Meta opens.

### §7.3 Primitive composition

| Primitive | Use |
|---|---|
| `READ` | Content fetch (gated via `read_node_as` per §3.Y dual-gate; SHARES the `IvmViewReadGate` **shape** — not the literal type — per D-4F-NEW-MATERIALIZER-READ-GATE post-triage resolution. The materializer crate defines its own `MaterializerCapRecheck` alias with the same `Fn(&Cid, &str, &Cid) -> bool` signature; the literal `IvmViewReadGate` type lives in `benten-engine` and cannot be imported here without violating the dep-direction commitment (arch-r1-1: `benten-platform-foundation` does NOT depend on `benten-engine` in production). The Materializer-view-IS-IVM-view commitment per D-4F-2 is preserved by shape parity + the same dual-gate composition rules.) |
| `TRANSFORM` | Field-level shaping (apply default values; format scalars; etc.). Vocabulary-edge type-descriptor primitives (targets of `ITEM_TYPE` / `KEY_TYPE` / `VALUE_TYPE` / `REF_TARGET` / `VARIANT` edges) are also `TRANSFORM`-kinded — they reuse the canonical 12 primitives rather than minting a 13th. |
| `SUBSCRIBE` | Reactive update via `on_change_as_with_cursor` ONLY (sec-3.5-r1-9) — the principal-aware variant; NEVER plain `subscribe_change_events` |
| `RESPOND` | Output emission consumed by `Renderer` trait |

**Cap-policy fan-out semantics:** the per-row cap-recheck closure is invoked once per primitive during materializer walk (observability + audit-trail surface — readers see N invocations for N primitives in the spec). The **authoritative substantive cap-decision** happens once at the per-row gate boundary against the row's `(content_cid, zone_hint)` — see `materializer.rs:940-947` for the inline comment. Per-primitive `scope` properties stamped on each primitive Node ARE consulted authoritatively at the T1 envelope-check entry (materializer-entry — *before* any READ fanout). So the defense composes: T1 envelope check (per-primitive scopes against the manifest's declared envelope, always-on when `declared_requires` is non-empty) + per-row gate (per-row admit/deny against content). The fan-out's per-invocation return value is observability-only.

### §7.4 Renderer pluggability (two layers)

Per cag-r1-6 disambiguation:

- **Output-FORMAT pluggability**: `Materializer` trait has multiple impls (`HtmlJsonMaterializer` default + `PlaintextMaterializer` 2nd impl per arch-r1-10, empirically validating pluggability with 2 impls).
- **Renderer-BACKEND pluggability**: `Renderer` trait abstracts the rendering target. Two backends ship at Phase 4-Foundation: `BrowserRender` (browser-wasm32 in `benten-platform-foundation`) + `TauriRenderer` (Tauri 2.x embedded-webview in `benten-renderer-tauri` per CLAUDE.md #19 — new 12th crate).

Both layers validated empirically by 2 impls; pluggability is not just declared but exercised.

### §7.5 Cap-scoped redaction (Compromise #11 floor)

Materializer's read fanout uses `Engine::read_node_as(walk_principal, cid)` at every read. Cap-policy is consulted at materializer-entry (T1 envelope check, per-primitive scopes against manifest envelope) + at the per-row gate (substantive admit/deny). The per-primitive fan-out is observability/audit-trail only (see §7.3 narrative). Compromise #11 closure floor is REAFFIRMED against the materializer surface (sec-3.5-r1-13).

Per post-R1-triage ratification #7 (D-4F-NEW-MATERIALIZER-SUBSCRIBE-RE-WALK-CONSISTENCY): SUBSCRIBE-re-walk is **option-D** (originally tracked as option-(c) at R1 ratification time; renamed at R1-FP G22-FP-1 landing — see `engine_subscribe.rs:303` source comment for the timeline). Semantic: re-filter at delivery; **Node-granularity redaction** (G22-FP-1 implementation). Redacted Nodes drop from the delivery stream; subscriber sees consistent stream-stays-open-but-elides-revoked-Nodes UX; whole-actor revoke cancels the subscription (Cancel arm).

---

## §8. Vocabulary ErrorCodes

Phase 4-Foundation mints (companion-with-canary at G23-A; atomic Rust+TS per §3.5g cross-language rule-mirror):

- `E_SCHEMA_VALIDATION_FAILED` — generic schema invariant violation
- `E_SCHEMA_EMIT_NEW_PRIMITIVE_REJECTED` — schema compiler attempted to mint a new `PrimitiveKind` (load-bearing cag-r1-1 guard)
- `E_SCHEMA_SANDBOX_HOST_FN_REJECTED` — schema referenced SANDBOX with storage-mutating host-fn request (sec-3.5-r1-14)
- `E_SCHEMA_VOCAB_INVALID_LABEL` — label not in the 8-label set
- `E_SCHEMA_VOCAB_EDGE_MISMATCH` — edge type doesn't match label's allowed edges
- `E_SCHEMA_VOCAB_SCALAR_UNKNOWN` — scalar type not in the 8-scalar set
- `E_SCHEMA_VOCAB_REF_TARGET_MISSING` — `FieldRef` lacks `REF_TARGET` edge
- `E_SCHEMA_VOCAB_CYCLE_REJECTED` — schema-subgraph contains a cycle (cycle-detection at register-time per mat-r1-13)
- `E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING` — `Field*` Node lacks one of the 4 mandatory properties

Materializer mints (companion-with-canary at G23-B):

- `E_MATERIALIZER_CAP_DENIED` — cap-policy denied read during materializer walk
- `E_MATERIALIZER_SCHEMA_MISMATCH` — content does not match referenced schema CID
- `E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE` — subscribe-cursor seam returned unexpected error

See [`ERROR-CATALOG.md`](ERROR-CATALOG.md) for the full catalogue.

---

## §9. Cross-references

- **CLAUDE.md baked-in #1** — 12 primitives are irreducible (vocabulary composability invariant honors this)
- **CLAUDE.md baked-in #2** — IVM Algorithm B + materializer-view-IS-IVM-view per D-4F-2
- [`ARCHITECTURE.md`](ARCHITECTURE.md) §"12 crates" — `benten-platform-foundation` is the 11th crate (schema-compiler + materializer + plugin-manifest + Renderer trait)
- [`PLUGIN-MANIFEST.md`](PLUGIN-MANIFEST.md) — plugin manifests reference schema CIDs; `requires_schema_authors` trust-list
- [`ADMIN-UI.md`](ADMIN-UI.md) — admin UI v0 workflow editor uses schema-driven form generation (consumes this vocabulary)
- [`ENGINE-SPEC.md`](ENGINE-SPEC.md) — Renderer trait surface; engine integration

---

(Phase-4-Foundation companion doc lands at G23-A canary per `feedback_post_fix_doc_coupling_preflight.md` §3.5b HARDENED + meth-r1-7 companion-with-canary discipline.)
