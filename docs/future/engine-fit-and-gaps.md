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
cycle-detect), a cumulative step budget (`evaluator.rs:279`, a counter), and a frame-stack cap
(`lib.rs:909`). A dynamic BRANCH condition changes *which* forward edge is taken, never
*whether* it is forward. **A value cannot increase the number of edges.**

These sentences should stop being cited as a reason operands must be static. They do not say
that, and the property they do assert is unaffected.

*Exception, and it is real:* **Inv-8** computes a multiplicative budget at registration by
reading ITERATE `max` and resolving CALL's callee. Its own comment (`invariants/budget.rs:157`)
names the attack: a subgraph claiming a cheap callee then pointing at an expensive one at
runtime. That is a genuine constraint on any dynamic-operand design.

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
  at `run_inner` (`evaluator.rs:225`). A parameter threaded three layers and discarded is not a
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

**One pre-tag item — a disclosure, not a mechanism:** `benten_core::Value` is named nowhere in
the freeze record. Freezing a type system without stating it is the rule-14 shape. See §5.

#### 3.2b Bulk data — tensors, media, anything ≥ tens of KiB

**Shape:** engine, resolved · **Freeze:** no · **Status:** ANSWERED — out-of-line, never inline

The LLM project's second problem is not a type problem: `Bytes(Vec<u8>)` is owned, so every
read of an inlined 285 MB tensor is a copy, against a machine achieving ~26 GB/s. No scalar
interpretation touches that — it is a memory-access question.

