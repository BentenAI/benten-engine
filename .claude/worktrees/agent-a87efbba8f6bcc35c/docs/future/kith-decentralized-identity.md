# Kith — decentralized identity & attestation system

**Status:** EXPLORATORY scope-stub. Named 2026-05-11 by Ben during Phase 4-Foundation R1 triage discussion (Q6 peer-DID rotation propagation). Working name; naming-research agent in flight to check trademark / ecosystem-conflict status. Final name TBD.

**Phase target:** **Phase 5+ or its own dedicated design-spike phase.** NOT Phase 4-Foundation. Touches Phase 7 Gardens/Groves (organizational attestation feeders) + Phase 6 AI assistant (contextual sharing decisions).

**Couples to:** Phase 4-Foundation's MVP rotation mechanism (`SelfRevocation` attestation + out-of-band new-key trust) is the floor — Kith extends it with richer semantics. The MVP shipping in Phase 4-Foundation does NOT block Kith design; Kith supersedes the MVP when it lands.

---

## § 1. Why this exists

Ben's framing 2026-05-11 evening conversation (Q6 thread): the basic peer-DID + RotationLog primitive from Phase 3 handles single-peer identity, but it's insufficient for the project's broader identity story. Specifically:

> "I was thinking we might need a decentralized ID system that relies on X has designated Y as Z with X being person you know, Y being person you're deciding whether or not to trust, and Z being their relationship. I guess it would kinda be like how you informally decide who is legit vs an imposter when deciding who to follow on social media by seeing who follows them or who your friends follow. We would want this to respect the ability of any individual to opt in/out maybe even on a per relationship basis rather than all my connections are private vs all are public. Then this could actually also be more than just personal relationships but in the future groups like gardens/groves, organizations like schools or certifying bodies, etc etc can each provide some kind of attestation about an individual (that all lives with that individual and the attester probably via UCAN) that the individual can then decide to contextually share whatever parts of for whatever circumstance (also rather than having to say all the connections that said I can share them are always shared (filterable/opt-out-able by the relationship attester and the individual trying to prove who they are to a new third party))."

This is a substantive extension of the basic DID model. It combines:

- **Relational attestations** — "X says Y is Z to me" as first-class graph data
- **Trust-graph traversal** — figure out whether to trust Y via your network of attesters
- **Per-relationship privacy controls** — the attester (X) and the subject (Y) BOTH have opt-in/opt-out per relationship; can be public, private, or scoped to specific contexts
- **Organizational attestations** — not just person-to-person; Gardens/Groves, schools, certifying bodies, professional associations all participate
- **UCAN-mediated contextual sharing** — attestations are signed delegations; subject chooses what subset to share with each verifier in each context
- **Filterability for proving identity in new contexts** — subject can share a curated subset of attestations to a third party without sharing their full attestation graph

---

## § 2. Differentiation from existing Phase 3 primitives

| Concern | Phase 3 (existing) | Kith (this proposal) |
|---|---|---|
| Single-peer identity | `did:key:...` minted per device + per user | Same — Kith builds on the existing DID primitive |
| Peer-to-peer trust | One-shot bootstrap (you exchange keys out-of-band; trust is binary "trusted peer" / "untrusted") | Trust derives from attestation graph; you can be "trusted-via-mutual-acquaintance" without direct bootstrap |
| Key rotation | Phase-4-Foundation MVP: SelfRevocation + out-of-band new-key trust | Web-of-trust assisted: rotations propagate with multi-peer attestation chains; no out-of-band needed for non-compromise rotations |
| Authorization | UCAN attenuation per cap | Attestations contextually shareable; ZK-style "prove this without revealing the full set" patterns possible |
| Organizational identity | Not modeled | First-class — schools, certifying bodies, Gardens/Groves issue attestations |
| Relationship semantics | Flat list of "trusted peers" | "X is Z to Y" — typed relational graph |

---

## § 3. Design questions (open)

These need dedicated design work in Phase 5+ or a design-spike phase:

