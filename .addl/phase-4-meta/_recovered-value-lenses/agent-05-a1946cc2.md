Read-only confirmed: tree clean at `bf964354`, nothing modified.

---

# ADVERSARIAL VERDICT — "does the engine actually need this, given how money is modelled here"

**Bottom line: the recommendation survives; roughly a third of its reasoning does not.** The central claim (do not add `Value::Decimal`) is correct and I could not break it. But one of its four headline findings is **REFUTED as framed**, it missed an in-repo design note that names `Decimal` by name for *this exact phase*, and its "no deadline pressure anywhere" conclusion is **wrong in one specific place** it never examined.

Citation accuracy was high — I opened every file:line and found no fabrications. Two-ish off-by-one line numbers, no false existence claims in either direction.

---

## Claim-by-claim

### 1. DAG-CBOR rejects all tags but 42, both directions — **CONFIRMED**

Verified against source, not docs. Decode: `serde_ipld_dagcbor-0.6.4/src/de.rs:211-219` — `match tag.0 { CBOR_TAGS_CID => …, _ => Err(DecodeError::TypeMismatch{name:"CBOR tag"}) }`, reached from the `deserialize_any` major-type dispatch at `de.rs:324` (`// The only supported tag is tag 42 (CID).`). `CBOR_TAGS_CID = 42` at `lib.rs:148`. Encode: the *only* `types::Tag(..)` construction in the entire crate is `ser.rs:614`, hardcoded to 42, reachable only via `serialize_newtype_struct` when `name == CID_SERDE_PRIVATE_IDENTIFIER` (`ser.rs:202-203`). Tags 4 and 5 are unreachable in both directions. The reframe is dead.

**But the analysis made the weaker of two available arguments.** It said emitting tag 4 "would require forking the codec." True, and irrelevant — the binding constraint is that IPLD dag-cbor *as a profile* admits only tag 42, so a forked codec would emit bytes that are not dag-cbor. The argument is not "we'd have to fork" (a cost argument, which HARD-RULE-12 disallows anyway) but "the output would no longer be the format we froze." Worth restating, because the cost-flavoured version is exactly the kind of reasoning rule 12 exists to reject.

### 2. `Value` rejects even tag 42 — **CONFIRMED**

`ValueVisitor` (`crates/benten-core/src/value.rs:185-292`) implements 16 visit methods; `visit_newtype_struct` is not among them. serde's default is `Err(Error::invalid_type(Unexpected::NewtypeStruct, &self))` — verified at `serde-1.0.229/src/core/de/mod.rs:1671-1677`. CID-as-`Value::Text` spot-checks all exact: `blob_backend.rs:273`, `engine_modules.rs:329`, `engine_caps.rs:647`.

### 3. Arithmetic is a closed `(Int, Float)` matrix with `_ => Err` — **CONFIRMED but INCOMPLETE**

All eight cited sites read verbatim as described. Line numbers off by one at three (`modulo` is :564 not :563; `arith_product` :307 not :306; `cmp_values` :968 not :967).

**Three sites missed**, one of which damages the analysis's own argument:
- `eval.rs:579 compare` — a *second* comparison matrix distinct from `builtins.rs:968 cmp_values`.
- `builtins.rs:926 as_i64` — `Value::Float(f) if f.fract() == 0.0 => Some(*f as i64)`.
- **`eval.rs:596 values_equal`** — the important one:
  ```rust
  (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
  (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
  _ => l == r,
  ```
  A `Decimal` falls to `_ => l == r`. That is **silently false**, not a typed error. So the analysis's cleanest rhetorical move — "a storage-only Decimal converts silent-wrongness into hard-failure" — is not uniformly true. There is at least one path where a Decimal would be silently unequal to a numerically-equal Int. Fail-closed is not the whole story.

### 4. `Value::Float` at 63 non-test sites / 16 files / 9 crates — **CONFIRMED (under-claimed)**

16 files exact match, 9 crates exact match. My count is 68 excluding `/tests/` directories (82 total); the 63 presumably also excludes inline `#[cfg(test)]` blocks. Under-claiming, per the brief's instruction. Fine.

### 5. IVM does no arithmetic; `ViewResult` has no scalar arm — **CONFIRMED**

Exhaustive grep of `crates/benten-ivm/src/` for `Value::Int|Value::Float` returns 12 hits: ordering keys (`algorithm_b.rs:1193,1196`; `content_listing.rs:474`), structural counts (`governance_inheritance.rs:277,281`), test fixtures (`capability_grants.rs:314`, `governance_inheritance.rs:313`, `algorithm_b.rs:1537,1544`), and three doc-comments. Zero domain aggregation. `ViewResult` at `view.rs:249-258` (analysis said 248-257) is `#[non_exhaustive]` with the F-22 comment and exactly 3 Cid/Cid/Rules arms — no scalar. A balance view is genuinely inexpressible. The asymmetry argument (aggregation half is additive post-tag, `Value` is not) holds.

