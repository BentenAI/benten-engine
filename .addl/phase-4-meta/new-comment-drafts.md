# New comment drafts — F4a + 14 NEW threads + CFRG datapoint

**Provenance**: drafted 2026-05-26 by orchestrator post-cryptographer-review + 5 ecosystem-scan returns + Ben's L12 ratification (LAMPS-default + Inv-15 framework + G-CORE-PQ-WIRE-1 bundled hardening).

**Status**: DRAFTS for Ben review + edit + post. Do NOT post without Ben sign-off — these are public artifacts.

**Standing posting discipline** (applies to ALL drafts below):
1. Verify links + thread state still resolve + verify cite-anchors still resolve before posting
2. Be brief — drafts are deliberately ~150-250 words each; resist expansion
3. Humble framing (5-week-old project; 1 contributor; pre-public-launch)
4. NO "we believe industry should..." language — endorse direction + share shipping evidence
5. NO AI-attribution
6. Cite-anchor every spec / OID / draft / PR reference

**Suggested posting order** (urgency × leverage):
1. **F3 JOSE adoption call** — see [f3-jose-comment-draft.md](f3-jose-comment-draft.md) (3-day deadline 2026-05-29)
2. **F4a LAMPS issue #289** (this file §1; offers test vectors — concrete deliverable)
3. **F2 multicodec PR #400 / #403** — see [f2-f7-comment-drafts.md](f2-f7-comment-drafts.md) (vmx APPROVE 2026-05-04; window narrowing)
4. **sigstore/rekor-tiles #425** (this file §2; PERFECT construction match)
5. **w3c/vc-data-integrity #338** (this file §3; SING WG-discussion-request)
6. **sigstore/sigstore #2129** (this file §4; PQC plugins direction)
7. **w3c-ccg/di-quantum-safe #5** (this file §5; hybrid-composite codepoint gap)
8. **F7 W3C CCG #70 / #74** — see [f2-f7-comment-drafts.md](f2-f7-comment-drafts.md) (stable timing)
9–15. Lower-urgency threads (this file §6–§14)
16. CFRG draft-prabel adopter datapoint (this file §15; ecosystem-presence signal)

---

## §1 — F4a — LAMPS issue #289 negative test-vectors comment

**Thread**: https://github.com/lamps-wg/draft-composite-sigs/issues/289
**Mailing list cross-post optional**: `spasm@ietf.org`
**Stance**: F4a vehicle per L9 finding ("highest-leverage adopter contribution"). Concrete deliverable — actual test vectors offered. Longer-than-typical draft because the substance IS the test-vector list.

