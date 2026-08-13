# Engine fit — the strengths pass

**What this is.** The companion to `engine-fit-and-gaps.md`. Both outside evaluations ended with a
section saying where Benten looked *strong* for their workload, and each of those sections carried
an open question its author could not answer from outside. We triaged the complaints and skipped
the compliments. That is backwards: someone telling you where your engine fits a real workload is
more actionable than most feature requests, because it is the only signal that tells you which of
your properties are load-bearing to somebody other than you.

**Written 2026-08-12 against `e032ee05`.** Three probes, one adversarial verification pass, and an
ORCH ground-truth pass over the load-bearing claims. Every claim below was opened in the source,
not inferred from a docstring. Measurements were taken on Apple M1 / 8 GB — deliberately the same
machine class as the LLM evaluation's stated dev target — with a throwaway BLAKE3 + DAG-CBOR
harness; they are floors, not end-to-end timings, and they are labelled as such.

> **The finding that outranks all three answers.** This pass was supposed to confirm strengths. It
> instead refuted a claim of ours — `engine-fit-and-gaps.md` §3.2b, stamped **ANSWERED**, asserts
> that bulk content lives out-of-line behind a CID reference and that Benten "already has all the
> pieces." **There is no out-of-line storage tier at `e032ee05`**, and the tripwire that row credits
> with enforcing the pattern does not fire on the case it names. That is a FALSE-RECORD in the
> CLAUDE.md rule-14 sense, on a row stamped at the top of the status ladder, in the document written
> to stop exactly this. It is a headline, not a footnote. Full accounting in §6.

---

## 1. The three answers

| # | Question | Answer |
|---|---|---|
| **Q1** | *LLM eval §5:* "IVM over prompt-prefix CIDs — structurally the same problem as a radix-tree prefix cache. I don't know whether value sizes make it practical." | **Yes, build it today — but Benten holds the 242-byte prefix-identity ladder and the KV bytes never enter a Node.** IVM is the wrong mechanism, and not for the reason anyone expected: prefix lookup is exact-match identity, so the aggregation gap is *irrelevant* here; what disqualifies IVM is that its public read surface cannot take a key. |
| **Q2** | *Museum eval:* "A membership verifies at the gate with the network down; reciprocity falls out of the substrate." | **Yes for the signature, no for the admission decision.** The credential really does verify with zero I/O and zero prior knowledge — the key is inside the issuer's name — and that is the half that is near-impossible to retrofit. The trust list, revocation, and holder binding are all theirs to build. Design on **short expiry** from day one, and do **not** call `verifyInTrustDomain`, which silently accepts expired credentials. |
| **Q3** | *LLM eval §6 Q6:* "Where can 14 GB live, and is there per-node MVCC/IVM/subscription overhead to opt out of?" | **Nowhere inside the engine — keep the bytes outside and let the graph hold a CID manifest, which works today and needs nothing new.** And there is no steady-state overhead to opt out of: an immutable node with no views registered costs its bytes and its index entries and nothing else. The real hazard is the opposite of overhead — **a view registered over an existing corpus never reads it, returns empty, and stays empty.** |

---

## 2. Evidence

### 2.1 Q1 — the prefix cache

**The key is free, measured.** A chained block node `{parent_cid, t00..t15}` encodes to **242
canonical DAG-CBOR bytes**: BLAKE3 0.361 µs + encode 1.568 µs = **1.93 µs per block**, so the
entire prefix-CID ladder for an 8K prompt derives in **0.99 ms** against a prefill costing tens of
seconds. Their "content-addressing gives you the key for free" claim is confirmed with numbers.
Independently re-measured by the verifier at 242 B / 2.07 µs. One limit: this comes from the
chained-node *encoding*, not from BLAKE3 streaming — every call site (`Node::cid`,
`Node::load_verified`, `Edge::cid` in `benten-core`) hashes complete canonical bytes one-shot, and
no incremental-finalize path is exposed. Chain the nodes.

**Longest-prefix-match is not a fold.** It decomposes into a chain of exact-match probes: derive
`block_cid[0..n]`, probe each until the first miss. This is the vLLM shape, arrived at for free.
The aggregation the workload *would* want is eviction accounting — resident bytes, LRU counters,
hit rate — which is node-local, hot, and not a shared fact, i.e. precisely the thing you would
never put in the engine. **So the `ivm-aggregation.md` gap is real and irrelevant to this use
case**, which inverts the premise their §5 was written under.

