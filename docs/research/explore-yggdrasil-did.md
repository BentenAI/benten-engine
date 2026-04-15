# Exploration: Yggdrasil Address as DID -- Unified Networking + Identity

**Created:** 2026-04-11
**Purpose:** Deep exploration of using Yggdrasil mesh network addresses as Decentralized Identifiers (DIDs). The thesis: your network address IS your identity IS your encryption key. One cryptographic primitive serving three purposes. Analyze feasibility, security implications, integration with UCAN, and the dual-stack question for Benten instances.
**Status:** Research exploration (pre-design)

---

## The Core Thesis

Every Yggdrasil node generates an Ed25519 keypair. The public key is hashed and truncated into a stable IPv6 address in the `200::/7` range. This means:

1. **Your network address** is deterministically derived from your public key
2. **Your identity** is your public key (which IS your address)
3. **Your encryption** uses the same key material (Ed25519 for signing, convertible to X25519 for key agreement)

Three traditionally separate systems -- addressing, identity, encryption -- collapse into one cryptographic primitive. The question is whether this collapse is elegant or dangerous.

---

## 1. How Yggdrasil Address Derivation Works

### 1.1 The Algorithm

Yggdrasil v0.5 (latest: v0.5.13, released 2026-02-24) uses Ed25519 keypairs for node identity. The IPv6 address derivation works as follows:

```
Input:  Ed25519 public key (32 bytes)
Step 1: Invert all bits (XOR each byte with 0xFF)
Step 2: Count leading 1-bits in the inverted key = N
Step 3: Set first byte to 0x02 (or 0x03 for /64 subnet prefix)
Step 4: Set second byte to N (the leading-ones count)
Step 5: Strip the N leading 1-bits and the first 0-bit from the inverted key
Step 6: Append remaining bits to fill the 128-bit address
Output: A 200::/7 IPv6 address (specifically in 200::/8)
```

**Security property:** The leading-ones count (N) acts as a proof-of-work difficulty metric. Higher N means more bits of the original key survive into the address, making collisions harder. Nodes can brute-force generate keys with higher N for stronger address-to-key binding.

**Key insight:** The address is a LOSSY compression of the public key. You cannot recover the full 32-byte public key from the 16-byte IPv6 address. The address is a fingerprint, not the key itself. To verify identity, you need the full public key, which is exchanged during the Yggdrasil handshake.

### 1.2 What This Gives You

- **Stable addressing:** Same key = same address, regardless of physical location, ISP, or network topology
- **Self-authenticating:** A connection to address X is guaranteed to be with the holder of the corresponding private key (the address is derived from the key, so spoofing is cryptographically impossible)
- **End-to-end encrypted:** All Yggdrasil traffic is encrypted using the node's key material
- **Location-independent:** Your address follows you across networks, countries, devices (if you carry the key)
- **NAT-transparent:** Yggdrasil handles NAT traversal -- you can receive incoming connections behind CGNAT

### 1.3 Network Architecture

Yggdrasil builds a spanning tree overlay on top of whatever physical network exists (LAN, internet, point-to-point links). Peering is via TCP/TLS connections. Local discovery uses link-local multicast. Remote peering requires explicit configuration (no automatic peer exchange -- this is intentional for security).

The v0.5 protocol uses greedy routing with Bloom filters for on-tree link tracking, replacing the previous DHT-based approach. This is simpler and more performant but still alpha-quality.

---

## 2. DID Compatibility

### 2.1 The Natural Fit: did:key

The most natural DID method for Yggdrasil keys is `did:key`, which is already a W3C CCG specification. It encodes a public key directly:

```
did:key:z6Mk...  (Ed25519 multicodec prefix 0xed01 + 32-byte key, base58-btc encoded)
```

**Format:** `did:key:MULTIBASE(base58-btc, MULTICODEC(0xed01, raw-ed25519-public-key))`

This is the same Ed25519 key that Yggdrasil uses. The DID and the Yggdrasil address are derived from the SAME key material. Given a `did:key`, you can:
1. Extract the Ed25519 public key
2. Run Yggdrasil's `AddrForKey()` algorithm
3. Get the corresponding `200::/7` IPv6 address

And vice versa: given a Yggdrasil connection, you receive the peer's public key during handshake, and can construct the `did:key`.

**Advantage:** `did:key` is already standardized. No custom DID method needed.

