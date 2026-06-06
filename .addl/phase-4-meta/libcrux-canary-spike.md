# libcrux-ml-kem cross-target build + FIPS-203 byte-equality canary

**Spike date:** 2026-06-05
**Goal:** De-risk adopting `libcrux-ml-kem` (Cryspen hax/F\*-verified constant-time
ML-KEM-768) as a replacement-or-companion for the in-tree RustCrypto `ml-kem`,
BEFORE wiring it into `benten-crypto-suite`. Reproduce-don't-assume: every result
below is from an actual build/run in a detached scratch crate, not reasoning.

**Method:** throwaway detached crate `.spike-libcrux/` (empty `[workspace]` table,
NOT part of the benten workspace). Deps: `libcrux-ml-kem = "0.0.9"` +
`ml-kem = { version = "0.2", default-features = false, features = ["deterministic"] }`
(matches the workspace pin `ml-kem 0.2.3`). Toolchain: Rust 1.95.0 (workspace MSRV).

---

## libcrux-ml-kem version
**`0.0.9`** (latest on crates.io as of 2026-06-05; resolved via `cargo add`).
RustCrypto comparison impl: **`ml-kem 0.2.3`** (the exact workspace-pinned version).

> ⚠️ Note the `0.0.x` version line — libcrux-ml-kem has NOT cut a 0.1/1.0. This is a
> maturity/stability signal (API + SemVer churn risk), independent of the
> correctness result below, which is excellent.

## BUILD
| Target | Result | Notes |
|---|---|---|
| `native` (aarch64-apple-darwin) | **PASS** | clean, 14.6s cold |
| `wasm32-unknown-unknown` | **PASS** | clean, 9.6s — **the load-bearing thin-client target** |
| `wasm32-wasip1` | **PASS** | clean, 3.9s |

**No FAILs.** All three targets compile with default features AND with the
minimal `default-features = false, features = ["mlkem768"]` subset (separately
verified in a second probe crate, then removed).

Why wasm works (the usual ML-KEM-in-wasm blockers, all absent here):
- **build.rs is portability-correct.** `build.rs` auto-enables `simd128` only on
  `target_arch == "aarch64"` and `simd256` only on `x86_64`. On `wasm32` neither
  fires, so the **portable (formally-verified, constant-time) backend is selected
  automatically**. No C, no asm, no intrinsics on wasm. The SIMD backends additionally
  carry runtime CPU-feature checks (`libcrux-platform`), so they're never blind-called.
- **No `getrandom` blocker.** The deterministic API (`generate_key_pair([u8;64])`,
  `encapsulate(pk, [u8;32])`, `decapsulate(sk, ct)`) takes caller-supplied
  randomness. `getrandom` is **absent from the wasm32-unknown-unknown dependency
  tree entirely** (both default-feature and minimal-feature builds). This is exactly
  the `getrandom` `js`/`wasm_js` feature-flag pain that has bitten benten's wasm
  builds before — it does NOT recur here. (`rand 0.10` is pulled transitively by
  `libcrux-traits`, but without its OS-RNG feature, so it compiles on wasm cleanly.)

## BYTE-EQUALITY vs RustCrypto FIPS-203
Full deterministic KAT: fixed seeds `d=[0x02;32]`, `z=[0x03;32]` (keygen),
`m=[0x07;32]` (encaps). Both impls expose a FIPS-203 deterministic path
(libcrux `generate_key_pair(d‖z)` + `encapsulate(pk, m)`; RustCrypto
`generate_deterministic(&d,&z)` + `encapsulate_deterministic(&m)`).

| Field | Expected | rc len | lc len | bytes identical |
|---|---|---|---|---|
| encapsulation-key | 1184 | 1184 | 1184 | **YES** |
| ciphertext | 1088 | 1088 | 1088 | **YES** |
| decapsulation-key | 2400 | 2400 | 2400 | **YES** |
| shared-secret | 32 | 32 | 32 | **YES** |

