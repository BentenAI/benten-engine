All anchors confirmed: 20 NQ-* (A1-2, C1-5, D1-4, T1-4, W1-5), Inv-16..22 (7 new), Compromise #30-#63 (34 rows). I have everything needed. Writing the unified deliverable.

---

# F-full R2 TEST LANDSCAPE — Unified, Deduplicated (R0.2, R1-CONVERGED)

**Plan:** `477529d0:.addl/phase-4-meta/f-full-r0-plan.md` (1547 lines, §0–§13). **Ground-truth at plan-write:** `2172cb6d`. **Live HEAD:** `bc592e75` — **NOT a descendant of `2172cb6d`** (`git merge-base --is-ancestor 2172cb6d HEAD` → NO). All named test files exist at both SHAs; **R3 must rebase onto whichever SHA the Wave-0 canary lands on** (the V2/BE/`EncryptedEnvelope` migration changes the AAD/codepoint byte layout every freeze-gating family pins).

**What this is.** The merged catalog of test FAMILIES the 6 discovery dimensions surfaced + the completeness-critic's gap-fills, deduplicated across dimension seams, with single-ownership assigned. R3 test-writers implement these as TDD red-phase. Convergence target stands: every FREEZE-GATING family green before any wire-minting canary; the §9 freeze (G-CORE-9) gated on the full set.

**Two laws every family inherits.**
1. **TDD red-phase** per the in-tree pattern (`pim-12 §3.6e`): stub-shim compiles green at baseline + `#[ignore = "RED-PHASE…"]`; the R5 closing wave deletes the stub, inserts the real `use`, un-ignores, verifies green. Every pin drives a **production** call site with an **observable consequence** + a **would-FAIL-if-no-op'd** body (`pim-2 sub-rule-4` + `pim-18 §3.6f` + `§3.6f-ext` regression-guard-substantive-arm).
2. **Wave-0 DAG edge (M-20).** Every byte-pinning family authors **V2 + BE + `EncryptedEnvelope`** from the first `#[ignore]` commit. There is no surviving V1/LE golden vector. A family authored against the in-tree LE/V1 `AeadEnvelope` pins soon-to-be-dead bytes — the red-phase failure is "still LE / still V1 / still `AeadEnvelope`."

**Legend.** **FG** = FREEZE-GATING (must be green before G-CORE-9 freeze; a miss = an untested frozen byte). **FN** = functional. **CF** = conformance/interop. **DC** = disclosure-coherence (doc-coupling, reuses the `tf3f_revocation_reach_forever_valid_documented` shape).

---

## 1. TEST-FAMILY CATALOG

The catalog is organized into **12 family-groups**. Each family carries a stable **F-ID** (the unified ID R3 cites), its source dimension(s), what it pins, R0/Inv/Compromise/codepoint/NQ mapping, class, red-phase intent (compressed), and size. Where ≥2 dimensions surfaced the same family, they are **merged under one F-ID** with the seam-owner named; the dropped duplicate IDs are noted so R3 doesn't double-implement.

### Group 1 — Wave-0 / `ENVELOPE_FORMAT_VERSION_V2` migration (ALL FG — hard upstream M-20)

These gate every downstream canony. A miss = every codepoint authored at the wrong endianness/construction.

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-W0-1** Real X-Wing SHA3-256 combiner (merges CE-A1 + WF-G1) | `0x647A` computes `SHA3-256(label‖ss_M‖ss_X‖ct_X‖pk_X)` per draft-connolly, NOT the in-tree HKDF-SHA256 stand-in (`cipher_suite.rs:37`) | §2.0 BR-3, §2.1 C-6, §3.2(a), §7.1, §9.1-2; cp `0x647A` | FG | ~5–7 | `assert_eq!(combine_x_wing(...), draft_KAT)`; negative `assert_ne!(out, legacy_hkdf(...))`; classical `0x6400` `classical_combine` consistency. Flips `tf3a_x_wing_hybrid_wrap_…0x647a.rs` KAT expectation. |
| **F-W0-2** X-Wing interop KAT (CE-A2) | round-trips against the **published** draft-connolly X-Wing vectors byte-for-byte | §3.2, §7.1 exit, §8.3, §9.1-2 | FG/CF | ~3–5 | KAT-row table; encap-determinism-from-seed; decap; negative wrong-pk. **External-vector seed.** |
| **F-W0-3** BE endianness sweep + **zero-`to_le_bytes`** scanner (merges CE-A3 + WF-F2 — the flagship M-19 family) | every wire/AAD integer is BE; the **13 in-tree `to_le_bytes` sites → 0** on wire/AAD paths (`aead.rs:165,244,277`, `benten-graph/aead_wrap.rs`, `platform-foundation/plugin_manifest.rs`); the X-Wing **info-tag ASCII string is NOT flagged** (m-1) | §2.3 Q2, §3.2(b), §4.1, §4.1.MIGRATION, §7.1; M-19; U7; §9.1-2; §10.8 seed | FG | ~6–9 | source-scanner pin (count==0) + per-field BE hex-pins + info-tag-not-flagged negative. **Single highest-leverage conformance gate.** |
| **F-W0-4** `AeadEnvelope`→`EncryptedEnvelope` rename + lift + typed `BindingContext` (merges CE-A4 + WF-A2) | shipped flat `AeadEnvelope` (`aead.rs:145`, untyped AAD) becomes codepoint-dispatched `EncryptedEnvelope` + `#[non_exhaustive]` typed `BindingContext`; grep=ZERO at HEAD ⇒ rename+lift not greenfield | §4.1, §4.1.MIGRATION-3, §6.2; Inv-16; M-18 | FG | ~5–7 | construct `EncryptedEnvelope{payload, aad_binding}`; codepoint committed in typed AAD; cross-variant `BindingContext` strict-reject (U2); `g_core_9_non_exhaustive_audit_drop.rs` audit shape. |
| **F-W0-5** Single V1→V2 bump + golden-vector regen (merges CE-A5 + WF-F1 + WF-F3) | exactly **ONE** `V1→V2` covers W0-1+W0-3+W0-4 (NOT three); regenerated corpus internally consistent; V1 typed-rejected | §4.1.MIGRATION, §7.1, NQ-W1; §9.1-2 | FG | ~4–6 | `assert_eq!(format_version, V2)`; golden-vector table per codepoint; `assert!(matches!(decode(v1_bytes), Err))`; single-bump-coverage meta-assertion. |

*Dropped duplicate IDs:* CE-A1≡WF-G1→F-W0-1; CE-A3≡WF-F2→F-W0-3; CE-A4≡WF-A2→F-W0-4; CE-A5≡WF-F1≡WF-F3→F-W0-5.

### Group 2 — Codepoint registry + dispatch (FG)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-CP-1** §4.0 codepoint integer pins (merges CE-B1 + WF-A1 + PMD-1 + L-codepoint pins) | every assigned integer wire-locked: sig `0x0001/0x0002/0x0003`; cipher `0x6400/0x647a/0x647b/0x647c`; **vault `0x6100`**; drop `0x6500`/**Sealed-Sender `0x6510`**/`0x6520`; **group `0x6610`**; MembershipSet band `0x6600/0x6610/0x6620`; Layer-D `0x6310..0x632F`; lifecycle `0x6700..0x67FF`; MLS/FS brackets `0x6380..0x63CF`; experimental/escape `0xFE00../0xFFFF` | §4.0 (whole table), §0.4, B-3; Inv-18; §9.1-9 | FG | ~24–28 | `assert_eq!(SYM.raw(), 0xNNNN, "wire-locked")` per row; `DROP_TO_RECIPIENT_SEALED_SENDER==0x6510` (the ONE canonical value, §0.4). Extends `canonical_bytes_v1_codepoints_and_aad.rs::codepoint_table_integer_values_pinned`. **Incorporates GAP-1b (`0x6100`).** |
| **F-CP-2** Intra-band non-collision + IANA-disjoint CI scanner (merges CE-B2 + WF-B1/B2 + L1 + GNI-21 + PMD-26) | every cp ∈ `0x6100..0x6FFF`, all integers unique, none in IANA HPKE `kem/kdf/aead` ranges; MembershipSet band disjoint from MLS bracket; Sealed-Sender exactly ONE value | §4.0 non-collision, §4.1, U8/U11/U15; Inv-18; NQ-W2; §9.1-9 | FG | ~4–6 | enumerate cps; `set.len()==vec.len()`; band/IANA disjointness; inject colliding const → scanner fires. **Authored at R2 as prerequisite per NQ-W2.** |
| **F-CP-3** Codepoint-dispatch strict-reject, no cross-variant fallback (merges CE-B3 + WF-D1 + GNI-20) | unknown/reserved cp → typed `UnsupportedAlgorithm`/`AeadError::Unsupported`, NEVER silent fallback (U2) | §2.1 C-3/C-7, §4.1; Inv-16; CLAUDE.md #5 | FG | ~6–8 | feed each reserved/unknown cp → `Err`; no path returns plaintext; cross-variant `BindingContext` mismatch reject. Extends `tf2_codepoint_dispatch_typed_unsupported_classical_downgrade.rs`. |
| **F-CP-4** `CodepointLifecycle` typed-state + band-integer pins (merges CE-B4 + WF-B3 + **GAP-1d**) | Live/Deprecated/Quarantined/Burned state machine; Quarantined/Burned reject; `0x6700..0x67FF` band-integer pins + one concrete lifecycle/revocation envelope round-trip | §4.1, U16; Inv-18 | FG | ~4–5 | `dispatch(burned_cp)→Err`; `dispatch(live_cp).is_ok()`; band-boundary integer pins. |
| **F-CP-5** Sig-side swap matrix `0x0002`/`0x0003` (**GAP-1a + GAP-1e cipher-twin on the sig axis**) | `0x0001` LIVE default; `0x0002` classical-Ed25519 = non-default explicit downgrade (never silent); `0x0003` MLDSA65⊕SLH-DSA typed-rejects as default + selectable for swap-matrix conformance only; sig combiner strip-resistance | §4.0 sig rows, NF-1; CLAUDE.md #5 | FG | ~5–7 | per-arm dispatch; unknown sig cp → typed-reject; extend `tf2_strip_resistance_negative.rs` into full sig-axis matrix. **Entirely absent in the 6 dimensions (all swap-matrix families were cipher-side).** |
| **F-CP-6** FS-future bracket typed-reject `0x63A0/0x63B0/0x63C0` (**GAP-1e**) | CGKA-Commit `0x63A0`, Bird-of-Prey `0x63B0`, draft-prabel `0x63C0` each typed-reject at v1-beta (beyond the `0x6380/0x6390` collision-guard) | §4.1 CODEPOINT-RESERVE; Inv-18 | FG | ~3–5 | each selecting at v1-beta → typed-reject. |
| **F-CP-7** MLS-bracket collision regression-guard `0x6380/0x6390` (merges L2 + WF-A1 §0.4 half) | `0x6380`=MLS-Application, `0x6390`=MLS-Welcome (NOT MembershipSet/Sealed-Sender — supersedes M-CONS-FINAL F8/F21); MembershipSetEncryption=`0x6600` | §0.4, §4.0, B-3 | FG | ~2–3 | `assert_eq!(MEMBERSHIP_SET_ENCRYPTION, 0x6600)`; `0x6380`→MLS reserve. |

