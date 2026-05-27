# Option F+ §6.2 envelope-layer-unification — C5 CRITIQUE (FORMAL-METHODS LENS)

**Branch:** `phase-4-meta-core/option-f-plus-critique-c5-formal-methods`
**Role:** SENIOR FORMAL-METHODS CRYPTOGRAPHER critiquing the 9-eyes panel + consolidation. Fill the "no-formal-methods-lens" coverage gap surfaced by the consolidator in MF5.
**Date:** 2026-05-27
**Tree-state pre-flight:** worktree clean against `origin/main @ 2172cb6d`; branch created from worktree HEAD (== origin/main).
**Inputs consolidated:** `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` (939 LOC); 9 origin lens reviews + e2r-ffull-scope-review @ `220b5aae`; main docs referenced via grep.
**Authority:** ADVISORY. Not load-bearing on the consolidator's recommendations. Surfaces formal-verification posture as a v1-beta/v1-GM/post-v1-GM strategic axis Ben should ratify independently.

---

## §1 Executive verdict + confidence

**Top-line verdict.** The 9-eyes panel + consolidation are formal-methods-naive but not formal-methods-hostile. Several amendments (U6, U31, U39) already gesture in the right direction (libcrux-ml-kem-as-formally-verified-dependency; kani injectivity proof of TLV serialization). What is missing is a **coherent formal-verification posture** spanning (a) which targets matter for v1-beta-tag vs v1-GM-audit vs post-v1-GM contribution; (b) which tools fit Benten's stack (pure-Rust + NAPI-RS-bridged TS); (c) how proofs compose with the `docs/THREAT-MODEL.md` recommended by L5; (d) where Benten benefits FOR FREE from upstream verification (libcrux) vs where it needs first-party proof investment.

**Bottom-line v1-beta recommendation (consolidator amend).**
1. **PROMOTE U39 (kani TLV-injectivity) from RECOMMENDED to LOAD-BEARING** — closes U3 collision class mechanically; 2-3 wave-days; high leverage for audit.
2. **ADD U39b — kani strict-decode dispatch correctness for U2 + codepoint endianness (BE/LE) round-trip injectivity for U7** — ~1.5 wave-days; closes a class of decoder bugs.
3. **ADD U39c — kani replay-window monotonicity + clock-skew bounds for U5** — ~1 wave-day; closes ill-formed expiry-check class.
4. **ADD U39d — kani CodepointLifecycle state-machine for U16** — ~0.5 wave-days; closes lifecycle illegal-transition class.
5. **PROMOTE U31 libcrux-ml-kem ANALYSIS DOCUMENT (NEW)** — write a 1-page `docs/SECURITY-PROOFS.md` §1 entry that records *exactly what libcrux proves, by which tool, against which spec, at which version pin* so external auditors don't re-derive. ~0.5 wave-day.
6. **MINT `docs/SECURITY-PROOFS.md`** as audit-deliverable sibling to `docs/THREAT-MODEL.md`. Companion not replacement. ~1 wave-day skeleton; ~1.5 wave-days populated.

**v1-GM-audit-window additions (pre-audit-gate).**
7. **PROVERIF or TAMARIN MULTI-STANZA HPKE RECIPIENT-ISOLATION MODEL for U17 + U25** — symbolic-protocol proof that cross-stanza substitution + per-recipient-unlinkability hold under Dolev-Yao + adaptive-corruption. ~5-8 wave-days. This is the single highest-value first-party formal verification investment for Benten.
8. **TAMARIN MODEL OF EXECUTEWORKFLOW CAPABILITY-BINDING (U21)** — proof that capability fields in AAD bind authorization correctly across delegation chain; ~3-5 wave-days. Can be deferred to G-CORE-PRIVACY-1 wave or earlier if capability-delegation gets used at v1-beta.
9. **CT-VALIDATION VIA dudect (U38) + ctgrind PROOF-OBLIGATION DOCUMENT** — record what dudect's negative result means + does not mean, what threshold (p<0.01 at 10⁶ samples) implies about CT-property strength + residual uncertainty. ~0.5 wave-day.

**Post-v1-GM strategic.**
10. CGKA/MLS-PQ symbolic verification when minted (U13 codepoint brackets reserved).
11. Creusot-class proof of `canonical_binding()` total-correctness for v2 audit (post-v1-GM).
12. hax-toolchain integration of `benten-crypto-suite::aead` for compile-time secret-independence on Benten-authored AEAD helpers (post-v1-GM).
13. Contribute Tamarin model upstream to MLS-WG / CFRG (post-v1-GM strategic deposit).

**Total formal-methods incremental cost over L4's ~35-45-wave-day baseline.**
- **v1-beta-LOAD-BEARING add:** ~5-7 wave-days (kani harnesses + SECURITY-PROOFS.md mint + libcrux pin-documentation).
- **v1-GM-DEFER add:** ~8-13 wave-days (tamarin/proverif multi-stanza model + ExecuteWorkflow model + CT-validation document).
- **POST-v1-GM strategic:** ~15-25 wave-days (CGKA/MLS-PQ verification + Creusot total-correctness + hax integration + upstream contribution).

**Confidence:** HIGH on tool-selection reasoning. HIGH on the v1-beta kani subset (well within kani's bounded-model-checker sweet spot). MED-HIGH on tamarin/proverif effort estimates (real-world IETF protocol-verification efforts span 2-6 person-weeks; Benten's HpkeMultiBase is simpler than full MLS but more complex than vanilla HPKE). MED on the libcrux-as-zero-cost claim — depends on which exact version pin Benten lands on (see §3.5).

**Top concerns the panel missed.**
- A1. **No formal-methods POSTURE.** Recommendations are scattered (U6+U31+U38+U39); no document articulates "this is what Benten formally verifies, why, by which tool, at what version."
- A2. **The U31 libcrux dependency claim is verification-by-pointer.** External auditors will ask "verified against which spec, by which tool, at which version, to which strength." None of L4 / L5 / consolidator addresses this. SECURITY-PROOFS.md §1 closes this in <1 day.
- A3. **Cross-stanza substitution defense (U17 A1) is informal.** L9 designed the AAD per-stanza binding; no lens verified it holds under adaptive recipient corruption + cross-codepoint substitution. This is the single largest first-party formal-verification gap. ProVerif or Tamarin model is the right answer.
- A4. **CodepointLifecycle (U16) is a state-machine.** Kani proves illegal-transition impossibility in ~0.5 wave-day; the consolidator scoped this as ~50 LOC Rust without flagging the formal-verification opportunity.
- A5. **L4 IMPL-B4 kani proof is scoped too narrow.** "TLV injectivity under bounded shapes" is good but should be extended to: codepoint-dispatch round-trip; aad_version round-trip; CodepointLifecycle transition; replay-window monotonicity. All within kani's sweet spot.
- A6. **`canonical_binding()` is Benten's most-load-bearing internal cryptographic primitive AFTER the underlying AEAD/HPKE.** It deserves first-party formal verification posture. Kani at v1-beta; Creusot full-correctness at v1-GM-pre-audit if budget allows.
- A7. **Adversarially-chosen-recipient-seed model for HPKE-mode-base[X-Wing] IND-CCA2** (MF5 first sub-bullet) — consolidator flagged as MED-HIGH gap. Symbolic-proof tools can't address this (it's a computational-model question). The honest answer is: defer to v1-GM audit firm + cite upstream X-Wing paper Barbosa-Connolly-Diniz-Kahl-Krämer; if customer demand emerges, fund a computational-proof contribution post-v1-GM via SandboxAQ / project-Everest collaboration.

---

## §2 Formal-verification target enumeration

