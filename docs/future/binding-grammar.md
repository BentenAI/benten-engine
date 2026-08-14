# The binding grammar — completing E3

**Status: R0-INPUT.** This is a design record, not a plan. Per the ADDL observance rule, the
build gets its own R0→R1 pipeline when it is scheduled; this document is that pipeline's input,
the way the e2r scope-review fed F-full. Written 2026-08-10 out of the engine-fit-and-gaps
investigation (`docs/future/engine-fit-and-gaps.md` §3.1).

---

## 1. The finding: four fragments of one feature

The engine contains four partial implementations of handler-input binding, in four subsystems,
none aware of the others:

| Fragment | Where | State |
|---|---|---|
| Per-call specialization | `engine.rs::subgraph_for_crud` (`:4263`) — clones the handler template, patches the caller's property bag into WRITE's static `properties` | live, crud-only |
| **The sigil resolver** | `engine_stream.rs:1075` `resolve_stream_source` — `$input` → full-map projection of the caller's input; `$input.<field>` → first-level field lookup | **live, STREAM-only** |
| The binding vocabulary | `benten-eval/src/context.rs` — `EvalContext`, a scoped binding stack with `$input` `$result` `$item` `$index` `$results` `$error` and documented scope-locality rules | designed, built, unit-tested, `pub use`d, **zero production callers** |
| Suspend-time persistence | `exec_state.rs:272` `context_binding_snapshots: Vec<(String, Cid, Vec<u8>)>` — doc comment: *"CID-substitution attack mitigation"* | **frozen into the v1-beta wire envelope, always written empty** |

Fifth piece: `docs/DSL-SPECIFICATION.md:88,90` teaches the convention as the normal way to
author handlers — `.read({ by: "cid", value: "$input.cid" })`,
`.write({ properties: { post_cid: "$input.cid" } })`. Only STREAM would honor it today; the
general walk drops handler input on the floor (`evaluator.rs` `_input`, `call.rs:127`
`Node::empty()`).

Phase-1's plan named this deliverable **E3, "Core of Phase 1"**, with a must-pass
`tests/context_scoping`. The container shipped (G6-A), the walker shipped (G6-C), nobody joined
them, the test was never written, and E3 was marked closed.

## 2. The design: generalize, don't invent

Promote STREAM's grammar to the walk. One grammar, one resolver, resolved at property-read time
inside the evaluator.

- **Grammar at v1:** `$input` and `$input.<field>` — exactly what is shipped and documented.
  The rest of `context.rs`'s vocabulary (`$result`, `$item`, `$index`, `$results`, `$error`)
  is reserved and lands additively, sigil by sigil, as ITERATE/error-path integration is
  designed. Codepoint-band discipline applied to a namespace: reserve the band, assign the
  first entry, grow additively.
- **CALL passes its staged input** instead of `Node::empty()`; the callee's `$input` binds.
  This is the one joint where handler composition currently drops data.
- **WAIT snapshots live bindings** into `context_binding_snapshots` — the slot that already
  exists, frozen, for exactly this. Shape verified adequate: any `Value` canonicalizes to
  bytes, so `(name, cid-of-canonical-bytes, inline canonical bytes)` accommodates every
  binding, and the inline bytes preserve the slot's stated CID-substitution mitigation.
- **STREAM's `resolve_stream_source` is retired into the general resolver** at generalization
  time. Two resolvers for one grammar is the dual-home drift class (see the GCS-16
  `ENVELOPE_MAGIC` precedent) expressed in semantics rather than bytes. Mandatory, not
  optional.
- **crud is harvested later, optionally.** Once bindings are proven, `subgraph_for_crud`
  re-expresses as a stored template whose WRITE carries `properties: "$input"` — deleting the
  per-call clone-and-patch and shrinking the documented arch-10 registered-CID-vs-walked-CID
  divergence. Sequenced after the general mechanism is proven, canary-style: crud is the
  most-exercised path in the engine and does not need to move first.

### The boundedness carve — data binds, control stays static

Properties that determine **graph shape or budget** — CALL's `target`/`call_op`, ITERATE's
`max` — may NOT carry sigils, ever. Properties that determine **values** — `target_cid`,
`properties`, `source`, BRANCH's condition input — may. A dynamic BRANCH condition selects
among statically-declared forward edges; it cannot add one.

