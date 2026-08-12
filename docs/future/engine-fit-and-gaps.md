# Engine fit and gaps — what real applications hit

**What this is.** A running record of applications we are considering building on Benten (or
that others are evaluating), the engine limitations each one hits, and what we decided to do
about them. Started 2026-08-10, when two unrelated outside evaluations arrived within days of
each other and hit overlapping gaps.

**Why it exists.** Until now every constraint on the engine came from inside — our own review
councils, our own invariants. These are the first readings by people who needed the engine to
do a specific job and read the source to find out whether it could. That is a different and
more valuable kind of signal, and it was arriving faster than any single decision-log entry
could hold.

> ## ⚠️ THIS DOCUMENT IS NOT A DESTINATION
>
> Recording a gap here **does not disposition it.** Under HARD-RULE-12 a deferral needs a named
> destination that has *actually received* the entry — and this project has ~93 findings
> currently reduced to tally marks because that rule was not held. A finding here is either
> (a) fixed, with the commit named, (b) carried to a real row in
> `docs/V1-FROZEN-INTERFACE-DEFERRED.md` or `docs/future/phase-4-backlog.md`, or (c) explicitly
> disagreed with and why. **If a row below has none of those three, it is not deferred — it is
> undispositioned, and that is a defect.**

---

## 1. The applications

| Application | Who | What it stresses | Status |
|---|---|---|---|
| **Museum revenue-operations** — admissions, timed ticketing, memberships, camps, rentals, retail, donations. Replacement for Versai. Specified against 2,464 behaviour records, a 248-table schema, 1.76M real transaction rows. | external evaluation, 2026-08-08 | exact decimal money, bounded resources under partition, offline-first multi-site, custodial→self-sovereign identity | **evaluating; NOT committed.** They said so explicitly and asked us not to prioritise on the assumption of an adopter. |
| **LLM inference (Gemma MoE)** — running a 14 GB MoE model, content-addressed weights, mesh routing with prefix affinity. | external evaluation, 2026-08-09 | heavy numerics, SANDBOX data channel, node size ceilings, verifiable compute | evaluating; landed on native-sidecar-plus-graph-control-plane |
| **Meta-circular admin (Composing)** — the admin UI that edits itself, built on the frozen v1 surface. | ours | composition of handlers, UI-as-graph, relative addressing | in design; see `.addl/phase-4-meta/COMPOSING-*` |

**The convergence is the point.** A ticketing system and an ML runtime have nothing in common
as products, and they hit the same three things: numerics that `Value` does not model, an IVM
layer that does not aggregate, and composition that does not compose. When unrelated workloads
fail the same way, the gap is in the engine rather than in the fit.

> ### ★ THE STRONGER INFERENCE — A BEN DECISION, NOT A TECHNICAL ITEM (added 2026-08-11)
>
> The paragraph above stops one step short. If two **randomly-arriving, unrelated** outside
> evaluations both hit binding, aggregation and rich types, then **most real applications
> will** — which reframes all three from adopter-specific gaps into **v1 completeness
> questions**.
>
> CLAUDE.md baked-in #15 defines v1 as the platform: installable and usable end-to-end. So the
> question that follows is: **does `v1-beta` ship with composition that does not compose and an
> IVM layer that cannot sum a column?** A handler that cannot receive its caller's input is not
> an exotic limitation — it is the joint where applications get built.
>
> Options: **(a)** tag as planned and treat both as Composing builds — the tag freezes the
> *interface*, and both gaps are additive with no wire change; **(b)** pull one or both into
> Core; **(c)** split — the binding-grammar reservation pre-tag (already planned) with the
> build after.
>
> **ORCH lean: (a).** But "is this shippable as v1" is precisely the judgment baked-in #15
> reserves for Ben, so this is recorded as an OPEN decision rather than a settled one, and it
> should not be allowed to resolve by expiry at the tag.

---

## 2. Conceptual clarifications — corrections to how we describe the engine

These came out of the evaluations and are **not** limitations; they are places where our own
framing was imprecise or where we had conflated independent properties. They belong in
`ARCHITECTURE.md` / `HOW-IT-WORKS.md` / `ENGINE-SPEC.md` eventually.

### 2.1 There is one graph, not two

Operation Nodes and data Nodes are Nodes in the same graph. A handler subgraph is stored as
ordinary Nodes — `plugin_library.rs` builds a real `benten_core::Subgraph` with `Anchor`
companions. Applications are composed by writing handlers that create and interpret custom
node/edge types, with the interpretation rules living in the handler subgraph, and then
composing those handlers into higher-level handlers.

Reasoning about "the code graph" and "the data graph" as separate things is the error
conventional runtimes train into you, and it produces wrong conclusions — it was the root of
two separate mis-framings during this analysis.

### 2.2 Bounded-by-construction does NOT require static operands

`README.md` ("Bounded by construction… guaranteed to terminate") and baked-in #4 are literally
true and stay true under dynamic operands. Termination rests on three mechanisms and **none of
them reads an operand value**: DAG structure validated at registration (Inv-1, Kahn
cycle-detect), a cumulative step budget (`evaluator.rs`, a counter), and a frame-stack cap
(`lib.rs`). A dynamic BRANCH condition changes *which* forward edge is taken, never
*whether* it is forward. **A value cannot increase the number of edges.**

These sentences should stop being cited as a reason operands must be static. They do not say
that, and the property they do assert is unaffected.

*Exception, and it is real:* **Inv-8** computes a multiplicative budget at registration by
reading ITERATE `max` and resolving CALL's callee. Its own comment (`invariants/budget.rs:157`)
names the attack: a subgraph claiming a cheap callee then pointing at an expensive one at
runtime. That is a genuine constraint on any dynamic-operand design.

### 2.4 Intrinsic goes in the node; extrinsic goes in an edge (added 2026-08-12)

> **Anything that is a property of the THING goes in the node. Anything that is a property of the
> DEPLOYMENT goes in an edge.**

The node's CID must be stable across deployments, so any fact that differs per-install cannot live
inside it. This single rule resolves two questions that looked unrelated:

- **A device-lowerable handler.** "This handler is lowerable to a GPU" is intrinsic — it travels
  with the subgraph and is part of its CID. "Run it on *that* device" is extrinsic — an edge to a
  device node. Put the placement inside the subgraph and the same handler acquires a different CID
  on every machine, destroying dedup and sharing.
- **Model weights.** *Granularity* (how the weights are cut into nodes) is intrinsic. *Sharding*
  (which machine holds which CIDs) is extrinsic. They are independent: a layer-sharded deployment
  can still store per-tensor nodes, because a shard is just "the set of CIDs machine A holds."

Worth promoting into `ARCHITECTURE.md` with the three clarifications above.

### 2.3 The gap is relative addressing, not "dataflow"

Calling this "missing dataflow" imports a values-on-wires model that does not fit. What is
actually missing is narrower and more precise: **a handler cannot say "the node I was given."**

`READ` resolves a `target_cid` (absolute, known at authoring time) or a `label` (a global
query, unscoped to the invocation). Neither composes: a handler composed into a larger
application does not know the CIDs its caller will supply, because those nodes do not exist
when the handler is authored.

