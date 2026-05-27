# Option F+ pseudo-keypair pattern — senior-cryptographer review

**Branch:** `phase-4-meta-core/option-f-plus-pseudo-keypair-review`
**Reviewer lens:** Senior cryptographer (construction soundness + side-channel surface + ecosystem precedent + architectural utility)
**Inputs reviewed:**
- `.addl/phase-4-meta/e2r-ffull-scope-review.md` §14.1 (Option F+ extra-reflection-pass) at `origin/phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`
- `.addl/phase-4-meta/encrypt-to-recipient-review-cryptographer.md` at `origin/phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17`
- RFC 9180 (HPKE)
- `draft-connolly-cfrg-xwing-kem-10`
- `draft-ietf-hpke-pq-04` + `draft-irtf-cfrg-concrete-hybrid-kems-03`
- FIPS 203 (ML-KEM)
- Arriaga et al., "Tempo: ML-KEM to PAKE Compiler Resilient to Timing Attacks" (IACR ePrint 2025/1399)
- Filippo Valsorda, "age and Authenticated Encryption" + age-discussion #463
- Bitwarden Security Whitepaper; 1Password Security Design Whitepaper; Signal Desktop / Molly at-rest design
- OPAQUE aPAKE construction (IACR ePrint 2018/163; `draft-irtf-cfrg-opaque-13`)

---

## 0. Reading-order note

The decision-load-bearing sections are **§1 (executive recommendation, NO-GO with high confidence)**, **§2 (the IND-CCA2 reduction analysis — the pattern IS sound under standard assumptions; that is NOT the failure mode)**, **§4 (the actual failure mode: ML-KEM keygen-from-secret-seed is a timing oracle that turns the password-guessing surface from "offline-after-Argon2id" into "online-timing-side-channel-during-keygen")**, **§5 (architectural utility analysis — the "one primitive" elegance is superficial; Layer-B can never use HPKE; the unification is 3-out-of-4 not 4-out-of-4)**, and **§6 (recommendation: take Option B-equivalent — AEAD-under-DAK for Layer-A; HPKE-mode-base for Layer-C/D)**.

Sections §3, §7, §8 are supporting evidence; §9 is the citation base; §10 is self-assessment.

---

## 1. Executive recommendation

**NO-GO on Option F+ pseudo-keypair pattern for Layer-A (vault).** Confidence: **HIGH** on the architectural recommendation; **MEDIUM-HIGH** on the magnitude of the side-channel concern (depends on specific ML-KEM implementation and adversary model). Biggest concern: **the ML-KEM-768 KeyGen step inside HPKE's `DeriveKeyPair` contains variable-time rejection sampling whose timing depends on the secret seed `ρ`** — when that seed is derived from a low-entropy password (even via Argon2id), the keygen timing becomes a side-channel through which an adversary with local code-execution or co-resident-VM measurement can mount an **online dictionary attack that bypasses Argon2id's memory-hardness**. This is the exact attack the Arriaga et al. "Tempo" paper (IACR ePrint 2025/1399) was constructed to mitigate in the PAKE setting, and the F+ vault design is structurally identical to the vulnerable PAKE construction it warns against. The mitigation (constant-time SampleNTT, or transmitting ρ in the clear) is feasible but adds substantial implementation + audit cost — for a layer that gains zero functional benefit from being asymmetric.

The pattern IS cryptographically sound in the IND-CCA2-reduction sense (HPKE's modular-KEM-substitution argument applies, RFC 9180 §7.1.3 DeriveKeyPair admits deterministic-derived keypairs by design, and X-Wing's `GenerateKeyPairDerand` is the explicit deterministic-from-seed API). The failure mode is NOT "the construction is unsound." The failure mode is **(a) implementation hazard surface (ML-KEM keygen side-channels under secret seed) + (b) zero functional benefit at Layer-A (the vault use case is encrypt-to-self; the natural primitive is symmetric AEAD; HPKE is the wrong-shape tool) + (c) the architectural elegance claim is superficial (Layer-B AEAD-per-Node is structurally symmetric and cannot use HPKE; the "one primitive across all 4 layers" claim is actually 3-out-of-4 at best)**.