**Disadvantage:** `did:key` has NO key rotation mechanism. The DID IS the key. Change the key, change the DID.

### 2.2 A Custom Method: did:ygg

A custom DID method could add Yggdrasil-specific semantics:

```
did:ygg:200:abcd:ef01:2345:6789:abcd:ef01:2345
```

Or more compactly:
```
did:ygg:z6Mk...  (same multibase encoding as did:key)
```

**What this adds over did:key:**
- Semantic clarity: "this DID is reachable on the Yggdrasil network"
- Resolution: a did:ygg resolver could attempt to connect to the Yggdrasil address and fetch the DID document from the node itself
- Service endpoints could be implicit (the node IS the service endpoint)

**What this costs:**
- A new DID method to specify, implement, and maintain
- Ecosystem tooling won't recognize it without custom resolvers
- The did:key method already works; this is sugar, not capability

**Recommendation:** Start with `did:key`. Consider `did:ygg` only if the resolution semantics (connect to node, fetch document) prove valuable enough to justify the maintenance burden.

### 2.3 The did:plc Model (For Key Rotation)

Bluesky's `did:plc` solves the key rotation problem elegantly:
- The DID is the hash of a signed **genesis operation**
- The genesis operation includes **rotation keys** that can sign updates
- Key rotation updates the signing/encryption keys while preserving the DID
- A central PLC directory validates and stores the operation log

**Relevance to Benten:** If we need persistent identity that survives key rotation, `did:plc` or a similar hash-of-genesis approach is the proven pattern. The cost is a directory service (centralization tradeoff).

### 2.4 KERI: The Most Rigorous Approach

Key Event Receipt Infrastructure (KERI) provides:
- **Self-certifying identifiers** derived from initial key material (like Yggdrasil)
- **Pre-rotation:** Commit to your NEXT key before you need it. If current key is compromised, the pre-committed rotation key takes over
- **Key Event Logs (KELs):** Append-only hash-chained log of all key state changes
- **Witnesses and Watchers:** Distributed receipt infrastructure for non-repudiable key events
- **No central authority:** Fully decentralized, ambient verifiability

**KERI + Yggdrasil:** A node's initial AID (Autonomic Identifier) could be derived from its Yggdrasil Ed25519 key. The AID persists even as keys rotate. When a key rotates, the Yggdrasil address changes, but the KERI AID does not. The KERI infrastructure provides the continuity layer that Yggdrasil alone cannot.

**Cost:** KERI is complex. Implementing a KERI witness/watcher infrastructure is a significant engineering effort. But it may be the "do it right" approach for a platform that treats identity as foundational.

---

## 3. Security Analysis

### 3.1 Key Compromise = Total Identity Theft

**The problem:** If someone steals your Yggdrasil private key, they:
- Become your network address (can receive your traffic)
- Become your DID (can sign as you)
- Can decrypt anything encrypted to you
- Can issue UCANs as you

With `did:key`, there is NO recovery. The DID IS the key. If the key is compromised, the identity is gone. You must create a new key (new address, new DID) and somehow convince everyone to update their records.

**Mitigations:**
1. **KERI pre-rotation:** Commit to rotation keys before compromise. Even if the active key is stolen, the pre-committed rotation key (stored separately, possibly offline) can reclaim the identity.
2. **did:plc-style rotation keys:** Separate signing keys from rotation keys. The rotation keys (ideally hardware-secured or multi-sig) can update the identity.
3. **UCAN revocation:** If UCANs are the authorization mechanism, a revocation list or epoch system can invalidate all tokens signed before a certain time.
4. **Multi-key identity:** The "identity" is not a single key but a KERI AID backed by multiple keys (M-of-N). Compromise of one key is survivable.

**Assessment:** Using a raw Yggdrasil address as the sole identity without a key rotation layer is a critical security risk. Any production system needs a rotation/recovery mechanism. The Yggdrasil address should be a TRANSPORT ADDRESS, not the canonical identity. The canonical identity should be a KERI AID or did:plc DID that can be re-bound to new transport addresses.

### 3.2 Key Rotation = Address Change

**The problem:** If you rotate your Ed25519 key, your Yggdrasil address changes. This means:
- All peers must update their peering configuration
- All services that reference your address must be updated
- DNS records (if any) must be updated
- Active connections are severed