We already designed relative addressing — **`SubgraphSpec`** (Roots / Expansion / Inclusion /
Termination) is exactly traversal-from-anchors, ratified for Sharing & Confidentiality. We gave
it to **sharing** and not to **execution**. That asymmetry looks accidental.

---

## 3. Findings

Legend — **Shape:** engine / application. **Freeze:** does the v1-beta tag foreclose it?
**Status vocabulary (deliberately narrow, because "answered" was caught doing two jobs):**
- **OPEN** — gap confirmed, no settled design.
- **DESIGNED** — investigation concluded and a design record exists; **the gap itself is still
  open**. Says nothing about code.
- **ANSWERED** — no engine change needed; the supporting pattern *ships today*. The strongest
  claim available short of LANDED, and reserved for it.
- **LANDED** — the change is in, with the commit named.

A status may only move rightward with evidence. "The design is written" never justifies
ANSWERED — that conflation is the record-overstates-the-binary shape (CLAUDE.md rule 14) that
this project spent its freeze phase hunting, and it was caught once in this very document.

### 3.1 Composition does not compose — relative addressing

**Shape:** engine · **Freeze:** narrowed to 4 small pre-tag items · **Status:** DESIGNED —
**the gap is still open**: user handlers still drop caller input, and nothing is built. Design
at **`docs/future/binding-grammar.md`** (R0-input; build gets its own ADDL pipeline); receiving
row **`phase-4-backlog.md` §4.170**; the 4 pre-tag items ride the W-REC wave.

