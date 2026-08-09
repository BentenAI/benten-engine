I have what I need. One material disclosure first, then the brief.

---

# DECISION BRIEF — `Value::Decimal` before the freeze

> **Disclosure on provenance.** I was told 4 lenses with adversarial verifications were attached. **They were not in my context** — nothing was pasted, and no lens JSONs exist on disk. Rather than synthesize from summaries I don't have, I recovered the two probe crates the lens agents left in the scratchpad, **re-ran them**, and re-verified every claim directly against source at `bf964354`. Everything below is first-hand evidence with file:line or a reproducible probe. Where a lens may have found something I couldn't reach independently, I say so in §8. Treat this as a verified-from-primary-sources brief, not a synthesis of four opinions.

---

## 1. THE ANSWER IN THREE SENTENCES

**Do not add `Value::Decimal`** — not because decimals don't matter, but because `#[serde(untagged)]` makes the variant *structurally inexpressible*: it would encode as an array or map and decode back as `List` or `Map`, so it cannot round-trip without either a wire break or a silent reinterpretation of existing data. The tag-based reframe that motivated this pass is **empirically dead** — DAG-CBOR rejects every tag except 42, so there is no unclaimed major-type-6 space to reserve. **The genuinely now-or-never decision is a different one**: `Value` has no agility seam at all, and the cheapest seam available — reserving a map-key namespace — is a *narrowing* change that is free today and impossible after the tag.

---

## 2. THE DECIDING FACT

**DAG-CBOR rejects CBOR tags. All of them except tag 42.**

Empirically, against `serde_ipld_dagcbor` 0.6.4 — the exact codec the workspace uses — with a visitor shaped identically to `Value`'s:

```
tag4 decimal-fraction [-2,27315]   => REJECTED: TypeMismatch { name: "CBOR tag", byte: 4 }
tag5 bigfloat [-1,3]               => REJECTED: TypeMismatch { name: "CBOR tag", byte: 5 }
tag2 bignum h'0102'                => REJECTED: TypeMismatch { name: "CBOR tag", byte: 2 }
plain array [-2,27315] (control)   => ACCEPTED as List([Int(-2), Int(27315)])
```

*(reproducible: `scratchpad/tagprobe`, sources preserved)*

**What this rules OUT:** RFC 8949 §3.4 tag 4 / tag 5 as an encoding for decimals. The premise "major type 6 is unclaimed and this is the last chance to claim it" is **false**. Major type 6 is not unclaimed — it is *claimed by the IPLD profile and closed*, with tag 42 (CIDs) as the sole permitted inhabitant. There is nothing to reserve, and no door closing on it. If you take one thing from this brief: **the reframe collapses, cleanly and completely.**

**What this rules IN — and this is the part that actually decides the question:** any `Value::Decimal` must serialize using an *already-permitted* major type — an array, a map, or a byte string. But `Value` is `#[serde(untagged)]` (`crates/benten-core/src/value.rs:79`) with **only `Serialize` derived** (`:78`), and its `Deserialize` is hand-written and dispatches purely on data-model type:

- `visit_seq` → always `Value::List`
- `visit_map` → always `Value::Map`
- `visit_bytes` → always `Value::Bytes`

So a `Decimal` encoded as `[-2, 27315]` **decodes back as `List([Int(-2), Int(27315)])`**. The variant cannot survive a round-trip. There is no discriminant on the wire to recover it from — that is the whole meaning of "untagged."

That leaves exactly two ways to make it work, and both are bad:

- **(a) Add a discriminant** → changes the encoding of *every existing variant* → a total wire break, at the freeze.
- **(b) Sniff for a reserved shape on decode** (e.g. a map with a `"/decimal"` key) → **silently reinterprets data that is legal today.** The probe confirms this is live exposure: `{"/decimal":[-2,27315]}`, `{"$dec":...}`, `{"\u0000dec":...}` and even `{"/": "a"}` are all **accepted right now as ordinary `Value::Map`** and reach a real `Node` property with a stable CID (`scratchpad/verify-probe`). Anyone with such a key today would have their data silently re-typed by a future sniffing decoder.

**The decimal question is therefore not a cost question. It is a possibility question, and the answer is no.**

---

## 3. THE OPTIONS, RANKED

### Option 1 — Do nothing on decimals; reserve a map-key namespace as `Value`'s agility seam ★ RECOMMENDED
Declare `"/"`-prefixed property keys **reserved** at the freeze, with a **typed-reject** on decode for any reserved key the engine doesn't recognize.

