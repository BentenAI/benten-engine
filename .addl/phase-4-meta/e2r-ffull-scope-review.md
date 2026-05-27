# F-full Scope + DAK Architectural Review — Full encrypt-everything-at-rest + ephemeral-permission + device-auth + remote-permission-call + multi-device-key-wrap

**Reviewer:** Senior cross-platform applications architect (engaged 2026-05-27; same-shape return as cryptographer + P2P-architect + standards-skeptic before)
**Decision-surface:** Full F-full scope shape + **wave-sequencing across Phase-4-Meta-Core + Phase-4-Meta-Composing** (both PRE-`v1-beta` tag, per CLAUDE.md baked-in #15 corrected 2026-05-27)
**Authority of this document:** ADVISORY. Final disposition rests with Ben.
**Lens:** scope rationality + cross-platform pragmatism + dependency-respecting wave ordering. Construction-soundness, P2P-fit, and standards-maturity are the other three lenses; I touch them only where they bear on the scope-and-sequencing call.

---

## 0. Reading-order note

The TL;DR sits in §1. Most decision-load-bearing sections are §1 (recommendation), §2 (DAK substrate design), §5 (wave-sequencing recommendation; replaces the prior "minimum-viable" framing), §6 (remote-permission-call design — REQUIRED scope per Ben), §9 (identity-recovery design-space flagging), §12 (LOC + timeline), and §14 (extra-reflection-pass).

This review is long because **the F-full scope is genuinely large** (~3,000–7,000 LOC across A+B+C+D), and the dependency map across the four layers + the two sub-phases + the per-platform integration choices is non-trivial. Skim §3 (package landscape table) and §4 (per-platform scope) and go directly to §5 if pressed for time.

---

## 1. Executive recommendation

**RECOMMEND: ship the FULL F-full scope across Phase-4-Meta-Core (wire-format-affecting + crypto substrate) + Phase-4-Meta-Composing (admin-UI-coupled UX surfaces); both close BEFORE v1-beta tag.** I find NO piece of F-full that genuinely requires deferring past v1-beta tag. The do-it-now bias holds.

**Wave-sequencing summary (full table in §5; everything pre-v1-beta-tag):**

- **Phase-4-Meta-Core** (wire-format-affecting + cryptographic substrate; ~5,000–6,000 LOC total):
  - **X-Wing-mislabel corrective** (~24 LOC). Pre-tag-must-fix; INDEPENDENT of F-full. Lands first.
  - **Layer-A: real K_principal store** — pull G-CORE-3e forward. Mints `K_principal` storage type + at-rest envelope. ~600–900 LOC. **Wire-format-affecting** (the storage envelope codepoint enters `EncryptionCodepoint` enum).
  - **Layer-B: per-Node AEAD corrective + rename** (~50 LOC residual after Layer-A lands; the existing combiner reaches into K_principal which is now a real type).
  - **Layer-C: encrypt-to-recipient** — HPKE-RFC-9180 + MLKEM768-X25519 + multi-stanza for groups + Inv-16 mint (~1,800–2,400 LOC). **Wire-format-affecting** (new codepoints; new envelope shape).
  - **Layer-D substrate** — DAK trait + Argon2id-derived DEK + at-rest encryption of K_principal + user-DID private key (~600–900 LOC). **Wire-format-affecting** (on-disk DAK-wrapped-key format goes through interface freeze).
  - **Layer-D wire** — multi-device key-wrap-on-device-link envelope + remote-permission-call wire protocol (~700–1,000 LOC). **Wire-format-affecting** (device-link provisioning message format + permission-call request/response envelopes go through interface freeze).
  - **Layer-D platform glue (minimum)** — desktop full-peer DAK-unlock path through `keyring-core` (or fallback to file-vault) + Tauri 2.x integration smoke ~300–500 LOC. The MINIMUM-glue here is wire-format-affecting because the on-disk DAK-vault format is part of the interface; the deeper per-platform polish (biometric / Stronghold / per-platform-fallbacks) defers to Phase-4-Meta-Composing.

- **Phase-4-Meta-Composing** (admin-UI / UX / polish; ~1,500–2,500 LOC total):
  - **Biometric layer** (Tauri-plugin-biometric integration + macOS Touch ID + Windows Hello + iOS/Android) — additive; doesn't change wire format. ~400–600 LOC.
  - **Device-link UX flow** (QR-code scan + approval dialog + the admin-UI surface for "Add a device") — needs admin-UI infrastructure. ~400–600 LOC.
  - **Remote-permission-call UX flow** (approval-dialog UX on approving device + push-notification or polling UX on requesting device) — needs admin-UI infrastructure. ~300–500 LOC.
  - **Stronghold integration as optional credential backend** for users who want IOTA Stronghold's secure-runtime memory protection (additive). ~200–300 LOC.
  - **Identity-recovery scaffolding** — flag only at this layer; full protocol design is the v1-assessment-window's own decision row. ~50–150 LOC for the stub `RecoveryHook` trait that the eventual protocol plugs into without wire-format break.

**Three load-bearing reasons for shipping full F-full across both sub-phases (vs. cutting):**

1. **Wire-format-must-land-before-freeze argument cuts against deferring ANYTHING that touches on-disk or on-wire bytes past the v1 freeze**. Per CLAUDE.md baked-in #15: Phase-4-Meta-Core "terminates by freezing the v1 public interface." DAK-wrapped on-disk envelopes, the device-link provisioning-message format, and the remote-permission-call envelope are all interface-frozen artifacts. Adding them post-v1-beta is either a wire-break (impossible per #5 crypto-agility framework) or a forever-additive-codepoint shape that's harder to retrofit cleanly than designing-in from the start. **Forward-correctness alone demands these land in Phase-4-Meta-Core.**

2. **Remote-permission-call-from-another-device is REQUIRED scope (Ben 2026-05-27 emphatic).** That requirement cascades into multi-device-key-wrap (since the same envelope shape transports K_principal at device-link time and ephemeral permissions at runtime). Multi-device-key-wrap then cascades into the at-rest DAK substrate (since the K_principal being wrapped IS the at-rest-protected key). The full Layer-D substrate is a closed dependency cluster; cutting any piece breaks the chain.

3. **The cross-platform package landscape is mature ENOUGH for Phase-4-Meta-Core to land the substrate** (Argon2id from RustCrypto: production-grade; HPKE-RFC-9180 for the key-wrap envelope: same primitive as Layer-C, reused; `keyring-core` for OS-keychain integration: 1.0 released; constant-time discipline via `zeroize` + `secrecy`: vendored and audited). The MISSING pieces are not blockers for Phase-4-Meta-Core substrate; they're UX/polish (biometric prompts, per-platform credential-store quirks) that genuinely fit Phase-4-Meta-Composing.

**Confidence:** HIGH on the wave-sequencing across the two sub-phases. MEDIUM-HIGH on the LOC estimates (the Layer-D substrate carries the most uncertainty because cross-platform glue surfaces vary). MEDIUM on the remote-permission-call protocol design — I have a Signal-Provisioning-derived skeleton (see §6) but it warrants a dedicated security mini-review before landing. LOW-MEDIUM on the identity-recovery flag (I deliberately don't design it; just name what F-full locks-in that constrains the eventual design).

**Most-load-bearing single finding:** **the "minimum-viable" framing actively misleads the orchestrator's decision-making.** Asking "what's the minimum to ship at v1-beta?" produces a different answer than asking "what wire-format pieces MUST land BEFORE the interface freeze given that Phase-4-Meta-Composing is also pre-tag?" The latter is the correct framing per Ben's 2026-05-27 phase-ordering correction; the former defaults to cutting valuable wire-format-affecting work into post-tag waves where it CAN'T cleanly land. **Reject "minimum-viable" framing for F-full and use "wire-format-affecting → Phase-4-Meta-Core; UX-polish → Phase-4-Meta-Composing; both pre-tag."**

---

## 2. DAK substrate design (full F-full Layer-D)

The DAK (Device Authentication Key) substrate is the crypto-agile foundation for everything Layer-D-onward. Design follows.

### 2.1 Architectural shape

```
                  ┌──────────────────────────────────────────┐
                  │  User-supplied credential at unlock time │
                  │  (password / biometric / hardware-key)   │
                  └────────────────────┬─────────────────────┘
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            │                                                     │
   [Password path: Argon2id]              [Biometric path: OS-keystore-wrapped]
   - Argon2id(password, salt, params)     - Native LocalAuthentication / 
   - 32-byte DAK_seed                        Windows Hello / Android Keystore
            │                                                     │
            └──────────────────────────┬──────────────────────────┘
                                       │
                                       ▼
                          ┌─────────────────────────┐
                          │  DAK  (32-byte AEAD key) │
                          │  derived via HKDF-SHA256 │
                          │  with info="benten-dak-v1"│
                          └────────────┬────────────┘
                                       │
                                       ▼
                          ┌─────────────────────────┐
                          │  ChaCha20-Poly1305 AEAD │
                          │  over on-disk Vault     │
                          └────────────┬────────────┘
                                       │
                                       ▼
              ┌────────────────────────┴────────────────────────┐
              │                                                 │
   [Vault payload: K_principal]                  [Vault payload: user-DID privkey]
   - 32 bytes                                    - Ed25519 or hybrid sigkey privkey
   - Used to derive all per-Node K(N)             - Used to sign UCAN delegations
              │                                                 │
              └──────────────────┬──────────────────────────────┘
                                 │
                  Held only in process memory after unlock
                  (zeroize on drop; never written back to disk plaintext)
```

### 2.2 Cryptographic choices (each cite-anchored)

- **KDF: Argon2id v0x13** ([RFC 9106](https://datatracker.ietf.org/doc/html/rfc9106)) with default parameters from the RustCrypto `argon2` crate (`m_cost=19456 KiB`, `t_cost=2`, `p_cost=1`) — matches OWASP recommendation per the RustCrypto password-hashing book ([rustcrypto.org/key-derivation/hashing-password.html](https://rustcrypto.org/key-derivation/hashing-password.html)). User-tunable via the manifest at vault-creation time; tuning persisted alongside the salt.
- **Salt:** 16 bytes from `rand_core::OsRng`. Stored unencrypted alongside the vault.
- **Intermediate KDF: HKDF-SHA256** to derive the actual DAK from the 32-byte Argon2id seed plus the codepoint label `"benten-dak-v1"`. This provides a clean cipher-suite-codepoint binding point (when a future Argon2id-v2-parameter-set lands, the HKDF info-tag is the codepoint slot).
- **Vault AEAD: ChaCha20-Poly1305** ([RFC 8439](https://datatracker.ietf.org/doc/html/rfc8439)) — already in Benten's crypto-suite. 12-byte nonce from OsRng; bound by the Vault header's codepoint + version-tag as AAD.
- **Vault payload format: CBOR** (consistent with Benten's CIDv1 / DAG-CBOR canonical-bytes contract per CLAUDE.md baked-in #5). Fields: `version: u8`, `codepoint: u16`, `salt: bytes`, `argon_params: {m_cost, t_cost, p_cost}`, `nonce: bytes`, `aead_ciphertext: bytes`. Outer envelope is itself a DAG-CBOR codepoint slot.
- **Memory hygiene:** `secrecy::SecretBox<[u8; 32]>` for DAK + K_principal + user-DID privkey ([docs.rs/secrecy](https://docs.rs/secrecy/)). `zeroize::Zeroize` on every Drop ([docs.rs/zeroize](https://docs.rs/zeroize)). The RustCrypto book recommends this stack as the standard pattern.
- **Constant-time discipline:** vault unlock failure on wrong-password MUST run Argon2id + AEAD-decrypt to completion (HKDF-deriving even when password is wrong) so wrong-password timing matches right-password timing. The RustCrypto `argon2` crate's `PasswordVerifier::verify_password` provides constant-time semantics ([docs.rs/argon2](https://docs.rs/argon2)); the AEAD-decrypt-failure path returns `UnsupportedAlgorithm`-style error AFTER fully running the decrypt, NOT short-circuiting on tag-mismatch with a faster-than-success codepath.

### 2.3 Trait surface (Rust sketch)

```rust
// In benten-crypto-suite::dak
pub trait DeviceAuthBackend: Send + Sync {
    /// Returns the 32-byte DAK seed for vault decrypt.
    fn unlock(&self, prompt: &UnlockPrompt) -> Result<SecretBox<[u8; 32]>, UnlockError>;
    
    /// Lock the engine; zero in-memory K_principal + user-DID privkey.
    fn lock(&self);
    
    fn supports_biometric(&self) -> bool;
    fn supports_remote_unlock(&self) -> bool;
}

// In benten-engine
pub struct Engine {
    // ... existing
    vault: Vault,                         // CBOR envelope on disk
    unlocked: Option<UnlockedKeyMaterial>, // None until DAK auth completes
}

pub struct UnlockedKeyMaterial {
    k_principal: SecretBox<[u8; 32]>,
    user_did_signing_key: SecretBox<HybridSigningKey>,
}
```

Sealed per CLAUDE.md baked-in #7 sealed-discipline refinement; not a public extension contract at v1-beta.

### 2.4 What happens on Engine startup

1. Read vault from disk.
2. Surface `DeviceAuthBackend::unlock(prompt)` to the embedding host (Tauri shell calls password-prompt UI; headless full-peer calls a stdin/IPC password channel; thin-client edge-runtime does NOT have its own vault).
3. Run Argon2id over the user-supplied password to get DAK seed.
4. HKDF-SHA256 over the seed to derive DAK.
5. ChaCha20-Poly1305-decrypt the vault payload.
6. Hydrate K_principal + user-DID-signing-key into `UnlockedKeyMaterial`.
7. From this point forward, all per-Node AEAD ops + all UCAN signing ops are gated through `UnlockedKeyMaterial`. Engine operations attempted before unlock return `EngineLocked` error (typed-reject, not silent failure).

### 2.5 What this design ENABLES that matters for F-full

- **At-rest theft protection** (Ben's "if a device/engine is stolen/compromised its all encrypted"): redb (storage) is plaintext, but every Node body is encrypted via Layer-B AEAD using K(N) derived from K_principal, AND K_principal itself is encrypted-at-rest in the vault under DAK. Steal the disk → get encrypted Node bodies + encrypted vault → can't decrypt either without the password.
- **Remote-permission-call surface** (§6): the same K_principal + the user-DID signing key are what the remote-unlock + remote-permission-grant flows operate on; once unlocked on device A, device A can sign permission grants for device B's pending operations OR encrypt ephemeral key material to device B's pubkey.
- **Multi-device key-wrap-on-device-link surface** (§7): K_principal transferred from A to B via HPKE-encap to B's device-pubkey, signed by user-DID, with B storing under B's own DAK.

### 2.6 What this design intentionally does NOT do

- **It does NOT provide identity-recovery** (lost password + lost device → unrecoverable). That's a separate protocol designed in the v1-assessment-window (§9).
- **It does NOT eliminate the "user picks a weak password" failure mode**. Argon2id memory-hardness raises brute-force cost ~10⁵–10⁶× vs SHA-256 but doesn't help against a 4-digit PIN. Password-strength UX in admin-UI is the partial mitigation; biometric layer (Phase-4-Meta-Composing) is the better one.
- **It does NOT bind to TPM / Secure Enclave at v1-beta** (the design ADMITS hardware backing as a future `DeviceAuthBackend` impl, but the v1-beta default is software-only). TPM-bound DAK lands in a post-v1-beta wave as an additive backend impl, NOT a wire-format change.

---

## 3. Cross-platform package landscape

Table evaluates each candidate against Benten's actual deployment shapes (per CLAUDE.md baked-in #17): full peer (native Rust on user-owned hardware) / thin compute surface (wasm32-unknown-unknown) / embedded webview (Tauri 2.x today; future Verso).

| Package | Version (May 2026) | Platforms covered | Benten role at v1-beta | Verdict |
|---|---|---|---|---|
| **`argon2` (RustCrypto)** | 0.5.x; [docs.rs/argon2](https://docs.rs/argon2) | Pure Rust; native + wasm32-unknown-unknown | Layer-D password-derived DAK seed | **REQUIRED**. Production-grade. OWASP-recommended Argon2id default params. No CVEs at write-time. |
| **`hkdf` (RustCrypto)** | 0.12.x | Pure Rust everywhere | Layer-D DAK derivation from Argon2id seed (codepoint-tagged via HKDF info) | **REQUIRED**. Already in Benten Cargo.toml transitively. |
| **`chacha20poly1305` (RustCrypto)** | 0.10.x | Pure Rust everywhere | Layer-D vault AEAD + Layer-B per-Node AEAD (already vendored) | **REQUIRED**. Already vendored. |
| **`zeroize` (RustCrypto)** | 1.8.x; [docs.rs/zeroize](https://docs.rs/zeroize) | Pure Rust everywhere | Memory hygiene for K_principal + DAK + private keys | **REQUIRED**. Already vendored transitively. |
| **`secrecy`** | 0.10.x; [docs.rs/secrecy](https://docs.rs/secrecy/0.7.0/secrecy/) | Pure Rust everywhere | Layer-D `SecretBox<[u8; 32]>` wrapper for K_principal + DAK | **REQUIRED**. Tiny dep; idiomatic for Rust crypto stack. |
| **`keyring-core` (open-source-cooperative)** | 1.0.0 (May 2026); [docs.rs/keyring-core](https://docs.rs/keyring-core) | macOS / Windows / Linux (server-grade tier-1). iOS / Android / browser NOT listed as built targets. | Layer-D optional: store DAK-wrapped passphrase OR biometric-bound key in OS credential store; Benten unlock prompts go through this when "remember on this device" is enabled | **RECOMMENDED for Layer-D Phase-4-Meta-Core minimum-glue** (desktop platforms). The recent (2026-05-12) refactor from `keyring` → `keyring-core` is a positive maintenance signal. Note: `keyring` v4.0.1 explicitly says "Do not depend on this crate!" and points at `keyring-core` instead — Benten should depend on `keyring-core` directly, NOT the old `keyring` crate. |
| **`tauri-plugin-stronghold`** | 2.3.1; [docs.rs/tauri-plugin-stronghold](https://docs.rs/crate/tauri-plugin-stronghold/latest) | Tauri-only; works on macOS/Linux/Windows/iOS/Android via Tauri | Layer-D Tauri-shell-only optional backend: IOTA Stronghold vault for secrets-at-rest with Argon2id-derived key | **CONDITIONAL**. Upstream IOTA `stronghold.rs` last release is 2024-05; maintenance signal is mixed. Audited 2022-05 by WITH Secure but no subsequent audit. **For v1-beta: name as a future-additive `DeviceAuthBackend` impl, NOT the default.** The default DAK substrate is the Benten-vended `keyring-core` + Argon2id stack which has no Tauri dependency and works on headless full-peers. Stronghold integration is Phase-4-Meta-Composing scope as an opt-in. |
| **`tauri-plugin-biometric`** (official) | 2.0.0; [v2.tauri.app/plugin/biometric/](https://v2.tauri.app/plugin/biometric/) | Tauri-only; docs emphasize iOS + Android | Layer-D Phase-4-Meta-Composing biometric-prompt path | **REQUIRED for Phase-4-Meta-Composing** (NOT Phase-4-Meta-Core). The official plugin documents iOS + Android; desktop biometric (Touch ID / Windows Hello) is via the community alternative. |
| **`tauri-plugin-biometry`** (Choochmeque, community) | 0.2.x; [github.com/Choochmeque/tauri-plugin-biometry](https://github.com/Choochmeque/tauri-plugin-biometry) | Android + iOS + macOS + Windows (no Linux) | Layer-D Phase-4-Meta-Composing biometric path on desktop | **CONDITIONAL** for Phase-4-Meta-Composing. Community-maintained; 0.x; documents AES-256 + Windows-Hello-protected-keys but doesn't disclose KDF or hardware-backing details. Pre-merge security mini-review required if adopted as default. |
| **`hpke` (Brendan McMillion)** | (depends on revision; verify on crates.io at impl-time) | Pure Rust everywhere | Layer-C encrypt-to-recipient envelope (single-recipient mode_base); Layer-D multi-device-key-wrap envelope (same shape reused) | **REQUIRED**. The McMillion crate is the canonical Rust HPKE impl. **Do NOT use `hpke-rs`** (Cryspen): the 2026-02 [Verification Theatre](https://symbolic.software/blog/2026-02-05-cryspen/) audit found 13 vulnerabilities including the nonce-reuse AES-GCM-recovery class; multiple RustSec advisories filed 2026-03 in the [RustSec database](https://rustsec.org/advisories/). |
| **`ml-kem`, `x25519-dalek`, `sha3`** (RustCrypto) | (current versions; pin per §3.3 discipline from cryptographer review) | Pure Rust everywhere | Layer-C + Layer-D KEM primitives | **REQUIRED**. Subscribe to RustSec for `ml-kem` (CVE-prone class per cryptographer review §3.3). |
| **`webauthn-rs`** | 0.5.x | Pure Rust; server-side authenticator+RP | Layer-D Phase-4-Meta-Composing: hybrid-transport (CTAP 2.2 caBLE) for cross-device unlock UX | **OPTIONAL** for v1-beta-tag. The CTAP 2.2 hybrid-transport pattern ([passkeys.dev](https://passkeys.dev/docs/reference/specs/); [corbado.com/glossary/cda](https://www.corbado.com/glossary/cda)) is the well-trodden wire shape for the remote-permission-call UX. Benten can implement the protocol shape WITHOUT using `webauthn-rs` (since Benten devices already have signing keys + iroh transport). |

### 3.1 Why NOT use `keyring` (the old name)

Per the [keyring-rs GitHub](https://github.com/open-source-cooperative/keyring-rs) README: the legacy `keyring` crate is now "just a sample code crate" and the maintainer explicitly says "Do not depend on this crate!" — the production dependency target is `keyring-core` ([github.com/open-source-cooperative/keyring-core](https://github.com/open-source-cooperative/keyring-core)). This is a recent (May 2026) refactor; older orchestrator handoff docs reference `keyring-rs` at the old path.

### 3.2 Why NOT vendor Stronghold as the default

- Last upstream release 2024-05; maintenance signal is weak.
- Single 2022-05 audit by WITH Secure; no follow-up published.
- Tauri-shell-dependency: Stronghold runs as a Tauri plugin, which means headless full-peers (Phase-9+ Benten Runtime instances) can't use it. Per CLAUDE.md baked-in #17, the engine MUST support all three deployment shapes including the headless full peer case.
- Stronghold has substantial features (audit log; multi-account; client identity) that Benten doesn't need; importing them adds attack surface.

**The Benten-vended `argon2` + `chacha20poly1305` + `keyring-core` stack is strictly better for v1-beta-tag** because (a) it's vetted RustCrypto-class primitives, (b) it works in all three deployment shapes, (c) Benten owns the failure modes, (d) the wire format is Benten-canonical-DAG-CBOR rather than IOTA-Stronghold-specific.

Stronghold as an OPTIONAL `DeviceAuthBackend` impl can land in Phase-4-Meta-Composing for Tauri-shell users who explicitly want IOTA-Stronghold's defense-in-depth runtime memory protection. NOT default; opt-in.

### 3.3 Why NOT use `hpke-rs`

Per Nadim Kobeissi's [Verification Theatre review (2026-02-05)](https://symbolic.software/blog/2026-02-05-cryspen/): 13 vulnerabilities found in Cryspen's `libcrux` and `hpke-rs`, including a nonce-reuse bug in `hpke-rs` Context API that enables full AES-GCM plaintext recovery + forgery after 2³² encryptions. Multiple RustSec advisories filed 2026-03-24 including "Nonce Reuse in HPKE Context."

Even though Benten uses ChaCha20-Poly1305 (not AES-GCM by default), the Context-API class-of-bug is concerning. Brendan McMillion's `hpke` crate is the canonical Rust HPKE impl with cleaner audit posture; use it.

---

## 4. Per-platform integration scope

Benten's three deployment shapes (per CLAUDE.md baked-in #17) have different DAK + remote-permission-call surfaces:

### 4.1 Shape (a): Full peer — native Rust on user-owned hardware

**Layer-D substrate work:**
- Vault file at `${BENTEN_DATA_DIR}/vault.cbor` (DAG-CBOR; encrypted via DAK).
- Password unlock via stdin (CLI) OR IPC channel (admin-UI Tauri shell calls into engine).
- `keyring-core` integration for "remember on this device" (DAK or password-encrypted-DAK stored in OS keychain).
- Headless full-peer (Phase-9+ Benten Runtime) MUST support password-via-env-var OR password-via-IPC at startup — no Tauri / no `keyring-core` dependency.

**Per-platform notes:**
- **macOS** (Phase-4-Meta-Core full peer): `keyring-core` wraps macOS Keychain Services. Code-signed app required for Keychain access; full peer in dev mode reads via dev profile. No biometric in Phase-4-Meta-Core; biometric (Touch ID via `LocalAuthentication`) lands Phase-4-Meta-Composing via `tauri-plugin-biometry`.
- **Linux** (Phase-4-Meta-Core): `keyring-core` wraps libsecret / GNOME Keyring / KDE KWallet. **HEADLESS LINUX has no Secret Service by default** — this is the well-known landmine. Mitigation: fall back to file-vault when `keyring-core` raises `NoBackend` (the user's vault stays password-protected, just can't be "remembered" on this device). Code that handles this fallback is ~50 LOC.
- **Windows** (Phase-4-Meta-Core): `keyring-core` wraps Windows Credential Manager. Works well; no major caveats.
- **iOS** (Phase-4-Meta-Composing — does NOT ship at Phase-4-Meta-Core): per `keyring-core` docs, iOS Protected Data store. Tauri-plugin-biometric Touch ID / Face ID prompt. Background-process re-unlock via Tauri lifecycle hooks.
- **Android** (Phase-4-Meta-Composing): Android Keystore + Shared Preferences via `keyring-core`. Biometric via Tauri plugin. Background-process re-unlock via Activity lifecycle.

### 4.2 Shape (b): Thin compute surface — wasm32-unknown-unknown

**Layer-D substrate work:**
- **No vault on the thin client.** The thin client connects to a full peer; the full peer holds K_principal + user-DID privkey. Thin client establishes a session with the full peer (via `Engine::thin_client_session` from G24-F per Phase-4-Foundation R5).
- Per-tab unlock is at the FULL PEER side; thin client just shows the password prompt and forwards the password to the full peer (over the authenticated session channel, NOT plaintext over fetch).
- Browser fingerprint device-identity: the thin client generates a Web Crypto API keypair per tab (or per browser-profile); the full peer issues a thin-client-session UCAN scoped to that device-pubkey. Multi-tab is multi-thin-client-session.

**Per-platform notes:**
- **Modern browsers** (Chrome / Safari / Firefox): Web Crypto API for the thin-client device keypair; SubtleCrypto for transport-channel encryption. NO biometric on the thin client (WebAuthn could plug in here but it's Phase-4-Meta-Composing UX work).
- **Edge runtimes** (Cloudflare Workers / Deno Deploy — exploratory per `docs/future/benten-runtime.md`): Web Crypto API equivalent; thin-client-session always authenticates back to a full peer.

### 4.3 Shape (c): Embedded webview — Tauri 2.x (today)

**Layer-D substrate work:**
- The Tauri shell IS a full peer (shape a internally); the webview IS a thin compute surface (shape b internally).
- Embedded IPC channel between webview and Tauri shell carries the password-prompt-and-response.
- Tauri's `tauri-plugin-biometric` (Phase-4-Meta-Composing) can biometric-unlock the engine through the IPC boundary.
- The webview NEVER holds K_principal or user-DID privkey; only the Tauri-shell-full-peer does.

**Per-platform notes (Tauri-supported targets):**
- **macOS / Linux / Windows** desktop: as shape (a) for the Tauri shell + standard Tauri webview-to-shell IPC for the UX layer.
- **iOS / Android** (when Tauri mobile lands stably): biometric-prompt path most natural; `tauri-plugin-biometric` is the canonical surface.
- **Future Verso (tauri-runtime-verso)**: same shape; webview-implementation-detail doesn't change the Layer-D surface.

### 4.4 The "headless server" case

CLAUDE.md baked-in #11 + Phase-3 G16-B-E full peer commitments imply Benten will run on headless servers (long-lived Atrium peers, future Benten Runtime instances). Headless servers cannot prompt for a password interactively.

**Recommendation for headless full peers:**
- Password unlock via `BENTEN_VAULT_PASSWORD` env var at startup (zeroize immediately after deriving DAK).
- Password unlock via IPC from a paired admin device (the remote-permission-call protocol §6 covers this; the "remote permission" includes "remote unlock of a locked engine").
- Auto-lock policy: server can be configured to stay-unlocked-while-process-up (the operator-runs-it scenario) OR to require periodic re-unlock (the high-security scenario).

These are interface-frozen at Phase-4-Meta-Core (the env-var name + the IPC channel format) but the UX surfacing them is fine to refine in Phase-4-Meta-Composing.

---

## 5. Wave-sequencing recommendation (replaces "minimum-viable scope")

Per Ben's 2026-05-27 do-it-now bias + phase-ordering correction: ALL F-full pieces land pre-v1-beta-tag. The wave-sequencing question is sub-phase placement, not cut-vs-ship.

**The decision rule:** wire-format-affecting → Phase-4-Meta-Core (before interface freeze); admin-UI-coupled-UX → Phase-4-Meta-Composing (after core substrate freezes); both pre-tag.

### 5.1 Per-piece sub-phase recommendation

| F-full piece | LOC est. | Sub-phase | Wire-format-affecting? | Dependency rationale |
|---|---|---|---|---|
| **X-Wing-mislabel corrective** | ~24 | Phase-4-Meta-Core (lands FIRST in the wave-set; independent of F-full) | Yes (codepoint `0x647a` semantics) | Pre-tag-must-fix REGARDLESS of F-full decision. Cryptographer review §1. |
| **Layer-A: real K_principal store (G-CORE-3e pulled forward)** | ~600–900 | Phase-4-Meta-Core | Yes (storage envelope format + DAK-wrapped on-disk shape) | Layer-B and Layer-C both depend on a real K_principal; pull forward. |
| **Layer-B: per-Node AEAD residual (post-K_principal)** | ~50 | Phase-4-Meta-Core | No (existing combiner just plugs into real K_principal) | Falls out from Layer-A. |
| **Layer-C: encrypt-to-recipient HPKE+MLKEM768-X25519** | ~1,800–2,400 | Phase-4-Meta-Core | Yes (new codepoints, new envelope) | Per Combined Option F from the 3 prior reviews. |
| **Layer-C: multi-stanza HPKE for groups** | ~300–500 | Phase-4-Meta-Core | Yes (group-envelope format) | Per P2P-architect review §1. |
| **Layer-C: Inv-16 mint + SECURITY-POSTURE entry** | ~50–100 | Phase-4-Meta-Core | No (doc) | Per all 3 reviewers. |
| **Layer-D: DAK trait + Argon2id substrate** | ~300–500 | Phase-4-Meta-Core | Yes (on-disk vault format) | Foundation for at-rest-encryption-of-K_principal + user-DID-privkey. |
| **Layer-D: at-rest encryption of K_principal + user-DID-privkey** | ~200–300 | Phase-4-Meta-Core | Yes (vault payload schema) | Per Ben's "if a device is stolen its all encrypted" requirement. |
| **Layer-D: multi-device key-wrap-on-device-link envelope** | ~400–600 | Phase-4-Meta-Core | Yes (device-link provisioning message format) | Per Ben emphatic; uses Layer-C HPKE primitive. |
| **Layer-D: remote-permission-call wire protocol** | ~400–600 | Phase-4-Meta-Core | Yes (permission-call request/response envelopes) | Per Ben "REQUIRED, not cuttable." §6 details. |
| **Layer-D: `keyring-core` integration on desktop platforms** | ~150–250 | Phase-4-Meta-Core | No (just an optional credential-cache layer) | Minimum-glue for "remember on this device" UX. |
| **Layer-D: file-vault fallback when no keychain** | ~50 | Phase-4-Meta-Core | No | Required for headless Linux servers + Tauri-Linux-on-Wayland-headless-Wayfire. |
| **Layer-D: Tauri-shell integration smoke** | ~150–250 | Phase-4-Meta-Core | No | Minimum IPC for password-prompt-from-webview-to-shell-full-peer. |
| **Layer-D: biometric layer (`tauri-plugin-biometric` / `tauri-plugin-biometry`)** | ~400–600 | **Phase-4-Meta-Composing** | No (additive on top of DAK) | Needs admin-UI infrastructure for biometric-fallback-to-password UX flows. |
| **Layer-D: Stronghold as optional backend** | ~200–300 | **Phase-4-Meta-Composing** | No (additive `DeviceAuthBackend` impl) | Optional; Tauri-shell-only; not default. |
| **Layer-D: device-link UX flow (QR scan + approval dialog)** | ~400–600 | **Phase-4-Meta-Composing** | No (uses Phase-4-Meta-Core wire format unchanged) | Needs admin-UI infrastructure (Tauri shell + plugin UI). |
| **Layer-D: remote-permission-call UX flow (approval dialog + notification)** | ~300–500 | **Phase-4-Meta-Composing** | No (uses Phase-4-Meta-Core wire format unchanged) | Needs admin-UI infrastructure. |
| **Layer-D: identity-recovery `RecoveryHook` stub trait** | ~50–150 | **Phase-4-Meta-Composing** | The trait shape is interface-affecting (define carefully) but the IMPL plugs in post-v1-beta | Identity-recovery protocol design is the v1-assessment-window's decision. |

**Total Phase-4-Meta-Core LOC: ~5,000–6,500 (with the X-Wing corrective inside).**
**Total Phase-4-Meta-Composing LOC: ~1,300–2,400.**
**Combined: ~6,300–8,900 LOC.**

### 5.2 The dependency DAG (which order to dispatch)

```
[X-Wing corrective] ─── independent; can land in parallel with anything
                                                       
[Layer-A K_principal store]
       │
       ├──► [Layer-B residual]
       │
       └──► [Layer-C HPKE+MLKEM768-X25519]
                │
                ├──► [Layer-C multi-stanza]
                ├──► [Layer-C Inv-16 mint]
                │
                └──► [Layer-D DAK trait]
                            │
                            ├──► [Layer-D at-rest K_principal encrypt]
                            │
                            ├──► [Layer-D multi-device key-wrap on device-link]
                            │           │
                            │           └─► (uses Layer-C HPKE primitive)
                            │
                            ├──► [Layer-D remote-permission-call wire]
                            │           │
                            │           └─► (uses Layer-C HPKE primitive)
                            │
                            ├──► [Layer-D keyring-core integration]
                            │
                            └──► [Layer-D Tauri-shell IPC smoke]

──── Phase-4-Meta-Core close + tag phase-4-meta-core-close ────

[Layer-D biometric] (parallel; additive; needs admin-UI infra)
[Layer-D Stronghold backend] (parallel; additive)
[Layer-D device-link UX] (needs admin-UI infra)
[Layer-D remote-permission UX] (needs admin-UI infra)
[Layer-D RecoveryHook stub trait] (admin-UI-coupled; lock the trait shape carefully)

──── Phase-4-Meta-Composing close + tag phase-4-meta-close ────

(independent crypto audit window per NF-2 / C-GM-AUDIT)

──── tag v1-beta ────
```

### 5.3 Canary-first discipline per feedback_canary_first_parallel_implementation

The natural canaries (the wave whose API surface unblocks all dependents):

- **Phase-4-Meta-Core canary 1:** Layer-A K_principal store. Mints the `K_principal` type + the vault on-disk format. Everything Layer-D-onward depends on this.
- **Phase-4-Meta-Core canary 2:** Layer-C HPKE-mode-base[MLKEM768-X25519]. Mints the `EncryptToRecipientCodepoint` enum + the `HpkeContext` type. Layer-D wire-format pieces (multi-device key-wrap + remote-permission-call) both use the HPKE primitive.

These two canaries can run in parallel since they touch different types. After both land, fan out Layer-D substrate + Layer-D wire-format pieces in parallel waves (subject to the parallelism cap of 7 implementer agents per CLAUDE.md §13).

---

## 6. Remote-permission-call-from-another-device design (REQUIRED scope)

Ben 2026-05-27: *"to grant engines/agents/peers / rented decentralized compute from our network temporary permissions to decrypt the nodes we store on them, execute a workflow, etc... ephemerally permissioned as necessary."*

This is **Required scope, not cuttable**. Design follows.

### 6.1 Two distinct flows the umbrella term covers

The "remote permission call" actually covers TWO scenarios:

**Flow R1: Remote unlock** — Device B is locked (DAK-not-unlocked-yet because nobody has typed the password). Device A (which IS unlocked) authorizes B's unlock without B's local password entry.

**Flow R2: Remote permission grant** — Device B is unlocked locally BUT wants to do something its DAK-unlocked K_principal doesn't authorize alone (e.g., decrypt a Node owned by user-DID with `audience=alice-did`; or run a workflow whose UCAN chain requires user-DID's signature). Device A issues an ephemeral UCAN delegation to device B with the required attenuation.

Both flows are part of F-full; both lock interface format at Phase-4-Meta-Core. Treat them as ONE wire protocol with two operation variants.

### 6.2 Threat model

The "approving device" (device A) is trusted (it's another device the same user owns). The "requesting device" (device B) MAY be trusted (the user's own device) OR untrusted (an Atrium peer that wants temporary decrypt-permission to run a workflow on the user's behalf in Phase 6+ AI-agent scenarios, OR a rented decentralized compute peer in Phase 7+).

Wire path between A and B is iroh (the existing native transport per CLAUDE.md baked-in #17). iroh provides connection-level E2EE between connected endpoints; the permission-call envelope adds application-layer authentication + authorization on top.

### 6.3 Cryptographic primitives (all already in F-full scope)

- **Transport:** iroh QUIC channel between the two endpoints. Identifies endpoints by their ed25519 device-pubkey (per the device-attestation envelope from Phase-3 G16-D wave-6b).
- **Authentication:** user-DID signature on the permission grant (signed by the unlocked user-DID-signing-key on device A).
- **Confidentiality:** the permission-grant payload (which may contain ephemeral wrapped key material) is HPKE-encrypted-to-device-B-pubkey using Layer-C's HPKE-mode-base[MLKEM768-X25519] envelope.
- **Replay defense:** nonce in the request + a `valid_until` timestamp from device A's clock; device B verifies clock-skew within tolerance.
- **Auditability:** the permission-grant is signed + content-addressed; lands as a Node in user's local graph with `kind=PermissionGrant` (so the user has a queryable audit log of "who I gave what permission to, when").

### 6.4 Wire protocol shape

```
1. B → A: PermissionRequest {
     request_id: uuid,
     requesting_device_did: did,
     requesting_device_pubkey: bytes,
     operation: { 
       Decrypt(node_cid) | 
       SignUcanDelegation(scope, audience, expires_at) | 
       RemoteUnlock 
     },
     reason: optional<string>,  // user-readable rationale
     timestamp: u64,
     ephemeral_signing_key_for_session: pubkey,
   }
   (signed by B's device-key)

2. A: validates B's device-key against A's user's known-device set;
      displays approval-dialog UX with the operation summary;
      user types password (if not already authenticated this session) 
      OR confirms via biometric;
      device A's UnlockedKeyMaterial.user_did_signing_key signs the grant.

3. A → B: PermissionGrant {
     request_id: uuid,
     granted_at: u64,
     valid_until: u64,
     operation_result: {
       Decrypt: HPKE_to_B(node_decryption_key) |   // ephemeral key material
       Ucan: signed_ucan_token |
       Unlock: HPKE_to_B(K_principal)              // remote-unlock special case
     },
     audit_node_cid: cid,  // pointer to the PermissionGrant Node device A wrote
   }
   (signed by A's user-DID-signing-key)

4. B: validates A's signature against the user-DID-pubkey;
      executes the granted operation;
      writes a local "permission-grant-received" audit Node.
```

### 6.5 UX shape (the part that lands in Phase-4-Meta-Composing)

- Device A receives a push-notification-equivalent (iroh-gossip event surfaced through Tauri to a system tray icon / Android notification / iOS notification).
- Device A's admin-UI surfaces an approval dialog with the operation details + reason + the requesting-device fingerprint.
- User confirms via biometric (preferred) or password.
- Approval-decision is signed + returned.
- Audit-log Node is written to user's graph.

**Why this is Phase-4-Meta-Composing-only for the UX:** the protocol wire format MUST be in Phase-4-Meta-Core (interface freeze). The notification + dialog + audit-log UI surfaces depend on admin-UI infrastructure which is Phase-4-Meta-Composing scope.

### 6.6 Hybrid-transport / CTAP-2.2-caBLE alignment opportunity

The wire shape above is structurally identical to CTAP 2.2 caBLE hybrid-transport (passkeys.dev). Benten doesn't need to be CTAP-compatible at v1-beta (we're not authenticating to a WebAuthn RP), but the SHAPE of the protocol — QR-code-on-device-A + Bluetooth-or-iroh-proximity-check + device-A-issues-signed-grant — is the well-trodden industry pattern. The Apple TV "scan QR with iPhone to authenticate" experience ([engineering.fb.com](https://engineering.fb.com/2026/02/04/security/cross-device-passkey-authentication-for-xr-devices-meta-quest/)) uses the same pattern.

This alignment doesn't constrain our wire format (we use Benten's own DAG-CBOR codepoint envelope, not CTAP); it just confirms the SHAPE we're picking is the industry Schelling point for cross-device authentication.

### 6.7 Pre-merge security mini-review required

This wire protocol carries some of the highest-value primitives in F-full (signed remote unlock granting K_principal access; signed remote UCAN delegations granting user-impersonation-equivalent capability). A dedicated security mini-review BEFORE landing — focused specifically on:
- Replay defense (nonce + timestamp)
- Device-key revocation interaction (if device B is later revoked, are in-flight grants still honored?)
- Clock-skew attack window
- Confused-deputy attacks (B requests permission on behalf of C without C's knowledge)
- UI-deception attacks (B sends misleading `reason` field; A's user approves under false pretenses)

— is non-negotiable. Add as a named pre-merge gate in the wave brief.

---

## 7. Multi-device key-wrap-on-device-link design

Per Ben supplementary 2026-05-27: *"when a user adds a 2nd device, how does K_principal transfer from device A to device B?"*

### 7.1 The construction (drawing on the Signal-Provisioning precedent)

The Signal provisioning protocol ([signal.org/blog/a-synchronized-start-for-linked-devices](https://signal.org/blog/a-synchronized-start-for-linked-devices/), [github.com/AsamK/signal-cli/wiki/Linking-other-devices-(Provisioning)](https://github.com/AsamK/signal-cli/wiki/Linking-other-devices-(Provisioning))) is the canonical pattern; Benten uses the same shape with PQ-hybrid wrap.

```
1. Device B (newly-installing engine):
   - Generates a fresh device-keypair (Ed25519 + hybrid ML-DSA-65 per LAMPS)
   - Generates a fresh device-encryption-pubkey (X25519 + ML-KEM-768 via the HPKE KEM = the same MLKEM768-X25519 hybrid Benten uses for Layer-C)
   - Generates a fresh local DAK by prompting user for a NEW device-local password (or biometric)
   - Encodes (device_signing_pubkey, device_encryption_pubkey, provisioning_session_id) into a QR code
   - Displays QR on screen

2. Device A (already-linked, user is authenticated):
   - User opens "Add a device" flow in admin-UI
   - Scans QR with camera (or pastes the URL form for headless cases)
   - Validates the QR is from a Benten-engine-provisioning flow
   - Displays "Adding device <fingerprint>; proceed?" with user confirmation
   - Constructs a DeviceLinkProvisioningPayload {
       k_principal: SecretBox<[u8; 32]>,   // user's K_principal
       user_did_signing_key: HybridSigningKey,
       user_did_pubkey: HybridPubKey,
       atrium_memberships: list<AtriumMembership>,
       provisioning_session_id: uuid,
       provisioning_timestamp: u64,
     }
   - HPKE-encrypts the payload to device B's encryption-pubkey (using the HPKE-mode-base[MLKEM768-X25519] primitive from Layer-C)
   - Signs the ciphertext + device-B's signing-pubkey with user-DID-signing-key
   - Writes a new DeviceAttestation Node to the user's local graph attesting to device B's signing-pubkey
   - Transmits the signed encrypted payload + the DeviceAttestation Node to device B via iroh (the provisioning_session_id keys the channel)

3. Device B:
   - HPKE-decrypts using its just-generated device-decryption-key
   - Verifies the signature against the user-DID-pubkey embedded in the payload
   - Stores K_principal + user_did_signing_key under its local DAK
   - Mints its own DeviceAttestation chain entry under user-DID and propagates it
   - Engine is now provisioned + locally-DAK-protected on device B
```

### 7.2 Wire format

```rust
// Device-B-side, displayed as QR (CBOR-encoded then base64'd)
pub struct ProvisioningOffer {
    version: u8,
    codepoint: u16,
    provisioning_session_id: [u8; 16],   // random nonce
    device_signing_pubkey: HybridPubKey,
    device_encryption_pubkey: HybridKemPubKey,
    iroh_endpoint_id: EndpointId,         // for transport rendezvous
}

// Sent from device A to device B over iroh
pub struct ProvisioningPayload {
    version: u8,
    codepoint: u16,
    hpke_envelope: HpkeEnvelope,           // encrypts the inner payload to device-B-encryption-pubkey
    signature: HybridSignature,            // by user-DID-signing-key over the HPKE envelope + device-signing-pubkey
    device_attestation_node_cid: Cid,      // the DeviceAttestation Node device A wrote
}

// Inner (HPKE-decrypted) payload
pub struct ProvisioningInnerPayload {
    k_principal: [u8; 32],
    user_did_signing_key: HybridSigningKeySerialized,
    user_did_pubkey: HybridPubKey,
    atrium_memberships: Vec<AtriumMembership>,
    provisioning_session_id: [u8; 16],
    granted_at: u64,
}
```

All codepoints frozen at Phase-4-Meta-Core. The HpkeEnvelope shape is exactly Layer-C's primitive; reuse is the architectural-coherence win.

### 7.3 Security properties

- **Confidentiality:** HPKE-mode-base[MLKEM768-X25519] guarantees IND-CCA2 against any third party who intercepts the ciphertext. Even if the iroh transport layer is compromised, the payload remains encrypted to device B's freshly-generated encryption-pubkey.
- **Authentication:** user-DID-signing-key signature proves device A's intent.
- **Forward secrecy of K_principal:** none (K_principal is long-lived; this is by design — K_principal is identity-equivalent, NOT a session key). HPKE base-mode does not provide FS w.r.t. recipient compromise per [RFC 9180 §9.1.4](https://datatracker.ietf.org/doc/html/rfc9180#section-9.1.4).
- **Forward secrecy at session layer:** the provisioning_session_id + the ephemeral nature of the iroh channel give session-layer FS. The K_principal payload itself is not session-ephemeral.
- **Replay defense:** provisioning_session_id binds the QR to the payload; if a payload is replayed without a matching pending session, device B rejects it.

### 7.4 What happens on multi-device divergence

If user has 3 devices (A, B, C) and revokes B (e.g. B stolen): device A's user-DID-key signs a RotationLog entry (per Phase-3 `benten-id::RotationLog` infrastructure) declaring B's device-key revoked. Device C observes the RotationLog entry through Atrium sync; subsequent permission-call requests from B fail (validated against the revocation list).

Note: B's locally-stored copy of K_principal is NOT recoverable by remote revocation. Per the threat model (CLAUDE.md baked-in #18), once K_principal has been delivered to a device, that device can decrypt all past content owned by user-DID. Revocation cuts future grants + future content shares; it does NOT retroactively un-decrypt content already on the revoked device. This matches the Spike-derived "revocation reach" R6 ratification 2026-05-21.

---

## 8. User-DID private key at-rest protection design

Per Ben supplementary 2026-05-27: *"the DAK substrate should protect BOTH `K_principal` AND the user-DID signing keypair."*

### 8.1 Current state

In Benten today, the user-DID signing key is generated via `rand_core::OsRng` + held in process memory. There is no at-rest encryption beyond OS full-disk encryption (which protects against cold-disk-theft but not against running-process compromise or unencrypted-backup leakage).

### 8.2 Proposed F-full design

Under F-full Layer-D, the user-DID signing keypair lives inside the same DAK-encrypted vault as K_principal. Specifically:

```rust
pub struct VaultPayload {
    k_principal: [u8; 32],
    user_did_signing_key: HybridSigningKeySerialized,   // Ed25519 + ML-DSA-65 hybrid
    user_did_creation_time: u64,
}
```

The entire VaultPayload is CBOR-encoded then ChaCha20-Poly1305-AEAD-encrypted under the DAK. On engine unlock:
1. Argon2id derives the DAK from the user password.
2. AEAD decrypts the vault.
3. Both K_principal AND user_did_signing_key hydrate into `UnlockedKeyMaterial` (each in its own `SecretBox`).
4. From this point forward, any UCAN-signing operation by user-DID OR any per-Node AEAD using K_principal goes through the unlocked-material handle. Engine ops attempted while locked return `EngineLocked` typed error.

### 8.3 Why bundle them in the same vault

- **Atomic lock/unlock:** locking the engine zeroes both keys simultaneously; no risk of K_principal being unlocked while user-DID is locked or vice versa.
- **Identity coherence:** the user-DID signing key + K_principal are the two halves of user-identity at v1-beta. Per CLAUDE.md baked-in #18: "the user-DID is the trust anchor + signs install records; K_principal is the encryption substrate." Same lifecycle.
- **Simpler interface:** one `DeviceAuthBackend::unlock` call hydrates everything the engine needs to operate as the user.

### 8.4 Why NOT keep user-DID-key in OS-keychain separately

A reasonable alternative is: K_principal in vault under DAK, user-DID-key in `keyring-core` OS-keychain under OS-biometric. This has UX advantages (biometric for the small frequent UCAN signing; password only for the rare K_principal-protected ops).

**Recommendation: NOT for Phase-4-Meta-Core default.** Reasons:
- Increased attack surface (two storage locations to defend; both must be compromised to fully compromise the user, but EITHER being compromised already breaks security claims).
- Headless full-peer doesn't have a keychain (file-vault is the only option).
- The "biometric for frequent ops" UX is a Phase-4-Meta-Composing concern; the substrate stays unified.

**Future-additive:** a `DeviceAuthBackend` impl that splits storage (e.g. user-DID-key in TPM, K_principal in vault) can land post-v1-beta as a new backend without wire-format break. The vault format is the same; just the unlock path differs per-backend.

### 8.5 Backup + multi-device implication

The vault file is small (a few hundred bytes). Users SHOULD back it up offline (cloud backup is fine if they trust the password; an offline encrypted backup is fine for paranoid users). The DAK-encryption + the password-derived-strength of Argon2id mean the backup is safe to store on untrusted cloud — same threat model as a backup of a Bitwarden vault ([bitwarden.com/help/bitwarden-security-white-paper/](https://bitwarden.com/help/bitwarden-security-white-paper/)).

Multi-device sync of the vault file itself is NOT recommended — different devices have different DAKs (different local passwords), so the vault contents are device-local even though K_principal + user-DID-key are user-global. The device-link key-wrap (§7) is the correct way to propagate K_principal + user-DID-key to a new device.

---

## 9. Identity-recovery design-space flagging (deliberately NOT designed; just constraints surfaced)

Per CLAUDE.md baked-in #15: identity-recovery protocol choice is a v1-assessment-window decision (Phase-4-Meta-Composing scope). I deliberately don't propose a design; I surface what F-full Layer-A + Layer-D decisions LOCK IN that constrain the eventual identity-recovery design.

### 9.1 What F-full at Phase-4-Meta-Core locks in

- **Per-DID, single-user-DID model.** K_principal is bound to user-DID; recovery must produce both the user-DID signing key AND K_principal.
- **DAG-CBOR + CIDv1 envelope format for the vault** — any social-recovery payload encrypted-to-trustees will share this envelope shape.
- **Multi-device key-wrap exists** (§7) — so recovery designs that produce "one new device that's been linked" are naturally compatible.
- **RotationLog infrastructure exists** (Phase-3) — for declaring "old device-keys revoked, new one trusted." Any recovery design must compose with RotationLog (not bypass it).

### 9.2 What F-full does NOT lock in

- **Whether recovery is social (Shamir-shard among trustees) vs hardware-token (offline encrypted backup) vs threshold-of-devices (k-of-n existing devices reconstruct K_principal) vs combination.** All three remain in the design space.
- **Whether recovery produces the SAME K_principal (recovers identity continuity) or a NEW K_principal (which would require re-encrypting all owned content to the new key, which is impractical for large user content sets) — STRONGLY constrains toward "recover SAME K_principal" but the protocol still has options.**
- **Cross-Atrium recovery semantics** — when a user recovers identity, do their Atrium memberships transparently transfer? RotationLog gives us the primitives but the protocol details remain open.

### 9.3 What I'd recommend for the recovery-stub trait that lands at Phase-4-Meta-Composing

```rust
// In benten-engine
pub trait IdentityRecoveryBackend: Send + Sync {
    /// Initiate a recovery — produce something the user can later restore from.
    fn prepare_recovery_artifact(&self, params: RecoveryParams) -> Result<RecoveryArtifact>;
    
    /// Given a recovery artifact + any required auxiliary inputs (trustee signatures, etc.), 
    /// hydrate Vault-equivalent material on a fresh device.
    fn restore_from_artifact(&self, artifact: RecoveryArtifact, aux: AuxInputs) 
        -> Result<UnlockedKeyMaterial>;
}
```

This trait surface is what F-full Phase-4-Meta-Composing locks in. The Phase-4-Meta-Core substrate (vault + multi-device key-wrap) gives the IMPLEMENTATIONS of this trait a clean foundation; the actual protocol design lands in the v1-assessment-window.

**Critically — designing this trait carefully NOW (Phase-4-Meta-Composing, pre-tag) prevents wire-format-breaks later.** The trait shape is interface-frozen at v1-beta-tag even though the IMPL plugs in over time. Recovery-artifact format SHOULD be a DAG-CBOR codepoint slot so future protocols slot in additively.

### 9.4 The "social recovery via Atrium-trustees" affinity

Benten's Atrium primitive is structurally well-suited to social recovery: trustees are simply Atrium members who've been issued a UCAN scoped to "hold a recovery shard." Shamir-shard distribution becomes "issue N UCANs to N trustees, each carrying a Shamir share encrypted to the trustee's pubkey." Recovery becomes "trustees re-encrypt their shares to the recovering user's new device-pubkey via the encrypt-to-recipient primitive."

This is an ELEGANT shape that F-full Phase-4-Meta-Core enables but doesn't lock in. Phase-4-Meta-Composing can land the social-recovery protocol as ONE concrete `IdentityRecoveryBackend` impl; alternative impls (hardware-token / threshold-of-devices) plug in as needed.

---

## 10. Implementation hazard surface + audit scope

### 10.1 Substantial hazard surfaces by layer

| Layer | Primary hazard | Mitigation |
|---|---|---|
| Layer-A K_principal store | At-rest plaintext leak if vault format is buggy | DAG-CBOR canonical encoding test fixtures; round-trip property tests; explicit "vault MUST never write plaintext K_principal" code-review checkpoint |
| Layer-B per-Node AEAD | Already shipped; X-Wing-mislabel is the only known hazard | Cryptographer-review §1 corrective; ~24 LOC |
| Layer-C HPKE+MLKEM768-X25519 | Per cryptographer review: ml-kem CVE-class; hpke-rs vulnerability class; nonce-handling | Pin `ml-kem` + monitor RustSec; use Brendan McMillion's `hpke` crate not `hpke-rs`; KAT cross-verify against X-Wing reference C impl |
| Layer-D DAK substrate | Password-derivation timing; vault-AEAD-decrypt timing; weak-password user-error | Constant-time discipline (verify with dudect on the unlock path); password-strength UX in admin-UI; biometric as Phase-4-Meta-Composing additional layer |
| Layer-D multi-device key-wrap | Provisioning-session replay; rogue-QR-from-attacker | Session-id binding + signed-by-user-DID; QR-scan UX must clearly show device-fingerprint before user confirms |
| Layer-D remote-permission-call | Replay; confused-deputy; UI-deception attacks (B sends misleading reason) | §6.7 pre-merge security mini-review NON-NEGOTIABLE; nonce + valid_until + clock-skew tolerance; audit-log Node for every grant |

### 10.2 External cryptographer audit scope

Per CLAUDE.md baked-in #15 NF-2 / C-GM-AUDIT exit criterion + the cryptographer-review §1 condition-3: external `ml-dsa`/`ml-kem` audit is required pre-v1-GM, NOT pre-v1-beta. For v1-beta, the audit scope I recommend:

- **Layer-A vault format + on-disk envelope** — ~½ person-week
- **Layer-C HPKE wrapper code** (the new code around the McMillion `hpke` crate) — ~1 person-week per cryptographer-review §1
- **Layer-D DAK substrate** (Argon2id parameter choice; HKDF-info-tagging; vault-AEAD-binding) — ~½ person-week
- **Layer-D remote-permission-call protocol** — ~½ person-week (focused on the §6.7 attack categories)
- **Layer-D multi-device key-wrap protocol** — ~½ person-week

**Total v1-beta external audit: ~3 person-weeks.** Audit firms: PQShield, Cure53, NCC Group, or Cryspen (note: Cryspen's recent track record per §3.3 is concerning; prefer one of the other three).

Schedule the audit window concurrent with Phase-4-Meta-Composing (the substrate-frozen tag `phase-4-meta-core-close` is the audit baseline; findings come back during Composing with sufficient time to close any blockers before v1-beta tag).

The PQ-primitives-specific audit (`ml-dsa` + `ml-kem` impls) stays the v1-GM gate; it's independent of the substrate audit above.

---

## 11. Composability with Combined Option F

The Combined Option F (HPKE + MLKEM768-X25519 + multi-stanza + CGKA-deferred + Inv-16 + X-Wing-mislabel-corrective + audit-gate) from the 3 prior reviews composes CLEANLY with F-full Layer-D. Specifically:

- **The HPKE-mode-base[MLKEM768-X25519] primitive from Layer-C is REUSED at Layer-D** (multi-device key-wrap envelope; remote-permission-call payload encryption). One primitive, two layers, architectural-coherence-positive. The P2P-architect's Option-F rationale ("reuse X-Wing across layers") generalizes from "storage substrate + Drop envelope" to "storage substrate + Drop envelope + device-link wrap + remote-permission-grant envelope" — four layers using the same primitive.

- **The Inv-16 3-layer decomposition (identity = canonical-payload-CID; encryption = codepoint-dispatched HPKE; revocation = semantic tuple) extends to F-full.** Vault-CIDs are never load-bearing identifiers (user-DID + device-DID are; vault is just a storage container). Permission-grant Nodes are content-addressed but identified by `(granter_did, grantee_did, grant_scope, granted_at)` semantic tuple, NOT by grant-Node-CID.

- **The Inv-15 sig-bundle-CIDs-never-load-bearing invariant applies to the device-link signed payload + the remote-permission-call signed payload.** Identity of "this is the right user-DID grant" goes through user-DID, not through the signed-payload-CID.

- **The pre-v1-beta-tag external-cryptographer-audit gate from Combined Option F absorbs the F-full Layer-D audit** into one ~3-person-week window (§10.2).

- **The CGKA-deferred decision from Combined Option F is unaffected by F-full Layer-D.** Multi-stanza HPKE remains the v1-beta group-fallback; CGKA reservation persists.

No design-conflicts surfaced. F-full extends Combined Option F naturally.

---

## 12. LOC + timeline estimate

### 12.1 LOC totals (range)

| Tier | LOC range |
|---|---|
| **Phase-4-Meta-Core (wire-format-affecting + substrate)** | **~5,000 – 6,500** |
| **Phase-4-Meta-Composing (UX + polish + additive)** | **~1,300 – 2,400** |
| **Combined** | **~6,300 – 8,900** |

This is large but within precedent: Phase-4-Foundation shipped ~50 new ErrorCodes + 12-crate workspace expansion (PR-volume in the dozens). F-full at ~8K LOC is comparable in scale to one substantial Phase-4-Foundation wave-cluster.

### 12.2 Timeline at AI-agent-dispatch tempo

Phase-4-Meta-Core observed tempo (per the 2026-05-22 substrate-cascade evidence): ~5 PRs ranging from ~150 LOC to ~3,000 LOC landed in ~12 hours of wall-clock when canary-first + scoped-pre-flight discipline applied + the parallelism cap of 7 was utilized. ~1,000 LOC/hour overall throughput.

F-full Phase-4-Meta-Core scope ~5,500 LOC → ~5–7 hours of wall-clock at peak parallelism + ~5–8 mini-review cycles + ~2–3 cargo-clean-idle cycles + a couple of fix-pass waves → **~3–5 days of AI-agent-dispatch wall-clock** to land Phase-4-Meta-Core scope.

F-full Phase-4-Meta-Composing scope ~1,800 LOC → **~1–2 days of AI-agent-dispatch wall-clock**.

**Plus the standard ADDL pipeline overhead** (R1/R2/R3/R4/R5/R6) at the phase-close cycle. Phase-4-Foundation R6 took 8 rounds; expect similar 6–8-round convergence for Phase-4-Meta-Core including the substantial F-full additions. Phase-4-Foundation R6 wall-clock was ~3 days; expect 3–5 days for Phase-4-Meta-Core R6 with the wider F-full surface.

**Plus the external cryptographer audit window:** 3 person-weeks. This is the binding constraint on the v1-beta-tag wall-clock. Schedule the audit START at `phase-4-meta-core-close` tag time; audit completes ~3 wall-clock weeks later. Phase-4-Meta-Composing work happens in parallel with the audit window so the audit findings (if any) close BEFORE v1-beta-tag.

**Total wall-clock estimate from now (2026-05-27) to v1-beta-tag:**
- Phase-4-Meta-Core: ~7–14 days (substrate dispatch + R6 convergence + tag)
- Phase-4-Meta-Composing in parallel with audit: ~21 days (limited by audit window)
- v1-beta-tag: **~4–5 weeks from now**

This is consistent with the "5-week-old project on AI-accelerated tempo" framing.

### 12.3 Variance + risk

- **Cryptographer audit findings could surface design-changes during Phase-4-Meta-Composing** — would push v1-beta-tag back by ~1–2 weeks if a major issue requires substantive redesign. Mitigation: dispatch a thorough cryptographer-review of the design BEFORE the formal audit (per cryptographer review §10 self-assessment-item-2); the formal audit then has lower probability of surprises.
- **Tauri / cross-platform glue surfaces are the most variable.** Linux headless / Tauri-mobile / Verso-runtime are all moving ecosystems. Mitigation: Phase-4-Meta-Core ships ONLY the minimum-glue (desktop full-peer + Tauri-shell smoke); Phase-4-Meta-Composing absorbs the rest. The MUST-LAND-PRE-v1-BETA-TAG bar is "works for desktop full-peer + Tauri-shell on macOS/Linux/Windows"; iOS/Android/Verso/exotic platforms are additive.
- **Remote-permission-call protocol design has the highest single-piece security risk.** Mitigation: §6.7 dedicated security mini-review; conservatively-scope the v1-beta operations (Decrypt + SignUcanDelegation + RemoteUnlock are the three operations); MORE operations can be additive codepoints later.

---

## 13. Risks + mitigations table

| Risk | Severity | Probability | Mitigation | Owner |
|---|---|---|---|---|
| F-full scope creep past phase-4-meta-core-close (interface-freeze) gate | HIGH | MED | Wave-sequencing table (§5.1) is the contract; refuse "just one more thing" requests unless they're genuinely wire-format-affecting AND in the wave-set | Ben + orchestrator |
| Cryptographer audit findings push v1-beta tag back | MED-HIGH | MED | Dispatch thorough cryptographer-review BEFORE formal audit; conservative protocol design; pre-merge security mini-review for remote-permission-call (§6.7) | Project budget + orchestrator |
| Cross-platform glue surfaces (Linux headless / iOS Tauri-mobile / Verso) drift from spec | MED | HIGH | Minimum-glue in Phase-4-Meta-Core; per-platform CI matrix; clear in-Phase-4-Meta-Composing scope-fence | Implementer agents per platform |
| Argon2id parameter choice ages poorly (faster hardware → weaker security) | LOW-MED | HIGH (over 5-year horizon) | Parameters stored in vault header → user can upgrade by re-deriving on next unlock; codepoint-tagged via HKDF info; v2 params are additive | Benten-core; admin-UI surface in v1-assessment-window |
| User picks weak password → Argon2id doesn't save them | MED | HIGH | Password-strength UX in admin-UI; biometric layer (Phase-4-Meta-Composing) as alternate unlock path | Phase-4-Meta-Composing UX |
| `keyring-core` 1.0 has lurking design-issues (recent refactor) | LOW-MED | LOW | Fall back to file-vault when `keyring-core` errors; the integration is additive not load-bearing | Benten-core |
| Stronghold deprecation / IOTA stops maintaining upstream | LOW | MED (2024-05 last release; pattern persistent) | Stronghold is opt-in NOT default; can be removed without wire-format break | Benten-core |
| Remote-permission-call wire format has lurking attack class | HIGH | LOW-MED | Dedicated security mini-review (§6.7); conservative operation set; nonce + valid_until + audit-log | Pre-merge security agent |
| Multi-device key-wrap exfiltrates K_principal to wrong device | HIGH | LOW | Provisioning_session_id binding; user confirmation of device-fingerprint at A; explicit "Adding device X; proceed?" UX | Phase-4-Meta-Composing UX |
| Identity-recovery design forecloses an option Ben wants | MED | LOW-MED | RecoveryHook trait shape locks down carefully at Phase-4-Meta-Composing; the protocol behind it is v1-assessment-window decision | Ben + Phase-4-Meta-Composing |
| F3 JOSE adoption-call deadline 2026-05-29 missed | LOW | HIGH | Per orchestrator handoff "acceptable-to-miss"; post-RFC-adopter-registration is the fallback | (no urgency) |

---

## 14. Extra-reflection-pass output — is there a better permanent shape?

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`: after writing §§1–13, I take an extra pass to look for an elegant unified shape that closes multiple F-full questions at once with strictly less code.

### 14.1 The candidate elegant shape: **"DAK-as-Sealed-DeviceAuthBackend with the HPKE primitive as the universal wrap-key transport"**

The four layers (A K_principal store; B per-Node AEAD; C encrypt-to-recipient; D DAK + multi-device + remote-permission) have a SHARED structural shape if we look at them with the right squint:

- All of A/B/C/D are doing some form of **"protect some key material X under some authorization context Y, such that some authorized party Z can recover X when Y is satisfied."**
  - Layer-A: protect K_principal under DAK (Y = "user knows password") — Z = unlocked engine
  - Layer-B: protect Node payload under K(N) = KDF(K_principal, N.cid) (Y = "have K_principal") — Z = engine-that-knows-K_principal
  - Layer-C: protect payload under recipient pubkey via HPKE (Y = "have recipient's secret key") — Z = recipient engine
  - Layer-D: protect K_principal at-rest under DAK; ALSO protect K_principal under device-B pubkey at device-link via HPKE; ALSO protect ephemeral grant under device-B pubkey at remote-permission-call via HPKE

**The HPKE-mode-base[MLKEM768-X25519] primitive can serve ALL FOUR layers**, with the distinction being WHO the recipient pubkey is:

- Layer-A vault: HPKE(plaintext = K_principal, recipient_pubkey = derived-from-DAK-via-HKDF). The "recipient" is a pseudo-pubkey whose secret key is the DAK. This unifies the vault format with the encrypt-to-recipient format.
- Layer-B per-Node AEAD: stays AEAD with derived K(N) — actually this IS structurally different (symmetric not asymmetric) so HPKE doesn't unify here. Per-Node AEAD remains its own primitive.
- Layer-C encrypt-to-recipient: HPKE-mode-base[MLKEM768-X25519] direct.
- Layer-D device-link: HPKE(K_principal, device-B-pubkey).
- Layer-D remote-permission-grant: HPKE(ephemeral-grant, device-B-pubkey).

**The elegant shape (Option F-full-refined):**

```rust
// Conceptual unified envelope
pub struct EncryptedEnvelope {
    codepoint: u16,                       // distinguishes Layer-A vault / Layer-C drop / Layer-D wrap
    hpke_envelope: HpkeEnvelope,          // ONE primitive for all asymmetric encryption
    aad_binding: BindingContext,          // codepoint + version + per-use-context binding
}

pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient { audience_did: Did },
    DeviceLink { provisioning_session_id: [u8; 16] },
    RemotePermission { request_id: [u8; 16], operation: PermissionOperation },
}
```

This unifies Layer-A vault format + Layer-C encrypt-to-recipient + Layer-D wraps under ONE envelope type with codepoint-discriminated AAD binding contexts.

**Pros:**
- ONE crypto primitive across four use-cases. Smaller audit surface.
- ONE envelope format → easier to test, fewer codepoint inventions.
- Future-additive codepoints slot in trivially (new BindingContext variant).
- Layer-A vault is "encrypted to a DAK-derived pubkey" which means the SAME unlock code path can also handle "vault sealed by device A on behalf of device B" (which is what device-link does anyway).

**Cons:**
- Layer-A vault DAK derivation needs to produce a pubkey/seckey pair (not just a symmetric key). Adds a tiny HKDF step (cheap).
- One layer of indirection between the user's password and the vault decryption (DAK → HKDF → MLKEM768-X25519 pseudo-keypair → HPKE-decap). Tiny overhead.
- The "pseudo-keypair derived from a symmetric secret" is an unconventional pattern; needs cryptographer review (probably sound — HPKE's Encap/Decap interface admits deterministic-derived pubkeys — but warrants verification against the X-Wing draft's deterministic-derivation appendix).

**Verdict:** this is GENUINELY MORE ELEGANT than the per-layer envelope-distinct shape I proposed in §§2–8. **Worth surfacing to Ben as Option-F-full-refined for ratification.** If Ben prefers the per-layer-distinct envelope shape (simpler to reason about per layer), the §§2–8 design is shippable. If Ben prefers the unified shape (smaller audit surface; consistent envelope semantics), Option-F-full-refined is the better permanent shape.

### 14.2 NAMED-deferred unselected elegant alternatives

Per the discipline of recording alternatives I considered but didn't select:

- **"DeviceAuthBackend impl per-device-type"** (TPM-backed / secure-enclave-backed / WebAuthn-backed) — deferred to post-v1-beta as additive backend impls. The DeviceAuthBackend trait shape locks them in at Phase-4-Meta-Core.
- **"Vault as a Benten Node with its own CID"** — considered then rejected: vault should NOT be in the user's graph (it's pre-unlock; the engine can't even read its graph yet without K_principal). Vault stays as a separate on-disk file at `${BENTEN_DATA_DIR}/vault.cbor`.
- **"Per-Atrium DAK"** (separate DAK for each Atrium membership) — considered then rejected: user-DID-centric model means user has ONE DAK; per-Atrium isolation is an authorization concern (UCAN scoping) not a confidentiality-substrate concern.
- **"Hardware-token-bound DAK by default at v1-beta"** — considered then rejected: would constrain Benten deployment to hardware-token-enabled devices, breaking the "all three deployment shapes" commitment (CLAUDE.md baked-in #17). Hardware-token-bound is additive backend post-v1-beta.

---

## 15. Honest disagreement

### 15.1 The "F-full" framing is itself slightly imprecise

Ben's 2026-05-27 framing assembles 4 layers under one umbrella, but I want to flag that **the X-Wing-mislabel corrective (~24 LOC) is INDEPENDENT of F-full** — it's a defect in the existing Layer-B code that must be corrected regardless of whether F-full lands. Treating it as "part of F-full" risks the corrective being held hostage to F-full ratification timing. It should land FIRST, on its own merits, independent of F-full sub-phase placement.

This is a process disagreement, not an architectural one. Ben's overall F-full framing is correct; the corrective is just OUTSIDE F-full (and inside "pre-tag must-fix").

### 15.2 The "minimum-viable" framing in the original brief actively undermines the do-it-now bias

The original brief (and the supplementary brief) talks about "minimum-viable scope" + asks "what can be DEFERRED to Phase-4-Meta-Composing." This framing pushes the orchestrator's decision-making toward cutting, when the actually-correct framing per Ben's 2026-05-27 phase-ordering correction is "what wire-format-affecting pieces MUST be in Phase-4-Meta-Core given that the interface freezes at the close of that sub-phase; what UX-coupled pieces fit Phase-4-Meta-Composing; everything pre-tag."

I'd argue the next iteration of this brief should drop the "minimum-viable" language entirely and ask "what's the right SUB-PHASE placement for each piece, with default = include unless there's a specific dependency reason to defer."

This is partly already-reflected in the reframed brief I'm responding to ("Don't cut scope unless..."), but the residual "minimum-viable" language survives in some sentences. Worth scrubbing on next iteration.

### 15.3 The orchestrator's repeated framing of Phase-4-Meta-Composing as "post-v1-beta" or "v1.x-defer"

I see this framing appear MULTIPLE TIMES in the input documents (even after Ben's 2026-05-27 correction). The handoff doc itself flags this as "imprecision Ben corrected." This is a HIGH-PRIORITY framing-correction-target for future orchestration: **anytime the orchestrator says "v1-beta scope" when discussing Phase-4-Meta-Core, sub-substitute "pre-tag Phase-4-Meta-Core scope" to avoid implying that Phase-4-Meta-Composing is somehow optional or post-tag.** This is a documentation-of-framing-discipline candidate for `feedback_phase_ordering_precision` (new memory).

### 15.4 The deepest framing question I have for Ben

**Is the per-Node AEAD design (`K(N) = KDF(K_principal, N.cid)`) actually serving Benten's threat model, or is it producing the wrong shape?**

The current per-Node AEAD design (per CLAUDE.md baked-in #18 + Spike-E Interpretation-B path-tagged derivation) protects Node payloads against an attacker who has on-disk access to the redb database but NOT to K_principal. This is the "stolen device, attacker doesn't know password" scenario.

But the F-full design with DAK-encrypted-K_principal-at-rest provides EXACTLY THIS PROTECTION at the vault layer: K_principal is encrypted under DAK; without the password, the disk-access attacker can't decrypt K_principal; without K_principal they can't derive K(N); so the per-Node AEAD is doing the same work twice.

**The candidate simpler shape:** drop per-Node AEAD and just use the DAK-encrypted-vault. Save the Node-body-decryption cost (which is per-read overhead) and the K(N) derivation cost (which is per-Node overhead).

**Why this might be wrong** (counterargument I owe Ben): the per-Node AEAD also serves the "encrypt-to-recipient" composition pattern (per cryptographer review §5.4) — when sharing a Node with Alice, the Owner encrypts K(N) (the small key) to Alice's pubkey via HPKE; Alice decrypts the wrap to get K(N), then decrypts the Node. This is the KEM-DEM composition pattern. If we drop per-Node AEAD, encrypt-to-recipient becomes "encrypt the entire Node payload to Alice's pubkey via HPKE" which is more expensive per-share-operation.

So the per-Node AEAD has TWO roles: at-rest theft-protection AND key-wrap-pivot for sharing. The first role is redundant under F-full DAK; the second isn't.

**My disagreement-flag:** the per-Node AEAD design + the F-full DAK design have OVERLAPPING threat-model coverage but each has SOME UNIQUE role. I don't think this overlap is wrong, but I want to flag that the design rationale needs to clearly articulate "AEAD is for sharing-key-wrap; DAK is for offline-device-theft; they're complementary not duplicative." If Ben's mental model is that ONE of them is doing the at-rest-theft work, the design conversation is muddied.

This is a Ben-decision-surfacing call, not a flat disagreement. Surfacing it for Ben's call.

---

## 16. Evidence base

### 16.1 Cross-platform packages

- **keyring-core** ([github.com/open-source-cooperative/keyring-core](https://github.com/open-source-cooperative/keyring-core)) — v1.0.0; cross-platform credential store with `mock` + `sample` reference impls + per-OS backends; macOS Keychain Services, Windows Credential Manager, libsecret on Linux; iOS / Android via separate companion impls; refactor from monolithic `keyring` (now sample-only) completed May 2026.
- **keyring (legacy)** ([github.com/open-source-cooperative/keyring-rs](https://github.com/open-source-cooperative/keyring-rs)) — v4.0.1 (May 12, 2026); README explicitly says "Do not depend on this crate!" and points at `keyring-core`. Apache-2.0 / MIT dual-license. 731 stars.
- **tauri-plugin-stronghold** ([v2.tauri.app/plugin/stronghold/](https://v2.tauri.app/plugin/stronghold/), [docs.rs/crate/tauri-plugin-stronghold/latest](https://docs.rs/crate/tauri-plugin-stronghold/latest)) — v2.3.1; Tauri-only; IOTA-Stronghold-backed; Argon2id default key-derivation; Windows/Linux/macOS/iOS/Android via Tauri.
- **iota_stronghold** (upstream) ([github.com/iotaledger/stronghold.rs](https://github.com/iotaledger/stronghold.rs)) — latest v2.1.0 released 2024-05-13; Apache-2.0; audited 2022-05-04 by WITH Secure; no subsequent audit published.
- **tauri-plugin-biometric** (official) ([v2.tauri.app/plugin/biometric/](https://v2.tauri.app/plugin/biometric/)) — v2.0.0; primarily iOS + Android; verifies biometric availability + prompts.
- **tauri-plugin-biometry** (community / Choochmeque) ([github.com/Choochmeque/tauri-plugin-biometry](https://github.com/Choochmeque/tauri-plugin-biometry)) — v0.2.x; iOS + Android + macOS Touch ID + Windows Hello (no Linux); AES-256 on Windows under Windows-Hello-protected keys; macOS requires code-signed app for Keychain access.
- **argon2** (RustCrypto) ([docs.rs/argon2](https://docs.rs/argon2), [rustcrypto.org/key-derivation/hashing-password.html](https://rustcrypto.org/key-derivation/hashing-password.html)) — RFC 9106 implementation; Argon2id v0x13 default; OWASP-recommended params (`m_cost=19456`, `t_cost=2`, `p_cost=1`); 12.1M lifetime downloads.
- **zeroize / secrecy** (RustCrypto) ([docs.rs/zeroize](https://docs.rs/zeroize), [docs.rs/secrecy](https://docs.rs/secrecy/0.7.0/secrecy/)) — memory-hygiene primitives; volatile-write + atomic-fence; pure Rust no-FFI; WASM-compatible.
- **hpke** (Brendan McMillion) — canonical Rust HPKE impl; clean audit posture relative to `hpke-rs`.
- **hpke-rs** (Cryspen) — DO NOT USE per [symbolic.software/blog/2026-02-05-cryspen/](https://symbolic.software/blog/2026-02-05-cryspen/) Verification Theatre review + [rustsec.org/advisories/](https://rustsec.org/advisories/) RustSec advisories filed 2026-03-24.

### 16.2 Standards + precedents for device-link + remote-permission

- **Signal Provisioning protocol** ([signal.org/blog/a-synchronized-start-for-linked-devices/](https://signal.org/blog/a-synchronized-start-for-linked-devices/), [github.com/AsamK/signal-cli/wiki/Linking-other-devices-(Provisioning)](https://github.com/AsamK/signal-cli/wiki/Linking-other-devices-(Provisioning))) — QR + Curve25519 ephemeral keypair + signed provisioning message + one-time AES key for archived-message transfer.
- **Bitwarden Trusted Devices** ([bitwarden.com/help/about-trusted-devices/](https://bitwarden.com/help/about-trusted-devices/), [bitwarden.com/help/bitwarden-security-white-paper/](https://bitwarden.com/help/bitwarden-security-white-paper/)) — Device RSA keypair + key-wrap of account encryption key via device pubkey; multi-device coordinated via server-side encrypted-wrap.
- **1Password Security Design** ([1password.com/files/1password-white-paper.pdf](https://1password.com/files/1password-white-paper.pdf)) — Two-secret key derivation (password + high-entropy Secret Key); SRP for authentication; multi-device via server-side encrypted-vault.
- **CTAP 2.2 hybrid transport / caBLE** ([passkeys.dev/docs/reference/specs/](https://passkeys.dev/docs/reference/specs/), [corbado.com/glossary/cda](https://www.corbado.com/glossary/cda), [corbado.com/blog/webauthn-passkey-qr-code](https://www.corbado.com/blog/webauthn-passkey-qr-code)) — QR code + Bluetooth proximity check + encrypted-tunnel-back-to-server; 97–100% browser support Q1 2026.

### 16.3 Identity-recovery design-space

- **Shamir Secret Sharing best practices** ([github.com/WebOfTrustInfo/rwot8-barcelona/blob/master/draft-documents/shamir-secret-sharing-best-practices.md](https://github.com/WebOfTrustInfo/rwot8-barcelona/blob/master/draft-documents/shamir-secret-sharing-best-practices.md)) — RWOT working group's discussion of social-recovery via Shamir-sharded keys.
- **Blockchain Commons Social Key Recovery** ([blockchaincommons.com/articles/Project-Proposal-New-Social-Key-Recovery-Approach/](https://www.blockchaincommons.com/articles/Project-Proposal-New-Social-Social-Key-Recovery-Approach/)) — social-recovery framework via trustees.
- **DKMS Decentralized Key Management for SSI** ([hyperledger-indy.readthedocs.io/projects/sdk/en/latest/docs/design/005-dkms/README.html](https://hyperledger-indy.readthedocs.io/projects/sdk/en/latest/docs/design/005-dkms/README.html)) — Hyperledger Indy's design notes for decentralized key management.

### 16.4 Crypto primitives + Benten alignment

- **RFC 9106** Argon2 memory-hard function for password hashing ([datatracker.ietf.org/doc/html/rfc9106](https://datatracker.ietf.org/doc/html/rfc9106)).
- **RFC 8439** ChaCha20 and Poly1305 ([datatracker.ietf.org/doc/html/rfc8439](https://datatracker.ietf.org/doc/html/rfc8439)).
- **RFC 9180** HPKE ([datatracker.ietf.org/doc/html/rfc9180](https://datatracker.ietf.org/doc/html/rfc9180)).
- **CLAUDE.md baked-in #5** crypto-agility refinement (multiformats framing; codepoint-dispatch; never fork primitives; PQ-hybrid from first format version).
- **CLAUDE.md baked-in #15** v1-beta scope gate (confidentiality-half-of-Principal as a v1-beta-blocker; release-stage split into v1-beta + v1-GM).
- **CLAUDE.md baked-in #17** three-deployment-shape commitment (full peer / thin compute / embedded webview).
- **CLAUDE.md baked-in #18** authority-isolation vs confidentiality-isolation split (capabilities-on-cooperating-engine vs encryption-on-untrusted-host).
- **Prior reviews:**
  - Cryptographer review of encrypt-to-recipient ([origin branch `phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17`](https://github.com/BentenAI/benten-engine/tree/phase-4-meta-core/encrypt-to-recipient-review-cryptographer))
  - P2P-architect review ([origin branch `phase-4-meta-core/encrypt-to-recipient-review-p2p-architect @ 8cfb079c`](https://github.com/BentenAI/benten-engine/tree/phase-4-meta-core/encrypt-to-recipient-review-p2p-architect))
  - Standards-skeptic review ([origin branch `phase-4-meta-core/encrypt-to-recipient-review-standards-skeptic @ f5a0d0e4`](https://github.com/BentenAI/benten-engine/tree/phase-4-meta-core/encrypt-to-recipient-review-standards-skeptic))
  - Cryptographer review of Bird-of-Prey-vs-LAMPS ([origin branch `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b`](https://github.com/BentenAI/benten-engine/tree/phase-4-meta-core/cryptographer-review-bird-of-prey))
  - Compromise #31 + Inv-15 (`docs/SECURITY-POSTURE.md` + `docs/INVARIANT-COVERAGE.md`)

### 16.5 Threat-model / attack-class background

- **Verification Theatre IACR 2026/192** ([symbolic.software/blog/2026-02-05-cryspen/](https://symbolic.software/blog/2026-02-05-cryspen/)) — Nadim Kobeissi's review of Cryspen's high-assurance crypto claims; 13 vulnerabilities identified; load-bearing for "don't use hpke-rs."
- **RustSec Advisory Database** ([rustsec.org/advisories/](https://rustsec.org/advisories/)) — `ml-kem` / `libcrux-ml-dsa` / `hpke-rs` advisories; subscribe + monitor.

---

## 17. Self-assessment

### Confidence level

- **HIGH confidence** on:
  - The wave-sequencing recommendation (§5.1) — the decision rule "wire-format-affecting → Phase-4-Meta-Core; UX-coupled → Phase-4-Meta-Composing; both pre-tag" is correctly derived from CLAUDE.md baked-in #15 + Ben's 2026-05-27 phase-ordering correction.
  - The package-landscape table (§3) — verified each entry against primary sources (docs.rs / GitHub / official Tauri docs).
  - The DAK substrate cryptographic design (§2) — RustCrypto-standard pattern; Argon2id + HKDF + ChaCha20-Poly1305 is the well-trodden pattern documented in [rustcrypto.org/key-derivation/hashing-password.html](https://rustcrypto.org/key-derivation/hashing-password.html).
  - The "don't use `hpke-rs`" recommendation (§3.3) — verified against the Verification Theatre review.
  - The "don't depend on legacy `keyring` crate" recommendation (§3.1) — verified against the GitHub README.

- **MEDIUM-HIGH confidence** on:
  - The LOC estimates (§5.1, §12.1) — order-of-magnitude; could be off by 30–50% per piece given cross-platform glue variance.
  - The remote-permission-call wire protocol design (§6) — modeled after Signal Provisioning + CTAP 2.2 hybrid-transport precedents; sound shape but warrants the §6.7 dedicated security mini-review.
  - The multi-device key-wrap design (§7) — same Signal Provisioning precedent; sound shape; minor open questions around RotationLog interaction.

- **MEDIUM confidence** on:
  - The wall-clock timeline estimate (§12.2) — extrapolated from Phase-4-Foundation + Phase-4-Meta-Core observed tempo; high variance.
  - The unified-envelope elegant shape (§14.1) — proposed but warrants cryptographer review against the "pseudo-keypair derived from symmetric DAK" pattern.
  - The audit-scope estimate (§10.2) — based on cryptographer-review's "1 person-week per major surface" heuristic; the actual scope depends on audit firm's depth.

- **LOWER confidence** on:
  - The "per-Node AEAD might be redundant under F-full DAK" disagreement-flag (§15.4) — this is a question to surface to Ben, not a claim. May be wrong; needs cryptographer + Ben input.
  - The exact Tauri biometric plugin choice (official vs Choochmeque community) — depends on the Phase-4-Meta-Composing UX requirements that aren't fully decided.
  - The identity-recovery design-space flagging (§9) — deliberately under-designed; the RecoveryHook trait shape should be reviewed by Ben before lock-in.

### What additional review I would want before commit

1. **Cryptographer review of the unified-envelope elegant shape (§14.1)** — specifically the "pseudo-keypair derived from symmetric DAK via HKDF" pattern. Is it sound? Is there a simpler shape?

2. **Cryptographer + security review of the remote-permission-call wire protocol (§6)** — per §6.7. The attack classes (replay, confused-deputy, UI-deception, clock-skew) all need formal threat-modeling before implementation lands.

3. **Tauri-ecosystem review of `tauri-plugin-stronghold` + `tauri-plugin-biometric` + `tauri-plugin-biometry`** — which is right for Phase-4-Meta-Composing? Depends on Tauri-stack maturity at write-time that I haven't deeply verified.

4. **Cross-platform CI matrix review** — what's the cost of testing each F-full piece on macOS / Linux (with + without Secret Service) / Windows / iOS / Android? The Phase-4-Meta-Core MINIMUM is desktop full-peer + Tauri-shell smoke; expanding to mobile adds CI cost.

5. **Identity-recovery `RecoveryHook` trait design review** before locking it at Phase-4-Meta-Composing close — the trait shape constrains the eventual protocol design space.

6. **Ben review of §15.2 + §15.3** (framing-discipline candidate) — the "minimum-viable scope" + "Phase-4-Meta-Composing as post-tag" language patterns should be scrubbed from future orchestrator docs.

### What this review does NOT cover

- The construction-soundness depth on per-primitive choices (cryptographer review territory).
- The standards-process maturity (standards-skeptic review territory).
- The P2P-architectural fit at fine grain (P2P-architect review territory).
- The Phase-4-Meta-Composing admin-UI design (separate from F-full Layer-D wire-format work).
- The full identity-recovery protocol design (deliberately deferred to v1-assessment-window per CLAUDE.md baked-in #15).
- The cryptographic audit RFP / firm selection (project-budget concern).

### Self-critique

I've recommended the largest possible F-full scope, which lands the substrate work pre-tag but at substantial wall-clock + audit cost (~4–5 weeks; ~3 person-weeks of external audit). A more conservative reviewer might recommend cutting Layer-D's remote-permission-call protocol to v1.x, on the grounds that it's the highest-security-risk piece and most prone to design churn during external audit. **I considered this but rejected it** because (a) Ben emphatic 2026-05-27 "REQUIRED, not cuttable," (b) the wire format must be locked at Phase-4-Meta-Core or it becomes a wire-break post-tag, (c) the AI-agent-dispatch tempo can absorb the work in the wall-clock window. If Ben's tolerance for v1-beta-tag schedule slippage is LOWER than I'm assuming, the §6 protocol is the natural item to defer-with-explanation per HARD-RULE-12 clause-c.

I've also assumed that "shipping the SUBSTRATE + minimum-platform-glue at Phase-4-Meta-Core" is achievable without admin-UI infrastructure. If admin-UI dependencies pull more F-full pieces into Phase-4-Meta-Composing than I've estimated, the LOC totals shift but the wave-sequencing-table approach holds.

---

**End of review.**
