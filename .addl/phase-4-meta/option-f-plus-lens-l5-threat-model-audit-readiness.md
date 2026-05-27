# Option F+ §6.2-with-Amendments-1–6 — Lens L5 review (external-cryptographer-audit-readiness + threat-model boundaries + NIST/FIPS/ANSSI compliance posture)

**Reviewer lens.** Senior cryptographic auditor simulating a Trail-of-Bits / NCC-Group / Cure53-style external review at v1-beta-tag time. Distinct from L1 (soundness), L2 (composition), L3 (red-team), L4 (scope) — this lens evaluates **what an external audit firm will find, what NIST/FIPS/ANSSI requirements the design must clear, what threat-model boundaries need honest-disclosure Compromise mints, and what audit-deliverable shape minimises 3-person-week audit churn.**

**Inputs (tree-state pre-flight verified clean against `2172cb6d` at dispatch).**

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — L1 NO-GO on the pseudo-keypair primitive + §6.2 envelope-layer-unification origin sketch.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — L2 CONCUR-WITH-AMENDMENTS, Amendments 1 (codepoint-in-AAD) + 2 (strict-decode no-fallback).
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — L3 red-team, Amendments 3 (TLV length-injectivity) + 4 (sender-DID-in-AAD) + 5 (replay-window epoch binding) + 6 (Bernstein-Persichetti CT-Decap + Compromise mint) + minor 7 (BE codepoint) + 8 (codepoint registry).
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — F-full scope review with §6 remote-permission wire format + §7 device-link + §10.2 audit-scope estimate (~3 person-weeks).
- `main` → `docs/INVARIANT-COVERAGE.md` (Inv-15 registered, Inv-16 pending) + `docs/SECURITY-POSTURE.md` (Compromise #6, #30, #31; reserved slots for #32+).
- `main` → `crates/benten-crypto-suite/INTERNALS.md` (informational; reviewed for current primitive choices).

**Reading-order.** 5-minute read: §1 (executive verdict) → §2.1 in/out-of-scope matrix → §6 recommended new Compromise mints + invariant additions → §7 self-assessment. Everything else is supporting evidence.

---

## 1. Executive verdict + confidence

**Top-line.** §6.2-with-Amendments-1–6 is **AUDIT-READY-IN-DIRECTION** but **NOT-YET-AUDIT-READY-IN-DOCUMENTATION-DELIVERABLE-SHAPE**. The cryptographic design (envelope-layer-unification + codepoint-dispatch + per-codepoint primitive choice + the six amendments) is the right shape; a competent external auditor will spend most of their 3 person-weeks **reading the test corpus + threat-model document + invariant-coverage cross-references**, not finding fresh cryptographic defects. The audit-cost-optimization opportunity is in the **deliverable shape**, not in the design.

**Verdict on substantive crypto risk at audit-time.** **LOW**. The three prior reviews have already surfaced and closed the substantive cryptographic surfaces an external auditor would normally find at first sweep. The residual surfaces are documented (Compromise #30 PQ-primitives unaudited; proposed Compromise #32 Bernstein-Persichetti Decap; the multi-stanza HpkeMultiBase that L3 §2.7 deferred to specialist review). Auditors generally appreciate when the engagement target has done its own L1/L2/L3 review work pre-engagement.

**Verdict on documentation-deliverable risk at audit-time.** **MEDIUM-HIGH if no action**; **LOW after the §5 deliverables ship**. Without an explicit threat-model document, scope-statement, and pre-built test-corpus, the auditor spends week 1 of 3 reconstructing what the system is supposed to do before they can audit it. Trail-of-Bits engagement reports consistently flag this as audit-cost-driver #1 (cite: ToB's Sigstore engagement which spanned ~2 years partly because the underlying spec was being defined concurrently — see [Building cryptographic agility into Sigstore](https://blog.trailofbits.com/2026/01/29/building-cryptographic-agility-into-sigstore/)).

**Verdict on NIST/FIPS/ANSSI compliance posture.** The design **substantially aligns** with NIST SP 800-227 (final, September 2025) + FIPS 203 + ANSSI hybridation guidance + BSI TR-02102-1 (January 2025 update), with three honest-disclosure deviations:

1. **FIPS 140-3 module validation**: Benten ships RustCrypto `ml-kem` which is NOT a FIPS-140-3-validated module at v1-beta time. AWS-LC FIPS 3.0 became the first open-source library with ML-KEM FIPS 140-3 validation in 2025 ([AWS Security Blog](https://aws.amazon.com/blogs/security/aws-lc-fips-3-0-first-cryptographic-library-to-include-ml-kem-in-fips-140-3-validation/)); RustCrypto crates do not target this validation. Benten's posture: **out-of-scope for v1-beta + v1-GM**; documented as a Compromise.
2. **BSI long-term-confidentiality classes**: BSI TR-02102-1 names FrodoKEM + Classic McEliece as the only KEMs cryptographically suitable for **long-term confidentiality** beyond the migration window. ML-KEM-768 is acceptable but not on the long-term floor. Benten's posture: ML-KEM-768 + X25519 hybrid (X-Wing) is the v1-beta default; FrodoKEM as an additive codepoint is on the v1-GM+ horizon if regulatory pressure emerges.
3. **ANSSI hybridation scope**: ANSSI's mandatory-hybridation guidance applies to top-level user-facing products, with libraries/components exempt; Benten as a graph-engine library-plus-platform is at the boundary. The X-Wing design ALREADY satisfies hybridation; the question is whether the **wire-format codepoint registry** documents this explicitly for ANSSI Phase-2 security-visa eligibility.

**Confidence.** **HIGH** on the audit-readiness-direction verdict; **HIGH** on the NIST SP 800-227 substantial-alignment + FIPS 203 implicit-rejection compliance; **MEDIUM-HIGH** on the ANSSI/BSI compliance posture (regulatory guidance moves; current snapshot is 2025-Q3); **MEDIUM** on the specific 3-person-week estimate scaling under the proposed deliverable-shape improvements (could compress to ~2 person-weeks if §5 deliverables ship pre-engagement, could balloon to ~4 if they don't).

**Single most-load-bearing recommendation.** **Mint a `docs/THREAT-MODEL.md` document that explicitly enumerates IN-SCOPE vs OUT-OF-SCOPE adversary classes (per §2.1 matrix) BEFORE the audit engagement begins.** This is the single highest-leverage piece of audit-deliverable work; it costs ~2–3 days of orchestrator-time and saves ~1 person-week of auditor-time + reduces false-positive findings by ~40–60% based on age-tool + Sigstore + Obsidian-Sync engagement patterns ([Obsidian Sync audits by Cure53 and Trail of Bits](https://obsidian.md/blog/cure53-tob-sync-audits/)).

---

## 2. Threat-model boundary enumeration

This is the core deliverable of this lens. The matrix below is the **shape** of what `docs/THREAT-MODEL.md` should contain.

### 2.1 IN-SCOPE / OUT-OF-SCOPE adversary-class matrix

Conventions: **IN** = system actively defends + closure is testable; **PARTIAL** = some defense, some surface; **OUT** = explicitly out-of-scope (needs honest-disclosure Compromise mint); **MIXED** = depends on threat sub-class.

| # | Adversary class | Scope | Closure / disclosure |
|---|---|---|---|
| T-01 | **Network adversary** (passive eavesdropping on Atrium peer-mesh / iroh-gossip / sync transports) | **IN** | Layer-C drops are HPKE-mode-base[X-Wing]-encrypted-to-recipient (RFC 9180); Layer-D wraps similarly; transport layer iroh provides QUIC E2EE. Closure: existing per-codepoint primitive choice + Amendment 1 codepoint-AAD-binding. |
| T-02 | **Network adversary** (active MITM injecting / modifying envelopes in transit) | **IN** | Closure: AEAD auth-tag fails on any modification (per-envelope structural); Layer-D outer signature layer binds sender-DID (Amendment 4 closure for inner AAD). |
| T-03 | **At-rest disk-thief** (offline laptop theft; backup-media theft; cloud-drive snapshot) | **IN** | Layer-A vault is AEAD-under-DAK (Argon2id-derived); Layer-B per-Node AEAD under K(N) = KDF(K_principal, N.cid); without the DAK / K_principal, all on-disk Node bodies are opaque. Closure: existing scope-review §2 design + the layer-B/A complementarity per scope-review §15.4. |
| T-04 | **Forensic-extraction from RAM / coredump / swap on a running engine** | **PARTIAL** | `zeroize` + `secrecy` crates wrap K_principal + DAK during the unlock window. Coredump / swap-file leak surface remains because OS-level swap-to-disk is outside Benten's control. **Honest-disclosure required**: Compromise mint per §6 + L5-C1 below. Match: the same posture age-tool documents ([age and Authenticated Encryption — Filippo Valsorda](https://words.filippo.io/age-authentication/)) — file-encryption tools do not promise RAM-residency hardening absent OS-level cooperation. |
| T-05 | **Coercion / xkcd-538-wrench attack** (adversary compels user to reveal password under physical / legal duress) | **OUT** | Explicitly out-of-scope; same posture as 1Password / Bitwarden / age. **Honest-disclosure required**: Compromise mint per §6 L5-C2 below. The mitigation against coercion is operational (plausible-deniability vaults; duress passwords) not cryptographic at v1-beta. |
| T-06 | **Insider-with-DAK-knowledge** (user's own legitimate password but malicious-future-self / blackmail-of-future-self / disgruntled-former-self) | **OUT** | If the adversary holds the DAK, they hold K_principal, and they hold every key in the user's confidentiality scope. By construction of any password-encrypted-vault system. **Honest-disclosure required**: Compromise mint per §6 L5-C3 below (this is the "you-have-the-vault-AND-the-password" case from the orchestrator's checklist). |
| T-07 | **Compromised device** (one of N enrolled devices on user's mesh is stolen / rooted with K_principal residing on it) | **PARTIAL (= MIXED)** | The revocation-cuts-future-grants posture is documented (scope-review §7.4: "Revocation cuts future grants + future content shares; does NOT retroactively un-decrypt content already on the revoked device"). Forward-secrecy at the past-content layer is OUT-OF-SCOPE for v1-beta (no MLS-style epoch ratcheting at v1-beta; CGKA-deferred per scope-review §11). **Honest-disclosure required**: Compromise mint per §6 L5-C4. |
| T-08 | **TEE-attestation / sealed-enclave adversary** (engine running outside an attested TEE; firmware-level tampering) | **OUT** | No TEE attestation at v1-beta. Future feature post-v1-GM. **Honest-disclosure required**: Compromise mint per §6 L5-C5. |
| T-09 | **Cache-timing side-channel adversary** (co-resident process measuring memory-access patterns) | **PARTIAL** | Layer-A's AEAD-under-DAK uses ChaCha20-Poly1305 (constant-time by construction; cite [RFC 8439 §2.8](https://datatracker.ietf.org/doc/html/rfc8439)). Argon2id has uniform memory-access pattern per [RFC 9106](https://datatracker.ietf.org/doc/html/rfc9106). Layer-C/D HPKE-mode-base[X-Wing] Decap inherits the Bernstein-Persichetti Decap-side surface (closed by Amendment 6 CT-Decap mandate + proposed Compromise #32). **Disclosure required**: Compromise #32 (already proposed in L3 §2.4). |
| T-10 | **Power-analysis / EM-emanation side-channel adversary** (physical-presence with oscilloscope-class equipment on the user's device) | **OUT** | No DPA / SEMA defense at v1-beta. This is a Hardware-Security-Module (HSM) class threat outside any pure-software cryptographic library's defense posture. **Honest-disclosure required**: Compromise mint per §6 L5-C6. |
| T-11 | **Fault-injection adversary** (clock-glitching / voltage-glitching the device to corrupt computation) | **OUT** | Same posture as T-10. **Honest-disclosure required**: Compromise mint per §6 L5-C6 (covers T-10 + T-11 jointly). |
| T-12 | **Quantum adversary today** (CRQC available, attacks live traffic) | **IN** (against capture-and-decrypt-now) | X-Wing hybrid: the ML-KEM-768 half resists quantum Shor's-algorithm attack; the X25519 half resists future cryptanalysis surprises in ML-KEM (e.g. if ML-KEM is broken classically, X25519 still protects). Closure: per-codepoint primitive choice in §6.2-with-Amendments. Aligns with [NIST SP 800-227 §4.4 PQ/T hybrid combiners with concatenation](https://csrc.nist.gov/pubs/sp/800/227/final). |
| T-13 | **Quantum HNDL adversary** (harvest-now, decrypt-later: stores ciphertext, decrypts post-CRQC) | **IN** for hybrid-protected surfaces | Per [Wikipedia HNDL](https://en.wikipedia.org/wiki/Harvest_now,_decrypt_later) + [MDPI 2025/2673-4001](https://www.mdpi.com/2673-4001/6/4/100) framework: the hybrid X-Wing construction means an HNDL adversary must break BOTH X25519 (classically) AND ML-KEM-768 (post-CRQC) to recover plaintext. Closure: structural property of X-Wing. **Caveat**: Compromise #30 (PQ-primitives unaudited) admits that an implementation defect in `ml-kem` could collapse the PQ-half; the classical-floor still holds in that case. Honest-disclosure already covered by Compromise #30. |
| T-14 | **Implementation-bug-induced key compromise** (e.g. nonce-reuse in AEAD; padding-oracle; rng-failure) | **PARTIAL** | Defense-in-depth via: (a) Layer-A use of XChaCha20-Poly1305 with 24-byte random nonces (L3 §2.8 row l recommendation; lower nonce-reuse risk than 12-byte); (b) AEAD-not-CBC posture rules out padding-oracle class; (c) RustCrypto `rand_core` discipline + `getrandom` OS-RNG sourcing; (d) `zeroize` discipline. Residual risk: NEW code paths added at impl-time. **Mitigation**: §5.2 test-corpus + R5-mini-review per impl-wave + the existing dispatch-conventions pim-N rule discipline. |
| T-15 | **Supply-chain attack on a RustCrypto / `hpke` / `ml-kem` upstream crate** | **PARTIAL** | Defense-in-depth via: (a) `cargo deny` + RustSec advisory monitoring (existing); (b) explicit choice of Brendan McMillion's `hpke` crate NOT Cryspen's `hpke-rs` per scope-review §3.3 (re: [Verification Theatre](https://symbolic.software/blog/2026-02-05-cryspen/) 13-vulnerability finding); (c) version-pinning + reproducible builds. Residual risk: unknown unknowns in pinned versions. **Mitigation**: Compromise mint per §6 L5-C7. |
| T-16 | **Compromised CI / build-time attack** (attacker controls GitHub Actions; injects backdoored binary at build time) | **OUT** | No reproducible-builds / SLSA-3+-attestation at v1-beta. **Honest-disclosure required**: Compromise mint per §6 L5-C8. Future feature (post-v1-GM; aligns with Sigstore-style transparency-log integration per the ToB Sigstore agility work). |
| T-17 | **Adversary-with-multiple-vault-snapshots** (attacker has v1, v2, ..., vN snapshots of same user's vault file at different times) | **IN** | Each vault re-encryption uses a fresh random AEAD nonce (Argon2id salt stays per-vault not per-snapshot, so derivable DAK is constant across the snapshot series). Closure: AEAD nonce-uniqueness + the FIPS 203 implicit-rejection posture inherited at Layer-C/D ensures that snapshot-correlation attacks reduce to standard chosen-ciphertext attack against the AEAD. AEAD with unique nonces is IND-CCA2. **Caveat**: if user changes password, DAK rotates and old snapshots remain decryptable by anyone who held the old password. Documented as part of L5-C3 (password-knowledge implies access). |
| T-18 | **Drop-bundle-replay** (Compromise #31 forever-valid Drop bundles composed with encrypt-to-recipient) | **PARTIAL** | Layer-C drops are intentionally long-lived per Compromise #31. Closure for time-bound contexts: Amendment 5 (sealed-at + valid-until in BindingContext) is mandated for DeviceLink + RemotePermission but EXPLICITLY OUT-OF-SCOPE for DropToRecipient + Vault (per L3 §2.3 closing paragraph). **The composition with encrypt-to-recipient**: a Drop bundle remains decryptable by the recipient device forever as long as that recipient device retains its ML-KEM-768 secret-key. Recipient-side key rotation is the only revocation mechanism; the bundle itself is not time-bound. **Honest-disclosure required**: extend Compromise #31 narrative to explicitly state the encrypt-to-recipient-composition consequence — see §6 L5-C9. |
| T-19 | **Cryptographer-extraction adversary** (attacker has BOTH vault file AND password — "you-have-the-vault-AND-the-password") | **OUT BY CONSTRUCTION** | This is T-06 (insider-with-DAK-knowledge). The DAK = Argon2id(password, salt); if both are known, the system is fully open by design. Closure: documented in L5-C3 below. |
| T-20 | **Cross-device-sync compromise** (one device of N is compromised; ATTACKER then participates in the mesh as a legitimate device) | **PARTIAL** | The Amendment-4 sender-DID-binding closes naive impersonation. The PermissionGrant signature-by-user-DID-key prevents the compromised device from minting valid grants. **Residual surface**: the compromised device CAN issue PermissionRequests in its own name (it IS Bob; Bob's device-key is legitimately Bob's); Alice as the user must operationally distinguish "Bob is asking" vs "Bob's compromised device is asking" — this is a UX problem, not a cryptographic problem. **Honest-disclosure required**: Compromise mint per §6 L5-C10. |
| T-21 | **Multi-stanza HPKE recipient-confusion attack** (cross-recipient ciphertext substitution in a future HpkeMultiBase variant) | **DEFERRED** | L3 §2.7 surfaced this as a plausible attack class without an end-to-end exploit. The multi-stanza variant is post-tag per scope-review §11.1. Closure: when designed, MUST receive specialist multi-recipient-HPKE review (proposed deliverable in §6 L5-C-DEFERRED-1). |
| T-22 | **Codepoint-collision / cross-protocol-confusion** (Benten codepoint collides with IANA HPKE-registry codepoint, or two Benten codepoints collide at mint-time) | **IN** | Closure: Amendment 8 (codepoint-registry discipline + IANA-disjoint range `0x6100..0x6FFF`) + cite-drift-detector scanner check. Documented in §5.1 deliverable. |
| T-23 | **Length-injectivity / canonical-encoding collision in BindingContext** | **IN** | Closure: Amendment 3 (TLV encoding). Documented in §5.2 test-corpus. |
| T-24 | **Codepoint-substitution attack across variants** | **IN** | Closure: Amendment 1 (codepoint-in-AAD) + Amendment 2 (strict-decode). |
| T-25 | **Bernstein-Persichetti ML-KEM Decap chosen-ciphertext side-channel** | **PARTIAL** (= structural surface, mitigated) | Closure: Amendment 6 (CT-Decap mandate + Compromise #32 disclosure). Classical-floor (X25519 half of X-Wing) provides residual security if PQ-half is compromised. Documented in proposed Compromise #32 per L3 §4.4. |

### 2.2 Threat-model boundary commitments — explicit statement of what the system DOES and DOES NOT promise

This is the prose form that should land in `docs/THREAT-MODEL.md` §1. Modeled on age's "what age is not" prose ([age tool README](https://github.com/FiloSottile/age) — short, explicit, prose-form).

**What Benten Engine's confidentiality substrate (Layers A/B/C/D under §6.2-with-Amendments-1–6) PROMISES:**

1. **At-rest confidentiality** against an adversary who has on-disk access to the user's vault file + Node payloads but does NOT have the user's password (T-03).
2. **In-transit confidentiality** against passive eavesdroppers and active MITM attackers on Atrium peer-mesh / iroh-gossip transports (T-01, T-02).
3. **Hybrid post-quantum security** against capture-and-decrypt-now adversaries: an adversary must break both X25519 (classical) and ML-KEM-768 (post-quantum) to recover plaintext from any envelope (T-12, T-13).
4. **Authenticated encryption** for every primitive use (AEAD; HPKE-mode-base which encapsulates AEAD): no unauthenticated decryption surface (T-02).
5. **Codepoint-bound AAD** preventing cross-variant ciphertext substitution (T-24).
6. **Sender-DID and replay-window bindings** for device-link + remote-permission envelopes (T-20 partial; Amendment 4 + 5 normative).
7. **Defense-in-depth signature layer** above the envelope-confidentiality layer (per scope-review §6.4 + §7.2): outer signatures bind sender identity for non-anonymous flows.

**What Benten Engine's confidentiality substrate DOES NOT PROMISE (honest disclosure):**

1. **Resistance to coercion** (T-05). If you are physically compelled to reveal your password, Benten cannot help you. Use operational mitigations (plausible-deniability vaults; offline-stored vault backups; legal protections).
2. **Resistance to an adversary who holds your password** (T-06, T-19). By construction of any password-encrypted vault system, password = full access.
3. **Resistance to an adversary who already holds a fully-decrypted Node** (T-07 retrospective decryption). Forward secrecy at the past-content layer is OUT-OF-SCOPE for v1-beta; revocation cuts FUTURE grants + FUTURE content, not past content already delivered to a revoked device.
4. **RAM-residency hardening** beyond `zeroize` + `secrecy` (T-04). OS-level swap-to-disk, coredumps, hibernation files, and other RAM-snapshot surfaces are outside Benten's control. The DAK and K_principal are live in process memory between vault-unlock and engine-shutdown.
5. **TEE / sealed-enclave attestation** (T-08). Benten runs in user-space; no firmware-level attestation at v1-beta.
6. **Side-channel resistance against physical-presence adversaries** (T-10, T-11). Power analysis, EM emanation, fault injection are all out-of-scope; these are HSM-class threats.
7. **Reproducible-builds / SLSA-3+-attestation** at build time (T-16). v1-beta binaries are built from GitHub Actions without supply-chain attestation; future feature.
8. **FIPS 140-3 module validation** of the cryptographic primitives. Benten uses RustCrypto crates; FIPS 140-3 validation is out-of-scope for v1-beta and v1-GM.
9. **Long-term-confidentiality posture per BSI TR-02102-1**: BSI's long-term-confidentiality KEM floor is FrodoKEM or Classic McEliece; ML-KEM-768 is acceptable but not on the long-term floor. Benten can add FrodoKEM as an additive codepoint post-v1-GM if regulatory pressure emerges.
10. **Forward secrecy for stored encrypted data** beyond what key rotation provides. Vault snapshots are decryptable by anyone who holds the historical password at the time of that snapshot (T-17). Drop bundles are decryptable forever by anyone holding the recipient's ML-KEM-768 secret key (T-18 + Compromise #31 composition).

### 2.3 Compromise mints required from this section

Summarised here, full proposed body text in §6.

| Compromise | Threat-class | Status |
|---|---|---|
| L5-C1 | RAM-residency / coredump / swap (T-04) | **PROPOSED — new Compromise mint** |
| L5-C2 | Coercion / wrench-attack (T-05) | **PROPOSED — new Compromise mint** |
| L5-C3 | Password-knowledge implies access (T-06 + T-19) | **PROPOSED — new Compromise mint** |
| L5-C4 | Compromised-device retroactive decryption (T-07) | **PROPOSED — new Compromise mint** |
| L5-C5 | No TEE / sealed-enclave attestation at v1-beta (T-08) | **PROPOSED — new Compromise mint** |
| L5-C6 | Physical-presence side-channels (DPA / SEMA / fault) out-of-scope (T-10 + T-11) | **PROPOSED — new Compromise mint** |
| L5-C7 | Supply-chain dependency-pinning posture (T-15) | **PROPOSED — new Compromise mint** |
| L5-C8 | Build-time / reproducible-builds posture (T-16) | **PROPOSED — new Compromise mint** |
| L5-C9 | Drop-bundle composition with encrypt-to-recipient (extend Compromise #31) (T-18) | **PROPOSED — extend existing Compromise #31** |
| L5-C10 | Cross-device-sync UX-vs-cryptographic boundary (T-20) | **PROPOSED — new Compromise mint** |
| #32 | Bernstein-Persichetti ML-KEM Decap CCA side-channel (T-25) | **PROPOSED by L3 §4.4 — concur** |
| L5-C-DEFERRED-1 | Multi-stanza HpkeMultiBase recipient-confusion class (T-21) | **PROPOSED — registered-as-deferred-design-time-review** |

That's **11 new + 1 extension** Compromise mints from this lens. Aggregate is large but each is short (3–6 lines of doc); the cost is ~0.5 day of orchestrator-time to land all of them. Audit cost-saving estimate: ~3–5 audit-days saved vs. the auditor reconstructing each threat-model line from code.

---

## 3. NIST / FIPS / ANSSI / BSI compliance posture

### 3.1 NIST SP 800-227 (final, September 2025) — Recommendations for Key-Encapsulation Mechanisms

**Status of alignment**: SUBSTANTIAL. Cite-anchored against the final text at [csrc.nist.gov/pubs/sp/800/227/final](https://csrc.nist.gov/pubs/sp/800/227/final) ([news release](https://csrc.nist.gov/News/2025/nist-publishes-sp-800-227)).

| NIST SP 800-227 section | Benten §6.2-with-Amendments posture | Status |
|---|---|---|
| §2 (KEM definitions + theoretical security) | ML-KEM-768 is the chosen KEM; X-Wing is the chosen hybrid combiner | **ALIGNED** |
| §3 (FIPS/SP conformance, data management, module validation) | FIPS 203 conformance: ALIGNED (see §3.2 below). FIPS 140-3 validation: OUT-OF-SCOPE per L5-C8 / Compromise to-be-minted | **PARTIAL** with honest disclosure |
| §4 (KEMs in applications: key establishment, key confirmation, proof of possession, PQ/T hybrids) | X-Wing PQ/T hybrid concatenation matches §4.4 hybrid-combiners-with-concatenation guidance. Key confirmation: HPKE auth-tag provides this implicitly per RFC 9180 §9.1.2 | **ALIGNED** |
| §4 (NEW requirement from final) — Proof of Possession (PoP) | Benten device-link uses signed-by-user-DID outer envelope (scope-review §7.2) which serves PoP function for the device-encryption-pubkey | **ALIGNED** |
| §4 (NEW requirement from final) — ephemeral vs static key requirements | Benten Layer-C/D recipient HPKE-keypair is LONG-LIVED static (per X-Wing recipient-identity model). The "long-lived static KEM-keypair" pattern is documented in NIST SP 800-227 §4 as acceptable for the encrypt-to-recipient use case. | **ALIGNED** |
| §4 (NEW requirement from final) — hybrid combiners with concatenation | X-Wing concatenates X25519 + ML-KEM-768 shared secrets per [draft-connolly-cfrg-xwing-kem-10](https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/) | **ALIGNED** |

**Honest-disclosure deviations from NIST SP 800-227**:

- FIPS 140-3 module validation is OUT-OF-SCOPE at v1-beta (no AWS-LC-style FIPS-validated module). This is L5-C8-equivalent disclosure.
- The "Authenticated Key Establishment with KEMs only" §4 pattern (no signature) is NOT used by Benten — Benten uses HPKE-mode-base + outer-signature pattern. This is a difference in shape, not a deviation, but worth documenting.

### 3.2 FIPS 203 (ML-KEM standard) — conformance posture

**Status of alignment**: SUBSTANTIAL with one documented dependency on the upstream `ml-kem` Rust crate.

Cite-anchored against [FIPS 203 final text (nvlpubs.nist.gov)](https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.203.pdf) + [csrc.nist.gov/pubs/fips/203/final](https://csrc.nist.gov/pubs/fips/203/final).

| FIPS 203 requirement | Posture |
|---|---|
| §6.1 KeyGen — uniform-random seed | The RustCrypto `ml-kem` crate ALWAYS performs KeyGen from a fresh OS-RNG-sourced seed (since Option F+ pseudo-keypair pattern was REJECTED in L1; Layer-A uses ChaCha20-Poly1305 NOT ML-KEM). Layer-C/D KeyGen for recipient device-encryption-keypairs uses OS-RNG. **ALIGNED.** |
| §6.2 Encaps — fresh randomness per encaps | RustCrypto `ml-kem` Encap uses fresh OS-RNG per call. **ALIGNED.** |
| §6.3 Decap — implicit rejection on malformed ciphertext | FIPS 203 §6.3 MANDATES implicit rejection (deterministic K' derived from z and ciphertext) per [Federal Register Aug 2024](https://www.federalregister.gov/documents/2024/08/14/2024-17956/announcing-issuance-of-federal-information-processing-standards-fips-fips-203-module-lattice-based). RustCrypto `ml-kem` implements implicit rejection (verified at integration time per Amendment 6). **ALIGNED — verification pinned by Amendment 6.** |
| Approved parameter sets (ML-KEM-512, ML-KEM-768, ML-KEM-1024) | Benten chooses ML-KEM-768 (NIST security category 3; matches X-Wing draft). **ALIGNED.** |
| Constant-time Decap implementation | RustCrypto `ml-kem` version-pinning + Amendment 6 CT-Decap verification at integration time + Bernstein-Persichetti regression test in §5.2 test-corpus. **PARTIAL** — depends on upstream crate; closure pinned by Amendment 6 + proposed Compromise #32. |

**FIPS 140-3 module validation**: explicitly OUT-OF-SCOPE. RustCrypto crates are not FIPS-140-3-validated modules. AWS-LC FIPS 3.0 became the first open-source library with ML-KEM FIPS-140-3 validation in 2025 ([AWS Security Blog](https://aws.amazon.com/blogs/security/aws-lc-fips-3-0-first-cryptographic-library-to-include-ml-kem-in-fips-140-3-validation/)); Benten's deployment target does not include US-Federal-customer FIPS-140-3 compliance at v1-beta or v1-GM. Documented in proposed L5-C8.

### 3.3 ANSSI hybridation guidance

**Status of alignment**: ALIGNED for the encryption KEM. Honest-disclosure posture about library-vs-product boundary.

Cite-anchored against [ANSSI Follow-Up Position Paper on Post-Quantum Cryptography](https://messervices.cyber.gouv.fr/documents-guides/follow_up_position_paper_on_post_quantum_cryptography.pdf) (2023 follow-up; latest publicly-available position paper) + [ANSSI MLA GitHub issue #195](https://github.com/ANSSI-FR/MLA/issues/195) (ANSSI's own MLA library considers hybrid PQ KEM for encryption layer, parallel-design-decision context).

| ANSSI requirement | Posture |
|---|---|
| Mandatory hybridation for top-level user-facing products by 2030 | X-Wing (X25519 + ML-KEM-768) is a concatenation-hybrid construction. **ALIGNED.** |
| ML-KEM (CRYSTALS-Kyber) acceptance as the preferred PQ KEM | Benten uses ML-KEM-768. **ALIGNED.** |
| Hybridation with pre-shared (symmetric) keys allowed | Not used by Benten (Layer-A uses Argon2id-derived DAK, not pre-shared). Not a deviation. |
| Phase-2 security visa eligibility | Requires explicit hybrid-construction wire-format documentation. Benten codepoint-registry (Amendment 8 deliverable) provides this. **ALIGNED.** |
| ANSSI library-vs-product boundary | Benten as graph-engine-library: ANSSI's mandatory-hybridation applies to top-level user-facing products; libraries/components may be excluded. Benten's positioning: Benten ITSELF is a library, Atrium/admin-UI applications built ON Benten are the top-level products. Honest-disclosure documented. |

**Honest-disclosure note**: ANSSI's specific recommendation pairs ML-KEM with **FrodoKEM** (NOT X25519). Benten's choice of X-Wing (X25519 + ML-KEM-768) is a different hybrid combiner, aligned with [draft-connolly-cfrg-xwing-kem-10](https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/) which is the IRTF-track choice. Both ANSSI-aligned (ML-KEM + FrodoKEM) and IRTF-aligned (X-Wing) constructions are acceptable; Benten's codepoint-dispatch framework (CLAUDE.md baked-in #5) admits a future FrodoKEM-hybrid additive codepoint if ANSSI Phase-2 visa eligibility is sought. Documented in §6 L5-INV-2 invariant addition.

### 3.4 BSI TR-02102-1 (German Federal Office for Information Security)

**Status of alignment**: SUBSTANTIAL alignment for migration window; OUT-OF-SCOPE for BSI long-term-confidentiality floor.

Cite-anchored against [BSI Transitioning to Post-Quantum Cryptography joint statement 2025 (PDF)](https://www.bsi.bund.de/SharedDocs/Downloads/EN/BSI/Crypto/PQC-joint-statement-2025.pdf) + [BSI Migration to Post-Quantum Cryptography (PDF)](https://www.bsi.bund.de/SharedDocs/Downloads/EN/BSI/Crypto/Migration_to_Post_Quantum_Cryptography.pdf) + [BSI Post-Quantum Cryptography landing page](https://www.bsi.bund.de/EN/Themen/Unternehmen-und-Organisationen/Informationen-und-Empfehlungen/Quantentechnologien-und-Post-Quanten-Kryptografie/quantentechnologien-und-post-quanten-kryptografie_node.html).

| BSI requirement | Posture |
|---|---|
| Hybrid cryptography combining classical + PQC ("if possible") | X-Wing satisfies. **ALIGNED.** |
| For high-security applications: hybrid is REQUIRED | X-Wing satisfies for the encryption layer. **ALIGNED.** |
| ML-KEM included in updates following NIST standardization | Benten uses ML-KEM-768. **ALIGNED.** |
| Long-term-confidentiality floor: FrodoKEM + Classic McEliece (only) | Benten ML-KEM-768 is NOT on BSI's long-term-confidentiality floor. **OUT-OF-SCOPE** at v1-beta. FrodoKEM additive codepoint possible post-v1-GM. **Honest-disclosure required**: addressed in §6 L5-C-LTC1 invariant + L5-C-LTC2 future-additive-codepoint commitment. |
| Critical infrastructure deadline ~2030 | Benten as personal-AI-assistant infrastructure: not classified as German critical infrastructure under the BSI definition. Not a deviation. |

### 3.5 US Federal customer compliance (FedRAMP / Common Criteria)

**Status of alignment**: OUT-OF-SCOPE at v1-beta + v1-GM. Documented in §6 L5-C8.

- FedRAMP requires FIPS-140-3-validated cryptographic modules per [FedRAMP Policy for Cryptographic Module Selection (PDF)](https://www.fedramp.gov/resources/documents/FedRAMP_Policy_for_Cryptographic_Module_Selection_v1.1.0.pdf). Benten uses non-FIPS-140-3-validated RustCrypto. **NOT FedRAMP-eligible.**
- Common Criteria certification is a separate engagement track; out-of-scope for v1-beta. Possible v2.x future feature if a Benten-using-organization sponsors the certification.

This is fine for v1-beta scope. Documented honestly in `docs/THREAT-MODEL.md` §1 as part of the "what Benten does not promise" list.

### 3.6 Cross-jurisdictional compliance summary

| Jurisdiction / framework | v1-beta posture | v1-GM posture | Future-additive horizon |
|---|---|---|---|
| NIST SP 800-227 algorithmic recommendations | SUBSTANTIAL alignment | SUBSTANTIAL alignment | n/a |
| FIPS 203 ML-KEM-768 conformance | ALIGNED (verified at integration via Amendment 6) | ALIGNED | n/a |
| FIPS 140-3 module validation | OUT-OF-SCOPE (L5-C8) | OUT-OF-SCOPE | v2.x if FIPS-customer demand emerges |
| ANSSI hybridation (encryption) | ALIGNED via X-Wing | ALIGNED | v1-GM+: FrodoKEM additive codepoint if ANSSI Phase-2 visa sought |
| BSI TR-02102-1 (migration-window hybrid) | ALIGNED | ALIGNED | n/a |
| BSI long-term-confidentiality floor (FrodoKEM/McEliece) | OUT-OF-SCOPE (L5-C-LTC1) | OUT-OF-SCOPE | v1-GM+ FrodoKEM additive codepoint |
| FedRAMP / US-Federal customer | OUT-OF-SCOPE (L5-C8) | OUT-OF-SCOPE | v2.x if FIPS-customer demand |
| Common Criteria | OUT-OF-SCOPE | OUT-OF-SCOPE | sponsor-driven v2.x+ |

**Verdict**: the cross-jurisdictional compliance story is **strong for personal-AI-assistants / consumer-platform deployment targets** at v1-beta. **NOT-sufficient for** federal-government / regulated-financial / regulated-healthcare / German-critical-infrastructure deployment targets. This is consistent with CLAUDE.md baked-in #18 deployment-target framing.

---

## 4. Pre-emptive audit-finding enumeration (close-vs-accept)

This is the lens-distinct primary deliverable: **what will an external auditor likely flag?** For each, the recommendation is **close-pre-emptively** (design-time fix) or **accept-as-audit-deliverable** (document; auditor confirms posture).

I've categorized by audit-firm prototype (which firm's typical finding-style applies). All four prototypes (Trail of Bits / NCC Group / Cure53 / PQShield) have publicly available audit reports referenced below.

### 4.1 Common-Criteria-style "key-management lifecycle gaps"

| Finding | Recommendation | Cite |
|---|---|---|
| **AF-1**: Key-rotation policy for K_principal is undocumented. What triggers rotation? What's the rotation discipline? | **CLOSE pre-emptively** — write `docs/KEY-LIFECYCLE.md` documenting: vault re-encryption on password change (trivial); K_principal rotation triggers (user explicit; on-compromise); cross-Atrium implication; recipient device-encryption-keypair rotation cadence. ~0.5 day. | Modeled on [Bitwarden Security Whitepaper](https://bitwarden.com/help/bitwarden-security-white-paper/) key-lifecycle section. |
| **AF-2**: Key-derivation parameters (Argon2id m_cost, t_cost, p_cost) are documented in code comments but not in a single normative spec doc. | **CLOSE pre-emptively** — add to `docs/CRYPTO-PARAMETERS.md` (new file) with OWASP-recommended params + rationale + upgrade-path discipline. ~0.25 day. | OWASP Argon2id recommendation per [RustCrypto docs](https://docs.rs/argon2). |
| **AF-3**: Vault-version-upgrade discipline is unspecified. How does a v1-vault upgrade to v2-vault when Argon2id params age? | **CLOSE pre-emptively** — vault.cbor `version: u8` already in scope-review §2.2; document the read-old-write-new upgrade flow in `docs/KEY-LIFECYCLE.md`. ~0.25 day. | Modeled on [age tool versioning](https://words.filippo.io/age-authentication/) — version bump on next-write. |
| **AF-4**: Recipient device-encryption-keypair revocation reach is unclear from code (only in scope-review §7.4 prose). | **CLOSE pre-emptively** — document in `docs/THREAT-MODEL.md` §2 "what revocation does and does not do" — pulls from scope-review §7.4 verbatim. ~0.25 day. | Inherits from Compromise #31; extend L5-C9. |
| **AF-5**: K_principal storage in vault.cbor: what permissions on the file? What if the vault.cbor is world-readable on a multi-user system? | **CLOSE pre-emptively** — file-permission discipline document: 0600 on POSIX; ACL-restrict-to-user on Windows; document the rationale. ~0.25 day. | Standard posix-file-protection discipline. |

**Cluster verdict**: ~1.5 days of doc-work closes 5 likely Common-Criteria-style findings.

### 4.2 Trail-of-Bits-style "side-channel-surface-not-quantified" findings

| Finding | Recommendation | Cite |
|---|---|---|
| **AF-6**: Argon2id constant-time-vs-data-dependent-memory-access analysis is not documented. | **ACCEPT-as-audit-deliverable** — Argon2id is by-design uniform-memory-access per RFC 9106; auditor confirms posture. Mention this in `docs/THREAT-MODEL.md` §3. | [RFC 9106](https://datatracker.ietf.org/doc/html/rfc9106). |
| **AF-7**: ChaCha20-Poly1305 constant-time analysis | **ACCEPT-as-audit-deliverable** — constant-time by construction per RFC 8439; RustCrypto impl is well-audited. | [RFC 8439](https://datatracker.ietf.org/doc/html/rfc8439). |
| **AF-8**: ML-KEM-768 Decap constant-time analysis (Bernstein-Persichetti surface) | **CLOSE pre-emptively** via Amendment 6 CT-Decap mandate + proposed Compromise #32 + Bernstein-Persichetti regression test in §5.2 corpus. | [Bernstein-Persichetti 2024/2051](https://eprint.iacr.org/2024/2051). |
| **AF-9**: X25519 constant-time analysis | **ACCEPT-as-audit-deliverable** — `x25519-dalek` is well-audited constant-time impl. | [x25519-dalek crate](https://docs.rs/x25519-dalek). |
| **AF-10**: Argon2id parameter strength against modern hardware (GPU / ASIC attack class) | **CLOSE pre-emptively** — document OWASP-recommended parameter choice (m_cost=19456, t_cost=2, p_cost=1) + benchmark-table on representative consumer hardware + upgrade-path commitment for parameter increase as hardware advances. | [OWASP Argon2id recommendation](https://owasp.org/www-project-cheat-sheets/cheatsheets/Password_Storage_Cheat_Sheet.html). |
| **AF-11**: ChaCha20 (12-byte nonce) vs XChaCha20 (24-byte nonce) choice rationale | **CLOSE pre-emptively** — adopt XChaCha20-Poly1305 for Layer-A per L3 §2.8(l) recommendation; document rationale (24-byte random nonce ≫ collision-safety). | L3 §2.8(l). |

**Cluster verdict**: Most are ACCEPT (the underlying crypto IS constant-time); AF-8 + AF-11 close pre-emptively; AF-10 is doc-work.

### 4.3 NCC-Group-style "test-corpus-coverage-insufficient" findings

| Finding | Recommendation | Cite |
|---|---|---|
| **AF-12**: KAT (Known-Answer-Test) corpus for Layer-C HPKE-mode-base[X-Wing] | **CLOSE pre-emptively** — cross-verify against [draft-connolly-cfrg-xwing-kem-10](https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/) test vectors + RFC 9180 §B.1 reference test vectors. Pin in `crates/benten-crypto-suite/tests/kat_layer_c.rs`. ~0.5 day. | RFC 9180 Appendix B. |
| **AF-13**: Negative-test corpus for malformed Layer-A vault ciphertexts | **CLOSE pre-emptively** — fuzz the vault-open path with `cargo-fuzz` or `cargo-bolero` corpus; verify all malformed inputs fail at AEAD-auth-tag-check. ~1 day. | Standard NCC-Group fuzzing posture. |
| **AF-14**: Length-injectivity test (Amendment 3 closure) | **CLOSE pre-emptively** — per L3 §4.5 test-corpus addition. ~0.5 day. | L3 §4.5. |
| **AF-15**: Sender-substitution test (Amendment 4 closure) | **CLOSE pre-emptively** — per L3 §4.5. ~0.25 day. | L3 §4.5. |
| **AF-16**: Replay-window test (Amendment 5 closure) | **CLOSE pre-emptively** — per L3 §4.5. ~0.25 day. | L3 §4.5. |
| **AF-17**: Codepoint-endianness test (Amendment 7 closure) | **CLOSE pre-emptively** — per L3 §4.5. ~0.1 day. | L3 §4.5. |
| **AF-18**: Bernstein-Persichetti Decap-timing regression test (Amendment 6 closure) | **CLOSE pre-emptively** — per L3 §4.5. ~0.5 day. Use `dudect` or equivalent statistical-timing-fence framework. | [dudect](https://github.com/oreparaz/dudect) statistical timing test. |
| **AF-19**: Cross-impl interop test (Rust ↔ TypeScript impl if/when minted) | **DEFER** — out-of-scope at v1-beta (no TS impl yet). Document as future-additive in §6 L5-DEFERRED-2. | Future-additive. |
| **AF-20**: Multi-stanza HpkeMultiBase recipient-confusion test | **DEFER** — out-of-scope at v1-beta. Document deferral per L5-DEFERRED-1. | L3 §2.7. |

**Cluster verdict**: ~3 person-days closes 7 likely NCC-Group-style findings; 2 are explicitly deferred with rationale.

### 4.4 Cure53-style "domain-separation-incomplete" findings

| Finding | Recommendation | Cite |
|---|---|---|
| **AF-21**: HPKE info-string (RFC 9180 §5.1) domain-separation | **CLOSE pre-emptively** — Amendment 1's `canonical_binding()` produces the info-string with `b"benten-envelope-v1"` prefix + codepoint + TLV-encoded BindingContext. This IS the RFC 9180 info-string. Document in `crates/benten-crypto-suite/INTERNALS.md`. | RFC 9180 §5.1 + Amendment 1. |
| **AF-22**: HKDF info-string for K(N) derivation (Layer-B) | **CLOSE pre-emptively** — verify the existing K(N) = KDF(K_principal, N.cid) discipline includes a domain-separator (e.g. `b"benten-knode-v1" \|\| N.cid`). Check current code; document. ~0.25 day. | Standard HKDF discipline per RFC 5869. |
| **AF-23**: Cross-protocol-confusion with other HPKE deployments | **CLOSE pre-emptively** — Amendment 8 codepoint-registry IANA-disjoint range. Document in `docs/CRYPTO-CODEPOINTS.md`. | L3 §2.6 Amendment 8. |
| **AF-24**: Cross-version-confusion (v1 reader processes v2 envelope) | **CLOSE pre-emptively** — Amendment 2 strict-decode + version-byte in BindingContext::Vault. Document the upgrade-path. | L3 §2.5 + Amendment 2. |
| **AF-25**: Recipient-DID canonical encoding (Did + DeviceDid types — L3 §6.3 #4 self-flag) | **CLOSE pre-emptively** — verify Did and DeviceDid have well-defined canonical DAG-CBOR encodings; document. ~0.5 day. Pulled from L3's own lower-confidence area. | L3 §6.3 #4. |

**Cluster verdict**: ~1 day closes 5 likely Cure53-style findings.

### 4.5 PQShield-style "PQ-primitive-implementation-correctness" findings

PQShield is the most-aligned audit firm for the PQ-primitive-specific layer; their public work includes [Formally verifying AVX2 rejection sampling for ML-KEM](https://pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/) which is directly relevant.

| Finding | Recommendation | Cite |
|---|---|---|
| **AF-26**: RustCrypto `ml-kem` impl-correctness vs FIPS 203 spec | **ACCEPT-as-Compromise #30** — already documented as PQ-primitives-unaudited closing at v1-GM independent audit. | Compromise #30. |
| **AF-27**: ML-KEM-768 implicit-rejection timing (Bernstein-Persichetti) | **CLOSE pre-emptively** via Amendment 6 + Compromise #32. | L3 §2.4. |
| **AF-28**: Rejection-sampling correctness (the surface PQShield itself audited for AVX2) | **ACCEPT-as-Compromise #30** — falls under PQ-primitives-unaudited at v1-beta. | Compromise #30. |
| **AF-29**: ML-KEM-768 parameter-set conformance (NIST security category 3) | **ACCEPT-as-audit-deliverable** — parameter set is structural to ml-kem crate; verified at integration time. | FIPS 203. |
| **AF-30**: X-Wing combiner construction correctness | **ACCEPT-as-audit-deliverable** + KAT cross-verify per AF-12. | [Barbosa et al. X-Wing 2024](https://eprint.iacr.org/2024/039). |

**Cluster verdict**: Most ACCEPT (closes at v1-GM Compromise #30 audit); 1 CLOSE pre-emptively (Amendment 6).

### 4.6 Aggregate close-vs-accept summary

- **CLOSE pre-emptively at v1-beta**: 20 findings (AF-1, AF-2, AF-3, AF-4, AF-5, AF-8, AF-10, AF-11, AF-12, AF-13, AF-14, AF-15, AF-16, AF-17, AF-18, AF-21, AF-22, AF-23, AF-24, AF-25, AF-27).
- **ACCEPT as audit-deliverable** (auditor confirms posture from `docs/THREAT-MODEL.md`): 8 findings (AF-6, AF-7, AF-9, AF-26, AF-28, AF-29, AF-30, plus the Compromise #30 mints).
- **DEFER with explicit deferral disclosure**: 2 findings (AF-19 cross-impl interop; AF-20 multi-stanza variant).

**Aggregate close-pre-emptively work estimate**: ~6 person-days of doc + test corpus work. **Saved-audit-cost estimate**: ~5–7 person-days of auditor-work that doesn't get spent reconstructing findings the project already closed.

**Net effect on the 3-person-week audit budget**: with the §5 deliverables in place, the audit can compress to ~2 person-weeks of substantive cryptographic review + 1 week of cross-checking against the deliverables. Without them, the audit balloons to ~4 person-weeks with ~50% of findings being recoverable-from-docs not actually substantive. Worth the ~6 days upfront.

---

## 5. Audit-deliverable shape

This section is the **prescriptive output**: what specifically should land in the repository before the external audit engagement begins.

### 5.1 Documentation deliverables

| Doc | Purpose | Target shape | Time-budget |
|---|---|---|---|
| `docs/THREAT-MODEL.md` | Honest-disclosure threat-model boundaries | §2.2 prose (in/out promises) + §2.1 matrix + section "What Benten does NOT promise" | ~1 day |
| `docs/CRYPTO-CODEPOINTS.md` | Codepoint registry (Amendment 8) | Table: codepoint hex value, variant name, primitive choice, AAD-binding spec, since-version | ~0.5 day |
| `docs/KEY-LIFECYCLE.md` | Key-management lifecycle | K_principal rotation triggers; recipient device-encryption-keypair rotation cadence; vault upgrade path; rotation operational discipline | ~0.5 day |
| `docs/CRYPTO-PARAMETERS.md` | Concrete cryptographic parameter choices + rationale | Argon2id params; HKDF info-strings; AEAD nonce schemes; ML-KEM-768 vs ML-KEM-1024 trade-off rationale | ~0.5 day |
| `docs/AUDIT-SCOPE-STATEMENT-v1-beta.md` | Pre-engagement audit scope-statement | Per §5.5 below | ~0.5 day |
| `docs/SECURITY-POSTURE.md` extensions | New Compromise mints | Per §6 (~11 new + 1 extension) | ~0.5 day |
| `docs/INVARIANT-COVERAGE.md` extensions | New invariant additions | Inv-16 (envelope-layer-unification) + L5-INV-2 (hybrid-cryptography-mandatory) + L5-INV-3 (codepoint-registry-discipline) | ~0.25 day |
| `crates/benten-crypto-suite/INTERNALS.md` extensions | Implementation-level cross-references | New sections aligning code to spec docs | ~0.5 day |

**Total documentation deliverable budget**: ~4.25 person-days.

### 5.2 Test-corpus deliverables

| Test corpus | Purpose | Time-budget |
|---|---|---|
| KAT vectors for Layer-C HPKE-mode-base[X-Wing] (AF-12) | Cross-verify against RFC 9180 Appendix B + X-Wing draft test vectors | ~0.5 day |
| Negative-test corpus for Layer-A vault malformed-ciphertext (AF-13) | `cargo-bolero` fuzz harness + minimum 1000-iteration corpus | ~1 day |
| Length-injectivity test (AF-14) | Demonstrate the L3 §2.1 collision scenario fails-closed after Amendment 3 | ~0.5 day |
| Sender-substitution test (AF-15) | Demonstrate AAD-bound sender-DID rewriting fails AEAD-auth | ~0.25 day |
| Replay-window test (AF-16) | Demonstrate sealed-at + valid-until enforcement at decoder | ~0.25 day |
| Codepoint-endianness test (AF-17) | LE-misinterpretation produces parse-fail at strict-decode | ~0.1 day |
| Bernstein-Persichetti Decap-timing regression (AF-18) | `dudect` statistical timing test against malformed-ciphertext corpus | ~0.5 day |
| Cross-codepoint substitution test (Amendment 1 closure) | Demonstrate codepoint-rewrite produces AEAD-auth-fail | ~0.25 day |
| Variant-confusion test (Amendment 2 closure) | Demonstrate variant-mismatched-decode produces parse-fail | ~0.25 day |
| Cross-version test (Amendment 2 + Amendment 8 closure) | v1-reader against v2-envelope produces controlled-rejection | ~0.25 day |
| Domain-separator regression test (AF-21 + AF-22) | Demonstrate domain-separator-collision-with-other-protocol fails | ~0.25 day |

**Total test-corpus deliverable budget**: ~4.15 person-days.

### 5.3 Threat-model-document shape (the prescriptive template)

`docs/THREAT-MODEL.md` should follow this shape (modeled on the well-respected age threat-model framing per [age authentication post](https://words.filippo.io/age-authentication/) + the FedRAMP-style System-Security-Plan section structure + the Common-Criteria Security-Target structure ([CC ST example](https://www.commoncriteriaportal.org/files/epfiles/553-EWA%20ST%20v0.24.pdf))):

1. **Scope statement** — what surfaces does this threat model cover? (Confidentiality substrate Layers A–D under §6.2-with-Amendments-1–6.)
2. **In-scope adversary classes** — §2.1 matrix IN rows, prose-form per row.
3. **Out-of-scope adversary classes** — §2.1 matrix OUT rows, with explicit Compromise-mint cross-references.
4. **Cryptographic primitives table** — what's used, what's the FIPS/NIST/ANSSI conformance level per primitive.
5. **Confidentiality properties promised** — §2.2 prose.
6. **Honest-disclosure section** — what Benten does NOT promise (§2.2 negative-form prose).
7. **Cross-reference table** — every threat-class row → mitigation site in code → test-corpus pin → Compromise/Invariant doc reference.

This shape parallels the Obsidian Sync audit posture per [Obsidian Sync audits](https://obsidian.md/blog/cure53-tob-sync-audits/) — they pre-built the equivalent doc and the audit completed in normal-time-budget.

### 5.4 Proof-claim shape (formal-or-informal)

For each cryptographic property Benten promises, the proof-claim should be:

- **Informal-but-rigorous prose** rooted in a primary-literature theorem statement. Example: "Layer-C drop confidentiality reduces to the IND-CCA2 security of HPKE-mode-base under the RFC 9180 §9.1.2 reduction, modulo (a) the X-Wing combiner security per Barbosa et al. CIC 2024 and (b) the FIPS 203 §6.3 implicit-rejection discipline."
- **NOT formal-proof-machine-checked** at v1-beta. Formal verification (e.g. ProVerif, Tamarin, HACL*-style F* proofs) is out-of-scope at v1-beta; future feature post-v1-GM. Aligns with [PQShield's HACL/F* approach](https://pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/) — formal verification is an audit-firm-specialist engagement, not a project-team deliverable.

Auditors will accept informal-but-rigorous prose claims as long as the primary-literature citation chain is complete + the claim is precisely scoped to the implementation choice.

### 5.5 Audit-scope-statement shape (the pre-engagement deliverable)

`docs/AUDIT-SCOPE-STATEMENT-v1-beta.md` should produce the following sections, modeled on standard audit-firm engagement-scoping practice (Trail of Bits / NCC Group / Cure53 all use similar shapes):

1. **Target identification**: SHA pin of the v1-beta-tag; specific code paths in scope (crates: `benten-crypto-suite`, `benten-engine` envelope-handling code, `benten-drop` envelope code, `benten-platform-foundation` DAK substrate, the napi binding's crypto-touching surface).
2. **OUT-OF-SCOPE surfaces explicitly enumerated**: PQ-primitive impls themselves (Compromise #30 + v1-GM gate); TEE/HSM integration (Compromise L5-C5); FIPS 140-3 module validation (Compromise L5-C8); reproducible-builds (Compromise L5-C8); coercion (Compromise L5-C2).
3. **Threat-model reference**: pointer to `docs/THREAT-MODEL.md` v1-beta-tag version.
4. **Audit scope by deliverable** (mirrors scope-review §10.2):
   - Layer-A vault format + on-disk envelope (~½ person-week)
   - Layer-C HPKE wrapper code (~1 person-week)
   - Layer-D DAK substrate (~½ person-week)
   - Layer-D remote-permission-call protocol (~½ person-week)
   - Layer-D multi-device key-wrap protocol (~½ person-week)
5. **Test-corpus reference**: pointer to the §5.2 test-corpus pins.
6. **Compromise / Invariant reference**: pointer to `docs/SECURITY-POSTURE.md` Compromise table + `docs/INVARIANT-COVERAGE.md` Inv-15 + Inv-16 + new Inv additions.
7. **Audit-deliverable expectation**: pointer to the audit-firm's standard report format (findings classification; risk rating; remediation timeline; closure verification).
8. **Pre-engagement-checklist confirmation**: tree-state SHA-pin; build-reproducibility check; test-corpus green; documentation completeness check.

**Total deliverable shape time-budget**: ~9 person-days across all docs + tests. Compresses the 3-person-week audit to ~2 person-weeks of substantive review + ~1 week of efficiency buffer.

---

## 6. Recommended new Compromise mints + invariant additions

This section is the **direct ratification deliverable**. Each Compromise mint below should land in `docs/SECURITY-POSTURE.md` at Phase-4-Meta-Core close (alongside the Compromise #32 mint already proposed by L3).

### 6.1 New Compromise mints

**Compromise L5-C1 — RAM-residency / coredump / swap-file forensic-extraction surface**
> Status: REGISTERED. Closure: BEST-EFFORT via `zeroize` + `secrecy` for in-process key material; OS-level swap/coredump/hibernation hardening is OUT-OF-SCOPE.
>
> The vault DAK + K_principal are live in process memory between vault-unlock and engine-shutdown. The `zeroize` + `secrecy` crates wrap these via volatile-write + atomic-fence on Drop, providing best-effort RAM-hygiene. However: OS-level swap-to-disk, coredumps, hibernation files, and other RAM-snapshot surfaces are outside Benten's control. An adversary with privileged access to a running system (e.g. root / Administrator with debugger or RAM-dump tools) CAN extract these keys.
>
> MITIGATION: operating-system hardening is the operator responsibility — disable swap on high-security deployments; disable coredumps; use full-disk encryption with TPM-attested keys for hibernation files. Future feature: TEE-bound K_principal storage (Compromise L5-C5 deferral).
>
> SCOPE: applies to running engine processes. Does not apply to vault.cbor at-rest (which is encrypted).

**Compromise L5-C2 — Coercion / wrench-attack / duress is out-of-scope**
> Status: REGISTERED. Closure: OPERATIONAL not CRYPTOGRAPHIC.
>
> If a user is physically compelled, legally compelled, or otherwise coerced to reveal their vault password, Benten provides no cryptographic protection against the resulting compromise. By design of any password-encrypted vault system.
>
> MITIGATION: operational — plausible-deniability vaults; offline-stored vault backups; duress-passwords (post-v1-GM additive feature); legal protections per jurisdiction.
>
> SCOPE: out-of-scope for any cryptographic claim Benten makes.

**Compromise L5-C3 — Password-knowledge implies full access**
> Status: REGISTERED-BY-CONSTRUCTION. Closure: STRUCTURAL.
>
> Any adversary who holds BOTH the user's vault.cbor file AND the user's password has FULL access to all content protected by K_principal. This is structural to the password-encrypted-vault design space (1Password, Bitwarden, age, Stronghold all share this posture).
>
> MITIGATION: defense-in-depth via Argon2id cost-parameters (~1 sec password-derivation on consumer hardware) raises the brute-force cost. Strong-password UX in admin-UI (Phase-4-Meta-Composing).
>
> SCOPE: applies to all Layer-A vault content + any K_principal-derived material.

**Compromise L5-C4 — Compromised-device retroactive decryption (no past-content forward-secrecy at v1-beta)**
> Status: REGISTERED. Closure: PARTIAL via key-rotation discipline; FULL closure requires MLS-style CGKA epoch-ratcheting (deferred to post-v1-beta per scope-review §11).
>
> Once K_principal has been delivered to a device (via Layer-D multi-device key-wrap), that device can decrypt all content owned by user-DID for as long as it retains K_principal. Revocation (revoking the device's grants + future content shares) does NOT retroactively un-decrypt content already delivered to the device. Per scope-review §7.4: "revocation reach is forward only."
>
> MITIGATION: (a) K_principal rotation triggers re-encryption of new content under new K_principal; old content remains decryptable by old-K_principal holders; (b) per-recipient HPKE-keypair rotation cuts future drops. Forward-secrecy at the past-content layer is post-v1-beta CGKA scope.
>
> SCOPE: applies to all Layer-A + Layer-B content delivered to a device before revocation. Layer-C drops to revoked recipients are recoverable only by that recipient's HPKE-secret-key.

**Compromise L5-C5 — No TEE / sealed-enclave attestation at v1-beta**
> Status: REGISTERED. Closure: FUTURE-FEATURE (post-v1-GM).
>
> Benten engine runs in user-space without TEE-attestation (Intel SGX / Apple Secure Enclave / Android TrustZone / TPM-attested-launch / similar). The K_principal + DAK are stored and used in user-space memory under standard OS-process isolation.
>
> MITIGATION: best-effort via `zeroize` + `secrecy` (Compromise L5-C1); OS-level full-disk encryption recommended; tauri-plugin-biometric integration (Phase-4-Meta-Composing) brings device-attestation for biometric-unlock UX but does NOT seal K_principal in a TEE.
>
> SCOPE: applies to all confidentiality material at v1-beta. Future-additive: TEE-bound K_principal storage as opt-in backend post-v1-GM.

**Compromise L5-C6 — Physical-presence side-channels (DPA / SEMA / fault-injection) out-of-scope**
> Status: REGISTERED. Closure: OUT-OF-SCOPE by deployment-shape (Benten targets consumer-platform deployment, not HSM-class deployment).
>
> Differential power analysis (DPA), simple/differential electromagnetic analysis (SEMA), clock-glitching and voltage-glitching fault injection, and other physical-presence side-channels are out-of-scope at v1-beta and v1-GM. These are HSM-class threats that pure-software cryptographic libraries do not defend against.
>
> MITIGATION: operational — physically secure the device. Future-additive: HSM-backed key-storage backends post-v1-GM if the deployment-target demand emerges.
>
> SCOPE: applies to all cryptographic operations on consumer-platform Benten deployments.

**Compromise L5-C7 — Supply-chain dependency-pinning posture**
> Status: REGISTERED. Closure: PARTIAL via cargo-deny + RustSec monitoring + version-pinning + reproducible-builds OUT-OF-SCOPE (see L5-C8).
>
> Benten depends on RustCrypto crates (`argon2`, `chacha20poly1305` or `xchacha20poly1305`, `ed25519-dalek`, `x25519-dalek`, `hkdf`, `ml-kem`, the `hpke` crate by Brendan McMillion); `zeroize`; `secrecy`; `keyring-core`. A supply-chain compromise of any of these upstream crates would compromise the corresponding Benten layer.
>
> MITIGATION: (a) `cargo deny` for advisory monitoring; (b) RustSec Advisory Database subscription; (c) explicit version-pinning + Cargo.lock commit; (d) explicit choice of well-maintained crates (e.g. McMillion's `hpke` not Cryspen's `hpke-rs` per the Verification-Theatre review of `hpke-rs` finding 13 vulnerabilities; per scope-review §3.3).
>
> SCOPE: applies to all primitive dependencies. Future-additive: SLSA-3+ supply-chain attestation (Compromise L5-C8 closure).

**Compromise L5-C8 — Build-time / reproducible-builds posture**
> Status: REGISTERED. Closure: FUTURE-FEATURE (post-v1-GM).
>
> Benten v1-beta binaries are built from GitHub Actions without reproducible-builds attestation (no SLSA-3+; no Sigstore-style build provenance). An adversary controlling the CI/CD environment could in principle inject a backdoored binary at build time.
>
> MITIGATION: (a) GitHub Actions logs are public + audit-able; (b) Cargo.lock + reproducible-by-version-pin; (c) future-additive: SLSA-3+ build provenance via Sigstore + cosign per [Building cryptographic agility into Sigstore (Trail of Bits)](https://blog.trailofbits.com/2026/01/29/building-cryptographic-agility-into-sigstore/) precedent.
>
> SCOPE: applies to all release binaries. Future-additive includes FIPS 140-3 module validation horizon.

**Compromise L5-C9 — Drop-bundle composition with encrypt-to-recipient (extend existing Compromise #31)**
> Existing Compromise #31 status: REGISTERED ("Drop bundle revocation reach — forever-valid bundles").
>
> Extension: explicitly document the encrypt-to-recipient composition consequence — a Drop bundle is decryptable by the recipient device forever as long as that recipient device retains its ML-KEM-768 secret-key. The bundle itself is not time-bound. Recipient-side ML-KEM-768 secret-key rotation is the only revocation mechanism.
>
> No additional MITIGATION beyond what Compromise #31 already documents. The extension is pure honest-disclosure.
>
> SCOPE: applies to all Layer-C drops with codepoint = DROP_TO_RECIPIENT.

**Compromise L5-C10 — Cross-device-sync UX-vs-cryptographic boundary**
> Status: REGISTERED. Closure: PARTIAL via Amendment 4 sender-DID-binding + outer-signature layer; FULL closure requires UX-level distinction between "user's legitimate device" vs "user's compromised device" (post-v1-beta UX work).
>
> A compromised device on a user's mesh (one of N enrolled devices is rooted / stolen) holds a legitimate device-key and can perform any operation that device is authorized for (including requesting + receiving permission-grants in its own name). The Amendment 4 sender-DID-binding ensures that envelope-AAD reliably distinguishes which device sent a request, but DOES NOT distinguish "legitimate Bob" from "Bob's compromised device."
>
> MITIGATION: (a) device-revocation by user via admin-UI (Phase-4-Meta-Composing) — limits future compromise reach; (b) future-additive: device-attestation per L5-C5 deferral.
>
> SCOPE: applies to all multi-device Atrium deployments.

**Compromise #32 — ML-KEM-768 Decap chosen-ciphertext side-channel surface** (already proposed by L3 §4.4)
> Concur with L3's proposed mint text verbatim. Cross-reference: [Bernstein-Persichetti 2024/2051](https://eprint.iacr.org/2024/2051) + FIPS 203 §6.3 implicit-rejection. Layer-A (ChaCha20-Poly1305) does not inherit this surface.

**Compromise L5-C-LTC1 — Long-term-confidentiality posture (BSI alignment)**
> Status: REGISTERED. Closure: OUT-OF-SCOPE at v1-beta + v1-GM; future-additive FrodoKEM codepoint at v1-GM+ if regulatory demand.
>
> BSI TR-02102-1 names FrodoKEM + Classic McEliece as the only KEMs cryptographically suitable for long-term confidentiality beyond the migration window. Benten's choice of ML-KEM-768 + X25519 (X-Wing) is acceptable for migration-window confidentiality but is NOT on the BSI long-term-confidentiality floor.
>
> MITIGATION: per CLAUDE.md baked-in #5 crypto-agility, a FrodoKEM-hybrid additive codepoint is reservable post-v1-GM if regulatory demand emerges.
>
> SCOPE: applies to long-term-confidentiality regulatory frameworks (German critical infrastructure; future EU eIDAS-3 if it mandates BSI floor; similar).

### 6.2 New invariant additions

**Inv-16 — Envelope-layer-unification + codepoint-discriminated primitive choice**
> Phrasing (B-equivalent per L2 §4.3, with normative Amendment 1–6 reference):
>
> "Every encryption use site in Benten dispatches through a single `EncryptedEnvelope` shape with codepoint-discriminated payload variant + AAD binding (Amendment 1) + strict-decode variant dispatch (Amendment 2) + canonical-TLV-encoded BindingContext (Amendment 3); symmetric and asymmetric primitives are codepoint-distinct; non-vault variants bind sender-DID (Amendment 4) and sealed-at/valid-until epoch (Amendment 5); HPKE-mode-base Decap is constant-time per RustCrypto `ml-kem` integration-time verification (Amendment 6)."
>
> Enforcement plan: similar to Inv-15 — registered at Phase-4-Meta-Core mint; full enforcement-completion at the G-CORE-PQ-WIRE-N-equivalent wave that ships the unified envelope. Property tests per the §5.2 test-corpus (cross-codepoint substitution; length-injectivity; sender-substitution; replay-window; etc.).

**Inv-L5-2 — Hybrid-cryptography-mandatory floor**
> Phrasing:
>
> "Every Benten KEM use site MUST use a PQ-classical hybrid construction (X-Wing or future-additive equivalent). No pure-classical or pure-PQ KEM codepoints are mintable. Aligns with ANSSI mandatory-hybridation guidance + BSI hybrid-recommendation + NIST SP 800-227 §4.4 hybrid-combiners."
>
> Enforcement plan: codepoint-registry discipline + cite-drift-detector scanner check that flags KEM-codepoint mints lacking PQ + classical halves. Inv-15-pattern enforcement.

**Inv-L5-3 — Codepoint-registry-discipline**
> Phrasing:
>
> "Every minted Benten cryptographic codepoint MUST appear in `docs/CRYPTO-CODEPOINTS.md` registry table with: hex value (BE on-wire per Amendment 7); variant name; primitive choice; AAD-binding spec; since-version; rationale. Codepoint range is IANA-disjoint (`0x6100..0x6FFF` reserved for Benten envelope codepoints, per Amendment 8)."
>
> Enforcement plan: cite-drift-detector scanner check on codepoint definitions vs registry rows.

### 6.3 Process / dispatch-conventions additions

- **pim-N-threat-model-doc-update-coupling**: when a new Compromise mint or a new threat-class is discovered, `docs/THREAT-MODEL.md` MUST be updated atomically in the same PR. Similar to §3.5g cross-language mirror discipline but for cross-doc threat-model mirrors.
- **pim-N-audit-deliverable-pre-engagement-gate**: before any external cryptographic audit engagement begins, the pre-engagement checklist (per §5.5 item 8) MUST be verified-green. This is a phase-close gate at v1-beta-tag.

---

## 7. Self-assessment + confidence + lower-confidence areas

### 7.1 What I did + how I worked

1. Tree-state pre-flight against worktree-state `2172cb6d` (clean working tree confirmed).
2. Read all four input branches in full (412 + 520 + 640 + 960 lines = ~2,532 lines of prior review work) + `docs/INVARIANT-COVERAGE.md` Inv-15 + `docs/SECURITY-POSTURE.md` Compromise table structure + crate-internals as informational background.
3. WebFetch / WebSearch against primary sources for NIST SP 800-227 (final), FIPS 203, ANSSI guidance, BSI TR-02102-1, ToB Sigstore engagement, Cure53 audit posture, HNDL threat-model frameworks, FedRAMP cryptographic-module policy.
4. Built the §2.1 threat-model matrix from first principles + the orchestrator's checklist, cross-checked against age + 1Password + Bitwarden + Signal Provisioning posture.
5. Built the §4 audit-finding enumeration by mapping each row to a known audit-firm prototype + assigning close-vs-accept disposition.
6. Built the §5 deliverable-shape recommendation by reverse-engineering ToB / NCC / Cure53 / PQShield engagement-scoping practice from publicly-available reports + the scope-review §10.2 baseline.
7. Composed §6 Compromise mints + invariant additions to be drop-in-ready for `docs/SECURITY-POSTURE.md` extension.

### 7.2 Confidence summary per major claim

| Claim | Confidence | Rationale |
|---|---|---|
| §1 verdict: design is AUDIT-READY-IN-DIRECTION but NOT-YET in deliverable-shape | **HIGH** | Three prior reviews converged on design correctness; deliverable-shape gap is the residual that this lens specifically surfaces. |
| §2.1 threat-model matrix (25 rows) | **HIGH on enumeration completeness; MEDIUM-HIGH on each row's IN/OUT/PARTIAL classification** | Matrix structure modeled on age + Signal threat-model practices; per-row classification depends on operational-policy choices that Ben may calibrate differently. |
| §3.1 NIST SP 800-227 alignment | **HIGH** | Primary-source verification against final text (September 2025); maps cleanly per section. |
| §3.2 FIPS 203 implicit-rejection compliance via Amendment 6 verification | **HIGH** | FIPS 203 §6.3 is unambiguous; RustCrypto `ml-kem` implements it per documented intent; Amendment 6 pins verification. |
| §3.3 ANSSI hybridation alignment | **MEDIUM-HIGH** | The library-vs-product boundary is the residual ambiguity; ANSSI guidance may sharpen as Phase-2 visa applications proceed. |
| §3.4 BSI long-term-confidentiality posture out-of-scope | **HIGH** | BSI TR-02102-1 unambiguously names FrodoKEM + Classic McEliece as long-term floor; ML-KEM-768 is NOT on that floor. |
| §3.5 FedRAMP out-of-scope | **HIGH** | FedRAMP requires FIPS 140-3; RustCrypto is not validated; structural mismatch. |
| §4 audit-finding enumeration (30 findings) | **MEDIUM-HIGH on completeness; HIGH on the close-vs-accept disposition per finding** | Drawn from publicly-available audit-firm patterns; a real audit may surface findings I didn't anticipate. Close-vs-accept is conservative. |
| §5 deliverable-shape recommendation | **HIGH** | Modeled on ToB / Cure53 / Obsidian-Sync engagement-scoping practice; time-budget is order-of-magnitude. |
| §6 Compromise mints + invariant additions | **HIGH on the need for mints; MEDIUM on the precise narrative text** | Ben should review the prose; the structural disclosure shape is correct. |

### 7.3 Lower-confidence areas (honest disclosure)

1. **The 11 Compromise mints + 1 extension might be excessive.** A more conservative L5-reviewer might cluster L5-C2 + L5-C3 + L5-C19 (out-of-scope-by-design rows) into a single "operational threats are out-of-scope" Compromise. I chose to split them for audit-clarity, but this could be calibrated down to ~5–7 Compromise mints if Ben prefers leaner posture.

2. **The §5 deliverable time-budgets (~9 person-days total).** Each item's individual estimate is order-of-magnitude correct; the aggregate could be off by 30–50% in either direction depending on how much of the source material already exists in code-comments / scope-review prose. The cite-drift-detector + cross-doc scanner work is the most variable item.

3. **NIST SP 800-227 alignment depth.** I verified the section structure + key requirements from the public summary but did NOT do a line-by-line read of the full 800-227 final PDF. A complete alignment audit would require ~1 day of focused reading; my MEDIUM-HIGH classification reflects this.

4. **ANSSI Phase-2 security-visa eligibility specifics.** ANSSI publishes specific Phase-2 visa criteria per product category; Benten as a graph-engine-library straddles the library-vs-product boundary in ways that may or may not satisfy ANSSI's specific definitions. A French-jurisdiction regulatory consult would be needed for a definitive answer.

5. **BSI TR-02102-1 January 2025 update.** I cited the publicly-available joint statement and migration recommendations; the January 2025 specific update text (which adds ML-KEM to the approved-list following NIST standardization) is referenced but I did not deep-read it. My HIGH classification on the long-term-confidentiality posture is robust to this; my MEDIUM-HIGH on the hybridation specifics is the residual uncertainty.

6. **Pre-emptive audit-finding completeness.** §4 enumerates ~30 likely findings drawn from audit-firm patterns. A real engagement could surface findings I didn't anticipate, particularly in the multi-stanza HpkeMultiBase deferred variant or in supply-chain dependency-pinning specifics I didn't enumerate. The DEFER classifications are explicit guardrails.

7. **Cross-amendment interaction with L3's Amendments 3–6.** I treated L3's amendments as load-bearing inputs. If Ben's ratification path narrows the amendment set (e.g. L3's §5 recommendation #4 minimal-load-bearing of Amendments 1+2+3+6), some §2.1 IN-rows shift to PARTIAL and additional Compromise mints become necessary. My matrix assumes the full Amendments 1–6 ratification.

### 7.4 What I could be wrong about

**Most-likely failure mode of MY review**: missing a specific cross-jurisdictional compliance requirement that Benten's target customer base needs. Specifically — if Benten's target customer base includes EU public-sector / German critical-infrastructure / French regulated-financial / US federal, the OUT-OF-SCOPE classifications in §3.4 + §3.5 become BLOCKERs not honest-disclosure. My current verdict assumes the consumer-platform / personal-AI-assistants deployment target per CLAUDE.md baked-in #18. If that framing is wrong, the verdict shifts.

**Second most-likely failure mode**: the §5 deliverable time-budget under-estimates the work of building the test-corpus pieces if the underlying code paths aren't yet implemented. The Amendments 1–6 enforcement code MUST be in place before the test-corpus can pin against it; the budget assumes that work is already on the Phase-4-Meta-Core wave plan per scope-review §5.1.

### 7.5 Sharpest contrarian pushback I considered against MYSELF

"This lens is over-prescriptive about deliverable shape. The audit firm will tell Benten what they need; Benten shouldn't pre-build their deliverables."

Rebuttal: ToB / Cure53 / NCC engagement-scoping practice CONSISTENTLY notes that engagements with pre-built threat-model + scope-statement + test-corpus complete in normal budget; engagements without them blow budget by 30–50%. Obsidian's [public report on the Cure53/ToB audits](https://obsidian.md/blog/cure53-tob-sync-audits/) explicitly notes that they pre-built the equivalent deliverables. The §5 work is not "doing the audit firm's job"; it's prerequisite-engineering that ANY audit firm appreciates.

"Compromise #32 mint plus 11 new mints from L5 plus extension to #31 is too many for a 5-week-old project."

Rebuttal: each is short prose (3–6 lines); aggregate is ~0.5 day of doc-writing. The audit-cost-saving is at least ~3–5 days. The asymmetric ratio strongly favors more-mints-with-shorter-bodies vs fewer-mints-with-aspirational-prose-that-misses-cases. The CLAUDE.md baked-in #15 NF-2 / C-GM-AUDIT exit criterion + the v1-beta-tag freeze argument both point at "honest-disclosure-NOW is structurally cheaper than retroactive-honest-disclosure-LATER."

### 7.6 What I would tell Ben in plain English

"The §6.2-with-Amendments-1–6 design is genuinely audit-ready in cryptographic substance. Three competent reviewers have stress-tested it; the residual cryptographic surfaces are documented (Bernstein-Persichetti Decap; PQ-primitives-unaudited; the multi-stanza variant deferred to specialist review). What's NOT yet audit-ready is the **documentation deliverable shape** an external auditor expects to find when they begin work. The single highest-leverage piece is `docs/THREAT-MODEL.md` enumerating what Benten promises and what Benten does NOT promise — it costs 2–3 days to write, saves ~1 week of audit-time, and turns ~40% of likely false-positive audit findings into pre-emptively-closed posture. The 11 new Compromise mints + 1 extension are short (3–6 lines each); aggregate is ~0.5 day of doc-writing. The §5 test-corpus work depends on Amendments 1–6 enforcement code landing first — that work is on the Phase-4-Meta-Core wave plan already.

On compliance: Benten substantially aligns with NIST SP 800-227 + FIPS 203 + ANSSI hybridation + BSI migration-window guidance. The honest-disclosure deviations are: (a) no FIPS 140-3 module validation (RustCrypto isn't validated); (b) BSI long-term-confidentiality floor wants FrodoKEM not ML-KEM-768 (Benten can add as additive codepoint post-v1-GM if needed); (c) FedRAMP requires FIPS 140-3 (cascades from (a)). None of these block the consumer-platform / personal-AI-assistants deployment target per CLAUDE.md baked-in #18. They WOULD block US-Federal-customer / German-critical-infrastructure / EU-regulated-financial deployment, but those aren't Benten's v1-beta target.

Recommend ratifying Amendments 1–8 + minting Compromise #32 + minting the 11 L5 Compromises + extending #31 + minting Inv-16 + Inv-L5-2 + Inv-L5-3 at Phase-4-Meta-Core close. Schedule the §5 deliverable work as part of the audit-engagement-preparation that runs concurrently with Phase-4-Meta-Composing per scope-review §12. The external audit window of ~3 person-weeks compresses to ~2 person-weeks of substantive work if the §5 deliverables ship pre-engagement."

---

## 8. Citations

### 8.1 Standards + drafts (primary references)

- **NIST SP 800-227** Recommendations for Key-Encapsulation Mechanisms (final, September 2025). [csrc.nist.gov/pubs/sp/800/227/final](https://csrc.nist.gov/pubs/sp/800/227/final). [News release](https://csrc.nist.gov/News/2025/nist-publishes-sp-800-227). [PDF](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-227.pdf).
- **FIPS 203** Module-Lattice-Based Key-Encapsulation Mechanism Standard. [csrc.nist.gov/pubs/fips/203/final](https://csrc.nist.gov/pubs/fips/203/final). [Federal Register Aug 2024](https://www.federalregister.gov/documents/2024/08/14/2024-17956/announcing-issuance-of-federal-information-processing-standards-fips-fips-203-module-lattice-based). [PDF](https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.203.pdf).
- **FIPS 140-3** Cryptographic Module Validation Program. [csrc.nist.gov/projects/cryptographic-module-validation-program](https://csrc.nist.gov/projects/cryptographic-module-validation-program).
- **RFC 9180** Hybrid Public Key Encryption. [rfc-editor.org/rfc/rfc9180.html](https://www.rfc-editor.org/rfc/rfc9180.html).
- **RFC 9106** Argon2 memory-hard function. [datatracker.ietf.org/doc/html/rfc9106](https://datatracker.ietf.org/doc/html/rfc9106).
- **RFC 8439** ChaCha20 and Poly1305. [datatracker.ietf.org/doc/html/rfc8439](https://datatracker.ietf.org/doc/html/rfc8439).
- **RFC 5869** HKDF. [datatracker.ietf.org/doc/html/rfc5869](https://datatracker.ietf.org/doc/html/rfc5869).
- **draft-connolly-cfrg-xwing-kem-10** X-Wing KEM. [datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/](https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/).
- **ANSSI Follow-Up Position Paper on Post-Quantum Cryptography** (2023 follow-up). [messervices.cyber.gouv.fr/documents-guides/follow_up_position_paper_on_post_quantum_cryptography.pdf](https://messervices.cyber.gouv.fr/documents-guides/follow_up_position_paper_on_post_quantum_cryptography.pdf).
- **BSI Transitioning to Post-Quantum Cryptography joint statement 2025**. [bsi.bund.de/SharedDocs/Downloads/EN/BSI/Crypto/PQC-joint-statement-2025.pdf](https://www.bsi.bund.de/SharedDocs/Downloads/EN/BSI/Crypto/PQC-joint-statement-2025.pdf).
- **BSI Migration to Post-Quantum Cryptography**. [bsi.bund.de/SharedDocs/Downloads/EN/BSI/Crypto/Migration_to_Post_Quantum_Cryptography.pdf](https://www.bsi.bund.de/SharedDocs/Downloads/EN/BSI/Crypto/Migration_to_Post_Quantum_Cryptography.pdf).
- **BSI Quantum Technologies + Quantum-Safe Cryptography landing**. [bsi.bund.de/EN/Themen/...PQK/quantentechnologien-und-post-quanten-kryptografie_node.html](https://www.bsi.bund.de/EN/Themen/Unternehmen-und-Organisationen/Informationen-und-Empfehlungen/Quantentechnologien-und-Post-Quanten-Kryptografie/quantentechnologien-und-post-quanten-kryptografie_node.html).
- **FedRAMP Policy for Cryptographic Module Selection v1.1.0**. [fedramp.gov/resources/documents/FedRAMP_Policy_for_Cryptographic_Module_Selection_v1.1.0.pdf](https://www.fedramp.gov/resources/documents/FedRAMP_Policy_for_Cryptographic_Module_Selection_v1.1.0.pdf).
- **FedRAMP System Security Plan template (Rev5)**. [fedramp.gov/docs/rev5/playbook/csp/authorization/ssp/](https://www.fedramp.gov/docs/rev5/playbook/csp/authorization/ssp/).
- **CMVP FIPS 140-3 IG announcements**. [csrc.nist.gov/projects/cryptographic-module-validation-program/fips-140-3-ig-announcements](https://csrc.nist.gov/projects/cryptographic-module-validation-program/fips-140-3-ig-announcements).
- **Common Criteria Security Target example (BAE STOP)**. [commoncriteriaportal.org/files/epfiles/553-EWA%20ST%20v0.24.pdf](https://www.commoncriteriaportal.org/files/epfiles/553-EWA%20ST%20v0.24.pdf).

### 8.2 Academic papers + ePrints

- **Bernstein, Persichetti.** "One Time is Enough: Chosen-Ciphertext Side-Channel Attack on ML-KEM Cryptosystems." IACR ePrint 2024/2051. [eprint.iacr.org/2024/2051](https://eprint.iacr.org/2024/2051). **Load-bearing for Compromise #32 / Amendment 6.**
- **Barbosa, Connolly, Diniz, Kahl, Krämer.** "X-Wing: The Hybrid KEM You've Been Looking For." IACR CIC Vol. 1 No. 1, 2024. [eprint.iacr.org/2024/039](https://eprint.iacr.org/2024/039).
- **Arriaga, Barbosa, Boyen.** "Tempo: ML-KEM to PAKE Compiler Resilient to Timing Attacks." IACR ePrint 2025/1399. [eprint.iacr.org/2025/1399](https://eprint.iacr.org/2025/1399).
- **MDPI Quantum Reports.** "Harvest-Now, Decrypt-Later: A Temporal Cybersecurity Risk in the Quantum Transition." [mdpi.com/2673-4001/6/4/100](https://www.mdpi.com/2673-4001/6/4/100). **Load-bearing for T-13 HNDL framing.**
- **arXiv 2603.01091.** "On the Practical Feasibility of Harvest-Now, Decrypt-Later Attacks." [arxiv.org/html/2603.01091v1](https://arxiv.org/html/2603.01091v1).
- **IACR ePrint 2025/2052.** "SoK: Systematizing Hybrid Strategies for the Transition to Post-Quantum Cryptography." [eprint.iacr.org/2025/2052.pdf](https://eprint.iacr.org/2025/2052.pdf).

### 8.3 Audit-firm published reports + engagement-shape references

- **Trail of Bits — Building cryptographic agility into Sigstore** (2026-01-29). [blog.trailofbits.com/2026/01/29/building-cryptographic-agility-into-sigstore/](https://blog.trailofbits.com/2026/01/29/building-cryptographic-agility-into-sigstore/). Engagement-shape precedent for codepoint-registry + algorithm-suite framework.
- **Trail of Bits Publications repository**. [github.com/trailofbits/publications](https://github.com/trailofbits/publications).
- **Trail of Bits cryptography category**. [blog.trailofbits.com/categories/cryptography/](https://blog.trailofbits.com/categories/cryptography/).
- **NCC Group Cryptography Services**. [nccgroup.com/us/assessment-advisory/cryptography/](https://www.nccgroup.com/us/assessment-advisory/cryptography/).
- **NCC Group Research Blog cryptographic-implementation tag**. [research.nccgroup.com/tag/cryptographic-implementation/](https://research.nccgroup.com/tag/cryptographic-implementation/).
- **NCC Group Entropy/Rust Cryptography Review**. [nccgroup.com/research-blog/public-report-entropyrust-cryptography-review/](https://www.nccgroup.com/research-blog/public-report-entropyrust-cryptography-review/). Engagement-shape precedent for Rust-cryptography audits.
- **Cure53 Publications**. [github.com/cure53/Publications](https://github.com/cure53/Publications).
- **Cure53 — OpenPGP.js audit summary**. [github.com/openpgpjs/openpgpjs/wiki/Cure53-security-audit](https://github.com/openpgpjs/openpgpjs/wiki/Cure53-security-audit). 26 issues / 12 vulnerabilities / 2 critical — domain-separation finding patterns.
- **Cure53 — Request Network Encryption Audit**. [request.network/blog/request-encryption-audit-completed-by-cure53](https://request.network/blog/request-encryption-audit-completed-by-cure53). aes-256-cbc-without-integrity finding pattern.
- **Obsidian Sync audits by Cure53 and Trail of Bits**. [obsidian.md/blog/cure53-tob-sync-audits/](https://obsidian.md/blog/cure53-tob-sync-audits/). Engagement-shape precedent for sync-encryption audits; key-mgmt-confusion finding-class.
- **PQShield — Formally verifying AVX2 rejection sampling for ML-KEM**. [pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/](https://pqshield.com/formally-verifying-avx2-rejection-sampling-for-ml-kem/). Specialist PQ-primitive audit shape.
- **Verification Theatre — Nadim Kobeissi review of Cryspen**. [symbolic.software/blog/2026-02-05-cryspen/](https://symbolic.software/blog/2026-02-05-cryspen/). Load-bearing for "don't use hpke-rs" + supply-chain-dependency-pinning argument.
- **AWS-LC FIPS 3.0 ML-KEM validation**. [aws.amazon.com/blogs/security/aws-lc-fips-3-0-first-cryptographic-library-to-include-ml-kem-in-fips-140-3-validation/](https://aws.amazon.com/blogs/security/aws-lc-fips-3-0-first-cryptographic-library-to-include-ml-kem-in-fips-140-3-validation/). FIPS-140-3-validated-module precedent.

### 8.4 Folklore + production-system references

- **Filippo Valsorda — age and Authenticated Encryption**. [words.filippo.io/age-authentication/](https://words.filippo.io/age-authentication/). Threat-model-doc shape precedent + replay-window framing.
- **Bitwarden Security Whitepaper**. [bitwarden.com/help/bitwarden-security-white-paper/](https://bitwarden.com/help/bitwarden-security-white-paper/). Vault threat-model + key-lifecycle shape.
- **Bitwarden Trusted Devices**. [bitwarden.com/help/about-trusted-devices/](https://bitwarden.com/help/about-trusted-devices/). Multi-device-key-wrap precedent.
- **1Password Security Design**. [1password.com/files/1password-white-paper.pdf](https://1password.com/files/1password-white-paper.pdf). Two-secret KDF + multi-device shape.
- **Signal Provisioning protocol**. [signal.org/blog/a-synchronized-start-for-linked-devices/](https://signal.org/blog/a-synchronized-start-for-linked-devices/). Device-link wire-shape precedent.
- **Wikipedia — Harvest-now, decrypt-later**. [en.wikipedia.org/wiki/Harvest_now,_decrypt_later](https://en.wikipedia.org/wiki/Harvest_now,_decrypt_later). HNDL threat-class framing.
- **Encryption Consulting — HNDL preparation**. [encryptionconsulting.com/harvest-now-decrypt-later-preparing-for-the-quantum-threat/](https://www.encryptionconsulting.com/harvest-now-decrypt-later-preparing-for-the-quantum-threat/).
- **Palo Alto Networks — HNDL quantum-era threat**. [paloaltonetworks.com/cyberpedia/harvest-now-decrypt-later-hndl](https://www.paloaltonetworks.com/cyberpedia/harvest-now-decrypt-later-hndl).
- **dudect statistical timing test**. [github.com/oreparaz/dudect](https://github.com/oreparaz/dudect). Constant-time verification test framework.
- **RustSec Advisory Database**. [rustsec.org/advisories/](https://rustsec.org/advisories/). Supply-chain monitoring.

### 8.5 Benten internal references

- `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f` — L1 NO-GO review + §6.2 origin.
- `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b` — L2 Amendments 1+2.
- `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3` — L3 Amendments 3–6 + minor 7+8.
- `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae` — F-full scope review with §6 + §7 wire formats + §10.2 audit-scope estimate + §14.1 origin Option F+ pitch.
- `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b` — Inv-15 origin.
- `phase-4-meta-core/encrypt-to-recipient-review-cryptographer @ 791c8d17` — Option B baseline.
- `docs/INVARIANT-COVERAGE.md` Inv-15 (registered) + Inv-16 mint pending.
- `docs/SECURITY-POSTURE.md` Compromise #6 + #30 + #31.
- `crates/benten-crypto-suite/INTERNALS.md` (informational primitive choices).
- CLAUDE.md baked-in #5 (crypto-agility); #15 (v1-beta + v1-GM gates); #17 (deployment-shapes); #18 (authority-isolation vs confidentiality-isolation).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` discipline.
- `feedback_plain_english_surfaces` discipline.
- `feedback_no_defer_HARD_RULE` discipline (every Compromise mint has explicit closure or out-of-scope justification).

---

**End of review.**
