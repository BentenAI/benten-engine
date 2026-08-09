Verification complete. I rebuilt the probe independently and opened every cited file.

## VERDICT SUMMARY

The empirical core (§1) is **CONFIRMED and reproducible**. The headline conclusion (§2 → "the deadline pressure here is false") is **REFUTED** — it rests on a non-sequitur that the analysis's own chosen analogy contradicts.

---

## CONFIRMED (my own evidence)

**§1 — DAG-CBOR forbids tags except 42.** All cites real: `serde_ipld_dagcbor-0.6.4/src/de.rs:211-220` (the `deserialize_cid` match), `de.rs:323-324` (`// The only supported tag is tag 42 (CID).` → `major::TAG => de.deserialize_cid(visitor)`), `lib.rs:148`, `Cargo.lock:6550`. I built a probe at `/private/tmp/claude-502/-Users-benwork-Documents-benten-engine/2117e319-bdd5-42f5-a762-a5cae0a6c8e4/scratchpad/verify-probe` linked against `benten-core` at `bf964354` and reproduced their table **exactly**, including the unusual tag-42 error string `Msg("invalid type: newtype struct, expected a DAG-CBOR value")`. Added: `tag 1000` → `TypeMismatch { byte: 232 }`. The encoder side is symmetric — `ser.rs:614` emits tag 42 and nothing else. **This claim is solid; the reframe does collapse.**

**§2 MT7** — confirmed, and I extended it: `0xf8` simple(32) → `Unsupported { byte: 248 }`, which they did not test.

**§3 Link gap** — confirmed. No `visit_newtype_struct` in `ValueVisitor` (`value.rs:185-292`); tag-42 fails at the `Value` layer. `Value::Link`/`Value::Decimal`: zero hits repo-wide.

**§4, §5** — confirmed. `redb_backend.rs:986`, `:990-999`, `:2821-2840`, `lib.rs:340` all say what is claimed. No raw-bytes read path exists.

**§6 fact** — `grep -c non_exhaustive crates/benten-core/src/value.rs` → **0**. Baseline pins 8 variants at `docs/public-api/benten-core.txt:721-728`. `to_canonical` (`value.rs:373`) is genuinely exhaustive, no wildcard.

**§7 sizes** — 39 / 50 / 36 bytes reproduced exactly; CID insertion-order-independence verified.

**§8 divide** — quoted **verbatim** at `eval.rs:544-561`, and reachable: `eval.rs:204` `BinaryOp::Div => divide(&l, &r)`. Float CID divergence reproduced, same repr `0.30000000000000004441`. IVM view list correct, none numeric.

---

## REFUTED

**R1 — "no reserved space to claim… waiting costs nothing." This is the load-bearing error.**

§2 argues "every major type is occupied → no seam exists." That is a non-sequitur. **Agility seams live *inside* occupied major types, not in free ones.** The analysis's own analogy proves it: crypto codepoints are reserved *values in an existing field*, not new wire shapes.

My probe — the map-key namespace is entirely unclaimed:

```
{"/": "a"}                  OK  Map({"/": Text("a")})
{"/decimal":[-2,27315]}     OK  Map({"/decimal": List([Int(-2), Int(27315)])})
{"\0dec":[-2,27315]}        OK  Map({"\0dec": List([...])})
{"$dec":[-2,27315]}         OK  Map({"$dec": List([...])})
```

A real `Node` with property key `/decimal` encodes (40 bytes), CIDs, and round-trips. No reserved-key policy exists in any doc. This is the **IPLD-native** mechanism (dag-json encodes links as `{"/": …}`), and this project **already uses a reserved-string-prefix for agility one layer up** — `engine:typed:` CALL targets, `docs/DSL-SPECIFICATION.md:150`.

And it is deadline-bound in exactly the direction §2 denies:
- **Pre-tag, reserving costs nothing** — no existing canonical bytes change; the sentinel CID pinned at `crates/benten-core/tests/node_cid.rs:16` is unaffected.
- **Post-tag, reserving is a breaking decode change** (bytes that decode today stop decoding).
- **Post-tag, *not* having reserved** means any future structurally-recognised `Value::Decimal` silently reinterprets pre-existing user data **under an unchanged CID** — a semantic fork on identical content-addressed bytes. In a content-addressed engine that is worse than a wire break, because the CID stops determining the interpretation.

Whether to take the seam is genuinely open (it constrains user key space, adds a decode-path check, and forces choosing a prefix before the need is known — `Map{unscaled,scale}` at app level may simply be right). But the *decision exists and expires at the tag*, and the analysis declares it nonexistent.

