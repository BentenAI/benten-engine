Verification complete. Base `bf964354` confirmed, tree clean.

---

# ADVERSARIAL VERDICT

**Headline: the recommendation survives; two of its supporting arguments do not; and it missed a larger freeze-relevant defect inside the exact surface it was asked to examine.**

I built a throwaway probe crate against the pinned `serde_ipld_dagcbor 0.6.4` and **executed** the decode paths the analysis said it could not. Probe at `/private/tmp/claude-502/-Users-benwork-Documents-benten-engine/2117e319-bdd5-42f5-a762-a5cae0a6c8e4/scratchpad/tagprobe/src/main.rs`.

## Verdict per claim

| # | Claim | Verdict |
|---|---|---|
| 1 | DAG-CBOR forbids tags other than 42; the reframe collapses | **CONFIRMED — and now empirically proven** |
| 2 | Codec cites (`de.rs:213-219`, `:308`, `:324`; `lib.rs:148`; `ser.rs:613`) | **CONFIRMED** (`:613` is the comment, `:614` the code — trivial) |
| 3 | Our visitor has no `visit_newtype_struct`; even tag 42 would fail | **CONFIRMED** — and load-bearing *against* the analysis (see MISS-1) |
| 4 | No discriminated space: decimal-as-array ≡ `List`, decimal-as-map ≡ `Map` | **CONFIRMED empirically** |
| 5 | `i128` unencodable ⇒ structural blocker on Option B | **PARTIAL — overclaimed** |
| 6 | Scale-significance irreconcilable with canonical bytes | **CONFIRMED** |
| 7 | `divide` silently promotes `Int/Int` → `Float` | **CONFIRMED, and reachability traced** |
| 8 | No arithmetic docs; `docs/ENGINE-SPEC.md` does not exist | **CONFIRMED** |
| 9 | A′ loss table (indexes / eval / DSL / TS) | **CONFIRMED** — all four cites exact |
| 10 | Option D precedent: `Scalar::BytesCid` / `TimestampHlc` | **CONFIRMED — stronger than claimed** |
| 11 | `Value` carries no `#[non_exhaustive]`; absent from §11 | **CONFIRMED — worse than claimed** |
| 12 | No test pins tag-rejection or cardinality | **CONFIRMED** |
| 13 | A `Value` carve-out is "substantively correct" | **REFUTED** |
| 14 | Engages the deadline | **PARTIAL** |

### Evidence on the load-bearing ones

**(1)(4) Empirical probe output:**
```
tag4 decimal-fraction [-2,27315]   => REJECTED: TypeMismatch { name: "CBOR tag", byte: 4 }
tag5 bigfloat [-1,3]               => REJECTED: TypeMismatch { name: "CBOR tag", byte: 5 }
tag2 bignum h'0102'                => REJECTED: TypeMismatch { name: "CBOR tag", byte: 2 }
tag42 CID (legal DAG-CBOR link)    => REJECTED: Msg("invalid type: newtype struct, ...")
plain array [-2,27315] (control)   => ACCEPTED as List([Int(-2), Int(27315)])
List([-2,27315]) encodes to        => 8221196ab3
```
The array control encodes to the exact bytes a tuple-variant `Decimal(-2, 27315)` would. **Options B-via-tag and C-via-tag are dead, not merely inadvisable.**

**(5) Downgrade.** `ser.rs:286-295` is verbatim as cited and my probe confirms `i128 > u64::MAX` → `ERR`. But this blocks `unscaled: i128`, **not `Value::Decimal`**. An `i64` unscaled encodes fine. Listing it as one of "three independent structural facts" blocking B inflates a shape constraint into a structural one. B is blocked by (4) alone.

**(7) Reachability the analysis did not trace** — `primitives/mod.rs:78` `PrimitiveKind::Transform => transform::execute` → `transform.rs:62-87` (reads the `expr` property, parses) → `eval.rs:204` `BinaryOp::Div => divide(&l, &r)` → `eval.rs:544-561`. **Live on the production TRANSFORM path.** The finding is real.

