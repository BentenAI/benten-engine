Verification complete. Writing the verdict.

## VERDICT: PARTIAL — core conclusion CONFIRMED on stronger evidence than the analysis supplied; two citation defects, one inverted citation, five substantive misses

Base verified: `bf964354`, tree clean. All claims re-derived from source, not from the analysis's summaries.

---

## Per-claim verdicts

### Claim 1 — "The major-type-6 reframe is dead" → **CONFIRMED (high confidence)**

Verified against the pinned dependency source, not docs. `Cargo.toml:243` = `serde_ipld_dagcbor = "0.6.4"`; `Cargo.lock` checksum `46182f4f…`. In `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_ipld_dagcbor-0.6.4/`:

- `src/de.rs:323-324` — `// The only supported tag is tag 42 (CID).` / `major::TAG => de.deserialize_cid(visitor),` — **quoted verbatim, line-exact.**
- `src/de.rs:211-219` — the `match tag.0 { CBOR_TAGS_CID => …, _ => Err(DecodeError::TypeMismatch { name: "CBOR tag", … }) }` — **line-exact.**
- `src/lib.rs:148` — `const CBOR_TAGS_CID: u64 = 42;`
- `src/ser.rs:614` — the sole `types::Tag(CBOR_TAGS_CID, …)` emission; `grep` for `Tag` across `ser.rs` returns only lines 14, 18, 197, 202, 614, 642. Confirmed: one emission site, hardcoded to 42.
- `src/de.rs:337-341` — major type 7 accepts only `FALSE`/`TRUE`/`NULL`/`F32`/`F64`, else `DecodeError::Unsupported`. Confirmed.

RFC 8949 tags 4 and 5 are **actively refused**, not unclaimed. The reframe genuinely collapses. One refinement below (Miss J) on *why* it collapses.

### Claim — "`Value` already fails closed" → **CONFIRMED**

`ValueVisitor` (`crates/benten-core/src/value.rs:185-292`) implements `visit_unit/none/some/bool/i64/i128/u64/u128/f64/f32/str/string/bytes/byte_buf/seq/map` — and **no `visit_newtype_struct`**. So a tag-42 CID reaches serde's default and errors `invalid_type`. Correct as stated. Not noted by the analysis: this also means `Value` **cannot round-trip an IPLD CID link in a property** — a real interop limitation for an external team expecting standard IPLD.

### Claim — "5 compile-forced sites" → **CONFIRMED, and complete**

I did not trust the list. I ran the necessary condition (an exhaustive `match` on `Value` must cover `Null`) across `crates/`, `bindings/`, `packages/`, then read every hit:

| Site | Exhaustive? | |
|---|---|---|
| `value.rs:373` `to_canonical` | 8 arms, no wildcard | ✅ forced |
| `value.rs:426` `is_already_canonical` | 8 arms via `\|`-group | ✅ forced |
| `eval.rs:506` `truthy` | 8 arms, no wildcard | ✅ forced |
| `materializer.rs:1123` `render_value` | 8 arms, no wildcard | ✅ forced |
| `node.rs:320` `value_to_json` | 8 arms, no wildcard | ✅ forced |
| `builtins.rs:515` (`join`) | `_ => format!("{v:?}")` | not forced |
| `builtins.rs:814` (`toString`) | `other => format!("{other:?}")` | not forced |
| `builtins.rs:862` (`isEmpty`) | `_ => false` | not forced |
| `proptests_value_json_cbor.rs:55` | `other => panic!` | not forced |

No glob imports (`use …Value::*`) exist to hide a site. The count is right and the correction it says it made ("5 not 9") holds.

### Claim — "no `Ord`/`PartialOrd`/`Eq`/`Hash` cost" → **CONFIRMED**

`docs/public-api/benten-core.txt:734-751` lists exactly `Clone`, `PartialEq`, `Debug`, `StructuralPartialEq`, `Serialize`, `Deserialize`, plus auto-traits. No ordering traits. The analysis's under-claiming is honest.