**Resolution summary.** Investigation found not three but **four** partial implementations of
handler-input binding — the crud specialization, `EvalContext`, the frozen-but-empty wire slot,
and a fourth nobody had connected: **STREAM already ships a live sigil resolver**
(`engine_stream.rs:1075` — `$input` and `$input.<field>` resolved against the caller's input),
and `DSL-SPECIFICATION.md:88` already teaches that convention as the normal way to author
handlers. The design therefore *generalizes* rather than invents: promote STREAM's grammar to
the walk, with a boundedness carve (data properties bind; graph-shape/budget properties stay
static, preserving Inv-8 and install-time consent) and a position-scoped registration-time
reservation of the `$` namespace on operation nodes. Only the reservation + disclosures are
pre-tag; the mechanism is post-tag additive. Details, security notes, and the
considered-and-declined list are in the design doc.

> ### ⚠️ THE DESIGN RECORD COVERS ONE HALF OF THIS GAP (added 2026-08-11)
>
> Relative addressing is **two** things: (a) a handler saying *"the node I was given"*, and
> (b) getting from that anchor to related nodes. `binding-grammar.md` answers (a) — `$input`
> and `$input.<field>` give the anchor and **property access**. It gives **no edge traversal**,
> and READ confirms the hole: it resolves by `cid` or by `label`, and there is no *"read the
> node at the end of edge E from node N."*
>
> That matters because the composition model this section exists to serve puts **edges** at the
> centre — handlers that create and interpret *custom node AND edge types*. A handler that can
> name its input but cannot walk an edge from it still cannot traverse the edge types the model
> is built on.
>
> **The answer was named during the originating investigation and then dropped from the record:**
> `SubgraphSpec` (Roots / Expansion / Inclusion / Termination) IS traversal-from-anchors,
> already ratified for Sharing & Confidentiality. The plausible complete shape is **a binding
> names the root and a SubgraphSpec-shaped expression performs the traversal** — we gave the
> engine relative addressing for *sharing* and not for *execution*, and that asymmetry looks
> accidental. `binding-grammar.md` does not mention `SubgraphSpec` anywhere; it should.
>
> Status is unchanged (DESIGNED, never LANDED) — what was wrong was the *scope* the record
> implied, not its status. **A related pre-tag verification is now owed; see §5.**

Built-in `crud()` handlers receive their caller's data; user-authored handlers do not. Same
`engine.call`, same evaluator, one `if` statement apart (`engine.rs:3966-3971`):
`subgraph_for_crud` does `input.properties.clone()`; `subgraph_for_spec` takes `_input: &Node`.

The engine's own workaround for `crud` is **per-call specialization** — clone the handler
template and patch the caller's property bag into WRITE's static properties
(`engine.rs:4320-4323`). That works because the engine generates both sides. A user-authored
handler has no equivalent, and `CALL` hands the callee `Node::empty()` (`call.rs:127`), so
handler-to-handler composition — the thesis the plugin model and the meta-circular admin rest
on — does not carry data across its one joint.

**Evidence it is unbuilt rather than designed out:**
- `input: Node` is threaded through `call_handler` → `dispatch_call` → and then underscore-bound
  at `run_inner` (`evaluator.rs`). A parameter threaded three layers and discarded is not a
  design.
- `EvalContext` (the `$input` / `$result` / `$item` / `$index` binding container) is complete,
  unit-tested, `pub use`d, and appears 56× in the frozen `docs/public-api/benten-eval.txt` —
  with zero production callers of `push_scope` / `pop_scope`.
- **`context_binding_snapshots` is frozen into the v1-beta `ExecutionState` wire envelope**
  (`exec_state.rs:272`) carrying the doc comment *"CID-substitution attack mitigation"* — and
  every production site passes `Vec::new()`. We attack-hardened and froze a wire slot for a
  feature we never built.
- Phase-1's plan named context binding as deliverable **E3, "Core of Phase 1"**, with a
  must-pass `tests/context_scoping`. The container shipped, the walker shipped, nobody joined
  them, the test was never written, and E3 was marked closed.
- `docs/DSL-SPECIFICATION.md:60-67` normatively states TRANSFORM `input`, BRANCH
  `condition_value` and CALL `parent_scope` are *"populated by the engine compile path."* Zero
  production writers. **This is a FALSE-RECORD in the sense of CLAUDE.md rule 14 and it would
  freeze.**

**Freeze coupling:** the reserved wire slot already exists, which is the expensive part. What
needs deciding pre-tag is whether an op node needs a *frozen* way to express "traverse edge E
from my anchor," or whether that is entirely internal to the walker plus a property convention.

### 3.2 Rich value types — TWO findings, not one

Both evaluations asked "types beyond what `Value` supports" as if it were one question. It is
two, with different mechanisms — and merging them makes the answer look like "add a type" when
it is really "declare a meaning" plus "don't inline the blob."

#### 3.2a Numeric interpretation — decimal, bf16/f32, timestamp, big integer, set

**Shape:** engine, resolved · **Freeze:** no · **Status:** ANSWERED — pattern already shipped

`Value` has 8 variants and cannot grow: it is `#[serde(untagged)]`, so a variant is identified
by CBOR major type, and DAG-CBOR permits exactly one tag — 42, for CIDs
(`serde_ipld_dagcbor-0.6.4/src/de.rs:323`). A new variant needs either a discriminant
(re-encodes every existing variant) or decode-sniffing (silently reinterprets data legal
today). Both rejected.

**The answer already exists, twice, in shipped code.** `bytes-cid` and `timestamp-hlc` are not
`Value` variants — they are *interpretations layered over* `Value::Bytes` and `Value::Int`,
declared by a schema Node (`docs/SCHEMA-DRIVEN-RENDERING.md:52`). `Scalar` is
`#[non_exhaustive]` with a comment from our own pre-tag F-22 sweep: *"a future scalar kind lands
additively"* (`vocab.rs:161`), and unknown scalars typed-reject via
`E_SCHEMA_VOCAB_SCALAR_UNKNOWN`. **A decimal is the third member of a two-member family, and one
of the existing two is a timestamp.** The museum's 3-scale money and the LLM project's
bf16/f32 activations are the same finding: bits an existing variant already carries, whose
meaning the schema declares.

Nothing here is now-or-never. The honest caveat: nothing yet validates an instance `Value`
against its declared `Scalar` — that is a missing implementation over shipped machinery, not a
missing concept, and it needs no wire change.

**Three sharpenings (2026-08-11), all of which belong in any reply to an adopter:**

1. **`Decimal` is NOT a minted `Scalar`.** The shipped set is exactly eight — `Text`, `Int`,
   `Float`, `Bool`, `Bytes`, `BytesCid`, `TimestampHlc`, `Null`. "The third member of a
   two-member family" implies this and never states it. **The pattern ships; the member does
   not.** Minting it is additive and post-tag-safe, but this section must not be read as "you
   can do this today."

2. **Scale belongs in the SCHEMA (per-field), never in the value (per-instance) — and that is
   why our answer BEATS the variant that was asked for.** If scale rides on the value, then
   1.50 as `(unscaled 150, scale 2)` and 1.5 as `(unscaled 15, scale 1)` are different bytes →
   different CIDs → **the same amount has two identities**, and content-addressed equality and
   dedup silently break. A `Value::Decimal` variant necessarily carries its own scale, so it
   would have shipped exactly that bug. This resolves the RDF tension flagged during the
   typed-values pass (*is `"1.50"^^xsd:decimal` the same term as `"1.5"^^xsd:decimal`?*) and
   never closed: in a system where the CID is the identity, canonical scale is not a nicety.
   Mechanically supported today — schema fields are Nodes with property bags, so a `scale`
   property needs no vocabulary change.

3. **The museum's objection does not "dissolve" — it becomes buildable.** They objected that
   scale-in-schema "moves it into metadata that nothing validates." Today nothing does. The
   difference from Versai is **structural, not yet actual**: a column comment can *never* be
   enforced; a Benten schema is content-addressed graph structure the walker already traverses,
   so it *can* be. State it that way; the stronger phrasing was an overstatement made in
   conversation while this document stayed correct.

**One pre-tag item — a disclosure, not a mechanism:** `benten_core::Value` is named nowhere in
the freeze record. Freezing a type system without stating it is the rule-14 shape. See §5.

#### 3.2b Bulk data — tensors, media, anything ≥ tens of KiB

**Shape:** engine, resolved · **Freeze:** no · **Status:** ANSWERED — out-of-line, never inline

The LLM project's second problem is not a type problem: `Bytes(Vec<u8>)` is owned, so every
read of an inlined 285 MB tensor is a copy, against a machine achieving ~26 GB/s. No scalar
interpretation touches that — it is a memory-access question.

**The answer is the two-tier shape every content-addressed system converged on** (git trees vs
blobs; IPFS DAG nodes vs chunked files; iroh docs vs blobs): small canonical inline properties,
and bulk behind a CID reference. **The recommendation stands. Everything this section originally
said in support of it was wrong, and it is corrected in place below rather than quietly edited.**

> ### ⚠️ FALSE-RECORD, corrected 2026-08-12 — this section was stamped ANSWERED on evidence that
> does not exist. The error is ORCH's and it reached adopter-facing text.
>
> **Clause 1 — "Benten already has all the pieces."** Checked all four. `bytes-cid` is a *schema
> interpretation* of a `Value::Bytes`, not a storage tier. `two_cid_map` is a **sync/encryption**
> plaintext-CID → stored-CID mapping, not storage. `IROH_BLOCK_SIZE` chunking is **per-chunk
> AEAD**, a crypto layout. And the only blob facility, `RedbBlobBackend`, stores blobs **AS NODES**
> (`blob_cid` + `blob_bytes`, a `Value::Bytes` round-tripped through DAG-CBOR) in the
> **privileged** `system:ModuleBytes` zone, with an **O(N) linear scan** decoding every node body
> in the zone. It exists to close Compromise #17 for SANDBOX module bytes. **None of the four is
> an out-of-line tier. There is no general out-of-line tier.**
>
> **Clause 2 — "the decode bound is the tripwire that enforces this."** `MAX_DECODE_BYTES` is on
> **`Subgraph`** and guards subgraph decode only. There is **no per-node body bound at all** on
> the native Rust write path — `put_node` has no length check. So nothing makes "you inlined a
> blob" fail fast from Rust; a 285 MB node encodes, hashes and persists.
>
> **What actually bounds it, and it is 16× tighter and in a different subsystem:**
> `MAX_MST_MESSAGE_BYTES` = **1 MiB**, receiver-side in `benten-sync`'s MST diff protocol, inside
> a 4 MiB frame cap — both byte-pinned in the frozen-bytes corpus. That is a **sync** bound, not a
> storage one, and it rejects this section's own recommended granularity (3.3 MB per expert) too.
>
> **The correct conclusion, which the mistaken evidence was pointing at anyway:** bulk content
> must not travel the MST diff path at all. Weights are not graph deltas; they are immutable
> content wanting a blob protocol with flow control, resumption and range requests — which is
> what **iroh-blobs** does, and iroh is already a dependency. Raising the MST caps would be the
> wrong fix: they are receiver-side DoS bounds on an untrusted path, and raising them raises
> attacker leverage to serve a use case that should not be there. On a blob channel the 1 MiB cap
> never applies and per-expert granularity works after all.
>
> **Disposition:** split this row. **§3.2b-i — the adopter pattern (small nodes + CID reference to
> out-of-line bulk) is ANSWERED**, with the mechanism named correctly: today that means the bytes
> live outside the engine and the graph holds a manifest of CIDs, which works now and needs
> nothing new. **§3.2b-ii — a first-class engine blob tier is OPEN**, and the freeze question is
> owed (a new backend trait is additive; `Value` cannot grow regardless, which is *why* the tier
> is not freeze-forced).

**Considered and declined as a pre-tag change:** switching `Bytes(Vec<u8>)` to a cheap-clone
payload (`Arc<[u8]>` / `bytes::Bytes`). Pre-tag is genuinely the only window (a payload-type
change on the frozen enum is HALT-AND-SURFACE after), but with bulk out-of-line, inline bytes
stay small and copying them is cheap; the zero-copy need lives at the blob tier, which is not
`Value` and is not frozen. Recorded so it reads as decided, not missed.

### 3.3 IVM does not aggregate

**Shape:** engine · **Freeze:** no (additive; disclosure sentences owed) · **Status:**
DESIGNED — **the gap is still open**: the engine still computes no balance. Design at
**`docs/future/ivm-aggregation.md`** (R0-input; build gets its own ADDL pipeline); receiving
row **`phase-4-backlog.md` §4.171**.

`ViewResult` has exactly three variants — `Cids`, `Current`, `Rules` (`view.rs:250`).
`Projection::apply` is the identity (`algorithm_b.rs:441`). There is no `sum`, `fold`,
`aggregate` or `reduce` anywhere in `benten-ivm`.

**Design summary.** A bespoke abelian fold kernel (Sum/Count, optionally grouped) rides
`Strategy::B`, declared by an additive `aggregate` field on the `#[non_exhaustive]`
`UserViewSpecBuilder`, returning an additive `ViewResult::Aggregates` variant, accumulating in
i128 and crossing to TS as a decimal string, always. DBSP's algebra is adopted as theory (a
linear operator is its own incremental version); the `dbsp` crate is declined (69-dep
scheduler-owning circuit runtime inside a must-not-block post-commit callback). Four
first-hand-verified facts shaped it: `Deleted` events carry the read-before-delete pre-image,
so retraction is O(1) — no Z-set machinery needed for Sum/Count; `ChangeKind::Updated` is
never emitted in production; version-chain CURRENT moves emit **no event**, so versioned
entities typed-reject at aggregate registration; and sync merges persist the zone as ONE
`"version"` snapshot node, so **balance views are per-engine at v1** (stated, not hidden). The
real feature is the correctness contract: backfill-on-register, fail-closed error routing (the
subscriber today logs fold errors and leaves the view Fresh — the silent-wrong-balance
channel), the uniqueness-nonce contract on ledger rows (byte-identical deposits dedup to one
CID), and typed refusal of floats, mixed scales, and order-sensitive folds. §3.2a's
schema-declared scale is confirmed as what makes balances exact integers.

