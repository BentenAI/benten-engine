# Option F+ §6.2-envelope-unification — LENS L4 implementation-engineering review

**Branch:** `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering`
**Reviewer lens:** **Senior implementation engineer / impl-bug-surface / CT-impl-hazard / FFI marshaling / formal-methods-targetability.**
**Distinct from:** L1 (cryptographer / construction soundness), L2 (second-opinion cryptographer / amendments 1+2), L3 (adversarial-design / amendments 3-6).
**Question I answer that L1-L3 did not:** *what will actually go wrong when one contributor + AI orchestration tries to ship the §6.2-with-Amendments design in Rust + TS in 7-15 weeks?*

**Inputs ground-truth-verified** (every cite below is anchored to a SHA-pinned branch; I `git show <branch>:<path>` verified):
- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — L1 NO-GO + §6.2 sketch.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — L2 CONCUR-WITH-AMENDMENTS (Amendments 1+2).
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — L3 CONCUR-WITH-CALIBRATION (Amendments 3+4+5+6 + minor 7+8).
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — F-full scope + wave sequencing.
- `main` — current `crates/benten-crypto-suite/src/aead.rs` + `cipher_suite.rs` + `codepoint.rs` (existing X-Wing wrap + ChaCha20-Poly1305 AEAD impl is the substrate §6.2 sits on top of).
- `docs/INVARIANT-COVERAGE.md` Inv-15 (signature 3-layer decomposition; Inv-16 mint pending this design freeze).
- `docs/SECURITY-POSTURE.md` Compromise #30 (PQ-impl-audit-maturity) + #31 (forever-valid drops).
- RFC 9180 (HPKE); RFC 8439 (ChaCha20-Poly1305); RFC 9106 (Argon2); FIPS 203 (ML-KEM); RFC 9420 (MLS); RFC 8949 (CBOR §4.2 deterministic encoding).
- **libcrux-ml-kem / hax / Cryspen** — verified-secret-independent ML-KEM in Rust ([cryspen.com/post/ml-kem-implementation](https://cryspen.com/post/ml-kem-implementation/), [github.com/cryspen/libcrux/tree/main/libcrux-ml-kem](https://github.com/cryspen/libcrux/tree/main/libcrux-ml-kem)).
- **RustCrypto `ml-kem` v0.2.x** — pure-Rust FIPS 203 ([github.com/RustCrypto/KEMs](https://github.com/RustCrypto/KEMs); [crates.io announce](https://users.rust-lang.org/t/ann-ml-kem-v0-2-0-pure-rust-implementation-of-the-fips-203-final-post-quantum-kem-construction-formerly-known-as-kyber/116111)).
- **`dudect-bencher`** + **`ctgrind`** — CT-validation tooling for Rust ([lib.rs/crates/dudect-bencher](https://lib.rs/crates/dudect-bencher); [crates.io/crates/ctgrind](https://crates.io/crates/ctgrind)).
- **getrandom wasm32-unknown-unknown** — wasm_js feature requirement ([docs.rs/getrandom](https://docs.rs/getrandom); [issue #268](https://github.com/rust-random/getrandom/issues/268)).
- **hax** — F* / Rocq backend for Rust crypto ([eprint.iacr.org/2025/142](https://eprint.iacr.org/2025/142.pdf)).
- **Cybernetist on Tokio cancel-safety** ([cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)) + **Oxide RFD 400** on cancel-safety.

**Authority:** ADVISORY. Final decision rests with Ben. Downstream DISAGREE-WITH-EXPLANATION first-class per `feedback_review_finding_ground_truth_verify`.

---

## 0. Reading order

5-minute read: §1 (verdict) → §2.1 (the load-bearing impl-trap I found that survives Amendments 1-6) → §5 (proposed impl-amendments) → §7 (honest cost).

§2 is the bulk of the work — impl-bug-surface enumeration per amendment + per layer.
§3 is CT-validation infra recommendation.
§4 is formal-methods targetability.
§6 is crate-pin recommendations.
§7 is the honest cost estimate.
§8 is self-assessment + confidence calibration.

---

## 1. Executive verdict

**CONCUR-WITH-IMPL-AMENDMENTS** on §6.2-with-Amendments-1-through-8. Recommendation strength: **HIGH** on the design being implementable in 7-15 weeks under v1-beta timeline with the impl-amendments below. The three prior cryptographer reviews collectively produced a cryptographically-sound design freeze; my lens surfaces a set of **implementation-engineering hazards** that are orthogonal to the construction-soundness questions and that — if not pre-empted in the R3/R5 implementer briefs — will produce CI-cycle burn, subtle correctness drift, or shipped vulnerabilities (CT-impl-regression + nonce-reuse-bug-class + Argon2-cross-target-perf-skew) at v1-beta freeze.

**Headline impl-engineering amendments (load-bearing for v1-beta-shippable impl):**

- **IMPL-A1 (load-bearing): ML-KEM crate selection MUST be `libcrux-ml-kem`, not RustCrypto `ml-kem`.** Cryspen's `libcrux-ml-kem` is **formally verified for panic freedom, correctness, AND secret independence** in F* via the hax toolchain ([cryspen.com](https://cryspen.com/post/ml-kem-verification/), [pq-code-package/rust-libcrux](https://github.com/pq-code-package/rust-libcrux)). The `check-secret-independence` feature enables compile-time CT verification via `libcrux-secrets` integer types. **This directly closes L3 Amendment 6** (Bernstein-Persichetti CT-Decap) at the crate level instead of requiring a CT-Decap audit pass against `ml-kem` v0.2.x — RustCrypto `ml-kem` documents constant-time-best-effort but does NOT provide compile-time secret-independence verification. The cost delta is near-zero (both crates expose FIPS 203 Decap with the same shape) but the assurance delta is enormous. The current `crates/benten-crypto-suite/src/cipher_suite.rs` uses `ml-kem` v0.x via the `KemCore`/`MlKem768` API; the migration to `libcrux-ml-kem` is a ~50-LOC crate swap. **Confidence: HIGH.**

- **IMPL-A2 (load-bearing): nonce-discipline MUST be counter-mode-with-restart-guard, not random-12-byte.** ChaCha20-Poly1305 with a 96-bit nonce has a birthday bound at 2^32 messages-per-key under random nonces. Layer-A vault unlock + per-Node Layer-B re-encryptions cumulatively easily hit 2^20-2^25 over a vault's lifetime; with multiple vaults across a user's devices the aggregate per-key-equivalence-class reuse becomes a real concern. **Use XChaCha20-Poly1305 (24-byte nonce; 2^96 birthday bound) for ALL Layer-A + Layer-B sites where the same key is reused across messages.** HPKE-mode-base sites (Layer-C + Layer-D) are unaffected (HPKE manages its own nonce-counter internal to the KeyScheduleContext). The current `aead.rs` uses ChaCha20-Poly1305 with random nonces; switching to XChaCha20-Poly1305 is a 5-LOC API swap (RustCrypto `chacha20poly1305::XChaCha20Poly1305`). **L3 §2.8 row (l) flagged this as an "impl concern"; I elevate to load-bearing.** Confidence: HIGH.

- **IMPL-A3 (load-bearing): Amendment 1's `canonical_binding()` + Amendment 3's TLV encoding MUST live in ONE crate that BOTH Rust and TypeScript call, via NAPI-RS, NOT be reimplemented in TS.** Re-implementing canonical serialization in TS guarantees cross-language drift (per `feedback_pim_cross_language_rule_mirror` §3.5g — Benten has 5+ historical instances). The TS surface MUST call into the Rust `canonical_binding()` via NAPI-RS, return `Buffer`, and treat the bytes as opaque. Confidence: HIGH.

- **IMPL-A4 (load-bearing): zeroize discipline MUST extend across the NAPI-RS boundary.** When Rust hands a `Buffer` of secret bytes (e.g., a decrypted K_principal during vault-unlock returning to a JS caller for further use) to V8, V8's GC owns the lifetime. `zeroize::Zeroizing<Vec<u8>>` on the Rust side does NOTHING for the V8-side copy. Either (a) NEVER return raw secret bytes across NAPI; instead return an opaque handle that the Rust side owns + zeroes on drop, OR (b) accept the V8-side leak and document it as a known Compromise. **Recommend (a) for K_principal + DAK + HPKE recipient sk; accept (b) for decrypted plaintext Nodes (the data the user is consuming anyway).** Confidence: HIGH.

- **IMPL-A5 (load-bearing): wasm32-unknown-unknown target requires `wasm_js` getrandom feature behind a gated cfg.** Per `getrandom` docs, wasm32-unknown-unknown does NOT auto-route to `Crypto.getRandomValues`; library code must NOT enable the `wasm_js` feature directly (it breaks non-Web wasm builds per [issue #268](https://github.com/rust-random/getrandom/issues/268)). Benten must add `[target.wasm32-unknown-unknown] rustflags = ['--cfg', 'getrandom_backend="wasm_js"']` at the **application** layer (Tauri shell + browser-shell crate), NOT at `benten-crypto-suite` library level. Forgetting this produces a runtime panic on first `OsRng::fill_bytes` call in the browser. Confidence: HIGH.

- **IMPL-A6 (load-bearing): Tokio cancel-safety for AEAD/HPKE encrypt paths — wrap every Seal/Open in `tokio::task::spawn_blocking` OR document that crypto futures are NOT cancel-safe.** A `tokio::select!` with a crypto future on one arm dropped mid-encryption leaves the AEAD context in a not-fully-zeroed state (the partial keystream + per-block state is on the heap; Drop runs but the cancellation may happen between zeroize-on-drop and the cleanup of `Aead::encrypt`'s internal buffers). For Benten's primary surfaces (vault unlock + drop seal/open) these are CPU-bound short ops that should NEVER be `select!`'d; document this as a §3.5 dispatch-conventions rule + add a clippy lint via `disallowed_methods`. Confidence: HIGH.

**Headline impl-engineering amendments (recommended but non-blocking on v1-beta):**

- **IMPL-B1: codepoint endianness on-wire** — L3 Amendment 7 pins BE; CURRENT `crates/benten-crypto-suite/src/aead.rs` uses **LE u16** on the wire (line 22 of the existing impl: *"bytes 2-3: cipher codepoint (LE u16; e.g. 0x647a for X-Wing-hybrid)"*). **This is a load-bearing wire-format conflict the prior reviewers did not catch.** Either (a) Amendment 7 is amended to LE to preserve the existing AEAD envelope wire format (no migration), OR (b) the AEAD envelope wire format is changed to BE at G-CORE-9 wire-freeze (1-line change + golden-vector regen). I recommend (b) — BE is network-byte-order canonical + matches the proposed `canonical_binding()` shape — and document the bump in `aead.rs` line 23 commentary. **This requires a Ben decision; tagging as IMPL-DECISION-PENDING.** Confidence on the finding: HIGH; recommendation: MEDIUM-HIGH on (b) over (a).

- **IMPL-B2: golden-test-vector corpus generation** — every codepoint × every amendment × every primitive needs ≥3 golden vectors (round-trip, tampered-AAD-rejects, replay-window-expired-rejects). Generate via a one-time Rust binary `cargo run --bin gen-crypto-golden-vectors` producing a versioned JSON corpus under `crates/benten-crypto-suite/tests/golden/`. The TS surface validates the same corpus via NAPI-RS round-trip. ~8 codepoint variants × ~4 properties × ~3 vectors = ~96 vectors; ~1 wave-day of work. Confidence: HIGH.

- **IMPL-B3: dudect-bencher integration in CI** — add a nightly-only `cargo bench --bench ct_validation` workflow that runs dudect over Decap + canonical_binding + AAD-verify; fail the build if Welch's t-test rejects constant-time at p<0.01 over 10⁶ samples. Cost: ~2 wave-days infra + ~1 wave-day per primitive. The L3 Amendment 6 deferral compromise (#32) closes faster if CI sentinel is in place. Confidence: MEDIUM-HIGH.

- **IMPL-B4: Formal-verification of `canonical_serialize_tlv` via kani** — Amendment 3's injectivity property is a natural kani/F* target. Spec: `∀ (a, b: BindingContext), a.canonical_serialize_tlv() == b.canonical_serialize_tlv() ⇒ a == b`. kani can verify by enumerated-variant bounded check + harness; ~2 wave-days. Confidence: MEDIUM-HIGH on tractability; HIGH on value (it removes the entire class of L3 §2.1 collisions).

---

## 2. Impl-bug-surface enumeration

### 2.1 LOAD-BEARING TRAP: codepoint-LE-vs-BE wire format conflict between Amendment 7 and existing G-CORE-3a code

**The finding.** L3 Amendment 7 mandates **big-endian** codepoint encoding on-wire. The existing production code in `crates/benten-crypto-suite/src/aead.rs` documents the wire format as:

```text
bytes 2-3: cipher codepoint  (LE u16; e.g. 0x647a for X-Wing-hybrid)
```

The two prior reviewers (L2 § Amendment 1 sketch + L3 § Amendment 7) BOTH wrote `self.codepoint.to_be_bytes()`. Neither checked whether this matches the existing AEAD envelope wire format. **The conflict is real.** If `EncryptedEnvelope::canonical_binding()` writes the codepoint BE into AAD but the AEAD envelope writes the codepoint LE on the wire, two scenarios diverge:

1. **Envelope sees BE codepoint in AAD; AEAD wire-format has LE codepoint.** The AAD doesn't match the wire-format codepoint byte-for-byte. An adversary who flips the wire-format codepoint bytes (e.g., changes `0x7a, 0x64` LE = 0x647a → `0x64, 0x7a` reads as 0x7a64 if parsed naive-LE) gets a parse-mismatch — but the AAD-binding still encodes 0x647a (BE). **The codepoint-in-AAD defense (Amendment 1) only works if the AAD codepoint matches the on-wire codepoint canonically.**

2. **Aggregate effect:** the cite-drift between "codepoint as written in `aead.rs` documentation" vs "codepoint as bound in `canonical_binding()`" creates a future maintainer hazard. Per `feedback_pim_cross_language_rule_mirror` §3.5g this is a 6-instance-class drift that has cost the project ~4-6 CI rounds historically.

**Resolution options:**

- **(a) Migrate `aead.rs` wire-format to BE u16.** ~5-LOC change in `aead.rs` line 22 + matching change in encode/decode + regen all G-CORE-3a golden vectors + bump `ENVELOPE_FORMAT_VERSION_V1 = 0x01` → `0x02`. Net cost: ~1 wave-day. Result: codepoint-in-AAD matches codepoint-on-wire; cite-drift closed.
- **(b) Amend Amendment 7 to LE u16.** ~1-line spec amendment. Result: codepoint matches; preserves existing G-CORE-3a wire format. Less idiomatic (network-byte-order is BE).

**Recommendation: (a).** BE is the network-byte-order canonical (RFC 791 §3.1). All IETF crypto protocols (TLS, HPKE, MLS) use BE multi-byte integers. Migrating G-CORE-3a to BE pre-wire-freeze is cheap; living with LE forever is a permanent oddity. This is an IMPL-DECISION-PENDING for Ben.

**Confidence on the finding: HIGH** (verified by reading the actual `aead.rs` source). **Confidence on recommendation: MEDIUM-HIGH** (depends on whether G-CORE-3a is past wire-freeze; per the source comment "G-CORE-9 freezes" suggests not-yet-frozen, so migration is cheap).

---

### 2.2 Layer-A (Vault) impl-bug-surface

#### 2.2.1 ChaCha20-Poly1305 nonce reuse hazard ⇒ IMPL-A2 (XChaCha20-Poly1305)

ChaCha20-Poly1305's 12-byte nonce gives ~2^32 birthday bound under random nonces. Layer-A vault may re-encrypt under the same DAK across:

- Every K_principal rotation event (when user changes password).
- Every Layer-B per-Node K(N) derivation that uses DAK as KDF input.
- Vault-snapshot backup operations.

Aggregate per-DAK message count over a 5-year vault lifetime conservatively ~2^20-2^25. Random nonce collision probability at 2^25 messages = 2^25 / 2^48 = 2^-23 (Poly1305 nonce-reuse breaks confidentiality + authenticity simultaneously). NOT acceptable for a v1-beta wire-format-freeze.

**Mitigation (IMPL-A2):** XChaCha20-Poly1305 with 24-byte nonce gives 2^96 birthday bound. RustCrypto provides `chacha20poly1305::XChaCha20Poly1305` with identical API. ~5-LOC swap.

**Counter alternative:** counter-based nonce (per-DAK monotonic counter + restart-guard) is theoretically tighter but adds operational complexity (counter persistence, atomic-increment across processes). XChaCha20 is the boring-crypto-is-best choice.

**Confidence: HIGH.** Industry consensus (age uses ChaCha20-Poly1305 with random nonces but each file gets a fresh random file-key, so aggregate-reuse is bounded by `# of files * 1 nonce`; Benten's design re-uses DAK across multiple seals so the bound doesn't hold).

#### 2.2.2 Argon2id parameter pinning across deployment shapes ⇒ tiered-params with explicit-pinning

OWASP-recommended Argon2id: `m_cost=46 MiB, t_cost=1, p_cost=1` per RFC 9106 §4. **But:** Benten targets macOS arm64 + Linux x86_64 + Windows x86_64 + wasm32 + mobile. The 46 MiB memory cost is fine on desktop/laptop; on wasm32 in a browser tab, allocating 46 MiB synchronously may trigger heap-OOM on memory-constrained mobile browsers (iOS Safari caps tab memory ~256 MiB).

**Trap:** if a single Argon2id parameter set is pinned across all deployments, vault unlock either (a) breaks on mobile/wasm with OOM, or (b) is too-weak on desktop because params are mobile-tuned.

**Mitigation:** Tier parameters at vault-creation-time, store the tier in the vault metadata header:

```rust
pub enum Argon2idParameterTier {
    Mobile { m: 19 * 1024, t: 2, p: 1 },         // OWASP minimum
    Desktop { m: 46 * 1024, t: 1, p: 1 },        // OWASP recommended
    Server { m: 256 * 1024, t: 2, p: 4 },        // Future
}
```

Vault metadata stores tier; unlock recomputes Argon2id with the stored tier. Cross-device vault portability works if any device can run the highest-tier Argon2id (note: a vault created on Desktop and copied to Mobile may take longer to unlock on Mobile; document this).

**Subtle hazard:** the tier value itself MUST be AAD-bound (per Amendment 1) so an adversary cannot downgrade Mobile tier → an even-weaker tier on a tampered vault.

**Confidence: HIGH** on the multi-tier need; MEDIUM-HIGH on the specific tier values (depends on empirical perf measurement on each target).

#### 2.2.3 Argon2id Drop+zeroize discipline

The RustCrypto `argon2` crate does NOT automatically zeroize its working memory after hash computation — the password bytes are passed by reference and the caller controls lifetime; the Argon2id matrix (46 MiB) IS allocated internally and zeroed via `Drop` IF the impl uses `Zeroizing<Vec<u8>>` internally. Per docs.rs/argon2, this is best-effort.

**Hazard:** if the Argon2id matrix lives in unzeroed heap after hash returns, a memory dump captures the matrix + can re-derive seed (which derives K_principal).

**Mitigation:** verify the chosen `argon2` crate version's Drop impl; add a `zeroize-on-drop` wrapper if needed; add a test that uses `mem::forget` + heap-inspection to verify zero-out.

**Confidence: MEDIUM-HIGH** on the discipline needed; LOW on whether it's automatic in the crate (verify at impl time).

#### 2.2.4 HKDF key derivation chain zeroize

`hkdf::Hkdf<Sha256>::extract` + `expand` use internal HMAC state. RustCrypto's `hkdf` crate uses `Zeroizing` for the PRK internally (verified docs.rs/hkdf §0.12). Should be fine; verify at impl time.

---

### 2.3 Layer-B (per-Node AEAD) impl-bug-surface

#### 2.3.1 K(N) = KDF(K_principal, N.cid) hazards

`K_principal` is rotated; CID is content-addressed. If K_principal rotates and the existing Layer-B-encrypted Nodes are NOT re-encrypted, old Nodes become un-decryptable (Compromise vs feature: Benten's design intent is unclear at write time; verify).

**Mitigation:** Layer-B re-encryption pass at K_principal rotation, OR maintain K_principal version history in vault. Either way, K(N) derivation must include K_principal version (currently CID-only).

#### 2.3.2 Per-Node nonce reuse across Layer-B sites

`K(N) = KDF(K_principal, N.cid)` is per-Node-deterministic. Same CID across two encrypts (e.g., re-encrypt on Node-update) MUST use different nonces. If implementation uses random nonces, see §2.2.1 (XChaCha20 needed). If counter-based, per-Node counter persistence is required.

#### 2.3.3 Chunk-index binding (already in `aead.rs`)

Existing `aead.rs` notes "AAD binds (plaintext-CID, chunk-index) for rebinding-attack defense" — verified. This is correctly aligned with §6.2 envelope; the chunk-index must be in `canonical_binding()`'s AAD for Layer-B variants. Add to BindingContext:

```rust
pub enum BindingContext {
    // ...
    PerNodeAead {
        node_cid: Cid,
        chunk_index: u32,
        k_principal_epoch: u32,  // for rotation handling
    },
}
```

Currently §6.2 doesn't include a Layer-B BindingContext variant (Layer-B was scoped out by L1-L3 as "existing design"); recommend adding for codepoint-dispatch uniformity per Inv-16. Confidence: HIGH on need; MEDIUM on the specific shape.

---

### 2.4 Layer-C (drop) impl-bug-surface

#### 2.4.1 HPKE-mode-base context lifecycle: single-shot vs multi-message

RFC 9180 §5 defines two modes for AEAD: single-shot (`AuthSeal/AuthOpen`, single message per context) vs multi-message (the HPKE context survives and can encrypt multiple messages with internal nonce counter). The McMillion `rust-hpke` crate exposes both APIs.

**Hazard:** Layer-C drops are typically single-message (a Drop encrypts ONE payload to ONE recipient). Single-shot is correct. **BUT** if a future Layer-C use case naturally fits multi-message (e.g., a drop-stream), and the impl conflates the two, the HPKE context's internal nonce counter might be wrongly reset.

**Mitigation:** make the §6.2 HpkeBase variant explicitly single-shot at the type level (no exposed context handle); a future HpkeStream variant minted at its own codepoint admits multi-message.

#### 2.4.2 ML-KEM Decap implicit-rejection-key-handling ⇒ IMPL-A1 (libcrux)

FIPS 203 §6.3 mandates implicit rejection: on malformed ciphertext, Decap returns a deterministic key K' indistinguishable from a true K under constant-time observation. **The "constant-time" qualifier is load-bearing for Bernstein-Persichetti resistance.**

- RustCrypto `ml-kem` v0.2.x: documents "constant-time best-effort" — no formal CT verification.
- Cryspen `libcrux-ml-kem`: **formally verified for secret independence** via hax/F*, with `check-secret-independence` feature for compile-time CT verification.

**Mitigation: IMPL-A1.** Switch from `ml-kem` to `libcrux-ml-kem` in `cipher_suite.rs`. Migration cost ~50 LOC.

#### 2.4.3 HPKE info-string canonicalization

L2 Amendment 1's `canonical_binding()` becomes the HPKE `info` parameter for the HpkeBase variant. RFC 9180 §5.1 specifies that info is bound into the KeyScheduleContext via `LabeledExtract`. If encoder + decoder canonicalize differently (e.g., differing CBOR map-key order per L3 §2.1 RFC 8949 §4.2.1 vs §4.2.2 ambiguity), the decoder's info mismatches encoder's → KeyScheduleContext diverges → AEAD-Open fails.

**Mitigation:** pin canonicalization to "CTAP2 Canonical CBOR" (RFC 8949 §4.2.2) explicitly + add a test that mutates one byte of `info` and asserts auth-fail. Add a cross-impl conformance test if Benten ever ships a non-Rust HPKE impl.

#### 2.4.4 Drop persistence + Compromise #31 + Amendment 5 interaction

L3 Amendment 5 binds `sealed_at + valid_until` into AAD for DeviceLink + RemotePermission BUT NOT for DropToRecipient. Compromise #31 explicitly states drops are forever-valid. Acceptable per design but: Bernstein-Persichetti's per-Decap chosen-ciphertext attack scales with how many times Decap is called. Forever-valid drops + cloud-backup-replay means Eve can rerun Decap arbitrarily often against the recipient's ML-KEM sk.

**Mitigation:** Compromise #32 mint (L3 Amendment 6) + IMPL-A1 (libcrux verified CT-Decap). The combination structurally addresses the attack.

---

### 2.5 Layer-D (device-link + remote-permission) impl-bug-surface

#### 2.5.1 Sender-DID binding (Amendment 4) FFI marshaling

`DeviceDid` and `Did` are Benten-defined types. Their canonical serialization MUST be deterministic; their NAPI-RS marshaling MUST go through a single Rust call (per IMPL-A3).

**Trap:** if TS code constructs a `DeviceDid` JSON-string via JS template-literal interpolation + passes to NAPI, the string-form is the canonical form for AAD. If TS uses `did:key:z6Mk...` and Rust normalizes to `did:key:Z6MK...` (case-folding), AAD doesn't match. Mitigation: case-sensitive identity on the wire; document in CRYPTO-CODEPOINTS.md.

#### 2.5.2 Provisioning session lifecycle hazards

`provisioning_session_id: [u8; 16]` is random per session. **Trap:** if the provisioning flow is async + cancellable (user cancels mid-pairing), the session_id may persist in some side-state (peer-mesh announcement, partial backup) and an attacker can replay it later. L3 Amendment 5's `valid_until` closes this provided the provisioning flow sets a tight window (~5 minutes).

**Mitigation:** strict TTL enforcement at decoder level; reject expired sessions before invoking primitive. The "before invoking primitive" detail is critical for IMPL-A6 (Tokio cancel-safety) — if expiry check is itself awaited, cancellation drop the check.

#### 2.5.3 Multi-device-key-wrap implementation

Layer-D wraps K_principal to N target devices. Each wrap is one HPKE-mode-base envelope. **Hazard:** if implementation loops with `for device in devices { wrap(K_principal, device.pk) }` synchronously, side-channel-resistant impls of HPKE-Seal could leak per-device timing patterns. **Boring fix:** use a fixed-iteration loop with fresh ephemeral randomness per iteration; use `subtle::Choice` for any cross-device decision branches.

---

### 2.6 Amendment-3 (TLV) impl-bug-surface

#### 2.6.1 Length-encoding ambiguity in `canonical_serialize_tlv`

L3 Amendment 3 specifies "u32-BE length prefix" but doesn't pin **whether the length includes the length-prefix-bytes itself**. Two conventions:

- TLV (Tag-Length-Value): length is the byte count of Value only.
- TLV-inclusive: length includes Length-field bytes.

If two impls disagree, AAD diverges → auth-fail. **Mitigation: pin "Value-only" length (matches RFC 9180 §4.1 `labeled_extract`).** Document in CRYPTO-CODEPOINTS.md + add cross-version golden vectors.

#### 2.6.2 `canonical_serialize_tlv` for nested enums

`PermissionOperation` is itself an enum with multiple variants, some carrying further nested fields. TLV encoding must recursively length-prefix at every level. Naive implementation may length-prefix only at top level. **Mitigation:** unit-test every nested-enum variant produces distinct bytes from sibling variants under the canonical serialization (kani-target, see §4).

#### 2.6.3 Test-vector requirement: collision-injectivity

L3 §2.1 worked example: `scope=[0xAA,0xBB,0xCC] || audience=Did("did:key:foo")` vs `scope=[0xAA,0xBB,0xCC,'d','i','d',':','k','e','y',':','f','o','o'] || audience=Did("")`. Test: BOTH canonicalize via TLV; assert resulting bytes differ. This is a single proptest invocation; ~30-LOC.

---

### 2.7 Amendment-4 (sender-DID) impl-bug-surface

#### 2.7.1 Did type ambiguity: which DID is "sender"?

For DeviceLink: `sender_device_did` is the LOCAL device's DID. For RemotePermission: `requesting_device_did` AND `granting_user_did` are BOTH bound. This is correct but **implementer-confusion-hazard**: which DID goes where? The struct field names help; a unit-test asserting "an envelope produced by Bob's device contains Bob's device_did as sender_device_did" verifies this.

#### 2.7.2 Did equality semantics for AAD-verification

DIDs may have multiple canonical forms (e.g., did:key has compressed vs uncompressed pubkey-form). AAD verification MUST use byte-equality on the canonical form. Mitigation: `Did::canonical()` method called before AAD assembly; document in Inv-16 phrasing.

---

### 2.8 Amendment-5 (replay-window) impl-bug-surface

#### 2.8.1 Clock skew tolerance: what value?

L3 Amendment 5 mandates `now() <= valid_until + clock_skew_tolerance` but doesn't pin the tolerance. Common values: 60s (TLS), 5min (NTP). **Recommend: 60s.** Document in CRYPTO-CODEPOINTS.md.

#### 2.8.2 Monotonic clock vs wall clock

`SystemTime::now()` on Linux is wall-clock (NTP-skewable). A user with a backwards-skewed clock could open valid envelopes after expiry. A user with forward-skewed clock could fail-to-open valid envelopes. **Mitigation:** accept the wall-clock semantics for envelope-expiry (this is what the protocol means by "valid_until"). Add a system-clock-sanity-check at unlock time (warn if clock differs from peer-mesh consensus by >5min).

#### 2.8.3 Clock manipulation as DoS

Adversary who controls user's system clock can either expire-everything (DoS) or never-expire (replay vulnerability). The Benten architecture doesn't fully solve this; document as a known Compromise (perhaps fold into existing #30/#31).

---

### 2.9 Amendment-6 (Bernstein-Persichetti CT-Decap) impl-bug-surface

#### 2.9.1 RustCrypto ml-kem vs libcrux-ml-kem — IMPL-A1

Already covered. Switching to libcrux-ml-kem closes Amendment 6 at the crate level.

#### 2.9.2 Decap CT regression on future crate updates

Even with libcrux-ml-kem, future version bumps could introduce a regression. **Mitigation:** dudect CI sentinel (IMPL-B3) flagging variance > Bernstein-Persichetti per-query leakage bound. Pin `libcrux-ml-kem` version with `=x.y.z` (not `^x.y`) until CI sentinel is in place.

#### 2.9.3 Compromise #32 mint (per L3 Amendment 6 option-c)

Belt-and-suspenders disclosure even with libcrux CT verification. Recommend mint.

---

### 2.10 Cross-cutting impl-bug-surface

#### 2.10.1 NAPI-RS marshaling of secret types

Per IMPL-A4: NEVER return raw secret bytes (K_principal, DAK, recipient sk) across NAPI boundary. Return opaque handles that the Rust side owns.

```rust
#[napi]
pub struct VaultHandle { inner: Arc<Mutex<VaultState>> }

#[napi]
impl VaultHandle {
    #[napi]
    pub fn unlock(password: String) -> Result<Self> { /* zeroize password after use */ }
    #[napi]
    pub fn decrypt_node(&self, cid: String) -> Result<Buffer> { /* plaintext OK to return */ }
}
```

Plaintext bytes (the Node content the user is consuming) are OK to return; secrets that derive other secrets are NOT.

#### 2.10.2 Tokio cancel-safety (IMPL-A6)

Wrap crypto ops in `spawn_blocking` if they need to be select'd:

```rust
let plaintext = tokio::task::spawn_blocking(move || cipher.decrypt(&nonce, ciphertext)).await??;
```

Add a clippy lint via `.cargo/config.toml` `disallowed_methods` for direct AEAD calls in async context.

#### 2.10.3 wasm32 perf for ML-KEM + Argon2

- ML-KEM-768 KeyGen on wasm32 (no SIMD by default): ~30-50ms (vs ~5-10ms on native).
- Argon2id at OWASP-Mobile params on wasm32: ~500-1000ms (vs ~100-300ms on native).

**Implication:** vault-unlock on browser-shell takes ~1-2s. Acceptable UX; document.

#### 2.10.4 cargo-llvm-cov instrumentation interferes with CT validation

Coverage instrumentation injects branches + counter-increments into compiled code. This is BENIGN for correctness tests but POISONS dudect-bencher measurements (the injected counters change per-branch-taken). **Mitigation:** run CT-validation jobs WITHOUT `cargo-llvm-cov` (separate CI workflow); explicitly exclude `ct_validation` benches from coverage runs. Per `feedback_pim_test_isolation_process_scoped_shared_state` §3.13 precedent: coverage instrumentation has historically broken process-scoped sentinels in Benten's CI.

#### 2.10.5 Test isolation per `feedback_pim_test_isolation_process_scoped_shared_state`

The shared `OsRng` thread-local state could interfere across parallel tests. Per Benten's pim-N-test-isolation discipline, use per-test deterministic seeds for golden-vector tests (`ChaCha20Rng::from_seed`) and OsRng only for runtime-random tests.

#### 2.10.6 Test-coverage gaps

- **How to test "AAD is bound" without mutating internal AEAD state:** seal with AAD=A, attempt open with AAD=A' (differing by 1 byte), assert auth-fail. This works without AEAD-state mutation; ~10-LOC proptest.
- **How to test "strict-decode rejects" without false-positive:** construct two ciphertext blobs (one Vault-shape, one Drop-shape); attempt decode-as-Vault on the Drop blob; assert ParseError before invoking AEAD. Strict-decode catches the parse before any crypto. ~15-LOC.
- **How to test "ML-KEM Decap is constant-time":** dudect-bencher harness over `decap(sk, valid_ct)` vs `decap(sk, malformed_ct)` over 10⁶ samples; Welch's t-test < 0.01 ⇒ PASS. ~50-LOC harness; ~1 wave-day.

---

## 3. CT-validation infrastructure recommendations

### 3.1 Layered approach

| Tool | Purpose | Cost | When |
|---|---|---|---|
| **libcrux-ml-kem `check-secret-independence`** | Compile-time CT verification of ML-KEM ops via libcrux-secrets type tagging. | ZERO (feature flag). | ALWAYS (every CI build). |
| **`dudect-bencher`** | Statistical CT verification (Welch's t-test) at runtime on selected ops. | ~50-LOC harness/primitive; ~5-30 min/run; nightly CI. | Per-primitive: Decap, AEAD-decrypt, canonical_binding, replay-window-check. |
| **`ctgrind` (valgrind extension)** | Dynamic CT verification via uninit-tainting; catches secret-dependent branches/memory accesses. | ~20-LOC harness/primitive; ~5x slower; nightly CI. | Per-primitive: complementary to dudect. |
| **proptest for AEAD-AAD-binding invariants** | Property-based testing of structural invariants (Amendment 1, 3 injectivity, 5 expiry). | ~30-LOC/property; runs in `cargo test`. | EVERY PR. |
| **Golden-vector corpus** | Cross-version + cross-language conformance. | ~96 vectors; ~1 wave-day one-time. | EVERY PR (regression-only). |

### 3.2 Specific CI workflows to add

1. **`.github/workflows/ct-validation.yml`** (nightly + on-release): runs `cargo bench --bench ct_validation` + `cargo build --features check-secret-independence` (libcrux); fails build if either gates. Cost: ~30min/run.

2. **`.github/workflows/golden-vectors.yml`** (every PR): runs `cargo test -p benten-crypto-suite --test golden_vectors`; validates the corpus byte-exact. Cost: ~30s/run.

3. **`.github/workflows/ct-validation.yml` MUST EXCLUDE cargo-llvm-cov instrumentation.** Per IMPL §2.10.4. The workflow uses a fresh checkout + plain `cargo bench`, not `cargo llvm-cov bench`.

### 3.3 Recommendation cost summary

- One-time infra setup: ~3-5 wave-days (4 workflows + harnesses + golden-vector generator).
- Per-primitive marginal cost: ~0.5-1 wave-day for dudect + ctgrind harnesses.
- Recurring cost: ~30-60 min/nightly CI; ~10-30s additional per PR.

---

## 4. Formal-methods targetability assessment

### 4.1 Most-tractable target: Amendment-3 `canonical_serialize_tlv` injectivity

**Claim:** `∀ a, b: BindingContext. a.canonical_serialize_tlv() == b.canonical_serialize_tlv() ⇒ a == b`.

**Tool: kani.** Bounded model checker over Rust MIR. The BindingContext enum has 4 variants × ~3 fields each = ~12 field types; each field has bounded-shape canonical encoding. kani harness: enumerate two BindingContext instances under bounded parameters, assert `serialize_a == serialize_b ⇒ a == b`. Bounded by Did length ≤ 256 bytes + scope ≤ 1024 bytes (reasonable for v1-beta).

**Tractability: MEDIUM-HIGH.** kani handles bounded recursive types well. The Did + PermissionOperation nested enum encoding may require manual lemma decomposition. Estimated cost: **2 wave-days** to set up + write harness + iterate to passing.

**Value: HIGH.** Closes the entire L3 §2.1 collision class. Once proved, future maintainers cannot accidentally break injectivity (kani harness regenerates on every PR).

### 4.2 Second-tractable target: Amendment-1 AAD-binding round-trip

**Claim:** `∀ env. decode(encode(env)) == env ∧ canonical_binding(env) is total over BindingContext`.

**Tool: kani or proptest+shrink.** Proptest is cheaper (1 wave-day); kani gives stronger guarantees (3 wave-days). Recommend proptest for v1-beta; kani as v1-GM stretch.

### 4.3 Third-tractable target: ML-KEM secret-independence

**Already covered by libcrux-ml-kem hax verification.** No additional Benten effort needed; just enable the feature flag.

### 4.4 NOT tractable for v1-beta

- IND-CCA2 reduction proof for HPKE-mode-base[X-Wing] in Benten's adversary model. Would need a paper-grade proof; out of scope.
- AEAD-AAD-binding security against IND-CCA2 adversary. Same.
- Replay-window enforcement under adversary control of system clock. Modeled, not provable.

### 4.5 Formal-methods cost summary

**Recommended for v1-beta:** kani on `canonical_serialize_tlv` injectivity (~2 days) + proptest on AAD-binding round-trip (~1 day). Total: ~3 days. Confidence: HIGH on tractability.

**Recommended for v1-GM:** kani on AAD-binding round-trip (~3 days) + hax-style F* extraction of `canonical_binding` (~5 days). Total: ~8 days. Confidence: MEDIUM-HIGH.

---

## 5. Impl-amendments (concrete additions to §6.2)

Consolidating the load-bearing impl-amendments from §1:

### 5.1 IMPL-A1: crate selection `libcrux-ml-kem` not RustCrypto `ml-kem`

```toml
# crates/benten-crypto-suite/Cargo.toml
libcrux-ml-kem = { version = "=0.0.x", features = ["check-secret-independence"] }
# REMOVED: ml-kem = "0.2"
```

Migration in `cipher_suite.rs`: ~50 LOC. API shape similar (Encapsulate/Decapsulate traits exist in both).

### 5.2 IMPL-A2: XChaCha20-Poly1305 for Layer-A + Layer-B

```rust
// aead.rs
use chacha20poly1305::{XChaCha20Poly1305, XNonce}; // 24-byte nonce
pub enum EnvelopePayload {
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 24] },  // CHANGED from [u8; 12]
    HpkeBase { enc: Bytes, ciphertext: Bytes },
}
```

Layer-C/D HPKE-mode-base manages its own nonce internally; unchanged.

### 5.3 IMPL-A3: single-source canonical_binding across Rust + TS

The `canonical_binding()` function lives in `benten-crypto-suite::envelope`; TS side calls via NAPI-RS `EncryptedEnvelope::canonicalBinding()` returning `Buffer`. NEVER reimplement in TS.

Add `#[napi]` binding + integration test that exercises round-trip across NAPI boundary.

### 5.4 IMPL-A4: opaque-handle pattern for secrets at NAPI boundary

```rust
#[napi]
pub struct VaultHandle { inner: Arc<Mutex<VaultState>> }
// Never return K_principal/DAK/sk via NAPI; only return decrypted plaintext.
```

### 5.5 IMPL-A5: wasm_js feature gating

```toml
# .cargo/config.toml (in Benten shell crates, NOT in benten-crypto-suite library)
[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```

Document in CLAUDE.md baked-in #17 (all-three-deployment-shapes).

### 5.6 IMPL-A6: Tokio cancel-safety discipline

Add `clippy.toml` entry:
```toml
disallowed-methods = [
    { path = "chacha20poly1305::ChaCha20Poly1305::encrypt", reason = "wrap in spawn_blocking" },
    # ... etc for every AEAD/HPKE method
]
```

Plus a `dispatch-conventions §3.5` rule: "Crypto ops in async context MUST be wrapped in `tokio::task::spawn_blocking`."

### 5.7 IMPL-B1: codepoint endianness migration in aead.rs (IMPL-DECISION-PENDING)

Migrate `aead.rs` wire format from LE u16 to BE u16. Regenerate G-CORE-3a golden vectors. Bump `ENVELOPE_FORMAT_VERSION_V1 → V2`. Confidence: MEDIUM-HIGH; defer to Ben.

### 5.8 IMPL-B2: golden-vector corpus

Generate ~96 vectors via `cargo run --bin gen-crypto-golden-vectors`. Place under `crates/benten-crypto-suite/tests/golden/`. ~1 wave-day.

### 5.9 IMPL-B3: dudect-bencher CI integration

Nightly workflow; ~3 wave-days infra + per-primitive harnesses.

### 5.10 IMPL-B4: kani injectivity proof

`canonical_serialize_tlv` injectivity via kani; ~2 wave-days.

---

## 6. Recommended Rust-crate selection + version pins

Building on the 3 tactical picks already ratified (keyring-core v1.0.0 / McMillion `rust-hpke` / Signal-Provisioning protocol shape):

| Crate | Version pin | Rationale | Audit-status |
|---|---|---|---|
| `libcrux-ml-kem` | `=0.0.x` (latest as of write-time; pin exact) | Formally verified secret-independence; hax/F*; pq-code-package multi-vendor stewardship. | hax-verified panic-freedom + correctness + secret-independence. ([cryspen.com](https://cryspen.com/post/ml-kem-verification/), [pq-code-package/rust-libcrux](https://github.com/pq-code-package/rust-libcrux)) |
| `x25519-dalek` | `=2.0.x` | Constant-time scalar mult documented; mature; widely audited. | Multiple audits (Trail of Bits, NCC Group; via dalek ecosystem). |
| `chacha20poly1305` (RustCrypto) | `=0.10.x` | Standard. Use `XChaCha20Poly1305` variant per IMPL-A2. | Reviewed; widely deployed. |
| `argon2` (RustCrypto) | `=0.5.x` | OWASP-canonical; rust-argon2 alternative exists but RustCrypto is the ecosystem standard. | Reviewed. |
| `hkdf` (RustCrypto) | `=0.12.x` | Standard; tested. | Reviewed. |
| `hpke` (McMillion `rust-hpke`) | `=0.13.x` | Already chosen per F-full ratification. RFC 9180 conformant. | Self-audit by author; community-reviewed. Per L3 §7.4 caveat: NOT `hpke-rs` (Verification-Theatre 2026-02-05). |
| `zeroize` | `=1.8.x` | Standard. `Zeroize` + `ZeroizeOnDrop` derive macros. | Reviewed. |
| `subtle` | `=2.5.x` | `Choice` + constant-time selection. | Reviewed; dalek ecosystem. |
| `sha2` / `sha3` | `=0.10.x` / `=0.10.x` | Standard. SHAKE-128/256 used by X-Wing combiner. | Reviewed. |
| `getrandom` | `=0.3.x` (or 0.2.x as ecosystem catches up) | OsRng backend. **MUST** configure `wasm_js` feature for wasm32-unknown-unknown per IMPL-A5. | Reviewed. |
| `keyring` / `keyring-core` | `=1.0.x` | Already ratified. OS keychain integration. | Per F-full review §15. |
| `napi-rs` | `=2.x` | Already used in Benten. | Mature; production-deployed. |

**Crates explicitly NOT chosen:**
- `ml-kem` (RustCrypto/KEMs) — replaced by libcrux-ml-kem per IMPL-A1.
- `hpke-rs` (cryspen) — explicitly excluded per Verification Theatre 2026-02-05 + L3 §7.4.
- `rust-argon2` (sru-systems) — RustCrypto `argon2` preferred for ecosystem consistency.

**Drift defense:** add to `cargo deny` allow-list + cite-drift-detector scanner for `Cargo.toml` mentioning any non-listed crypto crate.

---

## 7. Honest cost estimate for v1-beta-shippable §6.2-with-Amendments-1-through-8

### 7.1 Implementation LOC

| Component | LOC | Tests LOC | Wave-days |
|---|---|---|---|
| `EncryptedEnvelope` struct + variants | ~250 | ~400 | 1.5 |
| `canonical_binding()` + `canonical_serialize_tlv()` | ~200 | ~300 | 1.0 |
| Layer-A vault: Argon2id + XChaCha20-Poly1305 + tiered params | ~300 | ~500 | 2.0 |
| Layer-B per-Node K(N) KDF + XChaCha20-Poly1305 | ~150 | ~300 | 1.0 (mostly existing) |
| Layer-C drop: HPKE-mode-base seal/open via rust-hpke + libcrux-ml-kem | ~250 | ~400 | 2.0 |
| Layer-D device-link + remote-permission seal/open | ~350 | ~500 | 2.5 |
| Strict-decode dispatch table | ~150 | ~250 | 1.0 |
| Replay-window enforcement (Amendment 5) | ~100 | ~200 | 0.5 |
| Sender-DID binding (Amendment 4) | ~100 | ~200 | 0.5 |
| NAPI-RS bindings (opaque-handle pattern) | ~300 | ~400 | 2.0 |
| TS-side surface (call into NAPI; do NOT reimplement crypto) | ~400 | ~600 | 2.0 |
| Codepoint registry doc + cite-drift scanner extension | ~50 | ~100 | 0.5 |
| Compromise #32 mint + SECURITY-POSTURE.md update | ~30 | n/a | 0.25 |
| Inv-16 mint + INVARIANT-COVERAGE.md update | ~50 | n/a | 0.25 |
| migrate aead.rs LE→BE codepoint (IMPL-B1) + golden-vector regen | ~50 | ~50 | 0.5 |
| **SUBTOTAL** | **~2730** | **~4200** | **~17.5 wave-days** |

### 7.2 Test corpus + CT-validation infrastructure

| Component | Wave-days |
|---|---|
| Golden-vector corpus generator + ~96 vectors | 1.0 |
| dudect-bencher harnesses (per-primitive) | 3.0 |
| ctgrind harnesses (per-primitive) | 2.0 |
| Proptest property suite (AAD-binding, injectivity, expiry, sender-binding) | 2.0 |
| CI workflows (3 new: ct-validation, golden-vectors, fuzz) | 1.5 |
| kani injectivity proof | 2.0 |
| **SUBTOTAL** | **~11.5 wave-days** |

### 7.3 Per-platform validation

| Platform | Wave-days |
|---|---|
| macOS arm64: end-to-end perf measurement + Argon2id tier-calibration | 1.0 |
| Linux x86_64: same | 0.5 |
| Windows x86_64: same | 1.0 (keyring integration quirks) |
| wasm32: wasm_js getrandom + Argon2id tier + ML-KEM perf | 2.0 |
| Mobile (iOS/Android, deferred to v1-GM): TBD | 0 (post-v1-beta) |
| **SUBTOTAL** | **~4.5 wave-days** |

### 7.4 Documentation + audit-firm prep

| Component | Wave-days |
|---|---|
| CRYPTO-CODEPOINTS.md registry doc | 0.5 |
| SECURITY-POSTURE.md updates (Compromise #32 + Inv-16) | 0.5 |
| Audit-firm RFP scoping | 1.0 |
| **SUBTOTAL** | **~2.0 wave-days** |

### 7.5 Grand total

**~35.5 wave-days = ~7 calendar-weeks** at ~5 wave-days/week (single contributor + AI orchestration).

**Buffer for unknowns** (CI cycles + cross-target debugging + amendment-composition issues found at impl-time): **+30% = ~10 wave-days = ~2 calendar-weeks**.

**Total realistic estimate: ~9-10 calendar-weeks** for the §6.2-with-Amendments-1-through-8 substrate, golden-vector corpus, CT-validation CI, kani injectivity proof, per-platform perf calibration, and audit-firm-ready documentation.

**Fits the v1-beta 7-15 week window comfortably IF the impl-amendments are pre-resolved in R3/R5 briefs.** If they emerge at R5 review time, expect +2-3 weeks of late-discovery rework.

---

## 8. Self-assessment + confidence calibration

### 8.1 Confidence summary per finding

| Finding | Confidence | Rationale |
|---|---|---|
| IMPL-A1 (libcrux-ml-kem swap) | **HIGH** | Verified via multiple sources (Cryspen blog, pq-code-package, libcrux docs). Concrete migration target. |
| IMPL-A2 (XChaCha20 nonce-widening) | **HIGH** | RFC 8439 + RustCrypto docs explicitly call out nonce-reuse hazard. Industry consensus. |
| IMPL-A3 (single-source canonical_binding) | **HIGH** | Benten has 5+ historical cross-language drift instances per §3.5g; this is risk-mitigated. |
| IMPL-A4 (NAPI opaque-handle for secrets) | **HIGH** | V8 GC + Rust zeroize don't compose; structural finding. |
| IMPL-A5 (wasm_js getrandom config) | **HIGH** | Direct verification from getrandom docs + runtime-crash issue #268. |
| IMPL-A6 (Tokio cancel-safety wrap) | **MEDIUM-HIGH** | Class of hazard is real (Oxide RFD 400, Cybernetist post). Specific Benten reachability depends on whether crypto ops ever appear in `select!` arms. Defense-in-depth recommendation. |
| IMPL-B1 (codepoint LE→BE migration) | **HIGH** on finding; **MEDIUM-HIGH** on (a)-vs-(b) recommendation | Verified the LE inconsistency by reading aead.rs directly. The choice depends on G-CORE-3a freeze status which Ben knows better. |
| IMPL-B2 (golden-vector corpus) | **HIGH** | Standard discipline. |
| IMPL-B3 (dudect CI) | **MEDIUM-HIGH** | Tooling is mature but CI integration has historical flakiness. |
| IMPL-B4 (kani injectivity) | **MEDIUM-HIGH** on tractability; **HIGH** on value | Kani handles bounded enums well; nested enum may require lemma decomposition. |
| §7 cost estimate ~35-45 wave-days | **MEDIUM-HIGH** | Estimates from comparable crypto-substrate work (PASETO, age-rs); ±25% spread reasonable. |

### 8.2 Lower-confidence areas (honest disclosure)

1. **The specific Argon2id tier values for mobile/wasm.** §2.2.2 named OWASP recommendations but empirical measurement on actual Benten Tauri builds is required. Confidence MEDIUM.

2. **The cost of migrating from RustCrypto ml-kem to libcrux-ml-kem.** I estimated ~50 LOC based on API similarity. Actual cost could be 20-150 LOC depending on subtle API differences in error types + serialization. Confidence MEDIUM-HIGH on order-of-magnitude; LOW on exact number.

3. **kani's tractability on nested-enum injectivity.** My MEDIUM-HIGH may be optimistic. If `PermissionOperation` has variants with `Vec<Capability>` fields, kani's bounded-loop unrolling may not converge. Fallback: proptest property-test catches most collisions at lower assurance.

4. **Whether Benten's existing `aead.rs` G-CORE-3a wire format is past freeze.** I assumed not (per source comment "G-CORE-9 freezes"); Ben knows better.

5. **Whether the prior reviewers' CLAUDE.md baked-in #5 interpretation forbids switching ml-kem crates.** L1 §4.3 says "vendor-and-patch path is structurally forbidden." Switching to a vetted-upstream crate (libcrux-ml-kem) is NOT vendoring; it's choosing a different upstream. But the discipline-interpretation deserves Ben's explicit ratification.

6. **The HPKE multi-message-vs-single-shot hazard (§2.4.1).** I flagged structurally but couldn't find a concrete reachable Benten code path. Defense-in-depth recommendation.

### 8.3 What this review does NOT cover

- The IND-CCA2 reduction proof under Benten's specific adversary model (L1-L3 territory).
- Layer-B per-Node AEAD design changes beyond chunk-index binding (out of §6.2 scope).
- Audit-firm RFP scoping beyond the recommended ~$50k+ figure (project-management territory).
- Mobile (iOS/Android) keychain integration (post-v1-beta per F-full review §15).
- Multi-stanza HpkeMultiBase variant (deferred per L3 §2.7).

### 8.4 Most-likely failure mode of this review

Cross-amendment interaction with my IMPL amendments that I didn't catch. Specifically: IMPL-A2 (XChaCha20 nonce widening) changes the `EnvelopePayload::SymmetricAead` shape from `nonce: [u8; 12]` to `nonce: [u8; 24]`; if a future maintainer adds another AEAD codepoint with 12-byte nonce and naively reuses the SymmetricAead variant, type-confusion is possible. Mitigation: make the nonce length type-tied to the codepoint via newtype + add a codepoint→nonce-length lookup table.

### 8.5 Sharpest contrarian pushback I considered against myself

"You're over-engineering. Three cryptographers have already reviewed; the design is sound. Adding 6 more impl-amendments is creeping featurism."

Rebuttal: each IMPL-amendment closes a distinct implementation-class failure that the cryptographer-focused reviewers did NOT address. The cryptographer lens correctly handled construction soundness; impl-engineering lens correctly handles construction-implementation-correctness. The two are orthogonal. A design that's cryptographically sound but practically unimplementable in 7-15 weeks fails the v1-beta gate as surely as one that's cryptographically broken.

The aggregate cost of all 6 IMPL-amendments is ~5-7 wave-days incremental over a baseline "ship §6.2-with-Amendments-1-6" plan. The aggregate benefit is closing 6 distinct historically-recurring failure-mode classes (CT-regression, nonce-reuse, cross-language-drift, FFI-secret-leak, wasm-runtime-panic, cancel-safety). Each one has cost real production systems multiple incidents.

If Ben weighs minimal-scope more heavily: **the LOAD-BEARING subset is IMPL-A1 + IMPL-A2 + IMPL-A4 + IMPL-B1.** A1 closes Amendment 6 mechanically; A2 closes the nonce-reuse class; A4 closes the FFI-secret-leak class; B1 resolves the existing LE-vs-BE wire-format conflict. A3 + A5 + A6 are defense-in-depth.

### 8.6 What I would tell Ben in plain English

"The three cryptographer reviews produced a sound design freeze; eight amendments later it's ready to implement. Implementing it in Rust + TS in 7-15 weeks has six implementation-engineering pitfalls the cryptographers didn't address: pick the right ML-KEM crate (libcrux not RustCrypto's, for verified secret-independence), widen the nonce to XChaCha20-Poly1305 (12-byte is too narrow for Layer-A's lifecycle), keep the AAD canonical-serialization in ONE Rust function called from TS via NAPI (NEVER reimplement in TS), use opaque handles instead of returning raw secret bytes across NAPI (V8 GC + zeroize don't compose), configure wasm_js feature for getrandom on wasm32 (forgetting this is a runtime panic in the browser), and wrap crypto ops in spawn_blocking when in async context (cancel-safety). Plus a wire-format conflict I found: the existing aead.rs writes codepoints LE but Amendment 7 says BE — pick one and migrate. Total cost ~35-45 wave-days; fits comfortably in the 7-15 week v1-beta window IF you bake these into the R3/R5 implementer briefs."

---

## 9. Citations

### 9.1 Primary standards + drafts

- **RFC 9180** Hybrid Public Key Encryption. §4.1 labeled_extract/expand; §5.1 KeyScheduleContext; §5.2 AEAD AAD; §9.1.1 mode_base sender-auth limitation; §9.5 PSK low-entropy; §9.7.2 replay considerations. ([datatracker.ietf.org/doc/html/rfc9180](https://datatracker.ietf.org/doc/html/rfc9180))
- **RFC 9106** Argon2. §4 OWASP parameter recommendations. ([datatracker.ietf.org/doc/html/rfc9106](https://datatracker.ietf.org/doc/html/rfc9106))
- **RFC 8439** ChaCha20 + Poly1305. §2.8 AAD semantics; §4 nonce-uniqueness requirement. ([datatracker.ietf.org/doc/html/rfc8439](https://datatracker.ietf.org/doc/html/rfc8439))
- **RFC 8949** CBOR. §4.2.1 Core Deterministic Encoding; §4.2.2 CTAP2 Canonical CBOR. ([datatracker.ietf.org/doc/html/rfc8949](https://datatracker.ietf.org/doc/html/rfc8949))
- **FIPS 203** ML-KEM. §6.3 Decap implicit-rejection requirement. ([nvlpubs.nist.gov/nistpubs/fips/nist.fips.203.pdf](https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.203.pdf))
- **draft-connolly-cfrg-xwing-kem-10** X-Wing. ([datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10](https://datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10))
- **draft-irtf-cfrg-concrete-hybrid-kems-03** MLKEM768-X25519. ([datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/](https://datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/))

### 9.2 Crate references

- **libcrux-ml-kem** — pq-code-package/rust-libcrux, hax-verified. ([github.com/pq-code-package/rust-libcrux](https://github.com/pq-code-package/rust-libcrux), [github.com/cryspen/libcrux/tree/main/libcrux-ml-kem](https://github.com/cryspen/libcrux/tree/main/libcrux-ml-kem), [cryspen.com/post/ml-kem-implementation](https://cryspen.com/post/ml-kem-implementation/), [cryspen.com/post/ml-kem-verification](https://cryspen.com/post/ml-kem-verification/), [docs.rs/libcrux-ml-kem](https://docs.rs/libcrux-ml-kem/latest/libcrux_ml_kem/))
- **RustCrypto `ml-kem`** v0.2 — pure Rust FIPS 203 ([users.rust-lang.org announce](https://users.rust-lang.org/t/ann-ml-kem-v0-2-0-pure-rust-implementation-of-the-fips-203-final-post-quantum-kem-construction-formerly-known-as-kyber/116111))
- **RustCrypto `chacha20poly1305`** — including `XChaCha20Poly1305`. ([github.com/RustCrypto/AEADs/tree/master/chacha20poly1305](https://github.com/RustCrypto/AEADs/tree/master/chacha20poly1305), [docs.rs/chacha20poly1305](https://docs.rs/chacha20poly1305))
- **RustCrypto `argon2`** ([docs.rs/argon2](https://docs.rs/argon2)) — OWASP-canonical Argon2id impl.
- **RustCrypto `hkdf`** v0.12. ([docs.rs/hkdf](https://docs.rs/hkdf))
- **McMillion `rust-hpke`** — RFC 9180 conformant. ([github.com/rozbb/rust-hpke](https://github.com/rozbb/rust-hpke))
- **`x25519-dalek`** v2.x — CT scalar mult. ([docs.rs/x25519-dalek](https://docs.rs/x25519-dalek))
- **`zeroize`** v1.8 — `Zeroize` + `ZeroizeOnDrop`. ([docs.rs/zeroize](https://docs.rs/zeroize))
- **`subtle`** — `Choice` + CT selection ([docs.rs/subtle](https://docs.rs/subtle))
- **`getrandom`** wasm32 — wasm_js feature ([docs.rs/getrandom](https://docs.rs/getrandom), [github.com/rust-random/getrandom/issues/268](https://github.com/rust-random/getrandom/issues/268))
- **`napi-rs`** v2 ([napi.rs/blog/announce-v2](https://napi.rs/blog/announce-v2))

### 9.3 CT-validation + formal-methods tools

- **`dudect-bencher`** ([lib.rs/crates/dudect-bencher](https://lib.rs/crates/dudect-bencher), [docs.rs/dudect-bencher](https://docs.rs/dudect-bencher)) — Welch's t-test CT validation in Rust.
- **`ctgrind`** ([crates.io/crates/ctgrind](https://crates.io/crates/ctgrind)) — valgrind extension for uninit-taint CT validation.
- **CT-tools survey** ([crocs-muni.github.io/ct-tools](https://crocs-muni.github.io/ct-tools/))
- **Reparaz dudect** ([reparaz.net/oscar/misc/dudect](https://www.reparaz.net/oscar/misc/dudect/)) — original dudect paper.
- **hax** — IACR ePrint 2025/142, multi-prover verified Rust ([eprint.iacr.org/2025/142](https://eprint.iacr.org/2025/142.pdf))
- **kani** — Rust bounded model checker ([github.com/model-checking/kani](https://github.com/model-checking/kani))
- **Prusti** ([link.springer.com/chapter/10.1007/978-3-031-06773-0_5](https://link.springer.com/chapter/10.1007/978-3-031-06773-0_5))
- **Creusot** — prophecy-based Rust verification ([github.com/creusot-rs/creusot](https://github.com/creusot-rs/creusot))
- **Rust Verification Landscape survey** (Le Blanc 2024) ([arxiv.org/pdf/2410.01981](https://arxiv.org/pdf/2410.01981))
- **`cargo-llvm-cov`** ([github.com/taiki-e/cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov))

### 9.4 Tokio async cancel-safety

- **Cybernetist on tokio task cancellation patterns** ([cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/))
- **Oxide RFD 400 — cancel-safety in async Rust** ([rfd.shared.oxide.computer/rfd/0400](https://rfd.shared.oxide.computer/rfd/0400))
- **Tokio graceful shutdown** ([tokio.rs/tokio/topics/shutdown](https://tokio.rs/tokio/topics/shutdown))
- **CancellationToken docs** ([docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html))

### 9.5 Academic + industry references

- **Arriaga-Barbosa-Boyen "Tempo"** IACR ePrint 2025/1399 (background; ML-KEM keygen timing) ([eprint.iacr.org/2025/1399](https://eprint.iacr.org/2025/1399))
- **Bernstein-Persichetti "One Time is Enough"** IACR 2024/2051 (Decap CCA side-channel; load-bearing for IMPL-A1) ([eprint.iacr.org/2024/2051](https://eprint.iacr.org/2024/2051))
- **Malosdaf** — ChaCha20-Poly1305 nonce-reuse attack ([blog.malosdaf.me/posts/chacha20-poly1305-and-nonce-reuse-attack](https://blog.malosdaf.me/posts/chacha20-poly1305-and-nonce-reuse-attack/))
- **OWASP Password Storage Cheat Sheet** (Argon2id params) ([owasp.org](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html))
- **Argon2 adoption study** (Karasalo et al., 2024-04) ([arxiv.org/html/2504.17121v1](https://arxiv.org/html/2504.17121v1))
- **Verification Theatre on `hpke-rs`** ([symbolic.software/blog/2026-02-05-cryspen](https://symbolic.software/blog/2026-02-05-cryspen/)) — don't use `hpke-rs`.
- **PESTO** (Carbonnelle et al., NDSS 2019).
- **Bellare-Rogaway** "The exact security of digital signatures" CRYPTO 1996.

### 9.6 Benten internal references

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — L1.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — L2 (Amendments 1+2).
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — L3 (Amendments 3-8).
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — F-full scope.
- `main` — `crates/benten-crypto-suite/src/aead.rs` (LE codepoint wire format), `cipher_suite.rs` (X-Wing combiner), `codepoint.rs`.
- `docs/INVARIANT-COVERAGE.md` Inv-15 + Inv-16 mint pending.
- `docs/SECURITY-POSTURE.md` Compromise #30 + #31 + #32 (proposed mint).
- CLAUDE.md baked-in #5 (crypto-agility), #15 (v1-beta gate), #17 (deployment-shapes), #18 (authority-isolation).
- `feedback_pim_cross_language_rule_mirror` §3.5g — load-bearing for IMPL-A3.
- `feedback_pim_test_isolation_process_scoped_shared_state` §3.13 — load-bearing for §2.10.4.

---

## 10. Summary handoff to orchestrator (200 words)

**Top-line:** CONCUR-WITH-IMPL-AMENDMENTS on §6.2-with-Amendments-1-through-8 from the impl-engineering / impl-bug-surface / CT-impl-hazard / FFI / formal-methods lens. The three prior cryptographer reviews produced a sound design freeze; I add **6 load-bearing impl-amendments** orthogonal to construction soundness. **Most-substantive findings:** (IMPL-A1) switch ML-KEM crate from RustCrypto `ml-kem` to Cryspen `libcrux-ml-kem` with `check-secret-independence` feature — formally verified for secret-independence via hax/F*, mechanically closes L3 Amendment 6 at the crate level; (IMPL-A2) widen nonces to XChaCha20-Poly1305 24-byte for Layer-A + Layer-B to escape ChaCha20-Poly1305's 2^32 birthday bound under DAK reuse; (IMPL-A3+A4) single-source canonical_binding across Rust+TS via NAPI-RS + opaque-handle pattern for secrets crossing FFI; (IMPL-A5+A6) wasm_js getrandom config + Tokio cancel-safety discipline. **One found wire-format conflict:** existing `aead.rs` writes codepoints LE u16 but L3 Amendment 7 mandates BE — IMPL-DECISION-PENDING for Ben (recommend migrate to BE). **Honest cost:** ~35-45 wave-days = ~9-10 calendar-weeks for full §6.2-with-all-8-amendments substrate + golden-vector corpus + dudect CT-CI + kani injectivity proof + per-platform calibration. **Fits 7-15 week v1-beta window comfortably IF impl-amendments bake into R3/R5 briefs upfront.** Recommended minimal load-bearing subset: IMPL-A1+A2+A4+B1.