**Mitigations:**
1. **Indirection layer:** Use the KERI AID or did:plc DID as the stable identifier. Resolve it to the current Yggdrasil address via a lightweight lookup (DHT, directory, or KERI witness infrastructure).
2. **Dual-key period:** During rotation, operate both old and new keys simultaneously for a transition window.
3. **Address book / contact list abstraction:** Applications never store raw addresses; they store DIDs and resolve to addresses.

**Assessment:** This is solvable but requires an additional resolution layer. The "address IS identity" simplification breaks down the moment you need key rotation. The correct architecture is: **DID -> resolves to -> current Yggdrasil address**.

### 3.3 Multiple Devices

**The problem:** Each device generates its own Ed25519 key and thus has its own Yggdrasil address. "These 3 addresses are all the same person" is not expressible within Yggdrasil alone.

**Solutions:**
1. **UCAN Powerline delegation:** UCAN spec includes a "Powerline" mechanism specifically for this. A root DID delegates ALL capabilities to device DIDs. Any device can act with the full authority of the root identity. The delegation chain proves "device B is authorized by root A."
2. **KERI multi-key AID:** A KERI AID can be controlled by multiple key pairs. All devices are co-controllers of the same identity.
3. **did:plc approach:** The did:plc DID document can list multiple verification methods (one per device). Any device's key can authenticate as the DID.
4. **Shared key (bad idea):** Copying the same private key to all devices "works" for address identity but violates key hygiene. A compromise of any device compromises all.

**Recommendation:** UCAN Powerline delegation is the most practical near-term solution. Each device has its own key/address but carries a UCAN chain proving delegation from a root identity. The root identity is a DID (did:key of the primary device, or a KERI AID). No key sharing required.

### 3.4 Privacy and Linkability

**The problem:** Your Yggdrasil address is your public key fingerprint. Every packet you send reveals your identity. Anyone who has seen your address on one service can link your activity across ALL services.

**Yggdrasil's own statement:** "Yggdrasil does not provide sender/receiver anonymity, unlinkability, cover traffic, or resistance to global traffic analysis."

**Specific risks:**
- A forum you participate in can correlate your identity with your blog, your marketplace activity, your social presence -- all from the same address
- A malicious peer can enumerate the network (currently possible via debug handlers) and build a map of who is running what services
- NodeInfo (enabled by default) leaks OS version, architecture, and Yggdrasil version

**Mitigations:**
1. **Context-specific keys:** Generate a different keypair (and thus address) for each context/community. You are `200:abc...` on the forum but `200:def...` on the marketplace. Link them only when YOU choose to (via UCAN delegation from a root identity).
2. **Disable NodeInfo privacy:** Set `NodeInfoPrivacy: true` in Yggdrasil config.
3. **Yggdrasil + Tor/I2P layering:** For high-privacy contexts, route Yggdrasil traffic through Tor. This adds anonymity at the cost of performance.
4. **Ephemeral keys:** For one-time interactions, generate a throwaway key. The address is used once and discarded.

**Assessment:** Yggdrasil is NOT a privacy network. It is an encrypted mesh. Privacy requires additional layers. For Benten, the architecture should support pseudonymous participation (multiple addresses per person, linked only by choice) and explicit opt-in to identity correlation.

### 3.5 Ed25519/X25519 Dual Use

Ed25519 (signing) keys can be mathematically converted to X25519 (key agreement/encryption) keys. Yggdrasil uses this implicitly. The `did:key` spec also leverages this: an Ed25519 `did:key` automatically implies an X25519 keyAgreement relationship.

**Security concern:** Best practice is to use separate keys for signing and encryption. Using the same key for both means a compromise of the signing key also compromises encryption (and vice versa). However, this is the accepted tradeoff in both Yggdrasil and the `did:key` ecosystem. The convenience of one key pair serving both purposes is considered worth the theoretical risk for most use cases.

**For Benten:** Accept this tradeoff at the Yggdrasil/DID layer. For high-security contexts (financial transactions, legal documents), applications can require separate key pairs.

---

## 4. UCAN Integration

### 4.1 The Elegant Simplification

If the DID is a Yggdrasil address (via `did:key` of the Ed25519 key), and UCANs are signed by DIDs:

