# Comment Opportunities — Content-Addressed + Long-Term-Persistence Systems

Scan target: open issues / PRs / discussions in n0-computer (iroh + adjacent), earthstar-project / worm-blossom (Willow Protocol + willow-rs), IPFS / IPLD / multiformats, sigstore + cosign + openpgp-pqc, libp2p, Filecoin, Arweave, Veilid, Ceramic, git-PGP / SPHINCS+-for-git.

Scan window: 2026-05-26 (current date), with focus on threads active in 2025-2026.

Method: gh CLI issue/search queries on each org + WebSearch / WebFetch for off-GitHub references (Codeberg, IETF drafts, Cloudflare blog corroboration). Read-only scan; no posts made.

Each row = thread title + URL + dates + state + relevance to Benten v1-beta PQ-hybrid posture + recommended-action + (where action = COMMENT-NOW) outline of comment shape.

---

## 1. iroh-cluster (Shape-5-private-outreach-flagged where Position-B argument is on-table)

Per scan brief: **default disposition for any iroh thread that touches the content-addressed-blob-persistence-vs-ephemeral-session distinction = WATCH-ONLY-PENDING-SHAPE-5** so the Position B framing reaches the iroh team privately first. Threads NOT touching that argument can be COMMENT-NOW where low-risk.

### 1.1 iroh #4195 — feat: Allow inspecting negotiated key exchange group
- URL: https://github.com/n0-computer/iroh/issues/4195
- Opened 2026-04-28, last active 2026-05-04, state OPEN
- Body: rklaehn (iroh contrib) asks for accessor on iroh `Connection` for the negotiated key exchange group (e.g. `X25519MLKEM768`); matheus23 (iroh member) responds "useful for debugging & metrics, but it shouldn't be used to rely on not establishing non-PQ connections. Because if that were the case, then X25519 and other elliptic-curve algos just don't fulfill your threat model and you'd never want to rely on them for encryption."
- **Direct Position B touch-point**: matheus23's reasoning ("if PQ matters, classical doesn't fulfill threat model") is exactly the framing Benten's Position B blog will engage with: it treats PQ as binary-threshold rather than hybrid-precaution, and it implicitly assumes the ephemeral-session model where forward secrecy + connection-replacement hide HNDL exposure.
- **Recommended action**: **WATCH-ONLY-PENDING-SHAPE-5**. Adding our hybrid-precaution counter-argument here publicly leaks Position B before the private iroh conversation; defer until after Shape-5 outreach.
- After Shape-5: if iroh team is receptive, a comment naming the at-rest / content-addressed-artifact threat model (vs ephemeral-session) could be valuable here as the canonical public anchor.

### 1.2 iroh #3849 — Support non-Ed25519 endpoint IDs
- URL: https://github.com/n0-computer/iroh/issues/3849
- Opened 2026-01-13, last active 2026-01-26, state OPEN, "will do this not for 1.0" (dignifiedquire, iroh member)
- Body: tracking issue for endpoint-ID extensibility beyond Ed25519; address resolution / pkarr currently Ed25519-bound but "could just refuse to work with other keys"; WIP exploratory PR https://github.com/n0-computer/iroh/pull/3653
- **Direct Position B touch-point**: this is the structural seam through which PQ-or-hybrid endpoint IDs would land in iroh. Any comment about why hybrid (Ed25519⊕ML-DSA-65) might be the right shape for that seam is exactly the Position B argument.
- **Recommended action**: **WATCH-ONLY-PENDING-SHAPE-5**. Same reasoning as #4195. The right venue for the Benten experience-report-as-design-input is the private channel first.

### 1.3 iroh #2355 — Allow using iroh with other signers (HSM / FIDO / smartcard / lair-keystore)
- URL: https://github.com/n0-computer/iroh/issues/2355
- Opened 2024-06-09, last active 2025-09-25, state OPEN
- Body: external-signer trait extraction (HSM / smartcard / lair-keystore use cases). rklaehn confirmed direction in 2024 ("pass the discovery traits a 'signer' instead of always taking a private key"); FROST experiment cited; ephemeral-keys-with-certificate-chain proposal from matheus23.
- **Touches Position B? Only tangentially.** The signer trait is the surface through which any future PQ-hybrid signer would plug in (Benten could ship a `BentenHybridSigner` external to iroh that signs the iroh TLS cert chain with the hybrid scheme). Not directly arguing the PQ-default question.
- **Recommended action**: **WATCH-ONLY-PENDING-SHAPE-5**, with low confidence — the signer-trait conversation has been going on 2 years without convergence and a Benten comment here would either be premature (no Benten signer yet) or risk re-litigating the Position B question by proxy. After Shape 5, a "we plan to plug a PQ-hybrid signer in here" comment may be useful design-input.

