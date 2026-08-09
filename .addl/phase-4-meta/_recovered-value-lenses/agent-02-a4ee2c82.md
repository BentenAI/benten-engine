## Conclusion

**Do not add `Value::Decimal`. It is not implementable as an enum variant at all — now or ever — and the deadline pressure is illusory because every part of the requester's need is post-freeze-additive.**

Three findings, in order of decisiveness:

1. **The major-type-6 reframe is dead.** DAG-CBOR permits exactly one tag (42, CID) and `serde_ipld_dagcbor` hard-rejects every other tag *before* Benten's visitor runs. There is no unclaimed encoding space. Not "we should reserve it" — we *cannot*.
2. **Therefore any `Value::Decimal` must reuse Map/List/Bytes/Text, which under `#[serde(untagged)]` makes it wire-ambiguous with a user's genuine value of that shape** — a silent type-confusion at the content-addressed boundary, i.e. the exact bug class the request exists to prevent, relocated.
3. **The project already solved this problem twice, and the mechanism is `#[non_exhaustive]` in the frozen baseline.** `bytes-cid` and `timestamp-hlc` are first-class, named, schema-visible, admin-UI-visible scalars that are *interpretations layered over* `Value::Bytes` / `Value::Int` — **not** `Value` variants. `decimal` is a ninth `Scalar`, addable after the tag.

The honest answer to the requester's "first-classness, not possibility" counter: **Benten's first-classness mechanism is `Scalar`, not `Value`** — and it is deliberately additive.

---

## Evidence

### The reframe collapses at the codec (high confidence — read the dependency source)

`serde_ipld_dagcbor` v0.6.4 (`Cargo.toml:243`, lockfile-pinned), `src/de.rs:322-324`:

```rust
// The only supported tag is tag 42 (CID).
major::TAG => de.deserialize_cid(visitor),
```

and `src/de.rs:211-219`:

```rust
let tag = dec::TagStart::decode(&mut self.reader)?;
match tag.0 {
    CBOR_TAGS_CID => visitor.visit_newtype_struct(&mut CidDeserializer(self)),
    _ => Err(DecodeError::TypeMismatch { name: "CBOR tag", byte: tag.0 as u8 }),
}
```

Every tag ≠ 42 — including RFC 8949 §3.4 tag 4 (decimal fraction) and tag 5 (bigfloat) — is a decode error. **Tags 4/5 are not "unclaimed," they are actively refused.**

Encode side is equally closed: the *only* `types::Tag(...)` emission in the entire serializer is `src/ser.rs:613-614`, inside `CidSerializer::serialize_bytes`, reachable only via the CID private newtype. Serde's data model has no "tag" concept, so a `#[serde(untagged)]` enum **cannot emit a tag** even if we wanted one.

Major type 7 is likewise closed — `de.rs:337-341` accepts only `FALSE`/`TRUE`/`NULL`/`F32`/`F64` and returns `DecodeError::Unsupported` for all other simple values.

**Net: all 8 CBOR major types are either claimed by an existing `Value` variant or hard-rejected. The encoding has zero extension space.** Claiming a tag would require forking or replacing the codec — a far larger freeze decision than a variant.

### Good news the brief didn't have: `Value` already fails closed

Worth recording, because it satisfies baked-in #5's "typed-reject on unknown, never silent fallback": every unrecognised shape *already* rejects. Non-42 tags → `TypeMismatch`; exotic simple values → `Unsupported`; and a tag-42 CID reaches `visit_newtype_struct`, which `ValueVisitor` (`value.rs:185-292`) does not implement, so serde's default returns `invalid_type`. All wrap into `CoreError::Serialize`.

So `Value`'s decode boundary is fail-closed today. What it lacks is not a reject arm — it is **extension space**, and no variant addition creates any.

### Why a variant is ambiguous, not merely awkward (high confidence)

`Value` is `#[derive(..., Serialize)] #[serde(untagged)]` (`value.rs:78-79`) — no discriminant on the wire; variant identity **is** CBOR major type. A `Decimal { unscaled, scale }` serialises untagged as a 2-key map. Decoding, `ValueVisitor::visit_map` (`value.rs:279`) yields `Value::Map`. So:

- `Decimal → bytes → Map` — round-trip loss; and
- if the visitor sniffs for the magic key-pair to recover it, a user's genuine `Map{"scale":…,"unscaled":…}` decodes as `Decimal` — **two distinct `Value`s with one CID.**

Both directions break. This is not fixable by choosing a different shape; it follows from `untagged` + a saturated major-type space.

*(Aside, low-stakes: `de.rs:307-313` only widens to `i128` outside `i64` range, so a mantissa is effectively `i64` — ~18 significant digits. Fine for 3dp money, but not arbitrary precision.)*

### The precedent that answers "first-classness"

`docs/SCHEMA-DRIVEN-RENDERING.md:52`:

> Scalar-to-`Value` mapping is many-to-one: `bytes-cid` + `timestamp-hlc` are *interpretations layered over* `Value::Bytes` + `Value::Int` respectively, not distinct `Value` variants.

`crates/benten-platform-foundation/src/schema_compiler/vocab.rs:161-163`:

```rust
// §11 SemVer-readiness (F-22 pre-tag): a future scalar kind lands additively; cross-crate consumers add a `_` wildcard arm.
#[non_exhaustive]
pub enum Scalar {
```

And that attribute is **already in the frozen baseline** — `docs/public-api/benten-platform-foundation.txt:1460`. A ninth `Scalar` is a semver-minor addition after the tag, by construction.

The requester's consumers are exactly the ones `Scalar` reaches: the schema vocabulary, the DSL, the materializer (`materializer.rs:1097-1100` names a `Decimal` mint by name as a one-site change), `ADMIN-UI.md:36-44`'s field-type table, cross-language mirrors.

**And the engine anticipated this in writing** — `materializer.rs:1097-1100` literally says *"A future Phase-4-Meta `Value` scalar mint (e.g. `Decimal` / `Timestamp`) touches exactly one place."* The word used is **scalar**, not variant.

---

## Honest costs

### If a variant were added anyway — the real price

**Compile-forced (5 sites, not 9 — I over-counted first pass and corrected by checking each scrutinee):**

| Site | Function |
|---|---|
| `crates/benten-core/src/value.rs:374` | `to_canonical` |
| `crates/benten-core/src/value.rs:427` | `is_already_canonical` |
| `crates/benten-eval/src/expr/eval.rs:506` | `truthy` |
| `crates/benten-platform-foundation/src/materializer.rs:1123` | `render_value` |
| `bindings/napi/src/node.rs:320` | `value_to_json` |

**Not compile-broken but semantically dead-ended (3):** `bindings/napi/src/node.rs:133`, `bindings/napi/src/lib.rs:2667`, `bindings/napi/src/wait.rs:199` — all match `serde_json::Value`, so they still compile while being unable to *produce* a Decimal. JSON has no decimal type, and `Value::Bytes` already burns the only escape hatch (an object with numeric-string keys, `node.rs:327-349`, re-sniffed by heuristic at `node.rs:198-227`). A second magic-object shape would collide with that heuristic.

**Compiles-but-silently-wrong — the actual cost centre (~21 functions).** Every numeric path has a `_ => Err(type mismatch)` catch-all, so a Decimal falls into the error arm with no compiler help: `add:518`, `numeric_op:529`, `div:~550`, `rem:~570`, `compare:579`, `values_equal:596`, `negate:220` (all `eval.rs`), plus 13 `builtins.rs` arithmetic fns (`arith_abs:197` … `arith_sign:348`).

Adding Decimal turns a 2×2 Int/Float coercion matrix into 3×3, and the poisoned cell has no good answer: `Float + Decimal` either returns `Float` (silently destroying the exactness the variant existed to provide — the original bug, reintroduced) or becomes a typed reject (new ErrorCode, `CATALOG_VARIANT_COUNT` 201→202, the ~7-site mint path per `docs/ERROR-CATALOG.md:10`). `Decimal / Decimal` is not closed over fixed scale and needs a **frozen-forever rounding-mode policy**. `values_equal(Int(1), Decimal(1.00))` feeds `BinaryOp::Eq` → handler branching → content-addressed writes.

That is a **numeric tower**, and per `feedback_addl_pipeline_full_observance` it warrants a full R0→R4 pipeline, not a pre-tag patch.

