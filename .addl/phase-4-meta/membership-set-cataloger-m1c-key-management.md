# Membership-Set Cataloger M1c — Key-Management Primitives (existing state + plans)

> Cataloger M1c of 3 parallel (M1a multi-device-sync; M1b Atrium-membership-sharing; **M1c key-management = this doc**).
> Authored 2026-05-27 against main HEAD `2172cb6d` from branch
> `phase-4-meta-core/membership-set-cataloger-m1c-key-management`.
> Audience: MembershipSet unification specialist panel (M2-M6) reading
> a foundational existing-state + plans inventory of Benten's
> cryptographic key chain.
>
> ⚠️ Scope discipline. Key-management primitives ONLY (K_principal,
> DAK, K(N), K_Atrium, codepoint registry, hybrid sig, KDF chain,
> AEAD wrap surface). Multi-device-sync UX flow + Atrium membership
> sharing surfaces flagged briefly and DEFERRED to M1a / M1b.

---

## §1 — Executive summary

Benten ships **one Benten-owned crypto-suite integration crate**
(`benten-crypto-suite`, ~6.0k LOC across 14 files) per CLAUDE.md
baked-in #5. Every other crate that hashes / signs / verifies /
derives a key / wraps an AEAD ciphertext routes through this crate's
*typed* APIs (the "only-crypto-primitive call-site" rule).

**Current state at main HEAD `2172cb6d` (the load-bearing surfaces):**

1. **K_principal — STUB** at
   `crates/benten-graph/src/redb_backend.rs::derive_test_seam_key_from_cid_with_namespace`
   (~lines 117-210). Synthesized deterministically from
   `(namespace_did, K_PRINCIPAL_DOMAIN_KEY)` via BLAKE3 keyed-hash;
   `_test_seam_` is load-bearing in the symbol name; ANY party
   holding `(namespace_did, ciphertext_blob)` can decrypt. Named for
   replacement at `docs/future/phase-4-backlog.md §3.10` G-CORE-3e
   K_principal-per-DID secret store.

2. **Structural KDF chain — BUILT** at
   `crates/benten-crypto-suite/src/structural_kdf.rs` (Spike-E
   Interpretation-B, path-tagged): `derive_root(K_principal, root_cid,
   cipher_suite_codepoint)` + `derive_step(predecessor, edge_label,
   N.cid)`. The R6 R2 batch-A Item 7 (D-13 closure) extension binds
   the cipher-suite codepoint into the `K(root)` info-tag, closing
   the cross-codepoint key-reuse attack class.

3. **AEAD primitive — BUILT** at `crates/benten-crypto-suite/src/aead.rs`
   (ChaCha20-Poly1305; format-version byte + magic + LE codepoint +
   nonce + tag). Per-Node graph-layer wrap at
   `crates/benten-graph/src/aead_wrap.rs` (whole-content < 64 KiB
   vs per-chunk @ 16 KiB IROH_BLOCK_SIZE).

4. **Cipher suite codepoint registry — BUILT + FROZEN** at
   `crates/benten-crypto-suite/src/codepoint.rs`; FROZEN at
   G-CORE-9 V1-FROZEN-INTERFACE.md item 6. 11 codepoints across hash
   + sig + cipher + (no-encryption) tables; live arms `0x647a` /
   `0x6400`; reserved-typed-reject `0x647b` / `0x647c` / `0x0000` /
   `0x0003`.

5. **X-Wing combiner — BUILT but MISLABELED** at
   `crates/benten-crypto-suite/src/cipher_suite.rs::x_wing_combine`
   (~lines 401-438). Currently uses `HKDF-SHA256(SHA3-256-salt,
   "x-wing-v1-benten-0x647a")`; this is NOT real X-Wing math. Real
   X-Wing per `draft-irtf-cfrg-concrete-hybrid-kems-03` (RG-adopted
   under the name **`MLKEM768-X25519`**) is `SHA3-256(label || ss_M
   || ss_X || ct_X || pk_X)` with a specific 6-byte ASCII label.
   ~24 LOC corrective is **pre-v1-beta-tag-must-fix** REGARDLESS of
   F-full outcome (per cryptographer-review + `x-wing-to-mlkem768-
   x25519-rename-audit.md`).

6. **Hybrid signature primitive — BUILT** at
   `crates/benten-crypto-suite/src/sig.rs` (Ed25519⊕LAMPS Composite
   ML-DSA `id-MLDSA65-Ed25519-SHA512` at `SigCodepoint::HYBRID_
   ED25519_MLDSA65 = 0x0001`); Inv-15 application-layer
   3-layer-decomposition closes the LAMPS EUF-CMA-only-not-SUF-CMA
   gap per Compromise #31.

7. **HPKE primitive — NOT BUILT.** Ratified for v1-beta as
   HPKE-mode-base[MLKEM768-X25519] per Option F+ pseudo-keypair
   review (returned 2026-05-27 LATE-SESSION; NO-GO on
   pseudo-keypair; rationale: Tempo-paper IACR 2025/1399 side-channel
   risk against ML-KEM-768 KeyGen-from-secret-seed via SampleNTT
   rejection-sampling timing). Tactical lib pick: Brendan McMillion
   `hpke` crate (NOT Cryspen `hpke-rs` with 13 CVEs Feb 2026).

8. **DAK (Device Authorization Key) — NOT BUILT.** F-full Layer-D.
   Password-derived via Argon2id; tier parameters open. Pre-v1-beta
   scope per Ben 2026-05-27 ratification.

9. **K_Atrium — NOT BUILT.** Pending Ben ratification under
   MembershipSet framing (Q3 Option D semantics: CSPRNG-random on
   Atrium-creation; multi-stanza-HPKE distribution at member-join;
   FORK-ONLY rotation; preserves CGKA-deferral).

**Architectural arc** (load-bearing for unification analysis):

- F-full = 4-layer architecture (A: K_principal vault under DAK; B:
  per-Node AEAD with K(N); C: encrypt-to-recipient HPKE +
  multi-stanza groups; D: DAK + cross-platform credential-store +
  multi-device key-wrap + remote-permission-call).
- CGKA / MLS-PQ DEFERRED past v1-beta (Atrium-forkability semantics
  ≠ messaging-leave-forgets; multi-stanza-HPKE = v1-beta group
  fallback; CGKA codepoint reserved for when MLS-PQ matures).
- Bird-of-Prey-class SUF-CMA-preserving combiners RESERVED as
  future-additive codepoints when WG-adopted + impl-audited.
- Pure-PQ-only arms (`0x647c`, `0x0003`) gated by
  `AUDIT_LANDED_PURE_PQ_FLAG` compile-time `false`; flips at
  NF-2 / C-GM-AUDIT independent-audit landing for v1-GM.

---

## §2 — Current-state key-chain inventory (file-by-file at main HEAD `2172cb6d`)

### §2.1 Type-level surface (`benten-crypto-suite`)

