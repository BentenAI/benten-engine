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

Derived from `benten-core::Value` variants (per schema-r1-4 + cag-r1-1):

| Scalar | `Value` variant | Notes |
|---|---|---|
| `text` | `Value::String` | UTF-8 string |
| `int` | `Value::I64` | 64-bit signed integer |
| `float` | `Value::F64` | 64-bit float |
| `bool` | `Value::Bool` | True/false |
| `bytes` | `Value::Bytes` | Raw byte array |
| `bytes-cid` | `Value::Cid` | Content identifier; integrates with content-addressing |
| `timestamp-hlc` | `Value::Hlc` | Hybrid Logical Clock timestamp; carries causal ordering |
| `null` | `Value::Null` | Explicit null |

### §2.4 Four mandatory field properties

Every `Field*` Node MUST carry:

| Property | Type | Notes |
|---|---|---|
| `name` | `text` | Human-readable field name (used in admin UI form generation) |
| `required` | `bool` | Whether this field must be present on instances |
| `default` | value-or-null | Default value if omitted |
| `scope` | `text` | Schema-derived cap-scope (sec-3.5-r1-4) — NOT user-supplied. Computed at schema-compile time |

`scope` is the load-bearing security property: a user-supplied schema cannot specify an arbitrary cap-scope; the compiler maps schema content-types → cap-scope via a fixed derivation (`schema_field_to_cap_scope(schema_cid, field_name) -> CapScope`). This is the structural defense against subgraph-injection attacks via schemas (threat-model T1).

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

In the generalized IVM Algorithm B kernel (per Ben D-4F-2). The kernel walks the `SubgraphSpec` subgraph using existing engine evaluator dispatch on READ / TRANSFORM / SUBSCRIBE / RESPOND primitive Nodes. No host-side `Materializer::walk()` outer loop — walking is engine-internal subgraph evaluation.

### §7.3 Primitive composition

| Primitive | Use |
|---|---|
| `READ` | Content fetch (gated via `read_node_as` per §3.Y dual-gate; SHARES `IvmViewReadGate` per D-4F-NEW-MATERIALIZER-READ-GATE post-triage resolution) |
| `TRANSFORM` | Field-level shaping (apply default values; format scalars; etc.) |
| `SUBSCRIBE` | Reactive update via `on_change_as_with_cursor` ONLY (sec-3.5-r1-9) — the principal-aware variant; NEVER plain `subscribe_change_events` |
| `RESPOND` | Output emission consumed by `Renderer` trait |

### §7.4 Renderer pluggability (two layers)

Per cag-r1-6 disambiguation:

- **Output-FORMAT pluggability**: `Materializer` trait has multiple impls (`HtmlJsonMaterializer` default + `PlaintextMaterializer` 2nd impl per arch-r1-10, empirically validating pluggability with 2 impls).
- **Renderer-BACKEND pluggability**: `Renderer` trait abstracts the rendering target. Two backends ship at Phase 4-Foundation: `BrowserRender` (browser-wasm32 in `benten-platform-foundation`) + `TauriRender` (Tauri 2.x embedded-webview in `benten-renderer-tauri` per CLAUDE.md #19 — new 12th crate).

Both layers validated empirically by 2 impls; pluggability is not just declared but exercised.

### §7.5 Cap-scoped redaction (Compromise #11 floor)

Materializer's read fanout uses `Engine::read_node_as(walk_principal, cid)` at every read. Cap-policy fires per primitive. Compromise #11 closure floor is REAFFIRMED against the materializer surface (sec-3.5-r1-13).

Per post-R1-triage ratification #7 (D-4F-NEW-MATERIALIZER-SUBSCRIBE-RE-WALK-CONSISTENCY): SUBSCRIBE-re-walk is option (c) — re-filter at delivery; **Node-granularity redaction** (G22-FP-1 implementation). Redacted Nodes drop from the delivery stream; subscriber sees consistent stream-stays-open-but-elides-revoked-Nodes UX.

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