**Costs that are NOT there (under-claiming, verified):**
- **`Ord`/`PartialOrd`/`Eq`/`Hash`: absent.** `docs/public-api/benten-core.txt:734-751` lists only `Clone`/`PartialEq`/`Debug`/`StructuralPartialEq`/`Serialize`/`Deserialize` + auto-traits. `Map` keys are `String`. The brief's hunch was right — **no ordering cost.**
- **The frozen-bytes corpus does not break.** All 123 targets (`.github/frozen-bytes-corpus.txt`) pin existing encodings; a new variant leaves every existing `Value`'s bytes and every Node CID unchanged. Only `canonical_bytes_fastpath_stable` (line 28) needs a new case.
- Baselines are mechanical: `docs/public-api/benten-core.txt:720-728` (+1 line), regenerated via the tool never hand-authored (`feedback_baseline_regen_must_match_ci_tool_invocation`).

**Doc/mirror cascade:** `packages/engine/src/types.ts:33-40` (structural union — a Decimal is `{[k:string]:Value}`, indistinguishable in TS too) · `packages/engine/etc/public-api.txt` · `SCHEMA-DRIVEN-RENDERING.md:50-62` · `ADMIN-UI.md:36-44` · `GLOSSARY.md:67` · `V1-WIRE-FORMAT-INVENTORY.md:16-31` · three 8-scalar count pins (`schema_compiler_typed_field_vocab_…:65`, `tf5_46_…:107`, `schema_fixtures.rs:70`).

### One day or a week?

**Mechanical shell: ~1 day.** 5 matches, 2 baselines, TS mirror, doc cascade.
**Correct and frozen: not a week — it is a full ADDL pipeline, and it still lands on an ambiguous encoding at the end.** The mechanical day is the trap: it produces something that compiles, passes the corpus, and is wrong on the wire.

### Now-or-never vs. genuinely later

| Item | Verdict |
|---|---|
| `Value::Decimal` variant | **Never** — not now-or-never. Blocked by the codec, not the calendar. |
| `decimal` as 9th `Scalar` | **Later, safely.** `#[non_exhaustive]` already in the frozen baseline. |
| Decimal arithmetic builtins | **Later.** `dispatch_namespaced` (`builtins.rs:124`) is evaluator surface, not wire — purely additive. |
| `#[non_exhaustive]` on `Value` | **Now-or-never, and I recommend AGAINST** (see below). |
| Reserved property-key namespace | **Genuinely now-or-never. Surfacing for Ben.** |

**On `#[non_exhaustive]` for `Value`** — the naive "add it for agility" instinct is wrong here, and I'd have gotten it wrong without checking. `Value` lacks it (`value.rs:78`) while siblings have it (`WriteContext`, `CoreError` at `lib.rs:122`/`:757`, `Scalar`). But `GLOSSARY.md:93` records the project's *considered* stance on `MembershipSetKind`: *"a 4th arm is a compile-time halt, deliberately NOT a `#[non_exhaustive]` wildcard."* Adding it to `Value` would remove the compile-halt at the 3 cross-crate sites (`eval.rs:506`, `materializer.rs:1123`, `node.rs:320`) where silently defaulting a new variant is exactly a correctness bug. The brief's own observation — that `to_canonical`'s exhaustive match means *"the decision cannot be made accidentally"* — is the argument for leaving it off. **Keep the halt.**

**The one item I'd actually put in front of Ben** (medium confidence, my inference, not a demand): `system:` is reserved for *labels and node IDs* only (`redb_backend.rs:65`, `transaction.rs:372`, `GLOSSARY.md:159`) — **there is no reserved namespace for `Value::Map` property keys.** Any future layered scalar (decimal, or anything else) will need a magic key, and reserving that namespace *after* users hold data is a compatibility break in a content-addressed store. Reserving a prefix now is cheap and is the only real agility seam available to `Value` given the codec. Whether it is worth doing is a P-III-shaped call, not mine.

### What the recommendation costs the requester — stated plainly

They get exact decimal **storage** and schema/admin/DSL first-classness. They do **not** get decimal arithmetic inside TRANSFORM expressions until the additive builtins wave lands, so summing a tax column happens in application code or a SANDBOX module. And at the TS boundary their decimals are objects, not `number` — which is correct (JS `number` is IEEE-754 f64, so a `Value::Decimal` would have been destroyed crossing napi anyway, `node.rs:136-165`). That last point is worth telling them directly: **the variant they asked for would not have survived their own client boundary.**