### 6. `Scalar` is the unfrozen additive seam — **CONFIRMED**

`vocab.rs:162` `#[non_exhaustive]`, `:163` `pub enum Scalar`, `:176` `BytesCid`, `:179` `TimestampHlc`, doc-comments quoted accurately.

**And the analysis under-rated its own concession.** It marked "BytesCid/TimestampHlc are unenforced" at MEDIUM-HIGH. It is HIGH: a workspace-wide grep for both symbols returns exactly four hits outside `vocab.rs` — two module-doc references (`schema_compiler/mod.rs:37-38`) and two arms of an audit test (`g_core_9_non_exhaustive_audit.rs:723-724`). No validation, no rendering, no consumer. The precedent is real as a *pattern* and entirely nominal as an *implementation*.

### 7. `Value` absent from the freeze doc's §11 enum accounting — **CONFIRMED, and understated**

`docs/V1-FROZEN-INTERFACE.md` is 2367 lines. Exactly **one** line contains the string "Value": `:741`, a table header `| Surface | Constant | Value | Status at v1-beta |`. The `benten-core` registry rows (`:1324-1325`) and the F-22 EXTENSION set (`:1327`) enumerate `WriteAuthority / ChangeEvent / ChangeKind / Spec+SpecError / version_dag::* / PrimitiveKind / SubgraphSpecRestriction / Mode / VersionError / VersionDagError / Anchor / VersionDag / DagVersionChain / Subgraph / SubgraphBuilder / NodeHandle` — no `Value`. The G-COMP-1 walker deferral is honestly disclosed at `:1390-1397`. Not a false record; a genuine hole.

**One fairness correction the analysis owed and didn't give:** §4 (`:521+`) *does* freeze "The Phase-1 canonical Node/Edge DAG-CBOR encoding family (CIDv1 + BLAKE3-256 + multihash `0x1e` + multicodec `0x71`)", which implicitly covers `Value`'s bytes, plus the sentinel-CID round-trip pin. So the **wire** is frozen; it is the **variant-set accounting** that is missing. Stating it as flatly absent overstates by a hair.

### 8. napi / DSL f64 funnel — **CONFIRMED, and worse than described**

`node.rs:157-161`, `node.rs:319-345`, `dsl-compiler/src/lib.rs:1072-1081` all read as cited. The analysis missed that `node.rs:157` routes `f.fract() == 0.0` to `Value::Int`, not `Value::Float`. So a JS amount of `19.00` crosses as `Int(19)` and `19.99` crosses as `Float(19.99)` — **the variant depends on the value**. For money that is arguably worse than the float imprecision itself: the stored type of a price column changes row by row. Strengthens the analysis's own conclusion that `Value::Decimal` would not deliver first-classness at this boundary.

### 9. "The engine does not model money" — **REFUTED**

This is the framing underneath Findings 2 and 3, and it is false.

`crates/benten-eval/src/expr/builtins.rs:895-905`:
```rust
fn fmt_currency(args: &[Value]) -> Result<Value, EvalError> {
    let n = to_f64(args[0].clone())?;
    let code = match &args[1] { Value::Text(s) => s.clone(),
        _ => return Err(EvalError::new("currency code must be string")) };
    Ok(Value::Text(format!("{n:.2} {code}")))
}
```

Benten ships a **`formatCurrency` builtin**. It funnels through `to_f64` (`:918`, Int/Float only) and hardcodes `{n:.2}` — two decimal places **regardless of the currency code it was just handed**. JPY has 0 minor-unit digits; KWD, BHD and TND have 3. `formatCurrency(1000, "JPY")` renders `1000.00 JPY`. And `{:.2}` on an f64 rounds the *binary* value, so the canonical `1.005 → "1.00"` error is present in a function named for money. Siblings `fmt_number:875` and `fmt_percent:885` share the `to_f64` funnel.

This does not flip the recommendation — a `Value::Decimal` could not even reach `fmt_currency`, since `to_f64` would reject it. But it refutes the premise. The honest statement is *not* "the engine has no money model"; it is **"the engine has a money-shaped display surface that is independently wrong, sits in `benten-eval` outside the frozen wire, and nobody looked at it."** Given the lens was explicitly "how money is modelled here," missing the one function with `currency` in its name is a material gap.

---

## What it missed

**A. `fmt_currency` (above).** The highest value-per-LOC item in this entire review and absent from the analysis. Scale-aware formatting keyed off the ISO-4217 exponent is a `benten-eval` change — no wire bytes, no public enum, no baseline. Under rule 15 the deciding question is "if we were writing this today, which would we write?" and hardcoded-2dp is not what anyone would write. Nuance to state honestly: `fmt_currency` returns `Value::Text` which a caller may persist, so changing it changes *future* CIDs of nodes built from it — not past ones, not the format. Not a freeze break.

**B. In-repo prior intent naming `Decimal`, for this phase.** `crates/benten-platform-foundation/src/materializer.rs:1097-1100`:

