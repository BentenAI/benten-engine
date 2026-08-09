## CONCLUSION

**The reframe collapses. CBOR major type 6 is not unclaimed — it is *spec-forbidden*.** DAG-CBOR normatively permits exactly one tag (42, CID). Tag 4 is rejected by the codec before `ValueVisitor` is ever consulted, and again, independently, by the napi pre-scanner. There is no tag-based agility seam to claim, now or ever, without leaving the multicodec `0x71` profile that Benten's own CIDs assert.

Three further findings change the shape of the decision:

1. **No major type is free.** `Value`'s 8 variants exhaust the IPLD Data Model — *except* Link. Tag-42 CIDs also fail to decode into `Value`. **A Benten Node property cannot hold a CID link.** In a content-addressed graph engine, that is a larger freeze question than `Decimal`, and it is entirely unrecorded.
2. **The only real seam is Rust-side, and it is missing.** `Value` has no `#[non_exhaustive]`, while 9 sibling types in the same crate do. That — not a wire change — is the genuine last-moment, zero-wire-cost decision.
3. **The requester's "silent failure" framing is wrong for Benten, but the engine has a worse float bug of its own:** `Int / Int` in TRANSFORM has a **data-dependent return type**, silently promoting to `f64`. That defeats the "just use integer minor units" mitigation from inside the engine, independent of any adopter.

---

## EVIDENCE

### 1. DAG-CBOR forbids tags — spec, not crate quirk

IPLD dag-cbor spec (fetched): *"Use no tags other than the CID tag (`42`). A valid DAG-CBOR encoder must not encode using any additional tags and a valid DAG-CBOR decoder must reject objects containing additional tags as invalid."* This explicitly names RFC 8949's predefined tags (dates, bignums) as forbidden. **Tags 4 and 5 are exactly the class the profile excludes.**

Implementation, `serde_ipld_dagcbor` 0.6.4 (`Cargo.lock:6550`), `src/de.rs:211-220`:

```rust
let tag = dec::TagStart::decode(&mut self.reader)?;
match tag.0 {
    CBOR_TAGS_CID => visitor.visit_newtype_struct(&mut CidDeserializer(self)),
    _ => Err(DecodeError::TypeMismatch { name: "CBOR tag", byte: tag.0 as u8 }),
}
```

`src/de.rs:323-324` — `// The only supported tag is tag 42 (CID).` `src/lib.rs:148` — `const CBOR_TAGS_CID: u64 = 42;`

**Empirically** (probe at `/private/tmp/claude-502/.../scratchpad/tagprobe`, linked against `benten-core` at `bf964354`):

| input | result |
|---|---|
| `c4 82 21 19 6ab3` — tag 4 decimal 273.15 | `ERR TypeMismatch { name: "CBOR tag", byte: 4 }` |
| tag 5 bigfloat | `ERR TypeMismatch { byte: 5 }` |
| tag 2 bignum | `ERR TypeMismatch { byte: 2 }` |
| tag 0 datetime | `ERR TypeMismatch { byte: 0 }` |
| **tag 42 CID** | `ERR Msg("invalid type: newtype struct, expected a DAG-CBOR value")` |
| control: `82 21 19 6ab3` untagged array | `OK List([Int(-2), Int(27315)])` |
| control: `Map{scale,unscaled}` | `OK Map({"scale": Int(-2), "unscaled": Int(27315)})` |

Same failure at Node level: `{"labels":["Sale"],"properties":{"amt":4([-2,27315])}}` → `ERR TypeMismatch { byte: 4 }`.

**Second, independent enforcement layer** — `bindings/napi/src/input_limits.rs:367-378`, which never consults the codec:

```rust
// Tag. DAG-CBOR permits exactly one: 42, the CID link.
6 => {
    if arg != 42 { return Err(malformed("tag other than 42 (not valid DAG-CBOR)")); }
```

**The decisive framing point:** `crates/benten-core/src/lib.rs:227-229` documents the CID as `[0x01, 0x71, 0x1e, 0x20, <digest>]`. Codec `0x71` *is* dag-cbor. Emitting tag 4 under it would make the multicodec assert something false about the bytes — the exact inversion of baked-in #5, whose permanent commitment is the *self-describing framing*, not any payload. A tag-4 claim would not be an additive codepoint; it would be a lie in the frame. **Confidence: high.**

### 2. Major-type occupancy — nothing is free

From `de.rs:308-344`, confirmed empirically:

| MT | claimed by | free? |
|---|---|---|
| 0/1 int | `Int(i64)`; out-of-range → typed error (`value.rs:219-229`) | no |
| 2 bytes | `Bytes` (`value.rs:95`, `serde_bytes`) | no |
| 3 text | `Text` | no |
| 4 array | `List` | no |
| 5 map | `Map` (text keys only) | no |
| **6 tag** | **spec-forbidden except 42; 42 unrepresentable in `Value`** | **no — permanently** |
| 7 simple | `false`/`true`/`null`/f32/f64 only | no |

