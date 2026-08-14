# GPU compute and the engine — how far Benten should reach

**Status:** R0-INPUT design record. Lands beside `docs/future/engine-fit-and-gaps.md`.
**Receiving row:** `docs/future/phase-4-backlog.md` §4.173 (next free; §4.170–§4.172 are the
three fit-gaps records).
**Base:** `e032ee05`, read via `git show`. **Adopter evidence:**
`rusty-gemma/RESEARCH.md` (measured on the target machine) + `BENTEN-HANDOVER.md`.

**Evidence tags.** **[V]** = ORCH opened the artifact at `e032ee05` and read the text in this
pass. **[V-rg]** = measured in RESEARCH.md by the adopter, quote-checked here. **[V-panel]** =
a dimension agent opened it and the adversarial verifier independently confirmed it; ORCH did
not re-open. **[R]** = reasoned, not measured — treat as a claim with a stated mechanism.

---

## 1. The answer

**The engine should reach exactly one step further than "native sidecar, graph as control
plane" — and that step is a memory step, not a GPU step.** Of the four things Ben named, three
resolve outside the engine and the reasons are structural rather than budgetary: GPU kernels
cannot be admitted at runtime by anyone, ever, because nothing contains them and the one
containable GPU language is measured at half speed *with a known incoherent-output defect on
this exact chip family*; graph-mediated GPU dispatch is right at the granularity of the *turn*
and catastrophic at the granularity of the token, and the engine physically cannot do the latter
anyway because operation nodes have no runtime dataflow between them; and expert-aware LAN
routing is strictly worse than the expert-oblivious alternative, because MoE training adds a
load-balancing loss whose explicit purpose is to destroy the skew such a router would exploit.
The fourth — the RAM hot tier — is a real gap, but not the one it looks like: the engine already
has **three** RAM caches and every one of them copies on read, so a fourth cache fixes nothing.
What is missing is a way to hand out bytes the engine already holds **without copying them, at a
page-aligned address**. That is a return-type problem, it is additive, it is post-freeze, and it
is the whole of what GPU work asks the engine to grow. Everything else Benten already does — the
graph owning admission, capability, model resolution, session placement, tool-call control flow,
stop logic, streaming and provenance — is most of a serving stack, needs no new primitive, and
rides a seam (`engine:typed:` typed-CALL) that is **already freeze-safe today**.

**So: the boundary barely moves, and that is the useful result.** The single genuinely novel
thing Benten brings to this workload is not GPU anything — it is that *a CID is a cryptographic
claim about exactly which weights produced an output*, which is the adopter's own top-listed
strength **[V-panel: `BENTEN-HANDOVER.md` §5]**, and which the extent surface extends from
weights to the code path at nearly zero cost.

---

## 2. Per dimension — decisions Ben can act on

### 2.1 RAM hot tier — **YES, and it is not a hot tier**

**Decision: build a content-addressed, mmap-backed, page-aligned *extent* surface with a
borrowed accessor. Do not build a cache. Composing (post-freeze). Additive.**

The engine is not short of RAM caching. It has three, all copy-on-read **[V]**: redb's own
`LRUCache<Arc<[u8]>>` at a **1 GiB default we never configure** — `Builder::set_cache_size` is
called **nowhere in `crates/`**; `Engine.module_bytes: Mutex<BTreeMap<Cid, Vec<u8>>>`
(`engine.rs`), unbounded, no eviction; and `SnapshotBlobBackend`'s in-RAM
`BTreeMap<Cid, Vec<u8>>` (`snapshot_blob.rs`). The trait waist is the
problem, and it is one line — `KVBackend::get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, _>`
**[V, `crates/benten-graph/src/backend.rs`]**. Owned `Vec`. Every backend must copy, and
`RedbBackend::get` is literally `.value().to_vec()` — discarding an `AccessGuard` that redb
already hands us as a borrowed slice into a refcounted cached page.

Measured on this machine, 128 MiB payload, real `redb 4.1.0` **[V-panel, re-runnable at
`/tmp/gpu-graph/bench/`]**: borrow 0.001–0.006 ms; `to_vec` 6.4–16.5 ms; `blake3` 74–89 ms
serial; **full `get_node()` equivalent 129–137 ms ≈ 1.0 GB/s effective**. Against E2B's measured
1.4069 GB streamed per token **[V-rg]** that is ~0.71 tok/s — **~55× worse than the 25.7 ms/token
hardware floor**, and 60% of it is hashing that belongs at admission, not at read.

Two properties decide the shape, and only mmap has both:

| | redb page | mmap range |
|---|---|---|
| `ptr % 16384` (this box's `hw.pagesize`) | **33** — Metal-ineligible | **0** — eligible **[V-panel]** |
| reclaim under pressure | anonymous dirty → **swap** | file-backed clean → **dropped free** |

The second is the one an 8 GB machine lives or dies on, and RESEARCH.md names it independently:
mmap'd pages are dropped free while anonymous pages get compressed and swapped, *"the difference
between working and thrashing"* — with 57 MB free pages and 3.1/4.0 GB swap already in use at
session start **[V-rg]**.

**Honest correction to the panel:** this is largely `engine-fit-and-gaps.md` §3.2b rediscovered.
What is genuinely new is the pair above — **page-alignment** and **evictable-clean-vs-swap** —
neither of which the §3.2b design carries, and both of which are requirements, not preferences.
Claim those two; do not re-claim the two-tier shape.

**And a correction the verifier caught that materially reshapes the design:** "mmap the GGUF,
wrap each tensor's page-aligned range as its own `MTLBuffer`" **does not work** — GGUF tensor
offsets are 32-byte aligned, not 16384. Which is an argument *for* the extent store rather than
against it: if Benten owns the layout, it can page-align every extent start at admission. Raw
GGUF mmap cannot, and RESEARCH.md's own `Storage::Metal` already carries *"a buffer id plus
offset, never a single model-wide base pointer"* **[V-rg]** for exactly this reason.

**What the engine promises: page-aligned start, address stable for the guard's lifetime, a
length. No Metal in the engine.** Metal is the sidecar's business.

### 2.2 GPU kernels as engine-or-graph content — **SPLIT: yes / yes / permanently no**

**Decision (a): runtime compilation — YES, adopt it, and Benten supplies nothing.** The verified
fact is that `newLibraryWithSource:` compiled `simdgroup_matrix<float,8,8>` +
`simdgroup_multiply_accumulate` with no Metal Toolchain installed **[V-rg]**. But their own
conclusion on the strength of that probe is *"adopt candle's runtime-compile model —
`include_str!` the `.metal` → `new_library_with_source()`"* **[V-rg]**, and `include_str!` is a
**compile-time** macro. **Runtime compilation is orthogonal to foreign sourcing.** First-party
source, baked into the binary, compiled to a pipeline at run time, is precisely baked-in #19
with a JIT step at the end. No new category.

**Decision (b): kernel source as a content-addressed Node for local cache-keying and
provenance — YES, zero engine changes, lives in the sidecar.** A locally generated or autotuned
kernel keyed by `CID(source) × device-fingerprint → compiled pipeline`, and a provenance record
reading *"this token was produced by weight-CIDs W…, kernel-CIDs K…, on device D."* That extends
the adopter's stated strongest property from weights to code and costs nearly nothing. Ordinary
nodes in an ordinary zone — **not** `system:ModuleBytes` (privileged zone, O(N) decoding scan).
At ~60–80 KB of MSL it already fits through the shipped sync path (`MAX_MST_MESSAGE_BYTES` =
1 MiB), which is another way of saying kernels were never the artifact that needed new plumbing.

**Decision (c): peer-authored kernels compiled and executed — NO, permanently.** Three
independent kills, any one sufficient:

1. **The GPU is never reached.** Accepting peer MSL means feeding attacker-controlled text to
   Apple's closed-source shader frontend, in-process, synchronously, at full engine privilege —
   inside the process this phase has been putting `K_principal` and the DAK substrate into. A
   signature proves who wrote the string; it does not make the string safe to compile.
2. **Nothing contains it after dispatch.** No fuel, no deadline, no bounds checking in release,
   no host-call boundary at which to re-check a capability, and a hang's blast radius is the
   **whole device** — every process, including the compositor. A capability check gates *who may
   ask*, never *what runs*; the capability system has no representative on the GPU. This project
   already wrote the distinction down (baked-in #18: capabilities bind only a *cooperating*
   engine).
3. **The safe language is measured unusable here.** WebGPU is the only stack designed for hostile
   code — and it solves memory safety, still cannot guarantee termination, and measures **~50% of
   native Metal on prefill / ~67% on decode**, blocked on subgroup-matrix ops, with f16
   accumulation that *"caused some models to generate incoherent output on Apple M-series GPUs"*
   **[V-rg]**. On an 8 GB fanless M1 that already cannot hold its model, the trade is not "some
   speed for safety" — it is half the speed **plus a known correctness defect on the target
   hardware** for partial safety that still does not stop a hang.

Contrast the shipped wasm path, which survives a pinning mistake: `install_module`'s
`expected_cid` is deliberately not `Option`, and wasmtime enforces four non-opt-in axes for the
entire execution **[V-panel]**. Kernels-as-shareable-content is defense in one depth, asking the
pinning discipline to be perfect forever.

**The economics settle it independently even granting perfect safety.** Three hard kernels,
~60–80 KB of MSL, changing on the quantisation-format timescale — against 5.139 GB of weights
changing constantly. **≈1 : 70,000.** And the adopter's four pre-freeze asks are `Value`
`#[non_exhaustive]`, the `v128` silent-zero, the config surface, and `Cid::from_blake3_digest`
**[V-panel]**. Kernels appear nowhere. This demand is internally generated; the fit-and-gaps
discipline is *generalize, don't invent*.

*Cheap protective action:* pin structurally — with a mutation-proven guard — that MSL source is
reachable only from `include_str!` and never from a `Value::Text` off the wire. One edit,
otherwise invisible later.

### 2.3 Graph-native GPU dispatch — **YES at the turn, NO inside the token**

**Decision: the graph owns admission, capability, model/shard resolution, session placement,
tool-call control flow, per-token stop/branch, token streaming and provenance. It must not
mediate anything inside the decode step. No 13th primitive. Rides typed-CALL.**

The inner-loop question is moot before performance is reached: **there is no runtime dataflow
between operation nodes.** `run_inner(&mut self, subgraph, _input: Value, …)` — the handler
input is underscore-bound and never read **[V, `crates/benten-eval/src/evaluator.rs`]**; SANDBOX
calls `func.call(&mut store, &[], &mut results)` with an **empty argument slice** **[V]**; and
`EvalContext` — which defines the whole `$input`/`$result` binding vocabulary — appears **zero
times in the evaluator** **[V]**. Node *k*'s output cannot reach node *k+1*. Chaining 206 GEMVs
as 206 nodes is not a tradeoff to price; it does not typecheck. And the only node-to-node channel
is WRITE-then-READ, i.e. content-addressing an `f32[1536]` activation 206× per token — which
DAG-CBOR rejects outright anyway, since `Value::to_canonical` returns `FloatNan`/`FloatNonFinite`
**[V-panel]**, so a `-inf` attention mask cannot be a property value at all.

Kill the lazy version of this argument, because it is wrong: the evaluator's own per-step cost is
~200–500 ns **[R]** ≈ 0.16–0.40% of a 25.7 ms token — **3–7× cheaper than the sampler they
already accept at ~1%**. If cycles were the objection, graph-mediating the inner loop would be
affordable. The real cost is that a graph node that *decides* must observe the previous result,
and observing a GPU result CPU-side **ends the command buffer** — converting an intra-buffer
barrier into a full round trip against their central bet, *"pre-encode the entire 35-layer decode
step as one `MTLCommandBuffer` replayed per token"*, target <60 dispatches/token **[V-rg]**.
Per-layer mediation alone would consume 35 of that 60-dispatch budget in decisions.

**Freeze-safety is already preserved at zero cost.** `TypedCallOp` carries `#[non_exhaustive]`
**[V]**; `TYPED_CALL_PREFIX = "engine:typed:"` routes a CALL node to a closed enum registry whose
own doc says *"It is NOT a new primitive — the 12-primitive commitment holds"*; `PrimitiveHost`
is deliberately neither sealed nor non-exhaustive, documented as a #19 extension point where
defaulted methods are the preferred evolution path **[V-panel]**. **Nothing needs to happen
before `phase-4-meta-core-close` to keep this option.** If a pre-freeze GPU change is proposed on
option-preservation grounds, that argument is discharged.

Shape: three ops — `infer:probe`, `infer:open`, `infer:close` — with **one rule carrying it: the
handle is opaque and small, and no tensor ever crosses the `Value` boundary.** Weights, KV cache,
activations, command buffer and expert paging live behind the handle and are never engine state.
`infer:open` takes a **model CID, never kernel source** (§2.2c).

*The one real gap:* typed-CALL has **no wallclock axis**. `call.rs`'s "timeout check" compares
two author-supplied properties, `timeout_ms` vs `elapsed_ms`; the engine never measures elapsed
time, and Inv-8's budget is a step count **[V-panel]**. Harmless for ten microsecond-scale pure-
crypto ops; unacceptable for an op that is 25 ms at best and can hang the device. **Must land
before the first GPU op — additive, so post-freeze is fine.** Also: declare these ops
`is_deterministic() == false` (GPU reduction ordering + sampling RNG). Declaring `true` would be
exactly the FALSE-RECORD class rule 14 names.

### 2.4 LAN-distributed inference routing to whoever has the expert hot — **NO on experts, YES on sessions**

**Decision: layer-sharded pipeline placement, computed offline. Activations on the wire, never
weights. One routing decision per session, at admission, on prefix affinity and occupancy.
Never revisited.**

**The brief's premise contains a load-bearing error worth repairing: E4B is dense.**
`enable_moe_block` is TRUE only on 26B-A4B (128 experts, top-8) **[V-rg]**. The model that does
not fit (E4B, 5.139 GB > `maxBufferLength` 4.295 GB) *has no experts*, and the model with experts
is 14.424 GB — a harder capacity problem, not the same one. "Route to whichever machine has the
expert hot" is undefined for the model in the question's own framing.

**Never move weights: 594 : 1.** One expert slab is exactly 3,345,408 B **[V-rg, re-derived by
the panel to the byte from 704×2816 at Q4_0]**; one activation at `hidden_size` 2816 in f16 is
5,632 B. Fetching a slab from a peer beats reading it from your own SSD only above ~2.6 GB/s of
link goodput ≈ 21 Gb/s **[R]** — no consumer LAN reaches that; gigabit is ~23× worse than local
NVMe. And fetching consumes the receiving machine's **RAM**, the binding constraint, to conserve
**SSD bandwidth**, which is not. Weights cross the LAN exactly once, at install.

**Layer-sharding beats expert-sharding on expert-sharding's own arithmetic.** Expert-sharding's
genuine win — parallel expert reads, ~11 ms/token — is paid for by turning 3–4 hops into **30
sequential scatter-gather barriers**; 27 extra barriers cost 10.8 ms at a best-case 0.4 ms RTT
and 21.6 ms at a realistic 0.8 ms **[R]**. Break-even at best, loses everywhere realistic, and
collapses on Wi-Fi. **Barrier count is the design variable; bandwidth is not.**

**There is no signal to route on, by construction and by measurement.** MoE training adds a
load-balancing auxiliary loss whose explicit purpose is to equalize expert utilization —
"decoder-only MoE LLMs exhibit uniform expert activation and low access skewness" **[V-rg]**. And
their adversarial pass supersedes their own optimism: expert Jaccard sits **on chance**; the
decisive A/B removed 18.3% of synchronous reads at ~6.9 bytes of background read per byte saved
for a decode delta of **−2.9%**; and **heat-pinning loses to plain LRU** — on 2 of 3 topics,
trained and tested in the same session **[V-rg, verifier-narrowed]**. Heat-pinning *is* "keep the
popular experts hot." Even granting a signal, residency churns at **5–40 Hz** **[R]**, and our
measured damping win (2.7× fewer messages, better service) was obtained on a signal changing over
minutes. Damping a 40 Hz signal is not damping; it is deciding on a lie.

**And replicability inverts.** Aggregate practical residency across 4 × 8 GB M1 ≈ 14.4–16.0 GB;
26B-A4B is 14.42 GB. **Replication factor ≈ 1.0–1.1.** You cannot make a second copy of
anything, so replicability without spare capacity is operationally indistinguishable from
exclusivity — the conservation constraint returns at mesh scope, enforced at plan time instead of
commit time. The "some machines have some experts hot" regime needs aggregate RAM well above 1×
*and* real demand skew. **Both are false, independently.**

**Where our measured ROUTE (+23.3) actually transfers: to sessions.** ROUTE's preconditions —
atomic servable unit, conserved exclusive resource, slow-moving signal — all fail for experts
(240 sub-requests across 30 hard barriers; replicable; 5–40 Hz) and all hold for sessions (a
conversation is served start-to-finish; KV cache and prompt-prefix cache are conserved and
exclusive; changes at session boundaries). The adopter states **~86% of his wall-clock is prompt
processing** and that he already runs prefix-affinity routing **[V-panel]**. That is the highest-
value routing signal in the system and it has nothing to do with experts.

**This is the strongest part of the whole picture, and it needs no code mobility at all** — the
wire carries f16 activation vectors and, once at install, quantised weight bytes. Neither is
executable. The design sidesteps §2.2c rather than answering it, and that boundary should be
explicit: **if someone later proposes shipping MSL across the mesh, the whole unsolved problem
returns.**

---

## 3. The boundary line

**The graph decides *whether, where, and on whose authority*. The runtime decides *how*. The
physical marker is the Metal command buffer: everything inside it is native, everything that
decides whether to submit one is graph.**

The line sits there for a reason that is not taste. It is the last point at which latency is
amortized over a whole request rather than a token, **and** the last point at which the data
crossing it is bounded by construction and expressible in canonical DAG-CBOR. Both properties
fail together one step inward — the engine cannot pass a value between nodes (§2.3), and the
values in question are non-finite floats that `to_canonical` rejects.

| | Graph-native | Sidecar (#19, compiled in) | Out of scope, permanently |
|---|---|---|---|
| **What** | admission · capability · model + shard resolution · session placement · tool-call control flow · stop/branch · token streaming · provenance · **the shard plan as versioned content-addressed config** | kernels · tensors · KV cache · dispatch encoding · expert paging · everything inside the command buffer | peer-authored kernels · per-token graph mediation · tensors through `Value` · weights or activations over the MST/CRDT path |
| **Why fixed here** | one decision per request; data is small, finite, canonical | needs the previous result CPU-side; ends the command buffer | nothing contains a kernel; no node-to-node dataflow; DAG-CBOR rejects the values; MST is anti-entropy convergence, not RPC |

The shard plan belonging in the graph is not decoration: it is slow-changing, content-addressed,
signed, replicated config that changes on rare deliberate events and gets a version chain as its
audit trail for free (baked-in #8) — including the **declared single-machine fallback**, which is
where the graph earns its place, because a pipeline stage that dies kills the request and four
machines have ~4× the failure rate of one.

---

## 4. What the engine would need to gain

### Core (pre-tag) — **no functional change. Honesty items only.**

Every one is a rule-14 record-overstates-the-binary item that this dimension surfaced, and every
one is free. They belong in W-REC (task #47), not in a GPU wave.

| # | Item | Evidence |
|---|---|---|
| C1 | **`engine-fit-and-gaps.md` §3.2b: ANSWERED → DESIGNED.** ANSWERED means "the supporting pattern *ships today*" by that document's own ladder. It does not: `Scalar::BytesCid` parses and emits but **nothing validates a `Value` against it**; `BlobBackend::get` returns owned `Vec<u8>`; **`iroh-blobs` is in no crate's dependencies** (prose only, in 3 manifests); the only impl stores blobs *as Nodes* through DAG-CBOR in a privileged zone. Only the chunked AEAD holds. **Four dimensions of this panel built on that grade.** | **[V]** all four |
| C2 | **`V1-FROZEN-INTERFACE.md` §4.62 FALSE-RECORD.** It freezes `BlobBackend` and says the trait "carries `put_blob`/`get_blob`/`has_blob`." **None of those three exist anywhere in `crates/`.** The real trait is `get`/`put`/`is_persistent`/`delete`/`list_cids`. The record invents a presence check never built and omits the two methods carrying the GC story. | **[V]** both greps |
| C3 | **`SECURITY-POSTURE.md` verify-on-read overclaim.** Section head: *"verify-on-read at every Node-bytes surface"*; body: *"verify-on-read is unconditional."* Two shipped paths over `NODES_TABLE` do a bare `serde_ipld_dagcbor::from_slice` with no `load_verified`: `SnapshotHandle::get_node`, and `get_node_label_only` — **which is production-live and feeds the Inv-11 probe.** Per rule 15 this is code→doc for the *behaviour* (fix `get_node_label_only`; it feeds an invariant probe) and doc→code for the *disclosure*. | **[V]** both sites |
| C4 | **`CapabilityEnvelope` disclosure — the only tag-shaped item in the whole analysis. See §5.** | **[V]** |
| C5 | `HOST-FUNCTIONS.md` routes `kv:read` through "`KVBackend::get_blob`" — nonexistent, for a host fn that writes zero bytes. | **[V]** |
| C6 | **`kv:read` must fail closed.** It returns **`Ok(0)` — success — having written nothing to guest memory**, and charges 0 to the output budget (`write_n_bytes(0, …)`, comment: *"Wave-8b stub doesn't actually write"*). Same silent-wrong-answer class as the `v128` zero-encode in §3.5, on the same designated-for-ML-inference path, and **not in the ledger**. Ride the v128 fix. | **[V]** |
| C7 | `MAX_MODULE_BYTES_ZONE_SCAN` is documented as bounding "worst-case decode work." It bounds node **count** (100_000); with no per-node body cap on the write path, work is `100_000 × unbounded`. | **[V]** |
| C8 | `SandboxConfig::max_wasm_stack` is decoration — the real limit is hardcoded on the process singleton; the field reaches only an error message. Record phantom-confirmed under fit-gaps §3.7 (closes 1 of 5 owed checks). | **[V-panel]** |
| C9 | `call.rs` module doc claims G7 supplies `elapsed_ms` from the evaluator trace; the code reads a static `op.properties` value. | **[V-panel]** |

### Composing (pre-v1-beta, post-freeze) — all additive, none blocking

| # | Item | Note |
|---|---|---|
| P1 | **Extent store.** Content-addressed (`Cid::from_blake3_digest(blake3::hash(bytes))` — the constructor `RedbBlobBackend::put_sync` already uses, hence already an iroh-blobs `Hash`), mmap-backed, **page-aligned extent starts**, borrowed accessor `ExtentRef: Deref<Target=[u8]>` + `as_page_aligned_ptr()` + `range()`. Hash **once at admission** (0.67 s for 5.139 GB with `update_rayon`), not per read. | the wave |
| P2 | **Range verification via bao outboard** over the same BLAKE3 digest, so a range read verifies the range touched + O(log n), not 5 GB. If judged too much for a first cut, the honest fallback is verify-at-admission + verify-on-open **recorded as a named Compromise stating the weaker property** — never a silently-skipped `load_verified` (we already have two, C3). | see §7 open |
| P3 | **Wallclock axis on typed-CALL.** Before the first GPU op, not after. | §2.3 |
| P4 | Three `engine:typed:infer:*` ops; `is_deterministic() == false`; `validate_input` typed-rejects `Value::Bytes` above a few KiB so "no tensors in `Value`" is a guardrail rather than discipline. | §2.3 |
| P5 | **Wire `iroh-blobs`** for general out-of-line content. Nothing at runtime needs it; model *installation* is otherwise a 19-minute hand-rolled rsync with no integrity story. | see §6 Q1 |
| P6 | `EngineBuilder` knob for redb's cache size, with a target-aware default. **Do this regardless of GPU** — an unconfigured 1 GiB anonymous-memory LRU is 12.5% of an 8 GB box and is a defect on its own terms. | ~1 line |
| P7 | Value-level validation against `Scalar::BytesCid`. | closes C1's substance |

---

## 5. Now-or-never

**Exactly one, and the recommendation is restraint.**

`CapabilityEnvelope` is **four bare `pub` fields** — `runs_sandbox`, `holds_zones`,
`online_uptime`, `runs_atrium_peer` — with **no `#[non_exhaustive]`**, `Serialize`/`Deserialize`,
embedded by value in `DeviceAttestation` beside a 64-byte Ed25519 signature, and frozen in
`docs/public-api/benten-id.txt` **[V]**. Adding a `runs_gpu` dimension after the tag is a
**signed-wire break**.

**Do not add it.** Three reasons:

1. A signed `runs_gpu=false` would assert a ceiling that does not exist after dispatch (§2.2c).
   Minting it pre-emptively **manufactures a FALSE-RECORD in the freeze** — the exact defect
   class rule 14 singles out as worst on a freeze, and the shape 17 of 26 MAJORs already share.
2. The need it appears to serve is **scoring, not authority**. Residency is fast-changing
   telemetry; this is a parent-signed, replay-nonce'd, long-lived attestation. Wrong lifetime
   *and* wrong layer.
3. `runs_gpu: bool` cannot express what routing needs (family, VRAM, resident experts) and would
   be superseded anyway.

**Do instead (free):** one disclosure sentence in the freeze record stating that
`CapabilityEnvelope`'s dimension set is **closed at four**, and that new device dimensions arrive
as a **new attestation record type** rather than by widening this struct. That is C4.

**Considered and confirmed not-now-or-never:** `Value::Bytes(Vec<u8>)` → `Arc<[u8]>`. Pre-tag is
genuinely the only window, and §3.2b already declined it. **I concur, and this analysis
strengthens the decline** — with bulk out-of-line behind an extent CID, inline bytes stay small
and copying them is cheap; the zero-copy need lives at the extent tier, which is not `Value` and
is not frozen. Also confirmed additive: new trait methods (§4.62's additive-default mechanism,
already exercised twice by `delete`/`list_cids`), a new backend, iroh-blobs as a dependency, the
`blob_ref` property convention, chunk size and two-CID key layout (already frozen **and already
correct** at `IROH_BLOCK_SIZE` = 16 KiB).

**One soft now-or-never, adjacent and worth a decision:** the MST caps are enforced
**decode-side only** — `to_canonical_bytes` has no cap, so an oversized node writes successfully
and silently, then fails as a `HandshakeWireFormat` error arbitrarily later *on the receiving
peer's machine*. And the 1 MiB per-message cap **does not apply to messages arriving inside a
frame**, so the effective bound there is 4 MiB. Both constants are byte-pinned. Not breaking to
fix; after the tag the asymmetry becomes documented behaviour. **[V-panel]**

---

## 6. Open questions for Ben

**Q1 — Does Benten want to be in the model-distribution business?** P5 (wire `iroh-blobs`) is
real work whose only near-term consumer is this adopter, and nothing at inference time needs it.
It is, however, the difference between "content-addressed weight identity" being the adopter's
top-listed strength **[V-panel]** and it being a claim we cannot deliver on. My prediction: yes,
because the property is already half-built and the alternative is that §3.2b stays DESIGNED
indefinitely — but the cost is a dependency and a wave, and it is your call whether that belongs
in Composing's ratified scope or after it.

**Q2 — C1's downgrade touches a ledger row the whole panel built on. Round-#1 finding, or
pre-round fix-pass?** It is free either way and it is a rule-14 item on a freeze surface. My
prediction: fix-pass now, because leaving it means round #1 reviewers inherit the same bad grade
this panel did.

**Q3 — Determinism-by-CID across heterogeneous peers.** Content-addressing an inference *result*
across machines with different GPUs is **false** — RESEARCH.md's own tolerance ladder allows
`nmse` 5e-4 for `MUL_MAT` **[V-rg]**. Address the *request*, pin a peer, or document it. Mint a
Compromise now, or carry it as a post-tag item? My prediction: a one-paragraph disclosure now,
because it is a correctness claim in a freeze document and the fix is prose.

**Q4 — Is the adopter relationship a reason to pull P1 (the extent store) forward in Composing?**
It is the only engine work this entire investigation asks for, and it serves the general
bulk-data gap, not just GPU. My prediction: yes, and it should be framed as §3.2b's build rather
than as a GPU feature — which is also how it avoids importing GPU vocabulary into the engine.

---

## 7. Refutation ledger

**ORCH's own errors — both reached adopter-facing text.**

| | Claim | Correction |
|---|---|---|
| **O1** | `Subgraph::MAX_DECODE_BYTES` (16 MiB) is a per-node ceiling. Cited repeatedly. | **WRONG.** It bounds *subgraph* decode only. There is **no per-node body size bound on the native Rust write path** — `put_node` has no length check. Consequences: the "16 MiB node ceiling" argument for per-expert granularity does not hold as stated (the granularity conclusion survives on sync-unit and copy-cost grounds); and C7's scan bound is `100_000 × unbounded`. |
| **O2** | The blob facility is a general out-of-line tier. | **WRONG.** `RedbBlobBackend` stores blobs **as Nodes** with `blob_bytes` through DAG-CBOR, in a **privileged system zone**, and `get_sync` does an **O(N) scan decoding every node body in the zone**. It exists to close Compromise #17 for SANDBOX module bytes. 3,840 expert nodes × 3.345 MB behind an O(N) decoding scan is disqualifying by orders of magnitude. |

**Records in the repo that overstate the binary** — C1, C2, C3, C5, C7, C9 in §4, all **[V]** or
**[V-panel]**, all belonging to W-REC.

**Panel claims refuted or narrowed.**

| | Claim | Disposition |
|---|---|---|
| R1 | ram-tier §3 finding #2: a vacuous `read_bytes_since_reset()` test asserts a property the code lacks. | **REFUTED.** Already found, fixed, renamed and recorded at `phase-3-backlog` §7.21. The report cited the pre-rename name, which survives only in a stale doc comment it read instead of the test. Residue: a cite-drift MINOR in `crates/benten-graph/src/lib.rs`. |
| R2 | ram-tier §5.4: "wrap each tensor's page-aligned range as its own `MTLBuffer`." | **NARROWED — materially.** GGUF tensor offsets are 32 B aligned, not 16384. Works only if Benten owns the layout (which is an argument *for* P1) or via one buffer + offsets, which is what `Storage::Metal` already does. |
| R3 | ram-tier §6 Piece 2 is a new design. | **NARROWED.** It is §3.2b rediscovered. The genuinely new claims are page-alignment and evictable-clean-vs-swap. |
| R4 | distributed-inference: RESEARCH.md's 1.789-vs-1.578 GB sum is an unresolved discrepancy. | **REFUTED.** Already resolved in RESEARCH.md §1.4 row 23 — attention is 0.624 GB, not 0.840 **[V-rg]**. |
| R5 | distributed-inference: the 52 ms/token derivation "reproduces the independently-stated ~19 tok/s." | **REFUTED as corroboration.** It is the identical division. Not independent; do not present it as confirmation. |
| R6 | distributed-inference: the 2-machine E4B split. | **NARROWED.** It halves a 5.139 GB file of which 2.312 GB is the cold-gather PLE table, which is `MADV_RANDOM` and not resident. The fit conclusion survives; the arithmetic needs restating. |
| R7 | distributed-inference: "heat-pinning loses to LRU." | **NARROWED** to "on 2 of 3 topics" **[V-rg]**. Conclusion unaffected. |
| R8 | blob-tier: `consume_sync_replica_mst_diff` has zero call sites. | **CORRECTED.** It does not exist. The live Atrium path is Loro `export_update()` over `send_bytes` at `RECV_CAP` 4 MiB. |
| R9 | blob-tier: `Scalar::BytesCid` "means nothing — no code parses it." | **CORRECTED.** It *is* parsed and emitted (`vocab.rs`) **[V]**. What is missing is **value-level validation** — which is the real C1/P7 substance. |
| R10 | kernel-security: SANDBOX has "seven budget axes." | **NARROWED** to six enforced + one decorative (C8). Since the argument is *this is the bar SANDBOX set*, state the bar at its true height. |
| R11 | distributed-inference: "`iroh-blobs` is in no crate's `Cargo.toml`." | **UPHELD** after a false-positive scare: it appears in 3 manifests as **prose** (a workspace comment, a `description`, a section comment) and in **zero dependency entries** **[V]**. |
| R12 | The brief's own premise: an 8 GB M1 wants to run a model with experts. | **CORRECTED.** E4B is **dense**; the MoE model is 26B-A4B at 14.424 GB **[V-rg]**. The expert-routing question is undefined for the model that doesn't fit. |

---

## 8. Verifier finding triage — none dropped

**Accepted, ORCH-reverified in this pass [V]:** N1 §3.2b grade (all four supporting pieces
re-opened) · N2 verify-on-read holes (both sites read) · N4 `kv:read` returns `Ok(0)` · §4.62
false record · `Scalar::BytesCid` parsed-but-unvalidated · `iroh-blobs` prose-only ·
`CapabilityEnvelope` four bare fields · `TypedCallOp` `#[non_exhaustive]` · `_input`
underscore-bound · `EvalContext` absent from the evaluator · `KVBackend::get -> Vec<u8>` ·
`MAX_MODULE_BYTES_ZONE_SCAN` = node count · all three RAM caches + `set_cache_size` never called ·
SANDBOX empty-argument-slice call.

**Accepted on verifier evidence, not re-opened here [V-panel]:** N3 `max_wasm_stack` decoration
(→ C8) · N5 `call.rs` `elapsed_ms` doc (→ C9) · N6 `UcanBlobsHandler` zero production callers —
**cite it as validated logic against a seam, never as a working blob channel** · `PrimitiveHost`
unsealed-by-design · MST encode-side uncapped + in-frame 4 MiB asymmetry (→ §5) · zero GPU
references in the workspace (the 40 naive hits are substring matches inside `SigPubKey`) ·
`wait::evaluate(sg, ctx, _input)` as a fourth underscore-bound site.

**Accepted as refutations of the panel:** R1, R4, R5 (drop the finding / drop the corroboration
claim).

**Accepted as narrowings that change a design:** R2 (reshapes P1 — Benten must own the layout),
R3, R6, R7, R10.

**Accepted as corrections of fact:** R8, R9, R12.

**Rejected with reason:** none. Every verifier finding survived.

**Open — cannot be resolved from this repo, each load-bearing for something above:**

| | Open question | What it gates | How to close |
|---|---|---|---|
| O-a | Does `newBufferWithBytesNoCopy` accept an mmap pointer on that machine, with a real Metal call? | P1's last hop. If it fails, P1 still stands on evictability + no-per-read-hash, and the honest answer becomes *"yes, but the last hop costs one copy."* | one Objective-C probe on the M1, ~30 min |
| O-b | Does a bao outboard over our exact `blake3::hash` bytes verify to the same 32-byte digest we store in a `Cid`? **[R]**, not verified. | P2. If not, per-read verification falls back to the named Compromise. | one test |
| O-c | Is ≤4.0 GB per machine a real shard budget? **[R]** from the measured envelope, not measured. | every §2.4 capacity number. The developer's own machine sat at 3.10/4.0 GB swap with 57 MB free pages at session start **[V-rg]**. | boot quiesced, allocate, record `vm_stat` + RSS |
| O-d | Their M0 (`llama-bench`, 2 h, no code) and M4 stage-2 (one GPU capture, ~45 min). | §2.3's per-layer row is the **only** place the boundary could honestly move, and only to per-layer. | their measurement, not ours |

---

## 9. What would change these answers

- **A GPU execution environment with real bounds** — per-tenant partitioning with preemption and
  a watchdog that kills only the offender. Then the wasm analogy actually holds and §2.2c flips.
- **A validating language that can express the fast kernel** — if WGSL gains portable
  subgroup-matrix ops, the 50%/67% gap closes, **and** the Apple M-series incoherent-output defect
  is fixed. Re-measure; do not assume.
- **An MoE trained without a load-balancing auxiliary loss.** Then skew exists and §2.4's core
  premise is falsified. This is the one worth watching.
- **Concurrency ≫ 1**, or a sub-100 µs interconnect. Either flips §2.4's barrier arithmetic; the
  second is the only hardware change that makes expert-sharding win.
- **Adopter pull for peer-authored kernels** — as opposed to content-addressed *weights*, which
  they asked for in writing. Then §2.2's economics change and it is worth re-costing.
- **If the binding-grammar work lands real node-to-node dataflow** (§4.170 — and note
  `EvalContext` is a **fourth fragment** of that same Phase-1 E3 family, surfaced here), §2.3's
  structural objection dissolves into "expressible but wrong." The command-buffer argument stands
  alone, which is why the conclusion is robust to it.

---

# ADDENDUM — the subgraph IS the program (Ben, 2026-08-12)

**This addendum materially revises §"graph-native dispatch" above, and partially reopens the
kernel-sharing question the security dimension closed. It was written after the pass, out of a
reframe from Ben that the pass never considered.**

## 1. The pass analysed the wrong mechanism

Everything above evaluates **graph-mediated dispatch** — the graph deciding each step, at some
frequency, with the runtime executing between decisions. On that framing the conclusion is
correct and well-evidenced: yes at the turn, no inside the token, because a per-token decision
forces a GPU→CPU sync and the whole pre-encoded-command-buffer design exists to eliminate exactly
that stall.

Ben's framing is different and better: **hand the device the whole subgraph, and see the result of
the walk when it finishes.** The graph is not a driver. It is the **source**, and the pre-encoded
command buffer is its **compilation target**.

That is how every GPU framework already works — PyTorch graphs, XLA/HLO, MLIR, TVM. Build a
graph, lower it, run it. Under this framing **Benten's subgraph is the IR**, and the tension the
pass identified simply does not arise: there are no per-step decisions because there are no steps,
only one lowering.

**ORCH correction owed here.** I told Ben per-token graph logic was affordable at ~40 Hz and
therefore fine. That was wrong, and the reason I gave for it being fine was also the wrong reason.
It is not about whether 40 graph walks per second is fast — it is that each one forces a pipeline
stall. The pass was right and I was wrong; the IR reframe is what actually resolves it, not the
frequency argument.

## 2. It needs no new primitives — custom node types plus a lowering handler

Per Ben's own model of the engine: applications are composed from the primitives as handlers that
create and interpret **custom node and edge types**, with the interpretation rules living in the
handler. A GPU program is therefore custom node types (`matmul`, `rmsnorm`, `softmax`, …) plus a
handler that lowers them. **The nodes are data; the handler is the compiler; the command buffer is
the output.** Baked-in #1 is untouched.

And the compiled artifact **caches by the subgraph's CID** — free, because the subgraph is already
content-addressed. Same program, same CID, same command buffer.

## 3. This partially rescues the sharing story §"kernel containment" closed

That section's refusal stands **for arbitrary peer-authored MSL**, and the three kills are
unchanged: no containment after dispatch, a hang takes the whole device, and accepting peer MSL
feeds attacker-controlled text to a closed-source shader frontend in-process at full engine
privilege.

But a **peer-authored graph over a fixed op vocabulary that WE lower** is a materially different
risk. A hostile graph can only request bad-but-valid tensor ops, and shapes, total FLOPs and
memory are all **statically boundable at compile time, before anything reaches the device**. That
is a real containment boundary where MSL had none. It is not a licence to accept arbitrary
graphs — it is a reason the question is worth re-asking under the new framing rather than treated
as settled by the MSL answer.

## 4. Marking the region: intrinsic annotation + extrinsic edge + CALL as the boundary

Ben's question was whether the handler is marked as GPU-bound or attached to a GPU node by an
edge. **Both, and they are different kinds of fact:**

- **The annotation is INTRINSIC** — "this handler is lowerable to a device" is a property of the
  computation. It travels with the subgraph, it is part of its CID, and it means the same thing on
  an M1 and on a CUDA box.
- **The placement edge is EXTRINSIC** — "run it on *that* device" is a property of THIS deployment.
  It must live outside the subgraph, or the same handler acquires a different CID on every
  machine, destroying dedup and sharing.

**The region boundary is CALL.** A GPU region is a handler; calling it is the boundary; the result
returns through the ordinary return path. Ben's *"when the GPU logic is done it returns here and
connects to this graph logic"* is literally CALL returning — no new delimiting mechanism, no
annotating individual nodes and computing maximal connected regions.

**The marker must be VERIFIED AT REGISTRATION, not merely declared.** A GPU-marked handler
containing a node type the lowerer does not know is a typed reject at registration — the same
fail-closed shape as `E_SCHEMA_VOCAB_SCALAR_UNKNOWN`. This is not optional polish: this project
has found **four** declared-but-never-read fields in one week (`UptimePolicy`,
`SandboxConfig::max_wasm_stack`, `output_max_bytes`, and `CapabilityEnvelope`'s dimension set), and
an unverified device marker would be the fifth — failing at dispatch time on hardware that is
hard to debug, instead of at registration where the author is standing.

**Net new surface: exactly one piece.** The annotation (new, registration-verified). The placement
edge, CALL, and the extension mechanism (baked-in #19) all exist.

## 5. The principle this generalises into

> **Anything that is a property of the THING goes in the node. Anything that is a property of the
> DEPLOYMENT goes in an edge.** The node's CID must be stable across deployments, so any fact that
> differs per-install cannot live inside it.

This is the same split that separates **granularity** (intrinsic — how the weights are cut into
nodes) from **sharding** (extrinsic — which machine holds which CIDs), and it resolves both
questions with one rule. Worth promoting into `ARCHITECTURE.md` alongside the §2 clarifications in
`engine-fit-and-gaps.md`.

## 6. Node granularity, revised: per-tensor, chunked to uniform size

The expert-granular split (3,840 × 3.3 MB) was right for an MoE model — and the model that
actually needs distribution is **dense** (E4B; the MoE is 26B-A4B at 14.424 GB). So the split
changes:

- **Per-layer** (35 × ~95 MB) — too coarse for dedup, too large for scheduling.
- **Per-tensor** — the natural unit: it is what GGUF stores, and it is the boundary a fine-tune
  changes or does not, so dedup works there. **658 tensors, ~5 MB average.**
- The outlier is the embedding / `lm_head` at **285 MB**, so per-tensor alone is not uniform.

**Recommended: per-tensor, with oversized tensors chunked to a uniform ceiling (8–16 MB)** —
roughly 900–1,000 nodes, all under one bound. Uniformity is worth paying for: predictable
transfer scheduling, predictable memory, and a simple blob store. Chunk to a target size rather
than following natural boundaries exactly.
