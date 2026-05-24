# v1 Frozen Interface Contract — Phase-4-Meta-Core deliverable

> **Status: ARCHITECTURAL-PURIST DRAFT.** Planner-A angle (cleanest possible v1
> interface assuming complete freedom; willing to break more pre-v1 to get a
> cleaner end-state). Planner-B (conservative-minimal-freeze) is running in
> parallel on the opposite angle. The orchestrator triages both drafts into a
> single working artifact for the iterate-to-convergence council.

## Authority + scope

This document is the FROZEN-INTERFACE CONTRACT named by
`.addl/phase-4-meta/00-implementation-plan.md` §1.A.FROZEN (15-item spec, lines
111-148) as the **terminal deliverable of Phase-4-Meta-Core (Exit Criterion
C13)**. It is what Phase-4-Meta-Composing builds against; nothing in Composing
may alter a frozen surface, and a genuine need to do so is a
HALT-AND-SURFACE-TO-BEN event per methodology-r1-5 (NOT an orchestrator
autonomous Core re-open).

The freeze locks at git tag `phase-4-meta-core-close` (HEAD post-G-CORE-9
merge). The release cadence beyond is `phase-4-meta-close` → `v1-beta` →
(independent `ml-dsa`/`ml-kem` audit lands per C11c / NF-2 / C-GM-AUDIT) →
`v1-GM`, per `RATIFIED-pq-default-reframe-2026-05-19.md` §1.

**Authoritative provenance (read these BEFORE proposing any freeze mutation):**

- `.addl/phase-4-meta/00-implementation-plan.md` §1.A C1-C13 + §1.A.FROZEN
  (the 15-item spec).
- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
  (item 15 sub-clauses a-j).
- `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md` (items 6 + 14
  + the audit-gates-GM clause).
- `.addl/phase-4-meta/RATIFIED-crypto-agility-2026-05-18.md` (the
  multiformats-permanent framing distinct from algorithm choice).
- `.addl/phase-4-meta/RATIFIED-prework-forks-2026-05-18.md` (items 8 + 13 —
  sealed-CapabilityPolicy + engine-owns-its-own-tokio-runtime).
- `CLAUDE.md` baked-in items 1, 5, 7 (sealed-discipline refinement), 15, 17,
  18, 19.

## Verification mechanism (workspace-wide)

The freeze is enforceable, not merely declarative. Five structural backstops
fail CI on a frozen-surface mutation:

1. **`cargo-public-api` baseline regeneration + drift test** at
   `crates/benten-engine/tests/cargo_public_api_drift.rs` against
   `docs/public-api/benten-*.txt|json`. **FREEZE-WAVE FIX-NOW gap:** the
   current baselines (`docs/public-api/benten-caps.txt` 11 LOC etc.) are
   **seeded stubs from G20-A3, not real cargo-public-api output**. The
   G-CORE-9 wave MUST regenerate all 11 with `cargo public-api -p <crate>
   --simplified` against HEAD and commit them as the canonical v1 baseline,
   replacing the seed stubs. A future delta = CI failure on the test that
   diffs HEAD's output against the committed baseline.