### 1.4 iroh #4147 — Consider the use of post quantum key exchange (CLOSED 2026-04-22)
- URL: https://github.com/n0-computer/iroh/issues/4147
- State CLOSED. Documents the iroh team's pq-only-key-exchange example and the `__rustls-post-quantum-test` feature; closed out as "answered." Sibling #4191 ("Enforce the use of post quantum key exchange") also CLOSED 2026-04-28 with similar disposition (PQ KEX is implementation-detail-flagged-opt-in).
- **Recommended action**: **IRRELEVANT (closed)** but useful context. The Benten Position B blog should cite these two threads as evidence that iroh's PQ stance is *KEX-only / signature-deferred*, consistent with their blog post.

### 1.5 iroh #3535 — Custom validation of NodeIds prior to direct connection
- URL: https://github.com/n0-computer/iroh/issues/3535
- Opened 2025-10-14, last active 2026-04-20, state OPEN, tentative 1.1 milestone
- Body: NodeId-validation-before-IP-disclosure for privacy-sensitive workloads. flub mapped out the TLS round-trip sequence + proposed `PeerValidated` API.
- **Touches Position B? No.** Privacy/disclosure question, not crypto-agility.
- **Recommended action**: **IRRELEVANT** for the comment-opps campaign. Benten consumes iroh as transport and would benefit from the API but has no design-input to add beyond what's already on-thread.

### 1.6 noq #463 — Investigate interactions between 0-RTT and PQC handshakes
- URL: https://github.com/n0-computer/noq/issues/463
- Opened 2026-02-27, state OPEN. matheus23 notes PQC enabling appears to break the zero_rtt test; test marked ignore in the PQC arm pending investigation.
- **Touches Position B? No.** This is a TLS-1.3-implementation bug, not a posture question.
- **Recommended action**: **WATCH-ONLY** — Benten cares about 0-RTT-with-PQC working but has no diagnostic to contribute.

### 1.7 iroh-blobs threads (no PQ-touching threads at this scan)
- iroh-blobs open issues 2025-2026 are about storage backend / GC / wasm support / store-refactor / API-renaming — none touch signing or content-addressing-CID-scheme.
- The Benten-relevant thread is iroh-blobs #90 (Browser / wasm support) since shape (b) thin-compute-surface depends on it landing; ipfs-blobs #170 (append-only log replication) is conceptually adjacent to Benten's VersionDag work but Benten isn't using bao-tree's append mode.
- **Recommended action**: **IRRELEVANT** for comment-opps; **WATCH-ONLY** for shipping dependency on #90.

---

## 2. Willow Protocol + IPFS/IPLD threads

### 2.1 willow-rs #131 — Willow'25: Sideloading encryption / decryption
- URL: https://github.com/earthstar-project/willow-rs/issues/131
- Opened 2025-08-14, state OPEN (no recent activity). "Once Willow'25 specifies the encryption used for Sideloading, we need to implement it in the Willow'25 crate."
- **Relevance to Benten**: Willow's sideloading-encryption is the same threat-model space as Benten's `#1301` encryption-as-confidentiality + Drop bundles (HNDL-relevant at-rest content). Willow's choice of cipher-suite + agility framing is directly informative.
- **Recommended action**: **WATCH-ONLY**. Issue is too tracker-stub-y to engage productively yet. Worth checking back when Willow'25 sideloading spec proposes specific algorithms.

### 2.2 willow-rs #112 — Support for Entry Encryption
- URL: https://github.com/earthstar-project/willow-rs/issues/112
- Opened 2025-05-14, state OPEN. Tracking issue (no spec content yet).
- **Recommended action**: **WATCH-ONLY**. Same posture as #131 — placeholder, not yet actionable.

### 2.3 willowprotocol.org #175 — TODO: canonic Ed25519
- URL: https://github.com/earthstar-project/willowprotocol.org/issues/175
- Opened 2025-12-07, state OPEN. Aljoscha Meyer (or worm-blossom): "Our implementation currently uses `ed25519_dalek::VerifyingKey::verify_strict`. Need to update the spec to reflect this. Can also use this opportunity to consider ristretto25519 once again."
- **Relevance to Benten**: Same dalek-Ed25519 canonical-verify question Benten resolved at G16-B-B-rest (`verify_strict` plus the W3C did:key test vectors). The "consider ristretto25519" line is the agility seam.
- **Recommended action**: **COMMENT-NOW** (low-risk; spec-clarity contribution).
- Comment shape (~4 sentences): "We hit the same dalek-Ed25519 ambiguity recently in a content-addressed graph project (Benten Engine), and resolved it by pinning `verify_strict` + adopting the W3C did:key test vectors as conformance fixtures. We'd suggest spec language that mandates the strict canonical form rather than 'one of the verifier methods is fine'; the loose form admits malleable signatures that break content-addressing invariants downstream. Happy to share our test-vector pin if useful (link). On ristretto25519: in our experience the agility seam is more valuable as a *codepoint-dispatched* multikey framing than as a primary algorithm swap — see the multiformats discussion at #381."

