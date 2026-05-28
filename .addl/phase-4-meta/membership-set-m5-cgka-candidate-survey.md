# MembershipSet M5 — CGKA candidate survey

**Author:** M5 specialist (CGKA candidate survey, 5-of-5 parallel)
**Date:** 2026-05-27
**Branch:** `phase-4-meta-core/membership-set-m5-cgka-candidate-survey`
**Off:** `origin/main @ 2172cb6d`
**Sibling inputs:** M1a / M1b / M1c (cataloger triad, read via `git show origin/phase-4-meta-core/membership-set-cataloger-*`); M2/M3/M4 (parallel, not yet pushed at compose-time — survey is M1-grounded + literature-grounded only).
**Question Ben asked 2026-05-27 evening:**

> "is CGKA-LITE the right framing or should we be reconsidering looking for the strongest current full CGKA setup we could use/create? I'm definitely open to considering [Cryptree]; we're going to be the production valuator for a handful of other things that are central to our vision of the future. I do kinda feel like MLS-PQ (if it's ready to test in at least beta production) could be an ideal permanent shape as well. Though that Cryptree forkable by design seems so made for us. So the tradeoffs will be important."

**Output discipline.** §1 is the read-cold executive recommendation; §§2-8 are the substantive survey; §9 is the final recommendation with R0 plan-doc implications; §10 names deferred alternatives; §§11-12 are self-assessment + citations. Plain-English-with-prediction lens applied to §1 and §9.

---

## §1 Executive recommendation + confidence

**Recommendation: KEEP the Q3-Option-D CGKA-deferral posture for v1-beta. RE-FRAME "CGKA-LITE" as "no-CGKA + multi-recipient HPKE + FORK-ON-ADMIN-KICK" (NOT as a thin CGKA wrapper).** Reserve a single `MembershipSetKind::AtriumWithCGKA` codepoint slot at v1-beta-CODEPOINT-RESERVE; do NOT ship any CGKA implementation pre-v1-beta. Plan to **adopt OpenMLS-with-X-Wing-PQ-hybrid as the post-v1-beta CGKA when (a) draft-ietf-mls-pq-ciphersuites reaches RFC, (b) OpenMLS ships a stable PQ-merged release, and (c) Benten has a concrete use-case requiring PCS that doesn't violate the "forkable not forward-secret-on-leave" semantic.**