*Dropped duplicate IDs:* CE-B1≡WF-A1≡PMD-1→F-CP-1; CE-B2≡WF-B1/B2≡L1≡GNI-21(registry-slice)≡PMD-26→F-CP-2; CE-B3≡WF-D1≡GNI-20(dispatch-slice)→F-CP-3; L2→F-CP-7.

### Group 3 — Inv-17 hybrid floor + full bidirectional swap matrix (FG)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-SM-1** Hybrid-mandatory floor; no pure-PQ LIVE/selectable (merges CE-C1 + T-K1 + WF-I2 + GNI-21 Inv-17-slice) | every KEM use-site is PQ⊕classical; `0x647c` PURE_PQ reachable ONLY via gated `try_pure_pq_sole_trust_path` (audit-flag); typed-rejected until flag flips; `0x647c≠0x647b` | §2.1 C-8, §5.1 Inv-17, m-3; #30 | FG | ~5–7 | `0x647c` default-path → `Err`; classical-half-present pin; extend `tf4_codepoint_0x647c_pure_pq_mlkem_only.rs` + `tf4_pure_pq_gated_audit_landed.rs`. |
| **F-SM-2** Full bidirectional cipher swap matrix (merges CE-C2 + T-K2) | hybrid-default + classical `0x6400` + no-encryption + NF-1 PQ⊕PQ `0x647b` arm, each a real built path, BOTH directions (R2 F-2) | §9.1-1, C-5/C-7, G-CORE-3c | FG | ~6–9 | per-config `open(seal(pt,cfg),cfg)==pt`; bidirectional pin; downgrade flips a real dispatch arm. Extends `tf4_gcore3c_full_swap_matrix_*.rs`. |
| **F-SM-3** Strip-resistance / committing-combiner negative (CE-C3) | removing/zeroing PQ-half OR classical-half fails decryption | §2.1 C-5/C-6, Inv-17 | FG | ~3–5 | `assert!(open(strip_pq_half(env)).is_err())`; `…strip_classical_half…`. proptest-fallback (no kani obligation here). |

### Group 4 — KEM-impl / KAT cross-impl + HPKE byte-accuracy (FG — golden-vector survival)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-KAT-1** libcrux ↔ RustCrypto FIPS-203 serialization KAT (merges CE-D1 + WF-G2) | libcrux-ml-kem swap preserves encap/ct/decap-key serialization byte-for-byte vs RustCrypto `ml-kem 0.2` — golden vectors survive the swap (m-2 Wave-0 exit gate) | §2.2 Q1/NQ-C2, §3.2, §7.1, §9.1-2 | FG/CF | ~4–6 | cross-impl byte-equality over FIPS-203 KAT corpus; negative wrong-seed. **External-vector seed.** |
| **F-KAT-2** `check-secret-independence` CI gate (CE-D2) | the libcrux hax/F* ML-KEM secret-independence CI gate is wired + green; absence = freeze blocker | §3.2 exit, §7.1 | FG (CI) | ~1–2 | CI-integration presence/green pin + feature-enabled assert. |
| **F-KAT-3** NQ-C1 HPKE KEM-extensibility byte-accuracy (CE-D3 — resolves BEFORE Canary-ENC-2) | whether McMillion `hpke` admits X25519MLKEM768 into a real RFC-9180 mode_base context byte-accurately, OR Benten supplies the KEM + reuses only the HPKE KDF/AEAD/key-schedule | §2.2, §3.3, §7.1, NQ-C1, §11-1 | FG (pre-canary) | ~3–5 | `assert_eq!(hpke_seal(ctx,pt,aad), RFC9180_REF)`; **author as PREREQUISITE, not trailing.** |
| **F-KAT-4** Cross-ecosystem LAMPS Composite ML-DSA interop (merges CE-D4 + WF-G3, both directions, NQ-C3) | Benten verifier accepts BouncyCastle/OpenSSL-3.5/OpenPGP-PQC `id-MLDSA65-Ed25519-SHA512` sigs AND Benten sigs verify there; OID `1.3.6.1.5.5.7.6.48` | §2.1 C-4, §4.0 `0x0001`, NQ-C3; #31 | FG/CF | ~4–6 | external-fixture inbound ×3 + outbound shape pin + mismatched-OID negative. **External-vector seed; surfaces fixture-acquisition.** R2 must confirm freeze-gating vs v1-GM-deferred. |