**Recommendation: pursue Option B-equivalent.** Use **ChaCha20-Poly1305 AEAD under DAK for Layer-A** (vault); use **HPKE-mode-base[MLKEM768-X25519] for Layer-C (drop) + Layer-D (key wraps + remote permission)**; Layer-B per-Node AEAD stays as it is. This matches the consensus design across every reviewed production system (age scrypt-recipient; Bitwarden; 1Password; OPAQUE's encrypted-private-key blob; Molly Signal-desktop fork).

---

## 2. Cryptographic soundness analysis — is the pseudo-keypair pattern IND-CCA2-secure?

### 2.1 The construction restated formally

Let `Π = (KeyGen, Encap, Decap)` be a KEM (specifically X-Wing per `draft-connolly-cfrg-xwing-kem-10` §5). Let `Π.KeyGenDerand(seed)` denote the deterministic keypair-derivation function taking a 32-byte seed. Let `HKDF` be the HMAC-SHA256 extract-then-expand KDF. Let `Argon2id(pw, salt, params) → seed₀` be the password-derived seed.

The F+ Layer-A construction is:

```
(sk_vault, pk_vault) := Π.KeyGenDerand( HKDF(seed₀, "benten-vault-v1") )
ciphertext           := HPKE_mode_base.Seal(pk_vault, K_principal, aad)
```

Decryption:

```
seed₀                 := Argon2id(pw_input, salt, params)
(sk_vault, pk_vault') := Π.KeyGenDerand( HKDF(seed₀, "benten-vault-v1") )
K_principal           := HPKE_mode_base.Open(sk_vault, ciphertext, aad)
```

### 2.2 The IND-CCA2 reduction (this part DOES work)

HPKE-mode-base is IND-CCA2-secure under the assumption that its KEM is IND-CCA2-secure (RFC 9180 §9.1.2, citing [CS01]). X-Wing is IND-CCA2-secure under the hybrid assumption that either the strong-DH assumption holds in X25519 OR ML-KEM-768 is IND-CCA2-secure, with SHA3-256/SHAKE-256 modeled as random oracles (Barbosa et al., IACR CIC Vol. 1 No. 1, 2024-04-09).

The deterministic-keypair-from-seed construction (`Π.KeyGenDerand`) is **explicitly admitted** by both standards:

- **RFC 9180 §7.1.3 (DeriveKeyPair):** *"`DeriveKeyPair()` is a deterministic algorithm to derive a key pair `(skX, pkX)` from the byte string `ikm`."* And: *"The keys that `DeriveKeyPair()` produces have only as much entropy as the provided input keying material. For a given KEM, the `ikm` parameter given to `DeriveKeyPair()` SHOULD have length at least `Nsk`, and SHOULD have at least `Nsk` bytes of entropy."* ([RFC 9180 §7.1.3](https://datatracker.ietf.org/doc/html/rfc9180#section-7.1.3))
- **X-Wing draft `GenerateKeyPairDerand`:** *"Takes a 32-byte seed as input and deterministically derives the keypair... `sk` must be 32 bytes."* ([draft-connolly-cfrg-xwing-kem-10 §5](https://datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10))

**Therefore, in the formal-model sense, if `seed₀` is a uniformly random 32-byte string, the F+ construction is IND-CCA2-secure.** The proof goes: seed is uniform → `Π.KeyGenDerand(seed)` is computationally indistinguishable from `Π.KeyGen()` output (because the derand function is the same primitive used by KeyGen with `seed = random(32)`) → standard HPKE-mode-base[X-Wing] IND-CCA2 proof applies.

### 2.3 But `seed₀` is NOT uniformly random — it is Argon2id(low-entropy password)

This is the crack in the formal reduction. Argon2id raises the per-guess cost from ~1 hash to ~46 MiB-seconds of computation ([RFC 9106](https://datatracker.ietf.org/doc/html/rfc9106) §4 with OWASP params `m_cost=46 MiB, t_cost=1`), but it does NOT change the entropy of the underlying password. A 4-digit PIN remains a 4-digit PIN; Argon2id just makes each attempt cost ~$10⁻³$ instead of ~$10⁻⁹$ on commodity hardware.

**RFC 9180 §9.5 explicitly warns against this exact substitution:** *"HPKE's PSK mechanism is not suitable for use with a low-entropy password as the PSK. HKDF is not designed to slow down dictionary attacks."* ([RFC 9180 §9.5](https://datatracker.ietf.org/doc/html/rfc9180#section-9.5)). The same warning applies to using HKDF-of-low-entropy-input to derive a KEM-`ikm` — HKDF's extract-then-expand does NOT add work-factor.

The Argon2id wrapper PARTIALLY addresses this — at-rest brute-force is throttled to ~46 MiB-sec/attempt. So if the ONLY attack surface is "adversary has stolen the vault and is trying to brute-force the password offline," the F+ construction is **as secure as** AEAD-under-DAK (both are bottlenecked by Argon2id). **This part of the design is OK.**

The failure mode is the **side-channel** that the asymmetric-keypair-derivation introduces and that the symmetric-AEAD construction does not. That is §4.

### 2.4 Verdict on §2

**Construction-level soundness: GO.** The reduction works in the random-oracle model under the standard HPKE-RFC-9180 + X-Wing assumptions, provided `seed₀` has sufficient entropy. Whether `seed₀` has sufficient entropy depends on **(a) the user's password strength** (unbounded; the design can't fix a 4-digit PIN) and **(b) whether the side-channel surface in §4 effectively reduces the work factor to "online dictionary"** (the issue).

---

## 3. Real-world precedent table — who uses "asymmetric keypair derived from symmetric secret" for at-rest encryption?

This is the most informative section. **The pattern of "encrypt-to-derived-pseudo-keypair under a password-derived symmetric secret" is approximately not used in any production at-rest-encryption system I could identify.** Multiple production systems EXPLICITLY chose the symmetric-AEAD-under-KDF-derived-key path. Some use deterministic-keypair-derivation, but only for *different* use cases (HD wallets, code-signing reproducibility, OpenSSH key reproducibility) — never for the "encrypt-to-self vault" use case Option F+ targets.

| System | Vault / at-rest design | Asymmetric? | Derived-from-password? | Notes |
|---|---|---|---|---|
| **age (Filippo Valsorda)** — file encryption | `scrypt`-recipient: passphrase → scrypt → 32-byte file key → ChaCha20-Poly1305 over file. `x25519`-recipient: separate, uses real X25519 keypair generated by ECDSA-style random. | NO for passphrase; YES for x25519 | symmetric-from-passphrase | Valsorda EXPLICITLY chose symmetric-AEAD over derived-asymmetric-keypair. Per age-spec, scrypt-recipient is intentionally NOT a derived-x25519-keypair. Per age-discussion #463 ([github.com/FiloSottile/age/discussions/463](https://github.com/FiloSottile/age/discussions/463)): "if you decrypt a file with a passphrase you have an expectation that whoever produced it knew the passphrase." Symmetric is the right shape because the threat model is "I'm encrypting to myself / to someone who knows the passphrase," not "I'm encrypting to a public identity." |
| **Bitwarden** — password-manager vault | Master password → PBKDF2-SHA256 (600k iter) → Master Key (32 bytes) → HKDF-stretch → Stretched Master Key → encrypts Protected Symmetric Key (random 64-byte key) under AES-256-CBC + HMAC-SHA256. | NO | symmetric-from-password | The "vault key" is a freshly-random symmetric key; the master-password-derived key is purely a key-wrap key for the random vault key. No asymmetric primitive in the vault-unlock path. RSA-2048 keypair exists, but is for *cross-user vault sharing*, not for vault encryption itself. Bitwarden whitepaper. ([bitwarden.com/help/bitwarden-security-white-paper/](https://bitwarden.com/help/bitwarden-security-white-paper/)) |
| **1Password** — password-manager vault | Two-secret KDF: Master Password + Secret Key → HKDF → Account Unlock Key (AUK; symmetric 256-bit). Vault items encrypted with random vault key under AES-256-GCM. Vault key encrypted with AUK (symmetric). User RSA-2048 keypair exists for *sharing*, encrypted-at-rest under AUK. | NO for vault unlock; YES for sharing (RSA encrypted-at-rest under symmetric AUK) | symmetric-from-password | The RSA private key is encrypted-at-rest under the AUK using AES-256-GCM — i.e., the asymmetric private key is the *plaintext* protected by symmetric encryption-under-password, NOT *derived* from the password. This is the canonical "encrypted-private-key blob" pattern. ([1passwordstatic.com/files/security/1password-white-paper.pdf](https://1passwordstatic.com/files/security/1password-white-paper.pdf), [agilebits.github.io/security-design/deepKeys.html](https://agilebits.github.io/security-design/deepKeys.html)) |
| **OPAQUE aPAKE (CFRG draft-13)** | OPRF result → KDF → key-wrap-key → AES-GCM-encrypts the client's private signing/auth key blob stored at server. | encrypted-private-key blob is asymmetric content, but blob encryption is symmetric | symmetric-from-OPRF-output | The seminal aPAKE design. *Client* holds a long-lived asymmetric keypair; that keypair is **encrypted as a blob under a password-derived symmetric key** and stored at server. On unlock, client retrieves blob, runs OPRF, derives symmetric unwrap key, decrypts the asymmetric keypair. This is the canonical reference design for "encrypt-to-self with password" and it uses **symmetric AEAD**, NOT password-derived asymmetric keypair. OPAQUE IACR ePrint 2018/163 ([eprint.iacr.org/2018/163](https://eprint.iacr.org/2018/163)). |
| **Signal Desktop (pre-2024)** | Plaintext key-of-keys; OS file permissions only. (Acknowledged limitation; later improved.) | N/A | N/A | Signal stated: "never intended to provide encryption at rest." ([techtarget.com](https://www.techtarget.com/searchsecurity/answer/How-did-Signal-Desktop-expose-plaintext-passwords)) |
| **Molly (Signal-Android fork)** | Passphrase → Argon2id → 256-bit symmetric key → AES-256-CBC encrypts the database key blob. HMAC-SHA256 for authentication. | NO | symmetric-from-password | Independently re-derives the canonical "vault = symmetric-encrypted blob under password-derived key" pattern that the broader ecosystem has converged on. ([github.com/mollyim/mollyim-android/wiki/Data-Encryption-At-Rest](https://github.com/mollyim/mollyim-android/wiki/Data-Encryption-At-Rest)) |
| **Tauri Stronghold (IOTA)** | Argon2id-derived key → secret-box (XChaCha20-Poly1305) over the Stronghold "snapshot" containing all secrets including private keys. | NO | symmetric-from-password | Same canonical pattern. The Stronghold snapshot is one big AEAD-encrypted blob; private keys live inside. ([github.com/iotaledger/stronghold.rs](https://github.com/iotaledger/stronghold.rs)) |
| **BIP32 / SLIP-0010 HD wallets** | Master seed (typically derived from BIP-39 mnemonic via PBKDF2) → HMAC-SHA512 → deterministic chain of secp256k1 (or ed25519, NIST P-256) keypairs. | YES — asymmetric keypairs ARE derived from seed | YES — but for a *different use case* | This IS the "deterministic asymmetric keypair from secret seed" pattern. But the use case is **derive-an-identity-keypair-for-on-chain-signing**, NOT **encrypt-to-self-with-password-derived-pseudo-keypair**. The HD-wallet pattern is one-way: derive sk, publish pk, use sk to sign blockchain transactions. There is no "encrypt-to-self under this pseudo-pubkey" step. The seed timing-side-channel surface is well-studied for secp256k1 specifically and acceptable in the BIP32 threat model (no co-resident-VM adversary; key-use is signing, not keygen-during-unlock). ([github.com/satoshilabs/slips/blob/master/slip-0010.md](https://github.com/satoshilabs/slips/blob/master/slip-0010.md)) |
| **`determin-ed` (OpenSSH key reproducibility)** | Password + salt → PBKDF2 → 32-byte seed → Ed25519 keypair via standard Ed25519 keygen. | YES | YES — but use case is SSH-key reproducibility | Like BIP32: derive a signing keypair from password; PUBLISH the pubkey. Not "encrypt-to-self under pseudo-pubkey." Ed25519 keygen is also more constant-time than ML-KEM-768 keygen. ([github.com/joonakannisto/determin-ed](https://github.com/joonakannisto/determin-ed)) |
| **Signal PQXDH** | X25519 + ML-KEM-1024 hybrid KEM in PQXDH initial handshake. NOT password-derived; both pre-shared. | YES (for handshake) | NO | Reference for "ML-KEM-in-production" but explicitly NOT derived-from-password. |
| **MLS-PQ (`draft-ietf-mls-pq-ciphersuites-04`)** | HPKE-mode-base[MLKEM768-X25519] for group key encapsulation. Recipient pubkeys are real (generated with high-entropy randomness), published on Atrium. | YES | NO | Reference for "HPKE-mode-base[MLKEM768-X25519] in production." Recipient keypairs are never derived from passwords. |

**Pattern conclusion:** every reviewed production at-rest-vault system uses **symmetric AEAD under a KDF-derived key**. The "deterministic asymmetric keypair from seed" pattern exists, but ONLY for use cases where the public key is published / used for signing — never for the encrypt-to-self vault use case. **The absence of precedent is itself significant evidence.** The cryptographic community has had ~25 years to converge on a vault-encryption design; the consensus is unambiguous; F+ is proposing to depart from it.

---

## 4. Implementation hazard surface — the load-bearing concern

This is the section that drives the NO-GO. The IND-CCA2 reduction in §2 holds in the formal model. The failure mode is in the **implementation timing surface** of ML-KEM-768 keygen-from-secret-seed.

### 4.1 ML-KEM-768 KeyGen contains variable-time rejection sampling that depends on the seed

ML-KEM-768 KeyGen (FIPS 203 §6.1) takes a 32-byte seed `d` (and a 32-byte randomizer `z`) and expands `d` into a 3×3 matrix `Â` of polynomial coefficients via `SampleNTT`, which uses **rejection sampling** to convert SHAKE-128 output bytes into elements of Z_{3329}.

Quote from the Arriaga et al. "Tempo" paper (IACR ePrint 2025/1399), the closest published analysis of the exact pattern:

> *"ML-KEM expands a short seed ρ into a large matrix A of polynomial coefficients using rejection sampling—a process that is variable-time but usually does not depend on any secret. However, in PAKE protocols that password-encrypt the compressed public key, this introduces the risk of timing honest parties and mounting an offline dictionary attack against the measurement."* ([eprint.iacr.org/2025/1399](https://eprint.iacr.org/2025/1399.pdf))

The Tempo paper is constructed to mitigate exactly this attack class. **The F+ Layer-A vault construction is structurally identical to the vulnerable PAKE pattern Tempo addresses:** a password-derived seed feeds ML-KEM keygen, and the keygen's rejection-sampling timing depends on the seed.

### 4.2 The attack scenario against F+ Layer-A

Adversary model (one realistic instance): an attacker has malware-level code-execution on the user's device but does NOT have the password (e.g., a compromised npm dep, a malicious browser extension co-resident with the Tauri shell, a co-tenanted VM with cache-timing observability via a microarchitectural side-channel).

Attack flow:

1. User attempts vault unlock. Engine runs Argon2id → `seed₀` (32 bytes; depends on password).
2. Engine runs `HKDF(seed₀, "...")` → `ikm` for X-Wing.
3. Engine runs X-Wing `GenerateKeyPairDerand(ikm)`, which internally runs ML-KEM-768 KeyGen with `d = ikm[..32]`. **The SampleNTT rejection-sampling loop count depends on `d`.**
4. Attacker measures keygen-time (via timing oracle / cache side-channel / shared-resource contention).
5. Attacker has obtained the vault file. Attacker now runs an OFFLINE dictionary attack: for each candidate password `pw_i`, compute Argon2id(`pw_i`) → `seed_i` → HKDF → run ML-KEM-768 KeyGen → measure rejection-sampling loop count. If the loop count matches the measured target, `pw_i` is a strong candidate. (The loop-count signal is only a few bits of leakage per attempt, but it COMPOUNDS across multiple unlock observations and ESPECIALLY across the multiple rejection-sample loops inside one keygen.)
6. **The dictionary attack still pays Argon2id cost per candidate**, but it now has a *signal* that distinguishes correct passwords from wrong ones much faster than vault-decryption-trial. For passwords with limited entropy (<40 bits), this becomes practical.

This attack does NOT apply to AEAD-under-DAK: ChaCha20-Poly1305 has no rejection sampling; there's no internal data structure whose construction-time depends on the key. The keygen vs. encryption distinction is real here.

### 4.3 Constant-time mitigations exist but add real cost

Per the Tempo paper, three mitigations exist:
- **Constant-time SampleNTT** (refactor rejection sampling into branchless modular reduction). All proposed CT algorithms are "slower than rejection sampling implementations." Currently NOT implemented by mainstream ml-kem libraries (incl. RustCrypto `ml-kem`).
- **Reveal ρ in the clear** (Tempo's design). Inapplicable to F+ Layer-A: in F+, `ρ` IS the secret (it's the DAK-derived seed). The whole point of the design is that `ρ` is held under the password.
- **Vendor the patched ml-kem implementation + add to Benten's audit scope.**

Per CLAUDE.md baked-in #5 ("Never fork, never reimplement, crypto primitives"), the vendor-and-patch path is structurally forbidden. The constant-time alternatives don't yet exist in RustCrypto stable releases. **F+ would either require waiting for upstream CT-ml-kem (unscheduled), or violating baked-in #5.**

### 4.4 Other side-channel surfaces specific to pseudo-keypair pattern

- **X25519 scalar from secret seed:** X25519 keygen takes 32 bytes, clamps high/low bits, treats as scalar. The clamping is constant-time. The scalar multiplication for public-key generation `pk = scalar * G` is constant-time in well-implemented X25519 (RustCrypto `x25519-dalek` documents constant-time scalar mult). **This surface is safe.** The X25519 half of X-Wing is fine; the problem is the ML-KEM half.
- **HKDF on secret seed:** HMAC-SHA256 is constant-time in standard implementations. **Safe.**
- **Argon2id memory access:** Argon2**i** is constant-time; Argon2**d** is data-dependent; Argon2**id** is hybrid (Argon2i for first half-pass, Argon2d afterward). RFC 9106 recommends Argon2id when side-channels are a viable threat. **For the password-unlock path, Argon2id is the right choice and this surface is the same for F+ and AEAD-under-DAK.** No differential side-channel between the two designs here.
- **ML-KEM Decap inside HPKE.Open:** Has a known side-channel surface (CCA-style chosen-ciphertext attacks per "One Time is Enough," IACR 2024/2051). This surface is INDEPENDENT of whether the keypair is derived-from-password vs. random — it applies to all ML-KEM usage. But: in F+ Layer-A, the adversary controls the vault ciphertext (it's on disk; the adversary has it), so the CCA-attack surface is more reachable than in HPKE-mode-base[X-Wing] for ephemeral encap.

### 4.5 Verdict on §4

**The side-channel hazard surface for the F+ Layer-A pseudo-keypair pattern is materially larger than for the AEAD-under-DAK pattern.** The biggest issue is ML-KEM-768 KeyGen's variable-time rejection sampling on a secret seed, which is exactly the attack class the Tempo paper exists to fix and which has no off-the-shelf constant-time mitigation in the RustCrypto ml-kem crate at write-time. The auditor's job would include verifying that the keygen path is constant-time under secret seed — a real cost.

**This is the load-bearing reason for the NO-GO.**

---

## 5. Architectural utility analysis — is "one primitive" actually elegant?

Even if §4 is wrong and constant-time mitigations land in upstream ml-kem, the architectural utility claim deserves scrutiny.

### 5.1 The Option F+ claim restated

§14.1 of the F-full review claims: *"The HPKE-mode-base[MLKEM768-X25519] primitive can serve ALL FOUR layers."* The "four layers" are A (vault) / B (per-Node AEAD) / C (drop) / D (key wraps).

### 5.2 Layer-B is structurally NOT HPKE — the claim is already 3-of-4 not 4-of-4

The F+ §14.1 text itself concedes: *"Layer-B per-Node AEAD: stays AEAD with derived K(N) — actually this IS structurally different (symmetric not asymmetric) so HPKE doesn't unify here. Per-Node AEAD remains its own primitive."*

So even within the F+ proposal, the unification is **3 layers, not 4**. The "ONE primitive across four layers" framing in §1 + §6.4 of the F-full review is incorrect; the actual claim is "one asymmetric primitive across three of the four use sites." This weakens the elegance argument.

### 5.3 The three remaining sites (A/C/D) are NOT structurally the same use case

The F+ pitch treats A/C/D as "protect some key material X under some authorization context Y." That framing elides a critical distinction:

| Use site | Sender identity | Recipient identity | Sender == Recipient? | Threat model |
|---|---|---|---|---|
| Layer-A vault | The user (at vault-creation time) | The user (at unlock time) | YES | Offline at-rest brute-force; local side-channels |
| Layer-C drop | The user (at share time) | A remote peer (Alice) | NO | Network adversary; peer-side compromise |
| Layer-D device-link wrap | User-device-A | User-device-B (new device joining user's mesh) | NO (different devices, same user identity) | Network MITM; provisioning-session capture |
| Layer-D remote-permission | User-device-A | User-device-B | NO | Same as device-link, but per-operation |

**Layer-A is the encrypt-to-self case.** The "recipient" is a notional pseudo-identity (the DAK-derived pseudo-pubkey) — but it's not a separate party; it's a syntactic artifact of forcing the symmetric-encrypt-to-self use case into an asymmetric-encrypt-to-recipient shape.

**This is the same design tension Filippo Valsorda articulated for age:** the scrypt-recipient and x25519-recipient have *different security shapes*. The scrypt-recipient embeds "the encryptor knew the passphrase" semantics; the x25519-recipient does not. Forcing them into one primitive collapses meaningful design distinctions.

### 5.4 The "one audit surface" claim is partly correct but partly misleading

True: if all 3 use sites use HPKE-mode-base[X-Wing], the HPKE library + X-Wing library are audited once. The wrapper code (Encap / Decap / context binding) is one code path.

But: the F+ design ADDS new audit surface that AEAD-under-DAK doesn't have:
- The HKDF step from DAK to `ikm` (codepoint binding, "benten-vault-v1" domain separation correctness).
- The ML-KEM-768 KeyGen-from-secret-seed timing-side-channel question (§4).
- The composition argument: HPKE-RFC-9180 was proven IND-CCA2 against an adversary that does not control the recipient's keypair seed. Does the proof still hold when the recipient's seed is adversarially-influenced via a chosen-password attack? (The reduction works for adversaries that don't know the seed, but the chosen-seed attack class is not standard HPKE adversary model.)

The audit cost of HPKE-mode-base[X-Wing] for Layer-C/D is unchanged by F+ (it's the same code path). The added cost of F+ is **the analysis of the pseudo-keypair derivation step at Layer-A and its interaction with the password-attack surface**. That added cost > the AEAD-under-DAK audit cost. Net audit-surface delta is **larger**, not smaller.

### 5.5 The "Layer-A IS just Layer-C with the recipient being yourself" framing is structurally tempting but operationally wrong

The §14.1 pitch's most persuasive line: *"Layer-A vault is 'encrypted to a DAK-derived pubkey' which means the SAME unlock code path can also handle 'vault sealed by device A on behalf of device B' (which is what device-link does anyway)."*

This is appealing in the abstract but operationally wrong:

- **Layer-D device-link transfer of K_principal from device A to device B** uses device B's REAL keypair (generated with high-entropy randomness on device B; pubkey published in the Atrium peer-mesh; user-DID-signed). This is NOT a derived-from-password keypair. So the device-link code path is **already** HPKE-encap-to-real-keypair; it doesn't need Layer-A unification.
- **Layer-A vault unlock** never crosses a network boundary. It happens entirely on-device, during engine startup, against the local disk file. The "encrypted to a DAK-derived pubkey" framing makes vault-unlock LOOK like an HPKE.Open against a remote sender, but operationally it's just a local symmetric decryption with a key derived from a password. AEAD-under-DAK reflects this directly.

The conceptual unification of "encrypt-to-self == encrypt-to-recipient-who-happens-to-be-yourself" is exactly the kind of elegance-trap that gets cryptographic designs in trouble. It's the same trap that would say "all encryption is just public-key encryption with the public key being a single-party-known value." That framing is technically true (a symmetric key IS a degenerate keypair where pk == sk == k) but it doesn't make symmetric encryption a subset of HPKE; it makes HPKE the wrong-shape tool for symmetric use cases.

### 5.6 Verdict on §5

**The architectural elegance claim is superficial.** The unification is 3-of-4 not 4-of-4; the 3 sites have different security shapes that the unified envelope obscures; the audit-surface delta is net-larger not net-smaller; the "encrypt-to-self == encrypt-to-recipient" framing is a category error.

**The correct unification is at the WIRE-FORMAT-DISCRIMINATOR layer (codepoint dispatch), not at the PRIMITIVE-CHOICE layer.** Benten can have a single `EncryptedEnvelope { codepoint, payload, aad_binding }` outer shape that dispatches between AEAD-under-DAK (Layer-A codepoint) and HPKE-mode-base[X-Wing] (Layer-C/D codepoints). The envelope is unified; the primitive isn't. This is the CLAUDE.md baked-in #5 crypto-agility pattern operating as designed.

---

## 6. Recommendation

### 6.1 Primary recommendation: **Option B-equivalent (separate primitives)**

| Layer | Primitive | Rationale |
|---|---|---|
| **A (vault)** | ChaCha20-Poly1305 AEAD under DAK derived via Argon2id + HKDF | Matches ecosystem consensus (age scrypt-recipient, Bitwarden, 1Password, OPAQUE encrypted-private-key, Molly, Stronghold). Zero side-channel surface from password-derived seed feeding ML-KEM keygen. Strictly less code. Strictly faster (~0.1ms vs ~5-10ms for ML-KEM-768 keygen-from-seed). Straightforward to audit. |
| **B (per-Node AEAD)** | ChaCha20-Poly1305 AEAD under `K(N) = KDF(K_principal, N.cid)` | Existing design; out of F+ scope. |
| **C (encrypt-to-recipient drop)** | HPKE-mode-base[MLKEM768-X25519] (real X-Wing per §1 corrective in prior cryptographer review) | Per Option B in the prior cryptographer review. Standards-Schelling-point default. |
| **D (device-link key wrap; remote-permission grant)** | HPKE-mode-base[MLKEM768-X25519] to recipient device's REAL high-entropy keypair | Reuses Layer-C primitive at the wire level. Recipient keypair is generated with random(32) on device, NOT derived from password. |

**This is the §2.2 design that the F-full review §§2–8 already proposed.** F+ §14.1 was the deviation; reject it; the rest of the F-full design holds.

### 6.2 The architectural elegance Benten still gets

The "unified envelope" shape proposed in §14.1's code sketch is the right shape — at the envelope layer, not the primitive layer:

```rust
pub struct EncryptedEnvelope {
    codepoint: u16,                       // distinguishes Layer-A vault / Layer-C drop / Layer-D wrap
    payload: EnvelopePayload,             // codepoint-discriminated payload shape
    aad_binding: BindingContext,
}

pub enum EnvelopePayload {
    /// Layer-A vault: symmetric AEAD under DAK
    SymmetricAead { ciphertext: Bytes, nonce: [u8; 12] },
    /// Layer-C drop / Layer-D wrap: HPKE-mode-base[MLKEM768-X25519]
    HpkeBase { enc: Bytes, ciphertext: Bytes },
}

pub enum BindingContext {
    Vault { vault_version: u8 },
    DropToRecipient { audience_did: Did },
    DeviceLink { provisioning_session_id: [u8; 16] },
    RemotePermission { request_id: [u8; 16], operation: PermissionOperation },
}
```

This keeps the F+ unified-envelope spirit but uses the correct primitive per codepoint. Future-additive codepoints slot in trivially. This IS the design Benten's CLAUDE.md baked-in #5 crypto-agility pattern points at: codepoint-dispatch at the framing, real-best-primitive per codepoint.

### 6.3 What to record in `.addl/phase-4-meta/v1-FROZEN-INTERFACE-DEFERRED.md`

Per the discipline of recording considered-and-rejected designs (`feedback_extra_reflection_pass_for_elegant_permanent_shape` requires alternatives be NAMED-deferred not silently dropped):

- **Considered Option F+ pseudo-keypair-derived-from-DAK for Layer-A vault.** Rejected for v1-beta on (a) ML-KEM-768 KeyGen-from-secret-seed side-channel hazard surface (Arriaga et al. "Tempo" attack class; no constant-time SampleNTT in upstream RustCrypto ml-kem), (b) zero functional benefit at Layer-A (vault use case is encrypt-to-self; symmetric AEAD is the right-shape primitive), (c) architectural elegance superficial (Layer-B is structurally symmetric; the unification is 3-of-4 not 4-of-4; security shapes differ across A vs C vs D). **Revisit trigger:** if/when upstream RustCrypto `ml-kem` ships verified-constant-time SampleNTT + an independent published proof of "HPKE-mode-base[X-Wing] with adversarially-influenced recipient seed via chosen-password is IND-CCA2-secure" lands, F+ becomes architecturally viable. Neither condition is on the v1-beta critical path. Symmetric-AEAD-under-DAK is strictly stronger for v1-beta.

---

## 7. Risks + mitigations (if Ben overrides this NO-GO and pursues F+ anyway)

The §4 attack class is the load-bearing concern. If F+ is pursued despite this recommendation, the following are non-negotiable:

| Risk | Severity | Mitigation |
|---|---|---|
| ML-KEM-768 KeyGen-from-secret-seed timing side-channel enables online dictionary attack bypassing Argon2id work-factor | HIGH | Vendor a constant-time SampleNTT implementation (violates baked-in #5 "never fork primitives"); OR wait for upstream RustCrypto ml-kem to ship CT SampleNTT (currently unscheduled); OR limit the threat model to "no co-resident-adversary on unlock device" and document this explicitly in `SECURITY-POSTURE.md` as a v1-beta scoped-limitation Compromise. |
| HPKE-RFC-9180 + X-Wing IND-CCA2 proofs assume recipient keypair is generated with uniform randomness; chosen-password adversarial-seed-influence is outside standard adversary model | MEDIUM | Commission a custom security proof (~$50k+ cryptographer-time / 3-6 month wall-clock; would gate v1-beta tag). The hybrid-construction-with-classical-floor (X25519 half is still real) provides residual security even if ML-KEM half is broken, but the proof for "PKE secure when seed is chosen-password-derived" doesn't currently exist. |
| Multi-implementation interop breaks if Benten's Layer-A vault codepoint becomes mistaken for a Layer-C drop codepoint by future readers/writers | LOW-MED | Ensure Layer-A vault uses a clearly distinct codepoint (not `0x647a` or the encrypt-to-recipient `0x6500` from §5.2 of prior cryptographer review). Document in `SECURITY-POSTURE.md` the codepoint table with each one's "intended use" clearly stated. |
| Audit cost increase | MED | Budget ~+0.5 to +1 person-week of external cryptographer time over the AEAD-under-DAK baseline to cover the pseudo-keypair construction's novel surface. |
| Performance: ML-KEM-768 KeyGen ~5-10ms per call vs AEAD encrypt-decrypt ~0.01ms; vault unlock + per-pivot would pay this cost | LOW | Performance is unlock-time only; not a hot-path. Argon2id at OWASP params already dominates (~100-300ms); ML-KEM-768 keygen is small additional overhead. Not a real concern. |

**The audit-cost increase + the side-channel proof gap are the two reasons external auditors will probably flag F+ as a finding. If F+ ships with no mitigations, the external audit per CLAUDE.md baked-in #15 NF-2 / C-GM-AUDIT is more likely to surface BLOCKER-class findings than the AEAD-under-DAK design is.**

---

## 8. Honest disagreement

### 8.1 The F+ pitch's appeal IS real — but the elegance is at the wrong layer

I want to give credit where it's due: §14.1's instinct to unify is good cryptographic taste. The crypto-agility pattern (CLAUDE.md baked-in #5) DOES point toward codepoint-dispatched envelopes with consistent shapes. The mistake is unifying at the PRIMITIVE layer when the right place to unify is the ENVELOPE layer (per §6.2 above).

I disagree with the F+ pitch's *conclusion* but agree with its *motivation*. The §6.2 design captures the motivation without inheriting the §4 hazard surface.

### 8.2 I disagree with one piece of the F-full review's framing

The F-full review's §14.1 says: *"The 'pseudo-keypair derived from a symmetric secret' is an unconventional pattern; needs cryptographer review (probably sound — HPKE's Encap/Decap interface admits deterministic-derived pubkeys — but warrants verification against the X-Wing draft's deterministic-derivation appendix)."*

This framing implicitly suggests the soundness question is "does HPKE admit deterministic-derived pubkeys?" (answer: yes, per RFC 9180 §7.1.3) and concludes "probably sound." This misses the actual concern. The DeriveKeyPair API admitting a deterministic seed is NOT the same as "HPKE-mode-base[X-Wing] is secure when the recipient's seed is chosen-password-derived." The first is a syntactic admissibility question; the second is a semantic security question, and it's the one that matters.

This is a process disagreement, not a substantive one — the F-full reviewer correctly punted to cryptographer review (this review), which is exactly the right discipline.

### 8.3 I disagree gently with the orchestrator's brief framing of "pseudo-keypair pattern be sound + worth-it"

The brief frames the question as a single binary: "is it sound AND worth-it." But these are two independent questions:

- **Is it sound?** Yes, in the IND-CCA2-reduction sense under standard assumptions and uniform seed. NO when you account for the side-channel surface in §4 against a low-entropy seed. This is a CONDITIONAL answer, not a binary.
- **Is it worth-it?** Independent of soundness, the architectural utility argument fails on its own merits (per §5). Even if the side-channel concern in §4 were fully mitigated, F+ wouldn't be worth pursuing.

Recommending against F+ does NOT require both legs to fail. EITHER leg suffices. **§4 (side-channel) is the load-bearing one; §5 (architectural utility) is the independent one.** Both reach NO-GO.

### 8.4 What I would tell Ben in plain English

"The pitch is conceptually elegant but operationally wrong. The vault use case is encrypt-to-self under a password; the natural cryptographic primitive for that is symmetric AEAD (AES-GCM or ChaCha20-Poly1305) under a password-derived key. Every production vault system since 1Password v1 has converged on this pattern; there is no 'better hidden insight' the F+ proposal is exploiting. The 'one primitive across all layers' framing is appealing as architectural rhetoric but it forces an asymmetric tool onto a symmetric problem, and in doing so it inherits a side-channel surface (ML-KEM keygen-from-secret-seed) that the symmetric tool doesn't have. The right unification is at the envelope-format layer — same outer shape, codepoint-dispatched primitive — and the F+ proposal's code sketch is almost right; it just needs to dispatch to AEAD for the vault codepoint instead of HPKE."

---

## 9. Evidence base

### 9.1 Standards + drafts

- **RFC 9180** Hybrid Public Key Encryption ([datatracker.ietf.org/doc/html/rfc9180](https://datatracker.ietf.org/doc/html/rfc9180)). §7.1.3 DeriveKeyPair. §9.5 PSK security (low-entropy warning). §9.7 KDF security.
- **RFC 9106** Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work ([datatracker.ietf.org/doc/html/rfc9106](https://datatracker.ietf.org/doc/html/rfc9106)).
- **RFC 8439** ChaCha20 and Poly1305 ([datatracker.ietf.org/doc/html/rfc8439](https://datatracker.ietf.org/doc/html/rfc8439)).
- **FIPS 203** Module-Lattice-Based Key-Encapsulation Mechanism Standard (NIST, August 2024). §6.1 ML-KEM KeyGen. §8 Security analysis.
- **draft-connolly-cfrg-xwing-kem-10** X-Wing: General-Purpose Hybrid Post-Quantum KEM ([datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10](https://datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-10)). §5.1-5.4 KeyGen / GenerateKeyPairDerand.
- **draft-ietf-hpke-pq-04** Post-Quantum Hybrid Public Key Encryption ([datatracker.ietf.org/doc/draft-ietf-hpke-pq/](https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/)).
- **draft-irtf-cfrg-concrete-hybrid-kems-03** ([datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/](https://datatracker.ietf.org/doc/draft-irtf-cfrg-concrete-hybrid-kems/)). §4.2 MLKEM768-X25519 identical to X-Wing.

### 9.2 Academic papers

- **Barbosa, Connolly, Diniz, Kahl, Krämer.** "X-Wing: The Hybrid KEM You've Been Looking For." IACR Communications in Cryptology, Vol. 1, No. 1, 2024-04-09. ([cic.iacr.org](https://cic.iacr.org/p/1/1/21))
- **Arriaga, Barbosa, Boyen.** "Tempo: ML-KEM to PAKE Compiler Resilient to Timing Attacks." IACR ePrint 2025/1399 ([eprint.iacr.org/2025/1399](https://eprint.iacr.org/2025/1399)). The load-bearing reference for §4 (ML-KEM rejection-sampling timing attack on secret seed).
- **Bernstein, Persichetti.** "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems." ACNS 2024 ([eprint.iacr.org/2024/2051](https://eprint.iacr.org/2024/2051)). Reference for §4.4 (ML-KEM Decap-side surface, independent of pseudo-keypair pattern).
- **Bellare, Pointcheval, Rogaway et al.** OPAQUE: An Asymmetric PAKE Protocol Secure Against Pre-Computation Attacks. EUROCRYPT 2018 / IACR ePrint 2018/163 ([eprint.iacr.org/2018/163](https://eprint.iacr.org/2018/163)).
- **Hofheinz, Hövelmanns, Kiltz.** "A Modular Analysis of the Fujisaki-Okamoto Transformation." TCC 2017. Reference for IND-CCA2 reduction techniques used by ML-KEM.
- **PQShield.** "Formally verifying AVX2 rejection sampling for ML-KEM" ([pqshield.com](https://pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/)). Reference for the formal-verification effort on ML-KEM's rejection sampling — establishes that even the experts treat this as a non-trivial side-channel surface deserving formal verification.

### 9.3 Production-system whitepapers + designs

- **Filippo Valsorda.** "age and Authenticated Encryption" ([words.filippo.io/age-authentication](https://words.filippo.io/age-authentication/)). Plus age-discussion #463 on scrypt-recipient vs x25519-recipient design distinction ([github.com/FiloSottile/age/discussions/463](https://github.com/FiloSottile/age/discussions/463)).
- **Bitwarden Security Whitepaper** ([bitwarden.com/help/bitwarden-security-white-paper/](https://bitwarden.com/help/bitwarden-security-white-paper/)). PBKDF2-SHA256 600k iter → Master Key → HKDF stretch → Stretched Master Key → AES-256-CBC + HMAC-SHA256.
- **1Password Security Design Whitepaper** ([1passwordstatic.com/files/security/1password-white-paper.pdf](https://1passwordstatic.com/files/security/1password-white-paper.pdf)). Two-secret KDF → AUK → AES-256-GCM. Asymmetric private keys stored as encrypted blobs under AUK ([agilebits.github.io/security-design/deepKeys.html](https://agilebits.github.io/security-design/deepKeys.html)).
- **Molly (Signal-Android fork) at-rest encryption** ([github.com/mollyim/mollyim-android/wiki/Data-Encryption-At-Rest](https://github.com/mollyim/mollyim-android/wiki/Data-Encryption-At-Rest)). Argon2id + AES-256-CBC + HMAC-SHA256.
- **IOTA Stronghold** ([github.com/iotaledger/stronghold.rs](https://github.com/iotaledger/stronghold.rs)). Argon2id + XChaCha20-Poly1305 snapshot encryption.
- **Cryptography Caffè (SandboxAQ): Protecting Signal Keys on Desktop** ([cryptographycaffe.sandboxaq.com/posts/protecting-signal-desktop-keys/](https://cryptographycaffe.sandboxaq.com/posts/protecting-signal-desktop-keys/)). Reference for Signal-desktop limitations and the recommended pattern.

### 9.4 Implementation references

- **RustCrypto `argon2` 0.5.x** ([docs.rs/argon2](https://docs.rs/argon2)).
- **RustCrypto `chacha20poly1305` 0.10.x** ([docs.rs/chacha20poly1305](https://docs.rs/chacha20poly1305)).
- **RustCrypto `hkdf` 0.12.x** ([docs.rs/hkdf](https://docs.rs/hkdf)).
- **RustCrypto `ml-kem`** ([github.com/RustCrypto/KEMs](https://github.com/RustCrypto/KEMs)). Verify against RustSec advisories; pin version.
- **RustCrypto `x25519-dalek`** ([docs.rs/x25519-dalek](https://docs.rs/x25519-dalek)). Documents constant-time scalar mult.
- **Brendan McMillion `hpke` crate** ([github.com/rozbb/rust-hpke](https://github.com/rozbb/rust-hpke)). Reference HPKE Rust impl.
- **Verification Theatre (Nadim Kobeissi) on `hpke-rs`** ([symbolic.software/blog/2026-02-05-cryspen/](https://symbolic.software/blog/2026-02-05-cryspen/)). Reference for "don't use hpke-rs."

### 9.5 Prior Benten reviews

- Cryptographer review of encrypt-to-recipient: `origin/phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17`.
- F-full scope review: `origin/phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`.
- Benten CLAUDE.md baked-in #5 (crypto-agility); #15 (v1-beta gate); #18 (authority-isolation vs confidentiality-isolation).
- Benten `docs/SECURITY-POSTURE.md` Compromise #31 + `docs/INVARIANT-COVERAGE.md` Inv-15.

---

## 10. Self-assessment

### Confidence level

- **HIGH confidence** on:
  - The recommendation: NO-GO on F+ for Layer-A; pursue Option B-equivalent. Multiple independent lines of evidence converge on the same conclusion (real-world precedent, formal-model concern, architectural utility, audit-cost arithmetic).
  - §3 real-world precedent table — verified against primary sources for each row.
  - §6.2 envelope-layer unification as the correct alternative — this is the CLAUDE.md baked-in #5 crypto-agility pattern operating as designed.
  - The §4 attack vector EXISTING in the abstract. ML-KEM keygen rejection-sampling timing variation under secret seed is established literature (Tempo paper).

- **MEDIUM-HIGH confidence** on:
  - The §4 attack vector being PRACTICAL against a specific real-world adversary model on Benten's deployment shapes. The Tempo paper's attack is theoretically constructed; I don't have empirical end-to-end exploit numbers for "co-resident-VM adversary cache-times ML-KEM keygen during Benten vault unlock and recovers password in N attempts" specifically. The attack class is real; the exact attack instance against a Benten Tauri-shell user with no co-resident-VM threat may be unreachable. Conservative interpretation: assume it's reachable for at-least-some users; deserves blocker-class concern.
  - The IND-CCA2 reduction analysis in §2 — RFC 9180's modular composition argument holds, but I'm not aware of a published proof that specifically covers "HPKE-mode-base[X-Wing] with adversarially-chosen recipient-seed-via-password" as an adversary model. That proof gap is real and would be a finding in external audit.

- **MEDIUM confidence** on:
  - The exact magnitude of the audit-cost delta (§7). I estimated "+0.5 to +1 person-week" but audit firms scope differently.
  - The §5 architectural-utility analysis. This is partly subjective taste; reasonable cryptographers might disagree on whether the F+ pattern is "elegant superficial" or "elegant in a useful way." I think the Layer-B-must-stay-AEAD point is decisive; others might weight it differently.

- **LOWER confidence** on:
  - The X25519-half-of-X-Wing keygen being fully constant-time on all platforms. I cited `x25519-dalek` as documenting constant-time scalar mult, but the actual constant-time-ness depends on the specific impl + target architecture. If a future Benten deployment uses a different X25519 impl, this assumption could fail.
  - The specific recommendation that "Layer-A AEAD codepoint should be distinct from Layer-C codepoint" — this is a reasonable design discipline but the specific codepoint values are out-of-scope for this review.

### What additional review I would want

1. **Empirical timing measurement** of RustCrypto `ml-kem` v0.x KeyGen across 1000+ runs with varied seeds, to quantify the actual leakage rate on Benten's target platforms (macOS arm64; Linux x86_64; Windows x86_64; wasm32-unknown-unknown via JS engine). If leakage is <1 bit per measurement, the practical attack may be unreachable; if leakage is >4 bits per measurement, blocker.
2. **A formal proof of "HPKE-mode-base[X-Wing] IND-CCA2 with adversarially-chosen recipient seed"** OR a published theorem reduction that maps this to a known-secure model. Both don't currently exist in the literature.
3. **A second-opinion cryptographer review** specifically of §6.2's envelope-layer unification design — I'm confident it's correct but the codepoint-dispatch shape deserves independent eyes.
4. **External audit firm input** on whether the F+ pattern would be flagged in their normal Layer-1 review. (My estimate: YES, with high confidence.)

### What this review does NOT cover

- The full Phase-4-Meta-Core F-full scope (covered by `e2r-ffull-scope-review.md`).
- The specific RustCrypto `ml-kem` implementation version recommendation.
- The Layer-B per-Node AEAD design (out of F+ scope; design unchanged regardless of F+ outcome).
- The §3.5s cross-ecosystem-identifier-as-content discipline that the F+ pitch invokes — that discipline applies but is orthogonal to the primitive-choice question.
- The X-Wing-mislabel corrective from the prior cryptographer review (~24 LOC) — that lands independently per §15.1 of the F-full review.
- Audit-firm RFP / selection.

### Self-critique

I've recommended against the F+ pattern strongly. A more cautious reviewer might say "the IND-CCA2 reduction holds; the side-channel concern is theoretical; ship F+ and revisit if the audit firm flags it." I considered this and rejected it because:

(a) The Tempo paper exists specifically to address this attack class; it's not "theoretical" in the sense of "no one has thought about it" — it's "established attack class with a published mitigation that Benten can't apply without violating baked-in #5."

(b) The audit firm's flagging it post-hoc would cost more than designing-it-right pre-hoc. The point of doing this review at extra-reflection-pass time is precisely to catch hazards before they're baked-in.

(c) The architectural utility argument in §5 stands independent of §4. Even if §4 is overcautious, F+ wouldn't be worth pursuing on the basis of §5 alone.

If Ben weighs the elegance argument more strongly than I do, he might still pursue F+ with full audit-finding tolerance. That's his call; my recommendation is to take the safer, ecosystem-standard, less-novel design. The "boring crypto is the best crypto" stance is well-served by the AEAD-under-DAK choice.