This line preserves, exactly:
- **Termination.** DAG structure + step budget + frame cap; none reads an operand (established
  in `engine-fit-and-gaps.md` §2.2).
- **Inv-8.** The registration-time multiplicative budget reads ITERATE `max` and resolves
  CALL's callee — both stay static, so the analysis is untouched, and the attack its own
  comment names (claim a cheap callee, point at an expensive one at runtime) stays
  unexpressible.
- **Install-time consent (baked-in #18).** The manifest envelope is granted against
  label/scope shapes, which stay static.

### The reservation — position-scoped, registration-time, fail-fast

At v1-beta, registration (`validate_subgraph`) rejects any `$`-prefixed `Value::Text` in an
**operation-node property**, with a typed error, EXCEPT the assigned positions (STREAM `source`
with `$input`/`$input.<field>`). Consequences:

- No stored v1-beta handler can carry an unassigned sigil, so every future assignment is a
  clean widening of a reserved-and-rejected space — never a reinterpretation of live data.
- The scope is operation nodes only. **User data nodes keep full freedom** — a data property
  whose text happens to start with `$` is untouched, because op properties are code and data
  properties are data. This is what keeps the reservation out of the decode-sniffing class
  rejected for `Value::Decimal`.
- Cost: one registration check + one ErrorCode mint with its full mirror cascade (the ENG-3
  precedent).

## 3. Security notes

- **Confused deputy:** a handler running as a plugin principal, reading a caller-supplied CID
  via `$input.cid`, reads with the plugin's authority. This is not new — crud already threads
  caller data into the walk, and any handler already reads caller-influenced backend state.
  The containment is the one designed for it: the manifest envelope bounds what the plugin's
  principal can touch regardless of where the CID string came from. No new mechanism needed;
  the design must say so rather than discover it.
- **Read gating is unchanged:** `check_read` fires at resolution time against the active
  principal, whatever the source of the CID.

## 4. Considered and declined

- **Edge-based bindings** (a `BINDS` edge from an op node to a parameter node) — the
  graph-purer shape, declined: it would introduce a second convention fighting three shipped
  fragments (STREAM's resolver, the DSL examples, `context.rs`) rather than completing them.
- **Shipping the full vocabulary at once** — declined; `$result`/`$item`/`$index` need
  ITERATE and error-path design that `$input` does not. Reserve everything, assign `$input`.
- **Walk-time-only reservation** (unknown sigils route ON_NOT_FOUND) — declined; silent
  semi-brokenness. Registration-time rejection is fail-fast and makes the reserved space
  provably empty.

## 5. Pre-tag items (the now-or-never subset)

1. **Freeze-record disclosure**: name the STREAM sigil grammar (an interpreted mini-language in
   a frozen wire position, currently named nowhere), the `$` reservation and its position
   scoping, and `context_binding_snapshots`' intended semantics (reserved for binding
   persistence; empty at v1-beta).
2. **The registration-time reservation itself** + ErrorCode mint + mirrors.
3. **The `InfiniteEmptyProducer` wart** (`engine_stream.rs:974`): unresolved-`$input` with
   empty input yields an INFINITE empty-chunk producer because it "drives the for-await break
   test" — a production semantic shaped by test convenience, about to freeze de facto. Decide
   deliberately: the principled semantic is uniform EmptyProducer (immediate EOS) or a typed
   error, with the test constructing an explicit infinite source instead.
4. **The stale resolver comment** (`engine_stream.rs:1078`): says "first property"; the code
   builds the full map. The code is right; fix the comment (rule 15 micro-case).

Everything else — the general resolver, CALL threading, WAIT snapshots, STREAM unification,
crud harvest, further sigils — is post-tag additive, with this document as the R0-input and
**`docs/future/phase-4-backlog.md` §4.170** as the receiving row (it exists and carries the
ordered build list — not a forward reference).

## 6. What this closes

- `engine-fit-and-gaps.md` §3.1 (relative addressing / composition does not compose) — this is
  the design.
- The `DSL-SPECIFICATION.md:60-67` FALSE-RECORD — resolved per rule 15 by *fixing the code*
  (eventually) and marking the rows honestly now ("grammar reserved; STREAM-only at v1-beta")
  rather than deleting the design intent the spec correctly recorded.
- Phase-1 E3, reopened and finished properly instead of marked closed unbuilt.

## 7. Ben's proposal — dataflow AS nodes and edges (2026-08-13, OPEN)

*"do we want to represent executions of workflows/handlers/subgraphs as nodes themselves with edges
that get passed/added from step to step in that spirit of 'dataflow' is edges and nodes?"*

**Recorded as a genuine fork against §2, not as an adoption.** §2's mechanism is `EvalContext` — a
scoped binding stack, values living in a side-channel the evaluator threads. Ben's is
**materialisation**: the execution becomes graph structure, and a later step reads an earlier
step's output by *following an edge*.

**They are not rivals; they are different layers, and that is the interesting part.** Binding gives
an operand a NAME (`$result`); materialisation gives that name an ADDRESS. Compose them and
`$result` resolves to *"traverse the RESULT edge from my execution node"* — at which point:

- **dataflow** = follow an edge
- **relative addressing** (§3.1 of the fit-gaps ledger) = follow an edge from your anchor
- **aggregation** = fold the nodes an edge-set reaches

**All three of the Ben-set trio become one mechanism.** That is the strongest argument for the
proposal and it is an argument §2 alone cannot make.

**Second argument, and two adopters asked for it independently:** provenance falls out for free. A
materialised execution *is* the audit trail the museum's statutory obligations need, and the
substrate the LLM runtime's verifiable-compute question was reaching for. Neither needs a separate
feature.

**Third: it is already half-present.** `StepResult` carries `output: Value`, and the evaluator
already copies it into a trace record (`outputs: r.output.clone()`). Today that trace is ephemeral
and unaddressable. The proposal is largely *promote the trace into the graph* — generalize, don't
invent.

**The honest objections, in the order they need answering:**

1. **Cost.** A node mint per step is a hash plus a store write. On the museum's ~6 ms till line and
   anything per-token, that may not be affordable. Likely resolution: materialisation is a MODE
   (ephemeral by default, persisted when you want the trail), not a universal law — but that must
   be measured, not assumed.
2. **Do not mutate the handler.** "Edges added from step to step" must mint a SEPARATE execution
   graph that *references* the handler subgraph. Adding edges to the handler would change its CID
   mid-walk and collide with the immutability invariant. The distinction is code-vs-instance and it
   has to be explicit in the design.
3. **Where do executions live?** Zone, capability (who may read an execution?), retention, and
   whether they replicate over sync. All unanswered, all consequential.
4. **It does not remove the need for §2.** Something still has to resolve a name to an address at
   step time; `EvalContext` is a built, unit-tested candidate for exactly that.

**⚠️ THE FACT THAT CHANGES THE PRIORITY, whichever option wins.** VERIFIED at `875c3deb`:
`EvalContext` is **fully built** — `with_input(Value)`, `push_scope`/`pop_scope`, `get`/`set`,
`depth`, clock and suspension-store integration, and the binding names `$input` `$result` `$item`
`$error` — and it is **production-reachable only via WAIT**. The evaluator's step loop never
consults it. And `context_binding_snapshots` is **already in the frozen v1-beta wire envelope,
always written empty**, so the freeze anticipated suspend-time binding persistence and it was never
populated.

So the gap that gates everything has its mechanism **built, unit-tested, public, and unwired** —
the same shape as `get_by_property`, `read_view`, `put_edge`, the vault and `register_peer_did`.
**Closing the dataflow gap may be a wiring job rather than a build**, and that is measurable in an
afternoon.

**Recommended next step — do NOT decide this in conversation.** Two cheap moves that inform it:
**(a)** wire `EvalContext` into the step loop and measure what it actually closes (it either lights
up several of the six §5d observations at once, or it does not, and either answer is decisive);
**(b)** run the owed check on whether READ's accepted-property set is frozen CLOSED — because this
proposal *needs* edge-following as an addressing mode, and that is the one part of it the tag could
foreclose. This is ADDL-scale design; it wants the full pipeline, with §2 and §7 as the two
candidate shapes.