### 3.4 Bounded resources under partition — "at most N"

**Shape:** engine · **Freeze:** no structural (disclosure + one false-record fix owed) ·
**Status:** DESIGNED — **the gap is still open**: the engine still admits the oversell. Design
at **`docs/future/bounded-resources.md`** (R0-input; build gets its own ADDL pipeline; also the
canonical tracked home for the `executionPolicy` taxonomy, since `PLATFORM-DESIGN.md` is
local-only); receiving row **`phase-4-backlog.md` §4.172**. **The design-space map for every
OTHER regime — Ben's "one system for the tills AND all kinds of decentralized balancing" — is
`docs/future/bounded-resources-spectrum.md`**, which sits above the museum-regime record and does
not re-decide it.

**Two corrections to the summary below, from the spectrum pass** (both ORCH-verified at source):
(a) on the WRITE-primitive path — the one an admission check sits on — `CapWriteContext.actor_cid`
is never populated (`primitive_host.rs`), so the policy sees a **scope**, not a
**principal**: single-owner exclusivity rests on non-syncing local grants plus operational
discipline, not on engine-enforced principal ownership, and the record must say so. (b) The
borrow-on-demand question resolves as a *continuum*, not a rival design — deal 100% of C to one
peer and escrow **is** single-owner; the guard should therefore be written
`count(rows attributed to me) < allowance(me)`, which costs nothing today and is what keeps one
mechanism from forking into two.

**Design summary.** Single-owner admission: a bound is data (`bound:decl`), exactly one engine
— the declared `UptimePolicy::AlwaysOn` peer — holds the admit grant, and the check runs
**inside the commit's own serialization domain**: an additive in-transaction count read at the
graph layer, below all THREE write-entry families (the `Engine::transaction` cap-hook wrapper,
direct `backend.transaction` callers incl. `create_node` and `append_version` — the sync-merge
terminal write — and the privileged `put_node_with_context` family). redb's single-writer lock
is the serial point; the write-through contract plus the Inv-13 in-tx read precedent make the
check sound with ONE structural addition. The §3.3 fold is the REPORTING read, never the
enforcement read (it updates post-commit — stale by every in-flight tx); the
enforcement-vs-reporting split is stated once for both records. Ownership is an exclusive
write-capability (the record is discovery and audit; the grant is law), with a load-bearing
REQUIREMENT that each bound's labels map to a dedicated sync zone (the per-row merge recheck
is zone-granular). Refusal is a typed domain outcome carrying `{available, bound, owner_did}`;
the manager override is a first-class attributed write through the same guard — "never exceed
silently," stated positively. Escrow/bounded-counter CRDTs declined on the museum's 204K-booking
measurement with a recorded revisit-iff (`C > 3·n·q_p99` AND measured multi-homed demand) and
the Bailis theorem (coordination relocates, never removes).

CRDT convergence guarantees everyone agrees on the set; it does not guarantee the set satisfies
a bound. Two offline tills each sell the last seat, both writes are valid, both survive the
merge, the venue is oversold.

`executionPolicy: origin-only | local | leader-elected` is specified in `PLATFORM-DESIGN.md`
§3.2 and has **zero implementation hits** across every crate.

The museum evaluation measured this against 204,371 real bookings and **reversed its own
recommendation**: single-owner runs 91.6% local at any replica count; escrow 58.9% falling to
30.5%, and ties or loses in every capacity band at n=8. The crossover is not a seat count but
`C` vs `n · q_p99` — a 12-seat camp at n=3 with a p99 party of 9 has nothing to lease. Bailis
et al. (PVLDB 8(3), 2014) prove this invariant class is not coordination-free: escrow relocates
coordination, it does not remove it.

**Useful for us:** `executionPolicy` is a *dispatch policy on existing handlers*, not a 13th
primitive, so it does not touch baked-in #1. And it is additive, so it has no deadline.

### 3.5 SANDBOX has no data channel

**Shape:** engine · **Freeze:** partly · **Status:** the **return** half is CLOSED (code,
2026-08-11); the **argument** half remains OPEN.

Baked-in #16 designates SANDBOX as the escape hatch for "heavy math, ML inference, custom
transformers." Two halves were reported together; they had different answers.

**Return half — CLOSED.** `encode_return_values` in `crates/benten-eval/src/primitives/sandbox.rs`
had a catch-all arm commented *"V128, FuncRef, ExternRef — current corpus doesn't use them;
encode as zero placeholder"* that emitted sixteen zero bytes and an `Ok`. Confirmed against the
shipped executor before the fix: a module returning
`v128.const i32x4 0x11111111 0x22222222 0x33333333 0x44444444` returned
`Ok(SandboxResult { output: [0; 16], .. })`. A wasm SIMD kernel — the obvious way to write the
heavy math we explicitly invited — got **silent wrong answers on the path we designated for ML
inference.**

Three corrections to the original report, all ground-truthed:

- The catch-all covered **six** `wasmtime::Val` variants at wasmtime 43, not the three the
  comment named (`V128`, `FuncRef`, `ExternRef`, `AnyRef`, `ExnRef`, `ContRef`) — the comment
  was stale across the 40→43 bump.
- **Two** were reachable, not one: `v128` *and* `funcref` both compiled and executed to sixteen
  zero bytes. `externref` / `anyref` fail earlier at `Module::new` only because the `gc` cargo
  feature is off — a feature flag, not a guarantee.