### Claim — corpus / `Scalar` / doc quotes → **CONFIRMED, line-exact**

- `.github/frozen-bytes-corpus.txt` — non-comment non-blank lines = **exactly 123**; `canonical_bytes_fastpath_stable` at **line 28**. Both exact.
- `vocab.rs:161-163` — `#[non_exhaustive]` + the §11 SemVer comment, **verbatim**.
- `docs/public-api/benten-platform-foundation.txt:1460` — `#[non_exhaustive] pub enum …Scalar`. Exact.
- `SCHEMA-DRIVEN-RENDERING.md:52` — "interpretations layered over" quote, **verbatim**.
- `GLOSSARY.md:93` — MembershipSetKind "deliberately NOT a `#[non_exhaustive]` wildcard", **verbatim**.
- `types.ts:33-40`, `tf5_46_…:107`, `ERROR-CATALOG.md` count 201 — all confirmed.
- `node.rs:136-165` — JS numbers resolve through `as_i64`/`as_f64` to `Int`/`Float`. The closing point ("the variant they asked for would not have survived their own client boundary") is **CONFIRMED and is the analysis's strongest single argument**.

### Claim 2 — "any Decimal is wire-ambiguous" → **CONFIRMED, but stated too weakly** (see Miss H)

### Claim 3 — "the project already solved this twice; `Scalar` is the mechanism" → **PARTIAL**

The `bytes-cid` / `timestamp-hlc` precedent is real and correctly cited. But the supporting `materializer.rs` citation is deployed backwards — see below.

---

## Defects

**D1 — dead path.** `crates/benten-graph/src/backends/redb_backend.rs:65` **does not exist**. Real file: `crates/benten-graph/src/redb_backend.rs` (`guard_system_zone_node` at `:60`, the `system:` check at `:65` — line correct, path wrong). A `backends/` directory does exist (`blob_backend.rs`), which is likely how it crept in. Substance survives via `GLOSSARY.md:159`, which I confirmed independently.

**D2 — baseline churn is +2, not +1.** `benten-core.txt` lists `Value` **twice**: module path at `:720-728` and crate-root re-export at `:1062-1070` (`grep -c "::Value::Null"` = 2). The analysis's `:720-728` cite is correct — I initially mis-scored it as an error because my grep for `benten_core::Value` missed `benten_core::value::Value`; retracted. But "+1 line" is wrong.

**D3 — off-by-one.** `schema_fixtures.rs:70` → actually `:69`.

**D4 — "semver-minor by construction" overstates.** `#[non_exhaustive]` does not apply **within the defining crate**. `vocab.rs:189/195` (`as_str`) and `:205/211` (`from_str`) are exhaustive intra-crate matches a 9th `Scalar` breaks. Small, but not zero.

**D5 (most important) — one citation is deployed backwards; this is the advocacy tell.**

The analysis quotes `materializer.rs:1097-1100` and concludes: *"the engine anticipated this in writing… The word used is **scalar**, not variant."*

Read the full comment (`materializer.rs:1093-1100`):