**What actually disqualifies IVM.** The index shape a prefix cache wants already ships —
`CapabilityGrantsView` is literally `by_entity: BTreeMap<Cid, BTreeSet<Cid>>`, structurally
`prefix_cid → {entry_cids}`. Three facts kill it anyway:

- `ViewResult` has three variants (`Cids`, `Current`, `Rules`) and none is keyed
  (`crates/benten-ivm/src/view.rs`).
- The user-registerable kernel ignores the query: `GenericKernel::read` underscore-binds `_query`
  and returns `ViewResult::Cids(self.entries…)` whole (`crates/benten-ivm/src/algorithm_b.rs`). Its
  only selector is a first-label match. A user view over a 100k-block cache allocates and returns
  ~3.2 MB on every probe.
- The keyed views are unreachable *with a key*. `ViewQuery::entity_cid` exists on both
  `benten_ivm::ViewQuery` and `benten_eval::ViewQuery` and has **zero production writers**.
  `Engine::read_view_with` builds a default query and populates only `label`. Sharper still, from
  the verification pass: `PrimitiveHost::read_view` *receives* a query carrying `entity_cid` and
  **discards it**, calling `Engine::read_view(view_id)` which takes no query at all. This is not a
  field nobody populates; it is a field the evaluator could populate that is structurally dropped at
  the engine boundary. **Fourth instance of the `engine-fit-and-gaps.md` §3.1 complete-mechanism-
  no-caller shape**, alongside `EvalContext`, `context_binding_snapshots`, and the frozen wire slot.

**Value sizes — the question they could not answer.** Reconstructing their own 360 MiB @ 8K fp16
figure for Gemma 4 26B-A4B (30 layers, 25 sliding at `head_dim` 256 / 8 KV heads, 5 global at 512 /
2 heads, window 1024): 204,800 B/token sliding capped at the window + 20,480 B/token global = 200 +
160 = **360 MiB ✓**. Per 16-token block that is **3.4375 MiB** full, **320 KiB** global-layers-only.

The sliding window is what decides practicality and it is a *model* fact, not a Benten fact: a live
decode caps the sliding term at 1024 tokens; a prefix cache resuming at arbitrary cut points cannot.
**Full-block caching for an 8K prefix costs 1.72 GiB — 4.8× the live cache — on a machine with a
5.727 GB working set and 3.17 GB already swapped. It does not fit.** Archiving only the 5 global
layers gives 160 MiB for 8K, is position-independent (therefore genuinely shareable across the
mesh), and is the only variant that fits.

**Verify-on-read is the cost that bites, and it should not be relaxed.** `RedbBackend::get_node` →
`Node::load_verified` BLAKE3s the entire stored body *before* decoding, deliberately, so that a
tamper cannot surface as a decode error. ORCH re-read that body: the hash-first ordering and its
rationale are both in the source. MEASURED: BLAKE3 1.46–1.75 GB/s ⇒ **0.179 ms per 320 KiB block
(91.5 ms per 8K prefix)** vs **2.006 ms per 3.4375 MiB block (1.03 s)**; DAG-CBOR decode is 0.123 ms
at 3.4 MiB, so verify is ~94% of the cost. Against a REASONED ~81 ms/block recompute (their 1.51
TFLOP/s, 3.822B active params) the small granularity wins by ~450×.

**The honest comparison is not cache-vs-recompute but Benten-mediated-cache vs in-process cache**,
where the in-process map costs ~0. The 91.5 ms buys identity, capability-gating and cross-node
shareability. At 320 KiB that trade is clearly worth it; at 3.4 MiB it is marginal and the memory
arithmetic rules it out first anyway.

**Second cost, write side.** `ChangeEvent` carries full Node content and `Subscriber` fans every
event to every registered view under one `Mutex`, each filtering internally on first label. A
3.4 MiB block write pushes 3.4 MiB through a global mutex past views that immediately discard it.
High-frequency cache fills serialize there.

**Direct answer to their §6 Q3: there is no node-body size ceiling.** `Subgraph::MAX_DECODE_BYTES`
(16 MiB) guards `Subgraph::load_verified` only. ORCH re-read `Node::load_verified` and
`put_node_with_context` — **no length guard on either**. `MAX_VALUE_DECODE_DEPTH = 64` is nesting
depth. Adopters are reading 16 MiB as a general node limit and it is not one.