**(11) Worse than claimed.** `Value` appears **exactly once** in all of `docs/V1-FROZEN-INTERFACE.md` — line 741, an unrelated table header `| Surface | Constant | Value | Status at v1-beta |`. The property type on every Node and Edge is not named anywhere in the freeze document. `g_core_9_non_exhaustive_audit.rs` never mentions it either.

---

## MISS-1 — `Value`'s space is closed but **not fully claimed**. This refutes claim 13.

The analysis's principled argument for "no seam needed" is: *"`Value`'s variant set is pinned to the CBOR major-type set, an IANA-governed space that is closed… `Value` does not need an agility seam because its underlying space does not move."*

**The space doesn't move. It is also not fully occupied.** Exactly one legal DAG-CBOR shape is unclaimed by `Value`: **tag 42, the IPLD link.**

- Empirically, a well-formed tag-42 payload is rejected — **by our visitor, not the codec.** The codec supports it (`de.rs:214`); `ValueVisitor` has no `visit_newtype_struct` (verified absent), so it dies at `invalid type: newtype struct`.
- `benten_core::Cid` (`lib.rs:417-423`) serializes as a plain byte string via `serde_bytes_fixed`. **Benten emits no IPLD links at all.**
- `vocab.rs:174-176` — `Scalar::BytesCid`: *"Maps to `Value::Bytes` with a documented CID interpretation."*

The analysis cited that last line as proof the project *decided against* a CID variant. It is equally proof that a spec-legal shape is **already being worked around, in exactly the manner an application would work around a missing decimal.** `Value::Link(Cid)` is a strictly more plausible 9th variant than `Decimal` — and unlike `Decimal`, it has free encoding space, which is precisely the crypto-codepoint pattern baked-in #5 describes.

So recommended action #2 ("document `Value`'s frozen-cardinality carve-out") would **write a false rationale into the freeze record** — the same defect class as CLAUDE.md rule 14's FALSE-RECORD. The honest action is to put APPLY-vs-CARVE-OUT to Ben with the tag-42 slot named.

**And the cost the analysis never priced:** applying `#[non_exhaustive]` to `Value` is not free. 102 `Value::Map(` sites outside `benten-core`; 9 files carry apparently-exhaustive matches — `benten-platform-foundation/src/materializer.rs`, `benten-eval/src/expr/eval.rs`, `primitives/{wait,write,read}.rs`, `typed_call.rs`, `benten-engine/src/{engine_stream,primitive_host}.rs`, `bindings/napi/src/node.rs`. Small, but real, and **strictly now-or-never**: post-tag it is a SemVer break.

---

## MISS-2 — The `Value` decode boundary is not strict-canonical. Verified CID aliasing.

Nothing in the analysis touches this, and it is larger than the decimal question.

```
f32 1.5 (0xfa)                     => ACCEPTED as Float(1.5)
f32 input fa3fc00000 re-encodes to => fb3ff8000000000000 (idempotent: false)
int 23 NON-canonical (0x1817)      => ACCEPTED as Int(23)
map with DUPLICATE keys            => ACCEPTED as Map({"a": Int(2)})   # silent last-wins
map keys OUT OF canonical order    => ACCEPTED as Map({"a":2,"bb":1})  # silently reordered
```

DAG-CBOR requires 64-bit floats, shortest-form integers, no duplicate keys, and canonical key order. All four are accepted and normalized.

**Why it matters here.** `Node::load_verified` (`lib.rs:340-361`) hashes the **received bytes** and compares to the supplied CID:

```rust
let digest = blake3::hash(bytes);
let recomputed = Cid::from_blake3_digest(*digest.as_bytes());
if &recomputed != cid { return Err(CoreError::ContentHashMismatch { .. }); }
```

A peer supplying a non-canonically-encoded Node computes the CID over *its own* bytes, so verification **succeeds**. The write path then re-canonicalizes via `to_canonical_bytes()` (`transaction.rs:386`, `redb_backend.rs:1884`). Net: **two distinct CIDs, both verifying, decoding to `==`-equal `Value`s.**

The doc-comment at `lib.rs:345-346` asserts *"by the canonical-DAG-CBOR contract, a Node's CID is a pure function of its encoded bytes."* True, and insufficient — injectivity needs the converse, and the decoder breaks it.