> A future Phase-4-Meta `Value` scalar mint (e.g. `Decimal` / `Timestamp`) touches exactly one place (**the `render_value` dispatch** + each rule's new arm via the trait), not 3-4 duplicated dispatch sites.

`render_value` (`:1123`) matches on **`Value` variants**. A new `Scalar` — the `bytes-cid` pattern the analysis champions — maps onto an *existing* `Value` variant and requires **zero** `render_value` change. So this comment can only be describing a new **`Value` variant**. And `ValueRender`'s methods have default bodies (`:1103-1105`), which is exactly why the author could claim "exactly one place."

This is contemporaneous evidence that the codebase authors contemplated `Value::Decimal` as a real future **variant** mint and pre-refactored to make it cheap. The analysis cites it as proof of the opposite. Everything else in the analysis is careful; this one paragraph argues rather than reads.

---

## What it MISSED

**M1 — doc-coupling sites absent from the cascade (§3.5b).** `crates/benten-core/INTERNALS.md:62` ("The `Value` enum (eight variants: `Null`, `Bool`…)" — full enumeration) and `:178` ("round-trip each of the eight `Value` variants"); `schema_compiler/mod.rs:36-38` ("8 scalars"). None listed.

**M2 — the silently-wrong bucket is mis-scoped as arithmetic.** String coercion is poisoned too, and worse: `coerce_to_string` (`builtins.rs:807-816`, `other => format!("{other:?}")`) and `join` (`builtins.rs:510-517`, `_ => format!("{v:?}")`) would emit **Rust `Debug` output** — `Decimal { unscaled: 1234, scale: 2 }` — into user-visible strings. For a museum till receipt that is a worse, equally silent failure than the f64 bug the request exists to prevent. `coerce_is_empty` (`:861-866`) silently returns `false`. The compiler flags none of these.

**M3 — the CID/`PartialEq` break is unconditional, not sniffing-only.** The analysis pins "two distinct `Value`s with one CID" to the sniffing design. It holds for the **non**-sniffing design too: `Decimal{1234,2}` and `Map{unscaled:1234,scale:2}` encode to identical bytes → identical CID, while derived `PartialEq` (`benten-core.txt:736`) reports them unequal. *Both* branches break "equal bytes ⟺ equal value" in a content-addressed store. This strengthens the conclusion and should have been unconditional.

**M4 — one design never named: drop `#[serde(untagged)]`.** Pre-freeze is precisely the moment one *could* give `Value` a real discriminant, which is the only construction that delivers an unambiguous Decimal. It must be **rejected explicitly**, not omitted: it re-encodes every existing `Value`, invalidating the sentinel CID `bafyr4if…mwduda` (`V1-WIRE-FORMAT-INVENTORY.md` §1) and all 123 corpus targets, and abandons IPLD interop. A reader who thinks of it and doesn't find it addressed will discount the whole analysis.

**M5 — the permanence argument rests one level too shallow.** The analysis grounds "never" in the codec: *"would require forking or replacing the codec."* Crates are forkable; that phrasing leaves the door visibly ajar for the next agent. The durable ground is that **DAG-CBOR conformance is itself inside the freeze**: multicodec `0x71` is baked into every CID (`V1-WIRE-FORMAT-INVENTORY.md` §1), and per CLAUDE.md baked-in #5 the permanent commitment is the multiformats *framing*. Tag 4 is unavailable because Benten's CIDs assert "this is dag-cbor" and dag-cbor permits one tag — not because of a dependency choice.

**M6 (process) — the "do nothing" recommendation does not discharge HARD-RULE-12.** This lands as clause (c) DISAGREE-with-explanation, which is legitimate. But the reasoning has no named destination that receives the entry now. `V1-WIRE-FORMAT-INVENTORY.md` §1 freezes `Node`/`Edge` fields ("new fields cannot be added… without a P-III re-decision") and is **silent on the `Value` variant set** — the very thing the analysis concludes is closed forever by construction. Without a row recording that, the next external request re-litigates from zero, post-tag, as the governance event the brief warns about. Recording it is the one obligation a "do nothing" answer still carries, and the analysis's own deadline table omits it.

---

## Bottom line

The recommendation — **do not add `Value::Decimal`; a 9th `Scalar` is the mechanism** — survives adversarial review, and the codec evidence is stronger and more precisely cited than the analysis needed it to be. Its deadline engagement is real (the now-or-never table is the right shape, and the reserved-property-key-namespace item surfaced for Ben is a genuine catch).

Three things must change before this is acted on: **strike the `materializer.rs:1097-1100` argument** (it says the opposite of what it's quoted for, and its survival would let a later reader conclude the analysis was reverse-engineered from its conclusion); **add M4 and M5** so the two obvious counter-moves are visibly closed rather than absent; and **add M6's destination row**, without which the DISAGREE is undischarged. D1-D4 are corrections, not threats to the conclusion.