> "A future Phase-4-Meta `Value` scalar mint (e.g. `Decimal` / `Timestamp`) touches exactly one place (the `render_value` dispatch + each rule's new arm via the trait), not 3-4 duplicated dispatch sites."

The project already contemplated `Value::Decimal` **by name**, and placed it in **Phase-4-Meta** — the phase closing now. A review whose entire job is "mint this variant before the freeze?" must engage with prior in-repo intent naming the exact variant in the exact phase. It also cuts against the analysis's own concession #2 (render work "is work, not free"): the Qual-1 #730 refactor was done precisely so this is a one-place change. Note also that no `docs/` file anywhere mentions `Decimal` in this sense — grep across `docs/**.md` returns only unrelated "decimal-array rendering" hits. So this intent lives in a code comment and never reached the freeze record: a doc-coupling gap in its own right (§3.5b).

**C. The strongest argument for the analysis's own conclusion, unmade: `Value` is structurally saturated.** Walk `de.rs:308-341`: major 0/1 → `Int`; 2 → `Bytes`; 3 → `Text`; 4 → `List`; 5 → `Map`; 7 → `Bool`/`Null`/`Float`; 6 → tag → CID-only → `visit_newtype_struct` → rejected. **Every dag-cbor major type is claimed.** Under `#[serde(untagged)]` with no discriminant (`value.rs:79`), a 9th variant cannot get its own major type and must share one — which costs round-trip identity. So `Value` has been closed at 8 variants **since the untagged design was chosen**, not since this freeze. That reframes the answer from "we decline to add decimal" to "this was foreclosed years ago and the tag is merely where we write it down" — a far more durable position, and it makes the analysis's own carve-out-row suggestion more load-bearing than it presented.

**D. The one place the deadline genuinely binds — and the analysis concluded the opposite.** Tag 42 is the single dag-cbor-legal, codec-supported, `Value`-unclaimed slot in the encoding. Claiming it means `Value::Link(Cid)`. The analysis *noticed* CIDs-are-stored-as-`Value::Text` and used it only as ammunition against decimal — it never followed through to the fact that this is precisely where "claim it now or leave it permanently unusable" is **true**. Its blanket "no deadline pressure" is therefore wrong in one specific spot. Whether to mint `Value::Link` is a Ben fork, not mine, and it is out of scope for a decimal request — but a freeze review that surfaces "no deadline anywhere" when exactly one deadline exists has mis-served the tag.

**E. Round-trip coverage is thinner than "the decision can't be made accidentally" implies.** `to_canonical` (`value.rs:373-406`) and `is_already_canonical` (`:426`) are genuinely exhaustive — compile error confirmed. But `crates/benten-core/tests/proptests_value_json_cbor.rs:24-33` generates **only** Null/Bool/Int/Text and `panic!`s on anything else as a tripwire that its own strategy never fires. A variant colliding on a major type would compile-fail at canonicalization but would **not** be caught by the round-trip property suite.

**F. Cost estimate incomplete.** "63 sites is where the design work is" is right about design and silent on freeze mechanics: 15 `cargo-public-api` baselines, the `errors.generated.ts` TS mirror, the napi boundary, and the `g_core_9_non_exhaustive_audit` pins. Directionally harmless (it argues *against* adding), but incomplete.

---

## Advocacy check

Mildly conclusion-led — it opens with the verdict and Finding 3's "the freeze pressure is on the half they don't need" is rhetorical (the requester never asked for IVM aggregation). Against that: it carries a genuine three-item "where my own recommendation is weakest" section, it concedes the silent-f64 bug is real and present, it concedes the requester's first-classness complaint is "not wrong — just aimed at the wrong enum," and it under-claims its own confidence in at least one place. On balance **analysing, not advocating.**

---

## Net

| | |
|---|---|
| **Recommendation (do not add `Value::Decimal`)** | **SURVIVES** — and is better supported by argument C than by anything the analysis actually wrote |
| Reframe is dead (tags) | CONFIRMED |
| `Scalar` is the right, unfrozen seam | CONFIRMED |
| `Value` missing from §11 | CONFIRMED, understated; §4 fairness correction owed |
| "Engine doesn't model money" | **REFUTED** — `fmt_currency` |
| "No deadline pressure" | **PARTIAL** — true for decimal, false for tag 42 / `Link` |
| "Storage-only Decimal fails closed" | **PARTIAL** — `values_equal:596` is silent-false |

**Three additions I'd put in front of the tag that the analysis did not surface:** (1) the `Value` carve-out row + 8-arm exhaustive pin, on the *saturation* rationale rather than the decimal one; (2) `fmt_currency` scale-awareness, or an honest doc statement that it is 2dp-only — code-side per rule 15, and outside the freeze either way; (3) `Value::Link(Cid)` surfaced to Ben as the one genuinely deadline-bound slot, explicitly decided rather than defaulted.