### 2.2 Q2 — offline reciprocity

**The zero-I/O claim is true and stronger than they realize.** `Did::resolve_signing`
(`crates/benten-id/src/did.rs`) was read in the body, not trusted from its docstring: base58-decode,
dispatch on leading multicodec, key bytes come straight out of the identifier — `0xed01` → Ed25519,
`0x1211` → ML-DSA-65 ⊕ Ed25519 composite, `did:benten` → composite multikey with the trailing
36-byte key-set CID stripped ("NO key-set doc, NO I/O"). No resolver, no DID document, no
`.well-known`, no cache to warm. **The key is the name.** A gate laptop that has never heard of
Garden A can verify Garden A's signature. Reachable from Rust, from Node
(`bindings/napi/src/identity.rs`), and from a graph handler (`vc_verify` typed-CALL), with a
10,000-case malformed-input proptest on the untrusted-bytes entry. This is the differentiator, they
read it correctly, and it is the part that cannot be retrofitted.

**But a signature check is not an admission decision.** Anyone can mint a DID and sign themselves
"platinum tier" in a millisecond; it passes cleanly. `TrustDomain` exists — and **nothing builds,
distributes, updates, or revokes entries in it.** `new(Vec<Did>)` and `empty()` are the only
constructors. This is our own **Compromise #67** (TOFU first-contact) with "ASTC partner list"
substituted for "Alice." For a reciprocal network that is good news — an out-of-band trust root
already exists and museums already trust it — but the work is theirs.

**Revocation is worse than they expect.** UCAN revocation is real: durable `g14b:revoked:<payload_cid>`
markers keyed on the *signature-exclusive* CID so a malleable re-encoding cannot dodge them. VC
revocation is a `Mutex<HashSet<String>>` in RAM, empty every process start, and
**`verify_with_registry` — the only function that consults it — has no production caller**;
`RevocationRegistry` is not exported to Node at all. `g14b` appears nowhere in `benten-sync` or
`benten-graph`, so markers never leave the machine that wrote them. And `vc::verify` never consults
`RotationLog`, with the proving pin `#[ignore]`'d for a named un-shipped prerequisite. **Honest
failure window: unbounded, until the credential's own `exp`.**

**Which makes short expiry the mechanism that works** — and it is what physical reciprocal programs
already do. `verify_at` enforces `issuanceDate`/`expirationDate` correctly, with a regression pin
that exists because a prior version called bare `verify` and silently accepted expired credentials.

**`aud` does not do the job assigned to it.** `CredentialClaims` has no audience field — zero
occurrences of `aud`, `audience`, `nonce`, `challenge`, or `Presentation` in `vc.rs`. The real `aud`
machinery is on UCAN (`validate_chain_for_audience`, constant-time, genuinely enforced) and it
**binds the delegate, not the verifier**, so it structurally cannot prevent presentation at a
different gate — which is correct for reciprocity, and means it is not replay defence. The replay
that matters — the member screenshots the credential and texts it — is unaddressed: there is no
holder binding, no Verifiable Presentation, no challenge. `DeviceAttestation::issue_with_nonce` is a
pattern to copy, not a component to reuse.

**Four things that will bite concretely, all verified:**

1. **A credential carries exactly one claim.** `CredentialSubject { id, claim_name, claim_value }`,
   `claim_value: String`, and the struct is **not** `#[non_exhaustive]`. A membership needs tier,
   member-since, reciprocal category, household count, member number — today those concatenate into
   one signed string with no schema. **This is a wire shape and the tag freezes it.** See §4.
2. ~~**No single call does allow-list + expiry.**~~ **CLOSED 2026-08-12 — Ben ratified breaking the
   signature pre-tag.** The finding as written was correct: `verify_at` was the only entry point
   that checked expiry and it takes a single expected issuer, so it could not check a list;
   `verify_in_trust_domain` and `verify_with_registry` both composed the bare clock-free `verify`,
   so a gate calling `verifyInTrustDomain` accepted expired memberships and a gate that added
   revocation checking lost expiry checking. **`now` is now a REQUIRED parameter on all three
   composed entry points** (`verify_in_trust_domain`, `verify_with_registry`,
   `verify_bytes_in_trust_domain`), each composing `verify_at`; `verify` stays clock-free as the
   honest signature-and-issuer primitive. Chosen over an additive `_at` twin because the twin
   leaves the wrong function public and still cannot pair revocation with a clock — requiring the
   parameter makes the silent skip *unrepresentable* rather than documented. The napi mirror moved
   with it. Two mutation-proven arms in `crates/benten-id/tests/vc.rs` fail on revert and no others
   do; the frozen baseline was regenerated with CI's own invocation and the diff is exactly the
   three signature lines.