**Confidence: MEDIUM-HIGH** on the v1-beta keep-deferral verdict; **MEDIUM** on the long-term MLS-PQ-hybrid choice (Cryptree-derived shapes are a real alternative for the file-tree-shaped sub-surface, but they don't actually solve the same problem — see §3); **LOW-MEDIUM** on the codepoint-reserve sizing (one reserve may be insufficient if Benten ever wants both MLS-derived AND DCGKA-derived shapes — see §6).

### §1.1 Plain-English-with-prediction

**The situation.** Ben asked whether to upgrade "CGKA-LITE" (a thin Benten-authored wrapper around HPKE + ML-KEM-X25519 multi-recipient sealing) to one of three real CGKAs: MLS RFC 9420 (pre-quantum, mature), MLS-PQ (draft, OpenMLS experimental), or Cryptree (Ben's intuition: "forkable by design"). Ben wants the tradeoffs surfaced.

**The three concrete options.**

- **(A) CGKA-deferral with no-CGKA naming.** Keep Q3 Option D. Rename "CGKA-LITE" to "no-CGKA-multi-recipient-HPKE" so we stop accidentally promising properties (FS, PCS) we don't deliver. Reserve ONE `MembershipSetKind::AtriumWithCGKA` codepoint. ADMIN-KICK = fork-with-recipient-exclusion. **0-2 wave-days. Recommended.**
- **(B) Adopt MLS-PQ via OpenMLS-PQ-experimental NOW as the substrate.** Risk: experimental branch; no IANA codepoint for X-Wing ciphersuite (interop blocker); 9x message-size blow-up; OpenMLS server side not production-ready; commits Benten to MLS wire format permanently. **15-25 wave-days. Not recommended for v1-beta.**
- **(C) Adopt Cryptree (Peergos-style) for read-access control.** Cryptree is NOT a CGKA — it's a key-derivation-tree for filesystem-shaped read access. It SOLVES a different problem (per-file/folder read tokens; "give me access to /foo and I get the descendants for free") and DOES NOT solve admin-kick-with-future-content-exclusion any more cleanly than fork-on-kick already does. Ben's "forkable by design" intuition is actually a property of the Atrium-fork semantic Ben already ratified, NOT a property of Cryptree's mechanism. **8-15 wave-days IF the goal is per-Node fractal read tokens; ZERO benefit IF the goal is admin-kick.**

**My prediction of Ben's call.** Ben goes with (A) once the Cryptree-isn't-actually-a-CGKA correction lands. Cryptree's "forkable by design" framing was load-bearing in Ben's intuition because Benten IS adopting forkability as the central semantic — but the forkability comes from the Atrium-fork ratification, not from Cryptree. MLS-PQ is the right LONG-TERM eventual home but is NOT beta-production-ready in the SPECIFIC sense Ben asked about: the IETF draft is "Revised I-D Needed" (`draft-ietf-mls-pq-ciphersuites-04`, expires 2026-09), the OpenMLS PQ implementation uses X-Wing (NOT in the draft), and there are no production deployments of MLS-PQ to point to (Wire/Webex/Discord/Google Messages/Apple Messages all ship MLS classical only). Re-asking in Phase-4-Meta-Composing (or Phase-5+) with another year of standardization lets us pick the actual settled PQ ciphersuite.

**Confirm or redirect.** I'm 80%+ on (A) given Q3 Option D already ratified the structural decision. Redirect-vectors: if Ben values "be the production validator for Cryptree" as a strategic statement independent of the technical merit, that re-opens (C) for a NARROW use-case (per-Node fractal-read-tokens for the Phase-8 decentralized-registry trust-graph; NOT for Atrium membership), with the rest of (A) intact.

### §1.2 Critical correction Ben should see first

**Ben's note conflates two things.**

- **Cryptree** (Grolimund / Meisser / Schmid / Wattenhofer, ETH 2006) is a **key-derivation tree for filesystem read access control** in untrusted storage. It uses symmetric clearance keys + subfolder keys + backlink keys to implement "if you hold the key for /foo, you can derive the keys for /foo/bar and below in O(1) per subfolder." It is **NOT a CGKA**. It does NOT do group key agreement, FS, PCS, or member-rotation. It is "forkable" only in the trivial sense that any symmetric-key system is forkable by re-keying.
- **Willow** (Aljoscha Meyer + Sam Gwilym, NLnet-funded 2024-2026) is a meta-protocol for distributed data sync with a capability system (Meadowcap). Willow'25 spec uses Ed25519 + ChaCha20-Poly1305 for transport but explicitly **leaves application-layer encryption + key management to the application**. Willow does NOT specify a CGKA either.
- **Peergos** (active production) is the closest thing to "Cryptree in production" — it's Java/Scala (server) + Java (Android) + JS (web), with **NO Rust implementation**. Peergos does add quantum-resistant signing on top of Cryptree, but the key-derivation-tree itself is symmetric and pre-quantum-only.

**What Ben likely meant by "Cryptree forkable by design".** The actual property Ben values is the **Atrium-fork semantic** that Ben himself ratified 2026-05-27 morning: "member-leaves-keeps-past-content; future-content-excludes-via-recipient-set." That semantic is achieved by fork-creates-new-K_Atrium per Q3 Option D §2.7 — and does NOT require Cryptree. Cryptree is orthogonal: it answers "how do I efficiently grant per-folder read access?" not "how does a member-leave affect future content?"

**Action.** I treat Cryptree below as a real candidate for COMPLETENESS but flag it as solving a different problem than Ben asked about.

---

## §2 Candidate survey

Per Task 1: each candidate gets a fixed-shape subsection so the M5 reader can compare apples-to-apples. Severity flags: **PQ-NO** (pre-quantum only); **PQ-HYBRID** (draft hybrid); **PROD-NO** (no production users); **AUDIT-NO** (no 3rd-party audit); **RUST-NO** (no production Rust impl).

### §2.1 MLS RFC 9420 (classical)

- **Standardization status.** RFC 9420 (Messaging Layer Security Protocol) published July 2023; RFC 9750 (MLS Architecture) published 2024. Stable; mandatory-to-implement ciphersuite is MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519. [1][7]
- **Primitives.** TreeKEM (binary ratchet tree, log(N) update cost) + HPKE (RFC 9180) + Ed25519 signatures + AEAD. Default ciphersuite: X25519 + AES-128-GCM + SHA-256 + Ed25519. **PQ-NO**.
- **Forward secrecy.** Yes (epoch-based; old group secrets unrecoverable after key-schedule rotation). **PCS.** Yes (any member's update heals the group). **FS-on-remove.** Yes (the remove-Proposal + Commit creates a new epoch; the removed member's last-known group secret is FS-protected from future epochs).
- **Forkability fit.** **POOR** for Benten's semantic. MLS is designed AGAINST forkability — the whole point of an epoch transition is that everyone who was a member at epoch N-1 keeps a coherent shared key for epoch N if and only if they're still in the group. "Member who left keeps past content" works trivially (they already decrypted it) but "future content excludes them via recipient-set" is exactly what MLS does, except MLS additionally forward-secures the OLD group-secret against the leaver — which is GREATER security than Benten wants but is what MLS gives. So MLS over-delivers on the kick-semantic; under-delivers on the fork-semantic (a "fork" in MLS is just creating a new group with a new GroupID, which is fine but redundant with what fork-on-Atrium-creation already does).
- **Open-source Rust.** TWO production-grade impls. **OpenMLS** (Phoenix R&D + Cryspen; v0.8.1 released 2026-02-13; MIT; ~1875 commits; client-side fully MLS-compliant; server-side stubs only). **mls-rs** (AWS Labs; 100% RFC 9420 conformance; Apache-2.0; supports WASM, configurable storage, OpenSSL + RustCrypto cipher backends; **not yet 3rd-party-audited**). Both are well-maintained as of May 2026. [1][3]
- **Audit / formal verification.** Cryspen's libcrux (the cipher backend OpenMLS uses by default) has formally-verified ML-KEM + x25519. OpenMLS protocol layer itself has not received a full 3rd-party audit. mls-rs has not received a 3rd-party audit. [3][6]
- **License.** OpenMLS MIT; mls-rs Apache-2.0. Both compatible with Benten's licensing posture.
- **Wire-format compatibility with Benten F-full.** **POOR.** MLS imposes the entire MLS message framing + GroupContext + KeyPackage / Welcome / Commit / Proposal wire shape. Benten's F-full envelope (`EnvelopePayload` with `HpkeMultiBase` planned at L9-A1) is structurally different. Integration = wrap-in-wrap (Benten envelope outside, MLS commit/welcome inside). High-overhead; encoding-overhead-on-overhead.
- **Production users.** Wire (full deployment); Webex (production conferencing); Discord (voice/video calls 2026); Google Messages + Apple Messages (RCS rollout May 2026); Mozilla Thunderbird (planned). [4][7]
- **Integration cost estimate.** ~10-15 wave-days IF Benten accepts MLS framing on the wire + wraps it in Benten's envelope. ~20-30 wave-days IF Benten wants to use MLS as a key-derivation primitive only + keep its own framing.

### §2.2 MLS-PQ (draft-ietf-mls-pq-ciphersuites-04) — PQ-HYBRID, DRAFT

- **Standardization status.** Internet-Draft, revision -04 published 2026-03-18. State: **"Revised I-D Needed"** — waiting for WG-chair go-ahead. Intended status: Proposed Standard. Expires 2026-09-20; no telechat date assigned. NOT an RFC. Will need at least one more revision before WGLC. [2][5]
- **Primitives.** Nine registered ciphersuites combining ML-KEM (768 or 1024) with X25519/P-256/P-384/ML-DSA. Notably **does NOT include X-Wing** (OpenMLS's PQ impl uses X-Wing — a separate construction from `draft-connolly-cfrg-xwing-kem-10`, not in the MLS draft).
- **OpenMLS PQ support.** OpenMLS has shipped MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519 (ciphersuite 0x004D) since April 2024 (Cryspen post). This is **NOT in the MLS-PQ draft**; it's a parallel implementation choice. Has **no IANA codepoint** (interop blocker). v0.8.1 main branch shows no MLS-PQ ciphersuites in the README; PQ is in a separate feature branch / older Cryspen impl. [3][6]
- **mls-rs PQ support.** Not advertised in mls-rs README as of May 2026 search.
- **Combiner draft.** `draft-ietf-mls-combiner-02` (Amortized PQ MLS Combiner) is an orthogonal but related effort: combine a classical MLS session + a PQ MLS session, amortizing the PQ cost. Still draft; not in any production impl. [8]
- **Forward secrecy / PCS.** Same as MLS RFC 9420 (epoch-based; key-schedule rotation).
- **Forkability fit.** Same poor fit as MLS RFC 9420 — PQ doesn't change the semantic. **PQ-HYBRID, PROD-NO.**
- **Production users.** **NONE that I could verify.** Wire / Webex / Discord / Google Messages / Apple Messages all ship MLS classical. No publicly-announced MLS-PQ production deployment as of May 2026.
- **Cryptographer-audit readiness.** PRE-mature. The draft is unstable; the OpenMLS X-Wing impl uses a CFRG draft (`draft-connolly-cfrg-xwing-kem-10`); the libcrux ML-KEM impl IS formally verified for correctness but the integration into TreeKEM is not.
- **Integration cost estimate.** ~15-25 wave-days for an experimental integration on top of OpenMLS's X-Wing branch; **HIGH risk of churn** when the draft stabilizes on a different ciphersuite (the draft has 9 ciphersuites; X-Wing is NONE of them). Likely 5-10 more wave-days of churn-cost when re-pinning to the final RFC.
- **"Beta-production-ready" verdict (Ben's specific ask): NO.** With explicit caveats — see §4 for full reasoning. Re-assess at Phase-4-Meta-Composing OR when RFC publishes (whichever later).

### §2.3 Cryptree (Grolimund/Meisser/Schmid/Wattenhofer 2006; Peergos 2018+)

- **Standardization status.** Academic paper (SRDS 2006 + IEEE Xplore); **no IETF/IANA standardization**. Peergos has documented its own variant in book.peergos.org. [10][11]
- **Primitives.** Symmetric clearance keys + subfolder keys + backlink keys, organized as a tree mirroring the filesystem hierarchy. Underlying primitives: AES (any flavor) for key encryption; some hash function for key-derivation. Asymmetric signing (Ed25519 in Peergos; quantum-resistant variants discussed in book.peergos.org §4.9) for write authorization. **PQ-NO at the read-token layer**; PQ-HYBRID-POSSIBLE at the signing layer.
- **What it actually solves.** Per-folder/per-file read tokens with O(1) cascade-grant ("give me /foo and I get /foo/bar/baz for free"). The point is access control over a hierarchy, NOT group key agreement.
- **Forward secrecy.** NONE structurally. The keys persist as long as the data persists; "revocation" = re-key the data + redistribute new keys to remaining holders (lazy or active re-encryption). This is the SAME limitation Benten has with HPKE-mode-base (per Q3 conclusion).
- **PCS.** None.
- **Forkability fit.** Trivially "forkable" — re-keying a subtree creates a new key-tree. But this is no different from any symmetric-key cascade; it's not a property unique to Cryptree's mechanism. **Ben's intuition "Cryptree is forkable by design" actually corresponds to the Atrium-fork semantic, not to anything specific to Cryptree.**
- **Benten-fit:** GOOD for a hypothetical per-Node / per-Subgraph "share me this subtree" read-token feature; ORTHOGONAL to the Atrium MembershipSet question. Could be a Phase-5+ "Subgraph-share" surface.
- **Open-source Rust.** **NONE.** Peergos is Java/Scala (peergos.org / GitHub `Peergos/Peergos`). No production Rust Cryptree implementation exists as of May 2026 search. **RUST-NO**.
- **Audit / formal verification.** Peergos has had partial security audits (Cure53 2018, RHEA 2024 per peergos.org); the Cryptree paper itself is a 2006 academic publication with no formal-verification follow-up.
- **License.** Peergos AGPL-3.0 (incompatible with most commercial-friendly licensing; copyleft contagion).
- **Wire-format compatibility with Benten F-full.** N/A — Cryptree is a key-management scheme, not a wire format. Could compose with F-full envelope at the key-derivation seam (replacing K_Atrium-derives-K(N) with cryptree-derives-K(Node)).
- **Production users.** Peergos itself (small but real user base; NLnet-funded; recurring grant funding through 2026).
- **Integration cost estimate.** ~8-15 wave-days for a Rust port + Benten-shape adapter, IF the use-case is per-Node/per-Subgraph fractal-read-tokens. ZERO benefit for the admin-kick problem.

### §2.4 TreeKEM standalone (the algorithm inside MLS)

- **Standardization status.** Not standardized as a standalone; specified inside RFC 9420 §6 + §7. No standalone IANA registry.
- **Primitives.** Binary ratchet tree of HPKE keypairs; updates require log(N) ciphertexts.
- **FS / PCS.** Yes / yes (this is the source of MLS's properties).
- **Forkability fit.** Same as MLS (poor); plus loses MLS's framing benefits (KeyPackage, Welcome, GroupContext). Choosing TreeKEM-without-MLS = re-implementing MLS without the spec.
- **Open-source Rust.** No standalone production impl. Use OpenMLS or mls-rs.
- **Verdict:** DOMINATED by MLS itself; only consider if Benten wants TreeKEM as a primitive INSIDE a Benten-authored framing (high R&D cost; auditability worse than picking MLS).

### §2.5 Signal Sender Keys

- **Standardization status.** Proprietary; documented in Signal's protocol page + libsignal source. Not an IETF spec.
- **Primitives.** Pairwise Double Ratchet for sender-key distribution + symmetric chain-key + AES-CBC + HMAC. Sender key MOSTLY pre-quantum (Signal added PQXDH for handshake but Sender Keys remain classical).
- **FS / PCS.** Sender-key has chain-rotation FS but **NO PCS at the group level** — a compromised member's sender key reveals future messages from that sender until the sender voluntarily rotates. Adding/removing members requires re-issuing a sender key to all (or remaining) members via pairwise channels: O(N) per change.
- **Forkability fit.** GOOD on the "leave-keeps-past-content" axis (sender-key chains are linear; leaver retains the chain state they had). BAD on scaling: O(N²) message complexity for full-mesh sender-key distribution.
- **Open-source Rust.** **libsignal** (signalapp/libsignal) is the official AGPLv3 Rust implementation; widely deployed (powers Signal mobile, Signal Desktop, MobileCoin). The Sender Key API is exposed but is part of the larger libsignal_protocol crate. Community alternatives (couchand/signal-rs) are not production-grade.
- **Audit.** libsignal has had multiple 3rd-party audits over Signal's history.
- **License.** **AGPLv3 = blocker** for Benten's plugin/UCAN/embedded distribution model. Copyleft contagion via dynamic linking is an open legal question; conservative interpretation = AGPLv3 cannot be used.
- **Verdict:** DOMINATED — AGPLv3 alone disqualifies; even setting that aside, MLS subsumes Sender Keys with better complexity.

### §2.6 DCGKA (Weidner / Kleppmann / Hugenroth / Beresford 2021)

- **Standardization status.** Academic (ACM CCS 2021); **no IETF process**. Reference impl exists as a prototype. [9][12]
- **Primitives.** 2SM (Two-Party Secure Messaging, essentially Double-Ratchet) + a decentralized control message scheme. Causally-ordered control messages cooperate to converge on the group's current member set + key. **PQ-NO** as published; substitutable PQ KEM for the 2SM primitive (would require redesign).
- **FS / PCS.** YES / YES. The whole point of DCGKA is to achieve FS+PCS in a decentralized (no-central-server) setting.
- **Forkability fit.** **MIXED.** DCGKA is the ONLY candidate that's designed for the local-first / CRDT / offline-tolerant world Benten lives in. But DCGKA still FORWARD-SECURES against removed members, which contradicts Benten's "leaver retains past content" semantic. (The semantic difference: DCGKA forward-secures the SHARED group secret; the leaver still has whatever per-message ciphertexts they downloaded before the remove epoch. So in practice this matches Benten's "leaver keeps what they had" if Benten ensures content is delivered as plaintext-CID-blinded ciphertext before the remove, not derived live from group state.)
- **Composition with CRDT.** EXCELLENT — DCGKA is causally-ordered-control-message-based, which composes with Loro CRDT + MST sync exactly the way Benten's atrium-sync substrate works. This is the **most-Benten-shaped candidate by far**.
- **Open-source Rust.** **p2panda-encryption** (crates.io; pending Radically Open Security audit when last announced February 2025; license: per p2panda's typical, MIT or Apache-2.0; needs verification). The implementation is an "adaptation of DCGKA for offline-first use" — close to a production-grade Rust DCGKA. NOT yet a 1.0 release; audit-pending. [13][14]
- **Audit / formal verification.** p2panda-encryption has a pending Radically Open Security audit (per Feb 2025 p2panda post). Underlying DCGKA paper has formal-model security proofs in the academic literature.
- **License.** p2panda historically uses MIT/Apache-2.0 — needs version-pin verification.
- **Wire-format compatibility.** GOOD — DCGKA control messages compose with Benten's existing causally-ordered Loro CRDT update flow.
- **Production users.** p2panda itself (small, NLnet-funded but real); no large-scale deployment.
- **Integration cost estimate.** ~10-18 wave-days for a v1 integration via p2panda-encryption dependency adoption + Benten-shape adapter; **HIGH dependency risk** (p2panda is small-team; if they pause, Benten inherits maintenance).

### §2.7 Bespoke "CGKA-LITE" (Benten-authored thin HPKE + ML-KEM-X25519 wrapper)

- **What it is.** The Q3-Option-D-aligned shape: multi-recipient HPKE (per L9-A1 `HpkeMultiBase` stanza) + ML-KEM-X25519 PQ hybrid + per-Atrium K_Atrium random-on-creation + fork-on-rotate. **NOT actually a CGKA** in the technical literature sense (no continuous key agreement; no PCS; no FS-against-future-leakage-of-K_Atrium).
- **FS / PCS.** NONE structurally. The leak posture is "K_Atrium-leak compromises all past content for the lifetime of that K_Atrium; fork-frequency bounds the blast radius."
- **Forkability fit.** PERFECT — by construction Q3 Option D is exactly the Atrium-fork semantic Ben ratified.
- **Open-source Rust.** Benten authors. Uses `benten-crypto-suite` (already in tree).
- **Audit.** Whatever Benten arranges (e.g. NCC Group, Trail of Bits, Cure53 — none currently scheduled).
- **Integration cost.** Already partially shipped per M1b §3.1: L9-A1 multi-recipient + L9-A4 K_principal-generation. Estimated remaining: ~5-8 wave-days to close the U17 multi-stanza HPKE + U21 ExecuteWorkflow gaps.
- **Risk.** Benten-authored cryptographic substrate at the protocol-composition layer = reviewer-skepticism magnet. But Benten is already taking this risk (F-full encryption is a Benten-composed substrate); the question is whether to bound it tightly (Q3 Option D) or to expand it.

### §2.8 Saltpack / age / repudiable multi-recipient

- **Saltpack.** Keybase's NaCl-based message format. Multi-recipient (up to 2^32-1), forward-secret via per-message ephemeral key. NOT a group key agreement — every recipient gets the payload-key encrypted to them individually. Pre-quantum.
- **age** (str4d/FiloSottile). File-encryption tool with multi-recipient support. Not a group key agreement; per-message ephemerals. Pre-quantum.
- **Verdict.** Both are essentially "multi-recipient HPKE with extra framing." Already subsumed by Benten's HPKE-mode-base + multi-recipient stanza plan. Not competitors at the CGKA layer.

### §2.9 Pond / Briar / Cwtch group-keying

- **Pond.** Discontinued (last update ~2014).
- **Briar.** Bramble Transport Protocol over Tor; group messaging uses a "private group" scheme with single shared key + per-message Diffie-Hellman. No PCS at the group level. Pre-quantum. Java/Android; no Rust impl.
- **Cwtch.** Tor-based; uses single-shared-key group scheme similar to Briar. Go; no Rust impl.
- **Verdict.** All dominated by either MLS (for PCS) or Q3-Option-D (for forkability). Not viable Benten candidates.

### §2.10 OPRF-based group key agreement

- **What it is.** Oblivious Pseudo-Random Function constructions (e.g. OPAQUE for password-based KE; threshold OPRF for distributed PRF). Some recent papers propose group-key-agreement from OPRF.
- **Status.** Academic; no IETF group-keying standardization. OPRF itself is IETF-standardized (RFC 9497).
- **Verdict.** Not a v1-beta candidate; possibly a Phase-N+ research direction for threshold-admin (M-of-N admin signing) — orthogonal to MembershipSet rotation.

### §2.11 Nostr NIP-EE (MLS-over-Nostr)

- **What it is.** NIP-EE / NIP-104: Nostr's adoption of MLS for group messaging. Uses OpenMLS + nostr crates (nrc-mls). Reference impl: Marmot / White Noise. [15]
- **Status.** Proposed Nostr extension; reference impls in development.
- **Relevance to Benten.** Not directly relevant (Nostr is the relay model Benten chose NOT to adopt for confidentiality) but confirms MLS is the de-facto standard for "I want decentralized E2EE group messaging."
- **Verdict.** Confirms ecosystem direction is MLS-shaped. Reinforces §1's MLS-PQ-eventually recommendation.

---

## §3 Cryptree deep-dive

### §3.1 What it actually is

Cryptree (Grolimund et al., SRDS 2006) is a **key-derivation tree** for cryptographic file systems. The folder hierarchy IS the key hierarchy: each folder F has a clearance key C_F + a subfolder key S_F + a backlink key B_F. Holding C_F lets you decrypt the keys for all descendant folders (and their files) deterministically. Granting access = giving someone C_F at any point in the tree; revoking = re-keying the subtree (lazy or active).

### §3.2 Why Ben might find this attractive

Three structural properties match Benten's vision:

1. **Hierarchical inheritance.** Benten's Anchor+Version+CURRENT pattern + Subgraph composition is itself hierarchical. A Cryptree-shaped read-token scheme could naturally fold over the Subgraph tree.
2. **Untrusted storage.** Cryptree explicitly assumes the storage layer is hostile — same threat model as Benten's Atrium-sync + the K_Atrium plaintext-CID blinding (per Q3 Option D).
3. **Constant-time grant.** "Give me access to /family/2025/photos" = hand over ONE key + the recipient derives all descendants. Maps cleanly onto Benten's "grant access to a Subgraph (including its growing CURRENT tip)" semantic.

### §3.3 Why it doesn't solve the MembershipSet question

The MembershipSet question is about **WHO is in the set** (admin can add, admin can kick, members can leave). Cryptree's mechanism is about **WHAT a holder of key C_F can read** (the tree rooted at F). These are orthogonal concerns. You could imagine layering them: Cryptree manages per-Subgraph read tokens; MembershipSet manages who has the root tokens; rotation = re-key root + redistribute to remaining MembershipSet.

But that layering buys nothing over the simpler scheme Benten already has (multi-recipient HPKE-sealed K_Atrium per Atrium; sealing K_Atrium to the current MembershipSet's HPKE pubkeys; fork-on-rotate re-issues sealing). Cryptree's value would be **per-Node fine-grained read tokens within an Atrium**, which is a Phase-5+ feature, not a v1-beta MembershipSet design choice.

### §3.4 "Forkable by design" interpretation

What Cryptree HAS:
- O(1) re-key of a subtree = a "fork" at any node.
- Lazy re-encryption (existing data stays encrypted under old keys; new writes use new keys).

What Cryptree LACKS (which is what Ben's "forkable" really wants):
- A semantic for "this fork's MembershipSet diverged from the parent's MembershipSet."
- Forward secrecy guarantees against the parent (it doesn't have any).
- Post-compromise security.

The "fork" in "fork-by-design" is a property of the **Atrium-shape Ben ratified**, not of Cryptree. Cryptree just happens to make re-keying O(1) per subtree, which is nice but is a perf detail not a semantic.

### §3.5 PQ composition

Cryptree's read-token layer is purely symmetric (AES + hash). Symmetric primitives are PQ-secure with appropriate key sizes (AES-256 = ~128-bit PQ security per Grover). The **signing layer** (write authorization in Peergos: Ed25519 / dilithium-variant) is PQ-hybrid-able with ML-DSA. No PQ-research-grade adaptation needed.

### §3.6 Production-readiness as a Benten-validator

If Benten wanted to be the production-validator for Cryptree:
- **No Rust impl exists.** Benten would author one (~8-15 wave-days for a complete port).
- **Peergos has documented + lightly-audited it.** Benten would inherit some audit confidence but no formal Cryptree-specific verification.
- **Strategic value.** Cryptree-in-Rust is a genuine ecosystem gift if Benten ships it as a separate crate. The library would be useful to local-first / IPFS-ish / encrypted-FS projects beyond Benten.
- **Risk.** Cryptree is solving a NICHE problem (per-Subgraph read tokens) that Benten doesn't actually have at v1-beta. Authoring + auditing a Cryptree library FOR the strategic-validator value is a 5-10-wave-day investment for a feature that isn't on the v1-beta critical path.

### §3.7 Wave-day cost estimate

- **For MembershipSet (admin-kick): 0 wave-days from Cryptree (it doesn't apply).**
- **For Phase-5+ per-Subgraph read tokens: ~8-15 wave-days for a Rust port + ~5 wave-days for Benten-shape integration + ~10-20 wave-days for cryptographer audit.**
- **For "production-validator strategic statement" alone: ~10-15 wave-days minimum.**

**Cryptree verdict:** DEFER as a Phase-5+ candidate for per-Subgraph read tokens. NOT a MembershipSet candidate.

---

## §4 MLS-PQ beta-production-readiness assessment

### §4.1 The draft

`draft-ietf-mls-pq-ciphersuites-04` (Mahy / Hale / Mularczyk / Sullivan, 2026-03-18). State: "Revised I-D Needed" — explicitly, the WG has requested another revision before Working Group Last Call. Intended status: Proposed Standard. Expires 2026-09-20. No IETF telechat date assigned. Nine ciphersuites registered (ML-KEM-768/1024 + X25519/P-256/P-384 + AES-128/256-GCM + Ed25519/ECDSA/ML-DSA). [2][5]

**The draft is NOT in IETF Last Call.** Realistic publication timeline: 6-12 months minimum (1-2 more revisions + WGLC + IETF Last Call + IESG review + RFC editor queue). Likely RFC publication: late 2026 or 2027.

### §4.2 The OpenMLS PQ branch

OpenMLS shipped `MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519` (suite 0x004D) in April 2024, using **X-Wing** (Connolly et al.; `draft-connolly-cfrg-xwing-kem-10` as of March 2026). [3][6] X-Wing is a different hybrid construction from the ones in the MLS-PQ ciphersuites draft (`draft-ietf-mls-pq-ciphersuites` lists ML-KEM + X25519 in CONCATENATION, not the X-Wing construction). X-Wing also is not yet IETF-endorsed (the draft notes: "this I-D is not endorsed by the IETF and has no formal standing in the IETF standards process").

So OpenMLS PQ is **TWO drafts away from a stable IANA codepoint**:
1. X-Wing CFRG draft needs to advance + Get IANA codepoint, OR
2. OpenMLS needs to migrate to one of the `draft-ietf-mls-pq-ciphersuites` constructions and ship a new ciphersuite.

OpenMLS's main v0.8.1 README (released 2026-02-13) does **not** list any PQ ciphersuite as supported in the stable release. The 2024 PQ blog post described a feature-branch / Cryspen-fork implementation. The OpenMLS team plans interoperability work "with other implementers" but no IANA codepoint yet. [6]

### §4.3 mls-rs (AWS Labs) PQ status

mls-rs README (May 2026 fetch) lists 100% RFC 9420 conformance + WASM + configurable storage but does **not** advertise PQ ciphersuites. AWS-internal PQ work appears to be focused on TLS / KMS (X-Wing is in Google Cloud KMS but not announced for AWS).

### §4.4 Production deployments of MLS-PQ

**Zero verified.** Wire, Webex, Discord, Google Messages, Apple Messages, Mozilla Thunderbird (planned) — all on classical MLS RFC 9420 ciphersuites. No public announcement of MLS-PQ deployment as of May 2026.

### §4.5 Cryptographer-audit readiness

The cipher primitives (ML-KEM via libcrux) are formally verified. The TreeKEM-on-PQ integration has not been audited. The X-Wing construction has had formal analysis (a 2026 eprint paper on "anonymity of X-Wing variants" + a paper on "necessity of public contexts in hybrid KEMs: case study X-Wing" suggests active scrutiny). [11]

### §4.6 Integration cost into Benten

- **If we adopt OpenMLS's X-Wing branch as-is:** ~15-25 wave-days for adapter + wrapping in Benten's envelope + key-storage glue.
- **Plus ~5-10 wave-days of churn** when OpenMLS migrates to the IETF-standard ciphersuite (highly likely within 12-18 months).
- **Plus ~10-20 wave-days of churn** if AWS mls-rs ends up being the eventual production-grade Rust MLS and OpenMLS development slows (low probability but real).
- **Plus ~10-15 wave-days for cryptographer audit of the Benten integration layer**.

Total: **35-65 wave-days of work over the next 12-18 months** if we commit now, with **HIGH risk** of needing to redo significant portions.

### §4.7 "Beta-production-ready" verdict

**NO**, with the following structural caveats:

- The IETF DRAFT is not at WGLC, let alone RFC. "Revised I-D Needed" is an unambiguous "not yet."
- The most-mature Rust impl (OpenMLS X-Wing branch) uses a CIPHERSUITE THAT IS NOT IN THE DRAFT.
- There are NO production deployments to point to.
- Re-evaluation gate: **Re-assess at Phase-4-Meta-Composing OR when the IETF draft reaches WGLC OR when one of {Wire, Webex, Discord} announces MLS-PQ production deployment — whichever comes first.**

The honest answer to Ben's "if it's ready to test in at least beta production" gate: it isn't.

---

## §5 Trade-off matrix

Scoring legend: **A** = excellent fit / mature / no-concerns; **B** = good with caveats; **C** = workable with significant caveats; **D** = poor fit; **F** = disqualified.

| Criterion | MLS RFC 9420 (classical) | MLS-PQ draft + OpenMLS-XWing | Cryptree (Peergos) | CGKA-LITE (Q3 Option D) | Signal Sender Keys | TreeKEM standalone | DCGKA / p2panda | OPRF-GKA |
|---|---|---|---|---|---|---|---|---|
| Forkability semantic match | D | D | C (forkable but orthogonal) | **A** | C | D | C-B (FS over-delivers) | n/a |
| PQ-readiness | F (classical only) | B (hybrid, draft-stage) | C (symmetric-PQ at read-token; signing-PQ-hybridable) | **A** (ML-KEM-X25519 baked in) | F | F | C (substitutable but unimplemented) | C (varies) |
| Production impl quality | **A** (OpenMLS + mls-rs) | C (OpenMLS feature-branch) | D (no Rust) | B (Benten-authored, in tree) | A (libsignal) but AGPLv3 | n/a | C (p2panda, pending audit) | F |
| Cryptographer-audit-readiness | B (libcrux verified; OpenMLS-WG-reviewed; protocol layer not 3rd-party-audited) | C-D (draft + experimental ciphersuite + no protocol-layer audit) | C (Peergos partial audit; not formal-verified) | C (Benten-authored, no audit scheduled) | A | n/a | C (audit pending) | D |
| Benten-fit (composition) | C (wire-format clash; wrap-in-wrap) | C | C-B (good for per-Subgraph read tokens; orthogonal to MembershipSet) | **A** (already designed for it) | D | D | B (causal/CRDT-shaped) | D |
| Wave-day cost (initial integration) | 10-15 | 35-65 (with churn) | 8-15 (Rust port) + N/A for MembershipSet | 5-8 (already partial) | n/a | 30+ | 10-18 | n/a |
| Long-term ecosystem alignment | A (de facto standard) | A (once stable) | C (niche) | C (Benten-specific) | C (Signal-specific) | n/a | B (local-first community) | D |
| Future-additivity if we change later | **A** (codepoint reserve lets us add MLS later) | A | A | **A** (Option D is forward-compatible with adding CGKA later) | C | n/a | B | n/a |

**Pattern:** Q3-Option-D-aligned "CGKA-LITE" wins on forkability + PQ + cost + future-additivity. MLS RFC 9420 wins on production quality + ecosystem. MLS-PQ is the long-term destination but isn't ready. Cryptree is solving a different problem. DCGKA is the most-Benten-shaped real CGKA but is unproven + small-team.

---

## §6 Recommendation per use-case

Per Task 5, different MembershipSet operations may want different CGKA primitives:

### §6.1 FORK-ONLY rotation (Q3 Option D default)

**Recommendation:** **No CGKA.** Multi-recipient HPKE + ML-KEM-X25519 + fork-creates-new-K_Set. This is the Q3 Option D §2.7 ratified shape and is the most-Benten-shaped + least-risky path.

### §6.2 ADMIN-KICK-EPOCH rotation (Ben's new ask)

**Recommendation:** **Fold into FORK-ONLY.** "Admin kicks member" = "admin forks the set without member; signs an attestation to remaining members that the new fork is the canonical CURRENT; old set continues to exist for kicked-member's reads of pre-kick content but no member of the new set publishes to the old K_Set." This is operationally identical to FORK-ONLY + an attestation pointing the remaining members at the new fork. NO new cryptographic primitive needed.

**Why not real CGKA here?** Because the property a real CGKA gives you (FS against future leakage of the OLD group secret to the leaver) is NOT what Ben's "forkable" semantic asks for. Ben's semantic explicitly is "kicked member still has access to past content they already received" — which is exactly what fork-on-kick gives. Adding a CGKA here would deliver MORE security than the semantic, at the cost of complexity + a wire-format dependency.

**The ONLY reason to revisit:** if a future use-case appears that wants "kick + cryptographically guarantee future content is unreadable to leaver even if leaver compromises a CURRENT member's state" (which is what PCS would give). Phase-4-Meta-Composing or later.

### §6.3 DeviceMesh add/remove (single-user)

**Recommendation:** **No CGKA.** A user's DeviceMesh is the user's own DAK-derived per-device keys (M1c §4). Add device = mint new device-key via DAK; remove device = old-key signs SelfRevocation (Phase-4-Foundation MVP). Re-keying the user's K_principal on device-remove requires (a) the principal-DID still has at least one trusted device + (b) per-device sealing of the new K_principal. This is closer to a small-N multi-recipient HPKE than to a CGKA.

**The interesting sub-question:** does DeviceMesh need PCS? Use-case: "my old phone got stolen; I revoked it; I want my future K_principal-encrypted content to be unreadable to the thief even if they later compromise a CURRENT device." This IS a PCS use-case. But the most-practical answer at v1-beta is "minted-fresh K_principal-generation N+1 on rotation; old DAK-issued device keys can't read new generation" (per M1b § U20 / L9-A4). Not CGKA; just key-generation-counter discipline.

### §6.4 Singleton-set (single DID)

**Recommendation:** **N/A.** No group; no key agreement needed.

### §6.5 Multi-recipient HPKE seam (Drop bundles)

**Recommendation:** **Plain HPKE-mode-base + per-recipient stanza (L9-A1 `HpkeMultiBase`).** This is already the planned shape. Not CGKA-shaped.

### §6.6 Per-use-case verdict summary

| Use-case | Primitive | Why |
|---|---|---|
| FORK-ONLY rotation | Multi-recipient HPKE + K_Atrium-random + fork | Q3 Option D ratified |
| ADMIN-KICK-EPOCH | Fold into FORK-ONLY | Semantic-identical; no new primitive |
| DeviceMesh add/remove | DAK-derived per-device keys + K_principal-generation counter | Small-N; PCS via generation counter |
| Singleton-set | N/A | No group |
| Multi-recipient sealing | HPKE-mode-base + L9-A1 stanza | Already planned |

**Pattern:** ZERO use-cases at v1-beta require a true CGKA. The PCS use-case in §6.3 is handled by generation-counter not CGKA.

---

## §7 Composition with MembershipSet kind variants

Per M2's typed-variants (Atrium / DeviceMesh / SingleDevice): does the same CGKA work for all 3 kinds?

### §7.1 Same primitive across kinds?

**YES** — multi-recipient HPKE composes across all 3 kinds:
- Atrium = multi-recipient HPKE to N principal-DIDs (each principal then re-seals to its DeviceMesh)
- DeviceMesh = multi-recipient HPKE to M device-DIDs under one principal
- SingleDevice = degenerate multi-recipient HPKE with N=1

### §7.2 Different rotation semantics per kind?

**YES** — kinds get different rotation semantics:
- Atrium: FORK-ONLY (per Q3 Option D)
- DeviceMesh: K_principal-generation counter (per M1b § U20)
- SingleDevice: N/A (no rotation)

### §7.3 Codepoint-reserve implication

Reserve `MembershipSetKind::AtriumWithCGKA` (singular) at v1-beta-CODEPOINT-RESERVE. If a future need requires DeviceMeshWithCGKA, reserve as additive codepoint then. **DO NOT reserve multiple CGKA-shape variants speculatively at v1-beta** — additive codepoints are the named pattern.

### §7.4 Cross-kind sealing

The same K_Atrium can be sealed to BOTH per-principal stanzas AND per-device stanzas (recipient-types are mixed). This is exactly the L9-A1 multi-recipient stanza design — `EnvelopePayload::HpkeMultiBase { stanzas: Vec<HpkeStanza>, ... }` where each stanza has its own recipient-pubkey.

---

## §8 Long-term strategic considerations

Per Task 7 + Ben's framing "we're going to be the production-validator for a handful of other things."

### §8.1 Be-the-strongest-current-CGKA vs be-elegant-Benten-specific

Ben framed two non-exclusive strategic stances:
- **(α) Adopt the strongest available CGKA** = MLS-PQ when ready; Benten benefits from ecosystem audit + interop.
- **(β) Be elegant + Benten-specific** = Q3 Option D + Atrium-fork semantic; smaller surface; deeper Benten-shape fit; less audit-bandwidth.

These are not at war. The Q3-Option-D shape composes with future-MLS-PQ-adoption: if Benten ever ships `MembershipSetKind::AtriumWithCGKA`, the existing Q3 Option D shape stays valid for `MembershipSetKind::AtriumFork`. The two shapes coexist as additive codepoints.

### §8.2 Benten as Cryptree production-validator

**Value proposition:** ~15-25 wave-days for a Rust Cryptree port + Benten integration; produces an open-source Rust Cryptree library + a real-world production deployment record. Gift to ecosystem.

**Counter:** Cryptree is solving a problem Benten doesn't have at v1-beta. Spending 15-25 wave-days for strategic-statement value at the cost of v1-beta timeline = not worth it. **DEFER to Phase-5+ when per-Subgraph fine-grained-read-token feature appears.**

### §8.3 Benten as MLS-PQ early-adopter

**Value proposition:** ~35-65 wave-days over 12-18 months to adopt OpenMLS-PQ; Benten validates the cryptographer-audit at the integration layer + helps stabilize a draft that's currently "Revised I-D Needed."

**Counter:** the draft isn't stable enough to validate against. Early-adopting now means churn-cost when the draft settles. Re-assess at Phase-4-Meta-Composing OR when one of {RFC publishes, big production deployment lands}.

### §8.4 Benten as DCGKA / p2panda-encryption validator

**Value proposition:** p2panda-encryption is the closest existing thing to "a real CGKA for the local-first / CRDT world Benten lives in." Pending Radically Open Security audit. Adopting p2panda-encryption as a dependency would (a) give Benten real-CGKA-PCS-FS properties on a Benten-shaped substrate, (b) support a small team in our ecosystem niche, (c) commit Benten to a small-team dependency.

**Counter:** small-team dependency risk; not yet 1.0; license needs verification; FS still over-delivers vs Ben's fork-semantic. NOT a v1-beta pick. Worth a re-assessment in Phase-4-Meta-Composing.

### §8.5 Long-term ecosystem alignment recommendation

**Position Benten as "MLS-PQ-future-compatible, Atrium-fork-semantic-native."** That is: Benten ships its own shape (Q3 Option D + Atrium-fork) at v1-beta because the semantic is Benten's own, while reserving the additive codepoint to plug in MLS-PQ when stable for the use-cases that want PCS. This is the maximum-optionality move + the lowest-risk-path.

---

## §9 Final recommendation + R0 plan-doc implications

### §9.1 Final recommendation

**PRIMARY: KEEP Q3-Option-D-aligned no-CGKA shape for v1-beta. RENAME "CGKA-LITE" to "no-CGKA multi-recipient HPKE + fork-on-admin-kick" to stop accidentally promising PCS/FS properties we don't deliver. Reserve `MembershipSetKind::AtriumWithCGKA` codepoint at v1-beta.**

**SECONDARY: Plan to re-evaluate at Phase-4-Meta-Composing OR upon any of the following triggers: (a) `draft-ietf-mls-pq-ciphersuites` reaches RFC, (b) OpenMLS ships PQ in its stable main release with an IANA-assigned codepoint, (c) Benten has a use-case requiring PCS that the fork-semantic genuinely cannot satisfy.**

**TERTIARY: NAMED-defer Cryptree to Phase-5+ for a hypothetical per-Subgraph fine-grained-read-token feature (NOT MembershipSet).**

### §9.2 R0 plan-doc implications

The MembershipSet R0 plan should:

1. **Reframe the "CGKA-LITE" naming.** The term promises properties (continuous, agreement) the design doesn't deliver. Use "MultiRecipientSealing" or "FoldedMembershipSealing" or similar.
2. **Codify Q3 Option D §2.7 fork-on-rotate as the default Atrium rotation semantic.** Already ratified pending MembershipSet framing.
3. **Add an explicit "ADMIN-KICK-EPOCH = fork-on-kick with attestation" sub-section.** No new primitive; just a name for the operation.
4. **Reserve `MembershipSetKind::AtriumWithCGKA` codepoint** at v1-beta-CODEPOINT-RESERVE without specifying which CGKA. Leave the choice (MLS-PQ vs DCGKA vs something-else) to the Phase-N implementation phase.
5. **Add a NAMED-deferred row** in `docs/V1-FROZEN-INTERFACE-DEFERRED.md` for "CGKA-shape MembershipSet variant — re-evaluate at Phase-4-Meta-Composing per M5 triggers."
6. **Add to `docs/SECURITY-POSTURE.md`** a Compromise entry making the semantic explicit: "Benten Atrium MembershipSet does NOT provide forward-secrecy or post-compromise-security against removed members — the design semantic is fork-on-kick + recipient-set-exclusion, which means a kicked member retains their pre-kick K_Atrium and can read pre-kick content forever but cannot read post-kick content. Use-cases requiring stronger semantics (PCS against kicked members) are deferred to the post-v1-beta CGKA-variant MembershipSet kind."
7. **Update M2's typed-variants** if M2 has not already done so: `MembershipSetKind::{AtriumFork, DeviceMesh, SingleDevice}` at v1-beta; `AtriumWithCGKA` reserved.

### §9.3 What this DOES NOT preclude

- Adopting MLS-PQ later (additive codepoint).
- Adopting DCGKA / p2panda-encryption later (additive codepoint).
- Authoring + shipping a Rust Cryptree port for Phase-5+ per-Subgraph read tokens.
- Re-assessment at Phase-4-Meta-Composing per the gate triggers.

### §9.4 What this DOES preclude

- Shipping a "CGKA" property without the actual CGKA semantics (truth-in-naming).
- Committing v1-beta wire format to MLS framing (which would lock us into a wrap-in-wrap shape).
- Spending v1-beta wave-days on Cryptree-port-as-strategic-statement.

---

## §10 NAMED-deferred alternatives

Per HARD RULE (no "later" disposition): each deferred alternative cites a SPECIFIC destination + trigger.

| Alternative | Disposition | Destination | Trigger |
|---|---|---|---|
| MLS RFC 9420 classical adoption | OUT-OF-SCOPE for v1-beta | Phase-4-Meta-Composing re-evaluation | Production-grade PCS use-case appears + Benten accepts MLS framing cost |
| MLS-PQ draft adoption | BELONGS-NAMED-NOW | `docs/V1-FROZEN-INTERFACE-DEFERRED.md` new row "CGKA-shape MembershipSet variant" | Draft reaches RFC OR OpenMLS ships PQ in stable main with IANA codepoint OR big production deployment lands |
| Cryptree (Peergos-style) for per-Subgraph read tokens | BELONGS-NAMED-NOW | `docs/future/phase-4-backlog.md` (new §3.N "per-Subgraph fine-grained read tokens") with Phase-5+ target | Feature need emerges for per-Subgraph access-grant-with-cascade |
| DCGKA / p2panda-encryption adoption | OUT-OF-SCOPE for v1-beta | Phase-4-Meta-Composing re-evaluation | p2panda-encryption ships 1.0 + audit completes + small-team dependency risk acceptable |
| Signal Sender Keys | DISAGREE (license incompatible + dominated by MLS) | N/A | N/A |
| TreeKEM standalone | DISAGREE (dominated by MLS) | N/A | N/A |
| OPRF-GKA | OUT-OF-SCOPE (no v1-beta use-case) | Future research direction for threshold-admin | Threshold-admin (M-of-N) becomes a real feature ask |
| Saltpack / age multi-recipient | DISAGREE (already subsumed by HPKE multi-recipient stanza) | N/A | N/A |
| Pond / Briar / Cwtch | DISAGREE (dominated) | N/A | N/A |
| Authoring CGKA-LITE-as-thin-CGKA-wrapper | DISAGREE (the "CGKA" framing is wrong; rename to "MultiRecipientSealing" or similar) | R0 plan-doc rename action | N/A |

---

## §11 Self-assessment + confidence per finding

| Finding | Confidence | Confidence rationale |
|---|---|---|
| §1 PRIMARY recommendation (keep Q3 Option D; reframe naming) | **HIGH** | Grounded in Q3 ratification + Ben's 2026-05-27 fork-semantic + complete elimination of CGKA candidates for the actual v1-beta use-cases (§6). |
| §1 SECONDARY recommendation (MLS-PQ as eventual home) | MEDIUM | The "MLS is the de-facto standard" pattern is clear; whether MLS-PQ specifically settles within Phase-4-Meta-Composing timeline is genuine guesswork. |
| §1 Cryptree-not-CGKA correction | **HIGH** | Verified across multiple primary sources (Grolimund 2006 paper, Peergos docs, Willow spec, Cryptree literature). Ben's framing conflated two things. |
| §2.6 DCGKA / p2panda-encryption assessment | MEDIUM | p2panda-encryption's audit status + 1.0 timing not freshly verified (last datapoint Feb 2025 announcement); could be more or less ready than estimated. |
| §3 Cryptree deep-dive | MEDIUM-HIGH | Cryptree paper itself is well-understood (2006, 20-year track record). Peergos integration details are partial. |
| §4 MLS-PQ "NOT beta-production-ready" verdict | **HIGH** | IETF state ("Revised I-D Needed") + OpenMLS using non-draft ciphersuite + zero production deployments triangulate cleanly. |
| §5 trade-off matrix | MEDIUM | Letter grades are inherently judgment calls; I'd defend each cell but reasonable cryptographers could disagree on individual letters. |
| §6 per-use-case recommendations | MEDIUM-HIGH | Logic chain is clean; §6.3 DeviceMesh PCS-via-generation-counter is the most-arguable single claim. |
| §7 same-primitive-across-kinds | MEDIUM | Speculative — M2's typed-variants are still in flight; my projection is grounded in M1b but could collide with M2's actual proposal. |
| §8 long-term ecosystem alignment | MEDIUM | Strategic positioning is genuinely opinionated; Ben may weigh strategic-validator value higher than I do. |
| §9.2 R0 plan-doc implications (action list) | MEDIUM-HIGH | Concrete; each item is verifiable + the rename action is mechanically simple. |

### §11.1 Where I'm most likely wrong

- **DCGKA / p2panda-encryption may be MORE ready than I scored.** If the Radically Open Security audit completed in 2025 and they shipped 1.0, the "small-team dependency risk" is less acute. Worth a fresh check before committing the rec.
- **MLS-PQ standardization may move faster than I project.** If IETF closes -04 → -05 → WGLC in the next 3-6 months, my "12-18 month settlement" window is wrong. Worth re-checking quarterly.
- **Ben may legitimately want the strategic-validator stance ENOUGH that the production-cost analysis is the wrong frame.** If Ben values "be the strongest current available CGKA setup" as a vision-statement irrespective of v1-beta delivery cost, my recommendation flips toward adopting OpenMLS-X-Wing now with eyes open to the churn-cost. Surface decision per §1.1 prediction-confirm pattern.

### §11.2 Where I'm most likely right

- The Q3-Option-D-fork-semantic ≠ CGKA-semantic distinction. The crypto literature is unambiguous on this.
- The Cryptree-is-not-a-CGKA correction. Primary-source-verified.
- The "MLS-PQ draft is NOT beta-production-ready per Ben's specific framing." IETF state is publicly verifiable.

---

## §12 Citations

[1] OpenMLS GitHub + docs — `https://github.com/openmls/openmls` ; v0.8.1 released 2026-02-13; supports MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519 (mandatory-to-implement), MLS_128_DHKEMP256_AES128GCM_SHA256_P256, MLS_128_DHKEMX25519_CHACHA20POLY1305_SHA256_Ed25519. MIT-licensed. Maintained by Phoenix R&D + Cryspen.

[2] `draft-ietf-mls-pq-ciphersuites-04` — `https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/` ; revision -04 published 2026-03-18; state "Revised I-D Needed"; expires 2026-09-20; nine ciphersuites; not in WGLC.

[3] Cryspen "Post-Quantum OpenMLS" — `https://cryspen.com/post/pq-openmls/` (April 2024). Introduces MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519 (suite 0x004D) using X-Wing KEM + ChaCha20-Poly1305; uses formally-verified libcrux ML-KEM + x25519. No IANA codepoint.

[4] Wire "Wire welcomes the publication of MLS as RFC 9420" + Webex / Discord / Google Messages / Apple Messages production deployment notes (search-aggregated; multiple secondary sources).

[5] `messaginglayersecurity.rocks/mls-pq-ciphersuites` (HTML rendering of the IETF draft).

[6] OpenMLS Blog PQ tag — `https://blog.openmls.tech/tags/pq/` + OpenMLS v0.8.1 README.

[7] RFC 9420 (MLS Protocol) + RFC 9750 (MLS Architecture) — `https://www.rfc-editor.org/rfc/rfc9420.html` + `https://www.rfc-editor.org/rfc/rfc9750.html`.

[8] `draft-ietf-mls-combiner-02` Amortized PQ MLS Combiner — `https://datatracker.ietf.org/doc/draft-ietf-mls-combiner/` ; benchmarking paper eprint 2026/034.

[9] Weidner / Kleppmann / Hugenroth / Beresford "Key Agreement for Decentralized Secure Group Messaging with Strong Security Guarantees" (ACM CCS 2021) — `https://eprint.iacr.org/2020/1281` + `https://martin.kleppmann.com/2021/11/17/decentralized-key-agreement.html`.

[10] Grolimund / Meisser / Schmid / Wattenhofer "Cryptree: A Folder Tree Structure for Cryptographic File Systems" (IEEE SRDS 2006) — `https://tik-db.ee.ethz.ch/file/146566189b90f952b8ab1dcf98010781/srds06.pdf`.

[11] Peergos Cryptree docs — `https://book.peergos.org/security/cryptree.html`.

[12] CMU CyLab + TechXplore coverage of DCGKA (2021).

[13] p2panda "Local-First group- and message encryption" (Feb 2025) — `https://p2panda.org/2025/02/24/group-encryption.html`.

[14] p2panda-encryption crate — `https://crates.io/crates/p2panda-encryption`.

[15] Nostr NIP-EE / NIP-104 (MLS-over-Nostr) — `https://nips.nostr.com/EE` + `https://github.com/nostr-protocol/nips/pull/1427` + Marmot Protocol / White Noise reference impls.

[16] mls-rs (AWS Labs) — `https://github.com/awslabs/mls-rs` + `https://awslabs.github.io/mls-rs/`. 100% RFC 9420 conformance; Apache-2.0. Not 3rd-party-audited per their README.

[17] Willow protocol Willow'25 spec — `https://willowprotocol.org/specs/willow25/`. Uses Ed25519 + ChaCha20-Poly1305 at transport; leaves application-layer encryption + key management to the application. No CGKA spec.

[18] X-Wing KEM draft `draft-connolly-cfrg-xwing-kem-10` — `https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/` ; CFRG draft; not IETF-endorsed; revision -10 published 2026-03-02; expires 2026-09-03.

[19] libsignal (Signal Messenger) — `https://github.com/signalapp/libsignal`. AGPLv3.

[20] Sibling cataloger outputs M1a / M1b / M1c — `git show origin/phase-4-meta-core/membership-set-cataloger-{m1a-multi-device-sync,m1b-atrium-membership-sharing,m1c-key-management}:.addl/phase-4-meta/membership-set-cataloger-*.md`. Specifically M1b §1.2(k) Ben's 2026-05-27 forkability ratification + §3.3 "MLS-PQ-derived CGKA / Fork-Resilient CGKA" deferral.

---

**End of M5 candidate survey.**
