# P2P-Systems-Architect Review: Encrypt-to-Recipient at v1-beta — Architecture-Fit + Deployment-Shape Lens

**Reviewer:** Senior P2P-Systems Architect (engaged 2026-05-26)
**Decision-surface:** v1-beta wire-format freeze for Benten Engine encrypt-to-recipient construction
**Authority of this document:** ADVISORY. Final disposition rests with Ben.
**Lens:** architecture-fit + deployment-shape + composition-with-existing-Benten-stack. The senior cryptographer covers construction-soundness; the standards-skeptic covers maturity/ecosystem. I focus on WHAT SHAPE does encrypt-to-recipient need in Benten + how it composes with the rest of the stack.

---

## 0. Reading-order note

The TL;DR sits in §1. The most architecturally load-bearing sections are §1, §2 (per-option fit), §3 (the cross-perspective question), §7 (the Option-F extra-reflection-pass — this is where the most novel finding lives), and §10 (honest disagreement with the framing).

---

## 1. Executive recommendation

**RECOMMEND Option F (refined): ship the KEM-DEM-style single-recipient primitive at v1-beta as Layer-3 ("Drop-to-recipient envelope"), via Option B's X-Wing-inside-HPKE-mode-base shape, BUT DEFER the group/community primitive (the "Atrium-shared" use case) to a post-v1-beta wave behind a documented Layer-3.5 hook.** This is a structural split of what the brief treats as one decision into two clearly-separated decisions on architecture-fit grounds.

**Three load-bearing reasons:**

1. **The five vision use cases are NOT one primitive.** "Drop-to-Bob" is single-recipient encrypt-to-recipient. "Atrium-shared content" is fundamentally a *group* primitive whose right shape is closer to MLS-CGKA or Megolm than to HPKE-to-recipient. Conflating them into "one v1-beta envelope" forces a poor fit for at least one use case. The Matrix Olm/Megolm precedent (cite §8.4 — pairwise Olm channel distributes the Megolm group ratchet key) is the strongest evidence: even Matrix needed TWO layers because the use-cases are structurally different.

2. **Earthstar's "documents are not encrypted, apps encrypt at the application layer" stance (cite §8.3) is the wrong stance for Benten,** but the choice Earthstar made — *not freezing an envelope shape at v1* — is wise on architecture-fit grounds. Benten differs from Earthstar in that CLAUDE.md baked-in #15 + #18 commit confidentiality-half-of-Principal as a v1-beta-blocker, so we cannot defer ALL of it. But we CAN defer the group-shape decision (where the IETF MLS-PQ ciphersuites are still draft-04 / March 2026 / pre-WGLC — cite §8.5) while shipping the single-recipient envelope where the foundation is well-trodden.

3. **X-Wing is NOT the WG-adopted hpke-pq path; it is an Independent Submission (cite §8.2 — draft-connolly-cfrg-xwing-kem-10 "not endorsed by IETF, no formal standing").** Benten already vendors X-Wing for the storage-substrate per CLAUDE.md baked-in #5. *Reusing the SAME X-Wing combiner at the Drop-envelope layer is consistency-positive*. Switching to hpke-pq's `MLKEM768-X25519` combiner at the Drop layer means we ship TWO different KEM combiners at v1-beta (X-Wing at storage, hpke-pq at Drop) which is a worse architectural shape than one. The cryptographer-review-of-bird-of-prey precedent (§8.13) ratified X-Wing's vendored use for storage; extending that same primitive to Layer-3 Drop envelope is the consistency-preserving move.

**Confidence:** moderate-high on the structural split (Option F refined); moderate on Option-B-over-C-over-D for the single-recipient half; lower on the MLS-CGKA timing question (Option E timing is genuinely uncertain because MLS-PQ is moving). I'd want the senior cryptographer to validate that the "X-Wing inside HPKE-mode-base envelope" composition (Option B) is sound — that's THEIR lens not mine, but it's the load-bearing claim I'm relying on.

---

## 2. Per-option assessment from the architecture-fit lens

For each option I report GO / NO-GO / CONDITIONAL, the architecture-fit verdict, and the most important critique from this lens. I do NOT score cryptographic soundness rigorously (cryptographer's lens) or standards-process rigorously (skeptic's lens) — I cite where my verdict depends on them.

### Option A — Status quo (defer encrypt-to-recipient)

**Verdict: NO-GO** — does not meet the v1-beta commitment.

**Architecture-fit:** This option is internally inconsistent with the project's own ratifications. CLAUDE.md baked-in #15 explicitly states *"the v1 tag is now explicitly gated on the confidentiality half of the Principal primitive existing"* and baked-in #18 expands: *"capabilities give ZERO protection — encryption is the load-bearing substrate there"* for any untrusted-host scenario. The per-Node AEAD substrate is in place but it's encrypt-to-SELF (cite §8.1 — `K_principal` derivation from `namespace_did`), not encrypt-to-recipient. The Drop bundle / Garden-Grove / Kith selective-disclosure use cases NEED recipient-pubkey-encryption to function on untrusted hosts; without it, the v1-beta confidentiality story is "your data is encrypted to your own devices only," which is the trusted-engine case the project explicitly rules insufficient.

**Most-important critique:** Option A would force Ben to either (a) re-scope the v1-beta tag to exclude the untrusted-host use cases (rolling back baked-in #18), or (b) ship v1-beta without delivering on the confidentiality-half commitment, both of which weaken Position B's "we ship the full Principal-confidentiality story at v1-beta" framing materially. Option A is a deferral with downstream rationalization cost.

**Caveat:** A *partial* status quo — defer the *group/CGKA* half, ship the *single-recipient* half — is exactly Option F. So "defer EVERYTHING" is NO-GO; "defer the group half" is GO.

### Option B — HPKE-RFC-9180 envelope + X-Wing as KEM combiner

**Verdict: GO (for the single-recipient half) — recommended construction for Option F.**

**Architecture-fit:** This is the architecturally-cleanest single-recipient shape for Benten:

- **Reuses the existing X-Wing combiner** that the storage substrate already vendors (cite §8.1 + §8.10 — Benten already at codepoint `0x647A`). One KEM combiner across two layers. Versus Option C, which would introduce a SECOND KEM combiner (hpke-pq) at the Drop-envelope layer while X-Wing stayed at storage — an architecturally worse "two combiners at v1-beta" outcome.
- **HPKE-mode-base is the cleanest single-recipient envelope** (cite §8.6 — RFC 9180 §5.1.1). It does single-recipient KEM-DEM with a clean key schedule and explicit info/AAD binding, and it's the envelope MLS-PQ extends (cite §8.5). Reusing the same envelope shape that MLS uses means Benten's Drop-envelope architectural shape is forward-compatible with future MLS-style group constructions (Option E later).
- **X-Wing's design admits HPKE use natively** (cite §8.10 — X-Wing draft §5.6 "X-Wing satisfies the HPKE KEM interface"). This is not Benten inventing a new composition; this is documented intended use of X-Wing.

**Most-important critique:** The composition "X-Wing as the KEM inside HPKE-mode-base" is not a constellation that has a SINGLE published security proof — X-Wing has its own tight proof, HPKE-mode-base has its own proof (RFC 9180 §9), but the composition as a stack is "well-trodden by construction" rather than "proven jointly." A senior cryptographer should validate the composition; my read of the literature is that this is the standard KEM-DEM composition and the proofs DO compose (HPKE-mode-base requires only that the KEM is IND-CCA secure, which X-Wing provides classically and post-quantumly per Barbosa et al. 2024 — cite §8.10). But that's the cryptographer's call, not mine.

**Secondary critique:** This option does NOT serve the Atrium-shared / group use case well. Encrypting an Atrium snapshot to N recipients via N HPKE-mode-base envelopes (one per recipient) is O(N) per-recipient overhead at every write, and doesn't give the forward-secrecy / post-compromise-security properties that Atrium membership SHOULD have (members join/leave; old members shouldn't decrypt new content). For multi-recipient at scale, age/Saltpack's "wrap a shared payload key in N stanzas" (cite §8.11 + §8.12) is a better single-message shape but still NOT continuous-group-key-agreement. For the group case, Option E (MLS-CGKA) is the right primitive — but that's a separate, post-v1-beta decision.

