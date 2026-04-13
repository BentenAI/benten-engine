# Decentralized Technologies Overview — Complete Landscape (April 2026)

**Created:** 2026-04-11
**Purpose:** Comprehensive overview of ALL high-level decentralized technologies, evaluated through the lens of the Benten engine (graph-native data, P2P sync, capability-based security, community governance).
**Audience:** CEO/co-architect strategic reference.

---

## Reading Guide

Each technology gets:
1. **One-line explanation**
2. **How it works** (2-3 sentences)
3. **Relevance to Benten** (High / Medium / Low / None)
4. **If relevant:** How it fits into the architecture

At the end, a summary flags what Benten is already building equivalents of, what it should adopt as standards, and what becomes relevant in future phases.

---

## 1. Consensus Mechanisms

Consensus mechanisms answer: "How do multiple nodes agree on the state of shared data without trusting each other?"

### 1.1 Proof of Work (PoW)

**What:** Nodes compete to solve a computationally expensive puzzle; the winner appends the next block.

**How:** Miners hash block data with a random nonce until the hash meets a difficulty target. The first valid solution wins the right to propose the block. Other nodes verify cheaply (one hash check). Security comes from the economic cost of computation -- reversing history requires re-doing all that work.

**Relevance: None.** PoW is designed for open, adversarial, permissionless networks (Bitcoin, pre-Merge Ethereum). Benten is a cooperative platform where instances are owned by real people/organizations. PoW's energy waste and latency (minutes per block) are anti-patterns for our use case.

---

### 1.2 Proof of Stake (PoS)

**What:** Validators stake economic collateral; misbehavior forfeits their stake ("slashing").

**How:** Validators lock tokens as a bond. They are randomly selected (weighted by stake) to propose and attest to blocks. If they sign conflicting blocks or go offline, a portion of their stake is destroyed. This aligns incentives without wasting energy.

**Relevance: Low.** PoS solves the "open participation without trust" problem for global blockchains (Ethereum post-Merge, Solana, Cardano). Benten instances are small, trusted federations (a family, a team, a community). The economic-incentive model does not apply to our topology. However, the *slashing concept* -- "verifiable consequences for misbehavior" -- is architecturally interesting as a pattern for community governance penalties.

---

### 1.3 Proof of Authority (PoA)

**What:** A fixed, known set of validators signs blocks; identity (not economics) provides trust.

**How:** A governance process designates N validator identities. These validators take turns proposing blocks. If a validator misbehaves, the governance process removes them. Fast, cheap, but centralized by design.

**Relevance: Low.** PoA is what traditional hosted platforms already do implicitly -- the operator IS the authority. In Benten, the instance operator is already the authority over their own data. For multi-instance sync, we need something more nuanced than "designated validators" but the idea of "known, accountable identity" is already part of our DID-based capability model.

---

### 1.4 Byzantine Fault Tolerance (BFT) / Practical BFT (pBFT)

**What:** Consensus protocols that tolerate up to f malicious nodes in a network of 3f+1 nodes.

**How:** pBFT uses a three-phase protocol (pre-prepare, prepare, commit). The leader proposes a value, nodes exchange signed messages confirming they agree, and once 2/3+ sign off, the value is committed. Tolerates both crashes AND actively malicious nodes (unlike Raft/Paxos which only tolerate crashes). Finality is immediate -- no probabilistic confirmation.

**Relevance: Medium.** When Benten instances sync subgraphs across multiple peers, we need to handle the case where one peer sends corrupted or malicious data. We don't need full BFT consensus (we're not building a blockchain), but the *principles* -- message authentication, quorum verification, detecting equivocation -- inform how we validate incoming sync payloads. The version-chain + content-hashing approach in the Benten spec already provides some of this. If we ever build multi-instance governance (e.g., a community DAO voting on moderation policy), BFT concepts become more directly relevant.

---

### 1.5 Raft

**What:** A crash-fault-tolerant consensus algorithm designed for understandability.

**How:** One node is elected leader via randomized timeouts. The leader accepts writes, replicates them to followers via append-only log entries, and commits once a majority acknowledge. If the leader crashes, a new election occurs. Only tolerates crashes, NOT Byzantine (malicious) faults.