| Type | File | Status | Notes |
|---|---|---|---|
| `StructuralKdfKey` | `src/structural_kdf.rs:55` | LIVE | 32-byte HKDF-SHA256 PRK newtype; `Zeroize` on drop; `Clone`. Production constructor crate-private. Public test-helper `from_bytes_for_test`. |
| `AeadKeyMaterial` | `src/aead.rs:85` | LIVE | Carries `CipherSuiteCodepoint` + opaque `Vec<u8>` key. Renamed at G-CORE-3c from test-named API (`from_bytes_for_test`) to `from_raw_bytes` after call-site identified production use. Zeroizes on drop. |
| `GrantKeyMaterial` | `benten-caps` | LIVE | Distinct from `AeadKeyMaterial` per V1-FROZEN-INTERFACE item 15(d) (`#1344` rename — prevents compatible-interpretation trap). Lives in caps to carry grant-bound semantics. |
| `AeadEnvelope` | `src/aead.rs:145` | LIVE | wire-bytes envelope: `magic 0xae` + `format_version: u8` (= `0x01`) + `cipher_codepoint: u16` (LE) + `nonce_len` + nonce + ciphertext-with-tag. |
| `WrappedKey` | `src/cipher_suite.rs:538` | LIVE | encapsulated keys (X25519 ephemeral pub + ML-KEM-768 ct) + AEAD-encrypted K_root payload. |
| `RecipientKeypair` / `RecipientPublic` / `RecipientSecret` | `src/cipher_suite.rs:458+` | LIVE | Hybrid keypair carrying X25519 + (optional) ML-KEM-768 halves; adversarial-helper `with_only_classical_half_for_test` for F-3 `RecipientLacksKeysForSuite` arm. |
| `SignatureSuite` / `SuiteConfig` / `HybridSignature` | `src/sig.rs` (548 LOC) | LIVE | Trait + dispatch; Ed25519⊕ML-DSA-65 hybrid; both-must-verify strip-resistant via `hybrid_combine` committing combiner. |
| `HashSeam` | `src/hash.rs` (81 LOC) | LIVE | BLAKE3 default + SHA-512/256 + SHA3-256 pre-blessed agile fallbacks. |
| `SwapMatrix` (private `SignatureArm` + `EncryptionArm`) | `src/swap_matrix.rs` (2014 LOC) | LIVE | Full bidirectional swap matrix (9 default+downgrade combinations); `try_pure_pq_sole_trust_path` audit-gated constructor returns `SwapMatrixError::AuditNotLandedPurePqRejected` while `AUDIT_LANDED_PURE_PQ_FLAG = false`. |

### §2.2 K_principal STUB site

`crates/benten-graph/src/redb_backend.rs:117-210` (function
`derive_test_seam_key_from_cid_with_namespace`):

```rust
const K_PRINCIPAL_DOMAIN_KEY: [u8; 32] = *b"benten/g-core-3e/k-principal-v1\0";
let k_principal_bytes = blake3::keyed_hash(&K_PRINCIPAL_DOMAIN_KEY, did_bytes);
let k_principal = StructuralKdfKey::from_bytes_for_test(k_principal_bytes.as_bytes());
let k_root = derive_root(&k_principal, cid.as_bytes(),
    CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw());
```

- **Confidentiality limit (called out in docstring + SECURITY-POSTURE):**
  `(namespace_did, ciphertext_blob)` is sufficient to decrypt at this
  wave; only namespace-isolation at the storage backend defends.
- The `_test_seam_` token is load-bearing in EVERY caller name until
  the K_principal-store backend lands. Replacement is 1-line — same
  `(namespace_did, plaintext_cid)` signature — at the K_principal
  synthesis step.

### §2.3 Cipher-suite codepoint table (FROZEN at G-CORE-9 V1-FROZEN-INTERFACE.md item 6)