### 2.4 willow-rs #138 — Default vs Minima
- URL: https://github.com/earthstar-project/willow-rs/issues/138
- **Recommended action**: **IRRELEVANT** (Rust-trait design hygiene; unrelated to PQ/crypto).

### 2.5 ipfs/specs #448 — New IPNS key types
- URL: https://github.com/ipfs/specs/issues/448
- Opened 2023-10-13, last substantive 2023-10-30, state OPEN
- Body: ianopolous (Peergos) raises PQ / hybrid key types for IPNS, noting 8k-17k signature sizes hit the 10k IPNS record limit. aschmahmann + lidel mapped out the libp2p-key-types-vs-IPNS-divergence options; ianopolous committed to start the IPIP work "when I can get to it." No further activity in 2024-2026.
- **Direct strong relevance to Benten**: this is the same lampps-composite-hybrid problem Benten just decided on (Ed25519⊕ML-DSA-65 ~3.3KB combined signature). The thread has been dormant 2.5+ years.
- **Recommended action**: **COMMENT-NOW**. The thread is genuinely stalled; a Benten experience-report adding 2026 context (NIST-finalized + OpenPGP-PQC draft -17 + Sigstore moving) could legitimately revive it.
- Comment shape (~5 sentences): "Bumping this thread with 2026 context that may help break the chicken-and-egg. Since 2023: NIST finalized FIPS 204 (ML-DSA) Aug-2024; the IETF openpgp-pqc draft (now -17) standardized the composite Ed25519+ML-DSA-65 construction the OP was asking about; Sigstore is moving to dual-sign Rekor v2 checkpoints with the same composite (sigstore/rekor-tiles#425). The signature-size concern (~3.3KB for the composite) still pushes past 10k IPNS record limit only if IPNS records also carry chain context; the bare composite signature fits. We've recently been picking through these same questions in a separate content-addressed graph project (Benten Engine) and ended up at the same hybrid + multicodec-codepoint-dispatch + typed-reject-on-unknown shape that's standardizing across the ecosystem. Happy to share specific impl pointers if useful for the IPIP draft."

### 2.6 ipld/ipld #86 — Mission or Vision Statement?
- **Recommended action**: **IRRELEVANT** (governance, not crypto).

### 2.7 multiformats/multicodec #381 — PGP public keys
- URL: https://github.com/multiformats/multicodec/issues/381
- Opened 2025-05-24, last active 2025-05-30, state OPEN
- Body: Request for a PGP-public-key multicodec entry; rvagg (multiformats maintainer) asks for concrete use-case; silverpill wants did:key over PGP; tesaguri suggests the existing rsa/ed25519/ecdsa codepoints might suffice. rvagg's table-of-existing-entries comment lists what's already minted.
- **Relevance to Benten**: tangential. Benten doesn't need a PGP codec but the *agility framing* question (when does a multikey entry warrant a new codepoint vs reusing an existing curve?) parallels Benten's PQ-hybrid choice. **But**: there's NO multicodec entry for ML-DSA-65, SLH-DSA, X-Wing, or any composite-PQ-hybrid scheme as of this scan. Benten will need one (per the crypto-agility refinement #5: "the suite-selector codepoint table is Benten-owned but component algorithm IDs reference the IANA HPKE/COSE registries"; the codepoint `0x647a` for X-Wing was already reserved in the multicodec table though needs verification).
- **Recommended action**: **COMMENT-NOW** on a *separate* multicodec issue if there's an active PQ-codepoint discussion (none found in scan). On #381 itself: **IRRELEVANT** (don't dilute the PGP-specific request).
- **Follow-up scan** (lower priority): check multiformats/multicodec table for ML-DSA / SLH-DSA / X-Wing entries; if missing, file new issue requesting them.