```
1. Node A connects to Node B over Yggdrasil
2. Yggdrasil handshake proves A holds private key for address 200:A...
3. Node A presents a UCAN signed by did:key:zA... (same key)
4. Node B verifies:
   - The UCAN signature is valid (standard JWT verification)
   - The issuer DID matches the Yggdrasil peer (did:key:zA... <-> 200:A...)
   - NO separate authentication step needed
```

The Yggdrasil connection itself IS the authentication. The UCAN provides authorization. The two are unified by shared key material.

**What this eliminates:**
- Session tokens (the connection IS the session)
- Separate authentication protocols (OAuth, OIDC, etc.)
- Certificate authorities (the key IS the certificate)
- DNS-based identity verification

### 4.2 Coupling Risks

**Tight coupling of transport and identity:**
If the UCAN issuer is a Yggdrasil address, the UCAN is only verifiable in the context of Yggdrasil. A UCAN issued by `did:key:zA...` is verifiable anywhere (just check the signature), but the semantic link "this DID is reachable at this address" only holds on Yggdrasil.

**Mitigation:** Use `did:key` (not `did:ygg`). The DID is just a public key. It works on any transport. The Yggdrasil address is a BONUS -- you can verify the connection matches the DID, but the DID is independently useful.

**Transport-agnostic UCANs:**
The UCAN spec is explicitly transport-agnostic. UCANs can be carried over HTTP headers, WebSockets, libp2p, or raw TCP. This means a UCAN signed by a Yggdrasil key works on traditional HTTPS too. The key material is the same; only the transport differs.

### 4.3 Delegation and Capability Chains

UCAN's proof chain mechanism works naturally with Yggdrasil identities:

```
Root Identity (did:key:zRoot)
  |-- delegates "store:read:*" to -->
  Device A (did:key:zA, Yggdrasil addr 200:A...)
    |-- delegates "store:read:posts/*" to -->
    Service Worker (did:key:zSW)
```

The chain is verifiable by any party:
1. Check each UCAN signature
2. Check each delegation is a valid attenuation (subset) of the parent
3. The root issuer is the ultimate authority

**For Benten modules:** A module running on a Yggdrasil node receives UCANs that prove what the caller is authorized to do. The module doesn't need to query a central auth server. It verifies the UCAN chain locally. This is the "trustless" part of the decentralized web.

---

## 5. Comparison With Other DID Methods

| Method | Key Rotation | Resolution | Infrastructure | Privacy | Transport Binding | Benten Fit |
|--------|-------------|------------|----------------|---------|-------------------|------------|
| **did:key** | None (key IS identity) | None needed (self-describing) | Zero | Key is public | None | Good for transport + short-lived |
| **did:web** | Via DID document update | HTTP(S) fetch | DNS + web server | Domain-linked | HTTPS | Bad (centralized) |
| **did:plc** | Rotation keys in genesis | PLC directory HTTP | Central directory | DID is opaque hash | None | Good for persistent identity |
| **did:keri** | Pre-rotation + KEL | Witness/watcher infra | Distributed witnesses | AID is opaque | None | Best for "do it right" |
| **did:pkh** | Via chain mechanisms | Blockchain lookup | Blockchain node | Chain-linked | Chain-specific | Bad (heavy infra) |
| **did:ion** | Via Bitcoin anchoring | ION node | Bitcoin + IPFS | DID is opaque hash | None | Bad (heavy infra) |
| **did:peer** | Not supported (new DID) | Direct exchange | None | Ephemeral | None | Good for pairwise |
| **did:ygg (proposed)** | None (key IS address) | Yggdrasil connection | Yggdrasil network | Address is public key | Yggdrasil-native | Novel but fragile |

**Recommendation for Benten:**
- **Transport layer:** `did:key` (Ed25519) -- same key as Yggdrasil, works everywhere
- **Persistent identity:** KERI AID or `did:plc`-style -- survives key rotation
- **Pairwise/ephemeral:** `did:peer` or throwaway `did:key` -- for one-time interactions
- **Do NOT create did:ygg** unless resolution semantics prove essential

---

## 6. Practical Concerns: Yggdrasil in 2026

### 6.1 Maturity

- **Version:** 0.5.13 (February 2026). Still alpha. The project explicitly says: "you should probably not run any mission-critical or life-and-death workloads over Yggdrasil at this time."
- **Stability:** Rarely crashes. Wire protocol has broken backward compatibility (v0.4 -> v0.5). Another break before 1.0 is plausible.
- **Active development:** GitHub repo last updated April 2026. Ongoing maintenance and improvement.
- **No formal security audit:** Cryptographic primitives are sound (Ed25519, Noise framework), but the routing protocol and implementation have not been formally audited.