Walks the 28 amendments + 3 invariants + 14 Compromises (#32-#44 + #31 extension). For each: is this a formal-verification target? Which tool? Which property? When?

Notation: **FV-target**: Y (formally-verifiable, in-scope for Benten) / Y-upstream (verified by dependency; Benten cites) / N-prose (informal-rigorous prose adequate; computational-model territory) / N-operational (not a cryptographic property).

### §2.1 Amendments (U1-U40)

| # | Title (abbrev) | FV-target? | Tool | Property | When | Effort (wave-days) |
|---|---|---|---|---|---|---|
| **U1** | Codepoint committed in AAD/info | **Y** | kani | `seal(cp, msg, AAD).codepoint == cp ∧ open(seal_output, AAD').codepoint == cp ⇒ AAD == AAD'` (bounded) | **v1-beta** (rolled into U39a) | 0.3 |
| **U2** | Strict-decode; no cross-variant fallback | **Y** | kani | `∀ bytes. decode(bytes).variant == lookup(bytes.codepoint) ∨ decode(bytes) == Err` | **v1-beta** (U39b) | 0.5 |
| **U3** | Canonical TLV length-injectivity | **Y (already U39)** | kani | `∀ a,b: BindingContext. canonical_serialize_tlv(a) == canonical_serialize_tlv(b) ⇒ a == b` (bounded) | **v1-beta (U39 LOAD-BEARING)** | 2.0 |
| **U4** | Sender-DID bound into AAD | **Y** | kani (structural) + proverif/tamarin (adversarial) | **kani:** AAD round-trip + sender-DID-extraction injectivity. **Tamarin:** confused-deputy attack absent under Dolev-Yao. | **v1-beta** kani; **v1-GM** tamarin | kani 0.3 + tamarin 2.0 |
| **U5** | Sealed-at + valid-until in AAD | **Y** | kani | `decode(bytes).valid_until > now() ∨ open(bytes) == Err(Expired)`; monotonicity of `now()` callsites; clock-skew bounds | **v1-beta (U39c)** | 1.0 |
| **U6** | Bernstein-Persichetti Decap CT | **Y-upstream (libcrux hax+F*)** | hax+F* (Cryspen) | secret-independence of Decap under chosen-ciphertext queries | **v1-beta via libcrux pin** + SECURITY-PROOFS.md §1 records | 0.5 (doc) |
| **U7** | BE codepoint on-wire | **Y** | kani | `decode(encode(cp)) == cp` for BE round-trip across `[0..0xFFFF]` | **v1-beta (rolled into U39b)** | 0.2 |
| **U8** | Codepoint registry discipline | **Y** | cite-drift-detector (Benten's existing scanner extended) | every `pub const ..._CODEPOINT: u16` has registry row; no duplicate codepoints | **v1-beta** scanner; not formal-verification per se | 0.5 |
| **U9** | `EnvelopePayload` `#[non_exhaustive]` + typed-reject | **Y** | kani | `∀ unknown_tag. decode(bytes_with_unknown_tag) == Err(UnknownPayloadVariant)`; never panic | **v1-beta (U39b ext)** | 0.3 |
| **U10** | `BindingContext` `#[non_exhaustive]` + typed-reject | **Y** | kani (same as U9) | (sibling of U9) | **v1-beta (U39b ext)** | 0.3 |
| **U11** | Escape codepoint + experimental range | **Y** | kani | `∀ cp ∈ 0xFE00..0xFFFE. decode == Err(Experimental) ∨ Err(UnknownCodepoint)`; `0xFFFF == Err(ExtendedCodepointEscape)` | **v1-beta (U39b ext)** | 0.2 |
| **U12** | Nonce-length-variant discrimination | **Y** | kani | `nonce_len(variant) ∈ {12, 24}`; cross-variant nonce-length-confusion is structural-reject | **v1-beta (U39 ext)** | 0.3 |
| **U13** | FS-gap honest-disclosure + MLS-PQ codepoint reservation | **N-prose for #42** + **Y for codepoint-range non-overlap** | cite-drift-detector | reservation-range scanner | **v1-beta** scanner; FS-gap is computational-model territory deferred to v1-GM audit + future MLS-PQ-verification | 0.2 (scanner) |
| **U14** | `aad_version: u8` prefix bound | **Y** | kani | `decode_aad(bytes).aad_version == bytes[0]`; aad_version-mismatch = Err | **v1-beta (rolled into U39a)** | 0.2 |
| **U15** | `Did` multikey + `Did::Unknown` | **Y** | kani | `decode(encode(did)) == did ∨ encode(did) == varint(unknown_codepoint) ‖ bytes` round-trip injectivity | **v1-beta (rolled into U39 ext)** | 0.3 |
| **U16** | `CodepointLifecycle` state-machine | **Y** | kani | (a) no Live → Burned direct transition; (b) Burned admits no read or write; (c) Quarantined admits read-only; (d) transitions monotonic-non-cyclic | **v1-beta (U39d)** | 0.5 |
| **U17** | Multi-recipient stanzas + cross-stanza substitution defense | **Y** | **tamarin or proverif** | Recipient-isolation: `Recipient_j cannot decrypt stanza_i (i≠j)`; cross-stanza-substitution: `swap(stanza_i, stanza_j) ⇒ auth-tag fails`; per-stanza AAD distinguishes (cp, body-CID, sorted recipient-DID-list, sender_did, stanza-index, recipient_key_generation) | **v1-GM-DEFER (single highest-value first-party FV)** | **5-8 (tamarin/proverif model + proof)** |
| **U18** | Dual-CID (plaintext vs envelope_blob) | **N-prose** | (none) | property is semantic ("forkability survives Drop-CID stability"); formal verification not load-bearing; ADR-class doc adequate | **prose only** | 0 |
| **U19** | `recipient_key_generation: u32` binding | **Y** | kani + tamarin | **kani:** AAD round-trip. **Tamarin:** "envelope sealed to gen-N opens only with sk-of-gen-N" + 6-months-offline scenario (Compromise #31). | **v1-beta** kani; **v1-GM** tamarin (rolled into U17 model) | kani 0.3 + tamarin within U17 |
| **U20** | `k_principal_generation: u32` binding | **Y** | kani | AAD round-trip + Vault HashMap key-presence invariant | **v1-beta (U39 ext)** | 0.5 |
| **U21** | `ExecuteWorkflow` capability | **Y** | **tamarin or proverif** | Capability-binding: `workflow_cid, input_node_cids, max_decrypt_count, result_recipient_pubkey, executor_did` bound s.t. delegated executor cannot exceed scope; max_decrypt_count counter monotonicity | **v1-GM-DEFER** (hyper-scaling not on v1-beta-day-one critical path) | **3-5** |
| **U22** | Sealed-Sender additive codepoint slot | **Y-deferred** (impl deferred to G-CORE-PRIVACY-1) | proverif/tamarin (when implemented) | sender-anonymity property; analogous to Signal Sealed-Sender V2 verification | **v1-GM-DEFER or post-v1-GM** | (depends on impl wave) |
| **U23** | Per-relay-unlinkability (transport blinding) | **Y-deferred** | proverif/tamarin | per-hop transport-ID unlinkability under colluding-adjacent-relays | **post-v1-GM** | 3-5 |
| **U24** | Padding to size-class buckets | **N-prose** (statistical not symbolic) | (none for primary property); kani for "padding length-prefix decode is correct" | bucket-shape-correctness via kani; size-leakage-magnitude is measurement-based | **v1-beta kani for length-decode** + **post-v1-GM measurement empirics** | kani 0.3 |
| **U25** | Per-recipient-unlinkable copies invariant | **Y-deferred (with U17)** | tamarin | indistinguishability of two recipient copies under outsider observer | **v1-GM-DEFER (rolled into U17 model)** | within U17 |
| **U26** | Cover-traffic | **N-deferred** | (none until shaped-relay transport lands) | n/a | **NAMED-DEFERRED** | 0 |
| **U27** | DID-rotation discipline | **N-operational** | (none) | n/a | **NAMED-DEFERRED** | 0 |
| **U28** | Coarse 1-hour epoch buckets | **Y** | kani | `bucket(t) = (t + jitter) / 3600` injectivity-modulo-jitter; replay-window monotonic per bucket | **v1-beta (rolled into U39c)** | 0.2 |
| **U29** | Cross-ecosystem-identifier emit-discipline | **Y** | cite-drift-detector | every cross-ecosystem-emit-site uses `cross_ecosystem().<field>()` | **v1-beta** scanner; not formal-verification | 0.5 (scanner) |
| **U30** | DAG-CBOR outer framing | **Y** | kani | RFC 8949 §4.2.1 deterministic-encoding round-trip + Benten-private CBOR-tag injectivity | **v1-beta if DAG-CBOR LOAD-BEARING** (consolidator §5 Q4 ratify) | 1.0 (depends on if cbor4ii or quick-cbor or local impl) |
| **U31** | libcrux-ml-kem swap | **Y-upstream** | hax+F* (Cryspen) | panic-freedom + correctness + secret-independence | **v1-beta** via crate pin + **SECURITY-PROOFS.md §1 records exact version + property strength + spec version** | 0.5 (doc; impl is L4's ~50 LOC) |
| **U32** | XChaCha20-Poly1305 (24-byte nonce) | **Y-upstream-partial** | (RustCrypto chacha20poly1305 has no formal verification at HEAD; hax+F* coverage exists upstream for ChaCha20 stream primitive but not the full AEAD) | nonce-misuse-resistance is structural-only (24-byte nonce ⇒ collision probability 2^-96 over many messages) | **v1-beta-via-prose** + **post-v1-GM hax verification of chacha20poly1305 if Cryspen adds coverage** | 0.3 (doc; no FV at v1-beta) |
| **U33** | NAPI single-source `canonical_binding()` | **N-operational** (cross-language drift defense; §3.5g enforcement) | drift-detector scanner | every TS canonical_binding callsite is `await napi.canonicalBinding(...)` | **v1-beta** scanner | 0.5 |
| **U34** | Opaque-handle pattern at NAPI boundary | **N-operational** | clippy lint (`disallowed-methods` on returning raw key-bytes from NAPI functions) | static analysis catches; not FV proper | **v1-beta** clippy | 0.2 |
| **U35** | `wasm_js` getrandom cfg at shell crate | **N-operational** | CI test (cargo check --target wasm32 + smoke test in browser harness) | n/a | **v1-beta** CI | 0.3 |
| **U36** | Tokio cancel-safety wrap | **N-operational** | clippy `disallowed-methods` + CI test | n/a | **v1-beta** clippy + test | 0.3 |
| **U37** | Golden-vector corpus | **Y-adjacent** (KAT corpus + property-test-like; not formal verification but adjacent + audit-deliverable) | proptest harness + RFC 9180 KATs | n/a | **v1-beta** | within L4 estimate |
| **U38** | dudect-bencher CI | **Y-statistical (NOT formal)** | dudect | Welch's t-test fail at p<0.01 over 10⁶ samples; statistical-bound-not-proof | **v1-beta** + **SECURITY-PROOFS.md §4 records what dudect's negative result means + does not mean** | within L4 estimate + 0.5 doc |
| **U39** | kani TLV-injectivity proof | **Y (PROMOTE TO LOAD-BEARING)** | kani | (see U3) + extend to U39a-d harness families per §3 | **v1-beta-LOAD-BEARING (per this critique)** | 2.0 base (existing) + 1.5 ext = 3.5 |
| **U40** | THREAT-MODEL.md mint | **N-doc** (not FV but composes with SECURITY-PROOFS.md per §4) | n/a | n/a | **v1-beta** + **SECURITY-PROOFS.md as sibling** | within L5 estimate + 1.5 SECURITY-PROOFS.md mint |

### §2.2 Invariants (Inv-16, Inv-17, Inv-18)

| Inv | FV-target? | Tool | Property | When |
|---|---|---|---|---|
| **Inv-16** | Envelope-layer-unification + codepoint-dispatch + AAD-binding + strict-decode + canonical-TLV + sender-DID + replay | **Y (composite)** | kani family (U39a-d) covers most; tamarin (U17 model) covers the adversarial multi-stanza piece | (a) kani for round-trip + injectivity; (b) cite-drift-detector for "every use-site dispatches through `EncryptedEnvelope`" scanner | **v1-beta** kani + scanner; **v1-GM** tamarin |
| **Inv-17** | Hybrid-cryptography-mandatory floor | **Y (scanner)** | cite-drift-detector flags KEM-codepoint mint lacking PQ + classical halves | (already proposed by consolidator) — formal-verification adds: kani harness "no `decapsulate` callsite without classical-PRF combiner" | **v1-beta** scanner |
| **Inv-18** | Codepoint-registry + metadata-disclosure + CodepointLifecycle | **Y (mixed)** | (a) cite-drift-detector for registry-row presence + lifecycle-state presence; (b) kani for CodepointLifecycle transition-graph (U39d); (c) cite-drift-detector for "every plaintext-sender-DID variant has paired Sealed-Sender slot reserved" | **v1-beta** scanner + kani |

**Sub-finding F-1.** Splitting Inv-18 into Inv-18a/b/c (as consolidator's self-critique #1 considers) would also help formal-verification posture: each sub-invariant has a clean tool-fit (a-scanner, b-prose+disclosure, c-kani). Recommend Ben ratify the split. ~no incremental cost.

### §2.3 Compromises (#32-#44 + #31 extension)

| # | FV-target? | Notes |
|---|---|---|
| **#31 ext** | **N-operational** | Recipient-key-retention window is operational/UX. Tamarin model of U17 + U19 covers the cryptographic implications. |
| **#32 Bernstein-Persichetti Decap CCA** | **Y-upstream via U31 libcrux** | Honest-disclosure + libcrux verification together suffice for v1-beta. v1-GM auditor MAY ask for additional dudect (U38) corroboration on Benten's chosen libcrux version. |
| **#33-#41** | **N-operational or N-out-of-scope** | These are honest-disclosure of OUT-OF-SCOPE adversary classes (coercion, password-knowledge, RAM, TEE, physical, supply-chain). Not formal-verification targets. |
| **#42 Layer-C FS-gap** | **N-prose** (computational-model) | FS is a computational property; symbolic tools cannot capture it. SECURITY-PROOFS.md §3 should explicitly note "FS-gap is OUT-OF-SCOPE for symbolic-protocol verification; CGKA/MLS-PQ codepoint reservation per U13 is the future-path closure." |
| **#43 Envelope metadata leakage** | **Y-deferred** (tamarin/proverif when Sealed-Sender U22 implemented) | Metadata-leakage to honest-but-curious relays is symbolically modelable; defer to G-CORE-PRIVACY-1 wave + post-v1-GM. |
| **#44 BSI long-term-confidentiality** | **N-prose** | Long-term-confidentiality is regulatory + computational; honest-disclosure adequate. |

### §2.4 Cross-cutting targets the panel missed

| Target | FV-target? | Tool | Property | When |
|---|---|---|---|---|
| **CT-1: `canonical_binding()` total-correctness** | **Y** | Creusot (post-v1-GM) or extended kani at v1-beta | Function never panics; always returns Some(bytes); idempotent under aad_version=v1; output-length matches expected formula | v1-beta extended kani; v1-GM Creusot if budget | kani 0.5; Creusot 3-5 |
| **CT-2: Argon2id parameter tier consistency** | **Y (scanner)** | cite-drift-detector | every K_principal-derivation callsite uses the AAD-bound tier; no mid-tier-switch attacks | v1-beta scanner | 0.3 |
| **CT-3: Codepoint-uniqueness invariant** | **Y (scanner + kani)** | cite-drift-detector + kani | no two `pub const ..._CODEPOINT: u16` evaluate to the same u16; static-dispatch table is total function | v1-beta | 0.2 |
| **CT-4: `wasm32` parity** | **Y (CI)** | cargo check + smoke test | every codepoint dispatchable + decode-able on wasm32 | v1-beta CI | within L4 |
| **CT-5: Authenticated-Encryption ROBUSTNESS (key-binding for HPKE)** | **Y-upstream-partial** | RFC 9180 KAT corpus + (if hax+F* extends to McMillion-hpke) compile-time proof | key-committing AEAD property; relevant to L9 multi-recipient pattern; current HPKE-mode-base does NOT promise key-committing — attacker who knows two valid (sk, pk) pairs can construct ambiguous ciphertext | v1-beta-prose-only + v1-GM additional kani harness for ciphertext-uniqueness | 0.5 (prose) + 1-2 (kani v1-GM) |
| **CT-6: Replay-cache state-machine (UCAN nonce-cache integration with U5/U28)** | **Y** | kani | nonce-cache monotonic-add; no nonce-rebind; eviction-policy preserves nbf/exp bounds | v1-beta (U39c ext) or v1-GM | 1.0 |

**Sub-finding F-2.** CT-5 (key-committing AEAD) is a real formal-verification target the panel missed. ChaCha20-Poly1305 + AES-GCM are NOT key-committing by default (Albrecht-Bellare 2024). When Benten's U17 multi-stanza pattern allows the same ciphertext to bind to two distinct recipient-keys (because both sk's decapsulate to the same CEK by chance OR by adversarial construction), the auth-tag does NOT distinguish. This is a published attack class. Mitigation = add a key-commitment field (e.g. HMAC of key over ciphertext, or use AES-GCM-SIV + CMT-1 transform). Defer to v1-GM-pre-audit at minimum; verify-or-disclose.

---

## §3 Tool selection guide

For Benten's stack (pure-Rust workspace; NAPI-RS bridge to TS; existing proptest harness).

### §3.1 kani (Rust bounded model checker)

**Sweet spot for Benten.**
- Round-trip encode/decode injectivity (U1, U3, U7, U9-U12, U14, U15, U20).
- Strict-decode dispatch correctness (U2, U9-U11).
- State-machine reachability (U16 CodepointLifecycle).
- Replay-window monotonicity (U5, U28).
- Capability-counter monotonicity (U21 max_decrypt_count if shipped at v1-beta).

**Limitations.**
- Bounded-model-checker — must constrain input sizes; cannot prove for all `Vec<u8>`. For Benten's TLV format, bounding `BindingContext` to N=8 typed fields × 1024-byte values is adequate for closing the alg-confusion / TLV-collision classes (these are existence-of-collision proofs that don't need unbounded-length).
- No proof composition across modules (each harness independent).
- Loop-unwinding bound required; CT-Decap-loop verification is OUT-OF-SCOPE for kani (use hax+F* upstream instead).
- CBMC-backend solver-time grows superlinearly with bound; harnesses must be tuned.

**Setup cost.** ~0.5 wave-day for first harness (install kani via `cargo install --locked kani-verifier` + `cargo kani setup` + write first `#[kani::proof]`). Subsequent harnesses ~0.3 wave-day each. CI integration: kani offers a GitHub Action; ~0.3 wave-day to wire.

**Maintenance cost.** Each new `BindingContext` variant requires extending the kani harness's variant-enumeration. ~5 min per variant.

**Recommendation: PROMOTE U39 (LOAD-BEARING at v1-beta)** + extend with U39a-d harness family per §1.

**Citation.** [Kani Rust Verifier — model-checking/kani](https://github.com/model-checking/kani); [Kani limitations](https://model-checking.github.io/kani/limitations.html); v0.66 includes `BoundedArbitrary` trait useful for `Vec`-of-bounded-size proofs.

### §3.2 creusot (Rust deductive verification)

**Sweet spot.**
- Total-correctness proofs (panic-freedom + functional-correctness + termination) of pure functions with rich invariants — e.g., `canonical_binding()` total-correctness; `canonical_serialize_tlv()` length-formula correctness.
- Proof of complex iterator/state-machine invariants where kani can only cover bounded shapes.

**Limitations for Benten.**
- High proof-engineering cost — requires writing specifications in PEARLITE (Creusot's spec language). Typical effort: 2-3 weeks of verification engineer time per non-trivial function. Benten doesn't have a verification engineer on payroll.
- Best-fit for v1-GM-pre-audit window if cost justifies; otherwise post-v1-GM strategic.
- POPL 2026 tutorial (Jan 2026) implies upcoming uptick in community.

**Setup cost.** ~2 wave-days first-harness (need to learn PEARLITE + install Why3 + Z3/CVC4/Alt-Ergo SMT-solvers); subsequent ~0.5-1 wave-day each.

**Recommendation: DEFER to post-v1-GM strategic.** Kani covers Benten's v1-beta-LOAD-BEARING surface adequately. If Ben funds a creusot effort post-v1-GM, the highest-value target is `canonical_binding()` total-correctness + `BindingContext::canonical_serialize_tlv()` length-injectivity-as-functional-theorem (stronger than kani's bounded-existence-of-collision).

**Citation.** [Creusot — creusot-rs/creusot](https://github.com/creusot-rs/creusot); [Creusot: a Foundry for the Deductive Verification of Rust Programs (Denis et al., 2022)](https://jhjourdan.mketjh.fr/pdf/denis2022creusot.pdf); [POPL 2026 tutorial](https://popl26.sigplan.org/details/POPL-2026-tutorials/6/Creusot-Formal-verification-of-Rust-programs).

### §3.3 proverif (symbolic cryptographic protocol verifier)

**Sweet spot for Benten.**
- Authentication + secrecy + injective-agreement properties under Dolev-Yao + computational-soundness under CryptoVerif (proverif's sibling).
- Multi-stanza HPKE recipient-isolation (U17).
- Capability-binding for ExecuteWorkflow (U21).
- Sealed-Sender V2 sender-anonymity (U22 when implemented).

**Strengths vs Tamarin.** Faster on simple protocols; better automation; gentler learning curve; better tooling for IETF protocol working drafts.

**Limitations.**
- Unbounded-session reasoning limited (must abstract loops; Tamarin handles better).
- Less well-suited to stateful protocols (Tamarin's multiset rewriting better).
- Cryptographic primitives modeled as black-boxes (cannot model side-channels; that's CryptoVerif territory or computational-proof).

**Setup cost.** ~3-5 wave-days first model (steep learning curve if no prior experience; reasonable for someone with formal-cryptography background). Subsequent ~2-3 wave-days each.

**Recommendation for Benten: TAMARIN OVER PROVERIF** for U17 + U21 because (a) Benten's protocol has stateful elements (recipient-key-generation, k_principal-generation, capability-counter); (b) IETF-PQ-protocol verification efforts (PQXDH, MLS-PQ) precedent in Tamarin; (c) Tamarin's adversary model is more directly aligned with Benten's Atrium-peer-mesh threat surface.

### §3.4 tamarin-prover (symbolic protocol verifier; multiset rewriting)

**Sweet spot for Benten.**
- Multi-stanza HPKE recipient-isolation under adaptive corruption (U17).
- ExecuteWorkflow capability-binding with delegation counter (U21).
- Sealed-Sender V2 sender-anonymity (U22).
- Per-relay-unlinkability under colluding-relays (U23 post-v1-GM).
- (Post-v1-GM) MLS-PQ / CGKA epoch-ratcheting verification when those codepoints get implemented.

**Strengths vs ProVerif.** Better for stateful protocols; better for inductive proofs over unbounded sessions; precedent in IETF PQXDH + Signal-PQ + MLS verification.

**Limitations.**
- Steep learning curve (multiset rewriting + helper-lemma authoring).
- Proofs can require manual oracle-guidance; not fully push-button.
- Cryptographic primitives still black-boxed.

**Setup cost.** ~4-6 wave-days first model (Tamarin requires solver setup + theory authoring + lemma-discovery). Subsequent ~2-3 wave-days each (large fixed-cost amortizes).

**Recommendation: PRIMARY TOOL for U17 (v1-GM) + U21 (v1-GM)**.
- U17 multi-stanza model: ~5-8 wave-days end-to-end (theory + lemma + proof + literate write-up for SECURITY-PROOFS.md §2).
- U21 ExecuteWorkflow: ~3-5 wave-days; can reuse Tamarin theory from U17.

**Citation.** [tamarin-prover/tamarin-prover GitHub](https://github.com/tamarin-prover/tamarin-prover); [Formal Verification of a Post-quantum Signal Protocol with Tamarin (2024)](https://link.springer.com/chapter/10.1007/978-3-031-49737-7_8); [Tamarin Prover Manual](https://tamarin-prover.com/manual/master/book/001_introduction.html); precedent for IETF PQ-protocol verification.

### §3.5 hax+F* (Cryspen's stack for libcrux)

**Sweet spot for Benten.**
- VERIFICATION-VIA-DEPENDENCY: Benten consumes `libcrux-ml-kem` which is hax+F*-verified for panic-freedom + correctness + secret-independence (per [Cryspen blog](https://cryspen.com/post/ml-kem-verification/) + [pq-code-package/mlkem-rust-libcrux](https://github.com/pq-code-package/mlkem-rust-libcrux)).
- Compile-time secret-independence via `check-secret-independence` feature gating on libcrux-secrets types.

**Limitations for first-party use.**
- HIGH effort to formally verify Benten-authored Rust code via hax+F* directly. Would require: rewrite of target function in subset-of-Rust that hax accepts + F* specifications + proof obligations. Typical: 4-8 weeks of verification engineer time per non-trivial function.
- Not realistic at v1-beta or v1-GM-pre-audit for Benten-authored code.
- Post-v1-GM strategic collaboration with Cryspen could extend coverage to Benten's `canonical_binding()` or AEAD wrappers.

**Recommendation for Benten:**
1. **v1-beta:** Consume libcrux via U31; record exact version pin + property strength in SECURITY-PROOFS.md §1 (per A2 above).
2. **v1-GM:** Enable `check-secret-independence` feature in CI; document any libcrux API surface that re-exposes `Secret<T>` types (vs unwrapped raw bytes) — Benten's wrapper code at the FFI boundary should preserve the secret-independence wrapper to avoid breaking the upstream property.
3. **Post-v1-GM strategic:** Engage Cryspen for hax-verification of Benten-authored AEAD wrappers if customer demand emerges. Highest-value targets: `canonical_binding()` (panic-freedom + injectivity); chacha20poly1305 wrapper if Cryspen extends ChaCha20 coverage to full AEAD.

**Citation.** [pq-code-package/rust-libcrux (formally verified for panic freedom, correctness, secret independence in F* using hax)](https://github.com/pq-code-package/rust-libcrux); [Cryspen Verifying Libcrux's ML-KEM blog](https://cryspen.com/post/ml-kem-verification/); [libcrux-ml-kem on lib.rs](https://lib.rs/crates/libcrux-ml-kem).

### §3.6 ct-verif / dudect / ctgrind (constant-time verification)

**Sweet spot for Benten.**
- Statistical CT-validation at integration-test time (dudect via L4's IMPL-B3 / U38).
- Memory-access-pattern CT-verification at binary level via ctgrind (Valgrind-based).
- Formal CT-verification via ct-verif (binary-level LLVM analysis; harder to integrate; mostly research-tool).

**Limitations.**
- dudect is statistical not formal; bounds Type-I error to p<0.01 but does NOT prove CT.
- ctgrind catches secret-dependent-branches and secret-dependent-loads but is platform-binary-coupled.
- ct-verif (Almeida-Barbosa-Barthe-Dupressoir) is academic; production-readiness limited.

**Recommendation:**
1. **v1-beta:** dudect via U38 in nightly CI (per L4 + L5).
2. **v1-beta:** SECURITY-PROOFS.md §4 documents what dudect proves + does not prove + what threshold (p<0.01 at 10⁶ samples) implies + residual uncertainty. Critical for audit-firm-recognized CT-claims.
3. **v1-GM:** Add ctgrind run on libcrux + Benten AEAD wrappers; ~1 wave-day setup + maintenance.
4. **Post-v1-GM:** ct-verif if customer demand emerges for binary-level CT-proof; deprioritize.

**Citation.** [dudect (Reparaz-Balasch-Verbauwhede CHES 2015)](https://eprint.iacr.org/2014/857.pdf); [Albrecht-Bellare key-committing AEAD attacks 2024]; ctgrind originally Adam Langley.

### §3.7 proptest (already in Benten)

Not formal verification (no soundness guarantee) but adjacent. Sweet spot for property tests with rich generators that complement kani's bounded harnesses. Cost: ~free (already in stack).

**Recommendation:** Keep + extend. Specifically:
- Per-stanza HpkeMultiBase substitution property tests (complement to U17 Tamarin model when minted).
- AAD round-trip property tests across all 28 amendment-touched fields.

---

## §4 `docs/SECURITY-PROOFS.md` recommendation

**Yes — mint as audit-deliverable sibling to `docs/THREAT-MODEL.md`.** Composition rules below.

### §4.1 Charter

`SECURITY-PROOFS.md` documents what Benten formally verifies + by which tool + against which spec + at which version pin + to which strength + with which residual uncertainty. External auditors should be able to read this document and (a) verify the formal-verification claims without re-deriving; (b) understand the precise residual cryptographic uncertainty Benten accepts.

### §4.2 Composition with `THREAT-MODEL.md`

| Doc | What it answers |
|---|---|
| **THREAT-MODEL.md** | What adversary classes does Benten address (IN-SCOPE / OUT-OF-SCOPE / PARTIAL)? What Compromise rows + Invariants address each? |
| **SECURITY-PROOFS.md** | For each IN-SCOPE or PARTIAL adversary class, what formal verification underwrites the claim? Which tool + version + spec + property strength? What's NOT formally verified (and why prose-rigorous is adequate)? |
| **SECURITY-POSTURE.md** | What honest-disclosure Compromises does Benten mint? (existing) |
| **INVARIANT-COVERAGE.md** | What invariants does Benten enforce + how? (existing) |

Cross-references: THREAT-MODEL.md §2.1 row "T-XX" → SECURITY-PROOFS.md §N "FV-target FV-N" → INVARIANT-COVERAGE.md "Inv-NN" → SECURITY-POSTURE.md "Compromise #NN".

### §4.3 Section structure

```
SECURITY-PROOFS.md
§1 Dependency-verified properties (libcrux-ml-kem ml-kem core; chacha20poly1305 partial; etc.)
   - per-dependency: version pin, verification tool, property proved, spec version, residual uncertainty
§2 First-party symbolic-protocol proofs (U17 multi-stanza tamarin; U21 capability tamarin; ...)
   - per-model: tool, theory file path, lemmas proved, abstraction-gap, replay instructions
§3 First-party mechanical proofs (kani U39 family: U39a TLV; U39b dispatch; U39c replay; U39d lifecycle)
   - per-harness: kani version, bound, solver-time, replay command
§4 Statistical / empirical verification (dudect U38; KAT corpus U37; golden vectors)
   - per-test: methodology, threshold, sample size, replay command, what it proves + does not prove
§5 Honest-disclosure: what's NOT formally verified (Compromise # cross-references)
   - per-Compromise: why prose-rigorous adequate; what would need to change to upgrade to formal proof
§6 v1-GM audit-firm reference index (cross-reference to THREAT-MODEL.md adversary classes)
§7 Post-v1-GM strategic roadmap (creusot; hax integration; tamarin contributions upstream)
```

### §4.4 Format recommendation

**Format A (PREFERRED):** Literate Markdown with three cross-cutting cite forms:
- `[libcrux-ml-kem v0.X.Y]` for dependency pins
- `[tamarin/u17-multi-stanza.spthy @ Benten-internal-path]` for first-party proof scripts
- `[kani harness `bind_inj` in `crates/benten-crypto-suite/src/aead.rs` line N]` for mechanical proof references

**Format B (companion):** Proof scripts committed in-tree at `crates/benten-crypto-suite/proofs/` (kani harnesses inside `#[cfg(kani)]`; Tamarin theory files under `.tamarin/`).

**Format C (NOT recommended at v1-beta):** Standalone proof-PDFs (Coq/F*/Isabelle) — too heavy for v1-beta; reserve for post-v1-GM strategic if hax/creusot adopted.

### §4.5 What proofs SHOULD exist pre-v1-beta-tag

| FV-target | Tool | Status pre-v1-beta |
|---|---|---|
| Dependency-verified U31 libcrux-ml-kem | hax+F* (Cryspen) | **CITED** in §1 with version pin + property strength |
| First-party U39a TLV-injectivity | kani | **MECHANICAL PROOF in-tree** |
| First-party U39b strict-decode dispatch + codepoint endianness | kani | **MECHANICAL PROOF in-tree** |
| First-party U39c replay-window monotonicity + clock-skew bounds | kani | **MECHANICAL PROOF in-tree** |
| First-party U39d CodepointLifecycle state-machine | kani | **MECHANICAL PROOF in-tree** |
| Empirical dudect CT-validation (U38) | dudect | **STATISTICAL TEST in nightly CI + §4 record** |
| Empirical KAT corpus (U37) | proptest + manual KATs | **TEST CORPUS in-tree + §4 record** |
| Honest-disclosure: Compromise #32 (B-P Decap CCA) | (no first-party proof) | **§5 honest-disclosure: closed mechanically via U31; v1-GM auditor may re-validate** |
| Honest-disclosure: Compromise #42 (Layer-C FS-gap) | (no first-party proof) | **§5 honest-disclosure: out-of-scope for symbolic; CGKA reservation per U13** |
| Honest-disclosure: Compromise #43 (envelope metadata leakage) | (no first-party proof at v1-beta) | **§5 honest-disclosure: tamarin model deferred to G-CORE-PRIVACY-1 wave** |

**Mint effort:** ~2-3 wave-days for skeleton + populate first version. Subsequent updates ~0.3-0.5 wave-day per amendment touched.

### §4.6 What proofs SHOULD exist pre-v1-GM-audit

Add to §2 (symbolic-protocol proofs):
- U17 multi-stanza HPKE recipient-isolation + cross-stanza-substitution defense → Tamarin model.
- U21 ExecuteWorkflow capability-binding → Tamarin model.
- (Optional if budget allows) Sealed-Sender V2 sender-anonymity → Tamarin model (precedent: Signal Sealed Sender V2 verification by ETH-Zurich group).

Add to §3 (mechanical proofs):
- Extend kani family with U20 K_principal-generation tracking AAD round-trip.
- Extend kani family with CT-1 `canonical_binding()` total-correctness via extended bounded-shapes coverage.

Add to §4 (statistical/empirical):
- ctgrind run on Benten's AEAD wrappers + libcrux integration.

**Mint effort pre-v1-GM-audit:** ~8-13 wave-days incremental.

### §4.7 What proofs are post-v1-GM strategic

- CGKA/MLS-PQ symbolic verification (when minted per U13 codepoint brackets).
- Creusot `canonical_binding()` total-correctness (vs kani's bounded).
- hax+F* integration of Benten-authored AEAD wrappers (via Cryspen collaboration).
- Tamarin model contribution upstream to MLS-WG / CFRG.
- ct-verif binary-level CT-proof of full crypto-suite (if customer demand).

---

## §5 Formal-methods cost estimate per target + total

### §5.1 v1-beta-LOAD-BEARING (incremental over L4's ~35-45-wave-day baseline)

| Item | Tool | Effort (wave-days) |
|---|---|---|
| U39a kani TLV-injectivity (existing IMPL-B4; PROMOTE to LOAD-BEARING) | kani | 2.0 |
| U39b kani strict-decode + codepoint round-trip (NEW) | kani | 1.5 |
| U39c kani replay-window monotonicity (NEW) | kani | 1.0 |
| U39d kani CodepointLifecycle state-machine (NEW) | kani | 0.5 |
| CT-3 kani codepoint-uniqueness invariant (NEW) | kani | 0.2 |
| `SECURITY-PROOFS.md` mint (skeleton + populate) | doc | 1.5 |
| `SECURITY-PROOFS.md` §1 libcrux version-pin + property records | doc | 0.3 |
| `SECURITY-PROOFS.md` §4 dudect what-it-proves narrative | doc | 0.3 |
| kani CI workflow integration | infra | 0.3 |
| **Subtotal v1-beta-LOAD-BEARING incremental** | | **~7.6 wave-days** |
| **+ 25% buffer (formal-verification estimates run higher than impl estimates)** | | **~9.5 wave-days** |

This is on top of L4's estimate (~35-45 wave-days; consolidator-buffered to ~65-72). New total: **~75-82 wave-days = ~15-16 calendar-weeks**. Slightly exceeds the 15-week ceiling per consolidator §6 cost estimate — recommend Ben evaluate trade-off (the ~10 wave-days buys substantial audit-readiness uplift). If budget pressure: defer U39d + CT-3 (~0.7 wave-days) and keep U39a-c + SECURITY-PROOFS.md mint as load-bearing.

### §5.2 v1-GM-DEFER (add to v1-beta baseline pre-audit-window)

| Item | Tool | Effort (wave-days) |
|---|---|---|
| U17 multi-stanza HPKE Tamarin model + proof + literate writeup | Tamarin | 5-8 |
| U21 ExecuteWorkflow capability Tamarin model | Tamarin | 3-5 |
| ctgrind run on libcrux + Benten AEAD wrappers | ctgrind | 1.5 |
| Extended kani CT-1 `canonical_binding()` total-correctness | kani | 0.5-1 |
| SECURITY-PROOFS.md §2 expansion with tamarin theories + replay-instructions | doc | 1.0 |
| **Subtotal v1-GM-DEFER add** | | **~11-16.5 wave-days** |
| **+ 30% buffer (tamarin learning curve)** | | **~14.5-21.5 wave-days** |

This lands in the v1-GM-pre-audit-window (~3 person-weeks per brief = ~15 wave-days budget). Subset choice: U17 model is the HIGHEST-LEVERAGE single target. If only one Tamarin model fits v1-GM-window, **prioritize U17** (multi-stanza recipient-isolation is the central Atrium-composition cryptographic assurance).

### §5.3 Post-v1-GM strategic

| Item | Tool | Effort (wave-days) |
|---|---|---|
| CGKA/MLS-PQ symbolic verification (per U13 reservation) | Tamarin | 8-15 |
| Creusot `canonical_binding()` total-correctness | Creusot | 3-5 |
| hax+F* integration of Benten AEAD wrappers (Cryspen collab) | hax+F* | 15-30 (likely Cryspen-led with Benten review) |
| ct-verif binary-level CT-proof | ct-verif | 5-10 |
| Tamarin model upstream contribution to MLS-WG/CFRG | doc + advocacy | 3-5 |
| **Subtotal post-v1-GM strategic** | | **~34-65 wave-days** |

These are NOT load-bearing on v1-beta or v1-GM tag. They are strategic deposits that compound Benten's audit-firm-recognized cryptographic posture.

### §5.4 v1-beta-LOAD-BEARING vs v1-GM-DEFER subset (RECOMMENDATION TABLE)

| Target | v1-beta | v1-GM-pre-audit | Post-v1-GM strategic |
|---|---|---|---|
| U39a TLV-injectivity kani | **LOAD-BEARING** | — | — |
| U39b dispatch + endianness kani | **LOAD-BEARING** | — | — |
| U39c replay-window kani | **LOAD-BEARING** | — | — |
| U39d CodepointLifecycle kani | LOAD-BEARING (low-cost) | — | — |
| U17 multi-stanza Tamarin | — | **LOAD-BEARING** (highest leverage) | — |
| U21 ExecuteWorkflow Tamarin | — | LOAD-BEARING if capability ships at v1-beta; else DEFER | — |
| U22 Sealed-Sender Tamarin | — | DEFER to G-CORE-PRIVACY-1 wave | DEFER until impl lands |
| U31 libcrux cite + SECURITY-PROOFS.md §1 | **LOAD-BEARING** | strengthen with auditor's choice of version | — |
| U38 dudect + §4 narrative | LOAD-BEARING (per L4/L5) | ctgrind add | ct-verif if customer demand |
| CT-1 `canonical_binding()` total-correctness | partial via kani (bounded) | extended kani | Creusot full-correctness |
| CT-5 key-committing AEAD (NEW) | DISCLOSURE-only | LOAD-BEARING (verify-or-disclose) | upstream contribution |
| SECURITY-PROOFS.md mint | **LOAD-BEARING** (skeleton + populate v1-beta) | §2 tamarin expansion | §7 strategic roadmap |

---

## §6 v1-beta-LOAD-BEARING vs v1-GM-DEFER subset for formal-verification

Restating §5.4 as a clean ratification table for Ben.

### §6.1 ADD to v1-beta-LOAD-BEARING

1. **PROMOTE U39 from RECOMMENDED to LOAD-BEARING** (the existing kani TLV-injectivity proof IMPL-B4).
2. **MINT U39b** — kani strict-decode + codepoint-endianness round-trip.
3. **MINT U39c** — kani replay-window monotonicity + clock-skew bounds.
4. **MINT U39d** — kani CodepointLifecycle state-machine.
5. **MINT CT-3** — kani codepoint-uniqueness invariant (closes the "two distinct constants compiled to the same u16" class).
6. **MINT `docs/SECURITY-PROOFS.md`** with sections §1-§5 populated at v1-beta-tag.

**Cost:** ~7.6 wave-days incremental + 25% buffer = ~9.5 wave-days.

### §6.2 DEFER to v1-GM-pre-audit-window

7. **MINT U17 Tamarin model** (multi-stanza HPKE recipient-isolation + cross-stanza substitution defense).
8. **CONDITIONAL: MINT U21 Tamarin model** if ExecuteWorkflow ships at v1-beta or G-CORE-PRIVACY-1; else defer.
9. **ADD ctgrind to dudect CT-validation** in CI.
10. **EXTEND U39 kani family** with K_principal-generation + extended canonical_binding bounded coverage (CT-1 partial).
11. **EXPAND SECURITY-PROOFS.md §2** with tamarin theories + replay instructions.
12. **NEW: MINT CT-5 key-committing AEAD analysis** — verify-or-disclose; if not verifiable in v1-GM window, add to Compromise #43-adjacent honest-disclosure.

**Cost:** ~11-16.5 wave-days + 30% buffer = ~14.5-21.5 wave-days. **Fits 3-person-week v1-GM-audit-window budget** if Tamarin work is contracted to a verification engineer (Cryspen / SandboxAQ / academic collaboration).

### §6.3 NAMED-DEFERRED to post-v1-GM strategic

13. Creusot `canonical_binding()` total-correctness.
14. hax+F* integration of Benten-authored AEAD wrappers via Cryspen collaboration.
15. CGKA/MLS-PQ Tamarin verification when those codepoints get implemented (per U13 reservation).
16. ct-verif binary-level CT-proof of full crypto-suite.
17. Tamarin model contribution upstream to MLS-WG / CFRG.
18. Sealed-Sender V2 Tamarin model when U22 implementation lands.
19. Per-relay-unlinkability Tamarin model when U23 transport-blinding lands.

---

## §7 Coverage gap analysis vs the 9-eyes panel

### §7.1 What the 9-eyes panel did vs formal-methods discipline

| Lens | Formal-methods-relevant contribution | FV-lens commentary |
|---|---|---|
| **L1 (1st cryptographer)** | §6.2 envelope-layer-unification framing | Concentrated on construction-soundness; no formal-verification posture. |
| **L2 (2nd cryptographer)** | §2.2 IND-CCA2 reduction argument (informal) | INFORMAL PROOF SKETCH; should be either (a) formalized in CryptoVerif post-v1-GM or (b) explicitly cited as informal-prose-cryptographer-grade-rigor in SECURITY-PROOFS.md §5 with reference to upstream X-Wing paper. **Gap: not formalized.** |
| **L3 (adversarial design)** | Am3 (TLV length-injectivity) cites Bellare-Rogaway CRYPTO 1996 | Cites a published informal proof argument; the Benten-specific length-injectivity is a NEW property that should be formally verified via kani (U39 / U39a). **Gap: cited proof is for a different protocol's TLV; Benten-specific verification missing.** Closed by U39a load-bearing promotion. |
| **L4 (impl-engineering)** | IMPL-A1 libcrux swap; IMPL-B4 kani TLV-injectivity | Touches formal-verification but doesn't articulate posture. **Gap: U31 doesn't record what libcrux proves; U39 scoped too narrow.** Closed by §1 SECURITY-PROOFS.md §1 + U39 family extension. |
| **L5 (threat-model + audit-readiness)** | THREAT-MODEL.md mint; compliance mapping (NIST/FIPS/ANSSI/BSI/FedRAMP) | Standards-conformance != formal verification. THREAT-MODEL.md is the right audit-deliverable but its sibling SECURITY-PROOFS.md was not proposed. **Gap: closed by §4 SECURITY-PROOFS.md mint.** |
| **L6 (privacy/metadata)** | Sealed-Sender slot reservation U22; per-recipient-unlinkability U25 invariant | Designed PRIVACY properties without proposing formal verification of those properties. Tamarin model of U17 + U22 closes this (deferred to G-CORE-PRIVACY-1 wave). **Gap: closed in v1-GM via §6.2 #7 + post-v1-GM #18.** |
| **L7 (cross-ecosystem)** | §3.5s emit-discipline; multicodec table | Scanner-discipline (cite-drift-detector) — adjacent to formal verification. **No gap.** |
| **L8 (wire-format stability)** | `#[non_exhaustive]` + escape codepoint + CodepointLifecycle | Designed a state-machine (CodepointLifecycle U16) without proposing formal verification. **Gap: closed by U39d kani harness (~0.5 wave-day).** |
| **L9 (atrium-integration)** | Multi-stanza per-stanza AAD substitution defense; dual-CID | DESIGNED the cross-stanza-substitution defense (U17) without symbolic-protocol verification. The cryptographically central property of multi-recipient Drop bundles. **Gap: closed by U17 Tamarin model at v1-GM-pre-audit; the SINGLE HIGHEST-LEVERAGE first-party FV investment for Benten.** |
| **Consolidator MF5** | Self-critique acknowledging no-FV-lens | Honest disclosure; this critique fills the gap. |

### §7.2 Specific gaps the panel cited but didn't close

- **L2 §2.2 IND-CCA2 reduction is informal.** Adversarially-chosen-recipient-seed model (MF5 first sub-bullet) is computational not symbolic; tamarin/proverif can't address. Honest answer: cite X-Wing paper (Barbosa-Connolly-Diniz-Kahl-Krämer IACR CIC 2024) + defer to v1-GM-audit-firm.
- **L3 §2.7 multi-stanza substitution residual concern.** Defense designed by L9 in U17 A1; not symbolically verified. Tamarin model at v1-GM closes.
- **L9 §3.1 multi-stanza flat composition.** Recipient-isolation depends on per-stanza AAD distinguishing 6-tuple; symbolically modelable + verifiable.
- **L8 Am16 CodepointLifecycle.** State-machine; kani trivially verifies. Not surfaced as FV target by L8.
- **L8 Am11 escape-codepoint reservation correctness.** Kani trivially verifies "0xFFFF reads as escape-sentinel; 0xFE00..0xFFFE read as experimental; other codepoints read as defined-or-error". Not surfaced.

### §7.3 Gaps NEITHER the panel NOR consolidator surfaced (NEW from FV lens)

- **NEW-F1: Key-committing AEAD property** (§2.4 CT-5). Albrecht-Bellare 2024 published attacks against ChaCha20-Poly1305 + AES-GCM in multi-key settings. When U17 multi-stanza pattern allows the same ciphertext to bind to two distinct recipient-keys, the auth-tag does NOT distinguish. Mitigation = add a key-commitment field. **Verify-or-disclose at v1-GM minimum.**
- **NEW-F2: `canonical_binding()` is Benten's most-load-bearing internal cryptographic primitive AFTER the underlying AEAD/HPKE.** It deserves first-party formal-verification posture (not just bounded kani TLV-injectivity; total-correctness via Creusot post-v1-GM if budget; bounded kani at v1-beta).
- **NEW-F3: Argon2id parameter tier AAD-binding consistency** (§2.4 CT-2). Mid-tier-switch attack: adversary holding vault.cbor crafts a vault with different Argon2id tier in AAD-binding vs actual derived K_principal. Closed by cite-drift-detector scanner ensuring tier in AAD matches tier in vault metadata + AAD-binding-discipline. ~0.3 wave-day.
- **NEW-F4: UCAN nonce-cache state-machine integration with U5/U28** (§2.4 CT-6). Replay-cache monotonic-add + eviction policy + nbf/exp interaction. Kani-verifiable. Reachable at v1-beta-LOAD-BEARING if UCAN nonce-cache is in v1-beta surface.
- **NEW-F5: HPKE-mode-base context-string commitment.** RFC 9180 mandates specific labeling pattern (`HPKE-v1` + suite_id); Benten's `info` parameter MUST follow + be auditable. Kani-verifiable via round-trip + label-byte-string-equality.
- **NEW-F6: No proof obligations document for U31 libcrux swap.** What residual cryptographic uncertainty does swapping to libcrux INTRODUCE? E.g., libcrux's API surface might re-expose secret-bytes outside the `Secret<T>` wrapper at some FFI boundary. Document in SECURITY-PROOFS.md §1 + audit at v1-GM.

---

## §8 Refactoring recommendations to make verification easier

### §8.1 At the v1-beta-LOAD-BEARING coding level

1. **Pure-function `canonical_binding()`** — no `Result`, no I/O, no clock-reads inside. Take `now: SystemTime` as parameter, not call `SystemTime::now()` internally. **Why:** kani harnesses can pin `now` to a symbolic value. Currently L4 doesn't specify; refactor at design time, cheap.
2. **Single static dispatch table for codepoint → variant tag.** Implement as `const CODEPOINT_TABLE: &[(u16, &'static str)]` not `match` cascade. **Why:** kani proves table-completeness via array-iteration; `match` cascade is less analyzable.
3. **`BindingContext` fields all `pub(crate)` not `pub`.** **Why:** kani harnesses live in same crate; private fields don't escape the proof's API surface. Reduces external proof obligations.
4. **No closures or trait-object dispatch in `canonical_binding()` body.** **Why:** kani's CBMC backend struggles with higher-order; static-dispatch only.
5. **`Bytes` (or `Vec<u8>`) not `&[u8]` borrowed slices at canonical_binding entry.** **Why:** kani's bounded-Vec proofs are cleaner than bounded-slice proofs.
6. **`#[derive(kani::Arbitrary)]` on every `BindingContext` variant** when behind `#[cfg(kani)]`. **Why:** lets the harness symbolically enumerate. Zero production cost.
7. **Add `#[cfg_attr(kani, kani::proof)]` next to `#[test]`** for harnesses that are property-equivalent. **Why:** keep proofs near tests for discoverability.

### §8.2 At the v1-GM-pre-audit-window level

8. **Pull `EnvelopePayload::HpkeMultiBase` into its own module with documented invariants.** **Why:** Tamarin model targets this surface; clean module-boundary makes the abstraction-gap explicit.
9. **Document AAD canonicalization as a Tamarin theory `aad_canonicalization.spthy`** that other Tamarin models reuse. **Why:** amortizes the Tamarin learning curve across U17 + U21 + future U22 models.
10. **Add `#[cfg(test)] use libcrux_secrets::Secret;` wrapper preservation tests.** **Why:** ensures Benten doesn't accidentally unwrap `Secret<T>` at FFI boundary, breaking libcrux's secret-independence property upstream.

### §8.3 At post-v1-GM strategic level

11. **Add `#[hax::contract]` annotations on critical Benten functions** for future Cryspen collaboration. **Why:** lets hax-verification of Benten code amortize on existing hax-verified libcrux integration.
12. **Adopt no-`unsafe` policy in `benten-crypto-suite`.** **Why:** kani+Creusot both have limited `unsafe` support; staying safe-Rust enables full mechanical verification.
13. **Anti-pattern alert: NO `Box<dyn Trait>` or trait-object dispatch in crypto-suite hot paths.** **Why:** verifier-hostile.

### §8.4 Formal-verification anti-patterns to AVOID in Benten

- **DON'T** use macros that generate `match` cascades over codepoints — verifier-hostile + cite-drift-detector-hostile.
- **DON'T** mix sync + async in `canonical_binding()` call chain — kani doesn't analyze async.
- **DON'T** call `SystemTime::now()` inside `canonical_binding()` (per §8.1 #1).
- **DON'T** use `unsafe` for performance in crypto-suite (it foreclosures Creusot post-v1-GM).
- **DON'T** silently coerce `&[u8]` between codepoints — strict-decode discipline (U2) is verifier-friendly only if the type system enforces.

### §8.5 Collaboration opportunities

- **Cryspen.** Already verifies libcrux-ml-kem (Benten consumes via U31). Outreach for: (a) hax-verification of Benten-authored AEAD wrappers post-v1-GM; (b) review of SECURITY-PROOFS.md §1 dependency-verified-properties narrative; (c) ChaCha20-Poly1305 full-AEAD hax-verification (Cryspen has ChaCha20-stream coverage but not full AEAD at HEAD per public roadmap).
- **SandboxAQ.** Tamarin-prover expertise; potential v1-GM-pre-audit-window contract for U17 + U21 Tamarin models.
- **Project-Everest (MSR / INRIA / CMU).** F* + HACL* + miTLS upstream. Long-term strategic for post-v1-GM hax/F* integration.
- **ETH-Zurich Information Security Group (Tamarin core developers).** Academic-collaboration path; Signal-PQ + MLS-PQ Tamarin precedent.
- **CFRG / IETF.** Contribute Tamarin model upstream post-v1-GM as Benten's strategic public-goods deposit.

---

## §9 Self-assessment + confidence

### §9.1 What I did

1. Tree-state pre-flight on agent worktree branch (clean; @ origin/main).
2. Extracted consolidated registry @ `fbdfeb16` to scratch file via `git show`; read in full (939 LOC).
3. Identified formal-verification targets across 28 amendments + 3 invariants + 14 Compromises.
4. Web-searched libcrux/hax+F* + kani + Tamarin + Creusot current-state (5 searches; cited).
5. Built per-target table (§2.1, §2.2, §2.3) + cross-cutting NEW targets (§2.4).
6. Wrote per-tool selection guide (§3) with sweet-spot + limitations + setup cost.
7. Designed `docs/SECURITY-PROOFS.md` structure + composition rules with `THREAT-MODEL.md` (§4).
8. Estimated effort per target + total (§5).
9. Recommended v1-beta-LOAD-BEARING / v1-GM-DEFER / post-v1-GM strategic subset (§6).
10. Analyzed coverage gaps vs 9-eyes panel; surfaced 6 NEW formal-verification targets the panel missed (§7).
11. Wrote refactoring recommendations to make verification easier (§8).

### §9.2 Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §2 Target enumeration | **HIGH** on which targets are FV-tractable; **MED-HIGH** on tool-fit per target | Per-target tool selection is well-grounded in tool literature; effort estimates ±25%. |
| §3 Tool selection guide | **HIGH** for kani / Tamarin (well-documented sweet-spots); **MED-HIGH** for Creusot (less mature for Benten's use cases); **HIGH** for hax+F*-via-libcrux (well-documented upstream) | Setup costs from cited tool docs + Tamarin precedent. |
| §4 SECURITY-PROOFS.md mint | **HIGH** on direction; **MED-HIGH** on specific format (Markdown vs in-tree proof scripts) | Format-A literate Markdown is best-practice for audit-deliverable; reasonable to disagree on the exact §1-§7 section boundary. |
| §5 Cost estimate | **MED-HIGH** for kani targets (well-bounded); **MED** for Tamarin (high learning-curve variance); **MED-LOW** for post-v1-GM hax/Creusot (hard to estimate without verification engineer scoping) | Estimates roughly ±25-50% per tool; total v1-beta ~9.5 wave-days is solid; v1-GM ~14-22 wave-days has more variance. |
| §6 subset recommendation | **HIGH** on v1-beta LOAD-BEARING set (kani family + SECURITY-PROOFS.md); **MED-HIGH** on v1-GM Tamarin prioritization (U17 > U21 ranking is defensible) | Reasonable cryptographers may disagree on U39d (low-cost minor) and CT-5 (key-committing) priority. |
| §7 Coverage gap analysis | **HIGH** on enumerated gaps; **MED-HIGH** on NEW-F1..NEW-F6 (especially NEW-F1 key-committing AEAD — this is current published research that the panel missed) | NEW-F1 deserves Ben's attention; NEW-F6 is bookkeeping. |
| §8 Refactoring recommendations | **HIGH** on coding-level (verifier-friendly patterns are well-documented); **MED-HIGH** on collaboration opportunities | §8.5 collaboration opportunities are strategic-suggestion not load-bearing-mandate. |

### §9.3 What I could be wrong about

1. **Tamarin effort estimate for U17 (~5-8 wave-days).** Real-world IETF-protocol-verification efforts span 2-6 person-weeks for first-time Tamarin authors; Benten's HpkeMultiBase is simpler than MLS but with stateful elements (recipient-key-generation, k_principal-generation). Estimate could be ±50%. RISK: if no in-house Tamarin expertise, contract to academic / Cryspen / SandboxAQ; cost+time-on-clock both expand.
2. **U39 (kani TLV-injectivity) tractability at v1-beta.** Existing L4 IMPL-B4 estimated "~2 wave-days" without empirical kani-bound testing. If `BindingContext`'s variant-cross-product times bounded-field-sizes exceeds CBMC's solver-time budget at ~30 min, harness needs splitting + smaller bounds + multiple smaller proofs. RISK: kani harness time-out forces 2-3x cost. Mitigation: dispatch a spike at R0 to test kani-bound tractability before committing U39 as LOAD-BEARING.
3. **SECURITY-PROOFS.md effort (~1.5 wave-days).** Skeleton is cheap; populated v1-beta version may take 2-3 wave-days depending on how thorough §5 honest-disclosure section is.
4. **NEW-F1 (key-committing AEAD).** This is a real published-research gap; I'm HIGH on its existence but MED on its immediate-exploit reachability for Benten's specific U17 multi-stanza shape. A targeted Albrecht-Bellare-style analysis would clarify; could be 1-3 wave-days at v1-GM-pre-audit.
5. **The libcrux version-pin SECURITY-PROOFS.md §1 narrative.** I haven't checked whether libcrux-ml-kem's release notes specify which exact theorems are proved + at which version + against which spec; if Cryspen's release-process under-specifies, Benten's SECURITY-PROOFS.md §1 might need to do meta-research to reconstruct the property strength. Could be 0.3-1 wave-day.

### §9.4 Lower-confidence areas (honest disclosure)

- I did NOT empirically run kani on Benten's `aead.rs` to verify the TLV-injectivity-bound tractability. Estimate propagated from L4 IMPL-B4.
- I did NOT survey academic Tamarin/ProVerif literature for HPKE-multi-stanza-specific verification precedent (only IETF PQXDH + Signal-PQ + MLS-PQ broadly). The U17 model may benefit from existing MLS recipient-isolation theory; would-be useful at R0 review.
- I did NOT verify Cryspen's hax+F* coverage map against Benten's exact dependency list (chacha20poly1305 in particular); I propagated the public-roadmap-as-of-May-2026 assumption.
- Cost estimates for post-v1-GM strategic items (§5.3) are rough; would need verification-engineer scoping for accuracy.

### §9.5 What this critique does NOT cover

- Did NOT re-derive the 9-eyes panel's amendments / Compromises / invariants. Consolidator is authoritative on those.
- Did NOT propose new wire-format amendments; formal-methods lens is verification-of-existing-design not new-design.
- Did NOT critique the consolidator's §5 disagreement-matrix decisions; orthogonal to FV posture.
- Did NOT cost the operational impact of slipping from L4's ~35-45-wave-day baseline to ~75-82 with FV add-on; that's a Ben-level scope-vs-budget trade-off.

---

## §10 Citations

### §10.1 Inputs (frozen SHAs)

- Consolidated registry: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` (939 LOC).
- 9-lens reviews per consolidator §10.1 (L1 @ 6d4e173f .. L9 @ 1670aa03; e2r @ 220b5aae).

### §10.2 Tool documentation + citations (web-fetched 2026-05-27)

- **libcrux / hax+F***: [pq-code-package/rust-libcrux (formally verified for panic freedom, correctness, secret independence in F* using hax)](https://github.com/pq-code-package/rust-libcrux); [Cryspen Verifying Libcrux's ML-KEM blog](https://cryspen.com/post/ml-kem-verification/); [libcrux-ml-kem on lib.rs](https://lib.rs/crates/libcrux-ml-kem); [libcrux-ml-kem on crates.io](https://crates.io/crates/libcrux-ml-kem); [cryspen/libcrux](https://github.com/cryspen/libcrux); [mlkem-rust-libcrux MAINTAINERS](https://github.com/pq-code-package/mlkem-rust-libcrux/blob/main/MAINTAINERS.md).
- **kani**: [Kani Rust Verifier — model-checking/kani](https://github.com/model-checking/kani); [Kani limitations](https://model-checking.github.io/kani/limitations.html); [Kani usage guide](https://model-checking.github.io/kani/usage.html); [Kani releases](https://github.com/model-checking/kani/releases); [Kani blog](https://model-checking.github.io/kani-verifier-blog/); [Kani for verify-rust-std](https://model-checking.github.io/verify-rust-std/tools/kani.html).
- **Creusot**: [Creusot — creusot-rs/creusot](https://github.com/creusot-rs/creusot); [Creusot homepage](https://creusot.rs/); [Creusot: a Foundry for the Deductive Verification of Rust Programs (Denis et al., 2022)](https://jhjourdan.mketjh.fr/pdf/denis2022creusot.pdf); [Creusot tutorial at POPL 2026](https://popl26.sigplan.org/details/POPL-2026-tutorials/6/Creusot-Formal-verification-of-Rust-programs); [Creusot releases](https://github.com/creusot-rs/creusot/releases).
- **Tamarin**: [tamarin-prover/tamarin-prover GitHub](https://github.com/tamarin-prover/tamarin-prover); [Tamarin Prover homepage](https://tamarin-prover.com/); [Tamarin Prover Manual](https://tamarin-prover.com/manual/master/book/001_introduction.html); [Tamarin at ETH-Zurich Information Security Group](https://infsec.ethz.ch/research/projects/tamarin.html); [Formal Verification of a Post-quantum Signal Protocol with Tamarin (2024)](https://link.springer.com/chapter/10.1007/978-3-031-49737-7_8); [Modeling and Analyzing Security Protocols with Tamarin: A Comprehensive Guide (Springer 2025)](https://link.springer.com/book/10.1007/978-3-031-90936-8).
- **ProVerif**: (referenced via Tamarin / Signal-protocol-verification literature; standard tool).

### §10.3 Cryptographic-research citations

- Albrecht-Bellare 2024 (key-committing AEAD attacks) — referenced as NEW-F1 grounding.
- Bernstein-Persichetti "One Time is Enough" IACR 2024/2051 — referenced via Compromise #32 / U6 / U31.
- Barbosa-Connolly-Diniz-Kahl-Krämer X-Wing IACR CIC 2024 — referenced via U6 + L2 §2.2 IND-CCA2.
- Reparaz-Balasch-Verbauwhede "Dude, is my code constant time?" CHES 2015 — dudect originating paper for U38.
- Bellare-Rogaway CRYPTO 1996 — referenced via L3 Am3 / U3 informal TLV-length-injectivity argument.
- Almeida-Barbosa-Barthe-Dupressoir et al. — ct-verif binary-level CT-verification research grounding.

### §10.4 Benten internal references

- `docs/INVARIANT-COVERAGE.md` (Inv-15 enforced; Inv-16/17/18 pending per consolidator).
- `docs/SECURITY-POSTURE.md` Compromise # registry.
- `docs/V1-FROZEN-INTERFACE.md` + V1-FROZEN-INTERFACE-DEFERRED.md.
- CLAUDE.md baked-in #5 (crypto-agility codepoint-dispatch); #15 (v1-beta + v1-GM gates); #17 (deployment-shapes); #18 (authority-isolation vs confidentiality-isolation).
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` (load-bearing for §8 anti-pattern recommendations).
- `feedback_review_finding_ground_truth_verify` (load-bearing for §9.3 honest-uncertainty disclosure).
- `feedback_pim_cross_language_rule_mirror` §3.5g (load-bearing for U33 + §8.1 #2 single-static-dispatch-table refactor).
- `.addl/dispatch-conventions.md` §3.5s (cross-ecosystem-identifier discipline; orthogonal to FV but composes with kani scanner targets).

---

**End of C5 formal-methods critique.**