### 2.8 ucan-wg/spec #187 — Spec clarification on canonicalization for signing
- URL: https://github.com/ucan-wg/spec/issues/187
- Opened ~2025-05, state OPEN. dag-cbor vs dag-json canonicalization ambiguity for UCAN signing.
- **Relevance to Benten**: Benten signs UCAN-shaped capability grants; the canonicalization-shape decision is a small concrete cliff that affects content-addressing.
- **Recommended action**: **WATCH-ONLY**. Benten currently follows the dag-cbor canonical form (per `feedback_plugin_extension_trust_model`); the answer in the thread is already that dag-cbor is canonical. Adding a "+1 from Benten" doesn't move the spec forward.

---

## 3. Sigstore + git-PGP cluster (the L15 OpenPGP-PQC parallel)

### 3.1 sigstore/rekor-tiles #425 — Dual-sign checkpoint with PQC signing algorithm
- URL: https://github.com/sigstore/rekor-tiles/issues/425
- Opened 2025-04 area, last active 2025-09-03, state OPEN
- Body: Hayden Blauzvern (Sigstore core) proposes dual-signing Rekor v2 checkpoints with classical + ML-DSA-65, citing the "if Rekor's signing key is compromised, proofs can be forged" threat. Explicitly designed as "slow transition without requiring all signatures to be PQC."
- **Direct strong relevance to Benten**: this is **exactly the dual-sign reasoning Benten is shipping at v1-beta** (hybrid Ed25519⊕ML-DSA-65 signature default; the audited classical floor underwrites unaudited PQC). The framing is convergent. arthurus-rex's question about algorithm-flexibility-not-hardcoded mirrors Benten's codepoint-dispatch decision.
- **Recommended action**: **COMMENT-NOW**. This is one of the highest-value threads in the entire scan: independent confirmation of the same construction, on a similar threat model (long-lived signatures + adversarial-discovery delay).
- Comment shape (~5 sentences): "Just want to flag a parallel data-point from a content-addressed graph project shipping v1-beta with effectively the same construction. We landed on hybrid Ed25519+ML-DSA-65 (LAMPS composite) as the v1-beta signature default for content-addressed artifacts that are signed today but expected to remain verifiable 10-20 years — same 'signatures-as-discovery-evidence' framing the OP names. The 'unaudited PQC is never the *sole* trust path' invariant (the classical half is the audited floor) is what made us comfortable shipping before the ml-dsa audit lands. Curious whether the Rekor team is tracking the OpenPGP-PQC draft-17 composite construction as the standardization anchor, or considering the recent senior-cryptographer pushback that pure-concatenation composites need a SUF-CMA-preserving combiner (the 'Bird-of-Prey' line of work). For us that's an open question between v1-beta-tag and v1-GM. Happy to share more if useful."

### 3.2 sigstore/sigstore #2129 — Proposal: plugins for post-quantum cryptographic algorithms
- URL: https://github.com/sigstore/sigstore/issues/2129
- Opened ~2025-08, last active 2025-10-12, state OPEN
- Body: Trail of Bits (arthurus-rex) proposes plug-in architecture for OpenSSL + CIRCL PQC algorithms in Sigstore. jas4711 (mid-October comment): "I would prefer hybrid PQ/T signatures first ... ED25519+MLDSA65 is a common combination. SLH-DSA is more conservative choice if you really want PQ-only approach."
- **Direct strong relevance to Benten**: jas4711's comment IS the Benten posture. The plug-in-architecture vs in-tree question is the same as the Benten "one thin Benten-owned integration crate" pattern (CLAUDE.md #5).
- **Recommended action**: **COMMENT-NOW**.
- Comment shape (~4 sentences): "Echoing @jas4711's preference for hybrid PQ/T as default — we're shipping a separate content-addressed graph project (Benten Engine) at v1-beta with Ed25519+ML-DSA-65 as the default signature suite, classical-only and SLH-DSA as opt-in arms, all behind multicodec-codepoint dispatch with typed-reject on unknown (no silent fallback). Two observations from going through this recently: (1) a thin Benten-owned 'integration crate' that wraps vetted upstream primitive crates (RustCrypto-class) gives most of the plug-in flexibility without the bin-distribution complexity OP's proposal carries; (2) the recent senior-cryptographer pushback on naive pure-concatenation composites (need SUF-CMA-preserving combiners, e.g. the 'Bird-of-Prey' line) is worth tracking before locking the construction. Happy to share specific impl pointers if useful."