### 6.2 Network Size

- Exact node counts are not publicly reported in structured form
- Network map is generated via debug handler crawling (these handlers will be removed before 1.0)
- The network is used by several real-world projects (ThreeFold, various community mesh networks)
- It is a niche network. Thousands of nodes, not millions. Not comparable to Tor or I2P in adoption.

### 6.3 NAT Traversal

- Yggdrasil handles CGNAT effectively for IPv6-over-overlay traffic
- Peering requires at least one node with a publicly reachable address (or LAN multicast)
- Public peer lists are maintained by the community
- Works well for the "connect two nodes behind NATs" use case

### 6.4 Peer Discovery

- **Multicast:** Automatic on LAN (both nodes must have it enabled)
- **Static peering:** Manual configuration of remote peers (TCP/TLS URLs)
- **NO peer exchange:** Yggdrasil intentionally does not share peer information between nodes. This is a security/privacy choice. You must explicitly configure your peers.
- **DNS-based discovery:** Community proposal exists but not standardized

### 6.5 Assessment for Benten

Yggdrasil is a compelling transport layer for Benten's decentralized mode, but it is NOT production-grade. It should be:
- **Optional:** Benten instances must work without Yggdrasil
- **Additive:** Yggdrasil adds mesh networking, built-in encryption, and stable addresses ON TOP of traditional networking
- **Treated as alpha infrastructure:** Do not depend on wire protocol stability. Abstract behind an interface.

---

## 7. The Dual-Stack Question

### 7.1 The Requirement

Benten instances must work in three modes:
1. **Traditional only:** Standard HTTPS, DNS, OAuth/sessions. No Yggdrasil.
2. **Yggdrasil only:** Mesh network, DID-based auth, UCAN. No traditional internet dependency.
3. **Dual-stack:** Both simultaneously. Traditional internet for compatibility, Yggdrasil for mesh/decentralized features.

### 7.2 Identity Across Stacks

**The challenge:** A user with `did:key:zA...` is reachable at Yggdrasil address `200:A...` AND at HTTPS URL `https://alice.example.com`. How do these identities relate?

**Approach 1: DID document with multiple service endpoints**
```json
{
  "id": "did:key:zA...",
  "verificationMethod": [{ "type": "Ed25519VerificationKey2020", "publicKeyMultibase": "zA..." }],
  "service": [
    { "type": "BentenNode", "serviceEndpoint": "ygg://200:A..." },
    { "type": "BentenNode", "serviceEndpoint": "https://alice.example.com" }
  ]
}
```

**Approach 2: KERI AID with multiple transport bindings**
The KERI AID is the stable identity. It resolves to:
- Yggdrasil address (via KERI witness lookup or direct exchange)
- HTTPS URL (via well-known endpoint or DID document)
Both are verified against the AID's current key material.

**Approach 3: Independent identities, linked by UCAN**
- Traditional: session-based auth (Better Auth, as today)
- Yggdrasil: DID-based auth (UCAN)
- User links them by signing a UCAN that delegates from one identity to the other
This is the most pragmatic near-term approach. No changes to the existing auth system. Yggdrasil identity is layered on top.

### 7.3 Recommended Architecture

```
                   +-----------------------+
                   |   Persistent Identity |
                   |  (KERI AID or did:plc)|
                   +-----------+-----------+
                               |
              +----------------+----------------+
              |                                 |
    +---------v---------+           +-----------v-----------+
    | Transport: HTTPS  |           | Transport: Yggdrasil  |
    | Auth: Better Auth |           | Auth: UCAN + did:key  |
    | Address: DNS/IP   |           | Address: 200::/7 IPv6 |
    +---------+---------+           +-----------+-----------+
              |                                 |
              +----------------+----------------+
                               |
                   +-----------v-----------+
                   |    Benten Instance     |
                   |  (same data, same app) |
                   +-----------------------+
```

**Key design principle:** The persistent identity layer is ABOVE the transport layer. A Benten instance doesn't care whether a request came via HTTPS or Yggdrasil. It cares about the caller's identity and their capabilities (UCAN chain).

### 7.4 Migration Path

