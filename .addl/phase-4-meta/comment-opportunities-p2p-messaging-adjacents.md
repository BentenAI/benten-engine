# Comment Opportunities — P2P + Secure Messaging + Decentralized Protocol Adjacents

**Scope:** P2P + messaging + decentralized-protocol ecosystems (Signal/libsignal, MLS implementations, Nostr/NIPs, Veilid, ATProto, Holepunch/hypercore, Matrix, Briar, SimpleX, Session, SSB/Earthstar/Willow, CRDT-sync libs, related W3C SWICG ActivityPub-E2EE work).

**Date of scan:** 2026-05-26.
**Scope discipline:** READ-ONLY scan. Cite-anchored. ZERO suggestion that Signal/MLS/iMessage chose wrong. Different system class → different defaults → both defensible.

**Bottom-line up front:**
- **Strong cohort evidence Benten is NOT alone shipping/planning PQ-hybrid signatures-by-default for a persistence-flavored P2P system.** OpenMLS, ActivityPub-E2EE (SWICG), Session, Wire (MLS adopter), and Bitcoin/Ethereum hybrid-sig research all converge on hybrid construction for non-ephemeral surfaces.
- **The clean philosophical split between "ephemeral session" (Signal/PQXDH) and "persistent artifact" (Benten) is visible in the public record** — PQXDH §4.8 + Signal's stale issue #465 vs Nostr's NIP-44 #1971, the ML-DSA-87 ciphersuite request to OpenMLS, NIP-101 PR #391 disposition, and Cloudflare's "trust now, forge later" framing. This is exactly the blog-post line Benten wants to draw.
- **There are 4-5 legitimately commentable threads** where Benten can helpfully *add evidence* (cohort + content-addressed persistence framing) without criticizing anyone. Most other threads are WATCH-ONLY.
- **Top COMMENT-NOW targets:** Nostr NIP-44 #1971 (paulmillr, very alive) + OpenMLS #1940 ML-DSA-87 ciphersuite (soatok, very alive) + ATProto discussion #3928 (PLC verification-method relaxation — natural opening for PQ key-type registration request) + Matrix #975 (long-standing, low-traffic, value comes from cohort evidence not pressure).
- **Top WATCH-ONLY (do NOT comment) — Signal lane:** libsignal #465 (closed/stale), anything PQXDH-adjacent. These intersect Signal's intentional design choices.

---

## 1. P2P + decentralized systems shipping or planning PQ-hybrid: cohort evidence

This section enumerates systems Benten can cite as "we're not alone" in a future blog post. NOT public-comment targets at this stage — these are private-outreach + Shape-5 coalition candidates later.

### 1.1 OpenMLS — `MLS_256_MLKEM1024_AES256GCM_SHA512_MLDSA87` ciphersuite

