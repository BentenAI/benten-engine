# Spike: phase-1-stack

## Verdict

**STACK VALIDATED** — proceed to Phase 1 proper. The original `core2`-yank dependency concern raised during the spike is now resolved with our BentenAI/rust-cid fork and filed upstream PR ([multiformats/rust-cid#185](https://github.com/multiformats/rust-cid/pull/185)); see "Surprises" #1 and "Next Actions" #1 for the remaining upstream-merge watch.

## Assumption Tested

Phase 1 Rust stack composes end-to-end with deterministic content hashing:

- 6-crate workspace (`benten-core`, `benten-graph`, `benten-ivm`, `benten-caps`,
  `benten-eval`, `benten-engine`) + `bindings/napi` builds against the validated
  2026 dependency versions.
- Content hashing (BLAKE3 over DAG-CBOR via `serde_ipld_dagcbor`, CIDv1
  envelope with multicodec `0x71` / multihash `0x1e`) produces identical CIDs
  across runs and processes for identical Node content.
- `benten-core` compiles to `wasm32-unknown-unknown`, preserving the future
  WASM target.
- redb v4 provides the storage primitives (`get`, `put`, `delete`, prefix
  `scan`, atomic `put_batch`) that `KVBackend` promises.
- napi-rs v3 compiles the Node.js binding surface we need (`createNode`,
  `getNode`) without requiring a separate wasm-bindgen path.

## What Shipped

- [x] 6 crates created, workspace compiles
- [x] `benten-core`: `Node`, `Value`, content hash → `Cid` (CIDv1)
- [x] `benten-graph`: `KVBackend` trait + `RedbBackend` impl + Node CRUD
- [x] `benten-engine`: orchestrator API (`Engine::{open, create_node, get_node}`)
- [x] `bindings/napi`: `initEngine`, `createNode`, `getNode` exposed to TS
- [x] criterion benchmark: hash-only, create_node, get_node, full_roundtrip
- [x] `cargo fmt --check` / `cargo clippy --workspace --all-targets -- -D warnings`
      / `cargo nextest run --workspace` / `cargo doc --workspace --no-deps` all green
- [x] D1 intra-process determinism test passes
  (`benten-core::tests::d1_intra_process_determinism`)
- [x] D2 cross-process determinism test passes — fixture committed at
  `crates/benten-core/tests/fixtures/canonical_cid.txt`
  (`benten-core::d2_cross_process::d2_cross_process_determinism`)
- [x] D3 `wasm32-unknown-unknown` compile-check of `benten-core` passes
  (`cargo check --target wasm32-unknown-unknown -p benten-core`)

**Test count:** 15 passing (7 `benten-core` unit, 1 `benten-core` integration,
5 `benten-graph` unit, 2 `benten-engine` unit).

## What I Punted

- **`multihash` crate is declared in the workspace but not actually imported.**
  The spike hand-rolls the 36-byte CIDv1 envelope (version byte + multicodec +
  multihash-code + length + 32-byte digest) because it is fixed and tiny.
  Phase 1 proper should swap to the `cid`/`multihash` crates for full IPLD
  interop. The byte layout is already compatible, so it's a drop-in swap.
- **Base32 encoder/decoder is hand-rolled** (RFC 4648 lowercase, no padding)
  because the spike avoided pulling `multibase` into the hot path. Migrate to
  `multibase` alongside the `cid` swap.
- **IVM, Caps, Eval crates are stubs.** Per the brief, each has `Cargo.toml` +
  `src/lib.rs` with a doc comment describing Phase 1 responsibilities. No real
  implementation. They exist in the spike to validate the 6-crate workspace
  compiles cleanly and that `benten-engine` can depend on them.
- **TypeScript-side smoke test of the napi binding.** The Rust crate compiles
  and exports the symbols (napi-rs build script emits glue at `OUT_DIR`), but
  the spike does not yet invoke it from Node.js. The Rust-level round-trip
  tests in `benten-engine` cover the full storage path; the next Phase 1 task
  is to add a `bindings/napi/package.json` + Jest/Vitest smoke test.
- **`mimalloc` / `papaya` / `wasmtime` are declared in the workspace but not
  wired up.** They are listed for Phase 1 proper. Reserving them in
  `[workspace.dependencies]` at validated versions keeps each crate's per-use
  declaration to `{ workspace = true }` when the time comes.
- **`NoAuthBackend` capability hook is not yet exercised through the engine
  write path.** The crate stub exists; the wiring is a Phase 1 proper task.
- **`rustfmt.toml` fields that require nightly.** Left as-is; the spike stays
  on stable.

## Canonical Test Node CID

```
bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda
```

- Base32 (multibase `b`, lowercase, no padding).
- Node content: `{labels: ["Post"], properties: {published: true, tags:
  ["rust","graph"], title: "Hello, Benten", views: 42}}`.
- Fixture pinned at
  `crates/benten-core/tests/fixtures/canonical_cid.txt`. Every `cargo nextest`
  run asserts against this string; if the hash drifts, D2 fails loudly.
- Regenerate with `cargo run --example print_canonical_cid -p benten-core`.

## Benchmark Numbers

Measured on macOS / aarch64-apple-darwin (M-series), Rust 1.94.1, `cargo
bench --release` with LTO thin, codegen-units = 1.

| Bench | Median | ENGINE-SPEC §14.6 target | Verdict |
|---|---|---|---|
| `hash_only` | **892 ns** | no explicit target | Fine |
| `get_node` | **2.71 µs** | 1–50 µs (hot-cache lookup) | **Within target** |
| `create_node` | **4.02 ms** | 100–500 µs realistic | **Above target — redb fsync** |
| `full_roundtrip` | **4.23 ms** | — | Dominated by `create_node` fsync |

**Honest reading:**

- `create_node` is ~8x the realistic target because every call is a fully
  durable redb commit — a two-phase commit with `fsync`. Per §14.6 itself,
  "fsync to disk is 0.1–10ms; spec must define durability policy per write
  class". We are in the middle of that range. Phase 1 proper needs the
  "group commit for bulk, immediate for capability grants" policy called out
  in the spec.
- `get_node` at 2.71µs is excellent — confirms redb hot-cache reads are
  well inside the 1–50 µs band. Adding capability checks and IVM maintenance
  in Phase 1 will push this higher but the headroom is real.
- `hash_only` at 892ns shows BLAKE3 + DAG-CBOR canonical form is not the
  bottleneck; write path is.
- **10-node handler evaluation**: N/A for this spike — no evaluator yet.

Raw criterion output is in `target/criterion/`.

## Surprises

1. **`core2 = "^0.4"` is yanked from crates.io with no replacement 0.4.x.**
   Transitively required through two chains: (1) `cid` → `ipld-core` →
   `serde_ipld_dagcbor` (our hash path), and (2) `multihash` +
   `multihash-derive` + `multihash-codetable` (via `cid`'s feature wiring).
   Workspace was unbuildable against the default registry.

   **Resolution (2026-04-14):** The upstream `bbqsrc/core2` repository was
   archived the same day with the note "No longer supported. Use `core`
   directly." Since `core::io::{Read, Write}` are not yet stable in Rust
   ([rust-lang/rust#68315](https://github.com/rust-lang/rust/issues/68315),
   open since 2020), a direct migration to `core` is not currently possible.

   We forked `multiformats/rust-cid` to
   [`BentenAI/rust-cid`](https://github.com/BentenAI/rust-cid)
   and replaced `core2` with [`no_std_io2`](https://crates.io/crates/no_std_io2)
   (an API-compatible drop-in), mirroring the approach used in the sibling
   crate's PR [multiformats/rust-multihash#407](https://github.com/multiformats/rust-multihash/pull/407)
   for multiformats-org consistency. Fork commit:
   [`e11cf45399c951597725a9bc3ed49c805f7aa640`](https://github.com/BentenAI/rust-cid/commit/e11cf45399c951597725a9bc3ed49c805f7aa640).
   Upstream PR: [multiformats/rust-cid#185](https://github.com/multiformats/rust-cid/pull/185)
   (open, tracking #184).

   The fork resolves chain (1) directly. Chain (2) remains covered by a
   temporary `[patch.crates-io] core2 = { git = "technocreatives/core2", rev = "..." }`
   pin until rust-multihash#407 merges and ships a release; that release
   will propagate a core2-free multihash through the transitive tree and
   our workspace `core2` patch can be removed. Both patches are pinned to
   specific commit SHAs (no floating branch refs).

   The canonical CID fixture (`bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`)
   is byte-identical before and after repointing `cid` to our fork,
   confirming the `no_std_io2` swap preserves encoding semantics exactly.

2. **`#![forbid(unsafe_code)]` is incompatible with `#[napi]` macro
   expansion.** napi-rs v3 generates `unsafe extern "C"` ctor-registration
   shims. `deny(unsafe_code)` in the napi crate is equivalent for hand-
   written code and permitted for macro-generated code — which is correct
   for a thin FFI layer whose entire reason for existing is wrapping the
   Node.js C API. The other 5 crates keep `forbid(unsafe_code)`.

3. **`blake3 = { default-features = true }` is not WASM-compatible** because
   it pulls in SIMD detection that requires `std::arch`. Setting
   `features = ["pure"]` on the workspace's `blake3` dep produces a
   pure-Rust implementation that compiles to `wasm32-unknown-unknown`. At
   native runtime, `blake3` still uses SIMD paths via its inline assembly
   variants because "pure" only controls the C fallback, not the Rust
   intrinsics. No performance hit on aarch64 native.

4. **`clippy::doc_markdown` is aggressive.** `CIDv1`, `MVCC`, `IVM`, `UCAN`,
   `DID`, etc. all triggered the lint. Added a `doc-valid-idents` allowlist
   in `clippy.toml` so Benten-specific terms don't have to be backticked in
   every doc line. Kept standard clippy defaults (`KiB`/`MiB`/etc.) that
   would otherwise be lost when the key is specified.

5. **`[u8; 36]` has no default `Serialize`/`Deserialize` impl.** Serde only
   derives for `[u8; N]` up to N = 32. Wrote a small `serde_bytes_fixed`
   helper module in `benten-core` that round-trips `[u8; CID_LEN]` through a
   `ByteBuf`, validating the length on deserialize. Phase 1 proper migrates
   to the `cid` crate which handles this internally.

6. **redb v4 works exactly as the spec assumed.** No surprises on the
   transaction API, range scans, or durability model. The only note is that
   `redb::Database::create` requires the parent directory to already exist
   (it does not `mkdir -p`); the spike handles this, but callers of the
   `Engine::open` public API get the raw error if the path is bad.

7. **Cross-process determinism is real.** The same canonical test Node
   produces `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`
   from a fresh process on every run (D2 test, fixture-backed). DAG-CBOR's
   length-first sort, BLAKE3's deterministic hash, and our fixed 36-byte
   CIDv1 envelope combine cleanly.

## Decision

**Proceed to Phase 1 proper.** The stack is validated on every axis the spike
was meant to exercise:

- 6-crate workspace composes without friction.
- Content addressing is deterministic intra-process, cross-process, and
  compiles to WASM.
- Storage layer is functional with durable commits and atomic batches.
- TypeScript ↔ Rust binding layer via napi-rs v3 compiles and exports the
  expected symbols.

The `core2` yank is a real friction point but it's a single `[patch]`
directive and does not invalidate any architectural assumption. Treat it as
tech-debt to clean up inside Phase 1, not as a reason to iterate on the
spike or pivot dependencies.

## Next Actions for the Orchestrator

Numbered so a different agent can pick any of these up:

1. **Land the `core2` resolution.** Current state (per "Surprises" #1
   above): `cid` is patched to our `BentenAI/rust-cid` fork at
   commit `e11cf45…`, and `core2` itself is temporarily patched to
   `technocreatives/core2@545e84b…` to unblock the `multihash` chain.

   Next steps, in order of preference:

   (a) **Wait for [multiformats/rust-cid#185](https://github.com/multiformats/rust-cid/pull/185)
       to merge and a release to land on crates.io**, then drop our `cid`
       patch. If the maintainers prefer `embedded-io` over `no_std_io2`,
       rework the fork branch accordingly (our PR body explicitly offers
       this).

   (b) **Wait for [multiformats/rust-multihash#407](https://github.com/multiformats/rust-multihash/pull/407)
       to merge and a release**, then drop our `core2` patch. These two
       releases together remove all `core2` traces from Benten's dep tree.

   (c) If either PR stalls indefinitely, file a matching PR to the other
       sibling crate (`rust-multihash` migration under BentenAI,
       or offer to co-maintain). Avoid vendoring a shim — the
       `no_std_io2` approach is already the emerging multiformats-org
       precedent; diverging from it is strictly worse than keeping the
       patches until merge.

   **Owner:** orchestrator monitors the two PRs; no agent action until a
   maintainer signal arrives.

2. **Add the Node.js-side smoke test for the napi binding.** Create
   `bindings/napi/package.json`, `bindings/napi/index.mjs`, and a
   Jest/Vitest test that calls `initEngine` + `createNode` + `getNode` from
   TypeScript and asserts the returned CID matches the Rust-side fixture.
   This closes the TS → Rust → TS loop that the brief called for — the Rust
   side is proven end-to-end but the JS side is build-checked only.
   **Owner:** `implementation-developer` (bindings scope).

3. **Migrate `Cid` to the `cid` crate + drop the hand-rolled base32.** The
   envelope is identical; the migration is cosmetic and unlocks IPLD
   tooling interop. Keep `CID_V1` / `MULTICODEC_DAG_CBOR` / `MULTIHASH_BLAKE3`
   constants exposed for documentation even after migration.
   **Owner:** `implementation-developer` (benten-core scope).

4. **Define the durability policy the bench called out.** The 4ms
   `create_node` ceiling is fsync-bounded. `benten-graph` should expose a
   `DurabilityMode` (`Immediate`, `Group`, `Async`) that callers opt into
   per write class, per ENGINE-SPEC §14.6. The transaction primitive in
   `benten-eval` (Phase 1 proper) needs this to hit the realistic
   100–500 µs target for bulk writes.
   **Owner:** `performance-engineer` + `implementation-developer` pair.

5. **Fill in the stub crates** — `benten-ivm`, `benten-caps`, `benten-eval`
   — per their existing `lib.rs` doc comments and ENGINE-SPEC §3, §8, §9.
   Each should get its own `/spike <name>` pass before full
   implementation, since the IVM algorithm in particular is still an open
   question (§8, §14.6, Open Question 5).
   **Owner:** `implementation-developer` per crate.

6. **Watch upstream PR responses.** `multiformats/rust-cid#185` and
   `multiformats/rust-multihash#407` are the two open PRs whose merge
   eliminates Benten's `[patch.crates-io]` entries. Revisit weekly; file
   a matching migration PR under `BentenAI` if either stalls
   past four weeks with no maintainer engagement.
   **Owner:** orchestrator.

---

## Critic Triage (R5-equivalent, first pass)

Three critics reviewed the spike after the six-slice commit landed. All three
returned verdict `pass`. Raw JSON findings persisted at
`.addl/spike/{benten-core-guardian,determinism-verifier,code-reviewer-benten-graph}.json`.
Every finding has a disposition below — fix-now (code already updated in the
critic-triage commit), defer-with-explicit-phase-target, or
disagree-with-rationale.

### Fixed in the critic-triage commit

| Finding | Source | What landed |
|---|---|---|
| Prefix scan was O(n) full-table walk | `code-reviewer` M5 | Rewrote `RedbBackend::scan` to use `table.range(prefix..next_prefix)` with a `next_prefix` helper; added `scan_prefix_bounds_the_range`, `scan_all_0xff_prefix_is_open_ended`, and `next_prefix_increments_and_trims` tests |
| Test suite missed empty-prefix scan, zero-hit scan, empty-batch | `code-reviewer` m17 (partial) | Added `scan_empty_prefix_returns_everything`, `scan_zero_hit_prefix_returns_empty`, `batch_put_empty_slice_is_a_noop` |
| `canonical_test_node` duplicated between `mod tests` and `pub mod testing` | `benten-core-guardian` | Unit tests now import from `super::testing::canonical_test_node`; one source of truth |
| D2 bootstrap branch could silently re-pin a lost fixture | `benten-core-guardian` + `determinism-verifier` | Gated the bootstrap behind `BENTEN_D2_BOOTSTRAP=1`; missing fixture now fails loudly by default |
| Module doc claimed D3 was a shipped test | `benten-core-guardian` m1 | Reworded to call out D3 as a CI `cargo check`, not a unit test |
| Base32 comment confused RFC 4648 alphabet with Extended Hex | `benten-core-guardian` m2 | Rewrote the doc comment to clarify multibase `b` uses the lowercase standard alphabet, not Extended Hex |
| `#[serde(untagged)]` round-trip hazard undocumented | `benten-core-guardian` m4 | Added a doc comment on `Value` explaining why DAG-CBOR major-type tagging keeps this unambiguous; proptest validation deferred to Phase 1 (below) |
| BTreeMap's role in determinism overstated in doc | `determinism-verifier` cbor-determinism | Rewrote the `Determinism guarantees` section to credit `serde_ipld_dagcbor`'s encode-time canonicalization as the load-bearing guarantee, with BTreeMap as belt-and-suspenders |
| `RedbBackend` Clone/Arc ergonomics and path-validation responsibility were undocumented | `code-reviewer` m15 + m19 | Added `# Concurrency` and `# Path handling` sections to `RedbBackend` rustdoc |
| `Value::Float` deferral + version-chain deferral not called out in module doc | `benten-core-guardian` M1 + M2 | Added explicit deferred-to-Phase-1 paragraphs at the top of `benten-core` and on the `Value` enum doc |
| `GraphError` leaks redb bias; trait doesn't express error polymorphism | `code-reviewer` M3 | Added Phase 1 follow-up doc on `GraphError` explaining the `type Error` reshape plan |
| `scan` returning unbounded `Vec` locks out streaming/pagination | `code-reviewer` M4 | Added Phase 1 follow-up doc on `ScanResult` + `KVBackend` explaining the iterator reshape plan |

### Deferred to Phase 1 with explicit phase targets

All deferrals live in code comments (`Phase 1 follow-up — ...`) or below.
Each entry names the Phase 1 sub-phase where the work lands.

- **P1.core.float** — Add `Value::Float(f64)` with NaN rejection, shortest-form
  encoding test, and a property test over arbitrary `f64` bit patterns.
  Source: `benten-core-guardian` M1, `determinism-verifier` Float variant. Land
  alongside the `benten-core` proptest harness in Phase 1 R3.
- **P1.core.version-chain** — Implement Anchor type, Version Node, `CURRENT`
  and `NEXT_VERSION` edge labels, and version-walking helpers per
  ENGINE-SPEC §6. Source: `benten-core-guardian` M2. Phase 1 R5 work for the
  `benten-core` crate.
- **P1.core.proptest** — Add `proptest` dev-dep and a `prop_node_roundtrip_cid_stable`
  property test (100k+ instances) exercising hash → decode → re-hash. Source:
  `benten-core-guardian` untagged-enum hazard, `determinism-verifier` proptest
  gap. Phase 1 R3 when the test harness lands.
- **P1.ci.wasm-runtime** — Run `print_canonical_cid` under
  `wasm32-wasip1` via wasmtime in CI and assert the output matches the
  committed fixture. Source: `determinism-verifier` cross-target-wasm. Phase 1
  CI setup.
- **P1.ci.multi-arch** — CI matrix across `{aarch64-apple-darwin,
  x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu}`
  reproducing the fixture CID. Source: `determinism-verifier` cross-arch. Phase
  1 CI setup.
- **P1.ci.msrv** — CI builds at both MSRV (1.85) and latest stable, both
  reproducing the fixture. Source: `determinism-verifier` cross-Rust-version.
  Phase 1 CI setup.
- **P1.graph.error-polymorphism** — Reshape `KVBackend` so each backend can
  surface its own error type (`type Error: std::error::Error + Send + Sync + 'static`),
  or rename `GraphError::Redb` to a neutral `Backend(String)` and preserve the
  `redb::Error` source chain via `#[source]`. Source: `code-reviewer` M3, m13.
  Phase 1 R5 work on `benten-graph`.
- **P1.graph.scan-iterator** — Reshape `scan` to return an iterator instead of
  a Vec. Consider a `Scan` type or `Box<dyn Iterator<Item = Result<_, _>>>`.
  Source: `code-reviewer` M4. Phase 1 R5.
- **P1.graph.open-vs-create** — Separate `RedbBackend::open_existing` from
  `RedbBackend::open_or_create`; default the former. Source: `code-reviewer` m14.
  Phase 1 R5.
- **P1.graph.transaction-primitive** — Expose a `transaction(|tx| ...)`
  closure-based API or `WriteBatch` builder that accepts heterogeneous
  operations. Co-designed with the transaction primitive in `benten-eval`.
  Source: `code-reviewer` single-key-txn finding. Phase 1 R5.
- **P1.graph.node-store-trait** — Lift `put_node`/`get_node` off `RedbBackend`
  into a generic `NodeStore: KVBackend` trait with a blanket impl, so node
  helpers are not coupled to redb. Source: `code-reviewer` m20. Phase 1 R5.
- **P1.graph.doctests** — Add `# Examples` doctests to the `KVBackend` trait
  methods and `RedbBackend`. Source: `code-reviewer` m16. Phase 1 R5.
- **P1.graph.stress-tests** — Multi-MB value round-trip, concurrent
  reader+writer, failure-injection atomicity test. Source: `code-reviewer` m17
  (remainder). Phase 1 R3.

### Disagreed (with rationale)

None. Every finding is accepted as either a spike-stage fix or a Phase 1
deferral. The critics did not surface any claim we consider incorrect.