### Group 5 — Layer-A vault (FG vault format + FN)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-VA-1** Vault on-disk DAG-CBOR format freeze; XChaCha20 (m-4) (merges CE-E1 + WF-A3/A4-vault + T-J3) | `vault.cbor` `{k_principal[32], user_did_signing_key, creation_time:u64}` + Argon2id params + 16-B salt + **24-B XNonce**; uses `SymmetricAeadXNonce` (12-B would hit 2^32 reseal birthday); both 12-B + 24-B variants ship; nonce-width codepoint-discriminated | §3.1, §4.1, e2r §2.2, m-4; Inv-16 | FG | ~5–7 | vault-file hex-pin; `nonce.len()==24`; codepoint `SYMMETRIC_AEAD_XNONCE`; 12-B-on-XNonce-cp → decode reject; CBOR field-order pin. |
| **F-VA-2** Argon2id DAK derivation (CE-E2) | `DAK=HKDF-SHA256(Argon2id(pw,salt; m=19456,t=2,p=1), "benten-dak-v1")`; params persist + round-trip | §2.2, §3.1, e2r §2.2 | FG/FN | ~4–6 | `derive_dak(...)==KAT`; param-change ⇒ different DAK; info-tag domain-sep pin. |
| **F-VA-3** Constant-time wrong-password path (merges CE-E3 + T-J2) | wrong-password runs Argon2id+AEAD-decrypt to completion (no fast-fail early-return timing oracle) | §3.1; #34 | FN | ~2–4 | structural no-early-return-before-AEAD-open pin + coarse timing-invariance harness (best-effort). |
| **F-VA-4** Memory hygiene `secrecy::SecretBox` + `zeroize` (merges CE-E4 + #36/#39 disclosure) | `K_principal` in `SecretBox<[u8;32]>`, zeroized on Drop; `secrecy` new dep | §3.1, §5.2 #39, §6.2, O-1 | FN | ~2–3 | unlocked-key type is `SecretBox`; Drop-zeroization (best-effort post-drop scan). |
| **F-VA-5** Engine-lock + `EngineLocked` typed-reject (merges CE-E5 + T-J1) | pre-unlock ops → `EngineLocked`; post-unlock `UnlockedKeyMaterial{k_principal, user_did_signing_key}` gates all AEAD+UCAN-signing; vault hydrates both keys atomically | §3.1, §6.2 | FN/FG | ~3–5 | pre-unlock `put_node_encrypted → Err(EngineLocked)`; post-unlock `is_ok()`. |

### Group 6 — Layer-B per-Node AEAD / structural-KDF chain (FN + FG AAD)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-LB-1** `K(N)` structural-KDF chain on real `K_principal` (CE-F1) | `K(root)=HKDF(K_principal,"root"‖root_cid)`; `K(N)=HKDF(K(pred),"step"‖edge_label‖N.cid)`; same path→same key; diff pred→diff key; would-FAIL if `"step"` elided. Runs on real `K_principal` not stub | §3.2, §4.1, §7.1; CLAUDE.md #5 | FN/FG | ~5–7 | extend `tf3a_structural_kdf_step_root_derivation.rs` with real-`K_principal` arm; 5-Node parity; info-tag-elision negative. |
| **F-LB-2** Per-chunk/per-Recipe AAD truncation defense, BE-migrated (merges CE-F2 + WF-C3) | `aad_per_chunk(plaintext_cid, idx, total)` + `aad_per_recipe(...)` bind position+count; sliced list fails AEAD; domain-prefixes `benten-aead:chunk:` vs `:recipe:`; all index/count BE (F-W0-3 dep) | §4.1, §3.2, U3/U17; §9.1 | FG | ~5–7 | `open(sliced_list).is_err()`; cross-prefix-reinterpret negative; BE hex-pin. Extends `tf3d_inter_recipe_truncation_rejected.rs`. |
| **F-LB-3** Layer-B↔Layer-C KEM-DEM key-encryption mode (CE-F3, Q4) | sharing a Node = HPKE-wrap the small `K(N)`; recipient derives `K(N)` then AEAD-Opens body | §3.2, §2.3 Q4 | FN | ~3–4 | `recipient_open(body, derive_kn(hpke_unwrap(cek)))==body`. |

### Group 7 — Layer-C HPKE encrypt-to-recipient + Sealed-Sender (FG + FN)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-LC-1** HPKE mode_base single-recipient `HpkeBase[MLKEM768-X25519]` round-trip (CE-G1) | `0x647A` seals to one recipient + opens; ChaCha20-Poly1305; depends on F-KAT-3 | §3.3, §4.1, C-5/Q4; Inv-16 | FG | ~4–6 | `open(seal(pt,pk))==pt`; wrong-sk negative. |
| **F-LC-2** `HpkeMultiBase` group multi-stanza + cross-stanza AAD substitution (merges CE-G2 + T-E3 + PMD-21 + I4) | per-stanza AAD binds `(cp, body-CID, sorted recipient-DID-list, sender_did, stanza-index, recipient_key_generation)`; stanza-swap/reorder/re-target fails (U17); N recipients each open; defense in AAD NOT CID | §3.3, §4.1 (`0x6520`), U17/U19; Inv-16; #45/#46 | FG | ~6–8 | `open(swap_stanza(a,b)).is_err()`; reorder negative; re-target-to-different-recipient negative; multi-recipient parity. Extends `tf3b_scope_substitution_post_sign_rejected.rs`. |
| **F-LC-3** Sealed-Sender DEFAULT `0x6510` — sender-DID NOT on wire (merges CE-G3 + T-D1/D2 + PMD-2/3/4/5) | default path binds sender-DID INSIDE ciphertext (HPKE inner + post-decrypt-verify); on-wire AAD = audience+coarse-epoch only; `0x6500` sibling carries sender-DID-in-AAD (U4); forged inner sender-DID rejected; Inv-18 paired-disclosure satisfied (default IS metadata-hiding) | §2.0 BR-1, §2.3 Q5, §3.3, §4.0/§4.1, §5.2 #43; Inv-16/Inv-18 | FG | ~7–10 | scan default-path wire → `!contains(sender_did)`; **paired positive control**: `0x6500` MUST contain it; post-decrypt `recovered==sender_did`; tamper inner DID → reject. **Highest-novelty surface — canary-co-locate.** |
| **F-LC-4** DUAL-CID `envelope_blob_cid` vs `plaintext_cid` extending `TwoCidStore` (merges CE-G4 + I1 + PMD-18 + GNI-25) | `envelope_blob_cid=BLAKE3(serialized env)` changes on reseal; `plaintext_cid=BLAKE3(canonical payload)` stable + graph-referenced; **extends `two_cid_store.rs`/`two_cid_map.rs` (O-3), NOT net-new** | §2.3 Q3, §3.3, §4.1, O-3/F18 | FG/FN | ~4–6 | reseal: `plaintext_cid` equal, `envelope_blob_cid` distinct; `TwoCidStore` round-trip; graph-ref resolves via plaintext_cid. |
| **F-LC-5** `plaintext_cid_local` NEVER-serialized + `plaintext_cid_set` HMAC-blinded (merges CE-G4-local + I2/I3 + PMD-17/19 + GNI-25 + O-7) | `plaintext_cid_local` LOCAL-ONLY, in NO wire artifact (env/AAD/topic/blob/audit-Node); `plaintext_cid_set` HMAC-blinded under `K_Set` (set-scoped dedup, no cross-set linkage) | §3.3, §10.8 O-7; Inv-20 clause-d | FG | ~5–7 | sentinel `plaintext_cid_local` scanned absent across ALL serialization surfaces; `K_Set`-bit-flip ⇒ diff `plaintext_cid_set`; cross-set unlinkability. **kani/property — proptest floor (no kani in-tree); name kani arm as v1-GM strengthening.** |
| **F-LC-6** `recipient_key_generation` + `k_principal_generation` staleness (merges CE-G5 + **GAP-6a** U20) | stanza under stale `recipient_key_generation` (U19) rejected; **stanza under stale `k_principal_generation` after rotation (U20) rejected** | §3.3, §4.1, U19/U20 | FG | ~3–4 | `verify(stale_gen_stanza)→Err` for BOTH generation fields. |
| **F-LC-7** FS-gap honest disclosure (CE-G6) | HPKE-mode-base non-FS at long-term-sk axis (documented, not "fixed"); `DropToRecipient` forever-valid (#62); journalist per-msg FS (#56) deferred class | §3.3, §5.2 #42/#56/#62 | DC/FN | ~2–3 | `open_with_recovered_sk(old_env).is_ok()` documents the gap; revocation-reach doc-pin. Extends `tf3f_revocation_reach_forever_valid_documented.rs`. |
| **F-LC-8** Sealed-Sender abuse-control delivery-token (merges CE-G7 + T-D3/D4/D5 + PMD-6) | Sealed-Sender env WITHOUT valid recipient-issued UCAN delivery-token refused at receive boundary BEFORE decrypt; token-binding AAD sub-field (wire-affecting → pre-freeze); per-token rate-limit + revocation via UCAN `nbf`/`exp` | §2.0 BR-1, §3.11, §4.1, §5.2 #63 | FG/FN | ~6–8 | no-token → reject pre-decrypt; expired/over-rate → reject; valid → admit; extend `tf3e_replay_attack_ucan_expired.rs` + `rate_limit_policy.rs`. |
| **F-LC-9** Group-send Sealed-Sender posture `0x6520`/`0x6610` (**GAP-1f — DESIGN HOLE to surface**) | whether group multi-stanza sends carry plaintext sender-DID or honor Sealed-Sender; group `0x6610` round-trip distinct from `0x6520` (**GAP-1c**) | §3.3, §4.0/§4.1; Inv-18 | FG | ~4–6 | `0x6610` N-member K_Set group seal/open + per-stanza AAD; feed `0x6610`→`0x6520` dispatch → strict-reject; metadata-posture pin. **SURFACE to R2/Ben: if `0x6520`/`0x6610` carry plaintext sender-DID by construction, that silently defeats the Sealed-Sender default for every group send.** |

### Group 8 — Layer-D DAK / device-auth / remote-permission / multi-device-wrap (FG + FN)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-LD-1** `DeviceAuthBackend` sealed trait + headless (merges CE-H1 + GNI-22-DeviceAuth-slice) | sealed per #7; default Argon2id+XChaCha20+optional-keyring-core; headless via `BENTEN_VAULT_PASSWORD`/IPC | §3.4, §4.1, e2r §2/§4.4 | FN/FG | ~4–6 | headless env-var unlock round-trip; sealed-trait (no external impl); object-safe conformance per `kvbackend_conformance.rs` shape. |
| **F-LD-2** Remote-permission wire freeze `PermissionRequest`/`PermissionGrant` (merges CE-H2 + WF-A4-LayerD) | e2r §6.4 structs freeze; operation enum `Decrypt|SignUcanDelegation|RemoteUnlock|ExecuteWorkflow`; `0x6320..0x632F` | §3.4, §4.1, e2r §6 | FG | ~5–7 | golden CBOR per struct; sig round-trip; unknown-operation typed-reject. |
| **F-LD-3** `ExecuteWorkflow` codepoint-reserve slot + AAD-binding frozen (merges CE-H3 + T-H1, NQ-T3) | variant `{workflow_cid, input_node_cids, max_decrypt_count, result_recipient_pubkey, executor_did}` + AAD-binds `(executor_did, max_decrypt_count, result_recipient_pubkey)` frozen; runtime enforcement post-v1-beta but **AAD SUFFICIENT to express constraint** | §3.4, §4.1, M-3/U21, NQ-T3, §9.1-4 | FG | ~4–6 | variant exists; 3 fields in AAD (hex-pin); mutate each → Open fails (constraint bound not advisory); sufficiency assertion. **R2-gated on NQ-T3.** |
| **F-LD-4** Multi-device key-wrap `Provisioning*` freeze (merges CE-H4 + T-B1/B2/B3 + WF-A4-LayerD) | `ProvisioningOffer/Payload/InnerPayload{k_principal, user_did_signing_key+pubkey, atrium_memberships, provisioning_session_id, granted_at}` freeze; B's fresh X25519+ML-KEM-768 keypair receives HPKE inner (reuses Layer-C); session-id replay defense; user-DID-sig auth; **K_principal exfiltration to wrong device rejected (§10.2 HIGH)**; no FS for K_principal (identity-equivalent) | §3.4, §4.1, e2r §7; §10.2 | FG | ~7–10 | golden pin; HPKE to B's pubkey; substitute B's pubkey post-fingerprint → reject; replay old session-id → reject; forged/unsigned offer → `E_DEVICE_ATTESTATION_FORGED`-class; K_principal-FS-absent disclosure. Extends `device_attestation_envelope_direct.rs`. |
| **F-LD-5** Intra-hour-replay-rejected-by-nonce-cache (merges CE-H5 + T-A1 + T-C1/C2/C4 + PMD-11, M-1 — load-bearing) | re-presented `PermissionGrant`/DeviceLink within same 1-hr bucket rejected **by nonce-cache** (Compromise #25 substrate `handshake.rs:72`), NOT by time field; 1-hr bucket Layer-D-ONLY (M-14); nonce-cache retention ≥ full 1-hr window + durable-across-restart; rejection nonce-keyed not time-keyed | §3.4, §3.10, NQ-T4, §9.1-4, §10.8 seed; #25 | FG/FN | ~6–9 | present twice intra-bucket → 2nd `Err` from nonce-cache; **disable-cache negative** (replay would pass — bucket alone insufficient); restart-durability arm; clock-manipulation-both-ways → rejection unchanged. **R3 consumes NQ-T4 spec for multi-device shared-nonce semantics (T-C3).** |
| **F-LD-6** Remote-permission 6-class pre-merge mini-review (merges CE-H6 + T-A1..A7, M-12 — the §9.1-4 gate) | SIX executable pass-classes: (1) replay [F-LD-5]; (2) device-key-revocation (RotationLog cuts future grants); (3) clock-skew (NQ-T2: 1-hr bucket ⊥ `valid_until` enforcement clock); (4) confused-deputy (operation/audience bound, checked before time); (5) UI-deception (signed grant binds displayed operation-summary hash); (6) **audit-Node-binding — grant REJECTED if `audit_node_cid` absent/unresolvable**; audit-Node encrypted to device-mesh + replicated (NQ-T1) | §3.4, §9.1-4, M-12, NQ-T1/T2 | FG/FN | ~12–16 | one arm per class; headline `accept_grant(no_audit_cid)→Err`; clock-skew arm asserts 1-hr bucket ≠ `valid_until` clock (60s `valid_until` + 90s present → reject); confused-deputy extends `ucan_grounded_policy_rejects_proof_for_wrong_audience_before_time_check.rs`. **This group IS the harness the §6.7 mini-review reads.** |
| **F-LD-7** `keyring-core` + file-vault fallback (CE-H7) | `keyring-core` v1.0.0 (NOT legacy `keyring`) stores DAK-wrap; file-vault fallback; Tauri IPC smoke | §3.4, §2.2, §7.1 | FN | ~2–4 | store/retrieve round-trip; fallback when keychain absent. |
| **F-LD-8** Layer-D timestamp exclusion `DropToRecipient` (merges T-I1 + PMD-7/8/10, M-14) | `DropToRecipient`/Sealed-Sender struct carries NO `sealed_at`/`valid_until` (L6 finding closed-by-exclusion); 1-hr bucket on DeviceLink+RemotePermission ONLY; round-down-no-jitter (NQ-C5); bucket ⊥ `valid_until` clock (NQ-T2) | §3.10 M-14, §4.1, NQ-C5/T2; #62 | FG | ~5–7 | structural: drop struct has NO timestamp field; DeviceLink/RemotePermission DO; `bucket % 3600 == 0` no-jitter; merges CE-I3/T-I2/PMD-9 jitter pin here. |

### Group 9 — MembershipSet primitive + 5-role RBAC (FG + FN)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-MS-1** EXACTLY-3 `MembershipSetKind` structural-compile HALT-AND-SURFACE (merges A1 + WF-E1/E2 + GNI-1) | `{Atrium, DeviceMesh, SingleDevice}` EXACTLY-3, ordinal-stable; 4th arm = missing-pattern **compile error** (NOT `#[non_exhaustive]` wildcard); Garden/Grove NOT Kinds; `0x6600` bound to Kind; keying-reserves `AtriumWithRotatingGroupKey`/`EphemeralLobby` typed-reject | §1.3.A, §3.6.A, §4.2, §15.c HALT-AND-SURFACE, §9.1-5; #46 | FG | ~6–8 | exhaustive `match` no-wildcard (clone `tf3b_no_opaque_selector_arm_structural.rs` + `scope.rs:44`); `variant_count==3`; reserve-Kind → typed-reject. |
| **F-MS-2** Per-Kind constructor cardinality + Kind-determined `MemberRef` (merges A2 + WF-E3 + GNI-1) | Atrium ≥1 admin; DeviceMesh exactly-1 user-DID admin; SingleDevice exactly-1 self-admin; `MemberRef` Kind-determined (UserDid↔Atrium/DeviceDid↔DeviceMesh/LocalDevice↔SingleDevice) NOT nature; per-Kind wire-cost ceiling (#46: 32/5/1) | §3.6.A, §3.7, Inv-22; #46 | FN/FG | ~6–9 | 0-admin Atrium/2-admin DeviceMesh/2-member SingleDevice → `Err`; `MemberRef::DeviceDid` only in DeviceMesh. |
| **F-MS-3** `members_table` fusion one-DID-one-record (merges A3 + GNI-3) | `BTreeMap<Did, MemberEntry>` fuses members+authorities+role_assignments by construction; one DID→one record; `is_authority ⟹ sig_pubkey.is_some()`; ZERO `member_type`/nature field; snapshot = CURRENT-materialization of event chain | §2.5 BC-4, §3.5, §4.2, Inv-20 clause-i, Inv-22; §9.1-5 | FG | ~6–8 | second record for existing DID structurally impossible; `is_authority=true,sig_pubkey=None`→reject; compile-fence no `member_type` field. |
| **F-MS-4** RoleId 5-value ordinal golden-vector (merges C1 + T-F1 + WF-E4 + GNI-5, M-13) | `Invitee=0, Viewer=1, Member=2, Moderator=3, Admin=4` keying-AAD-bound; **supersedes M-CONS-FINAL Viewer=0/Invitee=1**; ship-all-5-active (Inv-20 clause-j corrected from "3 active 2 reserved") | §2.5 BC-9, §3.6.B, M-13, §4.2, §9.1-5, §10.8 seed; Inv-20 clause-j | FG | ~3–5 | `Invitee as u8==0 … Admin==4`; serialized-ordinal hex-pin (clone `tf2_no_hardcoded_sizes_ml_dsa65_vector.rs`); construct all 5 → all succeed. |
| **F-MS-5** Invitee derives ZERO content (merges C2 + T-F2 + GNI-5, M-11 hard floor) | Invitee (RoleId=0) gets NO `K(N)`, NO read cap — pre-acceptance handshake only | M-11, §3.6.B, §9.1-5, Inv-20 clause-j | FG | ~5–7 | admit Invitee → any `K(N)`/read → refused; Invitee UCAN template = NONE. **Most security-load-bearing RBAC constraint.** |
| **F-MS-6** Moderator ⊊ Admin strict-subset (merges C3 + T-F3 + GNI-5, M-11) | Moderator (read/write/share/moderate) ⊊ Admin; NO admit/kick/rotate/governance | M-11, §3.6.B, §9.1-5; #52/#60 | FN | ~6–8 | `admin.contains(mod)==true` AND `mod.contains(admin)==false`; Mod `kick`/`rotate` → denied (clone `tf3b_restricted_spec_contains_decidable.rs` + `grant_backed_policy_*.rs`). |
| **F-MS-7** Per-role UCAN ability-template golden-vectors (merges C4 + T-F4 + GNI-19) | each role's UCAN template pinned at canary + stable; semantics compose from UCAN + signed GovernanceConfig, NOT a frozen permission-flags bitfield | §3.6.B, R1-Q-9, §9.1-5 | CF/FG | ~6–8 | golden-vector per role; drift fails pin; permission-flags-bitfield grep-defense. |
| **F-MS-8** `role_assignments_generation` staleness `E_ROLE_STALE_AT_VERIFY` (merges B3 + T-E1) | stanza under stale `role_assignments_generation` rejected at verify (NEW ErrorCode) | §2.5 BC-5, §3.10, §3.6.B | FG | ~3–4 | seal at G → advance to G+1 → verify → `Err(E_ROLE_STALE_AT_VERIFY)`. |
| **F-MS-9** Role-transition survives prior UCAN attenuations + tight-`exp` (merges C5 + T-E5, #60) | ephemeral UCAN at role R survives downgrade (only `exp` bounds); §3.4 tight-`exp` default; honest #60 disclosure | #60, §3.4; #52 | FN/DC | ~3–4 | issue Admin UCAN → downgrade → still valid until `exp`; `exp-nbf ≤ default-bound`. |

### Group 10 — Membership AAD 9-tuple + HLC + Inv-21 + CRDT/MST/gossip convergence (FG + FN)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-AAD-1** `members_table` canonical-CBOR length-injective byte-pin (merges B1 + WF-C4 + GNI-2/3, NQ-W4 — flagship) | exact length-injective (U3) canonical-CBOR of AAD-bound `BTreeMap<Did,MemberEntry>` snapshot — field order, `Option<SigPubKey>` presence, `Hlc`, `RoleId` ordinal, `MemberRef` tag; **two engines materialize byte-identical AAD or a fork never converges** | NQ-W4, §3.5, §4.2, Inv-20 clause-c/i, U3 | FG | ~8–12 | hex-pin fixed fixture; length-extension/truncation non-collision; reorder ⇒ different bytes; BTreeMap-order-independent; re-serialize byte-identical. **One of two highest untested-byte risks.** |
| **F-AAD-2** AAD 9-tuple injectivity + opaque-bytes boundary (merges B2 + T-E2 + WF-C5 + GNI-2 + CE-I1) | `(codepoint, body-CID, sorted-member-DID-list, sender_did, stanza-index, member-key-generation, membership_set_id, membership_set_generation, role_assignments_generation)` — each field bound, mutating any → AEAD-open fails; per-stanza-LIVE vs envelope-CONSTANT partition; `benten-membership-set` hands OPAQUE `&[u8]` to `benten-crypto-suite` (NO reverse dep, m-15 GNC-5); canonical-TLV length-injective | §3.10, Inv-20 clause-c, U1/U3/U14, m-15 GNC-5, §4.2 | FG | ~12–16 | 9 single-field mutation arms → Open fails; sort-order negative; length-injectivity proptest; crypto-suite Cargo.toml no membership-set dep (compile-fence). **Encoding-injectivity (membership-side) here; crypto-binding round-trip co-owned — partition single-ownership.** |
| **F-HLC-1** `admitted_at_hlc` LWW (larger-HLC-wins) + clock-distinction (merges D1/D2 + GNI-7) | `admitted_at_hlc` (member property) = LWW larger-HLC (`crdt.rs:535`); `created_at_hlc` (set-anchor, immutable) ≠ `admitted_at_hlc`; ONLY `created_at_hlc` in Inv-21 tie-break (M-7) | §3.5.HLC, M-7, Inv-21 | FN/FG | ~5–7 | clone `hlc_loro_property_lww.rs`: concurrent admits t1<t2 → t2 wins; fork-fixture uses `created_at_hlc`; mutating `admitted_at_hlc` doesn't change fork winner; mutating `created_at_hlc` does. |
| **F-HLC-2** HLC-skew adversarial injection at membership-write boundary (merges D3 + GNI-3) | future-HLC membership write rejected by skew classifier (#25 substrate) | §3.10; #25 | FN | ~2–3 | clone `attack_hlc_skew_revocation_ordering.rs`: admit-write `physical_ms=u64::MAX/2` → `HlcSkewExceeded`. |
| **F-INV21-1** Smaller-`created_at_hlc`-wins asymmetry (merges E1 + GNI-7) | set-identity fork → **OLDEST anchor wins** (smaller HLC, deliberately opposite to property LWW); later adversarial re-fork with larger HLC NEVER displaces original | Inv-21, §3.8, §5.1, M-7 | FG | ~4–6 | two forks t1<t2 → t1 wins (a test passing under naive LWW must FAIL); adversary `u64::MAX` still loses. |
| **F-INV21-2** Tie-break TOTALITY via Version-Node-CID (merges E2 + GNI-8, M-8, NQ-D2) | `created_at_hlc` tie → `MembershipSetId` CANNOT disambiguate (concurrent forks share id); terminal discriminator = **forking-event Version-Node CID**; total order `(created_at_hlc ASC, fork_event_version_node_cid ASC)` | Inv-21, M-8, NQ-D2, §10.2 risk, §10.8 seed | FG | ~5–8 | tie fixture → Version-Node-CID decides NOT `MembershipSetId`; antisymmetry/transitivity/totality property pins. |
| **F-INV21-3** Inv-21 kani convergence proof (merges E2-kani + GNI-9, NQ-D2 — NET-NEW kani harness) | total + deterministic + commutative + associative + idempotent over bounded fork-set; **no kani in-tree** — stands up the harness | §3.8 "kani+property", §9.1-5 "kani-proven", §11 R3 deliverable, NQ-D2 | FG | ~3–5 | `#[kani::proof]` over tie-break fn; until kani lands → `#[ignore]` red-phase + proptest surrogate (clone `inv_13_*` red-phase + `proptest_algorithm_b_correctness.rs`). **One of two highest-risk families.** |
| **F-INV21-4** Losing-fork MUST-NOT-merge + archived-not-discarded + fork-as-DAG-branch (merges E3 + GNI-10) | losing fork's CRDT-vector NOT merged into winner (no silent absorption); archived-not-discarded; ALL event-authors fork; fork IS the DAG-branch over Anchor+Version+CURRENT; Inv-19 K(V) encrypts to immutable Version-Node-CID | Inv-21, §3.8, §2.5 Path-A.5, Inv-19, Inv-20 clause-f | FG | ~5–7 | winner has NO loser-only entry; loser anchor retained-not-CURRENT; non-Admin can fork; reuse `version_branched.rs`; key→immutable-Version-Node-CID. |
| **F-CRDT-1** Membership-set convergence proptest (concurrent admits/kicks/role-changes) (merges F1 + GNI-26) | any interleaving of N writers → same `members_table` snapshot post-merge; matches HLC-LWW per property + Inv-21 for set-identity | §3.8, §3.9 M-10, Inv-19, §10.8 seed | FG | ~3–5 | clone `prop_loro_converge.rs` (10k cases); admit/kick/role-change across 2–5 writers; snapshot-equality. |
| **F-CRDT-2** Convergence under out-of-order + duplicate delivery (merges F2/F3 + GNI-26) | causally-permuted + duplicated/replayed delivery → same converged snapshot (order-independent + idempotent) | §3.9 M-10 | FG | ~3–5 | permute delivery order → all converge; apply E, then E×2/×3 → snapshot unchanged. |
| **F-CRDT-3** LWW-property ∥ fork-set-identity co-existence (merges F4 + GNI-7) | the two rules co-exist over two object classes in same merge round without corruption (R0 does NOT claim byte-equivalence) | §3.8 Inv-21, M-7 | FG | ~3–4 | mixed fixture: property→larger-HLC, fork→smaller-`created_at_hlc`, simultaneously correct. |
| **F-MST-1** Membership MST diff convergence O(log n) (merges G1 + GNI-26) | event version-chain anti-entropies to convergence via `run_mst_diff_to_convergence` within `MAX_ROUNDS`; backstop holds independent of gossip | §3.9 M-10, §6.1; reuse `mst.rs:257` | FG | ~3–5 | clone `mst_diff.rs`: divergent event-sets → converge; `MAX_ROUNDS`-exceeded → typed `MstDiffError`. |
| **F-MST-2** MST entry CID-mismatch substitution defense (G2) | event with declared CID ≠ payload BLAKE3 rejected at app layer | §3.9; reuse `attack_mst_diff_cid_mismatch.rs` | FN | ~2–3 | mismatched declared CID → reject; matching → accept. |
| **F-MST-3** Revocation-vs-membership-write ordering priority (G3) | kick/revocation at HLC=T applied before any membership write at HLC<T from revoked party | §3.4; #52; reuse `mst_revocation_priority.rs` | FN | ~2–3 | kick DID-X at T ordered ahead of X's stale write at T−1. |
| **F-GOSSIP-1** `GossipTransport` placement + liveness-only (merges H1/H2 + G1 + GNI-26, NQ-D1) | `GossipTransport` lands in `benten-sync` (lean, NQ-D1), impl-in-isolation, NO iroh-concrete leak in trait sig; `benten-membership-set` defines NO transport; **dropping ALL gossip still converges via MST** (gossip = liveness-only, never the convergence path) | NQ-D1, §3.9 M-10, §6.1 B-1; reuse `transport_trait_boundary.rs` | FG/FN | ~5–7 | `MockGossipTransport` (channel-backed, no-iroh) compiles; iroh-leak breaks build; gossip-disabled → still converge; gossip-only (no MST) → NOT converge. |
| **F-GOSSIP-2** HMAC-blinded topic + fork-rotation + OOB (merges H3/H4 + PMD-12/13/14/15, NQ-D4, #61) | `topic=truncate(HMAC(K_Set, set_id‖generation_summary))`; current-gen members compute SAME topic; fork rotates generation ⇒ rotates topic; `generation_summary` = set-generation counter NOT per-member vector (O-5/m-11); observer without `K_Set` can't link/recover set_id; losing-fork re-converges onto winner's new topic via MST + OOB Drop-bundle rendezvous; topic is pure fn of `(K_Set, set_id, gen)` (no time input) | §3.9 P2 D6, §5.2 #61, NQ-D4, Inv-20 clause-d | FG/FN | ~7–10 | `K_Set`-bit-flip ⇒ diff topic; set_id not recoverable from topic; same-gen→same-topic; fork→diff-topic; two-clocks→identical-topic; OOB-less join impossible; doc-coupling for OOB first-contact residue (#43/#62). **Topic bytes = crypto-wire frozen rule §1.5.** |

### Group 11 — Federation (`SubsetRef`) + governance-as-graph + audit-as-version-chain (FG + FN)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-FED-1** `KSetAcquisitionPath` frozen field-set + offline-decidability (merges J1/J2/J3 + GNI-4, NQ-D3) | FROZEN `{target_set_id, hop_path:Vec<MembershipSetId> (≤4), acquisition_proof_cid}` makes depth-4 + cycle-detect decidable **offline from wire bytes alone** (path-carried NOT receiver-local); `MEMBERSHIP_RECURSION_MAX_DEPTH=4` (accept 4, reject 5); cycle `[A,B,C,A]` rejected | §3.6.C, M-9, NQ-D3, Inv-20 clause-k, §4.2 (`0x6620`) | FG | ~7–10 | offline verifier given ONLY `KSetAcquisitionPath` bytes decides depth+cycle (no DB); hex-pin field-set; depth-5 → `RecursionDepthExceeded`; cycle detection uses ONLY carried path. |
| **F-FED-2** `SubsetRef` refused-at-v1-beta + Model-B default (merges J4 + GNI-4) | `MemberRef::SubsetRef` reserved-and-REFUSED at v1-beta (typed-reject at `0x6620`); default Model-B (independent-`K_Set`-per-set); Model-A opt-in post-v1-beta additive | §3.6.C, Inv-20 clause-k/l, §4.2 | FG | ~3–4 | construct/admit `SubsetRef` at v1-beta → typed `UnsupportedAlgorithm`/`FederationReserved`; `0x6620` → typed-reject; Model-A not selectable. |
| **F-AUDIT-1** Audit-event via ENFORCED engine-API WRITE path (merges K1 + T-G1 + GNI-14, m-15 GNC-2) | admin ops emit audit Version Nodes through `is_actor_active`-gated WRITE so `(actor_cid, handler_cid, capability_grant_cid)` triple is SET; bare `put_node` leaves triple `None` (`store.rs:468`) — NOT the audit path; tamper-evidence NOT free | §2.6, §3.8 GNC-2, §4.2, §9.1-6 | FG/FN | ~6–8 | admit via Engine → triple populated; via backend `put_node` → triple `None` + chain not advanced (clone `attribution_mirror.rs`). |
| **F-AUDIT-2** Audit version-chain tamper-evidence + monotonic sequence (merges K1-tamper + T-G2 + GNI-15) | audit log = Anchor+immutable Version Nodes+CURRENT (Crosby-Wallach ≡); sequence advances monotonically per admin WRITE; dedup/no-op WRITE does NOT advance sequence/emit phantom event (Inv-13 side-channel re-asked) | §2.6, §3.8, GN-4, §9.1-6 | FG/FN | ~6–8 | clone `inv_13_dedup_path_does_not_advance_audit_sequence.rs` + `inv_13_dedup_does_not_emit_changeevent.rs`; tamper mid-chain Version Node → CID linkage fails (clone `get_node_verifies_content_hash_on_read.rs`); append-only. |
| **F-AUDIT-3** `AuditAccessGradation` UCAN read-scope on `audit:<set_id>:*` (merges K2 + T-G3 + GNI-16) | `AdminOnly` = only Admins hold read cap; `PublicAllMembers` = all members; scope is a `RestrictedScope` arm (m-15 GNC-1) NOT a codepoint; 4 reserved variants (MemberOnly/Threshold/TimeLocked/Anonymized) = UCAN-caveat/IVM compositions NOT codepoints | §2.5 BC-8, §3.8 GNC-1, M-17, §4.2; #58 | FG/FN | ~6–10 | AdminOnly + non-Admin → denied; PublicAllMembers + member → admit; `RestrictedScope` parses `audit:<set_id>:*` WITHOUT adding a 3rd `Scope` arm; no-audit-gradation-codepoint grep-defense (clone `view1_capability_grants.rs` + `tf3b_restricted_spec_contains_decidable.rs`). |
| **F-AUDIT-4** `audit_log_query` GRAPH-NATIVE NOT a frozen op + frozen-op-surface=12 (merges K2-M17 + T-G4 + GNI-17, NQ-W3) | `audit_log_query` composes 12 primitives + `audit:<set_id>:*` RestrictedScope, NO new frozen op signature; frozen op-surface = EXACTLY 12 `PrimitiveKind` variants (`subgraph.rs:71–93`); "24 ops" RETRACTED | §1.5 M-17, §4.2, NQ-W3, CLAUDE.md #1 | FG/CF | ~3–4 | `PrimitiveKind` exactly-12 arms (clone `tf3w_walker_is_a_subgraph_no_new_primitive_kind.rs`); `audit_log_query` is NOT a primitive (routes through READ + scope). |
| **F-GOV-1** GovernanceConfig top-level signed Node (merges K3 + GNI-18) | `GovernanceConfig{tier: Flat|Moderated|Polycentric}` top-level signed Node (InstallRecord precedent `PLUGIN-MANIFEST.md:68`), NEVER inside sealed `MembershipSetPolicy`; tier-promotion = add Node+grants+roles, NO re-key/NO new identity/NO Kind change; Garden/Grove = signed-Node content NOT sub-codepoints | §2.6, §3.6.B, §4.2, §9.1-6 | FG/FN | ~7–9 | promote Atrium→Garden → Kind unchanged, `K_Set` unchanged, new Node appears; GovernanceConfig NOT a Policy field (struct-fence); no Garden/Grove sub-codepoint (grep-defense). |
| **F-NAT-1** Inv-22 member-nature DERIVED never stored (merges K4 + GNI-11/12/13) | NO `MemberEntry`/Policy/wire nature field; `member_type` DELETED; `is_ai_operated(did)=(did.method()=="agent")` (did:agent = optional allowlist alias NOT stored discriminator); `is_plugin`/`is_autonomous_ai` derived (Inv-14/manifest); ownership from `root_issuers(agent_did)`; cached nature = IVM-materialized view never authoritative | Inv-22, §1.3, §3.7, §9.1-6 | FG/FN | ~6–8 | struct-fence (no nature field; clone `attribution_mirror.rs`); grep-defense `member_type`/`MemberKind` absent (clone `cap_r1_1_audience_binding_grep_defense.rs`); `is_ai_operated` from method-parse; IVM-view recomputes (clone `view2_event_dispatch.rs`); not-writable-as-authoritative. |

### Group 12 — Cross-cutting invariants + crate boundary + freeze-surface conformance (FG)

| F-ID | Pins | Mapping | Class | Size | Red-phase intent |
|---|---|---|---|---|---|
| **F-INV16-1** Inv-16 envelope-unification (merges CE-I1 + WF-J1 + GNI-20) | codepoint-dispatch + AAD-binding (U1) + strict-decode (U2) + canonical-TLV length-injective (U3) + sender-DID-or-Sealed-Sender + replay-window across all 4 layers; ONE HPKE primitive serves Layer-C drops + Layer-D wraps + remote-permission (not two impls) | §5.1 Inv-16, §2.1 C-2/C-3, §9.1-1/3 | FG | ~5–7 | cross-layer parametric U1–U3; one-HPKE-path-reused assertion; canonical-TLV injectivity. |
| **F-INV18-1** Inv-18 metadata-disclosure paired-Sealed-Sender (merges WF-J2 + PMD-5/22) | `0x6500` plaintext-sender variant MUST have paired Sealed-Sender sibling — satisfied by `0x6510` DEFAULT; residual on-wire metadata = exactly {audience, coarse-epoch}; `0x6500` discloses sender-DID-in-AAD (U4) | Inv-18, §3.3, BR-1, §5.2 #43/#63 | FG | ~4 | default = `0x6510` (sender-DID NOT in plaintext AAD); `0x6500` DOES carry it (U4); pairing exists; `observable_metadata=={audience,coarse_epoch}`. |
| **F-INV19-1** Inv-19 K(V) type-restriction (**GAP-3**) | K(V) API REJECTS payload that is not an immutable Version-Node-CID (or MembershipSet); would-FAIL = encrypting key material to a mutable/Anchor CID | §5.1 Inv-19, Inv-20 clause-f | FG | ~3–4 | feed non-Version-Node CID into K(V) → reject; reuse `version_branched.rs`. **Inv-19 had no dedicated family across the 6 dimensions (GNI-10 sub-point only).** |
| **F-NAT-2** Inv-22 unlinkability scope-honesty (merges PMD-20/25 + GNI-6, m-7) | per-recipient unlinkability = **network-observer-only**, does NOT protect against malicious admin (admin sees `members_table`); #58 insider-correlation survives as honest disclosure | §3.8 m-7, Inv-20 clause-d, §5.2 #58 | DC/FG | ~3–5 | network-observer can't link stanzas; admin CAN correlate (boundary asserted explicitly so not over-claimed); THREAT-MODEL.md states "network-observer-only". |
| **F-CRATE-1** EP-1 three-tier roster + sealed-trait + enum-dispatch + NO-registry (merges GNI-22) | Tier-1 open seams `{KVBackend, BlobBackend, GraphBackend, Renderer, Transport, Materializer, DeviceAuthBackend}` object-safe+conformance; Tier-2 sealed `{CapabilityPolicy, GrantReader#830, DeviceAuthBackend}`; Tier-3 `benten_ivm::Strategy` ENUM not trait (m-15 GNC-4); NO `EngineExtension`/`ExtensionRegistry`; `Scope` EXACTLY-2-arm | §2.7 EP-1, §6.3, §9.1-7 | FG/CF | ~8–10 | backend-trait object-safety (clone `graph_backend_trait.rs`/`blob_backend_trait_object_safety_*.rs`); `DeviceAuthBackend` sealed; no-registry grep-defense (clone `strategy_c_renamed_to_reserved_grep_assert.rs`); `Strategy` enum (clone `strategy_enum_present.rs`); `Scope` EXACTLY-2. |
| **F-CRATE-2** `benten-membership-set` 15th-crate SPLIT boundary (merges GNI-23, B-1) | crate exists; mechanism-half (frozen) = Kind enum + `0x6600/0x6610/0x6620` + multi-stanza keying glue (delegates to crypto-suite, NEVER forks #5) + `members_table` CBOR + AAD assembly + Inv-21 rule + clause-k bound; data-half (graph) = GovernanceConfig/RoleId-semantics/audit/federation/economics; deps {crypto-suite, core, caps, id, graph, **sync** B-1}; NONE depend on IT | §6.1, §6.3, §1.4, B-1, NQ-D1 | FG/CF | ~6–8 | exact B-1 dep set; crypto-suite + sync Cargo.toml no reverse dep; no `sha3::`/`chacha20`/`ml_kem::` direct primitive construction in membership crate (ONLY-call-site #5 grep-defense). |
| **F-FREEZE-1** Net-frozen-surface −8 tally + ZERO-hook confirmations (merges GNI-24 + **GAP-5b**, O-8) | GN wins shrink frozen surface −8 net (−1 audit structure, −1 `MembershipEvent` wire-enum, −4 AuditAccessGradation codepoints, −2 Garden/Grove sub-codepoints; R0.1 "−6" double-count corrected — O-8); `economic_policy` DROPPED (ZERO MembershipSet freeze hook); ZERO new member field; `PeerResource/ResourceKind/OwnerRef/CommunityEconomicPolicy` absent from frozen wire (D-28/D-29 PHASE-LATER-DEFER) | §4.4 O-8, §2.8 CE-1, §4.3, §9.1-6 | FG/CF | ~6–8 | `MembershipEvent` NOT frozen wire-enum (version-Node content); 4 AuditAccessGradation NOT codepoints; no Garden/Grove sub-codepoints; `MembershipSetPolicy` no `economic_policy` field (grep-defense); compute surfaces absent. |
| **F-NQC4-1** did:key hybrid-pubkey multicodec (merges CE-I2 + WF-H1, NQ-C4) | multicodec prefix for PQ-hybrid sig (Ed25519⊕ML-DSA-65) + KEM (X25519⊕ML-KEM-768) pubkeys in `did:key`; if no registered multiformats value, private-value-with-fallback reserved at G-CORE-9 round-trips; `did:agent:` optional allowlist alias | §10.6 NQ-C4, §4.1, U15; `did.rs:26,98` | FG | ~4–5 | `did_key_encode(hybrid_pk).prefix()==MULTICODEC`; round-trip; unknown multicodec → `UnknownMulticodec`; did:agent allowlist pin. **Surfaces multiformats-registration question if no value.** |
| **F-NQA1-1** Frozen-surface additive-extensibility (merges WF-D2/D3 + **GAP-4a/4b**, NQ-A1/W5) | new codepoint / `#[non_exhaustive]` variant / new reserved band does NOT break existing V2 envelope decode (NQ-A1 conservative fallback: post-freeze high/critical → reserve-codepoints + new tag, NEVER silent wire-break); reserved slots `{ExecuteWorkflow, SubsetRef@0x6620, RecoveryArtifact, RotatingGroupKeyChainedMode, ChainedStateTlv}` typed-reject at v1-beta; **`RecoveryArtifact` codepoint reserved at Core while `RecoveryHook` trait NOT frozen (NQ-W5/m-14)** | §1.5, §4.4, §4.1 CODEPOINT-RESERVE, NQ-A1/W5, M-3/U21/m-14 | FG | ~6–8 | seal V2, register new codepoint, re-decode OLD → byte-identical; `ExecuteWorkflow`/`SubsetRef` reserved-typed-reject; `RecoveryHook` trait grep-absent at Core; **GAP-6b `ChainedStateTlv` AAD-bind arm** (`Option<ChainedStateTlv>` sub-slot bound). |
| **F-DISC-1** Compromise disclosure-coherence catch-net (**GAP-2 — closes 18 rows in one parametrized family**) | for EACH Compromise #30..#63: (a) `SECURITY-POSTURE.md` row exists with correct `disposition_class` (ATO/SGD/CHD/OOS/MIT); (b) OOS/SGD disclosure text present + not over-claimed (#37/#38/#40/#44/#47/#49/#50/#51/#55/#57); (c) the BR-2 re-point triple (#31=LAMPS, #62=revocation-reach, #30=unaudited-PQ) at correct slots; #32 Decap-CT + #34 + #36 + #39 + #41 + #53 + #59 disclosures present | §5.2, §9.1-7, BR-2; pim-13 §3.12 | FG/DC | ~10–14 | parametrized doc-grep per disposition_class + re-point triple. **Reuses `tf3f` shape. The extra-reflection-pass elegant single-shape per `feedback_extra_reflection_pass_for_elegant_permanent_shape`.** |
| **F-DISC-2** Invariant + doc registration catch-net (**GAP-5a — §9.1-7 doc-wave gate**) | `INVARIANT-COVERAGE.md` REGISTERS Inv-16..22 AND Inv-15 NOT re-registered AND header count correct end-state (M-15); 4 new docs exist (`CRYPTO-CODEPOINTS.md`, `THREAT-MODEL.md`, `SECURITY-PROOFS.md`, `compute-marketplace.md`); `V1-FROZEN-INTERFACE-DEFERRED.md` has D-28/D-29 (in-tree max D-27); EP-1 roster + "Rust engine plugin" naming landed; §0.4 supersession recorded | §9.1-7, M-15/M-16; pim-13 | FG/CF | ~6–8 | per-doc existence + Inv-registration end-state pins. **No dimension owned the doc-wave deliverables that gate the freeze.** |
| **F-TRANS-1** TransportConfig reserve typed-reject (**GAP-2c**, #53) | Willow/iroh-roq/iroh-live `TransportConfig` variants typed-reject at v1-beta while `GossipPlusBlobs` ships | §5.2 #53, §3.9 | FG | ~3 | reserve transports → typed-reject; gossip ships. |

---

## 2. COVERAGE MATRIX

Every Inv-16..22, Compromise #30–#63, frozen/reserved codepoint, NQ-*, and §9 exit-criterion → covering family(ies). **No uncovered item remains** after the gap-fills; residual *thinness* is flagged in §5.

### 2.1 Invariants Inv-16..22

| Inv | Covering families | Status |
|---|---|---|
| Inv-16 envelope-unification | F-INV16-1, F-CP-3, F-W0-4, F-LC-1, F-AAD-2 | COVERED |
| Inv-17 hybrid floor | F-SM-1, F-SM-2, F-SM-3, F-CP-5 | COVERED |
| Inv-18 codepoint-registry + paired-disclosure | F-CP-2, F-CP-4, F-CP-6, F-CP-7, F-INV18-1 | COVERED |
| Inv-19 K(V) type-restriction | **F-INV19-1** (GAP-3 fill), F-INV21-4 | COVERED (was thin — now dedicated) |
| Inv-20 (12 clauses a–l) | F-MS-2/3 (a,i), F-AAD-1/2 (c), F-MS-4 (j), F-LC-5/F-NAT-2 (d), F-FED-1/2 (k,l), F-INV21-4 (f), F-AUDIT-3 (h) | COVERED |
| Inv-21 fork-tie-break | F-INV21-1/2/3/4, F-HLC-1, F-CRDT-3 | COVERED (strongest cluster) |
| Inv-22 member-nature derived | F-NAT-1, F-MS-2 (MemberRef boundary), F-NAT-2 | COVERED |

### 2.2 Compromise #30–#63

| # | Covering family | # | Covering family |
|---|---|---|---|
| #30 unaudited-PQ | F-SM-1, F-KAT-2, F-DISC-1 | #47 collab-edit-re-drop | **F-DISC-1** (was gap) |
| #31 LAMPS EUF-CMA | F-KAT-4, F-DISC-1 (re-point) | #48 MembershipSet-shape-leak | F-GOSSIP-2 (PMD-23 recovery), F-DISC-1 |
| #32 ML-KEM Decap CT | F-KAT-2, **F-DISC-1** (#32↔#30 disclosure) | #49 member-as-storage-host | **F-DISC-1** (was gap) |
| #33 coerced-approval | F-LD-6 (UI-deception), F-DISC-1 | #50 permanence-stewardship | **F-DISC-1** (was gap) |
| #34 password-knowledge | F-VA-3, **F-DISC-1** | #51 Tauri NAPI side-channel | F-LD-7, **F-DISC-1** (was gap) |
| #35 compromised-device retro | F-LD-4, F-DISC-1 | #52 role-downgrade subsume | F-MS-6, F-MS-9, F-MST-3 |
| #36 RAM/coredump OOS | F-VA-4, **F-DISC-1** | #53 TransportConfig reserve | **F-TRANS-1** (GAP-2c), F-DISC-1 |
| #37 no-TEE | **F-DISC-1** (was gap) | #54 continuous-rotation defer | F-MS-1 (keying-reserve), F-DISC-1 |
| #38 physical-presence OOS | **F-DISC-1** (was gap) | #55 GDPR-RTBF | **F-DISC-1** (was gap) |
| #39 supply-chain + secrecy | F-VA-4, **F-DISC-1** | #56 journalist per-msg FS | F-LC-7, F-DISC-1 |
| #40 reproducible-builds | **F-DISC-1** (was gap) | #57 grant-immutability | **F-DISC-1** (was gap) |
| #41 sync UX-vs-crypto + O-4 | F-LD-6 (revocation), F-DISC-1 | #58 insider-correlation | F-NAT-2, F-AUDIT-3, F-DISC-1 |
| #42 HPKE non-FS | F-LC-7, F-DISC-1 | #59 KEM-key-confirmation | F-LC-8 (abuse half), F-DISC-1 (audit-line) |
| #43 sender-metadata | F-LC-3, F-INV18-1, F-DISC-1 | #60 role-transition UCAN | F-MS-9, F-INV21-4 |
| #44 long-term-confidentiality | **F-DISC-1** (was gap) | #61 gossip-topic fingerprint | F-GOSSIP-2 |
| #45 MAL-BIND-K-CT/K-PK | F-LC-2 (app-binding), F-DISC-1 (§9.3 audit-line) | #62 revocation-reach | F-LC-7, F-LD-8, F-DISC-1 |
| #46 HpkeMultiBase O(N) cost | F-MS-2 | #63 Sealed-Sender abuse | F-LC-8, F-INV18-1 |

**All 34 rows covered.** The 18 honest-disclosure-only rows (#37/#38/#40/#44/#47/#49/#50/#55/#57 + thinly-touched #32/#34/#36/#39/#41/#51/#53/#54/#59) are closed by the single parametrized **F-DISC-1** per the extra-reflection-pass elegant-shape, plus the dedicated behavioral families for the load-bearing ones.

### 2.3 Frozen / reserved codepoints

| Codepoint / band | Family | Status |
|---|---|---|
| sig `0x0001` LAMPS | F-CP-1, F-KAT-4 | COVERED |
| sig `0x0002`/`0x0003` | **F-CP-5** (GAP-1a fill) | COVERED (was gap) |
| cipher `0x6400/0x647a/0x647b/0x647c` | F-CP-1, F-SM-1/2, F-W0-1 | COVERED |
| vault `0x6100` | **F-CP-1** (GAP-1b incorporated), F-VA-1 | COVERED (was gap) |
| drop `0x6500`/Sealed-Sender `0x6510`/`0x6520` | F-CP-1, F-LC-3, F-LC-9 | COVERED |
| group `0x6610` | **F-LC-9** (GAP-1c fill) | COVERED (was thin) |
| MembershipSet `0x6600/0x6610/0x6620` | F-CP-1, F-MS-1, F-FED-2 | COVERED |
| Layer-D `0x6310..0x632F` | F-CP-1, F-LD-2/4 | COVERED |
| lifecycle `0x6700..0x67FF` | **F-CP-4** (GAP-1d fill) | COVERED (was thin) |
| MLS `0x6380/0x6390` | F-CP-7 | COVERED |
| FS-future `0x63A0/0x63B0/0x63C0` | **F-CP-6** (GAP-1e fill) | COVERED (was thin) |
| experimental/escape `0xFE00../0xFFFF` | F-CP-1, F-NQA1-1 | COVERED |

### 2.4 NQ-* (20 questions)

| NQ | Family | NQ | Family |
|---|---|---|---|
| NQ-A1 frozen-surface additive | **F-NQA1-1** (GAP-4b) | NQ-D2 Inv-21 totality+kani | F-INV21-2, F-INV21-3 |
| NQ-A2 assessment-window placement | *no test by design* (tag-sequencing ratification) — flag §5 | NQ-D3 KSetAcquisitionPath | F-FED-1 |
| NQ-C1 HPKE byte-accuracy | F-KAT-3 (pre-canary) | NQ-D4 generation_summary | F-GOSSIP-2 |
| NQ-C2 libcrux↔RustCrypto KAT | F-KAT-1 | NQ-T1 audit-Node enc+replicated | F-LD-6 |
| NQ-C3 LAMPS cross-ecosystem | F-KAT-4 | NQ-T2 bucket ⊥ valid_until | F-LD-6, F-LD-8 |
| NQ-C4 did:key multicodec | F-NQC4-1 | NQ-T3 ExecuteWorkflow AAD-sufficiency | F-LD-3 |
| NQ-C5 jitter↔skew | F-LD-8 | NQ-T4 nonce-cache spec | F-LD-5 |
| NQ-D1 gossip placement/convergence | F-GOSSIP-1 | NQ-W1 one V2 bump | F-W0-5 |
| NQ-W2 codepoint scanner | F-CP-2 | NQ-W3 frozen-op=12 | F-AUDIT-4 |
| NQ-W4 members_table bytes | F-AAD-1 | NQ-W5 RecoveryArtifact reserve / trait-absent | **F-NQA1-1** (GAP-4a) |

**All 20 covered** except NQ-A2 which is a tag-sequencing ratification, not a unit-test family (flagged §5).

### 2.5 §9 exit-criteria (13 items)

| § | Covering families |
|---|---|
| §9.1-1 EncryptedEnvelope + swap matrix | F-W0-4, F-CP-1/3, F-SM-2, F-INV16-1, F-LC-1/2 |
| §9.1-2 Wave-0 (X-Wing + BE + rename + KATs) | F-W0-1..5, F-KAT-1/2 |
| §9.1-3 Layer-C + Inv-16 + DUAL-CID | F-LC-1..9, F-INV16-1, F-INV18-1 |
| §9.1-4 Layer-D + 6-class mini-review + nonce-cache | F-LD-1..8 (the 6 pass-classes = F-LD-6 1:1) |
| §9.1-5 MembershipSet + RoleId + Inv-20/21/22 + gossip | F-MS-1..9, F-AAD-1/2, F-INV21-1..4, F-CRDT-1..3, F-MST-1..3, F-GOSSIP-1/2, F-LC-4/5 |
| §9.1-6 governance/audit graph-native + ZERO-hook + ZERO-nature | F-AUDIT-1..4, F-GOV-1, F-NAT-1, F-FREEZE-1 |
| §9.1-7 REGISTER Inv-16..22 + Compromise + 4 docs + D-28/29 + EP-1 | **F-DISC-1, F-DISC-2** (the doc-wave catch-net — was the single biggest registration gap), F-CRATE-1 |
| §9.1-8 R6 council 0 substantive | *process, no family* |
| §9.1-9 G-CORE-9 freeze + §4.0 authoritative + additive | F-CP-1/2, F-AUDIT-4, F-FREEZE-1, F-NQA1-1 |
| §9.2-10 Composing UX on frozen substrate | *Phase-4-Meta-Composing — out of Core R3 scope; F-NQA1-1 pins the RecoveryHook-trait-absent-at-Core boundary* |
| §9.3-11 v1-assessment-window | *process* |
| §9.3-12 pre-tag external audit (M-6 IND-CCA2) | *audit-deliverable, not a unit family; F-DISC-1 pins the #45/#59/M-6 audit-line disclosure* |
| §9.3-13 v1-GM gate (#30 closes) | F-KAT-2 (audit-gate), F-DISC-1 (#30 disclosure) |

---

## 3. R3 TEST-WRITER SLICING (N parallel waves, disjoint file/crate ownership)

The slicing is **canary-first + freeze-gating-first**, partitioned by crate/file ownership so the waves are disjoint and parallelizable under the §13 7-implementer cap. The Wave-0 migration is the hard upstream that gates everything; the canaries that mint wire (`EncryptedEnvelope`, MembershipSet, Layer-D) own the home files others extend.

**Wave structure (dependency-ordered; ⟂ = parallel within a tier):**

**TIER 0 — Wave-0 canary (SOLE upstream; everything blocks on it).**
- **R3-W0** `crates/benten-crypto-suite/tests/` + `src/{aead.rs, cipher_suite.rs, codepoint.rs}` — **F-W0-1..5, F-CP-1..7, F-SM-1..3, F-KAT-3, F-INV16-1.** This is the canary: it mints `EncryptedEnvelope`/`BindingContext`, the §4.0 codepoint integer pins, V2/BE migration, real X-Wing, the dispatch/typed-reject/swap-matrix. **F-KAT-3 (NQ-C1) authored FIRST as a prerequisite** (gates the byte-format before Layer-C). No other wave authors envelope bytes until R3-W0's red-phase corpus is on origin. ~70–90 tests.

**TIER 1 — fan-out after R3-W0 (⟂, disjoint crates).**
- **R3-W1 (crypto-KAT)** `crates/benten-crypto-suite/tests/` (KAT slice) — **F-KAT-1/2/4, F-VA-1..5, F-LB-1..3.** Vault + Layer-A/B + cross-impl KATs. ~35–50 tests. *(External-vector seeds: F-KAT-1/4 — surface fixture-acquisition; deterministic synthesized witness fallback per `tf4 load_fips_204_kat_vector_for_test` precedent if vectors unavailable, flag real-corpus swap for R5.)*
- **R3-W2 (Layer-C)** `crates/benten-drop/tests/` + `crates/benten-sync/src/two_cid_store.rs` + `benten-graph/src/two_cid_map.rs` — **F-LC-1..9, F-INV18-1.** Sealed-Sender, multi-stanza, DUAL-CID, abuse-control. **F-LC-3/5/9 canary-co-located (100% greenfield — Sealed-Sender/plaintext_cid_local/group-posture).** ~45–60 tests.
- **R3-W3 (Layer-D / threat)** `crates/benten-engine/tests/` + `benten-sync/tests/` (handshake/nonce-cache) — **F-LD-1..8, F-VA-3 (constant-time shares with W1 — assign to W3 to keep vault-format in W1).** Remote-permission, nonce-cache, multi-device-wrap, the 6-class mini-review harness. ~50–65 tests. *(R2-gated arms: F-LD-3 on NQ-T3; F-LD-5 multi-device-nonce on NQ-T4 — author the resolved-spec arms once R2 ratifies.)*

**TIER 2 — MembershipSet crate (after R3-W0 mints codepoints; ⟂ internally by file).** New crate `crates/benten-membership-set/tests/` — split 3 ways to respect the cap:
- **R3-W4 (MS-structure + RBAC)** — **F-MS-1..9, F-FED-1/2, F-CRATE-1/2.** Kind enum, members_table, RoleId, federation, crate boundary. F-MS-1/3/4 canary-co-located (greenfield primitive). ~50–65 tests.
- **R3-W5 (MS-AAD + Inv-21 + CRDT/MST/gossip)** `benten-membership-set/tests/` + reuse `benten-sync/tests/` harnesses — **F-AAD-1/2, F-HLC-1/2, F-INV21-1..4, F-CRDT-1..3, F-MST-1..3, F-GOSSIP-1/2.** **F-INV21-3 (kani) stands up the kani harness — flag for R3 sizing (net-new infra; proptest surrogate + `#[ignore]` red-phase until kani lands).** ~45–60 tests.
- **R3-W6 (governance + audit + nature + Inv-19)** `benten-membership-set/tests/` + `benten-engine/tests/` (enforced-write path) — **F-AUDIT-1..4, F-GOV-1, F-NAT-1/2, F-INV19-1.** ~40–55 tests.

**TIER 3 — doc-wave + freeze conformance (after all wire lands; mostly doc-coupling/grep, low cap pressure, can run early-parallel since read-only).**
- **R3-W7 (registration + disclosure catch-net)** doc-coupling families — **F-DISC-1, F-DISC-2, F-FREEZE-1, F-NQA1-1, F-NQC4-1, F-TRANS-1.** These are the §9.1-7 doc-wave gate + the additive-extensibility/freeze-tally conformance + did:key. Read-only / grep / doc-coupling → cap-EXEMPT, can dispatch alongside Tier 0–2. ~30–45 tests.

**Slicing rationale:** R3-W0 is the single canary all wire depends on (canary-first per `feedback_canary_first_parallel_implementation`). Tiers 1–2 fan out by disjoint crate. The cross-dimension seams (AAD 9-tuple crypto-binding vs membership-encoding; gossip-topic-bytes vs convergence; RestrictedScope parse) are resolved by **single-ownership assignment**: F-AAD-2 membership-encoding+opaque-boundary → R3-W5; the crypto-binding round-trip stays in R3-W0's `tf2`-family (NOT re-implemented in W5). F-GOSSIP-2 topic-bytes → R3-W5 (crypto-wire §1.5 rule), convergence → same wave (no split needed since one crate). F-AUDIT-3 `audit:<set_id>:*` parse → R3-W6 (NOT duplicated in a caps-dimension wave). 7 waves ≤ the 7-implementer cap at peak; Tier-3 is cap-exempt.

---

## 4. FREEZE-GATING TEST PRIORITIES (MUST be green before any wire-minting canary)

A wire-minting canary that lands ahead of these freezes an untested byte. Priority order:

**P0 — gate R3-W0 itself (author FIRST, before any envelope byte is minted):**
1. **F-KAT-3 (NQ-C1)** — the HPKE KEM-extensibility byte-accuracy investigation. **Resolves BEFORE Canary-ENC-2.** Determines whether the entire Layer-C/HpkeBase/HpkeMultiBase byte-format is RFC-9180-faithful or Benten-supplies-the-KEM. Author as a *prerequisite*, not trailing.
2. **F-CP-1 + F-CP-2 (NQ-W2)** — the §4.0 codepoint integer pins + the non-collision/IANA-disjoint scanner. The foundation every envelope family builds on; the scanner is authored at R2 as a prerequisite per NQ-W2.
3. **F-W0-3 (M-19)** — the zero-`to_le_bytes` scanner + BE byte-pins. **Single highest-leverage conformance gate** — one surviving LE field is a permanent cross-engine wire incompatibility (LE is present TODAY at `aead.rs:165,244,277` + `aead_wrap.rs` + `plugin_manifest.rs`).
4. **F-W0-1/2 + F-KAT-1 (BR-3, NQ-C2)** — real X-Wing SHA3-256 construction + interop KAT + libcrux↔RustCrypto FIPS-203 serialization KAT. No in-tree precedent encodes the real X-Wing (the `tf3a` test encodes the HKDF stand-in — its KAT expectation must FLIP). Golden-vector survival.

**P1 — the canonical-bytes / AAD contract (the two highest untested-byte risks):**
5. **F-AAD-1 (NQ-W4)** — `members_table` canonical-CBOR length-injective bytes. **Divergent AAD silently breaks cross-engine decrypt for the *same* membership** — manifests only under cross-engine fork.
6. **F-AAD-2 + F-LB-2 + F-LC-2** — the AAD 9-tuple injectivity + per-chunk/per-recipe truncation + cross-stanza substitution. The U3 length-injectivity is the flagship structural property; a non-injective encoding silently breaks fork convergence and enables truncation/substitution.

**P2 — the Inv-21 convergence proof (the divergence-bug risk):**
7. **F-INV21-2/3 (NQ-D2)** — totality-via-Version-Node-CID + the kani convergence proof. A non-total tie-break is a convergence-divergence bug that only manifests under adversarial concurrent same-anchor forks. **F-INV21-3 stands up net-new kani infra** (proptest surrogate is the v1-beta floor; kani as v1-GM strengthening).

**P3 — the metadata-posture + Sealed-Sender wire (greenfield, highest novelty):**
8. **F-LC-3 + F-INV18-1** — Sealed-Sender DEFAULT `0x6510` sender-DID-not-on-wire + the Inv-18 paired-disclosure. 100% greenfield; the metadata posture IS the wire contract.
9. **F-LC-9 (GAP-1f) — SURFACE FIRST:** the group-send Sealed-Sender posture (`0x6520`/`0x6610`) is an **unresolved design hole**, not just a missing test. If group sends carry plaintext sender-DID by construction, that silently defeats the Sealed-Sender default for every group send. **R2 must resolve before the group codepoints freeze.**
10. **F-LC-5 (O-7)** — `plaintext_cid_local` never-serialized (kani/property; proptest floor).

**P4 — the registration catch-net (gates the doc-wave that gates the freeze):**
11. **F-DISC-1 + F-DISC-2** — Compromise disclosure-coherence + invariant/doc registration. These were the single biggest registration gap; they are the §9.1-7 gate (pim-13 spec-to-code compliance) and gate the doc-wave that gates G-CORE-9.

---

## 5. R2→R3 OPEN QUESTIONS CARRIED FORWARD

**A. Design holes to SURFACE to R2/Ben (not just missing tests):**
1. **F-LC-9 / GAP-1f — group-send Sealed-Sender metadata posture.** The plan does not state whether `0x6520`/`0x6610` group multi-stanza sends honor the Sealed-Sender default or carry plaintext sender-DID by construction. If the latter, the Sealed-Sender DEFAULT (BR-1) is silently defeated for every group send. **Needs a ratified answer before the group codepoints freeze.** *Orchestrator prediction: Ben rules group sends MUST honor Sealed-Sender (consistency with the single-recipient default), which adds a per-stanza inner-sender-DID binding to the group AAD — a pre-freeze wire change.*
2. **NQ-A2 — assessment-window placement.** A tag-sequencing ratification (window between `phase-4-meta-close` and `v1-beta`), not a unit-test family. Confirm with R2/Ben whether it gets ANY doc-conformance pin (F-DISC-2 could carry a "exit-criteria docs name the window" arm) or is intentionally test-free.

**B. R2-resolution-gated test arms (R3 cannot pin the exact assertion until the NQ resolves):**
3. **F-LD-3 (NQ-T3)** — ExecuteWorkflow frozen-AAD sufficiency must be confirmed before the variant slot freezes.
4. **F-LD-5 / T-C3 (NQ-T4)** — multi-device shared-nonce semantics (does device C reject a nonce device B consumed? after-sync vs pre-sync best-effort) gated on the ratified NQ-T4 default.
5. **F-LD-6 (NQ-T2)** — the 1-hr bucket ⊥ `valid_until` enforcement-clock decoupling.
6. **F-LD-8 (NQ-C5)** — round-down-no-jitter epoch granularity.
7. **F-KAT-4 (NQ-C3)** — R2 must confirm whether cross-ecosystem LAMPS interop is freeze-gating-at-v1-beta or v1-GM-deferred (enumerated freeze-gating-by-default; flag the decision).

**C. Net-new test infrastructure (flag for R3 sizing):**
8. **kani harness is NOT in-tree** (0 `#[kani::proof]` at both SHAs). F-INV21-3 + the kani arms of F-LC-5/F-INV19-1 stand it up. The v1-beta floor is **proptest** (the `prop_mst.rs`/`prop_loro_converge.rs` 10k-case harness); name kani arms as v1-GM strengthening. **Do NOT block the wave on kani standup.**

**D. External-vector / fixture-acquisition seeds (freeze-gating, may need real-corpus swap at R5):**
9. **F-W0-2** (draft-connolly X-Wing KAT), **F-KAT-1** (FIPS-203 KAT corpus), **F-KAT-4** (BouncyCastle/OpenSSL/OpenPGP LAMPS sigs), **F-NQC4-1** (multicodec registration). If unavailable at write-time, use a deterministic synthesized witness (the `tf4 load_fips_204_kat_vector_for_test` precedent) and flag the real-corpus swap for R5.

**E. Tree-divergence (orchestrator action):**
10. The plan-doc is pinned to `2172cb6d`; live HEAD `bc592e75` is **NOT a descendant**. R3 test-writers MUST rebase the byte-pinning families onto whichever SHA the Wave-0 canary (R3-W0) lands on. Every freeze-gating family authors V2+BE+`EncryptedEnvelope` from first commit (M-20).

---

## RESIDUAL COVERAGE GAP — HONEST ASSESSMENT

After the union + gap-fills, **zero items are uncovered**. The residual *thinness* (now flagged, not unfilled):

- **F-DISC-1 carries 18 Compromise rows in one parametrized family.** This is the deliberate extra-reflection-pass elegant-shape (per `feedback_extra_reflection_pass_for_elegant_permanent_shape`) — strictly less code than 18 separate disclosure families — but it is a **single point of failure**: if the parametrization misses a `disposition_class`, 18 disclosures regress together. Mitigation: the family is itself parametrized over the full `SECURITY-POSTURE.md` row-set (not a hand-list), so a new Compromise row is auto-included. R4 should verify the parametrization enumerates from the doc, not a literal.
- **§9.1-2/12 audit-deliverables (#45 MAL-BIND, #59 KEM-key-confirmation, M-6 IND-CCA2-adversarial-seed)** are correctly **NOT unit-test families** — they are external-cryptographer-audit scope lines. F-DISC-1 pins only their disclosure-coherence (that they appear as audit lines). R3 must NOT fabricate a "proof" test for these.
- **NQ-A2** has no test family by design (tag-sequencing ratification). Confirmed acceptable; flagged for R2.
- **§9.2-10 (Phase-4-Meta-Composing)** is out of Core R3 scope; only the Core-side boundary (RecoveryHook trait absent at Core, F-NQA1-1) is pinned now.

**Single highest untested-byte risk if a family is dropped:** **F-AAD-1 (NQ-W4 members_table bytes)** and **F-INV21-3 (NQ-D2 kani totality)** — the former silently breaks cross-engine decrypt for the same membership; the latter is a convergence-divergence bug under adversarial concurrent forks. Both are P1/P2 freeze-gating, canary-co-located in R3-W5.

**Family count:** 12 groups, **~95 unified families** (down from ~163 raw across the 6 dimensions + 22 gap-fills via seam-deduplication), **~720–950 red-phase tests**. **~74 of ~95 families are FREEZE-GATING** (must be green before G-CORE-9). The deduplication ratio (~163+22 → 95) is the seam-merge: every `[SEAM]`-flagged cross-dimension duplicate collapsed to one F-ID with single ownership.