### Option C — Skokan-draft HPKE-PQ-PQT-specific codepoints

**Verdict: NO-GO** — wrong architecture-fit for Benten at v1-beta on two counts.

**Architecture-fit:**

- **Two KEM combiners at v1-beta is worse than one.** Adopting Skokan-draft's hpke-pq KEM combiner at the Drop-envelope layer while X-Wing stays at the storage substrate splits Benten's crypto surface across two combiners (cite §8.7 — hpke-pq specifies sequential `KDF(ss_PQ, ss_T)` combination; cite §8.10 — X-Wing flattens the DH KEM into the SHA3-256 hash including `ct_X` and `pk_X`). These are STRUCTURALLY different combiners. Maintaining both at v1-beta doubles the surface for crypto-suite cite-drift, swap-matrix testing, and audit scope.
- **The "JOSE/COSE/MLS-stack interop" win is overstated for Benten.** Benten's wire format is CIDv1-multiformats native (cite §8.1 — CLAUDE.md baked-in #5: "self-describing multiformats framing"), NOT JOSE/JWE. Adopting Skokan-draft's codepoints at the Benten internal layer doesn't actually buy JOSE interop until the 3-layer-decomposition (cite §8.1 — "F3-JOSE-comment X-Wing-vs-HPKE-PQ disambiguation") exports to JOSE explicitly. That 3-layer-decomposition is the right place for JOSE-stack alignment; it shouldn't dictate the storage-internal envelope choice.

**Standards-maturity concern:** The brief notes Skokan-draft is "pre-WG-adopted" — that's also material per the recent Bird-of-Prey CONDITIONAL-NO-GO precedent (cite §8.13). But that's the standards-skeptic's lens; flagging here for cross-confirmation.

**Most-important critique:** Option C optimizes for the wrong axis. "Be the first JOSE-PQ adopter" is a marketing-strategy axis; "Be architecturally consistent across Benten's layers" is the engineering axis. Benten's vision (CLAUDE.md baked-in #5 + #18) is content-addressed + multiformats-native + P2P-untrusted-default — not JOSE-WG-aligned. JOSE export is an INTEROP layer, not the storage envelope.

### Option D — Benten-tailored wire format with X-Wing KEM (no HPKE envelope)

**Verdict: CONDITIONAL** — architecturally cleaner than Option B in one respect (smaller surface) but worse in another (no HPKE envelope means we re-invent the encrypted-context binding).

**Architecture-fit:**