- **Cost:** one check in `visit_map`; one `ErrorCode` mint + TS mirror; a doc row. Small.
- **Forecloses:** users' ability to use `/`-prefixed property keys (sampled as unused for *property keys* — the grep hits are URL paths and capability scopes, not `Value::Map` keys — **moderate confidence**, worth one confirming sweep).
- **Freeze-coupling: MAXIMAL — this is the real now-or-never.** Reserving a namespace is a **narrowing** change (data legal today becomes illegal). The project's own freeze semantics (D-101) are explicit that *additive has a narrow valve, narrowing does not*. Free today at zero adopters; unavailable forever after the tag.
- **Why it's the right shape:** it is baked-in #5's pattern applied to `Value` — a reserved band plus a typed-reject arm for the unknown, never a silent fallback. It is the only seam that fits in the untagged encoding without breaking it.

### Option 2 — Do nothing at all
- **Cost:** zero now.
- **Forecloses:** every future extension of `Value`. Any new shape becomes a whole-wire-format version bump — a governance event, exactly as the brief anticipated.
- **Freeze-coupling:** total and permanent. Defensible, but choose it *knowingly*.

### Option 3 — Add `Value::Decimal` with a discriminant (full re-encoding of `Value`)
- **Cost:** breaks every golden, every CID, every stored node, the napi boundary (**10 files** under `bindings/napi/src/` touch `Value::`), the TS mirror, and the freeze record. Enormous, days before a tag.
- **Forecloses:** the current `Serialize`-derive simplicity permanently.
- **Freeze-coupling:** must be now or never — but the cost/benefit is indefensible against Option 1.

### Option 4 — Add `Value::Decimal` with decode-sniffing
- **Rejected on correctness, not cost.** Silently reinterprets currently-legal data (probe-confirmed). This is the "silent fallback" baked-in #5 exists to forbid.

---

## 4. THE RECOMMENDATION

**Option 1.** Tell the museum team no on `Value::Decimal`, with the real reason (the encoding cannot express it). Separately, and on the engine's own terms, **spend the small budget to give `Value` the reserved-namespace seam before the tag.**

**The strongest counter-argument, stated fairly:** *"You're refusing the concrete, well-motivated request and instead spending the freeze window on a speculative seam nobody has asked for. IEEE-754 for currency is a real, silent, canonical bug — and the standing bias says push to the application layer, which also says don't add the seam."*

**The answer:** The refusal isn't a judgement call about priority — the variant is *inexpressible* in the untagged encoding, so no amount of wanting it makes it available. That part isn't a tradeoff at all. The seam is a different matter: it is not speculative in the way it looks, because it is the *only* change in this whole analysis whose availability is genuinely destroyed by the tag. Everything else — decimals included — remains as possible (or as impossible) the day after the tag as the day before. And the application-layer bias argues *for* Option 1, not against it: the seam is what makes future application-layer conventions safely distinguishable from user data, instead of colliding with it.

**Honest caveat:** if you judge that `Value` should simply never grow again, Option 2 is coherent and I won't argue it's wrong — but take it as a decision, not a default.

---

## 5. IF WE DO IT: the exact change

**For Option 1 (recommended):**

- **`crates/benten-core/src/value.rs`**, in `visit_map` (the `while let Some(k) = map.next_key::<String>()?` loop): reject any key where `k.starts_with('/')` and the key is not a recognized engine key, via `serde::de::Error::custom` — the same mechanism `depth_exceeded_msg()` already uses (`value.rs`, depth guard). No change to `Serialize`, no change to `to_canonical`, **no canonical-bytes change** — decode-side narrowing only.
- **`crates/benten-errors`**: mint one `ErrorCode` (e.g. `E_VALUE_RESERVED_KEY`), mirrored through enum / `as_str` / `from_str` / stable-shape / catalog per the standing §3.5g discipline.
- **`packages/engine/src/errors.generated.ts`**: regenerate.
- **`docs/V1-FROZEN-INTERFACE.md`** + **`docs/MODULE-MANIFEST.md` §2**: record the reserved band.

**If Ben overrides and wants `Decimal` anyway — the byte-level decision you asked for, and the trap in it:**

The encoding would be `{unscaled: i128, scale: i32}`. But note `i128` **cannot** be encoded: the probe shows `serde_ipld_dagcbor` rejects anything outside `[-u64::MAX-1, u64::MAX]` (`Integer must be within [-u64::MAX-1, u64::MAX] range`), so the mantissa is capped at 64-bit or must become `Bytes`.

**Scale-preservation vs canonical determinism — and this is the finding that should settle it even for someone who wants decimals:** `1.500` = `{1500, 3}` and `1.5` = `{15, 1}` are *different canonical bytes*, so they get **different CIDs**. Scale is preserved — but numeric equality no longer implies CID equality. In a content-addressed engine that means dedup, IVM view keys, and CRDT merge all treat two numerically-identical money amounts as **different values**. Normalizing to fix that destroys scale preservation, which was the entire requirement. **This tension is unavoidable, and it is identical whether the decimal lives in the engine or in the application layer** — so a `Value::Decimal` variant buys the requester *nothing* on the axis they care about most. That is worth telling them plainly.

