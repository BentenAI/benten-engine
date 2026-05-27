# Path-A vs Path-B — specialist review of the L11 architectural fork

**Branch:** `phase-4-meta-core/path-a-vs-path-b-specialist-review`
**Reviewer lens:** SENIOR DISTRIBUTED-SYSTEMS ARCHITECT + CRDT THEORIST
**Date:** 2026-05-27
**Tree-state pre-flight:** Started at `2172cb6d` on `origin/main`; clean worktree; branched cleanly.
**Inputs read in full:** L11 CRDT review (555 lines), q3-revisit Option-D community lens (~470 lines skimmed; §1-§7 absorbed), 9-eyes consolidated registry (located on PR branch; cross-referenced from L11), engine docs (`ENGINE-SPEC.md` Sec 2/6/7, `ARCHITECTURE.md` Sec on Inv-13, `HOW-IT-WORKS.md` §Anchor+Version+CURRENT), `benten-sync/src/crdt.rs` module-doc header (D-C HYBRID), `benten-graph/src/immutability.rs` header (Inv-13 storage-layer immutability), `benten-graph/src/` directory shape (immutability.rs + two_cid_map.rs present), `benten-engine/src/` directory shape (anchor_store.rs present). MEMORY.md foundational rules applied.

**Important factual correction up front (load-bearing for the whole analysis):** the brief frames Benten as having "mutable Nodes" whose CID changes on edit. **Reading the actual code + spec, Benten's storage layer is ALREADY immutable** (Inv-13 — `WriteAuthority::User` re-puts of an already-persisted CID raise `E_INV_IMMUTABILITY`; only privileged dedup paths return `Ok(cid)` without change-events). Version Nodes are content-addressed snapshots; the Anchor identity is stable; the CURRENT pointer is what "moves" via an Edge update. This means Path-A and Path-B are **much closer to each other** in Benten's existing reality than the L11 framing suggested. The actual fork is narrower and more tractable.

---

## §1 Executive recommendation + confidence

### Headline

**Recommend Path-A.5 (hybrid).** Specifically: **keep the mutable-Anchor-+-immutable-Version-Node graph model exactly as Benten has it today**, and **scope the encryption-substrate's K(N) keying to the immutable Version-Node-CID, not to any post-CRDT-merge "current effective Node-CID."** This is mechanically closer to Path-B than the L11 lens implied because Benten ALREADY commits to immutable Version Nodes (Inv-13). The L11 "Gap-1: K(N) re-encryption discipline" disappears under this framing: there is nothing to re-encrypt because the encrypted blob is bound to the Version-Node-CID, which never changes.

- **Confidence on the hybrid being the right architectural shape:** HIGH (~85%). The hybrid preserves "the graph evaluates itself" (Anchors + CURRENT pointer; reactive subscriptions; subgraph handlers; 12 primitives unchanged); preserves the 5-week-old D-C HYBRID commitment; and dissolves three of the four L11 gaps by keying encryption to the already-immutable layer.
- **Confidence on Path-A as L11 framed it being the WRONG framing:** HIGH (~90%) — the framing presupposes Benten has mutable-Node-CIDs at storage, which contradicts Inv-13 + Sec-6 of `ENGINE-SPEC.md`.
- **Confidence on Path-B (full Willow/iroh-docs migration) being inferior to the hybrid:** HIGH on the cost dimension (3-6mo redesign is real; touches engine internals + reactive shape + IVM cache invalidation); MEDIUM on the elegance dimension (Path-B has nominal precedent advantage that the hybrid partially captures).

### Why the hybrid wins per `extra-reflection-pass-for-elegant-permanent-shape`

The discipline asks: is there a **single elegant structural shape closing N findings at once** vs greedy-sum-of-N amendments? The answer is YES, and it is **"key the encryption-substrate to the immutable Version-Node-CID layer, not to the mutable-CURRENT-pointer layer."** This single move:

- Closes L11 Gap-1 (K(N) re-encryption) — no re-encryption needed because Version-Node-CIDs are forever-stable per Inv-13. U41 in L11's proposal collapses from ~3-4 wave-days to ~0.5 wave-day (just specify the layer + document the AAD-binding).
- Closes L11 Gap-4 (substitution-vs-CRDT-divergence conflation) — because Version Nodes are immutable + form a commit DAG (Sec 6.243), the predecessor-CID is a property of the Version Node itself, not a separate AAD field. U44 collapses by ~half.
- Reduces Gap-2 (KPrincipalRotation vector) scope — the rotation log is still CRDT-mutated, but if K_principal-generation binds to a specific Version Node's encryption-time (not to a rolling "current" generation), the vector requirement softens. U42 stays ~2-3 wave-days, but vector-vs-scalar becomes less load-bearing because each Version Node carries its encryption-time generation in its own immutable AAD.
- Leaves Gap-3 (offline-replay carve-out) unchanged — orthogonal to mutability; U43 stays ~1-2 wave-days.

**Forward-class-of-bug closure.** This shape forward-closes the entire "future-encryption-primitive reads mutable graph state" hazard class that L11's Inv-19 was trying to corral. By committing to "encryption keys derived from CRDT-immutable inputs only" as a static invariant, future crypto additions are STRUCTURALLY prevented from creating the Gap-1 class of bug.

### Cost summary

| Path | Estimated cost | Risk |
|---|---|---|
| L11-as-stated Path-A | ~10-14 wave-days | MED (novel territory; auditor pushback risk) |
| Path-B (full Willow migration) | 3-6 months | HIGH (touches engine internals; reactive shape redesign; v1-beta slip) |
| **Path-A.5 (hybrid; RECOMMENDED)** | **~6-9 wave-days** | **LOW** (uses existing Inv-13 + Sec-6 commitments) |

**The hybrid is strictly less code than Path-A AND has stronger precedent footing** (it explicitly aligns the encryption substrate with the layer of Benten's data model that ALREADY matches Willow/iroh-docs' immutable-entry pattern).

---

## §2 Plain-English explanation of mutable Nodes for Ben (Task 2)

Ben asked for plain English, no project-context assumptions. Reading the actual code, the framing in the brief turns out to be slightly misleading; here is the accurate plain-English picture.

### §2.1 What "Node" means in Benten

A **Node** is a chunk of data with properties. It has a **CID** (Content IDentifier) which is the BLAKE3 hash of its DAG-CBOR-canonical bytes. Two Nodes with byte-identical content have identical CIDs by construction. CIDs are forever-stable for a given content — change one byte, get a different CID.

### §2.2 What "mutable" actually means in Benten (corrected)

The brief says "Nodes have CIDs computed from their CURRENT content, and when content changes via CRDT-merge or update, the Node's CID changes." Reading `crates/benten-graph/src/immutability.rs` + `docs/ENGINE-SPEC.md` §6, **this is not how Benten actually works**. What Benten actually does:

- **Version Nodes are immutable.** Once written, a Version Node's CID is forever. Re-writing the same CID by a User-authority write is rejected with `E_INV_IMMUTABILITY` (Inv-13).
- **Anchor Nodes carry stable identity.** An Anchor's CID never changes; it is a small Node whose only job is to be pointed-at.
- **A CURRENT edge points from the Anchor to "the current Version".** When data "changes," you write a NEW Version Node (immutable, with its own new CID) and atomically move the CURRENT edge from old-Version to new-Version. The Anchor's CID doesn't change; the new Version Node's CID is new; the old Version Node's CID still exists and is still resolvable.