- **Cite:** [openmls/openmls#1940](https://github.com/openmls/openmls/issues/1940)
- **Opened:** 2026-01-27 by soatok. Open. Recent.
- **Status:** Requests adding two ciphersuites: `MLS_256_XWING_CHACHA20POLY1305_SHA512_Ed25519` AND `MLS_256_MLKEM1024_AES256GCM_SHA512_MLDSA87`. Notes that ML-KEM + ML-DSA landed in RustCrypto in 2025, enabling impl via RustCrypto rather than libcrux. Interop testing planned against a TypeScript MLS impl.
- **Relevance to Benten:** Strong cohort evidence. The MLDSA87 ciphersuite is exactly the hybrid-signature posture Benten is taking — ML-DSA-87 is the FIPS 204 maximum-security parameter set, comparable to what Benten ships. The fact that an MLS implementation (which is ostensibly an ephemeral-session system) is also adding ML-DSA-based identity-signature ciphersuites *weakens* the "only persistence systems need PQ identity sigs" claim — and that's good news for Benten, because it means Benten's choice is one of several legitimate readings of the same threat landscape.
- **Recommended action:** WATCH-ONLY. The thread is currently a maintainer-internal feature request and not a venue for cross-protocol commentary. Reference it from Benten's blog post as cohort evidence.
- **Cross-link:** The same issue cites `swicg/activitypub-e2ee` as coordinating on the same standards stack — direct evidence of a Shape-5 coalition forming organically.

### 1.2 Session — Protocol V2 with PQC (ML-KEM)

- **Cite:** [getsession.org/blog/session-protocol-v2](https://getsession.org/blog/session-protocol-v2) + [cyberinsider.com 2026 coverage](https://cyberinsider.com/session-starts-development-of-quantum-secure-messaging-protocol/)
- **Status:** Active design phase 2026; detailed spec expected in 2026 per Session blog. Will integrate ML-KEM. Open dev process.
- **Relevance to Benten:** Cohort evidence (P2P-ish, privacy-focused). Note: Session as far as public reporting indicates is currently adopting ML-KEM (encryption) NOT yet ML-DSA (signatures). This is the same "encryption first, signatures later" pattern as Signal/iMessage/WhatsApp — important to acknowledge in any future framing so we don't overclaim Session as a sig-hybrid cohort member.
- **Recommended action:** WATCH-ONLY. Watch for the spec release for whether sigs make it into V2.

### 1.3 SimpleX Chat — quantum-resistant double ratchet (KEM, not sigs)

- **Cite:** [simplex.chat/blog 20240314](https://simplex.chat/blog/20240314-simplex-chat-v5-6-quantum-resistance-signal-double-ratchet-algorithm.html)
- **Status:** Shipped PQ-resistant double ratchet (sntrup761 KEM) v5.6 beta March 2024; default for direct chats planned v5.7. Explicitly chose "augment rather than replace" and "avoid signatures when new keys are agreed" — actively de-emphasizes signatures in their PQ posture.
- **Relevance to Benten:** Negative cohort evidence on the signature front. SimpleX explicitly *avoids* signatures in PQ design, opposite Benten's posture. Useful in the blog post as "the design space has multiple sensible answers and the choice depends on what the system retains long-term."
- **Recommended action:** WATCH-ONLY. Do not comment.

### 1.4 Wire — MLS in production, "post-quantum ready" via MLS agility

- **Cite:** [wire.com/en/messaging-layer-security](https://wire.com/en/messaging-layer-security) + [thestack.technology coverage](https://www.thestack.technology/messaging-layer-security-is-coming-of-age/)
- **Status:** First messenger to ship MLS in production. Public posture is "MLS gives us cipher-suite agility so we can deploy PQ as IETF ciphersuites are registered." Currently waiting on the IETF PQ-MLS draft (`draft-ietf-mls-pq-ciphersuites-04`) to firm up.
- **Relevance to Benten:** Cohort evidence for "agility-first" posture. Wire didn't ship PQ as a hardcoded default — they shipped MLS *because* it's cipher-agile, leaving room to swap defaults later. Same architectural impulse as Benten's codepoint-dispatched suite.
- **Recommended action:** WATCH-ONLY. Wire is a customer of MLS, not a venue for our commentary.

### 1.5 SWICG ActivityPub-E2EE — MLS-based, X-Wing KEM, Ed25519 sigs (deferred PQ sigs)

- **Cite:** [swicg/activitypub-e2ee](https://github.com/swicg/activitypub-e2ee) + [soatok.blog 2024-09-13 update](https://soatok.blog/2024/09/13/e2ee-for-the-fediverse-update-were-going-post-quantum/) + [swicg.github.io/activitypub-e2ee/mls](https://swicg.github.io/activitypub-e2ee/mls)
- **Status:** Active 2026 (open issues #77-#89 all April-May 2026). Has chosen X-Wing (X25519 + ML-KEM-768 hybrid KEM) BUT explicitly *deferred* post-quantum signatures, citing "hybrid signature constructions may be problematic in subtle ways" and "no immediate threat exists."
- **Relevance to Benten:** *Highly relevant* — this is a closely-adjacent ecosystem (federated social, content-flavored persistence) that took exactly the opposite signature choice. Soatok's reasoning is honest and well-articulated. In a future blog post, this is the most important counter-position to engage with respectfully: *they* chose ephemeral-flavored framing for ActivityPub DMs (ratcheting + reportable abuse), *we* chose content-addressed persistence framing. Both defensible; the framing difference IS the substance.
- **Recommended action:** WATCH-ONLY for now. There IS a possible COMMENT-AFTER opportunity here on a future PQ-signatures issue (issue #35 Key Transparency is the closest existing thread), but the right time is after Benten's blog post is published — public-comment-before-blog is awkward when the framing distinction is the load-bearing point. Defer to post-blog Shape-5 outreach instead.

### 1.6 Keyhive (Ink and Switch) — BeeKEM (PCS + FS, classical sigs)

- **Cite:** [inkandswitch.com/keyhive/notebook](https://www.inkandswitch.com/keyhive/notebook/) + [github.com/inkandswitch/keyhive](https://github.com/inkandswitch/keyhive)
- **Status:** Active research-grade CGKA library for local-first apps. Provides forward secrecy + post-compromise security. Currently uses classical signatures (no PQ in published cryptanalysis).
- **Relevance to Benten:** Strong philosophical adjacent — Keyhive's threat model (multi-device local-first, untrusted-host capable) is closer to Benten's than Signal's. They are NOT yet PQ-default but the design space is the same one Benten lives in. Excellent Shape-5 candidate.
- **Recommended action:** WATCH-ONLY. Possible private outreach later; Ink and Switch has historically been receptive to design dialogue.

### 1.7 Bitcoin / Ethereum — hybrid PQ-signature research lane

- **Cite:** [preprints.org/manuscript/202509.2079](https://www.preprints.org/manuscript/202509.2079) (Hybrid Post-Quantum Signatures for Bitcoin and Ethereum) + [github.com/ethereum/pm#2035](https://github.com/ethereum/pm/issues/2035) (PQ Interop #37, April 2026)
- **Status:** Active research; testnet measurements show 52-57% throughput cost on permissioned implementations. Ethereum has a recurring PQ Interop call series.
- **Relevance to Benten:** Cohort evidence for "content-addressed long-term-persistence systems need hybrid sigs." Bitcoin/Ethereum literally cannot do ephemeral-session framing — every signed artifact is permanent on-chain. Strongly supports Benten's persistence-flavored framing in a future blog post.
- **Recommended action:** WATCH-ONLY for now. Blockchain ecosystem comments would be a separate sub-scan; cited here for cohort weight only.

### 1.8 SSH (GitHub-deployed) — sntrup761x25519 hybrid shipped

- **Cite:** [github.blog post-quantum security for SSH](https://github.blog/engineering/platform-security/post-quantum-security-for-ssh-access-on-github/)
- **Status:** Shipped 2023+ as a default-on hybrid KEM. NOT signatures, NOT P2P, NOT decentralized — but evidence that hybrid-PQ-default is a production-tested pattern in adjacent infrastructure.
- **Recommended action:** WATCH-ONLY; mention in cohort framing if needed.

---

## 2. Signal + MLS lane — mostly WATCH-ONLY

Per scope brief: default WATCH-ONLY unless very clearly content-addressed-persistence-tangent. The PQXDH §4.8 deniability-grounds finding (from L8) is *internal blog framing* — NOT public comment material.

### 2.1 libsignal #465 — Post-Quantum Cryptography

- **Cite:** [signalapp/libsignal#465](https://github.com/signalapp/libsignal/issues/465) — Qata 2022-06-23
- **State:** **CLOSED, stale label.** Signal did not engage; superseded by PQXDH announcement + spec publication.
- **Recommended action:** **IRRELEVANT for commentary.** Closed and stale. Any post-hoc comment would be unwelcome.

### 2.2 PQXDH ecosystem (signal.org/docs/specifications/pqxdh + Inria-Prosecco/pqxdh-analysis + Cryspen analysis)

- **Cite:** [signal.org/docs/specifications/pqxdh](https://signal.org/docs/specifications/pqxdh/) + [Cryspen analysis](https://cryspen.com/post/pqxdh/) + [Inria-Prosecco/pqxdh-analysis](https://github.com/Inria-Prosecco/pqxdh-analysis)
- **State:** Active formal-analysis ecosystem. §4.8 is the documented deniability rationale for *not* PQ-ing identity sigs. Cryspen + Inria flagged a public-key encoding confusion attack that produced PQXDH v2; further breaking changes contemplated.
- **Recommended action:** **WATCH-ONLY, do NOT comment.** This is Signal's intentional design choice and the public conversation is fully cryptographer-internal. Benten's frame (different system class, different choice) is for *Benten's own blog* — making it on Signal's turf would land as "we think your reasoning was wrong," which is exactly what we want to avoid.
- **Internal note:** L8's PQXDH §4.8 deniability finding is the load-bearing point for Benten's blog. Signal/PQXDH is the *prototypical ephemeral-session protocol* in the persistence-vs-ephemeral dichotomy. Use as the natural counterpoint, framed as "Signal's design goal of strong deniability + ephemeral sessions correctly leads them to defer PQ identity sigs; Benten's design goal of content-addressed persistence + non-repudiable artifacts correctly leads us to ship PQ identity sigs at v1-beta. Both are defensible reads of the threat landscape."

### 2.3 IETF LAMPS PQ Composite Sigs (`draft-ietf-lamps-pq-composite-sigs-18`)

- **Cite:** [datatracker.ietf.org draft-ietf-lamps-pq-composite-sigs](https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/)
- **State:** Active draft v18 (April 2026), in Last Call. Defines composite ML-DSA + traditional sigs (RSA, ECDSA, Ed25519, Ed448) for X.509 PKI. Benten currently aligns to this contract for the Composite ML-DSA wire format.
- **Recommended action:** **WATCH-ONLY.** The LAMPS WG mailing list (lamps@ietf.org) is the venue for this draft; Benten lurking + tracking version diffs is sufficient. If the rumored Bird-of-Prey-class SUF-CMA-preserving combiner switch happens, *that* is a separate IETF engagement (CFRG-adjacent).
- **Cross-link:** [draft-ietf-lamps-cms-composite-sigs-04](https://datatracker.ietf.org/doc/draft-ietf-lamps-cms-composite-sigs/) for the CMS variant.

### 2.4 IETF MLS PQ ciphersuites (`draft-ietf-mls-pq-ciphersuites-04`)

- **Cite:** [datatracker.ietf.org draft-ietf-mls-pq-ciphersuites](https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/)
- **State:** Active draft. Defines ML-KEM + (traditional OR ML-DSA) ciphersuites; explicitly addresses HNDL on key-exchange without forcing PQ-signature cost.
- **Relevance:** Cohort evidence + the textual basis for OpenMLS #1940. Notice the draft *itself* offers both variants (traditional sig OR ML-DSA sig) — IETF MLS WG is hedging on the same axis Benten is opinionated about.
- **Recommended action:** **WATCH-ONLY** on the mls@ietf.org list.

---

## 3. Nostr + ATProto + Matrix — primary commentable surface

### 3.1 Nostr NIP-44 #1971 — post-quantum security  ← **TOP COMMENT-NOW CANDIDATE**

- **Cite:** [nostr-protocol/nips#1971](https://github.com/nostr-protocol/nips/issues/1971)
- **Opened:** 2025-07-10 by paulmillr (highly respected JS crypto author of noble-*). Open.
- **What's discussed:** Hybrid ECDH + KEM for NIP-44 encryption. Acknowledges secp256k1 compromise would break Nostr's whole arch (events are signed with secp256k1). Surfaces the secret-key-distribution problem.
- **What's NOT discussed:** ML-DSA or post-quantum *signatures* (only encryption). The long-term-persistence-vs-ephemeral framing is also not surfaced explicitly, but Nostr is by design long-term-persistence (events are append-only public artifacts replicated across relays).
- **Why Benten can helpfully comment:** Nostr's design is the closest peer to Benten's design that exists in the wild — content-addressed-ish (event-ID = SHA-256 of canonical JSON), append-only across relays, identity-keyed signatures on every event. The "what about signatures?" question is *not yet on the table* in this thread but follows naturally. Benten can helpfully share: (a) we did the same gap analysis and concluded both KEM-hybrid AND sig-hybrid were necessary for an append-only-artifact system; (b) here are the concrete tradeoffs we hit (LAMPS Composite ML-DSA wire format, ~5KB sig size, suite-agility seam); (c) we're NOT recommending Nostr do the same — different community, different velocity — but here's our data.
- **Comment shape (~4 sentences):**
  > "Adjacent data-point from a content-addressed graph engine we're shipping for v1-beta with PQ-hybrid identity signatures (LAMPS Composite ML-DSA + Ed25519): we found the encryption-vs-signature split actually maps to a *system-class* split rather than a *threat-timeline* split. For append-only artifact systems (Nostr events, our content-addressed graph nodes, blockchains), a future identity-key forgery is permanently-bad in a way it isn't for ephemeral sessions (Signal/MLS), because the artifact persists. Wire-format cost is ~5KB per signature, which is acceptable for our use case but might not be for a high-volume relay protocol — happy to share concrete benchmarks if useful. (Not arguing Nostr should make the same choice — just adding evidence to the design-space discussion.)"
- **Risks:** Low. paulmillr is technical, audience is sympathetic to crypto-agility framing. The "not recommending" framing keeps it humble. The "different system class" framing also tees up the future blog post without pre-empting it.
- **Timing:** COMMENT-NOW (or COMMENT-AFTER-tag-v1-beta to anchor on real shipping, if Ben prefers waiting for the artifact).

### 3.2 Nostr NIP-101 PR #391 (CLOSED, not merged)

- **Cite:** [nostr-protocol/nips#391](https://github.com/nostr-protocol/nips/pull/391)
- **State:** Closed 2025-05-22, not merged. Author eznix86. mikedilger commented "post-quantum signature protection isn't currently necessary, only encryption-related harvest attacks pose near-term risks" — the Nostr-maintainer current public posture.
- **Relevance:** Documents Nostr's *current* stance that PQ sigs aren't urgent. Useful internal-blog-framing context. Benten's comment on #1971 (above) implicitly engages with this stance respectfully.
- **Recommended action:** **WATCH-ONLY.** Don't reopen a closed PR. Reference in the #1971 comment if natural.

### 3.3 ATProto Bluesky discussion #3928 — PLC verification-method relaxation  ← **STRONG COMMENT-NOW CANDIDATE**

- **Cite:** [bluesky-social/atproto#3928](https://github.com/bluesky-social/atproto/discussions/3928)
- **Opener:** DavidBuchanan314 (atproto team) on 2025-06-05. Open with one follow-up.
- **What's discussed:** PLC Directory now permits "verificationMethod" keys to be almost any `did:key:` type (any multicodec-encoded key). BUT rotation keys + repo authentication are STILL constrained to `p256` and `k256`.
- **Why Benten can helpfully comment:** The relaxation explicitly opens the door for non-Ed25519 verification methods. PQ key types (ML-DSA via `did:key:` multicodec) are a natural extension. The constraint-on-rotation-keys side is *also* exactly where Benten lives — content-addressed identity rotation under PQ is the same design problem. Benten can helpfully share what multicodec codepoints we picked (we're using IANA-registered codepoints rather than minting our own per CLAUDE.md #5 crypto-agility refinement) and note that ML-DSA-65 + ML-DSA-87 have multicodec assignments available.
- **Comment shape (~3-4 sentences):**
  > "Helpful context from an adjacent content-addressed system that ships with `did:key` + multicodec-dispatched keys: ML-DSA-65 and ML-DSA-87 have multicodec assignments (0x1207 and 0x1208) available and shipping in libraries like noble-post-quantum and RustCrypto. We landed on enforcing codepoint-dispatch with a typed-reject arm on unknown algorithms (P2P-mainstream choice per MLS/Nostr-NIP-44; age's silent-ignore was the dishonest outlier) — works well with the `did:key:` opaque-multibase shape. The rotation-key constraint to p256/k256 is the same fork we face for our rotation log; we ended up adopting Composite ML-DSA (LAMPS draft-18) for hybrid rotation signatures since rotation events are the most safety-critical persistent artifact in the identity system. Different threat model than atproto (we're peers-hold-ciphertext rather than indexer-relayed), but the multicodec registration shape is reusable. Not advocating PLC adopt PQ rotation keys yet — just flagging the IETF + multicodec state is now ready."
- **Risks:** Low-to-moderate. The atproto team is technical and friendly to design dialogue. The "not advocating" framing is the load-bearing risk-management piece — atproto rotation keys are a deeply load-bearing surface in their architecture and we explicitly don't want to be seen as advocating a breaking change.
- **Timing:** COMMENT-NOW. Discussion is fresh and the relaxation IS the natural opening.

### 3.4 Matrix-spec #975 — Quantum-resistant crypto

- **Cite:** [matrix-org/matrix-spec#975](https://github.com/matrix-org/matrix-spec/issues/975)
- **Opened:** 2022-01-26 by turt2live. Open, low-traffic.
- **State:** Placeholder issue. References Wire's PQ work but no substantive technical discussion. Opener explicitly says "I have no idea what this entails."
- **Why Benten can helpfully comment:** The issue has been sitting open for 4+ years. A *data-pointer* comment from a system that's actually shipping PQ-hybrid sigs would be welcome ballast. Specifically the IETF state (draft-ietf-mls-pq-ciphersuites-04, draft-ietf-lamps-pq-composite-sigs-18) + OpenMLS #1940 + the X-Wing KEM standardization status are all things a Matrix maintainer would want to know about.
- **Comment shape (~4-5 sentences):**
  > "Adjacent 2026 data-pointer from a P2P content-addressed graph engine preparing to ship PQ-hybrid identity sigs (LAMPS Composite ML-DSA + Ed25519) for v1-beta. The relevant standards are now near-shippable: draft-ietf-mls-pq-ciphersuites-04 defines `MLS_256_MLKEM1024_AES256GCM_SHA512_MLDSA87` plus an Ed25519 variant; draft-ietf-lamps-pq-composite-sigs is in Last Call (April 2026); X-Wing KEM draft (draft-connolly-cfrg-xwing-kem-10) standardized the hybrid combiner; ML-KEM/ML-DSA shipped in RustCrypto 2025. Matrix's MLS-migration discussion (e.g. MSC4153 + the inlined Olm/Megolm spec) could plug into the same ciphersuite tree. We took a 'PQ-hybrid is the v1-beta default + audited classical floor remains the trust anchor until independent PQ audit lands at v1-GM' posture; happy to share concrete numbers (key sizes, sig sizes, perf) if useful. Different system class than Matrix (we're long-term-persistent content-addressed artifacts vs Matrix ratcheted messages), but the cipher-agility seam is reusable."
- **Risks:** Low. Old placeholder issue, Matrix maintainers are friendly, and the comment is value-add (concrete current-2026 state-of-the-art digest), not advocacy.
- **Timing:** COMMENT-NOW (any time — the issue isn't on a critical path so timing isn't urgent).

### 3.5 ATProto discussion #3366 — `did:plc` rotation key management

- **Cite:** [bluesky-social/atproto#3366](https://github.com/bluesky-social/atproto/discussions/3366)
- **State:** Closed (answered). Operations / how-to question.
- **Recommended action:** **IRRELEVANT.** Closed Q&A thread, not a design discussion.

### 3.6 Matrix-spec #934 — Cipher upgrade feasibility (CLOSED)

- **Cite:** [matrix-org/matrix-spec#934](https://github.com/matrix-org/matrix-spec/issues/934) (redirected to spec-proposals#3520)
- **State:** Closed. Stale.
- **Recommended action:** **IRRELEVANT.** Don't reopen.

### 3.7 Matrix-spec #597 — Modern encryption (CLOSED, 2020)

- **State:** Closed, 6 years old. About XChaCha20/Blake2b not PQ.
- **Recommended action:** **IRRELEVANT.**

### 3.8 SWICG ActivityPub-E2EE #35 — Key Transparency (open)

- **Cite:** [swicg/activitypub-e2ee#35](https://github.com/swicg/activitypub-e2ee/issues/35)
- **Opener:** soatok 2025-06-12. Open. No PQ discussion.
- **Why NOT to comment now:** The key-transparency vs PQ-sigs question is tangential to the thread's actual topic; injecting PQ-cohort discussion here would be off-topic. Soatok's own blog separately explains why they deferred PQ sigs; engaging that question is for after Benten's blog post is published and Benten can link the published frame.
- **Recommended action:** **WATCH-ONLY for now → POSSIBLE COMMENT-AFTER-blog-post.**

---

## 4. CRDT-sync + local-first identity discussions

### 4.1 Automerge — automerge-classic#419 (claims-based authorization) + discussion #523 (local-first auth integration)

- **Cite:** [automerge/automerge-classic#419](https://github.com/automerge/automerge-classic/issues/419) + [automerge-classic discussions#523](https://github.com/automerge/automerge-classic/discussions/523)
- **State:** #419 open, multi-year. #523 from HerbCaudill (localfirst/auth author) Jan 2023, still open, no PQ mentions.
- **Relevance:** Automerge community is *aware* of signature-per-change problem and the larger local-first auth space. localfirst/auth (Caudill's project) is a concrete sibling-system. Neither thread discusses PQ.
- **Recommended action:** **WATCH-ONLY.** Comments here would need to be auth/sig-design grade, not PQ-grade. If Benten wants to engage the local-first auth design space substantively, that's a separate effort (private outreach to Caudill / Ink and Switch is the higher-leverage path; see §1.6 Keyhive).

### 4.2 Loro CRDT

- **Cite:** [github.com/loro-dev/loro](https://github.com/loro-dev/loro)
- **State:** No public signature/auth/identity discussion surfaced in the search. Loro is currently focused on CRDT mechanics, not auth.
- **Relevance:** Benten *uses* Loro (per CLAUDE.md) inside `benten-sync`. Benten's signature/identity work sits above Loro, not inside it. No comment-target here.
- **Recommended action:** **IRRELEVANT** for this scan (auth is above-Loro).

### 4.3 Yjs — community forums

- **State:** Active community at discuss.yjs.dev; no specific identity/signature/PQ thread surfaced in the scan.
- **Recommended action:** **IRRELEVANT** for this scan.

### 4.4 SSB (Secure Scuttlebutt) — Fusion Identity

- **Cite:** [ssb-ngi-pointer audit report on Fusion Identity](https://ssb-ngi-pointer.github.io/Audit%20Report_%20Secure%20Scuttlebutt%20Partial%20Replication%20and%20Fusion%20Identity.html)
- **State:** SSB community quieter post-2023 (most active devs moved to Willow / Earthstar). Fusion Identity was the most recent identity-work surface. No PQ discussion surfaced.
- **Relevance:** Architecturally the closest peer to Benten among pre-Benten content-addressed systems. SSB's append-only-feeds-with-ed25519-author-IDs is the structural ancestor of Benten's model.
- **Recommended action:** **IRRELEVANT** for active commenting (community quiet); cite in the blog as predecessor evidence.

### 4.5 Earthstar / Willow Protocol

- **Cite:** [willowprotocol.org/earthstar/spec](https://willowprotocol.org/earthstar/spec/) + [forum.malleable.systems thread](https://forum.malleable.systems/t/the-willow-protocol/161)
- **State:** Earthstar = Willow Protocol generalization (per the search). Uses cinn25519 (ed25519 + shortname). No PQ discussion surfaced in spec.
- **Relevance:** Direct conceptual sibling — content-addressed, append-only, identity-keyed. Per Phase-4-Meta-Core context, Benten already evaluated willow_rs in the Willow deep-dive 2026-05-20 + deferred iroh-willow adoption to Phase-5+. Willow's identity is classical-only.
- **Recommended action:** **WATCH-ONLY.** If Willow ecosystem opens a PQ discussion, that becomes a high-leverage COMMENT target (much closer architectural fit than Signal/Matrix). For now nothing to act on. Possible private outreach to Aljoscha Meyer / willow_rs Codeberg fork at Shape-5 stage.

---

## 5. Cluster pattern observations

### 5.1 The "system class" split is visible in the public record

The single sharpest pattern across this scan: **systems that retain artifacts long-term are converging toward PQ-hybrid signatures-by-default; systems that ratchet ephemeral sessions are deferring PQ signatures.**

Evidence:
- **Long-term retainers shipping/planning PQ-hybrid sigs:** Bitcoin/Ethereum (hybrid sig research lane), OpenMLS#1940 (`MLS_256_MLKEM1024_AES256GCM_SHA512_MLDSA87`), Wire (MLS = sig-agile by design), Benten (v1-beta default).
- **Ephemeral-session systems deferring PQ sigs:** Signal (PQXDH §4.8 deniability rationale), SimpleX (explicitly "avoid signatures"), Session (V2 ships ML-KEM only initially), SWICG ActivityPub-E2EE (X-Wing KEM yes, ML-DSA deferred), iMessage/WhatsApp (encryption-only PQ).
- **In the middle:** Nostr (event signatures ARE long-term but the maintainer-public position still says "PQ sigs not urgent"; NIP-44 thread is open to discussion); Matrix (sitting open since 2022, no clear position).

This is exactly Benten's blog argument. The scan validates the framing without Benten needing to manufacture it.

### 5.2 The standards substrate is ready

By April 2026, the IETF + NIST + RustCrypto substrate Benten needs is shipped or near-shipped:
- LAMPS Composite ML-DSA draft-18 in Last Call (sig wire format).
- MLS PQ ciphersuites draft-04 defining MLDSA87 + Ed25519 variants.
- X-Wing KEM draft-10 (March 2026).
- ML-KEM + ML-DSA in RustCrypto (2025).
- OpenSSL 3.5 PQ shipped March 2026.
- Multicodec assignments available for ML-DSA-65 / ML-DSA-87.

The "we're shipping at the right time" framing is honest — not too early.

### 5.3 The genuinely-novel position Benten could occupy

Across the scan, **no shipped system has yet taken the explicit "PQ-hybrid sigs by default at the v1 release with the audited classical half as the trust floor until independent PQ audit lands at v1-GM" posture** — except Benten (per CLAUDE.md baked-in #5 + v1-GM gate per #15 / NF-2 / C-GM-AUDIT).

OpenMLS #1940 is the closest cohort but ships as a *new ciphersuite alongside the classical default*, not as a new default. ActivityPub-E2EE explicitly defers. Signal explicitly defers. Wire ships sig-agile but classical defaults. Bitcoin/Ethereum are research.

This means Benten's posture is **legitimately novel-but-defensible at v1-beta tagging time** — the blog post is on solid ground. It also means Benten is a small-N early data point, which is exactly the kind of evidence the ecosystem needs and is also exactly why humble framing matters.

### 5.4 The Shape-5 coalition is forming organically

Most-promising private-outreach candidates for after the blog post lands (NOT now):

1. **OpenMLS team** (via #1940 + soatok contact) — they're building the same thing in a different system class.
2. **SWICG ActivityPub-E2EE team** (soatok + evanp + mayel) — closest design dialogue; opposite signature choice means most-substantive conversation.
3. **Ink and Switch / Keyhive team** — closest threat model + research-friendly.
4. **Willow Protocol team** (Aljoscha Meyer, Codeberg fork) — closest architectural fit; not yet PQ-engaged.
5. **paulmillr** (Nostr #1971 + noble-post-quantum maintainer) — implementation-side ally with PQ JS impl.

Not a public coalition yet — opportunity to *build* the coalition by being the first to publish the framing + the evidence-set.

---

## Appendix: scan provenance

- Scan branch: `phase-4-meta-core/comment-opps-p2p-messaging`
- All cites are public GitHub / IETF / blog / mailing-list links accessed via WebFetch + WebSearch 2026-05-26
- Zero comments posted anywhere; READ-ONLY discipline maintained
- ~17 WebSearches + ~8 WebFetches consumed across ~75 min of scan time

---