- The worst shape was multi-value: `(result i32 v128)` returned `[7,0,0,0] ++ [0; 16]`, a
  **partially** correct answer whose correct leading scalar lends false credibility to sixteen
  invented bytes.

The load-bearing error in the original reasoning was *"current corpus doesn't use them"*: the
corpus is OUR trusted test input, whereas the guest module is the UNTRUSTED party. What our
fixtures happen to return says nothing about what a guest may return.

Fixed per rule 15 (code, not doc) as a pre-call ABI gate plus an encoder backstop, both routing
to `E_SANDBOX_MODULE_INVALID`; no new ErrorCode was minted because that code already covers
`run`-entry/ABI incompatibility ("module has no exported `run` function" has always lived
there). The gate rejects **before** the guest body executes, so the failure is free of
guest-observable side effects. Scalar encodings are byte-identical, so no golden moved.
Falsification arms at
`crates/benten-eval/tests/sandbox_unencodable_return_type_rejected.rs` plus encoder unit tests
in `primitives::sandbox::tests`; four mutations were run and reported.

Why this class and not merely a wrong number: the sixteen invented zeros **hash to a perfectly
valid CID**. A content-addressed engine cannot tolerate a silent wrong answer the way a
request/response service can, because the wrong answer becomes a durable, referencable,
signed-over identity rather than one bad reply.

**Argument half — still OPEN.** The guest is still invoked with an empty argument slice, so
there is no inbound data channel. It is the actual "no data channel", it is not a one-line fix,
and nothing in this wave touches it. This section remains its own tracked record.

Open question for freeze scope: is the host-fn table extensible by third parties post-v1
without a fork?

### 3.6 Custodial → self-sovereign identity handoff

**Shape:** engine · **Freeze:** ⚠️ **YES — corrected 2026-08-11; the previous "no" was never
checked** · **Status:** CARRIED to Fork B for the DESIGN, but with a now-or-never component

> **The "Freeze: no" above was an assumption stated as a fact, on the document whose job is
> tracking exactly that. Checked 2026-08-11; it is wrong.**
>
> The custodial handoff **is** a rotation: `previous_did` = the organisation's custodial DID,
> `next_did` = the person's self-sovereign root, signed by the old (custodial) key. That is
> literally `benten_id::did_rotation::RotationAttestation`.
>
> And that structure is frozen three ways over: it is a **signed wire structure** (Ed25519 over
> the canonical bytes of the `previous_did` / `next_did` / `superseded_at` tuple), it appears
> **57 times in the frozen `docs/public-api/benten-id.txt` baseline**, and its canonical bytes
> have **two entries in the frozen-bytes corpus** — a REQUIRED CI check. Yet it is named
> **nowhere** in `V1-FROZEN-INTERFACE.md` or `V1-WIRE-INVENTORY.md`. Same shape as
> `benten_core::Value`: freezing de facto while the record does not mention it.
>
> **So the signing tuple locks at the tag.** Anything the custodial flow needs beyond those
> three fields cannot be added afterwards without a wire break. The concrete instance:
> **nothing distinguishes "custody was deliberately handed over" from "a key was compromised"** —
> both are `AttestationKind::SupersededBy`. A hosted product handing users their sovereignty and
> an incident response are semantically opposite events with identical wire encodings, and for
> an organisation with audit obligations that distinction plausibly matters. Post-tag it can
> only live out-of-band, forever.
>
> **DECIDED 2026-08-12 — change NOTHING in the tuple; the pre-tag obligation is DISCLOSURE only.**
> Three shapes were weighed: add a `reason`/`kind` enum to the signed tuple; add a generic
> `context: Option<Cid>` extension slot; or leave the tuple alone and carry handover semantics in a
> separate content-addressed record that REFERENCES the rotation.
>
> The enum is the trap. A `reason: CustodyTransfer` field is **the old key asserting its own
> motive** — a self-declared value nothing verifies, frozen into permanent bytes. That is precisely
> the `UptimePolicy` shape: a declared field that reads as a guarantee and enforces nothing, which
> this project has now found FOUR instances of in one week.
>
> And the deeper reason: **what they want to prove is not cryptographically provable.** "The
> organisation destroyed its copy" cannot be attested BY the organisation — a signature saying "I
> destroyed it" is worth nothing. It is evidenced two ways only: a signed *policy commitment* (a
> separate artifact, additive forever), and **the key never being used again**. The second is
> already free — once rotated, any use of the old key is a detectable protocol violation. So their
> feared failure ("an organisation retains silent access") is not silent; it is detectable by
> design, if someone looks. The missing piece is the LOOKING, which is a watch in Composing, not a
> field in the wire.
>
> The tuple stays minimal and correct: it attests one thing — *this key authorised that successor*.
> Rule-12 clause (c), DISAGREE-with-reason, not a deferral.
>
> This does not change Fork B: the rotation-survival and multi-device *design* still belongs in
> Composing. What changes is that the **tuple's adequacy is a pre-tag question**, and it was
> hiding behind an unverified "no".

An organisation mints and custodies a user DID; the person later installs their own engine and
the custodial key signs a rotation to their self-sovereign root. This is how anyone joins a
decentralised network from a hosted product.

Maps directly onto **Fork B**, ratified 2026-07-17: rotation-survival and multi-device deferred
to Composing with named must-nails. The museum's flow is a concrete driving use case for
exactly those must-nails, and the DID→principal-CID mapping question (D-102 Q3) is the same
seam.

### 3.7 Phantom config

**Shape:** engine · **Freeze:** the false claim freezes · **Status:** the three EVALUATOR knobs
and `memory_bytes` are CLOSED (code, 2026-08-11). `max_wasm_stack` is CLOSED as a *diagnostic*
and recorded as a per-process design property rather than a knob. Two further sites found while
checking — `output_max_bytes` and `GrantReaderConfig::max_chain_depth` — have their false claims
struck here and their wiring received at `phase-4-backlog.md` §4.173, which is this section's
receiving row.

The originating report: `ENGINE-SPEC` asserts all numeric limits are configurable per capability
grant, and five sites reportedly had no production writer — `InvariantConfig`, `memory_bytes`,
`max_wasm_stack`, `max_stack_depth`, `RunOptions::budget`. All five were checked individually;
the count was right and one verdict was not (row 5 below).

**Where the claim actually lives — and a warning for the next reader.** `docs/ENGINE-SPEC.md` is
**gitignored** (`.gitignore` line 79, local-only). It therefore does **not** materialise in a
`git worktree`, and a first pass from inside one concluded the file had been retired. It has not.
Verify against the main checkout before writing any finding about it — inferring
*does-not-exist* from *cannot-see* is the standing trap here, and this row walked into it.

The live text, in the invariant table and the line under it:

- *"Max operation-subgraph depth (**configurable per capability grant**)"* — invariant 2 row
- *"Max fan-out per node (configurable)"* / *"Max SANDBOX nesting: configurable"* — invariants 3, 4
- *"**All numeric limits are configurable per capability grant** — the operator (or community
  governance) decides the bounds."*

A separate instance of the same claim, the `E_INPUT_LIMIT` fix-hint *"Limits are configurable via
the engine builder"*, had already been struck as a false record (see the `ERROR-CATALOG.md`
blockquote and the `V1-FROZEN-INTERFACE.md` baseline-update log).

