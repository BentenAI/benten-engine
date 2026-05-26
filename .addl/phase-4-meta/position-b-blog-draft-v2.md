# Shipping PQ-hybrid signatures today, on purpose

*Benten Engine blog draft v2 — pre-publication. Authored for the v1-beta wire-format freeze.*

---

## 1. Lede

Benten Engine's v1-beta default signature is **LAMPS Composite ML-DSA** (`id-MLDSA65-Ed25519-SHA512`, OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20)[^lamps-oid]. Every UCAN delegation, plugin manifest, Atrium drop, sync merge proof, and device attestation we sign at v1-beta is signed with this construction. We did not have to do this for v1-beta. Most of the systems we sit next to — iroh, Signal, MLS, libp2p, Veilid, Apple iMessage — kept their long-lived signing keys classical and have publicly stated their reasoning for doing so. We think those teams are right for their systems. We think we are right for ours.

This post explains what we shipped, why we shipped it now instead of later, and what the construction does and does not give us. The argument hinges on a structural claim about the *kind* of signatures Benten makes — long-lived signatures that delegate capability into automated trust chains without per-use human review. We may be wrong, and the honest caveats in §5 are load-bearing rather than perfunctory.

We are not first to argue for this construction — the IETF OpenPGP working group is concurrently mandating the same composite (Ed25519 + ML-DSA-65) in draft-ietf-openpgp-pqc-17[^openpgp-pqc], Sequoia PGP has publicly committed to ship-on-publication, and Sigstore + Trail of Bits are independently retooling on the same threat-model[^sigstore]. We are early-shipping in a converging cohort. The implementation is at `crates/benten-crypto-suite/` and published; see §2.

[^lamps-oid]: IANA OID assignment per draft-ietf-lamps-pq-composite-sigs-19 §8.1.2. The draft is in the RFC Editor Queue as of May 2026: <https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/>
[^openpgp-pqc]: draft-ietf-openpgp-pqc-17: "A conformant implementation MUST implement ML-DSA-65+Ed25519 and ML-KEM-768+X25519." <https://datatracker.ietf.org/doc/html/draft-ietf-openpgp-pqc>
[^sigstore]: Sigstore PQC post: <https://blog.sigstore.dev/post-quantum-2025/>

---

## 2. The library — install + 30-line example