**The project already closed this class on a newer surface and not on its oldest one:** `benten-id/src/errors.rs:271` `KeysetDocNonCanonical` + `crates/benten-id/tests/kdb_f_inj_keyset_strict_canonical.rs` (a strict-canonical decode **reject matrix**). Meanwhile `docs/V1-WIRE-FORMAT-INVENTORY.md §1` (Node/Edge canonical bytes) is marked **"✅ COVERED"** on round-trip and sentinel-CID pins only — no non-canonical reject arm exists.

This is deadline-coupled in the hard direction: tightening decode is **narrowing**, and per the project's own D-101 framing, additive has a narrow valve and narrowing does not. Cheapest correct closure is a re-encode-and-compare in `load_verified` (one extra encode per verified load), which closes all four at once.

*Confidence: HIGH on decode acceptance (executed) and on `load_verified` hashing received bytes (read). MEDIUM on downstream impact — I did not trace a concrete exploit or prove no later gate catches it.*

---

## MISS-3 and MISS-4 (minor)

**MISS-3.** The TS mirror already destroys `Value`'s Int/Float distinction. `bindings/napi/src/node.rs:136-161` prefers `Int` whenever a number is integer-representable, so JS **cannot produce `Value::Float(1.0)`**; `types.ts:33-40` maps both to `number`. The analysis used the TS boundary to argue a Decimal variant would be unrepresentable there too — correct, but it missed that the existing variant identity is already lossy across that mirror, which is the sharper version of its own point.

**MISS-4 (method).** The analysis wrote: *"I did not execute a tag-4 payload against the decoder (read-only worktree)."* A read-only worktree does not prevent a throwaway probe against the same pinned crate — that is literally the project's M-20 golden-hex-via-throwaway-compute discipline. Its single most load-bearing claim went unfalsified when falsification cost about three minutes. Under `feedback_falsification_sweep_before_every_phase_close`, "the source path is unambiguous" is the reasoning that sweep exists to distrust.

---

## Advocacy / deadline

**Advocacy: mostly clean.** It steelmans C, tables A′'s losses honestly, and says "do nothing" is acceptable without inflating it. The one advocacy failure is §5's *"my read is that a carve-out is substantively correct"* — the single judgment that is wrong, arrived at without checking whether the space was fully claimed.

**Deadline: PARTIAL.** It hedges the one behavioural fix to "now-or-soon", and I found `benten-eval` expression semantics appear **nowhere** in `V1-FROZEN-INTERFACE.md` — so "freeze-coupled" is the analyst's judgment presented as near-fact. Meanwhile the two items that are strictly now-or-never — `#[non_exhaustive]` on `Value` (SemVer break after) and strict-canonical decode (narrowing after) — are respectively dismissed and absent.

---

## What survives

**The core recommendation stands, and is now stronger than argued: do not add `Value::Decimal`.** Not because of preference or cost, but because the encoding space does not exist — empirically demonstrated, not inferred. Option D is real (`Scalar` carries `#[non_exhaustive]` with an explicit "future scalar kind lands additively" doc-block, so it is genuinely post-freeze-additive). The `divide` finding is real and on the live path. The message to the external team is honest and correct.

**Revised priority, replacing the analysis's table:**

1. **Strict-canonical decode at the `Value`/Node boundary** (MISS-2) — narrowing, so hardest to fix post-tag; a live CID-aliasing gap; the project has the precedent and the typed-error pattern already.
2. **`Value` `#[non_exhaustive]`: APPLY vs CARVE-OUT, decided on the merits with tag-42 named** (MISS-1) — SemVer-irreversible after the tag; ~9 files.
3. **`Int`/`Int` division semantics** (`eval.rs:544-561`) — I agree with the prediction of a typed error, and agree it is Ben's fork.
4. Tag-rejection + cardinality pins; app-layer decimal recipe to a named destination; `wrapping_add` and the dual-equality split.

**Do not ship recommended action #2 as written** — "frozen-cardinality carve-out" is a rationale the code contradicts.