2. **TS-side public-API parity gate (#1204)** for `@benten/engine`. **FREEZE-
   WAVE FIX-NOW gap:** the gate does NOT exist at HEAD (scripts/ contains only
   `codegen-errors.ts` + `drift-detect.ts` + the new
   `drift-detect-error-variant-mirror.ts` from #1342). G-CORE-9 MUST commit
   the `@microsoft/api-extractor`-or-equivalent JS-side public-shape diff
   workflow as the JS analog of `cargo-public-api` (per item 10 + the napi-r1-
   1 PQ-hybrid JS-shape widening). Without #1204, a TS-only public-API
   regression slips past the Rust gate.
3. **§3.5g cross-language rule-mirror scanners** at
   `scripts/drift-detect-error-variant-mirror.ts` (item 6 of §3.5g, the
   `feedback_pub_error_variant_first_class_mirror` ratification 2026-05-24).
   Every variant of any Rust error type crossing public/napi/wire surfaces
   has a first-class `ErrorCode` entry + TS mirror; the scanner runs in CI
   as a workspace test (`packages/engine/scripts/drift-detect-error-variant-
   mirror.test.ts`) and rejects any new asymmetric variant.
4. **Workspace `missing_docs` sweep** = `cargo nextest run -p
   phase-3-workspace-tests --test missing_docs_workspace` (per
   `feedback_workspace_missing_docs_test_invocation`). Mandatory pre-push +
   CI lane. Every `pub` item carries `///` docs at freeze; post-freeze
   additions inherit the gate.
5. **CATALOG_VARIANT_COUNT exhaustive-match dual-tripwire** at
   `crates/benten-errors/tests/stable_shape.rs::catalog_variant_count_matches_
   enum`. CATALOG_VARIANT_COUNT = **191** at HEAD `ae7cd3d5`. Adding/removing
   an `ErrorCode` variant without updating the list fails to compile; the
   list-without-match-arm fails the runtime length assertion.

**Architectural-purist sharpening (NEW pim-N candidate, item-cross-cutting):**
**Every `pub` declaration in a frozen module gets a `// FROZEN: re-open
requires Ben sign-off` comment**, and a CI lint scans for the marker.
Documentation alone is too easy to drift past during Composing-time agent
dispatches; a structural marker that CI can grep beats "trust the developer
read the freeze doc." (Origin instance: this very draft. Same-wave close per
§3.6h: every `pub` item enumerated in items 1-15 below carries the marker by
G-CORE-9 merge.)

---

## 1. §8-A visibility cluster — TIGHTEN applied atomically

**Frozen surfaces (post-tighten, per RATIFIED-prework-forks-2026-05-18.md
§8-A option (a)):**

- `crates/benten-engine/src/engine_crud.rs:139` — `Engine::get_node` →
  `pub(crate) fn read_node(&self, cid: &Cid) -> Result<Option<Node>,
  EngineError>` **renamed to `read_node` to remove the un-attributed
  semantic from the name surface**. The public `Engine::read_node_as`
  (line `engine_wait.rs:1115`) carries the principal-bearing semantic.
- `crates/benten-engine/src/engine_wait.rs:1056` — `Engine::put_node` →
  `pub(crate) fn put_node`.
- `crates/benten-engine/src/engine_wait.rs:1027` —
  `Engine::get_node_label_only` → `pub(crate) fn read_node_label_only`
  (renamed; un-attributed label-only read; engine-internal only).
- `crates/benten-engine/src/engine_wait.rs:1137` —
  `Engine::resolve_subgraph_cid_for_test` → **DELETED from the public
  surface entirely**. Test-only use sites move into `pub(crate)` impl-block
  helpers inside `crates/benten-engine/src/testing.rs` (Test-API module
  already exists; that's the canonical location for test-only surfaces).
  Public surface MUST NOT carry `_for_test` suffixes (architectural-purist:
  a `_for_test` `pub fn` is a red-flag — it's either real public API or
  belongs in the `testing` module).
- `crates/benten-engine/src/engine.rs:1628` — `Engine::caps() ->
  &EngineCapsHandle` stays `pub`; this is the canonical cap-mutation surface
  per the §4.69-ALREADY-SHIPPED ground-truth. No `Engine`-direct cap-
  mutation method may regress (freeze invariant; orchestrator-mechanical).

**What "frozen" means here:**
- Type-wise: the four methods MUST be `pub(crate)` after the tighten + rename;
  external callers MUST go through `read_node_as(principal, cid)`. The
  `cargo-public-api` baseline catches any post-freeze re-`pub`-ing.
- Behaviorally: the engine-internal callers (IVM, sync, view materialization,
  audit) keep using the un-attributed pathway with zero overhead — the
  tighten is a visibility-only change, not a behavior change.
- The `Engine::caps()` handle pattern is the SemVer-locked cap-mutation
  organizing principle (item 1's §4.69 sub-clause; ALREADY SHIPPED at origin/
  main `ed03729a`).

**What's NOT frozen:**
- The internal implementation behind `read_node_as` (the principal-routing,
  cap-policy consultation, namespace-partition lookup) may change post-v1 as
  long as the function signature + semantic contract is preserved.
- `pub(crate)` internal helpers and their `pub(super)` re-exports inside
  `benten-engine` are NOT frozen.

**Verification mechanism:**
- `cargo-public-api` baseline `docs/public-api/benten-engine.txt` (regenerated
  in this wave) carries the locked `pub` set for `crates/benten-engine`. The
  `Engine::get_node`/`put_node`/`get_node_label_only`/`resolve_subgraph_cid_
  for_test` symbols MUST NOT appear in the baseline. A re-`pub`-ing fails
  the drift test.
- Napi-side verification: the `bindings/napi/src/*.rs` files MUST be swept
  for any `engine.get_node(...)` / `engine.put_node(...)` call — refactor to
  `engine.read_node_as(principal, cid)` or
  `engine.transaction().put_node(...)`. The migration MUST be a single
  atomic G-CORE-9 commit-set (per §8-A's "applied atomically" requirement).

**Composing-phase escape valve:**
A Composing-time discovery that genuinely needs un-attributed `Engine::
read_node` access (NOT routable through `read_node_as` for a documented
reason) is a HALT-AND-SURFACE-TO-BEN event. Likely outcome on surfacing:
the `read_node_as(principal=ENGINE_INTERNAL_PRINCIPAL_CID, ...)` pattern
covers it without re-opening the freeze.

---

## 2. Class-B-β visibility — `read_node_as` is the canonical principal-bearing read

**Frozen surfaces:**
- `crates/benten-engine/src/engine_wait.rs:1115` — `Engine::read_node_as(
  &self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, EngineError>` —
  the public principal-bearing read API per CLAUDE.md baked-in #18. **The
  ONLY public read pathway for non-trusted principals.**
- `crates/benten-engine/src/engine.rs::Engine::call_as` — the existing
  Phase-2a precedent for principal-attributed call dispatch; signature
  parity with `read_node_as` is the canonical mirror.

**What "frozen" means here:**
- The `(principal: &Cid, cid: &Cid)` signature shape is locked. A
  `(principal: SomeNewType, cid: &Cid)` rewrite is a breaking change and
  requires re-opening the freeze.
- The semantic contract — "the engine consults `CapabilityPolicy::check_read`
  with the supplied `ReadContext { principal, target_cid, .. }` before
  returning the Node" — is part of the freeze.
- The TODO at `crates/benten-engine/src/engine_wait.rs:1115` referencing
  Class-B-β alpha-shaped stubs is closed (shipped at PR #184).

**What's NOT frozen:**
- The internal routing inside `read_node_as` (principal-DID parsing, UCAN
  chain validation, namespace-partition lookup, IVM-cache consultation) can
  evolve as long as the contract holds.

**Verification mechanism:**
- `cargo-public-api` baseline locks the signature.
- `crates/benten-engine/tests/r1_fp_3_class_b_beta_read_node_as.rs` (existing
  test family) carries the behavior pins.

**Composing-phase escape valve:**
A new public principal-bearing READ shape (e.g. `read_subgraph_as`,
`stream_node_as`) is an ADDITIVE pub item — fine if it CO-EXISTS with
`read_node_as` (Composing may add such surfaces inside the canonical pattern).
Removing or changing `read_node_as` is a HALT-AND-SURFACE event.

---

## 3. Backend-trait SemVer-locks (§4.60/4.61/4.62/4.63/4.64/4.43)

**Frozen surfaces (per §1.A.FROZEN item 3):**

| Sub-clause | Surface | Frozen shape |
|---|---|---|
| §4.60 | `crates/benten-graph/src/graph_backend.rs:238` `GraphBackend::Transaction::run<F, R>` | The closure-shaped transaction surface stays as-shipped; `run<F: FnOnce(&mut Transaction) -> R, R>` |
| §4.61 | `GraphBackend::snapshot()` + `register_subscriber()` | Both **DECIDED infallible** (`-> SnapshotHandle` and `-> ()`); fail-modes route through the typed `GraphError` channel on dependent operations, NOT through Result on these allocation methods |
| §4.62 | `crates/benten-graph/src/backends/blob_backend_trait.rs:120` `BlobBackend` | **DECIDED additive-default** (NOT a split). The trait carries `put_blob`/`get_blob`/`has_blob` with `Send + Sync + 'static`; future additive methods land as defaulted methods |
| §4.63 | `crates/benten-graph/src/backend.rs:306` `KVBackend` | **DECIDED sync** (NOT RPITIT). RPITIT adds 2024-edition feature-gate complexity v1-beta cannot absorb; future-Composing-async migration is an additive `AsyncKVBackend` trait |
| §4.64 | `crates/benten-sync/src/transport_trait.rs:85` `Transport` + `TransportEndpoint` + `TransportConnection` + `MerkleRangeProofBackend` (TBD-location-§8-B) | `Transport` family stays in `benten-sync` per §8-B (b). `MerkleRangeProofBackend` lands above storage in `benten-sync` (NOT in `benten-graph`); the trait surface is `pub` + `Send + Sync + 'static` |
| §4.43 | `WriteContext` / `ChangeEvent` / `GraphError::TxAborted` `#[non_exhaustive]` | **ARCHITECTURAL-PURIST RECOMMENDATION: APPLY `#[non_exhaustive]` to all three.** `WriteContext` at `crates/benten-graph/src/lib.rs:935` is currently missing the attribute (verified HEAD); `ChangeEvent` + `GraphError::TxAborted` need similar audit. The freeze MUST not ship without these three carrying `#[non_exhaustive]` |

**What "frozen" means here:**
- Trait method signatures + `Send + Sync + 'static` bounds + the `async` /
  `sync` posture per row.
- The `#[non_exhaustive]` per-row decision is structural — applying it post-
  v1 is breaking (the discipline § item 11 enforces).
- The trait's CRATE PLACEMENT (graph vs sync vs caps) is part of the freeze:
  moving a trait crate-side post-v1 is a wire/type-import break.

**What's NOT frozen:**
- Default-method bodies inside each trait may evolve.
- Internal helper types referenced only inside the trait's method signatures
  (e.g. iteration-result handle types) are governed by their own per-type
  freeze decisions (item 11 sweep).

**Verification mechanism:**
- `cargo-public-api` baselines for `benten-graph` + `benten-sync` + `benten-
  caps`. Adding/removing a non-defaulted trait method = baseline delta = CI
  failure.
- `cargo +stable clippy --workspace --all-targets -- -D warnings` catches
  `non_exhaustive` ABI-break candidates (the `non_exhaustive_omitted_patterns`
  lint).

**Composing-phase escape valve:**
A new defaulted trait method is ADDITIVE (cargo-public-api accepts; the
`#[non_exhaustive]` discipline propagates). A non-defaulted method addition
OR a signature change OR a `Send`/`Sync`/`'static` bound change is a HALT-
AND-SURFACE event.

---

## 4. D2 v1-canonical-bytes contract frozen as version 1 (P-III Ben decision-point)

**Frozen surfaces (the wire-byte lock; P-III scheduled here per §8-F):**

- `crates/benten-graph/src/backends/snapshot_blob.rs:125` —
  `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 2` (locked per §8-B-(i); already
  applied 2026-05-22 per ground-truth at HEAD).
- The Phase-1 canonical Node/Edge DAG-CBOR encoding family (CIDv1 +
  BLAKE3-256 + multihash `0x1e` + multicodec `0x71`), per CLAUDE.md baked-in
  #5.
- The MerkleRangeProof v2 wire shape (§8-B mode-(b)) — bytewise locked
  including field order and CBOR encoding.
- The per-chunk AEAD wire layout — chunk_size = `IROH_BLOCK_SIZE = 16384`
  (item 15(g)) — locked at `crates/benten-graph/src/aead_wrap.rs:56`. AAD
  layout binds `(chunk_index: u64, total_chunks: u64, plaintext_cid:
  Cid)` per §6 CI gate (13).
- Sentinel CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`
  remains the canonical Phase-1 golden fixture and MUST round-trip identically
  under v1 canonical bytes.

**What "frozen" means here:**
- BYTEWISE: a one-bit change to any encoded value (Node, Edge, SnapshotBlob,
  MerkleRangeProof, per-chunk AEAD ciphertext, UCAN-Varsig v1 header,
  AuthorizationGrant CBOR, Drop bundle CBOR) is a P-III re-decision Ben must
  make. Not orchestrator-autonomous, not a refactor side-effect.
- The format-version discriminator (`schema_version: u32`) is the migration
  affordance: bumping it is the explicit re-open mechanism.
- Old codepoints / old format versions are decode-supported FOREVER per item
  14 (never-strand-content).

**What's NOT frozen:**
- The Rust struct representation in memory (we may add `#[serde(skip)]`
  fields, change field ORDER inside the struct as long as `Serialize` order
  is locked, refactor the encoder internally).
- Newly-written content uses whatever new format the schema-bump introduces;
  no in-place migration on existing redb partitions (immutable content-
  addressed objects per item 14).

**Verification mechanism:**
- Byte-pin tests under `tests/canonical_bytes_v1_*.rs` (FREEZE-WAVE FIX-NOW:
  add per-shape byte-pin where it doesn't exist — the architectural-purist
  shopping-list is: SnapshotBlob v2, MerkleRangeProof v2, per-chunk-AEAD,
  UCAN-Varsig v1 header, AuthorizationGrant CBOR, Drop bundle CBOR,
  encryption envelope per codepoint, signature envelope per codepoint).
  Each test loads a hex-pinned canonical bytes string + asserts encode +
  decode round-trip + CID stability.
- `crates/benten-graph/tests/redb_backend_*.rs` family covers the redb
  on-disk format.

**Composing-phase escape valve:**
ANY frozen-byte mutation is a P-III Ben decision-point — HALT-AND-SURFACE-
TO-BEN, with options + prediction per the standing surface discipline. The
mutation lands in Core re-open (a Core re-open IS allowed; it is announced,
deliberate, never silent).

---

## 5. `WriteContext` shape frozen (#989 / G-CORE-1 canary output)

**Frozen surfaces:**

- `crates/benten-graph/src/lib.rs:935` `pub struct WriteContext { label,
  is_privileged, authority, namespace_did }` — the **G-CORE-1 canary-shipped
  shape**. All four fields `pub`; `Default` impl carries `namespace_did =
  None` (the legacy un-namespaced keyspace; byte-identical to pre-#989).
- `WriteContext::with_namespace_did(self, did: Cid) -> Self` builder.
- `WriteContext::namespace_did(&self) -> Option<&Cid>` accessor.
- The C1 cross-DID non-leak invariant (`crates/benten-graph/src/lib.rs:925`
  doc-block) — structural: keys under per-DID prefix derived from
  `Cid::as_bytes()`; never collide with legacy `n:`/`e:`/`es:`/`et:` prefixes.

**What "frozen" means here:**
- Field-set frozen: no removal, no rename, no type-change of any of the four
  fields.
- `Default` semantics frozen: `namespace_did = None` means the legacy
  keyspace (byte-identical to pre-#989).
- Builder + accessor names frozen.
- The cross-DID non-leak invariant is part of the type's contract.

**ARCHITECTURAL-PURIST RECOMMENDATION:** `WriteContext` MUST carry
`#[non_exhaustive]` at the freeze (verified MISSING at HEAD `crates/benten-
graph/src/lib.rs:934`). Adding it post-v1 is breaking; adding it pre-freeze
is the cheap, correct call. This is the §1.A.FROZEN item 11 + item 5
coupling — they MUST close together in the G-CORE-9 atomic wave.

**What's NOT frozen:**
- The set of `Default::default()` field values beyond `namespace_did` may
  evolve (e.g. adding a defaulted `tenant_id: Option<TenantId>` post-v1 via
  `#[non_exhaustive]` discipline — that's why the attribute is mandatory).

**Verification mechanism:**
- `cargo-public-api` baseline `docs/public-api/benten-graph.txt` (regenerated
  in this wave).
- `crates/benten-graph/tests/tf1_write_context_namespace_did_*.rs` (G-CORE-1
  canary pin family).

**Composing-phase escape valve:**
Field addition is ADDITIVE-with-`#[non_exhaustive]`; field removal/rename
is HALT-AND-SURFACE.

---

## 6. #1300 signature + #1301 encryption boundaries frozen (PQ-hybrid DEFAULT + full swap matrix)

Per `RATIFIED-pq-default-reframe-2026-05-19.md` §1-2 + §1.A.FROZEN item 6 +
CLAUDE.md baked-in #5 (the multiformats-permanent framing).

**Frozen surfaces (the FULL CODEPOINT TABLE; algorithms behind = SWAPPABLE
within the framing):**

- `crates/benten-crypto-suite/src/codepoint.rs::SigCodepoint` —
  - `HYBRID_ED25519_MLDSA65 = 0x0001` (v1-beta **DEFAULT**; NF-4 concatenated/
    committing/strip-resistant per IETF `draft-ietf-lamps-pq-composite-sigs-
    18`).
  - `CLASSICAL_ED25519 = 0x0002` (non-default downgrade).
  - `HYBRID_MLDSA65_SLHDSA = 0x0003` (NF-1 end-state; live impl at G-CORE-3c
    swap-matrix; reserved-but-named codepoint).
- `crates/benten-crypto-suite/src/codepoint.rs::CipherSuiteCodepoint` —
  - `HYBRID_X25519_MLKEM768 = 0x647a` (v1-beta **DEFAULT**; X-Wing-style
    combiner vendored at ~30 LOC; ChaCha20-Poly1305 bulk).
  - `CLASSICAL_X25519_ONLY = 0x6400` (non-default classical-only downgrade).
  - `HYBRID_MLKEM768_HQC = 0x647b` (NF-1 KEM PQ⊕PQ end-state; reserved-named-
    only; build-trigger = FIPS 207 final).
  - `PURE_PQ_MLKEM768_ONLY = 0x647c` (pure-PQ swap-matrix arm; **typed-
    rejected by default — gated by `AUDIT_LANDED_PURE_PQ_FLAG` per the C11b
    safety gate**; minted 2026-05-24 per Ben morning queue item 1).
  - `NO_ENCRYPTION_PUBLIC_CLASS = 0x0000` (non-default plaintext-partition).
- `crates/benten-crypto-suite/src/swap_matrix.rs::SwapMatrix` constructors:
  `v1_beta_default()`, `classical_only()`, `no_encryption_public_class()`,
  `non_pq_encryption()`, `try_pure_pq_sole_trust_path() ->
  Result<Self, SwapMatrixError>`. **The `try_*` shape encodes the C11b safety
  gate at the type level** — pure-PQ cannot be a sole-trust-path config
  without an explicit `AUDIT_LANDED_PURE_PQ_FLAG = true` flip.
- The hybrid-sig wire envelope (UCAN-Varsig v1 header) — both Ed25519
  signature AND ML-DSA-65 signature travel together; both MUST verify; the
  CBOR layout is frozen bytewise per item 4.
- The X-Wing-style KEM combiner — vendored at ~30 LOC inside the integration
  crate; never forked from upstream X-Wing crate (per never-fork-never-
  reimplement-primitives baked-in #5); Benten owns the version bump.
- The hash-codepoint dispatch surface — BLAKE3-256 default (multihash `0x1e`)
  + SHA-512/256 (`0x1015`) + SHA3-256 (`0x16`) as pre-blessed agile
  fallbacks.

**Multi-device key-wrap/recovery envelope shape:**
- Frozen as part of #1301 per item 6. Recovery PROTOCOL choice (Shamir / social
  / hardware / MLS-style) stays G-COMP-3 v1-assessment-window; the ENVELOPE
  SHAPE around the wrap is frozen here so a recovery-protocol choice doesn't
  require re-opening the freeze.

**What "frozen" means here:**
- Codepoint table is BYTEWISE PERMANENT. Old codepoints supported FOREVER per
  item 14 (never-strand-content). New codepoints land additively at unused
  values.
- The framing (multiformats CIDv1 + multihash + multicodec + `did:key` +
  UCAN-Varsig + codepoint-dispatched suite-selector) is the PERMANENT
  commitment — algorithms behind ANY codepoint are swappable as long as the
  framing holds.
- Sizes are NEVER hardcoded. ML-DSA ~1952 B key / ~3309 B sig + ML-KEM-768
  ciphertext dimensions are exercised on the v1-beta DEFAULT path — no
  Ed25519-shaped (32 B-key / 64 B-sig) assumption survives anywhere in the
  workspace.
- Typed-reject discipline frozen: unknown codepoints fail-closed with typed
  `UnsupportedAlgorithm::{Signature, CipherSuite, Hash}` — NEVER silent
  fallback (Veilid / MLS / Nostr NIP-44 precedent; age's silent-ignore is the
  rejected outlier).
- The hybrid construction = NF-4 concatenated/committing/strip-resistant
  (both MUST verify). The safety invariant: PQC is NEVER the sole trust path
  (the classical half is the audited security floor — exactly what makes
  v1-beta shippable BEFORE the independent audit lands).

**ARCHITECTURAL-PURIST RECOMMENDATION:** `SigCodepoint` + `CipherSuiteCodepoint`
+ `HashCodepoint` are wrapper structs around `u16` (`pub struct
SigCodepoint(pub(crate) u16)`). This is the right shape — `#[non_exhaustive]`
doesn't apply to tuple structs with private fields (which `SigCodepoint`
effectively is via `pub(crate) u16`). **The `from_raw(raw: u16) -> Self`
constructor is `pub` (line 64 of codepoint.rs); architectural-purist accepts
this for deserializer use but mandates the construction is paired with
`resolve()` → `Result<(), UnsupportedAlgorithm>` at the dispatch site** —
i.e. you can construct any codepoint but you can't USE one that doesn't
typed-resolve. This is the C11b safety property and MUST be enforced
end-to-end at every dispatch site (auditable workspace-wide).

**What's NOT frozen:**
- The internal Rust implementation of any algorithm behind a codepoint — we
  may bump `ml-dsa` / `ml-kem` / `x25519-dalek` / `chacha20poly1305` crate
  versions freely. The CODEPOINT is the contract; the CRATE is the
  implementation behind it.
- The integration-crate glue logic (concat layout, HKDF info-tag binding,
  envelope serialization) is internal and may refactor as long as the wire
  bytes per codepoint stay byte-identical.

**Verification mechanism:**
- The `cargo-public-api` baselines for `benten-crypto-suite` + `benten-caps`
  + `benten-graph` lock the codepoint-typed constructors.
- G-CORE-3c's conformance test corpus
  (`crates/benten-crypto-suite/tests/conformance_*.rs`) exercises all 7
  swap-matrix arms — these tests are part of the freeze (CI lane).
- The P2P-interop conformance lane (item 14) MUST run on every push and pass
  for v1-beta to ship.

**Composing-phase escape valve:**
- Algorithm bump within a codepoint = NOT a freeze break (cargo-public-api
  ignores; conformance tests catch any breaking-byte-shape regression).
- New codepoint addition = ADDITIVE; lands at unused value; old codepoints
  decode forever.
- Removing a codepoint OR repurposing one OR changing the typed-reject-on-
  unknown discipline = HALT-AND-SURFACE.
- `AUDIT_LANDED_PURE_PQ_FLAG` flip from `false` to `true` is a Ben decision-
  point gated by C11c / NF-2 / C-GM-AUDIT (the independent audit landing).
  Orchestrator NEVER flips it autonomously.

---

## 7. §4.33 legacy `module_ecosystem::install_plugin*` path DELETED

**Frozen surfaces:**
- `crates/benten-platform-foundation/src/module_ecosystem.rs` —
  `module_ecosystem::install_plugin*` family **REMOVED** at Core opening
  wave (G-CORE-0; verified post-merge per HEAD: only
  `new_version_available_code` helper remains; the test file
  `tf_g_core_0_legacy_install_path_deletion_4_33.rs` carries the absence
  pin).

**What "frozen" means here:**
- DELETION, not deprecation. Per HARD RULE 12 clause-(a): two install paths
  with different security envelopes cannot coexist into the freeze.
- A future re-introduction is a Ben re-open (NEW pub item, distinct name).
- `cargo-public-api` baseline does NOT contain any
  `module_ecosystem::install_plugin*` symbol. The absence is structural.

**ARCHITECTURAL-PURIST EXTENSION:** Apply the same DELETION discipline to
ALL legacy/deprecated surfaces workspace-wide at the freeze. Survey:
- `grep -rn "deprecated\|legacy\|_v1_compat" crates/` — every match is a
  freeze-wave triage: delete-or-justify-keeping.
- Any `TODO(phase-N): remove this` older than 2026-05 — same triage.
- The `crates/benten-engine/src/engine_wait.rs:1137`
  `resolve_subgraph_cid_for_test` is one such target (handled under item 1).

**What's NOT frozen:**
- The replacement public install pipeline (`benten_platform_foundation::
  plugin_lifecycle::install_plugin`) and its sub-modules are governed by
  their own freeze item (covered by items 8 + 12).

**Verification mechanism:**
- `tf_g_core_0_legacy_install_path_deletion_4_33.rs` absence pin.
- `cargo-public-api` baseline does not contain the deleted symbols.

**Composing-phase escape valve:**
A genuine need to re-introduce a deleted install path is a HALT-AND-SURFACE
event with strong predisposition AGAINST re-introduction (per Compromise #
documenting why deletion was chosen).

---

## 8. `benten-caps` v1-API forks decided + recorded (SEALED `CapabilityPolicy`)

**Frozen surfaces (per `RATIFIED-prework-forks-2026-05-18.md` §8-E option
(a) + §1.A.FROZEN item 8):**

- `crates/benten-caps/src/policy.rs:341` `pub trait CapabilityPolicy:
  Send + Sync` — the canonical capability-policy trait.
- The three new G-CORE-8 §8-E defaulted hooks (`check_install_consent` /
  `check_per_delegation` / `check_write_with_audience`) — ADDITIVE
  defaulted methods; existing impls + `Arc<dyn CapabilityPolicy>` boxing
  compile unchanged.
- `crates/benten-caps/src/policy.rs::sealed_marker::SealedCapabilityPolicy`
  — the sealed-discipline marker trait.
- `CapWriteContext` + `ReadContext` + `PendingOp` (`crates/benten-caps/src/
  policy.rs:103, 154, 247`) — the cap-policy context types.
- `#993` `CapabilityPolicy` sealed-discipline frozen per §8-E (a) DECIDED.
- `#886` `[features]` decision recorded (per item 8).
- `#1005` `actor_hint` shape recorded.
- `#883b` prod-dep-edge recorded.
- `#887b` `check_read` default-impl policy: defaults to admit-all (the
  Phase-1 baseline; production policies override).
- `Engine::caps()` returns `&EngineCapsHandle` (organizing principle (a)
  ALREADY SHIPPED per origin/main `ed03729a`; freeze invariant per
  §1.A.FROZEN item 1 sub-clause: no `Engine`-direct cap-mutation method may
  exist).

**ARCHITECTURAL-PURIST RECOMMENDATION — HARDEN THE SOFT-SEAL TO A TRUE
PRIVATE SUPERTRAIT AT THE FREEZE.** Per `crates/benten-caps/src/policy.rs:14-
67`, the v1-beta posture is a "soft-seal" (the `SealedCapabilityPolicy`
marker exists but is NOT a private supertrait of `CapabilityPolicy`; external
impls compile and surface as "unsealed" only via a workspace-introspection
audit). The reason given: "≥20 test sites would need a workspace-wide
migration." **Architectural-purist position: do the workspace-wide migration
in the G-CORE-9 freeze wave** (the same atomic wave that applies item 1's
visibility tighten). Cost: ~20 test-file edits each adding `impl
SealedCapabilityPolicy for X {}`. Benefit: the seal is enforced by `rustc`,
not just by introspection — an external impl FAILS to compile post-freeze,
which is exactly the v1 guarantee §8-E (a) DECIDED. The cited HARD-RULE-12
BELONGS-NAMED-NOW `G-CORE-8.3 follow-up wave` is the correct destination
for the soft-seal hardening per the current `policy.rs` doc-block; the
architectural-purist call is to **roll G-CORE-8.3 INTO G-CORE-9** rather than
ship v1-beta with a soft-seal. Why: the freeze is the LAST opportunity to
do this without a SemVer break — post-v1 hardening is breaking; pre-freeze
hardening is the cheap, correct call. Reject the "20 test sites is too much"
argument per `feedback_agent_economics_prefer_thorough_cleanup`: agent
dispatch makes large mechanical migrations cheap.

**What "frozen" means here:**
- Trait shape (signature, defaulted-vs-required, return types) is locked.
- Sealed discipline is HARD-ENFORCED via private supertrait (architectural-
  purist) OR documented + introspection-audited (conservative — Planner-B
  may stay here). Either way, the SEAL IS THE CONTRACT.
- Object-safety preserved (`Arc<dyn CapabilityPolicy>` boxing compile-test
  pin at `crates/benten-caps/tests/object_safety_*.rs`).
- All three new G-CORE-8 hooks freeze at their CURRENT defaulted signature.

**ARCHITECTURAL-PURIST RECOMMENDATION — apply `#[non_exhaustive]` to
`CapWriteContext` + `ReadContext`** at the freeze. They are context structs
likely to grow new fields in Composing (e.g. tenant context, request-ID
trace). Adding fields post-v1 is breaking; the attribute is the cheap,
correct affordance.

**What's NOT frozen:**
- The `NoAuthBackend` / `GrantBackedPolicy` / `UcanGroundedPolicy` impl
  bodies may evolve (they are concrete implementations).
- Internal helper functions under `benten_caps::evaluator_delegation::*` are
  `pub(crate)` and not frozen.

**Verification mechanism:**
- `cargo-public-api` baseline `docs/public-api/benten-caps.txt`.
- `object_safety` test compile-pin.
- Workspace-wide audit script (or rustc compile-fail) on any external
  `impl CapabilityPolicy for X` without `impl SealedCapabilityPolicy for X`
  — architectural-purist: hard-fail at compile via private supertrait;
  conservative: introspection-audit lane.

**Composing-phase escape valve:**
- New defaulted trait method = ADDITIVE; fine.
- New required trait method = HALT-AND-SURFACE (breaks every impl).
- Removing or changing a hook signature = HALT-AND-SURFACE.

---

## 9. `cargo-public-api` baselines regenerated + committed as v1 surface

**Frozen surfaces:**
- `docs/public-api/benten-caps.txt`
- `docs/public-api/benten-core.txt`
- `docs/public-api/benten-crypto-suite.txt` (**FREEZE-WAVE FIX-NOW: doesn't
  exist at HEAD; MUST be added — verified `ls docs/public-api/` shows the
  crate is missing from the baseline set**)
- `docs/public-api/benten-drop.txt` (**FREEZE-WAVE FIX-NOW: doesn't exist at
  HEAD; MUST be added; benten-drop is a new Phase-4-Meta-Core crate**)
- `docs/public-api/benten-dsl-compiler.txt`
- `docs/public-api/benten-engine.txt`
- `docs/public-api/benten-errors.txt`
- `docs/public-api/benten-eval.txt`
- `docs/public-api/benten-graph.txt`
- `docs/public-api/benten-id.json`
- `docs/public-api/benten-ivm.txt`
- `docs/public-api/benten-platform-foundation.txt` (**FREEZE-WAVE FIX-NOW:
  doesn't exist at HEAD; MUST be added — `crates/benten-platform-foundation/`
  is a public crate post-Phase-4-Foundation**)
- `docs/public-api/benten-renderer-tauri.json`
- `docs/public-api/benten-sync.json`

**What "frozen" means here:**
- Each baseline is the AUTHORITATIVE list of every `pub` symbol the crate
  exports at `phase-4-meta-core-close`. Any post-freeze delta = CI failure
  on the drift test.
- The baseline format (`.txt` vs `.json`) per crate is locked.
- A NEW pub item post-v1 requires explicit baseline-update + manifest-review
  + Ben sign-off (the cargo-public-api gate is the freeze's structural backstop).

**ARCHITECTURAL-PURIST RECOMMENDATION:** the seeded-stubs at HEAD
(`benten-caps.txt = 11 LOC`, etc.) are **NOT REAL BASELINES** — they're
G20-A3 placeholder comments. The G-CORE-9 wave MUST run `cargo public-api -p
<crate> --simplified --omit blanket-impls` for every workspace crate and
commit the full output as the canonical v1 baseline. Without this, the gate
is a placebo (the drift test will pass against the placeholder regardless of
real public-API mutations). This is **the single most load-bearing freeze
mechanism** — the gap MUST close.

**What's NOT frozen:**
- The cargo-public-api tool version (carried in `Cargo.toml` dev-deps); tool
  bumps may produce slight diff in baseline serialization (the regeneration
  ritual handles this).
- The internal symbols (`pub(crate)`, `pub(super)`, private) are not in the
  baseline.

**Verification mechanism:**
- `crates/benten-engine/tests/cargo_public_api_drift.rs` runs the drift
  detection on every CI lane.
- `cargo +stable clippy --workspace --all-targets -- -D warnings` orthogonal
  catch on missing-docs / unused-pub.

**Composing-phase escape valve:**
A new pub item = baseline-update PR; reviewed against the freeze contract;
Composing may add but never remove or rename without HALT-AND-SURFACE.

---

## 10. TS/JS `@benten/engine` public API frozen (incl. #1204 parity gate)

**Frozen surfaces:**

- `packages/engine/src/errors.generated.ts` — 193 `BentenError`-extended
  classes (verified count at HEAD post-#1342); regenerated from `docs/ERROR-
  CATALOG.md` via `scripts/codegen-errors.ts`; **the auto-generation contract
  itself is frozen** (regen MUST produce a byte-identical file given the
  same input).
- `packages/engine/src/types.ts` — typed-call input/output shapes, incl. the
  `TypedCallInputShapes`/`TypedCallOutputShapes` `ed25519_*` / `keypair_*` /
  `did_resolve` arms + `ManifestSignature`. **No hardcoded Ed25519-shaped (32
  B-key / 64 B-sig) assumption** — the hybrid ML-DSA-65 (~1952 B / ~3309 B)
  + ML-KEM dimensions mirror atomically Rust↔TS per napi-r1-1.
- `packages/engine/src/index.d.ts` — the TS module declaration file;
  generated from napi-rs via the build pipeline.
- `packages/engine/src/stream.ts` — `StreamHandle` + the **`next()` final
  shape decided per G-CORE-10 PR-B** (sync-vs-Promise resolved; whatever
  PR-B shipped is the v1 frozen shape; verified post-#1340 batch merge).
- `packages/engine/src/atrium.ts` — `Atrium` public class.
- `packages/engine/src/subscribe.ts` — `SubscribeHandle`.
- `packages/engine/src/identity.ts` — `Keypair` / `VerifiableCredential` /
  `DeviceAttestation` JS-side wrappers.
- `packages/engine/src/manifest.ts` — `ManifestSignature` + plugin-manifest
  JS shapes (PQ-hybrid sized).
- `packages/engine/src/sandbox.ts` — SANDBOX JS API.
- `packages/engine/src/wait.ts` — WAIT JS API.

**FREEZE-WAVE FIX-NOW: #1204 JS-side public-API parity gate.** Verified at
HEAD: NO gate exists (no `api-extractor` / `api-report` / equivalent at
`scripts/` or `packages/engine/`). The G-CORE-9 wave MUST commit the workflow
per item 10's three-stage sequential placement: BUILT in G-CORE-10
(regenerate `index.d.ts` + author the parity gate workflow); COMMITTED in
G-CORE-9 (recorded here); EXECUTING in the §4 CI lane. **The likely
implementation: `@microsoft/api-extractor` running against `packages/engine/
dist/index.d.ts` with a committed `etc/engine.api.md` baseline.** A delta
= CI failure parallel to `cargo-public-api`.

**What "frozen" means here:**
- The exported TS class/type/function names are locked.
- The PQ-hybrid sizing (no Ed25519-shaped assumption) is locked at the type
  level — e.g. `keypair_publicKey: Uint8Array` with no length pin in the
  type, and runtime length-check tests at `packages/engine/src/manifest.
  test.ts` exercise the hybrid-sized inputs.
- The `errors.generated.ts` regen-determinism is part of the contract (a
  re-codegen produces zero diff).

**ARCHITECTURAL-PURIST RECOMMENDATION — close the `errors.generated.ts` ↔
catalog ↔ Rust `ErrorCode` enum loop atomically.** §3.5g item 6 ratification
2026-05-24 ships the drift-detect scanner; the freeze MUST verify all three
sides at parity at HEAD: `CATALOG_VARIANT_COUNT = 191` Rust-side = N entries
in `docs/ERROR-CATALOG.md` = N classes in `errors.generated.ts`. (Verified
at HEAD: 193 TS classes vs 191 Rust variants — **architectural-purist
FIX-NOW**: investigate the +2 TS-side classes; either they're legitimate
generic envelope classes (`BentenError` base + `BentenInternalError` etc.)
or there's drift to close.)

**What's NOT frozen:**
- The internal Rust→napi bridging logic.
- The `bindings/napi/src/*.rs` rust source (governed by item 9 cargo-public-
  api baselines for `bindings-napi` if/when that crate gets one; today napi
  is workspace-only with no cargo-public-api baseline).

**Verification mechanism:**
- #1204 JS-side parity gate (FREEZE-WAVE FIX-NOW to author).
- `scripts/drift-detect-error-variant-mirror.ts` enforces Rust↔TS error
  parity.
- `packages/engine/src/*.test.ts` carries behavior pins.

**Composing-phase escape valve:**
- New TS export = ADDITIVE; baseline-update PR; reviewed against the freeze.
- Removing/renaming = HALT-AND-SURFACE.

---

## 11. META #907 `#[non_exhaustive]` sweep frozen as ONE coherent freeze-wave

**Frozen scope (per v1-api-freeze-r1-3 + ARCHITECTURAL-PURIST MAXIMALIST
extension):**

The G-CORE-9 wave enumerates EVERY public enum + struct workspace-wide and
makes a per-item apply-or-D8-carve-out decision. Verified at HEAD: **158
total `pub enum` across `crates/`; only 48 currently carry
`#[non_exhaustive]`** (30% coverage). The freeze MUST close this gap.

**Architectural-purist position: APPLY `#[non_exhaustive]` UNIVERSALLY**
unless a D8-carve-out has a documented structural reason. Carve-out
candidates (the cases where `#[non_exhaustive]` is structurally WRONG):

- `benten-caps::Scope` (`crates/benten-caps/src/scope.rs:46`) — EXACTLY two
  arms per §1.A.FROZEN item 15(c); `#[non_exhaustive]` would defeat the
  exhaustive-match structural pin. **CARVE-OUT (documented; per the doc-
  block at scope.rs:33-44).**
- Any enum whose variant set is INTENTIONALLY closed (e.g. boolean-equivalent
  binary discriminants, RFC-mandated codepoint sets that can't expand
  without protocol bumps).

**The enumerated must-apply set** (from §1.A.FROZEN item 11 + workspace
verification at HEAD):

- `benten-engine`: `UserViewInputPattern` / `TraceStep` / `Transport` (in
  `thin_client.rs`) / `AtriumMode` / `SuspensionOutcome` / `DelegationResolution`
  / `NextChunkPoll` / `StreamCursor` / `SubscribeCursor` /
  `WriteBoundaryChainOutcome` / `ManifestEnvelopeRecheckOutcome` /
  `ManifestVerifyMode` — **12+ verified missing the attribute at HEAD**.
- `benten-core`: `WriteAuthority` (per item 11 spec) + the new `RestrictedSpec`
  variants in `crates/benten-core/src/subgraph_spec/spec.rs:126`.
- `benten-ivm`: `AlgorithmError`.
- `benten-sync`: §4.71 5-enum cluster.
- `benten-caps`: `TypedCapGroup` + `CapWriteContext` (struct) + `ReadContext`
  (struct) + `PendingOp` (already has it; verified).
- `benten-graph`: `WriteContext` (struct) + `ChangeEvent` (enum) +
  `GraphError::TxAborted` (struct variant) — **all three verified MISSING at
  HEAD per item 5 audit**.
- `benten-drop`: `DropBundleVersion` / `DropContentMode` / `DropBundleError`
  / `EnvelopeSigError`.
- `benten-crypto-suite`: `SwapMatrixError` / `UnsupportedAlgorithm`.
- `benten-errors`: `ErrorCode` (verified — has it per Phase-4-Foundation
  freeze).

**What "frozen" means here:**
- `#[non_exhaustive]` per-item decision is BAKED into the type — a future
  variant addition is a minor version bump, NOT a SemVer break.
- The carve-out list is part of the freeze (a carve-out can't be silently
  added later; an enum that ships exhaustive-by-design is exhaustive forever
  unless re-opened).
- The `cargo +stable clippy --workspace --all-targets -- -D warnings`
  catches the `non_exhaustive_omitted_patterns` lint on consumers.

**What's NOT frozen:**
- The variant SET inside the enum (the whole point of `#[non_exhaustive]` is
  permitting additive future variants).
- The struct FIELD SET (same — additive future fields).

**Verification mechanism:**
- A workspace-wide script (FREEZE-WAVE FIX-NOW to author) that greps every
  `pub enum` + `pub struct` and asserts each carries `#[non_exhaustive]` OR
  is in the carve-out registry. Lives at `tests/non_exhaustive_sweep.rs`;
  fails CI on any new public enum/struct lacking the attribute + the
  carve-out justification.
- `cargo-public-api` baseline catches the attribute (it's part of the
  declaration shape).

**Composing-phase escape valve:**
- New `pub enum` / `pub struct` in Composing MUST default to
  `#[non_exhaustive]` per the freeze policy; carve-out requires explicit
  registry entry + Ben sign-off.
- Removing `#[non_exhaustive]` from a frozen item = HALT-AND-SURFACE.

---

## 12. G-CORE-8 security-surface public-shape lock

**Frozen surfaces (per §1.A.FROZEN item 12 + security-r1-2):**

- `crates/benten-engine/src/manifest_envelope_recheck.rs:80` `pub enum
  ManifestEnvelopeRecheckOutcome { NotApplicable, UnresolvedDeny, Admitted,
  OutsideEnvelope { offending_plugin_did, cap_pattern } }` — **all four
  variants frozen** including the post-rename `UnresolvedDeny` semantic.
- The `Admitted` arm's structural invariant (security-r1-2 frozen): returned
  ONLY on a positively-verified envelope/chain match. **`outcome_to_row_
  reject` at `manifest_envelope_recheck.rs:124-144` MUST NOT collapse a
  non-positive outcome to `Ok(())`** — the structural property is part of
  the freeze.
- `pub trait ManifestEnvelopeRechecker` (`crates/benten-engine/src/
  manifest_envelope_recheck.rs:144`-ish) — the port interface; method
  signatures frozen.
- `NoopManifestEnvelopeRechecker` semantics — returns `UnresolvedDeny` for
  every input (per the §4.36 fail-CLOSED flip; the rename is NOT cosmetic).
- The DEFAULT engine-builder behavior — `ProductionManifestEnvelopeRechecker`
  is auto-wired (NOT opt-in) per security-r1-1 BLOCKER closure.
- Empty/sentinel `<unresolved-peer>` peer-DID MUST deny (never admit) at
  recheck AND §4.25 sync-hydrate (security-r1-2).

**Any §4.40 key-at-rest public type** (if it lands at HEAD; verified at
HEAD: §4.40 is a per-DID key-at-rest seam — placeholder until C1+C2 deliver
confidentiality; the public type doesn't ship until the §989→§1301 substrate
is complete which IS the C1+C2 deliverable that closes this).

**ARCHITECTURAL-PURIST RECOMMENDATION:** apply `#[non_exhaustive]` to
`ManifestEnvelopeRecheckOutcome` (verified MISSING at HEAD). Adding a fifth
recheck-outcome variant post-v1 is breaking; the attribute is the cheap,
correct affordance. (Note: per item 8 sealed-policy discipline, this enum
is part of the recheck PORT contract; consumers `match` against it and
adding a variant would break every `match` site.)

**What "frozen" means here:**
- The four variants + their semantics are bytewise + behaviorally locked.
- The fail-CLOSED `UnresolvedDeny` arm IS the load-bearing security property
  — re-introducing admit-on-unresolved is a HALT-AND-SURFACE event.
- The DEFAULT-builder wiring is part of the freeze; an opt-in posture would
  be a v1 BLOCKER regression.

**What's NOT frozen:**
- The internal logic inside `ProductionManifestEnvelopeRechecker` (how it
  consults `PluginLibrary` + `UserDidRegistry`) may evolve.

**Verification mechanism:**
- `crates/benten-engine/tests/g_core_8_manifest_envelope_recheck_*.rs` family
  carries the would-FAIL pins (incl. the
  `noop_rechecker_admit_everything_is_security_r1_1_would_fail_baseline`
  honest-name pin per mr-2 fix-up).
- `cargo-public-api` baseline locks the enum + trait shape.

**Composing-phase escape valve:**
- New variant = HALT-AND-SURFACE (every consumer's `match` breaks).
- Changing fail-CLOSED semantics = HALT-AND-SURFACE-WITH-STRONG-DEFAULT-NO.

---

## 13. Engine↔host runtime-ownership boundary frozen = bridged-dual-runtime

**Frozen surfaces (per `RATIFIED-prework-forks-2026-05-18.md` §8-C option (2)
+ §1.A.FROZEN item 13):**

- The engine OWNS its own tokio runtime — no `tauri::Builder::with_runtime`,
  no shell-runtime-handle threading through the engine API.
- `crates/benten-renderer-tauri/src/lib.rs` — the renderer crate has ZERO
  `tauri`/`tokio` deps (per the swappability thesis; verified by compile-test
  pin).
- `crates/benten-renderer-tauri/src/lib.rs:127` `pub const IPC_METHODS:
  &[IpcMethod]` — the explicit method-name allowlist. **IPC_METHODS IS THE
  IPC SURFACE** — webview cannot invoke a method not in the const-allowlist.
- `crates/benten-platform-foundation/src/materializer.rs:552` `pub trait
  Renderer: Send + Sync` with `render(&MaterializerOutput) -> Result<(),
  RenderError>` + `backend_name() -> &'static str`. **Trait surface carries
  NO transport-specific methods** (compile-test pin asserts a Tauri runtime
  type can't leak through the seam).
- `crates/benten-engine/src/thin_client_bridge.rs:86` `pub struct
  ThinClientBridge` — the §4.22 thin-client bridge; principal-resolution
  semantics frozen per G-CORE-8.

**The three IPC contracts that MUST lock CONSISTENT (per deployment-r1-6):**
- (a) The #838 IPC-surface shape (the const-allowlist + explicit baseline-
  update + manifest-review gate — a deliberate T3-defense; the freeze
  preserves the const-allowlist property).
- (b) The §4.22 thin-client bridge principal-resolution (G-CORE-8).
- (c) The §8-C bridged-dual-runtime channel/IPC boundary (engine ↔ host
  shell).

**ARCHITECTURAL-PURIST RECOMMENDATION — keep IPC as const-allowlist; reject
registration-affordance.** The §1.A.FROZEN item 13 question "does v1 keep
the const-allowlist property or add a registration affordance" has a clear
architectural-purist answer: KEEP const-allowlist. A registration
affordance is a runtime extension point — exactly the kind of late-binding
the CLAUDE.md #19 engine-extension trust model is meant to PREVENT (engine-
level extensions are compile-time linked; "you compiled this in"). A
runtime-registerable IPC method bypasses the compile-time review gate. The
const-allowlist baseline-update PR + manifest-review is the right
mechanism. Conservative may differ here.

**What "frozen" means here:**
- The Renderer trait method set is locked (`render` + `backend_name` only).
- IPC_METHODS is the AUTHORITATIVE method-name allowlist; additions require
  explicit baseline-update + manifest-review.
- The engine's runtime ownership is structural — no API change can thread a
  host runtime through.
- The ThinClientBridge principal-resolution semantics (G-CORE-8) are frozen
  as shipped.

**What's NOT frozen:**
- Concrete Renderer impls (`BrowserRender`, `TauriRenderer`, future
  `TauriVersoRender`, `SlintRender`, etc.) are NOT part of the freeze; new
  renderer backends can land in Composing per CLAUDE.md #19 compile-time-
  linked.
- The internal channel/IPC mechanism between engine + host shell may change
  (e.g. swap from native channel to lock-free queue) as long as the
  observable contract holds.

**Verification mechanism:**
- `crates/benten-renderer-tauri/tests/compile_test_no_tauri_dep.rs`
  compile-test pin.
- `crates/benten-renderer-tauri/tests/ipc_methods_allowlist_*.rs` IPC
  allowlist pins.
- `cargo-public-api` baselines for `benten-renderer-tauri` (JSON format) and
  `benten-platform-foundation` (FREEZE-WAVE FIX-NOW: missing baseline).

**Composing-phase escape valve:**
- New IPC method = baseline-update PR; manifest-review; passes if in the
  T3-defense spirit (no privileged operations, no auth-bypass).
- Threading a shell runtime through the engine API = HARD HALT-AND-SURFACE
  (this is the §8-C explicit rejection).
- New Renderer trait method = HALT-AND-SURFACE (breaks every backend
  implementation).

---

## 14. P2P-interop conformance invariant frozen

**Frozen surfaces (per `RATIFIED-pq-default-reframe-2026-05-19.md` §4 +
§1.A.FROZEN item 14):**

A mandatory baseline conformance suite that EVERY peer MUST satisfy, with
four structural properties:

**(a) Mandatory baseline conformance suite.** Every claimed-Benten peer MUST
pass the suite. The suite lives at
`crates/benten-crypto-suite/tests/p2p_interop_conformance_*.rs` (FREEZE-WAVE
FIX-NOW VERIFY: validate this file family exists and covers all 7 swap-matrix
arms × both wire directions).

**(b) Typed-unsupported-error, NEVER silent fallback.** Unknown crypto
codepoint surfaces typed `UnsupportedAlgorithm::{Signature, CipherSuite,
Hash}` — never silent fallback. Veilid `common_crypto_kinds`-intersection /
MLS `RequiredCapabilities`-floor / Nostr **NIP-44** explicit-MUST-indicate-
unsupported normative precedent. **Age's silent-ignore is the explicitly-
REJECTED outlier.**

**(c) No wire-break when a codepoint is added.** Additive-codepoint
discipline: a new codepoint at an unused value extends the table; old
deserializers see `UnsupportedAlgorithm` and route to the typed-reject arm;
old encoders never produce the new codepoint. Wire format is permanent.

**(d) Old codepoints supported FOREVER — algorithm-add never strands
previously-written content.** Stated STRONGER than MLS: Benten content is
immutable content-addressed objects, so old objects NEVER need an MLS-style
`ReInit` ceremony. Only NEW objects use the new codepoint; structurally
avoids MLS's hard migration case.

**Component-ID reuse policy:** Benten reuses IANA HPKE/COSE component IDs
(KEM / AEAD / KDF) — **NEVER mints Benten component algorithm numbers**.
Benten owns ONLY the thin one-codepoint-per-suite SELECTOR table (MLS RFC
9420 one-codepoint-per-suite model). The selector table is Benten-owned;
the algorithm IDs reference upstream registries.

**Rejected alternative (record + reject any future re-proposal):** "PQ-TLS-
as-envelope buys time" (Matrix's transport-relayed position). REJECTED for
Benten — Benten ciphertext rests at-rest on peer disks (CLAUDE.md #18 / item
6 substrate); the transport-envelope argument doesn't apply. iroh transport
is classical-only/no-PQ-roadmap so HNDL protection MUST be Benten-owned
application-layer object encryption.

**What "frozen" means here:**
- The four structural properties (a)-(d) are PERMANENT invariants — they
  outlast any single algorithm choice.
- The conformance suite shape is locked (FREEZE-WAVE FIX-NOW VERIFY: ensure
  the suite is real, not aspirational).
- The IANA component-ID reuse policy is a freeze item (a future agent
  proposing to mint Benten algorithm numbers MUST be rejected with reference
  to this item).
- The typed-unsupported-never-silent-fallback discipline is enforced at
  every dispatch site workspace-wide.

**What's NOT frozen:**
- Which algorithms occupy reserved codepoints (e.g. `0x647b` HQC build-go
  triggered by FIPS 207 final).
- The internal mechanism by which the conformance suite runs (CI lane vs
  GitHub Action vs scheduled cron).

**Verification mechanism:**
- `crates/benten-crypto-suite/tests/p2p_interop_conformance_*.rs` (FREEZE-WAVE
  FIX-NOW VERIFY/AUTHOR).
- A standing CI lane (per §4 plan) that runs the conformance suite on every
  push.

**Composing-phase escape valve:**
- New algorithm on reserved codepoint = ADDITIVE per (c)+(d); fine.
- Changing typed-reject discipline = HALT-AND-SURFACE (rejected outright).
- Minting Benten component algorithm number = HALT-AND-SURFACE (rejected
  outright with reference to this item).

---

## 15. Sharing & Confidentiality (S&C) public surface frozen

Per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` (the authoritative
post-spike-sequence record) + §1.A.FROZEN item 15 sub-clauses (a)-(j) +
CLAUDE.md baked-in #18 (Principal primitive + plugin trust model).

### 15.a — SubgraphSpec primitive

**Frozen surfaces:**
- `crates/benten-core/src/subgraph_spec/spec.rs:188` `pub struct Spec` — the
  4-thing thin core (Roots / Expansion / Inclusion / Termination).
- `crates/benten-core/src/subgraph_spec/walker.rs:78` `pub fn walk(spec:
  &Spec) -> Result<WalkResult, SubgraphSpecError>` — the canonical walker.
- `crates/benten-core/src/subgraph_spec/walker.rs:183` `pub fn
  walker_as_subgraph() -> Subgraph` — the fractal-property pin (the walker
  IS a Subgraph composed of the existing 12 operation primitives;
  CLAUDE.md baked-in #1 12-primitive irreducibility PRESERVED — no new
  `PrimitiveKind` variant minted).
- `pub struct WalkResult` (`walker.rs:50`).
- `pub struct StructuralPath` (`spec.rs:38`).
- `pub enum SubgraphSpecError` (`errors.rs:20`).
- `pub fn intersect(a: &Spec, b: &Spec)` / `pub fn union(...)` / `pub fn
  filter(...)` combinators (`combinators.rs:37, 99, 140`).

**What "frozen" means here:**
- The 4-thing structural decomposition (Roots / Expansion / Inclusion /
  Termination) is PERMANENT.
- The fractal-property invariant: `walker_as_subgraph()` is a Subgraph
  composed of the existing 12 primitives; ANY proposal to mint a new
  `PrimitiveKind::SubgraphSpec` variant is a HARD-HALT (re-opens
  CLAUDE.md #1).
- Walker enumeration order = BFS (per (h) below).

**ARCHITECTURAL-PURIST CONCERN — type-name collision.** Two `RestrictedSpec`
types exist at HEAD: (i) `crates/benten-caps/src/restricted_spec.rs:103` (the
6-dimension product per (b) below); (ii) `crates/benten-core/src/subgraph_
spec/spec.rs:126` (a different enum). **FREEZE-WAVE FIX-NOW: resolve the
name collision** — either alias them via `pub use`, or rename one to its
distinct semantic. Two distinct `pub struct RestrictedSpec` types in the same
workspace is a documentation / import-confusion liability that will burn
Composing-time developers. Architectural-purist call: the `benten-core`
type should be renamed to `subgraph_spec::SpecRestriction` (or similar) and
the `benten-caps::RestrictedSpec` keeps the canonical name (since (c)
`Scope::RestrictedSelector(RestrictedSpec)` references it as the load-
bearing API surface name).

### 15.b — `RestrictedSpec` shape (6-dimension product)

**Frozen surfaces:**
- `crates/benten-caps/src/restricted_spec.rs:103` `pub struct RestrictedSpec`
  with 6 dimensions: roots + edge-allowlist + max_depth + label-allowlist +
  label-denylist + property-equalities.
- Structural `contains(&self, other: &RestrictedSpec) -> bool` method —
  decidable per-dimension + composed with `&&`.

**Extension slots** for additive predicates (Boolean OR, numeric comparisons,
has_edge, anchor-chain LIMIT, etc.) are NAMED-not-opaque — each future
extension lands as a typed slot with its own decidable containment rule
(~50-100 LOC).

**Rejected and frozen-as-rejected:** the witness-bearing OpaqueSelector
mechanism (Path b in R1). Structurally unsound per Spike H+1.1's b.SEC #4.
Future agents proposing the witness pattern as a feature MUST be rejected
with reference to this freeze item.

**What "frozen" means here:**
- The 6 dimensions + their structural-containment semantics are locked.
- The NAMED-extension-slot pattern is the canonical add-mechanism (no opaque
  arm).
- `contains()` is decidable + total (no panics, no `unimplemented!()`).

**What's NOT frozen:**
- The internal representation of each dimension (e.g. `BTreeSet` vs `Vec`
  vs `Cow`) may evolve.
- The set of named extension slots is OPEN per the additive-predicate
  pattern.

### 15.c — Structured UCAN `Scope` enum

**Frozen surfaces:**
- `crates/benten-caps/src/scope.rs:46` `pub enum Scope` — EXACTLY TWO arms:
  `Hashes(Vec<Cid>)` + `RestrictedSelector(RestrictedSpec)`.
- **NOT `#[non_exhaustive]`** (per the doc-block at scope.rs:33-44: the
  freeze MUST be enforced by the type system so a future agent proposing a
  third arm gets exhaustive-match compile failures + the cite-drift CI lane
  fires on the `no-opaque-arm` sentinel).
- A third arm CANNOT be added post-freeze without explicit re-open.
- Wire envelope typed for additive future arms via codepoint-dispatch (same
  playbook as crypto-agility per CLAUDE.md #5) — i.e. a future arm lands at
  a NEW codepoint, never repurposing the existing two-arm enum.

**What "frozen" means here:**
- EXACTLY two arms — structural pin via exhaustive `match` at every consumer
  site.
- The wire form of each arm (CBOR encoding) is part of item 4 (D2 v1-
  canonical-bytes); bytewise locked.

**What's NOT frozen:**
- Wire-codepoint dispatch may evolve to ADD new scope variants at new
  codepoints (per the additive-codepoint discipline of item 14); existing
  codepoints stay.

### 15.d — `AuthorizationGrant` envelope = ONE signed artifact

**Frozen surfaces:**
- `crates/benten-caps/src/authorization_grant.rs:205` `pub struct
  AuthorizationGrant { ucan, key_material, binding_sig }`.
- `crates/benten-caps/src/authorization_grant.rs:166` `pub struct
  KeyMaterial` (the cap-side key material handle; distinct from the
  crypto-suite-side `KeyMaterial` at `crates/benten-crypto-suite/src/aead.
  rs:85`).

**ARCHITECTURAL-PURIST CONCERN — second `KeyMaterial` name collision.** Two
`KeyMaterial` types exist (caps + crypto-suite). Distinct semantics; same
name. **FREEZE-WAVE FIX-NOW: rename for clarity** — the caps-side is the
GRANT-bearing handle (carries audience binding + paths); the crypto-suite-
side is the AEAD-bearing key. Rename: `benten_caps::AuthorizationKeyMaterial`
vs `benten_crypto_suite::aead::AeadKeyMaterial`. The compatible interpretation
risk (someone imports the wrong one and the trait-bounds line up enough that
it compiles but runtime is wrong) is exactly the kind of trap a type-level
distinction prevents.

**Frozen semantics:**
- `binding_sig` is computed by the issuer over the CBOR-encoded
  `(ucan, key_material)` tuple, bound to the same audience.
- Validators check `binding_sig` BEFORE consulting the UCAN scope or the
  key material — the binding is the foundation.
- Consistent across online (custom-ALPN handler in G-CORE-3e) and offline
  (Drop bundle in G-CORE-3f) paths.

**What "frozen" means here:**
- Field set + CBOR encoding bytewise locked (item 4).
- Validation ORDER (binding → UCAN → keys) is part of the contract.
- The ONE-artifact-not-TWO discipline is permanent (avoids the per-
  deployment-shape ambiguity Spike H identified).

### 15.e — Encryption-class enum

**Frozen surfaces:**
- An `pub enum EncryptionClass { Public, Confidential }` — **CURRENTLY DOES
  NOT EXIST AT HEAD** (verified via grep). The freeze MUST mint this enum
  per item 15(e) — likely in `crates/benten-caps/src/lib.rs` or
  `crates/benten-core/src/lib.rs`.
- `#[non_exhaustive]` AT THE ENUM (architectural-purist position: this is
  one where `#[non_exhaustive]` IS the right call — future variants
  `AnonymousGroup`, `PrivateLocal` may add post-v1).

**FREEZE-WAVE FIX-NOW: author the `EncryptionClass` enum + wire-format
codepoint mapping + typed-reject discipline.** Per item 15(e): reserved-but-
unimplemented future variants typed at v1-beta with the typed-reject pattern;
explicit-add via additive enum variants post-v1.

**What "frozen" means here:**
- The two-variant baseline (Public + Confidential) is locked.
- `#[non_exhaustive]` permits additive future variants without re-freezing
  the shape.
- Wire-form per variant is locked (CBOR + codepoint dispatch).

### 15.f — Two-path key-derivation contract (Interpretation B per Spike E)

**Frozen surfaces:**
- The HKDF-SHA256 `derive_step` API in `crates/benten-crypto-suite/src/aead.
  rs` (or wherever the corrected derivation lives):
  - `K(N) = HKDF-SHA256(K(predecessor), info = "step" || edge_label ||
    N.cid)` (the `"step"` HKDF info-tag provides cross-role domain
    separation per Spike E's correction).
  - Root key `K(root) = HKDF-SHA256(K_principal, info = "root" || root_cid)`.
- Path-tagged keys: a Node reachable by multiple paths gets multiple distinct
  keys (feature for selective-share; envelope records which canonical path
  produced each ciphertext per (g) chunk + per (h) walker).
- **KDF = HKDF-SHA256 v1-beta DEFAULT** (codepoint-dispatched per CLAUDE.md
  #5; future codepoint additions land additively per item 14).

**FREEZE-WAVE FIX-NOW VERIFY:** the `derive_step` API exists at HEAD, carries
the `"step"`/`"root"` info-tag fix (R0.8 correction), and is exercised by
test pins. Verify and lock.

**What "frozen" means here:**
- The HKDF info-tag convention (`"step"` for step-derivation, `"root"` for
  root) is wire-permanent (a different tag = different key = decryption
  failure).
- Owner-derivable via owner-walks-canonical-path; recipient-derivable via
  given-K(N)-recipient-walks-edge-labels — the two-path symmetry is the
  contract.
- KDF codepoint-dispatch is in place; alternative KDFs may be added at new
  codepoints.

### 15.g — Two-CID mapping contract + per-chunk-AEAD chunk-size constant

**Frozen surfaces:**
- `crates/benten-sync/src/two_cid_store.rs::TwoCidStore` — the wave-3e
  adapter wrapping a ciphertext-bytes backing store + the two-CID mapping
  (plaintext_cid → ciphertext_cid).
- redb-backed `plaintext_cid → ciphertext_cid` mapping table — durable.
- UCAN scopes against plaintext_cid; iroh-blobs serves ciphertext blob by
  its own hash (preserves "served-bytes-hash == requested-hash" invariant).
- `crates/benten-graph/src/aead_wrap.rs:56` `pub const IROH_BLOCK_SIZE:
  usize = 16384` (16 KiB) — the load-bearing chunk-size constant.
- `crates/benten-graph/src/aead_wrap.rs:63` `pub const WHOLE_AEAD_THRESHOLD:
  usize` — Nodes < THRESHOLD use whole-content AEAD; ≥ THRESHOLD use
  per-chunk AEAD.

**For Nodes ≥64 KiB:** per-chunk AEAD with chunk size = `IROH_BLOCK_SIZE`
+ AAD-binds-chunk-index. Whole-content AEAD for smaller Nodes.

**What "frozen" means here:**
- The `IROH_BLOCK_SIZE = 16384` constant is wire-format-load-bearing
  (different chunk size = double-chunking overhead; bytewise dependency).
- The 64 KiB threshold for chunked-vs-whole is part of the freeze.
- The two-CID mapping shape (plaintext_cid → ciphertext_cid) is the
  permanent storage substrate seam.

**What's NOT frozen:**
- The redb table schema-version (already accommodated via `SnapshotBlob`
  precedent for any future migration).
- The iroh-blobs upstream crate version (Benten consumes via stable API).

### 15.h — SubgraphSpec walker IS a Subgraph shipped once in `benten_core`

**Frozen surfaces:**
- `crates/benten-core/src/subgraph_spec/walker.rs:78` `pub fn walk` — the
  canonical BFS-order walker.
- `crates/benten-core/src/subgraph_spec/walker.rs:183` `pub fn
  walker_as_subgraph() -> Subgraph` — the fractal-property pin.
- The BFS-order = canonical path contract (per R4) — recipients walk the
  same BFS the producer enumerated; canonical path carried in
  `AuthorizationGrant.key_material` per (d) + (f).

**Engine wrapper:** the public consumer surface is the engine method (e.g.
`Engine::walk_share_scope()` or similar) that wraps the walker. **FREEZE-
WAVE FIX-NOW VERIFY:** the engine wrapper exists at HEAD; if not, mint it
per item 15(h).

**What "frozen" means here:**
- The walker is data-not-evaluator-extension (no evaluator special-case for
  SubgraphSpec).
- BFS enumeration order is part of the wire-bytes (path-tagged keys depend
  on it).
- The walker ships ONCE in `benten_core`; no duplication elsewhere.

### 15.i — Revocation reach documentation

**Frozen surfaces (DOCUMENTED-DESIGN-CONSTANT, not code-shape):**
- `docs/SECURITY-POSTURE.md` carries an explicit section documenting:
  - UCAN revocation cuts future serves (cap-policy check fails for
    subsequent requests).
  - Already-derived keys remain decryptable FOREVER. Once Bob has derived
    `K(N)` for some Node, Bob can decrypt any ciphertext he obtains for
    that Node, regardless of UCAN revocation. Re-keying the Node requires
    Alice to re-encrypt + re-issue (a heavy operation).
  - Drop bundles are forever-valid once distributed. Producer has no
    callback to revoke an already-distributed Drop.
  - Mitigation: tight `nbf`/`exp` + key rotation.

**Considered for a Compromise # (Phase-4-Meta-Core close).** FREEZE-WAVE
FIX-NOW: assign the next compromise number + add the entry to
SECURITY-POSTURE.md.

**What "frozen" means here:**
- The documentation language is the v1-beta posture; users + plugins consume
  it as the security model.
- The cited mitigations (tight `nbf`/`exp` + key rotation) are the v1
  affordances — Composing may implement additional mitigations.

### 15.j — Resolver evaluation model = live-per-request

**Frozen surfaces (DOCUMENTED CONTRACT, not wire-format):**
- The resolver evaluates a SubgraphSpec against the CURRENT graph state on
  every request (NOT a frozen-snapshot semantics).
- IVM-cache-invalidation seam reuses G-CORE-4's CanonicalViews subscription.
- UCANs gate sub-graph SHAPES that EVOLVE (Alice's writes-since-issuance
  flow into Bob's accessible scope automatically).
- The ONLY frozen-snapshot path is the offline-Drop-bundle (Drop = snapshot-
  at-production-time by construction).

**What "frozen" means here:**
- The live-per-request semantics is the v1 contract; consumers depend on it.
- The IVM-cache seam interface (whatever it's called at HEAD; verify the
  CanonicalViews subscription surface) is part of the freeze.
- The Drop-bundle = frozen-snapshot dichotomy is permanent.

**What's NOT frozen:**
- The internal caching strategy (LRU / TTL / etc.).
- The CanonicalViews subscription implementation.

---

## Composing-phase escape valve (per methodology-r1-5)

Any Composing-time discovery that would require altering a §1.A.FROZEN
surface is a **HALT-AND-SURFACE-TO-BEN event**, NOT an orchestrator-
autonomous Core re-open:

1. The orchestrator STOPS the affected Composing lane.
2. Records in the Composing R6/handoff the specific frozen surface + the
   required change + why Composing cannot proceed without it.
3. Surfaces it to Ben as a decision (plain-English + options + prediction
   per the standing surface discipline).

**Structural backstop:** the `cargo-public-api` gate + the #1204 TS-side
parity gate (FREEZE-WAVE FIX-NOW to commit) + the per-shape byte-pin tests
+ the `#[non_exhaustive]` sweep + the cite-drift scanners + the workspace
`missing_docs` test catch silent frozen-surface mutations structurally —
the gate makes silent re-open structurally impossible, not merely policy-
forbidden.

**Architectural-purist sharpening (NEW pim-N candidate cross-cutting all 15
items):** every `pub` declaration in a frozen module gets a `// FROZEN: re-
open requires Ben sign-off` comment that a CI lint scans for. Documentation
+ tooling layers; defense in depth.

---

## Provenance + cross-cites

**Authoring:** Planner-A (architectural-purist angle) drafted this artifact
in `g-core-9/planner-a-architectural-purist` worktree branch on 2026-05-23
post-#1342 main HEAD `ae7cd3d5` (CATALOG_VARIANT_COUNT 191; 14 workspace
crates). Companion: Planner-B (conservative-minimal-freeze) draft on
parallel branch. Orchestrator triages both into single working artifact.

**Authoritative inputs (read in addition to this artifact):**
- `.addl/phase-4-meta/00-implementation-plan.md` §1.A C1-C13 + §1.A.FROZEN
  (15 items, lines 111-148).
- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
  (R1-R6 ratifications + 8 spike-derived refinements).
- `.addl/pq-research/RATIFIED-pq-default-reframe-2026-05-19.md` (PQ-hybrid
  default + audit-gates-GM + the four landscape passes).
- `.addl/phase-4-meta/RATIFIED-crypto-agility-2026-05-18.md` (multiformats-
  permanent framing).
- `.addl/phase-4-meta/RATIFIED-prework-forks-2026-05-18.md` (§8-A tighten +
  §8-E sealed + §8-C bridged-dual-runtime).
- `CLAUDE.md` Architectural Decisions Baked In items 1, 5, 7, 15, 17, 18, 19.

**Distinctive architectural-purist calls captured in this draft** (where
the angle chose purist over compromise; the orchestrator may pick the
conservative answer instead):

1. **Item 1 §8-A:** aggressive tighten incl. renaming `get_node` →
   `read_node`, `get_node_label_only` → `read_node_label_only`, DELETING
   `resolve_subgraph_cid_for_test` from the public surface entirely
   (relocated to `testing` module). Conservative may keep some `pub`
   "because napi uses them."
2. **Item 8 sealed-CapabilityPolicy:** HARDEN the soft-seal to a true
   private supertrait in G-CORE-9 (roll the cited G-CORE-8.3 follow-up
   wave INTO the freeze). Pay the ~20-test-file migration cost now;
   benefit is rustc-enforced seal at v1. Conservative ships with soft-seal +
   workspace-introspection-audit lane.
3. **Item 11 `#[non_exhaustive]` MAXIMALIST:** apply universally to all 158
   public enums + structs unless documented carve-out (item 15(c) `Scope`
   is the canonical carve-out — exactly-two-arms structural pin). The
   `WriteContext` + `ChangeEvent` + `GraphError::TxAborted` + 12+ engine
   enums currently missing the attribute MUST get it pre-freeze.
   Conservative defers some.
4. **Item 13 IPC:** KEEP const-allowlist; REJECT registration-affordance.
   Per CLAUDE.md #19 engine-extensions are compile-time-linked; runtime
   registration bypasses the compile-time review gate.
5. **Item 15(a) name-collision:** rename `benten_core::subgraph_spec::
   RestrictedSpec` to `SpecRestriction` so `benten_caps::RestrictedSpec`
   stays the canonical name referenced from `Scope::RestrictedSelector`.
6. **Item 15(d) name-collision:** rename `benten_caps::KeyMaterial` →
   `AuthorizationKeyMaterial` and `benten_crypto_suite::aead::KeyMaterial`
   → `AeadKeyMaterial` to prevent import confusion.
7. **Item 7 deletion-discipline extension:** sweep ALL `legacy` / `_v1_
   compat` / `deprecated` / old `TODO(remove)` surfaces workspace-wide for
   the same deletion treatment §4.33 received.
8. **NEW pim-N candidate:** `// FROZEN: re-open requires Ben sign-off`
   marker comment on every `pub` declaration in a frozen module + CI lint
   scanner. Per `feedback_handoff_top_banner_re_orient`'s pattern of
   structural-markers-beat-trust-the-reader.
9. **FREEZE-WAVE FIX-NOW gap surfaced**: `cargo-public-api` baselines are
   seeded stubs at HEAD, NOT real public-API surface lists. G-CORE-9 MUST
   regenerate ALL 14 crate baselines (including the 3 missing:
   `benten-crypto-suite`, `benten-drop`, `benten-platform-foundation`).
10. **FREEZE-WAVE FIX-NOW gap surfaced**: #1204 TS-side parity gate does
    NOT exist at HEAD; G-CORE-9 MUST commit the `@microsoft/api-extractor`-
    equivalent workflow as the JS analog of `cargo-public-api`.
11. **FREEZE-WAVE FIX-NOW gap surfaced**: item 15(e) `EncryptionClass` enum
    does not exist at HEAD; G-CORE-9 MUST mint it with the two-variant
    baseline + `#[non_exhaustive]` + typed-reject discipline.
12. **FREEZE-WAVE FIX-NOW gap surfaced**: item 12
    `ManifestEnvelopeRecheckOutcome` MUST get `#[non_exhaustive]` at the
    freeze.
13. **FREEZE-WAVE FIX-NOW gap surfaced**: TS classes (193) vs Rust
    ErrorCode variants (191) — investigate +2 delta; either legitimate
    envelope-class items or genuine drift to close before freeze.

**Hard escalations surfaced (Planner-A returns these for orchestrator
triage):**

- **E1.** Item 15(e) `EncryptionClass` enum is named in the freeze spec but
  doesn't exist in the codebase at `ae7cd3d5`. Planner-A flags as
  FREEZE-WAVE FIX-NOW; orchestrator should triage whether the gap is real
  or whether the spec references a planned-not-shipped surface that should
  be retired from item 15(e).
- **E2.** Two `RestrictedSpec` types (caps + core) and two `KeyMaterial`
  types (caps + crypto-suite) at HEAD. Architectural-purist position is
  rename for clarity; conservative may accept the duplication. Surface as
  a design-coherence decision-point.
- **E3.** The §8-A tighten + rename + delete-`_for_test`-from-public
  combination is a LARGER public-API delta than a pure `pub→pub(crate)`
  visibility flip. The G-CORE-9 wave MUST verify napi-side cascade migration
  + cargo-public-api baseline regen + all dependent test files migrate to
  the `testing` module + the renames don't conflict with any other `pub fn
  read_node*` surface workspace-wide. If the cascade is >100 file edits,
  surface as a scope-call to Ben (architectural-purist still recommends
  doing it; cost is real but worth it pre-freeze).

No inconsistency between the cited RATIFIED docs was found that requires
Ben resolution before the freeze can lock. The 6 R1-R6 ratifications, 8
spike-derived refinements, PQ-hybrid-default placement, sealed-policy
shape, and bridged-dual-runtime boundary are all internally coherent.

**End of architectural-purist draft.**