**R2 — "Cross-language: no TS mirror."** False. `packages/engine/src/types.ts:33-40` is a `Value` mirror whose doc comment reads "Mirrors `benten_core::Value`: null / bool / int / text / bytes / list / map." It is structural (`number` covers Int *and* Float), so a Decimal has no representation there — which strengthens their first-classness argument while breaking their cost estimate.

**R3 — `docs/V1-WIRE-INVENTORY.md` does not exist.** The real file is `docs/V1-WIRE-FORMAT-INVENTORY.md`. An absence-grep against a nonexistent path is vacuous — the exact `feedback_findings_carry_substance_not_bare_ids` failure mode. They also never opened `docs/V1-FROZEN-INTERFACE.md`, the primary freeze doc. **I re-ran it properly and the conclusion survives, stronger than claimed:** `benten_core::Value` appears in *none* of the three freeze docs. Right answer, unearned.

**R4 — §6 overstates what `#[non_exhaustive]` buys.** It is called "the one genuine last-moment item" preserving "the ability to add a variant post-tag." But by their own §2, every major type is occupied and untagged encoding cannot distinguish a new variant. `#[non_exhaustive]` alone yields a variant that **cannot be encoded**. The Rust seam is necessary but inert without the wire-side reserved namespace. The two are coupled; the analysis separates them and calls only one deadline-bound. **Both are, or neither is.**

---

## PARTIAL (substance right, evidence wrong)

- **`lib.rs:227-229`** for `[0x01,0x71,0x1e,0x20,…]` — **wrong line**. Actual: `lib.rs:272` and `lib.rs:597`. Lines 227-229 are the `# Errors` block of `to_canonical_bytes`. The framing argument survives; the cite would propagate wrong into a ledger row.
- **"applies it nine times"** — undercount. Actual `#[non_exhaustive]` sites in `benten-core/src`: **20, across 9 files** (`version.rs`, `lib.rs`, `version_dag.rs`, `subgraph.rs`, `version_chain.rs`, `encryption_class.rs`, `change_stream.rs`, `subgraph_spec/spec.rs`, `subgraph_spec/errors.rs`). Nine is the *file* count. Every individual line cited is real; the error favors their conclusion.
- **`eval.rs:582-599`** — `compare` is 579-594; 595+ is `values_equal`. Substance correct.
- **`input_limits.rs:367-378`** — the arm is at 375-378; the range contains it. Fine.

---

## WHAT IT MISSED

1. **The map-key seam** (R1) — the central omission.
2. **`bindings/napi/src/node.rs:319-354`** — `value_to_json` is a **fully exhaustive match, no wildcard**. A third compile-break site beyond `to_canonical`. **45 files** reference `Value::` variants; the cost estimate names one.
3. **A second public-api baseline block** at `docs/public-api/benten-core.txt:1062+` for the `benten_core::Value` re-export. Adding a variant touches two blocks, not one.
4. **`values_equal` (`eval.rs` ~596-602): `(Int, Float) => (*a as f64) == *b`** — cross-type equality coercion; `compare` coerces likewise at `:585,:588`, lossy above 2^53. **The evaluator is type-lenient on equality/ordering while content-addressing is type-strict.** Combined with the divide promotion, this is the actual mechanism by which "store integer minor units" fails *silently*: `total/3` compares equal to the expected Int while storing as Float under a different CID. This is a sharper statement of their own §8 and they missed it.
5. **Internal inconsistency.** Absence from the freeze ledger is a defect requiring a row (§3, Link) but a licence to defer (§8, "not frozen wire, fix on its own clock"). I verified TRANSFORM/expression semantics are indeed unmentioned in `V1-FROZEN-INTERFACE.md` — so §8's disposition rests on the same absence §3 calls a defect. Same evidence, opposite inference.

---

## ADVOCACY CHECK

Mostly analysing, and genuinely good at it — §8 volunteers two under-claims against its own thesis and reports the IVM non-finding. Credit that; it is rare.

The exception is the CONCLUSION block. "The reframe collapses" is correct and well-earned. "The deadline pressure here is false" is advocacy: it converts *one* seam being spec-forbidden into *no* seam existing, and the rhetorical confidence peaks precisely where the reasoning is weakest. **Net: keep §1 as settled fact; treat §2's "no seam" and the entire deadline disposition as unsupported and re-open the reserved-namespace question before the tag.**