**"Per capability grant" is the falser half, and it is false on its own terms.** Worth separating,
because a doc-fix that only softened "configurable" to "partly configurable" would leave the false
half standing. "Configurable" at least pointed at something real. But *nothing anywhere derives a
numeric bound from a grant*: `CapBundle` is `{caps, description, signature}` and carries no numeric
field at all; grants drive an allow/deny intersection, not budget sizing. A reader provisioning
per-tenant budgets from grants is not mis-tuning a knob — they are building on a mechanism that
exists at no layer.

**Half of the ENGINE-SPEC sentence is now true and half must come out.** "Configurable" and
"the operator decides the bounds" are true as of this change — per *engine*, via
`EngineBuilder::invariant_config`. **"Per capability grant" is false and should stay false**;
see the recommendation at the end of this section. The owed edit, named here so it is dispositioned
rather than deferred (HARD-RULE-12 clause (b)) — this row IS the destination, and it receives the
exact replacement text now:

> **`docs/ENGINE-SPEC.md`, the line under the invariant table.** Replace
> *"All numeric limits are configurable per capability grant -- the operator (or community
> governance) decides the bounds."*
> with
> *"All numeric limits are configurable per engine, via `EngineBuilder::invariant_config` — the
> operator decides the bounds at engine-construction time. They are NOT per capability grant:
> a registered subgraph's validity is a property of its content, not of who is calling it."*
>
> **Same file, invariant 2 row.** Replace *"(configurable per capability grant)"* with
> *"(configurable per engine)"*.

**LANDED in the primary checkout 2026-08-11**, and broader than the minimum above: `ENGINE-SPEC`
§4.1 is now a full "which of these limits are configurable, and by whom" section enumerating the
*three* real surfaces (`engine.toml` / operation-Node properties / `EngineBuilder`) plus direct
`benten-eval` embedding, with the "per capability grant" sentence quoted and struck rather than
silently deleted — it was load-bearing in two outside evaluations, so a reader returning to it needs
to see that it was wrong, not merely find it gone. Applying it also surfaced a **fresh** false record
in the drafted replacement: it described `engine.toml` as "read at `Engine::open`", which is untrue
(`EngineConfig::load_or_default` has no production caller — see `phase-4-backlog.md` §4.173 B), so
§4.1(a) now states that the engine has **no functioning operator-facing configuration surface** at
the freeze.

The minimum replacement text is retained in the blockquote above **on purpose**: the file is
gitignored, so it exists in exactly one checkout, cannot be recovered from git, and CI can never
gate it. This tracked row is the only durable record of the correction. **Verification of that edit
is therefore manual and permanently outside CI** — which is itself the argument in §4.173 E.

**Verdicts on the three evaluator knobs (each checked individually):**

| Knob | Verdict before | What was actually true |
|---|---|---|
| `InvariantConfig` | **(b) read, never written outside tests** | Read on every registration — all three of `register_subgraph` / `register_subgraph_replace` / `register_subgraph_aggregate` called `sg.validate(&cfg)`. But each built `InvariantConfig::default()` **inline**, so no `Engine` caller could supply one. The only non-default constructions in the tree were `SubgraphBuilderExt::build_validated_with_max_depth` (a test-support builder whose sole caller is `crates/benten-eval/tests/invariant_2_depth.rs`) and test bodies. |
| `max_stack_depth` | **(b) read, never written outside tests** | Read in `Evaluator::step` on the real dispatch path. The single engine-side `Evaluator::new()` left it at the literal `64` and never assigned it; the only writer in the tree was `crates/benten-eval/tests/evaluator_stack.rs`. |
| `RunOptions::budget` | **(b) written, but only from a test-gated source** | The engine *did* populate it — from `EngineInner::test_iteration_budget`, whose only setter `Engine::testing_set_iteration_budget` is `#[cfg(any(test, feature = "iteration-budget-test-grade"))]`. In a default build the field is permanently `None`, so the budget was always `DEFAULT_ITERATION_BUDGET`. `RunOptions` itself is honest as a *library* surface — a downstream crate driving `Evaluator::run_with` could always set it. The Engine could not. |

None of the three was verdict (c); every one is read on a real path. The defect was the missing
writer, which is why the fix is code.

**The fix (code, per rule 15).** Two additive `EngineBuilder` methods —
`invariant_config(InvariantConfig)` and `iteration_budget(u64)` — stored on `EngineInner` and
consumed at the four sites that previously hardcoded. The frame cap is **derived**, not exposed:

> `Evaluator::step` pushes one frame per non-terminal step and pops only on a `"terminal"` edge,
> so `max_stack_depth` is operationally *"nodes walkable along one path"* — the **same quantity**
> Inv-2 `max_depth` bounds at registration (`longest.len() > max_depth`). Both were the literal
> `64`, agreeing **by coincidence**. Raising `max_depth` alone would have let a 100-node handler
> register cleanly and then die at call time with `EvalError::StackOverflow`. That is not
> hypothetical: deleting the derive line makes
> `raised_max_depth_registers_and_walks_a_100_node_chain` fail with exactly that error. So the
> engine derives the runtime cap from `max_depth` — one knob, and the mismatch is
> unrepresentable.

Falsification arms at `crates/benten-engine/tests/engine_evaluator_limits_are_configurable.rs`;
each test names the exact one-line revert that breaks it, and all three mutations were run.

**Recommendation: do NOT make "per capability grant" true.** DISAGREE-with-reason, recorded so it
is decided rather than left dangling. Per-grant numeric limits would mean the bound a subgraph is
validated against depends on *who is calling*, so the same registered handler would be valid for
one principal and invalid for another. Handler identity is content-addressed; validity would stop
being a property of the content. Nothing in the tree has ever implemented per-grant limits, and
the one live assertion of it is the ENGINE-SPEC line above, whose correction is specified there.
Ben's call if he wants it built; the recommendation is to strike the claim and keep limits
per-engine.

**Not changed, with reason:** `Engine::register_crud` still calls the eval-side
`build_validated()` (hardcoded default config). Its subgraph is a fixed 2-node READ→RESPOND
shape, so the only configured bound that could ever reject it is `max_nodes < 2` — degenerate.
Threading the config there would require a new `build_validated_with_config` on `benten-eval`'s
**frozen** public surface for no reachable behaviour difference. DISAGREE-with-reason, recorded
here rather than deferred.

**The SANDBOX pair — a separate surface, closed in the same wave, opposite answers.** Rule 15 asks
which one we would write from scratch today, and `memory_bytes` and `max_wasm_stack` answer
differently. That difference *is* the split.