1. **Attestation primitive shape.** How is "X attests Y is Z to me" represented as graph data? A `Relationship` Node with edges to X, Y, Z + a signature by X? Or a UCAN-style envelope with attenuation?
2. **Trust-graph traversal.** Given my attestation graph, how do I decide whether to trust Z? Path-counting? Reputation-weighted? AI-assisted (Phase 6+)?
3. **Per-relationship privacy controls.** What's the granularity? Public / private / per-context / per-verifier? UI primitive for the subject to manage?
4. **Organizational attestation issuance.** How do Gardens / Groves / schools / certifying bodies issue attestations? Each as a Kith peer + their attestations propagate through normal Atrium sync?
5. **Contextual sharing semantics.** When subject A wants to prove to verifier V "I'm a member of school S," A reveals the school's attestation to V — but what's the protocol? Selective disclosure? Zero-knowledge proofs (for future cryptographic strength)?
6. **Compatibility with W3C DID spec.** Does Kith register as a new DID method (`did:kith:...`)? Use existing methods? Spec compatibility for cross-ecosystem interop.
7. **Revocation semantics for attestations.** What happens when X wants to revoke "Y is Z to me"? Old attestation marked revoked; revocation propagates; verifiers refresh.
8. **Sybil resistance.** Without a centralized issuer, what prevents attestation farming (fake peers attesting each other)? Probably out-of-band reputation + Gardens-tier governance for high-stakes attestations.

---

## § 4. Phase 4-Foundation interaction

Phase 4-Foundation does NOT depend on Kith. Its MVP rotation mechanism (per Q6 ratification 2026-05-11 evening) is:

- **`SelfRevocation` attestation** — old key signs "this key is revoked as of $timestamp"; propagates via Atrium sync; peers reject content signed by revoked key after timestamp.
- **Out-of-band new-key trust** — each peer re-establishes trust with the rotated peer via the same side-channel they used for initial bootstrap.

When Kith lands (Phase 5+), it can SUPERSEDE the out-of-band step by providing web-of-trust attestation chains that establish new-key trust without manual side-channel rebuilding. The `SelfRevocation` mechanism survives Kith's introduction (still useful for adversarial scenarios where the rotating peer wants to cleanly disavow their old key).

---

## § 5. Related prior art for reference

(For Phase 5+ design-spike work to cite + compare against.)

- **W3C DID Core** — spec for decentralized identifiers; defines did:method registry
- **WebID / WebID-TLS** — early decentralized identity attempts
- **Spritely Goblins / ocap-style identity** — capability-based identity; UCAN borrows from this lineage
- **Bluesky / AT Protocol DID handling** — recent decentralized social identity work
- **Iden3 / Polygon ID / Sismo** — ZK-attestation systems in the cryptocurrency space
- **PGP web-of-trust** — historical precedent for relational trust
- **Keybase** — pre-Zoom-acquisition identity proofs across services

---

## § 6. Naming status

Working name: **"Kith"** or **"Kith"**.

A naming-research agent (dispatched 2026-05-11 evening) is checking:
- USPTO trademark status for the names
- Ecosystem-conflict with Wave Financial / Waves protocol / similar named projects
- SEO / findability concerns
- Alternative names if the working names have conflicts

Results land at `.addl/phase-4-foundation/kith-naming-research.md`. Final name decision pending that research + Ben's review.

---

## § 7. Next steps

1. Naming research returns + Ben picks final name (or alternative).
2. Phase 4-Foundation continues with MVP rotation mechanism per Q6 ratification.
3. When Phase 5+ planning opens, this scope-stub seeds the dedicated design phase. R1 spec review for the design-spike phase would dispatch identity-system-specific lenses (cryptographer-reviewer / privacy-engineer / decentralized-systems-reviewer / ux-on-attestation).
4. Kith design supersedes the MVP rotation mechanism when the substrate lands.

---

(Document seeded 2026-05-11 as exploratory destination per Phase 4-Foundation R1 triage. Will grow as Phase 5+ planning matures.)