MT7 spare space is also closed: `0xf0` (simple 16) → `Unsupported { byte: 240 }`; `0xf7` (undefined) → `Unsupported { byte: 247 }`; `0xf9` (f16) → `Unsupported { byte: 249 }` (dag-cbor mandates 64-bit floats). The IPLD Data Model has no `undefined`, so this is profile-correct, not an implementation gap.

**`Value` is precisely the IPLD Data Model minus Link.** That is a much stronger statement than "8 variants someone chose," and it is the honest reason there is no room: the type is already exactly as wide as its encoding permits.

### 3. The unrecorded gap is Link, not Decimal

`ValueVisitor` (`value.rs:185-292`) implements no `visit_newtype_struct`. serde's default (`serde_core-1.0.228/src/de/mod.rs:1671-1677`) is `Err(Error::invalid_type(Unexpected::NewtypeStruct, &self))`. So the one tag dag-cbor *does* allow is unrepresentable in `Value`.

Repo-wide grep for `Value::Link` / `Value::Decimal` across `docs/` and `crates/`: **zero hits.** Neither appears in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` nor `docs/V1-WIRE-INVENTORY.md`. The freeze would close both silently, with no ledger row — which is the FALSE-RECORD-adjacent failure mode rule 14 exists to catch (here: an *absent* record rather than a false one).

Minor inconsistency worth a row: the napi pre-scanner **accepts** tag 42 (`input_limits.rs:376-377`, pushes a frame) and then `decode_value_bounded` (`:460-465`) fails on it. Fail-closed, but misclassified — a link-bearing payload passes bounds-checking then dies as a decode error. **Severity: OBSERVATION.**

### 4. Decode-on-unknown-shape: typed error, fail-closed — no silent coercion

Every unrecognised shape produces `DecodeError::{TypeMismatch, Unsupported}`, wrapped by callers into `CoreError::Serialize`. No coercion, no panic, no default. The hand-written `Deserialize` (`value.rs:156-162`) exists to *prevent* the collapse `#[serde(untagged)]` would cause between text/bytes channels — it is already a deliberate strictness mechanism (`value.rs:62-77`). The depth guard (`value.rs:139`, `MAX_VALUE_DECODE_DEPTH = 64`) is likewise deliberately tighter than the codec's ~256 so the guarantee is version-pinned rather than inherited. **This surface is in good shape; nothing here needs fixing.**

### 5. The CID / store-and-re-serve question — architecturally reachable, no API for it

The tension resolves more favourably than expected:

- Storage **is** a byte KV keyed by CID: `redb_backend.rs:986` — `self.get(&node_key(cid))`.
- Integrity is **decode-independent and hash-first**: `redb_backend.rs:990-999` verifies the BLAKE3 of stored bytes *before* attempting decode, deliberately, so that "a tamper that happens to corrupt the CBOR structure would otherwise surface as a `Serialize` error, masking the real failure (integrity)". Same shape at `:2824-2836` and in `benten_core::Node::load_verified` (`lib.rs:340`).

So opaque store-and-re-serve is **within the architecture**: bytes are already both the storage unit and the integrity unit. What blocks it is purely the typed API — `get_node` returns `Result<Option<Node>, _>` and there is **no public raw-bytes read path** (grep for `get_node_bytes` / `get_raw`: zero hits). An engine that received a value shape it could not interpret would fail the read entirely rather than pass the bytes through.

**Consequence for the decision:** a future "typed-reject but preserve" arm is *not* foreclosed by the wire or by the integrity design. It is foreclosed only by API surface — which is exactly what the freeze locks. **Confidence: high on mechanism; medium on whether a byte-passthrough read path is desirable, which is outside my lens.**

### 6. The actual seam: `#[non_exhaustive]` is absent from `Value`

`value.rs:78-80`:
```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
```

`grep -c non_exhaustive crates/benten-core/src/value.rs` → **0**. Meanwhile the same crate applies it nine times, with explicit widening rationale: `change_stream.rs:85,123` ("sync may add `Replicated` or `Conflict` arms"), `encryption_class.rs:35,99` ("per V1-FROZEN-INTERFACE item 15(e): reserved"), `lib.rs:122,757` ("future phases add variants"), `subgraph.rs:68,195,259` ("guards against a future decision to add a 13th…").

`docs/public-api/benten-core.txt:721-728` pins all 8 variants in the frozen baseline.

**This is the one genuine last-moment item.** Adding `#[non_exhaustive]` is zero-wire-cost (it does not touch encoding, canonical bytes, or any CID), and it preserves the ability to add a variant post-tag without a SemVer-major break for every downstream `match`. Omitting it makes any future `Value` variant a governance event *twice over* — wire freeze **and** API break.

