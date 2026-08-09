Base verified at `bf964354`. Read-only throughout; nothing modified.

---

# Lens verdict: the case is materially weaker than it looks — and the reframe that motivated this pass is **factually dead**

## Conclusion first

**Recommend: do NOT add `Value::Decimal`. Not on cost grounds — on the grounds that it does not deliver the thing it is being bought for, and the seam it is supposed to be the last chance for already exists elsewhere and is not frozen.**

Four findings, in descending order of how much they move the decision:

1. **The major-type-6 reframe collapses.** DAG-CBOR rejects every tag but 42, in both directions. There is no band to claim, and no "last moment" pressure from it.
2. **`Value::Decimal` would be arithmetically inert *and* actively worse than `Float` today.** All engine arithmetic is a hardcoded `(Int, Float)` 2×2 promotion matrix. A Decimal lands in the `_ => Err` arm of every one. "Storage only" is not a neutral position here — it converts silent-wrongness into hard-failure at every TRANSFORM that touches money.
3. **The companion doc's architecture (balances as IVM materialized views) describes a capability the engine does not have.** `benten-ivm` performs **zero** arithmetic over `Value`. `ViewResult` cannot even *express* a scalar.
4. **The right seam exists, is public, is `#[non_exhaustive]`, and is therefore additive after the tag: `Scalar`** — which already carries exactly this pattern twice (`BytesCid`, `TimestampHlc`). The deadline argument does not apply to it.

The one thing I'd change before the tag is **not** decimal-related: `benten_core::Value` is absent from the freeze doc's §11 enum accounting entirely — neither `#[non_exhaustive]` nor carve-out-registered. That is a freeze-record gap on the single highest-leverage type, and it is genuinely tag-shaped.

---

## Finding 1 — the reframe is dead (confidence: HIGH; source-read, not executed)

Tags are not unclaimed. They are claimed by CID and closed by the codec, **both directions**.

**Decode** — `serde_ipld_dagcbor-0.6.4/src/de.rs:211-219`:
```rust
let tag = dec::TagStart::decode(&mut self.reader)?;
match tag.0 {
    CBOR_TAGS_CID => visitor.visit_newtype_struct(&mut CidDeserializer(self)),
    _ => Err(DecodeError::TypeMismatch { name: "CBOR tag", byte: tag.0 as u8 }),
}
```
Reached from the `deserialize_any` dispatch at `de.rs:323`, commented `// The only supported tag is tag 42 (CID).` Tag 4 (decimal fraction) and tag 5 (bigfloat) are hard-rejected. `CBOR_TAGS_CID = 42` at `lib.rs:148`.

**Encode** — `ser.rs:197-207`: the *only* path that emits a tag is `serialize_newtype_struct` when `name == CID_SERDE_PRIVATE_IDENTIFIER`, and the only `types::Tag(..).encode` call in the crate is `ser.rs:614`, hardcoded to 42. A `Value::Decimal` variant has no reachable way to emit tag 4 through this codec at all — it would require forking the codec, which collides head-on with baked-in #5's never-fork posture (that clause is scoped to crypto primitives, but the reasoning transfers cleanly to the canonical-bytes codec).

Worth noting as a bonus: `Value` today rejects **all** of major type 6 including tag 42. `ValueVisitor` (`crates/benten-core/src/value.rs:185-292`) has no `visit_newtype_struct` override, so serde's default `Err(invalid_type)` fires. A CID in a property position does not decode. This is consistent — the engine stores CIDs in properties as `Value::Text(cid.to_base32())` (`crates/benten-graph/src/backends/blob_backend.rs:273`, `crates/benten-engine/src/engine_modules.rs:329`, `handler_versions.rs:147`, `primitive_host.rs:467`, `engine_caps.rs:647`).

**That last point is the strongest single precedent available.** The engine's own most load-bearing domain type — the CID — is *not* a `Value` variant, despite the codec natively supporting tag 42 for it. It is encoded into `Text` with a documented interpretation. The decimal request is structurally identical and has strictly less claim than CID does.

---

