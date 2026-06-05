# Crypto Codepoints — the Benten suite-selector table + reserve registry

Benten owns **only the thin suite-selector codepoint table** (one
codepoint-per-suite; the MLS RFC 9420 model, NOT the HPKE algorithm-triple).
Component algorithm IDs reference the IANA HPKE/COSE registries — Benten never
mints its own algorithm numbers. This document is the human-readable companion
to `crates/benten-crypto-suite/src/codepoint.rs` (the wire-canonical source of
truth) and `docs/V1-FROZEN-INTERFACE.md` item 6 (the freeze record).

## The conservative-fallback freeze contract (NQ-A1)

The codepoint table integer values are **FROZEN at v1-beta** (per
V1-FROZEN-INTERFACE.md item 6). Algorithms behind each codepoint are swappable
within the multiformats framing; the codepoint values themselves are
wire-canonical and permanent.

Post-freeze security findings are handled **additively** (reserve-codepoints + a new tag) — **never a silent wire-break**.
An unknown / reserved codepoint typed-rejects (fail-closed
`UnsupportedAlgorithm`) — never a silent fallback, never a silent downgrade.
This is the NQ-A1 conservative-fallback policy:

- New algorithms land at **unused reserved codepoints** (additive); existing
  codepoint values are never repurposed.
- A reserved codepoint is typed-rejected at v1-beta and becomes LIVE
  additively at v1-GM+ — adding it is **never a wire-format break** (old
  content with old codepoints still decodes forever).
- `old-codepoints-supported-forever`: a deprecated codepoint still decodes
  existing content (no new content); only `Quarantined`/`Burned` states
  typed-reject permanently.

## LIVE suite-selector codepoints (v1-beta)

| Codepoint | Suite | State |
|-----------|-------|-------|
| `0x647a`  | X25519⊕ML-KEM-768 hybrid KEM (real X-Wing SHA3-256 combiner) + ChaCha20-Poly1305 bulk | **LIVE — default** |
| `0x6400`  | X25519-only classical KEM + ChaCha20-Poly1305 bulk | LIVE (non-default downgrade) |
| `0x0001`  | Ed25519⊕ML-DSA-65 hybrid signature (LAMPS Composite) | LIVE (default sig) |
| `0x0002`  | Ed25519-only classical signature | LIVE (non-default downgrade) |
| `0x6500`  | Layer-C drop, plaintext-sender (sender-DID on wire) | LIVE (non-default) |
| `0x6510`  | Layer-C drop, **Sealed-Sender** (sender-DID inside ciphertext) | **LIVE — default (BR-1)** |
| `0x6520`  | Layer-C group multi-stanza (`HpkeMultiBase`, blinded per-stanza AAD) | LIVE |
| `0x6610`  | MembershipSet K_Set group multi-stanza (blinded AAD) | LIVE |

## RESERVED codepoints (typed-reject at v1-beta; additive at v1-GM+)

| Codepoint / band | Reserve | State |
|------------------|---------|-------|
| `0x647b`         | NF-1 ML-KEM-768⊕HQC PQ⊕PQ end-state | RESERVED (build-trigger = FIPS 207 final) |
| `0x647c`         | Pure-PQ ML-KEM-768-only swap-matrix arm | RESERVED (audit-gated) |
| `0x0003`         | NF-1 ML-DSA-65⊕SLH-DSA PQ⊕PQ signature | RESERVED |
| `0x0000`         | No-encryption (plaintext partition) | RESERVED |
| `0x6320..0x632F` | RemotePermission band (incl. `ExecuteWorkflow` reserve) | RESERVED |
| `0x6620`         | `SubsetRef` federation reserve (`MEMBERSHIP_SET_SUBSET_REF`) | RESERVED |
| `0x6380..0x63CF` | FS-future MLS/CGKA bracket (incl. `RotatingGroupKeyChainedMode` + `ChainedStateTlv`) | RESERVED |
| (no Core integer) | `RecoveryArtifact` — reserved-at-Core conceptually; the `RecoveryHook` trait lands in Phase-4-Meta-Composing alongside the allocated codepoint (NQ-W5/m-14) | RESERVED (Composing) |

The reserved set is enumerated in
`crates/benten-crypto-suite/src/codepoint.rs::ReservedCodepoint` — each slot's
`resolve()` ALWAYS typed-rejects at v1-beta (never a silent accept). The
`ChainedStateTlv` per-stanza sub-slot (GAP-6b) is **AAD-bound** when present
(`chained_state_tlv_aad_binding`) so a present-vs-absent flip is detectable at
decrypt (not advisory).

## did:key hybrid-pubkey multicodec (NQ-C4 / U15)

The PQ-**hybrid** public keys carried in `did:key` need their own multicodec
prefixes, distinct from the Ed25519-only `ED25519_MULTICODEC = [0xed, 0x01]`:

- **Hybrid signature pubkey** (Ed25519⊕ML-DSA-65) — `HYBRID_SIG_MULTICODEC`
  in `crates/benten-id/src/did.rs`.
- **Hybrid KEM pubkey** (X25519⊕ML-KEM-768) — `HYBRID_KEM_MULTICODEC` in
  `crates/benten-id/src/did.rs`.

**Multiformats-registration status (OPEN-SPEC, NQ-C4 / §5.D-9).** At the time
of writing the multiformats registry has **no assigned multicodec value** for
these specific PQ-hybrid pubkey shapes. Per the conservative-fallback policy
above, Benten reserves a **private-value-with-fallback** prefix for each hybrid
pubkey (reserved at G-CORE-9), to be swapped for the registered multiformats
value once one is allocated — an additive change, **never a wire-break**. The
`did:agent:` method is an **optional allowlist alias** (Inv-22: nature DERIVED
via method-parse; the alias is a hint, never a stored authoritative
discriminator and never authority-bearing).

## Transport-config reserve (Compromise #53 / §3.9)

Transport selection (`benten-sync::transport_config::TransportConfig`) follows
the same additive discipline: `GossipPlusBlobs` (iroh-gossip + iroh-blobs)
ships at v1-beta; the `Willow` / `IrohRoq` / `IrohLive` variants are
reserved-and-typed-rejected — adding them later is additive, never a
wire-break, never a silent fallback to gossip.