> Offering concrete negative test vectors as a production adopter shipping `id-MLDSA65-Ed25519-SHA512`.
>
> We (Benten Engine — content-addressed graph engine, v1-beta-tagging imminent) have been preparing a corpus of negative test cases as part of our internal conformance testing of LAMPS Composite ML-DSA. Happy to contribute the following 5 vectors as adopter-deposited test artifacts (will assemble + post in a follow-up reply, or via PR to the [lamps-wg/draft-composite-sigs](https://github.com/lamps-wg/draft-composite-sigs) repository if that's the preferred deposit path):
>
> 1. **classical-only-valid**: composite signature where the Ed25519 component verifies but the ML-DSA component fails — the both-must-verify check should reject. Tests that no implementation accidentally short-circuits to "classical verified → accept."
> 2. **PQ-only-valid**: composite signature where the ML-DSA component verifies but the Ed25519 component fails — symmetric to above. Tests the reverse short-circuit hazard.
> 3. **suite-swap (composite-as-classical)**: composite signature bytes presented as if they were a classical-only signature, i.e. with a classical algorithm OID — codepoint dispatch should reject as algorithm-identifier mismatch before reaching any verifier. Tests algorithm-substitution resistance.
> 4. **domain-separation corruption**: composite signature where the `Prefix || Label || len(ctx) || ctx || PH(M)` domain-separation construction (per draft-19 §4) is corrupted at the domain-separation byte — should reject. Tests domain-separation enforcement.
> 5. **order-swap**: composite signature with ML-DSA-65 and Ed25519 component bytes in the wrong concatenation order — should reject as decoding error or as cryptographic verification failure. Tests positional binding.
>
> Each vector to include: input message; signing key material (test-only keys, clearly marked); the corrupted signature bytes; the expected rejection failure mode + error path. Vectors generated against `draft-ietf-lamps-pq-composite-sigs-19` (current draft state at submission time).
>
> Standing offer: as Benten's LAMPS deployment matures + we accumulate edge-case findings from production traffic, happy to contribute additional vectors. The "highest-leverage adopter contribution" framing is the kind of cross-implementation discipline that strengthens the spec's testability — particularly with the EUF-CMA-only scope per §9.2.2 making negative-case coverage especially load-bearing for downstream implementers.

---

## §2 — sigstore/rekor-tiles #425 — dual-sign Rekor v2 with Ed25519+ML-DSA-65

**Thread**: https://github.com/sigstore/rekor-tiles/issues/425
**Stance**: PERFECT construction match — they're requesting EXACTLY what Benten ships. Highest-leverage concrete-shipping-evidence comment.

> Sharing concrete adopter perspective on this. We (Benten Engine — content-addressed graph engine, v1-beta imminent) ship `id-MLDSA65-Ed25519-SHA512` (LAMPS Composite ML-DSA per [draft-ietf-lamps-pq-composite-sigs-19](https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/); OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20) as our default signature construction — the exact same Ed25519+ML-DSA-65 hybrid this issue proposes. The threat-model logic is the same: long-term-persistent signed artifacts that get re-validated indefinitely + harvest-now-forge-later concern on persistent attestations.
>
> If useful: happy to contribute test vectors from our v1-beta shipping artifacts, including the negative-test-cases set we're preparing for LAMPS issue [#289](https://github.com/lamps-wg/draft-composite-sigs/issues/289) (suite-swap / classical-only-valid / PQ-only-valid / domain-sep corruption / order-swap).
>
> One concrete cross-system data point we've found load-bearing: LAMPS Composite ML-DSA is EUF-CMA-only (not SUF-CMA) per §9.2.2 of the draft. We close the SUF-CMA gap at the application layer via a 3-layer decomposition: identity = canonical-payload-CID; authentication = codepoint-dispatched signature; revocation = semantic tuple (never sig-bundle-CID). Worth a sanity-check on Rekor v2 — if any checkpoint-identification or revocation path is keyed off the signed-bundle bytes rather than the canonical-payload bytes, the SUF-CMA gap could matter; if everything stays payload-keyed, EUF-CMA suffices structurally.

---

## §3 — w3c/vc-data-integrity #338 — set-vs-chain hybrid signatures

**Thread**: https://github.com/w3c/vc-data-integrity/issues/338
**Stance**: SING WG explicitly invited adopter input + thread dormant since 2025-05; ripe for adopter signal with concrete deployment evidence.

> Adding adopter perspective on this set-vs-chain question. We (Benten Engine — content-addressed graph engine, v1-beta-tagging imminent) deployed the LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` strip-resistant CONCATENATED/COMMITTING combiner construction as our default signature scheme.
>
> Concrete data points on the set-vs-chain question:
>
> - **Wire-format simplicity** — single signature object (~3373 bytes per draft-19 sizes) is easier than multi-object set; encoding/decoding has one path, not N
> - **Verification protocol uniformity** — one verify call dispatched by multicodec/varsig codepoint, vs. per-component verification + result-combining at the application layer
> - **Cross-ecosystem interop** — OpenPGP-PQC [draft-ietf-openpgp-pqc-17](https://datatracker.ietf.org/doc/html/draft-ietf-openpgp-pqc) mandates the same Ed25519+ML-DSA-65 composite (RFC publication expected H1-2026; Sequoia PGP committed ship-on-publication); BouncyCastle 1.80+ + OpenSSL 3.5 + AWS KMS all ship LAMPS composite — the combiner-form has converging ecosystem support
>
> One application-layer hardening we found load-bearing: LAMPS is EUF-CMA-only per §9.2.2. We mitigate by keeping all CID-based identifiers off sig-bundle bytes (identity = canonical-payload-CID; revocation = semantic tuple). The 3-layer decomposition gives SUF-CMA-equivalent security at the application layer regardless of which combiner the VC-DI ecosystem lands on. Worth considering in any set-vs-chain trade-off analysis.

---

## §4 — sigstore/sigstore #2129 — PQC plugins proposal

**Thread**: https://github.com/sigstore/sigstore/issues/2129
**Stance**: jas4711 mid-thread already echoes Benten posture; complementary adopter signal helps the proposal.

> Adopter signal supporting this PQC plugins direction. We (Benten Engine — content-addressed graph engine) ship at v1-beta with LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`) as default, behind a codepoint-dispatched agility seam that already supports multiple sig algorithm arms (classical Ed25519 + LAMPS hybrid + reserved ML-DSA-65/SLH-DSA hybrid).
>
> The plugin direction maps directly onto how we've structured our crypto-agility:
> - one thin Benten-owned integration crate wrapping vetted upstream primitive crates (RustCrypto-class)
> - algorithm choice via multicodec/varsig codepoint dispatch + typed-unsupported-algorithm rejection (never silent fallback)
> - new algorithms add as additive impl + reserved codepoint without wire-format break
>
> One implementation note from shipping LAMPS specifically: keep all CID-based identifiers off sig-bundle bytes — identity = canonical-payload-CID; revocation = semantic tuple. Gives SUF-CMA-equivalent at the application layer despite LAMPS being EUF-CMA-only at the construction layer. Useful regardless of which PQ signature plugin Sigstore lands on; the discipline scales with any future SUF-CMA-preserving combiner addition too.

---

## §5 — w3c-ccg/di-quantum-safe #5 — Multikey names+prefix values

**Thread**: https://github.com/w3c-ccg/vc-di-quantum-safe/issues/5
**Stance**: Wind4Greg's 2026-05-19 update LEAVES hybrid composite codepoints as gap — Benten ships exactly the missing piece.

> Adopter signal on the hybrid composite codepoints gap. We (Benten Engine — content-addressed graph engine, v1-beta imminent) ship LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20) as default — the exact hybrid composite shape the current di-quantum-safe Multikey registration leaves unmapped.
>
> Two suggestions toward closing the gap:
>
> 1. **A "Composite Multikey" subsection** alongside the pure-NIST entries. Container-form codecs (multicodec PRs [#400](https://github.com/multiformats/multicodec/pull/400) + [#403](https://github.com/multiformats/multicodec/pull/403) — adding `cose-key` / `cose-key-set` / `jwk` / `jwk-set` at codepoints `0x40`-`0x45`) give a natural carrier for hybrid composite keys without requiring per-algorithm multicodec entries.
>
> 2. **Cross-link to LAMPS / JOSE / COSE algorithm-naming registries** — algorithm-naming stability across WGs lowers cross-stack adoption cost. JOSE [draft-skokan-jose-hpke-pq-pqt-05](https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/) (in adoption call ending 2026-05-29) is the parallel JOSE-side registration story; LAMPS draft-19 in RFC-Editor queue is the LAMPS-side.
>
> Happy to contribute an initial draft of the Composite Multikey subsection if useful — we have the codepoint-dispatched composite-key serialization pattern implemented in our integration crate, which could port to a reference impl.

---

## §6 — Nostr NIP-44 #1971

**Thread**: https://github.com/nostr-protocol/nips/issues/1971
**Stance**: paulmillr maintained; very alive; encryption-vs-sig system-class observation is the constructive cross-system framing.

> Adjacent-system observation on the system-class question. We (Benten Engine — content-addressed graph engine, v1-beta imminent with PQ-hybrid sigs as default) noticed during architectural planning that the PQ-encryption-vs-PQ-signature split tracks system-class more cleanly than threat-timeline:
>
> - **Long-term-persistent-artifact systems** (content-addressed storage, code-signing transparency logs, our content-addressed graph database) have a HNDL-equivalent for signatures — "forge-now-deploy-later" / TNFL on persistent attestations + retrospective evidence-planting
> - **Ephemeral-session systems** (Signal/PQXDH, MLS, TLS) reasonably defer PQ identity signatures because the session is over before any forgery would matter
>
> Nostr sits in an interesting middle: event-encryption-at-rest has HNDL applicability (PQ-hybrid AEAD like X-Wing-style fits cleanly today); event-signing system-class depends on how persistent the event corpus is treated long-term + whether events get re-validated by third parties decades later.
>
> No advocacy from us on what Nostr should do — different design space — just sharing the lens we found useful for distinguishing encryption-decision from signature-decision in our own architecture.

---

## §7 — w3c/cid #162 — constant-time decoding of `secretKeyMultibase`

**Thread**: https://github.com/w3c/cid/issues/162
**Stance**: dlongley explicitly invited concrete proposals + thread went quiet. Concrete proposal as production adopter.

> Adopter perspective on the constant-time decoding question. We (Benten Engine) ship LAMPS Composite ML-DSA at v1-beta, which makes PQ key sizes concrete: ML-DSA-65 private key is ~4032 bytes (vs Ed25519's 32 bytes — ~126× expansion). The non-constant-time decoding hazard scales with key size; what's marginal on Ed25519 is meaningful on ML-DSA-65.
>
> Concrete proposal toward @dlongley's "concrete proposals" ask: **per-codepoint constant-time gating** at the decoder level. The multicodec/multibase decoder dispatches on codepoint anyway; gate the constant-time-vs-fast-time path by codepoint:
>
> - PQ private-key codepoints route through `subtle`-style constant-time decoder
> - classical-only codepoints can use fast-time decoder (preserves current perf)
> - unknown codepoints typed-reject (never silent fallback)
>
> Avoids global perf regression while closing the side-channel surface on the codepoints where it actually matters. We have the pattern implemented in our Rust integration crate; happy to share the impl approach or port to a reference impl if useful.

---

## §8 — ATProto discussion #3928

**Thread**: https://github.com/bluesky-social/atproto/discussions/3928
**Stance**: Explicitly NOT advocating rotation-key changes; just readiness state signal on multicodec ML-DSA assignments.

> Adjacent-system data point — explicitly NOT advocating for rotation-key changes (different design constraints), just flagging readiness state on the multicodec assignments referenced upthread:
>
> - LAMPS Composite ML-DSA OIDs IANA early-allocated 2025-10-20 (incl. `id-MLDSA65-Ed25519-SHA512`, OID `1.3.6.1.5.5.7.6.48`)
> - Container-form multicodec entries landing at PRs [#400](https://github.com/multiformats/multicodec/pull/400) (`cose-key` etc., codepoints `0x40-0x43`) + [#403](https://github.com/multiformats/multicodec/pull/403) (`jwk` etc., `0x44-0x45`) — hybrid composite keys ride these containers without per-algorithm multicodec PRs
> - We (Benten Engine — content-addressed graph engine, v1-beta imminent) ship the LAMPS composite as default; production-deployment evidence will accumulate in coming months
>
> Useful if/when ATProto wants to consider the broader multikey-extensibility question. No opinion on the rotation-keys-specifically question raised upthread.

---

## §9 — sigstore/model-transparency #595

**Thread**: https://github.com/sigstore/model-transparency/issues/595
**Stance**: same persistent-artifact threat model; complementary adopter perspective on "why now."

> Adopter perspective on the persistent-artifact threat-model question. We (Benten Engine — content-addressed graph engine) ship at v1-beta with LAMPS Composite ML-DSA as our default signature construction, on essentially the same threat-model logic this thread is debating: signed artifacts intended to remain verifiable for decades face a "forge-now-deploy-later" / TNFL adversarial scenario that ephemeral-session protocols (TLS, messaging) don't.
>
> On the "why now" question specifically:
> - HNDL for encryption is canonical; the trust-now-forge-later (TNFL) analogue for signatures is an 8-year-old named threat class with industry literature (Josefsson's 2024 SPHINCS+-for-git proposal is one model-transparency-adjacent example)
> - OpenPGP-PQC [draft-ietf-openpgp-pqc-17](https://datatracker.ietf.org/doc/html/draft-ietf-openpgp-pqc) mandating the same Ed25519+ML-DSA-65 composite (RFC publication expected H1-2026; Sequoia PGP committed to ship-on-publication) is the cross-ecosystem signal that this isn't a niche concern
> - Sigstore + Trail of Bits retooling reported in Sigstore PQC blog (Feb 2025) is the model-transparency-specific signal
>
> Adopter timing data: we judged the harvest-now-forge-later concern severe enough on persistent attestations to ship PQ-hybrid as default at v1-beta even with imperfect implementation-maturity. Useful framing for the "why this artifact class matters" half of the question.

---

## §10 — ipfs/specs #448 — New IPNS key types

**Thread**: https://github.com/ipfs/specs/issues/448
**Stance**: 2.5-year-dormant; 2023 objections largely resolved 2024-2026 by multicodec ecosystem maturation.

> Light adopter signal — bringing this thread fresh data points. 2023 objections that paused it look largely resolved by 2024-2026 multicodec ecosystem maturation:
>
> - Container-form multicodec codecs (PRs [#400](https://github.com/multiformats/multicodec/pull/400) + [#403](https://github.com/multiformats/multicodec/pull/403)) make new key types adoptable without per-algorithm multicodec PRs — the "need a multicodec PR per new key type" objection is structurally addressed
> - PQ-hybrid composite key types (LAMPS `id-MLDSA65-Ed25519-SHA512`, IANA early-allocated 2025-10-20; we ship at v1-beta) are now mature enough to design IPNS around without speculating on what PQ landscape will look like
> - W3C did:key has emerging "any multikey value allowed without spec amendment" consensus (w3c-ccg/did-key-spec [#70](https://github.com/w3c-ccg/did-key-spec/issues/70)) — the extensibility-without-spec-amendment direction is cross-ecosystem
>
> No specific direction proposal from us; just noting that the ecosystem context has moved enough that picking this up fresh might be productive.

---

## §11 — RustCrypto/KEMs #25 — constant-time evaluation

**Thread**: https://github.com/RustCrypto/KEMs/issues/25
**Stance**: 2yr open; bumping as production user. Cite the ml-dsa CVE-class precedent.

> Bumping this from a production `ml-kem` user. We (Benten Engine — content-addressed graph engine shipping PQ-hybrid AEAD at v1-beta) use `ml-kem` for our X-Wing-hybrid combiner at multicodec codepoint `0x647A`. The constant-time-evaluation hazard surface this issue flags is more pressing in 2026 than it was at issue-open:
>
> [GHSA-hcp2-x6j4-29j7](https://github.com/RustCrypto/signatures/security/advisories/GHSA-hcp2-x6j4-29j7) (timing-side-channel in `ml-dsa`, sibling crate; patched in 0.1.0-rc.3) demonstrated that the bug-class IS exploitable in practice on lattice-based PQ primitives. If `Encode::<U1>::decode` similarly has timing-leakage paths, that's a production-relevant CVE-class.
>
> Happy to help with property-testing infrastructure (we run constant-time property tests in our integration crate against `dudect`-style oracles); not equipped to do the deeper cryptographic analysis but available as production-user-signal / test-channel if useful.

---

## §12 — RustCrypto/signatures #1020 — ZeroizeOnDrop for KeyPair

**Thread**: https://github.com/RustCrypto/signatures/issues/1020
**Stance**: trivial; +1 with adopter context (or small PR if Ben prefers).

> +1 from a production user. We (Benten Engine) ship `ed25519-dalek` + `ml-dsa` keypairs in production at v1-beta; the lack of `ZeroizeOnDrop` on `KeyPair` is a real hygiene gap. May contribute a small PR; otherwise this is a +1 with adopter context.

---

## §13 — Matrix-spec #975

**Thread**: https://github.com/matrix-org/matrix-spec/issues/975
**Stance**: 4yr-open thread; 2026 standards-state digest as value-add.

> 2026 standards-state digest for whoever picks this up:
>
> - Multicodec container-form codecs (PRs [#400](https://github.com/multiformats/multicodec/pull/400) / [#403](https://github.com/multiformats/multicodec/pull/403)) landed approval-wise — provides JWK/COSE-Key carrying without per-algorithm multicodec entries
> - LAMPS Composite ML-DSA OIDs IANA early-allocated 2025-10-20; OpenPGP-PQC [draft-17](https://datatracker.ietf.org/doc/html/draft-ietf-openpgp-pqc) mandates `ML-DSA-65+Ed25519` (RFC H1-2026)
> - W3C did:key emerging "any multikey value allowed" consensus (w3c-ccg/did-key-spec [#70](https://github.com/w3c-ccg/did-key-spec/issues/70))
>
> We (Benten Engine — content-addressed graph engine) ship at v1-beta with LAMPS hybrid as default. Adjacent-system signal that the underlying multikey + multibase landscape has moved meaningfully since this issue opened in 2022.

---

## §14 — w3c-ccg/did-cel-spec #4 — Multikey.id fingerprint

**Thread**: https://github.com/w3c-ccg/did-cel-spec/issues/4
**Stance**: same PQ-key-size-impractical-as-identifier problem; adopter datapoint.

> Adopter datapoint on the Multikey.id fingerprint question. We (Benten Engine) hit the same PQ-key-size-impractical-as-identifier problem at v1-beta — ML-DSA-65 public key is 1952 bytes, too large to use raw as a DID identifier suffix.
>
> Our pattern: `did:key:z<multihash(canonical_pubkey_bytes)>` where the multihash is BLAKE3-256 (or SHA-2-256, depending on agility profile). Keeps DID identifiers compact while remaining content-addressed to the actual key bytes. Concretely: `Multikey.id = multihash(pubkey)` aligns with @filip26's user-defined-with-fingerprint-fallback proposal in the natural shape.
>
> Happy to share specific impl details if useful.

---

## §15 — willowprotocol.org #175 — canonic Ed25519 spec clarification

**Thread**: https://github.com/earthstar-project/willowprotocol.org/issues/175
**Stance**: low-risk supporting clarification; Benten + Willow are adjacent-architecture systems.

> Light supporting signal on this clarification. We (Benten Engine — content-addressed graph engine; similar architecture-class to Willow in the content-addressed long-term-persistence space) similarly canonicalize Ed25519 signatures and use them in content-addressed long-term-persistent contexts; the spec clarification this issue requests would help adjacent-system implementers (us included) verify we're matching the canonical encoding correctly. No specific text proposal from us — just adopter +1.

---

## §16 — CFRG mailing list — draft-prabel-cfrg-suf-hybrid-sigs adopter datapoint

**Mailing list**: `cfrg@irtf.org`
**Stance**: Ecosystem-presence signal; flag Benten's awareness + future-direction posture (Bird-of-Prey-class as future-additive, not v1-beta-default). Lower-priority than the others but accumulates standing in CFRG.

> Adopter datapoint on draft-prabel-cfrg-suf-hybrid-sigs from a system that just considered + deferred SUF-CMA-preserving-hybrid adoption.
>
> We (Benten Engine — content-addressed graph engine, v1-beta-tagging imminent) evaluated draft-prabel + Bird-of-Prey (Bossuat et al., EUROCRYPT 2026, [IACR 2025/1844](https://eprint.iacr.org/2025/1844)) vs LAMPS Composite ML-DSA for our v1-beta default signature construction. We chose LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20) primarily on:
>
> - **Ecosystem interop**: OpenPGP-PQC draft-17 mandates the same composite; BouncyCastle 1.80+ / OpenSSL 3.5 / AWS KMS ship LAMPS; reference impls broadly available
> - **Implementation maturity**: production Rust impls + audited test vectors exist for LAMPS; not yet for SUF-CMA-preserving constructions
> - **WG adoption posture**: LAMPS is WG-document; draft-prabel is individual submission (not WG-adopted)
>
> The SUF-CMA gap LAMPS leaves (EUF-CMA-only per §9.2.2; only weak non-separability per [I-D.ietf-pquip-hybrid-signature-spectrums]): we close it at the application layer via a 3-layer decomposition — identity = canonical-payload-CID; authentication = codepoint-dispatched signature; revocation = semantic tuple (never sig-bundle-CID). Gives SUF-CMA-equivalent security regardless of construction choice.
>
> Future posture: when SUF-CMA-preserving constructions like draft-prabel mature (WG-adoption + production-quality reference impls + independent impl audit), our crypto-agility framework would add as an additive codepoint — pure additive upgrade, no wire-format break, no migration. We see this as a natural future v1.x direction.
>
> One data point that may be useful for the WG-adoption case: the application-layer 3-layer decomposition described above ALSO closes the "EUF-CMA-only is not enough" criticism architecturally — meaning systems with content-addressed-audit-trail threat models can ship LAMPS-default-with-discipline today AND adopt SUF-CMA-preserving as additive when ready. Reduces the deadlock between "wait for the perfect construction" and "ship what's available."

---

## Triage table

| # | Section | Priority | Window | Approx words |
|---|---|---|---|---|
| F3 | (other file) | P0-deadline | 2026-05-29 | ~190 |
| F4a §1 | LAMPS #289 | P0-substantive | stable | ~370 |
| F2 #400/#403 | (other file) | P0-window | narrowing | ~250 |
| §2 sigstore/rekor-tiles #425 | P0-leverage | stable | ~270 |
| §3 w3c/vc-data-integrity #338 | P0-invitation | dormant-but-warm | ~250 |
| §4 sigstore/sigstore #2129 | P1-complementary | active | ~210 |
| §5 w3c-ccg/di-quantum-safe #5 | P1-gap-fill | active | ~250 |
| F7 #70/#74 | (other file) | P1-stable | stable | ~190 each |
| §6 Nostr NIP-44 #1971 | P2-cross-system | active | ~230 |
| §7 w3c/cid #162 | P2-concrete-proposal | invited | ~210 |
| §8 ATProto #3928 | P2-readiness-signal | stable | ~180 |
| §9 sigstore/model-transparency #595 | P2-complementary | active | ~250 |
| §10 ipfs/specs #448 | P3-revival | dormant | ~210 |
| §11 RustCrypto/KEMs #25 | P3-bump | dormant | ~190 |
| §12 RustCrypto/signatures #1020 | P3-+1 | dormant | ~60 |
| §13 Matrix-spec #975 | P3-digest | 4yr-dormant | ~170 |
| §14 did-cel-spec #4 | P3-datapoint | recent | ~140 |
| §15 willow #175 | P3-+1 | recent | ~110 |
| §16 CFRG mailing list | P3-standing | ongoing | ~330 |

**Total drafts**: 21 ready (5 prior-batch in [f2-f7-comment-drafts.md](f2-f7-comment-drafts.md) + 1 F3 in [f3-jose-comment-draft.md](f3-jose-comment-draft.md) + 15 NEW + F4a + CFRG in this file).