## Finding 2 — a Decimal that cannot be summed is worse than a Float that can

The brief hypothesised IVM aggregates `Float(f64)`. **It does not — nothing in IVM does arithmetic at all** (Finding 3). But the hypothesis is right about the *shape* of the problem; it is just located in `benten-eval`.

Every numeric operation is a closed `(Int, Float)` matrix with an error fall-through:

| Site | Shape |
|---|---|
| `expr/eval.rs:518-527` `add` | 4 numeric arms + `Text` concat, `_ => Err("+ type mismatch")` |
| `expr/eval.rs:529-542` `numeric_op` (`-`, `*`) | 4 arms, `_ => Err` |
| `expr/eval.rs:544-562` `divide` | 4 arms; `Int/Int` non-even **promotes to `f64`** |
| `expr/eval.rs:563-577` `modulo` | 4 arms, `_ => Err` |
| `expr/builtins.rs:285-305` `arith_sum` | `any_float` flag → `Value::Float(total_f + total_i as f64)` |
| `expr/builtins.rs:306-326` `arith_product` | same shape |
| `expr/builtins.rs:918-924` `to_f64` | `Int`/`Float` only — gateway for `sqrt`/`pow`/`log`/`ceil`/`floor`/`round`/`trunc` |
| `expr/builtins.rs:967-982` `cmp_values` | 4 numeric arms — backs `min`/`max`/`sort`/comparison operators |

`Value::Float` appears at **63 non-test sites across 16 production files** (`benten-core`, `benten-eval`, `benten-graph`, `benten-ivm`, `benten-dsl-compiler`, `benten-platform-foundation`, `benten-engine`, `benten-errors`, `bindings/napi`). Adding a variant is a compile error at `to_canonical` (`value.rs:373`) and `is_already_canonical` (`value.rs:426`) — good, the decision can't be made accidentally — but the 63 sites are where the *design* work actually is.

The requester's "it does not need arithmetic in the engine" concedes exactly the wrong thing. Under that concession:
- `sum(line_items.amount)` → `Err("sum on non-numeric")`
- `order.total > 100` → `Err("compare type mismatch")`
- `sort by amount` → error

Today, with `Float`, those all *work* and are subtly wrong. With a storage-only `Decimal`, they hard-fail. That may be defensible as fail-closed design — but it is a different and larger proposal than "add a variant," and it pushes 100% of monetary arithmetic into application code, which is precisely the "re-create the problem one layer up" the brief asked me to test. **It re-creates it, and adds an error surface.**

---

## Finding 3 — the balance-view architecture is not implementable at HEAD

`benten-ivm` contains **no** `sum`, `fold`, or accumulation over `Value`. Exhaustive enumeration of numeric `Value` use in the crate:

- `algorithm_b.rs:1193,1196` — `createdAt` / disambiguator, **ordering keys**
- `algorithm_b.rs:1537,1544` — `seq`, **test fixtures**
- `views/content_listing.rs:474` — reads `Value::Int` as a **sort key**
- `views/governance_inheritance.rs:277,281` — `depth`, `rules.len()`, **structural counts**
- `views/capability_grants.rs:314`, `governance_inheritance.rs:313` — `n`, **test fixtures**

Not one accumulates a domain quantity. And the output type cannot carry one — `crates/benten-ivm/src/view.rs:248-257`:
```rust
#[non_exhaustive]
pub enum ViewResult {
    Cids(Vec<Cid>),
    Current(Option<Cid>),
    Rules(BTreeMap<String, Value>),
}
```
There is no scalar arm. **A balance view is not expressible.** So "money is an append-only Node in an add-wins set with balances as IVM materialized views" is not a design that de-risks `Value::Decimal` — it is a design that depends on aggregation machinery that does not exist and is not in Phase-4-Meta-Core scope.

**But note the asymmetry, because it is the whole answer:** `ViewResult` is `#[non_exhaustive]` (registered in the F-22 pre-tag sweep, `docs/V1-FROZEN-INTERFACE.md:1326`). The aggregation half is **additive after the tag**. `Value` is not. So the freeze pressure is on the half that the requester says they don't need, and absent from the half they actually do.