### 3.3 sigstore/sig-clients #16 — [META] Cryptographic agility for Sigstore clients and services
- URL: https://github.com/sigstore/sig-clients/issues/16
- Opened ~2025-01-21, last active 2025-07-01, state OPEN
- Body: Trail of Bits META tracker for crypto-agility across Sigstore Go clients + services. Tracks Fulcio/Rekor/sigstore-go/cosign sub-issues; jku notes TUF clients are already crypto-agile in theory.
- **Recommended action**: **COMMENT-NOW** but lower-priority than #2129 + rekor-tiles#425 (this META thread is operational tracking, not posture debate).
- Comment shape (~3 sentences): "From a sibling project shipping crypto-agility for the same v1-beta-with-PQ-hybrid-default reasoning: one cliff that bit us was that 'crypto-agility' under-specifies how the *codepoint dispatch* behaves on unknown algorithms — silent fallback (age) vs typed-reject (Veilid/MLS/Nostr-NIP-44). We ended up at typed-reject (fail-closed on unknown), which makes the agility seam predictable but means every supported algorithm must be enumerated at compile time, including the deprecated arms during the transition window. Worth surfacing as an explicit decision-point in the META if Sigstore hasn't already."

### 3.4 sigstore/sigstore #2195 — Standardize Key marshalling
- URL: https://github.com/sigstore/sigstore/issues/2195
- Opened ~2025-11, state OPEN. Trail of Bits POC for PQCA marshalling hit x509-codebase hardcoded-keytypes issues; proposes interface abstraction.
- **Relevance to Benten**: tangential — this is Go-x509-marshalling-specific; Benten is Rust-stack-only and uses RustCrypto.
- **Recommended action**: **IRRELEVANT** for comment.