**The answer is the two-tier shape every content-addressed system converged on** (git trees vs
blobs; IPFS DAG nodes vs chunked files; iroh docs vs blobs): small canonical inline properties,
and bulk behind a CID reference. Benten already has all the pieces — the `bytes-cid` scalar (a
`Value::Bytes` that *means* "a CID," one of the two shipped interpretations), `two_cid_map`
(plaintext-CID → stored-CID), chunked at-rest AEAD splitting at `IROH_BLOCK_SIZE` (16 KiB,
deliberately aligned with iroh's wire layer), and the §4.62 blob-store trait. **Identity
composes:** the node's CID covers the reference, and the reference is the hash of the content —
integrity over 285 MB without ever decoding 285 MB into a `Value`.

**The decode bound is the tripwire that enforces this, and it is policy, not wire.**
`Subgraph::MAX_DECODE_BYTES = 16 MiB` (`subgraph.rs:571`) and the META #629 bound cluster are
**test-pinned** (`canonical_bytes_v1_const_values_core.rs:87` asserts the literal), which makes
changing one a deliberate re-pin with re-derived DoS reasoning — not a wire break. Its job is to
make "you inlined a blob" fail fast rather than degrade slowly: it is the amount of allocation
an attacker-supplied byte string can force before rejection, and it is why the LLM project's
per-expert granularity (3,840 × ~3.3 MB) is right and tensor-granular (285 MB) is structurally
rejected. **Raising it was considered and declined** — it would hand an adversary bigger forced
allocations while making the copy problem it nominally serves *worse*, and a 285 MB node would
be a 285 MB sync unit fighting a 16 KiB-chunked transport.

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
is never populated (`primitive_host.rs:623-625`), so the policy sees a **scope**, not a
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

**Shape:** engine · **Freeze:** partly · **Status:** OPEN

Baked-in #16 designates SANDBOX as the escape hatch for "heavy math, ML inference, custom
transformers." At HEAD the guest is invoked with an empty argument slice
(`sandbox.rs`), and results return as little-endian scalars with **`v128` silently encoded as
sixteen zero bytes** (`sandbox.rs:1432-1436`, comment: *"current corpus doesn't use them; encode
as zero placeholder"*).

So a wasm SIMD kernel — the obvious way to write the heavy math we explicitly invited — returns
zeros with no error. **Silent wrong answers on the path we designated for ML inference.** The
fix (return a typed error) is additive and cheap.

Open question for freeze scope: is the host-fn table extensible by third parties post-v1
without a fork?

### 3.6 Custodial → self-sovereign identity handoff

**Shape:** engine · **Freeze:** no · **Status:** CARRIED — Fork B must-nails

An organisation mints and custodies a user DID; the person later installs their own engine and
the custodial key signs a rotation to their self-sovereign root. This is how anyone joins a
decentralised network from a hosted product.

Maps directly onto **Fork B**, ratified 2026-07-17: rotation-survival and multi-device deferred
to Composing with named must-nails. The museum's flow is a concrete driving use case for
exactly those must-nails, and the DID→principal-CID mapping question (D-102 Q3) is the same
seam.

### 3.7 Phantom config

**Shape:** engine · **Freeze:** the false claim freezes · **Status:** OPEN

`ENGINE-SPEC` asserts all numeric limits are configurable per capability grant. Five sites
reportedly have no production writer: `InvariantConfig`, `memory_bytes`, `max_wasm_stack`,
`max_stack_depth`, `RunOptions::budget`. **[unverified — 5 individual checks owed]**

This is the FALSE-RECORD class from the compromise-ledger audit: a documented guarantee with no
implementation, which the freeze makes permanent. Fix is code, doc, or both — but per CLAUDE.md
rule 15 the default is to fix the code.

---

## 4. Things that are fine, checked because we suspected otherwise

Recorded so nobody re-litigates them.

- **NaN / ±Inf rejection is correctly scoped.** `Value::to_canonical()` rejects them at
  hash time only. A `-inf` attention mask inside a wasm guest never touches it. Correct
  engineering for a content-addressed system — do not relax it.
- **The iteration budget is reachable.** An automated pass claimed a stack-depth guard fires
  first. It does not; the walker is iterative and the budget check is inside the flat loop.
- **Verifiable compute is considered, not built.** `BUSINESS-PLAN.md:145` names redundant
  execution, reputation and selective audit; `PLATFORM-DESIGN.md:331` names TEE attestation. No
  Rust implements any of it. The engine verifies *what* was executed (module bytes
  BLAKE3-rechecked) and has nothing for verifying *that* it was. Worth stating explicitly in
  the docs, since it is otherwise inferred wrongly in both directions.

---

## 5. Now-or-never

The only items the tag actually forecloses. Everything else in this document can be built after
v1-beta without penalty.

| Item | Why now | Status |
|---|---|---|
| **Binding-grammar reservation + disclosures** (§3.1) — resolved form of the relative-addressing question | The mechanism is walker-internal + a property convention = free forever. What the tag WOULD foreclose is the clean reservation: (a) freeze-record disclosure of the STREAM sigil grammar (an interpreted mini-language in a frozen wire position, currently named nowhere) + `context_binding_snapshots` intended semantics; (b) position-scoped registration-time `$`-reservation on op-node properties (+ ErrorCode mint); (c) the `InfiniteEmptyProducer` test-driven semantic at `engine_stream.rs:974`, decided deliberately; (d) the stale "first property" resolver comment. Design: `docs/future/binding-grammar.md` §5. | **RESOLVED — 4 small items, owed** |
| **`Value` inventory clause in the freeze record** (§3.2) | The freeze record does not name the property type of every Node and Edge. Freezing a type system without stating it is the rule-14 shape. Must land with its receiving row in the same commit. The same clause states the decode-bound POSTURE: `MAX_DECODE_BYTES` and the META #629 cluster are policy tripwires, test-pinned not wire-frozen, raisable later with re-derived DoS reasoning — the D-94 Argon2id lesson, so no adopter reads 16 MiB as a wire limit they may not touch. | owed |
| **`DSL-SPECIFICATION.md:60-67`** (§3.1) — normative claims with zero production writers | A FALSE-RECORD that freezes alongside the API. | owed |
| **`ENGINE-SPEC` config claim** (§3.7) | Same shape. | owed, verification first |

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