| Knob | Verdict before | Disposition |
|---|---|---|
| `memory_bytes` | **(b) read, never written outside tests.** Reaches `SandboxResourceLimiter`, so it looked wired. The one production construction (`primitive_host.rs::execute_sandbox`) applied per-handler overrides for `fuel` / `wallclock_ms` / `output_limit` and the manifest random budget — never memory. Every production SANDBOX call had always run at exactly 64 MiB, while the doc advertised a DSL override (`memoryLimitBytes`) whose string occurred in **two places in the repository, both in that one doc file**. | **CODE moved.** ~25 LOC mirroring the `output_limit` block: `memory_limit` read per-handler and applied **tighten-only**, joined to `SANDBOX_INT_PROPS` for `E_DSL_INVALID_SHAPE` typing, mirrored as `memoryLimitBytes` on both `SandboxArgs` variants. Tighten-only is deliberately asymmetric with `fuel`/`output_limit`: exhausting memory OOM-kills the host **process**, taking every other handler with it, so a handler may lower its own ceiling and must not raise the bound protecting everyone else. Over-ceiling requests are ignored **and `tracing::warn!`-logged** — the codebase already rejected silent clamping at `wsa-g7a-mr-3`. |
| `max_wasm_stack` | **(b) and worse — accepted-then-ignored.** The enforced ceiling is set on the process-wide `OnceLock<wasmtime::Engine>` in `sandbox/instance.rs`; the field was consumed at exactly one place, populating the `SandboxError::StackOverflow { max_wasm_stack }` **error payload**. Proven by execution, not by reading: a guest run with `max_wasm_stack: 4_000_000` overflowed at 512 KiB and reported *"guest exceeded max_wasm_stack (4000000 bytes)"*. An operator raising the knob to fix an overflow got a message confirming a limit that was never in force — a bug hunt with no bug at the end of it. | **DOC was right; the CODE was lying in the diagnostic.** This is the narrow case rule 15 permits: we genuinely do not want a per-call stack override, because `max_wasm_stack` is a `wasmtime::Config` setting fixed for an `Engine`'s life, so per-call variation means a fresh `wasmtime::Engine` per call — discarding the module cache the singleton exists to hold. Recorded as a design property so nobody "fixes" it without pricing the cache loss. The *diagnostic* half was a code bug regardless: `ENGINE_MAX_WASM_STACK_BYTES` is now the single source of truth, `MAX_WASM_STACK_DEFAULT` **derives** from it (the two independent 512 KiB literals become one, so divergence is unrepresentable rather than merely tested-against), and the executor passes the **enforced** ceiling into the error payload. The field keeps an `INERT` rustdoc; removing a `pub` field would be a breaking narrowing of the frozen surface. |

Falsification arms at `crates/benten-engine/tests/sandbox_memory_limit_per_handler.rs` and
`crates/benten-eval/tests/sandbox_stack_overflow.rs`; three mutations were run.

**Two more sites found while checking, neither in the original five.** Both are receiving-row
entries at `phase-4-backlog.md` §4.173, and both have their false *claims* struck in this same
commit — the disclosure never waits on the wiring.

- **`output_max_bytes` is validated at registration and never read at runtime.**
  `invariants/sandbox_output.rs` reads the node property and rejects declarations above the 16 MiB
  ceiling; the runtime budget comes from a **differently named** property, `output_limit`, feeding
  `SandboxConfig::output_bytes` (default 1 MiB). A handler declaring `output_max_bytes: 4_000_000`
  passes registration and then silently runs under 1 MiB. The `InvariantConfig::max_sandbox_output_bytes`
  rustdoc asserted that the runtime `CountedSink` enforces the per-node value — **a FALSE-RECORD in
  a rustdoc**, the rule-14 shape in the place reviewers are least likely to look. That sentence, and
  its twin in `invariants::sandbox_output`, are struck here. The *behaviour* fix is a genuine fork
  and is **surfaced, not decided**: making the runtime honour `output_max_bytes` widens budgets for
  handlers that declare it, while typed-rejecting the inert declaration narrows an accepted input
  set on the freeze branch. §4.173 A.1 holds the decision.
- **`GrantReaderConfig::max_chain_depth`** (default 64) is written only by
  `crates/benten-caps/tests/grant_reader_max_chain_depth.rs`, against a synthetic harness its own
  rustdoc labels a "test harness". A sixth phantom, sitting in the capability subsystem
  specifically — which is exactly where a reader who believed the "per capability grant" sentence
  would have gone looking. §4.173 C.

**`docs/ENGINE-SPEC.md`'s untrackedness is itself a finding, and it is Ben's call.** Re-tracking
publishes ~49 KB of internal-audience prose, which is a publication decision, not an orchestration
one. The repo has re-tracked four docs for exactly this reason (`SECURITY-POSTURE`,
`INVARIANT-COVERAGE`, `HOST-FUNCTIONS`, `DSL-SPECIFICATION`), each because a HARD-RULE clause-(b)
destination must exist in fresh clones — and this section is a fifth instance, since it cites an
untracked file as its evidence and CI can therefore never gate the correction. Options and costs
are laid out at §4.173 E. No default is assumed.

---

### 3.8 The question nobody asked — does the engine handle their VOLUME?

**Shape:** engine · **Freeze:** no · **Status:** OPEN — *and honestly labelled: this is an
unasked question, not a known problem.*

The museum evaluation cited **1.76 million real transaction rows** against a 248-table schema.
We answered all three of their asks and never asked whether the engine handles their **scale**.

A balance fold over 1.76M rows is O(n) with no aggregation, and even once §3.3's abelian fold
lands, the backfill-on-register path walks them once. redb keeps a 1 GiB page cache by default
so a hot scan is served from RAM rather than disk — but a cached O(n) scan is still O(n). The
engine's query and index story at that row count is **unexamined by us**.

This is the class of question that decides an adoption rather than shaping a design, and it
should be checked *before* we tell anyone the engine fits. Related and already recorded: §3.3
(no aggregation) is the mechanism that would make this a non-question.

---

## 4. Things that are fine, checked because we suspected otherwise

Recorded so nobody re-litigates them.

- **NaN / ±Inf rejection is correctly scoped.** `Value::to_canonical()` rejects them at
  hash time only. A `-inf` attention mask inside a wasm guest never touches it. Correct
  engineering for a content-addressed system — do not relax it.
- ~~**The iteration budget is reachable.** An automated pass claimed a stack-depth guard fires
  first. It does not; the walker is iterative and the budget check is inside the flat loop.~~
  **RETRACTED 2026-08-11 — this bullet was itself wrong, and it belongs in §3 not §4.** The
  automated pass was right. The budget check *is* inside the flat loop, but that does not make it
  the binding guard: `Evaluator::step` runs first on every iteration and pushes one frame per
  non-terminal step, popping only on `"terminal"`. So `max_stack_depth` is a step counter in
  disguise, and it is **64** while `DEFAULT_ITERATION_BUDGET` is **100 000**. Driving
  `Evaluator::run_with` over a two-node graph with a back-edge returns
  `EvalError::StackOverflow` at `stack.len() == 64`; the budget is never consulted. Through the
  `Engine` the same 64 binds from the other side — Inv-2 rejects any handler whose longest path
  exceeds `max_depth` (also 64), so a walk performs at most 64 steps and the flat budget cannot
  be reached at the default configuration. **What is true:** both guards exist, both are now
  operator-reachable (§3.7), and the flat budget becomes the operative bound only when
  `EngineBuilder::iteration_budget` lowers it or `max_depth` is raised past it. Treating the
  100 000 default as a live runtime backstop is the part to stop repeating.