The construction we describe in this post ships as **`benten-crypto-suite`**: a codepoint-dispatched PQ-hybrid signature crate, in-tree at [`crates/benten-crypto-suite/`](https://github.com/BentenAI/benten-engine/tree/main/crates/benten-crypto-suite) and published as a derivative artifact to both crates.io and npm. Both packages are byte-equivalent to the in-tree source; the engine itself depends on the in-tree path.

```sh
cargo add benten-crypto-suite
npm install @bentenai/crypto-suite
```

Rust quickstart:

```rust
use benten_crypto_suite::{KeyPair, SigCodepoint, Signer, Verifier};

fn main() {
    // Mint a hybrid Ed25519 + ML-DSA-65 keypair at codepoint 0x0001
    let kp = KeyPair::generate(SigCodepoint::Hybrid_Ed25519_MLDSA65)
        .expect("ML-DSA RNG should succeed");

    let msg = b"benten-crypto-suite quickstart";
    let ctx = b"benten-blog-example-2026-05";

    // Sign: produces a 3373-byte hybrid signature
    let sig = kp.signer().sign(msg, ctx).expect("sign should succeed");
    assert_eq!(sig.len(), 3373);

    // Verify with the public key
    let pk = kp.public_key();
    let verifier = pk.verifier();
    assert!(verifier.verify(msg, ctx, &sig).is_ok());

    // Tampered message MUST reject
    let tampered = b"benten-crypto-suite quickstart!";
    assert!(verifier.verify(tampered, ctx, &sig).is_err());

    // Different context MUST reject (domain separation)
    let other_ctx = b"benten-blog-different-ctx";
    assert!(verifier.verify(msg, other_ctx, &sig).is_err());
}
```

Test vectors, KAT corpus (FIPS 204 ACVP + LAMPS draft Appendix E), property tests for non-separability + cross-component forgery rejection + domain-separator integrity, README with the LAMPS Security Considerations §9.2 caveats reproduced verbatim, and `SECURITY.md` documenting the bug-report channel + response SLA (matching the engine's SLA) are all in-tree alongside the crate.

We subscribe to RustSec advisories for `ml-dsa`, `libcrux-ml-dsa`, and `fips204`, and have committed publicly to triage any new advisory within 48 hours.

---

## 3. Why we built it — the class of signatures this is for

The class of signatures we mean is precise: **long-lived signatures that delegate capability into automated trust chains without per-use human review.**

In Benten this includes UCAN delegations (signed `{issuer, audience, capabilities, validity}` tokens trusted by the next service in an automated chain), plugin manifests (signed `{requires, shares}` envelopes that gate runtime authority years after install), Atrium drops (signed content bundles any peer in the mesh may re-validate decades later), and sync merge proofs + device-attestation envelopes (same structural shape). For this class, three things are simultaneously true that are usually not all true together: the artifact may persist for *years to decades*; the signature is re-validated *by automation*, not by a human; and a successful forgery of one artifact unlocks downstream authority the issuer never granted.

We assume an adversary capable of breaking classical Ed25519 will eventually exist, at some unknown date, against the corpus of signatures harvested before that date. We do *not* claim the date is known, urgent, or imminent. We do claim the cost of being wrong is asymmetric *for us*: re-signing every UCAN, plugin manifest, and drop in the Benten corpus after a cryptographically-relevant quantum computer materializes is the kind of fork that decentralized projects either fail at or split over. Arweave's RSA-PSS lock-in is the cautionary example[^arweave].

**iroh's stated reasoning.** Their May 2026 post says, of post-quantum signatures specifically:

> While post-quantum key exchange algorithms are standardized and come with relatively modest overhead, post-quantum signature algorithms are the subject of active research. All current options have significant downsides regarding computational cost, key size or signature size. So there is no industry consensus yet which one to use. There is also less urgency because there is no harvest-now-decrypt-later equivalent for signatures.[^iroh]

Both reasons hold *for iroh*. The second — no industry consensus — is real (we picked LAMPS Composite ML-DSA precisely because it is the *emerging* consensus the OpenPGP-PQC RFC will ratify; "emerging," not "settled"). The first — no HNDL equivalent for signatures — we extend rather than dispute: it is correct for a transport-identity key authenticating a *live* handshake, and it does not apply to a UCAN that authorizes an automated capability lookup in 2046. iroh's `EndpointId` IS a long-lived Ed25519 identity (iroh-blobs is itself content-addressed long-term persistence, with BLAKE3 giving blobs integrity without per-blob signatures; the long-lived classical signing surface in iroh is narrowly `EndpointId`), so our argument *does* apply to that one surface. Reasonable people can read it as friendly disagreement on that single point. We are not saying iroh got it wrong. We are saying we read the same evidence and made a different defensible call for the kind of artifact we sign.

**Signal's stated reasoning.** PQXDH §4.8 explicitly considers adding a post-quantum identity-signature and rejects it:

> It is tempting to consider adding a post-quantum identity key that Bob could use to sign the post-quantum prekeys… but it does not provide mutual authentication. Bob does not have any cryptographic guarantee about who he is communicating with. The post-quantum KEM and signature schemes being standardized by NIST do not provide a mechanism for post-quantum deniable mutual authentication, although this can be achieved through the use of a post-quantum ring signature or designated verifier signature. We urge the community to work toward standardization of these or other mechanisms that will allow deniable mutual authentication.[^pqxdh]

Signal's reason is *deniability* — a load-bearing privacy property that XEdDSA's designated-verifier semantics deliver. This is not the wire-size folklore often attributed to Signal's choice; the spec text is explicit. And it is a property Benten **explicitly does not want**: plugin manifests, UCAN delegations, and drops are intended to be third-party non-repudiable — that is the point of signing them. Signal's reasoning and ours both serve their respective systems. They do not contradict; they apply to disjoint design goals.

[^arweave]: Arweave's permaweb commits to ~200-year persistence on RSA-PSS classical signatures with no public PQ roadmap, on a chain whose ~5 PB of signed history cannot be migrated without a fork-class event. We treat this as evidence for picking PQ-hybrid *before* a legacy you cannot migrate exists.
[^iroh]: Rüdiger Klaehn, "Iroh post-quantum key exchange," n0 / iroh.computer blog, May 19, 2026, <https://www.iroh.computer/blog/iroh-post-quantum-handshakes>. Quoted verbatim from the section "What about signatures?"
[^pqxdh]: Signal Foundation, "The PQXDH Key Agreement Protocol," §4.8 "Active quantum adversaries," <https://signal.org/docs/specifications/pqxdh/>. Quoted verbatim.

---

## 4. What we shipped — and exactly what the combiner does and doesn't give us

### The construction

We ship **LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512`** at multicodec/varsig codepoint `0x0001`, OID `1.3.6.1.5.5.7.6.48`, IANA early-allocated 2025-10-20. The construction is defined in `draft-ietf-lamps-pq-composite-sigs-19`, currently in the RFC Editor Queue. The signature size is **3373 bytes** (ML-DSA-65 component 3309 bytes + Ed25519 component 64 bytes), and the message both halves sign is `M' := Prefix || Label || len(ctx) || ctx || PH(M)` where `Prefix = "CompositeAlgorithmSignatures2025"` (the static domain-separation prefix that gives the construction its weak-non-separability property; see below) and `PH` is SHA-512[^lamps-construction].

### Why LAMPS, not the alternatives

We picked LAMPS because it is the WG-blessed Schelling point: BouncyCastle 1.80+ supports Composite ML-DSA in CMS SignedData; OpenSSL 3.5 has partial support; AWS KMS supports PQ signatures; the OpenPGP-PQC RFC will mandate the *same* composite Ed25519 + ML-DSA-65 construction; ANSSI specifically endorses concat-style combiners[^anssi]. When a Benten-signed UCAN someday needs to interoperate with a PKIX certificate workflow or a JOSE-wrapped composite signature, that interop is structural for LAMPS and bespoke for any of the alternatives.

The alternatives we considered and did not pick: **draft-prabel-cfrg-suf-hybrid-sigs §3**[^prabel] is the cleanest SUF-CMA-preserving combiner (binds `s2 = Sign(skPQ, M' || s1)`; foundational Bindel-Hale 2023[^bindel-hale]) — but it is an individual CFRG submission, not WG-adopted, no IANA codepoint, no production Rust impl, no test vectors. **Bird-of-Prey (Janneck, IACR 2025/1844)**[^bop] presents practical SUF-CMA-preserving combiners for EdDSA + ML-DSA with sizes equal-to-or-smaller-than concatenation; to appear at EUROCRYPT 2026[^bop-conf]. One peer-reviewed publication still in proceedings prep, one author, no Rust reference impl, no test vectors — locking it into a v1-beta wire freeze would put fresh academic crypto on the critical path of a permanent commitment. **Silithium (IACR 2025/2059)**[^silithium] requires the ML-DSA "external mu" API which no Rust crate exposes today and switches EdDSA→EC-Schnorr.

Codepoint `0x0002` is reserved for a SUF-CMA-preserving construction to be added when one clears WG adoption **and** independent impl-audit **and** conference proceedings without errata. Additive, not migratory.

### The formal-property scope — what this construction provides, verbatim

The LAMPS draft is unusually explicit about what its construction does and does not provide, and we reproduce its own language rather than paraphrase. From §9.2 ("Security Considerations"):

> Composite ML-DSA will be EUF-CMA secure if at least one of its component algorithms is EUF-CMA secure and the pre-hashed message representative PH is collision resistant.[^lamps-9-2]

From §9.2.2:

> While some of the algorithm combinations defined in this specification are likely to be SUF-CMA secure against classical adversaries, none are SUF-CMA secure against a quantum adversary. **Composite ML-DSA is NOT RECOMMENDED for use in applications where it has not been shown that EUF-CMA is acceptable.**[^lamps-9-2-2]

This matters and it is honest to surface it loudly: the construction is **EUF-CMA secure** (existential unforgeability: an adversary cannot produce a valid signature on a new message), and it provides **weak non-separability** via the static `CompositeAlgorithmSignatures2025` prefix — passive downgrade-by-stripping leaves cryptographic evidence (the prefix tells the verifier that the message-being-stripped was composed, not single-algorithm). It is **not SUF-CMA secure**: an adversary who has seen one valid composite signature on a message *may* be able to produce a different valid composite signature on the same message from the same key. And per the draft itself, it is **weakly non-separable** but not strongly non-separable (per draft-ietf-pquip-hybrid-signature-spectrums[^pquip-spectrums]).

### Why this is acceptable for Benten — the three-layer decomposition (Inv-15)

For the class of signatures in §3 to be safe under EUF-CMA-only, the application layer above the signature must not silently rely on SUF-CMA-equivalent properties. The hazard pattern would be: an application uses the *signature CID* as the stable identity or revocation key for the *signed payload*. Under SUF-CMA-only that would let an adversary mint a different valid signature on the same payload and produce a new "identity" or bypass the old revocation.

Benten avoids this by structural decomposition, codified as project-wide invariant **Inv-15**:

- **Identity = canonical-payload-CID.** The CID of a UCAN, a plugin manifest, a drop, or a device attestation is the CID of the canonical CBOR-encoded *payload body*, never the CID of the `{payload, signature}` bundle. A different valid signature over the same payload produces the same identity.
- **Authentication = codepoint-dispatched signature.** The signature is a separate artifact attached to the payload, dispatched at verify time by its varsig codepoint. The verify path is fail-closed on unknown codepoints. The signature can rotate, be re-issued under a future codepoint, or carry multiple attestations without changing identity.
- **Revocation = semantic tuple.** UCAN revocation refers to `(issuer, audience, capability-set, validity-window, payload-CID)`, not to the CID of any particular signed bundle. This matches the UCAN spec's revocation model (UCAN.xyz revocation spec: "Revoked delegation should be referenced by its canonical CID" — the canonical *payload* CID[^ucan-rev]) and means that even under a SUF-CMA-breaking adversary, the revocation set captures the semantic authority an issuer wanted to revoke, not an accidental shortcut on a particular signature representation.

This decomposition closes the EUF-CMA-vs-SUF-CMA gap for our use cases at the application layer, where it should be closed regardless of which signature algorithm we ship — because the next algorithm migration (NF-1 PQ⊕PQ at `0x0004` when ML-DSA-65 + SLH-DSA component impls mature) inherits the same architecture without re-keying any UCAN, plugin, or drop identity. Inv-15 is documented in `docs/INVARIANT-COVERAGE.md` and tested at the relevant code surfaces; we explicitly invite implementation review.

[^lamps-construction]: draft-ietf-lamps-pq-composite-sigs-19 §3 (Composite Signature Algorithm Definition) and §4 (Algorithm Identifiers). The Prefix is the ASCII string `CompositeAlgorithmSignatures2025` (hex `436F6D706F73697465416C676F726974686D5369676E61747572657332303235`).
[^prabel]: draft-prabel-cfrg-suf-hybrid-sigs-01, <https://datatracker.ietf.org/doc/draft-prabel-cfrg-suf-hybrid-sigs/>. Boilerplate (verbatim): "This Internet-Draft is not endorsed by the IETF and has no formal standing in the IETF standards process."
[^bindel-hale]: Bindel & Hale, "A Note on Hybrid Signature Schemes," IACR ePrint 2023/423, <https://eprint.iacr.org/2023/423>.
[^bop]: Janneck, "Bird of Prey: Practical Signature Combiners Preserving Strong Unforgeability," IACR ePrint 2025/1844, <https://eprint.iacr.org/2025/1844>.
[^bop-conf]: To appear in EUROCRYPT 2026 LNCS proceedings (Springer chapter: <https://link.springer.com/chapter/10.1007/978-3-032-25317-0_8>).
[^silithium]: Devevey, Guerreau, Roméas (PQShield), "Compact, Efficient and Non-Separable Hybrid Signatures," IACR ePrint 2025/2059, <https://eprint.iacr.org/2025/2059>.
[^anssi]: ANSSI position on hybrid-signature combiner constructions, cited via Synacktiv. ANSSI recommends concatenation as the only generic signature combiner.
[^lamps-9-2]: draft-ietf-lamps-pq-composite-sigs-19 §9.2, verbatim.
[^lamps-9-2-2]: draft-ietf-lamps-pq-composite-sigs-19 §9.2.2, verbatim. Emphasis as in the original draft.
[^pquip-spectrums]: draft-ietf-pquip-hybrid-signature-spectrums (PQUIP-WG, informational), <https://datatracker.ietf.org/doc/draft-ietf-pquip-hybrid-signature-spectrums/>. Defines WNS (Weak Non-Separability) and SNS (Strong Non-Separability) as distinct properties.
[^ucan-rev]: UCAN.xyz revocation specification, <https://ucan.xyz/revocation/>: "Revoked delegation should be referenced by its canonical CID."

---

## 5. Honest caveats — load-bearing

We are aware of nine specific places we could be wrong or could be hurt. We want every reader to see this list before they decide whether the argument convinces them.

**1. Cohort framing.** We are *first-shipping* in a converging cohort, not alone. draft-ietf-openpgp-pqc-17 mandates the same Ed25519 + ML-DSA-65 composite, with RFC publication targeted for H1 2026; Sequoia PGP committed ship-on-publication; Sigstore + Trail of Bits retooling on the same threat-model recognition; EBSI's `jwk_jcs-pub` (`0xeb51`) squat is governance-precedent for our private-codepoint approach. "First-shipping in a cohort" still means in front of the cohort — we will surface integration bugs downstream adopters benefit from us paying the cost of.

**2. Standards maturity.** LAMPS draft-19 is in the RFC Editor Queue but not yet a published RFC. The wire format is structurally stable (OID early-allocated; PASN.1 module frozen for IANA) but a post-AUTH48 editorial change is technically possible. We rely on RFC 7120's framework[^rfc7120] which explicitly contemplates pre-RFC production deployment when a WG has reached technical convergence. We track every revision through publication and commit to RFC-conform on publication.

**3. Implementation maturity.** Neither `ml-dsa` (RustCrypto) nor `libcrux-ml-dsa` nor `fips204` has had an independent third-party security audit comparable to `ed25519-dalek` or `ring`. We pin a single ML-DSA crate version, monitor the RustSec feed for all three, and our `SECURITY.md` commits to 48-hour triage of any new advisory. The independent audit of our chosen ML-DSA crate is committed as a gating exit criterion for the `v1-GM` tag (NF-2 / C-GM-AUDIT). We treat the present period as `v1-beta`, not GM, and we say so.

**4. Different threat model from messaging.** We are not arguing Signal, MLS, or iMessage should ship PQ-hybrid identity signatures. PQXDH §4.8 (quoted in §3) is explicit that Signal's reason for skipping PQ identity-sigs is **deniability** — a property structurally incompatible with what Benten signs. Both choices are correct under different design goals.

**5. Wire-size cost.** 3373 bytes per signature (~52.7× Ed25519). For Benten's signature density (one per UCAN, one per plugin manifest, one per drop, one per device attestation, one per merge proof — *not* one per byte of synced data) the absolute byte cost is small. For a deployment that signs per-handshake or per-message at scale (Cloudflare-class workloads) the cost would be meaningful. A 10-signature drop bundle adds ~33 KiB of signature material — significant for small payloads, negligible for large. The cost-tradeoff is shape-of-deployment-dependent.

**6. Implementation-vs-algorithm distinction.** The hybrid combiner protects against an *algorithmic* break of either component. It does **not** protect against implementation-side-channel leakage — `RUSTSEC-2025-0144`[^rustsec-0144] (Jan 2026, `ml-dsa` Decompose timing) and the Verification Theatre paper's 13 libcrux vulnerabilities[^vtheatre] (including two FIPS 204 verifier spec violations) are the recent reminders. The combiner is not the right tool against these; pinning + monitoring + the audit at GM is.

**7. Combiner formal-property scope (EUF-CMA-only honesty).** The construction is EUF-CMA secure and weakly non-separable. It is **not SUF-CMA secure**: LAMPS §9.2.2 verbatim, "Composite ML-DSA is NOT RECOMMENDED for use in applications where it has not been shown that EUF-CMA is acceptable." We have shown via Inv-15's three-layer decomposition (§4) that EUF-CMA is the right floor for Benten's signature semantics and that SUF-CMA-sensitive surfaces are closed at the application layer (identity = canonical-payload-CID; revocation = semantic tuple) rather than via a sig-CID shortcut. The right time to revisit the algorithmic floor is when a SUF-CMA-preserving construction (Bird-of-Prey / draft-prabel §3 / Silithium) clears WG adoption + independent impl audit + conference proceedings without errata. Codepoint `0x0002` is reserved for that addition; it will be additive, not migratory.

**8. Key-reuse hazard.** LAMPS §9.3 and §10.3 are explicit that the underlying ML-DSA-65 and Ed25519 keys MUST NOT be reused for any standalone or other-composite purpose, or the EUF-CMA guarantee collapses. `KeyPair::generate(SigCodepoint::Hybrid_Ed25519_MLDSA65)` mints the composite keypair as a single inseparable unit; component private keys are never exposed via Benten's public API. Implementers wrapping `benten-crypto-suite` MUST honor this restriction; the library README says so loudly.

**9. FIPS 140-3 scope.** Composite signatures are **explicitly outside** the FIPS 140-3 boundary in current NIST guidance — NIST left composition to IETF. For deployments requiring FIPS 140-3 module-bound signing (US federal, certain regulated industries), the LAMPS composite construction is not certifiable today as a single FIPS module. Classical Ed25519 at codepoint `0x0003` (for legacy interop) is still inside scope. The FIPS posture may evolve when NIST issues SP 800-227 or equivalent on composite signatures.

[^rfc7120]: Cotton, M., "Early IANA Allocation of Standards Track Code Points," RFC 7120, <https://www.rfc-editor.org/rfc/rfc7120>.
[^rustsec-0144]: RUSTSEC-2025-0144 (published 2026-01-27): `ml-dsa` Timing side-channel in Decompose, <https://rustsec.org/advisories/RUSTSEC-2025-0144.html>.
[^vtheatre]: "Verification Theatre," IACR ePrint 2026/192, <https://eprint.iacr.org/2026/192>.

---

## 6. The class of systems this argues about

The class boundary in §3 is load-bearing for whether this argument applies to a given system. We want to be explicit:

**In the class** (long-lived signatures delegating capability into automated trust chains without per-use human review): Benten (UCANs / plugin manifests / drops / merge proofs / device attestations); OpenPGP-PQC artifacts (draft-17 mandates the same composite); Sigstore receipts (retooling toward composite); hypothetically Willow attestations and other Atrium-class CRDT-sync systems.

**Out of the class — for system-specific reasons:**

- **git commits** — human-in-loop attention at install/merge (`git verify-commit` is human-gated). Classical Ed25519 commit-signing isn't wrong; it's a different shape.
- **Sigstore short-lived OIDC signatures** — ephemeral by design. The transparency-log binding is the long-lived artifact, and Sigstore is moving *that* to PQ-hybrid.
- **Filecoin consensus signatures** — sign blockchain rounds and storage proofs whose validity is gated by consensus, not by re-verification of historic signed artifacts decades later. What persists is the Merkle-committed chain state, not the signatures.
- **Arweave RSA-PSS** — *should* be in the class (~200-year persistence, classical, no PQ roadmap), but is locked into legacy by ~5 PB of signed history. A cautionary example for why picking the construction *before* the legacy exists matters.
- **TLS / QUIC / Signal-private 1:1 / messaging broadly** — ephemeral session sigs authenticating a *live* exchange. PQ-hybrid identity-signing is either irrelevant (handshake replaced) or actively orthogonal (PQXDH's deniability case).

The class boundary IS the load-bearing structural claim of this post. If we have it wrong — wrong inclusions, wrong exclusions, or wrong definition entirely — that is the critique we most want to hear.

---

## 7. What the industry is doing

The honest snapshot, May 2026 — long-lived classical signature surfaces vs the converging cohort:

| System | Long-lived sig surface | PQ-hybrid sig today? | Stated reasoning |
|---|---|---|---|
| iroh (n0) | `EndpointId` (Ed25519) | No (KEM is PQ-hybrid) | "no industry consensus yet… no HNDL equivalent for signatures"[^iroh] |
| Signal (PQXDH) | XEdDSA identity | No | §4.8 — deniability/designated-verifier requirement[^pqxdh] |
| Apple iMessage | CKV identity keys | No public statement | (none found) |
| MLS-default | ECDSA P-256 default | No (default profile) | (spec retains hybrid optionality) |
| libp2p / Veilid | Per-peer Ed25519 | No | (none published we found) |
| BouncyCastle 1.80+ | (Library) | Yes — Composite ML-DSA in CMS | Standards-following |
| OpenSSL 3.5 | (Library) | Partial composite | Standards-following |
| AWS KMS | Cloud HSM signing | PQ sig support live | Standards-following |
| OpenPGP-PQC (draft-17) | OpenPGP identity sigs | **MANDATED** (RFC H1 2026) | Same threat-model recognition as Benten |
| Sigstore (planned) | Code-signing receipts | Retooling toward composite | Trail of Bits: "signed today, deployed 20 years"[^sigstore] |
| Sequoia PGP | OpenPGP impl | Ship-on-publication committed | Tracks OpenPGP-PQC |
| EBSI | did:jwk identity infra | Squats `jwk_jcs-pub` (`0xeb51`) | Governance precedent |

The pattern is real both ways: deployed P2P and messaging systems keep long-lived signing keys classical today for substantive reasons; standards bodies and the implementations following them are simultaneously converging on composite ML-DSA + Ed25519 for the class of signatures *we* sign. The shipping cohort is forming (Benten + OpenPGP-PQC + Sigstore + EBSI's adjacent precedent). We are early in that cohort, openly an early adopter, not a leader.

The RFC 7120 framework supports pre-RFC production deployment of an IANA-early-allocated codepoint when the WG has converged on technical content[^rfc7120]. We are shipping under that framework, with the OID committed, the wire format frozen, the construction reproducible, and an explicit conformance-update commitment if the final RFC text shifts.

---

## 8. Where this leaves the standards bodies

Brief, in IETF-WG-precise vocabulary:

- **LAMPS WG** — draft-ietf-lamps-pq-composite-sigs-19 is in the **RFC Editor Queue**. We will file an adopter report **after** RFC publication (not during AUTH48), per RFC-process discipline. We will deposit Benten-derived test vectors into the LAMPS WG's vectors repository as soon as we have a stable pin against draft-19's PASN.1 module; this is the highest-leverage adopter contribution we can make.
- **JOSE WG** — draft-ietf-jose-pq-composite-sigs-01 is an active **early-stage WG document**. We are preparing an adopter-perspective comment on the Security Considerations section, specifically encouraging EUF-CMA-vs-SUF-CMA scope language mirroring LAMPS §9.2.2.
- **Multicodec maintainers (vmx, lidel, rvagg, MarcoPolo, achingbrain)** — PRs #400 and #403 land the container-form direction (pkix-pub / cose-key / jwk codepoints) rather than per-algorithm hybrid codepoints. We endorse this direction unconditionally; it is the right architectural answer to the PQ codepoint-proliferation problem and our v1-beta wire format is structured to interop with it.
- **W3C CCG (did-key-spec)** — Issues #70 / #74 are converging on "did:key allows any multikey value" without a spec amendment. We support that consensus; we will not advocate for a Benten-private multikey codepoint at the W3C surface.
- **CFRG** — draft-prabel-cfrg-suf-hybrid-sigs is an **individual submission**, not yet WG-adopted. When (if) CFRG opens an adoption call for a SUF-CMA-preserving hybrid signature draft, we will support adoption and we will mint codepoint `0x0002` accordingly. We are not in a position to drive that timeline.

We will not cross-link the LAMPS adopter report to this blog post; the LAMPS contribution stands on its own technical merit and the blog stands on its argument.

---

## 9. What we'd like to see

Three asks, in increasing scope:

1. **Critique of the §3 class boundary.** If the definition "long-lived signatures delegating capability into automated trust chains without per-use human review" is the wrong way to slice the problem, or if there is a system we have included or excluded incorrectly, we want to hear from you. The class boundary is structural; getting it wrong invalidates the argument.
2. **Critique of Inv-15 (the three-layer decomposition in §4).** The argument that EUF-CMA-only is sufficient for Benten depends on identity = payload-CID and revocation = semantic tuple at every site. If there is a code path or use case we have missed where Benten still depends on signature-CID semantics, we want to find it before v1-beta tag.
3. **An IACR-peer-reviewed analysis of the LAMPS composite-sig combiner** at the depth of X-Wing's analysis of the hybrid KEM (Barbosa et al., IACR Communications in Cryptology 2024-1-21). The depth-asymmetry between hybrid-KEM analysis and hybrid-sig analysis is real and worth closing publicly. We are not in a position to lead this work, but we are in a position to fund or sponsor it for our chosen construction if a credible team is interested.

Specific invitations: the **iroh team** for friendly disagreement on the EndpointId surface; the **LAMPS WG** for adopter feedback we may be missing; the **W3C CCG community** for DID-method critique; the **Signal cryptography team** for confirmation that we have characterized PQXDH §4.8 correctly (we cite it verbatim; we want to know if we have framed the deniability case fairly). Email is at `team@benten.ai`; issues at the GitHub tracker.

---

## 10. Acknowledgments

This post stands on three teams' work in particular.

The **multicodec maintainers** (vmx, lidel, MarcoPolo, rvagg, achingbrain) landed the container-form direction in PRs #400 and #403, which makes adopter approaches like ours possible without further codepoint-table proliferation.

The **Signal cryptography team** — both the PQXDH editors and the broader designed-verifier-signature design tradition — informed our framing of how content-addressed-persistent and deniability-anchored design goals can both be correct under different design constraints. PQXDH §4.8 is one of the clearest pieces of cryptographic design-rationale writing we have read, and our §3 quotation of it is meant in that spirit.

The **draft-ietf-openpgp-pqc working group** and the **Sequoia PGP team**, and the **Sigstore + Trail of Bits architectural-agility group**, for shipping or planning-to-ship the same construction independently on their own timelines. This work is in conversation with theirs; we are early-shipping in a cohort they are part of.

Finally, to **internal adversarial reviewers** for the pre-publication scrutiny that made this post substantially shorter, more precise, and more honest about its caveats than the first draft was. Specific text revisions in this v2 close findings from fifteen distinct adversarial lenses; remaining residual disagreements are flagged in the changelog accompanying this draft.

---

*If you are a cryptographer who wants to break our combiner, our wire format is at `crates/benten-crypto-suite/`, our KAT vectors are in-tree, and our SECURITY.md commits us to 48-hour triage on any advisory. We will not be defensive about findings. The signatures we sign at v1-beta will outlive us; we want them to outlive us correctly.*

— Benten Engine team, May 2026.