- **ALL FOUR fields byte-identical** for the same deterministic seeds.
- libcrux's `MlKemPrivateKey` serializes as the **2400-byte expanded FIPS-203 `dk`
  form** (`CPA_sk 1152 + ek 1184 + H(ek) 32 + z 32`), identical to RustCrypto's
  2400-byte decapsulation key — the canonical on-wire form benten already stores.
- **Cross-impl interop (both directions): YES**
  - libcrux-encap → RustCrypto-decap → same shared secret: **YES**
  - RustCrypto-encap → libcrux-decap → same shared secret: **YES**

This is the strongest possible **NQ-C2** confirmation: the two impls are
byte-for-byte interchangeable. Every on-wire ML-KEM-768 byte + every frozen
golden survives a swap unchanged. A node running libcrux and a node running
RustCrypto interoperate transparently — supporting a phased/dual rollout if ever
wanted, with zero wire risk.

## RECOMMENDED INTEGRATION SHAPE
**SINGLE-IMPL-SWAP.** `libcrux-ml-kem 0.0.9` builds on ALL three benten targets
(native + wasm32-unknown-unknown + wasm32-wasip1) AND is byte-identical +
bidirectionally interoperable with RustCrypto `ml-kem 0.2.3` under FIPS-203. No
native-libcrux / wasm-RustCrypto dual is required; the dual fallback is available
for free (interop is proven) but unnecessary.

## RISKS / NOTES (wiring-affecting)
- **Maturity / SemVer:** `0.0.9` (pre-0.1). Expect API/SemVer churn; pin exactly +
  vendor-lock the audited version. This is the single biggest adoption caveat — the
  *correctness* is verified-grade, but the *release stability* is early.
- **build.rs exists** (host-side only; sets `simd128`/`simd256` cfgs by target-arch).
  It does NOT break cross-compilation (runs on host), but CI/sccache should be aware
  a build-script runs. Env overrides exist: `LIBCRUX_DISABLE_SIMD128/256`,
  `LIBCRUX_ENABLE_SIMD128/256` — useful to force the portable backend deterministically
  in CI if reproducible-build/constant-time auditing wants one codepath everywhere.
- **Constant-time on the portable (wasm) backend:** portable code uses `libcrux-secrets`
  secret-typed integers; by default they transparently fall back to std integers, and
  the optional `check-secret-independence` feature turns ON compile-time
  secret-independence checking. CT is "best-effort + assembly-inspection-validated"
  (their words) — same honesty class as RustCrypto. The portable + AVX2 field
  arithmetic / NTT / serialization / generic high-level code is **formally verified via
  hax + F\***; this is libcrux's headline advantage over RustCrypto.
- **Feature surface:** use `default-features = false, features = ["mlkem768"]` for the
  leanest build (drops mlkem512/1024 + `std`; keeps the deterministic API benten uses).
  Verified to build native + wasm with `getrandom` absent.
- **`rand` coupling:** `rand 0.10` is a transitive dep of `libcrux-traits` even in
  minimal builds, but without OS-RNG features (no `getrandom`) — no wasm randomness
  wiring needed. benten's existing call sites already supply deterministic
  `(d, z)` / `m` seeds, so the randomized `rand`-feature API isn't on the critical path.
- **MSRV:** no `rust-version` pin declared (edition 2021). Compiles clean under the
  workspace's Rust 1.95.0.
- **Transitive deps added:** `hax-lib`/`hax-lib-macros` (proc-macro, build-time only),
  `libcrux-sha3`, `libcrux-secrets`, `libcrux-traits`, `libcrux-intrinsics`,
  `libcrux-platform`, `tls_codec(+derive)`, `keccak`. Adds ~a dozen crates to the
  graph vs RustCrypto's leaner tree — a supply-surface consideration for `cargo deny`
  / `cargo vet` (all Cryspen-authored or well-known).

### Bottom line
A clean single-impl swap is viable and low-risk **on the technical axis** (builds
everywhere, byte-identical, interoperable, formally verified). The one real gating
concern is **release maturity (0.0.9)** — recommend pinning exactly, vetting the
exact version, and treating the eventual independent ml-kem audit (NF-2 / C-GM-AUDIT)
as covering whichever impl is pinned at v1-GM time.