3. **The graph-level `vc_verify` requires the issuer up front** — `{credential, expected_issuer_did,
   now}`, no allow-list arm, no decode-then-verify. **A handler cannot express the reciprocal check
   today.**
4. **A VC is not a Node.** `benten_id_vc_issuance_receipt_persisted_as_graph_node` is `#[ignore]`'d
   with body `unreachable!()`. Storing credential bytes as a property at the app layer is probably
   what they want anyway; they should not expect entitlement-as-Node as a shipped identity feature.

**On our side:** `verify_in_trust_domain` calls `Did::from_string_for_test_fixture` on the
production path — the constructor whose own docstring says a production call is a review-flagged
regression — as does the graph-level dispatch and three napi sites. Not exploitable as far as either
pass could tell (allow-list first, malformed DID fails closed at `resolve_signing`), but it collapses
a distinguishable "your trust list contains a malformed DID" into an indistinguishable
`BadSignature` and defeats the naming convention.

### 2.3 Q3 — where 14 GB lives, and what it costs at rest

**Inline `Value::Bytes` is legal and wrong.** No Benten-side ceiling on Node put or get. What breaks,
in firing order:

1. **The property index inlines the full value into the redb key.** `property_index_key`
   (`crates/benten-graph/src/indexes.rs`) packs
   `u32_be(label.len) || label || u32_be(prop.len) || prop || value_bytes`. ORCH re-read the write
   loop in `redb_backend.rs`: it is `for label { for (prop_name, value) }`, so the value is inlined
   **once per label** — a 2-label node with a 3.3 MB property costs body + 2 × 3.3 MB of B-tree key.
   redb's 3 GiB cap means this fails as silent bloat, not as a rejection.
2. **Every read rehashes the whole body** (§2.1). No partial, range, mmap, or zero-copy surface.
3. **Sync refuses it at 1 MiB.** `MstDiffMessage.payload` *is* the Node's canonical bytes, and
   `from_canonical_bytes` rejects anything over `MAX_MST_MESSAGE_BYTES = 1 MiB` before decoding.
   Enforced **receiver-side**, with no sender-side guard — so an oversized node writes locally,
   looks replicated, and fails at the far peer as a typed wire-format rejection.
4. Snapshot import caps at 16 MiB. 5. Change events clone the full body when any subscriber matches.

**`system:ModuleBytes` is strictly worse.** It *is* inline storage (`put_sync` builds a Node with
`blob_bytes: Value::Bytes`) and inherits the prop-index duplication *because `blob_bytes` is itself
indexed*. Privileged zone; the only public door is `register_module_bytes`, SANDBOX-shaped and
wasm32-refusing. `get_sync` linear-scans the zone decoding every body per fetch, and
`rehydrate_module_bytes_from_zone` runs at engine open as O(N²) full-body decodes loading every blob
into an in-memory map. One free win worth naming: the property index already holds an exact
`blob_cid` entry, so `get_by_property` makes `get_sync` an O(log n) hit with no new machinery.

**iroh-blobs is not a dependency.** ORCH re-checked: `iroh-blobs` appears in `Cargo.toml` files only
inside comments and crate descriptions — there is no dependency line anywhere. `TwoCidStore`'s
backing is a `Mutex<BTreeMap<Cid, Vec<u8>>>`, self-documented as a stand-in. `two_cid_map` maps
plaintext-CID → **ciphertext**-CID for UCAN scope resolution; it stores nothing out of line.
`IROH_BLOCK_SIZE` / `WHOLE_CONTENT_AEAD_THRESHOLD` are per-chunk AEAD parameters.