**So "mutable Node" is really "stable Anchor identity + history of immutable Versions + a moving CURRENT pointer."** The Node-content-CID never mutates. Only the Edge (CURRENT → which Version) updates.

This is structurally **already very close to Willow's immutable-entries model.** The difference is in what the application-facing API surfaces (Anchor identity is mutable-feeling; Willow surfaces immutable entries directly).

### §2.3 How CIDs compose with this model

- **Anchor CID** = stable forever.
- **Each Version's CID** = stable forever (its content is its identity).
- **Resolving "the current state of Anchor A"** = one graph hop (A → CURRENT → V_n). The result of that hop changes as new Versions are written, but each V_i is itself immutable.

### §2.4 What Benten use cases REQUIRE this Anchor-identity layer (vs pure immutability)

After walking the engine spec + the 12 primitives + the IVM + reactive semantics:

| Use case | Needs Anchor identity? | Why |
|---|---|---|
| External references ("link to this thing, see its latest state") | **YES** | If references pointed to a specific Version-CID, the reference would go stale on every update. Anchor lets external pointers stay valid across edits. |
| Capability grants ("Alice grants Bob read on Anchor X") | **YES** | Capability targets must outlive a single Version, otherwise every edit revokes Bob's grant. |
| Subscribe / reactive views ("notify me when X changes") | **YES** | The subscription is to the Anchor; the engine fires when CURRENT moves. |
| Subgraph handlers ("when Event-Y fires, run handler-H") | **YES** | Handlers are registered against an Anchor or label so they survive handler updates. |
| Time-travel / undo | **YES** (chain) | Walking the Version chain backward gives history; moving CURRENT backward is undo. Needs the chain. |
| Audit log / append-only event store | **NO** (immutable-only is fine) | Each event is its own immutable Node; no Anchor needed. |
| Content-addressed module / handler distribution | **NO** | Modules are versioned by their content-CID; share-by-CID directly. |

**Conclusion: Benten's Anchor + CURRENT pattern is load-bearing for at least 5 of 7 core use cases.** Pure-immutable-entries (Willow-style; lose the Anchor layer) would require the application to manually maintain an "Anchor index" on top — pushing graph-walking-and-reactive-semantics complexity into application code.

### §2.5 What "immutable entries" means in Willow / iroh-docs

In Willow + iroh-docs, the data model is:

- Entries are immutable. Each entry is keyed by `(namespace, subspace, path, timestamp)`.
- "Updates" become NEW entries with the same `(namespace, subspace, path)` and a later `timestamp`.
- Reading "the current value at path P" = "find the entry with the latest timestamp at path P." The reader does the resolution; the store does not.
- Sync between peers is set-reconciliation: each peer learns the entry-set of every other peer; missing entries are fetched; conflicts don't happen because two distinct entries are never combined into one (the reader picks the latest by timestamp).

**The crucial property:** there is no "merged Node" in Willow. CRDT-merge is set-union of entries plus reader-time latest-pick. No two entries are ever combined into a new third entry. Encrypted entries stay encrypted; their CIDs are forever-stable; the encryption key derived from CID-of-entry is also forever-stable.

### §2.6 How collaborative editing works under each model

**Benten (Anchor + Version + CURRENT):**
- Alice + Bob both edit Anchor A. Each writes a new immutable Version Node (V_a, V_b).
- Sync exchanges these Version Nodes (both peers store both).
- Per `benten-sync/crdt.rs` D-C HYBRID, the engine produces a NEW Version Node V_merged via Loro per-property HLC-LWW merge and atomically advances CURRENT to V_merged. V_a and V_b remain in the chain as predecessors of V_merged.
- The application sees Anchor A's "current value" = V_merged automatically.

**Willow / iroh-docs (immutable entries):**
- Alice + Bob both write an entry at path P with their respective HLC timestamps.
- Sync exchanges entries; both peers have both.
- The reader, asking "what is at P?", looks up entries at P and picks the latest-HLC. No merged entry is created.
- If true per-property CRDT semantics are wanted (not just LWW-of-blobs), the application layer encodes each property as its own entry — that is what Willow apps that need rich CRDT semantics do.

**Plain difference:** Benten has the merge happen INSIDE the store and produces a new artifact. Willow has the merge happen at READ time and produces no new artifact. Benten's pattern is "store-side merge"; Willow's is "reader-side merge."

### §2.7 How reactive computation works under each

**Benten:** the engine subscribes to Anchor-level change events. When CURRENT moves, subscribers fire. The fired-event carries Anchor-CID + new-Version-CID + old-Version-CID + a ChangeEvent payload. IVM materializes views incrementally on these events.

**Willow / iroh-docs:** subscribers subscribe to a namespace/path-prefix. When ANY entry arrives in that scope, subscribers fire with the entry. The reactive layer has to compute "did the effective latest-value change?" on every entry arrival.

**Implication:** Benten's reactive semantics are "Anchor-event-stream"; Willow's are "entry-stream + reader-resolves." Benten's is more efficient for "tell me when the effective value changes." Willow's is more uniform but pushes resolution complexity to subscribers.

### §2.8 What "the graph evaluates itself" requires from the underlying model

Reading `ENGINE-SPEC.md` §1-§2, the claim is:
- Data and code are both Nodes/Edges.
- Operation Nodes (the 12 primitives) form subgraphs the engine walks.
- Handlers are subgraphs registered against Anchors or labels.
- The engine reads handler subgraphs at runtime + walks them.