1. **Phase 1 (now):** Benten works on traditional internet with Better Auth. No Yggdrasil.
2. **Phase 2:** Add optional Yggdrasil transport. Benten instances can peer over Yggdrasil. Auth is still traditional.
3. **Phase 3:** Add `did:key` identity layer. UCAN-based auth for Yggdrasil connections. Traditional auth unchanged.
4. **Phase 4:** Add persistent identity (KERI or did:plc). Bridge traditional and Yggdrasil identities under a single persistent DID.
5. **Phase 5:** Full dual-stack. Any auth method works on any transport. The persistent identity is the unifying layer.

---

## 8. The Big Question: Is the Collapse Worth It?

### 8.1 What You Gain

- **Radical simplification for the Yggdrasil path:** No DNS, no CAs, no OAuth, no session management. Connect, handshake, present UCAN, done.
- **Censorship resistance:** No domain name to seize, no certificate to revoke, no server to shut down. The identity IS the node.
- **Offline-first by default:** Two Benten nodes on the same LAN can discover each other via multicast and authenticate via DID/UCAN without any internet connection.
- **True self-sovereignty:** The user controls their key. No platform can deactivate their identity (assuming KERI/did:plc for rotation).

### 8.2 What You Risk

- **Key management complexity for users:** "Keep your private key safe" is hard. Most people lose keys. Hardware wallets help but add friction.
- **No anonymity:** Yggdrasil addresses are linkable. Users must manage multiple identities manually for privacy.
- **Alpha infrastructure:** Yggdrasil may change its wire protocol, address derivation, or routing algorithm before 1.0.
- **Niche network:** Most of the internet does not run Yggdrasil. Dual-stack adds complexity.
- **Coupling risk:** If the identity IS the transport key, changing transport technology means changing identity infrastructure.

### 8.3 Verdict

The collapse of address/identity/encryption into one key is **beautiful for the Yggdrasil path** and **irrelevant for the traditional path**. The correct architecture is:

1. **Do NOT make Yggdrasil address the canonical identity.** Use a persistent DID (KERI AID or did:plc) as the canonical identity.
2. **Use Yggdrasil address as a transport binding** that the persistent DID resolves to.
3. **Use `did:key` of the Yggdrasil Ed25519 key** for UCAN signing on the Yggdrasil transport.
4. **Let the persistent DID bridge** both traditional auth and Yggdrasil/UCAN auth.

This gives you the elegance of "address = key" on Yggdrasil without the fragility of tying your permanent identity to a single key pair. Key rotation changes the Yggdrasil address but NOT the persistent DID. Both worlds work.

---

## 9. Benten-Specific Design Implications

### 9.1 Identity Architecture

```
BentenIdentity
  |-- persistent_did: "did:plc:abc123" (or KERI AID)
  |-- signing_keys: [did:key:zA..., did:key:zB...]  (one per device)
  |-- yggdrasil_addrs: [200:A..., 200:B...]          (derived from signing keys)
  |-- traditional_auth: { provider: "better-auth", user_id: "..." }
  |-- delegation_chain: UCAN linking all of the above
```

### 9.2 Store/Graph Implications

In the Benten engine's graph model, identity becomes a first-class Node:

```
[Identity Node: did:plc:abc123]
  |--HAS_KEY--> [Key Node: did:key:zA...]
  |               |--HAS_TRANSPORT--> [Transport: ygg://200:A...]
  |               |--HAS_TRANSPORT--> [Transport: https://alice.example.com]
  |--HAS_KEY--> [Key Node: did:key:zB...]  (second device)
  |--DELEGATES_TO--> [UCAN: { iss: did:plc:abc123, aud: did:key:zA... }]
```

### 9.3 Module Trust Model

The existing TrustTier system (platform/verified/community/untrusted) maps naturally:
- **platform:** Modules signed by the Benten project's KERI AID
- **verified:** Modules signed by known community developers (their persistent DIDs)
- **community:** Modules signed by any valid DID (self-published)
- **untrusted:** Modules with no valid signature chain

UCAN delegation chains can express trust delegation: "The Benten project trusts developer X, who vouches for module Y." This is the UCAN equivalent of the existing TrustTier hierarchy.

### 9.4 Cross-Instance Sync