Honest costs: (a) it forces `_ =>` arms on all cross-crate matches — the project already tracks a "D-17 `non_exhaustive` cascade" item, so the cascade is known and non-trivial; (b) it slightly weakens the orchestrator's observation that `to_canonical` (`value.rs:373`) is an exhaustive match — though *not* in the defining crate, where exhaustiveness is preserved, so the compile-error-on-add property survives where it matters most.

### 7. The application-layer alternative, honestly costed

All measured on real `Node::to_canonical_bytes()`:

| encoding | bytes | notes |
|---|---|---|
| `Float(273.15)` | 39 | status quo |
| `Map{scale:-2, unscaled:27315}` | 50 | **+11 B**; round-trips; CID order-independent (verified) |
| `Bytes([0xfe,0,0,0x6a,0xb3])` | 36 | **−3 B**, *smaller* than the float |

The Map form is fully canonical and deterministic today — no engine change needed. The requester's counter is correct about *what* it costs, and it is not size:

- **Arithmetic**: `expr/eval.rs:540` — a `Map` in a numeric op yields `Err("numeric op type mismatch")`. Decimals become opaque to TRANSFORM entirely.
- **Ordering/comparison**: no path; `Map` has no numeric compare arm (`eval.rs:582-599`).
- **Indexing**: property-value indexes key on the encoded value, so range queries over money are unavailable.
- **Cross-language**: no TS mirror, no `ErrorCode`. The catalog carries `E_VALUE_FLOAT_NAN` / `E_VALUE_FLOAT_NONFINITE` (`packages/engine/src/errors.generated.ts:744,759`) and per-variant limits (`:671`) — a Map-encoded decimal inherits none of that.

That is an argument about first-classness, and it is a real one. It is *not* an argument that the wire cannot carry it.

### 8. The float hazard is real, but a different shape than described — and it is the engine's own

**Their "silent" claim is wrong here, in Benten's favour.** Probe: `0.1+0.2` and `0.3` produce **different CIDs** (`bafyr4i…` divergence; repr `0.30000000000000004441`). Content addressing makes float drift *loud* in the identity dimension — louder than a conventional database.

**But the engine has a worse, self-inflicted problem.** `crates/benten-eval/src/expr/eval.rs:544-561`:

```rust
(Value::Int(a), Value::Int(b)) => {
    if *b == 0 { return Err(EvalError::new("division by zero")); }
    // Integer division if evenly divisible; else promote to float.
    if a % b == 0 { Ok(Value::Int(a / b)) } else { Ok(Value::Float((*a as f64) / (*b as f64))) }
}
```

**The return type depends on the values, not the expression.** `total / 3` yields `Int` for 300 and `Float` for 1999. The universally-recommended mitigation — "store integer minor units" — is therefore defeated from *inside* the engine the first time anyone divides (tax split, per-unit cost, proration). Different property types → different canonical bytes → different CIDs, data-dependently.

This is an engine-on-its-own-terms finding and holds with zero adopters. **Severity: MAJOR. Confidence: high on the mechanism (quoted); medium on real-world blast radius**, which I did not survey.

**Two honest under-claims, against my own argument:**
- I did **not** reproduce float non-associativity on a realistic currency multiset — forward and reverse summation of `[0.1, 0.2, 0.3, 19.99, 0.07]` were bitwise identical. The property is real in general; I have no Benten-specific divergence case.
- **IVM does not aggregate floats at HEAD.** The five views are `capability_grants`, `version_current`, `event_handler_dispatch`, `content_listing`, `governance_inheritance` (`crates/benten-ivm/src/views/`) — none numeric. The incremental-vs-full-recompute divergence hazard I expected is **not live**. The float exposure is confined to application-authored TRANSFORM expressions.

---

## WHAT THIS MEANS FOR THE DEADLINE

Sorted by whether the tag actually forecloses them:

**Foreclosed by the tag — decide now or never:**
- `#[non_exhaustive]` on `Value`. Zero wire cost, pure API. This is the only item where "after the tag" genuinely means "not without a major version."

**Foreclosed by the tag, and currently unrecorded — needs at minimum a ledger row:**
- `Value` cannot represent an IPLD Link. Unrecorded anywhere. Whatever the disposition, the freeze should not close this silently.

**NOT foreclosed by the tag — the deadline pressure here is false:**
- `Value::Decimal` as a *wire* concern. There is no tag seam to preserve and no reserved space to claim, so waiting costs nothing that acting would save. The wire question is settled by the profile, permanently, in both directions.

**Not a freeze question at all — but a live defect:**
- `eval.rs:544-561` data-dependent `Int/Int` promotion. This is TRANSFORM semantics, not frozen wire. It should be fixed on its own merits and on its own clock.

The one framing I would push back on in the brief: this was posed as "claim major type 6 now or lose it forever." That choice does not exist. What does exist is a Rust-API seam that nobody has looked at, and a missing Link representation that nobody has recorded — both surfaced only because the tag question was asked. **Confidence: high on §1-§5, high on §6's fact and medium on its cascade cost, high on §8's quoted mechanism and explicitly low on its blast radius.**