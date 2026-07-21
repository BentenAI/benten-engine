# Benten Identity/Key Recovery — Neutral Reference for Reviewers

**Purpose.** This is the shared stimulus for the recovery-protocol review. It assembles 8 option-proposals into one comparison. It does **not** recommend or rank — every rating is a faithful compression of what each proposal argues about itself, including its own stated deal-breakers. Reviewers decide.

**The decision.** What (if anything) fills the pluggable `RecoveryHook` seam as the v1 default, given a user who may lose ALL devices in a system with no central authority to reset them. Every option is measured against the frozen v1-beta substrate (did:benten hybrid-PQ multikey + KEM CID, UCAN chains, RotationLog, device attestation, sealed-sender Drops, per-principal encryption, TOFU/no-PKI) and must wire additively.

**Rating legend.**
- Axes where more is better (self-sovereignty, coercion/duress, lost-all recoverability, decentralization, UX, precedent): **Strong · Moderate · Weak · Fails**.
- Implementation cost is a magnitude, not a quality: **Minimal · Low · Moderate · High** (lower = cheaper to build on the frozen substrate).
- Precedent maturity: **Mature · Moderate · Emerging**.

---

## (a) Comparison Matrix

| Option | Self-sovereignty | Coercion / duress resistance | Lost-ALL-devices recoverability | Decentralization / no-custodian | UX (non-technical) | Impl cost on frozen substrate | Precedent maturity |
|---|---|---|---|---|---|---|---|
| **1. Shamir threshold (SSS)** | Strong — no custodian; but K-collusion can silently seize identity | Weak — no duress signal/decoy; distribution only raises cost | Strong — best-in-class; K shares rebuild, no central party | Strong — fully custodian-free | Weak — nominating/maintaining K-of-N holders is the hardest step; silent share rot | Moderate — additive via RotationLog; needs SLIP-39 + VSS commitments + refresh + guardian UX | Mature — SLIP-39/Trezor, Vault, SSKR, Glacier |
| **2. Social recovery (UCAN guardians)** | Strong — guardians hold no secret; but ≥K collusion rotates to attacker | Moderate — forces coercing K distinct humans who can stall/refuse; degrades vs targeted adversary; needs guardian-set privacy | Strong — the case it's built for (**authority only**) | Strong — *if* no default custodial guardian (Argent lesson) | Weak — choose + keep-alive guardian set; recovery needs K live willing humans; guardian social-engineering | Moderate — one real additive change (2nd RotationLog rotation-auth variant); rest reuses UCAN/sealed-sender | Mature — Argent 500k+, ERC-4337 1M+, Safe/Loopring |
| **3. Hardware escrow / secure element** | Moderate — self-custodied, but single object + vendor/supply-chain trust | Fails — one token + one PIN, both coercible from one person; duress-PIN weak | Moderate — succeeds *iff token survived*; correlated loss & single-token fragility fatal | Strong — no custodial fallback (also the availability risk) | Moderate — familiar "backup key in a safe"; opaque to non-technical; forgotten PIN = permanent brick | High — cross-platform KekSealer glue; no SE signs ML-DSA (escrow-wrap keeps PQ in host RAM) | Mature — Google Advanced Protection, iCloud HSM, PIV/CAC, BitLocker+TPM (custodial variants) |
| **4. MLS / CGKA device groups** | Strong — device-only, no third party | Fails — coerced user unlocks any live member = full authority | Fails — group state dies with devices; external-join needs a credential that died too | Strong — untrusted DS; but needs an Authentication Service Benten lacks (TOFU) | Strong for multi-device continuity ("approve new device"); single-device user has none; epoch-desync lockouts | High — TreeKEM/epochs/external-commit/fork-defense; largely re-expresses existing device attestation | Moderate — RFC 9420/Webex/OpenMLS mature, but every shipped device-group added a *separate* recovery secret |
| **5. Passkey / FIDO2 sync** | Weak — lost-all reduces to platform (Apple/Google/PM) account recovery | Fails — no decoy; compelled biometric; weaker legal protection than a passphrase | Moderate — recoverable, but via custodian-controlled account-recovery ceremony | Fails — platform is de-facto custodian | Strong — near-frictionless biometric tap; ecosystem lock-in; PRF fragmentation | Low — additive (Design A: PRF-wrapped seed = ordinary sealed Drop); Design B adds WebAuthn-assertion verification to RotationLog | Mature — Bitwarden/1Password PRF, iCloud/Google sync at scale; CXP portability emerging |
| **6. Threshold signatures (FROST / BLS)** | Strong — group secret never reconstructed; but ≥K collusion seizes | Moderate — torture compels participation, not extraction; remote holders can decline; marginally > Shamir | Moderate — only if some shares live off-device (i.e., re-imports social guardians) | Strong — DKG needs no dealer; coordinator sees nothing | Weak — enroll N, DKG ceremony, guardian signer app; FROST 2-round liveness (BLS async is easier) | High — FROST/Ed25519 verify is *zero-wire-change*, **but threshold ML-DSA doesn't exist → classical-only weak link**; DKG/coordinator/refresh; impl fragility | Moderate — RFC 9591 young; drand/DFINITY BLS institutional; MPC-custody bugs (BitForge, TSSHOCK) |
| **7. Seed-phrase / offline paper** | Strong (maximal) — needs nothing and no one; survives Benten's own death | Fails (worst-in-class) — $5-wrench; bearer credential; decoy-passphrase is theater | Strong — the design case, *if the phrase survives* | Strong (maximal) — zero external trust | Weak — 24 words; loss/transcription/phishing/inheritance; permanent-loss modes | Low (cheapest) — client-side `derive_from_seed` + genesis flag; deterministic keygen native to FIPS 203/204 | Mature (universal) — BIP39/32/44, hardware wallets; ~20% BTC lost; Ledger Recover backlash |
| **8. No recovery / accept-loss** | Strong (maximal) — identity IS the key; no new surface | Strong (best) — nothing extra to coerce; user can truthfully say "I have no recovery" | Fails (by design) — permanent loss is the model | Strong (maximal) — nothing held anywhere | Weak — no phrase/guardians, but comprehension gap; re-onboarding reopens TOFU (#67); correlated device loss | Minimal — `NoRecoveryHook` null object; wires nothing | Mature — Session, Signal pre-2020 (later added PIN+SVR), Nostr, GPG |

---

## (b) Per-Option Summaries

### 1. Shamir Threshold (SSS)
**Mechanics.** Generate a 256-bit Recovery Root Secret (RRS), derive a recovery keypair mirroring the substrate, publish a signed RecoveryEnrollment record, then SLIP-39-split the RRS into N shares (guardian-sealed / mnemonic / spare device) and zeroize it. Recovery: gather K shares → Lagrange-interpolate → re-derive → the recovery key signs a RotationLog entry handing root to a fresh device.
**Biggest strength.** Information-theoretic secrecy below threshold and best-in-class lost-all availability with no custodian; wires almost entirely additively (RotationLog is the only integration point).
**Biggest weakness.** No coercion/duress resistance at all (no duress signal, decoy, or rate-limit), and plain SSS is unverifiable (malicious-dealer risk) — the proposal itself says bare Shamir is disqualified as a sole default and must add VSS commitments + proactive refresh + a duress factor.
**Confidentiality note.** Recovers a self-derived recovery identity; does not by itself restore old data-keys unless the KEM seed is escrowed in the RRS.
**Key precedent.** Trezor SLIP-39 Shamir Backup; HashiCorp Vault unseal (cautionary — drifted toward auto-unseal custodians).

### 2. Social Recovery (UCAN-native guardians)
**Mechanics.** User commits a signed GuardianSet `{guardian_dids[], K, policy}` to the RotationLog; guardians are DIDs that hold no secret. Recovery: fresh device mints K_new, K guardians each verify the human out-of-band and sign a rotation attestation; K attestations aggregate into a GuardianAuthorizedRotation entry that (after a timelock) makes K_new authoritative for the unchanged DID.
**Biggest strength.** Cleanest UCAN/RotationLog-native fit for *authority* recovery; strongest against remote/scalable attackers (raises the bar from 1 victim to K humans who can stall or refuse).
**Biggest weakness.** Rotation recovers authority, **not** decryption of past ciphertext — and the equivocation problem means a ≥K-collusion rotation is cryptographically indistinguishable from a legitimate one (defended only socially/temporally). PQ-threshold aggregation is immature, forcing N concatenated sigs or a classical downgrade.
**Key precedent.** Argent (500k+; instructive failure = default custodial guardian re-centralized recovery); Vitalik's social-recovery essay → ERC-4337.

### 3. Hardware Escrow / Secure Element
**Mechanics.** Derive a KEK bound to a secure element (YubiKey PIV/FIDO2, Apple SEP, TPM), wrap the Recovery Root Secret (incl. an escrowed KEM seed) into a Recovery Bundle stored redundantly as public ciphertext, and commit a recovery authorization to the RotationLog. Recovery: fetch ciphertext, tap token + PIN (hardware anti-hammering) → unwrap → append a signed rotation.
**Biggest strength.** Genuine two-factor (possession + PIN) with hardware-enforced anti-hammering; excellent as an availability-redundancy layer (inert ciphertext safe on peers).
**Biggest weakness.** Coercion-catastrophic — one object + one PIN extractable from one person, with no threshold/quorum; plus no 2026 SE signs ML-DSA (PQ secret transits host RAM), and a lost/forgotten-PIN token means irreversible lockout with no custodian.
**Confidentiality note.** Recovering old Drops forces escrowing the actual KEM seed — a permanent harvest-now-decrypt-later surface (the PIV sign-vs-encrypt split).
**Key precedent.** Google Advanced Protection offline backup key; Ledger Recover (2023) backlash; iCloud Keychain HSM escrow (custodial).

### 4. MLS / CGKA Device Groups
**Mechanics.** A user's devices form an MLS group (TreeKEM); the epoch secret wraps the root authority payload, so every device continuously co-holds the ability to act. did:benten device-DIDs serve as MLS credentials. Adding/removing a device is an Add/Remove + Commit; a surviving device heals via post-compromise security and appends RotationLog revocations.
**Biggest strength.** Best-in-class *device continuity* and post-compromise healing — atomic signed membership, thief eviction via PCS, no external guardians to betray.
**Biggest weakness.** Does not solve the stated problem: with zero surviving devices the group secret is gone and external-join needs a credential that died too. Zero coercion resistance, and covering lost-all by adding a friend as a member over-privileges them dangerously.
**Key precedent.** RFC 9420 / Cisco Webex / OpenMLS; Apple iCloud Keychain syncing circle — which *still* bolted on separate HSM escrow (the unanimous meta-lesson: device groups always add a separate cold-recovery secret).

### 5. Passkey / FIDO2 Synced Credentials
**Mechanics.** Use the WebAuthn PRF extension (CTAP2 `hmac-secret`) to derive a stable wrap key gated by biometric/PIN. Design A: PRF-wrap the seed into a sealed RecoveryDrop stored in the user's Atrium; recovery re-provisions the synced passkey on a fresh device, PRF re-derives the wrap key, decrypts the seed. (Design B: passkey public key pre-authorizes a RotationLog rotation.)
**Biggest strength.** Near-frictionless biometric UX and a production-real primitive (PRF-wrapped vaults ship in Bitwarden/1Password); an excellent *one-of-M* share.
**Biggest weakness.** Custodial by construction — lost-all reduces to Apple/Google/PM account recovery, reintroducing a mandatory custodian and violating the no-custodian constraint; zero coercion resistance (compelled biometric); ecosystem lock-in and PRF fragmentation.
**Key precedent.** iCloud Keychain / Google Password Manager passkey sync at scale; SIM-swap account-recovery takeovers ($72M+ cohort) as the dominant real-world failure.

### 6. Threshold Signatures (FROST / Threshold-BLS)
**Mechanics.** N key-shares held across devices/guardians; any K jointly produce one signature verifying against a single group public key, with the group secret **never reconstructed**. Register the group key in the RotationLog as an authorized rotation principal; on loss, K holders co-sign a rotation to a fresh did:benten key.
**Biggest strength.** FROST(Ed25519) aggregates are bit-identical ordinary Ed25519 signatures — the cleanest possible additive *verification* fit (zero wire change) — with strong no-secret-exists, thief-safe, malicious-minority properties.
**Biggest weakness.** The PQ wall: threshold ML-DSA-65 is not production-ready, so a threshold recovery-authority is realistically classical-only (a CRQC-forgeable weak link in a PQ system); alone it only rescues "lost-some" (lost-all still needs off-device social holders), and MPC implementations have leaked shares (BitForge/TSSHOCK).
**Confidentiality note.** Recovers control, not ML-KEM-sealed data, absent a separate threshold-decryption/re-wrap scheme.
**Key precedent.** RFC 9591 FROST (Zcash crate, Serai); drand / DFINITY threshold-BLS (institutional).

### 7. Seed-Phrase / Offline Paper (or Metal)
**Mechanics.** Make the identity seed-rooted: draw a 256-bit master seed, encode as a BIP39 24-word phrase, and HKDF-derive every substrate key (Ed25519, ML-DSA-65 ξ, X25519, ML-KEM-768 d‖z, DAK root) via deterministic keygen — native to FIPS 203/204. Recovery: re-enter words → re-derive identical keys, DID, KEM, and DAK → replay public RotationLog/UCAN state from peers.
**Biggest strength.** Maximal self-sovereignty and the cheapest possible hook — no network protocol, no guardians, no custodian; the only option that still works if the Benten project disappears, and a cleaner PQ fit than secp256k1 wallets (back up 32 bytes, re-expand).
**Biggest weakness.** Worst-in-class coercion resistance — a memorizable/physical bearer secret producible on demand; recovery is cryptographically indistinguishable from theft (re-derives the same root), and a single photograph is a permanent silent takeover of all past and future data.
**Key precedent.** BIP32/39/44 HD wallets (securing hundreds of billions); Ledger Recover backlash as Benten's exact tension in miniature.

### 8. No Recovery / Accept-Loss (Honest Baseline)
**Mechanics.** The identity *is* the key material; when it's gone, the identity is gone. `RecoveryHook` = a `NoRecoveryHook` null object. Optimize for prevention (force redundant attested devices; treat single-device as unhealthy) and graceful re-onboarding (a fresh DID + social TOFU re-verification). ≥1 surviving device = continuity via RotationLog; 0 devices = permanent loss.
**Biggest strength.** Security-maximal and attack-surface-minimal — the strongest duress story (no seed, guardian, or enclave to coerce; user can truthfully claim no recovery) and zero substrate change; every other option must beat this to justify the custody/coercion surface it adds.
**Biggest weakness.** Catastrophic, irreversible, correlated failure with no appeal — and for a *lifetime personal-data graph* (not chat history), "you lost a decade of your data and by design no one can help" is a different magnitude of harm; also loses revocation on key-death and reopens TOFU (#67) en masse at re-onboarding.
**Key precedent.** Session ("not even we can help"); Signal pre-2020 (found pure accept-loss too user-hostile and added PIN + SVR); Nostr (NIP-41 rotation efforts).

---

## Cross-Option Themes (reported, not judged)

Several proposals independently surface the same structural facts; reviewers should hold these in mind since they cut across the whole matrix:

1. **Authority recovery ≠ confidentiality recovery.** Rotating to a new key (social, threshold, hardware, MLS) restores the ability to *act* as the DID but grants **nothing** encrypted to the old KEM key. Recovering old sealed Drops requires separately escrowing/threshold-decrypting the KEM secret — which re-imports the exact coercion/collusion/HNDL surface that authority-only rotation avoided. Named explicitly by proposals 2, 3, 6; implicit in 1.
2. **The K-collusion floor is irreducible.** Every threshold/guardian scheme (1, 2, 6) has the same property: below K learns nothing; at ≥K the holders can not just recover but *seize and rotate you out*, indistinguishably from a legitimate recovery in a no-consensus system.
3. **PQ-threshold immaturity is a shared constraint.** Proposals 2 and 6 both flag that threshold ML-DSA-65 is research-stage, forcing either N concatenated signatures or a classical-only downgrade on the highest-value path.
4. **Device-continuity is not cold recovery.** Proposals 4 and 8 solve "≥1 device survives" well but do not answer lost-all; multiple precedents (Apple, Keybase, Matrix, Signal) always paired a device ring with a *separate* recovery secret.
5. **The pluggable-hook / hybrid framing recurs.** Nearly every proposal positions itself as one factor behind `RecoveryHook` (a share of an M-of-N, a second factor, a floor/null object) rather than a complete sole answer — consistent with the anticipated seam and the stated irreducible tension between perfect self-sovereignty and perfect recoverability.
6. **Human/operational failure dominates cryptographic failure.** Across proposals the recurring real-world loss event is social/operational (silent share rot, guardian unavailability, forgotten PIN/passphrase, phishing, correlated device loss), not cryptanalysis.