When two Benten instances sync over Yggdrasil:
1. Yggdrasil handshake authenticates both sides (key exchange)
2. Each side presents UCANs proving their authority to sync specific data
3. CRDTs resolve conflicts (the Benten engine's graph sync mechanism)
4. No central server mediates

The identity layer proves "who is syncing" and the UCAN layer proves "what they are allowed to sync." The Yggdrasil layer provides the transport. Clean separation.

---

## 10. Open Questions for Further Exploration

1. **KERI vs did:plc:** Which persistent identity layer is better for Benten? KERI is more rigorous but more complex. did:plc is simpler but depends on a directory service.
2. **Key storage UX:** How do we make key management invisible to users while keeping it secure? Hardware key integration? OS keychain? Passkey-style approach?
3. **Privacy contexts:** How do we let users create context-specific identities (different address per community) without losing the ability to prove "these are all me" when desired?
4. **Yggdrasil alternatives:** Should Benten also support libp2p, I2P, or Tor as transport layers? How does the identity architecture change (or not change) across transports?
5. **UCAN revocation:** The current UCAN spec has limited revocation support. For a platform where users can lose devices, revocation is critical. Research UCAN revocation proposals.
6. **Ed25519 quantum resistance:** Ed25519 is not quantum-resistant. If Benten is building for the long term, when and how should post-quantum key material be integrated?
7. **Yggdrasil 1.0 timeline:** Is the project moving toward a 1.0 release? What breaking changes are expected? Should Benten abstract Yggdrasil behind an interface to survive protocol changes?

---

## Sources

- [Yggdrasil Network -- Official Site](https://yggdrasil-network.github.io/)
- [Yggdrasil Addressing and Name-Independent Routing](https://yggdrasil-network.github.io/2018/07/28/addressing.html)
- [Yggdrasil Implementation Details](https://yggdrasil-network.github.io/implementation.html)
- [Yggdrasil Privacy](https://yggdrasil-network.github.io/privacy.html)
- [Yggdrasil FAQ](https://yggdrasil-network.github.io/faq.html)
- [Yggdrasil Configuration Reference](https://yggdrasil-network.github.io/configurationref.html)
- [Yggdrasil v0.5 Releases](https://github.com/yggdrasil-network/yggdrasil-go/releases)
- [Yggdrasil Whitepaper](https://github.com/Arceliar/yggdrasil-go/blob/master/doc/Whitepaper.md)
- [Yggdrasil Address Package (Go)](https://pkg.go.dev/github.com/yggdrasil-network/yggdrasil-go/src/address)
- [Yggstack -- Yggdrasil + Netstack](https://github.com/yggdrasil-network/yggstack)
- [W3C DID v1.1 Specification](https://www.w3.org/TR/did-1.1/)
- [W3C DID Primer](https://w3c-ccg.github.io/did-primer/)
- [did:key Method v0.9 Specification](https://w3c-ccg.github.io/did-key-spec/)
- [did:plc Specification v0.1](https://web.plc.directory/spec/v0.1/did-plc)
- [did:plc GitHub Repository](https://github.com/did-method-plc/did-method-plc)
- [did:peer Method Specification](https://identity.foundation/peer-did-method-spec/)
- [KERI (Key Event Receipt Infrastructure)](https://keri.one/)
- [KERI Specification at DIF](https://identity.foundation/keri/docs/KERI-made-easy.html)
- [KERI Whitepaper (arXiv)](https://arxiv.org/abs/1907.02143)
- [UCAN Specification](https://github.com/ucan-wg/spec)
- [UCAN Delegation Specification](https://github.com/ucan-wg/delegation)
- [UCAN Invocation Specification](https://github.com/ucan-wg/invocation)
- [UCAN HTTP Bearer Token Specification](https://github.com/ucan-wg/ucan-http-bearer-token)
- [UCAN Transport-Agnostic Discussion](https://github.com/ucan-wg/spec/discussions/18)
- [Fission Device Linking Whitepaper](https://github.com/fission-codes/whitepaper/blob/master/accounts/device-linking.md)
- [Storacha UCAN Documentation](https://docs.storacha.network/concepts/ucan/)
- [Ed25519 to X25519 Conversion (libsodium)](https://libsodium.gitbook.io/doc/advanced/ed25519-curve25519)
- [AT Protocol DID Documentation](https://atproto.com/specs/did)
- [Murmurations: Decentralized Data Sharing via DIDs and UCANs](https://murmurations.network/2025/12/31/2025-update-decentralised-data-sharing-via-dids-and-ucans/)