**What an engine meeting an unknown value shape must do:** typed-reject, never silent fallback (baked-in #5). Today it already does, for tags — `TypeMismatch { name: "CBOR tag" }`.

---

## 6. IF WE DON'T: what we tell them

> DAG-CBOR — the IPLD profile we're frozen on, not plain CBOR — permits no tags except tag 42 for links, so RFC 8949 tag 4 decimal fractions cannot be encoded. Our `Value` encoding is untagged and keyed on CBOR major type, so a `Decimal` variant would decode back as a `List` and cannot round-trip. This is a structural property of the format, not a scheduling decision, and it would be equally true after the freeze.

**The application-layer answer in Benten terms:** a decimal is `Value::Map{"unscaled": Int, "scale": Int}` — probe-confirmed to encode cleanly (`a2657363616c652168756e7363616c6564196ab3`) and round-trip exactly. Three simultaneous scales are fine; scale is per-value, so 3dp tax lines, 2dp order headers and 3dp unit costs coexist without a global minor unit.

**What it honestly costs them — their counter is correct and I'm not going to soften it:** it is *invisible to every other consumer of the graph*. IVM views can't order it numerically, the DSL sees a map, cross-language mirrors see a map, and nothing stops a peer writing `{"unscaled": "12.5"}`. That's a real first-classness loss, and it is the strongest part of their case.

**What it does NOT cost them, contrary to the natural assumption:** exact arithmetic and scale preservation are fully available — `i64` unscaled covers ±9.2×10¹⁸ minor units, far beyond museum revenue. And per §5, they'd face the identical CID-splitting behaviour with a native variant. **The gap between what they asked for and what they can have today is smaller than it appears.**

---

## 7. WHAT THIS SAYS ABOUT `Value` GENERALLY

**Yes — this is a real defect, and it is about to become permanent, entirely independent of decimals.**

`Value` is the one frozen structural type with **no agility seam whatsoever**. Every other frozen surface in this project has one by deliberate design: crypto has codepoint bands with typed-reject arms; the wire inventory has version fields. `Value` has `#[serde(untagged)]`, which is the precise opposite of a codepoint — it is a format that *by construction* cannot carry a discriminant. This isn't an oversight anyone can patch late; it's inherent, and §2 shows it's what makes the decimal request impossible rather than merely expensive.

**Three further findings from the probes, same family, offered with deliberate under-claiming — each deserves its own verification pass, and none should be treated as settled on my say-so:**

1. **f32 widens silently.** `visit_f32` returns `Ok(Value::Float(f64::from(v)))` (`value.rs`). Wire bytes `fa3fc00000` decode to `Float(1.5)` and **re-encode to `fb3ff8000000000000`** — probe-confirmed non-idempotent. Two distinct byte strings map to one `Value`. Benten never *produces* f32, so this is only reachable from a foreign or hostile producer, and the CID-mismatch checks at the plugin/sync boundaries (`plugin_lifecycle.rs:976`, `mst.rs`) would catch the common case — but decode-side permissiveness that breaks byte↔value bijection is exactly the malleability property a content-addressed system should not have. **Moderate confidence it matters; high confidence in the mechanism.**
2. **Duplicate map keys are accepted, last-wins.** `visit_map` does a plain `out.insert(k, value)` with no duplicate detection; `{"a":1,"a":2}` → `{"a": 2}`. DAG-CBOR forbids duplicate keys.
3. **Non-canonical integer encodings are accepted.** `0x1817` → `Int(23)`, re-encoding to `0x17`. Note `docs/MODULE-MANIFEST.md:44-48` claims the strict subset including "Smallest-int encoding" — though that claim is scoped to the canonical form we *produce*, so I would **not** call this a false record without a closer read.

All three are one shape: **decode is more permissive than encode, so decode∘encode is not the identity on bytes.** For a freeze whose entire premise is that bytes determine identity, that is worth a deliberate look — and it is a strictly better use of the remaining window than decimals.

---

## 8. OPEN QUESTIONS

1. **The four lens reports.** They never reached me (see disclosure). If they exist, they may contain findings I couldn't reach independently — worth locating before you act, though I'd note the decisive facts here are empirical and reproducible, so I don't expect them to *reverse* §2.
2. **Is `"/"` genuinely free as a property-key prefix?** I sampled and found only URL paths and capability scopes, not `Value::Map` keys. One targeted sweep would confirm before reserving. **This is the only thing gating Option 1.**
3. **Are the §7 decode-permissiveness findings reachable from a trusted path?** Determines whether they're hardening or a genuine pre-tag defect. Not answerable without tracing every `Value` decode boundary — out of scope here, and I'd rather flag it than guess.