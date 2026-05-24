# V1-FROZEN-INTERFACE — Benten Platform v1-beta public surface contract

> **Planner-B draft (conservative-minimal-freeze angle).** Co-planner with
> Planner-A (architectural-purist). Orchestrator triages both drafts into
> the single working artifact for the iterate-to-convergence council.
>
> **Status:** DRAFT — pre-triage. Not yet ratified.
>
> **Branch:** `g-core-9/planner-b-conservative-minimal`
> **Base:** `origin/main` @ `ae7cd3d5` (post pre-FREEZE bundle #1342)
>
> **Authority root:** `.addl/phase-4-meta/00-implementation-plan.md` §1.A.FROZEN
> (15-item spec; item 15 has 10 sub-clauses a-j). Every section below
> maps to one of those items. The orchestrator-triaged + R6-council-converged
> version of this file is the FROZEN-INTERFACE CONTRACT that v1-beta locks
> and that v1-Composing builds against.
>
> **Governing discipline.** The conservative-minimal angle takes the
> following posture on every clause:
>
> 1. Lock **what's shipped and tested**; defer **what's named-but-not-yet-shipped**.
> 2. When the cost asymmetry is SemVer-breaking-if-added-later
>    (`#[non_exhaustive]` being the canonical case), lean DEFENSIVE —
>    the under-freeze cost there is non-recoverable post-`v1-GM`.
> 3. When the cost asymmetry is recoverable (over-freeze → Composing-time
>    HALT-AND-SURFACE-TO-BEN re-open), lean MINIMAL — better to under-freeze
>    + escape-valve a re-open than to over-freeze + paint into a corner.
> 4. Every excluded item carries a HARD RULE 12 disposition (a/b/c) — no
>    phantom deferrals.
>
> **Cite key:**
> - [RATIFIED-S&C] = `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
> - [RATIFIED-PQ] = `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md`
> - [RATIFIED-PREWORK] = `.addl/phase-4-meta/RATIFIED-prework-forks-2026-05-18.md`
> - [RATIFIED-CRYPTO] = `.addl/pq-research/RATIFIED-crypto-agility-2026-05-18.md`
> - [PLAN] = `.addl/phase-4-meta/00-implementation-plan.md` §1.A.FROZEN
> - [BAKED] = `CLAUDE.md` Architectural Decisions Baked In (items 1, 5, 15, 17, 18, 19)
>
> **Composing-phase re-open escape valve** (methodology-r1-5, [PLAN] line 147).
> Any Composing-time discovery that would require altering ANY frozen
> surface below is a **HALT-AND-SURFACE-TO-BEN event**, not an
> orchestrator-autonomous re-open. Structural backstop = `cargo-public-api`
> drift CI (item 9) + `#1204` JS/TS parity gate (item 10). A frozen-surface
> mutation fails CI before silent re-open is possible.

---

## 1. §4.43 visibility cluster — `Engine` un-attributed-read surface tighten

### Frozen surfaces (the MINIMUM lock)

The four `EngineGeneric` methods named in [PLAN] item 1 — at
`crates/benten-engine/src/engine_wait.rs:1027/1056/1115/1137` + the
sibling `get_node` at `crates/benten-engine/src/engine_crud.rs:139` —
land in the v1-beta public-API contract with **the following visibility
matrix:**

| Method | v1-beta visibility | Rationale |
|---|---|---|
| `read_node_as` | `pub` (CLAUDE.md #18 boundary; Class-B-β) | The DESIGNATED un-attributed-read surface for principal-routed reads. Locked at v1-beta as the load-bearing #18 contract. |
| `get_node` | **stays `pub`** (conservative; flagged) | Tightening to `pub(crate)` would force every napi consumer (`bindings/napi/src/lib.rs:337` exposes it as `Engine::getNode`) AND every existing integration test (`crates/benten-eval/tests/read_denial.rs:96/100`, `crates/benten-engine/tests/inv_11_*.rs:93/155/159`, `crates/benten-engine/tests/noauth_startup_log.rs:47`, etc.) to migrate. The migration cost is **not measured at this writing**; tightening unsafely is the worse failure mode. **BELONGS-NAMED-NOW** to a Phase-4-Meta-Composing G-COMP-1.1 follow-up (cited in [PLAN] §1.B as a v1-assessment-window candidate) once consumer migration is mapped. |
| `put_node` | `pub(crate)` (TIGHTEN; SHIPPED) | Single-call-site usage internally; no napi exposure. Tighten cost = 0. |
| `get_node_label_only` | `pub(crate)` (TIGHTEN; SHIPPED) | Engine-internal helper only. Tighten cost = 0. |
| `resolve_subgraph_cid_for_test` | **rename + drop `_for_test` + `pub(crate)`** | The `_for_test` suffix in a `pub` surface is a known footgun ([PLAN] item 1 explicit). Rename to `resolve_subgraph_cid_internal` + `pub(crate)`. |

**§4.69 `EngineCapsHandle` cap-mutation organizing principle: FROZEN
AS-SHIPPED.** All 9 cap-mutation methods stay on `EngineCapsHandle`
(`crates/benten-engine/src/engine_caps.rs:81..429`); `Engine::caps()`
(`engine.rs:1688`) is the sole accessor; ZERO cap-mutation methods on
`Engine`-direct. Per orchestrator §3.5n ground-truth ([PLAN] item 1
correction; `ed03729a` baseline) this organizing principle is ALREADY
SHIPPED — G-CORE-9 adds ONE no-regression freeze-invariant (a
`cargo-public-api`-enforceable assert that no `Engine`-direct
cap-mutation method appears), NOT a Ben architectural fork.

### What "frozen" means here

- `pub(crate)` lockdowns are SemVer-checked at CI via `cargo-public-api`
  (item 9). A future revert to `pub` is a CI failure.
- The `resolve_subgraph_cid_for_test` → `resolve_subgraph_cid_internal`
  rename is atomic in this freeze wave; the old name is DELETED (no
  alias, no shim — per CLAUDE.md #5 + [PLAN] item 7).
- The `pub` retention on `get_node` is a SOFT freeze: the surface is
  preserved at v1-beta but explicitly flagged for v1-Composing
  reconsideration (Composing CAN tighten it without re-opening Core if
  the consumer migration completes within the v1-assessment-window;
  Composing CANNOT introduce new `Engine`-direct cap-mutation methods).
- §4.69 no-regression invariant pin: a new test
  `crates/benten-engine/tests/g_core_9_engine_no_direct_cap_mutation.rs`
  uses `cargo-public-api` output to assert ZERO cap-mutation methods on
  `Engine` (other than `caps()`).

### What's NOT frozen + WHY

- **`Engine::get_node` visibility tightening — BELONGS-NAMED-NOW** to a
  G-COMP-1.1 (or v1-assessment-window) follow-up row in
  `docs/future/phase-4-backlog.md` §4.43. Consumer migration cost
  must be measured before the tighten is safe.
- **Tightening other "private-by-intent" public methods on `Engine`** —
  the full sweep is OUT OF SCOPE for v1-beta; named in
  `docs/future/phase-4-backlog.md` (carry as a v1-Composing
  v1-assessment-window enhancement).

### Verification mechanism

- `cargo-public-api` diff against `docs/public-api/benten-engine.txt`
  (item 9) catches any post-freeze visibility regression.
- The §4.69 no-regression test pin fires if `Engine` direct
  cap-mutation methods regress.
- The `resolve_subgraph_cid_for_test` symbol's ABSENCE is
  cargo-public-api-verifiable (negative pin).

### Composing-phase escape valve

If Composing discovers `pub(crate)` is too tight on `put_node` /
`get_node_label_only` / `resolve_subgraph_cid_internal` (e.g. a new
napi binding wants direct access), HALT-AND-SURFACE-TO-BEN with options:
(a) re-widen to `pub`; (b) introduce a new typed pub surface that wraps
the internal method.

---

## 2. Class-B-β visibility — `read_node_as` / `read_node` boundary

### Frozen surfaces

- `pub fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, EngineError>`
  at `crates/benten-engine/src/engine_wait.rs:1115` — the DESIGNATED
  public surface for any read attributed to a non-trusted principal,
  per [BAKED] #18. **Signature frozen. Documentation frozen
  (the rustdoc explicitly identifies this as the Class-B-β boundary).**
- The `read_node` → `get_node` visibility decision is COUPLED to item 1
  above (`get_node` stays `pub` at v1-beta; tightening deferred to
  G-COMP-1.1).
- napi MUST NOT expose `read_node_as` directly (per `bindings/napi/INTERNALS.md:53`);
  napi exposes only `getNode` (the un-attributed path); the `_as` path
  flows through `call_as` (already frozen in item 1's `Engine::call_as`
  at `engine.rs:3376`).

### What "frozen" means here

The CLAUDE.md #18 plugin-trust contract requires this exact shape:
plugin authors author graph nodes; the evaluator is the only caller of
`_as`. Re-naming, signature changes, or making napi expose `_as`
directly would break #18.

### What's NOT frozen + WHY

- The eventual `pub(crate)` migration of `get_node` (the originally-intended
  #18 shape) — see item 1; v1-Composing decision.
- The `read_node` API name itself (the spec calls it the "read_node"
  boundary; the actual symbol IS `read_node_as`) — leave the spec
  reference; the symbol name is what's frozen.

### Verification mechanism

- `cargo-public-api` baseline (item 9).
- A documentation pin in `crates/benten-engine/src/engine_wait.rs`'s
  rustdoc cites [BAKED] #18 verbatim.

### Composing-phase escape valve

Any proposal to expose `read_node_as` over the napi boundary directly
= HALT-AND-SURFACE-TO-BEN ([BAKED] #18 explicit re-open).

---

## 3. Backend-trait SemVer-locks ([PLAN] item 3)

### Frozen surfaces (the MINIMUM)

**Conservative-minimal angle here:** the spec enumerates 6 backend-trait
forks ([PLAN] item 3, sub-clauses §4.60 / §4.61 / §4.62 / §4.63 /
§4.64 / `WriteContext`/`ChangeEvent`/`GraphError::TxAborted`
`#[non_exhaustive]`). Lock the **trait surface shape as currently
shipped**; for sub-decisions the shipped code already implements one
option, freeze THAT option, do NOT re-litigate.

| Sub-fork | Currently shipped option | Lock at v1-beta? |
|---|---|---|
| §4.60 `GraphBackend::Transaction::run<F,R>` | The trait is `pub trait GraphBackend` at `crates/benten-graph/src/graph_backend.rs:238`; `Transaction::run<F,R>` already shipped. | **YES — lock as-shipped.** |
| §4.61 `snapshot()` infallible-vs-Result | Current shape is whatever ships at `c4a37bb`-era baseline. | **Lock as-shipped.** Reason: pre-existing CI tests already exercise the chosen signature. |
| §4.61 `register_subscriber()` ()-vs-Result | As above. | **Lock as-shipped.** |
| §4.62 `BlobBackend` additive-default-vs-split | `pub trait BlobBackend` at `crates/benten-graph/src/backends/blob_backend_trait.rs:120`. | **Lock as-shipped** (additive-default if that's what's there; split if not). Either is a Composing-cost compatible shape. |
| §4.63 `KVBackend` sync-vs-RPITIT | `pub trait KVBackend: Send + Sync` at `crates/benten-graph/src/backend.rs:306`. | **Lock as-shipped** (sync today). RPITIT-or-async migration would be SemVer-breaking + is named for post-v1 in `docs/future/phase-4-backlog.md`. |
| §4.64 light-client mode-b/c trait destination | `MerkleRangeProofBackend` is **not yet a real trait** at HEAD (referenced as future scope in `crates/benten-graph/src/backends/snapshot_blob.rs:24`). | **NOT FROZEN at v1-beta**; the trait is named-but-unshipped. **BELONGS-NAMED-NOW to G-COMP-1 §8-B-in-`benten-sync`** per [RATIFIED-PREWORK] §8-B (b). The §1.A.FROZEN item 3 enumeration assumes the trait exists; the conservative angle says: don't freeze a trait that hasn't been built yet. |
| `WriteContext` `#[non_exhaustive]` | At `crates/benten-graph/src/lib.rs:935` (`pub struct WriteContext`). Not currently `#[non_exhaustive]`. | **APPLY** at the freeze wave — this is one of the defensive `#[non_exhaustive]` items (item 11 also covers it). |
| `ChangeEvent` `#[non_exhaustive]` | At `crates/benten-core/src/change_stream.rs:123` — ALREADY `#[non_exhaustive]`. | **Lock as-shipped.** |
| `GraphError::TxAborted` `#[non_exhaustive]` | At `crates/benten-graph/src/lib.rs:463` — `GraphError` already `#[non_exhaustive]`; the `TxAborted` variant per-variant `#[non_exhaustive]` decision unclear at HEAD. | **APPLY per-variant `#[non_exhaustive]`** to `TxAborted` defensively (item 11 sweep). |

### What "frozen" means here

The backend trait surfaces become SemVer-locked: signatures, generic
parameters, default-method-bodies, associated types, and trait bounds
are checked by `cargo-public-api`. Implementer crates (RedbBackend,
BrowserBackend, SnapshotBlobBackend) inherit the same lock by
implementing-the-trait.

### What's NOT frozen + WHY

- **`MerkleRangeProofBackend` trait** — not yet shipped at HEAD;
  BELONGS-NAMED-NOW to G-COMP-1's `benten-sync` light-client mode-b/c
  work per [RATIFIED-PREWORK] §8-B. **Reasoning:** [PLAN] item 3 lists
  it as a Ben decision-point for v1-beta because the *destination*
  (graph-vs-sync) was the §8-B fork, which is now DECIDED. But the
  trait shape itself hasn't been authored — freezing a phantom shape
  is overcommit. Composing builds the trait + freezes it
  *intra-Composing* per the standard module-internal SemVer discipline.
- **Backend-impl-specific signatures** (constructor sigs, redb-specific
  config types) — NOT in the GraphBackend trait surface; out of scope
  at v1-beta freeze. They CAN evolve post-v1 without breaking the trait
  contract.

### Verification mechanism

- `cargo-public-api` baseline for each of `benten-graph` (multi-trait),
  `benten-core` (`ChangeEvent`), `benten-sync` (transport trait —
  already in `docs/public-api/benten-sync.json`).
- A test that constructs `Arc<dyn GraphBackend>` for the three shipped
  backends compile-pins the trait's object-safety.

### Composing-phase escape valve

Composing introducing a new backend impl is FINE if it just
`impl GraphBackend for X { ... }` — no freeze break. Composing
adding a new method to `GraphBackend` directly = SemVer break = HALT.

---

## 4. D2 v1-canonical-bytes contract — P-III Ben decision-point

### Frozen surfaces

This is a **scheduled Ben decision-point** ([PLAN] item 4, marker
"P-III BEN DECISION-POINT the plan SCHEDULES, never makes"). The
G-CORE-9 freeze wave SURFACES the decision; Ben makes it.

**The conservative-minimal angle does NOT pre-decide the wire format
for Ben.** What G-CORE-9 produces here is:

1. A documented **inventory** of every wire-format-bearing surface that
   currently exists (CBOR envelopes; AEAD wrap envelopes; UCAN
   envelopes; DropBundle envelope; SnapshotBlob; TwoCidStore mapping
   format).
2. The current **explicit format-version discriminator** for each
   (e.g. `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 2` at
   `crates/benten-graph/src/backends/snapshot_blob.rs:125`;
   `DropBundleVersion` enum at `crates/benten-drop/src/lib.rs`).
3. The list of surfaces that **DO NOT YET HAVE** a byte-pin test in
   CI (the wire-format pre-flight gap).
4. **A separate Ben-decision document** (proposed
   `docs/V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md`) that Ben signs after
   reviewing the inventory + the §8-B-(i) `SnapshotBlob.schema_version`
   `1→2` bump in-place reasoning ([RATIFIED-PREWORK] §8-B; G-CORE-6b
   already discharged the bump at `#1331 ecc5111e` under
   "no-users-yet" P-III override).

The lock is: **once Ben signs the decision doc, every named wire format
+ its format-version discriminator are SemVer-locked**; the
discriminator + the canonical bytes of `format_version=1` (or
`format_version=2` for SnapshotBlob — already SHIPPED) cannot evolve
without explicit Core re-open.

### What "frozen" means here

After Ben's P-III decision, this section is amended to enumerate each
locked wire surface with:
- Its discriminator (the `format_version: u32` field name)
- Its canonical-bytes scheme (CBOR-with-deterministic-encoding +
  canonical-CID multihash combo per [BAKED] #5)
- Its byte-pin test path (any surface without one is a v1-beta release
  blocker — added before tag)

### What's NOT frozen + WHY

- **The wire format for a surface that does not yet have a byte-pin test**
  cannot legitimately freeze; even an inventory entry without a fixture
  is just narrative.
- **MerkleRangeProofBackend's proof bytes** — the trait isn't built at
  HEAD; **BELONGS-NAMED-NOW** to G-COMP-1 §8-B placement.

### Verification mechanism

- A new CI lane (extending the existing cite-drift workflow) walks every
  surface in the inventory, asserts a byte-pin test exists, and asserts
  the test's golden fixture is checked-in.
- Per-surface tests use canonical-bytes-stable encoding harnesses.

### Composing-phase escape valve

ANY wire-format mutation in Composing = HALT-AND-SURFACE-TO-BEN
(structural — the byte-pin test fails CI; you cannot land the change
silently). This is the strongest backstop in the entire freeze contract
because wire-format breaks are non-recoverable post-`v1-GM`.

---

## 5. #989 `WriteContext` shape — partition seam frozen

### Frozen surfaces

The full struct at `crates/benten-graph/src/lib.rs:935`:

```rust
#[non_exhaustive]  // ← APPLY at G-CORE-9 (item 11)
pub struct WriteContext {
    pub authority: WriteAuthority,
    pub actor_cid: Option<Cid>,
    pub clock: Arc<dyn TimeSource>,
    pub namespace_did: Option<Cid>,   // ← G-CORE-1 / #989
}
```

The four-field shape + every builder (`WriteContext::new`,
`with_namespace_did` at `lib.rs:1023`, etc.) + accessor signatures are
LOCKED.

### What "frozen" means here

`cargo-public-api` catches any field add/rename/type-change.
`#[non_exhaustive]` (applied here per item 11) PERMITS future additive
field additions without breaking external `WriteContext { .. }`
construction (external code uses builders, not direct struct literal —
this is the load-bearing reason for `#[non_exhaustive]`).

**Cross-DID non-leak invariant ([PLAN] C1):** the security property
that `namespace_did=X` writes cannot be read from `namespace_did=Y`
views is FROZEN as a runtime contract. Existing test pin at
`crates/benten-graph/tests/tf1_989_cross_did_partition_isolation.rs`.
A Composing-time backend impl that breaks the invariant = test fails =
HALT.

### What's NOT frozen + WHY

- **The `RedbBackend::scoped(did)` constructor signature** — that's a
  backend-impl surface, not part of the trait. Out of scope here;
  covered by item 3.
- **Whether other backends (BrowserBackend, SnapshotBlobBackend)
  implement scoped-views** — those backends fail-closed on
  `Some(namespace_did)` per `lib.rs:649`; the v1-beta lock is on the
  failure-mode shape, not on the backends implementing it.

### Verification mechanism

- `cargo-public-api` (item 9).
- `tf1_989_cross_did_partition_isolation` test pin.
- A new no-regression test that scans `WriteContext` builder code paths
  for accidental `namespace_did = None` overrides post-write.

### Composing-phase escape valve

A new `WriteContext` field is ADDITIVE per `#[non_exhaustive]` and
permitted in Composing as long as: (a) builder added, (b) default
behavior preserved, (c) byte-pin test for serialized form covers the
new field (the wire-format implication needs item 4's Ben decision-point).

---

## 6. Crypto seam boundary — the agile frame, not the algorithm specifics

### Frozen surfaces (the conservative-minimal angle SHARPENED)

The conservative-minimal angle on crypto = **lock the framing, NOT the
specific algorithm implementation versions**. This matches [BAKED] #5
exactly ("the permanent commitment is the self-describing multiformats
framing, NOT any one algorithm").

**LOCKED at v1-beta:**

1. **`benten-crypto-suite` integration-crate boundary** — the crate
   IS the seam; the public-API of `benten_crypto_suite` (re-exports
   at `crates/benten-crypto-suite/src/lib.rs:131..144`) is frozen.
   `cargo-public-api` enforces.

2. **Codepoint table values** — the integer values for every codepoint
   that ships at v1-beta:

   | Surface | Constant | Value | Status at v1-beta |
   |---|---|---|---|
   | Hash | `HashCodepoint::BLAKE3` | `0x1e` | LIVE, default |
   | Hash | `HashCodepoint::SHA2_512_256` | `0x1015` | reserved fallback |
   | Hash | `HashCodepoint::SHA3_256` | `0x16` | reserved fallback |
   | Sig | `SigCodepoint::HYBRID_ED25519_MLDSA65` | `0x0001` | LIVE, default |
   | Sig | `SigCodepoint::CLASSICAL_ED25519` | `0x0002` | LIVE, non-default downgrade |
   | Sig | `SigCodepoint::HYBRID_MLDSA65_SLHDSA` | `0x0003` | reserved-unimplemented (NF-1) |
   | Cipher | `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` | `0x647a` | LIVE, default |
   | Cipher | `CipherSuiteCodepoint::CLASSICAL_X25519` | `0x6400` | LIVE, non-default downgrade |
   | Cipher | `CipherSuiteCodepoint::NONE_PLAINTEXT` | `0x0000` | LIVE, non-default downgrade |
   | Cipher | `CipherSuiteCodepoint::HYBRID_MLKEM768_HQC` | `0x647b` | reserved-unimplemented (NF-1 KEM end-state; FIPS 207-final gated) |
   | Cipher | `CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY` | `0x647c` | reserved swap-matrix arm (NOT default; pre-FREEZE bundle #1342 mint) |

   These integer values are PERMANENT — adding new ones is additive
   (additive-codepoint invariant, item 14), but **NEVER reuse a value
   for a different algorithm** (the #1341 0x647b drift incident
   surfaced this discipline in writing; cite Ben morning ratification
   2026-05-24).

3. **Wire envelope structure** — the `AeadEnvelope` shape (at
   `crates/benten-graph/src/aead_wrap.rs`) + the `WrappedKey`
   wire form + the hybrid-sig concatenated/committing/strip-resistant
   construction (NF-4; both halves MUST verify). Byte-pinned at the
   item 4 wire-format Ben decision-point.

4. **Multi-device key-wrap/recovery envelope SHAPE** — NOT the recovery
   *protocol* (which stays G-COMP-3 per [PLAN] item 6). The envelope
   shape is frozen with the codepoint discipline: PQ-hybrid-by-default;
   never hardcode key/sig/ciphertext sizes.

5. **Typed-reject discipline** — `UnsupportedAlgorithm` error variant
   on every codepoint dispatcher; FAIL-CLOSED on unknown codepoint
   (NEVER silent fallback). This is a *behavior* freeze backed by tests
   in `crates/benten-crypto-suite/tests/tf3a_*.rs` + `tf4_*.rs`.

6. **No-hardcoded-sizes invariant** — at v1-beta this is a
   v1-PRODUCTION-correctness property because hybrid ML-DSA/ML-KEM is
   the default path. A test pin scans the crypto-suite + every
   consumer crate for `[u8; 32]` / `[u8; 64]` etc. that should be
   `Vec<u8>` per the agile-size discipline.

### What "frozen" means here

The seam is locked; the *algorithm versions* (RustCrypto `ml-dsa` /
`ml-kem` crate versions in `Cargo.lock`) are routine dependency updates
that DO NOT touch the codepoint table — they update behind a fixed
codepoint. C-GM-AUDIT ([RATIFIED-PQ] NF-2) pins the audited versions
once they land; until then routine updates are normal Cargo hygiene.

### What's NOT frozen + WHY

- **RustCrypto crate versions** (`ml-dsa = "X.Y.Z"`, etc.) — these are
  `Cargo.lock` semantics, not API freeze. C-GM-AUDIT formally pins them
  later.
- **The internal implementation of the X-Wing-style combiner** — the
  ~30-LOC vendored body at `crates/benten-crypto-suite/src/` is
  replaceable (Benten owns the draft version per [RATIFIED-PQ] NF-5).
  Only the codepoint `0x647a` + the inputs/outputs are frozen.
- **The independent audit DELIVERY date** — that's a v1-GM gate
  (C-GM-AUDIT), not a v1-beta freeze item.
- **NF-1 KEM end-state (`HYBRID_MLKEM768_HQC` at `0x647b`)** —
  reserved-typed-reject at v1-beta; build-trigger is FIPS-207-final
  per [RATIFIED-PQ] NF-1. **OUT OF SCOPE for v1-beta build; FROZEN as
  reserved codepoint only.**

### Verification mechanism

- `cargo-public-api` baseline for `benten-crypto-suite`.
- A test pin asserting the codepoint table integer values match the
  ratified values exactly (a single golden-file test).
- Existing `tf3a_*` + `tf4_*` tests pin typed-reject behavior.
- Item 4 byte-pin tests cover the wire envelope.

### Composing-phase escape valve

A new cipher suite is ADDITIVE: mint a new codepoint value, add a new
`pub const` on the codepoint type, add the dispatcher arm, ship the
implementation. No item 6 freeze break. **Re-using an existing
codepoint for a different algorithm = HALT-AND-SURFACE-TO-BEN
(structural — the wire-format byte-pin in item 4 will fire if anyone
tries; the #1341 0x647b incident is the in-tree example of this
discipline working).**

---

## 7. §4.33 legacy `module_ecosystem::install_plugin*` deletion

### Frozen surfaces

The deletion is **already discharged** ([PLAN] item 7 explicit hard
deadline = "Core opening wave"; verify-pass deliverable; G-CORE-0 §4.33
landed via [`#1311` wave](https://github.com/...)). G-CORE-9's role
here is **freeze the deletion as fait accompli**:

- The `module_ecosystem::install_plugin*` symbols (which used to live
  at `crates/benten-platform-foundation/src/module_ecosystem.rs`) MUST
  NOT regress.
- A no-regression test pin
  (`crates/benten-platform-foundation/tests/g_core_9_legacy_install_path_deleted.rs`)
  asserts NO public symbol matches the pattern `install_plugin*` in
  `benten_platform_foundation::module_ecosystem::*`.

### What "frozen" means here

The DELETION is permanent. A resurrection attempt fails the
no-regression test pin BEFORE landing.

### What's NOT frozen + WHY

The CURRENT install path (post-deletion) — that's
`benten_platform_foundation::plugin_lifecycle::install_plugin_*` (the
NEW path). It's a public surface frozen under items 9 + 10
(cargo-public-api).

### Verification mechanism

- The no-regression test pin above.
- `cargo-public-api` (item 9).

### Composing-phase escape valve

A new install path can be added IF AND ONLY IF the legacy-path-deletion
discipline is preserved (one canonical install path; per CLAUDE.md #15
"two install paths with different security envelopes cannot coexist").
Adding a second parallel path = HALT.

---

## 8. `benten-caps` v1-API forks decided + recorded

### Frozen surfaces

[PLAN] item 8 enumerates the decided shapes. Conservative-minimal lock:

| Sub-fork | Decision | Action at G-CORE-9 |
|---|---|---|
| #886 `[features]` | DECIDED (already shipped) | Pin `Cargo.toml` `[features]` block exactly as-is; comment-cite. |
| #993 `CapabilityPolicy` sealed-discipline shape | DECIDED (a) SEALED per [RATIFIED-PREWORK] §8-E | **Soft-seal at v1-beta** (already shipped at `crates/benten-caps/src/policy.rs:48-67` `sealed_marker::SealedCapabilityPolicy`). The hard-seal requires a workspace-wide migration; **BELONGS-NAMED-NOW** to G-CORE-8.3 (cited in `policy.rs:33` rustdoc). **Lock the soft-seal as-shipped.** Marker trait `SealedCapabilityPolicy` SemVer-locked; hard-seal upgrade in v1-Composing is additive (impls that don't add the marker compile today; adding the marker post-v1 is a behavior tighten, not a SemVer break). |
| 3 new Phase-4-Meta hooks (install-time consent / per-delegation runtime / audience-aware `check_write`) | DECIDED additive (defaulted trait methods + `CapWriteContext`/`ReadContext` audience field) | **Lock the new method signatures + the new field**. Object-safety preserved (`Arc<dyn CapabilityPolicy>` boxes; compile-test pin at `crates/benten-caps/INTERNALS.md` §9). |
| #1005 `actor_hint` shape | DECIDED | Lock as-shipped (the `actor_hint: String` placeholder per `policy.rs:81`). Tightening to a typed principal is a v1-Composing item (named in [PLAN] §1.B). |
| #883b prod-dep-edge | DECIDED | Lock as-shipped. |
| #887b `check_read` default-impl | DECIDED (defaulted; per [PLAN] §6 edit-5(g) pulled WITH/BEFORE G-CORE-8) | Lock at `crates/benten-caps/src/policy.rs:388` (`fn check_read(...) -> Result<(), CapError> { ... }` default body). |
| §4.69 organizing principle | RESOLVED (a) `EngineCapsHandle`-canonical — see item 1 | Already frozen at item 1; no-regression invariant pin. |

### What "frozen" means here

`cargo-public-api` for `benten-caps` (the txt baseline at
`docs/public-api/benten-caps.txt` is currently a seed-only stub; the
G-CORE-9 wave REGENERATES the real baseline). Sealed-discipline
locked: hard-seal is permitted in Composing (additive, not breaking).

### What's NOT frozen + WHY

- **The hard-seal migration** (workspace-wide test impl migration) —
  G-CORE-8.3 follow-up wave; v1-Composing destination.
- **`actor_hint` upgrade to typed principal** — v1-Composing
  v1-assessment-window per [PLAN] §1.B.

### Verification mechanism

- `cargo-public-api` baseline for `benten-caps` (item 9 — regenerated
  at this wave).
- Compile-test pin for `Arc<dyn CapabilityPolicy>` object-safety.
- A new pin asserting `SealedCapabilityPolicy` marker trait exists +
  is empty.

### Composing-phase escape valve

Adding a new defaulted trait method to `CapabilityPolicy` is ADDITIVE
+ permitted. Removing a method, changing a signature, or adding a
non-defaulted method = HALT.

---

## 9. `cargo-public-api` baseline regenerated + committed

### Frozen surfaces

**The G-CORE-9 wave REGENERATES every `docs/public-api/*.{txt,json}`
baseline file** from running `cargo public-api -p <crate> --simplified`
against the post-freeze code state. The regenerated files are committed
as the v1-beta public-API contract.

Crates with baselines (currently 11 — verify at wave time):
- `benten-caps.txt`
- `benten-core.txt`
- `benten-dsl-compiler.txt`
- `benten-engine.txt`
- `benten-errors.txt`
- `benten-eval.txt`
- `benten-graph.txt`
- `benten-ivm.txt`
- `benten-id.json` (json format)
- `benten-renderer-tauri.json`
- `benten-sync.json`

**Crates MISSING from the baseline that need to be ADDED at this wave:**
- `benten-crypto-suite.txt` — load-bearing crypto seam; MUST be in
  the v1-beta freeze.
- `benten-drop.txt` — DropBundle public surface (S&C item 15).
- `benten-platform-foundation.txt` — plugin lifecycle surface.

### What "frozen" means here

A `cargo-public-api` CI lane (per [PLAN] §4) runs against every crate's
baseline. A diff = CI failure. A change is landed by regenerating the
baseline + committing it intentionally + having the change pass R6
review.

### What's NOT frozen + WHY

- **Files added to a crate** that don't change the public API don't
  touch the baseline.
- **Documentation-only changes** to existing `pub` items.

### Verification mechanism

- The `cargo-public-api` CI workflow (item-9 lane).
- Existing test pin at `crates/benten-engine/tests/cargo_public_api_drift.rs`
  (per `docs/public-api/benten-caps.txt:9`).

### Composing-phase escape valve

A baseline diff = HALT. The Composing PR must include the baseline
regenerate as an explicit + reviewed change to land.

---

## 10. TS/JS `@benten/engine` public API frozen (#1204 parity gate)

### Frozen surfaces

**Conservative-minimal angle:** lock the JS surface AS-SHIPPED at v1-beta.
NOT the surface as-it-could-be.

| Surface | Source | v1-beta lock |
|---|---|---|
| `packages/engine/src/index.ts` exports | All `export` statements at HEAD | LOCKED as-shipped at the freeze wave; commit the post-freeze `index.d.ts` |
| `packages/engine/src/engine.ts` `Engine` + `PolicyKind` | As-shipped | LOCKED |
| `packages/engine/src/errors.generated.ts` `CATALOG_CODES` | The 191-entry catalog at HEAD (CATALOG_VARIANT_COUNT = 191 per #1339 post-Ben-ratify-Io-mirror) | LOCKED — mirror item 8's `ErrorCode` mirror discipline |
| `packages/engine/src/types.ts` typed-call shapes | `TypedCallInputShapes`, `TypedCallOutputShapes`, `ManifestSignature` | LOCKED — **PQ-hybrid-capable** sizes (NO hardcoded Ed25519 32B-key / 64B-sig assumption; per [PLAN] item 10 PQ-hybrid JS-shape widening) |
| `packages/engine/src/types.ts` other interface exports | `Subgraph`, `RegisteredHandler`, `AttributionFrame`, `Trace*`, `CapabilityClaim`, `DeviceAttestation`, `CapabilityGrant`, `Edge`, `TypedCallOp`, etc. | LOCKED as-shipped |
| `StreamHandle.next` shape post-PR-B | The G-CORE-10 PR-B AsyncTask migration (`#1340 05707357`) post-merge shape | LOCKED — sync→async break is ratified per [BAKED] #5 no-shims + the in-tree #1331 "no users yet" override |
| `Atrium` + `atrium_*` typed-call surfaces | As-shipped | LOCKED |

**The #1204 parity gate** (CI workflow built in G-CORE-10) is
COMMITTED-and-FREEZE-FLIPPED here per [PLAN] item 10's three-stage
sequential placement (BUILT in G-CORE-10 / COMMITTED in G-CORE-9 /
EXECUTING in §4 CI lane). The gate asserts ZERO drift between
`packages/engine/src/*.ts` post-freeze + the regenerated `index.d.ts`
at every Composing-time commit.

### What "frozen" means here

`#1204` parity gate fails CI on any JS/TS public-surface change. The
Composing PR landing such a change regenerates `index.d.ts` + commits
it (mirror of item 9's `cargo-public-api` baseline regen).

### What's NOT frozen + WHY

- **JS internal helpers** (`packages/engine/src/internal/*` or
  similar) — out of scope; not exported.
- **JSDoc-only changes** to existing exports.
- **Subpath exports** other than `errors` — out of scope; current set
  is the lock.

### Verification mechanism

- `#1204` parity gate (regenerate `index.d.ts` vs checked-in baseline).
- A new test that scans `types.ts` for hardcoded `[ 0-9]+ B` size
  literals on crypto-touching types (PQ-hybrid JS-shape widening
  invariant).

### Composing-phase escape valve

Adding a new JS export = HALT for Ben review (the parity gate fires).
Backward-compatible additions (new optional fields on existing
interfaces with `#[non_exhaustive]`-equivalent TS shape) can land with
explicit regenerate + R6 review.

---

## 11. META #907 `#[non_exhaustive]` sweep — DEFENSIVE-lean

### Frozen surfaces

**Conservative-minimal angle SHARPENED for SemVer-asymmetric items.**
The cost of NOT applying `#[non_exhaustive]` to an enum that will need
to grow post-v1 is a SemVer break — *non-recoverable post-`v1-GM`*.
The cost of applying it where it's not strictly needed is mostly a
documentation/match-arm-awkwardness tax. **Lean DEFENSIVE here.**

**Apply `#[non_exhaustive]` at v1-beta to every public enum + struct
in the following inventory unless EXPLICITLY justified otherwise**
([PLAN] item 11 enumerates the workspace surface; G-CORE-9 brief
EXPANDS the enumeration to "every public enum/struct"):

| Crate | Type | Currently `#[non_exhaustive]`? | v1-beta action |
|---|---|---|---|
| `benten-engine` | `EngineError` | YES (`error.rs:21`) | KEEP |
| `benten-engine` | `engine_config::*` | YES (`engine_config.rs:194`) | KEEP |
| `benten-engine` | `engine_sync::*` | YES (`engine_sync.rs:105`) | KEEP |
| `benten-engine` | All other public engine enums (11+ per [PLAN] item 11) | AUDIT + APPLY missing | At wave-time: scan + apply each as needed |
| `benten-core` | `WriteAuthority` | YES (`policy.rs` re-export) | KEEP |
| `benten-core` | `Subgraph::PrimitiveKind` | YES (`subgraph.rs:68`) — load-bearing per [BAKED] #1 12-primitive-irreducibility | KEEP — the `non_exhaustive` here is the **DEFENSIVE guard against future-13th-primitive proposals** that [BAKED] #1 rejects. |
| `benten-core` | `ChangeEvent` | YES (`change_stream.rs:123`) | KEEP |
| `benten-core` | `ChangeKind` | YES (`change_stream.rs:85`) | KEEP |
| `benten-core` | `subgraph_spec::Spec` + `SpecError` | YES (`spec.rs:125`, `errors.rs:19`) | KEEP |
| `benten-core` | `version_dag::*` | YES (`version_dag.rs:105`) | KEEP |
| `benten-ivm` | `AlgorithmError` | per [PLAN] item 11 | AUDIT + APPLY |
| `benten-sync` | §4.71 5-enum | per [PLAN] item 11 | AUDIT + APPLY |
| `benten-caps` | `CapError` | YES (`error.rs:26`) | KEEP |
| `benten-caps` | `RestrictedSpec` | YES (`restricted_spec.rs:61`) | KEEP |
| `benten-caps` | `PendingOp` | YES (`policy.rs:102`) | KEEP |
| `benten-caps` | `TypedCapGroup` | per [PLAN] item 11 | AUDIT + APPLY |
| `benten-caps` | **`Scope`** | NO (`scope.rs:45` — DELIBERATELY NOT, per item 15(c)) | **DO NOT APPLY** — explicit no-opaque-arm freeze per [RATIFIED-S&C] §R1; the EXACTLY-two-arms-by-the-type-system property IS the structural pin |
| `benten-graph` | `WriteContext` | NO at HEAD | **APPLY** (item 5; defensive future-additive-field) |
| `benten-graph` | `ChangeEvent` | YES (per `change_stream.rs:123` re-export) | KEEP |
| `benten-graph` | `GraphError` | YES (`lib.rs:463`) | KEEP |
| `benten-graph` | `GraphError::TxAborted` (per-variant) | Unclear at HEAD | **APPLY** if a future arm is plausible (it is) |
| `benten-graph` | `WriteAuthority` (re-export from core) | YES | KEEP |
| `benten-graph` | per `lib.rs:585` Fwd-2 #997 / #1207 decision NOT to apply — preserve | NO (explicit reason) | DO NOT APPLY — the reason at the cite is the law |
| `benten-crypto-suite` | `UnsupportedAlgorithm` | TBD | APPLY (future cipher suites) |
| `benten-crypto-suite` | `SwapMatrixError` | TBD | APPLY |
| `benten-drop` | `DropBundleError`, `DropBundleVersion`, `DropContentMode` | TBD | APPLY each |
| `benten-renderer-tauri` | `IpcMethod` (the per-method allowlist struct) | TBD | APPLY |

### What "frozen" means here

`#[non_exhaustive]` becomes part of the public-API contract (item 9
catches removal). The decision-per-type is recorded INLINE in this
document; a future PR proposing to remove `#[non_exhaustive]` from any
of the above = explicit re-open justification.

### What's NOT frozen + WHY

- **`Scope` enum** — DELIBERATE non-application; the
  EXACTLY-two-arms-by-type-system property is the freeze per item 15(c).
  This is a CARVED-OUT NON-APPLICATION, not an omission.
- **Per-variant `#[non_exhaustive]` on enum variants currently with no
  future arm in sight** — applying speculatively has a documentation
  cost; lean MINIMAL on the per-variant decision (the per-type
  decision is defensive).
- **Private types** — out of scope; only `pub` enums/structs.

### Verification mechanism

- `cargo-public-api` (item 9) — `#[non_exhaustive]` is part of the
  baseline diff.
- A new audit test pin
  (`crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs`) walks
  every `pub enum`/`pub struct` across the workspace + asserts
  `#[non_exhaustive]` presence per the inventory above (or an explicit
  carve-out cite).

### Composing-phase escape valve

ADDING `#[non_exhaustive]` to a type that doesn't have it = additive +
permitted in Composing (caveat: it's actually SemVer-breaking for
external direct-struct-literal construction, so the migration path
must be tested). REMOVING `#[non_exhaustive]` = HALT.

---

## 12. G-CORE-8 security-surface public-shape lock

### Frozen surfaces

[PLAN] item 12 enumerates these — all DECIDED + SHIPPED per `#1338`
(G-CORE-8 fix-pass) + the wave-2 batch `#1340`:

- `ManifestEnvelopeRecheckOutcome` enum at
  `crates/benten-engine/src/manifest_envelope_recheck.rs:80` —
  variants frozen: `Admitted`, `NotApplicable`, `UnresolvedDeny` (the
  fail-closed rename from the pre-G-CORE-8 BLOCKER), `OutsideEnvelope`,
  etc.
- `outcome_to_row_reject` mapping fail-closed semantic: `UnresolvedDeny`
  → `Err(...)`; **never** collapse to `Ok(())`. Test pin at
  `crates/benten-engine/tests/g_core_8_manifest_envelope_recheck_fail_closed_flip_4_36.rs`
  pins this.
- `ProductionManifestEnvelopeRechecker` auto-wired into DEFAULT engine
  builder (no opt-in required) — frozen behavior.
- `accept_atrium_share` (`crates/benten-platform-foundation::plugin_lifecycle::accept_atrium_share`)
  — the cross-peer install seam; signature frozen at v1-beta (re-verifies
  `bytes_cid == announced_cid` AND `peer_did_signature_valid_for_bytes`).
- §4.40 key-at-rest public type (if any was minted at G-CORE-7 install
  hardening) — AUDIT + freeze whatever shipped.

### What "frozen" means here

The security-shape semantics (fail-closed on unresolved peer-DID; never
admit-on-unresolved; production rechecker is the DEFAULT) are runtime
invariants pinned by tests. SemVer-locked.

### What's NOT frozen + WHY

- **Internal recheck implementation details** — out of scope; the
  semantic + the variant set are the freeze.

### Verification mechanism

- The `g_core_8_*_4_36.rs` test pin.
- `cargo-public-api` baseline (item 9).

### Composing-phase escape valve

Adding a new `ManifestEnvelopeRecheckOutcome` variant is gated by
`#[non_exhaustive]` (item 11); the variant MUST default to denying
behavior (the structural fail-closed invariant). Adding a new variant
that admits = HALT.

---

## 13. Engine↔host runtime-ownership boundary — bridged-dual-runtime

### Frozen surfaces

Per [RATIFIED-PREWORK] §8-C + [PLAN] item 13:

- **No `tauri`/`tokio` dep in `benten-renderer-tauri`** — pinned
  at the crate's `Cargo.toml`; verified by a compile-test that
  imports `benten_renderer_tauri::*` + asserts no `tauri::Runtime`
  or `tokio::runtime::*` type appears in `Renderer` trait signatures.
- **No host-shell runtime handle/ownership in the engine API surface**
  — pinned by a compile-test that constructs an `EngineBuilder` +
  asserts no signature accepts a `tauri::Runtime` or borrows a
  `tokio::runtime::Handle`.
- **`IPC_METHODS` const-allowlist** at
  `crates/benten-renderer-tauri/src/lib.rs:127` — the v1-beta freeze
  KEEPS the const-allowlist property (no registration affordance).
  Adding methods requires explicit `IPC_METHODS` const update + the
  pre-flight gate per the same file's narrative. [PLAN] item 13(a)
  explicitly notes this is a "deliberate T3-defense" not a "gap to
  fill" — conservative-minimal preserves the discipline.
- **The §4.22 thin-client bridge principal-resolution** (G-CORE-8) +
  the §8-C bridged-dual-runtime channel/IPC boundary contract: ONE
  consistent IPC contract across (a) `IPC_METHODS` shape, (b)
  principal-resolution, (c) the channel/IPC boundary. Frozen as the
  shipped shape from `#1340`.

### What "frozen" means here

A compile-test pins the runtime-handle-leak prevention. A test pin
asserts `IPC_METHODS` is a `const` (not a `static mut`, not a
dynamic registry).

### What's NOT frozen + WHY

- **Adding new IPC methods to `IPC_METHODS`** — additive; pre-flight
  gate per the rustdoc narrative; permitted in Composing.
- **The `Renderer` trait at `crates/benten-platform-foundation/src/materializer.rs:552`**
  itself — its public shape IS frozen (item 9), but it's not a
  runtime-ownership surface.

### Verification mechanism

- Compile-test pin for runtime-handle-leak prevention (new test).
- A pin asserting `IPC_METHODS` const-shape.
- Item 9 `cargo-public-api` baseline.

### Composing-phase escape valve

ANY proposal to let the engine borrow a host shell's runtime
(`tauri::Builder::with_runtime`-style) = HALT-AND-SURFACE-TO-BEN per
[RATIFIED-PREWORK] §8-C (explicit ratification — "the engine NEVER
borrows the host shell's runtime").

---

## 14. P2P-interop conformance invariant

### Frozen surfaces

Per [PLAN] item 14 + [RATIFIED-PQ] §4:

(a) **Mandatory baseline conformance suite** — every peer MUST satisfy
the conformance corpus. The corpus IS the existing G-CORE-3c swap-matrix
tests at `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance*.rs`
+ a new `crates/benten-crypto-suite/tests/conformance_baseline.rs`
that pins the v1-beta baseline as the lower bound (Veilid
`common_crypto_kinds`-intersection model).

(b) **Typed-unsupported-error, never silent fallback** on unknown
crypto — pinned by `UnsupportedAlgorithm` + per-codepoint dispatcher
tests already at HEAD. Fail-closed (Veilid + MLS + Nostr NIP-44
precedent).

(c) **No wire-break when a codepoint is added** — additive-codepoint
discipline. Pinned by a test that walks `CipherSuiteCodepoint` +
`SigCodepoint` + `HashCodepoint` constant lists + asserts every value
maps to EITHER a LIVE implementation OR a typed-reject arm (never a
panic / silent / wildcard match).

(d) **Old codepoints supported forever** — never strand previously-written
content. Pinned narratively in `docs/SECURITY-POSTURE.md` + by
a test that decodes a historical fixture (BLAKE3-encoded; classical
Ed25519 sig from a pre-#1300 baseline) under the post-freeze code
+ asserts it still verifies.

(e) **IANA component IDs reused; Benten owns only the suite-selector
codepoint table** — per [RATIFIED-PQ] §3. Pinned by a docstring on
`CipherSuiteCodepoint` citing IANA HPKE/COSE registries + MLS
one-codepoint-per-suite model.

### What "frozen" means here

Each invariant is pinned by a test OR a narrative-in-docs reference.
Violating any invariant in Composing = test failure or doc-review halt.

### What's NOT frozen + WHY

- **Future cipher suite additions** — additive per the
  additive-codepoint invariant; the FRAMEWORK is frozen, the
  ADDITIONS are permitted.
- **The NF-1 PQ⊕PQ end-state arms** — reserved-typed-reject codepoints;
  build-trigger is FIPS-207-final ([RATIFIED-PQ] NF-1); v1-beta locks
  the reservation, not the build.

### Verification mechanism

- Conformance baseline test (new).
- Additive-codepoint enumeration test (new; scans codepoint constant
  blocks).
- Historical-fixture decode test (new; pins old-codepoint-supported-forever).
- The cite-drift CI lane (item 6 mechanism).

### Composing-phase escape valve

Reusing a codepoint for a different algorithm = HALT (the #1341
incident already established this discipline). Silent fallback on
unknown codepoint = HALT (typed-reject is the law). Wire-break = HALT
(item 4 byte-pin tests fire).

---

## 15. Sharing & Confidentiality (S&C) public surface

This is the largest section by virtue of [PLAN] item 15's 10 sub-clauses.
Conservative-minimal angle: lock what's SHIPPED at HEAD ([RATIFIED-S&C]
section-by-section verified live at `ae7cd3d5` per the handoff sequencing);
items named-but-unshipped get HARD-RULE-12 BELONGS-NAMED-NOW
dispositions rather than premature freezes.

### 15(a) SubgraphSpec primitive — 4-thing thin core

**Frozen surfaces:**
- `benten_core::subgraph_spec::Spec` struct (`crates/benten-core/src/subgraph_spec/spec.rs:125`)
  — 4-thing thin core: Roots / Expansion / Inclusion / Termination.
- `#[non_exhaustive]` already APPLIED (per source) — KEEP.
- **CLAUDE.md baked-in #1 12-primitive-irreducibility preserved** — Spec
  IS a `Subgraph` composed of existing 12 primitives, NOT a new
  `PrimitiveKind` variant. A test pin in
  `crates/benten-core/tests/g_core_9_no_thirteenth_primitive.rs` asserts
  `PrimitiveKind::*` discriminant count remains 12.

**What "frozen" means here:** the 4-thing decomposition + the
12-primitive-irreducibility are runtime + type invariants. SemVer-locked.

**Not frozen + WHY:** internal evaluator details — out of scope.

**Verification:** the no-13th-primitive test pin; `cargo-public-api`
(item 9).

**Escape valve:** a proposed 13th primitive = HALT-AND-SURFACE-TO-BEN
([BAKED] #1 explicit).

### 15(b) `RestrictedSpec` shape — 6-dimension product

**Frozen surfaces:**
- `benten_caps::RestrictedSpec` struct (`crates/benten-caps/src/restricted_spec.rs:102`)
  — 6 dimensions: roots + edge-allowlist + max_depth + label-allowlist
  + label-denylist + property-equalities.
- `#[non_exhaustive]` APPLIED — KEEP (future-additive named dimensions).
- `RestrictedSpec::contains()` decidability contract documented.
- **`no-opaque-arm` decision** documented in scope's rustdoc + tested.

**What "frozen" means here:** the 6-dimension structural shape + the
no-opaque-arm semantic.

**Not frozen + WHY:** future-additive named dimensions (Boolean OR /
numeric comparisons / has_edge / anchor-chain LIMIT) — these are
ADDITIVE per `#[non_exhaustive]` + the named-extension-slot pattern.

**Verification:** `cargo-public-api`; the `restricted_spec.rs`
narrative-as-cite; existing tests in `crates/benten-caps/tests/tf3b_*.rs`.

**Escape valve:** adding an OPAQUE dimension (vs a NAMED slot) = HALT
([RATIFIED-S&C] R1 explicit).

### 15(c) Structured UCAN `Scope` enum — EXACTLY two arms

**Frozen surfaces:**
- `benten_caps::Scope` enum (`crates/benten-caps/src/scope.rs:46`) —
  TWO arms: `Hashes(Vec<Cid>)`, `RestrictedSelector(RestrictedSpec)`.
- **Explicitly NOT `#[non_exhaustive]`** — the EXACTLY-two-arms-by-the-
  type-system property IS the structural pin per item 11 + the
  rustdoc cite in `scope.rs:32`.
- Wire envelope is codepoint-dispatched (typed for additive future arms
  via the standard codepoint discipline) — even though the enum is
  closed, the WIRE can be widened additively WITHOUT touching this enum
  (a new wire arm would require both Core re-open + this enum +
  HALT-AND-SURFACE).

**What "frozen" means here:** the two-arm count IS the freeze. The
wave-3b structural test (`tf3b_no_opaque_selector_arm_structural`)
ensures a third arm trips an exhaustive-match compile error at every
downstream call site.

**Not frozen + WHY:** the WIRE codepoint encoding (item 4 P-III) — that
locks the bytes for the two arms; additive wire arms are out of scope.

**Verification:** the `tf3b_no_opaque_selector_arm_structural` test;
`cargo-public-api`; the no-opaque-arm sentinel scan in
the cite-drift CI lane.

**Escape valve:** any proposal for `OpaqueSelector` or third arm =
HALT-AND-SURFACE-TO-BEN per [RATIFIED-S&C] §R1 (structurally unsound
per Spike H+1.1; explicit re-open justification required).

### 15(d) `AuthorizationGrant` envelope = ONE signed artifact

**Frozen surfaces:**
- `benten_caps::AuthorizationGrant` struct (`crates/benten-caps/src/authorization_grant.rs:205`):
  - `ucan: UcanEnvelope` (the UCAN half)
  - `key_material: KeyMaterial` (the key-material half)
  - `binding_sig: Vec<u8>` (issuer's sig over canonical
    `(ucan, key_material, audience)` bytes)
  - `audience_binding: Cid`
  - `issuer_verifying_key: Vec<u8>` (Ed25519 verifying key)
  - `audience_pubkey: Option<Vec<u8>>` (G-CORE-3e ALPN handler use)
  - The G-CORE-3e `RestrictedSpec` scope field (per the partial
    file content)
- `#[non_exhaustive]` — APPLY at the freeze wave (per item 11; the
  struct grew between 3b + 3e and will plausibly grow further).
- **Binding-sig-validation-FIRST ordering** semantic: validators MUST
  check `binding_sig` BEFORE consulting UCAN scope or key material
  ([RATIFIED-S&C] §R3 quoted explicit). Pinned by the
  `verify_binding` implementation + tests.

**What "frozen" means here:** the cross-online-and-offline-shape
consistency (custom-ALPN online path + Drop bundle offline path BOTH
use this exact envelope) is part of the freeze.

**Not frozen + WHY:** field-additions are additive per
`#[non_exhaustive]`. The internal `binding_sig` computation algorithm
(canonical-bytes-of-`(ucan, key_material, audience)` tuple) is
byte-pin-frozen at item 4 (Ben P-III).

**Verification:** existing `tf3b_*` tests pin envelope shape + binding-
sig semantics; `cargo-public-api` (item 9); item 4 byte-pin.

**Escape valve:** changing the binding-sig algorithm or the
validation-ordering invariant = HALT (cryptographic).

### 15(e) Encryption-class enum — `Public` + `Confidential`

**Frozen surfaces:**

**CONSERVATIVE FLAG:** at HEAD-`ae7cd3d5`, NO `EncryptionClass` enum
appears in the codebase (grep confirms). [PLAN] item 15(e) names the
enum + reserved variants (`AnonymousGroup`, `PrivateLocal`) + the §8-CC
closure shape, but it hasn't been minted.

**Conservative disposition:** **NOT FROZEN at v1-beta in its full form;
HARD RULE 12 BELONGS-NAMED-NOW:**

| Option | Disposition |
|---|---|
| Mint `EncryptionClass { Public, Confidential, #[non_exhaustive] }` at the G-CORE-9 wave with reserved-typed-reject for `AnonymousGroup`/`PrivateLocal` future arms | **DO at G-CORE-9 IF** the §8-CC consumers exist at HEAD (the encryption-class call sites need the enum). Verify at wave time. |
| Defer the enum mint to G-COMP-1 entirely | **AVOID** — if a downstream consumer needs to distinguish Public vs Confidential, the freeze is needed before downstream Composing code shapes its API. |

**Wave-time decision:** orchestrator verifies the §8-CC consumers'
state. If they exist → mint + freeze the enum at this wave (with
`#[non_exhaustive]` + typed-reject for unknown classes). If they
don't → BELONGS-NAMED-NOW to G-COMP-1 with the enum's shape spec
captured here verbatim.

**What "frozen" means here (assuming wave-time mint):** the two-arm
LIVE set + the reserved-not-yet-built arms via `#[non_exhaustive]` +
typed-reject pattern.

**Not frozen + WHY:** `AnonymousGroup`/`PrivateLocal` impls — reserved
arms; build-deferred per [PLAN] item 15(e).

**Verification:** `cargo-public-api`; a new test pin for the typed-reject
on unknown class variants.

**Escape valve:** new enum variant = HALT-AND-SURFACE-TO-BEN if it
collapses the encryption-class taxonomy ([RATIFIED-S&C] §R6 +
[PLAN] item 15(e)).

### 15(f) Two-path key-derivation contract — Interpretation B + Spike-E

**Frozen surfaces:**
- `benten_crypto_suite::derive_root` + `derive_step` API
  (`crates/benten-crypto-suite/src/structural_kdf.rs`):
  - `K(root) = HKDF-SHA256(K_principal, info = "root" || root_cid)`
  - `K(N) = HKDF-SHA256(K(predecessor), info = "step" || edge_label || N.cid)`
- `StructuralKdfKey` zeroize-on-drop output type.
- **The `"step"` + `"root"` HKDF info-tags** for cross-role domain
  separation (per Spike E correction + R0.8).
- `derive_step_without_info_tag_for_test` — test-only escape; pinned
  as `_for_test`-suffixed (a HARD-RULE-12 BELONGS-NAMED-NOW for
  v1-Composing rename to `_internal` per the item 1 visibility-cluster
  pattern, but lock the test-only escape AS-IS at v1-beta; renaming a
  test helper isn't safety-critical).
- KDF = HKDF-SHA256 v1-beta default (codepoint-dispatched per
  [BAKED] #5; BLAKE3 stays the content-hash).
- **Path-tagged keys** semantic: a Node reachable by multiple paths
  gets multiple distinct keys. Documented contract; tested via the
  `derive_step` determinism + path-dependence test corpus.

**What "frozen" means here:** the corrected (per Spike E) derivation
formula with the explicit info-tags; not the literal-DESIGN-doc formula
(which Spike E proved doesn't converge).

**Not frozen + WHY:** the internal HKDF implementation crate version —
`Cargo.lock` semantics.

**Verification:** `cargo-public-api`; existing `structural_kdf.rs`
tests; the `tf3a_*` integration tests.

**Escape valve:** mutating the info-tag structure ("step"/"root" → different
strings) = HALT (cryptographic; breaks downstream derivation).

### 15(g) Two-CID mapping + per-chunk-AEAD chunk-size

**Frozen surfaces:**
- The `plaintext_cid → ciphertext_cid` redb-backed mapping (G-CORE-3d
  at `#1323`) — the mapping table shape; the column order; the
  on-disk format (item 4 P-III byte-pin).
- `IROH_BLOCK_SIZE: usize = 16 * 1024` constant at
  `crates/benten-crypto-suite/src/aead.rs:52` — the chunk-size
  alignment with iroh-blobs's native block size is LOAD-BEARING per
  [RATIFIED-S&C] §R2 (different chunk size = double-chunking overhead).
- The 64 KiB threshold for chunked-vs-whole-AEAD heuristic (documented
  per `aead.rs:35`).
- AAD-binds-chunk-index semantic (preserves per-chunk decryptability for
  range-fetched slices).
- The `TwoCidStore` wrapper in `benten-sync` (file
  `crates/benten-sync/src/two_cid_store.rs`).

**What "frozen" means here:** the byte-format of the mapping is part of
item 4. The CHUNK SIZE constant is locked; changing it breaks
backward-compat for previously-encrypted content.

**Not frozen + WHY:** the internal redb table layout — that's an
implementation detail behind the mapping API. The chunk-fetch
performance characteristics — measurable but not frozen.

**Verification:** byte-pin (item 4); a test pin asserting
`IROH_BLOCK_SIZE == 16 * 1024` (golden constant).

**Escape valve:** changing the chunk size = HALT (wire-format break +
content-incompatibility).

### 15(h) SubgraphSpec walker IS a Subgraph shipped once in `benten_core`

**Frozen surfaces:**
- `benten_core::subgraph_spec::walker` module + the
  `pub fn walk(spec: &Spec) -> Result<WalkResult, SubgraphSpecError>`
  function (`crates/benten-core/src/subgraph_spec/walker.rs:78`).
- The `pub fn walker_as_subgraph() -> Subgraph` (line 183) — the
  fractal property (walker IS a Subgraph composed of existing
  primitives).
- `WalkResult.enumerated: Vec<(Cid, StructuralPath)>` — the BFS-order
  enumeration shape ([RATIFIED-S&C] §R4 explicit).
- **BFS-order = canonical path** semantic; pinned in walker's rustdoc
  + tested.
- The walker is **data-not-evaluator-extension** — no evaluator
  special-case for SubgraphSpec; preserves [BAKED] #1.
- `Engine::walk_share_scope()` (or equivalent named at G-CORE-9 brief
  time — AUDIT current name; if absent, BELONGS-NAMED-NOW for
  G-COMP-1).

**What "frozen" means here:** the walker lives in `benten-core` (not
`benten-engine` and not `benten-caps`); recipients walk the same BFS
the producer enumerated; canonical path carried in
`AuthorizationGrant.key_material`.

**Not frozen + WHY:** the internal queue+visited-set implementation —
opaque.

**Verification:** `cargo-public-api`; walker test pins.

**Escape valve:** evaluator special-case for SubgraphSpec = HALT
([BAKED] #1).

### 15(i) Revocation reach documentation

**Frozen surfaces:**
- A SECURITY-POSTURE.md section "Revocation reach in encryption-at-rest"
  documenting the four-bullet design constant (UCAN revocation cuts
  future serves / already-derived keys remain decryptable forever /
  Drop bundles are forever-valid / mitigation = tight `nbf`/`exp` +
  key rotation).
- **Consider for SECURITY-POSTURE Compromise #** ([PLAN] item 15(i);
  [RATIFIED-S&C] §R6).

**What "frozen" means here:** the section text + the Compromise-#
assignment (TBD at wave time) are the freeze. This is a
**documented-design-constant freeze, not a code-shape freeze**.

**Not frozen + WHY:** the wording of the section — minor edits
permitted in Composing; the design constant + the four bullets are
the freeze.

**Verification:** the doc presence test (some doc-coverage CI lane
asserts the section exists).

**Escape valve:** removing or contradicting the design constant in
Composing = HALT.

### 15(j) Resolver evaluation model = live-per-request

**Frozen surfaces:**
- The semantic: the resolver evaluates SubgraphSpec against current
  graph state on every request (NOT frozen-snapshot). UCANs gate
  sub-graph SHAPES that evolve.
- The IVM-cache-invalidation seam reuses G-CORE-4's CanonicalViews
  subscription.
- The offline-Drop-bundle is the ONLY frozen-snapshot path (by
  construction).

**What "frozen" means here:** a documented contract (not a wire-format
freeze). Pinned narratively in `docs/SECURITY-POSTURE.md` + by
resolver-side tests.

**Not frozen + WHY:** the internal IVM cache implementation — implementation
detail.

**Verification:** test pin demonstrating live-per-request evaluation
(write new in-scope content after UCAN issued → recipient sees it on
next request); doc-coverage CI lane.

**Escape valve:** changing live-per-request to frozen-snapshot semantics
in Composing = HALT.

---

## Appendix: items that are NAMED in [PLAN] §1.A.FROZEN but conservative-minimal-defers

For orchestrator triage. Each carries a HARD RULE 12 disposition.

| Item | Conservative disposition | Destination |
|---|---|---|
| `MerkleRangeProofBackend` trait shape (item 3) | BELONGS-NAMED-NOW G-COMP-1 §8-B-in-`benten-sync` | Trait isn't built at HEAD; freezing a phantom = overcommit; build + freeze intra-Composing |
| `Engine::get_node` `pub` → `pub(crate)` tighten (item 1) | BELONGS-NAMED-NOW G-COMP-1.1 v1-assessment-window | Consumer migration cost unmeasured; tighten unsafely = worse failure mode than soft-deferred freeze |
| `derive_step_without_info_tag_for_test` rename to `_internal` (item 15(f)) | BELONGS-NAMED-NOW G-COMP-1 + v1-assessment-window | Cosmetic test helper rename; safety-orthogonal |
| Wire-format Ben P-III decision (item 4) | SCHEDULED Ben decision-point at this wave | Plan SCHEDULES it; orchestrator surfaces the inventory; Ben signs the decision doc |
| `EncryptionClass` enum mint (item 15(e)) | CONDITIONAL on §8-CC consumers at wave-time | Audit at wave time; if consumers exist → mint at this wave; if not → BELONGS-NAMED-NOW G-COMP-1 |
| Hard-seal migration of `CapabilityPolicy` (item 8) | BELONGS-NAMED-NOW G-CORE-8.3 (already cited in INTERNALS.md §9) | Workspace-wide test-impl migration; soft-seal at v1-beta is sufficient |
| `Engine::walk_share_scope()` public name (item 15(h)) | AUDIT at wave time; BELONGS-NAMED-NOW G-COMP-1 if absent | Per current grep, the name doesn't appear; verify + name appropriately |
| Per-variant `#[non_exhaustive]` speculative applications (item 11) | LEAN MINIMAL on per-variant (only apply where a future arm is plausible) | The per-type defensive lean is the high-signal application |

---

## Appendix: my conservative-vs-orchestrator-triage flags

These are sections where I (Planner-B) chose conservative-minimal but
Planner-A's architectural-purist angle WILL likely lean toward broader
locking. Orchestrator should weight Ben's overall posture (HARD RULE 12
defaults to FIX-NOW; pre-v1 no-shims; "do it right not fast") when
adjudicating:

1. **Item 1 — `Engine::get_node` pub retention.** Planner-A will probably
   want the full tighten at v1-beta. My conservative read: consumer
   migration cost is unmeasured + the napi `getNode` consumer is real.
   Triage: either confirm migration is cheap + tighten now (Planner-A
   wins) OR keep soft-deferred (my position).

2. **Item 3 — backend-trait sub-decisions.** Planner-A will probably want
   each sub-fork explicitly RESOLVED in writing. I locked them
   "as-shipped" — same effective lock, less litigation. Triage: if
   Ben/Planner-A prefer explicit per-fork text, expand.

3. **Item 11 — `#[non_exhaustive]` per-variant applications.** Planner-A
   will probably apply it more aggressively. I leaned per-type defensive +
   per-variant minimal. Triage: defensive bias here is reasonable since
   the cost-asymmetry favors application; expand per-variant if Ben
   prefers maximum future flexibility.

4. **Item 15(e) — `EncryptionClass` enum.** Planner-A will probably
   want the enum minted unconditionally. My conditional posture defers
   the decision to wave-time verification of consumer existence. Triage:
   defaulting to mint is the more defensive call.

5. **Composing escape valve enforcement mechanism.** Planner-A may propose
   per-file `// FROZEN:` comment lints as a structural backstop. I
   considered + rejected this in favor of the cargo-public-api +
   #1204 parity gate (already mandated). My reason: comment-lints add
   ceremony without adding structural enforcement that the two
   real CI gates don't already provide.

---

## End of draft

This document is the v1-beta public-interface contract proposed by
Planner-B (conservative-minimal-freeze angle). It maps section-by-section
to [PLAN] §1.A.FROZEN's 15 items; section 15 has sub-clauses (a)-(j).
Each section names FROZEN SURFACES (minimal lock) + WHAT'S NOT FROZEN +
VERIFICATION MECHANISM + COMPOSING ESCAPE VALVE.

The orchestrator triages this against Planner-A's architectural-purist
draft into a single working artifact; the iterate-to-convergence council
then reviews + refines until Q5 termination (0 BLK + 0 MAJ).

**LOC estimate:** ~720 lines tracked.

**Branch:** `g-core-9/planner-b-conservative-minimal`
**Base:** `origin/main` @ `ae7cd3d5`