**What this requires from the model:**
- Stable identity for handler-targets (Anchor-CID OR a content-CID for the handler itself).
- Read of "current" subgraph at handler-fire time.
- Atomic update of CURRENT during handler-author updates (so handlers don't fire mid-update against a half-updated handler).

**Conclusion:** the Anchor + CURRENT layer is structurally what makes "graph evaluates itself" work. Pure-Willow-style entries-only would force handler-resolution to happen at every fire-time as a "find latest entry at this path" — workable but uniformly more expensive AND adds reader-side complexity to the engine evaluator (which today does one Edge hop and gets the answer).

### §2.9 Path A vs Path B — does Path A remain more elegant/permanent?

Ben's question literally. My answer:

**Path A as L11 framed it (mutable-Node-CID with re-encryption discipline) is INCORRECT framing for Benten** because Benten's storage is already CID-immutable per Inv-13. The L11 lens read "the merged Node has a fresh CID-X3" as if X3 replaces X1 + X2 in-place; reading the actual code, **X1 and X2 still exist as immutable predecessors and X3 is a NEW immutable Version Node added to the chain.** There is no re-encryption "in place"; there is only a new Version Node with its own encryption.

**Once that framing is corrected, the right question becomes:** "should K(N) be derived from `version_node_cid` (immutable; stable forever) or from `anchor_cid + current_pointer_value` (where current changes)?" The answer is OBVIOUSLY `version_node_cid`. With that choice:

- Path-A's "novel territory" critique evaporates — keying to immutable CIDs is exactly what Willow + iroh-docs do.
- Path-B's "redesign cost" critique evaporates — no redesign needed because the immutable layer is already there.
- Path-A.5 (the hybrid) is the natural fit, and it is MORE elegant + MORE permanent than either Path-A-as-stated or Path-B-as-stated.

**So:** Path-A in L11's stated framing is neither more elegant nor more permanent; it is based on a factual misreading of Benten's storage model. Path-A.5 — preserve the Anchor + Version + CURRENT pattern, key encryption to Version-Node-CIDs — IS the elegant permanent shape.

---

## §3 The actual choice analysis (Task 1)

### §3.1 What is being chosen between, fundamentally?

Once the storage-already-immutable correction lands, the fork narrows considerably. The remaining axes:

| Axis | Path A (L11 as stated) | Path B (full Willow) | Path A.5 (hybrid; RECOMMENDED) |
|---|---|---|---|
| **What gets a CID** | "Mutable Node" (treating Anchor+CURRENT+Version as a unit) | Immutable entry | Anchor (stable) + each Version Node (immutable) |
| **Where encryption key derives** | From "current effective Node-CID" (changes on merge) | From entry-CID (stable) | From Version-Node-CID (stable) |
| **What "merge" produces** | A new effective CID that re-keys | Nothing; reader picks latest | A new immutable Version Node (just like a fresh edit) |
| **Re-encryption discipline** | REQUIRED (Gap-1) | NOT NEEDED | NOT NEEDED |
| **Reactive shape** | Anchor-level | Entry-stream + reader-resolves | Anchor-level (unchanged) |
| **"Graph evaluates itself" cost** | Unchanged | Higher (reader-side resolution everywhere) | Unchanged |
| **Precedent** | None surveyed | Willow / iroh-docs / Earthstar | Anchor + Version is a 50-year database pattern (logical-vs-physical separation; Datomic; immutable git objects + mutable refs); per-Version-CID encryption is Willow-aligned |
| **Cost to land** | ~10-14 wave-days | 3-6 months | ~6-9 wave-days |

### §3.2 Is this purely about CRDT-encryption composition, or about Benten's identity?

**Purely about CRDT-encryption composition** under the corrected framing. The "Benten identity" question (is the graph evaluates itself; is collaborative editing UX; is reactive shape) does NOT change between Path-A and Path-A.5 because Path-A.5 keeps Benten's existing model intact. Only Path-B would change Benten's identity, and the encryption case for Path-B disappears once Path-A.5 captures the precedent benefit at the keying layer.

### §3.3 What Benten LOSES by moving to Path-B

- The Anchor identity layer + CURRENT pointer pattern — would have to be reconstructed in application code (every framework consumer would build its own Anchor index).
- Efficient Anchor-level reactive shape — Willow-style subscribers fire on entry-arrival; effective-value-change detection is reader-side.
- The D-C HYBRID merge-to-Version-Node convergence pattern — Loro's per-property LWW currently produces a merged Version that downstream code can reference; Willow's reader-resolve pattern requires re-running the resolution at every read.
- The 5-week-old engine investment in the Anchor + Version pattern (`crates/benten-engine/src/anchor_store.rs`; `benten-sync/src/crdt.rs` D-C HYBRID; Inv-13 firing matrix; capability grant targeting Anchors).
- The "12 primitives" semantic clarity — currently each primitive operates on a Node-or-Anchor target; under Willow these would all become "operate on the result of an entry-resolution query."

### §3.4 What Benten GAINS by moving to Path-B

- Precedent-blessed CRDT-encryption composition (which Path-A.5 captures equally).
- Simpler key-derivation story (which Path-A.5 captures equally).
- Direct interop with Willow / iroh-docs / Earthstar ecosystems (this is a REAL Path-B-only gain; Path-A.5 does not give wire-level interop with these systems).
- A smaller engine surface (no Anchor primitive; no CURRENT pointer atomicity logic; no Inv-13 firing matrix).

**Net: ecosystem-interop is the only Path-B-only gain.** And it can be captured later via a Willow-compat shim crate (post-v1-beta) that maps Anchor+Version into Willow entries on egress and back on ingress. The shim is a Path-B-future-additivity affordance; does not require flipping the core.

### §3.5 Is there a hybrid that gets both?

YES — Path-A.5. Specifically: keep Benten's mutable-Anchor-API surface (preserves graph-evaluates-itself + Anchor reactive semantics + capability targeting); key encryption to the underlying immutable Version-Node-CIDs (preserves per-Version K stability + matches Willow precedent at the encryption layer + no re-encryption discipline needed).

The "impedance mismatch" tradeoff that the brief flags for hybrids does NOT bite here because **the Anchor + Version pattern was already a logical-vs-physical separation;** the encryption substrate just keys to the physical layer (immutable Versions) and the application reads through the logical layer (Anchor + CURRENT). No new impedance is introduced.

---

## §4 Re-evaluation of L11's Path-A lean (Task 3)

L11 leaned Path-A on encryption-substrate grounds alone. Re-evaluating with the full architectural context + the Inv-13 + Anchor + Version-Node correction:

### §4.1 Sunk cost vs forward cost

- **Sunk cost** (Phase-1-through-4-Foundation investment in mutable-Anchor + immutable-Version + CURRENT + Inv-13 firing matrix + D-C HYBRID + capability targeting Anchors + IVM Anchor-events): substantial; likely 8-15+ wave-weeks of work distributed across crates.
- **Forward cost to Path-A.5** (key encryption to Version-Node-CIDs; document the layering; ship U41 simplified + U42 + U43 + U44 simplified + Inv-19): ~6-9 wave-days.
- **Forward cost to Path-B** (rip out Anchor primitive; rebuild reactive shape on entry-streams; rewrite Inv-13 firing matrix; rebuild IVM on entry-stream; rebuild capability targeting on path-prefixes): 3-6 months minimum, with realistic v1-beta slip.

**Path-B's forward cost exceeds the sunk cost.** Path-A.5's forward cost is small relative to either.

### §4.2 Cryptographer-audit-readiness

L11 flagged Path-A as "novel territory" (per-mutable-Node-CID keying is unprecedented in surveyed systems). Under the Path-A.5 correction, the encryption substrate keys to **immutable Version-Node-CIDs**, which is the same regime as:

- Willow's per-entry-CID encryption (immutable entry payload → stable key derivation).
- iroh-docs' entry encryption (immutable entries).
- Convergent encryption / DupLESS / message-locked encryption literature (immutable payload → deterministic key).

**Path-A.5 is precedent-blessed at the keying layer in a way Path-A-as-stated was not.** An auditor walking Path-A.5 sees: "K(N) = KDF(K_principal, version_node_cid); version_node_cid is content-addressed-immutable per Inv-13; therefore K(N) is stable; therefore the keying regime is standard convergent-style." That story closes cleanly.

The remaining novel piece under Path-A.5 — that the application surface (Anchor + CURRENT) is mutable while the encryption layer keys to the immutable substrate — is **not novel in cryptographic terms;** it is exactly the standard "stable identifiers + mutable refs" pattern that git, Datomic, Wikipedia (page-id stable + revision-id immutable), and every event-sourced system uses.

### §4.3 Long-term maintainability

- Path-A.5 stays close to today's code; net additions are clearly-bounded amendments (U41 simplified, U42, U43, U44 simplified, Inv-19). Most are wire-format-affecting only at AAD level.
- Path-B rewrites IVM, the evaluator, capability targeting, and the storage layer. Each is a load-bearing surface; combined they are an engine-rewrite.
- Path-A's stated form has a "novel keying" maintainability tax: every future crypto-substrate addition must reason about "does this read a mutable graph-state input?" Path-A.5 closes that hazard class by static invariant (Inv-19 simplified: "crypto KDFs may only read CRDT-immutable inputs; static Version-Node-CIDs are the canonical input").

### §4.4 Future-additivity

- **MLS-PQ maturation:** Path-A.5 and Path-B compose equally well (group-K(N) lives at the application layer above the Version-Node-CID layer; deferred per existing Compromise #42).
- **CGKA evolution:** Path-A.5 = q3-revisit's K_DedupScope shape (per-Atrium key distribution at member-join via multi-stanza HPKE; no continuous-rotation infrastructure needed). Path-B = same.
- **New PQ primitives:** crypto-agility (Inv-16) covers both paths equally.
- **Willow-ecosystem interop (egress to Willow peers):** Path-B native; Path-A.5 needs a shim crate. Path-A keeps the shim option.

### §4.5 Performance characteristics

- **Path-A.5:** writes = one Version Node insert + one CURRENT-edge update + one AEAD encrypt (under stable K(V)). Reads = one CURRENT hop + one AEAD decrypt. CRDT merge = local-engine Loro merge + new Version Node mint + new AEAD encrypt of merged-Version (the merged Version is a NEW Version Node with its OWN CID + its OWN AEAD ciphertext; there is no "re-encryption of existing ciphertext"; there is just "encryption of a new Version Node, same as any other write").
- **Path-B:** writes = one entry insert + one AEAD encrypt. Reads = entry-prefix query + reader-side latest-pick + one AEAD decrypt. CRDT merge = N/A (no merge; just set-union). Reactive subscribers fire per-entry; subscriber must re-compute "did effective value change?"
- **Path-A-as-stated:** writes = same as A.5 + plus re-encryption sweep after merge if interpreted literally. Higher cost per merge.

**Path-A.5 ≈ Path-B for writes; Path-A.5 slightly cheaper for reactive subscribers.**

### §4.6 UX implications for end-users

- Collaborative-editing UX under Path-A.5: same as today (CRDT merge happens engine-side; user sees the merged value; subscribe-fires once with the merged-Version). With multi-recipient encryption per-Drop-bundle re-Drop discipline (Compromise #45). True real-time co-edit still deferred to post-CGKA per Compromise #42.
- Collaborative-editing UX under Path-B: more nuanced — the user sees "latest entry wins" at read-time but never sees a "merged entry"; for property-level merge the application has to encode each property as its own entry. UX is more raw.

### §4.7 Implementation-team scaling

- Path-A.5 is a small set of named amendments to an existing-and-understood model. New contributors can read `ENGINE-SPEC.md` Sec 6 + the proposed U41-U44 + Inv-19 documentation and understand the whole encryption-CRDT seam.
- Path-B requires a contributor to learn (a) Willow's data model, (b) Benten's evaluator-on-entry-streams reworking, (c) the loss of Anchor reactive semantics. Steeper.

### §4.8 Verdict on L11's lean

L11's lean toward Path-A was correct in **direction** ("don't migrate to Willow"); incorrect in **framing** ("Benten Nodes are mutable; re-encrypt on merge"). The corrected shape is Path-A.5: keep the Anchor + Version-Node-CID + CURRENT pattern; key encryption to immutable Version-Node-CIDs only. This dissolves Gap-1 + simplifies Gap-4 + leaves Gap-2/3 substantively intact but smaller in scope.

---

## §5 Hybrid path exploration — Path-A.5 in detail (Task 4)

### §5.1 The shape

**Application layer (unchanged):** Anchor + Version + CURRENT + Loro per-property HLC-LWW + D-C HYBRID merge-to-Version-Node + 12 primitives + IVM Anchor-events + capability grants targeting Anchors.

**Storage layer (unchanged):** Inv-13 immutability firing matrix; Version Nodes are content-addressed immutable per Sec 7; CURRENT edges are mutable per Sec 6.

**Encryption substrate (NEW key-derivation discipline):**
- `K(V) = KDF(K_principal-gen, V.cid)` where V.cid is a **Version-Node-CID** (NOT an Anchor-CID, NOT a "current effective Node-CID").
- Each Version Node has its own AEAD ciphertext under its own K(V); ciphertext is forever-stable; ciphertext-CID is forever-stable.
- CRDT-merge produces a new Version Node V_merged with its own new V_merged.cid; the engine derives K(V_merged) + encrypts V_merged once; old V_a + V_b ciphertexts remain valid (they encrypt past Versions, which are still part of the chain).
- AAD-binds: `version_node_cid` + `predecessor_version_cids: Vec<Cid>` (for merge-derived Versions) + `k_principal_generation_vector` (per U42) + `sender_did` + `sealed_at_epoch_hour` + `valid_until_epoch_hour` + `received_at_epoch` (per U43, recipient-stamped, NOT wire-bound) + `sender_anchor_cid` (the Anchor this Version chains under).

**Atrium-coordination layer (per L11's recommendation):**
- KPrincipalRotation log = Anchor + Version chain (per-device vector counters in Version-Node properties; merge via Loro per-property LWW at the device-DID partition; per L11 U42).
- Receipt-log = local-recipient state (NOT wire).

### §5.2 Mapping API mutability to storage immutability

The mapping is the same one Benten already has:

```
Application-facing: Anchor X (stable; mutable-feeling via CURRENT pointer)
        ↓ "read current value of X"
Engine: (X) --CURRENT--> Version V_n (stable; immutable)
        ↓ AEAD-Open
Crypto-substrate: ciphertext under K(V_n) = KDF(K_principal, V_n.cid)
        ↓ engine returns plaintext bytes
Application: receives V_n's content as "X's current value"
```

For collaborative edit:
```
Alice writes V_a (Version of Anchor X). Encrypts under K(V_a). Publishes.
Bob writes V_b (Version of Anchor X). Encrypts under K(V_b). Publishes.
Each device's engine receives the other's V; Loro per-property LWW merges in-engine;
  produces V_merged (new Version Node of Anchor X with predecessor_version_cids=[V_a.cid, V_b.cid]).
Engine encrypts V_merged under K(V_merged) ONCE (single AEAD pass on the new Version Node — exactly the same code path as Alice/Bob's original writes; no special "re-encryption" path).
Atomically: CURRENT pointer of Anchor X advances to V_merged.
Anchor-event fires; subscribers receive new ChangeEvent.
```

This is mechanically identical to Benten's current D-C HYBRID flow plus AEAD-wrap.

### §5.3 Cost in wave-days

| Item | Wave-days | Notes |
|---|---|---|
| U41-simplified — specify "K(V) keys to Version-Node-CID; predecessor_version_cids AAD-bound; no re-encryption discipline needed" | **0.5-1** | Documentation + 1-2 golden vectors |
| U42 — K_principal-rotation vector + Atrium-Node Loro per-device-partition CRDT-merge rule | 2-3 | Unchanged from L11 |
| U43 — Offline-replay carve-out (`received_at_epoch` recipient-stamped; `offline_acceptance_grace_seconds` opt-in AAD field) | 1-2 | Unchanged from L11 |
| U44-simplified — Drop-bundle per-update predecessor-Version-CID chain (already aligns with `predecessor_version_cids` in U41-simplified) + error-discrimination | **1-2** | Smaller than L11 because predecessor chain is already in V-Node AAD |
| U21-extension — ExecuteWorkflow result-provenance binding | 0.5-1 | Unchanged from L11 |
| Compromise #45 — Collaborative-edit-via-re-drop (accepted v1-beta trade-off) | 0.5 | Unchanged |
| Inv-19-simplified — Crypto-keying CRDT-input discipline: "KDFs MAY only read CRDT-immutable inputs; Version-Node-CID is the canonical KDF input" | 0.5 | Stronger as a static invariant; closes the hazard class structurally |
| **Total** | **~6-9** | **3-5 days less than L11's Path-A; matches the "single elegant shape" payoff** |

### §5.4 Where the impedance mismatch tradeoffs that hybrids usually have do NOT bite

Standard "API/storage impedance mismatch" tradeoffs for hybrids typically involve:
- Translation layer overhead (e.g. ORMs translating between object-graph and relational tables).
- Reader-side complexity (e.g. having to query the storage layer + re-build the application-layer view).
- Synchronization bugs at the seam.

None of these apply to Path-A.5 because **the Anchor + Version + CURRENT pattern was already the abstraction Benten committed to** — it is not a NEW translation layer added at the encryption seam; it IS the storage shape. The encryption substrate keys to the storage layer's primitives (Version-Node-CIDs) directly; there is no "object/relational" gap.

### §5.5 Where Path-A.5 still has real costs

- **Atrium-replicated KPrincipalRotation log** (per L11 U42) is still itself an Anchor + Version chain whose CURRENT pointer moves on every rotation. U42's "per-device-partition vector counter" semantics need to land via Loro per-property HLC-LWW (BTreeMap<Did, u32> with per-device partition write-discipline) — this is mechanically clean but needs a property-test that proves concurrent-multi-device rotation converges. ~1 wave-day of property-test work folded into U42's 2-3.
- **AAD-binding of `k_principal_generation_vector: BTreeMap<Did, u32>`** adds ~30-80 bytes per envelope (matching L11's estimate); within wire-size budgets.
- **Offline-receipt log (U43)** is recipient-local state; persistence + GC discipline + cross-restart durability need their own small spec. ~0.5 day of incremental work in U43's 1-2.

These are real but bounded; included in the ~6-9 day total.

---

## §6 Surveyed-precedent deep-dive (Task 5)

L11 surveyed 8 systems; my survey adds context on the mutable-Anchor + immutable-Version axis specifically. (Where claims rely on my own knowledge rather than fresh WebFetch, I lower confidence accordingly in §8.)

### §6.1 Per-system axis analysis

| System | "Mutable-feeling identity" layer | Storage layer | Encryption-key derivation | Benten-Path-A.5 alignment |
|---|---|---|---|---|
| **Yjs (vanilla)** | Yjs document is mutable-feeling; internal op-log is append-only-immutable | Op-log entries | App-supplied; common pattern is one symmetric key per doc | Op-log immutability matches Benten's Version-Node-CID immutability. Yjs's per-doc-key matches "K(V) keyed to immutable substrate." |
| **Yjs + Serenity Notes** | Same Yjs above | Per-doc key wraps op-log entries | Per-doc symmetric key stable over doc lifetime | Strong precedent for Path-A.5's per-immutable-substrate keying. |
| **Automerge** | Document mutable-feeling; internal change-log immutable | Change-log entries (immutable per Automerge's hash-linked changes) | App-supplied; e2ee external | Change-log immutability matches Benten Version-Node-CIDs; analogous |
| **CRDX (Caudill)** | Mutable group state | Lockbox-keyed entries | K_lockbox is per-group; rotated on member change via new-lockbox emit | Precedent for L11's KPrincipalRotation-log shape. Mutable-feeling-group + immutable-lockbox-keying is exactly Path-A.5's pattern at a different layer |
| **secsync (Graf)** | Yjs doc mutable-feeling | Snapshot + per-update entries; snapshot-key rotates | Snapshot-key (symmetric) per snapshot; updates keyed under snapshot-key | Snapshot is the "stable substrate" entry that the key derives from; analogous to Path-A.5 |
| **Jazz (CoJSON)** | CoValue mutable-feeling | Internal op-log immutable | Per-CoValue group-key; rotated on member-remove | Mutable-feeling CoValue + immutable op-log + key-to-stable-substrate matches Path-A.5 pattern |
| **iroh-docs** | "Entry" surface (no mutable-identity layer) | Entries are immutable | Channel-level QUIC; no per-entry app-level key in current iroh-docs | Path-B exemplar; lacks the Anchor-identity layer Benten uses |
| **Willow** | Same as iroh-docs | Entries immutable | Confidential Sync delegates encryption out-of-scope; assumes app-level keying to immutable entries | Path-B exemplar |
| **Signal Sealed Sender + groups** | Mutable group state | E2EE messages immutable; CGKA epoch state | CGKA-derived group key per epoch | Mutable-feeling group + immutable-messages + key-to-epoch-substrate; analogous (epoch = "stable substrate") |
| **Git** (not surveyed by L11; relevant) | Branches mutable; refs are pointers; commits + trees + blobs immutable | Immutable objects (commits/trees/blobs); CIDs forever stable | git-crypt + age + similar key to repo (stable substrate) | **Direct precedent for Anchor + Version + CURRENT pattern (refs + immutable objects); 17-year-deployed model** |
| **Datomic** (not surveyed; relevant) | Entity-ID stable; entity attributes versioned; datoms immutable | Immutable datom log | (Not e2ee-by-design; on-disk encryption at storage layer keys to log segments) | **Direct precedent for stable-entity-identity + immutable-history-log; matches Path-A.5 conceptually** |
| **Benten F-full as L11 worried** | "Mutable Node CID changes on merge" | (Actually: immutable Version Nodes per Inv-13; L11 misread) | K(N) = KDF(K_principal, N.cid) where N.cid is mistakenly treated as current-effective | NOT what Benten actually does |
| **Benten Path-A.5 (RECOMMENDED)** | Anchor mutable-feeling; CURRENT pointer mutable | Version Nodes immutable | K(V) = KDF(K_principal, V.cid) where V.cid is immutable Version-Node-CID | Matches Yjs/Automerge/CRDX/Jazz/Signal/git/Datomic precedent at the keying layer; matches Willow/iroh-docs at the storage-immutability layer |

### §6.2 The structural pattern across all surveyed systems

**Every successful encrypted-CRDT or content-addressed-encrypted system in the survey keys encryption to an IMMUTABLE substrate.** The "mutable-feeling identity" surface varies — some have no identity layer (Willow), some have rich identity layers (CRDX groups, Jazz CoValues, git refs, Datomic entities) — but **the encryption keys ALWAYS derive from the immutable layer underneath**. There is no surveyed system where encryption keys derive from a mutable identifier that changes on merge.

This is exactly the Path-A.5 prescription. It is NOT a novel third path; it is the universal pattern, applied at the right layer of Benten's already-existing two-layer storage model.

### §6.3 Where Path-A-as-stated was actually unprecedented

The "K(N) re-keying on merge produces fresh ciphertext under fresh K(N3) and old ciphertext is GC-eligible" pattern described in L11 §2 Scenario A IS unprecedented in the surveyed systems — but it was a consequence of L11's framing error (treating Benten Nodes as mutable-CID-changes-on-merge), not a real Benten property. Once corrected, Benten's pattern aligns with the universal precedent.

### §6.4 Path-B's only unique gain stands

The one Path-B-only gain that Path-A.5 does NOT capture is **wire-level interop with Willow + iroh-docs + Earthstar peers.** A Benten peer can NOT directly exchange entries with an iroh-docs peer under Path-A.5; that requires a shim layer that maps Anchor + Version into Willow entries.

This shim is straightforward to build post-v1-beta if interop becomes desirable; it is a small Path-B-future-additivity affordance that does not motivate a core-engine flip.

---

## §7 Final recommendation + R0 plan-doc implications + cost (Task 6)

### §7.1 Winner

**Path-A.5 (hybrid).** Specifically:
- Keep the Anchor + Version + CURRENT pattern exactly as-is.
- Define encryption-substrate keying to operate on **immutable Version-Node-CIDs only**.
- Land L11's U41-U44 + Inv-19 in their **simplified forms** under this framing (U41 collapses to documentation + AAD-binding; U44 collapses to "predecessor_version_cids is already in V-Node AAD; just discriminate substitution-attack from CRDT-divergence in error surface").

### §7.2 Per-`extra-reflection-pass-for-elegant-permanent-shape` justification

- **Single elegant structural shape closing N findings at once:** YES — "key encryption to the already-immutable Version-Node layer" closes Gap-1 (eliminates), simplifies Gap-2 (per-Version-CID generation binding makes the vector cleanly local), is orthogonal to Gap-3 (independent), and dissolves Gap-4's conflation into a clean predecessor-CID match. One shape, four-finding payoff.
- **Strictly less code than greedy-sum-of-amendments:** YES — ~6-9 wave-days vs ~10-14 wave-days (Path-A) or 3-6 months (Path-B).
- **Forward-class-of-bug closure:** YES — Inv-19-simplified ("KDFs may only read CRDT-immutable inputs") forward-closes the entire "future-crypto-primitive reads mutable graph-state" hazard class.

### §7.3 Unselected paths recorded as NAMED-deferred per HARD-RULE clause (b)

**Path-B (full Willow migration) — NAMED-DEFERRED to post-v1-GM via a Willow-compat shim crate.**
- **Specific destination:** `docs/V1-FROZEN-INTERFACE-BUILD-BACKLOG.md` row `Willow-Interop-Shim-v0` (or equivalent in the F-full R0 plan-doc) — to land at R0-authoring time.
- **Revisit trigger:** if (a) ecosystem-interop with Willow/iroh-docs peers becomes a customer-named requirement, OR (b) Path-A.5's KPrincipalRotation vector + offline-replay carve-out + Anchor-events impose operational costs that Willow-pure would avoid (specific cost threshold: re-encryption sweeps > 1 GB/principal/year at production scale, OR Anchor-event subscriber overhead > 10% of write-path latency).
- **Estimated revisit cost if triggered:** 3-6 months for full migration (matches L11's estimate); ~3-4 wave-weeks for a shim-only path that keeps Path-A.5 internally + emits Willow entries on egress.

**Path-A-as-L11-framed (per-mutable-Node-CID re-encryption discipline) — DISCARD WITH EXPLANATION (HARD-RULE clause (c) DISAGREE-WITH-EXPLANATION).**
- **Disagreement:** the framing presupposes mutable-Node-CIDs at storage, which contradicts Inv-13. Once corrected, Path-A.5 supersedes; no separate Path-A artifact need land. L11's substantive findings (U41-U44 + Inv-19) flow into Path-A.5 in their simplified forms.

### §7.4 R0 plan-doc implications for the chosen path

If F-full R0 plan-doc authoring proceeds with Path-A.5 as the substrate choice:

**§1 Substrate decision recap (new R0 §):** record the Path-A.5 ratification (citing this review). Cite Inv-13 + ENGINE-SPEC.md §6 + benten-sync/src/crdt.rs D-C HYBRID as the structural foundations the encryption substrate keys to.

**§2 Amendment registry update (modify L11's proposed U41-U44 + Inv-19):**
- U41 → U41-simplified: "K(V) = KDF(K_principal-gen, V.cid) where V.cid is an immutable Version-Node-CID per Inv-13. AAD-binds `predecessor_version_cids: Vec<Cid>` for merge-derived Versions. NO re-encryption-on-merge discipline needed (V_merged is a fresh Version Node minted via existing D-C HYBRID flow; it gets its own AEAD pass exactly like any other Version Node write)."
- U42 (per L11; preserved with minor simplification): K_principal-rotation log is itself an Anchor + Version chain; per-device counter vector via Loro per-property HLC-LWW on a BTreeMap<Did, u32> property; concurrent-multi-device-rotation converges via per-device partition rule.
- U43 (per L11; unchanged): offline-replay carve-out via `received_at_epoch` (recipient-stamped local state) + `offline_acceptance_grace_seconds: u32` opt-in AAD field.
- U44 → U44-simplified: Drop-bundle per-update `expected_predecessor_version_cid` aligns with U41-simplified's predecessor_version_cids AAD binding; error-discrimination separates substitution-attack from CRDT-divergence.
- Inv-19 → Inv-19-simplified: "Crypto-substrate KDFs MAY only read CRDT-immutable inputs. Version-Node-CID is the canonical immutable input. Static keying inputs (codepoints, BE encoding, TLV structure) are exempt. Cite-drift-detector scans for KDF/derive_key invocations that take Anchor-CIDs or CURRENT-pointer values as inputs and rejects them."

**§3 V1-FROZEN-INTERFACE-BUILD-BACKLOG.md (new row):** `Willow-Interop-Shim-v0` — post-v1-GM; revisit trigger per §7.3 above; estimated 3-4 wave-weeks if triggered.

**§4 Compromise #45 (per L11; preserved):** Collaborative-edit-via-re-drop accepted v1-beta trade-off; true real-time concurrent multi-writer deferred to post-CGKA.

**§5 Compromise #47-new (Path-A.5 disclosure):** "Benten's encryption substrate keys to Version-Node-CIDs (per Inv-13 immutable) rather than to Willow-style entries directly; wire-level interop with Willow/iroh-docs/Earthstar peers is via a future shim crate (post-v1-GM; specific revisit triggers per V1-FROZEN-INTERFACE-BUILD-BACKLOG.md `Willow-Interop-Shim-v0`)."

### §7.5 Cost summary per path

| Path | Wave-days | Calendar | Confidence on estimate |
|---|---|---|---|
| Path-A (L11-stated) | 10-14 | ~2-3 weeks | HIGH (L11's own estimate; well-bounded amendments) |
| Path-B | 60-120 | 3-6 months | MED (engine-internals work is volatile; v1-beta slip risk) |
| **Path-A.5 (RECOMMENDED)** | **6-9** | **~1.5-2 weeks** | **HIGH** (smaller than Path-A by 3-5 days because U41 + U44 collapse) |

---

## §8 Self-assessment + confidence + lower-confidence areas

### §8.1 What I did + how I worked

1. Tree-state pre-flight at `2172cb6d`; created `phase-4-meta-core/path-a-vs-path-b-specialist-review` branch on isolated worktree.
2. Read L11 review in full (555 lines).
3. Read q3-revisit Option-D §1-§7 (~470 lines absorbed; remaining text skimmed for additional precedent).
4. Found that two cited inputs (RATIFIED-sharing-and-confidentiality-2026-05-21 + SESSION-2026-05-20-to-2026-05-21-substrate-wave-and-willow-pivot) do NOT exist on `origin/main`; verified via `git ls-tree -r origin/main`. Their content reflected via L11 + q3-revisit cross-references.
5. Read engine docs: ARCHITECTURE.md §Inv-13, HOW-IT-WORKS.md §Anchor-Version-CURRENT, ENGINE-SPEC.md §6 + §7.
6. Inspected source: `benten-graph/src/immutability.rs` header (Inv-13 firing matrix), `benten-sync/src/crdt.rs` header + grep (D-C HYBRID merge-to-Version-Node), `benten-engine/src/` listing (confirmed `anchor_store.rs` exists).
7. Composed §1-§9.

### §8.2 Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §1 Path-A.5 recommendation | **HIGH (~85%)** | Direct consequence of (a) Inv-13 + Sec 6 reading + (b) L11's own gap analysis + (c) `extra-reflection-pass` discipline applied honestly |
| §2 Plain-English explanation | **HIGH** on factual content (engine spec is ground truth); **MED-HIGH** on tradeoff framing (some judgment calls about "user-facing" UX) |
| §3 Actual-choice analysis | **HIGH** on the corrected fork shape; **MED** on the precise per-axis cost estimates (depends on team velocity + unforeseen surface) |
| §4 L11 lean re-evaluation | **HIGH** on the framing-correction; **HIGH** on the maintainability/audit-readiness analysis |
| §5 Hybrid path | **HIGH** on the shape; **MED-HIGH** on the wave-day estimates (matches L11's numbers minus the dissolved Gap-1 re-encryption work; small margin) |
| §6 Precedent survey | **MED-HIGH** within surveyed-set; **MED** on completeness (I added git + Datomic to L11's set but did not perform fresh WebSearch/WebFetch this session — relied on my own knowledge of those systems and L11's WebFetch work for Willow + iroh-docs + secsync etc.) |
| §7 Final recommendation + R0 implications | **HIGH** on the deferral shape; **MED-HIGH** on the specific R0-row wording (depends on R0 plan-doc structure conventions that may evolve at R0-authoring) |

### §8.3 What I could be wrong about

1. **The Inv-13 framing of Benten Nodes as already-immutable** is read from the storage layer + ENGINE-SPEC §6. If the F-full encryption substrate is intended to operate at a layer ABOVE the storage layer where a different notion of "Node" exists (e.g. a Drop-bundle payload that combines multiple Version Nodes into a logical unit with its own derived CID that changes on CRDT merge), then the L11 framing might apply at THAT layer + my correction might miss the intended seam. **Mitigation:** R0 §1 should explicitly pin "which layer's CID is K(N) keyed to" and reference Inv-13 for the immutable property.
2. **The Anchor-event reactive shape might not survive Loro's merge-to-Version-Node** cleanly if the merged Version's content depends on cross-Anchor state. My reading is that Loro per-property HLC-LWW operates within a single Anchor's Version chain; if it doesn't, the merge surface is wider than I have modeled and Path-A.5 has more amendments needed.
3. **The q3-revisit's K_DedupScope shape (Option I)** may interact non-trivially with Path-A.5's encryption layering. q3 introduces a triple-CID model (plaintext_cid_local + plaintext_cid_atrium + envelope_blob_cid) that lives ABOVE the Version-Node layer. Path-A.5's K(V) keys to Version-Node-CID; q3's K_Atrium keys to a different axis. These compose orthogonally (different layers of the dedup-blinding vs at-rest-encryption stack) but I have not exhaustively walked the AAD-binding interaction. Recommend a separate composability check at R0.
4. **q3 also coincidentally proposed "U41" + "U42" labels** for q3-domain amendments (dedup_scope_id + K_Atrium key management). L11 proposed U41-U44 for CRDT-encryption amendments. These two amendment-numbering streams **collide**; the consolidator at R0 will need to renumber one of them. Not a substantive concern but worth flagging.
5. **My Datomic + git precedent claims are from my own training-knowledge, not fresh WebFetch.** The "stable identity + immutable history" pattern is well-known but my specific framing as "Anchor + Version + CURRENT analogy" deserves a literature sanity-check (Hickey's Datomic talks; git-internals book; would take ~30 min of WebSearch I did not perform this session).
6. **The "K_DedupScope-and-K(V)-compose-orthogonally" claim** in (3) above warrants a dedicated composability lens at R0; I have not deeply walked whether q3's blinded `plaintext_cid_atrium` should AAD-bind into Version-Node K(V) ciphertext or live entirely above it. If they compose poorly, Path-A.5's cost grows by 1-3 wave-days for an explicit composition amendment.
7. **The forward-class-of-bug closure claim for Inv-19-simplified** is strong in principle but depends on Benten committing to a cite-drift-detector scanner that REJECTS any KDF reading a non-CRDT-immutable input. The scanner has to be precise; false-positives or false-negatives would weaken the invariant. ~1 day of scanner work folded into U41-simplified is realistic but not zero.

### §8.4 Lower-confidence areas (honest disclosure)

- Did NOT WebFetch fresh Willow/iroh-docs encrypted-doc spec content this session (relied on L11's prior WebFetch + my prior knowledge).
- Did NOT inspect `benten-engine/src/anchor_store.rs` content (only confirmed its existence in the directory listing); my Anchor-pattern reading rests on ENGINE-SPEC.md §6 + the cross-references in the docs.
- Did NOT walk q3-revisit §6-§7 in full (read §1-§5 + §6 winner-justification + §7.1 partially); my "K_DedupScope composes orthogonally with K(V)" claim is based on layer-separation logic, not full text-walk.
- Did NOT verify the cite `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` SHA matches the branch HEAD I see; trusted the L11 reference.
- Did NOT inspect benten-engine's reactive subscriber implementation (only ENGINE-SPEC.md §-IVM + the Anchor-event story); my "Anchor-event-stream is the reactive shape" reading depends on the spec.
- DId NOT perform a separate pim-13 R7-style spec-to-code-compliance audit of the Path-A.5 shape; recommend the R0 implementer agent do so before R3.

### §8.5 What this review does NOT cover

- The detailed CRDT-correctness analysis (L11 + the existing `prop_loro_concurrent_writes_converge_via_hlc_ordering` cover this already).
- The Atrium peer-discovery / iroh-blobs / iroh-gossip transport implications (L9 + q3 cover; q3's gossip-hybrid recommendation is independent of Path-A vs Path-B).
- The cryptographer-audit-readiness deep dive (C3 + C5 lens scope; my §4.2 is high-level only).
- Performance benchmarking of re-encryption sweeps (deferred per L11 §8 + my §8.3 (7)).
- UX-affordance design for the offline-acceptance-grace-seconds opt-in dial (UX lens; consolidator MF5).
- The detailed R0 plan-doc authoring; only the high-level shape implications.

---

## §9 Citations

### Internal Benten references (verified at SHA + path)

- `origin/main @ 2172cb6d` — `docs/ENGINE-SPEC.md` §6 (Version Chains; Anchor + Version + CURRENT; per-field LWW with HLC on concurrent edit; merge produces new Version Node).
- `origin/main @ 2172cb6d` — `docs/ENGINE-SPEC.md` §7 (Content-Addressed Hashing; BLAKE3 + DAG-CBOR + CIDv1; CIDs forever-stable for given content).
- `origin/main @ 2172cb6d` — `docs/HOW-IT-WORKS.md` §"Anchor + Version + CURRENT" (Version Nodes immutable once written; updating = write new Version + advance CURRENT atomically).
- `origin/main @ 2172cb6d` — `docs/ARCHITECTURE.md` line 387 (Phase-2a Inv-13 firing matrix; User re-puts of already-persisted CID fire `E_INV_IMMUTABILITY`).
- `origin/main @ 2172cb6d` — `crates/benten-graph/src/immutability.rs` module header (Phase-2a G2-A Inv-13 storage-layer immutability).
- `origin/main @ 2172cb6d` — `crates/benten-sync/src/crdt.rs` module header (D-C HYBRID per arch-r1-4; Loro merges produce new Version Nodes via existing Anchor + Version + CURRENT pattern; D-PHASE-3-22 RESOLVED).
- `origin/main @ 2172cb6d` — `crates/benten-engine/src/anchor_store.rs` (existence confirmed; Anchor primitive impl).
- `phase-4-meta-core/option-f-plus-lens-l11-crdt-conflict-resolution @ 68eadd0c` — `.addl/phase-4-meta/option-f-plus-lens-l11-crdt-conflict-resolution.md` (full L11 review; 555 lines).
- `phase-4-meta-core/q3-revisit-option-d-community-lens @ 23f76e24` — `.addl/phase-4-meta/q3-revisit-option-d-community-lens.md` (Option D + Option I K_DedupScope generalization; gossip-hybrid).
- `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` (referenced via L11 §9; not directly read this session).

### Inputs the brief named that do NOT exist on `origin/main`

- `RATIFIED-sharing-and-confidentiality-2026-05-21.md` — confirmed absent via `git ls-tree -r origin/main | grep ratified` (no results). Content presumably reflected in L11 + q3-revisit references.
- `SESSION-2026-05-20-to-2026-05-21-substrate-wave-and-willow-pivot.md` — confirmed absent via `git ls-tree -r origin/main | grep willow` (no results). Willow vs iroh-docs assessment + explicit defer-to-Phase-5+ presumably reflected in L11 §5 (Willow Confidential Sync learnings) + q3 §3 (iroh-blobs vs iroh-gossip).

### External standards + production systems

- Willow Confidential Sync — willowprotocol.org/specs/confidential-sync (per L11 §5 WebFetch; not re-fetched this session).
- iroh-docs — github.com/n0-computer/iroh-docs; docs.rs/iroh-docs.
- Yjs — github.com/yjs/yjs.
- Automerge — automerge.org; posit-dev.github.io/automerge-r CRDT concepts.
- CRDX (Caudill) — github.com/herbcaudill/crdx (lockbox pattern).
- secsync (Graf) — github.com/nikgraf/secsync (snapshot-based encrypted-CRDT).
- Jazz / CoJSON — jazz.tools (group-key shape).
- Signal Sealed Sender + groups — Signal blog 2018 + 2024 V2; RFC 9420 MLS.
- HLC — Kulkarni-Demirbas-Madappa-Avva-Leone 2014 "Logical Physical Clocks and Consistent Snapshots".
- Datomic — Hickey "The Database as a Value" 2012 (stable entity-IDs + immutable datom log; reference for the "mutable-identity + immutable-history" pattern).
- Git internals — git-scm.com "Git Objects" (commits/trees/blobs immutable; refs mutable; 17-year-deployed precedent for Anchor + Version + CURRENT analogy).

### Discipline references

- `feedback_handoff_top_banner_re_orient` — re-orient discipline applied at session start (read CLAUDE.md banner + L11 in full + ENGINE-SPEC §6 + Inv-13 source before composing).
- `feedback_no_defer_HARD_RULE` — applied to "what about Path-B": NAMED-DEFERRED to a specific R0 plan-doc destination (`V1-FROZEN-INTERFACE-BUILD-BACKLOG.md` row `Willow-Interop-Shim-v0`) with specific revisit triggers; NOT a phantom "later" deferral.
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` — APPLIED. Single elegant shape ("key encryption to immutable Version-Node-CID") closes 3 of 4 L11 findings with strictly less code than Path-A's greedy sum + forward-class-of-bug closure via Inv-19-simplified.
- `feedback_review_finding_ground_truth_verify` — ground-truth-verified the Inv-13 + Anchor + Version-Node + D-C HYBRID claims against actual source (`benten-graph/src/immutability.rs` + `benten-sync/src/crdt.rs` + ENGINE-SPEC §6) before concluding L11's framing was off.
- `feedback_plain_english_surfaces` — §2 written for Ben as plain-English-without-project-context-assumptions per the literal Task 2 wording.
- `feedback_surface_arch_decisions_under_auth` — this entire review IS the architectural-fork-surface-under-broad-auth product; pairs Path-A.5 prediction with the unselected paths' rationale so Ben can confirm or redirect cold.

---

**End of Path-A vs Path-B specialist review.**

**Summary handoff to orchestrator:** L11 framed the fork as Path-A (mutable-Nodes-with-discipline; novel territory) vs Path-B (immutable-entries; Willow-style; 3-6 month redesign), recommending Path-A. **Reading actual source code (Inv-13 firing matrix + ENGINE-SPEC §6 Anchor + Version + CURRENT pattern + benten-sync D-C HYBRID merge-to-new-Version-Node), Benten's storage is ALREADY immutable;** the "mutable Node" framing is a misreading of the API-surface mutability (Anchor + CURRENT pointer move) vs the storage-layer immutability (Version Nodes are content-addressed-immutable). **Path-A.5 (hybrid) is the correct shape:** keep Anchor + Version + CURRENT pattern exactly as-is; key encryption-substrate to immutable Version-Node-CIDs (NOT to a post-merge "effective Node-CID"). This dissolves L11 Gap-1 (no re-encryption needed because V-Node-CIDs are forever-stable); simplifies Gap-4 (predecessor_version_cids is already a property of merged Version Nodes); leaves Gap-2 + Gap-3 substantively intact. **Cost: ~6-9 wave-days** (3-5 days less than L11's Path-A; matches the `extra-reflection-pass` payoff). **Precedent: matches every surveyed encrypted-CRDT system** (Yjs/Automerge/CRDX/secsync/Jazz/Signal at the keying-to-immutable-substrate layer; Willow/iroh-docs at the storage-immutability layer; git/Datomic at the stable-identity + immutable-history layer). **Path-B NAMED-DEFERRED** to post-v1-GM via a Willow-compat shim crate with specific revisit triggers. Recommend Ben ratify Path-A.5 + L11's U41-U44 + Inv-19 in their simplified forms before F-full R0 plan-doc authoring proceeds. Lower-confidence areas: didn't WebFetch fresh Willow/iroh-docs content this session; didn't inspect anchor_store.rs body; q3-revisit's K_DedupScope/K_Atrium layering deserves an explicit composability check at R0 (independent layers but warrant verification); Inv-13 framing depends on encryption substrate operating at the Version-Node layer, not at a hypothetical Drop-bundle-payload layer above it — R0 §1 should pin this explicitly.