- **Verifiable compute is considered, not built.** `BUSINESS-PLAN.md:145` names redundant
  execution, reputation and selective audit; `PLATFORM-DESIGN.md:331` names TEE attestation. No
  Rust implements any of it. The engine verifies *what* was executed (module bytes
  BLAKE3-rechecked) and has nothing for verifying *that* it was. Worth stating explicitly in
  the docs, since it is otherwise inferred wrongly in both directions.
- **Inv-8 is NOT inert — the worry is refuted.** The relative-addressing investigation flagged
  in passing that Inv-8 "may be inert: it analyses property key `handler` while `call.rs`
  dispatches on `target`," and never chased it. Chased 2026-08-11: the DSL compiler defines
  `KEY_CALL_HANDLER = "handler"` and `SubgraphBuilder::call_handler` sets `"handler"` on the
  node, so production CALL nodes carry the key Inv-8 reads. The invariant is live.
  **Narrow residual, stated as a question rather than a claim:** `call.rs` also dispatches on
  `target` + `call_op` (the typed-call form) — whether Inv-8's multiplicative budget covers
  *that* shape is unverified. One check, worth doing pre-tag, not worth asserting either way.
- **`Cid::from_blake3_digest` keeps its name — DECIDED, was Ben-deferred** (Surf-1 #1033,
  resolved 2026-08-11 at `e032ee05`). The LLM evaluation flagged it as the one genuinely live
  freeze question, and the code's own doc comment had deferred it. The constructor stamps
  `MULTIHASH_BLAKE3` + `BLAKE3_DIGEST_LEN` unconditionally, so a codec-neutral `from_digest`
  would *misdescribe* it. Baked-in #5's rule is that a frozen **slot** must never be named for
  an algorithm — which is why `benten_ivm::Strategy` kept the variant name `Reserved` rather
  than `ZSet` — but this is an **implementation** and names itself accurately. Agility arrives
  by ADDING `from_<algorithm>_digest` constructors, additive and post-tag-safe, never by
  vaguening this one into a name that lies. Byte layout unchanged either way.

---

## 5. Now-or-never

The only items the tag actually forecloses. Everything else in this document can be built after
v1-beta without penalty.

| Item | Why now | Status |
|---|---|---|
| **Binding-grammar reservation + disclosures** (§3.1) — resolved form of the relative-addressing question | The mechanism is walker-internal + a property convention = free forever. What the tag WOULD foreclose is the clean reservation: (a) freeze-record disclosure of the STREAM sigil grammar (an interpreted mini-language in a frozen wire position, currently named nowhere) + `context_binding_snapshots` intended semantics; (b) position-scoped registration-time `$`-reservation on op-node properties (+ ErrorCode mint); (c) the `InfiniteEmptyProducer` test-driven semantic at `engine_stream.rs:974`, decided deliberately; (d) the stale "first property" resolver comment. Design: `docs/future/binding-grammar.md` §5. | **RESOLVED — 4 small items, owed** |
| **`Value` inventory clause in the freeze record** (§3.2) | The freeze record does not name the property type of every Node and Edge. Freezing a type system without stating it is the rule-14 shape. Must land with its receiving row in the same commit. The same clause states the decode-bound POSTURE: `MAX_DECODE_BYTES` and the META #629 cluster are policy tripwires, test-pinned not wire-frozen, raisable later with re-derived DoS reasoning — the D-94 Argon2id lesson, so no adopter reads 16 MiB as a wire limit they may not touch. | owed |
| **`DSL-SPECIFICATION.md:60-67`** (§3.1) — normative claims with zero production writers | A FALSE-RECORD that freezes alongside the API. | owed |
| **`ENGINE-SPEC` config claim** (§3.7) | Same shape. | **DONE** — 5 checks + 2 more found (§3.7); §4.1 rewritten in the primary checkout 2026-08-11. **Applied to an untracked file**, so it rode no tracked patch and CI can never gate it; the replacement text is preserved verbatim at §3.7 as the only durable record. Re-tracking `ENGINE-SPEC` is a publication call for Ben (§4.173 E). |

**NOW-OR-NEVER, added 2026-08-11 — the `RotationAttestation` signing tuple** (§3.6). Its
canonical bytes are byte-pinned (two frozen-bytes-corpus entries, a REQUIRED check) and its type
is in the frozen `benten-id` public-api baseline, yet it is named in **neither**
`V1-FROZEN-INTERFACE.md` nor `V1-WIRE-INVENTORY.md`. Two things are owed pre-tag: **(a)** the
freeze-record disclosure (same shape as the `Value` clause above), and **(b)** a deliberate
decision on whether `(previous_did, next_did, superseded_at)` is *sufficient* — specifically
whether a rotation needs to distinguish deliberate custody-transfer from key-compromise, since
both are `SupersededBy` today and the distinction cannot be added post-tag. This was concealed
by a "Freeze: no" in §3.6 that nobody had verified.

**Owed VERIFICATION, added 2026-08-11 — is READ's addressing set frozen closed?** (§3.1)
The originating investigation asked: *"does relative addressing need anything in the frozen
surface — a way for an op node to express traverse-edge-E-from-my-anchor — or is it entirely
internal to the walker plus a property convention?"* That was answered **for bindings**
(walker-internal plus the position-scoped `$` reservation) and **never for traversal**. READ
accepts two addressing modes today (`cid`, `label`). A third is *probably* an additive new
accepted property value — but whether READ's accepted-property set is itself part of the frozen
surface **has not been checked**, and it is the one part of relative addressing the tag could
foreclose. If additive → nothing owed and say so. If frozen-closed → now-or-never.

Explicitly **not** now-or-never, despite being proposed or considered as such:
- `#[non_exhaustive]` on `Value` and peers — our own freeze contract (`V1-FROZEN-INTERFACE.md`
  §"Composing-phase escape valve") states that *adding* `#[non_exhaustive]` is additive and
  permitted in Composing.
- Raising `MAX_DECODE_BYTES` — test-pinned policy, not wire; and raising it is the wrong move
  regardless (§3.2b).
- `Bytes(Vec<u8>)` → cheap-clone payload — the one item where pre-tag genuinely IS the only
  window, evaluated on that basis and **deliberately declined** (§3.2b): bulk belongs
  out-of-line, so the zero-copy need never reaches `Value`.

---

## 6. Sources

- `/Users/benwork/Documents/versai/design/benten-engine-asks.md`, `benten-fit.md`,
  `leasing-research.md` — museum evaluation, 2026-08-08
- `/Users/benwork/Documents/rusty-gemma/BENTEN-HANDOVER.md`, `BENTEN-INTEGRATION.md` —
  LLM-inference evaluation, 2026-08-09
- Decision log `.addl/phase-4-meta/NIGHT-SHIFT-DECISIONS-2026-07-04.md` D-100 onward
- Recovered analysis: `.addl/phase-4-meta/_recovered-value-lenses/`

*Both external documents mark every claim verified-vs-reported and both retracted claims of
their own under measurement. Where they were wrong we have said so above; where they were right
and we had not noticed, that is recorded too.*