---

## Finding 4 — the seam already exists, one layer up, unfrozen

`crates/benten-platform-foundation/src/schema_compiler/vocab.rs:158-182`:
```rust
#[non_exhaustive]                    // :162
pub enum Scalar {
    Text, Int, Float, Bool, Bytes,
    /// Content identifier — bytes carry the CID multibase encoding. Maps
    /// to `Value::Bytes` with a documented CID interpretation.
    BytesCid,                        // :176
    /// HLC timestamp — int carries the HLC ticks. Maps to `Value::Int`
    /// with HLC interpretation.
    TimestampHlc,                    // :179
    Null,
}
```

This is the established, twice-used pattern: **a semantic refinement type over an existing `Value` primitive, declared at the schema layer, with no new wire variant.** `Scalar` is public (`docs/public-api/benten-platform-foundation.txt:1460-1471`) and `#[non_exhaustive]`, so `Scalar::Decimal` — "`Value::Text` (or `Map{unscaled,scale}`) with a documented fixed-scale-decimal interpretation" — lands additively in Composing or later. **No tag deadline. No wire break. No governance event.**

This is also where the per-field scale the requester needs actually belongs: 3dp line tax vs 2dp order header vs 3dp unit cost are *schema* facts, not *value* facts. A `Value::Decimal` carrying its own scale inline puts scale in the data where it can drift per-row; `Scalar` puts it in the schema where it is declared once and validated.

---

## Honest costs — where the requester is right, and where my recommendation is thin

I want these on the record rather than buried.

**The silent-f64 bug is real and present at HEAD, in two paths:**
- `crates/benten-dsl-compiler/src/lib.rs:1071-1081` — a DSL literal containing `.` parses via `f64::from_str` → `Value::Float`. `19.99` is `19.989999999999998`. Silent.
- `bindings/napi/src/node.rs:159-161` — a JS number with `fract() != 0.0` → `Value::Float`. Silent.

**The napi boundary undercuts the first-classness argument on its own terms.** JS has no decimal type. `value_to_json` (`node.rs:324-325`) maps `Float` → JSON number; `Value::Bytes` already has to cross as "an Object with numeric-string keys" (`node.rs:327-343`, with a standing Phase-2 note that this is a placeholder). A `Value::Decimal` would cross as a string or as that same object hack. **So `Value::Decimal` does not give TypeScript consumers a decimal.** The cross-language-mirror limb of the first-classness argument does not survive contact with the boundary — and that limb is doing a lot of the work in the request.

**Where my own recommendation is weakest — three concessions:**
1. `Scalar::BytesCid` and `TimestampHlc` are **declared and string-round-tripped only**. Grep for their use outside `vocab.rs:194,195,210,211`, the module doc at `schema_compiler/mod.rs:37-38`, and the audit pin at `g_core_9_non_exhaustive_audit.rs:723-724` returns nothing. **No validation, no renderer special-casing.** The precedent is real as a pattern but currently nominal. `Scalar::Decimal` inherits that thinness unless someone does the work.
2. **The render path is `Value`-driven, not `Scalar`-driven.** `render_value` (`materializer.rs:1121-1141`) is an exhaustive match on `Value` with no schema context in scope. So `Scalar::Decimal` alone would render a decimal-as-Text *as text*. Making it render correctly needs a `ValueRender` method — additive via default body (`materializer.rs:1103,1106` already use default bodies; the pattern is blessed at `benten-eval/src/host.rs:65-67`), but it is work, not free.
3. The `Map{unscaled, scale}` application-layer alternative is genuinely invisible to `render_value` (renders as a nested map) and to the `Scalar` vocabulary today. The requester's first-classness complaint is **not** wrong — it is just aimed at the wrong enum.

---

## Disposition, in HARD-RULE-12 terms