- **Smaller-surface argument:** Option D is the smallest possible v1-beta surface: X-Wing.encap(pubkey_recipient) → 32-byte shared secret → ChaCha20-Poly1305(plaintext, AAD = whatever-binding-data). No HPKE key schedule, no info/exporter machinery, no PSK-mode / Auth-mode / Auth-PSK-mode complications. This matches the "vendored ~30-LOC X-Wing combiner" discipline (cite §8.1 — CLAUDE.md baked-in #5).
- **HPKE-envelope-omission cost:** RFC 9180 HPKE's KeySchedule (§5.1 of the RFC) does more than just bind AAD; it derives an explicit `exporter_secret` for context-binding, supports streaming AEAD over the same context, and provides a documented re-use story across messages in the same context. Benten's Drop use case is one-shot (encrypt the Drop bundle once); the multi-device-sync-via-untrusted-peers use case might want streaming AEAD; the Garden-Grove use case wants long-lived context binding. Bespoke envelope means Benten authors the equivalent of HPKE KeySchedule from scratch, which is exactly the kind of "fresh academic crypto" that the cryptographer-review's CONDITIONAL-NO-GO precedent (§8.13) flags as ship-blocking.
- **Multicodec maintainer concern (brief, paraphrased: "container-form not Benten-specific envelopes"):** I have not directly verified the multicodec maintainer position cited in the brief. Treating that claim as accurate: Option D shipping a Benten-specific envelope IS a stance the multicodec ecosystem has counseled against. Independent of that, the "fragmentation" concern is real: Benten as a 5-week-old project with a custom envelope when an RFC-published equivalent exists (HPKE-mode-base) means EVERY future interop integration has to teach the Benten envelope first.

**Most-important critique:** Option D treats Benten as a closed-system optimization problem ("smallest LOC, tightest proof"). The project's stated vision (CLAUDE.md baked-in #11 third pillar, peer-mesh + community-distributed) is fundamentally a cross-ecosystem-interop problem. The Drop bundle use case in particular — Alice sends to Bob via an UNTRUSTED intermediate peer — means the envelope WILL be parsed by software Benten doesn't control. RFC-published envelope shape (HPKE-mode-base) is a forward-interop investment; Benten-specific envelope is forward-interop debt.

### Option E — MLS-style group key derivation (CGKA) for community-shared content

**Verdict: CONDITIONAL — NOT for v1-beta. STRONG candidate for the post-v1-beta group-primitive wave. Architecturally complementary to Option B, not substitutable.**

**Architecture-fit:**

- **Atrium-shared content IS structurally a group primitive.** The brief lists use case #1 as "Atrium-shared content — communities are distributed copies; member peers should read, non-member peers should not." This is exactly the MLS-CGKA threat model (cite §8.5 — RFC 9420 + draft-ietf-mls-pq-ciphersuites-04). Joining/leaving members, forward-secrecy across membership changes, post-compromise-security after a key rotation — these are the GROUP properties Atriums actually need.
- **Single-recipient envelopes O(N) is wrong-shape at scale.** If you try to model "Atrium with 100 members" as "encrypt every Atrium update to 100 single-recipient envelopes via Option B," every member churn = re-key + re-encrypt entire snapshot to N stanzas. This is precisely the cost MLS-CGKA was designed to eliminate (TreeKEM gives logarithmic re-key cost; per cite §8.5 the MLS-PQ draft uses TreeKEM with PQ-KEM).
- **But MLS-PQ is draft-04 at March 2026 (cite §8.5).** WG-adopted but not WGLC, not RFC. Shipping MLS-PQ at v1-beta as the DEFAULT for community content would lock Benten to a moving spec target. Per the cryptographer-review-of-bird-of-prey precedent (§8.13), shipping a draft-stage construction as v1-beta default is the same anti-pattern.
- **Matrix Olm/Megolm is the precedent.** Matrix Olm/Megolm (cite §8.4) was designed and shipped BEFORE MLS existed because the same "we need group encryption" need predated the WG-adopted standard. The architectural lesson: 2-layer-decomposition (pairwise + group-ratchet) was chosen because the WG-adopted single-layer didn't exist. *Today*, MLS exists; the architectural lesson is "use the WG-adopted CGKA when shipping the group primitive, don't roll your own." This is an argument for using MLS-PQ when it stabilizes — NOT for inventing a Megolm-like construction at v1-beta.

**Most-important critique:** The brief lumps Option E with Options A-D as substitutable; my view is Option E is structurally *complementary*. Single-recipient Drop-to-Bob (Option B/D/F) and group-Atrium-content (Option E or its successor) are SEPARATE primitives, just as Matrix has both Olm (pairwise) and Megolm (group). v1-beta should ship one (the simpler, well-trodden one: single-recipient HPKE-mode-base with X-Wing); the other (group CGKA) lands when MLS-PQ stabilizes.

**Secondary critique:** "MLS-PQ-CGKA" implementations in Rust are not mature. `openmls` (Cryspen) is the leading impl (cite §8.5 — Cryspen page) but PQ support is a moving target. The "never fork crypto primitives" discipline (CLAUDE.md baked-in #5) would push Benten toward using OpenMLS as an upstream dep, but OpenMLS is a substantial crate with its own opinions about state management that may not fit Benten's content-addressed model cleanly. Integration design needs its own pre-work pass; that's another reason it can't happen at v1-beta on a 5-week timeline.

### Option F — Something else (my recommended structural shape)

**Verdict: GO — this is my preferred recommendation.**

**The shape:** A *two-layer architectural split*, codified at v1-beta:

1. **v1-beta ships Layer-3a "single-recipient Drop envelope"** = Option B (HPKE-mode-base + X-Wing KEM). Use case: Drop-to-Bob, multi-device-sync-via-untrusted-peers, Kith-disclosure-to-single-recipient. Codepoint reservation in `SigCodepoint` (or a parallel `EncryptToRecipientCodepoint`).
2. **v1-beta documents but does NOT ship Layer-3b "group/community CGKA envelope"** — explicitly reserves codepoint range + names the future MLS-PQ-derived construction as the planned implementation. Concrete commitment: when MLS-PQ ciphersuites reach IETF WGLC + OpenMLS (or equivalent) reaches Rust-ecosystem-stable, Layer-3b lands as an additive codepoint via the crypto-agility framework (CLAUDE.md baked-in #5 — never wire-break, always additive).
3. **At v1-beta, group-shared content uses Option B repeated per-recipient (multi-stanza, age/Saltpack-style — cite §8.11 + §8.12).** This is the EXPLICIT fallback shape: it's correct, it's well-trodden (age uses it; Saltpack uses it), it has known cost properties (O(N) per-recipient on group write), and it's structurally upgradeable to MLS-CGKA when Layer-3b lands without wire-format break.

**Why this is strictly stronger than picking one of A-E:**

- It services ALL FIVE vision use cases at v1-beta (with the known O(N) cost on Atrium-shared content).
- It does NOT ship a pre-WG-adopted construction as default (avoids the Bird-of-Prey precedent's failure mode).
- It explicitly NAMES the future upgrade path with a codepoint reservation, so the v1-beta wire-format-freeze doesn't block the Phase-7+ vision.
- It reuses X-Wing across layers (storage substrate + Drop envelope), keeping the crypto surface coherent.
- It's honest about the cost tradeoff (group content is O(N) at v1-beta; users know this when scoping Atrium membership lists).

**The composition story:** Together with the existing Inv-15 3-layer-decomposition (cite §8.1 — SECURITY-POSTURE.md Inv-15 section) and the per-DID storage partition (C1, G-CORE-1), the full Benten crypto stack at v1-beta becomes:

- **Layer-1: Per-DID storage partition** — `namespace_did` isolation (already shipped).
- **Layer-2: Per-Node AEAD** — `K(N) = KDF(K_principal, N.cid)` with X-Wing-hybrid `0x647A` key material (already shipped, modulo wave-3e `K_principal` test-seam-to-production swap).
- **Layer-3a: Single-recipient Drop envelope** — HPKE-mode-base + X-Wing KEM, this proposal (Option B inside Option F).
- **Layer-3b: Group CGKA envelope** — DOCUMENTED + RESERVED, future MLS-PQ-derived; v1-beta ships multi-stanza Option-B fallback per the age/Saltpack precedent.
- **Layer-4: Signature** — LAMPS Composite ML-DSA + Inv-15 application-layer mitigation (already ratified per cryptographer-review-of-bird-of-prey, §8.13).

This 4-layer (well, 5-with-3b) decomposition is architecturally coherent + cleanly upgradeable.

**Most-important critique of Option F itself:** It requires us to be HONEST in the v1-beta release notes / Position B blog that the group-CGKA shape is intentionally deferred. That's a marketing-framing cost. But honesty here is forward-correct: shipping pre-MLS-WGLC CGKA at v1-beta would force us to either lock a draft target or rev wire-format later. The honest deferral is the right call.

---

## 3. The cross-perspective question — where my lens applies

My lens (architecture-fit + deployment-shape) is load-bearing for THE FOLLOWING QUESTIONS that the cryptographer and standards-skeptic CAN'T resolve from their lenses alone:

1. **"Is encrypt-to-recipient one primitive or two?"** This is an architecture-fit question, not a crypto-soundness question. The cryptographer would happily review either single-recipient HPKE-mode-base OR group MLS-CGKA on construction merits; only the architecture lens can answer "Benten needs BOTH eventually, but they're separate primitives and v1-beta is too early for the group one."

2. **"Should we reuse X-Wing across layers or use different combiners per layer?"** Both X-Wing and hpke-pq are cryptographically defensible per their reviewers; the cryptographer's job is to validate each individually. The architecture-fit question is whether reusing one combiner (X-Wing) across storage+Drop is BETTER than using the WG-blessed combiner (hpke-pq) at one layer and X-Wing at another. My answer: reuse is structurally better; one combiner, smaller surface, less drift opportunity.

3. **"What's the right deferral path for the group-shared use case?"** The standards-skeptic can tell us MLS-PQ is draft-04; the cryptographer can tell us TreeKEM-PQ properties; only the architecture lens can answer "defer the group primitive, fall back to multi-stanza single-recipient at v1-beta per age/Saltpack precedent, reserve codepoints for future MLS-PQ when it stabilizes" as a coherent architectural shape.

4. **"Does the Drop bundle use case need streaming AEAD or single-shot?"** Architecture question. Drop bundles ARE single-shot (a Drop is encrypted-once, distributed, decrypted-once-per-recipient). Multi-device-sync-via-untrusted-peers MIGHT be streaming. HPKE-mode-base supports both via its KeySchedule. This is why HPKE-mode-base is the right envelope rather than the bespoke Option D — the use cases call for the wider envelope.

Where my lens DOESN'T resolve things:

- Whether X-Wing-inside-HPKE-mode-base composition is jointly proof-sound (cryptographer's call).
- Whether HPKE-PQ KEM combiner will displace X-Wing in IETF practice (standards-skeptic's call).
- Whether the LAMPS Composite ML-DSA + Inv-15 mitigation interacts cleanly with the encrypt-to-recipient layer (cryptographer's call — but I note that the answer should be "they're orthogonal layers" architecturally).

---

## 4. Position B blog framing impact

Recommendation: pursuing Option F refined ENABLES a STRONGER Position B framing than any of Options A-E individually.

**The strong-framing shape:** *"Benten ships the full Principal-confidentiality story at v1-beta — at-rest per-DID partitioning, in-flight per-Node AEAD, single-recipient Drop envelopes via HPKE+X-Wing, AND a documented upgrade path for group-shared community content as MLS-PQ stabilizes. We don't ship pre-WGLC drafts as defaults; we DO ship the production-ready pieces today and we DO name the future-additive pieces explicitly."*

This is materially stronger than:

- "Encryption is deferred to a future phase" (Option A — weakens vision-fit story).
- "We shipped a Benten-specific envelope we couldn't fully validate" (Option D risk).
- "We adopted a pre-WG-adopted JOSE draft as our default" (Option C — repeats the Bird-of-Prey anti-pattern).
- "We shipped a group CGKA that doesn't exist yet" (Option E timing risk).

**The Inv-15 honesty caveats (compromise #31 in SECURITY-POSTURE — cite §8.1) provide the template** for how to surface the Option-F structural-split honestly: "encrypt-to-recipient single-recipient shipped at v1-beta as default; group CGKA reserved as additive codepoint for the post-MLS-PQ-WGLC wave; v1-beta uses multi-stanza single-recipient as the explicit group fallback per the age/Saltpack precedent."

That's an HONEST framing that holds up to public scrutiny without overpromising.

**Blog implementation note:** Position B's v2 revision (cite §8.1 — "phase-4-meta-core/position-b-revision-v2" branch) is currently mid-revision per the SECURITY-POSTURE excerpt. The Option-F structural-split should be incorporated as honesty caveats #10 + #11 + #12 alongside the existing Inv-15 / Compromise #31 caveats #7-9.

---

## 5. Implementation guidance for Option F

If Ben ratifies Option F, the implementation discipline should include:

### 5.1 Codepoint reservation

- Reserve `EncryptToRecipientCodepoint::HYBRID_HPKE_MODE_BASE_X_WING_CHACHA20POLY1305 = 0x...` at v1-beta (the Layer-3a default).
- Reserve `EncryptToRecipientCodepoint::GROUP_MLS_PQ_TREEKEM_RESERVED = 0x...` at v1-beta with explicit "future-additive, NOT-IMPLEMENTED-AT-v1-beta" status flag in the dispatch table.
- Reserve `EncryptToRecipientCodepoint::MULTI_STANZA_SINGLE_RECIPIENT_FALLBACK = 0x...` as the explicit group-shape-at-v1-beta fallback codepoint. (This is the age/Saltpack-precedent shape.)
- The fail-closed unsupported-algorithm arm (cite §8.1 — CLAUDE.md baked-in #5) MUST type-reject any unknown encrypt-to-recipient codepoint.

### 5.2 Test surfaces

Beyond the cryptographer's required KAT + property tests:

- **Cross-layer composition test:** Drop bundle = X-Wing-encrypted to Bob via HPKE-mode-base, with Bob's recipient key derived from his `namespace_did`; verify decrypt round-trip + Wrong-Recipient-Fail-Closed.
- **Multi-stanza fallback test:** Encrypt to N recipients via Option-B repeated; verify any one recipient can decrypt; verify forward-secrecy is NOT claimed at this layer (a removed-recipient still holds the wrap-key for past Drops; this is the known limitation that Layer-3b will resolve).
- **Codepoint-dispatch matrix test:** Each reserved codepoint roundtrips through the cipher-suite dispatch + the future MLS-PQ reservation FAILS-CLOSED at v1-beta (it's reserved but unimplemented). This is the swap-matrix discipline from CLAUDE.md baked-in #5.
- **AAD-bind-recipient-DID test:** The HPKE-mode-base `info` parameter MUST bind the recipient DID + the Drop CID; mis-recipient or substituted-Drop scenarios must fail-closed via AAD mismatch.

### 5.3 Audit gates

- v1-beta ships with the LAMPS Composite ML-DSA / X-Wing audit commitment per the v1-GM exit criterion (cite §8.13 — C-GM-AUDIT in CLAUDE.md baked-in #15).
- The Layer-3a Drop envelope (HPKE-mode-base + X-Wing) SHOULD be included in the v1-GM audit scope as the encrypt-to-recipient surface. This is an audit-scope decision Ben should ratify with the audit firm; my recommendation is to include it.
- The Layer-3b MLS-PQ-derived future construction would get its own pre-merge cryptographer review when it ships, NOT under the v1-GM audit umbrella (because it isn't shipped at v1-beta).

### 5.4 Vendoring discipline

The X-Wing combiner is already vendored at ~30 LOC per CLAUDE.md baked-in #5. The HPKE-mode-base implementation should consume an upstream Rust HPKE crate (e.g. `hpke-rs` from Cryspen or the RustCrypto `hpke` crate) — NOT reimplement HPKE primitives. The "never fork, never reimplement crypto primitives" discipline applies to HPKE as much as to the underlying ML-KEM / X25519 primitives.

### 5.5 Cite-drift surfaces

- Add `EncryptToRecipientCodepointDriftScanner` to `cite-drift-detector` matching the `LoadBearingSigBundleCidPattern` pattern from Inv-15 (cite §8.1).
- The drift-detector should flag any new `pub` constant or method that takes a recipient pubkey or returns an encrypted-to-recipient envelope but does NOT route through the codepoint-dispatched primitive.
- Codify a pim-N rule: "All encrypt-to-recipient surfaces route through `benten_crypto_suite::encrypt_to_recipient::{wrap, unwrap}`. Direct calls to HPKE-mode-base primitives outside this module are a §3.5g class violation."

---

## 6. Risks + mitigations

### Risk R-1: X-Wing-inside-HPKE-mode-base composition has an un-noticed soundness gap

- **Likelihood:** Low. HPKE's KEM-DEM composition is well-analyzed and requires only IND-CCA security from the KEM; X-Wing provides that with a tight proof in the standard model for the PQ half and ROM for the DH half (cite §8.10 — Barbosa et al.).
- **Mitigation:** Senior cryptographer pre-merge review of the composition (the same review tier that landed cleanly on Bird-of-Prey-vs-LAMPS, cite §8.13). The senior cryptographer reviewing alongside this architecture review is the right venue.

### Risk R-2: HPKE Rust impl crate (hpke-rs or RustCrypto hpke) is the upstream supply-chain risk

- **Likelihood:** Moderate. Cryspen's `hpke-rs` is well-maintained; RustCrypto `hpke` is also actively maintained. Both depend on transitive `ml-kem` / `x25519` crates that are themselves on the CVE-prone substrate (cite §8.13 — the RUSTSEC-2025-0144 + GHSA findings on ml-dsa).
- **Mitigation:** Pin a single HPKE crate at v1-beta with version-pin discipline matching the `ml-dsa` pinning required by §8.13. Subscribe to RustSec advisories for the chosen HPKE crate. Pre-v1-GM: fuzz the Benten wrapper boundary.

### Risk R-3: Multi-stanza single-recipient fallback at v1-beta has unstated weaknesses for "removed Atrium member" use case

- **Likelihood:** Certain — this is a KNOWN limitation. A member removed from an Atrium at time T can still decrypt content encrypted before T (because they hold the wrap-key from when they were a member).
- **Mitigation:** EXPLICITLY document this limitation in SECURITY-POSTURE.md as a Compromise # entry. Surface it in Position B as honesty caveat. Name "this is what Layer-3b MLS-PQ will resolve" with the post-WGLC timeline. Users scoping Atrium membership at v1-beta KNOW they're picking a fallback shape with these properties. This is honest-and-shippable; pretending otherwise would be ship-blocking.

### Risk R-4: Layer-3a Drop envelope encourages over-Drop-ing (sending data to many recipients via individual Drops, inefficient)

- **Likelihood:** Moderate. Engineering teams will reach for the available primitive. Without the group primitive, multi-recipient writes WILL use O(N) Drops.
- **Mitigation:** Document the O(N) cost in the DX layer. Provide a DX-level "multi-recipient Drop" helper that wraps the multi-stanza fallback shape so the O(N) cost is at least UNDER A SINGLE API. Don't let users invent ad-hoc multi-stanza-equivalents. Codify in the dispatch-conventions §3.X "multi-recipient writes MUST route through `multi_stanza_drop()` at v1-beta until Layer-3b lands."

### Risk R-5: The MLS-PQ WGLC timeline is unknowable

- **Likelihood:** High that MLS-PQ ciphersuites don't reach WGLC before late 2026 / early 2027.
- **Mitigation:** The Option-F structural split DOES NOT depend on a fast MLS-PQ timeline. Layer-3b is documented + reserved but not built; the multi-stanza fallback at Layer-3a is the v1-beta-deliverable for group use cases. If MLS-PQ takes years, Benten ships the fallback indefinitely and re-evaluates at Phase 5+. The structural split is the resilient choice.

### Risk R-6: The "Phase 7-8 acceleration" claim — peers-hold-ciphertext / Garden-Grove / E2EE-subgraph is "weeks out" — may be premature

- **Likelihood:** Moderate. Ben's framing in the brief is "weeks out not years" on AI-accelerated tempo. My architecture-fit reading: even at AI-accelerated tempo, the GROUP primitive is multi-month not multi-week (MLS-PQ integration is substantial). The SINGLE-RECIPIENT primitive (Option B) is weeks-of-agent-dispatch-feasible.
- **Mitigation:** The Option-F structural split AGAIN handles this gracefully: ship Layer-3a (single-recipient) at v1-beta because it's weeks-feasible; defer Layer-3b (group) because honest assessment says it's not weeks-feasible. The "weeks out" framing aligns with WHAT WE'RE PROPOSING TO SHIP at v1-beta (the single-recipient half), not with what we're proposing to defer.

---

## 7. Extra-reflection-pass output — additional Option-F refinements

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`, taking one more pass on whether there's a strictly-stronger Option-F refinement:

### F-refinement-1: Reserve the AUTHENTICATED encrypt-to-recipient codepoint

X-Wing is NOT authenticated (cite §8.10 — "X-Wing is not an authenticated KEM: it does not support AuthEncap() and AuthDecap()"). HPKE-mode-base is not authenticated; HPKE-mode-auth IS (RFC 9180 §5.1.2). Benten's use cases include "Bob receives a Drop and wants to verify Alice sent it" (Drop sender authentication). At v1-beta we satisfy this via Layer-4 (LAMPS-signature) ATTACHED to the Drop bundle, NOT via HPKE-mode-auth. This is the architecturally cleaner layering — keep encrypt-to-recipient and signature as separate concerns.

But: the future HPKE-mode-auth shape might still be useful for tighter integration (e.g. for the multi-device-sync use case where the recipient device wants to authenticate the sender device under the SAME identity). Reserve `EncryptToRecipientCodepoint::HYBRID_HPKE_MODE_AUTH_X_WING_CHACHA20POLY1305 = 0x...` at v1-beta as a future-additive codepoint, explicitly NAMED.

This is the kind of "reserve-now, build-later" discipline that the v1-beta wire-format-freeze affords. Costs ~zero LOC at v1-beta; preserves the upgrade path.

### F-refinement-2: The "Drop CID as recipient-bound identifier" pattern

A Drop bundle's CID should be derived from the *plaintext + recipient set + binding context*, NOT the ciphertext. This matches the Inv-15 3-layer-decomposition principle (cite §8.1): identity bytes don't include malleable signature/encryption bits. The Drop CID structure should be something like `BLAKE3(plaintext || recipient_did_set || context_label)`. Benten's existing CID derivation patterns already handle this for plaintext; the Drop-CID extension would be a v1-beta-time decision.

I flag this not because it's load-bearing for Option F vs A-E, but because it's an architectural decision that compounds with Option F + Inv-15 in a way that should be ratified together.

### F-refinement-3: PER-CHUNK encrypt-to-recipient for streaming use case

The brief notes the multi-device-sync-via-untrusted-peers use case (use case #3). For LARGE Drops, per-chunk AEAD is required (cite §8.1 — `IROH_BLOCK_SIZE` = 16 KiB chunking). HPKE-mode-base supports streaming AEAD via its key schedule. The Benten encrypt-to-recipient implementation MUST support per-chunk AEAD streaming for Drops ≥ 64 KiB (matching the existing per-Node AEAD threshold).

This is concretely implementable via HPKE's `OpenBase` / `OpenAuth` returning a streamable AEAD context. Document this as an architectural requirement in the Option-F brief.

### F-refinement-4: The Inv-15 + Option-F + future Layer-3b unified-mental-model

Take a step back: Benten's architectural posture at v1-beta = "everything that crosses a trust boundary is content-addressed at the PLAINTEXT layer + has Layer-3 wrappers (encrypt + sign + commit) that are codepoint-dispatched." This is a *unified* mental model:

- Storage (Layer-2): per-Node AEAD; content-addressed at plaintext; AEAD-bound to plaintext-CID.
- Drop (Layer-3a): HPKE-mode-base + X-Wing; content-addressed at plaintext-Drop; AAD-bound to recipient-DID + Drop-CID.
- Group (Layer-3b): MLS-PQ-TreeKEM + group-key-bound chunk AEAD; content-addressed at plaintext-group-content.
- Signature (Layer-4): LAMPS Composite ML-DSA + Inv-15; signs canonical-PAYLOAD-CID, never sig-inclusive-bundle.

Every layer is codepoint-dispatched. Every layer can swap algorithms via additive codepoints. Every layer is bound to PLAINTEXT-CID identity, never to malleable-wrapped-bytes. This is THE invariant the architecture should ratify at v1-beta.

If this unified-model is the v1-beta posture, then Option F is the only choice that COMPLETES the unified model (Options A leaves Layer-3 missing; Option D fails the codepoint-additive discipline for the envelope shape; Options C fails the cross-layer-combiner-consistency).

---

## 8. Evidence base

Every claim above is cite-anchored here. URLs verified via WebFetch at review time; specification version numbers reflect 2026-05-26 state.

### 8.1 Benten internal sources

- `CLAUDE.md` baked-in #5 (crypto-agility / never-fork-primitives / codepoint-dispatch / typed-reject) — full text in the brief preamble + CLAUDE.md sections cited in the input package.
- `CLAUDE.md` baked-in #15 (v1-milestone-gate + confidentiality-half-of-Principal-as-v1-blocker + audit gates v1-GM).
- `CLAUDE.md` baked-in #18 (capability-gating-binds-cooperating-engine-only; encryption-is-load-bearing-on-untrusted-hosts).
- `CLAUDE.md` baked-in #17 (three engine deployment shapes: full peer / thin compute / embedded webview).
- `docs/SECURITY-POSTURE.md` lines 1935-2625 — Per-DID storage partition (G-CORE-1), Per-Node AEAD (G-CORE-3d / #1301), K_principal derivation, IROH_BLOCK_SIZE chunking, Inv-15 3-layer-decomposition, Compromise #31 (LAMPS EUF-CMA-only + Inv-15 application-layer closure).
- `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` — the CONDITIONAL-NO-GO precedent on Bird-of-Prey + the elegant-permanent-shape Inv-15 recommendation.
- `.addl/phase-4-meta/encrypt-to-recipient-architectural-review-context.md` — the shared input package for this review (Ben-ratified dispatch 2026-05-26).

### 8.2 X-Wing (draft-connolly-cfrg-xwing-kem-10, Independent Submission)

- Datatracker: https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/ (verified 2026-05-26)
- Status: Internet-Draft v10, expires 3 September 2026. "Not endorsed by the IETF, no formal standing in the IETF standards process."
- Stream: Independent Submission (NOT CFRG WG).
- IACR paper: Barbosa, Connolly, Duarte, Kaiser, Schwabe, Varner, Westerbaan, "X-Wing: The Hybrid KEM You've Been Looking For," IACR Communications in Cryptology Vol 1 Issue 1 (March 2024). https://eprint.iacr.org/2024/039
- Security: IND-CCA classically in ROM (X25519 strong-DH); IND-CCA post-quantumly in standard model (assuming ML-KEM-768 IND-CCA + SHA3-256 PRF). Tight proofs per Barbosa et al.
- Construction: X25519 + ML-KEM-768 FIXED (not generic combiner). Combiner = `SHA3-256(ss_M || ss_X || ct_X || pk_X || XWingLabel)`. Multicodec codepoint `0x647A` per Benten's adoption.

### 8.3 Earthstar — architectural precedent (documents-not-encrypted)

- Earthstar docs: https://earthstar-project.org/docs/how-it-works
- Exact text: *"documents are not encrypted. They are signed, which means we can prove they were not tampered with during their multi-hop journey to your computer."* (verified 2026-05-26)
- Future direction: *"Apps will be able to encrypt the content of documents and Earthstar will provide helper functions for this."*
- Earthstar v6 protocol is built on Willow. Identity = ed25519-based `cinn25519` keypair. Path-based access control via tilde-prefix.

### 8.4 Matrix Olm/Megolm — 2-layer encryption precedent

- Megolm spec: https://spec.matrix.org/v1.17/olm-megolm/megolm/
- Architecture: Megolm = symmetric group ratchet (AES-256 + HMAC-SHA-256, keys derived from ratchet state). Distribution = Olm-encrypted to-device messages (X3DH-style pairwise channel).
- Quote: *"A sender generates a Megolm outbound session and distributes the current ratchet state to all room members via Olm-encrypted to-device messages."*
- Key architectural takeaway for Benten: pairwise + group are SEPARATE primitives in Matrix; the 2-layer-decomposition was the architecturally clean choice when no single-layer WG-adopted CGKA existed.
- NCC Group Olm review: https://www.nccgroup.com/media/5bspr3ie/_ncc_group_olm_cryptogrpahic_review_2016_11_01-1.pdf

### 8.5 MLS RFC 9420 + MLS-PQ ciphersuites draft-04 + MLS-Combiner draft-02

- MLS RFC 9420: https://datatracker.ietf.org/doc/html/rfc9420 (published July 2023)
- MLS-PQ ciphersuites: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/ (v04, published 19 March 2026; expires Nov 7 2026; WG-adopted but pre-WGLC)
- MLS-PQ defines 9 ciphersuites using KEMs from `I-D.ietf-hpke-pq`. Quote: *"For the PQ/T hybrid KEMs and the pure ML-KEM HPKE integration, we use the KEMs defined in [I-D.ietf-hpke-pq]."*
- MLS-Combiner draft-02: https://datatracker.ietf.org/doc/draft-ietf-mls-combiner/ — session-level combiner, NOT a KEM combiner.
- OpenMLS (Cryspen Rust impl): https://cryspen.com/openmls/
- Key architectural takeaway for Benten: MLS-PQ uses hpke-pq combiner (NOT X-Wing). Adopting MLS at Layer-3b means adopting hpke-pq AT THAT LAYER ONLY; the storage substrate stays X-Wing. This is acceptable IF the layers are architecturally separated; it would be unacceptable as the SOLE combiner for both Drop and storage.

### 8.6 HPKE RFC 9180

- RFC 9180: https://datatracker.ietf.org/doc/html/rfc9180 (Feb 2022, published standard)
- §5.1.1 mode_base = single-shot encryption to a recipient public key
- §5.1.2 mode_auth = adds sender authentication via static-static DH
- §5.1.3 mode_psk / 5.1.4 mode_auth_psk = pre-shared key variants
- KEM-DEM framework; requires IND-CCA KEM, exporter_secret derivation, optional streaming AEAD via the same context.

### 8.7 draft-ietf-hpke-pq-04

- https://datatracker.ietf.org/doc/draft-ietf-hpke-pq/ (v04, expires Sept 3 2026; WG-adopted in HPKE WG)
- Defines KEMs: MLKEM768-P256, MLKEM768-X25519, MLKEM1024-P384 (PQ/T hybrid) + pure ML-KEM variants.
- Construction: `KDF(ss_PQ, ss_T)` — sequential KDF of concatenated shared secrets. NOT X-Wing's flattened SHA3-256 design.
- Used by MLS-PQ ciphersuites (cite §8.5). NOT mentioned in X-Wing draft + vice-versa — these are PARALLEL not COMPATIBLE choices.

### 8.8 Skokan-draft JOSE-HPKE-PQ-PQT

- https://datatracker.ietf.org/doc/draft-skokan-jose-hpke-pq-pqt/ (v05, expires Nov 14 2026; INDIVIDUAL DRAFT — pre-JOSE-WG-adoption)
- Registers JOSE/COSE algorithm identifiers for HPKE-PQ KEMs. KEM definitions from draft-ietf-hpke-pq.
- Purpose: enable existing JOSE/JWE/JWS-deploying ecosystems to use PQ-KEMs in HPKE. NOT a Benten-internal envelope replacement.

### 8.9 iroh-blobs — no encryption layer

- iroh-blobs docs: https://docs.iroh.computer/protocols/blobs (verified 2026-05-26)
- iroh-blobs blog: https://www.iroh.computer/blog/blob-store-design-challenges
- Architectural posture: BLAKE3-content-addressed transfer with verified streaming. No application-layer encryption; QUIC TLS for in-transit + assumes plaintext at-rest. "Encrypt before put" is application's responsibility.
- Crucial for Benten architecture: this confirms iroh-blobs is the TRANSPORT layer; encryption MUST be at the Benten layer; iroh's classical-TLS-only stance per CLAUDE.md baked-in #5 reframe is the explicit reason Benten doesn't rely on transport-PQ.

### 8.10 X-Wing security + HPKE-integration

- X-Wing draft §5.6: *"X-Wing satisfies the HPKE KEM interface."*
- X-Wing abstract: *"a general-purpose post-quantum/traditional hybrid key encapsulation mechanism (PQ/T KEM) built on X25519 and ML-KEM-768."*
- §1.2 design choices: deliberately NOT general (locks to X25519 + ML-KEM-768 + SHA3-256); justified by simplicity + tightness. Different from generic combiners like draft-ounsworth-cfrg-kem-combiners.
- §5.3 combiner: `SHA3-256(ss_M || ss_X || ct_X || pk_X || "\.//^\\"_bytes)` — includes pk_X and ct_X explicitly; this is the "binding context to ciphertext" property that makes the standard-model proof go through.
- IACR ePrint 2026/396 "Anonymity of X-Wing and its Variants" — additional anonymity analysis (cite-anchor for whether X-Wing's anonymity properties match Drop-bundle requirements; deferred to cryptographer's lens for full validation).

### 8.11 age encryption — multi-recipient stanza precedent

- age docs: https://github.com/FiloSottile/age/blob/main/x25519.go + https://words.filippo.io/age-authentication/
- Multi-recipient shape: header contains N stanzas, one per recipient; each stanza wraps the shared file-key with that recipient's pubkey-derived ephemeral key.
- Quote: *"Files encrypted to multiple recipients contain multiple stanzas (introduced by ->) in the header, with each stanza representing a different recipient."*
- Architectural takeaway: O(N) wrap cost per encryption; correct for "send the same data to N parties at once"; does NOT support member-removal-with-forward-secrecy.

### 8.12 Saltpack — encrypted-payload-key + per-recipient-wrap precedent

- Saltpack encryption spec v2: https://github.com/keybase/saltpack/blob/master/specs/saltpack_encryption_v2.md
- Quote: *"At a high-level, the message is encrypted once using a symmetric key shared across all recipients, then MAC'ed for each recipient individually."*
- 1MiB chunk size with sequential nonce; final-flag truncation defense.
- Ephemeral sender pubkey per message for sender-anonymity.
- Architectural takeaway: confirms the age-shape multi-recipient pattern. Independent NaCl-based implementation. Same trade-offs: O(N) cost; no forward-secrecy on removal.

### 8.13 Cryptographer-review-of-bird-of-prey precedent

- File: `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` (branch `phase-4-meta-core/cryptographer-review-bird-of-prey @ 36afe06b`)
- Disposition: CONDITIONAL NO-GO on Bird-of-Prey; recommended LAMPS Composite ML-DSA + Inv-15 application-layer mitigation at v1-beta.
- Pattern: shipping fresh academic crypto / pre-WG-adopted construction at v1-beta wire-format-freeze on a 5-week-old project is structurally indefensible.
- v1-GM exit criterion C-GM-AUDIT: independent third-party audit of pinned ml-dsa + ml-kem versions.
- This precedent is the load-bearing analogy for why Option C (Skokan-draft as default) is NO-GO at v1-beta.

### 8.14 Signal PQXDH — session-key-establishment vs encrypt-to-recipient

- Signal PQXDH spec: https://signal.org/docs/specifications/pqxdh/ (verified)
- Architecture: server-mediated asynchronous handshake; multi-DH + PQ-KEM-encapsulation; HKDF derives session key SK; subsequent AEAD encryption.
- NOT encrypt-to-recipient. Architecturally session-establishment, NOT message-to-recipient.
- Architectural takeaway: Signal's threat-model is REAL-TIME messaging; Benten's threat-model includes ASYNCHRONOUS Drop bundles where there's no server-mediated handshake. The patterns are NOT directly transferable. Benten's Drop use case is more analogous to age/Saltpack than to Signal.

### 8.15 Hypercore — Noise XX for transport, app-layer for content

- Hypercore architecture references: Noise XX handshake at transport; content-verification via BLAKE2b Merkle trees; no encryption-to-recipient at the protocol layer.
- Architectural takeaway: same as iroh-blobs — transport-layer encryption is independent of application-layer encrypt-to-recipient. Benten's layering matches both.

---

## 9. Self-assessment + confidence + what additional review I'd want

### 9.1 Confidence levels

- **HIGH confidence (≥85%):** Option A is NO-GO; Option C is NO-GO; the structural split into single-recipient + group-primitive is correct architecturally; X-Wing cross-layer reuse is the right consistency choice.
- **MODERATE-HIGH confidence (≥70%):** Option B is the right single-recipient construction at v1-beta; Option E is correctly DEFERRED rather than rejected; Option F (refined) is strictly stronger than A-E individually.
- **MODERATE confidence (~60%):** The "Phase 7-8 acceleration weeks-out" framing aligns with WHAT WE'RE PROPOSING TO SHIP at v1-beta (single-recipient half) but NOT with what gets DEFERRED (group half).
- **LOWER confidence (<50%):** MLS-PQ WGLC timeline; specific OpenMLS integration design for Layer-3b; whether the cryptographer will validate X-Wing-inside-HPKE-mode-base composition without finding new caveats.

### 9.2 Knowledge limits — what I'm INFERRING vs CITING

- I'm CITING (cite §8.X): all WG draft statuses, spec text, IETF stream positions, the X-Wing IACR paper claims.
- I'm INFERRING from documented patterns: the precise integration cost of Option B with `hpke-rs` / RustCrypto-hpke (no LOC estimate verified); the timeline for MLS-PQ to reach WGLC (could be 2026-Q4, could be 2027-Q2, could be 2028).
- I'm RELYING ON the cryptographer's review: that X-Wing-inside-HPKE-mode-base is jointly sound. If the cryptographer raises new caveats here, Option B (and therefore Option F) need re-evaluation.
- I have NOT directly verified the "multicodec maintainer counseled against Benten-specific envelopes" claim in the brief — treated as accurate, but the standards-skeptic should verify.
- I have NOT independently verified the L8 critic finding "Signal doesn't do PQ identity sigs because deniability" framing referenced in the brief — that's the standards-skeptic's lens.

### 9.3 Additional review I'd want before committing

- **Senior cryptographer cross-confirmation:** specifically on the X-Wing-inside-HPKE-mode-base composition + on whether HPKE's `mode_base` exporter_secret + KeySchedule are sound when composed with X-Wing-derived shared secrets. (Expected outcome: confirmation, but worth getting.)
- **Standards-skeptic cross-confirmation:** specifically on the X-Wing-Independent-Submission concern + the long-term IETF trajectory question (does X-Wing reach IRTF Information RFC status, or does it get displaced by hpke-pq).
- **Implementation reality check:** an `hpke-rs` or RustCrypto `hpke` integration spike (1-2 days agent-dispatch) to verify the API + version-pin discipline before committing to the design. Bird-of-Prey's CONDITIONAL-NO-GO was based on "no reference implementation in Rust"; we should verify HPKE-mode-base + X-Wing IS reference-implementable in Rust at v1-beta-feasible quality.

---

## 10. Honest disagreement — where I diverge from the brief framing

### 10.1 The brief frames Options A-E as substitutable; my position is they aren't

The shared input package lists Options A-E as alternatives to evaluate. From the architecture lens, Options B/C/D address ONE primitive (single-recipient envelope) and Option E addresses A DIFFERENT primitive (group CGKA). Option A defers BOTH. This is a structural error in the option framing: the right shape is two-decisions, not one. My Option F refinement makes the split explicit.

The brief acknowledges this slightly — *"May complement Option B/C/D rather than replace them — Drop-to-individual-Bob is encrypt-to-recipient; Atrium-member-only-content might be group-key"* — but treats it as a side note rather than the architecturally load-bearing distinction. From my lens, this is THE central architectural question.

### 10.2 The brief's "Position B blog framing impact" criterion may be overweighted relative to engineering merit

Criterion 9 in the evaluation rubric is "Position B blog framing impact." Architecturally, this is a marketing-strategy consideration that *follows from* the engineering choice, not one that *drives* the choice. The risk pattern from the Bird-of-Prey review is: making the wire-format-freeze decision under blog-framing pressure leads to shipping pre-WG-adopted constructions as defaults to look more aggressive in public framing.

My recommendation: weight criteria 1-8 as the engineering decision; criterion 9 follows. The Option-F structural split is engineering-correct AND happens to produce a stronger Position B framing than Options A or C; that's a fortunate alignment, not the reason for the choice.

### 10.3 The brief's "5 weeks old / 1 contributor" framing is doubly load-bearing

The shared input package mentions this twice as context for the v1-beta decision. From the architecture lens this matters for TWO reasons, not one:

- **Reason 1 (in-the-brief):** the project's small history means shipping fresh academic crypto as default is the Bird-of-Prey anti-pattern.
- **Reason 2 (additional from my lens):** the project's small history means EVERY architectural commitment at v1-beta is LOAD-BEARING for years. The 4-5-layer decomposition I'm recommending (Layer-1/2/3a/3b/4) is not just a v1-beta choice; it's the schema that EVERY future encryption surface in Benten will compose against. Getting the layering right matters as much as getting the construction right.

This is why I'm pushing for the structural split (Option F) rather than picking the safest single-construction (Option B alone or Option A alone). The structural split FUTURE-PROOFS the architectural shape; the single-construction-only choice could leave Benten with a Layer-3 that doesn't compose cleanly with the eventual MLS-PQ group primitive.

### 10.4 The "Atrium-shared content" use case may have been over-specified in the input package

Use case #1 in the brief: *"member peers should be able to read community content, non-member peers (who may still store the bytes per P2P storage participation) should not."* This combines two distinct requirements:

- (i) members read, non-members don't (READ ACCESS CONTROL)
- (ii) non-members can still STORE bytes (PEERS-HOLD-CIPHERTEXT)

Requirement (i) is achievable at v1-beta via the multi-stanza Option-B fallback (Atrium has N members, encrypt each Atrium write to those N members). Requirement (ii) is achievable at v1-beta via the storage substrate (per-Node AEAD already keeps non-namespace_did peers from reading bytes they store).

The MLS-PQ-CGKA shape is NOT REQUIRED for either (i) or (ii); MLS-PQ-CGKA is required for (iii) "forward-secrecy across membership changes" which is NOT explicitly in the input package's use case #1 phrasing.

Honest disagreement with the brief framing: *the Atrium-shared use case is over-specified relative to what the brief literally says is required at v1-beta.* My Option-F refinement assumes only the (i) + (ii) requirements at v1-beta, with (iii) deferred to Layer-3b. If Ben actually NEEDS (iii) at v1-beta, that changes the analysis materially — but the brief doesn't explicitly require it.

This is the most architecturally consequential disagreement: scoping the actual Atrium-shared-content requirement is a design question that should be ratified explicitly before committing to a construction.

### 10.5 The Drop-CID + plaintext-CID + identity model interaction is under-discussed in the brief

The Inv-15 3-layer-decomposition principle is referenced in the brief (cite §8.1), but the brief doesn't connect Inv-15 to the encrypt-to-recipient layer directly. From my lens, Inv-15 ALREADY DETERMINES MUCH of the Drop-envelope shape:

- Drops are content-addressed (have CIDs)
- Drop CIDs MUST be derived from canonical PAYLOAD bytes per Inv-15
- Drop CIDs MUST NOT include the encrypted-to-recipient envelope bytes (those are malleable per Inv-15's "sig-inclusive bundle bytes are NOT identity" principle)
- Drop CIDs MUST bind the recipient set (so revocation-by-tuple matches the recipient context)

This is structural input to the Drop envelope design. The brief should explicitly call this out as a constraint on the envelope choice. My Option F refinement F-refinement-2 names this; Ben should ratify it as a v1-beta design constraint independently of which encryption option is chosen.

---

## Summary

The Option-F refined recommendation: **ship Option B (HPKE-mode-base + X-Wing KEM) as Layer-3a single-recipient Drop envelope at v1-beta; reserve Layer-3b group CGKA codepoint as future-additive; use multi-stanza Option-B-repeated as the explicit v1-beta group fallback per age/Saltpack precedent; document everything honestly in Position B and SECURITY-POSTURE.**

This is strictly stronger than Options A-E individually because (1) it services all 5 vision use cases at v1-beta with explicit cost trade-offs; (2) it doesn't ship pre-WGLC drafts as defaults; (3) it reuses X-Wing across layers for crypto-surface coherence; (4) it explicitly names the upgrade path; (5) it composes cleanly with the existing Inv-15 + per-DID-partition + per-Node-AEAD + LAMPS-Composite-ML-DSA layers into a unified 4-5-layer mental model.

The most important architectural insight from this lens: **encrypt-to-recipient at Benten v1-beta is TWO primitives (single-recipient + group), not one.** Treating them as one decision forces a poor architectural shape for at least one use case. Splitting them explicitly + deferring the group primitive cleanly is the right architectural move.