**What works today, and needs nothing new:** keep the bytes in the filesystem or any object store
keyed by BLAKE3 CID, and let the graph hold one small manifest node per artifact
(`{expert_id, blob_cid, layer, shape, dtype}`). **Integrity is free** — `Cid::from_blake3_digest` is
exactly BLAKE3-of-the-bytes in a CIDv1 envelope, which is what `RedbBlobBackend::put_sync` itself
recomputes as a defence-in-depth check, so an external store keyed the same way is verifiable with
the engine's own primitive: *the CID is the checksum*. Capability gating, per-DID partitioning and
encryption all still apply to the manifest; sync works, because a manifest node is three orders of
magnitude under the 1 MiB cap. And the query path is real and in-graph: `get_by_label` /
`get_by_property` are public on `RedbBackend` and `ScopedView` **and are implemented on
`PrimitiveHost`**, so a cap-gated handler can resolve a manifest by label or by exact property.

**Overhead at rest: essentially none.** Three redb tables per ordinary node. No per-node version
bookkeeping (version chains are opt-in), no per-node subscription bookkeeping (subscribers live on
the backend), no Benten-owned MVCC. IVM is a `ChangeSubscriber` — no polling loop, no tick, no
round; zero writes ⇒ zero work, and with no subscribers registered the write path skips even
building the event. Per write the honest cost is O(#views) predicate evaluations, synchronous on the
committing thread. **The one genuine exception is the one this workload cares about:** index entries
are not cheap when values are large, because the prop index inlines the value. A per-label or
per-property "do not index this" opt-out would be real, and does not exist.

**And the hazard that is bigger than the question.** `create_view` and `register_user_view` write a
definition node and push a live view; **neither reads the corpus.** `AlgorithmBView::rebuild()` is a
documented no-op whose own comment defers event-replay rebuild to a later phase. So *"what does
registering a view over 1.76M rows cost?"* has no number — **that operation does not exist. The view
returns empty and stays empty until those rows are re-written.** For the museum this is a design
constraint on ingest order: register views before ingest, or the views are wrong.

---

## 3. What the engine would need to gain

Placement: **Core** = pre-tag (the freeze forecloses it) · **Composing** = pre-v1-beta, post-freeze,
additive.

| # | Change | Placement | Why there |
|---|---|---|---|
| 1 | **Keyed view reads** — populate `ViewQuery::entity_cid` at the engine boundary and stop discarding it in `PrimitiveHost::read_view`; add a keyed `ViewResult` variant. | **Composing** | `ViewQuery` and `ViewResult` are both `#[non_exhaustive]`; the field already exists. Purely additive. |
| 2 | **Backfill-on-register for IVM views.** | **Composing** | Additive. But note: this is currently promised *only* inside `ivm-aggregation.md`'s correctness contract. It is not an aggregation feature — it is a general IVM defect that affects every view shape, and it needs its own row. |
| 3 | **`verify_in_trust_domain` must check expiry** (and `verify_with_registry` likewise). | **Core — now or never** | Neither can check expiry without an injected clock, and neither takes one; the crate has no ambient clock by design. Fixing it is a signature change on a function already in the frozen public-API baseline. See §4. |
| 4 | **Widen `CredentialSubject` beyond one string claim.** | **Core — now or never** | Not `#[non_exhaustive]`; the struct is inside the signed canonical bytes. See §4. |
| 5 | **A trust-domain verify arm reachable from a graph handler** (allow-list + expiry, decode-then-verify). | **Composing** | New typed-CALL input shape = additive. |
| 6 | **Property-index opt-out for large values.** Candidate shape worth a design pass: index the *hash* of the value for values over a threshold, preserving exact-match lookup (hash the probe) while giving up an ordering nothing publicly consumes. **UNVERIFIED design candidate, not a decision.** | **Composing** | Local index format; no canonical-bytes change. |
| 7 | **`get_sync` via `get_by_property` instead of a zone scan**, and a bounded `rehydrate_module_bytes_from_zone`. | **Composing** | Internal; the index entry it needs already exists. |
| 8 | **A genuine out-of-line tier** — a store keyed on blob CID with a range-capable read, a read path that verifies per chunk rather than per body, and a sync policy for out-of-line bytes. | **Composing, or never for v1** | See the refutation of probe 3's own proposal in §6(j): the *reference* layer needs no new `Value` variant, so none of this is wire-frozen. |
| 9 | **`Did::from_string_for_test_fixture` off the three production paths.** | **do now** (small, internal) | Rule 12: no reason to defer it. |

---

## 4. Now-or-never

Candidates for the `engine-fit-and-gaps.md` §5 table. Two are new and sharp.

| Item | Why the tag forecloses it | My reading |
|---|---|---|
| **`verify_in_trust_domain` accepts expired credentials** | It is in `docs/public-api/benten-id.txt` and it *structurally cannot* check expiry — no `now` parameter, no ambient clock (the `E_UCAN_CLOCK_NOT_INJECTED` precedent says clocks are injected here on purpose). After the tag the signature is frozen, so the footgun is permanent and the only remedy left is an additive `_at` twin sitting next to a public function that silently does the wrong thing. Same for `verify_with_registry`. | Fix the code (rule 15). Preferred: collapse to one function taking `now`, since list + expiry is the only correct gate call; the napi mirror moves with it (§3.5g). |
| **`CredentialSubject` carries one string claim** | Not `#[non_exhaustive]`, and it lives inside the signed canonical bytes — adding a field is a wire break, not a minor bump. Freeze it and every adopter concatenates five fields into one unschema'd signed string forever. | Genuinely a fork. Cheapest defensible widening is `claim_value: Value` (nested map, typed, one change now) rather than a claim list. Ben's call. |
| **§3.2b's two false clauses** (§6 below) | A false claim that freezes alongside the API — the exact shape §5 already exists to catch. | Correct the disclosure; the architectural conclusion survives unchanged. |
| **"No node-body size ceiling," and the tripwire that actually fires is 1 MiB at sync** | Belongs in the `Value` inventory clause §5 already owes. Both evaluations touched it; one asked directly. | Fold into the owed clause; it changes that clause's content. |

**Explicitly not now-or-never**, despite looking like it: keyed view reads, backfill-on-register,
the blob tier, the prop-index opt-out. All additive. And per §6(j), **an out-of-line reference does
not need a new `Value` variant** — §3.2a already settled that the reference is an *interpretation*
(`bytes-cid`), which ships today.

---

## 5. Open questions for Ben

1. ~~**`verify_in_trust_domain`** — break the signature pre-tag, or freeze it and ship an additive
   `_at` twin later?~~ **ANSWERED 2026-08-12: break it, and Ben ratified.** The prediction ("yes;
   it is a correctness defect on a public gate API and this is the last window") held. Landed —
   see §3 item 2 above for the shape and the evidence.
2. **`CredentialSubject`** — widen to a typed `Value` pre-tag, or accept the concatenated-string
   shape as the permanent v1 wire? An adopter is about to build a membership on it either way.
3. **Do we want an out-of-line blob tier at all for v1**, or do we state "bulk lives outside the
   engine, the graph holds the manifest" as a *positive architectural position*? Both evaluations
   landed there independently, and it is the position §3.2b's conclusion already takes. If it is a
   position rather than a gap, the §4.62 `BlobBackend` lock-vs-split fork should be decided in that
   light.
4. **Does any tracked doc claim keyed view reads?** If so it is a FALSE-RECORD and freezes.
   Verification owed — carried as OPEN in §7.
5. **`backfill-on-register`** currently exists only as a clause inside the aggregation design
   record. Does it get promoted to a general IVM row with its own receiving destination?

---

## 6. Refutation ledger

Claims that did not survive. **Ours first.**

**(a) ORCH / `engine-fit-and-gaps.md` §3.2b: "Benten already has all the pieces." REFUTED.** All
four named pieces were checked. `bytes-cid` is a `Scalar` variant in
`benten-platform-foundation::schema_compiler::vocab` — one layer above `benten_core::Value`, which
has no link variant, and §3.2a of the same document already concedes that nothing validates an
instance `Value` against its declared `Scalar`. `two_cid_map` maps plaintext-CID → **ciphertext**-CID
inside a per-DID partition for scope resolution; it stores no content out of line. `IROH_BLOCK_SIZE`
is a per-chunk AEAD parameter. The §4.62 `BlobBackend` trait has exactly one impl, `RedbBlobBackend`,
which stores blobs as **inline Nodes in a privileged system zone behind an O(N) scan** — and §4.62 is
itself an unresolved lock-vs-split fork row. Four pieces, none of which is an out-of-line tier.

**(b) ORCH / §3.2b: "The decode bound is the tripwire that enforces this." REFUTED, and the
correction is load-bearing.** `MAX_DECODE_BYTES` is checked inside `Subgraph::load_verified` only;
`Node::load_verified`, `RedbBackend::get_node` and `put_node_with_context` have **no length guard**.
The named tripwire does not fire on the exact failure it is credited with preventing — an inlined
blob on a Node. The bound that *does* fire is `MAX_MST_MESSAGE_BYTES = 1 MiB`, receiver-side at sync,
**16× tighter and in a different subsystem**. Consequence: §3.2b's own recommended granularity —
"per-expert (3,840 × ~3.3 MB) is right" — is *also* structurally rejected, by sync, at 1 MiB. The
guidance was right for a reason the row did not state, and its numbers were wrong.

**(c) ORCH / §3.2b's status stamp: ANSWERED. REFUTED — the row is doing two jobs.** Under the
document's own vocabulary ANSWERED means the supporting pattern *ships today*. Split it:
**3.2b-i, the adopter pattern — ANSWERED**, and now with the mechanism named correctly (external
store keyed by BLAKE3 CID + small manifest nodes + `get_by_label`/`get_by_property` on
`PrimitiveHost`, all shipped); **3.2b-ii, the engine tier — OPEN**, four named things, none of which
exist. This is the same "one word doing two jobs" defect the status vocabulary was introduced to fix,
recurring one section below where it was introduced.

**(d) LLM eval, implicit premise of §5: the IVM aggregation gap blocks prefix caching. REFUTED** —
prefix lookup is exact-match identity. The blocker is keyed reads, an independent gap.

**(e) LLM eval: "content-addressing gives you the key for free." SURVIVED, measured** — 1.93 µs per
block, 0.99 ms for an 8K ladder.

**(f) Museum eval: "offline verification falls out of the substrate." HALF SURVIVED** — the
cryptographic half does, completely and unusually well; the operational half (trust list,
revocation, holder binding) does not exist.

**(g) Museum eval: `aud` prevents cross-Garden replay. REFUTED** — VCs have no `aud`; UCAN `aud`
binds the delegate, not the verifier. **Scope caveat, unresolved:** the cited section lives in
`versai/design/benten-fit.md`, which is outside this repo (the ledger's own §6 Sources gives it an
absolute path under `/Users/benwork/Documents/versai`), so the attribution of this claim to them is
unverified. The fact is independent of the attribution.

**(h) Museum eval: "an entitlement is a content-addressed Node." REFUTED** — a `Credential` is
hashable but is not persisted as a graph Node; the proving pin is `#[ignore]`'d with body
`unreachable!()`.

**(i) LLM eval §6 Q3: "is there a node-body size ceiling?" ANSWERED — no**, and the 16 MiB they were
reading is a `Subgraph` bound.

**(j) Probe 3's own recommendation: add a `Value::Link(Cid)` out-of-line value class. REFUTED by our
own §3.2a** — `Value` is `#[serde(untagged)]` and cannot grow at all; a variant needs either a
discriminant (re-encodes everything) or decode-sniffing (reinterprets legal data), both already
rejected. The reference layer must therefore be an *interpretation over `Value::Bytes`*, i.e.
`bytes-cid`, which already ships. **This is why the blob tier is not now-or-never**: what is missing
is the store and the indexer's behaviour, not a frozen type.

**(k) Probe 1's stated "largest uncertainty" (that `attention_k_eq_v` halving might invert §4).
REJECTED as framed** — halving changes the number (1.72 GiB → 860 MiB), not the recommendation:
global-layers-only is better under both branches, and 860 MiB is still tight against a 5.727 GB
working set with 3.17 GB already swapped. It remains an open input, not an open conclusion.

---

## 7. Verifier-finding triage

Every finding from the adversarial pass, dispositioned. None dropped.

| # | Finding | Disposition |
|---|---|---|
| V1 | `entity_cid` is not merely unpopulated — `PrimitiveHost::read_view` receives the query and discards it. | **ACCEPTED**, folded into §2.1; it strengthens the finding from "no writer" to "structurally dropped at the boundary." |
| V2 | The k=v halving does not threaten the Q1 recommendation. | **ACCEPTED** — §6(k). |
| V3 | `ViewResult` is `#[non_exhaustive]`; "exactly three variants" is true today but the enum declares itself open. | **ACCEPTED**, and load-bearing for placement: it is what makes item 1 in §3 Composing rather than Core. |
| V4 | Only one of five VC verify entry points checks expiry, and it is the one that cannot take a list; adding revocation loses expiry too. | **ACCEPTED**, ORCH-reverified in source (`verify_with_registry` and `verify_in_trust_domain` both tail-call bare `verify`). Promoted to a now-or-never row. |
| V5 | The prop index inlines the value **once per label**, so cost is body + (#labels × value). | **ACCEPTED**, reverified against the nested write loop. |
| V6 | The 1 MiB sync cap is receiver-side on decode with no sender-side guard. | **ACCEPTED**, and sharpened: this is *worse* than a local rejection — the write succeeds and looks replicated. |
| V7 | §3.2b is a FALSE-RECORD in two clauses, stamped ANSWERED. | **ACCEPTED** — headline, §6(a)(b)(c). |
| V8.1 | Owe: demote §3.2b and replace its evidence. | **ACCEPTED** — §6(c) gives the split. |
| V8.2 | Owe: "no node-body ceiling" into the §5 `Value` inventory clause. | **ACCEPTED** — §4. |
| V8.3 | Owe: keyed lookup is a second, independent IVM gap; `ivm-aggregation.md` reads as though the fold were the whole gap. | **ACCEPTED** — §3 item 1 + §5 Q5. |
| V8.4 | Owe: "a view over an existing corpus returns empty" is documented nowhere. | **ACCEPTED** — §2.3; both adopters need it before they design ingest. |
| V8.5 | Owe: `verify_in_trust_domain` accepts expired credentials. | **ACCEPTED** — §4, promoted above the others because it is freeze-forced. |
| V9 | Scope gap: `versai/design/benten-fit.md` is not in the tree. | **ACCEPTED with explanation** — it was never a repo file; the ledger's Sources names it at an absolute path outside the repo. The probe's worry is resolved; the attribution of the `aud` claim stays **OPEN** (§6(g)). |
| O1 | Does any tracked doc claim keyed view reads? | **OPEN** — verification owed; FALSE-RECORD if yes. |
| O2 | redb B-tree behaviour with multi-MB keys. | **OPEN** — established legal (well under 3 GiB), never measured. "Pathological" is inference. |
| O3 | Whether `attention_k_eq_v` halving applies to 26B-A4B. | **OPEN** — their config to check; changes the numbers, not the recommendation. |
| O4 | Whether any other corpus-materialization path exists outside `create_view` / `register_user_view` / `rebuild` / `materialize_view_with_gate`. | **OPEN** — a targeted grep was run and every non-test hit read, but a negative cannot be proven that way. |
| O5 | How much of `DeviceAttestation::issue_with_nonce` is liftable for holder binding. | **OPEN** — signature read, verify path not traced. |
| — | Probe 1's Q1 verdict, Probe 2's verdict, Probe 3's headline. | **ACCEPTED** as SOUND per the verification pass, with the corrections above applied. |

---

## 8. What was measured, and what was not

Measured on Apple M1 / 8 GB with a throwaway harness: canonical node size (242 B, reproduced
independently), per-block key cost, BLAKE3 throughput at 1/3/16 MiB, DAG-CBOR decode. **These are
the hash + encode floor.** The real read path adds redb transaction setup, an Inv-11 system-zone
probe, and a `CapabilityPolicy::check_read` — so 91.5 ms and 1.03 s are lower bounds. The
recompute comparison (~81 ms/block) is REASONED from the evaluation's own TFLOP/s figure, which its
author flags as optimistic by 1.5–3×; that direction favours the cache. Nothing in §2.3 was
executed at all: every complexity claim there is structural (loop nesting × what each iteration
decodes), and no wall-clock number is asserted. The wasm32 IndexedDB blob backend was not examined —
`benten-sync` is compile-rejected on wasm32 and neither workload is a browser thin client.

---

## 9. Sources

- The three probe records: `/tmp/strengths/prefix-cache.md`, `/tmp/strengths/reciprocity.md`,
  `/tmp/strengths/bulk-and-overhead.md`, plus the adversarial verification pass over all three.
- `docs/future/engine-fit-and-gaps.md` — the companion ledger this record corrects in two places.
- The two outside evaluations, per that ledger's §6 Sources.

*Both external documents mark every claim verified-vs-reported and both retracted claims of their
own under measurement. This pass retracted two of ours.*