- **`Value::Decimal`** → **DISAGREE-WITH-EXPLANATION.** Not deferred, not out-of-scope. It does not deliver first-classness at the napi boundary, it is arithmetically inert-or-erroring at all 63 `Float` sites, its motivating reframe is refuted by the codec, and the engine already has an unfrozen, additive, twice-precedented seam for exactly this at `Scalar`. Cost is not the argument and I have not used it as one.
- **`Scalar::Decimal` + `ValueRender` decimal method** → genuinely additive post-tag; no deadline; belongs with the Composing schema/renderer work, not the freeze.
- **DO-NOW disclosure (cheap, doc-side, but per rule 15 pair it with the code question):** `docs/SCHEMA-DRIVEN-RENDERING.md` §2.3 and the DSL spec currently describe `Float`/`Scalar::Float` as "64-bit float" with no statement that it is IEEE-754 binary and unsuitable for currency. The value.rs module docs (`value.rs:6-24`) are meticulous about NaN/±Inf/-0.0 and silent on representation error. Asked the rule-15 question — *if we were writing this today, which would we write?* — the answer is doc-side: the behaviour is correct IEEE-754 and we are not changing it, so the disclosure is what's wrong.

---

## The one thing I'd actually put in front of the tag (unrelated to decimal)

**`benten_core::Value` is absent from the freeze doc's §11 enum accounting.** It carries neither `#[non_exhaustive]` (`value.rs:78-80`) nor a carve-out-registry entry. Grepping `docs/V1-FROZEN-INTERFACE.md` for `` `Value` `` / `value::Value` returns **zero** hits; the two `benten-core` registry rows (`:1324-1325`) list `WriteAuthority`, `ChangeEvent`, `ChangeKind`, `Spec`+`SpecError`, `version_dag::*`, `PrimitiveKind`, `SubgraphSpecRestriction` — and no `Value`.

Compare the types that *are* deliberately exhaustive: `Scope` (`:1332`), `Strategy` (`:1333`), `GrantRejection` (`:1323`), `DropContentMode` / `BindingContext` (`:1344`). Each has an explicit registry row **and** a pinning test — e.g. `strategy_carve_out_3_arms_exhaustive_pin` at `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs:161-170`, whose doc-block states the 3-arm shape is load-bearing and a 4th arm is an architectural decision, not a SemVer addition.

That is *precisely* the property `Value` has — 8 arms, wire-keyed by CBOR major type, a 9th arm is an architectural decision — and it is the one type where nobody wrote it down.

To be fair to the record and avoid overclaiming: this is **not** a false record. `docs/V1-FROZEN-INTERFACE.md:1390-1397` honestly discloses that the workspace-wide walker is **deferred to G-COMP-1 per Row D-17**, and that at v1-beta only the enumerated named-set audit runs. So the gap is *disclosed as a class*. It just happens that the single most load-bearing enum in the workspace fell into it.

**Suggested pre-tag action (~30 min, doc + one test):** add a `benten-core | Value | NO (deliberate) | DO NOT APPLY` carve-out row with the wire-keying reason, plus an 8-arm exhaustive pin mirroring `strategy_carve_out_3_arms_exhaustive_pin`. That converts today's *silence* into a *decision*, and it makes any future 9th-variant proposal — decimal or otherwise — a compile-fail that halts and surfaces, which is exactly the governance property the tag is supposed to buy.

---

### Confidence
- Codec tag rejection (both directions): **HIGH** — read `de.rs:211-219,323` and `ser.rs:197-207,614` directly; not executed.
- IVM has no arithmetic; `ViewResult` has no scalar arm: **HIGH** — exhaustive grep + full read of `view.rs:240-260`.
- Arithmetic is a closed `(Int,Float)` matrix: **HIGH** — read all 8 sites.
- napi f64 funnel: **HIGH** — read `node.rs:128-170,315-345`.
- `Value` absent from §11 registry: **HIGH** — targeted greps returned zero; but the freeze doc is ~2500 lines and I searched by symbol, so a differently-worded mention is conceivable. **Verify before acting** per §3.5n.
- `BytesCid`/`TimestampHlc` unenforced: **MEDIUM-HIGH** — grep-based negative; a dynamic/string-keyed consumer could exist that I did not see.