**Relevance: Medium.** Raft is the workhorse of non-blockchain distributed systems (etcd, Consul, CockroachDB). For Benten, Raft could be useful if we need consensus among a small cluster of instances that trust each other (e.g., a family's devices syncing). However, our CRDT-based sync model is more appropriate for our topology because it doesn't require a leader -- any node can write independently and merge later. Raft's log-replication approach informs our version-chain design but we chose CRDTs over leader-based consensus for a reason: offline-first, no single point of failure.

---

### 1.6 Paxos

**What:** The foundational consensus algorithm (Lamport, 1989) that Raft simplifies.

**How:** Similar to Raft but with a more complex multi-phase proposal mechanism. A proposer sends a proposal number, acceptors promise not to accept lower-numbered proposals, and the proposer can then issue a final accept message. Mathematically proven correct but notoriously difficult to implement.

**Relevance: Low.** Same rationale as Raft. Paxos is the theoretical foundation that Raft makes practical. We don't need either because our CRDT approach handles consensus differently -- eventual consistency via mathematical conflict resolution rather than leader-based agreement. Worth understanding conceptually but not a direct adoption target.

---

### 1.7 Tendermint / CometBFT

**What:** A BFT consensus engine that separates consensus from application logic via an interface (ABCI).

**How:** Tendermint implements BFT consensus and exposes an Application BlockChain Interface (ABCI/ABCI++). Your application receives proposed transactions, validates them, and returns results. Tendermint handles all the consensus, networking, and replication. Achieves 10k+ TPS with instant finality. Now maintained as CometBFT (fork/successor).

**Relevance: Low-Medium.** The ABCI separation of concerns is architecturally elegant -- "consensus engine doesn't know what the application does." This mirrors Benten's own separation: the graph engine doesn't know what the modules do, it just stores Nodes/Edges and maintains views. If we ever need deterministic ordering for multi-party transactions (e.g., a marketplace escrow), Tendermint's approach of plugging custom logic into a consensus engine is a proven pattern. Not an adoption target, but a design reference.

---

### 1.8 Avalanche Consensus

**What:** A probabilistic consensus protocol that achieves finality through repeated random subsampling.

**How:** Instead of electing a leader or broadcasting to all nodes, each node randomly polls a small subset of other nodes about which transaction they prefer. After enough rounds of subsampled polling, nodes converge on the same decision with mathematical certainty. Achieves sub-second finality with thousands of nodes and high throughput.

**Relevance: Low.** Avalanche consensus is optimized for large-scale open networks (thousands of validators). Benten's sync topology is small federations (3-50 peers), where full-mesh gossip is feasible and CRDTs handle conflicts. The subsampling approach is clever but solving a different scale problem than ours. Potentially interesting if Benten instances ever form very large public networks.

---

## 2. Cryptographic Primitives

The building blocks that make trustless systems possible.

### 2.1 Public Key Cryptography (Asymmetric Encryption)

**What:** A key pair where one key encrypts and the other decrypts; knowing one doesn't reveal the other.

**How:** A user generates a private key (secret) and derives a public key (shareable). Messages encrypted with the public key can only be decrypted with the private key, and vice versa. This enables both confidential communication (encrypt with recipient's public key) and authentication (sign with your private key, anyone can verify with your public key).

**Relevance: High -- ALREADY BUILDING.** Public key cryptography is the bedrock of Benten's identity model. Each instance has a keypair. DIDs are derived from public keys. Capability tokens are signed with private keys. Every sync message is signed. This is not something to "adopt" -- it's already foundational.

**Benten integration:** Instance identity, DID generation, capability token signing, sync message authentication, encrypted subgraph payloads.

---

### 2.2 Digital Signatures — Ed25519

**What:** A high-speed, high-security elliptic curve signature scheme.

**How:** Based on Curve25519 (Bernstein). Signing produces a 64-byte signature from a 32-byte private key. Verification uses the 32-byte public key. Deterministic (same message + key always produces same signature), resistant to timing side-channel attacks, and 20-30x faster than secp256k1. Widely adopted: SSH, Signal, Tor, AT Protocol.

**Relevance: High -- SHOULD ADOPT.** Ed25519 should be Benten's default signature algorithm. It's the modern standard for non-blockchain identity systems. AT Protocol uses it. Signal uses it. SSH uses it. It's faster and safer than secp256k1 (which exists primarily for Ethereum/Bitcoin compatibility).

**Benten integration:** Default signing algorithm for DIDs (`did:key` with Ed25519), capability token signatures, version-chain commit signatures, sync message authentication.

---

### 2.3 Digital Signatures — secp256k1

**What:** The elliptic curve used by Bitcoin and Ethereum for ECDSA signatures.

**How:** Based on a specific Koblitz curve. The same sign/verify pattern as Ed25519 but using a different curve with different tradeoffs. Slower, more susceptible to implementation bugs (non-deterministic without RFC 6979), but universally supported by blockchain ecosystems.

**Relevance: Low.** Only relevant if Benten needs to interact with Ethereum/Bitcoin wallets or verify blockchain-originated credentials. We should NOT use secp256k1 as a primary algorithm -- Ed25519 is strictly superior for our use case. However, supporting secp256k1 verification as an optional capability would enable interop with blockchain-based identity.

---

### 2.4 Hash Functions — SHA-256

**What:** The most widely deployed cryptographic hash function (256-bit output).

**How:** Takes arbitrary input, produces a fixed 256-bit digest. Collision-resistant (finding two inputs with the same hash is computationally infeasible). Used everywhere: Git, Bitcoin, TLS, content addressing. Well-studied, NIST-standardized, hardware-accelerated on most CPUs.

**Relevance: Medium.** SHA-256 is the safe default for interoperability. Content identifiers in IPFS (CIDv0) use SHA-256. Git uses SHA-1 (migrating to SHA-256). If Benten needs to produce content identifiers compatible with IPFS/IPLD, SHA-256 is required. However, BLAKE3 is faster for internal use.

---

### 2.5 Hash Functions — BLAKE3

**What:** A cryptographic hash function that is dramatically faster than SHA-256 while maintaining equivalent security.

**How:** Based on the BLAKE2 family but redesigned for parallelism. Uses a Merkle tree structure internally, so hashing large data can use all CPU cores. Produces 256-bit output by default but supports arbitrary output lengths. Can also function as a MAC (keyed hash) and KDF (key derivation function). Roughly 5-10x faster than SHA-256 in software (no hardware acceleration needed because it's already fast).

**Relevance: High -- SHOULD ADOPT.** BLAKE3 should be Benten's default hash function for all internal operations: content hashing for version chains, Merkle tree construction, property integrity verification. Use SHA-256 only when external interoperability requires it (e.g., producing IPFS-compatible CIDs). The Rust `blake3` crate is the reference implementation.

**Benten integration:** Version chain commit hashes, property content hashing, Merkle tree construction for subgraph sync, content-addressed node identity.

---

### 2.6 Merkle Trees

**What:** A tree of hashes where each parent is the hash of its children, culminating in a single root hash.

**How:** Leaf nodes contain data hashes. Each internal node contains the hash of its two children. The root hash represents the entire tree's contents. Changing any leaf changes the root. Verification is O(log n) -- to prove a leaf is in the tree, you only need the sibling hashes along the path to the root (a "Merkle proof").

**Relevance: High -- ALREADY BUILDING.** Merkle trees are central to Benten's sync protocol. When two instances want to synchronize, they compare root hashes first. If different, they walk down the tree to find the specific branches that diverge, transferring only the changed data. This is how Git works (for files), how AT Protocol works (for user repositories), and how Benten will work (for subgraphs).

**Benten integration:** Subgraph sync uses Merkle trees to identify divergence efficiently. The version chain itself is a Merkle-like structure (each commit hashes the previous commit + the changes).

---

### 2.7 Merkle DAGs

**What:** A generalization of Merkle trees where nodes can have multiple parents (forming a DAG, not a tree).

**How:** Same principle as Merkle trees, but the structure is a directed acyclic graph rather than a strict tree. Each node's hash incorporates its content and the hashes of all nodes it links to. Any node can be verified by following its hash links. This is the data structure underlying IPFS, Git, and IPLD.

**Relevance: High -- ALREADY BUILDING.** Benten's version chain is a Merkle DAG. When a node is updated, the new version links to the previous version(s). When two instances diverge and later merge, the merge commit links to both branches -- forming a DAG, not a linear chain. This is exactly how Git handles branches and merges, and it's how Benten handles concurrent edits to the same data.

**Benten integration:** Version chains are Merkle DAGs. The entire graph history is a Merkle DAG of commits. Content-addressed node identity uses the same structure.

---

### 2.8 Zero-Knowledge Proofs (zk-SNARKs, zk-STARKs, PLONK)

**What:** Cryptographic proofs that let you prove a statement is true without revealing the underlying data.

**How:**
- **zk-SNARKs** (Succinct Non-interactive ARguments of Knowledge): Compact proofs (~200 bytes), fast verification, but require a trusted setup ceremony. Used by Zcash, many ZK-rollups.
- **zk-STARKs** (Scalable Transparent ARguments of Knowledge): No trusted setup, post-quantum secure, but proofs are larger (~50-100KB). Used by StarkNet.
- **PLONK**: A universal zk-SNARK that reduces the trusted setup to a one-time ceremony (not per-circuit). Widely adopted in 2024-2026 as a compromise.

Recent developments (2026): ZKP-based rollups hold $28 billion TVL. zkVMs (RISC Zero, Succinct Labs) let you write proofs in Rust instead of circuit-specific languages. Hardware acceleration is reducing proof generation costs.

**Relevance: Medium (future).** ZKPs could enable powerful Benten features in later phases:
- **Privacy-preserving capability verification**: Prove "I have the capability to read this subgraph" without revealing which specific capability token you hold.
- **Selective disclosure**: Prove "I am over 18" from a verifiable credential without revealing your birthdate.
- **Integrity proofs for sync**: Prove "this subgraph was computed correctly from the source data" without the verifier needing to re-execute.

Not a near-term adoption target (the tooling is still complex and computationally expensive), but a strong candidate for Phase 2+ features around privacy and selective sharing.

---

### 2.9 Homomorphic Encryption

**What:** Encryption that allows computation on encrypted data without decrypting it.

**How:** Data is encrypted in a special form. Mathematical operations on the ciphertext produce results that, when decrypted, match the result of applying the same operations to the plaintext. Fully homomorphic encryption (FHE) supports arbitrary computation. Performance is improving (~10x faster every two years) but still 1000-10,000x slower than plaintext computation for general operations.

**Relevance: Low (far future).** The use case for Benten would be: a community instance aggregates statistics across member data without any member revealing their actual data (e.g., "average satisfaction score" without seeing individual scores). This is genuinely useful but the performance overhead makes it impractical for real-time systems in 2026. Check back in 2028-2030.

---

### 2.10 Multi-Party Computation (MPC)

**What:** A protocol where multiple parties jointly compute a function over their inputs without revealing those inputs to each other.

**How:** Each party holds a private input. Through a series of message exchanges, they collectively compute a result (e.g., "is anyone on the sanctions list?") without any party learning the others' inputs. Used in practice for key management (threshold wallets), private auctions, and cross-organizational data analysis.

**Relevance: Low-Medium (future).** MPC could enable:
- **Threshold key management**: An instance's private key is split across multiple devices (phone + laptop + hardware key); any 2-of-3 can sign.
- **Private governance voting**: Compute election results without revealing individual votes.

Threshold signatures (Section 2.12) are the most practical MPC application for Benten's near-term roadmap.

---

### 2.11 Verifiable Random Functions (VRF)

**What:** A function that produces a random output along with a proof that the output was correctly computed.

**How:** The holder of a private key can compute VRF(key, input) to get a pseudorandom output plus a proof. Anyone with the public key can verify the output is correct for that input, but cannot predict outputs for other inputs. Used for fair leader election (Algorand), random lottery selection, and unpredictable-but-verifiable randomness.

**Relevance: Low.** VRFs solve fairness problems in large-scale open networks where you need verifiable randomness (e.g., "who gets to propose the next block?"). Benten doesn't have this problem -- our instances are small, trusted, and don't need random leader election. Potentially useful if we build a decentralized marketplace that needs provably fair selection (e.g., "which seller gets featured?"), but that's a distant future concern.

---

### 2.12 Threshold Signatures

**What:** A signature scheme where t-of-n parties must cooperate to produce a valid signature.

**How:** A private key is split into n shares, distributed to n parties. Any t (threshold) of those parties can combine their partial signatures to produce a valid signature indistinguishable from a regular single-party signature. No single party ever holds the complete private key.

**Relevance: Medium (future).** Threshold signatures are directly relevant to Benten's security model:
- **Instance key recovery**: If a user loses one device, their other devices can still sign (2-of-3 threshold).
- **Community governance**: A community instance's actions require approval from multiple moderators (e.g., 3-of-5 moderators must sign to ban a user).
- **Backup**: The user's key is split across their devices, and one share is held by a trusted backup service (but the backup service alone cannot sign).

This is a Phase 2+ capability but architecturally aligned with Benten's vision of user sovereignty over keys.

---

## 3. Data Structures

The structures that make decentralized data possible.

### 3.1 Distributed Hash Tables (DHT) -- Kademlia

**What:** A decentralized key-value lookup system where no single node holds all the data.

**How:** Each node has a unique ID. Keys are mapped to the node whose ID is "closest" (by XOR distance in Kademlia). To find a value, you iteratively query nodes that are progressively closer to the target key. Kademlia guarantees finding any key in O(log n) hops. Used by BitTorrent, IPFS, and libp2p for peer discovery and content routing.

**Relevance: Medium.** DHTs solve the "how do I find which peer has the data I want?" problem. For Benten:
- **Peer discovery**: When an instance wants to sync with another instance, it needs to find that instance's network address. A DHT provides decentralized name resolution.
- **Content routing**: If a subgraph is replicated across multiple peers, a DHT can route requests to the nearest copy.

Benten will likely use libp2p's Kademlia DHT implementation rather than building one from scratch. This is an ADOPT candidate -- use the standard, don't reinvent.

---

### 3.2 Content-Addressed Storage (CAS / CID)

**What:** Data is identified by its cryptographic hash rather than by location (URL) or arbitrary name.

**How:** You hash the content to produce an identifier (Content Identifier / CID). To retrieve the content, you ask the network "who has the data for this hash?" and verify on receipt that the hash matches. This makes data self-certifying -- if the hash matches, the content is correct regardless of who served it. Immutable by definition: changing the content changes the hash.

**Relevance: High -- ALREADY BUILDING.** Benten's version chain is content-addressed. Each commit in the version chain is identified by the hash of its contents. This is the same principle as Git commits, IPFS blocks, and AT Protocol records. Content addressing gives us:
- **Integrity**: No node can serve corrupted data (the hash would mismatch).
- **Deduplication**: Identical content has the same hash, stored once.
- **Efficient sync**: Compare hashes to find differences, transfer only changed data.

**Benten integration:** Node version identity is a content hash. Edge identity is a content hash. Subgraph identity is the Merkle root of all contained hashes. This is already specified in the engine design.

---

### 3.3 Merkle Search Trees (MST) -- AT Protocol

**What:** A balanced Merkle tree that supports efficient key-value lookups with cryptographic verification.

**How:** AT Protocol stores each user's data repository as a Merkle Search Tree. Records are sorted by key (TID -- timestamp-based ID), and the tree remains balanced as records are inserted or deleted. The root hash of the MST is signed by the user, enabling efficient cryptographic proofs that a record exists in a user's repository. Unlike a regular Merkle tree, MSTs support range queries and ordered iteration.

**Relevance: High -- SHOULD STUDY.** The MST is one of AT Protocol's most significant innovations and directly relevant to Benten:
- It solves the same problem as our version chain: "How do you efficiently prove what data a user has and detect changes?"
- AT Protocol's MST is designed for user-owned data repositories with federated sync -- exactly Benten's topology.
- The MST's chronological sorting aligns with our version chain's commit ordering.

**Benten integration:** We should deeply study the MST design. Our version chain serves a similar purpose but our graph data model is richer (Nodes + Edges + Labels vs. AT Protocol's flat records). The key insight to borrow: the Merkle tree structure enables O(log n) sync negotiation between peers.

---

### 3.4 CRDTs (Conflict-free Replicated Data Types)

**What:** Data structures that can be independently modified on multiple replicas and always merge deterministically without conflicts.

**How:** CRDTs use mathematical properties (commutativity, associativity, idempotence) to ensure that regardless of the order operations are received, all replicas converge to the same state. Two main types: state-based (send full state, merge via join-semilattice) and operation-based (send operations, apply commutatively). Used in production by: Notion (collaborative editing), Apple (Notes sync), Figma (multiplayer design), military systems (Anduril's $100M edge data mesh).

**Relevance: High -- ALREADY BUILDING.** CRDTs are central to Benten's sync model. When two instances modify the same data independently and then sync, CRDTs ensure deterministic merge without human conflict resolution. The spec already calls for CRDT-based property conflict resolution within the graph engine.

**Benten integration:** Property-level CRDTs for concurrent edits to the same Node. The version chain itself is a CRDT (a grow-only DAG where merge commits resolve forks). Edge sets use add-wins semantics. The open question is which CRDT types to support for different property types (LWW-Register for simple values, RGA for text sequences, PN-Counter for counters).

---

### 3.5 DAGs (Directed Acyclic Graphs)

**What:** Graph data structures where edges have direction and no cycles exist.

**How:** Used as the core data structure in several decentralized systems: IOTA Tangle (transactions form a DAG instead of a chain), Hedera Hashgraph (gossip about gossip forms a DAG), Git (commits form a DAG). DAGs allow concurrent "branches" that can be merged later, unlike a linear blockchain where blocks must be strictly ordered.

**Relevance: High -- ALREADY BUILDING.** Benten's entire data model is a graph, and version chains are specifically DAGs. The engine's DAG module already provides cycle detection, topological sort, BFS, DFS, and traversal. The version chain (anchor -> snapshots -> CURRENT) is a DAG by design.

---

### 3.6 Append-Only Logs

**What:** A data structure where entries can only be added, never modified or deleted.

**How:** New entries are appended to the end of a log with a monotonically increasing sequence number. Each entry typically includes a hash of the previous entry, forming a hash chain. This provides tamper evidence -- altering any entry breaks all subsequent hashes. Used by: Kafka (event streaming), Certificate Transparency (audit log), AT Protocol (user repositories), Secure Scuttlebutt (social feeds).

**Relevance: High -- ALREADY BUILDING.** Benten's version chain IS an append-only log per Node. Each version (commit) links to its predecessor(s). The "history IS the graph" principle from the spec means we never delete history -- we only append new versions. This gives us full auditability, rollback capability, and sync-friendliness (append-only structures are trivially mergeable).

---

### 3.7 Vector Clocks

**What:** A data structure that tracks causal ordering of events across multiple processes.

**How:** Each process maintains a vector of counters (one per process). When a process does local work, it increments its own counter. When it sends a message, it attaches its current vector. When it receives a message, it takes the element-wise maximum of its vector and the received vector. Two events are concurrent if neither vector dominates the other. This detects causality: "event A happened before event B" vs. "events A and B are concurrent."

**Relevance: Medium.** Vector clocks (or their derivatives) help Benten detect concurrent edits during sync. If two instances modify the same Node, vector clocks tell us whether one edit causally preceded the other (in which case we take the later one) or they're concurrent (in which case we need CRDT merge rules). The main downside: vector clock size grows with the number of participants.

---

### 3.8 Hybrid Logical Clocks (HLC)

**What:** A clock that combines physical wall-clock time with a logical counter, providing both causality tracking and real-time-closeness.

**How:** An HLC timestamp has two components: (1) the maximum physical time seen so far, and (2) a logical counter that breaks ties when physical times are equal. When receiving a message, the node updates to max(local physical time, local HLC time, received HLC time). This produces timestamps that are causally ordered AND close to real wall-clock time -- unlike pure vector clocks which have no time component.

**Relevance: High -- SHOULD ADOPT.** HLCs are the best fit for Benten's version chain timestamps:
- They provide causal ordering (necessary for CRDT merge semantics).
- They stay close to wall-clock time (necessary for "when was this last edited?" UI).
- They are compact (two numbers instead of a vector that grows with participants).
- Used by CockroachDB, YugabyteDB, and many CRDT implementations.

**Benten integration:** Each version chain commit should carry an HLC timestamp. During sync, HLC timestamps determine causal ordering. For LWW (Last Writer Wins) conflict resolution, the HLC timestamp is the tiebreaker. The Rust `hlc` crate provides a solid implementation.

---

## 4. Identity

How entities are identified without centralized authorities.

### 4.1 Decentralized Identifiers (DIDs) -- W3C Standard

**What:** A W3C standard for globally unique identifiers that don't require a central registry.

**How:** A DID is a URI (e.g., `did:key:z6Mk...`) that resolves to a DID Document containing public keys, service endpoints, and authentication methods. The DID method (the part after `did:`) determines how resolution works. Over 100 DID methods exist. W3C published DID v1.1 as a Candidate Recommendation in March 2026. IETF standardization is in progress (January 2026 charter published).

**Relevance: High -- SHOULD ADOPT.** DIDs are the identity standard for Benten:
- Each instance has a DID (its cryptographic identity).
- Each user has a DID (their identity across instances).
- DIDs are the "subject" in capability grants ("this DID can read this subgraph").
- AT Protocol uses DIDs. UCAN uses DIDs. The entire decentralized identity ecosystem converges on DIDs.

**Benten integration:** Use `did:key` for self-certifying identities (derived directly from public key, no blockchain needed). Support `did:web` for organizations that want DNS-verifiable identity. Potentially support `did:plc` for AT Protocol interop. The DID Document stored in the graph contains the instance's public keys and sync endpoints.

---

### 4.2 Verifiable Credentials (VCs)

**What:** A W3C standard for digitally signed claims about a subject.

**How:** An issuer (e.g., a university) creates a credential (e.g., "this person has a degree"), signs it with their DID, and gives it to the subject. The subject can present the credential to any verifier, who checks the issuer's signature without contacting the issuer. The subject holds the credential in a wallet and controls who sees it. EU's EUDI Wallet mandates VC acceptance by banks, telecoms, and healthcare providers by 2027.

**Relevance: Medium (future).** VCs enable Benten users to:
- Present credentials from external issuers (e.g., "I'm verified as a real person" from BrightID).
- Issue credentials within their community (e.g., "this user is a trusted moderator").
- Gate capabilities based on credentials (e.g., "only users with a verified-adult credential can access this subgraph").

Not a Phase 1 priority, but the architecture should be VC-ready: capabilities could accept VCs as proof of eligibility.

---

### 4.3 Self-Sovereign Identity (SSI)

**What:** The principle that individuals should own and control their digital identities without relying on centralized authorities.

**How:** SSI is not a specific technology but a design philosophy implemented through DIDs + VCs + wallets. The user generates their own keypair (no "sign up" with a provider), holds credentials in a local wallet, and presents them selectively. No central authority can revoke, surveil, or censor their identity.

**Relevance: High -- CORE PHILOSOPHY.** SSI is Benten's identity philosophy stated in different words. "Every person, family, or organization runs their own instance. Data is owned by the user." SSI provides the vocabulary and standards ecosystem for implementing this vision. We should explicitly position Benten as an SSI-compatible platform.

---

### 4.4 Key-Based Identity (did:key)

**What:** A DID method where the identifier IS the public key -- no resolution infrastructure needed.

**How:** `did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK` -- the string after `z6Mk` is the base58-encoded Ed25519 public key. To resolve it, you just decode the key from the DID string. No blockchain, no DNS, no HTTP. Self-certifying, offline-capable, zero infrastructure.

**Relevance: High -- SHOULD ADOPT as default.** `did:key` with Ed25519 should be the default identity scheme for Benten:
- Zero infrastructure requirement (works offline, works without internet).
- Self-certifying (the DID IS the public key, unforgeable).
- Instant creation (generate keypair = have a DID).
- AT Protocol compatibility (AT Protocol's did:key support).

The limitation: did:key identifiers are permanently bound to one key. Key rotation requires a new DID. For instances that need key rotation, support `did:web` or `did:plc` as alternatives.

---

### 4.5 Web of Trust (PGP, Keybase model)

**What:** A decentralized trust model where users vouch for each other's identities.

**How:** Instead of certificate authorities (centralized trust), users sign each other's public keys to vouch for identity. If Alice trusts Bob, and Bob has signed Carol's key, Alice can transitively trust Carol. PGP pioneered this in the 1990s. Keybase modernized it (linking crypto keys to social accounts). The model works well in small communities but suffers from "long chain" trust degradation.

**Relevance: Medium.** Web-of-trust maps naturally to Benten's graph model:
- Trust relationships ARE edges in the graph (User A --trusts--> User B).
- Transitive trust IS graph traversal.
- Trust levels can be edge properties (how much does A trust B?).
- The graph engine's IVM can maintain materialized trust scores.

This is architecturally natural for Benten but not a Phase 1 priority. It becomes relevant when building cross-instance discovery ("should I accept a sync request from this unknown instance? Let me check my web of trust.").

---

### 4.6 Sybil Resistance / Proof of Personhood

**What:** Mechanisms to prevent one entity from creating multiple fake identities.

**How:** Multiple approaches exist in 2026:
- **Worldcoin/World ID**: Iris scan via hardware Orb, cryptographic hash of biometric. 2M+ users.
- **BrightID**: Social graph analysis (real humans have organic connection patterns, bots cluster).
- **Proof of Humanity**: Video submission + community vouching.
- **Gitcoin Passport**: Credential aggregation (GitHub, ENS, POAPs, social profiles).
- **Idena**: Synchronous cognitive tests (CAPTCHA-like "flips" at a global synchronized time).
- **Polkadot PoP**: ZK-based unique identity verification.

**Relevance: Medium (future).** Sybil resistance becomes critical when Benten has:
- **Quadratic voting in community governance** (only works if one-person-one-vote is enforced).
- **Public marketplaces** (prevent review manipulation).
- **Resource allocation** (prevent one user from claiming multiple "free tier" instances).

The modular approach (Gitcoin Passport style) is most aligned with Benten's philosophy -- accept multiple signals rather than one centralized biometric. This could be implemented as a VC-based capability requirement: "to vote, present a credential proving unique personhood."

---

## 5. Authorization

How permissions and access are controlled without centralized authorities.

### 5.1 UCAN (User Controlled Authorization Networks)

**What:** A decentralized authorization scheme based on signed capability tokens that can be delegated without contacting the original authority.

**How:** A UCAN is a signed JWT-like token that says "DID A grants DID B the capability to [action] on [resource] until [expiry]." B can delegate a subset of its authority to C by creating a new UCAN with A's original UCAN as proof. Verification is local -- you walk the chain of proofs to verify authority. No server roundtrip needed. Works offline.

**Relevance: High -- SHOULD ADOPT.** UCAN is the authorization standard that best fits Benten's architecture:
- **Offline-capable**: Verify permissions locally by walking the proof chain.
- **Delegable**: A user can share a subset of their capabilities with another user without admin intervention.
- **DID-native**: UCANs are issued to and by DIDs.
- **Principle of least authority**: Delegation can narrow scope, never widen.
- **Used by**: Storacha (IPFS pinning), Fission (web-native apps).

**Benten integration:** UCANs replace the current flat RBAC + AgePermissionStore. A capability grant is a UCAN stored as a Node in the graph. Verification is graph traversal: walk the delegation chain, check each signature, verify the authority narrows at each step. The IVM materializes "effective permissions" as a view, so runtime checks are O(1).

This is the single most important standard for Benten to adopt. The spec already calls for "UCAN-compatible capability grants."

---

### 5.2 ZCAP-LD (Authorization Capabilities for Linked Data)

**What:** A W3C Community Group specification for capability-based authorization using Linked Data (JSON-LD).

**How:** Similar to UCAN but uses JSON-LD formatting, URL-based addressing (instead of CIDs), and Linked Data Proofs (instead of JWT signatures). Supports delegation chains like UCAN. Developed by the W3C Credentials Community Group.

**Relevance: Low-Medium.** ZCAP-LD and UCAN solve the same problem with different formats. UCAN has stronger momentum in the decentralized app ecosystem and aligns better with Benten's CID-based data model. ZCAP-LD is more relevant if you're building on RDF/Linked Data (we're not). However, understanding ZCAP-LD informs the capability design -- some of its delegation semantics are more formally specified than UCAN's.

---

### 5.3 Object Capability Model (OCap)

**What:** A security model where access is controlled by possession of unforgeable references ("capabilities") rather than by identity-based access control lists.

**How:** In OCap, you can only interact with something if you hold a capability (an unforgeable reference) to it. Capabilities can be passed to others (delegation) but cannot be guessed or forged. This is fundamentally different from ACLs where a guard checks "is this user on the list?" -- in OCap, holding the capability IS the authorization. Implemented in: E programming language, Spritely Goblins, Agoric.

**Relevance: High -- CORE ARCHITECTURE.** OCap is the theoretical model that UCAN implements. Benten's capability system IS an object capability system:
- Holding a capability token (UCAN) IS the authorization -- no separate ACL check.
- Capabilities are unforgeable (cryptographically signed).
- Capabilities can be delegated (UCAN proof chains).
- Capabilities can be attenuated (narrowed in scope at each delegation).

The Benten spec already describes this model. UCAN is the concrete implementation of OCap for decentralized systems.

---

### 5.4 Macaroons (Google)

**What:** A bearer token authorization scheme with contextual caveats that can be added by intermediaries.

**How:** A macaroon is an HMAC-based token. The original issuer creates a base macaroon. Any holder can add "caveats" (restrictions) by chaining HMACs -- e.g., "valid only for read operations" or "valid only until midnight." Caveats can only narrow authority, never widen it. Verification requires the original secret key (unlike UCAN which uses public-key verification).

**Relevance: Low.** Macaroons solve the same problem as UCAN (delegable, attenuable authorization) but with a key difference: they require a central secret for verification. This makes them unsuitable for decentralized systems where you want any peer to verify without contacting the issuer. UCAN's public-key approach is strictly better for Benten's use case. Macaroons are worth understanding conceptually (the caveat-chaining pattern is elegant) but not an adoption target.

---

## 6. Networking

How nodes discover each other and communicate.

### 6.1 libp2p

**What:** A modular P2P networking stack that handles transport, discovery, NAT traversal, and multiplexing.

**How:** libp2p provides pluggable components: transports (TCP, QUIC, WebSocket, WebTransport, WebRTC), discovery (mDNS, DHT, rendezvous), NAT traversal (relay, hole-punching), multiplexing (yamux, mplex), security (Noise, TLS), and pub/sub (GossipSub). Each peer has a PeerId derived from its public key. Used by: IPFS, Filecoin, Ethereum (consensus layer), Polkadot. 300+ active contributors, 10+ implementations across languages.

**Relevance: High -- SHOULD ADOPT.** libp2p should be Benten's networking layer:
- **Solves all the hard problems**: NAT traversal, peer discovery, encrypted transport, multiplexing -- these are notoriously difficult to build correctly.
- **Rust implementation** (`rust-libp2p`): High quality, actively maintained, used by Substrate/Polkadot.
- **PeerId = DID-compatible**: libp2p peer IDs are derived from public keys, which can be used to construct `did:key` identifiers.
- **GossipSub**: A gossip protocol optimized for pub/sub messaging -- useful for propagating graph changes across a mesh of peers.

**Benten integration:** The sync layer uses libp2p for all peer-to-peer communication. Discovery via Kademlia DHT. Transport negotiation handles heterogeneous networks (QUIC for server-to-server, WebRTC for browser peers, WebTransport where supported). GossipSub for real-time event propagation.

---

### 6.2 Gossip Protocols

**What:** Decentralized communication protocols where nodes spread information by randomly telling other nodes, like rumors spreading through a population.

**How:** Each node periodically selects random peers and shares its latest state or new messages. Recipients do the same. Information propagates exponentially -- after O(log n) rounds, all nodes have the information with high probability. Three types: anti-entropy (sync full state), rumor-mongering (spread new updates), and epidemic broadcast (flood as fast as possible).

**Relevance: High -- WILL USE (via libp2p GossipSub).** Gossip protocols are how Benten instances learn about changes from other instances in a mesh network. When an instance modifies a subgraph, it gossips the change to its known peers, who gossip it to their peers. libp2p's GossipSub provides a production-grade gossip implementation.

**Benten integration:** Used for real-time event propagation across synced instances. Not for the sync protocol itself (which is more targeted -- Merkle tree comparison between specific peers), but for "notify peers that something changed so they know to initiate a sync."

---

### 6.3 Yggdrasil (Mesh Networking)

**What:** An experimental encrypted IPv6 mesh network that provides end-to-end encrypted addressing without NAT.

**How:** Each node generates a keypair and derives an IPv6 address from its public key. Nodes connect to peers (via TCP, TLS, or other transports) and form an overlay mesh. Routing uses a compact tree-based scheme. Every address is directly reachable -- no NAT traversal needed because the overlay network handles it. Traffic is end-to-end encrypted between any two nodes.

**Relevance: Low-Medium.** Yggdrasil is interesting as an infrastructure layer but operates at a different level than Benten needs. libp2p already handles NAT traversal and encrypted transport at the application level. Yggdrasil would be more relevant if Benten were building physical mesh networks (e.g., community mesh wifi). However, the concept of "address derived from public key" is the same as `did:key` -- validating our identity model.

---

### 6.4 WebRTC (Browser P2P)

**What:** A browser-native API for real-time peer-to-peer communication.

**How:** WebRTC handles NAT traversal via ICE (STUN/TURN servers), establishes encrypted data channels and media streams between browsers. The connection setup requires a signaling server (to exchange offers/answers), but once established, data flows directly between peers.

**Relevance: Medium.** WebRTC is how Benten's browser client (SvelteKit app) would participate in P2P sync without requiring a server intermediary. When a user opens the Benten web interface, their browser could directly sync with their instance's peers via WebRTC data channels. libp2p supports WebRTC as a transport, so this integrates naturally.

**Benten integration:** Browser peers connect via libp2p's WebRTC transport. The SvelteKit app uses the same sync protocol as native peers, just over a different transport.

---

### 6.5 Tor / I2P (Anonymous Networking)

**What:** Overlay networks that route traffic through multiple relays to hide the sender's identity.

**How:**
- **Tor**: Onion routing -- traffic is encrypted in layers and routed through 3+ relays. Each relay peels one layer, learning only the next hop. The exit node connects to the destination. Provides sender anonymity.
- **I2P**: Garlic routing -- similar concept but optimized for internal services (within the I2P network) rather than accessing the public internet. Provides both sender and receiver anonymity.

**Relevance: Low (optional).** For most Benten instances, anonymous networking is not needed -- you're syncing with known, trusted peers. However, for censorship-resistant deployment (e.g., activists, journalists), the ability to run a Benten instance as a Tor hidden service or over I2P would be a powerful feature. This is an optional transport layer addition, not a core architecture decision.

---

### 6.6 NAT Traversal / Hole Punching

**What:** Techniques for establishing direct connections between two devices behind NATs (home routers, corporate firewalls).

**How:** STUN servers help peers discover their public IP/port mapping. Hole punching coordinates simultaneous connection attempts so that both NATs open a path. TURN servers relay traffic when direct connection fails. ICE orchestrates trying STUN, then hole punching, then TURN as fallback. libp2p's implementation handles all of this.

**Relevance: High -- WILL USE (via libp2p).** Most Benten instances will be behind NATs (home networks, mobile networks, corporate firewalls). NAT traversal is essential for direct P2P sync. libp2p handles this, but we need to understand it to configure relay infrastructure for cases where direct connection fails.

---

## 7. Storage

How data is stored and retrieved without centralized servers.

### 7.1 IPFS (InterPlanetary File System)

**What:** A peer-to-peer content-addressed storage and distribution protocol.

**How:** Files are split into blocks, each identified by its CID (content hash). Blocks are stored in a local blockstore and announced to the network via DHT. Retrieval uses the CID -- any peer holding the block can serve it. Deduplication is automatic (same content = same CID). Helia is the current TypeScript implementation. Bitswap handles block exchange between peers. Trustless Gateways enable verified retrieval over HTTPS.

**Relevance: Medium.** IPFS is the largest content-addressed storage network. For Benten:
- **Media storage**: Large files (images, videos) could be stored on IPFS and referenced by CID in the graph. This offloads binary blob storage from the graph engine.
- **Subgraph distribution**: Published subgraphs (e.g., a public website) could be pinned to IPFS for CDN-like distribution.
- **CID compatibility**: Benten's content-addressed hashing should produce CIDs compatible with IPFS/IPLD so data can flow between systems.

**Benten integration:** Graph engine handles structured data (Nodes, Edges, properties). Large binary blobs reference IPFS CIDs. The graph stores the CID as a property; IPFS stores the actual bytes.

---

### 7.2 Arweave (Permanent Storage)

**What:** A decentralized storage network that provides permanent data storage for a one-time payment.

**How:** You upload data, pay once (an endowment that generates ongoing storage rewards), and the data is replicated across the network indefinitely. Uses a "proof of access" consensus where miners must prove they store random historical data. 347 TiB stored, ~33 GiB daily uploads as of early 2026. Arweave AO (launched 2025) adds a hyper-parallel computing layer on top.

**Relevance: Low.** Arweave solves permanent archival storage. For Benten, this could be relevant for:
- **Regulatory compliance**: Immutable audit logs stored permanently.
- **Digital preservation**: Community archives that must outlive any single instance.

But Benten's core use case (user-owned, editable, syncing data) is fundamentally mutable -- the opposite of Arweave's immutability. Arweave is a possible integration for specific use cases, not a core architectural component.

---

### 7.3 Filecoin (Incentivized Storage)

**What:** A decentralized storage marketplace built on IPFS where storage providers earn tokens for storing and serving data.

**How:** Storage providers prove they're storing data correctly (proof of replication + proof of spacetime). Clients pay recurring fees for storage deals. Filecoin "Onchain Cloud" mainnet launched January 2026. Costs are $200-1,000/TB/year. The network provides economic incentives for reliable storage.

**Relevance: Low.** Same reasoning as Arweave -- Filecoin provides decentralized blob storage with economic incentives. Could be useful for hosting media files associated with Benten data, but not architecturally central. The economic model (recurring payments in cryptocurrency) adds complexity that most Benten instances don't need.

---

### 7.4 BitTorrent

**What:** The original decentralized file distribution protocol.

**How:** Files are split into pieces. A torrent file (or magnet link) describes the pieces and their hashes. Peers download pieces from each other, verifying each piece's hash. The more popular a file, the more peers serve it -- naturally scaling bandwidth with demand. DHT enables trackerless peer discovery.

**Relevance: Low.** BitTorrent optimizes for distributing large, complete files to many recipients. Benten's sync model is different -- incremental subgraph updates between specific peers, not broadcasting complete datasets. The concepts (content hashing, piece verification, DHT-based discovery) are relevant but better served by IPFS + libp2p which provide the same capabilities in a more modern, programmatic form.

---

### 7.5 Content-Addressed Storage (General Concept)

**What:** The principle of identifying data by its cryptographic hash rather than by location.

**How:** Hash the data, use the hash as the identifier. Any copy of the data is equivalent (same hash = same data). This provides integrity (verify by re-hashing), deduplication (same content = same identifier), and location independence (retrieve from any source that has it).

**Relevance: High -- ALREADY BUILDING.** This is a foundational principle of Benten, not a technology to adopt. Covered in detail in Section 3.2.

---

## 8. Tokens and Economics

Financial primitives for decentralized systems.

### 8.1 Fungible Tokens (ERC-20 equivalent)

**What:** Interchangeable digital assets where each unit is identical (like currency).

**How:** A smart contract maintains a ledger of balances. Users can transfer tokens, check balances, and approve others to spend on their behalf. ERC-20 is the Ethereum standard (over $150B in circulating ERC-20 tokens). Every unit of the same token is equivalent.

**Relevance: Medium (future).** Fungible tokens become relevant when Benten needs:
- **Community credits**: A community issues tokens for contributions (content creation, moderation, support).
- **Marketplace payments**: Buyers pay sellers in community-issued tokens.
- **Staking**: Users stake tokens to gain governance weight or access privileges.

Benten doesn't need to issue tokens on Ethereum. The graph engine can natively represent token balances as Nodes with properties, with transfer operations validated by capability rules. "Token" is just a Node type with specific semantics.

---

### 8.2 Non-Fungible Tokens (NFTs / ERC-721 equivalent)

**What:** Unique digital assets where each token is distinct and non-interchangeable.

**How:** Each token has a unique ID and can hold metadata. Ownership is tracked by the smart contract. Used for: art, collectibles, event tickets, domain names, certifications.

**Relevance: Medium (future).** In Benten's graph model, every Node is already a unique, identifiable entity with an owner. "NFT" semantics are native to the graph:
- A Node with a `non-transferable` capability = a soulbound token.
- A Node with a `transfer` capability = an NFT.
- Provenance = version chain history.

Benten should NOT build on Ethereum's NFT standards. Instead, it should provide equivalent functionality natively: unique, owned, transferable, provenance-tracked digital assets -- all within the graph.

---

### 8.3 Semi-Fungible Tokens (ERC-1155 equivalent)

**What:** Tokens that can be both fungible and non-fungible in a single contract.

**How:** ERC-1155 defines a multi-token standard where each token ID can have a supply > 1 (making it fungible within that type) or supply = 1 (making it non-fungible). Enables efficient batch transfers and mixed-type collections.

**Relevance: Low-Medium (future).** The graph engine naturally handles this: a Node type can have a `supply` property. If supply = 1, it's unique. If supply > 1, units are fungible within that type. No special standard needed -- it's a property of the data model.

---

### 8.4 Token Bonding Curves

**What:** Mathematical functions that determine a token's price based on its supply.

**How:** Tokens are minted (bought) and burned (sold) against a curve implemented in a smart contract. As more tokens are minted, the price increases along the curve. As tokens are burned, the price decreases. The curve acts as an automated market maker with guaranteed liquidity. In 2026, bonding-curve DEXs hold $142.7B in locked value.

**Relevance: Low (future).** Bonding curves could be relevant if a Benten community issues its own token with automated pricing (e.g., a creator's "membership token" that increases in price as more people join). This is a Phase 3+ feature, and the curve logic would be a module (an operation subgraph in the engine's terms), not a core engine feature.

---

### 8.5 Automated Market Makers (AMMs)

**What:** Algorithms that provide liquidity for token trading without traditional order books.

**How:** Liquidity providers deposit token pairs into a pool. The AMM uses a mathematical formula (typically x*y=k for constant-product) to determine exchange rates. Traders swap against the pool, with the formula adjusting prices based on supply/demand. No counterparty matching needed.

**Relevance: Low (future).** Only relevant if Benten builds a decentralized marketplace with multiple community tokens that need to be exchanged. Very far-future. The concept is interesting as a graph operation (the AMM formula is a materialized view over pool balances), but not a near-term priority.

---

### 8.6 Staking and Slashing

**What:** Locking tokens as collateral to participate in a system, with automatic penalties for misbehavior.

**How:** Users "stake" (lock) tokens, gaining rights (voting power, validator role, access). If they violate rules (double-signing, extended downtime, malicious proposals), a portion of their stake is "slashed" (destroyed or redistributed). Creates economic incentives for good behavior.

**Relevance: Medium (future).** Staking/slashing maps naturally to Benten's community governance:
- **Moderation staking**: Moderators stake reputation tokens. Bad moderation decisions result in slashing (reputation loss).
- **Content curation**: Users stake on content quality predictions. Accurate curation earns rewards.
- **Sync participation**: Instances that consistently serve data reliably earn reputation.

The graph engine can represent stakes as Nodes with lock conditions, and slashing as capability-triggered operations. This is a Phase 3+ feature.

---

### 8.7 Token-Curated Registries (TCR)

**What:** Decentralized lists maintained through token-incentivized curation.

**How:** To add an item to a registry, you stake tokens on it. Others can challenge by staking tokens against it. A voting process resolves challenges. Winners receive the losers' stakes. This creates economic incentives for accurate curation. Used for: curated lists of quality content, verified business directories, grant eligibility lists.

**Relevance: Medium (future).** TCRs map directly to Benten's composable registry model:
- A community-curated "trusted modules" registry where developers stake reputation to list a module.
- A content quality registry where curators stake on recommended content.
- The graph engine's registry primitives + capability-based staking make this implementable as a module.

---

## 9. Governance

How decentralized communities make collective decisions.

### 9.1 On-Chain Governance (Compound, Tally)

**What:** Governance where proposals, votes, and execution all happen on a blockchain via smart contracts.

**How:** Anyone holding governance tokens can create proposals (code changes, treasury allocations, parameter adjustments). Token holders vote during a voting period. If the proposal passes quorum and threshold, it executes automatically on-chain. Fully transparent and auditable.

**Relevance: Medium (future).** Benten communities need governance, but ON-CHAIN is the wrong framing for us. The equivalent for Benten: proposals and votes are Nodes in the graph, with capability-gated participation and graph-executed outcomes. The principles (transparent proposals, auditable votes, automated execution) are core to Benten's governance vision; the blockchain-specific implementation is not.

---

### 9.2 Snapshot Voting (Off-Chain, Signed)

**What:** Gasless voting where votes are signed messages stored off-chain.

**How:** Users sign their votes with their wallet keys. Votes are stored on IPFS or a centralized server. The signature proves who voted and how. No gas fees because nothing happens on-chain. Results are tallied off-chain. Used by most DAOs for routine decisions because it's free and fast.

**Relevance: High -- SHOULD STUDY.** Snapshot's model maps almost perfectly to Benten:
- Users sign votes with their DID keys (equivalent to wallet signatures).
- Votes are Nodes in the graph (equivalent to IPFS storage but with richer structure).
- Tallying is an IVM materialized view (automatically maintained as votes arrive).
- No "gas" because there's no blockchain -- just graph operations.

Benten's governance system should provide Snapshot-equivalent functionality natively. The graph model makes this natural.

---

### 9.3 Quadratic Voting / Quadratic Funding

**What:** A voting system where the cost of votes increases quadratically, preventing plutocratic domination.

**How:** Each voter has a budget of "voice credits." Casting 1 vote costs 1 credit, 2 votes costs 4 credits, 3 votes costs 9 credits, etc. This means passionate minorities can express strong preferences without being drowned out by wealthy majorities. Quadratic Funding applies the same principle to public goods funding (Gitcoin Grants). Requires Sybil resistance to prevent gaming.

**Relevance: Medium (future).** Quadratic voting is the most promising governance mechanism for Benten communities:
- Prevents wealthy users from dominating decisions.
- Allows expressing intensity of preference (not just yes/no).
- Natural fit for resource allocation (which features to build, which content to promote).
- Requires proof of personhood (Section 4.6) to prevent Sybil attacks.

Implementable as a module: the voting formula is a graph operation, credits are Node properties, tallying is a materialized view.

---

### 9.4 Conviction Voting

**What:** A governance mechanism where voting power accumulates over time based on sustained commitment.

**How:** Instead of a fixed voting period, users continuously allocate tokens to proposals they support. Their voting power for that proposal increases over time the longer they keep tokens allocated. Moving tokens to a different proposal resets the accumulation. This rewards sustained conviction and reduces governance fatigue.

**Relevance: Medium (future).** Conviction voting is well-suited for ongoing resource allocation decisions in Benten communities:
- "Which features should the community fund?" -- sustained interest wins over flash voting.
- Naturally represented as time-weighted edges in the graph.
- IVM maintains running conviction scores in real-time as users adjust allocations.

---

### 9.5 Futarchy (Decision Markets)

**What:** Governance where elected representatives set goals, but prediction markets determine which policies achieve those goals.

**How:** The community defines a welfare metric (e.g., "monthly active users"). For each proposed policy, a prediction market opens: "What will the welfare metric be IF this policy is adopted?" The policy with the highest predicted outcome is adopted. If the prediction was wrong, market participants lose their stake.

**Relevance: Low (future).** Futarchy is intellectually fascinating but complex to implement and untested at scale. It requires liquid prediction markets, clear welfare metrics, and sophisticated participants. Could become relevant for large Benten ecosystems making strategic decisions, but it's a Phase 4+ experiment at best.

---

### 9.6 Liquid Democracy (Delegated Voting)

**What:** A hybrid of direct and representative democracy where voters can delegate their voting power to anyone, who can further delegate.

**How:** Each participant can either vote directly on any issue OR delegate their vote to a trusted representative. Representatives can further delegate, creating delegation chains. Delegation is revocable at any time. For any specific issue, a participant can override their delegate and vote directly. Used by: Gitcoin, some DAOs.

**Relevance: High (future).** Liquid democracy is deeply aligned with Benten's graph model:
- Delegation chains ARE graph paths (User A --delegates-to--> User B --delegates-to--> User C).
- Graph traversal computes effective voting power.
- IVM maintains real-time delegation aggregates.
- Capability-based: delegation is a UCAN ("I grant you my voting capability for governance topic X").

This is one of the governance mechanisms Benten should implement early because it maps so naturally to the graph data model.

---

### 9.7 Rage Quit (Moloch DAO Pattern)

**What:** A mechanism that lets dissenting members exit a DAO with their proportional share of the treasury.

**How:** After a vote passes, there's a grace period during which members who voted against (or didn't vote) can "rage quit" -- burn their shares and withdraw their proportional share of the treasury assets. This provides a credible exit option that constrains majority tyranny: if a proposal is too aggressive, enough members will rage quit that it becomes uneconomical.

**Relevance: Medium (future).** Rage quit is a powerful governance primitive for Benten communities:
- If a community makes a decision a member disagrees with, they can fork their data and leave with it -- this IS rage quit at the data level.
- The economic version (withdrawing proportional treasury) applies to communities with shared resources.
- Benten's sync model enables this naturally: your data is your data, you can fork at any time.

The insight: Benten's architecture already provides the data-level equivalent of rage quit. "Either party can fork at any time" (from the vision statement) IS rage quit.

---

## 10. Smart Contracts and Computation

How code executes in decentralized systems.

### 10.1 EVM (Ethereum Virtual Machine)

**What:** A deterministic, stack-based virtual machine that executes smart contracts on Ethereum.

**How:** Smart contracts are compiled to EVM bytecode, deployed on-chain, and executed by every node. Execution is metered by gas (prevents infinite loops, pays for computation). State changes are atomic (all-or-nothing per transaction). The EVM is the most widely deployed smart contract runtime (thousands of applications, billions in TVL).

**Relevance: Low.** The EVM is designed for trustless, global execution on a shared blockchain. Benten doesn't need global consensus on computation -- each instance executes its own logic locally. However, the EVM's concepts (deterministic execution, metered computation, atomic state changes) inform Benten's operation subgraph model. The key difference: Benten operations execute on a local graph, not a shared global state.

---

### 10.2 CosmWasm (WASM-Based Smart Contracts)

**What:** A WASM-based smart contract framework for the Cosmos ecosystem.

**How:** Contracts are written in Rust, compiled to WASM, and deployed on Cosmos SDK chains. CosmWasm provides a sandboxed execution environment with deterministic semantics and fine-grained capability controls. More efficient than EVM (native types, no 256-bit arithmetic overhead).

**Relevance: Medium -- CONCEPTUALLY RELEVANT.** CosmWasm's approach (Rust -> WASM, sandboxed, capability-controlled) is architecturally similar to Benten's module execution model. The key parallel: both use WASM for sandboxed third-party code execution. Benten's spec references `@sebastianwessel/quickjs` for JavaScript sandbox, but WASM (via CosmWasm-like patterns) would provide better performance and security for Rust-native modules.

---

### 10.3 Move Language (Aptos/Sui)

**What:** A Rust-inspired smart contract language designed for safety and resource-oriented programming.

**How:** Move treats digital assets as "resources" that cannot be copied or implicitly discarded -- they can only be moved between storage locations. This prevents entire classes of bugs (double-spending, accidental destruction). Aptos uses an account+resource model; Sui uses an object model where assets have unique IDs. MonoMove (2026) redesigns the VM for parallelism.

**Relevance: Medium -- CONCEPTUALLY RELEVANT.** Move's resource model is philosophically aligned with Benten's graph model:
- A Benten Node is a resource: unique identity, owned, transferable.
- Move's "resources can't be copied, only moved" maps to Benten's "Nodes have single owners and transfer is an explicit operation."
- Sui's object model (unique IDs, direct state access) is very similar to Benten's Node model.

We should study Move's type system for inspiration on how to enforce resource safety in operation subgraphs.

---

### 10.4 Solana's BPF/SBF

**What:** Solana's runtime for smart contracts (called "programs"), based on Berkeley Packet Filter.

**How:** Programs are compiled to SBF bytecode (Solana BPF Fork) and executed in a sandboxed VM. Solana's parallel transaction processing (Sealevel) enables high throughput by identifying transactions that don't share state and executing them concurrently.

**Relevance: Low.** Solana's specific runtime is not relevant, but its parallel execution model (identify independent transactions, run them concurrently) directly informs Benten's IVM design. When multiple graph operations don't touch overlapping Nodes/Edges, they can execute in parallel. This is an insight to borrow, not a technology to adopt.

---

### 10.5 Operation Subgraphs (What Benten Is Building)

**What:** Computation defined as graph structures that the engine executes, not external code that queries a database.

**How:** In Benten, an "operation" is a subgraph pattern: nodes representing computation steps, edges representing data flow and dependencies. The engine traverses the operation subgraph, executing each step and flowing data along edges. This unifies code and data -- operations ARE data in the graph. IVM ensures that operation outputs are incrementally maintained as inputs change.

**Relevance: High -- THIS IS BENTEN.** This is the core innovation. Unlike every other system in this document (which separates "smart contract code" from "data the code operates on"), Benten treats computation as a first-class part of the graph. This enables:
- Operations are versionable, forkable, and syncable (they're just graph data).
- Capabilities control which operations can run on which data.
- IVM makes operation outputs reactive (the result updates when the input changes).
- Modules define operations as subgraph patterns, not external code blobs.

---

## 11. Other Infrastructure

### 11.1 Oracles (Chainlink)

**What:** Services that bring external data (price feeds, weather, sports results) into decentralized systems.

**How:** A decentralized oracle network (DON) consists of multiple independent node operators who each fetch data from external sources, aggregate results, and commit the consensus value on-chain. Chainlink is the dominant provider ($18B monthly cross-chain volume via CCIP in 2026), also providing cross-chain interoperability. SWIFT partnership enables traditional banks to interact with blockchain assets.

**Relevance: Low.** Benten instances can fetch external data directly (they're full applications, not constrained smart contracts). The oracle problem ("how do I trust external data?") only matters when you need consensus among untrusted parties about external facts. In Benten, the instance operator controls which data sources they trust. However, if Benten communities make governance decisions based on external data (e.g., "fund this project if it reaches 1000 users"), a lightweight oracle pattern could be useful.

---

### 11.2 Bridges (Cross-Chain Communication)

**What:** Infrastructure that enables different blockchain networks to communicate and transfer assets.

**How:** Various approaches: lock-and-mint (lock on chain A, mint equivalent on chain B), message passing (relay signed messages between chains), light client verification (verify chain A's state on chain B using light client proofs). Chainlink's CCIP is becoming the institutional standard, processing $18B/month across 15+ chains.

**Relevance: Low.** Benten's cross-instance sync is not a "bridge" -- it's native subgraph replication between instances running the same engine. The bridge concept is relevant only if Benten needs to interact with blockchain networks (e.g., verifying an Ethereum-issued credential, receiving a payment on Solana). This would be a module, not a core feature.

---

### 11.3 Rollups (Layer 2 Scaling)

**What:** Scaling solutions that execute transactions off-chain but post proofs/data to a base layer blockchain.

**How:**
- **Optimistic Rollups**: Assume transactions are valid; a 7-day challenge period allows fraud proofs. (Arbitrum, Optimism, Base)
- **ZK Rollups**: Generate zero-knowledge proofs for every batch, verified instantly on L1. (zkSync, StarkNet, Polygon zkEVM)

L2 networks collectively hold $45B+ TVL in March 2026, processing more transactions than Ethereum mainnet.

**Relevance: Low.** Rollups solve blockchain scaling -- Benten doesn't have a blockchain to scale. However, the concept of "execute locally, prove to others" is philosophically relevant to Benten's sync model: an instance executes operations on its local graph and provides cryptographic proof (signed Merkle root) that the result is correct when syncing to peers.

---

### 11.4 Sidechains

**What:** Independent blockchains connected to a main chain via bridges.

**How:** A sidechain has its own consensus, security, and rules. Assets are transferred between the main chain and sidechain via a two-way bridge. The sidechain can optimize for different tradeoffs (speed, privacy, low fees) at the cost of not inheriting the main chain's security.

**Relevance: None.** Benten doesn't have a "main chain" and "sidechains." All instances are equal peers with their own data. No relevance.

---

### 11.5 State Channels

**What:** Off-chain transaction channels where two parties transact freely and only settle the final state on-chain.

**How:** Two parties open a channel by locking funds on-chain. They exchange unlimited signed transactions off-chain (microsecond latency, zero fees). When done, they submit the final state to the chain. Disputes are resolved by submitting the latest signed state. Bitcoin's Lightning Network is the largest implementation (~5,000 BTC capacity).

**Relevance: Low.** State channels solve a blockchain-specific problem (on-chain transaction costs). For Benten, all transactions are "off-chain" by default -- they happen in the local graph engine. The concept of "exchange signed state updates between two parties" does describe Benten's peer-to-peer sync, but we don't need the dispute resolution mechanism because both parties maintain their own authoritative copies.

---

### 11.6 IPLD (InterPlanetary Linked Data)

**What:** A data model that unifies all hash-linked data structures (IPFS, Git, Ethereum, Bitcoin) into a single namespace.

**How:** IPLD defines a canonical data model where any hash-linked data can be addressed and traversed uniformly. A CID (Content Identifier) encodes the hash function, codec, and hash value. Different codecs handle different formats (dag-cbor, dag-json, dag-pb, git-raw, eth-block). Any IPLD link can be traversed regardless of the underlying protocol.

**Relevance: High -- SHOULD STUDY.** IPLD is the closest existing standard to Benten's "everything is content-addressed graph data" philosophy:
- IPLD's data model (maps, lists, links, typed values) maps to Benten's Node properties.
- IPLD links (CIDs) map to Benten's content-addressed Edge references.
- IPLD codecs could provide interoperability with external content-addressed systems.

**Benten integration:** Benten's serialization format for graph data should be compatible with IPLD's dag-cbor codec. This means Benten Nodes/Edges can be stored in IPFS, traversed by IPLD tools, and referenced by standard CIDs. This is not a core requirement but a powerful interoperability feature.

---

## 12. Summary and Strategic Recommendations

### What Benten Is ALREADY Building Equivalents Of

| Technology | Benten Equivalent | Notes |
|-----------|-------------------|-------|
| Content-Addressed Storage | Version chain hashing | Same principle: hash = identity |
| Merkle DAGs | Version chain structure | Commits link to parents via hashes |
| Append-Only Logs | Version chain (per Node) | "History IS the graph" |
| CRDTs | Property-level conflict resolution | Core sync mechanism |
| DAGs | Core data model + version chains | The graph IS a DAG |
| NFTs/Tokens | Nodes with ownership semantics | Every Node is already a unique, owned entity |
| Smart Contracts | Operation subgraphs | Computation as graph data |
| Rage Quit | "Fork at any time" | Data sovereignty enables exit |
| Oracle-free external data | Direct fetch (not a smart contract) | Instances are full applications |

### What Benten SHOULD ADOPT (Use Standards, Don't Reinvent)

| Technology | Why Adopt | Priority |
|-----------|----------|----------|
| **Ed25519** | Default signature algorithm. Faster, safer than secp256k1. Universal support. | **Phase 1** |
| **BLAKE3** | Default hash function for internal use. 5-10x faster than SHA-256. Rust-native. | **Phase 1** |
| **DIDs (did:key)** | Identity standard. Self-certifying, offline, zero infrastructure. | **Phase 1** |
| **UCAN** | Authorization standard. Offline-capable, delegable, DID-native. Best fit for capability model. | **Phase 1** |
| **libp2p** | Networking stack. Solves NAT traversal, discovery, transport. Rust implementation. | **Phase 1** |
| **Hybrid Logical Clocks** | Timestamp standard. Causality + wall-clock closeness. Compact. | **Phase 1** |
| **Kademlia DHT** | Peer/content discovery. Via libp2p, not standalone. | **Phase 1** |
| **GossipSub** | Change propagation across mesh. Via libp2p. | **Phase 1** |
| **Merkle Search Trees** | Study AT Protocol's MST for sync negotiation. Adapt for graph topology. | **Phase 1** |
| **IPLD (dag-cbor)** | Serialization interoperability with IPFS/IPLD ecosystem. | **Phase 2** |
| **Verifiable Credentials** | External identity attestation. VC-ready architecture now, full support later. | **Phase 2** |

### What Becomes Relevant in Future Phases

| Technology | Phase | Use Case |
|-----------|-------|----------|
| Zero-Knowledge Proofs | Phase 2+ | Privacy-preserving capability verification, selective disclosure |
| Threshold Signatures | Phase 2+ | Key recovery, multi-device security, community governance signing |
| Quadratic Voting | Phase 3 | Fair community governance |
| Liquid Democracy | Phase 3 | Delegated governance (natural graph traversal) |
| Conviction Voting | Phase 3 | Sustained-interest resource allocation |
| Snapshot-style Voting | Phase 3 | Lightweight proposal voting |
| Token Bonding Curves | Phase 3+ | Community token issuance |
| Staking/Slashing | Phase 3+ | Moderation incentives, curation incentives |
| Token-Curated Registries | Phase 3+ | Community-curated module/content quality lists |
| Sybil Resistance | Phase 3+ | Prerequisite for quadratic voting |
| BFT Principles | Phase 2+ | Sync validation with untrusted peers |
| MPC | Phase 3+ | Threshold key management, private voting |
| Homomorphic Encryption | Phase 4+ | Privacy-preserving community analytics |
| Futarchy | Phase 4+ | Prediction-market governance (experimental) |

### What Benten Should NOT Adopt

| Technology | Why Not |
|-----------|---------|
| Proof of Work | Energy waste, wrong topology (global adversarial network vs. cooperative federation) |
| Proof of Stake | Economic incentive model for open networks; Benten is small, trusted federations |
| secp256k1 (as primary) | Ed25519 is strictly superior for non-blockchain use. Support secp256k1 only for interop. |
| EVM | Global execution VM for shared blockchain state. Benten executes locally. |
| Sidechains | No main chain to have side chains for. |
| Solana BPF | Blockchain-specific runtime. Not applicable. |
| Arweave/Filecoin (as core) | Benten data is mutable and user-owned; permanent/incentivized storage is an optional integration |

### The Big Picture

Benten sits at a unique intersection in the decentralized technology landscape. It is NOT a blockchain -- it doesn't need global consensus, economic incentives for open participation, or trustless execution. It IS a decentralized application platform -- it needs user-owned identity, capability-based security, P2P sync, and community governance.

The technologies that matter most are:
1. **Identity**: DIDs + UCAN (already specified)
2. **Data**: CRDTs + Merkle DAGs + Content Addressing + HLCs (already specified)
3. **Networking**: libp2p + GossipSub + DHT (to be adopted)
4. **Governance**: Liquid Democracy + Quadratic Voting + Snapshot-style Voting (future phases)

The technologies that DON'T matter are the ones solving blockchain-specific problems: PoW, PoS, rollups, bridges, sidechains, gas metering.

Benten's innovation is that it provides the SAME capabilities (user sovereignty, censorship resistance, data portability, community governance) WITHOUT the blockchain overhead (global consensus, token economics, gas fees, mining/staking). The graph engine IS the decentralized runtime -- no blockchain required.

---

## Sources

- [W3C DID v1.1 Candidate Recommendation (March 2026)](https://www.w3.org/news/2026/w3c-invites-implementations-of-decentralized-identifiers-dids-v1-1/)
- [Decentralized Identity Enterprise Playbook 2026](https://securityboulevard.com/2026/03/decentralized-identity-and-verifiable-credentials-the-enterprise-playbook-2026/)
- [UCAN Specification](https://ucan.xyz/specification/)
- [UCAN Working Group](https://github.com/ucan-wg/spec)
- [libp2p 2025 Annual Report](https://libp2p.io/reports/annual-reports/2025/)
- [libp2p Specifications](https://github.com/libp2p/specs)
- [AT Protocol Documentation](https://docs.bsky.app/docs/advanced-guides/atproto)
- [AT Protocol: Usable Decentralized Social Media (Kleppmann)](https://bsky.social/about/bluesky-and-the-at-protocol-usable-decentralized-social-media-martin-kleppmann.pdf)
- [Local-First Software Development Patterns 2026](https://tech-champion.com/software-engineering/the-local-first-manifesto-why-the-cloud-is-losing-its-luster-in-2026/)
- [FOSDEM 2026 Local-First Track](https://fosdem.org/2026/schedule/track/local-first/)
- [CRDT Survey Part 3: Algorithmic Techniques](https://mattweidner.com/2023/09/26/crdt-survey-3.html)
- [IPLD Data Model](https://ipld.io/)
- [Helia IPFS TypeScript Implementation](https://github.com/ipfs/helia)
- [ZCAP-LD Specification](https://w3c-ccg.github.io/zcap-spec/)
- [Chainlink CCIP](https://chain.link/)
- [Move Language 2025 Update](https://aptoslabs.medium.com/move-in-2025-building-a-modern-smart-contract-language-391fc8ce0fe8)
- [Sui vs Aptos 2026](https://eng.ambcrypto.com/sui-vs-aptos-in-2026-who-is-actually-winning-the-move-developer-war/)
- [zk-SNARKs vs zk-STARKs Analysis](https://arxiv.org/html/2512.10020v1)
- [Proof of Personhood 2026](https://academy.exmon.pro/proof-of-personhood-2026-crypto-vs-deepfakes-ai-agents/)
- [DAO Governance and Fairness (Frontiers 2026)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2026.1840145/full)
- [Quadratic Voting and Information Aggregation (Management Science)](https://pubsonline.informs.org/doi/10.1287/mnsc.2024.08469)
- [Delegated Voting in DAOs (Frontiers 2025)](https://www.frontiersin.org/journals/blockchain/articles/10.3389/fbloc.2025.1598283/full)
- [Web3 Storage War 2026](https://adipek.com/articles/the-web3-storage-war-is-here-why-decentralized-file-systems-are-suddenly-everywhere-in-2026)
- [Layer 2 Scaling 2026 Guide](https://thelinuxcode.com/what-are-layer-2-solutions-in-blockchain-a-practical-2026-guide/)
- [CometBFT Documentation](https://docs.cometbft.com/main/spec/consensus/consensus)
- [Yggdrasil Network](https://yggdrasil-network.github.io/)
- [WebRTC P2P Communication 2026](https://antmedia.io/how-to-create-webrtc-peer-to-peer-communication/)
- [Verifiable Random Functions RFC 9381](https://datatracker.ietf.org/doc/rfc9381/)
- [Verifiable Credentials Working Group Charter 2026](https://w3c.github.io/vc-charter-2026/)
- [Ed25519 Deployment](https://ianix.com/pub/ed25519-deployment.html)