| Class | Constant | Value | Status |
|---|---|---|---|
| Hash | `HashCodepoint::BLAKE3` | `0x1e` | LIVE default |
| Hash | `HashCodepoint::SHA2_512_256` | `0x1015` | LIVE agile fallback |
| Hash | `HashCodepoint::SHA3_256` | `0x16` | LIVE agile fallback |
| Sig | `SigCodepoint::HYBRID_ED25519_MLDSA65` | `0x0001` | LIVE v1-beta DEFAULT (LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512`; OID `1.3.6.1.5.5.7.6.48`; IANA early-allocated 2025-10-20) |
| Sig | `SigCodepoint::CLASSICAL_ED25519` | `0x0002` | LIVE non-default downgrade |
| Sig | `SigCodepoint::HYBRID_MLDSA65_SLHDSA` | `0x0003` | RESERVED-typed-reject; NF-1 PQ⊕PQ; gated by `AUDIT_LANDED_PURE_PQ_FLAG` |
| Cipher | `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` | `0x647a` | LIVE v1-beta DEFAULT (currently labeled "X-Wing"; needs rename to MLKEM768-X25519 + math corrective) |
| Cipher | `CipherSuiteCodepoint::CLASSICAL_X25519` | `0x6400` | LIVE non-default downgrade |
| Cipher | `CipherSuiteCodepoint::HYBRID_MLKEM768_HQC` | `0x647b` | RESERVED; NF-1 ML-KEM⊕HQC end-state (build-trigger = FIPS 207 final ~2027) |
| Cipher | `CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY` | `0x647c` | RESERVED-typed-reject; gated by `AUDIT_LANDED_PURE_PQ_FLAG` |
| Cipher | `CipherSuiteCodepoint::NONE_PLAINTEXT` | `0x0000` | RESERVED-typed-reject at dispatcher; the explicit no-encryption arm of the swap matrix |

**Endianness:** codepoints are encoded LE in `AeadEnvelope::to_wire_bytes`
+ in the structural-KDF info-tag (per `CipherSuiteCodepoint::raw()
.to_le_bytes()`). Lens L4 IMPL-A1 question of LE-vs-BE for wire
canonicalization was raised in the 9-eyes registry; **at HEAD the
choice is LE** (locked at V1-FROZEN-INTERFACE item 6 byte-pin).

### §2.4 KDF chain (LIVE at HEAD)

```
K_principal = (STUB: BLAKE3-keyed-hash(K_PRINCIPAL_DOMAIN_KEY,
                                       namespace_did.as_bytes()))
              [real: production K_principal-per-DID secret store —
               NOT BUILT]

K(root) = HKDF-SHA256(K_principal,
            info = "root:codepoint:" || codepoint_le_bytes || root_cid)

K(N) = HKDF-SHA256(K(predecessor),
            info = "step" || edge_label || N.cid)
```

**Path-tagged Spike-E Interpretation-B** is load-bearing: same Node
reached by different paths yields DIFFERENT K(N); selective-share
feature; structure-independent simplification disproved by Spike E
(DO NOT collapse to `HKDF(K_principal, N.cid)`).

**R6 R2 batch-A Item 7 / Row D-13 closure (2026-05-13):**
`derive_root` binds the `cipher_suite_codepoint` into the
info-tag → same `(K_principal, root_cid)` under different codepoints
yields DIFFERENT `K(root)` → cross-codepoint key-reuse attack class
closed at K_root.

### §2.5 AEAD layers

| Layer | File | Surface |
|---|---|---|
| Layer-Cryptosuite (per-codepoint AEAD seal/open) | `benten-crypto-suite/src/aead.rs::wrap` + `::unwrap` | `(plaintext, &AeadKeyMaterial, aad) → AeadEnvelope`; ChaCha20-Poly1305 only at HEAD; dispatch on `key.codepoint.raw()` matching `0x647a` / `0x6400` |
| Layer-Graph (per-Node wrap) | `benten-graph/src/aead_wrap.rs::encrypt` + `::decrypt` + `::encrypt_chunk` / `::decrypt_chunk` | Whole-content < 64 KiB → single envelope; ≥ 64 KiB → `ChunkedCiphertext` with 16-KiB chunks aligned to `iroh-blobs::IROH_BLOCK_SIZE`; `EncryptedNode::{Whole, Chunked}` enum |
| Layer-Recipe (DropBundle multi-Recipe) | `aad_per_recipe(plaintext_cid, recipe_index, total_recipes)` | R6 R2 fix-pass Bundle R6-R2-FP-A L4 sibling; closes inter-Recipe truncation attack |

**AAD shape (FROZEN at V1-FROZEN-INTERFACE item 15(g)):**

- Whole-content: `"benten-aead:whole:" || plaintext_cid`
- Per-chunk: `"benten-aead:chunk:" || plaintext_cid || chunk_index_le8 ||
  total_chunks_le4` (R6 R1 fix-pass Bundle F3 — supersedes 2-tuple;
  closes cross-chunk truncation attack)
- Per-Recipe: `"benten-aead:recipe:" || plaintext_cid || recipe_index_le4
  || total_recipes_le4`

### §2.6 Consumers of the crypto-suite (no direct primitive call sites elsewhere)

- `benten-id` — `did:key` minting + sig verify dispatch + Keypair (Ed25519 with `Zeroize + ZeroizeOnDrop` per crypto-blocker-1) + DeviceAttestation + RotationLog (in-RAM chain-walker)
- `benten-caps` — UCAN sig verify + `AuthorizationGrant.binding_sig` verify + `AeadEnvelope` wrap/unwrap for grant key-material
- `benten-graph` — per-Node AEAD wrap; `two_cid_map` (plaintext-CID → ciphertext-CID); K_principal STUB synthesizer
- `benten-drop` — `AuthorizationGrant` carrier + envelope-Ed25519 sig over bundle header
- `benten-engine` — `Engine::walk_share_scope` delegates to canonical walker; `AeadEnvelope` unwrap on read pathways
- `benten-sync` — AEAD wrap on outbound sync frames; MstDiff CIDs

---

## §3 — Planned-state key-chain inventory

### §3.1 Phase-4-Meta-Core (wire-format-affecting; pre-v1-beta-tag)

| Item | Source | Scope |
|---|---|---|
| **X-Wing→MLKEM768-X25519 rename + math corrective** | `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` + cryptographer-review | ~24 LOC code + naming sweep across Categories A (cross-ecosystem-boundary surfaces: SECURITY-POSTURE.md / V1-FROZEN-INTERFACE.md / ERROR-CATALOG.md / phase-4-backlog.md / errors.rs / errors.generated.ts) + B (internal Rust code) + D (working planning docs); Category C drafts held until F-full ratifies. Codepoint `0x647A` integer stays (matches IETF reservation). Required REGARDLESS of F-full. |
| **F-full Layer-A (K_principal store)** | NIGHT-SHIFT-2026-05-27 + Option F+ pseudo-keypair review NO-GO | Real K_principal at rest in encrypted form; NOT derivable-from-DID. **PULLED FORWARD into Phase-4-Meta-Core per Ben 2026-05-27.** ~500-1000 LOC. Vault = ChaCha20-Poly1305-AEAD-under-DAK (NOT pseudo-keypair-under-asymmetric; rejected per Tempo-paper IACR 2025/1399). |
| **F-full Layer-B (per-Node AEAD with K(N))** | Existing construction + ~24 LOC X-Wing→MLKEM768-X25519 corrective | Already mostly built; corrective shape == math change at `cipher_suite.rs::x_wing_combine` to `SHA3-256(label || ss_M || ss_X || ct_X || pk_X)` |
| **F-full Layer-C (encrypt-to-recipient HPKE)** | Combined-Option-F (3 e2r reviewers convergent 2026-05-26-27) | HPKE-mode-base[MLKEM768-X25519] + multi-stanza-HPKE for groups + CGKA-deferred + Inv-16 3-layer-decomposition mint. Library = Brendan McMillion `hpke` crate. ~1500-2500 LOC. NEW codepoint(s) at additive table positions. |
| **F-full Layer-D (DAK + cross-platform credential-store + multi-device key-wrap + remote-permission-call)** | NIGHT-SHIFT-2026-05-27 | DAK = Argon2id-derived from password; tier parameters per L10 U41 amendment. Cross-platform integration via `keyring-core` v1.0.0 (NOT legacy `keyring`). Remote-permission-call shape = Signal-Provisioning+CTAP-2.2 inspired. ~300-1500 LOC. **Remote-permission-call REQUIRED not cuttable** per Ben 2026-05-27. |
| **Inv-15 enforcement completion** | INVARIANT-COVERAGE.md + Compromise #31 | G-CORE-PQ-WIRE-1 wave bundles cross-surface audit (UCAN backend `revoke(ucan_cid)` + device attestation envelope + Atrium Drop bundles + sync merge proofs + EMIT event envelopes) + per-surface MallorySigner property tests + cite-drift-detector `LoadBearingSigBundleCidPattern` scanner. 14 of 15 fully-enforced at HEAD; Inv-15 partially-enforced-via-existing-discipline. |
| **Inv-16 mint** (per Q3 Option D + 3 e2r reviewers) | NIGHT-SHIFT-2026-05-27 + standards-skeptic review | Isomorphic-to-Inv-15 for encryption side: every encryption-targeting identifier refers to a payload (recipient-set spec) not ciphertext-bundle bytes. |
| **K_Atrium (pending Ben ratification under MembershipSet framing)** | Q3 Option D community lens | CSPRNG-random on Atrium-creation; multi-stanza-HPKE distributed at member-join (one-shot per recipient); FORK-ONLY rotation; preserves CGKA-deferral. Composes with K_principal + DAK. See §9. |
| **Path-A.5 hybrid (key K(N) to immutable Version-Node-CIDs not mutable Node-CIDs)** | path-a-vs-path-b-specialist-review (2026-05-24 commit `5f50a028`) | Selective-share recipients derive keys against immutable Version-Node-CIDs; CURRENT pointer mutations don't cascade to key rotation. |
| **K_principal_generation tracking** | L9 A4 amendment | Per-K_principal-generation counter for rotation accounting + recipient-grant invalidation on rotation. |
| **XChaCha20-Poly1305 widening** | L4 IMPL-A2 amendment | Add `XChaCha20-Poly1305` codepoint additive arm; 24-byte nonce reduces birthday-collision risk for long-lived per-Node keys. Currently ChaCha20-Poly1305 only. |
| **`CryptoPolicy::require_hybrid_pq` consumer-side flag** | V1-FROZEN-INTERFACE-DEFERRED Row D-15b | Defense-in-depth flag pre-resolve + rejects classical-only codepoints with typed error. Caller-opt-in. |
| **`AeadEnvelope::to_wire_bytes` nonce-panic → Result lift** | V1-FROZEN-INTERFACE-DEFERRED Row D-15d | Hardening; `nonce.len() > 255` currently panics; lift to typed error. |
| **`AuthorizationGrant.binding_sig` hardcoded `[u8; 64]` → varsig-tagged** | V1-FROZEN-INTERFACE-DEFERRED Row D-15e | Aligns binding-sig wire shape with multiformats varsig framing; PQ-half preserved via codepoint-dispatch fall-through. |

### §3.2 Phase-4-Meta-Composing (pre-v1-beta-tag; UX-coupled / self-circular)

| Item | Source | Scope |
|---|---|---|
| **Identity-recovery protocol design** | CLAUDE.md #15 v1-assessment-window | Named in v1-assessment-window per CLAUDE.md baked-in #15; Phase-4-Meta-Composing scope; benefits from F-full DAK substrate. Shamir / social / hardware shapes named but undecided. |
| **Biometric layer (additive on DAK)** | NIGHT-SHIFT-2026-05-27 | Ben open to "password-only at v1 + biometric layered later within Phase-4-Meta"; Phase-4-Meta-Composing if not Phase-4-Meta-Core. Platform-specific (Tauri-plugin-biometric / WebAuthn / etc.). |
| **Per-platform integration scope** | NIGHT-SHIFT-2026-05-27 | Tauri / native CLI / browser-tab / mobile; per-platform credential-store integration cost TBD. |

### §3.3 V1-FROZEN-INTERFACE-DEFERRED Row D-15 (post-v1-beta hardening watch-list)

- Row D-15a — AAD per-chunk `total_chunks` augmentation **CLOSED** at R6 R1 FP (already at HEAD)
- Row D-15b — `CryptoPolicy::require_hybrid_pq` consumer-side flag (deferred)
- Row D-15c — RETRACTED at R6 R2 fix-pass
- Row D-15d — `AeadEnvelope::to_wire_bytes` nonce-panic → Result (deferred)
- Row D-15e — `AuthorizationGrant.binding_sig` hardcoded `[u8; 64]` → varsig-tagged (deferred)

### §3.4 Pure-PQ + NF-1 end-state arms (audit-gated)

- `SwapMatrix::try_pure_pq_sole_trust_path` is the ONLY constructor for
  `EncryptionArm::PurePqMlKem768Only` + `SignatureArm::
  PurePqMlDsa65Slhdsa`; returns
  `SwapMatrixError::AuditNotLandedPurePqRejected` while
  `AUDIT_LANDED_PURE_PQ_FLAG = false` (compile-time constant at
  `swap_matrix.rs:140`).
- Flips at NF-2 / C-GM-AUDIT independent-audit landing for v1-GM tag
  per CLAUDE.md baked-in #5 + #15.
- `0x647b` (ML-KEM-768⊕HQC) build-trigger = FIPS 207 final (~2027
  per NIST projection).

### §3.5 Future-phase + long-horizon crypto-key-management scope (per coordinator scope-expansion)

> The MembershipSet unification panel needs to assess whether unification
> holds across the FULL ARCHITECTURAL ARC + crypto-evolution arc
> (~25 years; data signed today must verify in 2050). If long-horizon
> crypto plans require key-management shapes incompatible with
> MembershipSet unification, surface NOW.

#### §3.5.1 Phase-5+ (post-v1-beta committed-scope Phases per CLAUDE.md baked-in #12)

| Item | Source | Long-horizon implication |
|---|---|---|
| **Post-quantum signature-scheme migration** | `docs/future/phase-4-backlog.md §4.137` (umbrella #1186 META #1080) | Named as Phase-9+ committed-scope-adjacent — 4-crate cascading SemVer-major + missing Compromise-registry entry. Phase-4-Meta deliverable = naming + Compromise-registry mint; Phase-9+ deliverable = migration. The codepoint-additive discipline + Inv-15 mean the migration is a pure additive landing (new codepoint + new construction) — NOT a wire break + NOT a re-sign of historic content. |
| **CGKA / MLS-PQ adoption (post-v1-beta)** | NIGHT-SHIFT-2026-05-27 + Q3 Option D | CGKA codepoint RESERVED for post-v1-beta when MLS-PQ matures. v1-beta group fallback = multi-stanza-HPKE (age/Saltpack pattern). Atrium-forkability semantics ≠ MLS-messaging-leave-forgets; the unification analysis should treat CGKA as a future-additive Layer-C arm with its own codepoint, not as a replacement for K_Atrium. |
| **Hybrid-→-pure-PQ migration arc** | CLAUDE.md baked-in #5 + V1-FROZEN-INTERFACE-DEFERRED + Compromise #30 | Hybrid-PQ stays the v1-beta + v1-GM DEFAULT. Pure-PQ-only arms (`0x647c`, `0x0003`) are RESERVED today + audit-gated. Eventual classical-deprecation transition is a Phase-N additive when classical-side primitives become deprecated (no specific date; driven by NIST recommendations + audit landscape). The framing is permanent: Benten owns the SUITE codepoint, not the algorithm; algorithm-evolution is wire-format-additive. |
| **HSM / TEE integration** | Not yet rowed | NOT named in current backlog; would land as engine-level extension per CLAUDE.md baked-in #19 (engine-level extensions are Rust crates compile-time linked; trust = "you compiled this in") — distinct from app-level plugins. The K_principal-store backend seam at G-CORE-3e is the natural integration point (HSM-backed K_principal-per-DID vs file-backed); the existing `derive_test_seam_key_from_cid_with_namespace` signature `(namespace_did, plaintext_cid) → K(N)` is HSM-substitutable without ripple. |
| **Multi-Atrium key-management (Atrium-of-Atriums federation)** | Forkability-semantics-formalization (Ben articulated 2026-05-27; not yet codified) | Atrium is FORKABLE not messaging-leave-forgets. When member leaves, they keep copy of past content; future content excludes them. Multi-Atrium federation = each Atrium owns its K_Atrium independently; cross-Atrium sharing = re-grant pattern (issue HPKE-encrypted bundle to target-Atrium membership set). Unification implication: MembershipSet generalization MUST hold across (single-user multi-device + Atrium-membership + cross-Atrium federation) shapes; if it doesn't, the federation story breaks. |
| **Identity-recovery protocol** | CLAUDE.md baked-in #15 v1-assessment-window | Named for v1-assessment-window. Likely Phase-4-Meta-Composing (pre-v1-beta-tag per phase-ordering-precision codification). Decision-shapes: Shamir secret-sharing (k-of-n trusted-recovery-set; threshold cryptography); social-recovery (multi-signer attestation); hardware-recovery (YubiKey / FIDO2 with WebAuthn). Each shape implies different key-management primitives (Shamir adds a secret-sharing primitive; social adds a multi-sig threshold scheme; hardware adds platform-bound key-attestation). UCAN-spec already has `SelfRevocation` envelope as the canonical "old-key revokes-itself + propagates" primitive; identity-recovery composes with this. |

#### §3.5.2 Long-horizon (Phase 6-8+ + 25-year crypto-evolution arc)

| Phase | Crypto-key-management work surfaced |
|---|---|
| Phase 6 (AI workflow forking) | No NEW crypto-key-management surfaces named; forking inherits the K_Atrium semantics. |
| Phase 7 (Garden approval flows) | No NEW crypto-key-management surfaces named; approval = UCAN-bound capability extension. |
| Phase 8 (decentralized self-discovered registry) | Registry trait shape decision per `docs/future/phase-4-backlog.md §3.10` task (CID-keyed announce; `DiscoveryQuery` open-shape; richer error vocabulary). Registry signatures use the existing hybrid-sig primitive; no new crypto-key-management. |
| Phase 9+ (post-quantum migration) | Cascading SemVer-major across 4 crates per #1080; the additive-codepoint discipline absorbs the algorithm change. |
| ~2027 | FIPS 207 (HQC-KEM) final → `0x647b` (ML-KEM-768⊕HQC) build-trigger; landed as additive PQ⊕PQ end-state arm. |
| ~2030-2035 | Post-quantum production maturity + classical-deprecation pressure increases. The audit-gated pure-PQ arms (`0x647c`, `0x0003`) may flip live; hybrid stays as defense-in-depth at least through 2040. |
| ~2050 | Inv-15 + Inv-16 + the additive-codepoint discipline mean data signed today must verify in 2050. The framing commitment is **forever**: multiformats-permanent + codepoint-dispatched + never wire-break + never re-sign-historic-content. |

#### §3.5.3 Long-horizon shape implications for MembershipSet unification

- **Codepoint-additive + Inv-15 + Inv-16 mean key-management primitives are wire-format-additive forever.** Any unification must NOT assume the cipher-suite set is closed — it grows monotonically.
- **HSM / TEE integration would land at K_principal-store backend seam.** Unification at MembershipSet level should treat "K_principal" as an opaque secret with rotation+generation semantics, not as a specific byte source. The substrate-shape is stable.
- **Forkability semantics are Benten-specific.** Unlike MLS messaging (where leave = forgets), Atrium-forkability means departed members retain past content. K_Atrium rotation is **FORK-ONLY** — the K_Atrium changes when the Atrium forks, not when membership shrinks. This is a load-bearing distinction from CGKA-style group key agreement protocols; unification must not import CGKA assumptions.
- **Multi-Atrium federation = re-grant pattern.** Cross-Atrium sharing is NOT a new key primitive; it's an HPKE-encrypted bundle issued from one Atrium's authority to another Atrium's membership set. Unification implication: MembershipSet generalization should compose across nested membership sets (member-set-of-Atriums whose members are themselves membership-sets-of-users).
- **Identity-recovery shapes still TBD.** If Shamir-secret-sharing wins, a new primitive (secret-sharing) joins the key chain at K_principal level. If social-recovery wins, multi-sig threshold composes with existing hybrid-sig. If hardware-recovery wins, K_principal becomes platform-attestation-bound. Unification analysis MUST hold across all three branches (or surface that it can't).

---

## §4 — K_principal derivation + lifecycle

### §4.1 Current shape (STUB)

```text
K_principal = BLAKE3-keyed-hash(K_PRINCIPAL_DOMAIN_KEY,
                                namespace_did.as_bytes())
```

- Deterministic; same DID → same K_principal across runs / hosts.
- `K_PRINCIPAL_DOMAIN_KEY` is a published 32-byte constant.
- **Anyone with `namespace_did` + ciphertext can decrypt** — the
  STUB is a substrate-shape stand-in for the production key-store.

### §4.2 Planned shape (F-full Layer-A; pulled into Phase-4-Meta-Core per Ben 2026-05-27)

- **Real K_principal at rest in encrypted form; NOT derivable-from-DID.**
- Storage shape per Option F+ pseudo-keypair-NO-GO ratification:
  **ChaCha20-Poly1305 AEAD-under-DAK** (NOT under a password-derived
  asymmetric pseudo-keypair; that pattern rejected per Tempo-paper
  IACR 2025/1399 timing-side-channel risk against ML-KEM-768
  KeyGen-from-secret-seed + zero production-system precedent).
- Vault location = per-platform credential-store via `keyring-core`
  v1.0.0; fallback file-backed under DAK-encrypted blob.
- **K_principal_generation tracking** (per L9 A4): a monotonic
  generation counter increments on K_principal rotation; recipient
  grants embed the generation they were issued under; rotation
  invalidates older-generation grants.
- **Rotation semantics:** active rotation is a heavy operation
  (re-derive ALL K(root) + K(N) for every Node under this
  K_principal); per Compromise #31 "stays OPEN at v1-beta + v1-GM"
  — already-derived keys remain decryptable forever; mitigation =
  tight UCAN `nbf`/`exp` windows + rotation discipline.

### §4.3 Multi-device composition (M1a will detail; flagged here)

- One K_principal SHARED across all of a user's devices.
- New device → existing device runs **multi-device-key-wrap** =
  HPKE-encap of K_principal to new-device pubkey.
- Identity = user's DID; device-DIDs are sub-principals authorized
  to act-as the user-DID via device-attestation envelope chain.
- **DEFER to M1a for: UX flow + device-pairing handshake + sync-state-
  resolution + cross-platform credential-store integration shape.**

### §4.4 K_principal protection at rest (Layer-A vault)

- Vault = `ChaCha20-Poly1305(K_principal || user-DID-private-key,
  aad = device-attestation-CID, key = DAK)`.
- Unlocked on engine-start via DAK-derivation-from-password.
- Cross-platform: keyring-core for native; file-backed under DAK
  for browser-tab / non-keyring-supporting platforms.

---

## §5 — DAK design (planned; F-full Layer-D)

### §5.1 Shape

- **DAK = Argon2id(password, salt, params)** per-device.
- **NOT YET BUILT** (entirely).
- Argon2id tier per L10 U41 amendment (specific tier parameters TBD;
  recommended baseline `m_cost ≥ 64 MiB, t_cost ≥ 3, p_cost ≥ 4` for
  desktop-class hardware; reduced for mobile per platform-cost
  amendment).
- 32-byte output → ChaCha20-Poly1305 key for the Layer-A vault.

### §5.2 Per-device discipline

- Each device has its OWN DAK (each device has its own password OR
  per-device unlock secret; one user can have many DAKs).
- Devices do NOT share DAKs; they share K_principal (via
  multi-device-key-wrap).
- Engine-unlock-on-authentication: cold-start engine asks for
  password → derives DAK → opens Layer-A vault → loads K_principal
  + user-DID-private-key into memory.

### §5.3 Composition with biometric (Phase-4-Meta-Composing additive)

- Biometric layer = additive unlock pathway; same vault, different
  unlock-secret source.
- Platform-specific: Tauri-plugin-biometric / WebAuthn / etc.
- Open question: biometric-unlocks-DAK vs biometric-unlocks-DAK-derived-
  blob-which-unlocks-vault. Substrate stays the same either way.

### §5.4 Remote-permission-call-from-another-device (Ben emphatic 2026-05-27: REQUIRED)

- Shape inspired by Signal-Provisioning + CTAP-2.2.
- User on device-A wants to grant device-B permission. Device-A
  receives a permission-request notification; user approves; device-A
  signs an HPKE-encrypted permission-grant to device-B's pubkey.
- Composes with: K_principal (the underlying secret being shared
  via the grant); DAK (device-A's user authentication to act-as the
  user-DID); HPKE Layer-C primitive.

---

## §6 — K(N) per-Node AEAD chain

### §6.1 Current shape (LIVE)

Per `RATIFIED-S&C 2026-05-21` (refinement 2) + Spike-E Interpretation-B
+ §1.A.FROZEN item 15(f) + R6 R2 batch-A Item 7 extension:

```text
K(root) = HKDF-SHA256(K_principal,
                      info = "root:codepoint:" || codepoint_le_bytes || root_cid)

K(N) = HKDF-SHA256(K(predecessor),
                   info = "step" || edge_label || N.cid)
```

- 32-byte output (HKDF-SHA256-natural).
- Zeroize-on-drop newtype.
- Production usage at HEAD = `derive_root` only (single-Node case);
  multi-edge `derive_step` chain through subgraph-walks lands at the
  future subgraph-walk wire-up.

### §6.2 Path-tagged (Spike-E Interpretation-B) — LOAD-BEARING

Same Node reached by different predecessors yields different K(N)
(the selective-share feature). **Literal RATIFIED-S&C
"structure-independent" formula was DISPROVED by Spike E** — DO NOT
collapse `derive_step` to `HKDF(K_principal, N.cid)`.

### §6.3 Path-A.5 (key K(N) to immutable Version-Node-CIDs not mutable Node-CIDs)

Per `path-a-vs-path-b-specialist-review.md` (2026-05-24, commit
`5f50a028`): selective-share recipients derive keys against immutable
Version-Node-CIDs, NOT the mutable CURRENT pointer. Implication:

- K(N) is stable across CURRENT-pointer mutations.
- A recipient who derived K(N) for Version V1 keeps the ability to
  decrypt V1 even after CURRENT → V2 (consistent with "already-
  derived keys remain decryptable forever" per Compromise #31).
- V2 is encrypted under a DIFFERENT K(N) (because V2's
  Version-Node-CID differs from V1's), so the recipient must be
  re-granted to access V2.

### §6.4 Rotation cascade

- K_principal rotation → ALL K(root) + K(N) under it MUST be
  re-derived (heavy).
- K_principal_generation counter tracks which generation a grant
  was issued under.
- Forward-secret-rekey-on-every-revocation is OUT OF SCOPE for v1
  (would require MLS-style per-message keys); deferred to potential
  CGKA-future-additive landing.

---

## §7 — Cipher suite codepoint registry

### §7.1 Codepoint table (see §2.3 for full enumeration)

11 codepoints across hash + sig + cipher tables, FROZEN at G-CORE-9
V1-FROZEN-INTERFACE.md item 6 (commit `8cc4eddd` + R1 fix-pass
commit `235ad861` ratifying typed-rejection framing for 0x0003 +
0x647c). **Integer values are PERMANENT per the freeze contract.**
Algorithms behind each codepoint are SWAPPABLE; codepoint values
themselves are wire-canonical.

### §7.2 Dispatch semantics

- Each codepoint enum carries `from_codepoint(u16) -> Result<Self,
  UnsupportedAlgorithm>` typed dispatch.
- Unknown codepoints → typed `UnsupportedAlgorithm` arm; NEVER a
  silent fallback. P2P-mainstream choice per Veilid / MLS-RFC9420 /
  Nostr-NIP-44; age's silent-ignore is the deliberately-rejected
  outlier.
- Reserved-typed-reject arms (`0x0003`, `0x647b`, `0x647c`, `0x0000`)
  surface the typed error at `Codepoint::resolve` even though the
  values are registered.

### §7.3 LE-vs-BE issue (per L4 IMPL-A2)

**At HEAD: LE.** `AeadEnvelope::to_wire_bytes` encodes
`cipher_codepoint.raw().to_le_bytes()` at bytes 2-3. The structural-
KDF info-tag at `derive_root` also uses LE. Lens L4 IMPL-A2 raised
LE-vs-BE for wire canonicalization per Q2; **the choice was LE** and
is locked at the V1-FROZEN-INTERFACE item 6 byte-pin. Migration to BE
would be a wire-format break — NOT happening at v1-beta. Any future
codepoint additions stay LE for consistency.

### §7.4 X-Wing-mislabel corrective (per cryptographer-review)

The combiner at `cipher_suite.rs::x_wing_combine` is currently:

```rust
let salt = SHA3-256("x-wing-v1-benten-0x647a");
let combined = HKDF-SHA256(ikm = ss_x || ss_mlkem || ek_x || ek_mlkem || pub_x || pub_mlkem,
                           salt = salt,
                           info = "x-wing-v1-benten-0x647a") -> 32 bytes;
```

This is NOT real X-Wing math per `draft-connolly-cfrg-xwing-kem-10` /
`draft-irtf-cfrg-concrete-hybrid-kems-03`. Real construction:

```rust
combined = SHA3-256(label || ss_M || ss_X || ct_X || pk_X)  // 32 bytes
```

where `label` is the spec-defined 6-byte ASCII label.

**Implications:**

- The IACR CIC 2024 peer-reviewed tight IND-CCA proof for X-Wing
  does NOT transfer to Benten's current combiner.
- ~24 LOC corrective + rename to RG-blessed `MLKEM768-X25519` (per
  `draft-irtf-cfrg-concrete-hybrid-kems-03`).
- **PRE-V1-BETA-TAG-MUST-FIX INDEPENDENT of F-full outcome.**
- Codepoint `0x647A` integer stays (matches IETF reservation per
  `draft-ietf-hpke-pq-04` Table 1).

### §7.5 Registry-mint discipline (per L8 + §3.5g cross-language rule-mirror)

- New codepoint mints follow the additive-codepoint discipline:
  reserve at unused value + typed-reject at dispatcher until LIVE.
- Each codepoint constant has a TS-side mirror in
  `packages/engine/src/errors.generated.ts` when it surfaces in
  errors. Cross-language rule-mirror drift-detector enforces
  symmetric updates per §3.5g.
- Hash codepoints reference IANA multihash registry (`0x1e` BLAKE3,
  `0x16` SHA3-256, `0x1015` SHA-512/256); cipher / sig codepoints
  are Benten-owned table entries that reference IANA HPKE / COSE
  registries for component algorithm IDs but NEVER mint Benten
  algorithm numbers.

---

## §8 — Hybrid signature primitives

### §8.1 v1-beta DEFAULT — Ed25519⊕LAMPS Composite ML-DSA-65

- Codepoint `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`.
- Algorithm = LAMPS `id-MLDSA65-Ed25519-SHA512` per
  `draft-ietf-lamps-pq-composite-sigs-19`.
- OID `1.3.6.1.5.5.7.6.48`; IANA early-allocated 2025-10-20.
- Both-must-verify strip-resistant per NF-4 + the committing combiner
  `hybrid_combine` at `src/sig.rs`.
- Sizes (per `src/sizes.rs` dynamic dispatch): ~1952-B key /
  ~3309-B sig.

### §8.2 Compromise #31 — LAMPS EUF-CMA-only NOT SUF-CMA

- **OPEN at construction layer; CLOSED-EQUIVALENT at application
  layer via Inv-15** (Phase-4-Meta-Core mint per Ben ratification
  2026-05-26).
- Construction-layer scope per LAMPS draft-19: EUF-CMA-only +
  Weakly-Non-Separable; NOT SUF-CMA-preserving + NOT
  Strongly-Non-Separable.
- Application-layer closure: Inv-15 3-layer-decomposition (identity
  = canonical-payload-CID; authentication = codepoint-dispatched
  sig; revocation = semantic tuple `(issuer, subject, cap, audience,
  validity)`).

### §8.3 Bird-of-Prey reserved as future-additive

- SUF-CMA-preserving combiners (Bird-of-Prey per Bossuat et al.
  EUROCRYPT 2026 / IACR 2025/1844; `draft-prabel-cfrg-suf-hybrid-sigs-01`
  individual submission) RESERVED as future-additive codepoints when
  WG-adopted + production-quality reference impls + independent impl
  audit.
- Added via crypto-agility framework as additive impl + new
  codepoint, never a wire-format break.
- Inv-15 remains the load-bearing architectural property regardless
  of construction choice; SUF-CMA-preserving constructions become
  defense-in-depth additive layer.

### §8.4 Composition with multi-device + Atrium key chains

- Each device-DID has its own Ed25519⊕ML-DSA-65 keypair.
- User-DID has its own Ed25519⊕ML-DSA-65 keypair (the K_principal-
  vault-protected key).
- Device-attestation envelope chain (`DeviceAttestation` +
  `RotationLog`) is the load-bearing seam: device-DID signs
  payloads using its own keypair; parent attestation from a
  trusted-parent device-DID (or the user-DID directly at bootstrap)
  authorizes the device-DID to act-as the user-DID.
- Atrium-membership uses UCAN-bound capability grants whose signature
  is one of these hybrid signatures.

---

## §9 — K_Atrium (pending Ben ratification under MembershipSet framing)

### §9.1 Planned semantics (per Q3 Option D)

- **K_Atrium = CSPRNG-random on Atrium-creation.**
- Distributed via **U17 multi-stanza HPKE at member-join (one-shot)**
  — each new member receives an HPKE-encrypted bundle containing
  K_Atrium, encrypted to their HPKE pubkey.
- **FORK-ONLY rotation** — K_Atrium changes ONLY when the Atrium
  forks; not when membership shrinks (Atrium-forkability semantics
  per Ben articulated 2026-05-27).
- **Preserves CGKA-deferral** — no continuous group-key-agreement;
  no leave-forgets-forward-secrecy. v1-beta-shippable.

### §9.2 Composition with K_principal + DAK

- K_Atrium is INDEPENDENT of K_principal.
- A user-DID may participate in many Atriums, holding many K_Atriums
  (one per Atrium membership).
- K_Atrium is stored locally under the same DAK-protected vault as
  K_principal (Layer-A vault holds: K_principal, user-DID-private-
  key, ALL K_Atriums the user holds).
- Rotation: when a user-DID leaves an Atrium, the user retains the
  K_Atrium copy (and thus the ability to decrypt past Atrium content);
  the Atrium itself does NOT re-key on departure (FORK-ONLY).

### §9.3 NOT YET BUILT

- No K_Atrium type at HEAD.
- No multi-stanza-HPKE primitive at HEAD.
- No Atrium-creation key-mint at HEAD.
- All deferred to F-full Layer-C wave (Phase-4-Meta-Core);
  multi-stanza-HPKE is the building block.

---

## §10 — Key-chain MembershipSet-shape pattern observations

For each key, observation = "is this a MembershipSet-shared-key vs
per-member-key vs derivable-from-something-else?" + unification
implication:

| Key | Shape | MembershipSet-unification observation |
|---|---|---|
| **K_principal** | Per-user-identity (one per user-DID; shared across user's devices via multi-device-key-wrap) | Shape = **MembershipSet of devices owned by one user-DID**. Unification under MembershipSet would treat "user's set of devices" as a MembershipSet; K_principal is the shared-secret distributed via HPKE-encap to each member-device's pubkey. **Unification HELPS** — multi-device-key-wrap IS already a degenerate MembershipSet-key-distribution. |
| **DAK** | Per-device-per-user (one per (device, user) pair) | NOT a MembershipSet shared key; strictly local-to-device. Unification would obscure this; DAK should stay per-device. |
| **K(root)** + **K(N)** | Derived from K_principal + cipher-suite codepoint + (root_cid \| path of edge_labels) | Derivation chain; NOT a stored shared key. Unification implication: the K(N) shape is path-tagged (Spike-E Interpretation-B); selective-share semantics arise from the chain itself. K(N) is a **derivable function of K_principal + path** — its "membership" is implicit in who knows K_principal + the path. **Unification at MembershipSet should NOT try to abstract K(N) as a stored key** — it's a derivation. |
| **K_Atrium** | Per-Atrium (one per Atrium; shared by all members) | Shape = **MembershipSet of users in one Atrium**. The MembershipSet-unification candidate. Distribution = multi-stanza-HPKE per member (each member's HPKE-pubkey gets its own encrypted stanza). FORK-ONLY rotation = MembershipSet-identity changes on fork. **Unification HELPS** — this is exactly the shape MembershipSet captures. |
| **Hybrid-sig keypair (Ed25519⊕ML-DSA-65)** | Per-DID (one per device-DID + one per user-DID + one per plugin-DID) | NOT shared; strictly per-DID. Each DID owns its own keypair. NOT a MembershipSet candidate. |
| **HPKE keypair (per-recipient for encrypt-to-recipient)** | Per-recipient-DID | Each recipient has its own HPKE keypair. Encrypt-to-MembershipSet = multi-stanza-HPKE = N encrypted stanzas, one per recipient. Shape = **MembershipSet-fan-out at encryption time**; the keypair itself stays per-recipient. **Unification HELPS** — multi-stanza-HPKE IS the MembershipSet-encryption primitive. |
| **AeadKeyMaterial / K_root / K(N)-fed-AEAD** | Per-Node (one per (K_principal-or-K_Atrium, path-to-Node) tuple) | NOT a MembershipSet itself; it's the symmetric output of the derivation. Membership comes from upstream (who knows K_principal / K_Atrium). |
| **K_principal_generation counter** | Per-K_principal (monotonic) | Generational accounting; not a MembershipSet. Unification would track which generation each MembershipSet snapshot saw. |

### §10.1 Where unification under MembershipSet SIMPLIFIES

1. **Multi-device-key-wrap = MembershipSet-of-devices encrypt-to-recipient.**
   The shape is identical to Atrium-membership-K_Atrium-distribution: a
   shared secret + multi-stanza-HPKE-encrypt to each member's
   asymmetric pubkey. Unifying these under MembershipSet collapses
   two implementations into one.

2. **K_Atrium = MembershipSet-of-Atrium-members shared-secret.** First-
   class MembershipSet candidate.

3. **Encrypt-to-recipient HPKE Layer-C = MembershipSet-fan-out primitive.**
   Multi-stanza-HPKE per recipient member. Composable for both Drop
   bundles (M1b scope) AND multi-device-key-wrap (M1a scope) AND
   K_Atrium distribution.

4. **Cross-Atrium federation = nested MembershipSet** (member-of-
   Atriums whose members are users); unification should compose.

### §10.2 Where unification under MembershipSet OBSCURES

1. **DAK is strictly per-device-per-user; NOT a MembershipSet.**
   Forcing it under a MembershipSet umbrella loses the per-device-
   independence property + adds attack surface (one device's DAK
   compromise propagating across a "device-set" abstraction).

2. **K(N) derivation chain is path-tagged; NOT a stored membership-
   shared key.** The selective-share semantics arise from
   `derive_step(predecessor, edge_label, N.cid)` chain composition;
   collapsing K(N) into MembershipSet-shared-key shape would erase
   the path-tag distinction Spike-E PROVED load-bearing (different
   path → different key).

3. **Hybrid-sig keypairs are strictly per-DID.** Each DID owns its
   own. Unification at MembershipSet would conflate identity-
   ownership with membership.

4. **Compromise #31 / Inv-15 closure mechanism is at the IDENTITY
   layer (payload-CID), not the MembershipSet layer.** Unification
   must preserve the 3-layer-decomposition (identity = payload-CID;
   authentication = codepoint-dispatched sig; revocation = semantic
   tuple) — none of these layers IS a MembershipSet.

### §10.3 Compositional pattern (where unification SHINES)

The recurring shape **MembershipSet = {member-DID with HPKE-pubkey}**
+ **shared-secret = K_X (CSPRNG-random)** + **distribution = multi-
stanza-HPKE** + **rotation = FORK-ONLY (recipient retains past)**
appears in:

- Multi-device-key-wrap (member = user's device-DID; K_X = K_principal)
- K_Atrium distribution (member = user-DID in Atrium; K_X = K_Atrium)
- Drop bundle to-recipients (member = grantee-DID; K_X = ephemeral content-key)
- Cross-Atrium federation (member = recipient-Atrium-membership-set; K_X = federation-grant key)

**Unification opportunity:** one MembershipSet primitive + one multi-
stanza-HPKE distribution primitive + one FORK-ONLY-rotation discipline
substrate could replace four parallel implementations.

---

## §11 — Open questions for downstream specialists (M2-M6)

### §11.1 Cryptographic correctness questions

1. **K_Atrium HKDF role-separation** — when K_Atrium is distributed
   via multi-stanza-HPKE, does the receiver derive a per-Node key
   from K_Atrium directly OR via a `derive_root(K_Atrium, root_cid,
   codepoint)` chain mirroring K_principal? The structural-KDF
   chain at `structural_kdf.rs` was designed assuming K_principal-
   rooted; does it generalize cleanly to K_Atrium-rooted, or does
   K_Atrium need its own info-tag prefix ("root:atrium:..." vs
   "root:codepoint:...")?

2. **K_principal_generation in info-tag** — should K_principal_generation
   bind into the K(root) info-tag? Today the info-tag is
   `"root:codepoint:" || codepoint_le_bytes || root_cid`. Adding
   `|| generation_le_bytes` would make grants automatically generation-
   bound at the K_root layer. Does this conflict with Inv-15 /
   Inv-16 layering?

3. **Multi-stanza-HPKE AAD shape** — should the per-stanza AAD bind
   `(recipient_did, stanza_index, total_stanzas)` analogously to
   `aad_per_recipe` for DropBundle? Per §2.5 the recipe pattern is
   already established. Should multi-stanza-HPKE re-use exactly
   that AAD layout, or get its own `aad_per_stanza`?

4. **X-Wing-mislabel corrective + HPKE-mode-base** — when Layer-C
   HPKE lands using MLKEM768-X25519 KEM (real X-Wing math), does
   the corrective at Layer-B (~24 LOC at `cipher_suite.rs::x_wing_combine`)
   share the same combiner function with Layer-C's HPKE-encap-decap,
   or are they distinct call sites? Sharing would reduce duplication
   + ensure consistency; not sharing decouples the two upgrade arcs.

### §11.2 Architectural questions

5. **DAK + multi-device cross-platform** — when device-A on Tauri
   (keyring-core-backed DAK) shares K_principal with device-B on
   browser-tab (no keyring; file-backed DAK), does device-B's DAK
   need to be Argon2id-derived from a DIFFERENT password than
   device-A's? Or does multi-device-key-wrap mean device-B never
   needs a DAK (it gets K_principal directly via HPKE-encap)?

6. **K_principal rotation cascade scope** — when K_principal rotates,
   does the cascade re-derive K(N) for EVERY Node ever encrypted
   under any past K_principal-generation, OR only for Nodes "still
   under current grants"? Compromise #31 says "already-derived keys
   remain decryptable forever" — does this mean rotation only
   affects FUTURE writes?

7. **K_Atrium rotation triggers** — FORK-ONLY is the
   non-rotation-on-departure semantics; what events DO trigger
   K_Atrium rotation? Atrium-content-policy-change? Schema-fork?
   Explicit-user-request? The Q3 Option D ratification specified
   FORK-ONLY but didn't enumerate fork triggers.

8. **Identity-recovery + K_principal** — if Shamir-secret-sharing
   wins for identity-recovery, does the recovery-set hold shares of
   K_principal directly OR shares of the DAK-derivation-secret OR
   shares of a recovery-only K_principal_recovery? The K_principal
   protection model + the recovery model interact non-trivially.

### §11.3 Wire-format / additive-codepoint questions

9. **Multi-stanza-HPKE codepoint position** — current cipher table
   has `0x647a` (X-Wing-mislabeled-now-MLKEM768-X25519), `0x6400`
   (classical), `0x647b` (NF-1 reserved), `0x647c` (pure-PQ reserved),
   `0x0000` (no-encryption). Where in this table does HPKE-mode-base[MLKEM768-X25519]
   land? Is it a NEW cipher codepoint (e.g. `0x647d`)? Does
   multi-stanza-HPKE need a SEPARATE codepoint from single-recipient
   HPKE, or is fan-out a same-codepoint envelope arm?

10. **Inv-16 mint** — when minted, does Inv-16 replicate Inv-15's
    3-layer-decomposition (identity = payload-CID; authentication =
    AEAD-tag; revocation = semantic-tuple) exactly, OR does it need
    different layering because encryption-recipient-set is a 1st-class
    semantic vs sig's "this thing"?

### §11.4 MembershipSet-unification-specific questions

11. **Is MembershipSet a stored type OR a derived view?** If stored,
    where does it live (`benten-caps` / new crate / `benten-graph`)?
    If derived, what's the canonical query?

12. **Does MembershipSet name the cryptographic primitive (shared-
    key + multi-stanza-HPKE + FORK-ONLY-rotation) OR the abstract
    membership shape (set-of-DIDs + change-events)?** The four
    sites identified in §10.3 share the cryptographic primitive;
    do they also share the abstract shape?

13. **How does MembershipSet compose with UCAN-bound capability
    grants?** Today AuthorizationGrant carries `GrantKeyMaterial`
    + UCAN-attesting `binding_sig`. If MembershipSet replaces the
    per-recipient grant pattern, the UCAN chain must still bind
    grants to specific authority + scope; does MembershipSet
    membership-attestation compose with UCAN or replace some of it?

---

## §12 — Self-assessment + confidence + cross-references

### §12.1 Self-assessment

- **Read-coverage**: load-bearing crypto-suite source (`structural_kdf`,
  `aead`, `codepoint`, `cipher_suite`, `INTERNALS.md`); K_principal
  STUB site at `redb_backend.rs:117-210`; SECURITY-POSTURE (key terms
  + Compromise #30/#31); CLAUDE.md baked-in #5 (full); INVARIANT-
  COVERAGE Inv-15; NIGHT-SHIFT-2026-05-27 (F-full + ratifications);
  X-Wing→MLKEM768-X25519 rename audit; V1-FROZEN-INTERFACE.md item 6
  + 15(f/g); V1-FROZEN-INTERFACE-DEFERRED Row D-13/D-15; ERROR-CATALOG
  RecipientLacksKeysForSuite + AeadRebindingAttack; phase-4-backlog
  K_principal carry rows + Phase-9+ PQ-migration rows; benten-id
  keypair.rs surface.
- **Not-read** (acknowledged scope limits): full `sig.rs` body
  (548 LOC); `swap_matrix.rs` past line 240 of 2014; spike retros
  (no spikes/ directory at HEAD); RATIFIED-S&C 2026-05-21 doc (not
  in git tree — appears to be referenced-but-not-tracked); Position B
  v2 blog (untracked); `00-implementation-plan.md` (not in HEAD tree);
  full `phase-4-backlog.md §3.10` G-CORE-3 wave details.

### §12.2 Confidence

- **HIGH** on existing-state inventory (§2 + §4 + §6 + §7 + §8) —
  source-grounded with file+line cites.
- **MEDIUM-HIGH** on planned-state (§3) — based on NIGHT-SHIFT-2026-05-27
  + rename audit + CLAUDE.md baked-in #5 sharpening; some items
  pending Ben ratification (K_Atrium per §9; F-full per coordinator).
- **MEDIUM** on long-horizon §3.5 — sparse explicit roadmap;
  inference from CLAUDE.md baked-in #12 (committed Phases 1-8) +
  baked-in #19 (engine-level extensions) + Phase-9+ phase-4-backlog
  rows.
- **MEDIUM** on MembershipSet-shape observations (§10) — I'm
  pattern-matching the four sites at §10.3; the unification panel
  should validate the abstraction holds in detail.

### §12.3 Cross-references to sibling catalogers

- **M1a multi-device-sync** owns: multi-device pairing UX flow;
  multi-device-key-wrap handshake protocol; sync-state-resolution;
  cross-platform credential-store integration shape; device-DID
  attestation chain UX surface. M1c flags but does not duplicate:
  K_principal is shared across user's devices via M1a's wrap flow;
  DAK is per-device-per-user and M1a owns the cross-platform shape.
- **M1b Atrium-membership-sharing** owns: Atrium creation flow;
  member-add / member-remove semantics; SubgraphSpec walker;
  UCAN-gated sharing; Drop bundles; AuthorizationGrant carrier.
  M1c flags but does not duplicate: K_Atrium semantics (this doc
  §9) are the cryptographic substrate M1b composes with; the
  multi-stanza-HPKE primitive (M1c §3.1 + §10.3) is the building
  block for M1b's sharing flow.

### §12.4 Load-bearing references

- `crates/benten-crypto-suite/INTERNALS.md` — internal crypto design
- `crates/benten-crypto-suite/src/structural_kdf.rs` — K(root)/K(N) derivation
- `crates/benten-crypto-suite/src/aead.rs` — AEAD envelope + AAD layouts
- `crates/benten-crypto-suite/src/cipher_suite.rs` — X-Wing combiner site
- `crates/benten-crypto-suite/src/codepoint.rs` — codepoint table FROZEN at G-CORE-9
- `crates/benten-crypto-suite/src/swap_matrix.rs` — bidirectional swap matrix + audit-gate
- `crates/benten-graph/src/redb_backend.rs:117-210` — K_principal STUB site
- `crates/benten-graph/src/aead_wrap.rs` — per-Node graph-layer AEAD wrap
- `crates/benten-id/src/keypair.rs` — Ed25519 keypair + secret-hygiene
- `docs/SECURITY-POSTURE.md` — Compromise #30 (PQ-unaudited) + #31 (LAMPS EUF-CMA-only)
- `docs/INVARIANT-COVERAGE.md` — Inv-15 mint + 3-layer-decomposition
- `docs/V1-FROZEN-INTERFACE.md` item 6 + 15(f/g) — codepoint freeze + KDF + AAD pins
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-13 (CLOSED) + Row D-15a-e (post-v1-beta hardening)
- `docs/ERROR-CATALOG.md` — `E_RECIPIENT_LACKS_KEYS_FOR_SUITE` + `E_AEAD_REBINDING_ATTACK_DETECTED`
- `docs/future/phase-4-backlog.md §3.10 + §4.137` — K_principal carry + PQ-migration rows
- `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-27.md` — F-full + ratifications + Option F+ NO-GO
- `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` — corrective categories
- `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` — LAMPS-default + Inv-15 origin
- CLAUDE.md baked-in #5 (crypto-agility + Phase-4-Meta-Core 2026-05-26 sharpening)
- CLAUDE.md baked-in #15 (Benten Platform v1 milestone + identity-recovery v1-assessment-window)
- CLAUDE.md baked-in #18 + #19 (plugin trust model + engine-level extensions)

---

*Cataloger M1c complete. Hand off to M2-M6 specialist panel.*
