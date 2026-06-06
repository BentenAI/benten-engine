# benten-crypto-suite — Internals

A plain-English, code-grounded tour of the `benten-crypto-suite` crate — the **13th workspace crate** added in Phase-4-Meta-Core (G-CORE-2 canary at PR #1307 minting the crate; G-CORE-3a CANARY at PR #1319 minting `KeyMaterial` + `AeadEnvelope` + `structural_kdf` + X-Wing-hybrid wrap; G-CORE-3c at PRs #1330/#1332 building the bidirectional swap matrix). Read-only audit. Audience: a developer landing in this crate fresh and trying to find the load-bearing seams. Last refreshed: 2026-05-24 against main HEAD `a0b75637` (post `phase-4-meta-core/r4b-r1-fix-pass` base `4bbc4cac`).

---

## 1. What this crate does

`benten-crypto-suite` is **the ONE thin Benten-owned signature / hash / cipher-suite agility integration crate** per CLAUDE.md baked-in #5 + the `RATIFIED-crypto-agility-2026-05-18.md` + `RATIFIED-pq-default-reframe-2026-05-19.md` settlements. It is the **only crypto-primitive call site** in the workspace: every other crate that needs to sign, verify, hash, derive a key, wrap or unwrap an AEAD ciphertext routes through this crate's *typed* APIs. Nothing else instantiates a primitive directly.

Three load-bearing properties hold:

1. **Never fork / never reimplement primitives.** This crate is concat / hash / codepoint / envelope **GLUE** only, over vetted upstream RustCrypto-class primitive crates (`ed25519-dalek`, `ml-dsa`, `slh-dsa`, `x25519-dalek`, `chacha20poly1305`, `hkdf`, `sha2`, `sha3`, `blake3`, `ml-kem`). The integration glue is Benten-owned; the algorithm bodies are not.

2. **Never hardcode key / sig / ciphertext sizes.** Sizes are reported dynamically by codepoint at runtime (`crate::sizes`). The v1-PRODUCTION correctness property is that the PQ-default reframe shipped *before* G-CORE-9 freeze precisely because ML-DSA-65 dims (~1952 B key / ~3309 B sig) flow on the **default** path under the reframe, and no caller assumes a 64-byte signature any more.

3. **Codepoint dispatch has a typed `UnsupportedAlgorithm` arm — NEVER a silent fallback.** Unknown algorithm codepoints fail closed at the parsing boundary. P2P-mainstream choice per Veilid / MLS-RFC9420 / Nostr-NIP-44; age's silent-ignore behavior is the deliberately-rejected outlier.

The PQ-default reframe (per CLAUDE.md baked-in #5 + #15 + `RATIFIED-pq-default-reframe-2026-05-19.md`) makes **PQ-hybrid the v1-beta DEFAULT** for both signature (Ed25519⊕ML-DSA-65 at codepoint `0x0001`) and encryption (X25519⊕ML-KEM-768 at codepoint `0x647a` + ChaCha20-Poly1305 bulk). Classical-only / no-encryption / non-PQ downgrades are real, built, swappable downgrades (the full swap matrix at `swap_matrix.rs`). The safety invariant is the **classical floor**: the hybrid construction means unaudited PQC is never the SOLE trust path; the audited classical half holds even if the PQ half is later broken. The combiner is strip-resistant (committing combiner at `sig::hybrid_combine` + the X-Wing combiner for KEM).

---

## 2. Dependency chain

**Workspace deps (in):** `benten-errors` (typed errors flow through the catalog), `benten-core` (CidShape parsing through the varsig boundary).

**External deps:** `blake3` (default hash), `sha2` (`SHA-512/256` agile fallback), `sha3` (`SHA3-256` agile fallback), `ed25519-dalek` (classical signature), `ml-dsa` (FIPS-204 ML-DSA-65 PQ signature), `slh-dsa` (FIPS-205 SLH-DSA hash-sig PQ tier-2 — buildable NF-1 arm), `x25519-dalek` (classical KEM), `ml-kem` (FIPS-203 ML-KEM-768 PQ KEM), `chacha20poly1305` (bulk AEAD), `hkdf` (HKDF-SHA256 structural KDF), `subtle` (constant-time comparisons), `zeroize` (secret-clearing). Vendored: ~30-LOC X-Wing combiner inline at `cipher_suite::xwing_combine` (per Spike I — too small to crate-vendor; the IRTF draft is implementation-by-hand at this size).

**Consumers (out):** the `crypto-agility-contract:6` only-call-site rule means consumers route through the typed APIs only. Current consumers:
- `benten-id` — `did:key` minting + sig verify dispatch
- `benten-caps` — UCAN signature verify + AuthorizationGrant binding-sig verify + AeadEnvelope wrap/unwrap for grant key-material
- `benten-graph` — per-Node AEAD wrap via `aead_wrap.rs` (G-CORE-3d); two-CID map (`two_cid_map.rs`) for plaintext-CID → ciphertext-CID
- `benten-drop` — `AuthorizationGrant` carrier + envelope-Ed25519 sig over bundle header
- `benten-engine` — `Engine::walk_share_scope` delegates to canonical walker; AeadEnvelope unwrap on read pathways
- `benten-sync` — AEAD wrap on outbound sync frames

**Crates explicitly NOT reached:** all primitive crates are this crate's *internal* dependency surface. Direct consumption of `ed25519-dalek`, `ml-dsa`, etc., from any other workspace crate is forbidden per the only-call-site rule and arch-tested.

**Features:** none (default feature closure intentional — the PQ + classical paths both ship in the v1-beta binary unconditionally; no `--no-default-features` shrink path).

---

## 3. Files inventory in `src/` (~6.0k LOC across 14 files)

### 3a. Public dispatch surface

- **`lib.rs`** (153 LOC) — module declarations + public re-exports + the crypto-agility contract crate-level doc. Pub re-exports flow `AeadEnvelope` / `AeadError` / `AeadKeyMaterial` / codepoint enums / `CryptoError` / `UnsupportedAlgorithm` / `VerifyError` / `HashSeam` / `HybridSignature` / `SignatureSuite` / `SuiteConfig` / `StructuralKdfKey` + `derive_root` + `derive_step` / the full `swap_matrix::*` surface for the G-CORE-3c bidirectional swap-matrix tests.

- **`codepoint.rs`** (270 LOC) — the **5-table codepoint enum** locked at G-CORE-9 V1-FROZEN-INTERFACE item 6. Hash codepoints: BLAKE3 `0x1e` (default), SHA2-512/256 `0x1015`, SHA3-256 `0x16` (agile fallbacks). Signature codepoints: `HYBRID_ED25519_MLDSA65 = 0x0001` (v1-beta default), `CLASSICAL_ED25519 = 0x0002` (downgrade), `HYBRID_MLDSA65_SLHDSA = 0x0003` (NF-1 PQ⊕PQ non-default), `HYBRID_MLKEM768_HQC = 0x647b` reserved, `PURE_PQ_MLKEM768_ONLY = 0x647c` audit-gated. KEM/encryption codepoints: `HYBRID_X25519_MLKEM768 = 0x647a` (v1-beta default, X-Wing-style combiner), `CLASSICAL_X25519 = 0x6400` (downgrade), `NONE_PLAINTEXT = 0x0000` (downgrade — explicit no-encryption arm). Each codepoint enum carries `from_raw(u16) -> Self` (infallible const constructor; NOT a support guarantee) + `resolve(self) -> Result<(), UnsupportedAlgorithm>` typed dispatch (the typed-unsupported arm surfaces here — never a silent fallback).

- **`primitives.rs`** (48 LOC) — leaf type re-exports + the seam types that other crates name.

- **`sizes.rs`** (279 LOC) — **dynamic per-codepoint size reporting**. Every key / sig / ciphertext size is reported by a function call on the codepoint, never by a `const N: usize`. Test arms assert ML-DSA-65 returns ~1952 B key / ~3309 B sig; classical Ed25519 returns 32 B key / 64 B sig; the production code reads these dynamically.

### 3b. Hash + signature seams

- **`hash.rs`** (81 LOC) — `HashSeam` trait + the codepoint-dispatched `hash(codepoint, bytes)` entry-point. The BLAKE3 default path covers CIDv1 + content addressing; the SHA-2 / SHA-3 arms are agile fallbacks. Hash PQ is a non-issue (Grover quadratic-only; 256-bit safe indefinitely) so this seam was UNAFFECTED by the 2026-05-19 PQ-default reframe.

- **`sig.rs`** (548 LOC) — `SignatureSuite` trait + `SuiteConfig` + the codepoint-dispatched verify path. The classical Ed25519 arm calls `ed25519-dalek` directly; the hybrid Ed25519⊕ML-DSA-65 arm builds a `HybridSignature` (concatenated payload with the classical-first byte ordering + length-prefixed sub-fields) and verifies BOTH halves on the verify path (both-must-verify; strip-resistant per the committing combiner construction `hybrid_combine`). The NF-1 PQ⊕PQ arm (ML-DSA-65⊕SLH-DSA) is implemented as a non-default codepoint for future activation.

- **`varsig.rs`** (198 LOC) — UCAN-Varsig parsing + multiformats framing. Threads the codepoint dispatch into the Varsig wire-bytes shape (per G-CORE-9 R3-FP Bundle 10 D-9 sub-task naming the per-codepoint hex byte-pin test family).

### 3c. AEAD + KEM + KDF

- **`aead.rs`** (450 LOC) — the AEAD envelope + per-codepoint AEAD wrap/unwrap. Owns `AeadEnvelope` (the wire-bytes envelope carrying nonce + ciphertext + AAD); `AeadError`; `AeadKeyMaterial` (the wrapper distinct from `GrantKeyMaterial` per the #1344 rename); `IROH_BLOCK_SIZE = 16 * 1024` (golden constant pin at line 443 enforced by `assert_eq!(IROH_BLOCK_SIZE, 16 * 1024)` per V1-FROZEN-INTERFACE item 15(g)); `WHOLE_CONTENT_AEAD_THRESHOLD = 64 * 1024` (per-chunk-AEAD threshold per RATIFIED-S&C §R6 Spike H+1.2; below this, whole-content AEAD; at-or-above, per-chunk AEAD chunked at IROH_BLOCK_SIZE). The AAD layout is the 4-segment binding `benten-aead:chunk: || plaintext_cid || chunk_index (u64 BE) || total_chunks (u32 BE)` (`aad_per_chunk`) — the `total_chunks` truncation defense is BUILT (the R6 R1 retraction of Fork 1 replaced the earlier 2-tuple `(plaintext_cid, chunk_index)` layout). The integers are big-endian (M-19).

- **`cipher_suite.rs`** (739 LOC) — `CipherSuite` trait + per-codepoint suite implementations + the inline ~30-LOC X-Wing combiner (per Spike I). The X25519⊕ML-KEM-768 hybrid encapsulation path generates: an X25519 ephemeral KEM share, an ML-KEM-768 encapsulation, and combines the two shared secrets via X-Wing per the IRTF draft. The combiner is committing (binds the combiner output to both KEM ciphertexts via a SHA3-256 transcript hash).

- **`structural_kdf.rs`** (213 LOC) — **Two-path key-derivation contract** (Interpretation B per Spike E + R0.8 correction). HKDF-SHA256 as the v1-beta default. `derive_root(k_principal, root_cid) -> StructuralKdfKey` uses `info = "root" || root_cid`. `derive_step(k_predecessor, edge_label, child_cid) -> StructuralKdfKey` uses `info = "step" || edge_label || child_cid`. The "step"/"root" info-tag string is the cross-role domain separator (lines 124 + 141). `StructuralKdfKey` is zeroize-on-drop. A test-only `derive_step_without_info_tag_for_test` escape at line 96 exercises the failure-mode in regression tests (gated to `#[cfg(any(test, feature = "_for_test"))]`).

- **`discharge.rs`** (111 LOC) — discharge helpers for clearing key material across boundaries.

### 3d. Boundary + swap matrix

- **`boundary.rs`** (274 LOC) — the "everything that crosses an FFI / wire boundary goes through here" choke point. Threads `UnsupportedAlgorithm` typed errors across the napi binding + the wire bytes.

- **`swap_matrix.rs`** (2014 LOC, the largest file in the crate) — the **full bidirectional swap matrix** built at G-CORE-3c (PRs #1330/#1332) per CLAUDE.md baked-in #5 + #15. Covers all 9 default-and-downgrade combinations across sig × encryption: PQ-hybrid sig × PQ-hybrid enc (default × default), classical sig × default enc, no-enc × default sig, etc., plus the NF-1 PQ⊕PQ non-default arm. Each swap-matrix arm has a conformance test pin at `tests/tf4_gcore3c_swap_matrix_conformance*.rs`. The bidirectional property is: any party sending under any swap arm can be received + verified by any party reading that arm; bidirectional swap is the v1-beta correctness contract that the conformance tests enforce.

- **`error.rs`** (103 LOC) — `CryptoError` (top-level error type for the suite) + `UnsupportedAlgorithm` (the typed-reject arm at every codepoint dispatcher) + `VerifyError`. All flow through the workspace `benten-errors::ErrorCode` catalog per the #5 crypto-agility refinement.

---

## 4. Test pins of note (in `tests/`)

- `tf3a_*.rs` family — G-CORE-3a CANARY arms; KeyMaterial + AeadEnvelope wire-bytes pins.
- `tf4_gcore3c_swap_matrix_conformance*.rs` — the bidirectional swap-matrix conformance suite per V1-FROZEN-INTERFACE item 6.
- `tf4_*.rs` family — broader swap-matrix arm sweeps.
- `p2p_interop_conformance_*.rs` — invariant pins per V1-FROZEN-INTERFACE item 14 (mandatory baseline + typed-unsupported-error never silent fallback + additive-codepoint + old-codepoints supported forever + IANA component ID reuse). Discoverable via `git ls-files crates/benten-crypto-suite/tests/p2p_interop_conformance*.rs`.

---

## 5. Phase-4-Meta-Core wave provenance

- **G-CORE-2 (PR #1307)** — crate minted; signature-agility integration substrate per CLAUDE.md baked-in #5. ~101 pub additions; biggest single-PR public-surface add this phase.
- **G-CORE-3a CANARY (PR #1319)** — `KeyMaterial` + `AeadEnvelope` types minted; `structural_kdf` module + X-Wing-hybrid wrap; CATALOG 170→171 (RecipientLacksKeysForSuite).
- **G-CORE-3c (PRs #1330/#1332)** — bidirectional swap matrix built + conformance-tested; CATALOG mint AuditNotLandedPurePqRejected (the C11b safety gate `try_pure_pq_sole_trust_path` returning typed-reject at v1-beta until the independent audit lands per NF-2 / C-GM-AUDIT).
- **G-CORE-3d (PR #1323)** — per-Node AEAD wrap moved to `benten-graph::aead_wrap`; this crate kept the per-chunk-AEAD constants + envelope shape. The AAD was later extended (R6 R1 Fork-1 retraction) from the 2-tuple to the 4-segment `aad_per_chunk` binding total_chunks (the truncation defense; big-endian per M-19).
- **G-CORE-9 R1/R2/R3/R4/R4b (PRs #1346/#1347/#1348/#1349)** — V1-FROZEN-INTERFACE item 6 lock; the 5 cipher-suite tables + the 11-codepoint authoritative enum are wire-format-locked; the `pub fn` surface enters the cargo-public-api required-failing baseline.
- **#1344 GrantKeyMaterial vs AeadKeyMaterial rename** — type-level distinction (the rename is per V1-FROZEN-INTERFACE item 15(d) — prevents the compatible-interpretation trap).
- **PR #1342 §3.5g item-6 amendment** — `ErrorCode::DslIoError`-style pub-error-variant first-class mirrors flow through this crate via the workspace drift-detector at `scripts/drift-detect-error-variant-mirror.ts`.

---

## 6. Load-bearing seams (where future-Phase-4-Meta-Composing work plugs in)

- **`SignatureSuite` + `CipherSuite` traits** — additive new arms land by minting a new codepoint + adding a trait impl + extending the swap-matrix conformance suite. The crypto-agility contract permits this without breaking existing wire bytes.
- **`SuiteConfig`** — runtime-selected suite (default = PQ-hybrid; classical-only is a downgrade arm a deployment opts into).
- **`HashSeam`** — additional hash codepoints land as additive arms (the BLAKE3 default + SHA-2 + SHA-3 agile fallbacks are pre-blessed).
- **`structural_kdf`** — the two-path key derivation is the substrate for per-Node AEAD (`benten-graph::aead_wrap`) + per-chunk AEAD (`aead.rs`). The info-tag codepoint-binding extension is a wire-format-coupled change deferred to G-COMP-1 per V1-FROZEN-INTERFACE-DEFERRED Row D-13.
- **Audit-gated PQ-only arm** — `try_pure_pq_sole_trust_path` returns `AuditNotLandedPurePqRejected` at v1-beta; flips to permit when NF-2 / C-GM-AUDIT lands per CLAUDE.md baked-in #15.

---

## 7. v1-FROZEN-INTERFACE.md coupling

This crate's public surface IS frozen as part of **V1-FROZEN-INTERFACE.md item 6** (signature + encryption boundaries + swap matrix + 5-codepoint table). The cargo-public-api baseline at `docs/public-api/benten-crypto-suite.txt` is the byte-stable v1-beta surface; drift fails CI per the G-CORE-9 R1 Fork 3 FREEZE-FLIP.

V1-FROZEN-INTERFACE item 15(d) names `AuthorizationGrant` carrying the `GrantKeyMaterial` (the type distinct from this crate's `AeadKeyMaterial`); item 15(f) names the two-path KDF contract this crate implements; item 15(g) names the AEAD chunk-size + AAD layout this crate locks.