### 3.5 sigstore/model-transparency #595 — Support PQC algorithm(MLDSA) for signing/verifying Model
- URL: https://github.com/sigstore/model-transparency/issues/595
- Opened ~2025-12, last active 2026-01-22, state OPEN. Hayden-IO (Sigstore collab): "the first question is *why now?*. Rushing adoption of a new algorithm risks adding long-term technical debt and possibly even security risks for algorithms and implementations that have not yet been proven secure."
- **Direct relevance to Benten**: Hayden's "why now?" is the exact reasoning Benten's Position B blog will engage with. ML models are persistent-artifact-signing (Sigstore-side) just like Benten's content-addressed graph nodes — same threat model as Rekor checkpoints in #425.
- **Recommended action**: **COMMENT-NOW** (sibling to #2129; same persistent-signature threat model).
- Comment shape (~4 sentences): "On the 'why now?' question — in a sibling content-addressed graph project (Benten Engine) we landed on PQ-hybrid as v1-beta default specifically because the *signed today, verified 10-20 years* threat model is structurally different from ephemeral-session signing. ML model attestations look like the same threat-model shape (long-lived artifacts; adversarial-discovery delay; can't easily re-sign past artifacts after the algorithm break). The 'classical-half-is-the-audited-floor' invariant of the hybrid construction (per OpenPGP-PQC draft-17 / Sigstore rekor-tiles#425) is what makes shipping before the independent ML-DSA audit reasonable rather than reckless. We're shipping the audit as a gate between v1-beta and v1-GM rather than blocking initial v1-beta on it — happy to share that framing if useful."

### 3.6 sigstore/architecture-docs #48 — Rekor V2 updates
- URL: https://github.com/sigstore/architecture-docs/issues/48
- **Recommended action**: **WATCH-ONLY**. Rekor V2 client-spec checklist; not a venue for Benten input.

### 3.7 sigstore/cosign — no open PQ-tagged issues
- Direct scan of sigstore/cosign returned 0 open PQ issues (work is happening through sigstore/sigstore and sigstore/rekor-tiles upstream).
- **Recommended action**: **IRRELEVANT** for this scan.

### 3.8 openpgp-pqc/draft-openpgp-pqc — 0 open issues
- URL: https://github.com/openpgp-pqc/draft-openpgp-pqc
- Body: the draft repo for IETF openpgp-pqc-17 has 0 open issues at scan time (verified). Work happens on the IETF openpgp@ietf.org mailing list.
- **Recommended action**: **IRRELEVANT** (no GitHub venue for comment); **WATCH-ONLY** the repo for new issues / RFC ratification milestone.

### 3.9 SPHINCS+ / Josefsson git-PGP — no GitHub thread surfaced
- Josefsson Dec-2024 blog demonstrates git signing with SPHINCSPLUS via OpenSSH; no associated open RFC-thread / GitHub-issue tracker surfaced in scan beyond OpenSSH-side discussion.
- **Recommended action**: **WATCH-ONLY**. Relevant context for Benten's "Phase-N consider SLH-DSA as the non-default conservative-PQ-only arm" decision; not currently actionable.

---

## 4. libp2p / W3C / Filecoin / Arweave / Ceramic / Veilid

### 4.1 libp2p/rust-libp2p #6236 — Post-Quantum Key Exchange
- URL: https://github.com/libp2p/rust-libp2p/issues/6236
- Opened ~2025-12-29, state OPEN. OP asks for Kyber768 (ML-KEM) hybrid with X25519 via Noise; drHuangMHT (libp2p contrib): "All libp2p implementations shall follow the specs, so you may open an issue there."
- **Relevance to Benten**: tangential — this is libp2p-KEX (transport encryption) not signature scheme. But the deflection to libp2p/specs hasn't been actioned (no open libp2p/specs PQ issue surfaced in scan).
- **Recommended action**: **WATCH-ONLY**. Benten doesn't use libp2p directly (iroh is the transport stack); commenting risks scope-bleed. If a libp2p/specs PQ-KEX issue lands, that's the better venue.

### 4.2 libp2p/specs #696 — Use of hedged signatures
- URL: https://github.com/libp2p/specs/issues/696
- Opened 2025-09-03, last active 2025-09-08, state OPEN. MarcoPolo (libp2p maintainer): "From libp2p's perspective we should probably wait for the actual cryptographers at the CFRG to publish a recommendation."
- **Relevance to Benten**: tangential. Hedged signatures (deterministic-sig-with-noise) are an Ed25519-side defense-in-depth, not the PQ-hybrid posture. Worth tracking if the CFRG draft-irtf-cfrg-det-sigs-with-noise progresses but Benten's PQ-hybrid covers a different threat surface.
- **Recommended action**: **IRRELEVANT** for the Benten PQ-default campaign.

### 4.3 libp2p/specs #683 — WebTransport non-deterministic certs for FIPS
- URL: https://github.com/libp2p/specs/issues/683
- **Recommended action**: **IRRELEVANT** (FIPS-deterministic-key edge-case; not Benten posture).

### 4.4 w3c/vc-data-integrity #338 — Crypto layering — hybrid PQC and set vs chain signatures
- URL: https://github.com/w3c/vc-data-integrity/issues/338
- Opened ~2025-05-01, last active 2025-05-02, state OPEN. SING (W3C Security Interest Group) review notes that the VC-Data-Integrity §5.4 "agility and layering" section should better explain hybrid-PQC posture + the distinction between *chain* signatures (signature-of-signature) and *set* signatures (two separate sigs over same data). brentzundel (VC editor): "We are grateful for this response from SING and look forward to considering it as part of a future version of the specification."
- **Direct strong relevance to Benten**: this is **structurally the same question** Benten faces at the v1-beta hybrid-default-vs-Bird-of-Prey-combiner decision point. The set-vs-chain distinction *is* the SUF-CMA-preserving-combiner question (set-signatures with naive concat = the construction senior cryptographers are pushing back on; chain/composite-with-binding = the "right" shape).
- **Recommended action**: **COMMENT-NOW**. The thread is dormant but the issue is officially open; SING explicitly asked for someone to drive the explainer language. Benten experience-report is directly useful input.
- Comment shape (~5 sentences): "Surfacing a sibling-project data-point that may help the explainer language. In a content-addressed graph project (Benten Engine) shipping v1-beta with PQ-hybrid signatures as default, we hit the set-vs-chain distinction concretely: the IETF openpgp-pqc draft-17 composite Ed25519+ML-DSA-65 + Sigstore Rekor v2 dual-sign (sigstore/rekor-tiles#425) both standardize on the *set* shape (two separate sigs over same data). The recent senior-cryptographer pushback (LAMPS composite concerns; need for SUF-CMA-preserving combiners — the 'Bird-of-Prey' line) argues the naive set shape isn't strip-resistant and pushes toward a *committing* construction that has chain-signature semantics even though wire-format-wise it looks like a set. The decision question for spec writers IMO is: does VC-DI want to leave the set-vs-chain choice to the cryptosuite, or mandate the strip-resistant variant? We landed at 'mandate' (typed-reject on unknown crypto means downgrade-attacks fail closed). Happy to share the concrete construction we ended up at."

### 4.5 ipfs/specs #448 — covered in §2.5 above (cross-listed; primary mention)

### 4.6 Filecoin: no open PQ-related FIPs surfaced
- Filecoin properly excluded per L15: signs blockchain consensus rounds, not persistent artifacts. Scan confirms no PQ-tagged FIPs open.
- **Recommended action**: **IRRELEVANT** (correctly out of scope).

### 4.7 Arweave: only BLS12 verification request (#324)
- URL: https://github.com/ArweaveTeam/arweave/issues/324
- Opened 2021-09-27, state OPEN, no comments since open. Single-line "verify BLS12 sigs from smart contract" request.
- **Relevance to Benten**: NIL. Arweave is the cautionary example (RSA-PSS-locked, ~5 PB unmigrable per L15) but commenting "PS we shipped PQ-hybrid" on a 4-year-old BLS-verify request is off-topic.
- **Recommended action**: **IRRELEVANT**.

### 4.8 Ceramic CIPs — no open PQ issues surfaced
- Scan returned only 2 open CIPs (StreamType:DIDPublish + Service Definition), neither crypto-posture.
- **Recommended action**: **IRRELEVANT**.

### 4.9 Veilid — repo is on GitLab, not GitHub
- Out of GitHub-scan scope. Per WebSearch, no public PQ-discussion surfaced. The CLAUDE.md memory entry already cites Veilid as a typed-reject precedent for codepoint-dispatch.
- **Recommended action**: **IRRELEVANT** for this GitHub-scan campaign; **WATCH-ONLY** Veilid release notes for any PQ-posture announcement.

### 4.10 Aljoscha Meyer / worm-blossom willow_rs (Codeberg, not GitHub)
- The Codeberg fork (`codeberg.org/worm-blossom/willow_rs`) is the spec-faithful reference impl. No PQ-tagged issues surfaced via web search.
- **Recommended action**: **IRRELEVANT** for GitHub-scan; would need a separate Codeberg-direct scan if desired.

---

## 5. Other / Adjacent

### 5.1 UCAN-wg #139 — Should we support non DID principals?
- URL: https://github.com/ucan-wg/spec/issues/139
- Opened 2023-01, dormant since. Discussion of UCAN principals as arbitrary URIs (mailto: / acct:) rather than only `did:*`.
- **Relevance to Benten**: tangential to crypto-posture; relevant to Benten's principal-primitive (CLAUDE.md #18) but the thread is 3 years dormant and Benten's `system:Principal` shape diverges from the spec's `iss/aud` URI framing in ways that would need a much longer post to engage productively.
- **Recommended action**: **IRRELEVANT** for the PQ-default campaign; **WATCH-ONLY** if Benten ever needs to upstream the Principal framing.

### 5.2 ChainAgnostic CAIPs (varsig / CACAO etc.)
- A few open issues touch on `varsig` confusion (#265) but nothing PQ-specific. No actionable comment opportunity.
- **Recommended action**: **IRRELEVANT**.

---

## 6. Cluster pattern observations

1. **The OpenPGP-PQC composite is the de facto standard converging across orgs.** The Ed25519+ML-DSA-65 LAMPS composite — exactly Benten's v1-beta default — is independently being adopted by Sigstore (Rekor v2 checkpoints), proposed for IPNS (#448 stalled but the technical convergence is there), recommended in sigstore/sigstore #2129, the explicit topic of openpgp-pqc draft-17, and is the *implicit* shape behind W3C VC-DI #338's "set signatures over same data." Benten's choice is not an outlier — it's the ecosystem-wide convergent shape.

2. **The Bird-of-Prey / SUF-CMA-preserving combiner concern is NOT yet visible in public threads.** The senior-cryptographer pushback against pure-concatenation composites surfaced in academic / IETF-mailing-list venues; no GitHub-side thread in the scanned orgs is actively debating it. **If Benten switches to a SUF-CMA-preserving combiner after senior-cryptographer review, that is *novel public material* that any of #338, #2129, rekor-tiles#425 would welcome** — same construction question, no one currently driving it in those venues.

3. **iroh is the deliberate outlier on PQ-signatures.** iroh's stated position (FAQ + closed issues #4147 + #4191 + matheus23's #4195 comment) is: PQ-KEX yes (because HNDL on session-encryption is real), PQ-signatures no (their threat-model framing collapses to "if classical sigs were broken, the system is broken anyway"). Benten's Position B is the structural counter-argument: the threat model for content-addressed-at-rest-artifacts is *different* from the threat model for ephemeral connections. **This is exactly the gap the Shape-5 private outreach + the Position B blog are designed to bridge.** No other org in the scan shares iroh's PQ-sig-skeptical stance — everyone else is moving toward hybrid.

4. **"Why now?" is the recurring counter-argument** (Hayden-IO on model-transparency #595; matheus23 implicitly on iroh #4195; libp2p's "wait for CFRG" on hedged sigs #696). Benten's response is unified: the answer is the asymmetric threat model — late adoption is much worse than early adoption on persistent-artifact signing, and the hybrid construction means early adoption costs only the audit gap (covered by the v1-beta → v1-GM tag split).

5. **The crypto-agility framing is the universal cliff.** Every active thread surfaces the same sub-question: should the construction be hardcoded or codepoint-dispatched, and what happens on unknown codepoints? Benten's typed-reject choice (with the Veilid/MLS/Nostr-NIP-44 precedent + age as the rejected silent-fallback outlier) is mature material that several threads (sig-clients #16, sigstore #2129, VC-DI #338) would benefit from.

6. **The dormant-IPNS-thread (ipfs/specs#448) is a real reviv-able opportunity.** It's 2.5 years stalled; the technical objections of 2023 (signature size, lack-of-standard) have ALL been resolved in 2024-2026 (NIST finalization + draft-17 standardization + Sigstore moving). A Benten experience-report could genuinely move it forward, and the IPNS-record-size question is a useful design-cliff to enumerate.

7. **iroh-blobs threads are uniformly NOT PQ-touching.** The active iroh-blobs work is store-refactor / wasm / API-rename — no signing or content-addressing-CID-scheme work in flight. Benten's blob-layer dependency is on the wasm32 bundle (#90) rather than any PQ-relevant surface.

8. **Most actionable threads for Benten** (priority-ordered):
   - **#1 sigstore/rekor-tiles#425** (dual-sign checkpoint with PQC) — perfect construction-match; high-value comment
   - **#2 sigstore/sigstore#2129** (PQC plugins proposal; jas4711 already echoes Benten posture) — high-value comment
   - **#3 w3c/vc-data-integrity#338** (set-vs-chain composite; dormant + open invitation from SING) — high-value comment
   - **#4 sigstore/model-transparency#595** (PQC for model signing; "why now?" debate) — high-value comment
   - **#5 ipfs/specs#448** (revive stalled IPNS-PQ-key-types thread) — high-value comment if appetite for reviving 2.5-year-dormant thread
   - **#6 willowprotocol.org#175** (canonic Ed25519 + ristretto consideration) — low-risk spec-clarity contribution
   - **#7 sigstore/sig-clients#16** (META crypto-agility tracker) — lower-priority + operational
   - **All iroh threads — WATCH-ONLY-PENDING-SHAPE-5** per the brief's explicit ordering

9. **Risks to manage on COMMENT-NOW threads:**
   - Don't claim Benten is "shipped" — say "shipping v1-beta with" / "decided" / "landed on" (we're pre-tag).
   - Don't make absolute Bird-of-Prey claims pending senior-cryptographer review — flag as "open question between v1-beta-tag and v1-GM."
   - Always include the "happy to share specific pointers if useful" off-ramp so the comment is a hook for follow-up, not a megapost.
   - The Position B argument (content-addressed-persistence vs ephemeral-session) is *load-bearing for iroh* but is *just one motivation among several* for Sigstore/IPNS/VC-DI — keep it implicit on those threads, not central.
   - The audit-gate framing (v1-beta-with-unaudited-hybrid → v1-GM-after-audit) is uniquely Benten's; others may not have the GM-tag-with-audit-gate shape. Frame as "our specific choice was X" not "you should also do X."

---

## Scan footprint summary

- **Orgs scanned**: n0-computer (iroh + noq + iroh-blobs + iroh-gossip + iroh-docs + sendme), earthstar-project (willow-rs + willowprotocol.org), worm-blossom on Codeberg (web-searched only), ipfs/specs, ipld/ipld, multiformats/multicodec, ucan-wg/spec, libp2p/specs + libp2p/rust-libp2p, sigstore/sigstore + sigstore/cosign + sigstore/rekor-tiles + sigstore/architecture-docs + sigstore/sig-clients + sigstore/model-transparency, openpgp-pqc/draft-openpgp-pqc, w3c/vc-data-integrity, w3c-ccg/did-method-key, ChainAgnostic/CAIPs, ArweaveTeam/arweave, ceramicnetwork/CIPs, filecoin-project/FIPs + filecoin-project/lotus.
- **Threads triaged**: ~50.
- **COMMENT-NOW threads**: 6 (rekor-tiles#425, sigstore#2129, vc-data-integrity#338, model-transparency#595, ipfs/specs#448, willowprotocol#175).
- **COMMENT-NOW lower-priority**: 1 (sig-clients#16).
- **WATCH-ONLY-PENDING-SHAPE-5 (iroh-cluster)**: 3 (iroh#4195, iroh#3849, iroh#2355).
- **WATCH-ONLY**: noq#463, willow-rs#131/#112, ucan-wg#187, sigstore-architecture#48, ucan-wg#139.
- **IRRELEVANT** for this campaign: the remainder.
